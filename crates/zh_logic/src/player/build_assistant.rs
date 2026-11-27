//! `PKT-403` (partial): `buildObjectNow` of `BuildAssistant.cpp`, without a
//! constructor object (with the `Player::onStructureConstructionComplete` it ends
//! in) and with a dozer or worker, where it ends in
//! `DozerAIInterface::construct`.
//!
//! Not ported: building along a line, placement legality
//! (`isLocationLegalToBuild`: a recorded order passed it on the machine that
//! issued it), a constructor that is not a dozer, selling.
//!
//! Golden tests: `crates/zh_logic/tests/golden_ai_new_map.rs` and
//! `golden_game_start.rs`, recorded from the running reference engine;
//! `orders_construct.rs` for the dozer's order.

use std::collections::BTreeMap;

use zh_common::math::geometry::{GeometryInfo, GeometryType};
use zh_common::math::{gm_cosf, gm_sinf, gm_sqrt, Coord3D, Region3D};
use zh_common::types::{Int, ObjectId, Real, Relationship, TeamId, UnsignedInt, FALSE, TRUE};

use crate::arena::{ObjectSlot, ThingTemplateId};
use crate::gamelogic::global_data::PlayerType;
use crate::modules::module::{CanMakeType, CommandSourceType};
use crate::modules::update::dozer_ai_update::{new_task, DOZER_TASK_BUILD};
use crate::modules::update::ai_update::{AiCommandParms, AiCommandType};
use crate::player::player_list::PlayerList;
use crate::player::seams::PlayerIndex;
use crate::rules::kind_of::KindOfType;
use crate::rules::model_state::{ModelConditionFlagType, ModelConditionFlags};
use crate::rules::object_status_types::{ObjectStatusMaskType, ObjectStatusType};
use crate::sim::{SimApi, SimContext, SimView};
use crate::terrain::terrain_logic::TerrainShape;

/// The engine's `PI`.
pub const PI: Real = std::f32::consts::PI;

/// What map-start building works on besides the sim.
pub struct BuildContext<'a> {
    pub sim: &'a mut SimContext,
    pub players: &'a mut PlayerList,
    /// Objects given a name (`Object::setName`).
    pub named: &'a mut Vec<(String, ObjectId)>,
    /// Calls into code that is not ported, by name.
    pub unported: &'a mut BTreeMap<&'static str, u32>,
    /// `TheTerrainLogic->getMaximumPathfindExtent`.
    pub pathfind_extent: Region3D,
}

impl BuildContext<'_> {
    pub fn unported(&mut self, call: &'static str) {
        *self.unported.entry(call).or_insert(0) += 1;
    }
}

/// `isRemovableForConstruction`.
pub fn is_removable_for_construction(sim: &SimContext, slot: ObjectSlot) -> bool {
    let kind = |k| sim.object_is_kind_of(slot, k) != FALSE;
    if kind(KindOfType::Inert) {
        return false;
    }
    kind(KindOfType::Shrubbery)
        || kind(KindOfType::ClearedByBuild)
        || sim
            .world
            .objects
            .get(slot)
            .is_some_and(|o| o.is_effectively_dead() != FALSE)
}

/// `clearRemovableForConstruction`; the terrain visual part is client only.
fn clear_removable_for_construction(
    sim: &mut SimContext,
    geometry: &GeometryInfo,
    pos: &Coord3D,
    angle: Real,
) {
    for them in sim.find_potential_collisions(pos, geometry, angle, FALSE) {
        if is_removable_for_construction(sim, them)
            && sim.object_is_kind_of(them, KindOfType::AlwaysSelectable) == FALSE
        {
            sim.destroy_object_now(them);
        }
    }
}

/// The box `moveObjectsForConstruction` searches for a building shaped `what`, and
/// the distance it moves units by before its random factor.
pub fn move_away_box(what: &GeometryInfo) -> (GeometryInfo, Real) {
    let gi = if what.get_geom_type() == GeometryType::Box {
        what.clone()
    } else {
        GeometryInfo::new(
            GeometryType::Box,
            FALSE,
            10.0,
            what.get_major_radius(),
            what.get_major_radius(),
        )
    };
    // `WWMath::Pow(r, 2)` is `gm_powf`, which returns `r * r` for an exponent of 2,
    // widened to double; the sum then takes the double `WWMath::Sqrt`.
    let major = gi.get_major_radius();
    let minor = gi.get_minor_radius();
    let sum = f64::from(major * major) + f64::from(minor * minor);
    let mut radius = gm_sqrt(sum) as Real;
    radius *= 1.4; // determinism-ok: `radius *= 1.4f`
    (gi, radius)
}

/// Where a unit in the way is told to go: `radius` scaled by `scale` in direction
/// `dir` from `pos` (`Vector3::Rotate_Z`).
pub fn move_away_destination(pos: &Coord3D, radius: Real, scale: Real, dir: Real) -> Coord3D {
    let varied = scale * radius;
    let (x, y, z): (Real, Real, Real) = (varied, 0.0, 0.0);
    let s = gm_sinf(dir);
    let c = gm_cosf(dir);
    Coord3D {
        x: pos.x + (c * x - s * y),
        y: pos.y + (s * x + c * y),
        z: pos.z + z,
    }
}

