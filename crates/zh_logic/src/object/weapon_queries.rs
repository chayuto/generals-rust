//! The questions a weapon answers before it fires (`Weapon.cpp`): attack range, approach
//! position, terrain line of sight, target pitch and the damage estimate.
//!
//! Packet PKT-399. Golden test: `crates/zh_logic/tests/golden_s13_weapon_firing.rs`.
//!
//! Questions only unported code can answer go through [`crate::ai::seams::AiSeams`]: bridge
//! records and the destination adjustment of aircraft. Line of sight is asked of the terrain.

use zh_common::math::geometry::{GeometryInfo, GeometryType};
use zh_common::math::{self, gm_fabsf, Coord3D};
use zh_common::types::{Bool, Int, ObjectId, Real, FALSE, TRUE};

use super::weapon::{Weapon, WeaponReloadType, WeaponTemplate, PATHFIND_CELL_SIZE_F};
use super::weapon_fire::compute_bonus;
use crate::ai::seams::ai_seams;
use crate::arena::ObjectSlot;
use crate::combat::damage::DamageInfo;
use crate::gamelogic::global_data::WeaponBonus;
use crate::rules::combat_types::{DamageType, DeathType, WeaponStatus};
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::SimView;
use crate::spatial::partition_grid::PartitionObject;
use crate::spatial::partition_query::{measure, DistanceCalculationType};

/// `PI` (`BaseType.h`).
const PI: Real = std::f32::consts::PI;
/// `ATTACK_RANGE_CALC_TYPE`.
const ATTACK_RANGE: DistanceCalculationType = DistanceCalculationType::FromBoundingSphere2d;

fn sqr(x: Real) -> Real {
    x * x
}

fn position_of(sim: &dyn SimView, slot: ObjectSlot) -> Coord3D {
    sim.object(slot)
        .map(|o| *o.thing.get_position())
        .unwrap_or_default()
}

fn is_kind(sim: &dyn SimView, slot: ObjectSlot, kind: KindOfType) -> bool {
    sim.object_is_kind_of(slot, kind) != FALSE
}

/// An object as the distance procedures see it, or a bare position without geometry.
fn distance_end(sim: &dyn SimView, slot: Option<ObjectSlot>, pos: &Coord3D) -> PartitionObject {
    let object = slot.and_then(|s| sim.object(s));
    PartitionObject {
        id: object.map_or(ObjectId::INVALID, |o| o.id),
        pos: object.map_or(*pos, |o| *o.thing.get_position()),
        angle: 0.0,
        geometry: object.map_or_else(
            || GeometryInfo::new(GeometryType::Sphere, TRUE, 0.0, 0.0, 0.0),
            |o| o.geometry.clone(),
        ),
        is_immobile: FALSE,
        build_cost: 0,
    }
}

/// `PartitionManager::getGoalDistanceSquared`: the distance `source` would have standing at
/// `goal_pos`.
fn goal_distance_squared(
    sim: &dyn SimView,
    source: ObjectSlot,
    goal_pos: &Coord3D,
    target: Option<ObjectSlot>,
    target_pos: &Coord3D,
) -> Real {
    let from = distance_end(sim, Some(source), goal_pos);
    let to = distance_end(sim, target, target_pos);
    measure(ATTACK_RANGE, goal_pos, Some(&from), &to).0
}

/// `PartitionManager::getVectorTo`.
fn vector_to(
    sim: &dyn SimView,
    from: ObjectSlot,
    to: Option<ObjectSlot>,
    to_pos: &Coord3D,
) -> Coord3D {
    let from = distance_end(sim, Some(from), to_pos);
    let to = distance_end(sim, to, to_pos);
    measure(ATTACK_RANGE, &from.pos, Some(&from), &to).1
}

fn bridge_attack_points(sim: &dyn SimView, bridge: ObjectSlot) -> [Coord3D; 2] {
    let pos = position_of(sim, bridge);
    let id = sim.object(bridge).map_or(ObjectId::INVALID, |o| o.id);
    ai_seams(sim).bridge_attack_points(id, &pos)
}

