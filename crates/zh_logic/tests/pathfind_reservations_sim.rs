//! How the sim feeds the reservation half of the pathfinder
//! (`sim/pathfind_map.rs`). The map logic itself is held to the reference by
//! `golden_s4_reservations.rs`.

use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord3D, ICoord2D, Region3D};
use zh_common::types::{TeamId, FALSE, TRUE};
use zh_logic::arena::ObjectSlot;
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::modules::update::ai_update::AiUpdate;
use zh_logic::pathfinder::PathfindLayerEnum;
use zh_logic::sim::{SimApi, SimContext, SimView};

fn world() -> (SimContext, Vec<ObjectSlot>) {
    let mut sim = SimContext::new();
    let mut legs = LocomotorTemplate::new();
    legs.parse_ini_block("Locomotor Legs\n  Surfaces = GROUND\n  Appearance = TWO_LEGS\nEnd\n")
        .expect("locomotor");
    legs.name = "Legs".to_owned();
    sim.rules.locomotor_store.add_template(legs);
    // Radius 5 gives a footprint of one cell, centred in it.
    let ini = "Object Walker\n  KindOf = VEHICLE\n  Geometry = CYLINDER\n  \
               GeometryMajorRadius = 5.0\n  GeometryHeight = 10.0\n  \
               Behavior = AIUpdateInterface ModuleTag_AI\n  End\n  \
               Locomotor = SET_NORMAL Legs\nEnd\n";
    let mut reader = IniReader::new("object.ini", ini.as_bytes(), IniLoadType::Overwrite);
    sim.rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");
    let template = sim
        .rules
        .thing_factory
        .find_template_id("Walker")
        .expect("Walker");
    let units: Vec<ObjectSlot> = (0..2)
        .map(|i| {
            let at = Coord3D::new(25.0, 25.0 + 100.0 * i as f32, 0.0);
            sim.spawn_object(template, TeamId::default(), at, 0.0)
        })
        .collect();
    (sim, units)
}

fn map(sim: &mut SimContext) {
    let extent = Region3D {
        lo: Coord3D::default(),
        hi: Coord3D::new(200.0, 200.0, 100.0),
    };
    sim.new_pathfind_map(&extent, 12.0);
    assert!(sim
        .world
        .pathfinder
        .as_mut()
        .expect("map")
        .begin_path_queue(&extent, 100));
}

fn cells(sim: &mut SimContext, slot: ObjectSlot) -> (ICoord2D, ICoord2D) {
    let module = sim.object(slot).and_then(|o| o.ai_slot).expect("AI");
    let ai = sim
        .resident_state_mut(module)
        .and_then(|a| a.downcast_mut::<AiUpdate>())
        .expect("AIUpdate");
    (ai.pathfind_goal_cell, ai.pathfind_cur_cell)
}

fn holders(sim: &SimContext, x: i32, y: i32) -> (i32, i32) {
    let pf = sim.world.pathfinder.as_ref().expect("map");
    let cell = pf.get_cell(PathfindLayerEnum::Ground, x, y).expect("cell");
    (pf.goal_unit(cell).0, pf.unit_present(cell).0)
}

#[test]
fn goals_move_a_destination_aside() {
    let (mut sim, units) = world();
    map(&mut sim);
    let (first, second) = (units[0], units[1]);
    let first_id = sim.world.objects.id_of(first).expect("id").0;
    let goal = Coord3D::new(105.0, 105.0, 0.0);

    let mut dest = goal;
    assert_eq!(sim.pathfinder_adjust_destination(first, &mut dest), TRUE);
    assert_eq!(dest, Coord3D::new(105.0, 105.0, 0.0), "free cell");

    sim.pathfinder_update_goal(first, &goal, PathfindLayerEnum::Ground);
    assert_eq!(cells(&mut sim, first).0, ICoord2D::new(10, 10));
    assert_eq!(holders(&sim, 10, 10), (first_id, 0));

    // The holder keeps its cell, another unit gets the next one of the spiral.
    let mut own = goal;
    assert_eq!(sim.pathfinder_adjust_destination(first, &mut own), TRUE);
    assert_eq!(own, goal);
    let mut other = goal;
    assert_eq!(sim.pathfinder_adjust_destination(second, &mut other), TRUE);
    assert_eq!(other, Coord3D::new(115.0, 105.0, 0.0));

    let mut possible = goal;
    assert_eq!(
        sim.pathfinder_adjust_to_possible_destination(second, &mut possible),
        TRUE
    );
    assert_eq!(possible, Coord3D::new(115.0, 105.0, 0.0));

    sim.remove_unit_from_pathfind_map(first);
    assert_eq!(holders(&sim, 10, 10), (0, 0));
    assert_eq!(cells(&mut sim, first).0, ICoord2D::new(-1, -1));
}

#[test]
fn only_people_are_held_to_the_playable_area() {
    let (mut sim, units) = world();
    map(&mut sim);
    let person = sim.object(units[0]).expect("unit").team_id;
    sim.world.set_team_human(person, zh_common::types::TRUE);
    let playable = Region3D {
        lo: Coord3D::default(),
        hi: Coord3D::new(100.0, 100.0, 100.0),
    };
    let pathfinder = sim.world.pathfinder.as_mut().expect("map");
    assert!(pathfinder.begin_path_queue(&playable, 100));
    let beyond = Coord3D::new(155.0, 55.0, 0.0);

    let mut dest = beyond;
    assert_eq!(sim.pathfinder_adjust_destination(units[0], &mut dest), TRUE);
    assert!(
        dest.x <= 105.0,
        "a person's unit is brought inside: {dest:?}"
    );

    let team = sim.object(units[0]).expect("unit").team_id;
    sim.world.set_team_human(team, FALSE);
    let mut dest = beyond;
    assert_eq!(sim.pathfinder_adjust_destination(units[0], &mut dest), TRUE);
    assert_eq!(dest, beyond);
}

#[test]
fn positions_are_written_and_removed() {
    let (mut sim, units) = world();
    map(&mut sim);
    let id = sim.world.objects.id_of(units[0]).expect("id").0;
    sim.update_pathfind_position(units[0]);
    assert_eq!(cells(&mut sim, units[0]).1, ICoord2D::new(2, 2));
    assert_eq!(holders(&sim, 2, 2), (0, id));
    sim.remove_unit_from_pathfind_map(units[0]);
    assert_eq!(holders(&sim, 2, 2), (0, 0));
}

#[test]
fn without_a_map_nothing_is_reserved() {
    let (mut sim, units) = world();
    let goal = Coord3D::new(105.0, 105.0, 0.0);
    sim.pathfinder_update_goal(units[0], &goal, PathfindLayerEnum::Ground);
    assert_eq!(cells(&mut sim, units[0]).0, ICoord2D::new(-1, -1));
    sim.update_pathfind_position(units[0]);
    assert_eq!(cells(&mut sim, units[0]).1, ICoord2D::new(-1, -1));
}
