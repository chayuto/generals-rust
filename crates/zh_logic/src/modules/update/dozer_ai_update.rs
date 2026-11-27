//! `PKT-321`: `DozerAIUpdate` (AI plan slice S16): construction, the frame
//! update, the dozer's own state machine with its build, repair and mine clearing
//! behaviour, orders and `xfer`.
//!
//! Packet doc: `packets/phase5/PKT-321_DozerAIUpdate.md`. Golden tests:
//! `crates/zh_logic/tests/golden_s16_dozer_worker.rs` (construction) and
//! `golden_s16_dozer_worker_frames.rs` (frames and orders); `dozer_build_sim.rs`.
//!
//! The module is an `AIUpdateInterface` with a [`DozerBrain`], which a
//! `WorkerAIUpdate` has too. As in `ai_state_machine.rs`, the machines are plain
//! data in the module and every transition is a function of `(sim, ctx)`, so a
//! state can order its own unit while it runs.
//!
//! The order to start building (`DozerAIInterface::construct`) is not here: its
//! caller creates the building and hands it to [`new_task`].
//!
//! Not here: the repair of bridges (`canRepairObject` refuses a bridge and its
//! towers), loading a save, building sounds, the idle worker list, messages,
//! radar events, `AcademyStats::recordProduction`, and the computer player's
//! `onStructureProduced`.

use std::any::Any;
use std::sync::Arc;

use crate::arena::{ModuleContext, ObjectSlot};
use crate::modules::module::{BehaviorModule, CommandSourceType, ModuleData};
use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{duration_real, percent, real};
use crate::modules::update::ai_update::{
    self, AiCommandParms, AiCommandType, AiUpdate, AiUpdateModuleData, DerivedAi,
};
use crate::modules::update::worker_ai_update;
use crate::object::weapon::WEAPON_ANTI_MINE;
use crate::object::weapon_set::CanAttackResult;
use crate::player::action_manager::{can_repair_object, can_resume_construction_of};
use crate::rules::combat_types::WeaponSetType;
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::KindOfType;
use crate::rules::model_state::{ModelConditionFlagType, ModelConditionFlags};
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{FindPositionOptions, SimApi, PRIVATE_STATUS_OFF_MAP};
use crate::spatial::partition_query::DistanceCalculationType;
use zh_common::core::state_machine::{StateReturnType, INVALID_STATE_ID};
use zh_common::ini::LOGICFRAMES_PER_SECOND;
use zh_common::math::{Coord2D, Coord3D};
use zh_common::types::{
    Bool, Int, ObjectId, Real, Relationship, TeamId, UnsignedInt, UpdateSleepTime, XferVersion,
    FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError, XferMode};

module_data! {
    /// `DozerAIUpdateModuleData`.
    DozerAIUpdateModuleData = "DozerAIUpdate";
    part ai: AiUpdateModuleData = AiUpdateModuleData::new;
    "RepairHealthPercentPerSecond" =>
        repair_health_percent_per_second: Real = 0.0, percent;
    "BoredTime" => bored_time: Real = 0.0, duration_real;
    "BoredRange" => bored_range: Real = 0.0, real;
}

/// `DOZER_NUM_TASKS`: build, repair, fortify.
pub const DOZER_NUM_TASKS: usize = 3;
/// `DOZER_NUM_DOCK_POINTS`: start, action, end.
pub const DOZER_NUM_DOCK_POINTS: usize = 3;
/// `DOZER_TASK_INVALID`.
pub const DOZER_TASK_INVALID: Int = -1;
pub const DOZER_TASK_BUILD: Int = 0;
pub const DOZER_TASK_REPAIR: Int = 1;
pub const DOZER_TASK_FORTIFY: Int = 2;

/// The states of the primary machine. A task's state is its number plus one.
const PRIMARY_IDLE: UnsignedInt = 0;
const PRIMARY_GO_HOME: UnsignedInt = 4;
/// The states of the action machine of a task.
const ACTION_PICK_POS: UnsignedInt = 0;
const ACTION_MOVE_TO_POS: UnsignedInt = 1;
const ACTION_DO: UnsignedInt = 2;

/// `DozerBuildSubTask`.
const SELECT_BUILD_DOCK_LOCATION: Int = 0;
const MOVING_TO_BUILD_DOCK_LOCATION: Int = 1;
const DO_BUILD_AT_DOCK: Int = 2;

const DOCK_POINT_START: usize = 0;
const DOCK_POINT_ACTION: usize = 1;
const DOCK_POINT_END: usize = 2;
const MIN_ACTION_TOLERANCE: Real = 70.0;
const PATHFIND_CELL_SIZE_F: Real = 10.0;
/// `CONSTRUCTION_COMPLETE`.
const CONSTRUCTION_COMPLETE: Real = -1.0;
const LOCOMOTORSURFACE_AIR: Int = 1 << 3;
/// `AbleToAttackType::ATTACK_NEW_TARGET`.
const ATTACK_NEW_TARGET: UnsignedInt = 0;

/// `DozerTaskInfo`.
#[derive(Debug, Clone, Copy, Default)]
pub struct DozerTaskInfo {
    pub target: ObjectId,
    pub order_frame: UnsignedInt,
}

impl DozerTaskInfo {
    /// The members as `xferUser` writes the struct.
    fn bytes(self) -> [u8; 8] {
        let mut bytes = [0; 8];
        bytes[..4].copy_from_slice(&self.target.0.to_le_bytes());
        bytes[4..].copy_from_slice(&self.order_frame.to_le_bytes());
        bytes
    }
}

/// `DozerDockPointInfo`.
#[derive(Debug, Clone, Copy, Default)]
pub struct DozerDockPoint {
    pub valid: Bool,
    pub location: Coord3D,
}

/// The members of `DozerPrimaryIdleState`.
#[derive(Debug, Clone, Copy, Default)]
pub struct DozerIdleState {
    pub idle_too_long_timestamp: UnsignedInt,
    pub idle_player_number: Int,
    pub is_marked_as_idle: Bool,
}

/// An action machine of the dozer: `DozerActionStateMachine` and the members of
/// its three states.
#[derive(Debug, Clone, Copy)]
pub struct DozerActionMachine {
    /// [`INVALID_STATE_ID`] after the machine exited.
    pub current: UnsignedInt,
    pub goal_object: ObjectId,
    pub goal_position: Coord3D,
    /// `m_failedAttempts` of the first state, which never changes.
    pub failed_attempts: Int,
    /// `m_enterFrame` of the last state.
    pub enter_frame: UnsignedInt,
}

impl Default for DozerActionMachine {
    fn default() -> Self {
        Self {
            current: ACTION_PICK_POS,
            goal_object: ObjectId::INVALID,
            goal_position: Coord3D::default(),
            failed_attempts: 0,
            enter_frame: 0,
        }
    }
}

