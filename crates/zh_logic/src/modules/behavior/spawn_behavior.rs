//! `PKT-296`: `SpawnBehaviorModuleData` and `SpawnBehavior`
//! (`SpawnBehavior.h`, `SpawnBehavior.cpp`).
//!
//! Packet: `packets/phase5/PKT-296_SpawnBehavior.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt296_365_spawn_slaved.rs`.
//!
//! The selection mirroring of `computeAggregateStates` (select the whole group
//! when one member is selected) is client behaviour: the selection it makes
//! reaches the simulation as player input, so it is not done here.

use crate::arena::{DeferredSelfOp, FacetMask, ModuleContext, ObjectSlot};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::{xfer_behavior_module_header, xfer_module_version};
use crate::modules::contain::open_contain::kill_object;
use crate::modules::die::die_module::DieMuxData;
use crate::modules::module::{
    BehaviorModule, CommandSourceType, DamageFacet, DieFacet, ModuleData, SpawnBehaviorFacet,
    UpdateFacet,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::object::weapon_set::CanAttackResult;
use crate::rules::combat_types::{parse_damage_type_flags, DamageType, DamageTypeFlags, DeathType};
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, UnsignedInt, UpdateSleepTime, VeterancyLevel, FALSE,
    TRUE,
};
use zh_common::xfer::{Xfer, XferError, XferMode};

/// `SPAWN_UPDATE_RATE`: the module looks for work twice a second.
const SPAWN_UPDATE_RATE: Int = 15;
/// `NONE_SPAWNED_YET` (`0xffffffff`) as the `Int` it is stored in.
const NONE_SPAWNED_YET: Int = -1;

/// INI template data matching `SpawnBehaviorModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct SpawnBehaviorModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub spawn_number: Int,
    pub spawn_replace_delay: Int,
    pub initial_burst: Int,
    pub is_one_shot: Bool,
    pub can_reclaim_orphans: Bool,
    pub aggregate_health: Bool,
    pub exit_by_budding: Bool,
    pub spawned_require_spawner: Bool,
    pub slaves_have_free_will: Bool,
    /// Parsed and never read by the module.
    pub damage_types_to_propagate_to_slaves: DamageTypeFlags,
    pub spawn_template_names: Vec<String>,
    pub die_mux: DieMuxData,
}

impl SpawnBehaviorModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            spawn_number: 0,
            spawn_replace_delay: 0,
            initial_burst: 0,
            is_one_shot: FALSE,
            can_reclaim_orphans: FALSE,
            aggregate_health: FALSE,
            exit_by_budding: FALSE,
            spawned_require_spawner: FALSE,
            slaves_have_free_will: FALSE,
            damage_types_to_propagate_to_slaves: DamageTypeFlags::new(),
            spawn_template_names: Vec::new(),
            die_mux: DieMuxData::new(),
        }
    }
}

impl ModuleData for SpawnBehaviorModuleData {
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
        FacetMask(0x007 | FacetMask::SPAWN_BEHAVIOR.0)
    }
}

/// Runtime instance of `SpawnBehavior`.
pub struct SpawnBehavior {
    pub data: SpawnBehaviorModuleData,
    /// The name of `m_spawnTemplate`; empty for the null pointer.
    pub spawn_template_name: String,
    pub one_shot_countdown: Int,
    pub frames_to_wait: Int,
    pub first_batch_count: Int,
    pub replacement_times: Vec<Int>,
    pub spawn_ids: Vec<ObjectId>,
    pub active: Bool,
    pub aggregate_health: Bool,
    pub initial_burst_times_inited: Bool,
    pub spawn_count: Int,
    pub self_tasking_spawn_count: UnsignedInt,
    pub initial_burst_countdown: UnsignedInt,
    /// `m_templateNameIterator`; not saved.
    pub template_name_index: usize,
}

