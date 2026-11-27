//! A move order alone takes a unit to its goal on a loaded pathfind map: the
//! movement states ask for the path, drive it and end the move (slices S9 and
//! S10; held to the reference by `golden_s9_path_follow.rs` and
//! `golden_s10_movement_states.rs`).

use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord3D, Matrix3D, Region3D};
use zh_common::types::{Bool, Real, TeamId, FALSE};
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

fn order(sim: &mut SimContext, ctx: ModuleContext, cmd: AiCommandType, points: &[Coord3D]) {
    let mut parms = AiCommandParms::new(cmd, CommandSourceType::FromPlayer);
    parms.pos = points[0];
    parms.coords = points.to_vec();
    ai_update::execute(sim, ctx, &AiCall::Command(parms));
}

fn settled() -> (SimContext, ObjectSlot, ModuleSlot, ModuleContext) {
    let (mut sim, unit) = world();
    let ms = sim.object(unit).and_then(|o| o.ai_slot).expect("AI");
    let ctx = sim.world.modules.meta(ms).context(ms);
    // Frames the idle unit settles in.
    for _ in 0..3 {
        run_frame(&mut sim, &mut NoFrameSeams);
    }
    (sim, unit, ms, ctx)
}

/// Steps frames until the unit's machine is idle again; where it was after
/// each of them.
fn run_to_idle(sim: &mut SimContext, ms: ModuleSlot, unit: ObjectSlot) -> Vec<Coord3D> {
    let mut trail = Vec::new();
    for _ in 0..900 {
        run_frame(sim, &mut NoFrameSeams);
        let pos = *sim.object(unit).expect("unit").thing.get_position();
        assert!(
            Wall.is_cliff_cell(pos.x, pos.y) == FALSE,
            "walked into the wall at {pos:?}"
        );
        trail.push(pos);
        if ai(sim, ms).machine.current_state_id() == state_id::IDLE {
            return trail;
        }
    }
    panic!("still moving at {:?}", trail.last());
}

fn rests_at(sim: &mut SimContext, ms: ModuleSlot, unit: ObjectSlot, goal: Coord3D) {
    for _ in 0..30 {
        run_frame(sim, &mut NoFrameSeams);
    }
    let pos = *sim.object(unit).expect("unit").thing.get_position();
    assert!(
        (pos.x - goal.x).abs() < 6.0 && (pos.y - goal.y).abs() < 6.0,
        "rests at {pos:?}"
    );
    assert!(ai(sim, ms).is_moving() == FALSE);
    assert!(ai(sim, ms).waiting_for_path == FALSE);
}

#[test]
fn a_move_order_gets_a_path_around_the_wall_and_drives_it() {
    let (mut sim, unit, ms, ctx) = settled();
    let goal = Coord3D::new(185.0, 45.0, 0.0);
    order(&mut sim, ctx, AiCommandType::MoveToPosition, &[goal]);
    assert_eq!(
        ai(&mut sim, ms).machine.current_state_id(),
        state_id::MOVE_TO
    );
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

    let trail = run_to_idle(&mut sim, ms, unit);
    assert!(trail.iter().any(|p| p.y >= 140.0), "went round the wall");
    rests_at(&mut sim, ms, unit, goal);
    let cell = ai(&mut sim, ms).pathfind_goal_cell;
    assert_eq!((cell.x, cell.y), (18, 4));
}

#[test]
fn an_attack_move_order_without_enemies_goes_the_same_way() {
    let (mut sim, unit, ms, ctx) = settled();
    let goal = Coord3D::new(185.0, 45.0, 0.0);
    order(&mut sim, ctx, AiCommandType::AttackMoveToPosition, &[goal]);
    assert_eq!(
        ai(&mut sim, ms).machine.current_state_id(),
        state_id::ATTACK_MOVE_TO
    );
    run_to_idle(&mut sim, ms, unit);
    rests_at(&mut sim, ms, unit, goal);
}

#[test]
fn a_followed_path_visits_each_point() {
    let (mut sim, unit, ms, ctx) = settled();
    let points = [
        Coord3D::new(45.0, 185.0, 0.0),
        Coord3D::new(185.0, 185.0, 0.0),
        Coord3D::new(185.0, 45.0, 0.0),
    ];
    order(&mut sim, ctx, AiCommandType::FollowPath, &points);
    let trail = run_to_idle(&mut sim, ms, unit);
    for point in &points {
        let passed = trail
            .iter()
            .any(|p| (p.x - point.x).abs() < 15.0 && (p.y - point.y).abs() < 15.0);
        assert!(passed, "never near {point:?}");
    }
    rests_at(&mut sim, ms, unit, points[2]);
}
