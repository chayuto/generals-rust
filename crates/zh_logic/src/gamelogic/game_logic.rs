//! `PKT-440` (partial): `GameLogic::startNewGame` of `GameLogic.cpp`, as far as the
//! simulation reads it: the map's INI, terrain and sides, the players and teams, the
//! pathfind map, the map's objects and each slot's starting building and units.
//!
//! Nothing is skipped silently: every map object, module and call that the port cannot
//! run is counted in [`StartReport`].
//!
//! Packet doc: `packets/phase7/PKT-440_GameLogic.md`.
//! Golden test: `crates/zh_logic/tests/golden_game_start.rs`
//! (`docs/golden_vectors/GameStart.json`, recorded from the running reference engine by
//! `tests/cpp_harness/gen_game_start.py`).

use std::collections::BTreeMap;

use zh_common::core::dict::Dict;
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::io::data_chunk::{
    CachedFileInputStream, DataChunkError, DataChunkInfo, DataChunkInput,
};
use zh_common::io::map_object::{MapObject, FLAG_BRIDGE_FLAGS, FLAG_ROAD_FLAGS};
use zh_common::io::recorder::{GameOptions, ReplayRules, ReplaySlot};
use zh_common::math::{gm_sqrtf, normalize_angle, Coord3D, ICoord2D};
use zh_common::types::{
    Bool, Int, ObjectId, Real, TeamId, UnsignedInt, UnsignedShort, VeterancyLevel, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

use crate::arena::{FacetMask, ModuleSlot, ObjectSlot};
use crate::economy::academy_stats::AcademyStats;
use crate::gamelogic::game_engine::{GameDataLoader, StartupStores};
use crate::gamelogic::global_data::PlayerType;
use crate::modules::module::{MaxHealthChangeType, ModuleType};
use crate::object::object_script_status_bits::{
    OBJECT_STATUS_SCRIPT_DISABLED, OBJECT_STATUS_SCRIPT_TARGETABLE, OBJECT_STATUS_SCRIPT_UNPOWERED,
    OBJECT_STATUS_SCRIPT_UNSELLABLE,
};
use crate::player::player_list::PlayerList;
use crate::player::seams::{
    AiKind, PlayerEnv, PlayerIndex, PlayerPart, PlayerWorld, SkirmishScripts,
};
use crate::player::team::TeamFactory;
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::KindOfType;
use crate::rules::science::ScienceType;
use crate::rules::upgrade::UpgradeMaskType;
use crate::scripts::scripts::{ScriptList, ScriptTemplateRegistry};
use crate::sim::{FindPositionOptions, SimApi, SimContext, SimView};
use crate::spatial::sides_list::{
    SidesList, SidesScriptList, WellKnownSidesKeys, PLAYERS_NAMES_FOR_SCRIPTS_VERSION_2,
};
use crate::terrain::TerrainLogic;

/// `MAX_SLOTS` of `GameInfo.h`.
const MAX_SLOTS: usize = 8;
/// `MAX_MP_STARTING_UNITS` of `PlayerTemplate.h`.
const MAX_MP_STARTING_UNITS: Int = 10;
const LOGICFRAMES_PER_SECOND: Int = 30;
const PLAYERTEMPLATE_OBSERVER: i32 = -2;
const SLOT_EASY_AI: i32 = 2;
const SLOT_MED_AI: i32 = 3;
const SLOT_BRUTAL_AI: i32 = 4;
const SLOT_PLAYER: i32 = 5;

/// A map and the files beside it, already read.
#[derive(Debug, Clone, Default)]
pub struct MapSource {
    /// `TheGlobalData->m_mapName`.
    pub path: String,
    pub bytes: Vec<u8>,
    /// `map.ini` and `solo.ini`, in that order, when present.
    pub inis: Vec<(String, Vec<u8>)>,
}

/// What `startNewGame` reads from outside the rule data.
#[derive(Debug, Clone, Default)]
pub struct StartParms {
    pub map: MapSource,
    /// `TheGameInfo`: a skirmish or replay game. `None` plays the map as it is.
    pub game: Option<GameOptions>,
    /// The logic seed (`InitRandom`) when there is no `game`.
    pub seed: UnsignedInt,
    /// `Data\Scripts\SkirmishScripts.scb`.
    pub skirmish_scripts: Option<Vec<u8>>,
    /// `Data\INI\ChallengeMode.ini`, for the generals that start locked.
    pub challenge_mode_ini: Vec<u8>,
}

/// Everything the start left out, by reason.
#[derive(Debug, Clone, Default)]
pub struct StartReport {
    /// Map objects of the file.
    pub map_objects: u32,
    /// Objects made from map objects.
    pub created_from_map: u32,
    /// Map objects without an object, by reason.
    pub skipped_objects: BTreeMap<&'static str, u32>,
    /// Behavior modules an object was made without, by `(module, reason)`.
    pub skipped_modules: BTreeMap<(String, &'static str), u32>,
    /// Calls into code that is not ported, by name.
    pub unported: BTreeMap<&'static str, u32>,
}

impl StartReport {
    fn skip_object(&mut self, reason: &'static str) {
        *self.skipped_objects.entry(reason).or_insert(0) += 1;
    }

    fn unported(&mut self, call: &'static str) {
        *self.unported.entry(call).or_insert(0) += 1;
    }
}

#[derive(Debug)]
pub enum StartError {
    Terrain(crate::terrain::MapLoadError),
    Sides(String),
}

/// A started game.
pub struct GameStart {
    pub sim: SimContext,
    pub stores: StartupStores,
    pub sides: SidesList,
    pub players: PlayerList,
    pub teams: TeamFactory,
    /// The game options after the random choices.
    pub game: Option<GameOptions>,
    /// The script list of every side, in sides list order
    /// (`SidesInfo::getScriptList`), as `ScriptEngine::newMap` leaves them.
    pub scripts: Vec<Option<ScriptList>>,
    /// Objects given a script name by their map properties.
    pub named_objects: Vec<(String, ObjectId)>,
    /// `(name, location)` of the terrain's waypoints.
    pub waypoints: Vec<(String, Coord3D)>,
    pub report: StartReport,
}

/// `ParseAsciiStringToGameInfo`'s outside answers for a game start.
pub struct ReplayGameRules {
    pub num_colors: i32,
    pub player_template_count: i32,
    pub default_starting_cash: u32,
}

impl ReplayGameRules {
    pub fn new(loader: &GameDataLoader) -> Self {
        Self {
            num_colors: loader
                .stores
                .multiplayer
                .as_ref()
                .map_or(0, |m| m.num_colors_cached()),
            player_template_count: loader.stores.player_templates.player_template_count(),
            default_starting_cash: loader.rules.global_data.default_starting_cash.count_money(),
        }
    }
}

impl ReplayRules for ReplayGameRules {
    fn num_colors(&self) -> i32 {
        self.num_colors
    }
    fn player_template_count(&self) -> i32 {
        self.player_template_count
    }
    fn default_starting_cash(&self) -> u32 {
        self.default_starting_cash
    }
    /// The portable path is kept; the caller maps it to a file.
    fn real_map_path(&self, portable: &[u8]) -> Option<Vec<u8>> {
        (!portable.is_empty()).then(|| portable.to_vec())
    }
    fn portable_map_path(&self, real: &[u8]) -> Vec<u8> {
        real.to_vec()
    }
}

// ---- the sides of the map file ----------------------------------------------

struct SidesPass<'k> {
    sides: SidesList,
    name_keys: &'k mut NameKeyGenerator,
    scripts: Vec<ScriptList>,
    error: Option<String>,
}

fn parse_sides(
    input: &mut DataChunkInput<'_, SidesPass<'_>>,
    info: &DataChunkInfo,
    pass: &mut SidesPass<'_>,
) -> Result<bool, DataChunkError> {
    if let Err(e) = pass
        .sides
        .read_sides_and_teams(input, info.version, pass.name_keys)
    {
        pass.error = Some(format!("{e:?}"));
        return Ok(false);
    }
    if !input.at_end_of_chunk() {
        let (label, _) = input.open_data_chunk()?;
        if label == "PlayerScriptsList" {
            ScriptList::read_player_scripts_chunk(
                input,
                &mut pass.scripts,
                pass.name_keys,
                &ScriptTemplateRegistry::new(),
            )?;
        }
        input.close_data_chunk()?;
    }
    Ok(true)
}

/// Script lists by the name of the side that owns them.
type SideScripts = BTreeMap<String, ScriptList>;

/// The script-list flags the sides list keeps in place of the list itself.
fn script_flags(list: &ScriptList) -> SidesScriptList {
    SidesScriptList::new(
        Bool::from(!list.scripts.is_empty()),
        Bool::from(!list.groups.is_empty()),
        0,
    )
}

fn side_name(sides: &SidesList, index: Int, keys: &WellKnownSidesKeys) -> String {
    sides.side_info(index).map_or_else(
        |_| String::new(),
        |s| s.dict.get_ascii_string(keys.player_name, None),
    )
}

/// `SidesList::ParseSidesDataChunk` over a whole map file.
fn load_sides(
    bytes: &[u8],
    keys: &WellKnownSidesKeys,
    name_keys: &mut NameKeyGenerator,
) -> Result<(SidesList, SideScripts), StartError> {
    let mut stream = CachedFileInputStream::from_bytes(bytes);
    let mut pass = SidesPass {
        sides: SidesList::new(),
        name_keys,
        scripts: Vec::new(),
        error: None,
    };
    let mut file: DataChunkInput<'_, SidesPass<'_>> = DataChunkInput::new(&mut stream);
    if file.is_valid_file_type() {
        file.register_parser("SidesList", "", parse_sides);
        file.parse(&mut pass)
            .map_err(|e| StartError::Sides(format!("{e:?}")))?;
    }
    if let Some(error) = pass.error {
        return Err(StartError::Sides(error));
    }
    let mut sides = pass.sides;
    let mut scripts = SideScripts::new();
    for (i, list) in pass.scripts.into_iter().enumerate() {
        if let Ok(side) = sides.side_info_mut(i as Int) {
            side.scripts = Some(script_flags(&list));
            let name = side.dict.get_ascii_string(keys.player_name, None);
            scripts.insert(name, list);
        }
    }
    sides.validate_sides(keys);
    Ok((sides, scripts))
}

/// The side names and script lists of `SkirmishScripts.scb`
/// (`SidesList::prepareForMP_or_Skirmish`).
fn skirmish_file_scripts(
    bytes: &[u8],
    name_keys: &mut NameKeyGenerator,
) -> Result<SideScripts, DataChunkError> {
    let mut names = Vec::new();
    let mut lists = Vec::new();
    let mut stream = CachedFileInputStream::from_bytes(bytes);
    let mut input: DataChunkInput<'_, ()> = DataChunkInput::new(&mut stream);
    if !input.is_valid_file_type() {
        return Err(DataChunkError::InvalidHeader);
    }
    input.reset()?;
    while !input.at_end_of_file() {
        let (label, version) = input.open_data_chunk()?;
        if label == "ScriptsPlayers" {
            let read_dicts =
                version >= PLAYERS_NAMES_FOR_SCRIPTS_VERSION_2 && input.read_int()? != 0;
            for _ in 0..input.read_int()?.max(0) {
                names.push(input.read_ascii_string()?);
                if read_dicts {
                    input.read_dict(name_keys)?;
                }
            }
        } else if label == "PlayerScriptsList" {
            ScriptList::read_player_scripts_chunk(
                &mut input,
                &mut lists,
                name_keys,
                &ScriptTemplateRegistry::new(),
            )?;
        }
        input.close_data_chunk()?;
    }
    Ok(names.into_iter().zip(lists).collect())
}

/// `checkConditionsForTeamNames` over every script of the sides: a script with a
/// delay draws the frame of its first evaluation.
fn draw_script_delays<'a>(sim: &mut SimContext, lists: impl Iterator<Item = &'a mut ScriptList>) {
    for list in lists {
        let grouped = list.groups.iter_mut().flat_map(|g| g.scripts.iter_mut());
        for script in list.scripts.iter_mut().chain(grouped) {
            script.frame_to_evaluate_at = if script.delay_evaluation_seconds > 0 {
                sim.world
                    .rng
                    .get_game_logic_random_value(0, 2 * LOGICFRAMES_PER_SECOND)
                    as UnsignedInt
            } else {
                0
            };
        }
    }
}

