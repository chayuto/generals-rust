//! An attack order alone makes one unit destroy another on a loaded pathfind
//! map: the attack states choose the weapon, close in, aim and fire (slice S14;
//! held to the reference by `golden_s14_attack_states.rs`). An idle unit does it
//! unordered once an enemy is in range (`golden_s12_targeting.rs`).

use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::{Bool, ObjectId, Real, Relationship, TeamId, FALSE, TRUE};
use zh_logic::ai::ai_player::AiPlayer;
use zh_logic::ai::ai_state_machine::state_id;
use zh_logic::arena::{ModuleSlot, ObjectSlot};
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::gamelogic::frame::{run_frame, NoFrameSeams};
use zh_logic::modules::update::ai_update::{
    self, drop_team_target, offer_team_target, AiCall, AiCommandParms, AiCommandType, AiUpdate,
};
use zh_logic::modules::CommandSourceType;
use zh_logic::object::weapon::{WeaponTemplate, NO_MAX_SHOTS_LIMIT};
use zh_logic::player::seams::AiKind;
use zh_logic::rules::object_status_types::ObjectStatusType;
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
    let ini = object_ini(
        "Soldier",
        "SELECTABLE INFANTRY CAN_ATTACK",
        "Legs",
        4.0,
        gun,
    ) + &object_ini(
        "Gunner",
        "SELECTABLE VEHICLE CAN_ATTACK ATTACK_NEEDS_LINE_OF_SIGHT",
        "Treads",
        7.0,
        gun,
    ) + &object_ini("Dummy", "SELECTABLE VEHICLE", "Treads", 7.0, "")
        + &object_ini("Bunker", "STRUCTURE IMMOBILE", "", 14.0, "")
        + &object_ini("Beacon", "ALWAYS_VISIBLE IMMOBILE", "", 4.0, "");
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
    panic!(
        "still in state {}",
        ai(sim, unit).machine.current_state_id()
    );
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
    let moved = Coord3D::new(end.x - start.x, end.y - start.y, 0.0).length();
    assert!(moved > 60.0, "closed in: {moved}");
    let from_victim = Coord3D::new(end.x - to.0, end.y - to.1, 0.0).length();
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
    order(
        &mut sim,
        attacker,
        AiCommandType::AttackPosition,
        None,
        spot,
    );
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
    assert!(ai(&mut sim, attacker)
        .machine
        .attacks
        .main
        .machine
        .is_none());
    let status: Bool = sim.object(attacker).map_or(FALSE, |o| o.status.test(23));
    assert_eq!(status, FALSE, "no longer attacking");
}

/// `getNextMoodTarget` from the idle state: no order is given to the one that
/// shoots. A human's idle unit takes only what is already within weapon range.
#[test]
fn an_idle_unit_attacks_an_enemy_that_comes_into_range() {
    let mut sim = world();
    // `AIData.ini`'s `GuardOuterModifierHuman`: the share of the vision range a
    // human's unit scans. The constructor's value is zero.
    sim.rules.ai_data.guard_outer_modifier_human = 1.5;
    let ini = object_ini("Sentry", "SELECTABLE INFANTRY CAN_ATTACK", "Legs", 4.0, "")
        .replace(
            "Object Sentry",
            "Object Sentry\n  ShroudClearingRange = 150",
        )
        .replace(
            "ModuleTag_AI\n",
            "ModuleTag_AI\n    AutoAcquireEnemiesWhenIdle = Yes\n    MoodAttackCheckRate = 500\n",
        )
        .replacen(
            "  Locomotor = SET_NORMAL Legs\n",
            "  Locomotor = SET_NORMAL Legs\n  WeaponSet\n    Conditions = None\n    \
             Weapon = PRIMARY Rifle\n  End\n",
            1,
        );
    let mut reader = IniReader::new("sentry.ini", ini.as_bytes(), IniLoadType::Overwrite);
    sim.rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");
    let sentry = spawn(&mut sim, "Sentry", ATTACKERS, 45.0, 205.0);
    let enemy = spawn(&mut sim, "Dummy", DEFENDERS, 345.0, 205.0);
    for _ in 0..90 {
        run_frame(&mut sim, &mut NoFrameSeams);
    }
    assert_eq!(
        ai(&mut sim, sentry).machine.current_state_id(),
        state_id::IDLE,
        "out of range"
    );
    let start = position(&sim, sentry);
    order(
        &mut sim,
        enemy,
        AiCommandType::MoveToPosition,
        None,
        Coord3D::new(95.0, 205.0, 0.0),
    );
    let mut acquired_at = None;
    for _ in 0..600 {
        run_frame(&mut sim, &mut NoFrameSeams);
        if ai(&mut sim, sentry).machine.current_state_id() == state_id::ATTACK_OBJECT {
            acquired_at = Some(position(&sim, enemy).x - start.x);
            break;
        }
    }
    let distance = acquired_at.expect("the idle unit notices the enemy");
    assert!(
        distance > 40.0 && distance < 95.0,
        "taken inside weapon range: {distance}"
    );
    fight(&mut sim, sentry, enemy);
}

