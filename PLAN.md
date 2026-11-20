# Execution Plan: Zero Hour Engine Port to Rust

## 1. Objective & Milestones

Port the game engine of *Zero Hour* (~330k lines of C++ across `GameLogic` and `Common`) to Rust as a clean-room-structured, GPL-3.0-or-later public portfolio project, verified bit-exact against real game replays via the engine's built-in per-frame state checksums (`XferCRC`).

- **Stage 1 (Primary Milestone):** Headless Rust simulation that executes real *Zero Hour* replays at 30 logic frames/second (`LOGICFRAMES_PER_SECOND = 30`) and produces the exact same per-frame state checksums (`GameLogic::getCRC`) as the reference C++ build.
- **Stage 2 (Subsequent Milestone):** Native client and renderer (`wgpu`/Metal on macOS Apple Silicon) replacing `GameClient` and `WW3D2`.

---

## 2. Project Identity, Naming & Governance Rules

1. **Trademark-Clean Naming (EA GPLv3 Section 7 Compliance):**
   - No `"Command & Conquer"`, `"C&C"`, or `"cnc"` in the repository name, Cargo workspace/crate names, or binary names.
   - **Proposed GitHub Repository Name:** `zh-engine-rs` (started **private**; goes public only on owner instruction).
   - **Proposed Cargo Workspace Crates:**
     - `zh_common` — deterministic math (`GameMath` wrappers/port), PRNG (`RandomValue`), `CRC` / `XferCRC`, `BitFlags`, `NameKey`, `INI`, `Dict`, string and coordinate types.
     - `zh_logic` — `GameLogic`, `Object`, 222 `Module` implementations, `PartitionManager`, `Player`, `AI`, `Pathfinder`, `ScriptEngine`, `Weapon`, `Locomotor`.
     - `zh_ffi` — `staticlib` C-ABI bridge (`libzh_ffi.a`) linked into `z_gameengine` during the strangler migration.
     - `zh_sim` — standalone headless Rust replay runner binary (`zh-sim`) for Stage 1 completion.
   - *Note on local directory name:* If the local workspace folder is currently named `cnc-general-rust`, we recommend renaming the local folder to `zh-engine-rs` once convenient so local paths match the trademark-clean repository name.
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

## 3. Architectural Strategy: The Strangler Pattern & Rust State Model

### 3.1 Why the Strangler Pattern Works Here
A big-bang rewrite of 330k lines of stateful RTS logic will fail on its first CRC check at frame 1 with no way to isolate which of 1,000 functions diverged. Following Thyme's approach:
1. We start from a 100% passing C++ build of `fbraz3/GeneralsX` pinned at commit `84a7a20769aa5749f388b1f49bd542c79b3c33e8` (`GeneralsXZH`).
2. We link our Rust `staticlib` (`libzh_ffi.a`) directly into the `z_gameengine` CMake target.
3. We replace C++ functions and subsystems one module at a time with `extern "C"` calls into `zh_ffi` -> `zh_common` / `zh_logic`.
4. After **every single module** is switched over, we run the automated headless replay suite (`GeneralsXZH -headless -replay ... -jobs N`).
5. If a single replay mismatches on frame $F$, the bug is guaranteed to be inside the module just ported, and we use `-SaveDebugCRCPerFrame` + `-LogObjectCRCs` around frame $F$ to pinpoint the exact divergence within minutes.

### 3.2 Rust Data Ownership: Replacing `The*` Singletons and Pointer Webs
The original C++ engine relies on ~71 global singletons (`TheGameLogic`, `ThePartitionManager`, `ThePlayerList`, `TheAI`, `TheScriptEngine`, `TheThingFactory`, `TheWeaponStore`, `TheNameKeyGenerator`, etc.) and 352 manual `new`/`delete` sites (plus `MemoryPoolObject` intrusive allocators) in Zero Hour logic alone.

