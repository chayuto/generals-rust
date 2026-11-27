//! An attack order alone makes one unit destroy another on a loaded pathfind
//! map: the attack states choose the weapon, close in, aim and fire (slice S14;
//! held to the reference by `golden_s14_attack_states.rs`).

use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::{Bool, Real, Relationship, TeamId, FALSE, TRUE};
use zh_logic::ai::ai_state_machine::state_id;
use zh_logic::arena::{ModuleSlot, ObjectSlot};
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::gamelogic::frame::{run_frame, NoFrameSeams};
use zh_logic::modules::update::ai_update::{self, AiCall, AiCommandParms, AiCommandType, AiUpdate};
use zh_logic::modules::CommandSourceType;
use zh_logic::object::weapon::{WeaponTemplate, NO_MAX_SHOTS_LIMIT};
use zh_logic::sim::{SimApi, SimContext, SimView};

const ATTACKERS: TeamId = TeamId(1);
const DEFENDERS: TeamId = TeamId(2);

fn extent() -> Region3D {
    Region3D {
        lo: Coord3D::new(0.0, 0.0, 0.0),
        hi: Coord3D::new(400.0, 400.0, 100.0),
    }
}

fn object_ini(name: &str, kind: &str, locomotor: &str, radius: Real, extra: &str) -> String {
    let mut ini = format!(
        "Object {name}\n  KindOf = {kind}\n  Geometry = CYLINDER\n  \
         GeometryMajorRadius = {radius}\n  GeometryHeight = 10.0\n  VisionRange = 150\n  \
         Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n  \
         Behavior = DestroyDie ModuleTag_Die\n  End\n"
    );
    if !locomotor.is_empty() {
        ini.push_str(&format!(
            "  Behavior = PhysicsBehavior ModuleTag_Phys\n    Mass = 5\n  End\n  \
             Behavior = AIUpdateInterface ModuleTag_AI\n  End\n  \
             Locomotor = SET_NORMAL {locomotor}\n"
        ));
    }
    ini.push_str(extra);
    ini.push_str("End\n");
    ini
}

fn world() -> SimContext {
    let mut sim = SimContext::new();
    for (name, appearance) in [("Legs", "TWO_LEGS"), ("Treads", "TREADS")] {
        let mut locomotor = LocomotorTemplate::new();
        locomotor
            .parse_ini_block(&format!(
                "Locomotor {name}\n  Surfaces = GROUND\n  Appearance = {appearance}\n  \
                 Speed = 40\n  TurnRate = 360\n  Acceleration = 120\n  Braking = 120\n  \
                 MinSpeed = 0\nEnd\n"
            ))
            .expect("locomotor");
        locomotor.name = name.to_owned();
        sim.rules.locomotor_store.add_template(locomotor);
    }
    let mut rifle = WeaponTemplate::new();
    rifle
        .parse_ini_block(
            "Weapon Rifle\n  PrimaryDamage = 15\n  PrimaryDamageRadius = 0\n  AttackRange = 80\n  \
             DelayBetweenShots = 400\n  ClipSize = 3\n  ClipReloadTime = 900\n  \
             AutoReloadsClip = Yes\n  DamageType = SMALL_ARMS\n  DeathType = NORMAL\n  \
             WeaponSpeed = 999999\n  AntiGround = Yes\nEnd\n",
        )
        .expect("weapon");
    sim.rules.weapon_store.add_template(rifle);
    let gun = "  WeaponSet\n    Conditions = None\n    Weapon = PRIMARY Rifle\n  End\n";
    let ini = object_ini("Soldier", "SELECTABLE INFANTRY CAN_ATTACK", "Legs", 4.0, gun)
        + &object_ini(
            "Gunner",
            "SELECTABLE VEHICLE CAN_ATTACK ATTACK_NEEDS_LINE_OF_SIGHT",
            "Treads",
            7.0,
            gun,
        )
        + &object_ini("Dummy", "SELECTABLE VEHICLE", "Treads", 7.0, "")
        + &object_ini("Bunker", "STRUCTURE IMMOBILE", "", 14.0, "");
    let mut reader = IniReader::new("object.ini", ini.as_bytes(), IniLoadType::Overwrite);
    sim.rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");
    sim.rules.global_data.gravity = -1.0;
    sim.rules.global_data.partition_cell_size = 20.0;
    for (player, team) in [(1, ATTACKERS), (2, DEFENDERS)] {
        sim.world.player_default_teams.insert(player, team);
        sim.world.team_players.insert(team, player);
        sim.world.player_is_human.insert(team, TRUE);
    }
    for pair in [(1, 2), (2, 1)] {
        sim.world
            .player_relationships
            .insert(pair, Relationship::Enemies);
    }
    sim.init_partition(20.0, extent());
    sim.new_pathfind_map(&extent(), 12.0);
    sim
}

