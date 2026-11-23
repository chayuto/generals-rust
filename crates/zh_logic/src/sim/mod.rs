//! `WP-5.0`: Core simulation spine (`SimContext`, `RulesRegistry`, `SimWorld`, `ObjectRecord`,
//! `SimView`, `SimApi`, `ClientSink`, and `MockSimApi`).
//!
//! Enforces strict three-layer separation (`docs/ARCHITECTURE_SPEC.md` §1.2–§1.5):
//! 1. [`RulesRegistry`] (immutable during a match; reset truncates map overrides)
//! 2. [`SimWorld`] (all mutable state hashed by `GameLogic::getCRC`)
//! 3. [`ClientSink`] (write-only visual/audio notifications, no-op in headless simulation)

use crate::arena::{
    DeferredSelfOp, FacetMask, ModuleArena, ModuleContext, ModuleDataId, ModuleMeta,
    ModuleSidecar, ModuleSlot, ObjectArena, ObjectSlot, TakeError, ThingTemplateId,
};
use crate::combat::armor::ArmorStore;
use crate::combat::damage::DamageInfo;
use crate::combat::locomotor::LocomotorStore;
use crate::gamelogic::global_data::{BodyDamageType, GlobalData};
use crate::modules::{BehaviorModule, ModuleData, ModuleDataStore, ModuleRegistry};
use crate::object::experience_tracker::ExperienceTrackerState;
use crate::object::thing::{make_align_to_normal_matrix, Thing, ThingTerrainAccess};
use crate::object::thing_factory::ThingFactory;
use crate::object::weapon::WeaponStore;
use crate::object::weapon_set::WeaponSet;
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
use crate::rules::combat_types::ArmorSetFlags;
use crate::rules::disabled_types::{DisabledMaskType, DISABLED_COUNT};
use crate::rules::object_status_types::{ObjectStatusMaskType, ObjectStatusType};
use crate::rules::science::ScienceStore;
use crate::rules::special_power::SpecialPowerStore;
use crate::rules::upgrade::{UpgradeCenter, UpgradeMaskType};
use crate::scheduler::{
    should_process_disabled_update, ScheduleStore, SleepyUpdateHeap, SleepyUpdatePhase,
};
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::math::geometry::GeometryInfo;
use zh_common::math::Coord3D;
use zh_common::random_value::PrngState;
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, TeamId, UnsignedInt, UpdateSleepTime, FALSE,
};

/// Write-only client/audio notification sink (`GameClient` boundary in `SimContext`).
pub trait ClientSink: Send {
    fn on_object_created(&mut self, _id: ObjectId, _template_id: ThingTemplateId) {}
    fn on_object_destroyed(&mut self, _id: ObjectId) {}
    fn on_damage(&mut self, _target: ObjectId, _amount: Real) {}
    fn on_veterancy_changed(&mut self, _target: ObjectId, _old_level: Int, _new_level: Int) {}
}

/// Headless no-op [`ClientSink`] used for Gate V2 tests and headless simulation.
#[derive(Debug, Default)]
pub struct NoopClientSink;

impl ClientSink for NoopClientSink {}

/// Flat zero-altitude terrain stub implementing [`ThingTerrainAccess`] for unit/headless tests.
#[derive(Debug, Default)]
pub struct FlatTerrain;

impl ThingTerrainAccess for FlatTerrain {
    fn get_ground_height(&self, _x: Real, _y: Real, normal: Option<&mut Coord3D>) -> Real {
        if let Some(n) = normal {
            *n = Coord3D::new(0.0, 0.0, 1.0);
        }
        0.0
    }

    fn align_on_terrain(
        &self,
        angle: Real,
        pos: &Coord3D,
        _stick_to_ground: Bool,
        mtx: &mut zh_common::math::Matrix3D,
    ) -> PathfindLayerEnum {
        make_align_to_normal_matrix(angle, pos, &Coord3D::new(0.0, 0.0, 1.0), mtx);
        PathfindLayerEnum::Ground
    }

