//! `PKT-403` (partial): `buildObjectNow` of `BuildAssistant.cpp`, without a
//! constructor object (with the `Player::onStructureConstructionComplete` it ends
//! in) and with a dozer or worker, where it ends in
//! `DozerAIInterface::construct`; and placement legality
//! (`isLocationLegalToBuild`).
//!
//! Not ported: building along a line, a constructor that is not a dozer, selling.
//! A recorded human dozer order is not checked for legality: it passed on the
//! machine that issued it.
//!
//! Golden tests: `crates/zh_logic/tests/golden_ai_new_map.rs` and
//! `golden_game_start.rs`, recorded from the running reference engine;
//! `orders_construct.rs` for the dozer's order.

use std::collections::BTreeMap;

use zh_common::math::geometry::{GeometryInfo, GeometryType};
use zh_common::math::{
    gm_cosf, gm_floorf, gm_sinf, gm_sqrt, gm_sqrtf, Coord3D, Matrix3D, Region3D,
};
use zh_common::types::{Int, ObjectId, Real, Relationship, TeamId, UnsignedInt, FALSE, TRUE};

use crate::arena::{ObjectSlot, ThingTemplateId};
use crate::gamelogic::game_engine::StartupStores;
use crate::gamelogic::global_data::PlayerType;
use crate::ai::ai_state_machine::state_id;
use crate::ai::seams::MapTerrain;
use crate::modules::module::{CanMakeType, CommandSourceType};
use crate::modules::update::ai_update::{AiCommandParms, AiCommandType, AiUpdate, DerivedAi};
use crate::modules::update::dozer_ai_update::{new_task, DOZER_TASK_BUILD};
use crate::pathfinder::pathfind_cell::CellType;
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
use crate::player::seams::PlayerIndex;
use crate::player::team::TeamFactory;
use crate::rules::kind_of::KindOfType;
use crate::rules::model_state::{ModelConditionFlagType, ModelConditionFlags};
use crate::rules::object_status_types::{ObjectStatusMaskType, ObjectStatusType};
use crate::scripts::GameScripts;
use crate::sim::{SimApi, SimContext, SimView};
use crate::spatial::partition_collide::{geom_collides_with_geom, CollideInfo};
use crate::spatial::partition_query::{DistanceCalculationType, QueryCenter};
use crate::spatial::partition_shroud::CellShroudStatus;
use crate::terrain::terrain_logic::TerrainShape;

/// `MAP_XY_FACTOR` and `PATHFIND_CELL_SIZE_F`.
const MAP_XY_FACTOR: Real = 10.0;
const PATHFIND_CELL_SIZE_F: Real = 10.0;

/// The engine's `PI`.
pub const PI: Real = std::f32::consts::PI;

/// What map-start building works on besides the sim.
pub struct BuildContext<'a> {
    pub sim: &'a mut SimContext,
    /// Objects given a name (`Object::setName`).
    pub named: &'a mut Vec<(String, ObjectId)>,
    /// Calls into code that is not ported, by name.
    pub unported: &'a mut BTreeMap<&'static str, u32>,
    /// `TheTerrainLogic->getMaximumPathfindExtent`.
    pub pathfind_extent: Region3D,
    pub teams: Option<&'a mut TeamFactory>,
    pub scripts: Option<&'a mut GameScripts>,
    pub stores: Option<&'a StartupStores>,
    pub waypoints: &'a [(String, Coord3D)],
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
        .sim
        .world
        .players
        .player(player)
        .and_then(|p| Some((p.default_team?, p.player_type == PlayerType::Human)))?;

    clear_removable_for_construction(cx.sim, &geometry, pos, angle);
    // The computer gets to build even if something would not move.
    let relationship = |sim: &SimContext, team, owner| {
        sim.world
            .players
            .player(player)
            .map(|p| p.relationship(Some(team), owner))
    };
    let moved =
        move_objects_for_construction(cx.sim, cx.unported, &geometry, pos, angle, relationship);
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
    // `m_isRebuild` is set before anything is checked.
    let module = sim.world.objects.get(dozer)?.ai_slot?;
    if let Some(ai) = sim
        .resident_state_mut(module)
        .and_then(|state| state.downcast_mut::<AiUpdate>())
    {
        match &mut ai.derived {
            DerivedAi::Dozer(d) => d.brain.is_rebuild = FALSE,
            DerivedAi::Worker(w) => w.brain.is_rebuild = FALSE,
            DerivedAi::Base => return None,
        }
    }
    let human = sim.player_is_human(team) != FALSE;
    // The computer's worker skips the check; its orders do not come this way.
    if human && sim.production_can_make_unit(dozer, what, None) != CanMakeType::Ok {
        return None;
    }
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
    if cx.sim.world.players.player(player).is_some_and(|p| p.ai.is_some()) {
        crate::ai::ai_player::on_structure_produced(cx, player, structure);
    }
}

