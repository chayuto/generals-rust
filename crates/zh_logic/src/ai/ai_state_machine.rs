//! `PKT-428`: `AIStateMachine` (`AIStateMachine.h`, the machine part of
//! `AIStates.cpp`) with the `StateMachine` base it relies on. Packet doc:
//! `packets/phase6/PKT-428_AIStateMachine.md`. Golden test:
//! `tests/golden_s8_ai_orders.rs`.
//!
//! This is [`zh_common::core::state_machine::StateMachine`] again, in the form a
//! resident module needs (see [`crate::ai`]): the machine is plain data inside the
//! AI module, and every transition is a function of `(sim, ctx)` that borrows the
//! data one statement at a time, so a state can order its own unit and the order
//! reaches the machine while the state is still running.

use crate::ai::ai_states::{self, IdleState};
use crate::ai::seams::SeamsGuard;
use crate::arena::ModuleContext;
use crate::modules::update::ai_update::{self, ai};
use crate::sim::SimApi;
use zh_common::core::state_machine::{
    StateExitType, StateId, StateReturnType, EXIT_MACHINE_WITH_FAILURE, EXIT_MACHINE_WITH_SUCCESS,
    INVALID_STATE_ID, MACHINE_DONE_STATE_ID, MAX_TRANSITION_DEPTH,
};
use zh_common::ini::LOGICFRAMES_PER_SECOND;
use zh_common::math::Coord3D;
use zh_common::types::{Bool, Int, ObjectId, UnsignedInt, XferVersion, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError, XferMode};

/// `AIStateType`.
pub mod state_id {
    use zh_common::core::state_machine::StateId;

    pub const IDLE: StateId = 0;
    pub const MOVE_TO: StateId = 1;
    pub const FOLLOW_WAYPOINT_PATH_AS_TEAM: StateId = 2;
    pub const FOLLOW_WAYPOINT_PATH_AS_INDIVIDUALS: StateId = 3;
    pub const FOLLOW_WAYPOINT_PATH_AS_TEAM_EXACT: StateId = 4;
    pub const FOLLOW_WAYPOINT_PATH_AS_INDIVIDUALS_EXACT: StateId = 5;
    pub const FOLLOW_PATH: StateId = 6;
    pub const FOLLOW_EXITPRODUCTION_PATH: StateId = 7;
    pub const WAIT: StateId = 8;
    pub const ATTACK_POSITION: StateId = 9;
    pub const ATTACK_OBJECT: StateId = 10;
    pub const FORCE_ATTACK_OBJECT: StateId = 11;
    pub const ATTACK_AND_FOLLOW_OBJECT: StateId = 12;
    pub const DEAD: StateId = 13;
    pub const DOCK: StateId = 14;
    pub const ENTER: StateId = 15;
    pub const GUARD: StateId = 16;
    pub const HUNT: StateId = 17;
    pub const WANDER: StateId = 18;
    pub const PANIC: StateId = 19;
    pub const ATTACK_SQUAD: StateId = 20;
    pub const GUARD_TUNNEL_NETWORK: StateId = 21;
    pub const GET_REPAIRED: StateId = 22;
    pub const MOVE_OUT_OF_THE_WAY: StateId = 23;
    pub const MOVE_AND_TIGHTEN: StateId = 24;
    pub const MOVE_AND_EVACUATE: StateId = 25;
    pub const MOVE_AND_EVACUATE_AND_EXIT: StateId = 26;
    pub const MOVE_AND_DELETE: StateId = 27;
    pub const ATTACK_AREA: StateId = 28;
    pub const HACK_INTERNET: StateId = 29;
    pub const ATTACK_MOVE_TO: StateId = 30;
    pub const ATTACKFOLLOW_WAYPOINT_PATH_AS_INDIVIDUALS: StateId = 31;
    pub const ATTACKFOLLOW_WAYPOINT_PATH_AS_TEAM: StateId = 32;
    pub const FACE_OBJECT: StateId = 33;
    pub const FACE_POSITION: StateId = 34;
    pub const RAPPEL_INTO: StateId = 35;
    pub const COMBATDROP: StateId = 36;
    pub const EXIT: StateId = 37;
    pub const PICK_UP_CRATE: StateId = 38;
    pub const MOVE_AWAY_FROM_REPULSORS: StateId = 39;
    pub const WANDER_IN_PLACE: StateId = 40;
    pub const BUSY: StateId = 41;
    pub const EXIT_INSTANTLY: StateId = 42;
    pub const GUARD_RETALIATE: StateId = 43;
}