/// The dozer members of `DozerAIUpdate` and `WorkerAIUpdate`.
#[derive(Debug, Clone)]
pub struct DozerBrain {
    pub tasks: [DozerTaskInfo; DOZER_NUM_TASKS],
    pub current_task: Int,
    pub previous_task: Int,
    pub previous_task_info: DozerTaskInfo,
    pub dock_points: [[DozerDockPoint; DOZER_NUM_DOCK_POINTS]; DOZER_NUM_TASKS],
    /// `DOZER_SELECT_BUILD_DOCK_LOCATION` (0) at first.
    pub build_sub_task: Int,
    pub idle: DozerIdleState,
    /// The current state of `DozerPrimaryStateMachine`.
    pub state: UnsignedInt,
    pub actions: [DozerActionMachine; DOZER_NUM_TASKS],
    /// `m_isRebuild`, not saved.
    pub is_rebuild: Bool,
}

impl Default for DozerBrain {
    fn default() -> Self {
        Self {
            tasks: [DozerTaskInfo::default(); DOZER_NUM_TASKS],
            current_task: DOZER_TASK_INVALID,
            previous_task: DOZER_TASK_INVALID,
            previous_task_info: DozerTaskInfo::default(),
            dock_points: [[DozerDockPoint::default(); DOZER_NUM_DOCK_POINTS]; DOZER_NUM_TASKS],
            build_sub_task: SELECT_BUILD_DOCK_LOCATION,
            idle: DozerIdleState::default(),
            state: PRIMARY_IDLE,
            actions: [DozerActionMachine::default(); DOZER_NUM_TASKS],
            is_rebuild: FALSE,
        }
    }
}

/// `StateMachine::xfer` of a machine that never sleeps and is never locked, with
/// `state` writing the current state.
pub(super) fn xfer_machine(
    xfer: &mut dyn Xfer,
    default_state: UnsignedInt,
    current: UnsignedInt,
    goal: (ObjectId, Coord3D),
    state: impl FnOnce(&mut dyn Xfer) -> Result<(), XferError>,
) -> Result<(), XferError> {
    xfer_version_one(xfer)?;
    let mut sleep_till: UnsignedInt = 0;
    xfer.xfer_unsigned_int(&mut sleep_till)?;
    let (mut default_id, mut current_id) = (default_state, current);
    xfer.xfer_unsigned_int(&mut default_id)?;
    xfer.xfer_unsigned_int(&mut current_id)?;
    let mut all_states = FALSE;
    xfer.xfer_bool(&mut all_states)?;
    state(xfer)?;
    let (mut goal_object, mut goal_position) = goal;
    xfer.xfer_object_id(&mut goal_object)?;
    xfer.xfer_coord3d(&mut goal_position)?;
    let (mut locked, mut default_inited) = (FALSE, TRUE);
    xfer.xfer_bool(&mut locked)?;
    xfer.xfer_bool(&mut default_inited)
}

/// `xferUser` of an enum.
fn xfer_user_int(xfer: &mut dyn Xfer, value: Int) -> Result<(), XferError> {
    xfer.xfer_user(&mut value.to_le_bytes())
}

/// A version that was never raised.
pub(super) fn xfer_version_one(xfer: &mut dyn Xfer) -> Result<(), XferError> {
    let mut version: XferVersion = 1;
    xfer.xfer_version(&mut version, 1)
}

impl DozerBrain {
    /// The dozer half of `createMachines`: the primary machine enters its idle
    /// state, which asks for the mine clearing weapon set.
    pub fn create_machine(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.state = PRIMARY_IDLE;
        self.idle.idle_too_long_timestamp = sim.frame();
        self.idle.is_marked_as_idle = FALSE;
        sim.set_weapon_set_flag(ctx.owner_slot, WeaponSetType::MineClearingDetail, TRUE);
    }

    /// The primary machine's `xfer`. `version` 2 also writes the action machines.
    fn xfer_primary(&mut self, xfer: &mut dyn Xfer, version: XferVersion) -> Result<(), XferError> {
        // A machine without a state takes its default one when written.
        if self.state == INVALID_STATE_ID {
            self.state = PRIMARY_IDLE;
        }
        let state = self.state;
        let idle = &mut self.idle;
        let actions = &mut self.actions;
        xfer_version_one(xfer)?;
        let no_goal = (ObjectId::INVALID, Coord3D::default());
        xfer_machine(xfer, PRIMARY_IDLE, state, no_goal, |x| match state {
            PRIMARY_IDLE => {
                xfer_version_one(x)?;
                x.xfer_unsigned_int(&mut idle.idle_too_long_timestamp)?;
                x.xfer_int(&mut idle.idle_player_number)?;
                x.xfer_bool(&mut idle.is_marked_as_idle)
            }
            PRIMARY_GO_HOME => Ok(()),
            _ => {
                let task = (state as usize - 1) % DOZER_NUM_TASKS;
                xfer_version_one(x)?;
                xfer_user_int(x, task as Int)?;
                xfer_action(x, &mut actions[task], task as Int, version)
            }
        })
    }

    /// What `DozerAIUpdate::xfer` and `WorkerAIUpdate::xfer` write after the base
    /// class, up to the build sub task. The machine's `crc` is empty.
    pub fn xfer(&mut self, xfer: &mut dyn Xfer, version: XferVersion) -> Result<(), XferError> {
        let mut count = DOZER_NUM_TASKS as Int;
        xfer.xfer_int(&mut count)?;
        for task in &mut self.tasks {
            xfer.xfer_object_id(&mut task.target)?;
            xfer.xfer_unsigned_int(&mut task.order_frame)?;
        }
        if xfer.mode() != XferMode::Crc {
            self.xfer_primary(xfer, version)?;
        }
        xfer_user_int(xfer, self.current_task)?;
        if version >= 2 {
            xfer_user_int(xfer, self.previous_task)?;
            xfer.xfer_user(&mut self.previous_task_info.bytes())?;
        }
        let mut points = DOZER_NUM_DOCK_POINTS as Int;
        xfer.xfer_int(&mut points)?;
        for point in self.dock_points.iter_mut().flatten() {
            xfer.xfer_bool(&mut point.valid)?;
            xfer.xfer_coord3d(&mut point.location)?;
        }
        xfer_user_int(xfer, self.build_sub_task)
    }
}

