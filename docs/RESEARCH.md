# Research & Architectural Reference: Zero Hour Engine Port to Rust (`generals-rust`)

This document records verified architectural facts, subsystem invariants, execution pipelines, data layouts, and determinism rules extracted from the reference C++ codebase (`fbraz3/GeneralsX` at commit `84a7a20769aa5749f388b1f49bd542c79b3c33e8`) to guide the Rust port.

---

## 1. Upstream Lineage & Legal Constraints

### 1.1 Source Provenance
- **Original GPLv3 Release:** Electronic Arts Inc. released the source code for *Generals* and *Zero Hour* on 27 February 2025 under **GPL-3.0-or-later with Additional Terms per GNU GPL Section 7** (`electronicarts/CnC_Generals_Zero_Hour`, archived).
- **Community Mainline:** `TheSuperHackers/GeneralsGameCode` unifies shared engine subsystems under `Core/` while keeping game-specific code under `Generals/` and `GeneralsMD/`.
- **Native macOS / Linux Port (Reference Oracle):** `fbraz3/GeneralsX` adapts the community mainline to run natively on 64-bit Apple Silicon (`arm64`) and Linux (`x86_64`) using SDL3, DXVK-native + MoltenVK (Vulkan-to-Metal), GLM, and MiniAudio.
- **Pinned Reference Commit:** `84a7a20769aa5749f388b1f49bd542c79b3c33e8` (committed 15 November 2026 on `fbraz3/GeneralsX` `main`).

### 1.2 GPL-3.0-or-later & EA Section 7 Additional Terms
Every source file and binary distribution must comply with both GNU GPL v3 and the Electronic Arts Section 7 terms included in `LICENSE.md`:
1. **No Franchise Trademarks in Identifiers:** No rights are granted to `"Command & Conquer"`, `"C&C"`, or `"cnc"`. The repository name (`generals-rust`), Cargo workspace crates (`zh_common`, `zh_logic`, `zh_ffi`, `zh_sim`), and binary names (`zh-sim`) avoid those marks while describing compatibility with *Zero Hour* in prose.
2. **Marked as Modified Version:** Per GPLv3 Section 5(a) and EA Section 7, all distributions and repository documentation clearly mark the work as a modified version and do not claim affiliation with Electronic Arts Inc.
3. **Preservation of Notices:** All propagation or conveyance retains `LICENSE.md` including the EA copyright notice, indemnification clause, and warranty disclaimer.
4. **Strict Isolation:**
   - No retail game assets (`.big`, `.bik`, `.map`, `.tga`, `.w3d`, `.dds`, `.csf`, `.mp3`, `.wav`, retail binaries) may ever enter git history.
   - Zero dependencies, code, submodules, or crates from the owner's other repositories.
   - Zero upstream pull requests, issues, or comments to `fbraz3/GeneralsX` or `TheSuperHackers/GeneralsGameCode`.

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
| `GeneralsMD/.../Source/GameLogic/System` | 1 | 180,016 | `GameLogic.cpp` (top-level frame tick `update()`, sleepy-update heap, object lifecycle, `getCRC()`) |
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

## 3. Deep Engine Architecture: Subsystems, Frame Pipeline & Object Model

### 3.1 Engine Bootstrapping & The Singleton Graph (`GameEngine::init()`)

Inspecting `GeneralsMD/Code/GameEngine/Source/Common/GameEngine.cpp:602` reveals the exact initialization order of the global singletons (`The*`) and how `TheGlobalData->m_iniCRC` is accumulated via `XferCRC xferCRC` across INI directories:

