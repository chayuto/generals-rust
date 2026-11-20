# Research & Technical Reference: Zero Hour Engine Port to Rust

This document records verified technical facts from the reference C++ codebase, build system, replay verification oracle, and host environment for porting the *Zero Hour* game logic simulation to Rust.

---

## 1. Upstream Lineage & Legal Constraints

### 1.1 Source Provenance
- **Original GPLv3 Release:** Electronic Arts Inc. released the source code for *Generals* and *Zero Hour* on 27 February 2025 under **GPL-3.0-or-later with Additional Terms per GNU GPL Section 7** (`electronicarts/CnC_Generals_Zero_Hour`, archived).
- **Community Mainline:** `TheSuperHackers/GeneralsGameCode` unifies shared engine subsystems under `Core/` while preserving game-specific code under `Generals/` and `GeneralsMD/`.
- **Native macOS / Linux Port (Reference Oracle):** `fbraz3/GeneralsX` adapts the community mainline to run natively on 64-bit Apple Silicon (`arm64`) and Linux (`x86_64`) using SDL3, DXVK-native + MoltenVK (Vulkan-to-Metal), GLM, and MiniAudio.
- **Pinned Reference Commit:** `84a7a20769aa5749f388b1f49bd542c79b3c33e8` (committed 15 November 2026 on `fbraz3/GeneralsX` `main`).

### 1.2 GPL-3.0-or-later & EA Section 7 Additional Terms
Every source file and binary distribution must comply with both GNU GPL v3 and the Electronic Arts Section 7 terms included in `LICENSE.md`:
1. **No EA Trademarks in Identifiers:** No rights are granted to `"Command & Conquer"`, `"C&C"`, or `"cnc"`. Neither the repository name, Cargo workspace/crate names, nor binary names may contain `"Command & Conquer"`, `"C&C"`, or `"cnc"`. Compatibility with *Zero Hour* data files and replays may only be described in prose documentation.
2. **Marked as Modified Version:** Per GPLv3 Section 5(a) and EA Section 7, all distributions and repository documentation must clearly mark the work as a modified version and must not misrepresent it as the original program or claim affiliation with Electronic Arts Inc.
3. **Preservation of Notices:** All propagation or conveyance must retain `LICENSE.md` including the EA copyright notice, indemnification clause, and warranty disclaimer.
4. **Strict Isolation:**
   - No game assets (`.big`, `.bik`, `.map`, `.tga`, `.w3d`, `.dds`, `.csf`, `.mp3`, `.wav`, retail binaries) may ever enter git history.
   - No dependencies, code, submodules, or crates from the owner's other repositories.
   - No upstream pull requests, issues, or comments to `fbraz3/GeneralsX` or `TheSuperHackers/GeneralsGameCode`.

---

## 2. Reference Codebase Structure & Size (at `84a7a20`)

### 2.1 High-Level Line Counts

| Layer / Directory | Approx. Lines | Role in Stage 1 |
|---|---:|---|
| `GeneralsMD/Code/GameEngine/{Include,Source}/GameLogic` | ~210,000 | Zero Hour simulation, objects, AI, 222 modules, scripts |
| `GeneralsMD/Code/GameEngine/{Include,Source}/Common` | ~46,000 | Zero Hour common runtime, `Recorder`, `Player`, `Thing`, `GlobalData` |
| `Core/GameEngine/{Include,Source}/{GameLogic,Common}` | ~73,000 | Unified core simulation (`Pathfinder`, `ReplaySimulation`, `INI`, `XferCRC`, `RandomValue`) |
| **Stage 1 Logic + Common Total** | **~330,000** | **Target scope for Stage 1 headless bit-exact simulation** |
| `Core` + `GeneralsMD` `GameClient` | ~56,000 | Visual-only client, UI, drawable representations (stubbed/headless in Stage 1; Stage 2 target) |
| `Core/Libraries` (`WWVegas`, `WWMath`, `WW3D2`, etc.) | ~214,000 | Math primitives (`Matrix3D`, `Vector3`) used by logic; rendering used in Stage 2 |
| `Core/GameEngineDevice` | ~82,000 | OS/Graphics/Audio device drivers (SDL3, DXVK/MoltenVK, MiniAudio) |

### 2.2 Detailed Inventory of `GameLogic` and `Common` (871 Files, 9.65 MB)

Inspection of the git tree at `84a7a20769aa5749f388b1f49bd542c79b3c33e8` yields the exact module distribution across `Core/GameEngine` and `GeneralsMD/Code/GameEngine`:

