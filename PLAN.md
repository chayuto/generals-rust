# Architectural & Execution Plan: Zero Hour Engine Port to Rust (`generals-rust`)

## 1. Objective & Milestones

Port the game engine of *Zero Hour* (~330k lines of C++ across `GameLogic` and `Common` in 871 files) to Rust as a clean, idiomatic, GPL-3.0-or-later public portfolio project, verified bit-exact against real game replays via the engine's built-in per-frame state checksums (`XferCRC`).

- **Stage 1 (Primary Milestone):** Headless Rust simulation that executes real *Zero Hour* replays at 30 logic frames/second (`LOGICFRAMES_PER_SECOND = 30`) and produces the exact same per-frame state checksums (`GameLogic::getCRC`) as the reference C++ build (`fbraz3/GeneralsX` at `84a7a20769aa5749f388b1f49bd542c79b3c33e8`).
- **Stage 2 (Subsequent Milestone):** Native client and renderer (`wgpu`/Metal on macOS Apple Silicon) replacing `GameClient` and `WW3D2`.

---

## 2. Project Identity, Naming & Governance Rules

1. **Repository & Crate Naming (EA GPLv3 Section 7 Compliance):**
   - No `"Command & Conquer"`, `"C&C"`, or `"cnc"` in the repository name, Cargo workspace/crate names, or binary names.
   - **GitHub Repository:** `chayuto/generals-rust` (started **private**; goes public only on owner instruction).
   - **Local Workspace Folder:** `/Users/chayut/repos/generals-rust`.
   - **Cargo Workspace Layout:**
     - `crates/zh_common` — deterministic math (`GameMath` bindings/port), PRNG (`RandomValue`), `CRC` / `XferCRC`, `BitFlags<N>`, `NameKeyTable`, `INI` parsers, `Dict`, coordinate/matrix types (`Coord3D`, `Matrix3D`, `GeometryInfo`).
     - `crates/zh_logic` — `SimContext`, `ObjectArena`, `ModuleArena` (all 222 object modules), `SleepyUpdateHeap`, `PartitionGrid`, `TerrainLogic`, `PlayerStore`, `AiSystem`, `Pathfinder`, `ScriptSystem`, `Weapon`, `Locomotor`.
     - `crates/zh_ffi` — `staticlib` C-ABI strangler bridge (`libzh_ffi.a`) linked into `z_gameengine` during Phases 1–6.
     - `crates/zh_sim` — standalone pure-Rust headless replay runner binary (`zh-sim`) for Stage 1 completion (Phase 7).
2. **License:** `GPL-3.0-or-later` with Electronic Arts Section 7 Additional Terms and an explicit Notice of Modification in `LICENSE.md`.
3. **Complete Independence:** Zero code, crates, submodules, or packages from any other repository owned by the user (`cnc-gen-rig`, `cnc-gen-engine`, `rust-ra2-arena`, `ra2-arena-engine`, `civrig`, `rignarok`).
4. **Zero Game Assets in Git:** `.gitignore` and `.githooks/pre-commit` block `.big`, `.bik`, `.map`, `.wak`, `.tga`, `.dds`, `.w3d`, `.csf`, `.str`, `.mp3`, `.wav`, `.exe`, `.dll`, and archive formats. Only self-recorded `.rep` test replays under `replays/` are permitted in git.
5. **No Upstream Contact:** Zero PRs, issues, or comments to `fbraz3/GeneralsX` or `TheSuperHackers/GeneralsGameCode`.
6. **Commit Hygiene:**
   - Git `user.email` pinned to `5305832+chayuto@users.noreply.github.com`.
   - `.githooks/pre-commit` and `.githooks/commit-msg` active via `git config core.hooksPath .githooks`.
   - Zero co-author trailers, generator notices, session links, or assistant names anywhere in commits, PRs, code, or documentation.
7. **Desktop Courtesy:** All builds and replay checks run strictly headless (`-headless`, `SDL_VIDEODRIVER=dummy`, `SDL_AUDIODRIVER=dummy`). Never focus, resize, or `open` GUI windows.
8. **Continuous Documentation:** Record findings, benchmark timings, and desync root causes into `docs/` and commit before advancing to the next step.

---

## 3. Target Rust Architecture & Domain Model

The original C++ codebase uses ~71 global singletons (`The*`), 352 manual `new`/`delete` calls (plus `MemoryPoolObject` intrusive free-lists), deep multiple-inheritance hierarchies across 222 `BehaviorModule` classes, and cyclic raw pointers (`Object*`, `Team*`, `Player*`, `AIGroup*`, `UpdateModule*`).

Below is the target Rust architecture designed to replace those patterns with safe, deterministic arena ownership while preserving 100% bit-exact execution and CRC order.

```mermaid
flowchart TD
    subgraph SimContext["SimContext (Top-Level Engine State)"]
        Rules["Arc&lt;RulesRegistry&gt;\n(Immutable INI Templates & Stores)"]
        subgraph SimWorld["SimWorld (Mutable Per-Match State / Snapshot)"]
            RNG["LogicRngState\n([u32; 6] + Seed)"]
            Cmd["CommandList & Recorder\n(Frame GameMessages & CRC Queue)"]
            ObjArena["ObjectArena\n(Dense Slots + Intrusive m_objList Order)"]
            ModArena["ModuleArena\n(222 BehaviorModule Variants + Facet Masks)"]
            Sleepy["SleepyUpdateHeap\n(Custom Min-Heap: (frame &lt;&lt; 2) | phase)"]
            Part["PartitionGrid\n(Spatial Cells, Shroud/Sight, Threat/Value)"]
            Terrain["TerrainLogic\n(Heightmap, Bridges, Triggers, Waypoints)"]
            Players["PlayerStore & Teams\n(Money, Energy, Score, Tech, Squads)"]
            AI["AiSystem & Pathfinder\n(A* Grid/Zones, AIGroups, Skirmish AI)"]
            Scripts["ScriptSystem\n(Conditions, Actions, SequentialScripts)"]
        end
        Sink["Box&lt;dyn ClientSink&gt;\n(Headless No-Op in Stage 1 / wgpu in Stage 2)"]
    end

    Cmd --> Scripts
    Scripts --> Terrain
    Terrain --> Sleepy
    Sleepy --> ObjArena
    Sleepy --> ModArena
    ModArena --> Part
    ModArena --> AI
    ModArena --> Players
    ObjArena --> RNG
    ModArena --> RNG
    AI --> RNG
```

### 3.1 Partitioning the ~71 `The*` Singletons into Three Clean Layers

Instead of 71 mutable globals, `SimContext` partitions all engine singletons by lifetime and mutability:

