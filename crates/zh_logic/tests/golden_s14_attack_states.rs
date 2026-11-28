//! Slice S14 (first half) of `docs/AI_PORT_PLAN.md`: the attack states (attack,
//! force attack and attack-position orders, their pursue, approach, aim and fire
//! states, the weapon choice and the able-to-attack questions, and the attack
//! of the attack-move machine), against a trace of duels in the reference
//! (`tests/cpp_harness/gen_s14_attack_states.cpp`).
//!
//! Every frame of every scene is compared: the logic random seed, which unit
//! stands in and wants which ground cell, and for each unit its position,
//! heading, velocity, blocked counters, speed limits, path nodes, state, model
//! condition flags, victim, health, status bits, weapon save bytes, AI CRC and a
//! digest of its AI save bytes (which hold the attack machine and its current
//! state). The harness's calls (`ops`) are replayed, and the calls the reference
//! made to the searches and to targeting (`tape`) are checked in order and
//! answered from it.

use std::fmt::Write as _;
use std::sync::{Arc, Mutex};

use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::gamemath::real_to_int_floor;
use zh_common::math::{Coord3D, Matrix3D, Region3D};
use zh_common::types::{Bool, Int, ObjectId, Real, Relationship, TeamId, FALSE, TRUE};
use zh_common::xfer::{Snapshot, XferCrc, XferSave};
use zh_logic::ai::seams::{shared, AiSeams};
use zh_logic::combat::turret_ai::WhichTurretType;
use zh_logic::object::weapon_fire::launch_transform;
use zh_logic::arena::{ModuleContext, ModuleSlot, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::gamelogic::frame::{run_frame, NoFrameSeams};
use zh_logic::gamelogic::global_data::BodyDamageType;
use zh_logic::modules::update::ai_update::{self, AiCall, AiCommandParms, AiCommandType, AiUpdate};
use zh_logic::modules::update::physics_update;
use zh_logic::modules::AttitudeType;
use zh_logic::modules::CommandSourceType;
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::object::weapon::WeaponTemplate;
use zh_logic::object::weapon_set::{WeaponLockType, WeaponSlotType};
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;
use zh_logic::pathfinder::Path;
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::model_state::MODELCONDITION_COUNT;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::sim::{SimApi, SimContext, SimView};

const VECTORS: &str = include_str!("../../../docs/golden_vectors/S14_AttackStates.json");
/// The same with turreted units (`tests/cpp_harness/gen_s14_turrets.cpp`), where
/// idle units also look for targets as the reference does.
const TURRET_VECTORS: &str = include_str!("../../../docs/golden_vectors/S14_Turrets.json");

const BUNKER: usize = 11;
const TURRETLESS: usize = 12;
const HOVER: usize = 9;
const MODEL_CONDITIONS: Int = MODELCONDITION_COUNT as Int;
const STATUS_BITS: Int = 64;
/// The neutral side and the two that fight.
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

struct Seams {
    tape: SharedTape,
    real_idle_targets: bool,
}

impl Seams {
    fn call(&self, call: String) {
        self.tape.lock().expect("tape").made.push(call);
    }

    /// A call the reference answered: `asked=answer` on its tape.
    fn answer(&self, asked: String) -> Option<String> {
        let mut tape = self.tape.lock().expect("tape");
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
    fn next_mood_target(
        &mut self,
        owner: ObjectId,
        called_by_ai: Bool,
        during_idle: Bool,
    ) -> Option<Option<ObjectId>> {
        // The turretless reference's stand-in answers nothing, unrecorded, for an
        // idle unit.
        if during_idle != FALSE {
            return (!self.real_idle_targets).then_some(None);
        }
        let answer = self.answer(format!("T,{},{called_by_ai}", owner.0));
        let id = answer.map(|a| number(&a) as i32);
        Some(id.filter(|&id| id != 0).map(ObjectId))
    }
    fn replaces_searches(&mut self) -> Bool {
        TRUE
    }
    fn replaces_attack_path(&mut self) -> Bool {
        TRUE
    }
    fn attack_path(
        &mut self,
        owner: ObjectId,
        victim: Option<ObjectId>,
        victim_pos: &Coord3D,
    ) -> Option<Option<Path>> {
        let asked = format!(
            "A,{},{},{}",
            owner.0,
            victim.map_or(0, |id| id.0),
            c3(victim_pos)
        );
        let answered = self.tape.lock().expect("tape").expected.clone();
        let at = self.tape.lock().expect("tape").made.len();
        if answered.get(at) == Some(&format!("{asked}=F")) {
            self.call(format!("{asked}=F"));
            return None;
        }
        Some(self.search(asked))
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
    fn is_clear_line_of_sight(&self, a: &Coord3D, b: &Coord3D) -> Bool {
        Bool::from(self.is_cliff_cell((a.x + b.x) * 0.5, (a.y + b.y) * 0.5) == FALSE)
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

fn text(vectors: &str, key: &str) -> String {
    get_str(vectors, key).expect("text").replace("\\n", "\n")
}

fn world(vectors: &str, tape: &SharedTape, cells: Int, cliff: &str) -> SimContext {
    let mut sim = SimContext::new();
    for block in text(vectors, "locomotors").split_inclusive("End\n") {
        let mut t = LocomotorTemplate::new();
        t.parse_ini_block(block).expect("locomotor");
        t.name = block.split_whitespace().nth(1).expect("name").to_owned();
        sim.rules.locomotor_store.add_template(t);
    }
    for block in text(vectors, "weapons").split_inclusive("End\n") {
        let mut weapon = WeaponTemplate::new();
        weapon.parse_ini_block(block).expect("weapon");
        sim.rules.weapon_store.add_template(weapon);
    }
    let ini = text(vectors, "objects");
    let mut reader = IniReader::new("object.ini", ini.as_bytes(), IniLoadType::Overwrite);
    sim.rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");
    sim.rules.global_data.gravity = -1.0;
    sim.rules.global_data.partition_cell_size = 20.0;
    sim.rules.global_data.movement_penalty_damage_state = BodyDamageType::ReallyDamaged;
    sim.rules.global_data.unit_damaged_thresh = 0.7;
    sim.rules.global_data.unit_really_damaged_thresh = 0.35;
    sim.rules.ai_data.force_idle_frames_count = 45;
    sim.rules.ai_data.repulsed_distance = 50.0;
    sim.ai_seams = shared(Seams {
        tape: tape.clone(),
        real_idle_targets: vectors.len() == TURRET_VECTORS.len(),
    });
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

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut text, byte| {
        let _ = write!(text, "{byte:02x}");
        text
    })
}

/// Health, status bits and each weapon's save bytes.
fn combat_text(sim: &SimContext, slot: ObjectSlot) -> String {
    let mut text = format!("{}|", bits(sim.object_health(slot)));
    let record = sim.object(slot).expect("object");
    for bit in 0..STATUS_BITS {
        if record.status.test(bit) != FALSE {
            let _ = write!(text, "{bit}.");
        }
    }
    let set = &record.weapon_set;
    let _ = write!(text, "|{}", set.get_cur_weapon_slot().as_i32());
    for weapon in &set.weapons {
        text.push(',');
        if let Some(weapon) = weapon {
            let mut weapon = weapon.clone();
            let mut save = XferSave::new();
            Snapshot::xfer(&mut weapon, &mut save).expect("weapon save");
            text.push_str(&hex(save.as_bytes()));
        }
    }
    text
}

fn observe(sim: &mut SimContext, id: ObjectId) -> (String, String) {
    let none = String::new;
    let Some(slot) = sim.find_object(id) else {
        return ("gone".to_owned(), none());
    };
    let record = sim.object(slot).expect("object");
    let pos = c3(record.thing.get_position());
    let heading = bits(record.thing.get_orientation());
    let layer = record.layer as Int;
    let Some(ms) = record.ai_slot else {
        return (format!("{pos}|{}", combat_text(sim, slot)), none());
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
            (hex(save.as_bytes()), crc.get_crc())
        })
        .expect("dispatch");
    let attacking = sim
        .world
        .modules
        .payload(ms)
        .and_then(|m| m.as_ai())
        .map_or(FALSE, |a| a.is_attacking());
    let id_at = |slot: Option<ObjectSlot>| slot.and_then(|s| sim.object(s)).map_or(0, |o| o.id.0);
    let victim = format!(
        "{},{},{},{}",
        i32::from(attacking != FALSE),
        id_at(sim.ai_current_victim(slot)),
        id_at(sim.object_ai_goal_object(slot)),
        sim.ai_current_victim_pos(slot)
            .map_or_else(|| "-".to_owned(), |p| c3(&p))
    );
    let combat = combat_text(sim, slot);
    let ai = state(sim, ms);
    let seen = format!(
        "{pos}|{heading}|{velocity}|{}{}{}{}{}{}|{},{}|{},{},{}|{},{},{},{},{layer}|{path}|{flags}|{victim}|{combat}|{crc:08x}|{:016x}",
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
    );
    let turrets = turrets_text(ai);
    if turrets.is_empty() {
        return (seen, save);
    }
    (format!("{seen}|{turrets}"), save)
}

/// Each turret's angle, pitch and machine.
fn turrets_text(ai: &AiUpdate) -> String {
    let mut text = String::new();
    for (index, turret) in ai.turrets.iter().enumerate() {
        let Some(turret) = turret else { continue };
        let _ = write!(
            text,
            "/{index},{},{},{},{},{},{}",
            bits(turret.angle),
            bits(turret.pitch),
            turret.sm.get_current_state_id(),
            turret.sm.get_goal_object_id().0,
            c3(turret.sm.get_goal_position()),
            turret.sm.peek_sleep_till()
        );
    }
    text
}

/// The twelve words of each turreted unit's projectile launch transforms.
fn launch_text(sim: &mut SimContext, ids: &[ObjectId]) -> String {
    let mut rows = Vec::new();
    for (unit, &id) in ids.iter().enumerate() {
        let Some(slot) = sim.find_object(id) else {
            continue;
        };
        let Some(ms) = sim.object(slot).and_then(|o| o.ai_slot) else {
            continue;
        };
        if state(sim, ms).turrets[0].is_none() {
            continue;
        }
        for (index, weapon_slot) in [WeaponSlotType::Primary, WeaponSlotType::Secondary]
            .into_iter()
            .enumerate()
        {
            let matrix = launch_transform(sim, slot, weapon_slot, 0);
            let words: String = matrix.rows.iter().flatten().map(|&v| bits(v)).collect();
            rows.push(format!("{unit},{index},{words}"));
        }
    }
    rows.join(";")
}

const FIELDS: [&str; 15] = [
    "position",
    "heading",
    "velocity",
    "blocked frames, blocked, stuck, retry, waiting, moving",
    "blocked and bump speed",
    "path time, queue frame, ignore collisions",
    "goal cell, current cell, layer",
    "path",
    "state, model conditions",
    "attacking, victim, goal object, victim position",
    "health",
    "status",
    "weapons",
    "AI crc",
    "AI save digest",
];

fn check(context: &str, unit: usize, want: &str, got: &str, save: &str) {
    if want == got {
        return;
    }
    for (i, (w, g)) in want.split('|').zip(got.split('|')).enumerate() {
        let name = match want.split('|').count() {
            count if count >= FIELDS.len() => FIELDS.get(i).copied().unwrap_or("turrets"),
            _ => "?",
        };
        assert_eq!(g, w, "{context}: unit {unit}, {name}; AI save {save}");
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

/// `<name> <unit> <by player> <target or -1> <point or -> <shots>`.
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
    if f[4] != "-" {
        parms.pos = coord(&f[4].split(',').collect::<Vec<_>>());
        parms.coords = vec![parms.pos];
    }
    parms.int_value = number(f[5]) as Int;
    ai_update::execute(sim, ctx, &AiCall::Command(parms));
}

fn turret_of(f: &[&str]) -> WhichTurretType {
    WhichTurretType::from_i32(number(f[2]) as Int)
}

fn apply(sim: &mut SimContext, ids: &[ObjectId], op: &str, context_text: &str) {
    use AiCommandType as C;
    let f: Vec<&str> = op.split(' ').collect();
    let id = ids[number(f[1]) as usize];
    let slot = sim.find_object(id).expect("unit");
    let command = match f[0] {
        "attack" => Some(C::AttackObject),
        "force" => Some(C::ForceAttackObject),
        "attackpos" => Some(C::AttackPosition),
        "amove" => Some(C::AttackMoveToPosition),
        "move" => Some(C::MoveToPosition),
        "idle" => Some(C::Idle),
        _ => None,
    };
    if let Some(command) = command {
        let ms = sim.object(slot).and_then(|o| o.ai_slot).expect("AI");
        let ctx = context(sim, ms);
        order(sim, ids, ctx, command, &f);
        return;
    }
    match f[0] {
        "recenter" => sim.ai_recenter_turret(slot, turret_of(&f)),
        "turret" => sim.ai_set_turret_enabled(slot, turret_of(&f), Bool::from(f[3] == "1")),
        "pos" => {
            sim.set_object_position(slot, &coord(&f[2].split(',').collect::<Vec<_>>()));
            sim.set_object_orientation(slot, real(f[3]));
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
                -2 => AttitudeType::Sleep,
                2 => AttitudeType::Aggressive,
                _ => AttitudeType::Normal,
            };
            sim.ai_set_attitude(slot, attitude);
        }
        "lock" => {
            let set = &mut sim.object_mut(slot).expect("object").weapon_set;
            let weapon_slot = WeaponSlotType::from_i32(number(f[2]) as Int).expect("slot");
            match number(f[3]) {
                0 => set.release_weapon_lock(WeaponLockType::LockedPermanently),
                lock => {
                    let lock = WeaponLockType::from_i32(lock as Int).expect("lock");
                    set.set_weapon_lock(weapon_slot, lock);
                }
            }
        }
        "held" => {
            if f[2] == "1" {
                sim.set_disabled(slot, DisabledType::Held);
            } else {
                sim.clear_disabled(slot, DisabledType::Held);
            }
        }
        "kill" => {
            let mut damage = DamageInfo::default();
            damage.input.damage_type = DamageType::Unresistable;
            damage.input.death_type = DeathType::Normal;
            damage.input.amount = sim.object_max_health(slot);
            damage.input.kill = TRUE;
            sim.attempt_damage(slot, &mut damage);
        }
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

fn run(vectors: &str, scene: &str, frames: &[&str], names: &[&str], cells: Int) {
    let index = get_u32(scene, "s").expect("scene");
    let tape: SharedTape = Arc::default();
    let cliff = get_str(scene, "cliff").expect("cliff");
    let mut sim = world(vectors, &tape, cells, cliff);
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
            .find_template_id(names[kind])
            .expect("template");
        let team = TEAMS[match number(f[1]) {
            2 => 0,
            side => 1 + side as usize,
        }];
        // The reference creates the object at the origin, then places it.
        let slot = sim.spawn_object(template, team, Coord3D::default(), 0.0);
        sim.set_object_orientation(slot, real(f[5]));
        sim.set_object_position(slot, &coord(&f[2..5]));
        if kind == BUNKER || kind == TURRETLESS || names[kind] == "T_Base" {
            sim.add_object_to_pathfind_map(slot);
        }
        if kind == HOVER {
            sim.set_status(slot, ObjectStatusType::AirborneTarget);
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
        let seed = format!("{:08x}", sim.world.rng.get_game_logic_random_seed_crc());
        assert_eq!(
            seed,
            get_str(row, "seed").expect("seed"),
            "{context_text}: logic random seed"
        );
        assert_eq!(
            cells_digest(&sim, cells),
            get_str(row, "map").expect("map"),
            "{context_text}: cell units"
        );
        for (i, want) in get_str(row, "u").expect("units").split('~').enumerate() {
            if want != "=" {
                want.clone_into(&mut seen[i]);
            }
            let (got, save) = observe(&mut sim, ids[i]);
            check(&context_text, i, &seen[i], &got, &save);
        }
        if let Some(want) = get_str(row, "launch") {
            assert_eq!(
                launch_text(&mut sim, &ids),
                want,
                "{context_text}: launch transforms"
            );
        }
    }
}

#[test]
fn attack_states_match_the_reference() {
    check_trace(VECTORS);
}

#[test]
fn turrets_match_the_reference() {
    check_trace(TURRET_VECTORS);
}

fn check_trace(vectors: &str) {
    let cells = get_u32(vectors, "cells").expect("cells") as Int;
    let count = get_u32(vectors, "vector_count").expect("count");
    let frames = get_u32(vectors, "frames").expect("frames");
    let names_text = get_str(vectors, "names").expect("names");
    let names: Vec<&str> = names_text.split(',').collect();
    let mut lines = vectors.lines().skip(1).filter(|l| l.starts_with('{'));
    let mut scenes = 0;
    while let Some(scene) = lines.next() {
        let scene = scene.trim_end_matches(',');
        let rows: Vec<&str> = lines
            .by_ref()
            .take(frames as usize)
            .map(|l| l.trim_end_matches(','))
            .collect();
        assert_eq!(rows.len(), frames as usize);
        run(vectors, scene, &rows, &names, cells);
        scenes += 1;
    }
    assert_eq!(scenes, count);
}
