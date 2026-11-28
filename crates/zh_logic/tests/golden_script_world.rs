//! The adapter between the script engine and the running game
//! (`scripts/script_world.rs`), and `getAiEnemy`. What the engine does with the
//! answers is `golden_script_engine.rs`; how the first frames compare with the
//! reference is `golden_game_frames.rs`. The expected values here are read from
//! the reference source, not recorded from it.
//!
//! The retail test needs what `golden_game_start.rs` needs and passes without
//! checking when it is missing.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;

use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::io::file::OsDiskFileHost;
use zh_common::math::Coord3D;
use zh_common::types::{Int, ObjectId, Relationship, TeamId};
use zh_logic::ai::ai_player::AiPlayer;
use zh_logic::ai::ai_skirmish_player::{get_ai_enemy, EnemyView};
use zh_logic::arena::ObjectSlot;
use zh_logic::gamelogic::game_engine::{mount_game_archives, GameDataLoader};
use zh_logic::gamelogic::game_logic::{start_new_game_observed, GameStart};
use zh_logic::modules::update::ai_update::AiUpdate;
use zh_logic::modules::update::supply_warehouse_dock_update::SupplyWarehouseDockUpdate;
use zh_logic::player::seams::AiKind;
use zh_logic::rules::kind_of::KindOfType;
use zh_logic::scripts::script_types::{action_type as at, condition_type as ct};
use zh_logic::scripts::{
    script_template_registry, OrCondition, Parameter, Script, ScriptAction, ScriptEngine,
    ScriptGame, ScriptList, ScriptWorld,
};
use zh_logic::sim::{SimApi, SimView};

// The start comparison uses all of it; this one only part.
#[allow(dead_code)]
#[path = "game_start_support/mod.rs"]
mod support;

use support::{replay_parms, retail_dirs};

// ---- `getAiEnemy` ------------------------------------------------------------

#[derive(Default)]
struct Players {
    count: Int,
    enemies: Vec<(Int, Int)>,
    /// Players without any object, without units, without build facility.
    gone: Vec<Int>,
    no_units: Vec<Int>,
    no_facility: Vec<Int>,
    structures: BTreeMap<Int, Vec<Coord3D>>,
    asked: RefCell<Vec<Int>>, // determinism-ok: test spy behind a &self trait
}

impl EnemyView for Players {
    fn player_count(&self) -> Int {
        self.count
    }
    fn is_enemy(&self, player: Int, other: Int) -> bool {
        self.enemies.contains(&(player, other))
    }
    fn has_any_objects(&self, player: Int) -> bool {
        !self.gone.contains(&player)
    }
    fn has_any_units(&self, player: Int) -> bool {
        !self.no_units.contains(&player)
    }
    fn has_any_build_facility(&self, player: Int) -> bool {
        !self.no_facility.contains(&player)
    }
    fn structure_positions(&self, player: Int) -> Vec<Coord3D> {
        self.asked.borrow_mut().push(player);
        self.structures.get(&player).cloned().unwrap_or_default()
    }
}

fn at_point(x: f32, y: f32) -> Coord3D {
    Coord3D { x, y, z: 0.0 }
}

fn skirmish(player: Int, base: Coord3D) -> AiPlayer {
    let mut ai = AiPlayer::new(AiKind::Skirmish, player, 0, 1, 1);
    ai.base_center = base;
    ai
}

/// Player 1 is a skirmish AI at the origin; 2 and 3 are its enemies.
fn two_enemies(second: Coord3D, third: Coord3D) -> (Players, BTreeMap<Int, AiPlayer>) {
    let mut view = Players {
        count: 5,
        enemies: vec![(1, 2), (1, 3)],
        ..Players::default()
    };
    // The middle of the bounds counts, not the mean.
    view.structures.insert(
        2,
        vec![second, second, at_point(second.x + 20.0, second.y + 20.0)],
    );
    view.structures.insert(3, vec![third]);
    let ais = BTreeMap::from([(1, skirmish(1, at_point(0.0, 0.0)))]);
    (view, ais)
}

