//! `PKT-301`, path following (AI plan slice S9): the path requests, `doPathfind`,
//! `computePath`, `computeQuickPath`, and the drive along a path.
//!
//! The module owns its path (`AiUpdate::path`) and the pathfind map belongs to
//! the sim. Without a map the S8 seams still answer.
//!
//! The closest, patch, safe, move-away and attack searches are not ported (S6);
//! they are asked through [`crate::ai::seams::AiSeams`] from this module only,
//! in the functions at the end of the file.

use super::{ai, is_doing_ground_movement, is_projectile, layer_of, position_of};
use crate::ai::ai_state_machine::state_id;
use crate::ai::seams::ai_seams;
use crate::arena::{ModuleContext, ObjectSlot};
use crate::combat::locomotor::LocomotorAppearance;
use crate::modules::update::ai_update::{LocoGoalType, FAST_AS_POSSIBLE};
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
use crate::pathfinder::{ClosestPointOnPathInfo, Path, PathQuery, Pathfinder, SearchUnit};
use crate::rules::combat_types::LocomotorSurfaceType;
use crate::rules::kind_of::KindOfType;
use crate::sim::SimApi;
use zh_common::ini::LOGICFRAMES_PER_SECOND;
use zh_common::math::{gm_sqrtf, Coord3D, Region3D};
use zh_common::types::{Bool, ObjectId, Real, UnsignedInt, FALSE, TRUE};

const PATHFIND_CELL_SIZE_F: Real = 10.0;
const PATHFIND_CLOSE_ENOUGH: Real = 1.0;
const SECOND: UnsignedInt = LOGICFRAMES_PER_SECOND as UnsignedInt;

// -----------------------------------------------------------------------------
// The map
// -----------------------------------------------------------------------------

/// Runs `act` with the sim's pathfinder in hand; `None` without a map. While
/// `act` runs the sim has no pathfinder, so it must not give orders (see
/// `move_allies`, which hands the map back for each one).
pub(super) fn with_pathfinder<R>(
    sim: &mut dyn SimApi,
    act: impl FnOnce(&mut dyn SimApi, &mut Pathfinder) -> R,
) -> Option<R> {
    let mut pathfinder = sim.take_pathfinder()?;
    let result = act(sim, &mut pathfinder);
    sim.put_pathfinder(pathfinder);
    Some(result)
}

/// `Pathfinder::updateGoal` of the unit: the sim's map, else the seam.
pub(crate) fn update_goal(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    pos: &Coord3D,
    layer: PathfindLayerEnum,
) {
    if sim.pathfinder().is_some() {
        sim.pathfinder_update_goal(ctx.owner_slot, pos, layer);
    } else {
        ai_seams(sim).update_goal(ctx.owner_id, pos, layer);
    }
}

/// `Pathfinder::goalPosition`: where the unit's goal cell has it stand.
pub(crate) fn goal_position(sim: &mut dyn SimApi, ctx: ModuleContext) -> Option<Coord3D> {
    if sim.pathfinder().is_none() {
        return ai_seams(sim).goal_position(ctx.owner_id);
    }
    let cell = ai(sim, ctx).pathfind_goal_cell;
    let agent = sim.ai_path_view(ctx.owner_slot)?.agent;
    let mut goal = Coord3D::default();
    sim.pathfinder()?
        .goal_position(sim.terrain(), &agent, cell, &mut goal)
        .then_some(goal)
}

/// `Pathfinder::snapPosition`.
pub(super) fn snap_position(sim: &mut dyn SimApi, ctx: ModuleContext, pos: Coord3D) -> Coord3D {
    if sim.pathfinder().is_none() {
        return ai_seams(sim).snap_position(ctx.owner_id, pos);
    }
    let mut snapped = pos;
    sim.pathfinder_snap_position(ctx.owner_slot, &mut snapped);
    snapped
}

/// `clientSafeQuickDoesPathExist` from where the unit stands.
pub(super) fn quick_path_exists(sim: &mut dyn SimApi, ctx: ModuleContext, to: &Coord3D) -> bool {
    if sim.pathfinder().is_none() {
        return ai_seams(sim).quick_path_exists(ctx.owner_id, to) != FALSE;
    }
    let from = position_of(sim, ctx.owner_slot);
    let surfaces = ai(sim, ctx).locomotor_set.get_valid_surfaces();
    sim.pathfinder()
        .is_some_and(|p| p.quick_path_exists(sim.terrain(), surfaces, &from, to))
}

