//! `PKT-433` with the ported parts of `PKT-432` and `PKT-431`: the reference
//! `ScriptEngine` running random script lists
//! (`tests/cpp_harness/gen_script_engine.cpp`). Every frame compares the save
//! bytes, the logic random seed, the script and parameter state, the effects on
//! the world and the attack priorities.
//!
//! The reference library saves the retail layout (version 5), which the port
//! writes under `retail-xfer-save`. The default build writes version 6, which
//! is the same followed by the letter box flag.

use std::collections::BTreeSet;

use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::golden_json::{extract_vector_lines, get_i32, get_str, get_u32};
use zh_common::math::Coord3D;
use zh_common::random_value::PrngState;
use zh_common::types::{Int, ObjectId, Real, UnsignedInt};
use zh_common::xfer::{XferLoad, XferSave};
use zh_logic::gamelogic::global_data::GameDifficulty;
use zh_logic::scripts::script_types::{action_type as at, condition_type as ct};
use zh_logic::scripts::*;

const VECTORS: &str = include_str!("../../../docs/golden_vectors/SCRIPT_ENGINE.json");

const TEMPLATES: [&str; 5] = [
    "ScrTank",
    "ScrBarracks",
    "ScrDepot",
    "ScrCrate",
    "ScrVictim",
];
const PLAYERS: [&str; 3] = ["PlyrNeutral", "PlyrA", "PlyrB"];
const SIDES: [&str; 3] = ["Civilian", "America", "China"];

struct Tokens<'a>(std::str::SplitAsciiWhitespace<'a>);

impl<'a> Tokens<'a> {
    fn new(text: &'a str) -> Self {
        Self(text.split_ascii_whitespace())
    }
    fn word(&mut self) -> &'a str {
        self.0.next().expect("token")
    }
    fn text(&mut self) -> String {
        match self.word() {
            "-" => String::new(),
            w => w.replace('~', " "),
        }
    }
    fn int(&mut self) -> i64 {
        self.word().parse().expect("number")
    }
    fn i(&mut self) -> Int {
        self.int() as Int
    }
    fn u(&mut self) -> u32 {
        self.int() as u32
    }
    fn flag(&mut self) -> bool {
        self.int() != 0
    }
    fn real(&mut self) -> Real {
        Real::from_bits(self.u())
    }
}

#[derive(Clone, Default)]
struct PlayerState {
    money: UnsignedInt,
    power: PowerReading,
    any_objects: bool,
    any_build_facility: bool,
    buildings: Int,
    faction_buildings: Int,
    active: bool,
    attacked: u32,
    relations: [Int; 3],
    counts: [(Int, Int); 5],
    can_build_units: bool,
    can_build_base: bool,
    sciences: BTreeSet<Int>,
}

#[derive(Clone)]
struct TeamState {
    id: ScriptTeamId,
    active: bool,
    created: bool,
    state: String,
    owner: Int,
    any_objects: bool,
    any_units: bool,
}

#[derive(Clone)]
struct ObjectState {
    id: ObjectId,
    name: String,
    dead: bool,
    owner: Int,
    health: Real,
    initial: Real,
    ai_idle: Option<bool>,
    team: ScriptTeamId,
    structure: bool,
}

struct World {
    frame: UnsignedInt,
    rng: PrngState,
    players: [PlayerState; 3],
    protos: Vec<(String, TeamPrototypeInfo)>,
    teams: Vec<TeamState>,
    objects: Vec<ObjectState>,
    alpha: Int,
    beta: Int,
    crate_kind: Int,
    challenge: bool,
    /// The local player's default team.
    local_team: ScriptTeamId,
    unported: u32,
    /// What the hand written tests set; the reference scenarios leave them.
    waypoint: Option<Coord3D>,
    audio_ms: Real,
    waits: bool,
    enemy: Option<ScriptPlayer>,
    ui: Vec<ScriptUiCommand>,
    calls: Vec<String>,
}

impl World {
    fn new(seed: u32) -> Self {
        let mut rng = PrngState::default();
        rng.init_random_with_seed(seed);
        let mut players: [PlayerState; 3] = Default::default();
        for p in &mut players {
            p.can_build_units = true;
            p.can_build_base = true;
        }
        Self {
            frame: 0,
            rng,
            players,
            protos: Vec::new(),
            teams: Vec::new(),
            objects: Vec::new(),
            alpha: 0,
            beta: 0,
            crate_kind: 0,
            challenge: false,
            local_team: 0,
            unported: 0,
            waypoint: None,
            audio_ms: 0.0,
            waits: false,
            enemy: None,
            ui: Vec::new(),
            calls: Vec::new(),
        }
    }

    /// Reads a world snapshot, returning the object count change frame.
    fn read(&mut self, t: &mut Tokens) -> UnsignedInt {
        self.frame = t.u();
        let changed = t.u();
        for p in &mut self.players {
            p.money = t.u();
            p.power = PowerReading {
                production: t.i(),
                consumption: t.i(),
                sufficient: t.flag(),
                supply_ratio: t.real(),
            };
            p.any_objects = t.flag();
            p.any_build_facility = t.flag();
            p.buildings = t.i();
            p.faction_buildings = t.i();
            p.active = t.flag();
            p.attacked = t.u();
            for r in &mut p.relations {
                *r = t.i();
            }
            for c in &mut p.counts {
                *c = (t.i(), t.i());
            }
        }
        self.protos.clear();
        self.teams.clear();
        for _ in 0..t.int() {
            let name = t.text();
            let mut info = TeamPrototypeInfo {
                singleton: t.flag(),
                max_instances: t.i(),
                instances: Vec::new(),
            };
            for _ in 0..t.int() {
                let team = TeamState {
                    id: t.u(),
                    active: t.flag(),
                    created: t.flag(),
                    state: t.text(),
                    owner: t.i(),
                    any_objects: t.flag(),
                    any_units: t.flag(),
                };
                // The group queries are recomputed from the members.
                let _group = (t.flag(), t.flag());
                info.instances.push(team.id);
                self.teams.push(team);
            }
            self.protos.push((name, info));
        }
        self.objects = (0..t.int())
            .map(|_| ObjectState {
                id: ObjectId(t.i()),
                name: t.text(),
                dead: t.flag(),
                owner: t.i(),
                health: t.real(),
                initial: t.real(),
                ai_idle: match t.int() {
                    -1 => None,
                    v => Some(v != 0),
                },
                team: t.u(),
                structure: t.flag(),
            })
            .collect();
        changed
    }

    fn team(&self, id: ScriptTeamId) -> &TeamState {
        self.teams.iter().find(|t| t.id == id).expect("team")
    }

    fn object(&self, id: ObjectId) -> &ObjectState {
        self.objects.iter().find(|o| o.id == id).expect("object")
    }

    fn object_mut(&mut self, id: ObjectId) -> &mut ObjectState {
        self.objects
            .iter_mut()
            .find(|o| o.id == id)
            .expect("object")
    }

    /// Takes back what only the script actions change.
    fn keep(&mut self, old: [PlayerState; 3]) {
        for (p, old) in self.players.iter_mut().zip(old) {
            p.can_build_units = old.can_build_units;
            p.can_build_base = old.can_build_base;
            p.sciences = old.sciences;
        }
    }

    /// `getTeamAsAIGroup`: members with an AI, and structures.
    fn group(&self, team: ScriptTeamId) -> impl Iterator<Item = &ObjectState> {
        self.objects
            .iter()
            .filter(move |o| o.team == team && (o.ai_idle.is_some() || o.structure))
    }