/// A NaN the multiplication makes itself is `0x7fc0_0000` in the reference build.
fn reference_mul(a: Real, b: Real) -> Real {
    let p = a * b;
    if p.is_nan() && !a.is_nan() && !b.is_nan() {
        Real::from_bits(0x7fc0_0000)
    } else {
        p
    }
}

impl SpawnBehavior {
    pub fn new(data: SpawnBehaviorModuleData) -> Self {
        Self {
            spawn_template_name: String::new(),
            one_shot_countdown: if data.is_one_shot != FALSE {
                data.spawn_number
            } else {
                -1
            },
            frames_to_wait: 0,
            first_batch_count: 0,
            replacement_times: Vec::new(),
            spawn_ids: Vec::new(),
            active: TRUE,
            aggregate_health: data.aggregate_health,
            initial_burst_times_inited: FALSE,
            spawn_count: NONE_SPAWNED_YET,
            self_tasking_spawn_count: 0,
            initial_burst_countdown: data.initial_burst as UnsignedInt,
            template_name_index: 0,
            data,
        }
    }

    fn slaves<'a>(&'a self, sim: &'a dyn SimView) -> impl Iterator<Item = ObjectSlot> + 'a {
        self.spawn_ids.iter().filter_map(|&id| sim.find_object(id))
    }

    fn slave_slots(&self, sim: &dyn SimView) -> Vec<ObjectSlot> {
        self.slaves(sim).collect()
    }

    /// Calls `f` on every listed spawn that still exists, looking each one up in turn.
    fn for_each_slave(&self, sim: &mut dyn SimApi, mut f: impl FnMut(&mut dyn SimApi, ObjectSlot)) {
        for &id in &self.spawn_ids {
            if let Some(slave) = sim.find_object(id) {
                f(sim, slave);
            }
        }
    }

    fn is_effectively_dead(sim: &dyn SimView, obj: ObjectSlot) -> bool {
        sim.object(obj)
            .is_some_and(|o| o.is_effectively_dead() != FALSE)
    }

    fn status(sim: &dyn SimView, obj: ObjectSlot, status: ObjectStatusType) -> bool {
        sim.object(obj)
            .is_some_and(|o| o.status.test(status.as_i32()) != FALSE)
    }

    fn position(sim: &dyn SimView, obj: ObjectSlot) -> Coord3D {
        sim.object(obj)
            .map_or_else(Coord3D::default, |o| *o.thing.get_position())
    }

    /// `getDistanceSquared(.., FROM_CENTER_2D)`.
    fn distance_squared_2d(a: &Coord3D, b: &Coord3D) -> Real {
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        dx * dx + dy * dy
    }

    fn should_try_to_spawn(&mut self, sim: &dyn SimView, owner: ObjectSlot) -> bool {
        if self.active == FALSE {
            return false;
        }
        if Self::status(sim, owner, ObjectStatusType::Reconstructing)
            && self.data.is_one_shot != FALSE
        {
            self.stop_spawning();
            return false;
        }
        if Self::status(sim, owner, ObjectStatusType::UnderConstruction)
            || Self::status(sim, owner, ObjectStatusType::Sold)
        {
            return false;
        }
        sim.is_neutral_controlled(owner) == FALSE
    }

