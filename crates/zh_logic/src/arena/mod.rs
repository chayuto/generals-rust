//! `WP-5.0`: Slot-indexed `ObjectArena`, split-SoA `ModuleArena`, 64-bit `FacetMask`,
//! and re-entrancy-safe Take-and-Restore dispatch (`docs/ARCHITECTURE_SPEC.md` §1.2).

use std::collections::BTreeMap;

use crate::rules::combat_types::ArmorSetType;
use crate::scheduler::{ScheduleStore, SleepyUpdatePhase, UpdateModuleScheduleState};
use zh_common::math::Coord3D;
use zh_common::types::{Bool, Int, NameKeyType, ObjectId, Real, UnsignedInt, UnsignedShort};

/// Typed slot handle into [`ObjectArena`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ObjectSlot(pub u32);

impl ObjectSlot {
    pub const INVALID: Self = Self(u32::MAX);

    #[inline]
    pub const fn is_valid(self) -> bool {
        self.0 != u32::MAX
    }
}

/// Typed slot handle into [`ModuleArena`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ModuleSlot(pub u32);

impl ModuleSlot {
    pub const INVALID: Self = Self(u32::MAX);

    #[inline]
    pub const fn is_valid(self) -> bool {
        self.0 != u32::MAX
    }
}

/// Identifier for template `ModuleData` definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ModuleDataId(pub u32);

impl ModuleDataId {
    pub const INVALID: Self = Self(0);
}

/// Identifier for `ThingTemplate` definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ThingTemplateId(pub u32);

impl ThingTemplateId {
    pub const INVALID: Self = Self(0);
}

/// 64-bit capability mask identifying all SAGE `ModuleInterfaceType` and sub-facet interfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FacetMask(pub u64);

impl FacetMask {
    pub const NONE: Self = Self(0);
    // Primary SAGE ModuleInterfaceType bits (Module.h lines 80-94)
    pub const UPDATE: Self = Self(1 << 0);
    pub const DIE: Self = Self(1 << 1);
    pub const DAMAGE: Self = Self(1 << 2);
    pub const CREATE: Self = Self(1 << 3);
    pub const COLLIDE: Self = Self(1 << 4);
    pub const BODY: Self = Self(1 << 5);
    pub const CONTAIN: Self = Self(1 << 6);
    pub const UPGRADE: Self = Self(1 << 7);
    pub const SPECIAL_POWER: Self = Self(1 << 8);
    pub const DESTROY: Self = Self(1 << 9);
    pub const DRAW: Self = Self(1 << 10);
    pub const CLIENT_UPDATE: Self = Self(1 << 11);

    // Extended sub-facet interface bits
    pub const AI_UPDATE: Self = Self(1 << 12);
    pub const PHYSICS_UPDATE: Self = Self(1 << 13);
    pub const STEALTH_UPDATE: Self = Self(1 << 14);
    pub const DOCK_UPDATE: Self = Self(1 << 15);
    pub const EXIT_INTERFACE: Self = Self(1 << 16);
    pub const SLAVED_UPDATE: Self = Self(1 << 17);
    pub const PROJECTILE_UPDATE: Self = Self(1 << 18);
    pub const PRODUCTION_UPDATE: Self = Self(1 << 19);
    pub const SLOW_DEATH: Self = Self(1 << 20);
    pub const POWER_PLANT_UPDATE: Self = Self(1 << 21);
    pub const HORDE_UPDATE: Self = Self(1 << 22);
    pub const SPAWN_BEHAVIOR: Self = Self(1 << 23);
    pub const COUNTERMEASURES: Self = Self(1 << 24);
    pub const SPECIAL_POWER_UPDATE: Self = Self(1 << 25);
    pub const PARKING_PLACE: Self = Self(1 << 26);
    pub const CAVE_INTERFACE: Self = Self(1 << 27);
    pub const LAND_MINE: Self = Self(1 << 28);
    pub const RAILED_TRANSPORT_DOCK: Self = Self(1 << 29);
    pub const TRANSPORT_PASSENGER: Self = Self(1 << 30);
    pub const EJECT_PILOT_DIE: Self = Self(1 << 31);
    pub const BRIDGE_BEHAVIOR: Self = Self(1 << 32);
    pub const BRIDGE_TOWER: Self = Self(1 << 33);
    pub const BRIDGE_SCAFFOLD: Self = Self(1 << 34);
    pub const OVERCHARGE: Self = Self(1 << 35);
    pub const REBUILD_HOLE: Self = Self(1 << 36);
    pub const PROPAGANDA_TOWER: Self = Self(1 << 37);
    pub const TUNNEL_CONTAIN: Self = Self(1 << 38);

