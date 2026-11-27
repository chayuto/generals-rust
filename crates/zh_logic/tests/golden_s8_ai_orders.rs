//! Slice S8 of `docs/AI_PORT_PLAN.md`: the AI module shell, its orders, state
//! machine and idle against the reference (`tests/cpp_harness/gen_s8_ai_orders.cpp`).
//!
//! Every step of every vector is compared: the logic random seed, and for each
//! unit the AI's `XferSave` bytes and `XferCRC`, position, velocity and flags. The
//! seams are answered from the tape of the reference's calls, which also fails
//! on a call out of order or with other arguments. The reference saves version 4
//! (`RETAIL_COMPATIBLE_XFER_SAVE`), so the version is asked for explicitly.

use std::collections::VecDeque;
use std::fmt::Write as _;
use std::sync::{Arc, Mutex};

use zh_common::core::state_machine::{StateExitType, StateId, StateReturnType};
use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord3D, ICoord2D, Region3D};
use zh_common::types::{Bool, Int, ObjectId, Real, Relationship, TeamId, UpdateSleepTime, TRUE};
use zh_common::xfer::{Xfer, XferCrc, XferError, XferSave};
use zh_logic::ai::seams::{shared, AiActionQuery, AiSeams, PathServices};
use zh_logic::arena::{ModuleContext, ModuleSlot, ObjectSlot};
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::combat::locomotor_set::LocomotorSetType;
use zh_logic::gamelogic::global_data::BodyDamageType;
use zh_logic::modules::module::AttitudeType;
use zh_logic::modules::update::ai_update::{
    self, AiCall, AiCommandParms, AiCommandType, AiUpdate, MoodMatrixAction,
};
use zh_logic::modules::update::physics_update;
use zh_logic::modules::CommandSourceType;
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;
use zh_logic::rules::combat_types::LocomotorSurfaceTypeMask;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::sim::{SimApi, SimContext, SimView};

const VECTORS: &str = include_str!("../../../docs/golden_vectors/S8_AiOrders.json");

/// Neutral, A, B.
const TEAMS: [TeamId; 3] = [TeamId(3), TeamId(1), TeamId(2)];
const TEMPLATES: [&str; 9] = [
    "S8_Inf",
    "S8_Veh",
    "S8_Air",
    "S8_Fixed",
    "S8_Shy",
    "S8_Proj",
    "S8_Repulsed",
    "S8_Quiet",
    "S8_Crate",
];
const COMMANDS: [AiCommandType; 46] = {
    use AiCommandType as C;
    [
        C::MoveToPosition,
        C::MoveToPositionEvenIfSleeping,
        C::MoveToObject,
        C::TightenToPosition,
        C::MoveToPositionAndEvacuate,
        C::MoveToPositionAndEvacuateAndExit,
        C::Idle,
        C::FollowWaypointPath,
        C::FollowWaypointPathAsTeam,
        C::FollowWaypointPathExact,
        C::FollowWaypointPathAsTeamExact,
        C::FollowPath,
        C::FollowPathAppend,
        C::FollowExitProductionPath,
        C::AttackObject,
        C::ForceAttackObject,
        C::GuardRetaliate,
        C::AttackPosition,
        C::AttackMoveToPosition,
        C::AttackFollowWaypointPath,
        C::AttackFollowWaypointPathAsTeam,
        C::Hunt,
        C::AttackArea,
        C::Repair,
        C::ResumeConstruction,
        C::GetHealed,
        C::GetRepaired,
        C::Enter,
        C::Dock,
        C::Exit,
        C::ExitInstantly,
        C::Evacuate,
        C::EvacuateInstantly,
        C::ExecuteRailedTransport,
        C::GuardPosition,
        C::GuardObject,
        C::GuardTunnelNetwork,
        C::HackInternet,
        C::FaceObject,
        C::FacePosition,
        C::RappelInto,
        C::CombatDrop,
        C::Wander,
        C::WanderInPlace,
        C::Panic,
        C::Busy,
    ]
};

// -----------------------------------------------------------------------------
// The tape
// -----------------------------------------------------------------------------

#[derive(Default)]
struct Tape {
    entries: VecDeque<String>,
    context: String,
}

