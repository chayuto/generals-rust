//! `PKT-426` / `PKT-427`: `WorkOrder`, `TeamInQueue`, team building, recruiting,
//! production hooks, and `queueDozer` / `queueSupplyTruck` for `AIPlayer` and
//! `AISkirmishPlayer`.
//!
//! Reference: `AIPlayer.h` (L45-141), `AIPlayer.cpp`, and `AISkirmishPlayer.cpp`.

use std::collections::BTreeMap;

use zh_common::math::Coord3D;
use zh_common::types::{
    Bool, Int, ObjectId, Real, Relationship, TeamId, UnsignedInt, UnsignedShort, XferVersion,
    FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError, XferMode};

use crate::ai::ai_player::{
    ai_mut, owner_of, xfer_flag, AiPlayer, DIFFICULTY_EASY, DIFFICULTY_HARD,
};
use crate::ai::ai_state_machine::state_id;
use crate::arena::{ObjectSlot, ThingTemplateId};
use crate::gamelogic::game_engine::StartupStores;
use crate::gamelogic::global_data::GameDifficulty;
use crate::modules::module::CommandSourceType;
use crate::modules::update::ai_update::{AiUpdate, DerivedAi};
use crate::modules::update::supply_warehouse_dock_update::SupplyWarehouseDockUpdate;
use crate::modules::update::worker_ai_update::AS_SUPPLY_TRUCK;
use crate::object::thing_factory::ThingFactory;
use crate::player::build_assistant::BuildContext;
use crate::player::seams::{PlayerEnv, PlayerIndex};
use crate::player::team::{
    flag as xfer_bool, flush_pending_team_member_moves, remove_team_from_sim_world, team_is_idle,
    transfer_units_to, try_to_recruit, xfer_team_id, TeamFactory, TeamPrototypeId,
};
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::scripts::script_engine::ScriptTeamId;
use crate::scripts::script_world::FramePlayers;
use crate::scripts::{script_template_registry, ScriptGame};
use crate::sim::{SimApi, SimContext, SimView, PRIVATE_STATUS_OFF_MAP};
use crate::spatial::partition_query::DistanceCalculationType;

const FRAMES_PER_SECOND: Int = 30;
const SUPPLY_CENTER_CLOSE_DIST: Real = 20.0 * 10.0;

/// `WorkOrder` (`AIPlayer.h:45-84`).
#[derive(Debug, Clone, PartialEq)]
pub struct WorkOrder {
    pub thing_name: String,
    pub thing: Option<ThingTemplateId>,
    pub factory_id: ObjectId,
    pub num_completed: Int,
    pub num_required: Int,
    pub required: bool,
    pub is_resource_gatherer: bool,
}

impl Default for WorkOrder {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkOrder {
    pub fn new() -> Self {
        Self {
            thing_name: String::new(),
            thing: None,
            factory_id: ObjectId::INVALID,
            num_completed: 0,
            num_required: 1,
            required: false,
            is_resource_gatherer: false,
        }
    }

    /// `WorkOrder::isWaitingToBuild` (`AIPlayer.h:77-84`).
    pub fn is_waiting_to_build(&self) -> bool {
        if self.factory_id != ObjectId::INVALID {
            return false;
        }
        if self.num_completed >= self.num_required {
            return false;
        }
        true
    }

    /// `WorkOrder::validateFactory` (`AIPlayer.cpp:3757-3771`).
    pub fn validate_factory(&mut self, sim: &SimContext, owner_player: PlayerIndex) {
        if self.factory_id == ObjectId::INVALID {
            return;
        }
        let Some(factory) = sim.world.objects.slot_by_id(self.factory_id) else {
            self.factory_id = ObjectId::INVALID;
            return;
        };
        let owner = sim
            .world
            .objects
            .get(factory)
            .and_then(|o| sim.controlling_player(o.team_id));
        if owner != Some(owner_player) {
            self.factory_id = ObjectId::INVALID;
        }
    }

    /// `WorkOrder::crc` (`AIPlayer.cpp:3776-3779`).
    pub fn crc(&self, _xfer: &mut dyn Xfer) -> Result<(), XferError> {
        Ok(())
    }

    /// `WorkOrder::xfer` (`AIPlayer.cpp:3786-3815`).
    pub fn xfer(
        &mut self,
        xfer: &mut dyn Xfer,
        thing_factory: Option<&ThingFactory>,
    ) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        if xfer.mode() == XferMode::Save {
            let mut name = match (thing_factory, self.thing) {
                (Some(tf), Some(id)) => tf
                    .find_by_template_id(id.0 as u16)
                    .map_or_else(|| self.thing_name.clone(), |t| t.get_name().to_owned()),
                _ => self.thing_name.clone(),
            };
            xfer.xfer_ascii_string(&mut name)?;
        } else {
            xfer.xfer_ascii_string(&mut self.thing_name)?;
            self.thing = if self.thing_name.is_empty() {
                None
            } else {
                thing_factory.and_then(|tf| tf.find_template_id(&self.thing_name))
            };
        }
        xfer.xfer_object_id(&mut self.factory_id)?;
        xfer.xfer_int(&mut self.num_completed)?;
        xfer.xfer_int(&mut self.num_required)?;
        xfer_bool(xfer, &mut self.required)?;
        xfer_bool(xfer, &mut self.is_resource_gatherer)
    }
}

/// `TeamInQueue` (`AIPlayer.h:88-141`).
#[derive(Debug, Clone, PartialEq)]
pub struct TeamInQueue {
    pub work_orders: Vec<WorkOrder>,
    pub priority_build: bool,
    pub team_id: TeamId,
    pub prototype_id: Option<TeamPrototypeId>,
    pub has_home_location: bool,
    pub home_location: Coord3D,
    pub frame_started: Int,
    pub sent_to_start_location: bool,
    pub stop_queueing: bool,
    pub reinforcement: bool,
    pub reinforcement_id: ObjectId,
}

impl Default for TeamInQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl TeamInQueue {
    pub fn new() -> Self {
        Self {
            work_orders: Vec::new(),
            priority_build: false,
            team_id: TeamId::INVALID,
            prototype_id: None,
            has_home_location: false,
            home_location: Coord3D::default(),
            frame_started: 0,
            sent_to_start_location: false,
            stop_queueing: false,
            reinforcement: false,
            reinforcement_id: ObjectId::INVALID,
        }
    }

    fn from_team(teams: &TeamFactory, team_id: TeamId) -> Self {
        let mut q = Self::new();
        q.team_id = team_id;
        q.sync_prototype_cache(teams);
        q
    }

    pub fn sync_prototype_cache(&mut self, teams: &TeamFactory) {
        if let Some(proto) = teams.prototype_of(self.team_id) {
            self.prototype_id = Some(proto.id);
            self.has_home_location = proto.template.has_home_location;
            self.home_location = proto.template.home_location;
        }
    }

    /// `TeamInQueue::isAllBuilt` (`AIPlayer.cpp:3540-3553`).
    pub fn is_all_built(&self) -> bool {
        let mut still_building = false;
        for order in &self.work_orders {
            if order.num_required > order.num_completed {
                still_building = true;
            }
        }
        !still_building
    }

    /// `TeamInQueue::isBuildTimeExpired` (`AIPlayer.cpp:3557-3566`).
    pub fn is_build_time_expired(&self, frame: UnsignedInt, initial_idle_frames: Int) -> bool {
        if initial_idle_frames < 1 {
            return false;
        }
        let limit =
            (self.frame_started as UnsignedInt).wrapping_add(initial_idle_frames as UnsignedInt);
        frame > limit
    }

    /// `TeamInQueue::isMinimumBuilt` (`AIPlayer.cpp:3570-3588`).
    pub fn is_minimum_built(&self) -> bool {
        for order in &self.work_orders {
            let mut count = order.num_completed;
            if order.factory_id != ObjectId::INVALID {
                count = count.wrapping_add(1);
            }
            if order.num_required > count && order.required {
                return false;
            }
        }
        true
    }

    /// `TeamInQueue::includesADozer` (`AIPlayer.cpp:3592-3604`).
    pub fn includes_a_dozer(&self, thing_factory: &ThingFactory) -> bool {
        self.work_orders.iter().any(|order| {
            !order.is_resource_gatherer
                && order
                    .thing
                    .and_then(|id| thing_factory.find_by_template_id_final(id.0 as u16))
                    .is_some_and(|t| t.is_kind_of(KindOfType::Dozer) != FALSE)
        })
    }

    /// `TeamInQueue::areBuildsComplete` (`AIPlayer.cpp:3608-3619`).
    pub fn are_builds_complete(&self) -> bool {
        !self
            .work_orders
            .iter()
            .any(|o| o.factory_id != ObjectId::INVALID)
    }