    pub const PRIMARY_CPP_MASK: u64 = 0x0fff;

    #[inline]
    pub const fn from_cpp_interface_mask(mask: Int) -> Self {
        Self((mask as u32 as u64) & Self::PRIMARY_CPP_MASK)
    }

    #[inline]
    pub const fn to_cpp_interface_mask(self) -> Int {
        (self.0 & Self::PRIMARY_CPP_MASK) as Int
    }

    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline]
    pub const fn contains(self, required: Self) -> bool {
        (self.0 & required.0) == required.0
    }

    #[inline]
    pub const fn intersects(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

/// Context passed to a module handler during Take-and-Restore dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleContext {
    pub mod_slot: ModuleSlot,
    pub owner_slot: ObjectSlot,
    pub owner_id: ObjectId,
    pub tag_key: NameKeyType,
    pub template_data_id: ModuleDataId,
}

/// Error returned when [`ModuleArena::take_payload`] cannot take ownership of a slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TakeError {
    Dead,
    Reentered,
}

/// Hot read-only state mirror stored inside [`ModuleMeta`] so synchronous re-entrant
/// readers (and `Object::crc`) observe up-to-date fields while a payload is taken out.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ModuleSidecar {
    #[default]
    None,
    Body {
        health: Real,
        max_health: Real,
        prev_health: Real,
        initial_health: Real,
        damage_scalar: Real,
        damage_state: u8,
        front_crushed: Bool,
        back_crushed: Bool,
        indestructible: Bool,
        current_subdual_damage: Real,
        subdued: Bool,
        last_damage_ts: UnsignedInt,
        last_healing_ts: UnsignedInt,
        armor_set_bits: u8,
    },
    Contain {
        count: u32,
        max: i32,
        enclosing: Bool,
        stealth_units: u32,
        garrisoned: Bool,
        garrisonable: Bool,
        rider_id: ObjectId,
    },
    Bridge {
        damage_state: u8,
        passable: Bool,
        scaffold_present: Bool,
    },
    ParkingPlace {
        occupied_mask: u32,
        runway_mask: u32,
        heal_count: u32,
    },
    Physics {
        velocity: Coord3D,
        mass: Real,
        airborne: Bool,
    },
    Ai {
        moving: Bool,
        attacking: Bool,
        idle: Bool,
        cur_locomotor_set: u8,
    },
    SpecialPower {
        template_id: UnsignedInt,
        available_on_frame: UnsignedInt,
        paused_count: Int,
        paused_on_frame: UnsignedInt,
        paused_percent: Real,
        starts_ready: Bool,
    },
}

/// Deferred self-mutation queued when a synchronous re-entrant callback targets an
/// already-executing [`ModuleSlot`], drained immediately on [`with_module_dispatch_and_drain`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeferredSelfOp {
    ApplyForce(Coord3D),
    ChangeHealth {
        delta: Real,
        change_type: u8,
    },
    ApplyAdjustedDamage {
        amount: Real,
        damage_type: u8,
        death_type: u8,
        damage_fx_override: u8,
        damage_status_type: u8,
        kill: Bool,
        source_id: ObjectId,
        source_player_mask: UnsignedShort,
    },
    AddSubdualDamage(Real),
    SetMaxHealth {
        new_max: Real,
        change_type: u8,
    },
    SetIndestructible(Bool),
    ApplyDamageScalar(Real),
    SetCrushed {
        front: Option<Bool>,
        back: Option<Bool>,
    },
    SetArmorSetFlag {
        flag: ArmorSetType,
        set: Bool,
    },
    SpecialPowerPause {
        pause: Bool,
    },
    SpecialPowerSetReadyFrame(UnsignedInt),
    UpgradeResetKey(NameKeyType),
    AiSetIdle,
    /// `Object::onDie` reaching a die module that is itself executing (it killed its own owner).
    Die {
        damage_type: u8,
        death_type: u8,
        source_id: ObjectId,
        amount: Real,
        kill: Bool,
        actual_damage_dealt: Real,
        actual_damage_clipped: Real,
    },
    ContainAdd(ObjectSlot),
    ContainRemove {
        obj: ObjectSlot,
        expose_stealthed: Bool,
    },
    ContainRemoveAll {
        expose_stealthed: Bool,
    },
    ContainEvacuate {
        force_exit: Bool,
    },
    ContainProcessDamage(Real),
    SlowDeathBegin {
        damage_type: u8,
        death_type: u8,
        source_id: ObjectId,
    },
    SpawnNotifySlaveDied {
        slave_id: ObjectId,
        source_id: ObjectId,
    },
    ParkingRelease(ObjectId),
    BridgeSyncTowerState {
        tower_index: u8,
        damage_state: u8,
    },
    Delete,
}

