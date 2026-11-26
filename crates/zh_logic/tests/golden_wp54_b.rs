//! Golden test for WP-5.4-B: `DynamicGeometryInfoUpdate` (PKT-322) and
//! `ProjectileStreamUpdate` (PKT-355).
//!
//! Oracle: `tests/cpp_harness/gen_wp54_b.cpp` (runs the reference module code).
//! Vectors: `docs/golden_vectors/WP54_B.json` (150 per module).
//!
//! Stand-ins: the harness `step_single_update` replaces the sleepy-update loop;
//! `add_projectile`, `set_position` and `get_all_points` are called directly, as
//! their weapon and draw callers are not ported.
//!
//! Not covered: disabled owners; loading a save (including stream version 1 and a
//! non-zero `m_direction`); partition re-registration and the drawable reaction in
//! `Object::setGeometryInfo`; a stream whose source differs between calls; owners
//! other than the one source object; non-finite or negative geometry values;
//! durations beyond the few hundred milliseconds used by the fixture.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use zh_common::golden_json::{decode_hex_bytes, encode_hex_bytes, extract_vector_lines};
use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{Int, ObjectId, Real, TeamId, TRUE};
use zh_common::xfer::{XferCrc, XferSave};
use zh_logic::arena::{ModuleContext, ModuleSlot, ObjectSlot};
use zh_logic::modules::update::projectile_stream_update::ProjectileStreamUpdate;
use zh_logic::rules::object_status_types::ObjectStatusMaskType;
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimView, SimWorld};

const TEAM_A: TeamId = TeamId(2);

fn spawn(sim: &mut SimContext, name: &str) -> ObjectSlot {
    let tid = sim
        .rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"));
    sim.spawn_object_with_status(
        tid,
        TEAM_A,
        Coord3D::default(),
        0.0,
        ObjectStatusMaskType::new(),
    )
}

