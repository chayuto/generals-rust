//! `PKT-300`: `TunnelContain` (`TunnelContain.h`, `TunnelContain.cpp`) and the world half of
//! `TunnelTracker` (`TunnelTracker.cpp`).
//!
//! Packet: `packets/phase5/PKT-300_TunnelContain.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt300_tunnel.rs`.
//!
//! The tracker operations here also serve `CaveContain`, selected by [`TrackerKey`].
//!
//! Every tunnel of a player shares that player's [`TunnelTrackerState`]; the module's own
//! `OpenContain` list stays empty. `healObjects` runs once per tunnel per update, as the
//! reference builds it (`PRESERVE_TUNNEL_HEAL_STACKING`), so `updateFullHealTime` is compiled
//! out and not ported. `TunnelContain::scatterToNearbyPosition` has no caller.
//!
//! Not modelled: `Object::setSafeOcclusionFrame` (client occlusion) and the academy count of
//! units that entered a tunnel network.

use crate::arena::{ModuleContext, ObjectSlot};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::xfer_module_version;
#[cfg(not(feature = "retail-crc"))]
use crate::modules::contain::open_contain::add_or_remove_obj_from_world;
use crate::modules::contain::open_contain::{
    impl_contain_module, kill_object, on_contained_by, on_removed_from,
    order_all_passengers_to_exit, update_open_contain, ContainBehavior, OpenContainCore,
    OpenContainModuleData,
};
use crate::modules::contain::transport_contain::infantry_kind_of_mask;
use crate::modules::module::{BehaviorModule, CommandSourceType, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView, TunnelTrackerState};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, LOGICFRAMES_PER_SECOND};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, Relationship, TeamId, UnsignedInt, UpdateSleepTime,
    FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

#[derive(Debug, Clone, PartialEq)]
pub struct TunnelContainModuleData {
    pub open: OpenContainModuleData,
    pub frames_for_full_heal: Real,
}

impl TunnelContainModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        let mut open = OpenContainModuleData::new(tag_key, module_name_key);
        open.allow_inside_kind_of = infantry_kind_of_mask();
        Self {
            open,
            frames_for_full_heal: 1.0,
        }
    }
}

impl ModuleData for TunnelContainModuleData {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn tag_key(&self) -> NameKeyType {
        self.open.tag_key()
    }
    fn module_name_key(&self) -> NameKeyType {
        self.open.module_name_key()
    }
    fn facet_mask(&self) -> crate::arena::FacetMask {
        self.open.facet_mask()
    }
}

// -----------------------------------------------------------------------------
// TunnelTracker operations that read or change the world
// -----------------------------------------------------------------------------

/// `Object::getControllingPlayer` of `slot`.
fn controlling_player(sim: &dyn SimView, slot: ObjectSlot) -> Option<Int> {
    sim.object(slot).map(|o| sim.player_index(o.team_id))
}

/// Which `TunnelTracker` an operation works on: a player's tunnel network, or the one
/// `CaveSystem` keeps for a cave index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackerKey {
    Player(Int),
    Cave(Int),
}

impl TrackerKey {
    pub fn get(self, sim: &dyn SimView) -> Option<&TunnelTrackerState> {
        match self {
            Self::Player(p) => sim.tunnel_tracker(p),
            Self::Cave(i) => sim.cave_tracker(i),
        }
    }

    /// A player's tracker is made on demand. A cave tracker exists only once a cave has
    /// registered its index.
    pub fn get_mut(self, sim: &mut dyn SimApi) -> Option<&mut TunnelTrackerState> {
        match self {
            Self::Player(p) => Some(sim.tunnel_tracker_mut(p)),
            Self::Cave(i) => sim.cave_tracker_mut(i),
        }
    }
}

/// `m_containList` of the tracker.
pub fn tracker_contained(sim: &dyn SimView, key: TrackerKey) -> &[ObjectId] {
    key.get(sim).map_or(&[], |t| t.contain_list.as_slice())
}

/// `TunnelTracker::getContainCount`.
pub(crate) fn tracker_contain_count(sim: &dyn SimView, key: TrackerKey) -> UnsignedInt {
    key.get(sim)
        .map_or(0, |t| t.contain_list_size as UnsignedInt)
}

