//! `PKT-439` (partial): the INI part of `GameEngine::init` and `GameLogic::loadMapINI`
//! (`GeneralsMD/Code/GameEngine/Source/Common/GameEngine.cpp`), reading `.big` archives
//! through [`BigFileSystem`] in `INI::loadFileDirectory` order into [`RulesRegistry`].
//!
//! Every top-level block ends up in exactly one [`BlockStatus`] count; nothing is dropped
//! without a trace. A failing block is recorded and reading resumes at the next block
//! header in column 0 (the C++ would abort the load instead).
//!
//! Golden test: `crates/zh_logic/tests/golden_game_data_load.rs`
//! (`docs/golden_vectors/GameDataLoad.json`).

use std::collections::{BTreeMap, BTreeSet};

use zh_common::ini::{IniError, IniLoadType, IniReader, SEPS_DEFAULT};
use zh_common::io::archive_file_system::ArchiveFilter;
use zh_common::io::big_file_system::BigFileSystem;
use zh_common::io::file::LocalFileHost;

use crate::combat::locomotor::LocomotorTemplate;
use crate::modules::module::ModuleType;
use crate::object::object_creation_list::OclRefs;
use crate::object::thing_template::{DefaultModuleMaskLookup, ThingTemplateParseContext};
use crate::rules::command_set::{CommandButton, CommandButtonType};
use crate::rules::multiplayer_settings::{
    parse_multiplayer_color_definition, parse_multiplayer_settings_definition,
    parse_multiplayer_starting_money_choice_definition, MultiplayerSettings,
};
use crate::rules::player_template::PlayerTemplateStore;
use crate::rules::rank_info::RankInfoStore;
use crate::rules::science::ScienceTextLookup;
use crate::sim::{NoopGameText, RulesRegistry};
use crate::spatial::terrain_types::TerrainTypeCollection;

/// `theTypeTable` of `INI.cpp`. Matching is case sensitive.
pub const BLOCK_TYPES: [&str; 62] = [
    "AIData",
    "Animation",
    "Armor",
    "AudioEvent",
    "AudioSettings",
    "BenchProfile",
    "Bridge",
    "Campaign",
    "ChallengeGenerals",
    "CommandButton",
    "CommandMap",
    "CommandSet",
    "ControlBarResizer",
    "ControlBarScheme",
    "CrateData",
    "Credits",
    "DamageFX",
    "DialogEvent",
    "DrawGroupInfo",
    "DynamicGameLOD",
    "EvaEvent",
    "FXList",
    "GameData",
    "HeaderTemplate",
    "InGameUI",
    "LODPreset",
    "Language",
    "Locomotor",
    "MapCache",
    "MapData",
    "MappedImage",
    "MiscAudio",
    "Mouse",
    "MouseCursor",
    "MultiplayerColor",
    "MultiplayerSettings",
    "MultiplayerStartingMoneyChoice",
    "MusicTrack",
    "Object",
    "ObjectCreationList",
    "ObjectReskin",
    "OnlineChatColors",
    "ParticleSystem",
    "PlayerTemplate",
    "Rank",
    "ReallyLowMHz",
    "Road",
    "Science",
    "ScriptAction",
    "ScriptCondition",
    "ShellMenuScheme",
    "SpecialPower",
    "StaticGameLOD",
    "Terrain",
    "Upgrade",
    "Video",
    "WaterSet",
    "WaterTransparency",
    "Weapon",
    "Weather",
    "WebpageURL",
    "WindowTransition",
];

/// Blocks only the client reads.
const CLIENT_BLOCKS: [&str; 30] = [
    "Animation",
    "AudioEvent",
    "AudioSettings",
    "BenchProfile",
    "Campaign",
    "ChallengeGenerals",
    "CommandMap",
    "ControlBarResizer",
    "ControlBarScheme",
    "Credits",
    "DialogEvent",
    "DrawGroupInfo",
    "DynamicGameLOD",
    "EvaEvent",
    "HeaderTemplate",
    "InGameUI",
    "LODPreset",
    "Language",
    "MapCache",
    "MappedImage",
    "MiscAudio",
    "Mouse",
    "MouseCursor",
    "MusicTrack",
    "OnlineChatColors",
    "ReallyLowMHz",
    "ShellMenuScheme",
    "StaticGameLOD",
    "Video",
    "WindowTransition",
];