/// `moveObjectsForConstruction`: tells the units in the way to move. `false` if
/// something there cannot be told. Each unit moved draws two logic random numbers.
/// `relationship` is the building player's to a team and its player.
fn move_objects_for_construction(
    sim: &mut SimContext,
    unported: &mut BTreeMap<&'static str, u32>,
    geometry: &GeometryInfo,
    pos: &Coord3D,
    angle: Real,
    relationship: impl Fn(&SimContext, TeamId, Option<PlayerIndex>) -> Option<Relationship>,
) -> bool {
    let (gi, radius) = move_away_box(geometry);
    let found = sim.find_potential_collisions(pos, &gi, angle, FALSE);
    let mut any_unmovable = false;
    for them in found {
        let kind = |k| sim.object_is_kind_of(them, k) != FALSE;
        if kind(KindOfType::Mine) || kind(KindOfType::Inert) {
            continue;
        }
        if kind(KindOfType::AlwaysSelectable) || is_removable_for_construction(sim, them) {
            continue;
        }
        let Some((team, has_ai)) = sim
            .world
            .objects
            .get(them)
            .map(|o| (o.team_id, o.ai_slot.is_some()))
        else {
            continue;
        };
        let owner = sim.controlling_player(team);
        let friendly = matches!(
            relationship(sim, team, owner),
            Some(Relationship::Neutral | Relationship::Allies)
        );
        if !friendly || !has_ai {
            any_unmovable = true;
            continue;
        }
        let scale = sim.rng_logic_real(0.5, 1.5);
        let dir = sim.rng_logic_real(-PI, PI);
        let dest = move_away_destination(pos, radius, scale, dir);
        let mut parms = AiCommandParms::new(
            AiCommandType::MoveToPositionEvenIfSleeping,
            CommandSourceType::FromAi,
        );
        parms.pos = dest;
        if !sim.deliver_ai_command(them, parms) {
            *unported
                .entry("aiMoveToPositionEvenIfSleeping without resident AI")
                .or_insert(0) += 1;
        }
    }
    !any_unmovable
}

/// `buildObjectNow` without a constructor object.
pub fn build_object_now(
    cx: &mut BuildContext<'_>,
    what: ThingTemplateId,
    pos: &Coord3D,
    angle: Real,
    player: PlayerIndex,
) -> Option<ObjectSlot> {
    let (geometry, structure) = {
        let template = cx
            .sim
            .rules
            .thing_factory
            .find_by_template_id_final(what.0 as u16)?;
        (
            template.get_template_geometry_info().clone(),
            template.is_kind_of(KindOfType::Structure) != FALSE,
        )
    };
    let (team, human) = cx
        .players
        .player(player)
        .and_then(|p| Some((p.default_team?, p.player_type == PlayerType::Human)))?;

    clear_removable_for_construction(cx.sim, &geometry, pos, angle);
    // The computer gets to build even if something would not move.
    let players = &*cx.players;
    let relationship = |_: &SimContext, team, owner| {
        players
            .player(player)
            .map(|p| p.relationship(Some(team), owner))
    };
    let moved = move_objects_for_construction(
        cx.sim,
        cx.unported,
        &geometry,
        pos,
        angle,
        relationship,
    );
    if !moved && human {
        return None;
    }

    let mut status = ObjectStatusMaskType::new();
    if structure {
        status.set(ObjectStatusType::UnderConstruction.as_i32(), 1);
    }
    let slot = cx.sim.factory_new_object_with_status(what, team, status);
    let mut ground = *pos;
    ground.z = cx.sim.terrain.get_ground_height(ground.x, ground.y, None);
    cx.sim.set_object_position(slot, &ground);
    cx.sim.set_object_orientation(slot, angle);
    cx.sim.add_object_to_pathfind_map(slot);
    if structure {
        on_structure_construction_complete(cx, slot, player, false);
    } else {
        cx.unported("Player::onUnitCreated in buildObjectNow");
    }
    cx.sim.dispatch_on_build_complete(slot);
    cx.sim.handle_partition_cell_maintenance(slot);
    Some(slot)
}