/// What the searches read from the unit.
fn search_unit(sim: &mut dyn SimApi, ctx: ModuleContext) -> Option<SearchUnit> {
    let owner = ctx.owner_slot;
    let view = sim.ai_path_view(owner)?;
    let dozer = sim.object_is_kind_of(owner, KindOfType::Dozer) != FALSE;
    let vehicle = sim.object_is_kind_of(owner, KindOfType::Vehicle) != FALSE;
    let state = ai(sim, ctx);
    Some(SearchUnit {
        agent: view.agent,
        surfaces: state.locomotor_set.get_valid_surfaces(),
        downhill_only: state.locomotor_set.is_downhill_only() != FALSE,
        human: view.human,
        can_path_through_units: state.can_path_through_units != FALSE,
        dozer,
        vehicle,
    })
}

/// `getCrusherLevel() > 0`.
fn is_crusher(sim: &dyn SimApi, owner: ObjectSlot) -> bool {
    sim.object_crusher_level(owner) > 0
}

/// `getMaximumPathfindExtent`. A terrain without a map answers with the
/// world's extent.
fn maximum_pathfind_extent(sim: &dyn SimApi) -> Region3D {
    sim.terrain()
        .terrain_logic()
        .map_or_else(|| sim.terrain_extent(), |t| t.maximum_pathfind_extent())
}

/// `Region3D::containsWithin` of the 2D point.
fn contains_2d(region: &Region3D, pos: &Coord3D) -> bool {
    region.lo.x < pos.x && pos.x < region.hi.x && region.lo.y < pos.y && pos.y < region.hi.y
}

/// `Pathfinder::updateGoal` at the end of the unit's own path.
fn update_goal_at_path_end(sim: &mut dyn SimApi, ctx: ModuleContext) {
    let Some((pos, layer)) = ai(sim, ctx).path.as_ref().and_then(|path| {
        path.last_node()
            .map(|n| (*path.node(n).position(), path.node(n).layer()))
    }) else {
        return;
    };
    sim.pathfinder_update_goal(ctx.owner_slot, &pos, layer);
}

/// `computePointOnPath` of the unit's own path. `None` without a path or map.
pub(super) fn point_on_path(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
) -> Option<ClosestPointOnPathInfo> {
    let owner = ctx.owner_slot;
    sim.pathfinder()?;
    let pos = position_of(sim, owner);
    let surfaces = ai(sim, ctx).locomotor_set.get_valid_surfaces();
    let mut path = ai(sim, ctx).path.take()?;
    let agent = sim.ai_path_view(owner).map(|v| v.agent);
    let info = sim.pathfinder().map(|pathfinder| {
        path.compute_point_on_path(
            &PathQuery {
                pathfinder,
                units: sim.path_units(),
                agent: agent.as_ref(),
            },
            surfaces,
            &pos,
        )
    });
    ai(sim, ctx).path = Some(path);
    info
}

fn flight_dist_to_goal(sim: &mut dyn SimApi, ctx: ModuleContext) -> (Real, Coord3D) {
    let pos = position_of(sim, ctx.owner_slot);
    ai(sim, ctx)
        .path
        .as_mut()
        .map_or((0.0, pos), |p| p.compute_flight_dist_to_goal(&pos))
}

// -----------------------------------------------------------------------------
// Requests
// -----------------------------------------------------------------------------

/// What every `request*Path` starts from.
fn prepare_request(sim: &mut dyn SimApi, ctx: ModuleContext) {
    let state = ai(sim, ctx);
    state.is_attack_path = FALSE;
    state.requested_victim_id = ObjectId::INVALID;
    state.is_approach_path = FALSE;
    state.is_safe_path = FALSE;
}

/// `m_pathTimestamp > frame - 3`: a path was made in the last two frames. In
/// the first three frames the difference wraps, so nothing is recent.
fn pathed_recently(sim: &mut dyn SimApi, ctx: ModuleContext) -> bool {
    let now = sim.frame();
    ai(sim, ctx).path_timestamp > now.wrapping_sub(3)
}

/// `queueForPath`, or later when a path was made just now.
fn queue_or_delay(sim: &mut dyn SimApi, ctx: ModuleContext, delay: UnsignedInt) -> bool {
    if pathed_recently(sim, ctx) {
        super::set_queue_for_path_time(sim, ctx, delay);
        return false;
    }
    let _ = sim.queue_for_path(ctx.owner_id);
    true
}

