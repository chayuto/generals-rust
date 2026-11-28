//! `PKT-426`: `AIPlayer`: the constructor's state, `newMap`,
//! `buildStructureNow`, `xfer`, `update` (base building timers, `findDozer`,
//! `isLocationSafe`, `buildStructureWithDozer`, non-skirmish `processBaseBuilding`,
//! `updateBridgeRepair`, skill-set choice), `buildBySupplies`, `findSupplyCenter`,
//! `repairStructure`, `selectSkillset`, `buildUpgrade`,
//! `calcClosestConstructionZoneLocation`, `buildSpecificBuildingNearestTeam`,
//! `isSupplySourceAttacked`, `isSupplySourceSafe`, `guardSupplyCenter`,
//! `getPlayerStructureBounds`, `getPlayerSuperweaponValue`, and
//! `computeSuperweaponTarget`.
//!
//! Not ported, each counted in `BuildContext::unported` where it is reached:
//! `queueDozer` and science purchase. Not ported and not reached: team
//! building queues (`doTeamBuilding`, `checkReadyTeams`, `checkQueuedTeams`,
//! `queueUnits`, `selectTeamToBuild`, `startTraining`), `crc` and
//! `loadPostProcess`.
//!
//! Golden tests: `crates/zh_logic/tests/golden_ai_new_map.rs`
//! (`docs/golden_vectors/AiNewMap.json`, recorded from the running reference engine
//! by `tests/cpp_harness/gen_ai_new_map.py`), `golden_game_start.rs` and, for the
//! update, `golden_game_frames.rs`.

use zh_common::math::{gm_sqrtf, Coord2D, Coord3D, Region3D};
use zh_common::types::{
    Bool, Int, ObjectId, Real, Relationship, UnsignedInt, XferVersion, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError, XferMode};

use crate::ai::ai_skirmish_player::{self, AiSkirmishPlayer};
use crate::ai::ai_state_machine::state_id;
pub use crate::ai::ai_team_queue::{on_unit_produced_in_sim, TeamInQueue, WorkOrder};
use crate::ai::ai_team_queue;
use crate::arena::{FacetMask, ObjectSlot, ThingTemplateId};
use crate::gamelogic::game_engine::StartupStores;
use crate::modules::update::ai_update::{AiUpdate, DerivedAi};
use crate::modules::update::supply_warehouse_dock_update::SupplyWarehouseDockUpdate;
use crate::object::thing_factory::ThingFactory;
use crate::player::team::TeamFactory;
use crate::scripts::ScriptGame;
use crate::sim::PRIVATE_STATUS_OFF_MAP;
use crate::player::player::Player;
use crate::modules::update::dozer_ai_update::{is_task_pending, DOZER_TASK_BUILD};
use crate::object::object_script_status_bits::OBJECT_STATUS_SCRIPT_UNSELLABLE;
use crate::player::build_assistant::{
    build_object_now, build_object_with_dozer, is_location_legal_to_build, legal_options,
    quick_path_available, BuildContext, LegalBuildCode, PlacementQuery,
};
use crate::player::seams::{AiKind, PlayerIndex};
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimContext, SimView};
use crate::spatial::partition_query::DistanceCalculationType;
use crate::spatial::sides_list::BuildListInfo;

/// `MAX_STRUCTURES_TO_REPAIR`.
pub const MAX_STRUCTURES_TO_REPAIR: usize = 2;
/// `INVALID_SKILLSET_SELECTION`.
pub const INVALID_SKILLSET_SELECTION: Int = -1;
/// `DIFFICULTY_EASY`, `DIFFICULTY_NORMAL`, `DIFFICULTY_HARD`.
pub const DIFFICULTY_EASY: Int = 0;
pub const DIFFICULTY_NORMAL: Int = 1;
pub const DIFFICULTY_HARD: Int = 2;

/// The state of one computer player (`Player::m_ai`). The constructor leaves
/// some members to the zeroed memory pool block.
#[derive(Debug, Clone, PartialEq)]
pub struct AiPlayer {
    pub player: PlayerIndex,
    pub ready_to_build_team: Bool,
    pub ready_to_build_structure: Bool,
    pub team_timer: Int,
    pub structure_timer: Int,
    pub build_delay: Int,
    pub team_delay: Int,
    pub team_seconds: Int,
    pub cur_warehouse_id: ObjectId,
    pub frame_last_building_built: Int,
    pub difficulty: Int,
    pub skillset_selector: Int,
    pub base_center: Coord3D,
    pub base_center_set: Bool,
    pub base_radius: Real,
    pub structures_to_repair: [ObjectId; MAX_STRUCTURES_TO_REPAIR],
    pub repair_dozer: ObjectId,
    pub structures_in_queue: Int,
    pub dozer_queued_for_repair: Bool,
    pub dozer_is_repairing: Bool,
    pub bridge_timer: Int,
    pub team_build_queue: Vec<TeamInQueue>,
    pub team_ready_queue: Vec<TeamInQueue>,
    pub repair_dozer_origin: Coord3D,
    pub supply_source_attack_check_frame: UnsignedInt,
    pub attacked_supply_center: ObjectId,
    /// The members `AISkirmishPlayer` adds; `None` for a plain `AIPlayer`.
    pub skirmish: Option<AiSkirmishPlayer>,
}

impl AiPlayer {
    /// The constructor. `difficulty` is the script engine's global difficulty and
    /// `team_seconds` is `AIData::m_teamSeconds`.
    pub fn new(
        kind: AiKind,
        player: PlayerIndex,
        frame: Int,
        difficulty: Int,
        team_seconds: Int,
    ) -> Self {
        Self {
            player,
            ready_to_build_team: FALSE,
            ready_to_build_structure: FALSE,
            team_timer: 2,
            structure_timer: 2,
            build_delay: 0,
            team_delay: 0,
            team_seconds,
            cur_warehouse_id: ObjectId::INVALID,
            frame_last_building_built: frame,
            difficulty,
            skillset_selector: INVALID_SKILLSET_SELECTION,
            base_center: Coord3D::default(),
            base_center_set: FALSE,
            base_radius: 0.0,
            structures_to_repair: [ObjectId::INVALID; MAX_STRUCTURES_TO_REPAIR],
            repair_dozer: ObjectId::INVALID,
            structures_in_queue: 0,
            dozer_queued_for_repair: FALSE,
            dozer_is_repairing: FALSE,
            bridge_timer: 0,
            team_build_queue: Vec::new(),
            team_ready_queue: Vec::new(),
            repair_dozer_origin: Coord3D::default(),
            supply_source_attack_check_frame: 0,
            attacked_supply_center: ObjectId::INVALID,
            skirmish: (kind == AiKind::Skirmish).then(AiSkirmishPlayer::default),
        }
    }

