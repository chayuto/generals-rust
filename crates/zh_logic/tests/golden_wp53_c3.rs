//! Gate V2 differential verification for Wave `WP-5.3-C3`:
//! `PKT-262 BridgeTowerBehavior`, `PKT-263 BunkerBusterBehavior`,
//! `PKT-291 PropagandaTowerBehavior` against
//! `docs/golden_vectors/WP53_C3.json` (`tests/cpp_harness/gen_wp53_c3.cpp`).
//!
//! `BridgeBehavior` and `AIUpdateInterface` are not ported. Stand-ins answer `getTowerID` and
//! construct the AI. The test also does what unported code would: it links the towers to their
//! bridge, names the bomb's current victim, sets `MISSILE_KILLING_SELF` and records player
//! upgrades.
//!
//! The saved `PropagandaTowerBehavior` ids are compared in the order saved, which is the
//! order of the partition scan.

use std::any::Any;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use zh_common::ini::{IniError, IniLoadType, IniReader};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, Relationship, TeamId, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferCrc, XferError, XferLoad, XferSave};
use zh_logic::arena::{FacetMask, ModuleContext, ModuleSlot, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::gamelogic::global_data::WeaponBonusConditionType;
use zh_logic::modules::contain::open_contain::kill_object;
use zh_logic::modules::module::{
    AiUpdateFacet, BehaviorModule, BridgeFacet, BridgeTowerType, ModuleData, ModuleType,
};
use zh_logic::modules::module_factory::ModuleParseContext;
use zh_logic::object::weapon::WeaponTemplate;
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::object_status_types::{ObjectStatusMaskType, ObjectStatusType};
use zh_logic::rules::upgrade::UpgradeMaskType;
use zh_logic::scheduler::{SleepyUpdatePhase, UPDATE_SLEEP_FOREVER};
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimView, SimWorld};

// -----------------------------------------------------------------------------
// Minimal JSON reader. Numbers keep their source text so comparisons are exact.
// -----------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Json {
    Scalar(String),
    Arr(Vec<Json>),
    Obj(BTreeMap<String, Json>),
}

impl Json {
    fn arr(&self) -> &[Json] {
        match self {
            Json::Arr(a) => a,
            _ => panic!("expected JSON array"),
        }
    }

    fn text(&self) -> &str {
        match self {
            Json::Scalar(s) => s,
            _ => panic!("expected JSON scalar"),
        }
    }

    fn obj(&self) -> &BTreeMap<String, Json> {
        match self {
            Json::Obj(m) => m,
            _ => panic!("expected JSON object"),
        }
    }

    fn get(&self, key: &str) -> &Json {
        self.obj()
            .get(key)
            .unwrap_or_else(|| panic!("missing JSON key `{key}`"))
    }

    fn num(&self) -> u32 {
        self.text().parse().expect("u32")
    }

    fn u32(&self, key: &str) -> u32 {
        self.get(key).num()
    }
}

