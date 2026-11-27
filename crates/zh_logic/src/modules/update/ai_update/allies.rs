//! `PKT-301`, asking allies out of the way (AI plan slice S9): the sim's side of
//! `Pathfinder::moveAllies` and `privateMoveAwayFromUnit`.
//!
//! The search for the way aside, `getMoveAwayFromPath`, is not ported (S6) and
//! is asked through [`crate::ai::seams::AiSeams`].

use super::{ai, AiCall, AiCommandParms, AiCommandType};
use crate::ai::ai_state_machine::{self as machine, state_id};
use crate::ai::seams::ai_seams;
use crate::arena::{ModuleContext, ObjectSlot};
use crate::modules::module::CommandSourceType;
use crate::pathfinder::{AllyMover, AllyUnit, MoveAlliesWorld, Path, PathAgent, Pathfinder};
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::SimApi;
use zh_common::ini::LOGICFRAMES_PER_SECOND;
use zh_common::math::{Coord3D, Matrix3D};
use zh_common::types::{Bool, Int, ObjectId, Real, Relationship, FALSE, TRUE};

/// `isMovingAwayFrom`.
pub(super) fn is_moving_away_from(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    other: ObjectId,
) -> bool {
    let state = ai(sim, ctx);
    state.machine.temporary == state_id::MOVE_OUT_OF_THE_WAY
        && (state.move_out_of_way1 == other || state.move_out_of_way2 == other)
}

/// `findObjectByID(id)->getAI()` of a unit with a resident AI module.
pub(super) fn ai_of(sim: &dyn SimApi, id: ObjectId) -> Option<ModuleContext> {
    sim.find_object(id).and_then(|slot| sim.ai_context(slot))
}

/// `aiMoveAwayFromUnit(unit, CMD_FROM_AI)` of the AI at `ctx`.
pub(super) fn order_move_away(sim: &mut dyn SimApi, ctx: ModuleContext, unit: ObjectId) {
    let mut parms = AiCommandParms::new(AiCommandType::MoveAwayFromUnit, CommandSourceType::FromAi);
    parms.obj = Some(unit);
    super::execute(sim, ctx, &AiCall::Command(parms));
}

/// The units `moveAllies` looks up and orders about.
struct Allies<'a> {
    sim: &'a mut dyn SimApi,
}

impl MoveAlliesWorld for Allies<'_> {
    fn ally(&mut self, mover: ObjectId, other: ObjectId) -> Option<AllyUnit> {
        let sim = &mut *self.sim;
        let slot = sim.find_object(other)?;
        let ally = sim
            .find_object(mover)
            .is_some_and(|m| sim.player_relationship(m, slot) == Relationship::Allies);
        let infantry = sim.object_is_kind_of(slot, KindOfType::Infantry) != FALSE;
        let using_ability = has_status(sim, slot, ObjectStatusType::IsUsingAbility);
        let ctx = sim.ai_context(slot);
        let (moving, attacking, busy) = ctx.map_or((false, false, false), |ctx| {
            let state = ai(sim, ctx);
            (
                state.is_moving() != FALSE,
                state.is_attacking_state(),
                state.machine.is_in_busy_state() != FALSE,
            )
        });
        Some(AllyUnit {
            ally,
            infantry,
            has_ai: ctx.is_some(),
            moving,
            attacking,
            using_ability,
            busy,
        })
    }

    fn move_away_from_unit(
        &mut self,
        pathfinder: &mut Pathfinder,
        other: ObjectId,
        mover: ObjectId,
    ) {
        let Some(ctx) = ai_of(self.sim, other) else {
            return;
        };
        // The order searches and changes the map: the sim gets it back.
        self.sim
            .put_pathfinder(Box::new(std::mem::take(pathfinder)));
        order_move_away(self.sim, ctx, mover);
        if let Some(returned) = self.sim.take_pathfinder() {
            *pathfinder = *returned;
        }
    }
}

pub(super) fn has_status(sim: &dyn SimApi, slot: ObjectSlot, status: ObjectStatusType) -> bool {
    sim.object(slot)
        .is_some_and(|o| o.status.test(status as Int) != FALSE)
}

/// `Pathfinder::moveAllies` along the unit's own path.
pub(super) fn move_allies(sim: &mut dyn SimApi, ctx: ModuleContext) {
    let owner = ctx.owner_slot;
    let Some(path) = ai(sim, ctx).path.clone() else {
        return;
    };
    move_allies_along(sim, owner, &path);
}

