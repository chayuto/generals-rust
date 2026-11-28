//! `PKT-330` `FlammableUpdate` and `PKT-327` `FireSpreadUpdate` against
//! `docs/golden_vectors/PKT330_327_Fire.json`.
//!
//! Recorded by `tests/cpp_harness/gen_pkt330_327_fire.cpp`, which lists what is left out. The
//! operations below mirror `apply` there, and the fixture INI is the one stored in the JSON.
//! Set `FIRE_DUMP=<index>` to print the section text of a vector.

use zh_common::golden_json::{extract_vector_lines, get_i32_vec, get_str, get_u32};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::{Int, ObjectId, Relationship, TeamId, FALSE};
use zh_common::xfer::{XferCrc, XferLoad, XferSave};
use zh_logic::arena::{ModuleSlot, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::modules::contain::open_contain::kill_object;
use zh_logic::modules::module::BehaviorModule;
use zh_logic::modules::update::flammable_update::FlammableUpdate;
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::model_state::ModelConditionFlagType;
use zh_logic::rules::object_status_types::{ObjectStatusMaskType, ObjectStatusType};
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimView, SimWorld};

const JSON: &str = include_str!("../../../docs/golden_vectors/PKT330_327_Fire.json");
const SECTIONS: [&str; 4] = ["modules", "objects", "ocl", "rng"];
const OCL_NAMES: [&str; 2] = ["P330_Embers", "P330_Embers2"];

/// The value of a top-level string field, with its JSON escapes undone.
fn text_field(key: &str) -> String {
    let prefix = format!("\"{key}\": \"");
    let line = JSON
        .lines()
        .find(|l| l.starts_with(&prefix))
        .unwrap_or_else(|| panic!("missing {key}"));
    let body = line[prefix.len()..]
        .strip_suffix("\",")
        .expect("string line");
    let mut out = String::new();
    let mut chars = body.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next().expect("escape") {
            'n' => out.push('\n'),
            't' => out.push('\t'),
            other => out.push(other),
        }
    }
    out
}

fn team_ids() -> [TeamId; 2] {
    let line = JSON
        .lines()
        .find(|l| l.starts_with("\"team_ids\""))
        .map(|l| format!("{{{l}}}"))
        .expect("team_ids");
    let ids = get_i32_vec(&line, "team_ids").expect("team_ids");
    [TeamId(ids[0] as _), TeamId(ids[1] as _)]
}

fn build_rules() -> RulesRegistry {
    let mut rules = RulesRegistry::new();
    for name in OCL_NAMES {
        rules.ocl_names.insert(name.to_owned());
    }
    let armor = text_field("armor_ini");
    let mut ini = IniReader::new("armor.ini", armor.as_bytes(), IniLoadType::Overwrite);
    rules
        .armor_store
        .load_from_ini(&mut ini, &mut rules.name_keys)
        .expect("load Armor");
    rules.global_data.unit_damaged_thresh = 0.7;
    rules.global_data.unit_really_damaged_thresh = 0.35;
    let objects = text_field("object_ini");
    let mut ini = IniReader::new("object.ini", objects.as_bytes(), IniLoadType::Overwrite);
    if let Err(e) = rules.load_thing_templates_from_ini(&mut ini) {
        panic!("load thing templates at line {}: {e:?}", ini.get_line_num());
    }
    rules
}

fn reset_world(sim: &mut SimContext, teams: [TeamId; 2]) {
    sim.world = SimWorld::new();
    sim.init_partition(
        20.0,
        Region3D {
            lo: Coord3D::new(0.0, 0.0, 0.0),
            hi: Coord3D::new(1000.0, 1000.0, 100.0),
        },
    );
    let w = &mut sim.world;
    for (player, team) in teams.into_iter().enumerate() {
        let player = player as Int + 1;
        w.team_players.insert(team, player);
        w.player_default_teams.insert(player, team);
    }
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
}

struct Scene {
    slots: Vec<ObjectSlot>,
    ids: Vec<ObjectId>,
    /// Cleared when the harness nulls its pointer.
    live: Vec<bool>,
}