To make the Rust port idiomatic, safe, and free of raw-pointer aliasing traps while maintaining C++ FFI compatibility during the strangler phase:
1. **Explicit `SimContext` Struct:**
   - In Rust, all ~71 singletons become owned fields of a top-level `SimContext` (or domain sub-contexts: `RandomState`, `WorldObjects`, `PartitionGrid`, `Players`, `AiSystem`, `ScriptSystem`, `TemplateStores`).
   - During the FFI strangler phases (Phases 1–6), a single thread-local / global `SimContext` instance backs the `extern "C"` entry points (matching the single-threaded 30 Hz GameLogic execution model). In Phase 7, `SimContext` is instantiated directly by `zh-sim` with zero globals.
2. **Typed Handles & Generational/Slot Arenas Instead of Pointer Webs:**
   - Cross-object references in C++ already frequently use `ObjectID` (`i32`), `TeamID` (`i32`), `NameKeyType` (`i32`), and `ScienceType` (`i32`), alongside raw `Object*` and `Module*` pointers.
   - In Rust (`zh_logic`), objects live in a deterministic slot arena indexed by `ObjectId` (with an explicit intrusive doubly-linked list index preserving exact `m_objList` traversal order for `GameLogic::UPDATE()` and `GameLogic::getCRC()`).
   - Object modules (`BehaviorModule`, `UpdateModule`, `BodyModule`, `ContainModule`, `CollideModule`, `DieModule`, `UpgradeModule`, `SpecialPowerModule`, `CreateModule`, `DamageModule`, `DestroyModule`) are represented as strongly-typed Rust enums/structs stored in contiguous per-object or per-category slabs rather than 222 vtable heap allocations.
3. **Two-Way FFI Accessors During Transition:**
   - While an outer class (e.g. `Object`) still lives in C++ and an inner component (e.g. `ExperienceTracker`) lives in Rust, C++ holds either an opaque Rust handle/pointer or passes a thin Crepr struct/pointer across the `extern "C"` boundary, and Rust invokes small `extern "C"` callback shims (e.g. `zh_cxx_object_get_template_exp_required(obj_ptr, level)`) to read C++ state not yet migrated.
   - As outer classes move to Rust, those callback shims are deleted until the FFI boundary moves all the way out to `GameLogic::UPDATE()`.

---

## 4. Determinism Rules (Enforced Before Any Port Code)

Before Phase 1 translation starts, `docs/RUST_CONVENTIONS.md` will codify the 8 determinism rules identified in `docs/RESEARCH.md`:
1. **Explicit Wrapping Integer Ops:** Use `.wrapping_add()`, `.wrapping_sub()`, `.wrapping_mul()`, `.wrapping_neg()` for all 32-bit C++ `Int` / `UnsignedInt` arithmetic that can overflow.
2. **Strict Unfused `f32` Math:** Match `-ffp-contract=off`; never emit fused multiply-add (`mul_add`), never use fast-math flags, and preserve exact C++ parenthesization and left-to-right expression trees.
3. **Deterministic Transcendentals via `GameMath`:** Route all `sin`, `cos`, `tan`, `acos`, `asin`, `atan2`, `sqrt`, `floor`, `ceil`, `log`, `exp`, `pow` calls through `TheSuperHackers/GameMath` (`gm_*f`), never standard library `f32` transcendentals.
4. **C++-Exact Float-to-Int & Mixed Arithmetic:** Explicitly promote `i32` to `f32` for mixed `Int * Real` operations and truncate toward zero back to `i32`; use `gm_floorf` / `gm_ceilf` for `REAL_TO_INT_FLOOR` / `REAL_TO_INT_CEIL` (and preserve `fast_float_floor`/`ceil` behind `#[cfg(feature = "retail-crc")]`).
5. **Ordered Maps & Sets Only:** Replace `std::map` / `std::set` with `BTreeMap` / `BTreeSet` (never `HashMap` / `HashSet` with randomized hashes); key entity maps by deterministic ID (`ObjectID`, `NameKeyType`, `TeamID`), never by raw pointer address.
6. **Exact Intrusive List Traversal Order:** Preserve head/tail insertion semantics and deferred destruction queues (`m_objectsToDestroy`) so `m_objList` and `m_groupList` iterate in identical order during `UPDATE()` and `getCRC()`.
7. **Fixed Argument Evaluation Order:** Bind function arguments with side effects (especially `GameLogicRandomValue*`) to explicit `let` variables in Clang `arm64` left-to-right evaluation order before the call.
8. **Explicit Integer Widths & Signedness:** Map `Byte` -> `i8`, `UnsignedByte` -> `u8`, `Short` -> `i16`, `UnsignedShort` -> `u16`, `Int` -> `i32`, `UnsignedInt` -> `u32`, `Bool` -> `u8`, and match C++ signed/unsigned shift semantics (`>>`).

