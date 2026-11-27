//! Mood and target choice of a unit's AI: `getMoodMatrixValue`,
//! `getMoodMatrixActionAdjustment`, the mood check timer,
//! `canAutoAcquireWhileStealthed` and `getNextMoodTarget`.
//!
//! Source: `GameLogic/Object/Update/AIUpdate.cpp`, slice S12 of
//! `docs/AI_PORT_PLAN.md`. Part of PKT-301 (`AIUpdate`), whose file
//! `modules/update/ai_update.rs` is not written by this slice; the AI module
//! keeps a [`MoodState`], describes itself in a [`MoodUnit`] and calls in here.
//! `evaluateMoraleBonus` of the same class is
//! `modules::update::horde_update::evaluate_morale_bonus`.
//! Golden test: `tests/golden_s12_targeting.rs`
//! (harness `tests/cpp_harness/gen_s12_targeting.cpp`).

use zh_common::types::{Bool, Int, ObjectId, UnsignedInt, FALSE, TRUE};

use super::ai::{
    adjusted_vision_range, find_closest_enemy, AiData, AttackPriorityInfo, TargetingSeams,
    AI_VISIONFACTOR_MOOD, AI_VISIONFACTOR_OWNERTYPE, ATTACK_BUILDINGS, CAN_ATTACK, CAN_SEE,
    IGNORE_INSIGNIFICANT_BUILDINGS, MM_CONTROLLER_AI, MM_CONTROLLER_PLAYER, MM_MOOD_AGGRESSIVE,
    MM_MOOD_ALERT, MM_MOOD_BITMASK, MM_MOOD_NORMAL, MM_MOOD_PASSIVE, MM_MOOD_SLEEP,
    MM_UNIT_TYPE_AIR, MM_UNIT_TYPE_NON_TURRETED, MM_UNIT_TYPE_TURRETED, UNFOGGED,
    WITHIN_ATTACK_RANGE,
};
use crate::arena::ObjectSlot;
use crate::modules::module::AttitudeType;
use crate::object::weapon_set::CanAttackResult;
use crate::rules::combat_types::{DamageType, LocomotorSurfaceTypeMask};
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::SimApi;
use crate::sim::SimView;

/// `AutoAcquireEnemiesWhenIdle`.
pub const AAS_IDLE: UnsignedInt = 0x01;
pub const AAS_IDLE_STEALTHED: UnsignedInt = 0x02;
pub const AAS_IDLE_NO: UnsignedInt = 0x04;
pub const AAS_IDLE_NOT_WHILE_ATTACKING: UnsignedInt = 0x08;
pub const AAS_IDLE_ATTACK_BUILDINGS: UnsignedInt = 0x10;

/// `MoodMatrixAdjustment`.
pub const MAA_ACTION_OK: UnsignedInt = 0x01;
pub const MAA_ACTION_TO_IDLE: UnsignedInt = 0x02;
pub const MAA_ACTION_TO_ATTACK_MOVE: UnsignedInt = 0x04;
pub const MAA_AFFECT_RANGE_IGNORE_ALL: UnsignedInt = 0x10;
pub const MAA_AFFECT_RANGE_WAIT_FOR_ATTACK: UnsignedInt = 0x20;
pub const MAA_AFFECT_RANGE_ALERT: UnsignedInt = 0x40;
pub const MAA_AFFECT_RANGE_AGGRESSIVE: UnsignedInt = 0x80;

/// `LOCOMOTORSURFACE_AIR`.
const LOCOMOTORSURFACE_AIR: LocomotorSurfaceTypeMask = 1 << 3;

/// `MoodMatrixAction`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoodMatrixAction {
    Idle,
    Move,
    Attack,
    AttackMove,
}

/// `m_nextMoodCheckTime` and `m_randomlyOffsetMoodCheck`; the AI saves both.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MoodState {
    pub next_mood_check_time: UnsignedInt,
    pub randomly_offset_mood_check: Bool,
}

impl MoodState {
    /// `wakeUpAndAttemptToTarget`: only an idle unit looks again at once. It does
    /// not wake the module.
    pub fn wake_up_and_attempt_to_target(&mut self, is_idle: Bool, now: UnsignedInt) {
        if is_idle == FALSE {
            return;
        }
        self.next_mood_check_time = now;
        self.randomly_offset_mood_check = TRUE;
    }

    /// `resetNextMoodCheckTime`.
    pub fn reset_next_mood_check_time(&mut self, now: UnsignedInt, ai_data: &AiData) {
        self.next_mood_check_time = now.wrapping_add(ai_data.force_idle_frames_count);
        self.randomly_offset_mood_check = TRUE;
    }

    /// `setNextMoodCheckTime`.
    pub fn set_next_mood_check_time(&mut self, frame: UnsignedInt) {
        self.next_mood_check_time = frame;
        self.randomly_offset_mood_check = FALSE;
    }
}

