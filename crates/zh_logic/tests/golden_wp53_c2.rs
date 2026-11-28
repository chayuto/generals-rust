//! Gate V2 differential verification for Wave `WP-5.3-C2`: `PKT-275 HelixContain` against
//! `docs/golden_vectors/WP53_C2.json` (`tests/cpp_harness/gen_wp53_c2.cpp`).
//!
//! `PKT-264 CaveContain`, `PKT-284 OverlordContain` and `PKT-300 TunnelContain` are not part of
//! this oracle; see `docs/OPEN_DECISIONS.md`.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{Bool, Int, Relationship, TeamId, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{XferCrc, XferLoad, XferSave};
use zh_logic::arena::{ModuleSlot, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::gamelogic::global_data::WeaponBonusConditionType;
use zh_logic::modules::contain::helix_contain::HelixContain;
use zh_logic::modules::contain::open_contain::kill_object;
use zh_logic::modules::module::ExitDoorType;
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::model_state::ModelConditionFlagType;
use zh_logic::rules::object_status_types::{ObjectStatusMaskType, ObjectStatusType};
use zh_logic::scheduler::UPDATE_SLEEP_FOREVER;
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimView, SimWorld};

const VARIANTS: usize = 8;
const FIELDS: [&str; 16] = [
    "count",
    "alive",
    "inside",
    "fire",
    "nested_fire",
    "enclosing",
    "valid",
    "held",
    "bonus",
    "masked",
    "unselectable",
    "health",
    "damage_state",
    "loaded",
    "destroyed",
    "door",
];
const RIDER_NAMES: [&str; 4] = ["P53C2_Inf", "P53C2_Veh", "P53C2_Gun", "P53C2_Bunker"];
const MAX_RIDERS: usize = 24;

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
                while !matches!(self.b[self.i], b',' | b']' | b'}')
                    && !self.b[self.i].is_ascii_whitespace()
                {
                    self.i += 1;
                }
                Json::Scalar(String::from_utf8_lossy(&self.b[start..self.i]).into_owned())
            }
        }
    }
}

fn load_golden() -> Json {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/golden_vectors/WP53_C2.json");
    let raw = fs::read(path).expect("read WP53_C2.json");
    Parser { b: &raw, i: 0 }.value()
}

// -----------------------------------------------------------------------------
// Templates mirroring the harness
// -----------------------------------------------------------------------------

fn body(health: &str) -> String {
    format!(
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = {health}\n    InitialHealth = {health}\n  End\n"
    )
}

fn objects_ini() -> String {
    let mut s = String::new();
    for (name, kind, slots, health) in [
        ("P53C2_Inf", "INFANTRY", 1, "60.0"),
        ("P53C2_Veh", "VEHICLE", 3, "200.0"),
        ("P53C2_Gun", "PORTABLE_STRUCTURE", 1, "100.0"),
    ] {
        let _ = write!(
            s,
            "Object {name}\n  KindOf = PRELOAD {kind}\n  TransportSlotCount = {slots}\n{}End\n",
            body(health)
        );
    }
    let _ = write!(
        s,
        "Object P53C2_Bunker\n  KindOf = PRELOAD PORTABLE_STRUCTURE\n  TransportSlotCount = 1\n{}  Behavior = TransportContain ModuleTag_Contain\n    Slots = 3\n    AllowInsideKindOf = INFANTRY\n    PassengersAllowedToFire = Yes\n  End\nEnd\n",
        body("300.0")
    );

    let fields: [&str; VARIANTS] = [
        "    Slots = 5\n    AllowInsideKindOf = INFANTRY\n    PassengersAllowedToFire = Yes\n",
        "    Slots = 2\n    AllowInsideKindOf = INFANTRY\n    PassengersAllowedToFire = No\n    PayloadTemplateName = P53C2_Gun\n",
        "    Slots = 5\n    AllowInsideKindOf = INFANTRY\n    PassengersAllowedToFire = Yes\n    PayloadTemplateName = P53C2_Bunker\n    ShouldDrawPips = No\n",
        "    Slots = 4\n    AllowInsideKindOf = INFANTRY PORTABLE_STRUCTURE\n    PassengersAllowedToFire = Yes\n    PayloadTemplateName = P53C2_Inf P53C2_Gun P53C2_Veh P53C2_Missing\n    HealthRegen%PerSec = 10\n",
        "    Slots = 6\n    AllowInsideKindOf = INFANTRY PORTABLE_STRUCTURE\n    PassengersAllowedToFire = No\n    PayloadTemplateName = P53C2_Gun\n    PayloadTemplateName = P53C2_Bunker P53C2_Inf\n",
        "    Slots = 3\n    AllowInsideKindOf = INFANTRY VEHICLE\n    PassengersAllowedToFire = Yes\n    InitialPayload = P53C2_Inf 2\n    DestroyRidersWhoAreNotFreeToExit = Yes\n    ExitDelay = 500\n",
        "    Slots = 1\n    AllowInsideKindOf = INFANTRY\n    PassengersAllowedToFire = Yes\n    PayloadTemplateName = P53C2_Inf P53C2_Inf P53C2_Bunker\n",
        "    Slots = 0\n    PayloadTemplateName = P53C2_Veh P53C2_Bunker\n    HealthRegen%PerSec = 25\n",
    ];
    for (i, extra) in fields.iter().enumerate() {
        let _ = write!(
            s,
            "Object P53C2_Helix{i}\n  KindOf = PRELOAD VEHICLE\n{}  Behavior = HelixContain ModuleTag_Contain\n{extra}  End\nEnd\n",
            body("1000.0")
        );
    }
    s
}

