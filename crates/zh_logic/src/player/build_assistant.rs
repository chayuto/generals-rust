//! `PKT-403` (partial): the part of `BuildAssistant.cpp` that raises a building out
//! of thin air (`buildObjectNow` without a constructor object), and the
//! `Player::onStructureConstructionComplete` it ends in.
//!
//! Not ported: building with a dozer or along a line, placement legality
//! (`isLocationLegalToBuild`), selling.
//!
//! Golden tests: `crates/zh_logic/tests/golden_ai_new_map.rs` and
//! `golden_game_start.rs`, recorded from the running reference engine.

use std::collections::BTreeMap;

use zh_common::math::geometry::{GeometryInfo, GeometryType};
use zh_common::math::{gm_cosf, gm_sinf, gm_sqrt, Coord3D, Region3D};
use zh_common::types::{ObjectId, Real, Relationship, FALSE, TRUE};

use crate::arena::{ObjectSlot, ThingTemplateId};
use crate::gamelogic::global_data::PlayerType;
use crate::modules::module::CommandSourceType;
use crate::modules::update::ai_update::{AiCommandParms, AiCommandType};
use crate::player::player_list::PlayerList;
use crate::player::seams::PlayerIndex;
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::{ObjectStatusMaskType, ObjectStatusType};
use crate::sim::{SimApi, SimContext, SimView};

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
fn move_objects_for_construction(
    cx: &mut BuildContext<'_>,
    geometry: &GeometryInfo,
    pos: &Coord3D,
    angle: Real,
    player: PlayerIndex,
) -> bool {
    let (gi, radius) = move_away_box(geometry);
    let found = cx.sim.find_potential_collisions(pos, &gi, angle, FALSE);
    let mut any_unmovable = false;
    for them in found {
        let kind = |k| cx.sim.object_is_kind_of(them, k) != FALSE;
        if kind(KindOfType::Mine) || kind(KindOfType::Inert) {
            continue;
        }
        if kind(KindOfType::AlwaysSelectable) || is_removable_for_construction(cx.sim, them) {
            continue;
        }
        let Some((team, has_ai)) = cx
            .sim
            .world
            .objects
            .get(them)
            .map(|o| (o.team_id, o.ai_slot.is_some()))
        else {
            continue;
        };
        let owner = cx.sim.controlling_player(team);
        let relationship = cx
            .players
            .player(player)
            .map(|p| p.relationship(Some(team), owner));
        let friendly = matches!(
            relationship,
            Some(Relationship::Neutral | Relationship::Allies)
        );
        if !friendly || !has_ai {
            any_unmovable = true;
            continue;
        }
        let scale = cx.sim.rng_logic_real(0.5, 1.5);
        let dir = cx.sim.rng_logic_real(-PI, PI);
        let dest = move_away_destination(pos, radius, scale, dir);
        let mut parms = AiCommandParms::new(
            AiCommandType::MoveToPositionEvenIfSleeping,
            CommandSourceType::FromAi,
        );
        parms.pos = dest;
        if !cx.sim.deliver_ai_command(them, parms) {
            cx.unported("aiMoveToPositionEvenIfSleeping without resident AI");
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
    if !move_objects_for_construction(cx, &geometry, pos, angle, player) && human {
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