    fn is_underwater(
        &self,
        _x: Real,
        _y: Real,
        water_z: Option<&mut Real>,
        terrain_z: Option<&mut Real>,
    ) -> Bool {
        if let Some(wz) = water_z {
            *wz = 0.0;
        }
        if let Some(tz) = terrain_z {
            *tz = 0.0;
        }
        FALSE
    }
}

/// Immutable INI-loaded simulation rules and module factories.
pub struct RulesRegistry {
    pub name_keys: NameKeyGenerator,
    pub thing_factory: ThingFactory,
    pub module_registry: ModuleRegistry,
    pub module_data_store: ModuleDataStore,
    pub armor_store: ArmorStore,
    pub weapon_store: WeaponStore,
    pub locomotor_store: LocomotorStore,
    pub science_store: ScienceStore,
    pub upgrade_center: UpgradeCenter,
    pub special_power_store: SpecialPowerStore,
    pub global_data: GlobalData,
}

impl Default for RulesRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl RulesRegistry {
    pub fn new() -> Self {
        let mut name_keys = NameKeyGenerator::new();
        name_keys.init();
        let module_registry = ModuleRegistry::with_builtin_zh_modules(&mut name_keys);
        let mut upgrade_center = UpgradeCenter::new();
        upgrade_center.init(&mut name_keys);
        Self {
            name_keys,
            thing_factory: ThingFactory::new(),
            module_registry,
            module_data_store: ModuleDataStore::new(),
            armor_store: ArmorStore::new(),
            weapon_store: WeaponStore::new(),
            locomotor_store: LocomotorStore::new(),
            science_store: ScienceStore::new(),
            upgrade_center,
            special_power_store: SpecialPowerStore::new(),
            global_data: GlobalData::new(),
        }
    }

    /// Snapshots base INI boundary before loading map-local overrides.
    pub fn mark_base_loaded(&mut self) {
        self.module_data_store.mark_base_loaded();
    }

    /// Cleans up map-local overrides between matches (`ThingFactory::reset` + `ModuleDataStore::reset`).
    pub fn reset(&mut self) {
        self.thing_factory.reset();
        self.module_data_store.reset();
    }
}

/// Slot-stored object state (`Object` in `ObjectArena<ObjectRecord>`), embedding [`Thing`].
#[derive(Debug, Clone)]
pub struct ObjectRecord {
    pub id: ObjectId,
    pub thing: Thing,
    pub team_id: TeamId,
    pub status: ObjectStatusMaskType,
    pub private_status: u8,
    pub geometry: GeometryInfo,
    pub disabled_mask: DisabledMaskType,
    pub disabled_till_frame: [UnsignedInt; DISABLED_COUNT],
    pub upgrades_completed: UpgradeMaskType,
    pub weapon_bonus_condition: UnsignedInt,
    pub experience_tracker: ExperienceTrackerState,
    pub weapon_set: WeaponSet,
    pub armor_set_flags: ArmorSetFlags,
    pub layer: PathfindLayerEnum,
    pub contained_by: Option<ObjectSlot>,
    pub producer_id: ObjectId,
    pub builder_id: ObjectId,
    pub behaviors: Vec<(ModuleSlot, FacetMask)>,
    pub body_slot: Option<ModuleSlot>,
    pub contain_slot: Option<ModuleSlot>,
    pub stealth_slot: Option<ModuleSlot>,
    pub ai_slot: Option<ModuleSlot>,
    pub physics_slot: Option<ModuleSlot>,
}

impl ObjectRecord {
    pub fn new(
        id: ObjectId,
        template_id: ThingTemplateId,
        team_id: TeamId,
        geometry: GeometryInfo,
    ) -> Self {
        Self {
            id,
            thing: Thing::new(template_id),
            team_id,
            status: ObjectStatusMaskType::new(),
            private_status: 0,
            geometry,
            disabled_mask: DisabledMaskType::new(),
            disabled_till_frame: [0; DISABLED_COUNT],
            upgrades_completed: UpgradeMaskType::new(),
            weapon_bonus_condition: 0,
            experience_tracker: ExperienceTrackerState::default(),
            weapon_set: WeaponSet::new(),
            armor_set_flags: ArmorSetFlags::new(),
            layer: PathfindLayerEnum::Ground,
            contained_by: None,
            producer_id: ObjectId::INVALID,
            builder_id: ObjectId::INVALID,
            behaviors: Vec::new(),
            body_slot: None,
            contain_slot: None,
            stealth_slot: None,
            ai_slot: None,
            physics_slot: None,
        }
    }

