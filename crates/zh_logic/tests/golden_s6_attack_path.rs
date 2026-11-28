//! The path requests that end in the S6 searches, against the reference:
//! attack, approach, safe, stuck and move-away requests served by `doPathfind`
//! on the pathfind map, every search real.
//!
//! Vectors: `tests/cpp_harness/gen_s6_attack_path.cpp`.

use std::fmt::Write as _;

use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::gamemath::real_to_int_floor;
use zh_common::math::{Coord3D, Matrix3D, Region3D};
use zh_common::types::{Bool, Int, ObjectId, Real, Relationship, TeamId, FALSE, TRUE};
use zh_logic::arena::{ModuleContext, ModuleSlot, ObjectSlot};
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::gamelogic::global_data::BodyDamageType;
use zh_logic::modules::update::ai_update::{self, AiCall, AiCommandParms, AiCommandType, AiUpdate};
use zh_logic::modules::CommandSourceType;
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::object::weapon::WeaponTemplate;
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;
use zh_logic::pathfinder::Path;
use zh_logic::sim::{SimApi, SimContext, SimView};

const VECTORS: &str = include_str!("../../../docs/golden_vectors/S6_AttackPath.json");

const TEMPLATES: [&str; 8] = [
    "S6_Rifle",
    "S6_Scout",
    "S6_Mortar",
    "S6_Stab",
    "S6_Flyer",
    "S6_Truck",
    "S6_Block",
    "S6_Fence",
];
const BLOCK: usize = 6;
const TEAMS: [TeamId; 3] = [TeamId(1), TeamId(2), TeamId(3)];

fn bits(value: Real) -> String {
    format!("{:08x}", value.to_bits())
}

fn real(text: &str) -> Real {
    Real::from_bits(u32::from_str_radix(text, 16).expect("real bits"))
}

fn c3(p: &Coord3D) -> String {
    format!("{},{},{}", bits(p.x), bits(p.y), bits(p.z))
}

fn coord(text: &str) -> Coord3D {
    let f: Vec<&str> = text.split(',').collect();
    Coord3D::new(real(f[0]), real(f[1]), real(f[2]))
}

fn number(text: &str) -> usize {
    text.parse().expect("number")
}

fn fnv(text: &str) -> u64 {
    text.bytes().fold(1_469_598_103_934_665_603, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(1_099_511_628_211)
    })
}

/// Flat ground with cliff cells.
struct Terrain {
    cells: Int,
    cliff: Vec<u8>,
}

impl ThingTerrainAccess for Terrain {
    fn get_ground_height(&self, _x: Real, _y: Real, normal: Option<&mut Coord3D>) -> Real {
        if let Some(normal) = normal {
            *normal = Coord3D::new(0.0, 0.0, 1.0);
        }
        0.0
    }
    fn align_on_terrain(
        &self,
        angle: Real,
        pos: &Coord3D,
        stick_to_ground: Bool,
        mtx: &mut Matrix3D,
    ) -> PathfindLayerEnum {
        zh_logic::sim::FlatTerrain.align_on_terrain(angle, pos, stick_to_ground, mtx)
    }
    fn is_cliff_cell(&self, x: Real, y: Real) -> Bool {
        let cell = |v: Real| real_to_int_floor(v / 10.0).clamp(0, self.cells - 1);
        Bool::from(self.cliff[(cell(y) * self.cells + cell(x)) as usize] == b'1')
    }
    fn is_underwater(
        &self,
        _x: Real,
        _y: Real,
        water_z: Option<&mut Real>,
        terrain_z: Option<&mut Real>,
    ) -> Bool {
        if let Some(z) = water_z {
            *z = 0.0;
        }
        if let Some(z) = terrain_z {
            *z = 0.0;
        }
        FALSE
    }
    fn playable_extent(&self) -> Option<Region3D> {
        Some(extent(self.cells))
    }
}

fn extent(cells: Int) -> Region3D {
    let side = cells as Real * 10.0;
    Region3D {
        lo: Coord3D::new(0.0, 0.0, 0.0),
        hi: Coord3D::new(side, side, 100.0),
    }
}

fn text(key: &str) -> String {
    get_str(VECTORS, key).expect("text").replace("\\n", "\n")
}

fn blocks(key: &str) -> Vec<String> {
    text(key)
        .split_inclusive("End\n")
        .map(str::to_owned)
        .collect()
}