| Subsystem Path | Files | Size (Bytes) | Key Components |
|---|---:|---:|---|
| **Unified `Core/GameEngine`** | | | |
| `Core/GameEngine/Include/Common` | 73 | 508,597 | `crc.h`, `RandomValue.h`, `INI.h`, `Xfer.h`, `XferCRC.h`, `GameDefines.h`, `MessageStream.h`, `Snapshot.h` |
| `Core/GameEngine/Source/Common/(root)` | 23 | 283,958 | `CommandLine.cpp`, `crc.cpp`, `RandomValue.cpp`, `ReplaySimulation.cpp`, `MessageStream.cpp`, `Dict.cpp` |
| `Core/GameEngine/Source/Common/INI` | 28 | 166,027 | `INI.cpp` + type scanners (`from_chars` / range fallbacks) |
| `Core/GameEngine/Source/Common/System` | 29 | 470,557 | `Xfer.cpp`, `XferCRC.cpp`, `GameMemory.cpp`, `ArchiveFileSystem.cpp`, `LocalFileSystem.cpp` |
| `Core/GameEngine/Source/Common/{Audio,Bezier,Diagnostic,RTS,Thing}` | 13 | 232,716 | `Thing.cpp`, `PlayerTemplate.cpp`, `BezierSegment.cpp` |
| `Core/GameEngine/Include/GameLogic` | 19 | 122,401 | `Pathfinder/*.h`, `TerrainLogic.h`, `PolygonTrigger.h`, `SidesList.h` |
| `Core/GameEngine/Source/GameLogic/AI` | 10 | 344,792 | Unified `Pathfinder` (`Pathfind.cpp`, `PathfindCell.cpp`, `PathfindZone.cpp`, `PathfindServices.cpp`) |
| `Core/GameEngine/Source/GameLogic/Map` | 3 | 143,456 | `TerrainLogic.cpp`, `PolygonTrigger.cpp`, `SidesList.cpp` |
| `Core/GameEngine/Source/GameLogic/System` | 5 | 99,323 | `CrateSystem.cpp`, `RankInfo.cpp`, `Science.cpp`, `SpecialPower.cpp`, `Upgrade.cpp` |
| **Zero Hour `GeneralsMD/Code/GameEngine`** | | | |
| `GeneralsMD/.../Include/Common` | 73 | 483,360 | `Recorder.h`, `Player.h`, `PlayerList.h`, `GlobalData.h`, `Money.h`, `Energy.h`, `BitFlags.h`, `StateMachine.h` |
| `GeneralsMD/.../Source/Common/(root)` | 12 | 298,089 | `Recorder.cpp`, `GlobalData.cpp`, `GameEngine.cpp`, `StateMachine.cpp`, `NameKeyGenerator.cpp`, `BitFlags.cpp` |
| `GeneralsMD/.../Source/Common/RTS` | 15 | 425,507 | `Player.cpp`, `PlayerList.cpp`, `Team.cpp`, `Money.cpp`, `Energy.cpp`, `ScoreKeeper.cpp`, `BuildAssistant.cpp`, `ActionManager.cpp` |
| `GeneralsMD/.../Source/Common/System` | 16 | 298,166 | `KindOf.cpp`, `DisabledTypes.cpp`, `FunctionLexicon.cpp`, `ModuleFactory.cpp`, `GameState.cpp` |
| `GeneralsMD/.../Source/Common/Thing` | 5 | 73,033 | `ThingTemplate.cpp`, `ThingFactory.cpp`, `Module.cpp` |
| `GeneralsMD/.../Include/GameLogic/(root)` | 37 | 538,072 | `GameLogic.h`, `Object.h`, `PartitionManager.h`, `Weapon.h`, `Locomotor.h`, `AI.h`, `AIStateMachine.h`, `Scripts.h` |
| `GeneralsMD/.../Include/GameLogic/Module` | 222 | 1,108,811 | All 222 object module headers (`*Update.h`, `*Behavior.h`, `*Body.h`, `*Contain.h`, `*Collide.h`, `*Die.h`, `*Upgrade.h`, etc.) |
| `GeneralsMD/.../Source/GameLogic/System` | 1 | 180,016 | `GameLogic.cpp` (top-level frame tick `UPDATE()`, object lifecycle, `getCRC()`) |
| `GeneralsMD/.../Source/GameLogic/ScriptEngine` | 5 | 884,145 | `ScriptEngine.cpp`, `Scripts.cpp`, `ScriptActions.cpp`, `ScriptConditions.cpp`, `SequentialScript.cpp`, `VictoryConditions.cpp` |
| `GeneralsMD/.../Source/GameLogic/AI` | 11 | 737,366 | `AI.cpp`, `AIGroup.cpp`, `AIPathfind.cpp`, `AIPlayer.cpp`, `AISkirmishPlayer.cpp`, `AIStateMachine.cpp`, `AIStates.cpp`, `AITNGuard.cpp`, `Squad.cpp`, `TurretAI.cpp` |
| `GeneralsMD/.../Source/GameLogic/Object/(root)` | 12 | 794,901 | `Object.cpp`, `PartitionManager.cpp`, `Weapon.cpp`, `WeaponSet.cpp`, `Locomotor.cpp`, `LocomotorSet.cpp`, `Armor.cpp`, `ExperienceTracker.cpp`, `FiringTracker.cpp`, `GhostObject.cpp`, `ObjectCreationList.cpp`, `SimpleObjectIterator.cpp` |
| `GeneralsMD/.../Source/GameLogic/Object/Update` | 86 | 1,821,419 | `AIUpdate.cpp`, `PhysicsUpdate.cpp`, `LaserUpdate.cpp`, `MissileAIUpdate.cpp`, `SupplyTruckAIUpdate.cpp`, `ChinookAIUpdate.cpp`, `FireWeaponUpdate.cpp`, etc. |
| `GeneralsMD/.../Source/GameLogic/Object/Behavior` | 30 | 498,801 | `AutoHealBehavior.cpp`, `BridgeBehavior.cpp`, `CountermeasuresBehavior.cpp`, `MinefieldBehavior.cpp`, `ParkingPlaceBehavior.cpp`, `PropagandaTowerBehavior.cpp`, `RebuildHoleBehavior.cpp`, `SlowDeathBehavior.cpp`, `SpawnBehavior.cpp`, `TechBuildingBehavior.cpp`, etc. |
| `GeneralsMD/.../Source/GameLogic/Object/Contain` | 13 | 323,882 | `OpenContain.cpp`, `TransportContain.cpp`, `GarrisonContain.cpp`, `OverlordContain.cpp`, `HelixContain.cpp`, `TunnelContain.cpp`, `InternetHackContain.cpp`, etc. |
| `GeneralsMD/.../Source/GameLogic/Object/Collide` | 20 | 135,281 | `CrateCollide/*`, `SquishCollide.cpp`, `FireWeaponCollide.cpp` |
| `GeneralsMD/.../Source/GameLogic/Object/Upgrade` | 21 | 115,182 | `ArmorUpgrade.cpp`, `CommandSetUpgrade.cpp`, `LocomotorSetUpgrade.cpp`, `ObjectCreationUpgrade.cpp`, `PowerPlantUpgrade.cpp`, `RadarUpgrade.cpp`, `SubObjectsUpgrade.cpp`, `WeaponBonusUpgrade.cpp`, `WeaponSetUpgrade.cpp`, etc. |
| `GeneralsMD/.../Source/GameLogic/Object/SpecialPower` | 11 | 107,057 | `SpecialPowerModule.cpp`, `OCLSpecialPower.cpp`, `CashHackSpecialPower.cpp`, `DefectorSpecialPower.cpp`, `SpyVisionSpecialPower.cpp`, etc. |
| `GeneralsMD/.../Source/GameLogic/Object/Body` | 8 | 92,647 | `BodyModule.cpp`, `ActiveBody.cpp`, `HighlanderBody.cpp`, `HiveStructureBody.cpp`, `ImmortalBody.cpp`, `InactiveBody.cpp`, `StructureBody.cpp`, `UndeadBody.cpp` |
| `GeneralsMD/.../Source/GameLogic/Object/Die` | 12 | 71,237 | `CreateObjectDie.cpp`, `CrushDie.cpp`, `DamDie.cpp`, `DestroyDie.cpp`, `EjectPilotDie.cpp`, `FXListDie.cpp`, `KeepObjectDie.cpp`, `RebuildHoleExposeDie.cpp`, `SpecialPowerCompletionDie.cpp`, `UpgradeDie.cpp` |
| `GeneralsMD/.../Source/GameLogic/Object/Create` | 8 | 37,185 | `CreateModule.cpp`, `GrantUpgradeCreate.cpp`, `LockWeaponCreate.cpp`, `PreorderCreate.cpp`, `SpecialPowerCreate.cpp`, `SupplyCenterCreate.cpp`, `SupplyWarehouseCreate.cpp`, `VeterancyGainCreate.cpp` |
| `GeneralsMD/.../Source/GameLogic/Object/Helper` | 8 | 35,399 | `ObjectDefectionHelper.cpp`, `ObjectRepulsorHelper.cpp`, `ObjectSMCHelper.cpp`, `ObjectSpecialPowerHelper.cpp`, `ObjectWeaponStatusHelper.cpp`, `StatusDamageHelper.cpp`, `SubdualDamageHelper.cpp`, `TempWeaponBonusHelper.cpp` |
| `GeneralsMD/.../Source/GameLogic/Object/Damage` | 3 | 22,401 | `BoneFXDamage.cpp`, `TransitionDamageFX.cpp` |
| `GeneralsMD/.../Source/GameLogic/Object/Destroy` | 1 | 3,201 | `DestroyModule.cpp` |

