//! `PKT-383` `ToppleUpdate`, `PKT-377` `StructureToppleUpdate` and `PKT-305`
//! `AutoDepositUpdate` against vectors from the reference modules
//! (`tests/cpp_harness/gen_pkt383_377_305_topple.cpp`).
//!
//! The rules the reference vectors cannot observe are checked at the end without a
//! reference.

use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniLoadType, IniReader, SEPS_DEFAULT};
use zh_common::math::{Coord3D, Matrix3D};
use zh_common::types::{Bool, ObjectId, Relationship, TeamId, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{XferCrc, XferSave};
use zh_logic::arena::{ModuleSlot, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::modules::contain::open_contain::kill_object;
use zh_logic::modules::update::physics_update;
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::object::weapon::WeaponTemplate;
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::rules::model_state::ModelConditionFlagType;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::scheduler::UPDATE_SLEEP_FOREVER;
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimView, SimWorld};

const TEAM_A: TeamId = TeamId(1);
const TEAM_B: TeamId = TeamId(2);
const TEAM_NEUTRAL: TeamId = TeamId(3);
/// A second team of player A.
const TEAM_A2: TeamId = TeamId(4);
/// Teams of each player, by player index; the first is the default team.
const PLAYER_TEAMS: [&[TeamId]; 3] = [&[TEAM_NEUTRAL], &[TEAM_A, TEAM_A2], &[TEAM_B]];
const FX_NAMES: [&str; 9] = [
    "P383_Topple",
    "P383_Bounce",
    "P377_Start",
    "P377_Delay",
    "P377_Done",
    "P377_Crush",
    "P377_Toppling",
    "P377_Angle1",
    "P377_Angle2",
];
const OCL_NAMES: [&str; 3] = ["P377_O0", "P377_O1", "P377_O2"];

/// The generator's `SlopeTerrain`.
struct SlopeTerrain;

impl ThingTerrainAccess for SlopeTerrain {
    fn get_ground_height(&self, x: f32, y: f32, normal: Option<&mut Coord3D>) -> f32 {
        if let Some(n) = normal {
            *n = Coord3D::new(0.0, 0.0, 1.0);
        }
        x * 0.01 + y * 0.02
    }

    fn align_on_terrain(
        &self,
        angle: f32,
        pos: &Coord3D,
        stick_to_ground: Bool,
        mtx: &mut Matrix3D,
    ) -> PathfindLayerEnum {
        let mut pos = *pos;
        if stick_to_ground != FALSE {
            pos.z = self.get_ground_height(pos.x, pos.y, None);
        }
        zh_logic::object::thing::make_align_to_normal_matrix(
            angle,
            &pos,
            &Coord3D::new(0.0, 0.0, 1.0),
            mtx,
        );
        PathfindLayerEnum::Ground
    }

    fn is_underwater(
        &self,
        _x: f32,
        _y: f32,
        _water_z: Option<&mut f32>,
        _terrain_z: Option<&mut f32>,
    ) -> Bool {
        FALSE
    }
}

/// Loads the generator's INI text, block by block.
fn rules(ini: &str) -> RulesRegistry {
    let mut rules = RulesRegistry::new();
    let mut objects = String::new();
    let mut block = String::new();
    for line in ini.lines() {
        block.push_str(line);
        block.push('\n');
        if line != "End" {
            continue;
        }
        if block.starts_with("Upgrade ") {
            let mut r = IniReader::new("upgrade.ini", block.as_bytes(), IniLoadType::Overwrite);
            r.load_blocks(None, |_, reader| {
                let _ = reader.next_token(SEPS_DEFAULT)?;
                rules
                    .upgrade_center
                    .parse_upgrade_definition(reader, &mut rules.name_keys)
            })
            .expect("upgrade");
        } else if block.starts_with("Weapon ") {
            let mut weapon = WeaponTemplate::new();
            weapon.parse_ini_block(&block).expect("weapon");
            rules.weapon_store.add_template(weapon);
        } else {
            objects.push_str(&block);
        }
        block.clear();
    }
    let mut r = IniReader::new("object.ini", objects.as_bytes(), IniLoadType::Overwrite);
    if let Err(e) = rules.load_thing_templates_from_ini(&mut r) {
        panic!("fixture line {}: {e:?}", r.get_line_num());
    }
    rules
}

fn new_world(frame: u32, seed: u32) -> SimWorld {
    let mut world = SimWorld::new();
    world.frame = frame;
    world.rng.init_random_with_seed(seed);
    for (player, teams) in PLAYER_TEAMS.iter().enumerate() {
        world.player_default_teams.insert(player as i32, teams[0]);
        for &team in *teams {
            world.team_players.insert(team, player as i32);
            world.team_is_neutral.insert(team, Bool::from(player == 0));
        }
    }
    world.neutral_team_id = TEAM_NEUTRAL;
    world.local_player_index = 1;
    for (a, b, relationship) in [
        (1, 2, Relationship::Enemies),
        (2, 1, Relationship::Enemies),
        (1, 0, Relationship::Neutral),
        (2, 0, Relationship::Neutral),
    ] {
        world.player_relationships.insert((a, b), relationship);
    }
    world
}

fn spawn(sim: &mut SimContext, name: &str, team: TeamId, pos: Coord3D, angle: f32) -> ObjectSlot {
    let tid = sim
        .rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"));
    let slot = sim.spawn_object(tid, team, Coord3D::default(), 0.0);
    sim.set_object_position(slot, &pos);
    sim.set_object_orientation(slot, angle);
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

fn next_call_frame(sim: &SimContext, slot: ModuleSlot) -> u32 {
    sim.module_schedule(slot)
        .expect("schedule")
        .next_call_frame()
}

/// "<save> <crc> <update return> <next call frame>"
fn head(sim: &mut SimContext, slot: ModuleSlot, ret: u32) -> String {
    let (mut payload, _ctx, generation) = sim.world.modules.take_payload(slot).expect("payload");
    let mut xs = XferSave::new();
    payload.xfer(&mut xs, sim).expect("xfer save");
    let mut xc = XferCrc::new();
    payload.crc(&mut xc);
    sim.world.modules.restore_payload(slot, generation, payload);
    let hex: String = xs.as_bytes().iter().map(|b| format!("{b:02x}")).collect();
    format!(
        "{hex} {} {ret} {}",
        xc.get_crc(),
        next_call_frame(sim, slot)
    )
}

fn update(sim: &mut SimContext, m: ModuleSlot) -> UpdateSleepTime {
    sim.dispatch_module(m, |b, s, c| b.as_update_mut().expect("update").update(s, c))
        .expect("dispatch")
}

/// The generator's `step`.
fn step(sim: &mut SimContext, m: ModuleSlot, frame: u32) {
    sim.world.frame = frame;
    if next_call_frame(sim, m) > frame {
        return;
    }
    let sleep = update(sim, m);
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

fn matrix_of(sim: &SimContext, o: ObjectSlot) -> String {
    let m = sim
        .world
        .objects
        .get(o)
        .expect("object")
        .thing
        .get_transform_matrix();
    m.rows
        .iter()
        .flatten()
        .map(|v| v.to_bits().to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn condition(sim: &SimContext, o: ObjectSlot, flag: ModelConditionFlagType) -> u8 {
    let obj = sim.world.objects.get(o).expect("object");
    u8::from(obj.model_condition.test(flag.as_i32()) != FALSE)
}

fn dead(sim: &SimContext, o: ObjectSlot) -> u8 {
    u8::from(
        sim.world
            .objects
            .get(o)
            .expect("object")
            .is_effectively_dead()
            != FALSE,
    )
}

fn joined(items: Vec<String>) -> String {
    if items.is_empty() {
        "-".to_string()
    } else {
        items.concat()
    }
}

fn point(p: &Coord3D) -> String {
    format!("{},{},{}", p.x.to_bits(), p.y.to_bits(), p.z.to_bits())
}

fn take_fx(sim: &mut SimContext) -> String {
    joined(
        sim.world
            .triggered_fx_lists
            .drain(..)
            .map(|(name, pos)| {
                let list = FX_NAMES.iter().position(|&n| n == name).expect("FX name");
                format!("{list},{};", point(&pos))
            })
            .collect(),
    )
}

fn take_ocl(sim: &mut SimContext) -> String {
    joined(
        sim.world
            .spawned_ocls
            .drain(..)
            .map(|(name, _, pos, _)| {
                let list = OCL_NAMES.iter().position(|&n| n == name).expect("OCL name");
                format!("{list},{};", point(&pos))
            })
            .collect(),
    )
}

fn take_fires(sim: &mut SimContext) -> String {
    joined(
        sim.world
            .weapon_fires
            .drain(..)
            .map(|(_, name, _, pos)| {
                assert_eq!(name, "P377_Crush");
                format!("{};", point(&pos))
            })
            .collect(),
    )
}

fn between<'a>(line: &'a str, start: &str, end: &str) -> &'a str {
    let rest = &line[line.find(start).expect(start) + start.len()..];
    &rest[..rest.find(end).expect(end)]
}

/// One vector line.
struct Vector<'a> {
    line: &'a str,
    index: usize,
}

impl Vector<'_> {
    fn num(&self, key: &str) -> u32 {
        get_u32(self.line, key).unwrap_or_else(|| panic!("vector {} lacks {key}", self.index))
    }

    fn real(&self, key: &str) -> f32 {
        f32::from_bits(self.num(key))
    }

    fn pos(&self) -> Coord3D {
        Coord3D::new(self.real("x"), self.real("y"), self.real("z"))
    }

    fn ops(&self) -> Vec<[u32; 6]> {
        let numbers: Vec<u32> = between(self.line, "\"ops\":[", "],\"rows\"")
            .split(['[', ']', ','])
            .filter(|t| !t.is_empty())
            .map(|t| t.parse().expect("op number"))
            .collect();
        numbers
            .chunks_exact(6)
            .map(|c| c.try_into().expect("six"))
            .collect()
    }

    fn rows(&self) -> Vec<&str> {
        between(self.line, "\"rows\":[\"", "\"],\"client\"")
            .split("\",\"")
            .collect()
    }

    fn check(&self, what: &str, got: &str, want: &str) {
        assert!(
            got == want,
            "vector {} (kind {}, variant {}) {what}\n got  {got}\n want {want}",
            self.index,
            self.num("kind"),
            self.num("variant"),
        );
    }
}

fn topple_row(
    sim: &mut SimContext,
    o: ObjectSlot,
    m: ModuleSlot,
    ret: u32,
    stump_id: ObjectId,
) -> String {
    let head = head(sim, m, ret);
    let matrix = matrix_of(sim, o);
    let health = sim.object_health(o).to_bits();
    let flags = format!(
        "{}{}{}",
        dead(sim, o),
        condition(sim, o, ModelConditionFlagType::Toppled),
        condition(sim, o, ModelConditionFlagType::Burned)
    );
    let seed = sim.world.rng.get_game_logic_random_seed_crc();
    let fx = take_fx(sim);
    let stump = sim.find_object(stump_id).map_or_else(
        || "-".to_string(),
        |s| {
            format!(
                "{},{},{},{},{}",
                matrix_of(sim, s),
                sim.object_health(s).to_bits(),
                dead(sim, s),
                condition(sim, s, ModelConditionFlagType::Burned),
                u8::from(sim.world.objects.get(s).expect("stump").team_id == TEAM_NEUTRAL)
            )
        },
    );
    format!("{head} {matrix} {health} {flags} {seed} {fx} {stump}")
}

fn run_topple(sim: &mut SimContext, v: &Vector<'_>) -> usize {
    let pos = v.pos();
    let o = spawn(
        sim,
        &format!("P383_T{}", v.num("variant")),
        TEAM_A,
        pos,
        v.real("angle"),
    );
    let m = module_named(sim, o, "ToppleUpdate");
    let stump_id = ObjectId(sim.world.objects.get(o).expect("object").id.0 + 1);
    let mut crushers: [Option<ObjectSlot>; 8] = [None; 8];
    let spawn_row = topple_row(sim, o, m, 0, stump_id);
    v.check(
        "spawn",
        &spawn_row,
        get_str(v.line, "spawn").expect("spawn"),
    );

    let rows = v.rows();
    let ops = v.ops();
    assert_eq!(ops.len(), rows.len(), "vector {}", v.index);
    let mut frame = v.num("frame");
    for (k, (op, want)) in ops.iter().zip(&rows).enumerate() {
        let real = |i: usize| f32::from_bits(op[i]);
        let mut ret = 0;
        sim.world.frame = frame;
        match op[0] {
            0 => {
                let dir = Coord3D::new(real(1), real(2), real(3));
                sim.topple_object(o, &dir, real(4), op[5]);
            }
            1 => {
                for _ in 0..op[1] {
                    frame += 1;
                    step(sim, m, frame);
                }
            }
            2 => {
                let kind = op[1] as usize;
                let c = *crushers[kind].get_or_insert_with(|| {
                    let saved = sim.world.next_object_id;
                    sim.world.next_object_id = 100 + kind as i32;
                    let c = spawn(sim, &format!("P383_Crusher{kind}"), TEAM_B, pos, 0.0);
                    sim.world.next_object_id = saved;
                    c
                });
                let at = *sim
                    .world
                    .objects
                    .get(o)
                    .expect("object")
                    .thing
                    .get_position();
                let cpos = Coord3D::new(at.x + real(2), at.y + real(3), 3.0);
                sim.set_object_position(c, &cpos);
                if let Some(p) = physics_update::physics_mut(sim, c) {
                    p.vel = Coord3D::new(real(4), real(5), real(4) * 0.5);
                }
                let normal = Coord3D::new(0.0, 0.0, 1.0);
                sim.dispatch_on_collide(o, Some(c), Some(&cpos), Some(&normal));
            }
            3 => ret = update(sim, m).0,
            4 => {
                let here = *sim
                    .world
                    .objects
                    .get(o)
                    .expect("object")
                    .thing
                    .get_position();
                sim.dispatch_on_collide(o, None, Some(&here), Some(&here));
            }
            5 => sim.set_model_condition_state(o, ModelConditionFlagType::Burned),
            6 => {
                let id = sim.world.objects.get(o).expect("object").id;
                sim.world
                    .script_topple_directions
                    .insert(id, Coord3D::new(real(1), real(2), real(3)));
            }
            _ => kill_object(sim, o, DamageType::Unresistable, DeathType::Normal),
        }
        let got = topple_row(sim, o, m, ret, stump_id);
        v.check(&format!("op {k} {op:?}"), &got, want);
    }
    rows.len()
}

fn struct_row(sim: &mut SimContext, o: ObjectSlot, m: ModuleSlot, ret: u32) -> String {
    let head = head(sim, m, ret);
    let matrix = matrix_of(sim, o);
    let flags = format!(
        "{}{}{}",
        dead(sim, o),
        condition(sim, o, ModelConditionFlagType::Rubble),
        condition(sim, o, ModelConditionFlagType::PostCollapse)
    );
    let seed = sim.world.rng.get_game_logic_random_seed_crc();
    let fx = take_fx(sim);
    let ocl = take_ocl(sim);
    let fires = take_fires(sim);
    format!("{head} {matrix} {flags} {seed} {fx} {ocl} {fires}")
}

fn damage_info(
    sim: &mut SimContext,
    o: ObjectSlot,
    op: &[u32; 6],
    attacker: &mut Option<ObjectSlot>,
) -> DamageInfo {
    let mut info = DamageInfo::default();
    info.input.damage_type = DamageType::from_i32(op[1] as i32).expect("damage type");
    info.input.death_type = DeathType::from_i32((op[2] & 0xff) as i32).expect("death type");
    info.input.source_id = ObjectId::INVALID;
    if op[3] == 1 {
        info.input.source_id = ObjectId(9999);
    }
    if op[3] >= 2 {
        let mut p = *sim
            .world
            .objects
            .get(o)
            .expect("object")
            .thing
            .get_position();
        if op[3] == 3 {
            p.x += f32::from_bits(op[4]);
            p.y += f32::from_bits(op[5]);
        }
        let a = *attacker.get_or_insert_with(|| spawn(sim, "P377_Attacker", TEAM_B, p, 0.0));
        sim.set_object_position(a, &p);
        info.input.source_id = sim.world.objects.get(a).expect("attacker").id;
    }
    info
}

fn run_struct(sim: &mut SimContext, v: &Vector<'_>) -> usize {
    let o = spawn(
        sim,
        &format!("P377_S{}", v.num("variant")),
        TEAM_A,
        v.pos(),
        v.real("angle"),
    );
    let m = module_named(sim, o, "StructureToppleUpdate");
    let mut attacker = None;
    let spawn_row = struct_row(sim, o, m, 0);
    v.check(
        "spawn",
        &spawn_row,
        get_str(v.line, "spawn").expect("spawn"),
    );

    let rows = v.rows();
    let ops = v.ops();
    assert_eq!(ops.len(), rows.len(), "vector {}", v.index);
    let mut frame = v.num("frame");
    for (k, (op, want)) in ops.iter().zip(&rows).enumerate() {
        let mut ret = 0;
        sim.world.frame = frame;
        match op[0] {
            0 => {
                let mut info = damage_info(sim, o, op, &mut attacker);
                info.input.amount = 100_000.0;
                sim.attempt_damage(o, &mut info);
            }
            1 => {
                for _ in 0..op[1] {
                    frame += 1;
                    step(sim, m, frame);
                }
            }
            2 => {
                let mut info = damage_info(sim, o, op, &mut attacker);
                info.input.amount = ((op[2] >> 8) & 0xff) as f32;
                if info.input.damage_type == DamageType::Healing {
                    info.input.death_type = DeathType::NoneType;
                    info.input.source_id = ObjectId::INVALID;
                    sim.attempt_healing(o, &mut info);
                } else {
                    sim.attempt_damage(o, &mut info);
                }
                frame += op[2] >> 16;
            }
            3 => {
                let info = damage_info(sim, o, op, &mut attacker);
                sim.dispatch_module(m, |b, s, c| {
                    b.as_die_mut().expect("die").on_die(s, c, &info);
                })
                .expect("dispatch");
            }
            4 => {
                let id = sim.world.objects.get(o).expect("object").id;
                let f = |i: usize| f32::from_bits(op[i]);
                sim.world
                    .script_topple_directions
                    .insert(id, Coord3D::new(f(1), f(2), f(3)));
            }
            5 => ret = update(sim, m).0,
            _ => sim.set_status(o, ObjectStatusType::from_i32(op[1] as i32).expect("status")),
        }
        let got = struct_row(sim, o, m, ret);
        v.check(&format!("op {k} {op:?}"), &got, want);
    }
    rows.len()
}

fn deposit_row(sim: &mut SimContext, m: ModuleSlot, ret: u32) -> String {
    let head = head(sim, m, ret);
    // The port keeps money by team; a player's money is the sum over its teams.
    let money: Vec<String> = PLAYER_TEAMS
        .iter()
        .map(|teams| {
            teams
                .iter()
                .fold(0u32, |sum, &t| sum.wrapping_add(sim.player_money(t)))
                .to_string()
        })
        .collect();
    let score: Vec<String> = (0..3)
        .map(|p| {
            sim.world
                .score_keepers
                .get(&p)
                .map_or(0, |k| k.total_money_earned)
                .to_string()
        })
        .collect();
    let texts = joined(
        sim.world
            .floating_money_texts
            .drain(..)
            .map(|(_, pos, amount)| format!("{},{amount};", point(&pos)))
            .collect(),
    );
    let seed = sim.world.rng.get_game_logic_random_seed_crc();
    format!(
        "{head} {} {} {texts} {seed}",
        money.join(","),
        score.join(",")
    )
}

fn deposit_team(code: u32) -> TeamId {
    [TEAM_NEUTRAL, TEAM_A, TEAM_B, TEAM_A2][code as usize]
}

fn run_deposit(sim: &mut SimContext, v: &Vector<'_>) -> usize {
    for (player, key) in ["m0", "m1", "m2"].into_iter().enumerate() {
        sim.world
            .player_money
            .insert(PLAYER_TEAMS[player][0], v.num(key));
    }
    let o = spawn(
        sim,
        &format!("P305_D{}", v.num("variant")),
        deposit_team(v.num("team")),
        v.pos(),
        v.real("angle"),
    );
    let m = module_named(sim, o, "AutoDepositUpdate");
    let spawn_row = deposit_row(sim, m, 0);
    v.check(
        "spawn",
        &spawn_row,
        get_str(v.line, "spawn").expect("spawn"),
    );

    let rows = v.rows();
    let ops = v.ops();
    assert_eq!(ops.len(), rows.len(), "vector {}", v.index);
    let mut frame = v.num("frame");
    for (k, (op, want)) in ops.iter().zip(&rows).enumerate() {
        let mut ret = 0;
        sim.world.frame = frame;
        match op[0] {
            0 => {
                for _ in 0..op[1] {
                    frame += 1;
                    step(sim, m, frame);
                }
            }
            1 => sim.capture_object(o, deposit_team(op[1])),
            2 => {
                let name = if op[2] != 0 { "P305_U2" } else { "P305_U1" };
                let mask = sim
                    .rules
                    .upgrade_center
                    .find_upgrade(name, &mut sim.rules.name_keys)
                    .expect("upgrade")
                    .upgrade_mask()
                    .clone();
                let none = Default::default();
                // The port keeps upgrades by team: give every team of the player.
                for &team in PLAYER_TEAMS[op[1] as usize] {
                    let owned = sim.world.player_upgrades.entry(team).or_default();
                    if op[3] != 0 {
                        owned.clear_and_set(&none, &mask);
                    } else {
                        owned.clear_and_set(&mask, &none);
                    }
                }
            }
            3 => sim.set_object_construction_percent(o, f32::from_bits(op[1])),
            4 => {
                let status = ObjectStatusType::from_i32(op[1] as i32).expect("status");
                if op[2] != 0 {
                    sim.set_status(o, status);
                } else {
                    sim.clear_status(o, status);
                }
            }
            _ => ret = update(sim, m).0,
        }
        let got = deposit_row(sim, m, ret);
        v.check(&format!("op {k} {op:?}"), &got, want);
    }
    rows.len()
}

fn golden() -> &'static str {
    include_str!("../../../docs/golden_vectors/PKT383_377_305_Topple.json")
}

