//! `PKT-301`: `computeAttackPath`, the answer of `doPathfind` to an attack path
//! request, with what `Pathfinder::findAttackPath` asks of the attacker's weapon
//! and `isAttackViewBlockedByObstacle` over the sim.
//!
//! Golden test: `crates/zh_logic/tests/golden_s6_attack_path.rs`.

use zh_common::ini::LOGICFRAMES_PER_SECOND;
use zh_common::math::Coord3D;
use zh_common::types::{Bool, Real, FALSE, TRUE};

use super::path_follow::{
    closest_path, compute_path, search_unit, searches_replaced, set_ignore_obstacle,
};
use super::{ai, is_doing_ground_movement, position_of, set_path, with_pathfinder};
use crate::arena::{ModuleContext, ObjectSlot};
use crate::object::weapon::Weapon;
use crate::object::weapon_queries;
use crate::pathfinder::ai_pathfind::{AttackProbe, AttackRequest, AttackVictim, TerrainSight};
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
use crate::pathfinder::{Path, Pathfinder};
use crate::rules::combat_types::LocomotorSurfaceType;
use crate::rules::kind_of::KindOfType;
use crate::sim::{SimApi, SimView};
use crate::spatial::partition_query::DistanceCalculationType;

const PATHFIND_CELL_SIZE_F: Real = 10.0;

/// `getCurrentWeapon`.
fn current_weapon(sim: &dyn SimView, owner: ObjectSlot) -> Option<&Weapon> {
    let object = sim.object(owner)?;
    let slot = object.weapon_set.get_cur_weapon_slot();
    object.weapon_set.get_weapon_in_weapon_slot(slot)
}

/// `Pathfinder::isAttackViewBlockedByObstacle`: whether `attacker`, standing at
/// `attacker_pos`, cannot see the victim or the position it attacks.
pub fn is_attack_view_blocked_by_obstacle(
    sim: &dyn SimView,
    pathfinder: &Pathfinder,
    attacker: ObjectSlot,
    attacker_pos: &Coord3D,
    victim: Option<ObjectSlot>,
    victim_pos: &Coord3D,
) -> bool {
    let uses_line_of_sight = sim.rules().ai_data.attack_uses_line_of_sight != FALSE;
    let Some(party) = sim.view_party(attacker) else {
        return false;
    };
    if !uses_line_of_sight || !party.needs_line_of_sight {
        return false;
    }
    // Terrain does not count for a unit that cannot move round it.
    let weapon = if sim.object_is_kind_of(attacker, KindOfType::Immobile) != FALSE {
        None
    } else {
        current_weapon(sim, attacker)
    };
    let sight = match weapon {
        None => TerrainSight::NoWeapon,
        Some(weapon) => {
            let clear = match victim {
                Some(victim) => weapon_queries::is_clear_goal_firing_los_terrain(
                    sim,
                    attacker,
                    attacker_pos,
                    victim,
                ),
                None => weapon_queries::is_clear_goal_firing_los_terrain_to_position(
                    sim,
                    attacker,
                    attacker_pos,
                    victim_pos,
                ),
            };
            let terrain_blocked = clear == FALSE;
            if weapon.template.is_contact_weapon() != FALSE {
                TerrainSight::Contact { terrain_blocked }
            } else {
                TerrainSight::Weapon { terrain_blocked }
            }
        }
    };
    pathfinder.is_attack_view_blocked(
        uses_line_of_sight,
        &party,
        attacker_pos,
        victim.and_then(|v| sim.view_party(v)).as_ref(),
        victim_pos,
        sight,
    )
}

/// The attacker's weapon as `findAttackPath` asks about it.
struct Probe<'a> {
    sim: &'a dyn SimApi,
    weapon: &'a Weapon,
    attacker: ObjectSlot,
    victim: Option<ObjectSlot>,
    victim_pos: Coord3D,
}

impl AttackProbe for Probe<'_> {
    fn attack_distance(&mut self) -> Real {
        weapon_queries::attack_distance(self.weapon, self.sim, self.attacker, self.victim)
    }

    fn in_range(&mut self, position: &Coord3D) -> bool {
        weapon_queries::is_goal_pos_within_attack_range(
            self.weapon,
            self.sim,
            self.attacker,
            position,
            self.victim,
            &self.victim_pos,
        ) != FALSE
    }

    fn view_blocked(&mut self, pathfinder: &Pathfinder, position: &Coord3D) -> bool {
        is_attack_view_blocked_by_obstacle(
            self.sim,
            pathfinder,
            self.attacker,
            position,
            self.victim,
            &self.victim_pos,
        )
    }
}

