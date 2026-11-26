//! Golden test for WP-5.4-C: `SpecialPowerUpdateModule` (PKT-369),
//! `RadiusDecalUpdate` (PKT-359) and `HeightDieUpdate` (PKT-333).
//!
//! Oracle: `tests/cpp_harness/gen_wp54_c.cpp` (runs the reference module code).
//! Vectors: `docs/golden_vectors/WP54_C.json` (150 per module).
//!
//! Stand-ins: the harness `step_single_update` replaces the sleepy-update loop;
//! a minimal subclass drives the abstract special power base, here a bare
//! schedule tracker; `create_radius_decal`, `kill_radius_decal` and the
//! attack flag setter are called directly, as their callers are not ported;
//! `HeightTerrain` mirrors the harness slope and reproduces, for its one flat
//! bridge, the layer choice of `TerrainLogic::getHighestLayerForDestination`.
//!
//! Not covered: loading a save; the pure virtual special power interface; decal
//! drawing, visibility to the local player and decal templates (only an empty or
//! non-empty name is used); disabled owners; an owner without a player;
//! structures that the reference partition cell walk misses although their
//! bounding sphere is in range, where the port is known to differ (see the harness
//! header; all structures here start within 40 units of the owner and none of the
//! 150 vectors had to be discarded for this);
//! wall layers, several bridges and rubble bridges; contained or destroyed
//! structures other than a plain destroy; particle systems; death by a body other
//! than `ActiveBody`; a frame counter that wraps.

use std::fs;
use std::path::Path;
use zh_common::golden_json::{decode_hex_bytes, encode_hex_bytes, extract_vector_lines};
use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord3D, Matrix3D};
use zh_common::types::{Bool, Int, Real, TeamId, FALSE, TRUE};
use zh_common::xfer::{XferCrc, XferSave};
use zh_logic::arena::{ModuleContext, ModuleSlot, ObjectSlot};
use zh_logic::modules::behavior::behavior_module::UpdateScheduleTracker;
use zh_logic::modules::update::radius_decal_update::RadiusDecalUpdate;
use zh_logic::modules::update::special_power_update_module::{
    crc_special_power_update_module, does_special_power_update_pass_science_test,
    xfer_special_power_update_module,
};
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;
use zh_logic::rules::object_status_types::{ObjectStatusMaskType, ObjectStatusType};
use zh_logic::rules::science::{ScienceType, SCIENCE_INVALID};
use zh_logic::sim::{FlatTerrain, RulesRegistry, SimApi, SimContext, SimView, SimWorld};

const NEUTRAL: TeamId = TeamId(1);
const TEAM_A: TeamId = TeamId(2);

const KINDS: usize = 3;

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

/// Mirrors the harness: a save identical to the previous row is written as "=".
fn compact(previous: &mut String, hex: String) -> String {
    if *previous == hex {
        return "=".to_string();
    }
    previous.clone_from(&hex);
    hex
}

fn with_decal(
    sim: &mut SimContext,
    slot: ModuleSlot,
    f: impl FnOnce(&mut RadiusDecalUpdate, &mut dyn SimApi, ModuleContext),
) {
    sim.dispatch_module(slot, |m, s, c| {
        let typed = m
            .as_any_mut()
            .and_then(|a| a.downcast_mut::<RadiusDecalUpdate>())
            .expect("module type");
        f(typed, s, c);
    })
    .expect("dispatch module");
}

/// The harness's `HeightTerrain` plus the layer choice for its bridge.
struct HeightTerrain;

impl ThingTerrainAccess for HeightTerrain {
    fn get_ground_height(&self, x: Real, y: Real, normal: Option<&mut Coord3D>) -> Real {
        if let Some(n) = normal {
            *n = Coord3D::new(0.0, 0.0, 1.0);
        }
        (x - 500.0) * 0.125 + (y - 500.0) * 0.0625
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
        y: Real,
        water_z: Option<&mut Real>,
        terrain_z: Option<&mut Real>,
    ) -> Bool {
        FlatTerrain.is_underwater(x, y, water_z, terrain_z)
    }

    fn get_layer_height(
        &self,
        x: Real,
        _y: Real,
        _layer: PathfindLayerEnum,
        _normal: Option<&mut Coord3D>,
        _clip: Bool,
    ) -> Real {
        (x - 500.0) * 0.5 + 3.0
    }

    fn get_highest_layer_for_destination(
        &self,
        pos: &Coord3D,
        _only_healthy_bridges: Bool,
    ) -> PathfindLayerEnum {
        const DECK: Real = 10.0;
        let on_deck = (440.0..=560.0).contains(&pos.x) && (440.0..=560.0).contains(&pos.y);
        let to_ground = pos.z - self.get_ground_height(pos.x, pos.y, None);
        let delta = pos.z - DECK;
        if on_deck && delta >= 0.0 && delta.abs() < to_ground.abs() {
            PathfindLayerEnum::Bridge1
        } else {
            PathfindLayerEnum::Ground
        }
    }
}