    #[inline]
    pub fn is_destroyed(&self) -> bool {
        self.status.test(ObjectStatusType::Destroyed.as_i32()) != FALSE
    }
}

/// Read-only simulation view passed to `Snapshot::xfer` and const module queries.
pub trait SimView {
    fn frame(&self) -> UnsignedInt;
    fn rules(&self) -> &RulesRegistry;
    fn module_data(&self, id: ModuleDataId) -> Option<&dyn ModuleData>;
    fn find_object(&self, id: ObjectId) -> Option<ObjectSlot>;
    fn object(&self, slot: ObjectSlot) -> Option<&ObjectRecord>;
    fn module_sidecar(&self, slot: ModuleSlot) -> ModuleSidecar;
    fn terrain(&self) -> &dyn ThingTerrainAccess;
}

/// Mutable simulation interface passed into all [`BehaviorModule`] facet callbacks.
pub trait SimApi: SimView {
    fn rng_logic_int(&mut self, lo: Int, hi: Int) -> Int;
    fn rng_logic_real(&mut self, lo: Real, hi: Real) -> Real;
    fn object_mut(&mut self, slot: ObjectSlot) -> Option<&mut ObjectRecord>;
    fn set_module_sidecar(&mut self, slot: ModuleSlot, sidecar: ModuleSidecar);
    fn awaken_module(&mut self, slot: ModuleSlot, when: UnsignedInt) -> bool;
    fn set_wake_frame(&mut self, slot: ModuleSlot, sleep: UpdateSleepTime);
    fn spawn_object(
        &mut self,
        tmpl: ThingTemplateId,
        team: TeamId,
        pos: Coord3D,
        angle: Real,
    ) -> ObjectSlot;
    fn destroy_object(&mut self, slot: ObjectSlot);
    fn attempt_damage(&mut self, target: ObjectSlot, dmg: &mut DamageInfo);
    fn client(&mut self) -> &mut dyn ClientSink;
}

/// Mutable per-match simulation state (`GameLogic`).
pub struct SimWorld {
    pub frame: UnsignedInt,
    pub rng: PrngState,
    pub objects: ObjectArena<ObjectRecord>,
    pub modules: ModuleArena<Box<dyn BehaviorModule>>,
    pub sleepy_updates: SleepyUpdateHeap<ModuleSlot>,
    pub objects_to_destroy: Vec<(ObjectSlot, u32)>,
    pub next_object_id: Int,
}

impl Default for SimWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl SimWorld {
    pub fn new() -> Self {
        Self {
            frame: 1,
            rng: PrngState::new(),
            objects: ObjectArena::new(),
            modules: ModuleArena::new(),
            sleepy_updates: SleepyUpdateHeap::new(),
            objects_to_destroy: Vec::new(),
            next_object_id: 1,
        }
    }

    pub fn allocate_object_id(&mut self) -> ObjectId {
        let id = ObjectId(self.next_object_id);
        self.next_object_id = self.next_object_id.wrapping_add(1);
        if self.next_object_id <= 0 {
            self.next_object_id = 1;
        }
        id
    }

