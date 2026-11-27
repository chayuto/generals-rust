//! `PKT-044`: `GameLogic::logicMessageDispatcher` (`GameLogicDispatch.cpp`), in part.
//!
//! Golden vectors: `docs/golden_vectors/PKT-044.json`
//! (`tests/cpp_harness/gen_pkt044_dispatch.cpp`), `crates/zh_logic/tests/golden_pkt044_dispatch.rs`.
//!
//! Handled here: the current selection and the ten hotkey squads. Decoded here and
//! handed to [`CommandTarget`]: rally points, production, upgrades, selling, sciences,
//! special powers and the CRC messages. Every other message, and every message the
//! target cannot carry out, reaches [`CommandTarget::unhandled`].
//!
//! `AIGroup` membership belongs to the target (`zh_logic::gamelogic::command_target`),
//! which [`CommandTarget::transient_group`] tells of the groups made here.
//!
//! Not modelled: the statistics collector, and the replay camera.

use crate::io::recorder::{
    ReplayCommand, MSG_BEGIN_NETWORK_MESSAGES, MSG_END_NETWORK_MESSAGES, MSG_LOGIC_CRC,
};

pub const MSG_LOGIC_CRC_PLAYBACK: i32 = 29;
pub const MSG_CREATE_SELECTED_GROUP: i32 = 1001;
pub const MSG_CREATE_SELECTED_GROUP_NO_SOUND: i32 = 1002;
pub const MSG_DESTROY_SELECTED_GROUP: i32 = 1003;
pub const MSG_REMOVE_FROM_SELECTED_GROUP: i32 = 1004;
pub const MSG_CREATE_TEAM0: i32 = 1006;
pub const MSG_SELECT_TEAM0: i32 = 1016;
pub const MSG_ADD_TEAM0: i32 = 1026;
pub const MSG_DO_SPECIAL_POWER: i32 = 1040;
pub const MSG_DO_SPECIAL_POWER_AT_LOCATION: i32 = 1041;
pub const MSG_DO_SPECIAL_POWER_AT_OBJECT: i32 = 1042;
pub const MSG_SET_RALLY_POINT: i32 = 1043;
pub const MSG_PURCHASE_SCIENCE: i32 = 1044;
pub const MSG_QUEUE_UPGRADE: i32 = 1045;
pub const MSG_CANCEL_UPGRADE: i32 = 1046;
pub const MSG_QUEUE_UNIT_CREATE: i32 = 1047;
pub const MSG_CANCEL_UNIT_CREATE: i32 = 1048;
pub const MSG_SELL: i32 = 1052;
pub const MSG_SET_REPLAY_CAMERA: i32 = 1092;

pub const NUM_HOTKEY_SQUADS: usize = 10;
const SCIENCE_INVALID: i32 = -1;
const INVALID_ANGLE: f32 = -100.0;

/// The message types this dispatcher routes somewhere other than
/// [`CommandTarget::unhandled`] directly.
pub fn is_routed(msg_type: i32) -> bool {
    matches!(
        msg_type,
        MSG_LOGIC_CRC_PLAYBACK
            | MSG_LOGIC_CRC
            | MSG_CREATE_SELECTED_GROUP..=MSG_REMOVE_FROM_SELECTED_GROUP
            | MSG_DO_SPECIAL_POWER..=MSG_CANCEL_UNIT_CREATE
            | MSG_SELL
    ) || (MSG_CREATE_TEAM0..MSG_ADD_TEAM0 + NUM_HOTKEY_SQUADS as i32).contains(&msg_type)
}

/// `Squad`: object ids in the order they were added, duplicates included.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Squad {
    pub ids: Vec<u32>,
}

impl Squad {
    /// `getAllObjects`: forgets ids whose object no longer exists.
    fn prune(&mut self, target: &(impl CommandTarget + ?Sized)) {
        self.ids.retain(|&id| target.object_exists(id));
    }

    /// `removeObject`: the first occurrence only.
    fn remove_first(&mut self, id: u32) {
        if let Some(i) = self.ids.iter().position(|&x| x == id) {
            self.ids.remove(i);
        }
    }
}

/// A player's current selection and hotkey squads.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlayerSquads {
    pub selection: Squad,
    pub hotkeys: [Squad; NUM_HOTKEY_SQUADS],
}

