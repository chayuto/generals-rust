//! `PKT-399` `Weapon`: the effects of a fired weapon against vectors from the reference
//! (`tests/cpp_harness/gen_pkt399_weapon_effects.cpp`, whose `state` and `run` this mirrors).
//!
//! Rows after the first 40 vectors are hashed in the reference file.
//! `WEAPON_DUMP=<vector>` prints the rows this port produces.

use std::fmt::Write as _;
use std::sync::mpsc::{channel, Sender};

use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord3D, Matrix3D, Region3D};
use zh_common::types::{Bool, ObjectId, Relationship, TeamId, VeterancyLevel, FALSE};
use zh_common::xfer::{XferCrc, XferSave};
use zh_logic::arena::{FacetMask, ModuleSlot, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::gamelogic::frame::{run_frame, NoFrameSeams};
use zh_logic::gamelogic::global_data::{WeaponBonusConditionType, WeaponBonusField};
use zh_logic::modules::die::special_power_completion_die::SpecialPowerCompletionDie;
use zh_logic::modules::update::battle_plan_update::BattlePlanBonuses;
use zh_logic::modules::update::projectile_stream_update::ProjectileStreamUpdate;
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::object::weapon::WeaponTemplate;
use zh_logic::object::weapon_set::WeaponSlotType;
use zh_logic::object::{weapon_effects, weapon_fire};
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::rules::kind_of::{KindOfMaskType, KindOfType};
use zh_logic::rules::model_state::ModelConditionFlagType;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::sim::{ClientSink, RulesRegistry, SimApi, SimContext, SimView, SimWorld};

const GOLDEN: &str = include_str!("../../../docs/golden_vectors/PKT399_WeaponEffects.json");

const TEAM_A: TeamId = TeamId(1);
const TEAM_B: TeamId = TeamId(2);
const TEAM_NEUTRAL: TeamId = TeamId(3);
const OCLS: [&str; 6] = ["E_O0", "E_O1", "E_O2", "E_O3", "E_O4", "E_O5"];
const ANGLES: [f32; 8] = [0.0, 0.5, 1.570_796_4, 2.5, 3.141_592_7, -1.0, -2.0, 4.5];

/// The generator's `StepTerrain`.
struct StepTerrain;

impl ThingTerrainAccess for StepTerrain {
    fn get_ground_height(&self, x: f32, y: f32, normal: Option<&mut Coord3D>) -> f32 {
        if let Some(n) = normal {
            *n = Coord3D::new(0.0, 0.0, 1.0);
        }
        ((x / 40.0) as i32 % 3) as f32 * 4.0 + ((y / 60.0) as i32 % 4) as f32 * 3.0
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

/// The text of a JSON string whose only escapes are `\n`, `\"` and `\\`.
fn text(line: &str, key: &str) -> String {
    get_str(line, key)
        .unwrap_or_else(|| panic!("missing {key}"))
        .replace("\\n", "\n")
}

fn header(key: &str) -> String {
    let start = format!("\"{key}\":");
    let line = GOLDEN
        .lines()
        .find(|l| l.starts_with(&start))
        .unwrap_or_else(|| panic!("missing {key}"));
    text(line, key)
}

fn rules() -> RulesRegistry {
    let mut rules = RulesRegistry::new();
    for name in OCLS {
        rules.ocl_names.insert(name.to_owned());
    }
    let armor = header("armor_ini");
    let mut ini = IniReader::new("armor.ini", armor.as_bytes(), IniLoadType::Overwrite);
    rules
        .armor_store
        .load_from_ini(&mut ini, &mut rules.name_keys)
        .expect("armor");
    let power = "SpecialPower E_Power\n  Enum = SPECIAL_DAISY_CUTTER\nEnd\n";
    let mut ini = IniReader::new("power.ini", power.as_bytes(), IniLoadType::Overwrite);
    ini.load_blocks(None, |_, reader| {
        let _kw = reader.next_token(zh_common::ini::SEPS_DEFAULT)?;
        rules.special_power_store.parse_special_power_definition(
            reader,
            &rules.science_store,
            &mut rules.name_keys,
        )
    })
    .expect("special power");

    let weapons = header("weapon_ini");
    let mut block = String::new();
    for line in weapons.lines() {
        block.push_str(line);
        block.push('\n');
        if line == "End" {
            let mut weapon = WeaponTemplate::new();
            weapon.parse_ini_block(&block).expect("weapon");
            rules.weapon_store.add_template(weapon);
            block.clear();
        }
    }
    rules.global_data.unit_damaged_thresh = 0.7;
    rules.global_data.unit_really_damaged_thresh = 0.35;
    rules.global_data.gravity = -1.0;
    let objects = header("object_ini");
    let mut ini = IniReader::new("object.ini", objects.as_bytes(), IniLoadType::Overwrite);
    if let Err(e) = rules.load_thing_templates_from_ini(&mut ini) {
        panic!("object ini line {}: {e:?}", ini.get_line_num());
    }
    rules
}

fn new_world(frame: u32, seed: u32) -> SimWorld {
    let mut world = SimWorld::new();
    world.frame = frame;
    world.rng.init_random_with_seed(seed);
    for (player, team) in [TEAM_NEUTRAL, TEAM_A, TEAM_B].into_iter().enumerate() {
        world.player_default_teams.insert(player as i32, team);
        world.team_players.insert(team, player as i32);
        world.team_is_neutral.insert(team, Bool::from(player == 0));
    }
    world.neutral_team_id = TEAM_NEUTRAL;
    world.local_player_index = 1;
    for (a, b, relationship) in [
        (1, 2, Relationship::Enemies),
        (2, 1, Relationship::Enemies),
        (1, 0, Relationship::Neutral),
        (0, 1, Relationship::Neutral),
        (2, 0, Relationship::Neutral),
        (0, 2, Relationship::Neutral),
    ] {
        world.player_relationships.insert((a, b), relationship);
    }
    world
}

/// The arrays of numbers of `key`: `[[..],..]`.
fn arrays(line: &str, key: &str) -> Vec<Vec<i64>> {
    let start = format!("\"{key}\":[[");
    let from = line.find(&start).unwrap_or_else(|| panic!("missing {key}")) + start.len();
    let to = from + line[from..].find("]]").expect("end of arrays");
    line[from..to]
        .split("],[")
        .map(|a| a.split(',').map(|n| n.parse().expect("number")).collect())
        .collect()
}

fn rows(line: &str) -> Vec<&str> {
    let start = line.find("\"rows\":[\"").expect("rows") + 9;
    let end = line.rfind("\"]}").expect("end of rows");
    line[start..end].split("\",\"").collect()
}

fn fnv(s: &str) -> u64 {
    s.bytes().fold(0xcbf2_9ce4_8422_2325, |h, c| {
        (h ^ u64::from(c)).wrapping_mul(0x100_0000_01b3)
    })
}

fn live_objects(sim: &SimContext) -> Vec<ObjectSlot> {
    (1..4000)
        .filter_map(|id| sim.world.objects.slot_by_id(ObjectId(id)))
        .collect()
}

/// `fnv(save hex).crc` of one module.
fn module_state(sim: &mut SimContext, slot: ModuleSlot) -> String {
    let (mut payload, _ctx, generation) = sim.world.modules.take_payload(slot).expect("payload");
    let mut save = XferSave::new();
    payload.xfer(&mut save, sim).expect("save");
    let mut crc = XferCrc::new();
    payload.crc(&mut crc);
    sim.world.modules.restore_payload(slot, generation, payload);
    let hex = save.as_bytes().iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    });
    format!("{}.{}", fnv(&hex), crc.get_crc())
}

fn module_where(
    sim: &SimContext,
    owner: ObjectSlot,
    test: impl Fn(&dyn std::any::Any) -> bool,
) -> Option<ModuleSlot> {
    sim.object(owner)?
        .behaviors
        .iter()
        .map(|&(ms, _)| ms)
        .find(|&ms| {
            sim.world
                .modules
                .payload(ms)
                .and_then(|m| m.as_any())
                .is_some_and(&test)
        })
}

fn state(sim: &mut SimContext, ret: u32) -> String {
    let mut out = format!("R{},{ret};", sim.world.rng.get_game_logic_random_seed_crc());
    for slot in live_objects(sim) {
        let health = sim.object_health(slot);
        let o = sim.object(slot).expect("object");
        let pos = *o.thing.get_position();
        let name = o
            .thing
            .template_override()
            .get_final(&sim.rules.thing_factory)
            .expect("template")
            .get_name()
            .to_string();
        let (body, physics, projectile) = (o.body_slot, o.physics_slot, o.projectile_slot);
        let stunned = o
            .model_condition
            .test(ModelConditionFlagType::StunnedFlailing.as_i32())
            != FALSE;
        let _ = write!(
            out,
            "O{},{name},{}{},{},{},{},{},{},{}",
            o.id.0,
            u8::from(o.is_destroyed()),
            u8::from(o.is_effectively_dead() != FALSE),
            pos.x.to_bits(),
            pos.y.to_bits(),
            pos.z.to_bits(),
            o.thing.get_orientation().to_bits(),
            health.to_bits(),
            o.experience_tracker.current_experience,
        );
        if let Some(ms) = body {
            let _ = write!(out, ",b{}", module_state(sim, ms));
        }
        if let Some(ms) = physics {
            let _ = write!(out, ",p{}", module_state(sim, ms));
        }
        if projectile.is_some() {
            let launcher = sim.projectile_launcher_id(slot).expect("launcher");
            let _ = write!(out, ",f{}", launcher.0);
        }
        if let Some(ms) = module_where(sim, slot, |a| a.is::<SpecialPowerCompletionDie>()) {
            let _ = write!(out, ",s{}", module_state(sim, ms));
        }
        if let Some(ms) = module_where(sim, slot, |a| a.is::<ProjectileStreamUpdate>()) {
            let _ = write!(out, ",t{}", module_state(sim, ms));
        }
        let _ = write!(out, ",{};", u8::from(stunned));
    }
    let store = &sim.world.weapon_store_state;
    for d in &store.delayed {
        let _ = write!(
            out,
            "D{},{},{},{},{},{},{},{},{};",
            d.weapon_name,
            d.pos.x.to_bits(),
            d.pos.y.to_bits(),
            d.pos.z.to_bits(),
            d.frame,
            d.source_id.0,
            d.victim_id.0,
            d.bonus.get_field(WeaponBonusField::Damage).to_bits(),
            d.bonus.get_field(WeaponBonusField::Radius).to_bits(),
        );
    }
    for i in 56..72 {
        if let Some(history) = store.history.get(&format!("E_W{i}")) {
            for h in &history.damage {
                let _ = write!(
                    out,
                    "H{i},{},{},{};",
                    h.frame,
                    h.location.x.to_bits(),
                    h.location.y.to_bits()
                );
            }
        }
    }
    for (name, creator, pos, _) in sim.world.spawned_ocls.drain(..) {
        let list = OCLS.iter().position(|&n| n == name).expect("OCL name");
        let _ = write!(
            out,
            "X{list},{},{},{},{};",
            creator.0,
            pos.x.to_bits(),
            pos.y.to_bits(),
            pos.z.to_bits()
        );
    }
    let mut creators: Vec<i32> = sim
        .world
        .special_power_completed_events
        .drain(..)
        .map(|(id, _)| id.0 as i32)
        .collect();
    creators.sort_unstable();
    for creator in creators {
        let _ = write!(out, "S{creator};");
    }
    out
}

fn spawn_units(sim: &mut SimContext, units: &[Vec<i64>]) {
    let mut spawned = Vec::new();
    for u in units {
        let name = format!("E_U{}", u[0]);
        let tid = sim
            .rules
            .thing_factory
            .find_template_id(&name)
            .unwrap_or_else(|| panic!("missing template {name}"));
        let team = [TEAM_A, TEAM_B, TEAM_NEUTRAL][u[1] as usize];
        let (x, y) = (u[2] as f32, u[3] as f32);
        let z = StepTerrain.get_ground_height(x, y, None) + u[4] as f32;
        let slot = sim.spawn_object(tid, team, Coord3D::new(x, y, z), ANGLES[u[5] as usize]);
        let o = sim.world.objects.get_mut(slot).expect("spawned");
        o.experience_tracker.current_level = match u[6] {
            0 => VeterancyLevel::Regular,
            1 => VeterancyLevel::Veteran,
            2 => VeterancyLevel::Elite,
            _ => VeterancyLevel::Heroic,
        };
        o.weapon_bonus_condition |= u[7] as u32;
        if u[4] > 0 {
            o.status.set(ObjectStatusType::AirborneTarget.as_i32(), 1);
        }
        spawned.push(slot);
    }
    for (u, &slot) in units.iter().zip(&spawned) {
        if u[8] >= 0 {
            let producer = sim.object(spawned[u[8] as usize]).expect("producer").id;
            sim.world
                .objects
                .get_mut(slot)
                .expect("spawned")
                .producer_id = producer;
        }
    }
}

fn step(sim: &mut SimContext, frame: &mut u32, frames: i64) {
    for _ in 0..frames {
        *frame += 1;
        sim.world.frame = *frame;
        for slot in live_objects(sim) {
            let Some(ms) = sim
                .object(slot)
                .filter(|o| !o.is_destroyed())
                .and_then(|o| o.projectile_slot)
            else {
                continue;
            };
            let _ = sim.dispatch_module(ms, |m, s, c| {
                m.as_update_mut().expect("update").update(s, c)
            });
        }
        // The order of `GameLogic::update`.
        sim.update_partition();
        sim.process_destroy_list();
        weapon_effects::update_delayed_damage(sim);
    }
}

fn run(sim: &mut SimContext, op: &[i64], frame: &mut u32) -> u32 {
    sim.world.frame = *frame;
    let code = op[0];
    if code == 4 {
        step(sim, frame, op[1]);
        return 0;
    }
    let live: Vec<ObjectSlot> = live_objects(sim)
        .into_iter()
        .filter(|&s| sim.object(s).is_some_and(|o| o.body_slot.is_some()))
        .collect();
    if live.is_empty() {
        return 0;
    }
    let src = live[op[if code >= 5 { 1 } else { 2 }] as usize % live.len()];
    if code <= 3 {
        let name = format!("E_W{}", op[1]);
        if code == 0 {
            let target = live[op[3] as usize % live.len()];
            if target != src {
                weapon_fire::create_and_fire_temp_weapon_at_object(sim, src, &name, target);
            }
            return 0;
        }
        let pos = Coord3D::new(op[3] as f32, op[4] as f32, op[5] as f32);
        if code == 1 {
            weapon_fire::create_and_fire_temp_weapon(sim, src, &name, &pos);
        } else if code == 2 {
            weapon_fire::handle_projectile_detonation(sim, src, &name, &pos, op[6] as u32);
        } else {
            let template = sim
                .rules
                .weapon_store
                .find_weapon_template(&name)
                .expect("weapon")
                .clone();
            let mut weapon = sim.rules.weapon_store.allocate_new_weapon(
                &template,
                WeaponSlotType::Primary,
                *frame,
            );
            weapon_fire::load_ammo_now(&mut weapon, sim, src);
            let shell = weapon_fire::force_fire_weapon(&mut weapon, sim, src, &pos);
            return shell
                .and_then(|s| sim.object(s))
                .map_or(0, |o| o.id.0 as u32);
        }
    } else if code == 5 {
        let (x, y) = (op[2] as f32, op[3] as f32);
        let pos = Coord3D::new(x, y, StepTerrain.get_ground_height(x, y, None));
        sim.set_object_position(src, &pos);
        sim.set_object_orientation(src, ANGLES[op[4] as usize]);
        sim.update_partition();
    } else {
        // `Object::kill`.
        let max = sim
            .object(src)
            .and_then(|o| o.body_slot)
            .and_then(|ms| sim.world.modules.payload(ms))
            .and_then(|m| m.as_body())
            .map_or(0.0, |b| b.get_max_health());
        let mut info = DamageInfo::default();
        info.input.damage_type = DamageType::Unresistable;
        info.input.death_type = DeathType::Normal;
        info.input.source_id = ObjectId::INVALID;
        info.input.amount = max;
        info.input.kill = 1;
        sim.attempt_damage(src, &mut info);
    }
    0
}

#[test]
fn weapon_effects_match_reference() {
    let lines: Vec<&str> = GOLDEN
        .lines()
        .map(|l| l.trim_end_matches(','))
        .filter(|l| l.starts_with("{\"vector\":"))
        .collect();
    assert_eq!(lines.len(), 1500);
    assert!(FacetMask::DIE != FacetMask::UPDATE);
    let dump: Option<u32> = std::env::var("WEAPON_DUMP")
        .ok()
        .and_then(|v| v.parse().ok());
    let mut sim = SimContext::new();
    sim.rules = rules();
    sim.terrain = Box::new(StepTerrain);
    let mut failed = Vec::new();
    for &line in &lines {
        let index = get_u32(line, "vector").expect("vector");
        let mut frame = get_u32(line, "frame").expect("frame");
        sim.world = new_world(frame, get_u32(line, "seed").expect("seed"));
        sim.init_partition(
            20.0,
            Region3D {
                lo: Coord3D::new(0.0, 0.0, 0.0),
                hi: Coord3D::new(1000.0, 1000.0, 100.0),
            },
        );
        let ops = arrays(line, "ops");
        let expected = rows(line);
        assert_eq!(expected.len(), ops.len() + 1, "vector {index}");
        spawn_units(&mut sim, &arrays(line, "units"));
        sim.update_partition();
        for (step_index, want) in expected.iter().enumerate() {
            let ret = match step_index.checked_sub(1) {
                Some(i) => run(&mut sim, &ops[i], &mut frame),
                None => 0,
            };
            let got = state(&mut sim, ret);
            if step_index > 0 {
                sim.process_destroy_list();
            }
            if dump == Some(index) {
                eprintln!("row {step_index}: {got}");
            }
            let same = match want.strip_prefix('#') {
                Some(hash) => fnv(&got).to_string() == hash,
                None => got == *want,
            };
            if !same {
                let op = step_index.checked_sub(1).map(|i| ops[i].clone());
                let first = want
                    .split(';')
                    .zip(got.split(';'))
                    .find(|(a, b)| a != b)
                    .map(|(a, b)| format!("want {a}\n got {b}"))
                    .unwrap_or_default();
                failed.push(format!(
                    "vector {index} row {step_index} op {op:?}\n{first}"
                ));
                break;
            }
        }
        if dump.is_some_and(|d| d == index) {
            break;
        }
    }
    assert!(
        failed.is_empty(),
        "{} vectors differ:\n{}",
        failed.len(),
        failed
            .iter()
            .take(6)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// `(laser, source, target, start, end)` of a beam the client was told about.
type Beam = (ObjectId, ObjectId, ObjectId, Coord3D, Coord3D);

struct Beams(Sender<Beam>);

impl ClientSink for Beams {
    fn on_laser_init(
        &mut self,
        laser: ObjectId,
        parent: ObjectId,
        target: ObjectId,
        start: Coord3D,
        end: Coord3D,
    ) {
        self.0
            .send((laser, parent, target, start, end))
            .expect("beam");
    }
}

/// A world with a launcher at (100, 100) and the extra weapons of the tests below.
fn small_world(sim: &mut SimContext, units: &[Vec<i64>]) -> Vec<ObjectSlot> {
    sim.rules = rules();
    for text in [
        "Weapon T_Laser\n  PrimaryDamage = 1\n  AttackRange = 1000\n  LaserName = E_Laser\nEnd\n",
        "Weapon T_Slow\n  PrimaryDamage = 7\n  AttackRange = 1000\n  WeaponSpeed = 90\nEnd\n",
    ] {
        let mut weapon = WeaponTemplate::new();
        weapon.parse_ini_block(text).expect("weapon");
        sim.rules.weapon_store.add_template(weapon);
    }
    sim.terrain = Box::new(StepTerrain);
    sim.world = new_world(100, 1);
    sim.init_partition(
        20.0,
        Region3D {
            lo: Coord3D::new(0.0, 0.0, 0.0),
            hi: Coord3D::new(1000.0, 1000.0, 100.0),
        },
    );
    spawn_units(sim, units);
    sim.update_partition();
    live_objects(sim)
}

/// `Weapon::createLaser`: the beam ends above a victim that is neither a projectile nor
/// airborne.
#[test]
fn laser_beam_end_points() {
    let (told, beams) = channel();
    let mut sim = SimContext::new();
    sim.client = Box::new(Beams(told));
    // Source, a ground vehicle, an aircraft, a projectile unit.
    let units = [
        vec![0, 0, 100, 100, 0, 0, 0, 0, -1],
        vec![0, 1, 130, 100, 0, 0, 0, 0, -1],
        vec![3, 1, 100, 130, 20, 0, 0, 0, -1],
        vec![7, 1, 70, 100, 0, 0, 0, 0, -1],
    ];
    let slots = small_world(&mut sim, &units);
    let source = slots[0];
    for (victim, lift) in [(slots[1], 10.0), (slots[2], 0.0), (slots[3], 0.0)] {
        let at = *sim.object(victim).expect("victim").thing.get_position();
        weapon_fire::create_and_fire_temp_weapon_at_object(&mut sim, source, "T_Laser", victim);
        let (_, parent, target, start, end) = beams.try_recv().expect("beam");
        assert_eq!(parent, sim.object(source).expect("source").id);
        assert_eq!(target, sim.object(victim).expect("victim").id);
        assert_eq!(
            start,
            *sim.object(source).expect("source").thing.get_position()
        );
        assert_eq!(end, Coord3D::new(at.x, at.y, at.z + lift));
    }
    let ground = Coord3D::new(140.0, 140.0, 3.0);
    weapon_fire::create_and_fire_temp_weapon(&mut sim, source, "T_Laser", &ground);
    let (_, _, target, _, end) = beams.try_recv().expect("beam");
    assert_eq!((target, end), (ObjectId::INVALID, ground));
}

/// The frame step delivers delayed damage when it is due, not before.
#[test]
fn frame_step_delivers_delayed_damage() {
    let mut sim = SimContext::new();
    let units = [
        vec![0, 0, 100, 100, 0, 0, 0, 0, -1],
        vec![5, 1, 400, 100, 0, 0, 0, 0, -1],
    ];
    let slots = small_world(&mut sim, &units);
    let full = sim.object_health(slots[1]);
    weapon_fire::create_and_fire_temp_weapon_at_object(&mut sim, slots[0], "T_Slow", slots[1]);
    // 300 units at 90 per second is 100 frames.
    let due = sim.world.weapon_store_state.delayed[0].frame;
    assert!(due > sim.world.frame + 1);
    while sim.world.frame < due {
        run_frame(&mut sim, &mut NoFrameSeams);
        assert_eq!(sim.object_health(slots[1]), full);
    }
    run_frame(&mut sim, &mut NoFrameSeams);
    assert_eq!(sim.object_health(slots[1]), full - 7.0);
    assert!(sim.world.weapon_store_state.delayed.is_empty());
}

/// A projectile gets its launcher's battle plan bonuses when it is fired. The launcher's
/// kind decides; C++ skips the projectile at creation because it has no producer yet.
#[test]
fn projectile_receives_battle_plan_bonus_at_launch() {
    let mut sim = SimContext::new();
    let units = [vec![0, 0, 100, 100, 0, 0, 0, 0, -1]];
    let slots = small_world(&mut sim, &units);
    let vehicle = KindOfType::Vehicle.as_i32();
    let mut valid = KindOfMaskType::new();
    valid.set(vehicle, 1);
    let plans = sim.player_battle_plans_mut(1);
    plans.bombardment_plans = 1;
    plans.bonuses = Some(BattlePlanBonuses {
        bombardment: 1,
        valid_kind_of: valid,
        ..BattlePlanBonuses::default()
    });
    let mut text = "Weapon T_Shell\n  PrimaryDamage = 1\n  AttackRange = 1000\n".to_owned();
    text.push_str("  WeaponSpeed = 10\n  ProjectileObject = E_Shell0\nEnd\n");
    let mut weapon = WeaponTemplate::new();
    weapon.parse_ini_block(&text).expect("weapon");
    sim.rules.weapon_store.add_template(weapon);

    let at = Coord3D::new(160.0, 100.0, 0.0);
    weapon_fire::create_and_fire_temp_weapon(&mut sim, slots[0], "T_Shell", &at);
    let shell = *live_objects(&sim).last().expect("shell");
    let flags = sim.object(shell).expect("shell").weapon_bonus_condition;
    let bit = WeaponBonusConditionType::BattleplanBombardment as u32;
    assert_ne!(flags & (1 << bit), 0, "{flags:x}");
}