/// What the mood code asks an AI about itself.
#[derive(Debug, Clone, Copy)]
pub struct MoodUnit<'a> {
    /// `getStateMachine() != nullptr`.
    pub has_state_machine: Bool,
    pub attitude: AttitudeType,
    /// `getLocomotorSet().getValidSurfaces()`.
    pub valid_surfaces: LocomotorSurfaceTypeMask,
    /// `m_turretAI[0] != nullptr`.
    pub has_primary_turret: Bool,
    /// `isAttacking()`.
    pub is_attacking: Bool,
    /// Module data `AutoAcquireEnemiesWhenIdle`.
    pub auto_acquire_enemies_when_idle: UnsignedInt,
    /// Module data `MoodAttackCheckRate`.
    pub mood_attack_check_rate: UnsignedInt,
    /// `getAttackInfo()`; `None` for a null or the default set.
    pub attack_info: Option<&'a AttackPriorityInfo>,
}

/// `getMoodMatrixValue`.
pub fn mood_matrix_value(sim: &dyn SimView, owner: ObjectSlot, unit: &MoodUnit<'_>) -> UnsignedInt {
    if unit.has_state_machine == FALSE {
        return 0;
    }
    let Some(object) = sim.object(owner) else {
        return 0;
    };
    let mut value = if sim.player_is_human(object.team_id) != FALSE {
        MM_CONTROLLER_PLAYER
    } else {
        MM_CONTROLLER_AI
            | match unit.attitude {
                AttitudeType::Sleep => MM_MOOD_SLEEP,
                AttitudeType::Passive => MM_MOOD_PASSIVE,
                AttitudeType::Normal | AttitudeType::Invalid => MM_MOOD_NORMAL,
                AttitudeType::Alert => MM_MOOD_ALERT,
                AttitudeType::Aggressive => MM_MOOD_AGGRESSIVE,
            }
    };
    value |= if unit.valid_surfaces & LOCOMOTORSURFACE_AIR != 0 {
        MM_UNIT_TYPE_AIR
    } else if unit.has_primary_turret != FALSE {
        MM_UNIT_TYPE_TURRETED
    } else {
        MM_UNIT_TYPE_NON_TURRETED
    };
    value
}

/// `getMoodMatrixActionAdjustment`.
pub fn mood_matrix_action_adjustment(
    sim: &dyn SimView,
    owner: ObjectSlot,
    unit: &MoodUnit<'_>,
    action: MoodMatrixAction,
) -> UnsignedInt {
    // Stinger soldiers and their like obey no mood.
    if sim.object_is_kind_of(owner, KindOfType::Infantry) != FALSE
        && sim.object_is_kind_of(owner, KindOfType::IgnoredInGui) != FALSE
    {
        return MAA_ACTION_OK;
    }
    let matrix = mood_matrix_value(sim, owner, unit);
    if matrix & MM_CONTROLLER_PLAYER != 0 {
        return MAA_ACTION_OK;
    }
    use MoodMatrixAction as Action;
    match (action, matrix & MM_MOOD_BITMASK) {
        (Action::Idle, MM_MOOD_SLEEP) => MAA_ACTION_OK | MAA_AFFECT_RANGE_IGNORE_ALL,
        (Action::Idle | Action::Move, MM_MOOD_PASSIVE) => {
            MAA_ACTION_OK | MAA_AFFECT_RANGE_WAIT_FOR_ATTACK
        }
        (Action::Idle | Action::AttackMove, MM_MOOD_ALERT) => {
            MAA_ACTION_OK | MAA_AFFECT_RANGE_ALERT
        }
        (Action::Idle | Action::AttackMove, MM_MOOD_AGGRESSIVE) => {
            MAA_ACTION_OK | MAA_AFFECT_RANGE_AGGRESSIVE
        }
        (Action::Move | Action::Attack | Action::AttackMove, MM_MOOD_SLEEP) => {
            MAA_ACTION_TO_IDLE | MAA_AFFECT_RANGE_IGNORE_ALL
        }
        (Action::Move, MM_MOOD_ALERT) => MAA_ACTION_TO_ATTACK_MOVE | MAA_AFFECT_RANGE_ALERT,
        (Action::Move, MM_MOOD_AGGRESSIVE) => {
            MAA_ACTION_TO_ATTACK_MOVE | MAA_AFFECT_RANGE_AGGRESSIVE
        }
        _ => MAA_ACTION_OK,
    }
}

/// `canAutoAcquireWhileStealthed`.
pub fn can_auto_acquire_while_stealthed(
    sim: &dyn SimView,
    owner: ObjectSlot,
    unit: &MoodUnit<'_>,
) -> Bool {
    if sim
        .stealth_params(owner)
        .is_some_and(|s| s.granted_by_special_power != FALSE)
    {
        return TRUE;
    }
    Bool::from(unit.auto_acquire_enemies_when_idle & AAS_IDLE_STEALTHED != 0)
}