fn run_die(sim: &mut SimContext, case: &Case) -> (Vec<String>, u32) {
    let owner = spawn(sim, &format!("P54C_Die_{}", case.variant), TEAM_A);
    let container = spawn(sim, "P54C_Plain", TEAM_A);
    let m = module_named(sim, owner, "HeightDieUpdate");
    let mut others: Vec<ObjectSlot> = Vec::new();
    let mut last = String::new();
    let mut rows = Vec::new();
    let mut record = |sim: &mut SimContext| {
        let hex = compact(&mut last, save_hex(sim, m));
        let health = sim.object_health(owner).to_bits();
        let next = next_call_frame(sim, m);
        let obj = sim.object(owner).expect("owner");
        let p = *obj.thing.get_position();
        rows.push(format!(
            "{hex},{},{},{health},{},{},{},{next}",
            u32::from(obj.is_destroyed()),
            u32::from(obj.is_effectively_dead() != FALSE),
            p.x.to_bits(),
            p.y.to_bits(),
            p.z.to_bits()
        ));
    };
    record(sim);
    let at = |a: u32, b: u32, c: u32| {
        Coord3D::new(Real::from_bits(a), Real::from_bits(b), Real::from_bits(c))
    };
    let mut frame = case.frame;
    for op in &case.ops {
        match op[0] {
            0 => {
                frame += op[1];
                step(sim, m, owner, frame);
            }
            1 => sim.set_object_position(owner, &at(op[1], op[2], op[3])),
            2 => sim.set_contained_by(owner, (op[1] != 0).then_some(container)),
            3 => sim.destroy_object(others[op[1] as usize]),
            4 => {
                let slot = spawn(sim, &format!("P54C_Structure_{}", op[1]), TEAM_A);
                sim.set_object_position(slot, &at(op[2], op[3], op[4]));
                others.push(slot);
            }
            other => panic!("unknown op {other}"),
        }
        record(sim);
    }
    let crc = module_crc(sim, m);
    (rows, crc)
}

struct Case {
    variant: u32,
    frame: u32,
    ops: Vec<[u32; 5]>,
}

/// The harness's science index; 0 is `SCIENCE_INVALID`.
fn science_of(index: u32) -> ScienceType {
    if index == 0 {
        SCIENCE_INVALID
    } else {
        100 + index as ScienceType
    }
}

fn run_special_power(sim: &mut SimContext, case: &Case) -> (Vec<String>, u32) {
    let teams = [TEAM_A, NEUTRAL];
    let owners = [
        spawn(sim, "P54C_Plain", teams[0]),
        spawn(sim, "P54C_Plain", teams[1]),
    ];
    let mut trackers = [UpdateScheduleTracker::new(), UpdateScheduleTracker::new()];
    let mut required = [SCIENCE_INVALID; 2];
    let mut last = [String::new(), String::new()];
    let mut rows = Vec::new();
    let mut record = |sim: &SimContext, tracker: &mut UpdateScheduleTracker, who: usize, result| {
        let mut xs = XferSave::new();
        xfer_special_power_update_module(&mut xs, tracker, sim).expect("xfer save");
        let hex = compact(&mut last[who], encode_hex_bytes(xs.as_bytes()));
        rows.push(format!("{hex},{result}"));
    };
    record(sim, &mut trackers[0], 0, 2);
    record(sim, &mut trackers[1], 1, 2);
    for op in &case.ops {
        let who = op[1] as usize;
        let mut result = 2;
        match op[0] {
            0 => {
                required[who] = science_of(op[2]);
                let pass =
                    does_special_power_update_pass_science_test(sim, owners[who], required[who]);
                result = i32::from(pass == TRUE);
            }
            1 => {
                sim.world
                    .player_sciences
                    .entry(teams[who])
                    .or_default()
                    .insert(science_of(op[2]));
            }
            2 => {
                sim.world.player_sciences.remove(&teams[who]);
            }
            3 => trackers[who].schedule.set_next_call_frame(op[2]),
            other => panic!("unknown op {other}"),
        }
        record(sim, &mut trackers[who], who, result);
    }
    let mut xc = XferCrc::new();
    crc_special_power_update_module(&mut xc, &trackers[0]);
    (rows, xc.get_crc())
}

fn run_decal(sim: &mut SimContext, case: &Case) -> (Vec<String>, u32) {
    let owner = spawn(sim, "P54C_Decal", TEAM_A);
    let m = module_named(sim, owner, "RadiusDecalUpdate");
    let mut last = String::new();
    let mut rows = Vec::new();
    let mut record = |sim: &mut SimContext| {
        let hex = compact(&mut last, save_hex(sim, m));
        rows.push(format!("{hex},{}", next_call_frame(sim, m)));
    };
    record(sim);
    let mut frame = case.frame;
    for op in &case.ops {
        match op[0] {
            0 => {
                frame += 1;
                step(sim, m, owner, frame);
            }
            1 => {
                let name = if op[1] != 0 { "SCCDecal" } else { "" };
                let radius = Real::from_bits(op[2]);
                with_decal(sim, m, |d, api, c| {
                    d.create_radius_decal(api, c, name, radius)
                });
            }
            2 => with_decal(sim, m, |d, api, c| d.kill_radius_decal(api, c)),
            3 => with_decal(sim, m, |d, _, _| {
                d.set_kill_when_no_longer_attacking(if op[1] != 0 { TRUE } else { FALSE })
            }),
            4 => {
                if op[1] != 0 {
                    sim.set_status(owner, ObjectStatusType::IsAttacking);
                } else {
                    sim.clear_status(owner, ObjectStatusType::IsAttacking);
                }
            }
            other => panic!("unknown op {other}"),
        }
        record(sim);
    }
    let crc = module_crc(sim, m);
    (rows, crc)
}

#[test]
fn wp54_c_matches_reference() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/golden_vectors/WP54_C.json");
    let doc = fs::read_to_string(path).expect("read WP54_C.json");
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
    assert_eq!(lines.len(), KINDS * 150, "vector count");
    let mut failures = [0_usize; KINDS];
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
        sim.terrain = Box::new(HeightTerrain);
        let (rows, crc) = match kind {
            0 => run_special_power(&mut sim, &case),
            1 => run_decal(&mut sim, &case),
            2 => run_die(&mut sim, &case),
            other => panic!("unknown kind {other}"),
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
        failures == [0; KINDS],
        "failures {failures:?}\n{}",
        first.join("\n")
    );
}