/// `DozerActionStateMachine::xfer`. Version 1 keeps only the task.
fn xfer_action(
    xfer: &mut dyn Xfer,
    action: &mut DozerActionMachine,
    task: Int,
    version: XferVersion,
) -> Result<(), XferError> {
    let mut written = version;
    xfer.xfer_version(&mut written, version)?;
    xfer_user_int(xfer, task)?;
    if version < 2 {
        return Ok(());
    }
    if action.current == INVALID_STATE_ID {
        action.current = ACTION_PICK_POS;
    }
    let a = *action;
    let goal = (a.goal_object, a.goal_position);
    xfer_machine(xfer, ACTION_PICK_POS, a.current, goal, |x| {
        xfer_version_one(x)?;
        xfer_user_int(x, task)?;
        let (mut failed, mut entered) = (a.failed_attempts, a.enter_frame);
        match a.current {
            ACTION_PICK_POS => x.xfer_int(&mut failed),
            ACTION_DO => x.xfer_unsigned_int(&mut entered),
            _ => Ok(()),
        }
    })
}

// -----------------------------------------------------------------------------
// The module's members
// -----------------------------------------------------------------------------

fn ai(sim: &mut dyn SimApi, ctx: ModuleContext) -> &mut AiUpdate {
    ai_update::ai(sim, ctx)
}

fn brain(sim: &mut dyn SimApi, ctx: ModuleContext) -> &mut DozerBrain {
    match &mut ai(sim, ctx).derived {
        DerivedAi::Dozer(dozer) => &mut dozer.brain,
        DerivedAi::Worker(worker) => &mut worker.brain,
        DerivedAi::Base => panic!("a module without the dozer machine"),
    }
}

fn is_worker(sim: &mut dyn SimApi, ctx: ModuleContext) -> bool {
    matches!(ai(sim, ctx).derived, DerivedAi::Worker(_))
}

/// `getRepairHealthPerSecond`, `getBoredTime` and the data's bored range.
fn numbers(sim: &mut dyn SimApi, ctx: ModuleContext) -> (Real, Real, Real) {
    match &ai(sim, ctx).derived {
        DerivedAi::Dozer(d) => (
            d.data.repair_health_percent_per_second,
            d.data.bored_time,
            d.data.bored_range,
        ),
        DerivedAi::Worker(w) => (
            w.data.repair_health_percent_per_second,
            w.data.bored_time,
            w.data.bored_range,
        ),
        DerivedAi::Base => (0.0, 0.0, 0.0),
    }
}

/// `getBoredRange`: the computer's dozers look farther.
fn bored_range(sim: &mut dyn SimApi, ctx: ModuleContext) -> Real {
    let range = numbers(sim, ctx).2;
    if is_worker(sim, ctx) {
        return range;
    }
    match team_of(sim, ctx.owner_slot) {
        Some(team) if sim.player_is_human(team) == FALSE => {
            sim.rules().ai_data.ai_dozer_bored_radius_modifier * range
        }
        _ => range,
    }
}

/// The current task and the target of the build task of the dozer AI of `object`.
pub fn dozer_task_of(sim: &mut dyn SimApi, object: ObjectSlot) -> Option<(Int, ObjectId)> {
    let module = sim.object(object)?.ai_slot?;
    let state = sim
        .resident_state_mut(module)?
        .downcast_mut::<AiUpdate>()?;
    let brain = match &state.derived {
        DerivedAi::Dozer(d) => &d.brain,
        DerivedAi::Worker(w) => &w.brain,
        DerivedAi::Base => return None,
    };
    Some((
        brain.current_task,
        brain.tasks[DOZER_TASK_BUILD as usize].target,
    ))
}

fn team_of(sim: &dyn SimApi, object: ObjectSlot) -> Option<TeamId> {
    sim.object(object).map(|o| o.team_id)
}

fn player_of(sim: &dyn SimApi, object: ObjectSlot) -> Option<Int> {
    team_of(sim, object).map(|team| sim.player_index(team))
}

fn position(sim: &dyn SimApi, object: ObjectSlot) -> Coord3D {
    sim.object(object)
        .map_or_else(Coord3D::default, |o| *o.thing.get_position())
}

fn dead(sim: &dyn SimApi, object: ObjectSlot) -> bool {
    sim.object(object)
        .is_none_or(|o| o.is_effectively_dead() != FALSE)
}

fn off_map(sim: &dyn SimApi, object: ObjectSlot) -> bool {
    sim.object(object)
        .is_some_and(|o| o.private_status & PRIVATE_STATUS_OFF_MAP != 0)
}

fn has_status(sim: &dyn SimApi, object: ObjectSlot, status: ObjectStatusType) -> bool {
    sim.object(object)
        .is_some_and(|o| o.status.test(status.as_i32()) != FALSE)
}

fn mine_clearing_weapon(sim: &mut dyn SimApi, ctx: ModuleContext, set: Bool) {
    sim.set_weapon_set_flag(ctx.owner_slot, WeaponSetType::MineClearingDetail, set);
}

fn conditions(flags: &[ModelConditionFlagType]) -> ModelConditionFlags {
    let mut set = ModelConditionFlags::new();
    for &flag in flags {
        set.set(flag as Int, 1);
    }
    set
}

/// `AIUpdateInterface::isClearingMines`.
pub fn is_clearing_mines(sim: &dyn SimApi, object: ObjectSlot) -> bool {
    if !has_status(sim, object, ObjectStatusType::IsAttacking) {
        return false;
    }
    sim.object(object).is_some_and(|o| {
        let slot = o.weapon_set.get_cur_weapon_slot();
        o.weapon_set
            .get_weapon_in_weapon_slot(slot)
            .is_some_and(|w| w.get_template().anti_mask & WEAPON_ANTI_MINE != 0)
    })
}

// -----------------------------------------------------------------------------
// Orders to itself
// -----------------------------------------------------------------------------

fn order(sim: &mut dyn SimApi, ctx: ModuleContext, parms: &AiCommandParms) {
    ai_update::execute_command(sim, ctx, parms);
}

fn idle_self(sim: &mut dyn SimApi, ctx: ModuleContext) {
    let parms = AiCommandParms::new(AiCommandType::Idle, CommandSourceType::FromAi);
    order(sim, ctx, &parms);
}

fn move_self(sim: &mut dyn SimApi, ctx: ModuleContext, pos: &Coord3D) {
    let mut parms = AiCommandParms::new(AiCommandType::MoveToPosition, CommandSourceType::FromAi);
    parms.pos = *pos;
    order(sim, ctx, &parms);
}

// -----------------------------------------------------------------------------
// Tasks
// -----------------------------------------------------------------------------

/// `isTaskPending`.
pub fn is_task_pending(brain: &DozerBrain, task: Int) -> bool {
    brain.tasks[task as usize].target != ObjectId::INVALID
}

