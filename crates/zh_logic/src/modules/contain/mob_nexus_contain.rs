//! `PKT-280`: `MobNexusContain` (`MobNexusContain.h`, `MobNexusContain.cpp`).
//!
//! Packet: `packets/phase5/PKT-280_MobNexusContain.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp53_b.rs`.
//!
//! Not modelled: `ExitBone` repositioning (headless art has no bones).

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::behavior::behavior_module::xfer_module_version;
use crate::modules::contain::open_contain::{
    add_to_contain, impl_contain_module, scatter_to_nearby_position, update_open_contain,
    ContainBehavior, OpenContainCore, OpenContainModuleData,
};
use crate::modules::contain::transport_contain::{
    infantry_kind_of_mask, inherit_container_motion, is_using_airborne_locomotor,
    regenerate_riders, transport_slot_count, unwrap_special_zero_slot_rider, InitialPayload,
};
use crate::modules::module::{BehaviorModule, ExitDoorType, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::KindOfType;
use crate::rules::model_state::ModelConditionFlagType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{
    Bool, Int, NameKeyType, Real, Relationship, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

/// INI template data matching `MobNexusContainModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct MobNexusContainModuleData {
    pub open: OpenContainModuleData,
    pub slot_capacity: Int,
    pub exit_pitch_rate: Real,
    pub scatter_nearby_on_exit: Bool,
    pub orient_like_container_on_exit: Bool,
    pub keep_container_velocity_on_exit: Bool,
    pub exit_bone: String,
    pub initial_payload: InitialPayload,
    pub health_regen: Real,
}

impl MobNexusContainModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        let mut open = OpenContainModuleData::new(tag_key, module_name_key);
        open.allow_inside_kind_of = infantry_kind_of_mask();
        Self {
            open,
            slot_capacity: 0,
            exit_pitch_rate: 0.0,
            scatter_nearby_on_exit: TRUE,
            orient_like_container_on_exit: FALSE,
            keep_container_velocity_on_exit: FALSE,
            exit_bone: String::new(),
            initial_payload: InitialPayload::default(),
            health_regen: 0.0,
        }
    }

    pub fn parse_field(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        parse_ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        let is = |name: &str| field.eq_ignore_ascii_case(name);
        let result = if is("Slots") {
            reader.parse_int(&mut self.slot_capacity)
        } else if is("ScatterNearbyOnExit") {
            reader.parse_bool(&mut self.scatter_nearby_on_exit)
        } else if is("OrientLikeContainerOnExit") {
            reader.parse_bool(&mut self.orient_like_container_on_exit)
        } else if is("KeepContainerVelocityOnExit") {
            reader.parse_bool(&mut self.keep_container_velocity_on_exit)
        } else if is("ExitBone") {
            reader.parse_ascii_string(&mut self.exit_bone)
        } else if is("ExitPitchRate") {
            reader.parse_angular_velocity_real(&mut self.exit_pitch_rate)
        } else if is("InitialPayload") {
            self.initial_payload.parse(reader)
        } else if is("HealthRegen%PerSec") {
            reader.parse_real(&mut self.health_regen)
        } else {
            return self.open.parse_field(field, reader, parse_ctx);
        };
        Some(result)
    }
}

impl ModuleData for MobNexusContainModuleData {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.open.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.open.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0 | FacetMask::TRANSPORT_PASSENGER.0)
    }
}

/// Runtime instance of `MobNexusContain` (`PKT-280`).
pub struct MobNexusContain {
    pub data: MobNexusContainModuleData,
    pub core: OpenContainCore,
    pub extra_slots_in_use: Int,
}

impl MobNexusContain {
    pub fn new(data: MobNexusContainModuleData) -> Self {
        let core = OpenContainCore::new(data.open.passengers_allowed_to_fire);
        Self {
            data,
            core,
            extra_slots_in_use: 0,
        }
    }
}

impl ContainBehavior for MobNexusContain {
    fn open_data(&self) -> &OpenContainModuleData {
        &self.data.open
    }
    fn open_core(&self) -> &OpenContainCore {
        &self.core
    }
    fn open_core_mut(&mut self) -> &mut OpenContainCore {
        &mut self.core
    }
    fn contain_max_value(&self) -> Int {
        self.data.slot_capacity
    }

    fn xfer_module(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.core.xfer(xfer, sim)?;
        xfer.xfer_int(&mut self.extra_slots_in_use)
    }

    /// `MobNexusContain::onObjectCreated`: spawn the initial payload.
    fn object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.core.schedule.bind(ctx);
        let sidecar = self
            .core
            .sidecar(self.data.slot_capacity, TRUE, FALSE);
        sim.set_module_sidecar(ctx.mod_slot, sidecar);

