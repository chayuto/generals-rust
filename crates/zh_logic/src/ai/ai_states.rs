//! `PKT-429`: `AIStates.cpp`, so far `AIIdleState`, `AIDeadState`, `AIBusyState`,
//! the movement states ([`movement`]) and the attack states ([`attack`]). Packet
//! doc: `packets/phase6/PKT-429_AIStates.md`. Golden test:
//! `tests/golden_s8_ai_orders.rs`.
//!
//! Every other state of `AIStateMachine` (the rest of the non-combat states S11,
//! squad, hunt and attack area, guard S18) is answered by
//! [`crate::ai::seams::AiSeams`].
//! A port adds its arm to the three functions here and to
//! `AiStateMachine::xfer_state`.

pub mod attack;
pub mod movement;

use crate::ai::ai_state_machine::{self as machine, state_id};
use crate::ai::seams::ai_seams;
use crate::arena::ModuleContext;
use crate::modules::module::CommandSourceType;
use crate::modules::update::ai_update::{
    self, ai, AiCommandParms, AiCommandType, MoodMatrixAction,
};
use crate::modules::update::physics_update;
use crate::object::weapon::NO_MAX_SHOTS_LIMIT;
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::KindOfType;
use crate::rules::model_state::{ModelConditionFlagType, ModelConditionFlags};
use crate::sim::SimApi;
use zh_common::core::state_machine::{StateExitType, StateId, StateReturnType};
use zh_common::ini::LOGICFRAMES_PER_SECOND;
use zh_common::types::{Bool, Int, UnsignedInt, UnsignedShort, XferVersion, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// `IDLE_COUNTDOWN_DELAY`.
pub const IDLE_COUNTDOWN_DELAY: UnsignedInt = LOGICFRAMES_PER_SECOND as UnsignedInt * 2;

const MAA_AFFECT_RANGE_IGNORE_ALL: UnsignedInt = 0x0000_0010;

/// The members of an `AIIdleState`.
#[derive(Debug, Clone, Copy)]
pub struct IdleState {
    pub initial_sleep_offset: UnsignedShort,
    pub should_look_for_targets: Bool,
    pub inited: Bool,
}

impl IdleState {
    pub const fn new(should_look_for_targets: Bool) -> Self {
        Self {
            initial_sleep_offset: UnsignedShort::MAX,
            should_look_for_targets,
            inited: FALSE,
        }
    }

    /// `AIIdleState::xfer`.
    pub fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        xfer.xfer_unsigned_short(&mut self.initial_sleep_offset)?;
        xfer.xfer_bool(&mut self.should_look_for_targets)?;
        xfer.xfer_bool(&mut self.inited)
    }
}

/// `AIIdleState::onEnter`, for the machine's own idle state and for those of the
/// attack-move sub machines. The caller stores the draw.
pub fn idle_on_enter(sim: &mut dyn SimApi) -> UnsignedShort {
    sim.rng_logic_int(0, IDLE_COUNTDOWN_DELAY as Int) as UnsignedShort
}