    /// `AIPlayer::crc` / `AISkirmishPlayer::crc` (`AIPlayer.cpp:3333-3336`).
    pub fn crc(&self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        if let Some(skirmish) = &self.skirmish {
            skirmish.crc(xfer)?;
        }
        Ok(())
    }

    /// `AIPlayer::clearTeamsInQueue` (`AIPlayer.cpp:444-448`).
    pub fn clear_teams_in_queue(&mut self, teams: &mut TeamFactory) {
        for item in self
            .team_build_queue
            .drain(..)
            .chain(self.team_ready_queue.drain(..))
        {
            if item.team_id != zh_common::types::TeamId::INVALID {
                if let Some(team) = teams.team_mut(item.team_id) {
                    team.set_active();
                }
            }
        }
    }

    /// `AIPlayer::xfer` or `AISkirmishPlayer::xfer`, without factory/team resolution.
    pub fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        self.xfer_with(xfer, None, None)
    }

    /// `AIPlayer::xfer` or `AISkirmishPlayer::xfer`, resolving template and team ids.
    pub fn xfer_with(
        &mut self,
        xfer: &mut dyn Xfer,
        thing_factory: Option<&ThingFactory>,
        teams: Option<&TeamFactory>,
    ) -> Result<(), XferError> {
        match self.skirmish.take() {
            None => self.xfer_base(xfer, thing_factory, teams),
            Some(mut skirmish) => {
                let mut version: XferVersion = 1;
                let result = xfer
                    .xfer_version(&mut version, 1)
                    .and_then(|()| self.xfer_base(xfer, thing_factory, teams))
                    .and_then(|()| skirmish.xfer_members(xfer));
                self.skirmish = Some(skirmish);
                result
            }
        }
    }

    fn xfer_base(
        &mut self,
        xfer: &mut dyn Xfer,
        thing_factory: Option<&ThingFactory>,
        teams: Option<&TeamFactory>,
    ) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        // Team build queue (head-to-tail order matching C++ prepend + reverse on load).
        let mut build_count = self.team_build_queue.len() as zh_common::types::UnsignedShort;
        xfer.xfer_unsigned_short(&mut build_count)?;
        if xfer.mode() == XferMode::Save {
            for team in &mut self.team_build_queue {
                team.xfer(xfer, thing_factory, teams)?;
            }
        } else {
            if !self.team_build_queue.is_empty() {
                return Err(XferError::InvalidData);
            }
            for _ in 0..build_count {
                let mut team = TeamInQueue::new();
                team.xfer(xfer, thing_factory, teams)?;
                self.team_build_queue.push(team);
            }
        }
        // Team ready queue (head-to-tail order matching C++ prepend + reverse on load).
        let mut ready_count = self.team_ready_queue.len() as zh_common::types::UnsignedShort;
        xfer.xfer_unsigned_short(&mut ready_count)?;
        if xfer.mode() == XferMode::Save {
            for team in &mut self.team_ready_queue {
                team.xfer(xfer, thing_factory, teams)?;
            }
        } else {
            if !self.team_ready_queue.is_empty() {
                return Err(XferError::InvalidData);
            }
            for _ in 0..ready_count {
                let mut team = TeamInQueue::new();
                team.xfer(xfer, thing_factory, teams)?;
                self.team_ready_queue.push(team);
            }
        }
        let mut player = self.player;
        xfer.xfer_int(&mut player)?;
        if player != self.player {
            return Err(XferError::InvalidData);
        }
        xfer_flag(xfer, &mut self.ready_to_build_team)?;
        xfer_flag(xfer, &mut self.ready_to_build_structure)?;
        xfer.xfer_int(&mut self.team_timer)?;
        xfer.xfer_int(&mut self.structure_timer)?;
        xfer.xfer_int(&mut self.build_delay)?;
        xfer.xfer_int(&mut self.team_delay)?;
        xfer.xfer_int(&mut self.team_seconds)?;
        xfer.xfer_object_id(&mut self.cur_warehouse_id)?;
        xfer.xfer_int(&mut self.frame_last_building_built)?;
        xfer.xfer_int(&mut self.difficulty)?;
        xfer.xfer_int(&mut self.skillset_selector)?;
        xfer.xfer_coord3d(&mut self.base_center)?;
        xfer_flag(xfer, &mut self.base_center_set)?;
        xfer.xfer_real(&mut self.base_radius)?;
        for id in &mut self.structures_to_repair {
            xfer.xfer_object_id(id)?;
        }
        xfer.xfer_object_id(&mut self.repair_dozer)?;
        xfer.xfer_int(&mut self.structures_in_queue)?;
        xfer_flag(xfer, &mut self.dozer_queued_for_repair)?;
        xfer_flag(xfer, &mut self.dozer_is_repairing)?;
        xfer.xfer_int(&mut self.bridge_timer)
    }
}

pub(super) fn xfer_flag(xfer: &mut dyn Xfer, value: &mut Bool) -> Result<(), XferError> {
    let mut byte = i8::from(*value != FALSE);
    xfer.xfer_byte(&mut byte)?;
    *value = if byte != 0 { TRUE } else { FALSE };
    Ok(())
}

