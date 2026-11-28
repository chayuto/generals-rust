//! 3-way C++ differential verification and unit tests for remaining `AIPlayer`
//! (`PKT-426`) and `AISkirmishPlayer` (`PKT-427`) methods, `Pathfinder::find_broken_bridge`,
//! and all newly wired AI script actions & conditions.

use std::collections::BTreeMap;

use zh_common::core::dict::Dict;
use zh_common::ini::IniLoadType;
use zh_common::math::{Coord2D, Coord3D, Matrix3D, Region3D};
use zh_common::types::{
    Bool, Int, ObjectId, Real, Relationship, TeamId, UnsignedInt, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

use zh_logic::ai::ai_player::{
    update as ai_player_update, AiPlayer, DIFFICULTY_NORMAL,
};
use zh_logic::ai::ai_player_ops::{
    build_ai_base_defense, build_ai_base_defense_structure,
    build_specific_building_nearest_team, build_upgrade, calc_closest_construction_zone_location,
    check_bridges, compute_superweapon_target, compute_superweapon_target_skirmish,
    get_player_structure_bounds, get_player_superweapon_value, guard_supply_center,
    is_supply_source_attacked, is_supply_source_safe, repair_structure, select_skillset,
};
use zh_logic::arena::ObjectSlot;
use zh_logic::combat::damage::DamageInfo;
use zh_logic::economy::money::NoopMoneyCallbacks;
use zh_logic::gamelogic::game_engine::{GameDataLoader, StartupStores};
use zh_logic::gamelogic::global_data::PlayerType;
use zh_logic::modules::module::CommandSourceType;
use zh_logic::modules::update::ai_update::AiUpdate;
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;
use zh_logic::pathfinder::{NoPathWorld, Pathfinder};
use zh_logic::player::build_assistant::BuildContext;
use zh_logic::player::player_list::PlayerList;
use zh_logic::player::seams::{
    AiKind, PlayerEnv, PlayerIndex, PlayerPart, PlayerWorld, SkirmishScripts,
};
use zh_logic::player::team::TeamFactory;
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::rules::science::ScienceType;
use zh_logic::rules::upgrade::UpgradeMaskType;
use zh_logic::scripts::script_types::{action_type as at, condition_type as ct};
use zh_logic::scripts::{
    script_template_registry, Condition, GameScripts, ScriptAction, ScriptGame,
};
use zh_logic::sim::{SimApi, SimContext, SimView, SimWorld};
use zh_logic::spatial::sides_list::{BuildListInfo, SidesList};
use zh_logic::terrain::{Bridge, BridgeGeometry, TerrainLogic};

const GOLDEN: &str = include_str!("../../../docs/golden_vectors/AiPlayerOps.json");

fn rows() -> impl Iterator<Item = &'static str> {
    GOLDEN.lines().filter_map(|l| {
        l.trim()
            .trim_end_matches(',')
            .strip_prefix('"')?
            .strip_suffix('"')
    })
}

fn fx(v: Real) -> String {
    format!("{:08x}", v.to_bits())
}

fn unfx(s: &str) -> Real {
    f32::from_bits(u32::from_str_radix(s, 16).expect("f32 hex"))
}

fn ops_ini() -> &'static str {
    concat!(
        "Locomotor AptGround\n",
        "  Surfaces = GROUND\n",
        "  Appearance = FOUR_WHEELS\n",
        "  Speed = 30\n  TurnRate = 180\n  Acceleration = 60\n  Braking = 60\n  MinSpeed = 0\n",
        "End\n",
        "Upgrade UpgPlayerAlpha\n",
        "  Type = PLAYER\n  BuildCost = 500\n  BuildTime = 5.0\n",
        "End\n",
        "Upgrade UpgObjBeta\n",
        "  Type = OBJECT\n  BuildCost = 200\n  BuildTime = 2.0\n",
        "End\n",
        "SpecialPower SpNeutron\n",
        "  Enum = SPECIAL_NEUTRON_MISSILE\n",
        "  ReloadTime = 60000\n",
        "End\n",
        "SpecialPower SpSneak\n",
        "  Enum = SPECIAL_SNEAK_ATTACK\n",
        "  ReloadTime = 60000\n",
        "End\n",
        "SpecialPower SpClusterMines\n",
        "  Enum = SPECIAL_CLUSTER_MINES\n",
        "  ReloadTime = 60000\n",
        "End\n",
        "SpecialPower SpNukeMines\n",
        "  Enum = NUKE_SPECIAL_CLUSTER_MINES\n",
        "  ReloadTime = 60000\n",
        "End\n",
        "CommandButton Btn_UpgAlpha\n",
        "  Command = PLAYER_UPGRADE\n  Upgrade = UpgPlayerAlpha\n",
        "End\n",
        "CommandButton Btn_UnitA\n",
        "  Command = UNIT_BUILD\n  Object = UnitA\n",
        "End\n",
        "CommandSet CmdWithUpgrade\n",
        "  1 = Btn_UnitA\n  2 = Btn_UpgAlpha\n",
        "End\n",
        "CommandSet CmdNoUpgrade\n",
        "  1 = Btn_UnitA\n",
        "End\n",
        "PlayerTemplate FactionCivilian\n  Side = Civilian\n  PreferredColor = R:100 G:100 B:100\nEnd\n",
        "PlayerTemplate FactionAmerica\n  Side = America\n  PreferredColor = R:10 G:20 B:255\nEnd\n",
        "Object UnitA\n",
        "  Side = America\n  KindOf = PRELOAD SELECTABLE VEHICLE\n",
        "  BuildCost = 800\n  BuildTime = 1.0\n",
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 6\n  GeometryHeight = 8\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n",
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n",
        "  Locomotor = SET_NORMAL AptGround\n",
        "End\n",
        "Object InfA\n",
        "  Side = America\n  KindOf = PRELOAD SELECTABLE INFANTRY\n",
        "  BuildCost = 300\n  BuildTime = 0.5\n",
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 4\n  GeometryHeight = 6\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n",
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n",
        "  Locomotor = SET_NORMAL AptGround\n",
        "End\n",
        "Object AirA\n",
        "  Side = America\n  KindOf = PRELOAD SELECTABLE VEHICLE AIRCRAFT\n",
        "  BuildCost = 1500\n  BuildTime = 1.0\n",
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 8\n  GeometryHeight = 8\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n",
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n",
        "  Locomotor = SET_NORMAL AptGround\n",
        "End\n",
        "Object DozerA\n",
        "  Side = America\n  KindOf = PRELOAD SELECTABLE VEHICLE DOZER\n",
        "  BuildCost = 1000\n  BuildTime = 1.0\n",
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 7\n  GeometryHeight = 8\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n",
        "  Behavior = DozerAIUpdate ModuleTag_AI\n    RepairHealthPercentPerSecond = 50%\n    BoredTime = 5000\n    BoredRange = 150\n  End\n",
        "  Locomotor = SET_NORMAL AptGround\n",
        "End\n",
        "Object HarvA\n",
        "  Side = America\n  KindOf = PRELOAD SELECTABLE VEHICLE HARVESTER\n",
        "  BuildCost = 600\n  BuildTime = 1.0\n",
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 6\n  GeometryHeight = 8\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n",
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n",
        "  Locomotor = SET_NORMAL AptGround\n",
        "End\n",
        "Object CmdCenter\n",
        "  Side = America\n  KindOf = PRELOAD STRUCTURE COMMANDCENTER\n",
        "  BuildCost = 2000\n  BuildTime = 1.0\n",
        "  Geometry = BOX\n  GeometryMajorRadius = 25\n  GeometryMinorRadius = 25\n  GeometryHeight = 20\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 2000\n    InitialHealth = 2000\n  End\n",
        "End\n",
        "Object Superwep\n",
        "  Side = America\n  KindOf = PRELOAD STRUCTURE FS_SUPERWEAPON\n",
        "  BuildCost = 5000\n  BuildTime = 1.0\n",
        "  Geometry = BOX\n  GeometryMajorRadius = 25\n  GeometryMinorRadius = 25\n  GeometryHeight = 20\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 2000\n    InitialHealth = 2000\n  End\n",
        "End\n",
        "Object BaseDef\n",
        "  Side = America\n  KindOf = PRELOAD STRUCTURE FS_BASE_DEFENSE\n",
        "  BuildCost = 900\n  BuildTime = 1.0\n",
        "  Geometry = BOX\n  GeometryMajorRadius = 12\n  GeometryMinorRadius = 12\n  GeometryHeight = 15\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 600\n    InitialHealth = 600\n  End\n",
        "End\n",
        "Object TechDef\n",
        "  Side = America\n  KindOf = PRELOAD STRUCTURE TECH_BASE_DEFENSE\n",
        "  BuildCost = 700\n  BuildTime = 1.0\n",
        "  Geometry = BOX\n  GeometryMajorRadius = 12\n  GeometryMinorRadius = 12\n  GeometryHeight = 15\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 600\n    InitialHealth = 600\n  End\n",
        "End\n",
        "Object ConservBldg\n",
        "  Side = America\n  KindOf = PRELOAD STRUCTURE CONSERVATIVE_BUILDING\n",
        "  BuildCost = 400\n  BuildTime = 1.0\n",
        "  Geometry = BOX\n  GeometryMajorRadius = 15\n  GeometryMinorRadius = 15\n  GeometryHeight = 15\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 600\n    InitialHealth = 600\n  End\n",
        "End\n",
        "Object CashGen\n",
        "  Side = America\n  KindOf = PRELOAD STRUCTURE CASH_GENERATOR FS_SUPPLY_CENTER\n",
        "  BuildCost = 1500\n  BuildTime = 1.0\n",
        "  Geometry = BOX\n  GeometryMajorRadius = 20\n  GeometryMinorRadius = 20\n  GeometryHeight = 15\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 1000\n  End\n",
        "End\n",
        "Object Warehouse\n",
        "  Side = America\n  KindOf = PRELOAD STRUCTURE SUPPLY_SOURCE\n",
        "  Geometry = BOX\n  GeometryMajorRadius = 20\n  GeometryMinorRadius = 20\n  GeometryHeight = 15\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 1000\n  End\n",
        "  Behavior = SupplyWarehouseDockUpdate ModuleTag_Dock\n    NumberApproachPositions = 2\n    StartingBoxes = 20\n  End\n",
        "End\n",
        "Object FactUp\n",
        "  Side = America\n  KindOf = PRELOAD STRUCTURE FS_FACTORY FS_WARFACTORY\n",
        "  CommandSet = CmdWithUpgrade\n",
        "  BuildCost = 2000\n  BuildTime = 1.0\n",
        "  Geometry = BOX\n  GeometryMajorRadius = 20\n  GeometryMinorRadius = 20\n  GeometryHeight = 20\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 1000\n  End\n",
        "  Behavior = ProductionUpdate ModuleTag_Prod\n  End\n",
        "End\n",
        "Object FactPlain\n",
        "  Side = America\n  KindOf = PRELOAD STRUCTURE FS_FACTORY FS_WARFACTORY\n",
        "  CommandSet = CmdNoUpgrade\n",
        "  BuildCost = 1500\n  BuildTime = 1.0\n",
        "  Geometry = BOX\n  GeometryMajorRadius = 20\n  GeometryMinorRadius = 20\n  GeometryHeight = 20\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 1000\n  End\n",
        "  Behavior = ProductionUpdate ModuleTag_Prod\n  End\n",
        "End\n",
        "Object RebuildHole\n",
        "  Side = America\n  KindOf = PRELOAD STRUCTURE REBUILD_HOLE\n",
        "  Geometry = BOX\n  GeometryMajorRadius = 15\n  GeometryMinorRadius = 15\n  GeometryHeight = 5\n",
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n",
        "  Behavior = RebuildHoleBehavior ModuleTag_Hole\n    WorkerRespawnDelay = 5000\n  End\n",
        "End\n",
    )
}

