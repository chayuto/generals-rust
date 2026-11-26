//! `PKT-354`: `ProductionUpdateModuleData`, `ProductionEntry` and `ProductionUpdate`
//! (`ProductionUpdate.h`, `ProductionUpdate.cpp`): the build queue of a factory.
//!
//! Packet: `packets/phase5/PKT-354_ProductionUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt354_production.rs`.
//!
//! Not modelled, none of which reaches saved state or the logic PRNG: the voice and
//! research sounds, the Eva message, the "upgrade complete" text and radar event of the
//! local player, and `ControlBar::markUIDirty`.

use crate::arena::{DeferredSelfOp, FacetMask, ModuleContext, ThingTemplateId};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{
    BehaviorModule, CanMakeType, DieFacet, ExitDoorType, ModuleData, ProductionFacet, UpdateFacet,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::disabled_types::{DisabledMaskType, DisabledType, DISABLED_TYPE_NAMES};
use crate::rules::kind_of::KindOfType;
use crate::rules::model_state::{
    ModelConditionFlagType, ModelConditionFlags, MODEL_CONDITION_NAMES,
};
use crate::rules::object_status_types::ObjectStatusType;
use crate::rules::upgrade::{UpgradeMaskType, UpgradeType};
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{
    Bool, Int, NameKeyType, Real, UnsignedInt, UnsignedShort, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError, XferMode};

/// `ProductionType`.
pub const PRODUCTION_INVALID: Int = 0;
pub const PRODUCTION_UNIT: Int = 1;
pub const PRODUCTION_UPGRADE: Int = 2;

/// `PRODUCTIONID_INVALID`.
pub const PRODUCTION_ID_INVALID: Int = 0;

const DOOR_COUNT_MAX: usize = ExitDoorType::DOOR_COUNT_MAX;
const NONE_AVAILABLE: Int = ExitDoorType::DoorNoneAvailable.as_i32();

/// `theOpeningFlags`, `theClosingFlags` and `theWaitingOpenFlags`: four bits per door.
fn opening_flag(door: usize) -> Int {
    ModelConditionFlagType::Door1Opening.as_i32() + 4 * door as Int
}

fn closing_flag(door: usize) -> Int {
    opening_flag(door) + 1
}

fn waiting_open_flag(door: usize) -> Int {
    opening_flag(door) + 2
}

/// `QuantityModifier`.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantityModifier {
    pub template_name: String,
    pub quantity: Int,
}

/// INI template data matching `ProductionUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct ProductionUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub num_door_animations: Int,
    pub door_opening_time: UnsignedInt,
    pub door_wait_open_time: UnsignedInt,
    pub door_closing_time: UnsignedInt,
    pub construction_complete_duration: UnsignedInt,
    pub quantity_modifiers: Vec<QuantityModifier>,
    pub max_queue_entries: Int,
    pub disabled_types_to_process: DisabledMaskType,
}

impl ProductionUpdateModuleData {
    pub const FACET_MASK: FacetMask = FacetMask(0x003 | FacetMask::PRODUCTION_UPDATE.0);

    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        let mut disabled_types_to_process = DisabledMaskType::new();
        disabled_types_to_process.set(DisabledType::Held.as_i32(), 1);
        Self {
            tag_key,
            module_name_key,
            num_door_animations: 0,
            door_opening_time: 0,
            door_wait_open_time: 0,
            door_closing_time: 0,
            construction_complete_duration: 0,
            quantity_modifiers: Vec::new(),
            max_queue_entries: 9,
            disabled_types_to_process,
        }
    }
}

impl ModuleData for ProductionUpdateModuleData {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        Self::FACET_MASK
    }
}

