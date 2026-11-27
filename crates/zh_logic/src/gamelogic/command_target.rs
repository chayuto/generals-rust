//! The world side of `PKT-044` (`GameLogic::logicMessageDispatcher`): the
//! [`CommandTarget`] over a [`SimContext`], with the `AIGroup` the dispatcher
//! builds for each in-game command and the handlers of the group orders. The
//! dispatcher itself is `zh_common`, which cannot see the world.
//!
//! Golden vectors: `docs/golden_vectors/PKT-423.json` runs commands through the
//! unmodified dispatcher with real selections
//! (`crates/zh_logic/tests/golden_pkt423_ai_group.rs`).
//!
//! The selections and hotkey squads are `Player` members; they are held in
//! [`CommandState`] until players are ported. What nothing carries out is listed
//! in [`CommandState::not_carried_out`].

use crate::ai::ai_group::{self, GroupId, PowerTarget, NO_MAX_SHOTS_LIMIT};
use crate::arena::ObjectSlot;
use crate::modules::module::CommandSourceType;
use crate::modules::update::ai_update::{AiCommandParms, AiCommandType};
use crate::object::weapon_set::{WeaponLockType, WeaponSlotType};
use crate::player::squad::{self, PlayerSquads};
use crate::rules::kind_of::KindOfType;
use crate::sim::{SimApi, SimContext, SimView};
use std::collections::BTreeMap;
use zh_common::io::recorder::{
    ReplayCommand, MSG_BEGIN_NETWORK_MESSAGES, MSG_END_NETWORK_MESSAGES, MSG_LOGIC_CRC,
};
use zh_common::math::Coord3D;
use zh_common::runtime::game_logic_dispatch::{self as dispatcher, CommandTarget};
use zh_common::types::{Int, NameKeyType, ObjectId};

pub const MSG_DO_WEAPON: Int = 1037;
pub const MSG_DO_WEAPON_AT_LOCATION: Int = 1038;
pub const MSG_DO_WEAPON_AT_OBJECT: Int = 1039;
pub const MSG_EXIT: Int = 1053;
pub const MSG_EVACUATE: Int = 1054;
pub const MSG_EXECUTE_RAILED_TRANSPORT: Int = 1055;
pub const MSG_COMBATDROP_AT_LOCATION: Int = 1056;
pub const MSG_COMBATDROP_AT_OBJECT: Int = 1057;
pub const MSG_DO_ATTACK_OBJECT: Int = 1059;
pub const MSG_DO_FORCE_ATTACK_OBJECT: Int = 1060;
pub const MSG_DO_FORCE_ATTACK_GROUND: Int = 1061;
pub const MSG_GET_REPAIRED: Int = 1062;
pub const MSG_GET_HEALED: Int = 1063;
pub const MSG_DO_REPAIR: Int = 1064;
pub const MSG_RESUME_CONSTRUCTION: Int = 1065;
pub const MSG_ENTER: Int = 1066;
pub const MSG_DOCK: Int = 1067;
pub const MSG_DO_MOVETO: Int = 1068;
pub const MSG_DO_ATTACKMOVETO: Int = 1069;
pub const MSG_DO_FORCEMOVETO: Int = 1070;
pub const MSG_ADD_WAYPOINT: Int = 1071;
pub const MSG_DO_GUARD_POSITION: Int = 1072;
pub const MSG_DO_GUARD_OBJECT: Int = 1073;
pub const MSG_DO_STOP: Int = 1074;
pub const MSG_DO_SCATTER: Int = 1075;
pub const MSG_INTERNET_HACK: Int = 1076;
pub const MSG_DO_CHEER: Int = 1077;
pub const MSG_TOGGLE_OVERCHARGE: Int = 1078;
pub const MSG_SWITCH_WEAPONS: Int = 1079;
pub const MSG_DO_SPECIAL_POWER_OVERRIDE_DESTINATION: Int = 1086;
pub const MSG_DO_SALVAGE: Int = 1087;
pub const MSG_CREATE_FORMATION: Int = 1094;
pub const MSG_SET_MINE_CLEARING_DETAIL: Int = 1096;