---

## 5. Phased Execution Roadmap

### Phase 0: Ground Truth & Replay Corpus (C++ Baseline)
**Goal:** Build unmodified `fbraz3/GeneralsX` at commit `84a7a20769aa5749f388b1f49bd542c79b3c33e8` on this Mac, assemble a multi-replay verification corpus, and prove 100% headless replay pass rate on C++.

- **Step 0.1 — Disk Budget & Retail Game Data (`~/Games/generalszh`):**
  - Check available disk space (currently ~13 GiB free; keep total build + asset footprint under 7 GiB).
  - If `~/Games/generalszh` does not exist, install Steam App `2732960` into `~/Games/generalszh` via `steamcmd +@sSteamCmdForcePlatformType windows` (owner provides interactive Steam login).
  - Never copy `.big` archives into build or repo directories; mount `~/Games/generalszh` read-only in place via `-assetPath ~/Games/generalszh` / `CNC_GENERALS_ZH_PATH`.
- **Step 0.2 — Build Reference C++ Engine (`GeneralsXZH` at `84a7a20`):**
  - Clone `fbraz3/GeneralsX` at `84a7a20769aa5749f388b1f49bd542c79b3c33e8` into an ignored local path (`vendor/GeneralsX`).
  - Configure CMake using preset `macos-vulkan` with `-DRTS_BUILD_OPTION_DEBUG=OFF -DRTS_BUILD_GENERALS=OFF -DRTS_BUILD_CORE_TOOLS=OFF` and build target `z_generals` (`GeneralsXZH`).
- **Step 0.3 — Assemble Replay Corpus:**
  - Fetch `generalsx-project/GeneralsXReplays` (6 cross-platform Zero Hour replays: `linux_2p_vanilla_map.rep`, `linux_2p_custom_map.rep`, `linux_6p_custom_map.rep`, `macos_2p_vanilla_map.rep`, `macos_2p_custom_map.rep`, `macos_6p_custom_map.rep` + required custom maps) and install into `~/Library/Application Support/GeneralsX/GeneralsZH/{Maps,Replays}/`.
  - Record additional local skirmish replays with `GeneralsXZH` covering USA, China, and GLA generals, superweapons, air/ground combat, veterancy promotions, and base power brownouts; store self-recorded `.rep` files in `replays/`.
  - Also test `TheSuperHackers/GeneralsReplays` (10 legacy 1.04 retail replays) with `-ignoreReplaySyncErrors` to document their exact behavior under Regime A vs Regime B in `docs/BASELINE.md`.
- **Step 0.4 — Baseline Verification Harness (`scripts/check_replays.sh`):**
  - Run `GeneralsXZH -headless -replay "*.rep" -jobs 4` across the corpus.
  - Record wall-clock runtime, total frame count, frames/sec throughput, and pass/fail exit codes in `docs/BASELINE.md`.
  - **Gate 0:** All corpus replays pass (`exit 0`, zero CRC mismatches) on stock C++.

---

### Phase 1: Translation Spike (Go / No-Go Gate)
**Goal:** Validate the multi-model translation workflow, FFI build integration, and determinism conventions on two real modules before committing to the full 330k-line campaign.

- **Step 1.1 — Rust Workspace, Conventions & FFI Seam (Lead Model):**
  - Author `docs/RUST_CONVENTIONS.md` with concrete before/after C++ -> Rust examples for all 8 determinism traps.
  - Initialize the Cargo workspace (`crates/zh_common`, `crates/zh_logic`, `crates/zh_ffi`) producing `libzh_ffi.a`.
  - Wire `libzh_ffi.a` into `GeneralsMD/Code/GameEngine/CMakeLists.txt` (`z_gameengine`).
  - Define the C-ABI header (`zh_ffi.h`) and thin C++ forwarding hooks for the two spike modules.