#[test]
fn ai_enemy_is_the_nearest_and_is_kept_while_healthy() {
    // 2's middle is (90, 10), nearer than 3 at (0, 95); its mean would not be.
    let (mut view, mut ais) = two_enemies(at_point(80.0, 0.0), at_point(0.0, 95.0));
    assert_eq!(get_ai_enemy(&mut ais, &view, 7, 1), Some(2));
    assert_eq!(ais[&1].skirmish.as_ref().unwrap().frame_to_check_enemy, 157);

    // Nothing is looked at again before frame 157.
    view.no_units = vec![2];
    view.asked.borrow_mut().clear();
    assert_eq!(get_ai_enemy(&mut ais, &view, 156, 1), Some(2));
    assert!(view.asked.borrow().is_empty());
    // In bad shape, it is left for the healthy one.
    assert_eq!(get_ai_enemy(&mut ais, &view, 157, 1), Some(3));

    // A healthy enemy is kept even when another is nearer now.
    view.no_units.clear();
    view.asked.borrow_mut().clear();
    assert_eq!(get_ai_enemy(&mut ais, &view, 400, 1), Some(3));
    assert!(view.asked.borrow().is_empty());
    // Without a build facility it is in bad shape too.
    view.no_facility = vec![3];
    assert_eq!(get_ai_enemy(&mut ais, &view, 600, 1), Some(2));
}

#[test]
fn ai_enemy_skips_the_gone_and_takes_the_unhealthy_only_last() {
    let (mut view, mut ais) = two_enemies(at_point(10.0, 0.0), at_point(900.0, 0.0));
    view.no_units = vec![2];
    assert_eq!(get_ai_enemy(&mut ais, &view, 0, 1), Some(3));

    let (mut view, mut ais) = two_enemies(at_point(10.0, 0.0), at_point(900.0, 0.0));
    view.gone = vec![2];
    assert_eq!(get_ai_enemy(&mut ais, &view, 0, 1), Some(3));

    // Only unhealthy ones: the first keeps the shared bad distance.
    let (mut view, mut ais) = two_enemies(at_point(900.0, 0.0), at_point(10.0, 0.0));
    view.no_units = vec![2, 3];
    assert_eq!(get_ai_enemy(&mut ais, &view, 0, 1), Some(2));
}

#[test]
fn ai_enemy_spreads_the_skirmish_players() {
    // Player 4 already fights 2, which then counts 500 units farther.
    let (mut view, mut ais) = two_enemies(at_point(100.0, 0.0), at_point(400.0, 0.0));
    view.enemies.push((4, 2));
    ais.insert(4, skirmish(4, at_point(0.0, 0.0)));
    assert_eq!(get_ai_enemy(&mut ais, &view, 0, 1), Some(3));
    // Asking made player 4 choose.
    assert_eq!(ais[&4].skirmish.as_ref().unwrap().current_enemy, Some(2));

    // Player 4 fighting player 1 brings 4 itself 25 units nearer.
    let (mut view, mut ais) = two_enemies(at_point(100.0, 0.0), at_point(0.0, 200.0));
    view.enemies = vec![(1, 2), (1, 4), (4, 1)];
    view.structures.insert(4, vec![at_point(99.0, -20.0)]);
    ais.insert(4, skirmish(4, at_point(0.0, 0.0)));
    // 2: (110, 10), 12200 less 625. 4: 99^2 + 20^2 = 10201.
    assert_eq!(get_ai_enemy(&mut ais, &view, 0, 1), Some(4));

    // A player without a skirmish AI has none.
    assert_eq!(get_ai_enemy(&mut ais, &view, 0, 2), None);
    ais.insert(2, AiPlayer::new(AiKind::Standard, 2, 0, 1, 1));
    assert_eq!(get_ai_enemy(&mut ais, &view, 0, 2), None);
}