    fn effects(&self) -> String {
        let mut out = String::new();
        for p in &self.players {
            out += &format!(
                "{} {}{}",
                p.money,
                u8::from(p.can_build_units),
                u8::from(p.can_build_base)
            );
            for r in p.relations {
                out += &format!(" {r}");
            }
            out.push(' ');
        }
        for team in &self.teams {
            let singleton = self
                .protos
                .iter()
                .any(|p| p.1.singleton && p.1.instances.contains(&team.id));
            if !singleton {
                continue;
            }
            let s = if team.state.is_empty() {
                "-"
            } else {
                &team.state
            };
            out += &format!("{s} ");
        }
        for o in &self.objects {
            out += &format!("{} ", o.ai_idle.map_or(-1, i32::from));
        }
        out
    }
}

impl ScriptWorld for World {
    fn frame(&self) -> UnsignedInt {
        self.frame
    }
    fn logic_random(&mut self, lo: Int, hi: Int) -> Int {
        self.rng.get_game_logic_random_value(lo, hi)
    }
    fn player_count(&self) -> Int {
        3
    }
    fn find_player(&self, name: &str) -> Option<ScriptPlayer> {
        PLAYERS.iter().position(|&p| p == name).map(|i| i as Int)
    }
    fn local_player(&self) -> ScriptPlayer {
        1
    }
    fn is_challenge_campaign(&self) -> bool {
        self.challenge
    }
    fn player_default_team(&self, player: ScriptPlayer) -> Option<ScriptTeamId> {
        assert_eq!(player, 1, "only the local player's is asked for");
        Some(self.local_team)
    }
    fn player_name(&self, player: ScriptPlayer) -> String {
        PLAYERS[player as usize].to_string()
    }
    fn player_is_human(&self, player: ScriptPlayer) -> bool {
        player == 1
    }
    fn player_is_skirmish_ai(&self, _player: ScriptPlayer) -> bool {
        false
    }
    fn player_is_active(&self, player: ScriptPlayer) -> bool {
        self.players[player as usize].active
    }
    fn player_difficulty(&self, player: ScriptPlayer) -> Option<GameDifficulty> {
        // PlyrB is a computer player, whose AI was made at normal difficulty.
        Some(GameDifficulty::Normal).filter(|_| player == 2)
    }
    fn player_current_enemy(&self, _player: ScriptPlayer) -> Option<ScriptPlayer> {
        self.enemy
    }
    fn player_start_index(&self, player: ScriptPlayer) -> Int {
        player - 1
    }
    fn player_side(&self, player: ScriptPlayer) -> String {
        SIDES[player as usize].to_string()
    }
    fn player_money(&self, player: ScriptPlayer) -> UnsignedInt {
        self.players[player as usize].money
    }
    fn player_power(&self, player: ScriptPlayer) -> PowerReading {
        self.players[player as usize].power
    }
    fn player_has_any_objects(&self, player: ScriptPlayer) -> bool {
        self.players[player as usize].any_objects
    }
    fn player_has_any_build_facility(&self, player: ScriptPlayer) -> bool {
        self.players[player as usize].any_build_facility
    }
    fn player_count_buildings(&self, player: ScriptPlayer) -> Int {
        self.players[player as usize].buildings
    }
    fn player_count_faction_buildings(&self, player: ScriptPlayer) -> Int {
        self.players[player as usize].faction_buildings
    }
    fn player_count_objects_by_template(
        &self,
        player: ScriptPlayer,
        templates: &[String],
        ignore_dead: bool,
    ) -> Vec<Int> {
        templates
            .iter()
            .map(|name| {
                let t = TEMPLATES.iter().position(|t| t == name).expect("template");
                let (all, living) = self.players[player as usize].counts[t];
                if ignore_dead {
                    living
                } else {
                    all
                }
            })
            .collect()
    }
    fn player_attacked_by(&self, player: ScriptPlayer, attacker: ScriptPlayer) -> bool {
        self.players[player as usize].attacked & (1 << attacker) != 0
    }
    fn money_withdraw(&mut self, player: ScriptPlayer, amount: UnsignedInt) {
        let money = &mut self.players[player as usize].money;
        *money -= amount.min(*money);
    }
    fn money_deposit(
        &mut self,
        player: ScriptPlayer,
        amount: UnsignedInt,
        sound: bool,
        score: bool,
    ) {
        self.players[player as usize].money =
            self.players[player as usize].money.wrapping_add(amount);
        self.calls.push(format!("deposit {sound} {score}"));
    }
    fn set_can_build_units(&mut self, player: ScriptPlayer, can: bool) {
        self.players[player as usize].can_build_units = can;
    }
    fn set_can_build_base(&mut self, player: ScriptPlayer, can: bool) {
        self.players[player as usize].can_build_base = can;
    }
    fn set_player_relationship(&mut self, from: ScriptPlayer, to: ScriptPlayer, relationship: Int) {
        self.players[from as usize].relations[to as usize] = relationship;
    }
    fn grant_science(&mut self, player: ScriptPlayer, science: Int) -> bool {
        science == self.alpha && self.players[player as usize].sciences.insert(science)
    }
    fn kill_player(&mut self, player: ScriptPlayer) {
        self.calls.push(format!("kill_player {player}"));
    }
    fn team_prototype(&self, name: &str) -> Option<TeamPrototypeInfo> {
        self.protos
            .iter()
            .find(|p| p.0 == name)
            .map(|p| p.1.clone())
    }
    fn team_name(&self, team: ScriptTeamId) -> String {
        self.protos
            .iter()
            .find(|p| p.1.instances.contains(&team))
            .map(|p| p.0.clone())
            .unwrap_or_default()
    }
    fn team_exists(&self, team: ScriptTeamId) -> bool {
        self.teams.iter().any(|t| t.id == team)
    }
    fn team_is_active(&self, team: ScriptTeamId) -> bool {
        self.team(team).active
    }
    fn team_is_created(&self, team: ScriptTeamId) -> bool {
        self.team(team).created
    }
    fn team_state(&self, team: ScriptTeamId) -> String {
        self.team(team).state.clone()
    }
    fn team_controlling_player(&self, team: ScriptTeamId) -> Option<ScriptPlayer> {
        Some(self.team(team).owner).filter(|&p| p >= 0)
    }
    fn team_has_any_objects(&self, team: ScriptTeamId) -> bool {
        self.team(team).any_objects
    }
    fn team_has_any_units(&self, team: ScriptTeamId) -> bool {
        self.team(team).any_units
    }
    fn team_members(&self, team: ScriptTeamId) -> Vec<ObjectId> {
        self.objects
            .iter()
            .filter(|o| o.team == team)
            .map(|o| o.id)
            .collect()
    }
    fn update_team_states(&mut self) {}
    fn team_group_is_idle(&mut self, team: ScriptTeamId) -> bool {
        self.group(team)
            .all(|o| o.ai_idle.map_or(true, |idle| idle || o.dead))
    }
    fn team_group_is_ai_dead(&mut self, team: ScriptTeamId) -> bool {
        self.group(team).all(|o| o.dead)
    }
    fn team_group_idle(&mut self, team: ScriptTeamId) {
        let members: Vec<ObjectId> = self.group(team).map(|o| o.id).collect();
        for id in members {
            self.object_ai_idle(id);
        }
    }
    fn set_team_state(&mut self, team: ScriptTeamId, state: &str) {
        let t = self.teams.iter_mut().find(|t| t.id == team).expect("team");
        t.state = state.to_string();
    }
    fn delete_team(&mut self, team: ScriptTeamId, ignore_dead: bool) {
        self.calls.push(format!("delete_team {team} {ignore_dead}"));
    }
    fn kill_team(&mut self, team: ScriptTeamId) {
        self.calls.push(format!("kill_team {team}"));
    }
    fn named_objects(&self) -> Vec<(String, ObjectId)> {
        // The object list runs from the newest.
        self.objects
            .iter()
            .rev()
            .map(|o| (o.name.clone(), o.id))
            .collect()
    }
    fn object_exists(&self, id: ObjectId) -> bool {
        self.objects.iter().any(|o| o.id == id)
    }
    fn object_name(&self, id: ObjectId) -> String {
        self.object(id).name.clone()
    }
    fn object_is_effectively_dead(&self, id: ObjectId) -> bool {
        self.object(id).dead
    }
    fn object_controlling_player(&self, id: ObjectId) -> Option<ScriptPlayer> {
        Some(self.object(id).owner).filter(|&p| p >= 0)
    }
    fn object_health(&self, id: ObjectId) -> Option<(Real, Real)> {
        let o = self.object(id);
        Some((o.health, o.initial))
    }
    fn object_ai_is_idle(&self, id: ObjectId) -> Option<bool> {
        self.object(id).ai_idle
    }
    fn object_ai_idle(&mut self, id: ObjectId) {
        let o = self.object_mut(id);
        if o.ai_idle.is_some() {
            o.ai_idle = Some(true);
        }
    }
    fn object_ai_guard_here(&mut self, id: ObjectId, _choose_normal_locomotor: bool) {
        let o = self.object_mut(id);
        if o.ai_idle.is_some() {
            o.ai_idle = Some(false);
        }
    }
    fn set_object_name(&mut self, id: ObjectId, name: &str) {
        self.object_mut(id).name = name.to_string();
    }
    fn copy_indicator_color(&mut self, from: ObjectId, to: ObjectId) {
        self.calls.push(format!("color {} {}", from.0, to.0));
    }
    fn destroy_object(&mut self, id: ObjectId) {
        self.calls.push(format!("destroy {}", id.0));
    }
    fn kill_object(&mut self, id: ObjectId) {
        self.calls.push(format!("kill {}", id.0));
    }
    fn set_all_difficulty_bonuses(&mut self, enable: bool) {
        self.calls.push(format!("bonuses {enable}"));
    }
    fn has_thing_template(&self, name: &str) -> bool {
        TEMPLATES.contains(&name)
    }
    fn thing_template_index(&self, name: &str) -> Option<u32> {
        TEMPLATES.iter().position(|&t| t == name).map(|i| i as u32)
    }
    fn thing_template_name(&self, index: u32) -> String {
        TEMPLATES[index as usize].to_string()
    }
    fn thing_templates_of_kind(&self, kind: Int) -> Vec<u32> {
        if kind == self.crate_kind {
            vec![3]
        } else {
            Vec::new()
        }
    }
    fn science_from_name(&self, name: &str) -> Option<Int> {
        match name {
            "SCIENCE_Alpha" => Some(self.alpha),
            "SCIENCE_Beta" => Some(self.beta),
            _ => None,
        }
    }
    fn science_name(&self, science: Int) -> String {
        if science == self.alpha {
            "SCIENCE_Alpha".to_string()
        } else {
            "SCIENCE_Beta".to_string()
        }
    }
    fn waypoint_location(&self, _name: &str) -> Option<Coord3D> {
        self.waypoint
    }
    fn do_shroud_reveal(&mut self, at: Coord3D, radius: Real, player: ScriptPlayer) {
        self.calls
            .push(format!("reveal {} {radius} {player}", at.x));
    }
    fn undo_shroud_reveal(&mut self, at: Coord3D, radius: Real, player: ScriptPlayer) {
        self.calls
            .push(format!("unreveal {} {radius} {player}", at.x));
    }
    fn audio_length_ms(&self, _event: &str) -> Real {
        self.audio_ms
    }
    fn music_track_completed(&self, _track: &str, _times: Int) -> bool {
        false
    }
    fn set_team_attack_priority(&mut self, team: ScriptTeamId, set: Option<&str>, members: &str) {
        self.calls
            .push(format!("priority {team} {set:?} {members}"));
    }
    fn set_warehouse_cash(&mut self, id: ObjectId, cash: Int) {
        self.calls.push(format!("warehouse {} {cash}", id.0));
    }
    fn ui(&mut self, command: ScriptUiCommand) {
        self.ui.push(command);
    }
    fn exit_game(&mut self) {
        self.calls.push("exit".to_string());
    }
    fn command_button_ready(&mut self, team: &Parameter, button: &Parameter, all: bool) -> bool {
        self.calls.push(format!(
            "button {} {} {all}",
            team.string_val, button.string_val
        ));
        !self.waits
    }
    fn team_is_contained(&mut self, team: &Parameter, all: bool) -> bool {
        self.calls
            .push(format!("contained {} {all}", team.string_val));
        self.waits
    }
    fn unported_condition(&mut self, _condition: &Condition) -> bool {
        self.unported += 1;
        self.waits
    }
    fn unported_action(&mut self, _action: &ScriptAction) {
        self.unported += 1;
    }
    // Computer players are not part of the recording.
    fn ai_build_building(&mut self, _player: ScriptPlayer, _template: &str) {}
    fn ai_build_by_supplies(&mut self, _player: ScriptPlayer, _cash: Int, _template: &str) {}
}