impl Tape {
    /// The next call must be `call`; returns what the reference answered.
    fn take(&mut self, call: String) -> String {
        let Some(entry) = self.entries.pop_front() else {
            panic!("{}: unexpected call {call}", self.context);
        };
        let (want, answer) = entry.split_once('=').unwrap_or((&entry, ""));
        assert_eq!(call, want, "{}: call", self.context);
        answer.to_owned()
    }
}

type SharedTape = Arc<Mutex<Tape>>; // determinism-ok: the harness log

struct Seams(SharedTape);

fn bits(value: Real) -> String {
    format!("{:08x}", value.to_bits())
}
fn real(text: &str) -> Real {
    Real::from_bits(u32::from_str_radix(text, 16).expect("float bits"))
}
fn c3(p: &Coord3D) -> String {
    format!("{},{},{}", bits(p.x), bits(p.y), bits(p.z))
}
fn coord(text: &str) -> Coord3D {
    let mut it = text.split(',').map(real);
    Coord3D::new(
        it.next().expect("x"),
        it.next().expect("y"),
        it.next().expect("z"),
    )
}
fn number(text: &str) -> i64 {
    text.parse().expect("number")
}
fn object(text: &str) -> Option<ObjectId> {
    let id = number(text) as i32;
    (id != 0).then_some(ObjectId(id))
}
fn state_return(code: &str) -> StateReturnType {
    match number(code) {
        0 => StateReturnType::Continue,
        -1 => StateReturnType::Success,
        -2 => StateReturnType::Failure,
        frames => StateReturnType::Sleep(frames as u32),
    }
}

impl Seams {
    fn take(&mut self, call: String) -> String {
        self.0.lock().expect("tape").take(call)
    }
}

impl AiSeams for Seams {
    fn valid_movement_terrain(
        &mut self,
        layer: PathfindLayerEnum,
        surfaces: LocomotorSurfaceTypeMask,
        pos: &Coord3D,
    ) -> Bool {
        let a = self.take(format!("V,{},{surfaces},{}", layer as i32, c3(pos)));
        number(&a) as Bool
    }

    fn ground_or_structure_height(&mut self, x: Real, y: Real) -> Real {
        real(&self.take(format!("H,{},{}", bits(x), bits(y))))
    }

    fn choose_best_locomotor(
        &mut self,
        _owner: ObjectId,
        layer: PathfindLayerEnum,
        count: usize,
        pos: &Coord3D,
    ) -> Option<usize> {
        let a = self.take(format!("L,{},{count},{}", layer as i32, c3(pos)));
        usize::try_from(number(&a)).ok()
    }

    fn update_goal(&mut self, owner: ObjectId, pos: &Coord3D, layer: PathfindLayerEnum) {
        self.take(format!("G,{},{},{}", owner.0, c3(pos), layer as i32));
    }

    fn update_pos(&mut self, owner: ObjectId, pos: &Coord3D) {
        self.take(format!("O,{},{}", owner.0, c3(pos)));
    }

    fn goal_position(&mut self, owner: ObjectId) -> Option<Coord3D> {
        let a = self.take(format!("P,{}", owner.0));
        a.strip_prefix("1,").map(coord)
    }

    fn snap_position(&mut self, owner: ObjectId, pos: Coord3D) -> Coord3D {
        coord(&self.take(format!("N,{},{}", owner.0, c3(&pos))))
    }

    /// The reference harness replaced the movement states too.
    fn replaces_state(&mut self, _owner: ObjectId, _state: StateId) -> Bool {
        TRUE
    }

    fn state_enter(&mut self, owner: ObjectId, state: StateId) -> StateReturnType {
        state_return(&self.take(format!("E,{},{state}", owner.0)))
    }

    fn state_update(&mut self, owner: ObjectId, state: StateId) -> StateReturnType {
        state_return(&self.take(format!("U,{},{state}", owner.0)))
    }

