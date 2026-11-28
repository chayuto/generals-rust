//! `AIUpdateInterface::getNextMoodTarget` over the sim: the targeting questions
//! of [`crate::ai::ai`] answered by the world, and the team's common target
//! (`Team::getTeamTargetObject` / `setTeamTargetObject`).
//!
//! Source: `AIUpdate.cpp`, `Team.cpp`, `Pathfinder::isViewBlockedByObstacle`.
//! Part of PKT-301 (`AIUpdate`). The mood and search logic is [`crate::ai::mood`]
//! (`tests/golden_s12_targeting.rs`).
//! No vector of its own for the world's answers; behaviour: `tests/attack_sim.rs`.

use zh_common::types::{Bool, Int, ObjectId, FALSE};

use super::{ai, mood_unit};
use crate::ai::ai::TargetingSeams;
use crate::ai::mood::{self, MoodState};
use crate::ai::seams::ai_seams;
use crate::arena::ModuleContext;
use crate::arena::ObjectSlot;
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};

/// `Team::getTeamTargetObject` for the team of `me` without its side effect:
/// the target, and whether the stored one is to be forgotten.
fn team_target(sim: &dyn SimView, me: ObjectSlot) -> (Option<ObjectSlot>, bool) {
    let Some(id) = sim
        .team_common_target(me)
        .filter(|&id| id != ObjectId::INVALID)
    else {
        return (None, false);
    };
    let target = sim.find_object(id).filter(|&slot| {
        let Some(object) = sim.object(slot) else {
            return false;
        };
        let status = |s: ObjectStatusType| object.status.test(s.as_i32()) != FALSE;
        let hidden = status(ObjectStatusType::Stealthed)
            && !status(ObjectStatusType::Detected)
            && !status(ObjectStatusType::Disguised);
        // Aircraft make bad team targets; members pick them individually.
        !hidden
            && object.is_effectively_dead() == FALSE
            && object.contained_by.is_none()
            && sim.object_is_kind_of(slot, KindOfType::Aircraft) == FALSE
    });
    (target, target.is_none())
}

fn current_team_target(sim: &mut dyn SimApi, me: ObjectSlot) -> Option<ObjectSlot> {
    let (target, stale) = team_target(sim, me);
    if stale {
        sim.set_team_common_target(me, ObjectId::INVALID);
    }
    target
}

/// What `AIAttackFireWeaponState::onEnter` does for a team that attacks a common
/// target: a team without one takes `victim`. Only a computer player that is not
/// easy keeps it.
pub fn offer_team_target(sim: &mut dyn SimApi, me: ObjectSlot, victim: ObjectSlot) {
    if sim.team_common_target(me).is_none() || current_team_target(sim, me).is_some() {
        return;
    }
    if sim.team_keeps_common_target(me) == FALSE {
        return;
    }
    if let Some(id) = sim.object(victim).map(|o| o.id) {
        sim.set_team_common_target(me, id);
    }
}

/// A team forgets a `victim` that may no longer be attacked.
pub fn drop_team_target(sim: &mut dyn SimApi, me: ObjectSlot, victim: ObjectSlot) {
    if current_team_target(sim, me) == Some(victim) {
        sim.set_team_common_target(me, ObjectId::INVALID);
    }
}

/// The targeting questions answered by the sim.
#[derive(Default)]
struct SimTargeting {
    /// The team target was asked and is to be forgotten; the question is
    /// read-only, so the caller does it.
    stale_team_target: bool,
}

impl TargetingSeams for SimTargeting {
    fn view_blocked_by_obstacle(
        &mut self,
        sim: &dyn SimView,
        me: ObjectSlot,
        other: ObjectSlot,
    ) -> Bool {
        Bool::from(super::attack_path::is_view_blocked_by_obstacle(sim, me, other))
    }

    fn is_clear_of_fog(&mut self, sim: &dyn SimView, other: ObjectSlot, player_index: Int) -> Bool {
        sim.object_is_clear_of_shroud(other, player_index)
    }

    fn team_target_object(&mut self, sim: &dyn SimView, me: ObjectSlot) -> Option<ObjectSlot> {
        let (target, stale) = team_target(sim, me);
        self.stale_team_target |= stale;
        target
    }
}

/// `getNextMoodTarget`: whom the unit of `ctx` would attack unordered.
pub fn next_mood_target(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    called_by_ai: Bool,
    called_during_idle: Bool,
) -> Option<ObjectId> {
    if let Some(answer) =
        ai_seams(sim).next_mood_target(ctx.owner_id, called_by_ai, called_during_idle)
    {
        return answer;
    }
    let owner = ctx.owner_slot;
    let state = ai(sim, ctx);
    let unit = mood_unit(state);
    let mut timer = MoodState {
        next_mood_check_time: state.next_mood_check_time,
        randomly_offset_mood_check: state.randomly_offset_mood_check,
    };
    let mut seams = SimTargeting::default();
    let checked = mood::mood_check(
        sim,
        &mut seams,
        owner,
        &unit,
        &mut timer,
        called_by_ai,
        called_during_idle,
    );
    if seams.stale_team_target {
        sim.set_team_common_target(owner, ObjectId::INVALID);
    }
    let state = ai(sim, ctx);
    state.next_mood_check_time = timer.next_mood_check_time;
    state.randomly_offset_mood_check = timer.randomly_offset_mood_check;
    let found = match checked {
        Err(answer) => answer,
        Ok(()) => {
            let ai_data = &sim.rules().ai_data;
            mood::mood_search(sim, &mut seams, ai_data, owner, &unit, called_by_ai)
        }
    };
    found.and_then(|slot| sim.object(slot).map(|o| o.id))
}