### 2.3 CMake Target Graph & FFI Link Seam
In `fbraz3/GeneralsX`:
1. `Core/GameEngine/CMakeLists.txt` defines two `INTERFACE` libraries:
   - `corei_gameengine_private`: propagates all `Core/GameEngine` `.cpp` sources and precompiled headers into game targets.
   - `corei_gameengine_public`: propagates `Core/GameEngine/Include` and dependencies (`core_compression`, `d3d8lib`, `gamespy::gamespy`, `stlport`, `gamemath`).
2. `GeneralsMD/Code/GameEngine/CMakeLists.txt` defines `add_library(z_gameengine STATIC)`, compiling both `GeneralsMD/Code/GameEngine` sources and `corei_gameengine_private` sources with `RTS_ZEROHOUR=1` (via `zi_always`).
3. `GeneralsMD/Code/Main/CMakeLists.txt` defines `add_executable(z_generals)` with `OUTPUT_NAME GeneralsXZH`, linking `z_gameengine`, `z_gameenginedevice`, `z_wwvegas`, `d3dx8`, and `sdl3lib`.
4. **Strangler Link Seam:** Linking a Rust `staticlib` into `z_gameengine` (`target_link_libraries(z_gameengine PUBLIC /path/to/libzh_ffi.a)`) allows individual C++ functions or class methods in `z_gameengine` to forward directly to `extern "C"` Rust symbols with zero dynamic-library overhead.