    /// `TeamInQueue::disband` (`AIPlayer.cpp:3623-3636`).
    #[allow(clippy::too_many_arguments)]
    pub fn disband(
        &mut self,
        sim: &mut SimContext,
        ai_players: &mut BTreeMap<Int, AiPlayer>,
        teams: &mut TeamFactory,
        stores: &StartupStores,
        waypoints: &[(String, Coord3D)],
        destroyed_teams: &mut Vec<TeamId>,
    ) {
        let Some(proto) = teams.prototype_of(self.team_id) else {
            self.team_id = TeamId::INVALID;
            return;
        };
        let owner = proto.owner;
        let singleton = proto.singleton;
        let Some(default_team) = sim.world.players.player(owner).and_then(|p| p.default_team) else {
            return;
        };
        if self.team_id != default_team {
            let dead = self.team_id;
            transfer_units_to(sim, teams, dead, default_team);
            if !singleton {
                let mut unported = BTreeMap::new();
                let SimContext { rules, world, .. } = sim;
                let mut list = std::mem::take(&mut world.players);
                let mut host = FramePlayers {
                    world,
                    ai_players,
                    waypoints,
                    unported: &mut unported,
                    destroyed_teams,
                };
                let mut env = PlayerEnv {
                    templates: &stores.player_templates,
                    ranks: &stores.ranks,
                    sciences: &rules.science_store,
                    name_keys: &mut rules.name_keys,
                    world: &mut host,
                };
                teams.delete_team_instance(dead, &mut list, &mut env);
                world.players = list;
                remove_team_from_sim_world(world, dead);
            }
            self.team_id = TeamId::INVALID;
        }
    }

    /// `TeamInQueue::crc` (`AIPlayer.cpp:3641-3644`).
    pub fn crc(&self, _xfer: &mut dyn Xfer) -> Result<(), XferError> {
        Ok(())
    }

    /// `TeamInQueue::xfer` (`AIPlayer.cpp:3651-3733`).
    pub fn xfer(
        &mut self,
        xfer: &mut dyn Xfer,
        thing_factory: Option<&ThingFactory>,
        teams: Option<&TeamFactory>,
    ) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        let mut count = self.work_orders.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut count)?;
        if xfer.mode() == XferMode::Save {
            for order in &mut self.work_orders {
                order.xfer(xfer, thing_factory)?;
            }
        } else {
            if !self.work_orders.is_empty() {
                return Err(XferError::InvalidData);
            }
            for _ in 0..count {
                let mut order = WorkOrder::new();
                order.xfer(xfer, thing_factory)?;
                self.work_orders.push(order);
            }
        }
        let mut prio = Bool::from(self.priority_build);
        xfer_flag(xfer, &mut prio)?;
        self.priority_build = prio != FALSE;
        xfer_team_id(xfer, &mut self.team_id)?;
        if xfer.mode() == XferMode::Load {
            if let Some(tf) = teams {
                self.sync_prototype_cache(tf);
            }
        }
        xfer.xfer_int(&mut self.frame_started)?;
        let mut sent = Bool::from(self.sent_to_start_location);
        xfer_flag(xfer, &mut sent)?;
        self.sent_to_start_location = sent != FALSE;
        let mut stop = Bool::from(self.stop_queueing);
        xfer_flag(xfer, &mut stop)?;
        self.stop_queueing = stop != FALSE;
        let mut reinf = Bool::from(self.reinforcement);
        xfer_flag(xfer, &mut reinf)?;
        self.reinforcement = reinf != FALSE;
        xfer.xfer_object_id(&mut self.reinforcement_id)
    }
}

/// Restores `Player::can_build_units` and `SimWorld::teams_cannot_build_units`
/// after temporary unit-building enablement in `queueDozer`/`queueSupplyTruck`.
struct TempUnitBuildGuard {
    player: PlayerIndex,
    was_enabled: bool,
}

impl TempUnitBuildGuard {
    fn enable(sim: &mut SimContext, player: PlayerIndex) -> Self {
        let was_enabled = sim
            .world
            .players
            .player_mut(player)
            .is_some_and(|p| std::mem::replace(&mut p.can_build_units, true));
        Self {
            player,
            was_enabled,
        }
    }

    fn restore(self, sim: &mut SimContext) {
        if let Some(p) = sim.world.players.player_mut(self.player) {
            p.can_build_units = self.was_enabled;
        }
    }
}

/// `AIPlayer::aiPreTeamDestroy` (`AIPlayer.cpp:862-888`).
pub fn ai_pre_team_destroy(
    ai_players: &mut BTreeMap<Int, AiPlayer>,
    player: PlayerIndex,
    deleted_team: TeamId,
) {
    let Some(ai) = ai_players.get_mut(&player) else {
        return;
    };
    ai.team_build_queue.retain(|t| t.team_id != deleted_team);
    ai.team_ready_queue.retain(|t| t.team_id != deleted_team);
}

/// Synchronous `AIPlayer::onUnitProduced` hook invoked from `SimContext::unit_created`
/// (`AIPlayer.cpp:1062-1168`).
pub fn on_unit_produced_in_sim(
    sim: &mut SimContext,
    player: PlayerIndex,
    factory: ObjectSlot,
    unit: ObjectSlot,
) {
    let Some(factory_id) = sim.world.objects.id_of(factory) else {
        return;
    };
    let Some(unit_id) = sim.world.objects.id_of(unit) else {
        return;
    };
    let unit_tmpl_id = sim.object_template_id(unit);

    let mut match_info: Option<(TeamId, bool, Coord3D, bool)> = None;
    let mut found = false;
    {
        let Some(ai) = sim.world.ai_players.get_mut(&player) else {
            return;
        };
        let tf = &sim.rules.thing_factory;
        let unit_tmpl = unit_tmpl_id.and_then(|id| tf.find_by_template_id_final(id.0 as u16));
        for team in &mut ai.team_build_queue {
            if found {
                break;
            }
            for order in &mut team.work_orders {
                let order_tmpl = order.thing.and_then(|id| tf.find_by_template_id_final(id.0 as u16));
                let equiv = matches!(
                    (unit_tmpl, order_tmpl),
                    (Some(u), Some(o)) if u.is_equivalent_to(o) != FALSE
                );
                if order.factory_id == factory_id
                    && order.num_completed < order.num_required
                    && equiv
                {
                    order.num_completed = order.num_completed.wrapping_add(1);
                    if team.reinforcement {
                        team.reinforcement_id = unit_id;
                    }
                    order.factory_id = ObjectId::INVALID;
                    match_info = Some((
                        team.team_id,
                        team.has_home_location,
                        team.home_location,
                        order.is_resource_gatherer,
                    ));
                    found = true;
                    break;
                }
            }
        }
    }

    let mut supply_truck = false;
    if let Some((team_id, has_home, home_location, is_gatherer)) = match_info {
        if team_id != TeamId::INVALID {
            sim.set_object_team_same_player(unit, team_id);
        }
        if has_home && sim.world.objects.get(unit).is_some_and(|o| o.ai_slot.is_some()) {
            let (_, goal_pos) = sim.ai_goal_of(unit);
            sim.ai_follow_exit_production_path(
                unit,
                &[goal_pos, home_location],
                None,
                CommandSourceType::FromAi,
            );
        }
        if let Some(ai_ms) = sim.world.objects.get(unit).and_then(|o| o.ai_slot) {
            let mut handled_worker = false;
            if let Some(ai_mod) = sim
                .world
                .modules
                .payload_mut(ai_ms)
                .and_then(|m| m.as_any_mut())
                .and_then(|a| a.downcast_mut::<AiUpdate>())
            {
                if let DerivedAi::Worker(ref mut w) = ai_mod.derived {
                    supply_truck = is_gatherer;
                    w.force_pending = Bool::from(supply_truck);
                    handled_worker = true;
                }
            }
            if !handled_worker {
                let mut found_truck = false;
                sim.with_supply_truck(unit, &mut |truck, _, _| {
                    supply_truck = is_gatherer;
                    truck.set_force_wanting_state(Bool::from(supply_truck));
                    found_truck = true;
                });
                let _ = found_truck;
            }
            if supply_truck {
                sim.world.pending_supply_truck_dock.push((player, unit_id));
            }
        }
    }

    if !supply_truck && sim.object_is_kind_of(unit, KindOfType::Dozer) != FALSE {
        if let Some(ai) = sim.world.ai_players.get_mut(&player) {
            if ai.dozer_queued_for_repair != FALSE {
                ai.repair_dozer = unit_id;
                ai.dozer_queued_for_repair = FALSE;
            } else {
                ai.build_delay = 0;
                ai.structure_timer = 1;
            }
        }
    }

    if let Some(ai) = sim.world.ai_players.get_mut(&player) {
        ai.team_delay = 0;
    }
}