/// The player's selection as an `AIGroup`: live, selectable members that may join.
fn selection_group<T: CommandTarget + ?Sized>(target: &mut T, player: i32) -> Vec<u32> {
    let mut selection = std::mem::take(&mut target.squads(player).selection);
    selection.prune(target);
    // An object is in one group only: adding it again removes the earlier entry,
    // so a repeated id keeps its last place.
    let mut group = Vec::new();
    for &id in &selection.ids {
        if target.is_selectable(id) && target.can_join_group(id) {
            group.retain(|&x| x != id);
            group.push(id);
        }
    }
    target.squads(player).selection = selection;
    group
}

/// `getLiveObjects` of a hotkey squad.
fn hotkey_live<T: CommandTarget + ?Sized>(target: &mut T, player: i32, n: usize) -> Vec<u32> {
    let mut squad = std::mem::take(&mut target.squads(player).hotkeys[n]);
    squad.prune(target);
    let live = squad
        .ids
        .iter()
        .copied()
        .filter(|&id| target.is_selectable(id))
        .collect();
    target.squads(player).hotkeys[n] = squad;
    live
}

/// What the dispatcher asks of the simulation. Queries answer for a live object id;
/// commands are required, so that an implementation has to decide about each.
pub trait CommandTarget {
    /// `ThePlayerList->getNthPlayer` is not null.
    fn is_player(&self, player: i32) -> bool;
    fn is_in_game(&self) -> bool;
    /// `findObjectByID` is not null.
    fn object_exists(&self, id: u32) -> bool;
    /// `Object::isSelectable`.
    fn is_selectable(&self, id: u32) -> bool;
    /// `Object::isMassSelectable`.
    fn is_mass_selectable(&self, id: u32) -> bool;
    /// `AIGroup::add` keeps it: it has an AI, or is a structure or always selectable.
    fn can_join_group(&self, id: u32) -> bool;
    /// `getControllingPlayer() == player`.
    fn is_controlled_by(&self, id: u32, player: i32) -> bool;
    fn squads(&mut self, player: i32) -> &mut PlayerSquads;

    /// `doSetRallyPoint`.
    fn set_rally_point(&mut self, msg: &ReplayCommand, object: u32, pos: [f32; 3]);
    /// `groupDoSpecialPower`.
    fn do_special_power(&mut self, msg: &ReplayCommand, group: &[u32], power: u32, options: u32);
    /// `groupDoSpecialPowerAtLocation`.
    #[allow(clippy::too_many_arguments)]
    fn do_special_power_at_location(
        &mut self,
        msg: &ReplayCommand,
        group: &[u32],
        power: u32,
        location: [f32; 3],
        angle: f32,
        object_in_way: Option<u32>,
        options: u32,
    );
    /// `groupDoSpecialPowerAtObject`.
    fn do_special_power_at_object(
        &mut self,
        msg: &ReplayCommand,
        group: &[u32],
        power: u32,
        target: u32,
        options: u32,
    );
    /// `Player::attemptToPurchaseScience`.
    fn purchase_science(&mut self, msg: &ReplayCommand, player: i32, science: i32);
    /// `AIGroup::queueUpgrade`; `upgrade` is the name key of the template.
    fn queue_upgrade(&mut self, msg: &ReplayCommand, group: &[u32], upgrade: u32);
    /// `ProductionUpdateInterface::cancelUpgrade`, if the template and interface exist.
    fn cancel_upgrade(&mut self, msg: &ReplayCommand, producer: u32, upgrade: u32);
    /// `queueCreateUnit`, if the template and interface exist.
    fn queue_unit_create(
        &mut self,
        msg: &ReplayCommand,
        producer: u32,
        template: i32,
        production_id: i32,
    );
    /// `cancelUnitCreate`.
    fn cancel_unit_create(&mut self, msg: &ReplayCommand, producer: u32, production_id: i32);
    /// `AIGroup::groupSell`.
    fn sell(&mut self, msg: &ReplayCommand, group: &[u32]);
    /// `MSG_LOGIC_CRC`: a checksum recorded or sent by `player`.
    fn logic_crc(&mut self, player: i32, crc: u32);
    /// `MSG_LOGIC_CRC_PLAYBACK`: this playback's own checksum.
    fn playback_logic_crc(&mut self, crc: u32);
    /// A message nothing here carries out.
    fn unhandled(&mut self, msg: &ReplayCommand, group: Option<&[u32]>);
    /// A throwaway `AIGroup` that `selectObject` or `deselectObject` fills with
    /// `members` and destroys: it uses up a group id, and the members end up in
    /// no group. A target without groups has nothing to do.
    fn transient_group(&mut self, _members: &[u32]) {}
}