/// What the sim answers the targeting search about walls, fog and a team that
/// attacks a common target, asked outside the idle state.
#[test]
fn the_mood_search_heeds_walls_fog_and_the_teams_target() {
    let mut sim = world();
    sim.rules.ai_data.guard_outer_modifier_human = 1.5;
    sim.rules.ai_data.guard_outer_modifier_ai = 1.5;
    let gunner = spawn(&mut sim, "Gunner", ATTACKERS, 105.0, 205.0);
    let enemy = spawn(&mut sim, "Dummy", DEFENDERS, 175.0, 205.0);
    let far = spawn(&mut sim, "Dummy", DEFENDERS, 385.0, 25.0);
    settle(&mut sim);
    let id = |sim: &SimContext, slot| sim.object(slot).expect("object").id;
    let (enemy_id, far_id) = (id(&sim, enemy), id(&sim, far));
    let ms = ai_slot(&sim, gunner);
    let ctx = sim.world.modules.meta(ms).context(ms);
    let ask = |sim: &mut SimContext, by_ai| ai_update::next_mood_target(sim, ctx, by_ai, FALSE);

    assert_eq!(sim.object_is_clear_of_shroud(far, 1), FALSE);
    let beacon = spawn(&mut sim, "Beacon", DEFENDERS, 385.0, 385.0);
    settle(&mut sim);
    assert_eq!(sim.object_is_clear_of_shroud(beacon, 1), TRUE);
    sim.world.partition.unregister(far_id);
    assert_eq!(sim.object_is_clear_of_shroud(far, 1), TRUE);
    sim.world.partition.register(far_id);
    settle(&mut sim);
    assert_eq!(ask(&mut sim, FALSE), Some(enemy_id));
    sim.rules.ai_data.attack_uses_line_of_sight = TRUE;
    assert_eq!(ask(&mut sim, FALSE), Some(enemy_id), "nothing between");
    let wall = spawn(&mut sim, "Bunker", DEFENDERS, 140.0, 205.0);
    sim.add_object_to_pathfind_map(wall);
    settle(&mut sim);
    assert_ne!(ask(&mut sim, FALSE), Some(enemy_id), "behind the wall");
    sim.remove_object_from_pathfind_map(wall);
    sim.destroy_object(wall);
    settle(&mut sim);

    // A computer team that is not easy keeps the target it first fires at.
    sim.world.player_is_human.insert(ATTACKERS, FALSE);
    sim.world
        .ai_players
        .insert(1, AiPlayer::new(AiKind::Skirmish, 1, 0, 1, 10));
    offer_team_target(&mut sim, gunner, far);
    assert_eq!(sim.team_common_target(gunner), None, "no common target");
    sim.world
        .team_common_targets
        .insert(ATTACKERS, ObjectId::INVALID);
    sim.world.ai_players.get_mut(&1).expect("ai").difficulty = 0;
    offer_team_target(&mut sim, gunner, far);
    assert_eq!(sim.team_common_target(gunner), Some(ObjectId::INVALID));
    sim.world.ai_players.get_mut(&1).expect("ai").difficulty = 1;
    offer_team_target(&mut sim, gunner, far);
    offer_team_target(&mut sim, gunner, enemy);
    assert_eq!(sim.team_common_target(gunner), Some(far_id), "the first");
    // Out of range, still the team's.
    assert_eq!(ask(&mut sim, TRUE), Some(far_id));
    drop_team_target(&mut sim, gunner, enemy);
    assert_eq!(sim.team_common_target(gunner), Some(far_id));
    drop_team_target(&mut sim, gunner, far);
    assert_eq!(sim.team_common_target(gunner), Some(ObjectId::INVALID));
    // A stealthed one is forgotten unless detected, whatever is asked about.
    offer_team_target(&mut sim, gunner, far);
    sim.set_status(far, ObjectStatusType::Stealthed);
    sim.set_status(far, ObjectStatusType::Detected);
    drop_team_target(&mut sim, gunner, enemy);
    assert_eq!(sim.team_common_target(gunner), Some(far_id));
    sim.clear_status(far, ObjectStatusType::Detected);
    drop_team_target(&mut sim, gunner, enemy);
    assert_eq!(sim.team_common_target(gunner), Some(ObjectId::INVALID));
    sim.clear_status(far, ObjectStatusType::Stealthed);
    // A dead target is forgotten when asked for.
    offer_team_target(&mut sim, gunner, far);
    sim.object_mut(far).expect("far").set_effectively_dead(TRUE);
    assert_ne!(ask(&mut sim, TRUE), Some(far_id));
    assert_eq!(sim.team_common_target(gunner), Some(ObjectId::INVALID));
}

