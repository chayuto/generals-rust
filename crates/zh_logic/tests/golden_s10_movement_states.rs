//! Slice S10 of `docs/AI_PORT_PLAN.md`: the movement states (move to, attack
//! move, move out of the way, tighten, follow path) entered by real orders,
//! against a trace of the reference (`tests/cpp_harness/gen_s10_movement_states.cpp`).
//!
//! Every frame of every scene is compared: the logic random seed, which unit
//! stands in and wants which ground cell, and for each unit its position,
//! heading, velocity, blocked counters, speed limits, path nodes, state, model
//! condition flags, AI CRC and a digest of its AI save bytes (which hold the
//! state machine's and the current state's members). The harness's orders
//! (`ops`) are replayed, and the calls the reference made to the states,
//! searches and targeting that are not ported (`tape`) are checked in order and
//! answered from it.

use std::fmt::Write as _;
use std::sync::{Arc, Mutex};

use zh_common::core::state_machine::{StateExitType, StateId, StateReturnType};
use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::gamemath::real_to_int_floor;
use zh_common::math::{Coord2D, Coord3D, Matrix3D, Region3D};
use zh_common::types::{Bool, Int, ObjectId, Real, Relationship, TeamId, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferCrc, XferError, XferSave};
use zh_logic::ai::ai_group;
use zh_logic::ai::seams::{shared, AiSeams};
use zh_logic::arena::{ModuleContext, ModuleSlot, ObjectSlot};
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::gamelogic::frame::{run_frame, NoFrameSeams};
use zh_logic::gamelogic::global_data::BodyDamageType;
use zh_logic::modules::update::ai_update::{self, AiCall, AiCommandParms, AiCommandType, AiUpdate};
use zh_logic::modules::update::physics_update;
use zh_logic::modules::AttitudeType;
use zh_logic::modules::CommandSourceType;
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::object::weapon::NO_MAX_SHOTS_LIMIT;
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;
use zh_logic::pathfinder::Path;
use zh_logic::rules::model_state::MODELCONDITION_COUNT;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::sim::{SimApi, SimContext, SimView};

const VECTORS: &str = include_str!("../../../docs/golden_vectors/S10_MovementStates.json");

const TEMPLATES: [&str; 14] = [
    "S9_Inf",
    "S9_Veh",
    "S9_Tank",
    "S9_Dozer",
    "S9_Big",
    "S9_Climber",
    "S9_Air",
    "S9_Block",
    "S9_Jet",
    "S9_Tower",
    "S9_Meander",
    "S9_Crusher",
    "S10_Missile",
    "S10_Digger",
];
const BLOCK: usize = 7;
const TOWER: usize = 9;
const MODEL_CONDITIONS: Int = MODELCONDITION_COUNT as Int;
const TEAMS: [TeamId; 3] = [TeamId(1), TeamId(2), TeamId(3)];

// -----------------------------------------------------------------------------
// Text
// -----------------------------------------------------------------------------

fn bits(value: Real) -> String {
    format!("{:08x}", value.to_bits())
}

fn real(text: &str) -> Real {
    Real::from_bits(u32::from_str_radix(text, 16).expect("real bits"))
}

fn c3(p: &Coord3D) -> String {
    format!("{},{},{}", bits(p.x), bits(p.y), bits(p.z))
}

fn coord(f: &[&str]) -> Coord3D {
    Coord3D::new(real(f[0]), real(f[1]), real(f[2]))
}

fn number(text: &str) -> i64 {
    text.parse().expect("number")
}

fn fnv(text: &str) -> u64 {
    text.bytes().fold(1_469_598_103_934_665_603, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(1_099_511_628_211)
    })
}

// -----------------------------------------------------------------------------
// What is not ported
// -----------------------------------------------------------------------------

#[derive(Default)]
struct Tape {
    expected: Vec<String>,
    made: Vec<String>,
}

type SharedTape = Arc<Mutex<Tape>>; // determinism-ok: single-threaded harness log

struct Seams(SharedTape);

impl Seams {
    fn call(&self, call: String) {
        self.0.lock().expect("tape").made.push(call);
    }

    /// A call the reference answered: `asked=answer` on its tape.
    fn answer(&self, asked: String) -> Option<String> {
        let mut tape = self.0.lock().expect("tape");
        let at = tape.made.len();
        let answer = tape
            .expected
            .get(at)
            .and_then(|e| e.strip_prefix(&format!("{asked}=")))
            .map(str::to_owned);
        match &answer {
            Some(answer) => tape.made.push(format!("{asked}={answer}")),
            None => tape.made.push(asked),
        }
        answer
    }