// ---- `PlayerWorld` at game start --------------------------------------------

struct StartWorld<'a> {
    sides_keys: WellKnownSidesKeys,
    frame: UnsignedInt,
    starting_cash: UnsignedInt,
    occupied: [bool; MAX_SLOTS],
    general_name: Vec<u16>,
    waypoints: &'a [(String, Coord3D)],
    waypoint_ids: &'a [UnsignedInt],
    upgrade_masks: Vec<(String, UpgradeMaskType)>,
    script_copies: Vec<ScriptCopy>,
    report: &'a mut StartReport,
}

impl PlayerWorld for StartWorld<'_> {
    fn frame(&self) -> UnsignedInt {
        self.frame
    }
    fn starting_cash(&self) -> UnsignedInt {
        self.starting_cash
    }
    fn is_network(&self) -> bool {
        false
    }
    fn game_slot_occupied(&self, slot: usize) -> bool {
        self.occupied.get(slot).copied().unwrap_or(false)
    }
    fn global_difficulty(&self) -> Int {
        // `DIFFICULTY_NORMAL`.
        1
    }
    fn force_skirmish_ai(&self) -> bool {
        false
    }
    fn general_name(&self) -> Vec<u16> {
        self.general_name.clone()
    }
    fn waypoint_location(&self, name: &str) -> Option<Coord3D> {
        self.waypoints
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|&(_, p)| p)
    }
    fn waypoint_exists(&self, id: UnsignedInt) -> bool {
        self.waypoint_ids.contains(&id)
    }
    fn becoming_local_player(&mut self, _player: PlayerIndex, _yes: bool) {}
    fn local_player_update(&mut self, _player: PlayerIndex) {}
    fn create_ai(&mut self, _player: PlayerIndex, _kind: AiKind) {
        self.report.unported("AIPlayer");
    }
    fn delete_ai(&mut self, _player: PlayerIndex) {}
    fn set_ai_difficulty(&mut self, _player: PlayerIndex, _difficulty: Int) {}
    fn ai_update(&mut self, _player: PlayerIndex) {}
    fn ai_new_map(&mut self, _player: PlayerIndex) {
        self.report.unported("AIPlayer::newMap");
    }
    fn ai_pre_team_destroy(&mut self, _player: PlayerIndex, _team: TeamId) {}
    fn qualify_skirmish_scripts(&mut self, sides: &mut SidesList, copy: &SkirmishScripts<'_>) {
        let key = self.sides_keys.player_name;
        self.script_copies.push(ScriptCopy {
            side: copy.side,
            source: sides
                .skirmish_sides
                .get(copy.skirmish_side as usize)
                .map_or_else(String::new, |s| s.dict.get_ascii_string(key, None)),
            qualifier: copy.qualifier.to_owned(),
            template_player_name: copy.template_player_name.to_owned(),
            player_name: copy.player_name.to_owned(),
        });
    }
    fn create_economy_trackers(&mut self, _player: PlayerIndex) {}
    fn delete_economy_trackers(&mut self, _player: PlayerIndex) {}
    fn academy_init(&mut self, _player: PlayerIndex) {}
    fn academy_update(&mut self, _player: PlayerIndex) {}
    fn academy_record_income(&mut self, _player: PlayerIndex) {}
    fn power_brownout_changed(&mut self, _player: PlayerIndex, _brownout: bool) {}
    fn science_added(&mut self, _player: PlayerIndex, _science: ScienceType) {}
    fn upgrade_count(&self, _player: PlayerIndex) -> UnsignedShort {
        0
    }
    fn delete_upgrades(&mut self, _player: PlayerIndex) {}
    fn upgrade_masks(&self) -> Vec<(String, UpgradeMaskType)> {
        self.upgrade_masks.clone()
    }
    fn notify_team_destroyed(&mut self, _team: TeamId) {}
    fn detach_member(&mut self, _object: ObjectId) {}
    fn member_changed_player(&mut self, _object: ObjectId, _old: PlayerIndex, _new: PlayerIndex) {}
    fn execute_team_create_actions(
        &mut self,
        _players: &mut PlayerList,
        _teams: &mut TeamFactory,
        _condition: &str,
    ) {
        self.report.unported("team create actions");
    }
    fn duplicate_script(&mut self, _name: &str) -> Option<UnsignedInt> {
        self.report.unported("team generic script");
        None
    }
    fn evaluate_script(
        &mut self,
        _players: &mut PlayerList,
        _teams: &mut TeamFactory,
        _script: UnsignedInt,
        _team: TeamId,
    ) -> bool {
        false
    }
    fn script_is_one_shot(&self, _script: UnsignedInt) -> bool {
        false
    }
    fn execute_script_action(
        &mut self,
        _players: &mut PlayerList,
        _teams: &mut TeamFactory,
        _script: UnsignedInt,
        _team: TeamId,
    ) {
    }
    fn run_script(
        &mut self,
        _players: &mut PlayerList,
        _teams: &mut TeamFactory,
        _name: &str,
        _team: TeamId,
    ) {
        self.report.unported("ScriptEngine::runScript");
    }
    fn member_is_effectively_dead(&self, _object: ObjectId) -> bool {
        false
    }
    fn member_ai_idle(&self, _object: ObjectId) -> Option<bool> {
        None
    }
    fn member_sees_enemy(&mut self, _object: ObjectId) -> bool {
        false
    }
    fn xfer_player_part(
        &mut self,
        _player: PlayerIndex,
        _part: PlayerPart,
        _xfer: &mut dyn Xfer,
    ) -> Result<(), XferError> {
        Err(XferError::InvalidData)
    }
}

