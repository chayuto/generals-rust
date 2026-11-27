//! Slice S16 of `docs/AI_PORT_PLAN.md`: the dozer and worker frame updates, their
//! state machines (build, repair, mine clearing, bored behaviour) and orders,
//! against a trace of the reference
//! (`tests/cpp_harness/gen_s16_dozer_worker_frames.cpp`).
//!
//! Every frame of every scene is compared: the logic random seed, the pathfind
//! cells (types and the units standing in or wanting them), the players'
//! buildings built and energy, and for every object its health and status. A
//! unit also has its position, heading, velocity, cells, path, AI state, wake
//! frame, the dozer's machines, tasks and dock points, AI CRC and a digest of its
//! AI save bytes; a building its construction percent, builder, producer, healer
//! and model condition flags. The harness's calls (`ops`) are replayed, and the
//! calls the reference made to the states, searches and targeting that are not
//! ported (`tape`) are checked in order and answered from it.

use std::fmt::Write as _;
use std::sync::{Arc, Mutex};

use zh_common::core::state_machine::{StateExitType, StateId, StateReturnType};
use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::gamemath::real_to_int_floor;
use zh_common::math::{Coord3D, Matrix3D, Region3D};
use zh_common::types::{Bool, Int, ObjectId, Real, Relationship, TeamId, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferCrc, XferError, XferSave};
use zh_logic::ai::seams::{shared, AiSeams};
use zh_logic::arena::{ModuleSlot, ObjectSlot};
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::gamelogic::frame::{run_frame, NoFrameSeams};
use zh_logic::gamelogic::global_data::BodyDamageType;
use zh_logic::modules::update::ai_update::{
    self, AiCall, AiCommandParms, AiCommandType, AiUpdate, DerivedAi,
};
use zh_logic::modules::update::dozer_ai_update::DozerBrain;
use zh_logic::modules::update::physics_update;
use zh_logic::modules::CommandSourceType;
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::object::weapon::{WeaponTemplate, NO_MAX_SHOTS_LIMIT};
use zh_logic::object::weapon_set::CanAttackResult;
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;
use zh_logic::pathfinder::Path;
use zh_logic::rules::combat_types::WeaponSetType;
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::model_state::{
    ModelConditionFlagType, ModelConditionFlags, MODELCONDITION_COUNT,
};
use zh_logic::rules::object_status_types::{
    ObjectStatusMaskType, ObjectStatusType, OBJECT_STATUS_COUNT,
};
use zh_logic::sim::{
    AttackAbility, FindPositionOptions, FlatTerrain, SimApi, SimContext, SimView,
    FPF_IGNORE_ALL_OBJECTS,
};

const VECTORS: &str = include_str!("../../../docs/golden_vectors/S16_DozerWorkerFrames.json");

const TEMPLATES: [&str; 7] = [
    "S16_Dozer",
    "S16_Worker",
    "S16_Unit",
    "S16_House",
    "S16_Quick",
    "S16_Reactor",
    "S16_Mine",
];
const DOZER: usize = 0;
const WORKER: usize = 1;
const UNIT: usize = 2;
const MINE: usize = 6;
/// The layout of a scene with mines, whose builders may attack them.
const MINE_LAYOUT: u32 = 2;
/// Neutral, then the two playing teams.
const TEAMS: [TeamId; 3] = [TeamId(1), TeamId(2), TeamId(3)];

// -----------------------------------------------------------------------------
// Text
// -----------------------------------------------------------------------------

fn bits(value: Real) -> String {
    format!("{:08x}", value.to_bits())
}

