//! Target finding: `AI::findClosestEnemy`, `findClosestAlly`, `findClosestRepulsor` and
//! `getAdjustedVisionRangeForObject`, with the partition filters they use.
//!
//! Source: `GameLogic/AI/AI.cpp`, the `PartitionFilter*::allow` functions of
//! `GameLogic/Object/PartitionManager.cpp` and `AttackPriorityInfo::getPriority`
//! of `GameLogic/ScriptEngine/ScriptEngine.cpp`. Part of PKT-421 (`AI`, no packet
//! document yet), slice S12 of `docs/AI_PORT_PLAN.md`; the rest of the class is
//! not here.
//! Golden test: `tests/golden_s12_targeting.rs`
//! (harness `tests/cpp_harness/gen_s12_targeting.cpp`).
//!
//! What these functions ask of code that is not ported goes through
//! [`TargetingSeams`].

use std::collections::BTreeMap;

use zh_common::math::{gm_sqrtf, Coord3D};
use zh_common::types::{Bool, Int, ObjectId, Real, Relationship, UnsignedInt, FALSE, TRUE};

use crate::arena::ObjectSlot;
use crate::modules::module::CommandSourceType;
use crate::object::weapon_fire::compute_bonus;
use crate::object::weapon_set::{CanAttackResult, WeaponSlotType};
use crate::rules::kind_of::{kindof_mask_fs, kindof_mask_none, KindOfType};
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimView, PRIVATE_STATUS_OFF_MAP};
use crate::spatial::partition_query::DistanceCalculationType;

/// `AI::findClosestEnemy` qualifiers.
pub const CAN_SEE: UnsignedInt = 1 << 0;
pub const CAN_ATTACK: UnsignedInt = 1 << 1;
pub const IGNORE_INSIGNIFICANT_BUILDINGS: UnsignedInt = 1 << 2;
pub const ATTACK_BUILDINGS: UnsignedInt = 1 << 3;
pub const WITHIN_ATTACK_RANGE: UnsignedInt = 1 << 4;
pub const UNFOGGED: UnsignedInt = 1 << 5;

/// `getAdjustedVisionRangeForObject` factors.
pub const AI_VISIONFACTOR_OWNERTYPE: Int = 0x01;
pub const AI_VISIONFACTOR_MOOD: Int = 0x02;
pub const AI_VISIONFACTOR_GUARDINNER: Int = 0x04;

/// `AbleToAttackType::ATTACK_NEW_TARGET`.
pub const ATTACK_NEW_TARGET: UnsignedInt = 0;

/// `MoodMatrixParameters`.
pub const MM_CONTROLLER_PLAYER: UnsignedInt = 0x0000_0001;
pub const MM_CONTROLLER_AI: UnsignedInt = 0x0000_0002;
pub const MM_UNIT_TYPE_NON_TURRETED: UnsignedInt = 0x0000_0010;
pub const MM_UNIT_TYPE_TURRETED: UnsignedInt = 0x0000_0020;
pub const MM_UNIT_TYPE_AIR: UnsignedInt = 0x0000_0040;
pub const MM_MOOD_SLEEP: UnsignedInt = 0x0000_0100;
pub const MM_MOOD_PASSIVE: UnsignedInt = 0x0000_0200;
pub const MM_MOOD_NORMAL: UnsignedInt = 0x0000_0400;
pub const MM_MOOD_ALERT: UnsignedInt = 0x0000_0800;
pub const MM_MOOD_AGGRESSIVE: UnsignedInt = 0x0000_1000;
pub const MM_MOOD_BITMASK: UnsignedInt =
    MM_MOOD_SLEEP | MM_MOOD_PASSIVE | MM_MOOD_NORMAL | MM_MOOD_ALERT | MM_MOOD_AGGRESSIVE;

const WWMATH_EPSILON: Real = 0.0001;

/// The members of `TAiData` that targeting and mood read, with the defaults of
/// its constructor.
#[derive(Debug, Clone, PartialEq)]
pub struct AiData {
    pub guard_inner_modifier_ai: Real,
    pub guard_outer_modifier_ai: Real,
    pub guard_inner_modifier_human: Real,
    pub guard_outer_modifier_human: Real,
    pub alert_range_modifier: Real,
    pub aggressive_range_modifier: Real,
    pub attack_priority_distance_modifier: Real,
    pub force_idle_frames_count: UnsignedInt,
    pub enable_repulsors: Bool,
    pub attack_uses_line_of_sight: Bool,
    pub attack_ignore_insignificant_buildings: Bool,
}

