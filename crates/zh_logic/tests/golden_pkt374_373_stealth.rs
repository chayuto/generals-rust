//! `PKT-374` `StealthUpdate` and `PKT-373` `StealthDetectorUpdate` against vectors from
//! the reference modules (`tests/cpp_harness/gen_pkt374_373_stealth.cpp`).

use std::fmt::Write as _;
use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniError, IniLoadType, IniReader};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::{NameKeyType, ObjectId, Relationship, TeamId, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferCrc, XferError, XferLoad, XferSave};
use zh_logic::arena::{FacetMask, ModuleContext, ModuleSlot, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::modules::module::{ContainFacet, SpawnBehaviorFacet};
use zh_logic::modules::module_factory::ModuleParseContext;
use zh_logic::modules::{BehaviorModule, CommandSourceType, ModuleData, ModuleType};
use zh_logic::object::weapon::{Weapon, WeaponTemplate};
use zh_logic::object::weapon_set::WeaponSlotType;
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::model_state::ModelConditionFlagType;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::scheduler::{SleepyUpdatePhase, UPDATE_SLEEP_FOREVER};
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimView, SimWorld};

const S: usize = 0;
const R: usize = 1;
const X: usize = 2;
const D: usize = 3;
const E: usize = 4;
const M: usize = 5;
const G: usize = 6;
const T: usize = 7;
const OBJECTS: usize = 8;

/// Neutral, A, B by player index, with the reference's `TeamID`s.
const TEAMS: [TeamId; 3] = [TeamId(3), TeamId(1), TeamId(2)];

const OFFSETS: [[f32; 3]; 15] = [
    [0.0, 0.0, 0.0],
    [10.0, 0.0, 0.0],
    [0.0, 29.0, 0.0],
    [31.0, 0.0, 0.0],
    [0.0, 49.0, 0.0],
    [50.0, 0.0, 0.0],
    [0.0, 51.0, 0.0],
    [99.0, 0.0, 0.0],
    [60.0, 80.0, 0.0],
    [0.0, 101.0, 0.0],
    [80.0, 0.0, 60.0],
    [150.0, 0.0, 0.0],
    [400.0, 0.0, 0.0],
    [0.0, 20.0, 40.0],
    [30.0, 40.0, 0.0],
];

const STATUSES: [ObjectStatusType; 10] = [
    ObjectStatusType::IsFiringWeapon,
    ObjectStatusType::IsUsingAbility,
    ObjectStatusType::IsAttacking,
    ObjectStatusType::CanStealth,
    ObjectStatusType::Stealthed,
    ObjectStatusType::Detected,
    ObjectStatusType::IsAimingWeapon,
    ObjectStatusType::Braking,
    ObjectStatusType::UnderConstruction,
    ObjectStatusType::Sold,
];

const VELOCITIES: [[f32; 3]; 8] = [
    [0.0, 0.0, 0.0],
    [0.1, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [1.01, 0.0, 0.0],
    [0.0, 0.0, 4.9],
    [3.0, 4.0, 0.0],
    [6.0, 6.0, 6.0],
    [0.07, 0.07, 0.0],
];

fn position(p: usize) -> Coord3D {
    Coord3D::new(200.0 + OFFSETS[p][0], 200.0 + OFFSETS[p][1], OFFSETS[p][2])
}

/// Stand-in for the unported `AIUpdateInterface`: only the slot it gives the object
/// matters here.
struct SlotStub(NameKeyType);

impl BehaviorModule for SlotStub {
    fn module_name_key(&self) -> NameKeyType {
        self.0
    }
    fn xfer(&mut self, _xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        Ok(())
    }
}

#[derive(Debug)]
struct SlotStubData(NameKeyType, NameKeyType, FacetMask);

impl ModuleData for SlotStubData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn tag_key(&self) -> NameKeyType {
        self.0
    }
    fn module_name_key(&self) -> NameKeyType {
        self.1
    }
    fn facet_mask(&self) -> FacetMask {
        self.2
    }
}

fn parse_slot_stub(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    reader.read_block_fields(|_, _| Err(IniError::UnknownToken))?;
    Ok(Box::new(SlotStubData(
        tag_key,
        module_name_key,
        FacetMask::NONE,
    )))
}

fn create_slot_stub(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    Box::new(SlotStub(data.module_name_key()))
}

