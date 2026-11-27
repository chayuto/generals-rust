//! Orders that end in the S6 searches, on a loaded pathfind map with the game
//! stepping. What the searches answer is held to the reference by
//! `golden_s6_other_searches.rs` and `golden_s6_attack_path.rs`; here it is
//! that orders reach them.

use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{gm_sqrtf, Coord3D, ICoord2D, Region3D};
use zh_common::types::{ObjectId, Relationship, TeamId, TRUE};
use zh_logic::ai::ai_group;
use zh_logic::ai::ai_state_machine::state_id;
use zh_logic::arena::{ModuleSlot, ObjectSlot};
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::gamelogic::frame::{run_frame, NoFrameSeams};
use zh_logic::modules::update::ai_update::{self, AiCall, AiCommandParms, AiCommandType, AiUpdate};
use zh_logic::modules::CommandSourceType;
use zh_logic::pathfinder::PathfindLayerEnum;
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::sim::{SimApi, SimContext, SimView};

const TEAM: TeamId = TeamId(1);

fn extent() -> Region3D {
    Region3D {
        lo: Coord3D::default(),
        hi: Coord3D::new(300.0, 300.0, 100.0),
    }
}

fn world() -> SimContext {
    let mut sim = SimContext::new();
    let mut legs = LocomotorTemplate::new();
    legs.parse_ini_block(
        "Locomotor Legs\n  Surfaces = GROUND\n  Appearance = TWO_LEGS\n  Speed = 30\n  \
         TurnRate = 360\n  Acceleration = 90\n  Braking = 90\nEnd\n",
    )
    .expect("locomotor");
    legs.name = "Legs".to_owned();
    sim.rules.locomotor_store.add_template(legs);
    let ini = "Object Walker\n  KindOf = SELECTABLE INFANTRY\n  Geometry = CYLINDER\n  \
               GeometryMajorRadius = 4\n  GeometryHeight = 8\n  \
               Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  \
               End\n  Behavior = PhysicsBehavior ModuleTag_Phys\n    Mass = 5\n  End\n  \
               Behavior = AIUpdateInterface ModuleTag_AI\n  End\n  \
               Locomotor = SET_NORMAL Legs\nEnd\n\
               Object Dozer\n  KindOf = SELECTABLE VEHICLE DOZER\n  Geometry = CYLINDER\n  \
               GeometryMajorRadius = 8\n  GeometryHeight = 8\n  \
               Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  \
               End\n  Behavior = PhysicsBehavior ModuleTag_Phys\n    Mass = 5\n  End\n  \
               Behavior = AIUpdateInterface ModuleTag_AI\n  End\n  \
               Locomotor = SET_NORMAL Legs\nEnd\n\
               Object Block\n  KindOf = STRUCTURE IMMOBILE\n  Geometry = CYLINDER\n  \
               GeometryMajorRadius = 25\n  GeometryHeight = 8\nEnd\n";
    let mut reader = IniReader::new("object.ini", ini.as_bytes(), IniLoadType::Overwrite);
    sim.rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");
    sim.init_partition(20.0, extent());
    sim.world.team_players.insert(TEAM, 0);
    sim.world.player_default_teams.insert(0, TEAM);
    sim.world.player_is_human.insert(TEAM, TRUE);
    sim.world
        .player_relationships
        .insert((0, 0), Relationship::Allies);
    sim.world.frame = 100;
    sim.new_pathfind_map(&extent(), 0.0);
    sim
}

fn spawn(sim: &mut SimContext, name: &str, x: f32, y: f32) -> ObjectSlot {
    let template = sim
        .rules
        .thing_factory
        .find_template_id(name)
        .expect("template");
    let slot = sim.spawn_object(template, TEAM, Coord3D::default(), 0.0);
    sim.set_object_position(slot, &Coord3D::new(x, y, 0.0));
    if name == "Block" {
        sim.add_object_to_pathfind_map(slot);
    }
    slot
}

fn module(sim: &SimContext, slot: ObjectSlot) -> ModuleSlot {
    sim.object(slot).and_then(|o| o.ai_slot).expect("AI")
}

fn state(sim: &mut SimContext, slot: ObjectSlot) -> &mut AiUpdate {
    let module = module(sim, slot);
    sim.resident_state_mut(module)
        .and_then(|a| a.downcast_mut::<AiUpdate>())
        .expect("AIUpdate")
}

fn order(sim: &mut SimContext, slot: ObjectSlot, cmd: AiCommandType, pos: Coord3D) {
    let module = module(sim, slot);
    let ctx = sim.world.modules.meta(module).context(module);
    let mut parms = AiCommandParms::new(cmd, CommandSourceType::FromPlayer);
    parms.pos = pos;
    ai_update::execute(sim, ctx, &AiCall::Command(parms));
}