/// `BuildAssistant::LocalLegalToBuildOptions`.
pub mod legal_options {
    use zh_common::types::UnsignedInt;

    pub const TERRAIN_RESTRICTIONS: UnsignedInt = 0x01;
    pub const CLEAR_PATH: UnsignedInt = 0x02;
    pub const NO_OBJECT_OVERLAP: UnsignedInt = 0x04;
    pub const USE_QUICK_PATHFIND: UnsignedInt = 0x08;
    pub const SHROUD_REVEALED: UnsignedInt = 0x10;
    pub const NO_ENEMY_OBJECT_OVERLAP: UnsignedInt = 0x20;
    pub const IGNORE_STEALTHED: UnsignedInt = 0x40;
    pub const FAIL_STEALTHED_WITHOUT_FEEDBACK: UnsignedInt = 0x80;
}

/// `LegalBuildCode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegalBuildCode {
    Ok,
    RestrictedTerrain,
    NotFlatEnough,
    ObjectsInTheWay,
    NoClearPath,
    Shroud,
    TooCloseToSupplies,
    GenericFailure,
}

/// What `isLocationLegalToBuild` is asked about.
#[derive(Debug, Clone, Copy)]
pub struct PlacementQuery<'a> {
    pub pos: &'a Coord3D,
    pub what: ThingTemplateId,
    pub angle: Real,
    pub options: UnsignedInt,
    pub builder: Option<ObjectSlot>,
    /// `Player::isSkirmishAIPlayer` of the player passed in, if any.
    pub skirmish_ai: bool,
    /// `TheTerrainLogic->getMaximumPathfindExtent`.
    pub pathfind_extent: &'a Region3D,
}

/// What the queries read of the template to be built.
struct Planned {
    geometry: GeometryInfo,
    factory_exit_width: Real,
    factory_extra_bib_width: Real,
    near_supplies_forbidden: bool,
}

/// `getShroudedStatus(player) >= OBJECTSHROUD_FOGGED`: no cell of the object is
/// clear. The client's memory of fogged objects is not kept.
fn is_fogged_or_shrouded(sim: &SimContext, slot: ObjectSlot, player: PlayerIndex) -> bool {
    if sim.object_is_kind_of(slot, KindOfType::AlwaysVisible) != FALSE {
        return false;
    }
    let Some(id) = sim.world.objects.id_of(slot) else {
        return false;
    };
    let partition = &sim.world.partition;
    // No `PartitionData` is clear; before the first update the status is invalid.
    if !partition.is_registered(id) || !partition.updated_since_last_reset() {
        return false;
    }
    !partition.object_cells(id).iter().any(|&cell| {
        partition
            .shroud_cell(cell)
            .is_some_and(|c| c.status(player as usize) == CellShroudStatus::Clear)
    })
}

fn builder_player(sim: &SimContext, builder: Option<ObjectSlot>) -> Option<PlayerIndex> {
    let object = sim.world.objects.get(builder?)?;
    sim.controlling_player(object.team_id)
}

fn resident_ai(sim: &SimContext, slot: ObjectSlot) -> Option<&AiUpdate> {
    let module = sim.world.objects.get(slot)?.ai_slot?;
    sim.world
        .modules
        .payload(module)?
        .as_any()?
        .downcast_ref::<AiUpdate>()
}

