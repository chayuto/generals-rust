//! `PKT-357`: `QueueProductionExitUpdateModuleData` and `QueueProductionExitUpdate`
//! (`QueueProductionExitUpdate.h`/`.cpp`).
//!
//! Packet: `packets/phase5/PKT-357_QueueProductionExitUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt315_379_357_367_exits.rs`.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot, ThingTemplateId};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{
    BehaviorModule, CommandSourceType, ExitDoorType, ExitFacet, ModuleData, UpdateFacet,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::update::default_production_exit_update::{
    exit_position, follow_exit_path, natural_rally_point, producer, push_ground_rally_point,
};
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::{Coord3D, Matrix3D};
use zh_common::types::{Bool, NameKeyType, Real, UnsignedInt, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

const STARTING_PITCH_COEFF: Real = 0.04;

/// INI template data matching `QueueProductionExitUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct QueueProductionExitUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub unit_create_point: Coord3D,
    pub natural_rally_point: Coord3D,
    /// `ExitDelay`, in frames.
    pub exit_delay: UnsignedInt,
    pub allow_airborne_creation: Bool,
    pub initial_burst: UnsignedInt,
}

impl QueueProductionExitUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            unit_create_point: Coord3D::default(),
            natural_rally_point: Coord3D::default(),
            exit_delay: 0,
            allow_airborne_creation: FALSE,
            initial_burst: 0,
        }
    }
}

impl ModuleData for QueueProductionExitUpdateModuleData {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask(0x001 | FacetMask::EXIT_INTERFACE.0)
    }
}

/// Runtime instance of `QueueProductionExitUpdate`.
pub struct QueueProductionExitUpdate {
    pub data: QueueProductionExitUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub current_delay: UnsignedInt,
    pub rally_point: Coord3D,
    pub rally_point_exists: Bool,
    /// Never written after construction, but saved.
    pub creation_clear_distance: Real,
    pub current_burst_count: UnsignedInt,
}

impl QueueProductionExitUpdate {
    pub fn new(data: QueueProductionExitUpdateModuleData) -> Self {
        let current_burst_count = data.initial_burst;
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            current_delay: 0,
            rally_point: Coord3D::default(),
            rally_point_exists: FALSE,
            creation_clear_distance: 0.0,
            current_burst_count,
        }
    }

    fn is_free_to_exit(&self) -> bool {
        self.current_burst_count > 0 || self.current_delay == 0
    }

    fn start_delay(&mut self) {
        self.current_delay = self.data.exit_delay;
        if self.current_burst_count != 0 {
            self.current_burst_count -= 1;
        }
    }

    /// A unit created in mid-air inherits the producer's momentum.
    fn throw_new_object(sim: &mut dyn SimApi, owner: ObjectSlot, new_obj: ObjectSlot) {
        if sim.has_physics(owner) == FALSE || sim.has_physics(new_obj) == FALSE {
            return;
        }
        let mut force = sim.physics_velocity(owner);
        let mass = sim.physics_mass(new_obj);
        force.x *= mass;
        force.y *= mass;
        force.z *= mass;
        sim.physics_apply_motive_force(new_obj, &force);
        let pitch_rate = sim.physics_center_of_mass_offset(new_obj) * STARTING_PITCH_COEFF;
        sim.physics_set_pitch_rate(new_obj, pitch_rate);
    }
}

impl ExitFacet for QueueProductionExitUpdate {
    fn reserve_door_for_exit(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _tmpl_id: ThingTemplateId,
        _specific_obj: Option<ObjectSlot>,
    ) -> ExitDoorType {
        if self.is_free_to_exit() {
            ExitDoorType::Door1
        } else {
            ExitDoorType::DoorNoneAvailable
        }
    }

    fn exit_object_via_door(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        new_obj: ObjectSlot,
        _exit_door: ExitDoorType,
    ) {
        let Some(me) = producer(sim, ctx.owner_slot) else {
            return;
        };
        let mut loc = Matrix3D::transform_vector(&me.transform, &self.data.unit_create_point);
        let ground = sim.terrain().get_ground_height(loc.x, loc.y, None);
        // Stays set when the point is then dropped to the ground.
        let creation_in_air = loc.z != ground;
        if creation_in_air && self.data.allow_airborne_creation == FALSE {
            loc.z = ground;
        }
        sim.set_object_position(new_obj, &loc);
        sim.set_object_orientation(new_obj, me.angle);
        if creation_in_air {
            Self::throw_new_object(sim, ctx.owner_slot, new_obj);
        }
        sim.pathfinder_add_object(new_obj);

        let mut tmp = Coord3D::default();
        natural_rally_point(
            sim,
            ctx.owner_slot,
            &self.data.natural_rally_point,
            &mut tmp,
            TRUE,
        );
        sim.pathfinder_snap_position(new_obj, &mut tmp);
        let mut exit_path = vec![tmp];
        if self.rally_point_exists != FALSE {
            push_ground_rally_point(sim, new_obj, self.rally_point, &mut exit_path);
        } else {
            exit_path.push(tmp);
        }
        follow_exit_path(sim, new_obj, &exit_path, ctx.owner_slot);
        self.start_delay();
    }