#[test]
fn ai_enemy_looks_at_every_player_and_both_axes() {
    // Player 0 can be the enemy, and the height of the bounds counts as the width.
    let mut view = Players {
        count: 3,
        enemies: vec![(1, 0), (1, 2)],
        ..Players::default()
    };
    view.structures
        .insert(0, vec![at_point(0.0, 100.0), at_point(0.0, 300.0)]);
    view.structures
        .insert(2, vec![at_point(0.0, 0.0), at_point(0.0, 420.0)]);
    let mut ais = BTreeMap::from([(1, skirmish(1, at_point(0.0, 0.0)))]);
    assert_eq!(get_ai_enemy(&mut ais, &view, 0, 1), Some(0));

    // Player 0 fighting an enemy counts against it too.
    let (mut view, mut ais) = two_enemies(at_point(100.0, 0.0), at_point(400.0, 0.0));
    view.enemies.push((0, 2));
    ais.insert(0, skirmish(0, at_point(0.0, 0.0)));
    assert_eq!(get_ai_enemy(&mut ais, &view, 0, 1), Some(3));

    // The nearness of a player fighting back is 25 squared, taken off the others.
    let (mut view, mut ais) = two_enemies(at_point(93.0, 0.0), at_point(0.0, 900.0));
    view.enemies = vec![(1, 2), (1, 4), (4, 1)];
    view.structures.insert(4, vec![at_point(99.0, -20.0)]);
    ais.insert(4, skirmish(4, at_point(0.0, 0.0)));
    // 2, at (103, 10), is 10709 less 625 against the 10201 of player 4.
    assert_eq!(get_ai_enemy(&mut ais, &view, 0, 1), Some(2));
}

// ---- the adapter over a started game -----------------------------------------

fn start() -> Option<GameStart> {
    let (game_dir, user_dir) = retail_dirs()?;
    let fs = mount_game_archives(&OsDiskFileHost, &game_dir);
    let mut loader = GameDataLoader::new();
    loader.load_startup(&fs);
    let replay = PathBuf::from(&user_dir)
        .join("Replays")
        .join("macos_2p_vanilla_map.rep");
    let parms = replay_parms(&fs, &loader, &replay, &game_dir, &user_dir)?;
    Some(start_new_game_observed(loader, parms, |_, _, _, _| {}).expect("start"))
}

fn with<R>(
    game: &mut GameStart,
    f: impl FnOnce(&mut ScriptEngine, &mut dyn ScriptWorld) -> R,
) -> R {
    let parts = ScriptGame {
        teams: &mut game.teams,
        stores: &game.stores,
        waypoints: &game.waypoints,
        named_objects: &mut game.named_objects,
    };
    game.scripts.with_world(&mut game.sim, parts, f)
}

fn update(game: &mut GameStart) {
    let parts = ScriptGame {
        teams: &mut game.teams,
        stores: &game.stores,
        waypoints: &game.waypoints,
        named_objects: &mut game.named_objects,
    };
    game.scripts.update(&mut game.sim, parts);
}

fn word(value: &str) -> Parameter {
    let mut p = Parameter::new(zh_logic::scripts::ParameterType::TextString, 0);
    p.string_val = value.to_owned();
    p
}

fn number(value: Int) -> Parameter {
    Parameter::new(zh_logic::scripts::ParameterType::Int, value)
}

fn action(kind: Int, parms: Vec<Parameter>) -> ScriptAction {
    let mut names = NameKeyGenerator::new();
    let mut a = ScriptAction::new(kind, &script_template_registry(&mut names));
    a.parms = parms;
    a
}

fn run(game: &mut GameStart, kind: Int, parms: Vec<Parameter>) {
    with(game, |engine, world| {
        engine.execute_action(world, &mut action(kind, parms))
    });
}

/// A script list of one script that always runs `actions`.
fn always(actions: Vec<ScriptAction>) -> ScriptList {
    let mut names = NameKeyGenerator::new();
    let registry = script_template_registry(&mut names);
    let mut or = OrCondition::new();
    or.conditions.push(zh_logic::scripts::Condition::new(
        ct::CONDITION_TRUE,
        &registry,
    ));
    let mut script = Script::new();
    script.or_conditions.push(or);
    script.actions = actions;
    let mut list = ScriptList::new();
    list.scripts.push(script);
    list
}

fn objects_of(game: &GameStart, player: Int) -> Vec<(ObjectId, ObjectSlot)> {
    let world = &game.sim.world;
    world
        .objects
        .world_iter()
        .filter_map(|slot| {
            let object = world.objects.get(slot)?;
            (game.teams.controlling_player(object.team_id) == Some(player))
                .then_some((object.id, slot))
        })
        .collect()
}

