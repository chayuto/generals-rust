//! A resume-construction order alone makes a dozer, and a worker, drive to a
//! building under construction and finish it; a bored dozer repairs a damaged
//! one unasked (slice S16, held to the reference by
//! `golden_s16_dozer_worker_frames.rs`).

use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::{Real, TeamId, FALSE, TRUE};
use zh_logic::arena::{ModuleContext, ModuleSlot, ObjectSlot};
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::gamelogic::frame::{run_frame, NoFrameSeams};
use zh_logic::gamelogic::global_data::BodyDamageType;
use zh_logic::modules::update::ai_update::{
    self, AiCall, AiCommandParms, AiCommandType, AiUpdate, DerivedAi,
};
use zh_logic::modules::update::dozer_ai_update::{DozerBrain, DOZER_TASK_INVALID};
use zh_logic::modules::CommandSourceType;
use zh_logic::rules::object_status_types::{ObjectStatusMaskType, ObjectStatusType};
use zh_logic::sim::{SimApi, SimContext, SimView};

const TEAM: TeamId = TeamId(1);
const BUILD_FRAMES: Real = 90.0;

fn world() -> SimContext {
    let mut sim = SimContext::new();
    let mut wheels = LocomotorTemplate::new();
    wheels
        .parse_ini_block(
            "Locomotor Wheels\n  Surfaces = GROUND\n  Appearance = TREADS\n  Speed = 60\n  \
             TurnRate = 360\n  Acceleration = 120\n  Braking = 120\nEnd\n",
        )
        .expect("locomotor");
    wheels.name = "Wheels".to_owned();
    sim.rules.locomotor_store.add_template(wheels);
    let builder = |name: &str, module: &str| {
        format!(
            "Object {name}\n  KindOf = VEHICLE DOZER\n  Geometry = CYLINDER\n  \
             GeometryMajorRadius = 6.0\n  GeometryHeight = 8.0\n  \
             Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n  \
             Behavior = PhysicsBehavior ModuleTag_Phys\n    Mass = 5\n  End\n  \
             Behavior = {module} ModuleTag_AI\n    RepairHealthPercentPerSecond = 20%\n    \
             BoredTime = 2000\n    BoredRange = 120\n  End\n  \
             Locomotor = SET_NORMAL Wheels\nEnd\n"
        )
    };
    let ini = builder("Dozer", "DozerAIUpdate")
        + &builder("Worker", "WorkerAIUpdate")
        + "Object Barracks\n  KindOf = STRUCTURE IMMOBILE SCORE\n  Geometry = BOX\n  \
           GeometryMajorRadius = 20.0\n  GeometryMinorRadius = 14.0\n  GeometryHeight = 10.0\n  \
           GeometryIsSmall = No\n  BuildTime = 3.0\n  \
           Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n\
           End\n";
    let mut reader = IniReader::new("object.ini", ini.as_bytes(), IniLoadType::Overwrite);
    sim.rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");
    sim.rules.global_data.gravity = -1.0;
    // As the game's data: only a badly damaged unit moves slower.
    sim.rules.global_data.movement_penalty_damage_state = BodyDamageType::ReallyDamaged;
    sim.rules.global_data.partition_cell_size = 20.0;
    // A player with neither power plants nor consumers counts as out of power.
    sim.rules.global_data.max_low_energy_production_speed = 1.0;
    let extent = Region3D {
        lo: Coord3D::new(0.0, 0.0, 0.0),
        hi: Coord3D::new(300.0, 300.0, 100.0),
    };
    sim.init_partition(20.0, extent);
    sim.new_pathfind_map(&extent, 0.0);
    sim.world.team_players.insert(TEAM, 1);
    sim.world.player_default_teams.insert(1, TEAM);
    sim.world.player_is_human.insert(TEAM, TRUE);
    sim.world
        .player_relationships
        .insert((1, 1), zh_common::types::Relationship::Allies);
    sim
}

fn spawn(sim: &mut SimContext, name: &str, x: Real, y: Real, unbuilt: bool) -> ObjectSlot {
    let template = sim
        .rules
        .thing_factory
        .find_template_id(name)
        .expect("template");
    let mut status = ObjectStatusMaskType::new();
    if unbuilt {
        status.set(ObjectStatusType::UnderConstruction as i32, 1);
    }
    let slot = sim.factory_new_object_with_status(template, TEAM, status);
    sim.set_object_position(slot, &Coord3D::new(x, y, 0.0));
    if name == "Barracks" {
        sim.add_object_to_pathfind_map(slot);
    }
    if unbuilt {
        sim.set_object_construction_percent(slot, 0.0);
        let health = sim.object_health(slot);
        sim.body_internal_change_health(slot, 1.0 - health);
    }
    slot
}