    fn exit_object_by_budding(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        new_obj: ObjectSlot,
        bud_host: Option<ObjectSlot>,
    ) {
        let source = bud_host.unwrap_or(ctx.owner_slot);
        if let Some((pos, angle, layer)) = sim
            .object(source)
            .map(|o| (*o.thing.get_position(), o.thing.get_orientation(), o.layer))
        {
            sim.set_object_position(new_obj, &pos);
            sim.set_object_orientation(new_obj, angle);
            // Only a bud host passes its layer on.
            if bud_host.is_some() {
                sim.set_object_layer(new_obj, layer);
            }
        }
        if sim.has_ai(new_obj) != FALSE {
            if let Some(pos) = sim.object(new_obj).map(|o| *o.thing.get_position()) {
                sim.ai_move_to_position(new_obj, &pos, CommandSourceType::FromAi);
            }
        }
        self.start_delay();
    }

    fn set_rally_point(&mut self, pos: &Coord3D) {
        self.rally_point = *pos;
        self.rally_point_exists = TRUE;
    }

    fn get_rally_point(&self) -> Option<&Coord3D> {
        (self.rally_point_exists != FALSE).then_some(&self.rally_point)
    }

    fn get_natural_rally_point(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        rally_point: &mut Coord3D,
        offset: Bool,
    ) -> Bool {
        natural_rally_point(
            sim,
            ctx.owner_slot,
            &self.data.natural_rally_point,
            rally_point,
            offset,
        )
    }

    fn get_exit_position(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        exit_pos: &mut Coord3D,
    ) -> Bool {
        exit_position(sim, ctx.owner_slot, &self.data.unit_create_point, exit_pos)
    }
}

impl UpdateFacet for QueueProductionExitUpdate {
    fn update(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) -> UpdateSleepTime {
        if self.is_free_to_exit() {
            // A running burst also cancels the delay.
            self.current_delay = 0;
        } else {
            self.current_delay -= 1;
        }
        UpdateSleepTime::NONE
    }
}

impl BehaviorModule for QueueProductionExitUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, _sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_unsigned_int(&mut self.current_delay)?;
        xfer.xfer_coord3d(&mut self.rally_point)?;
        xfer.xfer_bool(&mut self.rally_point_exists)?;
        xfer.xfer_real(&mut self.creation_clear_distance)?;
        xfer.xfer_unsigned_int(&mut self.current_burst_count)
    }

    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.load_post_process(sim, ctx);
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }

    fn as_update(&self) -> Option<&dyn UpdateFacet> {
        Some(self)
    }

    fn as_update_mut(&mut self) -> Option<&mut dyn UpdateFacet> {
        Some(self)
    }

    fn as_exit(&self) -> Option<&dyn ExitFacet> {
        Some(self)
    }

    fn as_exit_mut(&mut self) -> Option<&mut dyn ExitFacet> {
        Some(self)
    }
}

pub fn parse_queue_production_exit_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = QueueProductionExitUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("UnitCreatePoint") {
            r.parse_coord3d(&mut data.unit_create_point)
        } else if field.eq_ignore_ascii_case("NaturalRallyPoint") {
            r.parse_coord3d(&mut data.natural_rally_point)
        } else if field.eq_ignore_ascii_case("ExitDelay") {
            r.parse_duration_unsigned_int(&mut data.exit_delay)
        } else if field.eq_ignore_ascii_case("AllowAirborneCreation") {
            r.parse_bool(&mut data.allow_airborne_creation)
        } else if field.eq_ignore_ascii_case("InitialBurst") {
            r.parse_unsigned_int(&mut data.initial_burst)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_queue_production_exit_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<QueueProductionExitUpdateModuleData>()
        .expect("QueueProductionExitUpdate data type mismatch")
        .clone();
    Box::new(QueueProductionExitUpdate::new(typed))
}