/// What the dispatcher keeps between commands.
#[derive(Debug, Clone)]
pub struct CommandState {
    /// `isInGame`.
    pub in_game: bool,
    /// `Player::m_currentSelection` and `m_squads`, by player index.
    pub squads: BTreeMap<Int, PlayerSquads>,
    /// The types of the commands nothing here carries out, in arrival order.
    pub not_carried_out: Vec<Int>,
}

impl Default for CommandState {
    fn default() -> Self {
        Self {
            in_game: true,
            squads: BTreeMap::new(),
            not_carried_out: Vec::new(),
        }
    }
}

struct SimCommands<'a> {
    sim: &'a mut SimContext,
    state: &'a mut CommandState,
    /// `currentlySelectedGroup`.
    group: Option<GroupId>,
}

/// `logicMessageDispatcher` over the world.
pub fn dispatch_command(sim: &mut SimContext, state: &mut CommandState, msg: &ReplayCommand) {
    let player = msg.player_index;
    let mut group = None;
    if sim.world.player_default_teams.contains_key(&player)
        && state.in_game
        && (MSG_BEGIN_NETWORK_MESSAGES..=MSG_END_NETWORK_MESSAGES).contains(&msg.msg_type)
        && msg.msg_type != MSG_LOGIC_CRC
        && msg.msg_type != dispatcher::MSG_SET_REPLAY_CAMERA
    {
        let new = ai_group::create_group(sim);
        let selection = &mut state.squads.entry(player).or_default().selection;
        squad::ai_group_from_squad(selection, sim, new);
        if ai_group::member_ids(sim, new).is_empty() {
            ai_group::destroy_group(sim, new);
        } else if !ai_group::remove_any_not_owned_by(sim, new, player) {
            group = Some(new);
        }
    }
    dispatcher::dispatch(&mut SimCommands { sim, state, group }, msg);
    // Retail destroys the group of the command once it has been carried out.
    if let Some(group) = group {
        ai_group::destroy_group(sim, group);
    }
}

fn object_id(raw: u32) -> ObjectId {
    ObjectId(raw as Int)
}

fn coord(raw: [f32; 3]) -> Coord3D {
    Coord3D::new(raw[0], raw[1], raw[2])
}