fn move_allies_along(sim: &mut dyn SimApi, owner: ObjectSlot, path: &Path) {
    let Some(view) = sim.ai_path_view(owner) else {
        return;
    };
    let mover = AllyMover {
        agent: view.agent,
        dozer_or_harvester: sim.object_is_kind_of(owner, KindOfType::Dozer) != FALSE
            || sim.object_is_kind_of(owner, KindOfType::Harvester) != FALSE,
    };
    super::with_pathfinder(sim, |sim, pathfinder| {
        pathfinder.move_allies(&mut Allies { sim }, &mover, path);
    });
}

/// A terrain that answers the one question `moveAlliesAwayFromDestination`
/// asks, with the answer taken before the sim is lent out for the orders.
struct DestinationLayer(crate::pathfinder::pathfind_constants::PathfindLayerEnum);

impl crate::object::thing::ThingTerrainAccess for DestinationLayer {
    fn get_ground_height(&self, _x: Real, _y: Real, _normal: Option<&mut Coord3D>) -> Real {
        0.0
    }
    fn align_on_terrain(
        &self,
        _angle: Real,
        _pos: &Coord3D,
        _stick_to_ground: Bool,
        _mtx: &mut Matrix3D,
    ) -> crate::pathfinder::pathfind_constants::PathfindLayerEnum {
        self.0
    }
    fn is_underwater(
        &self,
        _x: Real,
        _y: Real,
        _water_z: Option<&mut Real>,
        _terrain_z: Option<&mut Real>,
    ) -> Bool {
        FALSE
    }
    fn get_layer_for_destination(
        &self,
        _pos: &Coord3D,
    ) -> crate::pathfinder::pathfind_constants::PathfindLayerEnum {
        self.0
    }
}

/// `Pathfinder::moveAlliesAwayFromDestination`.
pub(super) fn move_allies_away_from_destination(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    destination: &Coord3D,
) {
    let owner = ctx.owner_slot;
    let Some(view) = sim.ai_path_view(owner) else {
        return;
    };
    let agent: PathAgent = view.agent;
    let layer = DestinationLayer(sim.terrain().get_layer_for_destination(destination));
    super::with_pathfinder(sim, |sim, pathfinder| {
        pathfinder.move_allies_away_from_destination(
            &layer,
            &mut Allies { sim },
            &agent,
            &view.position,
            destination,
        );
    });
}

/// `privateMoveAwayFromUnit`: step off `unit`'s path.
pub(super) fn private_move_away_from_unit(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    unit: Option<ObjectId>,
) {
    let owner = ctx.owner_slot;
    if ai(sim, ctx).is_ai_dead != FALSE || !super::is_mobile(sim, owner) {
        return;
    }
    let Some(unit) = unit else {
        return;
    };
    let now = sim.frame();
    if ai(sim, ctx).machine.temporary == state_id::MOVE_OUT_OF_THE_WAY {
        let state = ai(sim, ctx);
        if state.move_out_of_way1 == unit || state.move_out_of_way2 == unit {
            if state.is_blocked != FALSE {
                state.ignore_collisions_until = now.wrapping_add(2 * LOGICFRAMES_PER_SECOND as u32);
            }
            return;
        }
    }
    let state = ai(sim, ctx);
    state.move_out_of_way2 = state.move_out_of_way1;
    state.move_out_of_way1 = unit;
    let second = state.move_out_of_way2;
    let second_path = ai_of(sim, second).and_then(|c| ai(sim, c).path.clone());
    let second = sim.find_object(second).map(|_| second);
    let Some(unit_path) = ai_of(sim, unit).and_then(|c| ai(sim, c).path.clone()) else {
        return;
    };
    let search = |sim: &mut dyn SimApi| {
        ai_seams(sim).move_away_path(ctx.owner_id, unit, &unit_path, second, second_path.as_ref())
    };
    let mut new_path = search(sim);
    if new_path.is_none() && ai(sim, ctx).can_path_through_units == FALSE {
        ai(sim, ctx).set_can_path_through_units(TRUE);
        new_path = search(sim);
    }
    let Some(new_path) = new_path else {
        return;
    };
    ai(sim, ctx).destroy_path();
    super::set_path(sim, ctx, Some(new_path));
    super::wake_up_now(sim, ctx);
    machine::set_temporary_state(
        sim,
        ctx,
        state_id::MOVE_OUT_OF_THE_WAY,
        10 * LOGICFRAMES_PER_SECOND as Int,
    );
    if ai(sim, ctx).path.is_some() && sim.object_is_kind_of(owner, KindOfType::NoCollide) == FALSE {
        move_allies(sim, ctx);
    }
}