fn fixture_text() -> String {
    let line = golden()
        .lines()
        .find(|l| l.starts_with("\"ini\":\""))
        .expect("ini line");
    line["\"ini\":\"".len()..line.rfind('"').expect("quote")].replace("\\n", "\n")
}

fn context() -> SimContext {
    let mut sim = SimContext::new();
    sim.rules = rules(&fixture_text());
    sim.terrain = Box::new(SlopeTerrain);
    sim
}

#[test]
fn topple_and_deposit_modules_match_reference() {
    let lines: Vec<&str> = golden()
        .lines()
        .map(|l| l.trim_end_matches(','))
        .filter(|l| l.starts_with("{\"kind\""))
        .collect();
    assert_eq!(lines.len(), 1800);
    let mut sim = context();
    let mut checked = [0usize; 3];
    for (index, &line) in lines.iter().enumerate() {
        let v = Vector { line, index };
        // The cached upgrade template lives as long as the reference process.
        let cache = sim.world.auto_deposit_boost_upgrade.take();
        sim.world = new_world(v.num("frame"), v.num("seed"));
        sim.world.auto_deposit_boost_upgrade = cache;
        let kind = v.num("kind");
        checked[kind as usize] += match kind {
            0 => run_topple(&mut sim, &v),
            1 => run_struct(&mut sim, &v),
            _ => run_deposit(&mut sim, &v),
        };
        let client = sim.rng_client_int(0, 1_000_000);
        v.check(
            "client seed",
            &client.to_string(),
            &v.num("client").to_string(),
        );
    }
    assert!(
        checked.iter().all(|&n| n > 4000),
        "too few rows: {checked:?}"
    );
}