    fn state_exit(&mut self, owner: ObjectId, state: StateId, status: StateExitType) {
        self.take(format!("X,{},{state},{}", owner.0, status as i32));
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

    fn next_mood_target(
        &mut self,
        owner: ObjectId,
        by_ai: Bool,
        during_idle: Bool,
    ) -> Option<ObjectId> {
        object(&self.take(format!("M,{},{by_ai},{during_idle}", owner.0)))
    }

    fn closest_repulsor(&mut self, owner: ObjectId, range: Real) -> Option<ObjectId> {
        object(&self.take(format!("R,{},{}", owner.0, bits(range))))
    }

    fn can_act_on(
        &mut self,
        query: AiActionQuery,
        owner: ObjectId,
        target: ObjectId,
        source: CommandSourceType,
    ) -> Bool {
        let code = match query {
            AiActionQuery::GetHealedAt => 0,
            AiActionQuery::GetRepairedAt => 1,
            AiActionQuery::Enter => 2,
        };
        let a = self.take(format!(
            "A,{code},{},{},{}",
            owner.0, target.0, source as i32
        ));
        number(&a) as Bool
    }

    fn closest_edge_point(&mut self, pos: &Coord3D) -> Coord3D {
        coord(&self.take(format!("D,{}", c3(pos))))
    }
}

struct Queue(SharedTape);

impl PathServices for Queue {
    fn queue_for_path(&mut self, id: ObjectId) -> Bool {
        self.0.lock().expect("tape").take(format!("Q,{}", id.0));
        TRUE
    }
    fn begin_queue(&mut self) {}
    fn next_request(&mut self) -> Option<ObjectId> {
        None
    }
}

// -----------------------------------------------------------------------------
// The world
// -----------------------------------------------------------------------------

fn text(key: &str) -> String {
    get_str(VECTORS, key).expect("text").replace("\\n", "\n")
}

fn world(tape: &SharedTape, humans: &str) -> SimContext {
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
    sim.ai_seams = shared(Seams(tape.clone()));
    sim.paths = Box::new(Queue(tape.clone()));
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
    for (team, human) in TEAMS[1..].iter().zip(humans.chars()) {
        w.player_is_human.insert(*team, Bool::from(human == '1'));
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
    sim
}

struct Unit {
    id: ObjectId,
    /// The line the reference last observed for this unit.
    seen: String,
}

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

fn observe(sim: &mut SimContext, id: ObjectId) -> String {
    let Some(slot) = sim.find_object(id) else {
        return "gone".to_owned();
    };
    let record = sim.object(slot).expect("object");
    let pos = c3(record.thing.get_position());
    let airborne = record
        .status
        .test(ObjectStatusType::AirborneTarget.as_i32())
        != 0;
    let dead = record.is_effectively_dead();
    let Some(ms) = record.ai_slot else {
        return format!("p{pos}");
    };
    let velocity =
        physics_update::physics_mut(sim, slot).map_or_else(|| "-".to_owned(), |p| c3(&p.vel));
    let (save, crc, flags) = sim
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
            let flags = (module.is_idle(), module.is_moving());
            (hex, crc.get_crc(), flags)
        })
        .expect("dispatch");
    let (idle, moving) = flags;
    let ctx = context(sim, ms);
    let mut mood = format!("{:08x}", ai_update::mood_matrix_value(sim, ctx));
    for action in [
        MoodMatrixAction::Idle,
        MoodMatrixAction::Move,
        MoodMatrixAction::Attack,
        MoodMatrixAction::AttackMove,
    ] {
        let value = ai_update::mood_matrix_action_adjustment(sim, ctx, action);
        let _ = write!(mood, ".{value:08x}");
    }
    format!(
        "{crc:08x},{mood},{pos},{},{idle}{moving}{dead},{velocity},{save}",
        i32::from(airborne)
    )
}

/// Compares one observation field by field.
fn check(context: &str, unit: usize, want: &str, got: &str) {
    if want == got {
        return;
    }
    const NAMES: [&str; 9] = [
        "crc",
        "mood",
        "x",
        "y",
        "z",
        "airborne",
        "idle/moving/dead",
        "velocity x",
        "velocity y",
    ];
    for (i, (w, g)) in want.split(',').zip(got.split(',')).enumerate() {
        if w != g {
            let at = w
                .bytes()
                .zip(g.bytes())
                .position(|(a, b)| a != b)
                .unwrap_or(w.len().min(g.len()));
            panic!(
                "{context}: unit {unit} field {} ({}) differs at {at} (byte {}):\n want {w}\n got  {g}",
                i,
                NAMES.get(i).unwrap_or(&"velocity z or save"),
                at / 2
            );
        }
    }
    panic!("{context}: unit {unit}:\n want {want}\n got  {got}");
}