impl Default for AiData {
    fn default() -> Self {
        Self {
            guard_inner_modifier_ai: 0.0,
            guard_outer_modifier_ai: 0.0,
            guard_inner_modifier_human: 0.0,
            guard_outer_modifier_human: 0.0,
            alert_range_modifier: 0.0,
            aggressive_range_modifier: 0.0,
            attack_priority_distance_modifier: 0.0,
            force_idle_frames_count: 1,
            enable_repulsors: FALSE,
            attack_uses_line_of_sight: TRUE,
            attack_ignore_insignificant_buildings: FALSE,
        }
    }
}

/// An attack priority set other than the default one. The C++ keys the map by
/// the final override of the template; a template name stands for it here.
#[derive(Debug, Clone, PartialEq)]
pub struct AttackPriorityInfo {
    pub name: String,
    pub default_priority: Int,
    pub priorities: BTreeMap<String, Int>,
}

impl AttackPriorityInfo {
    /// `AttackPriorityInfo::getPriority`. `None` is a null template.
    pub fn get_priority(&self, template_name: Option<&str>) -> Int {
        template_name
            .and_then(|name| self.priorities.get(name))
            .copied()
            .unwrap_or(self.default_priority)
    }

    /// `getPriority` of the template of `object`.
    pub fn priority_of(&self, sim: &dyn SimView, object: ObjectSlot) -> Int {
        let name = sim.object(object).and_then(|o| {
            o.thing
                .template_override()
                .get_final(&sim.rules().thing_factory)
                .map(|t| t.get_name())
        });
        self.get_priority(name)
    }
}

/// The answers targeting needs from code that is not ported. A test answers them
/// from a tape of what the reference did.
pub trait TargetingSeams {
    /// `Object::getAbleToAttackSpecificObject(ATTACK_NEW_TARGET, other, CMD_FROM_AI)`.
    fn able_to_attack_specific_object(
        &mut self,
        sim: &dyn SimView,
        me: ObjectSlot,
        other: ObjectSlot,
    ) -> CanAttackResult {
        sim.able_to_attack_specific_object(me, ATTACK_NEW_TARGET, other, CommandSourceType::FromAi)
    }

    /// `Weapon::isWithinAttackRange(me, other)` of any weapon `me` holds
    /// (`PartitionFilterWithinAttackRange`).
    fn within_attack_range(&mut self, sim: &dyn SimView, me: ObjectSlot, other: ObjectSlot)
        -> Bool;

    /// `Object::getLargestWeaponRange`, ported; see [`largest_weapon_range`].
    fn largest_weapon_range(&mut self, sim: &dyn SimView, me: ObjectSlot) -> Real {
        largest_weapon_range(sim, me)
    }

    /// `Pathfinder::isViewBlockedByObstacle`.
    fn view_blocked_by_obstacle(
        &mut self,
        sim: &dyn SimView,
        me: ObjectSlot,
        other: ObjectSlot,
    ) -> Bool;

    /// `other->getShroudedStatus(player_index) == OBJECTSHROUD_CLEAR`.
    fn is_clear_of_fog(&mut self, sim: &dyn SimView, other: ObjectSlot, player_index: Int) -> Bool;

    /// The target of the team of `me`: `Team::getTeamTargetObject` if the team
    /// prototype has `m_attackCommonTarget`, `None` otherwise.
    fn team_target_object(&mut self, sim: &dyn SimView, me: ObjectSlot) -> Option<ObjectSlot>;
}

/// `Object::getLargestWeaponRange`: the longest attack range of the weapons held,
/// under their current bonuses; -1 without a weapon.
pub fn largest_weapon_range(sim: &dyn SimView, object: ObjectSlot) -> Real {
    let mut largest = -1.0;
    let Some(record) = sim.object(object) else {
        return largest;
    };
    for slot in [
        WeaponSlotType::Primary,
        WeaponSlotType::Secondary,
        WeaponSlotType::Tertiary,
    ] {
        let Some(weapon) = record.weapon_set.get_weapon_in_weapon_slot(slot) else {
            continue;
        };
        let bonus = compute_bonus(weapon, sim, object, 0);
        let range = weapon.get_template().get_attack_range(&bonus);
        if range > largest {
            largest = range;
        }
    }
    largest
}

