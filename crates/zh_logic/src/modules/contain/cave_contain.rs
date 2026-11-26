//! `PKT-264`: `CaveContain` (`CaveContain.h`, `CaveContain.cpp`).
//!
//! Packet: `packets/phase5/PKT-264_CaveContain.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt264_cave.rs`.
//!
//! Every cave with the same index shares the tracker `CaveSystem` keeps for it; the module's
//! own `OpenContain` list stays empty. Passengers entering or leaving move all those caves
//! between teams. `TeamId::INVALID` stands for a null `Team*`.
//!
//! A cave has no `onDelete`: one that is destroyed without dying stays on its tracker's list.
//!
//! Where the reference dereferences the null tracker of an index no cave has registered,
//! the port does nothing.

use crate::arena::{ModuleContext, ObjectSlot};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::xfer_module_version;
use crate::modules::contain::open_contain::{
    impl_contain_module, on_removed_from, order_all_passengers_to_exit, ContainBehavior,
    OpenContainCore, OpenContainModuleData,
};
use crate::modules::contain::tunnel_contain::{
    tracker_contain_count, tracker_contained, tracker_is_valid_container_for,
    tracker_on_tunnel_destroyed, TrackerKey,
};
use crate::modules::module::{
    BehaviorModule, CaveFacet, CommandSourceType, CreateFacet, ModuleData,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Bool, Int, NameKeyType, ObjectId, TeamId, UnsignedInt, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError, XferMode};

#[derive(Debug, Clone, PartialEq)]
pub struct CaveContainModuleData {
    pub open: OpenContainModuleData,
    pub cave_index: Int,
}

impl CaveContainModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            open: OpenContainModuleData::new(tag_key, module_name_key),
            cave_index: 0,
        }
    }
}

impl ModuleData for CaveContainModuleData {
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
        use crate::arena::FacetMask;
        FacetMask(0x05f | FacetMask::CAVE_INTERFACE.0 | FacetMask::EXIT_INTERFACE.0)
    }
}

pub struct CaveContain {
    pub data: CaveContainModuleData,
    pub core: OpenContainCore,
    pub need_to_run_on_build_complete: Bool,
    pub cave_index: Int,
    /// The team to go back to when the network empties.
    pub original_team: TeamId,
}

impl CaveContain {
    pub fn new(data: CaveContainModuleData) -> Self {
        let core = OpenContainCore::new(data.open.passengers_allowed_to_fire);
        Self {
            data,
            core,
            need_to_run_on_build_complete: TRUE,
            cave_index: 0,
            original_team: TeamId::INVALID,
        }
    }

    fn tracker(&self) -> TrackerKey {
        TrackerKey::Cave(self.cave_index)
    }

    /// `registerNewCave` and `onTunnelCreated`.
    fn join_network(&self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        sim.cave_register(self.cave_index);
        let Some(id) = sim.object(ctx.owner_slot).map(|o| o.id) else {
            return;
        };
        if let Some(tracker) = self.tracker().get_mut(sim) {
            tracker.on_tunnel_created(id);
        }
    }

    /// `unregisterCave` and `onTunnelDestroyed`.
    fn leave_network(&self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        sim.cave_unregister(self.cave_index);
        tracker_on_tunnel_destroyed(sim, self.tracker(), ctx.owner_slot);
    }

    /// `changeTeamOnAllConnectedCaves`.
    fn change_team_on_all_connected_caves(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        new_team: TeamId,
        set_original_teams: Bool,
    ) {
        let caves = self
            .tracker()
            .get(sim)
            .map_or_else(Vec::new, |t| t.tunnel_ids.clone());
        for id in caves {
            let Some(cave) = sim.find_object(id) else {
                continue;
            };
            let Some(team) = sim.object(cave).map(|o| o.team_id) else {
                continue;
            };
            let original = if set_original_teams != FALSE {
                team
            } else {
                TeamId::INVALID
            };
            if cave == ctx.owner_slot {
                self.original_team = original;
            } else if sim.cave_set_original_team(cave, original) == FALSE {
                continue;
            }
            sim.defect_object(cave, new_team, 0, TRUE);
        }
    }