fn rules_from(objects: &str) -> SimContext {
    let mut sim = SimContext::new();
    let mut rules = RulesRegistry::new();
    rules.module_registry.register_module(
        &mut rules.name_keys,
        "AIUpdateInterface",
        ModuleType::Behavior,
        FacetMask::AI_UPDATE,
        SleepyUpdatePhase::Normal,
        true,
        parse_slot_stub,
        create_slot_stub,
    );
    let mut reader = IniReader::new("object.ini", objects.as_bytes(), IniLoadType::Overwrite);
    if let Err(e) = rules.load_thing_templates_from_ini(&mut reader) {
        panic!("thing templates, line {}: {e:?}", reader.get_line_num());
    }
    sim.rules = rules;
    sim
}

fn reset_world(sim: &mut SimContext) {
    sim.world = SimWorld::new();
    sim.init_partition(
        20.0,
        Region3D {
            lo: Coord3D::new(0.0, 0.0, 0.0),
            hi: Coord3D::new(1000.0, 1000.0, 100.0),
        },
    );
    let w = &mut sim.world;
    for (player, team) in TEAMS.into_iter().enumerate() {
        w.team_players.insert(team, player as i32);
        w.player_default_teams.insert(player as i32, team);
    }
    w.team_is_neutral.insert(TEAMS[0], TRUE);
    w.neutral_team_id = TEAMS[0];
    w.local_player_index = 5;
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
}

fn spawn(sim: &mut SimContext, name: &str, team: TeamId, pos: Coord3D) -> ObjectSlot {
    let tid = sim
        .rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"));
    sim.spawn_object(tid, team, pos, 0.0)
}

fn module_named(sim: &mut SimContext, owner: ObjectSlot, name: &str) -> Option<ModuleSlot> {
    let key = sim.rules.name_keys.name_to_key(name);
    let obj = sim.world.objects.get(owner).expect("object");
    obj.behaviors.iter().map(|&(ms, _)| ms).find(|&ms| {
        sim.world
            .modules
            .payload(ms)
            .is_some_and(|p| p.module_name_key() == key)
    })
}

fn save(sim: &mut SimContext, slot: ModuleSlot) -> Vec<u8> {
    let (mut payload, _ctx, generation) = sim.world.modules.take_payload(slot).expect("payload");
    let mut xs = XferSave::new();
    payload.xfer(&mut xs, sim).expect("xfer save");
    sim.world.modules.restore_payload(slot, generation, payload);
    xs.as_bytes().to_vec()
}

fn next_call_frame(sim: &SimContext, slot: ModuleSlot) -> u32 {
    sim.module_schedule(slot)
        .expect("schedule")
        .next_call_frame()
}

fn step(sim: &mut SimContext, m: ModuleSlot, frame: u32) {
    sim.world.frame = frame;
    if next_call_frame(sim, m) > frame {
        return;
    }
    let sleep = sim
        .dispatch_module(m, |b, s, c| b.as_update_mut().expect("update").update(s, c))
        .expect("dispatch");
    let sleep = if sleep.0 < 1 {
        UpdateSleepTime::NONE
    } else {
        sleep
    };
    let next = frame.saturating_add(sleep.0).min(UPDATE_SLEEP_FOREVER);
    sim.world
        .modules
        .meta_mut(m)
        .schedule
        .set_next_call_frame(next);
    let index = sim.world.modules.meta(m).schedule.index_in_logic();
    if index >= 0 {
        sim.world
            .sleepy_updates
            .rebalance(&mut sim.world.modules, index as usize);
    }
}

struct Scene {
    o: [Option<ObjectSlot>; OBJECTS],
    /// Stealth unit, rider, second disguiser, detector.
    m: [Option<ModuleSlot>; 4],
}

impl Scene {
    fn obj(&self, i: usize) -> ObjectSlot {
        self.o[i].expect("object in vector")
    }