fn is_kind(game: &GameStart, slot: ObjectSlot, kind: KindOfType) -> bool {
    game.sim
        .world
        .objects
        .get(slot)
        .and_then(|o| {
            o.thing
                .template_override()
                .get_final(&game.sim.rules.thing_factory)
        })
        .is_some_and(|t| t.is_kind_of(kind) != 0)
}

fn template_name(game: &GameStart, slot: ObjectSlot) -> String {
    game.sim
        .world
        .objects
        .get(slot)
        .and_then(|o| {
            o.thing
                .template_override()
                .get_final(&game.sim.rules.thing_factory)
        })
        .map_or_else(String::new, |t| t.get_name().to_owned())
}

#[test]
fn retail_script_world_answers_and_acts() {
    let Some(mut game) = start() else {
        eprintln!("skipped: the game is not installed");
        return;
    };
    // The start gives the engine its lists and delays; nothing is left out.
    assert!(game.scripts.engine.sides.iter().flatten().count() >= 2);
    assert!(game.scripts.unported.is_empty());
    let named = |game: &GameStart, name: &str| {
        let key = game.sim.rules.name_keys.lookup_key(name).expect("key");
        game.sim.world.players.find_player_with_name_key(key).expect("player")
    };
    let human = named(&game, "player0");
    let computer = named(&game, "player1");
    let home = game.sim.world.players.player(human).unwrap().default_team.unwrap();
    let home_name = game.teams.prototype_of(home).unwrap().name.clone();
    queries(&mut game, human, computer, home, &home_name);
    more_queries(&mut game, human, computer);
    actions(&mut game, human, computer, &home_name);
    updates(&mut game, human, computer);
}

fn queries(game: &mut GameStart, human: Int, computer: Int, home: TeamId, home_name: &str) {
    let mine = objects_of(game, human);
    let structures = mine
        .iter()
        .filter(|&&(_, s)| is_kind(game, s, KindOfType::Structure))
        .count() as Int;
    assert!(structures > 0 && mine.len() as Int > structures);
    let (building, building_slot) = *mine
        .iter()
        .find(|&&(_, s)| is_kind(game, s, KindOfType::Structure))
        .unwrap();
    let building_name = template_name(game, building_slot);
    let health = game.sim.object_health(building_slot);
    let members: Vec<ObjectId> = game.sim.world.team_members[&home]
        .iter()
        .filter_map(|&s| game.sim.world.objects.id_of(s))
        .collect();
    let money = game.sim.world.player_money[&human];
    let at = *game
        .sim
        .world
        .objects
        .get(building_slot)
        .unwrap()
        .thing
        .get_position();
    game.named_objects.push(("hq".to_owned(), building));
    let waypoint = game.waypoints[0].clone();
    let human_name = game.sim.world.players.player(human).unwrap().name.clone();

    let list_order: Vec<ObjectId> = game
        .sim
        .world
        .objects
        .world_iter()
        .filter_map(|s| game.sim.world.objects.id_of(s))
        .collect();

    with(game, |_, world| {
        assert_eq!(world.find_player("player0"), Some(human));
        assert_eq!(world.find_player("nobody"), None);
        assert!(world.player_is_human(human) && !world.player_is_human(computer));
        assert!(world.player_is_skirmish_ai(computer));
        assert!(!world.player_is_skirmish_ai(human));
        assert!(world.player_is_active(human) && world.player_is_active(computer));
        assert!(world.player_difficulty(computer).is_some());
        assert!(world.player_difficulty(human).is_none());
        assert_eq!(world.player_current_enemy(computer), Some(human));
        assert_eq!(world.player_current_enemy(human), None);
        assert_eq!(world.player_money(human), money);
        assert_eq!(world.player_name(human), human_name);
        assert_eq!(world.player_default_team(human), Some(home.0 as u32));
        assert_eq!(world.player_count_buildings(human), structures);
        assert!(world.player_count_faction_buildings(human) <= structures);
        assert!(world.player_has_any_objects(human));
        assert!(world.player_has_any_build_facility(human));
        assert_eq!(
            world.player_count_objects_by_template(
                human,
                &[building_name.clone(), "NoSuchThing".to_owned()],
                false
            ),
            [1, 0]
        );

        let team = home.0 as u32;
        let info = world.team_prototype(home_name).expect("prototype");
        assert_eq!(info.instances, [team]);
        assert_eq!(world.team_name(team), home_name);
        assert!(world.team_exists(team) && !world.team_exists(100_000));
        assert_eq!(world.team_controlling_player(team), Some(human));
        assert_eq!(world.team_members(team), members);
        assert!(world.team_has_any_objects(team) && world.team_has_any_units(team));

        // The map's names and ours, in object list order.
        let names = world.named_objects();
        assert!(names.contains(&("hq".to_owned(), building)));
        let ids: Vec<ObjectId> = names.iter().map(|n| n.1).collect();
        let listed: Vec<ObjectId> = list_order
            .iter()
            .copied()
            .filter(|id| ids.contains(id))
            .collect();
        assert_eq!(ids, listed);
        assert!(world.object_exists(building));
        assert_eq!(world.object_name(building), "hq");
        assert!(!world.object_is_effectively_dead(building));
        assert_eq!(world.object_controlling_player(building), Some(human));
        let (now, initial) = world.object_health(building).expect("body");
        assert_eq!(now.to_bits(), health.to_bits());
        assert!(initial > 0.0);

        assert!(world.has_thing_template(&building_name));
        let index = world.thing_template_index(&building_name).expect("index");
        assert_eq!(world.thing_template_name(index), building_name);
        let kind = KindOfType::Structure.as_i32();
        assert!(world.thing_templates_of_kind(kind).contains(&index));
        assert_eq!(world.waypoint_location(&waypoint.0), Some(waypoint.1));
        assert!(at.x > 0.0);
    });
    assert!(
        game.scripts.unported.is_empty(),
        "{:?}",
        game.scripts.unported
    );
}