/// `TunnelTracker::isValidContainerFor`: anything but aircraft, whatever the module's
/// `AllowInsideKindOf` says.
pub fn tracker_is_valid_container_for(
    sim: &dyn SimView,
    key: TrackerKey,
    obj: ObjectSlot,
    check_capacity: Bool,
) -> Bool {
    if sim.object(obj).is_none() || sim.object_is_kind_of(obj, KindOfType::Aircraft) != FALSE {
        return FALSE;
    }
    if check_capacity != FALSE {
        let count = tracker_contain_count(sim, key) as Int;
        return Bool::from(count < sim.max_tunnel_capacity());
    }
    TRUE
}

/// `Object::onContainedBy(nullptr)`: the front entrance is no longer an object. The rider
/// stays listed but is contained by nothing.
fn on_contained_by_nothing(sim: &mut dyn SimApi, rider: ObjectSlot) {
    sim.set_status(rider, ObjectStatusType::Unselectable);
    sim.clear_status(rider, ObjectStatusType::Masked);
    sim.set_contained_by(rider, None);
    let now = sim.frame();
    if let Some(o) = sim.object_mut(rider) {
        o.contained_by_frame = now;
    }
    sim.handle_partition_cell_maintenance(rider);
}

/// `TunnelTracker::onTunnelDestroyed`. The last entrance takes every passenger with it;
/// otherwise the passengers that came in through it move to the front entrance.
pub fn tracker_on_tunnel_destroyed(sim: &mut dyn SimApi, key: TrackerKey, dead: ObjectSlot) {
    let Some(dead_id) = sim.object(dead).map(|o| o.id) else {
        return;
    };
    let Some(tracker) = key.get_mut(sim) else {
        return;
    };
    let was_last = tracker.on_tunnel_destroyed(dead_id);
    let front = tracker.tunnel_ids.first().copied();
    // Neither `destroyObject` nor `onContainedBy` edits the list, so a copy walks it.
    let passengers = tracker.contain_list.clone();

    if was_last != FALSE {
        for id in passengers {
            if let Some(rider) = sim.find_object(id) {
                on_removed_from(sim, rider);
                sim.destroy_object(rider);
            }
        }
        if let Some(tracker) = key.get_mut(sim) {
            tracker.contain_list.clear();
            tracker.contain_list_size = 0;
        }
        return;
    }

    let front = front.and_then(|id| sim.find_object(id));
    for id in passengers {
        let Some(rider) = sim.find_object(id) else {
            continue;
        };
        if sim.object(rider).and_then(|o| o.contained_by) != Some(dead) {
            continue;
        }
        // A tunnel leaves the list when it is deleted; a cave that is deleted without
        // dying stays on it.
        match front {
            // Tunnels and caves both enclose. `front` can be the dying cave itself, if it
            // was listed twice, and its own module cannot be asked while it executes.
            Some(front) => on_contained_by(sim, rider, front, TRUE),
            None => on_contained_by_nothing(sim, rider),
        }
    }
}

/// `TunnelTracker::healObjects` and `healObject`.
pub fn tracker_heal_objects(sim: &mut dyn SimApi, key: TrackerKey, frames: Real) {
    let passengers = tracker_contained(sim, key).to_vec();
    for id in passengers {
        let Some(rider) = sim.find_object(id) else {
            continue;
        };
        let Some(entered) = sim.object(rider).map(|o| o.contained_by_frame) else {
            continue;
        };
        let max_health = sim.object_max_health(rider);
        let mut heal = DamageInfo::default();
        heal.input.damage_type = DamageType::Healing;
        heal.input.death_type = DeathType::NoneType;
        heal.input.amount = if sim.frame().wrapping_sub(entered) as Real >= frames {
            max_health
        } else {
            max_health / frames
        };
        sim.attempt_healing(rider, &mut heal);
    }
}