fn reset_world(sim: &mut SimContext, frame: u32, seed: u32) {
    sim.world = SimWorld::new();
    let w = &mut sim.world;
    let neutral = TeamId(1);
    for (player, team) in [neutral, TEAM_A].into_iter().enumerate() {
        w.team_players.insert(team, player as Int);
        w.player_default_teams.insert(player as Int, team);
    }
    w.team_is_neutral.insert(neutral, TRUE);
    w.neutral_team_id = neutral;
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

fn step(sim: &mut SimContext, slot: ModuleSlot, owner: ObjectSlot, frame: u32) {
    sim.world.frame = frame;
    if sim.object(owner).is_none_or(|o| o.is_destroyed()) {
        return;
    }
    if next_call_frame(sim, slot) > frame {
        return;
    }
    let sleep = sim
        .dispatch_module(slot, |m, s, c| {
            let sleep = m.as_update_mut().expect("update facet").update(s, c);
            sleep.0.max(1)
        })
        .expect("dispatch update");
    let next = frame.saturating_add(sleep).min(0x3fff_ffff);
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

fn next_call_frame(sim: &SimContext, slot: ModuleSlot) -> u32 {
    sim.module_schedule(slot)
        .expect("schedule")
        .next_call_frame()
}

fn save_hex(sim: &mut SimContext, slot: ModuleSlot) -> String {
    let (mut payload, _ctx, generation) =
        sim.world.modules.take_payload(slot).expect("take payload");
    let mut xs = XferSave::new();
    payload.xfer(&mut xs, sim).expect("xfer save");
    sim.world.modules.restore_payload(slot, generation, payload);
    encode_hex_bytes(xs.as_bytes())
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

fn with_stream<R>(
    sim: &mut SimContext,
    slot: ModuleSlot,
    f: impl FnOnce(&mut ProjectileStreamUpdate, &mut dyn SimApi, ModuleContext) -> R,
) -> R {
    sim.dispatch_module(slot, |m, s, c| {
        let typed = m
            .as_any_mut()
            .and_then(|a| a.downcast_mut::<ProjectileStreamUpdate>())
            .expect("module type");
        f(typed, s, c)
    })
    .expect("dispatch module")
}

/// Mirrors the harness: a save identical to the previous row is written as "=".
fn compact(previous: &mut String, hex: String) -> String {
    if *previous == hex {
        return "=".to_string();
    }
    previous.clone_from(&hex);
    hex
}

struct Case {
    variant: u32,
    frame: u32,
    ops: Vec<[u32; 5]>,
}

/// Rows of the vector and the module CRC.
fn run_dynamic(sim: &mut SimContext, case: &Case) -> (Vec<String>, u32) {
    let owner = spawn(sim, &format!("P54B_Dyn_{}", case.variant));
    let m = module_named(sim, owner, "DynamicGeometryInfoUpdate");
    let mut rows = Vec::new();
    let mut frame = case.frame;
    let mut last = String::new();
    for i in 0..=case.ops.len() {
        if i > 0 {
            frame += 1;
            step(sim, m, owner, frame);
        }
        let mut row = compact(&mut last, save_hex(sim, m));
        let next = next_call_frame(sim, m);
        let obj = sim.object(owner).expect("owner");
        let g = &obj.geometry;
        write!(
            row,
            ",{},{next},{},{},{},{},{},{},{}",
            u32::from(obj.is_destroyed()),
            g.get_geom_type() as u32,
            u32::from(g.get_is_small() == TRUE),
            g.get_max_height_above_position().to_bits(),
            g.get_major_radius().to_bits(),
            g.get_minor_radius().to_bits(),
            g.get_bounding_circle_radius().to_bits(),
            g.get_bounding_sphere_radius().to_bits(),
        )
        .unwrap();
        rows.push(row);
    }
    (rows, module_crc(sim, m))
}

fn id_of(sim: &SimContext, slot: ObjectSlot) -> ObjectId {
    sim.object(slot).expect("object").id
}

fn destroy_id(sim: &mut SimContext, id: ObjectId) {
    if let Some(slot) = sim.find_object(id) {
        if !sim.object(slot).expect("object").is_destroyed() {
            sim.destroy_object(slot);
        }
    }
}

fn coord(op: &[u32; 5]) -> Coord3D {
    Coord3D::new(
        Real::from_bits(op[1]),
        Real::from_bits(op[2]),
        Real::from_bits(op[3]),
    )
}

fn run_stream(sim: &mut SimContext, case: &Case) -> (Vec<String>, u32) {
    let owner = spawn(sim, "P54B_Stream");
    let source = spawn(sim, &format!("P54B_Source_{}", case.variant));
    sim.set_object_position(source, &Coord3D::new(100.0, 200.0, 10.0));
    let victims = [spawn(sim, "P54B_Thing"), spawn(sim, "P54B_Thing")];
    let source_id = id_of(sim, source);
    let victim_ids = victims.map(|slot| id_of(sim, slot));
    let m = module_named(sim, owner, "ProjectileStreamUpdate");
    let targets = [
        Coord3D::new(0.0, 0.0, 0.0),
        Coord3D::new(5.0, 6.0, 7.0),
        Coord3D::new(5.0, 6.0, 7.5),
    ];
    let mut projectiles = Vec::new();
    let mut points = String::new();
    let mut pos = Coord3D::default();
    let mut destroyed = false;
    let mut rows = Vec::new();
    let mut frame = case.frame;
    let mut last = String::new();
    let mut record = |sim: &mut SimContext, points: &mut String| {
        if let Some(obj) = sim.object(owner) {
            pos = *obj.thing.get_position();
            destroyed = obj.is_destroyed();
        }
        let mut row = compact(&mut last, save_hex(sim, m));
        write!(
            row,
            ",{},{},{},{},{}{points}",
            u32::from(destroyed),
            next_call_frame(sim, m),
            pos.x.to_bits(),
            pos.y.to_bits(),
            pos.z.to_bits(),
        )
        .unwrap();
        points.clear();
        rows.push(row);
    };
    record(sim, &mut points);
    for op in &case.ops {
        match op[0] {
            0 => {
                let p = spawn(sim, "P54B_Thing");
                sim.set_object_position(p, &coord(op));
                let id = id_of(sim, p);
                projectiles.push(id);
                let d = op[4] as usize;
                let victim = if d < 2 {
                    victim_ids[d]
                } else {
                    ObjectId::INVALID
                };
                let place = (2..5).contains(&d).then(|| &targets[d - 2]);
                with_stream(sim, m, |s, _, _| {
                    s.add_projectile(source_id, id, victim, place);
                });
            }
            1 => {
                if !projectiles.is_empty() {
                    destroy_id(sim, projectiles[op[1] as usize % projectiles.len()]);
                }
            }
            4 => destroy_id(sim, source_id),
            7 => destroy_id(sim, victim_ids[op[1] as usize % 2]),
            2 => {
                if sim.object(owner).is_some_and(|o| !o.is_destroyed()) {
                    sim.process_destroy_list();
                }
            }
            3 => {
                frame += 1;
                step(sim, m, owner, frame);
            }
            5 => {
                let all = sim
                    .world
                    .modules
                    .payload(m)
                    .and_then(|p| p.as_any())
                    .and_then(|a| a.downcast_ref::<ProjectileStreamUpdate>())
                    .expect("module type")
                    .get_all_points(sim);
                write!(points, ",{}", all.len()).unwrap();
                for p in all {
                    write!(
                        points,
                        ",{},{},{}",
                        p.x.to_bits(),
                        p.y.to_bits(),
                        p.z.to_bits()
                    )
                    .unwrap();
                }
            }
            6 => {
                // `set_position` takes the three values from the second field on.
                let at = coord(op);
                with_stream(sim, m, |s, api, c| s.set_position(api, c, &at));
            }
            other => panic!("unknown op {other}"),
        }
        record(sim, &mut points);
    }
    let crc = module_crc(sim, m);
    (rows, crc)
}

#[test]
fn wp54_b_matches_reference() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/golden_vectors/WP54_B.json");
    let doc = fs::read_to_string(path).expect("read WP54_B.json");
    let ini_hex = doc
        .lines()
        .find_map(|l| get_str(l, "ini_hex"))
        .expect("ini_hex");
    let text = decode_hex_bytes(ini_hex).expect("ini bytes");
    let mut sim = SimContext::new();
    sim.rules = RulesRegistry::new();
    // The leading PlayerTemplate block is for the harness only.
    let start = text
        .windows(7)
        .position(|w| w == b"Object ")
        .expect("first object");
    let mut ini = IniReader::new("object.ini", &text[start..], IniLoadType::Overwrite);
    if let Err(e) = sim.rules.load_thing_templates_from_ini(&mut ini) {
        panic!("load thing templates at line {}: {e:?}", ini.get_line_num());
    }

    let lines = extract_vector_lines(&doc);
    assert_eq!(lines.len(), 300, "vector count");
    let mut failures = [0_usize; 2];
    let mut first = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let num = |key: &str| get_u32(line, key).unwrap_or_else(|| panic!("{key}"));
        let kind = num("k") as usize;
        let case = Case {
            variant: num("v"),
            frame: num("frame"),
            ops: get_str(line, "ops")
                .expect("ops")
                .split_terminator(';')
                .map(|op| {
                    let mut it = op.split(' ').map(|n| n.parse().expect("op"));
                    std::array::from_fn(|_| it.next().expect("op field"))
                })
                .collect(),
        };
        reset_world(&mut sim, case.frame, num("seed"));
        let (rows, crc) = if kind == 0 {
            run_dynamic(&mut sim, &case)
        } else {
            run_stream(&mut sim, &case)
        };
        let want: Vec<&str> = get_str(line, "out")
            .expect("out")
            .split_terminator(';')
            .collect();
        let mut why = rows
            .iter()
            .zip(&want)
            .position(|(got, want)| got != want)
            .map(|i| format!("row {i}\n  got  {}\n  want {}", rows[i], want[i]));
        if why.is_none() && rows.len() != want.len() {
            why = Some(format!("row count {} != {}", rows.len(), want.len()));
        }
        if why.is_none() && crc != num("crc") {
            why = Some(format!("crc {crc} != {}", num("crc")));
        }
        let rng = sim.world.rng.get_game_logic_random_seed_crc();
        if why.is_none() && rng != num("rng") {
            why = Some(format!("rng {rng} != {}", num("rng")));
        }
        if let Some(why) = why {
            failures[kind] += 1;
            if first.len() < 3 {
                first.push(format!("vector {index} kind {kind}: {why}"));
            }
        }
    }
    assert!(
        failures == [0, 0],
        "failures {failures:?}\n{}",
        first.join("\n")
    );
}