/// `getMostRecentCommand`: a task ordered in frame 0 never counts.
pub fn most_recent_command(brain: &DozerBrain) -> Int {
    let mut task = DOZER_TASK_INVALID;
    let mut frame = 0;
    for (i, info) in brain.tasks.iter().enumerate() {
        if info.target != ObjectId::INVALID && info.order_frame > frame {
            task = i as Int;
            frame = info.order_frame;
        }
    }
    task
}

fn clear_previous_task(brain: &mut DozerBrain) {
    brain.previous_task = DOZER_TASK_INVALID;
    brain.previous_task_info = DozerTaskInfo::default();
}

fn forget_task(brain: &mut DozerBrain, task: Int) {
    brain.tasks[task as usize] = DozerTaskInfo::default();
    for point in &mut brain.dock_points[task as usize] {
        point.valid = FALSE;
    }
}

/// `internalTaskCompleteOrCancelled`.
fn task_complete_or_cancelled(sim: &mut dyn SimApi, ctx: ModuleContext, task: Int) {
    if task == DOZER_TASK_BUILD || task == DOZER_TASK_REPAIR {
        sim.clear_model_condition_state(
            ctx.owner_slot,
            ModelConditionFlagType::ActivelyConstructing,
        );
    }
}

/// `internalTaskComplete`.
fn internal_task_complete(sim: &mut dyn SimApi, ctx: ModuleContext, task: Int) {
    task_complete_or_cancelled(sim, ctx, task);
    let brain = brain(sim, ctx);
    forget_task(brain, task);
    clear_previous_task(brain);
}

/// `internalCancelTask`: a dozer goes idle, a worker is ordered to where it
/// stands.
fn internal_cancel_task(sim: &mut dyn SimApi, ctx: ModuleContext, task: Int) {
    if task < 0 || task >= DOZER_NUM_TASKS as Int {
        return;
    }
    task_complete_or_cancelled(sim, ctx, task);
    forget_task(brain(sim, ctx), task);
    if is_worker(sim, ctx) {
        let here = position(sim, ctx.owner_slot);
        move_self(sim, ctx, &here);
    } else {
        idle_self(sim, ctx);
    }
}

/// `cancelTask`.
pub fn cancel_task(sim: &mut dyn SimApi, ctx: ModuleContext, task: Int, remember: bool) {
    let state = brain(sim, ctx);
    if !remember {
        clear_previous_task(state);
    } else if task != DOZER_TASK_INVALID {
        state.previous_task = task;
        state.previous_task_info = state.tasks[task as usize];
    }
    internal_cancel_task(sim, ctx, task);
    reset_machine(sim, ctx);
}

/// `cancelAllTasks`.
pub fn cancel_all_tasks(sim: &mut dyn SimApi, ctx: ModuleContext) {
    clear_previous_task(brain(sim, ctx));
    for task in 0..DOZER_NUM_TASKS as Int {
        internal_cancel_task(sim, ctx, task);
    }
    reset_machine(sim, ctx);
}

/// `resumePreviousTask`.
fn resume_previous_task(sim: &mut dyn SimApi, ctx: ModuleContext) {
    let task = brain(sim, ctx).previous_task;
    if task == DOZER_TASK_INVALID {
        return;
    }
    let id = brain(sim, ctx).previous_task_info.target;
    if let Some(target) = sim.find_object(id) {
        if task != DOZER_TASK_BUILD || has_status(sim, target, ObjectStatusType::UnderConstruction)
        {
            new_task(sim, ctx, task, target);
        }
    }
    clear_previous_task(brain(sim, ctx));
}

/// `findGoodBuildOrRepairPosition`. A bridge, whose towers the reference tries
/// instead, cannot be the target of a task here.
fn good_build_position(sim: &mut dyn SimApi, me: ObjectSlot, target: ObjectSlot) -> Coord3D {
    let ours = position(sim, me);
    let theirs = position(sim, target);
    let mut offset = Coord3D {
        x: ours.x - theirs.x,
        y: ours.y - theirs.y,
        z: ours.z - theirs.z,
    };
    offset.normalize_ww_vector3();
    let half = sim
        .object(target)
        .map_or(0.0, |o| o.geometry.get_major_radius())
        / 2.0;
    let working = Coord3D {
        x: theirs.x + offset.x * half,
        y: theirs.y + offset.y * half,
        z: theirs.z + offset.z * half,
    };
    let airborne = sim.object(me).and_then(|o| o.ai_slot).is_some_and(|ms| {
        sim.resident_state_mut(ms)
            .and_then(|a| a.downcast_mut::<AiUpdate>())
            .and_then(|a| a.cur_locomotor())
            .is_some_and(|l| l.template.surfaces & LOCOMOTORSURFACE_AIR != 0)
    });
    let mut options = FindPositionOptions {
        min_radius: 0.0,
        max_radius: 100.0,
        source_to_path_to_dest: Some(me),
        ..FindPositionOptions::default()
    };
    if airborne {
        options.ignore_object = Some(target);
    } else {
        options.max_z_delta = 10.0;
    }
    sim.find_position_around(&working, &options)
        .unwrap_or(working)
}

/// `newTask` of the dozer and of the worker.
pub fn new_task(sim: &mut dyn SimApi, ctx: ModuleContext, task: Int, target: ObjectSlot) {
    let worker = is_worker(sim, ctx);
    if worker {
        worker_ai_update::forget_preferred_dock(sim, ctx);
    }
    let Some(target_id) = sim.object(target).map(|o| o.id) else {
        return;
    };
    if task == DOZER_TASK_BUILD || task == DOZER_TASK_REPAIR {
        if is_task_pending(brain(sim, ctx), task) {
            cancel_task(sim, ctx, task, false);
        }
        let me = ctx.owner_slot;
        let spot = good_build_position(sim, me, target);
        if task == DOZER_TASK_BUILD {
            let builder = ctx.owner_id;
            if let Some(o) = sim.object_mut(target) {
                o.builder_id = builder;
            }
        }
        // A dozer leaves the building away from it; a worker stays.
        let mut end = spot;
        if !worker {
            let theirs = position(sim, target);
            let mut offset = Coord3D {
                x: spot.x - theirs.x,
                y: spot.y - theirs.y,
                z: 0.0,
            };
            offset.normalize();
            let scale = 5.0 * PATHFIND_CELL_SIZE_F;
            end.x += offset.x * scale;
            end.y += offset.y * scale;
            end.z += offset.z * scale;
        }
        let points = &mut brain(sim, ctx).dock_points[task as usize];
        for (index, location) in [
            (DOCK_POINT_START, spot),
            (DOCK_POINT_ACTION, spot),
            (DOCK_POINT_END, end),
        ] {
            points[index] = DozerDockPoint {
                valid: TRUE,
                location,
            };
        }
    }
    let frame = sim.frame();
    brain(sim, ctx).tasks[task as usize] = DozerTaskInfo {
        target: target_id,
        order_frame: frame,
    };
    reset_machine(sim, ctx);
    if worker {
        worker_ai_update::exiting_supply_truck_state(sim, ctx);
    }
}