/// The success and failure targets of a state `AIStateMachine` defines; `None` for
/// an id it does not define.
pub fn state_links(id: StateId) -> Option<(StateId, StateId)> {
    use state_id as s;
    Some(match id {
        s::ATTACK_AND_FOLLOW_OBJECT => (s::IDLE, s::IDLE),
        s::GET_REPAIRED | s::HACK_INTERNET | s::COMBATDROP => return None,
        s::MOVE_AWAY_FROM_REPULSORS => (s::WANDER_IN_PLACE, s::WANDER_IN_PLACE),
        s::WANDER_IN_PLACE => (s::MOVE_AWAY_FROM_REPULSORS, s::MOVE_AWAY_FROM_REPULSORS),
        s::MOVE_AND_EVACUATE_AND_EXIT => (s::MOVE_AND_DELETE, s::MOVE_AND_DELETE),
        s::WANDER | s::PANIC => (s::IDLE, s::MOVE_AWAY_FROM_REPULSORS),
        _ if id <= s::GUARD_RETALIATE => (s::IDLE, s::IDLE),
        _ => return None,
    })
}

/// The members of `AIStateMachine` and of its `StateMachine` base.
#[derive(Debug, Clone)]
pub struct AiStateMachine {
    pub sleep_till: UnsignedInt,
    pub default_state_id: StateId,
    /// `m_currentState`; [`INVALID_STATE_ID`] for none.
    pub current: StateId,
    pub goal_object_id: ObjectId,
    pub goal_position: Coord3D,
    pub locked: Bool,
    pub default_state_inited: Bool,
    transition_depth: u32,
    pub goal_path: Vec<Coord3D>,
    /// `m_goalWaypoint`: id and name.
    pub goal_waypoint: Option<(UnsignedInt, String)>,
    /// `m_goalSquad` exists. Its members are not ported.
    pub has_goal_squad: Bool,
    /// `m_temporaryState`; [`INVALID_STATE_ID`] for none.
    pub temporary: StateId,
    pub temporary_frame_end: UnsignedInt,
    /// The `AI_IDLE` state object.
    pub idle: IdleState,
    /// The idle states of the three attack-move sub machines.
    pub attack_move_idle: [IdleState; 3],
}

impl Default for AiStateMachine {
    fn default() -> Self {
        Self {
            sleep_till: 0,
            default_state_id: state_id::IDLE,
            current: INVALID_STATE_ID,
            goal_object_id: ObjectId::INVALID,
            goal_position: Coord3D::default(),
            locked: FALSE,
            default_state_inited: FALSE,
            transition_depth: 0,
            goal_path: Vec::new(),
            goal_waypoint: None,
            has_goal_squad: FALSE,
            temporary: INVALID_STATE_ID,
            temporary_frame_end: 0,
            idle: IdleState::new(TRUE),
            attack_move_idle: [IdleState::new(FALSE); 3],
        }
    }
}

impl AiStateMachine {
    /// `getCurrentStateID`.
    pub fn current_state_id(&self) -> StateId {
        self.current
    }

    /// `isInIdleState`.
    pub fn is_in_idle_state(&self) -> Bool {
        Bool::from(self.current == INVALID_STATE_ID || self.current == state_id::IDLE)
    }

    /// `isInBusyState`.
    pub fn is_in_busy_state(&self) -> Bool {
        Bool::from(self.current == state_id::BUSY)
    }

