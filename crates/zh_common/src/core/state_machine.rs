//! `PKT-077`: Deterministic finite state machine (`GeneralsMD/Code/GameEngine/Source/Common/StateMachine.cpp`).
//!
//! Reference C++ packet: `packets/phase2/PKT-077_StateMachine.md`
//! Golden Vector test: `crates/zh_common/tests/golden_pkt077.rs` (`docs/golden_vectors/PKT-077.json`)

#![allow(unused_variables, dead_code)]

use std::collections::BTreeMap;

use crate::math::Coord3D;
use crate::types::{Bool, Int, ObjectId, UnsignedInt, XferVersion, FALSE, TRUE};
use crate::xfer::{Xfer, XferError, XferMode};

pub type StateId = UnsignedInt;

pub const MACHINE_DONE_STATE_ID: StateId = 999_998;
pub const INVALID_STATE_ID: StateId = 999_999;

pub const EXIT_MACHINE_WITH_SUCCESS: StateId = 9_998;
pub const EXIT_MACHINE_WITH_FAILURE: StateId = 9_999;

pub const STATE_CONTINUE_CODE: Int = 0;
pub const STATE_SUCCESS_CODE: Int = -1;
pub const STATE_FAILURE_CODE: Int = -2;
pub const STATE_SLEEP_FOREVER_FRAMES: UnsignedInt = 0x3fff_ffff;
pub const MAX_TRANSITION_DEPTH: u32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateReturnType {
    Continue,
    Success,
    Failure,
    Sleep(UnsignedInt),
}

impl StateReturnType {
    #[inline]
    pub const fn sleep_forever() -> Self {
        Self::Sleep(STATE_SLEEP_FOREVER_FRAMES)
    }

    #[inline]
    pub const fn as_raw(self) -> Int {
        match self {
            Self::Continue => STATE_CONTINUE_CODE,
            Self::Success => STATE_SUCCESS_CODE,
            Self::Failure => STATE_FAILURE_CODE,
            Self::Sleep(frames) => frames as Int,
        }
    }

    #[inline]
    pub const fn from_raw(raw: Int) -> Self {
        if raw > 0 {
            Self::Sleep(raw as UnsignedInt)
        } else if raw == STATE_SUCCESS_CODE {
            Self::Success
        } else if raw == STATE_FAILURE_CODE {
            Self::Failure
        } else {
            Self::Continue
        }
    }

    #[inline]
    pub const fn is_sleep(self) -> bool {
        matches!(self, Self::Sleep(n) if n > 0)
    }

    #[inline]
    pub const fn sleep_frames(self) -> UnsignedInt {
        match self {
            Self::Sleep(n) => n,
            _ => 0,
        }
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateExitType {
    Normal = 0,
    Reset = 1,
}

pub trait StateWorldView {
    fn current_frame(&self) -> UnsignedInt;
    fn object_exists(&self, id: ObjectId) -> bool;
}

pub type StateTransFn<Ctx> = fn(&mut Ctx) -> Bool;
pub type StateEnterFn<Ctx> = fn(&mut StateMachine<Ctx>, &mut Ctx) -> StateReturnType;
pub type StateUpdateFn<Ctx> = fn(&mut StateMachine<Ctx>, &mut Ctx) -> StateReturnType;
pub type StateExitFn<Ctx> = fn(StateExitType, &mut Ctx);
pub type StateXferFn<Ctx> = fn(StateId, &mut dyn Xfer, &mut Ctx) -> Result<(), XferError>;

pub struct StateCondition<Ctx> {
    pub test: StateTransFn<Ctx>,
    pub to_state_id: StateId,
}

pub struct StateCallbacks<Ctx> {
    pub on_enter: StateEnterFn<Ctx>,
    pub on_exit: StateExitFn<Ctx>,
    pub update: StateUpdateFn<Ctx>,
    pub xfer_state: StateXferFn<Ctx>,
    pub is_idle: Bool,
    pub is_attack: Bool,
    pub is_guard_idle: Bool,
    pub is_busy: Bool,
}

struct StateEntry<Ctx> {
    id: StateId,
    success_id: StateId,
    failure_id: StateId,
    conditions: Vec<StateCondition<Ctx>>,
    callbacks: StateCallbacks<Ctx>,
}

impl<Ctx> Copy for StateCondition<Ctx> {}
impl<Ctx> Clone for StateCondition<Ctx> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<Ctx> Copy for StateCallbacks<Ctx> {}
impl<Ctx> Clone for StateCallbacks<Ctx> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<Ctx> Clone for StateEntry<Ctx> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            success_id: self.success_id,
            failure_id: self.failure_id,
            conditions: self.conditions.iter().copied().collect(),
            callbacks: self.callbacks,
        }
    }
}

