//! The running game behind the script engine: the [`ScriptWorld`] over the sim,
//! the players and the teams, and what `ScriptEngine` is told between frames.
//! `PKT-433`; the reference reaches all of it through its singletons.
//! Golden test: `tests/golden_script_world.rs`.
//!
//! Anything the adapter cannot do is counted by name in [`GameScripts::unported`].
//! Conditions and actions the engine does not port are counted by the engine
//! (`ScriptEngine::unported_conditions` and `unported_actions`); an unported
//! condition answers false.
//!
//! Known differences:
//! - the reference interns a team or player name when a script looks it up
//!   (`NameKeyGenerator::nameToKey`); the lookups here do not;
//! - team creation, idle, sighting and destroyed scripts run inside
//!   `updateTeamStates` there; here they are counted as unported.

use std::cell::RefCell;
use std::collections::BTreeMap;

use zh_common::math::{Coord2D, Coord3D};
use zh_common::types::{
    Int, ObjectId, Real, Relationship, TeamId, UnsignedInt, UnsignedShort, FALSE,
};
use zh_common::xfer::{Xfer, XferError};

use super::script_engine::{
    PowerReading, ScriptEngine, ScriptPlayer, ScriptTeamId, ScriptUiCommand, ScriptWorld,
    TeamPrototypeInfo,
};
use super::scripts::{Condition, Parameter, ScriptAction, ScriptList};
use crate::ai::ai_player::{build_by_supplies, AiPlayer, SupplyOrder};
use crate::ai::ai_player_ops::{
    build_ai_base_defense, build_ai_base_defense_structure, build_specific_building_nearest_team,
    build_upgrade, guard_supply_center, is_supply_source_attacked, is_supply_source_safe,
    repair_structure, select_skillset, supply_guard_enemy_center,
};
use crate::ai::ai_skirmish_player::{build_specific_ai_building, get_ai_enemy, EnemyView};
use crate::ai::ai_team_queue::{
    ai_pre_team_destroy, build_specific_ai_team_with_env, recruit_specific_ai_team_with_env,
};
use crate::arena::{with_module_dispatch, ModuleSlot};
use crate::modules::update::ai_update::AiUpdate;
use crate::modules::update::supply_warehouse_dock_update::SupplyWarehouseDockUpdate;
use crate::arena::ObjectSlot;
use crate::gamelogic::game_engine::StartupStores;
use crate::gamelogic::global_data::GameDifficulty;
use crate::player::science_purchase::science_added_to_world;
use crate::player::player_list::PlayerList;
use crate::player::seams::{
    AiKind, PlayerEnv, PlayerIndex, PlayerPart, PlayerWorld, SkirmishScripts,
};
use crate::player::team::{
    flush_pending_team_member_moves, remove_team_from_sim_world, sync_team_to_sim_world,
    TeamFactory,
};
use crate::rules::kind_of::KindOfType;
use crate::rules::science::{ScienceType, SCIENCE_INVALID};
use crate::rules::upgrade::UpgradeMaskType;
use crate::sim::{ObjectRecord, SimApi, SimContext, SimView, SimWorld};
use crate::spatial::sides_list::SidesList;

/// The script engine of a game and what it keeps outside `ScriptEngine`.
#[derive(Default)]
pub struct GameScripts {
    pub engine: ScriptEngine,
    /// Calls the world could not answer or carry out, by name.
    pub unported: BTreeMap<&'static str, u64>,
    /// Client calls of the scripts, in order.
    pub ui: Vec<ScriptUiCommand>,
    pub exit_requested: bool,
    /// `(id, team)` of every object at the last update, to tell the engine of
    /// objects made, destroyed or moved to another team since.
    seen: Vec<(ObjectId, TeamId)>,
    /// `Object::getNextObjectID` at the last update.
    seen_next_id: Int,
    seen_any: bool,
}

/// The parts of a game outside the sim that the scripts reach.
pub struct ScriptGame<'a> {
    pub teams: &'a mut TeamFactory,
    pub stores: &'a StartupStores,
    /// `(name, location)`, newest first.
    pub waypoints: &'a [(String, Coord3D)],
    /// Objects with a script name.
    pub named_objects: &'a mut Vec<(String, ObjectId)>,
}

impl GameScripts {
    /// `ScriptEngine::newMap` with the script lists of the sides.
    pub fn new_map(
        &mut self,
        sim: &mut SimContext,
        game: ScriptGame<'_>,
        sides: Vec<Option<ScriptList>>,
    ) {
        let Self {
            engine,
            unported,
            ui,
            exit_requested,
            ..
        } = self;
        let mut world = LiveWorld::new(sim, game, unported, ui, exit_requested);
        engine.new_map(&mut world, sides);
    }

    /// `TheScriptEngine->UPDATE()`.
    pub fn update(&mut self, sim: &mut SimContext, game: ScriptGame<'_>) {
        flush_pending_team_member_moves(sim, game.teams);
        self.notify_object_changes(sim, game.named_objects);
        let Self {
            engine,
            unported,
            ui,
            exit_requested,
            ..
        } = self;
        let mut world = LiveWorld::new(sim, game, unported, ui, exit_requested);
        engine.update(&mut world);
        let destroyed = std::mem::take(&mut world.destroyed_teams);
        drop(world);
        for team in destroyed {
            engine.notify_of_team_destruction(team.0 as ScriptTeamId);
        }
    }

    /// Hands `f` the world the engine sees, as `update` does.
    pub fn with_world<R>(
        &mut self,
        sim: &mut SimContext,
        game: ScriptGame<'_>,
        f: impl FnOnce(&mut ScriptEngine, &mut dyn ScriptWorld) -> R,
    ) -> R {
        let Self {
            engine,
            unported,
            ui,
            exit_requested,
            ..
        } = self;
        let mut world = LiveWorld::new(sim, game, unported, ui, exit_requested);
        f(engine, &mut world)
    }

    /// `TheGameEngine->isTimeFrozen()` as far as scripts decide it.
    pub fn is_time_frozen(&self) -> bool {
        self.engine.is_time_frozen()
    }