    /// `recalcApparentControllingPlayer`. The indicator colour it sets afterwards is
    /// skipped by the reference too: `OpenContain` has no apparent controlling player.
    fn recalc(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let Some(own) = sim.object(ctx.owner_slot).map(|o| o.team_id) else {
            return;
        };
        if self.original_team == TeamId::INVALID {
            self.original_team = own;
        }
        if own == TeamId::INVALID {
            self.original_team = TeamId::INVALID;
        }
        let count = tracker_contain_count(sim, self.tracker());
        if count == 1 {
            let rider_team = tracker_contained(sim, self.tracker())
                .first()
                .and_then(|&id| sim.find_object(id))
                .and_then(|slot| sim.object(slot))
                .map(|o| o.team_id);
            if let Some(team) = rider_team.filter(|&t| t != TeamId::INVALID) {
                let default_team = sim.player_default_team(team);
                self.change_team_on_all_connected_caves(sim, ctx, default_team, TRUE);
            }
        } else if count == 0 {
            let original = self.original_team;
            self.change_team_on_all_connected_caves(sim, ctx, original, FALSE);
        }
    }
}

impl CreateFacet for CaveContain {
    fn on_create(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {
        self.cave_index = self.data.cave_index;
    }

    fn on_build_complete(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.need_to_run_on_build_complete == FALSE {
            return;
        }
        self.need_to_run_on_build_complete = FALSE;
        self.join_network(sim, ctx);
    }

    fn should_do_on_build_complete(&self) -> Bool {
        self.need_to_run_on_build_complete
    }
}

impl CaveFacet for CaveContain {
    /// Allowed only while both the old and the new network are empty.
    fn try_to_set_cave_index(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, new_index: Int) {
        if sim.cave_can_switch_index(self.cave_index, new_index) == FALSE {
            return;
        }
        if self.tracker().get(sim).is_none() {
            return;
        }
        self.leave_network(sim, ctx);
        self.cave_index = new_index;
        self.join_network(sim, ctx);
    }

    fn set_original_team(&mut self, old_team: TeamId) {
        self.original_team = old_team;
    }

    fn recalc_apparent_controlling_player(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.recalc(sim, ctx);
    }
}

impl ContainBehavior for CaveContain {
    fn open_data(&self) -> &OpenContainModuleData {
        &self.data.open
    }
    fn open_core(&self) -> &OpenContainCore {
        &self.core
    }
    fn open_core_mut(&mut self) -> &mut OpenContainCore {
        &mut self.core
    }

    /// A cave in, when it is the last of its index: every passenger goes with it.
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
        let under_construction = sim
            .object(ctx.owner_slot)
            .is_some_and(|o| o.status.test(ObjectStatusType::UnderConstruction.as_i32()) != FALSE);
        if under_construction {
            return;
        }
        self.leave_network(sim, ctx);
    }

    fn add_to_list(&mut self, sim: &mut dyn SimApi, _ctx: ModuleContext, rider: ObjectSlot) {
        let hero = sim.object_is_kind_of(rider, KindOfType::Hero);
        let Some(id) = sim.object(rider).map(|o| o.id) else {
            return;
        };
        if let Some(tracker) = self.tracker().get_mut(sim) {
            tracker.add_to_contain_list(id, hero);
        }
    }

