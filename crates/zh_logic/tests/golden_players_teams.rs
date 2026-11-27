//! `PlayerList`, `Team`, `TeamPrototype` and `TeamFactory` against the reference
//! library (`tests/cpp_harness/gen_players_teams.cpp`).
//!
//! The vectors are a script: rows without a leading `=` drive the game, `=` rows
//! must be reproduced exactly.
//!
//! `retail_maps_match` is local only: it builds the players and teams of every
//! map of an installed copy.

use std::collections::{BTreeMap, BTreeSet};

use zh_common::core::dict::Dict;
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::io::data_chunk::{
    CachedFileInputStream, DataChunkError, DataChunkInfo, DataChunkInput,
};
use zh_common::io::{BigFileSystem, FileInfo};
use zh_common::math::{Coord3D, ICoord2D};
use zh_common::types::{Int, ObjectId, Real, Relationship, TeamId, UnsignedInt, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferCrc, XferError, XferLoad, XferSave};

use zh_logic::player::player::{Player, SpecialPowerReadyTimer};
use zh_logic::player::player_list::PlayerList;
use zh_logic::player::seams::{
    AiKind, PlayerEnv, PlayerIndex, PlayerPart, PlayerWorld, SkirmishScripts,
};
use zh_logic::player::team::{RelationMap, TeamFactory};
use zh_logic::rules::kind_of::KindOfMaskType;
use zh_logic::rules::player_template::PlayerTemplateStore;
use zh_logic::rules::rank_info::RankInfoStore;
use zh_logic::rules::science::{ScienceStore, ScienceTextLookup, ScienceType};
use zh_logic::rules::upgrade::UpgradeMaskType;
use zh_logic::spatial::sides_list::{
    BuildListInfo, SidesList, SidesScriptList, WellKnownSidesKeys,
};
use zh_logic::terrain::TerrainLogic;

const VECTORS: &str = include_str!("../../../docs/golden_vectors/PlayersTeams.json");

const RETAIL: &str = include_str!("../../../docs/golden_vectors/PlayersTeamsRetail.json");

const FNV: u64 = 0xcbf2_9ce4_8422_2325;