---

## 3. The Replay & State-Checksum Oracle

### 3.1 Headless Simulation Loop
The simulation runs at a fixed rate of **30 logic frames per second** (`LOGICFRAMES_PER_SECOND = WWSyncPerSecond = 30`).
In `Core/GameEngine/Source/Common/ReplaySimulation.cpp`, `-headless -replay <pattern>` executes `ReplaySimulation::simulateReplaysInThisProcess`:

```cpp
if (TheRecorder->simulateReplay(filename)) {
    while (TheRecorder->isPlaybackInProgress()) {
        TheGameClient->updateHeadless();
        TheGameLogic->UPDATE();
        if (TheRecorder->sawCRCMismatch()) {
            numErrors++;
            break;
        }
    }
}
```

When `-jobs N` is passed on the command line, `ReplaySimulation::simulateReplays` spawns up to `N` parallel worker processes (`WorkerProcess`), each running `-headless -replay <single_file>`, and returns exit code `0` if and only if all replays complete with zero CRC mismatches.

### 3.2 Command-Line Flags (`Core/GameEngine/Source/Common/CommandLine.cpp`)

- **Startup & Asset Flags (`paramsForStartup`):**
  - `-headless`: Runs without creating a window, graphics context, input device, or audio output.
  - `-replay <file_or_glob>`: Queues replay file(s) from the user data `Replays/` folder (repeatable, supports wildcards).
  - `-jobs <N>`: Runs queued replays across `1..N` worker processes in parallel.
  - `-assetPath <dir>` / `-generalsAssetPath <dir>`: Points the archive file system directly to an external read-only directory containing the retail `.big` archives (can also be supplied via `CNC_GENERALS_ZH_PATH`).
  - `-looseWins`: Forces loose files on disk to override `.big` archives.
- **Engine Init & CRC Flags (`paramsForEngineInit`):**
  - `-ignoreReplaySyncErrors`: Logs CRC mismatches (`InGame:<hex> Replay:<hex> Frame:<N>`) and continues playback instead of stopping on the first mismatch.
  - `-ReplayCRCInterval <N>` (`#ifdef DEBUG_CRC`): Sets interval between CRCs written during single-player/skirmish recording (default `REPLAY_CRC_INTERVAL = 100` frames).
  - `-NetCRCInterval <N>` (`#ifdef DEBUG_CRC`): Sets CRC interval for multiplayer sync checks.
  - `-DebugCRCFromFrame <N>` / `-DebugCRCUntilFrame <N>` (`#ifdef DEBUG_CRC`): Enables granular per-frame `CRCGEN_LOG` / `CRCDEBUG_LOG` output across the specified frame window.
  - `-SaveDebugCRCPerFrame <dir>` (`#ifdef DEBUG_CRC`): Writes per-frame text CRC debug logs into `<dir>/` for frame-by-frame diffing between two runs.
  - `-LogObjectCRCs` (`#ifdef DEBUG_CRC`): Logs individual object and weapon CRCs inside `Object::crc` and `Weapon::crc`.
  - `-VerifyClientCRC` (`#ifdef DEBUG_CRC`): Asserts that `GameClient::update` does not mutate any `GameLogic` state or RNG seed.

### 3.3 Exact CRC Algorithms (`crc.h` and `XferCRC.cpp`)

The engine uses two related 32-bit rolling rotate-and-add checksum accumulators (not standard polynomial CRC-32):

1. **`CRC` (`Core/GameEngine/Include/Common/crc.h`) — Byte-wise accumulator:**
   Used by `GetGameLogicRandomSeedCRC()` over the 24-byte `theGameLogicSeed[6]` array:
   ```rust
   pub fn compute_crc_bytes(mut crc: u32, buf: &[u8]) -> u32 {
       for &b in buf {
           crc = crc.rotate_left(1).wrapping_add(b as u32);
       }
       crc
   }
   ```
2. **`XferCRC` (`Core/GameEngine/Source/Common/System/XferCRC.cpp`) — 32-bit word-wise accumulator:**
   Used by `GameLogic::getCRC()` for all `Snapshot::crc` traversal:
   - Processes `dataSize / 4` little-endian `u32` words:
     - When `RETAIL_COMPATIBLE_CRC == 0` (GeneralsX cross-platform default):
       ```rust
       crc = crc.rotate_left(1).wrapping_add(word);
       ```
     - When `RETAIL_COMPATIBLE_CRC == 1` (legacy VC6 retail mode):
       ```rust
       crc = crc.rotate_left(1).wrapping_add(word.swap_bytes());
       ```
   - Processes `dataSize & 3` trailing bytes (`1..=3` bytes) packed little-endian into `val = c[0] + (c[1] << 8) + (c[2] << 16)`:
     ```rust
     crc = crc.rotate_left(1).wrapping_add(val);
     ```
   - `XferCRC::getCRC()` returns `m_crc` (or `m_crc.swap_bytes()` when `RETAIL_COMPATIBLE_CRC == 1`).