fn add_turret_tank(sim: &mut SimContext) {
    let ini = object_ini(
        "Turreted",
        "SELECTABLE VEHICLE CAN_ATTACK",
        "Treads",
        7.0,
        "  WeaponSet\n    Conditions = None\n    Weapon = PRIMARY Rifle\n  End\n",
    )
    .replace(
        "ModuleTag_AI\n",
        "ModuleTag_AI\n    Turret\n      TurretTurnRate = 180\n      \
         ControlledWeaponSlots = PRIMARY\n      RecenterTime = 300\n      \
         MinIdleScanInterval = 300\n      MaxIdleScanInterval = 600\n      \
         MinIdleScanAngle = 30\n      MaxIdleScanAngle = 60\n    End\n",
    );
    let mut reader = IniReader::new("turret.ini", ini.as_bytes(), IniLoadType::Overwrite);
    sim.rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");
}

fn turret_angle(sim: &mut SimContext, unit: ObjectSlot) -> Real {
    ai(sim, unit).turrets[0].as_ref().expect("turret").angle
}

/// The turret looks around while idle, then turns to the victim behind and fires
/// while the body stays put (held to the reference by the turret trace of
/// `golden_s14_attack_states.rs`).
#[test]
fn a_turreted_vehicle_scans_when_idle_and_aims_with_its_turret() {
    let mut sim = world();
    add_turret_tank(&mut sim);
    let tank = spawn(&mut sim, "Turreted", ATTACKERS, 205.0, 105.0);
    let mut turned = false;
    let mut returned = false;
    for _ in 0..200 {
        run_frame(&mut sim, &mut NoFrameSeams);
        let angle = turret_angle(&mut sim, tank);
        turned |= angle.abs() > 0.3;
        returned |= turned && angle == 0.0;
    }
    assert!(turned && returned, "scanned {turned}, recentred {returned}");

    let victim = spawn(&mut sim, "Dummy", DEFENDERS, 145.0, 105.0);
    settle(&mut sim);
    let heading = sim.object(tank).expect("tank").thing.get_orientation();
    order(
        &mut sim,
        tank,
        AiCommandType::AttackObject,
        Some(victim),
        Coord3D::default(),
    );
    let mut widest: Real = 0.0;
    for _ in 0..300 {
        run_frame(&mut sim, &mut NoFrameSeams);
        widest = widest.max(turret_angle(&mut sim, tank).abs());
        if health(&sim, victim).is_none() {
            break;
        }
    }
    assert_eq!(health(&sim, victim), None, "destroyed");
    assert!(widest > 3.0, "the turret turned round: {widest}");
    let body = sim.object(tank).expect("tank").thing.get_orientation();
    assert_eq!(body, heading, "the body did not turn");
    let pos = position(&sim, tank);
    assert!((pos.x - 205.0).abs() < 5.0, "fired from where it stood");
}