| Order | Singleton Global | Class / Factory | INI Path(s) Loaded & Hashed into `xferCRC` | Category |
|---:|---|---|---|---|
| 1 | `TheSubsystemList` | `SubsystemInterfaceList` | — | Lifecycle |
| 2 | *(PRNG state)* | `InitRandom()` | — | Logic/Common |
| 3 | `TheFileSystem` | `createFileSystem()` | — | I/O (non-reset) |
| 4 | `TheNameKeyGenerator` | `NameKeyGenerator` | — | ID Registry (non-reset) |
| 5 | `TheCommandList` | `CommandList` | — | Messaging (non-reset) |
| 6 | `TheLocalFileSystem` | `createLocalFileSystem()` | — | I/O |
| 7 | `TheArchiveFileSystem` | `createArchiveFileSystem()` | Mounts `.big` files & mods | I/O |
| 8 | `TheWritableGlobalData` | `GlobalData` | `Data\INI\Default\GameData`, `Data\INI\GameData` (+ `SagePatch.ini`) | Config |
| 9 | `TheGameLODManager` | `GameLODManager` | `GameLOD.ini`, `GameLODPresets.ini` | Config (non-reset) |
| 10 | *(Water & Weather)* | `INI::loadFileDirectory` | `Data\INI\{Default\,}Water`, `Data\INI\{Default\,}Weather` | Environment |
| 11 | `TheGameText` | `CreateGameTextInterface()` | `generals.csf` | Localization |
| 12 | `TheScienceStore` | `ScienceStore` | `Data\INI\Default\Science`, `Data\INI\Science` | Logic Rules |
| 13 | `TheMultiplayerSettings` | `MultiplayerSettings` | `Data\INI\Default\Multiplayer`, `Data\INI\Multiplayer` | Logic Rules |
| 14 | `TheTerrainTypes` | `TerrainTypeCollection` | `Data\INI\Default\Terrain`, `Data\INI\Terrain` | Map/Terrain |
| 15 | `TheTerrainRoads` | `TerrainRoadCollection` | `Data\INI\Default\Roads`, `Data\INI\Roads` | Map/Terrain |
| 16 | `TheGlobalLanguageData` | `GlobalLanguage` | `Data\<lang>\Language.ini` | Localization |
| 17 | `TheAudio` | `createAudioManager(headless)` | `AudioSettings.ini`, `SoundEffects.ini`, `Music.ini`, `Speech.ini`, `Eva.ini` | Audio (headless dummy) |
| 18 | `TheFunctionLexicon` | `createFunctionLexicon()` | Table of UI/draw callback pointers | UI/Client |
| 19 | `TheModuleFactory` | `createModuleFactory()` | Registers all 222+ `BehaviorModule` & `DrawableModule` templates | Factory |
| 20 | `TheMessageStream` | `createMessageStream()` | — | Messaging |
| 21 | `TheSidesList` | `SidesList` | Populated per-map (players, teams, scripts) | Map/Logic |
| 22 | `TheCaveSystem` | `CaveSystem` | Tunnel/cave network links (`CaveContain`) | Logic |
| 23 | `TheRankInfoStore` | `RankInfoStore` | `Data\INI\Rank` | Logic Rules |
| 24 | `ThePlayerTemplateStore` | `PlayerTemplateStore` | `Data\INI\Default\PlayerTemplate`, `Data\INI\PlayerTemplate` | Logic Rules |
| 25 | `TheParticleSystemManager` | `createParticleSystemManager(headless)` | `Data\INI\ParticleSystem` | Client ( queried by logic) |
| 26 | `TheFXListStore` | `FXListStore` | `Data\INI\Default\FXList`, `Data\INI\FXList` | Client/FX |
| 27 | `TheWeaponStore` | `WeaponStore` | `Data\INI\Weapon` | Logic Rules |
| 28 | `TheObjectCreationListStore` | `ObjectCreationListStore` | `Data\INI\Default\ObjectCreationList`, `Data\INI\ObjectCreationList` | Logic Rules |
| 29 | `TheLocomotorStore` | `LocomotorStore` | `Data\INI\Locomotor` | Logic Rules |
| 30 | `TheSpecialPowerStore` | `SpecialPowerStore` | `Data\INI\Default\SpecialPower`, `Data\INI\SpecialPower` | Logic Rules |
| 31 | `TheDamageFXStore` | `DamageFXStore` | `Data\INI\DamageFX` | Client/FX |
| 32 | `TheArmorStore` | `ArmorStore` | `Data\INI\Armor` | Logic Rules |
| 33 | `TheBuildAssistant` | `BuildAssistant` | — | Logic |
| 34 | `TheThingFactory` | `createThingFactory()` | `Data\INI\Default\Object`, `Data\INI\Object` | Logic Rules (~3,000 templates) |
| 35 | `TheUpgradeCenter` | `UpgradeCenter` | `Data\INI\Default\Upgrade`, `Data\INI\Upgrade` | Logic Rules |
| 36 | `TheGameClient` | `createGameClient()` | Creates `TheDisplay`, `TheInGameUI`, `TheControlBar`, `TheEva`, `TheTacticalView`, `TheSnowManager`, etc. | Client |
| 37 | `TheAI` | `AI` | `Data\INI\Default\AIData`, `Data\INI\AIData` (creates `TheAI->pathfinder()`) | Logic |
| 38 | `TheGameLogic` | `createGameLogic()` | Creates `ThePartitionManager`, `TheGhostObjectManager`, `TheTerrainLogic`, `TheScriptActions`, `TheScriptConditions`, `TheScriptEngine` | Logic Core |
| 39 | `TheTeamFactory` | `TeamFactory` | — | Logic |
| 40 | `TheCrateSystem` | `CrateSystem` | `Data\INI\Default\Crate`, `Data\INI\Crate` | Logic Rules |
| 41 | `ThePlayerList` | `PlayerList` | — | Logic |
| 42 | `TheRecorder` | `createRecorder()` | — | Replay/CRC Oracle |
| 43 | `TheRadar` | `createRadar(headless)` | — | Logic/UI hybrid |
| 44 | `TheVictoryConditions` | `createVictoryConditions()` | — | Logic |
| 45 | `TheMetaMap` | `MetaMap` | `Data\INI\CommandMap` | Input/UI |
| 46 | `TheActionManager` | `ActionManager` | Target validity & command range queries | Logic |
| 47 | `TheGameStateMap` / `TheGameState` | `GameStateMap` / `GameState` | Save/Load & deep CRC snapshot manager | Serialization |
| 48 | `TheGameResultsQueue` | `GameResultsInterface` | Match stats queue | Match Reporting |