/// One `ScriptList::duplicateAndQualify` of `Player::initFromDict`.
struct ScriptCopy {
    side: Int,
    source: String,
    qualifier: String,
    template_player_name: String,
    player_name: String,
}

/// The borrows every player or team call needs, split from the sim.
struct Env<'a> {
    sides_keys: WellKnownSidesKeys,
    waypoints: &'a [(String, Coord3D)],
    waypoint_ids: &'a [UnsignedInt],
    starting_cash: UnsignedInt,
    occupied: [bool; MAX_SLOTS],
}

impl Env<'_> {
    fn run<R>(
        &self,
        sim: &mut SimContext,
        stores: &StartupStores,
        report: &mut StartReport,
        f: impl FnOnce(&mut PlayerEnv<'_>) -> R,
    ) -> (R, Vec<ScriptCopy>) {
        let rules = &mut sim.rules;
        let upgrade_masks = rules
            .upgrade_center
            .get_upgrade_names()
            .into_iter()
            .filter_map(|n| {
                let mask = rules
                    .upgrade_center
                    .find_upgrade(&n, &mut rules.name_keys)?
                    .upgrade_mask()
                    .clone();
                Some((n, mask))
            })
            .collect();
        let mut world = StartWorld {
            sides_keys: self.sides_keys,
            script_copies: Vec::new(),
            frame: sim.world.frame,
            starting_cash: self.starting_cash,
            occupied: self.occupied,
            general_name: "SCIENCE:GeneralName".encode_utf16().collect(),
            waypoints: self.waypoints,
            waypoint_ids: self.waypoint_ids,
            upgrade_masks,
            report,
        };
        let mut env = PlayerEnv {
            templates: &stores.player_templates,
            ranks: &stores.ranks,
            sciences: &rules.science_store,
            name_keys: &mut rules.name_keys,
            world: &mut world,
        };
        let out = f(&mut env);
        (out, world.script_copies)
    }
}

// ---- random sides, colors and start positions -------------------------------

fn is_occupied(slot: &ReplaySlot) -> bool {
    matches!(
        slot.state,
        SLOT_PLAYER | SLOT_EASY_AI | SLOT_MED_AI | SLOT_BRUTAL_AI
    )
}

fn color_taken(game: &GameOptions, color: i32) -> bool {
    game.slots.iter().any(|s| s.color == color)
}

fn start_taken(game: &GameOptions, pos: i32) -> bool {
    game.slots.iter().any(|s| s.start_pos == pos)
}

fn check_for_duplicate_colors(game: &mut GameOptions, num_colors: i32) {
    for i in (0..MAX_SLOTS).rev() {
        if !is_occupied(&game.slots[i]) {
            continue;
        }
        let color = game.slots[i].color;
        if color < 0 || color >= num_colors {
            continue;
        }
        game.slots[i].color = -1;
        if !color_taken(game, color) {
            game.slots[i].color = color;
        }
    }
}

/// `StartsEnabled` of the `GeneralPersona` blocks, as `(template, enabled)`.
fn general_personas(ini: &[u8]) -> Vec<(String, bool)> {
    let text = String::from_utf8_lossy(ini);
    let mut out: Vec<(String, bool)> = Vec::new();
    let mut current: Option<(String, bool)> = None;
    for line in text.lines() {
        let line = line.split(';').next().unwrap_or("").trim();
        let mut words = line
            .split(|c: char| c.is_whitespace() || c == '=')
            .filter(|w| !w.is_empty());
        let Some(first) = words.next() else {
            continue;
        };
        if first.starts_with("GeneralPersona") {
            out.extend(current.take());
            current = Some((String::new(), false));
        } else if let Some(persona) = current.as_mut() {
            let value = words.next().unwrap_or("");
            if first == "PlayerTemplate" {
                persona.0 = value.to_owned();
            } else if first == "StartsEnabled" {
                persona.1 = value.eq_ignore_ascii_case("yes");
            }
        }
    }
    out.extend(current);
    out
}

fn starts_locked(personas: &[(String, bool)], template: &str) -> bool {
    template.eq_ignore_ascii_case("FactionBossGeneral")
        || personas
            .iter()
            .find(|(t, _)| t == template)
            .is_some_and(|&(_, enabled)| !enabled)
}

fn populate_random_side_and_color(
    game: &mut GameOptions,
    sim: &mut SimContext,
    stores: &StartupStores,
    personas: &[(String, bool)],
    num_colors: i32,
) {
    let templates = &stores.player_templates;
    let count = templates.player_template_count();
    let mut start_slots: Vec<i32> = Vec::new();
    for i in 0..count {
        let Some(pt) = templates.get_nth_player_template(i) else {
            continue;
        };
        if pt.starting_building().is_empty()
            || (game.old_factions_only && pt.is_old_faction() == FALSE)
            || starts_locked(personas, pt.name(&sim.rules.name_keys))
        {
            continue;
        }
        start_slots.push(i);
    }
    for i in 0..MAX_SLOTS {
        if !is_occupied(&game.slots[i]) {
            continue;
        }
        let mut template = game.slots[i].player_template;
        while template != PLAYERTEMPLATE_OBSERVER && (template < 0 || template >= count) {
            if start_slots.is_empty() {
                break;
            }
            let silly = sim.world.rng.get_game_logic_random_seed() % 7;
            for _ in 0..silly {
                sim.world.rng.get_game_logic_random_value(0, 1);
            }
            let index = sim.world.rng.get_game_logic_random_value(0, 1000) as usize;
            template = start_slots[index % start_slots.len()];
            match templates.get_nth_player_template(template) {
                Some(pt) if !pt.starting_building().is_empty() => {
                    game.slots[i].player_template = template;
                }
                _ => template = -1,
            }
        }
        let mut color = game.slots[i].color;
        if color < 0 || color >= num_colors {
            while color == -1 {
                color = sim.world.rng.get_game_logic_random_value(0, num_colors - 1);
                if color_taken(game, color) {
                    color = -1;
                }
            }
            game.slots[i].color = color;
        }
    }
}

