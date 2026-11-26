//! Gate V2 differential verification for `PKT-388 WeaponBonusUpdate` against
//! `docs/golden_vectors/PKT-388.json` (`tests/cpp_harness/gen_wp55_weapon_bonus.cpp`).
//!
//! `FiringTracker` is not attached as a module here. A sleeping stand-in takes its place beside
//! the bonus helper, and the sleepy update heap starts in the recorded layout, so the layout
//! after each operation shows the order in which the helpers were woken.

use std::any::Any;
use zh_common::golden_json::{decode_hex_bytes, extract_vector_lines, get_str, get_u32};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::{Int, NameKeyType, ObjectId, Real, Relationship, TeamId, FALSE};
use zh_common::xfer::{Xfer, XferCrc, XferError, XferSave};
use zh_logic::arena::{FacetMask, ModuleDataId, ModuleSlot, ObjectSlot};
use zh_logic::modules::contain::open_contain::kill_object;
use zh_logic::modules::module::BehaviorModule;
use zh_logic::modules::update::weapon_bonus_update::WeaponBonusUpdate;
use zh_logic::object::weapon::WeaponTemplate;
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::object_status_types::ObjectStatusMaskType;
use zh_logic::scheduler::{SleepyUpdatePhase, UPDATE_SLEEP_FOREVER};
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimView, SimWorld};

const JSON: &str = include_str!("../../../docs/golden_vectors/PKT-388.json");

const NOT_CALLED: u32 = 0xffff_ffff;
const DISABLED: u32 = 0xffff_fffe;

// Harness team index order: the two sides, then neutral (player 0).
const TEAMS: [TeamId; 3] = [TeamId(2), TeamId(3), TeamId(1)];
const UNITS: [&str; 9] = [
    "WB_Inf",
    "WB_Veh",
    "WB_Unarmed",
    "WB_Hero",
    "WB_Struct",
    "WB_Trans",
    "WB_BareTrans",
    "WB_Garr",
    "WB_OpenGarr",
];
const TAGS: [&str; 10] = [
    "ModuleTag_SMCHelper",
    "ModuleTag_StatusDamageHelper",
    "ModuleTag_SubdualDamageHelper",
    "ModuleTag_RepulsorHelper",
    "ModuleTag_DefectionHelper",
    "ModuleTag_WeaponStatusHelper",
    "ModuleTag_FiringTrackerHelper",
    "ModuleTag_TempWeaponBonusHelper",
    "ModuleTag_WBU",
    "ModuleTag_Contain",
];
const FIRING_TRACKER: usize = 6;
const COLUMNS: [&str; 7] = [
    "sleep", "next0", "next1", "modules", "objects", "heap", "bonused",
];

struct SleepingStandIn(NameKeyType);