/// `PlayerList::newMap`: every computer player's `newMap`, in player order.
pub fn players_new_map(cx: &mut BuildContext<'_>) {
    for player in 0..cx.players.player_count() {
        let Some(kind) = cx.players.player(player).and_then(|p| p.ai) else {
            continue;
        };
        match kind {
            AiKind::Standard => new_map(cx, player),
            AiKind::Skirmish => ai_skirmish_player::new_map(cx, player),
        }
    }
}

/// `AIPlayer::newMap`: lists the player's factories, then builds what the map's
/// build list says is there from the start.
pub fn new_map(cx: &mut BuildContext<'_>, player: PlayerIndex) {
    // The reference takes the head of the list before adding to its front, so the
    // added entries are not built.
    let owned: Vec<ObjectSlot> = cx
        .sim
        .world
        .objects
        .world_iter()
        .filter(|&slot| {
            let Some(obj) = cx.sim.world.objects.get(slot) else {
                return false;
            };
            cx.sim.controlling_player(obj.team_id) == Some(player)
                && cx.sim.production_module(slot).is_some()
        })
        .collect();
    let added = owned.len();
    for slot in owned {
        add_to_build_list(cx, player, slot);
    }
    compute_center_and_radius_of_base(cx, player);
    build_initial_structures(cx, player, added);
}

/// `Player::addToBuildList`.
fn add_to_build_list(cx: &mut BuildContext<'_>, player: PlayerIndex, slot: ObjectSlot) {
    let Some(obj) = cx.sim.world.objects.get(slot) else {
        return;
    };
    let mut info = BuildListInfo::new();
    info.object_id = obj.id;
    info.location = *obj.thing.get_position();
    info.angle = obj.thing.get_orientation();
    info.num_rebuilds = 0;
    info.template_name = obj
        .thing
        .template_override()
        .get_final(&cx.sim.rules.thing_factory)
        .map_or_else(String::new, |t| t.get_name().to_owned());
    if let Some(p) = cx.players.player_mut(player) {
        p.build_list.insert(0, info);
    }
}

/// The loop both `newMap`s end in, from entry `first` of the build list.
pub(super) fn build_initial_structures(
    cx: &mut BuildContext<'_>,
    player: PlayerIndex,
    first: usize,
) {
    let mut at = first;
    while let Some(info) = cx.players.player(player).and_then(|p| p.build_list.get(at)) {
        let initially_built = info.is_initially_built != FALSE;
        let known = !info.template_name.is_empty()
            && cx
                .sim
                .rules
                .thing_factory
                .find_template_id(&info.template_name)
                .is_some();
        if known {
            if initially_built {
                build_structure_now(cx, player, at);
            } else if let Some(p) = cx.players.player_mut(player) {
                p.build_list[at].increment_num_rebuilds();
            }
        }
        at += 1;
    }
}

/// `computeCenterAndRadiusOfBase` over the `(location, bounding circle radius)` of
/// every build list entry with a known template: the mean location, the distance to
/// the farthest edge, and whether there was any entry.
pub fn base_center_and_radius(buildings: &[(Coord3D, Real)]) -> (Coord2D, Real, Bool) {
    let mut total = Coord2D { x: 0.0, y: 0.0 };
    for (at, _) in buildings {
        total.x += at.x;
        total.y += at.y;
    }
    let count = buildings.len() as Int;
    if count > 0 {
        total.x /= count as Real;
        total.y /= count as Real;
    }
    let mut max_squared: Real = 0.0;
    for (at, circle) in buildings {
        let mut dx = at.x - total.x;
        let mut dy = at.y - total.y;
        if dx < 0.0 {
            dx = -dx;
        }
        if dy < 0.0 {
            dy = -dy;
        }
        let radius = circle * 0.4; // determinism-ok: `getBoundingCircleRadius()*0.4f`
        dx += radius;
        dy += radius;
        let squared = dx * dx + dy * dy;
        if squared > max_squared {
            max_squared = squared;
        }
    }
    let set = if count > 0 { TRUE } else { FALSE };
    (total, gm_sqrtf(max_squared), set)
}

/// `computeCenterAndRadiusOfBase( &m_baseCenter, &m_baseRadius )`.
pub(super) fn compute_center_and_radius_of_base(cx: &mut BuildContext<'_>, player: PlayerIndex) {
    let buildings: Vec<(Coord3D, Real)> = cx
        .players
        .player(player)
        .map_or(&[][..], |p| p.build_list.as_slice())
        .iter()
        .filter(|info| !info.template_name.is_empty())
        .filter_map(|info| {
            let template = cx
                .sim
                .rules
                .thing_factory
                .find_template_final(&info.template_name)?;
            Some((
                info.location,
                template
                    .get_template_geometry_info()
                    .get_bounding_circle_radius(),
            ))
        })
        .collect();
    let (center, radius, set) = base_center_and_radius(&buildings);
    if let Some(ai) = cx.sim.world.ai_players.get_mut(&player) {
        ai.base_center_set = set;
        ai.base_center.x = center.x;
        ai.base_center.y = center.y;
        ai.base_radius = radius;
    }
}

/// `buildStructureNow` of build list entry `at`. The rally point offset is read by
/// the reference and never applied (`gotOffset;` instead of an assignment).
fn build_structure_now(cx: &mut BuildContext<'_>, player: PlayerIndex, at: usize) {
    let Some(info) = cx
        .players
        .player(player)
        .and_then(|p| p.build_list.get(at))
        .cloned()
    else {
        return;
    };
    let Some(template) = cx
        .sim
        .rules
        .thing_factory
        .find_template_id(&info.template_name)
    else {
        return;
    };
    let Some(building) = build_object_now(cx, template, &info.location, info.angle, player) else {
        return;
    };
    let Some(id) = cx.sim.world.objects.id_of(building) else {
        return;
    };
    // `updateObjValuesFromMapProperties` with the entry's name, health and
    // sellability.
    if !info.building_name.is_empty() {
        cx.named.push((info.building_name.clone(), id));
    }
    #[cfg(feature = "retail-crc")]
    cx.sim.set_initial_health(building, info.health);
    if let Some(obj) = cx.sim.world.objects.get_mut(building) {
        if info.unsellable != FALSE {
            obj.script_status |= OBJECT_STATUS_SCRIPT_UNSELLABLE;
        } else {
            obj.script_status &= !OBJECT_STATUS_SCRIPT_UNSELLABLE;
        }
    }
    let frame = cx.sim.world.frame;
    if let Some(p) = cx.players.player_mut(player) {
        p.build_list[at].object_id = id;
        p.build_list[at].object_timestamp = frame.wrapping_add(1);
    }
    cx.sim
        .clear_status(building, ObjectStatusType::UnderConstruction);
    cx.sim
        .clear_status(building, ObjectStatusType::Reconstructing);
    cx.sim.update_upgrade_modules(building);
    if !info.script.is_empty() {
        cx.unported("ScriptEngine::runObjectScript");
    }
    check_for_supply_center(cx, player, at, building);
}