/// `getAI() && getAI()->isBusy()`.
fn ai_is_busy(sim: &SimContext, slot: ObjectSlot) -> bool {
    resident_ai(sim, slot).is_some_and(|ai| ai.machine.current_state_id() == state_id::BUSY)
}

fn is_undetected_stealth(sim: &SimContext, slot: ObjectSlot) -> bool {
    sim.world.objects.get(slot).is_some_and(|o| {
        let has = |s: ObjectStatusType| o.status.test(s.as_i32()) != FALSE;
        has(ObjectStatusType::Stealthed)
            && !has(ObjectStatusType::Detected)
            && !has(ObjectStatusType::Disguised)
    })
}

/// `BuildAssistant::isLocationClearOfObjects`. The client's faction bibs are left
/// out.
fn is_location_clear_of_objects(
    sim: &SimContext,
    query: &PlacementQuery<'_>,
    build: &Planned,
) -> LegalBuildCode {
    use legal_options::*;
    let PlacementQuery {
        pos,
        angle,
        options,
        builder,
        ..
    } = *query;
    let only_check_enemies = options == NO_ENEMY_OBJECT_OVERLAP;
    let kind = |slot, k| sim.object_is_kind_of(slot, k) != FALSE;
    for them in sim.find_potential_collisions(pos, &build.geometry, angle, FALSE) {
        let mut feedback = true;
        let rel = builder.map_or(Relationship::Neutral, |b| sim.player_relationship(b, them));
        if options & IGNORE_STEALTHED != 0
            && rel != Relationship::Allies
            && is_undetected_stealth(sim, them)
        {
            if options & FAIL_STEALTHED_WITHOUT_FEEDBACK != 0 {
                feedback = false;
            } else {
                continue;
            }
        }
        if is_removable_for_construction(sim, them)
            || kind(them, KindOfType::Mine)
            || kind(them, KindOfType::Inert)
        {
            continue;
        }
        let immobile = kind(them, KindOfType::Immobile);
        if immobile && only_check_enemies && builder.is_some() && rel != Relationship::Enemies {
            continue;
        }
        if !cfg!(feature = "retail-crc")
            && builder_player(sim, builder)
                .is_some_and(|player| is_fogged_or_shrouded(sim, them, player))
        {
            return LegalBuildCode::Shroud;
        }
        let object = sim.world.objects.get(them);
        let using_ability = object.is_some_and(|o| {
            o.status.test(ObjectStatusType::IsUsingAbility.as_i32()) != FALSE
        });
        if rel == Relationship::Allies && (using_ability || ai_is_busy(sim, them)) {
            return LegalBuildCode::ObjectsInTheWay;
        }
        let disabled = object.is_some_and(|o| o.is_disabled() != FALSE);
        if immobile || disabled || (builder.is_some() && rel == Relationship::Enemies) {
            return if feedback {
                LegalBuildCode::ObjectsInTheWay
            } else {
                LegalBuildCode::GenericFailure
            };
        }
    }
    if only_check_enemies {
        return LegalBuildCode::Ok;
    }
    exit_areas_clear(sim, query, build)
}

/// Where a factory exit rectangle of width `width` lies: past the front edge of a
/// building of major radius `major`.
fn exit_area(
    pos: &Coord3D,
    angle: Real,
    major: Real,
    width: Real,
    geometry: GeometryInfo,
) -> Option<CollideInfo> {
    (width > 0.0).then(|| {
        let mut at = *pos;
        let c = gm_cosf(angle);
        let s = gm_sinf(angle);
        let offset = major + width / 2.0;
        at.x += c * offset;
        at.y += s * offset;
        CollideInfo {
            pos: at,
            geometry,
            angle,
        }
    })
}

/// The footprint widened by the extra bib width. A box widens its minor radius by
/// `minor_extra`, which for the other building is also the new building's width.
fn bib_bounds(geometry: &GeometryInfo, extra: Real, minor_extra: Real) -> GeometryInfo {
    let mut bounds = geometry.clone();
    bounds.set_major_radius(bounds.get_major_radius() + extra);
    if bounds.get_geom_type() != GeometryType::Box {
        let major = bounds.get_major_radius();
        bounds.set(GeometryType::Box, FALSE, 40.0, major, major);
    } else {
        bounds.set_minor_radius(bounds.get_minor_radius() + minor_extra);
    }
    bounds
}