    /// A search: the call, and the path the reference's stand-in answered with.
    fn search(&self, asked: String) -> Option<Path> {
        let answer = self.answer(asked)?;
        if answer == "-" {
            return None;
        }
        let f: Vec<&str> = answer.split(',').collect();
        let (from, mid, to) = (coord(&f[0..3]), coord(&f[3..6]), coord(&f[6..9]));
        let mut path = Path::new();
        path.prepend_node(&to, PathfindLayerEnum::Ground);
        path.prepend_node(&mid, PathfindLayerEnum::Ground);
        path.prepend_node(&from, PathfindLayerEnum::Ground);
        let first = path.first_node().expect("node");
        let second = path.node(first).next().expect("node");
        path.set_next_optimized(first, Some(second));
        path.set_next_optimized(second, path.last_node());
        Some(path)
    }
}

fn path_ends(path: Option<&Path>) -> String {
    let Some(path) = path else {
        return "-".to_owned();
    };
    let mut count = 0;
    let mut node = path.first_node();
    while let Some(n) = node {
        count += 1;
        node = path.node(n).next();
    }
    let at = |n: Option<_>| c3(path.node(n.expect("node")).position());
    format!("{count},{},{}", at(path.first_node()), at(path.last_node()))
}

impl AiSeams for Seams {
    fn state_enter(&mut self, owner: ObjectId, state: StateId) -> StateReturnType {
        self.call(format!("E,{},{state}", owner.0));
        StateReturnType::Continue
    }
    fn state_update(&mut self, owner: ObjectId, state: StateId) -> StateReturnType {
        match self.answer(format!("U,{},{state}", owner.0)).as_deref() {
            Some("1") => StateReturnType::Success,
            Some("2") => StateReturnType::Failure,
            _ => StateReturnType::Continue,
        }
    }
    fn next_mood_target(
        &mut self,
        owner: ObjectId,
        called_by_ai: Bool,
        during_idle: Bool,
    ) -> Option<ObjectId> {
        // The reference's stand-in answers nothing, unrecorded, for an idle unit.
        if during_idle != FALSE {
            return None;
        }
        let answer = self.answer(format!("T,{},{called_by_ai}", owner.0))?;
        let id = number(&answer) as i32;
        (id != 0).then_some(ObjectId(id))
    }
    fn state_exit(&mut self, owner: ObjectId, state: StateId, status: StateExitType) {
        self.call(format!("X,{},{state},{}", owner.0, status as Int));
    }
    fn state_xfer(
        &mut self,
        _owner: ObjectId,
        state: StateId,
        xfer: &mut dyn Xfer,
    ) -> Result<(), XferError> {
        let mut mark = 0xa5u8;
        xfer.xfer_unsigned_byte(&mut mark)?;
        let mut id = state;
        xfer.xfer_unsigned_int(&mut id)
    }
    fn closest_path(
        &mut self,
        owner: ObjectId,
        from: &Coord3D,
        to: &mut Coord3D,
        blocked: Bool,
        path_cost_factor: Real,
    ) -> Option<Path> {
        let path = self.search(format!(
            "C,{},{},{},{blocked},{}",
            owner.0,
            c3(from),
            c3(to),
            bits(path_cost_factor)
        ));
        if let Some(path) = &path {
            *to = *path.node(path.last_node().expect("node")).position();
        }
        path
    }
    fn patch_path(
        &mut self,
        owner: ObjectId,
        current: Option<&Path>,
        blocked: Bool,
    ) -> Option<Path> {
        self.search(format!("P,{},{},{blocked}", owner.0, path_ends(current)))
    }
    fn safe_path(
        &mut self,
        owner: ObjectId,
        from: &Coord3D,
        repulsor1: &Coord3D,
        repulsor2: &Coord3D,
        radius: Real,
    ) -> Option<Path> {
        self.search(format!(
            "S,{},{},{},{},{}",
            owner.0,
            c3(from),
            c3(repulsor1),
            c3(repulsor2),
            bits(radius)
        ))
    }
    fn move_away_path(
        &mut self,
        owner: ObjectId,
        unit: ObjectId,
        unit_path: &Path,
        second: Option<ObjectId>,
        second_path: Option<&Path>,
    ) -> Option<Path> {
        self.search(format!(
            "M,{},{},{},{},{}",
            owner.0,
            unit.0,
            path_ends(Some(unit_path)),
            second.map_or(0, |id| id.0),
            path_ends(second_path)
        ))
    }
}