impl Scene {
    fn get(&self, i: u32) -> Option<ObjectSlot> {
        self.live[i as usize].then(|| self.slots[i as usize])
    }
}

fn module_slots(sim: &SimContext, owner: ObjectSlot) -> Vec<(ModuleSlot, &'static str)> {
    let key = |name: &str| sim.rules.name_keys.lookup_key(name);
    let (flame, spread) = (key("FlammableUpdate"), key("FireSpreadUpdate"));
    sim.object(owner)
        .expect("object")
        .behaviors
        .iter()
        .filter_map(|&(ms, _)| {
            let name = Some(sim.world.modules.payload(ms)?.module_name_key());
            if name == flame {
                Some((ms, "F"))
            } else if name == spread {
                Some((ms, "S"))
            } else {
                None
            }
        })
        .collect()
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
    xs.into_bytes()
}

fn flammable(sim: &SimContext, slot: ModuleSlot) -> &FlammableUpdate {
    sim.world
        .modules
        .payload(slot)
        .and_then(|m| m.as_any())
        .and_then(|a| a.downcast_ref::<FlammableUpdate>())
        .expect("FlammableUpdate")
}

/// The reference saves version 1; the default build writes version 2, which appends the
/// flame source. The source is compared next to the bytes either way.
fn flammable_text(sim: &mut SimContext, slot: ModuleSlot) -> Result<String, String> {
    let mut bytes = save_module(sim, slot);
    let (source, would) = {
        let f = flammable(sim, slot);
        let would = f.status == 0;
        (f.flame_source.0, u8::from(would))
    };
    if cfg!(not(feature = "retail-xfer-save")) {
        let tail = bytes.split_off(bytes.len() - 4);
        if bytes[0] != 2 || tail != source.to_le_bytes() {
            return Err(format!("version 2 save {} + {}", hex(&bytes), hex(&tail)));
        }
        bytes[0] = 1;
    }
    check_reload(sim, slot, &bytes)?;
    Ok(format!("F{} {source} {would} ", hex(&bytes)))
}

/// Loads the reference layout into a fresh module and compares what it holds.
fn check_reload(sim: &SimContext, slot: ModuleSlot, bytes: &[u8]) -> Result<(), String> {
    let live = flammable(sim, slot);
    let mut loaded = FlammableUpdate::new(live.data.clone());
    let mut xl = XferLoad::new(bytes);
    loaded
        .xfer(&mut xl, sim)
        .map_err(|e| format!("load: {e:?}"))?;
    let fields = |f: &FlammableUpdate| {
        (
            f.status,
            f.aflame_end_frame,
            f.burned_end_frame,
            f.damage_end_frame,
            f.flame_damage_limit.to_bits(),
            f.last_flame_damage_dealt,
        )
    };
    if fields(&loaded) != fields(live) {
        return Err(format!(
            "reload {:?} != {:?}",
            fields(&loaded),
            fields(live)
        ));
    }
    Ok(())
}

fn observe(sim: &mut SimContext, s: &Scene) -> Result<[String; 4], String> {
    let (mut modules, mut objects, mut ocl) = (String::new(), String::new(), String::new());
    for i in 0..s.slots.len() {
        let Some(slot) = s.get(i as u32) else {
            modules.push_str("-;");
            objects.push_str("-;");
            continue;
        };
        for (ms, kind) in module_slots(sim, slot) {
            if kind == "F" {
                modules.push_str(&flammable_text(sim, ms)?);
            } else {
                let bytes = save_module(sim, ms);
                modules.push_str(&format!("S{} ", hex(&bytes)));
            }
        }
        modules.push(';');
        let health = sim.object_health(slot);
        let o = sim.object(slot).expect("object");
        let pos = o.thing.get_position();
        objects.push_str(&format!(
            "{} {} {} {} {} {} {} {} {} {};",
            u8::from(o.is_destroyed()),
            u8::from(o.is_effectively_dead() != FALSE),
            u8::from(o.status.test(ObjectStatusType::Aflame.as_i32()) != FALSE),
            u8::from(o.status.test(ObjectStatusType::Burned.as_i32()) != FALSE),
            u8::from(
                o.model_condition
                    .test(ModelConditionFlagType::Aflame.as_i32())
                    != FALSE
            ),
            u8::from(
                o.model_condition
                    .test(ModelConditionFlagType::Smoldering.as_i32())
                    != FALSE
            ),
            health.to_bits(),
            pos.x.to_bits(),
            pos.y.to_bits(),
            pos.z.to_bits(),
        ));
    }
    for (name, creator, pos, _) in sim.world.spawned_ocls.drain(..) {
        let list = OCL_NAMES.iter().position(|&n| n == name).expect("OCL name");
        ocl.push_str(&format!(
            "{list} {} {} {} {};",
            creator.0,
            pos.x.to_bits(),
            pos.y.to_bits(),
            pos.z.to_bits()
        ));
    }
    let rng = sim.world.rng.get_game_logic_random_seed_crc().to_string();
    Ok([modules, objects, ocl, rng])
}