1. **Layer 1: `RulesRegistry` (Immutable After `GameEngine::init()`, Shared via `&RulesRegistry`):**
   - `GlobalData` (`TheGlobalData`)
   - `NameKeyTable` (`TheNameKeyGenerator` — sequential `NameKeyType` interning table; deterministic INI load order preserved)
   - `ScienceStore` (`TheScienceStore`)
   - `MultiplayerSettings` (`TheMultiplayerSettings`)
   - `TerrainTypeCollection` & `TerrainRoadCollection` (`TheTerrainTypes`, `TheTerrainRoads`)
   - `RankInfoStore` (`TheRankInfoStore`)
   - `PlayerTemplateStore` (`ThePlayerTemplateStore`)
   - `WeaponTemplateStore` (`TheWeaponStore` static templates)
   - `ObjectCreationListStore` (`TheObjectCreationListStore`)
   - `LocomotorTemplateStore` (`TheLocomotorStore` static templates)
   - `SpecialPowerStore` (`TheSpecialPowerStore`)
   - `ArmorStore` (`TheArmorStore`)
   - `ThingTemplateStore` (`TheThingFactory` — ~3,000 `ThingTemplate` definitions and their immutable `ModuleData` descriptors)
   - `UpgradeCenter` (`TheUpgradeCenter`)
   - `CrateSystem` (`TheCrateSystem`)
   - `StaticAiData` (`TheAI->getAiData()` static tuning parameters)

2. **Layer 2: `SimWorld` (Mutable Per-Match State, Reset Between Replays, Hashed by `getCRC()`):**
   - `frame: u32`, `game_mode: GameMode`, `crc: u32`, `rank_level_limit: i32`
   - `rng: LogicRngState` (`theGameLogicSeed: [u32; 6]`, `theGameLogicBaseSeed: u32`)
   - `objects: ObjectArena` (`m_objList`, `m_objHash`, `m_objectsToDestroy`, `m_nextObjID`)
   - `modules: ModuleArena` (all live `BehaviorModule` instances owned by objects)
   - `sleepy_updates: SleepyUpdateHeap` (`m_sleepyUpdates` custom binary min-heap)
   - `partition: PartitionGrid` (`ThePartitionManager` + `TheGhostObjectManager`)
   - `terrain: TerrainLogic` (`TheTerrainLogic`, dynamic bridge states, `PolygonTrigger` states, `SidesList`)
   - `players: PlayerStore` (`ThePlayerList`, `Player` instances, `TheTeamFactory`, `Team` instances, `Squad` instances, `TheBuildAssistant`, `TheCaveSystem`, `TheActionManager`)
   - `ai: AiSystem` (`TheAI`, `Pathfinder`, `AIGroup` store + `m_groupList`, `AIPlayer` / `AISkirmishPlayer`)
   - `scripts: ScriptSystem` (`TheScriptEngine`, `TheScriptConditions`, `TheScriptActions`, `TheVictoryConditions`)
   - `recorder: ReplayRecorder` (`TheRecorder`, `TheCommandList`, `TheMessageStream`)

3. **Layer 3: `ClientSink` Trait (Visual/Audio Side-Effects — Zero Logic Mutation):**
   - Wraps `TheGameClient`, `TheDisplay`, `TheInGameUI`, `TheControlBar`, `TheEva`, `TheAudio`, `TheRadar` (visual markers), `TheTacticalView`, `TheParticleSystemManager`, `TheFXListStore`, `TheDamageFXStore`.
   - In Stage 1 headless mode, `HeadlessClientSink` implements these as fast no-ops (except where `GameLogic` queries read-only drawable/bone metadata or `TheRadar` object registration), verified by `-VerifyClientCRC` to confirm zero impact on `GameLogic::getCRC()`.

---

### 3.2 `ObjectArena` & Deterministic Intrusive Lists Without Raw Pointers

`Object` is the central entity in `GameLogic`, participating simultaneously in:
- `GameLogic::m_objList` (intrusive doubly-linked list of all world objects, newest-first)
- `GameLogic::m_objHash` (`ObjectID -> Object*` lookup table)
- `Team::m_teamMemberList` (intrusive doubly-linked list of team members)
- `GameLogic::m_objectsToDestroy` (deferred end-of-frame destruction vector)
- `PartitionCell` / `PartitionData` spatial links

In Rust (`zh_logic`), we model `Object` storage as a generational slot arena (`ObjectArena`) with explicit intrusive link indices:

```rust
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct ObjectId(pub i32); // Matches C++ ObjectID (INVALID_ID = 0)

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ObjectSlot(pub u32); // Dense index into ObjectArena::slots

pub struct ObjectArena {
    pub slots: Vec<Option<ObjectRecord>>,
    pub free_slots: Vec<u32>,
    pub id_to_slot: BTreeMap<ObjectId, ObjectSlot>, // Deterministic lookup (or flat page table)
    pub world_head: Option<ObjectSlot>,             // Head of m_objList (newest object first)
    pub objects_to_destroy: Vec<ObjectSlot>,        // Deferred destroy queue (m_objectsToDestroy)
    pub next_obj_id: ObjectId,                      // Monotonic counter (m_nextObjID)
}
```

**Preserving Exact `Object` Lifecycle Invariants:**
1. **Prepend-on-Register (`registerObject`):** When an object is created, it is inserted at `world_head` (`new_obj.next_in_world = world_head; world_head = Some(slot)`), matching `obj->prependToList(&m_objList)`.
2. **Deferred Destruction (`destroyObject` -> `processDestroyList`):**
   - `destroy_object(slot)` checks `OBJECT_STATUS_DESTROYED`; if already set, returns immediately.
   - Otherwise, runs `DestroyModuleInterface::on_destroy()` on all matching modules, sets `OBJECT_STATUS_DESTROYED`, clears `AIUpdate` locomotor goals/paths, appends `slot` to `objects_to_destroy`, runs `obj.on_destroy()`, and removes wall pieces from `Pathfinder`.
   - At Step 15a of `GameLogic::update()`, `process_destroy_list()` iterates `objects_to_destroy` in FIFO order (allowing `onDelete` callbacks to push sub-objects to the tail of `objects_to_destroy`), removes the object's `UpdateModule`s from `SleepyUpdateHeap` using the **two-pass reverse-heap-index algorithm** (Section 3.4), unlinks `slot` from `world_head` and `id_to_slot`, and frees the slot.

---

### 3.3 `ModuleArena` & Facet Dispatch: Translating the 222 `BehaviorModule` Classes

In C++, `BehaviorModule` uses multiple inheritance across 10 lifecycle interfaces (`UpdateModuleInterface`, `DieModuleInterface`, `DamageModuleInterface`, `ContainModuleInterface`, `CollideModuleInterface`, `CreateModuleInterface`, `UpgradeModuleInterface`, `SpecialPowerModuleInterface`, `BodyModuleInterface`, `DestroyModuleInterface`) plus 25 specialized sub-interfaces (`AIUpdateInterface`, `ExitInterface`, `DockUpdateInterface`, `ParkingPlaceBehaviorInterface`, `ProjectileUpdateInterface`, `SlavedUpdateInterface`, `ProductionUpdateInterface`, `SpawnBehaviorInterface`, etc.).