/// `canAcceptNewRepair`. A bridge tower is refused by `canRepairObject`, so the
/// test of two towers of one bridge is not made.
fn can_accept_new_repair(sim: &mut dyn SimApi, ctx: ModuleContext, target: ObjectSlot) -> bool {
    let state = brain(sim, ctx);
    if state.current_task != DOZER_TASK_REPAIR {
        return true;
    }
    let current = state.tasks[DOZER_TASK_REPAIR as usize].target;
    sim.find_object(current) != Some(target)
}

/// `getSoleHealingBenefactor`.
fn sole_healing_benefactor(sim: &dyn SimApi, object: ObjectSlot) -> ObjectId {
    let now = sim.frame();
    sim.object(object).map_or(ObjectId::INVALID, |o| {
        if now > o.sole_healing_benefactor_expiration_frame {
            ObjectId::INVALID
        } else {
            o.sole_healing_benefactor_id
        }
    })
}

/// `privateRepair`.
fn private_repair(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    target: Option<ObjectSlot>,
    source: CommandSourceType,
) {
    let Some(target) = target else {
        return;
    };
    if !can_accept_new_repair(sim, ctx, target)
        || can_repair_object(sim, ctx.owner_slot, Some(target), source) == FALSE
    {
        return;
    }
    let repairer = sole_healing_benefactor(sim, target);
    if repairer != ObjectId::INVALID && repairer != ctx.owner_id {
        return;
    }
    new_task(sim, ctx, DOZER_TASK_REPAIR, target);
}

/// `privateResumeConstruction`.
fn private_resume_construction(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    target: Option<ObjectSlot>,
    source: CommandSourceType,
) {
    let Some(target) = target else {
        return;
    };
    if can_resume_construction_of(sim, ctx.owner_slot, Some(target), source) == FALSE {
        return;
    }
    new_task(sim, ctx, DOZER_TASK_BUILD, target);
}

// -----------------------------------------------------------------------------
// The primary machine
// -----------------------------------------------------------------------------

fn primary_on_enter(sim: &mut dyn SimApi, ctx: ModuleContext, state: UnsignedInt) {
    match state {
        PRIMARY_IDLE => {
            let frame = sim.frame();
            let idle = &mut brain(sim, ctx).idle;
            idle.idle_too_long_timestamp = frame;
            idle.is_marked_as_idle = FALSE;
            mine_clearing_weapon(sim, ctx, TRUE);
        }
        PRIMARY_GO_HOME => {}
        _ => {
            let task = state as usize - 1;
            brain(sim, ctx).current_task = task as Int;
            reset_action(sim, ctx, task);
        }
    }
}

fn primary_on_exit(sim: &mut dyn SimApi, ctx: ModuleContext, state: UnsignedInt) {
    match state {
        PRIMARY_IDLE => {
            let idle = &mut brain(sim, ctx).idle;
            if idle.is_marked_as_idle != FALSE {
                idle.idle_player_number = -1;
                idle.is_marked_as_idle = FALSE;
            }
        }
        PRIMARY_GO_HOME => {}
        _ => brain(sim, ctx).current_task = DOZER_TASK_INVALID,
    }
}

/// `friend_checkForTransitions`: only the idle state has conditions, and every
/// state succeeds and fails to it.
fn primary_transitions(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    state: UnsignedInt,
    status: StateReturnType,
) {
    match status {
        StateReturnType::Success | StateReturnType::Failure => {
            primary_set_state(sim, ctx, PRIMARY_IDLE);
        }
        StateReturnType::Continue if state == PRIMARY_IDLE => {
            if ai(sim, ctx).is_idle() == FALSE {
                return;
            }
            let task = most_recent_command(brain(sim, ctx));
            if task != DOZER_TASK_INVALID {
                primary_set_state(sim, ctx, task as UnsignedInt + 1);
            }
        }
        _ => {}
    }
}

/// `internalSetState`.
fn primary_set_state(sim: &mut dyn SimApi, ctx: ModuleContext, new_state: UnsignedInt) {
    let old = brain(sim, ctx).state;
    if old != INVALID_STATE_ID {
        primary_on_exit(sim, ctx, old);
    }
    brain(sim, ctx).state = new_state;
    primary_on_enter(sim, ctx, new_state);
    // An `onEnter` that changed the state already checked the new one's conditions.
    if brain(sim, ctx).state == new_state {
        primary_transitions(sim, ctx, new_state, StateReturnType::Continue);
    }
}

/// `resetToDefaultState` of the primary machine.
pub fn reset_machine(sim: &mut dyn SimApi, ctx: ModuleContext) {
    let old = brain(sim, ctx).state;
    if old != INVALID_STATE_ID {
        primary_on_exit(sim, ctx, old);
    }
    brain(sim, ctx).state = INVALID_STATE_ID;
    primary_set_state(sim, ctx, PRIMARY_IDLE);
}

/// `updateStateMachine` of the primary machine.
pub fn update_machine(sim: &mut dyn SimApi, ctx: ModuleContext) {
    let state = brain(sim, ctx).state;
    let status = match state {
        INVALID_STATE_ID => return,
        PRIMARY_IDLE => idle_update(sim, ctx),
        PRIMARY_GO_HOME => StateReturnType::Failure,
        _ => update_action(sim, ctx, state as usize - 1),
    };
    let current = brain(sim, ctx).state;
    if current == INVALID_STATE_ID {
        return;
    }
    let status = if current == state {
        status
    } else {
        StateReturnType::Continue
    };
    primary_transitions(sim, ctx, current, status);
}

/// `findObjectToRepair`.
fn find_object_to_repair(sim: &mut dyn SimApi, ctx: ModuleContext) -> Option<ObjectSlot> {
    let me = ctx.owner_slot;
    let range = bored_range(sim, ctx);
    let player = player_of(sim, me);
    let mine_off_map = off_map(sim, me);
    let mut closest: Option<(ObjectSlot, Real)> = None;
    for other in sim.find_objects_in_range_of(me, range) {
        if player_of(sim, other) != player
            || sim.object_is_kind_of(other, KindOfType::Structure) == FALSE
            || off_map(sim, other) != mine_off_map
            || can_repair_object(sim, me, Some(other), CommandSourceType::FromAi) == FALSE
        {
            continue;
        }
        let pos = position(sim, other);
        let distance = sim.partition_distance_squared(
            me,
            Some(other),
            &pos,
            DistanceCalculationType::FromCenter2d,
        );
        if closest.is_none_or(|(_, best)| distance < best) {
            closest = Some((other, distance));
        }
    }
    closest.map(|(slot, _)| slot)
}

