//! Differential suite for `AIGroup` (`PKT-423`) and `Squad` (`PKT-409`) against
//! `docs/golden_vectors/PKT-423.json`, recorded by
//! `tests/cpp_harness/gen_pkt423_ai_group.cpp`. Compared at every step: the orders
//! each member is given and the pathfinder calls, in order, with their float bits;
//! the logic random seed; each live group's members, `crc` and `xfer` bytes; each
//! object's group, formation and mood check time.

use std::collections::VecDeque;
use std::fmt::Write as _;
use std::sync::{Arc, Mutex}; // determinism-ok: the harness tape
use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::io::recorder::{ReplayArg, ReplayCommand};
use zh_common::math::{Coord3D, Matrix3D, Region3D};
use zh_common::types::{Bool, ObjectId, Real, Relationship, TeamId, UnsignedInt, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferCrc, XferLoad, XferSave};
use zh_logic::ai::ai_group::{self, GroupAiData};
use zh_logic::ai::ai_state_machine::{self as machine, state_id};
use zh_logic::ai::seams::{shared, AiSeams};
use zh_logic::arena::{ModuleContext, ObjectSlot};
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::gamelogic::command_target::{self, dispatch_command, CommandState};
use zh_logic::gamelogic::global_data::BodyDamageType;
use zh_logic::modules::module::{AttitudeType, CommandSourceType};
use zh_logic::modules::update::ai_update::{AiCommandParms, AiCommandType, AiUpdate};
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::object::weapon::WeaponTemplate;
use zh_logic::object::weapon_set::CanAttackResult;
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;
use zh_logic::player::squad::{self, Squad};
use zh_logic::rules::combat_types::{LocomotorSurfaceTypeMask, WeaponSetType};
use zh_logic::rules::command_set::{CommandButton, CommandButtonType};
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::sim::{AttackAbility, SimApi, SimContext, SimView};

const VECTORS: &str = include_str!("../../../docs/golden_vectors/PKT-423.json");

const TEAMS: [TeamId; 3] = [TeamId(0), TeamId(1), TeamId(2)];

const TEMPLATES: [&str; 16] = [
    "G_Inf",
    "G_InfFront",
    "G_InfBack",
    "G_Veh",
    "G_Tank",
    "G_Cliff",
    "G_Heli",
    "G_Jet",
    "G_Fixed",
    "G_House",
    "G_Rock",
    "G_Sneak",
    "G_Bold",
    "G_Granted",
    "G_Nexus",
    "G_Tower",
];

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
    let f: Vec<&str> = text.split(',').collect();
    Coord3D::new(real(f[0]), real(f[1]), real(f[2]))
}

fn number(text: &str) -> i64 {
    text.parse().expect("number")
}