    /// Allocates `module` into [`Self::modules`], links it to `obj_slot`, and pushes any
    /// `UPDATE` facet into [`Self::sleepy_updates`] if the object is linked in the world list.
    pub fn attach_module(
        &mut self,
        obj_slot: ObjectSlot,
        tag_key: NameKeyType,
        data_id: ModuleDataId,
        facet_mask: FacetMask,
        phase: SleepyUpdatePhase,
        module: Box<dyn BehaviorModule>,
    ) -> ModuleSlot {
        let owner_id = self.objects.id_of(obj_slot).unwrap_or(ObjectId::INVALID);
        let meta = ModuleMeta::new(obj_slot, owner_id, tag_key, data_id, facet_mask, phase);
        let mod_slot = self.modules.allocate(meta, module);

        if let Some(obj) = self.objects.get_mut(obj_slot) {
            obj.behaviors.push((mod_slot, facet_mask));
            if facet_mask.contains(FacetMask::BODY) && obj.body_slot.is_none() {
                obj.body_slot = Some(mod_slot);
            }
            if facet_mask.contains(FacetMask::CONTAIN) && obj.contain_slot.is_none() {
                obj.contain_slot = Some(mod_slot);
            }
            if facet_mask.contains(FacetMask::STEALTH_UPDATE) && obj.stealth_slot.is_none() {
                obj.stealth_slot = Some(mod_slot);
            }
            if facet_mask.contains(FacetMask::AI_UPDATE) && obj.ai_slot.is_none() {
                obj.ai_slot = Some(mod_slot);
            }
            if facet_mask.contains(FacetMask::PHYSICS_UPDATE) && obj.physics_slot.is_none() {
                obj.physics_slot = Some(mod_slot);
            }
        }

        if facet_mask.contains(FacetMask::UPDATE) && self.objects.is_in_world_list(obj_slot) {
            self.sleepy_updates
                .register_update_module(&mut self.modules, mod_slot, self.frame);
        }

        mod_slot
    }

    /// Enqueues `(slot, generation)` for Step 15a destruction (`GameLogic::m_objectsToDestroy`).
    pub fn enqueue_destroy(&mut self, slot: ObjectSlot) {
        if let Some(gen_id) = self.objects.generation(slot) {
            if !self
                .objects_to_destroy
                .iter()
                .any(|&(s, g)| s == slot && g == gen_id)
            {
                self.objects_to_destroy.push((slot, gen_id));
            }
        }
    }
}

/// Top-level three-layer simulation context (`RulesRegistry` + `SimWorld` + `ClientSink`).
pub struct SimContext {
    pub rules: RulesRegistry,
    pub world: SimWorld,
    pub terrain: Box<dyn ThingTerrainAccess>,
    pub client: Box<dyn ClientSink>,
}

impl Default for SimContext {
    fn default() -> Self {
        Self::new()
    }
}

impl SimContext {
    pub fn new() -> Self {
        Self {
            rules: RulesRegistry::new(),
            world: SimWorld::new(),
            terrain: Box::new(FlatTerrain),
            client: Box::new(NoopClientSink),
        }
    }

    /// Dispatches `f` on `mod_slot` via Take-and-Restore, draining any re-entrant [`DeferredSelfOp`]s
    /// and refreshing [`ModuleSidecar`] before restoring ownership to `world.modules`.
    pub fn dispatch_module<R>(
        &mut self,
        mod_slot: ModuleSlot,
        f: impl FnOnce(&mut dyn BehaviorModule, &mut dyn SimApi, ModuleContext) -> R,
    ) -> Result<R, TakeError> {
        let (mut payload, ctx, gen_id) = self.world.modules.take_payload(mod_slot)?;
        let out = f(&mut *payload, self, ctx);

        loop {
            let ops = if self.world.modules.is_executing(mod_slot)
                && self.world.modules.meta(mod_slot).generation == gen_id
            {
                self.world.modules.drain_deferred_ops(mod_slot)
            } else {
                Vec::new()
            };
            if ops.is_empty() {
                break;
            }
            for op in ops {
                payload.apply_deferred_op(op, self, ctx);
            }
        }

        if self.world.modules.is_executing(mod_slot)
            && self.world.modules.meta(mod_slot).generation == gen_id
        {
            let sidecar = payload.sync_sidecar();
            if sidecar != ModuleSidecar::None {
                self.world.modules.set_sidecar(mod_slot, sidecar);
            }
        }
        self.world.modules.restore_payload(mod_slot, gen_id, payload);
        Ok(out)
    }