fn real(text: &str) -> Real {
    Real::from_bits(u32::from_str_radix(text, 16).expect("float bits"))
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
    // The reference run replaced the attack states and the searches.
    fn replaces_state(&mut self, _owner: ObjectId, state: StateId) -> Bool {
        Bool::from(zh_logic::ai::ai_states::attack::is_attack_state(state))
    }
    fn replaces_searches(&mut self) -> Bool {
        TRUE
    }
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
        FlatTerrain.align_on_terrain(angle, pos, stick_to_ground, mtx)
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

fn world(tape: &SharedTape, cells: Int, cliff: &str) -> SimContext {
    let mut sim = SimContext::new();
    for block in text("locomotors").split_inclusive("End\n") {
        let mut t = LocomotorTemplate::new();
        t.parse_ini_block(block).expect("locomotor");
        t.name = block.split_whitespace().nth(1).expect("name").to_owned();
        sim.rules.locomotor_store.add_template(t);
    }
    for block in text("weapons").split_inclusive("End\n") {
        let mut weapon = WeaponTemplate::new();
        weapon.parse_ini_block(block).expect("weapon");
        sim.rules.weapon_store.add_template(weapon);
    }
    let ini = text("objects");
    let mut reader = IniReader::new("object.ini", ini.as_bytes(), IniLoadType::Overwrite);
    sim.rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");
    sim.rules.global_data.gravity = -1.0;
    sim.rules.global_data.partition_cell_size = 20.0;
    sim.rules.global_data.max_low_energy_production_speed = 0.8;
    sim.rules.global_data.min_low_energy_production_speed = 0.5;
    sim.rules.global_data.low_energy_penalty_modifier = 0.4;
    sim.rules.global_data.movement_penalty_damage_state = BodyDamageType::ReallyDamaged;
    sim.rules.ai_data.force_idle_frames_count = 45;
    sim.rules.ai_data.repulsed_distance = 50.0;
    sim.rules.ai_data.ai_dozer_bored_radius_modifier = 2.0;
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

fn model_text(sim: &SimContext, slot: ObjectSlot) -> String {
    let condition = &sim.object(slot).expect("object").model_condition;
    let mut text = String::new();
    for bit in 0..MODELCONDITION_COUNT as Int {
        if condition.test(bit) != FALSE {
            let _ = write!(text, "{bit}.");
        }
    }
    text
}

fn status_text(sim: &SimContext, slot: ObjectSlot) -> String {
    let status = &sim.object(slot).expect("object").status;
    let mut text = String::new();
    for bit in 0..OBJECT_STATUS_COUNT as Int {
        if status.test(bit) != FALSE {
            let _ = write!(text, "{bit}.");
        }
    }
    text
}

fn dozer_text(derived: &DerivedAi) -> String {
    let (brain, worker): (&DozerBrain, Option<u32>) = match derived {
        DerivedAi::Base => return "-".to_owned(),
        DerivedAi::Dozer(d) => (&d.brain, None),
        DerivedAi::Worker(w) => (&w.brain, Some(w.state)),
    };
    let mut text = format!(
        "{},{},{},{}",
        brain.state, brain.current_task, brain.previous_task, brain.build_sub_task
    );
    for (task, action) in brain.actions.iter().enumerate() {
        let _ = write!(text, "/{}", brain.tasks[task].target.0);
        for point in &brain.dock_points[task] {
            if point.valid != FALSE {
                let _ = write!(text, ";{}", c3(&point.location));
            } else {
                text.push_str(";-");
            }
        }
        let _ = write!(
            text,
            ";{};{};{}",
            action.current,
            action.goal_object.0,
            c3(&action.goal_position)
        );
    }
    if let Some(state) = worker {
        let _ = write!(text, "/{state}");
    }
    text
}

fn observe(sim: &mut SimContext, id: ObjectId) -> String {
    let Some(slot) = sim.find_object(id) else {
        return "gone".to_owned();
    };
    let health = bits(sim.object_health(slot));
    let record = sim.object(slot).expect("object");
    let Some(ms) = record.ai_slot else {
        let healer = if sim.world.frame > record.sole_healing_benefactor_expiration_frame {
            0
        } else {
            record.sole_healing_benefactor_id.0
        };
        let who = format!("{},{},{healer}", record.builder_id.0, record.producer_id.0);
        return format!(
            "{health}|{}|{}|{who}|{}",
            bits(sim.object_construction_percent(slot)),
            status_text(sim, slot),
            model_text(sim, slot)
        );
    };
    let pos = c3(record.thing.get_position());
    let heading = bits(record.thing.get_orientation());
    let mines = record
        .weapon_set_flags
        .test(WeaponSetType::MineClearingDetail.as_i32());
    let velocity =
        physics_update::physics_mut(sim, slot).map_or_else(|| "-".to_owned(), |p| c3(&p.vel));
    let wake = sim.world.modules.meta(ms).schedule.next_call_frame();
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
    let states = format!(
        "{},{}",
        state(sim, ms).machine.current_state_id(),
        model_text(sim, slot)
    );
    let object = format!(
        "{},{health},{}",
        i32::from(mines != FALSE),
        status_text(sim, slot)
    );
    let ai = state(sim, ms);
    format!(
        "{pos}|{heading}|{velocity}|{},{},{},{}|{}|{states}|{object}|{wake}|{}|{crc:08x}|{:016x}",
        ai.pathfind_goal_cell.x,
        ai.pathfind_goal_cell.y,
        ai.pathfind_cur_cell.x,
        ai.pathfind_cur_cell.y,
        path_text(ai.path.as_ref()),
        dozer_text(&ai.derived),
        fnv(&save)
    )
}

const UNIT_FIELDS: [&str; 11] = [
    "position",
    "heading",
    "velocity",
    "goal cell, current cell",
    "path",
    "state, model conditions",
    "mine clearing set, health, status",
    "wake frame",
    "dozer machines",
    "AI crc",
    "AI save digest",
];

const BUILDING_FIELDS: [&str; 5] = [
    "health",
    "construction percent",
    "status",
    "builder, producer, healer",
    "model conditions",
];

/// Takes the changed fields of `want` into `seen`.
fn merge(seen: &mut Vec<String>, want: &str) {
    if want == "=" {
        return;
    }
    let fields: Vec<&str> = want.split('|').collect();
    if fields.len() != seen.len() {
        *seen = fields.into_iter().map(str::to_owned).collect();
        return;
    }
    for (old, new) in seen.iter_mut().zip(fields) {
        if new != "=" {
            *old = new.to_owned();
        }
    }
}

fn check(context: &str, unit: usize, want: &[String], got: &str) {
    let got: Vec<&str> = got.split('|').collect();
    let names: &[&str] = if want.len() == UNIT_FIELDS.len() {
        &UNIT_FIELDS
    } else {
        &BUILDING_FIELDS
    };
    for (i, (w, g)) in want.iter().zip(&got).enumerate() {
        assert_eq!(
            g,
            w,
            "{context}: unit {unit}, {}",
            names.get(i).unwrap_or(&"?")
        );
    }
    assert_eq!(got.len(), want.len(), "{context}: unit {unit}: {got:?}");
}

/// Which unit stands in and wants which ground cell, and the cell types.
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
            let kind = pathfinder
                .cell_type(PathfindLayerEnum::Ground, x, y)
                .map_or(0, |t| t as Int);
            if goal != 0 || present != 0 || kind != 0 {
                let _ = write!(text, "{x},{y},{goal},{present},{kind};");
            }
        }
    }
    format!("{:016x}", fnv(&text))
}

