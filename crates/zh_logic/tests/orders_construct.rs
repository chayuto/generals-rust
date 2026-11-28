//! The recorded orders `onDozerConstruct`, `onQueueUnitCreate` and
//! `onQueueUpgrade` over a synthetic world: what each one changes, and that an
//! order naming a template id or key the rules lack is reported. The values are
//! those of the C++ handlers read directly; the same orders of the retail replays
//! are compared with the running reference in `golden_game_frames.rs`.

use zh_common::ini::IniLoadType;
use zh_common::io::recorder::{ReplayArg, ReplayCommand};
use zh_common::math::{Coord3D, Matrix3D, Region3D};
use zh_common::types::{Bool, ObjectId, Real, TeamId, UnsignedInt, TRUE};
use zh_logic::arena::ObjectSlot;
use zh_logic::gamelogic::command_target::{dispatch_command, CommandState, MSG_DOZER_CONSTRUCT};
use zh_logic::gamelogic::game_engine::GameDataLoader;
use zh_logic::modules::update::dozer_ai_update::{dozer_task_of, DOZER_TASK_BUILD};
use zh_logic::object::thing::{make_align_to_normal_matrix, ThingTerrainAccess};
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;
use zh_logic::rules::model_state::ModelConditionFlagType as Flag;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::sim::{SimApi, SimContext, SimView};

const MSG_CREATE_SELECTED_GROUP: i32 = 1001;
const MSG_QUEUE_UPGRADE: i32 = 1045;
const MSG_QUEUE_UNIT_CREATE: i32 = 1047;
const PLAYER: i32 = 1;
const TEAM: TeamId = TeamId(2);

/// Level ground above zero, so that the building's height shows.
struct Raised;

impl ThingTerrainAccess for Raised {
    fn get_ground_height(&self, _x: Real, _y: Real, normal: Option<&mut Coord3D>) -> Real {
        if let Some(n) = normal {
            *n = Coord3D::new(0.0, 0.0, 1.0);
        }
        12.5
    }

    fn align_on_terrain(
        &self,
        angle: Real,
        pos: &Coord3D,
        _stick_to_ground: Bool,
        mtx: &mut Matrix3D,
    ) -> PathfindLayerEnum {
        make_align_to_normal_matrix(angle, pos, &Coord3D::new(0.0, 0.0, 1.0), mtx);
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
            *t = 12.5;
        }
        Bool::from(false)
    }
}

const INI: &str = "\
CommandButton BuildBarracks
  Command = DOZER_CONSTRUCT
  Object = Barracks
End
CommandSet DozerSet
  1 = BuildBarracks
End
Object Dozer
  KindOf = SELECTABLE DOZER VEHICLE
  CommandSet = DozerSet
  Body = ActiveBody Tag_Body
    MaxHealth = 250
  End
  Behavior = DozerAIUpdate Tag_AI
  End
End
Object Barracks
  KindOf = STRUCTURE
  BuildCost = 500
  Geometry = BOX
  GeometryHeight = 20
  GeometryMajorRadius = 30
  GeometryMinorRadius = 20
  Body = StructureBody Tag_Body
    MaxHealth = 1000
  End
End
Object Hut
  KindOf = STRUCTURE
  BuildCost = 100
  Geometry = BOX
  GeometryHeight = 20
  GeometryMajorRadius = 10
  GeometryMinorRadius = 10
  Body = StructureBody Tag_Body
    MaxHealth = 100
  End
End
";

struct World {
    sim: SimContext,
    state: CommandState,
    dozer: ObjectSlot,
    dozer_id: ObjectId,
}

fn world(money: UnsignedInt) -> World {
    let mut loader = GameDataLoader::new();
    loader.load_bytes("orders.ini", INI.as_bytes(), IniLoadType::Overwrite);
    assert!(
        loader.report.failures.is_empty(),
        "{:?}",
        loader.report.failures
    );
    let mut sim = SimContext::new();
    sim.rules = loader.rules;
    sim.terrain = Box::new(Raised);
    sim.rules.global_data.partition_cell_size = 20.0;
    sim.init_partition(
        20.0,
        Region3D {
            lo: Coord3D::new(0.0, 0.0, 0.0),
            hi: Coord3D::new(2000.0, 2000.0, 100.0),
        },
    );
    let w = &mut sim.world;
    w.team_players.insert(TeamId(1), 0);
    w.player_default_teams.insert(0, TeamId(1));
    w.team_players.insert(TEAM, PLAYER);
    w.player_default_teams.insert(PLAYER, TEAM);
    w.player_is_human.insert(TEAM, Bool::from(true));
    w.team_is_neutral.insert(TeamId(1), TRUE);
    w.neutral_team_id = TeamId(1);
    w.players.player_mut(PLAYER).expect("player").money.set_starting_cash(money);
    w.player_relationships
        .insert((PLAYER, PLAYER), zh_common::types::Relationship::Allies);
    let id = sim
        .rules
        .thing_factory
        .find_template_id("Dozer")
        .expect("dozer");
    let dozer = sim.spawn_object(id, TEAM, Coord3D::new(500.0, 500.0, 0.0), 0.0);
    let dozer_id = sim.object(dozer).expect("dozer").id;
    let mut state = CommandState::default();
    let select = ReplayCommand::new(MSG_CREATE_SELECTED_GROUP, PLAYER)
        .with(ReplayArg::boolean(true))
        .with(ReplayArg::object_id(dozer_id.0 as u32));
    dispatch_command(&mut sim, &mut state, &select);
    World {
        sim,
        state,
        dozer,
        dozer_id,
    }
}

