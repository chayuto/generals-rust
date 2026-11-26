//! `PKT-363`: `RepairDockUpdate`.
//!
//! Packet: `packets/phase5/PKT-363_RepairDockUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt320_381_378_363_docks.rs`.

use super::dock_update::{
    dock_behavior_common, dock_module_data, DockUpdateModuleData, DockUpdateState,
};
use crate::arena::{ModuleContext, ObjectSlot};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::xfer_module_version;
use crate::modules::module::{BehaviorModule, DockFacet, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Bool, NameKeyType, ObjectId, Real, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

#[derive(Debug, Clone, PartialEq)]
pub struct RepairDockUpdateModuleData {
    pub dock: DockUpdateModuleData,
    pub frames_for_full_heal: Real,
}

dock_module_data!(RepairDockUpdateModuleData, dock);

impl RepairDockUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            dock: DockUpdateModuleData::new(tag_key, module_name_key),
            frames_for_full_heal: 1.0,
        }
    }
}

pub struct RepairDockUpdate {
    pub state: DockUpdateState,
    pub data: RepairDockUpdateModuleData,
    pub last_repair: ObjectId,
    pub health_to_add_per_frame: Real,
}

impl RepairDockUpdate {
    pub fn new(data: RepairDockUpdateModuleData) -> Self {
        Self {
            state: DockUpdateState::new(data.dock.clone()),
            data,
            last_repair: ObjectId::INVALID,
            health_to_add_per_frame: 0.0,
        }
    }
}

impl DockFacet for RepairDockUpdate {
    fn dock_state(&self) -> &DockUpdateState {
        &self.state
    }

    fn dock_state_mut(&mut self) -> &mut DockUpdateState {
        &mut self.state
    }

    /// One frame of repair. The rate is fixed when a repair starts and is kept
    /// until a docker is found at full health, so a docker that leaves early
    /// passes its rate to the next one.
    fn action(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        docker: ObjectSlot,
        drone: Option<ObjectSlot>,
    ) -> Bool {
        // The reference crashes on a docker without a body.
        let Some(docker_id) = sim
            .object(docker)
            .filter(|o| o.body_slot.is_some())
            .map(|o| o.id)
        else {
            return FALSE;
        };
        if self.last_repair == ObjectId::INVALID {
            self.last_repair = docker_id;
            self.health_to_add_per_frame = (sim.object_max_health(docker)
                - sim.object_health(docker))
                / self.data.frames_for_full_heal;
        }
        if sim.object_health(docker) >= sim.object_max_health(docker) {
            self.last_repair = ObjectId::INVALID;
            return FALSE;
        }
        let mut healing = DamageInfo::default();
        healing.input.amount = self.health_to_add_per_frame;
        healing.input.source_id = sim
            .object(ctx.owner_slot)
            .map_or(ObjectId::INVALID, |o| o.id);
        healing.input.damage_type = DamageType::Healing;
        healing.input.death_type = DeathType::NoneType;
        sim.attempt_healing(docker, &mut healing);
        if let Some(drone) = drone {
            healing.input.amount = sim.object_max_health(drone);
            sim.attempt_healing(drone, &mut healing);
        }
        TRUE
    }

    fn is_rally_point_after_dock_type(&self) -> Bool {
        TRUE
    }
}

impl UpdateFacet for RepairDockUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        self.state.update(sim, ctx)
    }
}

impl BehaviorModule for RepairDockUpdate {
    dock_behavior_common!(state);

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.state.xfer(xfer, sim)?;
        xfer.xfer_object_id(&mut self.last_repair)?;
        xfer.xfer_real(&mut self.health_to_add_per_frame)
    }
}

pub fn parse_repair_dock_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = RepairDockUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("TimeForFullHeal") {
            r.parse_duration_real(&mut data.frames_for_full_heal)
        } else {
            data.dock.parse_field(field, r)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_repair_dock_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<RepairDockUpdateModuleData>()
        .expect("RepairDockUpdate data type mismatch")
        .clone();
    Box::new(RepairDockUpdate::new(typed))
}