fn more_queries(game: &mut GameStart, human: Int, computer: Int) {
    let players = game.sim.world.players.player_count();
    let facility: Vec<bool> = (0..players)
        .map(|p| {
            objects_of(game, p).iter().any(|&(_, slot)| {
                game.sim
                    .world
                    .objects
                    .get(slot)
                    .and_then(|o| {
                        o.thing
                            .template_override()
                            .get_final(&game.sim.rules.thing_factory)
                    })
                    .is_some_and(|t| t.is_build_facility() != 0)
            })
        })
        .collect();
    let start_index = game.sim.world.players.player(computer).unwrap().mp_start_index;
    let frame = game.sim.world.frame;
    let energy = game.sim.world.player_energy[&human].clone();
    let (id, slot) = objects_of(game, human)
        .into_iter()
        .find(|&(_, s)| !is_kind(game, s, KindOfType::Structure))
        .expect("a unit");
    let name = template_name(game, slot);
    let team = game.sim.world.objects.get(slot).unwrap().team_id;
    game.sim.player_of_team_mut(team).expect("player").attacked_by[computer as usize] = true;
    let neutral = game.sim.world.players.player(0).unwrap().default_team.unwrap();
    let neutral_members = game.sim.world.team_members[&neutral].len();
    assert!(neutral_members > 0);
    game.teams.team_mut(neutral).unwrap().members.clear();

    let alive = with(game, |_, world| {
        for (p, &expected) in facility.iter().enumerate() {
            assert_eq!(
                world.player_has_any_build_facility(p as Int),
                expected,
                "{p}"
            );
        }
        assert_eq!(world.player_start_index(computer), start_index);
        assert_eq!(world.player_start_index(99), 0);
        assert_eq!(world.player_money(99), 0);
        let power = world.player_power(human);
        assert_eq!(power.production, energy.get_production(frame));
        assert_eq!(power.consumption, energy.get_consumption());
        assert_eq!(power.sufficient, energy.has_sufficient_power(frame) != 0);
        assert!(world.player_attacked_by(human, computer));
        assert!(!world.player_attacked_by(computer, human));
        assert_eq!(world.waypoint_location("no such waypoint"), None);
        assert_eq!(world.science_from_name("SCIENCE_NoSuchThing"), None);
        world.update_team_states();
        world.player_count_objects_by_template(human, std::slice::from_ref(&name), true)
    });
    assert_eq!(
        game.teams.team(neutral).unwrap().members.len(),
        neutral_members
    );

    // A dead member counts only when the dead are asked for.
    let object = game.sim.world.objects.get_mut(slot).unwrap();
    object.set_effectively_dead(1);
    with(game, |_, world| {
        assert!(world.object_is_effectively_dead(id));
        let names = std::slice::from_ref(&name);
        let living = world.player_count_objects_by_template(human, names, true);
        let all = world.player_count_objects_by_template(human, names, false);
        assert_eq!((living[0], all[0]), (alive[0] - 1, alive[0]));
    });
    game.sim
        .world
        .objects
        .get_mut(slot)
        .unwrap()
        .set_effectively_dead(0);

    // Only a grantable science is granted, and only once.
    let science = with(game, |_, world| {
        world.science_from_name("SCIENCE_PaladinTank")
    })
    .expect("science");
    let grantable = game.sim.rules.science_store.is_science_grantable(science);
    with(game, |_, world| {
        assert_eq!(world.science_name(science), "SCIENCE_PaladinTank");
        assert_eq!(world.grant_science(human, science), grantable);
        assert!(!world.grant_science(human, science));
    });
    assert_eq!(
        game.sim.world.players.player(human).unwrap().has_science(science),
        grantable
    );
    assert!(
        game.scripts.unported.is_empty(),
        "{:?}",
        game.scripts.unported
    );
}

