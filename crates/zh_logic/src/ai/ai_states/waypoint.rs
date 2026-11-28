//! `PKT-429`: the waypoint states of `AIStates.cpp`: `AIFollowWaypointPathState`
//! (as individuals and as a team), `AIFollowWaypointPathExactState` (both),
//! `AIWanderState`, `AIWanderInPlaceState`, `AIPanicState` and
//! `AIMoveAwayFromRepulsorsState`. Packet doc:
//! `packets/phase6/PKT-429_AIStates.md`. Golden test:
//! `tests/golden_s11_waypoint_states.rs`.
//!
//! `AIAttackFollowWaypointPathState` is not here: it is answered by
//! [`crate::ai::seams::AiSeams`]. Waypoints are those of the map's
//! `TerrainLogic`, kept by id.

use crate::ai::ai_state_machine::state_id;
use crate::ai::ai_states::movement::{
    adjust_destination, adjusts_destination, compute_path, internal_on_enter, internal_on_exit,
    internal_update, last_node, position, remove_goal, set_adjusts_destination,
    set_use_precise_z_pos, InternalMove, TightenData, MAA_ACTION_TO_ATTACK_MOVE,
};
use crate::ai::seams::{MapTerrain, WaypointInfo};
use crate::arena::ModuleContext;
use crate::combat::locomotor_set::LocomotorSetType;
use crate::modules::update::ai_update::{
    self, ai, AiCommandParms, AiCommandType, MoodMatrixAction, FAST_AS_POSSIBLE,
};
use crate::object::weapon::NO_MAX_SHOTS_LIMIT;
use crate::pathfinder::path::Path;
use crate::pathfinder::pathfind_constants::{PathfindLayerEnum, PATHFIND_CELL_SIZE_F};
use crate::rules::kind_of::KindOfType;
use crate::rules::model_state::ModelConditionFlagType;
use crate::sim::SimApi;
use zh_common::core::state_machine::{StateId, StateReturnType};
use zh_common::math::gamemath::real_to_int_floor;
use zh_common::math::{Coord2D, Coord3D};
use zh_common::types::{Bool, Int, Real, UnsignedInt, XferVersion, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// `INVALID_WAYPOINT_ID`.
const INVALID_WAYPOINT_ID: UnsignedInt = 0x7fff_ffff;
/// `WAYPOINT_PATH_LIMIT`.
const WAYPOINT_PATH_LIMIT: Int = 1024;

/// Whether this module ports the state.
pub fn is_waypoint_state(id: StateId) -> bool {
    matches!(
        id,
        state_id::FOLLOW_WAYPOINT_PATH_AS_TEAM
            | state_id::FOLLOW_WAYPOINT_PATH_AS_INDIVIDUALS
            | state_id::FOLLOW_WAYPOINT_PATH_AS_TEAM_EXACT
            | state_id::FOLLOW_WAYPOINT_PATH_AS_INDIVIDUALS_EXACT
            | state_id::WANDER
            | state_id::PANIC
            | state_id::WANDER_IN_PLACE
            | state_id::MOVE_AWAY_FROM_REPULSORS
    )
}

/// The members of an `AIFollowWaypointPathState`, with those `AIWanderState` and
/// `AIPanicState` add. The reference leaves most of them to the pool's zeroes,
/// and a stay keeps what it does not set.
#[derive(Debug, Clone, Copy, Default)]
pub struct WaypointFollow {
    pub base: InternalMove,
    pub group_offset: Coord2D,
    /// `m_angle`; only rotated group offsets (compiled out) change it.
    pub angle: Real,
    pub frames_sleeping: Int,
    pub current: Option<UnsignedInt>,
    pub prior: Option<UnsignedInt>,
    pub append_goal_position: Bool,
    pub wait_frames: Int,
    pub timer: Int,
}

/// The members of an `AIFollowWaypointPathExactState`.
#[derive(Debug, Clone, Copy, Default)]
pub struct ExactFollow {
    pub base: InternalMove,
    pub last: Option<UnsignedInt>,
}

/// The members of an `AIWanderInPlaceState`.
#[derive(Debug, Clone, Copy, Default)]
pub struct WanderInPlace {
    pub base: InternalMove,
    pub origin: Coord3D,
    pub wait_frames: Int,
    pub timer: Int,
}

/// The state objects of this module in an `AIStateMachine`.
#[derive(Debug, Clone, Copy, Default)]
pub struct WaypointStates {
    pub follow_team: WaypointFollow,
    pub follow_individuals: WaypointFollow,
    pub wander: WaypointFollow,
    pub panic: WaypointFollow,
    pub exact_team: ExactFollow,
    pub exact_individuals: ExactFollow,
    pub wander_in_place: WanderInPlace,
    pub repulsors: TightenData,
}

fn xfer_waypoint(xfer: &mut dyn Xfer, waypoint: &mut Option<UnsignedInt>) -> Result<(), XferError> {
    let mut id = waypoint.unwrap_or(INVALID_WAYPOINT_ID);
    xfer.xfer_unsigned_int(&mut id)?;
    // A load looks the id up; one the map lacks gives no waypoint.
    *waypoint = Some(id).filter(|&id| id != INVALID_WAYPOINT_ID);
    Ok(())
}

impl WaypointStates {
    pub(super) fn internal(&mut self, id: StateId) -> &mut InternalMove {
        match id {
            state_id::FOLLOW_WAYPOINT_PATH_AS_TEAM_EXACT => &mut self.exact_team.base,
            state_id::FOLLOW_WAYPOINT_PATH_AS_INDIVIDUALS_EXACT => &mut self.exact_individuals.base,
            state_id::WANDER_IN_PLACE => &mut self.wander_in_place.base,
            state_id::MOVE_AWAY_FROM_REPULSORS => &mut self.repulsors.base,
            _ => &mut self.follow(id).base,
        }
    }

    fn follow(&mut self, id: StateId) -> &mut WaypointFollow {
        match id {
            state_id::FOLLOW_WAYPOINT_PATH_AS_TEAM => &mut self.follow_team,
            state_id::WANDER => &mut self.wander,
            state_id::PANIC => &mut self.panic,
            _ => &mut self.follow_individuals,
        }
    }

    fn exact(&mut self, id: StateId) -> &mut ExactFollow {
        if id == state_id::FOLLOW_WAYPOINT_PATH_AS_TEAM_EXACT {
            &mut self.exact_team
        } else {
            &mut self.exact_individuals
        }
    }

    /// `State::xfer` of one of these states.
    pub(crate) fn xfer(&mut self, id: StateId, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        match id {
            state_id::FOLLOW_WAYPOINT_PATH_AS_TEAM_EXACT
            | state_id::FOLLOW_WAYPOINT_PATH_AS_INDIVIDUALS_EXACT => {
                let state = self.exact(id);
                xfer.xfer_version(&mut version, 1)?;
                state.base.xfer(xfer)?;
                xfer_waypoint(xfer, &mut state.last)
            }
            state_id::WANDER_IN_PLACE => {
                let state = &mut self.wander_in_place;
                xfer.xfer_version(&mut version, 1)?;
                state.base.xfer(xfer)?;
                xfer.xfer_coord3d(&mut state.origin)?;
                xfer.xfer_int(&mut state.wait_frames)?;
                xfer.xfer_int(&mut state.timer)
            }
            state_id::MOVE_AWAY_FROM_REPULSORS => {
                let state = &mut self.repulsors;
                xfer.xfer_version(&mut version, 1)?;
                state.base.xfer(xfer)?;
                xfer.xfer_int(&mut state.ok_to_repath_times)?;
                xfer.xfer_bool(&mut state.check_for_path)
            }
            _ => {
                let timed = id == state_id::WANDER || id == state_id::PANIC;
                if timed {
                    // The version of the derived class.
                    xfer.xfer_version(&mut version, 1)?;
                    version = 1;
                }
                let state = self.follow(id);
                xfer.xfer_version(&mut version, 1)?;
                state.base.xfer(xfer)?;
                xfer.xfer_coord2d(&mut state.group_offset)?;
                xfer.xfer_real(&mut state.angle)?;
                xfer.xfer_int(&mut state.frames_sleeping)?;
                xfer_waypoint(xfer, &mut state.current)?;
                xfer_waypoint(xfer, &mut state.prior)?;
                xfer.xfer_bool(&mut state.append_goal_position)?;
                if timed {
                    xfer.xfer_int(&mut state.wait_frames)?;
                    xfer.xfer_int(&mut state.timer)?;
                }
                Ok(())
            }
        }
    }
}

fn states(sim: &mut dyn SimApi, ctx: ModuleContext) -> &mut WaypointStates {
    &mut ai(sim, ctx).machine.moves.waypoint
}

fn follow(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> &mut WaypointFollow {
    states(sim, ctx).follow(id)
}

// -----------------------------------------------------------------------------
// Waypoints
// -----------------------------------------------------------------------------

fn waypoint(sim: &dyn SimApi, id: Option<UnsignedInt>) -> Option<WaypointInfo> {
    let terrain = sim.terrain().terrain_logic()?;
    MapTerrain::waypoint_by_id(terrain, id?)
}

/// `getGoalWaypoint`.
fn goal_waypoint(sim: &mut dyn SimApi, ctx: ModuleContext) -> Option<WaypointInfo> {
    let id = ai(sim, ctx).machine.goal_waypoint.as_ref().map(|w| w.0);
    waypoint(sim, id)
}

fn set_machine_goal(sim: &mut dyn SimApi, ctx: ModuleContext, pos: &Coord3D) {
    ai(sim, ctx).machine.set_goal_position(Some(pos));
}

/// `calcExtraPathDistance`: along the first links, five waypoints ahead.
fn calc_extra_path_distance(sim: &dyn SimApi, current: Option<UnsignedInt>) -> Real {
    let mut extra = PATHFIND_CELL_SIZE_F / 10.0;
    let mut current = waypoint(sim, current);
    let mut limit = 5;
    while let Some(way) = current {
        if limit <= 0 {
            break;
        }
        limit -= 1;
        let Some(next) = waypoint(sim, way.links.first().copied()) else {
            return extra;
        };
        let delta = Coord2D {
            x: next.location.x - way.location.x,
            y: next.location.y - way.location.y,
        };
        extra += delta.length();
        current = Some(next);
    }
    extra
}

fn has_next_waypoint(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> bool {
    let current = follow(sim, ctx, id).current;
    waypoint(sim, current).is_some_and(|w| !w.links.is_empty())
}

/// `getNextWaypoint` (`ALLOW_BACKTRACK`): a random link. The machine's goal is
/// the waypoint being left.
fn get_next_waypoint(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> Option<UnsignedInt> {
    let current = follow(sim, ctx, id).current;
    let way = waypoint(sim, current);
    let count = way.as_ref().map_or(0, |w| w.links.len() as Int);
    let which = sim.rng_logic_int(0, count.wrapping_sub(1));
    let next = way
        .as_ref()
        .and_then(|w| w.links.get(usize::try_from(which).ok()?).copied());
    follow(sim, ctx, id).prior = current;
    if let Some(way) = way {
        set_machine_goal(sim, ctx, &way.location);
    }
    next
}

fn is_point_on_wall(sim: &dyn SimApi, pos: &Coord3D) -> Option<Real> {
    let map = sim.pathfinder()?;
    map.is_point_on_wall(pos).then(|| map.wall_height())
}

fn set_allow_invalid_position(sim: &mut dyn SimApi, ctx: ModuleContext, allow: Bool) {
    if let Some(locomotor) = ai(sim, ctx).cur_locomotor_mut() {
        locomotor.set_allow_invalid_position(allow);
    }
}

/// `computeGoal`.
fn compute_goal(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) {
    let current = follow(sim, ctx, id).current;
    let Some(way) = waypoint(sim, current) else {
        return;
    };
    let mut dest = way.location;
    // Waypoints are on the ground, except when they are on the wall.
    let mut layer = PathfindLayerEnum::Ground;
    if let Some(height) = is_point_on_wall(sim, &dest) {
        dest.z = height;
        layer = PathfindLayerEnum::Wall;
    }
    follow(sim, ctx, id).base.goal_layer = layer;

    let extra = calc_extra_path_distance(sim, current);
    ai(sim, ctx).path_extra_distance = extra;
    if has_next_waypoint(sim, ctx, id) {
        set_adjusts_destination(sim, ctx, id, false);
    } else {
        set_adjusts_destination(sim, ctx, id, true);
        if sim.object_is_kind_of(ctx.owner_slot, KindOfType::Projectile) != FALSE {
            set_use_precise_z_pos(sim, ctx, TRUE);
        }
    }

    let offset = follow(sim, ctx, id).group_offset;
    let mut goal = dest;
    goal.x += offset.x;
    goal.y += offset.y;
    if layer == PathfindLayerEnum::Wall {
        if is_point_on_wall(sim, &goal).is_none() {
            goal = dest;
        }
    } else {
        goal.z = sim.terrain().get_ground_height(goal.x, goal.y, None);
    }

    let extent = ai_update::maximum_pathfind_extent(sim);
    let at = |p: &Coord3D| Coord2D { x: p.x, y: p.y };
    // A group offset is not to push a waypoint of the map off it.
    if extent.contains_within_coord2d(&at(&dest)) != FALSE
        && extent.contains_within_coord2d(&at(&goal)) == FALSE
    {
        if goal.x < extent.lo.x + PATHFIND_CELL_SIZE_F {
            goal.x = extent.lo.x + PATHFIND_CELL_SIZE_F;
        }
        if goal.y < extent.lo.y + PATHFIND_CELL_SIZE_F {
            goal.y = extent.lo.y + PATHFIND_CELL_SIZE_F;
        }
        if goal.x > extent.hi.x - PATHFIND_CELL_SIZE_F {
            goal.x = extent.hi.x - PATHFIND_CELL_SIZE_F;
        }
        if goal.y > extent.hi.y - PATHFIND_CELL_SIZE_F {
            goal.y = extent.hi.y - PATHFIND_CELL_SIZE_F;
        }
    }
    follow(sim, ctx, id).base.goal_position = goal;

    if extent.contains_within_coord2d(&at(&goal)) == FALSE {
        // Moving off the map.
        set_adjusts_destination(sim, ctx, id, false);
        set_allow_invalid_position(sim, ctx, TRUE);
        follow(sim, ctx, id).append_goal_position = TRUE;
    }
}

/// `PATHFIND_CELL_SIZE_F` wander offsets, for a locomotor that wanders sideways.
fn draw_wander_offset(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) {
    let width = ai(sim, ctx)
        .cur_locomotor()
        .map_or(0.0, |l| l.template.wander_width_factor);
    if width > 0.0 {
        let mut delta = real_to_int_floor(width + 0.5);
        if delta < 1 {
            delta = 1;
        }
        let x = sim.rng_logic_int(-delta, delta) as Real * PATHFIND_CELL_SIZE_F;
        let y = sim.rng_logic_int(-delta, delta) as Real * PATHFIND_CELL_SIZE_F;
        follow(sim, ctx, id).group_offset = Coord2D { x, y };
    }
}

fn wait_frames(ctx: ModuleContext) -> Int {
    10 + (ctx.owner_id.0 & 0x7) as Int
}

fn closest_repulsor(sim: &mut dyn SimApi, ctx: ModuleContext) -> bool {
    let range = sim.object_vision_range(ctx.owner_slot);
    ai_update::closest_repulsor(sim, ctx, range).is_some()
}

/// The periodic look for a repulsor of the wandering and panicking states.
fn sees_repulsor(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    timer: impl Fn(&mut WaypointStates) -> (&mut Int, Int),
) -> bool {
    if sim.object_is_kind_of(ctx.owner_slot, KindOfType::CanBeRepulsed) == FALSE {
        return false;
    }
    let (timer, wait) = timer(states(sim, ctx));
    *timer = timer.wrapping_sub(1);
    if *timer >= 0 {
        return false;
    }
    *timer = wait;
    closest_repulsor(sim, ctx)
}

fn follow_sees_repulsor(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> bool {
    sees_repulsor(sim, ctx, move |states| {
        let state = states.follow(id);
        let wait = state.wait_frames;
        (&mut state.timer, wait)
    })
}

// -----------------------------------------------------------------------------
// `AIFollowWaypointPathState`
// -----------------------------------------------------------------------------

fn as_team(id: StateId) -> bool {
    id == state_id::FOLLOW_WAYPOINT_PATH_AS_TEAM
        || id == state_id::FOLLOW_WAYPOINT_PATH_AS_TEAM_EXACT
}

/// The speed of the unit's group and where the unit stands in it.
fn group_offset(sim: &mut dyn SimApi, ctx: ModuleContext) -> Option<(Real, Coord2D)> {
    let (speed, center) = sim.ai_group_speed_and_center(ctx.owner_slot)?;
    let pos = position(sim, ctx.owner_slot);
    let offset = Coord2D {
        x: pos.x - center.x,
        y: pos.y - center.y,
    };
    Some((speed, offset))
}

/// `adjustDestination` and `updateGoal` as the end of a path asks for them.
fn ground_adjusts(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> bool {
    adjusts_destination(sim, ctx, id) && ai_update::is_doing_ground_movement(sim, ctx) != FALSE
}

fn adjust_goal(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> bool {
    let mut goal = follow(sim, ctx, id).base.goal_position;
    let found = adjust_destination(sim, ctx, &mut goal);
    follow(sim, ctx, id).base.goal_position = goal;
    found
}

fn reserve(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) {
    let (goal, layer) = {
        let base = &follow(sim, ctx, id).base;
        (base.goal_position, base.goal_layer)
    };
    ai_update::update_goal(sim, ctx, &goal, layer);
}

/// Sets off for the waypoint the state has just moved to. `false` when there is
/// no cell to go to.
fn head_for_waypoint(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> bool {
    compute_goal(sim, ctx, id);
    if ground_adjusts(sim, ctx, id) && !adjust_goal(sim, ctx, id) {
        return false;
    }
    ai(sim, ctx).starting_move();
    let _ = compute_path(sim, ctx, id);
    if adjusts_destination(sim, ctx, id) {
        reserve(sim, ctx, id);
    }
    true
}

fn follow_on_enter(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> StateReturnType {
    let team = as_team(id);
    let way = goal_waypoint(sim, ctx);
    let state = follow(sim, ctx, id);
    state.append_goal_position = FALSE;
    state.prior = None;
    state.current = way.as_ref().map(|w| w.id);
    // A team order without a waypoint crashes the reference.
    let Some(way) = way else {
        return StateReturnType::Failure;
    };
    set_machine_goal(sim, ctx, &way.location);
    let state = follow(sim, ctx, id);
    state.frames_sleeping = 0;
    state.group_offset = Coord2D::default();

    let mut speed = FAST_AS_POSSIBLE;
    if team {
        sim.ai_set_team_waypoint(ctx.owner_slot, Some(way.id));
        if let Some((group_speed, offset)) = group_offset(sim, ctx) {
            speed = group_speed;
            follow(sim, ctx, id).group_offset = offset;
        }
    }
    compute_goal(sim, ctx, id);
    let result = internal_on_enter(sim, ctx, id);
    ai(sim, ctx).desired_speed = speed;

    let current = follow(sim, ctx, id).current;
    let extra = calc_extra_path_distance(sim, current);
    ai(sim, ctx).path_extra_distance = extra;
    if has_next_waypoint(sim, ctx, id) {
        set_adjusts_destination(sim, ctx, id, false);
    } else {
        let ground = ai_update::is_doing_ground_movement(sim, ctx) != FALSE;
        set_adjusts_destination(sim, ctx, id, ground);
        if adjusts_destination(sim, ctx, id) {
            if !adjust_goal(sim, ctx, id) {
                return StateReturnType::Failure;
            }
            reserve(sim, ctx, id);
        }
        if sim.object_is_kind_of(ctx.owner_slot, KindOfType::Projectile) != FALSE {
            set_use_precise_z_pos(sim, ctx, TRUE);
        }
    }
    result
}

fn follow_on_exit(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) {
    internal_on_exit(sim, ctx, id);
    set_use_precise_z_pos(sim, ctx, FALSE);
    if let Some(locomotor) = ai(sim, ctx).cur_locomotor_mut() {
        locomotor.set_ultra_accurate(FALSE);
    }
}

fn follow_update(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> StateReturnType {
    let state = follow(sim, ctx, id);
    if state.frames_sleeping > 0 {
        state.frames_sleeping -= 1;
        return StateReturnType::Continue;
    }
    let current = state.current;
    let Some(way) = waypoint(sim, current) else {
        return StateReturnType::Failure;
    };
    set_machine_goal(sim, ctx, &way.location);

    let adjustment = ai_update::mood_matrix_action_adjustment(sim, ctx, MoodMatrixAction::Move);
    if adjustment & MAA_ACTION_TO_ATTACK_MOVE != 0 {
        // The order leaves this state, which still runs to its end.
        let command = if as_team(id) {
            AiCommandType::AttackFollowWaypointPathAsTeam
        } else {
            AiCommandType::AttackFollowWaypointPath
        };
        let mut parms =
            AiCommandParms::new(command, crate::modules::module::CommandSourceType::FromAi);
        parms.waypoint = Some((way.id, way.name.clone()));
        parms.int_value = NO_MAX_SHOTS_LIMIT;
        ai_update::execute_command(sim, ctx, &parms);
    }

    if follow(sim, ctx, id).append_goal_position != FALSE {
        let goal = follow(sim, ctx, id).base.goal_position;
        let unit = ai(sim, ctx);
        if unit.waiting_for_path == FALSE {
            if let Some(path) = unit.path.as_mut() {
                path.append_node(&goal, PathfindLayerEnum::Ground);
                follow(sim, ctx, id).append_goal_position = FALSE;
            }
        }
    }

    if as_team(id) {
        let shared = sim.ai_team_waypoint(ctx.owner_slot);
        let state = follow(sim, ctx, id);
        if state.current != shared {
            state.prior = state.current;
            state.current = shared;
            if shared.is_none() {
                return StateReturnType::Success;
            }
            if !head_for_waypoint(sim, ctx, id) {
                return StateReturnType::Failure;
            }
        }
    }

    let mut status = internal_update(sim, ctx, id);

    // A computer team counts itself there when its middle is near enough.
    if as_team(id) {
        let team = sim.object(ctx.owner_slot).map(|o| o.team_id);
        if team.is_some_and(|team| sim.player_is_skirmish_ai(team) != FALSE) {
            let (mut pos, count) = sim.ai_team_as_group_center(ctx.owner_slot);
            let goal = follow(sim, ctx, id).base.goal_position;
            pos.x -= goal.x;
            pos.y -= goal.y;
            pos.z = 0.0;
            if pos.length() <= count as Real * sim.rules().ai_data.skirmish_group_fudge_value {
                status = StateReturnType::Success;
            }
        }
    }

    if status == StateReturnType::Continue {
        return status;
    }
    let next = get_next_waypoint(sim, ctx, id);
    follow(sim, ctx, id).current = next;
    let prior = follow(sim, ctx, id).prior;
    let unit = ai(sim, ctx);
    if let Some(prior) = prior {
        unit.prior_waypoint_id = prior;
    }
    if let Some(next) = next {
        unit.current_waypoint_id = next;
    } else {
        unit.completed_waypoint = prior;
        return StateReturnType::Success;
    }
    if as_team(id) {
        sim.ai_set_team_waypoint(ctx.owner_slot, next);
    }
    if !head_for_waypoint(sim, ctx, id) {
        return StateReturnType::Failure;
    }
    StateReturnType::Continue
}

// -----------------------------------------------------------------------------
// `AIFollowWaypointPathExactState`
// -----------------------------------------------------------------------------

/// `setPathFromWaypoint`: the first links as one path, taken as it is.
fn set_path_from_waypoint(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    from: &WaypointInfo,
    offset: &Coord2D,
) {
    ai(sim, ctx).destroy_path();
    let mut path = Path::new();
    let pos = position(sim, ctx.owner_slot);
    path.prepend_node(&pos, PathfindLayerEnum::Ground);
    path.mark_optimized();
    let mut count = 0;
    let mut way = Some(from.clone());
    while let Some(current) = way {
        let mut point = current.location;
        point.x += offset.x;
        point.y += offset.y;
        let next = current.links.first().copied();
        if next.is_none() {
            sim.pathfinder_snap_position(ctx.owner_slot, &mut point);
        }
        path.append_node(&point, PathfindLayerEnum::Ground);
        way = waypoint(sim, next);
        count += 1;
        if count > WAYPOINT_PATH_LIMIT {
            break;
        }
    }
    let unit = ai(sim, ctx);
    unit.has_path = TRUE;
    unit.path = Some(path);
    unit.waiting_for_path = FALSE;
    ai_update::wake_up_now(sim, ctx);
}

fn exact_on_enter(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> StateReturnType {
    let Some(way) = goal_waypoint(sim, ctx) else {
        return StateReturnType::Failure;
    };
    set_machine_goal(sim, ctx, &way.location);
    let mut offset = Coord2D::default();
    let mut speed = FAST_AS_POSSIBLE;
    if as_team(id) {
        if let Some((group_speed, group)) = group_offset(sim, ctx) {
            speed = group_speed;
            offset = group;
        }
    }
    ai(sim, ctx).set_can_path_through_units(TRUE);
    set_adjusts_destination(sim, ctx, id, false);
    let result = internal_on_enter(sim, ctx, id);
    set_path_from_waypoint(sim, ctx, &way, &offset);
    states(sim, ctx).exact(id).last = Some(way.id);
    set_allow_invalid_position(sim, ctx, TRUE);
    ai(sim, ctx).desired_speed = speed;
    result
}

fn exact_on_exit(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) {
    internal_on_exit(sim, ctx, id);
    let last = states(sim, ctx).exact(id).last;
    let unit = ai(sim, ctx);
    unit.completed_waypoint = last;
    unit.set_can_path_through_units(FALSE);
    set_allow_invalid_position(sim, ctx, FALSE);
}

// -----------------------------------------------------------------------------
// `AIWanderState` and `AIPanicState`
// -----------------------------------------------------------------------------

fn wander_on_enter(sim: &mut dyn SimApi, ctx: ModuleContext) -> StateReturnType {
    const ID: StateId = state_id::WANDER;
    let current = goal_waypoint(sim, ctx).map(|w| w.id);
    let state = follow(sim, ctx, ID);
    state.current = current;
    state.prior = None;
    if current.is_none() {
        return StateReturnType::Failure;
    }
    state.group_offset = Coord2D::default();
    draw_wander_offset(sim, ctx, ID);
    let state = follow(sim, ctx, ID);
    state.timer = 0;
    state.wait_frames = wait_frames(ctx);
    compute_goal(sim, ctx, ID);
    let result = internal_on_enter(sim, ctx, ID);
    let extra = calc_extra_path_distance(sim, current);
    ai(sim, ctx).path_extra_distance = extra;
    result
}

/// Takes the next link, or ends the walk.
fn wander_on(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> StateReturnType {
    let next = get_next_waypoint(sim, ctx, id);
    follow(sim, ctx, id).current = next;
    if next.is_none() {
        let prior = follow(sim, ctx, id).prior;
        ai(sim, ctx).completed_waypoint = prior;
        return StateReturnType::Success;
    }
    draw_wander_offset(sim, ctx, id);
    compute_goal(sim, ctx, id);
    let _ = compute_path(sim, ctx, id);
    StateReturnType::Continue
}

fn wander_update(sim: &mut dyn SimApi, ctx: ModuleContext) -> StateReturnType {
    const ID: StateId = state_id::WANDER;
    let status = internal_update(sim, ctx, ID);
    if follow_sees_repulsor(sim, ctx, ID) {
        return StateReturnType::Failure;
    }
    if status != StateReturnType::Continue {
        return wander_on(sim, ctx, ID);
    }
    StateReturnType::Continue
}

fn panic_on_enter(sim: &mut dyn SimApi, ctx: ModuleContext) -> StateReturnType {
    const ID: StateId = state_id::PANIC;
    let current = goal_waypoint(sim, ctx).map(|w| w.id);
    follow(sim, ctx, ID).current = current;
    if current.is_none() {
        return StateReturnType::Failure;
    }
    // Unlike wandering, the offset and the prior waypoint are kept.
    draw_wander_offset(sim, ctx, ID);
    compute_goal(sim, ctx, ID);
    let result = internal_on_enter(sim, ctx, ID);
    let state = follow(sim, ctx, ID);
    state.timer = 0;
    state.wait_frames = wait_frames(ctx);
    let extra = calc_extra_path_distance(sim, current);
    ai(sim, ctx).path_extra_distance = extra;
    sim.set_model_condition_state(ctx.owner_slot, ModelConditionFlagType::Panicking);
    result
}

fn panic_update(sim: &mut dyn SimApi, ctx: ModuleContext) -> StateReturnType {
    const ID: StateId = state_id::PANIC;
    let status = internal_update(sim, ctx, ID);
    if follow_sees_repulsor(sim, ctx, ID) {
        return StateReturnType::Failure;
    }
    if status == StateReturnType::Success {
        return wander_on(sim, ctx, ID);
    }
    StateReturnType::Continue
}

// -----------------------------------------------------------------------------
// `AIWanderInPlaceState`
// -----------------------------------------------------------------------------

/// A point of the cell grid around the origin.
fn wander_goal(sim: &mut dyn SimApi, ctx: ModuleContext) {
    let delta = ai(sim, ctx).cur_locomotor().map_or(3, |l| {
        real_to_int_floor((l.template.wander_about_point_radius / PATHFIND_CELL_SIZE_F) + 0.5)
    });
    let x = sim.rng_logic_int(delta.wrapping_neg(), delta) as Real * PATHFIND_CELL_SIZE_F;
    let y = sim.rng_logic_int(delta.wrapping_neg(), delta) as Real * PATHFIND_CELL_SIZE_F;
    let state = &mut states(sim, ctx).wander_in_place;
    state.base.goal_position = state.origin;
    state.base.goal_position.x += x;
    state.base.goal_position.y += y;
}

fn wander_in_place_on_enter(sim: &mut dyn SimApi, ctx: ModuleContext) -> StateReturnType {
    let pos = position(sim, ctx.owner_slot);
    states(sim, ctx).wander_in_place.origin = pos;
    let _ = ai_update::choose_locomotor_set(sim, ctx, LocomotorSetType::Wander);
    wander_goal(sim, ctx);
    let state = &mut states(sim, ctx).wander_in_place;
    state.timer = 0;
    state.wait_frames = wait_frames(ctx);
    internal_on_enter(sim, ctx, state_id::WANDER_IN_PLACE)
}

fn wander_in_place_update(sim: &mut dyn SimApi, ctx: ModuleContext) -> StateReturnType {
    const ID: StateId = state_id::WANDER_IN_PLACE;
    let status = internal_update(sim, ctx, ID);
    let seen = sees_repulsor(sim, ctx, |states| {
        let state = &mut states.wander_in_place;
        let wait = state.wait_frames;
        (&mut state.timer, wait)
    });
    if seen {
        return StateReturnType::Failure;
    }
    if status != StateReturnType::Continue {
        wander_goal(sim, ctx);
        let _ = internal_on_enter(sim, ctx, ID);
    }
    StateReturnType::Continue
}

// -----------------------------------------------------------------------------
// `AIMoveAwayFromRepulsorsState`
// -----------------------------------------------------------------------------

/// `computePath`: the safe path was asked for on entry; one more try at most.
pub(super) fn repulsors_compute_path(sim: &mut dyn SimApi, ctx: ModuleContext) -> bool {
    let state = &mut states(sim, ctx).repulsors;
    if state.ok_to_repath_times > 0 {
        state.ok_to_repath_times -= 1;
        return true;
    }
    false
}

fn repulsors_on_enter(sim: &mut dyn SimApi, ctx: ModuleContext) -> StateReturnType {
    const ID: StateId = state_id::MOVE_AWAY_FROM_REPULSORS;
    set_adjusts_destination(sim, ctx, ID, false);
    let range = sim.object_vision_range(ctx.owner_slot);
    let Some(enemy) = ai_update::closest_repulsor(sim, ctx, range) else {
        return StateReturnType::Failure;
    };
    let _ = ai_update::choose_locomotor_set(sim, ctx, LocomotorSetType::Panic);
    sim.set_model_condition_state(ctx.owner_slot, ModelConditionFlagType::Panicking);
    let state = &mut states(sim, ctx).repulsors;
    state.ok_to_repath_times = 1;
    state.check_for_path = TRUE;
    remove_goal(sim, ctx);
    ai_update::request_safe_path(sim, ctx, enemy);
    internal_on_enter(sim, ctx, ID)
}

fn repulsors_update(sim: &mut dyn SimApi, ctx: ModuleContext) -> StateReturnType {
    const ID: StateId = state_id::MOVE_AWAY_FROM_REPULSORS;
    if states(sim, ctx).repulsors.check_for_path != FALSE && ai(sim, ctx).waiting_for_path == FALSE
    {
        if let Some((pos, _)) = last_node(sim, ctx) {
            states(sim, ctx).repulsors.base.goal_position = pos;
            set_adjusts_destination(sim, ctx, ID, false);
            states(sim, ctx).repulsors.check_for_path = FALSE;
        }
    }
    internal_update(sim, ctx, ID)
}

// -----------------------------------------------------------------------------
// The state table
// -----------------------------------------------------------------------------

pub fn on_enter(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> StateReturnType {
    match id {
        state_id::FOLLOW_WAYPOINT_PATH_AS_TEAM_EXACT
        | state_id::FOLLOW_WAYPOINT_PATH_AS_INDIVIDUALS_EXACT => exact_on_enter(sim, ctx, id),
        state_id::WANDER => wander_on_enter(sim, ctx),
        state_id::PANIC => panic_on_enter(sim, ctx),
        state_id::WANDER_IN_PLACE => wander_in_place_on_enter(sim, ctx),
        state_id::MOVE_AWAY_FROM_REPULSORS => repulsors_on_enter(sim, ctx),
        _ => follow_on_enter(sim, ctx, id),
    }
}

pub fn update(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> StateReturnType {
    match id {
        state_id::FOLLOW_WAYPOINT_PATH_AS_TEAM_EXACT
        | state_id::FOLLOW_WAYPOINT_PATH_AS_INDIVIDUALS_EXACT => {
            ai(sim, ctx).set_can_path_through_units(TRUE);
            internal_update(sim, ctx, id)
        }
        state_id::WANDER => wander_update(sim, ctx),
        state_id::PANIC => panic_update(sim, ctx),
        state_id::WANDER_IN_PLACE => wander_in_place_update(sim, ctx),
        state_id::MOVE_AWAY_FROM_REPULSORS => repulsors_update(sim, ctx),
        _ => follow_update(sim, ctx, id),
    }
}

pub fn on_exit(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) {
    match id {
        state_id::FOLLOW_WAYPOINT_PATH_AS_TEAM_EXACT
        | state_id::FOLLOW_WAYPOINT_PATH_AS_INDIVIDUALS_EXACT => exact_on_exit(sim, ctx, id),
        state_id::PANIC => {
            sim.clear_model_condition_state(ctx.owner_slot, ModelConditionFlagType::Panicking);
            internal_on_exit(sim, ctx, id);
        }
        state_id::WANDER_IN_PLACE => internal_on_exit(sim, ctx, id),
        state_id::MOVE_AWAY_FROM_REPULSORS => {
            internal_on_exit(sim, ctx, id);
            sim.clear_model_condition_state(ctx.owner_slot, ModelConditionFlagType::Panicking);
        }
        _ => follow_on_exit(sim, ctx, id),
    }
}