    /// Marks `slot` destroyed (`GameLogic::destroyObject`), running `DestroyFacet::on_destroy`
    /// and queueing `slot` for end-of-frame deletion in [`Self::process_destroy_list`].
    pub fn destroy_object_now(&mut self, slot: ObjectSlot) {
        let Some(obj) = self.world.objects.get(slot) else {
            return;
        };
        if obj.is_destroyed() {
            return;
        }
        let obj_id = obj.id;
        let destroy_slots: Vec<ModuleSlot> = obj
            .behaviors
            .iter()
            .filter(|(_, mask)| mask.contains(FacetMask::DESTROY))
            .map(|&(ms, _)| ms)
            .collect();
        let ai_slot = obj.ai_slot;

        for ms in destroy_slots {
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(d) = m.as_destroy_mut() {
                    d.on_destroy(sim, ctx);
                }
            });
        }

        if let Some(obj_mut) = self.world.objects.get_mut(slot) {
            obj_mut.status.set(ObjectStatusType::Destroyed.as_i32(), 1);
        }

        if let Some(ai_ms) = ai_slot {
            let _ = self.dispatch_module(ai_ms, |m, sim, ctx| {
                if let Some(ai) = m.as_ai_mut() {
                    ai.set_locomotor_goal_none(sim, ctx);
                    ai.destroy_path();
                }
            });
        }

        self.world.enqueue_destroy(slot);
        self.client.on_object_destroyed(obj_id);
    }

    /// Destroys all queued objects (`GameLogic::processDestroyList`), handling child objects
    /// enqueued dynamically inside `on_delete()`, two-pass heap removal, and arena despawn.
    pub fn process_destroy_list(&mut self) {
        let mut i = 0;
        while i < self.world.objects_to_destroy.len() {
            let (slot, gen_id) = self.world.objects_to_destroy[i];
            i += 1;

            if !self.world.objects.is_slot_current(slot, gen_id) {
                continue;
            }
            let beh_slots: Vec<ModuleSlot> = self
                .world
                .objects
                .get(slot)
                .map_or_else(Vec::new, |o| o.behaviors.iter().map(|&(ms, _)| ms).collect());

            for &ms in &beh_slots {
                if self.world.modules.is_alive(ms) {
                    let _ = self.dispatch_module(ms, |m, sim, ctx| {
                        m.on_delete(sim, ctx);
                    });
                }
            }

            let modules = &mut self.world.modules;
            self.world
                .sleepy_updates
                .erase_for_owner(modules, |m, ms| m.meta(ms).owner_slot == slot);

            for ms in beh_slots {
                let _ = self.world.modules.free(ms);
            }
            let _ = self.world.objects.despawn(slot);
        }
        self.world.objects_to_destroy.clear();
    }

    /// Executes all due sleepy updates for `self.world.frame` (`GameLogic.cpp:4062-4113`).
    pub fn step_sleepy_updates(&mut self) {
        let now = self.world.frame;
        while let Some(u) = self.world.sleepy_updates.peek_due(&self.world.modules, now) {
            let owner_slot = self.world.modules.meta(u).owner_slot;
            let disabled_bits = self
                .world
                .objects
                .get(owner_slot)
                .map_or(0, |o| o.disabled_mask.to_unsigned_int() as u64);
            let proc_bits = self
                .world
                .modules
                .payload(u)
                .and_then(|m| m.as_update())
                .map_or(0, |upd| upd.disabled_types_to_process().to_unsigned_int() as u64);

            let mut sleep_len = UpdateSleepTime::NONE;
            if should_process_disabled_update(disabled_bits, proc_bits) {
                self.world.sleepy_updates.set_cur_update_module(Some(u));
                if let Ok(ret) = self.dispatch_module(u, |m, sim, ctx| {
                    m.as_update_mut()
                        .map_or(UpdateSleepTime::FOREVER, |upd| upd.update(sim, ctx))
                }) {
                    sleep_len = ret;
                }
            }

            if self.world.modules.is_alive(u)
                && self.world.modules.schedule(u).index_in_logic() >= 0
            {
                self.world
                    .sleepy_updates
                    .finish_due_update(&mut self.world.modules, u, now, sleep_len);
            } else {
                self.world.sleepy_updates.set_cur_update_module(None);
            }
        }
    }

    /// Routes damage through `target`'s `BodyFacet`, handling same-slot re-entrancy via
    /// [`ModuleSidecar::Body`] immediate sync + [`DeferredSelfOp::ChangeHealth`].
    pub fn attempt_damage_on(&mut self, target: ObjectSlot, dmg: &mut DamageInfo) {
        let Some(body_slot) = self.world.objects.get(target).and_then(|o| o.body_slot) else {
            return;
        };
        if self.world.modules.is_executing(body_slot) {
            let delta = -dmg.input.amount;
            if let ModuleSidecar::Body {
                mut health,
                max_health,
                prev_health: _,
                damage_scalar,
                mut damage_state,
                front_crushed,
                back_crushed,
                indestructible,
            } = self.world.modules.sidecar(body_slot)
            {
                if indestructible == FALSE {
                    let prev_health = health;
                    health += delta;
                    if health < 0.0 {
                        health = 0.0;
                    }
                    if health > max_health && max_health > 0.0 {
                        health = max_health;
                    }
                    if health <= 0.0 {
                        damage_state = BodyDamageType::Rubble.as_i32() as u8;
                    }
                    self.world.modules.set_sidecar(
                        body_slot,
                        ModuleSidecar::Body {
                            health,
                            max_health,
                            prev_health,
                            damage_scalar,
                            damage_state,
                            front_crushed,
                            back_crushed,
                            indestructible,
                        },
                    );
                }
            }
            let _ = self.world.modules.push_deferred_op(
                body_slot,
                DeferredSelfOp::ChangeHealth {
                    delta,
                    change_type: 0,
                },
            );
            return;
        }

        if let Some(p) = self.world.modules.payload(body_slot) {
            let sc = p.sync_sidecar();
            if sc != ModuleSidecar::None {
                self.world.modules.set_sidecar(body_slot, sc);
            }
        }
        let _ = self.dispatch_module(body_slot, |m, sim, ctx| {
            if let Some(body) = m.as_body_mut() {
                body.attempt_damage(sim, ctx, dmg);
            }
        });
    }
}