// -----------------------------------------------------------------------------
// The world
// -----------------------------------------------------------------------------

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

fn world(tape: &SharedTape, cells: Int, cliff: &str) -> SimContext {
    let mut sim = SimContext::new();
    for block in text("locomotors").split_inclusive("End\n") {
        let mut t = LocomotorTemplate::new();
        t.parse_ini_block(block).expect("locomotor");
        t.name = block.split_whitespace().nth(1).expect("name").to_owned();
        sim.rules.locomotor_store.add_template(t);
    }
    let ini = text("objects");
    let mut reader = IniReader::new("object.ini", ini.as_bytes(), IniLoadType::Overwrite);
    sim.rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");
    sim.rules.global_data.gravity = -1.0;
    sim.rules.global_data.partition_cell_size = 20.0;
    sim.rules.global_data.movement_penalty_damage_state = BodyDamageType::ReallyDamaged;
    sim.rules.ai_data.force_idle_frames_count = 45;
    sim.rules.ai_data.repulsed_distance = 50.0;
    sim.ai_seams = shared(Seams(tape.clone()));
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
    w.player_is_human.insert(TEAMS[1], TRUE);
    w.player_is_human.insert(TEAMS[2], FALSE);
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
    sim
}

// -----------------------------------------------------------------------------
// Observation
// -----------------------------------------------------------------------------

fn ai_module(sim: &SimContext, id: ObjectId) -> Option<(ObjectSlot, ModuleSlot)> {
    let slot = sim.find_object(id)?;
    Some((slot, sim.object(slot)?.ai_slot?))
}

fn context(sim: &SimContext, ms: ModuleSlot) -> ModuleContext {
    sim.world.modules.meta(ms).context(ms)
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
    let Some(slot) = sim.find_object(id) else {
        return "gone".to_owned();
    };
    let record = sim.object(slot).expect("object");
    let pos = c3(record.thing.get_position());
    let heading = bits(record.thing.get_orientation());
    let layer = record.layer as Int;
    let Some(ms) = record.ai_slot else {
        return pos;
    };
    let velocity =
        physics_update::physics_mut(sim, slot).map_or_else(|| "-".to_owned(), |p| c3(&p.vel));
    let path = path_text(state(sim, ms).path.as_ref());
    let mut flags = format!("{},", state(sim, ms).machine.current_state_id());
    let condition = &sim.object(slot).expect("object").model_condition;
    for bit in 0..MODEL_CONDITIONS {
        if condition.test(bit) != FALSE {
            let _ = write!(flags, "{bit}.");
        }
    }
    let (save, crc) = sim
        .dispatch_module(ms, |m, sim, ctx| {
            let module = m
                .as_any_mut()
                .and_then(|a| a.downcast_mut::<AiUpdate>())
                .expect("AI module");
            let mut save = XferSave::new();
            module
                .xfer_as_version(&mut save, sim, ctx.owner_id, 4)
                .expect("save");
            let mut crc = XferCrc::new();
            module
                .xfer_as_version(&mut crc, sim, ctx.owner_id, 4)
                .expect("crc");
            let mut hex = String::new();
            for byte in save.as_bytes() {
                let _ = write!(hex, "{byte:02x}");
            }
            (hex, crc.get_crc())
        })
        .expect("dispatch");
    let ai = state(sim, ms);
    format!(
        "{pos}|{heading}|{velocity}|{}{}{}{}{}{}|{},{}|{},{},{}|{},{},{},{},{layer}|{path}|{flags}|{crc:08x}|{:016x}",
        ai.blocked_frames,
        i32::from(ai.is_blocked != FALSE),
        i32::from(ai.is_blocked_and_stuck != FALSE),
        i32::from(ai.retry_path != FALSE),
        i32::from(ai.waiting_for_path != FALSE),
        i32::from(ai.is_moving() != FALSE),
        bits(ai.cur_max_blocked_speed),
        bits(ai.bump_speed_limit),
        ai.path_timestamp,
        ai.queue_for_path_frame,
        ai.ignore_collisions_until,
        ai.pathfind_goal_cell.x,
        ai.pathfind_goal_cell.y,
        ai.pathfind_cur_cell.x,
        ai.pathfind_cur_cell.y,
        fnv(&save)
    )
}