fn damage(damage_type: DamageType, amount: f32, source: ObjectId) -> DamageInfo {
    let mut info = DamageInfo::default();
    info.input.damage_type = damage_type;
    info.input.death_type = DeathType::Normal;
    info.input.amount = amount;
    info.input.source_id = source;
    info
}

fn apply(sim: &mut SimContext, s: &mut Scene, op: [u32; 5]) {
    let [code, a, b, c, d] = op;
    let target = if code == 2 || code == 10 {
        None
    } else {
        s.get(a)
    };
    match (code, target) {
        (0, Some(o)) => {
            let types = [
                DamageType::Flame,
                DamageType::ParticleBeam,
                DamageType::Explosion,
                DamageType::Unresistable,
            ];
            let source = if d == 0 {
                ObjectId::INVALID
            } else {
                s.ids[d as usize - 1]
            };
            let mut info = damage(types[b as usize], f32::from_bits(c), source);
            sim.attempt_damage(o, &mut info);
        }
        (1, Some(o)) => {
            let mut info = damage(DamageType::Healing, f32::from_bits(b), ObjectId::INVALID);
            info.input.death_type = DeathType::NoneType;
            sim.attempt_healing(o, &mut info);
        }
        (2, _) => {
            for _ in 0..a {
                sim.world.frame = sim.world.frame.wrapping_add(1);
                sim.step_sleepy_updates();
                sim.update_partition();
                sim.process_destroy_list();
                for i in 0..s.slots.len() {
                    if s.live[i] && sim.find_object(s.ids[i]).is_none() {
                        s.live[i] = false;
                    }
                }
            }
        }
        (3, Some(o)) => sim.try_to_ignite(o),
        (4, Some(o)) => sim.start_fire_spreading(o),
        (5, Some(o)) => {
            let pos = Coord3D::new(f32::from_bits(b), f32::from_bits(c), f32::from_bits(d));
            sim.set_object_position(o, &pos);
            sim.update_partition();
        }
        (6, Some(o)) => sim.destroy_object(o),
        (7, Some(o)) => kill_object(sim, o, DamageType::Unresistable, DeathType::Normal),
        (8, Some(o)) => {
            let status = if b == 0 {
                ObjectStatusType::Aflame
            } else {
                ObjectStatusType::Burned
            };
            if c != 0 {
                sim.set_status(o, status);
            } else {
                sim.clear_status(o, status);
            }
        }
        (9, Some(o)) => {
            if b != 0 {
                sim.set_disabled(o, DisabledType::Held);
            } else {
                sim.clear_disabled(o, DisabledType::Held);
            }
        }
        (10, _) => sim.world.frame = sim.world.frame.wrapping_add(a),
        _ => {}
    }
}

fn fnv32(text: &str) -> u32 {
    text.bytes().fold(0x811c_9dc5, |h, b| {
        (h ^ u32::from(b)).wrapping_mul(0x0100_0193)
    })
}

