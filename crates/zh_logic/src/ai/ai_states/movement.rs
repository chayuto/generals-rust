//! `PKT-429`: the movement states of `AIStates.cpp`: `AIInternalMoveToState`,
//! `AIMoveToState`, `AIMoveOutOfTheWayState`, `AIMoveAndTightenState`,
//! `AIFollowPathState` (both ids) and `AIAttackMoveToState` with its sub
//! machine. Packet doc: `packets/phase6/PKT-429_AIStates.md`. Golden test:
//! `tests/golden_s10_movement_states.rs`.
//!
//! A state object lives as long as its machine, so what one stay in a state
//! leaves behind is there at the next one. The sounds of `startMoveSound` are
//! the client's; nothing here draws for them. The attack state of the attack-move
//! sub machine is [`crate::ai::ai_states::attack`], which also drives its approach
//! through `AIInternalMoveToState` here; crate pickup (S11) is answered by
//! [`crate::ai::seams::AiSeams`].

use crate::ai::ai_state_machine::state_id;
use crate::ai::ai_states::attack::{self, AttackState, Host};
use crate::ai::ai_states::IdleState;
use crate::ai::seams::{ai_seams, SeamsGuard};
use crate::arena::{ModuleContext, ObjectSlot};
use crate::modules::module::CommandSourceType;
use crate::modules::update::ai_update::{
    self, ai, AiCommandParms, AiCommandType, MoodMatrixAction, FAST_AS_POSSIBLE,
};
use crate::modules::update::physics_update;
use crate::object::weapon::NO_MAX_SHOTS_LIMIT;
use crate::pathfinder::pathfind_cell::CellType;
use crate::pathfinder::pathfind_constants::{PathfindLayerEnum, PATHFIND_CELL_SIZE_F};
use crate::rules::kind_of::KindOfType;
use crate::rules::model_state::ModelConditionFlagType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::SimApi;
use zh_common::core::state_machine::{StateExitType, StateId, StateReturnType, INVALID_STATE_ID};
use zh_common::ini::LOGICFRAMES_PER_SECOND;
use zh_common::math::{Coord2D, Coord3D};
use zh_common::types::{Bool, Int, ObjectId, Real, UnsignedInt, XferVersion, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

const SECOND: UnsignedInt = LOGICFRAMES_PER_SECOND as UnsignedInt;
/// `AIInternalMoveToState::MIN_REPATH_TIME`.
const MIN_REPATH_TIME: UnsignedInt = 10;
/// `AIAttackMoveToState::ATTACK_RETRY_COUNT`.
const ATTACK_RETRY_COUNT: Int = 5;
/// `AIAttackMoveToState::ATTACK_CLOSE_ENOUGH_CELLS`.
const ATTACK_CLOSE_ENOUGH_CELLS: Int = 8;
/// `MAA_Action_To_AttackMove`.
const MAA_ACTION_TO_ATTACK_MOVE: UnsignedInt = 0x04;

/// Whether this module ports the state.
pub fn is_movement_state(id: StateId) -> bool {
    matches!(
        id,
        state_id::MOVE_TO
            | state_id::ATTACK_MOVE_TO
            | state_id::MOVE_OUT_OF_THE_WAY
            | state_id::MOVE_AND_TIGHTEN
            | state_id::FOLLOW_PATH
            | state_id::FOLLOW_EXITPRODUCTION_PATH
    )
}

/// The members of `AIInternalMoveToState`.
#[derive(Debug, Clone, Copy)]
pub struct InternalMove {
    pub goal_position: Coord3D,
    pub goal_layer: PathfindLayerEnum,
    pub waiting_for_path: Bool,
    pub path_goal_position: Coord3D,
    pub path_timestamp: UnsignedInt,
    pub blocked_repath_timestamp: UnsignedInt,
    pub adjust_destinations: Bool,
    /// `m_tryOneMoreRepath`; not saved.
    pub try_one_more_repath: Bool,
}

impl Default for InternalMove {
    fn default() -> Self {
        Self {
            goal_position: Coord3D::default(),
            goal_layer: PathfindLayerEnum::Invalid,
            waiting_for_path: FALSE,
            path_goal_position: Coord3D::default(),
            path_timestamp: 0,
            blocked_repath_timestamp: 0,
            adjust_destinations: TRUE,
            try_one_more_repath: FALSE,
        }
    }
}

impl InternalMove {
    /// `AIInternalMoveToState::xfer`.
    pub(super) fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        xfer.xfer_coord3d(&mut self.goal_position)?;
        // `xferUser` of the enum. Nothing changes it from `LAYER_INVALID`.
        let mut layer = self.goal_layer as Int;
        xfer.xfer_int(&mut layer)?;
        xfer.xfer_bool(&mut self.waiting_for_path)?;
        xfer.xfer_coord3d(&mut self.path_goal_position)?;
        xfer.xfer_unsigned_int(&mut self.path_timestamp)?;
        xfer.xfer_unsigned_int(&mut self.blocked_repath_timestamp)?;
        xfer.xfer_bool(&mut self.adjust_destinations)
    }

    /// `forceRepath`.
    fn force_repath(&mut self) {
        self.path_goal_position = Coord3D::new(-100.0, -100.0, -100.0);
        self.path_timestamp = 0u32.wrapping_sub(MIN_REPATH_TIME);
    }
}

/// The members of an `AIFollowPathState`.
#[derive(Debug, Clone, Copy)]
pub struct FollowPathData {
    pub base: InternalMove,
    pub index: Int,
    pub adjust_final: Bool,
    pub adjust_final_override: Bool,
    /// `m_retryCount`; not saved and never refilled.
    pub retry_count: Int,
}

impl Default for FollowPathData {
    fn default() -> Self {
        Self {
            base: InternalMove::default(),
            index: 0,
            adjust_final: TRUE,
            adjust_final_override: FALSE,
            retry_count: 10,
        }
    }
}