struct OracleTerrain(TerrainLogic);

impl ThingTerrainAccess for OracleTerrain {
    fn terrain_logic(&self) -> Option<&TerrainLogic> {
        Some(&self.0)
    }
    fn terrain_logic_mut(&mut self) -> Option<&mut TerrainLogic> {
        Some(&mut self.0)
    }
    fn get_ground_height(&self, _x: Real, _y: Real, normal: Option<&mut Coord3D>) -> Real {
        if let Some(n) = normal {
            *n = Coord3D::new(0.0, 0.0, 1.0);
        }
        0.0
    }
    fn align_on_terrain(
        &self,
        _angle: Real,
        _pos: &Coord3D,
        _stick_to_ground: Bool,
        _mtx: &mut Matrix3D,
    ) -> PathfindLayerEnum {
        PathfindLayerEnum::Ground
    }
    fn is_underwater(
        &self,
        _x: Real,
        _y: Real,
        water_z: Option<&mut Real>,
        terrain_z: Option<&mut Real>,
    ) -> Bool {
        if let Some(w) = water_z {
            *w = 0.0;
        }
        if let Some(t) = terrain_z {
            *t = 0.0;
        }
        FALSE
    }
    fn get_layer_height(
        &self,
        _x: Real,
        _y: Real,
        _layer: PathfindLayerEnum,
        normal: Option<&mut Coord3D>,
        _clip: Bool,
    ) -> Real {
        if let Some(n) = normal {
            *n = Coord3D::new(0.0, 0.0, 1.0);
        }
        0.0
    }
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
    fn ai_pre_team_destroy(&mut self, _player: PlayerIndex, _team: TeamId) {}
    fn qualify_skirmish_scripts(&mut self, _sides: &mut SidesList, _copy: &SkirmishScripts<'_>) {}
    fn create_economy_trackers(&mut self, _player: PlayerIndex) {}
    fn delete_economy_trackers(&mut self, _player: PlayerIndex) {}
    fn academy_init(&mut self, _player: PlayerIndex) {}
    fn academy_update(&mut self, _player: PlayerIndex) {}
    fn academy_record_income(&mut self, _player: PlayerIndex) {}
    fn power_brownout_changed(&mut self, _player: PlayerIndex, _brownout: bool) {}
    fn science_added(&mut self, _player: PlayerIndex, _science: ScienceType) {}
    fn score_template(&self, _name: &str) -> Option<zh_logic::economy::score_keeper::ScoreTemplateDesc> {
        None
    }
    fn upgrade_masks(&self) -> Vec<(String, UpgradeMaskType)> {
        Vec::new()
    }
    fn notify_team_created(&mut self, players: &PlayerList, teams: &TeamFactory, team: TeamId) {
        let Some(owner) = teams.controlling_player(team) else {
            return;
        };
        if players.player(owner).is_none() {
            return;
        }
        self.world.team_players.insert(team, owner);
        self.world
            .team_is_neutral
            .insert(team, Bool::from(owner == 0));
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

fn sync_all(sim: &mut SimContext, teams: &TeamFactory) {
    let players = &sim.world.players;
    if let Some(team) = players.player(0).and_then(|p| p.default_team) {
        sim.world.neutral_team_id = team;
    }
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
    setup_world_with_extra(skirmish, "")
}

fn setup_world_with_extra(skirmish: bool, extra_ini: &str) -> Fixture {
    let mut loader = GameDataLoader::new();
    let full_ini = format!("{}{}", ops_ini(), extra_ini);
    loader.load_bytes("ops.ini", full_ini.as_bytes(), IniLoadType::Overwrite);
    assert!(
        loader.report.failures.is_empty(),
        "{:?}",
        loader.report.failures
    );
    loader.rules.thing_factory.post_process_load();
    loader.rules.global_data.gravity = -1.0;

    let mut terrain = TerrainLogic::new();
    terrain.set_synthetic_bounds(100, 100, 100.0);
    let extent = terrain.extent();
    let mut pf = Pathfinder::new(0.0);
    pf.new_map(&terrain, &extent, 0.0, &[], &NoPathWorld, 0);

    let mut sim = SimContext {
        rules: loader.rules,
        ..SimContext::new()
    };
    sim.terrain = Box::new(OracleTerrain(terrain));
    sim.init_partition(20.0, extent);
    sim.world.pathfinder = Some(Box::new(pf));
    sim.world.frame = 1;
    let stores = loader.stores;

    let mut sides = SidesList::default();
    let mut k = |s: &str| sim.rules.name_keys.name_to_key(s);
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
    {
        let mut td = Dict::default();
        td.set_ascii_string(k("teamName"), "GuardTeam");
        td.set_ascii_string(k("teamOwner"), "AiPlayer1");
        td.set_bool(k("teamIsSingleton"), TRUE);
        sides.add_team(Some(&td));
    }

    let mut teams = TeamFactory::new();
    let team_seconds = sim.rules.ai_data.team_seconds as Int;
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
    sim.world.players
        .player_mut(1)
        .unwrap()
        .set_player_relationship(2, Relationship::Enemies);
    sim.world.players
        .player_mut(2)
        .unwrap()
        .set_player_relationship(1, Relationship::Enemies);
    sim.world.set_player_relation(1, 2, Relationship::Enemies);
    sim.world.set_player_relation(2, 1, Relationship::Enemies);
    sync_all(&mut sim, &teams);

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
        vec![None, None, None],
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

fn clear_all_objects(f: &mut Fixture) {
    let slots: Vec<ObjectSlot> = f.sim.world.objects.world_iter().collect();
    for s in slots {
        f.sim.destroy_object_now(s);
    }
    f.sim.world.team_members.clear();
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

#[test]
fn matches_cpp_oracle_rows() {
    let all_rows: Vec<&str> = rows().collect();
    assert_eq!(all_rows.len(), 143);

    let mut f = setup_world(true);
    let mut t2 = f.sim.world.players.player(2).unwrap().default_team.unwrap();
    let mut idx = 0;

    let init_skirmish_p1 = |f: &mut Fixture| {
        let ai = f.sim.world.ai_players.get_mut(&1).unwrap();
        ai.base_center = Coord3D::new(200.0, 200.0, 0.0);
        ai.base_center_set = TRUE;
        ai.base_radius = 120.0;
        let sk = ai.skirmish.as_mut().unwrap();
        sk.current_enemy = Some(2);
        sk.frame_to_check_enemy = 999_999;
    };
    init_skirmish_p1(&mut f);

    // 1) Superweapon & bounds
    for s in 0..16 {
        let b_parts: Vec<&str> = all_rows[idx].split_whitespace().collect();
        assert_eq!(b_parts[0], "BOUNDS");
        assert_eq!(b_parts[1].parse::<usize>().unwrap(), s);
        idx += 1;

        clear_all_objects(&mut f);
        f.sim.world.rng.init_random_with_seed(1000 + (s as UnsignedInt) * 17);
        if b_parts[2] != "-" {
            for spec in b_parts[2].split(';') {
                let p: Vec<&str> = spec.split(',').collect();
                spawn(
                    &mut f,
                    p[0],
                    t2,
                    Coord3D::new(unfx(p[1]), unfx(p[2]), unfx(p[3])),
                );
            }
        }
        let b0 = get_player_structure_bounds(&f.sim, 2, false);
        let b1 = get_player_structure_bounds(&f.sim, 2, true);
        assert_eq!(
            (fx(b0.lo.x), fx(b0.lo.y), fx(b0.hi.x), fx(b0.hi.y)),
            (
                b_parts[3].to_owned(),
                b_parts[4].to_owned(),
                b_parts[5].to_owned(),
                b_parts[6].to_owned()
            ),
            "BOUNDS0 s={s}"
        );
        assert_eq!(
            (fx(b1.lo.x), fx(b1.lo.y), fx(b1.hi.x), fx(b1.hi.y)),
            (
                b_parts[7].to_owned(),
                b_parts[8].to_owned(),
                b_parts[9].to_owned(),
                b_parts[10].to_owned()
            ),
            "BOUNDS1 s={s}"
        );

        for q in 0..2 {
            let parts: Vec<&str> = all_rows[idx].split_whitespace().collect();
            assert_eq!(parts[0], "SWVAL");
            idx += 1;
            let c = Coord3D::new(unfx(parts[2]), unfx(parts[3]), 0.0);
            let rad = unfx(parts[4]);
            let v0 = get_player_superweapon_value(&f.sim, &c, 2, rad, false);
            let v1 = get_player_superweapon_value(&f.sim, &c, 2, rad, true);
            assert_eq!(v0, parts[5].parse::<Int>().unwrap(), "SWVAL0 s={s} q={q}");
            assert_eq!(v1, parts[6].parse::<Int>().unwrap(), "SWVAL1 s={s} q={q}");
        }

        for p in 0..2 {
            let parts: Vec<&str> = all_rows[idx].split_whitespace().collect();
            assert_eq!(parts[0], "SWBASE");
            idx += 1;
            let sp_name = if p == 0 { "SpNeutron" } else { "SpSneak" };
            let sp = f
                .sim
                .rules
                .special_power_store
                .find_special_power_template(sp_name)
                .unwrap()
                .clone();
            let wr = unfx(parts[3]);
            let ret = compute_superweapon_target(&mut f.sim, &sp, 2, wr);
            let crc = f.sim.world.rng.get_game_logic_random_seed_crc();
            assert_eq!(ret.is_some() as Int, parts[4].parse::<Int>().unwrap(), "SWBASE ok s={s} p={p}");
            let ret = ret.unwrap_or_default();
            assert_eq!(
                (fx(ret.x), fx(ret.y), fx(ret.z), crc),
                (
                    parts[5].to_owned(),
                    parts[6].to_owned(),
                    parts[7].to_owned(),
                    parts[8].parse::<UnsignedInt>().unwrap()
                ),
                "SWBASE s={s} p={p}"
            );
        }

        {
            let tl = f.sim.terrain.terrain_logic_mut().unwrap();
            tl.clear_waypoints_for_test();
            if s % 2 == 1 {
                tl.push_waypoint_for_test(
                    1,
                    "W1",
                    Coord3D::new(400.0, 200.0, 0.0),
                    ["Center1".into(), String::new(), String::new()],
                );
                tl.push_waypoint_for_test(
                    2,
                    "W2",
                    Coord3D::new(200.0, 450.0, 0.0),
                    ["Flank1".into(), String::new(), String::new()],
                );
                tl.push_waypoint_for_test(
                    3,
                    "W3",
                    Coord3D::new(50.0, 50.0, 0.0),
                    ["Backdoor1".into(), String::new(), String::new()],
                );
            }
        }
        for k in 0..2 {
            let parts: Vec<&str> = all_rows[idx].split_whitespace().collect();
            assert_eq!(parts[0], "SWSKIRMISH");
            idx += 1;
            let sp_name = if k == 0 { "SpClusterMines" } else { "SpNukeMines" };
            let sp = f
                .sim
                .rules
                .special_power_store
                .find_special_power_template(sp_name)
                .unwrap()
                .clone();
            let ai = f.sim.world.ai_players.get(&1).unwrap().clone();
            let p1 = f.sim.world.players.player(1).unwrap().clone();
            let ret = compute_superweapon_target_skirmish(
                &mut f.sim,
                &ai,
                &p1,
                &sp,
                2,
                75.0,
            );
            let crc = f.sim.world.rng.get_game_logic_random_seed_crc();
            assert_eq!(ret.is_some() as Int, parts[4].parse::<Int>().unwrap());
            let ret = ret.unwrap_or_default();
            assert_eq!(
                (fx(ret.x), fx(ret.y), fx(ret.z), crc),
                (
                    parts[5].to_owned(),
                    parts[6].to_owned(),
                    parts[7].to_owned(),
                    parts[8].parse::<UnsignedInt>().unwrap()
                ),
                "SWSKIRMISH s={s} k={k}"
            );
        }
    }

    // 2) Base defense placement
    f = setup_world(true);
    {
        let ai = f.sim.world.ai_players.get_mut(&1).unwrap();
        ai.base_center = Coord3D::new(500.0, 500.0, 0.0);
        ai.base_center_set = TRUE;
        ai.base_radius = 100.0;
        let sk = ai.skirmish.as_mut().unwrap();
        sk.current_enemy = Some(2);
        sk.frame_to_check_enemy = 999_999;
    }
    {
        let tl = f.sim.terrain.terrain_logic_mut().unwrap();
        tl.push_waypoint_for_test(1, "C", Coord3D::new(700.0, 500.0, 0.0), ["Center1".into(), String::new(), String::new()]);
        tl.push_waypoint_for_test(2, "F", Coord3D::new(500.0, 700.0, 0.0), ["Flank1".into(), String::new(), String::new()]);
        tl.push_waypoint_for_test(3, "B", Coord3D::new(300.0, 500.0, 0.0), ["Backdoor1".into(), String::new(), String::new()]);
    }
    let t1 = f.sim.world.players.player(1).unwrap().default_team.unwrap();
    spawn(&mut f, "BaseDef", t1, Coord3D::new(600.0, 500.0, 0.0));
    let extent = f.sim.terrain.terrain_logic().unwrap().maximum_pathfind_extent();
    for step in 0..10 {
        let parts: Vec<&str> = all_rows[idx].split_whitespace().collect();
        assert_eq!(parts[0], "BASEDEF");
        idx += 1;
        let flank = step >= 4;
        let mut ai = f.sim.world.ai_players.remove(&1).unwrap();
        build_ai_base_defense_structure(&mut f.sim, &mut ai, 1, "BaseDef", flank, &extent);
        let sk = ai.skirmish.as_ref().unwrap();
        let head = f.sim.world.players.player(1).unwrap().build_list.first().map(|h| {
            format!("{},{},{},{}", h.template_name, fx(h.location.x), fx(h.location.y), fx(h.angle))
        }).unwrap_or_else(|| "-".into());
        assert_eq!(
            (
                sk.cur_front_base_defense,
                sk.cur_flank_base_defense,
                fx(sk.cur_front_left_defense_angle),
                fx(sk.cur_front_right_defense_angle),
                fx(sk.cur_left_flank_left_defense_angle),
                fx(sk.cur_left_flank_right_defense_angle),
                fx(sk.cur_right_flank_left_defense_angle),
                fx(sk.cur_right_flank_right_defense_angle),
                f.sim.world.players.player(1).unwrap().build_list.len(),
                head
            ),
            (
                parts[3].parse::<Int>().unwrap(),
                parts[4].parse::<Int>().unwrap(),
                parts[5].to_owned(),
                parts[6].to_owned(),
                parts[7].to_owned(),
                parts[8].to_owned(),
                parts[9].to_owned(),
                parts[10].to_owned(),
                parts[11].parse::<usize>().unwrap(),
                parts[12].to_owned()
            ),
            "BASEDEF step={step}"
        );
        f.sim.world.ai_players.insert(1, ai);
    }

    // 3) CCZONE & NEARTEAM
    for skirmish in 0..2 {
        f = setup_world(skirmish != 0);
        f.sim.world.ai_players.get_mut(&1).unwrap().base_center = Coord3D::new(200.0, 200.0, 0.0);
        f.sim.world.ai_players.get_mut(&1).unwrap().base_center_set = TRUE;
        let extent = f.sim.terrain.terrain_logic().unwrap().maximum_pathfind_extent();

        let p0: Vec<&str> = all_rows[idx].split_whitespace().collect();
        assert_eq!(p0[0], "CCZONE");
        idx += 1;
        let mut loc = Coord3D::new(400.0, 400.0, 0.0);
        let ok0 = calc_closest_construction_zone_location(
            &f.sim,
            f.sim.world.ai_players.get(&1).unwrap(),
            "BaseDef",
            &mut loc,
            &extent,
        );
        assert_eq!((ok0 as Int, fx(loc.x), fx(loc.y), fx(loc.z)),
            (p0[3].parse().unwrap(), p0[4].into(), p0[5].into(), p0[6].into()));

        let t1 = f.sim.world.players.player(1).unwrap().default_team.unwrap();
        spawn(&mut f, "FactPlain", t1, Coord3D::new(400.0, 400.0, 0.0));
        let p1: Vec<&str> = all_rows[idx].split_whitespace().collect();
        assert_eq!(p1[0], "CCZONE");
        idx += 1;
        let mut loc1 = Coord3D::new(400.0, 400.0, 0.0);
        let ok1 = calc_closest_construction_zone_location(
            &f.sim,
            f.sim.world.ai_players.get(&1).unwrap(),
            "BaseDef",
            &mut loc1,
            &extent,
        );
        assert_eq!((ok1 as Int, fx(loc1.x), fx(loc1.y), fx(loc1.z)),
            (p1[3].parse().unwrap(), p1[4].into(), p1[5].into(), p1[6].into()));

        let p2: Vec<&str> = all_rows[idx].split_whitespace().collect();
        assert_eq!(p2[0], "NEARTEAM");
        idx += 1;
        let gt_proto = f
            .teams
            .find_team_prototype("GuardTeam", &mut f.sim.rules.name_keys)
            .unwrap();
        let gt = f.teams.instance_ids(gt_proto)[0];
        spawn(&mut f, "UnitA", gt, Coord3D::new(400.0, 400.0, 0.0));
        let ai = f.sim.world.ai_players.get(&1).unwrap().clone();
        let mut p1 = f.sim.world.players.player(1).unwrap().clone();
        build_specific_building_nearest_team(&f.sim, &ai, &mut p1, "BaseDef", gt, &extent);
        f.sim.world.players.player_mut(1).unwrap().build_list = p1.build_list;
        let head = f.sim.world.players.player(1).unwrap().build_list.first();
        assert_eq!(head.map_or("-", |h| h.template_name.as_str()), p2[2]);
    }

    // 4) Supply ops
    f = setup_world(true);
    init_skirmish_p1(&mut f);
    let extent = f.sim.terrain.terrain_logic().unwrap().maximum_pathfind_extent();
    let t0 = f.sim.world.players.player(0).unwrap().default_team.unwrap();
    let t1 = f.sim.world.players.player(1).unwrap().default_team.unwrap();
    t2 = f.sim.world.players.player(2).unwrap().default_team.unwrap();

    for step in 0..3 {
        let parts: Vec<&str> = all_rows[idx].split_whitespace().collect();
        assert_eq!(parts[0], "SUP_SAFE");
        idx += 1;
        let mut enemy_slot = None;
        if step == 1 {
            spawn(&mut f, "Warehouse", t0, Coord3D::new(300.0, 300.0, 0.0));
        } else if step == 2 {
            enemy_slot = Some(spawn(&mut f, "UnitA", t2, Coord3D::new(310.0, 300.0, 0.0)));
        }
        let mut ai = f.sim.world.ai_players.remove(&1).unwrap();
        let mut p1 = f.sim.world.players.player(1).unwrap().clone();
        let safe = is_supply_source_safe(
            &zh_logic::ai::ai_player::SupplyOrder {
                sim: &f.sim,
                ai: &mut ai,
                player: &mut p1,
                enemy_center: None,
                pathfind_extent: extent,
            },
            100,
        );
        f.sim.world.ai_players.insert(1, ai);
        assert_eq!(safe as Int, parts[2].parse::<Int>().unwrap(), "SUP_SAFE {step}");
        if let Some(es) = enemy_slot {
            f.sim.destroy_object_now(es);
        }
    }

    let harv = spawn(&mut f, "HarvA", t1, Coord3D::new(250.0, 250.0, 0.0));
    let harv_id = f.sim.world.objects.id_of(harv).unwrap();
    let set_harv_dmg = |f: &mut Fixture, frame: UnsignedInt, no_effect: Bool| {
        f.sim.world.players.player_mut(1).unwrap().attacked_frame = frame;
        let bs = f.sim.world.objects.get(harv).unwrap().body_slot.unwrap();
        let mut di = DamageInfo::default();
        di.output.no_effect = no_effect;
        if let Some(m) = f.sim.world.modules.payload_mut(bs) {
            if let Some(ab) = m.as_any_mut().and_then(|a| a.downcast_mut::<zh_logic::modules::body::active_body::ActiveBody>()) {
                ab.last_damage_timestamp = frame;
                ab.last_damage_info = di;
            }
        }
    };

    for step in 0..4 {
        let parts: Vec<&str> = all_rows[idx].split_whitespace().collect();
        assert_eq!(parts[0], "SUP_ATK");
        idx += 1;
        match step {
            0 => f.sim.world.frame = 0,
            1 => f.sim.world.frame = 5,
            2 => {
                f.sim.world.frame = 15;
                set_harv_dmg(&mut f, 14, TRUE);
            }
            _ => {
                f.sim.world.frame = 30;
                set_harv_dmg(&mut f, 28, FALSE);
            }
        }
        let mut ai = f.sim.world.ai_players.remove(&1).unwrap();
        let atk = is_supply_source_attacked(&f.sim, &mut ai);
        assert_eq!(atk as Int, parts[2].parse::<Int>().unwrap(), "SUP_ATK {step}");
        assert_eq!(ai.supply_source_attack_check_frame, parts[3].parse::<UnsignedInt>().unwrap());
        let expected_id = parts[4].parse::<Int>().unwrap();
        assert_eq!(ai.attacked_supply_center.0, if expected_id == 0 { 0 } else { harv_id.0 });
        f.sim.world.ai_players.insert(1, ai);
    }

    // guardSupplyCenter
    let g_parts: Vec<&str> = all_rows[idx].split_whitespace().collect();
    assert_eq!(g_parts[0], "GUARD");
    idx += 1;
    spawn(&mut f, "CmdCenter", t2, Coord3D::new(700.0, 300.0, 0.0));
    let gt_proto = f
        .teams
        .find_team_prototype("GuardTeam", &mut f.sim.rules.name_keys)
        .unwrap();
    let gt = f.teams.instance_ids(gt_proto)[0];
    let g1 = spawn(&mut f, "UnitA", gt, Coord3D::new(100.0, 100.0, 0.0));
    let mut ai = f.sim.world.ai_players.remove(&1).unwrap();
    let mut p1 = f.sim.world.players.player(1).unwrap().clone();
    guard_supply_center(
        &mut f.sim,
        &mut ai,
        &mut p1,
        Some(Coord2D::new(700.0, 300.0)),
        extent,
        gt,
        100,
    );
    f.sim.world.players.player_mut(1).unwrap().build_list = p1.build_list;
    assert_eq!(ai.supply_source_attack_check_frame, g_parts[1].parse::<UnsignedInt>().unwrap());
    assert_eq!(f.sim.world.ai_groups.next_group_id, g_parts[2].parse::<UnsignedInt>().unwrap());
    let g1_ai_slot = f.sim.world.objects.get(g1).unwrap().ai_slot.unwrap();
    let g1_ai = f.sim.world.modules.payload(g1_ai_slot).unwrap().as_any().unwrap().downcast_ref::<AiUpdate>().unwrap();
    assert_eq!(g1_ai.guard_mode, 0);
    assert_eq!(g1_ai.last_command_source as Int, g_parts[3].parse::<Int>().unwrap());
    assert!((g1_ai.location_to_guard.x - 254.7706).abs() < 0.01);
    assert!((g1_ai.location_to_guard.y - 250.5301).abs() < 0.01);
    f.sim.world.ai_players.insert(1, ai);

    // 5) Upgrade, Skillset, Repair, Bridge repair
    f = setup_world(false);
    let t1 = f.sim.world.players.player(1).unwrap().default_team.unwrap();
    let sk_parts: Vec<&str> = all_rows[idx].split_whitespace().collect();
    assert_eq!(sk_parts[0], "SKILLSET");
    idx += 1;
    select_skillset(f.sim.world.ai_players.get_mut(&1).unwrap(), 3);
    assert_eq!(f.sim.world.ai_players.get(&1).unwrap().skillset_selector, sk_parts[1].parse::<Int>().unwrap());

    let up_parts: Vec<&str> = all_rows[idx].split_whitespace().collect();
    assert_eq!(up_parts[0], "UPGRADE");
    idx += 1;
    set_money(&mut f, 1, 1000);
    let f_plain = spawn(&mut f, "FactPlain", t1, Coord3D::new(200.0, 200.0, 0.0));
    let f_up = spawn(&mut f, "FactUp", t1, Coord3D::new(300.0, 200.0, 0.0));
    let mut bl0 = BuildListInfo::new();
    bl0.template_name = "FactPlain".into();
    bl0.object_id = f.sim.world.objects.id_of(f_plain).unwrap();
    let mut bl1 = BuildListInfo::new();
    bl1.template_name = "FactUp".into();
    bl1.object_id = f.sim.world.objects.id_of(f_up).unwrap();
    f.sim.world.players.player_mut(1).unwrap().build_list = vec![bl0, bl1];
    let list = f.sim.world.players.player(1).unwrap().build_list.clone();
    build_upgrade(&mut f.sim, 1, &list, "NoSuchUpg");
    build_upgrade(&mut f.sim, 1, &list, "UpgObjBeta");
    let m_before = f.sim.player_money(t1);
    build_upgrade(&mut f.sim, 1, &list, "UpgPlayerAlpha");
    let m_after = f.sim.player_money(t1);
    assert_eq!(m_before, up_parts[1].parse::<UnsignedInt>().unwrap());
    assert_eq!(m_after, up_parts[2].parse::<UnsignedInt>().unwrap());
    let queued_cnt = f.sim.with_production(f_up, |pu, _, _| pu.production_count()).unwrap();
    assert_eq!(queued_cnt, up_parts[3].parse::<UnsignedInt>().unwrap());

    let rq_parts: Vec<&str> = all_rows[idx].split_whitespace().collect();
    assert_eq!(rq_parts[0], "REPAIRQ");
    idx += 1;
    let pristine = spawn(&mut f, "CmdCenter", t1, Coord3D::new(400.0, 400.0, 0.0));
    let dmg_bldg = spawn(&mut f, "BaseDef", t1, Coord3D::new(450.0, 450.0, 0.0));
    let pristine_id = f.sim.world.objects.id_of(pristine).unwrap();
    let dmg_id = f.sim.world.objects.id_of(dmg_bldg).unwrap();
    let mut di = DamageInfo::default();
    di.input.damage_type = DamageType::Explosion;
    di.input.death_type = DeathType::Normal;
    di.input.amount = 450.0;
    f.sim.attempt_damage_on(dmg_bldg, &mut di);
    {
        let mut ai = f.sim.world.ai_players.remove(&1).unwrap();
        repair_structure(&f.sim, &mut ai, ObjectId(99999));
        repair_structure(&f.sim, &mut ai, pristine_id);
        repair_structure(&f.sim, &mut ai, dmg_id);
        repair_structure(&f.sim, &mut ai, dmg_id);
        assert_eq!(ai.structures_in_queue, rq_parts[1].parse::<Int>().unwrap());
        assert_eq!(ai.structures_to_repair[0], dmg_id);
        f.sim.world.ai_players.insert(1, ai);
    }

    // BREP 0, 1, 2
    let b0: Vec<&str> = all_rows[idx].split_whitespace().collect();
    assert_eq!(b0[0], "BREP");
    idx += 1;
    {
        let mut cx = BuildContext {
            sim: &mut f.sim,
            named: &mut f.named,
            unported: &mut f.unported,
            pathfind_extent: extent,
            teams: Some(&mut f.teams),
            scripts: Some(&mut f.scripts),
            stores: Some(&f.stores),
            waypoints: &f.waypoints,
        };
        ai_player_update(&mut cx, 1);
    }
    {
        let ai = f.sim.world.ai_players.get_mut(&1).unwrap();
        assert_eq!(ai.bridge_timer, b0[2].parse::<Int>().unwrap());
        assert_eq!(ai.repair_dozer, ObjectId::INVALID);
        assert_eq!(ai.dozer_queued_for_repair, b0[4].parse::<Bool>().unwrap());
        assert_eq!(ai.dozer_is_repairing, b0[5].parse::<Bool>().unwrap());
        ai.dozer_queued_for_repair = FALSE;
        ai.bridge_timer = 1;
    }

    let b1: Vec<&str> = all_rows[idx].split_whitespace().collect();
    assert_eq!(b1[0], "BREP");
    idx += 1;
    let doz = spawn(&mut f, "DozerA", t1, Coord3D::new(220.0, 220.0, 0.0));
    let doz_id = f.sim.world.objects.id_of(doz).unwrap();
    {
        let mut cx = BuildContext {
            sim: &mut f.sim,
            named: &mut f.named,
            unported: &mut f.unported,
            pathfind_extent: extent,
            teams: Some(&mut f.teams),
            scripts: Some(&mut f.scripts),
            stores: Some(&f.stores),
            waypoints: &f.waypoints,
        };
        ai_player_update(&mut cx, 1);
    }
    {
        let ai = f.sim.world.ai_players.get_mut(&1).unwrap();
        assert_eq!(ai.bridge_timer, b1[2].parse::<Int>().unwrap());
        assert_eq!(ai.repair_dozer, doz_id);
        assert_eq!(ai.dozer_is_repairing, b1[5].parse::<Bool>().unwrap());
        assert_eq!(fx(ai.repair_dozer_origin.x), b1[6]);
        assert_eq!(fx(ai.repair_dozer_origin.y), b1[7]);
    }

    let b2: Vec<&str> = all_rows[idx].split_whitespace().collect();
    assert_eq!(b2[0], "BREP");
    idx += 1;
    let mut heal_di = DamageInfo::default();
    heal_di.input.damage_type = DamageType::Healing;
    heal_di.input.amount = 1000.0;
    f.sim.attempt_healing_on(dmg_bldg, &mut heal_di);
    {
        let ai = f.sim.world.ai_players.get_mut(&1).unwrap();
        ai.base_center = Coord3D::new(180.0, 180.0, 0.0);
        ai.base_center_set = TRUE;
        ai.bridge_timer = 1;
    }
    {
        let mut cx = BuildContext {
            sim: &mut f.sim,
            named: &mut f.named,
            unported: &mut f.unported,
            pathfind_extent: extent,
            teams: Some(&mut f.teams),
            scripts: Some(&mut f.scripts),
            stores: Some(&f.stores),
            waypoints: &f.waypoints,
        };
        ai_player_update(&mut cx, 1);
    }
    {
        let ai = f.sim.world.ai_players.get(&1).unwrap();
        assert_eq!(ai.structures_in_queue, b2[2].parse::<Int>().unwrap());
        assert_eq!(ai.dozer_is_repairing, b2[3].parse::<Bool>().unwrap());
        assert_eq!(ai.repair_dozer, doz_id);
    }

    // 6) Non-skirmish processBaseBuilding (with RebuildHole adoption)
    let ns_parts: Vec<&str> = all_rows[idx].split_whitespace().collect();
    assert_eq!(ns_parts[0], "NSBASE");
    idx += 1;
    assert_eq!(idx, all_rows.len());

    f = setup_world(false);
    set_money(&mut f, 1, 5000);
    let t1 = f.sim.world.players.player(1).unwrap().default_team.unwrap();
    let hole = spawn(&mut f, "RebuildHole", t1, Coord3D::new(300.0, 300.0, 0.0));
    let hole_id = f.sim.world.objects.id_of(hole).unwrap();
    let base_def_tmpl = f.sim.rules.thing_factory.find_template_id("BaseDef").unwrap();
    let old_spawner = ObjectId(777);
    SimApi::start_rebuild_hole(&mut f.sim, hole, base_def_tmpl, old_spawner);

    let mut bl_hole = BuildListInfo::new();
    bl_hole.template_name = "BaseDef".into();
    bl_hole.object_id = old_spawner;
    bl_hole.num_rebuilds = 0;

    let mut bl_new = BuildListInfo::new();
    bl_new.template_name = "FactPlain".into();
    bl_new.location = Coord3D::new(500.0, 500.0, 0.0);
    bl_new.num_rebuilds = 2;
    f.sim.world.players.player_mut(1).unwrap().build_list = vec![bl_hole, bl_new];

    spawn(&mut f, "DozerA", t1, Coord3D::new(480.0, 500.0, 0.0));
    f.sim.world.ai_players.get_mut(&1).unwrap().ready_to_build_structure = TRUE;
    f.sim.world.frame = 50;
    {
        let mut cx = BuildContext {
            sim: &mut f.sim,
            named: &mut f.named,
            unported: &mut f.unported,
            pathfind_extent: extent,
            teams: Some(&mut f.teams),
            scripts: Some(&mut f.scripts),
            stores: Some(&f.stores),
            waypoints: &f.waypoints,
        };
        ai_player_update(&mut cx, 1);
    }
    let bl = &f.sim.world.players.player(1).unwrap().build_list;
    let hole_ported = f
        .sim
        .world
        .objects
        .get(hole)
        .and_then(|o| o.rebuild_hole_slot)
        .and_then(|ms| f.sim.world.modules.payload(ms))
        .and_then(|m| m.as_rebuild_hole())
        .is_some();
    assert_eq!(
        bl[0].object_id,
        if hole_ported { hole_id } else { ObjectId::INVALID }
    );
    assert_eq!(bl[1].object_id, ObjectId::INVALID);
    assert_eq!(bl[1].num_rebuilds, ns_parts[4].parse::<UnsignedInt>().unwrap());
    let ai = f.sim.world.ai_players.get(&1).unwrap();
    assert_eq!(ai.ready_to_build_structure, ns_parts[5].parse::<Bool>().unwrap());
    assert_eq!(ai.structure_timer, ns_parts[6].parse::<Int>().unwrap());
    assert_eq!(ai.frame_last_building_built, ns_parts[7].parse::<Int>().unwrap());
}

#[test]
fn non_skirmish_rebuild_delay_captured_and_replacement_dozer() {
    let mut f = setup_world(false);
    set_money(&mut f, 1, 5000);
    let t1 = f.sim.world.players.player(1).unwrap().default_team.unwrap();
    let t2 = f.sim.world.players.player(2).unwrap().default_team.unwrap();
    let extent = f.sim.terrain.terrain_logic().unwrap().maximum_pathfind_extent();

    // Building 0: under construction by dead builder -> replacement dozer gets ResumeConstruction
    let bldg = spawn(&mut f, "FactPlain", t1, Coord3D::new(300.0, 300.0, 0.0));
    let bldg_id = f.sim.world.objects.id_of(bldg).unwrap();
    if let Some(o) = f.sim.world.objects.get_mut(bldg) {
        o.status.set(ObjectStatusType::UnderConstruction.as_i32(), 1);
        o.builder_id = ObjectId(8888);
    }
    let doz = spawn(&mut f, "DozerA", t1, Coord3D::new(320.0, 300.0, 0.0));

    // Building 1: owned by enemy (captured!) -> cleared to INVALID_ID
    let cap = spawn(&mut f, "FactPlain", t2, Coord3D::new(400.0, 400.0, 0.0));
    let cap_id = f.sim.world.objects.id_of(cap).unwrap();

    // Building 2: waiting rebuild delay -> built once delay expires
    let mut bl0 = BuildListInfo::new();
    bl0.template_name = "FactPlain".into();
    bl0.object_id = bldg_id;
    bl0.num_rebuilds = 0;
    let mut bl1 = BuildListInfo::new();
    bl1.template_name = "FactPlain".into();
    bl1.object_id = cap_id;
    bl1.num_rebuilds = 0;
    let mut bl2 = BuildListInfo::new();
    bl2.template_name = "BaseDef".into();
    bl2.object_id = ObjectId::INVALID;
    bl2.object_timestamp = 100;
    bl2.location = Coord3D::new(500.0, 500.0, 0.0);
    bl2.num_rebuilds = 2;

    f.sim.world.players.player_mut(1).unwrap().build_list = vec![bl0, bl1, bl2];
    f.sim.rules.ai_data.rebuild_delay_seconds = 1;
    f.sim.world.frame = 110;
    f.sim.world.ai_players.get_mut(&1).unwrap().ready_to_build_structure = TRUE;
    {
        let mut cx = BuildContext {
            sim: &mut f.sim,
            named: &mut f.named,
            unported: &mut f.unported,
            pathfind_extent: extent,
            teams: Some(&mut f.teams),
            scripts: Some(&mut f.scripts),
            stores: Some(&f.stores),
            waypoints: &f.waypoints,
        };
        ai_player_update(&mut cx, 1);
    }
    let bl = &f.sim.world.players.player(1).unwrap().build_list;
    assert_eq!(bl[1].object_id, ObjectId::INVALID);
    assert_eq!(bl[1].object_timestamp, 111);
    assert_eq!(bl[2].object_id, ObjectId::INVALID); // delay (100 + 30 > 110) kept it pending
    assert_eq!(
        f.sim
            .world
            .objects
            .get(doz)
            .and_then(|o| o.ai_slot)
            .and_then(|ms| f.sim.world.modules.payload(ms))
            .and_then(|m| m.as_any())
            .and_then(|a| a.downcast_ref::<AiUpdate>())
            .map(|ai| ai.last_command_source),
        Some(CommandSourceType::FromAi)
    );
    assert!(f.unported.is_empty(), "{:?}", f.unported);
}

#[test]
fn broken_bridge_check_and_repair_completion_to_home() {
    let mut f = setup_world(true);
    let t1 = f.sim.world.players.player(1).unwrap().default_team.unwrap();
    let t0 = f.sim.world.players.player(0).unwrap().default_team.unwrap();
    let extent = Region3D {
        lo: Coord3D::default(),
        hi: Coord3D::new(200.0, 120.0, 100.0),
    };
    let mut terrain = TerrainLogic::new();
    terrain.set_synthetic_bounds(20, 12, 100.0);
    let mut pf = Pathfinder::new(0.0);
    let corner = |x, y| Coord3D::new(x, y, 30.0);
    let geometry = BridgeGeometry::from_corners(
        corner(40.0, 55.0),
        corner(160.0, 55.0),
        corner(40.0, 70.0),
        corner(40.0, 40.0),
        corner(160.0, 70.0),
        corner(160.0, 40.0),
    );
    struct CliffTerrain(TerrainLogic);
    impl ThingTerrainAccess for CliffTerrain {
        fn terrain_logic(&self) -> Option<&TerrainLogic> {
            Some(&self.0)
        }
        fn terrain_logic_mut(&mut self) -> Option<&mut TerrainLogic> {
            Some(&mut self.0)
        }
        fn get_ground_height(&self, _x: Real, _y: Real, normal: Option<&mut Coord3D>) -> Real {
            if let Some(n) = normal {
                *n = Coord3D::new(0.0, 0.0, 1.0);
            }
            0.0
        }
        fn align_on_terrain(
            &self,
            _angle: Real,
            _pos: &Coord3D,
            _stick_to_ground: Bool,
            _mtx: &mut Matrix3D,
        ) -> PathfindLayerEnum {
            PathfindLayerEnum::Ground
        }
        fn is_cliff_cell(&self, x: Real, _y: Real) -> Bool {
            Bool::from((95.0..=105.0).contains(&x))
        }
        fn is_underwater(
            &self,
            _x: Real,
            _y: Real,
            water_z: Option<&mut Real>,
            terrain_z: Option<&mut Real>,
        ) -> Bool {
            if let Some(w) = water_z {
                *w = 0.0;
            }
            if let Some(t) = terrain_z {
                *t = 0.0;
            }
            FALSE
        }
        fn get_layer_height(
            &self,
            _x: Real,
            _y: Real,
            _layer: PathfindLayerEnum,
            normal: Option<&mut Coord3D>,
            _clip: Bool,
        ) -> Real {
            if let Some(n) = normal {
                *n = Coord3D::new(0.0, 0.0, 1.0);
            }
            0.0
        }
    }

    let bridge_obj = spawn(&mut f, "BaseDef", t0, Coord3D::new(100.0, 55.0, 0.0));
    let bridge_id = f.sim.world.objects.id_of(bridge_obj).unwrap();
    terrain.add_bridge(Bridge::new(geometry, "road", bridge_id), &mut pf);
    let mut cliff_terrain = CliffTerrain(terrain);
    pf.new_map(&cliff_terrain, &extent, 0.0, &[], &NoPathWorld, 0);
    let layer = PathfindLayerEnum::Bridge1;
    pf.change_bridge_state(layer, false, &cliff_terrain, 10);
    pf.calculate_zones();
    assert_eq!(
        pf.find_broken_bridge(
            &cliff_terrain,
            1,
            &Coord3D::new(20.0, 55.0, 0.0),
            &Coord3D::new(180.0, 55.0, 0.0),
        ),
        Some(bridge_id)
    );

    cliff_terrain.0.push_waypoint_for_test(
        1,
        "Start",
        Coord3D::new(180.0, 55.0, 0.0),
        [String::new(), String::new(), String::new()],
    );
    f.sim.terrain = Box::new(cliff_terrain);
    f.sim.world.pathfinder = Some(Box::new(pf));

    let mut di = DamageInfo::default();
    di.input.damage_type = DamageType::Explosion;
    di.input.death_type = DeathType::Normal;
    di.input.amount = 450.0;
    f.sim.attempt_damage_on(bridge_obj, &mut di);

    let unit = spawn(&mut f, "UnitA", t1, Coord3D::new(20.0, 55.0, 0.0));
    let start_wp = f
        .sim
        .terrain
        .terrain_logic()
        .unwrap()
        .waypoint_by_id(1)
        .cloned();
    let mut ai = f.sim.world.ai_players.remove(&1).unwrap();
    assert!(check_bridges(&f.sim, &mut ai, unit, start_wp.as_ref()));
    assert_eq!(ai.structures_in_queue, 1);
    assert_eq!(ai.structures_to_repair[0], bridge_id);
    f.sim.world.ai_players.insert(1, ai);
}

#[test]
fn script_engine_dispatches_all_ai_ops() {
    let mut f = setup_world(true);
    let mut ai_loader = GameDataLoader::new();
    ai_loader.load_bytes(
        "ai_side.ini",
        b"AIData\n  SideInfo America\n    BaseDefenseStructure1 = BaseDef\n  End\nEnd\n",
        IniLoadType::Overwrite,
    );
    f.sim.rules.ai_data.side_info = ai_loader.rules.ai_data.side_info;
    {
        let tl = f.sim.terrain.terrain_logic_mut().unwrap();
        tl.push_waypoint_for_test(
            1,
            "C",
            Coord3D::new(700.0, 500.0, 0.0),
            ["Center1".into(), String::new(), String::new()],
        );
    }
    let extent = f
        .sim
        .terrain
        .terrain_logic()
        .unwrap()
        .maximum_pathfind_extent();
    {
        let mut ai = f.sim.world.ai_players.remove(&1).unwrap();
        ai.base_center = Coord3D::new(500.0, 500.0, 0.0);
        ai.base_center_set = TRUE;
        ai.base_radius = 100.0;
        build_ai_base_defense(&mut f.sim, &mut ai, 1, false, &extent);
        assert_eq!(f.sim.world.players.player(1).unwrap().build_list.len(), 1);
        f.sim.world.ai_players.insert(1, ai);
    }

    let t1 = f.sim.world.players.player(1).unwrap().default_team.unwrap();
    let dmg = spawn(&mut f, "BaseDef", t1, Coord3D::new(350.0, 350.0, 0.0));
    let dmg_id = f.sim.world.objects.id_of(dmg).unwrap();
    f.named.push(("Bldg1".into(), dmg_id));
    f.scripts.engine.add_object_to_cache("Bldg1", dmg_id);
    let mut di = DamageInfo::default();
    di.input.damage_type = DamageType::Explosion;
    di.input.death_type = DeathType::Normal;
    di.input.amount = 450.0;
    f.sim.attempt_damage_on(dmg, &mut di);

    set_money(&mut f, 1, 1000);
    let fac = spawn(&mut f, "FactUp", t1, Coord3D::new(200.0, 200.0, 0.0));
    let mut bl = BuildListInfo::new();
    bl.template_name = "FactUp".into();
    bl.object_id = f.sim.world.objects.id_of(fac).unwrap();
    f.sim.world.players.player_mut(1).unwrap().build_list.push(bl);

    let reg = script_template_registry(&mut f.sim.rules.name_keys);
    f.sim.world.frame = 10;

    let mut cond_safe = Condition::new(ct::SUPPLY_SOURCE_SAFE, &reg);
    cond_safe.parms[0].string_val = "AiPlayer1".into();
    cond_safe.parms[1].int_val = 200;

    let mut cond_atk = Condition::new(ct::SUPPLY_SOURCE_ATTACKED, &reg);
    cond_atk.parms[0].string_val = "AiPlayer1".into();

    let mut a_skill = ScriptAction::new(at::PLAYER_SELECT_SKILLSET, &reg);
    a_skill.parms[0].string_val = "AiPlayer1".into();
    a_skill.parms[1].int_val = 2;

    let mut a_del = ScriptAction::new(at::SET_BASE_CONSTRUCTION_SPEED, &reg);
    a_del.parms[0].string_val = "AiPlayer1".into();
    a_del.parms[1].int_val = 45;

    let mut a_rep = ScriptAction::new(at::PLAYER_REPAIR_NAMED_STRUCTURE, &reg);
    a_rep.parms[0].string_val = "AiPlayer1".into();
    a_rep.parms[1].string_val = "Bldg1".into();

    let mut a_upg = ScriptAction::new(at::AI_PLAYER_BUILD_UPGRADE, &reg);
    a_upg.parms[0].string_val = "AiPlayer1".into();
    a_upg.parms[1].string_val = "UpgPlayerAlpha".into();

    f.scripts.with_world(
        &mut f.sim,
        ScriptGame {
            teams: &mut f.teams,
            stores: &f.stores,
            waypoints: &f.waypoints,
            named_objects: &mut f.named,
        },
        |engine, world| {
            assert!(engine.evaluate_condition(world, &mut cond_safe));
            assert!(!engine.evaluate_condition(world, &mut cond_atk));
            engine.execute_action(world, &mut a_skill);
            engine.execute_action(world, &mut a_del);
            engine.execute_action(world, &mut a_rep);
            engine.execute_action(world, &mut a_upg);
            assert!(engine.unported_conditions.is_empty());
            assert!(engine.unported_actions.is_empty());
        },
    );

    let ai = f.sim.world.ai_players.get(&1).unwrap();
    assert_eq!(ai.skillset_selector, 1); // 1-based 2 -> 0-based 1
    assert_eq!(ai.team_seconds, 45);
    assert_eq!(ai.structures_in_queue, 1);
    assert_eq!(ai.structures_to_repair[0], dmg_id);
    assert_eq!(
        f.sim
            .with_production(fac, |pu, _, _| pu.production_count()),
        Some(1)
    );
}

#[test]
fn mutation_edge_cases_and_remaining_script_ops() {
    // 1) calc_closest_construction_zone_location: positive wiggle, 2*cell step, 2*SUPPLY_CENTER_CLOSE_DIST cap
    let mut f = setup_world(false);
    f.sim.world.ai_players.get_mut(&1).unwrap().base_center = Coord3D::new(200.0, 200.0, 0.0);
    let extent = f.sim.terrain.terrain_logic().unwrap().maximum_pathfind_extent();
    {
        let ai = f.sim.world.ai_players.get(&1).unwrap();
        let mut loc1 = Coord3D::new(-5.0, 400.0, 0.0);
        assert!(calc_closest_construction_zone_location(&f.sim, ai, "BaseDef", &mut loc1, &extent));
        assert!((loc1.x - 15.0).abs() < 0.01 && (loc1.y - 380.0).abs() < 0.01, "loc1={loc1:?}");

        let mut loc2 = Coord3D::new(-15.0, 400.0, 0.0);
        assert!(calc_closest_construction_zone_location(&f.sim, ai, "BaseDef", &mut loc2, &extent));
        assert!((loc2.x - 15.0).abs() < 0.01, "loc2={loc2:?}");

        let mut loc_far = Coord3D::new(-285.0, 400.0, 0.0);
        assert!(!calc_closest_construction_zone_location(&f.sim, ai, "BaseDef", &mut loc_far, &extent));
    }

    // 2) is_supply_source_attacked boundary checks (unwrap_or(0), check_frame <, wrap+scan <)
    let t1 = f.sim.world.players.player(1).unwrap().default_team.unwrap();
    let harv = spawn(&mut f, "HarvA", t1, Coord3D::new(250.0, 250.0, 0.0));
    let set_harv = |f: &mut Fixture, ts: UnsignedInt| {
        let bs = f.sim.world.objects.get(harv).unwrap().body_slot.unwrap();
        let mut di = DamageInfo::default();
        di.output.no_effect = FALSE;
        if let Some(m) = f.sim.world.modules.payload_mut(bs) {
            if let Some(ab) = m.as_any_mut().and_then(|a| a.downcast_mut::<zh_logic::modules::body::active_body::ActiveBody>()) {
                ab.last_damage_timestamp = ts;
                ab.last_damage_info = di;
            }
        }
    };
    set_harv(&mut f, 200);
    f.sim.world.frame = 301;
    {
        let mut ai = f.sim.world.ai_players.remove(&1).unwrap();
        ai.supply_source_attack_check_frame = 0;
        assert!(!is_supply_source_attacked(&f.sim, &mut ai));

        f.sim.world.players.player_mut(1).unwrap().attacked_frame = 9;
        set_harv(&mut f, 9);
        f.sim.world.frame = 10;
        ai.supply_source_attack_check_frame = 10;
        assert!(is_supply_source_attacked(&f.sim, &mut ai));

        f.sim.world.players.player_mut(1).unwrap().attacked_frame = 10;
        set_harv(&mut f, 20);
        f.sim.world.frame = 310;
        ai.supply_source_attack_check_frame = 0;
        assert!(is_supply_source_attacked(&f.sim, &mut ai));
        f.sim.world.ai_players.insert(1, ai);
    }

    // 3) update_bridge_repair: bridge_timer > 0, shift_repair_queue(2), killed dozer -> timer = 0
    let dead_b = spawn(&mut f, "BaseDef", t1, Coord3D::new(410.0, 410.0, 0.0));
    let live_b = spawn(&mut f, "BaseDef", t1, Coord3D::new(450.0, 450.0, 0.0));
    let live_id = f.sim.world.objects.id_of(live_b).unwrap();
    let doz = spawn(&mut f, "DozerA", t1, Coord3D::new(220.0, 220.0, 0.0));
    let doz_id = f.sim.world.objects.id_of(doz).unwrap();
    let mut di = DamageInfo::default();
    di.input.damage_type = DamageType::Explosion;
    di.input.death_type = DeathType::Normal;
    di.input.amount = 450.0;
    f.sim.attempt_damage_on(dead_b, &mut di);
    f.sim.attempt_damage_on(live_b, &mut di);
    {
        let dead_id = f.sim.world.objects.id_of(dead_b).unwrap();
        let mut ai = f.sim.world.ai_players.remove(&1).unwrap();
        repair_structure(&f.sim, &mut ai, dead_id);
        repair_structure(&f.sim, &mut ai, live_id);
        assert_eq!(ai.structures_in_queue, 2);
        ai.bridge_timer = 2;
        f.sim.world.ai_players.insert(1, ai);
    }
    f.sim.destroy_object_now(dead_b);
    f.sim.process_destroy_list();
    let step_ai = |f: &mut Fixture| {
        let mut cx = BuildContext {
            sim: &mut f.sim,
            named: &mut f.named,
            unported: &mut f.unported,
            pathfind_extent: extent,
            teams: Some(&mut f.teams),
            scripts: Some(&mut f.scripts),
            stores: Some(&f.stores),
            waypoints: &f.waypoints,
        };
        ai_player_update(&mut cx, 1);
    };
    step_ai(&mut f);
    assert_eq!(f.sim.world.ai_players.get(&1).unwrap().bridge_timer, 1);
    assert_eq!(f.sim.world.ai_players.get(&1).unwrap().structures_in_queue, 2);

    step_ai(&mut f);
    {
        let ai = f.sim.world.ai_players.get(&1).unwrap();
        assert_eq!(ai.structures_in_queue, 1);
        assert_eq!(ai.structures_to_repair[0], live_id);
        assert_eq!(ai.repair_dozer, doz_id);
    }
    f.sim.destroy_object_now(doz);
    f.sim.process_destroy_list();
    f.sim.world.ai_players.get_mut(&1).unwrap().bridge_timer = 1;
    step_ai(&mut f);
    {
        let ai = f.sim.world.ai_players.get(&1).unwrap();
        assert_eq!(ai.repair_dozer, ObjectId::INVALID);
        assert_eq!(ai.bridge_timer, 0);
    }

    // 4) Non-skirmish process_base_building: rebuild delay + timestamp clear + wealthy exact boundary
    let mut f2 = setup_world_with_extra(
        false,
        concat!(
            "CommandButton Btn_BuildBaseDef\n  Command = DOZER_CONSTRUCT\n  Object = BaseDef\nEnd\n",
            "CommandSet CmdDozerBuild\n  1 = Btn_BuildBaseDef\nEnd\n",
            "Object DozerBuild\n",
            "  Side = America\n  KindOf = PRELOAD SELECTABLE VEHICLE DOZER\n  CommandSet = CmdDozerBuild\n",
            "  BuildCost = 1000\n  BuildTime = 1.0\n",
            "  Geometry = CYLINDER\n  GeometryMajorRadius = 7\n  GeometryHeight = 8\n",
            "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n",
            "  Behavior = DozerAIUpdate ModuleTag_AI\n    RepairHealthPercentPerSecond = 50%\n    BoredTime = 5000\n    BoredRange = 150\n  End\n",
            "  Locomotor = SET_NORMAL AptGround\n",
            "End\n",
        ),
    );
    f2.sim.rules.ai_data.rebuild_delay_seconds = 1;
    f2.sim.rules.ai_data.structure_seconds = 10.0;
    f2.sim.rules.ai_data.resources_wealthy = 2000;
    f2.sim.rules.ai_data.structures_wealthy_mod = 2.0;
    set_money(&mut f2, 1, 2000);
    let mut bl = BuildListInfo::new();
    bl.template_name = "BaseDef".into();
    bl.object_id = ObjectId::INVALID;
    bl.object_timestamp = 100;
    bl.location = Coord3D::new(500.0, 500.0, 0.0);
    bl.num_rebuilds = 2;
    f2.sim.world.players.player_mut(1).unwrap().build_list = vec![bl];
    let t1_2 = f2.sim.world.players.player(1).unwrap().default_team.unwrap();
    let d2 = spawn(&mut f2, "DozerBuild", t1_2, Coord3D::new(480.0, 500.0, 0.0));
    f2.sim.world.frame = 110;
    f2.sim.world.ai_players.get_mut(&1).unwrap().ready_to_build_structure = TRUE;
    {
        let mut cx = BuildContext {
            sim: &mut f2.sim,
            named: &mut f2.named,
            unported: &mut f2.unported,
            pathfind_extent: extent,
            teams: Some(&mut f2.teams),
            scripts: Some(&mut f2.scripts),
            stores: Some(&f2.stores),
            waypoints: &f2.waypoints,
        };
        ai_player_update(&mut cx, 1);
    }
    assert_eq!(f2.sim.world.players.player(1).unwrap().build_list[0].object_id, ObjectId::INVALID);
    assert_eq!(f2.sim.world.players.player(1).unwrap().build_list[0].object_timestamp, 100);

    // Without dozer, delay expiration clears object_timestamp to 0
    f2.sim.destroy_object_now(d2);
    f2.sim.process_destroy_list();
    f2.sim.world.frame = 140;
    {
        let ai = f2.sim.world.ai_players.get_mut(&1).unwrap();
        ai.ready_to_build_structure = TRUE;
        ai.build_delay = 0;
    }
    {
        let mut cx = BuildContext {
            sim: &mut f2.sim,
            named: &mut f2.named,
            unported: &mut f2.unported,
            pathfind_extent: extent,
            teams: Some(&mut f2.teams),
            scripts: Some(&mut f2.scripts),
            stores: Some(&f2.stores),
            waypoints: &f2.waypoints,
        };
        ai_player_update(&mut cx, 1);
    }
    assert_eq!(f2.sim.world.players.player(1).unwrap().build_list[0].object_timestamp, 0);

    // With dozer and money == wealthy (not > wealthy), starts build, resets ready_to_build_structure, full timer
    spawn(&mut f2, "DozerBuild", t1_2, Coord3D::new(480.0, 500.0, 0.0));
    {
        let ai = f2.sim.world.ai_players.get_mut(&1).unwrap();
        ai.ready_to_build_structure = TRUE;
        ai.build_delay = 0;
    }
    {
        let mut cx = BuildContext {
            sim: &mut f2.sim,
            named: &mut f2.named,
            unported: &mut f2.unported,
            pathfind_extent: extent,
            teams: Some(&mut f2.teams),
            scripts: Some(&mut f2.scripts),
            stores: Some(&f2.stores),
            waypoints: &f2.waypoints,
        };
        ai_player_update(&mut cx, 1);
    }
    assert_ne!(f2.sim.world.players.player(1).unwrap().build_list[0].object_id, ObjectId::INVALID);
    let ai2 = f2.sim.world.ai_players.get(&1).unwrap();
    assert_eq!(ai2.ready_to_build_structure, FALSE);
    assert_eq!(ai2.structure_timer, 300);

    // 5) build_upgrade cost == money boundary + superweapon CommandCenter/10 + check_bridges non-AiUpdate
    let fac = spawn(&mut f2, "FactUp", t1_2, Coord3D::new(200.0, 200.0, 0.0));
    let mut ubl = BuildListInfo::new();
    ubl.template_name = "FactUp".into();
    ubl.object_id = f2.sim.world.objects.id_of(fac).unwrap();
    f2.sim.world.players.player_mut(1).unwrap().build_list.push(ubl);
    let list = f2.sim.world.players.player(1).unwrap().build_list.clone();
    set_money(&mut f2, 1, 499);
    build_upgrade(&mut f2.sim, 1, &list, "UpgPlayerAlpha");
    assert_eq!(f2.sim.with_production(fac, |pu, _, _| pu.production_count()), Some(0));
    set_money(&mut f2, 1, 500);
    build_upgrade(&mut f2.sim, 1, &list, "UpgPlayerAlpha");
    assert_eq!(f2.sim.with_production(fac, |pu, _, _| pu.production_count()), Some(1));

    let t2_2 = f2.sim.world.players.player(2).unwrap().default_team.unwrap();
    spawn(&mut f2, "CmdCenter", t2_2, Coord3D::new(500.0, 500.0, 0.0));
    assert_eq!(get_player_superweapon_value(&f2.sim, &Coord3D::new(500.0, 500.0, 0.0), 2, 100.0, true), 200);
    let mut ai2_owned = f2.sim.world.ai_players.remove(&1).unwrap();
    ai2_owned.skirmish = Some(Default::default());
    assert!(!check_bridges(&f2.sim, &mut ai2_owned, fac, None));

    // 6) skirmish_enemy_index player 0 Human + bounds.y fallback + angle > PI/3.0 + remaining script actions
    let mut f3 = setup_world(true);
    let mut ai_loader = GameDataLoader::new();
    ai_loader.load_bytes(
        "ai_side.ini",
        b"AIData\n  SideInfo America\n    BaseDefenseStructure1 = BaseDef\n  End\nEnd\n",
        IniLoadType::Overwrite,
    );
    f3.sim.rules.ai_data.side_info = ai_loader.rules.ai_data.side_info;
    f3.sim.world.players.player_mut(0).unwrap().player_type = PlayerType::Human;
    let t0_3 = f3.sim.world.players.player(0).unwrap().default_team.unwrap();
    spawn(&mut f3, "CmdCenter", t0_3, Coord3D::new(700.0, 800.0, 0.0));
    let t2_3 = f3.sim.world.players.player(2).unwrap().default_team.unwrap();
    spawn(&mut f3, "CmdCenter", t2_3, Coord3D::new(700.0, 300.0, 0.0));
    spawn(&mut f3, "CmdCenter", t2_3, Coord3D::new(700.0, 700.0, 0.0));
    let t1_3 = f3.sim.world.players.player(1).unwrap().default_team.unwrap();
    spawn(&mut f3, "Warehouse", t1_3, Coord3D::new(300.0, 300.0, 0.0));

    let gt_proto = f3.teams.find_team_prototype("GuardTeam", &mut f3.sim.rules.name_keys).unwrap();
    let gt = f3.teams.instance_ids(gt_proto)[0];
    if let Some(t) = f3.teams.team_mut(gt) {
        t.active = true;
        t.created = true;
    }
    let g1 = spawn(&mut f3, "UnitA", gt, Coord3D::new(-5.0, 400.0, 0.0));
    {
        let mut ai = f3.sim.world.ai_players.remove(&1).unwrap();
        ai.base_center = Coord3D::new(500.0, 500.0, 0.0);
        ai.base_center_set = TRUE;
        ai.base_radius = 100.0;
        {
            let sk = ai.skirmish.as_mut().unwrap();
            sk.current_enemy = None;
            sk.frame_to_check_enemy = 999_999;
        }
        build_ai_base_defense(&mut f3.sim, &mut ai, 1, false, &extent);
        let bl_pos = f3.sim.world.players.player(1).unwrap().build_list[0].location;
        assert!(bl_pos.y > 500.0); // came from player 0 at y=800, not player 2

        {
            let sk = ai.skirmish.as_mut().unwrap();
            sk.cur_front_base_defense = 2;
            sk.cur_front_left_defense_angle = 1.02; // PI/3.1 < 1.02 < PI/3.0
        }
        let before = f3.sim.world.players.player(1).unwrap().build_list.len();
        build_ai_base_defense_structure(&mut f3.sim, &mut ai, 1, "BaseDef", false, &extent);
        assert_eq!(f3.sim.world.players.player(1).unwrap().build_list.len(), before + 1);
        ai.skirmish.as_mut().unwrap().current_enemy = Some(2);
        f3.sim.world.ai_players.insert(1, ai);
    }

    {
        let tl = f3.sim.terrain.terrain_logic_mut().unwrap();
        tl.push_waypoint_for_test(1, "C", Coord3D::new(700.0, 500.0, 0.0), ["Center1".into(), String::new(), String::new()]);
        tl.push_waypoint_for_test(2, "F", Coord3D::new(500.0, 700.0, 0.0), ["Flank1".into(), String::new(), String::new()]);
        tl.push_waypoint_for_test(3, "B", Coord3D::new(300.0, 500.0, 0.0), ["Backdoor1".into(), String::new(), String::new()]);
    }
    let reg = script_template_registry(&mut f3.sim.rules.name_keys);
    let mut a_df = ScriptAction::new(at::SKIRMISH_BUILD_BASE_DEFENSE_FRONT, &reg);
    let mut a_dl = ScriptAction::new(at::SKIRMISH_BUILD_BASE_DEFENSE_FLANK, &reg);
    let mut a_sf = ScriptAction::new(at::SKIRMISH_BUILD_STRUCTURE_FRONT, &reg);
    a_sf.parms[0].string_val = "BaseDef".into();
    let mut a_sl = ScriptAction::new(at::SKIRMISH_BUILD_STRUCTURE_FLANK, &reg);
    a_sl.parms[0].string_val = "BaseDef".into();
    let mut a_gs = ScriptAction::new(at::TEAM_GUARD_SUPPLY_CENTER, &reg);
    a_gs.parms[0].string_val = "GuardTeam".into();
    a_gs.parms[1].int_val = 200;
    let mut a_nt = ScriptAction::new(at::AI_PLAYER_BUILD_TYPE_NEAREST_TEAM, &reg);
    a_nt.parms[0].string_val = "AiPlayer1".into();
    a_nt.parms[1].string_val = "BaseDef".into();
    a_nt.parms[2].string_val = "GuardTeam".into();

    f3.scripts.with_world(
        &mut f3.sim,
        ScriptGame {
            teams: &mut f3.teams,
            stores: &f3.stores,
            waypoints: &f3.waypoints,
            named_objects: &mut f3.named,
        },
        |engine, world| {
            engine.execute_action(world, &mut a_df);
            engine.execute_action(world, &mut a_dl);
            engine.execute_action(world, &mut a_sf);
            engine.execute_action(world, &mut a_sl);
            engine.execute_action(world, &mut a_gs);
            engine.execute_action(world, &mut a_nt);
            assert!(engine.unported_actions.is_empty());
        },
    );
    assert_eq!(f3.sim.world.players.player(1).unwrap().build_list[0].template_name, "BaseDef");
    assert!((f3.sim.world.players.player(1).unwrap().build_list[0].location.x - 15.0).abs() < 0.01);
    let g1_ai_slot = f3.sim.world.objects.get(g1).unwrap().ai_slot.unwrap();
    let g1_ai = f3.sim.world.modules.payload(g1_ai_slot).unwrap().as_any().unwrap().downcast_ref::<AiUpdate>().unwrap();
    assert!(g1_ai.location_to_guard.y > 300.0); // enemy center y = 500 > 300 -> guard shifts +y
}