impl SimCommands<'_> {
    fn slot(&self, id: u32) -> Option<ObjectSlot> {
        self.sim.find_object(object_id(id))
    }

    /// The live object an argument names.
    fn target(&self, msg: &ReplayCommand, index: usize) -> Option<ObjectId> {
        let id = msg.arg(index).as_object_id();
        self.slot(id).map(|_| object_id(id))
    }

    fn owned_by(&self, id: ObjectId, player: Int) -> bool {
        self.sim
            .find_object(id)
            .and_then(|slot| self.sim.object(slot))
            .and_then(|o| self.sim.controlling_player(o.team_id))
            == Some(player)
    }

    fn skip(&mut self, msg: &ReplayCommand) {
        self.state.not_carried_out.push(msg.msg_type);
    }

    fn release_locks(&mut self, group: GroupId) {
        ai_group::release_weapon_lock(self.sim, group, WeaponLockType::LockedTemporarily);
    }

    fn lock(&mut self, group: GroupId, weapon: Int, lock: WeaponLockType) -> bool {
        WeaponSlotType::from_i32(weapon)
            .is_some_and(|w| ai_group::set_weapon_lock(self.sim, group, w, lock))
    }

    /// A group of the one object a command names, destroyed afterwards.
    fn with_group_of(&mut self, source: ObjectId, f: impl FnOnce(&mut SimContext, GroupId)) {
        let group = ai_group::create_group(self.sim);
        ai_group::add(self.sim, group, source);
        f(self.sim, group);
        ai_group::destroy_group(self.sim, group);
    }

    fn special_power(&mut self, source: u32, power: u32, target: PowerTarget, options: u32) {
        if self.slot(source).is_some() {
            self.with_group_of(object_id(source), |sim, group| {
                ai_group::group_do_special_power(sim, group, power, target, options);
            });
        } else if let Some(group) = self.group {
            ai_group::group_do_special_power(self.sim, group, power, target, options);
        }
    }

    /// The orders that act on the selection.
    fn group_order(&mut self, msg: &ReplayCommand, group: GroupId) {
        let source = CommandSourceType::FromPlayer;
        let int = |i: usize| msg.arg(i).as_int();
        let loc = |i: usize| coord(msg.arg(i).as_location());
        let on_object = match msg.msg_type {
            MSG_GET_REPAIRED => Some((AiCommandType::GetRepaired, 0, false)),
            MSG_GET_HEALED => Some((AiCommandType::GetHealed, 0, false)),
            MSG_DO_REPAIR => Some((AiCommandType::Repair, 0, false)),
            MSG_RESUME_CONSTRUCTION => Some((AiCommandType::ResumeConstruction, 0, false)),
            MSG_DOCK => Some((AiCommandType::Dock, 0, false)),
            MSG_ENTER => Some((AiCommandType::Enter, 1, true)),
            _ => None,
        };
        if let Some((cmd, index, release)) = on_object {
            if let Some(target) = self.target(msg, index) {
                if release {
                    self.release_locks(group);
                }
                ai_group::group_order_on_object(self.sim, group, cmd, target, source);
            }
            return;
        }
        match msg.msg_type {
            MSG_DO_MOVETO | MSG_DO_SALVAGE | MSG_DO_FORCEMOVETO | MSG_ADD_WAYPOINT => {
                self.release_locks(group);
                let waypoint = msg.msg_type == MSG_ADD_WAYPOINT;
                ai_group::group_move_to_position(self.sim, group, &loc(0), waypoint, source);
            }
            MSG_DO_ATTACKMOVETO => {
                self.release_locks(group);
                ai_group::group_attack_move_to_position(
                    self.sim,
                    group,
                    &loc(0),
                    NO_MAX_SHOTS_LIMIT,
                    source,
                );
            }
            MSG_DO_ATTACK_OBJECT | MSG_DO_FORCE_ATTACK_OBJECT => {
                if let Some(victim) = self.target(msg, 0) {
                    self.release_locks(group);
                    ai_group::group_attack_object(
                        self.sim,
                        group,
                        msg.msg_type == MSG_DO_FORCE_ATTACK_OBJECT,
                        victim,
                        NO_MAX_SHOTS_LIMIT,
                        source,
                    );
                }
            }
            MSG_DO_FORCE_ATTACK_GROUND => {
                let pos = loc(0);
                // A busy group keeps to its primary weapon for this order.
                let primary = !ai_group::is_idle(self.sim, group);
                if primary {
                    ai_group::set_weapon_lock(
                        self.sim,
                        group,
                        WeaponSlotType::Primary,
                        WeaponLockType::LockedTemporarily,
                    );
                } else {
                    self.release_locks(group);
                }
                ai_group::group_attack_position(
                    self.sim,
                    group,
                    Some(&pos),
                    NO_MAX_SHOTS_LIMIT,
                    source,
                );
                if primary {
                    self.release_locks(group);
                }
            }
            MSG_DO_WEAPON => {
                if self.lock(group, int(0), WeaponLockType::LockedTemporarily) {
                    ai_group::group_attack_position(self.sim, group, None, int(1), source);
                }
            }
            MSG_DO_WEAPON_AT_LOCATION => {
                if self.lock(group, int(0), WeaponLockType::LockedTemporarily) {
                    let pos = loc(1);
                    ai_group::group_attack_position(self.sim, group, Some(&pos), int(2), source);
                }
            }
            MSG_DO_WEAPON_AT_OBJECT => {
                if let Some(victim) = self.target(msg, 1) {
                    if self.lock(group, int(0), WeaponLockType::LockedTemporarily) {
                        ai_group::group_attack_object(
                            self.sim,
                            group,
                            false,
                            victim,
                            int(2),
                            source,
                        );
                    }
                }
            }
            MSG_SWITCH_WEAPONS => {
                self.lock(group, int(0), WeaponLockType::LockedPermanently);
            }
            MSG_DO_GUARD_POSITION => {
                ai_group::group_guard_position(self.sim, group, &loc(0), int(1), source);
            }
            MSG_DO_GUARD_OBJECT => {
                if let Some(target) = self.target(msg, 0) {
                    ai_group::group_guard_object(self.sim, group, target, int(1), source);
                }
            }
            MSG_DO_STOP => ai_group::group_idle(self.sim, group, source),
            MSG_DO_SCATTER => ai_group::group_scatter(self.sim, group, source),
            MSG_CREATE_FORMATION => ai_group::group_create_formation(self.sim, group),
            MSG_DO_CHEER => ai_group::group_cheer(self.sim, group),
            MSG_EVACUATE => {
                self.release_locks(group);
                ai_group::group_evacuate(self.sim, group, source);
            }
            MSG_EXECUTE_RAILED_TRANSPORT => {
                ai_group::group_execute_railed_transport(self.sim, group, source);
            }
            MSG_INTERNET_HACK => {
                self.release_locks(group);
                ai_group::group_hack_internet(self.sim, group, source);
            }
            MSG_TOGGLE_OVERCHARGE => ai_group::group_toggle_overcharge(self.sim, group),
            MSG_SET_MINE_CLEARING_DETAIL => {
                ai_group::set_mine_clearing_detail(self.sim, group, true);
            }
            MSG_COMBATDROP_AT_LOCATION => {
                ai_group::group_combat_drop(self.sim, group, None, &loc(0), source);
            }
            MSG_COMBATDROP_AT_OBJECT => {
                if let Some(target) = self.target(msg, 0) {
                    let pos = self
                        .sim
                        .find_object(target)
                        .and_then(|s| self.sim.object(s))
                        .map_or_else(Coord3D::default, |o| *o.thing.get_position());
                    ai_group::group_combat_drop(self.sim, group, Some(target), &pos, source);
                }
            }
            _ => self.skip(msg),
        }
    }
}