pub struct StateMachine<Ctx> {
    states: BTreeMap<StateId, StateEntry<Ctx>>,
    owner_id: ObjectId,
    sleep_till: UnsignedInt,
    default_state_id: StateId,
    current_state_id: Option<StateId>,
    goal_object_id: ObjectId,
    goal_position: Coord3D,
    locked: Bool,
    default_state_inited: Bool,
    transition_depth: u32,
    sleep_transition_depth: u32,
}

impl<Ctx: StateWorldView> StateMachine<Ctx> {
    pub fn new(owner_id: ObjectId) -> Self {
        Self {
            states: BTreeMap::new(),
            owner_id,
            sleep_till: 0,
            default_state_id: INVALID_STATE_ID,
            current_state_id: None,
            goal_object_id: ObjectId::INVALID,
            goal_position: Coord3D::default(),
            locked: FALSE,
            default_state_inited: FALSE,
            transition_depth: 0,
            sleep_transition_depth: 0,
        }
    }

    pub fn define_state(
        &mut self,
        id: StateId,
        callbacks: StateCallbacks<Ctx>,
        success_id: StateId,
        failure_id: StateId,
        conditions: &[StateCondition<Ctx>],
    ) {
        if self.default_state_id == INVALID_STATE_ID {
            self.default_state_id = id;
        }
        self.states.insert(
            id,
            StateEntry {
                id,
                success_id,
                failure_id,
                conditions: conditions.to_vec(),
                callbacks,
            },
        );
    }

    pub fn init_default_state(&mut self, ctx: &mut Ctx) -> StateReturnType {
        if self.default_state_inited == TRUE {
            return StateReturnType::Failure;
        }
        if self.locked == TRUE {
            return StateReturnType::Continue;
        }
        self.default_state_inited = TRUE;
        self.internal_set_state(self.default_state_id, ctx)
    }

    pub fn update_state_machine(&mut self, ctx: &mut Ctx) -> StateReturnType {
        let now = ctx.current_frame();
        if self.sleep_till != 0 && now < self.sleep_till {
            let Some(id) = self.current_state_id else {
                return StateReturnType::Failure;
            };
            let Some(entry) = self.states.get(&id) else {
                return StateReturnType::Failure;
            };
            let callbacks = entry.callbacks;
            let remaining = self.sleep_till.wrapping_sub(now);
            return self.check_sleep_transitions(StateReturnType::Sleep(remaining), ctx, callbacks);
        }

        self.sleep_till = 0;
        let Some(id) = self.current_state_id else {
            return StateReturnType::Failure;
        };
        let Some(entry) = self.states.get(&id) else {
            return StateReturnType::Failure;
        };
        let callbacks = entry.callbacks;
        let mut status = (callbacks.update)(self, ctx);

        if self.current_state_id.is_none() {
            return StateReturnType::Failure;
        }
        if self.current_state_id != Some(id) {
            status = StateReturnType::Continue;
        }

        if let StateReturnType::Sleep(frames) = status {
            self.sleep_till = ctx.current_frame().wrapping_add(frames);
            let remaining = self.sleep_till.wrapping_sub(ctx.current_frame());
            return self.check_sleep_transitions(StateReturnType::Sleep(remaining), ctx, callbacks);
        }
        let result_id = self.current_state_id.unwrap_or(id);
        self.check_normal_transitions(status, ctx, result_id)
    }

    pub fn clear(&mut self, ctx: &mut Ctx) {
        if self.locked == TRUE {
            return;
        }
        if let Some(id) = self.current_state_id {
            if let Some(entry) = self.states.get(&id) {
                let on_exit = entry.callbacks.on_exit;
                on_exit(StateExitType::Reset, ctx);
            }
        }
        self.current_state_id = None;
        self.internal_clear();
    }