/// The second half of `isLocationClearOfObjects`: no structure nearby may overlap
/// the building, its factory exit or theirs.
fn exit_areas_clear(
    sim: &SimContext,
    query: &PlacementQuery<'_>,
    build: &Planned,
) -> LegalBuildCode {
    use legal_options::IGNORE_STEALTHED;
    let PlacementQuery {
        pos,
        angle,
        options,
        builder,
        skirmish_ai,
        ..
    } = *query;
    let range = 2.0 * (build.geometry.get_major_radius() + build.geometry.get_minor_radius());
    let mut my_exit_width = build.factory_exit_width;
    let mut my_extra_width = build.factory_extra_bib_width;
    // The skirmish AI keeps a gap so it does not build itself into a corner.
    if skirmish_ai && my_extra_width < 3.0 * PATHFIND_CELL_SIZE_F {
        my_extra_width = 3.0 * PATHFIND_CELL_SIZE_F;
        my_exit_width -= my_extra_width;
        if my_exit_width < 0.0 {
            my_exit_width = 0.0;
        }
    }
    let mine = CollideInfo {
        pos: *pos,
        geometry: bib_bounds(&build.geometry, my_extra_width, my_extra_width),
        angle,
    };
    let mut my_exit_geometry = build.geometry.clone();
    if my_exit_geometry.get_geom_type() != GeometryType::Box {
        my_exit_geometry.set_minor_radius(my_exit_geometry.get_major_radius());
    }
    my_exit_geometry.set_major_radius(my_exit_width / 2.0);
    let my_exit = exit_area(
        pos,
        angle,
        build.geometry.get_major_radius(),
        my_exit_width,
        my_exit_geometry,
    );

    let nearby = sim.search_partition(
        QueryCenter::Position(*pos),
        range,
        DistanceCalculationType::FromBoundingSphere2d,
    );
    for them in nearby {
        if sim.object_is_kind_of(them, KindOfType::Structure) == FALSE {
            continue;
        }
        if options & IGNORE_STEALTHED != 0
            && builder.map_or(Relationship::Neutral, |b| sim.player_relationship(b, them))
                != Relationship::Allies
            && is_undetected_stealth(sim, them)
        {
            continue;
        }
        if is_removable_for_construction(sim, them) {
            continue;
        }
        let Some(object) = sim.world.objects.get(them) else {
            continue;
        };
        let (his_exit_width, his_extra_width) = object
            .thing
            .template_override()
            .get_final(&sim.rules.thing_factory)
            .map_or((0.0, 0.0), |t| {
                (t.get_factory_exit_width(), t.get_factory_extra_bib_width())
            });
        let his_pos = *object.thing.get_position();
        let his_angle = object.thing.get_orientation();
        let his = CollideInfo {
            pos: his_pos,
            geometry: bib_bounds(&object.geometry, his_extra_width, my_extra_width),
            angle: his_angle,
        };
        let mut his_exit_geometry = object.geometry.clone();
        his_exit_geometry.set_major_radius(his_exit_width / 2.0);
        if his_exit_geometry.get_geom_type() != GeometryType::Box {
            his_exit_geometry.set_minor_radius(object.geometry.get_major_radius());
        }
        let his_exit = exit_area(
            &his_pos,
            his_angle,
            object.geometry.get_major_radius(),
            his_exit_width,
            his_exit_geometry,
        );
        if geom_collides_with_geom(&his, &mine) {
            return LegalBuildCode::ObjectsInTheWay;
        }
        if my_exit.is_none()
            && his_exit.is_none()
            && his_extra_width == 0.0
            && my_extra_width == 0.0
        {
            continue;
        }
        if sim.object_is_kind_of(them, KindOfType::Immobile) == FALSE {
            continue;
        }
        // My exit against him, his exit against me, and the two exits.
        let overlaps = my_exit
            .as_ref()
            .is_some_and(|exit| geom_collides_with_geom(&his, exit))
            || his_exit
                .as_ref()
                .is_some_and(|exit| geom_collides_with_geom(exit, &mine))
            || my_exit
                .as_ref()
                .zip(his_exit.as_ref())
                .is_some_and(|(my, theirs)| geom_collides_with_geom(theirs, my));
        if overlaps {
            let shrouded = builder_player(sim, builder)
                .is_some_and(|player| is_fogged_or_shrouded(sim, them, player));
            return if shrouded {
                LegalBuildCode::Shroud
            } else {
                LegalBuildCode::ObjectsInTheWay
            };
        }
    }
    LegalBuildCode::Ok
}