    /// `createSpawn`.
    ///
    /// `reclaimOrphanSpawn` never finds anything: its loop skips every name that
    /// differs from the previous one, which starts out empty. So `CanReclaimOrphans`
    /// has no effect and the door is never unreserved.
    fn create_spawn(&mut self, sim: &mut dyn SimApi, parent: ObjectSlot) -> bool {
        let Some(exit_door) = sim.try_reserve_exit_door(parent) else {
            return false;
        };
        if exit_door == crate::modules::module::ExitDoorType::DoorNoneAvailable {
            return false;
        }

        let Some((parent_id, team)) = sim.object(parent).map(|o| (o.id, o.team_id)) else {
            return false;
        };
        let name = self
            .data
            .spawn_template_names
            .get(self.template_name_index)
            .cloned()
            .unwrap_or_default();
        // The reference throws on an unknown template; nothing is spawned here.
        let Some(new_spawn) = sim.new_object_from_template(&name, team) else {
            self.spawn_template_name.clear();
            return false;
        };
        self.spawn_template_name = name;
        sim.on_unit_created(parent, new_spawn);
        self.template_name_index += 1;
        if self.template_name_index >= self.data.spawn_template_names.len() {
            self.template_name_index = 0;
        }

        Self::set_producer(sim, new_spawn, parent_id);
        sim.slaved_on_enslave(new_spawn, parent);
        let new_id = sim.object(new_spawn).map_or(ObjectId::INVALID, |o| o.id);
        self.spawn_ids.push(new_id);

        if self.data.exit_by_budding != FALSE {
            if !self.exit_through_barracks(sim, parent, parent_id, new_spawn) {
                let here = Self::position(sim, parent);
                let mut bud_host = None;
                let mut closest: Real = 999_999.9;
                for current in self.slaves(sim) {
                    if current == new_spawn {
                        continue;
                    }
                    let distance = Self::distance_squared_2d(&Self::position(sim, current), &here);
                    if distance < closest {
                        closest = distance;
                        bud_host = Some(current);
                    }
                }
                sim.exit_object_by_budding(parent, new_spawn, bud_host);
            }
        } else {
            sim.exit_object_via_door(parent, new_spawn, exit_door);
        }

        if self.data.is_one_shot != FALSE {
            self.one_shot_countdown = self.one_shot_countdown.wrapping_sub(1);
        }
        if self.spawn_count == NONE_SPAWNED_YET {
            self.spawn_count = 1;
        } else {
            self.spawn_count = self.spawn_count.wrapping_add(1);
        }
        true
    }

    fn set_producer(sim: &mut dyn SimApi, obj: ObjectSlot, producer: ObjectId) {
        if let Some(o) = sim.object_mut(obj) {
            o.producer_id = producer;
        }
    }

    /// The initial burst leaves through the door of the barracks that built the parent.
    fn exit_through_barracks(
        &mut self,
        sim: &mut dyn SimApi,
        parent: ObjectSlot,
        parent_id: ObjectId,
        new_spawn: ObjectSlot,
    ) -> bool {
        if self.initial_burst_countdown == 0 {
            return false;
        }
        let producer_id = sim
            .object(parent)
            .map_or(ObjectId::INVALID, |o| o.producer_id);
        let Some(barracks) = sim.find_object(producer_id) else {
            return false;
        };
        if sim.object_is_kind_of(barracks, KindOfType::Structure) == FALSE {
            return false;
        }
        let Some(door) = sim.try_reserve_exit_door(barracks) else {
            return false;
        };
        sim.exit_object_via_door(barracks, new_spawn, door);
        Self::set_producer(sim, new_spawn, parent_id);
        self.initial_burst_countdown -= 1;
        true
    }