/// Whether a unit may look for targets at all, before any timer.
fn may_acquire(
    sim: &dyn SimView,
    owner: ObjectSlot,
    unit: &MoodUnit<'_>,
    called_during_idle: Bool,
) -> bool {
    let Some(object) = sim.object(owner) else {
        return false;
    };
    let status = |s: ObjectStatusType| object.status.test(s.as_i32()) != FALSE;
    let flags = unit.auto_acquire_enemies_when_idle;
    if object.is_effectively_dead() != FALSE || status(ObjectStatusType::IsUsingAbility) {
        return false;
    }
    if called_during_idle != FALSE && flags & AAS_IDLE == 0 {
        return false;
    }
    if unit.is_attacking != FALSE && flags & AAS_IDLE_NOT_WHILE_ATTACKING != 0 {
        return false;
    }
    // A stealthed unit would reveal itself, unless it fires from inside something.
    if called_during_idle != FALSE
        && status(ObjectStatusType::Stealthed)
        && can_auto_acquire_while_stealthed(sim, owner, unit) == FALSE
    {
        return object.contained_by.is_some_and(|container| {
            sim.contain_is_passenger_allowed_to_fire(container, ObjectId::INVALID) != FALSE
        });
    }
    true
}

/// `getNextMoodTarget`. It draws one logic random number when it spreads the
/// next check of an AI caller.
pub fn next_mood_target(
    sim: &mut dyn SimApi,
    seams: &mut dyn TargetingSeams,
    ai_data: &AiData,
    owner: ObjectSlot,
    unit: &MoodUnit<'_>,
    state: &mut MoodState,
    called_by_ai: Bool,
    called_during_idle: Bool,
) -> Option<ObjectSlot> {
    if !may_acquire(sim, owner, unit, called_during_idle) {
        return None;
    }
    let now = sim.frame();

    if called_by_ai != FALSE {
        let team_victim = seams.team_target_object(sim, owner).filter(|&victim| {
            matches!(
                seams.able_to_attack_specific_object(sim, owner, victim),
                CanAttackResult::Possible | CanAttackResult::PossibleAfterMoving
            )
        });
        if let Some(victim) = team_victim {
            if unit.attitude as i32 >= AttitudeType::Normal as i32 {
                return Some(victim);
            }
        }

        if now < state.next_mood_check_time {
            return None;
        }
        let check_rate = unit.mood_attack_check_rate as Int;
        state.next_mood_check_time = now.wrapping_add(check_rate as UnsignedInt);
        if state.randomly_offset_mood_check != FALSE {
            let half = check_rate >> 1;
            let offset = sim.rng_logic_int(half.wrapping_neg(), half);
            state.next_mood_check_time =
                (state.next_mood_check_time as Int).wrapping_add(offset) as UnsignedInt;
            state.randomly_offset_mood_check = FALSE;
        }
    }

    let matrix = mood_matrix_value(sim, owner, unit);
    let mut range = adjusted_vision_range(
        sim,
        seams,
        ai_data,
        owner,
        Some(matrix),
        AI_VISIONFACTOR_OWNERTYPE | AI_VISIONFACTOR_MOOD,
    );
    if range <= 0.0 {
        return None;
    }
    let (team, container) = sim.object(owner).map(|o| (o.team_id, o.contained_by))?;
    if let Some(container) = container.and_then(|c| sim.object(c)) {
        range += container.geometry.get_bounding_circle_radius();
    }

    // A passive computer unit only answers whoever hurt it.
    if matrix & MM_CONTROLLER_AI != 0 && matrix & MM_MOOD_PASSIVE != 0 {
        if sim.object_last_damage_type(owner)? != DamageType::Healing {
            let (source, _) = sim.object_last_damage(owner)?;
            return sim.find_object(source);
        }
    }

    let human = sim.player_is_human(team) != FALSE;
    let mut flags = CAN_ATTACK;
    if ai_data.attack_uses_line_of_sight != FALSE
        && sim.object_is_kind_of(owner, KindOfType::AttackNeedsLineOfSight) != FALSE
    {
        flags |= CAN_SEE;
    }
    if ai_data.attack_ignore_insignificant_buildings != FALSE {
        flags |= IGNORE_INSIGNIFICANT_BUILDINGS;
    }
    if unit.auto_acquire_enemies_when_idle & AAS_IDLE_ATTACK_BUILDINGS != 0 {
        flags |= ATTACK_BUILDINGS;
    }
    // A human player's idle unit does not walk off to fight, nor see into fog.
    if called_by_ai != FALSE && human {
        flags |= WITHIN_ATTACK_RANGE | UNFOGGED;
    }

    find_closest_enemy(
        sim,
        seams,
        ai_data,
        owner,
        range,
        flags,
        unit.attack_info,
        None,
    )
}