fn apply(sim: &mut SimContext, units: &[Unit], op: &str, context_text: &str) {
    let mut f = op.split(' ');
    let code = f.next().expect("op");
    let v: Vec<&str> = f.collect();
    let n = |i: usize| number(v[i]);
    if code == "f" {
        sim.world.frame += n(0) as u32;
        return;
    }
    let id = units[n(0) as usize].id;
    let other = |i: usize| usize::try_from(n(i)).ok().map(|k| units[k].id);
    let (slot, ms) = ai_module(sim, id).expect("unit with AI");
    let ctx = context(sim, ms);
    let call = |sim: &mut SimContext, call: AiCall| ai_update::execute(sim, ctx, &call);
    match code {
        "c" => {
            let source = match n(2) {
                0 => CommandSourceType::FromPlayer,
                1 => CommandSourceType::FromScript,
                _ => CommandSourceType::FromAi,
            };
            let mut parms = AiCommandParms::new(COMMANDS[n(1) as usize], source);
            parms.obj = other(3);
            parms.pos = coord(v[4]);
            parms.int_value = n(5) as Int;
            parms.coords = v[7..].iter().map(|p| coord(p)).collect();
            call(sim, AiCall::Command(parms));
        }
        "t" => {
            let entries = sim
                .world
                .modules
                .payload(ms)
                .and_then(|m| m.resident())
                .expect("resident");
            let sleep: UpdateSleepTime = (entries.update)(sim, ctx);
            assert_eq!(sleep.0, n(1) as u32, "{context_text}: sleep time");
            sim.set_wake_frame(ms, sleep);
        }
        "dead" => sim.ai_mark_as_dead(slot),
        "loco" => {
            sim.ai_choose_locomotor_set(slot, LocomotorSetType::from_i32(n(1) as Int).expect("set"))
        }
        "upg" => call(sim, AiCall::SetLocomotorUpgrade(n(1) as Bool)),
        "att" => sim.ai_set_attitude(
            slot,
            match n(1) {
                -2 => AttitudeType::Sleep,
                -1 => AttitudeType::Passive,
                0 => AttitudeType::Normal,
                1 => AttitudeType::Alert,
                _ => AttitudeType::Aggressive,
            },
        ),
        "ign" => {
            let target = other(1).expect("target");
            sim.ai_ignore_obstacle(slot, target);
        }
        "wq" => {
            let pos = coord(v[1]);
            let module = state(sim, ms);
            for _ in 0..n(2) {
                module.queue_waypoint(&pos);
            }
            if n(3) != 0 {
                module.execute_waypoint_queue();
            }
        }
        "dis" => {
            let record = sim.object_mut(slot).expect("object");
            record.disabled_mask.set(n(1) as Int, n(2) as Int);
        }
        "move" => {
            if n(1) != 0 {
                state(sim, ms).ending_move();
            } else {
                state(sim, ms).starting_move();
            }
        }
        "qp" => ai_update::set_queue_for_path_time(sim, ctx, n(1) as u32),
        "gp" => state(sim, ms).set_goal_position_explicit(&coord(v[1])),
        "ga" => state(sim, ms).set_goal_orientation(real(v[1])),
        "gn" => ai_update::set_locomotor_goal_none(sim, ctx),
        "speed" => state(sim, ms).desired_speed = real(v[1]),
        "join" => call(sim, AiCall::JoinTeam),
        "wake" => sim.ai_wake_up_and_attempt_to_target(slot),
        "goalobj" => call(sim, AiCall::SetGoalObject(other(1))),
        "quick" => call(
            sim,
            AiCall::QuickExit(v[2..].iter().map(|p| coord(p)).collect()),
        ),
        "destroy" => state(sim, ms).destroy_path(),
        "notify" => {
            let target = other(1).expect("crate");
            state(sim, ms).crate_created = target;
        }
        "pos" => {
            sim.set_object_position(slot, &coord(v[1]));
            let velocity = real(v[2]);
            if let Some(p) = physics_update::physics_mut(sim, slot) {
                p.vel.x = velocity;
                p.vel.y = velocity * 0.5;
            }
        }
        "load" => {
            state(sim, ms).pathfind_goal_cell = ICoord2D {
                x: n(1) as Int,
                y: n(2) as Int,
            };
            call(sim, AiCall::LoadPostProcess);
        }
        _ => panic!("unknown op {op}"),
    }
}

