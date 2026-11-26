//! Gate V2 differential verification for Wave `WP-5.3-C1`:
//! `PKT-261 BridgeScaffoldBehavior`, `PKT-283 OverchargeBehavior`, `PKT-287 PoisonedBehavior`,
//! `PKT-292 RailedTransportContain`, `PKT-297 SupplyWarehouseCripplingBehavior`,
//! `PKT-298 TechBuildingBehavior` against `docs/golden_vectors/WP53_C1.json`
//! (`tests/cpp_harness/gen_wp53_c1.cpp`).
//!
//! The reference library saves `PoisonedBehavior` version 2 (`RETAIL_COMPATIBLE_XFER_SAVE=1`);
//! this crate's default writes version 3, which appends the poison source. The test removes
//! those four bytes and compares them with the reference's `m_poisonSource` column.
//!
//! `PowerPlantUpdate` and the dock updates in the C++ fixture are not ported; the Rust side
//! records the calls made into them.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{
    Bool, Int, ObjectId, Real, Relationship, TeamId, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{XferCrc, XferLoad, XferSave};
use zh_logic::arena::{ModuleSlot, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::modules::behavior::bridge_scaffold_behavior::BridgeScaffoldBehavior;
use zh_logic::modules::module::{ExitDoorType, ScaffoldTargetMotion};
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::model_state::ModelConditionFlagType;
use zh_logic::rules::object_status_types::ObjectStatusMaskType;
use zh_logic::scheduler::UPDATE_SLEEP_FOREVER;
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

const KINDS: [&str; 6] = [
    "scaffold",
    "overcharge",
    "poisoned",
    "railed",
    "supply",
    "tech",
];

fn fields(kind: usize) -> &'static [&'static str] {
    match kind {
        0 => &["px", "py", "pz", "destroyed", "motion"],
        1 => &[
            "health",
            "prod_a",
            "prod_b",
            "extended",
            "destroyed",
            "active",
        ],
        2 => &["health", "dead", "source"],
        3 => &[
            "count",
            "dock_open",
            "door",
            "alive_mask",
            "contained_mask",
            "destroyed",
        ],
        4 => &["health", "crippled", "damage_state"],
        _ => &["captured", "team", "health"],
    }
}

fn body(health: &str) -> String {
    format!("  Body = ActiveBody ModuleTag_Body\n    MaxHealth = {health}\n    InitialHealth = {health}\n  End\n")
}