/// The `SampleBuildData` of the footprint samples.
struct Footprint {
    restricted: bool,
    hi_z: Real,
    lo_z: Real,
    map: Region3D,
}

impl Footprint {
    /// `checkSampleBuildLocation`. Terrain types flagged `RestrictConstruction`
    /// are not looked up: the shipped data has none.
    fn sample(&mut self, sim: &SimContext, at: &Coord3D) {
        let cell_x = gm_floorf(at.x / PATHFIND_CELL_SIZE_F) as Int;
        let cell_y = gm_floorf(at.y / PATHFIND_CELL_SIZE_F) as Int;
        let cell = sim
            .world
            .pathfinder
            .as_deref()
            .and_then(|p| p.cell_type(PathfindLayerEnum::Ground, cell_x, cell_y));
        if matches!(
            cell,
            None | Some(CellType::Water | CellType::Cliff | CellType::Impassable)
        ) {
            self.restricted = true;
        }
        if at.z < self.lo_z {
            self.lo_z = at.z;
        }
        if at.z > self.hi_z {
            self.hi_z = at.z;
        }
        let border = sim.rules.global_data.min_dist_from_edge_of_map_for_build;
        if border > 0.0
            && (at.x < self.map.lo.x + border
                || at.x > self.map.hi.x - border
                || at.y < self.map.lo.y + border
                || at.y > self.map.hi.y - border)
        {
            self.restricted = true;
        }
    }
}

/// `BuildAssistant::iterateFootprint` with `checkSampleBuildLocation`.
fn sample_footprint(
    sim: &SimContext,
    geometry: &GeometryInfo,
    angle: Real,
    pos: &Coord3D,
    resolution: Real,
    data: &mut Footprint,
) {
    let mut transform = Matrix3D::identity();
    transform.rows[0][3] += pos.x;
    transform.rows[1][3] += pos.y;
    transform.rows[2][3] += pos.z;
    transform.rotate_z(angle);

    let circular = geometry.get_geom_type() != GeometryType::Box;
    let (half_height, half_width) = if circular {
        let radius = geometry.get_bounding_circle_radius();
        (radius, radius)
    } else {
        (geometry.get_minor_radius(), geometry.get_major_radius())
    };
    let mut y = -half_height;
    while y < half_height + resolution {
        if y > half_height {
            y = half_height;
        }
        let mut x = -half_width;
        while x < half_width + resolution {
            if x > half_width {
                x = half_width;
            }
            let local = Coord3D::new(x, y, sim.terrain.get_ground_height(x, y, None));
            let v = Matrix3D::transform_vector(&transform, &local);
            // A circle takes only the points within it.
            let outside = circular && {
                let dx = v.x - pos.x;
                let dy = v.y - pos.y;
                gm_sqrtf(dx * dx + dy * dy) > half_width
            };
            if !outside {
                let z = sim.terrain.get_ground_height(v.x, v.y, None);
                data.sample(sim, &Coord3D::new(v.x, v.y, z));
            }
            x += resolution;
        }
        y += resolution;
    }
}

/// `AIUpdateInterface::isQuickPathAvailable`; no without an AI.
pub(crate) fn quick_path_available(sim: &SimContext, slot: ObjectSlot, to: &Coord3D) -> bool {
    let (Some(ai), Some(object)) = (resident_ai(sim, slot), sim.world.objects.get(slot)) else {
        return false;
    };
    let surfaces = ai.locomotor_set.get_valid_surfaces();
    sim.world.pathfinder.as_deref().is_some_and(|p| {
        p.quick_path_exists(sim.terrain(), surfaces, object.thing.get_position(), to)
    })
}

