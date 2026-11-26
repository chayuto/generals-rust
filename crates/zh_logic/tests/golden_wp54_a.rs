//! Gate V2 differential verification for the `WP-5.4-A` update modules (`PKT-307`/`PKT-308`
//! `BaseRegenerateUpdate`, `PKT-316`, `PKT-331`, `PKT-339`, `PKT-356`, `PKT-366`) against
//! `docs/golden_vectors/WP54_A.json` (`tests/cpp_harness/gen_wp54_a.cpp`).
//!
//! Stand-ins for unported code:
//! - the test calls `go_prone`, `set_target_position`, `set_lifetime_range` and `set_enabled`
//!   itself (their C++ callers are not ported);
//! - the terrain has water everywhere, at a height that depends on `x`;
//! - the test sets the under-construction, sold, no-attack and held states, and changes the
//!   base-regeneration globals mid-run.
//!
//! Left out on purpose:
//! - `FloatUpdate` over dry ground, where C++ writes an uninitialised `waterZ` to the object;
//! - the client-only `FloatUpdate` drawable bobbing;
//! - lifetime ranges long enough to reach `UPDATE_SLEEP_FOREVER`, hulk overrides below `-1`
//!   and prone damage outside the `Int` range;
//! - objects without a drawable for `ProneUpdate` (C++ dereferences it unchecked);
//! - save/load round trips.
//!
//! Also not covered: disabled types other than `Held`, base-regeneration damage types other than
//! explosion, healing and unresistable, a `FloatUpdate` with no `Enabled` line, and the real
//! scheduler order (the test steps modules by hand).

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord3D, Matrix3D};
use zh_common::types::{Bool, Int, Real, TeamId, FALSE, TRUE};
use zh_common::xfer::{XferCrc, XferSave};
use zh_logic::arena::{ModuleContext, ModuleSlot, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::modules::contain::open_contain::kill_object;
use zh_logic::modules::update::deletion_update::DeletionUpdate;
use zh_logic::modules::update::float_update::FloatUpdate;
use zh_logic::modules::update::lifetime_update::LifetimeUpdate;
use zh_logic::modules::update::prone_update::ProneUpdate;
use zh_logic::modules::update::smart_bomb_target_homing_update::SmartBombTargetHomingUpdate;
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::pathfinder::PathfindLayerEnum;
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::model_state::ModelConditionFlagType;
use zh_logic::rules::object_status_types::{ObjectStatusMaskType, ObjectStatusType};
use zh_logic::scheduler::UPDATE_SLEEP_FOREVER;
use zh_logic::sim::{FlatTerrain, RulesRegistry, SimApi, SimContext, SimView, SimWorld};

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
// Fixture (the harness's `fixture_ini` and `WaterTerrain`)
// -----------------------------------------------------------------------------

const KINDS: [&str; 6] = [
    "deletion",
    "lifetime",
    "base_regen",
    "float",
    "prone",
    "smart_bomb",
];
const MODULES: [&str; 6] = [
    "DeletionUpdate",
    "LifetimeUpdate",
    "BaseRegenerateUpdate",
    "FloatUpdate",
    "ProneUpdate",
    "SmartBombTargetHomingUpdate",
];
const FIELDS: [&str; 8] = ["state", "x", "y", "z", "health", "flags", "wake", "rng"];
const VARIANTS: [usize; 6] = [8, 16, 4, 2, 8, 8];
const RANGES: [(i32, i32); 8] = [
    (-1, -1),
    (0, 0),
    (100, 100),
    (500, 3000),
    (3000, 500),
    (33, 34),
    (10000, 60000),
    (-1, 200),
];
const HEALTH: [&str; 4] = ["100.0", "333.3", "1000.0", "7.0"];
const RATIO: [&str; 8] = ["", "0.0", "1.0", "0.5", "2.5", "-1.0", "0.1", "0.3333"];
const SCALAR: [&str; 8] = ["", "-0.5", "0.0", "0.25", "0.5", "0.99", "1.0", "1.5"];

struct WaterTerrain {
    level: u32,
}

impl ThingTerrainAccess for WaterTerrain {
    fn get_ground_height(&self, x: Real, y: Real, normal: Option<&mut Coord3D>) -> Real {
        FlatTerrain.get_ground_height(x, y, normal)
    }

    fn align_on_terrain(
        &self,
        angle: Real,
        pos: &Coord3D,
        stick_to_ground: Bool,
        mtx: &mut Matrix3D,
    ) -> PathfindLayerEnum {
        FlatTerrain.align_on_terrain(angle, pos, stick_to_ground, mtx)
    }

    fn is_underwater(
        &self,
        x: Real,
        _y: Real,
        water_z: Option<&mut Real>,
        terrain_z: Option<&mut Real>,
    ) -> Bool {
        if let Some(wz) = water_z {
            *wz = Real::from_bits(self.level) + x * 0.125;
        }
        if let Some(tz) = terrain_z {
            *tz = 0.0;
        }
        TRUE
    }
}

fn objects_ini() -> String {
    let mut s = String::new();
    for kind in 0..6 {
        for i in 0..VARIANTS[kind] {
            let hulk = if kind == 1 && i >= 8 { " HULK" } else { "" };
            let health = if kind == 2 { HEALTH[i] } else { "100.0" };
            let module = MODULES[kind];
            let _ = write!(
                s,
                "Object P54A_{kind}_{i}\n  KindOf = PRELOAD VEHICLE{hulk}\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = {health}\n    InitialHealth = {health}\n  End\n  Behavior = {module} ModuleTag_Update\n"
            );
            if kind <= 1 {
                let (lo, hi) = RANGES[i % 8];
                if lo >= 0 {
                    let _ = writeln!(s, "    MinLifetime = {lo}");
                }
                if hi >= 0 {
                    let _ = writeln!(s, "    MaxLifetime = {hi}");
                }
            }
            if kind == 3 {
                let _ = writeln!(s, "    Enabled = {}", if i != 0 { "Yes" } else { "No" });
            }
            if kind == 4 && !RATIO[i].is_empty() {
                let _ = writeln!(s, "    DamageToFramesRatio = {}", RATIO[i]);
            }
            if kind == 5 && !SCALAR[i].is_empty() {
                let _ = writeln!(s, "    CourseCorrectionScalar = {}", SCALAR[i]);
            }
            s.push_str("  End\nEnd\n");
        }
    }
    s
}

fn build_rules() -> RulesRegistry {
    let mut rules = RulesRegistry::new();
    rules.global_data.gravity = -1.0;
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

fn reset_world(sim: &mut SimContext, frame: u32, seed: u32) {
    sim.world = SimWorld::new();
    let w = &mut sim.world;
    for (player, team) in [NEUTRAL, TEAM_A].into_iter().enumerate() {
        w.team_players.insert(team, player as Int);
        w.player_default_teams.insert(player as Int, team);
    }
    w.team_is_neutral.insert(NEUTRAL, TRUE);
    w.neutral_team_id = NEUTRAL;
    w.frame = frame;
    w.rng.init_random_with_seed(seed);
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

/// The harness's `step_single_update`.
fn step(sim: &mut SimContext, slot: ModuleSlot, owner: ObjectSlot, frame: u32) {
    sim.world.frame = frame;
    if sim.object(owner).is_none_or(|o| o.is_destroyed()) {
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
                return zh_common::types::UpdateSleepTime::NONE;
            }
            let sleep = update.update(s, c);
            if sleep.0 < 1 {
                zh_common::types::UpdateSleepTime::NONE
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

fn with_module<T: 'static>(
    sim: &mut SimContext,
    slot: ModuleSlot,
    f: impl FnOnce(&mut T, &mut dyn SimApi, ModuleContext),
) {
    sim.dispatch_module(slot, |m, s, c| {
        let typed = m
            .as_any_mut()
            .and_then(|a| a.downcast_mut::<T>())
            .expect("module type");
        f(typed, s, c);
    })
    .expect("dispatch module");
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
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

fn rng_crc(sim: &SimContext) -> u32 {
    sim.world.rng.get_game_logic_random_seed_crc()
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
    xfer: String,
    obs: [u32; 8],
}

fn row(sim: &mut SimContext, owner: ObjectSlot, module: ModuleSlot) -> Row {
    let o = sim.object(owner).expect("owner");
    let state = u32::from(o.is_destroyed()) | (u32::from(o.is_effectively_dead() != FALSE) << 1);
    let pos = *o.thing.get_position();
    let flags = u32::from(o.status.test(ObjectStatusType::NoAttack as Int) != FALSE)
        | (u32::from(
            o.model_condition
                .test(ModelConditionFlagType::Prone.as_i32())
                != FALSE,
        ) << 1);
    let obs = [
        state,
        pos.x.to_bits(),
        pos.y.to_bits(),
        pos.z.to_bits(),
        sim.object_health(owner).to_bits(),
        flags,
        sim.module_schedule(module)
            .expect("schedule")
            .next_call_frame(),
        rng_crc(sim),
    ];
    Row {
        xfer: hex(&save_module(sim, module)),
        obs,
    }
}

fn set_regen_globals(sim: &mut SimContext, percent: u32, delay: u32) {
    sim.rules.global_data.base_regen_health_percent_per_second = Real::from_bits(percent);
    sim.rules.global_data.base_regen_delay = delay;
}

struct Actual {
    rows: Vec<Row>,
    crc: u32,
    rng: u32,
}

fn run(sim: &mut SimContext, v: &Input) -> Actual {
    reset_world(sim, v.frame, v.seed);
    sim.terrain = Box::new(WaterTerrain {
        level: (5.0 as Real).to_bits(),
    });
    sim.world.hulk_max_lifetime_override = if v.kind == 1 { v.p[3] as Int - 1 } else { -1 };
    if v.kind == 2 {
        set_regen_globals(sim, v.p[3], v.p[4]);
    } else {
        set_regen_globals(sim, 0, 0);
    }

    let name = format!("P54A_{}_{}", v.kind, v.variant);
    let tid = sim
        .rules
        .thing_factory
        .find_template_id(&name)
        .unwrap_or_else(|| panic!("missing template {name}"));
    let owner = sim.spawn_object_with_status(
        tid,
        TEAM_A,
        Coord3D::default(),
        0.0,
        ObjectStatusMaskType::new(),
    );
    let start = Coord3D::new(
        Real::from_bits(v.p[0]),
        Real::from_bits(v.p[1]),
        Real::from_bits(v.p[2]),
    );
    sim.set_object_position(owner, &start);
    let m = module_named(sim, owner, MODULES[v.kind]);

    let mut rows = vec![row(sim, owner, m)];
    let mut frame = v.frame;
    for &[code, a, b, c, d] in &v.ops {
        sim.world.frame = frame;
        let point = Coord3D::new(Real::from_bits(b), Real::from_bits(c), Real::from_bits(d));
        match code {
            0 => {
                frame += a;
                step(sim, m, owner, frame);
            }
            1 => kill_object(sim, owner, DamageType::Unresistable, DeathType::Normal),
            2 => {
                let mut dmg = DamageInfo::default();
                dmg.input.damage_type = match a {
                    0 => DamageType::Explosion,
                    1 => DamageType::Healing,
                    _ => DamageType::Unresistable,
                };
                dmg.input.death_type = DeathType::Normal;
                dmg.input.amount = Real::from_bits(b);
                dmg.input.source_id = Default::default();
                sim.attempt_damage(owner, &mut dmg);
            }
            3 => {
                let mut dmg = DamageInfo::default();
                dmg.input.damage_type = DamageType::Healing;
                dmg.input.death_type = DeathType::NoneType;
                dmg.input.source_id = sim.object(owner).expect("owner").id;
                dmg.input.amount = Real::from_bits(b);
                sim.attempt_healing(owner, &mut dmg);
            }
            4 => {
                let status = match a {
                    0 => ObjectStatusType::UnderConstruction,
                    1 => ObjectStatusType::Sold,
                    _ => ObjectStatusType::NoAttack,
                };
                if b != 0 {
                    sim.set_status(owner, status);
                } else {
                    sim.clear_status(owner, status);
                }
            }
            5 => {
                if a != 0 {
                    sim.set_disabled(owner, DisabledType::Held);
                } else {
                    sim.clear_disabled(owner, DisabledType::Held);
                }
            }
            6 if v.kind == 0 => with_module::<DeletionUpdate>(sim, m, |u, s, ctx| {
                u.set_lifetime_range(s, ctx, a, b);
            }),
            6 => with_module::<LifetimeUpdate>(sim, m, |u, s, ctx| {
                u.set_lifetime_range(s, ctx, a, b);
            }),
            7 => with_module::<FloatUpdate>(sim, m, |u, _, _| {
                u.set_enabled(if a != 0 { TRUE } else { FALSE });
            }),
            8 => sim.set_object_position(owner, &point),
            9 => sim.terrain = Box::new(WaterTerrain { level: b }),
            10 => with_module::<ProneUpdate>(sim, m, |u, s, ctx| {
                let mut dmg = DamageInfo::default();
                dmg.output.actual_damage_dealt = Real::from_bits(b);
                u.go_prone(s, ctx, &dmg);
            }),
            11 => with_module::<SmartBombTargetHomingUpdate>(sim, m, |u, _, _| {
                u.set_target_position(&point);
            }),
            12 => set_regen_globals(sim, a, b),
            other => panic!("op {other}"),
        }
        rows.push(row(sim, owner, m));
    }
    Actual {
        rows,
        crc: module_crc(sim, m),
        rng: rng_crc(sim),
    }
}

/// First difference between one vector's expected and actual outputs.
fn difference(expected: &Json, actual: &Actual) -> Option<String> {
    let rows = expected.get("rows").arr();
    if rows.len() != actual.rows.len() {
        return Some(format!("row count {} != {}", actual.rows.len(), rows.len()));
    }
    for (i, (want, got)) in rows.iter().zip(&actual.rows).enumerate() {
        let want = want.arr();
        for (k, name) in FIELDS.iter().enumerate() {
            if got.obs[k] != want[k + 1].num() {
                return Some(format!(
                    "row {i} {name}: got {} want {}",
                    got.obs[k],
                    want[k + 1].num()
                ));
            }
        }
        if got.xfer != want[0].text() {
            return Some(format!("row {i} xfer {} want {}", got.xfer, want[0].text()));
        }
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
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/golden_vectors/WP54_A.json");
    let raw = fs::read(path).expect("read WP54_A.json");
    Parser { b: &raw, i: 0 }.value()
}

#[test]
fn wp54_a_matches_reference() {
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

        // The first vector of each module carries the diagnostics.
        if index < 6 {
            let name = KINDS[v.kind];
            let mut got: Vec<(String, String)> = vec![
                (format!("{name}_rows"), actual.rows.len().to_string()),
                (format!("{name}_xfer"), actual.rows[0].xfer.clone()),
            ];
            let last = actual.rows.last().expect("row");
            for (k, field) in FIELDS.iter().enumerate() {
                got.push((format!("{name}_{field}"), last.obs[k].to_string()));
            }
            for (key, value) in got {
                if diag[&key].text() != value {
                    first.push(format!(
                        "seed0 {key}: got {value} want {}",
                        diag[&key].text()
                    ));
                }
                checked_diag += 1;
            }
        }

        if let Some(why) = difference(vector.get("out"), &actual) {
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
        failures.iter().all(|&n| n == 0) && first.is_empty(),
        "mismatches per module {failures:?}\n{}",
        first.join("\n")
    );
}
