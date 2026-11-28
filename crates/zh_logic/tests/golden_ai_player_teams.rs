//! Differential golden verification for `AIPlayer` / `AISkirmishPlayer` team building,
//! recruiting, `WorkOrder` / `TeamInQueue` / `AIPlayer` `xfer` & `crc`, `onUnitProduced`,
//! `onStructureProduced`, `queueDozer`, `queueSupplyTruck`, and `BUILD_TEAM` / `RECRUIT_TEAM`.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use zh_common::core::dict::Dict;
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::{
    Bool, Int, ObjectId, Real, TeamId, UnsignedInt, UnsignedShort, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferCrc, XferError, XferLoad, XferSave};

use zh_logic::ai::ai_player::{
    on_structure_produced, update as ai_player_update, AiPlayer, DIFFICULTY_EASY, DIFFICULTY_HARD,
    DIFFICULTY_NORMAL,
};
use zh_logic::ai::ai_state_machine::state_id;
use zh_logic::ai::ai_team_queue::{
    ai_pre_team_destroy, build_specific_ai_team, check_queued_teams, check_ready_teams,
    do_team_building, drain_pending_sim_events_for_player, evaluate_production_condition,
    is_possible_to_build_team, queue_dozer, queue_supply_truck, queue_units, select_team_to_build,
    select_team_to_reinforce, TeamInQueue, WorkOrder,
};
use zh_logic::arena::ObjectSlot;
use zh_logic::economy::money::NoopMoneyCallbacks;
use zh_logic::gamelogic::game_engine::{GameDataLoader, StartupStores};
use zh_logic::gamelogic::global_data::PlayerType;
use zh_logic::modules::module::CommandSourceType;
use zh_logic::modules::update::ai_update::{AiUpdate, DerivedAi};
use zh_logic::player::build_assistant::BuildContext;
use zh_logic::player::player_list::PlayerList;
use zh_logic::player::seams::{
    AiKind, PlayerEnv, PlayerIndex, PlayerPart, PlayerWorld, SkirmishScripts,
};
use zh_logic::player::team::{
    count_objects_by_thing_template, sync_team_to_sim_world, transfer_units_to, try_to_recruit,
    CreateUnitsInfo, TeamFactory,
};
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::rules::science::ScienceType;
use zh_logic::rules::upgrade::UpgradeMaskType;
use zh_logic::scripts::script_types::{action_type as at, condition_type as ct};
use zh_logic::scripts::{
    script_template_registry, Condition, GameScripts, OrCondition, Parameter, ParameterType,
    Script, ScriptAction, ScriptGame, ScriptList, ScriptTemplateRegistry,
};
use zh_logic::sim::{SimApi, SimContext, SimWorld};
use zh_logic::spatial::sides_list::{BuildListInfo, SidesList};

const GOLDEN: &str = include_str!("../../../docs/golden_vectors/AiPlayerTeams.json");

fn rows() -> impl Iterator<Item = &'static str> {
    GOLDEN.lines().filter_map(|l| {
        l.trim()
            .trim_end_matches(',')
            .strip_prefix('"')?
            .strip_suffix('"')
    })
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

fn unhex(s: &str) -> Vec<u8> {
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).expect("hex"))
        .collect()
}

fn obj_id(s: &str) -> ObjectId {
    let raw: u32 = s.parse().unwrap();
    ObjectId(raw as Int)
}

fn fixture_ini() -> &'static str {
    concat!(
        "Locomotor AptGround\n",
        "  Surfaces = GROUND\n",
        "  Appearance = FOUR_WHEELS\n",
        "  Speed = 30\n",
        "  TurnRate = 180\n",
        "  Acceleration = 60\n",
        "  Braking = 60\n",
        "  MinSpeed = 0\n",
        "End\n",
        "PlayerTemplate FactionCivilian\n  Side = Civilian\n  PreferredColor = R:100 G:100 B:100\nEnd\n",
        "PlayerTemplate FactionAmerica\n  Side = America\n  PreferredColor = R:10 G:20 B:255\nEnd\n",
        "Object UnitA\n",
        "  Side = America\n  KindOf = PRELOAD SELECTABLE VEHICLE\n",
        "  BuildCost = 1000\n  BuildTime = 1.0\n",
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 6\n  GeometryHeight = 8\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n",
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n",
        "  Locomotor = SET_NORMAL AptGround\n",
        "End\n",
        "Object UnitBVar\n",
        "  Side = America\n  KindOf = PRELOAD SELECTABLE INFANTRY\n",
        "  BuildCost = 333\n  BuildTime = 0.5\n",
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 4\n  GeometryHeight = 6\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n",
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n",
        "  Locomotor = SET_NORMAL AptGround\n",
        "End\n",
        "Object UnitB\n",
        "  Side = America\n  KindOf = PRELOAD SELECTABLE INFANTRY\n",
        "  BuildCost = 333\n  BuildTime = 0.5\n",
        "  BuildVariations = UnitBVar\n",
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 4\n  GeometryHeight = 6\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n",
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n",
        "  Locomotor = SET_NORMAL AptGround\n",
        "End\n",
        "Object UnitC\n",
        "  Side = America\n  KindOf = PRELOAD SELECTABLE VEHICLE\n",
        "  BuildCost = 777\n  BuildTime = 1.0\n",
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 6\n  GeometryHeight = 8\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 150\n    InitialHealth = 150\n  End\n",
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n",
        "  Locomotor = SET_NORMAL AptGround\n",
        "End\n",
        "Object UnitNoFact\n",
        "  Side = America\n  KindOf = PRELOAD SELECTABLE VEHICLE\n",
        "  BuildCost = 500\n  BuildTime = 1.0\n",
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 6\n  GeometryHeight = 8\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 150\n    InitialHealth = 150\n  End\n",
        "End\n",
        "Object DozerA\n",
        "  Side = America\n  KindOf = PRELOAD SELECTABLE VEHICLE DOZER\n",
        "  BuildCost = 1000\n  BuildTime = 1.0\n",
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 7\n  GeometryHeight = 8\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n",
        "  Behavior = DozerAIUpdate ModuleTag_AI\n    RepairHealthPercentPerSecond = 2%\n    BoredTime = 5000\n    BoredRange = 150\n  End\n",
        "  Locomotor = SET_NORMAL AptGround\n",
        "End\n",
        "Object DozerB\n",
        "  Side = America\n  KindOf = PRELOAD SELECTABLE VEHICLE DOZER\n",
        "  BuildCost = 1200\n  BuildTime = 1.0\n",
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 7\n  GeometryHeight = 8\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n",
        "  Behavior = DozerAIUpdate ModuleTag_AI\n    RepairHealthPercentPerSecond = 2%\n    BoredTime = 5000\n    BoredRange = 150\n  End\n",
        "  Locomotor = SET_NORMAL AptGround\n",
        "End\n",
        "Object WorkerGatherer\n",
        "  Side = America\n  KindOf = PRELOAD SELECTABLE VEHICLE HARVESTER\n",
        "  BuildCost = 600\n  BuildTime = 1.0\n",
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 6\n  GeometryHeight = 8\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 150\n    InitialHealth = 150\n  End\n",
        "  Behavior = SupplyTruckAIUpdate ModuleTag_AI\n    MaxBoxes = 4\n  End\n",
        "  Locomotor = SET_NORMAL AptGround\n",
        "End\n",
        "Object FactA\n",
        "  Side = America\n  KindOf = PRELOAD STRUCTURE FS_FACTORY FS_WARFACTORY\n",
        "  CommandSet = CmdFactA\n",
        "  Geometry = BOX\n  GeometryMajorRadius = 20\n  GeometryMinorRadius = 20\n  GeometryHeight = 20\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 1000\n  End\n",
        "  Behavior = ProductionUpdate ModuleTag_Prod\n  End\n",
        "  Behavior = DefaultProductionExitUpdate ModuleTag_Exit\n",
        "    UnitCreatePoint = X:0.0 Y:0.0 Z:0.0\n",
        "    NaturalRallyPoint = X:40.0 Y:0.0 Z:0.0\n",
        "  End\n",
        "End\n",
        "Object FactB\n",
        "  Side = America\n  KindOf = PRELOAD STRUCTURE FS_FACTORY FS_BARRACKS\n",
        "  CommandSet = CmdFactB\n",
        "  Geometry = BOX\n  GeometryMajorRadius = 20\n  GeometryMinorRadius = 20\n  GeometryHeight = 20\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 1000\n  End\n",
        "  Behavior = ProductionUpdate ModuleTag_Prod\n  End\n",
        "  Behavior = DefaultProductionExitUpdate ModuleTag_Exit\n",
        "    UnitCreatePoint = X:0.0 Y:0.0 Z:0.0\n",
        "    NaturalRallyPoint = X:40.0 Y:0.0 Z:0.0\n",
        "  End\n",
        "End\n",
        "Object SupplyCtr\n",
        "  Side = America\n  KindOf = PRELOAD STRUCTURE FS_SUPPLY_CENTER\n",
        "  Geometry = BOX\n  GeometryMajorRadius = 25\n  GeometryMinorRadius = 20\n  GeometryHeight = 15\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 1000\n  End\n",
        "  Behavior = SupplyCenterDockUpdate ModuleTag_Dock\n    NumberApproachPositions = 2\n  End\n",
        "End\n",
        "Object Warehouse\n",
        "  Side = America\n  KindOf = PRELOAD STRUCTURE SUPPLY_SOURCE\n",
        "  Geometry = BOX\n  GeometryMajorRadius = 20\n  GeometryMinorRadius = 20\n  GeometryHeight = 15\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 1000\n  End\n",
        "  Behavior = SupplyWarehouseDockUpdate ModuleTag_Dock\n    NumberApproachPositions = 2\n    StartingBoxes = 20\n  End\n",
        "End\n",
    )
}

fn commands_ini() -> &'static str {
    concat!(
        "CommandButton Btn_UnitA\n  Command = UNIT_BUILD\n  Object = UnitA\nEnd\n",
        "CommandButton Btn_UnitB\n  Command = UNIT_BUILD\n  Object = UnitB\nEnd\n",
        "CommandButton Btn_UnitC\n  Command = UNIT_BUILD\n  Object = UnitC\nEnd\n",
        "CommandButton Btn_DozerA\n  Command = UNIT_BUILD\n  Object = DozerA\nEnd\n",
        "CommandButton Btn_DozerB\n  Command = UNIT_BUILD\n  Object = DozerB\nEnd\n",
        "CommandButton Btn_WorkerGatherer\n  Command = UNIT_BUILD\n  Object = WorkerGatherer\nEnd\n",
        "CommandSet CmdFactA\n",
        "  1 = Btn_UnitA\n",
        "  2 = Btn_UnitC\n",
        "  3 = Btn_DozerA\n",
        "  4 = Btn_DozerB\n",
        "  5 = Btn_WorkerGatherer\n",
        "End\n",
        "CommandSet CmdFactB\n",
        "  1 = Btn_UnitB\n",
        "End\n",
    )
}

struct TestPlayerWorld<'a> {
    world: &'a mut SimWorld,
    team_seconds: Int,
    global_difficulty: Int,
}