/// `findMine`: the closest enemy or neutral thing the dozer may attack as such.
fn find_mine(sim: &mut dyn SimApi, ctx: ModuleContext) -> Option<ObjectSlot> {
    let me = ctx.owner_slot;
    let range = bored_range(sim, ctx);
    let mine_off_map = off_map(sim, me);
    let view: &dyn SimApi = sim;
    view.closest_object_from(
        me,
        range,
        DistanceCalculationType::FromCenter2d,
        &mut |other| {
            matches!(
                view.player_relationship(me, other),
                Relationship::Enemies | Relationship::Neutral
            ) && matches!(
                view.able_to_attack_specific_object(
                    me,
                    ATTACK_NEW_TARGET,
                    other,
                    CommandSourceType::FromDozer,
                ),
                CanAttackResult::Possible | CanAttackResult::PossibleAfterMoving
            ) && off_map(view, other) == mine_off_map
        },
    )
}

/// `DozerPrimaryIdleState::update`.
fn idle_update(sim: &mut dyn SimApi, ctx: ModuleContext) -> StateReturnType {
    let me = ctx.owner_slot;
    let ai_idle = ai(sim, ctx).is_idle() != FALSE;
    let is_dead = dead(sim, me);
    if ai_idle && brain(sim, ctx).idle.is_marked_as_idle == FALSE && !is_dead {
        let player = player_of(sim, me).unwrap_or(0);
        let idle = &mut brain(sim, ctx).idle;
        idle.idle_player_number = player;
        idle.is_marked_as_idle = TRUE;
        mine_clearing_weapon(sim, ctx, TRUE);
    }
    let idle = &mut brain(sim, ctx).idle;
    if idle.is_marked_as_idle != FALSE && (!ai_idle || is_dead) {
        idle.idle_player_number = -1;
        idle.is_marked_as_idle = FALSE;
    }
    let frame = sim.frame();
    if !ai_idle {
        brain(sim, ctx).idle.idle_too_long_timestamp = frame;
    }
    let bored_time = numbers(sim, ctx).1;
    let state = brain(sim, ctx);
    let waited = frame.wrapping_sub(state.idle.idle_too_long_timestamp);
    let any_task = (0..DOZER_NUM_TASKS as Int).any(|t| is_task_pending(state, t));
    if waited as Real > bored_time && !any_task {
        state.idle.idle_too_long_timestamp = frame;
        if let Some(target) = find_object_to_repair(sim, ctx) {
            let mut parms = AiCommandParms::new(AiCommandType::Repair, CommandSourceType::FromAi);
            parms.obj = sim.object(target).map(|o| o.id);
            order(sim, ctx, &parms);
        } else {
            mine_clearing_weapon(sim, ctx, TRUE);
            if let Some(mine) = find_mine(sim, ctx) {
                let mut parms = AiCommandParms::new(
                    AiCommandType::AttackObject,
                    CommandSourceType::FromDozer,
                );
                parms.obj = sim.object(mine).map(|o| o.id);
                parms.int_value = 1;
                order(sim, ctx, &parms);
            }
        }
    }
    StateReturnType::Continue
}

// -----------------------------------------------------------------------------
// The action machines
// -----------------------------------------------------------------------------

fn action(sim: &mut dyn SimApi, ctx: ModuleContext, task: usize) -> &mut DozerActionMachine {
    &mut brain(sim, ctx).actions[task % DOZER_NUM_TASKS]
}

fn action_set_goal(sim: &mut dyn SimApi, ctx: ModuleContext, task: usize, id: Option<ObjectId>) {
    let goal = id.and_then(|id| sim.find_object(id).map(|slot| (id, position(sim, slot))));
    let machine = action(sim, ctx, task);
    match goal {
        Some((id, pos)) => {
            machine.goal_object = id;
            machine.goal_position = pos;
        }
        None => machine.goal_object = ObjectId::INVALID,
    }
}

/// `internalSetState` of an action machine, with the states' `onEnter`.
fn action_set_state(sim: &mut dyn SimApi, ctx: ModuleContext, task: usize, state: UnsignedInt) {
    action(sim, ctx, task).current = state;
    if state == ACTION_DO {
        let frame = sim.frame();
        action(sim, ctx, task).enter_frame = frame;
        if task as Int == DOZER_TASK_BUILD {
            brain(sim, ctx).build_sub_task = SELECT_BUILD_DOCK_LOCATION;
        }
    }
}

/// `resetToDefaultState` of an action machine.
fn reset_action(sim: &mut dyn SimApi, ctx: ModuleContext, task: usize) {
    let machine = action(sim, ctx, task);
    machine.goal_object = ObjectId::INVALID;
    machine.goal_position = Coord3D::default();
    action_set_state(sim, ctx, task, ACTION_PICK_POS);
}

/// `updateStateMachine` of an action machine.
fn update_action(sim: &mut dyn SimApi, ctx: ModuleContext, task: usize) -> StateReturnType {
    let state = action(sim, ctx, task).current;
    let status = match state {
        ACTION_PICK_POS => pick_action_pos(sim, ctx, task),
        ACTION_MOVE_TO_POS => move_to_action_pos(sim, ctx, task),
        ACTION_DO => do_action(sim, ctx, task),
        _ => return StateReturnType::Failure,
    };
    // `None` leaves the machine.
    let next = match (state, status) {
        (ACTION_PICK_POS, StateReturnType::Success) => Some(ACTION_MOVE_TO_POS),
        (ACTION_MOVE_TO_POS, StateReturnType::Success) => Some(ACTION_DO),
        (ACTION_MOVE_TO_POS, StateReturnType::Failure) => Some(ACTION_PICK_POS),
        (_, StateReturnType::Success | StateReturnType::Failure) => None,
        _ => return status,
    };
    match next {
        Some(next) => {
            action_set_state(sim, ctx, task, next);
            StateReturnType::Continue
        }
        None => {
            action(sim, ctx, task).current = INVALID_STATE_ID;
            status
        }
    }
}