In Rust, we eliminate vtable pointer webs and `dynamic_cast`-style hops with a **Two-Level Tagged Module Arena + 64-bit Facet Bitmask**:

```rust
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct ModuleSlot(pub u32);

bitflags::bitflags! {
    #[derive(Copy, Clone, Debug)]
    pub struct FacetMask: u64 {
        // 10 Primary Lifecycle Facets
        const UPDATE        = 1 << 0;
        const DIE           = 1 << 1;
        const DAMAGE        = 1 << 2;
        const CREATE        = 1 << 3;
        const COLLIDE       = 1 << 4;
        const BODY          = 1 << 5;
        const CONTAIN       = 1 << 6;
        const UPGRADE       = 1 << 7;
        const SPECIAL_POWER = 1 << 8;
        const DESTROY       = 1 << 9;

        // 25 Specialized Query Sub-Interfaces
        const AI_UPDATE             = 1 << 10;
        const PHYSICS_BEHAVIOR      = 1 << 11;
        const STEALTH_UPDATE        = 1 << 12;
        const SPY_VISION_UPDATE     = 1 << 13;
        const PROJECTILE_UPDATE     = 1 << 14;
        const EXIT_INTERFACE        = 1 << 15;
        const DOCK_UPDATE           = 1 << 16;
        const RAILED_DOCK_UPDATE    = 1 << 17;
        const SLOW_DEATH_BEHAVIOR   = 1 << 18;
        const SPECIAL_POWER_UPDATE  = 1 << 19;
        const SLAVED_UPDATE         = 1 << 20;
        const PRODUCTION_UPDATE     = 1 << 21;
        const HORDE_UPDATE          = 1 << 22;
        const POWER_PLANT_UPDATE    = 1 << 23;
        const SPAWN_BEHAVIOR        = 1 << 24;
        const COUNTERMEASURES       = 1 << 25;
        const PARKING_PLACE         = 1 << 26;
        const REBUILD_HOLE          = 1 << 27;
        const BRIDGE_BEHAVIOR       = 1 << 28;
        const BRIDGE_TOWER          = 1 << 29;
        const BRIDGE_SCAFFOLD       = 1 << 30;
        const OVERCHARGE_BEHAVIOR   = 1 << 31;
        const TRANSPORT_PASSENGER   = 1 << 32;
        const CAVE_INTERFACE        = 1 << 33;
        const LAND_MINE_INTERFACE   = 1 << 34;
        const EJECT_PILOT_DIE       = 1 << 35;
    }
}

pub struct ModuleRecord {
    pub owner: ObjectSlot,
    pub tag_key: NameKeyType,
    pub template_data_id: ModuleDataId,
    pub facets: FacetMask,
    // SleepyUpdate scheduler fields (valid when facets.contains(FacetMask::UPDATE))
    pub next_call_frame_and_phase: u32,
    pub index_in_logic: i32,
    // Concrete module state
    pub kind: ModuleKind,
}
```

**Why This Design Fits Zero Hour's Object Model:**
1. **Multi-Facet Modules Share State Naturally:** A module like `SlowDeathBehavior` (which implements `UpdateModuleInterface`, `DieModuleInterface`, and `SlowDeathBehaviorInterface`) is a single `ModuleKind::Behavior(BehaviorKind::SlowDeath(SlowDeathState))` record with `facets = UPDATE | DIE | SLOW_DEATH_BEHAVIOR`. Both `ModuleRecord::update(&mut self, ...)` and `ModuleRecord::on_die(&mut self, ...)` dispatch to that same `SlowDeathState` instance!
2. **Exact Helper & Behavior Ordering on `ObjectRecord`:**
   `ObjectRecord` stores `behaviors: SmallVec<[(ModuleSlot, FacetMask); 16]>`, populated in the exact C++ constructor order (up to 8 conditional sleep helpers first: `SMCHelper`, `StatusDamageHelper`, `SubdualDamageHelper`, `RepulsorHelper`, `DefectionHelper`, `WeaponStatusHelper`, `FiringTracker`, `TempWeaponBonusHelper`; followed by INI behavior modules in `ThingTemplate` order), plus cached `Option<ModuleSlot>` fields for `body`, `contain`, `stealth`, `ai`, and `physics`.
3. **Zero-Overhead Facet Iteration:** When `Object::onDie` or `Object::onDamage` iterates `obj.behaviors`, checking `(slot, mask) if mask.contains(FacetMask::DIE)` skips non-matching modules in a single register test without touching `ModuleArena` memory.
4. **No Borrow-Checker Conflicts During Update/Event Callbacks:**
   When module `M` on object `O` executes `update()`, `M` may mutate other objects (e.g. dealing damage to a target) or call methods on `O` itself. To avoid `&mut SimWorld` aliasing panics, `SimWorld` uses the standard **take-and-restore (or index-passed context)** pattern: either the active `ModuleKind` state is temporarily swapped out with a lightweight placeholder while `update(&mut self, ctx: &mut SimWorld, owner: ObjectSlot, mod_slot: ModuleSlot)` runs, or `ModuleRecord` methods take `(world: &mut SimWorld, mod_slot: ModuleSlot)` and read/write fields via indexed arena access.

---

### 3.4 `SleepyUpdateHeap`: 1:1 Port of `GameLogic::m_sleepyUpdates`

As proven in `docs/RESEARCH.md` Section 3.3, `SleepyUpdateHeap` must replicate the exact array-backed min-heap semantics of `GameLogic.cpp:3040-3350`:

```rust
pub struct SleepyUpdateHeap {
    pub heap: Vec<ModuleSlot>,
}
```

With the five mandatory invariants:
1. **Priority Encoding:** `priority = (wake_frame.min(0x3fff_ffff) << 2) | (phase as u32)` where `SleepyUpdatePhase` is `Initial = 0`, `Physics = 1`, `Normal = 2`, `Final = 3`.
2. **Strict `>` Comparator (`is_lower_priority`):** `a_pri > b_pri`. Sifting down (`rebalance_child`) picks the right child `2*i + 2` **only** if `left_pri > right_pri`; when equal, it always picks the left child `2*i + 1`, and stops sifting as soon as `!(parent_pri > child_pri)`.
3. **In-Place Root Reschedule:** At Step 11 of `GameLogic::update()`, after `heap[0]` runs and returns `sleep_len`, update `heap[0]`'s priority to `((now + sleep_len) << 2) | phase` and call `rebalance_sleepy_update(0)` in place (never pop + push).
4. **Already-Awake Wakeup Guard (`awaken_update_module`):** If `now > 0 && mod.next_call_frame() == now && when_to_wake_up == now + 1`, return immediately without modifying `heap`.
5. **Two-Pass Reverse-Heap-Index Erase (`process_destroy_list`):** When destroying `obj`, first scan `heap[0..len]` linearly to collect all `ModuleSlot`s owned by `obj` into a stack buffer `suo[0..num_suo]`; second, iterate `k` from `num_suo - 1` down to `0`, reading `idx = modules[suo[k]].index_in_logic` and calling `erase_sleepy_update(idx)`.