**Key Architectural Insight on `NameKeyGenerator` Determinism:**
During `GameEngine::init()`, every INI file parsed calls `TheNameKeyGenerator->nameToKey(...)`, which assigns sequential integer IDs (`NameKeyType = 1, 2, 3, ...`) on first sight of each string. Because `NameKeyType` values are later used as keys in `std::map<NameKeyType, ...>` (e.g. `ArmorStore`, `UpgradeCenter`, `ScienceStore`, `Module::getModuleTagNameKey()`), **INI load order directly determines `NameKeyType` integer ordering**. Notice lines 847, 868, and 941 of `GameEngine.cpp`, where `RETAIL_COMPATIBLE_CRC` explicitly asserts the exact `NameKeyType` counter after `TheGameText`, `TheAudio`, and `TheThingFactory`!

---

### 3.2 Exact Per-Frame Simulation Pipeline (`GameLogic::update()`)

In `GeneralsMD/Code/GameEngine/Source/GameLogic/System/GameLogic.cpp:3912`, every 30 Hz logic tick executes the following **16 steps in strict sequence**:

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                        GameLogic::update()  [Frame N]                        │
├──────────────────────────────────────────────────────────────────────────────┤
│  1. ScopedFPUGuard + setFPMode() (locks round-to-nearest FE_TONEAREST)       │
│  2. If m_startNewGame: startNewGame(FALSE), m_startNewGame = FALSE           │
│  3. TheGameClient->setFrame(now) (where now = m_frame)                       │
│  4. TheScriptEngine->UPDATE() (evaluates map script conditions & actions)    │
│  5. TheFramePacer->setTimeFrozen(...); if frozen -> return early             │
│  6. TheTerrainLogic->UPDATE() (bridge state transitions, water updates)      │
│  7. Periodic CRC Generation: if (m_frame % interval == 0):                   │
│       m_CRC = getCRC(CRC_RECALC);                                            │
│       append MSG_LOGIC_CRC_PLAYBACK (or MSG_LOGIC_CRC) to TheCommandList     │
│  8. If TheStatsCollector: TheStatsCollector->update()                        │
│  9. TheRecorder->UPDATE() (in playback: queues commands for frame == now)    │
│ 10. processCommandList(TheCommandList) (dispatches all GameMessages for now, │
│       including MSG_NEW_GAME, unit commands, and MSG_LOGIC_CRC_PLAYBACK)     │
│ 11. Sleepy Update Priority Queue Drain (m_sleepyUpdates):                    │
│       while !m_sleepyUpdates.empty():                                        │
│         u = peekSleepyUpdate()                                               │
│         if u.nextCallFrame() > now: break                                    │
│         if u.getDisabledTypesToProcess().testForAll(obj.disabledFlags):      │
│           sleepLen = max(1, u->update())                                     │
│         else:                                                                │
│           sleepLen = UPDATE_SLEEP_NONE (1)                                   │
│         u.setNextCallFrame(now + sleepLen)                                   │
│         rebalanceSleepyUpdate(0)                                             │
│ 12. TheAI->UPDATE() (pathfinder queue + AIPlayer/AISkirmishPlayer + AIGroups)│
│ 13. TheBuildAssistant->UPDATE()                                              │
│ 14. ThePartitionManager->UPDATE() (sight/shroud look/unlook queues)          │
│ 15. End-of-Frame Cleanup:                                                    │
│       a) processDestroyList() (removes destroyed objects & sleepy updates)   │
│       b) TheCommandList->reset()                                             │
│       c) TheWeaponStore->UPDATE()                                            │
│       d) TheLocomotorStore->UPDATE()                                         │
│       e) TheVictoryConditions->UPDATE()                                      │
│       f) For obj in m_objList: if obj->isDisabled(): obj->checkDisabledStatus│
│ 16. If !m_startNewGame: m_frame++, m_hasUpdated = TRUE                       │
└──────────────────────────────────────────────────────────────────────────────┘
```

**Critical Observation on Step 7 vs Step 9 vs Step 10:**
Notice that on a CRC frame (`m_frame % interval == 0`), `m_CRC = getCRC(CRC_RECALC)` is computed **at Step 7** (after `TheScriptEngine->UPDATE()` and `TheTerrainLogic->UPDATE()`, but **before** `TheRecorder->UPDATE()`, `processCommandList`, `m_sleepyUpdates`, `TheAI->UPDATE()`, and `processDestroyList()`!). Then `MSG_LOGIC_CRC_PLAYBACK` is appended to `TheCommandList`, `TheRecorder->UPDATE()` appends the recorded commands (including the recorded `MSG_BEGIN_NETWORK_MESSAGES` CRC packet) to `TheCommandList`, and `processCommandList(TheCommandList)` compares them at Step 10!

---

### 3.3 The `m_sleepyUpdates` Priority Queue Architecture & Determinism Invariants

Almost all `UpdateModule` subclasses in Zero Hour are scheduled via `GameLogic::m_sleepyUpdates` (`std::vector<UpdateModulePtr>` maintained as a custom binary min-heap in `GameLogic.cpp:3040-3350`).

#### 3.3.1 Priority Key Encoding (`UpdateModule.h:919-1060`)
Each `UpdateModule` stores two scheduler fields:
- `m_nextCallFrameAndPhase: u32 = (wake_frame << 2) | (phase as u32)`
  - Upper 30 bits (`m_nextCallFrameAndPhase >> 2`): target wake frame (`1..=0x3fffffff`, where `UPDATE_SLEEP_FOREVER = 0x3fffffff`).
  - Lower 2 bits (`m_nextCallFrameAndPhase & 3`): `SleepyUpdatePhase`:
    - `PHASE_INITIAL = 0`
    - `PHASE_PHYSICS = 1` (`PhysicsBehavior`)
    - `PHASE_NORMAL = 2` (default for almost all modules)
    - `PHASE_FINAL = 3`
- `m_indexInLogic: i32`: the module's current array index in `m_sleepyUpdates` (`-1` when not registered).

#### 3.3.2 Why Standard Binary Heaps Will Desync Replays
Replacing `m_sleepyUpdates` with `std::collections::BinaryHeap` will immediately desync replays because of four subtle invariants in `GameLogic.cpp`:

1. **Strict `>` Comparison with Left-Child Tie-Breaking:**
   ```cpp
   inline Bool isLowerPriority(const UpdateModulePtr a, const UpdateModulePtr b) {
       return a->friend_getPriority() > b->friend_getPriority();
   }
   ```
   - In `rebalanceChildSleepyUpdate(i)`:
     - Left child is `child = (i << 1) + 1`, right child is `child + 1`.
     - `if (child < sz - 1 && isLowerPriority(m_sleepyUpdates[child], m_sleepyUpdates[child + 1])) ++child;`
     - When left and right children have the **exact same priority** (`left.pri == right.pri`), `isLowerPriority(left, right)` is `false`, so the heap **always favors the left child (`(i << 1) + 1`)**.
     - Furthermore, `if (!isLowerPriority(m_sleepyUpdates[i], m_sleepyUpdates[child])) break;` means a parent never swaps with a child of **equal** priority.
2. **In-Place Root Sifting After `u->update()`:**
   - At Step 11 of `GameLogic::update()`, after `u = m_sleepyUpdates[0]` executes `u->update()`, `u` is **not** popped and re-pushed. Instead, `u->friend_setNextCallFrame(now + sleepLen)` mutates `m_sleepyUpdates[0]` in place and calls `rebalanceSleepyUpdate(0)` (which sifts index `0` down via `rebalanceChildSleepyUpdate(0)`).
   - Pop-then-push would swap `m_sleepyUpdates[sz - 1]` into index `0` before sifting, producing a completely different heap tree topology for equal-priority nodes!
3. **The Already-Awake Guard in `friend_awakenUpdateModule` (`GameLogic.cpp:3310`):**
   ```cpp
   if ((now > 0) && (u->friend_getNextCallFrame() == now) && (whenToWakeUp == now + 1)) {
       return;
   }
   ```
   - When an external event calls `setWakeFrame(obj, UPDATE_SLEEP_NONE)` (`whenToWakeUp = now + 1`) on a module that is **already scheduled to run on the current frame `now`**, `friend_awakenUpdateModule` does **nothing** so the module still runs during frame `now` and preserves its exact heap slot!
4. **Two-Pass Reverse-Heap-Index Erase in `processDestroyList()` (`GameLogic.cpp:2701-2726`):**
   - When an object is destroyed at the end of a frame, `processDestroyList()` does **not** iterate `currentObject->getBehaviorModules()` to remove its `UpdateModule`s.
   - Instead:
     - **Pass 1:** It scans `m_sleepyUpdates` linearly from index `0` to `len - 1`, collecting every `UpdateModulePtr` belonging to `currentObject` into `sleepyUpdatesForThisObject[0..numSUO]`.
     - **Pass 2:** It iterates `sleepyUpdatesForThisObject` **backwards** (`for (--numSUO; numSUO >= 0; --numSUO)`), looking up `idx = sleepyUpdatesForThisObject[numSUO]->friend_getIndexInLogic()` and calling `eraseSleepyUpdate(idx)` (which moves `m_sleepyUpdates[last]` into `m_sleepyUpdates[idx]`, pops back, and calls `rebalanceSleepyUpdate(idx)`).
   - Because each `eraseSleepyUpdate` swaps with `m_sleepyUpdates[last]` and rebalances, erasing in reverse heap-index order produces a specific permutation of the remaining heap elements that must be replicated 1:1 in Rust.

---

### 3.4 `Object`, `Thing`, Sleep Helpers & The 222 `BehaviorModule` Hierarchy

#### 3.4.1 `Object` Construction & Module Registration Sequence (`Object.cpp:181-515`)
When `TheThingFactory->newObject(template, team)` constructs an `Object`:
1. **Base `Thing` & Transform Init:** Initializes `m_template` (resolved via `tt->getFinalOverride()`), `Coord3D m_pos`, `Real m_angle`, `Matrix3D m_transform`, and lazy transform/altitude dirty flags.
2. **SightingInfos & ID Allocation:** Allocates 5 persistent `SightingInfo` structs (`m_partitionLastLook`, `m_partitionRevealAllLastLook`, `m_partitionLastShroud`, `m_partitionLastThreat`, `m_partitionLastValue`) and assigns `m_id = TheGameLogic->allocateObjectID()` (monotonic counter `m_nextObjID`).
3. **Team Assignment:** `setTeam(team ? team : ThePlayerList->getNeutralPlayer()->getDefaultTeam())`.
4. **Conditional Sleep-Helper Modules (Prepended to `m_behaviors` Before INI Modules!):**
   Up to `NUM_SLEEP_HELPERS = 8` built-in `UpdateModule` helpers are instantiated and placed at the **front** of `m_behaviors` in this exact order:
   1. `m_smcHelper` (`ObjectSMCHelper`) — always created.
   2. `m_statusDamageHelper` (`StatusDamageHelper`) — created unless `m_behaviors` contains `"InactiveBody"`.
   3. `m_subdualDamageHelper` (`SubdualDamageHelper`) — created unless `m_behaviors` contains `"InactiveBody"`.
   4. `m_repulsorHelper` (`ObjectRepulsorHelper`) — created if `TheAI->getAiData()->m_enableRepulsors && isKindOf(KINDOF_CAN_BE_REPULSED)`.
   5. `m_defectionHelper` (`ObjectDefectionHelper`) — created if `!tt->isKindOf(KINDOF_SHRUBBERY)`.
   6. `m_wsHelper` (`ObjectWeaponStatusHelper`) — created if `tt->canPossiblyHaveAnyWeapon()`.
   7. `m_firingTracker` (`FiringTracker`) — created if `tt->canPossiblyHaveAnyWeapon()`.
   8. `m_tempWeaponBonusHelper` (`TempWeaponBonusHelper`) — created if `tt->canPossiblyHaveAnyWeapon()`.
5. **INI `BehaviorModule` Instantiation:** Iterates `tt->getBehaviorModuleInfo()` (`0..mi.getCount()`), calling `TheModuleFactory->newModule(this, modName, modData, MODULETYPE_BEHAVIOR)`, appending each to `m_behaviors`, and caching fast-access pointers on `Object`:
   - `m_body: BodyModuleInterface*`
   - `m_contain: ContainModuleInterface*`
   - `m_stealth: StealthUpdate*`
   - `m_ai: AIUpdateInterface*`
   - `m_physics: PhysicsBehavior*`
6. **`ExperienceTracker` & `onObjectCreated()`:**
   - Allocates `m_experienceTracker = newInstance(ExperienceTracker)(this)` and sets initial veterancy from `controller->getProductionVeterancyLevel(templateName)`.
   - Calls `(*b)->onObjectCreated()` on every module in `m_behaviors` from index `0` to `N-1` so modules can cross-link to sibling modules on the same `Object`.
7. **World Registration (`TheGameLogic->registerObject(this)`):**
   - **Prepends** `this` to the head of `GameLogic::m_objList` (`obj->prependToList(&m_objList)`). *(Note: `m_objList` is therefore ordered from **newest object to oldest object**!)*
   - Inserts `this` into `GameLogic::m_objHash`.
   - Iterates `m_behaviors` in order (`0..N-1`), pushing every module where `b->getUpdate() != nullptr` onto `m_sleepyUpdates` via `pushSleepyUpdate(u)` (with default wake frame `max(1, m_frame)` if the module's constructor did not call `setWakeFrame`).

#### 3.4.2 Multiple Inheritance & Facet Dispatch in `BehaviorModule` (`BehaviorModule.h:1314-1410`)
C++ `BehaviorModule` subclasses use multiple inheritance to implement combinations of **10 primary lifecycle facets** and **25 domain sub-interfaces** without `dynamic_cast`:
- **10 Primary Lifecycle Facets on `BehaviorModuleInterface`:**
  1. `getBody() -> BodyModuleInterface*` (`ActiveBody`, `StructureBody`, `HighlanderBody`, `HiveStructureBody`, `ImmortalBody`, `InactiveBody`, `UndeadBody`)
  2. `getCollide() -> CollideModuleInterface*` (`CrateCollide` subclasses, `SquishCollide`, `FireWeaponCollide`, etc.)
  3. `getContain() -> ContainModuleInterface*` (`OpenContain`, `TransportContain`, `GarrisonContain`, `OverlordContain`, `HelixContain`, `TunnelContain`, `CaveContain`, `HealContain`, `InternetHackContain`, `MobNexusContain`, `ParachuteContain`, `RailedTransportContain`)
  4. `getCreate() -> CreateModuleInterface*` (`GrantUpgradeCreate`, `LockWeaponCreate`, `PreorderCreate`, `SpecialPowerCreate`, `SupplyCenterCreate`, `SupplyWarehouseCreate`, `VeterancyGainCreate`, plus `AutoDepositUpdate`, `SpawnBehavior`, etc.)
  5. `getDamage() -> DamageModuleInterface*` (`BoneFXDamage`, `TransitionDamageFX`, plus dual-interface behaviors)
  6. `getDestroy() -> DestroyModuleInterface*` (`DestroyModule`)
  7. `getDie() -> DieModuleInterface*` (`CreateObjectDie`, `CrushDie`, `DamDie`, `DestroyDie`, `EjectPilotDie`, `FXListDie`, `KeepObjectDie`, `RebuildHoleExposeDie`, `SpecialPowerCompletionDie`, `UpgradeDie`, plus `SlowDeathBehavior`, `SpawnBehavior`, `MinefieldBehavior`, `TechBuildingBehavior`, etc.)
  8. `getSpecialPower() -> SpecialPowerModuleInterface*` (`OCLSpecialPower`, `CashHackSpecialPower`, `CleanupAreaPower`, `CommandButtonHuntUpdate`, `DefectorSpecialPower`, `DemoralizeSpecialPower`, `FireWeaponPower`, `SpyVisionSpecialPower`, `BaikonurLaunchPower`, etc.)
  9. `getUpdate() -> UpdateModuleInterface*` (all 86 `Update/*` classes + most `Behavior/*` + all `Contain/*` + the 8 `Helper/*` classes!)
  10. `getUpgrade() -> UpgradeModuleInterface*` (all 21 `Upgrade/*` classes + dual-interface behaviors like `PowerPlantUpdate`, `RadarUpdate`, `StealthUpdate`, `WeaponBonusUpdate`)
- **25 Specialized Sub-Interfaces on `BehaviorModuleInterface`:**
  `ParkingPlaceBehaviorInterface`, `RebuildHoleBehaviorInterface`, `BridgeBehaviorInterface`, `BridgeTowerBehaviorInterface`, `BridgeScaffoldBehaviorInterface`, `OverchargeBehaviorInterface`, `TransportPassengerInterface`, `CaveInterface`, `LandMineInterface`, `EjectPilotDieInterface`, `ProjectileUpdateInterface`, `AIUpdateInterface`, `ExitInterface`, `DockUpdateInterface`, `RailedTransportDockUpdateInterface`, `SlowDeathBehaviorInterface`, `SpecialPowerUpdateInterface`, `SlavedUpdateInterface`, `ProductionUpdateInterface`, `HordeUpdateInterface`, `PowerPlantUpdateInterface`, `SpawnBehaviorInterface`, `CountermeasuresBehaviorInterface`, `StealthUpdate`, `SpyVisionUpdate`.

**Why This Matters for Rust Architecture:**
A single C++ class (for example, `SlowDeathBehavior`, `SpawnBehavior`, `OpenContain`, or `PropagandaTowerBehavior`) frequently implements **3 to 5 of these interfaces simultaneously** and shares mutable fields across `update()`, `onDie()`, `onDamage()`, `onCollide()`, and `onUpgrade()`. In Rust, rather than splitting one C++ module into separate structs (which would break shared state and `Snapshot::xfer` layout), each `ModuleInstance` in our Rust `ModuleArena` is a single enum variant implementing those facet methods, indexed by a precomputed `u64` facet bitmask on `Object` so facet loops (`for m in obj.modules_with_facet(FACET_DIE)`) touch only matching modules!

---

## 4. The Replay & State-Checksum Oracle

### 4.1 Headless Simulation Loop
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

### 4.2 Command-Line Flags (`Core/GameEngine/Source/Common/CommandLine.cpp`)

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

### 4.3 Exact CRC Algorithms (`crc.h` and `XferCRC.cpp`)

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

### 4.4 Exact Per-Frame State Hashing Sequence (`GameLogic::getCRC`)

In `GeneralsMD/Code/GameEngine/Source/GameLogic/System/GameLogic.cpp:4360`, `GameLogic::getCRC` feeds the following state into `XferCRC` in strict deterministic order:

1. **Marker `"MARKER:Objects"`** (`xferAsciiString`: `u16` length `14` + 14 ASCII bytes).
2. **All `Object` instances in `m_objList` linked-list order** (`Object::crc` in `Object.cpp:3976`):
   - `m_privateStatus` (`u8`)
   - `*getTransformMatrix()` (`Matrix3D`, 3x4 `f32` = 48 bytes)
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

---

## 5. Critical Finding: Two Replay Compatibility Regimes

Inspecting `Core/Libraries/Source/Lib/Include/Lib/BaseDefines.h`, `Core/GameEngine/Include/Common/GameDefines.h`, `cmake/config-retail.cmake`, `cmake/gamemath.cmake`, and `cmake/compilers.cmake` reveals two distinct compile-time determinism regimes:

### 5.1 Regime A: Cross-Platform Deterministic Mode (GeneralsX Default on macOS `arm64` & Linux `x86_64`)
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

### 5.2 Regime B: Legacy 2003 Windows Retail 1.04 Mode (`RETAIL_COMPATIBLE_CRC = 1`)
- **Configuration:**
  - As warned by `cmake/config-retail.cmake`:
    `Retail compatibility: Clang ... is not CRC-compatible with retail. Retail builds need the VC6 SP6 compiler (12.00.8804).`
  - Setting `RETAIL_COMPATIBLE_CRC = 1` undefines `USE_DETERMINISTIC_MATH` (`BaseDefines.h:40`), switches `REAL_TO_INT_FLOOR` / `REAL_TO_INT_CEIL` back to EA's broken `fast_float_floor` / `fast_float_ceil` (which assume round-toward-zero while the FPU is set to round-to-nearest, making `REAL_TO_INT_CEIL(2.0f) == 3` and `fast_float_floor(-1.0f) == -2`), changes `XferCRC::addCRC` endianness (`htobe(val)`), truncates `m_objectUpgradesCompleted` CRC to 8 bytes, omits `m_isTrainable` from `ExperienceTracker::crc`, and enables legacy bug macros in `GameDefines.h`.
  - Even with those macros enabled, non-VC6 compilers cannot match 2003 VC6 retail CRCs over full games because VC6 32-bit x86 keeps intermediate `float` expressions in 80-bit x87 FPU registers and uses VC6 CRT transcendental approximations.
- **Implication for Our Port:**
  - **Primary Oracle:** Our Rust simulation targets **Regime A (100% bit-exact parity with `GeneralsX` at `84a7a20` on Apple Silicon `arm64` and Linux `x86_64`)**, verified against the 6 `GeneralsXReplays` fixtures plus additional multi-faction skirmish replays recorded on the stock `GeneralsXZH` binary.
  - **Optional Feature Flag:** Where `RETAIL_COMPATIBLE_CRC` alters explicit data structures or CRC byte layout (`RandomValue`, `XferCRC`, `Object::crc`, `ExperienceTracker::crc`, `fast_float_floor`/`ceil`), we isolate the behavior behind a Cargo feature (`retail-crc`) so both code paths are preserved cleanly.

---

## 6. Deep-Dive on Phase 1 Spike Modules

To test translation velocity, token cost, and determinism discipline before scaling to 330k lines, Phase 1 ports **`RandomValue` (PRNG + seed CRC)** and **`ExperienceTracker`**. Both are self-contained, invoked continuously in every replay, and directly hashed in `GameLogic::getCRC()`.

### 6.1 Module 1A: `RandomValue` (`Core/GameEngine/Source/Common/RandomValue.cpp`)
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

### 6.2 Module 1B: `ExperienceTracker` (`GeneralsMD/Code/GameEngine/Source/GameLogic/Object/ExperienceTracker.cpp`)
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

## 7. Ten Mandatory Determinism Traps & Rust Translation Rules

Every module ported from C++ to Rust must obey ten mandatory determinism rules:

1. **Integer Overflow & Wrapping Arithmetic**
   - C++ compilation of Zero Hour relies on 32-bit two's-complement wrapping for both `UnsignedInt` (`u32`) and `Int` (`i32`).
   - Rust panics on integer overflow in debug builds. All translated arithmetic that can wrap (hashes, PRNG, frame counters, coordinate deltas, bitmasks) must use `.wrapping_add()`, `.wrapping_sub()`, `.wrapping_mul()`, `.wrapping_neg()`, or `std::num::Wrapping<T>`.
2. **Floating-Point Operation Order & No FMA Contraction**
   - GeneralsX compiles C++ with `-ffp-contract=off` (Clang/GCC) and `/fp:precise` (MSVC) so `a * b + c` performs two separate round-to-nearest-even `f32` operations (`fmul` then `fadd`), never a single fused `fmadd`.
   - Rust's standard `*` and `+` operators on `f32` emit unfused LLVM `fmul` and `fadd` instructions by default. Never use `f32::mul_add` unless the C++ source explicitly calls `fmaf`, and never enable LLVM fast-math flags.
   - Parenthesization and left-to-right association of `f32` expressions (`(a + b) + c` vs `a + (b + c)`) must match the C++ AST identically.
3. **Deterministic Transcendentals (`TheSuperHackers/GameMath`)**
   - Never call Rust's `f32::sin`, `f32::cos`, `f32::tan`, `f32::acos`, `f32::asin`, `f32::atan2`, `f32::sqrt`, `f32::floor`, or `f32::ceil` directly in simulation logic, because `libm` implementations vary across OS/CPU targets.
   - Route all simulation math through FFI bindings (and later a bit-identical Rust port) of `TheSuperHackers/GameMath` (`gm_sinf`, `gm_cosf`, `gm_tanf`, `gm_acosf`, `gm_asinf`, `gm_atan2f`, `gm_sqrtf`, `gm_floorf`, `gm_ceilf`, `gm_logf`, `gm_expf`, `gm_powf`).
4. **Implicit Conversions & Float-to-Int Truncation**
   - C++ silently promotes mixed `Int` and `Real` arithmetic to `float` and truncates toward zero on assignment back to `Int`.
   - In Rust, translate `(Int)(real_expr)` and `int_var *= real_expr` via explicit float promotion and truncation helper functions (matching C++ `static_cast<int32_t>(f)`), and route `REAL_TO_INT_FLOOR` / `REAL_TO_INT_CEIL` through `gm_floorf` / `gm_ceilf` (or `fast_float_floor` / `fast_float_ceil` under `retail-crc`).
5. **Associative Container Ordering (`std::map` / `std::set`)**
   - C++ `std::map` and `std::set` iterate in deterministic key-sorted order. Rust's `std::collections::HashMap` and `HashSet` use randomized SipHash seeds per process, which will immediately desync replays if iterated during update or CRC.
   - Always translate `std::map<K, V>` to `BTreeMap<K, V>` (or a deterministically ordered vector/index map).
   - **Pointer-Keyed Maps/Sets:** Any C++ `std::map<Object*, ...>` or `std::set<Object*>` where iteration order affects logic or RNG calls must be keyed by deterministic ID (`ObjectID`, `NameKeyType`, `TeamID`, or arena creation sequence index), never by raw memory address.
6. **Intrusive Linked Lists & Pool Allocation Order**
   - `GameLogic::m_objList` is a doubly-linked intrusive list (`m_next`, `m_prev` on `Object`) whose order determines both the per-frame disabled-status sweep and the `MARKER:Objects` CRC hash order.
   - `registerObject` prepends new objects to the **head** of `m_objList` (`obj->prependToList(&m_objList)`), so `m_objList` iterates from newest object to oldest object.
7. **Evaluation Order of Function Arguments**
   - In C++, the evaluation order of function arguments (`foo(GameLogicRandomValue(0, 10), GameLogicRandomValue(0, 10))`) is unspecified.
   - Whenever a C++ call site passes multiple arguments that invoke `GameLogicRandomValue*` or mutate simulation state, bind each argument to an explicit `let` temporary in the exact evaluation order used by the reference `macos-vulkan` Clang `arm64` compiler (left-to-right).
8. **Signed Bit-Shifts & Narrow Integer Promotions**
   - Specify signed (`i8`, `i16`, `i32`) vs unsigned (`u8`, `u16`, `u32`) types explicitly (`Byte` = `i8`, `UnsignedByte` = `u8`, `Short` = `i16`, `UnsignedShort` = `u16`, `Int` = `i32`, `UnsignedInt` = `u32`, `Bool` = `u8`).
   - Note that `char` is signed on `x86_64` and `arm64` macOS (`i8`), whereas on Linux `arm64` `char` is unsigned (`u8`); always use explicit `i8` or `u8`.
9. **Sleepy-Update Priority Queue Tie-Breaking & Reverse-Index Deletion**
   - Never replace `GameLogic::m_sleepyUpdates` with `std::collections::BinaryHeap`.
   - Replicate the exact custom min-heap in `GameLogic.cpp`: strict `>` comparator (`f1 > f2`), left-child preference (`2*i + 1`) on equal priority, in-place root key update (`rebalanceSleepyUpdate(0)`), the already-awake `friend_awakenUpdateModule` guard, and the two-pass reverse-heap-index removal in `processDestroyList()`.
10. **FPU Rounding Mode & Denormal / NaN Discipline**
    - At the start of every `GameLogic::update()` and `GameLogic::getCRC()`, the engine enforces IEEE-754 round-to-nearest-even (`FE_TONEAREST`) via `ScopedFPUGuard` and `setFPMode()`.
    - Ensure no external library or GPU driver thread mutates the thread's FPCR/MXCSR rounding mode or flush-to-zero (FZ/DAZ) bits on the simulation thread.

---

## 8. Host Environment & Disk Space Findings

Checked on 20 November 2026 on the target Apple Silicon Mac:
1. **Free Disk Space Alert:**
   - `df -h /` reports **~13 GiB available** (down from ~36 GiB when the project brief was drafted).
   - A full Zero Hour Steam installation is ~2.8–3.5 GiB, and a C++ CMake + vcpkg build tree with debug symbols for both Generals and Zero Hour can easily consume 6–10 GiB if unconstrained.
2. **Mandatory Disk Budget Measures for Phase 0 & Beyond:**
   - **Never copy `.big` files:** Keep the retail game data in one single read-only directory (`~/Games/generalszh`) and pass `-assetPath ~/Games/generalszh` (or `export CNC_GENERALS_ZH_PATH=$HOME/Games/generalszh`) to `GeneralsXZH`.
   - **Build Zero Hour Only:** Configure CMake with `-DRTS_BUILD_GENERALS=OFF -DRTS_BUILD_CORE_TOOLS=OFF -DCMAKE_BUILD_TYPE=Release` and build only `--target z_generals`.
   - **Shallow Reference Clone:** Clone `fbraz3/GeneralsX` with `--depth 1` at `84a7a20769aa5749f388b1f49bd542c79b3c33e8` rather than fetching full git history.
3. **Game Data Status (`~/Games/generalszh`):**
   - `~/Games/generalszh` is not present yet on disk (`~/Games/` currently contains only `ra2yr`).
   - In Phase 0, game data will be fetched from Steam app `2732960` using `steamcmd +@sSteamCmdForcePlatformType windows +force_install_dir ~/Games/generalszh +login <user> +app_update 2732960 validate +quit` (with the owner entering their Steam credentials interactively).
