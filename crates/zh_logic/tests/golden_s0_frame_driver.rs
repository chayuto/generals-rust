//! Slice S0 of `docs/AI_PORT_PLAN.md`: [`run_frame`] against a trace of the
//! reference `GameLogic::update` (`tests/cpp_harness/gen_s0_frame_driver.cpp`).
//!
//! The unported parts of the frame are answered here the way the harness watched
//! them: every [`FrameSeams`] call records the state, and the path queue is a
//! tape that holds the ids and serves as many per frame as the reference did. The
//! AI is a resident module with no behaviour that follows each vector's plan.

use std::any::Any;
use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::fmt::Write as _;
use std::sync::{Arc, Mutex};

use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniError, IniLoadType, IniReader};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, Relationship, TeamId, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferCrc, XferError, XferSave};
use zh_logic::ai::seams::PathServices;
use zh_logic::ai::ResidentEntries;
use zh_logic::arena::{FacetMask, ModuleContext, ModuleSlot, ObjectSlot, ThingTemplateId};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::gamelogic::frame::{run_frame, FrameSeams};
use zh_logic::modules::module::AiUpdateFacet;
use zh_logic::modules::module_factory::ModuleParseContext;
use zh_logic::modules::{BehaviorModule, CommandSourceType, ModuleData, ModuleType};
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::rules::disabled_types::{DisabledMaskType, DisabledType};
use zh_logic::scheduler::{SleepyUpdatePhase, UPDATE_SLEEP_FOREVER};
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimView};

const VECTORS: &str = include_str!("../../../docs/golden_vectors/S0_FrameDriver.json");

/// Neutral, A, B: the team of player 0, 1 and 2.
const TEAMS: [TeamId; 3] = [TeamId(3), TeamId(1), TeamId(2)];
const TEMPLATES: [&str; 6] = [
    "S0_Ai",
    "S0_Crate",
    "S0_Fader",
    "S0_Healer",
    "S0_Dier",
    "S0_Spy",
];
const DISABLED: [DisabledType; 6] = [
    DisabledType::Paralyzed,
    DisabledType::Emp,
    DisabledType::Held,
    DisabledType::Hacked,
    DisabledType::Underpowered,
    DisabledType::Default,
];
const MODULES: [&str; 9] = [
    "ActiveBody",
    "PhysicsBehavior",
    "LifetimeUpdate",
    "DeletionUpdate",
    "AutoHealBehavior",
    "SlowDeathBehavior",
    "DestroyDie",
    "AIUpdateInterface",
    "SpyVisionUpdate",
];
const AI_MASK: FacetMask = FacetMask(FacetMask::UPDATE.0 | FacetMask::AI_UPDATE.0);

// -----------------------------------------------------------------------------
// The recording AI
// -----------------------------------------------------------------------------

#[derive(Default)]
struct Shared {
    out: String,
    /// The scenario's objects in spawn order.
    ids: Vec<ObjectId>,
    /// The template of each kind in [`TEMPLATES`].
    templates: Vec<ThingTemplateId>,
}

thread_local! {
    static SHARED: RefCell<Shared> = RefCell::default(); // determinism-ok: the harness log
}

fn emit(text: &str) {
    SHARED.with(|s| s.borrow_mut().out.push_str(text));
}

#[derive(Clone, Default)]
struct Plan {
    sleep: u32,
    actions: Vec<(u32, u32, u32)>,
}

struct Ai {
    key: NameKeyType,
    id: ObjectId,
    idle: Bool,
    waiting: Bool,
    plan: Option<Plan>,
}

impl AiUpdateFacet for Ai {
    fn is_idle(&self) -> Bool {
        self.idle
    }

    fn ai_attack_position(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        pos: &Coord3D,
        max_shots: Int,
        _src: CommandSourceType,
    ) {
        self.idle = FALSE;
        emit(&format!(
            "[o {} {} {max_shots}]",
            self.id.0,
            pos.x.to_bits()
        ));
    }
}

impl BehaviorModule for Ai {
    fn module_name_key(&self) -> NameKeyType {
        self.key
    }