fn run(line: &str) {
    let index = get_u32(line, "i").expect("i");
    let tape: SharedTape = Arc::default();
    let humans = get_str(line, "humans").expect("humans");
    let mut sim = world(&tape, &humans);
    sim.world.frame = get_u32(line, "frame").expect("frame");
    sim.world
        .rng
        .init_random_with_seed(get_u32(line, "seed").expect("seed"));
    sim.world.next_object_id = 1000;

    let created = get_str(line, "created").expect("created");
    let mut units = Vec::new();
    for (spawn, calls) in get_str(line, "spawns")
        .expect("spawns")
        .split(';')
        .zip(created.split('|'))
    {
        let f: Vec<&str> = spawn.split(',').collect();
        let tid = sim
            .rules
            .thing_factory
            .find_template_id(TEMPLATES[number(f[0]) as usize])
            .expect("template");
        {
            let mut t = tape.lock().expect("tape");
            t.context = format!("vector {index} spawn {}", units.len());
            t.entries = calls
                .split(';')
                .filter(|c| !c.is_empty())
                .map(str::to_owned)
                .collect();
        }
        // The reference creates the object at the origin, then places it.
        let team = TEAMS[1 + number(f[1]) as usize];
        let slot = sim.spawn_object(tid, team, Coord3D::default(), 0.0);
        sim.set_object_position(slot, &Coord3D::new(real(f[2]), real(f[3]), real(f[4])));
        sim.set_object_orientation(slot, real(f[5]));
        let t = tape.lock().expect("tape");
        assert!(
            t.entries.is_empty(),
            "{}: calls not made: {:?}",
            t.context,
            t.entries
        );
        units.push(Unit {
            id: sim.object(slot).expect("spawned").id,
            seen: String::new(),
        });
    }

    let steps = get_str(line, "steps").expect("steps");
    for (number_of_step, step) in steps.split('#').enumerate() {
        let mut f = step.split('~');
        let op = f.next().expect("op");
        let calls = f.next().expect("tape");
        let seed = f.next().expect("seed");
        let context_text = format!("vector {index} step {number_of_step} ({op}) [{calls}]");
        {
            let mut t = tape.lock().expect("tape");
            t.context = context_text.clone();
            t.entries = calls
                .split(';')
                .filter(|c| !c.is_empty())
                .map(str::to_owned)
                .collect();
        }
        if op != "init" {
            apply(&mut sim, &units, op, &context_text);
        }
        {
            let t = tape.lock().expect("tape");
            assert!(
                t.entries.is_empty(),
                "{context_text}: calls not made: {:?}",
                t.entries
            );
        }
        assert_eq!(
            format!("{:08x}", sim.world.rng.get_game_logic_random_seed_crc()),
            seed,
            "{context_text}: random seed"
        );
        for (i, (unit, want)) in units.iter_mut().zip(f).enumerate() {
            if want != "=" {
                unit.seen = want.to_owned();
            }
            let got = observe(&mut sim, unit.id);
            check(&context_text, i, &unit.seen, &got);
        }
    }
}

#[test]
fn ai_orders_match_the_reference() {
    let lines: Vec<&str> = VECTORS
        .lines()
        .filter(|l| l.starts_with("{\"i\":"))
        .collect();
    assert_eq!(
        lines.len() as u32,
        get_u32(VECTORS, "vector_count").expect("count")
    );
    assert!(lines.len() >= 200);
    for line in lines {
        run(line);
    }
}

/// Records whose state is saved.
struct OwnerSeams(Arc<Mutex<Vec<ObjectId>>>); // determinism-ok: the harness log

impl AiSeams for OwnerSeams {
    fn replaces_state(&mut self, _owner: ObjectId, _state: StateId) -> Bool {
        TRUE
    }

    fn state_xfer(
        &mut self,
        owner: ObjectId,
        _state: StateId,
        _xfer: &mut dyn Xfer,
    ) -> Result<(), XferError> {
        self.0.lock().expect("log").push(owner);
        Ok(())
    }
}