/// `requestPath`.
pub fn request_path(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    destination: &Coord3D,
    is_final_goal: Bool,
) {
    let destination = *destination;
    prepare_request(sim, ctx);
    let state = ai(sim, ctx);
    state.requested_destination = destination;
    state.is_final_goal = is_final_goal;
    if can_compute_quick_path(sim, ctx) {
        compute_quick_path(sim, ctx, &destination);
        return;
    }
    ai(sim, ctx).waiting_for_path = TRUE;
    if pathed_recently(sim, ctx) {
        super::set_queue_for_path_time(sim, ctx, SECOND);
        let now = sim.frame();
        let state = ai(sim, ctx);
        if state.path.is_some() && state.is_blocked_and_stuck != FALSE {
            state.ignore_collisions_until = now.wrapping_add(2 * SECOND);
            state.blocked_frames = 0;
            state.is_blocked = FALSE;
            state.is_blocked_and_stuck = FALSE;
        }
        return;
    }
    let _ = sim.queue_for_path(ctx.owner_id);
}

/// `requestAttackPath`.
pub fn request_attack_path(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    victim: ObjectId,
    victim_pos: &Coord3D,
) {
    let victim_pos = *victim_pos;
    let state = ai(sim, ctx);
    state.requested_destination = victim_pos;
    state.requested_victim_id = victim;
    state.is_attack_path = TRUE;
    state.is_approach_path = FALSE;
    state.is_safe_path = FALSE;
    state.waiting_for_path = TRUE;
    if !queue_or_delay(sim, ctx, 2 * SECOND) {
        super::set_locomotor_goal_none(sim, ctx);
    }
}

/// `requestApproachPath`.
pub fn request_approach_path(sim: &mut dyn SimApi, ctx: ModuleContext, destination: &Coord3D) {
    let destination = *destination;
    prepare_request(sim, ctx);
    let state = ai(sim, ctx);
    state.requested_destination = destination;
    state.is_final_goal = TRUE;
    state.is_approach_path = TRUE;
    state.waiting_for_path = TRUE;
    queue_or_delay(sim, ctx, 2 * SECOND);
}

/// `requestSafePath`.
pub fn request_safe_path(sim: &mut dyn SimApi, ctx: ModuleContext, repulsor: ObjectId) {
    prepare_request(sim, ctx);
    let state = ai(sim, ctx);
    if repulsor != state.repulsor1 {
        state.repulsor2 = state.repulsor1;
    }
    state.repulsor1 = repulsor;
    state.is_final_goal = FALSE;
    state.is_safe_path = TRUE;
    state.waiting_for_path = TRUE;
    queue_or_delay(sim, ctx, 2 * SECOND);
}

// -----------------------------------------------------------------------------
// Serving the request
// -----------------------------------------------------------------------------