fn ai_of(sim: &SimContext, unit: ObjectSlot) -> (ModuleSlot, ModuleContext) {
    let ms = sim.object(unit).and_then(|o| o.ai_slot).expect("AI");
    (ms, sim.world.modules.meta(ms).context(ms))
}

fn brain(sim: &mut SimContext, ms: ModuleSlot) -> DozerBrain {
    let ai = sim
        .resident_state_mut(ms)
        .and_then(|a| a.downcast_mut::<AiUpdate>())
        .expect("AIUpdate");
    match &ai.derived {
        DerivedAi::Dozer(d) => d.brain.clone(),
        DerivedAi::Worker(w) => w.brain.clone(),
        DerivedAi::Base => panic!("not a builder"),
    }
}

fn under_construction(sim: &SimContext, building: ObjectSlot) -> bool {
    sim.object(building)
        .is_some_and(|o| o.status.test(ObjectStatusType::UnderConstruction as i32) != FALSE)
}

fn builds_to_completion(builder: &str) {
    let mut sim = world();
    let building = spawn(&mut sim, "Barracks", 200.0, 200.0, true);
    let unit = spawn(&mut sim, builder, 60.0, 80.0, false);
    run_frame(&mut sim, &mut NoFrameSeams);
    let (ms, ctx) = ai_of(&sim, unit);
    let mut parms = AiCommandParms::new(
        AiCommandType::ResumeConstruction,
        CommandSourceType::FromPlayer,
    );
    parms.obj = sim.object(building).map(|o| o.id);
    ai_update::execute(&mut sim, ctx, &AiCall::Command(parms));
    assert_eq!(
        brain(&mut sim, ms).tasks[0].target,
        parms_target(&sim, building)
    );

    let mut working = 0;
    let mut last = 0.0;
    let mut frames = 0;
    while under_construction(&sim, building) {
        run_frame(&mut sim, &mut NoFrameSeams);
        let percent = sim.object_construction_percent(building);
        if under_construction(&sim, building) {
            assert!(percent >= last, "construction went back");
            if percent > last {
                working += 1;
                last = percent;
            }
        }
        frames += 1;
        assert!(frames < 1500, "not built, at {percent} percent");
    }
    // One step of 100 / frames per working frame, the last one completing.
    assert!(
        (working as Real - BUILD_FRAMES).abs() <= 2.0,
        "{working} working frames"
    );
    assert_eq!(sim.object_construction_percent(building), -1.0);
    assert_eq!(sim.object_health(building), 200.0);
    let id = sim.object(unit).expect("builder").id;
    assert_eq!(sim.object(building).expect("building").producer_id, id);
    assert_eq!(sim.world.score_keepers[&1].total_buildings_built, 1);
    let state = brain(&mut sim, ms);
    assert_eq!(state.current_task, DOZER_TASK_INVALID);
    assert_eq!(state.tasks[0].target.0, 0);
}

fn parms_target(sim: &SimContext, building: ObjectSlot) -> zh_common::types::ObjectId {
    sim.object(building).expect("building").id
}

#[test]
fn a_dozer_drives_to_a_building_and_finishes_it() {
    builds_to_completion("Dozer");
}

#[test]
fn a_worker_drives_to_a_building_and_finishes_it() {
    builds_to_completion("Worker");
}

#[test]
fn a_bored_dozer_repairs_a_damaged_building() {
    let mut sim = world();
    let building = spawn(&mut sim, "Barracks", 150.0, 150.0, false);
    spawn(&mut sim, "Dozer", 90.0, 150.0, false);
    sim.body_internal_change_health(building, -80.0);
    // Nothing happens before the bored time of two seconds has passed.
    for _ in 0..60 {
        run_frame(&mut sim, &mut NoFrameSeams);
    }
    assert_eq!(sim.object_health(building), 120.0);
    for _ in 0..600 {
        run_frame(&mut sim, &mut NoFrameSeams);
    }
    assert_eq!(sim.object_health(building), 200.0);
}
