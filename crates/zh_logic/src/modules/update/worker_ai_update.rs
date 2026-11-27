//! `PKT-389`: `WorkerAIUpdate` (AI plan slice S16): construction, the frame
//! update, the machine choosing between the dozer and the supply truck behaviour,
//! orders, `xfer` and the module data.
//!
//! Packet doc: `packets/phase5/PKT-389_WorkerAIUpdate.md`. Golden tests:
//! `crates/zh_logic/tests/golden_s16_dozer_worker.rs` (construction) and
//! `golden_s16_dozer_worker_frames.rs` (frames and orders).
//!
//! The module is an `AIUpdateInterface` with a [`DozerBrain`], whose behaviour is
//! in `dozer_ai_update.rs`, a supply truck machine and a machine choosing between
//! the two.
//!
//! The supply half is not ported (slice S15): the supply truck machine stays in
//! the busy state it is built in, so a worker that is sent to a dock leaves the
//! dozer behaviour and comes back when its AI leaves the dock state, and boxes,
//! `exitingSupplyTruckState` from the outside and the forced states have no
//! caller.
//!
//! The reference's `onExit` of the two states of the choosing machine takes no
//! argument and so overrides nothing: neither brain is ever reset by a switch.

use std::any::Any;
use std::sync::Arc;

use crate::arena::ModuleContext;
use crate::modules::module::{BehaviorModule, CommandSourceType, ModuleData};
use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{audio, duration, duration_real, int, percent, real};
use crate::modules::update::ai_update::{
    self, AiCommandParms, AiCommandType, AiUpdate, AiUpdateModuleData, DerivedAi,
};
use crate::modules::update::dozer_ai_update::{
    self, xfer_machine, xfer_version_one, DozerBrain, DOZER_TASK_INVALID,
};
use crate::rules::combat_types::WeaponSetType;
use crate::sim::SimApi;
use zh_common::math::Coord3D;
use zh_common::types::{
    Bool, Int, ObjectId, Real, UnsignedInt, UpdateSleepTime, XferVersion, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError, XferMode};

module_data! {
    /// `WorkerAIUpdateModuleData`.
    WorkerAIUpdateModuleData = "WorkerAIUpdate";
    part ai: AiUpdateModuleData = AiUpdateModuleData::new;
    "MaxBoxes" => max_boxes: Int = 0, int;
    "RepairHealthPercentPerSecond" =>
        repair_health_percent_per_second: Real = 0.0, percent;
    "BoredTime" => bored_time: Real = 0.0, duration_real;
    "BoredRange" => bored_range: Real = 0.0, real;
    "SupplyCenterActionDelay" => center_delay: UnsignedInt = 0, duration;
    "SupplyWarehouseActionDelay" => warehouse_delay: UnsignedInt = 0, duration;
    "SupplyWarehouseScanDistance" => warehouse_scan_distance: Real = 100.0, real;
    "SuppliesDepletedVoice" => supplies_depleted_voice: String = String::new(), audio;
    "UpgradedSupplyBoost" => upgraded_supply_boost: Int = 0, int;
}

/// `ST_BUSY`, the first state the supply truck machine defines.
const ST_BUSY: UnsignedInt = 1;
/// `AS_DOZER`.
pub const AS_DOZER: UnsignedInt = 0;
/// `AS_SUPPLY_TRUCK`.
pub const AS_SUPPLY_TRUCK: UnsignedInt = 1;

/// The members `WorkerAIUpdate` adds to its base.
#[derive(Debug, Clone)]
pub struct WorkerAi {
    pub data: Arc<WorkerAIUpdateModuleData>,
    pub brain: DozerBrain,
    /// The constructor never sets it; the module's memory starts zeroed.
    pub preferred_dock: ObjectId,
    pub number_boxes: Int,
    pub force_pending: Bool,
    /// The current state of `WorkerStateMachine`.
    pub state: UnsignedInt,
}

impl WorkerAi {
    /// What `WorkerAIUpdate::xfer` writes after the base class. The states of the
    /// two machines here write nothing, and the machines' `crc` is empty.
    pub fn xfer(&mut self, xfer: &mut dyn Xfer, version: XferVersion) -> Result<(), XferError> {
        let crc = xfer.mode() == XferMode::Crc;
        let no_goal = (ObjectId::INVALID, Coord3D::default());
        self.brain.xfer(xfer, version)?;
        if !crc {
            xfer_version_one(xfer)?;
            xfer_machine(xfer, ST_BUSY, ST_BUSY, no_goal, |_| Ok(()))?;
        }
        xfer.xfer_object_id(&mut self.preferred_dock)?;
        xfer.xfer_int(&mut self.number_boxes)?;
        xfer.xfer_bool(&mut self.force_pending)?;
        if !crc {
            xfer_version_one(xfer)?;
            xfer_machine(xfer, AS_DOZER, self.state, no_goal, |_| Ok(()))?;
        }
        Ok(())
    }
}