/// One queued unit or upgrade (`ProductionEntry`).
#[derive(Debug, Clone, PartialEq)]
pub struct ProductionEntry {
    pub production_type: Int,
    /// `m_objectToProduce` and `m_upgradeToResearch` share a union in the original.
    pub object: Option<ThingTemplateId>,
    pub upgrade: Option<NameKeyType>,
    pub production_id: Int,
    pub percent_complete: Real,
    pub frames_under_construction: Int,
    pub quantity_total: Int,
    pub quantity_produced: Int,
    /// `ExitDoorType`. The constructor leaves it to the zeroed pool block, so an upgrade
    /// entry saves door 0.
    pub exit_door: Int,
}

impl ProductionEntry {
    fn new() -> Self {
        Self {
            production_type: PRODUCTION_INVALID,
            object: None,
            upgrade: None,
            production_id: 1,
            percent_complete: 0.0,
            frames_under_construction: 0,
            quantity_total: 0,
            quantity_produced: 0,
            exit_door: 0,
        }
    }

    pub fn quantity_remaining(&self) -> Int {
        self.quantity_total.wrapping_sub(self.quantity_produced)
    }
}

/// `ProductionUpdate::DoorInfo`, saved as raw memory (16 bytes with its padding).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DoorInfo {
    pub opened_frame: UnsignedInt,
    pub wait_open_frame: UnsignedInt,
    pub closed_frame: UnsignedInt,
    pub hold_open: Bool,
}

impl DoorInfo {
    fn is_idle(&self) -> bool {
        self.opened_frame == 0 && self.wait_open_frame == 0 && self.closed_frame == 0
    }
}

/// Runtime instance of `ProductionUpdate`.
pub struct ProductionUpdate {
    pub data: ProductionUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    /// `m_specialPowerConstructionCommandButton`, reduced to its thing template. Not saved.
    pub special_power_construction_template: Option<ThingTemplateId>,
    /// The linked list `m_productionQueue`, head first.
    pub queue: Vec<ProductionEntry>,
    pub unique_id: Int,
    /// `m_productionCount`, saved apart from the list.
    pub production_count: UnsignedInt,
    pub construction_complete_frame: UnsignedInt,
    pub doors: [DoorInfo; DOOR_COUNT_MAX],
    pub clear_flags: ModelConditionFlags,
    pub set_flags: ModelConditionFlags,
    pub flags_dirty: Bool,
}