/// `checkForSupplyCenter`.
pub(super) fn check_for_supply_center(
    cx: &mut BuildContext<'_>,
    player: PlayerIndex,
    at: usize,
    building: ObjectSlot,
) {
    let rules = &cx.sim.rules;
    let docks_supplies = cx
        .sim
        .world
        .objects
        .get(building)
        .and_then(|o| o.thing.template_override().get_final(&rules.thing_factory))
        .is_some_and(|t| {
            t.get_behavior_module_info()
                .nuggets()
                .iter()
                .any(|n| n.name == "SupplyCenterDockUpdate")
        });
    if !docks_supplies {
        return;
    }
    let difficulty = cx
        .sim
        .world
        .ai_players
        .get(&player)
        .map_or(DIFFICULTY_NORMAL, |ai| ai.difficulty);
    let Some(p) = cx.players.player_mut(player) else {
        return;
    };
    let mut desired = 0;
    for side in &cx.sim.rules.ai_data.side_info {
        if side.side == p.side {
            match difficulty {
                DIFFICULTY_EASY => desired = side.easy,
                DIFFICULTY_NORMAL => desired = side.normal,
                DIFFICULTY_HARD => desired = side.hard,
                _ => {}
            }
        }
    }
    let info = &mut p.build_list[at];
    info.is_supply_building = TRUE;
    info.current_gatherers = -1;
    info.desired_gatherers = desired.wrapping_add(1);
}

/// `AIPlayer::onStructureProduced`.
pub fn on_structure_produced(cx: &mut BuildContext<'_>, player: PlayerIndex, building: ObjectSlot) {
    if let Some(ai) = cx.sim.world.ai_players.get_mut(&player) {
        ai.team_delay = 0;
        ai.build_delay = 0;
    }
    let Some(id) = cx.sim.world.objects.id_of(building) else {
        return;
    };
    let found_at = cx
        .players
        .player(player)
        .and_then(|p| p.build_list.iter().position(|info| info.object_id == id));
    if let Some(at) = found_at {
        let Some((building_name, script, health, unsellable)) = cx
            .players
            .player_mut(player)
            .and_then(|p| p.build_list.get_mut(at))
            .map(|info| {
                info.under_construction = FALSE;
                (
                    info.building_name.clone(),
                    info.script.clone(),
                    info.health,
                    info.unsellable,
                )
            })
        else {
            return;
        };
        if !building_name.is_empty() {
            cx.named.push((building_name.clone(), id));
            if let Some(scripts) = cx.scripts.as_deref_mut() {
                scripts.engine.add_object_to_cache(&building_name, id);
            }
        }
        #[cfg(feature = "retail-crc")]
        cx.sim.set_initial_health(building, health);
        #[cfg(not(feature = "retail-crc"))]
        let _ = health;
        if let Some(obj) = cx.sim.world.objects.get_mut(building) {
            if unsellable != FALSE {
                obj.script_status |= OBJECT_STATUS_SCRIPT_UNSELLABLE;
            } else {
                obj.script_status &= !OBJECT_STATUS_SCRIPT_UNSELLABLE;
            }
        }
        cx.sim
            .clear_status(building, ObjectStatusType::UnderConstruction);
        cx.sim
            .clear_status(building, ObjectStatusType::Reconstructing);
        cx.sim.update_upgrade_modules(building);
        if !script.is_empty() {
            if let (Some(scripts), Some(teams)) =
                (cx.scripts.as_deref_mut(), cx.teams.as_deref_mut())
            {
                let default_stores;
                let stores = match cx.stores {
                    Some(s) => s,
                    None => {
                        default_stores = StartupStores::default();
                        &default_stores
                    }
                };
                scripts.with_world(
                    cx.sim,
                    ScriptGame {
                        players: cx.players,
                        teams,
                        stores,
                        waypoints: cx.waypoints,
                        named_objects: cx.named,
                    },
                    |engine, world| engine.run_object_script(world, &script, Some(id)),
                );
            }
        }
        check_for_supply_center(cx, player, at, building);
        return;
    }

    // Check rebuild holes in the build list (`AIPlayer.cpp:151-178`).
    let len = cx.players.player(player).map_or(0, |p| p.build_list.len());
    for at in 0..len {
        let Some((tmpl_name, hole_id)) = cx
            .players
            .player(player)
            .and_then(|p| p.build_list.get(at))
            .map(|info| (info.template_name.clone(), info.object_id))
        else {
            continue;
        };
        if hole_id == ObjectId::INVALID {
            continue;
        }
        let equiv = {
            let tf = &cx.sim.rules.thing_factory;
            let bldg_plan = tf.find_template_final(&tmpl_name);
            let bldg_tmpl = cx
                .sim
                .world
                .objects
                .get(building)
                .and_then(|o| o.thing.template_override().get_final(tf));
            matches!((bldg_plan, bldg_tmpl), (Some(p), Some(b)) if p.is_equivalent_to(b) != FALSE)
        };
        if !equiv {
            continue;
        }
        let Some(hole_slot) = cx.sim.world.objects.slot_by_id(hole_id) else {
            continue;
        };
        if cx.sim.object_is_kind_of(hole_slot, KindOfType::RebuildHole) == FALSE {
            continue;
        }
        let spawned_match = cx
            .sim
            .world
            .objects
            .get(hole_slot)
            .is_some_and(|hole_obj| {
                hole_obj
                    .behaviors
                    .iter()
                    .filter(|(_, facets)| facets.contains(FacetMask::REBUILD_HOLE))
                    .any(|&(ms, _)| {
                        cx.sim
                            .world
                            .modules
                            .payload(ms)
                            .and_then(|m| m.as_rebuild_hole())
                            .is_some_and(|rh| rh.get_reconstructed_building_id() == id)
                    })
            });
        if spawned_match {
            if let Some(info) = cx
                .players
                .player_mut(player)
                .and_then(|p| p.build_list.get_mut(at))
            {
                info.object_id = id;
            }
            return;
        }
    }
}

