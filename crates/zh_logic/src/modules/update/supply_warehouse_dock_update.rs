//! `PKT-381`: `SupplyWarehouseDockUpdate`.
//!
//! Packet: `packets/phase5/PKT-381_SupplyWarehouseDockUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt320_381_378_363_docks.rs`.
//!
//! `Drawable::updateDrawableSupplyStatus` (`onObjectCreated`, `loadPostProcess`,
//! `setCashValue`, `action`) only changes how the model looks, so it is left out.

use super::dock_update::{
    dock_behavior_common, dock_module_data, DockUpdateModuleData, DockUpdateState,
};
use crate::arena::{ModuleContext, ObjectSlot};
use crate::modules::behavior::behavior_module::xfer_module_version;
use crate::modules::contain::open_contain::kill_object;
use crate::modules::contain::transport_contain::is_using_airborne_locomotor;
use crate::modules::module::{BehaviorModule, DockFacet, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::CommandSourceType;
use crate::object::weapon::PATHFIND_CELL_SIZE_F;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader};
use zh_common::math::{gm_ceilf, gm_sqrtf};
use zh_common::types::{Bool, Int, NameKeyType, ObjectId, Real, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

#[derive(Debug, Clone, PartialEq)]
pub struct SupplyWarehouseDockUpdateModuleData {
    pub dock: DockUpdateModuleData,
    pub starting_boxes: Int,
    pub delete_when_empty: Bool,
}

dock_module_data!(SupplyWarehouseDockUpdateModuleData, dock);

impl SupplyWarehouseDockUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            dock: DockUpdateModuleData::new(tag_key, module_name_key),
            starting_boxes: 1,
            delete_when_empty: FALSE,
        }
    }
}

pub struct SupplyWarehouseDockUpdate {
    pub state: DockUpdateState,
    pub data: SupplyWarehouseDockUpdateModuleData,
    pub boxes_stored: Int,
}

impl SupplyWarehouseDockUpdate {
    pub fn new(data: SupplyWarehouseDockUpdateModuleData) -> Self {
        Self {
            state: DockUpdateState::new(data.dock.clone()),
            boxes_stored: data.starting_boxes,
            data,
        }
    }

    /// `setCashValue`.
    pub fn set_cash_value(&mut self, sim: &dyn SimView, cash_value: Int) {
        let per_box = sim.rules().global_data.base_value_per_supply_box as Real;
        self.boxes_stored = gm_ceilf(cash_value as Real / per_box) as Int;
    }

    /// The failing start of `action`: the docker is too far away, so it is jostled.
    fn too_far(sim: &mut dyn SimApi, owner: ObjectSlot, docker: ObjectSlot) -> bool {
        let (Some(a), Some(b)) = (sim.object(docker), sim.object(owner)) else {
            return false;
        };
        let radius = a.geometry.get_bounding_circle_radius();
        let close_enough_sqr = (radius * 2.0) * (radius * 2.0);
        // `getDistanceSquared(docker, dock, FROM_BOUNDINGSPHERE_2D)`.
        let from = *a.thing.get_position();
        let to = *b.thing.get_position();
        let (dx, dy) = (to.x - from.x, to.y - from.y);
        let mut dist_sqr = dx * dx + dy * dy;
        let total = radius + b.geometry.get_bounding_circle_radius();
        if total > 0.0 {
            let shrunk = gm_sqrtf(dist_sqr) - total;
            dist_sqr = if shrunk <= 0.0 { 0.0 } else { shrunk * shrunk };
        }
        if dist_sqr <= close_enough_sqr {
            return false;
        }
        let mut pos = from;
        // `GameLogicRandomValue` is the integer draw, so the offset is a whole number.
        let range = (0.4_f64 * f64::from(PATHFIND_CELL_SIZE_F)) as Real;
        pos.x += sim.rng_logic_int((-range) as Int, range as Int) as Real;
        pos.y += sim.rng_logic_int((-range) as Int, range as Int) as Real;
        sim.set_object_position(docker, &pos);
        true
    }
}

impl DockFacet for SupplyWarehouseDockUpdate {
    fn dock_state(&self) -> &DockUpdateState {
        &self.state
    }

    fn dock_state_mut(&mut self) -> &mut DockUpdateState {
        &mut self.state
    }

    /// Hands the docker one box.
    fn action(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        docker: ObjectSlot,
        _drone: Option<ObjectSlot>,
    ) -> Bool {
        if self.boxes_stored == 0 || Self::too_far(sim, ctx.owner_slot, docker) {
            return FALSE;
        }
        self.boxes_stored = self.boxes_stored.wrapping_sub(1);
        let remaining = self.boxes_stored;
        let mut gained = FALSE;
        // The reference crashes on a docker without an AI; here it gains nothing.
        sim.with_supply_truck(docker, &mut |truck, sim, ctx| {
            gained = truck.gain_one_box(sim, ctx, remaining);
        });
        if gained == FALSE {
            self.boxes_stored = self.boxes_stored.wrapping_add(1);
            return FALSE;
        }
        if self.boxes_stored == 0 && self.data.delete_when_empty != FALSE {
            sim.destroy_object(ctx.owner_slot);
            return FALSE;
        }
        TRUE
    }

    /// A crippled warehouse kills the docker inside it and turns away the one on
    /// its way in.
    fn set_dock_crippled(&mut self, sim: &mut dyn SimApi, _ctx: ModuleContext, crippled: Bool) {
        if crippled != FALSE && self.state.active_docker != ObjectId::INVALID {
            if let Some(victim) = sim.find_object(self.state.active_docker) {
                if self.state.docker_inside != FALSE {
                    if !is_using_airborne_locomotor(sim, victim) {
                        kill_object(sim, victim, DamageType::Unresistable, DeathType::Normal);
                    }
                } else if sim.with_supply_truck(victim, &mut |_, _, _| {}) != FALSE {
                    sim.ai_idle(victim, CommandSourceType::FromAi);
                    sim.with_supply_truck(victim, &mut |truck, _, _| {
                        truck.set_force_wanting_state(TRUE);
                    });
                }
            }
        }
        self.state.dock_crippled = crippled;
    }
}

impl UpdateFacet for SupplyWarehouseDockUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        self.state.update(sim, ctx)
    }
}

impl BehaviorModule for SupplyWarehouseDockUpdate {
    dock_behavior_common!(state);

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.state.xfer(xfer, sim)?;
        xfer.xfer_int(&mut self.boxes_stored)
    }
}

pub fn parse_supply_warehouse_dock_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = SupplyWarehouseDockUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("StartingBoxes") {
            r.parse_int(&mut data.starting_boxes)
        } else if field.eq_ignore_ascii_case("DeleteWhenEmpty") {
            r.parse_bool(&mut data.delete_when_empty)
        } else {
            data.dock.parse_field(field, r)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_supply_warehouse_dock_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<SupplyWarehouseDockUpdateModuleData>()
        .expect("SupplyWarehouseDockUpdate data type mismatch")
        .clone();
    Box::new(SupplyWarehouseDockUpdate::new(typed))
}