/// `buildObjectNow` with a constructor object, as `onDozerConstruct` calls it:
/// the builder must offer the building, units in the way are told to move, and a
/// dozer or worker stops and starts it. `None` when nothing was built.
pub fn build_object_with_dozer(
    sim: &mut SimContext,
    unported: &mut BTreeMap<&'static str, u32>,
    dozer: ObjectSlot,
    what: ThingTemplateId,
    pos: &Coord3D,
    angle: Real,
) -> Option<ObjectSlot> {
    let (geometry, line_build) = {
        let template = sim
            .rules
            .thing_factory
            .find_by_template_id_final(what.0 as u16)?;
        (
            template.get_template_geometry_info().clone(),
            template.is_kind_of(KindOfType::LineBuild) != FALSE,
        )
    };
    let player = sim.controlling_player(sim.world.objects.get(dozer)?.team_id)?;
    if !sim.is_possible_to_make_unit(dozer, what, None) {
        return None;
    }
    clear_removable_for_construction(sim, &geometry, pos, angle);
    let team = *sim.world.player_default_teams.get(&player)?;
    let human = sim.player_is_human(team) != FALSE;
    // `Player::getRelationship`; the sim keeps the player entries.
    let relationship = |sim: &SimContext, _, owner: Option<PlayerIndex>| {
        let entry = owner.and_then(|o| sim.world.player_relationships.get(&(player, o)));
        Some(entry.copied().unwrap_or(Relationship::Neutral))
    };
    let moved = move_objects_for_construction(sim, unported, &geometry, pos, angle, relationship);
    if !moved && human {
        return None;
    }
    if sim.object_is_kind_of(dozer, KindOfType::Dozer) == FALSE || line_build {
        *unported
            .entry("buildObjectNow by a builder that is not a dozer")
            .or_insert(0) += 1;
        return None;
    }
    sim.world.objects.get(dozer)?.ai_slot?;
    let idle = AiCommandParms::new(AiCommandType::Idle, CommandSourceType::FromAi);
    if !sim.deliver_ai_command(dozer, idle) {
        *unported.entry("aiIdle without resident AI").or_insert(0) += 1;
        return None;
    }
    dozer_construct(sim, dozer, what, pos, angle, team)
}

/// `DozerAIUpdate::construct` and `WorkerAIUpdate::construct` of a new building
/// (not a rebuild). The worker leaves its supply truck state in `new_task`.
fn dozer_construct(
    sim: &mut SimContext,
    dozer: ObjectSlot,
    what: ThingTemplateId,
    pos: &Coord3D,
    angle: Real,
    team: TeamId,
) -> Option<ObjectSlot> {
    let human = sim.player_is_human(team) != FALSE;
    // The computer's worker skips the check; its orders do not come this way.
    if human && sim.production_can_make_unit(dozer, what, None) != CanMakeType::Ok {
        return None;
    }
    let module = sim.world.objects.get(dozer)?.ai_slot?;
    let ctx = sim.world.modules.meta(module).context(module);
    let dozer_id = sim.world.objects.get(dozer)?.id;

    let mut status = ObjectStatusMaskType::new();
    status.set(ObjectStatusType::UnderConstruction.as_i32(), 1);
    let slot = sim.factory_new_object_with_status(what, team, status);
    {
        let object = sim.world.objects.get_mut(slot)?;
        object.producer_id = dozer_id;
        object.builder_id = dozer_id;
    }
    let cost = sim.unit_cost_to_build(team, what);
    sim.withdraw_player_money(team, cost as UnsignedInt);
    sim.set_object_position(slot, pos);
    sim.set_object_orientation(slot, angle);
    let shape = {
        let object = sim.world.objects.get(slot)?;
        TerrainShape {
            position: *object.thing.get_position(),
            angle: object.thing.get_orientation(),
            geom_type: object.geometry.get_geom_type(),
            is_small: object.geometry.get_is_small(),
            major_radius: object.geometry.get_major_radius(),
            minor_radius: object.geometry.get_minor_radius(),
        }
    };
    if let Some(terrain) = sim.terrain.terrain_logic_mut() {
        terrain.flatten_terrain(&shape);
    }
    let mut adjusted = *pos;
    adjusted.z = sim.terrain.get_ground_height(pos.x, pos.y, None);
    sim.set_object_position(slot, &adjusted);
    sim.add_object_to_pathfind_map(slot);
    // `Player::onStructureCreated` is empty.
    sim.set_object_construction_percent(slot, 0.0);
    let health = sim.object_health(slot);
    sim.body_internal_change_health(slot, -health + 1.0);
    let mut clear = ModelConditionFlags::new();
    clear.set(ModelConditionFlagType::PartiallyConstructed as Int, 1);
    clear.set(ModelConditionFlagType::ActivelyBeingConstructed as Int, 1);
    let mut set = ModelConditionFlags::new();
    set.set(ModelConditionFlagType::AwaitingConstruction as Int, 1);
    sim.clear_and_set_model_condition_flags(slot, &clear, &set);
    new_task(sim, ctx, DOZER_TASK_BUILD, slot);
    Some(slot)
}

/// `Player::onStructureConstructionComplete` (`onStructureCreated` is empty). The
/// academy, control bar and Eva calls are left to the client.
pub fn on_structure_construction_complete(
    cx: &mut BuildContext<'_>,
    structure: ObjectSlot,
    player: PlayerIndex,
    is_rebuild: bool,
) {
    cx.sim.world.object_count_changed_frame = cx.sim.world.frame;
    cx.sim.remove_object_from_pathfind_map(structure);
    cx.sim.add_object_to_pathfind_map(structure);
    if !is_rebuild {
        cx.sim.record_structure_built(structure);
    }
    cx.sim.friend_adjust_power_for_player(structure, TRUE);
    if cx.players.player(player).is_some_and(|p| p.ai.is_some()) {
        crate::ai::ai_player::on_structure_produced(cx, player, structure);
    }
}