/// `logicMessageDispatcher`.
pub fn dispatch<T: CommandTarget + ?Sized>(target: &mut T, msg: &ReplayCommand) {
    let player = msg.player_index;
    if !target.is_player(player) {
        target.unhandled(msg, None);
        return;
    }
    let mut group: Option<Vec<u32>> = None;
    if target.is_in_game()
        && (MSG_BEGIN_NETWORK_MESSAGES..=MSG_END_NETWORK_MESSAGES).contains(&msg.msg_type)
        && msg.msg_type != MSG_LOGIC_CRC
        && msg.msg_type != MSG_SET_REPLAY_CAMERA
    {
        let mut members = selection_group(target, player);
        // `removeAnyObjectsNotOwnedByPlayer` drops the group once it is emptied.
        members.retain(|&id| target.is_controlled_by(id, player));
        group = (!members.is_empty()).then_some(members);
    }
    let group = group.as_deref();
    let int = |i: usize| msg.arg(i).as_int();
    let id = |i: usize| msg.arg(i).as_object_id();
    // `getSingleObjectFromSelection`.
    let single = |target: &T| {
        group
            .and_then(|g| g.first().copied())
            .filter(|&o| target.object_exists(o))
    };
    let owner_allowed = |target: &T, object: u32| {
        cfg!(feature = "retail-crc") || target.is_controlled_by(object, player)
    };

    match msg.msg_type {
        MSG_LOGIC_CRC_PLAYBACK => target.playback_logic_crc(int(0) as u32),
        MSG_LOGIC_CRC => target.logic_crc(player, int(0) as u32),
        MSG_SET_RALLY_POINT => {
            let object = id(0);
            if target.object_exists(object) && owner_allowed(target, object) {
                target.set_rally_point(msg, object, msg.arg(1).as_location());
            }
        }
        MSG_DO_SPECIAL_POWER => {
            let (power, options, source) = (int(0) as u32, int(1) as u32, id(2));
            if let Some(members) = source_group(target, source, player, group) {
                target.do_special_power(msg, &members, power, options);
            }
        }
        MSG_DO_SPECIAL_POWER_AT_LOCATION => {
            let has_angle = msg.argument_count() >= 6;
            let mut next = 0;
            let mut take = || {
                next += 1;
                msg.arg(next - 1)
            };
            let power = take().as_int() as u32;
            let location = take().as_location();
            let angle = if has_angle {
                take().as_real()
            } else {
                INVALID_ANGLE
            };
            let in_way = take().as_object_id();
            let options = take().as_int() as u32;
            let source = take().as_object_id();
            let in_way = target.object_exists(in_way).then_some(in_way);
            if let Some(members) = source_group(target, source, player, group) {
                target.do_special_power_at_location(
                    msg, &members, power, location, angle, in_way, options,
                );
            }
        }
        MSG_DO_SPECIAL_POWER_AT_OBJECT => {
            let (power, victim) = (int(0) as u32, id(1));
            if !target.object_exists(victim) {
                return;
            }
            let (options, source) = (int(2) as u32, id(3));
            if let Some(members) = source_group(target, source, player, group) {
                target.do_special_power_at_object(msg, &members, power, victim, options);
            }
        }
        MSG_PURCHASE_SCIENCE => {
            if int(0) != SCIENCE_INVALID {
                target.purchase_science(msg, player, int(0));
            }
        }
        MSG_QUEUE_UPGRADE => {
            if let Some(members) = group {
                target.queue_upgrade(msg, members, int(1) as u32);
            }
        }
        MSG_CANCEL_UPGRADE => {
            if let Some(producer) = single(target) {
                if target.is_controlled_by(producer, player) {
                    target.cancel_upgrade(msg, producer, int(0) as u32);
                }
            }
        }
        MSG_QUEUE_UNIT_CREATE => {
            if let Some(producer) = single(target) {
                target.queue_unit_create(msg, producer, int(0), int(1));
            }
        }
        MSG_CANCEL_UNIT_CREATE => {
            if let Some(producer) = single(target) {
                if target.is_controlled_by(producer, player) {
                    target.cancel_unit_create(msg, producer, int(0));
                }
            }
        }
        MSG_SELL => {
            if let Some(members) = group {
                target.sell(msg, members);
            }
        }
        MSG_CREATE_SELECTED_GROUP | MSG_CREATE_SELECTED_GROUP_NO_SOUND => {
            let create_new = msg.arg(0).as_bool();
            let mut first = true;
            for i in 1..usize::from(msg.argument_count()) {
                let object = id(i);
                if !target.object_exists(object) {
                    continue;
                }
                select_object(target, object, create_new && first, player);
                first = false;
            }
        }
        MSG_REMOVE_FROM_SELECTED_GROUP => {
            for i in 0..usize::from(msg.argument_count()) {
                let object = id(i);
                if target.object_exists(object) {
                    deselect_object(target, object, player);
                }
            }
        }
        MSG_DESTROY_SELECTED_GROUP => target.squads(player).selection.ids.clear(),
        t if (MSG_CREATE_TEAM0..MSG_SELECT_TEAM0).contains(&t) => {
            create_hotkey(target, msg, (t - MSG_CREATE_TEAM0) as usize);
        }
        t if (MSG_SELECT_TEAM0..MSG_ADD_TEAM0).contains(&t) => {
            let n = (t - MSG_SELECT_TEAM0) as usize;
            target.squads(player).selection.ids.clear();
            let live = hotkey_live(target, player, n);
            target.squads(player).selection.ids.extend(live);
        }
        t if (MSG_ADD_TEAM0..MSG_ADD_TEAM0 + NUM_HOTKEY_SQUADS as i32).contains(&t) => {
            let live = hotkey_live(target, player, (t - MSG_ADD_TEAM0) as usize);
            target.squads(player).selection.ids.extend(live);
        }
        _ => target.unhandled(msg, group),
    }
}