- **Step 1.2 — Autonomous Module Translation (Cost-Optimized Model):**
  - Provide the cost-optimized model only `docs/RUST_CONVENTIONS.md`, the C++ source/header pair, and the FFI signature contract, and have it translate:
    1. **`RandomValue` + `CRC`** (`Core/GameEngine/Source/Common/RandomValue.cpp`, `crc.h`/`crc.cpp`, ~320 lines): all 3 seed banks (`Audio`, `Client`, `Logic`), `InitRandom`, `GetGameLogicRandomSeed`, `GetGameLogicRandomSeedCRC`, `GetGameLogicRandomValue`, `GetGameLogicRandomValueReal`, `*Unchanged` variants, and `GameLogicRandomVariable`.
    2. **`ExperienceTracker`** (`GeneralsMD/Code/GameEngine/Source/GameLogic/Object/ExperienceTracker.cpp`, ~280 lines): veterancy state, float-scaled XP accumulation (`experienceGain * m_experienceScalar`), sink forwarding (`m_experienceSink`), level threshold promotion, and `ExperienceTracker::crc` / `xfer`.
- **Step 1.3 — Verification & Telemetry Report (`docs/PHASE1_SPIKE_REPORT.md`):**
  - Compile the hybrid C++/Rust `GeneralsXZH` binary and run `scripts/check_replays.sh` + Rust unit tests (`cargo test`).
  - Record:
    - Input & output tokens consumed by the cost-optimized model per line of C++ translated;
    - First-pass compilation success and first-pass replay CRC pass rate;
    - Exact count and category of fixes (if any) required from the lead model.
  - **Gate 1 (Owner Go / No-Go Checkpoint):** Review `docs/PHASE1_SPIKE_REPORT.md` with the owner before scaling to Phase 2+.

---

### Phase 2: Foundation, Math & Serialization (`Common` Layer, ~40k LOC)
**Goal:** Move the core deterministic primitives and serialization backbone into `zh_common`.
- **Batch 2.1 — Checksums & Serialization:** `Xfer`, `XferCRC`, `XferDeepCRC`, `Snapshot` trait, `BitFlags<N>`.
- **Batch 2.2 — Deterministic Math & Geometry:** FFI bindings + Rust wrappers for `TheSuperHackers/GameMath` (`gm_*f`), `Coord2D`, `Coord3D`, `ICoord2D`, `ICoord3D`, `Region2D`, `Region3D`, `Matrix3D`, `Vector3`, `GeometryInfo`, `BezierSegment`, `DiscreteCircle`.
- **Batch 2.3 — Identifiers, Strings & Containers:** `NameKeyGenerator`, `AsciiString` / `UnicodeString` interoperability views, `Dict`, `StateMachine` base framework.
- **Batch 2.4 — INI & Data Scanners:** `INI` tokenizers and numeric/flag parsers (`scanInt`, `scanReal`, `scanPercentToReal`, `scanIndexList`, `BitFlagsIO`).
- **Gate 2:** Full replay suite green after each batch.

---

### Phase 3: Core RTS Domain Models & Leaf Trackers (~35k LOC)
**Goal:** Port self-contained gameplay value trackers and template stores that directly contribute to `Object::crc` and `Player::crc`.
- **Batch 3.1 — Economy & Player Trackers:** `Money`, `Energy` (production, consumption, brownout triggers, sabotage frame checks, `Energy::crc`), `ScoreKeeper`, `Handicap`, `MissionStats`, `AcademyStats`.
- **Batch 3.2 — Tech Tree & Rules:** `Science`, `RankInfo`, `Upgrade` masks/templates, `SpecialPower` masks/templates, `ProductionPrerequisite`, `DisabledTypes`, `KindOf`, `ObjectStatusTypes`.
- **Batch 3.3 — Combat & Movement Leaf Logic:** `Armor` / `ArmorTemplate` / `ArmorSet`, `WeaponBonusConditionFlags`, `FiringTracker`, `TurretAI`, `Locomotor` / `LocomotorSet` physics math.
- **Gate 3:** Full replay suite green after each batch.

---