impl CommandTarget for SimCommands<'_> {
    fn is_player(&self, player: i32) -> bool {
        self.sim.world.player_default_teams.contains_key(&player)
    }

    fn is_in_game(&self) -> bool {
        self.state.in_game
    }

    fn object_exists(&self, id: u32) -> bool {
        self.slot(id).is_some()
    }

    fn is_selectable(&self, id: u32) -> bool {
        squad::is_selectable(self.sim, object_id(id))
    }

    fn is_mass_selectable(&self, id: u32) -> bool {
        self.is_selectable(id)
            && self
                .slot(id)
                .is_some_and(|s| self.sim.object_is_kind_of(s, KindOfType::Structure) == 0)
    }

    fn can_join_group(&self, id: u32) -> bool {
        ai_group::can_join(self.sim, object_id(id))
    }

    fn is_controlled_by(&self, id: u32, player: i32) -> bool {
        self.owned_by(object_id(id), player)
    }

    fn squads(&mut self, player: i32) -> &mut PlayerSquads {
        self.state.squads.entry(player).or_default()
    }

    fn transient_group(&mut self, members: &[u32]) {
        let group = ai_group::create_group(self.sim);
        for &member in members {
            ai_group::add(self.sim, group, object_id(member));
        }
        ai_group::destroy_group(self.sim, group);
    }

    fn set_rally_point(&mut self, msg: &ReplayCommand, _object: u32, _pos: [f32; 3]) {
        self.skip(msg);
    }

    fn do_special_power(&mut self, msg: &ReplayCommand, _group: &[u32], power: u32, options: u32) {
        let source = msg.arg(2).as_object_id();
        self.special_power(source, power, PowerTarget::None, options);
    }

    fn do_special_power_at_location(
        &mut self,
        msg: &ReplayCommand,
        _group: &[u32],
        power: u32,
        location: [f32; 3],
        angle: f32,
        object_in_way: Option<u32>,
        options: u32,
    ) {
        let last = usize::from(msg.argument_count()).saturating_sub(1);
        let source = msg.arg(last).as_object_id();
        let target = PowerTarget::Location {
            location: coord(location),
            angle,
            object_in_way: object_in_way.map(object_id),
        };
        self.special_power(source, power, target, options);
    }

    fn do_special_power_at_object(
        &mut self,
        msg: &ReplayCommand,
        _group: &[u32],
        power: u32,
        target: u32,
        options: u32,
    ) {
        let source = msg.arg(3).as_object_id();
        self.special_power(
            source,
            power,
            PowerTarget::Object(object_id(target)),
            options,
        );
    }

    fn purchase_science(&mut self, msg: &ReplayCommand, _player: i32, _science: i32) {
        self.skip(msg);
    }

    fn queue_upgrade(&mut self, _msg: &ReplayCommand, _group: &[u32], upgrade: u32) {
        if let Some(group) = self.group {
            ai_group::queue_upgrade(self.sim, group, NameKeyType(upgrade as Int));
        }
    }

    fn cancel_upgrade(&mut self, _msg: &ReplayCommand, producer: u32, upgrade: u32) {
        let key = NameKeyType(upgrade as Int);
        if self
            .sim
            .rules
            .upgrade_center
            .find_upgrade_by_key(key)
            .is_none()
        {
            return;
        }
        if let Some(slot) = self.slot(producer) {
            let _ = self.sim.with_production(slot, |production, sim, ctx| {
                production.cancel_upgrade(sim, ctx, key)
            });
        }
    }

    /// The template argument is a `ThingTemplate` id of the retail rule data,
    /// which has no counterpart here yet.
    fn queue_unit_create(
        &mut self,
        msg: &ReplayCommand,
        _producer: u32,
        _template: i32,
        _production_id: i32,
    ) {
        self.skip(msg);
    }

    fn cancel_unit_create(&mut self, _msg: &ReplayCommand, producer: u32, production_id: i32) {
        if let Some(slot) = self.slot(producer) {
            let _ = self.sim.with_production(slot, |production, sim, ctx| {
                production.cancel_unit_create(sim, ctx, production_id)
            });
        }
    }

    fn sell(&mut self, _msg: &ReplayCommand, _group: &[u32]) {
        if let Some(group) = self.group {
            ai_group::group_sell(self.sim, group);
        }
    }

    /// The checksum comparison belongs to the recorder.
    fn logic_crc(&mut self, _player: i32, _crc: u32) {}

    fn playback_logic_crc(&mut self, _crc: u32) {}

    fn unhandled(&mut self, msg: &ReplayCommand, _group: Option<&[u32]>) {
        let source = CommandSourceType::FromPlayer;
        match msg.msg_type {
            MSG_EXIT => {
                // The selected container puts out the object the command names.
                let rider = self.target(msg, 0);
                let container = self
                    .group
                    .and_then(|g| ai_group::member_ids(self.sim, g).first().copied());
                let (Some(rider), Some(container)) = (rider, container) else {
                    return;
                };
                if !self.owned_by(rider, msg.player_index) {
                    return;
                }
                let Some(slot) = self.sim.find_object(rider) else {
                    return;
                };
                if let Some(object) = self.sim.object_mut(slot) {
                    object
                        .weapon_set
                        .release_weapon_lock(WeaponLockType::LockedTemporarily);
                }
                let mut command = AiCommandParms::new(AiCommandType::Exit, source);
                command.obj = Some(container);
                ai_group::send_order(self.sim, slot, command);
            }
            MSG_DO_SPECIAL_POWER_OVERRIDE_DESTINATION => {
                let location = coord(msg.arg(0).as_location());
                let power_type = msg.arg(1).as_int();
                match self.target(msg, 2) {
                    Some(from) => {
                        if !cfg!(feature = "retail-crc") && !self.owned_by(from, msg.player_index) {
                            return;
                        }
                        self.with_group_of(from, |sim, group| {
                            ai_group::group_override_special_power_destination(
                                sim, group, power_type, &location,
                            );
                        });
                    }
                    None => {
                        if let Some(group) = self.group {
                            ai_group::group_override_special_power_destination(
                                self.sim, group, power_type, &location,
                            );
                        }
                    }
                }
            }
            _ => match self.group {
                Some(group) => self.group_order(msg, group),
                None => self.skip(msg),
            },
        }
    }
}