// -----------------------------------------------------------------------------
// The frame update
// -----------------------------------------------------------------------------

/// `LOGICFRAMES_PER_SECOND` and `PATHFIND_CELL_SIZE_F`.
pub(super) const FRAMES_PER_SECOND: Int = 30;
pub(super) const PATHFIND_CELL_SIZE_F: Real = 10.0;

/// `Player::update`'s `m_ai->update()`: `AIPlayer::update`, with the overrides of
/// `AISkirmishPlayer`.
pub fn update(cx: &mut BuildContext<'_>, player: PlayerIndex) {
    if !cx.sim.world.ai_players.contains_key(&player) {
        return;
    }
    ai_team_queue::drain_pending_sim_events_for_player(cx, player);
    do_base_building(cx, player);
    ai_team_queue::check_ready_teams(cx, player);
    ai_team_queue::check_queued_teams(cx, player);
    ai_team_queue::do_team_building(cx, player);
    do_upgrades_and_skills(cx, player);
    update_bridge_repair(cx, player);
}

/// `doUpgradesAndSkills`: once the player has points to spend, a skirmish AI picks
/// one of its side's skill sets at random, and attempts to purchase each science from it.
fn do_upgrades_and_skills(cx: &mut BuildContext<'_>, player: PlayerIndex) {
    // Nothing in the first two frames.
    if cx.sim.world.frame < 2 {
        return;
    }
    let Some(p) = cx.players.player(player) else {
        return;
    };
    if p.science_purchase_points <= 0 {
        return;
    }
    let Some(side_idx) = cx
        .sim
        .rules
        .ai_data
        .side_info
        .iter()
        .position(|info| info.side == p.side)
    else {
        return;
    };
    let sizes: [usize; 5] =
        std::array::from_fn(|i| cx.sim.rules.ai_data.side_info[side_idx].skill_sets[i].len());
    let Some(ai) = cx.sim.world.ai_players.get(&player) else {
        return;
    };
    let mut selector = ai.skillset_selector;
    if selector == INVALID_SKILLSET_SELECTION {
        selector = if ai.skirmish.is_some() {
            // The sets after the first count while each of them has skills.
            let limit = sizes[1..].iter().take_while(|&&size| size > 0).count();
            cx.sim.world.rng.get_game_logic_random_value(0, limit as Int)
        } else {
            0
        };
        if let Some(ai) = ai_mut(cx, player) {
            ai.skillset_selector = selector;
        }
    }
    let set = usize::try_from(selector).ok().filter(|&i| i < 5).unwrap_or(0);
    let skills: Vec<_> = cx.sim.rules.ai_data.side_info[side_idx].skill_sets[set].clone();
    for science in skills {
        crate::player::science_purchase::attempt_to_purchase_science(
            cx.sim,
            cx.players,
            cx.scripts.as_deref_mut().map(|s| &mut s.engine),
            player,
            science,
        );
    }
}

/// The state of the player's AI. The callers have checked that there is one.
pub(super) fn ai_mut<'s>(
    cx: &'s mut BuildContext<'_>,
    player: PlayerIndex,
) -> Option<&'s mut AiPlayer> {
    cx.sim.world.ai_players.get_mut(&player)
}

/// `doBaseBuilding` of either class. The skirmish AI never waits longer than three
/// seconds to be ready.
fn do_base_building(cx: &mut BuildContext<'_>, player: PlayerIndex) {
    if !cx.players.player(player).is_some_and(|p| p.can_build_base) {
        return;
    }
    let Some(ai) = ai_mut(cx, player) else {
        return;
    };
    let skirmish = ai.skirmish.is_some();
    if ai.ready_to_build_structure == FALSE {
        ai.structure_timer = ai.structure_timer.wrapping_sub(1);
        if ai.structure_timer <= 0 {
            ai.ready_to_build_structure = TRUE;
            ai.build_delay = 0;
        }
        if skirmish && ai.structure_timer > 3 * FRAMES_PER_SECOND {
            ai.structure_timer = 3 * FRAMES_PER_SECOND;
        }
    }
    // A building or a unit finishing shortcuts the delay.
    ai.build_delay = ai.build_delay.wrapping_sub(1);
    if ai.build_delay >= 1 {
        return;
    }
    if ai.ready_to_build_structure != FALSE {
        if skirmish {
            ai_skirmish_player::process_base_building(cx, player);
        } else {
            crate::ai::ai_player_ops::process_base_building(cx, player);
        }
    }
    if let Some(ai) = ai_mut(cx, player) {
        if ai.build_delay < 1 {
            ai.build_delay = 2 * FRAMES_PER_SECOND;
        }
    }
}

/// What `findDozer` asks a dozer or worker.
struct DozerState {
    any_task_pending: bool,
    building: bool,
    /// A worker made to go for supplies.
    forced_to_gather: bool,
}