/// One step of the startup sequence: the `loadFileDirectory` paths of one subsystem.
/// The order is that of `GameEngine::init`; audio and client subsystems are included
/// because their blocks must be accounted for.
pub const INIT_PATHS: &[&str] = &[
    "Data\\INI\\Default\\GameData",
    "Data\\INI\\GameData",
    // `GameDataDebug` is read only by `RTS_DEBUG` builds.
    "Data\\INI\\GameLOD",
    "Data\\INI\\GameLODPresets",
    "Data\\INI\\Default\\Water",
    "Data\\INI\\Water",
    "Data\\INI\\Default\\Weather",
    "Data\\INI\\Weather",
    "Data\\INI\\Default\\Science",
    "Data\\INI\\Science",
    "Data\\INI\\Default\\Multiplayer",
    "Data\\INI\\Multiplayer",
    "Data\\INI\\Default\\Terrain",
    "Data\\INI\\Terrain",
    "Data\\INI\\Default\\Roads",
    "Data\\INI\\Roads",
    "Data\\English\\Language",
    "Data\\INI\\AudioSettings",
    "Data\\INI\\Default\\Music",
    "Data\\INI\\Music",
    "Data\\INI\\Default\\SoundEffects",
    "Data\\INI\\SoundEffects",
    "Data\\INI\\Default\\Speech",
    "Data\\INI\\Speech",
    "Data\\INI\\Default\\Voice",
    "Data\\INI\\Voice",
    "Data\\INI\\MiscAudio",
    "Data\\INI\\Rank",
    "Data\\INI\\Default\\PlayerTemplate",
    "Data\\INI\\PlayerTemplate",
    "Data\\INI\\ParticleSystem",
    "Data\\INI\\Default\\FXList",
    "Data\\INI\\FXList",
    "Data\\INI\\Weapon",
    "Data\\INI\\Default\\ObjectCreationList",
    "Data\\INI\\ObjectCreationList",
    "Data\\INI\\Locomotor",
    "Data\\INI\\Default\\SpecialPower",
    "Data\\INI\\SpecialPower",
    "Data\\INI\\DamageFX",
    "Data\\INI\\Armor",
    "Data\\INI\\Default\\Object",
    "Data\\INI\\Object",
    "Data\\INI\\Default\\Upgrade",
    "Data\\INI\\Upgrade",
    // `TheGameClient->init`.
    "Data\\INI\\DrawGroupInfo",
    "Data\\INI\\MappedImages\\TextureSize_512",
    "Data\\INI\\MappedImages\\HandCreated",
    "Data\\INI\\Animation2D",
    "Data\\INI\\Default\\ShellMenuScheme",
    "Data\\INI\\ShellMenuScheme",
    "Data\\INI\\InGameUI",
    "Data\\INI\\Default\\CommandButton",
    "Data\\INI\\CommandButton",
    "Data\\INI\\CommandSet",
    "Data\\INI\\Default\\ControlBarScheme",
    "Data\\INI\\ControlBarScheme",
    "Data\\INI\\ControlBarResizer",
    "Data\\INI\\ChallengeMode",
    "Data\\INI\\Mouse",
    "Data\\INI\\Default\\Video",
    "Data\\INI\\Video",
    "Data\\INI\\Campaign",
    "Data\\INI\\Eva",
    // Back in `GameEngine::init`.
    "Data\\INI\\Default\\AIData",
    "Data\\INI\\AIData",
    "Data\\INI\\Default\\Crate",
    "Data\\INI\\Crate",
    "Data\\INI\\CommandMap",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BlockStatus {
    /// Parsed into a store.
    Loaded,
    /// Parsed by a parser that does not cover the whole block.
    Partial,
    /// Only the name was recorded.
    NameOnly,
    /// Client-only, skipped.
    Client,
    /// The simulation needs it and there is no parser yet.
    Unported,
    /// The parser returned an error.
    Failed,
    /// A line where a block header was expected.
    Unknown,
}

impl BlockStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Loaded => "loaded",
            Self::Partial => "partial",
            Self::NameOnly => "name_only",
            Self::Client => "client_skipped",
            Self::Unported => "unported",
            Self::Failed => "failed",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoadFailure {
    pub file: String,
    /// 1-based line of the block header.
    pub line: usize,
    pub block: String,
    pub name: String,
    pub error: IniError,
    /// Lines the parser had consumed from the header when it failed.
    pub consumed: usize,
}

#[derive(Debug, Clone, Default)]
pub struct IniLoadReport {
    /// Files read, in order.
    pub files: Vec<String>,
    pub blocks: BTreeMap<(String, BlockStatus), u32>,
    pub failures: Vec<LoadFailure>,
    /// `Object` module declarations whose module data has no parser, by module name.
    pub stub_modules: BTreeMap<String, u32>,
    /// Fields the partial parsers ignore, by `Block.Field`.
    pub ignored_fields: BTreeMap<String, u32>,
    /// `(block type, name)` of every block a parser accepted.
    pub entries: BTreeSet<(&'static str, String)>,
}

impl IniLoadReport {
    fn count(&mut self, block: &str, status: BlockStatus) {
        *self.blocks.entry((block.to_owned(), status)).or_insert(0) += 1;
    }

    pub fn total(&self, status: BlockStatus) -> u32 {
        self.blocks
            .iter()
            .filter(|((_, s), _)| *s == status)
            .map(|(_, n)| *n)
            .sum()
    }
}

/// Stores the simulation reads that [`RulesRegistry`] does not hold yet.
#[derive(Default)]
pub struct StartupStores {
    pub multiplayer: Option<MultiplayerSettings>,
    pub terrain_types: TerrainTypeCollection,
    pub ranks: RankInfoStore,
    pub player_templates: PlayerTemplateStore,
}

/// Science and rank names stay untranslated: `TheGameText` is client data.
struct RawLabels;

impl ScienceTextLookup for RawLabels {
    fn fetch_label(&mut self, label: &str) -> String {
        label.to_owned()
    }
}

fn first_token(line: &[u8]) -> Option<&str> {
    let end = line.iter().position(|&b| b == b';').unwrap_or(line.len());
    std::str::from_utf8(&line[..end])
        .ok()
        .or_else(|| std::str::from_utf8(line.split(|&b| b >= 0x80).next()?).ok())?
        .split(|c: char| c <= ' ' || c == '=')
        .find(|t| !t.is_empty())
}

fn block_type(token: &str) -> Option<&'static str> {
    BLOCK_TYPES.iter().copied().find(|t| *t == token)
}

/// Mounts the retail archives as `BIGFileSystem::init` does for Zero Hour: the game
/// directory first, then the base Generals archives underneath.
pub fn mount_game_archives(host: &dyn LocalFileHost, game_dir: &str) -> BigFileSystem {
    let mut fs = BigFileSystem::new();
    fs.load_archive_files_from_directory(host, game_dir, "*.big", false, true);
    let base = format!("{game_dir}/ZH_Generals");
    fs.load_archive_files_from_directory(host, &base, "*.big", false, false);
    fs
}

/// The files `INI::loadFileDirectory(path)` reads, in order.
pub fn file_directory_list(fs: &BigFileSystem, path: &str) -> Vec<String> {
    let mut out = Vec::new();
    let file = format!("{path}.ini");
    if fs.does_file_exist(&file, 0, ArchiveFilter::All) != 0 {
        out.push(file);
    }
    let dir = format!("{path}\\");
    let listed = fs.get_file_list_in_directory(path, "*.ini");
    let (nested, direct): (Vec<_>, Vec<_>) = listed.into_iter().map(|k| k.0).partition(|f| {
        f.get(dir.len()..)
            .is_some_and(|rest| rest.contains(['\\', '/']))
    });
    out.extend(direct);
    out.extend(nested);
    out
}

pub struct GameDataLoader {
    pub rules: RulesRegistry,
    pub stores: StartupStores,
    pub report: IniLoadReport,
    /// `CommandButton` blocks by name, for resolving `CommandSet` slots.
    buttons: BTreeMap<String, CommandButton>,
}

impl Default for GameDataLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl GameDataLoader {
    pub fn new() -> Self {
        let mut rules = RulesRegistry::new();
        let RulesRegistry {
            module_registry,
            name_keys,
            ..
        } = &mut rules;
        module_registry.register_w3d_draw_stubs(name_keys);
        Self {
            rules,
            stores: StartupStores::default(),
            report: IniLoadReport::default(),
            buttons: BTreeMap::new(),
        }
    }

    /// Loads everything `GameEngine::init` loads, then runs the post-load passes.
    pub fn load_startup(&mut self, fs: &BigFileSystem) {
        for path in INIT_PATHS {
            for file in file_directory_list(fs, path) {
                self.load_archive_file(fs, &file, IniLoadType::Overwrite);
            }
        }
        self.rules.thing_factory.post_process_load();
        self.rules.mark_base_loaded();
    }

    /// `GameLogic::loadMapINI`: `map.ini` and `solo.ini` beside the map, as overrides.
    pub fn load_map_ini(&mut self, fs: &BigFileSystem, map_directory: &str) {
        for leaf in ["map.ini", "solo.ini"] {
            let file = format!("{map_directory}\\{leaf}");
            if fs.does_file_exist(&file, 0, ArchiveFilter::All) != 0 {
                self.load_archive_file(fs, &file, IniLoadType::CreateOverrides);
            }
        }
    }

    fn load_archive_file(&mut self, fs: &BigFileSystem, file: &str, load_type: IniLoadType) {
        match fs.open_file(file, 0, ArchiveFilter::All) {
            Some(ram) => self.load_bytes(file, ram.as_bytes(), load_type),
            None => self.report.failures.push(LoadFailure {
                file: file.to_owned(),
                line: 0,
                block: String::new(),
                name: String::new(),
                error: IniError::CantOpenFile,
                consumed: 0,
            }),
        }
    }

    /// `INI::load` over one file already in memory.
    pub fn load_bytes(&mut self, file: &str, data: &[u8], load_type: IniLoadType) {
        self.report.files.push(file.to_owned());
        let lines: Vec<&[u8]> = data.split_inclusive(|&b| b == b'\n').collect();
        let mut offsets = Vec::with_capacity(lines.len() + 1);
        let mut at = 0;
        for line in &lines {
            offsets.push(at);
            at += line.len();
        }
        offsets.push(at);

        let mut i = 0;
        while i < lines.len() {
            let Some(token) = first_token(lines[i]) else {
                i += 1;
                continue;
            };
            let Some(kind) = block_type(token) else {
                self.report.count(token, BlockStatus::Unknown);
                i = next_header(&lines, i + 1);
                continue;
            };
            i = self.load_block(file, kind, &lines, &offsets, data, i, load_type);
        }
    }

    /// Handles the block whose header is line `i`; returns the line after it.
    #[allow(clippy::too_many_arguments)]
    fn load_block(
        &mut self,
        file: &str,
        kind: &'static str,
        lines: &[&[u8]],
        offsets: &[usize],
        data: &[u8],
        i: usize,
        load_type: IniLoadType,
    ) -> usize {
        // One-line blocks.
        if kind == "ReallyLowMHz" {
            self.report.count(kind, BlockStatus::Client);
            return i + 1;
        }
        let name = std::str::from_utf8(lines[i])
            .unwrap_or("")
            .split_whitespace()
            .nth(1)
            .unwrap_or("")
            .to_owned();
        let resync = next_header(lines, i + 1);
        let status = match kind {
            k if CLIENT_BLOCKS.contains(&k) => Some(BlockStatus::Client),
            "FXList" => {
                self.rules.fx_list_names.insert(name.clone());
                Some(BlockStatus::NameOnly)
            }
            "ParticleSystem" => {
                self.rules.particle_system_names.insert(name.clone());
                Some(BlockStatus::NameOnly)
            }
            "MapData" | "ScriptAction" | "ScriptCondition" | "WaterSet" | "WaterTransparency"
            | "Weather" | "WebpageURL" => Some(BlockStatus::Unported),
            _ => None,
        };
        if let Some(status) = status {
            self.report.count(kind, status);
            return resync;
        }

        let mut ini = IniReader::new(file, &data[offsets[i]..], load_type);
        let result = self.parse_block(kind, &mut ini, load_type);
        let consumed = ini.get_line_num() as usize;
        match result {
            Ok(status) => {
                self.report.count(kind, status);
                self.report.entries.insert((kind, name.clone()));
                if kind == "Object" || kind == "ObjectReskin" {
                    self.note_stub_modules(&lines[i..(i + consumed).min(lines.len())]);
                }
                (i + consumed.max(1)).min(lines.len())
            }
            Err(error) => {
                self.report.count(kind, BlockStatus::Failed);
                self.report.failures.push(LoadFailure {
                    file: file.to_owned(),
                    line: i + 1,
                    block: kind.to_owned(),
                    name,
                    error,
                    consumed,
                });
                resync
            }
        }
    }

    fn parse_block(
        &mut self,
        kind: &str,
        ini: &mut IniReader<'_>,
        load_type: IniLoadType,
    ) -> Result<BlockStatus, IniError> {
        ini.read_line(None)?;
        let _keyword = ini.next_token(SEPS_DEFAULT)?;
        let r = &mut self.rules;
        let mut text = NoopGameText;
        let mut labels = RawLabels;
        match kind {
            // A map's `GameData` writes in place: the registry holds no override stack.
            "GameData" => r.global_data.parse_ini_block(ini)?,
            "Science" => {
                r.science_store
                    .parse_science_definition(ini, &mut r.name_keys, &mut labels)?;
            }
            "Upgrade" => r
                .upgrade_center
                .parse_upgrade_definition(ini, &mut r.name_keys)?,
            "SpecialPower" => r.special_power_store.parse_special_power_definition(
                ini,
                &r.science_store,
                &mut r.name_keys,
            )?,
            "Armor" => {
                r.armor_store
                    .parse_armor_definition(ini, &mut r.name_keys)?;
            }
            "DamageFX" => {
                let fx = &r.fx_list_names;
                let mut known: Vec<String> = Vec::new();
                r.damage_fx_store
                    .parse_damage_fx_definition(ini, &mut r.name_keys, |name| {
                        if fx.contains(name) {
                            known.push(name.to_owned());
                        }
                        None
                    })?;
                // Only the existence of the list is checked; no key is attached.
                return Ok(BlockStatus::Partial);
            }
            "CrateData" => r.crate_system.parse_crate_template_definition(
                ini,
                load_type,
                &r.science_store,
                &mut r.name_keys,
            )?,
            "Rank" => self.stores.ranks.parse_rank_definition(
                ini,
                &r.science_store,
                &mut r.name_keys,
                &mut labels,
            )?,
            "PlayerTemplate" => self
                .stores
                .player_templates
                .parse_player_template_definition(
                    ini,
                    &r.science_store,
                    &mut r.name_keys,
                    &mut labels,
                )?,
            "MultiplayerSettings" => {
                parse_multiplayer_settings_definition(&mut self.stores.multiplayer, ini)?
            }
            "MultiplayerColor" => parse_multiplayer_color_definition(
                self.stores.multiplayer.get_or_insert_with(Default::default),
                ini,
            )?,
            "MultiplayerStartingMoneyChoice" => parse_multiplayer_starting_money_choice_definition(
                self.stores.multiplayer.get_or_insert_with(Default::default),
                ini,
            )?,
            "Terrain" => self.stores.terrain_types.parse_terrain_definition(ini)?,
            "Object" | "ObjectReskin" => {
                let name = ini.next_token(SEPS_DEFAULT)?.to_owned();
                let reskin_from = if kind == "ObjectReskin" {
                    ini.next_token(SEPS_DEFAULT)?.to_owned()
                } else {
                    String::new()
                };
                let masks = DefaultModuleMaskLookup;
                let mut ctx = ThingTemplateParseContext {
                    name_keys: &mut r.name_keys,
                    armor_store: &r.armor_store,
                    weapon_store: &r.weapon_store,
                    science_store: &r.science_store,
                    fx_list_names: &r.fx_list_names,
                    damage_fx_names: &r.damage_fx_names,
                    game_text: &mut text,
                    module_masks: &masks,
                    default_occlusion_delay: r.global_data.default_occlusion_delay,
                    locomotor_store: Some(&r.locomotor_store),
                    module_registry: Some(&r.module_registry),
                    module_data_store: Some(&mut r.module_data_store),
                    upgrade_center: Some(&r.upgrade_center),
                    ocl_names: Some(&r.ocl_names),
                    particle_system_names: Some(&r.particle_system_names),
                    special_power_store: Some(&r.special_power_store),
                    crate_system: Some(&r.crate_system),
                };
                r.thing_factory
                    .parse_object_definition(ini, &name, &reskin_from, &mut ctx)?;
            }
            "Weapon" => return self.parse_weapon(ini, load_type),
            "Locomotor" => return self.parse_locomotor(ini),
            "AIData" | "Bridge" | "Road" | "ObjectCreationList" => {
                return self.parse_with_field_tables(kind, ini)
            }
            "CommandSet" => return self.parse_command_set(ini),
            "CommandButton" => return self.parse_command_button(ini),
            _ => return Err(IniError::UnknownToken),
        }
        Ok(BlockStatus::Loaded)
    }

    /// Counts the keys a parser's field tables lack. The reference build skips them
    /// (`DEBUG_CRASH` only), so the block stays, marked as partial.
    fn note_ignored(&mut self, kind: &str, ignored: Vec<String>) -> BlockStatus {
        if ignored.is_empty() {
            return BlockStatus::Loaded;
        }
        for field in ignored {
            *self
                .report
                .ignored_fields
                .entry(format!("{kind}.{field}"))
                .or_insert(0) += 1;
        }
        BlockStatus::Partial
    }

    /// `WeaponStore::parseWeaponTemplateDefinition`.
    fn parse_weapon(
        &mut self,
        ini: &mut IniReader<'_>,
        load_type: IniLoadType,
    ) -> Result<BlockStatus, IniError> {
        let name = ini.next_token(SEPS_DEFAULT)?.to_owned();
        let r = &mut self.rules;
        let existing = r.weapon_store.find_weapon_template(&name).cloned();
        // The reference returns early on a repeated name and then trips over the
        // block's fields.
        if existing.is_some() && load_type != IniLoadType::CreateOverrides {
            return Err(IniError::UnknownToken);
        }
        // An override copy never reaches the name map, so lookups keep the original.
        let is_override = existing.is_some();
        let mut t = existing.unwrap_or_default();
        t.name = name;
        let mut ignored = Vec::new();
        t.parse_fields(ini, &mut ignored)?;

        // `parseFXList`, `parseParticleSystemTemplate` and `parseWeaponTemplate` keep
        // only what exists.
        let keep = |names: &mut [String; 4], known: &BTreeSet<String>| {
            for name in names {
                if !known.contains(name.as_str()) {
                    name.clear();
                }
            }
        };
        keep(&mut t.fire_fx, &r.fx_list_names);
        keep(&mut t.projectile_detonation_fx, &r.fx_list_names);
        keep(&mut t.projectile_exhaust, &r.particle_system_names);
        if t.historic_bonus_weapon != t.name
            && !r.weapon_store.has_template(&t.historic_bonus_weapon)
        {
            t.historic_bonus_weapon.clear();
        }
        if !is_override {
            r.weapon_store.add_template(t);
        }
        Ok(self.note_ignored("Weapon", ignored))
    }

    /// `LocomotorStore::parseLocomotorTemplateDefinition`: a repeated name is parsed on
    /// top of what it holds.
    fn parse_locomotor(&mut self, ini: &mut IniReader<'_>) -> Result<BlockStatus, IniError> {
        let name = ini.next_token(SEPS_DEFAULT)?.to_owned();
        let store = &mut self.rules.locomotor_store;
        let mut t = store
            .find_by_name(&name)
            .cloned()
            .unwrap_or_else(LocomotorTemplate::new);
        t.name = name;
        let mut ignored = Vec::new();
        t.parse_fields(ini, &mut ignored)?;
        store.add_template(t);
        Ok(self.note_ignored("Locomotor", ignored))
    }

    fn parse_with_field_tables(
        &mut self,
        kind: &str,
        ini: &mut IniReader<'_>,
    ) -> Result<BlockStatus, IniError> {
        let r = &mut self.rules;
        let mut ignored = Vec::new();
        match kind {
            "AIData" => r.ai_data.parse_definition(
                ini,
                &r.science_store,
                &mut r.name_keys,
                &mut ignored,
            )?,
            "Bridge" => r.terrain_roads.parse_bridge_definition(ini, &mut ignored)?,
            "Road" => r.terrain_roads.parse_road_definition(ini, &mut ignored)?,
            _ => {
                let refs = OclRefs {
                    weapon_store: &r.weapon_store,
                    fx_list_names: &r.fx_list_names,
                };
                // The name is known from the header on, as `ocl_names` records it.
                let name = std::str::from_utf8(ini.current_line_bytes())
                    .unwrap_or("")
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or("")
                    .to_owned();
                r.ocl_names.insert(name);
                r.ocl_store.parse_definition(ini, refs, &mut ignored)?;
            }
        }
        Ok(self.note_ignored(kind, ignored))
    }

    /// Reads `Command`, `Object` and `Upgrade` of a button, which is what the sim's
    /// [`CommandSetStore`](crate::rules::command_set::CommandSetStore) holds.
    fn parse_command_button(&mut self, ini: &mut IniReader<'_>) -> Result<BlockStatus, IniError> {
        let name = ini.next_token(SEPS_DEFAULT)?.to_owned();
        let mut button = CommandButton::default();
        let ignored = &mut self.report.ignored_fields;
        ini.read_block_fields(|field, ini| {
            match field {
                "Command" => {
                    button.command = match ini.next_token(SEPS_DEFAULT)? {
                        "UNIT_BUILD" => CommandButtonType::UnitBuild,
                        "DOZER_CONSTRUCT" => CommandButtonType::DozerConstruct,
                        _ => CommandButtonType::Other,
                    }
                }
                "Object" => button.object = name_or_none(ini.next_token(SEPS_DEFAULT)?),
                "Upgrade" => button.upgrade = name_or_none(ini.next_token(SEPS_DEFAULT)?),
                other => *ignored.entry(format!("CommandButton.{other}")).or_insert(0) += 1,
            }
            Ok(())
        })?;
        self.buttons.insert(name, button);
        Ok(BlockStatus::Partial)
    }

    fn parse_command_set(&mut self, ini: &mut IniReader<'_>) -> Result<BlockStatus, IniError> {
        let name = ini.next_token(SEPS_DEFAULT)?.to_owned();
        let mut slots: BTreeMap<u32, CommandButton> = BTreeMap::new();
        let buttons = &self.buttons;
        ini.read_block_fields(|field, ini| {
            let value = ini.next_token(SEPS_DEFAULT)?;
            let slot: u32 = field.parse().map_err(|_| IniError::UnknownToken)?;
            let button = buttons.get(value).ok_or(IniError::InvalidData)?;
            slots.insert(slot, button.clone());
            Ok(())
        })?;
        self.rules
            .command_sets
            .insert(&name, slots.into_values().collect());
        Ok(BlockStatus::Loaded)
    }

    fn note_stub_modules(&mut self, lines: &[&[u8]]) {
        for line in lines {
            let Ok(s) = std::str::from_utf8(line) else {
                continue;
            };
            let s = s.split(';').next().unwrap_or("");
            let mut words = s
                .split(|c: char| c <= ' ' || c == '=')
                .filter(|w| !w.is_empty());
            let mod_type = match words.next() {
                Some("Behavior" | "Body") => ModuleType::Behavior,
                Some("Draw") => ModuleType::Draw,
                Some("ClientUpdate") => ModuleType::ClientUpdate,
                _ => continue,
            };
            let Some(module) = words.next() else { continue };
            let parsed = self
                .rules
                .module_registry
                .find_descriptor(module, mod_type)
                .is_some_and(|d| d.parse_data.is_some());
            if !parsed {
                let kind = match mod_type {
                    ModuleType::Behavior => "logic",
                    ModuleType::Draw => "draw",
                    _ => "client",
                };
                *self
                    .report
                    .stub_modules
                    .entry(format!("{kind}:{module}"))
                    .or_insert(0) += 1;
            }
        }
    }

    /// Distinct names of the files read.
    pub fn distinct_files(&self) -> usize {
        self.report.files.iter().collect::<BTreeSet<_>>().len()
    }
}

/// `None` names no template or upgrade.
fn name_or_none(token: &str) -> String {
    if token.eq_ignore_ascii_case("None") {
        String::new()
    } else {
        token.to_owned()
    }
}

/// The next line at or after `from` that starts a block in column 0.
fn next_header(lines: &[&[u8]], from: usize) -> usize {
    (from..lines.len())
        .find(|&j| {
            lines[j].first().is_some_and(u8::is_ascii_alphabetic)
                && first_token(lines[j]).and_then(block_type).is_some()
        })
        .unwrap_or(lines.len())
}