    /// `computeAggregateStates`: the parent shows the combined state of its spawns.
    pub fn compute_aggregate_states(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.aggregate_health == FALSE {
            return;
        }
        let owner = ctx.owner_slot;
        let mut spawn_count: Int = 0;
        let mut average_position = Coord3D::default();
        let mut health: Real = 0.0;
        let mut max_health: Real = 0.0;
        // Every spawn is compared against the level the parent had on entry.
        let level: VeterancyLevel = sim.object_veterancy_level(owner);
        self.self_tasking_spawn_count = 0;

        for i in 0..self.spawn_ids.len() {
            let Some(current) = sim.find_object(self.spawn_ids[i]) else {
                continue;
            };
            if sim.slaved_is_self_tasking(current) != FALSE {
                self.self_tasking_spawn_count = self.self_tasking_spawn_count.wrapping_add(1);
            }
            let spawn_level = sim.object_veterancy_level(current);
            if spawn_level > level {
                sim.set_veterancy_level(owner, spawn_level, FALSE, TRUE);
            } else if spawn_level < level {
                sim.set_veterancy_level(current, level, FALSE, TRUE);
            }
            average_position.add(&Self::position(sim, current));
            health += sim.object_health(current);
            max_health += sim.object_max_health(current);
            spawn_count += 1;
        }

        let scale = 1.0 / spawn_count as Real;
        average_position.x = reference_mul(average_position.x, scale);
        average_position.y = reference_mul(average_position.y, scale);
        average_position.z = reference_mul(average_position.z, scale);
        average_position.sub(&Self::position(sim, owner));
        if let Some(o) = sim.object_mut(owner) {
            o.health_box_offset = average_position;
        }

        if spawn_count != 0 {
            max_health /= spawn_count as Real;
            let perfect_total = max_health * self.data.spawn_number as Real;
            let actual = health / perfect_total;
            sim.set_initial_health(owner, (100.0 * actual) as Int);
        } else {
            sim.set_initial_health(owner, 0);
        }
        sim.set_status(owner, ObjectStatusType::Masked);
    }

    fn apply_on_spawn_death(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        dead_spawn: ObjectId,
        source_id: ObjectId,
    ) {
        let Some(at) = self.spawn_ids.iter().position(|&id| id == dead_spawn) else {
            return;
        };
        let replacement = (self.data.spawn_replace_delay as UnsignedInt).wrapping_add(sim.frame());
        self.replacement_times.push(replacement as Int);
        self.spawn_ids.remove(at);
        self.spawn_count = self.spawn_count.wrapping_sub(1);

        if self.spawn_count == 0 && self.aggregate_health != FALSE {
            if sim.find_object(source_id).is_some() {
                let mut dmg = DamageInfo::default();
                dmg.input.source_id = source_id;
                sim.notify_object_killed(ctx.owner_slot, source_id, &dmg);
            }
            sim.destroy_object(ctx.owner_slot);
        }
    }

    /// The answer of the first slave that can attack, else whether any shot was invalid.
    fn best_attack_result(
        &self,
        sim: &dyn SimView,
        mut ask: impl FnMut(ObjectSlot) -> CanAttackResult,
    ) -> CanAttackResult {
        let mut invalid_shot = false;
        for slave in self.slaves(sim) {
            match ask(slave) {
                result @ (CanAttackResult::Possible | CanAttackResult::PossibleAfterMoving) => {
                    return result;
                }
                CanAttackResult::NotPossible => {}
                CanAttackResult::InvalidShot => invalid_shot = true,
            }
        }
        if invalid_shot {
            CanAttackResult::InvalidShot
        } else {
            CanAttackResult::NotPossible
        }
    }
}

impl SpawnBehaviorFacet for SpawnBehavior {
    fn may_spawn_self_task_ai(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        max_self_taskers_ratio: Real,
    ) -> Bool {
        if self.spawn_count == 0
            || max_self_taskers_ratio == 0.0
            || sim.has_ai(ctx.owner_slot) == FALSE
            || sim.ai_last_command_source(ctx.owner_slot) != CommandSourceType::FromAi
        {
            return FALSE;
        }
        let ratio = self.self_tasking_spawn_count as Real / self.spawn_count as Real;
        Bool::from(ratio < max_self_taskers_ratio)
    }

    fn on_spawn_death(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        dead_spawn: ObjectId,
        dmg: Option<&DamageInfo>,
    ) {
        let source = dmg.map_or(ObjectId::INVALID, |d| d.input.source_id);
        self.apply_on_spawn_death(sim, ctx, dead_spawn, source);
    }

    fn get_closest_slave(&self, sim: &dyn SimView, pos: &Coord3D) -> Option<ObjectSlot> {
        let mut closest = None;
        let mut closest_distance: Real = 0.0;
        for slave in self.slaves(sim) {
            let distance = Self::distance_squared_2d(&Self::position(sim, slave), pos);
            if closest.is_none() || closest_distance > distance {
                closest = Some(slave);
                closest_distance = distance;
            }
        }
        closest
    }