fn read_parms(t: &mut Tokens) -> (Int, Vec<Parameter>) {
    let kind = t.i();
    let parms = (0..t.int())
        .map(|_| {
            let mut p = Parameter::new(ParameterType::from_i32(t.i()), t.i());
            p.real_val = t.real();
            p.string_val = t.text();
            p.coord = Coord3D {
                x: t.real(),
                y: t.real(),
                z: t.real(),
            };
            p
        })
        .collect();
    (kind, parms)
}

fn read_actions(t: &mut Tokens, templates: &ScriptTemplateRegistry) -> Vec<ScriptAction> {
    (0..t.int())
        .map(|_| {
            let (kind, parms) = read_parms(t);
            let mut a = ScriptAction::new(kind, templates);
            // The reference's templates and the port's agree on the shape.
            assert_eq!(a.parms.len(), parms.len(), "action {kind}");
            a.parms = parms;
            a
        })
        .collect()
}

fn read_script(t: &mut Tokens, templates: &ScriptTemplateRegistry) -> Script {
    let mut s = Script::new();
    s.script_name = t.text();
    s.is_active = u8::from(t.flag());
    s.is_one_shot = u8::from(t.flag());
    s.is_subroutine = u8::from(t.flag());
    s.easy = u8::from(t.flag());
    s.normal = u8::from(t.flag());
    s.hard = u8::from(t.flag());
    s.delay_evaluation_seconds = t.i();
    for _ in 0..t.int() {
        let mut or = OrCondition::new();
        for _ in 0..t.int() {
            let (kind, parms) = read_parms(t);
            let mut c = Condition::new(kind, templates);
            assert_eq!(c.parms.len(), parms.len(), "condition {kind}");
            c.parms = parms;
            or.conditions.push(c);
        }
        s.or_conditions.push(or);
    }
    s.actions = read_actions(t, templates);
    s.actions_false = read_actions(t, templates);
    s
}

fn put_state(out: &mut String, s: &Script) {
    let team = if s.condition_team_name.is_empty() {
        "-".to_string()
    } else {
        s.condition_team_name.replace(' ', "~")
    };
    *out += &format!(" {} {} {team}", s.is_active, s.frame_to_evaluate_at);
    for or in &s.or_conditions {
        for c in &or.conditions {
            *out += &format!(" {} {}", c.custom_data, c.custom_frame);
            for p in &c.parms {
                *out += &format!(" {}", p.int_val);
            }
        }
    }
    for a in s.actions.iter().chain(&s.actions_false) {
        for p in &a.parms {
            *out += &format!(" {}", p.int_val);
        }
    }
}

