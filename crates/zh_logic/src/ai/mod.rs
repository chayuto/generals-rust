//! The seams the unit AI port is built on (`docs/AI_PORT_PLAN.md`, slice S0).
//! The frame step that drives them is [`crate::gamelogic::frame`]; the trace test
//! is `tests/golden_s0_frame_driver.rs`.
//!
//! # A module that is called back while it runs
//!
//! An ordinary module is moved out of the arena for the length of a call, so a
//! call back into it is skipped and a query about it answers with a default. The
//! AI cannot live with that: its states order, damage and move things that call
//! straight back into it. So the AI module is **resident**: it is never moved out
//! of the arena to run. Its entry points are plain functions ([`ResidentEntries`])
//! that get the world and the module's address, and borrow the module's state
//! ([`SimApi::resident_state_mut`]) only for as long as one statement needs it.
//! Everything that calls into the module while one of them runs is delivered at
//! once and sees the current state, as through the C++ `this` pointer.
//!
//! The rule that keeps this true:
//!
//! * a `&mut self` method of a resident module (its facet methods included) only
//!   touches the module's own state, and calls nothing in the world that could
//!   come back;
//! * anything that does call out is an entry point, or a function called by one,
//!   taking `(&mut dyn SimApi, ModuleContext)` and never `&mut self`.
//!
//! Orders still arrive through `AiUpdateFacet` methods, which take `&mut self`.
//! That is safe for the module stand-ins there are now; the AI module port has to
//! turn the orders that act into entry points here.

pub mod seams;

use crate::arena::ModuleContext;
use crate::rules::disabled_types::DisabledMaskType;
use crate::sim::SimApi;
use zh_common::types::UpdateSleepTime;

/// The entry points of a resident module ([`crate::modules::BehaviorModule::resident`]).
#[derive(Clone)]
pub struct ResidentEntries {
    /// `getDisabledTypesToProcess`.
    pub disabled_types_to_process: DisabledMaskType,
    /// `update`.
    pub update: fn(&mut dyn SimApi, ModuleContext) -> UpdateSleepTime,
    /// `doPathfind`: the path queue reached this unit.
    pub do_pathfind: fn(&mut dyn SimApi, ModuleContext),
}
