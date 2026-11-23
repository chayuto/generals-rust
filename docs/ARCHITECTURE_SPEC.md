# Interface-Level Architectural Specification (`generals-rust`)

This specification locks down the exact Rust trait contracts, arena borrow-splitting rules, C-ABI FFI headers (`zh_ffi.h`), and Gate V2 Golden-Vector schemas so that `Tier 1A`, `Tier 1B`, `Tier 2`, and `Tier 3` agents (across both `env:ephemeral-web` and `env:local-full`) code against a single frozen interface contract.

---

## 1. Core Simulation Spine & Borrow-Splitting Protocol (`crates/zh_logic`)

### 1.1 The Re-Entrancy & Borrow-Checker Problem in RTS Object Graphs
In C++, `UpdateModule::update()` holds a `this` pointer (`BehaviorModule*`) and `getObject()` (`Object*`), and frequently calls:
- `otherObj->attemptDamage(&damageInfo)` -> which synchronously invokes `BodyModule::attemptDamage`, `DamageModule::onDamage`, and (if health reaches 0) `Object::onDie` -> `DieModule::onDie` -> `SlowDeathBehavior::onDie` -> `TheGameLogic->friend_awakenUpdateModule(...)`!
- In naive Rust, holding `&mut ObjectRecord` or `&mut ModuleRecord` while calling `world.attempt_damage(target_id, ...)` would fail to compile (or panic at runtime with `RefCell::borrow_mut`) whenever an object damages itself (e.g. splash damage, suicide demo-trap, overcharge health drain) or awakens a sibling module on the same object.

### 1.2 Architectural Solution: Slot-Indexed State + Take-and-Restore Module Dispatch

1. **`ObjectRecord` Embeds `Thing` & Never Owns Modules Inline:**
   - `ObjectRecord` lives in `ObjectArena<ObjectRecord>` (`ObjectSlot(u32)`) and embeds `pub thing: Thing` directly (never duplicating `template_id`, `transform`, `position`, or `angle`):
     ```rust
     pub struct ObjectRecord {
         pub id: ObjectId,
         pub thing: Thing, // owns template Override, transform, cached pos/angle, altitude caches
         pub team_id: TeamId,
         pub status: ObjectStatusMask,
         pub private_status: u8,
         pub geometry: GeometryInfo,
         pub disabled_mask: DisabledMask,
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
         // Modules in exact C++ order (conditional sleep helpers, then INI behaviors)
         pub behaviors: Vec<(ModuleSlot, FacetMask)>,
         pub body_slot: Option<ModuleSlot>,
         pub contain_slot: Option<ModuleSlot>,
         pub stealth_slot: Option<ModuleSlot>,
         pub ai_slot: Option<ModuleSlot>,
         pub physics_slot: Option<ModuleSlot>,
     }
     ```
   - **`ObjectArena` & Destruction Invariants:**
     - `link_into_world(slot)` **must prepend to `world_head`** (newest-first, matching `GameLogic::registerObject` -> `obj->prependToList(&m_objList)`), preserving `MARKER:Objects` CRC and Step 15f ordering.
     - `ObjectNode` tracks `generation: u32` (and `objects_to_destroy` validates `(ObjectSlot, generation)` or resolves via `ObjectId`) to prevent ABA reuse bugs.
     - `destroy_object(slot)` dispatches `FacetMask::DESTROY`, sets `OBJECT_STATUS_DESTROYED`, clears `AIUpdate` paths, pushes to `objects_to_destroy`, and runs `on_destroy` + wall pathfinder cleanup while leaving the object in `m_objList`/`id_to_slot` until Step 15a.
     - Step 15a `process_destroy_list()` loops `let mut i = 0; while i < queue.len()` (never fixed `drain`) so child objects enqueued by `on_delete()` are destroyed in the same frame; per slot it invokes `on_delete()`, runs a **two-pass reverse-heap-index erase** on `SleepyUpdateHeap` (`erase_for_owner`), frees each `ModuleSlot` in `ModuleArena`, and calls `objects.despawn(slot)`.