fn script_state(engine: &ScriptEngine) -> String {
    let mut out = String::new();
    for list in engine.sides.iter().flatten() {
        for s in &list.scripts {
            put_state(&mut out, s);
        }
        for g in &list.groups {
            for s in &g.scripts {
                put_state(&mut out, s);
            }
        }
        for g in &list.groups {
            out += &format!(" {}", g.is_group_active);
        }
    }
    out
}

fn save_bytes(engine: &mut ScriptEngine, world: &World) -> Vec<u8> {
    let mut xfer = XferSave::new();
    engine.xfer(&mut xfer, world).expect("save");
    xfer.into_bytes()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The hook calls the generator made between two frames.
fn apply_ops(engine: &mut ScriptEngine, world: &mut World, ops: &str) {
    let mut t = Tokens::new(ops);
    for _ in 0..t.int() {
        match t.word() {
            "world" => {
                let kept = world.players.clone();
                engine.frame_object_count_changed = world.read(&mut t);
                world.keep(kept);
            }
            "video" => engine.notify_of_completed_video(t.word()),
            "special" => {
                let kind = t.int();
                let player = t.i();
                let name = t.word();
                let source = ObjectId(t.i());
                match kind {
                    0 => engine.notify_of_triggered_special_power(player, name, source),
                    1 => engine.notify_of_midway_special_power(player, name, source),
                    2 => engine.notify_of_completed_special_power(player, name, source),
                    _ => engine.notify_of_completed_upgrade(player, name, source),
                }
            }
            "science" => {
                let player = t.i();
                engine.notify_of_acquired_science(player, t.i());
            }
            "ui" => {
                let hook = t.text();
                engine.signal_ui_interact(&hook);
            }
            "add" => {
                let id = ObjectId(t.i());
                let name = t.text();
                world.set_object_name(id, &name);
                engine.add_object_to_cache(&name, id);
            }
            "remove" => engine.remove_object_from_cache(ObjectId(t.i())),
            "transfer" => {
                let id = ObjectId(t.i());
                let name = t.text();
                engine.transfer_object_name(world, &name, id);
            }
            "run" => {
                let name = t.text();
                let team = Some(t.u()).filter(|&id| id != 0);
                engine.run_script(world, &name, team);
            }
            "runobj" => {
                let name = t.text();
                let id = ObjectId(t.i());
                engine.run_object_script(world, &name, Some(id));
            }
            "clearteam" => engine.clear_team_flags(),
            "teamgone" => engine.notify_of_team_destruction(t.u()),
            "topple" => {
                let name = t.text();
                let mut d = Coord3D {
                    x: 3.0,
                    y: 4.0,
                    z: 0.5,
                };
                engine.adjust_topple_direction(&name, &mut d);
                let want = [t.u(), t.u(), t.u()];
                assert_eq!(
                    [d.x.to_bits(), d.y.to_bits(), d.z.to_bits()],
                    want,
                    "topple"
                );
            }
            other => panic!("unknown op {other}"),
        }
    }
}

fn kind_lines(kind: &str) -> Vec<&'static str> {
    extract_vector_lines(VECTORS)
        .into_iter()
        .filter(|l| get_str(l, "t") == Some(kind))
        .collect()
}

#[test]
fn templates_match_the_reference() {
    let mut names = NameKeyGenerator::default();
    let registry = script_template_registry(&mut names);
    let lines = kind_lines("template");
    assert_eq!(
        lines.len(),
        registry.conditions.len() + registry.actions.len()
    );
    for line in lines {
        let mut t = Tokens::new(get_str(line, "d").unwrap());
        let list = if t.int() == 0 {
            &registry.conditions
        } else {
            &registry.actions
        };
        let index = t.int() as usize;
        let ours = &list[index];
        assert_eq!(ours.internal_name, t.text(), "{line}");
        assert_eq!(ours.num_parameters, t.i(), "{line}");
        for i in 0..ours.num_parameters as usize {
            assert_eq!(ours.parameters[i] as Int, t.i(), "{line}");
        }
    }
}