impl BehaviorModule for SleepingStandIn {
    fn module_name_key(&self) -> NameKeyType {
        self.0
    }
    fn xfer(&mut self, _: &mut dyn Xfer, _: &dyn SimView) -> Result<(), XferError> {
        Ok(())
    }
    fn crc(&self, _: &mut dyn Xfer) {}
    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

struct Fnv(u32);

impl Fnv {
    fn new() -> Self {
        Self(0x811c_9dc5)
    }
    fn bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0 ^ u32::from(b)).wrapping_mul(0x0100_0193);
        }
    }
    fn word(&mut self, w: u32) {
        self.bytes(&w.to_le_bytes());
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn numbers(line: &str, key: &str) -> Vec<u32> {
    get_str(line, key)
        .unwrap_or_else(|| panic!("missing {key}"))
        .split_whitespace()
        .map(|t| t.parse().expect("number"))
        .collect()
}

fn build_rules() -> RulesRegistry {
    let mut rules = RulesRegistry::new();
    let mut weapon = WeaponTemplate::new();
    weapon
        .parse_ini_block(
            "Weapon Wpn_WB\n  PrimaryDamage = 10.0\n  AttackRange = 100.0\n  DamageType = SMALL_ARMS\nEnd",
        )
        .expect("parse weapon");
    rules.weapon_store.add_template(weapon);
    let line = JSON
        .lines()
        .find(|l| l.starts_with("\"ini_hex\""))
        .expect("ini");
    let text = decode_hex_bytes(get_str(line, "ini_hex").expect("ini")).expect("hex");
    let mut ini = IniReader::new("object.ini", &text, IniLoadType::Overwrite);
    if let Err(e) = rules.load_thing_templates_from_ini(&mut ini) {
        panic!("load thing templates at line {}: {e:?}", ini.get_line_num());
    }
    rules
}

fn reset_world(sim: &mut SimContext, frame: u32) {
    sim.world = SimWorld::new();
    let w = &mut sim.world;
    let players = [TeamId(1), TeamId(2), TeamId(3)];
    for (player, team) in players.into_iter().enumerate() {
        w.team_players.insert(team, player as Int);
        w.player_default_teams.insert(player as Int, team);
    }
    w.team_is_neutral.insert(TeamId(1), 1);
    w.neutral_team_id = TeamId(1);
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
    w.rng.init_random_with_seed(1);
    // The harness world: 20-unit partition cells over its 1000 by 1000 terrain.
    sim.init_partition(
        20.0,
        Region3D {
            lo: Coord3D::new(0.0, 0.0, 0.0),
            hi: Coord3D::new(1000.0, 1000.0, 100.0),
        },
    );
}

fn spawn(sim: &mut SimContext, name: &str, team: TeamId, x: u32, y: u32) -> ObjectSlot {
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
    sim.set_object_position(
        slot,
        &Coord3D::new(Real::from_bits(x), Real::from_bits(y), 0.0),
    );
    add_firing_tracker_stand_in(sim, slot);
    slot
}

/// Puts the stand-in where `FiringTracker` sits: just before the bonus helper.
fn add_firing_tracker_stand_in(sim: &mut SimContext, owner: ObjectSlot) {
    let Some(helper) = sim
        .object(owner)
        .and_then(|o| o.temp_weapon_bonus_helper_slot)
    else {
        return;
    };
    let tag = sim.rules.name_keys.name_to_key(TAGS[FIRING_TRACKER]);
    let slot = sim.world.attach_module(
        owner,
        tag,
        ModuleDataId::INVALID,
        FacetMask::UPDATE,
        SleepyUpdatePhase::Final,
        Box::new(SleepingStandIn(tag)),
    );
    sim.world
        .modules
        .meta_mut(slot)
        .schedule
        .set_next_call_frame(UPDATE_SLEEP_FOREVER);
    let behaviors = &mut sim.world.objects.get_mut(owner).expect("object").behaviors;
    let entry = behaviors.pop().expect("stand-in entry");
    let at = behaviors
        .iter()
        .position(|&(ms, _)| ms == helper)
        .expect("helper entry");
    behaviors.insert(at, entry);
}

/// The object, unless it was never made or has been destroyed.
fn live(sim: &SimContext, slot: Option<ObjectSlot>) -> Option<ObjectSlot> {
    slot.filter(|&s| sim.object(s).is_some_and(|o| !o.is_destroyed()))
}

fn tag_code(sim: &mut SimContext, slot: ModuleSlot) -> u32 {
    let key = sim.world.modules.meta(slot).tag_key;
    TAGS.iter()
        .position(|t| sim.rules.name_keys.name_to_key(t) == key)
        .expect("known update tag") as u32
}

/// `(object place * 16 + tag, priority)` for every entry of the sleepy update heap.
fn read_heap(sim: &mut SimContext, places: &[Option<ObjectSlot>]) -> Vec<(u32, u32, ModuleSlot)> {
    let entries = sim.world.sleepy_updates.as_slice().to_vec();
    entries
        .into_iter()
        .map(|ms| {
            let owner = sim.world.modules.meta(ms).owner_slot;
            let place = places
                .iter()
                .position(|&p| p == Some(owner))
                .expect("known owner") as u32;
            let code = place * 16 + tag_code(sim, ms);
            (code, sim.world.modules.meta(ms).schedule.priority(), ms)
        })
        .collect()
}

/// Rearranges the heap into the recorded layout.
fn adopt_layout(sim: &mut SimContext, places: &[Option<ObjectSlot>], codes: &[u32]) {
    let current = read_heap(sim, places);
    assert_eq!(current.len(), codes.len(), "sleepy update count");
    let world = &mut sim.world;
    world.sleepy_updates.clear(&mut world.modules);
    for &code in codes {
        let slot = current
            .iter()
            .find(|e| e.0 == code)
            .unwrap_or_else(|| panic!("no update module {code}"))
            .2;
        world.sleepy_updates.push(&mut world.modules, slot);
    }
}

fn tower_module(sim: &mut SimContext, owner: ObjectSlot) -> ModuleSlot {
    let key = sim.rules.name_keys.name_to_key("WeaponBonusUpdate");
    sim.object(owner)
        .expect("tower")
        .behaviors
        .iter()
        .map(|&(ms, _)| ms)
        .find(|&ms| {
            sim.world
                .modules
                .payload(ms)
                .is_some_and(|p| p.module_name_key() == key)
        })
        .expect("WeaponBonusUpdate")
}

fn save_module(sim: &mut SimContext, slot: ModuleSlot) -> Vec<u8> {
    let (mut payload, _ctx, generation) =
        sim.world.modules.take_payload(slot).expect("take payload");
    let mut xs = XferSave::new();
    payload.xfer(&mut xs, sim).expect("xfer save");
    sim.world.modules.restore_payload(slot, generation, payload);
    xs.as_bytes().to_vec()
}

/// The harness's `step_single_update`, which leaves the heap as it is.
fn step(sim: &mut SimContext, slot: ModuleSlot, owner: ObjectSlot, frame: u32) -> u32 {
    if live(sim, Some(owner)).is_none()
        || sim
            .module_schedule(slot)
            .expect("schedule")
            .next_call_frame()
            > frame
    {
        return NOT_CALLED;
    }
    let disabled = sim.object(owner).expect("owner").disabled_mask.clone();
    let (raw, sleep) = sim
        .dispatch_module(slot, |m, s, c| {
            let update = m.as_update_mut().expect("update facet");
            if update.disabled_types_to_process().test_for_all(&disabled) == FALSE {
                return (DISABLED, 1);
            }
            let sleep = update.update(s, c).0;
            (sleep, sleep.max(1))
        })
        .expect("dispatch update");
    let next = frame.saturating_add(sleep).min(UPDATE_SLEEP_FOREVER);
    sim.world
        .modules
        .meta_mut(slot)
        .schedule
        .set_next_call_frame(next);
    raw
}

struct Outcome {
    rows: Vec<[u32; 7]>,
    priorities: u32,
    detail: Vec<String>,
    xfer: String,
    crc: u32,
}

fn run(sim: &mut SimContext, line: &str) -> Outcome {
    let head = numbers(line, "head");
    let [frame0, towers, variant0, variant1, tower_team, tx, ty] = head[..] else {
        panic!("head");
    };
    let units = numbers(line, "units");
    let units: Vec<&[u32]> = units.chunks(5).collect();
    let ops = numbers(line, "ops");
    reset_world(sim, frame0);

    let mut places: Vec<Option<ObjectSlot>> = Vec::new();
    for (t, variant) in [variant0, variant1].into_iter().enumerate() {
        places.push((t < towers as usize).then(|| {
            let name = format!("WB_Tower{variant}");
            spawn(sim, &name, TEAMS[tower_team as usize], tx, ty)
        }));
    }
    for u in &units {
        let slot = spawn(sim, UNITS[u[0] as usize], TEAMS[u[1] as usize], u[2], u[3]);
        places.push(Some(slot));
    }
    for (k, u) in units.iter().enumerate() {
        if u[4] != 0 {
            let holder = places[1 + u[4] as usize].expect("box");
            let rider = places[2 + k].expect("rider");
            sim.contain_add(holder, rider);
        }
    }
    // Identities outlive the objects, so that destroyed places stay empty.
    let ids: Vec<Option<ObjectId>> = places
        .iter()
        .map(|p| p.map(|s| sim.object(s).expect("object").id))
        .collect();

    adopt_layout(sim, &places, &numbers(line, "heap"));
    let mut prio = Fnv::new();
    for entry in read_heap(sim, &places) {
        prio.word(entry.1);
    }

    let mut frame = frame0;
    let mut rows = Vec::new();
    let mut detail = Vec::new();
    for op in ops.chunks(4) {
        let (code, a, b, c) = (op[0], op[1], op[2], op[3]);
        sim.update_partition();
        let current = |sim: &SimContext, place: usize| {
            ids[place]
                .and_then(|id| sim.find_object(id))
                .and_then(|s| live(sim, Some(s)))
        };
        let unit = matches!(code, 2 | 3 | 4 | 5 | 8)
            .then(|| current(sim, 2 + a as usize))
            .flatten();
        let mut sleep = NOT_CALLED;
        match code {
            0 | 9 => {
                frame += a;
                sim.world.frame = frame;
                if let Some(t) = current(sim, usize::from(code == 9)) {
                    let m = tower_module(sim, t);
                    sleep = step(sim, m, t, frame);
                }
            }
            1 => {
                frame += a;
                sim.world.frame = frame;
                for place in 0..ids.len() {
                    let Some(o) = current(sim, place) else {
                        continue;
                    };
                    if let Some(h) = sim.object(o).and_then(|o| o.temp_weapon_bonus_helper_slot) {
                        step(sim, h, o, frame);
                    }
                }
            }
            2 => {
                if let Some(u) = unit {
                    let pos = Coord3D::new(Real::from_bits(b), Real::from_bits(c), 0.0);
                    sim.set_object_position(u, &pos);
                }
            }
            3 => {
                if let Some(u) = unit {
                    if sim.object(u).expect("unit").is_effectively_dead() == FALSE {
                        kill_object(sim, u, DamageType::Unresistable, DeathType::Normal);
                    }
                }
            }
            4 => {
                let rider = matches!(units[a as usize][0], 0 | 2 | 3);
                if let Some(u) = unit.filter(|_| rider) {
                    let o = sim.object(u).expect("unit");
                    let free = o.is_effectively_dead() == FALSE && o.contained_by.is_none();
                    let team = o.team_id;
                    if let Some(holder) = current(sim, 2 + b as usize).filter(|_| free) {
                        let h = sim.object(holder).expect("box");
                        if h.is_effectively_dead() == FALSE && h.team_id == team {
                            sim.contain_add(holder, u);
                        }
                    }
                }
            }
            5 => {
                if let Some(u) = unit {
                    if let Some(holder) = sim.object(u).expect("unit").contained_by {
                        sim.contain_remove(holder, u, FALSE);
                    }
                }
            }
            6 => {
                let place = if a == 0 { 0 } else { 1 + a as usize };
                if let Some(o) = current(sim, place) {
                    sim.world
                        .objects
                        .get_mut(o)
                        .expect("object")
                        .set_undetected_defector(u8::from(b != 0).into());
                }
            }
            7 => {
                if let Some(t) = current(sim, 0) {
                    if a != 0 {
                        sim.set_disabled(t, DisabledType::Emp);
                    } else {
                        sim.clear_disabled(t, DisabledType::Emp);
                    }
                }
            }
            8 => {
                if let Some(u) = unit {
                    sim.destroy_object(u);
                    sim.process_destroy_list();
                }
            }
            other => panic!("op {other}"),
        }

        let mut modules = Fnv::new();
        let mut objects = Fnv::new();
        let mut next = [0_u32; 2];
        let mut bonused = 0;
        for (t, slot) in next.iter_mut().enumerate() {
            let Some(o) = current(sim, t) else {
                continue;
            };
            let m = tower_module(sim, o);
            *slot = sim.module_schedule(m).expect("schedule").next_call_frame();
            let text = hex(&save_module(sim, m));
            modules.bytes(text.as_bytes());
        }
        detail.clear();
        for place in 0..ids.len() {
            let mut d = [0_u32; 4];
            if let Some(o) = current(sim, place) {
                let obj = sim.object(o).expect("object");
                d[0] = 1;
                d[1] = obj.weapon_bonus_condition;
                if let Some(h) = obj.temp_weapon_bonus_helper_slot {
                    d[2] = sim.module_schedule(h).expect("schedule").next_call_frame();
                    let mut f = Fnv::new();
                    f.bytes(hex(&save_module(sim, h)).as_bytes());
                    d[3] = f.0;
                }
                bonused += u32::from(d[1] != 0);
            }
            for word in d {
                objects.word(word);
            }
            detail.push(format!("{} {} {} {}", d[0], d[1], d[2], d[3]));
        }
        // A destroyed object's place keeps its slot, which no heap entry names any more.
        let mut heap = Fnv::new();
        for (code, priority, _) in read_heap(sim, &places) {
            heap.word(code);
            heap.word(priority);
        }
        rows.push([
            sleep, next[0], next[1], modules.0, objects.0, heap.0, bonused,
        ]);
    }

    let (mut xfer, mut crc) = (String::new(), 0);
    if let Some(o) = ids[0]
        .and_then(|id| sim.find_object(id))
        .and_then(|s| live(sim, Some(s)))
    {
        let m = tower_module(sim, o);
        xfer = hex(&save_module(sim, m));
        let mut xc = XferCrc::new();
        sim.world.modules.payload(m).expect("payload").crc(&mut xc);
        crc = xc.get_crc();
    }
    Outcome {
        rows,
        priorities: prio.0,
        detail,
        xfer,
        crc,
    }
}

#[test]
fn module_data_matches_the_reference_parser() {
    let mut sim = SimContext::new();
    sim.rules = build_rules();
    reset_world(&mut sim, 100);
    let rows: Vec<&str> = JSON
        .lines()
        .filter(|l| l.starts_with("{\"variant\""))
        .collect();
    assert_eq!(rows.len(), 9);
    for line in rows {
        let variant = get_u32(line, "variant").expect("variant");
        let slot = spawn(&mut sim, &format!("WB_Tower{variant}"), TEAMS[0], 0, 0);
        let m = tower_module(&mut sim, slot);
        let data = &sim
            .world
            .modules
            .payload(m)
            .and_then(|p| p.as_any())
            .and_then(|a| a.downcast_ref::<WeaponBonusUpdate>())
            .expect("module")
            .data;
        let set = |mask: &zh_logic::rules::kind_of::KindOfMaskType| -> Vec<u32> {
            (0..256).filter(|&i| mask.test(i as Int) != FALSE).collect()
        };
        assert_eq!(
            set(&data.required_affect_kind_of),
            numbers(line, "required"),
            "variant {variant} required"
        );
        assert_eq!(
            set(&data.forbidden_affect_kind_of),
            numbers(line, "forbidden"),
            "variant {variant} forbidden"
        );
        let condition = line
            .split("\"condition\":")
            .nth(1)
            .and_then(|t| t.split('}').next())
            .and_then(|t| t.parse::<Int>().ok())
            .expect("condition");
        assert_eq!(
            (
                Some(data.bonus_duration),
                Some(data.bonus_delay),
                Some(data.bonus_range.to_bits()),
                data.bonus_condition_type,
            ),
            (
                get_u32(line, "duration"),
                get_u32(line, "delay"),
                get_u32(line, "range"),
                condition,
            ),
            "variant {variant}"
        );
    }
}

#[test]
fn pkt388_matches_reference() {
    let mut sim = SimContext::new();
    sim.rules = build_rules();
    let vectors: Vec<&str> = extract_vector_lines(JSON)
        .into_iter()
        .filter(|l| l.contains("\"seed\""))
        .collect();
    assert_eq!(vectors.len(), 1000);

    let mut failures = 0;
    let mut first = Vec::new();
    for (index, line) in vectors.into_iter().enumerate() {
        let actual = run(&mut sim, line);
        let mut why = Vec::new();
        if get_u32(line, "priorities") != Some(actual.priorities) {
            why.push("starting priorities".to_owned());
        }
        let expected = numbers(line, "rows");
        let ops = numbers(line, "ops");
        assert_eq!(expected.len(), actual.rows.len() * COLUMNS.len());
        'rows: for (r, (want, got)) in expected.chunks(7).zip(&actual.rows).enumerate() {
            for (c, name) in COLUMNS.iter().enumerate() {
                if want[c] != got[c] {
                    why.push(format!(
                        "row {r} op {:?} {name}: want {} got {}",
                        &ops[r * 4..r * 4 + 4],
                        want[c],
                        got[c]
                    ));
                    break 'rows;
                }
            }
        }
        if let Some(detail) = get_str(line, "detail") {
            if detail != actual.detail.join(",") {
                why.push(format!(
                    "final objects: want {detail} got {}",
                    actual.detail.join(",")
                ));
            }
        }
        if get_str(line, "xfer") != Some(actual.xfer.as_str()) {
            why.push("final xfer".to_owned());
        }
        if get_u32(line, "crc") != Some(actual.crc) {
            why.push("crc".to_owned());
        }
        if !why.is_empty() {
            failures += 1;
            if first.len() < 5 {
                first.push(format!("vector {index}: {}", why.join("; ")));
            }
        }
    }
    assert_eq!(failures, 0, "mismatching vectors\n{}", first.join("\n"));
}