    fn observe(&self, sim: &mut SimContext) -> Vec<String> {
        let mut out = Vec::new();
        for m in self.m {
            let mut s = String::new();
            if let Some(m) = m {
                for b in save(sim, m) {
                    let _ = write!(s, "{b:02x}");
                }
                let mut xc = XferCrc::new();
                sim.world.modules.payload(m).expect("payload").crc(&mut xc);
                let _ = write!(s, " {} {}", xc.get_crc(), next_call_frame(sim, m));
            }
            out.push(s);
        }
        for o in self.o {
            let mut s = String::new();
            if let Some(o) = o {
                let player = {
                    let team = sim.object(o).expect("object").team_id;
                    sim.player_index(team)
                };
                let record = sim.object(o).expect("object");
                let bits = (0..45)
                    .filter(|&b| record.status.test(b) != FALSE)
                    .fold(0u64, |acc, b| acc | (1 << b));
                let v = record
                    .physics
                    .as_ref()
                    .map_or_else(Coord3D::default, |p| p.vel);
                let disguised = record
                    .model_condition
                    .test(ModelConditionFlagType::Disguised.as_i32());
                let garrisoned = record
                    .model_condition
                    .test(ModelConditionFlagType::Garrisoned.as_i32());
                let _ = write!(
                    s,
                    "{bits} {disguised}{garrisoned} {player} {} {} {}",
                    v.x.to_bits(),
                    v.y.to_bits(),
                    v.z.to_bits()
                );
            }
            out.push(s);
        }
        let count = |p: i32| sim.world.vehicles_disguised.get(&p).copied().unwrap_or(0);
        out.push(format!(
            "{} {} {}",
            sim.world.rng.get_game_logic_random_seed_crc(),
            count(1),
            count(2)
        ));
        out
    }

    fn woken(&self, sim: &SimContext) -> String {
        [S, E]
            .iter()
            .map(|&i| {
                let id = self.o[i]
                    .and_then(|o| sim.object(o))
                    .map_or(ObjectId::INVALID, |o| o.id);
                let woke = sim
                    .world
                    .ai_commands
                    .iter()
                    .any(|(who, what)| *who == id && what == "wakeUpAndAttemptToTarget");
                if woke {
                    '1'
                } else {
                    '0'
                }
            })
            .collect()
    }
}

/// Compares one row, where `=` repeats the previous row's entry.
fn check(actual: &[String], expected: &str, previous: &mut Vec<String>, what: &str) {
    let parts: Vec<&str> = expected.split(';').collect();
    assert_eq!(parts.len(), actual.len(), "{what}: {expected}");
    let full: Vec<String> = parts
        .iter()
        .enumerate()
        .map(|(i, p)| {
            if *p == "=" {
                previous[i].clone()
            } else {
                (*p).to_owned()
            }
        })
        .collect();
    for (i, (a, e)) in actual.iter().zip(&full).enumerate() {
        assert_eq!(a, e, "{what}: field {i}");
    }
    *previous = full;
}

fn between<'a>(line: &'a str, start: &str, end: &str) -> &'a str {
    let rest = &line[line.find(start).expect(start) + start.len()..];
    &rest[..rest.find(end).expect(end)]
}

fn numbers(text: &str) -> Vec<u32> {
    text.split(['[', ']', ','])
        .filter(|t| !t.is_empty())
        .map(|t| t.parse().expect("number"))
        .collect()
}