fn objects_ini() -> String {
    let mut s = String::new();
    let _ = write!(
        s,
        "Object P53C_Scaffold\n  KindOf = PRELOAD IMMOBILE\n{}  Behavior = BridgeScaffoldBehavior ModuleTag_Scaffold\n  End\nEnd\n",
        body("100.0")
    );
    let drain = ["2%", "10%", "100%", "25%"];
    let floor = ["50%", "25%", "0%", "90%"];
    let bonus = [50, 7, 3, 100];
    for i in 0..4 {
        let _ = write!(
            s,
            "Object P53C_Over{i}\n  KindOf = PRELOAD STRUCTURE\n  EnergyProduction = 0\n  EnergyBonus = {}\n{}\
             \x20 Behavior = PowerPlantUpdate ModuleTag_Rods\n    RodsExtendTime = 600\n  End\n\
             \x20 Behavior = OverchargeBehavior ModuleTag_Over\n\
             \x20   HealthPercentToDrainPerSecond = {}\n\
             \x20   NotAllowedWhenHealthBelowPercent = {}\n  End\nEnd\n",
            bonus[i], body("1000.0"), drain[i], floor[i]
        );
    }
    let interval = [333, 1000, 100, 0, 500];
    let duration = [3000, 500, 2000, 1000, 0];
    for i in 0..5 {
        let _ = write!(
            s,
            "Object P53C_Poison{i}\n  KindOf = PRELOAD INFANTRY\n{}  Behavior = PoisonedBehavior ModuleTag_Poison\n    PoisonDamageInterval = {}\n    PoisonDuration = {}\n  End\nEnd\n",
            body("200.0"), interval[i], duration[i]
        );
    }
    let _ = write!(
        s,
        "Object P53C_Rider\n  KindOf = PRELOAD INFANTRY\n  TransportSlotCount = 1\n{}End\n",
        body("50.0")
    );
    for i in 0..8 {
        let _ = write!(
            s,
            "Object P53C_Rail{i}\n  KindOf = PRELOAD VEHICLE\n{}  Behavior = RailedTransportContain ModuleTag_Contain\n    Slots = {}\n    AllowInsideKindOf = INFANTRY\n    ExitDelay = 100\n    DestroyRidersWhoAreNotFreeToExit = {}\n  End\n",
            body("300.0"),
            if i & 1 != 0 { 6 } else { 2 },
            if i & 2 != 0 { "Yes" } else { "No" }
        );
        if i & 4 == 0 {
            s.push_str("  Behavior = RepairDockUpdate ModuleTag_Dock\n  End\n");
        }
        s.push_str("End\n");
    }
    let suppression = [1000, 3000, 33, 500];
    let delay = [500, 100, 33, 1000];
    let amount = ["50.0", "7.5", "2000.0", "0.0"];
    for i in 0..4 {
        let _ = write!(
            s,
            "Object P53C_Supply{i}\n  KindOf = PRELOAD STRUCTURE\n{}  Behavior = RepairDockUpdate ModuleTag_Dock\n  End\n  Behavior = SupplyWarehouseCripplingBehavior ModuleTag_Cripple\n    SelfHealSupression = {}\n    SelfHealDelay = {}\n    SelfHealAmount = {}\n  End\nEnd\n",
            body("1000.0"), suppression[i], delay[i], amount[i]
        );
    }
    let tech = [
        "",
        "    PulseFX = FX_P53C_Pulse\n    PulseFXRate = 1000\n",
        "    PulseFX = FX_P53C_Pulse\n",
        "    PulseFXRate = 500\n",
    ];
    for (i, extra) in tech.iter().enumerate() {
        let _ = write!(
            s,
            "Object P53C_Tech{i}\n  KindOf = PRELOAD STRUCTURE\n{}  Behavior = TechBuildingBehavior ModuleTag_Tech\n{extra}  End\nEnd\n",
            body("500.0")
        );
    }
    s
}

fn build_rules() -> RulesRegistry {
    let mut rules = RulesRegistry::new();
    rules.fx_list_names.insert("FX_P53C_Pulse".to_owned());
    rules.global_data.unit_damaged_thresh = 0.7;
    rules.global_data.unit_really_damaged_thresh = 0.35;
    let text = objects_ini();
    let mut ini = IniReader::new("object.ini", text.as_bytes(), IniLoadType::Overwrite);
    if let Err(e) = rules.load_thing_templates_from_ini(&mut ini) {
        panic!("load thing templates at line {}: {e:?}", ini.get_line_num());
    }
    rules
}

// -----------------------------------------------------------------------------
// World helpers mirroring the harness
// -----------------------------------------------------------------------------

const NEUTRAL: TeamId = TeamId(1);
const TEAM_A: TeamId = TeamId(2);
const TEAM_B: TeamId = TeamId(3);
const TEAMS: [TeamId; 3] = [NEUTRAL, TEAM_A, TEAM_B];

fn reset_world(sim: &mut SimContext, frame: u32, seed: u32) {
    sim.world = SimWorld::new();
    let w = &mut sim.world;
    for (player, team) in TEAMS.into_iter().enumerate() {
        w.team_players.insert(team, player as Int);
        w.player_default_teams.insert(player as Int, team);
    }
    w.team_is_neutral.insert(NEUTRAL, TRUE);
    w.neutral_team_id = NEUTRAL;
    w.playable_side_teams.insert(TEAM_A);
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
}