/// `TunnelTracker::getCurNemesis`. A stale nemesis is forgotten as a side effect.
pub fn tracker_cur_nemesis(sim: &mut dyn SimApi, key: TrackerKey) -> Option<ObjectSlot> {
    let (id, timestamp) = key
        .get(sim)
        .map(|t| (t.cur_nemesis_id, t.nemesis_timestamp))?;
    if id == ObjectId::INVALID {
        return None;
    }
    if timestamp.wrapping_add(4 * LOGICFRAMES_PER_SECOND as UnsignedInt) < sim.frame() {
        if let Some(tracker) = key.get_mut(sim) {
            tracker.cur_nemesis_id = ObjectId::INVALID;
        }
        return None;
    }
    let target = sim.find_object(id).filter(|&slot| {
        sim.object(slot).is_some_and(|o| {
            let status = |s: ObjectStatusType| o.status.test(s.as_i32()) != FALSE;
            let hidden = status(ObjectStatusType::Stealthed)
                && !status(ObjectStatusType::Detected)
                && !status(ObjectStatusType::Disguised);
            !hidden && o.is_effectively_dead() == FALSE
        })
    });
    if target.is_none() {
        if let Some(tracker) = key.get_mut(sim) {
            tracker.cur_nemesis_id = ObjectId::INVALID;
        }
    }
    target
}

/// `TunnelTracker::updateNemesis`.
pub fn tracker_update_nemesis(sim: &mut dyn SimApi, key: TrackerKey, target: Option<ObjectSlot>) {
    if tracker_cur_nemesis(sim, key).is_none() {
        let Some(target) = target else {
            return;
        };
        let wanted = [
            KindOfType::Vehicle,
            KindOfType::Structure,
            KindOfType::Infantry,
            KindOfType::Aircraft,
        ]
        .into_iter()
        .any(|k| sim.object_is_kind_of(target, k) != FALSE);
        let Some(id) = sim.object(target).map(|o| o.id) else {
            return;
        };
        if wanted {
            let now = sim.frame();
            if let Some(tracker) = key.get_mut(sim) {
                tracker.cur_nemesis_id = id;
                tracker.nemesis_timestamp = now;
            }
        }
    } else if tracker_cur_nemesis(sim, key) == target {
        let now = sim.frame();
        if let Some(tracker) = key.get_mut(sim) {
            tracker.nemesis_timestamp = now;
        }
    }
}

// -----------------------------------------------------------------------------
// TunnelContain
// -----------------------------------------------------------------------------

pub struct TunnelContain {
    pub data: TunnelContainModuleData,
    pub core: OpenContainCore,
    pub need_to_run_on_build_complete: Bool,
    pub is_currently_registered: Bool,
}

impl TunnelContain {
    pub fn new(data: TunnelContainModuleData) -> Self {
        let core = OpenContainCore::new(data.open.passengers_allowed_to_fire);
        Self {
            data,
            core,
            need_to_run_on_build_complete: TRUE,
            is_currently_registered: FALSE,
        }
    }

    /// `onDie` and `onDelete`: leave the network once.
    fn unregister(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.is_currently_registered == FALSE {
            return;
        }
        let Some(player) = controlling_player(sim, ctx.owner_slot) else {
            return;
        };
        tracker_on_tunnel_destroyed(sim, TrackerKey::Player(player), ctx.owner_slot);
        self.is_currently_registered = FALSE;
    }

    /// The list `swapContainedItemsList` hands to `removeAllContained` and `killAllContained`.
    /// The hero count is left alone, and the removals that follow no longer find their rider.
    fn swap_out_passengers(sim: &mut dyn SimApi, ctx: ModuleContext) -> Vec<ObjectId> {
        let Some(player) = controlling_player(sim, ctx.owner_slot) else {
            return Vec::new();
        };
        let tracker = sim.tunnel_tracker_mut(player);
        tracker.contain_list_size = 0;
        std::mem::take(&mut tracker.contain_list)
    }
}

impl ContainBehavior for TunnelContain {
    fn open_data(&self) -> &OpenContainModuleData {
        &self.data.open
    }
    fn open_core(&self) -> &OpenContainCore {
        &self.core
    }
    fn open_core_mut(&mut self) -> &mut OpenContainCore {
        &mut self.core
    }