impl ProductionUpdate {
    pub fn new(data: ProductionUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            special_power_construction_template: None,
            queue: Vec::new(),
            unique_id: 1,
            production_count: 0,
            construction_complete_frame: 0,
            doors: [DoorInfo::default(); DOOR_COUNT_MAX],
            clear_flags: ModelConditionFlags::new(),
            set_flags: ModelConditionFlags::new(),
            flags_dirty: FALSE,
        }
    }

    fn queue_is_full(&self) -> bool {
        // `UnsignedInt >= Int`: a negative limit compares as a huge one.
        self.production_count >= self.data.max_queue_entries as UnsignedInt
    }

    fn set_flag(&mut self, flag: Int) {
        self.set_flags.set(flag, 1);
        self.flags_dirty = TRUE;
    }

    /// The `clear.set(f, true); set.set(f, false);` pair.
    fn drop_flag(&mut self, flag: Int) {
        self.clear_flags.set(flag, 1);
        self.set_flags.set(flag, 0);
        self.flags_dirty = TRUE;
    }

    /// `ACTIVELY_CONSTRUCTING` on the drawable, if there is one.
    fn actively_constructing(sim: &dyn SimApi, ctx: ModuleContext) -> Option<Bool> {
        if sim.object_has_drawable(ctx.owner_slot) == FALSE {
            return None;
        }
        sim.object(ctx.owner_slot).map(|o| {
            o.model_condition
                .test(ModelConditionFlagType::ActivelyConstructing.as_i32())
        })
    }

    /// `addToProductionQueue`.
    fn add_to_queue(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, entry: ProductionEntry) {
        self.queue.push(entry);
        self.production_count = self.production_count.wrapping_add(1);
        if Self::actively_constructing(sim, ctx) == Some(FALSE) {
            self.set_flag(ModelConditionFlagType::ActivelyConstructing.as_i32());
        }
    }

    /// `removeFromProductionQueue`.
    fn remove_from_queue(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, index: usize) {
        let (kind, door) = (
            self.queue[index].production_type,
            self.queue[index].exit_door,
        );
        if kind == PRODUCTION_UNIT && door != NONE_AVAILABLE {
            if let Some(door) = ExitDoorType::from_i32(door) {
                sim.unreserve_exit_door(ctx.owner_slot, door);
            }
        }
        self.queue.remove(index);
        self.production_count = self.production_count.wrapping_sub(1);
        if let Some(constructing) = Self::actively_constructing(sim, ctx) {
            if self.production_count == 0 && constructing != FALSE {
                self.drop_flag(ModelConditionFlagType::ActivelyConstructing.as_i32());
            }
        }
    }

    /// Type, mask, cost and build time of a queued upgrade.
    fn upgrade_info(
        sim: &dyn SimView,
        key: NameKeyType,
    ) -> Option<(UpgradeType, UpgradeMaskType, Int, Int)> {
        sim.rules()
            .upgrade_center
            .find_upgrade_by_key(key)
            .map(|u| {
                (
                    u.upgrade_type(),
                    u.upgrade_mask().clone(),
                    u.calc_cost_to_build(None),
                    u.calc_time_to_build(None),
                )
            })
    }

    /// `cancelUnitCreate(ProductionEntry *)`.
    fn cancel_unit_at(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, index: usize) -> Bool {
        let entry = &self.queue[index];
        #[cfg(not(feature = "retail-crc"))]
        if entry.quantity_remaining() < entry.quantity_total {
            return FALSE;
        }
        // On an upgrade entry the original reads the union as a thing template.
        let Some(tmpl) = entry.object else {
            return FALSE;
        };
        let Some(team) = sim.object(ctx.owner_slot).map(|o| o.team_id) else {
            return FALSE;
        };
        let cost = sim.unit_cost_to_build(team, tmpl);
        sim.deposit_player_money(team, cost as UnsignedInt);
        self.remove_from_queue(sim, ctx, index);
        TRUE
    }

    /// `cancelUpgrade(ProductionEntry *)`.
    fn cancel_upgrade_at(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        index: usize,
    ) -> Bool {
        let Some(key) = self.queue[index].upgrade else {
            return FALSE;
        };
        let Some(team) = sim.object(ctx.owner_slot).map(|o| o.team_id) else {
            return FALSE;
        };
        let Some((upgrade_type, mask, cost, _)) = Self::upgrade_info(sim, key) else {
            return FALSE;
        };
        if upgrade_type == UpgradeType::Player
            && sim.player_has_upgrade_in_production(team, &mask) == FALSE
        {
            return FALSE;
        }
        sim.deposit_player_money(team, cost as UnsignedInt);
        self.remove_from_queue(sim, ctx, index);
        if upgrade_type == UpgradeType::Player {
            sim.player_remove_upgrade(team, key);
        }
        TRUE
    }

    /// `updateDoors`.
    fn update_doors(&mut self, now: UnsignedInt) {
        for i in 0..DOOR_COUNT_MAX {
            let door = self.doors[i];
            if door.opened_frame != 0 {
                if now.wrapping_sub(door.opened_frame) > self.data.door_opening_time {
                    self.doors[i].opened_frame = 0;
                    self.doors[i].wait_open_frame = now;
                    self.drop_flag(opening_flag(i));
                    self.set_flag(waiting_open_flag(i));
                }
            } else if door.wait_open_frame != 0 {
                if now.wrapping_sub(door.wait_open_frame) > self.data.door_wait_open_time
                    && door.hold_open == FALSE
                {
                    self.doors[i].wait_open_frame = 0;
                    self.doors[i].closed_frame = now;
                    self.drop_flag(waiting_open_flag(i));
                    self.set_flag(closing_flag(i));
                }
            } else if door.closed_frame != 0
                && door.hold_open == FALSE
                && now.wrapping_sub(door.closed_frame) > self.data.door_closing_time
            {
                self.doors[i].closed_frame = 0;
                self.drop_flag(closing_flag(i));
            }
        }
    }

    /// The part of `update` that runs when a unit entry reaches 100 percent.
    fn complete_unit(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let owner = ctx.owner_slot;
        let now = sim.frame();
        let Some(tmpl) = self.queue[0].object else {
            return;
        };
        if sim.object_has_exit_interface(owner) == FALSE {
            self.remove_from_queue(sim, ctx, 0);
            return;
        }
        let number_to_try = self.queue[0].quantity_remaining();
        for _ in 0..number_to_try {
            let mut exit_door = self.queue[0].exit_door;
            if exit_door == NONE_AVAILABLE {
                exit_door = sim.reserve_exit_door(owner, tmpl).as_i32();
                self.queue[0].exit_door = exit_door;
            }
            if exit_door == NONE_AVAILABLE {
                continue;
            }
            let door = usize::try_from(exit_door)
                .ok()
                .filter(|&d| d < DOOR_COUNT_MAX);
            if let Some(d) = door.filter(|_| self.data.num_door_animations > 0) {
                if self.doors[d].is_idle() {
                    self.doors[d].opened_frame = now;
                    self.set_flag(opening_flag(d));
                } else if self.doors[d].wait_open_frame != 0 {
                    self.doors[d].wait_open_frame = now;
                } else if self.doors[d].closed_frame != 0 {
                    // `closed_frame` stays set.
                    self.doors[d].wait_open_frame = now;
                    self.drop_flag(opening_flag(d));
                    self.drop_flag(closing_flag(d));
                    self.set_flag(waiting_open_flag(d));
                }
            }
            if self.construction_complete_frame == 0 {
                self.construction_complete_frame = now;
                self.set_flag(ModelConditionFlagType::ConstructionComplete.as_i32());
            }
            if self.data.num_door_animations == 0
                || door.is_none_or(|d| self.doors[d].wait_open_frame != 0)
            {
                self.produce_one(sim, ctx, tmpl, exit_door);
            }
        }
        if self.queue[0].quantity_remaining() == 0 {
            self.remove_from_queue(sim, ctx, 0);
        }
    }

    /// Creates one unit of the head entry and sends it out through `exit_door`.
    fn produce_one(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        tmpl: ThingTemplateId,
        exit_door: Int,
    ) {
        let owner = ctx.owner_slot;
        let Some((team, factory_id)) = sim.object(owner).map(|o| (o.team_id, o.id)) else {
            return;
        };
        let default_team = sim.player_default_team(team);
        let new_obj = sim.factory_new_object(tmpl, default_team);
        if let Some(o) = sim.object_mut(new_obj) {
            o.producer_id = factory_id;
        }
        if let Some(door) = ExitDoorType::from_i32(exit_door) {
            sim.exit_object_via_door(owner, new_obj, door);
        }
        self.queue[0].exit_door = NONE_AVAILABLE;
        sim.production_unit_created(owner, new_obj);
        sim.dispatch_on_build_complete(new_obj);
        sim.record_unit_production(owner, new_obj);
        // `oneProductionSuccessful`.
        self.queue[0].quantity_produced = self.queue[0].quantity_produced.wrapping_add(1);
        self.queue[0].exit_door = NONE_AVAILABLE;
    }
}