/// The members of an `AIMoveAndTightenState`.
#[derive(Debug, Clone, Copy, Default)]
pub struct TightenData {
    pub base: InternalMove,
    pub ok_to_repath_times: Int,
    pub check_for_path: Bool,
}

/// The `AIAttackMoveStateMachine`: idle, crate pickup and attack. Its idle
/// state's members are `AiStateMachine::attack_move_idle[0]`.
#[derive(Debug, Clone, Copy)]
pub struct SubMachine {
    pub sleep_till: UnsignedInt,
    pub current: StateId,
    pub goal_object_id: ObjectId,
    pub goal_position: Coord3D,
    pub locked: Bool,
    pub default_state_inited: Bool,
}

impl Default for SubMachine {
    fn default() -> Self {
        Self {
            sleep_till: 0,
            current: INVALID_STATE_ID,
            goal_object_id: ObjectId::INVALID,
            goal_position: Coord3D::default(),
            locked: FALSE,
            default_state_inited: FALSE,
        }
    }
}

impl SubMachine {
    /// `isInIdleState`.
    pub fn is_in_idle_state(&self) -> bool {
        self.current == INVALID_STATE_ID || self.current == state_id::IDLE
    }

    /// `isInAttackState`: a machine without a state says yes.
    pub fn is_in_attack_state(&self) -> bool {
        self.current == INVALID_STATE_ID || self.current == state_id::ATTACK_OBJECT
    }
}

/// The members of an `AIAttackMoveToState`.
#[derive(Debug, Clone, Copy)]
pub struct AttackMoveData {
    pub base: InternalMove,
    pub command_source: CommandSourceType,
    pub frame_to_sleep_until: UnsignedInt,
    pub retry_count: Int,
    pub sub: SubMachine,
}

impl Default for AttackMoveData {
    fn default() -> Self {
        Self {
            base: InternalMove::default(),
            command_source: CommandSourceType::FromPlayer,
            frame_to_sleep_until: 0,
            retry_count: ATTACK_RETRY_COUNT,
            sub: SubMachine::default(),
        }
    }
}

/// The movement state objects of an `AIStateMachine`.
#[derive(Debug, Clone, Default)]
pub struct MoveStates {
    pub move_to: InternalMove,
    pub out_of_way: InternalMove,
    pub tighten: TightenData,
    pub follow_path: FollowPathData,
    pub exit_path: FollowPathData,
    pub attack_move: AttackMoveData,
}

impl MoveStates {
    /// What the constructor of `AIAttackMoveToState` does to its sub machine:
    /// `initDefaultState`. The caller runs the idle state's `onEnter`.
    pub fn init_attack_move_machine(&mut self) {
        self.attack_move.sub.current = state_id::IDLE;
        self.attack_move.sub.default_state_inited = TRUE;
    }

    fn internal(&mut self, id: StateId) -> &mut InternalMove {
        match id {
            state_id::ATTACK_MOVE_TO => &mut self.attack_move.base,
            state_id::MOVE_OUT_OF_THE_WAY => &mut self.out_of_way,
            state_id::MOVE_AND_TIGHTEN => &mut self.tighten.base,
            state_id::FOLLOW_PATH => &mut self.follow_path.base,
            state_id::FOLLOW_EXITPRODUCTION_PATH => &mut self.exit_path.base,
            _ => &mut self.move_to,
        }
    }

    fn follow(&mut self, id: StateId) -> &mut FollowPathData {
        if id == state_id::FOLLOW_EXITPRODUCTION_PATH {
            &mut self.exit_path
        } else {
            &mut self.follow_path
        }
    }

    /// `State::xfer` of a movement state. `idle` and `attack` are states of the
    /// attack-move sub machine.
    pub fn xfer(
        &mut self,
        id: StateId,
        idle: &mut IdleState,
        attack: &mut AttackState,
        xfer: &mut dyn Xfer,
        owner: ObjectId,
        seams: &mut SeamsGuard,
    ) -> Result<(), XferError> {
        match id {
            state_id::MOVE_AND_TIGHTEN => {
                let mut version: XferVersion = 1;
                xfer.xfer_version(&mut version, 1)?;
                self.tighten.base.xfer(xfer)?;
                xfer.xfer_int(&mut self.tighten.ok_to_repath_times)?;
                xfer.xfer_bool(&mut self.tighten.check_for_path)
            }
            state_id::FOLLOW_PATH | state_id::FOLLOW_EXITPRODUCTION_PATH => {
                let data = self.follow(id);
                let mut version: XferVersion = 1;
                xfer.xfer_version(&mut version, 1)?;
                data.base.xfer(xfer)?;
                xfer.xfer_int(&mut data.index)?;
                xfer.xfer_bool(&mut data.adjust_final)?;
                xfer.xfer_bool(&mut data.adjust_final_override)
            }
            state_id::ATTACK_MOVE_TO => {
                let data = &mut self.attack_move;
                let mut version: XferVersion = 2;
                xfer.xfer_version(&mut version, 2)?;
                data.base.xfer(xfer)?;
                xfer.xfer_unsigned_int(&mut data.frame_to_sleep_until)?;
                xfer.xfer_int(&mut data.retry_count)?;

                let sub = &mut data.sub;
                let mut machine_version: XferVersion = 1;
                xfer.xfer_version(&mut machine_version, 1)?;
                let mut base_version: XferVersion = 1;
                xfer.xfer_version(&mut base_version, 1)?;
                xfer.xfer_unsigned_int(&mut sub.sleep_till)?;
                let mut default_state = state_id::IDLE;
                xfer.xfer_unsigned_int(&mut default_state)?;
                let mut current = sub.current;
                xfer.xfer_unsigned_int(&mut current)?;
                let mut snapshot_all = FALSE;
                xfer.xfer_bool(&mut snapshot_all)?;
                if sub.current == INVALID_STATE_ID {
                    sub.current = state_id::IDLE;
                }
                if sub.current == state_id::IDLE {
                    idle.xfer(xfer)?;
                } else if sub.current == state_id::ATTACK_OBJECT
                    && seams.replaces_state(owner, sub.current) == FALSE
                {
                    attack.xfer(xfer)?;
                } else {
                    seams.state_xfer(owner, sub.current, xfer)?;
                }
                xfer.xfer_object_id(&mut sub.goal_object_id)?;
                xfer.xfer_coord3d(&mut sub.goal_position)?;
                xfer.xfer_bool(&mut sub.locked)?;
                xfer.xfer_bool(&mut sub.default_state_inited)
            }
            _ => self.internal(id).xfer(xfer),
        }
    }
}