    /// `notifyOfObjectCreationOrDestruction` and `notifyOfObjectDestruction`
    /// for what happened since the last update, which was in the frame before.
    fn notify_object_changes(&mut self, sim: &SimContext, named: &mut [(String, ObjectId)]) {
        let now: Vec<(ObjectId, TeamId)> = sim
            .world
            .objects
            .world_iter()
            .filter_map(|slot| sim.world.objects.get(slot).map(|o| (o.id, o.team_id)))
            .collect();
        let changed = now != self.seen || sim.world.next_object_id != self.seen_next_id;
        if self.seen_any && changed {
            self.engine
                .notify_of_object_creation_or_destruction(sim.world.frame.wrapping_sub(1));
            for (name, id) in named.iter() {
                if sim.world.objects.slot_by_id(*id).is_none() {
                    self.engine.notify_of_object_destruction(name, *id);
                }
            }
        }
        self.seen = now;
        self.seen_next_id = sim.world.next_object_id;
        self.seen_any = true;
    }
}

/// [`ScriptWorld`] for the length of one engine call.
struct LiveWorld<'a> {
    sim: &'a mut SimContext,
    game: ScriptGame<'a>,
    /// Queries count here too, and they take `&self`.
    unported: RefCell<&'a mut BTreeMap<&'static str, u64>>, // determinism-ok: single-threaded adapter, counters only
    ui: &'a mut Vec<ScriptUiCommand>,
    exit_requested: &'a mut bool,
    destroyed_teams: Vec<TeamId>,
    /// The world's computer players, put back when the adapter is dropped:
    /// asking one for its enemy may make it choose one.
    ai_players: RefCell<BTreeMap<Int, AiPlayer>>, // determinism-ok: single-threaded take-and-restore for a &self query
}

impl Drop for LiveWorld<'_> {
    fn drop(&mut self) {
        self.sim.world.ai_players = std::mem::take(self.ai_players.get_mut());
    }
}

fn team_id(team: ScriptTeamId) -> TeamId {
    TeamId(team as Int)
}

fn difficulty(value: Int) -> GameDifficulty {
    match value {
        0 => GameDifficulty::Easy,
        2 => GameDifficulty::Hard,
        _ => GameDifficulty::Normal,
    }
}

impl<'a> LiveWorld<'a> {
    fn new(
        sim: &'a mut SimContext,
        game: ScriptGame<'a>,
        unported: &'a mut BTreeMap<&'static str, u64>,
        ui: &'a mut Vec<ScriptUiCommand>,
        exit_requested: &'a mut bool,
    ) -> Self {
        let ai_players = RefCell::new(std::mem::take(&mut sim.world.ai_players));
        Self {
            ai_players,
            sim,
            game,
            unported: RefCell::new(unported),
            ui,
            exit_requested,
            destroyed_teams: Vec::new(),
        }
    }

    fn missing(&self, call: &'static str) {
        *self.unported.borrow_mut().entry(call).or_insert(0) += 1;
    }

    /// Stores the build list a computer player's order changed on `copy`.
    fn keep_build_list(&mut self, player: ScriptPlayer, copy: crate::player::player::Player) {
        if let Some(p) = self.sim.world.players.player_mut(player) {
            p.build_list = copy.build_list;
        }
    }

    fn player(&self, player: ScriptPlayer) -> Option<&crate::player::player::Player> {
        self.sim.world.players.player(player)
    }

    fn default_team(&self, player: ScriptPlayer) -> Option<TeamId> {
        self.player(player)?.default_team
    }

    fn slot(&self, id: ObjectId) -> Option<ObjectSlot> {
        self.sim.world.objects.slot_by_id(id)
    }

    fn object(&self, id: ObjectId) -> Option<&ObjectRecord> {
        self.sim.world.objects.get(self.slot(id)?)
    }

    fn members(&self, team: TeamId) -> &[ObjectSlot] {
        self.sim
            .world
            .team_members
            .get(&team)
            .map_or(&[], Vec::as_slice)
    }

    /// The team instances of the player in `Player::iterateObjects` order.
    fn player_teams(&self, player: ScriptPlayer) -> Vec<TeamId> {
        self.player(player).map_or_else(Vec::new, |p| {
            p.team_prototypes
                .iter()
                .flat_map(|&proto| self.game.teams.instance_ids(proto))
                .collect()
        })
    }

    fn player_objects<'s>(
        &'s self,
        player: ScriptPlayer,
    ) -> impl Iterator<Item = &'s ObjectRecord> + use<'s, 'a> {
        self.player_teams(player)
            .into_iter()
            .flat_map(move |team| self.members(team).iter())
            .filter_map(move |&slot| self.sim.world.objects.get(slot))
    }

    fn template_of<'s>(
        &'s self,
        object: &ObjectRecord,
    ) -> Option<&'s crate::object::thing_template::ThingTemplate> {
        object
            .thing
            .template_override()
            .get_final(&self.sim.rules.thing_factory)
    }

    fn is_kind(&self, object: &ObjectRecord, kind: KindOfType) -> bool {
        self.template_of(object)
            .is_some_and(|t| t.is_kind_of(kind) != FALSE)
    }

    /// `Team::hasAnyObjects` and `Team::hasAnyUnits`.
    fn counts_as_present(&self, object: &ObjectRecord, units_only: bool) -> bool {
        if object.is_effectively_dead() != FALSE || object.is_destroyed() {
            return false;
        }
        if self.is_kind(object, KindOfType::Projectile) || self.is_kind(object, KindOfType::Mine) {
            return false;
        }
        let left_out = if units_only {
            KindOfType::Structure
        } else {
            KindOfType::Inert
        };
        !self.is_kind(object, left_out)
    }

    fn team_has(&self, team: ScriptTeamId, units_only: bool) -> bool {
        self.members(team_id(team))
            .iter()
            .filter_map(|&slot| self.sim.world.objects.get(slot))
            .any(|o| self.counts_as_present(o, units_only))
    }

    /// Runs `f` with the player environment, over a [`FramePlayers`].
    fn with_players(
        &mut self,
        f: impl FnOnce(&mut PlayerList, &mut TeamFactory, &mut PlayerEnv<'_>),
    ) {
        let SimContext { rules, world, .. } = &mut *self.sim;
        // `f` gets the list alongside a host over the rest of the world.
        let mut players = std::mem::take(&mut world.players);
        let ai_players = self.ai_players.get_mut();
        let mut host = FramePlayers {
            world,
            ai_players,
            waypoints: self.game.waypoints,
            unported: &mut **self.unported.get_mut(),
            destroyed_teams: &mut self.destroyed_teams,
        };
        let mut env = PlayerEnv {
            templates: &self.game.stores.player_templates,
            ranks: &self.game.stores.ranks,
            sciences: &rules.science_store,
            name_keys: &mut rules.name_keys,
            world: &mut host,
        };
        f(&mut players, self.game.teams, &mut env);
        self.sim.world.players = players;
    }
}