    fn resident(&self) -> Option<ResidentEntries> {
        let mut held = DisabledMaskType::new();
        held.set(DisabledType::Held.as_i32(), 1);
        Some(ResidentEntries {
            disabled_types_to_process: held,
            update: ai_update,
            do_pathfind: ai_do_pathfind,
        })
    }

    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.id = ctx.owner_id;
        sim.ai_construct(ctx.owner_slot);
        // The constructor ends by asking for the next frame (`SLEEPY_AI`).
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        // The state machine's draws happen, but no order is acted on afterwards.
        sim.ai_init_state_machine(ctx.owner_slot);
        if let Some(o) = sim.object_mut(ctx.owner_slot) {
            o.ai_state_machine_ready = FALSE;
        }
    }

    fn xfer(&mut self, _xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        Ok(())
    }

    fn as_ai(&self) -> Option<&dyn AiUpdateFacet> {
        Some(self)
    }

    fn as_ai_mut(&mut self) -> Option<&mut dyn AiUpdateFacet> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }
}

fn ai_state(sim: &mut dyn SimApi, ms: ModuleSlot) -> &mut Ai {
    sim.resident_state_mut(ms)
        .and_then(|a| a.downcast_mut::<Ai>())
        .expect("the module stays in the arena while it runs")
}

fn by_index(sim: &dyn SimApi, index: u32) -> Option<ObjectSlot> {
    let id = SHARED.with(|s| {
        let ids = &s.borrow().ids;
        ids.get(index as usize % ids.len().max(1)).copied()
    })?;
    sim.find_object(id)
}

fn ai_slot_of(sim: &dyn SimApi, o: ObjectSlot) -> Option<ModuleSlot> {
    sim.object(o).and_then(|o| o.ai_slot)
}

fn id_of(sim: &dyn SimApi, o: ObjectSlot) -> i32 {
    sim.object(o).map_or(0, |o| o.id.0)
}

fn spawn(sim: &mut dyn SimApi, kind: usize, team: TeamId, pos: Coord3D, angle: Real) -> ObjectId {
    let tid = SHARED.with(|s| s.borrow().templates[kind]);
    let o = sim.spawn_object(tid, team, pos, angle);
    let id = sim.object(o).expect("spawned").id;
    SHARED.with(|s| s.borrow_mut().ids.push(id));
    id
}

fn damage(sim: &mut dyn SimApi, o: ObjectSlot, amount: Real, unresistable: bool) {
    let mut d = DamageInfo::default();
    d.input.amount = amount;
    d.input.damage_type = if unresistable {
        DamageType::Unresistable
    } else {
        DamageType::Explosion
    };
    d.input.death_type = DeathType::Normal;
    d.input.source_id = ObjectId::INVALID;
    sim.attempt_damage(o, &mut d);
}

/// The harness's way into the path queue: mark the unit as waiting and queue it.
fn queue_path(sim: &mut dyn SimApi, o: ObjectSlot) {
    if let Some(ms) = ai_slot_of(sim, o) {
        ai_state(sim, ms).waiting = TRUE;
    }
    let id = sim.object(o).expect("object").id;
    assert_ne!(sim.paths().queue_for_path(id), FALSE, "queue full");
}

/// The reference computes the path here; the probe keeps only what shows:
/// `doPathfind` ends in `wakeUpNow`.
fn ai_do_pathfind(sim: &mut dyn SimApi, ctx: ModuleContext) {
    ai_state(sim, ctx.mod_slot).waiting = FALSE;
    let next = sim
        .module_schedule(ctx.mod_slot)
        .expect("schedule")
        .next_call_frame();
    if next > sim.frame() + 1 {
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
    }
}