    /// Works through any cave of the index, not only the one the rider entered.
    /// `expose_stealthed` is dropped by `TunnelTracker::removeFromContain`.
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
        let hero = sim.object_is_kind_of(rider, KindOfType::Hero);
        let Some(tracker) = self.tracker().get_mut(sim) else {
            return;
        };
        if tracker.remove_from_contain(id, hero) == FALSE {
            return;
        }
        self.removing(sim, ctx, rider);
        on_removed_from(sim, rider);
    }

    fn remove_all(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, expose_stealthed: Bool) {
        for id in tracker_contained(sim, self.tracker()).to_vec() {
            if let Some(rider) = sim.find_object(id) {
                self.remove_one(sim, ctx, rider, expose_stealthed);
            }
        }
    }

    fn contained(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, rider: ObjectSlot) {
        sim.set_disabled(rider, DisabledType::Held);
        self.recalc(sim, ctx);
    }

    fn removing(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, rider: ObjectSlot) {
        sim.clear_disabled(rider, DisabledType::Held);
        sim.set_in_partition(rider, TRUE);
        let Some((position, team)) = sim
            .object(ctx.owner_slot)
            .map(|o| (*o.thing.get_position(), o.team_id))
        else {
            return;
        };
        sim.set_object_position(rider, &position);
        self.core.last_unload_sound_frame = sim.frame();
        if tracker_contain_count(sim, self.tracker()) == 0 && team != TeamId::INVALID {
            let original = self.original_team;
            self.change_team_on_all_connected_caves(sim, ctx, original, FALSE);
            self.original_team = TeamId::INVALID;
        }
    }

    fn xfer_module(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.core.xfer(xfer, sim)?;
        xfer.xfer_bool(&mut self.need_to_run_on_build_complete)?;
        xfer.xfer_int(&mut self.cave_index)?;
        let mut team_id = self.original_team.0;
        xfer.xfer_int(&mut team_id)?;
        if xfer.mode() == XferMode::Load {
            self.original_team = TeamId(team_id);
        }
        Ok(())
    }

    // The getters below answer from the index's tracker.

    fn contain_count_view(&self, sim: &dyn SimView, _ctx: ModuleContext) -> UnsignedInt {
        tracker_contain_count(sim, self.tracker())
    }
    fn contain_max_view(&self, sim: &dyn SimView, _ctx: ModuleContext) -> Int {
        self.tracker()
            .get(sim)
            .map_or(0, |_| sim.max_tunnel_capacity())
    }
    fn contained_view<'a>(&'a self, sim: &'a dyn SimView, _ctx: ModuleContext) -> &'a [ObjectId] {
        tracker_contained(sim, self.tracker())
    }
    fn contains_view(&self, sim: &dyn SimView, ctx: ModuleContext, id: ObjectId) -> Bool {
        let inside = sim
            .find_object(id)
            .and_then(|slot| sim.object(slot))
            .is_some_and(|o| o.contained_by == Some(ctx.owner_slot));
        Bool::from(inside || tracker_contained(sim, self.tracker()).contains(&id))
    }
    fn is_valid_container(
        &self,
        sim: &dyn SimView,
        _ctx: ModuleContext,
        rider: ObjectSlot,
        check_capacity: Bool,
    ) -> Bool {
        if self.tracker().get(sim).is_none() {
            return FALSE;
        }
        tracker_is_valid_container_for(sim, self.tracker(), rider, check_capacity)
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
    fn kicks_out_on_capture(&self) -> Bool {
        FALSE
    }
}

impl_contain_module!(CaveContain, {
    fn as_create_mut(&mut self) -> Option<&mut dyn CreateFacet> {
        Some(self)
    }
    fn as_cave(&self) -> Option<&dyn CaveFacet> {
        Some(self)
    }
    fn as_cave_mut(&mut self) -> Option<&mut dyn CaveFacet> {
        Some(self)
    }
});

pub fn parse_cave_contain_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = CaveContainModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("CaveIndex") {
            r.parse_int(&mut data.cave_index)
        } else {
            data.open
                .parse_field(field, r, parse_ctx)
                .unwrap_or(Err(IniError::UnknownToken))
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_cave_contain(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<CaveContainModuleData>()
        .expect("CaveContain data type mismatch")
        .clone();
    Box::new(CaveContain::new(typed))
}