fn position(sim: &SimContext, slot: ObjectSlot) -> Coord3D {
    *sim.object(slot).expect("object").thing.get_position()
}

fn distance(a: Coord3D, b: Coord3D) -> f32 {
    let (dx, dy) = (a.x - b.x, a.y - b.y);
    gm_sqrtf(dx * dx + dy * dy)
}

fn path_end(sim: &mut SimContext, slot: ObjectSlot) -> Option<Coord3D> {
    let path = state(sim, slot).path.as_ref()?;
    path.last_node().map(|n| *path.node(n).position())
}

/// Steps the game, keeping the end of the first path `slot` gets.
fn step(sim: &mut SimContext, slot: ObjectSlot, frames: u32) -> Option<Coord3D> {
    let mut end = None;
    for _ in 0..frames {
        run_frame(sim, &mut NoFrameSeams);
        end = end.or_else(|| path_end(sim, slot));
    }
    end
}

/// `findPath` has no answer for a goal inside a building: `findClosestPath`
/// takes the unit as near as it can stand.
#[test]
fn a_blocked_goal_gets_the_closest_path() {
    let mut sim = world();
    spawn(&mut sim, "Block", 205.0, 105.0);
    let unit = spawn(&mut sim, "Walker", 45.0, 105.0);
    let goal = Coord3D::new(205.0, 105.0, 0.0);
    order(&mut sim, unit, AiCommandType::MoveToPosition, goal);
    let end = step(&mut sim, unit, 10).expect("a path");
    let short = distance(end, goal);
    assert!(
        short > 20.0 && short < 50.0,
        "path ends {short} from the goal"
    );
}

/// A tighten order asks for an approach path, which is a closest path.
#[test]
fn a_tighten_order_gets_a_path() {
    let mut sim = world();
    let unit = spawn(&mut sim, "Walker", 45.0, 105.0);
    let goal = Coord3D::new(205.0, 105.0, 0.0);
    order(&mut sim, unit, AiCommandType::TightenToPosition, goal);
    assert_eq!(
        state(&mut sim, unit).machine.current_state_id(),
        state_id::MOVE_AND_TIGHTEN
    );
    let end = step(&mut sim, unit, 10).expect("a path");
    assert!(distance(end, goal) < 15.0, "path ends at {end:?}");
    step(&mut sim, unit, 300);
    assert!(distance(position(&sim, unit), goal) < 15.0);
}

/// Idle units in the way of a new path are asked to leave it, and each finds its
/// own way off. The line of them leaves no way round, and a dozer asks even
/// where its path is not blocked.
#[test]
fn idle_allies_step_aside() {
    let mut sim = world();
    let mover = spawn(&mut sim, "Dozer", 45.0, 105.0);
    let line: Vec<ObjectSlot> = (0..30)
        .map(|i| spawn(&mut sim, "Walker", 125.0, 5.0 + 10.0 * i as f32))
        .collect();
    step(&mut sim, mover, 2);
    order(
        &mut sim,
        mover,
        AiCommandType::MoveToPosition,
        Coord3D::new(205.0, 105.0, 0.0),
    );
    let mut asked = None;
    for _ in 0..10 {
        run_frame(&mut sim, &mut NoFrameSeams);
        asked = asked.or_else(|| {
            line.iter().copied().find(|&unit| {
                state(&mut sim, unit).machine.temporary == state_id::MOVE_OUT_OF_THE_WAY
                    && path_end(&mut sim, unit).is_some()
            })
        });
    }
    let idle = asked.expect("a move-away path");
    let mover_id = sim.object(mover).expect("mover").id;
    assert_eq!(state(&mut sim, idle).move_out_of_way1, mover_id);
}

/// A group order drops its members' goals from the map, not through the seam.
#[test]
fn a_group_order_removes_goals_from_the_map() {
    let mut sim = world();
    let unit = spawn(&mut sim, "Walker", 45.0, 105.0);
    let id: ObjectId = sim.object(unit).expect("object").id;
    let goal = Coord3D::new(105.0, 105.0, 0.0);
    sim.pathfinder_update_goal(unit, &goal, PathfindLayerEnum::Ground);
    assert_eq!(
        state(&mut sim, unit).pathfind_goal_cell,
        ICoord2D { x: 10, y: 10 }
    );
    // A disabled unit ignores the move, so only the group touches its goal.
    sim.set_disabled(unit, DisabledType::Emp);
    let group = ai_group::create_group(&mut sim);
    ai_group::add(&mut sim, group, id);
    ai_group::group_scatter(&mut sim, group, CommandSourceType::FromPlayer);
    assert_eq!(
        state(&mut sim, unit).pathfind_goal_cell,
        ICoord2D { x: -1, y: -1 }
    );
}