fn ai_update(sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
    let me = ctx.owner_slot;
    let frame = sim.frame();
    emit(&format!("[u {} {frame}]", ctx.owner_id.0));
    let plan = ai_state(sim, ctx.mod_slot).plan.take().unwrap_or(Plan {
        sleep: 1,
        actions: Vec::new(),
    });
    for (code, a, b) in plan.actions {
        let other = by_index(sim, a);
        match code {
            1 | 3 => {
                let target = if code == 1 { Some(me) } else { other };
                if let Some(t) = target.filter(|&t| ai_slot_of(sim, t).is_some()) {
                    let pos = Coord3D::new(Real::from_bits(b), 2.0, 3.0);
                    sim.ai_attack_position(t, &pos, a as Int, CommandSourceType::FromAi);
                }
            }
            2 | 11 => {
                let target = if code == 2 { Some(me) } else { other };
                if let Some(t) = target.filter(|&t| ai_slot_of(sim, t).is_some()) {
                    let idle = sim.ai_is_idle(t);
                    emit(&format!(
                        "[q {} {}]",
                        id_of(sim, t),
                        i32::from(idle != FALSE)
                    ));
                }
            }
            4 => sim.destroy_object(me),
            5 => {
                if let Some(o) = other {
                    sim.destroy_object(o);
                }
            }
            6 => {
                if let Some(o) = other {
                    let n = DISABLED.len() as u32;
                    let till = sim.frame() + b / n;
                    sim.set_disabled_until(o, DISABLED[(b % n) as usize], till);
                }
            }
            7 => {
                if let Some(o) = other {
                    damage(sim, o, b as Real, true);
                }
            }
            8 => queue_path(sim, me),
            9 => ai_state(sim, ctx.mod_slot).idle = TRUE,
            10 => {
                if let Some(ms) = other.filter(|&o| o != me).and_then(|o| ai_slot_of(sim, o)) {
                    sim.set_wake_frame(ms, UpdateSleepTime(b));
                }
            }
            12 => {
                let kind = a as usize % TEMPLATES.len();
                let x = if kind == 0 { 600.0 } else { 150.0 } + (b % 97) as Real;
                let y = 200.0 + (b % 89) as Real;
                let id = spawn(sim, kind, TEAMS[1], Coord3D::new(x, y, 0.0), 0.0);
                emit(&format!("[s {}]", id.0));
            }
            _ => panic!("unknown action {code}"),
        }
    }
    emit(&format!("[x {}]", ctx.owner_id.0));
    if plan.sleep >= UPDATE_SLEEP_FOREVER {
        UpdateSleepTime::FOREVER
    } else {
        UpdateSleepTime(plan.sleep)
    }
}

#[derive(Debug, Clone)]
struct AiData {
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
}

impl ModuleData for AiData {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn tag_key(&self) -> NameKeyType {
        self.tag_key
    }
    fn module_name_key(&self) -> NameKeyType {
        self.module_name_key
    }
    fn facet_mask(&self) -> FacetMask {
        AI_MASK
    }
}

fn parse_ai(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    reader.read_block_fields(|_, _| Err(IniError::UnknownToken))?;
    Ok(Box::new(AiData {
        tag_key,
        module_name_key,
    }))
}

fn create_ai(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    let data = data.as_any().downcast_ref::<AiData>().expect("ai");
    Box::new(Ai {
        key: data.module_name_key,
        id: ObjectId::INVALID,
        idle: TRUE,
        waiting: FALSE,
        plan: None,
    })
}

// -----------------------------------------------------------------------------
// The path queue tape
// -----------------------------------------------------------------------------

#[derive(Default)]
struct Tape {
    pending: VecDeque<ObjectId>,
    head: u32,
    tail: u32,
    /// How many requests the reference served in each coming frame.
    served: VecDeque<u32>,
    budget: u32,
}

const QUEUE_LEN: u32 = 512;
type TapeLock = Mutex<Tape>; // determinism-ok: the test's tape
type SharedTape = Arc<TapeLock>;

struct TapeQueue(SharedTape);

impl PathServices for TapeQueue {
    fn queue_for_path(&mut self, id: ObjectId) -> Bool {
        let mut t = self.0.lock().expect("tape");
        if !t.pending.contains(&id) {
            t.pending.push_back(id);
            t.tail = (t.tail + 1) % QUEUE_LEN;
        }
        TRUE
    }

    fn begin_queue(&mut self) {
        let mut t = self.0.lock().expect("tape");
        t.budget = t.served.pop_front().expect("a count for this frame");
    }

    fn next_request(&mut self) -> Option<ObjectId> {
        let mut t = self.0.lock().expect("tape");
        if t.budget == 0 {
            return None;
        }
        let id = t.pending.pop_front()?;
        t.budget -= 1;
        t.head = (t.head + 1) % QUEUE_LEN;
        Some(id)
    }
}

// -----------------------------------------------------------------------------
// Observation: the same text the harness writes
// -----------------------------------------------------------------------------

#[derive(Default)]
struct Watch {
    seen: BTreeMap<i32, String>,
    seen_full: BTreeMap<i32, String>,
    global: String,
    list: String,
    frozen: Bool,
}

fn fnv(bytes: &[u8]) -> u32 {
    let mut hex = String::new();
    for b in bytes {
        let _ = write!(hex, "{b:02x}");
    }
    hex.bytes().fold(2_166_136_261u32, |h, c| {
        (h ^ u32::from(c)).wrapping_mul(16_777_619)
    })
}