fn hex(bytes: &[u8]) -> String {
    let mut out = String::new();
    for byte in bytes {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

fn command_name(cmd: AiCommandType) -> &'static str {
    use AiCommandType as C;
    match cmd {
        C::MoveToPosition => "move",
        C::TightenToPosition => "tighten",
        C::MoveToPositionAndEvacuate => "moveevac",
        C::MoveToPositionAndEvacuateAndExit => "moveevacexit",
        C::Idle => "idle",
        C::FollowPath => "path",
        C::FollowPathAppend => "append",
        C::AttackObject => "attack",
        C::ForceAttackObject => "forceattack",
        C::AttackPosition => "attackpos",
        C::AttackMoveToPosition => "attackmove",
        C::Hunt => "hunt",
        C::Repair => "repair",
        C::ResumeConstruction => "resume",
        C::GetHealed => "healed",
        C::GetRepaired => "repaired",
        C::Enter => "enter",
        C::Dock => "dock",
        C::Exit => "exit",
        C::Evacuate => "evacuate",
        C::ExecuteRailedTransport => "rail",
        C::GuardPosition => "guardpos",
        C::GuardObject => "guardobj",
        C::HackInternet => "hack",
        C::CombatDrop => "drop",
        _ => "?",
    }
}

#[derive(Default)]
struct Tape {
    /// The reference's calls of the step still to be made.
    entries: VecDeque<String>,
    recording: bool,
    context: String,
}

impl Tape {
    /// The next call must be `call`; returns what the reference answered.
    fn take(&mut self, call: &str) -> String {
        if !self.recording {
            return String::new();
        }
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

impl Seams {
    fn take(&self, call: String) -> String {
        self.0.lock().expect("tape").take(&call)
    }

    fn recording(&self) -> bool {
        self.0.lock().expect("tape").recording
    }
}

impl AiSeams for Seams {
    fn group_order(&mut self, owner: ObjectId, p: &AiCommandParms) -> Bool {
        let mut call = format!(
            "o,{},{},{},{},{},{},{}",
            owner.0,
            command_name(p.cmd),
            p.source as i32,
            c3(&p.pos),
            p.obj.map_or(0, |o| o.0),
            p.int_value,
            p.coords.len()
        );
        for c in &p.coords {
            let _ = write!(call, ",{}", c3(c));
        }
        self.take(call);
        TRUE
    }

    fn remove_goal(&mut self, owner: ObjectId) {
        self.take(format!("R,{}", owner.0));
    }

    fn update_goal(&mut self, owner: ObjectId, pos: &Coord3D, layer: PathfindLayerEnum) {
        self.take(format!("G,{},{},{}", owner.0, c3(pos), layer as i32));
    }

    fn update_pos(&mut self, owner: ObjectId, pos: &Coord3D) {
        self.take(format!("P,{},{}", owner.0, c3(pos)));
    }

    fn adjust_destination(
        &mut self,
        owner: ObjectId,
        dest: &mut Coord3D,
        group_dest: Option<&Coord3D>,
    ) {
        let toward = group_dest.map_or_else(|| "-".to_owned(), c3);
        let answer = self.take(format!("A,{},{},{toward}", owner.0, c3(dest)));
        if self.recording() {
            *dest = coord(&answer);
        }
    }

    fn is_line_passable(
        &mut self,
        owner: ObjectId,
        surfaces: LocomotorSurfaceTypeMask,
        layer: PathfindLayerEnum,
        from: &Coord3D,
        to: &Coord3D,
    ) -> Bool {
        let answer = self.take(format!(
            "L,{},{surfaces},{},{},{},01",
            owner.0,
            layer as i32,
            c3(from),
            c3(to)
        ));
        Bool::from(answer == "1")
    }

    fn find_ground_path(
        &mut self,
        from: &Coord3D,
        to: &Coord3D,
        diameter: i32,
    ) -> Option<Vec<Coord3D>> {
        let answer = self.take(format!("F,{},{},{diameter}0", c3(from), c3(to)));
        let f: Vec<&str> = answer.split(',').collect();
        let count = number(f[0]) as usize;
        (count != 0).then(|| {
            f[1..]
                .chunks(3)
                .map(|n| Coord3D::new(real(n[0]), real(n[1]), real(n[2])))
                .collect()
        })
    }

    fn map_extent(&mut self, extent: Region3D, maximum_pathfind: bool) -> Region3D {
        if maximum_pathfind {
            Region3D {
                lo: Coord3D::new(60.0, 40.0, 0.0),
                hi: Coord3D::new(1850.0, 1900.0, 100.0),
            }
        } else {
            extent
        }
    }
}

/// The harness's sloped ground.
struct Slope;

impl ThingTerrainAccess for Slope {
    fn get_ground_height(&self, x: Real, y: Real, normal: Option<&mut Coord3D>) -> Real {
        if let Some(n) = normal {
            *n = Coord3D::new(0.0, 0.0, 1.0);
        }
        x * 0.01 + y * 0.02
    }

    fn align_on_terrain(
        &self,
        angle: Real,
        pos: &Coord3D,
        _stick_to_ground: Bool,
        mtx: &mut Matrix3D,
    ) -> PathfindLayerEnum {
        zh_logic::object::thing::make_align_to_normal_matrix(
            angle,
            pos,
            &Coord3D::new(0.0, 0.0, 1.0),
            mtx,
        );
        PathfindLayerEnum::Ground
    }

    fn is_underwater(
        &self,
        x: Real,
        y: Real,
        water_z: Option<&mut Real>,
        terrain_z: Option<&mut Real>,
    ) -> Bool {
        if let Some(w) = water_z {
            *w = 0.0;
        }
        if let Some(t) = terrain_z {
            *t = self.get_ground_height(x, y, None);
        }
        FALSE
    }
}

fn text(key: &str) -> String {
    get_str(VECTORS, key).expect("text").replace("\\n", "\n")
}

fn world(tape: &SharedTape, humans: &str) -> SimContext {
    let mut sim = SimContext::new();
    for block in text("weapons").split_inclusive("End\n") {
        let mut w = WeaponTemplate::new();
        w.parse_ini_block(block).expect("weapon");
        sim.rules.weapon_store.add_template(w);
    }
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
    sim.terrain = Box::new(Slope);
    sim.ai_seams = shared(Seams(tape.clone()));
    sim.init_partition(
        20.0,
        Region3D {
            lo: Coord3D::new(0.0, 0.0, 0.0),
            hi: Coord3D::new(2000.0, 2000.0, 100.0),
        },
    );
    let w = &mut sim.world;
    for (player, team) in TEAMS.into_iter().enumerate() {
        w.team_players.insert(team, player as i32);
        w.player_default_teams.insert(player as i32, team);
    }
    for (team, human) in TEAMS[1..].iter().zip(humans.chars()) {
        w.set_team_human(*team, Bool::from(human == '1'));
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
            w.set_player_relation(i, j, rel);
        }
    }
    sim
}

fn spawn(sim: &mut SimContext, kind: usize, team: usize, pos: &Coord3D) -> ObjectSlot {
    let tid = sim
        .rules
        .thing_factory
        .find_template_id(TEMPLATES[kind])
        .expect("template");
    let slot = sim.spawn_object(tid, TEAMS[1 + team], Coord3D::default(), 0.0);
    sim.set_object_position(slot, pos);
    slot
}

struct Run {
    commands: CommandState,
    sim: SimContext,
    units: Vec<ObjectId>,
    groups: Vec<u32>,
}

impl Run {
    fn observe_groups(&mut self) -> String {
        let mut out = Vec::new();
        for &id in &self.groups {
            let Some(group) = self.sim.world.ai_groups.find(id) else {
                continue;
            };
            let members: Vec<String> = group.members.iter().map(|m| m.0.to_string()).collect();
            let mut crc = XferCrc::new();
            group.crc(&mut crc).expect("crc");
            let mut save = XferSave::new();
            group.clone().xfer(&mut save).expect("save");
            out.push(format!(
                "{id}:{}:{}:{:08x}:{}",
                group.member_list_size,
                members.join("."),
                crc.get_crc(),
                hex(save.as_bytes())
            ));
        }
        out.join("|")
    }

    fn observe_unit(&mut self, index: usize) -> String {
        let Some(slot) = self.sim.find_object(self.units[index]) else {
            return "gone".to_owned();
        };
        let o = self.sim.object(slot).expect("object");
        let mut s = format!(
            "{},{},{},{}",
            o.formation_id,
            bits(o.formation_offset.x),
            bits(o.formation_offset.y),
            o.ai_group.unwrap_or(0)
        );
        let _ = write!(
            s,
            ",{}{}{}",
            o.weapon_set.cur_weapon as i32,
            o.weapon_set.cur_weapon_locked_status as i32,
            i32::from(
                o.weapon_set_flags
                    .test(WeaponSetType::MineClearingDetail.as_i32())
                    != 0
            )
        );
        if let Some(ms) = o.ai_slot {
            let ai = self
                .sim
                .resident_state_mut(ms)
                .and_then(|a| a.downcast_mut::<AiUpdate>())
                .expect("AI");
            let _ = write!(
                s,
                ",{},{}",
                ai.next_mood_check_time, ai.randomly_offset_mood_check
            );
        }
        s
    }

    fn apply(&mut self, op: &str) {
        let f: Vec<&str> = op.split(' ').collect();
        let index = |i: usize| number(f[i]) as usize;
        let source = |i: usize| match number(f[i]) {
            0 => CommandSourceType::FromPlayer,
            1 => CommandSourceType::FromScript,
            _ => CommandSourceType::FromAi,
        };
        let groups = self.groups.clone();
        let group = |i: usize| groups[index(i)];
        let units = self.units.clone();
        let unit = |i: usize| units[index(i)];
        let sim = &mut self.sim;
        let slot = |sim: &SimContext, id: ObjectId| sim.find_object(id).expect("unit");
        match f[0] {
            "init" => {
                for (i, g) in f[1..].iter().enumerate() {
                    ai_group::add(sim, groups[number(g) as usize], units[i]);
                }
            }
            "new" => {
                let id = ai_group::create_group(sim);
                self.groups.push(id);
            }
            "add" => ai_group::add(sim, group(1), unit(2)),
            "remove" => {
                let emptied = ai_group::remove(sim, group(1), unit(2));
                // The harness reports the result of `remove` alone.
                assert_eq!(i32::from(emptied), number(f[3]) as i32, "{op}");
            }
            "drop" => ai_group::destroy_group(sim, group(1)),
            "move" => ai_group::group_move_to_position(
                sim,
                group(1),
                &coord(f[2]),
                f[3] == "1",
                source(4),
            ),
            "tighten" => ai_group::group_tighten_to_position(
                sim,
                group(1),
                &coord(f[2]),
                f[3] == "1",
                source(4),
            ),
            "scatter" => ai_group::group_scatter(sim, group(1), source(2)),
            "idle" => ai_group::group_idle(sim, group(1), source(2)),
            "attack" => ai_group::group_attack_object(
                sim,
                group(1),
                f[2] == "1",
                unit(3),
                number(f[4]) as i32,
                source(5),
            ),
            "apos" => {
                let pos = (f[2] == "1").then(|| coord(f[3]));
                ai_group::group_attack_position(
                    sim,
                    group(1),
                    pos.as_ref(),
                    number(f[4]) as i32,
                    source(5),
                );
            }
            "amove" => {
                ai_group::group_attack_move_to_position(sim, group(1), &coord(f[2]), 7, source(3))
            }
            "order" => {
                let (g, target, pos) = (group(1), unit(3), coord(f[4]));
                let mode = number(f[5]) as i32;
                let src = source(6);
                use AiCommandType as C;
                let on_object = |cmd| Some(cmd);
                let cmd = match number(f[2]) {
                    0 => on_object(C::Repair),
                    1 => on_object(C::ResumeConstruction),
                    2 => on_object(C::GetHealed),
                    3 => on_object(C::GetRepaired),
                    4 => on_object(C::Enter),
                    5 => on_object(C::Dock),
                    6 => on_object(C::Exit),
                    _ => None,
                };
                match (cmd, number(f[2])) {
                    (Some(cmd), _) => ai_group::group_order_on_object(sim, g, cmd, target, src),
                    (None, 7) => ai_group::group_hunt(sim, g, src),
                    (None, 8) => ai_group::group_execute_railed_transport(sim, g, src),
                    (None, 9) => ai_group::group_guard_position(sim, g, &pos, mode, src),
                    (None, 10) => ai_group::group_guard_object(sim, g, target, mode, src),
                    (None, 11) => ai_group::group_hack_internet(sim, g, src),
                    _ => ai_group::group_combat_drop(
                        sim,
                        g,
                        (mode != 0).then_some(target),
                        &pos,
                        src,
                    ),
                }
            }
            "evac" => match number(f[2]) {
                0 => ai_group::group_evacuate(sim, group(1), source(4)),
                kind => ai_group::group_move_to_and_evacuate(
                    sim,
                    group(1),
                    &coord(f[3]),
                    kind == 2,
                    source(4),
                ),
            },
            "form" => ai_group::group_create_formation(sim, group(1)),
            "owner" => {
                let player = number(f[2]) as i32;
                let g = group(1);
                let any = ai_group::contains_any_not_owned_by(sim, g, player);
                let destroyed = ai_group::remove_any_not_owned_by(sim, g, player);
                assert_eq!(
                    format!("{}{}", u8::from(any), u8::from(destroyed)),
                    f[3],
                    "{op}"
                );
            }
            "query" => {
                let g = group(1);
                let (has, center) = ai_group::center(sim, g);
                let (formation, min, max, other) = ai_group::min_max_and_center(sim, g);
                let speed = ai_group::speed(sim, g);
                let idle = ai_group::is_idle(sim, g);
                let busy = ai_group::is_busy(sim, g);
                let dead = ai_group::is_ai_dead(sim, g);
                let got = format!(
                    "{},{},{},{},{},{},{},{},{},{}{}{}",
                    u8::from(has),
                    c3(&center),
                    u8::from(formation),
                    bits(min.x),
                    bits(min.y),
                    bits(max.x),
                    bits(max.y),
                    c3(&other),
                    bits(speed),
                    u8::from(idle),
                    u8::from(busy),
                    u8::from(dead)
                );
                assert_eq!(got, f[2], "{op}");
            }
            "squad" => {
                let g = group(1);
                let mut s = Squad::default();
                if f[2] != "-" {
                    s.ids = f[2].split('.').map(|id| number(id) as u32).collect();
                }
                squad::ai_group_from_squad(&mut s, sim, g);
                let mut kept = XferSave::new();
                squad::xfer_squad(&mut s, &mut kept).expect("save");
                assert_eq!(hex(kept.as_bytes()), f[3], "{op}");
                squad::squad_from_ai_group(&mut s, sim, g);
                let mut taken = XferSave::new();
                squad::xfer_squad(&mut s, &mut taken).expect("save");
                assert_eq!(hex(taken.as_bytes()), f[4], "{op}");
            }
            "pos" => {
                let s = slot(sim, unit(1));
                sim.set_object_position(s, &coord(f[2]));
            }
            "status" => {
                let s = slot(sim, unit(1));
                let o = sim.object_mut(s).expect("object");
                o.status.set(number(f[2]) as i32, number(f[3]) as _);
            }
            "held" => {
                let s = slot(sim, unit(1));
                let o = sim.object_mut(s).expect("object");
                o.disabled_mask
                    .set(DisabledType::Held.as_i32(), number(f[2]) as _);
            }
            "damage" => {
                let s = slot(sim, unit(1));
                let state = match number(f[2]) {
                    0 => BodyDamageType::Pristine,
                    1 => BodyDamageType::Damaged,
                    _ => BodyDamageType::ReallyDamaged,
                };
                sim.set_body_damage_state(s, state);
            }
            "busy" => {
                let s = slot(sim, unit(1));
                let ms = sim.object(s).and_then(|o| o.ai_slot).expect("AI");
                let ctx: ModuleContext = sim.world.modules.meta(ms).context(ms);
                let _ = machine::set_state(sim, ctx, state_id::BUSY);
            }
            "dead" => {
                let s = slot(sim, unit(1));
                sim.object_mut(s)
                    .expect("object")
                    .set_effectively_dead(TRUE);
            }
            "destroy" => {
                let s = slot(sim, unit(1));
                sim.destroy_object(s);
                sim.process_destroy_list();
            }
            "cmd" => {
                let mut msg = ReplayCommand::new(number(f[2]) as i32, number(f[1]) as i32);
                for arg in &f[4..] {
                    let (kind, value) = arg.split_once(':').expect("argument");
                    msg = msg.with(match kind {
                        "i" => ReplayArg::integer(number(value) as i32),
                        "b" => ReplayArg::boolean(value == "1"),
                        "o" => ReplayArg::object_id(number(value) as u32),
                        _ => {
                            let p = coord(value);
                            ReplayArg::location(p.x, p.y, p.z)
                        }
                    });
                }
                dispatch_command(sim, &mut self.commands, &msg);
                assert_eq!(
                    i64::from(sim.world.ai_groups.next_group_id),
                    number(f[3]),
                    "{op}: group ids used"
                );
            }
            "frames" => sim.world.frame += number(f[1]) as UnsignedInt,
            other => panic!("unknown operation {other}"),
        }
    }
}

fn run(line: &str) {
    let tape: SharedTape = Arc::default();
    let index = get_u32(line, "i").expect("index");
    let mut sim = world(&tape, get_str(line, "humans").expect("humans"));
    sim.world.frame = get_u32(line, "frame").expect("frame");
    sim.world
        .rng
        .init_random_with_seed(get_u32(line, "seed").expect("seed"));
    sim.world.next_object_id = 1000;
    let data: Vec<&str> = get_str(line, "data").expect("data").split(',').collect();
    sim.world.ai_groups.data = GroupAiData {
        min_infantry_for_group: number(data[0]) as i32,
        min_vehicles_for_group: number(data[1]) as i32,
        min_distance_for_group: real(data[2]),
        distance_requires_group: real(data[3]),
    };
    sim.rules.global_data.group_move_click_to_gather_factor = real(data[4]);
    sim.rules.global_data.movement_penalty_damage_state = if number(data[5]) == 2 {
        BodyDamageType::ReallyDamaged
    } else {
        BodyDamageType::Rubble
    };
    sim.world.ai_groups.next_group_id = get_u32(line, "gid").expect("gid") - 1;
    sim.world.ai_groups.next_formation_id = get_u32(line, "fid").expect("fid") + 1;

    let mut units = Vec::new();
    for spawned in get_str(line, "spawns").expect("spawns").split(';') {
        let f: Vec<&str> = spawned.split(',').collect();
        let pos = Coord3D::new(real(f[2]), real(f[3]), real(f[4]));
        let slot = spawn(&mut sim, number(f[0]) as usize, number(f[1]) as usize, &pos);
        units.push(sim.object(slot).expect("spawned").id);
    }
    let mut run = Run {
        commands: CommandState::default(),
        sim,
        units,
        groups: Vec::new(),
    };
    for _ in 0..get_u32(line, "groups").expect("groups") {
        let id = ai_group::create_group(&mut run.sim);
        run.groups.push(id);
    }

    let mut seen = vec![String::new(); run.units.len()];
    for (step, record) in get_str(line, "steps")
        .expect("steps")
        .split('#')
        .enumerate()
    {
        let f: Vec<&str> = record.split('~').collect();
        let context = format!("vector {index} step {step} `{}`", f[0]);
        {
            let mut t = tape.lock().expect("tape");
            t.context.clone_from(&context);
            t.entries = f[1]
                .split(';')
                .filter(|c| !c.is_empty())
                .map(str::to_owned)
                .collect();
            t.recording = true;
        }
        run.apply(f[0]);
        {
            let mut t = tape.lock().expect("tape");
            t.recording = false;
            assert!(
                t.entries.is_empty(),
                "{context}: calls not made: {:?}",
                t.entries
            );
        }
        let seed = format!("{:08x}", run.sim.world.rng.get_game_logic_random_seed_crc());
        assert_eq!(seed, f[2], "{context}: random seed");
        assert_eq!(run.observe_groups(), f[3], "{context}: groups");
        for unit in 0..run.units.len() {
            if f[4 + unit] != "=" {
                seen[unit] = f[4 + unit].to_owned();
            }
            assert_eq!(run.observe_unit(unit), seen[unit], "{context}: unit {unit}");
        }
    }
}

#[test]
fn groups_match_the_reference() {
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

// -----------------------------------------------------------------------------
// What the oracle does not reach: Rust-only checks of behaviour read off the C++
// -----------------------------------------------------------------------------

type Log = Arc<Mutex<Vec<String>>>; // determinism-ok: the harness log

/// Records what groups hand to unported code; orders reach the units when `deliver`.
struct Recorder {
    log: Log,
    deliver: bool,
}

impl Recorder {
    fn push(&self, line: String) {
        self.log.lock().expect("log").push(line);
    }
}

impl AiSeams for Recorder {
    fn group_order(&mut self, owner: ObjectId, p: &AiCommandParms) -> Bool {
        self.push(format!(
            "{} {:?} {} {:?} {:?}",
            owner.0,
            p.cmd,
            p.int_value,
            p.waypoint,
            p.area.as_ref().map(|a| &a.0)
        ));
        Bool::from(!self.deliver)
    }
    fn can_do_special_power(
        &mut self,
        owner: ObjectId,
        power: UnsignedInt,
        _target: (Option<ObjectId>, Option<Coord3D>),
        options: UnsignedInt,
    ) -> Bool {
        self.push(format!("{} power {power} {options}", owner.0));
        FALSE
    }
    fn sell_object(&mut self, owner: ObjectId) {
        self.push(format!("{} sell", owner.0));
    }
    fn override_special_power_destination(&mut self, owner: ObjectId, kind: i32, _to: &Coord3D) {
        self.push(format!("{} override {kind}", owner.0));
    }
}

fn small_world(deliver: bool, kinds: &[usize]) -> (SimContext, Vec<ObjectSlot>, Log) {
    let tape: SharedTape = Arc::default();
    let mut sim = world(&tape, "11");
    let log: Log = Arc::default();
    sim.ai_seams = shared(Recorder {
        log: log.clone(),
        deliver,
    });
    sim.world.frame = 5;
    let units = kinds
        .iter()
        .enumerate()
        .map(|(i, &kind)| {
            let pos = Coord3D::new(300.0 + 40.0 * i as Real, 300.0, 0.0);
            spawn(&mut sim, kind, 0, &pos)
        })
        .collect();
    (sim, units, log)
}

fn id_of(sim: &SimContext, slot: ObjectSlot) -> ObjectId {
    sim.object(slot).expect("object").id
}

fn select(player: i32, ids: &[ObjectId]) -> ReplayCommand {
    let mut msg = ReplayCommand::new(1001, player).with(ReplayArg::boolean(true));
    for id in ids {
        msg = msg.with(ReplayArg::object_id(id.0 as u32));
    }
    msg
}

fn ai_state(sim: &mut SimContext, slot: ObjectSlot) -> u32 {
    let ms = sim.object(slot).and_then(|o| o.ai_slot).expect("AI");
    let ai = sim
        .resident_state_mut(ms)
        .and_then(|a| a.downcast_mut::<AiUpdate>())
        .expect("AI");
    ai.machine.current_state_id()
}

#[test]
fn a_recorded_command_stream_reaches_the_units() {
    let (mut sim, units, _) = small_world(true, &[0, 3]);
    let ids: Vec<ObjectId> = units.iter().map(|&s| id_of(&sim, s)).collect();
    let mut commands = CommandState::default();
    dispatch_command(&mut sim, &mut commands, &select(1, &ids));
    assert_eq!(ai_state(&mut sim, units[0]), state_id::IDLE);
    let to = ReplayCommand::new(command_target::MSG_DO_MOVETO, 1)
        .with(ReplayArg::location(900.0, 900.0, 0.0));
    dispatch_command(&mut sim, &mut commands, &to);
    for &slot in &units {
        assert_eq!(ai_state(&mut sim, slot), state_id::MOVE_TO);
        // The group of the command is gone, and with it the membership.
        assert_eq!(sim.object(slot).expect("object").ai_group, None);
    }
    assert!(sim.world.ai_groups.list.is_empty());
    let stop = ReplayCommand::new(command_target::MSG_DO_STOP, 1);
    dispatch_command(&mut sim, &mut commands, &stop);
    assert_eq!(ai_state(&mut sim, units[1]), state_id::IDLE);
    // Another player's command does not move them: they are not theirs.
    dispatch_command(&mut sim, &mut commands, &select(2, &ids));
    let other = ReplayCommand::new(command_target::MSG_DO_MOVETO, 2)
        .with(ReplayArg::location(100.0, 100.0, 0.0));
    dispatch_command(&mut sim, &mut commands, &other);
    assert_eq!(ai_state(&mut sim, units[0]), state_id::IDLE);
    assert!(commands.not_carried_out.contains(&other.msg_type));
}

#[test]
fn the_orders_the_oracle_does_not_give() {
    // A vehicle, a house without an AI, and a refused shrub.
    let (mut sim, units, log) = small_world(false, &[3, 9, 10]);
    let ids: Vec<ObjectId> = units.iter().map(|&s| id_of(&sim, s)).collect();
    let group = ai_group::create_group(&mut sim);
    for &id in &ids {
        ai_group::add(&mut sim, group, id);
    }
    assert_eq!(ai_group::member_ids(&sim, group), ids[..2]);
    assert!(ai_group::is_member(&sim, group, ids[1]));
    assert!(!ai_group::is_member(&sim, group, ids[2]));
    let taken = || std::mem::take(&mut *log.lock().expect("log"));
    let script = CommandSourceType::FromScript;
    let (v, house) = (ids[0].0, ids[1].0);

    ai_group::group_hunt(&mut sim, group, script);
    ai_group::group_attack_team(&mut sim, group, 4, script);
    ai_group::group_follow_waypoint_path(
        &mut sim,
        group,
        AiCommandType::FollowWaypointPathAsTeam,
        (7, "Route".to_owned()),
        script,
    );
    ai_group::group_order_on_area(
        &mut sim,
        group,
        AiCommandType::GuardArea,
        ("Base".to_owned(), Coord3D::default()),
        2,
        script,
    );
    assert_eq!(
        taken(),
        [
            format!("{v} Hunt 0 None None"),
            format!("{v} AttackTeam 4 None None"),
            format!("{v} FollowWaypointPathAsTeam 0 Some((7, \"Route\")) None"),
            format!("{v} GuardArea 2 None Some(\"Base\")"),
        ]
    );

    ai_group::group_sell(&mut sim, group);
    ai_group::group_override_special_power_destination(&mut sim, group, 3, &Coord3D::default());
    assert_eq!(
        taken(),
        [
            format!("{v} sell"),
            format!("{house} sell"),
            format!("{v} override 3"),
            format!("{house} override 3"),
        ]
    );

    ai_group::set_attitude(&mut sim, group, AttitudeType::Aggressive);
    let ms = sim.object(units[0]).and_then(|o| o.ai_slot).expect("AI");
    let ai = sim
        .resident_state_mut(ms)
        .and_then(|a| a.downcast_mut::<AiUpdate>())
        .expect("AI");
    assert_eq!(ai.attitude, AttitudeType::Aggressive);

    ai_group::group_cheer(&mut sim, group);
    let cheering = sim.object(units[0]).expect("object");
    assert_eq!(cheering.special_model_condition_until, 5 + 90);
    ai_group::set_mine_clearing_detail(&mut sim, group, true);
    for &slot in &units[..2] {
        let flags = &sim.object(slot).expect("object").weapon_set_flags;
        assert_ne!(flags.test(WeaponSetType::MineClearingDetail.as_i32()), 0);
    }

    // A formation id is used up by each toggle.
    let first = sim.world.ai_groups.next_formation_id;
    ai_group::group_create_formation(&mut sim, group);
    assert_eq!(sim.object(units[0]).expect("object").formation_id, first);
    assert_eq!(sim.world.ai_groups.next_formation_id, first + 1);

    // Removing the last member destroys the group.
    assert!(!ai_group::remove(&mut sim, group, ids[0]));
    assert!(ai_group::remove(&mut sim, group, ids[1]));
    assert!(sim.world.ai_groups.find(group).is_none());
}

#[test]
fn the_group_list_is_hashed_in_order() {
    let (mut sim, units, _) = small_world(false, &[0, 0]);
    let ids: Vec<ObjectId> = units.iter().map(|&s| id_of(&sim, s)).collect();
    let crc = |sim: &SimContext| {
        let mut x = XferCrc::new();
        sim.world.ai_groups.crc_groups(&mut x).expect("crc");
        x.get_crc()
    };
    let empty = crc(&sim);
    let a = ai_group::create_group(&mut sim);
    ai_group::add(&mut sim, a, ids[0]);
    let b = ai_group::create_group(&mut sim);
    ai_group::add(&mut sim, b, ids[1]);
    assert_eq!((a, b), (1, 2));
    let mut want = XferCrc::new();
    for group in &sim.world.ai_groups.list {
        let mut marker = String::from("MARKER:AIGroup");
        want.xfer_ascii_string(&mut marker).expect("marker");
        group.crc(&mut want).expect("group");
    }
    assert_eq!(crc(&sim), want.get_crc());
    assert_ne!(crc(&sim), empty);
}

#[test]
fn a_squad_loads_by_appending() {
    let mut saved = Squad { ids: vec![7, 3, 7] };
    let mut out = XferSave::new();
    squad::xfer_squad(&mut saved, &mut out).expect("save");
    let mut loaded = Squad { ids: vec![1] };
    let mut load = XferLoad::new(out.as_bytes());
    squad::xfer_squad(&mut loaded, &mut load).expect("load");
    assert_eq!(loaded.ids, [1, 7, 3, 7]);
    assert!(squad::is_on_squad(&loaded, ObjectId(3)));
    assert!(!squad::is_on_squad(&loaded, ObjectId(4)));

    let (mut sim, units, _) = small_world(false, &[0, 0]);
    let ids: Vec<u32> = units.iter().map(|&s| id_of(&sim, s).0 as u32).collect();
    let mut from_team = Squad { ids: vec![99] };
    squad::squad_from_team(&mut from_team, &sim, TEAMS[1], false);
    assert_eq!(from_team.ids.len(), 3);
    squad::squad_from_team(&mut from_team, &sim, TEAMS[1], true);
    let mut sorted = from_team.ids.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, ids);
    // A squad of a group that is gone keeps what it had.
    squad::squad_from_ai_group(&mut from_team, &sim, 77);
    assert_eq!(from_team.ids.len(), 2);
    sim.destroy_object(units[0]);
    sim.process_destroy_list();
    assert_eq!(squad::all_objects(&mut from_team, &sim).len(), 1);
}

/// The orders only reach code that is behind a recorder or a stand-in unless the
/// generator's recorder is in place: a regeneration without it gives no orders.
#[test]
fn the_oracle_recorded_orders_and_ground_paths() {
    let count = |needle: &str| VECTORS.matches(needle).count();
    assert!(count(";o,") > 5_000);
    assert!(count(",path,0,") > 500);
    assert!(count("F,") > 300);
    assert!(count("#cmd ") > 2_000);
}

const EXTRA_OBJECTS: &str = "\
Object T_Bunker\n  KindOf = STRUCTURE SELECTABLE\n  Geometry = CYLINDER\n  GeometryMajorRadius = 20\n\
  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
  Behavior = GarrisonContain ModuleTag_Contain\n    ContainMax = 5\n  End\nEnd\n\
Object T_Rider\n  KindOf = INFANTRY SELECTABLE\n  Geometry = CYLINDER\n  GeometryMajorRadius = 5\n\
  TransportSlotCount = 1\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n  Locomotor = SET_NORMAL G_Foot\nEnd\n\
Object T_Lab\n  KindOf = STRUCTURE SELECTABLE\n  CommandSet = T_Set\n  Geometry = CYLINDER\n  GeometryMajorRadius = 20\n\
  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
  Behavior = ProductionUpdate ModuleTag_Prod\n  End\n\
  Behavior = StatusBitsUpgrade ModuleTag_Upg\n    TriggeredBy = T_Up\n    StatusToSet = HIJACKED\n  End\n\
  Behavior = SpecialAbility ModuleTag_Power\n    SpecialPowerTemplate = T_Power\n  End\nEnd\n";

fn extra_rules(sim: &mut SimContext) {
    let reader =
        |text: &'static str| IniReader::new("extra.ini", text.as_bytes(), IniLoadType::Overwrite);
    sim.rules
        .upgrade_center
        .load_from_ini(
            &mut reader(
                "Upgrade T_Up\n  Type = OBJECT\n  BuildTime = 5.0\n  BuildCost = 500\nEnd\n",
            ),
            &mut sim.rules.name_keys,
        )
        .expect("upgrade");
    sim.rules
        .special_power_store
        .load_from_ini(
            &mut reader("SpecialPower T_Power\n  Enum = SPECIAL_DAISY_CUTTER\nEnd\n"),
            &sim.rules.science_store,
            &mut sim.rules.name_keys,
        )
        .expect("special power");
    sim.rules.command_sets.insert(
        "T_Set",
        vec![CommandButton {
            command: CommandButtonType::Other,
            object: String::new(),
            upgrade: "T_Up".to_owned(),
        }],
    );
    sim.rules
        .load_thing_templates_from_ini(&mut reader(EXTRA_OBJECTS))
        .expect("templates");
}

fn spawn_named(sim: &mut SimContext, name: &str) -> ObjectSlot {
    let tid = sim
        .rules
        .thing_factory
        .find_template_id(name)
        .expect("template");
    sim.spawn_object(tid, TEAMS[1], Coord3D::new(500.0, 500.0, 0.0), 0.0)
}

#[test]
fn riders_get_the_orders_of_their_garrison() {
    let (mut sim, _, log) = small_world(false, &[]);
    extra_rules(&mut sim);
    let bunker = spawn_named(&mut sim, "T_Bunker");
    let rider = spawn_named(&mut sim, "T_Rider");
    let target = spawn_named(&mut sim, "T_Rider");
    sim.contain_add(bunker, rider);
    let (b, r, t) = (id_of(&sim, bunker), id_of(&sim, rider), id_of(&sim, target));
    let group = ai_group::create_group(&mut sim);
    ai_group::add(&mut sim, group, b);
    let taken = || std::mem::take(&mut *log.lock().expect("log"));
    let player = CommandSourceType::FromPlayer;

    ai_group::group_idle(&mut sim, group, player);
    assert_eq!(taken(), [format!("{} Idle 0 None None", r.0)]);

    // What the rider's weapons answer decides.
    let ability = |result| AttackAbility {
        able: TRUE,
        specific_object: result,
        weapon_against_target: result,
    };
    for (result, orders) in [
        (CanAttackResult::NotPossible, 0),
        (CanAttackResult::InvalidShot, 0),
        (CanAttackResult::PossibleAfterMoving, 1),
        (CanAttackResult::Possible, 1),
    ] {
        sim.world.attack_abilities.insert(r, ability(result));
        ai_group::group_attack_object(&mut sim, group, false, t, 2, player);
        let attacked = vec![format!("{} AttackObject 2 None None", r.0); orders];
        assert_eq!(taken(), attacked, "{result:?}");
        ai_group::group_attack_position(&mut sim, group, None, 3, player);
        let shot = vec![format!("{} AttackPosition 3 None None", r.0); orders];
        assert_eq!(taken(), shot, "{result:?}");
    }

    // A structure puts its riders out, not instantly.
    ai_group::group_evacuate(&mut sim, group, player);
    assert!(taken().is_empty());
    assert_eq!(ai_state(&mut sim, rider), state_id::EXIT);
}

#[test]
fn upgrades_and_special_powers_reach_the_members() {
    let (mut sim, _, log) = small_world(false, &[]);
    extra_rules(&mut sim);
    let lab = spawn_named(&mut sim, "T_Lab");
    let l = id_of(&sim, lab);
    let key = sim.rules.name_keys.lookup_key("T_Up").expect("key");
    let mut commands = CommandState::default();
    dispatch_command(&mut sim, &mut commands, &select(1, &[l]));
    let queued = |sim: &SimContext| {
        sim.production_module(lab)
            .map_or(0, |p| p.production_count())
    };
    let upgrade = ReplayCommand::new(1045, 1)
        .with(ReplayArg::object_id(l.0 as u32))
        .with(ReplayArg::integer(key.0));

    sim.set_player_cash(1, 499);
    dispatch_command(&mut sim, &mut commands, &upgrade);
    assert_eq!(queued(&sim), 0);
    sim.set_player_cash(1, 500);
    dispatch_command(&mut sim, &mut commands, &upgrade);
    assert_eq!(queued(&sim), 1);

    // Only the member with the power is asked about it; a named source is asked
    // alone, in a group of its own.
    let power = sim
        .rules
        .special_power_store
        .find_special_power_template("T_Power")
        .expect("power")
        .id();
    let order = |source: ObjectId| {
        ReplayCommand::new(1040, 1)
            .with(ReplayArg::integer(power as i32))
            .with(ReplayArg::integer(9))
            .with(ReplayArg::object_id(source.0 as u32))
    };
    log.lock().expect("log").clear();
    let before = sim.world.ai_groups.next_group_id;
    dispatch_command(&mut sim, &mut commands, &order(ObjectId::INVALID));
    assert_eq!(sim.world.ai_groups.next_group_id, before + 1);
    dispatch_command(&mut sim, &mut commands, &order(l));
    assert_eq!(sim.world.ai_groups.next_group_id, before + 3);
    let asked = format!("{} power {power} 9", l.0);
    assert_eq!(
        std::mem::take(&mut *log.lock().expect("log")),
        [asked.clone(), asked]
    );
    assert_eq!(
        ai_group::special_power_source(&sim, 999, power),
        None,
        "no group"
    );
}

#[test]
fn what_nothing_carries_out_is_listed() {
    let (mut sim, _, _) = small_world(false, &[0]);
    assert_eq!(sim.world.ai_groups.next_formation_id, 1);
    assert!(!squad::is_selectable(&sim, ObjectId(999)));
    let mut commands = CommandState::default();
    let science = ReplayCommand::new(1044, 1).with(ReplayArg::integer(3));
    dispatch_command(&mut sim, &mut commands, &science);
    assert_eq!(commands.not_carried_out, [1044]);
    // A command of no player makes no group, nor does one outside a game.
    let used = sim.world.ai_groups.next_group_id;
    dispatch_command(
        &mut sim,
        &mut commands,
        &ReplayCommand::new(command_target::MSG_DO_STOP, 9),
    );
    commands.in_game = false;
    dispatch_command(
        &mut sim,
        &mut commands,
        &ReplayCommand::new(command_target::MSG_DO_STOP, 1),
    );
    assert_eq!(sim.world.ai_groups.next_group_id, used);
    assert_eq!(
        commands.not_carried_out,
        [
            1044,
            command_target::MSG_DO_STOP,
            command_target::MSG_DO_STOP
        ]
    );
}

/// Which orders give up a temporary weapon lock, and the force attack on the
/// ground that first returns a busy group to its primary weapon.
#[test]
fn temporary_weapon_locks_follow_the_order() {
    let (mut sim, units, _) = small_world(false, &[3, 0]);
    let ids: Vec<ObjectId> = units.iter().map(|&s| id_of(&sim, s)).collect();
    let mut commands = CommandState::default();
    dispatch_command(&mut sim, &mut commands, &select(1, &ids[..1]));
    let weapon = |sim: &SimContext| {
        let set = &sim.object(units[0]).expect("object").weapon_set;
        (set.cur_weapon as i32, set.cur_weapon_locked_status as i32)
    };
    let fire_secondary = ReplayCommand::new(command_target::MSG_DO_WEAPON, 1)
        .with(ReplayArg::integer(1))
        .with(ReplayArg::integer(1));
    let on_other = |msg_type: i32| {
        let arg = ReplayArg::object_id(ids[1].0 as u32);
        let msg = ReplayCommand::new(msg_type, 1);
        if msg_type == command_target::MSG_ENTER {
            msg.with(arg).with(arg)
        } else {
            msg.with(arg)
        }
    };

    for (msg_type, lock) in [
        (command_target::MSG_DO_REPAIR, 1),
        (command_target::MSG_DOCK, 1),
        (command_target::MSG_GET_HEALED, 1),
        (command_target::MSG_ENTER, 0),
    ] {
        dispatch_command(&mut sim, &mut commands, &fire_secondary);
        assert_eq!(weapon(&sim), (1, 1));
        dispatch_command(&mut sim, &mut commands, &on_other(msg_type));
        assert_eq!(weapon(&sim), (1, lock), "{msg_type}");
    }

    let at_ground = ReplayCommand::new(command_target::MSG_DO_FORCE_ATTACK_GROUND, 1)
        .with(ReplayArg::location(600.0, 600.0, 0.0));
    dispatch_command(&mut sim, &mut commands, &fire_secondary);
    dispatch_command(&mut sim, &mut commands, &at_ground);
    assert_eq!(weapon(&sim), (1, 0), "an idle group keeps its weapon");
    let ms = sim.object(units[0]).and_then(|o| o.ai_slot).expect("AI");
    let ctx = sim.world.modules.meta(ms).context(ms);
    let _ = machine::set_state(&mut sim, ctx, state_id::BUSY);
    dispatch_command(&mut sim, &mut commands, &fire_secondary);
    dispatch_command(&mut sim, &mut commands, &at_ground);
    assert_eq!(
        weapon(&sim),
        (0, 0),
        "a busy group goes back to the primary"
    );
}
