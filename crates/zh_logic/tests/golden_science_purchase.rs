//! `Player::attemptToPurchaseScience` against the unmodified reference:
//! `docs/golden_vectors/SciencePurchase.json`
//! (`tests/cpp_harness/gen_science_purchase.cpp`). Every row holds each player's
//! points, academy count and sciences, and the ready frame of every special power.

use zh_common::ini::{IniLoadType, IniReader};
use zh_common::io::recorder::{ReplayArg, ReplayCommand};
use zh_common::math::Coord3D;
use zh_common::runtime::game_logic_dispatch::MSG_PURCHASE_SCIENCE;
use zh_common::types::{Int, ObjectId, TeamId, UnsignedInt, UnsignedShort};
use zh_common::xfer::{Xfer, XferError};
use zh_logic::arena::{FacetMask, ModuleSlot, ObjectSlot};
use zh_logic::gamelogic::command_target::{dispatch_game_command, CommandGame, CommandState};
use zh_logic::player::player_list::PlayerList;
use zh_logic::player::science_purchase::attempt_to_purchase_science;
use zh_logic::player::seams::*;
use zh_logic::player::team::TeamFactory;
use zh_logic::rules::player_template::PlayerTemplateStore;
use zh_logic::rules::rank_info::RankInfoStore;
use zh_logic::rules::science::{ScienceTextLookup, ScienceType, SCIENCE_INVALID};
use zh_logic::rules::upgrade::UpgradeMaskType;
use zh_logic::scripts::script_engine::ScriptEngine;
use zh_logic::sim::{SimApi, SimContext, SimView, SimWorld};
use zh_logic::spatial::sides_list::SidesList;

const GOLDEN: &str = include_str!("../../../docs/golden_vectors/SciencePurchase.json");
const NAMES: [&str; 6] = [
    "SCI_A",
    "SCI_B",
    "SCI_C",
    "SCI_Free",
    "SCI_Locked",
    "SCI_Dear",
];

/// A player list outside a game: nothing it asks for exists.
struct NullHost;