fn actions(game: &mut GameStart, human: Int, computer: Int, home_name: &str) {
    let name = word("player0");
    run(game, at::PLAYER_SET_MONEY, vec![name.clone(), number(500)]);
    assert_eq!(game.sim.world.player_money[&human], 500);
    run(game, at::PLAYER_GIVE_MONEY, vec![name.clone(), number(70)]);
    assert_eq!(game.sim.world.player_money[&human], 570);
    run(
        game,
        at::PLAYER_GIVE_MONEY,
        vec![name.clone(), number(-600)],
    );
    assert_eq!(game.sim.world.player_money[&human], 0);

    with(game, |_, world| {
        world.set_player_relationship(human, computer, Relationship::Allies as Int);
        world.set_can_build_base(human, false);
        world.set_can_build_units(human, false);
    });
    assert_eq!(
        game.sim.world.player_relationships[&(human, computer)],
        Relationship::Allies
    );
    let player = game.sim.world.players.player(human).unwrap();
    assert!(!player.can_build_base && !player.can_build_units);

    // The set is named on the team and given to its members' AI.
    run(
        game,
        at::SET_DEFAULT_ATTACK_PRIORITY,
        vec![word("rush"), number(5)],
    );
    run(
        game,
        at::TEAM_APPLY_ATTACK_PRIORITY_SET,
        vec![word(home_name), word("rush")],
    );
    let home = game.sim.world.players.player(human).unwrap().default_team.unwrap();
    assert_eq!(
        game.teams.prototype_of(home).unwrap().attack_priority_name,
        "rush"
    );
    let given: Vec<String> = game.sim.world.team_members[&home]
        .clone()
        .into_iter()
        .filter_map(|slot| game.sim.world.objects.get(slot)?.ai_slot)
        .filter_map(|ai| {
            let module = game.sim.world.modules.payload(ai)?;
            Some(
                module
                    .as_any()?
                    .downcast_ref::<AiUpdate>()?
                    .attack_info
                    .clone(),
            )
        })
        .collect();
    assert!(!given.is_empty() && given.iter().all(|n| n == "rush"));
    // An unknown set clears the members and leaves the team's name.
    run(
        game,
        at::TEAM_APPLY_ATTACK_PRIORITY_SET,
        vec![word(home_name), word("unknown")],
    );
    assert_eq!(
        game.teams.prototype_of(home).unwrap().attack_priority_name,
        "rush"
    );

    // The first supply warehouse of the map.
    let slots: Vec<ObjectSlot> = game.sim.world.objects.world_iter().collect();
    let boxes = |game: &GameStart, slot: ObjectSlot| {
        let object = game.sim.world.objects.get(slot)?;
        object.behaviors.iter().find_map(|&(module, _)| {
            let module = game.sim.world.modules.payload(module)?;
            Some(
                module
                    .as_any()?
                    .downcast_ref::<SupplyWarehouseDockUpdate>()?
                    .boxes_stored,
            )
        })
    };
    let depot = slots
        .into_iter()
        .find(|&slot| boxes(game, slot).is_some())
        .expect("a warehouse");
    let id = game.sim.world.objects.id_of(depot).unwrap();
    with(game, |engine, _| engine.add_object_to_cache("depot", id));
    let per_box = game.sim.rules.global_data.base_value_per_supply_box as Int;
    run(
        game,
        at::WAREHOUSE_SET_VALUE,
        vec![word("depot"), number(per_box * 3 + 1)],
    );
    assert_eq!(boxes(game, depot), Some(4));
}