impl SimView for SimContext {
    fn frame(&self) -> UnsignedInt {
        self.world.frame
    }

    fn rules(&self) -> &RulesRegistry {
        &self.rules
    }

    fn module_data(&self, id: ModuleDataId) -> Option<&dyn ModuleData> {
        self.rules.module_data_store.get(id)
    }

    fn find_object(&self, id: ObjectId) -> Option<ObjectSlot> {
        self.world.objects.slot_by_id(id)
    }

    fn object(&self, slot: ObjectSlot) -> Option<&ObjectRecord> {
        self.world.objects.get(slot)
    }

    fn module_sidecar(&self, slot: ModuleSlot) -> ModuleSidecar {
        self.world.modules.sidecar(slot)
    }

    fn terrain(&self) -> &dyn ThingTerrainAccess {
        self.terrain.as_ref()
    }
}

impl SimApi for SimContext {
    fn rng_logic_int(&mut self, lo: Int, hi: Int) -> Int {
        self.world.rng.get_game_logic_random_value(lo, hi)
    }

    fn rng_logic_real(&mut self, lo: Real, hi: Real) -> Real {
        self.world.rng.get_game_logic_random_value_real(lo, hi)
    }

    fn object_mut(&mut self, slot: ObjectSlot) -> Option<&mut ObjectRecord> {
        self.world.objects.get_mut(slot)
    }

    fn set_module_sidecar(&mut self, slot: ModuleSlot, sidecar: ModuleSidecar) {
        self.world.modules.set_sidecar(slot, sidecar);
    }

    fn awaken_module(&mut self, slot: ModuleSlot, when: UnsignedInt) -> bool {
        if !self.world.modules.is_alive(slot) {
            return false;
        }
        let owner_slot = self.world.modules.meta(slot).owner_slot;
        let in_world = self.world.objects.is_in_world_list(owner_slot);
        self.world.sleepy_updates.awaken_module(
            &mut self.world.modules,
            slot,
            in_world,
            self.world.frame,
            when,
        )
    }

    fn set_wake_frame(&mut self, slot: ModuleSlot, sleep: UpdateSleepTime) {
        let when = self.world.frame.saturating_add(sleep.0);
        let _ = self.awaken_module(slot, when);
    }