/// `AIPlayer::findFactory` (`AIPlayer.cpp:1446-1480`).
pub fn find_factory(
    sim: &mut SimContext,
    player: PlayerIndex,
    thing: ThingTemplateId,
    busy_ok: bool,
) -> Option<ObjectSlot> {
    let mut busy_factory: Option<ObjectSlot> = None;
    let len = sim.world.players.player(player)?.build_list.len();
    for at in 0..len {
        let obj_id = sim.world.players.player(player)?.build_list[at].object_id;
        let Some(factory) = sim.world.objects.slot_by_id(obj_id) else {
            continue;
        };
        let owner = sim
            .world
            .objects
            .get(factory)
            .and_then(|o| sim.controlling_player(o.team_id));
        if owner != Some(player) {
            if let Some(p) = sim.world.players.player_mut(player) {
                p.build_list[at].object_id = ObjectId::INVALID;
            }
            continue;
        }
        let Some(obj) = sim.world.objects.get(factory) else {
            continue;
        };
        if obj.status.test(ObjectStatusType::UnderConstruction.as_i32()) != FALSE {
            continue;
        }
        if obj.status.test(ObjectStatusType::Sold.as_i32()) != FALSE {
            continue;
        }
        let Some(pu) = sim.production_module(factory) else {
            continue;
        };
        if !sim.is_possible_to_make_unit(factory, thing, None) {
            continue;
        }
        if pu.production_count() == 0 {
            return Some(factory);
        }
        if busy_ok {
            busy_factory = Some(factory);
        }
    }
    if busy_ok {
        busy_factory
    } else {
        None
    }
}

/// `AIPlayer::startTraining` (`AIPlayer.cpp:1418-1439`).
pub fn start_training(
    sim: &mut SimContext,
    player: PlayerIndex,
    order: &mut WorkOrder,
    busy_ok: bool,
) -> bool {
    let Some(thing) = order.thing else {
        return false;
    };
    let Some(factory) = find_factory(sim, player, thing, busy_ok) else {
        return false;
    };
    let Some(factory_id) = sim.world.objects.id_of(factory) else {
        return false;
    };
    if sim.ai_queue_create_unit(factory, thing) {
        order.factory_id = factory_id;
        return true;
    }
    false
}

/// `AIPlayer::isPossibleToBuildTeam` (`AIPlayer.cpp:1485-1523`).
pub fn is_possible_to_build_team(
    sim: &mut SimContext,
    teams: &TeamFactory,
    player: PlayerIndex,
    proto_id: TeamPrototypeId,
    require_idle_factory: bool,
) -> (bool, bool) {
    let Some(units) = teams
        .prototype(proto_id)
        .map(|p| p.template.units.clone())
    else {
        return (false, false);
    };
    let Some(default_team) = sim.world.players.player(player).and_then(|p| p.default_team) else {
        return (false, false);
    };
    let mut any_idle = false;
    let mut cost: Int = 0;
    for unit in &units {
        let Some(thing_id) = sim.rules.thing_factory.find_template_id(&unit.unit_thing_name) else {
            continue;
        };
        let thing_cost = sim.unit_cost_to_build(default_team, thing_id);
        if find_factory(sim, player, thing_id, true).is_none() {
            return (false, false);
        }
        if find_factory(sim, player, thing_id, false).is_some() {
            any_idle = true;
        }
        let avg = (unit.max_units.wrapping_add(unit.min_units) as Real) / 2.0;
        let add = (thing_cost as Real) * avg;
        cost = ((cost as Real) + add) as Int;
    }
    cost = ((cost as Real) * sim.rules.ai_data.team_resources_to_build) as Int;
    let money = sim.player_money(default_team);
    if money < (cost as UnsignedInt) {
        return (false, true);
    }
    if any_idle || !require_idle_factory {
        return (true, false);
    }
    (false, false)
}

/// Evaluates `TeamPrototype::evaluateProductionCondition` (`Team.cpp:1130-1186`).
pub fn evaluate_production_condition(
    cx: &mut BuildContext<'_>,
    proto_id: TeamPrototypeId,
) -> bool {
    let frame = cx.sim.world.frame;
    let (Some(teams_mut), Some(scripts)) = (cx.teams.as_deref_mut(), cx.scripts.as_deref_mut())
    else {
        return false;
    };
    let Some(proto) = teams_mut.prototype_mut(proto_id) else {
        return false;
    };
    if proto.production_condition_always_false {
        return false;
    }
    let owner = proto.owner;
    let mut script = if let Some(mut cached) = proto.production_condition_script.take() {
        if frame < cached.frame_to_evaluate_at {
            proto.production_condition_script = Some(cached);
            return false;
        }
        let delay = cached.delay_evaluation_seconds;
        if delay > 0 {
            cached.frame_to_evaluate_at =
                frame.wrapping_add(delay.wrapping_mul(FRAMES_PER_SECOND) as UnsignedInt);
        }
        cached
    } else {
        if proto.template.production_condition.is_empty() {
            proto.production_condition_always_false = true;
            return false;
        }
        let Some(r) = scripts.engine.find_script(&proto.template.production_condition) else {
            proto.production_condition_always_false = true;
            return false;
        };
        let Some(p_script) = scripts.engine.script(r) else {
            proto.production_condition_always_false = true;
            return false;
        };
        let diff = cx
            .sim
            .world
            .ai_players
            .get(&owner)
            .map_or(scripts.engine.difficulty, |ai| match ai.difficulty {
                DIFFICULTY_EASY => GameDifficulty::Easy,
                DIFFICULTY_HARD => GameDifficulty::Hard,
                _ => GameDifficulty::Normal,
            });
        let allowed = match diff {
            GameDifficulty::Easy => p_script.easy != 0,
            GameDifficulty::Normal => p_script.normal != 0,
            GameDifficulty::Hard => p_script.hard != 0,
        };
        if !allowed {
            proto.production_condition_always_false = true;
            return false;
        }
        p_script.duplicate(&script_template_registry(&mut cx.sim.rules.name_keys))
    };

    let default_stores;
    let stores = match cx.stores {
        Some(s) => s,
        None => {
            default_stores = StartupStores::default();
            &default_stores
        }
    };
    let result = scripts.with_world(
        cx.sim,
        ScriptGame {
            teams: teams_mut,
            stores,
            waypoints: cx.waypoints,
            named_objects: cx.named,
        },
        |engine, world| engine.evaluate_script_conditions(world, &mut script, None, Some(owner)),
    );
    if let Some(teams_mut) = cx.teams.as_deref_mut() {
        if let Some(proto) = teams_mut.prototype_mut(proto_id) {
            proto.production_condition_script = Some(script);
        }
    }
    result
}

/// `AIPlayer::isAGoodIdeaToBuildTeam` (`AIPlayer.cpp:1529-1566`).
pub fn is_a_good_idea_to_build_team(
    cx: &mut BuildContext<'_>,
    player: PlayerIndex,
    proto_id: TeamPrototypeId,
) -> bool {
    if !evaluate_production_condition(cx, proto_id) {
        return false;
    }
    let Some(teams) = cx.teams.as_deref() else {
        return false;
    };
    let Some(proto) = teams.prototype(proto_id) else {
        return false;
    };
    if proto.count_team_instances() >= proto.template.max_instances {
        return false;
    }
    if let Some(ai) = cx.sim.world.ai_players.get(&player) {
        for q in &ai.team_build_queue {
            if teams.team(q.team_id).map(|t| t.prototype) == Some(proto_id) {
                return false;
            }
        }
    }
    let (possible, _) =
        is_possible_to_build_team(cx.sim, teams, player, proto_id, true);
    possible
}