### 3.4 Exact Per-Frame State Hashing Sequence (`GameLogic::getCRC`)

In `GeneralsMD/Code/GameEngine/Source/GameLogic/System/GameLogic.cpp:4360`, `GameLogic::getCRC` feeds the following state into `XferCRC` in strict deterministic order:

1. **Marker `"MARKER:Objects"`** (`xferAsciiString`: `u16` length `14` + 14 ASCII bytes).
2. **All `Object` instances in `m_objList` linked-list order** (`Object::crc` in `Object.cpp:3976`):
   - `m_privateStatus` (`u8`)
   - ` *getTransformMatrix()` (`Matrix3D`, 3x4 `f32` = 48 bytes)
   - `m_id` (`ObjectID` = `i32`, 4 bytes)
   - `m_objectUpgradesCompleted` (`BitFlags<128>` = 16 bytes when `RETAIL_COMPATIBLE_CRC == 0`; `sizeof(Int64)` = 8 bytes when `RETAIL_COMPATIBLE_CRC == 1`)
   - If `m_experienceTracker != nullptr`, `ExperienceTracker::crc`:
     - `m_currentExperience` (`i32`, 4 bytes)
     - `m_currentLevel` (`VeterancyLevel` enum = `i32`, 4 bytes)
     - `m_isTrainable` (`Bool` = `u8`, 1 byte — only when `RETAIL_COMPATIBLE_CRC == 0`)
   - `getBodyModule()->getHealth()` (`f32`, 4 bytes)
   - `m_weaponBonusCondition` (`u32`, 4 bytes)
   - `getBodyModule()->getDamageScalar()` (`f32`, 4 bytes)
   - For each weapon slot `0..WEAPONSLOT_COUNT` (`3`): if `thisWeapon != nullptr`, `Weapon::crc(xfer)`.
3. **`GetGameLogicRandomSeedCRC()`** (`u32`, 4 bytes — byte-wise `CRC` of `theGameLogicSeed[6]`).
4. **Marker `"MARKER:ThePartitionManager"`** + `ThePartitionManager->crc(xfer)`:
   - Loops `i` from `0` to `m_totalCellCount - 1`, calling `m_cells[i].crc(xfer)` (`PartitionCell` shroud/sight and threat/value data).
5. **Marker `"MARKER:ThePlayerList"`** + `ThePlayerList->crc(xfer)`:
   - `m_playerCount` (`i32`) followed by `m_players[0..m_playerCount]->crc(xfer)` (money, energy, score, upgrades, sciences, kindof masks, build lists, team prototypes).
6. **Marker `"MARKER:TheAI"`** + `TheAI->crc(xfer)`:
   - `m_pathfinder->crc(xfer)`
   - Each `TAiData` node prefixed by `"MARKER:TAiData"`
   - Each non-null `AIGroup*` in `m_groupList` prefixed by `"MARKER:AIGroup"` (`AIGroup::crc`).

### 3.5 Replay Verification & Frame-0 Guard (`Recorder.cpp`)
During playback, `MSG_BEGIN_NETWORK_MESSAGES` packets embedded in the `.rep` stream carry the recorded CRCs and push them into `RecorderClass::m_crcInfo`. As the simulation advances, local CRC messages compare `newCRC` against `playbackCRC = m_crcInfo.readCRC()`.
- Note: GeneralsX commit `#315` (22 Sep 2026) fixed two crucial playback edge cases in `Recorder.cpp`:
  1. `RecorderClass::updatePlayback()` defers queuing frame-0 replay commands while `MSG_NEW_GAME` is still in `TheCommandList`, preventing frame-0 commands from executing before map scripts initialize initial `AIGroup` IDs.
  2. `RecorderClass::handleCRCMessage()` only triggers a desync when `mismatchFrame > 0`, ignoring pre-simulation frame 0 differences.

---

## 4. Critical Finding: Two Replay Compatibility Regimes

Inspecting `Core/Libraries/Source/Lib/Include/Lib/BaseDefines.h`, `Core/GameEngine/Include/Common/GameDefines.h`, `cmake/config-retail.cmake`, `cmake/gamemath.cmake`, and `cmake/compilers.cmake` reveals two distinct compile-time determinism regimes:

### 4.1 Regime A: Cross-Platform Deterministic Mode (GeneralsX Default on macOS `arm64` & Linux `x86_64`)
- **Configuration:**
  - `RETAIL_COMPATIBLE_CRC = 0`
  - `RETAIL_COMPATIBLE_DATA = 1` (loads unmodified retail `.big` archives, `.ini`, and `.map` files)
  - `USE_DETERMINISTIC_MATH = 1` (links `TheSuperHackers/GameMath` at commit `59f7ccd494f7e7c916a784ac26ef266f9f09d78d` with `GM_ENABLE_INTRINSICS=OFF`)
  - Compiler flag `-ffp-contract=off` on Clang/GCC (and `/fp:precise` on MSVC) to forbid FMA contraction (`a * b + c -> fmadd`)
  - `RTS_BUILD_OPTION_DEBUG = OFF` (assertions and debug timings must be compiled out; debug builds alter state/timing)
  - `WWMath::Floorf` / `REAL_TO_INT_FLOOR` / `REAL_TO_INT_CEIL` routed through deterministic `gm_floorf` / `gm_ceilf` (see commit `84a7a20` / `#3645`).