    fn order_slaves_to_attack_target(
        &mut self,
        sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        target: ObjectSlot,
        max_shots_to_fire: Int,
        cmd_source: CommandSourceType,
    ) {
        self.for_each_slave(sim, |sim, slave| {
            sim.ai_force_attack_object(slave, target, max_shots_to_fire, cmd_source);
        });
    }

    fn order_slaves_to_attack_position(
        &mut self,
        sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        pos: &Coord3D,
        max_shots_to_fire: Int,
        cmd_source: CommandSourceType,
    ) {
        self.for_each_slave(sim, |sim, slave| {
            sim.ai_attack_position(slave, pos, max_shots_to_fire, cmd_source);
        });
    }

    fn get_can_any_slaves_attack_specific_target(
        &self,
        sim: &dyn SimView,
        attack_type: UnsignedInt,
        target: ObjectSlot,
        cmd_source: CommandSourceType,
    ) -> CanAttackResult {
        self.best_attack_result(sim, |slave| {
            sim.able_to_attack_specific_object(slave, attack_type, target, cmd_source)
        })
    }

    fn get_can_any_slaves_use_weapon_against_target(
        &self,
        sim: &dyn SimView,
        attack_type: UnsignedInt,
        victim: Option<ObjectSlot>,
        pos: &Coord3D,
        cmd_source: CommandSourceType,
    ) -> CanAttackResult {
        self.best_attack_result(sim, |slave| {
            sim.able_to_use_weapon_against_target(slave, attack_type, victim, pos, cmd_source)
        })
    }

    fn can_any_slaves_attack(&self, sim: &dyn SimView) -> Bool {
        Bool::from(
            self.slaves(sim)
                .any(|slave| sim.object_is_able_to_attack(slave) != FALSE),
        )
    }

    fn order_slaves_to_go_idle(
        &mut self,
        sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        cmd_source: CommandSourceType,
    ) {
        self.for_each_slave(sim, |sim, slave| sim.ai_idle(slave, cmd_source));
    }

    fn order_slaves_disabled_until(
        &mut self,
        sim: &mut dyn SimApi,
        disabled_type: DisabledType,
        frame: UnsignedInt,
    ) {
        self.for_each_slave(sim, |sim, slave| {
            sim.ai_idle(slave, CommandSourceType::FromAi);
            sim.set_disabled_until(slave, disabled_type, frame);
        });
    }

    fn order_slaves_to_clear_disabled(
        &mut self,
        sim: &mut dyn SimApi,
        disabled_type: DisabledType,
    ) {
        self.for_each_slave(sim, |sim, slave| {
            sim.clear_disabled(slave, disabled_type);
        });
    }

    fn give_slaves_stealth_upgrade(&mut self, sim: &mut dyn SimApi, grant_stealth: Bool) {
        self.for_each_slave(sim, |sim, slave| {
            if grant_stealth != FALSE {
                sim.set_status(slave, ObjectStatusType::CanStealth);
            } else {
                sim.clear_status(slave, ObjectStatusType::CanStealth);
            }
        });
    }

    fn are_all_slaves_stealthed(&self, sim: &mut dyn SimApi) -> Bool {
        for slave in self.slave_slots(sim) {
            let stealth = sim.object(slave).and_then(|o| o.stealth_slot);
            if stealth.is_none() || sim.stealth_allowed_to_stealth(slave) == FALSE {
                return FALSE;
            }
        }
        TRUE
    }

    fn reveal_slaves(&mut self, sim: &mut dyn SimApi) {
        self.for_each_slave(sim, |sim, slave| sim.mark_stealth_detected(slave, 0));
    }

    fn do_slaves_have_freedom(&self) -> Bool {
        self.data.slaves_have_free_will
    }

    fn get_slave_count(&self) -> Int {
        self.spawn_count
    }