### Phase 4: Spatial & World Subsystems (~30k LOC)
**Goal:** Port map geometry, triggers, and the spatial partition grid (`MARKER:ThePartitionManager` in `GameLogic::getCRC`).
- **Batch 4.1 — Map & Terrain Logic:** `TerrainLogic` (heightmap queries, cliff/water/bridge queries, `fast_float_floor` vs `gm_floorf` parity), `PolygonTrigger`, `SidesList`, `CrateSystem`.
- **Batch 4.2 — Spatial Partition & Fog of War:** `PartitionManager`, `PartitionCell`, `CellAndObjectIntersection`, `SightingInfo`, shroud look/unlook queues (`m_pendingUndoShroudReveals`), geometric range/cone filters, `PartitionManager::crc`, `GhostObject`.
- **Gate 4:** Full replay suite green (`MARKER:ThePartitionManager` checksums verified bit-exact).

---

### Phase 5: `Object`, `Weapon`, and the 222 `Module` Classes (~135k LOC)
**Goal:** Strangle `Object` and its 11 module categories (`MARKER:Objects` in `GameLogic::getCRC`) inward into the Rust `WorldObjects` arena.
- **Batch 5.1 — Leaf Modules (28 files):** `Destroy` (1), `Damage` (3), `Body` (8: `ActiveBody`, `StructureBody`, `HighlanderBody`, `HiveStructureBody`, `ImmortalBody`, `InactiveBody`, `UndeadBody`), `Create` (8), `Helper` (8).
- **Batch 5.2 — Event & Modifier Modules (64 files):** `Die` (12), `SpecialPower` (11), `Upgrade` (21), `Collide` (20, including `CrateCollide`).
- **Batch 5.3 — Container & Behavior Modules (43 files):** `Contain` (13: `OpenContain`, `TransportContain`, `GarrisonContain`, `OverlordContain`, `HelixContain`, `TunnelContain`, `InternetHackContain`, etc.), `Behavior` (30: `AutoHealBehavior`, `BridgeBehavior`, `MinefieldBehavior`, `ParkingPlaceBehavior`, `PropagandaTowerBehavior`, `RebuildHoleBehavior`, `SlowDeathBehavior`, `SpawnBehavior`, etc.).
- **Batch 5.4 — Update Modules (86 files, split into 5 sub-batches of ~17 files each):**
  - 5.4a: Simple timer/lifetime/stealth/detector/radar updates.
  - 5.4b: Physics, projectile, missile, and laser/beam updates (`PhysicsUpdate`, `MissileAIUpdate`, `LaserUpdate`, `FireWeaponUpdate`).
  - 5.4c: Production, dock, supply, repair, and worker updates (`ProductionUpdate`, `SupplyTruckAIUpdate`, `WorkerAIUpdate`, `RailedTransportDockUpdate`).
  - 5.4d: Aircraft, transport, and vehicle AI updates (`JetAIUpdate`, `ChinookAIUpdate`, `DeployStyleAIUpdate`, `AssaultTransportAIUpdate`).
  - 5.4e: Base `AIUpdate` state machine integration, superweapon/OCL updates, and topple/structure collapse updates.
- **Batch 5.5 — `Weapon`, `WeaponSet`, `ObjectCreationList`, and `Object` Core:** `Weapon.cpp` (`Weapon::crc`), `WeaponSet.cpp`, `ObjectCreationList.cpp`, `Object.cpp` (`Object::crc`), migrating `m_objList` ownership into Rust.
- **Gate 5:** Full replay suite green after every sub-batch (`MARKER:Objects` verified bit-exact).

---