/// `doPathfind` on the sim's map. `waiting_for_path` is already cleared.
pub(super) fn do_pathfind_on_map(sim: &mut dyn SimApi, ctx: ModuleContext) {
    let owner = ctx.owner_slot;
    if ai(sim, ctx).is_safe_path != FALSE {
        ai(sim, ctx).destroy_path();
        let (repulsor1, repulsor2) = {
            let state = ai(sim, ctx);
            (state.repulsor1, state.repulsor2)
        };
        let mut first = Coord3D::new(-1000.0, -1000.0, 0.0);
        if let Some(slot) = sim.find_object(repulsor1) {
            first = position_of(sim, slot);
        }
        let mut second = first;
        if let Some(slot) = sim.find_object(repulsor2) {
            second = position_of(sim, slot);
        }
        let radius = sim.object_vision_range(owner) + sim.rules().ai_data.repulsed_distance;
        let path = safe_path(sim, ctx, &first, &second, radius);
        set_path(sim, ctx, path);
        return;
    }
    // The reference's `&` evaluates both sides.
    let ground = is_doing_ground_movement(sim, ctx) != FALSE;
    if ai(sim, ctx).is_approach_path != FALSE && !ground {
        ai(sim, ctx).is_approach_path = FALSE;
    }
    if ai(sim, ctx).is_approach_path != FALSE {
        ai(sim, ctx).destroy_path();
        let mut to = ai(sim, ctx).requested_destination;
        let blocked = ai(sim, ctx).is_blocked_and_stuck;
        let path = closest_path(sim, ctx, &mut to, blocked, 0.2);
        ai(sim, ctx).requested_destination = to;
        set_path(sim, ctx, path);
        if is_doing_ground_movement(sim, ctx) != FALSE {
            update_goal_at_path_end(sim, ctx);
        }
        return;
    }
    if ai(sim, ctx).is_attack_path != FALSE {
        let wanted = ai(sim, ctx).requested_victim_id;
        let victim = sim.find_object(wanted);
        let victim_id = victim.map(|_| wanted);
        let to = ai(sim, ctx).requested_destination;
        if let Some(path) = attack_path(sim, ctx, victim_id, &to) {
            set_path(sim, ctx, path);
            update_goal_at_path_end(sim, ctx);
            ai(sim, ctx).is_attack_path = TRUE;
            return;
        }
        ai(sim, ctx).is_attack_path = FALSE;
        if let Some(victim) = victim {
            let mut to = position_of(sim, victim);
            sim.pathfinder_adjust_to_possible_destination(owner, &mut to);
            let state = ai(sim, ctx);
            state.requested_destination = to;
            state.ignore_obstacle_id = wanted;
        }
    }
    let mut to = ai(sim, ctx).requested_destination;
    compute_path(sim, ctx, &mut to);
    ai(sim, ctx).requested_destination = to;
    if ai(sim, ctx).is_final_goal != FALSE && is_doing_ground_movement(sim, ctx) != FALSE {
        update_goal_at_path_end(sim, ctx);
    }
    if ai(sim, ctx).queue_for_path_frame > sim.frame() {
        ai(sim, ctx).waiting_for_path = TRUE;
    }
    if ai(sim, ctx).waiting_for_path == FALSE {
        super::wake_up_now(sim, ctx);
    }
}

/// `m_path = path`.
pub(super) fn set_path(sim: &mut dyn SimApi, ctx: ModuleContext, path: Option<Path>) {
    let state = ai(sim, ctx);
    state.has_path = Bool::from(path.is_some());
    state.path = path;
}

/// `canComputeQuickPath`: only a unit flying through the air takes one.
pub fn can_compute_quick_path(sim: &mut dyn SimApi, ctx: ModuleContext) -> bool {
    let air = LocomotorSurfaceType::Air.as_i32();
    if ai(sim, ctx).locomotor_set.get_valid_surfaces() & air == 0 {
        return false;
    }
    is_doing_ground_movement(sim, ctx) == FALSE
}

/// `computeQuickPath`: a straight line, or around the tall buildings.
pub fn compute_quick_path(sim: &mut dyn SimApi, ctx: ModuleContext, destination: &Coord3D) -> bool {
    let owner = ctx.owner_slot;
    if let Some(path) = ai(sim, ctx).path.as_ref() {
        if let Some(close) = path.last_node() {
            let node = path.node(close);
            if node.next_optimized().is_none() {
                let mut dx = destination.x - node.position().x;
                dx *= dx;
                let mut dy = destination.y - node.position().y;
                dy *= dy;
                let mut dz = destination.z - node.position().z;
                dz *= dz;
                if dx + dy + dz < 0.25 {
                    return true;
                }
            }
        }
    }
    ai(sim, ctx).destroy_path();
    let path = if sim.object_is_kind_of(owner, KindOfType::Aircraft) != FALSE
        && !is_projectile(sim, owner)
    {
        // `getAircraftPath`. Only the jet names a building not to go around.
        let winged = ai(sim, ctx)
            .cur_locomotor()
            .is_some_and(|l| l.template.appearance == LocomotorAppearance::Wings);
        let from = position_of(sim, owner);
        sim.pathfinder().map(|p| {
            p.aircraft_path(
                sim.tall_buildings(),
                &from,
                destination,
                winged,
                ObjectId::INVALID,
            )
        })
    } else {
        let mut path = Path::new();
        path.prepend_node(destination, PathfindLayerEnum::Ground);
        let mut pos = position_of(sim, owner);
        pos.z = destination.z;
        path.prepend_node(&pos, layer_of(sim, owner));
        if let Some(first) = path.first_node() {
            let next = path.node(first).next();
            path.set_next_optimized(first, next);
        }
        Some(path)
    };
    set_path(sim, ctx, path);
    let now = sim.frame();
    let state = ai(sim, ctx);
    state.path_timestamp = now;
    state.blocked_frames = 0;
    state.is_blocked_and_stuck = FALSE;
    true
}