    /// `setGoalObject`: the id and position of the object, or none.
    pub fn set_goal_object(&mut self, obj: Option<(ObjectId, Coord3D)>) {
        if self.locked != FALSE {
            return;
        }
        match obj {
            Some((id, pos)) => {
                self.goal_object_id = id;
                self.goal_position = pos;
            }
            None => self.goal_object_id = ObjectId::INVALID,
        }
    }

    /// `setGoalPosition`.
    pub fn set_goal_position(&mut self, pos: Option<&Coord3D>) {
        if self.locked != FALSE {
            return;
        }
        if let Some(pos) = pos {
            self.goal_position = *pos;
        }
    }

    /// `addToGoalPath`.
    pub fn add_to_goal_path(&mut self, point: &Coord3D) {
        if self.goal_path.last() != Some(point) {
            self.goal_path.push(*point);
        }
    }

    pub fn lock(&mut self) {
        self.locked = TRUE;
    }

    pub fn unlock(&mut self) {
        self.locked = FALSE;
    }

    /// `halt`.
    pub fn halt(&mut self) {
        self.locked = TRUE;
        self.current = INVALID_STATE_ID;
    }

    /// `AIStateMachine::xfer` with `StateMachine::xfer`. Loading is not
    /// supported. In CRC mode the reference's `crc` is empty, so nothing is
    /// written.
    pub fn xfer(
        &mut self,
        xfer: &mut dyn Xfer,
        owner: ObjectId,
        seams: &mut SeamsGuard,
    ) -> Result<(), XferError> {
        match xfer.mode() {
            XferMode::Crc => return Ok(()),
            XferMode::Load => return Err(XferError::InvalidParameters),
            XferMode::Save => {}
        }
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        let mut base_version: XferVersion = 1;
        xfer.xfer_version(&mut base_version, 1)?;
        let mut current = self.current;
        xfer.xfer_unsigned_int(&mut self.sleep_till)?;
        xfer.xfer_unsigned_int(&mut self.default_state_id)?;
        xfer.xfer_unsigned_int(&mut current)?;
        let mut snapshot_all = FALSE;
        xfer.xfer_bool(&mut snapshot_all)?;
        if self.current == INVALID_STATE_ID {
            // The reference makes the default state current while saving; the
            // id written above is still the invalid one.
            self.current = self.default_state_id;
        }
        self.xfer_state(self.current, xfer, owner, seams)?;
        xfer.xfer_object_id(&mut self.goal_object_id)?;
        xfer.xfer_coord3d(&mut self.goal_position)?;
        xfer.xfer_bool(&mut self.locked)?;
        xfer.xfer_bool(&mut self.default_state_inited)?;

        let mut count = self.goal_path.len() as Int;
        xfer.xfer_int(&mut count)?;
        for point in &mut self.goal_path {
            xfer.xfer_coord3d(point)?;
        }
        let mut waypoint = self
            .goal_waypoint
            .as_ref()
            .map_or_else(String::new, |(_, name)| name.clone());
        xfer.xfer_ascii_string(&mut waypoint)?;
        let mut has_squad = self.has_goal_squad;
        xfer.xfer_bool(&mut has_squad)?;
        if has_squad != FALSE {
            // `Squad::xfer` is not ported.
            return Err(XferError::InvalidData);
        }
        let mut temporary = self.temporary;
        xfer.xfer_unsigned_int(&mut temporary)?;
        if temporary != INVALID_STATE_ID {
            self.xfer_state(temporary, xfer, owner, seams)?;
        }
        xfer.xfer_unsigned_int(&mut self.temporary_frame_end)
    }

    /// `State::xfer` of the state with this id.
    fn xfer_state(
        &mut self,
        id: StateId,
        xfer: &mut dyn Xfer,
        owner: ObjectId,
        seams: &mut SeamsGuard,
    ) -> Result<(), XferError> {
        match id {
            state_id::IDLE => self.idle.xfer(xfer),
            state_id::DEAD | state_id::BUSY => {
                let mut version: XferVersion = 1;
                xfer.xfer_version(&mut version, 1)
            }
            _ => seams.state_xfer(owner, id, xfer),
        }
    }
}