fn object_state(sim: &mut SimContext, slot: ObjectSlot) -> String {
    let health = sim.object_health(slot);
    let o = sim.world.objects.get(slot).expect("object");
    let pos = o.thing.get_position();
    let mut s = format!(
        "{} {} {} {} {} {} {}",
        pos.x.to_bits(),
        pos.y.to_bits(),
        pos.z.to_bits(),
        o.thing.get_orientation().to_bits(),
        health.to_bits(),
        o.disabled_mask.to_unsigned_int(),
        i32::from(o.is_destroyed())
    );
    if let Some(ms) = o.ai_slot {
        let ai = ai_state(sim, ms);
        let _ = write!(
            s,
            " {} {}",
            i32::from(ai.waiting != FALSE),
            i32::from(ai.idle != FALSE)
        );
    }
    s
}

fn module_states(sim: &mut SimContext, slot: ObjectSlot) -> String {
    let mut s = String::new();
    for name in MODULES {
        let ai = name == "AIUpdateInterface";
        let key = sim
            .rules
            .name_keys
            .name_to_key(if ai { "OracleAI" } else { name });
        let behaviors = sim
            .world
            .objects
            .get(slot)
            .expect("object")
            .behaviors
            .clone();
        for (ms, mask) in behaviors {
            if sim.world.modules.payload(ms).map(|m| m.module_name_key()) != Some(key) {
                continue;
            }
            let _ = write!(s, " {name}:");
            if mask.contains(FacetMask::UPDATE) {
                let schedule = sim.module_schedule(ms).expect("schedule");
                let _ = write!(s, "{}", schedule.next_call_frame());
            }
            s.push(':');
            if !ai {
                let mut xc = XferCrc::new();
                sim.world.modules.payload(ms).expect("payload").crc(&mut xc);
                let (mut payload, _ctx, generation) =
                    sim.world.modules.take_payload(ms).expect("payload");
                let mut xs = XferSave::new();
                payload.xfer(&mut xs, sim).expect("xfer save");
                sim.world.modules.restore_payload(ms, generation, payload);
                let _ = write!(s, "{}:{}", xc.get_crc(), fnv(xs.as_bytes()));
            }
        }
    }
    s
}

struct Seams<'a> {
    watch: &'a mut Watch,
    brown_outs: &'a mut Vec<Int>,
    tape: SharedTape,
}

fn observe(watch: &mut Watch, tape: &TapeLock, sim: &mut SimContext, hook: &str, full: bool) {
    emit(hook);
    let (head, tail) = {
        let t = tape.lock().expect("tape");
        (t.head, t.tail)
    };
    let sabotage = |team: TeamId| {
        sim.world.players.player(sim.player_index(team)).map(|p| &p.energy)
            .map_or(0, |e| e.get_power_sabotaged_till_frame())
    };
    let global = format!(
        "{} {} {head} {tail} {} {}",
        sim.world.frame,
        sim.world.rng.get_game_logic_random_seed_crc(),
        sabotage(TEAMS[0]),
        sabotage(TEAMS[1])
    );
    if global != watch.global {
        emit(&format!("{{G {global}}}"));
        watch.global = global;
    }
    let slots: Vec<ObjectSlot> = sim.world.objects.world_iter().collect();
    let mut list = String::new();
    let mut present = Vec::new();
    for slot in slots {
        let id = sim.world.objects.get(slot).expect("object").id.0;
        present.push(id);
        let _ = write!(list, " {id}");
        let state = object_state(sim, slot);
        if watch.seen.get(&id) != Some(&state) {
            emit(&format!("{{O {id} {state}}}"));
            watch.seen.insert(id, state);
        }
        if full {
            let modules = module_states(sim, slot);
            let was = watch.seen_full.get(&id);
            // The harness keys the full entry on state and modules together.
            let all = format!("{}{modules}", watch.seen[&id]);
            if was != Some(&all) {
                emit(&format!("{{F {id}{modules}}}"));
                watch.seen_full.insert(id, all);
            }
        }
    }
    // The harness reports removed objects in the order of their ids as text.
    let mut gone: Vec<String> = watch
        .seen
        .keys()
        .filter(|id| !present.contains(id))
        .map(ToString::to_string)
        .collect();
    gone.sort();
    for id in gone {
        emit(&format!("{{-{id}}}"));
        let id: i32 = id.parse().expect("id");
        watch.seen.remove(&id);
        watch.seen_full.remove(&id);
    }
    if list != watch.list {
        emit(&format!("{{L{list}}}"));
        watch.list = list;
    }
}