/// The attack point of a bridge that is nearer to `source`'s bounding sphere.
pub(crate) fn closest_bridge_attack_point(
    sim: &dyn SimView,
    source: ObjectSlot,
    bridge: ObjectSlot,
) -> Coord3D {
    let [first, second] = bridge_attack_points(sim, bridge);
    let distance = |p: &Coord3D| {
        let from = distance_end(sim, Some(source), p);
        let to = distance_end(sim, None, p);
        measure(
            DistanceCalculationType::FromBoundingSphere3d,
            &from.pos,
            Some(&from),
            &to,
        )
        .0
    };
    if distance(&first) > distance(&second) {
        second
    } else {
        first
    }
}

/// The distance to a bridge: its first attack point, or the second when the first is out of
/// `range_sqr`.
fn bridge_distance_squared(
    range_sqr: Real,
    sim: &dyn SimView,
    bridge: ObjectSlot,
    distance_to: &dyn Fn(&Coord3D) -> Real,
) -> Real {
    let points = bridge_attack_points(sim, bridge);
    let first = distance_to(&points[0]);
    if first > range_sqr {
        distance_to(&points[1])
    } else {
        first
    }
}

/// `Weapon::getAttackRange`.
pub fn attack_range(weapon: &Weapon, sim: &dyn SimView, source: ObjectSlot) -> Real {
    let bonus = compute_bonus(weapon, sim, source, 0);
    weapon.template.get_attack_range(&bonus)
}

/// `Weapon::getAttackDistance`: the range plus both bounding circles.
pub fn attack_distance(
    weapon: &Weapon,
    sim: &dyn SimView,
    source: ObjectSlot,
    victim: Option<ObjectSlot>,
) -> Real {
    let mut range = attack_range(weapon, sim, source);
    if let (Some(s), Some(v)) = (sim.object(source), victim.and_then(|v| sim.object(v))) {
        range += s.geometry.get_bounding_circle_radius();
        range += v.geometry.get_bounding_circle_radius();
    }
    range
}

/// `Weapon::getPrimaryDamageRadius`.
pub fn primary_damage_radius(weapon: &Weapon, sim: &dyn SimView, source: ObjectSlot) -> Real {
    let bonus = compute_bonus(weapon, sim, source, 0);
    weapon.template.get_primary_damage_radius(&bonus)
}

/// `Weapon::isWithinAttackRange(source, pos)`.
pub fn is_within_attack_range_of_position(
    weapon: &Weapon,
    sim: &dyn SimView,
    source: ObjectSlot,
    pos: &Coord3D,
) -> Bool {
    let dist_sqr = sim.partition_distance_squared(source, None, pos, ATTACK_RANGE);
    let range_sqr = sqr(attack_range(weapon, sim, source));
    let min_sqr = sqr(weapon.template.get_minimum_attack_range());
    if dist_sqr < min_sqr {
        return FALSE;
    }
    Bool::from(dist_sqr <= range_sqr)
}

/// `Weapon::isWithinAttackRange(source, target)`.
pub fn is_within_attack_range(
    weapon: &Weapon,
    sim: &dyn SimView,
    source: ObjectSlot,
    target: ObjectSlot,
) -> Bool {
    let range_sqr = sqr(attack_range(weapon, sim, source));
    let dist_sqr = if !is_kind(sim, target, KindOfType::Bridge) {
        sim.partition_distance_squared(source, Some(target), &Coord3D::default(), ATTACK_RANGE)
    } else {
        bridge_distance_squared(range_sqr, sim, target, &|point| {
            sim.partition_distance_squared(source, None, point, ATTACK_RANGE)
        })
    };
    if dist_sqr < sqr(weapon.template.get_minimum_attack_range()) {
        return FALSE;
    }
    // A `NaN` distance is out of range.
    if !(dist_sqr <= range_sqr) {
        return FALSE;
    }
    // A contact weapon must really touch a structure.
    if weapon.template.is_contact_weapon() != FALSE && is_kind(sim, target, KindOfType::Structure) {
        let Some(me) = sim.object(source) else {
            return FALSE;
        };
        let hits = sim.find_potential_collisions(me.thing.get_position(), &me.geometry, 0.0, FALSE);
        return Bool::from(hits.contains(&target));
    }
    TRUE
}