/// `Pathfinder::findAttackPath` from where the unit stands.
fn find_attack_path(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    victim: Option<ObjectSlot>,
    victim_pos: &Coord3D,
) -> Option<Path> {
    // The reference whose searches were replaced never got here.
    if searches_replaced(sim) {
        return None;
    }
    let owner = ctx.owner_slot;
    let unit = search_unit(sim, ctx)?;
    let position = position_of(sim, owner);
    with_pathfinder(sim, |sim, pathfinder| {
        let sim: &dyn SimApi = sim;
        let mut probe = Probe {
            sim,
            weapon: current_weapon(sim, owner)?,
            attacker: owner,
            victim,
            victim_pos: *victim_pos,
        };
        let request = AttackRequest {
            unit: &unit,
            position,
            from: position,
            victim: victim.map(|v| AttackVictim {
                position: position_of(sim, v),
                significantly_above_terrain: sim.object_is_significantly_above_terrain(v) != FALSE,
            }),
            victim_position: *victim_pos,
        };
        pathfinder.find_attack_path(sim.terrain(), sim.path_units(), &mut probe, &request)
    })
    .flatten()
}

/// Whether the weapon reaches from where the unit stands and nothing is in the
/// way.
fn can_shoot_from_here(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    victim: Option<ObjectSlot>,
    victim_pos: &Coord3D,
) -> bool {
    let owner = ctx.owner_slot;
    let Some(weapon) = current_weapon(sim, owner) else {
        return false;
    };
    let in_range = match victim {
        Some(victim) => weapon_queries::is_within_attack_range(weapon, sim, owner, victim),
        None => weapon_queries::is_within_attack_range_of_position(weapon, sim, owner, victim_pos),
    };
    if in_range == FALSE {
        return false;
    }
    if is_doing_ground_movement(sim, ctx) == FALSE
        || victim.is_some_and(|v| sim.object_is_significantly_above_terrain(v) != FALSE)
    {
        return true;
    }
    let from = position_of(sim, owner);
    let target = victim.map_or(*victim_pos, |v| position_of(sim, v));
    !sim.pathfinder()
        .is_some_and(|p| is_attack_view_blocked_by_obstacle(sim, p, owner, &from, victim, &target))
}

/// The approach of a contact weapon: a plain path to the target itself.
fn contact_path(sim: &mut dyn SimApi, ctx: ModuleContext) -> bool {
    let owner = ctx.owner_slot;
    let mut to = ai(sim, ctx).requested_destination;
    ai(sim, ctx).destroy_path();
    if let Some(locomotor) = ai(sim, ctx).cur_locomotor_mut() {
        locomotor.set_no_slow_down(TRUE);
    }
    let ok = compute_path(sim, ctx, &mut to);
    let victim_pos = ai(sim, ctx).requested_destination;
    let Some(end) = path_end(sim, ctx) else {
        return false;
    };
    let dx = victim_pos.x - end.x;
    let dy = victim_pos.y - end.y;
    let near = PATHFIND_CELL_SIZE_F * 3.0;
    if dx * dx + dy * dy < near * near {
        if let Some(mut path) = ai(sim, ctx).path.take() {
            path.update_last_node(sim.terrain(), &victim_pos);
            ai(sim, ctx).path = Some(path);
        }
    }
    let end = path_end(sim, ctx).unwrap_or(end);
    let from = position_of(sim, owner);
    let dx = from.x - end.x;
    let dy = from.y - end.y;
    if dx * dx + dy * dy < PATHFIND_CELL_SIZE_F * PATHFIND_CELL_SIZE_F {
        // Too short: the goal cannot be reached.
        ai(sim, ctx).destroy_path();
        return false;
    }
    ok
}

/// The position of the last node of the unit's path.
fn path_end(sim: &mut dyn SimApi, ctx: ModuleContext) -> Option<Coord3D> {
    let path = ai(sim, ctx).path.as_ref()?;
    path.last_node().map(|node| *path.node(node).position())
}

