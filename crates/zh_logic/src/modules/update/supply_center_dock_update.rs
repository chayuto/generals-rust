//! `PKT-378`: `SupplyCenterDockUpdate`.
//!
//! Packet: `packets/phase5/PKT-378_SupplyCenterDockUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt320_381_378_363_docks.rs`.
//!

use super::dock_update::{
    dock_behavior_common, dock_module_data, DockUpdateModuleData, DockUpdateState,
};
use crate::arena::{ModuleContext, ObjectSlot};
use crate::modules::behavior::behavior_module::xfer_module_version;
use crate::modules::module::{BehaviorModule, DockFacet, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Bool, Int, NameKeyType, UnsignedInt, UpdateSleepTime, FALSE};
use zh_common::xfer::{Xfer, XferError};

#[derive(Debug, Clone, PartialEq)]
pub struct SupplyCenterDockUpdateModuleData {
    pub dock: DockUpdateModuleData,
    pub grant_temporary_stealth_frames: UnsignedInt,
}

dock_module_data!(SupplyCenterDockUpdateModuleData, dock);

impl SupplyCenterDockUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            dock: DockUpdateModuleData::new(tag_key, module_name_key),
            grant_temporary_stealth_frames: 0,
        }
    }
}

pub struct SupplyCenterDockUpdate {
    pub state: DockUpdateState,
    pub data: SupplyCenterDockUpdateModuleData,
}

impl SupplyCenterDockUpdate {
    pub fn new(data: SupplyCenterDockUpdateModuleData) -> Self {
        Self {
            state: DockUpdateState::new(data.dock.clone()),
            data,
        }
    }

    /// The end of `action`: a stealthed center lends its stealth to the docker.
    fn grant_stealth(&self, sim: &mut dyn SimApi, owner: ObjectSlot, docker: ObjectSlot) {
        let has = |sim: &dyn SimApi, o: ObjectSlot, s: ObjectStatusType| {
            sim.object(o)
                .is_some_and(|o| o.status.test(s.as_i32()) != FALSE)
        };
        if self.data.grant_temporary_stealth_frames == 0
            || !has(sim, owner, ObjectStatusType::Stealthed)
            || sim.object(docker).is_none_or(|o| o.stealth_slot.is_none())
        {
            return;
        }
        if sim.stealth_is_temporary_grant(docker) != FALSE
            || !has(sim, docker, ObjectStatusType::CanStealth)
        {
            sim.receive_temporary_stealth_grant(docker, self.data.grant_temporary_stealth_frames);
        }
    }
}

impl DockFacet for SupplyCenterDockUpdate {
    fn dock_state(&self) -> &DockUpdateState {
        &self.state
    }

    fn dock_state_mut(&mut self) -> &mut DockUpdateState {
        &mut self.state
    }

    /// Takes every box the docker carries and pays for them. Always `FALSE`.
    fn action(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        docker: ObjectSlot,
        _drone: Option<ObjectSlot>,
    ) -> Bool {
        let owner = ctx.owner_slot;
        let Some(team) = sim.object(owner).map(|o| o.team_id) else {
            return FALSE;
        };
        let box_value = sim.rules().global_data.base_value_per_supply_box as UnsignedInt;
        let mut value: UnsignedInt = 0;
        sim.with_supply_truck(docker, &mut |truck, sim, ctx| {
            let boost = truck.upgraded_supply_boost(sim, ctx);
            // The default build scales the boost by how full the truck is.
            #[cfg(not(feature = "retail-crc"))]
            let boost: Int = match truck.max_boxes() {
                max if max > 0 => boost.wrapping_mul(truck.number_boxes()) / max,
                _ => 0,
            };
            value = boost as UnsignedInt;
            while truck.lose_one_box(sim, ctx) != FALSE {
                value = value.wrapping_add(box_value);
            }
        });
        if value > 0 {
            let _ = sim.add_player_money(team, value as Int);
            sim.add_money_earned(team, value as Int);
            self.grant_stealth(sim, owner, docker);
        }
        // The cash text over the docker, at ground height.
        if value > 0 && sim.object_is_logically_visible(owner) != FALSE {
            if let Some(mut pos) = sim.object(docker).map(|o| *o.thing.get_position()) {
                pos.z = sim.terrain().get_ground_height(pos.x, pos.y, None);
                sim.add_floating_money_text(owner, &pos, value as Int);
            }
        }
        FALSE
    }
}

impl UpdateFacet for SupplyCenterDockUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        self.state.update(sim, ctx)
    }
}

impl BehaviorModule for SupplyCenterDockUpdate {
    dock_behavior_common!(state);

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.state.xfer(xfer, sim)
    }
}

pub fn parse_supply_center_dock_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = SupplyCenterDockUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("GrantTemporaryStealth") {
            r.parse_duration_unsigned_int(&mut data.grant_temporary_stealth_frames)
        } else {
            data.dock.parse_field(field, r)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_supply_center_dock_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<SupplyCenterDockUpdateModuleData>()
        .expect("SupplyCenterDockUpdate data type mismatch")
        .clone();
    Box::new(SupplyCenterDockUpdate::new(typed))
}