/// `computePath`: a path to `destination`, which the closest search may move.
pub fn compute_path(sim: &mut dyn SimApi, ctx: ModuleContext, destination: &mut Coord3D) -> bool {
    let owner = ctx.owner_slot;
    if ai(sim, ctx).is_blocked_and_stuck == FALSE {
        ai(sim, ctx).destroy_path();
    }
    if can_compute_quick_path(sim, ctx) {
        return compute_quick_path(sim, ctx, destination);
    }
    ai(sim, ctx).retry_path = FALSE;
    let extent = maximum_pathfind_extent(sim);
    if !contains_2d(&extent, destination) && !contains_2d(&extent, &position_of(sim, owner)) {
        return compute_quick_path(sim, ctx, destination);
    }
    if ai(sim, ctx).machine.current_state_id() == state_id::FOLLOW_EXITPRODUCTION_PATH
        && ai(sim, ctx).can_path_through_units != FALSE
    {
        let ok = compute_quick_path(sim, ctx, destination);
        if ok {
            super::move_allies_away_from_destination(sim, ctx, destination);
            ai(sim, ctx).set_can_path_through_units(FALSE);
            let to = *destination;
            super::set_goal_position_clipped(
                sim,
                ctx,
                &to,
                crate::modules::module::CommandSourceType::FromAi,
            );
            return ok;
        }
    }
    let ignored = ai(sim, ctx).ignore_obstacle_id;
    set_ignore_obstacle(sim, ignored);
    let original = *destination;
    let surfaces = ai(sim, ctx).locomotor_set.get_valid_surfaces();
    if ai(sim, ctx).is_final_goal == FALSE {
        let from = position_of(sim, owner);
        let layer = layer_of(sim, owner);
        let agent = sim.ai_path_view(owner).map(|v| v.agent);
        let passable = sim.pathfinder().is_some_and(|p| {
            p.is_line_passable(
                sim.path_units(),
                agent.as_ref(),
                surfaces,
                layer,
                &from,
                &original,
                false,
                true,
            )
        });
        // The ignored obstacle stays set on the map.
        if passable {
            return compute_quick_path(sim, ctx, destination);
        }
    }
    let layer = sim.terrain().get_layer_for_destination(destination);
    let crusher = is_crusher(sim, owner);
    let valid = sim
        .pathfinder()
        .is_some_and(|p| p.valid_movement_position(crusher, layer, surfaces, destination));
    let mut new_path = None;
    if valid {
        let stuck = ai(sim, ctx).is_blocked_and_stuck;
        new_path = if stuck != FALSE {
            patch_path(sim, ctx, stuck)
        } else {
            find_path(sim, ctx, destination)
        };
    }
    if new_path.is_none() && ai(sim, ctx).path.is_none() {
        let blocked = ai(sim, ctx).is_blocked_and_stuck;
        new_path = closest_path(sim, ctx, destination, blocked, 0.0);
        ai(sim, ctx).retry_path = TRUE;
    }
    set_ignore_obstacle(sim, ObjectId::INVALID);
    if let Some(new_path) = new_path {
        ai(sim, ctx).destroy_path();
        set_path(sim, ctx, Some(new_path));
        if ai(sim, ctx)
            .cur_locomotor()
            .is_some_and(|l| l.is_ultra_accurate() != FALSE)
        {
            if let Some(mut path) = ai(sim, ctx).path.take() {
                path.update_last_node(sim.terrain(), &original);
                ai(sim, ctx).path = Some(path);
            }
        }
        ai(sim, ctx).set_goal_position_on_path();
        if sim.object_is_kind_of(owner, KindOfType::NoCollide) == FALSE {
            super::move_allies(sim, ctx);
        }
    } else if ai(sim, ctx).path.is_some() && ai(sim, ctx).is_blocked_and_stuck != FALSE {
        // Stop and ask again later.
        ai(sim, ctx).destroy_path();
        super::set_queue_for_path_time(sim, ctx, SECOND);
        let mut goal = position_of(sim, owner);
        sim.pathfinder_snap_position(owner, &mut goal);
        let state = ai(sim, ctx);
        state.set_final_position(&goal);
        state.locomotor_goal_type = LocoGoalType::None;
        state.blocked_frames = 0;
        state.is_blocked = FALSE;
        state.is_blocked_and_stuck = FALSE;
    }
    let now = sim.frame();
    let state = ai(sim, ctx);
    state.path_timestamp = now;
    state.blocked_frames = 0;
    state.is_blocked_and_stuck = FALSE;
    state.path.is_some()
}