- **Replay Corpus:**
  - The repository `generalsx-project/GeneralsXReplays` (MIT licensed) provides 6 official Zero Hour cross-platform CI replays (`linux_2p_vanilla_map.rep`, `linux_2p_custom_map.rep`, `linux_6p_custom_map.rep`, `macos_2p_vanilla_map.rep`, `macos_2p_custom_map.rep`, `macos_6p_custom_map.rep`) and their required custom maps (`tansooo`, `[RANK] Arctic Arena ZH v1`, `AOD - Snipe Fest Final! v2 By Twinsen`).
  - Because `USE_DETERMINISTIC_MATH = 1` and `-ffp-contract=off` eliminate hardware FPU divergence, **all 3 Linux `x86_64` replays and all 3 macOS `arm64` replays pass bit-exact on macOS Apple Silicon (`macos-vulkan`)**.

### 4.2 Regime B: Legacy 2003 Windows Retail 1.04 Mode (`RETAIL_COMPATIBLE_CRC = 1`)
- **Configuration:**
  - As warned by `cmake/config-retail.cmake`:
    `Retail compatibility: Clang ... is not CRC-compatible with retail. Retail builds need the VC6 SP6 compiler (12.00.8804).`
  - Setting `RETAIL_COMPATIBLE_CRC = 1` undefines `USE_DETERMINISTIC_MATH` (`BaseDefines.h:40`), switches `REAL_TO_INT_FLOOR` / `REAL_TO_INT_CEIL` back to EA's broken `fast_float_floor` / `fast_float_ceil` (which assume round-toward-zero while the FPU is set to round-to-nearest, making `REAL_TO_INT_CEIL(2.0f) == 3` and `fast_float_floor(-1.0f) == -2`), changes `XferCRC::addCRC` endianness (`htobe(val)`), truncates `m_objectUpgradesCompleted` CRC to 8 bytes, omits `m_isTrainable` from `ExperienceTracker::crc`, and enables legacy bug macros in `GameDefines.h`.
  - Even with those macros enabled, non-VC6 compilers cannot match 2003 VC6 retail CRCs over full games because VC6 32-bit x86 keeps intermediate `float` expressions in 80-bit x87 FPU registers and uses VC6 CRT transcendental approximations.
- **Implication for Our Port:**
  - **Primary Oracle:** Our Rust simulation targets **Regime A (100% bit-exact parity with `GeneralsX` at `84a7a20` on Apple Silicon `arm64` and Linux `x86_64`)**, verified against the 6 `GeneralsXReplays` fixtures plus additional multi-faction skirmish replays recorded on the stock `GeneralsXZH` binary.
  - **Optional Feature Flag:** Where `RETAIL_COMPATIBLE_CRC` alters explicit data structures or CRC byte layout (`RandomValue`, `XferCRC`, `Object::crc`, `ExperienceTracker::crc`, `fast_float_floor`/`ceil`), we will also document and isolate the behavior behind a Cargo feature (`retail-crc`) so both code paths are preserved cleanly.

---

## 5. Deep-Dive on Phase 1 Spike Modules

To test translation velocity, token cost, and determinism discipline before scaling to 330k lines, Phase 1 ports **`RandomValue` (PRNG + seed CRC)** and **`ExperienceTracker`**. Both are self-contained, invoked continuously in every replay, and directly hashed in `GameLogic::getCRC()`.

### 5.1 Module 1A: `RandomValue` (`Core/GameEngine/Source/Common/RandomValue.cpp`)
- **State:**
  - Three independent 6-word (`[u32; 6]`) state arrays: `theGameAudioSeed`, `theGameClientSeed`, `theGameLogicSeed`, each initialized to:
    `[0xf22d0e56, 0x883126e9, 0xc624dd2f, 0x0702c49c, 0x9e353f7d, 0x6fdf3b64]`.
  - `theGameLogicBaseSeed: u32 = 0`.
  - Constant `THE_MULT_FACTOR: f32 = 1.0f32 / (u32::MAX as f32)` (IEEE-754 `0x2f800000`, i.e., $2^{-32}$).
- **Seeding (`seedRandom(u32, &mut [u32; 6])`):**
  - Wrapping 32-bit additions starting from `ax = seed.wrapping_add(0xf22d0e56)` through the 6 initial constants.
- **Step Function (`randomValue(&mut [u32; 6]) -> u32`):**
  - Carry-chain addition macro:
    `ADC(sum, a, b, c)` computes `sum = a.wrapping_add(b).wrapping_add(c)` and sets `c = u32::from(sum < a || sum < b)`.
    *(Note: In C++, `(SUM < A) || (SUM < B)` is the exact predicate used—even when `c == 1` and `b == u32::MAX`, `sum == a`, so `sum < b` is true (`a < u32::MAX`) unless `a == u32::MAX`, in which case `sum == u32::MAX - 1 < a` is true! Thus `(sum < a) || (sum < b)` is exact across all `u32` inputs.)*
  - Cascades `ADC` across `seed[5]` + `seed[4]` -> `seed[4]`, down to `seed[0]`.
  - Increments the 192-bit counter `seed[5]..seed[0]` with ripple carry (`if seed[5].wrapping_add(1) == 0 { ... }`), incrementing `ax` if all 6 words wrap to zero.
  - Returns `ax`.