/// Disjoint module header record stored separately from the `ModuleArena` payload array so
/// `SleepyUpdateHeap` can always inspect/mutate schedules while any payload is taken out.
#[derive(Debug, Clone)]
pub struct ModuleMeta {
    pub owner_slot: ObjectSlot,
    pub owner_id: ObjectId,
    pub tag_key: NameKeyType,
    pub template_data_id: ModuleDataId,
    pub facet_mask: FacetMask,
    pub schedule: UpdateModuleScheduleState,
    pub sidecar: ModuleSidecar,
    pub deferred_ops: Vec<DeferredSelfOp>,
    pub alive: bool,
    pub executing: bool,
    pub generation: u32,
}

impl ModuleMeta {
    pub fn new(
        owner_slot: ObjectSlot,
        owner_id: ObjectId,
        tag_key: NameKeyType,
        template_data_id: ModuleDataId,
        facet_mask: FacetMask,
        phase: SleepyUpdatePhase,
    ) -> Self {
        Self {
            owner_slot,
            owner_id,
            tag_key,
            template_data_id,
            facet_mask,
            schedule: UpdateModuleScheduleState::new(phase),
            sidecar: ModuleSidecar::None,
            deferred_ops: Vec::new(),
            alive: true,
            executing: false,
            generation: 0,
        }
    }

    #[inline]
    pub const fn context(&self, mod_slot: ModuleSlot) -> ModuleContext {
        ModuleContext {
            mod_slot,
            owner_slot: self.owner_slot,
            owner_id: self.owner_id,
            tag_key: self.tag_key,
            template_data_id: self.template_data_id,
        }
    }
}

/// Split-SoA arena holding module metadata + payloads for re-entrant Take-and-Restore dispatch.
#[derive(Debug, Clone)]
pub struct ModuleArena<K> {
    metas: Vec<ModuleMeta>,
    payloads: Vec<Option<K>>,
    free_list: Vec<u32>,
    active_count: usize,
}