impl ProductionFacet for ProductionUpdate {
    fn production_count(&self) -> UnsignedInt {
        self.production_count
    }

    fn count_unit_type_in_queue(&self, tmpl: ThingTemplateId) -> UnsignedInt {
        self.queue
            .iter()
            .filter(|e| e.production_type == PRODUCTION_UNIT && e.object == Some(tmpl))
            .fold(0, |n: UnsignedInt, _| n.wrapping_add(1))
    }

    fn is_upgrade_in_queue(&self, upgrade: NameKeyType) -> Bool {
        Bool::from(
            self.queue
                .iter()
                .any(|e| e.production_type == PRODUCTION_UPGRADE && e.upgrade == Some(upgrade)),
        )
    }

    fn special_power_construction_template(&self) -> Option<ThingTemplateId> {
        self.special_power_construction_template
    }

    fn set_special_power_construction_template(&mut self, tmpl: Option<ThingTemplateId>) {
        self.special_power_construction_template = tmpl;
    }

    fn can_queue_create_unit(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        tmpl: ThingTemplateId,
    ) -> CanMakeType {
        let (_, full) = sim.parking_door_requirement(ctx.owner_slot, tmpl);
        if full != FALSE {
            CanMakeType::ParkingPlacesFull
        } else {
            self.can_queue_upgrade()
        }
    }