// ---------------------------------------------------------------------------------------------
// Behaviour the vectors cannot observe, without a reference
// ---------------------------------------------------------------------------------------------

/// The first template of the given family whose INI block holds `needle` and
/// not `without`.
fn variant_with(family: &str, needle: &str, without: &str) -> String {
    let text = fixture_text();
    let header = format!("Object {family}");
    let block = text
        .split("\nEnd\n")
        .find(|b| b.contains(&header) && b.contains(needle) && !b.contains(without))
        .unwrap_or_else(|| panic!("no {family} with {needle}"));
    let at = block.find(&header).expect("object");
    block[at + "Object ".len()..]
        .split_whitespace()
        .next()
        .expect("name")
        .to_string()
}

fn plain_world() -> SimContext {
    let mut sim = context();
    sim.world = new_world(100, 7);
    sim
}

#[test]
fn left_or_right_toppling_leaves_the_pathfind_map() {
    for (needle, removed) in [
        ("ToppleLeftOrRightOnly = Yes", true),
        ("ToppleLeftOrRightOnly = No", false),
    ] {
        let mut sim = plain_world();
        let name = variant_with("P383_T", needle, "KillWhenStartToppling");
        let o = spawn(&mut sim, &name, TEAM_A, Coord3D::new(50.0, 50.0, 0.0), 0.0);
        sim.pathfinder_add_object(o);
        let id = sim.world.objects.get(o).expect("object").id;
        sim.topple_object(o, &Coord3D::new(1.0, 1.0, 0.0), 1.0, 0);
        assert_eq!(
            !sim.world.pathfind_map_objects.contains(&id),
            removed,
            "{needle}"
        );
    }
}