/// `Weapon::isTooClose(source, target)`.
pub fn is_too_close(
    weapon: &Weapon,
    sim: &dyn SimView,
    source: ObjectSlot,
    target: ObjectSlot,
) -> Bool {
    let min = weapon.template.get_minimum_attack_range();
    if min == 0.0 {
        return FALSE;
    }
    let dist_sqr =
        sim.partition_distance_squared(source, Some(target), &Coord3D::default(), ATTACK_RANGE);
    Bool::from(dist_sqr < sqr(min))
}

/// `Weapon::isTooClose(source, pos)`.
pub fn is_too_close_to_position(
    weapon: &Weapon,
    sim: &dyn SimView,
    source: ObjectSlot,
    pos: &Coord3D,
) -> Bool {
    let min = weapon.template.get_minimum_attack_range();
    if min == 0.0 {
        return FALSE;
    }
    let dist_sqr = sim.partition_distance_squared(source, None, pos, ATTACK_RANGE);
    Bool::from(dist_sqr < sqr(min))
}

/// `Weapon::isGoalPosWithinAttackRange`: whether `source`, standing at `goal_pos`, would be
/// in range. The range is undersized and the minimum oversized by a quarter cell.
pub fn is_goal_pos_within_attack_range(
    weapon: &Weapon,
    sim: &dyn SimView,
    source: ObjectSlot,
    goal_pos: &Coord3D,
    target: Option<ObjectSlot>,
    target_pos: &Coord3D,
) -> Bool {
    let range_sqr = sqr(attack_range(weapon, sim, source) - (PATHFIND_CELL_SIZE_F * 0.25));
    let dist_sqr = match target {
        Some(t) if is_kind(sim, t, KindOfType::Bridge) => {
            bridge_distance_squared(range_sqr, sim, t, &|point| {
                goal_distance_squared(sim, source, goal_pos, None, point)
            })
        }
        _ => goal_distance_squared(sim, source, goal_pos, target, target_pos),
    };
    let min_sqr = sqr(weapon.template.get_minimum_attack_range() + (PATHFIND_CELL_SIZE_F * 0.25));
    if dist_sqr < min_sqr {
        return FALSE;
    }
    Bool::from(dist_sqr <= range_sqr)
}

/// `Weapon::isSourceObjectWithGoalPositionWithinAttackRange`: as above without the quarter
/// cell margins and without bridge points (used for garrison firing positions).
pub fn is_source_at_goal_within_attack_range(
    weapon: &Weapon,
    sim: &dyn SimView,
    source: ObjectSlot,
    goal_pos: &Coord3D,
    target: Option<ObjectSlot>,
    target_pos: &Coord3D,
) -> Bool {
    let dist_sqr = goal_distance_squared(sim, source, goal_pos, target, target_pos);
    let range_sqr = sqr(attack_range(weapon, sim, source));
    if dist_sqr < sqr(weapon.template.get_minimum_attack_range()) {
        return FALSE;
    }
    Bool::from(dist_sqr <= range_sqr)
}