/// `DozerActionPickActionPosState::update`.
fn pick_action_pos(sim: &mut dyn SimApi, ctx: ModuleContext, task: usize) -> StateReturnType {
    let me = ctx.owner_slot;
    let target = brain(sim, ctx).tasks[task].target;
    let Some(goal) = sim.find_object(target) else {
        action_set_goal(sim, ctx, task, None);
        cancel_task(sim, ctx, task as Int, false);
        return StateReturnType::Failure;
    };
    let dock = brain(sim, ctx).dock_points[task][DOCK_POINT_START];
    let goal_pos = if dock.valid != FALSE {
        dock.location
    } else {
        let ours = position(sim, me);
        let theirs = position(sim, goal);
        let away = Coord2D {
            x: ours.x - theirs.x,
            y: ours.y - theirs.y,
        };
        let radius = sim
            .object(goal)
            .map_or(0.0, |o| o.geometry.get_bounding_sphere_radius());
        let options = FindPositionOptions {
            min_radius: radius,
            max_radius: radius,
            start_angle: Some(away.to_angle()),
            ..FindPositionOptions::default()
        };
        let spot = sim.find_position_around(&theirs, &options).unwrap_or(theirs);
        ai(sim, ctx).ignore_obstacle_id = target;
        spot
    };
    action_set_goal(sim, ctx, task, Some(target));
    action(sim, ctx, task).goal_position = goal_pos;
    ai(sim, ctx).ignore_obstacle_id = target;
    move_self(sim, ctx, &goal_pos);
    StateReturnType::Success
}

/// `DozerActionMoveToActionPosState::update`.
fn move_to_action_pos(sim: &mut dyn SimApi, ctx: ModuleContext, task: usize) -> StateReturnType {
    let me = ctx.owner_slot;
    let id = action(sim, ctx, task).goal_object;
    let Some(goal) = sim.find_object(id) else {
        return StateReturnType::Failure;
    };
    let repairer = sole_healing_benefactor(sim, goal);
    let taken = task as Int == DOZER_TASK_REPAIR
        && repairer != ObjectId::INVALID
        && repairer != ctx.owner_id;
    let other_builds = task as Int == DOZER_TASK_BUILD
        && sim
            .object(goal)
            .is_some_and(|o| o.builder_id != ctx.owner_id);
    if taken || other_builds {
        internal_task_complete(sim, ctx, task as Int);
        action_set_goal(sim, ctx, task, None);
        return StateReturnType::Failure;
    }
    let goal_pos = action(sim, ctx, task).goal_position;
    let distance = sim.partition_distance_squared(
        me,
        None,
        &goal_pos,
        DistanceCalculationType::FromBoundingSphere2d,
    );
    let reach = sim
        .object(me)
        .map_or(0.0, |o| o.geometry.get_bounding_sphere_radius())
        + 15.0;
    let allowed = if MIN_ACTION_TOLERANCE > reach {
        MIN_ACTION_TOLERANCE
    } else {
        reach
    };
    if distance <= allowed * allowed {
        if task as Int == DOZER_TASK_BUILD {
            use ModelConditionFlagType as F;
            sim.clear_and_set_model_condition_flags(
                goal,
                &conditions(&[F::AwaitingConstruction]),
                &conditions(&[F::PartiallyConstructed, F::ActivelyBeingConstructed]),
            );
        }
        return StateReturnType::Success;
    }
    if ai(sim, ctx).is_idle() != FALSE {
        return StateReturnType::Failure;
    }
    StateReturnType::Continue
}

/// `DozerActionDoActionState::update`.
fn do_action(sim: &mut dyn SimApi, ctx: ModuleContext, task: usize) -> StateReturnType {
    let me = ctx.owner_slot;
    let id = action(sim, ctx, task).goal_object;
    let Some(goal) = sim.find_object(id) else {
        return StateReturnType::Failure;
    };
    if sim
        .object(me)
        .is_some_and(|o| o.is_disabled_by_type(DisabledType::Unmanned) != FALSE)
    {
        return StateReturnType::Failure;
    }
    let complete = match task as Int {
        DOZER_TASK_BUILD => {
            if player_of(sim, me) != player_of(sim, goal) {
                return StateReturnType::Failure;
            }
            build(sim, ctx, goal)
        }
        DOZER_TASK_REPAIR => {
            if sim.object_health(goal) == sim.object_max_health(goal) {
                true
            } else {
                sim.set_model_condition_state(me, ModelConditionFlagType::ActivelyConstructing);
                let health = sim.object_max_health(goal) * numbers(sim, ctx).0
                    / LOGICFRAMES_PER_SECOND as Real;
                if sim.attempt_healing_from_sole_benefactor(goal, health, ctx.owner_id, 2) == FALSE
                {
                    internal_task_complete(sim, ctx, task as Int);
                    action_set_goal(sim, ctx, task, None);
                    return StateReturnType::Failure;
                }
                false
            }
        }
        _ => false,
    };
    if complete {
        internal_task_complete(sim, ctx, task as Int);
        action_set_goal(sim, ctx, task, None);
        mine_clearing_weapon(sim, ctx, TRUE);
        return StateReturnType::Success;
    }
    mine_clearing_weapon(sim, ctx, FALSE);
    StateReturnType::Continue
}

/// The build task of `DozerActionDoActionState::update`: `true` once complete.
fn build(sim: &mut dyn SimApi, ctx: ModuleContext, goal: ObjectSlot) -> bool {
    let me = ctx.owner_slot;
    let task = DOZER_TASK_BUILD as usize;
    if brain(sim, ctx).build_sub_task == SELECT_BUILD_DOCK_LOCATION {
        let dock = brain(sim, ctx).dock_points[task][DOCK_POINT_ACTION];
        if dock.valid != FALSE {
            move_self(sim, ctx, &dock.location);
        }
        brain(sim, ctx).build_sub_task = MOVING_TO_BUILD_DOCK_LOCATION;
    }
    if brain(sim, ctx).build_sub_task == MOVING_TO_BUILD_DOCK_LOCATION
        && ai(sim, ctx).is_idle() != FALSE
    {
        brain(sim, ctx).build_sub_task = DO_BUILD_AT_DOCK;
    }
    if brain(sim, ctx).build_sub_task != DO_BUILD_AT_DOCK {
        return false;
    }
    sim.set_model_condition_state(me, ModelConditionFlagType::ActivelyConstructing);
    let Some((team, template)) = team_of(sim, me).zip(sim.object_template_id(goal)) else {
        return false;
    };
    let frames = sim.unit_time_to_build(team, template);
    // `WWMath::Div_Safe(100, frames, 1)` of the deterministic build.
    let divisor = if frames as Real > 1.0 {
        frames as Real
    } else {
        1.0
    };
    let percent = sim.object_construction_percent(goal) + 100.0 / divisor;
    sim.set_object_construction_percent(goal, percent);
    let health = sim.object_max_health(goal) / frames as Real;
    sim.body_internal_change_health(goal, health);
    let builder = ctx.owner_id;
    if let Some(o) = sim.object_mut(goal) {
        o.producer_id = builder;
        o.builder_id = builder;
    }
    if sim.object_construction_percent(goal) < 100.0 {
        return false;
    }
    sim.clear_status(goal, ObjectStatusType::UnderConstruction);
    sim.clear_status(goal, ObjectStatusType::Reconstructing);
    use ModelConditionFlagType as F;
    sim.clear_and_set_model_condition_flags(
        goal,
        &conditions(&[
            F::AwaitingConstruction,
            F::PartiallyConstructed,
            F::ActivelyBeingConstructed,
        ]),
        &conditions(&[]),
    );
    sim.set_object_construction_percent(goal, CONSTRUCTION_COMPLETE);
    sim.body_evaluate_visual_condition(goal);
    let is_rebuild = brain(sim, ctx).is_rebuild;
    sim.player_on_structure_construction_complete(goal, is_rebuild);
    sim.dispatch_on_build_complete(goal);
    sim.handle_partition_cell_maintenance(goal);
    sim.update_upgrade_modules(goal);

    let end = brain(sim, ctx).dock_points[task][DOCK_POINT_END];
    let mut pos = if end.valid != FALSE {
        end.location
    } else {
        position(sim, me)
    };
    sim.pathfinder_adjust_to_possible_destination(me, &mut pos);
    move_self(sim, ctx, &pos);
    true
}