fn fnv(mut hash: u64, bytes: &[u8]) -> u64 {
    for &b in bytes {
        hash ^= u64::from(b);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    hash
}

fn rows(text: &str) -> impl Iterator<Item = &str> {
    text.lines().filter_map(|l| {
        let l = l.strip_prefix("  \"")?;
        l.strip_suffix("\",").or_else(|| l.strip_suffix('"'))
    })
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(text: &str) -> Vec<u8> {
    (0..text.len() / 2)
        .map(|i| u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).expect("hex"))
        .collect()
}

fn digest(text: &str, full: bool) -> String {
    if full {
        text.to_owned()
    } else {
        format!("{}:{:016x}", text.len() / 2, fnv(FNV, text.as_bytes()))
    }
}

fn num(text: &str) -> i64 {
    text.parse().unwrap_or_else(|_| panic!("number {text:?}"))
}

fn bits(text: &str) -> Real {
    Real::from_bits(u32::from_str_radix(text, 16).expect("bits"))
}

fn relationship(text: &str) -> Relationship {
    Relationship::from_i32(num(text) as i32)
}

struct NoText;

impl ScienceTextLookup for NoText {
    fn fetch_label(&mut self, _label: &str) -> String {
        "x".to_owned()
    }
}

#[derive(Default)]
struct Rules {
    sciences: ScienceStore,
    ranks: RankInfoStore,
    templates: PlayerTemplateStore,
}

impl Default for Host {
    fn default() -> Self {
        Self {
            frame: 0,
            cash: 0,
            slots: 0,
            network: false,
            difficulty: 1,
            force_skirmish_ai: false,
            general_name: Vec::new(),
            waypoints: Vec::new(),
            blobs: BTreeMap::new(),
            dead: BTreeSet::new(),
            events: Vec::new(),
            sighting: BTreeSet::new(),
            ai_idle: BTreeMap::new(),
            scripts: false,
            conditions: false,
            one_shot: false,
            upgrades: Vec::new(),
        }
    }
}

/// The world of the vectors: scripts resolve to nothing, AI and the other
/// unported systems answer with the bytes the reference wrote.
struct Host {
    frame: UnsignedInt,
    cash: UnsignedInt,
    slots: u32,
    network: bool,
    difficulty: Int,
    force_skirmish_ai: bool,
    general_name: Vec<u16>,
    /// `getFirstWaypoint` order.
    waypoints: Vec<(u32, String, Coord3D)>,
    blobs: BTreeMap<(PlayerIndex, &'static str), Vec<u8>>,
    dead: BTreeSet<ObjectId>,
    events: Vec<String>,
    /// Members that see an enemy, and the idle state of members with an AI.
    sighting: BTreeSet<ObjectId>,
    ai_idle: BTreeMap<ObjectId, bool>,
    /// Whether scripts exist, whether their conditions hold and run once.
    scripts: bool,
    conditions: bool,
    one_shot: bool,
    /// `firstUpgradeTemplate` order, with each template's mask bit.
    upgrades: Vec<(String, Int)>,
}

impl PlayerWorld for Host {
    fn frame(&self) -> UnsignedInt {
        self.frame
    }
    fn starting_cash(&self) -> UnsignedInt {
        self.cash
    }
    fn is_network(&self) -> bool {
        self.network
    }
    fn game_slot_occupied(&self, slot: usize) -> bool {
        self.slots >> slot & 1 != 0
    }
    fn global_difficulty(&self) -> Int {
        self.difficulty
    }
    fn force_skirmish_ai(&self) -> bool {
        self.force_skirmish_ai
    }
    fn general_name(&self) -> Vec<u16> {
        self.general_name.clone()
    }
    fn waypoint_location(&self, name: &str) -> Option<Coord3D> {
        self.waypoints
            .iter()
            .filter(|w| w.1 == name)
            .last()
            .map(|w| w.2)
    }
    fn waypoint_exists(&self, id: UnsignedInt) -> bool {
        self.waypoints.iter().any(|w| w.0 == id)
    }
    fn becoming_local_player(&mut self, player: PlayerIndex, yes: bool) {
        self.events.push(format!("local {player} {yes}"));
    }
    fn local_player_update(&mut self, player: PlayerIndex) {
        self.events.push(format!("local_update {player}"));
    }
    fn create_ai(&mut self, player: PlayerIndex, kind: AiKind) {
        self.events.push(format!("create_ai {player} {kind:?}"));
    }
    fn delete_ai(&mut self, player: PlayerIndex) {
        self.events.push(format!("delete_ai {player}"));
    }
    fn set_ai_difficulty(&mut self, player: PlayerIndex, difficulty: Int) {
        self.events
            .push(format!("difficulty {player} {difficulty}"));
    }
    fn ai_update(&mut self, player: PlayerIndex) {
        self.events.push(format!("ai_update {player}"));
    }
    fn ai_new_map(&mut self, player: PlayerIndex) {
        self.events.push(format!("ai_new_map {player}"));
    }
    fn ai_pre_team_destroy(&mut self, player: PlayerIndex, team: TeamId) {
        self.events
            .push(format!("ai_pre_destroy {player} {}", team.0));
    }
    fn qualify_skirmish_scripts(&mut self, sides: &mut SidesList, copy: &SkirmishScripts<'_>) {
        self.events.push(format!(
            "scripts {} {} {} {} {}",
            copy.side,
            copy.skirmish_side,
            copy.qualifier,
            copy.template_player_name,
            copy.player_name
        ));
        if let Ok(side) = sides.side_info_mut(copy.side) {
            side.scripts = Some(SidesScriptList::default());
        }
    }
    fn create_economy_trackers(&mut self, player: PlayerIndex) {
        self.events.push(format!("trackers {player}"));
    }
    fn delete_economy_trackers(&mut self, player: PlayerIndex) {
        self.events.push(format!("no_trackers {player}"));
    }
    fn academy_init(&mut self, player: PlayerIndex) {
        self.events.push(format!("academy_init {player}"));
    }
    fn academy_update(&mut self, player: PlayerIndex) {
        self.events.push(format!("academy_update {player}"));
    }
    fn academy_record_income(&mut self, player: PlayerIndex) {
        self.events.push(format!("income {player}"));
    }
    fn power_brownout_changed(&mut self, player: PlayerIndex, brownout: bool) {
        self.events.push(format!("brownout {player} {brownout}"));
    }
    fn science_added(&mut self, player: PlayerIndex, science: ScienceType) {
        self.events.push(format!("science {player} {science}"));
    }
    fn upgrade_count(&self, _player: PlayerIndex) -> u16 {
        0
    }
    fn delete_upgrades(&mut self, _player: PlayerIndex) {}
    fn upgrade_masks(&self) -> Vec<(String, UpgradeMaskType)> {
        self.upgrades
            .iter()
            .map(|(name, bit)| {
                let mut mask = UpgradeMaskType::default();
                mask.set(*bit, 1);
                (name.clone(), mask)
            })
            .collect()
    }
    fn notify_team_destroyed(&mut self, team: TeamId) {
        self.events.push(format!("destroyed {}", team.0));
    }
    fn detach_member(&mut self, object: ObjectId) {
        self.events.push(format!("detach {}", object.0));
    }
    fn member_changed_player(&mut self, object: ObjectId, old: PlayerIndex, new: PlayerIndex) {
        self.events
            .push(format!("captured {} {old} {new}", object.0));
    }
    fn execute_team_create_actions(
        &mut self,
        _players: &mut PlayerList,
        _teams: &mut TeamFactory,
        condition: &str,
    ) {
        self.events.push(format!("create_actions {condition}"));
    }
    fn duplicate_script(&mut self, name: &str) -> Option<UnsignedInt> {
        if !self.scripts {
            return None;
        }
        self.events.push(format!("duplicate {name}"));
        Some(77)
    }
    fn evaluate_script(
        &mut self,
        _players: &mut PlayerList,
        _teams: &mut TeamFactory,
        script: UnsignedInt,
        team: TeamId,
    ) -> bool {
        self.events.push(format!("evaluate {script} {}", team.0));
        self.conditions
    }
    fn script_is_one_shot(&self, _script: UnsignedInt) -> bool {
        self.one_shot
    }
    fn execute_script_action(
        &mut self,
        _players: &mut PlayerList,
        _teams: &mut TeamFactory,
        script: UnsignedInt,
        team: TeamId,
    ) {
        self.events.push(format!("execute {script} {}", team.0));
    }
    fn run_script(
        &mut self,
        _players: &mut PlayerList,
        _teams: &mut TeamFactory,
        name: &str,
        team: TeamId,
    ) {
        self.events.push(format!("run {name} {}", team.0));
    }
    fn member_is_effectively_dead(&self, object: ObjectId) -> bool {
        self.dead.contains(&object)
    }
    fn member_ai_idle(&self, object: ObjectId) -> Option<bool> {
        self.ai_idle.get(&object).copied()
    }
    fn member_sees_enemy(&mut self, object: ObjectId) -> bool {
        self.sighting.contains(&object)
    }
    fn xfer_player_part(
        &mut self,
        player: PlayerIndex,
        part: PlayerPart,
        xfer: &mut dyn Xfer,
    ) -> Result<(), XferError> {
        let tag = match part {
            PlayerPart::Upgrades => return Ok(()),
            PlayerPart::Ai => "ai",
            PlayerPart::ResourceGathering => "rgm",
            PlayerPart::TunnelTracker => "tunnel",
        };
        let mut bytes = self
            .blobs
            .get(&(player, tag))
            .cloned()
            .ok_or(XferError::InvalidData)?;
        xfer.xfer_user(&mut bytes)
    }
}

struct Game {
    rules: Rules,
    keys: NameKeyGenerator,
    host: Host,
    sides: SidesList,
    list: PlayerList,
    teams: TeamFactory,
    /// Spawned objects, by spawn order.
    objects: Vec<ObjectId>,
    reloads: usize,
    refused: usize,
}

macro_rules! env {
    ($rules:expr, $keys:expr, $host:expr) => {
        &mut PlayerEnv {
            templates: &$rules.templates,
            ranks: &$rules.ranks,
            sciences: &$rules.sciences,
            name_keys: &mut $keys,
            world: &mut $host,
        }
    };
}

fn load_rules(rules: &mut Rules, keys: &mut NameKeyGenerator, blocks: &[Vec<String>]) {
    for kind in ["Science", "Rank", "PlayerTemplate"] {
        let text: String = blocks
            .iter()
            .filter(|b| b[0].split_whitespace().next() == Some(kind))
            .map(|b| b.join("\n") + "\n")
            .collect();
        let mut ini = IniReader::new("rules.ini", text.as_bytes(), IniLoadType::Overwrite);
        match kind {
            "Science" => {
                rules
                    .sciences
                    .load_from_ini(&mut ini, keys, &mut NoText)
                    .expect("sciences");
            }
            "Rank" => {
                rules
                    .ranks
                    .load_from_ini(&mut ini, &rules.sciences, keys, &mut NoText)
                    .expect("ranks");
            }
            _ => rules
                .templates
                .load_from_ini(&mut ini, &rules.sciences, keys, &mut NoText)
                .expect("templates"),
        }
    }
}

fn dict(text: &str, keys: &mut NameKeyGenerator) -> Dict {
    let mut d = Dict::new();
    for field in text.split(';') {
        let mut parts = field.splitn(3, '|');
        let (name, kind, value) = (
            parts.next().expect("key"),
            parts.next().expect("type"),
            parts.next().expect("value"),
        );
        let key = keys.name_to_key(name);
        match kind {
            "s" => d.set_ascii_string(key, value),
            "b" => d.set_bool(key, if value == "1" { TRUE } else { FALSE }),
            "i" => d.set_int(key, num(value) as Int),
            "r" => d.set_real(key, bits(value)),
            "u" => d.set_unicode_string(key, &value.encode_utf16().collect::<Vec<_>>()),
            _ => panic!("type {kind}"),
        }
    }
    d
}

fn save(f: impl FnOnce(&mut XferSave) -> Result<(), XferError>) -> Vec<u8> {
    let mut x = XferSave::new();
    f(&mut x).expect("save");
    x.into_bytes()
}

/// The reference library keeps the retail save format, where `Money` (version 1)
/// ends after the amount. Without the `retail-xfer-save` feature the port writes
/// version 2 with the income buckets (checked by `Money`'s own vectors), which
/// this removes from one player's save.
fn as_reference(mut player: Vec<u8>) -> Vec<u8> {
    if !cfg!(feature = "retail-xfer-save") {
        assert_eq!(player[1], 2, "money version");
        player[1] = 1;
        player.drain(6..6 + 60 * 4 + 4);
    }
    player
}

fn on(b: bool) -> u8 {
    u8::from(b)
}

impl Game {
    fn new(blocks: &[Vec<String>], general_name: Vec<u16>) -> Self {
        let mut rules = Rules::default();
        let mut keys = NameKeyGenerator::new();
        load_rules(&mut rules, &mut keys, blocks);
        let mut host = Host {
            general_name,
            ..Host::default()
        };
        let list = PlayerList::new(env!(rules, keys, host));
        Self {
            rules,
            keys,
            host,
            sides: SidesList::new(),
            list,
            teams: TeamFactory::new(),
            objects: Vec::new(),
            reloads: 0,
            refused: 0,
        }
    }

    fn player(&self, i: PlayerIndex) -> &Player {
        self.list.player(i).expect("player")
    }

    fn live_teams(&self) -> Vec<TeamId> {
        (0..16)
            .flat_map(|i| self.player(i).team_prototypes.clone())
            .flat_map(|p| self.teams.instance_ids(p))
            .collect()
    }

    fn check_list(&self) -> String {
        let slots: String = (0..16)
            .map(|i| format!("{},", self.list.slot_index(i)))
            .collect();
        format!(
            "= list local={} count={} slots={slots}",
            self.list.local_player(),
            self.list.player_count()
        )
    }

    fn check_player(&mut self, i: PlayerIndex) -> String {
        let unit = Some(self.keys.name_to_key("PtUnit"));
        let p = self.player(i);
        let protos: String = p
            .team_prototypes
            .iter()
            .map(|id| format!("{id},"))
            .collect();
        format!(
            "= player {i} side={} base={} type={} color={:08x} night={:08x} money={} def={} pre={} obs={} dead={} mp={} keyfound={} mask={} retal={} cost={:08x} time={:08x} vet={} protos={protos}",
            p.side,
            p.base_side,
            p.player_type as i32,
            p.color as u32,
            p.night_color as u32,
            p.money.count_money(),
            p.default_team.map_or(0, |t| t.0),
            on(p.is_preorder),
            on(p.observer),
            on(p.is_dead),
            p.mp_start_index,
            self.list.find_player_with_name_key(p.name_key).unwrap_or(-1),
            p.mask(),
            on(p.logical_retaliation_mode),
            unit
                .and_then(|k| p.production_cost_changes.get(&k))
                .map_or(0, |v| v.to_bits()),
            unit
                .and_then(|k| p.production_time_changes.get(&k))
                .map_or(0, |v| v.to_bits()),
            unit
                .and_then(|k| p.production_veterancy_levels.get(&k))
                .map_or(0, |v| *v as i32),
        )
    }

    fn template_text(&self, id: u32) -> String {
        let t = self.teams.prototype(id).expect("prototype");
        let n = &t.template;
        let units: String = n
            .units
            .iter()
            .map(|u| format!("{}/{}/{},", u.min_units, u.max_units, u.unit_thing_name))
            .collect();
        let generic: String = n
            .generic_scripts
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.is_empty())
            .map(|(i, s)| format!("{i}:{s},"))
            .collect();
        let h = &n.home_location;
        format!(
            "{} owner={} single={} units={units} home={}:{:08x},{:08x},{:08x} s={}|{}|{}|{}|{}|{} idle={} thresh={:08x} f={}{}{}{}{}{}{}{}{}{} max={} pri={},{},{} att={} tr={} at={} vet={} cond={} g={generic}",
            t.name,
            t.owner,
            on(t.singleton),
            on(n.has_home_location),
            h.x.to_bits(),
            h.y.to_bits(),
            h.z.to_bits(),
            n.script_on_create,
            n.script_on_idle,
            n.script_on_enemy_sighted,
            n.script_on_all_clear,
            n.script_on_unit_destroyed,
            n.script_on_destroyed,
            n.initial_idle_frames,
            n.destroyed_threshold.to_bits(),
            on(n.is_ai_recruitable),
            on(n.is_base_defense),
            on(n.is_perimeter_defense),
            on(n.automatically_reinforce),
            on(n.transports_return),
            on(n.avoid_threats),
            on(n.attack_common_target),
            on(n.team_starts_full),
            on(n.transports_exit),
            on(n.execute_actions),
            n.max_instances,
            n.production_priority,
            n.production_priority_success_increase,
            n.production_priority_failure_decrease,
            n.initial_team_attitude,
            n.transport_unit_type,
            n.start_reinforce_waypoint,
            n.veterancy,
            n.production_condition,
        )
    }

    fn check_sides(&mut self) -> String {
        let name = self.keys.name_to_key("teamName");
        let owner = self.keys.name_to_key("teamOwner");
        let create = self.keys.name_to_key("teamOnCreateScript");
        let hook = self.keys.name_to_key("teamGenericScriptHook3");
        let teams: String = self
            .sides
            .team_rec
            .teams
            .iter()
            .map(|t| {
                let get = |k| t.dict.get_ascii_string(k, None);
                format!(
                    "{}:{}:{}:{},",
                    get(name),
                    get(owner),
                    get(create),
                    get(hook)
                )
            })
            .collect();
        let flags = |v: Vec<bool>| {
            v.into_iter()
                .map(|b| if b { '1' } else { '0' })
                .collect::<String>()
        };
        format!(
            "= sides teams={teams} scripts={} skscripts={} build={}",
            flags(
                self.sides
                    .sides
                    .iter()
                    .map(|s| s.scripts.is_some())
                    .collect()
            ),
            flags(
                self.sides
                    .skirmish_sides
                    .iter()
                    .map(|s| s.scripts.is_some())
                    .collect()
            ),
            flags(
                self.sides
                    .sides
                    .iter()
                    .map(|s| !s.build_list.is_empty())
                    .collect()
            ),
        )
    }

    fn check_instances(&self) -> String {
        let mut out = String::new();
        for i in 0..16 {
            for &p in &self.player(i).team_prototypes {
                out += &format!("{p}:");
                for t in self.teams.instance_ids(p) {
                    out += &format!("{},", t.0);
                }
                out.push(';');
            }
        }
        format!("= inst {out}")
    }

    fn check_masks(&self) -> String {
        let mut out = String::new();
        for source in -1..=self.list.player_count() {
            for allowed in 0..16 {
                let mask = self
                    .list
                    .players_with_relationship(source, allowed, &self.teams);
                out += &format!("{mask:x}.");
            }
        }
        format!("= masks {}", digest(&out, false))
    }

    fn check_relations(&self) -> String {
        let live = self.live_teams();
        let (mut tt, mut pt) = (String::new(), String::new());
        for &a in &live {
            for &b in &live {
                tt += &(self.teams.team_relationship(a, Some(b), &self.list) as i32).to_string();
            }
            tt += &(self.teams.team_relationship(a, None, &self.list) as i32).to_string();
            for i in 0..self.list.player_count() {
                let owner = self.teams.controlling_player(a);
                pt += &(self.player(i).relationship(Some(a), owner) as i32).to_string();
            }
        }
        format!(
            "= trel {} {} {}",
            live.len(),
            digest(&tt, false),
            digest(&pt, false)
        )
    }

    fn player_bytes(&mut self, i: PlayerIndex) -> Vec<u8> {
        let (rules, keys, host, list) =
            (&self.rules, &mut self.keys, &mut self.host, &mut self.list);
        let teams = &self.teams;
        save(|x| {
            list.player_mut(i)
                .expect("player")
                .xfer(x, teams, env!(rules, *keys, *host))
        })
    }

    fn list_bytes(&mut self) -> Vec<u8> {
        let (rules, keys, host, list) =
            (&self.rules, &mut self.keys, &mut self.host, &mut self.list);
        let teams = &self.teams;
        save(|x| list.xfer(x, teams, env!(rules, *keys, *host)))
    }

    /// `PlayerList::xfer` is its version, the count and each player.
    fn reference_list_bytes(&mut self) -> Vec<u8> {
        let bytes = self.list_bytes();
        let count = self.list.player_count();
        let mut head = vec![1];
        head.extend(count.to_le_bytes());
        let (mut whole, mut reference) = (head.clone(), head);
        for i in 0..count {
            let player = self.player_bytes(i);
            whole.extend(&player);
            reference.extend(as_reference(player));
        }
        assert_eq!(bytes, whole, "player list save");
        reference
    }

    fn team_bytes(&mut self) -> Vec<u8> {
        let (teams, host) = (&mut self.teams, &self.host);
        save(|x| teams.xfer(x, host))
    }

    fn crc(&self) -> u32 {
        let mut x = XferCrc::new();
        self.list.crc(&mut x).expect("crc");
        x.get_crc()
    }

    /// Loading what was saved restores the same bytes over a changed state.
    fn check_reload(&mut self) {
        // A prototype whose name was taken cannot be found again by id: the
        // original refuses such a save too. Round trip without those.
        let untouched = self.list.clone();
        let mut duplicate = false;
        for i in 0..16 {
            let teams = &self.teams;
            let p = self.list.player_mut(i).expect("player");
            let before = p.team_prototypes.len();
            p.team_prototypes
                .retain(|&id| teams.find_team_prototype_by_id(id).is_some());
            duplicate |= before != p.team_prototypes.len() && i < untouched.player_count();
        }
        if duplicate {
            let stripped = std::mem::replace(&mut self.list, untouched.clone());
            let bytes = self.list_bytes();
            let (rules, keys, host) = (&self.rules, &mut self.keys, &mut self.host);
            let loaded = self.list.xfer(
                &mut XferLoad::new(&bytes),
                &self.teams,
                env!(rules, *keys, *host),
            );
            assert_eq!(loaded, Err(XferError::InvalidData), "duplicate prototype");
            self.refused += 1;
            self.list = stripped;
        }
        self.reloads += 1;
        let (list_bytes, team_bytes) = (self.list_bytes(), self.team_bytes());
        let teams = self.teams.clone();
        for i in 0..self.list.player_count() {
            let p = self.list.player_mut(i).expect("player");
            p.skill_points += 7;
            p.sciences.clear();
            p.team_prototypes.reverse();
            p.build_list.clear();
            p.default_team = None;
            // Loading appends to these and requires the last two to be empty.
            p.squads = Default::default();
            p.upgrades_completed.clear();
            p.production_changes.clear();
            p.special_power_timers.clear();
        }
        for id in self.live_teams() {
            if let Some(t) = self.teams.team_mut(id) {
                t.state = "changed".to_owned();
                t.destroy_threshold += 9;
            }
        }
        let mut x = XferLoad::new(&team_bytes);
        self.teams.xfer(&mut x, &self.host).expect("load teams");
        let (rules, keys, host) = (&self.rules, &mut self.keys, &mut self.host);
        let mut x = XferLoad::new(&list_bytes);
        self.list
            .xfer(&mut x, &self.teams, env!(rules, *keys, *host))
            .expect("load players");
        self.teams.load_post_process_teams().expect("members");
        assert_eq!(self.list_bytes(), list_bytes, "player reload");
        assert_eq!(self.team_bytes(), team_bytes, "team reload");
        // Loading rebuilds the relation maps in another order, so carry on from
        // the state before the save.
        self.list = untouched;
        self.teams = teams;
    }

    /// `Object::setTeam` gives the objects of a dead or observing player to the
    /// neutral player.
    fn team_for_object(&self, team: TeamId) -> TeamId {
        let owner = self.player(self.teams.controlling_player(team).expect("team"));
        if owner.observer || owner.is_dead {
            self.player(0).default_team.expect("neutral team")
        } else {
            team
        }
    }

    fn remove_from_team(&mut self, object: ObjectId) -> Option<TeamId> {
        let team = self.live_teams().into_iter().find(|&t| {
            self.teams
                .team(t)
                .is_some_and(|t| t.members.contains(&object))
        })?;
        self.teams.team_mut(team)?.remove_member(object);
        Some(team)
    }

    /// One `op` row; returns what the following `=` row should say.
    fn apply(&mut self, words: &[&str]) -> Option<String> {
        let arg = |i: usize| num(words[i]);
        let team = |i: usize| TeamId(num(words[i]) as i32);
        let player = |i: usize| num(words[i]) as PlayerIndex;
        let done = |b: bool| Some(format!("= done {}", on(b)));
        let made = |t: Option<TeamId>| Some(format!("= team {}", t.map_or(0, |t| t.0)));
        let (rules, keys, host) = (&self.rules, &mut self.keys, &mut self.host);
        match words[0] {
            "prel" => {
                let p = self.list.player_mut(player(1)).expect("player");
                p.set_player_relationship(player(2), relationship(words[3]));
            }
            "prem" => {
                let p = self.list.player_mut(player(1)).expect("player");
                let that = Some(player(2)).filter(|&t| t >= 0);
                return done(p.remove_player_relationship(that));
            }
            "trel" => {
                let p = self.list.player_mut(player(1)).expect("player");
                p.set_team_relationship(team(2), relationship(words[3]));
            }
            "trem" => {
                let p = self.list.player_mut(player(1)).expect("player");
                let that = Some(team(2)).filter(|&t| t != TeamId::INVALID);
                return done(p.remove_team_relationship(that));
            }
            "ovt" => self
                .teams
                .team_mut(team(1))
                .expect("team")
                .set_override_team_relationship(team(2), relationship(words[3])),
            "ovtr" => {
                let t = self.teams.team_mut(team(1)).expect("team");
                return done(t.remove_override_team_relationship(team(2)));
            }
            "ovp" => self
                .teams
                .team_mut(team(1))
                .expect("team")
                .set_override_player_relationship(player(2), relationship(words[3])),
            "ovpr" => {
                let t = self.teams.team_mut(team(1)).expect("team");
                return done(t.remove_override_player_relationship(player(2)));
            }
            "make" => {
                let name = words[1];
                let mut t = None;
                if self.teams.find_team_prototype(name, keys).is_some() {
                    let env = env!(rules, *keys, *host);
                    t = if arg(2) != 0 {
                        self.teams.create_team(name, &mut self.list, env)
                    } else {
                        self.teams.create_inactive_team(name, &mut self.list, env)
                    };
                }
                return made(t);
            }
            "find" => {
                let env = env!(rules, *keys, *host);
                return made(self.teams.find_team(words[1], &mut self.list, env));
            }
            "onproto" => {
                let proto = self.teams.find_team_prototype_by_id(arg(1) as u32);
                return made(proto.and_then(|p| self.teams.create_team_on_prototype(p)));
            }
            "move" => {
                let env = env!(rules, *keys, *host);
                self.teams
                    .set_controlling_player(team(1), player(2), &mut self.list, env);
            }
            "spawn" => {
                let target = self.team_for_object(team(3));
                let object = ObjectId(arg(2) as i32);
                assert_eq!(arg(1) as usize, self.objects.len());
                self.objects.push(object);
                self.teams
                    .team_mut(target)
                    .expect("team")
                    .add_member(object);
            }
            "setteam" => {
                let object = self.objects[arg(1) as usize];
                let target = self.team_for_object(team(2));
                let already = self
                    .teams
                    .team(target)
                    .is_some_and(|t| t.members.contains(&object));
                if !already {
                    self.remove_from_team(object);
                    self.teams
                        .team_mut(target)
                        .expect("team")
                        .add_member(object);
                }
            }
            "dead" => {
                self.host.dead.insert(ObjectId(arg(1) as i32));
            }
            "set" => {
                let t = self.teams.team_mut(team(1)).expect("team");
                match arg(2) {
                    0 => t.state = words[3].to_owned(),
                    1 => t.set_recruitable(arg(4) != 0),
                    2 => t.set_active(),
                    _ => t.entered_or_exited = true,
                }
            }
            "delete" => {
                let env = env!(rules, *keys, *host);
                self.teams
                    .delete_team_instance(team(1), &mut self.list, env);
            }
            "sabotage" => self
                .list
                .player_mut(player(1))
                .expect("player")
                .energy
                .set_power_sabotaged_till_frame(arg(2) as u32),
            "income" => self
                .list
                .player_mut(player(1))
                .expect("player")
                .money
                .deposit(
                    arg(2) as u32,
                    FALSE,
                    TRUE,
                    &mut zh_logic::economy::money::NoopMoneyCallbacks,
                ),
            "spp" => {
                let p = self.list.player_mut(player(1)).expect("player");
                p.science_purchase_points = (p.science_purchase_points + arg(2) as Int).max(0);
            }
            "bonus" => {
                let mut b = zh_logic::modules::update::battle_plan_update::BattlePlanBonuses {
                    armor_scalar: bits(words[2]),
                    sight_range_scalar: bits(words[3]),
                    bombardment: arg(4) as Int,
                    hold_the_line: arg(5) as Int,
                    search_and_destroy: arg(6) as Int,
                    ..Default::default()
                };
                b.valid_kind_of = KindOfMaskType::default();
                b.invalid_kind_of = KindOfMaskType::default();
                b.valid_kind_of.set(arg(7) as Int, 1);
                b.invalid_kind_of.set(arg(8) as Int, 1);
                self.list
                    .player_mut(player(1))
                    .expect("player")
                    .battle_plan_bonuses = Some(b);
            }
            "poke" => {
                let p = self.list.player_mut(player(1)).expect("player");
                let value = arg(3) as Int;
                match arg(2) {
                    0 => p.cash_bounty_percent = value as Real / 7.0,
                    1 => p.skill_points_modifier = value as Real / 3.0,
                    2 => p.list_in_score_screen = value & 1 != 0,
                    3 => p.attacked_by[value as usize] = true,
                    4 => p.units_should_hunt = value & 1 != 0,
                    _ => p.logical_retaliation_mode = value & 1 != 0,
                }
            }
            "kindof" => {
                let p = self.list.player_mut(player(1)).expect("player");
                let mut kind_of = KindOfMaskType::default();
                kind_of.set(arg(2) as Int, 1);
                let percent = bits(words[3]);
                p.add_production_cost_change(kind_of, percent);
            }
            "timer" => self
                .list
                .player_mut(player(1))
                .expect("player")
                .special_power_timers
                .push(SpecialPowerReadyTimer {
                    template_id: arg(2) as u32,
                    ready_frame: arg(3) as u32,
                }),
            "upgrade" => {
                let masks = host.upgrade_masks();
                let mask = &masks.iter().find(|u| u.0 == words[3]).expect("upgrade").1;
                let p = self.list.player_mut(player(1)).expect("player");
                if arg(2) != 0 {
                    p.upgrades_completed.set_flags(mask);
                } else {
                    p.upgrades_in_progress.set_flags(mask);
                }
            }
            "squad" => self
                .list
                .player_mut(player(1))
                .expect("player")
                .squads
                .hotkeys[arg(2) as usize]
                .ids
                .push(arg(3) as u32),
            "avail" => {
                let science = rules
                    .sciences
                    .get_science_from_internal_name(words[2], keys);
                let p = self.list.player_mut(player(1)).expect("player");
                p.set_science_availability(science, arg(3) as Int);
            }
            "mask" => {
                let mut mask = arg(1) as u16;
                let each = (mask != 0)
                    .then(|| self.list.each_player_from_mask(&mut mask))
                    .flatten();
                let one = self.list.player_from_mask(arg(2) as u16).expect("mask");
                return Some(format!("= mask {} {mask} {one}", each.unwrap_or(-1)));
            }
            "validate" => {
                let env = env!(rules, *keys, *host);
                return made(self.list.validate_team(words[1], &mut self.teams, env));
            }
            "update_teams" => {
                let env = env!(rules, *keys, *host);
                self.list.update_team_states(&mut self.teams, env);
            }
            "update" => {
                host.frame = arg(1) as u32;
                let env = env!(rules, *keys, *host);
                self.list.update(&mut self.teams, env);
            }
            "postprocess" => self.teams.load_post_process(),
            other => panic!("op {other}"),
        }
        None
    }
}

fn rule_blocks(all: &[&str]) -> Vec<Vec<String>> {
    let mut blocks: Vec<Vec<String>> = vec![Vec::new()];
    for line in all.iter().filter_map(|r| r.strip_prefix("INI ")) {
        blocks.last_mut().expect("block").push(line.to_owned());
        if line.trim() == "End" {
            blocks.push(Vec::new());
        }
    }
    blocks.retain(|b| !b.is_empty());
    blocks
}

#[test]
fn matches_reference_vectors() {
    let all: Vec<&str> = rows(VECTORS).collect();
    assert!(all.len() > 30_000, "{} rows", all.len());
    let blocks = rule_blocks(&all);
    let general = all
        .iter()
        .find_map(|r| r.strip_prefix("GENERAL "))
        .expect("general name");
    let general: Vec<u16> = if general == "-" {
        Vec::new()
    } else {
        (0..general.len() / 4)
            .map(|i| u16::from_str_radix(&general[i * 4..i * 4 + 4], 16).expect("utf16"))
            .collect()
    };

    let mut game = Game::new(&blocks, general);
    let mut case = 0;
    let mut pending: Option<String> = None;
    let mut checked = 0;
    let mut map = RelationMap::default();
    for (n, row) in all.iter().enumerate() {
        let (tag, rest) = row.split_once(' ').unwrap_or((row, ""));
        let words: Vec<&str> = rest.split(' ').collect();
        let expect = |got: String| {
            assert_eq!(&got, row, "case {case}, row {n}");
        };
        match tag {
            "INI" | "GENERAL" => {}
            "UPG" => game
                .host
                .upgrades
                .push((words[0].to_owned(), num(words[1]) as Int)),
            "G" => {
                case = num(rest);
                game.host.waypoints.clear();
                game.host.events.clear();
                game.host.dead.clear();
            }
            "INFO" => {
                game.host.slots = num(words[0]) as u32;
                game.host.cash = num(words[1]) as u32;
                game.host.frame = num(words[2]) as u32;
                game.host.network = num(words[3]) != 0;
                game.host.difficulty = num(words[4]) as Int;
                // The sides of the previous case go after its objects.
                game.sides = SidesList::new();
            }
            "AIDATA" => game.host.force_skirmish_ai = num(rest) != 0,
            "WP" => game.host.waypoints.push((
                num(words[0]) as u32,
                words[1].to_owned(),
                Coord3D::new(bits(words[2]), bits(words[3]), bits(words[4])),
            )),
            "S" => {
                let d = dict(rest, &mut game.keys);
                game.sides.add_side(Some(&d));
            }
            "SK" => {
                let d = dict(rest, &mut game.keys);
                game.sides.add_skirmish_side(Some(&d));
            }
            "SS" => {
                game.sides.skirmish_sides[num(rest) as usize].scripts =
                    Some(SidesScriptList::default());
            }
            "T" => {
                let d = dict(rest, &mut game.keys);
                game.sides.add_team(Some(&d));
            }
            "ST" => {
                let d = dict(rest, &mut game.keys);
                game.sides.add_skirmish_team(Some(&d));
            }
            "B" => {
                let mut info = BuildListInfo::new();
                info.template_name = words[1].to_owned();
                info.location = Coord3D::new(bits(words[2]), bits(words[3]), 0.0);
                info.angle = bits(words[4]);
                info.num_rebuilds = num(words[5]) as u32;
                game.sides.sides[num(words[0]) as usize]
                    .build_list
                    .push(info);
            }
            "newgame" => {
                let (rules, keys, host) = (&game.rules, &mut game.keys, &mut game.host);
                game.list
                    .new_game(&mut game.sides, &mut game.teams, env!(rules, *keys, *host));
                game.objects.clear();
            }
            "BLOB" => {
                let tag = match words[1] {
                    "ai" => "ai",
                    "rgm" => "rgm",
                    _ => "tunnel",
                };
                game.host
                    .blobs
                    .insert((num(words[0]) as PlayerIndex, tag), unhex(words[2]));
            }
            "op" => pending = game.apply(&words),
            "RM" => match words[0] {
                "new" => map = RelationMap::default(),
                "clear" => map.clear(),
                "remove" => {
                    map.remove(num(words[1]) as u32);
                }
                _ => map.set(num(words[1]) as u32, relationship(words[2])),
            },
            "=" => {
                checked += 1;
                let full = case < 2;
                match words[0] {
                    "rm" => {
                        let bytes = save(|x| map.xfer(x));
                        let full = !words[1].contains(':');
                        expect(format!("= rm {}", digest(&hex(&bytes), full)));
                    }
                    "list" => expect(game.check_list()),
                    "player" => expect(game.check_player(num(words[1]) as PlayerIndex)),
                    "tpl" => {
                        let id = num(words[1]) as u32;
                        expect(format!(
                            "= tpl {id} {}",
                            digest(&game.template_text(id), full)
                        ));
                    }
                    "sides" => expect(game.check_sides()),
                    "inst" => {
                        expect(game.check_instances());
                    }
                    "masks" => expect(game.check_masks()),
                    "trel" => expect(game.check_relations()),
                    "xp" => {
                        let i = num(words[1]) as PlayerIndex;
                        // Only the checks right after `newGame` are written out.
                        let full = !words[2].contains(':');
                        let bytes = as_reference(game.player_bytes(i));
                        expect(format!("= xp {i} {}", digest(&hex(&bytes), full)));
                    }
                    "xl" => {
                        let bytes = game.reference_list_bytes();
                        expect(format!("= xl {}", digest(&hex(&bytes), false)));
                    }
                    "xt" => {
                        let full = !words[1].contains(':');
                        let bytes = game.team_bytes();
                        expect(format!("= xt {}", digest(&hex(&bytes), full)));
                    }
                    "crc" => {
                        expect(format!("= crc {:08x}", game.crc()));
                        if n % 7 == 0 {
                            game.check_reload();
                        }
                    }
                    _ => expect(pending.take().expect("an op that answers")),
                }
            }
            other => panic!("row {other}"),
        }
    }
    assert_eq!(case, 219, "cases");
    assert!(checked > 15_000, "{checked} checks");
    assert!(game.reloads > 60, "{} reloads", game.reloads);
    assert!(game.refused > 0, "no duplicate prototype case");
}

// ---- calls out -----------------------------------------------------------------
// The reference runs no scripts or AI in the vectors, so the calls made into
// them are checked here against the order of `Team.cpp` and `Player.cpp`.

impl Game {
    fn run(&mut self, f: impl FnOnce(&mut PlayerList, &mut TeamFactory, &mut PlayerEnv<'_>)) {
        let (rules, keys, host) = (&self.rules, &mut self.keys, &mut self.host);
        f(&mut self.list, &mut self.teams, env!(rules, *keys, *host));
    }

    /// The events since the last call, without the academy and money chatter.
    fn events(&mut self) -> Vec<String> {
        std::mem::take(&mut self.host.events)
            .into_iter()
            .filter(|e| !e.starts_with("academy_"))
            .collect()
    }
}

/// A neutral side, a human `A` and a computer `B`, with a non-singleton team
/// `Band` of `A` that has every team script.
fn small_game() -> Game {
    let all: Vec<&str> = rows(VECTORS).collect();
    let mut game = Game::new(&rule_blocks(&all), Vec::new());
    for side in [
        "playerName|s|;playerFaction|s|",
        "playerName|s|A;playerFaction|s|FactionAmerica;playerIsHuman|b|1;playerAllies|s|B",
        "playerName|s|B;playerFaction|s|FactionChina;playerEnemies|s|A",
    ] {
        let d = dict(side, &mut game.keys);
        game.sides.add_side(Some(&d));
    }
    for team in [
        "teamName|s|team;teamOwner|s|;teamIsSingleton|b|1",
        "teamName|s|teamA;teamOwner|s|A;teamIsSingleton|b|1",
        "teamName|s|teamB;teamOwner|s|B;teamIsSingleton|b|1",
        "teamName|s|Band;teamOwner|s|A;teamOnCreateScript|s|c;teamEnemySightedScript|s|s;\
         teamAllClearScript|s|a;teamOnIdleScript|s|i;teamOnDestroyedScript|s|d;\
         teamDestroyedThreshold|r|3f000000;teamOnUnitDestroyedScript|s|u;\
         teamGenericScriptHook2|s|g;teamExecutesActionsOnCreate|b|1;\
         teamProductionCondition|s|cond;teamProductionPriority|i|10;\
         teamProductionPrioritySuccessIncrease|i|4;teamProductionPriorityFailureDecrease|i|3",
    ] {
        let d = dict(team, &mut game.keys);
        game.sides.add_team(Some(&d));
    }
    let (rules, keys, host) = (&game.rules, &mut game.keys, &mut game.host);
    game.list
        .new_game(&mut game.sides, &mut game.teams, env!(rules, *keys, *host));
    game
}

#[test]
fn game_start_calls() {
    let mut game = small_game();
    let events = game.events();
    let at = |e: &str| {
        events
            .iter()
            .position(|x| x == e)
            .unwrap_or_else(|| panic!("no {e} in {events:?}"))
    };
    assert!(at("trackers 1") < at("local 1 true"));
    assert!(at("local 0 false") < at("local 1 true"));
    assert!(at("create_ai 2 Standard") < at("trackers 2"));
    assert!(!events.iter().any(|e| e.starts_with("create_ai 1")));
    assert_eq!(game.list.local_player(), 1);
    assert_eq!(game.list.player_count(), 3);

    // A reset deletes the AI first, then the teams, then the players.
    game.run(|list, teams, env| list.reset(teams, env));
    let events = game.events();
    assert_eq!(events[0], "delete_ai 2");
    assert!(events[1].starts_with("destroyed "), "{events:?}");
    assert!(events.contains(&"no_trackers 1".to_owned()));
    assert_eq!(game.list.local_player(), 0);
    assert_eq!(game.list.player_count(), 1);
}

#[test]
fn team_script_calls() {
    let mut game = small_game();
    game.events();
    let mut band = TeamId::INVALID;
    game.run(|list, teams, env| band = teams.create_team("Band", list, env).expect("team"));
    let id = band.0;
    assert_eq!(game.events(), ["create_actions cond"]);
    for object in 1..=4 {
        game.teams
            .team_mut(band)
            .expect("team")
            .add_member(ObjectId(object));
    }
    let step = |game: &mut Game| {
        game.run(|list, teams, env| list.update_team_states(teams, env));
        game.events()
    };

    assert_eq!(step(&mut game), [format!("run c {id}")]);
    assert_eq!(game.teams.team(band).expect("team").destroy_threshold, 2);
    assert_eq!(game.teams.team(band).expect("team").cur_units, 4);

    game.host.sighting.insert(ObjectId(2));
    assert_eq!(step(&mut game), [format!("run s {id}")]);
    assert_eq!(step(&mut game), Vec::<String>::new());
    game.host.sighting.clear();
    assert_eq!(step(&mut game), [format!("run a {id}")]);

    // Idle must hold on two passes; members without AI do not count.
    game.host.ai_idle.insert(ObjectId(1), false);
    assert_eq!(step(&mut game), Vec::<String>::new());
    game.host.ai_idle.insert(ObjectId(1), true);
    assert_eq!(step(&mut game), Vec::<String>::new());
    assert_eq!(step(&mut game), [format!("run i {id}")]);
    game.host.ai_idle.clear();

    game.host.dead.insert(ObjectId(1));
    assert_eq!(step(&mut game), Vec::<String>::new());
    game.host.dead.insert(ObjectId(2));
    assert_eq!(step(&mut game), [format!("run d {id}")]);
    game.host.dead.insert(ObjectId(3));
    assert_eq!(step(&mut game), Vec::<String>::new());

    // All dead: no all clear.
    game.host.sighting.insert(ObjectId(4));
    assert_eq!(step(&mut game), [format!("run s {id}")]);
    game.host.dead.insert(ObjectId(4));
    assert_eq!(step(&mut game), Vec::<String>::new());

    game.run(|list, teams, env| teams.notify_team_of_object_death(band, list, env));
    assert_eq!(game.events(), [format!("run u {id}")]);

    let prototype = game.teams.team(band).expect("team").prototype;
    let p = game.teams.prototype_mut(prototype).expect("prototype");
    p.increase_ai_priority_for_success();
    assert_eq!(p.template.production_priority, 14);
    p.decrease_ai_priority_for_failure();
    p.decrease_ai_priority_for_failure();
    assert_eq!(p.template.production_priority, 8);
    assert_eq!(p.count_team_instances(), 1);
}

#[test]
fn generic_script_and_update_calls() {
    for one_shot in [true, false] {
        let mut game = small_game();
        let mut band = TeamId::INVALID;
        game.run(|list, teams, env| band = teams.create_team("Band", list, env).expect("team"));
        let id = band.0;
        game.events();
        game.host.scripts = true;
        game.host.conditions = true;
        game.host.one_shot = one_shot;
        game.host.frame = 90;
        game.list
            .player_mut(1)
            .expect("player")
            .energy
            .set_power_sabotaged_till_frame(89);
        let step = |game: &mut Game| {
            game.run(|list, teams, env| list.update(teams, env));
            game.events()
        };
        let events = step(&mut game);
        assert_eq!(
            events,
            [
                "duplicate g".to_owned(),
                format!("evaluate 77 {id}"),
                format!("execute 77 {id}"),
                "brownout 1 false".to_owned(),
                "local_update 1".to_owned(),
                "ai_update 2".to_owned(),
            ]
        );
        let mut again = vec!["local_update 1".to_owned(), "ai_update 2".to_owned()];
        if !one_shot {
            again.insert(0, format!("execute 77 {id}"));
            again.insert(0, format!("evaluate 77 {id}"));
        }
        assert_eq!(step(&mut game), again);

        // A condition that fails leaves the script to be tried again.
        game.host.conditions = false;
        let tried = step(&mut game);
        assert_eq!(tried.contains(&format!("evaluate 77 {id}")), !one_shot);
        assert_eq!(step(&mut game), tried);
    }
}

#[test]
fn team_deletion_and_moves() {
    let mut game = small_game();
    let mut band = TeamId::INVALID;
    game.run(|list, teams, env| band = teams.create_team("Band", list, env).expect("team"));
    let id = band.0;
    for object in [5, 6] {
        let t = game.teams.team_mut(band).expect("team");
        t.add_member(ObjectId(object));
        t.add_member(ObjectId(object));
    }
    assert_eq!(
        game.teams.team(band).expect("team").members,
        [ObjectId(6), ObjectId(5)]
    );
    game.events();

    game.run(|list, teams, env| teams.set_controlling_player(band, 2, list, env));
    assert_eq!(game.events(), ["captured 6 1 2", "captured 5 1 2"]);
    assert_eq!(game.teams.controlling_player(band), Some(2));
    let prototype = game.teams.team(band).expect("team").prototype;
    assert!(!game.player(1).team_prototypes.contains(&prototype));
    assert_eq!(game.player(2).team_prototypes.last(), Some(&prototype));
    game.run(|list, teams, env| teams.set_controlling_player(band, 99, list, env));
    assert_eq!(game.teams.controlling_player(band), Some(2));

    let own = game.player(1).default_team.expect("default");
    game.list
        .player_mut(2)
        .expect("player")
        .set_team_relationship(band, Relationship::Allies);
    game.teams
        .team_mut(own)
        .expect("team")
        .set_override_team_relationship(band, Relationship::Allies);
    game.list.player_mut(1).expect("player").default_team = Some(band);
    game.run(|list, teams, env| teams.delete_team_instance(band, list, env));
    assert_eq!(
        game.events(),
        [
            format!("destroyed {id}"),
            format!("ai_pre_destroy 2 {id}"),
            "detach 6".to_owned(),
            "detach 5".to_owned(),
        ]
    );
    assert!(game.teams.team(band).is_none());
    assert!(game.player(2).team_relations.is_empty());
    assert!(game
        .teams
        .team(own)
        .expect("team")
        .team_relations
        .is_empty());
    assert_eq!(game.player(1).default_team, None);

    game.run(|list, _, env| list.new_map(env.world));
    assert_eq!(game.events(), ["ai_new_map 2"]);
    game.list.load_post_process();
    assert_eq!(game.list.slot_index(3), 255);
    assert_eq!(game.list.player_from_slot_index(0), None);
    assert_eq!(game.list.slot_index(-1), -1);
    assert_eq!(game.list.slot_index(16), -1);
}

// ---- retail maps ---------------------------------------------------------------

struct MapSides {
    sides: SidesList,
    keys: NameKeyGenerator,
}

fn parse_sides(
    input: &mut DataChunkInput<'_, MapSides>,
    info: &DataChunkInfo,
    map: &mut MapSides,
) -> Result<bool, DataChunkError> {
    map.sides
        .read_sides_and_teams(input, info.version, &mut map.keys)
        .map_err(|_| DataChunkError::CorruptFormat)?;
    Ok(true)
}

/// The sides of a map file. Script lists are not read: nothing here needs them.
fn read_map_sides(bytes: &[u8], keys: &mut NameKeyGenerator) -> SidesList {
    let mut map = MapSides {
        sides: SidesList::new(),
        keys: std::mem::take(keys),
    };
    let mut stream = CachedFileInputStream::from_bytes(bytes);
    {
        let mut file: DataChunkInput<'_, MapSides> = DataChunkInput::new(&mut stream);
        assert!(file.is_valid_file_type(), "map file");
        file.register_parser("SidesList", "", parse_sides);
        assert!(file.parse(&mut map).expect("sides"), "parsed");
    }
    let well_known = WellKnownSidesKeys::intern(&mut map.keys);
    map.sides.validate_sides(&well_known);
    *keys = map.keys;
    map.sides
}

fn retail_rules(root: &str, keys: &mut NameKeyGenerator) -> Option<Rules> {
    let bytes = std::fs::read(format!("{root}/INIZH.big")).ok()?;
    let archive = BigFileSystem::open_archive_bytes("INIZH.big", bytes, FileInfo::default())
        .expect("archive");
    let read = |name: &str| {
        archive
            .open_file(name)
            .unwrap_or_else(|| panic!("{name} missing"))
            .read_entire_and_close()
    };
    let mut rules = Rules::default();
    for name in ["Data\\INI\\Default\\Science.ini", "Data\\INI\\Science.ini"] {
        let text = read(name);
        let mut ini = IniReader::new(name, &text, IniLoadType::Overwrite);
        rules
            .sciences
            .load_from_ini(&mut ini, keys, &mut NoText)
            .expect("sciences");
    }
    let text = read("Data\\INI\\Rank.ini");
    let mut ini = IniReader::new("Rank.ini", &text, IniLoadType::Overwrite);
    rules
        .ranks
        .load_from_ini(&mut ini, &rules.sciences, keys, &mut NoText)
        .expect("ranks");
    for name in [
        "Data\\INI\\Default\\PlayerTemplate.ini",
        "Data\\INI\\PlayerTemplate.ini",
    ] {
        let text = read(name);
        let mut ini = IniReader::new(name, &text, IniLoadType::Overwrite);
        rules
            .templates
            .load_from_ini(&mut ini, &rules.sciences, keys, &mut NoText)
            .expect("templates");
    }
    Some(rules)
}

impl Game {
    /// A player's save without the parts other systems own.
    fn bare_player_hex(&mut self, i: PlayerIndex) -> String {
        let before = self.list.clone();
        let p = self.list.player_mut(i).expect("player");
        p.ai = None;
        p.has_economy_trackers = false;
        let bytes = as_reference(self.player_bytes(i));
        self.list = before;
        hex(&bytes)
    }

    fn retail_digest(&mut self) -> String {
        // Script lists are the sides list's own concern.
        let mut sides = self.check_sides();
        let from = sides.find(" scripts=").expect("scripts");
        let to = sides.find(" build=").expect("build");
        sides.replace_range(from..to, "");
        let mut rows = vec![sides, self.check_list()];
        let count = self.list.player_count();
        rows.extend((0..=count.min(15)).map(|i| self.check_player(i)));
        let protos: Vec<u32> = (0..16)
            .flat_map(|i| self.player(i).team_prototypes.clone())
            .collect();
        for &id in &protos {
            rows.push(format!(
                "= tpl {id} {}",
                digest(&self.template_text(id), false)
            ));
        }
        rows.push(self.check_instances());
        rows.push(self.check_masks());
        rows.push(self.check_relations());
        let saves: String = (0..count).map(|i| self.bare_player_hex(i) + ";").collect();
        rows.push(format!("= xp {saves}"));
        rows.push(format!("= xt {}", hex(&self.team_bytes())));
        rows.push(format!("= crc {:08x}", self.crc()));

        let mut out = format!(
            "players={count} teams={} protos={}",
            self.live_teams().len(),
            protos.len()
        );
        for section in [
            "sides", "list", "player", "tpl", "inst", "masks", "trel", "xp", "xt", "crc",
        ] {
            let prefix = format!("= {section} ");
            let joined: String = rows
                .iter()
                .filter(|r| r.starts_with(&prefix))
                .map(|r| format!("{r}\n"))
                .collect();
            out += &format!(" {section}={:016x}", fnv(FNV, joined.as_bytes()));
        }
        out
    }
}

#[test]
fn retail_maps_match() {
    let root = std::env::var("CNC_GENERALS_ZH_PATH").unwrap_or_else(|_| {
        format!(
            "{}/Games/generalszh",
            std::env::var("HOME").unwrap_or_default()
        )
    });
    let mut keys = NameKeyGenerator::new();
    let (Ok(maps), Some(rules)) = (
        std::fs::read(format!("{root}/MapsZH.big")),
        retail_rules(&root, &mut keys),
    ) else {
        eprintln!("retail_maps_match: no archives under {root}; nothing checked");
        return;
    };
    let archive = BigFileSystem::open_archive_bytes("MapsZH.big", maps, FileInfo::default())
        .expect("archive");
    let mut names = BTreeSet::new();
    archive.get_file_list_in_directory("", "*.map", &mut names);
    let by_key: BTreeMap<String, Vec<u8>> = names
        .iter()
        .map(|name| {
            let data = archive
                .open_file(name.as_str())
                .expect("listed file")
                .read_entire_and_close();
            (format!("{:016x}", fnv(FNV, &data)), data)
        })
        .collect();

    let expected: Vec<(&str, &str)> = rows(RETAIL).filter_map(|r| r.split_once(' ')).collect();
    assert!(expected.len() >= 100, "{} maps recorded", expected.len());

    let mut host = Host::default();
    // The reference ran without `GameData.ini`: no starting cash.
    host.cash = 0;
    let list = PlayerList::new(env!(rules, keys, host));
    let mut game = Game {
        rules,
        keys,
        host,
        sides: SidesList::new(),
        list,
        teams: TeamFactory::new(),
        objects: Vec::new(),
        reloads: 0,
        refused: 0,
    };

    let mut failed = Vec::new();
    for (n, (key, want)) in expected.iter().enumerate() {
        let data = by_key.get(*key).expect("recorded map in the archive");
        let mut terrain = TerrainLogic::new();
        terrain
            .load_map(
                data,
                "retail.map",
                &mut game.keys,
                ICoord2D { x: 70, y: 90 },
                TRUE,
            )
            .expect("map loads");
        game.host.waypoints = terrain
            .waypoints()
            .map(|w| (w.id, w.name.clone(), w.location))
            .collect();
        game.sides = read_map_sides(data, &mut game.keys);
        let (rules, keys, host) = (&game.rules, &mut game.keys, &mut game.host);
        game.list
            .new_game(&mut game.sides, &mut game.teams, env!(rules, *keys, *host));
        let got = game.retail_digest();
        if got != *want {
            let differing: Vec<&str> = got
                .split(' ')
                .zip(want.split(' '))
                .filter_map(|(g, w)| (g != w).then_some(g.split('=').next().unwrap_or(g)))
                .collect();
            failed.push(format!("map {n} ({key}): {}", differing.join(" ")));
        }
    }
    assert!(
        failed.is_empty(),
        "{} of {} differ:\n{}",
        failed.len(),
        expected.len(),
        failed.join("\n")
    );
}