fn apply(sim: &mut SimContext, sc: &Scene, frame: &mut u32, code: u32, a: u32, b: u32) {
    let b = b as usize;
    match code {
        0 => {
            for _ in 0..a {
                *frame += 1;
                sim.world.frame = *frame;
                sim.update_partition();
                for m in sc.m.into_iter().flatten() {
                    step(sim, m, *frame);
                }
            }
        }
        1 => {
            let o = sc.obj(b);
            let status = STATUSES[a as usize];
            if sim.object(o).expect("object").status.test(status.as_i32()) != FALSE {
                sim.clear_status(o, status);
            } else {
                sim.set_status(o, status);
            }
        }
        2 => {
            let o = sc.obj(b);
            let mut info = DamageInfo::default();
            info.input.damage_type = if a == 0 {
                DamageType::Unresistable
            } else {
                DamageType::Healing
            };
            info.input.death_type = DeathType::Normal;
            info.input.amount = 1.0;
            info.input.source_id = ObjectId::INVALID;
            if a == 0 {
                sim.attempt_damage(o, &mut info);
            } else {
                info.input.source_id = sim.object(sc.obj(E)).expect("enemy").id;
                sim.attempt_healing(o, &mut info);
            }
        }
        3 => {
            let o = sc.obj(b);
            let record = sim.object_mut(o).expect("object");
            if let Some(p) = record.physics.as_mut() {
                let v = VELOCITIES[a as usize];
                p.vel = Coord3D::new(v[0], v[1], v[2]);
            }
        }
        4 => {
            let (slot, ago) = ((a & 0xf) as usize, (a >> 4) & 0xf);
            let now = *frame;
            let o = sc.obj(S);
            if let Some(gun) = sim.object_mut(o).expect("object").weapon_set.weapons[slot].as_mut()
            {
                gun.last_fire_frame = if now >= ago { now - ago } else { 0 };
            }
            if a >> 8 != 0 {
                sim.set_status(o, ObjectStatusType::IsFiringWeapon);
            }
        }
        5 => {
            let o = sc.obj(b);
            sim.object_mut(o).expect("object").script_status ^= 0x08;
        }
        6 => sim.mark_stealth_detected(sc.obj(b), a),
        7 => sim.receive_stealth_grant(sc.obj(b), (a & 1) as u8, a >> 1),
        8 => {
            let target = (a != 0).then(|| sc.obj(a as usize));
            sim.stealth_disguise_as_object(sc.obj(b), target);
        }
        9 => {
            let victim = (a != 0).then(|| sc.obj(a as usize));
            let o = sc.obj(S);
            let record = sim.object_mut(o).expect("object");
            if record.ai_slot.is_some() {
                record.ai_current_victim = victim;
            }
        }
        10 => {
            let source = match a {
                0 => CommandSourceType::FromPlayer,
                1 => CommandSourceType::FromScript,
                2 => CommandSourceType::FromAi,
                _ => panic!("command source {a}"),
            };
            let o = sc.obj(S);
            sim.object_mut(o).expect("object").ai_last_command_source = source;
        }
        11 => {
            if let Some(o) = sc.o[b] {
                if sim.object(o).expect("object").contained_by.is_none() {
                    sim.set_object_position(o, &position(a as usize));
                }
            }
        }
        12 => sim.set_stealth_detector_enabled(sc.obj(D), a as u8),
        13 => {
            let unit = sc.obj(b);
            if let Some(holder) = sim.object(unit).expect("object").contained_by {
                sim.contain_remove(holder, unit, (a >> 8) as u8);
            } else if a & 0xff != 0 {
                sim.contain_add(sc.obj((a & 0xff) as usize), unit);
            }
        }
        14 => {
            let o = sc.obj(b);
            let record = sim.object_mut(o).expect("object");
            let dead = record.is_effectively_dead();
            record.set_effectively_dead(u8::from(dead == FALSE));
        }
        15 => {
            let m = sc.m[b].expect("module");
            let bytes = save(sim, m);
            let (mut payload, _ctx, generation) =
                sim.world.modules.take_payload(m).expect("payload");
            let mut load = XferLoad::new(&bytes);
            payload.xfer(&mut load, sim).expect("xfer load");
            sim.world.modules.restore_payload(m, generation, payload);
            sim.dispatch_module(m, |p, s, c| p.load_post_process(s, c))
                .expect("dispatch");
        }
        _ => panic!("op {code}"),
    }
}