fn world(cells: Int, cliff: &str) -> SimContext {
    let mut sim = SimContext::new();
    for block in blocks("locomotors") {
        let mut t = LocomotorTemplate::new();
        t.parse_ini_block(&block).expect("locomotor");
        t.name = block.split_whitespace().nth(1).expect("name").to_owned();
        sim.rules.locomotor_store.add_template(t);
    }
    for block in blocks("weapons") {
        let mut weapon = WeaponTemplate::new();
        weapon.parse_ini_block(&block).expect("weapon");
        sim.rules.weapon_store.add_template(weapon);
    }
    let ini = text("objects");
    let mut reader = IniReader::new("object.ini", ini.as_bytes(), IniLoadType::Overwrite);
    sim.rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");
    sim.rules.global_data.gravity = -1.0;
    sim.rules.global_data.partition_cell_size = 20.0;
    sim.rules.global_data.movement_penalty_damage_state = BodyDamageType::ReallyDamaged;
    sim.rules.ai_data.repulsed_distance = 50.0;
    sim.rules.ai_data.attack_uses_line_of_sight = TRUE;
    sim.terrain = Box::new(Terrain {
        cells,
        cliff: cliff.as_bytes().to_vec(),
    });
    sim.init_partition(20.0, extent(cells));
    let w = &mut sim.world;
    for (player, team) in TEAMS.into_iter().enumerate() {
        w.team_players.insert(team, player as i32);
        w.player_default_teams.insert(player as i32, team);
    }
    w.set_team_human(TEAMS[1], TRUE);
    w.set_team_human(TEAMS[2], FALSE);
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
            w.set_player_relation(i, j, rel);
        }
    }
    sim
}

fn ai_module(sim: &SimContext, id: ObjectId) -> Option<(ObjectSlot, ModuleSlot)> {
    let slot = sim.find_object(id)?;
    Some((slot, sim.object(slot)?.ai_slot?))
}

fn state(sim: &mut SimContext, ms: ModuleSlot) -> &mut AiUpdate {
    sim.resident_state_mut(ms)
        .and_then(|a| a.downcast_mut::<AiUpdate>())
        .expect("AI module")
}

fn path_text(path: Option<&Path>) -> String {
    let Some(path) = path else {
        return "-".to_owned();
    };
    let mut nodes = Vec::new();
    let mut node = path.first_node();
    while let Some(n) = node {
        nodes.push(n);
        node = path.node(n).next();
    }
    let mut text = nodes.len().to_string();
    for &n in &nodes {
        let node = path.node(n);
        let next = node
            .next_optimized()
            .and_then(|to| nodes.iter().position(|&k| k == to))
            .map_or(-1, |k| k as i64);
        let _ = write!(
            text,
            "/{},{},{next}",
            c3(node.position()),
            node.layer() as Int
        );
    }
    text
}

fn observe(sim: &mut SimContext, id: ObjectId) -> String {
    let slot = sim.find_object(id).expect("object");
    let record = sim.object(slot).expect("object");
    let pos = c3(record.thing.get_position());
    let Some(ms) = record.ai_slot else {
        return pos;
    };
    let ai = state(sim, ms);
    format!(
        "{pos}|{}{}{}{}{}{},{}|{},{}|{}|{}|{}|{},{},{},{}|{}",
        ai.is_attack_path,
        ai.is_approach_path,
        ai.is_safe_path,
        ai.waiting_for_path,
        ai.is_blocked_and_stuck,
        ai.is_blocked,
        ai.blocked_frames,
        ai.path_timestamp,
        ai.ignore_collisions_until,
        c3(&ai.requested_destination),
        ai.ignore_obstacle_id.0,
        ai.machine.current_state_id(),
        ai.pathfind_goal_cell.x,
        ai.pathfind_goal_cell.y,
        ai.pathfind_cur_cell.x,
        ai.pathfind_cur_cell.y,
        path_text(ai.path.as_ref())
    )
}

fn cells_digest(sim: &SimContext, cells: Int) -> String {
    let pathfinder = sim.world.pathfinder.as_ref().expect("map");
    let mut text = String::new();
    for y in 0..cells {
        for x in 0..cells {
            let Some(cell) = pathfinder.get_cell(PathfindLayerEnum::Ground, x, y) else {
                continue;
            };
            let (goal, present) = (
                pathfinder.goal_unit(cell).0,
                pathfinder.unit_present(cell).0,
            );
            if goal != 0 || present != 0 {
                let _ = write!(text, "{x},{y},{goal},{present};");
            }
        }
    }
    format!("{:016x}", fnv(&text))
}

fn order(sim: &mut SimContext, ctx: ModuleContext, cmd: AiCommandType, obj: Option<ObjectId>) {
    let mut parms = AiCommandParms::new(cmd, CommandSourceType::FromAi);
    parms.obj = obj;
    ai_update::execute(sim, ctx, &AiCall::Command(parms));
}