impl PlayerWorld for TestPlayerWorld<'_> {
    fn frame(&self) -> UnsignedInt {
        self.world.frame
    }
    fn starting_cash(&self) -> UnsignedInt {
        0
    }
    fn is_network(&self) -> bool {
        false
    }
    fn game_slot_occupied(&self, _slot: usize) -> bool {
        false
    }
    fn global_difficulty(&self) -> Int {
        self.global_difficulty
    }
    fn force_skirmish_ai(&self) -> bool {
        false
    }
    fn general_name(&self) -> Vec<u16> {
        Vec::new()
    }
    fn waypoint_location(&self, _name: &str) -> Option<Coord3D> {
        None
    }
    fn waypoint_exists(&self, _id: UnsignedInt) -> bool {
        false
    }
    fn becoming_local_player(&mut self, _player: PlayerIndex, _yes: bool) {}
    fn local_player_update(&mut self, _player: PlayerIndex) {}
    fn create_ai(&mut self, player: PlayerIndex, kind: AiKind) {
        let frame = self.world.frame as Int;
        let ai = AiPlayer::new(
            kind,
            player,
            frame,
            self.global_difficulty,
            self.team_seconds,
        );
        self.world.ai_players.insert(player, ai);
    }
    fn delete_ai(&mut self, player: PlayerIndex) {
        self.world.ai_players.remove(&player);
    }
    fn set_ai_difficulty(&mut self, player: PlayerIndex, difficulty: Int) {
        if let Some(ai) = self.world.ai_players.get_mut(&player) {
            ai.difficulty = difficulty;
        }
    }
    fn ai_update(&mut self, _player: PlayerIndex) {}
    fn ai_new_map(&mut self, _player: PlayerIndex) {}
    fn ai_pre_team_destroy(&mut self, player: PlayerIndex, team: TeamId) {
        ai_pre_team_destroy(&mut self.world.ai_players, player, team);
    }
    fn qualify_skirmish_scripts(&mut self, _sides: &mut SidesList, _copy: &SkirmishScripts<'_>) {}
    fn create_economy_trackers(&mut self, _player: PlayerIndex) {}
    fn delete_economy_trackers(&mut self, _player: PlayerIndex) {}
    fn academy_init(&mut self, _player: PlayerIndex) {}
    fn academy_update(&mut self, _player: PlayerIndex) {}
    fn academy_record_income(&mut self, _player: PlayerIndex) {}
    fn power_brownout_changed(&mut self, _player: PlayerIndex, _brownout: bool) {}
    fn science_added(&mut self, _player: PlayerIndex, _science: ScienceType) {}
    fn upgrade_count(&self, _player: PlayerIndex) -> UnsignedShort {
        0
    }
    fn delete_upgrades(&mut self, _player: PlayerIndex) {}
    fn upgrade_masks(&self) -> Vec<(String, UpgradeMaskType)> {
        Vec::new()
    }
    fn notify_team_created(&mut self, players: &PlayerList, teams: &TeamFactory, team: TeamId) {
        let Some(owner) = teams.controlling_player(team) else {
            return;
        };
        let Some(p) = players.player(owner) else {
            return;
        };
        self.world.team_players.insert(team, owner);
        self.world
            .team_is_neutral
            .insert(team, Bool::from(owner == 0));
        self.world
            .player_is_human
            .insert(team, Bool::from(p.player_type == PlayerType::Human));
        if p.ai == Some(AiKind::Skirmish) {
            self.world.skirmish_ai_teams.insert(team);
        }
        if teams
            .prototype_of(team)
            .is_some_and(|pr| pr.template.attack_common_target)
        {
            self.world
                .team_common_targets
                .entry(team)
                .or_insert(ObjectId::INVALID);
        }
    }
    fn notify_team_destroyed(&mut self, _team: TeamId) {}
    fn detach_member(&mut self, _object: ObjectId) {}
    fn member_changed_player(&mut self, _object: ObjectId, _old: PlayerIndex, _new: PlayerIndex) {}
    fn execute_team_create_actions(
        &mut self,
        _players: &mut PlayerList,
        _teams: &mut TeamFactory,
        _condition: &str,
    ) {
    }
    fn duplicate_script(&mut self, _name: &str) -> Option<UnsignedInt> {
        None
    }
    fn evaluate_script(
        &mut self,
        _players: &mut PlayerList,
        _teams: &mut TeamFactory,
        _script: UnsignedInt,
        _team: TeamId,
    ) -> bool {
        false
    }
    fn script_is_one_shot(&self, _script: UnsignedInt) -> bool {
        false
    }
    fn execute_script_action(
        &mut self,
        _players: &mut PlayerList,
        _teams: &mut TeamFactory,
        _script: UnsignedInt,
        _team: TeamId,
    ) {
    }
    fn run_script(
        &mut self,
        _players: &mut PlayerList,
        _teams: &mut TeamFactory,
        _name: &str,
        _team: TeamId,
    ) {
    }
    fn member_is_effectively_dead(&self, _object: ObjectId) -> bool {
        false
    }
    fn member_ai_idle(&self, _object: ObjectId) -> Option<bool> {
        Some(false)
    }
    fn member_sees_enemy(&mut self, _object: ObjectId) -> bool {
        false
    }
    fn xfer_player_part(
        &mut self,
        _player: PlayerIndex,
        _part: PlayerPart,
        _xfer: &mut dyn Xfer,
    ) -> Result<(), XferError> {
        Ok(())
    }
}

struct Fixture {
    sim: SimContext,
    stores: StartupStores,
    teams: TeamFactory,
    scripts: GameScripts,
    waypoints: Vec<(String, Coord3D)>,
    named: Vec<(String, ObjectId)>,
    unported: BTreeMap<&'static str, u32>,
}

fn make_cond_script(
    reg: &ScriptTemplateRegistry,
    name: &str,
    cond_type: Int,
    easy: bool,
    normal: bool,
    hard: bool,
    delay_sec: Int,
) -> Script {
    let mut s = Script::new();
    s.script_name = name.to_owned();
    s.is_active = TRUE;
    s.easy = Bool::from(easy);
    s.normal = Bool::from(normal);
    s.hard = Bool::from(hard);
    s.delay_evaluation_seconds = delay_sec;
    let mut oc = OrCondition::new();
    oc.conditions.push(Condition::new(cond_type, reg));
    s.or_conditions.push(oc);
    s
}

fn add_team_proto(
    sides: &mut SidesList,
    sim: &mut SimContext,
    name: &str,
    owner: &str,
    singleton: bool,
    prio: Int,
    max_inst: Int,
    prod_cond: &str,
    units: &[(Int, Int, &str)],
    transport: &str,
    starts_full: bool,
) {
    let mut d = Dict::default();
    let mut k = |s: &str| sim.rules.name_keys.name_to_key(s);
    d.set_ascii_string(k("teamName"), name);
    d.set_ascii_string(k("teamOwner"), owner);
    d.set_bool(k("teamIsSingleton"), Bool::from(singleton));
    d.set_int(k("teamProductionPriority"), prio);
    d.set_int(k("teamMaxInstances"), max_inst);
    d.set_bool(k("teamStartsFull"), Bool::from(starts_full));
    if !prod_cond.is_empty() {
        d.set_ascii_string(k("teamProductionCondition"), prod_cond);
    }
    if !transport.is_empty() {
        d.set_ascii_string(k("teamTransport"), transport);
    }
    let u0_min = units.first().map_or(0, |u| u.0);
    let u0_max = units.first().map_or(0, |u| u.1);
    d.set_int(k("teamUnitMinCount1"), u0_min);
    d.set_int(k("teamUnitMaxCount1"), u0_max);
    if let Some((_, _, u0)) = units.first() {
        d.set_ascii_string(k("teamUnitType1"), u0);
    }
    for (idx, &(mn, mx, nm)) in units.iter().enumerate().skip(1) {
        let one = idx + 1;
        d.set_int(k(&format!("teamUnitMinCount{one}")), mn);
        d.set_int(k(&format!("teamUnitMaxCount{one}")), mx);
        d.set_ascii_string(k(&format!("teamUnitType{one}")), nm);
    }
    sides.add_team(Some(&d));
}

fn sync_all(sim: &mut SimContext, teams: &TeamFactory) {
    let players = &sim.world.players;
    if let Some(team) = players.player(0).and_then(|p| p.default_team) {
        sim.world.neutral_team_id = team;
    }
    sim.world.skirmish_ai_teams.clear();
    for index in 0..players.player_count() {
        let Some(player) = players.player(index) else {
            continue;
        };
        let mut lists = Vec::new();
        for &proto in &player.team_prototypes {
            let instances = teams.instance_ids(proto);
            for &team in &instances {
                sim.world.team_players.insert(team, index);
                sim.world
                    .team_is_neutral
                    .insert(team, Bool::from(index == 0));
                sim.world
                    .player_is_human
                    .insert(team, Bool::from(player.player_type == PlayerType::Human));
                if player.ai == Some(AiKind::Skirmish) {
                    sim.world.skirmish_ai_teams.insert(team);
                }
            }
            if !instances.is_empty() {
                lists.push(instances);
            }
        }
        sim.world.player_team_prototypes.insert(index, lists);
        if let Some(team) = player.default_team {
            sim.world.player_default_teams.insert(index, team);
        }
    }
}

fn setup_world(skirmish: bool) -> Fixture {
    setup_world_at(skirmish, 1)
}

fn setup_world_at(skirmish: bool, init_frame: UnsignedInt) -> Fixture {
    let mut loader = GameDataLoader::new();
    loader.load_bytes(
        "fixture.ini",
        fixture_ini().as_bytes(),
        IniLoadType::Overwrite,
    );
    loader.load_bytes(
        "commands.ini",
        commands_ini().as_bytes(),
        IniLoadType::Overwrite,
    );
    loader.rules.thing_factory.post_process_load();
    loader.rules.global_data.gravity = -1.0;

    let mut sim = SimContext {
        rules: loader.rules,
        ..SimContext::new()
    };
    sim.world.frame = init_frame;
    let stores = loader.stores;

    let mut sides = SidesList::default();
    let mut k = |s: &str| sim.rules.name_keys.name_to_key(s);
    // Neutral
    {
        let mut d = Dict::default();
        d.set_ascii_string(k("playerName"), "");
        d.set_ascii_string(k("playerFaction"), "FactionCivilian");
        d.set_bool(k("playerIsHuman"), FALSE);
        sides.add_side(Some(&d));
        let mut td = Dict::default();
        td.set_ascii_string(k("teamName"), "team");
        td.set_ascii_string(k("teamOwner"), "");
        td.set_bool(k("teamIsSingleton"), TRUE);
        sides.add_team(Some(&td));
    }
    // Player 1: AiPlayer1
    {
        let mut d = Dict::default();
        d.set_ascii_string(k("playerName"), "AiPlayer1");
        d.set_ascii_string(k("playerFaction"), "FactionAmerica");
        d.set_bool(k("playerIsHuman"), FALSE);
        d.set_int(k("skirmishDifficulty"), DIFFICULTY_NORMAL);
        sides.add_side(Some(&d));
        let mut td = Dict::default();
        td.set_ascii_string(k("teamName"), "teamAiPlayer1");
        td.set_ascii_string(k("teamOwner"), "AiPlayer1");
        td.set_bool(k("teamIsSingleton"), TRUE);
        sides.add_team(Some(&td));
    }
    // Player 2: EnemyPlayer2
    {
        let mut d = Dict::default();
        d.set_ascii_string(k("playerName"), "EnemyPlayer2");
        d.set_ascii_string(k("playerFaction"), "FactionAmerica");
        d.set_bool(k("playerIsHuman"), TRUE);
        sides.add_side(Some(&d));
        let mut td = Dict::default();
        td.set_ascii_string(k("teamName"), "teamEnemyPlayer2");
        td.set_ascii_string(k("teamOwner"), "EnemyPlayer2");
        td.set_bool(k("teamIsSingleton"), TRUE);
        sides.add_team(Some(&td));
    }

    add_team_proto(
        &mut sides,
        &mut sim,
        "TeamAlpha",
        "AiPlayer1",
        false,
        10,
        3,
        "ScrTrueAll",
        &[(1, 3, "UnitA")],
        "",
        false,
    );
    add_team_proto(
        &mut sides,
        &mut sim,
        "TeamLow",
        "AiPlayer1",
        true,
        5,
        1,
        "",
        &[],
        "",
        false,
    );
    add_team_proto(
        &mut sides,
        &mut sim,
        "TeamHigh",
        "AiPlayer1",
        true,
        20,
        1,
        "",
        &[],
        "",
        false,
    );
    add_team_proto(
        &mut sides,
        &mut sim,
        "TeamTestPoss",
        "AiPlayer1",
        false,
        10,
        1,
        "",
        &[(1, 1, "UnitA")],
        "",
        false,
    );
    add_team_proto(
        &mut sides,
        &mut sim,
        "BT_Min0Max3",
        "AiPlayer1",
        false,
        10,
        5,
        "",
        &[(0, 3, "UnitA")],
        "",
        false,
    );
    add_team_proto(
        &mut sides,
        &mut sim,
        "BT_Min2Max2",
        "AiPlayer1",
        false,
        10,
        5,
        "",
        &[(2, 2, "UnitA")],
        "",
        false,
    );
    add_team_proto(
        &mut sides,
        &mut sim,
        "BT_Min1Max4",
        "AiPlayer1",
        false,
        10,
        5,
        "",
        &[(1, 4, "UnitA")],
        "",
        false,
    );
    add_team_proto(
        &mut sides,
        &mut sim,
        "BT_Multi",
        "AiPlayer1",
        false,
        10,
        5,
        "",
        &[(1, 3, "UnitA"), (0, 2, "UnitB"), (2, 2, "UnitC")],
        "",
        false,
    );
    add_team_proto(
        &mut sides,
        &mut sim,
        "BT_Transport",
        "AiPlayer1",
        false,
        10,
        5,
        "",
        &[(1, 2, "UnitA")],
        "UnitC",
        true,
    );
    for (name, cond) in [
        ("EC_Empty", ""),
        ("EC_Missing", "NoSuchScript"),
        ("EC_True", "ScrTrueAll"),
        ("EC_False", "ScrFalseAll"),
        ("EC_Easy", "ScrEasyOnly"),
        ("EC_Delay3", "ScrDelay3"),
    ] {
        add_team_proto(
            &mut sides,
            &mut sim,
            name,
            "AiPlayer1",
            false,
            10,
            3,
            cond,
            &[],
            "",
            false,
        );
    }
    for (name, prio) in [
        ("RecruitTarget", 10),
        ("RecSrcOk", 5),
        ("RecSrcHighPrio", 20),
        ("RecSrcNoFlag", 5),
    ] {
        add_team_proto(
            &mut sides,
            &mut sim,
            name,
            "AiPlayer1",
            false,
            prio,
            3,
            "",
            &[],
            "",
            false,
        );
    }

    let mut teams = TeamFactory::new();
    let team_seconds = sim.rules.ai_data.team_seconds as Int;
    let reg = script_template_registry(&mut sim.rules.name_keys);

    let players = {
        let mut world = TestPlayerWorld {
            world: &mut sim.world,
            team_seconds,
            global_difficulty: DIFFICULTY_NORMAL,
        };
        let mut env = PlayerEnv {
            templates: &stores.player_templates,
            ranks: &stores.ranks,
            sciences: &sim.rules.science_store,
            name_keys: &mut sim.rules.name_keys,
            world: &mut world,
        };
        let mut players = PlayerList::new(&mut env);
        players.new_game(&mut sides, &mut teams, &mut env);
        if skirmish {
            if let Some(p1) = players.player_mut(1) {
                p1.set_player_type(PlayerType::Computer, true, env.world);
            }
        }
        players
    };
    sim.world.players = players;
    // Enemy relationship setup matching PlayerList::new_game default (enemy with P2)
    sim.world.players
        .player_mut(1)
        .unwrap()
        .set_player_relationship(2, zh_common::types::Relationship::Enemies);
    sim.world.players
        .player_mut(2)
        .unwrap()
        .set_player_relationship(1, zh_common::types::Relationship::Enemies);
    sim.world
        .player_relationships
        .insert((1, 2), zh_common::types::Relationship::Enemies);
    sim.world
        .player_relationships
        .insert((2, 1), zh_common::types::Relationship::Enemies);
    sync_all(&mut sim, &teams);

    let mut sl_ai = ScriptList::new();
    sl_ai.add_script(
        make_cond_script(&reg, "ScrTrueAll", ct::CONDITION_TRUE, true, true, true, 0),
        0,
    );
    sl_ai.add_script(
        make_cond_script(
            &reg,
            "ScrFalseAll",
            ct::CONDITION_FALSE,
            true,
            true,
            true,
            0,
        ),
        1,
    );
    sl_ai.add_script(
        make_cond_script(
            &reg,
            "ScrEasyOnly",
            ct::CONDITION_TRUE,
            true,
            false,
            false,
            0,
        ),
        2,
    );
    sl_ai.add_script(
        make_cond_script(&reg, "ScrDelay3", ct::CONDITION_TRUE, true, true, true, 3),
        3,
    );

    let mut scripts = GameScripts::default();
    let mut named = Vec::new();
    let waypoints = Vec::new();
    scripts.new_map(
        &mut sim,
        ScriptGame {
            teams: &mut teams,
            stores: &stores,
            waypoints: &waypoints,
            named_objects: &mut named,
        },
        vec![None, Some(sl_ai), None],
    );
    sim.world.frame = 1;

    Fixture {
        sim,
        stores,
        teams,
        scripts,
        waypoints,
        named,
        unported: BTreeMap::new(),
    }
}