fn players_text(sim: &SimContext) -> String {
    let mut text = String::new();
    for player in 1..=2 {
        let built = sim
            .world
            .score_keepers
            .get(&player)
            .map_or(0, |s| s.total_buildings_built);
        let (production, consumption) = sim.world.player_energy.get(&player).map_or((0, 0), |e| {
            (e.get_production(sim.world.frame), e.get_consumption())
        });
        let _ = write!(text, "{built},{production},{consumption};");
    }
    text
}

// -----------------------------------------------------------------------------
// The harness's calls
// -----------------------------------------------------------------------------

fn order(sim: &mut SimContext, ids: &[ObjectId], cmd: AiCommandType, f: &[&str]) {
    let slot = sim.find_object(ids[number(f[1]) as usize]).expect("unit");
    let ms = sim.object(slot).and_then(|o| o.ai_slot).expect("AI");
    let ctx = sim.world.modules.meta(ms).context(ms);
    let source = if f[2] == "1" {
        CommandSourceType::FromPlayer
    } else {
        CommandSourceType::FromAi
    };
    let mut parms = AiCommandParms::new(cmd, source);
    parms.obj = usize::try_from(number(f[3])).ok().map(|i| ids[i]);
    if f[4] != "-" {
        let point = coord(&f[4].split(',').collect::<Vec<_>>());
        parms.pos = point;
        parms.coords = vec![point];
    }
    parms.int_value = NO_MAX_SHOTS_LIMIT;
    ai_update::execute(sim, ctx, &AiCall::Command(parms));
}