/// Who a special power order addresses: the named source alone, or the selection.
/// `None` when the order is dropped.
fn source_group<T: CommandTarget + ?Sized>(
    target: &T,
    source: u32,
    player: i32,
    selection: Option<&[u32]>,
) -> Option<Vec<u32>> {
    if !target.object_exists(source) {
        return selection.map(<[u32]>::to_vec);
    }
    if !cfg!(feature = "retail-crc") && !target.is_controlled_by(source, player) {
        return None;
    }
    Some(if target.can_join_group(source) {
        vec![source]
    } else {
        Vec::new()
    })
}

/// `GameLogic::selectObject` for one player.
fn select_object<T: CommandTarget + ?Sized>(
    target: &mut T,
    object: u32,
    create_new: bool,
    player: i32,
) {
    if !target.is_mass_selectable(object) && !create_new {
        return;
    }
    target.transient_group(&[object]);
    let joins = target.can_join_group(object);
    let selection = &mut target.squads(player).selection.ids;
    if create_new {
        selection.clear();
    }
    if joins {
        selection.push(object);
    }
}

/// `GameLogic::deselectObject` for one player. The selection is rebuilt from the
/// group, so members that cannot be selected now are dropped too.
fn deselect_object<T: CommandTarget + ?Sized>(target: &mut T, object: u32, player: i32) {
    let mut members = selection_group(target, player);
    target.transient_group(&members);
    if let Some(i) = members.iter().position(|&x| x == object) {
        members.remove(i);
    }
    target.squads(player).selection.ids = members;
}

/// `processCreateTeamGameMessage`.
fn create_hotkey<T: CommandTarget + ?Sized>(target: &mut T, msg: &ReplayCommand, n: usize) {
    let player = msg.player_index;
    target.squads(player).hotkeys[n].ids.clear();
    for i in 0..usize::from(msg.argument_count()) {
        let object = msg.arg(i).as_object_id();
        if target.object_exists(object) {
            let squads = target.squads(player);
            for squad in &mut squads.hotkeys {
                squad.remove_first(object);
            }
            squads.hotkeys[n].ids.push(object);
        }
    }
}