impl World {
    fn template(&self, name: &str) -> i32 {
        self.sim
            .rules
            .thing_factory
            .find_template_id(name)
            .expect("template")
            .0 as i32
    }

    fn money(&self) -> UnsignedInt {
        self.sim.money_of(PLAYER)
    }

    fn construct(&mut self, template: i32) -> Option<ObjectSlot> {
        let next = ObjectId(self.dozer_id.0 + 1);
        let order = ReplayCommand::new(MSG_DOZER_CONSTRUCT, PLAYER)
            .with(ReplayArg::integer(template))
            .with(ReplayArg::location(800.0, 700.0, 0.0))
            .with(ReplayArg::real(0.5));
        dispatch_command(&mut self.sim, &mut self.state, &order);
        self.sim.find_object(next)
    }
}

#[test]
fn a_dozer_starts_the_building_and_pays_for_it() {
    let mut w = world(1200);
    let barracks = w.template("Barracks");
    let building = w.construct(barracks).expect("a building");
    assert_eq!(w.money(), 700);
    let object = w.sim.object(building).expect("building");
    assert_eq!(object.team_id, TEAM);
    assert_eq!(object.producer_id, w.dozer_id);
    assert_eq!(object.builder_id, w.dozer_id);
    assert_ne!(
        object
            .status
            .test(ObjectStatusType::UnderConstruction.as_i32()),
        0
    );
    assert_eq!(object.construction_percent.to_bits(), 0.0f32.to_bits());
    assert_eq!(object.thing.get_position().x.to_bits(), 800.0f32.to_bits());
    assert_eq!(object.thing.get_orientation().to_bits(), 0.5f32.to_bits());
    assert_eq!(object.thing.get_position().z.to_bits(), 12.5f32.to_bits());
    let condition = |flag: Flag| object.model_condition.test(flag as i32) != 0;
    assert!(condition(Flag::AwaitingConstruction));
    assert!(!condition(Flag::PartiallyConstructed));
    assert!(!condition(Flag::ActivelyBeingConstructed));
    let id = object.id;
    assert_eq!(w.sim.object_health(building).to_bits(), 1.0f32.to_bits());
    // Under construction a structure sees only itself.
    let radius = object_radius(&w.sim, building);
    assert_eq!(
        w.sim.object_shroud_clearing_range(building).to_bits(),
        radius
    );
    assert_eq!(
        dozer_task_of(&mut w.sim, w.dozer),
        Some((DOZER_TASK_BUILD, id))
    );
    assert!(w.state.unknown_names.is_empty());
}

fn object_radius(sim: &SimContext, slot: ObjectSlot) -> u32 {
    sim.object(slot)
        .expect("object")
        .geometry
        .get_bounding_circle_radius()
        .to_bits()
}

#[test]
fn nothing_is_built_without_the_money_or_the_button() {
    for (money, name) in [(499, "Barracks"), (1200, "Hut")] {
        let mut w = world(money);
        let template = w.template(name);
        assert!(w.construct(template).is_none(), "{name}");
        assert_eq!(w.money(), money, "{name}");
        assert_eq!(
            dozer_task_of(&mut w.sim, w.dozer),
            Some((-1, ObjectId::INVALID))
        );
    }
}

#[test]
fn a_human_does_not_build_over_what_cannot_move() {
    let mut w = world(1200);
    let hut = w
        .sim
        .rules
        .thing_factory
        .find_template_id("Hut")
        .expect("hut");
    let at = Coord3D::new(800.0, 700.0, 0.0);
    let slot = w.sim.spawn_object(hut, TEAM, Coord3D::default(), 0.0);
    w.sim.set_object_position(slot, &at);
    w.sim.update_partition();
    let barracks = w.template("Barracks");
    let before = w.sim.world.objects.len();
    w.construct(barracks);
    assert_eq!(w.sim.world.objects.len(), before);
    assert_eq!(w.money(), 1200);
}

#[test]
fn another_players_selection_does_not_build() {
    let mut w = world(1200);
    let barracks = w.template("Barracks");
    let dozer = w.dozer;
    w.sim.world.objects.get_mut(dozer).expect("dozer").team_id = TeamId(1);
    assert!(w.construct(barracks).is_none());
    assert_eq!(w.money(), 1200);
}

#[test]
fn an_order_naming_what_the_rules_lack_is_reported() {
    let mut w = world(1200);
    assert!(w.construct(999).is_none());
    let unit = ReplayCommand::new(MSG_QUEUE_UNIT_CREATE, PLAYER)
        .with(ReplayArg::integer(998))
        .with(ReplayArg::integer(1));
    dispatch_command(&mut w.sim, &mut w.state, &unit);
    let upgrade = ReplayCommand::new(MSG_QUEUE_UPGRADE, PLAYER)
        .with(ReplayArg::object_id(w.dozer_id.0 as u32))
        .with(ReplayArg::integer(997));
    dispatch_command(&mut w.sim, &mut w.state, &upgrade);
    assert_eq!(
        w.state.unknown_names,
        [
            (MSG_DOZER_CONSTRUCT, 999),
            (MSG_QUEUE_UNIT_CREATE, 998),
            (MSG_QUEUE_UPGRADE, 997)
        ]
    );
    assert_eq!(w.money(), 1200);
}