#[test]
fn stealth_and_detector_match_reference() {
    let json = include_str!("../../../docs/golden_vectors/PKT374_373_Stealth.json");
    let header = |key: &str| {
        let line = json
            .lines()
            .find(|l| l.starts_with(&format!("\"{key}\"")))
            .expect(key);
        get_str(line, key).expect(key).replace('|', "\n")
    };
    let mut sim = rules_from(&header("object_ini"));
    let lines: Vec<&str> = json
        .lines()
        .map(|l| l.trim_end_matches(','))
        .filter(|l| l.starts_with("{\"v\""))
        .collect();
    assert_eq!(lines.len(), 1600);

    let mut checked = 0;
    for (index, line) in lines.iter().enumerate() {
        let get = |key: &str| get_u32(line, key).expect(key);
        let seed = get("seed");
        let mut frame = get("frame");
        let dteam = TEAMS[get("dteam") as usize];
        let rider = get("rider") == 1;
        let weapons = get("weapons");
        let pos = numbers(between(line, "\"pos\":[", "]"));
        let at = |i: usize| position(pos[i] as usize);

        reset_world(&mut sim);
        let rel = match get("rel") {
            0 => Relationship::Enemies,
            1 => Relationship::Neutral,
            2 => Relationship::Allies,
            other => panic!("relationship {other}"),
        };
        sim.world.player_relationships.insert((1, 2), rel);
        sim.world.player_relationships.insert((2, 1), rel);
        sim.world.frame = frame;
        sim.world.rng.init_random_with_seed(seed);
        let mut o = [None; OBJECTS];
        o[S] = Some(spawn(
            &mut sim,
            &format!("P374_S{}", get("sv")),
            TEAMS[1],
            at(S),
        ));
        if rider {
            sim.world.rng.init_random_with_seed(seed + 1);
            o[R] = Some(spawn(
                &mut sim,
                &format!("P374_R{}", get("rv")),
                TEAMS[1],
                at(R),
            ));
        }
        sim.world.rng.init_random_with_seed(seed + 2);
        o[X] = Some(spawn(
            &mut sim,
            &format!("P374_X{}", get("xv")),
            TEAMS[2],
            at(X),
        ));
        sim.world.rng.init_random_with_seed(seed + 3);
        o[D] = Some(spawn(
            &mut sim,
            &format!("P373_D{}", get("dv")),
            dteam,
            at(D),
        ));
        o[E] = Some(spawn(&mut sim, "P374_Enemy", TEAMS[2], at(E)));
        o[M] = Some(spawn(&mut sim, "P374_Market", TEAMS[1], at(M)));
        o[G] = Some(spawn(&mut sim, "P374_House", TEAMS[0], at(G)));
        o[T] = Some(spawn(&mut sim, "P374_Truck", dteam, at(T)));

        // The harness builds the weapons from the template's weapon set.
        let s = o[S].expect("stealth unit");
        for (slot, wslot) in [
            WeaponSlotType::Primary,
            WeaponSlotType::Secondary,
            WeaponSlotType::Tertiary,
        ]
        .into_iter()
        .enumerate()
        {
            if weapons & (1 << slot) != 0 {
                sim.object_mut(s).expect("object").weapon_set.weapons[slot] =
                    Some(Weapon::new(WeaponTemplate::default(), wslot, frame));
            }
        }

        let mut m = [None; 4];
        for (i, slot) in m.iter_mut().enumerate() {
            let name = if i == D {
                "StealthDetectorUpdate"
            } else {
                "StealthUpdate"
            };
            *slot = o[i].and_then(|obj| module_named(&mut sim, obj, name));
        }
        let sc = Scene { o, m };
        if rider {
            sim.contain_add(sc.obj(S), sc.obj(R));
        }
        sim.update_partition();
        sim.world.rng.init_random_with_seed(seed + 4);

        let mut previous = Vec::new();
        let first = sc.observe(&mut sim);
        check(
            &first,
            get_str(line, "spawn").expect("spawn"),
            &mut previous,
            &format!("vector {index} spawn"),
        );

        let ops = numbers(between(line, "\"ops\":[", "],\"rows\""));
        let rows: Vec<&str> = between(line, "\"rows\":[\"", "\"]}")
            .split("\",\"")
            .collect();
        assert_eq!(ops.len(), rows.len() * 3, "vector {index}");
        for (k, (op, expected)) in ops.chunks(3).zip(&rows).enumerate() {
            let (code, a, b) = (op[0], op[1], op[2]);
            sim.world.ai_commands.clear();
            sim.world.frame = frame;
            apply(&mut sim, &sc, &mut frame, code, a, b);
            let what = format!(
                "vector {index} (sv {}, dv {}) op {k} ({code},{a},{b})",
                get("sv"),
                get("dv")
            );
            let (woken, rest) = expected.split_once(';').expect("row");
            assert_eq!(sc.woken(&sim), woken, "{what}: woken");
            let now = sc.observe(&mut sim);
            check(&now, rest, &mut previous, &what);
            checked += 1;
        }
    }
    assert!(checked > 25_000, "only {checked} rows");
}

// What follows has no reference: the paths whose other side is not ported
// (`RiderChangeContain`, `SpawnBehavior`) and a few that the vectors cannot reach.

/// A rider-change contain and a spawn behavior with unstealthed slaves.
#[derive(Default)]
struct HostStub {
    riders: Vec<ObjectId>,
}