    /// `canQueueUpgrade`: only the queue length matters.
    fn can_queue_upgrade(&self) -> CanMakeType {
        if self.queue_is_full() {
            CanMakeType::QueueFull
        } else {
            CanMakeType::Ok
        }
    }

    fn request_unique_unit_id(&mut self) -> Int {
        let id = self.unique_id;
        self.unique_id = id.wrapping_add(1);
        id
    }

    fn queue_upgrade(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        upgrade: NameKeyType,
    ) -> Bool {
        let owner = ctx.owner_slot;
        let Some((upgrade_type, mask, cost, _)) = Self::upgrade_info(sim, upgrade) else {
            return FALSE;
        };
        let Some(team) = sim.object(owner).map(|o| o.team_id) else {
            return FALSE;
        };
        // Only player upgrades are checked for money.
        if upgrade_type == UpgradeType::Player && sim.player_money(team) < cost as UnsignedInt {
            return FALSE;
        } else if upgrade_type == UpgradeType::Object
            && (sim.object_has_upgrade(owner, &mask) != FALSE
                || sim.affected_by_upgrade(owner, upgrade) == FALSE)
        {
            return FALSE;
        }
        if self.is_upgrade_in_queue(upgrade) != FALSE {
            return FALSE;
        }
        if sim.can_produce_upgrade(owner, upgrade) == FALSE {
            return FALSE;
        }
        if upgrade_type == UpgradeType::Player
            && (sim.player_has_upgrade(team, &mask) != FALSE
                || sim.player_has_upgrade_in_production(team, &mask) != FALSE)
        {
            return FALSE;
        }
        if self.queue_is_full() {
            return FALSE;
        }
        sim.withdraw_player_money(team, cost as UnsignedInt);
        let mut entry = ProductionEntry::new();
        entry.production_type = PRODUCTION_UPGRADE;
        entry.upgrade = Some(upgrade);
        entry.production_id = PRODUCTION_ID_INVALID;
        self.add_to_queue(sim, ctx, entry);
        // Object upgrades are marked on the player too, and nothing clears that mark.
        sim.player_add_upgrade_in_production(team, upgrade);
        TRUE
    }

    fn cancel_upgrade(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        upgrade: NameKeyType,
    ) -> Bool {
        let found = self
            .queue
            .iter()
            .position(|e| e.production_type == PRODUCTION_UPGRADE && e.upgrade == Some(upgrade));
        found.map_or(FALSE, |i| self.cancel_upgrade_at(sim, ctx, i))
    }