    fn stop_spawning(&mut self) {
        self.active = FALSE;
    }

    fn start_spawning(&mut self) {
        self.active = TRUE;
    }
}

impl UpdateFacet for SpawnBehavior {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let owner = ctx.owner_slot;
        if self.aggregate_health != FALSE {
            self.compute_aggregate_states(sim, ctx);
        }

        if self.initial_burst_times_inited == FALSE {
            self.initial_burst_times_inited = TRUE;
            let runtime_produced = sim
                .object(owner)
                .is_some_and(|o| o.producer_id != ObjectId::INVALID);
            for index in 0..self.data.spawn_number {
                // With a burst the reference queues `runtimeProduced` itself, not the
                // staggered birth frame it computes.
                self.replacement_times.push(if self.data.initial_burst > 0 {
                    Int::from(runtime_produced)
                } else {
                    index
                });
            }
        }

        self.frames_to_wait = self.frames_to_wait.wrapping_sub(1);
        if self.frames_to_wait > 0 {
            return UpdateSleepTime::NONE;
        }
        self.frames_to_wait = SPAWN_UPDATE_RATE;

        if self.should_try_to_spawn(sim, owner) {
            let mut i = 0;
            while i < self.replacement_times.len() {
                let replacement = self.replacement_times[i];
                if sim.frame() > replacement as UnsignedInt && self.create_spawn(sim, owner) {
                    self.replacement_times.remove(i);
                } else {
                    i += 1;
                }
            }
            if self.data.is_one_shot != FALSE && self.one_shot_countdown <= 0 {
                self.stop_spawning();
            }
        }
        UpdateSleepTime::NONE
    }
}

impl DieFacet for SpawnBehavior {
    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        if self
            .data
            .die_mux
            .is_die_applicable(sim, ctx.owner_slot, dmg)
            == FALSE
        {
            return;
        }
        self.for_each_slave(sim, |sim, slave| {
            sim.notify_slaver_die(slave);
            Self::set_producer(sim, slave, ObjectId::INVALID);
        });
        if self.data.spawned_require_spawner != FALSE {
            self.for_each_slave(sim, |sim, slave| {
                if !Self::is_effectively_dead(sim, slave) {
                    kill_object(sim, slave, DamageType::Unresistable, DeathType::Normal);
                }
            });
        }
    }
}

impl DamageFacet for SpawnBehavior {
    fn on_damage(&mut self, sim: &mut dyn SimApi, _ctx: ModuleContext, dmg: &mut DamageInfo) {
        self.for_each_slave(sim, |sim, slave| sim.slaved_on_slaver_damage(slave, dmg));
    }
}