    fn object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.core
            .on_object_created(sim, ctx, self.data.open.contain_max);
        if self.need_to_run_on_build_complete == FALSE {
            return;
        }
        self.need_to_run_on_build_complete = FALSE;
        let Some((player, id)) = sim
            .object(ctx.owner_slot)
            .map(|o| (sim.player_index(o.team_id), o.id))
        else {
            return;
        };
        sim.tunnel_tracker_mut(player).on_tunnel_created(id);
        self.is_currently_registered = TRUE;
    }

    /// `onDelete` replaces `OpenContain`'s: the passengers belong to the player.
    fn delete(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.unregister(sim, ctx);
    }

    fn die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        if self
            .data
            .open
            .die_mux
            .is_die_applicable(sim, ctx.owner_slot, dmg)
            == FALSE
        {
            return;
        }
        self.unregister(sim, ctx);
    }

    fn selling(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let Some(player) = controlling_player(sim, ctx.owner_slot) else {
            return;
        };
        let tunnels = sim
            .tunnel_tracker(player)
            .map_or(0, |t| t.tunnel_count_effective());
        if tunnels == 1 {
            self.remove_all(sim, ctx, FALSE);
        }
        self.unregister(sim, ctx);
    }

    /// Runs for every team change, even one that keeps the player.
    fn capture(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        old_team: TeamId,
        new_team: TeamId,
    ) {
        if self.is_currently_registered == FALSE {
            return;
        }
        let old_player = sim.player_index(old_team);
        tracker_on_tunnel_destroyed(sim, TrackerKey::Player(old_player), ctx.owner_slot);
        let new_player = sim.player_index(new_team);
        let Some(id) = sim.object(ctx.owner_slot).map(|o| o.id) else {
            return;
        };
        sim.tunnel_tracker_mut(new_player).on_tunnel_created(id);
    }

    fn add_to_list(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, rider: ObjectSlot) {
        let Some(player) = controlling_player(sim, ctx.owner_slot) else {
            return;
        };
        let hero = sim.object_is_kind_of(rider, KindOfType::Hero);
        let Some(id) = sim.object(rider).map(|o| o.id) else {
            return;
        };
        sim.tunnel_tracker_mut(player).add_to_contain_list(id, hero);
    }

    /// Does not ask whether `rider` is inside, or through which entrance it came.
    fn remove_one(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        rider: ObjectSlot,
        _expose_stealthed: Bool,
    ) {
        let Some(id) = sim.object(rider).map(|o| o.id) else {
            return;
        };
        self.removing(sim, ctx, rider);
        on_removed_from(sim, rider);
        let Some(player) = controlling_player(sim, ctx.owner_slot) else {
            return;
        };
        let hero = sim.object_is_kind_of(rider, KindOfType::Hero);
        sim.tunnel_tracker_mut(player).remove_from_contain(id, hero);
    }

    fn remove_all(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, expose_stealthed: Bool) {
        for id in Self::swap_out_passengers(sim, ctx) {
            if let Some(rider) = sim.find_object(id) {
                self.remove_one(sim, ctx, rider, expose_stealthed);
            }
        }
    }

    fn kill_all(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        for id in Self::swap_out_passengers(sim, ctx) {
            if let Some(rider) = sim.find_object(id) {
                self.remove_one(sim, ctx, rider, TRUE);
                kill_object(sim, rider, DamageType::Unresistable, DeathType::Normal);
            }
        }
    }

    fn harm_all(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        let Some(player) = controlling_player(sim, ctx.owner_slot) else {
            return;
        };
        // One `DamageInfo` serves every passenger.
        let mut dmg = dmg.clone();
        while let Some(&id) = tracker_contained(sim, TrackerKey::Player(player)).first() {
            let Some(rider) = sim.find_object(id) else {
                break;
            };
            self.remove_one(sim, ctx, rider, TRUE);
            sim.attempt_damage(rider, &mut dmg);
        }
    }

    /// `OpenContain::onContaining` only plays a sound.
    fn contained(&mut self, sim: &mut dyn SimApi, _ctx: ModuleContext, rider: ObjectSlot) {
        sim.set_disabled(rider, DisabledType::Held);
        sim.handle_partition_cell_maintenance(rider);
    }

    fn removing(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, rider: ObjectSlot) {
        sim.clear_disabled(rider, DisabledType::Held);
        let position = sim.object(ctx.owner_slot).map(|o| *o.thing.get_position());
        #[cfg(feature = "retail-crc")]
        {
            sim.set_in_partition(rider, TRUE);
            if let Some(position) = position {
                sim.set_object_position(rider, &position);
            }
        }
        #[cfg(not(feature = "retail-crc"))]
        {
            if let Some(position) = position {
                sim.set_object_position(rider, &position);
            }
            add_or_remove_obj_from_world(sim, rider, TRUE);
        }
        self.core.last_unload_sound_frame = sim.frame();
    }

    fn update_module(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        update_open_contain(self, sim, ctx);
        let Some(player) = controlling_player(sim, ctx.owner_slot) else {
            return UpdateSleepTime::NONE;
        };
        tracker_heal_objects(
            sim,
            TrackerKey::Player(player),
            self.data.frames_for_full_heal,
        );

        let Some((source, timestamp)) = sim.object_last_damage(ctx.owner_slot) else {
            return UpdateSleepTime::NONE;
        };
        if timestamp.wrapping_add(LOGICFRAMES_PER_SECOND as UnsignedInt) > sim.frame() {
            if let Some(attacker) = sim.find_object(source) {
                if sim.player_relationship(ctx.owner_slot, attacker) == Relationship::Enemies {
                    tracker_update_nemesis(sim, TrackerKey::Player(player), Some(attacker));
                }
            }
        }
        UpdateSleepTime::NONE
    }

    fn xfer_module(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.core.xfer(xfer, sim)?;
        xfer.xfer_bool(&mut self.need_to_run_on_build_complete)?;
        xfer.xfer_bool(&mut self.is_currently_registered)
    }

    // The getters below answer from the player's tracker.

    fn contain_count_view(&self, sim: &dyn SimView, ctx: ModuleContext) -> UnsignedInt {
        controlling_player(sim, ctx.owner_slot)
            .map_or(0, |p| tracker_contain_count(sim, TrackerKey::Player(p)))
    }
    fn contain_max_view(&self, sim: &dyn SimView, ctx: ModuleContext) -> Int {
        controlling_player(sim, ctx.owner_slot).map_or(0, |_| sim.max_tunnel_capacity())
    }
    fn contained_view<'a>(&'a self, sim: &'a dyn SimView, ctx: ModuleContext) -> &'a [ObjectId] {
        controlling_player(sim, ctx.owner_slot)
            .map_or(&[], |p| tracker_contained(sim, TrackerKey::Player(p)))
    }
    fn contains_view(&self, sim: &dyn SimView, ctx: ModuleContext, id: ObjectId) -> Bool {
        let inside = sim
            .find_object(id)
            .and_then(|slot| sim.object(slot))
            .is_some_and(|o| o.contained_by == Some(ctx.owner_slot));
        Bool::from(inside || self.contained_view(sim, ctx).contains(&id))
    }
    fn hero_count_view(&self, sim: &dyn SimView, ctx: ModuleContext) -> UnsignedInt {
        controlling_player(sim, ctx.owner_slot)
            .and_then(|p| sim.tunnel_tracker(p))
            .map_or(0, |t| t.hero_units_contained)
    }
    fn is_valid_container(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        rider: ObjectSlot,
        check_capacity: Bool,
    ) -> Bool {
        controlling_player(sim, ctx.owner_slot).map_or(FALSE, |p| {
            tracker_is_valid_container_for(sim, TrackerKey::Player(p), rider, check_capacity)
        })
    }
    fn order_exit(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        cmd_source: CommandSourceType,
    ) {
        order_all_passengers_to_exit(self, sim, ctx, cmd_source);
    }

    fn is_bustable_container(&self) -> Bool {
        TRUE
    }
    fn is_immune_to_clear(&self) -> Bool {
        TRUE
    }
    fn is_tunnel(&self) -> Bool {
        TRUE
    }
    fn kicks_out_on_capture(&self) -> Bool {
        FALSE
    }
}

impl_contain_module!(TunnelContain);

pub fn parse_tunnel_contain_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = TunnelContainModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("TimeForFullHeal") {
            r.parse_duration_real(&mut data.frames_for_full_heal)
        } else {
            data.open
                .parse_field(field, r, parse_ctx)
                .unwrap_or(Err(IniError::UnknownToken))
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_tunnel_contain(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<TunnelContainModuleData>()
        .expect("TunnelContain data type mismatch")
        .clone();
    Box::new(TunnelContain::new(typed))
}