fn has_status(sim: &dyn SimView, object: ObjectSlot, status: ObjectStatusType) -> bool {
    sim.object(object)
        .is_some_and(|o| o.status.test(status.as_i32()) != FALSE)
}

fn is_kind(sim: &dyn SimView, object: ObjectSlot, kind: KindOfType) -> bool {
    sim.object_is_kind_of(object, kind) != FALSE
}

fn is_dead(sim: &dyn SimView, object: ObjectSlot) -> bool {
    sim.object(object)
        .is_some_and(|o| o.is_effectively_dead() != FALSE)
}

fn is_off_map(sim: &dyn SimView, object: ObjectSlot) -> bool {
    sim.object(object)
        .is_some_and(|o| o.private_status & PRIVATE_STATUS_OFF_MAP != 0)
}

fn is_enemy(sim: &dyn SimView, me: ObjectSlot, other: ObjectSlot) -> bool {
    sim.player_relationship(me, other) == Relationship::Enemies
}

fn able_to_attack(sim: &dyn SimView, object: ObjectSlot) -> bool {
    sim.object_is_able_to_attack(object) != FALSE
}

/// `myPlayer->getRelationship(otherPlayer->getDefaultTeam()) == ENEMIES`, where
/// `myPlayer` controls `me`.
fn player_is_enemy_of_player(sim: &dyn SimView, me: ObjectSlot, other_player: Int) -> bool {
    sim.object(me).is_some_and(|o| {
        sim.team_relationship_to_player(o.team_id, other_player) == Relationship::Enemies
    })
}

/// `PartitionFilterLiveMapEnemies`.
fn allow_live_map_enemy(sim: &dyn SimView, me: ObjectSlot, other: ObjectSlot) -> bool {
    !is_dead(sim, other)
        && is_off_map(sim, other) == is_off_map(sim, me)
        && is_enemy(sim, me, other)
}

/// `PartitionFilterRejectBuildings`.
fn allow_reject_buildings(sim: &dyn SimView, me: ObjectSlot, other: ObjectSlot) -> bool {
    let Some((my_team, other_team)) = sim
        .object(me)
        .zip(sim.object(other))
        .map(|(a, b)| (a.team_id, b.team_id))
    else {
        return false;
    };
    if !is_kind(sim, other, KindOfType::Structure) {
        return true;
    }
    let apparent = if sim.object_has_contain(other) != FALSE {
        sim.contain_apparent_controlling_player(other, sim.player_index(my_team))
    } else {
        None
    };
    let other_player = apparent.unwrap_or_else(|| sim.player_index(other_team));
    if !player_is_enemy_of_player(sim, me, other_player) {
        return false;
    }
    // A computer player acquires every enemy building.
    if sim.player_is_human(my_team) == FALSE {
        return true;
    }
    is_kind(sim, other, KindOfType::FsBaseDefense)
        || (sim.object_has_contain(other) != FALSE && able_to_attack(sim, other))
}

/// `PartitionFilterInsignificantBuildings(true, false)`: non buildings pass,
/// insignificant buildings do not.
fn allow_insignificant_buildings(sim: &dyn SimView, other: ObjectSlot) -> bool {
    if !is_kind(sim, other, KindOfType::Structure) {
        return true;
    }
    let faction =
        sim.object_is_kind_of_multi(other, &kindof_mask_none(), &kindof_mask_fs()) == FALSE;
    if !faction
        && sim.object_has_contain(other) != FALSE
        && (sim.contain_is_garrisonable(other) == FALSE || sim.contain_count(other) == 0)
    {
        return false;
    }
    true
}

/// `PartitionFilterStealthedAndUndetected(me, false)`: `true` keeps `other`.
fn allow_not_stealthed(sim: &dyn SimView, me: ObjectSlot, other: ObjectSlot) -> bool {
    let stealthed = has_status(sim, other, ObjectStatusType::Stealthed);
    let detected = has_status(sim, other, ObjectStatusType::Detected);
    if stealthed && !detected {
        if !is_kind(sim, other, KindOfType::Disguiser) {
            return false;
        }
        if has_status(sim, other, ObjectStatusType::Disguised) {
            if let Some(update) = sim.stealth_params(other) {
                if update.disguised_template.is_some() && update.disguised_player_index >= 0 {
                    // Disguised as an enemy of ours, it counts as not stealthed.
                    return player_is_enemy_of_player(sim, me, update.disguised_player_index);
                }
            }
        }
    } else if sim.object_has_contain(other) != FALSE {
        let items = sim.contain_items(other);
        if let Some(&first) = items.first() {
            let observer = sim.object(me).map_or(0, |o| sim.player_index(o.team_id));
            let apparent = sim.contain_apparent_controlling_player(other, observer);
            if sim.contain_stealth_units_contained(other) == sim.contain_count(other) {
                let undetected = sim
                    .find_object(first)
                    .is_some_and(|m| !has_status(sim, m, ObjectStatusType::Detected));
                if undetected && apparent.is_some_and(|p| player_is_enemy_of_player(sim, me, p)) {
                    return false;
                }
            }
        }
    }
    true
}