/// Sum of the distances from `from` to the other taken spots, added to `base`.
fn taken_distance(
    distance: &[[Real; MAX_SLOTS]; MAX_SLOTS],
    taken: &[bool; MAX_SLOTS],
    from: usize,
    count: usize,
    base: Real,
) -> Real {
    let mut sum = base;
    for n in 0..count {
        if taken[n] && n != from {
            sum += distance[from][n];
        }
    }
    sum
}

/// The free spot farthest from the taken ones, or `None`.
fn farthest_spot(
    distance: &[[Real; MAX_SLOTS]; MAX_SLOTS],
    taken: &[bool; MAX_SLOTS],
    count: usize,
) -> Option<usize> {
    let mut best: Option<usize> = None;
    let mut best_distance: Real = 0.0;
    for pos in 0..count {
        if taken[pos] {
            continue;
        }
        if best.is_none() {
            best = Some(pos);
            best_distance = taken_distance(distance, taken, pos, count, best_distance);
        } else {
            let d = taken_distance(distance, taken, pos, count, 0.0);
            if d > best_distance {
                best_distance = d;
                best = Some(pos);
            }
        }
    }
    best
}

fn populate_random_start_position(
    game: &mut GameOptions,
    sim: &mut SimContext,
    waypoints: &[(String, Coord3D)],
) {
    let spot = |i: usize| {
        let name = format!("Player_{}_Start", i + 1);
        waypoints
            .iter()
            .rev()
            .find(|(n, _)| *n == name)
            .map(|&(_, p)| p)
    };
    // `MapMetaData::m_numPlayers`: the start spots numbered from one without a gap.
    let count = (0..MAX_SLOTS).take_while(|&i| spot(i).is_some()).count();
    let mut distance = [[0.0 as Real; MAX_SLOTS]; MAX_SLOTS];
    for (i, row) in distance.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            if i != j && i < count && j < count {
                *cell = match (spot(i), spot(j)) {
                    (Some(a), Some(b)) => {
                        let (dx, dy) = (a.x - b.x, a.y - b.y);
                        gm_sqrtf(dx * dx + dy * dy)
                    }
                    _ => 1_000_000.0,
                };
            }
        }
    }
    if count == 0 {
        return;
    }
    let playing = |s: &ReplaySlot| is_occupied(s) && s.player_template != PLAYERTEMPLATE_OBSERVER;
    let in_range = |p: i32| p >= 0 && (p as usize) < count;
    let mut taken = [false; MAX_SLOTS];
    for (i, t) in taken.iter_mut().enumerate() {
        *t = i >= count;
    }
    let mut picked = false;
    for slot in game.slots.iter().filter(|s| playing(s)) {
        let pos = slot.start_pos;
        if pos >= 0 {
            picked = true;
            if let Some(t) = taken.get_mut(pos as usize) {
                *t = true;
            }
        }
    }
    let mut team_pos = [-1 as i32; MAX_SLOTS];
    for i in 0..MAX_SLOTS {
        if !playing(&game.slots[i]) || in_range(game.slots[i].start_pos) {
            continue;
        }
        let team = game.slots[i].team_number;
        let pos = if !picked {
            let mut pos = game.slots[i].start_pos;
            while pos == -1 {
                pos = sim
                    .world
                    .rng
                    .get_game_logic_random_value(0, count as Int - 1);
                if start_taken(game, pos) {
                    pos = -1;
                }
            }
            picked = true;
            Some(pos as usize)
        } else if team < 0 || team_pos.get(team as usize).copied() == Some(-1) {
            farthest_spot(&distance, &taken, count)
        } else {
            let mate = team_pos[team as usize] as usize;
            let mut closest = Real::MAX;
            let mut index = 0;
            for n in 0..count {
                let d = distance[mate][n];
                if !taken[n] && d < closest {
                    closest = d;
                    index = n;
                }
            }
            Some(index)
        };
        game.slots[i].start_pos = pos.map_or(-1, |p| p as i32);
        if let Some(p) = pos {
            taken[p] = true;
            if team > -1 {
                if let Some(t) = team_pos.get_mut(team as usize) {
                    if *t == -1 {
                        *t = p as i32;
                    }
                }
            }
        }
    }
    let playing_count = game.slots.iter().filter(|s| playing(s)).count();
    for i in 0..MAX_SLOTS {
        if !is_occupied(&game.slots[i]) || game.slots[i].player_template != PLAYERTEMPLATE_OBSERVER
        {
            continue;
        }
        let mut pos = if playing_count == 0 { 0 } else { -1 };
        while pos == -1 {
            pos = sim
                .world
                .rng
                .get_game_logic_random_value(0, count as Int - 1);
            if !start_taken(game, pos) {
                pos = -1;
            }
        }
        game.slots[i].start_pos = pos;
    }
}

// ---- the slots' sides -------------------------------------------------------

/// The `player%d` side and team of every occupied slot, and `ReplayObserver`.
fn add_game_sides(
    game: &GameOptions,
    skirmish: bool,
    sides: &mut SidesList,
    sim: &mut SimContext,
    stores: &mut StartupStores,
    keys: &WellKnownSidesKeys,
) {
    let name_keys = &mut sim.rules.name_keys;
    let color_key = name_keys.name_to_key("playerColor");
    let night_key = name_keys.name_to_key("playerNightColor");
    let start_key = name_keys.name_to_key("multiplayerStartIndex");
    let local_key = name_keys.name_to_key("multiplayerIsLocal");
    let difficulty_key = name_keys.name_to_key("skirmishDifficulty");
    let skirmish_key = name_keys.name_to_key("playerIsSkirmish");
    // `GameInfo::isPlayerPreorder` is not in the options string.
    let observer = name_keys.name_to_key("FactionObserver");
    let templates = &stores.player_templates;
    let multiplayer = &mut stores.multiplayer;
    let mut colors = |which: i32| {
        multiplayer
            .as_mut()
            .and_then(|m| m.get_color(which).map(|c| (c.color(), c.night_color())))
            .unwrap_or((0, 0))
    };
    let add_team = |sides: &mut SidesList, owner: &str| {
        let mut d = Dict::new();
        d.set_ascii_string(keys.team_name, &format!("team{owner}"));
        d.set_ascii_string(keys.team_owner, owner);
        d.set_bool(keys.team_is_singleton, TRUE);
        sides.add_team(Some(&d));
    };
    for i in 0..MAX_SLOTS {
        let slot = &game.slots[i];
        if !is_occupied(slot) {
            continue;
        }
        let mut d = Dict::new();
        let name = format!("player{i}");
        d.set_ascii_string(keys.player_name, &name);
        let human = slot.state == SLOT_PLAYER;
        d.set_bool(keys.player_is_human, Bool::from(human));
        let display: Vec<u16> = slot.name.iter().map(|&c| c as u16).collect();
        d.set_unicode_string(keys.player_display_name, &display);
        let template = if slot.player_template >= 0 {
            templates.get_nth_player_template(slot.player_template)
        } else {
            templates.find_player_template(observer, name_keys)
        };
        if let Some(pt) = template {
            let faction = name_keys.key_to_name(pt.name_key()).to_owned();
            d.set_ascii_string(keys.player_faction, &faction);
        }
        let mut enemies: Vec<String> = Vec::new();
        let mut allies: Vec<String> = Vec::new();
        for (j, other) in game.slots.iter().enumerate() {
            if i == j || !is_occupied(other) {
                continue;
            }
            let list = if slot.team_number == -1 || other.team_number != slot.team_number {
                &mut enemies
            } else {
                &mut allies
            };
            list.push(format!("player{j}"));
        }
        d.set_ascii_string(keys.player_allies, &allies.join(" "));
        d.set_ascii_string(keys.player_enemies, &enemies.join(" "));
        let (color, night) = colors(slot.color);
        d.set_int(color_key, color as Int);
        d.set_int(night_key, night as Int);
        d.set_int(start_key, slot.start_pos);
        d.set_bool(local_key, Bool::from(human && i as i32 == game.local_slot));
        if skirmish {
            d.set_bool(skirmish_key, TRUE);
            let difficulty = match slot.state {
                SLOT_EASY_AI => Some(0),
                SLOT_MED_AI => Some(1),
                SLOT_BRUTAL_AI => Some(2),
                _ => None,
            };
            if let Some(value) = difficulty {
                d.set_int(difficulty_key, value);
            }
        }
        sides.add_side(Some(&d));
        add_team(sides, &name);
    }
    let mut d = Dict::new();
    d.set_ascii_string(keys.player_name, "ReplayObserver");
    d.set_bool(keys.player_is_human, TRUE);
    let display: Vec<u16> = "Observer".encode_utf16().collect();
    d.set_unicode_string(keys.player_display_name, &display);
    if let Some(pt) = templates.find_player_template(observer, name_keys) {
        let faction = name_keys.key_to_name(pt.name_key()).to_owned();
        d.set_ascii_string(keys.player_faction, &faction);
    }
    d.set_ascii_string(keys.player_allies, "");
    d.set_ascii_string(keys.player_enemies, "");
    let (color, night) = colors(0);
    d.set_int(color_key, color as Int);
    d.set_int(night_key, night as Int);
    d.set_int(start_key, 0);
    d.set_bool(local_key, FALSE);
    sides.add_side(Some(&d));
    add_team(sides, "ReplayObserver");
}