    pub fn reset_to_default_state(&mut self, ctx: &mut Ctx) -> StateReturnType {
        if self.locked == TRUE {
            return StateReturnType::Failure;
        }
        if self.default_state_inited != TRUE {
            return StateReturnType::Failure;
        }
        if let Some(id) = self.current_state_id {
            if let Some(entry) = self.states.get(&id) {
                let on_exit = entry.callbacks.on_exit;
                on_exit(StateExitType::Reset, ctx);
            }
        }
        self.current_state_id = None;
        self.internal_clear();
        self.internal_set_state(self.default_state_id, ctx)
    }

    pub fn set_state(&mut self, new_state_id: StateId, ctx: &mut Ctx) -> StateReturnType {
        if self.locked == TRUE {
            return StateReturnType::Continue;
        }
        self.internal_set_state(new_state_id, ctx)
    }

    pub fn internal_set_state(&mut self, new_state_id: StateId, ctx: &mut Ctx) -> StateReturnType {
        self.sleep_till = 0;
        if new_state_id == MACHINE_DONE_STATE_ID {
            if let Some(id) = self.current_state_id {
                if let Some(entry) = self.states.get(&id) {
                    let on_exit = entry.callbacks.on_exit;
                    on_exit(StateExitType::Normal, ctx);
                }
            }
            self.current_state_id = None;
            return StateReturnType::Continue;
        }

        let mut resolved_id = new_state_id;
        if resolved_id == INVALID_STATE_ID {
            resolved_id = self.default_state_id;
            if resolved_id == INVALID_STATE_ID {
                return StateReturnType::Failure;
            }
        }
        let Some(entry) = self.states.get(&resolved_id).cloned() else {
            return StateReturnType::Failure;
        };

        if let Some(id) = self.current_state_id {
            if let Some(old_entry) = self.states.get(&id) {
                let on_exit = old_entry.callbacks.on_exit;
                on_exit(StateExitType::Normal, ctx);
            }
        }

        self.current_state_id = Some(resolved_id);
        let mut status = (entry.callbacks.on_enter)(self, ctx);
        if self.current_state_id.is_none() {
            return StateReturnType::Failure;
        }
        if self.current_state_id != Some(resolved_id) {
            status = StateReturnType::Continue;
        }

        if let StateReturnType::Sleep(frames) = status {
            self.sleep_till = ctx.current_frame().wrapping_add(frames);
            let remaining = self.sleep_till.wrapping_sub(ctx.current_frame());
            let current_id = self.current_state_id;
            let callbacks = current_id
                .and_then(|id| self.states.get(&id))
                .map(|entry| entry.callbacks);
            if let Some(callbacks) = callbacks {
                return self.check_sleep_transitions(
                    StateReturnType::Sleep(remaining),
                    ctx,
                    callbacks,
                );
            }
            return StateReturnType::Failure;
        }
        let result_id = self.current_state_id.unwrap_or(resolved_id);
        self.check_normal_transitions(status, ctx, result_id)
    }

    fn internal_clear(&mut self) {
        self.goal_object_id = ObjectId::INVALID;
        self.goal_position = Coord3D::default();
    }