### Phase 6: `Player`, `Team`, `Pathfinder`, `AI`, and `ScriptEngine` (~70k LOC)
**Goal:** Port player management, pathfinding, tactical/skirmish AI, and map scripting (`MARKER:ThePlayerList` and `MARKER:TheAI` in `GameLogic::getCRC`).
- **Batch 6.1 — Players & Teams:** `Player`, `PlayerList` (`PlayerList::crc`), `Team`, `Squad`, `BuildAssistant`, `ResourceGatheringManager`, `ActionManager`.
- **Batch 6.2 — Pathfinder:** `Core/GameEngine/Source/GameLogic/AI/Pathfind*.cpp` + `GeneralsMD/.../AIPathfind.cpp` (A* grid, zones, obstacles, bridge transitions, wall pieces, `Pathfinder::crc`).
- **Batch 6.3 — AI & AIGroup:** `AI`, `AIGroup` (`AIGroup::crc`), `AIPlayer`, `AISkirmishPlayer`, `AIStateMachine`, `AIStates`, `AITNGuard` (`AI::crc`).
- **Batch 6.4 — ScriptEngine:** `Scripts.cpp`, `ScriptEngine.cpp`, `ScriptConditions.cpp`, `ScriptActions.cpp`, `SequentialScript.cpp`, `VictoryConditions.cpp`.
- **Gate 6:** Full replay suite green (`MARKER:ThePlayerList` and `MARKER:TheAI` verified bit-exact).

---

### Phase 7: Top-Level `GameLogic` Loop & Standalone Stage 1 Milestone (~20k LOC)
**Goal:** Port `GameLogic.cpp`, `Recorder.cpp` (replay parser & CRC verifier), `ArchiveFileSystem` (`.big` reader), and `MapReader` so the simulation runs as a standalone 100% pure-Rust executable (`zh-sim`) with zero C++ linked.
- **Batch 7.1 — `GameLogic::UPDATE()` & `GameLogic::getCRC()` in Rust:** Command stream dispatch, sleep/wake update priority queue, object creation/destruction queues, full state CRC calculation.
- **Batch 7.2 — Native `.big`, `.ini`, `.map`, and `.rep` Loader in `zh_common`:** Read retail archives and `.rep` streams directly in Rust.
- **Batch 7.3 — Standalone `zh-sim` Binary Verification:**
  - Run `zh-sim --headless --replay <corpus> --jobs N` with zero C++ dependencies.
  - Verify that every frame of every replay matches the C++ reference CRCs 100% bit-exact, and benchmark pure-Rust simulation speed (frames/sec) against the Phase 0 C++ baseline.

---

### Stage 2 (Future): Native Client & `wgpu`/Metal Renderer
Once Stage 1 is complete and signed off:
- Port `.w3d` mesh/hierarchy/animation loader, `.dds`/`.tga` texture pipeline, terrain heightmap renderer, particle system, and UI (`wnd`) to a pure-Rust `wgpu` (Metal backend on macOS) client crate (`zh_client`) driving `zh_logic` at 30 Hz logic / unlocked render framerate.

---

## 6. First Deliverable Status & Sign-Off Questions for Owner

This repository now contains the First Deliverable:
- `PLAN.md` (this execution plan)
- `docs/RESEARCH.md` (technical reference, module inventory, CRC pipeline, determinism traps, disk budget analysis)
- `LICENSE.md` (GPL-3.0-or-later + EA Section 7 Additional Terms + Notice of Modification)
- `.gitignore` (blocks all retail `.big`, `.map`, `.w3d`, `.dds`, `.bik`, audio/video, binary, and C++/Rust build artifacts)
- `.githooks/pre-commit` and `.githooks/commit-msg` (installed via `git config core.hooksPath .githooks`, enforcing commit email `5305832+chayuto@users.noreply.github.com`, blocking game assets, and blocking attribution trailers or assistant names)

**Before we begin Phase 0 / Phase 1 execution, please confirm:**
1. **Repository & Crate Naming:** Are you happy with **`zh-engine-rs`** as the private GitHub repository name (and `zh_common`, `zh_logic`, `zh_ffi`, `zh-sim` for the crates/binary), and would you like to keep the local folder path as `/Users/chayut/repos/cnc-general-rust` or rename it to `/Users/chayut/repos/zh-engine-rs`?
2. **Disk Space & Game Assets (`~/Games/generalszh`):** Since free disk space is currently ~13 GiB and `~/Games/generalszh` is not installed yet, how would you like to populate `~/Games/generalszh` for Phase 0 (e.g., interactive `steamcmd` download of App `2732960`, or copying/mounting from an external drive/archive)?
3. **Plan Sign-Off:** Do you approve this plan and the Phase 1 spike scope (`RandomValue` + `CRC` + `ExperienceTracker`) to proceed to Phase 0?