// ---- players and teams into the sim -----------------------------------------

/// Gives the sim the player of every team and the players' prototype lists, and the
/// per-team copies of player state that the sim still keys by team.
pub fn sync_teams(sim: &mut SimContext, players: &PlayerList, teams: &TeamFactory) {
    if let Some(team) = players.player(0).and_then(|p| p.default_team) {
        sim.world.neutral_team_id = team;
    }
    for index in 0..players.player_count() {
        let Some(player) = players.player(index) else {
            continue;
        };
        let mut lists: Vec<Vec<TeamId>> = Vec::new();
        for &prototype in &player.team_prototypes {
            let instances = teams.instance_ids(prototype);
            for &team in &instances {
                sim.world.team_players.insert(team, index);
                sim.world
                    .team_is_neutral
                    .insert(team, Bool::from(index == 0));
                sim.world
                    .player_is_human
                    .insert(team, Bool::from(player.player_type == PlayerType::Human));
                sim.world.team_sides.insert(team, player.side.clone());
                sim.world.team_colors.insert(team, player.color);
                sim.world
                    .player_sciences
                    .insert(team, player.sciences.iter().copied().collect());
                sim.world
                    .player_preorder
                    .insert(team, Bool::from(player.is_preorder));
                sim.world
                    .player_handicaps
                    .insert(team, player.handicap.clone());
                if player.ai == Some(AiKind::Skirmish) {
                    sim.world.skirmish_ai_teams.insert(team);
                }
            }
            if !instances.is_empty() {
                lists.push(instances);
            }
        }
        sim.world.player_team_prototypes.insert(index, lists);
        if let Some(team) = player.default_team {
            sim.world.player_default_teams.insert(index, team);
        }
    }
}

/// The player state the sim keeps itself, once after `PlayerList::newGame`.
fn sync_players(sim: &mut SimContext, stores: &StartupStores, players: &PlayerList) {
    sim.world.local_player_index = players.local_player();
    for index in 0..players.player_count() {
        let Some(player) = players.player(index) else {
            continue;
        };
        sim.world
            .player_money
            .insert(index, player.money.count_money());
        sim.world
            .player_upgrades
            .insert(index, player.upgrades_completed.clone());
        for other in 0..players.player_count() {
            if let Some(r) = player.player_relations.get(other as u32) {
                sim.world.player_relationships.insert((index, other), r);
            }
        }
        let playable = player
            .template_name_key
            .and_then(|k| {
                stores
                    .player_templates
                    .find_player_template(k, &sim.rules.name_keys)
            })
            .is_some_and(|t| t.is_playable_side() != FALSE);
        if playable {
            if let Some(team) = player.default_team {
                sim.world.playable_side_teams.insert(team);
            }
        }
        sim.world
            .player_academy
            .insert(index, AcademyStats::default());
    }
}

/// Copies the sim's member lists into the team factory.
fn sync_members(sim: &SimContext, players: &PlayerList, teams: &mut TeamFactory) {
    for index in 0..players.player_count() {
        let Some(player) = players.player(index) else {
            continue;
        };
        for &prototype in &player.team_prototypes {
            for team in teams.instance_ids(prototype) {
                let members: Vec<ObjectId> =
                    sim.world
                        .team_members
                        .get(&team)
                        .map_or_else(Vec::new, |m| {
                            m.iter()
                                .filter_map(|&s| sim.world.objects.id_of(s))
                                .collect()
                        });
                if let Some(t) = teams.team_mut(team) {
                    t.members = members;
                }
            }
        }
    }
}

// ---- objects ----------------------------------------------------------------

struct Loader<'a> {
    sim: &'a mut SimContext,
    report: &'a mut StartReport,
    named: &'a mut Vec<(String, ObjectId)>,
}