#[test]
fn a_collapsing_structure_is_deselected_and_stops_its_bone_fx() {
    let mut sim = plain_world();
    // Any variant whose die mux accepts a plain death.
    let o = (0..32)
        .find_map(|i| {
            let o = spawn(
                &mut sim,
                &format!("P377_S{i}"),
                TEAM_A,
                Coord3D::new(500.0, 500.0, 0.0),
                0.0,
            );
            sim.world.deselected_objects.clear();
            kill_object(&mut sim, o, DamageType::Unresistable, DeathType::Normal);
            (!sim.world.deselected_objects.is_empty()).then_some(o)
        })
        .expect("a variant that topples");
    let id = sim.world.objects.get(o).expect("object").id;
    assert_eq!(sim.world.deselected_objects, vec![(id, 0xffff, TRUE)]);
    // Without a `BoneFXUpdate` on the object nothing is stopped.
    sim.stop_all_bone_fx(o);
    assert!(sim.world.bone_fx_stops.is_empty());
}

#[test]
fn a_contained_or_unobserved_deposit_follows_the_outer_object() {
    let mut sim = plain_world();
    let at = Coord3D::new(500.0, 500.0, 0.0);
    let inner = spawn(&mut sim, "P305_D1", TEAM_B, at, 0.0);
    let outer = spawn(&mut sim, "P305_D2", TEAM_B, at, 0.0);
    sim.set_status(outer, ObjectStatusType::Stealthed);
    assert_eq!(sim.object_is_logically_visible(inner), TRUE);
    assert_eq!(sim.object_is_logically_visible(outer), FALSE);
    sim.set_contained_by(inner, Some(outer));
    assert_eq!(sim.object_is_logically_visible(inner), FALSE);
    // An observer who is out of the game sees everything.
    sim.world.player_inactive.insert(TEAM_A);
    assert_eq!(sim.object_is_logically_visible(inner), TRUE);
}