fn worker(sim: &mut dyn SimApi, ctx: ModuleContext) -> &mut WorkerAi {
    match &mut ai_update::ai(sim, ctx).derived {
        DerivedAi::Worker(worker) => worker,
        _ => panic!("a module that is no worker"),
    }
}

/// `supplyTruckSubMachineWantsToEnter`.
fn wants_supply_truck(sim: &mut dyn SimApi, ctx: ModuleContext) -> bool {
    let docking = ai_update::ai(sim, ctx).machine.current_state_id()
        == crate::ai::ai_state_machine::state_id::DOCK;
    worker(sim, ctx).force_pending != FALSE || docking
}

/// The transition conditions of the current state. The supply truck machine is
/// always busy, which is what `supplyTruckSubMachineReadyToLeave` asks besides.
fn check_conditions(sim: &mut dyn SimApi, ctx: ModuleContext) {
    let wants = wants_supply_truck(sim, ctx);
    match worker(sim, ctx).state {
        AS_DOZER if wants => set_state(sim, ctx, AS_SUPPLY_TRUCK),
        AS_SUPPLY_TRUCK if !wants => set_state(sim, ctx, AS_DOZER),
        _ => {}
    }
}

/// `setState` of the choosing machine: entering a state checks its conditions.
fn set_state(sim: &mut dyn SimApi, ctx: ModuleContext, state: UnsignedInt) {
    worker(sim, ctx).state = state;
    check_conditions(sim, ctx);
}

/// `m_preferredDock = INVALID_ID` of `newTask`.
pub(super) fn forget_preferred_dock(sim: &mut dyn SimApi, ctx: ModuleContext) {
    worker(sim, ctx).preferred_dock = ObjectId::INVALID;
}

/// `exitingSupplyTruckState`, which `newTask` ends with.
pub fn exiting_supply_truck_state(sim: &mut dyn SimApi, ctx: ModuleContext) {
    if worker(sim, ctx).state != AS_SUPPLY_TRUCK {
        return;
    }
    let parms = AiCommandParms::new(AiCommandType::Idle, CommandSourceType::FromAi);
    ai_update::execute_command(sim, ctx, &parms);
    set_state(sim, ctx, AS_DOZER);
}

/// `WorkerAIUpdate::update`.
pub fn update(sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
    let _ = ai_update::update(sim, ctx);
    if sim
        .object(ctx.owner_slot)
        .is_none_or(|o| o.is_effectively_dead() != FALSE)
    {
        return UpdateSleepTime::NONE;
    }
    check_conditions(sim, ctx);
    if worker(sim, ctx).state == AS_DOZER {
        let task = worker(sim, ctx).brain.current_task;
        if task != DOZER_TASK_INVALID {
            dozer_ai_update::check_task(sim, ctx, task);
        }
        dozer_ai_update::update_machine(sim, ctx);
    } else {
        sim.set_weapon_set_flag(ctx.owner_slot, WeaponSetType::MineClearingDetail, TRUE);
    }
    UpdateSleepTime::NONE
}

/// `WorkerAIUpdate::aiDoCommand`, with `privateDock`.
pub fn do_command(sim: &mut dyn SimApi, ctx: ModuleContext, parms: &AiCommandParms) {
    if !dozer_ai_update::takes_command(sim, ctx, parms) {
        return;
    }
    dozer_ai_update::command(sim, ctx, parms);
    if parms.cmd == AiCommandType::Dock && parms.source == CommandSourceType::FromPlayer {
        if let Some(dock) = parms.obj.filter(|&id| sim.find_object(id).is_some()) {
            worker(sim, ctx).preferred_dock = dock;
        }
    }
    // Clearing mines drops what the worker carries.
    if dozer_ai_update::is_clearing_mines(sim, ctx.owner_slot) && worker(sim, ctx).number_boxes > 0
    {
        worker(sim, ctx).number_boxes = 0;
    }
}

pub fn create_worker_ai_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let data: &dyn Any = data.as_any();
    let typed = data
        .downcast_ref::<WorkerAIUpdateModuleData>()
        .expect("WorkerAIUpdate data type mismatch");
    let mut ai = AiUpdate::new(Arc::new(typed.ai.clone()));
    ai.derived = DerivedAi::Worker(Box::new(WorkerAi {
        data: Arc::new(typed.clone()),
        brain: DozerBrain::default(),
        preferred_dock: ObjectId::INVALID,
        number_boxes: 0,
        force_pending: FALSE,
        state: AS_DOZER,
    }));
    Box::new(ai)
}