    fn queue_create_unit(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        tmpl: ThingTemplateId,
        production_id: Int,
    ) -> Bool {
        let owner = ctx.owner_slot;
        if sim.production_can_make_unit(owner, tmpl, Some(&*self)) != CanMakeType::Ok {
            return FALSE;
        }
        let mut exit_door = NONE_AVAILABLE;
        let (reserve, _) = sim.parking_door_requirement(owner, tmpl);
        if reserve != FALSE {
            if sim.object_has_exit_interface(owner) != FALSE {
                exit_door = sim.reserve_exit_door(owner, tmpl).as_i32();
            }
            if exit_door == NONE_AVAILABLE {
                return FALSE;
            }
        }
        if self.queue_is_full() {
            return FALSE;
        }
        let Some(team) = sim.object(owner).map(|o| o.team_id) else {
            return FALSE;
        };
        let cost = sim.unit_cost_to_build(team, tmpl);
        sim.withdraw_player_money(team, cost as UnsignedInt);
        let mut entry = ProductionEntry::new();
        entry.quantity_total = 1;
        let factory = &sim.rules().thing_factory;
        if let Some(unit) = factory.find_by_template_id_final(tmpl.0 as UnsignedShort) {
            if let Some(modifier) = self.data.quantity_modifiers.iter().find(|m| {
                factory
                    .find_template(&m.template_name)
                    .is_some_and(|t| t.is_equivalent_to(unit) != FALSE)
            }) {
                entry.quantity_total = modifier.quantity;
            }
        }
        entry.production_type = PRODUCTION_UNIT;
        entry.object = Some(tmpl);
        entry.production_id = production_id;
        entry.exit_door = exit_door;
        self.add_to_queue(sim, ctx, entry);
        TRUE
    }

    fn cancel_unit_create(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        production_id: Int,
    ) -> Bool {
        let found = self
            .queue
            .iter()
            .position(|e| e.production_id == production_id);
        found.map_or(FALSE, |i| self.cancel_unit_at(sim, ctx, i))
    }

    fn cancel_all_units_of_type(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        tmpl: ThingTemplateId,
    ) {
        let mut i = 0;
        while i < self.queue.len() {
            let entry = &self.queue[i];
            let matches = entry.production_type == PRODUCTION_UNIT && entry.object == Some(tmpl);
            if !(matches && self.cancel_unit_at(sim, ctx, i) != FALSE) {
                i += 1;
            }
        }
    }

    fn cancel_and_refund_all_production(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        for _ in 0..100 {
            let Some(head) = self.queue.first() else {
                break;
            };
            match head.production_type {
                PRODUCTION_UNIT => {
                    if self.cancel_unit_at(sim, ctx, 0) == FALSE {
                        self.remove_from_queue(sim, ctx, 0);
                    }
                }
                PRODUCTION_UPGRADE => {
                    if self.cancel_upgrade_at(sim, ctx, 0) == FALSE {
                        if cfg!(feature = "retail-crc") {
                            return;
                        }
                        self.remove_from_queue(sim, ctx, 0);
                    }
                }
                _ => return,
            }
        }
    }

    fn set_hold_door_open(&mut self, sim: &dyn SimView, door: ExitDoorType, hold: Bool) {
        let Ok(i) = usize::try_from(door.as_i32()) else {
            return;
        };
        if i >= DOOR_COUNT_MAX {
            return;
        }
        self.doors[i].hold_open = hold;
        if hold != FALSE && self.doors[i].is_idle() {
            self.doors[i].opened_frame = sim.frame();
            self.set_flag(opening_flag(i));
        }
    }
}