impl Seams<'_> {
    fn look(&mut self, sim: &mut SimContext, hook: &str) {
        observe(self.watch, &self.tape, sim, hook, false);
    }
}

impl FrameSeams for Seams<'_> {
    fn script_engine(&mut self, sim: &mut SimContext) {
        self.look(sim, "S");
    }
    fn is_time_frozen(&mut self, _sim: &SimContext) -> Bool {
        self.watch.frozen
    }
    fn terrain_logic(&mut self, sim: &mut SimContext) {
        self.look(sim, "T");
    }
    fn recorder(&mut self, sim: &mut SimContext) {
        self.look(sim, "R");
    }
    fn player_ai(&mut self, sim: &mut SimContext, player: Int) {
        // The harness gave the first two players a computer player.
        if player < 2 {
            self.look(sim, &format!("A{player}"));
        }
    }
    fn power_brown_out_change(&mut self, _sim: &mut SimContext, player: Int) {
        self.brown_outs.push(player);
    }
    fn build_assistant(&mut self, sim: &mut SimContext) {
        self.look(sim, "B");
        // The reference's partition hook runs here too: nothing lies between.
        self.look(sim, "P");
    }
    fn weapon_store(&mut self, sim: &mut SimContext) {
        self.look(sim, "W");
    }
    fn victory_conditions(&mut self, sim: &mut SimContext) {
        self.look(sim, "V");
    }
}

// -----------------------------------------------------------------------------
// World and operations
// -----------------------------------------------------------------------------

fn rules() -> RulesRegistry {
    let mut rules = RulesRegistry::new();
    rules.module_registry.register_module(
        &mut rules.name_keys,
        "OracleAI",
        ModuleType::Behavior,
        AI_MASK,
        SleepyUpdatePhase::Initial,
        true,
        parse_ai,
        create_ai,
    );
    let locomotors = get_str(VECTORS, "locomotors")
        .expect("locomotors")
        .replace('~', "\n");
    for block in locomotors.split_inclusive("End\n") {
        let mut t = LocomotorTemplate::new();
        t.parse_ini_block(block).expect("locomotor");
        t.name = block.split_whitespace().nth(1).expect("name").to_owned();
        rules.locomotor_store.add_template(t);
    }
    let ini = get_str(VECTORS, "objects")
        .expect("objects")
        .replace('~', "\n");
    let mut reader = IniReader::new("object.ini", ini.as_bytes(), IniLoadType::Overwrite);
    rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");
    rules
}