fn spawn(sim: &mut SimContext, name: &str, team: TeamId, x: Real, y: Real) -> ObjectSlot {
    let template = sim
        .rules
        .thing_factory
        .find_template_id(name)
        .expect("template");
    let slot = sim.spawn_object(template, team, Coord3D::default(), 0.0);
    sim.set_object_position(slot, &Coord3D::new(x, y, 0.0));
    slot
}

fn settle(sim: &mut SimContext) {
    for _ in 0..3 {
        run_frame(sim, &mut NoFrameSeams);
    }
}

fn ai_slot(sim: &SimContext, unit: ObjectSlot) -> ModuleSlot {
    sim.object(unit).and_then(|o| o.ai_slot).expect("AI")
}

fn ai(sim: &mut SimContext, unit: ObjectSlot) -> &mut AiUpdate {
    let ms = ai_slot(sim, unit);
    sim.resident_state_mut(ms)
        .and_then(|a| a.downcast_mut::<AiUpdate>())
        .expect("AIUpdate")
}

fn order(
    sim: &mut SimContext,
    unit: ObjectSlot,
    cmd: AiCommandType,
    victim: Option<ObjectSlot>,
    pos: Coord3D,
) {
    let ms = ai_slot(sim, unit);
    let ctx = sim.world.modules.meta(ms).context(ms);
    let mut parms = AiCommandParms::new(cmd, CommandSourceType::FromPlayer);
    parms.obj = victim.and_then(|v| sim.object(v).map(|o| o.id));
    parms.pos = pos;
    parms.int_value = NO_MAX_SHOTS_LIMIT;
    ai_update::execute(sim, ctx, &AiCall::Command(parms));
}

fn position(sim: &SimContext, unit: ObjectSlot) -> Coord3D {
    *sim.object(unit).expect("unit").thing.get_position()
}

fn health(sim: &SimContext, unit: ObjectSlot) -> Option<Real> {
    sim.object(unit)
        .filter(|o| o.is_effectively_dead() == FALSE)
        .map(|_| sim.object_health(unit))
}

/// Runs frames until the victim is gone; the attacker's states along the way and
/// the victim's health after each frame it lost some.
fn fight(sim: &mut SimContext, attacker: ObjectSlot, victim: ObjectSlot) -> (Vec<u32>, usize) {
    let victim_id = sim.object(victim).expect("victim").id;
    let mut states = Vec::new();
    let mut hits = 0;
    let mut last = health(sim, victim);
    for _ in 0..900 {
        run_frame(sim, &mut NoFrameSeams);
        let machine = &ai(sim, attacker).machine;
        if let Some(attack) = &machine.attacks.main.machine {
            if states.last() != Some(&attack.current) {
                states.push(attack.current);
            }
        }
        let now = sim.find_object(victim_id).and_then(|v| health(sim, v));
        if now != last {
            hits += 1;
            last = now;
        }
        if sim.find_object(victim_id).is_none() {
            return (states, hits);
        }
    }
    panic!(
        "the victim lives with {:?}; attacker at {:?} in state {}",
        last,
        position(sim, attacker),
        ai(sim, attacker).machine.current_state_id()
    );
}

fn run_to_idle(sim: &mut SimContext, unit: ObjectSlot) {
    for _ in 0..120 {
        run_frame(sim, &mut NoFrameSeams);
        if ai(sim, unit).machine.current_state_id() == state_id::IDLE {
            return;
        }
    }
    panic!("still in state {}", ai(sim, unit).machine.current_state_id());
}