impl Loader<'_> {
    /// Counts the behavior modules of `template` that the sim will not attach.
    fn count_skipped_modules(&mut self, slot: ObjectSlot) {
        let rules = &self.sim.rules;
        let Some(template) = self
            .sim
            .world
            .objects
            .get(slot)
            .and_then(|o| o.thing.template_override().get_final(&rules.thing_factory))
        else {
            return;
        };
        for nugget in template.get_behavior_module_info().nuggets() {
            let descriptor = rules
                .module_registry
                .find_descriptor(&nugget.name, ModuleType::Behavior);
            let reason = match descriptor {
                None => "not registered",
                Some(d) if d.create_module.is_none() => "module not ported",
                Some(_) if rules.module_data_store.get(nugget.data_id).is_none() => {
                    "no module data"
                }
                Some(_) => continue,
            };
            *self
                .report
                .skipped_modules
                .entry((nugget.name.clone(), reason))
                .or_insert(0) += 1;
        }
    }

    /// `TheThingFactory->newObject`.
    fn new_object(&mut self, name: &str, team: TeamId) -> Option<ObjectSlot> {
        let tmpl = self.sim.rules.thing_factory.find_template_id(name)?;
        let slot = self.sim.factory_new_object(tmpl, team);
        self.count_skipped_modules(slot);
        Some(slot)
    }

    /// `onBuildComplete` of every create module.
    fn build_complete(&mut self, slot: ObjectSlot) {
        let behaviors: Vec<(ModuleSlot, FacetMask)> = self
            .sim
            .world
            .objects
            .get(slot)
            .map_or_else(Vec::new, |o| o.behaviors.clone());
        for (module, mask) in behaviors {
            if mask.contains(FacetMask::CREATE) && self.sim.world.modules.is_alive(module) {
                let _ = self.sim.dispatch_module(module, |m, sim, ctx| {
                    if let Some(c) = m.as_create_mut() {
                        c.on_build_complete(sim, ctx);
                    }
                });
            }
        }
    }

    fn set_script_status(&mut self, slot: ObjectSlot, bit: u8, set: bool) {
        let Some(obj) = self.sim.world.objects.get_mut(slot) else {
            return;
        };
        let old = obj.script_status;
        if set {
            obj.script_status |= bit;
        } else {
            obj.script_status &= !bit;
        }
        let new = obj.script_status;
        for (flag, disabled) in [
            (OBJECT_STATUS_SCRIPT_DISABLED, DisabledType::ScriptDisabled),
            (
                OBJECT_STATUS_SCRIPT_UNPOWERED,
                DisabledType::ScriptUnderpowered,
            ),
        ] {
            if new & flag != old & flag {
                if new & flag != 0 {
                    self.sim.set_disabled(slot, disabled);
                } else {
                    self.sim.clear_disabled(slot, disabled);
                }
            }
        }
    }

    /// `Object::updateObjValuesFromMapProperties`, less the drawable's settings.
    fn apply_map_properties(&mut self, slot: ObjectSlot, d: &Dict) {
        let mut key = |name: &str| self.sim.rules.name_keys.name_to_key(name);
        let name_key = key("objectName");
        let max_hp_key = key("objectMaxHPs");
        let health_key = key("objectInitialHealth");
        let veterancy_key = key("objectVeterancy");
        let attitude_key = key("objectAggressiveness");
        let recruit_key = key("objectRecruitableAI");
        let selectable_key = key("objectSelectable");
        let stopping_key = key("objectStoppingDistance");
        let enabled_key = key("objectEnabled");
        let powered_key = key("objectPowered");
        let indestructible_key = key("objectIndestructible");
        let unsellable_key = key("objectUnsellable");
        let targetable_key = key("objectTargetable");
        let vision_key = key("objectVisualRange");
        let shroud_key = key("objectShroudClearingDistance");

        let mut exists = FALSE;
        let name = d.get_ascii_string(name_key, Some(&mut exists));
        if exists != FALSE {
            if let Some(id) = self.sim.world.objects.id_of(slot) {
                self.named.push((name, id));
            }
        }
        let value = d.get_int(max_hp_key, Some(&mut exists));
        if exists != FALSE && value >= 0 {
            self.sim
                .set_max_health(slot, value as Real, MaxHealthChangeType::SameCurrentHealth);
        }
        let value = d.get_int(health_key, Some(&mut exists));
        if exists != FALSE {
            self.sim.set_initial_health(slot, value);
        }
        let value = d.get_int(veterancy_key, Some(&mut exists));
        if exists != FALSE {
            let trainable = self
                .sim
                .world
                .objects
                .get(slot)
                .is_some_and(|o| o.experience_tracker.is_trainable() != FALSE);
            if trainable {
                self.sim
                    .set_veterancy_level(slot, VeterancyLevel::from_i32(value), FALSE, TRUE);
            }
        }
        let value = d.get_int(attitude_key, Some(&mut exists));
        if exists != FALSE && value != 0 {
            self.report.unported("AIUpdateInterface::setAttitude");
        }
        let flag = d.get_bool(recruit_key, Some(&mut exists));
        if exists != FALSE && flag == FALSE {
            self.report.unported("AIUpdateInterface::setIsRecruitable");
        }
        let _ = d.get_bool(selectable_key, Some(&mut exists));
        if exists != FALSE {
            self.report.unported("Object::setSelectable");
        }
        let distance = d.get_real(stopping_key, Some(&mut exists));
        if exists != FALSE && distance >= 0.5 {
            self.report.unported("Locomotor::setCloseEnoughDist");
        }
        let flag = d.get_bool(enabled_key, Some(&mut exists));
        if exists != FALSE {
            self.set_script_status(slot, OBJECT_STATUS_SCRIPT_DISABLED, flag == FALSE);
        }
        let flag = d.get_bool(powered_key, Some(&mut exists));
        if exists != FALSE {
            self.set_script_status(slot, OBJECT_STATUS_SCRIPT_UNPOWERED, flag == FALSE);
        }
        let flag = d.get_bool(indestructible_key, Some(&mut exists));
        if exists != FALSE {
            if let Some(body) = self.sim.world.objects.get(slot).and_then(|o| o.body_slot) {
                let _ = self.sim.dispatch_module(body, |m, sim, ctx| {
                    if let Some(b) = m.as_body_mut() {
                        b.set_indestructible(sim, ctx, flag);
                    }
                });
            }
        }
        let flag = d.get_bool(unsellable_key, Some(&mut exists));
        if exists != FALSE {
            self.set_script_status(slot, OBJECT_STATUS_SCRIPT_UNSELLABLE, flag != FALSE);
        }
        let flag = d.get_bool(targetable_key, Some(&mut exists));
        if exists != FALSE {
            self.set_script_status(slot, OBJECT_STATUS_SCRIPT_TARGETABLE, flag != FALSE);
        }
        let value = d.get_int(vision_key, Some(&mut exists));
        if exists != FALSE {
            self.sim.set_vision_range(slot, value.max(0) as Real);
        }
        let value = d.get_int(shroud_key, Some(&mut exists));
        if exists != FALSE {
            if let Some(obj) = self.sim.world.objects.get_mut(slot) {
                obj.shroud_clearing_range = value.max(0) as Real;
            }
        }
        let mut number = 0;
        loop {
            let key = self
                .sim
                .rules
                .name_keys
                .name_to_key(&format!("objectGrantUpgrade{number}"));
            let upgrade = d.get_ascii_string(key, Some(&mut exists));
            if exists == FALSE || upgrade.is_empty() {
                break;
            }
            let rules = &mut self.sim.rules;
            let mask = rules
                .upgrade_center
                .find_upgrade(&upgrade, &mut rules.name_keys)
                .map(|u| u.upgrade_mask().clone());
            if let Some(mask) = mask {
                if let Some(obj) = self.sim.world.objects.get_mut(slot) {
                    obj.upgrades_completed.set_flags(&mask);
                }
                self.sim.update_upgrade_modules(slot);
            }
            number += 1;
        }
    }
}

fn is_kind(sim: &SimContext, name: &str, kind: KindOfType) -> bool {
    sim.rules
        .thing_factory
        .find_template_final(name)
        .is_some_and(|t| t.is_kind_of(kind) != FALSE)
}

/// `handleNameChange`.
fn map_object_name(object: &MapObject) -> &str {
    match object.name() {
        "AmericaTankLeopard" => "AmericaTankCrusader",
        "AmericaVehicleHumVee" => "AmericaVehicleHumvee",
        name => name,
    }
}

/// The map object's position, raised onto the ground.
fn map_position(sim: &SimContext, object: &MapObject) -> Coord3D {
    let mut pos = *object.location();
    pos.z += sim.terrain.get_ground_height(pos.x, pos.y, None);
    pos
}

fn load_bridge_like_objects(loader: &mut Loader<'_>, map_objects: &[MapObject], neutral: TeamId) {
    for object in map_objects {
        if object.flags() & (FLAG_BRIDGE_FLAGS | FLAG_ROAD_FLAGS) != 0 {
            continue;
        }
        let name = object.name();
        let Some(template) = loader.sim.rules.thing_factory.find_template_final(name) else {
            continue;
        };
        if template.is_bridge_like() == FALSE {
            continue;
        }
        let is_bridge = template.is_bridge() != FALSE;
        let wall = template.is_kind_of(KindOfType::WalkOnTopOfWall) != FALSE;
        let Some(slot) = loader.new_object(name, neutral) else {
            continue;
        };
        loader.report.created_from_map += 1;
        let pos = map_position(loader.sim, object);
        loader
            .sim
            .set_object_orientation(slot, normalize_angle(object.angle()));
        loader.sim.set_object_position(slot, &pos);
        if is_bridge {
            loader
                .report
                .unported("TerrainLogic::addLandmarkBridgeToLogic");
        }
        if wall {
            let id = loader.sim.world.objects.id_of(slot);
            if let (Some(id), Some(pathfinder)) = (id, loader.sim.world.pathfinder.as_mut()) {
                pathfinder.add_wall_piece(id);
            }
        }
        loader.apply_map_properties(slot, object.properties());
    }
}