fn dozer_state(sim: &SimContext, slot: ObjectSlot) -> Option<DozerState> {
    let module = sim.world.objects.get(slot)?.ai_slot?;
    let ai = sim
        .world
        .modules
        .payload(module)?
        .as_any()?
        .downcast_ref::<AiUpdate>()?;
    let (brain, forced) = match &ai.derived {
        DerivedAi::Dozer(d) => (&d.brain, false),
        DerivedAi::Worker(w) => (
            &w.brain,
            ai.machine.current_state_id() == state_id::DOCK || w.force_pending != FALSE,
        ),
        DerivedAi::Base => return None,
    };
    Some(DozerState {
        any_task_pending: (0..brain.tasks.len() as Int).any(|t| is_task_pending(brain, t)),
        building: is_task_pending(brain, DOZER_TASK_BUILD),
        forced_to_gather: forced,
    })
}

pub(super) fn owner_of(sim: &SimContext, slot: ObjectSlot) -> Option<PlayerIndex> {
    sim.controlling_player(sim.world.objects.get(slot)?.team_id)
}

/// `findDozer`: the closest idle dozer to `pos`, else the first that is not
/// building. Queues a dozer if the player has none.
pub(super) fn find_dozer(
    cx: &mut BuildContext<'_>,
    player: PlayerIndex,
    pos: &Coord3D,
) -> Option<ObjectSlot> {
    let repair_dozer = cx.sim.world.ai_players.get(&player)?.repair_dozer;
    let mut dozer = None;
    let mut need_dozer = true;
    let mut closest: Option<(ObjectSlot, Real)> = None;
    for slot in cx.sim.world.objects.world_iter() {
        if owner_of(cx.sim, slot) != Some(player)
            || cx.sim.object_is_kind_of(slot, KindOfType::Dozer) == FALSE
        {
            continue;
        }
        let Some(state) = dozer_state(cx.sim, slot) else {
            continue;
        };
        if !state.any_task_pending && state.forced_to_gather {
            continue;
        }
        if cx.sim.world.objects.id_of(slot) == Some(repair_dozer) {
            continue;
        }
        need_dozer = false;
        if state.building {
            continue;
        }
        if !state.any_task_pending || dozer.is_none() {
            dozer = Some(slot);
        }
        if state.any_task_pending {
            continue;
        }
        let Some(at) = dozer
            .and_then(|d| cx.sim.world.objects.get(d))
            .map(|o| *o.thing.get_position())
        else {
            continue;
        };
        let dx = pos.x - at.x;
        let dy = pos.y - at.y;
        let squared = dx * dx + dy * dy;
        if closest.is_none_or(|(_, best)| squared < best) {
            closest = dozer.map(|d| (d, squared));
        }
    }
    if need_dozer {
        queue_dozer(cx, player);
    }
    closest.map(|c| c.0).or(dozer)
}

/// `queueDozer`.
pub(super) fn queue_dozer(cx: &mut BuildContext<'_>, player: PlayerIndex) {
    ai_team_queue::queue_dozer(cx, player);
}

/// `isLocationSafe`: no enemy of any account stands within the supply center safe
/// radius of where `template`'s footprint would reach.
pub(super) fn is_location_safe(
    sim: &SimContext,
    player: PlayerIndex,
    pos: &Coord3D,
    footprint_radius: Real,
) -> bool {
    let radius = sim.rules.ai_data.supply_center_safe_radius + footprint_radius;
    let mut allow = |slot| {
        let Some(other) = sim.world.objects.get(slot) else {
            return false;
        };
        let kind = |k| sim.object_is_kind_of(slot, k) != FALSE;
        // Enemies only, and not the player's own.
        let owner = sim.controlling_player(other.team_id);
        let relationship = owner.and_then(|o| sim.world.player_relationships.get(&(player, o)));
        if relationship != Some(&Relationship::Enemies) || owner == Some(player) {
            return false;
        }
        if other.is_effectively_dead() != FALSE {
            return false;
        }
        let has = |s: ObjectStatusType| other.status.test(s.as_i32()) != FALSE;
        if has(ObjectStatusType::Stealthed)
            && !has(ObjectStatusType::Detected)
            && !has(ObjectStatusType::Disguised)
        {
            return false;
        }
        // An empty or ungarrisonable civilian building is not significant.
        if kind(KindOfType::Structure)
            && !FACTION_STRUCTURE_KINDS.iter().any(|&k| kind(k))
            && other.contain_slot.is_some()
            && (sim.contain_is_garrisonable(slot) == FALSE || sim.contain_count(slot) == 0)
        {
            return false;
        }
        !kind(KindOfType::Harvester) && !kind(KindOfType::Dozer)
    };
    sim.closest_object_to_position(
        pos,
        radius,
        DistanceCalculationType::FromBoundingSphere2d,
        &mut allow,
    )
    .is_none()
}

/// `KINDOFMASK_FS`.
const FACTION_STRUCTURE_KINDS: [KindOfType; 14] = [
    KindOfType::FsFactory,
    KindOfType::FsBaseDefense,
    KindOfType::FsTechnology,
    KindOfType::FsSupplyDropzone,
    KindOfType::FsSuperweapon,
    KindOfType::FsBlackMarket,
    KindOfType::FsSupplyCenter,
    KindOfType::FsStrategyCenter,
    KindOfType::FsFake,
    KindOfType::FsInternetCenter,
    KindOfType::FsAdvancedTech,
    KindOfType::FsBarracks,
    KindOfType::FsWarfactory,
    KindOfType::FsAirfield,
];

/// `buildStructureWithDozer` of build list entry `at`: finds a legal place near the
/// listed one and gives a dozer the order. Returns the new building.
pub(super) fn build_structure_with_dozer(
    cx: &mut BuildContext<'_>,
    player: PlayerIndex,
    what: ThingTemplateId,
    at: usize,
) -> Option<ObjectId> {
    let info = cx.players.player(player)?.build_list.get(at)?.clone();
    let dozer = find_dozer(cx, player, &info.location)?;
    let team = cx.sim.world.objects.get(dozer)?.team_id;
    let default_team = *cx.sim.world.player_default_teams.get(&player)?;
    if cx.sim.player_money(default_team) < cx.sim.unit_cost_to_build(team, what) as UnsignedInt {
        return None;
    }
    let mut pos = info.location;
    pos.z += cx.sim.terrain.get_ground_height(pos.x, pos.y, None);
    let angle = info.angle;
    let skirmish = cx.sim.world.ai_players.get(&player)?.skirmish.is_some();
    let pos = find_place(cx, player, what, dozer, pos, angle, skirmish)?;

    // A dozer that cannot reach the site is put there.
    if !quick_path_available(cx.sim, dozer, &pos) {
        cx.sim.set_object_position(dozer, &pos);
    }
    let built = build_object_with_dozer(cx.sim, &mut *cx.unported, dozer, what, &pos, angle)?;
    // The natural rally point is asked for and never set.
    let id = cx.sim.world.objects.id_of(built)?;
    let frame = cx.sim.world.frame;
    let info = cx.players.player_mut(player)?.build_list.get_mut(at)?;
    info.object_id = id;
    info.object_timestamp = frame.wrapping_add(1);
    info.under_construction = TRUE;
    Some(id)
}