impl ScriptWorld for LiveWorld<'_> {
    fn frame(&self) -> UnsignedInt {
        self.sim.world.frame
    }

    fn logic_random(&mut self, lo: Int, hi: Int) -> Int {
        self.sim.world.rng.get_game_logic_random_value(lo, hi)
    }

    fn player_count(&self) -> Int {
        self.sim.world.players.player_count()
    }

    fn find_player(&self, name: &str) -> Option<ScriptPlayer> {
        // `findPlayerWithNameKey`: a renamed skirmish player keeps its key.
        let key = self.sim.rules.name_keys.lookup_key(name)?;
        self.sim.world.players.find_player_with_name_key(key)
    }

    fn local_player(&self) -> ScriptPlayer {
        self.sim.world.players.local_player()
    }

    fn is_challenge_campaign(&self) -> bool {
        // Campaigns are not ported; the games here are skirmish and multiplayer.
        false
    }

    fn player_default_team(&self, player: ScriptPlayer) -> Option<ScriptTeamId> {
        self.default_team(player).map(|t| t.0 as ScriptTeamId)
    }

    fn player_name(&self, player: ScriptPlayer) -> String {
        self.player(player).map_or_else(String::new, |p| p.name.clone())
    }

    fn player_is_human(&self, player: ScriptPlayer) -> bool {
        self.player(player)
            .is_some_and(|p| p.player_type == crate::gamelogic::global_data::PlayerType::Human)
    }

    fn player_is_skirmish_ai(&self, player: ScriptPlayer) -> bool {
        self.player(player)
            .is_some_and(|p| p.ai == Some(AiKind::Skirmish))
    }

    fn player_is_active(&self, player: ScriptPlayer) -> bool {
        self.player(player)
            .is_some_and(|p| !p.observer && !p.is_dead)
    }

    fn player_difficulty(&self, player: ScriptPlayer) -> Option<GameDifficulty> {
        self.ai_players
            .borrow()
            .get(&player)
            .map(|ai| difficulty(ai.difficulty))
    }

    fn player_current_enemy(&self, player: ScriptPlayer) -> Option<ScriptPlayer> {
        let frame = self.sim.world.frame;
        get_ai_enemy(&mut self.ai_players.borrow_mut(), self, frame, player)
    }

    fn player_start_index(&self, player: ScriptPlayer) -> Int {
        self.player(player).map_or(0, |p| p.mp_start_index)
    }

    fn player_side(&self, player: ScriptPlayer) -> String {
        self.player(player).map_or_else(String::new, |p| p.side.clone())
    }

    fn player_money(&self, player: ScriptPlayer) -> UnsignedInt {
        self.sim.world.player_money.get(&player).copied().unwrap_or(0)
    }

    fn player_power(&self, player: ScriptPlayer) -> PowerReading {
        let frame = self.sim.world.frame;
        let energy = self
            .sim
            .world
            .player_energy
            .get(&player)
            .cloned()
            .unwrap_or_default();
        PowerReading {
            production: energy.get_production(frame),
            consumption: energy.get_consumption(),
            sufficient: energy.has_sufficient_power(frame) != FALSE,
            supply_ratio: energy.get_energy_supply_ratio(frame),
        }
    }

    fn player_has_any_objects(&self, player: ScriptPlayer) -> bool {
        self.player_teams(player)
            .into_iter()
            .any(|t| self.team_has(t.0 as ScriptTeamId, false))
    }

    fn player_has_any_build_facility(&self, player: ScriptPlayer) -> bool {
        self.player_objects(player).any(|o| {
            self.template_of(o)
                .is_some_and(|t| t.is_build_facility() != FALSE)
        })
    }

    fn player_count_buildings(&self, player: ScriptPlayer) -> Int {
        self.player_objects(player)
            .filter(|o| self.is_kind(o, KindOfType::Structure))
            .count() as Int
    }

    fn player_count_faction_buildings(&self, player: ScriptPlayer) -> Int {
        self.player_objects(player)
            .filter(|o| {
                self.is_kind(o, KindOfType::Structure)
                    && self.is_kind(o, KindOfType::MpCountForVictory)
            })
            .count() as Int
    }

    fn player_count_objects_by_template(
        &self,
        player: ScriptPlayer,
        templates: &[String],
        ignore_dead: bool,
    ) -> Vec<Int> {
        let factory = &self.sim.rules.thing_factory;
        let wanted: Vec<_> = templates
            .iter()
            .map(|n| factory.find_template_final(n))
            .collect();
        let mut counts = vec![0; templates.len()];
        for object in self.player_objects(player) {
            let Some(own) = self.template_of(object) else {
                continue;
            };
            for (i, template) in wanted.iter().enumerate() {
                let Some(template) = template else { continue };
                if own.is_equivalent_to(template) == FALSE {
                    continue;
                }
                if ignore_dead && object.is_effectively_dead() != FALSE {
                    continue;
                }
                counts[i] += 1;
                break;
            }
        }
        counts
    }

    fn player_attacked_by(&self, player: ScriptPlayer, attacker: ScriptPlayer) -> bool {
        self.player(player).is_some_and(|p| {
            usize::try_from(attacker)
                .ok()
                .and_then(|a| p.attacked_by.get(a))
                .copied()
                .unwrap_or(false)
        })
    }

    fn money_withdraw(&mut self, player: ScriptPlayer, amount: UnsignedInt) {
        let Some(team) = self.default_team(player) else {
            return;
        };
        self.sim.withdraw_player_money(team, amount);
    }

    fn money_deposit(
        &mut self,
        player: ScriptPlayer,
        amount: UnsignedInt,
        _play_sound: bool,
        count_in_score: bool,
    ) {
        if count_in_score {
            if let Some(team) = self.default_team(player) {
                self.sim.deposit_player_money(team, amount);
            }
        } else if amount != 0 {
            let money = self.sim.world.player_money.entry(player).or_insert(0);
            *money = money.wrapping_add(amount);
        }
    }

    fn set_can_build_units(&mut self, player: ScriptPlayer, can: bool) {
        if let Some(p) = self.sim.world.players.player_mut(player) {
            p.can_build_units = can;
        }
    }

    fn set_can_build_base(&mut self, player: ScriptPlayer, can: bool) {
        if let Some(p) = self.sim.world.players.player_mut(player) {
            p.can_build_base = can;
        }
    }

    fn set_player_relationship(&mut self, from: ScriptPlayer, to: ScriptPlayer, relationship: Int) {
        let relationship = match relationship {
            0 => Relationship::Enemies,
            2 => Relationship::Allies,
            _ => Relationship::Neutral,
        };
        if let Some(p) = self.sim.world.players.player_mut(from) {
            p.player_relations.set(to as u32, relationship);
            self.sim
                .world
                .player_relationships
                .insert((from, to), relationship);
        }
    }

    fn grant_science(&mut self, player: ScriptPlayer, science: ScienceType) -> bool {
        if !self.sim.rules.science_store.is_science_grantable(science) {
            return false;
        }
        let mut added = false;
        self.with_players(|players, _, env| {
            if let Some(p) = players.player_mut(player) {
                added = p.add_science(science, env.world);
            }
        });
        if added {
            science_added_to_world(self.sim, player, science);
        }
        added
    }

    fn kill_player(&mut self, _player: ScriptPlayer) {
        self.missing("Player::killPlayer");
    }

    fn team_prototype(&self, name: &str) -> Option<TeamPrototypeInfo> {
        let teams = &*self.game.teams;
        let proto = teams
            .listed_prototypes()
            .filter_map(|id| teams.prototype(id))
            .find(|p| p.name == name)?;
        Some(TeamPrototypeInfo {
            singleton: proto.singleton,
            max_instances: proto.template.max_instances,
            instances: proto
                .instances
                .iter()
                .map(|t| t.id.0 as ScriptTeamId)
                .collect(),
        })
    }

    fn team_name(&self, team: ScriptTeamId) -> String {
        self.game
            .teams
            .prototype_of(team_id(team))
            .map_or_else(String::new, |p| p.name.clone())
    }

    fn team_exists(&self, team: ScriptTeamId) -> bool {
        self.game.teams.team(team_id(team)).is_some()
    }

    fn team_is_active(&self, team: ScriptTeamId) -> bool {
        self.game.teams.team(team_id(team)).is_some_and(|t| t.active)
    }

    fn team_is_created(&self, team: ScriptTeamId) -> bool {
        self.game.teams.team(team_id(team)).is_some_and(|t| t.created)
    }

    fn team_state(&self, team: ScriptTeamId) -> String {
        self.game
            .teams
            .team(team_id(team))
            .map_or_else(String::new, |t| t.state.clone())
    }

    fn team_controlling_player(&self, team: ScriptTeamId) -> Option<ScriptPlayer> {
        self.game.teams.controlling_player(team_id(team))
    }

    fn team_has_any_objects(&self, team: ScriptTeamId) -> bool {
        self.team_has(team, false)
    }

    fn team_has_any_units(&self, team: ScriptTeamId) -> bool {
        self.team_has(team, true)
    }

    fn team_members(&self, team: ScriptTeamId) -> Vec<ObjectId> {
        self.members(team_id(team))
            .iter()
            .filter_map(|&slot| self.sim.world.objects.id_of(slot))
            .collect()
    }

    fn update_team_states(&mut self) {
        // The frames keep the member lists in the world.
        for index in 0..self.sim.world.players.player_count() {
            for team in self.player_teams(index) {
                let members: Vec<ObjectId> = self.team_members(team.0 as ScriptTeamId);
                if let Some(t) = self.game.teams.team_mut(team) {
                    t.members = members;
                }
            }
        }
        self.with_players(|players, teams, env| players.update_team_states(teams, env));
        for &team in &self.destroyed_teams {
            remove_team_from_sim_world(&mut self.sim.world, team);
        }
    }

    fn team_group_is_idle(&mut self, _team: ScriptTeamId) -> bool {
        self.missing("AIGroup::isIdle");
        false
    }

    fn team_group_is_ai_dead(&mut self, _team: ScriptTeamId) -> bool {
        self.missing("AIGroup::isGroupAiDead");
        false
    }

    fn team_group_idle(&mut self, _team: ScriptTeamId) {
        self.missing("AIGroup::groupIdle");
    }

    fn set_team_state(&mut self, team: ScriptTeamId, state: &str) {
        if let Some(t) = self.game.teams.team_mut(team_id(team)) {
            t.state = state.to_owned();
        }
    }

    fn delete_team(&mut self, _team: ScriptTeamId, _ignore_dead: bool) {
        self.missing("Team::deleteTeam");
    }

    fn kill_team(&mut self, _team: ScriptTeamId) {
        self.missing("Team::killTeam");
    }

    fn named_objects(&self) -> Vec<(String, ObjectId)> {
        self.sim
            .world
            .objects
            .world_iter()
            .filter_map(|slot| self.sim.world.objects.id_of(slot))
            .filter_map(|id| {
                let name = self.game.named_objects.iter().find(|(_, n)| *n == id)?;
                Some((name.0.clone(), id))
            })
            .collect()
    }

    fn object_exists(&self, id: ObjectId) -> bool {
        self.object(id).is_some()
    }

    fn object_name(&self, id: ObjectId) -> String {
        self.game
            .named_objects
            .iter()
            .find(|(_, n)| *n == id)
            .map_or_else(String::new, |(name, _)| name.clone())
    }

    fn object_is_effectively_dead(&self, id: ObjectId) -> bool {
        self.object(id)
            .is_some_and(|o| o.is_effectively_dead() != FALSE)
    }

    fn object_controlling_player(&self, id: ObjectId) -> Option<ScriptPlayer> {
        self.game.teams.controlling_player(self.object(id)?.team_id)
    }

    fn object_health(&self, id: ObjectId) -> Option<(Real, Real)> {
        let slot = self.slot(id)?;
        let body = self.sim.world.objects.get(slot)?.body_slot?;
        let initial = self
            .sim
            .world
            .modules
            .payload(body)
            .and_then(|p| p.as_body())
            .map(|b| b.get_initial_health());
        if initial.is_none() {
            self.missing("BodyModule::getInitialHealth");
        }
        Some((self.sim.object_health(slot), initial.unwrap_or(0.0)))
    }

    fn object_ai_is_idle(&self, id: ObjectId) -> Option<bool> {
        let slot = self.slot(id)?;
        self.sim.world.objects.get(slot)?.ai_slot?;
        Some(self.sim.ai_is_idle(slot) != FALSE)
    }

    fn object_ai_idle(&mut self, id: ObjectId) {
        if let Some(slot) = self.slot(id) {
            self.sim
                .ai_idle(slot, crate::modules::module::CommandSourceType::FromScript);
        }
    }

    fn object_ai_guard_here(&mut self, _id: ObjectId, _choose_normal_locomotor: bool) {
        self.missing("aiGuardPosition");
    }

    fn set_object_name(&mut self, id: ObjectId, name: &str) {
        self.game.named_objects.retain(|(_, n)| *n != id);
        if !name.is_empty() {
            self.game.named_objects.push((name.to_owned(), id));
        }
    }

    fn copy_indicator_color(&mut self, _from: ObjectId, _to: ObjectId) {
        self.missing("Object::setCustomIndicatorColor");
    }

    fn destroy_object(&mut self, id: ObjectId) {
        if let Some(slot) = self.slot(id) {
            self.sim.destroy_object(slot);
        }
    }

    fn kill_object(&mut self, _id: ObjectId) {
        self.missing("Object::kill");
    }

    fn set_all_difficulty_bonuses(&mut self, _enable: bool) {
        self.missing("Object::setReceivingDifficultyBonus");
    }

    fn has_thing_template(&self, name: &str) -> bool {
        self.sim.rules.thing_factory.find_template(name).is_some()
    }

    fn thing_template_index(&self, name: &str) -> Option<u32> {
        self.sim
            .rules
            .thing_factory
            .find_template(name)
            .map(|t| u32::from(t.get_template_id()))
    }

    fn thing_template_name(&self, index: u32) -> String {
        self.sim
            .rules
            .thing_factory
            .find_by_template_id(index as UnsignedShort)
            .map_or_else(String::new, |t| t.get_name().to_owned())
    }

    fn thing_templates_of_kind(&self, kind: Int) -> Vec<u32> {
        let kind = KindOfType::from_i32(kind);
        if kind == KindOfType::Invalid {
            return Vec::new();
        }
        let factory = &self.sim.rules.thing_factory;
        (1..=u32::from(UnsignedShort::MAX))
            .map(|id| (id, factory.find_by_template_id(id as UnsignedShort)))
            .take_while(|(_, t)| t.is_some())
            .filter(|(_, t)| t.is_some_and(|t| t.is_kind_of(kind) != FALSE))
            .map(|(id, _)| id)
            .collect()
    }

    fn science_from_name(&self, name: &str) -> Option<ScienceType> {
        // `getScienceFromInternalName` without interning an unknown name.
        let science = self
            .sim
            .rules
            .name_keys
            .lookup_key(name)
            .map_or(SCIENCE_INVALID, |key| key.0);
        (science != SCIENCE_INVALID).then_some(science)
    }

    fn science_name(&self, science: ScienceType) -> String {
        self.sim
            .rules
            .science_store
            .get_internal_name_for_science(science, &self.sim.rules.name_keys)
            .to_owned()
    }

    fn waypoint_location(&self, name: &str) -> Option<Coord3D> {
        self.game
            .waypoints
            .iter()
            .find(|(n, _)| n == name)
            .map(|&(_, p)| p)
    }

    fn do_shroud_reveal(&mut self, _at: Coord3D, _radius: Real, _player: ScriptPlayer) {
        self.missing("PartitionManager::doShroudReveal");
    }

    fn undo_shroud_reveal(&mut self, _at: Coord3D, _radius: Real, _player: ScriptPlayer) {
        self.missing("PartitionManager::undoShroudReveal");
    }

    fn music_track_completed(&self, _track: &str, _times: Int) -> bool {
        // No music plays without a client.
        self.missing("AudioManager::hasMusicTrackCompleted");
        false
    }

    fn set_team_attack_priority(&mut self, team: ScriptTeamId, set: Option<&str>, members: &str) {
        let team = team_id(team);
        if let (Some(name), Some(prototype)) =
            (set, self.game.teams.team(team).map(|t| t.prototype))
        {
            if let Some(p) = self.game.teams.prototype_mut(prototype) {
                p.attack_priority_name = name.to_owned();
            }
        }
        for slot in self.members(team).to_vec() {
            let Some(ai) = self.sim.world.objects.get(slot).and_then(|o| o.ai_slot) else {
                continue;
            };
            let given = self
                .sim
                .world
                .modules
                .payload_mut(ai)
                .and_then(|m| m.as_any_mut())
                .and_then(|a| a.downcast_mut::<AiUpdate>())
                .map(|ai| ai.attack_info = members.to_owned());
            if given.is_none() {
                self.missing("AIUpdateInterface::setAttackInfo");
            }
        }
    }

    fn set_warehouse_cash(&mut self, id: ObjectId, cash: Int) {
        let Some(slot) = self.slot(id) else {
            return;
        };
        let modules: Vec<ModuleSlot> = self.sim.world.objects.get(slot).map_or_else(Vec::new, |o| {
            o.behaviors.iter().map(|&(module, _)| module).collect()
        });
        for module in modules {
            // `findUpdateModule` takes the first.
            let found = with_module_dispatch(
                &mut *self.sim,
                module,
                |sim| &mut sim.world.modules,
                |payload, sim, _| {
                    let dock = payload
                        .as_any_mut()
                        .and_then(|a| a.downcast_mut::<SupplyWarehouseDockUpdate>());
                    match dock {
                        Some(dock) => {
                            dock.set_cash_value(sim, cash);
                            true
                        }
                        None => false,
                    }
                },
            );
            if found == Ok(true) {
                return;
            }
        }
    }

    fn audio_length_ms(&self, _event: &str) -> Real {
        self.missing("AudioManager::getAudioLengthMS");
        0.0
    }

    fn ui(&mut self, command: ScriptUiCommand) {
        self.ui.push(command);
    }

    fn exit_game(&mut self) {
        *self.exit_requested = true;
    }

    fn command_button_ready(&mut self, _team: &Parameter, _button: &Parameter, _all: bool) -> bool {
        self.missing("evaluateSkirmishCommandButtonIsReady");
        false
    }

    fn team_is_contained(&mut self, _team: &Parameter, _all: bool) -> bool {
        self.missing("evaluateTeamIsContained");
        false
    }

    fn unported_condition(&mut self, _condition: &Condition) -> bool {
        // The engine counts them by type.
        false
    }

    fn unported_action(&mut self, _action: &ScriptAction) {}

    fn ai_build_building(&mut self, player: ScriptPlayer, template: &str) {
        let ai_players = self.ai_players.get_mut();
        let (Some(ai), Some(p)) = (
            ai_players.get_mut(&player),
            self.sim.world.players.player_mut(player),
        ) else {
            return;
        };
        if ai.skirmish.is_none() {
            self.missing("AIPlayer::buildSpecificAIBuilding of a non-skirmish player");
            return;
        }
        let mut list = std::mem::take(&mut p.build_list);
        build_specific_ai_building(self.sim, ai, &mut list, template);
        if let Some(p) = self.sim.world.players.player_mut(player) {
            p.build_list = list;
        }
    }

    fn ai_build_by_supplies(&mut self, player: ScriptPlayer, minimum_cash: Int, template: &str) {
        if !self.ai_players.get_mut().contains_key(&player) {
            return;
        }
        // `getAiEnemy`, which may make the AI choose one; only a skirmish AI does.
        let frame = self.sim.world.frame;
        let enemy = get_ai_enemy(&mut self.ai_players.borrow_mut(), self, frame, player);
        let enemy_center = enemy.map(|enemy| middle(&self.structure_positions(enemy)));
        let pathfind_extent = self
            .sim
            .terrain
            .terrain_logic()
            .map_or_else(Default::default, |t| t.maximum_pathfind_extent());
        let ai_players = self.ai_players.get_mut();
        let (Some(ai), Some(p)) = (
            ai_players.get_mut(&player),
            self.sim.world.players.player(player),
        ) else {
            return;
        };
        let mut copy = p.clone();
        let mut order = SupplyOrder {
            sim: self.sim,
            ai,
            player: &mut copy,
            enemy_center,
            pathfind_extent,
        };
        build_by_supplies(&mut order, minimum_cash, template);
        self.keep_build_list(player, copy);
    }

    fn ai_build_building_nearest_team(
        &mut self,
        player: ScriptPlayer,
        template: &str,
        team: ScriptTeamId,
    ) {
        let pathfind_extent = self
            .sim
            .terrain
            .terrain_logic()
            .map_or_else(Default::default, |t| t.maximum_pathfind_extent());
        let ai_players = self.ai_players.get_mut();
        let (Some(ai), Some(p)) = (
            ai_players.get_mut(&player),
            self.sim.world.players.player(player),
        ) else {
            return;
        };
        let mut copy = p.clone();
        build_specific_building_nearest_team(
            self.sim,
            ai,
            &mut copy,
            template,
            team_id(team),
            &pathfind_extent,
        );
        self.keep_build_list(player, copy);
    }

    fn ai_build_upgrade(&mut self, player: ScriptPlayer, upgrade: &str) {
        let Some(p) = self.sim.world.players.player(player) else {
            return;
        };
        if !self.ai_players.get_mut().contains_key(&player) {
            return;
        }
        let list = p.build_list.clone();
        build_upgrade(self.sim, player, &list, upgrade);
    }

    fn ai_repair_structure(&mut self, player: ScriptPlayer, structure: ObjectId) {
        if let Some(ai) = self.ai_players.get_mut().get_mut(&player) {
            repair_structure(self.sim, ai, structure);
        }
    }

    fn ai_select_skillset(&mut self, player: ScriptPlayer, skillset: Int) {
        if let Some(ai) = self.ai_players.get_mut().get_mut(&player) {
            select_skillset(ai, skillset);
        }
    }

    fn ai_set_team_delay_seconds(&mut self, player: ScriptPlayer, delay: Int) {
        if let Some(ai) = self.ai_players.get_mut().get_mut(&player) {
            ai.team_seconds = delay;
        }
    }

    fn ai_guard_supply_center(
        &mut self,
        player: ScriptPlayer,
        team: ScriptTeamId,
        min_supplies: Int,
    ) {
        if !self.ai_players.get_mut().contains_key(&player) {
            return;
        }
        let enemy_center =
            supply_guard_enemy_center(self.sim, &mut self.ai_players.borrow_mut(), self, player);
        let pathfind_extent = self
            .sim
            .terrain
            .terrain_logic()
            .map_or_else(Default::default, |t| t.maximum_pathfind_extent());
        let ai_players = self.ai_players.get_mut();
        let (Some(ai), Some(p)) = (
            ai_players.get_mut(&player),
            self.sim.world.players.player(player),
        ) else {
            return;
        };
        let mut copy = p.clone();
        guard_supply_center(
            self.sim,
            ai,
            &mut copy,
            enemy_center,
            pathfind_extent,
            team_id(team),
            min_supplies,
        );
        self.keep_build_list(player, copy);
    }

    fn ai_build_base_defense(&mut self, player: ScriptPlayer, flank: bool) {
        let pathfind_extent = self
            .sim
            .terrain
            .terrain_logic()
            .map_or_else(Default::default, |t| t.maximum_pathfind_extent());
        if let Some(ai) = self.ai_players.get_mut().get_mut(&player) {
            build_ai_base_defense(
                self.sim,
                ai,
                player,
                flank,
                &pathfind_extent,
            );
        }
    }

    fn ai_build_base_defense_structure(
        &mut self,
        player: ScriptPlayer,
        template: &str,
        flank: bool,
    ) {
        let pathfind_extent = self
            .sim
            .terrain
            .terrain_logic()
            .map_or_else(Default::default, |t| t.maximum_pathfind_extent());
        if let Some(ai) = self.ai_players.get_mut().get_mut(&player) {
            build_ai_base_defense_structure(
                self.sim,
                ai,
                player,
                template,
                flank,
                &pathfind_extent,
            );
        }
    }

    fn ai_is_supply_source_safe(&self, player: ScriptPlayer, min_supplies: Int) -> bool {
        if !self.ai_players.borrow().contains_key(&player) {
            return true;
        }
        let frame = self.sim.world.frame;
        let enemy = get_ai_enemy(&mut self.ai_players.borrow_mut(), self, frame, player);
        let enemy_center = enemy.map(|enemy| middle(&self.structure_positions(enemy)));
        let pathfind_extent = self
            .sim
            .terrain
            .terrain_logic()
            .map_or_else(Default::default, |t| t.maximum_pathfind_extent());
        let mut ai_players = self.ai_players.borrow_mut();
        let Some(ai) = ai_players.get_mut(&player) else {
            return true;
        };
        let Some(p) = self.sim.world.players.player(player) else {
            return true;
        };
        let mut player_copy = p.clone();
        let order = SupplyOrder {
            sim: self.sim,
            ai,
            player: &mut player_copy,
            enemy_center,
            pathfind_extent,
        };
        is_supply_source_safe(&order, min_supplies)
    }

    fn ai_is_supply_source_attacked(&self, player: ScriptPlayer) -> bool {
        self.ai_players
            .borrow_mut()
            .get_mut(&player)
            .is_some_and(|ai| is_supply_source_attacked(self.sim, ai))
    }

    fn ai_build_team(&mut self, team_name: &str) {
        flush_pending_team_member_moves(self.sim, self.game.teams);
        let Some(proto_id) = self
            .game
            .teams
            .find_team_prototype(team_name, &mut self.sim.rules.name_keys)
        else {
            return;
        };
        let Some(player) = self.game.teams.prototype(proto_id).map(|p| p.owner) else {
            return;
        };
        if !self.ai_players.get_mut().contains_key(&player) {
            return;
        }
        build_specific_ai_team_with_env(
            self.sim,
            self.ai_players.get_mut(),
            self.game.teams,
            self.game.stores,
            self.game.waypoints,
            &mut self.destroyed_teams,
            player,
            proto_id,
            true,
        );
    }

    fn ai_recruit_team(&mut self, team_name: &str, recruit_radius: Real) {
        flush_pending_team_member_moves(self.sim, self.game.teams);
        let Some(proto_id) = self
            .game
            .teams
            .find_team_prototype(team_name, &mut self.sim.rules.name_keys)
        else {
            return;
        };
        let Some(player) = self.game.teams.prototype(proto_id).map(|p| p.owner) else {
            return;
        };
        if !self.ai_players.get_mut().contains_key(&player) {
            return;
        }
        recruit_specific_ai_team_with_env(
            self.sim,
            self.ai_players.get_mut(),
            self.game.teams,
            self.game.stores,
            self.game.waypoints,
            &mut self.destroyed_teams,
            player,
            proto_id,
            recruit_radius,
        );
    }

    fn take_pending_team_actions(&mut self) -> Vec<(String, Option<ScriptTeamId>)> {
        std::mem::take(&mut self.sim.world.pending_team_actions)
            .into_iter()
            .map(|(name, team)| (name, team.map(|t| t.0 as ScriptTeamId)))
            .collect()
    }
}