fn build_rules() -> RulesRegistry {
    let mut rules = RulesRegistry::new();
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

fn reset_world(sim: &mut SimContext, frame: u32, seed: u32) {
    sim.world = SimWorld::new();
    let w = &mut sim.world;
    for (player, team) in [NEUTRAL, TEAM_A, TEAM_B].into_iter().enumerate() {
        w.team_players.insert(team, player as Int);
        w.player_default_teams.insert(player as Int, team);
    }
    w.team_is_neutral.insert(NEUTRAL, TRUE);
    w.neutral_team_id = NEUTRAL;
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

fn contain_slot(sim: &SimContext, owner: ObjectSlot) -> Option<ModuleSlot> {
    sim.object(owner).and_then(|o| o.contain_slot)
}

fn is_destroyed(sim: &SimContext, slot: ObjectSlot) -> bool {
    sim.object(slot).is_none_or(|o| o.is_destroyed())
}

fn is_live(sim: &SimContext, slot: ObjectSlot) -> bool {
    sim.object(slot)
        .is_some_and(|o| !o.is_destroyed() && o.is_effectively_dead() == FALSE)
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
    variant: u32,
    frame: u32,
    seed: u32,
    ops: Vec<[u32; 4]>,
}

fn input(j: &Json) -> Input {
    Input {
        variant: j.u32("variant"),
        frame: j.u32("frame"),
        seed: j.u32("seed"),
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
    riders: String,
    obs: [u32; 16],
}

struct Actual {
    rows: Vec<Row>,
    crc: u32,
    rng: u32,
}

fn team_index(team: TeamId) -> u32 {
    match team {
        TEAM_A => 0,
        TEAM_B => 1,
        NEUTRAL => 2,
        _ => 3,
    }
}

fn observe(
    sim: &mut SimContext,
    o: ObjectSlot,
    m: ModuleSlot,
    riders: &[ObjectSlot],
    door: u32,
) -> Row {
    let xfer = save_module(sim, m);
    let mut masks = [0_u32; 10];
    let [alive, inside, fire, nested, encl, valid, held, bonus, masked, unsel] = &mut masks;
    let mut bytes = Vec::new();
    for (k, &r) in riders.iter().enumerate() {
        let bit = 1_u32 << k;
        let live = is_live(sim, r);
        let rec = sim.object(r).expect("rider");
        let (id, contained_by) = (rec.id, rec.contained_by);
        let pos = *rec.thing.get_position();
        let angle = rec.thing.get_orientation();
        let team = rec.team_id;
        if live {
            *alive |= bit;
        }
        if contained_by == Some(o) {
            *inside |= bit;
        }
        if rec.is_disabled_by_type(DisabledType::Held) != FALSE {
            *held |= bit;
        }
        if rec.weapon_bonus_condition & (1 << WeaponBonusConditionType::Garrisoned as u32) != 0 {
            *bonus |= bit;
        }
        if rec.status.test(ObjectStatusType::Masked.as_i32()) != FALSE {
            *masked |= bit;
        }
        if rec.status.test(ObjectStatusType::Unselectable.as_i32()) != FALSE {
            *unsel |= bit;
        }
        if live {
            if sim.contain_is_passenger_allowed_to_fire(o, id) != FALSE {
                *fire |= bit;
            }
            let enclosing = sim
                .world
                .modules
                .payload(m)
                .and_then(|p| p.as_contain())
                .expect("contain")
                .is_enclosing_container_for(r);
            if enclosing != FALSE {
                *encl |= bit;
            }
            if contained_by.is_none() && sim.is_valid_container_for(o, r, TRUE) != FALSE {
                *valid |= bit;
            }
            for x in sim.contain_items(r) {
                if sim.contain_is_passenger_allowed_to_fire(r, x) != FALSE {
                    *nested |= bit;
                }
            }
        }
        for word in [
            pos.x.to_bits(),
            pos.y.to_bits(),
            pos.z.to_bits(),
            angle.to_bits(),
            sim.object_health(r).to_bits(),
            sim.object_body_damage_state(r) as u32,
            team_index(team),
        ] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
    }
    let owner = sim.object(o).expect("helix");
    let loaded = owner
        .model_condition
        .test(ModelConditionFlagType::Loaded as Int)
        != FALSE;
    let destroyed = owner.is_destroyed();
    let mut obs = [0_u32; 16];
    obs[0] = sim.contain_count(o);
    obs[1..11].copy_from_slice(&masks);
    obs[11] = sim.object_health(o).to_bits();
    obs[12] = sim.object_body_damage_state(o) as u32;
    obs[13] = u32::from(loaded);
    obs[14] = u32::from(destroyed);
    obs[15] = door;
    Row {
        xfer,
        riders: digest(&bytes),
        obs,
    }
}

fn run(sim: &mut SimContext, v: &Input) -> Actual {
    reset_world(sim, v.frame, v.seed);
    let o = spawn(sim, &format!("P53C2_Helix{}", v.variant), TEAM_A);
    let m = contain_slot(sim, o).expect("contain slot");

    let mut riders: Vec<ObjectSlot> = sim
        .world_object_slots()
        .into_iter()
        .filter(|&s| s != o && !is_destroyed(sim, s))
        .collect();
    riders.sort_by_key(|&s| sim.object(s).expect("rider").id.0);

    let mut frame = v.frame;
    let mut rows = vec![observe(sim, o, m, &riders, 99)];
    for &[code, a, b, c] in &v.ops {
        let mut door = 99;
        let r = (!riders.is_empty()).then(|| riders[a as usize % riders.len()]);
        let live = r.filter(|&r| is_live(sim, r));
        let held = live.filter(|&r| sim.object(r).expect("rider").contained_by == Some(o));
        match code {
            0 => {
                if riders.len() < MAX_RIDERS {
                    let team = sim.object(o).expect("helix").team_id;
                    let x = spawn(sim, RIDER_NAMES[a as usize], team);
                    riders.push(x);
                    if sim.is_valid_container_for(o, x, TRUE) != FALSE {
                        sim.contain_add(o, x);
                    }
                }
            }
            1 => {
                if let Some(r) = held {
                    sim.contain_remove(o, r, FALSE);
                }
            }
            2 => {
                sim.set_object_position(o, &Coord3D::new(a as f32, b as f32, 0.0));
                sim.set_object_orientation(o, c as f32 / 100.0);
            }
            3 => {
                for _ in 0..a {
                    frame += 1;
                    step(sim, m, o, frame);
                }
            }
            4 => {
                sim.world.frame = frame;
                let mut d = DamageInfo::default();
                d.input.damage_type = DamageType::Unresistable;
                d.input.death_type = DeathType::Normal;
                d.input.amount = a as f32;
                sim.attempt_damage_on(o, &mut d);
            }
            5 => {
                sim.world.frame = frame;
                let mut d = DamageInfo::default();
                d.input.damage_type = DamageType::Healing;
                d.input.death_type = DeathType::NoneType;
                d.input.amount = a as f32;
                sim.attempt_healing(o, &mut d);
            }
            6 => {
                sim.world.frame = frame;
                sim.capture_object(o, [TEAM_A, TEAM_B, NEUTRAL][a as usize]);
            }
            7 => {
                sim.world.frame = frame;
                kill_object(sim, o, DamageType::Unresistable, DeathType::Normal);
            }
            8 => {
                if let Some(r) = live {
                    kill_object(sim, r, DamageType::Unresistable, DeathType::Normal);
                }
            }
            9 => {
                if let Some(r) = held {
                    sim.dispatch_module(m, |b, s, c| {
                        b.as_exit_mut().expect("exit").exit_object_via_door(
                            s,
                            c,
                            r,
                            ExitDoorType::Door1,
                        );
                    })
                    .expect("exit");
                }
            }
            10 => sim.contain_remove_all(o, FALSE),
            11 => {
                if let Some(r) = live {
                    if contain_slot(sim, r).is_some() && riders.len() < MAX_RIDERS {
                        let team = sim.object(r).expect("rider").team_id;
                        let x = spawn(sim, "P53C2_Inf", team);
                        riders.push(x);
                        if sim.is_valid_container_for(r, x, TRUE) != FALSE {
                            sim.contain_add(r, x);
                        }
                    }
                }
            }
            12 => {
                if let Some(r) = live {
                    let mut d = DamageInfo::default();
                    d.input.damage_type = DamageType::Unresistable;
                    d.input.death_type = DeathType::Normal;
                    d.input.amount = b as f32;
                    sim.attempt_damage_on(r, &mut d);
                }
            }
            13 => {
                if let Some(r) =
                    r.filter(|&r| !is_destroyed(sim, r) && contain_slot(sim, r).is_none())
                {
                    sim.destroy_object(r);
                }
            }
            14 => {
                if let Some(r) = live {
                    let tmpl = sim
                        .rules
                        .thing_factory
                        .find_template_id("P53C2_Inf")
                        .expect("rider template");
                    let reserved = sim
                        .dispatch_module(m, |b, s, c| {
                            b.as_exit_mut().expect("exit").reserve_door_for_exit(
                                s,
                                c,
                                tmpl,
                                Some(r),
                            )
                        })
                        .expect("reserve");
                    door = (reserved as Int) as u32;
                }
            }
            _ => panic!("unknown op {code}"),
        }
        rows.push(observe(sim, o, m, &riders, door));
    }
    let crc = module_crc(sim, m);
    Actual {
        rows,
        crc,
        rng: sim.world.rng.get_game_logic_random_seed_crc(),
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
        assert_eq!(want.len(), 2 + FIELDS.len());
        for (k, name) in FIELDS.iter().enumerate() {
            if got.obs[k] != want[k + 2].num() {
                return Some(format!(
                    "row {i} {name}: got {} want {}",
                    got.obs[k],
                    want[k + 2].num()
                ));
            }
        }
        if got.riders != want[1].text() {
            return Some(format!("row {i} riders digest"));
        }
        if digest(&got.xfer) != want[0].text() {
            return Some(format!("row {i} xfer digest ({} bytes)", got.xfer.len()));
        }
    }
    let final_xfer = expected.get("final_xfer").text();
    if final_xfer != "-" && hex(&actual.rows.last().expect("row").xfer) != final_xfer {
        return Some("final xfer bytes".to_owned());
    }
    if actual.crc != expected.u32("crc") {
        return Some(format!("crc {} != {}", actual.crc, expected.u32("crc")));
    }
    if actual.rng != expected.u32("rng") {
        return Some(format!("rng {} != {}", actual.rng, expected.u32("rng")));
    }
    None
}

#[test]
fn wp53_c2_matches_reference() {
    let doc = load_golden();
    let vectors = doc.get("vectors").arr();
    assert_eq!(doc.u32("vector_count"), 1000);
    assert_eq!(vectors.len(), 1000);
    let diag = doc.get("seed0_diagnostics").obj();

    let mut sim = SimContext::new();
    sim.rules = build_rules();
    let mut checked_diag = 0;
    let mut failures = 0;
    let mut first = Vec::new();
    for (index, vector) in vectors.iter().enumerate() {
        let actual = run(&mut sim, &input(vector.get("in")));

        // Vector `k` is the first one of variant `k`.
        if index < VARIANTS {
            let name = format!("helix{index}");
            let created = actual.rows.first().expect("row");
            let last = actual.rows.last().expect("row");
            let mut got: Vec<(String, String)> = vec![
                (format!("{name}_rows"), actual.rows.len().to_string()),
                (format!("{name}_xfer"), hex(&created.xfer)),
                (format!("{name}_riders"), last.riders.clone()),
            ];
            for (k, field) in FIELDS.iter().enumerate() {
                got.push((
                    format!("{name}_created_{field}"),
                    created.obs[k].to_string(),
                ));
                got.push((format!("{name}_last_{field}"), last.obs[k].to_string()));
            }
            for (key, value) in got {
                assert_eq!(diag[&key].text(), value, "seed0 diagnostic {key}");
                checked_diag += 1;
            }
        }

        if let Some(why) = difference(vector.get("out"), &actual) {
            failures += 1;
            if failures <= 6 {
                first.push(format!(
                    "vector {index} (variant {}): {why}",
                    vector.get("in").u32("variant")
                ));
            }
        }
    }
    assert_eq!(
        checked_diag,
        diag.len(),
        "every seed0 diagnostic is compared"
    );
    assert!(failures == 0, "{failures} mismatches\n{}", first.join("\n"));
}

/// Loading a module's saved bytes into a fresh instance and saving again reproduces them, and
/// the loaded instance still treats the portable structure as not enclosed.
#[test]
fn wp53_c2_xfer_load_round_trips() {
    let mut sim = SimContext::new();
    sim.rules = build_rules();
    reset_world(&mut sim, 100, 1);
    let source = spawn(&mut sim, "P53C2_Helix4", TEAM_A);
    let m = contain_slot(&sim, source).expect("contain slot");
    let portable_slot = sim
        .world_object_slots()
        .into_iter()
        .filter(|&s| s != source)
        .min_by_key(|&s| sim.object(s).expect("payload").id.0)
        .expect("portable payload");
    let portable_id = sim.object(portable_slot).expect("portable").id;
    let bytes = save_module(&mut sim, m);

    let target = spawn(&mut sim, "P53C2_Helix0", TEAM_A);
    let t = contain_slot(&sim, target).expect("contain slot");
    sim.dispatch_module(t, |b, s, c| {
        let mut load = XferLoad::new(&bytes);
        b.xfer(&mut load, s).expect("xfer load");
        b.load_post_process(s, c);
    })
    .expect("load");
    assert_eq!(save_module(&mut sim, t), bytes);

    let helix = sim
        .world
        .modules
        .payload(t)
        .and_then(|p| p.as_any())
        .and_then(|a| a.downcast_ref::<HelixContain>())
        .expect("helix");
    assert_eq!(helix.0.portable_structure_id, portable_id);
    let enclosing: Bool = sim
        .world
        .modules
        .payload(t)
        .and_then(|p| p.as_contain())
        .expect("contain")
        .is_enclosing_container_for(portable_slot);
    assert_eq!(enclosing, FALSE);
}

/// `HelixContain::onDelete` has no vector: the spine runs it at the end of the frame
/// (`docs/OPEN_DECISIONS.md`). After the destroy list is processed the portable structure and
/// every listed rider are gone, as they are in the reference.
#[test]
fn wp53_c2_delete_takes_the_portable_and_riders_along() {
    let mut sim = SimContext::new();
    sim.rules = build_rules();
    reset_world(&mut sim, 100, 1);
    let helix = spawn(&mut sim, "P53C2_Helix4", TEAM_A);
    let payload: Vec<ObjectSlot> = sim
        .world_object_slots()
        .into_iter()
        .filter(|&s| s != helix)
        .collect();
    assert_eq!(payload.len(), 3);
    sim.destroy_object(helix);
    sim.process_destroy_list();
    for slot in payload {
        assert!(is_destroyed(&sim, slot));
    }
}