/// Where `buildStructureWithDozer` builds: the listed place, else the first legal
/// one of widening squares around it, else the listed place if only friends are in
/// the way.
fn find_place(
    cx: &BuildContext<'_>,
    player: PlayerIndex,
    what: ThingTemplateId,
    dozer: ObjectSlot,
    pos: Coord3D,
    angle: Real,
    skirmish: bool,
) -> Option<Coord3D> {
    use legal_options::*;
    let skirmish_player = cx
        .players
        .player(player)
        .is_some_and(|p| p.ai == Some(AiKind::Skirmish));
    let sim = &*cx.sim;
    let extent = cx.pathfind_extent;
    let legal = |at: &Coord3D, options| {
        is_location_legal_to_build(
            sim,
            &PlacementQuery {
                pos: at,
                what,
                angle,
                options,
                builder: Some(dozer),
                skirmish_ai: skirmish_player,
                pathfind_extent: &extent,
            },
        ) == LegalBuildCode::Ok
    };
    // Enemy units or structures there: do not build at all.
    if !legal(&pos, NO_ENEMY_OBJECT_OVERLAP) {
        return None;
    }
    let full = CLEAR_PATH | TERRAIN_RESTRICTIONS | NO_OBJECT_OVERLAP;
    if legal(&pos, full) {
        return Some(pos);
    }
    let cell = PATHFIND_CELL_SIZE_F;
    let limit = if skirmish { 120.0 * cell } else { 10.0 * cell };
    // The skirmish AI steps twice as far.
    let mut new_pos = pos;
    let mut pos_offset: Real = 0.0;
    while pos_offset < limit {
        if skirmish {
            pos_offset += 2.0 * cell;
        }
        let offset = pos_offset / 2.0;
        let mut valid = false;
        let mut y = pos.y - offset;
        let mut x = pos.x - offset;
        while x <= pos.x + offset {
            if skirmish {
                x += cell;
            }
            new_pos.x = x;
            new_pos.y = y;
            valid = legal(&new_pos, full);
            if valid {
                break;
            }
            new_pos.y = y + pos_offset;
            valid = legal(&new_pos, full);
            x += cell;
        }
        // The `if (valid) break` after the second test is missing in the
        // reference, so the row is finished and `valid` is the last test.
        if valid {
            return Some(new_pos);
        }
        x = pos.x - offset;
        y = pos.y - offset;
        while y <= pos.y + offset {
            if skirmish {
                y += cell;
            }
            new_pos.x = x;
            new_pos.y = y;
            valid = legal(&new_pos, full);
            if valid {
                break;
            }
            new_pos.x = x + pos_offset;
            valid = legal(&new_pos, full);
            y += cell;
        }
        if valid {
            return Some(new_pos);
        }
        pos_offset += 2.0 * cell;
    }
    legal(&pos, NO_ENEMY_OBJECT_OVERLAP).then_some(pos)
}

/// `AIPlayer::updateBridgeRepair`.
fn update_bridge_repair(cx: &mut BuildContext<'_>, player: PlayerIndex) {
    crate::ai::ai_player_ops::update_bridge_repair(cx, player);
}

// -----------------------------------------------------------------------------
// Orders from scripts
// -----------------------------------------------------------------------------

/// `SUPPLY_CENTER_CLOSE_DIST`.
pub(super) const SUPPLY_CENTER_CLOSE_DIST: Real = 20.0 * PATHFIND_CELL_SIZE_F;

/// The computer player a script order is for.
pub struct SupplyOrder<'a> {
    pub sim: &'a SimContext,
    pub ai: &'a mut AiPlayer,
    pub player: &'a mut Player,
    /// The middle of the structures of `getAiEnemy`, if it has an enemy.
    pub enemy_center: Option<Coord2D>,
    /// `TheTerrainLogic->getMaximumPathfindExtent`.
    pub pathfind_extent: Region3D,
}

/// `getBoxesStored` of the object's `SupplyWarehouseDockUpdate`.
fn warehouse_boxes(sim: &SimContext, slot: ObjectSlot) -> Option<Int> {
    sim.world
        .objects
        .get(slot)?
        .behaviors
        .iter()
        .find_map(|&(module, _)| {
            sim.world
                .modules
                .payload(module)?
                .as_any()?
                .downcast_ref::<SupplyWarehouseDockUpdate>()
                .map(|dock| dock.boxes_stored)
        })
}