fn spawn(f: &mut Fixture, template: &str, team: TeamId, pos: Coord3D) -> ObjectSlot {
    let tmpl = f
        .sim
        .rules
        .thing_factory
        .find_template_id(template)
        .expect(template);
    let slot = f.sim.spawn_object(tmpl, team, pos, 0.0);
    let id = f.sim.world.objects.id_of(slot).expect("id");
    if let Some(t) = f.teams.team_mut(team) {
        t.add_member(id);
    }
    slot
}

fn set_money(f: &mut Fixture, player: PlayerIndex, amount: UnsignedInt) {
    let mut cb = NoopMoneyCallbacks;
    if let Some(p) = f.sim.world.players.player_mut(player) {
        let cur = p.money.count_money();
        if cur > 0 {
            p.money.withdraw(cur, FALSE, &mut cb);
        }
        if amount > 0 {
            p.money.deposit(amount, FALSE, FALSE, &mut cb);
        }
    }
    f.sim.set_player_cash(player, amount);
}

fn make_tiq(teams: &TeamFactory, team_id: TeamId) -> TeamInQueue {
    let mut tq = TeamInQueue::new();
    tq.team_id = team_id;
    if let Some(t) = teams.team(team_id) {
        tq.prototype_id = Some(t.prototype);
        if let Some(proto) = teams.prototype(t.prototype) {
            tq.has_home_location = proto.template.has_home_location;
            tq.home_location = proto.template.home_location;
        }
    }
    tq
}

fn create_team(f: &mut Fixture, name: &str, active: bool) -> TeamId {
    let ts = f.sim.rules.ai_data.team_seconds as Int;
    let id = {
        let mut list = std::mem::take(&mut f.sim.world.players);
        let mut world = TestPlayerWorld {
            world: &mut f.sim.world,
            team_seconds: ts,
            global_difficulty: DIFFICULTY_EASY,
        };
        let mut env = PlayerEnv {
            templates: &f.stores.player_templates,
            ranks: &f.stores.ranks,
            sciences: &f.sim.rules.science_store,
            name_keys: &mut f.sim.rules.name_keys,
            world: &mut world,
        };
        let made = if active {
            f.teams.create_team(name, &mut list, &mut env)
        } else {
            f.teams.create_inactive_team(name, &mut list, &mut env)
        };
        f.sim.world.players = list;
        made
    }
    .expect("create_team");
    sync_all(&mut f.sim, &f.teams);
    id
}

fn find_team(f: &mut Fixture, name: &str) -> TeamId {
    let ts = f.sim.rules.ai_data.team_seconds as Int;
    let id = {
        let mut list = std::mem::take(&mut f.sim.world.players);
        let mut world = TestPlayerWorld {
            world: &mut f.sim.world,
            team_seconds: ts,
            global_difficulty: DIFFICULTY_EASY,
        };
        let mut env = PlayerEnv {
            templates: &f.stores.player_templates,
            ranks: &f.stores.ranks,
            sciences: &f.sim.rules.science_store,
            name_keys: &mut f.sim.rules.name_keys,
            world: &mut world,
        };
        let found = f.teams.find_team(name, &mut list, &mut env);
        f.sim.world.players = list;
        found
    }
    .expect("find_team");
    sync_all(&mut f.sim, &f.teams);
    id
}