impl PlayerWorld for NullHost {
    fn frame(&self) -> UnsignedInt {
        0
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
        0
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
    fn create_ai(&mut self, _player: PlayerIndex, _kind: AiKind) {}
    fn delete_ai(&mut self, _player: PlayerIndex) {}
    fn set_ai_difficulty(&mut self, _player: PlayerIndex, _difficulty: Int) {}
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
    fn upgrade_count(&self, _player: PlayerIndex) -> UnsignedShort {
        0
    }
    fn delete_upgrades(&mut self, _player: PlayerIndex) {}
    fn upgrade_masks(&self) -> Vec<(String, UpgradeMaskType)> {
        Vec::new()
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
        None
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

fn lines() -> Vec<String> {
    GOLDEN
        .lines()
        .filter_map(|l| {
            let l = l.trim().trim_end_matches(',').strip_prefix('"')?;
            Some(l.strip_suffix('"')?.replace("\\n", "\n"))
        })
        .collect()
}

fn header(lines: &[String], name: &str) -> String {
    lines
        .iter()
        .find_map(|l| l.strip_prefix(&format!("{name} ")))
        .expect(name)
        .to_owned()
}

fn reader(text: &str) -> IniReader<'_> {
    IniReader::new("science.ini", text.as_bytes(), IniLoadType::Overwrite)
}

struct NoText;

impl ScienceTextLookup for NoText {
    fn fetch_label(&mut self, _label: &str) -> String {
        String::new()
    }
}

struct Game {
    sim: SimContext,
    scripts: ScriptEngine,
    modules: Vec<Vec<ModuleSlot>>,
}

fn new_game(lines: &[String]) -> Game {
    let mut sim = SimContext::new();
    let (sciences, powers, objects) = (
        header(lines, "sciences"),
        header(lines, "powers"),
        header(lines, "objects"),
    );
    sim.rules
        .science_store
        .load_from_ini(
            &mut reader(&sciences),
            &mut sim.rules.name_keys,
            &mut NoText,
        )
        .expect("sciences");
    sim.rules
        .special_power_store
        .load_from_ini(
            &mut reader(&powers),
            &sim.rules.science_store,
            &mut sim.rules.name_keys,
        )
        .expect("powers");
    sim.rules
        .load_thing_templates_from_ini(&mut reader(&objects))
        .expect("objects");
    sim.rules.thing_factory.post_process_load();
    sim.world = SimWorld::new();
    sim.world.frame = 1;
    let templates = PlayerTemplateStore::new();
    let ranks = RankInfoStore::new();
    let mut host = NullHost;
    let players = PlayerList::new(&mut PlayerEnv {
        templates: &templates,
        ranks: &ranks,
        sciences: &sim.rules.science_store,
        name_keys: &mut sim.rules.name_keys,
        world: &mut host,
    });
    sim.world.players = players;
    for player in [1, 2] {
        sim.world
            .player_default_teams
            .insert(player, TeamId(player));
        sim.add_player_team(TeamId(player), player);
    }
    let slots: Vec<ObjectSlot> = [("G_Centre", 1), ("G_Other", 1), ("G_Centre", 2)]
        .into_iter()
        .map(|(name, team)| {
            let template = sim.rules.thing_factory.find_template_id(name).expect(name);
            sim.factory_new_object(template, TeamId(team))
        })
        .collect();
    let modules = slots
        .iter()
        .map(|&slot| {
            let object = sim.world.objects.get(slot).expect("object");
            object
                .behaviors
                .iter()
                .filter(|(_, facets)| facets.contains(FacetMask::SPECIAL_POWER))
                .map(|&(module, _)| module)
                .collect()
        })
        .collect();
    Game {
        sim,
        scripts: ScriptEngine::new(),
        modules,
    }
}

impl Game {
    fn science(&mut self, index: Int) -> ScienceType {
        usize::try_from(index).map_or(SCIENCE_INVALID, |i| {
            self.sim
                .rules
                .science_store
                .get_science_from_internal_name(NAMES[i], &mut self.sim.rules.name_keys)
        })
    }

    fn state(&mut self) -> String {
        let mut out = String::new();
        for p in [1, 2] {
            let spent = self
                .sim
                .world
                .player_academy
                .get(&p)
                .map_or(0, |a| a.generals_points_spent);
            let player = self.sim.world.players.player(p).expect("player");
            let (points, held) = (player.science_purchase_points, player.sciences.clone());
            out.push_str(&format!(" {points}/{spent}/"));
            for science in held {
                for i in 0..NAMES.len() {
                    if self.science(i as Int) == science {
                        out.push_str(&i.to_string());
                    }
                }
            }
        }
        for object in &self.modules {
            out.push(' ');
            for &module in object {
                let ready = self
                    .sim
                    .dispatch_module(module, |m, sim, ctx| {
                        m.as_special_power()
                            .expect("power")
                            .get_ready_frame(sim, ctx)
                    })
                    .expect("module");
                out.push_str(&format!("{ready},"));
            }
        }
        out
    }
}

#[test]
fn science_purchase_matches_reference() {
    let lines = lines();
    let mut game = new_game(&lines);
    let start = lines
        .iter()
        .find_map(|l| l.strip_prefix("start"))
        .expect("start");
    assert_eq!(game.state(), start, "start");
    let mut rows = 0;
    let mut bought = 0;
    for line in lines.iter().filter_map(|l| l.strip_prefix("op ")) {
        let mut words = line.splitn(5, ' ');
        let mut next = || -> i64 { words.next().expect("word").parse().expect("number") };
        let (code, p, a, frame) = (next(), next() as Int + 1, next() as Int, next());
        let rest = words.next().expect("state");
        let (ret, state) = rest.split_once(' ').map_or((rest, ""), |(r, s)| (r, s));
        game.sim.world.frame = frame as u32;
        let mut got = false;
        match code {
            0 => {
                for q in [1, 2] {
                    game.sim.world.players.player_mut(q).expect("player").sciences.clear();
                    for i in 0..NAMES.len() {
                        let science = game.science(i as Int);
                        let player = game.sim.world.players.player_mut(q).expect("player");
                        player.set_science_availability(science, 0);
                    }
                }
            }
            1 => {
                game.sim.world.players
                    .player_mut(p)
                    .expect("player")
                    .science_purchase_points = a
            }
            2 => {
                let science = game.science(a / 3);
                let player = game.sim.world.players.player_mut(p).expect("player");
                player.set_science_availability(science, a % 3);
            }
            // The frame column already holds the new frame.
            3 => {}
            _ => {
                let science = game.science(a);
                got = attempt_to_purchase_science(
                    &mut game.sim,
                    Some(&mut game.scripts),
                    p,
                    science,
                );
                if got {
                    bought += 1;
                    assert!(game.scripts.acquired_sciences[p as usize].ends_with(&[science]));
                }
            }
        }
        assert_eq!(u8::from(got).to_string(), ret, "{line}");
        assert_eq!(game.state(), format!(" {state}"), "{line}");
        rows += 1;
    }
    assert_eq!(rows, 400);
    assert!(bought > 30, "{bought}");
    let heard: usize = game.scripts.acquired_sciences.iter().map(Vec::len).sum();
    assert_eq!(heard, bought);
}

/// The replay's order reaches the player that sent it.
#[test]
fn the_recorded_order_buys_the_science() {
    let lines = lines();
    let mut game = new_game(&lines);
    let science = game.science(0);
    game.sim.world.players
        .player_mut(1)
        .expect("player")
        .science_purchase_points = 2;
    let message = ReplayCommand::new(MSG_PURCHASE_SCIENCE, 1).with(ReplayArg::integer(science));
    let mut state = CommandState::default();
    let parts = CommandGame {
        scripts: &mut game.scripts,
    };
    dispatch_game_command(&mut game.sim, parts, &mut state, &message);
    let player = game.sim.world.players.player(1).expect("player");
    assert_eq!(player.science_purchase_points, 1);
    assert_eq!(player.sciences, [science]);
    assert!(state.not_carried_out.is_empty());
    assert!(state.unknown_names.is_empty());
}

/// Every team of the player reads the player's science, the others none. No
/// player, no purchase.
#[test]
fn every_team_of_the_player_holds_the_science() {
    let lines = lines();
    let mut game = new_game(&lines);
    let science = game.science(0);
    game.sim.add_player_team(TeamId(7), 1);
    game.sim.world.players
        .player_mut(1)
        .expect("player")
        .science_purchase_points = 2;
    let buy = |game: &mut Game, index| {
        attempt_to_purchase_science(&mut game.sim, None, index, science)
    };
    assert!(!buy(&mut game, 99));
    assert!(buy(&mut game, 1));
    let holders: Vec<TeamId> = [1, 2, 7]
        .map(TeamId)
        .into_iter()
        .filter(|&team| game.sim.player_has_science(team, science) != 0)
        .collect();
    assert_eq!(holders, [TeamId(1), TeamId(7)]);
}