impl<K> Default for ModuleArena<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K> ModuleArena<K> {
    pub fn new() -> Self {
        Self {
            metas: Vec::new(),
            payloads: Vec::new(),
            free_list: Vec::new(),
            active_count: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.active_count
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.active_count == 0
    }

    pub fn allocate(&mut self, mut meta: ModuleMeta, payload: K) -> ModuleSlot {
        meta.alive = true;
        meta.executing = false;
        if let Some(reuse_idx) = self.free_list.pop() {
            let idx = reuse_idx as usize;
            let next_gen = self.metas[idx].generation.wrapping_add(1);
            meta.generation = next_gen;
            self.metas[idx] = meta;
            self.payloads[idx] = Some(payload);
            self.active_count += 1;
            ModuleSlot(reuse_idx)
        } else {
            let slot = ModuleSlot(self.metas.len() as u32);
            meta.generation = 1;
            self.metas.push(meta);
            self.payloads.push(Some(payload));
            self.active_count += 1;
            slot
        }
    }

    pub fn free(&mut self, slot: ModuleSlot) -> Option<K> {
        let idx = slot.0 as usize;
        if idx >= self.metas.len() || !self.metas[idx].alive {
            return None;
        }
        self.metas[idx].alive = false;
        self.metas[idx].executing = false;
        self.metas[idx].generation = self.metas[idx].generation.wrapping_add(1);
        self.metas[idx].schedule.set_index_in_logic(-1);
        self.metas[idx].sidecar = ModuleSidecar::None;
        self.metas[idx].deferred_ops.clear();
        self.free_list.push(slot.0);
        self.active_count -= 1;
        self.payloads[idx].take()
    }

    #[inline]
    pub fn is_alive(&self, slot: ModuleSlot) -> bool {
        let idx = slot.0 as usize;
        idx < self.metas.len() && self.metas[idx].alive
    }

    #[inline]
    pub fn is_executing(&self, slot: ModuleSlot) -> bool {
        let idx = slot.0 as usize;
        idx < self.metas.len() && self.metas[idx].alive && self.metas[idx].executing
    }

    #[inline]
    pub fn sidecar(&self, slot: ModuleSlot) -> ModuleSidecar {
        let idx = slot.0 as usize;
        if idx < self.metas.len() && self.metas[idx].alive {
            self.metas[idx].sidecar
        } else {
            ModuleSidecar::None
        }
    }

    #[inline]
    pub fn set_sidecar(&mut self, slot: ModuleSlot, sidecar: ModuleSidecar) {
        let idx = slot.0 as usize;
        if idx < self.metas.len() && self.metas[idx].alive {
            self.metas[idx].sidecar = sidecar;
        }
    }

    #[inline]
    pub fn push_deferred_op(&mut self, slot: ModuleSlot, op: DeferredSelfOp) -> bool {
        let idx = slot.0 as usize;
        if idx < self.metas.len() && self.metas[idx].alive && self.metas[idx].executing {
            self.metas[idx].deferred_ops.push(op);
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn drain_deferred_ops(&mut self, slot: ModuleSlot) -> Vec<DeferredSelfOp> {
        let idx = slot.0 as usize;
        if idx < self.metas.len() && !self.metas[idx].deferred_ops.is_empty() {
            std::mem::take(&mut self.metas[idx].deferred_ops)
        } else {
            Vec::new()
        }
    }

    #[inline]
    pub fn meta(&self, slot: ModuleSlot) -> &ModuleMeta {
        &self.metas[slot.0 as usize]
    }

    #[inline]
    pub fn meta_mut(&mut self, slot: ModuleSlot) -> &mut ModuleMeta {
        &mut self.metas[slot.0 as usize]
    }

    #[inline]
    pub fn payload(&self, slot: ModuleSlot) -> Option<&K> {
        let idx = slot.0 as usize;
        if idx < self.metas.len() && self.metas[idx].alive {
            self.payloads[idx].as_ref()
        } else {
            None
        }
    }

    #[inline]
    pub fn payload_mut(&mut self, slot: ModuleSlot) -> Option<&mut K> {
        let idx = slot.0 as usize;
        if idx < self.metas.len() && self.metas[idx].alive {
            self.payloads[idx].as_mut()
        } else {
            None
        }
    }

    /// Takes the payload out for Take-and-Restore dispatch; returns `(payload, ctx, generation)`.
    pub fn take_payload(&mut self, slot: ModuleSlot) -> Result<(K, ModuleContext, u32), TakeError> {
        let idx = slot.0 as usize;
        if idx >= self.metas.len() || !self.metas[idx].alive {
            return Err(TakeError::Dead);
        }
        let meta = &mut self.metas[idx];
        if meta.executing {
            return Err(TakeError::Reentered);
        }
        let payload = self.payloads[idx].take().ok_or(TakeError::Reentered)?;
        meta.executing = true;
        Ok((payload, meta.context(slot), meta.generation))
    }

    /// Restores a taken payload if the slot was not freed/reallocated during execution.
    pub fn restore_payload(&mut self, slot: ModuleSlot, generation: u32, payload: K) {
        let idx = slot.0 as usize;
        if idx >= self.metas.len() {
            return;
        }
        let meta = &mut self.metas[idx];
        if meta.alive && meta.executing && meta.generation == generation {
            self.payloads[idx] = Some(payload);
            meta.executing = false;
        }
    }
}

impl<K> ScheduleStore<ModuleSlot> for ModuleArena<K> {
    #[inline]
    fn schedule(&self, handle: ModuleSlot) -> &UpdateModuleScheduleState {
        &self.metas[handle.0 as usize].schedule
    }

    #[inline]
    fn schedule_mut(&mut self, handle: ModuleSlot) -> &mut UpdateModuleScheduleState {
        &mut self.metas[handle.0 as usize].schedule
    }
}

/// Executes `f(&mut payload, world, ctx)` using Take-and-Restore so `world` (objects, scheduler,
/// and all sibling modules in the arena) remains mutably accessible with zero borrow conflicts.
pub fn with_module_dispatch<W, K, R>(
    world: &mut W,
    mod_slot: ModuleSlot,
    arena_mut: fn(&mut W) -> &mut ModuleArena<K>,
    f: impl FnOnce(&mut K, &mut W, ModuleContext) -> R,
) -> Result<R, TakeError> {
    let (mut payload, ctx, gen_id) = arena_mut(world).take_payload(mod_slot)?;
    let out = f(&mut payload, world, ctx);
    arena_mut(world).restore_payload(mod_slot, gen_id, payload);
    Ok(out)
}

/// Executes `f(&mut payload, world, ctx)` using Take-and-Restore and drains any deferred self-ops
/// queued on `mod_slot` before restoring ownership to the arena.
pub fn with_module_dispatch_and_drain<W, K, R>(
    world: &mut W,
    mod_slot: ModuleSlot,
    arena_mut: fn(&mut W) -> &mut ModuleArena<K>,
    f: impl FnOnce(&mut K, &mut W, ModuleContext) -> R,
    mut apply_op: impl FnMut(&mut K, &mut W, ModuleContext, DeferredSelfOp),
) -> Result<R, TakeError> {
    let (mut payload, ctx, gen_id) = arena_mut(world).take_payload(mod_slot)?;
    let out = f(&mut payload, world, ctx);
    loop {
        let ops = {
            let arena = arena_mut(world);
            let idx = mod_slot.0 as usize;
            if idx >= arena.metas.len()
                || !arena.metas[idx].alive
                || !arena.metas[idx].executing
                || arena.metas[idx].generation != gen_id
            {
                Vec::new()
            } else {
                arena.drain_deferred_ops(mod_slot)
            }
        };
        if ops.is_empty() {
            break;
        }
        for op in ops {
            apply_op(&mut payload, world, ctx, op);
        }
    }
    arena_mut(world).restore_payload(mod_slot, gen_id, payload);
    Ok(out)
}

#[derive(Debug, Clone)]
struct ObjectNode<T> {
    id: ObjectId,
    alive: bool,
    generation: u32,
    in_world_list: bool,
    prev_in_world: Option<ObjectSlot>,
    next_in_world: Option<ObjectSlot>,
    data: Option<T>,
}

/// Deterministic slot-indexed object storage with intrusive doubly-linked world list (`m_objList`).
#[derive(Debug, Clone)]
pub struct ObjectArena<T> {
    nodes: Vec<ObjectNode<T>>,
    id_to_slot: BTreeMap<ObjectId, ObjectSlot>,
    free_list: Vec<u32>,
    world_head: Option<ObjectSlot>,
    world_tail: Option<ObjectSlot>,
    active_count: usize,
}

impl<T> Default for ObjectArena<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ObjectArena<T> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            id_to_slot: BTreeMap::new(),
            free_list: Vec::new(),
            world_head: None,
            world_tail: None,
            active_count: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.active_count
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.active_count == 0
    }

    #[inline]
    pub fn slot_by_id(&self, id: ObjectId) -> Option<ObjectSlot> {
        self.id_to_slot.get(&id).copied()
    }

    #[inline]
    pub fn is_alive(&self, slot: ObjectSlot) -> bool {
        let idx = slot.0 as usize;
        idx < self.nodes.len() && self.nodes[idx].alive
    }

    #[inline]
    pub fn generation(&self, slot: ObjectSlot) -> Option<u32> {
        let idx = slot.0 as usize;
        if idx < self.nodes.len() && self.nodes[idx].alive {
            Some(self.nodes[idx].generation)
        } else {
            None
        }
    }

    #[inline]
    pub fn is_slot_current(&self, slot: ObjectSlot, generation: u32) -> bool {
        let idx = slot.0 as usize;
        idx < self.nodes.len()
            && self.nodes[idx].alive
            && self.nodes[idx].generation == generation
    }

    #[inline]
    pub fn is_in_world_list(&self, slot: ObjectSlot) -> bool {
        let idx = slot.0 as usize;
        idx < self.nodes.len() && self.nodes[idx].alive && self.nodes[idx].in_world_list
    }

    #[inline]
    pub fn id_of(&self, slot: ObjectSlot) -> Option<ObjectId> {
        let idx = slot.0 as usize;
        if idx < self.nodes.len() && self.nodes[idx].alive {
            Some(self.nodes[idx].id)
        } else {
            None
        }
    }

    #[inline]
    pub fn get(&self, slot: ObjectSlot) -> Option<&T> {
        let idx = slot.0 as usize;
        if idx < self.nodes.len() && self.nodes[idx].alive {
            self.nodes[idx].data.as_ref()
        } else {
            None
        }
    }

    #[inline]
    pub fn get_checked(&self, slot: ObjectSlot, generation: u32) -> Option<&T> {
        let idx = slot.0 as usize;
        if idx < self.nodes.len()
            && self.nodes[idx].alive
            && self.nodes[idx].generation == generation
        {
            self.nodes[idx].data.as_ref()
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut(&mut self, slot: ObjectSlot) -> Option<&mut T> {
        let idx = slot.0 as usize;
        if idx < self.nodes.len() && self.nodes[idx].alive {
            self.nodes[idx].data.as_mut()
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut_checked(&mut self, slot: ObjectSlot, generation: u32) -> Option<&mut T> {
        let idx = slot.0 as usize;
        if idx < self.nodes.len()
            && self.nodes[idx].alive
            && self.nodes[idx].generation == generation
        {
            self.nodes[idx].data.as_mut()
        } else {
            None
        }
    }

    pub fn spawn(&mut self, id: ObjectId, data: T, link_world: bool) -> ObjectSlot {
        debug_assert!(!self.id_to_slot.contains_key(&id), "duplicate ObjectId {:?}", id);
        let slot = if let Some(reuse_idx) = self.free_list.pop() {
            let idx = reuse_idx as usize;
            let raw_next = self.nodes[idx].generation.wrapping_add(1);
            let next_gen = if raw_next == 0 { 1 } else { raw_next };
            self.nodes[idx] = ObjectNode {
                id,
                alive: true,
                generation: next_gen,
                in_world_list: false,
                prev_in_world: None,
                next_in_world: None,
                data: Some(data),
            };
            ObjectSlot(reuse_idx)
        } else {
            let s = ObjectSlot(self.nodes.len() as u32);
            self.nodes.push(ObjectNode {
                id,
                alive: true,
                generation: 1,
                in_world_list: false,
                prev_in_world: None,
                next_in_world: None,
                data: Some(data),
            });
            s
        };
        self.id_to_slot.insert(id, slot);
        self.active_count += 1;
        if link_world {
            self.link_into_world(slot);
        }
        slot
    }

    /// Links `slot` at the head of `world_head` (`GameLogic::registerObject` -> `obj->prependToList(&m_objList)`).
    pub fn link_into_world(&mut self, slot: ObjectSlot) {
        let idx = slot.0 as usize;
        if idx >= self.nodes.len() || !self.nodes[idx].alive || self.nodes[idx].in_world_list {
            return;
        }
        let prev_head = self.world_head;
        self.nodes[idx].prev_in_world = None;
        self.nodes[idx].next_in_world = prev_head;
        self.nodes[idx].in_world_list = true;
        if let Some(head) = prev_head {
            self.nodes[head.0 as usize].prev_in_world = Some(slot);
        } else {
            self.world_tail = Some(slot);
        }
        self.world_head = Some(slot);
    }

    pub fn unlink_from_world(&mut self, slot: ObjectSlot) {
        let idx = slot.0 as usize;
        if idx >= self.nodes.len() || !self.nodes[idx].alive || !self.nodes[idx].in_world_list {
            return;
        }
        let prev = self.nodes[idx].prev_in_world;
        let next = self.nodes[idx].next_in_world;
        if let Some(p) = prev {
            self.nodes[p.0 as usize].next_in_world = next;
        } else {
            self.world_head = next;
        }
        if let Some(n) = next {
            self.nodes[n.0 as usize].prev_in_world = prev;
        } else {
            self.world_tail = prev;
        }
        self.nodes[idx].prev_in_world = None;
        self.nodes[idx].next_in_world = None;
        self.nodes[idx].in_world_list = false;
    }

    pub fn despawn(&mut self, slot: ObjectSlot) -> Option<T> {
        let idx = slot.0 as usize;
        if idx >= self.nodes.len() || !self.nodes[idx].alive {
            return None;
        }
        self.unlink_from_world(slot);
        let id = self.nodes[idx].id;
        self.id_to_slot.remove(&id);
        self.nodes[idx].alive = false;
        self.nodes[idx].generation = self.nodes[idx].generation.wrapping_add(1);
        self.free_list.push(slot.0);
        self.active_count -= 1;
        self.nodes[idx].data.take()
    }

    /// Returns a deterministic iterator over active world-list [`ObjectSlot`]s in linked order.
    #[inline]
    pub fn world_iter(&self) -> WorldObjectIter<'_, T> {
        WorldObjectIter {
            arena: self,
            next: self.world_head,
        }
    }
}

pub struct WorldObjectIter<'a, T> {
    arena: &'a ObjectArena<T>,
    next: Option<ObjectSlot>,
}

impl<'a, T> Iterator for WorldObjectIter<'a, T> {
    type Item = ObjectSlot;

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.next?;
        self.next = self.arena.nodes[cur.0 as usize].next_in_world;
        Some(cur)
    }
}