struct Parser<'a> {
    b: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn peek(&mut self) -> u8 {
        while self.b[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
        self.b[self.i]
    }

    fn string(&mut self) -> String {
        assert_eq!(self.peek(), b'"');
        self.i += 1;
        let start = self.i;
        while self.b[self.i] != b'"' {
            self.i += 1;
        }
        let out = String::from_utf8(self.b[start..self.i].to_vec()).expect("utf8");
        self.i += 1;
        out
    }

    fn value(&mut self) -> Json {
        match self.peek() {
            b'{' => {
                self.i += 1;
                let mut map = BTreeMap::new();
                while self.peek() != b'}' {
                    let key = self.string();
                    assert_eq!(self.peek(), b':');
                    self.i += 1;
                    let value = self.value();
                    map.insert(key, value);
                    if self.peek() == b',' {
                        self.i += 1;
                    }
                }
                self.i += 1;
                Json::Obj(map)
            }
            b'[' => {
                self.i += 1;
                let mut arr = Vec::new();
                while self.peek() != b']' {
                    arr.push(self.value());
                    if self.peek() == b',' {
                        self.i += 1;
                    }
                }
                self.i += 1;
                Json::Arr(arr)
            }
            b'"' => Json::Scalar(self.string()),
            _ => {
                let start = self.i;
                while !matches!(self.b[self.i], b',' | b'}' | b']') {
                    self.i += 1;
                }
                let text = String::from_utf8(self.b[start..self.i].to_vec()).expect("utf8");
                Json::Scalar(text.trim().to_owned())
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Fixture templates (mirrors `fixture_ini` in the harness)
// -----------------------------------------------------------------------------

const KINDS: [&str; 3] = ["bridge_tower", "bunker_buster", "propaganda"];

fn fields(kind: usize) -> &'static [&'static str] {
    match kind {
        0 => &["hp0", "hp1", "hp2", "hp3", "bridge_hp", "dead_mask", "rng"],
        1 => &[
            "target_hp",
            "rider_hp0",
            "rider_hp1",
            "rider_hp2",
            "contained_mask",
            "dead_mask",
            "target_team",
            "fx_count",
            "fx_digest",
            "shock_count",
            "shock_digest",
            "rng",
        ],
        _ => &[
            "hp0",
            "bonus0",
            "hp1",
            "bonus1",
            "hp2",
            "bonus2",
            "hp3",
            "bonus3",
            "tower_hp",
            "tower_bonus",
            "next_frame",
            "count",
            "fx_count",
            "fx_digest",
            "rng",
        ],
    }
}

// -----------------------------------------------------------------------------
// Stand-ins for unported modules
// -----------------------------------------------------------------------------

const AI_MASK: FacetMask = FacetMask(0x001 | FacetMask::AI_UPDATE.0);
const BRIDGE_MASK: FacetMask = FacetMask(0x007 | FacetMask::BRIDGE_BEHAVIOR.0);

struct StubData {
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    facet_mask: FacetMask,
}

impl ModuleData for StubData {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn tag_key(&self) -> NameKeyType {
        self.tag_key
    }
    fn module_name_key(&self) -> NameKeyType {
        self.module_name_key
    }
    fn facet_mask(&self) -> FacetMask {
        self.facet_mask
    }
}

/// `AIUpdateInterface`: only the construction draws, which the spine imitates.
struct AiStandIn {
    key: NameKeyType,
}

impl AiUpdateFacet for AiStandIn {}

impl BehaviorModule for AiStandIn {
    fn module_name_key(&self) -> NameKeyType {
        self.key
    }
    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        sim.ai_construct(ctx.owner_slot);
    }
    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        sim.ai_init_state_machine(ctx.owner_slot);
    }
    fn xfer(&mut self, _xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        Ok(())
    }
    fn as_ai(&self) -> Option<&dyn AiUpdateFacet> {
        Some(self)
    }
    fn as_ai_mut(&mut self) -> Option<&mut dyn AiUpdateFacet> {
        Some(self)
    }
}

/// `BridgeBehavior`: only the tower table behind `getTowerID`.
struct BridgeStandIn {
    key: NameKeyType,
    towers: [ObjectId; 4],
}

impl BridgeFacet for BridgeStandIn {
    fn set_tower(&mut self, tower_type: BridgeTowerType, tower_id: ObjectId) {
        self.towers[tower_type.as_i32() as usize] = tower_id;
    }
    fn get_tower_id(&self, tower_type: BridgeTowerType) -> ObjectId {
        self.towers[tower_type.as_i32() as usize]
    }
    fn create_scaffolding(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn remove_scaffolding(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn is_scaffold_in_motion(&self, _sim: &dyn SimView) -> Bool {
        FALSE
    }
    fn is_scaffold_present(&self) -> Bool {
        FALSE
    }
}

impl BehaviorModule for BridgeStandIn {
    fn module_name_key(&self) -> NameKeyType {
        self.key
    }
    fn xfer(&mut self, _xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        Ok(())
    }
    fn as_bridge(&self) -> Option<&dyn BridgeFacet> {
        Some(self)
    }
    fn as_bridge_mut(&mut self) -> Option<&mut dyn BridgeFacet> {
        Some(self)
    }
}

fn parse_stand_in(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    facet_mask: FacetMask,
) -> Result<Box<dyn ModuleData>, IniError> {
    reader.read_block_fields(|_, _| Err(IniError::UnknownToken))?;
    Ok(Box::new(StubData {
        tag_key,
        module_name_key,
        facet_mask,
    }))
}

fn parse_ai(
    r: &mut IniReader<'_>,
    t: NameKeyType,
    n: NameKeyType,
    _c: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    parse_stand_in(r, t, n, AI_MASK)
}

fn parse_bridge(
    r: &mut IniReader<'_>,
    t: NameKeyType,
    n: NameKeyType,
    _c: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    parse_stand_in(r, t, n, BRIDGE_MASK)
}

fn create_ai(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    Box::new(AiStandIn {
        key: data.module_name_key(),
    })
}

fn create_bridge(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    Box::new(BridgeStandIn {
        key: data.module_name_key(),
        towers: [ObjectId::INVALID; 4],
    })
}

// -----------------------------------------------------------------------------
// Fixture (the harness's `fixture_ini`)
// -----------------------------------------------------------------------------

const BRIDGE_HEALTH: [u32; 4] = [500, 1000, 777, 123];
const TOWER_HEALTH: [u32; 4] = [100, 250, 333, 1000];
const FX_NAMES: [&str; 5] = [
    "FX_P53C3_Det",
    "FX_P53C3_Crash",
    "FX_P53C3_Shock",
    "FX_P53C3_Pulse",
    "FX_P53C3_UpPulse",
];
const SHOCK_WEAPON: &str = "Wpn_P53C3_Shock";

const UPGRADE_INI: &str = "\
Upgrade Upgrade_P53C3_Bust\n  Type = PLAYER\nEnd\n\
Upgrade Upgrade_P53C3_Sub\n  Type = PLAYER\nEnd\n\
Upgrade Upgrade_P53C3_Obj\n  Type = OBJECT\nEnd\n";

const WEAPON_BLOCKS: [&str; 5] = [
    "Weapon Wpn_P53C3_Occ0\n  PrimaryDamage = 1.0\n  DamageType = EXPLOSION\n  DeathType = EXPLODED\nEnd",
    "Weapon Wpn_P53C3_Occ1\n  PrimaryDamage = 1.0\n  DamageType = FLAME\n  DeathType = BURNED\nEnd",
    "Weapon Wpn_P53C3_Shock\n  PrimaryDamage = 0.0\n  AttackRange = 100.0\n  DelayBetweenShots = 100\nEnd",
    "Weapon Wpn_P53C3_Rifle\n  PrimaryDamage = 10.0\n  AttackRange = 100.0\n  DamageType = SMALL_ARMS\nEnd",
    "Weapon Wpn_P53C3_Zero\n  PrimaryDamage = 0.0\n  AttackRange = 100.0\n  DamageType = HACK\nEnd",
];

fn object(s: &mut String, name: &str, kind_of: &str, health: &str, rest: &str) {
    let _ = write!(
        s,
        "Object {name}\n  KindOf = PRELOAD {kind_of}\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = {health}\n    InitialHealth = {health}\n  End\n{rest}End\n"
    );
}

fn weapon_set(weapon: &str) -> String {
    format!("  WeaponSet\n    Conditions = None\n    Weapon = PRIMARY {weapon}\n  End\n")
}

fn objects_ini() -> String {
    let mut s = String::new();
    let ai = "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n";
    for i in 0..4 {
        let health = format!("{}.0", BRIDGE_HEALTH[i]);
        object(
            &mut s,
            &format!("P53C3_Bridge{i}"),
            "IMMOBILE BRIDGE",
            &health,
            "  Behavior = BridgeBehavior ModuleTag_Bridge\n  End\n",
        );
        object(
            &mut s,
            &format!("P53C3_Plain{i}"),
            "IMMOBILE BRIDGE",
            &health,
            "",
        );
        object(
            &mut s,
            &format!("P53C3_Tower{i}"),
            "IMMOBILE BRIDGE_TOWER",
            &format!("{}.0", TOWER_HEALTH[i]),
            "  Behavior = BridgeTowerBehavior ModuleTag_Tower\n  End\n",
        );
    }
    object(&mut s, "P53C3_Source", "VEHICLE", "100.0", "");

    // (upgrade, occupant weapon, shockwave, detonation FX, crash FX, crash frequency)
    let bust: [(bool, usize, bool, bool, bool, &str); 8] = [
        (false, 0, false, true, true, ""),
        (false, 1, true, true, true, "100"),
        (true, 1, true, true, false, "133"),
        (true, 0, true, false, true, "333"),
        (false, 2, false, true, true, "100"),
        (false, 2, true, false, false, ""),
        (true, 2, true, true, true, "66"),
        (false, 0, true, true, true, "167"),
    ];
    for (i, (upgrade, occupant, shock, det, crash, freq)) in bust.into_iter().enumerate() {
        for (suffix, ai_block) in [("N", ""), ("A", ai)] {
            let mut m = format!("{ai_block}  Behavior = BunkerBusterBehavior ModuleTag_Buster\n");
            if upgrade {
                m.push_str("    UpgradeRequired = Upgrade_P53C3_Bust\n");
            }
            if occupant != 0 {
                let _ = writeln!(
                    m,
                    "    OccupantDamageWeaponTemplate = Wpn_P53C3_Occ{}",
                    occupant - 1
                );
            }
            if shock {
                let _ = writeln!(m, "    ShockwaveWeaponTemplate = {SHOCK_WEAPON}");
            }
            if det {
                m.push_str("    DetonationFX = FX_P53C3_Det\n");
            }
            if crash {
                m.push_str("    CrashThroughBunkerFX = FX_P53C3_Crash\n");
            }
            if !freq.is_empty() {
                let _ = writeln!(m, "    CrashThroughBunkerFXFrequency = {freq}");
            }
            m.push_str("  End\n");
            object(
                &mut s,
                &format!("P53C3_Bomb{i}{suffix}"),
                "PROJECTILE",
                "10.0",
                &m,
            );
        }
    }
    for (i, health) in ["50.0", "100.0", "101.0", "250.0"].into_iter().enumerate() {
        object(
            &mut s,
            &format!("P53C3_Rider{i}"),
            "INFANTRY\n  TransportSlotCount = 1",
            health,
            "",
        );
    }
    object(&mut s, "P53C3_PlainBldg", "STRUCTURE", "1000.0", "");
    let garrison = "  Behavior = GarrisonContain ModuleTag_Contain\n    ContainMax = 10\n";
    object(
        &mut s,
        "P53C3_Garr",
        "STRUCTURE",
        "1000.0",
        &format!("{garrison}  End\n"),
    );
    object(
        &mut s,
        "P53C3_OpenGarr",
        "STRUCTURE",
        "1000.0",
        &format!("{garrison}    IsEnclosingContainer = No\n  End\n"),
    );
    object(
        &mut s,
        "P53C3_Trans",
        "VEHICLE",
        "500.0",
        "  Behavior = TransportContain ModuleTag_Contain\n    Slots = 8\n    AllowInsideKindOf = INFANTRY\n  End\n",
    );

    // (radius, delay, heal, upgraded heal, object upgrade, affects self, kind of)
    let prop: [(&str, &str, &str, &str, bool, bool, &str); 8] = [
        ("60.0", "100", "1%", "2%", false, false, "STRUCTURE"),
        ("60.0", "500", "30%", "60%", false, true, "STRUCTURE"),
        ("120.0", "1000", "5%", "7%", true, false, "STRUCTURE"),
        ("120.0", "100", "30%", "10%", true, true, "STRUCTURE"),
        ("60.0", "333", "1%", "2%", false, true, "INFANTRY"),
        (
            "80.0",
            "100",
            "15%",
            "45%",
            false,
            false,
            "INFANTRY VEHICLE",
        ),
        (
            "80.0",
            "200",
            "15%",
            "45%",
            false,
            true,
            "INFANTRY VEHICLE PORTABLE_STRUCTURE",
        ),
        ("100.0", "0", "3%", "9%", false, false, "INFANTRY"),
    ];
    for (i, (radius, delay, heal, up_heal, object_upgrade, affects_self, kind_of)) in
        prop.into_iter().enumerate()
    {
        let upgrade = if object_upgrade {
            "Upgrade_P53C3_Obj"
        } else {
            "Upgrade_P53C3_Sub"
        };
        let affects = if affects_self { "Yes" } else { "No" };
        let module = format!(
            "{}  Behavior = PropagandaTowerBehavior ModuleTag_Prop\n    Radius = {radius}\n    DelayBetweenUpdates = {delay}\n    HealPercentEachSecond = {heal}\n    UpgradedHealPercentEachSecond = {up_heal}\n    UpgradeRequired = {upgrade}\n    PulseFX = FX_P53C3_Pulse\n    UpgradedPulseFX = FX_P53C3_UpPulse\n    AffectsSelf = {affects}\n  End\n",
            weapon_set("Wpn_P53C3_Rifle")
        );
        object(
            &mut s,
            &format!("P53C3_Prop{i}"),
            &format!("SCORE {kind_of}\n  TransportSlotCount = 1"),
            "1000.0",
            &module,
        );
    }
    let units = [
        ("INFANTRY SCORE", "Wpn_P53C3_Rifle"),
        ("INFANTRY SCORE", ""),
        ("INFANTRY SCORE", "Wpn_P53C3_Zero"),
        ("STRUCTURE SCORE", "Wpn_P53C3_Rifle"),
        ("INFANTRY", "Wpn_P53C3_Rifle"),
        ("VEHICLE MP_COUNT_FOR_VICTORY", "Wpn_P53C3_Rifle"),
        ("INFANTRY SCORE_CREATE", "Wpn_P53C3_Rifle"),
    ];
    for (i, (kind_of, weapon)) in units.into_iter().enumerate() {
        let set = if weapon.is_empty() {
            String::new()
        } else {
            weapon_set(weapon)
        };
        object(&mut s, UNITS[i], kind_of, "200.0", &set);
    }
    s
}

const UNITS: [&str; 7] = [
    "P53C3_URifle",
    "P53C3_UUnarmed",
    "P53C3_UZero",
    "P53C3_UBldg",
    "P53C3_UNoScore",
    "P53C3_UMp",
    "P53C3_UCreate",
];

fn build_rules() -> RulesRegistry {
    let mut rules = RulesRegistry::new();
    rules.module_registry.register_module(
        &mut rules.name_keys,
        "AIUpdateInterface",
        ModuleType::Behavior,
        AI_MASK,
        SleepyUpdatePhase::Initial,
        true,
        parse_ai,
        create_ai,
    );
    rules.module_registry.register_module(
        &mut rules.name_keys,
        "BridgeBehavior",
        ModuleType::Behavior,
        BRIDGE_MASK,
        SleepyUpdatePhase::Normal,
        false,
        parse_bridge,
        create_bridge,
    );
    for name in FX_NAMES {
        rules.fx_list_names.insert(name.to_owned());
    }
    rules.global_data.unit_damaged_thresh = 0.7;
    rules.global_data.unit_really_damaged_thresh = 0.35;
    rules
        .upgrade_center
        .load_from_ini(
            &mut IniReader::new(
                "upgrade.ini",
                UPGRADE_INI.as_bytes(),
                IniLoadType::Overwrite,
            ),
            &mut rules.name_keys,
        )
        .expect("load upgrades");
    for block in WEAPON_BLOCKS {
        let mut weapon = WeaponTemplate::new();
        weapon.parse_ini_block(block).expect("parse weapon");
        rules.weapon_store.add_template(weapon);
    }
    let text = objects_ini();
    let mut ini = IniReader::new("object.ini", text.as_bytes(), IniLoadType::Overwrite);
    if let Err(e) = rules.load_thing_templates_from_ini(&mut ini) {
        panic!("load thing templates at line {}: {e:?}", ini.get_line_num());
    }
    rules
}

fn upgrade_mask(sim: &SimContext, name: &str) -> UpgradeMaskType {
    sim.rules
        .upgrade_center
        .lookup_upgrade(name, &sim.rules.name_keys)
        .expect("upgrade")
        .upgrade_mask()
        .clone()
}

// -----------------------------------------------------------------------------
// World helpers mirroring the harness
// -----------------------------------------------------------------------------

const NEUTRAL: TeamId = TeamId(1);
const TEAM_A: TeamId = TeamId(2);
const TEAM_B: TeamId = TeamId(3);
const TEAMS: [TeamId; 3] = [NEUTRAL, TEAM_A, TEAM_B];

fn reset_world(sim: &mut SimContext, frame: u32, seed: u32, local_player: Int) {
    sim.world = SimWorld::new();
    let w = &mut sim.world;
    for (player, team) in TEAMS.into_iter().enumerate() {
        w.team_players.insert(team, player as Int);
        w.player_default_teams.insert(player as Int, team);
    }
    w.team_is_neutral.insert(NEUTRAL, TRUE);
    w.neutral_team_id = NEUTRAL;
    w.local_player_index = local_player;
    for i in 0..3 {
        for j in 0..3 {
            let rel = if i == j {
                Relationship::Allies
            } else if i == 0 || j == 0 {
                Relationship::Neutral
            } else {
                Relationship::Enemies
            };
            w.player_relationships.insert((i, j), rel);
        }
    }
    w.frame = frame;
    w.rng.init_random_with_seed(seed);
    // The harness world: 20-unit partition cells over its 1000 by 1000 terrain.
    sim.init_partition(
        20.0,
        Region3D {
            lo: Coord3D::new(0.0, 0.0, 0.0),
            hi: Coord3D::new(1000.0, 1000.0, 100.0),
        },
    );
}

fn team_index(team: TeamId) -> u32 {
    TEAMS.iter().position(|&t| t == team).expect("team") as u32
}

fn spawn_at(sim: &mut SimContext, name: &str, team: TeamId, pos: Coord3D) -> ObjectSlot {
    let tid = sim
        .rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"));
    let slot = sim.spawn_object_with_status(
        tid,
        team,
        Coord3D::default(),
        0.0,
        ObjectStatusMaskType::new(),
    );
    sim.set_object_position(slot, &pos);
    slot
}

fn spawn(sim: &mut SimContext, name: &str, team: TeamId) -> ObjectSlot {
    spawn_at(sim, name, team, Coord3D::default())
}

fn id_of(sim: &SimContext, slot: ObjectSlot) -> ObjectId {
    sim.object(slot).expect("object").id
}

fn module_named(sim: &mut SimContext, owner: ObjectSlot, name: &str) -> ModuleSlot {
    let key = sim.rules.name_keys.name_to_key(name);
    let obj = sim.world.objects.get(owner).expect("object");
    obj.behaviors
        .iter()
        .map(|&(ms, _)| ms)
        .find(|&ms| {
            sim.world
                .modules
                .payload(ms)
                .is_some_and(|p| p.module_name_key() == key)
        })
        .unwrap_or_else(|| panic!("missing module {name}"))
}

/// The object, unless it has been destroyed.
fn live(sim: &SimContext, slot: ObjectSlot) -> Option<ObjectSlot> {
    sim.object(slot)
        .is_some_and(|o| !o.is_destroyed())
        .then_some(slot)
}

fn is_dead(sim: &SimContext, slot: ObjectSlot) -> bool {
    sim.object(slot)
        .is_some_and(|o| o.is_effectively_dead() != FALSE)
}

fn health_bits(sim: &SimContext, slot: Option<ObjectSlot>) -> u32 {
    slot.map_or(0, |s| sim.object_health(s).to_bits())
}

fn rng_crc(sim: &SimContext) -> u32 {
    sim.world.rng.get_game_logic_random_seed_crc()
}

fn kill(sim: &mut SimContext, slot: ObjectSlot) {
    kill_object(sim, slot, DamageType::Unresistable, DeathType::Normal);
}

fn destroy_and_flush(sim: &mut SimContext, slot: ObjectSlot) {
    sim.destroy_object(slot);
    sim.process_destroy_list();
}

/// The harness's `step_single_update`.
fn step(sim: &mut SimContext, slot: ModuleSlot, owner: ObjectSlot, frame: u32) {
    sim.world.frame = frame;
    if live(sim, owner).is_none() {
        return;
    }
    if sim
        .module_schedule(slot)
        .expect("schedule")
        .next_call_frame()
        > frame
    {
        return;
    }
    let disabled = sim
        .world
        .objects
        .get(owner)
        .expect("owner")
        .disabled_mask
        .clone();
    let sleep = sim
        .dispatch_module(slot, |m, s, c| {
            let update = m.as_update_mut().expect("update facet");
            if update.disabled_types_to_process().test_for_all(&disabled) == FALSE {
                return UpdateSleepTime::NONE;
            }
            let sleep = update.update(s, c);
            if sleep.0 < 1 {
                UpdateSleepTime::NONE
            } else {
                sleep
            }
        })
        .expect("dispatch update");
    let next = frame.saturating_add(sleep.0).min(UPDATE_SLEEP_FOREVER);
    sim.world
        .modules
        .meta_mut(slot)
        .schedule
        .set_next_call_frame(next);
    let index = sim.world.modules.meta(slot).schedule.index_in_logic();
    if index >= 0 {
        sim.world
            .sleepy_updates
            .rebalance(&mut sim.world.modules, index as usize);
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn digest(bytes: &[u8]) -> String {
    let h = bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    });
    format!("{h:016x}")
}

fn save_module(sim: &mut SimContext, slot: ModuleSlot) -> Vec<u8> {
    let (mut payload, _ctx, generation) =
        sim.world.modules.take_payload(slot).expect("take payload");
    let mut xs = XferSave::new();
    payload.xfer(&mut xs, sim).expect("xfer save");
    sim.world.modules.restore_payload(slot, generation, payload);
    xs.as_bytes().to_vec()
}

fn module_crc(sim: &SimContext, slot: ModuleSlot) -> u32 {
    let mut xc = XferCrc::new();
    sim.world
        .modules
        .payload(slot)
        .expect("payload")
        .crc(&mut xc);
    xc.get_crc()
}

fn fnv32(entries: &[[u32; 3]]) -> u32 {
    entries
        .iter()
        .flatten()
        .fold(0x811c_9dc5_u32, |h, &w| (h ^ w).wrapping_mul(0x0100_0193))
}

/// The effect lists played so far, as the harness's nuggets record them.
fn fx_log(sim: &SimContext) -> Vec<[u32; 3]> {
    sim.world
        .triggered_fx_lists
        .iter()
        .map(|(name, pos)| {
            let index = FX_NAMES.iter().position(|n| n == name).expect("known FX");
            [index as u32, pos.x.to_bits(), pos.y.to_bits()]
        })
        .collect()
}

/// Shockwave weapons fired, as the harness sees them through the weapon's `FireFX`.
fn shock_log(sim: &SimContext) -> Vec<[u32; 3]> {
    sim.world
        .weapon_fires
        .iter()
        .map(|(_, name, target, pos)| {
            assert_eq!(name, SHOCK_WEAPON);
            assert_eq!(*target, None, "the shockwave is fired at a position");
            [2, pos.x.to_bits(), pos.y.to_bits()]
        })
        .collect()
}

// -----------------------------------------------------------------------------
// Vector execution
// -----------------------------------------------------------------------------

struct Input {
    kind: usize,
    variant: u32,
    frame: u32,
    seed: u32,
    p: Vec<u32>,
    ops: Vec<[u32; 5]>,
}

fn input(j: &Json) -> Input {
    Input {
        kind: j.u32("kind") as usize,
        variant: j.u32("variant"),
        frame: j.u32("frame"),
        seed: j.u32("seed"),
        p: j.get("p").arr().iter().map(Json::num).collect(),
        ops: j
            .get("ops")
            .arr()
            .iter()
            .map(|op| std::array::from_fn(|i| op.arr()[i].num()))
            .collect(),
    }
}

struct Row {
    xfer: Vec<u8>,
    gone: bool,
    obs: Vec<u32>,
}

struct Actual {
    rows: Vec<Row>,
    crc: u32,
    rng: u32,
}

fn row(sim: &mut SimContext, module: ModuleSlot, obs: Vec<u32>) -> Row {
    Row {
        xfer: save_module(sim, module),
        gone: false,
        obs,
    }
}

fn run_bridge(sim: &mut SimContext, v: &Input) -> (Vec<Row>, u32) {
    let (kind, present, listed) = (v.p[0], v.p[1], v.p[2]);
    let bridge = match kind {
        0 => Some(spawn(sim, &format!("P53C3_Bridge{}", v.variant), TEAM_A)),
        1 => Some(spawn(sim, &format!("P53C3_Plain{}", v.variant), TEAM_A)),
        _ => None,
    };
    let bridge_id = bridge.map(|b| id_of(sim, b));
    let mut tower = [None; 4];
    let mut module = [None; 4];
    for k in 0..4 {
        if present & (1 << k) == 0 {
            continue;
        }
        let t = spawn(sim, &format!("P53C3_Tower{}", v.p[3 + k]), TEAM_A);
        let m = module_named(sim, t, "BridgeTowerBehavior");
        tower[k] = Some(t);
        module[k] = Some(m);
        // What the unported `BridgeBehavior` does when it creates a tower.
        let tower_type = BridgeTowerType::from_i32(v.p[7 + k] as Int).expect("tower type");
        sim.dispatch_module(m, |b, _, _| {
            let f = b.as_bridge_tower_mut().expect("tower facet");
            f.set_tower_type(tower_type);
            if let Some(id) = bridge_id {
                f.set_bridge(id);
            }
        })
        .expect("link tower");
        if kind == 0 && listed & (1 << k) != 0 {
            let b = bridge.expect("bridge");
            let id = id_of(sim, t);
            let bm = module_named(sim, b, "BridgeBehavior");
            let slot_type = BridgeTowerType::from_i32(k as Int).expect("tower type");
            sim.dispatch_module(bm, |m, _, _| {
                m.as_bridge_mut()
                    .expect("bridge facet")
                    .set_tower(slot_type, id);
            })
            .expect("list tower");
        }
    }
    let plain = spawn(sim, "P53C3_Source", TEAM_B);

    let mut rows = Vec::new();
    for &[code, t, amount, source, damage_type] in &v.ops {
        let t = t as usize;
        let target = tower[t].expect("tower");
        let source_id = if code <= 1 {
            let src = match source {
                1 => Some(plain),
                2 => bridge,
                3 => tower[(t + 1) % 4],
                5 => tower[t],
                _ => None,
            };
            match src {
                Some(s) => id_of(sim, s),
                // An unknown id: damage keeps it, healing is given a null source.
                None if source == 4 && code == 0 => ObjectId(9999),
                None => ObjectId::INVALID,
            }
        } else {
            ObjectId::INVALID
        };
        let m = module[t].expect("module");
        match code {
            0 => {
                let mut d = DamageInfo::default();
                d.input.damage_type = DamageType::from_i32(damage_type as Int).expect("type");
                d.input.death_type = DeathType::Normal;
                d.input.amount = Real::from_bits(amount);
                d.input.source_id = source_id;
                sim.attempt_damage(target, &mut d);
            }
            1 => {
                let mut d = DamageInfo::default();
                d.input.damage_type = DamageType::Healing;
                d.input.death_type = DeathType::NoneType;
                d.input.amount = Real::from_bits(amount);
                d.input.source_id = source_id;
                sim.attempt_healing(target, &mut d);
            }
            2 => kill(sim, target),
            3 | 4 => {
                let id = if code == 3 {
                    Some(ObjectId::INVALID)
                } else {
                    bridge_id
                };
                if let Some(id) = id {
                    sim.dispatch_module(m, |b, _, _| {
                        b.as_bridge_tower_mut().expect("tower facet").set_bridge(id);
                    })
                    .expect("set bridge");
                }
            }
            other => panic!("op {other}"),
        }

        let mut dead = 0;
        for k in 0..4 {
            if tower[k].is_some_and(|s| is_dead(sim, s)) {
                dead |= 1 << k;
            }
        }
        if bridge.is_some_and(|s| is_dead(sim, s)) {
            dead |= 16;
        }
        let mut obs: Vec<u32> = tower.iter().map(|&s| health_bits(sim, s)).collect();
        obs.push(health_bits(sim, bridge));
        obs.push(dead);
        obs.push(rng_crc(sim));
        rows.push(row(sim, m, obs));
    }
    let first = module.into_iter().flatten().next().expect("a tower");
    let crc = module_crc(sim, first);
    (rows, crc)
}

fn run_bunker(sim: &mut SimContext, v: &Input) -> (Vec<Row>, u32) {
    let (kind, nriders, has_ai) = (v.p[0] as usize, v.p[1] as usize, v.p[5] != 0);
    let at = |x: u32, y: u32| Coord3D::new(Real::from_bits(x), Real::from_bits(y), 0.0);
    let mask = upgrade_mask(sim, "Upgrade_P53C3_Bust");
    let suffix = if has_ai { "A" } else { "N" };
    let bomb = spawn_at(
        sim,
        &format!("P53C3_Bomb{}{suffix}", v.variant),
        TEAM_A,
        at(v.p[6], v.p[7]),
    );
    let target_pos = at(v.p[8], v.p[9]);
    let names = ["", "P53C3_PlainBldg", "P53C3_Garr", "P53C3_Trans"];
    let target = (kind != 0).then(|| spawn_at(sim, names[kind], TEAM_B, target_pos));
    let other = spawn(sim, "P53C3_PlainBldg", TEAM_B);
    let mut rider = [None; 3];
    if kind >= 2 {
        for k in 0..nriders {
            let r = spawn_at(
                sim,
                &format!("P53C3_Rider{}", v.p[2 + k]),
                TEAM_B,
                target_pos,
            );
            rider[k] = Some(r);
            sim.contain_add(target.expect("container"), r);
        }
    }
    let m = module_named(sim, bomb, "BunkerBusterBehavior");

    sim.world.rng.init_random_with_seed(v.seed);
    let mut frame = v.frame;
    let mut rows = Vec::new();
    for &[code, a, ..] in &v.ops {
        // The harness updates the partition before each operation.
        sim.update_partition();
        let t = target.and_then(|s| live(sim, s));
        match code {
            0 if has_ai => {
                // The victim the unported AI would report.
                let victim = match a {
                    0 => t,
                    2 => Some(other),
                    _ => None,
                };
                sim.object_mut(bomb).expect("bomb").ai_current_victim = victim;
            }
            0 => {}
            1 => {
                for _ in 0..a {
                    frame += 1;
                    step(sim, m, bomb, frame);
                }
            }
            2 if a != 0 => sim.set_status(bomb, ObjectStatusType::MissileKillingSelf),
            2 => sim.clear_status(bomb, ObjectStatusType::MissileKillingSelf),
            3 => kill(sim, bomb),
            4 => {
                if let Some(t) = t {
                    for r in rider.into_iter().flatten() {
                        if sim.object(r).is_some_and(|o| o.contained_by == Some(t)) {
                            sim.contain_remove(t, r, FALSE);
                        }
                    }
                    destroy_and_flush(sim, t);
                }
            }
            5 => {
                let player = sim.player_index(TEAM_A);
                sim.world.player_upgrades.insert(player, mask.clone());
            }
            6 => {
                let player = sim.player_index(TEAM_A);
                sim.world.player_upgrades.remove(&player);
            }
            other => panic!("op {other}"),
        }

        let t = target.and_then(|s| live(sim, s));
        let (mut contained, mut dead, mut hp) = (0, 0, [0_u32; 3]);
        for k in 0..3 {
            let Some(r) = rider[k].and_then(|s| live(sim, s)) else {
                continue;
            };
            hp[k] = sim.object_health(r).to_bits();
            if t.is_some() && sim.object(r).expect("rider").contained_by == t {
                contained |= 1 << k;
            }
            if is_dead(sim, r) {
                dead |= 1 << k;
            }
        }
        if t.is_some_and(|s| is_dead(sim, s)) {
            dead |= 8;
        }
        if is_dead(sim, bomb) {
            dead |= 16;
        }
        let (fx, shock) = (fx_log(sim), shock_log(sim));
        let obs = vec![
            health_bits(sim, t),
            hp[0],
            hp[1],
            hp[2],
            contained,
            dead,
            t.map_or(3, |s| team_index(sim.object(s).expect("target").team_id)),
            fx.len() as u32,
            fnv32(&fx),
            shock.len() as u32,
            fnv32(&shock),
            rng_crc(sim),
        ];
        rows.push(row(sim, m, obs));
    }
    let crc = module_crc(sim, m);
    (rows, crc)
}

fn bonus_bits(sim: &SimContext, slot: Option<ObjectSlot>) -> u32 {
    let Some(slot) = slot else {
        return 4;
    };
    let flags = sim.object(slot).expect("object").weapon_bonus_condition;
    let bit = |c: WeaponBonusConditionType| u32::from(flags & (1 << c.as_i32()) != 0);
    bit(WeaponBonusConditionType::Enthusiastic) | (bit(WeaponBonusConditionType::Subliminal) << 1)
}

fn run_propaganda(sim: &mut SimContext, v: &Input) -> (Vec<Row>, u32) {
    let (team, container, second, count) =
        (TEAMS[v.p[0] as usize], v.p[1], v.p[2] != 0, v.p[3] as usize);
    let sub = upgrade_mask(sim, "Upgrade_P53C3_Sub");
    let name = format!("P53C3_Prop{}", v.variant);
    let center = Coord3D::new(200.0, 200.0, 0.0);
    let tower = spawn_at(sim, &name, team, center);
    let m = module_named(sim, tower, "PropagandaTowerBehavior");
    let fixed = save_module(sim, m).len();
    let second = second.then(|| {
        let t = spawn_at(sim, &name, team, center);
        (t, module_named(sim, t, "PropagandaTowerBehavior"))
    });
    let holder = (container != 0).then(|| {
        let name = if container == 1 {
            "P53C3_Trans"
        } else {
            "P53C3_OpenGarr"
        };
        spawn_at(sim, name, team, center)
    });
    let mut units = Vec::new();
    for k in 0..count {
        let u = &v.p[4 + k * 4..8 + k * 4];
        let pos = Coord3D::new(Real::from_bits(u[2]), Real::from_bits(u[3]), 0.0);
        units.push(spawn_at(
            sim,
            UNITS[u[0] as usize],
            TEAMS[u[1] as usize],
            pos,
        ));
    }

    sim.world.rng.init_random_with_seed(v.seed);
    let mut frame = v.frame;
    let mut rows = Vec::new();
    for &[code, a, b, c, _] in &v.ops {
        let t = live(sim, tower);
        let u = matches!(code, 1 | 2 | 5 | 6)
            .then(|| live(sim, units[a as usize]))
            .flatten();
        // The harness updates the partition before each operation.
        sim.update_partition();
        match code {
            0 => {
                for _ in 0..a {
                    frame += 1;
                    sim.world.frame = frame;
                    if live(sim, tower).is_some() {
                        step(sim, m, tower, frame);
                    }
                    if let Some((t2, m2)) = second {
                        step(sim, m2, t2, frame);
                    }
                }
            }
            1 => {
                if let Some(u) = u {
                    let mut d = DamageInfo::default();
                    d.input.damage_type = DamageType::Unresistable;
                    d.input.death_type = DeathType::Normal;
                    d.input.amount = Real::from_bits(b);
                    sim.attempt_damage(u, &mut d);
                }
            }
            2 => {
                if let Some(u) = u {
                    let pos = Coord3D::new(Real::from_bits(b), Real::from_bits(c), 0.0);
                    sim.set_object_position(u, &pos);
                }
            }
            3 => {
                if let Some(t) = t {
                    let status = [
                        ObjectStatusType::UnderConstruction,
                        ObjectStatusType::Sold,
                        ObjectStatusType::Stealthed,
                        ObjectStatusType::Detected,
                    ][a as usize];
                    if b != 0 {
                        sim.set_status(t, status);
                    } else {
                        sim.clear_status(t, status);
                    }
                }
            }
            4 => {
                if let Some(t) = t {
                    let kind = if a == 0 {
                        DisabledType::Held
                    } else {
                        DisabledType::Emp
                    };
                    if b != 0 {
                        sim.set_disabled(t, kind);
                    } else {
                        sim.clear_disabled(t, kind);
                    }
                }
            }
            5 => {
                if let Some(u) = u.filter(|&u| !is_dead(sim, u)) {
                    kill(sim, u);
                }
            }
            6 => {
                if let Some(u) = u {
                    destroy_and_flush(sim, u);
                }
            }
            7 => {
                let player = sim.player_index(team);
                if a != 0 {
                    sim.world.player_upgrades.insert(player, sub.clone());
                } else {
                    sim.world.player_upgrades.remove(&player);
                }
            }
            8 => {
                if let Some(t) = t {
                    sim.grant_upgrade(t, "Upgrade_P53C3_Obj", FALSE);
                }
            }
            9 => {
                if let Some(t) = t {
                    sim.capture_object(t, TEAMS[a as usize]);
                }
            }
            10 => {
                if let Some(t) = t.filter(|&t| !is_dead(sim, t)) {
                    kill(sim, t);
                }
            }
            11 => {
                if let (Some(t), Some(holder)) = (t.filter(|&t| !is_dead(sim, t)), holder) {
                    let inside = sim.object(t).expect("tower").contained_by;
                    if a != 0 && inside.is_none() {
                        sim.contain_add(holder, t);
                    }
                    if a == 0 && inside == Some(holder) {
                        sim.contain_remove(holder, t, FALSE);
                    }
                }
            }
            12 => {
                if let Some(t) = t {
                    destroy_and_flush(sim, t);
                }
            }
            other => panic!("op {other}"),
        }

        let t = live(sim, tower);
        let mut obs = Vec::new();
        for k in 0..4 {
            let o = units.get(k).and_then(|&s| live(sim, s));
            obs.push(health_bits(sim, o));
            obs.push(bonus_bits(sim, o));
        }
        obs.push(health_bits(sim, t));
        obs.push(bonus_bits(sim, t));
        let (xfer, next, tracked) = if t.is_some() {
            let next = sim.module_schedule(m).expect("schedule").next_call_frame();
            let raw = save_module(sim, m);
            let tracked = ((raw.len() - fixed) / 4) as u32;
            (Some(raw), next, tracked)
        } else {
            (None, 0, 0)
        };
        let fx = fx_log(sim);
        obs.extend([next, tracked, fx.len() as u32, fnv32(&fx), rng_crc(sim)]);
        rows.push(Row {
            gone: xfer.is_none(),
            xfer: xfer.unwrap_or_default(),
            obs,
        });
    }
    let crc = if live(sim, tower).is_some() {
        module_crc(sim, m)
    } else {
        0
    };
    (rows, crc)
}

fn run(sim: &mut SimContext, v: &Input, local_player: Int) -> Actual {
    reset_world(sim, v.frame, v.seed, local_player);
    let (rows, crc) = match v.kind {
        0 => run_bridge(sim, v),
        1 => run_bunker(sim, v),
        _ => run_propaganda(sim, v),
    };
    Actual {
        rows,
        crc,
        rng: rng_crc(sim),
    }
}

/// First difference between one vector's expected and actual outputs.
fn difference(kind: usize, expected: &Json, actual: &Actual) -> Option<String> {
    let rows = expected.get("rows").arr();
    if rows.len() != actual.rows.len() {
        return Some(format!("row count {} != {}", actual.rows.len(), rows.len()));
    }
    for (i, (want, got)) in rows.iter().zip(&actual.rows).enumerate() {
        let want = want.arr();
        for (k, name) in fields(kind).iter().enumerate() {
            if got.obs[k] != want[k + 1].num() {
                return Some(format!(
                    "row {i} {name}: got {} want {}",
                    got.obs[k],
                    want[k + 1].num()
                ));
            }
        }
        let got_xfer = if got.gone {
            "-".to_owned()
        } else {
            digest(&got.xfer)
        };
        if got_xfer != want[0].text() {
            return Some(format!(
                "row {i} xfer {} (digest {got_xfer})",
                hex(&got.xfer)
            ));
        }
    }
    let last = actual.rows.last().expect("row");
    let want_final = expected.get("final_xfer").text();
    if want_final != "-" && hex(&last.xfer) != want_final {
        return Some(format!("final_xfer {} want {want_final}", hex(&last.xfer)));
    }
    if actual.crc != expected.u32("crc") {
        return Some(format!("crc {} want {}", actual.crc, expected.u32("crc")));
    }
    if actual.rng != expected.u32("rng") {
        return Some(format!("rng {} want {}", actual.rng, expected.u32("rng")));
    }
    None
}

fn load_golden() -> Json {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/golden_vectors/WP53_C3.json");
    let raw = fs::read(path).expect("read WP53_C3.json");
    Parser { b: &raw, i: 0 }.value()
}

#[test]
fn wp53_c3_matches_reference() {
    let doc = load_golden();
    let vectors = doc.get("vectors").arr();
    assert_eq!(doc.u32("vector_count"), 1000);
    assert_eq!(vectors.len(), 1000);
    let diag = doc.get("seed0_diagnostics").obj();

    let mut sim = SimContext::new();
    sim.rules = build_rules();
    let local_player = doc.u32("local_player") as Int;
    let mut checked_diag = 0;
    let mut failures = [0_usize; 3];
    let mut first = Vec::new();
    for (index, vector) in vectors.iter().enumerate() {
        let v = input(vector.get("in"));
        let actual = run(&mut sim, &v, local_player);

        // Vector `k` is the first one of kind `k`.
        if index < 3 {
            let name = KINDS[v.kind];
            let mut got: Vec<(String, String)> = vec![
                (format!("{name}_rows"), actual.rows.len().to_string()),
                (format!("{name}_xfer"), hex(&actual.rows[0].xfer)),
            ];
            let last = actual.rows.last().expect("row");
            for (k, field) in fields(v.kind).iter().enumerate() {
                got.push((format!("{name}_{field}"), last.obs[k].to_string()));
            }
            for (key, value) in got {
                assert_eq!(diag[&key].text(), value, "seed0 diagnostic {key}");
                checked_diag += 1;
            }
        }

        if let Some(why) = difference(v.kind, vector.get("out"), &actual) {
            failures[v.kind] += 1;
            if failures[v.kind] <= 2 {
                first.push(format!("vector {index} ({}): {why}", KINDS[v.kind]));
            }
        }
    }
    assert_eq!(
        checked_diag,
        diag.len(),
        "every seed0 diagnostic is compared"
    );
    assert!(
        failures.iter().all(|&n| n == 0),
        "mismatches per module {failures:?}\n{}",
        first.join("\n")
    );
}

/// A loaded `PropagandaTowerBehavior` rebuilds its list at the head, so saving it again
/// writes the ids reversed.
#[test]
fn propaganda_load_reverses_the_tracked_ids() {
    let mut sim = SimContext::new();
    sim.rules = build_rules();
    reset_world(&mut sim, 100, 1, 1);
    let tower = spawn(&mut sim, "P53C3_Prop0", TEAM_A);
    let m = module_named(&mut sim, tower, "PropagandaTowerBehavior");
    spawn(&mut sim, "P53C3_URifle", TEAM_A);
    spawn(&mut sim, "P53C3_URifle", TEAM_A);
    sim.update_partition();
    step(&mut sim, m, tower, 101);
    let saved = save_module(&mut sim, m);
    let fixed = saved.len() - 8;

    let fresh_tower = spawn(&mut sim, "P53C3_Prop0", TEAM_A);
    let fresh = module_named(&mut sim, fresh_tower, "PropagandaTowerBehavior");
    let (mut payload, _ctx, generation) =
        sim.world.modules.take_payload(fresh).expect("take payload");
    let mut load = XferLoad::new(&saved);
    payload.xfer(&mut load, &sim).expect("xfer load");
    let mut resave = XferSave::new();
    // `Load` keeps the saved schedule; saving through a view would resync the live one.
    let empty = SimContext::new();
    payload.xfer(&mut resave, &empty).expect("xfer resave");
    sim.world
        .modules
        .restore_payload(fresh, generation, payload);

    let mut reversed = saved[..fixed].to_vec();
    reversed.extend_from_slice(&saved[fixed + 4..]);
    reversed.extend_from_slice(&saved[fixed..fixed + 4]);
    assert_eq!(hex(resave.as_bytes()), hex(&reversed));
}

/// Contract test, not an oracle vector: `OpenContain::addOrRemoveObjFromWorld` also takes the
/// non-enclosed riders of a unit that is shut inside an enclosing container out of range scans.
#[test]
fn range_scan_skips_riders_nested_in_an_enclosed_container() {
    let mut sim = SimContext::new();
    sim.rules = build_rules();
    reset_world(&mut sim, 100, 1, 1);
    let transport = spawn(&mut sim, "P53C3_Trans", TEAM_A);
    let open = spawn(&mut sim, "P53C3_OpenGarr", TEAM_A);
    let rider = spawn(&mut sim, "P53C3_URifle", TEAM_A);
    let seen = |sim: &SimContext| {
        let found = sim.find_objects_in_range(&Coord3D::default(), 10.0);
        [open, rider].map(|slot| found.contains(&slot))
    };

    sim.contain_add(open, rider);
    sim.update_partition();
    assert_eq!(seen(&sim), [true, true], "an open garrison hides nothing");
    sim.contain_add(transport, open);
    assert_eq!(seen(&sim), [false, false], "both leave with the transport");
}