/// `PartitionFilterLineOfSight`.
fn allow_line_of_sight(
    sim: &dyn SimView,
    seams: &mut dyn TargetingSeams,
    me: ObjectSlot,
    other: ObjectSlot,
) -> bool {
    sim.is_clear_line_of_sight(me, other) != FALSE
        && seams.view_blocked_by_obstacle(sim, me, other) == FALSE
}

/// The highest priority among `curr` and the objects inside `enemy`.
fn priority_with_contained(
    sim: &dyn SimView,
    info: &AttackPriorityInfo,
    enemy: ObjectSlot,
    current: Int,
) -> Int {
    if sim.object_has_contain(enemy) == FALSE {
        return current;
    }
    sim.contain_items(enemy)
        .into_iter()
        .filter_map(|id: ObjectId| sim.find_object(id))
        .fold(current, |best, item| best.max(info.priority_of(sim, item)))
}

/// `AI::findClosestEnemy`. `info` is `None` for a null and for the default
/// attack priority set, which behave alike.
#[allow(clippy::too_many_arguments)]
pub fn find_closest_enemy(
    sim: &dyn SimView,
    seams: &mut dyn TargetingSeams,
    ai_data: &AiData,
    me: ObjectSlot,
    range: Real,
    qualifiers: UnsignedInt,
    info: Option<&AttackPriorityInfo>,
    mut optional_filter: Option<&mut dyn FnMut(ObjectSlot) -> bool>,
) -> Option<ObjectSlot> {
    if qualifiers & CAN_ATTACK != 0 && !able_to_attack(sim, me) {
        return None;
    }
    let player = sim.object(me).map(|o| sim.player_index(o.team_id))?;

    let mut filter = |other: ObjectSlot| -> bool {
        if !allow_live_map_enemy(sim, me, other) {
            return false;
        }
        if qualifiers & ATTACK_BUILDINGS == 0 && !allow_reject_buildings(sim, me, other) {
            return false;
        }
        if qualifiers & WITHIN_ATTACK_RANGE != 0
            && seams.within_attack_range(sim, me, other) == FALSE
        {
            return false;
        }
        if qualifiers & CAN_SEE != 0 && !allow_line_of_sight(sim, seams, me, other) {
            return false;
        }
        if qualifiers & CAN_ATTACK != 0
            && !matches!(
                seams.able_to_attack_specific_object(sim, me, other),
                CanAttackResult::Possible | CanAttackResult::PossibleAfterMoving
            )
        {
            return false;
        }
        if qualifiers & UNFOGGED != 0 && seams.is_clear_of_fog(sim, other, player) == FALSE {
            return false;
        }
        if qualifiers & IGNORE_INSIGNIFICANT_BUILDINGS != 0
            && !allow_insignificant_buildings(sim, other)
        {
            return false;
        }
        if !allow_not_stealthed(sim, me, other) {
            return false;
        }
        optional_filter.as_mut().is_none_or(|f| f(other))
    };

    let rule = DistanceCalculationType::FromBoundingSphere2d;
    let Some(info) = info else {
        return sim.closest_object_from(me, range, rule, &mut filter);
    };

    let mut best = None;
    let mut effective_priority: Int = 0;
    let mut actual_priority: Int = 0;
    for enemy in sim.objects_from_near_to_far(me, range, rule, &mut filter) {
        let mut current = info.priority_of(sim, enemy);
        if current == 0 {
            continue;
        }
        // A container is worth the best of what it holds.
        current = priority_with_contained(sim, info, enemy, current);

        let dist_sqr = sim.partition_distance_squared(me, Some(enemy), &Coord3D::default(), rule);
        let dist = gm_sqrtf(dist_sqr);
        let divisor = if ai_data.attack_priority_distance_modifier < WWMATH_EPSILON {
            WWMATH_EPSILON
        } else {
            ai_data.attack_priority_distance_modifier
        };
        let modifier = (dist / divisor) as Int;
        let modified = current.wrapping_sub(modifier).max(1);
        if modified > effective_priority {
            effective_priority = modified;
            actual_priority = current;
            best = Some(enemy);
        }
        if modified == effective_priority && current > actual_priority {
            effective_priority = modified;
            actual_priority = current;
            best = Some(enemy);
        }
    }
    best
}

