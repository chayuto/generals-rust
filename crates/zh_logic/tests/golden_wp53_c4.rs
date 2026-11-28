//! Gate V2 differential verification for `PKT-281 NeutronBlastBehavior` and the
//! `Object::setTeam` side effects it depends on, against `docs/golden_vectors/WP53_C4.json`
//! (`tests/cpp_harness/gen_wp53_c4.cpp`).
//!
//! `AIUpdateInterface` is not ported; a stand-in constructs the AI. The test marks targets
//! airborne and defeats players, both of which unported code does.

use std::any::Any;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use zh_common::ini::{IniError, IniLoadType, IniReader};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::{
    Int, NameKeyType, Real, Relationship, TeamId, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferCrc, XferError, XferSave};
use zh_logic::arena::{FacetMask, ModuleContext, ModuleSlot, ObjectSlot};
use zh_logic::modules::contain::open_contain::kill_object;
use zh_logic::modules::module::{
    AiUpdateFacet, BehaviorModule, ModuleData, ModuleType,
};
use zh_logic::modules::module_factory::ModuleParseContext;
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::object_status_types::{ObjectStatusMaskType, ObjectStatusType};
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

const KINDS: [&str; 1] = ["neutron_blast"];

fn fields(_kind: usize) -> &'static [&'static str] {
    &["t0", "t1", "t2", "t3", "t4", "t5", "rider_dead_mask", "wake", "rng"]
}

// -----------------------------------------------------------------------------
// Stand-ins for unported modules
// -----------------------------------------------------------------------------

const AI_MASK: FacetMask = FacetMask(0x001 | FacetMask::AI_UPDATE.0);

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

fn create_ai(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    Box::new(AiStandIn {
        key: data.module_name_key(),
    })
}

// -----------------------------------------------------------------------------
// Fixture (the harness's `fixture_ini`)
// -----------------------------------------------------------------------------

fn object(s: &mut String, name: &str, kind_of: &str, health: &str, rest: &str) {
    let _ = write!(
        s,
        "Object {name}\n  KindOf = PRELOAD {kind_of}\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = {health}\n    InitialHealth = {health}\n  End\n{rest}End\n"
    );
}

fn objects_ini() -> String {
    let mut s = String::new();
    let ai = "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n";
    object(&mut s, "P53C4_Rider1", "INFANTRY\n  TransportSlotCount = 1", "100.0", "");
    object(&mut s, "P53C4_PlainBldg", "STRUCTURE", "1000.0", "");
    object(
        &mut s,
        "P53C4_Garr",
        "STRUCTURE",
        "1000.0",
        "  Behavior = GarrisonContain ModuleTag_Contain\n    ContainMax = 10\n  End\n",
    );
    object(
        &mut s,
        "P53C4_Trans",
        "VEHICLE",
        "500.0",
        "  Behavior = TransportContain ModuleTag_Contain\n    Slots = 8\n    AllowInsideKindOf = INFANTRY\n  End\n",
    );
    for i in 0..12 {
        let radius = ["10.0", "25.0", "60.0"][i % 3];
        let airborne = if (i / 3) % 2 == 1 { "Yes" } else { "No" };
        let allies = if i / 6 == 1 { "Yes" } else { "No" };
        object(
            &mut s,
            &format!("P53C4_Neutron{i}"),
            "PROJECTILE",
            "10.0",
            &format!("  Behavior = NeutronBlastBehavior ModuleTag_Neutron\n    BlastRadius = {radius}\n    AffectAirborne = {airborne}\n    AffectAllies = {allies}\n  End\n"),
        );
    }
    object(&mut s, "P53C4_Veh", "VEHICLE", "300.0", "");
    object(&mut s, "P53C4_Drone", "VEHICLE DRONE", "80.0", "");
    object(&mut s, "P53C4_Bike", "VEHICLE CLIFF_JUMPER", "120.0", "");
    object(&mut s, "P53C4_Air", "VEHICLE AIRCRAFT", "150.0", "");
    object(&mut s, "P53C4_AiVeh", "VEHICLE", "300.0", ai);
    s
}

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

fn reset_world(sim: &mut SimContext, frame: u32, seed: u32, local_player: Int) {
    sim.world = SimWorld::new();
    // The harness world: 20-unit partition cells over its 1000 by 1000 terrain.
    sim.init_partition(
        20.0,
        Region3D {
            lo: Coord3D::new(0.0, 0.0, 0.0),
            hi: Coord3D::new(1000.0, 1000.0, 100.0),
        },
    );
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
            w.set_player_relation(i, j, rel);
        }
    }
    w.frame = frame;
    w.rng.init_random_with_seed(seed);
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

fn rng_crc(sim: &SimContext) -> u32 {
    sim.world.rng.get_game_logic_random_seed_crc()
}