pub fn on_enter(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> StateReturnType {
    match id {
        state_id::IDLE => {
            ai_update::reset_next_mood_check_time(sim, ctx);
            ai(sim, ctx).machine.idle.inited = TRUE;
            let offset = idle_on_enter(sim);
            ai(sim, ctx).machine.idle.initial_sleep_offset = offset;
            StateReturnType::Continue
        }
        state_id::DEAD => dead_on_enter(sim, ctx),
        state_id::BUSY => StateReturnType::Continue,
        _ if is_ported_movement(sim, ctx, id) => movement::on_enter(sim, ctx, id),
        _ if is_ported_attack(sim, ctx, id) => {
            attack::on_enter(sim, ctx, attack::Host::Main, id)
        }
        _ => ai_seams(sim).state_enter(ctx.owner_id, id),
    }
}

pub fn update(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> StateReturnType {
    match id {
        state_id::IDLE => idle_update(sim, ctx),
        state_id::DEAD => dead_update(sim, ctx),
        state_id::BUSY => StateReturnType::Continue,
        _ if is_ported_movement(sim, ctx, id) => movement::update(sim, ctx, id),
        _ if is_ported_attack(sim, ctx, id) => attack::update(sim, ctx, attack::Host::Main),
        _ => ai_seams(sim).state_update(ctx.owner_id, id),
    }
}

pub fn on_exit(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId, status: StateExitType) {
    match id {
        state_id::IDLE | state_id::BUSY => {}
        state_id::DEAD => {
            sim.clear_model_condition_state(ctx.owner_slot, ModelConditionFlagType::Dying);
        }
        _ if is_ported_movement(sim, ctx, id) => movement::on_exit(sim, ctx, id),
        _ if is_ported_attack(sim, ctx, id) => {
            attack::on_exit(sim, ctx, attack::Host::Main, status)
        }
        _ => ai_seams(sim).state_exit(ctx.owner_id, id, status),
    }
}

/// A movement state that the seams do not answer in its stead.
fn is_ported_movement(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> bool {
    movement::is_movement_state(id) && ai_seams(sim).replaces_state(ctx.owner_id, id) == FALSE
}

/// An attack state that the seams do not answer in its stead.
fn is_ported_attack(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> bool {
    attack::is_attack_state(id) && ai_seams(sim).replaces_state(ctx.owner_id, id) == FALSE
}

/// `AIIdleState::doInitIdleState`.
fn do_init_idle_state(sim: &mut dyn SimApi, ctx: ModuleContext) {
    if ai(sim, ctx).machine.idle.inited == FALSE {
        return;
    }
    ai(sim, ctx).machine.idle.inited = FALSE;
    let ultra_accurate = ai(sim, ctx)
        .cur_locomotor()
        .map_or(FALSE, |l| l.is_ultra_accurate());
    if ai(sim, ctx).is_idle() != FALSE && ai_update::is_doing_ground_movement(sim, ctx) != FALSE {
        let Some((mut goal, layer)) = sim
            .object(ctx.owner_slot)
            .map(|o| (*o.thing.get_position(), o.layer))
        else {
            return;
        };
        if goal.x != 0.0 || goal.y != 0.0 || goal.z != 0.0 {
            ai_update::update_goal(sim, ctx, &goal, layer);
            if ultra_accurate == FALSE {
                let adjusted = ai_update::goal_position(sim, ctx);
                if let Some(pos) = adjusted {
                    goal = pos;
                    if sim.frame() <= 1 {
                        sim.set_object_position(ctx.owner_slot, &goal);
                    }
                    ai_update::update_goal(sim, ctx, &goal, layer);
                }
            }
        }
    }
    ai_update::set_locomotor_goal_none(sim, ctx);
    ai_update::set_current_victim(sim, ctx, None);
}

/// `AIIdleState::update`.
fn idle_update(sim: &mut dyn SimApi, ctx: ModuleContext) -> StateReturnType {
    do_init_idle_state(sim, ctx);

    let state = &mut ai(sim, ctx).machine.idle;
    let old_offset = state.initial_sleep_offset;
    let mut time_to_sleep = IDLE_COUNTDOWN_DELAY.wrapping_add(UnsignedInt::from(old_offset));
    state.initial_sleep_offset = 0;
    let should_look = state.should_look_for_targets;

    if should_look != FALSE && ai(sim, ctx).machine.locked == FALSE {
        let owner = ctx.owner_slot;
        if sim.object_is_kind_of(owner, KindOfType::CanBeRepulsed) != FALSE
            && ai(sim, ctx).is_idle() != FALSE
        {
            let range = sim.object_vision_range(owner);
            let enemy = ai_seams(sim).closest_repulsor(ctx.owner_id, range);
            if enemy.is_some() {
                machine::set_state(sim, ctx, state_id::MOVE_AWAY_FROM_REPULSORS);
                return StateReturnType::Continue;
            }
        }

        if let Some(crate_id) = ai_update::check_for_crate_to_pickup(sim, ctx) {
            let mut parms =
                AiCommandParms::new(AiCommandType::MoveToObject, CommandSourceType::FromAi);
            parms.obj = Some(crate_id);
            ai_update::execute_command(sim, ctx, &parms);
            return StateReturnType::Continue;
        }

        let disabled = sim.object(owner).is_some_and(|o| {
            [
                DisabledType::Paralyzed,
                DisabledType::Unmanned,
                DisabledType::Emp,
                DisabledType::Subdued,
                DisabledType::Hacked,
            ]
            .iter()
            .any(|&d| o.is_disabled_by_type(d) != FALSE)
        });
        if !disabled {
            let adjust = ai_update::mood_matrix_action_adjustment(sim, ctx, MoodMatrixAction::Idle);
            if adjust & MAA_AFFECT_RANGE_IGNORE_ALL == 0 {
                let enemy = ai_seams(sim).next_mood_target(ctx.owner_id, TRUE, TRUE);
                if let Some(enemy) = enemy {
                    let mut parms =
                        AiCommandParms::new(AiCommandType::AttackObject, CommandSourceType::FromAi);
                    parms.obj = Some(enemy);
                    parms.int_value = NO_MAX_SHOTS_LIMIT;
                    ai_update::execute_command(sim, ctx, &parms);
                    return StateReturnType::Continue;
                }
            }
        }

        let now = sim.frame();
        let next_check = ai(sim, ctx).next_mood_check_time;
        if next_check > now {
            let mood_sleep = next_check - now;
            if mood_sleep < time_to_sleep {
                time_to_sleep = mood_sleep;
                ai(sim, ctx).machine.idle.initial_sleep_offset = old_offset;
            }
        }
    }
    StateReturnType::Sleep(time_to_sleep)
}

/// `AIDeadState::onEnter`.
fn dead_on_enter(sim: &mut dyn SimApi, ctx: ModuleContext) -> StateReturnType {
    use ModelConditionFlagType as M;
    let mut non_dying = ModelConditionFlags::default();
    for flag in [
        M::UsingWeaponA,
        M::UsingWeaponB,
        M::UsingWeaponC,
        M::FiringA,
        M::FiringB,
        M::FiringC,
        M::BetweenFiringShotsA,
        M::BetweenFiringShotsB,
        M::BetweenFiringShotsC,
        M::ReloadingA,
        M::ReloadingB,
        M::ReloadingC,
        M::PreAttackA,
        M::PreAttackB,
        M::PreAttackC,
        M::Moving,
    ] {
        non_dying.set(flag.as_i32(), 1);
    }
    let mut dying = ModelConditionFlags::default();
    dying.set(M::Dying.as_i32(), 1);
    sim.clear_and_set_model_condition_flags(ctx.owner_slot, &non_dying, &dying);
    // `ScriptEngine::notifyOfObjectCreationOrDestruction`: scripts are not ported.
    StateReturnType::Continue
}

/// `AIDeadState::update`.
fn dead_update(sim: &mut dyn SimApi, ctx: ModuleContext) -> StateReturnType {
    ai_update::set_locomotor_goal_none(sim, ctx);
    if let Some(o) = sim.object_mut(ctx.owner_slot) {
        o.set_effectively_dead(TRUE);
    }
    if sim.object_is_kind_of(ctx.owner_slot, KindOfType::Infantry) != FALSE {
        if let Some(physics) = physics_update::physics_mut(sim, ctx.owner_slot) {
            let velocity = physics.velocity_magnitude();
            physics.scrub_velocity_2d(velocity * 0.8);
        }
    }
    StateReturnType::Continue
}