fn updates(game: &mut GameStart, human: Int, computer: Int) {
    // Each side's list runs once a frame, for that side.
    let sides = game.sim.world.players.player_count() as usize;
    game.scripts.engine.sides = vec![None; sides];
    game.scripts.engine.sides[human as usize] = Some(always(vec![action(
        at::PLAYER_GIVE_MONEY,
        vec![word("<This Player>"), number(3)],
    )]));
    let money = |game: &GameStart, p: Int| game.sim.world.player_money[&p];
    let before = (money(game, human), money(game, computer));
    game.sim.world.frame = 1;
    update(game);
    assert_eq!(money(game, human), before.0 + 3);
    assert_eq!(money(game, computer), before.1);
    assert_eq!(game.scripts.engine.frame_object_count_changed, 0);

    // A named object that is gone by the next frame was reported with the frame
    // it went in.
    let (id, slot) = objects_of(game, human)
        .into_iter()
        .find(|&(_, s)| !is_kind(game, s, KindOfType::Structure))
        .expect("a unit");
    game.named_objects.push(("scout".to_owned(), id));
    with(game, |engine, _| engine.add_object_to_cache("scout", id));
    game.sim.world.frame = 5;
    update(game);
    assert_eq!(game.scripts.engine.frame_object_count_changed, 0);
    game.sim.destroy_object(slot);
    game.sim.process_destroy_list();
    game.sim.world.frame = 9;
    update(game);
    assert_eq!(game.scripts.engine.frame_object_count_changed, 8);
    assert_eq!(game.scripts.engine.unit_named("scout"), None);
    assert!(game.scripts.engine.did_unit_exist("scout"));

    // Unported calls are counted by name, and their answers are the empty ones.
    let before = game.scripts.unported.clone();
    with(game, |_, world| {
        world.kill_player(human);
        world.kill_object(id);
        world.delete_team(1, false);
        world.kill_team(1);
        world.object_ai_guard_here(id, true);
        world.copy_indicator_color(id, id);
        world.set_all_difficulty_bonuses(true);
        world.do_shroud_reveal(at_point(0.0, 0.0), 1.0, human);
        world.undo_shroud_reveal(at_point(0.0, 0.0), 1.0, human);
        world.team_group_idle(1);
        assert!(!world.team_group_is_idle(1));
        assert!(!world.team_group_is_ai_dead(1));
        assert_eq!(world.audio_length_ms("x").to_bits(), 0);
        assert!(!world.music_track_completed("x", 1));
        let parameter = number(0);
        assert!(!world.command_button_ready(&parameter, &parameter, true));
        assert!(!world.team_is_contained(&parameter, true));
    });
    assert_eq!(game.scripts.unported["Object::kill"], 1);
    let new: Vec<&str> = game
        .scripts
        .unported
        .iter()
        .filter(|(name, count)| before.get(*name) != Some(count))
        .map(|(&name, _)| name)
        .collect();
    assert_eq!(
        new,
        [
            "AIGroup::groupIdle",
            "AIGroup::isGroupAiDead",
            "AIGroup::isIdle",
            "AudioManager::getAudioLengthMS",
            "AudioManager::hasMusicTrackCompleted",
            "Object::kill",
            "Object::setCustomIndicatorColor",
            "Object::setReceivingDifficultyBonus",
            "PartitionManager::doShroudReveal",
            "PartitionManager::undoShroudReveal",
            "Player::killPlayer",
            "Team::deleteTeam",
            "Team::killTeam",
            "aiGuardPosition",
            "evaluateSkirmishCommandButtonIsReady",
            "evaluateTeamIsContained",
        ]
    );
}