fn world(tape: &SharedTape) -> SimContext {
    let mut sim = SimContext::new();
    sim.rules = rules();
    sim.paths = Box::new(TapeQueue(tape.clone()));
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

fn apply_op(sim: &mut SimContext, watch: &mut Watch, op: &str) {
    let v: Vec<u32> = op.split(' ').map(|t| t.parse().expect("number")).collect();
    let (code, v) = (v[0], &v[1..]);
    let real = |i: usize| Real::from_bits(v[i]);
    if code == 0 {
        let pos = Coord3D::new(real(1), real(2), real(3));
        spawn(sim, v[0] as usize, TEAMS[1 + v[5] as usize], pos, real(4));
        return;
    }
    if code == 8 {
        sim.sabotage_power(TEAMS[v[0] as usize], v[1]);
        return;
    }
    if code == 10 {
        watch.frozen = if v[0] != 0 { TRUE } else { FALSE };
        return;
    }
    let Some(o) = by_index(sim, v[0]) else {
        return;
    };
    match code {
        1 => {
            if let Some(ms) = ai_slot_of(sim, o) {
                ai_state(sim, ms).plan = Some(Plan {
                    sleep: v[1],
                    actions: v[2..].chunks_exact(3).map(|c| (c[0], c[1], c[2])).collect(),
                });
            }
        }
        2 => sim.physics_apply_force(o, &Coord3D::new(real(1), real(2), real(3))),
        3 => sim.physics_apply_shock(o, &Coord3D::new(real(1), real(2), real(3))),
        4 => damage(sim, o, real(1), v[2] != 0),
        5 => sim.destroy_object(o),
        6 => {
            let till = sim.world.frame + v[2];
            sim.set_disabled_until(o, DISABLED[v[1] as usize], till);
        }
        7 => {
            sim.clear_disabled(o, DISABLED[v[1] as usize]);
        }
        9 => queue_path(sim, o),
        _ => panic!("unknown op {code}"),
    }
}

/// How far the reference's queue head moved, from the vector's global entries.
fn served(line_out: &str, head: &mut u32) -> u32 {
    let before = *head;
    for entry in line_out.split("{G ").skip(1) {
        let fields = entry.split('}').next().expect("entry");
        *head = fields
            .split(' ')
            .nth(2)
            .and_then(|f| f.parse().ok())
            .expect("queue head");
    }
    (*head + QUEUE_LEN - before) % QUEUE_LEN
}

/// The sabotage end of the two watched players.
fn sabotage(sim: &SimContext) -> [u32; 2] {
    [TEAMS[0], TEAMS[1]].map(|team| {
        sim.world.players.player(sim.player_index(team)).map(|p| &p.energy)
            .map_or(0, |e| e.get_power_sabotaged_till_frame())
    })
}

/// The same value at the end of the reference's frame.
fn sabotage_after(line_out: &str, before: u32, player: usize) -> u32 {
    line_out
        .split("{G ")
        .skip(1)
        .last()
        .map_or(before, |entry| {
            let fields = entry.split('}').next().expect("entry");
            fields
                .split(' ')
                .nth(4 + player)
                .and_then(|f| f.parse().ok())
                .expect("sabotage")
        })
}

fn first_difference(got: &str, want: &str) -> String {
    let at = got
        .bytes()
        .zip(want.bytes())
        .position(|(a, b)| a != b)
        .unwrap_or_else(|| got.len().min(want.len()));
    let from = at.saturating_sub(120);
    let cut = |s: &str| {
        s.get(from..(at + 200).min(s.len()))
            .unwrap_or("")
            .to_owned()
    };
    format!("at byte {at}\n rust: {}\n  c++: {}", cut(got), cut(want))
}

#[test]
fn the_frame_step_matches_the_reference_trace() {
    let lines: Vec<&str> = VECTORS
        .lines()
        .filter(|l| l.starts_with("{\"scenario\""))
        .collect();
    assert_eq!(
        lines.len() as u32,
        get_u32(VECTORS, "vector_count").expect("count")
    );
    assert!(lines.len() >= 1000);

    let tape: SharedTape = Arc::default();
    let mut sim = world(&tape);
    let mut watch = Watch::default();
    let mut head = 0;
    for line in lines {
        let scenario = get_u32(line, "scenario").expect("scenario");
        let frame = get_u32(line, "frame").expect("frame");
        if frame == 0 {
            *tape.lock().expect("tape") = Tape::default();
            sim = world(&tape);
            watch = Watch::default();
            head = 0;
            let templates = TEMPLATES
                .map(|name| {
                    sim.rules
                        .thing_factory
                        .find_template_id(name)
                        .expect("template")
                })
                .to_vec();
            SHARED.with(|s| {
                *s.borrow_mut() = Shared {
                    templates,
                    ..Shared::default()
                }
            });
            sim.world.frame = get_u32(line, "first").expect("first");
            sim.world.next_object_id = get_u32(line, "id").expect("id") as i32;
            sim.world
                .rng
                .init_random_with_seed(get_u32(line, "seed").expect("seed"));
        }
        let ops = get_str(line, "ops").expect("ops");
        for op in ops.split(';').filter(|o| !o.is_empty()) {
            apply_op(&mut sim, &mut watch, op);
        }
        let want = get_str(line, "out").expect("out");
        let count = served(want, &mut head);
        tape.lock().expect("tape").served.clear();
        tape.lock().expect("tape").served.push_back(count);

        // The reference calls `onPowerBrownOutChange` where a sabotage ends.
        let before = sabotage(&sim);
        let expected: Vec<Int> = (0..2)
            .filter(|&p| before[p] != 0 && sabotage_after(want, before[p], p) == 0)
            .map(|p| p as Int)
            .collect();

        SHARED.with(|s| s.borrow_mut().out.clear());
        let mut brown_outs = Vec::new();
        run_frame(
            &mut sim,
            &mut Seams {
                watch: &mut watch,
                brown_outs: &mut brown_outs,
                tape: tape.clone(),
            },
        );
        observe(&mut watch, &tape, &mut sim, "E", true);
        let got = SHARED.with(|s| s.borrow().out.clone());
        if got != want && std::env::var_os("S0_DUMP").is_some() {
            println!("rust: {got}\n c++: {want}");
        }
        assert!(
            got == want,
            "scenario {scenario} frame {frame} (ops {ops}) differs {}",
            first_difference(&got, want)
        );
        assert_eq!(brown_outs, expected, "scenario {scenario} frame {frame}");
    }
}