/// `AIPlayer::selectTeamToReinforce` (`AIPlayer.cpp:1571-1683`).
pub fn select_team_to_reinforce(cx: &mut BuildContext<'_>, player: PlayerIndex, min_priority: Int) -> bool {
    let Some(teams) = cx.teams.as_deref() else {
        return false;
    };
    let Some(protos) = cx.sim.world.players.player(player).map(|p| p.team_prototypes.clone()) else {
        return false;
    };
    let mut cur_team: Option<TeamId> = None;
    let mut cur_priority = min_priority;
    let mut cur_thing: Option<ThingTemplateId> = None;

    for proto_id in protos {
        let busy = cx.sim.world.ai_players.get(&player).is_some_and(|ai| {
            ai.team_build_queue
                .iter()
                .any(|q| teams.team(q.team_id).map(|t| t.prototype) == Some(proto_id))
        });
        if busy {
            continue;
        }
        let Some(proto) = teams.prototype(proto_id) else {
            continue;
        };
        if !(proto.template.automatically_reinforce
            && proto.template.production_priority > cur_priority)
        {
            continue;
        }
        let proto_prio = proto.template.production_priority;
        let units = proto.template.units.clone();
        let instances = teams.instance_ids(proto_id);
        for team_id in instances {
            if !team_has_any_units(cx.sim, team_id) {
                continue;
            }
            for unit in &units {
                if unit.max_units < 1 {
                    continue;
                }
                let Some(thing_id) = cx
                    .sim
                    .rules
                    .thing_factory
                    .find_template_id(&unit.unit_thing_name)
                else {
                    continue;
                };
                let count = crate::player::team::count_objects_by_thing_template(
                    cx.sim,
                    team_id,
                    &[thing_id],
                    false,
                    false,
                )[0];
                if count < unit.max_units
                    && find_factory(cx.sim, player, thing_id, false).is_some()
                {
                    cur_team = Some(team_id);
                    cur_priority = proto_prio;
                    cur_thing = Some(thing_id);
                }
            }
        }
    }

    let (Some(cur_team), Some(cur_thing)) = (cur_team, cur_thing) else {
        return false;
    };
    let mut team_q = TeamInQueue::from_team(teams, cur_team);
    team_q.priority_build = false;
    team_q.reinforcement = true;
    team_q.frame_started = cx.sim.world.frame as Int;
    let mut order = WorkOrder::new();
    order.thing = Some(cur_thing);
    order.factory_id = ObjectId::INVALID;
    order.num_required = 1;
    order.required = true;

    let mut origin = teams
        .prototype_of(cur_team)
        .map_or_else(Coord3D::default, |p| p.template.home_location);
    if let Some(first_slot) = cx
        .sim
        .world
        .team_members
        .get(&cur_team)
        .and_then(|m| m.first().copied())
    {
        if let Some(pos) = cx
            .sim
            .world
            .objects
            .get(first_slot)
            .map(|o| *o.thing.get_position())
        {
            origin = pos;
        }
    }
    let max_recruit = cx.sim.rules.ai_data.max_recruit_distance;
    if let Some(unit_slot) = try_to_recruit(
        cx.sim,
        teams,
        cur_team,
        cur_thing,
        &origin,
        max_recruit,
    ) {
        order.num_completed = 1;
        let unit_id = cx
            .sim
            .world
            .objects
            .id_of(unit_slot)
            .unwrap_or(ObjectId::INVALID);
        cx.sim.set_object_team_same_player(unit_slot, cur_team);
        if let Some(tf) = cx.teams.as_deref_mut() {
            flush_pending_team_member_moves(cx.sim, tf);
        }
        team_q.reinforcement_id = unit_id;
        cx.sim.ai_idle(unit_slot, CommandSourceType::FromAi);
    } else {
        start_training(cx.sim, player, &mut order, team_q.priority_build);
    }
    team_q.work_orders.push(order);

    if let Some(ai) = ai_mut(cx, player) {
        ai.team_build_queue.insert(0, team_q);
        ai.team_delay = 0;
    }
    true
}

/// `AIPlayer::selectTeamToBuild` (`AIPlayer.cpp:1688-1769`).
pub fn select_team_to_build(cx: &mut BuildContext<'_>, player: PlayerIndex) -> bool {
    let Some(protos) = cx.sim.world.players.player(player).map(|p| p.team_prototypes.clone()) else {
        return false;
    };
    const INVALID_PRI: Int = -99999;
    let mut hi_pri = INVALID_PRI;
    let mut candidates1: Vec<(TeamPrototypeId, Int)> = Vec::new();
    for proto_id in protos {
        if is_a_good_idea_to_build_team(cx, player, proto_id) {
            let pri = cx
                .teams
                .as_deref()
                .and_then(|t| t.prototype(proto_id))
                .map_or(0, |p| p.template.production_priority);
            candidates1.push((proto_id, pri));
            if pri > hi_pri {
                hi_pri = pri;
            }
        }
    }
    if select_team_to_reinforce(cx, player, hi_pri) {
        return true;
    }
    if hi_pri == INVALID_PRI {
        return false;
    }
    let candidates: Vec<TeamPrototypeId> = candidates1
        .into_iter()
        .filter(|&(_, pri)| pri == hi_pri)
        .map(|(id, _)| id)
        .collect();
    if candidates.is_empty() {
        return false;
    }
    let which = cx
        .sim
        .world
        .rng
        .get_game_logic_random_value(0, (candidates.len() as Int).wrapping_sub(1));
    let Some(&proto_id) = candidates.get(which as usize) else {
        return false;
    };
    build_specific_ai_team(cx, player, proto_id, false);
    let default_team = cx.sim.world.players.player(player).and_then(|p| p.default_team);
    let money = default_team.map_or(0, |t| cx.sim.player_money(t));
    let poor = cx.sim.rules.ai_data.resources_poor as UnsignedInt;
    let wealthy = cx.sim.rules.ai_data.resources_wealthy as UnsignedInt;
    let poor_mod = cx.sim.rules.ai_data.team_poor_mod;
    let wealthy_mod = cx.sim.rules.ai_data.team_wealthy_mod;
    if let Some(ai) = ai_mut(cx, player) {
        ai.ready_to_build_team = FALSE;
        ai.team_timer = ai.team_seconds.wrapping_mul(FRAMES_PER_SECOND);
        if money < poor && poor_mod != 0.0 {
            ai.team_timer = (ai.team_timer as Real / poor_mod) as Int;
        } else if money > wealthy && wealthy_mod != 0.0 {
            ai.team_timer = (ai.team_timer as Real / wealthy_mod) as Int;
        }
    }
    true
}

/// Core of `AIPlayer::buildSpecificAITeam` (`AIPlayer.cpp:2453-2575`) shared
/// between `BuildContext` and `LiveWorld`.
#[allow(clippy::too_many_arguments)]
pub fn build_specific_ai_team_with_env(
    sim: &mut SimContext,
    ai_players: &mut BTreeMap<Int, AiPlayer>,
    teams: &mut TeamFactory,
    stores: &StartupStores,
    waypoints: &[(String, Coord3D)],
    destroyed_teams: &mut Vec<TeamId>,
    player: PlayerIndex,
    proto_id: TeamPrototypeId,
    priority_build: bool,
) {
    if !sim.world.players.player(player).is_some_and(|p| p.can_build_units) {
        return;
    }
    let Some(proto) = teams.prototype(proto_id) else {
        return;
    };
    let proto_name = proto.name.clone();
    let is_singleton = proto.singleton;
    let units_info = proto.template.units.clone();
    let execute_actions = proto.template.execute_actions;
    let production_condition = proto.template.production_condition.clone();
    if priority_build && is_singleton {
        if let Some(team) = proto.instances.first() {
            if team_has_any_objects(sim, team.id) {
                return;
            }
        }
    }
    let (possible, need_money) =
        is_possible_to_build_team(sim, teams, player, proto_id, false);
    if !possible && !need_money {
        return;
    }

    let mut orders: Vec<WorkOrder> = Vec::new();
    // Optional units first, only if max - min > 0, each prepended.
    for u in &units_info {
        if let Some(thing) = sim.rules.thing_factory.find_template_id(&u.unit_thing_name) {
            let count = u.max_units.wrapping_sub(u.min_units);
            if count > 0 {
                let mut order = WorkOrder::new();
                order.thing = Some(thing);
                order.factory_id = ObjectId::INVALID;
                order.num_required = count;
                orders.insert(0, order);
            }
        }
    }
    // Required units second, even if min == 0, each prepended.
    for u in &units_info {
        if let Some(thing) = sim.rules.thing_factory.find_template_id(&u.unit_thing_name) {
            let mut order = WorkOrder::new();
            order.thing = Some(thing);
            order.factory_id = ObjectId::INVALID;
            order.num_required = u.min_units;
            order.required = true;
            orders.insert(0, order);
        }
    }
    if orders.is_empty() {
        return;
    }

    let mut unported = BTreeMap::new();
    let created_team = {
        let SimContext { rules, world, .. } = sim;
        let mut list = std::mem::take(&mut world.players);
        let mut host = FramePlayers {
            world,
            ai_players,
            waypoints,
            unported: &mut unported,
            destroyed_teams,
        };
        let mut env = PlayerEnv {
            templates: &stores.player_templates,
            ranks: &stores.ranks,
            sciences: &rules.science_store,
            name_keys: &mut rules.name_keys,
            world: &mut host,
        };
        let created = teams.create_inactive_team(&proto_name, &mut list, &mut env);
        world.players = list;
        created
    };
    let Some(team_id) = created_team else {
        return;
    };

    let mut team_q = TeamInQueue::from_team(teams, team_id);
    team_q.work_orders = orders;
    team_q.frame_started = sim.world.frame as Int;
    team_q.priority_build = priority_build;

    if let Some(ai) = ai_players.get_mut(&player) {
        if priority_build {
            ai.team_build_queue.insert(0, team_q);
        } else {
            ai.team_build_queue.push(team_q);
        }
        ai.team_delay = 0;
    }

    if execute_actions && !production_condition.is_empty() {
        sim.world
            .pending_team_actions
            .push((production_condition, Some(team_id)));
    }
}

/// Runs `AIPlayer::buildSpecificAITeam` within `BuildContext` and drains any
/// `executeActions` scripts immediately when `cx.scripts` is present.
pub fn build_specific_ai_team(
    cx: &mut BuildContext<'_>,
    player: PlayerIndex,
    proto_id: TeamPrototypeId,
    priority_build: bool,
) {
    let default_stores;
    let stores = match cx.stores {
        Some(s) => s,
        None => {
            default_stores = StartupStores::default();
            &default_stores
        }
    };
    if let Some(teams) = cx.teams.as_deref_mut() {
        let mut destroyed = Vec::new();
        with_ai_players(cx.sim, |sim, ai_players| {
            build_specific_ai_team_with_env(
                sim,
                ai_players,
                teams,
                stores,
                cx.waypoints,
                &mut destroyed,
                player,
                proto_id,
                priority_build,
            );
        });
        notify_destroyed_teams(cx, &destroyed);
    }
    drain_pending_team_actions(cx);
}