#[test]
fn script_engine_matches_the_reference() {
    let mut names = NameKeyGenerator::default();
    let templates = script_template_registry(&mut names);
    let mut engine = ScriptEngine::new();
    let mut world = World::new(0);
    let mut scenarios = 0;
    let mut frames = 0;
    let mut met_conditions = BTreeSet::new();
    let mut met_actions = BTreeSet::new();
    for line in extract_vector_lines(VECTORS) {
        let seed = get_i32(line, "seed").unwrap();
        match get_str(line, "t").unwrap() {
            "setup" => {
                let mut t = Tokens::new(get_str(line, "d").unwrap());
                world = World::new(t.u());
                let difficulty = match t.int() {
                    0 => GameDifficulty::Easy,
                    1 => GameDifficulty::Normal,
                    _ => GameDifficulty::Hard,
                };
                let _frame = t.u();
                world.crate_kind = t.i();
                world.alpha = t.i();
                world.beta = t.i();
                world.challenge = t.flag();
                world.local_team = t.u();
                let sides: Vec<Option<ScriptList>> = (0..t.int())
                    .map(|_| {
                        if !t.flag() {
                            return None;
                        }
                        let mut list = ScriptList::new();
                        for _ in 0..t.int() {
                            list.scripts.push(read_script(&mut t, &templates));
                        }
                        for _ in 0..t.int() {
                            let mut group = ScriptGroup::new();
                            group.group_name = t.text();
                            group.is_group_active = u8::from(t.flag());
                            group.is_group_subroutine = u8::from(t.flag());
                            for _ in 0..t.int() {
                                group.scripts.push(read_script(&mut t, &templates));
                            }
                            list.groups.push(group);
                        }
                        Some(list)
                    })
                    .collect();
                for s in sides.iter().flatten().flat_map(|l| {
                    l.scripts
                        .iter()
                        .chain(l.groups.iter().flat_map(|g| &g.scripts))
                }) {
                    for c in s.or_conditions.iter().flat_map(|o| &o.conditions) {
                        met_conditions.insert(c.condition_type);
                    }
                    for a in s.actions.iter().chain(&s.actions_false) {
                        met_actions.insert(a.action_type);
                    }
                }
                world.read(&mut Tokens::new(get_str(line, "w").unwrap()));
                engine = ScriptEngine::new();
                engine.difficulty = difficulty;
                engine.new_map(&mut world, sides);
                assert_eq!(
                    world.rng.get_game_logic_random_seed_crc(),
                    get_u32(line, "rng").unwrap(),
                    "seed {seed}: new map random state"
                );
                scenarios += 1;
            }
            "frame" => {
                let at_line = format!("seed {seed} frame {}", get_u32(line, "frame").unwrap());
                apply_ops(&mut engine, &mut world, get_str(line, "ops").unwrap());
                let kept = world.players.clone();
                engine.frame_object_count_changed =
                    world.read(&mut Tokens::new(get_str(line, "w").unwrap()));
                world.keep(kept);
                engine.update(&mut world);

                same(
                    &world.effects(),
                    get_str(line, "fx").unwrap(),
                    &at_line,
                    "effects",
                );
                same(
                    &script_state(&engine),
                    get_str(line, "state").unwrap(),
                    &at_line,
                    "state",
                );
                let saved = save_bytes(&mut engine, &world);
                let mut want = get_str(line, "save").unwrap().to_string();
                if !cfg!(feature = "retail-xfer-save") {
                    assert!(want.starts_with("05"));
                    want = format!("06{}{:02x}", &want[2..], u8::from(engine.letter_box_active));
                }
                same(&hex(&saved), &want, &at_line, "save");
                // In the even scenarios the reference's AI draws numbers too.
                if seed % 2 == 1 {
                    assert_eq!(
                        world.rng.get_game_logic_random_seed_crc(),
                        get_u32(line, "rng").unwrap(),
                        "{at_line}: random state"
                    );
                }
                let mut priorities = String::new();
                for set in ["AP0", "AP1", "AP2", "AP3", "Unset"] {
                    for t in 0..TEMPLATES.len() as u32 {
                        priorities += &format!("{} ", engine.attack_info(set).priority(Some(t)));
                    }
                }
                same(
                    &priorities,
                    get_str(line, "prio").unwrap(),
                    &at_line,
                    "priorities",
                );
                assert_eq!(world.unported, 0, "{at_line}");
                assert!(engine.unported_conditions.is_empty(), "{at_line}");
                assert!(engine.unported_actions.is_empty(), "{at_line}");

                // Loading the bytes into a fresh engine gives them back.
                let mut loaded = ScriptEngine::new();
                loaded.sides = engine.sides.clone();
                let mut xfer = XferLoad::new(&saved);
                loaded.xfer(&mut xfer, &world).expect("load");
                // As the reference does, loading restarts a fade in that has ended.
                if engine.fade == ScriptFade::None {
                    assert_eq!(
                        (loaded.fade, loaded.fade_frames_decrease, loaded.min_fade),
                        (ScriptFade::Multiply, 33, 1.0),
                        "{at_line}: fade after load"
                    );
                    loaded.fade = engine.fade;
                    loaded.min_fade = engine.min_fade;
                    loaded.max_fade = engine.max_fade;
                    loaded.cur_fade_value = engine.cur_fade_value;
                    loaded.cur_fade_frame = engine.cur_fade_frame;
                    loaded.fade_frames_increase = engine.fade_frames_increase;
                    loaded.fade_frames_hold = engine.fade_frames_hold;
                    loaded.fade_frames_decrease = engine.fade_frames_decrease;
                }
                assert_eq!(save_bytes(&mut loaded, &world), saved, "{at_line}: reload");
                frames += 1;
            }
            _ => {}
        }
    }
    assert_eq!((scenarios, frames), (120, 1440));
    // Every ported condition and action is in the lists, but for those the
    // reference cannot run headless.
    for c in PORTED_CONDITIONS {
        assert!(
            met_conditions.contains(c)
                || matches!(
                    *c,
                    ct::DEFUNCT_PLAYER_SELECTED_GENERAL
                        | ct::DEFUNCT_PLAYER_SELECTED_GENERAL_FROM_NAMED
                        | ct::MUSIC_TRACK_HAS_COMPLETED
                ),
            "condition {c} never generated"
        );
    }
    let headless = [
        at::CAMERA_LETTERBOX_BEGIN,
        at::CAMERA_LETTERBOX_END,
        at::VICTORY,
        at::QUICKVICTORY,
        at::DEFEAT,
        at::LOCALDEFEAT,
        at::PLAYER_KILL,
        at::TEAM_KILL,
        at::TEAM_DELETE,
        at::TEAM_DELETE_LIVING,
        at::NAMED_DELETE,
        at::NAMED_KILL,
        // Added for the first frames of the retail games and not in the
        // recording: `client_and_object_actions_of_the_first_frames`.
        at::TEAM_APPLY_ATTACK_PRIORITY_SET,
        at::MUSIC_SET_TRACK,
        at::SHOW_MILITARY_CAPTION,
        at::WAREHOUSE_SET_VALUE,
        at::AUDIO_OVERRIDE_VOLUME_TYPE,
        at::SHOW_WEATHER,
        // Orders to computer players, carried out by the host:
        // `golden_game_frames`.
        at::SKIRMISH_BUILD_BUILDING,
        at::AI_PLAYER_BUILD_SUPPLY_CENTER,
    ];
    for a in PORTED_ACTIONS {
        assert!(
            met_actions.contains(a) || headless.contains(a),
            "action {a} never generated"
        );
    }
}

/// Token for token, naming the first that differs.
fn same(ours: &str, want: &str, at_line: &str, what: &str) {
    if ours.trim() == want.trim() {
        return;
    }
    let (a, b): (Vec<&str>, Vec<&str>) = (
        ours.split_whitespace().collect(),
        want.split_whitespace().collect(),
    );
    let i = a
        .iter()
        .zip(&b)
        .position(|(x, y)| x != y)
        .unwrap_or(a.len().min(b.len()));
    let mut j = 0;
    if a.len() == 1 && b.len() == 1 {
        j = a[0]
            .bytes()
            .zip(b[0].bytes())
            .position(|(x, y)| x != y)
            .unwrap_or(0);
    }
    panic!(
        "{at_line}: {what} differs at token {i} (byte {}), ours {:?} want {:?}; lengths {} {}",
        j / 2,
        &a.get(i)
            .map(|s| &s[j.saturating_sub(16)..s.len().min(j + 48)]),
        &b.get(i)
            .map(|s| &s[j.saturating_sub(16)..s.len().min(j + 48)]),
        ours.len(),
        want.len()
    );
}

// What follows is read from the reference source, not recorded from it: the
// paths its library cannot run headless, and the routing of what is not ported.

fn text(value: &str) -> Parameter {
    let mut p = Parameter::new(ParameterType::TextString, 0);
    p.string_val = value.to_string();
    p
}

fn number(value: Int) -> Parameter {
    Parameter::new(ParameterType::Int, value)
}

fn action(kind: Int, parms: Vec<Parameter>) -> ScriptAction {
    let mut a = ScriptAction::new(kind, &ScriptTemplateRegistry::new());
    a.parms = parms;
    a
}

fn condition(kind: Int, parms: Vec<Parameter>) -> Condition {
    let mut c = Condition::new(kind, &ScriptTemplateRegistry::new());
    c.parms = parms;
    c
}

/// A world with one team `T` (id 7) of two AI objects and a building.
fn small_world() -> World {
    let mut world = World::new(1);
    world.protos.push((
        "T".to_string(),
        TeamPrototypeInfo {
            singleton: true,
            max_instances: 1,
            instances: vec![7],
        },
    ));
    world.teams.push(TeamState {
        id: 7,
        active: true,
        created: false,
        state: String::new(),
        owner: 2,
        any_objects: true,
        any_units: true,
    });
    for (id, name) in [(1, "A"), (2, ""), (3, "B")] {
        world.objects.push(ObjectState {
            id: ObjectId(id),
            name: name.to_string(),
            dead: false,
            owner: 2,
            health: 1.0,
            initial: 1.0,
            ai_idle: Some(true).filter(|_| id != 3),
            team: 7,
            structure: id == 3,
        });
    }
    world
}

/// One script `Main` on the first side, run by every update.
fn engine_with(
    world: &mut World,
    conditions: Vec<Condition>,
    actions: Vec<ScriptAction>,
) -> ScriptEngine {
    let mut main = Script::new();
    main.script_name = "Main".to_string();
    main.is_one_shot = 0;
    main.or_conditions.push(OrCondition { conditions });
    main.actions = actions;
    let mut list = ScriptList::new();
    list.scripts.push(main);
    let mut engine = ScriptEngine::new();
    engine.new_map(world, vec![Some(list)]);
    engine
}

fn always() -> Vec<Condition> {
    vec![condition(ct::CONDITION_TRUE, Vec::new())]
}