/// The middle of `getPlayerStructureBounds`, all zero without structures.
fn middle(positions: &[Coord3D]) -> Coord2D {
    let first = positions.first().copied().unwrap_or_default();
    let (mut lo, mut hi) = ((first.x, first.y), (first.x, first.y));
    for p in positions {
        if p.x < lo.0 {
            lo.0 = p.x;
        }
        if p.y < lo.1 {
            lo.1 = p.y;
        }
        if p.x > hi.0 {
            hi.0 = p.x;
        }
        if p.y > hi.1 {
            hi.1 = p.y;
        }
    }
    Coord2D {
        x: (lo.0 + hi.0) * 0.5,
        y: (lo.1 + hi.1) * 0.5,
    }
}

impl EnemyView for LiveWorld<'_> {
    fn player_count(&self) -> Int {
        self.sim.world.players.player_count()
    }

    fn is_enemy(&self, player: PlayerIndex, other: PlayerIndex) -> bool {
        let (Some(own), Some(team)) = (self.player(player), self.default_team(other)) else {
            return false;
        };
        own.relationship(Some(team), Some(other)) == Relationship::Enemies
    }

    fn has_any_objects(&self, player: PlayerIndex) -> bool {
        self.player_has_any_objects(player)
    }

    fn has_any_units(&self, player: PlayerIndex) -> bool {
        self.player_teams(player)
            .into_iter()
            .any(|t| self.team_has(t.0 as ScriptTeamId, true))
    }

    fn has_any_build_facility(&self, player: PlayerIndex) -> bool {
        self.player_has_any_build_facility(player)
    }

    fn structure_positions(&self, player: PlayerIndex) -> Vec<Coord3D> {
        self.player_objects(player)
            .filter(|o| self.is_kind(o, KindOfType::Structure))
            .map(|o| *o.thing.get_position())
            .collect()
    }
}