---

## 4. Three FFI Strangler Bridge Patterns (C++ <-> Rust Coexistence)

To port ~330k lines incrementally while keeping `GeneralsXZH -headless -replay` 100% green at every commit, `crates/zh_ffi` uses three structured FFI patterns depending on subsystem granularity:

```mermaid
flowchart LR
    subgraph Pattern1["Pattern 1: Leaf / Global C-ABI"]
        CPP1["C++ Caller\n(e.g. RandomValue.cpp)"] -->|"extern C fn"| FFI1["zh_ffi\n#[no_mangle]"]
        FFI1 --> R1["zh_common / zh_logic\nRust Implementation"]
    end

    subgraph Pattern2["Pattern 2: Handle-Backed Sub-Component"]
        CPP2["C++ Class\n(e.g. ExperienceTracker)"] -->|"u32 Handle + Parent Ptr"| FFI2["zh_ffi Component API"]
        FFI2 --> R2["Rust Arena\nSlotStorage&lt;T&gt;"]
        R2 -.->|"C-ABI Parent Callback\n(until parent ported)"| CPP2
    end

    subgraph Pattern3["Pattern 3: Universal BehaviorModule Proxy"]
        MF["C++ ModuleFactory"] -->|"Instantiates for ported key"| Proxy["C++ RustBehaviorModuleAdapter\n(1 class for all 222 modules)"]
        Proxy -->|"ModuleSlot (u32)"| FFI3["zh_ffi Module Dispatch Table"]
        FFI3 --> R3["zh_logic::ModuleArena\n(222 ModuleKind variants)"]
    end
```

### 4.1 Pattern 1: Stateless & Global-Singleton C-ABI Replacement (Phases 1–2)
- **Target Subsystems:** `RandomValue`, `CRC`, `XferCRC`, `GameMath` (`gm_*f`), `BitFlags`, `NameKeyGenerator`, `INI` numeric scanners.
- **Mechanism:** The C++ `.cpp` implementation bodies are replaced with 1-line forwarding calls to `extern "C"` functions exported by `libzh_ffi.a` (`zh_random_init`, `zh_get_game_logic_random_value`, `zh_xfer_crc_add`, etc.).
- **Zero Layout Coupling:** Only primitive values (`i32`, `u32`, `f32`, `*const u8`, `usize`) cross the boundary.

### 4.2 Pattern 2: Handle-Backed Component Bridge with Parent Callbacks (Phases 1, 3, 4)
- **Target Subsystems:** Self-contained classes owned by `Object` or `Player` before `Object`/`Player` themselves are moved to Rust (`ExperienceTracker`, `FiringTracker`, `Energy`, `Money`, `ScoreKeeper`, `ArmorSet`, `WeaponSet`, `Locomotor`, `PartitionManager`).
- **Mechanism:**
  - Either the C++ class's `#[repr(C)]` data layout is passed directly by pointer to Rust `extern "C"` methods, **or** the C++ class stores a `u32` Rust arena handle allocated in its constructor and freed in its destructor.
  - Where the Rust component needs to query or notify its unported C++ parent (e.g., `ExperienceTracker::addExperiencePoints` calling `m_parent->getTemplate()->getExperienceRequired(level)` and `m_parent->onVeterancyLevelChanged(...)`), C++ exports thin `extern "C"` accessor functions (`zh_cxx_obj_get_exp_required(obj_ptr, level)`, `zh_cxx_obj_on_veterancy_changed(obj_ptr, old_lvl, new_lvl, feedback)`).
  - Once the parent (`Object` or `Player`) is ported to Rust in Phase 5/6, those `zh_cxx_*` callbacks are deleted.

### 4.3 Pattern 3: Universal `RustBehaviorModuleAdapter` Proxy for the 222 `BehaviorModule` Classes (Phase 5)
- **Architectural Problem:** Writing 222 bespoke C++ wrapper classes for the 222 `BehaviorModule` subclasses would create massive FFI boilerplate and fragile vtable glue.
- **Architectural Solution:** We introduce **one** C++ adapter class in `z_gameengine`, `RustBehaviorModuleAdapter`, which inherits from `UpdateModule` and implements the sibling facet interfaces (`DieModuleInterface`, `DamageModuleInterface`, `CollideModuleInterface`, `ContainModuleInterface`, `CreateModuleInterface`, `UpgradeModuleInterface`, `SpecialPowerModuleInterface`, `BodyModuleInterface`, `DestroyModuleInterface`, and the specialized sub-interfaces).
  - `RustBehaviorModuleAdapter` holds a single `uint32_t m_rustModuleSlot` and a `uint64_t m_facetMask` queried from Rust at construction time.
  - Its facet query methods (`getDie()`, `getDamage()`, `getContain()`, `getCollide()`, `getUpdate()`, etc.) simply test `m_facetMask`:
    ```cpp
    DieModuleInterface* getDie() override {
        return (m_facetMask & FACET_DIE) ? this : nullptr;
    }
    ```
  - Each virtual method (`update()`, `onDie()`, `onDamage()`, `onCollide()`, `crc()`, `xfer()`, `loadPostProcess()`) forwards `(m_rustModuleSlot, args...)` through a single C-ABI entry point in `zh_ffi` (`zh_module_update(slot)`, `zh_module_on_die(slot, damage_info)`, etc.).
  - When `ModuleFactory::newModule` is called for a module name whose Rust port is complete, it calls `zh_module_create(module_kind_id, thing_ptr, module_data_ptr)` and returns a `RustBehaviorModuleAdapter(slot, mask)`.
  - **Result:** Any of the 222 modules can be translated to pure Rust inside `crates/zh_logic` and enabled in `ModuleFactory` with **zero new C++ glue code**! And at the end of Phase 5 when `Object` itself moves into Rust's `ObjectArena`, `RustBehaviorModuleAdapter` is deleted entirely.

---

## 5. Shadow Verification & Automated CRC Desync Triage Architecture

Waiting for a periodic frame-100 replay CRC mismatch to tell us that a newly translated module has a bug makes debugging slow. We pair the replay CRC oracle with two fast-feedback verification layers:

### 5.1 Layer 1: Dual-Execution Shadow Verification (`#[cfg(feature = "shadow-verify")]`)
During initial bring-up of any tricky mathematical or stateful module (e.g. `RandomValue`, `ExperienceTracker`, `Locomotor`, `Weapon`, `PartitionManager`, `Pathfinder`):
- Both the C++ reference implementation and the Rust implementation execute on the exact same inputs during a headless replay run.
- At the end of every call, the FFI bridge compares the Rust return value, mutated struct state, and `GetGameLogicRandomSeedCRC()` against the C++ result.
- If a single bit differs (even on frame 17, 83 frames before the next replay CRC check), `shadow-verify` logs the exact function name, input arguments, C++ output, and Rust output, and aborts immediately.

### 5.2 Layer 2: 5-Marker Frame Bisection Playbook (`GameLogic::getCRC`)
If a full replay check fails with `CRC Mismatch in Frame F (InGame: 0xAAAA != Replay: 0xBBBB)`:
1. Re-run that single replay with:
   `-headless -replay <file> -DebugCRCFromFrame (F-2) -DebugCRCUntilFrame (F+1) -LogObjectCRCs -SaveDebugCRCPerFrame /tmp/crc_rust`
   and compare against the stock C++ baseline output `/tmp/crc_cpp`.
2. Inspect the 5 section checkpoints logged inside `GameLogic::getCRC()` on frame $F$:
   - `CRC after objects for frame F` (`MARKER:Objects` — if this already differs, diff the per-object `CRC of Object <ID> (<TemplateName>)` lines from `-LogObjectCRCs` to identify the exact `ObjectID` and whether `m_privateStatus`, `Matrix3D`, `m_objectUpgradesCompleted`, `ExperienceTracker`, `health`, `m_weaponBonusCondition`, `damageScalar`, or `Weapon` diverged!).
   - `RandomSeed: <seed>` (`GetGameLogicRandomSeedCRC()` — if `MARKER:Objects` matched but `RandomSeed` diverged, an extra or missing `GameLogicRandomValue*` call occurred during frame $F$).
   - `CRC after partition manager for frame F` (`MARKER:ThePartitionManager` — cell shroud/sight/threat/value divergence).
   - `CRC after PlayerList for frame F` (`MARKER:ThePlayerList` — money, energy, score, science, upgrade, or team membership divergence).
   - `CRC after AI pathfinder` / `CRC after AI for frame F` (`MARKER:TheAI` — pathfinder grid or `AIGroup` state divergence).

---

## 6. In-Depth Phased Execution Roadmap & Work-Package Breakdown

### Phase 0: Ground Truth & Replay Corpus (C++ Baseline)
**Goal:** Build unmodified `fbraz3/GeneralsX` at commit `84a7a20769aa5749f388b1f49bd542c79b3c33e8` on this Mac, assemble a multi-replay verification corpus, and prove 100% headless replay pass rate on C++.

| Work Package | Scope & Tasks | Gate / Deliverable |
|---|---|---|
| **WP-0.1: Disk Budget & Retail Game Data** | Verify free disk space (~13 GiB available; cap total build + assets under 7 GiB). Populate `~/Games/generalszh` (Steam App `2732960` via interactive `steamcmd +@sSteamCmdForcePlatformType windows`, or existing read-only path). Never copy `.big` archives; pass `-assetPath ~/Games/generalszh` (`CNC_GENERALS_ZH_PATH`). | `~/Games/generalszh/*.big` readable in place. |
| **WP-0.2: Reference C++ Build (`GeneralsXZH`)** | Shallow-clone `fbraz3/GeneralsX` at `84a7a20769aa5749f388b1f49bd542c79b3c33e8` into ignored `vendor/GeneralsX`. Configure preset `macos-vulkan` with `-DRTS_BUILD_OPTION_DEBUG=OFF -DRTS_BUILD_GENERALS=OFF -DRTS_BUILD_CORE_TOOLS=OFF` and build target `z_generals` (`GeneralsXZH`). | `GeneralsXZH` `arm64` binary builds cleanly. |
| **WP-0.3: Replay Corpus Assembly** | Install `generalsx-project/GeneralsXReplays` (6 cross-platform Zero Hour replays: `linux_2p_vanilla_map.rep`, `linux_2p_custom_map.rep`, `linux_6p_custom_map.rep`, `macos_2p_vanilla_map.rep`, `macos_2p_custom_map.rep`, `macos_6p_custom_map.rep` + custom maps) into `~/Library/Application Support/GeneralsX/GeneralsZH/`. Record additional local skirmish replays covering USA/China/GLA generals, superweapons, air/ground combat, veterancy, and power brownouts into `replays/`. Characterize `TheSuperHackers/GeneralsReplays` under `-ignoreReplaySyncErrors`. | Corpus installed and indexed. |
| **WP-0.4: Automated Verification Harness** | Write `scripts/check_replays.sh` running `GeneralsXZH -headless -replay "*.rep" -jobs 4`. Record wall-clock time, frame count, and pass status per replay in `docs/BASELINE.md`. | **Gate 0:** 100% pass (`exit 0`, zero CRC mismatches) on stock C++. |

---

### Phase 1: Translation Spike (Go / No-Go Gate)
**Goal:** Validate the multi-model translation workflow, FFI build integration, and determinism conventions on two real modules (`RandomValue` + `CRC` and `ExperienceTracker`) before scaling to 330k lines.

| Work Package | Scope & Tasks | Gate / Deliverable |
|---|---|---|
| **WP-1.1: Rust Conventions, Workspace & FFI Seam (Lead Model)** | Author `docs/RUST_CONVENTIONS.md` encoding all 10 determinism rules from `docs/RESEARCH.md` with C++ -> Rust templates. Create Cargo workspace (`crates/zh_common`, `crates/zh_logic`, `crates/zh_ffi`) compiling `libzh_ffi.a`. Link `libzh_ffi.a` into `z_gameengine` in `GeneralsMD/Code/GameEngine/CMakeLists.txt`. Define C-ABI header `zh_ffi.h`. | Hybrid C++/Rust `GeneralsXZH` links and passes Gate 0. |
| **WP-1.2: Autonomous Translation of `RandomValue` + `CRC` & `ExperienceTracker` (Cost-Optimized Model)** | Cost-optimized model translates under `docs/RUST_CONVENTIONS.md`:<br>1. `RandomValue.cpp` + `crc.h`/`crc.cpp` (~320 LOC, Pattern 1 FFI): 3 `[u32; 6]` seed banks, `ADC` carry chain, `InitRandom`, `GetGameLogicRandomSeed`, `GetGameLogicRandomSeedCRC`, `GetGameLogicRandomValue`, `GetGameLogicRandomValueReal`, `*Unchanged`, `GameLogicRandomVariable`.<br>2. `ExperienceTracker.cpp` (~280 LOC, Pattern 2 FFI): veterancy state, float-scaled XP truncation (`((gain as f32) * scalar) as i32`), sink forwarding (`m_experienceSink`), level threshold promotion, `ExperienceTracker::crc` / `xfer`. | Both modules compiled into `libzh_ffi.a` and wired into `z_gameengine`. |
| **WP-1.3: Spike Verification & Telemetry Report** | Run `cargo test` + `scripts/check_replays.sh` (with and without `shadow-verify`). Record prompt/completion tokens consumed, first-pass compile & replay pass rate, and exact count/category of lead-model fixes in `docs/PHASE1_SPIKE_REPORT.md`. | **Gate 1 (Owner Go/No-Go):** All replays green; owner reviews `docs/PHASE1_SPIKE_REPORT.md`. |