impl UpdateFacet for ProductionUpdate {
    fn disabled_types_to_process(&self) -> DisabledMaskType {
        self.data.disabled_types_to_process.clone()
    }

    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let owner = ctx.owner_slot;
        let now = sim.frame();
        if self.data.num_door_animations > 0 {
            self.update_doors(now);
        }
        if self.construction_complete_frame != 0
            && now.wrapping_sub(self.construction_complete_frame)
                > self.data.construction_complete_duration
        {
            self.construction_complete_frame = 0;
            self.drop_flag(ModelConditionFlagType::ConstructionComplete.as_i32());
        }
        if self.flags_dirty != FALSE {
            if sim.object_has_drawable(owner) != FALSE {
                sim.clear_and_set_model_condition_flags(owner, &self.clear_flags, &self.set_flags);
            }
            self.clear_flags.clear();
            self.set_flags.clear();
            self.flags_dirty = FALSE;
        }
        let Some(head) = self.queue.first() else {
            return UpdateSleepTime::NONE;
        };
        let (kind, object, upgrade) = (head.production_type, head.object, head.upgrade);
        let Some((team, sold)) = sim
            .object(owner)
            .map(|o| (o.team_id, o.status.test(ObjectStatusType::Sold.as_i32())))
        else {
            return UpdateSleepTime::NONE;
        };
        if sold != FALSE {
            return UpdateSleepTime::NONE;
        }
        if let Some(tmpl) = object.filter(|_| kind == PRODUCTION_UNIT) {
            let dozer = sim
                .rules()
                .thing_factory
                .find_by_template_id_final(tmpl.0 as UnsignedShort)
                .is_some_and(|t| t.is_kind_of(KindOfType::Dozer) != FALSE);
            if sim.player_allowed_to_build(team, tmpl) == FALSE
                && !dozer
                && self.cancel_unit_at(sim, ctx, 0) != FALSE
            {
                return UpdateSleepTime::NONE;
            }
        }
        self.queue[0].frames_under_construction =
            self.queue[0].frames_under_construction.wrapping_add(1);
        let total_frames = if kind == PRODUCTION_UNIT {
            object.map(|t| sim.unit_time_to_build(team, t))
        } else {
            upgrade.and_then(|k| Self::upgrade_info(sim, k).map(|u| u.3))
        };
        let Some(total_frames) = total_frames else {
            return UpdateSleepTime::NONE;
        };
        // A build time of zero frames gives infinity, which counts as complete.
        self.queue[0].percent_complete =
            self.queue[0].frames_under_construction as Real / total_frames as Real * 100.0;
        if self.queue[0].percent_complete >= 100.0 {
            if kind == PRODUCTION_UNIT {
                self.complete_unit(sim, ctx);
            } else if let Some(key) = upgrade.filter(|_| kind == PRODUCTION_UPGRADE) {
                sim.complete_production_upgrade(owner, key);
                self.remove_from_queue(sim, ctx, 0);
            }
        }
        UpdateSleepTime::NONE
    }
}

impl DieFacet for ProductionUpdate {
    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, _dmg: &DamageInfo) {
        self.cancel_and_refund_all_production(sim, ctx);
    }
}