fn apply(sim: &mut SimContext, ids: &[ObjectId], op: &str) {
    let f: Vec<&str> = op.split(' ').collect();
    let (slot, ms) = ai_module(sim, ids[number(f[1])]).expect("unit with AI");
    let ctx = sim.world.modules.meta(ms).context(ms);
    if f[0] == "pos" {
        sim.set_object_orientation(slot, real(f[3]));
        sim.set_object_position(slot, &coord(f[2]));
        return;
    }
    if f[0] == "away" {
        let other = ids[number(f[2])];
        order(sim, ctx, AiCommandType::MoveAwayFromUnit, Some(other));
        return;
    }
    order(sim, ctx, AiCommandType::Busy, None);
    match f[0] {
        "attack" => {
            let victim = ids[number(f[2])];
            let at = sim.find_object(victim).expect("victim");
            let at = *sim.object(at).expect("victim").thing.get_position();
            ai_update::request_attack_path(sim, ctx, victim, &at);
        }
        "ground" => ai_update::request_attack_path(sim, ctx, ObjectId::INVALID, &coord(f[2])),
        "approach" => ai_update::request_approach_path(sim, ctx, &coord(f[2])),
        "safe" => ai_update::request_safe_path(sim, ctx, ids[number(f[2])]),
        "move" => {
            ai_update::request_path(sim, ctx, &coord(f[2]), TRUE);
            if f[3] == "1" {
                state(sim, ms).is_blocked_and_stuck = TRUE;
            }
        }
        other => panic!("unknown op {other}"),
    }
    if f.last() == Some(&"stuck") {
        let ai = state(sim, ms);
        ai.is_blocked_and_stuck = TRUE;
        ai.is_blocked = TRUE;
        ai.blocked_frames = 7;
    }
    ai_update::do_pathfind(sim, ctx);
}

/// `processPathfindQueue` with nothing queued: the zones are brought up to date.
fn settle_map(sim: &mut SimContext, cells: Int) {
    let frame = sim.world.frame;
    let map = sim.world.pathfinder.as_mut().expect("map");
    map.begin_path_queue(&extent(cells), frame);
    map.begin_path_queue(&extent(cells), frame);
}

fn run(scene: &str, rows: &[&str], cells: Int) {
    let index = get_u32(scene, "s").expect("scene");
    let mut sim = world(cells, get_str(scene, "cliff").expect("cliff"));
    sim.world.frame = get_u32(scene, "frame").expect("frame");
    sim.world.rng.init_random_with_seed(1 + index);
    sim.world.next_object_id = 1000;
    sim.new_pathfind_map(&extent(cells), 0.0);

    let mut ids = Vec::new();
    for spawn in get_str(scene, "spawns").expect("spawns").split(';') {
        let f: Vec<&str> = spawn.splitn(3, ',').collect();
        let kind = number(f[0]);
        let template = sim
            .rules
            .thing_factory
            .find_template_id(TEMPLATES[kind])
            .expect("template");
        let team = TEAMS[1 + number(f[1])];
        // The reference creates the object at the origin, then places it.
        let slot = sim.spawn_object(template, team, Coord3D::default(), 0.0);
        sim.set_object_position(slot, &coord(f[2]));
        if kind >= BLOCK {
            sim.add_object_to_pathfind_map(slot);
        }
        ids.push(sim.object(slot).expect("spawned").id);
    }
    settle_map(&mut sim, cells);

    let mut seen = vec![String::new(); ids.len()];
    for (step, row) in rows.iter().enumerate() {
        sim.world.frame = get_u32(row, "f").expect("frame");
        let op = get_str(row, "op").expect("op");
        let context = format!("scene {index} row {step} [{op}]");
        apply(&mut sim, &ids, op);
        assert_eq!(
            format!("{:08x}", sim.world.rng.get_game_logic_random_seed_crc()),
            get_str(row, "seed").expect("seed"),
            "{context}: random seed"
        );
        for (i, want) in get_str(row, "u").expect("units").split('~').enumerate() {
            if want != "=" {
                seen[i] = want.to_owned();
            }
            assert_eq!(observe(&mut sim, ids[i]), seen[i], "{context}: unit {i}");
        }
        assert_eq!(
            cells_digest(&sim, cells),
            get_str(row, "map").expect("map"),
            "{context}: unit cells"
        );
    }
}

#[test]
fn path_requests_match_the_reference() {
    let lines: Vec<&str> = VECTORS
        .lines()
        .filter(|l| l.starts_with("{\"s\":"))
        .collect();
    let cells = get_u32(VECTORS, "cells").expect("cells") as Int;
    let rows = get_u32(VECTORS, "rows").expect("rows") as usize;
    let scenes = get_u32(VECTORS, "vector_count").expect("count") as usize;
    assert!(scenes >= 100);
    assert_eq!(lines.len(), scenes * (rows + 1));
    for scene in lines.chunks(rows + 1) {
        run(scene[0], &scene[1..], cells);
    }
}