impl BehaviorModule for HostStub {
    fn module_name_key(&self) -> NameKeyType {
        NameKeyType(0)
    }
    fn xfer(&mut self, _xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        Ok(())
    }
    fn as_contain(&self) -> Option<&dyn ContainFacet> {
        Some(self)
    }
    fn as_contain_mut(&mut self) -> Option<&mut dyn ContainFacet> {
        Some(self)
    }
    fn as_spawn_behavior(&self) -> Option<&dyn SpawnBehaviorFacet> {
        Some(self)
    }
    fn as_spawn_behavior_mut(&mut self) -> Option<&mut dyn SpawnBehaviorFacet> {
        Some(self)
    }
}

impl ContainFacet for HostStub {
    fn get_contain_count(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> u32 {
        self.riders.len() as u32
    }
    fn is_rider_change_contain(&self) -> u8 {
        TRUE
    }
    fn get_contained_items<'a>(
        &'a self,
        _sim: &'a dyn SimView,
        _ctx: ModuleContext,
    ) -> &'a [ObjectId] {
        &self.riders
    }
    fn add_to_contain(&mut self, sim: &mut dyn SimApi, _ctx: ModuleContext, obj: ObjectSlot) {
        self.riders.push(sim.object(obj).expect("rider").id);
    }
    fn remove_from_contain(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _obj: ObjectSlot,
        _expose: u8,
    ) {
    }
}

impl SpawnBehaviorFacet for HostStub {
    fn reveal_slaves(&mut self, sim: &mut dyn SimApi) {
        sim.rng_logic_int(0, 1);
    }
}

fn parse_host_stub(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    reader.read_block_fields(|_, _| Err(IniError::UnknownToken))?;
    Ok(Box::new(SlotStubData(
        tag_key,
        module_name_key,
        FacetMask::NONE,
    )))
}

fn create_host_stub(_data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    Box::new(HostStub::default())
}

fn unit_sim(objects: &str) -> SimContext {
    let mut sim = SimContext::new();
    let mut rules = RulesRegistry::new();
    rules.module_registry.register_module(
        &mut rules.name_keys,
        "HostStub",
        ModuleType::Behavior,
        FacetMask(FacetMask::CONTAIN.0 | FacetMask::SPAWN_BEHAVIOR.0),
        SleepyUpdatePhase::Normal,
        false,
        parse_host_stub,
        create_host_stub,
    );
    rules
        .load_thing_templates_from_ini(&mut IniReader::new(
            "object.ini",
            objects.as_bytes(),
            IniLoadType::Overwrite,
        ))
        .expect("templates");
    sim.rules = rules;
    reset_world(&mut sim);
    sim.world.frame = 100;
    sim
}

fn has(sim: &SimContext, o: ObjectSlot, status: ObjectStatusType) -> bool {
    sim.object(o).expect("object").status.test(status.as_i32()) != FALSE
}

const HOSTS: &str = "\
Object Bike
  Body = InactiveBody ModuleTag_Body
  End
  Behavior = HostStub ModuleTag_Host
  End
  Behavior = StealthUpdate ModuleTag_Stealth
    InnateStealth = No
    StealthDelay = 1000
  End
End
Object Rider
  Body = InactiveBody ModuleTag_Body
  End
  Behavior = StealthUpdate ModuleTag_Stealth
    InnateStealth = No
  End
End
Object Nest
  KindOf = SPAWNS_ARE_THE_WEAPONS
  Body = InactiveBody ModuleTag_Body
  End
  Behavior = HostStub ModuleTag_Host
  End
  Behavior = StealthUpdate ModuleTag_Stealth
    StealthDelay = 1000
  End
End
";

#[test]
fn a_grant_passes_to_the_rider_of_a_rider_change_contain() {
    let mut sim = unit_sim(HOSTS);
    let bike = spawn(&mut sim, "Bike", TEAMS[1], position(0));
    let rider = spawn(&mut sim, "Rider", TEAMS[1], position(0));
    sim.contain_add(bike, rider);
    sim.receive_stealth_grant(bike, TRUE, 1);
    for o in [bike, rider] {
        assert!(has(&sim, o, ObjectStatusType::CanStealth));
        assert!(has(&sim, o, ObjectStatusType::Stealthed));
        assert!(sim.stealth_params(o).expect("stealth").temporary_grant != FALSE);
    }
    sim.receive_stealth_grant(bike, FALSE, 0);
    for o in [bike, rider] {
        assert!(!has(&sim, o, ObjectStatusType::CanStealth));
        assert!(!has(&sim, o, ObjectStatusType::Stealthed));
        assert!(sim.stealth_params(o).expect("stealth").temporary_grant == FALSE);
    }
}