fn apply(sim: &mut SimContext, ids: &[ObjectId], op: &str) {
    use AiCommandType as C;
    let f: Vec<&str> = op.split(' ').collect();
    let command = match f[0] {
        "resume" => Some(C::ResumeConstruction),
        "repair" => Some(C::Repair),
        "move" => Some(C::MoveToPosition),
        "idle" => Some(C::Idle),
        "away" => Some(C::MoveAwayFromUnit),
        _ => None,
    };
    if let Some(command) = command {
        order(sim, ids, command, &f);
        return;
    }
    let slot = sim.find_object(ids[number(f[1]) as usize]).expect("object");
    let until = |sim: &SimContext, frames: &str| sim.world.frame + number(frames) as u32;
    match f[0] {
        "damage" => sim.body_internal_change_health(slot, -real(f[2])),
        "emp" => {
            let frame = until(sim, f[2]);
            sim.set_disabled_until(slot, DisabledType::Emp, frame);
        }
        "under" => {
            let frame = until(sim, f[2]);
            sim.set_disabled_until(slot, DisabledType::Underpowered, frame);
        }
        "unmanned" => sim.set_disabled(slot, DisabledType::Unmanned),
        "destroy" => sim.destroy_object(slot),
        other => panic!("unknown call {other}"),
    }
}

// -----------------------------------------------------------------------------
// The test
// -----------------------------------------------------------------------------

