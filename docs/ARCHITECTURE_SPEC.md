# Interface-Level Architectural Specification (`generals-rust`)

This specification locks down the exact Rust trait contracts, arena borrow-splitting rules, C-ABI FFI headers (`zh_ffi.h`), and Gate V2 Golden-Vector schemas so that `Tier 1A`, `Tier 1B`, `Tier 2`, and `Tier 3` agents (across both `env:ephemeral-web` and `env:local-full`) code against a single frozen interface contract.

---

## 1. Core Simulation Spine & Borrow-Splitting Protocol (`crates/zh_logic`)

### 1.1 The Re-Entrancy & Borrow-Checker Problem in RTS Object Graphs
In C++, `UpdateModule::update()` holds a `this` pointer (`BehaviorModule*`) and `getObject()` (`Object*`), and frequently calls:
- `otherObj->attemptDamage(&damageInfo)` -> which synchronously invokes `BodyModule::attemptDamage`, `DamageModule::onDamage`, and (if health reaches 0) `Object::onDie` -> `DieModule::onDie` -> `SlowDeathBehavior::onDie` -> `TheGameLogic->friend_awakenUpdateModule(...)`!
- In naive Rust, holding `&mut ObjectRecord` or `&mut ModuleRecord` while calling `world.attempt_damage(target_id, ...)` would fail to compile (or panic at runtime with `RefCell::borrow_mut`) whenever an object damages itself (e.g. splash damage, suicide demo-trap, overcharge health drain) or awakens a sibling module on the same object.

### 1.2 Architectural Solution: Slot-Indexed State + Take-and-Restore Module Dispatch

1. **`ObjectRecord` Never Owns `ModuleRecord` Inline:**
   - `ObjectRecord` lives in `ObjectArena` (`ObjectSlot(u32)`) and holds a compact list of module handles:
     ```rust
     pub struct ObjectRecord {
         pub id: ObjectId,
         pub template_id: ThingTemplateId,
         pub team_id: TeamId,
         pub prev_in_world: Option<ObjectSlot>,
         pub next_in_world: Option<ObjectSlot>,
         pub status: ObjectStatusMask,
         pub private_status: u8,
         pub transform: Matrix3D,
         pub position: Coord3D,
         pub angle: f32,
         pub geometry: GeometryInfo,
         pub disabled_mask: DisabledMask,
         pub disabled_till_frame: [u32; DISABLED_COUNT],
         pub upgrades_completed: BitFlags128,
         pub weapon_bonus_condition: u32,
         pub experience_tracker: ExperienceTrackerState,
         pub behaviors: SmallVec<[(ModuleSlot, FacetMask); 16]>,
         pub body_slot: Option<ModuleSlot>,
         pub contain_slot: Option<ModuleSlot>,
         pub stealth_slot: Option<ModuleSlot>,
         pub ai_slot: Option<ModuleSlot>,
         pub physics_slot: Option<ModuleSlot>,
     }
     ```
2. **`ModuleArena` Uses Temporary Take-and-Restore (`ModuleKind::ExecutingPlaceholder`) During Callbacks:**
   - When `SimWorld` invokes a callback on `mod_slot: ModuleSlot` (such as `update`, `on_die`, `on_damage`, `on_collide`, `on_create`, `on_upgrade`), it swaps `modules[mod_slot].kind` with `ModuleKind::ExecutingPlaceholder`, passes `(&mut module_kind, &mut SimWorld, ModuleContext { mod_slot, owner_slot, tag_key, template_data_id })` into the handler, and restores `modules[mod_slot].kind` on return:
     ```rust
     pub struct ModuleContext {
         pub mod_slot: ModuleSlot,
         pub owner_slot: ObjectSlot,
         pub owner_id: ObjectId,
         pub tag_key: NameKeyType,
         pub template_data_id: ModuleDataId,
     }

     impl SimWorld {
         pub fn with_module_dispatch<R>(
             &mut self,
             mod_slot: ModuleSlot,
             f: impl FnOnce(&mut ModuleKind, &mut SimWorld, ModuleContext) -> R,
         ) -> R {
             let meta = self.modules.meta(mod_slot);
             let mut kind = self.modules.take_kind(mod_slot);
             let res = f(&mut kind, self, meta);
             self.modules.restore_kind(mod_slot, kind);
             res
         }
     }
     ```
   - **Why This Guarantees Zero Borrow Conflicts:**
     - While `kind.update(world, ctx)` is running, `world.objects` (including `world.objects[ctx.owner_slot]`), `world.sleepy_updates`, `world.partition`, `world.players`, `world.ai`, `world.rng`, and **every other module in `world.modules`** are 100% accessible via `&mut SimWorld`!
     - Even if `kind.update` calls `ctx.set_wake_frame(world, delay)`, `world.modules.meta_mut(mod_slot)` (`next_call_frame_and_phase` and `index_in_logic`) remains in the metadata header array, completely disjoint from the taken `ModuleKind` payload!

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

// Lifecycle & Factory
uint8_t      zh_module_is_ported(int32_t module_name_key);
ZhModuleSlot zh_module_create(int32_t module_name_key, void* cxx_object_ptr, const void* cxx_module_data_ptr, ZhFacetMask* out_facet_mask, uint32_t* out_initial_priority);
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
void         zh_module_on_collide(ZhModuleSlot slot, void* other_obj_ptr, const void* loc_ptr, const void* normal_ptr);
void         zh_module_on_destroy(ZhModuleSlot slot);
void         zh_module_on_capture(ZhModuleSlot slot, void* old_owner_ptr, void* new_owner_ptr);
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
      "expected_xfer_crc_u32": "0x000000CD",
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