/// The `ai_*` orders of the world reach the module's command entry. The reference
/// for what each command does is the trace above.
#[test]
fn world_orders_reach_the_module() {
    use zh_logic::ai::ai_state_machine::state_id as s;
    use zh_logic::ai::seams::NoAiSeams;
    let tape: SharedTape = Arc::default();
    let mut sim = world(&tape, "00");
    sim.ai_seams = shared(NoAiSeams);
    sim.paths = Box::new(zh_logic::ai::seams::NoPathServices);
    sim.world.frame = 10;
    let spawn = |sim: &mut SimContext, name: &str, team: usize| {
        let tid = sim
            .rules
            .thing_factory
            .find_template_id(name)
            .expect("template");
        sim.spawn_object(tid, TEAMS[team], Coord3D::new(100.0, 100.0, 0.0), 0.0)
    };
    let unit = spawn(&mut sim, "S8_Veh", 1);
    let other = spawn(&mut sim, "S8_Veh", 1);
    let ms = sim.object(unit).and_then(|o| o.ai_slot).expect("AI");
    let other_id = sim.object(other).expect("object").id;
    let look = |sim: &mut SimContext| {
        let source = sim.ai_last_command_source(unit);
        let m = state(sim, ms);
        (
            m.machine.current_state_id(),
            source,
            m.machine.goal_object_id,
        )
    };
    let source = CommandSourceType::FromScript;
    let none = ObjectId::INVALID;
    let pos = Coord3D::new(200.0, 220.0, 0.0);
    assert_eq!(look(&mut sim).0, s::IDLE);
    assert!(sim.ai_has_cur_locomotor(unit) != 0);
    assert!(sim.cur_locomotor(unit).is_some());
    assert!(sim.ai_cur_locomotor_preferred_height(unit).is_some());

    sim.ai_busy(unit, source);
    assert_eq!(look(&mut sim), (s::BUSY, source, none));
    assert_eq!(sim.ai_is_idle(unit), 0);
    sim.ai_idle(unit, CommandSourceType::FromPlayer);
    assert_eq!(
        look(&mut sim),
        (s::IDLE, CommandSourceType::FromPlayer, none)
    );
    sim.ai_move_to_object(unit, other, source);
    assert_eq!(look(&mut sim), (s::MOVE_TO, source, other_id));
    assert_eq!(sim.object_ai_goal_object(unit), Some(other));
    sim.ai_clear_goal_object(unit);
    assert_eq!(sim.object_ai_goal_object(unit), None);
    sim.ai_attack_object(unit, other, 3, source);
    assert_eq!(look(&mut sim), (s::ATTACK_OBJECT, source, other_id));
    assert_eq!(
        sim.world
            .modules
            .payload(ms)
            .and_then(|m| m.as_ai())
            .map(|a| a.is_attacking()),
        Some(TRUE)
    );
    sim.ai_force_attack_object(unit, other, 3, source);
    assert_eq!(look(&mut sim), (s::FORCE_ATTACK_OBJECT, source, other_id));
    sim.ai_face_object(unit, other, source);
    assert_eq!(look(&mut sim), (s::FACE_OBJECT, source, other_id));
    sim.ai_face_position(unit, &pos, source);
    assert_eq!(look(&mut sim).0, s::FACE_POSITION);
    assert_eq!(state(&mut sim, ms).machine.goal_position, pos);
    sim.ai_attack_position(unit, &pos, 1, source);
    assert_eq!(look(&mut sim).0, s::ATTACK_POSITION);
    sim.ai_hunt(unit, source);
    assert_eq!(look(&mut sim).0, s::HUNT);
    sim.ai_move_to_position(unit, &pos, source);
    assert_eq!(look(&mut sim).0, s::MOVE_TO);
    sim.ai_follow_path(unit, &[pos, pos], Some(other), source);
    assert_eq!(look(&mut sim).0, s::FOLLOW_PATH);
    assert_eq!(state(&mut sim, ms).machine.goal_path.len(), 2);
    assert_eq!(sim.ai_ignored_obstacle_id(unit), other_id);
    sim.ai_follow_exit_production_path(unit, &[pos], None, source);
    assert_eq!(look(&mut sim).0, s::FOLLOW_EXITPRODUCTION_PATH);
    assert_eq!(state(&mut sim, ms).machine.goal_path.len(), 1);
    assert_eq!(sim.world.exit_path_orders.len(), 1);
    // Without an action manager these are refused.
    sim.ai_idle(unit, source);
    sim.ai_enter(unit, other, source);
    sim.ai_get_healed(unit, other, source);
    assert_eq!(look(&mut sim).0, s::IDLE);
    sim.ai_hack_internet(unit, source);
    assert_eq!(
        look(&mut sim).0,
        zh_common::core::state_machine::INVALID_STATE_ID
    );
    sim.ai_choose_locomotor_set(unit, LocomotorSetType::Wander);
    assert_eq!(
        state(&mut sim, ms).cur_locomotor_set,
        LocomotorSetType::Wander
    );
    sim.ai_choose_locomotor_set(unit, LocomotorSetType::Normal);
    sim.set_locomotor_set(unit, LocomotorSetType::NormalUpgraded);
    assert_eq!(
        state(&mut sim, ms).cur_locomotor_set,
        LocomotorSetType::NormalUpgraded
    );
    sim.world.frame = 20;
    sim.ai_idle(unit, source);
    sim.ai_wake_up_and_attempt_to_target(unit);
    assert_eq!(state(&mut sim, ms).next_mood_check_time, 20);
    // `doPathfind` takes the request and ends in `wakeUpNow`.
    let entries = sim
        .world
        .modules
        .payload(ms)
        .and_then(|m| m.resident())
        .expect("resident");
    let ctx = context(&sim, ms);
    state(&mut sim, ms).waiting_for_path = TRUE;
    sim.set_wake_frame(ms, UpdateSleepTime::FOREVER);
    (entries.do_pathfind)(&mut sim, ctx);
    assert_eq!(state(&mut sim, ms).waiting_for_path, 0);
    assert_eq!(sim.get_wake_frame(ms), UpdateSleepTime::NONE);
    // A request that is deferred stays and does not wake.
    state(&mut sim, ms).waiting_for_path = TRUE;
    state(&mut sim, ms).queue_for_path_frame = 99;
    sim.set_wake_frame(ms, UpdateSleepTime::FOREVER);
    (entries.do_pathfind)(&mut sim, ctx);
    assert_eq!(state(&mut sim, ms).waiting_for_path, TRUE);
    assert_ne!(sim.get_wake_frame(ms), UpdateSleepTime::NONE);

    // A module's own `xfer` passes its object's id to the seams.
    let owner_id = sim.object(unit).expect("object").id;
    let asked: Arc<Mutex<Vec<ObjectId>>> = Arc::default(); // determinism-ok: the harness log
    sim.ai_seams = shared(OwnerSeams(asked.clone()));
    sim.ai_hunt(unit, source);
    sim.dispatch_module(ms, |m, sim, _| {
        m.xfer(&mut XferSave::new(), sim).expect("save");
    })
    .expect("dispatch");
    assert_eq!(*asked.lock().expect("log"), vec![owner_id]);
    sim.ai_seams = shared(NoAiSeams);

    // The setter other code calls through the facet.
    state(&mut sim, ms).set_goal_orientation(1.0);
    sim.dispatch_module(ms, |m, sim, ctx| {
        m.as_ai_mut()
            .expect("facet")
            .set_locomotor_goal_none(sim, ctx);
    })
    .expect("dispatch");
    assert_eq!(
        state(&mut sim, ms).locomotor_goal_type,
        ai_update::LocoGoalType::None
    );

    // An exit order reaches the module too.
    sim.ai_idle(unit, source);
    sim.object_mut(unit).expect("object").contained_by = Some(other);
    sim.order_ai_exit(unit, other, source);
    assert_eq!(look(&mut sim), (s::EXIT, source, other_id));
    sim.object_mut(unit).expect("object").contained_by = None;
    sim.ai_idle(unit, source);

    // An object without an AI keeps its goal on the record.
    let bare = spawn(&mut sim, "S8_Crate", 1);
    sim.object_mut(bare).expect("object").ai_goal_object = Some(other);
    assert_eq!(sim.object_ai_goal_object(bare), Some(other));
    sim.ai_clear_goal_object(bare);
    assert_eq!(sim.object_ai_goal_object(bare), None);

    sim.ai_mark_as_dead(unit);
    assert_eq!(sim.object(unit).expect("object").ai_dead, TRUE);
    assert_eq!(sim.ai_is_in_dead_state(unit), TRUE);
    assert!(sim.world.ai_commands.iter().any(|(_, c)| c == "aiHunt"));
}