/// `findSupplyCenter`: the supply source closest to the base which holds enough,
/// is no enemy's, has no cash generator of the player by it and is not much closer
/// to the enemy. The wanted sum is halved until one is found.
pub fn find_supply_center(order: &SupplyOrder<'_>, minimum_cash: Int) -> Option<ObjectSlot> {
    let sim = order.sim;
    let me = order.ai.player;
    let base = order.ai.base_center;
    let mut minimum = minimum_cash;
    loop {
        let mut best: Option<(ObjectSlot, Real)> = None;
        for slot in sim.world.objects.world_iter() {
            let kind = |k| sim.object_is_kind_of(slot, k) != FALSE;
            if !kind(KindOfType::Structure) || !kind(KindOfType::SupplySource) {
                continue;
            }
            let (Some(boxes), Some(object)) =
                (warehouse_boxes(sim, slot), sim.world.objects.get(slot))
            else {
                continue;
            };
            let cash = boxes.wrapping_mul(sim.rules.global_data.base_value_per_supply_box);
            if cash < minimum {
                continue;
            }
            let owner = sim.controlling_player(object.team_id);
            if owner.and_then(|o| sim.world.player_relationships.get(&(me, o)))
                == Some(&Relationship::Enemies)
            {
                continue;
            }
            let center = *object.thing.get_position();
            let radius = SUPPLY_CENTER_CLOSE_DIST + object.geometry.get_bounding_circle_radius();
            let mut mine = |s| {
                sim.object_is_kind_of(s, KindOfType::CashGenerator) != FALSE
                    && owner_of(sim, s) == Some(me)
                    && sim
                        .world
                        .objects
                        .get(s)
                        .is_some_and(|o| o.private_status & PRIVATE_STATUS_OFF_MAP == 0)
            };
            let taken = sim.closest_object_to_position(
                &center,
                radius,
                DistanceCalculationType::FromBoundingSphere2d,
                &mut mine,
            );
            if taken.is_some() {
                continue;
            }
            let dx = center.x - base.x;
            let dy = center.y - base.y;
            let squared = dx * dx + dy * dy;
            if let Some(enemy) = order.enemy_center {
                let ex = center.x - enemy.x;
                let ey = center.y - enemy.y;
                // `distSqr*0.4` is a double, the other side a float.
                let theirs = (ex * ex + ey * ey) * 0.6;
                if f64::from(squared) * 0.4 > f64::from(theirs) {
                    continue;
                }
            }
            if best.is_none_or(|(_, d)| d > squared) {
                best = Some((slot, squared));
            }
        }
        if best.is_some() {
            return best.map(|b| b.0);
        }
        minimum /= 2;
        if minimum <= 100 {
            return None;
        }
    }
}

/// `buildBySupplies`: lists one `template_name` to be built next to a supply
/// source, a cash generator on the side facing the base and anything else on the
/// side facing the enemy.
pub fn build_by_supplies(order: &mut SupplyOrder<'_>, minimum_cash: Int, template_name: &str) {
    use legal_options::*;
    let sim = order.sim;
    let factory = &sim.rules.thing_factory;
    let (Some(what), Some(template)) = (
        factory.find_template_id(template_name),
        factory.find_template_final(template_name),
    ) else {
        return;
    };
    let cash_generator = template.is_kind_of(KindOfType::CashGenerator) != FALSE;
    let mut best = find_supply_center(order, minimum_cash);
    if !cash_generator {
        best = sim
            .world
            .objects
            .slot_by_id(order.ai.cur_warehouse_id)
            .or(best);
    }
    let Some((warehouse, object)) = best.and_then(|s| Some((s, sim.world.objects.get(s)?))) else {
        return;
    };
    let mut location = *object.thing.get_position();
    let normalized = |x: Real, y: Real| {
        let length = gm_sqrtf(x * x + y * y);
        if length != 0.0 {
            (x / length, y / length)
        } else {
            (x, y)
        }
    };
    // Back towards the base.
    let mut offset = normalized(
        location.x - order.ai.base_center.x,
        location.y - order.ai.base_center.y,
    );
    let mut radius = 3.0 * PATHFIND_CELL_SIZE_F;
    if !cash_generator {
        // Probably a defensive structure: towards the enemy.
        let enemy = order.enemy_center.unwrap_or_default();
        offset = normalized(location.x - enemy.x, location.y - enemy.y);
        radius = object.geometry.get_bounding_circle_radius();
    }
    location.x -= offset.0 * radius;
    location.y -= offset.1 * radius;
    let angle = template.get_placement_view_angle();
    let skirmish_ai = order.player.ai == Some(AiKind::Skirmish);
    let extent = order.pathfind_extent;
    let legal = |at: &Coord3D, options| {
        is_location_legal_to_build(
            sim,
            &PlacementQuery {
                pos: at,
                what,
                angle,
                options,
                builder: None,
                skirmish_ai,
                pathfind_extent: &extent,
            },
        ) == LegalBuildCode::Ok
    };
    if !legal(&location, NO_OBJECT_OVERLAP) {
        let full = CLEAR_PATH | TERRAIN_RESTRICTIONS | NO_OBJECT_OVERLAP;
        if let Some(found) = wiggle(&location, |at| legal(at, full)) {
            location = found;
        }
    }
    // Build list locations are relative to the ground.
    location.z = 0.0;
    let warehouse_id = sim.world.objects.id_of(warehouse);
    let mut info = BuildListInfo::new();
    info.template_name = template_name.to_owned();
    info.location = location;
    info.angle = angle;
    info.priority_build = TRUE;
    info.num_rebuilds = 1;
    order.player.build_list.insert(0, info);
    if let Some(id) = warehouse_id {
        order.ai.cur_warehouse_id = id;
    }
}

/// The widening squares `buildBySupplies` searches around `center`.
fn wiggle(center: &Coord3D, legal: impl Fn(&Coord3D) -> bool) -> Option<Coord3D> {
    let cell = PATHFIND_CELL_SIZE_F;
    let mut at = *center;
    let mut pos_offset: Real = 0.0;
    while pos_offset < 2.0 * SUPPLY_CENTER_CLOSE_DIST {
        let offset = pos_offset / 2.0;
        let y = center.y - offset;
        let mut x = center.x - offset;
        while x <= center.x + offset {
            at.x = x;
            at.y = y;
            if legal(&at) {
                return Some(at);
            }
            at.y = y + pos_offset;
            if legal(&at) {
                return Some(at);
            }
            x += cell;
        }
        let x = center.x - offset;
        let mut y = center.y - offset;
        while y <= center.y + offset {
            at.x = x;
            at.y = y;
            if legal(&at) {
                return Some(at);
            }
            at.x = x + pos_offset;
            if legal(&at) {
                return Some(at);
            }
            y += cell;
        }
        pos_offset += 2.0 * cell;
    }
    None
}