/// `AI::findClosestAlly`.
pub fn find_closest_ally(
    sim: &dyn SimView,
    seams: &mut dyn TargetingSeams,
    me: ObjectSlot,
    range: Real,
    qualifiers: UnsignedInt,
) -> Option<ObjectSlot> {
    let mut filter = |other: ObjectSlot| -> bool {
        allow_reject_buildings(sim, me, other)
            && sim.player_relationship(me, other) == Relationship::Allies
            && !is_dead(sim, other)
            && is_off_map(sim, other) == is_off_map(sim, me)
            && (qualifiers & CAN_SEE == 0 || allow_line_of_sight(sim, seams, me, other))
    };
    sim.closest_object_from(
        me,
        range,
        DistanceCalculationType::FromBoundingSphere2d,
        &mut filter,
    )
}

/// `PartitionFilterRepulsor`.
fn allow_repulsor(sim: &dyn SimView, me: ObjectSlot, other: ObjectSlot) -> bool {
    if other == me {
        return false;
    }
    if has_status(sim, other, ObjectStatusType::Repulsor) {
        return true;
    }
    if is_dead(sim, other) || !is_enemy(sim, me, other) {
        return false;
    }
    if is_kind(sim, other, KindOfType::Structure) {
        return able_to_attack(sim, other);
    }
    !is_kind(sim, other, KindOfType::Inert) && able_to_attack(sim, other)
}

/// `AI::findClosestRepulsor`.
pub fn find_closest_repulsor(
    sim: &dyn SimView,
    ai_data: &AiData,
    me: ObjectSlot,
    range: Real,
) -> Option<ObjectSlot> {
    if ai_data.enable_repulsors == FALSE {
        return None;
    }
    let mut filter = |other: ObjectSlot| -> bool {
        // The second filter rejects what is stealthed and neither detected nor disguised.
        allow_repulsor(sim, me, other)
            && !(has_status(sim, other, ObjectStatusType::Stealthed)
                && !has_status(sim, other, ObjectStatusType::Detected)
                && !has_status(sim, other, ObjectStatusType::Disguised))
    };
    sim.closest_object_from(
        me,
        range,
        DistanceCalculationType::FromBoundingSphere2d,
        &mut filter,
    )
}

/// `AI::getAdjustedVisionRangeForObject`. `mood_matrix` is the
/// `getMoodMatrixValue` of the object's AI, `None` without an AI.
pub fn adjusted_vision_range(
    sim: &dyn SimView,
    seams: &mut dyn TargetingSeams,
    ai_data: &AiData,
    object: ObjectSlot,
    mood_matrix: Option<UnsignedInt>,
    factors: Int,
) -> Real {
    let mut range = sim.object_vision_range(object);
    let Some(mood) = mood_matrix else {
        return 0.0;
    };

    if factors & AI_VISIONFACTOR_OWNERTYPE != 0 {
        let inner = factors & AI_VISIONFACTOR_GUARDINNER != 0;
        range *= match (mood & MM_CONTROLLER_PLAYER != 0, inner) {
            (true, true) => ai_data.guard_inner_modifier_human,
            (true, false) => ai_data.guard_outer_modifier_human,
            (false, true) => ai_data.guard_inner_modifier_ai,
            (false, false) => ai_data.guard_outer_modifier_ai,
        };
    }

    if sim.object(object).is_some_and(|o| o.contained_by.is_some()) {
        return seams.largest_weapon_range(sim, object);
    }
    if factors & AI_VISIONFACTOR_MOOD != 0 && mood & MM_CONTROLLER_PLAYER == 0 {
        match mood & MM_MOOD_BITMASK {
            MM_MOOD_SLEEP => return 0.0,
            MM_MOOD_ALERT => range *= ai_data.alert_range_modifier,
            MM_MOOD_AGGRESSIVE => range *= ai_data.aggressive_range_modifier,
            _ => {}
        }
    }
    range
}