---

### Phase 2: Foundation, Math & Serialization (`Common` Layer, ~40k LOC)
**Goal:** Move the core deterministic primitives, serialization backbone, and INI scanners into `crates/zh_common` (Pattern 1 FFI).

| Work Package | Files & Subsystems | Approx. LOC | Verification Gate |
|---|---|---:|---|
| **WP-2.1: Checksums, BitFlags & Xfer** | `Xfer.cpp`, `XferCRC.cpp`, `XferDeepCRC`, `BitFlags.h`/`BitFlags.cpp` (`ObjectStatusMaskType`, `UpgradeMaskType` [`BitFlags<128>`], `KindOfMaskType`, `DisabledMaskType`), `Snapshot` trait. | ~6,500 | `XferCRC` unit tests + full replay suite green. |
| **WP-2.2: Deterministic Math & Geometry** | FFI bindings + Rust wrappers/port of `TheSuperHackers/GameMath` (`gm_sinf`, `gm_cosf`, `gm_tanf`, `gm_acosf`, `gm_asinf`, `gm_atan2f`, `gm_sqrtf`, `gm_floorf`, `gm_ceilf`), `Coord2D`, `Coord3D`, `ICoord2D`, `ICoord3D`, `Region2D`, `Region3D`, `Matrix3D`, `Vector3`, `GeometryInfo`, `BezierSegment`, `DiscreteCircle`, `PartitionSolver`. | ~11,000 | Math bit-parity fuzz tests against C++ `GameMath` + full replay suite green. |
| **WP-2.3: NameKeys, Strings, Dict & StateMachine** | `NameKeyGenerator.cpp`, `AsciiString` / `UnicodeString` FFI views, `Dict.cpp` (ordered key-value dictionary used by maps/scripts), `StateMachine.cpp` (base finite state machine engine used by AI). | ~9,500 | Verify exact `NameKeyType` IDs after `GameEngine::init()` + full replay suite green. |
| **WP-2.4: INI Parser & Numeric Scanners** | `Core/GameEngine/Source/Common/INI/*` (28 files: tokenizers, `from_chars` numeric scanners, `scanPercentToReal`, `scanIndexList`, `BitFlagsIO`, `MultiIniFieldParse`). | ~13,000 | Verify `TheGlobalData->m_iniCRC` matches C++ bit-exact + full replay suite green. |

---

### Phase 3: Core RTS Domain Models & Leaf Trackers (~35k LOC)
**Goal:** Port self-contained gameplay value trackers and template stores into `crates/zh_logic` (Pattern 2 FFI).

| Work Package | Files & Subsystems | Approx. LOC | Verification Gate |
|---|---|---:|---|
| **WP-3.1: Economy & Player Leaf Trackers** | `Money.cpp`, `Energy.cpp` (power production/consumption, brownout callbacks, sabotage frame checks, `Energy::crc`), `ScoreKeeper.cpp`, `Handicap.cpp`, `MissionStats.cpp`, `AcademyStats.cpp`. | ~8,000 | `MARKER:ThePlayerList` CRCs bit-exact across all replays. |
| **WP-3.2: Tech Tree, Upgrades & Special Powers** | `Science.cpp`, `RankInfo.cpp`, `Upgrade.cpp` (`UpgradeCenter`, `UpgradeTemplate`), `SpecialPower.cpp` (`SpecialPowerStore`), `ProductionPrerequisite.cpp`, `PlayerTemplate.cpp`, `MultiplayerSettings.cpp`. | ~11,500 | Full replay suite green. |
| **WP-3.3: Combat & Locomotion Leaf Models** | `Armor.cpp` (`ArmorTemplate`, `ArmorSet`), `Damage.cpp`, `FiringTracker.cpp`, `TurretAI.cpp`, `Locomotor.cpp` & `LocomotorSet.cpp` (ground, wheeled, tracks, hover, wings, thrust, cliff, parachute physics math). | ~15,500 | Shadow-verify `Locomotor` trajectories & `TurretAI` angles + full replay suite green. |

---

### Phase 4: Spatial & World Subsystems (~30k LOC)
**Goal:** Port map geometry, triggers, and the spatial partition grid (`MARKER:ThePartitionManager` in `GameLogic::getCRC`).

| Work Package | Files & Subsystems | Approx. LOC | Verification Gate |
|---|---|---:|---|
| **WP-4.1: TerrainLogic, Triggers & Map Metadata** | `TerrainLogic.cpp` (heightmap interpolation, normal calculation, cliff/water/bridge queries), `PolygonTrigger.cpp`, `SidesList.cpp`, `CrateSystem.cpp`, `CaveSystem.cpp`. | ~11,000 | Shadow-verify `getGroundHeight` & trigger area checks + full replay suite green. |
| **WP-4.2: PartitionManager, Shroud/Sight & GhostObject** | `PartitionManager.cpp` (~55 KB header + ~140 KB source: `PartitionCell`, `PartitionData`, `SightingInfo`, `m_pendingUndoShroudReveals`, threat/value maps, range/geometric object iterators, `PartitionManager::crc`), `SimpleObjectIterator.cpp`, `GhostObject.cpp`. | ~19,000 | `MARKER:ThePartitionManager` CRCs bit-exact across all replays. |

---

### Phase 5: `Object`, `Weapon`, and the 222 `BehaviorModule` Classes (~135k LOC)
**Goal:** Deploy the Universal `RustBehaviorModuleAdapter` proxy (Pattern 3 FFI) and strangle all 222 object modules plus `Weapon` and `Object` into `crates/zh_logic` (`MARKER:Objects` in `GameLogic::getCRC`).