fn load_map_objects(
    loader: &mut Loader<'_>,
    stores: &StartupStores,
    players: &mut PlayerList,
    teams: &mut TeamFactory,
    env: &Env<'_>,
    map_objects: &[MapObject],
    use_trees: bool,
    force_fluff_to_prop: bool,
) {
    let owner_key = loader.sim.rules.name_keys.name_to_key("originalOwner");
    for object in map_objects {
        if object.flags() & (FLAG_BRIDGE_FLAGS | FLAG_ROAD_FLAGS) != 0 {
            loader.report.skip_object("road or bridge point");
            continue;
        }
        let name = map_object_name(object);
        let Some(template) = loader.sim.rules.thing_factory.find_template_final(name) else {
            loader.report.skip_object("no template");
            continue;
        };
        if template.is_bridge_like() != FALSE {
            continue;
        }
        let fence_width = template.get_fence_width();
        let kind = |k| is_kind(loader.sim, name, k);
        if kind(KindOfType::Shrubbery) && !use_trees {
            loader.report.skip_object("trees off");
            continue;
        }
        if kind(KindOfType::OptimizedTree) {
            loader.report.skip_object("optimized tree");
            continue;
        }
        let prop = kind(KindOfType::Prop);
        let fluff = kind(KindOfType::ClearedByBuild) && fence_width == 0.0;
        if prop || (fluff && force_fluff_to_prop) {
            loader.report.skip_object("terrain prop");
            continue;
        }
        let pos = map_position(loader.sim, object);
        let angle = normalize_angle(object.angle());
        let owner = object.properties().get_ascii_string(owner_key, None);
        let (team, _) = env.run(loader.sim, stores, loader.report, |e| {
            players.validate_team(&owner, teams, e)
        });
        let Some(team) = team else {
            loader.report.skip_object("no team");
            continue;
        };
        sync_teams(loader.sim, players, teams);
        let Some(slot) = loader.new_object(name, team) else {
            loader.report.skip_object("no object");
            continue;
        };
        loader.report.created_from_map += 1;
        loader.sim.set_object_orientation(slot, angle);
        loader.sim.set_object_position(slot, &pos);
        loader.apply_map_properties(slot, object.properties());
        let layer = loader.sim.get_layer_for_destination(&pos);
        loader.sim.set_object_layer(slot, layer);
        loader.build_complete(slot);
        if let Some(t) = teams.team_mut(team) {
            t.set_active();
        }
        loader.sim.add_object_to_pathfind_map(slot);
    }
}

/// `placeObjectAtPosition`.
fn place_object(
    loader: &mut Loader<'_>,
    teams: &mut TeamFactory,
    name: &str,
    pos: &Coord3D,
    team: TeamId,
) -> Option<ObjectSlot> {
    let angle = loader
        .sim
        .rules
        .thing_factory
        .find_template_final(name)?
        .get_placement_view_angle();
    let slot = loader.new_object(name, team)?;
    loader.sim.set_object_orientation(slot, angle);
    loader.sim.set_object_position(slot, pos);
    loader.build_complete(slot);
    if let Some(t) = teams.team_mut(team) {
        t.set_active();
    }
    loader.sim.add_object_to_pathfind_map(slot);
    // Asked of every unit with an AI there; a unit whose AI module is not ported
    // has none here, so the count is by kind.
    if loader.sim.object_is_kind_of(slot, KindOfType::Immobile) == FALSE {
        loader.report.unported("Pathfinder::adjustDestination");
    }
    Some(slot)
}

/// `placeNetworkBuildingsForPlayer`: the starting building and units of one slot.
fn place_network_buildings(
    loader: &mut Loader<'_>,
    teams: &mut TeamFactory,
    waypoints: &[(String, Coord3D)],
    team: TeamId,
    start_pos: i32,
    building: &str,
    units: &[String],
) {
    let find = |name: String| {
        waypoints
            .iter()
            .rev()
            .find(|(n, _)| *n == name)
            .map(|w| w.1)
    };
    let Some(mut pos) = find(format!("Player_{}_Start", start_pos + 1)) else {
        loader.report.skip_object("no start waypoint");
        return;
    };
    let rally = find(format!("Player_{}_Rally", start_pos + 1));
    pos.z = loader.sim.terrain.get_ground_height(pos.x, pos.y, None);
    if building.is_empty() {
        return;
    }
    let Some(yard) = place_object(loader, teams, building, &pos, team) else {
        loader.report.skip_object("no starting building");
        return;
    };
    // `Player::onStructureCreated` and `onStructureConstructionComplete`: the score,
    // academy and AI records.
    loader
        .report
        .unported("Player::onStructureConstructionComplete");
    let radius = loader
        .sim
        .world
        .objects
        .get(yard)
        .map_or(0.0, |o| o.geometry.get_bounding_sphere_radius());
    pos.y -= radius / 2.0;
    pos.z = loader.sim.terrain.get_ground_height(pos.x, pos.y, None);
    if let Some(r) = rally {
        pos = r;
        pos.z = loader.sim.terrain.get_ground_height(pos.x, pos.y, None);
    }
    for name in units.iter().filter(|n| !n.is_empty()) {
        let options = FindPositionOptions {
            min_radius: radius * 0.7,
            max_radius: radius * 1.3,
            ..FindPositionOptions::default()
        };
        loader.sim.update_partition();
        let Some(unit_pos) = loader.sim.find_position_around(&pos, &options) else {
            loader.report.skip_object("no position for starting unit");
            continue;
        };
        if let Some(unit) = place_object(loader, teams, name, &unit_pos, team) {
            loader.sim.on_unit_created(yard, unit);
        } else {
            loader.report.skip_object("no starting unit");
        }
    }
}

// ---- startNewGame -----------------------------------------------------------

/// `GameLogic::startNewGame` over loaded rule data.
pub fn start_new_game(loader: GameDataLoader, parms: StartParms) -> Result<GameStart, StartError> {
    start_new_game_observed(loader, parms, |_, _, _, _| {})
}

/// A point of `startNewGame` at which [`start_new_game_observed`] shows the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartStage {
    /// The map's objects are made; no starting building or unit is.
    BeforeStartingUnits,
    /// The call of `ThePlayerList->newMap`, where the computer players set up
    /// their bases.
    BeforePlayerMaps,
}