#[test]
fn what_is_not_ported_is_told_apart() {
    let mut world = small_world();
    let mut engine = engine_with(
        &mut world,
        vec![condition(ct::NAMED_SELECTED, vec![text("A")])],
        vec![action(at::DEBUG_STRING, vec![text("x")])],
    );
    // The host's answer decides the condition.
    engine.update(&mut world);
    assert_eq!(world.unported, 1);
    assert_eq!(engine.unported_conditions[&ct::NAMED_SELECTED], 1);
    assert!(engine.unported_actions.is_empty());
    world.waits = true;
    engine.update(&mut world);
    assert_eq!(world.unported, 3);
    assert_eq!(engine.unported_conditions[&ct::NAMED_SELECTED], 2);
    assert_eq!(engine.unported_actions[&at::DEBUG_STRING], 1);
}

#[test]
fn every_type_is_ported_or_reaches_the_host() {
    let mut world = small_world();
    let mut engine = ScriptEngine::new();
    for kind in 0..109 {
        let before = world.unported;
        let ported = PORTED_CONDITIONS.contains(&kind);
        if !ported {
            engine.evaluate_condition(&mut world, &mut condition(kind, Vec::new()));
        }
        assert_eq!(
            world.unported - before,
            u32::from(!ported),
            "condition {kind}"
        );
    }
    for kind in 0..344 {
        if !PORTED_ACTIONS.contains(&kind) {
            let before = world.unported;
            engine.execute_action(&mut world, &mut action(kind, Vec::new()));
            assert_eq!(world.unported - before, 1, "action {kind}");
        }
    }
    assert_eq!(PORTED_CONDITIONS.len(), 49);
    assert_eq!(PORTED_ACTIONS.len(), 72);
    let distinct: BTreeSet<Int> = PORTED_ACTIONS.iter().copied().collect();
    assert_eq!(distinct.len(), PORTED_ACTIONS.len());
    // None of the ported ones goes to the host.
    let mut world = small_world();
    for &kind in PORTED_CONDITIONS {
        engine.evaluate_condition(&mut world, &mut condition(kind, Vec::new()));
    }
    for &kind in PORTED_ACTIONS {
        engine.execute_action(&mut world, &mut action(kind, Vec::new()));
    }
    assert_eq!(world.unported, 0);
}

#[test]
fn client_and_object_actions_of_the_first_frames() {
    let mut world = small_world();
    let mut engine = ScriptEngine::new();
    for kind in [
        at::SHOW_MILITARY_CAPTION,
        at::AUDIO_OVERRIDE_VOLUME_TYPE,
        at::SHOW_WEATHER,
        at::MUSIC_SET_TRACK,
    ] {
        engine.execute_action(&mut world, &mut action(kind, vec![text("Track")]));
        assert_eq!(world.ui.pop(), Some(ScriptUiCommand::Client(kind)), "{kind}");
    }
    // Only the track change leaves state behind.
    assert_eq!(engine.current_track_name, "Track");

    // An unknown unit, team or priority set does what `updateTeam...` does.
    let before = world.calls.len();
    engine.execute_action(
        &mut world,
        &mut action(at::WAREHOUSE_SET_VALUE, vec![text("nobody"), number(7)]),
    );
    engine.execute_action(
        &mut world,
        &mut action(
            at::TEAM_APPLY_ATTACK_PRIORITY_SET,
            vec![text("no such team"), text("set")],
        ),
    );
    assert_eq!(world.calls.len(), before);
    engine.add_object_to_cache("depot", ObjectId(5));
    engine.execute_action(
        &mut world,
        &mut action(at::WAREHOUSE_SET_VALUE, vec![text("depot"), number(7)]),
    );
    assert_eq!(world.calls.last().map(String::as_str), Some("warehouse 5 7"));
    assert_eq!(world.unported, 0);
}

#[test]
fn end_of_game_timers() {
    use ScriptUiCommand::*;
    for (kind, ui, end, close) in [
        (
            at::VICTORY,
            vec![CloseWindows, DisableInput, ShowVictory, SetVictorious(true)],
            120,
            -1,
        ),
        (
            at::DEFEAT,
            vec![CloseWindows, DisableInput, ShowDefeat, SetVictorious(false)],
            120,
            -1,
        ),
        (
            at::QUICKVICTORY,
            vec![CloseWindows, DisableInput, SetVictorious(true)],
            1,
            -1,
        ),
        (
            at::LOCALDEFEAT,
            vec![CloseWindows, ShowLocalDefeat, SetVictorious(false)],
            -1,
            120,
        ),
    ] {
        let mut world = small_world();
        let mut engine = ScriptEngine::new();
        engine.execute_action(&mut world, &mut action(kind, Vec::new()));
        assert_eq!(world.ui, ui, "{kind}");
        assert_eq!(
            (engine.end_game_timer, engine.close_window_timer),
            (end, close)
        );
        assert_eq!(
            engine.has_shown_mp_local_defeat_window(),
            kind == at::LOCALDEFEAT
        );
    }
}

#[test]
fn timers_run_out_the_game_and_close_the_window() {
    let mut world = small_world();
    let mut engine = engine_with(
        &mut world,
        always(),
        vec![action(at::SET_COUNTER, vec![text("n"), number(1)])],
    );
    engine.start_close_window_timer();
    for _ in 0..119 {
        engine.update(&mut world);
    }
    assert!(world.ui.is_empty());
    engine.update(&mut world);
    assert_eq!(world.ui, [ScriptUiCommand::CloseWindows]);
    engine.update(&mut world);
    assert_eq!(world.ui.len(), 1);

    engine.start_end_game_timer();
    let n = engine.allocate_counter("n") as usize;
    engine.counters[n].value = 0;
    for _ in 0..119 {
        engine.update(&mut world);
    }
    assert!(world.calls.is_empty());
    engine.update(&mut world);
    assert_eq!(world.calls, ["exit"]);
    // No script runs while the game is ending.
    assert_eq!(engine.counter("n").map(|c| c.value), Some(0));
    engine.update(&mut world);
    assert_eq!(world.calls, ["exit"]);
}

#[test]
fn pass_through_commands() {
    let mut world = small_world();
    let mut engine = engine_with(&mut world, always(), Vec::new());
    engine.create_named_cache(&world);
    for (kind, name) in [
        (at::NAMED_DELETE, "A"),
        (at::NAMED_KILL, "B"),
        (at::TEAM_DELETE, "T"),
        (at::TEAM_DELETE_LIVING, "T"),
        (at::TEAM_KILL, "T"),
        (at::PLAYER_KILL, "PlyrB"),
        (at::NAMED_DELETE, "Nobody"),
        (at::TEAM_KILL, "Nobody"),
        (at::PLAYER_KILL, "Nobody"),
    ] {
        engine.execute_action(&mut world, &mut action(kind, vec![text(name)]));
    }
    assert_eq!(
        world.calls,
        [
            "destroy 1",
            "kill 3",
            "delete_team 7 false",
            "delete_team 7 true",
            "kill_team 7",
            "kill_player 2"
        ]
    );
    world.calls.clear();
    engine.execute_action(
        &mut world,
        &mut action(at::PLAYER_SET_MONEY, vec![text("PlyrB"), number(50)]),
    );
    engine.execute_action(
        &mut world,
        &mut action(at::PLAYER_GIVE_MONEY, vec![text("PlyrB"), number(5)]),
    );
    engine.execute_action(
        &mut world,
        &mut action(at::PLAYER_GIVE_MONEY, vec![text("PlyrB"), number(-100)]),
    );
    assert_eq!(world.players[2].money, 0);
    assert_eq!(world.calls, ["deposit false false", "deposit true true"]);
    world.calls.clear();
    engine.execute_action(
        &mut world,
        &mut action(at::OBJECT_ALLOW_BONUSES, vec![number(0)]),
    );
    assert_eq!(world.calls, ["bonuses false"]);
    assert!(!engine.objects_receive_difficulty_bonus);
    for (kind, active) in [
        (at::CAMERA_LETTERBOX_BEGIN, true),
        (at::CAMERA_LETTERBOX_END, false),
    ] {
        engine.execute_action(&mut world, &mut action(kind, Vec::new()));
        assert_eq!(engine.letter_box_active, active);
        assert_eq!(world.ui.pop(), Some(ScriptUiCommand::LetterBox(active)));
    }
}

