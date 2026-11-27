//! `PKT-301`, blocked and collision handling (AI plan slice S9):
//! `processCollision`, `blockedBy`, the path priority and the speed a blocked
//! unit may keep.
//!
//! `other` is always the AI of the unit collided with. Not here: formations
//! (no object has a formation id yet, so the formation slow-down never applies).

use super::allies::{ai_of, has_status, is_moving_away_from, order_move_away};
use super::{ai, is_doing_ground_movement, position_of, AiCall, AiCommandParms, AiCommandType};
use crate::ai::ai_state_machine::state_id;
use crate::arena::{ModuleContext, ObjectSlot};
use crate::modules::module::CommandSourceType;
use crate::modules::update::physics_update;
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::SimApi;
use zh_common::ini::LOGICFRAMES_PER_SECOND;
use zh_common::math::{gm_acosf, gm_fabsf, gm_sqrtf, Coord3D};
use zh_common::types::{Bool, ObjectId, Real, Relationship, FALSE, TRUE};

const PATHFIND_CELL_SIZE_F: Real = 10.0;
const PI: Real = std::f32::consts::PI;

fn direction_of(sim: &dyn SimApi, slot: ObjectSlot) -> Coord3D {
    sim.object(slot)
        .map_or_else(Coord3D::default, |o| o.thing.get_unit_direction_vector_2d())
}

fn is_kind(sim: &dyn SimApi, slot: ObjectSlot, kind: KindOfType) -> bool {
    sim.object_is_kind_of(slot, kind) != FALSE
}

/// `PartitionManager::getRelativeAngle2D`: the angle from where the object
/// faces to `pos`.
fn relative_angle_2d(sim: &dyn SimApi, slot: ObjectSlot, pos: &Coord3D) -> Real {
    let from = position_of(sim, slot);
    let mut x = pos.x - from.x;
    let mut y = pos.y - from.y;
    let distance = gm_sqrtf(x * x + y * y);
    if distance == 0.0 {
        return 0.0;
    }
    let direction = direction_of(sim, slot);
    let inverse = 1.0 / distance;
    x *= inverse;
    y *= inverse;
    let mut cosine = direction.x * x + direction.y * y;
    if cosine < -1.0 {
        cosine = -1.0;
    } else if cosine > 1.0 {
        cosine = 1.0;
    }
    let mut value = gm_acosf(cosine);
    if direction.x * y - direction.y * x < 0.0 {
        value = -value;
    }
    value
}

/// `canCrushOrSquish(other, TEST_SQUISH_ONLY)`.
fn can_squish(sim: &dyn SimApi, crusher: ObjectSlot, victim: ObjectSlot) -> bool {
    let Some(object) = sim.object(crusher) else {
        return false;
    };
    object.is_disabled_by_type(DisabledType::Unmanned) == FALSE
        && sim.player_relationship(crusher, victim) != Relationship::Allies
        && sim.object_crusher_level(crusher) != 0
        && sim
            .rules()
            .name_keys
            .lookup_key("SquishCollide")
            .is_some_and(|key| sim.has_module_by_name_key(victim, key))
}

/// `hasHigherPathPriority`.
pub(super) fn has_higher_path_priority(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    other: ModuleContext,
) -> bool {
    let (me, them) = (ctx.owner_slot, other.owner_slot);
    let ours = |sim: &dyn SimApi, kind| is_kind(sim, me, kind);
    let theirs = |sim: &dyn SimApi, kind| is_kind(sim, them, kind);
    if ours(sim, KindOfType::Dozer) && !theirs(sim, KindOfType::Dozer) {
        return true;
    }
    if !ours(sim, KindOfType::Dozer) && theirs(sim, KindOfType::Dozer) {
        return false;
    }
    if ours(sim, KindOfType::Vehicle) && theirs(sim, KindOfType::Infantry) {
        return true;
    }
    if ours(sim, KindOfType::Infantry) && theirs(sim, KindOfType::Vehicle) {
        return false;
    }
    let our = direction_of(sim, me);
    let their = direction_of(sim, them);
    if our.x * their.x + our.y * their.y <= 0.0 {
        return ctx.owner_id.0 < other.owner_id.0;
    }
    let combined_x = our.x + their.x;
    let combined_y = our.y + their.y;
    let from = position_of(sim, me);
    let to = position_of(sim, them);
    let dot = combined_x * (to.x - from.x) + combined_y * (to.y - from.y);
    if dot > 0.0 {
        return false;
    }
    if dot < 0.0 {
        return true;
    }
    ctx.owner_id.0 < other.owner_id.0
}

