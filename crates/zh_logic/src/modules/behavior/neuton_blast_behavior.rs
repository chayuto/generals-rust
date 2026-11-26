//! `PKT-281`: `NeutronBlastBehaviorModuleData` and `NeutronBlastBehavior`
//! (`NeutronBlastBehavior.h`, `NeutonBlastBehavior.cpp`; the manifest keeps the file spelling).
//!
//! Packet: `packets/phase5/PKT-281_NeutonBlastBehavior.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp53_c4.rs`.
//!
//! Not modelled: the off-map scan filter and the client terrain decal.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::contain::open_contain::kill_object;
use crate::modules::module::{
    BehaviorModule, CommandSourceType, DieFacet, ModuleData, UpdateFacet,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{
    Bool, Int, NameKeyType, Real, Relationship, UnsignedShort, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

const PLAYERMASK_ALL: UnsignedShort = 0xffff;

/// INI template data matching `NeutronBlastBehaviorModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct NeutronBlastBehaviorModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub blast_radius: Real,
    pub affect_airborne: Bool,
    pub affect_allies: Bool,
}

impl NeutronBlastBehaviorModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            blast_radius: 10.0,
            affect_airborne: TRUE,
            affect_allies: TRUE,
        }
    }
}

impl ModuleData for NeutronBlastBehaviorModuleData {
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
        FacetMask(0x003)
    }
}

/// Runtime instance of `NeutronBlastBehavior` (`PKT-281`).
pub struct NeutronBlastBehavior {
    pub data: NeutronBlastBehaviorModuleData,
    pub schedule: UpdateScheduleTracker,
}

impl NeutronBlastBehavior {
    pub fn new(data: NeutronBlastBehaviorModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
        }
    }

    /// `neutronBlastToObject`.
    fn neutron_blast_to_object(&self, sim: &mut dyn SimApi, owner: ObjectSlot, obj: ObjectSlot) {
        if obj == owner {
            return;
        }
        if self.data.affect_allies == FALSE
            && sim.player_relationship(owner, obj) == Relationship::Allies
        {
            return;
        }

        if sim.object_is_kind_of(obj, KindOfType::Infantry) != FALSE {
            kill_object(sim, obj, DamageType::Unresistable, DeathType::Normal);
        }

        if sim.object(obj).is_some_and(|o| o.contain_slot.is_some()) {
            sim.contain_kill_all(obj);
        }

        if sim.object_is_kind_of(obj, KindOfType::Vehicle) != FALSE
            && sim.object_is_kind_of(obj, KindOfType::Drone) == FALSE
        {
            if sim.object_is_kind_of(obj, KindOfType::CliffJumper) != FALSE {
                kill_object(sim, obj, DamageType::Unresistable, DeathType::Normal);
            } else {
                sim.set_disabled(obj, DisabledType::Unmanned);
                if sim.has_ai(obj) != FALSE {
                    sim.ai_idle(obj, CommandSourceType::FromAi);
                }
                sim.deselect_object(obj, PLAYERMASK_ALL, TRUE);
                let neutral = sim.neutral_team_id();
                sim.set_object_team(obj, neutral);
            }
        }
    }
}

impl UpdateFacet for NeutronBlastBehavior {
    fn update(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) -> UpdateSleepTime {
        UpdateSleepTime::FOREVER
    }
}

impl DieFacet for NeutronBlastBehavior {
    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, _dmg: &DamageInfo) {
        let owner = ctx.owner_slot;
        let Some(center) = sim.object(owner).map(|o| *o.thing.get_position()) else {
            return;
        };

        // The scan filters run when the iterator is built, before anything is hurt.
        let targets: Vec<ObjectSlot> = sim
            .find_objects_in_range(&center, self.data.blast_radius)
            .into_iter()
            .filter(|&slot| {
                sim.object(slot)
                    .is_some_and(|o| o.is_effectively_dead() == FALSE)
            })
            .collect();

        for obj in targets {
            let airborne = sim.object(obj).is_some_and(|o| {
                o.status.test(ObjectStatusType::AirborneTarget as Int) != FALSE
            });
            if self.data.affect_airborne != FALSE
                || (sim.object_is_kind_of(obj, KindOfType::Aircraft) == FALSE && !airborne)
            {
                self.neutron_blast_to_object(sim, owner, obj);
            }
        }
    }
}

impl BehaviorModule for NeutronBlastBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)
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

    fn as_die(&self) -> Option<&dyn DieFacet> {
        Some(self)
    }

    fn as_die_mut(&mut self) -> Option<&mut dyn DieFacet> {
        Some(self)
    }
}

pub fn parse_neutron_blast_behavior_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = NeutronBlastBehaviorModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("BlastRadius") {
            r.parse_real(&mut data.blast_radius)
        } else if field.eq_ignore_ascii_case("AffectAirborne") {
            r.parse_bool(&mut data.affect_airborne)
        } else if field.eq_ignore_ascii_case("AffectAllies") {
            r.parse_bool(&mut data.affect_allies)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_neutron_blast_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<NeutronBlastBehaviorModuleData>()
        .expect("NeutronBlastBehavior data type mismatch")
        .clone();
    Box::new(NeutronBlastBehavior::new(typed))
}