impl BehaviorModule for SpawnBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, sim: &mut dyn SimApi, _ctx: ModuleContext) {
        // The constructor looks up the first spawn template.
        if let Some(name) = self.data.spawn_template_names.first() {
            if sim.thing_template_exists(name) != FALSE {
                self.spawn_template_name = name.clone();
            }
        }
    }

    fn on_delete(&mut self, sim: &mut dyn SimApi, _ctx: ModuleContext) {
        if self.data.spawned_require_spawner == FALSE {
            return;
        }
        self.for_each_slave(sim, |sim, slave| {
            if !Self::is_effectively_dead(sim, slave) {
                sim.destroy_object(slave);
            }
        });
    }

    fn apply_deferred_op(&mut self, op: DeferredSelfOp, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if let DeferredSelfOp::SpawnNotifySlaveDied {
            slave_id,
            source_id,
        } = op
        {
            self.apply_on_spawn_death(sim, ctx, slave_id, source_id);
        }
    }

    // `SpawnBehavior` calls `BehaviorModule::crc` and `BehaviorModule::xfer`, not the
    // `UpdateModule` ones: the wake frame is neither saved nor checksummed.
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let current = if cfg!(feature = "retail-xfer-save") {
            2
        } else {
            3
        };
        let version = xfer_module_version(xfer, current)?;
        xfer_behavior_module_header(xfer)?;
        if version >= 2 {
            xfer.xfer_bool(&mut self.initial_burst_times_inited)?;
        }
        if version >= 3 {
            xfer.xfer_unsigned_int(&mut self.initial_burst_countdown)?;
        }
        let mut name = self.spawn_template_name.clone();
        xfer.xfer_ascii_string(&mut name)?;
        if xfer.mode() == XferMode::Load {
            if !name.is_empty() && sim.thing_template_exists(&name) == FALSE {
                return Err(XferError::InvalidData);
            }
            self.spawn_template_name = name;
        }
        xfer.xfer_int(&mut self.one_shot_countdown)?;
        xfer.xfer_int(&mut self.frames_to_wait)?;
        xfer.xfer_int(&mut self.first_batch_count)?;
        if xfer.mode() == XferMode::Load {
            self.replacement_times.clear();
        }
        xfer.xfer_stl_int_list(&mut self.replacement_times)?;
        xfer.xfer_stl_object_id_vec(&mut self.spawn_ids)?;
        xfer.xfer_bool(&mut self.active)?;
        xfer.xfer_bool(&mut self.aggregate_health)?;
        xfer.xfer_int(&mut self.spawn_count)?;
        xfer.xfer_unsigned_int(&mut self.self_tasking_spawn_count)
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

    fn as_damage_mut(&mut self) -> Option<&mut dyn DamageFacet> {
        Some(self)
    }

    fn as_spawn_behavior(&self) -> Option<&dyn SpawnBehaviorFacet> {
        Some(self)
    }

    fn as_spawn_behavior_mut(&mut self) -> Option<&mut dyn SpawnBehaviorFacet> {
        Some(self)
    }
}

pub fn parse_spawn_behavior_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = SpawnBehaviorModuleData::new(tag_key, module_name_key);
    ini.read_block_fields(|field, reader| {
        if let Some(result) = data.die_mux.parse_field(field, reader) {
            return result;
        }
        if field.eq_ignore_ascii_case("SpawnNumber") {
            reader.parse_int(&mut data.spawn_number)
        } else if field.eq_ignore_ascii_case("SpawnReplaceDelay") {
            let mut frames: UnsignedInt = 0;
            reader.parse_duration_unsigned_int(&mut frames)?;
            data.spawn_replace_delay = frames as Int;
            Ok(())
        } else if field.eq_ignore_ascii_case("OneShot") {
            reader.parse_bool(&mut data.is_one_shot)
        } else if field.eq_ignore_ascii_case("CanReclaimOrphans") {
            reader.parse_bool(&mut data.can_reclaim_orphans)
        } else if field.eq_ignore_ascii_case("AggregateHealth") {
            reader.parse_bool(&mut data.aggregate_health)
        } else if field.eq_ignore_ascii_case("ExitByBudding") {
            reader.parse_bool(&mut data.exit_by_budding)
        } else if field.eq_ignore_ascii_case("SpawnTemplateName") {
            reader.parse_ascii_string_vector_append(&mut data.spawn_template_names)
        } else if field.eq_ignore_ascii_case("SpawnedRequireSpawner") {
            reader.parse_bool(&mut data.spawned_require_spawner)
        } else if field.eq_ignore_ascii_case("PropagateDamageTypesToSlavesWhenExisting") {
            data.damage_types_to_propagate_to_slaves = parse_damage_type_flags(reader)?;
            Ok(())
        } else if field.eq_ignore_ascii_case("InitialBurst") {
            reader.parse_int(&mut data.initial_burst)
        } else if field.eq_ignore_ascii_case("SlavesHaveFreeWill") {
            reader.parse_bool(&mut data.slaves_have_free_will)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_spawn_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let data = data
        .as_any()
        .downcast_ref::<SpawnBehaviorModuleData>()
        .expect("SpawnBehaviorModuleData")
        .clone();
    Box::new(SpawnBehavior::new(data))
}