/// `calculateMaxBlockedSpeed`: how fast to follow the unit in the way.
pub(super) fn calculate_max_blocked_speed(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    other: ModuleContext,
) -> Real {
    let (me, them) = (ctx.owner_slot, other.owner_slot);
    let our = direction_of(sim, me);
    let their = direction_of(sim, them);
    let from = position_of(sim, me);
    let to = position_of(sim, them);
    let mut x = to.x - from.x;
    let mut y = to.y - from.y;
    // `Coord2D::normalize`.
    let length = gm_sqrtf(x * x + y * y);
    if length != 0.0 {
        x /= length;
        y /= length;
    }
    let dot = x * their.x + y * their.y;
    if dot < 0.0 {
        return 0.0;
    }
    let factor = dot;
    let current = ai(sim, ctx).cur_max_blocked_speed;
    let Some(velocity) = physics_update::physics(sim, them).map(|p| p.vel) else {
        return current;
    };
    let away = gm_sqrtf(velocity.x * velocity.x + velocity.y * velocity.y) * factor;
    let dot = x * our.x + y * our.y;
    if dot <= 0.0 {
        return current;
    }
    let max = away / dot;
    if max > current {
        return current;
    }
    max
}

/// `blockedBy`.
pub(super) fn blocked_by(sim: &mut dyn SimApi, ctx: ModuleContext, other: ModuleContext) -> bool {
    let (me, them) = (ctx.owner_slot, other.owner_slot);
    let state = ai(sim, ctx);
    let goal = state.machine.goal_position;
    let cell = state.pathfind_goal_cell;
    let pos = position_of(sim, me);
    if cell.x > 0
        && cell.y > 0
        && gm_fabsf(goal.x - pos.x) < PATHFIND_CELL_SIZE_F
        && gm_fabsf(goal.y - pos.y) < PATHFIND_CELL_SIZE_F
    {
        return false;
    }
    if sim
        .path_units()
        .can_crush_or_squish(ctx.owner_id, other.owner_id)
    {
        return false;
    }
    if is_doing_ground_movement(sim, other) == FALSE {
        return false;
    }
    if ai(sim, ctx)
        .cur_locomotor()
        .is_some_and(|l| l.is_moving_backwards() != FALSE)
    {
        return false;
    }
    let other_moving = ai(sim, other).is_locomotor_goal_set();
    let other_pos = position_of(sim, them);
    let mut dx = pos.x - other_pos.x;
    let mut dy = pos.y - other_pos.y;
    let distance_sqr = dx * dx + dy * dy;
    // `INFANTRY_MOVES_THROUGH_INFANTRY` (`AIPathfind.h`).
    if is_kind(sim, me, KindOfType::Infantry) && is_kind(sim, them, KindOfType::Infantry) {
        return false;
    }
    if distance_sqr < PATHFIND_CELL_SIZE_F * PATHFIND_CELL_SIZE_F * 0.0001 {
        return has_higher_path_priority(sim, ctx, other);
    }
    let our = direction_of(sim, me);
    let their = direction_of(sim, them);
    let dot = our.x * their.x + our.y * their.y;
    if ai(sim, ctx).blocked_frames > LOGICFRAMES_PER_SECOND as u32 && dot <= 0.0 {
        return false;
    }
    let collision_angle = relative_angle_2d(sim, me, &other_pos);
    let other_angle = relative_angle_2d(sim, them, &pos);
    let mut limit = PI / 4.0;
    if collision_angle > PI / 2.0 || collision_angle < -PI / 2.0 {
        return false;
    }
    if !other_moving {
        limit *= 0.75;
    }
    if collision_angle > limit || collision_angle < -limit {
        if dot <= 0.0 {
            return false;
        }
        if !(other_moving && (other_angle > limit || other_angle < -limit)) {
            return false;
        }
        dx += our.x - their.x;
        dy += our.y - their.y;
        // Closing on a unit that goes first is not being blocked.
        if distance_sqr > dx * dx + dy * dy {
            if has_higher_path_priority(sim, ctx, other) {
                return false;
            }
        } else {
            return false;
        }
    }
    ai(sim, other).is_ai_dead == FALSE
}

/// `needToRotate`. It moves the path's cached point.
pub(super) fn need_to_rotate(sim: &mut dyn SimApi, ctx: ModuleContext) -> bool {
    let state = ai(sim, ctx);
    if state.waiting_for_path != FALSE {
        return true;
    }
    if state
        .cur_locomotor()
        .is_some_and(|l| l.template.wander_width_factor > 0.0)
    {
        return false;
    }
    let mut delta = 0.0;
    if let Some(info) = super::point_on_path(sim, ctx) {
        delta = relative_angle_2d(sim, ctx.owner_slot, &info.pos_on_path);
    }
    gm_fabsf(delta) > PI / 30.0
}