/// [`start_new_game`], showing `observe` the game at each [`StartStage`].
pub fn start_new_game_observed(
    mut loader: GameDataLoader,
    parms: StartParms,
    mut observe: impl FnMut(StartStage, &SimContext, &PlayerList, &TeamFactory),
) -> Result<GameStart, StartError> {
    // `GameLogic::loadMapINI`.
    for (name, bytes) in &parms.map.inis {
        loader.load_bytes(name, bytes, zh_common::ini::IniLoadType::CreateOverrides);
    }
    let GameDataLoader {
        rules, mut stores, ..
    } = loader;
    let mut sim = SimContext::new();
    sim.rules = rules;
    let mut report = StartReport::default();
    let mut game = parms.game.clone();
    let seed = game.as_ref().map_or(parms.seed, |g| g.seed as UnsignedInt);
    sim.world.rng.init_random_with_seed(seed);
    sim.world.frame = 0;
    sim.world.next_object_id = 1;

    // The terrain, whose waypoints stand in for `TheMapCache` too.
    let water = ICoord2D::new(
        sim.rules.global_data.water_extent_x as Int,
        sim.rules.global_data.water_extent_y as Int,
    );
    let mut terrain = TerrainLogic::new();
    terrain
        .load_map(
            &parms.map.bytes,
            &parms.map.path,
            &mut sim.rules.name_keys,
            water,
            FALSE,
        )
        .map_err(StartError::Terrain)?;
    let map_objects: Vec<MapObject> = terrain.map_objects().to_vec();
    report.map_objects = map_objects.len() as u32;
    // `getFirstWaypoint` walks newest first, so the last added wins a name.
    let waypoint_list = |terrain: &TerrainLogic| -> Vec<(String, Coord3D)> {
        let mut list: Vec<(String, Coord3D)> = terrain
            .waypoints()
            .map(|w| (w.name.clone(), w.location))
            .collect();
        list.reverse();
        list
    };
    let mut waypoints = waypoint_list(&terrain);

    let keys = WellKnownSidesKeys::intern(&mut sim.rules.name_keys);
    let (mut sides, map_scripts) = load_sides(&parms.map.bytes, &keys, &mut sim.rules.name_keys)?;
    let mut skirmish_scripts = SideScripts::new();

    let num_colors = stores
        .multiplayer
        .as_ref()
        .map_or(0, |m| m.num_colors_cached());
    let personas = general_personas(&parms.challenge_mode_ini);
    let mut occupied = [false; MAX_SLOTS];
    if let Some(g) = game.as_mut() {
        check_for_duplicate_colors(g, num_colors);
        let skirmish = g
            .slots
            .iter()
            .any(|s| (SLOT_EASY_AI..=SLOT_BRUTAL_AI).contains(&s.state));
        populate_random_side_and_color(g, &mut sim, &stores, &personas, num_colors);
        populate_random_start_position(g, &mut sim, &waypoints);
        for (i, slot) in g.slots.iter().enumerate() {
            occupied[i] = is_occupied(slot);
        }
        let multiplayer = occupied.iter().any(|&o| o);
        if multiplayer || skirmish {
            // The map's own scripts are used if a playing side has any.
            let map_has_scripts = sides.sides.iter().any(|s| {
                s.dict.get_ascii_string(keys.player_faction, None) != "FactionCivilian"
                    && s.scripts.as_ref().is_some_and(|l| !l.is_empty())
            });
            skirmish_scripts = map_scripts.clone();
            if !map_has_scripts {
                if let Some(bytes) = parms.skirmish_scripts.as_deref() {
                    let file = skirmish_file_scripts(bytes, &mut sim.rules.name_keys)
                        .map_err(|e| StartError::Sides(format!("{e:?}")))?;
                    skirmish_scripts.extend(file);
                }
            }
            sides
                .prepare_for_mp_or_skirmish(
                    parms.skirmish_scripts.as_deref(),
                    &keys,
                    &mut sim.rules.name_keys,
                )
                .map_err(|e| StartError::Sides(format!("{e:?}")))?;
            for side in &mut sides.skirmish_sides {
                let name = side.dict.get_ascii_string(keys.player_name, None);
                side.scripts = skirmish_scripts.get(&name).map(script_flags);
            }
        }
        add_game_sides(g, skirmish, &mut sides, &mut sim, &mut stores, &keys);
    }
    sides.validate_sides(&keys);

    let waypoint_ids: Vec<UnsignedInt> = terrain.waypoints().map(|w| w.id).collect();
    let env = Env {
        sides_keys: keys,
        waypoints: &waypoints.clone(),
        waypoint_ids: &waypoint_ids,
        starting_cash: game.as_ref().map_or(
            sim.rules.global_data.default_starting_cash.count_money(),
            |g| g.starting_cash,
        ),
        occupied,
    };
    let mut teams = TeamFactory::new();
    let (mut players, _) = env.run(&mut sim, &stores, &mut report, PlayerList::new);
    let (_, copies) = env.run(&mut sim, &stores, &mut report, |e| {
        players.new_game(&mut sides, &mut teams, e)
    });
    // The sides that stayed keep their map scripts; players take the copies.
    let mut scripts: Vec<Option<ScriptList>> = (0..sides.num_sides())
        .map(|i| map_scripts.get(&side_name(&sides, i, &keys)).cloned())
        .collect();
    let provider = ScriptTemplateRegistry::new();
    for copy in copies {
        let list = skirmish_scripts.get(&copy.source).map(|l| {
            l.duplicate_and_qualify(
                &copy.qualifier,
                &copy.template_player_name,
                &copy.player_name,
                &provider,
            )
        });
        if let Some(entry) = scripts.get_mut(copy.side as usize) {
            *entry = list;
        }
        // A human player takes the civilian scripts away from their side.
        if copy.source.ends_with("Civilian") {
            skirmish_scripts.remove(&copy.source);
        }
    }
    sync_players(&mut sim, &stores, &players);
    sync_teams(&mut sim, &players, &teams);
    // The part of `ScriptEngine::newMap` that draws; the engine itself is not run.
    draw_script_delays(&mut sim, scripts.iter_mut().flatten());
    report.unported("ScriptEngine");

    // `ThePartitionManager->init` and `TheTerrainLogic->newMap`.
    let extent = terrain.extent();
    let cell = sim.rules.global_data.partition_cell_size;
    terrain.new_map();
    waypoints = waypoint_list(&terrain);
    let pathfind_extent = terrain.maximum_pathfind_extent();
    sim.terrain = Box::new(terrain);
    sim.init_partition(cell, extent);
    for object in &map_objects {
        if object.flags() & FLAG_BRIDGE_FLAGS != 0 {
            report.unported("map bridge");
        }
    }

    let mut named = Vec::new();
    let neutral = sim.world.neutral_team_id;
    // `AIData::m_wallHeight`; `AIData` is not parsed here.
    let wall_height = 0.0;
    sim.world.pathfinder = Some(Box::new(crate::pathfinder::Pathfinder::new(wall_height)));
    load_bridge_like_objects(
        &mut Loader {
            sim: &mut sim,
            report: &mut report,
            named: &mut named,
        },
        &map_objects,
        neutral,
    );
    sim.new_pathfind_map(&pathfind_extent, wall_height);

    let observer = players.find_player_with_name("ReplayObserver");
    if let Some(team) = observer.and_then(|p| players.player(p)?.default_team) {
        sim.reveal_map_for_player_permanently(team);
    }
    if let Some(g) = game.as_ref() {
        let shroud = stores
            .multiplayer
            .as_ref()
            .is_some_and(|m| m.is_shroud_in_multiplayer() != FALSE);
        for (i, slot) in g.slots.iter().enumerate() {
            if !is_occupied(slot) {
                continue;
            }
            let player = players.player_from_slot_index(i as Int);
            let Some(team) = player.and_then(|p| players.player(p)?.default_team) else {
                continue;
            };
            if slot.player_template == PLAYERTEMPLATE_OBSERVER {
                sim.reveal_map_for_player_permanently(team);
            } else if !shroud {
                sim.reveal_map_for_player(team);
            }
        }
    }

    // A replay of a game with occupied slots is a multiplayer game.
    let multiplayer = game.is_some();
    let use_trees = multiplayer || sim.rules.global_data.use_trees != FALSE;
    load_map_objects(
        &mut Loader {
            sim: &mut sim,
            report: &mut report,
            named: &mut named,
        },
        &stores,
        &mut players,
        &mut teams,
        &env,
        &map_objects,
        use_trees,
        multiplayer,
    );
    sync_teams(&mut sim, &players, &teams);
    sync_members(&sim, &players, &mut teams);
    observe(StartStage::BeforeStartingUnits, &sim, &players, &teams);

    if let Some(g) = game.as_ref() {
        for (i, slot) in g.slots.iter().enumerate() {
            if !is_occupied(slot) {
                continue;
            }
            let Some(player) = players.player_from_slot_index(i as Int) else {
                continue;
            };
            let Some(team) = players.player(player).and_then(|p| p.default_team) else {
                continue;
            };
            if slot.player_template == PLAYERTEMPLATE_OBSERVER {
                continue;
            }
            let Some(pt) = stores
                .player_templates
                .get_nth_player_template(slot.player_template)
            else {
                continue;
            };
            if starts_locked(&personas, pt.name(&sim.rules.name_keys)) {
                continue;
            }
            if g.old_factions_only && pt.is_old_faction() == FALSE {
                continue;
            }
            let units: Vec<String> = (0..MAX_MP_STARTING_UNITS)
                .map(|n| pt.starting_unit(n).to_owned())
                .collect();
            place_network_buildings(
                &mut Loader {
                    sim: &mut sim,
                    report: &mut report,
                    named: &mut named,
                },
                &mut teams,
                &waypoints,
                team,
                slot.start_pos,
                pt.starting_building(),
                &units,
            );
        }
    }

    sim.update_partition();
    sync_teams(&mut sim, &players, &teams);
    sync_members(&sim, &players, &mut teams);
    observe(StartStage::BeforePlayerMaps, &sim, &players, &teams);
    env.run(&mut sim, &stores, &mut report, |e| players.new_map(e.world));
    sync_teams(&mut sim, &players, &teams);
    sync_members(&sim, &players, &mut teams);
    Ok(GameStart {
        sim,
        stores,
        sides,
        players,
        teams,
        scripts,
        game,
        named_objects: named,
        waypoints,
        report,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn personas_default_to_locked_unless_enabled() {
        let ini = b"ChallengeGenerals\n GeneralPersona0\n  PlayerTemplate = A ; x\n  StartsEnabled = yes\n End\n GeneralPersona1\n  PlayerTemplate = B\n End\nEnd\n";
        let personas = general_personas(ini);
        assert!(!starts_locked(&personas, "A"));
        assert!(starts_locked(&personas, "B"));
        assert!(!starts_locked(&personas, "C"));
        assert!(starts_locked(&personas, "FactionBossGeneral"));
    }
}