/// `BuildAssistant::isLocationLegalToBuild`.
pub fn is_location_legal_to_build(sim: &SimContext, query: &PlacementQuery<'_>) -> LegalBuildCode {
    use legal_options::*;
    let PlacementQuery {
        pos,
        angle,
        options,
        builder,
        pathfind_extent: extent,
        ..
    } = *query;
    // Never off the map, whatever the options.
    if !(extent.lo.x < pos.x && pos.x < extent.hi.x && extent.lo.y < pos.y && pos.y < extent.hi.y) {
        return LegalBuildCode::RestrictedTerrain;
    }
    if options & SHROUD_REVEALED != 0 {
        let partition = &sim.world.partition;
        let (x, y) = partition.world_to_cell(pos.x, pos.y);
        let player = builder_player(sim, builder).unwrap_or(-1);
        if partition.shroud_status(player, x, y) != CellShroudStatus::Clear {
            return LegalBuildCode::Shroud;
        }
    }
    let Some(build) = sim
        .rules
        .thing_factory
        .find_by_template_id_final(query.what.0 as u16)
        .map(|t| Planned {
            geometry: t.get_template_geometry_info().clone(),
            factory_exit_width: t.get_factory_exit_width(),
            factory_extra_bib_width: t.get_factory_extra_bib_width(),
            near_supplies_forbidden: t.is_kind_of(KindOfType::CannotBuildNearSupplies) != FALSE,
        })
    else {
        return LegalBuildCode::GenericFailure;
    };
    // Both overlap options run the same check with all the options.
    for overlap in [NO_OBJECT_OVERLAP, NO_ENEMY_OBJECT_OVERLAP] {
        if options & overlap != 0 {
            let code = is_location_clear_of_objects(sim, query, &build);
            if code != LegalBuildCode::Ok {
                return code;
            }
        }
    }
    let border = sim.rules.global_data.supply_build_border;
    if build.near_supplies_forbidden && border > 0.0 {
        let range = build.geometry.get_bounding_circle_radius() + border * 2.0;
        let supply = sim.closest_object_to_position(
            pos,
            range,
            DistanceCalculationType::FromBoundingSphere2d,
            &mut |s| sim.object_is_kind_of(s, KindOfType::SupplySource) != FALSE,
        );
        if let Some(object) = supply.and_then(|s| sim.world.objects.get(s)) {
            let mut expanded = object.geometry.clone();
            expanded.expand_footprint(border);
            let mine = CollideInfo {
                pos: *pos,
                geometry: build.geometry.clone(),
                angle,
            };
            let theirs = CollideInfo {
                pos: *object.thing.get_position(),
                geometry: expanded,
                angle: object.thing.get_orientation(),
            };
            if geom_collides_with_geom(&mine, &theirs) {
                return LegalBuildCode::TooCloseToSupplies;
            }
        }
    }
    if options & CLEAR_PATH != 0 {
        let walker = builder.filter(|&b| sim.object_is_kind_of(b, KindOfType::Immobile) == FALSE);
        if walker.is_some_and(|b| !quick_path_available(sim, b, pos)) {
            return LegalBuildCode::NoClearPath;
        }
    }
    if options & TERRAIN_RESTRICTIONS != 0 {
        if sim.terrain().get_layer_for_destination(pos) != PathfindLayerEnum::Ground {
            return LegalBuildCode::RestrictedTerrain;
        }
        let map = sim
            .terrain
            .terrain_logic()
            .map_or_else(Region3D::default, |t| t.get_extent());
        let mut data = Footprint {
            restricted: false,
            hi_z: map.lo.z,
            lo_z: map.hi.z,
            map,
        };
        let allowed = sim.rules.global_data.allowed_height_variation_for_building;
        // A coarse pass at three times the resolution, then a full one.
        for step in [3.0 * MAP_XY_FACTOR, MAP_XY_FACTOR] {
            sample_footprint(sim, &build.geometry, angle, pos, step, &mut data);
            if data.restricted {
                return LegalBuildCode::RestrictedTerrain;
            }
            if data.hi_z - data.lo_z > allowed {
                return LegalBuildCode::NotFlatEnough;
            }
        }
    }
    LegalBuildCode::Ok
}