/// `Pathfinder::setIgnoreObstacleID`.
fn set_ignore_obstacle(sim: &mut dyn SimApi, id: ObjectId) {
    with_pathfinder(sim, |_, pathfinder| pathfinder.ignore_obstacle_id = id);
}

/// `Pathfinder::findPath` from where the unit stands.
fn find_path(sim: &mut dyn SimApi, ctx: ModuleContext, to: &Coord3D) -> Option<Path> {
    let unit = search_unit(sim, ctx)?;
    let from = position_of(sim, ctx.owner_slot);
    with_pathfinder(sim, |sim, pathfinder| {
        pathfinder.find_path(sim.terrain(), sim.path_units(), &unit, &from, to)
    })
    .flatten()
}

// -----------------------------------------------------------------------------
// The drive
// -----------------------------------------------------------------------------

/// The speed along the path, slowing for what blocks the way.
fn path_speed(sim: &mut dyn SimApi, ctx: ModuleContext, blocked: &mut Bool) -> Real {
    let mut speed = super::goal_speed(sim, ctx);
    let state = ai(sim, ctx);
    if *blocked != FALSE && speed > state.cur_max_blocked_speed {
        speed = state.cur_max_blocked_speed;
        if state.bump_speed_limit > speed {
            state.bump_speed_limit = speed;
        }
        state.bump_speed_limit *= 0.95;
        speed = state.bump_speed_limit;
    } else {
        *blocked = FALSE;
        if state.bump_speed_limit < FAST_AS_POSSIBLE {
            if state.bump_speed_limit < speed * 0.2 {
                state.bump_speed_limit = speed * 0.2;
            }
            state.bump_speed_limit *= 1.05;
        }
        if speed > state.bump_speed_limit {
            speed = state.bump_speed_limit;
        }
    }
    speed
}

/// The `POSITION_ON_PATH` case of `doLocomotor`. `Err` is the early return of a
/// unit still waiting for its path.
pub(super) fn drive_on_path(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    blocked: &mut Bool,
) -> Result<(), ()> {
    // Without a map a path may exist only behind the seams.
    if ai(sim, ctx).path.is_none() {
        return if ai(sim, ctx).waiting_for_path != FALSE {
            Err(())
        } else {
            Ok(())
        };
    }
    let (on_path_distance, goal) = if is_doing_ground_movement(sim, ctx) == FALSE {
        flight_dist_to_goal(sim, ctx)
    } else {
        let Some(info) = point_on_path(sim, ctx) else {
            return Ok(());
        };
        update_layer(sim, ctx.owner_slot, info.layer);
        (info.dist_along_path, info.pos_on_path)
    };
    let speed = path_speed(sim, ctx, blocked);
    let distance = on_path_distance + ai(sim, ctx).path_extra_distance;
    super::drive(sim, ctx, |l, w| {
        l.move_towards_position(w, &goal, distance, speed, blocked)
    });
    ai(sim, ctx).do_final_position = FALSE;
    Ok(())
}

/// `Pathfinder::updateLayer`.
fn update_layer(sim: &mut dyn SimApi, owner: ObjectSlot, wanted: PathfindLayerEnum) {
    let mut layer = wanted;
    if layer != PathfindLayerEnum::Ground {
        let interacts = sim.object(owner).is_some_and(|o| {
            let pos = *o.thing.get_position();
            let radius = o.geometry.get_minor_radius();
            let own = o.layer;
            sim.terrain().terrain_logic().is_some_and(|t| {
                t.object_interacts_with_bridge_layer(&pos, radius, own, wanted, true)
            })
        });
        if !interacts {
            layer = PathfindLayerEnum::Ground;
        }
    }
    sim.set_object_layer(owner, layer);
}

// -----------------------------------------------------------------------------
// Questions the movement states ask
// -----------------------------------------------------------------------------