        let payload = self.data.initial_payload.clone();
        let template = sim.rules().thing_factory.find_template_id(&payload.name);
        let owner_team = sim.object(ctx.owner_slot).map(|o| o.team_id);
        let (Some(template), Some(owner_team)) = (template, owner_team) else {
            return;
        };
        let team = sim.player_default_team(owner_team);
        for _ in 0..payload.count {
            let rider = sim.spawn_object(template, team, Coord3D::default(), 0.0);
            if self.is_valid_container(sim, ctx, rider, TRUE) != FALSE {
                add_to_contain(self, sim, ctx, rider);
            }
        }
    }

    fn is_valid_container(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        rider: ObjectSlot,
        check_capacity: Bool,
    ) -> Bool {
        let rider = unwrap_special_zero_slot_rider(sim, rider);
        if self
            .core
            .is_valid_container_for(&self.data.open, sim, ctx, rider)
            == FALSE
        {
            return FALSE;
        }
        // The rider must consider the nexus an ally (not the other way round).
        if sim.player_relationship(rider, ctx.owner_slot) != Relationship::Allies {
            return FALSE;
        }
        let slots = transport_slot_count(sim, rider);
        if slots == 0 {
            return FALSE;
        }
        if check_capacity != FALSE {
            let used = self.extra_slots_in_use + self.core.contain_list_size as Int;
            Bool::from(used + slots <= self.data.slot_capacity)
        } else {
            TRUE
        }
    }

    fn contained(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, rider: ObjectSlot) {
        sim.set_disabled(rider, DisabledType::Held);
        self.extra_slots_in_use += transport_slot_count(sim, rider) - 1;

        if self.core.contain_list_size == 1 && sim.object_has_drawable(ctx.owner_slot) != FALSE {
            sim.set_model_condition_state(ctx.owner_slot, ModelConditionFlagType::Loaded);
        }
    }

    fn removing(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, rider: ObjectSlot) {
        let owner = ctx.owner_slot;
        sim.clear_disabled(rider, DisabledType::Held);

        inherit_container_motion(
            sim,
            owner,
            rider,
            self.data.orient_like_container_on_exit,
            self.data.keep_container_velocity_on_exit,
            self.data.exit_pitch_rate,
        );
        if self.data.scatter_nearby_on_exit != FALSE {
            scatter_to_nearby_position(sim, owner, rider);
        }
        self.extra_slots_in_use -= transport_slot_count(sim, rider) - 1;

        if self.core.contain_list_size == 0 && sim.object_has_drawable(owner) != FALSE {
            sim.clear_model_condition_state(owner, ModelConditionFlagType::Loaded);
        }
        if sim.object_is_above_terrain(owner) != FALSE && sim.has_physics(rider) != FALSE {
            sim.set_physics_allow_to_fall(rider, TRUE);
        }
    }

    fn update_module(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        if self.data.health_regen != 0.0 {
            let riders = self.core.contain_list.clone();
            regenerate_riders(sim, ctx.owner_slot, &riders, self.data.health_regen);
        }
        update_open_contain(self, sim, ctx)
    }

    fn reserve_door(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        specific_obj: Option<ObjectSlot>,
    ) -> ExitDoorType {
        let Some(rider) = specific_obj else {
            return ExitDoorType::Door1;
        };
        let me = ctx.owner_slot;
        // The C++ asks the nexus AI about the nexus itself.
        if sim.has_ai(me) != FALSE && sim.ai_free_to_exit(me) == FALSE {
            return ExitDoorType::DoorNoneAvailable;
        }
        if is_using_airborne_locomotor(sim, me) {
            return ExitDoorType::Door1;
        }
        if sim.has_ai(rider) == FALSE {
            return ExitDoorType::DoorNoneAvailable;
        }
        let Some((layer, pos)) = sim.object(me).map(|o| (o.layer, *o.thing.get_position())) else {
            return ExitDoorType::DoorNoneAvailable;
        };
        if sim.valid_movement_terrain(layer, rider, &pos) == FALSE {
            return ExitDoorType::DoorNoneAvailable;
        }
        ExitDoorType::Door1
    }

    fn is_transport_passenger(&self) -> Bool {
        TRUE
    }

    /// `MobNexusContain::tryToEvacuate`.
    fn try_evacuate(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        expose_stealthed: Bool,
    ) -> Bool {
        let mut exited_anyone = FALSE;
        for id in self.core.contain_list.clone() {
            let Some(obj) = sim.find_object(id) else {
                continue;
            };
            let door = self.reserve_door(sim, ctx, Some(obj));
            if door == ExitDoorType::DoorNoneAvailable {
                continue;
            }
            self.exit_via_door(sim, ctx, obj, door);
            exited_anyone = TRUE;

            if expose_stealthed != FALSE
                && sim.object_is_kind_of(obj, KindOfType::StealthGarrison) != FALSE
            {
                sim.mark_stealth_detected(obj, 0);
            }
        }
        exited_anyone
    }
}

impl_contain_module!(MobNexusContain);

pub fn parse_mob_nexus_contain_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = MobNexusContainModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        data.parse_field(field, r, parse_ctx)
            .unwrap_or(Err(IniError::UnknownToken))
    })?;
    Ok(Box::new(data))
}

pub fn create_mob_nexus_contain(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<MobNexusContainModuleData>()
        .expect("MobNexusContain data type mismatch")
        .clone();
    Box::new(MobNexusContain::new(typed))
}