2. **`ModuleArena` Uses Temporary Take-and-Restore (`Option<Box<dyn BehaviorModule>>`) During Callbacks:**
   - When `SimWorld` invokes a callback on `mod_slot: ModuleSlot` (such as `update`, `on_die`, `on_damage`, `on_collide`, `on_create`, `on_upgrade`), `ModuleArena::take_payload(mod_slot)` returns `Result<(Box<dyn BehaviorModule>, ModuleContext, u32), TakeError>`, passes `(&mut *module, &mut sim, ctx)` into the handler, drains `meta.deferred_ops`, and calls `restore_payload(mod_slot, generation, module)` on return:
     ```rust
     pub struct ModuleContext {
         pub mod_slot: ModuleSlot,
         pub owner_slot: ObjectSlot,
         pub owner_id: ObjectId,
         pub tag_key: NameKeyType,
         pub template_data_id: ModuleDataId,
     }

     pub fn with_module_dispatch<W, R>(
         world: &mut W,
         mod_slot: ModuleSlot,
         arena_mut: fn(&mut W) -> &mut ModuleArena<Box<dyn BehaviorModule>>,
         f: impl FnOnce(&mut dyn BehaviorModule, &mut W, ModuleContext) -> Result<R, SimError>,
     ) -> Result<R, DispatchError> { ... }
     ```
   - **Why This Guarantees Zero Borrow Conflicts (`DET010`-Safe):**
     - While `module.update(sim, ctx)` runs, `world.objects` (including `world.objects[ctx.owner_slot]`), `world.sleepy_updates`, `world.partition`, `world.players`, `world.ai`, `world.rng`, and **all sibling modules in `world.modules`** remain mutably accessible!
     - `ModuleMeta` (`schedule`, `sidecar`, `deferred_ops`) lives in a parallel `metas: Vec<ModuleMeta>` vector completely disjoint from the taken `payloads: Vec<Option<Box<dyn BehaviorModule>>>`.

### 1.3 Same-Slot Re-Entrancy Protocol (`DET010`-Safe)

Synchronous SAGE callback chains routinely re-enter the *same* executing module slot (`Body` -> `onDamage`/`onDie` -> `Body`, `Contain` -> passenger hooks -> `Contain`, `AIUpdate` -> splash `onDamage` -> `AIUpdate`, `PhysicsBehavior` -> `onCollide` -> `PhysicsBehavior`). Returning `Option::None` would silently drop these chains and desync replays. Instead:

1. **Hot Sidecar Mirror on `ModuleMeta`:** Before every outward callback (`onDamage`, `onDie`, `onCollide`, passenger hooks), the executing module syncs hot read-only fields into `ModuleMeta::sidecar`. Synchronous re-entrant readers (and `Object::crc()`) read `meta.sidecar` even while `meta.executing == true`.
2. **Deferred Self-Mutation Queue + Immediate Sidecar Sync + Loud `TakeError`:**
   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub enum TakeError { Dead, Reentered }

   #[derive(Debug, Clone, Copy, Default)]
   pub enum ModuleSidecar {
       #[default]
       None,
       Body {
           health: Real, max_health: Real, prev_health: Real, damage_scalar: Real,
           damage_state: u8, front_crushed: Bool, back_crushed: Bool, indestructible: Bool,
       },
       Contain { count: u32, max: i32, enclosing: Bool },
       Physics { velocity: Coord3D, mass: Real, airborne: Bool },
       Ai { moving: Bool, attacking: Bool, idle: Bool, cur_locomotor_set: u8 },
   }

   #[derive(Debug, Clone, Copy)]
   pub enum DeferredSelfOp {
       ApplyForce(Coord3D),
       ChangeHealth { delta: Real, change_type: u8 },
       AiSetIdle,
       ContainRemove(ObjectSlot),
   }
   // Added to ModuleMeta:
   // pub sidecar: ModuleSidecar,
   // pub deferred_ops: Vec<DeferredSelfOp>,
   ```
   - Known re-entrant writes targeting an `executing == true` slot append a `DeferredSelfOp` onto `meta.deferred_ops` **and immediately update `meta.sidecar` in the same call** so subsequent synchronous reads in the same callback chain observe post-write state (e.g., health/damage-state after `ChangeHealth`) before `restore_payload` drains `deferred_ops`.
   - Unexpected re-entrant `take_payload` calls return `Err(TakeError::Reentered)` (panicking in debug/tests instead of silently no-oping).

### 1.4 Scalable 225-Module Dispatch & `ModuleDataStore`

To allow parallel module porting across `225` `BehaviorModule` subclasses without central enum merge conflicts or large `Copy`/move overhead:

1. **Trait-Object Strategy (`Box<dyn BehaviorModule>` + Typed Facet Sub-Traits + Single Canonical 64-bit `FacetMask`):**
   - Each module lives in its own file implementing `BehaviorModule` plus only its supported facet sub-traits (`UpdateFacet`, `BodyFacet`, `DieFacet`, `DamageFacet`, `CreateFacet`, `CollideFacet`, `ContainFacet`, `UpgradeFacet`, `SpecialPowerFacet`, `DestroyFacet`, `AiUpdateFacet`, `PhysicsFacet`, `StealthFacet`, etc.).
   - Modules register inside per-category registrar files (`modules/{body,die,update,contain,...}/mod.rs`), never in a single monolithic `enum`.
   - `FacetMask` in `ObjectRecord::behaviors` and `ModuleMeta` pre-filters dispatch in one bitmask test; one canonical 64-bit layout is frozen across `arena::FacetMask` and `zh_ffi.h`.
   ```rust
   pub trait BehaviorModule: Send {
       fn module_name_key(&self) -> NameKeyType;
       fn on_object_created(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
       fn on_delete(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
       fn on_capture(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _old: i32, _new: i32) {}
       fn on_disabled_edge(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _disabled: Bool) {}
       fn crc(&self, _xfer: &mut dyn Xfer) {}
       fn xfer(&mut self, _xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError>;
       fn load_post_process(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
       // Typed facet downcasts (default None; override only facets owned by module)
       fn as_update(&self) -> Option<&dyn UpdateFacet> { None }
       fn as_update_mut(&mut self) -> Option<&mut dyn UpdateFacet> { None }
       fn as_body(&self) -> Option<&dyn BodyFacet> { None }
       fn as_body_mut(&mut self) -> Option<&mut dyn BodyFacet> { None }
       fn as_die_mut(&mut self) -> Option<&mut dyn DieFacet> { None }
       fn as_damage_mut(&mut self) -> Option<&mut dyn DamageFacet> { None }
       fn as_create_mut(&mut self) -> Option<&mut dyn CreateFacet> { None }
       fn as_collide_mut(&mut self) -> Option<&mut dyn CollideFacet> { None }
       fn as_contain(&self) -> Option<&dyn ContainFacet> { None }
       fn as_contain_mut(&mut self) -> Option<&mut dyn ContainFacet> { None }
       fn as_upgrade_mut(&mut self) -> Option<&mut dyn UpgradeFacet> { None }
       fn as_special_power_mut(&mut self) -> Option<&mut dyn SpecialPowerFacet> { None }
       fn as_destroy_mut(&mut self) -> Option<&mut dyn DestroyFacet> { None }
       fn as_ai(&self) -> Option<&dyn AiUpdateFacet> { None }
       fn as_ai_mut(&mut self) -> Option<&mut dyn AiUpdateFacet> { None }
       fn as_physics_mut(&mut self) -> Option<&mut dyn PhysicsFacet> { None }
       fn as_stealth_mut(&mut self) -> Option<&mut dyn StealthFacet> { None }
   }

   pub trait UpdateFacet {
       fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime;
       fn disabled_types_to_process(&self) -> u64 { 0 }
   }
   ```
2. **Typed `ModuleData` Storage & `ThingTemplate` Linkage:**
   ```rust
   pub trait ModuleData: std::any::Any + Send + Sync {
       fn as_any(&self) -> &dyn std::any::Any;
       fn tag_key(&self) -> NameKeyType;
       fn module_name_key(&self) -> NameKeyType;
       fn facet_mask(&self) -> FacetMask;
       fn update_phase(&self) -> SleepyUpdatePhase { SleepyUpdatePhase::Normal }
       fn is_ai_module(&self) -> bool { false }
   }

   pub struct ModuleDataStore {
       entries: Vec<Box<dyn ModuleData>>, // 1-based ModuleDataId (0 = INVALID)
       base_len: usize,                   // Truncated here on ThingFactory::reset()
       by_template_and_tag: BTreeMap<(ThingTemplateId, NameKeyType), ModuleDataId>,
   }

   pub struct ModuleRegistry {
       by_name: BTreeMap<String, usize>, // ASCII-lowercase normalized
       by_key: BTreeMap<NameKeyType, usize>,
       descriptors: Vec<ModuleDescriptor>,
   }
   ```
   - Add `pub tag_key: NameKeyType` and `pub data_id: ModuleDataId` to `ModuleNugget`, replace `ini.skip_block()` with `ModuleRegistry` parsing into `ModuleDataStore` (skip+stub only for unported modules during bring-up), and implement `ModuleMaskLookup` directly on `ModuleRegistry`.
   - In Pattern-3 FFI (`zh_module_create`), C++ passes `(template_id: u16, tag_key: i32)` so Rust resolves the already-parsed `ModuleDataId` from `by_template_and_tag`—eliminating 225 C++ `ModuleData` FFI layout mirrors.

### 1.5 `SimContext` / `SimWorld` / `SimView` + `SimApi` & Verified Pre-`WP-5.0` Trap Checklist

```rust
pub struct SimContext {
    pub rules: RulesRegistry,        // Immutable during match; reset() truncates map overrides
    pub world: SimWorld,             // Mutable per-match state hashed by get_crc()
    pub client: Box<dyn ClientSink>, // No-op in headless; renderer sink later
}

pub trait SimView {
    fn frame(&self) -> UnsignedInt;
    fn rules(&self) -> &RulesRegistry;
    fn module_data(&self, id: ModuleDataId) -> &dyn ModuleData;
    fn find_object(&self, id: ObjectId) -> Option<ObjectSlot>;
    fn object(&self, slot: ObjectSlot) -> Option<&ObjectRecord>;
    fn module_sidecar(&self, slot: ModuleSlot) -> ModuleSidecar;
    fn terrain(&self) -> &dyn ThingTerrainAccess;
}

pub trait SimApi: SimView {
    fn rng_logic_int(&mut self, lo: Int, hi: Int) -> Int;
    fn rng_logic_real(&mut self, lo: Real, hi: Real) -> Real;
    fn object_mut(&mut self, slot: ObjectSlot) -> Option<&mut ObjectRecord>;
    fn set_module_sidecar(&mut self, slot: ModuleSlot, sidecar: ModuleSidecar);
    fn awaken_module(&mut self, slot: ModuleSlot, when: UnsignedInt) -> bool;
    fn set_wake_frame(&mut self, slot: ModuleSlot, sleep: UpdateSleepTime);
    fn spawn_object(&mut self, tmpl: ThingTemplateId, team: TeamId, pos: Coord3D, angle: Real) -> ObjectSlot;
    fn destroy_object(&mut self, slot: ObjectSlot);
    fn attempt_damage(&mut self, target: ObjectSlot, dmg: &mut DamageInfo);
    fn client(&mut self) -> &mut dyn ClientSink;
}
```
- **3-Mode Portability:** Modules compile once against `&mut dyn SimApi` and run unchanged under `MockSimApi` (Gate V2), `FullSimApi` (`GameLogic::update()`), and `FfiSimApi` (Pattern-3 C++ bridge).

#### Verified Pre-`WP-5.0` Fix Checklist (Audited Against `vendor/GeneralsX` @ `84a7a20`)
1. `crates/zh_logic/src/arena/mod.rs` (`L447-L462`): Fix `ObjectArena::link_into_world` to prepend to `world_head` (`GameLogic.cpp:4239`) + add `generation: u32` to `ObjectNode`.
2. `crates/zh_logic/src/scheduler/sleepy_heap.rs` (`L447-L464`): Fix `finish_due_update_at_root` to track `let handle = self.cur_update_module.take()?` instead of re-peeking root `0` (verify `rebalance(idx)` vs C++ `GameLogic.cpp:4108-4109` `rebalanceSleepyUpdate(0)` with a Gate V2 heap golden test) + add two-pass reverse-heap-index `erase_for_owner`.
3. `crates/zh_logic/src/arena/mod.rs` (`L51-L83`): Freeze one canonical 64-bit `FacetMask` layout shared with `zh_ffi.h`.
4. `crates/zh_logic/src/object/thing_template.rs` (`L336`, `L1270`, `L1451`): Wire `Locomotor` parsing, replace `ini.skip_block()` with `ModuleRegistry` + `ModuleDataStore`, and replace `DefaultModuleMaskLookup`.
5. `crates/zh_logic/src/object/thing_factory.rs`: Keep `by_name` strictly case-sensitive to match C++ `ThingTemplateHashMap` (`ThingFactory.h:44`, `STLTypedefs.h:224`, `ThingFactory.cpp:303`); note `ObjectRecord::behaviors` and `ModuleMeta::deferred_ops` use `Vec` (zero external dependencies) and `ObjectRecord::upgrades_completed` uses `UpgradeMaskType`.
6. `scripts/lint_determinism.py` (`L48-L66`): Never name `SimApi`/`BehaviorModule` methods `.log`, `.round`, `.floor`, `.ceil`, `.trunc`, or `.exp` (`DET003`), and never key `BTreeMap` by raw pointers across FFI (`DET005`; pass `ObjectId` integers instead).

---

## 2. Exact C-ABI Contracts (`crates/zh_ffi/include/zh_ffi.h`)

### 2.1 Pattern 1: `PKT-001` (`crc`) & `PKT-002` (`RandomValue`) C-ABI Contract

```c
// crates/zh_ffi/include/zh_ffi.h (Pattern 1: PKT-001 & PKT-002)
#pragma once
#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// --- PKT-001: Byte-wise CRC (Core/GameEngine/Include/Common/crc.h) ---
uint32_t zh_crc_compute_bytes(uint32_t initial_crc, const uint8_t* buf, int32_t len);

// --- PKT-002: RandomValue (Core/GameEngine/Include/Common/RandomValue.h) ---
void     zh_random_init_time(void);
void     zh_random_init_seed(uint32_t seed);
uint32_t zh_random_get_logic_base_seed(void);
uint32_t zh_random_get_logic_seed_crc(void);

int32_t  zh_random_get_logic_int(int32_t lo, int32_t hi);
float    zh_random_get_logic_real(float lo, float hi);
int32_t  zh_random_get_logic_int_unchanged(int32_t lo, int32_t hi);
float    zh_random_get_logic_real_unchanged(float lo, float hi);

int32_t  zh_random_get_client_int(int32_t lo, int32_t hi);
float    zh_random_get_client_real(float lo, float hi);
int32_t  zh_random_get_audio_int(int32_t lo, int32_t hi);
float    zh_random_get_audio_real(float lo, float hi);

// Direct state inspection for Gate V2 differential harness & shadow-verify
void     zh_random_get_logic_state_words(uint32_t out_words[6]);
void     zh_random_set_logic_state_words(const uint32_t in_words[6], uint32_t base_seed);

#ifdef __cplusplus
}
#endif
```

---

### 2.2 Pattern 2: `PKT-003` (`ExperienceTracker`) C-ABI Contract with Parent Callback VTable

To allow `ExperienceTracker` (`PKT-003`) to run in pure Rust inside `crates/zh_logic` while `Object` is still in C++ (during Phases 1–4) **and** to run in standalone pure-Rust tests (`env:ephemeral-web` with a mock vtable):

```c
// crates/zh_ffi/include/zh_ffi.h (Pattern 2: PKT-003 ExperienceTracker)
typedef struct ZhExpTrackerState {
    int32_t  current_level;       // VeterancyLevel (0..3)
    int32_t  current_experience;  // Int
    int32_t  experience_sink;     // ObjectID (INVALID_ID = 0)
    float    experience_scalar;   // Real (1.0f)
    uint8_t  is_trainable;        // Bool (0 or 1)
} ZhExpTrackerState;

// Callback bridge supplied by C++ Object (or by pure-Rust test mock in Gate V2-Golden)
typedef struct ZhExpTrackerParentCallbacks {
    void* parent_ctx;
    uint8_t (*is_template_trainable)(void* parent_ctx);
    int32_t (*get_experience_required)(void* parent_ctx, int32_t level);
    int32_t (*get_experience_value)(void* parent_ctx, int32_t level);
    int32_t (*get_relationship_with_killer)(void* parent_ctx, const void* killer_ctx);
    void    (*on_veterancy_level_changed)(void* parent_ctx, int32_t old_level, int32_t new_level, uint8_t provide_feedback);
    uint8_t (*forward_experience_to_sink)(void* parent_ctx, int32_t sink_obj_id, int32_t scaled_exp_gain, uint8_t can_scale_for_bonus);
    uint8_t (*forward_set_exp_and_level_to_sink)(void* parent_ctx, int32_t sink_obj_id, int32_t exp_in, uint8_t provide_feedback);
} ZhExpTrackerParentCallbacks;

void    zh_exp_tracker_init(ZhExpTrackerState* state, const ZhExpTrackerParentCallbacks* cb);
int32_t zh_exp_tracker_get_experience_value(const ZhExpTrackerState* state, const ZhExpTrackerParentCallbacks* cb, const void* killer_ctx);
uint8_t zh_exp_tracker_is_accepting_exp(const ZhExpTrackerState* state);
void    zh_exp_tracker_set_min_veterancy_level(ZhExpTrackerState* state, const ZhExpTrackerParentCallbacks* cb, int32_t new_level, uint8_t provide_feedback);
void    zh_exp_tracker_set_veterancy_level(ZhExpTrackerState* state, const ZhExpTrackerParentCallbacks* cb, int32_t new_level, uint8_t provide_feedback);
uint8_t zh_exp_tracker_gain_exp_for_level(ZhExpTrackerState* state, const ZhExpTrackerParentCallbacks* cb, int32_t levels_to_gain, uint8_t can_scale_for_bonus);
uint8_t zh_exp_tracker_can_gain_exp_for_level(const ZhExpTrackerState* state, int32_t levels_to_gain);
void    zh_exp_tracker_add_experience_points(ZhExpTrackerState* state, const ZhExpTrackerParentCallbacks* cb, int32_t experience_gain, uint8_t can_scale_for_bonus);
void    zh_exp_tracker_set_experience_and_level(ZhExpTrackerState* state, const ZhExpTrackerParentCallbacks* cb, int32_t experience_in, uint8_t provide_feedback);
```

**Why This VTable Decoupling Is Crucial for `env:ephemeral-web` Agents:**
Because `ZhExpTrackerState` interacts with `Object` exclusively through `ZhExpTrackerParentCallbacks` (in Rust: `trait ExperienceParentAccess`), `crates/zh_logic/src/object/experience_tracker.rs` has **zero dependency on C++ headers or raw C++ pointers**!
- In `cargo test` (inside an `env:ephemeral-web` container), `MockExperienceParent` implements `ExperienceParentAccess` in 20 lines of pure Rust.
- In `crates/zh_ffi` (linked into `GeneralsXZH`), `FfiExperienceParent` wraps `ZhExpTrackerParentCallbacks` and forwards to C++ `Object`!

---

### 2.3 Pattern 3: Universal `RustBehaviorModuleAdapter` C-ABI Contract (Phase 5, All 222 Modules)

In Phase 5 (`WP-5.0`), a single C++ adapter class `RustBehaviorModuleAdapter` in `z_gameengine` bridges all 222 `BehaviorModule` subclasses to `crates/zh_logic::ModuleArena` via a 32-bit `ModuleSlot`:

```c
// crates/zh_ffi/include/zh_ffi.h (Pattern 3: Universal BehaviorModule Bridge)
typedef uint32_t ZhModuleSlot;
typedef uint64_t ZhFacetMask;

// Lifecycle & Factory (pass ObjectId + (template_id, tag_key) — never void* BTreeMap keys or C++ ModuleData mirrors)
uint8_t      zh_module_is_ported(int32_t module_name_key);
ZhModuleSlot zh_module_create(
    int32_t module_name_key,
    uint32_t owner_obj_id,
    uint16_t template_id,
    int32_t tag_key,
    ZhFacetMask* out_facet_mask,
    uint32_t* out_initial_priority);
void         zh_module_on_object_created(ZhModuleSlot slot);
void         zh_module_on_delete(ZhModuleSlot slot);
void         zh_module_destroy(ZhModuleSlot slot);

// Primary Facet Dispatch
uint32_t     zh_module_update(ZhModuleSlot slot); // returns UpdateSleepTime (1..0x3fffffff)
uint64_t     zh_module_get_disabled_types_to_process(ZhModuleSlot slot);
void         zh_module_on_die(ZhModuleSlot slot, const void* damage_info_ptr);
void         zh_module_on_damage(ZhModuleSlot slot, void* damage_info_ptr);
void         zh_module_on_create(ZhModuleSlot slot);
void         zh_module_on_build_complete(ZhModuleSlot slot);
void         zh_module_on_collide(ZhModuleSlot slot, uint32_t other_obj_id, const void* loc_ptr, const void* normal_ptr);
void         zh_module_on_destroy(ZhModuleSlot slot);
void         zh_module_on_capture(ZhModuleSlot slot, int32_t old_player_idx, int32_t new_player_idx);
void         zh_module_on_disabled_edge(ZhModuleSlot slot, uint8_t now_disabled);

// Snapshot Serialization & CRC
void         zh_module_crc(ZhModuleSlot slot, void* cxx_xfer_ptr);
void         zh_module_xfer(ZhModuleSlot slot, void* cxx_xfer_ptr);
void         zh_module_load_post_process(ZhModuleSlot slot);
```

---

## 3. Gate V2 Golden-Vector Schema (`docs/golden_vectors/<PKT-ID>.json`)

To uphold **"Never build something you cannot verify"** for `env:ephemeral-web` agents (which lack a local C++ build and retail `.big` files), every packet `PKT-XXX` has a deterministic Golden-Vector test suite verifiable via pure-Rust `cargo test`:

```json
{
  "packet_id": "PKT-003",
  "module_name": "ExperienceTracker",
  "reference_commit": "84a7a20769aa5749f388b1f49bd542c79b3c33e8",
  "regime": "USE_DETERMINISTIC_MATH=1,RETAIL_COMPATIBLE_CRC=0",
  "vectors": [
    {
      "case_id": "exp_float_scale_truncation_001",
      "initial_state": {
        "current_level": 0,
        "current_experience": 40,
        "experience_sink": 0,
        "experience_scalar": 1.25,
        "is_trainable": true,
        "template_thresholds": [0, 50, 100, 200]
      },
      "operation": {
        "method": "add_experience_points",
        "args": { "experience_gain": 9, "can_scale_for_bonus": true }
      },
      "expected_post_state": {
        "current_level": 1,
        "current_experience": 51,
        "level_changed_callback_fired": true,
        "old_level": 0,
        "new_level": 1
      },
      "expected_xfer_crc_u32": "0x000000CF",
      "expected_xfer_save_hex": "020100000033000000000000000000a03f01"
    }
  ]
}
```

### How Golden Vectors Bridge `env:local-full` and `env:ephemeral-web`:
1. **Verified Against C++ Reference:** Notice in `exp_float_scale_truncation_001` above:
   - `experience_gain = 9`, `experience_scalar = 1.25f32`.
   - `(9 as f32) * 1.25f32 = 11.25f32`, truncated toward zero to `i32` = `11`.
   - `40 + 11 = 51 >= 50` (`LEVEL_VETERAN = 1` threshold), promoting `current_level` from `0` to `1`!
   - `ExperienceTracker::crc` hashes:
     - `m_currentExperience = 51` (`0x00000033`): `crc = 0u32.rotate_left(1).wrapping_add(51) = 51`
     - `m_currentLevel = 1` (`0x00000001`): `crc = 51u32.rotate_left(1).wrapping_add(1) = 103`
     - `m_isTrainable = 1` (`0x01`, 1 trailing byte): `crc = 103u32.rotate_left(1).wrapping_add(1) = 207 = 0x000000CF`!
2. **Instant Pure-Rust Verification:** When an `env:ephemeral-web` agent runs `cargo test`, the Golden Vector harness feeds each case's `initial_state` and `operation` into the translated Rust module and asserts exact match on `expected_post_state`, `expected_xfer_crc_u32`, and `expected_xfer_save_hex`!