fn spawn(sim: &mut SimContext, name: &str, team: TeamId) -> ObjectSlot {
    let tid = sim
        .rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"));
    sim.spawn_object_with_status(
        tid,
        team,
        Coord3D::default(),
        0.0,
        ObjectStatusMaskType::new(),
    )
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

fn is_destroyed(sim: &SimContext, slot: ObjectSlot) -> bool {
    sim.object(slot).is_none_or(|o| o.is_destroyed())
}

fn is_dead(sim: &SimContext, slot: ObjectSlot) -> bool {
    sim.object(slot)
        .is_none_or(|o| o.is_effectively_dead() != FALSE)
}

/// The harness's `step_single_update`.
fn step(sim: &mut SimContext, slot: ModuleSlot, owner: ObjectSlot, frame: u32) {
    sim.world.frame = frame;
    if is_destroyed(sim, owner) {
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

fn hurt(
    sim: &mut SimContext,
    slot: ObjectSlot,
    kind: DamageType,
    amount: Real,
    source: u32,
    death: DeathType,
) {
    let mut d = DamageInfo::default();
    d.input.damage_type = kind;
    d.input.death_type = death;
    d.input.amount = amount;
    d.input.source_id = ObjectId(source as Int);
    sim.attempt_damage_on(slot, &mut d);
}

fn heal(sim: &mut SimContext, slot: ObjectSlot, amount: Real) {
    let mut d = DamageInfo::default();
    d.input.damage_type = DamageType::Healing;
    d.input.death_type = DeathType::NoneType;
    d.input.amount = amount;
    sim.attempt_healing(slot, &mut d);
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

fn run_scaffold(sim: &mut SimContext, v: &Input) -> (Vec<Row>, u32) {
    let o = spawn(sim, "P53C_Scaffold", TEAM_A);
    let coord = |k: usize| {
        Coord3D::new(
            Real::from_bits(v.p[k * 3]),
            Real::from_bits(v.p[k * 3 + 1]),
            Real::from_bits(v.p[k * 3 + 2]),
        )
    };
    sim.set_object_position(o, &coord(3));
    let m = module_named(sim, o, "BridgeScaffoldBehavior");
    sim.dispatch_module(m, |b, _, _| {
        let f = b.as_bridge_scaffold_mut().expect("scaffold");
        f.set_positions(&coord(0), &coord(1), &coord(2));
        f.set_lateral_speed(Real::from_bits(v.p[12]));
        f.set_vertical_speed(Real::from_bits(v.p[13]));
    })
    .expect("setup");
    let motions = [
        ScaffoldTargetMotion::Still,
        ScaffoldTargetMotion::Rise,
        ScaffoldTargetMotion::BuildAcross,
        ScaffoldTargetMotion::TearDownAcross,
        ScaffoldTargetMotion::Sink,
    ];
    let mut frame = v.frame;
    let mut rows = Vec::new();
    for &[code, a, ..] in &v.ops {
        if code == 2 {
            for _ in 0..a {
                frame += 1;
                step(sim, m, o, frame);
            }
        } else {
            sim.dispatch_module(m, |b, _, _| {
                let f = b.as_bridge_scaffold_mut().expect("scaffold");
                match code {
                    0 => f.set_motion(motions[a as usize]),
                    1 => f.reverse_motion(),
                    3 => f.set_lateral_speed(Real::from_bits(a)),
                    4 => f.set_vertical_speed(Real::from_bits(a)),
                    _ => panic!("op"),
                }
            })
            .expect("op");
        }
        let pos = *sim.object(o).expect("object").thing.get_position();
        let motion = sim
            .world
            .modules
            .payload(m)
            .and_then(|p| p.as_any())
            .and_then(|a| a.downcast_ref::<BridgeScaffoldBehavior>())
            .expect("scaffold")
            .target_motion as u32;
        let obs = vec![
            pos.x.to_bits(),
            pos.y.to_bits(),
            pos.z.to_bits(),
            u32::from(is_destroyed(sim, o)),
            motion,
        ];
        rows.push(row(sim, m, obs));
    }
    let crc = module_crc(sim, m);
    (rows, crc)
}

fn run_overcharge(sim: &mut SimContext, v: &Input) -> (Vec<Row>, u32) {
    let o = spawn(sim, &format!("P53C_Over{}", v.variant), TEAM_A);
    let m = module_named(sim, o, "OverchargeBehavior");
    let id = sim.object(o).expect("object").id;
    let crc = module_crc(sim, m);
    let mut frame = v.frame;
    let mut rows = Vec::new();
    let mut gone = false;
    let mut last = save_module(sim, m);
    let (mut health, mut extended, mut active) = (0, 0, 0);
    for &[code, a, ..] in &v.ops {
        if !gone {
            match code {
                0 => sim
                    .dispatch_module(m, |b, s, c| b.as_overcharge_mut().expect("oc").toggle(s, c))
                    .expect("toggle"),
                1 => sim
                    .dispatch_module(m, |b, s, c| {
                        b.as_overcharge_mut().expect("oc").enable(s, c, a as Bool)
                    })
                    .expect("enable"),
                2 => {
                    for _ in 0..a {
                        frame += 1;
                        step(sim, m, o, frame);
                    }
                }
                3 => sim.set_disabled(o, DisabledType::Emp),
                4 => {
                    sim.clear_disabled(o, DisabledType::Emp);
                }
                5 => {
                    let next = if sim.object(o).expect("object").team_id == TEAM_A {
                        TEAM_B
                    } else {
                        TEAM_A
                    };
                    sim.capture_object(o, next);
                }
                6 => hurt(
                    sim,
                    o,
                    DamageType::Explosion,
                    Real::from_bits(a),
                    0,
                    DeathType::Normal,
                ),
                7 => heal(sim, o, Real::from_bits(a)),
                8 => sim
                    .dispatch_module(m, |b, s, c| b.load_post_process(s, c))
                    .expect("load post process"),
                9 => {}
                _ => panic!("op"),
            }
            last = save_module(sim, m);
            health = sim.object_health(o).to_bits();
            extended = u32::from(sim.world.power_plant_extended.contains(&id));
            active = u32::from(
                sim.dispatch_module(m, |b, _, _| {
                    b.as_overcharge().expect("oc").is_overcharge_active()
                })
                .expect("active"),
            );
            if code == 9 {
                sim.destroy_object(o);
                sim.process_destroy_list();
                gone = true;
            }
        }
        let obs = vec![
            if gone { 0 } else { health },
            sim.player_energy_production(TEAM_A) as u32,
            sim.player_energy_production(TEAM_B) as u32,
            if gone { 0 } else { extended },
            u32::from(gone),
            if gone { 0 } else { active },
        ];
        rows.push(Row {
            xfer: last.clone(),
            gone,
            obs,
        });
    }
    (rows, crc)
}

fn run_poisoned(sim: &mut SimContext, v: &Input) -> (Vec<Row>, u32) {
    let o = spawn(sim, &format!("P53C_Poison{}", v.variant), TEAM_A);
    let m = module_named(sim, o, "PoisonedBehavior");
    let mut frame = v.frame;
    let mut rows = Vec::new();
    for &[code, a, b, c, d] in &v.ops {
        match code {
            0 => {
                let kind = if a != 0 {
                    DamageType::Poison
                } else {
                    DamageType::Explosion
                };
                let death = DeathType::from_i32(d as Int).expect("death type");
                hurt(sim, o, kind, Real::from_bits(b), c, death);
            }
            1 => heal(sim, o, Real::from_bits(a)),
            2 => {
                for _ in 0..a {
                    frame += 1;
                    step(sim, m, o, frame);
                }
            }
            3 => {
                frame += a;
                step(sim, m, o, frame);
            }
            4 => sim.set_disabled(o, DisabledType::Emp),
            5 => {
                sim.clear_disabled(o, DisabledType::Emp);
            }
            _ => panic!("op"),
        }
        let obs = vec![sim.object_health(o).to_bits(), u32::from(is_dead(sim, o))];
        let mut r = row(sim, m, obs);
        // Version 3 bytes to the reference's version 2 layout.
        assert_eq!(r.xfer[0], 3, "PoisonedBehavior save version");
        let source = r.xfer.split_off(r.xfer.len() - 4);
        r.xfer[0] = 2;
        r.obs
            .push(u32::from_le_bytes(source.try_into().expect("source id")));
        rows.push(r);
    }
    let crc = module_crc(sim, m);
    (rows, crc)
}

fn run_railed(sim: &mut SimContext, v: &Input) -> (Vec<Row>, u32) {
    let o = spawn(sim, &format!("P53C_Rail{}", v.variant), TEAM_A);
    let m = module_named(sim, o, "RailedTransportContain");
    let has_dock = v.variant & 4 == 0;
    let mut riders: Vec<ObjectSlot> = Vec::new();
    let mut frame = v.frame;
    let mut rows = Vec::new();
    let contained = |sim: &SimContext, r: ObjectSlot| {
        sim.object(r).is_some_and(|rec| rec.contained_by == Some(o))
    };
    for &[code, a, ..] in &v.ops {
        let mut door = 99_u32;
        let chosen = riders.get(a as usize % riders.len().max(1)).copied();
        match (code, chosen) {
            (0, _) => {
                if riders.len() < 24 {
                    let r = spawn(sim, "P53C_Rider", TEAM_A);
                    riders.push(r);
                    if sim.is_valid_container_for(o, r, TRUE) != FALSE {
                        sim.contain_add(o, r);
                    }
                }
            }
            (1, Some(r)) => {
                if contained(sim, r) {
                    sim.contain_remove(o, r, FALSE);
                }
            }
            (2, _) => {
                if has_dock {
                    sim.set_dock_open(o, a as Bool);
                }
            }
            (3, Some(r)) => {
                let tmpl = sim
                    .rules
                    .thing_factory
                    .find_template_id("P53C_Rider")
                    .expect("rider template");
                let reserved = sim
                    .dispatch_module(m, |b, s, c| {
                        b.as_exit_mut()
                            .expect("exit")
                            .reserve_door_for_exit(s, c, tmpl, Some(r))
                    })
                    .expect("reserve");
                door = (reserved as Int) as u32;
            }
            (4, Some(r)) => {
                if contained(sim, r) {
                    sim.dispatch_module(m, |b, s, c| {
                        b.as_exit_mut().expect("exit").exit_object_via_door(
                            s,
                            c,
                            r,
                            ExitDoorType::Door1,
                        )
                    })
                    .expect("exit via door");
                }
            }
            (5, _) => {
                for _ in 0..a {
                    frame += 1;
                    step(sim, m, o, frame);
                }
            }
            (6, _) => hurt(
                sim,
                o,
                DamageType::Unresistable,
                100_000.0,
                0,
                DeathType::Normal,
            ),
            (1 | 3 | 4, None) => {}
            _ => panic!("op"),
        }
        let (mut alive, mut inside) = (0_u32, 0_u32);
        for (k, &r) in riders.iter().enumerate() {
            if !is_destroyed(sim, r) && !is_dead(sim, r) {
                alive |= 1 << k;
            }
            if contained(sim, r) {
                inside |= 1 << k;
            }
        }
        let obs = vec![
            sim.contain_count(o),
            u32::from(!has_dock || sim.dock_is_open(o) != FALSE),
            door,
            alive,
            inside,
            u32::from(is_destroyed(sim, o)),
        ];
        rows.push(row(sim, m, obs));
    }
    let crc = module_crc(sim, m);
    (rows, crc)
}

fn run_supply(sim: &mut SimContext, v: &Input) -> (Vec<Row>, u32) {
    let o = spawn(sim, &format!("P53C_Supply{}", v.variant), TEAM_A);
    let m = module_named(sim, o, "SupplyWarehouseCripplingBehavior");
    let id = sim.object(o).expect("object").id;
    let mut frame = v.frame;
    let mut rows = Vec::new();
    for &[code, a, ..] in &v.ops {
        match code {
            0 => hurt(
                sim,
                o,
                DamageType::Explosion,
                Real::from_bits(a),
                0,
                DeathType::Normal,
            ),
            1 => {
                for _ in 0..a {
                    frame += 1;
                    step(sim, m, o, frame);
                }
            }
            2 => heal(sim, o, Real::from_bits(a)),
            3 => {
                frame += a;
                step(sim, m, o, frame);
            }
            _ => panic!("op"),
        }
        let obs = vec![
            sim.object_health(o).to_bits(),
            u32::from(sim.world.dock_crippled.contains(&id)),
            sim.object_body_damage_state(o) as u32,
        ];
        rows.push(row(sim, m, obs));
    }
    let crc = module_crc(sim, m);
    (rows, crc)
}

fn run_tech(sim: &mut SimContext, v: &Input) -> (Vec<Row>, u32) {
    let o = spawn(
        sim,
        &format!("P53C_Tech{}", v.variant),
        TEAMS[v.p[0] as usize],
    );
    let m = module_named(sim, o, "TechBuildingBehavior");
    let mut frame = v.frame;
    let mut rows = Vec::new();
    for &[code, a, ..] in &v.ops {
        match code {
            0 => {
                for _ in 0..a {
                    frame += 1;
                    step(sim, m, o, frame);
                }
            }
            1 => {
                sim.world.frame = frame;
                sim.capture_object(o, TEAMS[a as usize]);
            }
            2 => {
                sim.world.frame = frame;
                hurt(
                    sim,
                    o,
                    DamageType::Unresistable,
                    100_000.0,
                    0,
                    DeathType::Normal,
                );
            }
            _ => panic!("op"),
        }
        let rec = sim.object(o).expect("object");
        let captured = u32::from(
            rec.model_condition
                .test(ModelConditionFlagType::Captured as Int)
                != FALSE,
        );
        let team = TEAMS.iter().position(|&t| t == rec.team_id).expect("team") as u32;
        let obs = vec![captured, team, sim.object_health(o).to_bits()];
        rows.push(row(sim, m, obs));
    }
    let crc = module_crc(sim, m);
    (rows, crc)
}

fn run(sim: &mut SimContext, v: &Input) -> Actual {
    reset_world(sim, v.frame, v.seed);
    let (rows, crc) = match v.kind {
        0 => run_scaffold(sim, v),
        1 => run_overcharge(sim, v),
        2 => run_poisoned(sim, v),
        3 => run_railed(sim, v),
        4 => run_supply(sim, v),
        _ => run_tech(sim, v),
    };
    Actual {
        rows,
        crc,
        rng: sim.world.rng.get_game_logic_random_seed_crc(),
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
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/golden_vectors/WP53_C1.json");
    let raw = fs::read(path).expect("read WP53_C1.json");
    Parser { b: &raw, i: 0 }.value()
}

#[test]
fn wp53_c1_matches_reference() {
    let doc = load_golden();
    let vectors = doc.get("vectors").arr();
    assert_eq!(doc.u32("vector_count"), 1000);
    assert_eq!(vectors.len(), 1000);
    let diag = doc.get("seed0_diagnostics").obj();

    let mut sim = SimContext::new();
    sim.rules = build_rules();
    let mut checked_diag = 0;
    let mut failures = [0_usize; 6];
    let mut first = Vec::new();
    for (index, vector) in vectors.iter().enumerate() {
        let v = input(vector.get("in"));
        let actual = run(&mut sim, &v);

        // Vector `k` is the first one of kind `k`.
        if index < 6 {
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

/// Loading a module's saved bytes into a fresh instance and saving again reproduces them.
#[test]
fn wp53_c1_xfer_load_round_trips() {
    let doc = load_golden();
    let mut sim = SimContext::new();
    sim.rules = build_rules();
    let modules = [
        ("P53C_Scaffold", "BridgeScaffoldBehavior"),
        ("P53C_Over0", "OverchargeBehavior"),
        ("P53C_Poison0", "PoisonedBehavior"),
        ("P53C_Rail0", "RailedTransportContain"),
        ("P53C_Supply0", "SupplyWarehouseCripplingBehavior"),
        ("P53C_Tech1", "TechBuildingBehavior"),
    ];
    for (kind, (template, name)) in modules.into_iter().enumerate() {
        // Run the first vector of this kind for non-default state.
        let v = input(doc.get("vectors").arr()[kind].get("in"));
        reset_world(&mut sim, v.frame, v.seed);
        let source = spawn(&mut sim, template, TEAM_A);
        let m = module_named(&mut sim, source, name);
        match kind {
            0 => sim
                .dispatch_module(m, |b, _, _| {
                    let f = b.as_bridge_scaffold_mut().expect("scaffold");
                    let p = Coord3D::new(1.0, 2.0, 3.0);
                    f.set_positions(&p, &p, &p);
                    f.set_motion(ScaffoldTargetMotion::Sink);
                })
                .expect("setup"),
            1 => sim
                .dispatch_module(m, |b, s, c| b.as_overcharge_mut().expect("oc").toggle(s, c))
                .expect("toggle"),
            2 => hurt(
                &mut sim,
                source,
                DamageType::Poison,
                5.0,
                4001,
                DeathType::Burned,
            ),
            // An empty container: loaded passenger ids are only resolved in `loadPostProcess`.
            4 => hurt(
                &mut sim,
                source,
                DamageType::Explosion,
                5.0,
                0,
                DeathType::Normal,
            ),
            _ => {}
        }
        let saved = save_module(&mut sim, m);

        let target = spawn(&mut sim, template, TEAM_A);
        let fresh = module_named(&mut sim, target, name);
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
        assert_eq!(hex(resave.as_bytes()), hex(&saved), "{name}");
    }
}