fn clip_to_terrain_extent(sim: &dyn SimView, pos: &mut Coord3D) {
    let fallback = sim.terrain_extent();
    let bounds = ai_seams(sim).map_extent(fallback, false);
    if pos.x < bounds.lo.x + PATHFIND_CELL_SIZE_F {
        pos.x = bounds.lo.x + PATHFIND_CELL_SIZE_F;
    }
    if pos.y < bounds.lo.y + PATHFIND_CELL_SIZE_F {
        pos.y = bounds.lo.y + PATHFIND_CELL_SIZE_F;
    }
    if pos.x > bounds.hi.x - PATHFIND_CELL_SIZE_F {
        pos.x = bounds.hi.x - PATHFIND_CELL_SIZE_F;
    }
    if pos.y > bounds.hi.y - PATHFIND_CELL_SIZE_F {
        pos.y = bounds.hi.y - PATHFIND_CELL_SIZE_F;
    }
}

fn turn(dir: &mut Coord3D, angle_offset: Real) {
    if angle_offset != 0.0 {
        let angle = math::gamemath::atan2(dir.y, dir.x);
        dir.x = math::gamemath::cos(angle + angle_offset);
        dir.y = math::gamemath::sin(angle + angle_offset);
    }
}

/// `Weapon::computeApproachTarget`: where `source` should go to attack. The `Bool` is `TRUE`
/// when it is close enough already (the position is then its own).
///
/// The reference zeroes the position and answers `FALSE` with neither a target nor a
/// position; here `target_pos` is that of the target when one is given.
pub fn compute_approach_target(
    weapon: &Weapon,
    sim: &dyn SimView,
    source: ObjectSlot,
    target: Option<ObjectSlot>,
    pos: &Coord3D,
    angle_offset: Real,
) -> (Bool, Coord3D) {
    let (target_pos, mut dir) = match target {
        Some(t) => (position_of(sim, t), vector_to(sim, t, Some(source), pos)),
        None => {
            let mut dir = vector_to(sim, source, None, pos);
            // Flip the vector to get from source to pos.
            dir.x = -dir.x;
            dir.y = -dir.y;
            dir.z = -dir.z;
            (*pos, dir)
        }
    };
    let dist = dir.length();
    let min_range = weapon.template.get_minimum_attack_range();
    let mut approach = Coord3D::default();
    if min_range > PATHFIND_CELL_SIZE_F && dist < min_range {
        // Too close: move away. The direction is recomputed from the centres, which is zero
        // when the bounding circles touch.
        let src = position_of(sim, source);
        dir.x = src.x - target_pos.x;
        dir.y = src.y - target_pos.y;
        dir.z = 0.0;
        dir.normalize();

        // An airborne attacker heads for the opposite side instead of turning round.
        if sim.object_is_above_terrain(source) != FALSE {
            let angle = math::gamemath::atan2(-dir.y, -dir.x);
            let mut relative = sim.object_orientation(source).unwrap_or(0.0) - angle;
            if relative > 2.0 * PI {
                relative -= 2.0 * PI;
            }
            if relative < -2.0 * PI {
                relative += 2.0 * PI;
            }
            if gm_fabsf(relative) < PI / 2.0 {
                dir.x = -dir.x;
                dir.y = -dir.y;
                dir.z = -dir.z;
            }
        }
        turn(&mut dir, angle_offset);

        // Halfway between the minimum and maximum range.
        let mut range = (attack_range(weapon, sim, source) + min_range) / 2.0;
        if let Some(t) = target.and_then(|t| sim.object(t)) {
            range += t.geometry.get_bounding_circle_radius();
        }
        if let Some(s) = sim.object(source) {
            range += s.geometry.get_bounding_circle_radius();
        }
        approach.x = range * dir.x + target_pos.x;
        approach.y = range * dir.y + target_pos.y;
        approach.z = range * dir.z + target_pos.z;
        clip_to_terrain_extent(sim, &mut approach);
        return (FALSE, approach);
    }

    const FUDGE: Real = 0.001;
    if dist < FUDGE {
        return (TRUE, position_of(sim, source));
    }
    if weapon.template.is_contact_weapon() != FALSE {
        // A contact weapon approaches the object itself.
        return (FALSE, target_pos);
    }
    dir.x /= dist;
    dir.y /= dist;
    dir.z /= dist;
    turn(&mut dir, angle_offset);

    // A spot along the line between the two, within range.
    const APPROACH_FUDGE: Real = 0.9;
    let range = attack_range(weapon, sim, source) * APPROACH_FUDGE;
    approach.x = range * dir.x + target_pos.x;
    approach.y = range * dir.y + target_pos.y;
    approach.z = range * dir.z + target_pos.z;

    if let Some((id, true)) = sim.object(source).map(|o| (o.id, o.ai_slot.is_some())) {
        let mut seams = ai_seams(sim);
        if seams.is_aircraft_that_adjusts_destination(id) != FALSE {
            // Not stacked atop another aircraft.
            let target_id = target.and_then(|t| sim.object(t)).map(|o| o.id);
            approach = seams.adjust_target_destination(
                id,
                target_id,
                pos,
                &weapon.template.name,
                approach,
            );
        }
    }
    (FALSE, approach)
}

