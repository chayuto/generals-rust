//! `Player::xfer` and `Player::crc` with the snapshots they call, over varied
//! player states (`tests/cpp_harness/gen_player_xfer.cpp`).
//!
//! The reference library writes the retail save format. Without
//! `retail-xfer-save` the port writes `Money` and `TunnelTracker` version 2,
//! which [`as_reference`] turns back. It was also built where `WideChar` has four
//! bytes, while the port writes the two of the Windows game: the general name
//! alone is converted ([`general_name_forms`]).

use std::collections::BTreeMap;

use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::golden_json::{decode_hex_bytes, encode_hex_bytes};
use zh_common::golden_json::{get_i32, get_str, get_u32};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{Int, ObjectId, Relationship, TeamId, UnsignedInt};
use zh_common::xfer::{Xfer, XferCrc, XferError, XferLoad, XferMode, XferSave};
use zh_logic::ai::ai_player::AiPlayer;
use zh_logic::economy::money::NoopMoneyCallbacks;
use zh_logic::economy::score_keeper::{ScoreCountEntry, ScoreTemplateDesc};
use zh_logic::gamelogic::global_data::PlayerType;
use zh_logic::modules::update::battle_plan_update::BattlePlanBonuses;
use zh_logic::player::player::{Player, SpecialPowerReadyTimer};
use zh_logic::player::player_list::PlayerList;
use zh_logic::player::player_snapshots::PlayerParts;
use zh_logic::player::seams::{
    AiKind, PlayerEnv, PlayerIndex, PlayerPart, PlayerWorld, SkirmishScripts,
};
use zh_logic::player::team::TeamFactory;
use zh_logic::rules::kind_of::{KindOfMaskType, KindOfType};
use zh_logic::rules::player_template::PlayerTemplateStore;
use zh_logic::rules::rank_info::RankInfoStore;
use zh_logic::rules::science::{ScienceStore, ScienceTextLookup, ScienceType};
use zh_logic::rules::upgrade::UpgradeMaskType;
use zh_logic::sim::TunnelTrackerState;
use zh_logic::spatial::sides_list::SidesList;

const VECTORS: &str = include_str!("../../../docs/golden_vectors/PlayerXfer.json");

#[derive(Default)]
struct Host {
    upgrades: Vec<(String, UpgradeMaskType)>,
    templates: Vec<ScoreTemplateDesc>,
    ai: BTreeMap<Int, AiPlayer>,
    tunnels: BTreeMap<Int, TunnelTrackerState>,
    /// The tunnel tracker's part of the last save.
    tunnel_bytes: Vec<u8>,
}

