//! A unit ordered to move on a pathfind map asks for a path and drives it
//! (slice S9; held to the reference by `golden_s9_path_follow.rs`).
//!
//! The move states are not ported (S10): the order enters the `MOVE_TO` state,
//! which does nothing yet, so [`enter_move_state`] and the arrival check make
//! the calls that state would make.

use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord3D, Matrix3D, Region3D};
use zh_common::types::{Bool, Real, TeamId, FALSE, TRUE};
use zh_logic::ai::ai_state_machine::state_id;
use zh_logic::arena::{ModuleContext, ModuleSlot, ObjectSlot};
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::gamelogic::frame::{run_frame, NoFrameSeams};
use zh_logic::modules::update::ai_update::{self, AiCall, AiCommandParms, AiCommandType, AiUpdate};
use zh_logic::modules::CommandSourceType;
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;
use zh_logic::sim::{FlatTerrain, SimApi, SimContext, SimView};

/// Flat ground, with a wall of cliffs across x = 100..120 below y = 150.
struct Wall;

impl ThingTerrainAccess for Wall {
    fn get_ground_height(&self, _x: Real, _y: Real, _normal: Option<&mut Coord3D>) -> Real {
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
        Bool::from((100.0..120.0).contains(&x) && y < 150.0)
    }
    fn is_underwater(
        &self,
        _x: Real,
        _y: Real,
        _water_z: Option<&mut Real>,
        _terrain_z: Option<&mut Real>,
    ) -> Bool {
        FALSE
    }
}

fn extent() -> Region3D {
    Region3D {
        lo: Coord3D::new(0.0, 0.0, 0.0),
        hi: Coord3D::new(250.0, 250.0, 100.0),
    }
}

fn world() -> (SimContext, ObjectSlot) {
    let mut sim = SimContext::new();
    let mut legs = LocomotorTemplate::new();
    legs.parse_ini_block(
        "Locomotor Legs\n  Surfaces = GROUND\n  Appearance = TWO_LEGS\n  Speed = 40\n  \
         TurnRate = 360\n  Acceleration = 120\n  Braking = 120\nEnd\n",
    )
    .expect("locomotor");
    legs.name = "Legs".to_owned();
    sim.rules.locomotor_store.add_template(legs);
    let ini = "Object Walker\n  KindOf = INFANTRY\n  Geometry = CYLINDER\n  \
               GeometryMajorRadius = 4.0\n  GeometryHeight = 10.0\n  \
               Behavior = PhysicsBehavior ModuleTag_Phys\n    Mass = 5\n  End\n  \
               Behavior = AIUpdateInterface ModuleTag_AI\n  End\n  \
               Locomotor = SET_NORMAL Legs\nEnd\n";
    let mut reader = IniReader::new("object.ini", ini.as_bytes(), IniLoadType::Overwrite);
    sim.rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");
    sim.rules.global_data.gravity = -1.0;
    sim.rules.global_data.partition_cell_size = 20.0;
    sim.terrain = Box::new(Wall);
    sim.init_partition(20.0, extent());
    sim.new_pathfind_map(&extent(), 12.0);
    let template = sim
        .rules
        .thing_factory
        .find_template_id("Walker")
        .expect("Walker");
    let unit = sim.spawn_object(template, TeamId::default(), Coord3D::default(), 0.0);
    sim.set_object_position(unit, &Coord3D::new(45.0, 45.0, 0.0));
    (sim, unit)
}

fn ai(sim: &mut SimContext, ms: ModuleSlot) -> &mut AiUpdate {
    sim.resident_state_mut(ms)
        .and_then(|a| a.downcast_mut::<AiUpdate>())
        .expect("AIUpdate")
}

/// `AIInternalMoveToState::onEnter`, as far as a ground unit needs it.
fn enter_move_state(sim: &mut SimContext, ms: ModuleSlot, ctx: ModuleContext) {
    let goal = ai(sim, ms).machine.goal_position;
    ai_update::request_path(sim, ctx, &goal, TRUE);
    let unit = ai(sim, ms);
    unit.starting_move();
    unit.set_goal_position_on_path();
}

#[test]
fn a_move_order_gets_a_path_around_the_wall_and_drives_it() {
    let (mut sim, unit) = world();
    let ms = sim.object(unit).and_then(|o| o.ai_slot).expect("AI");
    let ctx = sim.world.modules.meta(ms).context(ms);
    // Frames the idle unit settles in.
    for _ in 0..3 {
        run_frame(&mut sim, &mut NoFrameSeams);
    }
    let goal = Coord3D::new(185.0, 45.0, 0.0);
    let mut parms =
        AiCommandParms::new(AiCommandType::MoveToPosition, CommandSourceType::FromPlayer);
    parms.pos = goal;
    ai_update::execute(&mut sim, ctx, &AiCall::Command(parms));
    assert_eq!(
        ai(&mut sim, ms).machine.current_state_id(),
        state_id::MOVE_TO
    );

    enter_move_state(&mut sim, ms, ctx);
    assert!(ai(&mut sim, ms).waiting_for_path != FALSE);
    run_frame(&mut sim, &mut NoFrameSeams);

    let path = ai(&mut sim, ms).path.as_ref().expect("a path");
    let mut highest: Real = 0.0;
    let mut node = path.first_node();
    while let Some(n) = node {
        highest = highest.max(path.node(n).position().y);
        node = path.node(n).next();
    }
    assert!(highest >= 145.0, "the path goes round the wall: {highest}");

    let mut furthest_y: Real = 0.0;
    let mut arrived = false;
    for _ in 0..600 {
        run_frame(&mut sim, &mut NoFrameSeams);
        let pos = *sim.object(unit).expect("unit").thing.get_position();
        assert!(
            Wall.is_cliff_cell(pos.x, pos.y) == FALSE,
            "walked into the wall at {pos:?}"
        );
        furthest_y = furthest_y.max(pos.y);
        if ai_update::locomotor_distance_to_goal(&mut sim, ctx) < 2.0 {
            ai(&mut sim, ms).ending_move();
            arrived = true;
            break;
        }
    }
    let pos = *sim.object(unit).expect("unit").thing.get_position();
    assert!(arrived, "still at {pos:?}");
    assert!(furthest_y >= 140.0, "went round the wall: {furthest_y}");
    for _ in 0..30 {
        run_frame(&mut sim, &mut NoFrameSeams);
    }
    let pos = *sim.object(unit).expect("unit").thing.get_position();
    assert!(
        (pos.x - goal.x).abs() < 6.0 && (pos.y - goal.y).abs() < 6.0,
        "rests at {pos:?}"
    );
    assert!(ai(&mut sim, ms).path.is_none());
    let cell = ai(&mut sim, ms).pathfind_goal_cell;
    assert_eq!((cell.x, cell.y), (18, 4));
}