- **Range Mapping:**
  - Integer `GetGameLogicRandomValue(lo: i32, hi: i32)`:
    - Under `RETAIL_COMPATIBLE_CRC == 0`: if `lo >= hi`, returns `hi` **without** advancing `theGameLogicSeed`. Otherwise `delta = (hi as u32).wrapping_sub(lo as u32).wrapping_add(1)`, returns `((randomValue(&mut theGameLogicSeed) % delta) as i32).wrapping_add(lo)`.
  - Float `GetGameLogicRandomValueReal(lo: f32, hi: f32)`:
    - Under `RETAIL_COMPATIBLE_CRC == 0`: if `lo >= hi`, returns `hi` **without** advancing `theGameLogicSeed`.
    - Otherwise `delta = hi - lo`, returns `((randomValue(&mut theGameLogicSeed) as f32) * THE_MULT_FACTOR) * delta + lo` using strict unfused IEEE-754 `f32` multiply and add.
  - `*Unchanged` variants copy `theGameLogicSeed` to a stack temporary `[u32; 6]` before calling `randomValue` (when `RETAIL_COMPATIBLE_CRC == 0`).

### 5.2 Module 1B: `ExperienceTracker` (`GeneralsMD/Code/GameEngine/Source/GameLogic/Object/ExperienceTracker.cpp`)
- **State (`ExperienceTracker`):**
  - `m_parent: *mut Object`
  - `m_currentLevel: VeterancyLevel` (`i32`: `LEVEL_REGULAR = 0`, `LEVEL_VETERAN = 1`, `LEVEL_ELITE = 2`, `LEVEL_HEROIC = 3`)
  - `m_currentExperience: i32`
  - `m_experienceSink: ObjectID` (`i32`, `INVALID_ID = 0`)
  - `m_experienceScalar: f32` (initialized to `1.0f32`)
  - `m_isTrainable: Bool` (`u8`, initialized from `m_parent->getTemplate()->isTrainable()`)
- **Determinism Trap Inside `ExperienceTracker::addExperiencePoints`:**
  - Lines 255 & 267 of `ExperienceTracker.cpp`:
    ```cpp
    // Sink forwarding:
    sinkPointer->getExperienceTracker()->addExperiencePoints(
        experienceGain * m_experienceScalar, canScaleForBonus );
    // Local gain:
    Int amountToGain = experienceGain;
    if ( canScaleForBonus )
        amountToGain *= m_experienceScalar;
    ```
    In C++, `Int * Real` promotes `Int` (`i32`) to `float` (`f32`), multiplies by `m_experienceScalar` in 32-bit IEEE-754 float, and converts back to `i32` by truncating toward zero (`((experience_gain as f32) * self.experience_scalar) as i32`).
- **CRC Contribution (`ExperienceTracker::crc`):**
  - Hashes `m_currentExperience` (4 bytes), `m_currentLevel` (4 bytes), and (when `RETAIL_COMPATIBLE_CRC == 0`) `m_isTrainable` (1 byte).

---

## 6. Determinism Traps & Rust Translation Rules

Every module ported from C++ to Rust must obey eight mandatory determinism rules:

1. **Integer Overflow & Wrapping Arithmetic**
   - C++ compilation of Zero Hour relies on 32-bit two's-complement wrapping for both `UnsignedInt` (`u32`) and `Int` (`i32`).
   - Rust panics on integer overflow in debug builds. All translated arithmetic that can wrap (hashes, PRNG, frame counters, coordinate deltas, bitmasks) must use `.wrapping_add()`, `.wrapping_sub()`, `.wrapping_mul()`, `.wrapping_neg()`, or `std::num::Wrapping<T>`.
2. **Floating-Point Operation Order & No FMA Contraction**
   - GeneralsX compiles C++ with `-ffp-contract=off` (Clang/GCC) and `/fp:precise` (MSVC) so `a * b + c` performs two separate round-to-nearest-even `f32` operations (`fmul` then `fadd`), never a single fused `fmadd`.
   - Rust's standard `*` and `+` operators on `f32` emit unfused LLVM `fmul` and `fadd` instructions by default. Never use `f32::mul_add` unless the C++ source explicitly calls `fmaf`, and never enable LLVM fast-math flags.
   - Parenthesization and left-to-right association of `f32` expressions (`(a + b) + c` vs `a + (b + c)`) must match the C++ AST identically.