fn run_vector(sim: &mut SimContext, line: &str, seed0: &[&str], dump: bool) -> Result<(), String> {
    let teams = team_ids();
    let num = |key: &str| get_u32(line, key).unwrap_or_else(|| panic!("missing {key}"));
    let array = |key: &str| -> Vec<u32> {
        let values = get_i32_vec(line, key).unwrap_or_else(|| panic!("missing {key}"));
        values.into_iter().map(|v| v as u32).collect()
    };
    let want_rows = get_str(line, "rows").expect("rows");

    reset_world(sim, teams);
    let mut s = Scene {
        slots: Vec::new(),
        ids: Vec::new(),
        live: Vec::new(),
    };
    for spawn in array("spawns").chunks(5) {
        let name = format!("P330_{}", spawn[0]);
        let template = sim
            .rules
            .thing_factory
            .find_template_id(&name)
            .unwrap_or_else(|| panic!("missing template {name}"));
        let slot = sim.spawn_object_with_status(
            template,
            teams[spawn[1] as usize - 1],
            Coord3D::default(),
            0.0,
            ObjectStatusMaskType::new(),
        );
        let pos = Coord3D::new(
            f32::from_bits(spawn[2]),
            f32::from_bits(spawn[3]),
            f32::from_bits(spawn[4]),
        );
        sim.set_object_position(slot, &pos);
        s.slots.push(slot);
        s.ids.push(sim.object(slot).expect("spawned").id);
        s.live.push(true);
    }
    sim.update_partition();
    sim.world.rng.init_random_with_seed(num("seed"));
    sim.world.frame = num("frame");
    sim.world.spawned_ocls.clear();

    let ops = array("ops");
    let steps: Vec<Option<&[u32]>> = std::iter::once(None)
        .chain(ops.chunks(5).map(Some))
        .collect();
    assert_eq!(want_rows.len(), steps.len() * 8 * SECTIONS.len());
    for (row, step) in steps.into_iter().enumerate() {
        if let Some(op) = step {
            apply(sim, &mut s, [op[0], op[1], op[2], op[3], op[4]]);
        }
        let got = observe(sim, &s).map_err(|e| format!("row {row}: {e}"))?;
        for (k, text) in got.iter().enumerate() {
            if dump {
                eprintln!("row {row}: {text}");
            }
            let at = (row * SECTIONS.len() + k) * 8;
            let want = u32::from_str_radix(&want_rows[at..at + 8], 16).expect("digest");
            let reference = seed0.get(row).and_then(|l| get_str(l, SECTIONS[k]));
            if fnv32(text) != want || reference.is_some_and(|r| r != text) {
                return Err(format!(
                    "row {row} ({:?}) section {}:\n got {text}\nwant {}",
                    step,
                    SECTIONS[k],
                    reference.unwrap_or("(digest only)")
                ));
            }
        }
    }

    if sim.world.rng.get_game_logic_random_seed_crc() != num("rng") {
        return Err("final rng".to_owned());
    }
    let mut crc: u32 = 0;
    for i in 0..s.slots.len() {
        let Some(slot) = s.get(i as u32) else {
            continue;
        };
        for (ms, _) in module_slots(sim, slot) {
            let mut xc = XferCrc::new();
            sim.world.modules.payload(ms).expect("payload").crc(&mut xc);
            crc = crc.wrapping_mul(31).wrapping_add(xc.get_crc());
        }
    }
    if crc != num("crc") {
        return Err(format!("final crc {crc}, want {}", num("crc")));
    }
    Ok(())
}

#[test]
fn pkt330_327_matches_reference() {
    let seed0: Vec<&str> = JSON
        .lines()
        .filter(|l| l.starts_with("{\"row\":"))
        .collect();
    let vectors = extract_vector_lines(JSON);
    assert_eq!(vectors.len(), 1000);
    let dump: Option<usize> = std::env::var("FIRE_DUMP").ok().and_then(|v| v.parse().ok());

    let mut sim = SimContext::new();
    sim.rules = build_rules();
    let mut failures = Vec::new();
    for (i, line) in vectors.iter().enumerate() {
        let reference: &[&str] = if i == 0 { &seed0 } else { &[] };
        if let Err(e) = run_vector(&mut sim, line, reference, dump == Some(i)) {
            failures.push(format!("vector {i}: {e}"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} vectors differ; first:\n{}",
        failures.len(),
        vectors.len(),
        failures[0]
    );
}