| Work Package | Files & Subsystems | Approx. LOC | Verification Gate |
|---|---|---:|---|
| **WP-5.0: `ModuleArena` & Universal `RustBehaviorModuleAdapter`** | Build `ModuleArena`, `FacetMask`, and the single C++ `RustBehaviorModuleAdapter` class in `z_gameengine` + `ModuleFactory` hook. | ~2,500 | Adapter verified with 1 test module; full replay suite green. |
| **WP-5.1: Helpers, Destroy, Damage, Create & Body (28 files)** | `Object/Helper/*` (8 sleep helpers: `ObjectSMCHelper`, `StatusDamageHelper`, `SubdualDamageHelper`, `ObjectRepulsorHelper`, `ObjectDefectionHelper`, `ObjectWeaponStatusHelper`, `TempWeaponBonusHelper`, `ObjectSpecialPowerHelper`), `Object/Destroy/*` (1), `Object/Damage/*` (3), `Object/Create/*` (8), `Object/Body/*` (8: `BodyModule`, `ActiveBody`, `StructureBody`, `HighlanderBody`, `HiveStructureBody`, `ImmortalBody`, `InactiveBody`, `UndeadBody`). | ~13,500 | All 28 modules running in `ModuleArena`; `Object::crc` health/damageScalar bit-exact. |
| **WP-5.2: Die, SpecialPower, Upgrade & Collide (64 files)** | `Object/Die/*` (12), `Object/SpecialPower/*` (11), `Object/Upgrade/*` (21), `Object/Collide/*` (20, including `CrateCollide`). | ~26,000 | All 64 modules running in `ModuleArena`; full replay suite green. |
| **WP-5.3: Contain & Behavior Modules (43 files)** | `Object/Contain/*` (13: `OpenContain`, `TransportContain`, `GarrisonContain`, `OverlordContain`, `HelixContain`, `TunnelContain`, `CaveContain`, `HealContain`, `InternetHackContain`, `MobNexusContain`, `ParachuteContain`, `RailedTransportContain`), `Object/Behavior/*` (30: `AutoHealBehavior`, `BridgeBehavior`, `BridgeTowerBehavior`, `BridgeScaffoldBehavior`, `CountermeasuresBehavior`, `DumbProjectileBehavior`, `FirestormDynamicGeometryInfoUpdate`, `GrantStealthBehavior`, `InstantDeathBehavior`, `JetSlowDeathBehavior`, `MinefieldBehavior`, `NeutronBlastBehavior`, `OverchargeBehavior`, `ParkingPlaceBehavior`, `PoisonedBehavior`, `POWTruckBehavior`, `PrisonBehavior`, `PropagandaTowerBehavior`, `RebuildHoleBehavior`, `SlowDeathBehavior`, `SpawnBehavior`, `SupplyWarehouseCripplingBehavior`, `TechBuildingBehavior`). | ~42,000 | All 43 modules running in `ModuleArena`; full replay suite green. |
| **WP-5.4a: Update Modules Batch A — Timers, Stealth & Detectors (18 files)** | `UpdateModule`, `AutoFindHealingUpdate`, `AudioLoopUpgrade`, `BoneFXUpdate`, `CheckpointUpdate`, `CleanupHazardUpdate`, `CommandButtonHuntUpdate`, `DeletionUpdate`, `DemoTrapUpdate`, `DynamicGeometryInfoUpdate`, `EnemyNearUpdate`, `FireOCLAfterWeaponCooldownUpdate`, `FireSpreadUpdate`, `FlammableUpdate`, `FloatUpdate`, `HeightDieUpdate`, `LifetimeUpdate`, `StealthDetectorUpdate`, `StealthUpdate`. | ~11,000 | Full replay suite green. |
| **WP-5.4b: Update Modules Batch B — Physics, Projectiles & Beams (17 files)** | `PhysicsUpdate` (`PhysicsBehavior`), `MissileAIUpdate`, `LaserUpdate`, `FireWeaponUpdate`, `FireWeaponWhenDamagedBehavior`, `FireWeaponWhenDeadBehavior`, `NeutronMissileUpdate`, `NeutronMissileSlowDeathBehavior`, `PartTheCloudUpdate`, `PointDefenseLaserUpdate`, `ProjectileStreamUpdate`, `RadiusDecalUpdate`, `SmartBombTargetHomingUpdate`, `SpectreGunshipUpdate`, `SpectreGunshipDeploymentUpdate`, `StickyBombUpdate`, `StrafeAreaUpdate`, `WaveGuideUpdate`. | ~13,500 | Full replay suite green. |
| **WP-5.4c: Update Modules Batch C — Production, Economy, Dock & Workers (17 files)** | `ProductionUpdate`, `ProductionExitUpdate` (`DefaultProductionExitUpdate`, `QueueProductionExitUpdate`, `SpawnPointProductionExitUpdate`, `SupplyCenterProductionExitUpdate`), `DockUpdate`, `SupplyCenterDockUpdate`, `SupplyWarehouseDockUpdate`, `RepairDockUpdate`, `RailedTransportDockUpdate`, `PrisonDockUpdate`, `AutoDepositUpdate`, `SupplyTruckAIUpdate`, `WorkerAIUpdate`, `DozerAIUpdate`, `EMPUpdate`, `PowerPlantUpdate`, `RadarUpdate`. | ~15,000 | Full replay suite green. |
| **WP-5.4d: Update Modules Batch D — Aircraft, Vehicles & Transports (17 files)** | `JetAIUpdate`, `ChinookAIUpdate`, `TransportAIUpdate`, `AssaultTransportAIUpdate`, `RailedTransportAIUpdate`, `HackInternetAIUpdate`, `POWTruckAIUpdate`, `WanderAIUpdate`, `DeliverPayloadAIUpdate`, `DeliverPayloadStateMachine`, `DeployStyleAIUpdate`, `AssistedTargetingUpdate`, `BattlePlanUpdate`, `HordeUpdate`, `LargeGroupAudioUpdate`, `MobMemberSlavedUpdate`, `SlavedUpdate`. | ~17,500 | Full replay suite green. |
| **WP-5.4e: Update Modules Batch E — Core `AIUpdate`, Superweapons & Topple (17 files)** | `AIUpdate.cpp` (base unit AI state machine driver), `OCLUpdate`, `SpecialPowerUpdateModule`, `SpecialAbilityUpdate`, `ParticleUplinkCannonUpdate`, `BaikonurLaunchPower`, `SpyVisionUpdate`, `ToppleUpdate`, `StructureToppleUpdate`, `StructureCollapseUpdate`, `HijackerUpdate`, `PilotFindVehicleUpdate`, `DynamicShroudClearingRangeUpdate`, `BaseRegenerateUpdate`, `ProneUpdate`, `TensileFormationUpdate`. | ~18,000 | All 86 `Update` modules running in `ModuleArena`; full replay suite green. |
| **WP-5.5: `Weapon`, `WeaponSet`, `ObjectCreationList` & `Object` Core** | `Weapon.cpp` (`Weapon`, `WeaponTemplate`, `Weapon::crc`), `WeaponSet.cpp`, `ObjectCreationList.cpp`, `Object.cpp` (`ObjectRecord` in `ObjectArena`, `Object::crc`). Delete `RustBehaviorModuleAdapter` proxy once `Object` owns `ModuleSlot`s directly in Rust. | ~26,000 | `MARKER:Objects` CRCs 100% computed by Rust across all replays. |