3. **Deterministic Transcendentals (`TheSuperHackers/GameMath`)**
   - Never call Rust's `f32::sin`, `f32::cos`, `f32::tan`, `f32::acos`, `f32::asin`, `f32::atan2`, `f32::sqrt`, `f32::floor`, or `f32::ceil` directly in simulation logic, because `libm` implementations vary across OS/CPU targets.
   - Instead, route all simulation math through FFI bindings (and later a bit-identical Rust port) of `TheSuperHackers/GameMath` (`gm_sinf`, `gm_cosf`, `gm_tanf`, `gm_acosf`, `gm_asinf`, `gm_atan2f`, `gm_sqrtf`, `gm_floorf`, `gm_ceilf`, `gm_logf`, `gm_expf`, `gm_powf`).
4. **Implicit Conversions & Float-to-Int Truncation**
   - C++ silently promotes mixed `Int` and `Real` arithmetic to `float` and truncates toward zero on assignment back to `Int`.
   - In Rust, translate `(Int)(real_expr)` and `int_var *= real_expr` via explicit float promotion and truncation helper functions (matching C++ `static_cast<int32_t>(f)` for in-range values), and route `REAL_TO_INT_FLOOR` / `REAL_TO_INT_CEIL` through `gm_floorf` / `gm_ceilf` (or `fast_float_floor` / `fast_float_ceil` under `retail-crc`).
5. **Associative Container Ordering (`std::map` / `std::set`)**
   - C++ `std::map` and `std::set` iterate in deterministic key-sorted order. Rust's `std::collections::HashMap` and `HashSet` use randomized SipHash seeds per process, which will immediately desync replays if iterated during update or CRC.
   - Always translate `std::map<K, V>` to `BTreeMap<K, V>` (or a deterministically ordered vector/index map).
   - **Pointer-Keyed Maps/Sets:** Any C++ `std::map<Object*, ...>` or `std::set<Object*>` where iteration order affects logic or RNG calls must be keyed by deterministic ID (`ObjectID`, `NameKeyType`, `TeamID`, or arena creation sequence index), never by raw memory address.
6. **Intrusive Linked Lists & Pool Allocation Order**
   - `GameLogic::m_objList` is a doubly-linked intrusive list (`m_next`, `m_prev` on `Object`) whose order determines both the per-frame object update order and the `MARKER:Objects` CRC hash order.
   - Insertion at head vs tail (`prependToList` vs `appendToList`) and deferred deletion via `GameLogic::m_objectsToDestroy` must be preserved identically.
7. **Evaluation Order of Function Arguments**
   - In C++, the evaluation order of function arguments (`foo(GameLogicRandomValue(0, 10), GameLogicRandomValue(0, 10))`) is unspecified and differs between MSVC (typically right-to-left on x86 `cdecl`/`stdcall`) and Clang/GCC on `arm64` (left-to-right).
   - Whenever a C++ call site passes multiple arguments that invoke `GameLogicRandomValue*` or mutate simulation state, bind each argument to an explicit `let` temporary in the exact evaluation order used by the reference `macos-vulkan` Clang `arm64` compiler (left-to-right).
8. **Signed Bit-Shifts & Narrow Integer Promotions**
   - Specify signed (`i8`, `i16`, `i32`) vs unsigned (`u8`, `u16`, `u32`) types explicitly (`Byte` = `i8`, `UnsignedByte` = `u8`, `Short` = `i16`, `UnsignedShort` = `u16`, `Int` = `i32`, `UnsignedInt` = `u32`, `Bool` = `u8`).
   - Note that `char` is signed on `x86_64` and `arm64` macOS (`i8`), whereas on Linux `arm64` `char` is unsigned (`u8`); always use explicit `i8` or `u8`.

---

## 7. Host Environment & Disk Space Findings

Checked on 20 November 2026 on the target Apple Silicon Mac:
1. **Free Disk Space Alert:**
   - `df -h /` reports **~13 GiB available** (down from ~36 GiB when the project brief was drafted).
   - A full Zero Hour Steam installation is ~2.8–3.5 GiB, and a C++ CMake + vcpkg build tree with debug symbols for both Generals and Zero Hour can easily consume 6–10 GiB if unconstrained.
2. **Mandatory Disk Budget Measures for Phase 0 & Beyond:**
   - **Never copy `.big` files:** Keep the retail game data in one single read-only directory (`~/Games/generalszh`) and pass `-assetPath ~/Games/generalszh` (or `export CNC_GENERALS_ZH_PATH=$HOME/Games/generalszh`) to `GeneralsXZH`.
   - **Build Zero Hour Only:** Configure CMake with `-DRTS_BUILD_GENERALS=OFF -DRTS_BUILD_CORE_TOOLS=OFF -DCMAKE_BUILD_TYPE=Release` (or stripped `RelWithDebInfo` without multi-gigabyte debug bloat) and build only `--target z_generals`.
   - **Shallow Reference Clone:** Clone `fbraz3/GeneralsX` with `--depth 1` at `84a7a20769aa5749f388b1f49bd542c79b3c33e8` rather than fetching full git history.
3. **Game Data Status (`~/Games/generalszh`):**
   - `~/Games/generalszh` is not present yet on disk (`~/Games/` currently contains only `ra2yr`).
   - In Phase 0, game data will be fetched from Steam app `2732960` using `steamcmd +@sSteamCmdForcePlatformType windows +force_install_dir ~/Games/generalszh +login <user> +app_update 2732960 validate +quit` (with the owner entering their Steam credentials interactively).