#[test]
fn map_reveals_need_a_waypoint_and_a_player() {
    let mut world = small_world();
    world.waypoint = Some(Coord3D {
        x: 4.0,
        y: 0.0,
        z: 0.0,
    });
    let mut engine = ScriptEngine::new();
    let mut radius = Parameter::new(ParameterType::Real, 0);
    radius.real_val = 2.5;
    let reveal = |player: &str| {
        action(
            at::MAP_REVEAL_PERMANENTLY_AT_WAYPOINT,
            vec![text("W"), radius.clone(), text(player), text("R")],
        )
    };
    engine.execute_action(&mut world, &mut reveal("PlyrA"));
    // A name already defined keeps its first meaning.
    engine.execute_action(&mut world, &mut reveal("PlyrB"));
    engine.execute_action(
        &mut world,
        &mut action(at::MAP_UNDO_REVEAL_PERMANENTLY_AT_WAYPOINT, vec![text("R")]),
    );
    assert_eq!(
        world.calls,
        ["reveal 4 2.5 1", "reveal 4 2.5 1", "unreveal 4 2.5 1"]
    );
    assert!(engine.named_reveals.is_empty());
}

#[test]
fn sounds_finish_after_their_length() {
    let mut world = small_world();
    world.audio_ms = 100.0;
    world.frame = 10;
    let mut engine = ScriptEngine::new();
    for kind in [ct::HAS_FINISHED_SPEECH, ct::HAS_FINISHED_AUDIO] {
        world.frame = 10;
        let mut c = condition(kind, vec![text("s")]);
        // Three frames of 33 and a third milliseconds.
        let want = [
            (10, false),
            (12, false),
            (13, true),
            (13, false),
            (16, true),
        ];
        for (frame, done) in want {
            world.frame = frame;
            assert_eq!(
                engine.evaluate_condition(&mut world, &mut c),
                done,
                "{kind} {frame}"
            );
        }
    }
}

#[test]
fn names_of_players_and_teams() {
    let mut world = small_world();
    let mut engine = ScriptEngine::new();
    assert_eq!(engine.player_from_name(&world, "<This Player>"), None);
    assert_eq!(engine.skirmish_enemy(&world), None);
    engine.current_player = Some(2);
    // Without a current enemy, the first human.
    assert_eq!(
        engine.player_from_name(&world, "<This Player's Enemy>"),
        Some(1)
    );
    world.enemy = Some(0);
    assert_eq!(
        engine.player_from_name(&world, "<This Player's Enemy>"),
        Some(0)
    );
    // The enemy is not cached in the parameter: it can change.
    let mut c = condition(
        ct::SKIRMISH_PLAYER_FACTION,
        vec![text("<This Player's Enemy>"), text("Civilian")],
    );
    assert!(engine.evaluate_condition(&mut world, &mut c));
    assert_eq!(c.parms[0].int_val, 0);
    world.enemy = Some(1);
    assert!(!engine.evaluate_condition(&mut world, &mut c));

    assert_eq!(engine.team_named(&world, "T"), Some(7));
    world.teams[0].active = false;
    assert_eq!(engine.team_named(&world, "T"), None);
    world.protos[0].1.singleton = false;
    assert_eq!(engine.team_named(&world, "T"), Some(7));
}

#[test]
fn sequential_waits() {
    for (kind, name, call) in [
        (
            at::SKIRMISH_WAIT_FOR_COMMANDBUTTON_AVAILABLE_ALL,
            "button",
            "button t b true",
        ),
        (
            at::SKIRMISH_WAIT_FOR_COMMANDBUTTON_AVAILABLE_PARTIAL,
            "button",
            "button t b false",
        ),
        (
            at::TEAM_WAIT_FOR_NOT_CONTAINED_ALL,
            "contained",
            "contained p true",
        ),
        (
            at::TEAM_WAIT_FOR_NOT_CONTAINED_PARTIAL,
            "contained",
            "contained p false",
        ),
    ] {
        let mut world = small_world();
        let parms = if name == "button" {
            vec![text("p"), text("t"), text("b")]
        } else {
            vec![text("p")]
        };
        let count = action(at::INCREMENT_COUNTER, vec![number(1), text("n")]);
        let mut engine = engine_with(&mut world, Vec::new(), vec![action(kind, parms), count]);
        engine.append_sequential_script(SequentialScript {
            team: Some(7),
            script: engine.find_script("Main"),
            ..SequentialScript::default()
        });
        world.waits = true;
        engine.update(&mut world);
        engine.update(&mut world);
        assert_eq!(world.calls, [call, call], "{kind}");
        assert_eq!(engine.sequential[0][0].current_instruction, 0);
        assert_eq!(engine.counter("n"), None);
        world.waits = false;
        engine.update(&mut world);
        assert_eq!(engine.counter("n").map(|c| c.value), Some(1), "{kind}");
        assert!(engine.sequential.is_empty());
    }
}

#[test]
fn sequential_chains() {
    let world = small_world();
    let mut engine = ScriptEngine::new();
    let entry = |team: Option<ScriptTeamId>, id: i32, loops: Int| SequentialScript {
        team,
        object_id: ObjectId(id),
        times_to_loop: loops,
        current_instruction: 5,
        ..SequentialScript::default()
    };
    engine.append_sequential_script(entry(Some(7), 0, 0));
    engine.append_sequential_script(entry(None, 1, 1));
    engine.append_sequential_script(entry(Some(7), 0, 2));
    engine.append_sequential_script(entry(None, 1, 3));
    engine.append_sequential_script(entry(None, 2, 4));
    let loops = |e: &ScriptEngine| -> Vec<Vec<Int>> {
        e.sequential
            .iter()
            .map(|c| c.iter().map(|s| s.times_to_loop).collect())
            .collect()
    };
    assert_eq!(loops(&engine), [vec![0, 2], vec![1, 3], vec![4]]);
    assert!(engine
        .sequential
        .iter()
        .flatten()
        .all(|s| s.current_instruction == -1));
    engine.set_sequential_timer_object(ObjectId(1), 9);
    engine.set_sequential_timer_team(7, 8);
    let waits: Vec<Int> = engine
        .sequential
        .iter()
        .flatten()
        .map(|s| s.frames_to_wait)
        .collect();
    assert_eq!(waits, [8, -1, 9, -1, -1]);
    engine.remove_sequential_scripts_of_object(ObjectId(1));
    assert_eq!(loops(&engine), [vec![0, 2], vec![4]]);
    engine.condition_team = Some(7);
    engine.calling_team = Some(7);
    engine.notify_of_team_destruction(7);
    assert_eq!(loops(&engine), [vec![4]]);
    assert_eq!((engine.condition_team, engine.calling_team), (None, None));
    engine.condition_object = Some(ObjectId(2));
    engine.calling_object = Some(ObjectId(2));
    engine.add_object_to_cache("x", ObjectId(2));
    engine.notify_of_object_destruction("x", ObjectId(2));
    assert_eq!(
        (engine.condition_object, engine.calling_object),
        (None, None)
    );
    assert!(engine.did_unit_exist("x"));
    let _ = world;
}