/// Core of `AIPlayer::recruitSpecificAITeam` / `AISkirmishPlayer::recruitSpecificAITeam`
/// (`AIPlayer.cpp:2580-2679`, `AISkirmishPlayer.cpp:750-849`).
#[allow(clippy::too_many_arguments)]
pub fn recruit_specific_ai_team_with_env(
    sim: &mut SimContext,
    ai_players: &mut BTreeMap<Int, AiPlayer>,
    teams: &mut TeamFactory,
    stores: &StartupStores,
    waypoints: &[(String, Coord3D)],
    destroyed_teams: &mut Vec<TeamId>,
    player: PlayerIndex,
    proto_id: TeamPrototypeId,
    mut recruit_radius: Real,
) {
    if recruit_radius < 1.0 {
        recruit_radius = 99999.0;
    }
    let Some(proto) = teams.prototype(proto_id) else {
        return;
    };
    let proto_name = proto.name.clone();
    let is_singleton = proto.singleton;
    let home_location = proto.template.home_location;
    let units_info = proto.template.units.clone();
    if is_singleton {
        if let Some(team) = proto.instances.first() {
            if team_has_any_objects(sim, team.id) {
                return;
            }
        }
    }

    let mut unported = BTreeMap::new();
    let created = {
        let SimContext { rules, world, .. } = sim;
        let mut list = std::mem::take(&mut world.players);
        let mut host = FramePlayers {
            world,
            ai_players,
            waypoints,
            unported: &mut unported,
            destroyed_teams,
        };
        let mut env = PlayerEnv {
            templates: &stores.player_templates,
            ranks: &stores.ranks,
            sciences: &rules.science_store,
            name_keys: &mut rules.name_keys,
            world: &mut host,
        };
        let created = teams.create_inactive_team(&proto_name, &mut list, &mut env);
        world.players = list;
        created
    };
    let Some(the_team) = created else {
        return;
    };

    let mut units_recruited = 0;
    for u in &units_info {
        let Some(thing) = sim.rules.thing_factory.find_template_id(&u.unit_thing_name) else {
            continue;
        };
        let mut count = u.max_units;
        while count > 0 {
            let max_dist = recruit_radius;
            let Some(unit_slot) = try_to_recruit(
                sim,
                teams,
                the_team,
                thing,
                &home_location,
                max_dist,
            ) else {
                break;
            };
            units_recruited += 1;
            sim.set_object_team_same_player(unit_slot, the_team);
            flush_pending_team_member_moves(sim, teams);
            if sim
                .world
                .objects
                .get(unit_slot)
                .is_some_and(|o| o.ai_slot.is_some())
            {
                sim.ai_move_to_position(unit_slot, &home_location, CommandSourceType::FromAi);
            }
            count -= 1;
        }
    }

    if units_recruited > 0 {
        let mut team_q = TeamInQueue::from_team(teams, the_team);
        team_q.priority_build = false;
        team_q.work_orders.clear();
        team_q.frame_started = sim.world.frame as Int;
        if let Some(ai) = ai_players.get_mut(&player) {
            ai.team_ready_queue.insert(0, team_q);
        }
    } else if !is_singleton {
        let SimContext { rules, world, .. } = sim;
        let mut list = std::mem::take(&mut world.players);
        let mut host = FramePlayers {
            world,
            ai_players,
            waypoints,
            unported: &mut unported,
            destroyed_teams,
        };
        let mut env = PlayerEnv {
            templates: &stores.player_templates,
            ranks: &stores.ranks,
            sciences: &rules.science_store,
            name_keys: &mut rules.name_keys,
            world: &mut host,
        };
        teams.delete_team_instance(the_team, &mut list, &mut env);
        world.players = list;
        remove_team_from_sim_world(world, the_team);
    }
}

/// `AIPlayer::queueSupplyTruck` (`AIPlayer.cpp:223-429`).
pub fn queue_supply_truck(cx: &mut BuildContext<'_>, player: PlayerIndex) {
    if cx
        .sim
        .world
        .ai_players
        .get(&player)
        .is_some_and(|ai| {
            ai.team_build_queue
                .iter()
                .any(|t| t.work_orders.iter().any(|o| o.is_resource_gatherer))
        })
    {
        return;
    }

    let mut total_harvesters: Int = 0;
    let player_slots: Vec<ObjectSlot> = collect_player_harvesters(cx, player);
    for slot in &player_slots {
        if has_supply_truck_ai(cx.sim, *slot) {
            total_harvesters = total_harvesters.wrapping_add(1);
        }
    }

    let build_len = cx.sim.world.players.player(player).map_or(0, |p| p.build_list.len());
    for at in 0..build_len {
        let Some((is_supply, desired_gatherers, cur_gatherers, center_id)) = cx
            .sim
            .world
            .players
            .player(player)
            .and_then(|p| p.build_list.get(at))
            .map(|i| {
                (
                    i.is_supply_building != FALSE,
                    i.desired_gatherers,
                    i.current_gatherers,
                    i.object_id,
                )
            })
        else {
            continue;
        };
        if !is_supply {
            continue;
        }
        if cur_gatherers >= desired_gatherers {
            let Some(center_slot) = cx.sim.world.objects.slot_by_id(center_id) else {
                continue;
            };
            if cx.sim.object_is_kind_of(center_slot, KindOfType::RebuildHole) != FALSE {
                continue;
            }
            if !supply_center_has_supplies(cx.sim, player, center_slot) {
                continue;
            }
            crate::ai::ai_player::check_for_supply_center(cx, player, at, center_slot);
            let mut counted: Int = 0;
            for &slot in &collect_player_harvesters(cx, player) {
                if let Some((preferred_dock, ferrying, _)) = supply_truck_state(cx.sim, slot) {
                    if preferred_dock == center_id {
                        counted = counted.wrapping_add(1);
                        if !ferrying {
                            cx.sim.ai_dock(slot, center_id, CommandSourceType::FromPlayer);
                        }
                    }
                }
            }
            if let Some(p) = cx.sim.world.players.player_mut(player) {
                if let Some(info) = p.build_list.get_mut(at) {
                    info.current_gatherers = counted;
                }
            }
        } else {
            for &slot in &collect_player_harvesters(cx, player) {
                let Some((preferred_dock, ferrying, forced)) = supply_truck_state(cx.sim, slot)
                else {
                    continue;
                };
                if cx.sim.world.objects.slot_by_id(preferred_dock).is_some() {
                    continue;
                }
                if ferrying || forced {
                    if cx.sim.world.objects.slot_by_id(center_id).is_some() {
                        if let Some(info) = cx
                            .sim
                            .world
                            .players
                            .player_mut(player)
                            .and_then(|p| p.build_list.get_mut(at))
                        {
                            info.current_gatherers = info.current_gatherers.wrapping_add(1);
                        }
                        cx.sim
                            .ai_dock(slot, center_id, CommandSourceType::FromPlayer);
                        return;
                    }
                }
            }
            if total_harvesters >= desired_gatherers.wrapping_mul(3) {
                continue;
            }
            let guard = TempUnitBuildGuard::enable(cx.sim, player);
            let harvester_ids: Vec<ThingTemplateId> = cx
                .sim
                .rules
                .thing_factory
                .templates_in_first_order()
                .filter(|t| t.is_kind_of(KindOfType::Harvester) != FALSE)
                .map(|t| ThingTemplateId(t.get_template_id() as u32))
                .collect();
            for tmpl_id in harvester_ids {
                let Some(factory) = find_factory(cx.sim, player, tmpl_id, false) else {
                    continue;
                };
                let Some(factory_id) = cx.sim.world.objects.id_of(factory) else {
                    continue;
                };
                let default_team = cx
                    .sim
                    .world
                    .players
                    .player(player)
                    .and_then(|p| p.default_team)
                    .unwrap_or(TeamId::INVALID);
                let mut order = WorkOrder::new();
                order.thing = Some(tmpl_id);
                order.factory_id = ObjectId::INVALID;
                order.num_required = 1;
                order.required = true;
                order.is_resource_gatherer = true;

                let mut team_q = cx
                    .teams
                    .as_deref()
                    .map_or_else(TeamInQueue::new, |tf| TeamInQueue::from_team(tf, default_team));
                team_q.team_id = default_team;
                team_q.priority_build = true;
                team_q.frame_started = cx.sim.world.frame as Int;

                let initial = cx
                    .sim
                    .world
                    .players
                    .player(player)
                    .and_then(|p| p.build_list.get(at))
                    .is_some_and(|i| i.current_gatherers == -1);
                if initial {
                    order.factory_id = factory_id;
                    if let Some(info) = cx
                        .sim
                        .world
                        .players
                        .player_mut(player)
                        .and_then(|p| p.build_list.get_mut(at))
                    {
                        info.current_gatherers = 0;
                    }
                } else {
                    start_training(cx.sim, player, &mut order, team_q.priority_build);
                }
                team_q.work_orders.push(order);
                if let Some(ai) = ai_mut(cx, player) {
                    ai.team_build_queue.insert(0, team_q);
                    ai.team_delay = 0;
                }
                break;
            }
            guard.restore(cx.sim);
        }
    }
}

