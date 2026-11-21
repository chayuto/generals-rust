//! `PKT-077`: Deterministic finite state machine (`GeneralsMD/Code/GameEngine/Source/Common/StateMachine.cpp`).
//!
//! Reference C++ packet: `packets/phase2/PKT-077_StateMachine.md`
//! Golden Vector test: `crates/zh_common/tests/golden_pkt077.rs` (`docs/golden_vectors/PKT-077.json`)

#![allow(unused_variables, dead_code)]

use std::collections::BTreeMap;

use crate::math::Coord3D;
use crate::types::{Bool, Int, ObjectId, UnsignedInt, FALSE, TRUE};
use crate::xfer::{Xfer, XferError};

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

#[derive(Clone, Copy)]
pub struct StateCondition<Ctx> {
    pub test: StateTransFn<Ctx>,
    pub to_state_id: StateId,
}

#[derive(Clone, Copy)]
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
        let _ = (id, callbacks, success_id, failure_id, conditions);
        todo!("PKT-077: implement StateMachine::defineState")
    }

    pub fn init_default_state(&mut self, ctx: &mut Ctx) -> StateReturnType {
        let _ = ctx;
        todo!("PKT-077: implement StateMachine::initDefaultState")
    }

    pub fn update_state_machine(&mut self, ctx: &mut Ctx) -> StateReturnType {
        let _ = ctx;
        todo!("PKT-077: implement StateMachine::updateStateMachine")
    }

    pub fn clear(&mut self, ctx: &mut Ctx) {
        let _ = ctx;
        todo!("PKT-077: implement StateMachine::clear")
    }

    pub fn reset_to_default_state(&mut self, ctx: &mut Ctx) -> StateReturnType {
        let _ = ctx;
        todo!("PKT-077: implement StateMachine::resetToDefaultState")
    }

    pub fn set_state(&mut self, new_state_id: StateId, ctx: &mut Ctx) -> StateReturnType {
        let _ = (new_state_id, ctx);
        todo!("PKT-077: implement StateMachine::setState")
    }

    pub fn internal_set_state(
        &mut self,
        new_state_id: StateId,
        ctx: &mut Ctx,
    ) -> StateReturnType {
        let _ = (new_state_id, ctx);
        todo!("PKT-077: implement StateMachine::internalSetState")
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
            Some(id) => self
                .states
                .get(&id)
                .map_or(TRUE, |s| s.callbacks.is_idle),
            None => TRUE,
        }
    }

    pub fn is_in_attack_state(&self) -> Bool {
        match self.current_state_id {
            Some(id) => self
                .states
                .get(&id)
                .map_or(TRUE, |s| s.callbacks.is_attack),
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
            Some(id) => self
                .states
                .get(&id)
                .map_or(FALSE, |s| s.callbacks.is_busy),
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
        let _ = obj;
        todo!("PKT-077: implement StateMachine::setGoalObject")
    }

    pub fn set_goal_position(&mut self, pos: Option<&Coord3D>) {
        let _ = pos;
        todo!("PKT-077: implement StateMachine::setGoalPosition")
    }

    pub fn is_goal_object_destroyed(&self, world: &Ctx) -> Bool {
        let _ = world;
        todo!("PKT-077: implement StateMachine::isGoalObjectDestroyed")
    }

    pub fn halt(&mut self) {
        todo!("PKT-077: implement StateMachine::halt")
    }

    pub fn crc(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let _ = xfer;
        Ok(())
    }

    pub fn xfer(&mut self, xfer: &mut dyn Xfer, ctx: &mut Ctx) -> Result<(), XferError> {
        let _ = (xfer, ctx);
        todo!("PKT-077: implement StateMachine::xfer")
    }

    pub fn load_post_process(&mut self) {}
}