fn order_move_to(sim: &mut dyn SimApi, ctx: ModuleContext, pos: Coord3D) {
    let mut parms = AiCommandParms::new(AiCommandType::MoveToPosition, CommandSourceType::FromAi);
    parms.pos = pos;
    super::execute(sim, ctx, &AiCall::Command(parms));
}

/// `processCollision`: `TRUE` for the physics to treat it as a collision.
pub fn process_collision(sim: &mut dyn SimApi, ctx: ModuleContext, other: ObjectSlot) -> Bool {
    let me = ctx.owner_slot;
    let now = sim.frame();
    let state = ai(sim, ctx);
    if state.ignore_collisions_until > now || state.can_path_through_units != FALSE {
        return FALSE;
    }
    let Some(them) = sim.ai_context(other) else {
        return FALSE;
    };
    let self_moving = ai(sim, ctx).is_moving() != FALSE;
    let other_moving = ai(sim, them).is_moving() != FALSE;
    if is_doing_ground_movement(sim, ctx) == FALSE || is_doing_ground_movement(sim, them) == FALSE {
        return FALSE;
    }
    let other_id: ObjectId = them.owner_id;
    if self_moving {
        if !blocked_by(sim, ctx, them) {
            return FALSE;
        }
        if is_kind(sim, me, KindOfType::Infantry)
            && ai(sim, ctx).machine.current_state_id() == state_id::PANIC
        {
            return TRUE;
        }
        ai(sim, ctx).is_blocked = TRUE;
        if other_moving && ai(sim, them).waiting_for_path != FALSE {
            return FALSE;
        }
        let max = calculate_max_blocked_speed(sim, ctx, them);
        let state = ai(sim, ctx);
        if max < state.cur_max_blocked_speed {
            state.cur_max_blocked_speed = max;
        }
        // Vehicles push idle infantry out of the way.
        if !is_moving_away_from(sim, them, ctx.owner_id)
            && is_kind(sim, other, KindOfType::Infantry)
            && !is_kind(sim, me, KindOfType::Infantry)
        {
            if has_status(sim, other, ObjectStatusType::IsUsingAbility)
                || ai(sim, them).machine.is_in_busy_state() != FALSE
            {
                return FALSE;
            }
            order_move_away(sim, them, ctx.owner_id);
            return FALSE;
        }
        if ai(sim, ctx).blocked_frames == 0 {
            ai(sim, ctx).blocked_frames = 1;
        }
        if !need_to_rotate(sim, ctx) {
            if !other_moving {
                ai(sim, ctx).is_blocked_and_stuck = TRUE;
                return FALSE;
            }
            // Two units blocking each other: the one that goes second moves.
            if blocked_by(sim, them, ctx)
                && !need_to_rotate(sim, them)
                && !has_higher_path_priority(sim, ctx, them)
            {
                if ai_of(sim, other_id).is_some() {
                    order_move_away(sim, ctx, other_id);
                }
            }
        } else {
            ai(sim, ctx).blocked_frames = 1;
        }
        return FALSE;
    }
    if ai(sim, ctx).is_ai_dead != FALSE
        && is_kind(sim, me, KindOfType::Infantry)
        && can_squish(sim, other, me)
    {
        return TRUE;
    }
    let other_pos = position_of(sim, other);
    let pos = position_of(sim, me);
    let dx = pos.x - other_pos.x;
    let dy = pos.y - other_pos.y;
    if !other_moving && dx * dx + dy * dy < PATHFIND_CELL_SIZE_F * PATHFIND_CELL_SIZE_F * 0.25 {
        // Standing on each other: the idle ones step apart.
        if ai(sim, ctx).machine.current_state_id() == state_id::BUSY
            || has_status(sim, me, ObjectStatusType::IsUsingAbility)
        {
            return FALSE;
        }
        if ai(sim, ctx).is_idle() != FALSE {
            let mut safe = position_of(sim, me);
            sim.pathfinder_adjust_to_possible_destination(me, &mut safe);
            order_move_to(sim, ctx, safe);
        }
        if ai(sim, them).is_idle() != FALSE {
            let mut safe = other_pos;
            sim.pathfinder_adjust_to_possible_destination(other, &mut safe);
            order_move_to(sim, them, safe);
        }
    }
    FALSE
}