#[test]
fn unstealthed_slaves_stop_the_spawner_and_are_revealed_first() {
    let mut sim = unit_sim(HOSTS);
    let nest = spawn(&mut sim, "Nest", TEAMS[1], position(0));
    let m = module_named(&mut sim, nest, "StealthUpdate").expect("module");
    sim.set_status(nest, ObjectStatusType::Stealthed);
    let before = sim.world.rng.get_game_logic_random_seed_crc();
    step(&mut sim, m, 101);
    assert!(!has(&sim, nest, ObjectStatusType::Stealthed));
    // The stub's `revealSlaves` draws once.
    assert_ne!(sim.world.rng.get_game_logic_random_seed_crc(), before);

    // `SpawnBehavior` will ask each slave through `allowedToStealth`.
    let rider = spawn(&mut sim, "Rider", TEAMS[1], position(0));
    let slave = module_named(&mut sim, rider, "StealthUpdate").expect("module");
    let ask = |sim: &mut SimContext| {
        sim.dispatch_module(slave, |b, s, c| {
            let stealth = b.as_stealth_mut().expect("stealth");
            stealth.allowed_to_stealth(s, c, c.owner_slot)
        })
        .expect("dispatch")
    };
    assert_eq!(ask(&mut sim), FALSE);
    sim.set_status(rider, ObjectStatusType::CanStealth);
    assert_eq!(ask(&mut sim), TRUE);

    // Without `SPAWNS_ARE_THE_WEAPONS` the slaves are not asked.
    let bike = spawn(&mut sim, "Bike", TEAMS[1], position(0));
    let m = module_named(&mut sim, bike, "StealthUpdate").expect("module");
    sim.set_status(bike, ObjectStatusType::CanStealth);
    sim.set_status(bike, ObjectStatusType::Stealthed);
    let before = sim.world.rng.get_game_logic_random_seed_crc();
    sim.world.frame = 5000;
    step(&mut sim, m, 5000);
    assert!(has(&sim, bike, ObjectStatusType::Stealthed));
    assert_eq!(sim.world.rng.get_game_logic_random_seed_crc(), before);
}

#[test]
fn only_the_first_detector_is_switched_and_bad_input_is_rejected() {
    let ini = "\
Object Tower
  Body = InactiveBody ModuleTag_Body
  End
  Behavior = StealthDetectorUpdate ModuleTag_A
    InitiallyDisabled = Yes
    PingSound = Ping
    IRParticleSysBone = Bone01
  End
  Behavior = StealthDetectorUpdate ModuleTag_B
    InitiallyDisabled = Yes
  End
End
";
    let mut sim = unit_sim(ini);
    let tower = spawn(&mut sim, "Tower", TEAMS[1], position(0));
    let slots: Vec<ModuleSlot> = sim
        .object(tower)
        .expect("tower")
        .behaviors
        .iter()
        .map(|&(ms, _)| ms)
        .collect();
    let detectors = &slots[slots.len() - 2..];
    let wakes = |sim: &SimContext| -> Vec<u32> {
        detectors.iter().map(|&m| next_call_frame(sim, m)).collect()
    };
    assert_eq!(wakes(&sim), [UPDATE_SLEEP_FOREVER; 2]);
    sim.set_stealth_detector_enabled(tower, TRUE);
    assert_eq!(wakes(&sim), [101, UPDATE_SLEEP_FOREVER]);
    let enabled: Vec<u8> = detectors
        .iter()
        .map(|&m| {
            sim.dispatch_module(m, |b, _, _| {
                b.as_stealth_detector_mut()
                    .expect("detector")
                    .is_sd_enabled()
            })
            .expect("dispatch")
        })
        .collect();
    assert_eq!(enabled, [TRUE, FALSE]);
    sim.set_stealth_detector_enabled(tower, FALSE);
    assert_eq!(wakes(&sim), [UPDATE_SLEEP_FOREVER; 2]);

    for bad in [
        "  Behavior = StealthUpdate T\n    EnemyDetectionEvaEvent = Nothing\n  End\n",
        "  Behavior = StealthUpdate T\n    Unknown = 1\n  End\n",
        "  Behavior = StealthDetectorUpdate T\n    Unknown = 1\n  End\n",
    ] {
        let ini = format!("Object Bad\n{bad}End\n");
        let mut rules = RulesRegistry::new();
        let mut reader = IniReader::new("o.ini", ini.as_bytes(), IniLoadType::Overwrite);
        assert!(
            rules.load_thing_templates_from_ini(&mut reader).is_err(),
            "{bad}"
        );
    }
}