/// `AIPlayer::queueUnits` (`AIPlayer.cpp:2694-2757`).
pub fn queue_units(cx: &mut BuildContext<'_>, player: PlayerIndex) {
    queue_supply_truck(cx, player);

    let queue_len = cx
        .sim
        .world
        .ai_players
        .get(&player)
        .map_or(0, |ai| ai.team_build_queue.len());
    for ti in 0..queue_len {
        let orders_len = cx
            .sim
            .world
            .ai_players
            .get(&player)
            .and_then(|ai| ai.team_build_queue.get(ti))
            .map_or(0, |t| t.work_orders.len());
        for oi in 0..orders_len {
            let Some((team_id, priority_build, has_team_home, team_home, thing_opt)) = cx
                .sim
                .world
                .ai_players
                .get(&player)
                .and_then(|ai| ai.team_build_queue.get(ti))
                .and_then(|t| {
                    t.work_orders
                        .get(oi)
                        .map(|o| (t.team_id, t.priority_build, t.has_home_location, t.home_location, o.thing))
                })
            else {
                continue;
            };
            let (has_home, home) = if has_team_home {
                (true, team_home)
            } else if let Some(ai) = cx.sim.world.ai_players.get(&player) {
                if ai.base_center_set != FALSE {
                    (true, ai.base_center)
                } else {
                    (false, Coord3D::default())
                }
            } else {
                (false, Coord3D::default())
            };

            loop {
                let waiting = cx
                    .sim
                    .world
                    .ai_players
                    .get(&player)
                    .and_then(|ai| ai.team_build_queue.get(ti))
                    .and_then(|t| t.work_orders.get(oi))
                    .is_some_and(WorkOrder::is_waiting_to_build);
                if !waiting {
                    break;
                }
                let (Some(thing_id), Some(teams)) = (thing_opt, cx.teams.as_deref()) else {
                    break;
                };
                let max_dist = cx.sim.rules.ai_data.max_recruit_distance;
                let Some(unit_slot) = try_to_recruit(
                    cx.sim,
                    teams,
                    team_id,
                    thing_id,
                    &home,
                    max_dist,
                ) else {
                    break;
                };
                if let Some(order) = cx
                    .sim
                    .world
                    .ai_players
                    .get_mut(&player)
                    .and_then(|ai| ai.team_build_queue.get_mut(ti))
                    .and_then(|t| t.work_orders.get_mut(oi))
                {
                    order.num_completed = order.num_completed.wrapping_add(1);
                }
                cx.sim.set_object_team_same_player(unit_slot, team_id);
                if let Some(tf) = cx.teams.as_deref_mut() {
                    flush_pending_team_member_moves(cx.sim, tf);
                }
                if has_home {
                    cx.sim
                        .ai_move_to_position(unit_slot, &home, CommandSourceType::FromAi);
                } else {
                    cx.sim.ai_idle(unit_slot, CommandSourceType::FromAi);
                }
            }

            let waiting = cx
                .sim
                .world
                .ai_players
                .get(&player)
                .and_then(|ai| ai.team_build_queue.get(ti))
                .and_then(|t| t.work_orders.get(oi))
                .is_some_and(WorkOrder::is_waiting_to_build);
            if waiting {
                if let Some(mut order) = cx
                    .sim
                    .world
                    .ai_players
                    .get(&player)
                    .and_then(|ai| ai.team_build_queue.get(ti))
                    .and_then(|t| t.work_orders.get(oi))
                    .cloned()
                {
                    start_training(cx.sim, player, &mut order, priority_build);
                    if let Some(dst) = cx
                        .sim
                        .world
                        .ai_players
                        .get_mut(&player)
                        .and_then(|ai| ai.team_build_queue.get_mut(ti))
                        .and_then(|t| t.work_orders.get_mut(oi))
                    {
                        *dst = order;
                    }
                }
            } else if let Some(mut order) = cx
                .sim
                .world
                .ai_players
                .get(&player)
                .and_then(|ai| ai.team_build_queue.get(ti))
                .and_then(|t| t.work_orders.get(oi))
                .cloned()
            {
                order.validate_factory(cx.sim, player);
                if let Some(dst) = cx
                    .sim
                    .world
                    .ai_players
                    .get_mut(&player)
                    .and_then(|ai| ai.team_build_queue.get_mut(ti))
                    .and_then(|t| t.work_orders.get_mut(oi))
                {
                    *dst = order;
                }
            }
        }
    }
}

/// `AIPlayer::dozerInQueue` (`AIPlayer.cpp:3177-3191`).
pub fn dozer_in_queue(sim: &SimContext, player: PlayerIndex) -> bool {
    let tf = &sim.rules.thing_factory;
    sim.world
        .ai_players
        .get(&player)
        .is_some_and(|ai| ai.team_build_queue.iter().any(|t| t.includes_a_dozer(tf)))
}

/// `AIPlayer::queueDozer` (`AIPlayer.cpp:3197-3240`).
pub fn queue_dozer(cx: &mut BuildContext<'_>, player: PlayerIndex) {
    if dozer_in_queue(cx.sim, player) {
        return;
    }
    let guard = TempUnitBuildGuard::enable(cx.sim, player);
    let dozer_ids: Vec<ThingTemplateId> = cx
        .sim
        .rules
        .thing_factory
        .templates_in_first_order()
        .filter(|t| t.is_kind_of(KindOfType::Dozer) != FALSE)
        .map(|t| ThingTemplateId(t.get_template_id() as u32))
        .collect();
    for tmpl_id in dozer_ids {
        if find_factory(cx.sim, player, tmpl_id, true).is_none() {
            continue;
        }
        let default_team = cx
            .sim
            .world
            .players
            .player(player)
            .and_then(|p| p.default_team)
            .unwrap_or(TeamId::INVALID);
        let mut order = WorkOrder::new();
        order.thing = Some(tmpl_id);
        order.factory_id = ObjectId::INVALID;
        order.num_required = 1;
        order.required = true;
        order.is_resource_gatherer = false;

        let mut team_q = cx
            .teams
            .as_deref()
            .map_or_else(TeamInQueue::new, |tf| TeamInQueue::from_team(tf, default_team));
        team_q.team_id = default_team;
        team_q.priority_build = true;
        team_q.frame_started = cx.sim.world.frame as Int;

        start_training(cx.sim, player, &mut order, team_q.priority_build);
        team_q.work_orders.push(order);
        if let Some(ai) = ai_mut(cx, player) {
            ai.team_build_queue.insert(0, team_q);
            ai.team_delay = 0;
        }
        break;
    }
    guard.restore(cx.sim);
}