#[test]
fn load_restores_the_lists_and_refuses_a_used_engine() {
    let mut world = small_world();
    world.alpha = 5;
    world.beta = 6;
    world.crate_kind = 3;
    let mut engine = engine_with(&mut world, Vec::new(), Vec::new());
    engine.append_sequential_script(SequentialScript {
        team: Some(7),
        script: engine.find_script("Main"),
        times_to_loop: 2,
        ..SequentialScript::default()
    });
    engine.execute_action(
        &mut world,
        &mut action(
            at::SET_ATTACK_PRIORITY_KIND_OF,
            vec![text("S"), number(3), number(9)],
        ),
    );
    engine.execute_action(
        &mut world,
        &mut action(
            at::SET_ATTACK_PRIORITY_THING,
            vec![text("S"), text("ScrTank"), number(4)],
        ),
    );
    engine.notify_of_acquired_science(15, 6);
    engine.notify_of_completed_video("v");
    engine.signal_ui_interact("h");
    engine.create_named_map_reveal(NamedReveal {
        reveal_name: "r".into(),
        waypoint_name: "w".into(),
        radius: 1.5,
        player_name: "p".into(),
    });
    engine.maintain_object_type_list("L", "ScrTank", true);
    engine.set_topple_direction(
        "A",
        Coord3D {
            x: 0.0,
            y: 3.0,
            z: 4.0,
        },
    );
    engine.current_track_name = "track".into();
    engine.letter_box_active = true;
    let saved = save_bytes(&mut engine, &world);
    let mut loaded = ScriptEngine::new();
    loaded.sides = engine.sides.clone();
    loaded
        .xfer(&mut XferLoad::new(&saved), &world)
        .expect("load");
    assert_eq!(loaded.sequential, engine.sequential);
    assert_eq!(loaded.attack_info("S").priority(Some(0)), 4);
    assert_eq!(loaded.attack_info("S").priority(Some(3)), 9);
    assert_eq!(loaded.acquired_sciences[15], [6]);
    assert_eq!(loaded.completed_video, ["v"]);
    assert_eq!(loaded.ui_interactions, ["h"]);
    assert_eq!(loaded.named_reveals, engine.named_reveals);
    assert_eq!(loaded.current_track_name, "track");
    assert_eq!(
        loaded.letter_box_active,
        !cfg!(feature = "retail-xfer-save")
    );
    assert_eq!(loaded.object_types("L").map(|t| t.get_list_size()), Some(1));
    assert_eq!(save_bytes(&mut loaded, &world), saved);
    // A second load finds lists that should be empty.
    assert!(loaded.xfer(&mut XferLoad::new(&saved), &world).is_err());

    let mut topple = Coord3D {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };
    loaded.adjust_topple_direction("A", &mut topple);
    assert_eq!((topple.y, topple.z), (0.6, 0.8));
}

#[test]
fn full_tables_hand_out_the_unused_slot() {
    let mut engine = ScriptEngine::new();
    for i in 1..MAX_COUNTERS {
        assert_eq!(engine.allocate_counter(&format!("c{i}")), i as Int);
        assert_eq!(engine.allocate_flag(&format!("f{i}")), i as Int);
    }
    assert_eq!(engine.allocate_counter("over"), 0);
    assert_eq!(engine.allocate_flag("over"), 0);
    assert_eq!(engine.allocate_counter("c255"), 255);
    assert_eq!(engine.allocate_flag("f255"), 255);
}

#[test]
fn a_fresh_engine_saves_as_the_reference_does() {
    let world = World::new(0);
    let lines = kind_lines("fresh");
    assert_eq!(lines.len(), 1);
    let mut engine = ScriptEngine::new();
    assert!(engine.first_update);
    assert_eq!(engine.difficulty, GameDifficulty::Normal);
    let mut want = get_str(lines[0], "save").unwrap().to_string();
    if !cfg!(feature = "retail-xfer-save") {
        want = format!("06{}00", &want[2..]);
    }
    assert_eq!(hex(&save_bytes(&mut engine, &world)), want);
}

#[test]
fn built_by_player_counts_the_dead_and_lost_does_not() {
    let mut world = small_world();
    world.players[1].active = true;
    world.players[1].counts[0] = (1, 0);
    let mut engine = ScriptEngine::new();
    let mut built = condition(ct::BUILT_BY_PLAYER, vec![text("ScrTank"), text("PlyrA")]);
    assert!(engine.evaluate_condition(&mut world, &mut built));
    engine.set_object_count(1, "ScrTank", 1);
    let mut lost = condition(
        ct::PLAYER_LOST_OBJECT_TYPE,
        vec![text("PlyrA"), text("ScrTank")],
    );
    assert!(engine.evaluate_condition(&mut world, &mut lost));
}

/// `Main` runs every frame; `Sub` is a subroutine; both on the first side.
fn engine_with_sub(
    world: &mut World,
    main: Vec<ScriptAction>,
    sub: Vec<ScriptAction>,
) -> ScriptEngine {
    let script = |name: &str, subroutine: u8, actions: Vec<ScriptAction>| {
        let mut s = Script::new();
        s.script_name = name.to_string();
        s.is_one_shot = 0;
        s.is_subroutine = subroutine;
        s.or_conditions.push(OrCondition {
            conditions: always(),
        });
        s.actions = actions;
        s
    };
    let mut list = ScriptList::new();
    list.scripts.push(script("Main", 0, main));
    list.scripts.push(script("Sub", 1, sub));
    let mut engine = ScriptEngine::new();
    engine.new_map(world, vec![Some(list)]);
    engine
}

#[test]
fn the_calling_object_and_condition_team_of_a_subroutine() {
    let mut world = small_world();
    let kill = action(at::NAMED_KILL, vec![text("<This Object>")]);
    let state = action(at::TEAM_SET_STATE, vec![text("<This Team>"), text("x")]);
    let mut engine = engine_with_sub(&mut world, Vec::new(), vec![kill, state]);
    engine.run_object_script(&mut world, "Sub", Some(ObjectId(2)));
    assert_eq!(world.calls, ["kill 2"]);
    assert_eq!(engine.calling_object, None);
    // A script naming no team clears the condition team for its own run.
    world.calls.clear();
    engine.condition_team = Some(7);
    let r = engine.find_script("Sub").unwrap();
    engine.execute_script(&mut world, r);
    assert_eq!(world.team(7).state, "");
    assert_eq!(engine.condition_team, Some(7));
}

#[test]
fn a_team_can_stop_its_sequence_and_the_next_one_follows() {
    let mut world = small_world();
    let count = |name: &str| action(at::INCREMENT_COUNTER, vec![number(1), text(name)]);
    let mut engine = engine_with_sub(&mut world, vec![count("a")], vec![count("b")]);
    engine.append_sequential_script(SequentialScript {
        team: Some(7),
        script: engine.find_script("Main"),
        ..SequentialScript::default()
    });
    engine.execute_action(
        &mut world,
        &mut action(at::TEAM_STOP_SEQUENTIAL_SCRIPT, vec![text("T")]),
    );
    assert!(engine.sequential.is_empty());
    for name in ["Main", "Sub"] {
        engine.append_sequential_script(SequentialScript {
            team: Some(7),
            script: engine.find_script(name),
            ..SequentialScript::default()
        });
    }
    // The finished head leaves and the queued script runs. `Main` also runs as
    // an ordinary script each frame.
    engine.update(&mut world);
    assert_eq!(engine.counter("b").map(|c| c.value), Some(1));
    assert!(engine.sequential.is_empty());
}