impl PlayerWorld for Host {
    fn frame(&self) -> UnsignedInt {
        0
    }
    fn starting_cash(&self) -> UnsignedInt {
        0
    }
    fn is_network(&self) -> bool {
        false
    }
    fn game_slot_occupied(&self, _slot: usize) -> bool {
        false
    }
    fn global_difficulty(&self) -> Int {
        0
    }
    fn force_skirmish_ai(&self) -> bool {
        false
    }
    fn general_name(&self) -> Vec<u16> {
        Vec::new()
    }
    fn waypoint_location(&self, _name: &str) -> Option<Coord3D> {
        None
    }
    fn waypoint_exists(&self, _id: UnsignedInt) -> bool {
        false
    }
    fn becoming_local_player(&mut self, _player: PlayerIndex, _yes: bool) {}
    fn local_player_update(&mut self, _player: PlayerIndex) {}
    fn create_ai(&mut self, _player: PlayerIndex, _kind: AiKind) {}
    fn delete_ai(&mut self, _player: PlayerIndex) {}
    fn set_ai_difficulty(&mut self, _player: PlayerIndex, _difficulty: Int) {}
    fn ai_update(&mut self, _player: PlayerIndex) {}
    fn ai_new_map(&mut self, _player: PlayerIndex) {}
    fn ai_pre_team_destroy(&mut self, _player: PlayerIndex, _team: TeamId) {}
    fn qualify_skirmish_scripts(&mut self, _sides: &mut SidesList, _copy: &SkirmishScripts<'_>) {}
    fn create_economy_trackers(&mut self, _player: PlayerIndex) {}
    fn delete_economy_trackers(&mut self, _player: PlayerIndex) {}
    fn academy_init(&mut self, _player: PlayerIndex) {}
    fn academy_update(&mut self, _player: PlayerIndex) {}
    fn academy_record_income(&mut self, _player: PlayerIndex) {}
    fn power_brownout_changed(&mut self, _player: PlayerIndex, _brownout: bool) {}
    fn science_added(&mut self, _player: PlayerIndex, _science: ScienceType) {}
    fn score_template(&self, name: &str) -> Option<ScoreTemplateDesc> {
        self.templates.iter().find(|t| t.name == name).cloned()
    }
    fn upgrade_masks(&self) -> Vec<(String, UpgradeMaskType)> {
        self.upgrades.clone()
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
    }
    fn duplicate_script(&mut self, _name: &str) -> Option<UnsignedInt> {
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
        player: PlayerIndex,
        part: PlayerPart,
        xfer: &mut dyn Xfer,
    ) -> Result<(), XferError> {
        let mut parts = PlayerParts {
            ai_players: &mut self.ai,
            tunnel_trackers: &mut self.tunnels,
            thing_factory: None,
            teams: None,
        };
        if part == PlayerPart::TunnelTracker && xfer.mode() == XferMode::Save {
            let mut own = XferSave::new();
            parts.xfer(player, part, &mut own)?;
            self.tunnel_bytes = own.into_bytes();
        }
        parts.xfer(player, part, xfer)
    }
}

struct Rules {
    sciences: ScienceStore,
    templates: PlayerTemplateStore,
    ranks: RankInfoStore,
    keys: NameKeyGenerator,
    /// The harness's upgrade and template indices.
    upgrade_names: [&'static str; 5],
    template_names: [&'static str; 4],
}

struct NoText;

impl ScienceTextLookup for NoText {
    fn fetch_label(&mut self, _label: &str) -> String {
        String::new()
    }
}

/// The header array `key` of `"name number"` strings.
fn pairs(key: &str) -> Vec<(String, u32)> {
    let start = VECTORS.find(&format!("\"{key}\":[")).expect("header") + key.len() + 4;
    let end = start + VECTORS[start..].find(']').expect("end");
    VECTORS[start..end]
        .split(',')
        .map(|entry| {
            let (name, n) = entry.trim_matches('"').split_once(' ').expect("pair");
            (name.to_owned(), n.parse().expect("number"))
        })
        .collect()
}

fn rules() -> Rules {
    let mut rules = Rules {
        sciences: ScienceStore::default(),
        templates: PlayerTemplateStore::new(),
        ranks: RankInfoStore::new(),
        keys: NameKeyGenerator::new(),
        upgrade_names: ["XU_A", "XU_B", "XU_C", "XU_D", "XU_E"],
        template_names: ["X_Hut", "X_Car", "X_Gun", "X_Yard"],
    };
    let start = VECTORS.find("\"sciences\":\"").expect("sciences") + 12;
    let end = start + VECTORS[start..].find('"').expect("end");
    let text = VECTORS[start..end].replace("\\n", "\n");
    let (sciences, keys) = (&mut rules.sciences, &mut rules.keys);
    IniReader::new("science.ini", text.as_bytes(), IniLoadType::Overwrite)
        .load_blocks(None, |_, reader| {
            reader.next_token(zh_common::ini::SEPS_DEFAULT)?;
            sciences.parse_science_definition(reader, keys, &mut NoText)?;
            Ok(())
        })
        .expect("sciences");
    rules
}

fn host() -> Host {
    let upgrades = pairs("upgrades")
        .into_iter()
        .map(|(name, bit)| {
            let mut mask = UpgradeMaskType::default();
            mask.set(bit as i32, 1);
            (name, mask)
        })
        .collect();
    let templates = pairs("templates")
        .into_iter()
        .map(|(name, id)| ScoreTemplateDesc {
            id,
            name,
            kind_of: KindOfMaskType::default(),
            equivalent_root_id: id,
        })
        .collect();
    Host {
        upgrades,
        templates,
        ..Host::default()
    }
}

fn kind_mask(i: i64) -> KindOfMaskType {
    let mut mask = KindOfMaskType::default();
    if i == 0 || i == 2 {
        mask.set(KindOfType::Structure as i32, 1);
    }
    if i == 1 || i == 2 {
        mask.set(KindOfType::Vehicle as i32, 1);
    }
    mask
}

/// What the harness's `clear_player` leaves.
fn cleared(index: Int, skirmish: bool, host: &mut Host) -> Player {
    let kind = if skirmish {
        AiKind::Skirmish
    } else {
        AiKind::Standard
    };
    let mut p = Player::blank(index);
    p.player_type = PlayerType::Computer;
    p.ai = Some(kind);
    p.has_economy_trackers = true;
    host.ai = BTreeMap::from([(index, AiPlayer::new(kind, index, 0, 1, 0))]);
    host.tunnels.clear();
    p
}

fn apply(p: &mut Player, rules: &mut Rules, host: &mut Host, state: &str) {
    let index = p.index;
    for field in state.split(';').filter(|f| !f.is_empty()) {
        let mut words = field.split(' ');
        let key = words.next().expect("key");
        let v: Vec<i64> = words.map(|w| w.parse().expect("value")).collect();
        let n = v[0] as Int;
        let on = v[0] != 0;
        let real = |at: usize| f32::from_bits(v[at] as u32);
        let id = ObjectId(n);
        let tunnel = host.tunnels.entry(index).or_default();
        let ai = host.ai.get_mut(&index).expect("ai");
        match key {
            "deposit" => p.money.deposit(n as u32, 0, 1, &mut NoopMoneyCallbacks),
            "bucket" => p.money.update_income_bucket(n as u32),
            "preorder" => p.is_preorder = on,
            "dead" => p.is_dead = on,
            "radar_off" => p.radar_disabled = on,
            "units" => p.can_build_units = on,
            "base" => p.can_build_base = on,
            "observer" => p.observer = on,
            "listed" => p.list_in_score_screen = on,
            "hunt" => p.units_should_hunt = on,
            "radar" => p.radar_count = n,
            "proof" => p.disable_proof_radar_count = n,
            "rank" => p.rank_level = n,
            "skill" => p.skill_points = n,
            "points" => p.science_purchase_points = n,
            "up" => p.level_up = n,
            "down" => p.level_down = n,
            "bombard" => p.battle_plans.bombardment_plans = n,
            "hold" => p.battle_plans.hold_the_line_plans = n,
            "search" => p.battle_plans.search_and_destroy_plans = n,
            "modifier" => p.skill_points_modifier = real(0),
            "bounty" => p.cash_bounty_percent = real(0),
            "science" => {
                let name = ["SCI_A", "SCI_B", "SCI_C", "SCI_D"][v[1] as usize];
                let science: ScienceType = rules
                    .sciences
                    .get_science_from_internal_name(name, &mut rules.keys);
                [
                    &mut p.sciences,
                    &mut p.sciences_disabled,
                    &mut p.sciences_hidden,
                ][n as usize]
                    .push(science);
            }
            "add_upgrade" | "remove_upgrade" => {
                let name = rules.upgrade_names[n as usize];
                if key == "remove_upgrade" {
                    p.remove_upgrade(name);
                } else {
                    let mask = host.upgrades.iter().find(|u| u.0 == name).expect("mask");
                    p.add_upgrade(name, &mask.1, v[1] as Int);
                }
            }
            "squad" => {
                let squad = match p.squads.hotkeys.get_mut(n as usize) {
                    Some(squad) => squad,
                    None => &mut p.squads.selection,
                };
                squad.ids.push(v[1] as u32);
            }
            "general" => p.general_name.push(v[0] as u16),
            "relation" => {
                let r = match v[2] {
                    0 => Relationship::Enemies,
                    1 => Relationship::Neutral,
                    _ => Relationship::Allies,
                };
                let map = if on {
                    &mut p.team_relations
                } else {
                    &mut p.player_relations
                };
                map.set(v[1] as u32, r);
            }
            "attacked" => p.attacked_by[n as usize] = true,
            "change" => p.add_production_cost_change(kind_mask(v[0]), real(1)),
            "timer" => p.special_power_timers.push(SpecialPowerReadyTimer {
                template_id: v[0] as u32,
                ready_frame: v[1] as u32,
            }),
            "bonus" => {
                p.battle_plans.bonuses = Some(BattlePlanBonuses {
                    armor_scalar: real(0),
                    sight_range_scalar: real(1),
                    bombardment: v[2] as Int,
                    hold_the_line: v[3] as Int,
                    search_and_destroy: v[4] as Int,
                    valid_kind_of: kind_mask(v[5]),
                    invalid_kind_of: kind_mask(v[6]),
                })
            }
            "earned" => p.score_keeper.total_money_earned = n,
            "score" => p.score_keeper.current_score = n,
            "count" => {
                let name = rules.template_names[v[1] as usize];
                let t = host.templates.iter().find(|t| t.name == name).expect("t");
                let keeper = &mut p.score_keeper;
                let map = [
                    &mut keeper.objects_built,
                    &mut keeper.objects_lost,
                    &mut keeper.objects_captured,
                ];
                map.into_iter().nth(n as usize).expect("map").insert(
                    t.id,
                    ScoreCountEntry {
                        name: t.name.clone(),
                        kind_of: t.kind_of.clone(),
                        equivalent_root_id: t.equivalent_root_id,
                        count: v[2] as Int,
                    },
                );
            }
            "id" => {
                let id = ObjectId(v[1] as Int);
                match n {
                    0 => p.supply_warehouses.push(id),
                    1 => p.supply_centers.push(id),
                    _ => tunnel.tunnel_ids.push(id),
                }
            }
            "team_timer" => ai.team_timer = n,
            "structure_timer" => ai.structure_timer = n,
            "team_seconds" => ai.team_seconds = n,
            "build_delay" => ai.build_delay = n,
            "team_delay" => ai.team_delay = n,
            "frame_built" => ai.frame_last_building_built = n,
            "skillset" => ai.skillset_selector = n,
            "bridge_timer" => ai.bridge_timer = n,
            "difficulty" => ai.difficulty = n,
            "radius" => ai.base_radius = real(0),
            "dozer" => ai.repair_dozer = id,
            "warehouse" => ai.cur_warehouse_id = id,
            other => panic!("field {other}"),
        }
    }
}

macro_rules! env {
    ($rules:ident, $host:expr) => {
        &mut PlayerEnv {
            templates: &$rules.templates,
            ranks: &$rules.ranks,
            sciences: &$rules.sciences,
            name_keys: &mut $rules.keys,
            world: $host,
        }
    };
}

/// The general name as the port saves it and as this reference build does.
fn general_name_forms(name: &[u16]) -> (Vec<u8>, Vec<u8>) {
    let mut narrow = vec![name.len() as u8];
    let mut wide = narrow.clone();
    for &c in name {
        narrow.extend_from_slice(&c.to_le_bytes());
        wide.extend_from_slice(&u32::from(c).to_le_bytes());
    }
    (narrow, wide)
}

fn replace_once(bytes: &mut Vec<u8>, from: &[u8], to: &[u8]) {
    let at = bytes
        .windows(from.len())
        .position(|w| w == from)
        .expect("section");
    bytes.splice(at..at + from.len(), to.iter().copied());
}

/// A port save in the reference library's retail format.
fn as_reference(mut bytes: Vec<u8>, host: &Host, p: &Player) -> Vec<u8> {
    let index = p.index;
    if !p.general_name.is_empty() {
        let (narrow, wide) = general_name_forms(&p.general_name);
        replace_once(&mut bytes, &narrow, &wide);
    }
    if cfg!(feature = "retail-xfer-save") {
        return bytes;
    }
    assert_eq!(bytes[1], 2, "money version");
    bytes[1] = 1;
    bytes.drain(6..6 + 60 * 4 + 4);
    let own = &host.tunnel_bytes;
    let at = bytes
        .windows(own.len())
        .rposition(|w| w == own)
        .expect("tunnel section");
    let mut retail = own.clone();
    assert_eq!(retail[0], 2, "tunnel version");
    retail[0] = 1;
    let count = host.tunnels.get(&index).map_or(0, |t| t.tunnel_ids.len());
    retail.extend_from_slice(&(count as u32).to_le_bytes());
    bytes.splice(at..at + own.len(), retail);
    bytes
}

fn save(p: &mut Player, rules: &mut Rules, host: &mut Host, teams: &TeamFactory) -> Vec<u8> {
    let mut x = XferSave::new();
    p.xfer(&mut x, teams, env!(rules, host)).expect("save");
    as_reference(x.into_bytes(), host, p)
}

#[test]
fn player_saves_and_checksums_match_the_reference() {
    let mut rules = rules();
    let teams = TeamFactory::new();
    let rows: Vec<&str> = VECTORS
        .lines()
        .filter(|l| l.starts_with("{\"player\""))
        .collect();
    assert_eq!(rows.len(), 120);
    for (n, row) in rows.iter().enumerate() {
        let index = get_i32(row, "player").expect("player");
        let skirmish = get_i32(row, "skirmish").expect("skirmish") != 0;
        let mut host = host();
        let mut p = cleared(index, skirmish, &mut host);
        apply(
            &mut p,
            &mut rules,
            &mut host,
            get_str(row, "state").expect("state"),
        );

        let want = get_str(row, "save").expect("save");
        let got = save(&mut p, &mut rules, &mut host, &teams);
        assert_eq!(
            encode_hex_bytes(&got).to_ascii_lowercase(),
            want,
            "vector {n}: save"
        );

        let mut crc = XferCrc::new();
        p.crc(&mut crc).expect("crc");
        assert_eq!(
            crc.get_crc(),
            get_u32(row, "crc").expect("crc"),
            "vector {n}: crc"
        );

        let mut bytes = decode_hex_bytes(want).expect("hex");
        if !p.general_name.is_empty() {
            let (narrow, wide) = general_name_forms(&p.general_name);
            replace_once(&mut bytes, &wide, &narrow);
        }
        let mut loaded = cleared(index, skirmish, &mut host);
        let mut x = XferLoad::new(&bytes);
        loaded
            .xfer(&mut x, &teams, env!(rules, &mut host))
            .unwrap_or_else(|e| panic!("vector {n}: load {e:?}"));
        let got = save(&mut loaded, &mut rules, &mut host, &teams);
        assert_eq!(
            encode_hex_bytes(&got).to_ascii_lowercase(),
            get_str(row, "resave").expect("resave"),
            "vector {n}: save after load"
        );
    }
}

/// What cannot be restored is refused.
#[test]
fn a_part_the_sim_does_not_hold_is_refused() {
    let mut rules = rules();
    let teams = TeamFactory::new();
    let mut host = host();
    let mut p = cleared(1, false, &mut host);
    host.ai.clear();
    let mut x = XferSave::new();
    let refused = p.xfer(&mut x, &teams, env!(rules, &mut host));
    assert_eq!(refused, Err(XferError::InvalidData), "no AI player");

    let mut p = cleared(1, false, &mut host);
    let tracker = host.tunnels.entry(1).or_default();
    tracker.contain_list.push(ObjectId(7));
    tracker.contain_list_size = 1;
    let mut x = XferSave::new();
    p.xfer(&mut x, &teams, env!(rules, &mut host))
        .expect("save");
    let bytes = x.into_bytes();
    let mut loaded = cleared(1, false, &mut host);
    let refused = loaded.xfer(&mut XferLoad::new(&bytes), &teams, env!(rules, &mut host));
    assert_eq!(refused, Err(XferError::InvalidData), "contained object");
}