/// `AIPlayer::checkReadyTeams` (`AIPlayer.cpp:2794-2870`).
pub fn check_ready_teams(cx: &mut BuildContext<'_>, player: PlayerIndex) {
    let mut idx = 0;
    loop {
        let len = cx
            .sim
            .world
            .ai_players
            .get(&player)
            .map_or(0, |ai| ai.team_ready_queue.len());
        if idx >= len {
            break;
        }
        let (frame_started, reinforcement, reinforcement_id, team_id) = {
            let Some(team) = cx
                .sim
                .world
                .ai_players
                .get(&player)
                .and_then(|ai| ai.team_ready_queue.get(idx))
            else {
                break;
            };
            (
                team.frame_started,
                team.reinforcement,
                team.reinforcement_id,
                team.team_id,
            )
        };
        let frame = cx.sim.world.frame;
        let time_expired =
            (frame_started as UnsignedInt).wrapping_add(60 * FRAMES_PER_SECOND as UnsignedInt)
                < frame;
        let mut all_idle = true;
        let mut any_idle = false;
        if reinforcement {
            if let Some(slot) = cx.sim.world.objects.slot_by_id(reinforcement_id) {
                if cx
                    .sim
                    .world
                    .objects
                    .get(slot)
                    .is_some_and(|o| o.ai_slot.is_some())
                {
                    all_idle = cx.sim.ai_is_idle(slot) != FALSE;
                    any_idle = all_idle;
                }
            }
        } else {
            all_idle = team_is_idle(cx.sim, team_id);
            if let Some(members) = cx.sim.world.team_members.get(&team_id) {
                for &slot in members {
                    if cx
                        .sim
                        .world
                        .objects
                        .get(slot)
                        .is_some_and(|o| o.ai_slot.is_some())
                        && cx.sim.ai_is_idle(slot) != FALSE
                    {
                        any_idle = true;
                    }
                }
            }
        }
        if any_idle {
            let has_start_action = cx
                .teams
                .as_deref()
                .and_then(|tf| tf.prototype_of(team_id))
                .filter(|p| p.template.execute_actions)
                .and_then(|p| {
                    cx.scripts
                        .as_deref()
                        .and_then(|s| s.engine.find_script(&p.template.production_condition))
                        .and_then(|r| cx.scripts.as_deref().and_then(|s| s.engine.script(r)))
                        .filter(|sc| !sc.actions.is_empty())
                })
                .is_some();
            if has_start_action {
                all_idle = true;
            }
        }
        if time_expired {
            all_idle = true;
        }
        if !all_idle {
            idx += 1;
            continue;
        }

        let item = {
            let Some(ai) = ai_mut(cx, player) else {
                break;
            };
            ai.team_ready_queue.remove(idx)
        };
        let is_skirmish = cx
            .sim
            .world
            .ai_players
            .get(&player)
            .is_some_and(|ai| ai.skirmish.is_some());
        if item.reinforcement {
            if let Some(slot) = cx.sim.world.objects.slot_by_id(item.reinforcement_id) {
                if cx
                    .sim
                    .world
                    .objects
                    .get(slot)
                    .is_some_and(|o| o.ai_slot.is_some())
                {
                    cx.sim.ai_join_team(slot);
                }
            }
        } else {
            if let Some(tf) = cx.teams.as_deref_mut() {
                if let Some(t) = tf.team_mut(item.team_id) {
                    t.set_active();
                }
            }
            if is_skirmish {
                if let Some(scripts) = cx.scripts.as_deref_mut() {
                    scripts.engine.clear_team_flags();
                }
            }
        }
        idx = 0;
    }
}

/// `AIPlayer::checkQueuedTeams` (`AIPlayer.cpp:2876-2939`).
pub fn check_queued_teams(cx: &mut BuildContext<'_>, player: PlayerIndex) {
    let default_stores;
    let stores = match cx.stores {
        Some(s) => s,
        None => {
            default_stores = StartupStores::default();
            &default_stores
        }
    };

    // Pass 1: handle expired build times.
    let mut idx = 0;
    loop {
        let len = cx
            .sim
            .world
            .ai_players
            .get(&player)
            .map_or(0, |ai| ai.team_build_queue.len());
        if idx >= len {
            break;
        }
        let initial_idle_frames = cx
            .sim
            .world
            .ai_players
            .get(&player)
            .and_then(|ai| ai.team_build_queue.get(idx))
            .and_then(|t| {
                cx.teams
                    .as_deref()
                    .and_then(|tf| tf.prototype_of(t.team_id))
                    .map(|p| p.template.initial_idle_frames)
            })
            .unwrap_or(0);
        let frame = cx.sim.world.frame;
        let Some((expired, min_built, builds_complete)) = cx
            .sim
            .world
            .ai_players
            .get(&player)
            .and_then(|ai| ai.team_build_queue.get(idx))
            .map(|t| {
                (
                    t.is_build_time_expired(frame, initial_idle_frames),
                    t.is_minimum_built(),
                    t.are_builds_complete(),
                )
            })
        else {
            break;
        };
        if !expired {
            idx += 1;
            continue;
        }
        if min_built {
            if builds_complete {
                if let Some(ai) = ai_mut(cx, player) {
                    let item = ai.team_build_queue.remove(idx);
                    ai.team_ready_queue.insert(0, item);
                }
                idx = 0;
            } else {
                idx += 1;
            }
        } else {
            let mut item = match ai_mut(cx, player) {
                Some(ai) => ai.team_build_queue.remove(idx),
                None => break,
            };
            let mut destroyed = Vec::new();
            if let Some(teams) = cx.teams.as_deref_mut() {
                with_ai_players(cx.sim, |sim, ai_players| {
                    item.disband(
                        sim,
                        ai_players,
                        teams,
                        stores,
                        cx.waypoints,
                        &mut destroyed,
                    );
                });
            }
            notify_destroyed_teams(cx, &destroyed);
            let is_skirmish = cx
                .sim
                .world
                .ai_players
                .get(&player)
                .is_some_and(|ai| ai.skirmish.is_some());
            if is_skirmish {
                if let Some(scripts) = cx.scripts.as_deref_mut() {
                    scripts.engine.clear_team_flags();
                }
            }
            idx = 0;
        }
    }

    // Pass 2: promote completed teams, or execute `executeActions` if any member is idle.
    let mut idx = 0;
    loop {
        let len = cx
            .sim
            .world
            .ai_players
            .get(&player)
            .map_or(0, |ai| ai.team_build_queue.len());
        if idx >= len {
            break;
        }
        let Some((all_built, team_id)) = cx
            .sim
            .world
            .ai_players
            .get(&player)
            .and_then(|ai| ai.team_build_queue.get(idx))
            .map(|t| (t.is_all_built(), t.team_id))
        else {
            break;
        };
        if all_built {
            if let Some(ai) = ai_mut(cx, player) {
                let item = ai.team_build_queue.remove(idx);
                ai.team_ready_queue.insert(0, item);
            }
            idx = 0;
            continue;
        }
        let mut any_idle = false;
        if let Some(members) = cx.sim.world.team_members.get(&team_id) {
            for &slot in members {
                if cx
                    .sim
                    .world
                    .objects
                    .get(slot)
                    .is_some_and(|o| o.ai_slot.is_some())
                    && cx.sim.ai_is_idle(slot) != FALSE
                {
                    any_idle = true;
                }
            }
        }
        if any_idle {
            let cond = cx
                .teams
                .as_deref()
                .and_then(|tf| tf.prototype_of(team_id))
                .filter(|p| p.template.execute_actions)
                .map(|p| p.template.production_condition.clone());
            if let Some(cond) = cond {
                if !cond.is_empty() {
                    cx.sim
                        .world
                        .pending_team_actions
                        .push((cond, Some(team_id)));
                    drain_pending_team_actions(cx);
                }
            }
        }
        idx += 1;
    }
}

/// `AIPlayer::doTeamBuilding` (`AIPlayer.cpp:2945-2970`) and
/// `AISkirmishPlayer::doTeamBuilding` (`AISkirmishPlayer.cpp:921-949`).
pub fn do_team_building(cx: &mut BuildContext<'_>, player: PlayerIndex) {
    if !cx.sim.world.players.player(player).is_some_and(|p| p.can_build_units) {
        return;
    }
    let Some(ai) = ai_mut(cx, player) else {
        return;
    };
    let skirmish = ai.skirmish.is_some();
    if ai.ready_to_build_team == FALSE {
        ai.team_timer = ai.team_timer.wrapping_sub(1);
        if ai.team_timer <= 0 {
            ai.ready_to_build_team = TRUE;
            ai.team_delay = 0;
        }
        if skirmish && ai.team_timer > 3 * FRAMES_PER_SECOND {
            ai.team_timer = 3 * FRAMES_PER_SECOND;
        }
    }
    ai.team_delay = ai.team_delay.wrapping_sub(1);
    if ai.team_delay >= 1 {
        return;
    }
    queue_units(cx, player);
    let ready = cx
        .sim
        .world
        .ai_players
        .get(&player)
        .is_some_and(|ai| ai.ready_to_build_team != FALSE);
    if ready {
        process_team_building(cx, player);
    }
    if let Some(ai) = ai_mut(cx, player) {
        ai.team_delay = if skirmish {
            2 * FRAMES_PER_SECOND
        } else {
            5 * FRAMES_PER_SECOND
        };
    }
}

/// `AIPlayer::processTeamBuilding` (`AIPlayer.cpp:2684-2690`).
pub fn process_team_building(cx: &mut BuildContext<'_>, player: PlayerIndex) {
    if select_team_to_build(cx, player) {
        queue_units(cx, player);
    }
}

/// Executes queued `pending_team_actions` through `ScriptEngine`.
pub fn drain_pending_team_actions(cx: &mut BuildContext<'_>) {
    while !cx.sim.world.pending_team_actions.is_empty() {
        let pending = std::mem::take(&mut cx.sim.world.pending_team_actions);
        let (Some(scripts), Some(teams)) = (cx.scripts.as_deref_mut(), cx.teams.as_deref_mut())
        else {
            return;
        };
        let default_stores;
        let stores = match cx.stores {
            Some(s) => s,
            None => {
                default_stores = StartupStores::default();
                &default_stores
            }
        };
        scripts.with_world(
            cx.sim,
            ScriptGame {
                teams,
                stores,
                waypoints: cx.waypoints,
                named_objects: cx.named,
            },
            |engine, world| {
                for (name, team) in pending {
                    if let Some(r) = engine.find_script(&name) {
                        engine.execute_actions_for_team(
                            world,
                            r,
                            team.map(|t| t.0 as ScriptTeamId),
                        );
                    }
                }
            },
        );
    }
}