fn with_cx<R>(f: &mut Fixture, func: impl FnOnce(&mut BuildContext<'_>) -> R) -> R {
    let mut cx = BuildContext {
        sim: &mut f.sim,
        named: &mut f.named,
        unported: &mut f.unported,
        pathfind_extent: Region3D {
            lo: Coord3D::new(0.0, 0.0, 0.0),
            hi: Coord3D::new(1000.0, 1000.0, 100.0),
        },
        teams: Some(&mut f.teams),
        scripts: Some(&mut f.scripts),
        stores: Some(&f.stores),
        waypoints: &f.waypoints,
    };
    func(&mut cx)
}

fn make_wo_by_idx(
    f: &Fixture,
    thing_idx: usize,
    fac_id: ObjectId,
    comp: Int,
    req_cnt: Int,
    req: bool,
    gath: bool,
) -> WorkOrder {
    const THINGS: [Option<&str>; 8] = [
        None,
        Some("UnitA"),
        Some("UnitB"),
        Some("UnitBVar"),
        Some("UnitC"),
        Some("DozerA"),
        Some("DozerB"),
        Some("WorkerGatherer"),
    ];
    let name = THINGS[thing_idx];
    let mut wo = WorkOrder::new();
    wo.thing_name = name.unwrap_or_default().to_owned();
    wo.thing = name.and_then(|n| f.sim.rules.thing_factory.find_template_id(n));
    wo.factory_id = fac_id;
    wo.num_completed = comp;
    wo.num_required = req_cnt;
    wo.required = req;
    wo.is_resource_gatherer = gath;
    wo
}

fn make_wo_by_name(
    f: &Fixture,
    name: &str,
    fac_id: ObjectId,
    comp: Int,
    req_cnt: Int,
    req: bool,
    gath: bool,
) -> WorkOrder {
    let mut wo = WorkOrder::new();
    if name != "-" {
        wo.thing_name = name.to_owned();
        wo.thing = f.sim.rules.thing_factory.find_template_id(name);
    }
    wo.factory_id = fac_id;
    wo.num_completed = comp;
    wo.num_required = req_cnt;
    wo.required = req;
    wo.is_resource_gatherer = gath;
    wo
}

#[test]
fn golden_work_order_and_team_in_queue_and_ai_xfer() {
    let mut f = setup_world(false);
    let p1_def = f.sim.world.players.player(1).unwrap().default_team.unwrap();
    let p2_def = f.sim.world.players.player(2).unwrap().default_team.unwrap();
    let my_fac = spawn(&mut f, "FactA", p1_def, Coord3D::new(100.0, 100.0, 0.0));
    let enemy_fac = spawn(&mut f, "FactA", p2_def, Coord3D::new(300.0, 300.0, 0.0));
    let _my_id = f.sim.world.objects.id_of(my_fac).unwrap();
    let _enemy_id = f.sim.world.objects.id_of(enemy_fac).unwrap();

    let mut wo_count = 0;
    for line in rows().filter(|l| l.starts_with("WOXFER ")) {
        let p: Vec<&str> = line.split_whitespace().collect();
        let idx: usize = p[1].parse().unwrap();
        let thing_name = p[2];
        let fac_id = obj_id(p[3]);
        let comp: Int = p[4].parse().unwrap();
        let req_cnt: Int = p[5].parse().unwrap();
        let is_req = p[6] == "1";
        let gath = p[7] == "1";
        let exp_wait = p[8] == "1";
        let exp_fac_after = obj_id(p[9]);
        let exp_crc = u32::from_str_radix(p[10], 16).unwrap();
        let exp_hex = p[11];

        let mut wo = make_wo_by_name(&f, thing_name, fac_id, comp, req_cnt, is_req, gath);
        assert_eq!(wo.is_waiting_to_build(), exp_wait, "WOXFER {idx} wait");

        let mut xc = XferCrc::new();
        wo.crc(&mut xc).unwrap();
        assert_eq!(xc.get_crc(), exp_crc, "WOXFER {idx} crc");

        let mut xs = XferSave::new();
        wo.xfer(&mut xs, Some(&f.sim.rules.thing_factory)).unwrap();
        let bytes = xs.into_bytes();
        assert_eq!(hex(&bytes), exp_hex, "WOXFER {idx} xfer bytes");

        let mut loaded = WorkOrder::new();
        let mut xl = XferLoad::new(&bytes);
        loaded
            .xfer(&mut xl, Some(&f.sim.rules.thing_factory))
            .unwrap();
        assert_eq!(loaded.thing, wo.thing, "WOXFER {idx} load thing");
        assert_eq!(loaded.factory_id, wo.factory_id, "WOXFER {idx} load fac");
        assert_eq!(loaded.num_completed, wo.num_completed);
        assert_eq!(loaded.num_required, wo.num_required);
        assert_eq!(loaded.required, wo.required);
        assert_eq!(loaded.is_resource_gatherer, wo.is_resource_gatherer);

        wo.validate_factory(&f.sim, 1);
        assert_eq!(
            wo.factory_id, exp_fac_after,
            "WOXFER {idx} validate_factory"
        );
        wo_count += 1;
    }
    assert_eq!(wo_count, 200);

    let _t_alpha1 = create_team(&mut f, "TeamAlpha", true);
    let _t_low = find_team(&mut f, "TeamLow");

    let mut tiq_count = 0;
    for line in rows().filter(|l| l.starts_with("TIQXFER ")) {
        let p: Vec<&str> = line.split_whitespace().collect();
        let idx: usize = p[1].parse().unwrap();
        let orders_spec = p[2];
        let prio = p[3] == "1";
        let team_raw: u32 = p[4].parse().unwrap();
        let team_id = if team_raw == 0 {
            TeamId::INVALID
        } else {
            TeamId(team_raw as Int)
        };
        let frame_started: Int = p[5].parse().unwrap();
        let sent = p[6] == "1";
        let stop = p[7] == "1";
        let reinf = p[8] == "1";
        let reinf_id = obj_id(p[9]);
        let flags = p[10];
        let exp_bits = p[11];
        let exp_crc = u32::from_str_radix(p[12], 16).unwrap();
        let exp_hex = p[13];

        let mut tq = if team_id != TeamId::INVALID {
            make_tiq(&f.teams, team_id)
        } else {
            TeamInQueue::new()
        };
        if orders_spec != "-" {
            for spec in orders_spec.split(';') {
                let sp: Vec<&str> = spec.split(',').collect();
                let wo = make_wo_by_name(
                    &f,
                    sp[0],
                    obj_id(sp[1]),
                    sp[2].parse().unwrap(),
                    sp[3].parse().unwrap(),
                    sp[4] == "1",
                    sp[5] == "1",
                );
                tq.work_orders.insert(0, wo);
            }
        }
        tq.priority_build = prio;
        tq.frame_started = frame_started;
        tq.sent_to_start_location = sent;
        tq.stop_queueing = stop;
        tq.reinforcement = reinf;
        tq.reinforcement_id = reinf_id;

        let got_flags = format!(
            "{}{}{}{}",
            u8::from(tq.is_all_built()),
            u8::from(tq.is_minimum_built()),
            u8::from(tq.includes_a_dozer(&f.sim.rules.thing_factory)),
            u8::from(tq.are_builds_complete()),
        );
        assert_eq!(got_flags, flags, "TIQXFER {idx} flags");

        if team_id != TeamId::INVALID {
            let proto_id = f.teams.team(team_id).unwrap().prototype;
            let mut got_bits = String::new();
            for idle in [-1, 0, 1, 30] {
                let frames = [
                    0u32,
                    frame_started.wrapping_add(idle).wrapping_sub(1) as UnsignedInt,
                    frame_started.wrapping_add(idle) as UnsignedInt,
                    frame_started.wrapping_add(idle).wrapping_add(1) as UnsignedInt,
                ];
                for frame in frames {
                    got_bits.push(if tq.is_build_time_expired(frame, idle) {
                        '1'
                    } else {
                        '0'
                    });
                }
            }
            let _ = proto_id;
            assert_eq!(got_bits, exp_bits, "TIQXFER {idx} exp_bits");
        }

        let mut xc = XferCrc::new();
        tq.crc(&mut xc).unwrap();
        assert_eq!(xc.get_crc(), exp_crc, "TIQXFER {idx} crc");

        let mut xs = XferSave::new();
        tq.xfer(&mut xs, Some(&f.sim.rules.thing_factory), Some(&f.teams))
            .unwrap();
        let bytes = xs.into_bytes();
        assert_eq!(hex(&bytes), exp_hex, "TIQXFER {idx} xfer");

        let mut loaded = TeamInQueue::new();
        let mut xl = XferLoad::new(&bytes);
        loaded
            .xfer(&mut xl, Some(&f.sim.rules.thing_factory), Some(&f.teams))
            .unwrap();
        assert_eq!(loaded.work_orders.len(), tq.work_orders.len());
        assert_eq!(loaded.team_id, tq.team_id);
        assert_eq!(loaded.priority_build, tq.priority_build);
        assert_eq!(loaded.frame_started, tq.frame_started);
        assert_eq!(loaded.reinforcement_id, tq.reinforcement_id);
        tiq_count += 1;
    }
    assert_eq!(tiq_count, 200);

    let mut ai_count = 0;
    for line in rows().filter(|l| l.starts_with("AIXFER ")) {
        let p: Vec<&str> = line.split_whitespace().collect();
        let idx: usize = p[1].parse().unwrap();
        let sk = p[2] == "1";
        let nb: usize = p[3].parse().unwrap();
        let nr: usize = p[4].parse().unwrap();
        let exp_crc = u32::from_str_radix(p[5], 16).unwrap();
        let exp_hex = p[6];

        let mut af = setup_world_at(sk, if idx == 0 { 32 } else { 1 });
        let t1 = create_team(&mut af, "TeamAlpha", true);
        let t2 = find_team(&mut af, "TeamLow");

        let mut build_items = Vec::new();
        for b in 0..nb {
            let team = if b == 0 { t1 } else { t2 };
            let mut tq = make_tiq(&af.teams, team);
            let wo = make_wo_by_idx(
                &af,
                1 + b,
                ObjectId::INVALID,
                b as Int,
                b as Int + 2,
                true,
                false,
            );
            tq.work_orders.push(wo);
            tq.priority_build = b % 2 == 0;
            tq.frame_started = 10 + b as Int;
            build_items.push(tq);
        }
        let mut ready_items = Vec::new();
        for r in 0..nr {
            let mut tq = make_tiq(&af.teams, t2);
            let wo = make_wo_by_idx(&af, 2 + r, ObjectId::INVALID, 2, 2, true, false);
            tq.work_orders.push(wo);
            tq.frame_started = 20 + r as Int;
            tq.reinforcement = true;
            tq.reinforcement_id = ObjectId(50 + r as Int);
            ready_items.push(tq);
        }

        let ai = af.sim.world.ai_players.get_mut(&1).unwrap();
        for tq in build_items {
            ai.team_build_queue.insert(0, tq);
        }
        for tq in ready_items {
            ai.team_ready_queue.insert(0, tq);
        }
        ai.repair_dozer_origin = Coord3D::new(12.5 + idx as Real, -4.0, 0.0);
        ai.supply_source_attack_check_frame = 100 + idx as UnsignedInt;
        ai.attacked_supply_center = if idx % 3 == 0 {
            ObjectId(77)
        } else {
            ObjectId::INVALID
        };

        let mut xc = XferCrc::new();
        ai.crc(&mut xc).unwrap();
        assert_eq!(xc.get_crc(), exp_crc, "AIXFER {idx} crc");

        let mut xs = XferSave::new();
        ai.xfer_with(&mut xs, Some(&af.sim.rules.thing_factory), Some(&af.teams))
            .unwrap();
        let bytes = xs.into_bytes();
        assert_eq!(hex(&bytes), exp_hex, "AIXFER {idx} xfer");

        // Load into empty AiPlayer succeeds; loading into non-empty queue fails with InvalidData.
        let mut empty = AiPlayer::new(
            if sk {
                AiKind::Skirmish
            } else {
                AiKind::Standard
            },
            1,
            1,
            DIFFICULTY_EASY,
            0,
        );
        let mut xl = XferLoad::new(&bytes);
        empty
            .xfer_with(&mut xl, Some(&af.sim.rules.thing_factory), Some(&af.teams))
            .unwrap();
        assert_eq!(empty.team_build_queue.len(), nb);
        assert_eq!(empty.team_ready_queue.len(), nr);
        assert_eq!(empty.difficulty, ai.difficulty);
        assert_eq!(
            empty.frame_last_building_built,
            ai.frame_last_building_built
        );
        let mut xs_rt = XferSave::new();
        empty
            .xfer_with(
                &mut xs_rt,
                Some(&af.sim.rules.thing_factory),
                Some(&af.teams),
            )
            .unwrap();
        assert_eq!(hex(&xs_rt.into_bytes()), exp_hex, "AIXFER {idx} roundtrip");

        if nb > 0 {
            let mut non_empty = empty.clone();
            let mut xl2 = XferLoad::new(&bytes);
            assert_eq!(
                non_empty.xfer_with(&mut xl2, Some(&af.sim.rules.thing_factory), Some(&af.teams)),
                Err(XferError::InvalidData)
            );
        }

        let ai_mut = af.sim.world.ai_players.get_mut(&1).unwrap();
        ai_mut.clear_teams_in_queue(&mut af.teams);
        assert!(ai_mut.team_build_queue.is_empty());
        assert!(ai_mut.team_ready_queue.is_empty());
        ai_count += 1;
    }
    assert_eq!(ai_count, 60);
    let _ = unhex;
}

#[test]
fn golden_is_possible_to_build_team_1000() {
    let mut f = setup_world(true);
    let p1_def = f.sim.world.players.player(1).unwrap().default_team.unwrap();
    f.sim.world.players.player_mut(1).unwrap().can_build_units = true;
    let fa = spawn(&mut f, "FactA", p1_def, Coord3D::new(100.0, 100.0, 0.0));
    let fb = spawn(&mut f, "FactB", p1_def, Coord3D::new(200.0, 100.0, 0.0));
    let fa_id = f.sim.world.objects.id_of(fa).unwrap();
    let fb_id = f.sim.world.objects.id_of(fb).unwrap();

    let mut bl_a = BuildListInfo::new();
    bl_a.template_name = "FactA".to_owned();
    bl_a.object_id = fa_id;
    let mut bl_b = BuildListInfo::new();
    bl_b.template_name = "FactB".to_owned();
    bl_b.object_id = fb_id;
    f.sim.world.players.player_mut(1).unwrap().build_list = vec![bl_a, bl_b];

    let proto_id = f
        .teams
        .find_team_prototype("TeamTestPoss", &mut f.sim.rules.name_keys)
        .unwrap();
    let tmpl_a = f.sim.rules.thing_factory.find_template_id("UnitA").unwrap();
    let tmpl_b = f.sim.rules.thing_factory.find_template_id("UnitB").unwrap();

    let mut count = 0;
    for line in rows().filter(|l| l.starts_with("ISPOSS ")) {
        let p: Vec<&str> = line.split_whitespace().collect();
        let idx: usize = p[1].parse().unwrap();
        let res = Real::from_bits(u32::from_str_radix(p[2], 16).unwrap());
        let fa_present = p[3] == "1";
        let fa_busy = p[4] == "1";
        let fb_busy = p[5] == "1";
        let require_idle = p[6] == "1";
        let money: UnsignedInt = p[7].parse().unwrap();
        let uspec = p[8];
        let exp_ok = p[9] == "1";
        let exp_need = p[10] == "1";

        f.sim.rules.ai_data.team_resources_to_build = res;
        f.sim.with_production(fa, |pu, sim, ctx| {
            pu.cancel_and_refund_all_production(sim, ctx)
        });
        f.sim.with_production(fb, |pu, sim, ctx| {
            pu.cancel_and_refund_all_production(sim, ctx)
        });
        if fa_busy {
            f.sim.with_production(fa, |pu, sim, ctx| {
                let uid = pu.request_unique_unit_id();
                pu.queue_create_unit(sim, ctx, tmpl_a, uid);
            });
        }
        if fb_busy {
            f.sim.with_production(fb, |pu, sim, ctx| {
                let uid = pu.request_unique_unit_id();
                pu.queue_create_unit(sim, ctx, tmpl_b, uid);
            });
        }
        f.sim.world.players.player_mut(1).unwrap().build_list[0].object_id =
            if fa_present { fa_id } else { ObjectId::INVALID };
        let mut units = Vec::new();
        for entry in uspec.split(';') {
            let up: Vec<&str> = entry.split(',').collect();
            units.push(CreateUnitsInfo {
                unit_thing_name: up[0].to_owned(),
                min_units: up[1].parse().unwrap(),
                max_units: up[2].parse().unwrap(),
            });
        }
        f.teams.prototype_mut(proto_id).unwrap().template.units = units;
        set_money(&mut f, 1, money);

        let (ok, need) =
            is_possible_to_build_team(&mut f.sim, &f.teams, 1, proto_id, require_idle);
        assert_eq!(
            (ok, need),
            (exp_ok, exp_need),
            "ISPOSS {idx}: res={res} fa_present={fa_present} fa_busy={fa_busy} fb_busy={fb_busy} req_idle={require_idle} money={money} uspec={uspec}"
        );
        count += 1;
    }
    assert_eq!(count, 1000);
}

#[test]
fn golden_do_team_building_and_build_specific_and_eval_and_recruit_and_queues() {
    // 1. DOTB
    let mut dotb_count = 0;
    for line in rows().filter(|l| l.starts_with("DOTB ")) {
        let p: Vec<&str> = line.split_whitespace().collect();
        let ci: usize = p[1].parse().unwrap();
        let sk = p[2] == "1";
        let can_units = p[3] == "1";
        let timer: Int = p[4].parse().unwrap();
        let ready = p[5] == "1";
        let delay: Int = p[6].parse().unwrap();
        let seq = p[7];

        let mut f = setup_world(sk);
        f.sim.world.players.player_mut(1).unwrap().can_build_units = can_units;
        {
            let ai = f.sim.world.ai_players.get_mut(&1).unwrap();
            ai.team_timer = timer;
            ai.ready_to_build_team = Bool::from(ready);
            ai.team_delay = delay;
        }
        let mut got = Vec::new();
        for _ in 0..8 {
            with_cx(&mut f, |cx| do_team_building(cx, 1));
            let ai = f.sim.world.ai_players.get(&1).unwrap();
            got.push(format!(
                "{},{},{}",
                i32::from(ai.ready_to_build_team != FALSE),
                ai.team_timer,
                ai.team_delay
            ));
        }
        assert_eq!(got.join(";"), seq, "DOTB {ci}");
        dotb_count += 1;
    }
    assert_eq!(dotb_count, 6);

    // 2. BUILDTEAM
    let mut bt_count = 0;
    {
        let mut f = setup_world(true);
        let p1_def = f.sim.world.players.player(1).unwrap().default_team.unwrap();
        f.sim.world.players.player_mut(1).unwrap().can_build_units = true;
        let fa = spawn(&mut f, "FactA", p1_def, Coord3D::new(100.0, 100.0, 0.0));
        let fb = spawn(&mut f, "FactB", p1_def, Coord3D::new(200.0, 100.0, 0.0));
        let fa_id = f.sim.world.objects.id_of(fa).unwrap();
        let fb_id = f.sim.world.objects.id_of(fb).unwrap();
        let mut bl_a = BuildListInfo::new();
        bl_a.template_name = "FactA".to_owned();
        bl_a.object_id = fa_id;
        let mut bl_b = BuildListInfo::new();
        bl_b.template_name = "FactB".to_owned();
        bl_b.object_id = fb_id;
        f.sim.world.players.player_mut(1).unwrap().build_list = vec![bl_a, bl_b];

        for line in rows().filter(|l| l.starts_with("BUILDTEAM ")) {
            let p: Vec<&str> = line.split_whitespace().collect();
            let proto_name = p[1];
            let prio = p[2] == "1";
            let exp_q = p[3];

            f.sim
                .world
                .ai_players
                .get_mut(&1)
                .unwrap()
                .clear_teams_in_queue(&mut f.teams);
            f.sim.with_production(fa, |pu, sim, ctx| {
                pu.cancel_and_refund_all_production(sim, ctx)
            });
            f.sim.with_production(fb, |pu, sim, ctx| {
                pu.cancel_and_refund_all_production(sim, ctx)
            });

            let proto_id = f
                .teams
                .find_team_prototype(proto_name, &mut f.sim.rules.name_keys)
                .unwrap();
            with_cx(&mut f, |cx| build_specific_ai_team(cx, 1, proto_id, prio));

            let ai = f.sim.world.ai_players.get(&1).unwrap();
            let mut qparts = Vec::new();
            for tq in &ai.team_build_queue {
                let mut wparts = Vec::new();
                for wo in &tq.work_orders {
                    let name = wo
                        .thing
                        .and_then(|id| {
                            f.sim
                                .rules
                                .thing_factory
                                .find_by_template_id_final(id.0 as u16)
                        })
                        .map_or("-", |t| t.get_name());
                    wparts.push(format!(
                        "{name},{},{},{}",
                        wo.num_required,
                        u8::from(wo.required),
                        u8::from(wo.factory_id != ObjectId::INVALID)
                    ));
                }
                qparts.push(format!(
                    "{}:{}:{}:{}",
                    u8::from(tq.priority_build),
                    u8::from(tq.reinforcement),
                    u8::from(tq.team_id != TeamId::INVALID),
                    wparts.join(";")
                ));
            }
            let got = if qparts.is_empty() {
                "-".to_owned()
            } else {
                qparts.join("|")
            };
            assert_eq!(got, exp_q, "BUILDTEAM {proto_name} {prio}");
            bt_count += 1;
        }
    }
    assert_eq!(bt_count, 10);

    // 3. EVALCOND & EVALDELAY
    let mut ec_count = 0;
    let mut ed_count = 0;
    {
        let mut f = setup_world(true);
        let diffs = [DIFFICULTY_EASY, DIFFICULTY_NORMAL, DIFFICULTY_HARD];
        for line in rows().filter(|l| l.starts_with("EVALCOND ")) {
            let p: Vec<&str> = line.split_whitespace().collect();
            let d_idx: usize = p[1].parse().unwrap();
            let pname = p[2];
            let exp_r1 = p[3] == "1";
            let exp_af = p[4] == "1";
            let exp_r2 = p[5] == "1";

            f.sim.world.ai_players.get_mut(&1).unwrap().difficulty = diffs[d_idx];
            let proto_id = f
                .teams
                .find_team_prototype(pname, &mut f.sim.rules.name_keys)
                .unwrap();
            f.teams
                .prototype_mut(proto_id)
                .unwrap()
                .production_condition_always_false = false;
            let r1 = with_cx(&mut f, |cx| evaluate_production_condition(cx, proto_id));
            let af = f
                .teams
                .prototype(proto_id)
                .unwrap()
                .production_condition_always_false;
            let r2 = with_cx(&mut f, |cx| evaluate_production_condition(cx, proto_id));
            assert_eq!(
                (r1, af, r2),
                (exp_r1, exp_af, exp_r2),
                "EVALCOND d={d_idx} {pname}"
            );
            ec_count += 1;
        }
        f.sim.world.ai_players.get_mut(&1).unwrap().difficulty = DIFFICULTY_NORMAL;
        let pdel = f
            .teams
            .find_team_prototype("EC_Delay3", &mut f.sim.rules.name_keys)
            .unwrap();
        for line in rows().filter(|l| l.starts_with("EVALDELAY ")) {
            let p: Vec<&str> = line.split_whitespace().collect();
            let frame: UnsignedInt = p[1].parse().unwrap();
            let exp_r = p[2] == "1";
            f.sim.world.frame = frame;
            let r = with_cx(&mut f, |cx| evaluate_production_condition(cx, pdel));
            assert_eq!(r, exp_r, "EVALDELAY frame={frame}");
            ed_count += 1;
        }
    }
    assert_eq!(ec_count, 15);
    assert_eq!(ed_count, 8);

    // 4. RECRUIT
    let mut rec_count = 0;
    {
        let mut f = setup_world(true);
        let p1_def = f.sim.world.players.player(1).unwrap().default_team.unwrap();
        let p2_def = f.sim.world.players.player(2).unwrap().default_team.unwrap();

        for name in ["RecSrcOk", "RecSrcHighPrio"] {
            let pid = f
                .teams
                .find_team_prototype(name, &mut f.sim.rules.name_keys)
                .unwrap();
            f.teams
                .prototype_mut(pid)
                .unwrap()
                .template
                .is_ai_recruitable = true;
        }
        let t_target = create_team(&mut f, "RecruitTarget", true);
        let t_ok = create_team(&mut f, "RecSrcOk", true);
        let _t_ok_inactive = create_team(&mut f, "RecSrcOk", false);
        let t_high = create_team(&mut f, "RecSrcHighPrio", true);
        let t_noflag = create_team(&mut f, "RecSrcNoFlag", true);
        let t_override_on = create_team(&mut f, "RecSrcNoFlag", true);
        f.teams
            .team_mut(t_override_on)
            .unwrap()
            .set_recruitable(true);

        let tb = f.sim.rules.thing_factory.find_template_id("UnitB").unwrap();
        let ta = f.sim.rules.thing_factory.find_template_id("UnitA").unwrap();
        let home = Coord3D::new(0.0, 0.0, 0.0);

        f.sim.world.next_object_id = 7;
        let u_def_far = spawn(&mut f, "UnitBVar", p1_def, Coord3D::new(5000.0, 0.0, 0.0));
        let _u_enemy = spawn(&mut f, "UnitB", p2_def, Coord3D::new(5.0, 0.0, 0.0));
        let _u_high = spawn(&mut f, "UnitB", t_high, Coord3D::new(10.0, 0.0, 0.0));
        let _u_inact = spawn(
            &mut f,
            "UnitB",
            _t_ok_inactive,
            Coord3D::new(12.0, 0.0, 0.0),
        );
        let _u_noflag = spawn(&mut f, "UnitB", t_noflag, Coord3D::new(14.0, 0.0, 0.0));
        let u_held = spawn(&mut f, "UnitB", t_ok, Coord3D::new(15.0, 0.0, 0.0));
        f.sim.set_disabled(u_held, DisabledType::Held);
        let _ = u_def_far;

        for line in rows().filter(|l| l.starts_with("RECRUIT ")) {
            let p: Vec<&str> = line.split_whitespace().collect();
            let case = p[1];
            let exp_id: Int = p[2].parse().unwrap();
            let got_slot = match case {
                "def_far" => {
                    try_to_recruit(&f.sim, &f.teams, t_target, tb, &home, 100.0)
                }
                "nondef_out_of_range" => {
                    let _u_ok_far = spawn(&mut f, "UnitB", t_ok, Coord3D::new(250.0, 0.0, 0.0));
                    try_to_recruit(&f.sim, &f.teams, t_target, tb, &home, 100.0)
                }
                "nondef_in_range" => {
                    try_to_recruit(&f.sim, &f.teams, t_target, tb, &home, 300.0)
                }
                "override_on" => {
                    let _u_ov_on =
                        spawn(&mut f, "UnitB", t_override_on, Coord3D::new(80.0, 0.0, 0.0));
                    try_to_recruit(&f.sim, &f.teams, t_target, tb, &home, 300.0)
                }
                "none_ta" => {
                    f.teams.team_mut(t_ok).unwrap().set_recruitable(false);
                    f.teams
                        .team_mut(t_override_on)
                        .unwrap()
                        .set_recruitable(false);
                    try_to_recruit(&f.sim, &f.teams, t_target, ta, &home, 300.0)
                }
                other => panic!("unexpected RECRUIT case {other}"),
            };
            let got_id = got_slot
                .and_then(|s| f.sim.world.objects.id_of(s))
                .map_or(0, |id| id.0);
            assert_eq!(got_id, exp_id, "RECRUIT {case}");
            rec_count += 1;
        }
    }
    assert_eq!(rec_count, 5);

    // 5. QDOZ & QTRUCK
    for line in rows().filter(|l| l.starts_with("QDOZ ")) {
        let p: Vec<&str> = line.split_whitespace().collect();
        let exp_can_after = p[1] == "1";
        let exp_cnt: usize = p[2].parse().unwrap();
        let exp_first = p[3];

        let mut f = setup_world(true);
        let p1_def = f.sim.world.players.player(1).unwrap().default_team.unwrap();
        f.sim.world.players.player_mut(1).unwrap().can_build_units = false;
        set_money(&mut f, 1, 5000);
        let fa = spawn(&mut f, "FactA", p1_def, Coord3D::new(100.0, 100.0, 0.0));
        let mut bl_a = BuildListInfo::new();
        bl_a.template_name = "FactA".to_owned();
        bl_a.object_id = f.sim.world.objects.id_of(fa).unwrap();
        f.sim.world.players.player_mut(1).unwrap().build_list = vec![bl_a];

        with_cx(&mut f, |cx| queue_dozer(cx, 1));
        let can_after1 = f.sim.world.players.player(1).unwrap().can_build_units;
        with_cx(&mut f, |cx| queue_dozer(cx, 1));
        let ai = f.sim.world.ai_players.get(&1).unwrap();
        let first_thing = ai
            .team_build_queue
            .first()
            .and_then(|tq| tq.work_orders.first())
            .and_then(|wo| wo.thing)
            .and_then(|id| {
                f.sim
                    .rules
                    .thing_factory
                    .find_by_template_id_final(id.0 as u16)
            })
            .map_or("-", |t| t.get_name());
        assert_eq!(can_after1, exp_can_after, "QDOZ can_after");
        assert_eq!(ai.team_build_queue.len(), exp_cnt, "QDOZ len");
        assert_eq!(first_thing, exp_first, "QDOZ first_thing");
    }

    for line in rows().filter(|l| l.starts_with("QTRUCK ")) {
        let p: Vec<&str> = line.split_whitespace().collect();
        let exp_q1: usize = p[1].parse().unwrap();
        let exp_q2: usize = p[2].parse().unwrap();
        let exp_thing = p[3];
        let exp_gath = p[4] == "1";

        let mut f = setup_world(true);
        let p0_def = f.sim.world.players.player(0).unwrap().default_team.unwrap();
        let p1_def = f.sim.world.players.player(1).unwrap().default_team.unwrap();
        f.sim.world.players.player_mut(1).unwrap().can_build_units = true;
        set_money(&mut f, 1, 5000);
        let fa = spawn(&mut f, "FactA", p1_def, Coord3D::new(100.0, 100.0, 0.0));
        let sc = spawn(&mut f, "SupplyCtr", p1_def, Coord3D::new(200.0, 100.0, 0.0));
        let _wh = spawn(&mut f, "Warehouse", p0_def, Coord3D::new(230.0, 100.0, 0.0));

        let mut bl_a = BuildListInfo::new();
        bl_a.template_name = "FactA".to_owned();
        bl_a.object_id = f.sim.world.objects.id_of(fa).unwrap();
        let mut bl_sc = BuildListInfo::new();
        bl_sc.template_name = "SupplyCtr".to_owned();
        bl_sc.object_id = f.sim.world.objects.id_of(sc).unwrap();
        bl_sc.is_supply_building = TRUE;
        bl_sc.current_gatherers = -1;
        bl_sc.desired_gatherers = 1;
        f.sim.world.players.player_mut(1).unwrap().build_list = vec![bl_a, bl_sc];

        with_cx(&mut f, |cx| queue_supply_truck(cx, 1));
        let q1 = f
            .sim
            .world
            .ai_players
            .get(&1)
            .unwrap()
            .team_build_queue
            .len();
        let (thing, is_gath) = {
            let ai = f.sim.world.ai_players.get(&1).unwrap();
            let wo = &ai.team_build_queue[0].work_orders[0];
            let nm = wo
                .thing
                .and_then(|id| {
                    f.sim
                        .rules
                        .thing_factory
                        .find_by_template_id_final(id.0 as u16)
                })
                .map_or("-", |t| t.get_name());
            (nm.to_owned(), wo.is_resource_gatherer)
        };
        with_cx(&mut f, |cx| queue_supply_truck(cx, 1));
        let q2 = f
            .sim
            .world
            .ai_players
            .get(&1)
            .unwrap()
            .team_build_queue
            .len();
        assert_eq!(
            (q1, q2, thing.as_str(), is_gath),
            (exp_q1, exp_q2, exp_thing, exp_gath)
        );
    }
}

#[test]
fn integration_production_hooks_ready_queued_teams_and_scripts() {
    // 1. on_unit_produced_in_sim: same-player transfer (no capture score!), FollowExitProductionPath,
    //    Worker force_pending + pending_supply_truck_dock, Dozer structure_timer=1, clear factory_id.
    {
        let mut f = setup_world(true);
        let worker_ini = concat!(
            "Object GlaWorker\n",
            "  Side = America\n",
            "  KindOf = PRELOAD SELECTABLE VEHICLE HARVESTER\n",
            "  BuildCost = 600\n  BuildTime = 1.0\n",
            "  Geometry = CYLINDER\n  GeometryMajorRadius = 6\n  GeometryHeight = 8\n",
            "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 150\n    InitialHealth = 150\n  End\n",
            "  Behavior = WorkerAIUpdate ModuleTag_AI\n",
            "    MaxBoxes = 4\n",
            "    RepairHealthPercentPerSecond = 2%\n",
            "    BoredTime = 5000\n",
            "    BoredRange = 150\n",
            "  End\n",
            "  Locomotor = SET_NORMAL AptGround\n",
            "End\n",
        );
        let mut reader =
            IniReader::new("worker.ini", worker_ini.as_bytes(), IniLoadType::Overwrite);
        f.sim
            .rules
            .load_thing_templates_from_ini(&mut reader)
            .unwrap();
        f.sim.rules.thing_factory.post_process_load();

        let p1_def = f.sim.world.players.player(1).unwrap().default_team.unwrap();
        let t_alpha = create_team(&mut f, "TeamAlpha", false);
        f.teams
            .prototype_mut(f.teams.team(t_alpha).unwrap().prototype)
            .unwrap()
            .template
            .has_home_location = true;
        f.teams
            .prototype_mut(f.teams.team(t_alpha).unwrap().prototype)
            .unwrap()
            .template
            .home_location = Coord3D::new(300.0, 200.0, 0.0);

        let fa = spawn(&mut f, "FactA", p1_def, Coord3D::new(100.0, 100.0, 0.0));
        let fa_id = f.sim.world.objects.id_of(fa).unwrap();
        let sc = spawn(&mut f, "SupplyCtr", p1_def, Coord3D::new(200.0, 100.0, 0.0));
        let sc_id = f.sim.world.objects.id_of(sc).unwrap();

        let unit_a = spawn(&mut f, "UnitA", p1_def, Coord3D::new(105.0, 100.0, 0.0));
        let worker = spawn(&mut f, "GlaWorker", p1_def, Coord3D::new(110.0, 100.0, 0.0));
        let dozer = spawn(&mut f, "DozerA", p1_def, Coord3D::new(115.0, 100.0, 0.0));

        let t_unit = f.sim.rules.thing_factory.find_template_id("UnitA").unwrap();
        let t_worker = f
            .sim
            .rules
            .thing_factory
            .find_template_id("GlaWorker")
            .unwrap();
        let t_dozer = f
            .sim
            .rules
            .thing_factory
            .find_template_id("DozerA")
            .unwrap();

        let mut tq = make_tiq(&f.teams, t_alpha);
        tq.has_home_location = true;
        tq.home_location = Coord3D::new(300.0, 200.0, 0.0);
        tq.reinforcement = true;
        let mut wo1 = WorkOrder::new();
        wo1.thing = Some(t_unit);
        wo1.factory_id = fa_id;
        wo1.num_required = 1;
        let mut wo2 = WorkOrder::new();
        wo2.thing = Some(t_worker);
        wo2.factory_id = fa_id;
        wo2.num_required = 1;
        wo2.is_resource_gatherer = true;
        let mut wo3 = WorkOrder::new();
        wo3.thing = Some(t_dozer);
        wo3.factory_id = fa_id;
        wo3.num_required = 1;
        tq.work_orders = vec![wo1, wo2, wo3];
        {
            let ai = f.sim.world.ai_players.get_mut(&1).unwrap();
            ai.team_build_queue.push(tq);
            ai.team_delay = 50;
        }

        let mut bl_sc = BuildListInfo::new();
        bl_sc.template_name = "SupplyCtr".to_owned();
        bl_sc.object_id = sc_id;
        bl_sc.is_supply_building = TRUE;
        bl_sc.current_gatherers = 0;
        bl_sc.desired_gatherers = 1;
        f.sim.world.players.player_mut(1).unwrap().build_list = vec![bl_sc];

        f.sim.on_unit_created(fa, unit_a);
        f.sim.on_unit_created(fa, worker);
        f.sim.on_unit_created(fa, dozer);

        // Same-player transfer must NOT increment object capture score.
        assert!(f
            .sim
            .world
            .players
            .player(1)
            .unwrap()
            .score_keeper
            .objects_captured
            .is_empty());
        assert_eq!(f.sim.world.objects.get(unit_a).unwrap().team_id, t_alpha);
        assert_eq!(f.sim.world.objects.get(worker).unwrap().team_id, t_alpha);
        assert_eq!(f.sim.world.objects.get(dozer).unwrap().team_id, t_alpha);

        // Worker has force_pending set and is queued for docking.
        let worker_ai_slot = f.sim.world.objects.get(worker).unwrap().ai_slot.unwrap();
        let forced = f
            .sim
            .dispatch_module(worker_ai_slot, |m, _, _| {
                m.as_any()
                    .and_then(|a| a.downcast_ref::<AiUpdate>())
                    .map(|ai| match &ai.derived {
                        DerivedAi::Worker(w) => w.force_pending != FALSE,
                        _ => false,
                    })
            })
            .unwrap()
            .unwrap();
        assert!(forced);

        // UnitA received FollowExitProductionPath toward home_location.
        let unit_ai_slot = f.sim.world.objects.get(unit_a).unwrap().ai_slot.unwrap();
        let state = f
            .sim
            .dispatch_module(unit_ai_slot, |m, _, _| {
                m.as_any()
                    .and_then(|a| a.downcast_ref::<AiUpdate>())
                    .map(|ai| ai.machine.current_state_id())
            })
            .unwrap()
            .unwrap();
        assert_eq!(state, state_id::FOLLOW_EXITPRODUCTION_PATH);

        // AI timers & reinforcement_id updated synchronously.
        {
            let ai = f.sim.world.ai_players.get(&1).unwrap();
            assert_eq!(ai.team_delay, 0);
            assert_eq!(ai.build_delay, 0);
            assert_eq!(ai.structure_timer, 1);
            assert_eq!(
                ai.team_build_queue[0].reinforcement_id,
                f.sim.world.objects.id_of(dozer).unwrap()
            );
            assert_eq!(ai.team_build_queue[0].work_orders[0].num_completed, 1);
            assert_eq!(
                ai.team_build_queue[0].work_orders[0].factory_id,
                ObjectId::INVALID
            );
        }

        with_cx(&mut f, |cx| drain_pending_sim_events_for_player(cx, 1));
        assert_eq!(
            f.sim.world.players.player(1).unwrap().build_list[0].current_gatherers,
            1
        );
        assert!(f
            .teams
            .team(t_alpha)
            .unwrap()
            .members
            .contains(&f.sim.world.objects.id_of(unit_a).unwrap()));
    }

    // 2. player_on_structure_construction_complete -> pending_structures_produced -> on_structure_produced
    {
        let mut f = setup_world(true);
        let p1_def = f.sim.world.players.player(1).unwrap().default_team.unwrap();
        let sc = spawn(&mut f, "SupplyCtr", p1_def, Coord3D::new(200.0, 100.0, 0.0));
        let sc_id = f.sim.world.objects.id_of(sc).unwrap();
        f.sim.set_status(sc, ObjectStatusType::UnderConstruction);
        let mut bl_sc = BuildListInfo::new();
        bl_sc.template_name = "SupplyCtr".to_owned();
        bl_sc.building_name = "MySC".to_owned();
        bl_sc.object_id = sc_id;
        bl_sc.under_construction = TRUE;
        bl_sc.unsellable = TRUE;
        bl_sc.is_supply_building = TRUE;
        f.sim.world.players.player_mut(1).unwrap().build_list = vec![bl_sc];

        {
            let ai = f.sim.world.ai_players.get_mut(&1).unwrap();
            ai.team_delay = 99;
            ai.build_delay = 99;
        }
        f.sim.player_on_structure_construction_complete(sc, FALSE);
        {
            let ai = f.sim.world.ai_players.get(&1).unwrap();
            assert_eq!(ai.team_delay, 0);
            assert_eq!(ai.build_delay, 0);
        }
        with_cx(&mut f, |cx| on_structure_produced(cx, 1, sc));
        assert_eq!(
            f.sim.world.players.player(1).unwrap().build_list[0].under_construction,
            FALSE
        );
        assert!(f.named.iter().any(|(n, id)| n == "MySC" && *id == sc_id));
        assert_eq!(
            f.sim
                .world
                .objects
                .get(sc)
                .unwrap()
                .status
                .test(ObjectStatusType::UnderConstruction as Int),
            FALSE
        );
        assert!(
            f.unported.is_empty(),
            "unexpected unported: {:?}",
            f.unported
        );
    }

    // 3. check_queued_teams (promote all-built + disband expired incomplete) & check_ready_teams (activate vs joinTeam)
    {
        let mut f = setup_world(true);
        let p1_def = f.sim.world.players.player(1).unwrap().default_team.unwrap();
        let t_built = create_team(&mut f, "TeamAlpha", false);
        let t_expired = create_team(&mut f, "TeamAlpha", false);
        let u_exp = spawn(&mut f, "UnitA", t_expired, Coord3D::new(50.0, 50.0, 0.0));
        let u_exp_id = f.sim.world.objects.id_of(u_exp).unwrap();
        let u_ready = spawn(&mut f, "UnitA", t_built, Coord3D::new(60.0, 60.0, 0.0));

        // Built team -> promoted from build_queue to ready_queue -> activated by check_ready_teams.
        let mut tq_built = make_tiq(&f.teams, t_built);
        let mut wo_done = WorkOrder::new();
        wo_done.num_completed = 1;
        wo_done.num_required = 1;
        wo_done.required = true;
        tq_built.work_orders.push(wo_done);

        // Expired incomplete team -> disbanded (units transferred to default team, non-singleton instance deleted).
        let p_alpha = f.teams.team(t_expired).unwrap().prototype;
        f.teams
            .prototype_mut(p_alpha)
            .unwrap()
            .template
            .initial_idle_frames = 5;
        let mut tq_exp = make_tiq(&f.teams, t_expired);
        tq_exp.frame_started = 1;
        let mut wo_inc = WorkOrder::new();
        wo_inc.num_completed = 0;
        wo_inc.num_required = 2;
        wo_inc.required = true;
        tq_exp.work_orders.push(wo_inc);

        // Reinforcement ready team -> issues joinTeam.
        let t_reinf = create_team(&mut f, "TeamAlpha", true);
        let r_unit = spawn(&mut f, "UnitA", t_reinf, Coord3D::new(70.0, 70.0, 0.0));
        let r_id = f.sim.world.objects.id_of(r_unit).unwrap();
        let mut tq_reinf = make_tiq(&f.teams, t_reinf);
        tq_reinf.reinforcement = true;
        tq_reinf.reinforcement_id = r_id;

        {
            let ai = f.sim.world.ai_players.get_mut(&1).unwrap();
            ai.team_build_queue.push(tq_built);
            ai.team_build_queue.push(tq_exp);
            ai.team_ready_queue.push(tq_reinf);
        }

        f.sim.world.frame = 20;
        with_cx(&mut f, |cx| check_queued_teams(cx, 1));
        assert!(f
            .sim
            .world
            .ai_players
            .get(&1)
            .unwrap()
            .team_build_queue
            .is_empty());
        assert!(
            f.teams.team(t_expired).is_none(),
            "expired non-singleton deleted"
        );
        assert_eq!(f.sim.world.objects.get(u_exp).unwrap().team_id, p1_def);
        assert!(f.teams.team(p1_def).unwrap().members.contains(&u_exp_id));

        with_cx(&mut f, |cx| check_ready_teams(cx, 1));
        assert!(f
            .sim
            .world
            .ai_players
            .get(&1)
            .unwrap()
            .team_ready_queue
            .is_empty());
        assert!(f.teams.team(t_built).unwrap().active);
        let _ = u_ready;
    }

    // 4. Script actions BUILD_TEAM (69) & RECRUIT_TEAM (177) + ai_pre_team_destroy
    {
        let mut f = setup_world(true);
        let p1_def = f.sim.world.players.player(1).unwrap().default_team.unwrap();
        let fa = spawn(&mut f, "FactA", p1_def, Coord3D::new(100.0, 100.0, 0.0));
        let mut bl_a = BuildListInfo::new();
        bl_a.template_name = "FactA".to_owned();
        bl_a.object_id = f.sim.world.objects.id_of(fa).unwrap();
        f.sim.world.players.player_mut(1).unwrap().build_list = vec![bl_a];
        // One idle unit on default team so RECRUIT_TEAM recruits it immediately.
        let recruitable_unit = spawn(&mut f, "UnitA", p1_def, Coord3D::new(10.0, 10.0, 0.0));

        let reg = script_template_registry(&mut f.sim.rules.name_keys);
        let mut act_build = ScriptAction::new(at::BUILD_TEAM, &reg);
        let mut p_team = Parameter::new(ParameterType::Team, 0);
        p_team.string_val = "BT_Min1Max4".to_owned();
        act_build.parms = vec![p_team];

        let mut act_recruit = ScriptAction::new(at::RECRUIT_TEAM, &reg);
        let mut p_rec = Parameter::new(ParameterType::Team, 0);
        p_rec.string_val = "TeamAlpha".to_owned();
        let mut p_rad = Parameter::new(ParameterType::Real, 0);
        p_rad.real_val = 500.0;
        act_recruit.parms = vec![p_rec, p_rad];

        let parts = ScriptGame {
            teams: &mut f.teams,
            stores: &f.stores,
            waypoints: &f.waypoints,
            named_objects: &mut f.named,
        };
        f.scripts.with_world(&mut f.sim, parts, |engine, world| {
            engine.execute_action(world, &mut act_build);
            engine.execute_action(world, &mut act_recruit);
        });

        let ai = f.sim.world.ai_players.get(&1).unwrap();
        assert_eq!(ai.team_build_queue.len(), 1);
        assert_eq!(ai.team_ready_queue.len(), 1);
        let recruited_team = ai.team_ready_queue[0].team_id;
        assert_eq!(
            f.sim.world.objects.get(recruitable_unit).unwrap().team_id,
            recruited_team
        );

        // Verify ai_pre_team_destroy clears queued entries on team deletion.
        let queued_team = ai.team_build_queue[0].team_id;
        let ts = f.sim.rules.ai_data.team_seconds as Int;
        let mut list = std::mem::take(&mut f.sim.world.players);
        let mut world = TestPlayerWorld {
            world: &mut f.sim.world,
            team_seconds: ts,
            global_difficulty: DIFFICULTY_EASY,
        };
        let mut env = PlayerEnv {
            templates: &f.stores.player_templates,
            ranks: &f.stores.ranks,
            sciences: &f.sim.rules.science_store,
            name_keys: &mut f.sim.rules.name_keys,
            world: &mut world,
        };
        f.teams
            .delete_team_instance(queued_team, &mut list, &mut env);
        f.sim.world.players = list;
        assert!(f
            .sim
            .world
            .ai_players
            .get(&1)
            .unwrap()
            .team_build_queue
            .is_empty());
    }
}

#[test]
fn integration_team_selection_queue_units_and_survivor_branches() {
    // 1. Fresh Hard-only and missing scripts in evaluate_production_condition.
    {
        let mut f = setup_world(true);
        let reg = script_template_registry(&mut f.sim.rules.name_keys);
        let hard_scr = make_cond_script(
            &reg,
            "ScrHardOnly",
            ct::CONDITION_TRUE,
            false,
            false,
            true,
            0,
        );
        f.scripts.engine.sides[1]
            .as_mut()
            .unwrap()
            .add_script(hard_scr, 4);
        let p_miss = f
            .teams
            .find_team_prototype("EC_Missing", &mut f.sim.rules.name_keys)
            .unwrap();
        assert!(!with_cx(&mut f, |cx| evaluate_production_condition(
            cx, p_miss
        )));
        assert!(
            f.teams
                .prototype(p_miss)
                .unwrap()
                .production_condition_always_false
        );

        let p_easy = f
            .teams
            .find_team_prototype("EC_Easy", &mut f.sim.rules.name_keys)
            .unwrap();
        let p_hard = f
            .teams
            .find_team_prototype("EC_True", &mut f.sim.rules.name_keys)
            .unwrap();
        f.teams
            .prototype_mut(p_easy)
            .unwrap()
            .template
            .production_condition = "ScrHardOnly".to_owned();
        f.teams
            .prototype_mut(p_hard)
            .unwrap()
            .template
            .production_condition = "ScrHardOnly".to_owned();

        // Easy player caches always_false=true and blocks even if difficulty later changes.
        f.sim.world.ai_players.get_mut(&1).unwrap().difficulty = DIFFICULTY_EASY;
        assert!(!with_cx(&mut f, |cx| evaluate_production_condition(
            cx, p_easy
        )));
        assert!(
            f.teams
                .prototype(p_easy)
                .unwrap()
                .production_condition_always_false
        );
        f.sim.world.ai_players.get_mut(&1).unwrap().difficulty = DIFFICULTY_HARD;
        assert!(!with_cx(&mut f, |cx| evaluate_production_condition(
            cx, p_easy
        )));

        // Fresh Hard prototype succeeds only on Hard.
        assert!(with_cx(&mut f, |cx| evaluate_production_condition(
            cx, p_hard
        )));
        assert!(
            !f.teams
                .prototype(p_hard)
                .unwrap()
                .production_condition_always_false
        );
    }

    // 2. select_team_to_reinforce, select_team_to_build, queue_units, check_queued_teams, check_ready_teams.
    {
        let mut f = setup_world(true);
        let p1_def = f.sim.world.players.player(1).unwrap().default_team.unwrap();
        f.sim.world.players.player_mut(1).unwrap().can_build_units = true;
        set_money(&mut f, 1, 5000);
        let fa = spawn(&mut f, "FactA", p1_def, Coord3D::new(100.0, 100.0, 0.0));
        let mut bl_a = BuildListInfo::new();
        bl_a.template_name = "FactA".to_owned();
        bl_a.object_id = f.sim.world.objects.id_of(fa).unwrap();
        f.sim.world.players.player_mut(1).unwrap().build_list = vec![bl_a];

        let p_no = f
            .teams
            .find_team_prototype("RecSrcNoFlag", &mut f.sim.rules.name_keys)
            .unwrap();
        let p_tgt = f
            .teams
            .find_team_prototype("RecruitTarget", &mut f.sim.rules.name_keys)
            .unwrap();
        let p_eq = f
            .teams
            .find_team_prototype("RecSrcOk", &mut f.sim.rules.name_keys)
            .unwrap();
        f.teams
            .prototype_mut(p_no)
            .unwrap()
            .template
            .automatically_reinforce = true;
        f.teams.prototype_mut(p_no).unwrap().template.units = vec![CreateUnitsInfo {
            min_units: 0,
            max_units: 0,
            unit_thing_name: "UnitA".to_owned(),
        }];
        f.teams
            .prototype_mut(p_tgt)
            .unwrap()
            .template
            .automatically_reinforce = true;
        f.teams.prototype_mut(p_tgt).unwrap().template.units = vec![CreateUnitsInfo {
            min_units: 1,
            max_units: 2,
            unit_thing_name: "UnitA".to_owned(),
        }];
        f.teams
            .prototype_mut(p_eq)
            .unwrap()
            .template
            .automatically_reinforce = true;
        f.teams
            .prototype_mut(p_eq)
            .unwrap()
            .template
            .production_priority = 10;
        f.teams.prototype_mut(p_eq).unwrap().template.units = vec![CreateUnitsInfo {
            min_units: 1,
            max_units: 2,
            unit_thing_name: "UnitA".to_owned(),
        }];
        let t_no = create_team(&mut f, "RecSrcNoFlag", true);
        let t_tgt = create_team(&mut f, "RecruitTarget", true);
        let _u_no = spawn(&mut f, "UnitA", t_no, Coord3D::new(20.0, 20.0, 0.0));
        let _u_tgt = spawn(&mut f, "UnitA", t_tgt, Coord3D::new(30.0, 30.0, 0.0));

        // Busy factory prevents select_team_to_reinforce (find_factory(..., false)).
        let tmpl_a = f.sim.rules.thing_factory.find_template_id("UnitA").unwrap();
        f.sim.with_production(fa, |pu, sim, ctx| {
            let uid = pu.request_unique_unit_id();
            pu.queue_create_unit(sim, ctx, tmpl_a, uid);
        });
        assert!(!with_cx(&mut f, |cx| select_team_to_reinforce(cx, 1, 9)));
        f.sim.with_production(fa, |pu, sim, ctx| {
            pu.cancel_and_refund_all_production(sim, ctx)
        });

        // Equal priority min_priority=10 must NOT reinforce (must be strictly >).
        assert!(!with_cx(&mut f, |cx| select_team_to_reinforce(cx, 1, 10)));

        f.sim.world.ai_players.get_mut(&1).unwrap().team_delay = 55;
        assert!(with_cx(&mut f, |cx| select_team_to_reinforce(cx, 1, 9)));
        {
            let ai = f.sim.world.ai_players.get(&1).unwrap();
            assert_eq!(ai.team_delay, 0);
            assert_eq!(ai.team_build_queue.len(), 1);
            assert!(!ai.team_build_queue[0].priority_build);
            assert_eq!(ai.team_build_queue[0].team_id, t_tgt);
        }
        f.sim
            .world
            .ai_players
            .get_mut(&1)
            .unwrap()
            .clear_teams_in_queue(&mut f.teams);
        f.sim.with_production(fa, |pu, sim, ctx| {
            pu.cancel_and_refund_all_production(sim, ctx)
        });

        // try_to_recruit ignores equal-priority team (priority 10 == target priority 10).
        let t_eq = create_team(&mut f, "RecSrcOk", true);
        f.teams
            .prototype_mut(p_eq)
            .unwrap()
            .template
            .is_ai_recruitable = true;
        let _u_eq = spawn(&mut f, "UnitA", t_eq, Coord3D::new(5.0, 0.0, 0.0));
        assert!(try_to_recruit(
            &f.sim,
            &f.teams,
            t_tgt,
            tmpl_a,
            &Coord3D::new(0.0, 0.0, 0.0),
            500.0
        )
        .is_none());

        // select_team_to_build sets ready_to_build_team=false, team_timer, and builds TeamAlpha (highest valid prio=10).
        {
            let ai = f.sim.world.ai_players.get_mut(&1).unwrap();
            ai.ready_to_build_team = TRUE;
            ai.team_delay = 44;
        }
        assert!(with_cx(&mut f, |cx| select_team_to_build(cx, 1)));
        assert_eq!(
            f.sim.world.ai_players.get(&1).unwrap().ready_to_build_team,
            FALSE
        );
        assert_eq!(f.sim.world.ai_players.get(&1).unwrap().team_delay, 0);
        f.sim
            .world
            .ai_players
            .get_mut(&1)
            .unwrap()
            .clear_teams_in_queue(&mut f.teams);
        f.sim.with_production(fa, |pu, sim, ctx| {
            pu.cancel_and_refund_all_production(sim, ctx)
        });

        // queue_units recruits 1 UnitA (num_completed += 1, NOT +2) and queues 1 at FactA.
        let idle_u = spawn(&mut f, "UnitA", p1_def, Coord3D::new(10.0, 10.0, 0.0));
        let t_q = create_team(&mut f, "BT_Min1Max4", false);
        let mut tq = make_tiq(&f.teams, t_q);
        let mut wo = WorkOrder::new();
        wo.thing = Some(tmpl_a);
        wo.num_required = 2;
        wo.required = true;
        tq.work_orders.push(wo);
        f.sim
            .world
            .ai_players
            .get_mut(&1)
            .unwrap()
            .team_build_queue
            .push(tq);
        f.sim.rules.ai_data.max_recruit_distance = 500.0;
        with_cx(&mut f, |cx| queue_units(cx, 1));
        {
            let ai = f.sim.world.ai_players.get(&1).unwrap();
            assert_eq!(ai.team_build_queue[0].work_orders[0].num_completed, 1);
            assert_ne!(
                ai.team_build_queue[0].work_orders[0].factory_id,
                ObjectId::INVALID
            );
            assert_eq!(f.sim.world.objects.get(idle_u).unwrap().team_id, t_q);
        }
        f.sim
            .world
            .ai_players
            .get_mut(&1)
            .unwrap()
            .clear_teams_in_queue(&mut f.teams);
        f.sim.with_production(fa, |pu, sim, ctx| {
            pu.cancel_and_refund_all_production(sim, ctx)
        });

        // check_queued_teams inserts promoted teams at head index 0 in reverse iteration order.
        let t_p1 = create_team(&mut f, "TeamAlpha", false);
        let t_p2 = create_team(&mut f, "TeamAlpha", false);
        let mut tq1 = make_tiq(&f.teams, t_p1);
        let mut tq2 = make_tiq(&f.teams, t_p2);
        let mut wo_done = WorkOrder::new();
        wo_done.num_completed = 1;
        wo_done.num_required = 1;
        wo_done.required = true;
        tq1.work_orders.push(wo_done.clone());
        tq2.work_orders.push(wo_done);
        {
            let ai = f.sim.world.ai_players.get_mut(&1).unwrap();
            ai.team_build_queue.push(tq1);
            ai.team_build_queue.push(tq2);
        }
        with_cx(&mut f, |cx| check_queued_teams(cx, 1));
        {
            let ai = f.sim.world.ai_players.get(&1).unwrap();
            assert_eq!(ai.team_ready_queue.len(), 2);
            assert_eq!(ai.team_ready_queue[0].team_id, t_p2);
            assert_eq!(ai.team_ready_queue[1].team_id, t_p1);
        }
        f.sim
            .world
            .ai_players
            .get_mut(&1)
            .unwrap()
            .clear_teams_in_queue(&mut f.teams);

        // check_ready_teams: reinforcement without AI object activates (all_idle=true);
        // moving unit waits until time_expired; execute_actions + one idle unit activates early.
        let t_r_noobj = create_team(&mut f, "TeamAlpha", false);
        let mut tq_noobj = make_tiq(&f.teams, t_r_noobj);
        tq_noobj.reinforcement = true;
        tq_noobj.reinforcement_id = ObjectId::INVALID;
        tq_noobj.frame_started = 1;

        let t_moving = create_team(&mut f, "TeamAlpha", false);
        let u_mv = spawn(&mut f, "UnitA", t_moving, Coord3D::new(20.0, 20.0, 0.0));
        f.sim.ai_move_to_position(
            u_mv,
            &Coord3D::new(200.0, 200.0, 0.0),
            CommandSourceType::FromAi,
        );
        let mut tq_mv = make_tiq(&f.teams, t_moving);
        tq_mv.frame_started = 1;

        let p_act = f
            .teams
            .find_team_prototype("TeamTestPoss", &mut f.sim.rules.name_keys)
            .unwrap();
        f.teams
            .prototype_mut(p_act)
            .unwrap()
            .template
            .execute_actions = true;
        f.teams
            .prototype_mut(p_act)
            .unwrap()
            .template
            .production_condition = "ScrTrueAll".to_owned();
        let reg = script_template_registry(&mut f.sim.rules.name_keys);
        if let Some(side) = f.scripts.engine.sides[1].as_mut() {
            if let Some(sc) = side
                .scripts
                .iter_mut()
                .find(|s| s.script_name == "ScrTrueAll")
            {
                sc.actions.push(ScriptAction::new(at::NO_OP, &reg));
            }
        }
        let t_exec = create_team(&mut f, "TeamTestPoss", false);
        let _u_idle = spawn(&mut f, "UnitA", t_exec, Coord3D::new(30.0, 30.0, 0.0));
        let u_busy = spawn(&mut f, "UnitA", t_exec, Coord3D::new(35.0, 35.0, 0.0));
        f.sim.ai_move_to_position(
            u_busy,
            &Coord3D::new(220.0, 220.0, 0.0),
            CommandSourceType::FromAi,
        );
        let mut tq_exec = make_tiq(&f.teams, t_exec);
        tq_exec.frame_started = 1;

        {
            let ai = f.sim.world.ai_players.get_mut(&1).unwrap();
            ai.team_ready_queue.push(tq_noobj);
            ai.team_ready_queue.push(tq_mv);
            ai.team_ready_queue.push(tq_exec);
        }
        f.sim.world.frame = 10;
        with_cx(&mut f, |cx| check_ready_teams(cx, 1));
        // Only t_r_noobj and t_exec popped; t_moving stays until timeout!
        {
            let ai = f.sim.world.ai_players.get(&1).unwrap();
            assert_eq!(ai.team_ready_queue.len(), 1);
            assert_eq!(ai.team_ready_queue[0].team_id, t_moving);
        }
        f.sim.world.frame = 60 * 30 + 5;
        with_cx(&mut f, |cx| check_ready_teams(cx, 1));
        assert!(f
            .sim
            .world
            .ai_players
            .get(&1)
            .unwrap()
            .team_ready_queue
            .is_empty());
        assert!(f.teams.team(t_moving).unwrap().active);
    }

    // 3. on_unit_produced (skip wrong/completed orders, GlaWorker supply_truck!=dozer timer, repair_dozer),
    //    queue_supply_truck priority_build & supply_center_has_supplies (mine, dead, desired==current),
    //    on_structure_produced script + rebuild hole, dozer_state, team helpers.
    {
        let mut f = setup_world(true);
        let extra_ini = concat!(
            "Object GlaWorker\n",
            "  Side = America\n",
            "  KindOf = PRELOAD SELECTABLE VEHICLE DOZER HARVESTER\n",
            "  BuildCost = 600\n  BuildTime = 1.0\n",
            "  Geometry = CYLINDER\n  GeometryMajorRadius = 6\n  GeometryHeight = 8\n",
            "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 150\n    InitialHealth = 150\n  End\n",
            "  Behavior = WorkerAIUpdate ModuleTag_AI\n",
            "    MaxBoxes = 4\n    RepairHealthPercentPerSecond = 2%\n    BoredTime = 5000\n    BoredRange = 150\n",
            "  End\n",
            "  Locomotor = SET_NORMAL AptGround\n",
            "End\n",
            "Object WarehouseMine\n",
            "  Side = America\n  KindOf = PRELOAD STRUCTURE SUPPLY_SOURCE MINE\n",
            "  Geometry = BOX\n  GeometryMajorRadius = 20\n  GeometryMinorRadius = 20\n  GeometryHeight = 15\n",
            "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 1000\n  End\n",
            "End\n",
        );
        let mut reader = IniReader::new("extra.ini", extra_ini.as_bytes(), IniLoadType::Overwrite);
        f.sim
            .rules
            .load_thing_templates_from_ini(&mut reader)
            .unwrap();
        f.sim.rules.thing_factory.post_process_load();

        let p0_def = f.sim.world.players.player(0).unwrap().default_team.unwrap();
        let p1_def = f.sim.world.players.player(1).unwrap().default_team.unwrap();
        f.sim.world.players.player_mut(1).unwrap().can_build_units = true;
        set_money(&mut f, 1, 5000);

        let fa = spawn(&mut f, "FactA", p1_def, Coord3D::new(100.0, 100.0, 0.0));
        let fa_id = f.sim.world.objects.id_of(fa).unwrap();
        let sc = spawn(&mut f, "SupplyCtr", p1_def, Coord3D::new(200.0, 100.0, 0.0));
        let sc_id = f.sim.world.objects.id_of(sc).unwrap();
        let mine_wh = spawn(
            &mut f,
            "WarehouseMine",
            p0_def,
            Coord3D::new(210.0, 100.0, 0.0),
        );
        let dead_wh = spawn(&mut f, "Warehouse", p0_def, Coord3D::new(220.0, 100.0, 0.0));
        f.sim
            .world
            .objects
            .get_mut(dead_wh)
            .unwrap()
            .set_effectively_dead(TRUE);
        let _ = mine_wh;

        let mut bl_a = BuildListInfo::new();
        bl_a.template_name = "FactA".to_owned();
        bl_a.object_id = fa_id;
        let mut bl_sc = BuildListInfo::new();
        bl_sc.template_name = "SupplyCtr".to_owned();
        bl_sc.object_id = sc_id;
        bl_sc.is_supply_building = TRUE;
        bl_sc.current_gatherers = 1;
        bl_sc.desired_gatherers = 1;
        bl_sc.script = "ObjInitScript".to_owned();
        f.sim.world.players.player_mut(1).unwrap().build_list = vec![bl_a, bl_sc];

        // desired == current -> does not queue; nor does empty/mine/dead warehouse trigger dock.
        with_cx(&mut f, |cx| queue_supply_truck(cx, 1));
        assert!(f
            .sim
            .world
            .ai_players
            .get(&1)
            .unwrap()
            .team_build_queue
            .is_empty());

        // Now add real Warehouse and set desired=2 -> queues GlaWorker with priority_build=true.
        let _real_wh = spawn(&mut f, "Warehouse", p0_def, Coord3D::new(230.0, 100.0, 0.0));
        f.sim.world.players.player_mut(1).unwrap().build_list[1].desired_gatherers = 2;
        with_cx(&mut f, |cx| queue_supply_truck(cx, 1));
        assert!(f.sim.world.ai_players.get(&1).unwrap().team_build_queue[0].priority_build);
        f.sim
            .world
            .ai_players
            .get_mut(&1)
            .unwrap()
            .clear_teams_in_queue(&mut f.teams);
        f.sim.with_production(fa, |pu, sim, ctx| {
            pu.cancel_and_refund_all_production(sim, ctx)
        });

        // on_unit_produced: skip pre-completed UnitB order and mismatched UnitA order;
        // GlaWorker (DOZER + HARVESTER) keeps structure_timer=99; DozerA clears dozer_queued_for_repair.
        let t_a = f.sim.rules.thing_factory.find_template_id("UnitA").unwrap();
        let t_b = f.sim.rules.thing_factory.find_template_id("UnitB").unwrap();
        let t_w = f
            .sim
            .rules
            .thing_factory
            .find_template_id("GlaWorker")
            .unwrap();
        let t_d = f
            .sim
            .rules
            .thing_factory
            .find_template_id("DozerA")
            .unwrap();
        let worker = spawn(&mut f, "GlaWorker", p1_def, Coord3D::new(105.0, 100.0, 0.0));
        let dozer = spawn(&mut f, "DozerA", p1_def, Coord3D::new(110.0, 100.0, 0.0));
        let t_team = create_team(&mut f, "TeamAlpha", false);
        let mut tq = make_tiq(&f.teams, t_team);
        let mut wo_done = WorkOrder::new();
        wo_done.thing = Some(t_w);
        wo_done.factory_id = fa_id;
        wo_done.num_completed = 1;
        wo_done.num_required = 1;
        let mut wo_other = WorkOrder::new();
        wo_other.thing = Some(t_a);
        wo_other.factory_id = fa_id;
        wo_other.num_completed = 0;
        wo_other.num_required = 1;
        let mut wo_w = WorkOrder::new();
        wo_w.thing = Some(t_w);
        wo_w.factory_id = fa_id;
        wo_w.num_required = 1;
        wo_w.is_resource_gatherer = true;
        let mut wo_d = WorkOrder::new();
        wo_d.thing = Some(t_d);
        wo_d.factory_id = fa_id;
        wo_d.num_required = 1;
        tq.work_orders = vec![wo_done, wo_other, wo_w, wo_d];
        {
            let ai = f.sim.world.ai_players.get_mut(&1).unwrap();
            ai.team_build_queue.push(tq);
            ai.structure_timer = 99;
            ai.dozer_queued_for_repair = TRUE;
        }
        f.sim.on_unit_created(fa, worker);
        assert_eq!(f.sim.world.ai_players.get(&1).unwrap().structure_timer, 99);
        f.sim.on_unit_created(fa, dozer);
        {
            let ai = f.sim.world.ai_players.get(&1).unwrap();
            assert_eq!(ai.structure_timer, 99);
            assert_eq!(ai.repair_dozer, f.sim.world.objects.id_of(dozer).unwrap());
            assert_eq!(ai.dozer_queued_for_repair, FALSE);
            assert_eq!(ai.team_build_queue[0].work_orders[2].num_completed, 1);
            assert_eq!(ai.team_build_queue[0].work_orders[3].num_completed, 1);
        }

        // on_structure_produced runs non-empty info.script, executing its script action.
        let reg = script_template_registry(&mut f.sim.rules.name_keys);
        let mut obj_scr = make_cond_script(
            &reg,
            "ObjInitScript",
            ct::CONDITION_TRUE,
            true,
            true,
            true,
            0,
        );
        obj_scr.is_subroutine = TRUE;
        let mut act_flag = ScriptAction::new(at::SET_FLAG, &reg);
        let mut p_flag = Parameter::new(ParameterType::Flag, 0);
        p_flag.string_val = "obj_script_ran".to_owned();
        let p_bool = Parameter::new(ParameterType::Boolean, 1);
        act_flag.parms = vec![p_flag, p_bool];
        obj_scr.actions.push(act_flag);
        f.scripts.engine.sides[1]
            .as_mut()
            .unwrap()
            .add_script(obj_scr, 5);
        with_cx(&mut f, |cx| on_structure_produced(cx, 1, sc));
        assert!(f
            .scripts
            .engine
            .flags
            .iter()
            .any(|fl| fl.name == "obj_script_ran" && fl.value));

        // dozer_state: force_pending=TRUE treats GlaWorker as busy so ai_player_update skips building.
        let worker_ai = f.sim.world.objects.get(worker).unwrap().ai_slot.unwrap();
        let _ = f.sim.dispatch_module(worker_ai, |m, _, _| {
            if let Some(ai) = m.as_any_mut().and_then(|a| a.downcast_mut::<AiUpdate>()) {
                if let DerivedAi::Worker(ref mut w) = ai.derived {
                    w.force_pending = TRUE;
                }
            }
        });
        f.sim.world.players.player_mut(1).unwrap().can_build_base = false;
        let t_test = create_team(&mut f, "TeamLow", true);
        let list = f.sim.world.players.clone();
        sync_team_to_sim_world(&mut f.sim.world, &list, &f.teams, t_test);
        assert!(!f
            .sim
            .world
            .team_is_neutral
            .get(&t_test)
            .is_some_and(|&v| v != FALSE));

        // count_objects_by_thing_template ignore_dead & transfer_units_to same-team no-op.
        let dead_u = spawn(&mut f, "UnitB", t_team, Coord3D::new(40.0, 40.0, 0.0));
        f.sim
            .world
            .objects
            .get_mut(dead_u)
            .unwrap()
            .set_effectively_dead(TRUE);
        assert_eq!(
            count_objects_by_thing_template(&f.sim, t_team, &[t_b], true, false),
            vec![0]
        );
        assert_eq!(
            count_objects_by_thing_template(&f.sim, t_team, &[t_b], false, false),
            vec![1]
        );
        transfer_units_to(&mut f.sim, &mut f.teams, t_team, t_team);
        assert_eq!(f.sim.world.objects.get(dead_u).unwrap().team_id, t_team);

        // Drain pending supply truck dock for GlaWorker with empty queue keeps unported clean except runObjectScript.
        with_cx(&mut f, |cx| drain_pending_sim_events_for_player(cx, 1));
        with_cx(&mut f, |cx| ai_player_update(cx, 1));
    }
}