/// What `updateTeamStates` and `addScience` need during a frame. Everything of
/// the game start is not reached from them and is counted if it is.
pub(crate) struct FramePlayers<'a> {
    pub(crate) world: &'a mut SimWorld,
    pub(crate) ai_players: &'a mut BTreeMap<Int, AiPlayer>,
    pub(crate) waypoints: &'a [(String, Coord3D)],
    pub(crate) unported: &'a mut BTreeMap<&'static str, u64>,
    pub(crate) destroyed_teams: &'a mut Vec<TeamId>,
}

impl FramePlayers<'_> {
    fn missing(&mut self, call: &'static str) {
        *self.unported.entry(call).or_insert(0) += 1;
    }

    fn member(&self, object: ObjectId) -> Option<(ObjectSlot, &ObjectRecord)> {
        let slot = self.world.objects.slot_by_id(object)?;
        Some((slot, self.world.objects.get(slot)?))
    }
}

impl PlayerWorld for FramePlayers<'_> {
    fn frame(&self) -> UnsignedInt {
        self.world.frame
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
        1
    }
    fn force_skirmish_ai(&self) -> bool {
        false
    }
    fn general_name(&self) -> Vec<u16> {
        Vec::new()
    }
    fn waypoint_location(&self, name: &str) -> Option<Coord3D> {
        self.waypoints
            .iter()
            .find(|(n, _)| n == name)
            .map(|&(_, p)| p)
    }
    fn waypoint_exists(&self, _id: UnsignedInt) -> bool {
        false
    }
    fn becoming_local_player(&mut self, _player: PlayerIndex, _yes: bool) {}
    fn local_player_update(&mut self, _player: PlayerIndex) {}
    fn create_ai(&mut self, _player: PlayerIndex, _kind: AiKind) {
        self.missing("Player::createAi");
    }
    fn delete_ai(&mut self, _player: PlayerIndex) {
        self.missing("Player::deleteAi");
    }
    fn set_ai_difficulty(&mut self, _player: PlayerIndex, _difficulty: Int) {
        self.missing("AIPlayer::setAIDifficulty");
    }
    fn ai_update(&mut self, _player: PlayerIndex) {
        self.missing("AIPlayer::update");
    }
    fn ai_new_map(&mut self, _player: PlayerIndex) {
        self.missing("AIPlayer::newMap");
    }
    fn ai_pre_team_destroy(&mut self, player: PlayerIndex, team: TeamId) {
        ai_pre_team_destroy(self.ai_players, player, team);
    }
    fn qualify_skirmish_scripts(&mut self, _sides: &mut SidesList, _copy: &SkirmishScripts<'_>) {
        self.missing("Player::initFromDict");
    }
    fn create_economy_trackers(&mut self, _player: PlayerIndex) {}
    fn delete_economy_trackers(&mut self, _player: PlayerIndex) {}
    fn academy_init(&mut self, _player: PlayerIndex) {}
    fn academy_update(&mut self, _player: PlayerIndex) {}
    fn academy_record_income(&mut self, _player: PlayerIndex) {
        self.missing("AcademyStats::recordIncome");
    }
    fn power_brownout_changed(&mut self, _player: PlayerIndex, _brownout: bool) {
        self.missing("Player::onPowerBrownOutChange");
    }
    // `grant_science` brings the world up to date itself.
    fn science_added(&mut self, _player: PlayerIndex, _science: ScienceType) {}
    fn upgrade_count(&self, _player: PlayerIndex) -> UnsignedShort {
        0
    }
    fn delete_upgrades(&mut self, _player: PlayerIndex) {
        self.missing("Player::deleteUpgrades");
    }
    fn upgrade_masks(&self) -> Vec<(String, UpgradeMaskType)> {
        Vec::new()
    }
    fn notify_team_created(&mut self, players: &PlayerList, teams: &TeamFactory, team: TeamId) {
        sync_team_to_sim_world(self.world, players, teams, team);
    }
    fn notify_team_destroyed(&mut self, team: TeamId) {
        self.destroyed_teams.push(team);
    }
    fn detach_member(&mut self, _object: ObjectId) {
        self.missing("Object::setTeam");
    }
    fn member_changed_player(&mut self, _object: ObjectId, _old: PlayerIndex, _new: PlayerIndex) {
        self.missing("Object::onCapture");
    }
    fn execute_team_create_actions(
        &mut self,
        _players: &mut PlayerList,
        _teams: &mut TeamFactory,
        condition: &str,
    ) {
        if !condition.is_empty() {
            self.world
                .pending_team_actions
                .push((condition.to_owned(), None));
        }
    }
    fn duplicate_script(&mut self, _name: &str) -> Option<UnsignedInt> {
        self.missing("team generic script");
        None
    }
    fn evaluate_script(
        &mut self,
        _players: &mut PlayerList,
        _teams: &mut TeamFactory,
        _script: UnsignedInt,
        _team: TeamId,
    ) -> bool {
        self.missing("team generic script");
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
        self.missing("team generic script");
    }
    fn run_script(
        &mut self,
        _players: &mut PlayerList,
        _teams: &mut TeamFactory,
        name: &str,
        _team: TeamId,
    ) {
        // The reference asks for every unset script name too.
        if !name.is_empty() {
            self.missing("ScriptEngine::runScript");
        }
    }
    fn member_is_effectively_dead(&self, object: ObjectId) -> bool {
        self.member(object)
            .is_some_and(|(_, o)| o.is_effectively_dead() != FALSE)
    }
    fn member_ai_idle(&self, object: ObjectId) -> Option<bool> {
        let (_, record) = self.member(object)?;
        let ai = record.ai_slot?;
        self.world
            .modules
            .payload(ai)
            .and_then(|m| m.as_ai())
            .map(|ai| ai.is_idle() != FALSE)
    }
    fn member_sees_enemy(&mut self, _object: ObjectId) -> bool {
        self.missing("Team::updateState enemy sighting");
        false
    }
    fn xfer_player_part(
        &mut self,
        _player: PlayerIndex,
        _part: PlayerPart,
        _xfer: &mut dyn Xfer,
    ) -> Result<(), XferError> {
        Err(XferError::InvalidData)
    }
}