#[test]
fn both_updates_run_while_held_and_a_disguise_stops_the_physics() {
    let ini = "\
Object Spy
  Body = InactiveBody ModuleTag_Body
  End
  Behavior = StealthUpdate ModuleTag_Stealth
    DisguisesAsTeam = Yes
    DisguiseTransitionTime = 33
  End
  Behavior = StealthDetectorUpdate ModuleTag_Detect
  End
  Behavior = PhysicsBehavior ModuleTag_Phys
  End
End
";
    let mut sim = SimContext::new();
    let mut rules = RulesRegistry::new();
    rules
        .load_thing_templates_from_ini(&mut IniReader::new(
            "object.ini",
            ini.as_bytes(),
            IniLoadType::Overwrite,
        ))
        .expect("templates");
    sim.rules = rules;
    reset_world(&mut sim);
    sim.world.frame = 100;
    let spy = spawn(&mut sim, "Spy", TEAMS[1], position(0));
    for name in ["StealthUpdate", "StealthDetectorUpdate"] {
        let m = module_named(&mut sim, spy, name).expect(name);
        let mask = sim
            .dispatch_module(m, |b, _, _| {
                b.as_update().expect("update").disabled_types_to_process()
            })
            .expect("dispatch");
        assert!(mask.test(DisabledType::Held as i32) != FALSE, "{name}");
        assert_eq!(mask.to_unsigned_int().count_ones(), 1, "{name}");
    }

    let record = sim.object_mut(spy).expect("object");
    let p = record.physics.as_mut().expect("physics");
    p.vel = Coord3D::new(1.0, 2.0, 3.0);
    p.pitch_rate = 1.0;
    p.roll_rate = 2.0;
    p.yaw_rate = 3.0;
    let enemy = spawn(&mut sim, "Spy", TEAMS[2], position(1));
    sim.stealth_disguise_as_object(spy, Some(enemy));
    let m = module_named(&mut sim, spy, "StealthUpdate").expect("module");
    step(&mut sim, m, 101);
    let record = sim.object(spy).expect("object");
    assert!(has(&sim, spy, ObjectStatusType::Disguised));
    let p = record.physics.as_ref().expect("physics");
    assert_eq!(p.vel, Coord3D::default());
    assert_eq!([p.pitch_rate, p.roll_rate, p.yaw_rate], [0.0; 3]);
}

#[test]
fn a_save_naming_an_unknown_disguise_does_not_load() {
    let mut sim = unit_sim(HOSTS);
    let rider = spawn(&mut sim, "Rider", TEAMS[1], position(0));
    let other = spawn(&mut sim, "Bike", TEAMS[2], position(1));
    sim.stealth_disguise_as_object(rider, Some(other));
    let params = sim.stealth_params(rider).expect("stealth");
    assert_eq!(params.disguised_player_index, 2);
    let m = module_named(&mut sim, rider, "StealthUpdate").expect("module");
    let mut bytes = save(&mut sim, m);
    let at = bytes
        .windows(4)
        .position(|w| w == b"Bike")
        .expect("template name");
    bytes[at] = b'x';
    let load = |sim: &mut SimContext, bytes: &[u8]| {
        let (mut payload, _ctx, generation) = sim.world.modules.take_payload(m).expect("payload");
        let result = payload.xfer(&mut XferLoad::new(bytes), sim);
        sim.world.modules.restore_payload(m, generation, payload);
        result
    };
    assert!(load(&mut sim, &bytes).is_err());

    // A save without a disguise name clears the one in place.
    let other_rider = spawn(&mut sim, "Rider", TEAMS[1], position(0));
    let plain = module_named(&mut sim, other_rider, "StealthUpdate").expect("module");
    let plain = save(&mut sim, plain);
    sim.stealth_disguise_as_object(rider, Some(other));
    load(&mut sim, &plain).expect("load");
    let params = sim.stealth_params(rider).expect("stealth");
    assert_eq!(params.disguised_template, None);
}