fn kill(sim: &mut SimContext, slot: ObjectSlot) {
    kill_object(sim, slot, DamageType::Unresistable, DeathType::Normal);
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

const NEUTRON_TARGETS: [&str; 9] = [
    "P53C4_Rider1",
    "P53C4_Veh",
    "P53C4_Drone",
    "P53C4_Bike",
    "P53C4_Air",
    "P53C4_Garr",
    "P53C4_Trans",
    "P53C4_PlainBldg",
    "P53C4_AiVeh",
];

fn run_neutron(sim: &mut SimContext, v: &Input) -> (Vec<Row>, u32) {
    let bomb = spawn_at(
        sim,
        &format!("P53C4_Neutron{}", v.variant),
        TEAMS[v.p[0] as usize],
        Coord3D::new(200.0, 200.0, 0.0),
    );
    let mut targets = Vec::new();
    let mut riders = Vec::new();
    for k in 0..v.p[1] as usize {
        let t = &v.p[2 + k * 7..9 + k * 7];
        let team = TEAMS[t[1] as usize];
        let pos = Coord3D::new(
            Real::from_bits(t[2]),
            Real::from_bits(t[3]),
            Real::from_bits(t[4]),
        );
        let o = spawn_at(sim, NEUTRON_TARGETS[t[0] as usize], team, pos);
        for _ in 0..t[6] {
            let r = spawn_at(sim, "P53C4_Rider1", team, pos);
            sim.contain_add(o, r);
            riders.push(r);
        }
        if t[5] & 1 != 0 {
            // Set by the unported locomotor.
            sim.set_status(o, ObjectStatusType::AirborneTarget);
        }
        if t[5] & 2 != 0 {
            kill(sim, o);
        }
        targets.push(o);
    }
    let m = module_named(sim, bomb, "NeutronBlastBehavior");
    // The harness updates the partition once, after setting the scene.
    sim.update_partition();
    let allies = *v.p.last().expect("relationship mask");
    for (bit, pair) in [(0, 1), (1, 0), (0, 2), (2, 0), (1, 2), (2, 1)].into_iter().enumerate() {
        if allies & (1 << bit) != 0 {
            sim.world.set_player_relation(pair.0, pair.1, Relationship::Allies);
        }
    }

    sim.world.rng.init_random_with_seed(v.seed);
    let mut frame = v.frame;
    let mut rows = Vec::new();
    for &[code, a, b, ..] in &v.ops {
        match code {
            0 => kill(sim, bomb),
            1 => {
                frame += 1;
                step(sim, m, bomb, frame);
            }
            2 => sim.set_object_team(targets[a as usize], TEAMS[b as usize]),
            3 => {
                sim.world.player_inactive.insert(TEAMS[a as usize]);
            }
            other => panic!("op {other}"),
        }
        let mut obs = vec![0_u32; 6];
        for (k, &s) in targets.iter().enumerate() {
            let o = sim.object(s).expect("target");
            obs[k] = 1
                | (u32::from(o.is_effectively_dead() != FALSE) << 1)
                | (u32::from(o.is_disabled_by_type(DisabledType::Unmanned) != FALSE) << 2)
                | (team_index(o.team_id) << 4);
        }
        let mut rider_dead = 0;
        for (k, &r) in riders.iter().enumerate() {
            if is_dead(sim, r) {
                rider_dead |= 1 << k;
            }
        }
        obs.push(rider_dead);
        let next = sim.module_schedule(m).expect("schedule").next_call_frame();
        obs.push(u32::from(next >= UPDATE_SLEEP_FOREVER));
        obs.push(rng_crc(sim));
        rows.push(row(sim, m, obs));
    }
    let crc = module_crc(sim, m);
    (rows, crc)
}

fn run(sim: &mut SimContext, v: &Input, local_player: Int) -> Actual {
    reset_world(sim, v.frame, v.seed, local_player);
    let (rows, crc) = run_neutron(sim, v);
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
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/golden_vectors/WP53_C4.json");
    let raw = fs::read(path).expect("read WP53_C4.json");
    Parser { b: &raw, i: 0 }.value()
}

#[test]
fn wp53_c4_matches_reference() {
    let doc = load_golden();
    let vectors = doc.get("vectors").arr();
    assert_eq!(doc.u32("vector_count"), 1000);
    assert_eq!(vectors.len(), 1000);
    let diag = doc.get("seed0_diagnostics").obj();

    let mut sim = SimContext::new();
    sim.rules = build_rules();
    let local_player = doc.u32("local_player") as Int;
    let mut checked_diag = 0;
    let mut failures = [0_usize; 1];
    let mut first = Vec::new();
    for (index, vector) in vectors.iter().enumerate() {
        let v = input(vector.get("in"));
        let actual = run(&mut sim, &v, local_player);

        // Vector 0 carries the diagnostics.
        if index < 1 {
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