const FIELDS: [&str; 11] = [
    "position",
    "heading",
    "velocity",
    "blocked frames, blocked, stuck, retry, waiting, moving",
    "blocked and bump speed",
    "path time, queue frame, ignore collisions",
    "goal cell, current cell, layer",
    "path",
    "state, model conditions",
    "AI crc",
    "AI save digest",
];

fn check(context: &str, unit: usize, want: &str, got: &str) {
    if want == got {
        return;
    }
    for (i, (w, g)) in want.split('|').zip(got.split('|')).enumerate() {
        assert_eq!(
            g,
            w,
            "{context}: unit {unit}, {}",
            FIELDS.get(i).unwrap_or(&"?")
        );
    }
    assert_eq!(got, want, "{context}: unit {unit}");
}

/// Which unit stands in and wants which ground cell.
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

// -----------------------------------------------------------------------------
// The harness's calls
// -----------------------------------------------------------------------------

fn points(text: &str) -> Vec<Coord3D> {
    if text == "-" {
        return Vec::new();
    }
    text.split('/')
        .map(|p| coord(&p.split(',').collect::<Vec<_>>()))
        .collect()
}

/// `<name> <unit> <by player> <target or -1> <points or ->`.
fn order(
    sim: &mut SimContext,
    ids: &[ObjectId],
    ctx: ModuleContext,
    cmd: AiCommandType,
    f: &[&str],
) {
    let source = if f[2] == "1" {
        CommandSourceType::FromPlayer
    } else {
        CommandSourceType::FromAi
    };
    let mut parms = AiCommandParms::new(cmd, source);
    parms.obj = usize::try_from(number(f[3])).ok().map(|i| ids[i]);
    parms.coords = points(f[4]);
    if let Some(first) = parms.coords.first() {
        parms.pos = *first;
    }
    parms.int_value = NO_MAX_SHOTS_LIMIT;
    ai_update::execute(sim, ctx, &AiCall::Command(parms));
}

fn apply(sim: &mut SimContext, ids: &[ObjectId], op: &str, context_text: &str) {
    use AiCommandType as C;
    let f: Vec<&str> = op.split(' ').collect();
    let id = ids[number(f[1]) as usize];
    let (slot, ms) = ai_module(sim, id).expect("unit with AI");
    let ctx = context(sim, ms);
    let other = |k: usize| ids[number(f[k]) as usize];
    let command = match f[0] {
        "move" => Some(C::MoveToPosition),
        "moveobj" => Some(C::MoveToObject),
        "amove" => Some(C::AttackMoveToPosition),
        "follow" => Some(C::FollowPath),
        "append" => Some(C::FollowPathAppend),
        "exit" => Some(C::FollowExitProductionPath),
        "tighten" => Some(C::TightenToPosition),
        "away" => Some(C::MoveAwayFromUnit),
        "idle" => Some(C::Idle),
        _ => None,
    };
    if let Some(command) = command {
        order(sim, ids, ctx, command, &f);
        return;
    }
    match f[0] {
        "quickexit" => ai_update::execute(sim, ctx, &AiCall::QuickExit(points(f[2]))),
        "ign" => state(sim, ms).ignore_obstacle_id = other(2),
        "pos" => {
            sim.set_object_position(slot, &coord(&f[2].split(',').collect::<Vec<_>>()));
            sim.set_object_orientation(slot, real(f[3]));
        }
        "coll" => {
            let them = sim.find_object(other(2)).expect("other");
            let result = sim.ai_process_collision(slot, them);
            assert_eq!(
                i64::from(result != FALSE),
                number(f[3]),
                "{context_text}: processCollision"
            );
        }
        "ultra" => {
            let ai = state(sim, ms);
            let index = ai.cur_locomotor.expect("locomotor");
            ai.locomotor_set
                .get_mut(index)
                .expect("locomotor")
                .set_ultra_accurate(TRUE);
        }
        "status" => {
            let bit = ObjectStatusType::from_i32(number(f[2]) as Int).expect("status");
            if f[3] == "1" {
                sim.set_status(slot, bit);
            } else {
                sim.clear_status(slot, bit);
            }
        }
        "att" => {
            let attitude = match number(f[2]) {
                2 => AttitudeType::Aggressive,
                1 => AttitudeType::Alert,
                _ => AttitudeType::Normal,
            };
            sim.ai_set_attitude(slot, attitude);
        }
        "form" => {
            let them = other(2);
            let group = ai_group::create_group(sim);
            ai_group::add(sim, group, id);
            ai_group::add(sim, group, them);
            let object = sim.object_mut(slot).expect("object");
            object.formation_id = 7;
            object.formation_offset = Coord2D::default();
        }
        "nocoll" => {
            let until = sim.world.frame.wrapping_add(number(f[2]) as u32);
            state(sim, ms).ignore_collisions_until = until;
        }
        "speed" => state(sim, ms).desired_speed = real(f[2]),
        other => panic!("{context_text}: unknown call {other}"),
    }
}