---

### Phase 6: `Player`, `Team`, `Pathfinder`, `AI`, and `ScriptEngine` (~70k LOC)
**Goal:** Port player management, A* pathfinding, tactical/skirmish AI, and map scripting (`MARKER:ThePlayerList` and `MARKER:TheAI` in `GameLogic::getCRC`).

| Work Package | Files & Subsystems | Approx. LOC | Verification Gate |
|---|---|---:|---|
| **WP-6.1: `Player`, `PlayerList`, `Team`, `Squad` & `BuildAssistant`** | `Player.cpp` (~90 KB), `PlayerList.cpp` (`PlayerList::crc`), `Team.cpp` (~55 KB), `Squad.cpp`, `BuildAssistant.cpp`, `ResourceGatheringManager.cpp`, `ActionManager.cpp`. | ~18,500 | `MARKER:ThePlayerList` CRCs 100% computed by Rust across all replays. |
| **WP-6.2: `Pathfinder` (`Core` + `GeneralsMD`)** | `Core/GameEngine/Source/GameLogic/AI/Pathfind*.cpp` (10 files) + `GeneralsMD/.../AIPathfind.cpp` (pathfind grid cells, zones, hierarchical A*, unit footprint collision, tunnel/bridge pathing, `Pathfinder::crc`). | ~16,500 | Shadow-verify path output & `CRC after AI pathfinder` bit-exact across all replays. |
| **WP-6.3: `AI`, `AIGroup`, `AIPlayer`, `AISkirmishPlayer` & `AIStateMachine`** | `AI.cpp` (`AI::crc`), `AIGroup.cpp` (`AIGroup::crc`), `AIPlayer.cpp`, `AISkirmishPlayer.cpp`, `AIStateMachine.cpp`, `AIStates.cpp`, `AITNGuard.cpp`. | ~18,000 | `MARKER:TheAI` CRCs 100% computed by Rust across all replays. |
| **WP-6.4: `ScriptEngine`, `ScriptConditions`, `ScriptActions` & `VictoryConditions`** | `Scripts.cpp`, `ScriptEngine.cpp`, `ScriptConditions.cpp` (~170 KB), `ScriptActions.cpp` (~270 KB), `SequentialScript.cpp`, `VictoryConditions.cpp`. | ~22,000 | Map scripts execute 100% in Rust; full replay suite green. |

---

### Phase 7: Top-Level `GameLogic` Loop & Standalone Stage 1 Milestone (~20k LOC)
**Goal:** Port `GameLogic.cpp` (`SleepyUpdateHeap`, `processCommandList`, `getCRC`), `Recorder.cpp` (`.rep` parser & CRC verifier), `ArchiveFileSystem` (`.big` reader), and `MapReader` so the simulation runs as a standalone 100% pure-Rust executable (`zh-sim`) with zero C++ linked.

| Work Package | Files & Subsystems | Approx. LOC | Verification Gate |
|---|---|---:|---|
| **WP-7.1: `GameLogic` Frame Scheduler & Command Dispatcher** | `GameLogic.cpp`: `SleepyUpdateHeap` (`m_sleepyUpdates`), all 55 `on*` `GameMessage` command handlers in `logicMessageDispatcher`, `startNewGame`, `processDestroyList`, `GameLogic::update()`, `GameLogic::getCRC()`. | ~9,500 | Entire 16-step `GameLogic::update()` runs in Rust inside `GeneralsXZH`. |
| **WP-7.2: Pure-Rust `.big`, `.map`, `.csf` & `.rep` Readers** | `ArchiveFileSystem` / `BIGFileSystem` (`.big` archive directory reader), `.map` chunk reader (`DataChunk.cpp`, `MapObject.cpp`), `Recorder.cpp` (`GENREP` header, UTF-16/UTF-32 string probe, command stream, `handleCRCMessage`). | ~7,500 | Standalone Rust readers parse retail `.big`, `.map`, and `.rep` files identically to C++. |
| **WP-7.3: Standalone `zh-sim` Binary & Stage 1 Benchmark** | Wire `crates/zh_sim` (`zh-sim -headless -replay <glob> -jobs N`). Run the entire replay corpus in 100% pure Rust with zero C++ linked. Compare every frame's CRC and benchmark simulation throughput (frames/sec) vs the Phase 0 C++ baseline. | ~3,000 | **Stage 1 Milestone Complete:** 100% bit-exact replay parity in standalone pure Rust. |

---

### Stage 2 (Future): Native Client & `wgpu`/Metal Renderer
Once Stage 1 is complete and signed off:
- Implement `WgpuClientSink` in `crates/zh_client` using `wgpu` (Metal backend on macOS Apple Silicon): `.w3d` hierarchical mesh/bone/animation loader, `.dds`/`.tga` texture pipeline, terrain heightmap/road/water renderer, particle system, `.wnd` UI layout engine, and MiniAudio/cpal audio output driving `SimContext` at 30 Hz logic / unlocked render framerate.

---

## 7. Sign-Off Checkpoints Before Phase 0 / Phase 1 Execution

This repository (`chayuto/generals-rust`, local path `/Users/chayut/repos/generals-rust`) contains the complete architectural blueprint and first deliverable:
- `PLAN.md` (this architectural & work-package execution plan)
- `docs/RESEARCH.md` (deep technical reference: 48-singleton `GameEngine::init()` sequence, 16-step `GameLogic::update()` pipeline, `m_sleepyUpdates` heap invariants, `Object` + 222 `BehaviorModule` facet architecture, `XferCRC` / `GameLogic::getCRC()` specification, determinism regimes, and disk budget analysis)
- `LICENSE.md` (GPL-3.0-or-later + EA Section 7 Additional Terms + Notice of Modification)
- `.gitignore` (blocks all retail `.big`, `.map`, `.w3d`, `.dds`, `.bik`, audio/video, binary, and C++/Rust build artifacts)
- `.githooks/pre-commit` and `.githooks/commit-msg` (installed via `git config core.hooksPath .githooks`)

**Ready for your OK on:**
1. **Retail Game Data (`~/Games/generalszh`) for Phase 0:** How would you like to populate `~/Games/generalszh` (interactive `steamcmd` download of Steam App `2732960`, or pointing `-assetPath` to an existing copy)?
2. **Proceed to Phase 0 & Phase 1:** Do you approve this architectural plan to begin Phase 0 (building the C++ reference baseline and replay harness) and Phase 1 (the `RandomValue` + `CRC` + `ExperienceTracker` translation spike)?