fn spawn(sim: &mut SimContext, text: &str) -> (usize, ObjectSlot) {
    let f: Vec<&str> = text.split(',').collect();
    let kind = number(f[0]) as usize;
    let template = sim
        .rules
        .thing_factory
        .find_template_id(TEMPLATES[kind])
        .expect("template");
    let team = TEAMS[1 + number(f[1]) as usize];
    let mode = number(f[6]);
    let mut status = ObjectStatusMaskType::new();
    if mode >= 2 {
        status.set(ObjectStatusType::UnderConstruction as Int, 1);
    }
    if mode == 3 {
        status.set(ObjectStatusType::Reconstructing as Int, 1);
    }
    // The reference creates the object at the origin, then places it.
    let slot = sim.factory_new_object_with_status(template, team, status);
    sim.set_object_orientation(slot, real(f[5]));
    sim.set_object_position(slot, &coord(&f[2..5]));
    if kind != DOZER && kind != WORKER && kind != UNIT {
        sim.add_object_to_pathfind_map(slot);
    }
    if mode >= 2 {
        sim.set_object_construction_percent(slot, 0.0);
        let health = sim.object_health(slot);
        sim.body_internal_change_health(slot, -health + 1.0);
        let mut old = ModelConditionFlags::new();
        let mut awaiting = ModelConditionFlags::new();
        old.set(ModelConditionFlagType::PartiallyConstructed as Int, 1);
        old.set(ModelConditionFlagType::ActivelyBeingConstructed as Int, 1);
        awaiting.set(ModelConditionFlagType::AwaitingConstruction as Int, 1);
        sim.clear_and_set_model_condition_flags(slot, &old, &awaiting);
    } else if mode == 1 {
        sim.body_internal_change_health(slot, -real(f[7]));
    }
    (kind, slot)
}

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
    for text in get_str(scene, "spawns").expect("spawns").split(';') {
        let (kind, slot) = spawn(&mut sim, text);
        let id = sim.object(slot).expect("spawned").id;
        ids.push(id);
        // Which weapons may be fired at what is not ported.
        if index % 4 == MINE_LAYOUT && kind != MINE {
            sim.world.attack_abilities.insert(
                id,
                AttackAbility {
                    able: TRUE,
                    specific_object: CanAttackResult::Possible,
                    weapon_against_target: CanAttackResult::Possible,
                },
            );
        }
    }
    check_tape(&tape, &format!("scene {index} spawns"));
    let probes = get_str(scene, "probes").expect("probes");
    for probe in probes.split(';').filter(|p| !p.is_empty()) {
        let f: Vec<&str> = probe.split(',').collect();
        let options = FindPositionOptions {
            flags: FPF_IGNORE_ALL_OBJECTS,
            max_radius: 20.0,
            start_angle: Some(real(f[3])),
            ..FindPositionOptions::default()
        };
        let found = sim.find_position_around(&coord(&f[0..3]), &options);
        assert_eq!(
            found.map_or_else(
                || "0,00000000,00000000,00000000".to_owned(),
                |p| format!("1,{}", c3(&p))
            ),
            f[4..].join(","),
            "scene {index}: findPositionAround {probe}"
        );
    }

    let mut seen = vec![Vec::new(); ids.len()];
    for row in frames {
        let frame = get_u32(row, "f").expect("frame");
        let ops = get_str(row, "ops").expect("ops");
        let context_text = format!("scene {index} frame {frame} [{ops}]");
        set_tape(&tape, get_str(row, "tape").expect("tape"));
        for op in ops.split(';').filter(|o| !o.is_empty()) {
            apply(&mut sim, &ids, op);
        }
        run_frame(&mut sim, &mut NoFrameSeams);
        check_tape(&tape, &context_text);
        let units = get_str(row, "u").expect("units");
        for (i, want) in units.split('~').enumerate() {
            merge(&mut seen[i], want);
            let got = observe(&mut sim, ids[i]);
            check(&context_text, i, &seen[i], &got);
        }
        assert_eq!(
            format!("{:08x}", sim.world.rng.get_game_logic_random_seed_crc()),
            get_str(row, "seed").expect("seed"),
            "{context_text}: random seed"
        );
        assert_eq!(
            players_text(&sim),
            get_str(row, "p").expect("players"),
            "{context_text}: players"
        );
        assert_eq!(
            cells_digest(&sim, cells),
            get_str(row, "map").expect("map"),
            "{context_text}: cells"
        );
    }
}

#[test]
fn dozer_and_worker_frames_match_the_reference() {
    let rows: Vec<&str> = VECTORS
        .lines()
        .filter(|l| l.starts_with("{\"s\":"))
        .collect();
    let cells = get_u32(VECTORS, "cells").expect("cells") as Int;
    let scenes = get_u32(VECTORS, "vector_count").expect("count") as usize;
    // A scene's first row has its spawns; the rows of its frames follow.
    let starts: Vec<usize> = (0..rows.len())
        .filter(|&i| get_str(rows[i], "spawns").is_some())
        .collect();
    assert!(scenes >= 48);
    assert_eq!(starts.len(), scenes);
    for (n, &start) in starts.iter().enumerate() {
        let end = starts.get(n + 1).copied().unwrap_or(rows.len());
        assert!(end - start > 200, "scene {n} is cut short");
        run(rows[start], &rows[start + 1..end], cells);
    }
}