/// `Weapon::isWithinTargetPitch`.
pub fn is_within_target_pitch(
    weapon: &Weapon,
    sim: &dyn SimView,
    source: ObjectSlot,
    victim: ObjectSlot,
) -> Bool {
    if weapon.template.is_contact_weapon() != FALSE || weapon.is_pitch_limited() == FALSE {
        return TRUE;
    }
    let (Some(me), Some(them)) = (sim.object(source), sim.object(victim)) else {
        return TRUE;
    };
    let src = me.thing.get_position();
    let dst = them.thing.get_position();
    const ACCEPTABLE_DZ: Real = 10.0;
    if gm_fabsf(dst.z - src.z) < ACCEPTABLE_DZ {
        return TRUE;
    }
    let (mut min_pitch, mut max_pitch) = (0.0, 0.0);
    me.geometry
        .calc_pitches(src, &them.geometry, dst, &mut min_pitch, &mut max_pitch);
    let lo = weapon.template.min_target_pitch;
    let hi = weapon.template.max_target_pitch;
    Bool::from(
        (min_pitch >= lo && min_pitch <= hi)
            || (max_pitch >= lo && max_pitch <= hi)
            || (min_pitch <= lo && max_pitch >= hi),
    )
}

/// `Weapon::getFiringLineOfSightOrigin`: the top of the shooter, not the muzzle.
fn firing_origin(sim: &dyn SimView, source: ObjectSlot, base: Coord3D) -> Coord3D {
    let mut origin = base;
    if let Some(o) = sim.object(source) {
        origin.z += o.geometry.get_max_height_above_position();
    }
    origin
}

fn center_of(sim: &dyn SimView, slot: ObjectSlot) -> Coord3D {
    let mut center = Coord3D::default();
    if let Some(o) = sim.object(slot) {
        o.geometry
            .get_center_position(o.thing.get_position(), &mut center);
    }
    center
}

/// `Weapon::isClearFiringLineOfSightTerrain(source, victim)`.
pub fn is_clear_firing_los_terrain(
    sim: &dyn SimView,
    source: ObjectSlot,
    victim: ObjectSlot,
) -> Bool {
    let origin = firing_origin(sim, source, position_of(sim, source));
    sim.terrain()
        .is_clear_line_of_sight(&origin, &center_of(sim, victim))
}

/// `Weapon::isClearFiringLineOfSightTerrain(source, pos)`.
pub fn is_clear_firing_los_terrain_to_position(
    sim: &dyn SimView,
    source: ObjectSlot,
    victim_pos: &Coord3D,
) -> Bool {
    let origin = firing_origin(sim, source, position_of(sim, source));
    sim.terrain().is_clear_line_of_sight(&origin, victim_pos)
}

/// `Weapon::isClearGoalFiringLineOfSightTerrain(source, goalPos, victim)`.
pub fn is_clear_goal_firing_los_terrain(
    sim: &dyn SimView,
    source: ObjectSlot,
    goal_pos: &Coord3D,
    victim: ObjectSlot,
) -> Bool {
    let origin = firing_origin(sim, source, *goal_pos);
    sim.terrain()
        .is_clear_line_of_sight(&origin, &center_of(sim, victim))
}