    fn check_normal_transitions(
        &mut self,
        status: StateReturnType,
        ctx: &mut Ctx,
        state_id: StateId,
    ) -> StateReturnType {
        self.transition_depth = self.transition_depth.saturating_add(1);
        if self.transition_depth >= MAX_TRANSITION_DEPTH {
            self.transition_depth = self.transition_depth.saturating_sub(1);
            return StateReturnType::Failure;
        }

        let result = match status {
            StateReturnType::Success => {
                let Some(entry) = self.states.get(&state_id).cloned() else {
                    self.transition_depth = self.transition_depth.saturating_sub(1);
                    return StateReturnType::Failure;
                };
                match entry.success_id {
                    EXIT_MACHINE_WITH_SUCCESS => {
                        let _ = self.internal_set_state(MACHINE_DONE_STATE_ID, ctx);
                        StateReturnType::Success
                    }
                    EXIT_MACHINE_WITH_FAILURE => {
                        let _ = self.internal_set_state(MACHINE_DONE_STATE_ID, ctx);
                        StateReturnType::Failure
                    }
                    next_id => self.internal_set_state(next_id, ctx),
                }
            }
            StateReturnType::Failure => {
                let Some(entry) = self.states.get(&state_id).cloned() else {
                    self.transition_depth = self.transition_depth.saturating_sub(1);
                    return StateReturnType::Failure;
                };
                match entry.failure_id {
                    EXIT_MACHINE_WITH_SUCCESS => {
                        let _ = self.internal_set_state(MACHINE_DONE_STATE_ID, ctx);
                        StateReturnType::Success
                    }
                    EXIT_MACHINE_WITH_FAILURE => {
                        let _ = self.internal_set_state(MACHINE_DONE_STATE_ID, ctx);
                        StateReturnType::Failure
                    }
                    next_id => self.internal_set_state(next_id, ctx),
                }
            }
            StateReturnType::Continue => {
                let Some(entry) = self.states.get(&state_id).cloned() else {
                    self.transition_depth = self.transition_depth.saturating_sub(1);
                    return StateReturnType::Continue;
                };
                let mut result = StateReturnType::Continue;
                for condition in &entry.conditions {
                    if (condition.test)(ctx) == TRUE {
                        match condition.to_state_id {
                            EXIT_MACHINE_WITH_SUCCESS => {
                                result = StateReturnType::Success;
                            }
                            EXIT_MACHINE_WITH_FAILURE => {
                                result = StateReturnType::Failure;
                            }
                            next_id => {
                                result = self.internal_set_state(next_id, ctx);
                            }
                        }
                        break;
                    }
                }
                result
            }
            sleep @ StateReturnType::Sleep(_) => sleep,
        };
        self.transition_depth = self.transition_depth.saturating_sub(1);
        result
    }

    fn check_sleep_transitions(
        &mut self,
        status: StateReturnType,
        ctx: &mut Ctx,
        callbacks: StateCallbacks<Ctx>,
    ) -> StateReturnType {
        self.sleep_transition_depth = self.sleep_transition_depth.saturating_add(1);
        if self.sleep_transition_depth >= MAX_TRANSITION_DEPTH {
            self.sleep_transition_depth = self.sleep_transition_depth.saturating_sub(1);
            return StateReturnType::Failure;
        }

        let Some(id) = self.current_state_id else {
            self.sleep_transition_depth = self.sleep_transition_depth.saturating_sub(1);
            return StateReturnType::Failure;
        };
        let Some(entry) = self.states.get(&id).cloned() else {
            self.sleep_transition_depth = self.sleep_transition_depth.saturating_sub(1);
            return StateReturnType::Failure;
        };
        let _ = callbacks;

        let mut result = status;
        for condition in &entry.conditions {
            if (condition.test)(ctx) == TRUE {
                match condition.to_state_id {
                    EXIT_MACHINE_WITH_SUCCESS => result = StateReturnType::Success,
                    EXIT_MACHINE_WITH_FAILURE => result = StateReturnType::Failure,
                    next_id => result = self.internal_set_state(next_id, ctx),
                }
                break;
            }
        }
        self.sleep_transition_depth = self.sleep_transition_depth.saturating_sub(1);
        result
    }

    #[inline]
    pub const fn get_current_state_id(&self) -> StateId {
        match self.current_state_id {
            Some(id) => id,
            None => INVALID_STATE_ID,
        }
    }

    pub fn is_in_idle_state(&self) -> Bool {
        match self.current_state_id {
            Some(id) => self.states.get(&id).map_or(TRUE, |s| s.callbacks.is_idle),
            None => TRUE,
        }
    }

    pub fn is_in_attack_state(&self) -> Bool {
        match self.current_state_id {
            Some(id) => self.states.get(&id).map_or(TRUE, |s| s.callbacks.is_attack),
            None => TRUE,
        }
    }

    pub fn is_in_force_attack_state(&self) -> Bool {
        self.is_in_idle_state()
    }

    pub fn is_in_guard_idle_state(&self) -> Bool {
        match self.current_state_id {
            Some(id) => self
                .states
                .get(&id)
                .map_or(FALSE, |s| s.callbacks.is_guard_idle),
            None => FALSE,
        }
    }

    pub fn is_in_busy_state(&self) -> Bool {
        match self.current_state_id {
            Some(id) => self.states.get(&id).map_or(FALSE, |s| s.callbacks.is_busy),
            None => FALSE,
        }
    }

    #[inline]
    pub fn lock(&mut self) {
        self.locked = TRUE;
    }

    #[inline]
    pub fn unlock(&mut self) {
        self.locked = FALSE;
    }

    #[inline]
    pub const fn is_locked(&self) -> Bool {
        self.locked
    }