/// Drains `pending_structures_produced`, `pending_supply_truck_dock`, and
/// `pending_team_member_moves` for `player` at the start of `ai_player::update`.
pub fn drain_pending_sim_events_for_player(cx: &mut BuildContext<'_>, player: PlayerIndex) {
    if let Some(teams) = cx.teams.as_deref_mut() {
        flush_pending_team_member_moves(cx.sim, teams);
    }

    let mut mine_structures = Vec::new();
    let mut other_structures = Vec::new();
    for (p, id) in std::mem::take(&mut cx.sim.world.pending_structures_produced) {
        if p == player {
            mine_structures.push(id);
        } else {
            other_structures.push((p, id));
        }
    }
    cx.sim.world.pending_structures_produced = other_structures;
    for id in mine_structures {
        if let Some(slot) = cx.sim.world.objects.slot_by_id(id) {
            crate::ai::ai_player::on_structure_produced(cx, player, slot);
        }
    }

    let mut mine_trucks = Vec::new();
    let mut other_trucks = Vec::new();
    for (p, id) in std::mem::take(&mut cx.sim.world.pending_supply_truck_dock) {
        if p == player {
            mine_trucks.push(id);
        } else {
            other_trucks.push((p, id));
        }
    }
    cx.sim.world.pending_supply_truck_dock = other_trucks;
    for unit_id in mine_trucks {
        let Some(unit_slot) = cx.sim.world.objects.slot_by_id(unit_id) else {
            continue;
        };
        let len = cx.sim.world.players.player(player).map_or(0, |p| p.build_list.len());
        for at in 0..len {
            let center_id = {
                let Some(info) = cx
                    .sim
                    .world
                    .players
                    .player(player)
                    .and_then(|p| p.build_list.get(at))
                else {
                    continue;
                };
                if info.is_supply_building != FALSE
                    && info.desired_gatherers > 0
                    && info.desired_gatherers > info.current_gatherers
                {
                    info.object_id
                } else {
                    continue;
                }
            };
            if cx.sim.world.objects.slot_by_id(center_id).is_some() {
                if let Some(info) = cx
                    .sim
                    .world
                    .players
                    .player_mut(player)
                    .and_then(|p| p.build_list.get_mut(at))
                {
                    info.current_gatherers = info.current_gatherers.wrapping_add(1);
                }
                cx.sim
                    .ai_dock(unit_slot, center_id, CommandSourceType::FromPlayer);
            }
        }
    }
}

/// Temporarily moves `sim.world.ai_players` out and restores it unconditionally.
fn with_ai_players<R>(
    sim: &mut SimContext,
    f: impl FnOnce(&mut SimContext, &mut BTreeMap<Int, AiPlayer>) -> R,
) -> R {
    struct Restore<'a>(&'a mut SimContext, BTreeMap<Int, AiPlayer>);
    impl Drop for Restore<'_> {
        fn drop(&mut self) {
            self.0.world.ai_players = std::mem::take(&mut self.1);
        }
    }
    let taken = std::mem::take(&mut sim.world.ai_players);
    let mut guard = Restore(sim, taken);
    f(guard.0, &mut guard.1)
}

fn notify_destroyed_teams(cx: &mut BuildContext<'_>, destroyed: &[TeamId]) {
    if destroyed.is_empty() {
        return;
    }
    if let Some(scripts) = cx.scripts.as_deref_mut() {
        for &team in destroyed {
            scripts
                .engine
                .notify_of_team_destruction(team.0 as ScriptTeamId);
        }
    }
}

fn team_has_any_objects(sim: &SimContext, team: TeamId) -> bool {
    let Some(members) = sim.world.team_members.get(&team) else {
        return false;
    };
    members.iter().any(|&slot| {
        let Some(obj) = sim.world.objects.get(slot) else {
            return false;
        };
        if obj.is_effectively_dead() != FALSE || obj.is_destroyed() {
            return false;
        }
        sim.object_is_kind_of(slot, KindOfType::Projectile) == FALSE
            && sim.object_is_kind_of(slot, KindOfType::Mine) == FALSE
            && sim.object_is_kind_of(slot, KindOfType::Inert) == FALSE
    })
}

fn team_has_any_units(sim: &SimContext, team: TeamId) -> bool {
    let Some(members) = sim.world.team_members.get(&team) else {
        return false;
    };
    members.iter().any(|&slot| {
        let Some(obj) = sim.world.objects.get(slot) else {
            return false;
        };
        if obj.is_effectively_dead() != FALSE || obj.is_destroyed() {
            return false;
        }
        sim.object_is_kind_of(slot, KindOfType::Projectile) == FALSE
            && sim.object_is_kind_of(slot, KindOfType::Mine) == FALSE
            && sim.object_is_kind_of(slot, KindOfType::Structure) == FALSE
    })
}

fn collect_player_harvesters(cx: &BuildContext<'_>, player: PlayerIndex) -> Vec<ObjectSlot> {
    let Some(teams) = cx.teams.as_deref() else {
        return Vec::new();
    };
    let Some(protos) = cx.sim.world.players.player(player).map(|p| &p.team_prototypes) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for &proto_id in protos {
        for team_id in teams.instance_ids(proto_id) {
            if let Some(members) = cx.sim.world.team_members.get(&team_id) {
                for &slot in members {
                    if cx.sim.object_is_kind_of(slot, KindOfType::Harvester) != FALSE
                        && cx
                            .sim
                            .world
                            .objects
                            .get(slot)
                            .is_some_and(|o| o.ai_slot.is_some())
                    {
                        out.push(slot);
                    }
                }
            }
        }
    }
    out
}

fn has_supply_truck_ai(sim: &mut SimContext, slot: ObjectSlot) -> bool {
    let is_worker = sim
        .world
        .objects
        .get(slot)
        .and_then(|o| o.ai_slot)
        .and_then(|ms| sim.world.modules.payload(ms))
        .and_then(|p| p.as_any())
        .and_then(|a| a.downcast_ref::<AiUpdate>())
        .is_some_and(|ai| matches!(ai.derived, DerivedAi::Worker(_)));
    if is_worker {
        return true;
    }
    sim.with_supply_truck(slot, &mut |_, _, _| {}) != FALSE
}

/// Returns `(preferred_dock, is_currently_ferrying, is_forced_wanting)`.
fn supply_truck_state(sim: &mut SimContext, slot: ObjectSlot) -> Option<(ObjectId, bool, bool)> {
    let ai_ms = sim.world.objects.get(slot)?.ai_slot?;
    let worker_state = sim
        .world
        .modules
        .payload(ai_ms)?
        .as_any()
        .and_then(|a| a.downcast_ref::<AiUpdate>())
        .and_then(|ai| match &ai.derived {
            DerivedAi::Worker(w) => {
                let ferrying =
                    w.state == AS_SUPPLY_TRUCK || ai.machine.current_state_id() == state_id::DOCK;
                Some((w.preferred_dock, ferrying, w.force_pending != FALSE))
            }
            _ => None,
        });
    if worker_state.is_some() {
        return worker_state;
    }
    if sim.with_supply_truck(slot, &mut |_, _, _| {}) != FALSE {
        return Some((ObjectId::INVALID, false, false));
    }
    None
}

fn supply_center_has_supplies(
    sim: &SimContext,
    me: PlayerIndex,
    supply_center: ObjectSlot,
) -> bool {
    let Some(obj) = sim.world.objects.get(supply_center) else {
        return false;
    };
    let center = *obj.thing.get_position();
    let radius = SUPPLY_CENTER_CLOSE_DIST + obj.geometry.get_bounding_circle_radius();
    let mut filter = |s: ObjectSlot| {
        sim.object_is_kind_of(s, KindOfType::SupplySource) != FALSE
            && owner_of(sim, s) != Some(me)
            && sim
                .world
                .objects
                .get(s)
                .is_some_and(|o| o.private_status & PRIVATE_STATUS_OFF_MAP == 0)
    };
    let Some(source_slot) = sim.closest_object_to_position(
        &center,
        radius,
        DistanceCalculationType::FromBoundingSphere2d,
        &mut filter,
    ) else {
        return false;
    };
    let Some(source_obj) = sim.world.objects.get(source_slot) else {
        return false;
    };
    let warehouse_boxes = source_obj.behaviors.iter().find_map(|&(ms, _)| {
        sim.world
            .modules
            .payload(ms)?
            .as_any()?
            .downcast_ref::<SupplyWarehouseDockUpdate>()
            .map(|d| d.boxes_stored)
    });
    if let Some(boxes) = warehouse_boxes {
        let cash = boxes.wrapping_mul(sim.rules.global_data.base_value_per_supply_box);
        if cash <= 0 {
            return false;
        }
        let owner = sim.controlling_player(source_obj.team_id);
        if owner.and_then(|o| sim.world.player_relationships.get(&(me, o)))
            == Some(&Relationship::Enemies)
        {
            return false;
        }
    }
    true
}