/// `getTreatAsAircraftForLocoDistToGoal`.
fn treat_as_aircraft(sim: &mut dyn SimApi, ctx: ModuleContext) -> bool {
    let mut treat = is_doing_ground_movement(sim, ctx) == FALSE;
    let state = ai(sim, ctx);
    if state.path_extra_distance > PATHFIND_CLOSE_ENOUGH {
        treat = true;
    }
    if state
        .cur_locomotor()
        .is_some_and(|l| l.template.appearance == LocomotorAppearance::Hover)
    {
        treat = true;
    }
    treat
}

/// `getLocomotorDistanceToGoal`. It moves the path's cached point.
pub fn locomotor_distance_to_goal(sim: &mut dyn SimApi, ctx: ModuleContext) -> Real {
    let owner = ctx.owner_slot;
    let state = ai(sim, ctx);
    if state.locomotor_goal_type != LocoGoalType::PositionOnPath || state.path.is_none() {
        return 0.0;
    }
    let Some(locomotor) = state.cur_locomotor() else {
        return 0.0;
    };
    if locomotor.is_close_enough_dist3d() != FALSE || is_projectile(sim, owner) {
        let mut destination = ai(sim, ctx).machine.goal_position;
        if let Some(path) = ai(sim, ctx).path.as_ref() {
            if let Some(last) = path.last_node() {
                destination = *path.node(last).position();
            }
        }
        // `getDistanceSquared(me, dest, FROM_CENTER_3D)`, from the object's
        // position, not the middle of its geometry.
        let from = position_of(sim, owner);
        let dx = destination.x - from.x;
        let dy = destination.y - from.y;
        let dz = destination.z - from.z;
        return gm_sqrtf(dx * dx + dy * dy + dz * dz);
    }
    let aircraft = treat_as_aircraft(sim, ctx);
    let (distance, mut goal) = if aircraft {
        flight_dist_to_goal(sim, ctx)
    } else {
        let info = point_on_path(sim, ctx).unwrap_or_default();
        (info.dist_along_path, info.pos_on_path)
    };
    if let Some(path) = ai(sim, ctx).path.as_ref() {
        if let Some(last) = path.last_node() {
            goal = *path.node(last).position();
        }
    }
    let pos = position_of(sim, owner);
    let dx = goal.x - pos.x;
    let dy = goal.y - pos.y;
    let direct = dx * dx + dy * dy;
    if aircraft {
        return if distance * distance > direct {
            gm_sqrtf(direct)
        } else {
            distance
        };
    }
    if distance < PATHFIND_CELL_SIZE_F || distance * distance < direct {
        gm_sqrtf(direct)
    } else {
        distance
    }
}

/// `isValidLocomotorPosition`.
pub fn is_valid_locomotor_position(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    pos: &Coord3D,
) -> bool {
    let owner = ctx.owner_slot;
    let surfaces = ai(sim, ctx).locomotor_set.get_valid_surfaces();
    let (crusher, layer) = (is_crusher(sim, owner), layer_of(sim, owner));
    sim.pathfinder()
        .is_some_and(|p| p.valid_movement_position(crusher, layer, surfaces, pos))
}

// -----------------------------------------------------------------------------
// The searches that are not ported (S6)
// -----------------------------------------------------------------------------

/// `Pathfinder::findClosestPath` from where the unit stands. It may move `to`.
fn closest_path(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    to: &mut Coord3D,
    blocked: Bool,
    path_cost_factor: Real,
) -> Option<Path> {
    let from = position_of(sim, ctx.owner_slot);
    ai_seams(sim).closest_path(ctx.owner_id, &from, to, blocked, path_cost_factor)
}

/// `Pathfinder::patchPath` of the unit's current path.
fn patch_path(sim: &mut dyn SimApi, ctx: ModuleContext, blocked: Bool) -> Option<Path> {
    let current = ai(sim, ctx).path.clone();
    ai_seams(sim).patch_path(ctx.owner_id, current.as_ref(), blocked)
}

/// `Pathfinder::findSafePath`.
fn safe_path(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    repulsor1: &Coord3D,
    repulsor2: &Coord3D,
    radius: Real,
) -> Option<Path> {
    let from = position_of(sim, ctx.owner_slot);
    ai_seams(sim).safe_path(ctx.owner_id, &from, repulsor1, repulsor2, radius)
}

/// `computeAttackPath`: `None` for FALSE, else the path it left.
fn attack_path(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    victim: Option<ObjectId>,
    victim_pos: &Coord3D,
) -> Option<Option<Path>> {
    ai_seams(sim).attack_path(ctx.owner_id, victim, victim_pos)
}