fn sm(sim: &mut dyn SimApi, ctx: ModuleContext) -> &mut AiStateMachine {
    &mut ai(sim, ctx).machine
}

/// The base `StateMachine::internalSetState`.
fn internal_set_state(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    new_id: StateId,
) -> StateReturnType {
    sm(sim, ctx).sleep_till = 0;
    let mut next = INVALID_STATE_ID;
    if new_id != MACHINE_DONE_STATE_ID {
        next = if new_id == INVALID_STATE_ID {
            sm(sim, ctx).default_state_id
        } else {
            new_id
        };
        if state_links(next).is_none() {
            return StateReturnType::Failure;
        }
    }
    let old = sm(sim, ctx).current;
    if old != INVALID_STATE_ID {
        ai_states::on_exit(sim, ctx, old, StateExitType::Normal);
    }
    sm(sim, ctx).current = next;
    if next == INVALID_STATE_ID {
        return StateReturnType::Continue;
    }
    let status = ai_states::on_enter(sim, ctx, next);
    finish_state_call(sim, ctx, next, status)
}

/// What follows `onEnter` and `update` alike.
fn finish_state_call(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    ran: StateId,
    mut status: StateReturnType,
) -> StateReturnType {
    let current = sm(sim, ctx).current;
    if current == INVALID_STATE_ID {
        return StateReturnType::Failure;
    }
    if current != ran {
        status = StateReturnType::Continue;
    }
    if let StateReturnType::Sleep(frames) = status {
        let now = sim.frame();
        sm(sim, ctx).sleep_till = now.wrapping_add(frames);
        // No `AIStateMachine` state has sleep transitions.
        return status;
    }
    check_normal_transitions(sim, ctx, status, current)
}

fn check_normal_transitions(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    status: StateReturnType,
    id: StateId,
) -> StateReturnType {
    let machine = sm(sim, ctx);
    machine.transition_depth = machine.transition_depth.wrapping_add(1);
    if machine.transition_depth >= MAX_TRANSITION_DEPTH {
        machine.transition_depth = machine.transition_depth.wrapping_sub(1);
        return StateReturnType::Failure;
    }
    let Some((success, failure)) = state_links(id) else {
        return StateReturnType::Failure;
    };
    let result = match status {
        StateReturnType::Success | StateReturnType::Failure => {
            let next = if status == StateReturnType::Success {
                success
            } else {
                failure
            };
            match next {
                EXIT_MACHINE_WITH_SUCCESS => {
                    let _ = internal_set_state(sim, ctx, MACHINE_DONE_STATE_ID);
                    StateReturnType::Success
                }
                EXIT_MACHINE_WITH_FAILURE => {
                    let _ = internal_set_state(sim, ctx, MACHINE_DONE_STATE_ID);
                    StateReturnType::Failure
                }
                next => internal_set_state(sim, ctx, next),
            }
        }
        other => other,
    };
    let machine = sm(sim, ctx);
    machine.transition_depth = machine.transition_depth.wrapping_sub(1);
    result
}

/// `StateMachine::initDefaultState`.
pub fn init_default_state(sim: &mut dyn SimApi, ctx: ModuleContext) -> StateReturnType {
    let machine = sm(sim, ctx);
    if machine.default_state_inited != FALSE {
        return StateReturnType::Failure;
    }
    if machine.locked != FALSE {
        return StateReturnType::Continue;
    }
    machine.default_state_inited = TRUE;
    let default = machine.default_state_id;
    internal_set_state(sim, ctx, default)
}

/// `StateMachine::updateStateMachine`.
fn base_update(sim: &mut dyn SimApi, ctx: ModuleContext) -> StateReturnType {
    let now = sim.frame();
    let machine = sm(sim, ctx);
    let id = machine.current;
    if machine.sleep_till != 0 && now < machine.sleep_till {
        if id == INVALID_STATE_ID {
            return StateReturnType::Failure;
        }
        return StateReturnType::Sleep(machine.sleep_till.wrapping_sub(now));
    }
    machine.sleep_till = 0;
    if id == INVALID_STATE_ID {
        return StateReturnType::Failure;
    }
    let status = ai_states::update(sim, ctx, id);
    finish_state_call(sim, ctx, id, status)
}