// -----------------------------------------------------------------------------
// The class
// -----------------------------------------------------------------------------

/// The task check of `DozerAIUpdate::update` and `WorkerAIUpdate::update`: a task
/// that is no longer possible is cancelled.
pub(super) fn check_task(sim: &mut dyn SimApi, ctx: ModuleContext, task: Int) {
    let target = brain(sim, ctx).tasks[task as usize].target;
    let target = sim.find_object(target);
    let source = ai(sim, ctx).last_command_source;
    let invalid = (task == DOZER_TASK_REPAIR
        && can_repair_object(sim, ctx.owner_slot, target, source) == FALSE)
        || (cfg!(not(feature = "retail-crc")) && task == DOZER_TASK_BUILD && target.is_none());
    if invalid {
        cancel_task(sim, ctx, task, false);
    }
}

/// `DozerAIUpdate::update`: the base update's answer is dropped.
pub fn update(sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
    let _ = ai_update::update(sim, ctx);
    if dead(sim, ctx.owner_slot) {
        return UpdateSleepTime::NONE;
    }
    let task = brain(sim, ctx).current_task;
    if task != DOZER_TASK_INVALID {
        check_task(sim, ctx, task);
    } else {
        mine_clearing_weapon(sim, ctx, TRUE);
    }
    update_machine(sim, ctx);
    UpdateSleepTime::NONE
}

/// The start of `aiDoCommand`: `false` when the order is refused.
pub(super) fn takes_command(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    parms: &AiCommandParms,
) -> bool {
    sim.clear_model_condition_state(ctx.owner_slot, ModelConditionFlagType::ActivelyConstructing);
    ai_update::is_allowed_to_respond(sim, ctx, parms)
}

/// The repair, resume-construction and default cases of `aiDoCommand`.
pub(super) fn command(sim: &mut dyn SimApi, ctx: ModuleContext, parms: &AiCommandParms) {
    let target = parms.obj.and_then(|id| sim.find_object(id));
    match parms.cmd {
        AiCommandType::Repair | AiCommandType::ResumeConstruction => {
            if brain(sim, ctx).current_task == DOZER_TASK_INVALID {
                idle_self(sim, ctx);
            }
            if parms.cmd == AiCommandType::Repair {
                private_repair(sim, ctx, target, parms.source);
            } else {
                private_resume_construction(sim, ctx, target, parms.source);
            }
        }
        _ => {
            let from_player = parms.source == CommandSourceType::FromPlayer;
            let task = brain(sim, ctx).current_task;
            if from_player && task != DOZER_TASK_INVALID {
                cancel_task(sim, ctx, task, false);
            }
            ai_update::base_command(sim, ctx, parms);
            if from_player {
                reset_machine(sim, ctx);
            }
        }
    }
}

/// `DozerAIUpdate::aiDoCommand`. A working dozer only moves aside for a dozer.
pub fn do_command(sim: &mut dyn SimApi, ctx: ModuleContext, parms: &AiCommandParms) {
    if !takes_command(sim, ctx, parms) {
        return;
    }
    if parms.cmd == AiCommandType::MoveAwayFromUnit {
        let other_is_dozer = parms
            .obj
            .and_then(|id| sim.find_object(id))
            .is_some_and(|o| sim.object_is_kind_of(o, KindOfType::Dozer) != FALSE);
        if other_is_dozer || brain(sim, ctx).current_task == DOZER_TASK_INVALID {
            ai_update::base_command(sim, ctx, parms);
        }
        return;
    }
    command(sim, ctx, parms);
}

/// `onDisabledEdge` of the dozer and of the worker.
pub fn on_disabled_edge(sim: &mut dyn SimApi, ctx: ModuleContext, now_disabled: Bool) {
    if now_disabled != FALSE {
        let task = brain(sim, ctx).current_task;
        if task != DOZER_TASK_INVALID {
            let remember = sim.object(ctx.owner_slot).is_some_and(|o| {
                [
                    DisabledType::Emp,
                    DisabledType::Hacked,
                    DisabledType::Subdued,
                    DisabledType::Underpowered,
                ]
                .iter()
                .any(|&d| o.is_disabled_by_type(d) != FALSE)
            });
            cancel_task(sim, ctx, task, remember);
        }
    } else if cfg!(not(feature = "retail-crc")) {
        resume_previous_task(sim, ctx);
    }
}

/// `onDelete` of the dozer and of the worker.
pub fn on_delete(sim: &mut dyn SimApi, ctx: ModuleContext) {
    for task in 0..DOZER_NUM_TASKS as Int {
        if is_task_pending(brain(sim, ctx), task) {
            cancel_task(sim, ctx, task, false);
        }
    }
    for task in 0..DOZER_NUM_TASKS {
        let target = brain(sim, ctx).tasks[task].target;
        if let Some(goal) = sim.find_object(target) {
            sim.clear_model_condition_state(goal, ModelConditionFlagType::ActivelyBeingConstructed);
        }
    }
}

/// The members `DozerAIUpdate` adds to its base.
#[derive(Debug, Clone)]
pub struct DozerAi {
    pub data: Arc<DozerAIUpdateModuleData>,
    pub brain: DozerBrain,
}

pub fn create_dozer_ai_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let data: &dyn Any = data.as_any();
    let typed = data
        .downcast_ref::<DozerAIUpdateModuleData>()
        .expect("DozerAIUpdate data type mismatch");
    let mut ai = AiUpdate::new(Arc::new(typed.ai.clone()));
    ai.derived = DerivedAi::Dozer(Box::new(DozerAi {
        data: Arc::new(typed.clone()),
        brain: DozerBrain::default(),
    }));
    Box::new(ai)
}