fn moves(sim: &mut dyn SimApi, ctx: ModuleContext) -> &mut MoveStates {
    &mut ai(sim, ctx).machine.moves
}

fn data(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> &mut InternalMove {
    match attack::split_move_key(id) {
        Some((host, state)) => ai(sim, ctx).machine.attacks.internal_move(host, state),
        None => moves(sim, ctx).internal(id),
    }
}

// -----------------------------------------------------------------------------
// The unit and the map
// -----------------------------------------------------------------------------

fn has_status(sim: &dyn SimApi, owner: ObjectSlot, status: ObjectStatusType) -> bool {
    sim.object(owner)
        .is_some_and(|o| o.status.test(status as Int) != FALSE)
}

fn position(sim: &dyn SimApi, owner: ObjectSlot) -> Coord3D {
    sim.object(owner)
        .map(|o| *o.thing.get_position())
        .unwrap_or_default()
}

fn set_flag(sim: &mut dyn SimApi, owner: ObjectSlot, flag: ModelConditionFlagType) {
    sim.set_model_condition_state(owner, flag);
}

fn clear_flag(sim: &mut dyn SimApi, owner: ObjectSlot, flag: ModelConditionFlagType) {
    sim.clear_model_condition_state(owner, flag);
}

/// The type of the cell under the unit and whether it is pinched.
enum Underfoot {
    /// A sim without a pathfind map.
    NoMap,
    OffMap,
    Cell(CellType, bool),
}

fn underfoot(sim: &mut dyn SimApi, owner: ObjectSlot) -> Underfoot {
    let Some((pos, layer)) = sim
        .object(owner)
        .map(|o| (*o.thing.get_position(), o.layer))
    else {
        return Underfoot::OffMap;
    };
    let x = (pos.x / PATHFIND_CELL_SIZE_F) as Int;
    let y = (pos.y / PATHFIND_CELL_SIZE_F) as Int;
    match sim.pathfinder() {
        None => Underfoot::NoMap,
        Some(map) => match map.cell_type_and_pinched(layer, x, y) {
            Some((kind, pinched)) => Underfoot::Cell(kind, pinched),
            None => Underfoot::OffMap,
        },
    }
}

fn is_ultra_accurate(sim: &mut dyn SimApi, ctx: ModuleContext) -> bool {
    ai(sim, ctx)
        .cur_locomotor()
        .is_some_and(|l| l.is_ultra_accurate() != FALSE)
}

fn set_use_precise_z_pos(sim: &mut dyn SimApi, ctx: ModuleContext, precise: Bool) {
    if let Some(locomotor) = ai(sim, ctx).cur_locomotor_mut() {
        locomotor.set_use_precise_z_pos(precise);
    }
}

/// `Pathfinder::removeGoal`.
fn remove_goal(sim: &mut dyn SimApi, ctx: ModuleContext) {
    if sim.pathfinder().is_some() {
        sim.pathfinder_remove_goal(ctx.owner_slot);
    } else {
        ai_seams(sim).remove_goal(ctx.owner_id);
    }
}

/// `adjustDestination` for the unit's own locomotors; `false` when it found
/// no cell.
fn adjust_destination(sim: &mut dyn SimApi, ctx: ModuleContext, goal: &mut Coord3D) -> bool {
    if sim.pathfinder().is_some() {
        return sim.pathfinder_adjust_destination(ctx.owner_slot, goal) != FALSE;
    }
    ai_seams(sim).adjust_destination(ctx.owner_id, goal, None);
    true
}

fn reserve_goal(sim: &mut dyn SimApi, ctx: ModuleContext, goal: &Coord3D) {
    let layer = sim.get_layer_for_destination(goal);
    ai_update::update_goal(sim, ctx, goal, layer);
}

/// The speed of the unit's formation group, if it is in one.
fn take_group_speed(sim: &mut dyn SimApi, ctx: ModuleContext) {
    if let Some(speed) = sim.ai_formation_speed(ctx.owner_slot) {
        ai(sim, ctx).desired_speed = speed;
    }
}

fn last_node(sim: &mut dyn SimApi, ctx: ModuleContext) -> Option<(Coord3D, PathfindLayerEnum)> {
    let path = ai(sim, ctx).path.as_ref()?;
    let node = path.node(path.last_node()?);
    Some((*node.position(), node.layer()))
}

/// `isSamePosition`: the target moved no more than a tenth of its distance.
pub(super) fn is_same_position(ours: &Coord3D, previous: &Coord3D, current: &Coord3D) -> bool {
    let dx = current.x - previous.x;
    let dy = current.y - previous.y;
    let tx = current.x - ours.x;
    let ty = current.y - ours.y;
    let tolerance = (tx * tx + ty * ty) * (1.0 / (10.0 * 10.0));
    !(dx * dx + dy * dy > tolerance)
}

// -----------------------------------------------------------------------------
// `AIInternalMoveToState`
// -----------------------------------------------------------------------------

/// `getAdjustsDestination`.
fn adjusts_destination(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> bool {
    if has_status(sim, ctx.owner_slot, ObjectStatusType::Parachuting) {
        return false;
    }
    // `isAllowedToAdjustDestination` is true but for the Chinook (not ported).
    data(sim, ctx, id).adjust_destinations != FALSE
}

fn set_adjusts_destination(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId, adjusts: bool) {
    data(sim, ctx, id).adjust_destinations = Bool::from(adjusts);
}

/// `computePath`, as each state overrides it.
fn compute_path(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> bool {
    if let Some((host, state)) = attack::split_move_key(id) {
        return attack::compute_path(sim, ctx, host, state);
    }
    match id {
        state_id::MOVE_OUT_OF_THE_WAY => {
            data(sim, ctx, id).waiting_for_path = TRUE;
            if ai(sim, ctx).is_blocked_and_stuck != FALSE {
                ai(sim, ctx).set_can_path_through_units(TRUE);
            }
            true
        }
        state_id::MOVE_AND_TIGHTEN => {
            if ai(sim, ctx).is_blocked_and_stuck == FALSE {
                return true;
            }
            let state = &mut moves(sim, ctx).tighten;
            if state.ok_to_repath_times <= 0 {
                return false;
            }
            state.ok_to_repath_times -= 1;
            state.base.waiting_for_path = TRUE;
            let goal = state.base.goal_position;
            ai_update::request_path(sim, ctx, &goal, TRUE);
            true
        }
        _ => {
            data(sim, ctx, id).waiting_for_path = TRUE;
            let goal = data(sim, ctx, id).goal_position;
            let adjusts = adjusts_destination(sim, ctx, id);
            ai_update::request_path(sim, ctx, &goal, Bool::from(adjusts));
            ai(sim, ctx).starting_move();
            true
        }
    }
}

pub(super) fn internal_on_enter(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> StateReturnType {
    let owner = ctx.owner_slot;
    let waiting = ai(sim, ctx).waiting_for_path;
    data(sim, ctx, id).waiting_for_path = waiting;

    if has_status(sim, owner, ObjectStatusType::Immobile) {
        return StateReturnType::Failure;
    }

    let now = sim.frame();
    if let Some(locomotor) = ai(sim, ctx).cur_locomotor_mut() {
        locomotor.start_move(now);
    }
    if is_ultra_accurate(sim, ctx) {
        set_adjusts_destination(sim, ctx, id, false);
    }
    data(sim, ctx, id).try_one_more_repath = TRUE;
    ai(sim, ctx).starting_move();

    if physics_update::is_motive(sim, owner) {
        let flag = match underfoot(sim, owner) {
            Underfoot::NoMap => ModelConditionFlagType::Moving,
            Underfoot::Cell(kind, _) if kind != CellType::Cliff => ModelConditionFlagType::Moving,
            _ => ModelConditionFlagType::Climbing,
        };
        set_flag(sim, owner, flag);
    } else if sim.object_is_kind_of(owner, KindOfType::Dozer) != FALSE
        && sim.object_is_kind_of(owner, KindOfType::Harvester) != FALSE
    {
        set_flag(sim, owner, ModelConditionFlagType::Moving);
    }

    if adjusts_destination(sim, ctx, id) && !has_status(sim, owner, ObjectStatusType::Rider8) {
        let mut goal = data(sim, ctx, id).goal_position;
        if !adjust_destination(sim, ctx, &mut goal) && sim.pathfinder().is_some() {
            sim.pathfinder_snap_closest_goal_position(owner, &mut goal);
        }
        data(sim, ctx, id).goal_position = goal;
        reserve_goal(sim, ctx, &goal);
    }

    if !compute_path(sim, ctx, id) {
        ai(sim, ctx).ending_move();
        return StateReturnType::Failure;
    }

    let unit = ai(sim, ctx);
    unit.path_extra_distance = 0.0;
    unit.desired_speed = FAST_AS_POSSIBLE;
    StateReturnType::Continue
}

pub(super) fn internal_on_exit(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) {
    ai(sim, ctx).ending_move();
    // Every object has a team here.
    if ai_update::is_doing_ground_movement(sim, ctx) != FALSE && is_ultra_accurate(sim, ctx) {
        let goal = data(sim, ctx, id).goal_position;
        let pos = position(sim, ctx.owner_slot);
        let dx = goal.x - pos.x;
        let dy = goal.y - pos.y;
        if dx * dx + dy * dy < PATHFIND_CELL_SIZE_F * PATHFIND_CELL_SIZE_F {
            ai(sim, ctx).set_final_position(&goal);
        }
    }
}

pub(super) fn internal_update(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> StateReturnType {
    use ModelConditionFlagType as M;
    let owner = ctx.owner_slot;
    let had_path = ai(sim, ctx).has_path != FALSE;

    if data(sim, ctx, id).waiting_for_path != FALSE {
        let now = sim.frame();
        data(sim, ctx, id).path_timestamp = now;
        if ai(sim, ctx).waiting_for_path != FALSE {
            return StateReturnType::Continue;
        }
        if !had_path {
            return StateReturnType::Failure;
        }
        let state = data(sim, ctx, id);
        state.waiting_for_path = FALSE;
        state.path_goal_position = state.goal_position;
        if adjusts_destination(sim, ctx, id) {
            if let Some((pos, layer)) = last_node(sim, ctx) {
                ai_update::update_goal(sim, ctx, &pos, layer);
            }
        } else {
            remove_goal(sim, ctx);
        }
        if ai(sim, ctx).retry_path == FALSE {
            data(sim, ctx, id).try_one_more_repath = FALSE;
        }
    }

    let mut force_recompute = !had_path;

    let unit = ai(sim, ctx);
    if unit.is_blocked_and_stuck != FALSE || unit.blocked_frames > 2 * SECOND {
        force_recompute = true;
        let now = sim.frame();
        data(sim, ctx, id).blocked_repath_timestamp = now;
    }

    let mut condition = M::Moving;
    if let Underfoot::Cell(CellType::Cliff, false) = underfoot(sim, owner) {
        let backwards = ai(sim, ctx)
            .cur_locomotor()
            .is_some_and(|l| l.is_moving_backwards() != FALSE);
        if backwards {
            condition = M::Rappelling;
            clear_flag(sim, owner, M::Climbing);
        } else {
            condition = M::Climbing;
            clear_flag(sim, owner, M::Rappelling);
        }
    }
    if ai(sim, ctx).blocked_frames as Real > SECOND as Real / 4.0 {
        clear_flag(sim, owner, M::Moving);
    } else {
        if condition == M::Moving {
            clear_flag(sim, owner, M::Climbing);
            clear_flag(sim, owner, M::Rappelling);
        }
        set_flag(sim, owner, M::Moving);
        if condition != M::Moving {
            set_flag(sim, owner, condition);
        }
    }

    if ai_update::can_compute_quick_path(sim, ctx)
        && sim.object_is_kind_of(owner, KindOfType::Projectile) != FALSE
    {
        data(sim, ctx, id).path_timestamp = 0;
        force_recompute = true;
    }

    if had_path {
        ai(sim, ctx).set_goal_position_on_path();
    }

    let now = sim.frame();
    if force_recompute || now.wrapping_sub(data(sim, ctx, id).path_timestamp) > MIN_REPATH_TIME {
        let pos = position(sim, owner);
        let state = data(sim, ctx, id);
        if force_recompute
            || !is_same_position(&pos, &state.path_goal_position, &state.goal_position)
        {
            if !compute_path(sim, ctx, id) {
                return StateReturnType::Failure;
            }
            if ai(sim, ctx).has_path == FALSE {
                return StateReturnType::Continue;
            }
            ai(sim, ctx).set_goal_position_on_path();
        }
    }

    let distance = ai_update::locomotor_distance_to_goal(sim, ctx);
    let close_enough = ai(sim, ctx)
        .cur_locomotor()
        .is_some_and(|l| distance < l.close_enough_dist);
    if close_enough {
        if ai_update::is_doing_ground_movement(sim, ctx) != FALSE {
            let mut goal = data(sim, ctx, id).goal_position;
            if let Some((pos, _)) = last_node(sim, ctx) {
                goal = pos;
            }
            let pos = position(sim, owner);
            let delta = Coord3D::new(pos.x - goal.x, pos.y - goal.y, 0.0);
            if delta.length() > 4.0 * PATHFIND_CELL_SIZE_F {
                let _ = ai_update::locomotor_distance_to_goal(sim, ctx);
                return StateReturnType::Continue;
            }
        }
        if adjusts_destination(sim, ctx, id) {
            ai_update::set_locomotor_goal_none(sim, ctx);
        }
        return StateReturnType::Success;
    }
    StateReturnType::Continue
}

// -----------------------------------------------------------------------------
// `AIMoveToState`
// -----------------------------------------------------------------------------

/// The goal object of the unit's own machine.
fn goal_object(sim: &mut dyn SimApi, ctx: ModuleContext) -> Option<(ObjectId, ObjectSlot)> {
    let id = ai(sim, ctx).machine.goal_object_id;
    sim.find_object(id).map(|slot| (id, slot))
}

fn half_height(sim: &dyn SimApi, slot: ObjectSlot) -> Real {
    sim.object(slot)
        .map_or(0.0, |o| o.geometry.get_max_height_above_position())
        / 2.0
}

fn move_to_on_enter(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> StateReturnType {
    set_adjusts_destination(sim, ctx, id, true);
    let goal_obj = goal_object(sim, ctx);
    if let Some((goal_id, _)) = goal_obj {
        if goal_id == ai(sim, ctx).ignore_obstacle_id {
            set_adjusts_destination(sim, ctx, id, false);
        }
    }
    let goal = match goal_obj {
        Some((_, slot)) => {
            let mut goal = position(sim, slot);
            if sim.object_is_kind_of(ctx.owner_slot, KindOfType::Projectile) != FALSE {
                let half = half_height(sim, slot);
                goal.z += half;
                if position(sim, slot).z < goal.z {
                    goal.z += half;
                }
            }
            goal
        }
        None => ai(sim, ctx).machine.goal_position,
    };
    data(sim, ctx, id).goal_position = goal;

    let result = internal_on_enter(sim, ctx, id);
    take_group_speed(sim, ctx);
    result
}

/// A projectile aims ahead of a goal that can move.
fn lead_goal(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId, goal_slot: ObjectSlot) {
    let owner = ctx.owner_slot;
    let mut goal = position(sim, goal_slot);
    let got_physics = physics_update::physics(sim, owner).is_some()
        && physics_update::physics(sim, goal_slot).is_some();
    let missile = sim.object_is_kind_of(owner, KindOfType::Projectile) != FALSE;
    if missile {
        goal.z += half_height(sim, goal_slot);
        let rise = goal.z - position(sim, owner).z;
        if rise > 0.0 {
            goal.z += rise;
        }
    }
    if got_physics && missile && sim.object_is_kind_of(goal_slot, KindOfType::Immobile) == FALSE {
        let ours = position(sim, owner);
        let delta = Coord3D::new(goal.x - ours.x, goal.y - ours.y, goal.z - ours.z);
        let mut speed = physics_update::velocity_magnitude(sim, owner);
        let goal_speed = physics_update::velocity_magnitude(sim, goal_slot);
        if speed < 5.0 {
            speed = 5.0;
        }
        // The reference does this in double precision.
        let lead =
            (0.5f64 * f64::from(delta.length()) * f64::from(goal_speed) / f64::from(speed)) as Real;
        let dir = sim
            .object(goal_slot)
            .map(|o| o.thing.get_unit_direction_vector_3d())
            .unwrap_or_default();
        goal.x += dir.x * lead;
        goal.y += dir.y * lead;
        goal.z += dir.z * lead;
    }
    data(sim, ctx, id).goal_position = goal;
}

fn move_to_update(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> StateReturnType {
    let adjustment = ai_update::mood_matrix_action_adjustment(sim, ctx, MoodMatrixAction::Move);
    if id == state_id::MOVE_TO && adjustment & MAA_ACTION_TO_ATTACK_MOVE != 0 {
        // The order leaves this state, which still runs to its end.
        let mut parms = AiCommandParms::new(
            AiCommandType::AttackMoveToPosition,
            CommandSourceType::FromAi,
        );
        parms.pos = data(sim, ctx, id).goal_position;
        parms.int_value = NO_MAX_SHOTS_LIMIT;
        ai_update::execute_command(sim, ctx, &parms);
    }

    match goal_object(sim, ctx) {
        Some((_, slot)) => lead_goal(sim, ctx, id, slot),
        None => {
            if sim.object_is_kind_of(ctx.owner_slot, KindOfType::Projectile) != FALSE {
                let mut goal = ai(sim, ctx).machine.goal_position;
                let rise = goal.z - position(sim, ctx.owner_slot).z;
                if rise > 0.0 {
                    goal.z += rise;
                }
                data(sim, ctx, id).goal_position = goal;
            }
        }
    }
    internal_update(sim, ctx, id)
}

// -----------------------------------------------------------------------------
// `AIFollowPathState`
// -----------------------------------------------------------------------------

fn path_point(sim: &mut dyn SimApi, ctx: ModuleContext, index: Int) -> Option<Coord3D> {
    let index = usize::try_from(index).ok()?;
    ai(sim, ctx).machine.goal_path.get(index).copied()
}

/// The distance to drive past a point that has another one behind it.
fn extra_distance(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    index: Int,
    from: &Coord3D,
    to: &Coord3D,
) {
    let delta = Coord2D {
        x: to.x - from.x,
        y: to.y - from.y,
    };
    let mut offset = delta.length();
    if path_point(sim, ctx, index.wrapping_add(2)).is_some() {
        offset += 4.0 * PATHFIND_CELL_SIZE_F;
    }
    ai(sim, ctx).path_extra_distance = offset;
}

fn follow_on_enter(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> StateReturnType {
    moves(sim, ctx).follow(id).index = 0;
    let Some(pos) = path_point(sim, ctx, 0) else {
        return StateReturnType::Failure;
    };
    let state = moves(sim, ctx).follow(id);
    state.base.goal_position = pos;
    state.adjust_final = TRUE;
    let next = path_point(sim, ctx, 1);

    if id == state_id::FOLLOW_EXITPRODUCTION_PATH {
        ai(sim, ctx).set_can_path_through_units(TRUE);
        set_adjusts_destination(sim, ctx, id, false);
        moves(sim, ctx).follow(id).adjust_final = TRUE;
    }

    let result = internal_on_enter(sim, ctx, id);
    take_group_speed(sim, ctx);

    match next {
        Some(next) => {
            extra_distance(sim, ctx, 0, &pos, &next);
            set_adjusts_destination(sim, ctx, id, false);
        }
        None => {
            let adjust = moves(sim, ctx).follow(id).adjust_final != FALSE;
            set_adjusts_destination(sim, ctx, id, adjust);
            ai(sim, ctx).path_extra_distance = 0.0;
            if sim.object_is_kind_of(ctx.owner_slot, KindOfType::Projectile) != FALSE {
                set_use_precise_z_pos(sim, ctx, TRUE);
            }
        }
    }
    result
}

fn follow_on_exit(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) {
    internal_on_exit(sim, ctx, id);
    ai(sim, ctx).set_can_path_through_units(FALSE);
    set_use_precise_z_pos(sim, ctx, FALSE);
}

fn follow_update(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> StateReturnType {
    let goal = data(sim, ctx, id).goal_position;
    ai(sim, ctx).machine.set_goal_position(Some(&goal));

    let status = internal_update(sim, ctx, id);
    if status != StateReturnType::Success && status != StateReturnType::Failure {
        return status;
    }
    let state = moves(sim, ctx).follow(id);
    if status == StateReturnType::Failure && state.retry_count > 0 {
        state.retry_count -= 1;
    } else {
        state.index = state.index.wrapping_add(1);
    }

    let mut index = state.index;
    let mut point = path_point(sim, ctx, index);
    while let Some(pos) = point {
        let ours = position(sim, ctx.owner_slot);
        let dx = pos.x - ours.x;
        let dy = pos.y - ours.y;
        if !(dx * dx + dy * dy < PATHFIND_CELL_SIZE_F * PATHFIND_CELL_SIZE_F) {
            break;
        }
        index = index.wrapping_add(1);
        point = path_point(sim, ctx, index);
    }
    moves(sim, ctx).follow(id).index = index;
    // `friend_setCurrentGoalPathIndex` only feeds the client's path drawing.
    ai(sim, ctx).ignore_obstacle_id = ObjectId::INVALID;
    let Some(pos) = point else {
        return StateReturnType::Success;
    };
    ai(sim, ctx).starting_move();
    data(sim, ctx, id).goal_position = pos;

    match path_point(sim, ctx, index.wrapping_add(1)) {
        Some(next) => {
            extra_distance(sim, ctx, index, &pos, &next);
            set_adjusts_destination(sim, ctx, id, false);
        }
        None => {
            let state = moves(sim, ctx).follow(id);
            let (adjust_final, anyway) = (state.adjust_final, state.adjust_final_override);
            let adjust = adjust_final != FALSE
                && (anyway != FALSE || ai_update::is_doing_ground_movement(sim, ctx) != FALSE);
            set_adjusts_destination(sim, ctx, id, adjust);
            if adjusts_destination(sim, ctx, id) {
                let mut goal = data(sim, ctx, id).goal_position;
                let found = adjust_destination(sim, ctx, &mut goal);
                data(sim, ctx, id).goal_position = goal;
                if !found {
                    return StateReturnType::Failure;
                }
                reserve_goal(sim, ctx, &goal);
            }
            if sim.object_is_kind_of(ctx.owner_slot, KindOfType::Projectile) != FALSE {
                set_use_precise_z_pos(sim, ctx, TRUE);
            }
        }
    }
    let _ = compute_path(sim, ctx, id);
    StateReturnType::Continue
}

// -----------------------------------------------------------------------------
// `AIAttackMoveStateMachine`
// -----------------------------------------------------------------------------

fn sub(sim: &mut dyn SimApi, ctx: ModuleContext) -> &mut SubMachine {
    &mut moves(sim, ctx).attack_move.sub
}

/// The attack state of the sub machine, unless the seams stand in for it.
fn is_ported_attack(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> bool {
    id == state_id::ATTACK_OBJECT && ai_seams(sim).replaces_state(ctx.owner_id, id) == FALSE
}

fn sub_on_enter(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> StateReturnType {
    if is_ported_attack(sim, ctx, id) {
        return attack::on_enter(sim, ctx, Host::AttackMove, id);
    }
    if id != state_id::IDLE {
        return ai_seams(sim).state_enter(ctx.owner_id, id);
    }
    ai_update::reset_next_mood_check_time(sim, ctx);
    ai(sim, ctx).machine.attack_move_idle[0].inited = TRUE;
    let offset = crate::ai::ai_states::idle_on_enter(sim);
    ai(sim, ctx).machine.attack_move_idle[0].initial_sleep_offset = offset;
    StateReturnType::Continue
}

fn sub_on_exit(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId, status: StateExitType) {
    if is_ported_attack(sim, ctx, id) {
        attack::on_exit(sim, ctx, Host::AttackMove, status);
    } else if id != state_id::IDLE {
        ai_seams(sim).state_exit(ctx.owner_id, id, status);
    }
}

/// `internalSetState`. All three states lead back to idle.
fn sub_internal_set_state(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    new_id: StateId,
    depth: u32,
) -> StateReturnType {
    sub(sim, ctx).sleep_till = 0;
    if !matches!(
        new_id,
        state_id::IDLE | state_id::PICK_UP_CRATE | state_id::ATTACK_OBJECT
    ) {
        return StateReturnType::Failure;
    }
    let old = sub(sim, ctx).current;
    if old != INVALID_STATE_ID {
        sub_on_exit(sim, ctx, old, StateExitType::Normal);
    }
    sub(sim, ctx).current = new_id;
    let status = sub_on_enter(sim, ctx, new_id);
    sub_finish(sim, ctx, new_id, status, depth)
}

/// What follows `onEnter` and `update` of a sub state.
fn sub_finish(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    ran: StateId,
    mut status: StateReturnType,
    depth: u32,
) -> StateReturnType {
    let current = sub(sim, ctx).current;
    if current == INVALID_STATE_ID {
        return StateReturnType::Failure;
    }
    if current != ran {
        status = StateReturnType::Continue;
    }
    match status {
        StateReturnType::Sleep(frames) => {
            let now = sim.frame();
            sub(sim, ctx).sleep_till = now.wrapping_add(frames);
            status
        }
        StateReturnType::Success | StateReturnType::Failure => {
            if depth + 1 >= zh_common::core::state_machine::MAX_TRANSITION_DEPTH {
                return StateReturnType::Failure;
            }
            sub_internal_set_state(sim, ctx, state_id::IDLE, depth + 1)
        }
        other => other,
    }
}

/// `StateMachine::setState`.
fn sub_set_state(sim: &mut dyn SimApi, ctx: ModuleContext, new_id: StateId) {
    if sub(sim, ctx).locked == FALSE {
        let _ = sub_internal_set_state(sim, ctx, new_id, 0);
    }
}

/// `StateMachine::clear`.
fn sub_clear(sim: &mut dyn SimApi, ctx: ModuleContext) {
    if sub(sim, ctx).locked != FALSE {
        return;
    }
    let old = sub(sim, ctx).current;
    if old != INVALID_STATE_ID {
        sub_on_exit(sim, ctx, old, StateExitType::Reset);
    }
    let machine = sub(sim, ctx);
    machine.current = INVALID_STATE_ID;
    machine.goal_object_id = ObjectId::INVALID;
    machine.goal_position = Coord3D::default();
}

/// `StateMachine::setGoalObject`.
fn sub_set_goal_object(sim: &mut dyn SimApi, ctx: ModuleContext, id: ObjectId) {
    if sub(sim, ctx).locked != FALSE {
        return;
    }
    let Some(slot) = sim.find_object(id) else {
        sub(sim, ctx).goal_object_id = ObjectId::INVALID;
        return;
    };
    let pos = position(sim, slot);
    let machine = sub(sim, ctx);
    machine.goal_object_id = id;
    machine.goal_position = pos;
}

/// `StateMachine::updateStateMachine`.
fn sub_update_machine(sim: &mut dyn SimApi, ctx: ModuleContext) {
    let now = sim.frame();
    let machine = sub(sim, ctx);
    let id = machine.current;
    if machine.sleep_till != 0 && now < machine.sleep_till {
        return;
    }
    machine.sleep_till = 0;
    // Only a machine out of idle is updated.
    if id == INVALID_STATE_ID || id == state_id::IDLE {
        return;
    }
    let status = if is_ported_attack(sim, ctx, id) {
        attack::update(sim, ctx, Host::AttackMove)
    } else {
        ai_seams(sim).state_update(ctx.owner_id, id)
    };
    let _ = sub_finish(sim, ctx, id, status, 0);
}

// -----------------------------------------------------------------------------
// `AIAttackMoveToState`
// -----------------------------------------------------------------------------

fn attack_move_on_enter(sim: &mut dyn SimApi, ctx: ModuleContext) -> StateReturnType {
    sub_clear(sim, ctx);
    sub_set_state(sim, ctx, state_id::IDLE);
    let source = ai(sim, ctx).last_command_source;
    let state = &mut moves(sim, ctx).attack_move;
    state.command_source = source;
    state.retry_count = ATTACK_RETRY_COUNT;
    state.frame_to_sleep_until = 0;
    move_to_on_enter(sim, ctx, state_id::ATTACK_MOVE_TO)
}

fn attack_move_on_exit(sim: &mut dyn SimApi, ctx: ModuleContext) {
    sub_set_state(sim, ctx, state_id::IDLE);
    internal_on_exit(sim, ctx, state_id::ATTACK_MOVE_TO);
}

fn set_last_command_source(sim: &mut dyn SimApi, ctx: ModuleContext, source: CommandSourceType) {
    ai(sim, ctx).last_command_source = source;
    if let Some(o) = sim.object_mut(ctx.owner_slot) {
        o.ai_last_command_source = source;
    }
}

fn attack_move_update(sim: &mut dyn SimApi, ctx: ModuleContext) -> StateReturnType {
    const ID: StateId = state_id::ATTACK_MOVE_TO;
    let owner = ctx.owner_slot;
    let mut retarget_now = false;
    let mut repath_now = false;
    // `JetAIUpdate::isOutOfSpecialReloadAmmo`: the jet AI is not ported.

    if !sub(sim, ctx).is_in_idle_state() {
        ai_update::set_locomotor_goal_none(sim, ctx);
        clear_flag(sim, owner, ModelConditionFlagType::Moving);
        sub_update_machine(sim, ctx);
        if !sub(sim, ctx).is_in_idle_state() {
            return StateReturnType::Continue;
        }
        retarget_now = true;
        repath_now = true;
        let source = moves(sim, ctx).attack_move.command_source;
        set_last_command_source(sim, ctx, source);
    }

    if sub(sim, ctx).is_in_idle_state() {
        if let Some(crate_id) = ai_update::check_for_crate_to_pickup(sim, ctx) {
            sub_set_goal_object(sim, ctx, crate_id);
            sub_set_state(sim, ctx, state_id::PICK_UP_CRATE);
            return StateReturnType::Continue;
        }
        let target = ai_update::next_mood_target(sim, ctx, Bool::from(!retarget_now), FALSE);
        if let Some(target) = target {
            ai(sim, ctx).ending_move();
            sub_set_goal_object(sim, ctx, target);
            sub_set_state(sim, ctx, state_id::ATTACK_OBJECT);
            set_last_command_source(sim, ctx, CommandSourceType::FromAi);
            return StateReturnType::Continue;
        }
    }

    let now = sim.frame();
    let wake = moves(sim, ctx).attack_move.frame_to_sleep_until;
    if wake > now {
        return StateReturnType::Continue;
    } else if wake == now {
        repath_now = true;
    }

    if repath_now {
        let _ = move_to_on_enter(sim, ctx, ID);
        data(sim, ctx, ID).force_repath();
    }

    let mut status = move_to_update(sim, ctx, ID);
    if status != StateReturnType::Continue {
        let state = moves(sim, ctx).attack_move;
        if state.retry_count < 1 {
            return status;
        }
        let pos = position(sim, owner);
        let dx = pos.x - state.base.path_goal_position.x;
        let dy = pos.y - state.base.path_goal_position.y;
        let near = ATTACK_CLOSE_ENOUGH_CELLS as Real * PATHFIND_CELL_SIZE_F;
        if dx * dx + dy * dy < near * near {
            return status;
        }
        status = StateReturnType::Continue;
        let now = sim.frame();
        let state = &mut moves(sim, ctx).attack_move;
        state.retry_count -= 1;
        state.frame_to_sleep_until = now.wrapping_add(3 * SECOND);
    }
    status
}

// -----------------------------------------------------------------------------
// The state table
// -----------------------------------------------------------------------------

pub fn on_enter(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> StateReturnType {
    match id {
        state_id::ATTACK_MOVE_TO => attack_move_on_enter(sim, ctx),
        state_id::MOVE_OUT_OF_THE_WAY => {
            set_adjusts_destination(sim, ctx, id, true);
            if ai(sim, ctx).has_path == FALSE {
                return StateReturnType::Failure;
            }
            if let Some((pos, _)) = last_node(sim, ctx) {
                data(sim, ctx, id).goal_position = pos;
            }
            internal_on_enter(sim, ctx, id)
        }
        state_id::MOVE_AND_TIGHTEN => {
            set_adjusts_destination(sim, ctx, id, false);
            let state = &mut moves(sim, ctx).tighten;
            state.ok_to_repath_times = 1;
            state.check_for_path = TRUE;
            remove_goal(sim, ctx);
            let goal = ai(sim, ctx).machine.goal_position;
            data(sim, ctx, id).goal_position = goal;
            ai_update::request_approach_path(sim, ctx, &goal);
            internal_on_enter(sim, ctx, id)
        }
        state_id::FOLLOW_PATH | state_id::FOLLOW_EXITPRODUCTION_PATH => {
            follow_on_enter(sim, ctx, id)
        }
        _ => move_to_on_enter(sim, ctx, state_id::MOVE_TO),
    }
}

pub fn update(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> StateReturnType {
    match id {
        state_id::ATTACK_MOVE_TO => attack_move_update(sim, ctx),
        state_id::MOVE_OUT_OF_THE_WAY => {
            let dead = sim
                .object(ctx.owner_slot)
                .is_some_and(|o| o.is_effectively_dead() != FALSE);
            if dead {
                return StateReturnType::Success;
            }
            internal_update(sim, ctx, id)
        }
        state_id::MOVE_AND_TIGHTEN => {
            if moves(sim, ctx).tighten.check_for_path != FALSE {
                let unit = ai(sim, ctx);
                if unit.has_path != FALSE && unit.waiting_for_path == FALSE {
                    set_adjusts_destination(sim, ctx, id, true);
                    moves(sim, ctx).tighten.check_for_path = FALSE;
                }
            }
            internal_update(sim, ctx, id)
        }
        state_id::FOLLOW_PATH | state_id::FOLLOW_EXITPRODUCTION_PATH => follow_update(sim, ctx, id),
        _ => move_to_update(sim, ctx, state_id::MOVE_TO),
    }
}

pub fn on_exit(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) {
    match id {
        state_id::ATTACK_MOVE_TO => attack_move_on_exit(sim, ctx),
        state_id::MOVE_OUT_OF_THE_WAY => {
            internal_on_exit(sim, ctx, id);
            let unit = ai(sim, ctx);
            unit.destroy_path();
            unit.set_can_path_through_units(FALSE);
            unit.move_out_of_way1 = ObjectId::INVALID;
            unit.move_out_of_way2 = ObjectId::INVALID;
        }
        state_id::FOLLOW_PATH | state_id::FOLLOW_EXITPRODUCTION_PATH => {
            follow_on_exit(sim, ctx, id)
        }
        _ => internal_on_exit(sim, ctx, id),
    }
}