fn destroys(name: &str, from: (Real, Real), to: (Real, Real)) {
    let mut sim = world();
    let attacker = spawn(&mut sim, name, ATTACKERS, from.0, from.1);
    let victim = spawn(&mut sim, "Dummy", DEFENDERS, to.0, to.1);
    settle(&mut sim);
    let start = position(&sim, attacker);
    order(
        &mut sim,
        attacker,
        AiCommandType::AttackObject,
        Some(victim),
        Coord3D::default(),
    );
    assert_eq!(
        ai(&mut sim, attacker).machine.current_state_id(),
        state_id::ATTACK_OBJECT
    );
    assert_eq!(
        sim.ai_current_victim(attacker),
        None,
        "set by the first update"
    );
    let (states, hits) = fight(&mut sim, attacker, victim);
    // Approach (1), aim (2) and fire (3).
    for state in [1, 2, 3] {
        assert!(states.contains(&state), "states {states:?}");
    }
    assert_eq!(hits, 7, "fifteen damage a shot on a hundred health");
    run_to_idle(&mut sim, attacker);
    let end = position(&sim, attacker);
    let moved = (end.x - start.x).hypot(end.y - start.y);
    assert!(moved > 60.0, "closed in: {moved}");
    let from_victim = (end.x - to.0).hypot(end.y - to.1);
    assert!(from_victim > 40.0, "stopped at range: {from_victim}");
    assert_eq!(sim.ai_current_victim(attacker), None);
}

#[test]
fn an_attack_order_makes_a_soldier_close_in_and_destroy_its_target() {
    destroys("Soldier", (45.0, 45.0), (245.0, 125.0));
}

#[test]
fn a_vehicle_that_needs_line_of_sight_does_the_same() {
    destroys("Gunner", (45.0, 245.0), (245.0, 245.0));
}

#[test]
fn an_immobile_victim_behind_the_attacker_is_turned_to_first() {
    let mut sim = world();
    let attacker = spawn(&mut sim, "Soldier", ATTACKERS, 205.0, 205.0);
    let victim = spawn(&mut sim, "Bunker", DEFENDERS, 145.0, 205.0);
    settle(&mut sim);
    order(
        &mut sim,
        attacker,
        AiCommandType::AttackObject,
        Some(victim),
        Coord3D::default(),
    );
    fight(&mut sim, attacker, victim);
    let pos = position(&sim, attacker);
    assert!((pos.x - 205.0).abs() < 5.0, "fired from where it stood");
    let angle = sim.object(attacker).expect("unit").thing.get_orientation();
    assert!(angle.abs() > 3.0, "turned round: {angle}");
}

#[test]
fn an_attack_on_a_spot_keeps_firing_until_ordered_to_stop() {
    let mut sim = world();
    let attacker = spawn(&mut sim, "Soldier", ATTACKERS, 45.0, 345.0);
    settle(&mut sim);
    let spot = Coord3D::new(245.0, 345.0, 0.0);
    order(&mut sim, attacker, AiCommandType::AttackPosition, None, spot);
    assert_eq!(
        ai(&mut sim, attacker).machine.current_state_id(),
        state_id::ATTACK_POSITION
    );
    for _ in 0..300 {
        run_frame(&mut sim, &mut NoFrameSeams);
    }
    let pos = position(&sim, attacker);
    assert!(
        (spot.x - pos.x) < 90.0 && (spot.x - pos.x) > 50.0,
        "at {pos:?}"
    );
    assert_eq!(sim.ai_current_victim_pos(attacker), Some(spot));
    let unit = ai(&mut sim, attacker);
    assert_eq!(unit.machine.current_state_id(), state_id::ATTACK_POSITION);
    order(
        &mut sim,
        attacker,
        AiCommandType::Idle,
        None,
        Coord3D::default(),
    );
    assert_eq!(sim.ai_current_victim_pos(attacker), None);
    assert!(ai(&mut sim, attacker).machine.attacks.main.machine.is_none());
    let status: Bool = sim
        .object(attacker)
        .map_or(FALSE, |o| o.status.test(23));
    assert_eq!(status, FALSE, "no longer attacking");
}