/// `AIStateMachine::updateStateMachine`.
pub fn update_state_machine(sim: &mut dyn SimApi, ctx: ModuleContext) -> StateReturnType {
    let temporary = sm(sim, ctx).temporary;
    if temporary != INVALID_STATE_ID {
        let mut status = ai_states::update(sim, ctx, temporary);
        let now = sim.frame();
        let machine = sm(sim, ctx);
        if machine.temporary != temporary {
            if machine.temporary != INVALID_STATE_ID {
                return StateReturnType::Continue;
            }
        } else {
            if machine.temporary_frame_end < now && status == StateReturnType::Continue {
                status = StateReturnType::Success;
            }
            if status == StateReturnType::Continue {
                return status;
            }
            ai_states::on_exit(sim, ctx, temporary, StateExitType::Normal);
            sm(sim, ctx).temporary = INVALID_STATE_ID;
        }
    }
    base_update(sim, ctx)
}

/// `AIStateMachine::setTemporaryState`.
pub fn set_temporary_state(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    new_id: StateId,
    frame_limit: Int,
) -> StateReturnType {
    let old = sm(sim, ctx).temporary;
    if old != INVALID_STATE_ID {
        ai_states::on_exit(sim, ctx, old, StateExitType::Reset);
        sm(sim, ctx).temporary = INVALID_STATE_ID;
    }
    if state_links(new_id).is_none() {
        return StateReturnType::Failure;
    }
    sm(sim, ctx).temporary = new_id;
    let ret = ai_states::on_enter(sim, ctx, new_id);
    if ret != StateReturnType::Continue {
        ai_states::on_exit(sim, ctx, new_id, StateExitType::Normal);
        sm(sim, ctx).temporary = INVALID_STATE_ID;
        return ret;
    }
    let limit = frame_limit.min(60 * LOGICFRAMES_PER_SECOND as Int);
    let end = sim.frame().wrapping_add(limit as UnsignedInt);
    sm(sim, ctx).temporary_frame_end = end;
    ret
}

/// `StateMachine::clear`.
fn base_clear(sim: &mut dyn SimApi, ctx: ModuleContext) {
    if sm(sim, ctx).locked != FALSE {
        return;
    }
    let old = sm(sim, ctx).current;
    if old != INVALID_STATE_ID {
        ai_states::on_exit(sim, ctx, old, StateExitType::Reset);
    }
    let machine = sm(sim, ctx);
    machine.current = INVALID_STATE_ID;
    machine.goal_object_id = ObjectId::INVALID;
    machine.goal_position = Coord3D::default();
}

/// `AIStateMachine::clear`.
pub fn clear(sim: &mut dyn SimApi, ctx: ModuleContext) {
    base_clear(sim, ctx);
    let machine = sm(sim, ctx);
    machine.goal_path.clear();
    machine.goal_waypoint = None;
    machine.has_goal_squad = FALSE;
    if cfg!(not(feature = "retail-crc")) {
        let temporary = machine.temporary;
        if temporary != INVALID_STATE_ID {
            ai_states::on_exit(sim, ctx, temporary, StateExitType::Reset);
        }
        sm(sim, ctx).temporary = INVALID_STATE_ID;
    }
    ai_update::wake_up_now(sim, ctx);
}

/// `AIStateMachine::setState`.
pub fn set_state(sim: &mut dyn SimApi, ctx: ModuleContext, new_id: StateId) -> StateReturnType {
    let old = sm(sim, ctx).current;
    let result = if sm(sim, ctx).locked != FALSE {
        StateReturnType::Continue
    } else {
        internal_set_state(sim, ctx, new_id)
    };
    if old != new_id {
        ai_update::wake_up_now(sim, ctx);
    }
    result
}