impl BehaviorModule for ProductionUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, _sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
    }

    fn apply_deferred_op(&mut self, op: DeferredSelfOp, sim: &mut dyn SimApi, ctx: ModuleContext) {
        match op {
            DeferredSelfOp::Die { .. } => self.cancel_and_refund_all_production(sim, ctx),
            DeferredSelfOp::ProductionHoldDoor { door, hold } => {
                if let Some(door) = ExitDoorType::from_i32(door) {
                    self.set_hold_door_open(&*sim, door, hold);
                }
            }
            _ => {}
        }
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        let mut count = self.queue.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut count)?;
        let loading = xfer.mode() == XferMode::Load;
        if loading {
            if !self.queue.is_empty() {
                return Err(XferError::InvalidData);
            }
            self.queue = vec![ProductionEntry::new(); usize::from(count)];
        }
        let rules = sim.rules();
        for entry in &mut self.queue {
            xfer.xfer_int(&mut entry.production_type)?;
            let mut name = if loading {
                String::new()
            } else if entry.production_type == PRODUCTION_UNIT {
                entry
                    .object
                    .and_then(|t| {
                        rules
                            .thing_factory
                            .find_by_template_id_final(t.0 as UnsignedShort)
                    })
                    .map(|t| t.name_string.clone())
                    .ok_or(XferError::InvalidData)?
            } else {
                entry
                    .upgrade
                    .and_then(|k| rules.upgrade_center.find_upgrade_by_key(k))
                    .map(|u| u.upgrade_name().to_owned())
                    .ok_or(XferError::InvalidData)?
            };
            xfer.xfer_ascii_string(&mut name)?;
            if loading {
                if entry.production_type == PRODUCTION_UNIT {
                    let id = rules.thing_factory.find_template_id(&name);
                    entry.object = Some(id.ok_or(XferError::InvalidData)?);
                } else {
                    let key = rules
                        .name_keys
                        .lookup_key(&name)
                        .filter(|&k| rules.upgrade_center.find_upgrade_by_key(k).is_some());
                    entry.upgrade = Some(key.ok_or(XferError::InvalidData)?);
                }
            }
            xfer.xfer_int(&mut entry.production_id)?;
            xfer.xfer_real(&mut entry.percent_complete)?;
            xfer.xfer_int(&mut entry.frames_under_construction)?;
            xfer.xfer_int(&mut entry.quantity_total)?;
            xfer.xfer_int(&mut entry.quantity_produced)?;
            xfer.xfer_int(&mut entry.exit_door)?;
        }
        xfer.xfer_int(&mut self.unique_id)?;
        xfer.xfer_unsigned_int(&mut self.production_count)?;
        xfer.xfer_unsigned_int(&mut self.construction_complete_frame)?;
        for door in &mut self.doors {
            xfer.xfer_unsigned_int(&mut door.opened_frame)?;
            xfer.xfer_unsigned_int(&mut door.wait_open_frame)?;
            xfer.xfer_unsigned_int(&mut door.closed_frame)?;
            xfer.xfer_bool(&mut door.hold_open)?;
            let mut padding = [0u8; 3];
            xfer.xfer_user(&mut padding)?;
        }
        self.clear_flags.xfer_named(xfer, &MODEL_CONDITION_NAMES)?;
        self.set_flags.xfer_named(xfer, &MODEL_CONDITION_NAMES)?;
        xfer.xfer_bool(&mut self.flags_dirty)
    }

    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.load_post_process(sim, ctx);
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }

    fn as_update(&self) -> Option<&dyn UpdateFacet> {
        Some(self)
    }

    fn as_update_mut(&mut self) -> Option<&mut dyn UpdateFacet> {
        Some(self)
    }

    fn as_die(&self) -> Option<&dyn DieFacet> {
        Some(self)
    }

    fn as_die_mut(&mut self) -> Option<&mut dyn DieFacet> {
        Some(self)
    }

    fn as_production(&self) -> Option<&dyn ProductionFacet> {
        Some(self)
    }

    fn as_production_mut(&mut self) -> Option<&mut dyn ProductionFacet> {
        Some(self)
    }
}

pub fn parse_production_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = ProductionUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("MaxQueueEntries") {
            r.parse_int(&mut data.max_queue_entries)
        } else if field.eq_ignore_ascii_case("NumDoorAnimations") {
            r.parse_int(&mut data.num_door_animations)
        } else if field.eq_ignore_ascii_case("DoorOpeningTime") {
            r.parse_duration_unsigned_int(&mut data.door_opening_time)
        } else if field.eq_ignore_ascii_case("DoorWaitOpenTime") {
            r.parse_duration_unsigned_int(&mut data.door_wait_open_time)
        } else if field.eq_ignore_ascii_case("DoorCloseTime") {
            r.parse_duration_unsigned_int(&mut data.door_closing_time)
        } else if field.eq_ignore_ascii_case("ConstructionCompleteDuration") {
            r.parse_duration_unsigned_int(&mut data.construction_complete_duration)
        } else if field.eq_ignore_ascii_case("QuantityModifier") {
            let template_name = r.next_token(SEPS_DEFAULT)?.to_owned();
            let quantity = match r.next_token_or_null(SEPS_DEFAULT) {
                Some(token) => IniReader::scan_int(token)?,
                None => 1,
            };
            data.quantity_modifiers.push(QuantityModifier {
                template_name,
                quantity,
            });
            Ok(())
        } else if field.eq_ignore_ascii_case("DisabledTypesToProcess") {
            data.disabled_types_to_process
                .parse_from_ini(r, &DISABLED_TYPE_NAMES)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_production_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<ProductionUpdateModuleData>()
        .expect("ProductionUpdate data type mismatch")
        .clone();
    Box::new(ProductionUpdate::new(typed))
}