    #[inline]
    pub const fn get_owner_id(&self) -> ObjectId {
        self.owner_id
    }

    #[inline]
    pub const fn get_goal_object_id(&self) -> ObjectId {
        self.goal_object_id
    }

    #[inline]
    pub const fn get_goal_position(&self) -> &Coord3D {
        &self.goal_position
    }

    #[inline]
    pub const fn peek_sleep_till(&self) -> UnsignedInt {
        self.sleep_till
    }

    pub fn set_goal_object(&mut self, obj: Option<(ObjectId, Coord3D)>) {
        if self.locked == TRUE {
            return;
        }
        if let Some((id, position)) = obj {
            self.goal_object_id = id;
            self.goal_position = position;
        } else {
            self.goal_object_id = ObjectId::INVALID;
        }
    }

    pub fn set_goal_position(&mut self, pos: Option<&Coord3D>) {
        if self.locked == TRUE {
            return;
        }
        if let Some(pos) = pos {
            self.goal_position = *pos;
        }
    }

    pub fn is_goal_object_destroyed(&self, world: &Ctx) -> Bool {
        if self.goal_object_id == ObjectId::INVALID {
            return FALSE;
        }
        Bool::from(!world.object_exists(self.goal_object_id))
    }

    pub fn halt(&mut self) {
        self.locked = TRUE;
        self.current_state_id = None;
    }

    pub fn crc(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let _ = xfer;
        Ok(())
    }

    pub fn xfer(&mut self, xfer: &mut dyn Xfer, ctx: &mut Ctx) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;

        xfer.xfer_unsigned_int(&mut self.sleep_till)?;
        xfer.xfer_unsigned_int(&mut self.default_state_id)?;

        let mut cur_state_id = self.get_current_state_id();
        xfer.xfer_unsigned_int(&mut cur_state_id)?;
        if xfer.mode() == XferMode::Load {
            if cur_state_id == INVALID_STATE_ID
                || cur_state_id == MACHINE_DONE_STATE_ID
                || !self.states.contains_key(&cur_state_id)
            {
                cur_state_id = self.default_state_id;
            }
            self.current_state_id = self
                .states
                .contains_key(&cur_state_id)
                .then_some(cur_state_id);
        }

        let mut snapshot_all_states = FALSE;
        xfer.xfer_bool(&mut snapshot_all_states)?;

        if snapshot_all_states == TRUE {
            let count = self.states.len();
            let mut save_count = count as Int;
            xfer.xfer_int(&mut save_count)?;
            if xfer.mode() == XferMode::Load {
                if save_count as usize != count {
                    return Err(XferError::InvalidData);
                }
            }
            let ids: Vec<StateId> = self.states.keys().copied().collect();
            for id in ids {
                let mut saved_id = id;
                xfer.xfer_unsigned_int(&mut saved_id)?;
                if saved_id != id {
                    return Err(XferError::InvalidData);
                }
                if let Some(entry) = self.states.get(&id) {
                    let callback = entry.callbacks.xfer_state;
                    callback(id, xfer, ctx)?;
                }
            }
        } else {
            if xfer.mode() == XferMode::Load
                && (cur_state_id == INVALID_STATE_ID
                    || cur_state_id == MACHINE_DONE_STATE_ID
                    || !self.states.contains_key(&cur_state_id))
            {
                cur_state_id = self.default_state_id;
                self.current_state_id = self
                    .states
                    .contains_key(&cur_state_id)
                    .then_some(cur_state_id);
            }
            if self.current_state_id.is_none() {
                let healed_id = if self.states.contains_key(&self.default_state_id) {
                    Some(self.default_state_id)
                } else {
                    self.states.keys().next().copied()
                };
                if let Some(id) = healed_id {
                    self.current_state_id = Some(id);
                }
            }
            if let Some(id) = self.current_state_id {
                if let Some(entry) = self.states.get(&id) {
                    let callback = entry.callbacks.xfer_state;
                    callback(id, xfer, ctx)?;
                }
            }
        }

        xfer.xfer_object_id(&mut self.goal_object_id)?;
        xfer.xfer_coord3d(&mut self.goal_position)?;
        xfer.xfer_bool(&mut self.locked)?;
        xfer.xfer_bool(&mut self.default_state_inited)?;
        Ok(())
    }

    pub fn load_post_process(&mut self) {}
}