// -----------------------------------------------------------------------------
// The test
// -----------------------------------------------------------------------------

fn set_tape(tape: &SharedTape, text: &str) {
    let mut t = tape.lock().expect("tape");
    t.expected = text
        .split(';')
        .filter(|c| !c.is_empty())
        .map(str::to_owned)
        .collect();
    t.made.clear();
}

fn check_tape(tape: &SharedTape, context_text: &str) {
    let t = tape.lock().expect("tape");
    for (i, (made, expected)) in t.made.iter().zip(&t.expected).enumerate() {
        assert_eq!(made, expected, "{context_text}: call {i}");
    }
    assert_eq!(t.made, t.expected, "{context_text}: calls");
}

fn run(scene: &str, frames: &[&str], cells: Int) {
    let index = get_u32(scene, "s").expect("scene");
    let tape: SharedTape = Arc::default();
    let mut sim = world(&tape, cells, get_str(scene, "cliff").expect("cliff"));
    sim.world.frame = get_u32(scene, "frame").expect("frame");
    sim.world
        .rng
        .init_random_with_seed(get_u32(scene, "seed").expect("seed"));
    sim.world.next_object_id = 1000;
    sim.new_pathfind_map(&extent(cells), 0.0);

    set_tape(&tape, get_str(scene, "tape").expect("tape"));
    let mut ids = Vec::new();
    for spawn in get_str(scene, "spawns").expect("spawns").split(';') {
        let f: Vec<&str> = spawn.split(',').collect();
        let kind = number(f[0]) as usize;
        let template = sim
            .rules
            .thing_factory
            .find_template_id(TEMPLATES[kind])
            .expect("template");
        let team = TEAMS[1 + number(f[1]) as usize];
        // The reference creates the object at the origin, then places it.
        let slot = sim.spawn_object(template, team, Coord3D::default(), 0.0);
        sim.set_object_orientation(slot, real(f[5]));
        sim.set_object_position(slot, &coord(&f[2..5]));
        if kind == BLOCK || kind == TOWER {
            sim.add_object_to_pathfind_map(slot);
        }
        ids.push(sim.object(slot).expect("spawned").id);
    }
    check_tape(&tape, &format!("scene {index} spawns"));

    let mut seen = vec![String::new(); ids.len()];
    for row in frames {
        let frame = get_u32(row, "f").expect("frame");
        let ops = get_str(row, "ops").expect("ops");
        let context_text = format!("scene {index} frame {frame} [{ops}]");
        set_tape(&tape, get_str(row, "tape").expect("tape"));
        for op in ops.split(';').filter(|o| !o.is_empty()) {
            apply(&mut sim, &ids, op, &context_text);
        }
        run_frame(&mut sim, &mut NoFrameSeams);
        check_tape(&tape, &context_text);
        assert_eq!(
            format!("{:08x}", sim.world.rng.get_game_logic_random_seed_crc()),
            get_str(row, "seed").expect("seed"),
            "{context_text}: random seed"
        );
        let units = get_str(row, "u").expect("units");
        for (i, want) in units.split('~').enumerate() {
            if want != "=" {
                seen[i] = want.to_owned();
            }
            let got = observe(&mut sim, ids[i]);
            check(&context_text, i, &seen[i], &got);
        }
        assert_eq!(
            cells_digest(&sim, cells),
            get_str(row, "map").expect("map"),
            "{context_text}: unit cells"
        );
    }
}

#[test]
fn movement_states_match_the_reference() {
    let rows: Vec<&str> = VECTORS
        .lines()
        .filter(|l| l.starts_with("{\"s\":"))
        .collect();
    let cells = get_u32(VECTORS, "cells").expect("cells") as Int;
    let per_scene = get_u32(VECTORS, "frames").expect("frames") as usize;
    let scenes = get_u32(VECTORS, "vector_count").expect("count") as usize;
    assert!(scenes >= 56);
    assert_eq!(rows.len(), scenes * (per_scene + 1));
    for scene in rows.chunks(per_scene + 1) {
        run(scene[0], &scene[1..], cells);
    }
}