    fn spawn_object(
        &mut self,
        tmpl: ThingTemplateId,
        team: TeamId,
        pos: Coord3D,
        angle: Real,
    ) -> ObjectSlot {
        let id = self.world.allocate_object_id();
        let geom = self
            .rules
            .thing_factory
            .find_by_template_id_final(tmpl.0 as u16)
            .map_or_else(GeometryInfo::default, |t| {
                t.get_template_geometry_info().clone()
            });
        let mut record = ObjectRecord::new(id, tmpl, team, geom);
        let mut dummy_reactor = crate::object::thing::NoopTransformReactor;
        let mut mtx = zh_common::math::Matrix3D::identity();
        self.terrain.align_on_terrain(angle, &pos, FALSE, &mut mtx);
        record.thing.set_transform_matrix(&mtx, &mut dummy_reactor);
        let slot = self.world.objects.spawn(id, record, true);
        self.client.on_object_created(id, tmpl);
        slot
    }

    fn destroy_object(&mut self, slot: ObjectSlot) {
        self.destroy_object_now(slot);
    }

    fn attempt_damage(&mut self, target: ObjectSlot, dmg: &mut DamageInfo) {
        self.attempt_damage_on(target, dmg);
    }

    fn client(&mut self) -> &mut dyn ClientSink {
        self.client.as_mut()
    }
}

/// Lightweight [`SimApi`] wrapper used in standalone Gate V2 module tests.
pub struct MockSimApi {
    pub ctx: SimContext,
}

impl Default for MockSimApi {
    fn default() -> Self {
        Self::new()
    }
}

impl MockSimApi {
    pub fn new() -> Self {
        Self {
            ctx: SimContext::new(),
        }
    }
}

impl SimView for MockSimApi {
    fn frame(&self) -> UnsignedInt {
        self.ctx.frame()
    }

    fn rules(&self) -> &RulesRegistry {
        self.ctx.rules()
    }

    fn module_data(&self, id: ModuleDataId) -> Option<&dyn ModuleData> {
        self.ctx.module_data(id)
    }

    fn find_object(&self, id: ObjectId) -> Option<ObjectSlot> {
        self.ctx.find_object(id)
    }

    fn object(&self, slot: ObjectSlot) -> Option<&ObjectRecord> {
        self.ctx.object(slot)
    }

    fn module_sidecar(&self, slot: ModuleSlot) -> ModuleSidecar {
        self.ctx.module_sidecar(slot)
    }

    fn terrain(&self) -> &dyn ThingTerrainAccess {
        self.ctx.terrain()
    }
}

impl SimApi for MockSimApi {
    fn rng_logic_int(&mut self, lo: Int, hi: Int) -> Int {
        self.ctx.rng_logic_int(lo, hi)
    }

    fn rng_logic_real(&mut self, lo: Real, hi: Real) -> Real {
        self.ctx.rng_logic_real(lo, hi)
    }

    fn object_mut(&mut self, slot: ObjectSlot) -> Option<&mut ObjectRecord> {
        self.ctx.object_mut(slot)
    }

    fn set_module_sidecar(&mut self, slot: ModuleSlot, sidecar: ModuleSidecar) {
        self.ctx.set_module_sidecar(slot, sidecar);
    }

    fn awaken_module(&mut self, slot: ModuleSlot, when: UnsignedInt) -> bool {
        self.ctx.awaken_module(slot, when)
    }

    fn set_wake_frame(&mut self, slot: ModuleSlot, sleep: UpdateSleepTime) {
        self.ctx.set_wake_frame(slot, sleep);
    }

    fn spawn_object(
        &mut self,
        tmpl: ThingTemplateId,
        team: TeamId,
        pos: Coord3D,
        angle: Real,
    ) -> ObjectSlot {
        self.ctx.spawn_object(tmpl, team, pos, angle)
    }

    fn destroy_object(&mut self, slot: ObjectSlot) {
        self.ctx.destroy_object(slot);
    }

    fn attempt_damage(&mut self, target: ObjectSlot, dmg: &mut DamageInfo) {
        self.ctx.attempt_damage(target, dmg);
    }

    fn client(&mut self) -> &mut dyn ClientSink {
        self.ctx.client()
    }
}