/// `Weapon::isClearGoalFiringLineOfSightTerrain(source, goalPos, victimPos)`.
pub fn is_clear_goal_firing_los_terrain_to_position(
    sim: &dyn SimView,
    source: ObjectSlot,
    goal_pos: &Coord3D,
    victim_pos: &Coord3D,
) -> Bool {
    let origin = firing_origin(sim, source, *goal_pos);
    sim.terrain().is_clear_line_of_sight(&origin, victim_pos)
}

/// `Weapon::estimateWeaponDamage`. Without a victim the answer is the primary damage.
pub fn estimate_weapon_damage(
    weapon: &mut Weapon,
    sim: &dyn SimView,
    source: ObjectSlot,
    victim: Option<ObjectSlot>,
) -> Real {
    // Reloading is fine; out of ammo for good is not.
    if weapon.get_status(sim.frame()) == WeaponStatus::OutOfAmmo
        && weapon.template.reload_type != WeaponReloadType::AutoReload
    {
        return 0.0;
    }
    let bonus = compute_bonus(weapon, sim, source, 0);
    estimate_template_damage(&weapon.template, sim, source, victim, &bonus)
}

/// `WeaponTemplate::estimateWeaponTemplateDamage`.
pub fn estimate_template_damage(
    template: &WeaponTemplate,
    sim: &dyn SimView,
    source: ObjectSlot,
    victim: Option<ObjectSlot>,
    bonus: &WeaponBonus,
) -> Real {
    let Some(source_id) = sim.object(source).map(|o| o.id) else {
        return 0.0;
    };
    let amount = template.get_primary_damage(bonus);
    let Some(victim) = victim else {
        return amount;
    };
    let damage_type = template.damage_type;
    let death_type = template.death_type;

    if is_kind(sim, victim, KindOfType::Shrubbery) {
        // Nonzero so that flame weapons can always target shrubbery.
        return if death_type == DeathType::Burned {
            1.0
        } else {
            0.0
        };
    }

    // A sniper has nothing to hit in an empty structure.
    if is_kind(sim, victim, KindOfType::Structure) && damage_type == DamageType::Sniper {
        let contains = sim.object_has_contain(victim) != FALSE;
        let occupants = contains && sim.contain_count(victim) > 0;
        if cfg!(feature = "retail-crc") {
            if contains && !occupants {
                return 0.0;
            }
        } else {
            let slaves: Int = sim.spawn_slave_count(victim).unwrap_or(0);
            if !occupants && slaves <= 0 {
                return 0.0;
            }
        }
    }

    if (damage_type == DamageType::Surrender || template.allow_attack_garrisoned_buildings != FALSE)
        && sim.object_has_contain(victim) != FALSE
        && sim.contain_count(victim) > 0
        && sim.contain_is_garrisonable(victim) != FALSE
        && sim.contain_is_immune_to_clear(victim) == FALSE
    {
        return 1.0;
    }

    if damage_type == DamageType::Disarm {
        let mine = is_kind(sim, victim, KindOfType::Mine)
            || is_kind(sim, victim, KindOfType::BoobyTrap)
            || is_kind(sim, victim, KindOfType::Demotrap);
        return if mine { 1.0 } else { 0.0 };
    }
    if damage_type == DamageType::Deploy
        && sim
            .object(victim)
            .is_some_and(|o| o.status.test(ObjectStatusType::AirborneTarget.as_i32()) == FALSE)
    {
        return 1.0;
    }

    let mut info = DamageInfo::default();
    info.input.damage_type = damage_type;
    info.input.death_type = death_type;
    info.input.source_id = source_id;
    info.input.amount = amount;
    sim.estimate_object_damage(victim, &info)
}