/// `computeAttackPath` against `victim`, or against the requested destination.
/// `true` with no path means the unit can shoot from where it stands.
pub(super) fn compute_attack_path(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    victim: Option<ObjectSlot>,
) -> bool {
    let owner = ctx.owner_slot;
    let now = sim.frame();
    if crate::ai::seams::ai_seams(sim).replaces_attack_path() != FALSE {
        let state = ai(sim, ctx);
        let (wanted, to) = (state.requested_victim_id, state.requested_destination);
        let wanted = victim.map(|_| wanted);
        let answer = crate::ai::seams::ai_seams(sim).attack_path(ctx.owner_id, wanted, &to);
        return match answer {
            Some(path) => {
                set_path(sim, ctx, path);
                true
            }
            None => false,
        };
    }
    // Too soon since the last path.
    let state = ai(sim, ctx);
    if state.path_timestamp >= now.wrapping_sub(2)
        && state.path.is_some()
        && state.is_blocked_and_stuck != FALSE
    {
        state.ignore_collisions_until = now.wrapping_add(2 * LOGICFRAMES_PER_SECOND as u32);
        state.blocked_frames = 0;
        state.is_blocked = FALSE;
        state.is_blocked_and_stuck = FALSE;
        return true;
    }
    let air = LocomotorSurfaceType::Air.as_i32();
    let land_bound = state.locomotor_set.get_valid_surfaces() & air == 0;
    let victim_pos = state.requested_destination;
    let victim_layer = victim
        .and_then(|v| sim.object(v))
        .map_or(PathfindLayerEnum::Ground, |o| o.layer);
    if current_weapon(sim, owner).is_none() {
        return false;
    }
    if can_shoot_from_here(sim, ctx, victim, &victim_pos) {
        ai(sim, ctx).destroy_path();
        return true;
    }
    if current_weapon(sim, owner).is_some_and(|w| w.template.is_contact_weapon() != FALSE) {
        return contact_path(sim, ctx);
    }

    let mut aim = match victim {
        Some(v) if sim.object_is_kind_of(v, KindOfType::Bridge) != FALSE => {
            let position = position_of(sim, v);
            let id = sim.object(v).map_or_else(Default::default, |o| o.id);
            let [first, second] =
                crate::ai::seams::ai_seams(sim).bridge_attack_points(id, &position);
            let distance = |point: &Coord3D| {
                sim.partition_distance_squared(
                    owner,
                    None,
                    point,
                    DistanceCalculationType::FromBoundingSphere3d,
                )
            };
            if distance(&second) < distance(&first) {
                second
            } else {
                first
            }
        }
        Some(v) => position_of(sim, v),
        None => victim_pos,
    };
    aim.z = sim.get_layer_height(aim.x, aim.y, victim_layer);

    if sim.object_is_above_terrain(owner) != FALSE && !land_bound {
        // Flying: a straight line to where the weapon wants the unit.
        let Some(weapon) = current_weapon(sim, owner) else {
            return false;
        };
        let (_, aim) =
            weapon_queries::compute_approach_target(weapon, sim, owner, victim, &aim, 0.0);
        if let Some(path) = ai(sim, ctx).path.as_ref() {
            if let Some(close) = path
                .first_node()
                .and_then(|first| path.node(first).next_optimized())
            {
                let node = path.node(close);
                let dx = aim.x - node.position().x;
                let dy = aim.y - node.position().y;
                if node.next_optimized().is_none() && dx * dx + dy * dy < 0.25 {
                    return true;
                }
            }
        }
        ai(sim, ctx).destroy_path();
        let mut path = Path::new();
        path.prepend_node(&aim, PathfindLayerEnum::Ground);
        let mut from = position_of(sim, owner);
        from.z = aim.z;
        path.prepend_node(&from, PathfindLayerEnum::Ground);
        if let Some(first) = path.first_node() {
            let next = path.node(first).next();
            path.set_next_optimized(first, next);
        }
        set_path(sim, ctx, Some(path));
    } else {
        ai(sim, ctx).destroy_path();
        let ignored = ai(sim, ctx).ignore_obstacle_id;
        set_ignore_obstacle(sim, ignored);
        let path = find_attack_path(sim, ctx, victim, &aim);
        set_path(sim, ctx, path);
        if let Some(mut goal) = path_end(sim, ctx) {
            let reaches = current_weapon(sim, owner).is_some_and(|w| {
                weapon_queries::is_goal_pos_within_attack_range(w, sim, owner, &goal, victim, &aim)
                    == TRUE
            });
            if !reaches {
                // No spot to attack from. A short move unstacks the unit.
                let mut position = position_of(sim, owner);
                goal.x -= position.x;
                goal.y -= position.y;
                goal.z -= position.z;
                if goal.length() < 3.0 * PATHFIND_CELL_SIZE_F {
                    ai(sim, ctx).destroy_path();
                    sim.pathfinder_adjust_destination(owner, &mut position);
                    let path = closest_path(sim, ctx, &mut position, Bool::from(false), 0.2);
                    set_path(sim, ctx, path);
                }
                // The ignored obstacle stays set on the map.
                if ai(sim, ctx).path.is_none() {
                    return false;
                }
            }
            if let Some(goal) = path_end(sim, ctx) {
                let layer = sim.terrain().get_layer_for_destination(&goal);
                sim.pathfinder_update_goal(owner, &goal, layer);
            }
            let blocked_by_ally = ai(sim, ctx)
                .path
                .as_ref()
                .is_some_and(|p| p.blocked_by_ally() != FALSE);
            if blocked_by_ally && sim.object_is_kind_of(owner, KindOfType::NoCollide) == FALSE {
                super::move_allies(sim, ctx);
            }
        }
        set_ignore_obstacle(sim, Default::default());
    }

    let state = ai(sim, ctx);
    state.path_timestamp = now;
    state.blocked_frames = 0;
    state.is_blocked_and_stuck = FALSE;
    state.path.is_some()
}
