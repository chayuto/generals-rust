# Master Module Manifest (`generals-rust`)

- **Reference Upstream:** `fbraz3/GeneralsX` at commit `84a7a20769aa5749f388b1f49bd542c79b3c33e8`
- **Total Atomic Translation Packets:** `463` (`PKT-001` .. `PKT-463`)
- **Total C++ Header/Source Files Covered:** `833` (`100.0%` of `Core/GameEngine` + `GeneralsMD/Code/GameEngine` `Common` & `GameLogic`)
- **Total C++ Source Volume:** `11,456,985` bytes (~`358,029` est. LOC)
- **Machine-Readable Source of Truth:** [docs/MODULE_MANIFEST.json](MODULE_MANIFEST.json) (validated by `scripts/validate_manifest.py`)

---

## 1. Distribution by Agent Complexity Tier (`tier:*`)

| Agent Tier | Packets | C++ Files | Source Bytes | Est. LOC | Share of Packets |
|---|---:|---:|---:|---:|---:|
| `tier:1a-lite` | 217 | 366 | 2,462,554 | 76,957 | 46.9% |
| `tier:1b-fast` | 114 | 216 | 2,152,348 | 67,256 | 24.6% |
| `tier:2-engineer` | 81 | 156 | 3,040,144 | 95,004 | 17.5% |
| `tier:3-architect` | 51 | 95 | 3,801,939 | 118,812 | 11.0% |

## 2. Distribution by Execution Environment & Access Capability (`env:*`)

| Environment Label | Packets | C++ Files | Source Bytes | Est. LOC | Share of Packets |
|---|---:|---:|---:|---:|---:|
| `env:ephemeral-web` | 387 | 694 | 7,745,979 | 242,059 | 83.6% |
| `env:hybrid-ci` | 57 | 102 | 3,291,532 | 102,860 | 12.3% |
| `env:local-full` | 19 | 37 | 419,474 | 13,110 | 4.1% |

---

## 3. Complete Packet Directory (`PKT-001` .. `PKT-463`)

| Packet ID | Module Name | Work Package | Class | Agent Tier | Env Access | FFI Seam | Files | Bytes | Target Rust Path |
|---|---|---|---|---|---|---|---:|---:|---|
| `PKT-001` | `crc` | `WP-1.2a` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 5,066 | `crates/zh_common/src/crc.rs` |
| `PKT-002` | `RandomValue` | `WP-1.2a` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 15,306 | `crates/zh_common/src/random_value.rs` |
| `PKT-003` | `ExperienceTracker` | `WP-1.2b` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 14,146 | `crates/zh_logic/src/object/experience_tracker.rs` |
| `PKT-004` | `BitFlags` | `WP-2.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 12,533 | `crates/zh_common/src/xfer/bit_flags.rs` |
| `PKT-005` | `BitFlagsIO` | `WP-2.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 6,387 | `crates/zh_common/src/xfer/bit_flags_io.rs` |
| `PKT-006` | `CRCDebug` | `WP-2.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 12,504 | `crates/zh_common/src/xfer/crc_debug.rs` |
| `PKT-007` | `GameState` | `WP-2.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 64,193 | `crates/zh_common/src/xfer/game_state.rs` |
| `PKT-008` | `GameStateMap` | `WP-2.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 19,101 | `crates/zh_common/src/xfer/game_state_map.rs` |
| `PKT-009` | `Snapshot` | `WP-2.1` | `C4` | `tier:3-architect` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 5,271 | `crates/zh_common/src/xfer/snapshot.rs` |
| `PKT-010` | `Xfer` | `WP-2.1` | `C4` | `tier:3-architect` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 34,421 | `crates/zh_common/src/xfer/xfer.rs` |
| `PKT-011` | `XferCRC` | `WP-2.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 14,481 | `crates/zh_common/src/xfer/xfer_crc.rs` |
| `PKT-012` | `XferDeepCRC` | `WP-2.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 2,958 | `crates/zh_common/src/xfer/xfer_deep_crc.rs` |
| `PKT-013` | `XferLoad` | `WP-2.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 10,857 | `crates/zh_common/src/xfer/xfer_load.rs` |
| `PKT-014` | `XferSave` | `WP-2.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 14,193 | `crates/zh_common/src/xfer/xfer_save.rs` |
| `PKT-015` | `BezFwdIterator` | `WP-2.2` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 6,050 | `crates/zh_common/src/math/bez_fwd_iterator.rs` |
| `PKT-016` | `BezierSegment` | `WP-2.2` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 13,027 | `crates/zh_common/src/math/bezier_segment.rs` |
| `PKT-017` | `DiscreteCircle` | `WP-2.2` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 6,186 | `crates/zh_common/src/math/discrete_circle.rs` |
| `PKT-018` | `FPUControl` | `WP-2.2` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 1,844 | `crates/zh_common/src/math/fpu_control.rs` |
| `PKT-019` | `GameCommon` | `WP-2.2` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 23,114 | `crates/zh_common/src/math/game_common.rs` |
| `PKT-020` | `GameDefines` | `WP-2.2` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 10,507 | `crates/zh_common/src/math/game_defines.rs` |
| `PKT-021` | `GameType` | `WP-2.2` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 6,548 | `crates/zh_common/src/math/game_type.rs` |
| `PKT-022` | `Geometry` | `WP-2.2` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 25,162 | `crates/zh_common/src/math/geometry.rs` |
| `PKT-023` | `PartitionSolver` | `WP-2.2` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 8,675 | `crates/zh_common/src/math/partition_solver.rs` |
| `PKT-024` | `STLTypedefs` | `WP-2.2` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 8,191 | `crates/zh_common/src/math/stl_typedefs.rs` |
| `PKT-025` | `AddonCompat` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 1,898 | `crates/zh_common/src/runtime/addon_compat.rs` |
| `PKT-026` | `AsciiString` | `WP-2.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 31,777 | `crates/zh_common/src/core/ascii_string.rs` |
| `PKT-027` | `AudioAffect` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 1,560 | `crates/zh_common/src/runtime/audio_affect.rs` |
| `PKT-028` | `AudioEventInfo` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 5,346 | `crates/zh_common/src/runtime/audio_event_info.rs` |
| `PKT-029` | `AudioEventRTS` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 34,131 | `crates/zh_common/src/runtime/audio_event_rts.rs` |
| `PKT-030` | `AudioHandleSpecialValues` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 1,302 | `crates/zh_common/src/runtime/audio_handle_special_values.rs` |
| `PKT-031` | `AudioRandomValue` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 1,839 | `crates/zh_common/src/runtime/audio_random_value.rs` |
| `PKT-032` | `AudioRequest` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 3,054 | `crates/zh_common/src/runtime/audio_request.rs` |
| `PKT-033` | `AudioSettings` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 4,218 | `crates/zh_common/src/runtime/audio_settings.rs` |
| `PKT-034` | `ClientUpdateModule` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 2,583 | `crates/zh_common/src/runtime/client_update_module.rs` |
| `PKT-035` | `CriticalSection` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 4,398 | `crates/zh_common/src/runtime/critical_section.rs` |
| `PKT-036` | `CustomMatchPreferences` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 3,163 | `crates/zh_common/src/runtime/custom_match_preferences.rs` |
| `PKT-037` | `Debug` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 14,688 | `crates/zh_common/src/runtime/debug.rs` |
| `PKT-038` | `Dict` | `WP-2.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 25,770 | `crates/zh_common/src/core/dict.rs` |
| `PKT-039` | `DrawModule` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 14,032 | `crates/zh_common/src/runtime/draw_module.rs` |
| `PKT-040` | `DynamicAudioEventInfo` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 12,119 | `crates/zh_common/src/runtime/dynamic_audio_event_info.rs` |
| `PKT-041` | `Errors` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 2,639 | `crates/zh_common/src/runtime/errors.rs` |
| `PKT-042` | `GameAudio` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 55,309 | `crates/zh_common/src/runtime/game_audio.rs` |
| `PKT-043` | `GameLOD` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 36,866 | `crates/zh_common/src/runtime/game_lod.rs` |
| `PKT-044` | `GameLogicDispatch` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 76,054 | `crates/zh_common/src/runtime/game_logic_dispatch.rs` |
| `PKT-045` | `GameMemory` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 152,465 | `crates/zh_common/src/runtime/game_memory.rs` |
| `PKT-046` | `GameMemoryInit` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 4,458 | `crates/zh_common/src/runtime/game_memory_init.rs` |
| `PKT-047` | `GameMemoryInitDMA_Generals` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 1,253 | `crates/zh_common/src/runtime/game_memory_init_dma__generals.rs` |
| `PKT-048` | `GameMemoryInitDMA_GeneralsMD` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 1,263 | `crates/zh_common/src/runtime/game_memory_init_dma__generals_md.rs` |
| `PKT-049` | `GameMemoryInitPools_Generals` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 23,752 | `crates/zh_common/src/runtime/game_memory_init_pools__generals.rs` |
| `PKT-050` | `GameMemoryInitPools_GeneralsMD` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 23,654 | `crates/zh_common/src/runtime/game_memory_init_pools__generals_md.rs` |
| `PKT-051` | `GameMemoryNull` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 11,732 | `crates/zh_common/src/runtime/game_memory_null.rs` |
| `PKT-052` | `GameMode` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 2,133 | `crates/zh_common/src/runtime/game_mode.rs` |
| `PKT-053` | `GameMusic` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 8,386 | `crates/zh_common/src/runtime/game_music.rs` |
| `PKT-054` | `GameSounds` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 11,568 | `crates/zh_common/src/runtime/game_sounds.rs` |
| `PKT-055` | `GameSpyMiscPreferences` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 2,170 | `crates/zh_common/src/runtime/game_spy_misc_preferences.rs` |
| `PKT-056` | `GameUtility` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 5,746 | `crates/zh_common/src/runtime/game_utility.rs` |
| `PKT-057` | `IgnorePreferences` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 3,500 | `crates/zh_common/src/runtime/ignore_preferences.rs` |
| `PKT-058` | `LadderPreferences` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 2,627 | `crates/zh_common/src/runtime/ladder_preferences.rs` |
| `PKT-059` | `Language` | `WP-2.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 5,702 | `crates/zh_common/src/core/language.rs` |
| `PKT-060` | `List` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 17,948 | `crates/zh_common/src/runtime/list.rs` |
| `PKT-061` | `LogicRandomValue` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 4,098 | `crates/zh_common/src/runtime/logic_random_value.rs` |
| `PKT-062` | `MiniLog` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 4,204 | `crates/zh_common/src/runtime/mini_log.rs` |
| `PKT-063` | `MiscAudio` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 4,696 | `crates/zh_common/src/runtime/misc_audio.rs` |
| `PKT-064` | `NameKeyGenerator` | `WP-2.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 15,320 | `crates/zh_common/src/core/name_key_generator.rs` |
| `PKT-065` | `OptionPreferences` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 36,952 | `crates/zh_common/src/runtime/option_preferences.rs` |
| `PKT-066` | `OSDisplay` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 1,725 | `crates/zh_common/src/runtime/os_display.rs` |
| `PKT-067` | `PerfMetrics` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 1,509 | `crates/zh_common/src/runtime/perf_metrics.rs` |
| `PKT-068` | `PerfTimer` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 29,263 | `crates/zh_common/src/runtime/perf_timer.rs` |
| `PKT-069` | `QuickmatchPreferences` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 2,631 | `crates/zh_common/src/runtime/quickmatch_preferences.rs` |
| `PKT-070` | `QuotedPrintable` | `WP-2.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 7,591 | `crates/zh_common/src/core/quoted_printable.rs` |
| `PKT-071` | `Registry` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 17,636 | `crates/zh_common/src/runtime/registry.rs` |
| `PKT-072` | `ScopedMutex` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 1,292 | `crates/zh_common/src/runtime/scoped_mutex.rs` |
| `PKT-073` | `simpleplayer` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 20,761 | `crates/zh_common/src/runtime/simpleplayer.rs` |
| `PKT-074` | `SimulationMathCrc` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 3,083 | `crates/zh_common/src/runtime/simulation_math_crc.rs` |
| `PKT-075` | `SkirmishPreferences` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 2,663 | `crates/zh_common/src/runtime/skirmish_preferences.rs` |
| `PKT-076` | `StackDump` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 23,020 | `crates/zh_common/src/runtime/stack_dump.rs` |
| `PKT-077` | `StateMachine` | `WP-2.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 45,444 | `crates/zh_common/src/core/state_machine.rs` |
| `PKT-078` | `ThingSort` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 2,486 | `crates/zh_common/src/runtime/thing_sort.rs` |
| `PKT-079` | `TunnelTracker` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 21,040 | `crates/zh_common/src/runtime/tunnel_tracker.rs` |
| `PKT-080` | `UnicodeString` | `WP-2.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 32,341 | `crates/zh_common/src/core/unicode_string.rs` |
| `PKT-081` | `UnitTimings` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 1,404 | `crates/zh_common/src/runtime/unit_timings.rs` |
| `PKT-082` | `UpdateChecker` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 17,793 | `crates/zh_common/src/runtime/update_checker.rs` |
| `PKT-083` | `urllaunch` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 9,208 | `crates/zh_common/src/runtime/urllaunch.rs` |
| `PKT-084` | `UserPreferences` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 29,231 | `crates/zh_common/src/runtime/user_preferences.rs` |
| `PKT-085` | `version` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 17,549 | `crates/zh_common/src/runtime/version.rs` |
| `PKT-086` | `WellKnownKeys` | `WP-2.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 20,081 | `crates/zh_common/src/runtime/well_known_keys.rs` |
| `PKT-087` | `INI` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 2 | 96,972 | `crates/zh_common/src/ini/ini.rs` |
| `PKT-088` | `INIAiData` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 2,466 | `crates/zh_common/src/ini/ini_ai_data.rs` |
| `PKT-089` | `INIAnimation` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 3,022 | `crates/zh_common/src/ini/ini_animation.rs` |
| `PKT-090` | `INIAudioEventInfo` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 7,866 | `crates/zh_common/src/ini/ini_audio_event_info.rs` |
| `PKT-091` | `INICommandButton` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 4,460 | `crates/zh_common/src/ini/ini_command_button.rs` |
| `PKT-092` | `INICommandSet` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 1,988 | `crates/zh_common/src/ini/ini_command_set.rs` |
| `PKT-093` | `INIControlBarScheme` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 4,274 | `crates/zh_common/src/ini/ini_control_bar_scheme.rs` |
| `PKT-094` | `INICrate` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 2,213 | `crates/zh_common/src/ini/ini_crate.rs` |
| `PKT-095` | `INIDamageFX` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 1,974 | `crates/zh_common/src/ini/ini_damage_fx.rs` |
| `PKT-096` | `INIDrawGroupInfo` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 3,769 | `crates/zh_common/src/ini/ini_draw_group_info.rs` |
| `PKT-097` | `INIException` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 1,784 | `crates/zh_common/src/ini/ini_exception.rs` |
| `PKT-098` | `INIGameData` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 2,486 | `crates/zh_common/src/ini/ini_game_data.rs` |
| `PKT-099` | `INIMapCache` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 7,975 | `crates/zh_common/src/ini/ini_map_cache.rs` |
| `PKT-100` | `INIMapData` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 2,409 | `crates/zh_common/src/ini/ini_map_data.rs` |
| `PKT-101` | `INIMappedImage` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 3,120 | `crates/zh_common/src/ini/ini_mapped_image.rs` |
| `PKT-102` | `INIMiscAudio` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 5,679 | `crates/zh_common/src/ini/ini_misc_audio.rs` |
| `PKT-103` | `INIModel` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 2,129 | `crates/zh_common/src/ini/ini_model.rs` |
| `PKT-104` | `INIMultiplayer` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 3,948 | `crates/zh_common/src/ini/ini_multiplayer.rs` |
| `PKT-105` | `INIObject` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 2,763 | `crates/zh_common/src/ini/ini_object.rs` |
| `PKT-106` | `INIParticleSys` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 2,075 | `crates/zh_common/src/ini/ini_particle_sys.rs` |
| `PKT-107` | `INISpecialPower` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 1,869 | `crates/zh_common/src/ini/ini_special_power.rs` |
| `PKT-108` | `INITerrain` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 2,329 | `crates/zh_common/src/ini/ini_terrain.rs` |
| `PKT-109` | `INITerrainBridge` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 2,552 | `crates/zh_common/src/ini/ini_terrain_bridge.rs` |
| `PKT-110` | `INITerrainRoad` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 2,531 | `crates/zh_common/src/ini/ini_terrain_road.rs` |
| `PKT-111` | `INIUpgrade` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 1,875 | `crates/zh_common/src/ini/ini_upgrade.rs` |
| `PKT-112` | `INIVideo` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 2,645 | `crates/zh_common/src/ini/ini_video.rs` |
| `PKT-113` | `INIWater` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 5,458 | `crates/zh_common/src/ini/ini_water.rs` |
| `PKT-114` | `INIWeapon` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 2,193 | `crates/zh_common/src/ini/ini_weapon.rs` |
| `PKT-115` | `INIWebpageURL` | `WP-2.4` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 1: C-ABI Leaf | 1 | 3,938 | `crates/zh_common/src/ini/ini_webpage_url.rs` |
| `PKT-116` | `AcademyStats` | `WP-3.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 45,002 | `crates/zh_logic/src/economy/academy_stats.rs` |
| `PKT-117` | `BattleHonors` | `WP-3.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 1 | 5,159 | `crates/zh_logic/src/economy/battle_honors.rs` |
| `PKT-118` | `Energy` | `WP-3.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 13,530 | `crates/zh_logic/src/economy/energy.rs` |
| `PKT-119` | `Handicap` | `WP-3.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 7,317 | `crates/zh_logic/src/economy/handicap.rs` |
| `PKT-120` | `MissionStats` | `WP-3.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 6,421 | `crates/zh_logic/src/economy/mission_stats.rs` |
| `PKT-121` | `Money` | `WP-3.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 9,976 | `crates/zh_logic/src/economy/money.rs` |
| `PKT-122` | `ScoreKeeper` | `WP-3.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 22,056 | `crates/zh_logic/src/economy/score_keeper.rs` |
| `PKT-123` | `SkirmishBattleHonors` | `WP-3.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 11,996 | `crates/zh_logic/src/economy/skirmish_battle_honors.rs` |
| `PKT-124` | `StatsCollector` | `WP-3.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 17,362 | `crates/zh_logic/src/economy/stats_collector.rs` |
| `PKT-125` | `DisabledTypes` | `WP-3.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 6,283 | `crates/zh_logic/src/rules/disabled_types.rs` |
| `PKT-126` | `KindOf` | `WP-3.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 18,287 | `crates/zh_logic/src/rules/kind_of.rs` |
| `PKT-127` | `ModelState` | `WP-3.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 1 | 10,637 | `crates/zh_logic/src/rules/model_state.rs` |
| `PKT-128` | `MultiplayerSettings` | `WP-3.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 12,934 | `crates/zh_logic/src/rules/multiplayer_settings.rs` |
| `PKT-129` | `ObjectStatusTypes` | `WP-3.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 10,067 | `crates/zh_logic/src/rules/object_status_types.rs` |
| `PKT-130` | `PlayerTemplate` | `WP-3.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 29,001 | `crates/zh_logic/src/rules/player_template.rs` |
| `PKT-131` | `ProductionPrerequisite` | `WP-3.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 13,778 | `crates/zh_logic/src/rules/production_prerequisite.rs` |
| `PKT-132` | `RankInfo` | `WP-3.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 7,659 | `crates/zh_logic/src/rules/rank_info.rs` |
| `PKT-133` | `Science` | `WP-3.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 15,861 | `crates/zh_logic/src/rules/science.rs` |
| `PKT-134` | `SpecialPower` | `WP-3.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 22,461 | `crates/zh_logic/src/rules/special_power.rs` |
| `PKT-135` | `SpecialPowerMaskType` | `WP-3.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 1 | 1,400 | `crates/zh_logic/src/rules/special_power_mask_type.rs` |
| `PKT-136` | `SpecialPowerType` | `WP-3.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 1 | 4,479 | `crates/zh_logic/src/rules/special_power_type.rs` |
| `PKT-137` | `Upgrade` | `WP-3.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 29,138 | `crates/zh_logic/src/rules/upgrade.rs` |
| `PKT-138` | `Armor` | `WP-3.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 10,540 | `crates/zh_logic/src/combat/armor.rs` |
| `PKT-139` | `ArmorSet` | `WP-3.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 1 | 3,154 | `crates/zh_logic/src/combat/armor_set.rs` |
| `PKT-140` | `Damage` | `WP-3.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 19,143 | `crates/zh_logic/src/combat/damage.rs` |
| `PKT-141` | `DamageFX` | `WP-3.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 17,797 | `crates/zh_logic/src/combat/damage_fx.rs` |
| `PKT-142` | `FiringTracker` | `WP-3.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 17,267 | `crates/zh_logic/src/combat/firing_tracker.rs` |
| `PKT-143` | `Locomotor` | `WP-3.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 122,846 | `crates/zh_logic/src/combat/locomotor.rs` |
| `PKT-144` | `LocomotorSet` | `WP-3.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 2: Handle Component | 1 | 3,755 | `crates/zh_logic/src/combat/locomotor_set.rs` |
| `PKT-145` | `TurretAI` | `WP-3.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 2: Handle Component | 2 | 65,633 | `crates/zh_logic/src/combat/turret_ai.rs` |
| `PKT-146` | `WeaponBonusConditionFlags` | `WP-3.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 1 | 1,258 | `crates/zh_logic/src/combat/weapon_bonus_condition_flags.rs` |
| `PKT-147` | `WeaponSetFlags` | `WP-3.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 1 | 1,325 | `crates/zh_logic/src/combat/weapon_set_flags.rs` |
| `PKT-148` | `WeaponSetType` | `WP-3.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 1 | 2,318 | `crates/zh_logic/src/combat/weapon_set_type.rs` |
| `PKT-149` | `WeaponStatus` | `WP-3.3` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 2: Handle Component | 1 | 1,353 | `crates/zh_logic/src/combat/weapon_status.rs` |
| `PKT-150` | `CaveSystem` | `WP-4.1` | `C2` | `tier:1b-fast` | `env:hybrid-ci` | Pattern 2: Handle Component | 2 | 8,322 | `crates/zh_logic/src/spatial/cave_system.rs` |
| `PKT-151` | `CrateSystem` | `WP-4.1` | `C2` | `tier:1b-fast` | `env:hybrid-ci` | Pattern 2: Handle Component | 2 | 10,886 | `crates/zh_logic/src/spatial/crate_system.rs` |
| `PKT-152` | `PolygonTrigger` | `WP-4.1` | `C2` | `tier:1b-fast` | `env:hybrid-ci` | Pattern 2: Handle Component | 2 | 21,642 | `crates/zh_logic/src/spatial/polygon_trigger.rs` |
| `PKT-153` | `Powers` | `WP-4.1` | `C2` | `tier:1b-fast` | `env:hybrid-ci` | Pattern 2: Handle Component | 1 | 1,888 | `crates/zh_logic/src/spatial/powers.rs` |
| `PKT-154` | `SidesList` | `WP-4.1` | `C2` | `tier:1b-fast` | `env:hybrid-ci` | Pattern 2: Handle Component | 2 | 47,655 | `crates/zh_logic/src/spatial/sides_list.rs` |
| `PKT-155` | `Terrain` | `WP-4.1` | `C2` | `tier:1b-fast` | `env:hybrid-ci` | Pattern 2: Handle Component | 1 | 1,653 | `crates/zh_logic/src/spatial/terrain.rs` |
| `PKT-156` | `TerrainLogic` | `WP-4.1` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Pattern 2: Handle Component | 2 | 110,703 | `crates/zh_logic/src/spatial/terrain_logic.rs` |
| `PKT-157` | `TerrainTypes` | `WP-4.1` | `C2` | `tier:1b-fast` | `env:hybrid-ci` | Pattern 2: Handle Component | 2 | 12,906 | `crates/zh_logic/src/spatial/terrain_types.rs` |
| `PKT-158` | `GhostObject` | `WP-4.2` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Pattern 2: Handle Component | 2 | 12,871 | `crates/zh_logic/src/spatial/ghost_object.rs` |
| `PKT-159` | `ObjectIter` | `WP-4.2` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Pattern 2: Handle Component | 1 | 5,007 | `crates/zh_logic/src/spatial/object_iter.rs` |
| `PKT-160` | `PartitionManager` | `WP-4.2` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Pattern 2: Handle Component | 2 | 247,116 | `crates/zh_logic/src/spatial/partition_manager.rs` |
| `PKT-161` | `SimpleObjectIterator` | `WP-4.2` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Pattern 2: Handle Component | 1 | 7,165 | `crates/zh_logic/src/spatial/simple_object_iterator.rs` |
| `PKT-162` | `FunctionLexicon` | `WP-5.0` | `C4` | `tier:3-architect` | `env:local-full` | Pattern 3: RustBehaviorModuleAdapter | 2 | 43,603 | `crates/zh_logic/src/modules/function_lexicon.rs` |
| `PKT-163` | `Module` | `WP-5.0` | `C4` | `tier:3-architect` | `env:local-full` | Pattern 3: RustBehaviorModuleAdapter | 2 | 24,253 | `crates/zh_logic/src/modules/module.rs` |
| `PKT-164` | `ModuleFactory` | `WP-5.0` | `C4` | `tier:3-architect` | `env:local-full` | Pattern 3: RustBehaviorModuleAdapter | 2 | 32,493 | `crates/zh_logic/src/modules/module_factory.rs` |
| `PKT-165` | `ActiveBody` | `WP-5.1` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 65,957 | `crates/zh_logic/src/modules/body/active_body.rs` |
| `PKT-166` | `BodyModule` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 13,201 | `crates/zh_logic/src/modules/body/body_module.rs` |
| `PKT-167` | `BoneFXDamage` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 6,981 | `crates/zh_logic/src/modules/damage/bone_fx_damage.rs` |
| `PKT-168` | `CreateModule` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 6,438 | `crates/zh_logic/src/modules/create/create_module.rs` |
| `PKT-169` | `DamageModule` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 6,291 | `crates/zh_logic/src/modules/damage/damage_module.rs` |
| `PKT-170` | `DestroyModule` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 5,480 | `crates/zh_logic/src/modules/destroy/destroy_module.rs` |
| `PKT-171` | `GrantUpgradeCreate` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 9,893 | `crates/zh_logic/src/modules/create/grant_upgrade_create.rs` |
| `PKT-172` | `HighlanderBody` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 6,282 | `crates/zh_logic/src/modules/body/highlander_body.rs` |
| `PKT-173` | `HiveStructureBody` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 9,282 | `crates/zh_logic/src/modules/body/hive_structure_body.rs` |
| `PKT-174` | `ImmortalBody` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 6,360 | `crates/zh_logic/src/modules/body/immortal_body.rs` |
| `PKT-175` | `InactiveBody` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 10,426 | `crates/zh_logic/src/modules/body/inactive_body.rs` |
| `PKT-176` | `LockWeaponCreate` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 8,075 | `crates/zh_logic/src/modules/create/lock_weapon_create.rs` |
| `PKT-177` | `ObjectDefectionHelper` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 9,815 | `crates/zh_logic/src/modules/helper/object_defection_helper.rs` |
| `PKT-178` | `ObjectHelper` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 5,831 | `crates/zh_logic/src/modules/helper/object_helper.rs` |
| `PKT-179` | `ObjectRepulsorHelper` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 5,834 | `crates/zh_logic/src/modules/helper/object_repulsor_helper.rs` |
| `PKT-180` | `ObjectSMCHelper` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 5,742 | `crates/zh_logic/src/modules/helper/object_smc_helper.rs` |
| `PKT-181` | `ObjectWeaponStatusHelper` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 6,193 | `crates/zh_logic/src/modules/helper/object_weapon_status_helper.rs` |
| `PKT-182` | `PreorderCreate` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 6,029 | `crates/zh_logic/src/modules/create/preorder_create.rs` |
| `PKT-183` | `SpecialPowerCreate` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 6,258 | `crates/zh_logic/src/modules/create/special_power_create.rs` |
| `PKT-184` | `StatusDamageHelper` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 7,847 | `crates/zh_logic/src/modules/helper/status_damage_helper.rs` |
| `PKT-185` | `StructureBody` | `WP-5.1` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 6,850 | `crates/zh_logic/src/modules/body/structure_body.rs` |
| `PKT-186` | `SubdualDamageHelper` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 7,367 | `crates/zh_logic/src/modules/helper/subdual_damage_helper.rs` |
| `PKT-187` | `SupplyCenterCreate` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 6,549 | `crates/zh_logic/src/modules/create/supply_center_create.rs` |
| `PKT-188` | `SupplyWarehouseCreate` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 6,298 | `crates/zh_logic/src/modules/create/supply_warehouse_create.rs` |
| `PKT-189` | `TempWeaponBonusHelper` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 8,500 | `crates/zh_logic/src/modules/helper/temp_weapon_bonus_helper.rs` |
| `PKT-190` | `TransitionDamageFX` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 40,129 | `crates/zh_logic/src/modules/damage/transition_damage_fx.rs` |
| `PKT-191` | `UndeadBody` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 9,821 | `crates/zh_logic/src/modules/body/undead_body.rs` |
| `PKT-192` | `VeterancyGainCreate` | `WP-5.1` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 8,229 | `crates/zh_logic/src/modules/create/veterancy_gain_create.rs` |
| `PKT-193` | `ActiveShroudUpgrade` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 7,919 | `crates/zh_logic/src/modules/upgrade/active_shroud_upgrade.rs` |
| `PKT-194` | `ArmorUpgrade` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 9,370 | `crates/zh_logic/src/modules/upgrade/armor_upgrade.rs` |
| `PKT-195` | `BaikonurLaunchPower` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 8,382 | `crates/zh_logic/src/modules/special_power/baikonur_launch_power.rs` |
| `PKT-196` | `CashBountyPower` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 13,397 | `crates/zh_logic/src/modules/special_power/cash_bounty_power.rs` |
| `PKT-197` | `CashHackSpecialPower` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 11,666 | `crates/zh_logic/src/modules/special_power/cash_hack_special_power.rs` |
| `PKT-198` | `CleanupAreaPower` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 7,857 | `crates/zh_logic/src/modules/special_power/cleanup_area_power.rs` |
| `PKT-199` | `CollideModule` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 6,792 | `crates/zh_logic/src/modules/collide/collide_module.rs` |
| `PKT-200` | `CommandSetUpgrade` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 8,140 | `crates/zh_logic/src/modules/upgrade/command_set_upgrade.rs` |
| `PKT-201` | `ConvertToCarBombCrateCollide` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 10,316 | `crates/zh_logic/src/modules/collide/convert_to_car_bomb_crate_collide.rs` |
| `PKT-202` | `ConvertToHijackedVehicleCrateCollide` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 13,126 | `crates/zh_logic/src/modules/collide/convert_to_hijacked_vehicle_crate_collide.rs` |
| `PKT-203` | `CostModifierUpgrade` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 13,011 | `crates/zh_logic/src/modules/upgrade/cost_modifier_upgrade.rs` |
| `PKT-204` | `CrateCollide` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 15,207 | `crates/zh_logic/src/modules/collide/crate_collide.rs` |
| `PKT-205` | `CreateCrateDie` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 13,492 | `crates/zh_logic/src/modules/die/create_crate_die.rs` |
| `PKT-206` | `CreateObjectDie` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 10,581 | `crates/zh_logic/src/modules/die/create_object_die.rs` |
| `PKT-207` | `CrushDie` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 11,804 | `crates/zh_logic/src/modules/die/crush_die.rs` |
| `PKT-208` | `DamDie` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 7,688 | `crates/zh_logic/src/modules/die/dam_die.rs` |
| `PKT-209` | `DefectorSpecialPower` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 9,028 | `crates/zh_logic/src/modules/special_power/defector_special_power.rs` |
| `PKT-210` | `DemoralizeSpecialPower` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 12,179 | `crates/zh_logic/src/modules/special_power/demoralize_special_power.rs` |
| `PKT-211` | `DestroyDie` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 5,628 | `crates/zh_logic/src/modules/die/destroy_die.rs` |
| `PKT-212` | `DieModule` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 8,337 | `crates/zh_logic/src/modules/die/die_module.rs` |
| `PKT-213` | `EjectPilotDie` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 8,998 | `crates/zh_logic/src/modules/die/eject_pilot_die.rs` |
| `PKT-214` | `ExperienceScalarUpgrade` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 7,727 | `crates/zh_logic/src/modules/upgrade/experience_scalar_upgrade.rs` |
| `PKT-215` | `FireWeaponCollide` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 9,331 | `crates/zh_logic/src/modules/collide/fire_weapon_collide.rs` |
| `PKT-216` | `FireWeaponPower` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 9,738 | `crates/zh_logic/src/modules/special_power/fire_weapon_power.rs` |
| `PKT-217` | `FXListDie` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 9,297 | `crates/zh_logic/src/modules/die/fx_list_die.rs` |
| `PKT-218` | `GrantScienceUpgrade` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 7,768 | `crates/zh_logic/src/modules/upgrade/grant_science_upgrade.rs` |
| `PKT-219` | `HealCrateCollide` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 6,124 | `crates/zh_logic/src/modules/collide/heal_crate_collide.rs` |
| `PKT-220` | `KeepObjectDie` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 6,301 | `crates/zh_logic/src/modules/die/keep_object_die.rs` |
| `PKT-221` | `LocomotorSetUpgrade` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 6,107 | `crates/zh_logic/src/modules/upgrade/locomotor_set_upgrade.rs` |
| `PKT-222` | `MaxHealthUpgrade` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 7,913 | `crates/zh_logic/src/modules/upgrade/max_health_upgrade.rs` |
| `PKT-223` | `ModelConditionUpgrade` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 7,038 | `crates/zh_logic/src/modules/upgrade/model_condition_upgrade.rs` |
| `PKT-224` | `MoneyCrateCollide` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 8,083 | `crates/zh_logic/src/modules/collide/money_crate_collide.rs` |
| `PKT-225` | `ObjectCreationUpgrade` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 8,379 | `crates/zh_logic/src/modules/upgrade/object_creation_upgrade.rs` |
| `PKT-226` | `OCLSpecialPower` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 15,492 | `crates/zh_logic/src/modules/special_power/ocl_special_power.rs` |
| `PKT-227` | `PassengersFireUpgrade` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 6,258 | `crates/zh_logic/src/modules/upgrade/passengers_fire_upgrade.rs` |
| `PKT-228` | `PowerPlantUpgrade` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 8,286 | `crates/zh_logic/src/modules/upgrade/power_plant_upgrade.rs` |
| `PKT-229` | `RadarUpgrade` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 9,234 | `crates/zh_logic/src/modules/upgrade/radar_upgrade.rs` |
| `PKT-230` | `RebuildHoleExposeDie` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 10,851 | `crates/zh_logic/src/modules/die/rebuild_hole_expose_die.rs` |
| `PKT-231` | `ReplaceObjectUpgrade` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 9,408 | `crates/zh_logic/src/modules/upgrade/replace_object_upgrade.rs` |
| `PKT-232` | `SabotageCommandCenterCrateCollide` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 9,769 | `crates/zh_logic/src/modules/collide/sabotage_command_center_crate_collide.rs` |
| `PKT-233` | `SabotageFakeBuilding` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 1 | 6,540 | `crates/zh_logic/src/modules/collide/sabotage_fake_building.rs` |
| `PKT-234` | `SabotageInternetCenterCrateCollide` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 11,900 | `crates/zh_logic/src/modules/collide/sabotage_internet_center_crate_collide.rs` |
| `PKT-235` | `SabotageMilitaryFactoryCrateCollide` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 9,990 | `crates/zh_logic/src/modules/collide/sabotage_military_factory_crate_collide.rs` |
| `PKT-236` | `SabotagePowerPlantCrateCollide` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 10,086 | `crates/zh_logic/src/modules/collide/sabotage_power_plant_crate_collide.rs` |
| `PKT-237` | `SabotageSuperweaponCrateCollide` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 9,764 | `crates/zh_logic/src/modules/collide/sabotage_superweapon_crate_collide.rs` |
| `PKT-238` | `SabotageSupplyCenterCrateCollide` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 11,116 | `crates/zh_logic/src/modules/collide/sabotage_supply_center_crate_collide.rs` |
| `PKT-239` | `SabotageSupplyDropzoneCrateCollide` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 11,538 | `crates/zh_logic/src/modules/collide/sabotage_supply_dropzone_crate_collide.rs` |
| `PKT-240` | `SalvageCrateCollide` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 14,782 | `crates/zh_logic/src/modules/collide/salvage_crate_collide.rs` |
| `PKT-241` | `ShroudCrateCollide` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 6,201 | `crates/zh_logic/src/modules/collide/shroud_crate_collide.rs` |
| `PKT-242` | `SpecialAbility` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 8,020 | `crates/zh_logic/src/modules/special_power/special_ability.rs` |
| `PKT-243` | `SpecialPowerCompletionDie` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 8,034 | `crates/zh_logic/src/modules/die/special_power_completion_die.rs` |
| `PKT-244` | `SpecialPowerModule` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 43,378 | `crates/zh_logic/src/modules/special_power/special_power_module.rs` |
| `PKT-245` | `SpyVisionSpecialPower` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 9,430 | `crates/zh_logic/src/modules/special_power/spy_vision_special_power.rs` |
| `PKT-246` | `SquishCollide` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 8,283 | `crates/zh_logic/src/modules/collide/squish_collide.rs` |
| `PKT-247` | `StatusBitsUpgrade` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 10,210 | `crates/zh_logic/src/modules/upgrade/status_bits_upgrade.rs` |
| `PKT-248` | `StealthUpgrade` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 6,298 | `crates/zh_logic/src/modules/upgrade/stealth_upgrade.rs` |
| `PKT-249` | `SubObjectsUpgrade` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 9,217 | `crates/zh_logic/src/modules/upgrade/sub_objects_upgrade.rs` |
| `PKT-250` | `UnitCrateCollide` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 7,885 | `crates/zh_logic/src/modules/collide/unit_crate_collide.rs` |
| `PKT-251` | `UnpauseSpecialPowerUpgrade` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 8,559 | `crates/zh_logic/src/modules/upgrade/unpause_special_power_upgrade.rs` |
| `PKT-252` | `UpgradeDie` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 7,611 | `crates/zh_logic/src/modules/die/upgrade_die.rs` |
| `PKT-253` | `UpgradeModule` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 16,667 | `crates/zh_logic/src/modules/upgrade/upgrade_module.rs` |
| `PKT-254` | `VeterancyCrateCollide` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 10,810 | `crates/zh_logic/src/modules/collide/veterancy_crate_collide.rs` |
| `PKT-255` | `WeaponBonusUpgrade` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 9,151 | `crates/zh_logic/src/modules/upgrade/weapon_bonus_upgrade.rs` |
| `PKT-256` | `WeaponSetUpgrade` | `WP-5.2` | `C1` | `tier:1a-lite` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 6,286 | `crates/zh_logic/src/modules/upgrade/weapon_set_upgrade.rs` |
| `PKT-257` | `AutoHealBehavior` | `WP-5.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 21,634 | `crates/zh_logic/src/modules/behavior/auto_heal_behavior.rs` |
| `PKT-258` | `BattleBusSlowDeathBehavior` | `WP-5.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 15,820 | `crates/zh_logic/src/modules/behavior/battle_bus_slow_death_behavior.rs` |
| `PKT-259` | `BehaviorModule` | `WP-5.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 15,357 | `crates/zh_logic/src/modules/behavior/behavior_module.rs` |
| `PKT-260` | `BridgeBehavior` | `WP-5.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 56,039 | `crates/zh_logic/src/modules/behavior/bridge_behavior.rs` |
| `PKT-261` | `BridgeScaffoldBehavior` | `WP-5.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 15,425 | `crates/zh_logic/src/modules/behavior/bridge_scaffold_behavior.rs` |
| `PKT-262` | `BridgeTowerBehavior` | `WP-5.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 15,386 | `crates/zh_logic/src/modules/behavior/bridge_tower_behavior.rs` |
| `PKT-263` | `BunkerBusterBehavior` | `WP-5.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 15,107 | `crates/zh_logic/src/modules/behavior/bunker_buster_behavior.rs` |
| `PKT-264` | `CaveContain` | `WP-5.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 21,779 | `crates/zh_logic/src/modules/contain/cave_contain.rs` |
| `PKT-265` | `ContainModule` | `WP-5.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 1 | 10,284 | `crates/zh_logic/src/modules/contain/contain_module.rs` |
| `PKT-266` | `CountermeasuresBehavior` | `WP-5.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 22,989 | `crates/zh_logic/src/modules/behavior/countermeasures_behavior.rs` |
| `PKT-267` | `DumbProjectileBehavior` | `WP-5.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 36,155 | `crates/zh_logic/src/modules/behavior/dumb_projectile_behavior.rs` |
| `PKT-268` | `FireWeaponWhenDamagedBehavior` | `WP-5.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 19,350 | `crates/zh_logic/src/modules/behavior/fire_weapon_when_damaged_behavior.rs` |
| `PKT-269` | `FireWeaponWhenDeadBehavior` | `WP-5.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 10,337 | `crates/zh_logic/src/modules/behavior/fire_weapon_when_dead_behavior.rs` |
| `PKT-270` | `FlightDeckBehavior` | `WP-5.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 64,358 | `crates/zh_logic/src/modules/behavior/flight_deck_behavior.rs` |
| `PKT-271` | `GarrisonContain` | `WP-5.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 80,697 | `crates/zh_logic/src/modules/contain/garrison_contain.rs` |
| `PKT-272` | `GenerateMinefieldBehavior` | `WP-5.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 26,449 | `crates/zh_logic/src/modules/behavior/generate_minefield_behavior.rs` |
| `PKT-273` | `GrantStealthBehavior` | `WP-5.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 12,482 | `crates/zh_logic/src/modules/behavior/grant_stealth_behavior.rs` |
| `PKT-274` | `HealContain` | `WP-5.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 10,601 | `crates/zh_logic/src/modules/contain/heal_contain.rs` |
| `PKT-275` | `HelixContain` | `WP-5.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 21,018 | `crates/zh_logic/src/modules/contain/helix_contain.rs` |
| `PKT-276` | `InstantDeathBehavior` | `WP-5.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 11,151 | `crates/zh_logic/src/modules/behavior/instant_death_behavior.rs` |
| `PKT-277` | `InternetHackContain` | `WP-5.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 7,256 | `crates/zh_logic/src/modules/contain/internet_hack_contain.rs` |
| `PKT-278` | `JetSlowDeathBehavior` | `WP-5.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 18,218 | `crates/zh_logic/src/modules/behavior/jet_slow_death_behavior.rs` |
| `PKT-279` | `MinefieldBehavior` | `WP-5.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 29,795 | `crates/zh_logic/src/modules/behavior/minefield_behavior.rs` |
| `PKT-280` | `MobNexusContain` | `WP-5.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 22,303 | `crates/zh_logic/src/modules/contain/mob_nexus_contain.rs` |
| `PKT-281` | `NeutonBlastBehavior` | `WP-5.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 1 | 6,810 | `crates/zh_logic/src/modules/behavior/neuton_blast_behavior.rs` |
| `PKT-282` | `OpenContain` | `WP-5.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 83,656 | `crates/zh_logic/src/modules/contain/open_contain.rs` |
| `PKT-283` | `OverchargeBehavior` | `WP-5.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 15,032 | `crates/zh_logic/src/modules/behavior/overcharge_behavior.rs` |
| `PKT-284` | `OverlordContain` | `WP-5.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 33,459 | `crates/zh_logic/src/modules/contain/overlord_contain.rs` |
| `PKT-285` | `ParachuteContain` | `WP-5.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 31,089 | `crates/zh_logic/src/modules/contain/parachute_contain.rs` |
| `PKT-286` | `ParkingPlaceBehavior` | `WP-5.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 41,782 | `crates/zh_logic/src/modules/behavior/parking_place_behavior.rs` |
| `PKT-287` | `PoisonedBehavior` | `WP-5.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 13,696 | `crates/zh_logic/src/modules/behavior/poisoned_behavior.rs` |
| `PKT-288` | `POWTruckBehavior` | `WP-5.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 8,376 | `crates/zh_logic/src/modules/behavior/pow_truck_behavior.rs` |
| `PKT-289` | `PrisonBehavior` | `WP-5.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 17,830 | `crates/zh_logic/src/modules/behavior/prison_behavior.rs` |
| `PKT-290` | `PropagandaCenterBehavior` | `WP-5.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 12,618 | `crates/zh_logic/src/modules/behavior/propaganda_center_behavior.rs` |
| `PKT-291` | `PropagandaTowerBehavior` | `WP-5.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 26,498 | `crates/zh_logic/src/modules/behavior/propaganda_tower_behavior.rs` |
| `PKT-292` | `RailedTransportContain` | `WP-5.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 8,106 | `crates/zh_logic/src/modules/contain/railed_transport_contain.rs` |
| `PKT-293` | `RebuildHoleBehavior` | `WP-5.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 20,753 | `crates/zh_logic/src/modules/behavior/rebuild_hole_behavior.rs` |
| `PKT-294` | `RiderChangeContain` | `WP-5.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 24,239 | `crates/zh_logic/src/modules/contain/rider_change_contain.rs` |
| `PKT-295` | `SlowDeathBehavior` | `WP-5.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 27,354 | `crates/zh_logic/src/modules/behavior/slow_death_behavior.rs` |
| `PKT-296` | `SpawnBehavior` | `WP-5.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 46,341 | `crates/zh_logic/src/modules/behavior/spawn_behavior.rs` |
| `PKT-297` | `SupplyWarehouseCripplingBehavior` | `WP-5.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 11,745 | `crates/zh_logic/src/modules/behavior/supply_warehouse_crippling_behavior.rs` |
| `PKT-298` | `TechBuildingBehavior` | `WP-5.3` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 9,715 | `crates/zh_logic/src/modules/behavior/tech_building_behavior.rs` |
| `PKT-299` | `TransportContain` | `WP-5.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 31,113 | `crates/zh_logic/src/modules/contain/transport_contain.rs` |
| `PKT-300` | `TunnelContain` | `WP-5.3` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 29,656 | `crates/zh_logic/src/modules/contain/tunnel_contain.rs` |
| `PKT-301` | `AIUpdate` | `WP-5.4` | `C4` | `tier:3-architect` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 221,431 | `crates/zh_logic/src/modules/update/ai_update.rs` |
| `PKT-302` | `AnimationSteeringUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 9,927 | `crates/zh_logic/src/modules/update/animation_steering_update.rs` |
| `PKT-303` | `AssaultTransportAIUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 21,863 | `crates/zh_logic/src/modules/update/assault_transport_ai_update.rs` |
| `PKT-304` | `AssistedTargetingUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 10,759 | `crates/zh_logic/src/modules/update/assisted_targeting_update.rs` |
| `PKT-305` | `AutoDepositUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 15,532 | `crates/zh_logic/src/modules/update/auto_deposit_update.rs` |
| `PKT-306` | `AutoFindHealingUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 10,536 | `crates/zh_logic/src/modules/update/auto_find_healing_update.rs` |
| `PKT-307` | `BaseRegenerateUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 1 | 3,358 | `crates/zh_logic/src/modules/update/base_regenerate_update.rs` |
| `PKT-308` | `BaseRenerateUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 1 | 6,869 | `crates/zh_logic/src/modules/update/base_renerate_update.rs` |
| `PKT-309` | `BattlePlanUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 43,241 | `crates/zh_logic/src/modules/update/battle_plan_update.rs` |
| `PKT-310` | `BoneFXUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 45,718 | `crates/zh_logic/src/modules/update/bone_fx_update.rs` |
| `PKT-311` | `CheckpointUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 10,344 | `crates/zh_logic/src/modules/update/checkpoint_update.rs` |
| `PKT-312` | `ChinookAIUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 61,770 | `crates/zh_logic/src/modules/update/chinook_ai_update.rs` |
| `PKT-313` | `CleanupHazardUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 15,663 | `crates/zh_logic/src/modules/update/cleanup_hazard_update.rs` |
| `PKT-314` | `CommandButtonHuntUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 19,021 | `crates/zh_logic/src/modules/update/command_button_hunt_update.rs` |
| `PKT-315` | `DefaultProductionExitUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 12,787 | `crates/zh_logic/src/modules/update/default_production_exit_update.rs` |
| `PKT-316` | `DeletionUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 8,608 | `crates/zh_logic/src/modules/update/deletion_update.rs` |
| `PKT-317` | `DeliverPayloadAIUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 61,371 | `crates/zh_logic/src/modules/update/deliver_payload_ai_update.rs` |
| `PKT-318` | `DemoTrapUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 14,422 | `crates/zh_logic/src/modules/update/demo_trap_update.rs` |
| `PKT-319` | `DeployStyleAIUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 19,485 | `crates/zh_logic/src/modules/update/deploy_style_ai_update.rs` |
| `PKT-320` | `DockUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 28,078 | `crates/zh_logic/src/modules/update/dock_update.rs` |
| `PKT-321` | `DozerAIUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 105,594 | `crates/zh_logic/src/modules/update/dozer_ai_update.rs` |
| `PKT-322` | `DynamicGeometryInfoUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 12,747 | `crates/zh_logic/src/modules/update/dynamic_geometry_info_update.rs` |
| `PKT-323` | `DynamicShroudClearingRangeUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 17,998 | `crates/zh_logic/src/modules/update/dynamic_shroud_clearing_range_update.rs` |
| `PKT-324` | `EMPUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 27,197 | `crates/zh_logic/src/modules/update/emp_update.rs` |
| `PKT-325` | `EnemyNearUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 8,262 | `crates/zh_logic/src/modules/update/enemy_near_update.rs` |
| `PKT-326` | `FireOCLAfterWeaponCooldownUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 13,076 | `crates/zh_logic/src/modules/update/fire_ocl_after_weapon_cooldown_update.rs` |
| `PKT-327` | `FireSpreadUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 11,332 | `crates/zh_logic/src/modules/update/fire_spread_update.rs` |
| `PKT-328` | `FirestormDynamicGeometryInfoUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 16,520 | `crates/zh_logic/src/modules/update/firestorm_dynamic_geometry_info_update.rs` |
| `PKT-329` | `FireWeaponUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 8,936 | `crates/zh_logic/src/modules/update/fire_weapon_update.rs` |
| `PKT-330` | `FlammableUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 17,139 | `crates/zh_logic/src/modules/update/flammable_update.rs` |
| `PKT-331` | `FloatUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 8,325 | `crates/zh_logic/src/modules/update/float_update.rs` |
| `PKT-332` | `HackInternetAIUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 29,279 | `crates/zh_logic/src/modules/update/hack_internet_ai_update.rs` |
| `PKT-333` | `HeightDieUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 14,156 | `crates/zh_logic/src/modules/update/height_die_update.rs` |
| `PKT-334` | `HelicopterSlowDeathUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 26,512 | `crates/zh_logic/src/modules/update/helicopter_slow_death_update.rs` |
| `PKT-335` | `HijackerUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 11,459 | `crates/zh_logic/src/modules/update/hijacker_update.rs` |
| `PKT-336` | `HordeUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 21,101 | `crates/zh_logic/src/modules/update/horde_update.rs` |
| `PKT-337` | `JetAIUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 96,846 | `crates/zh_logic/src/modules/update/jet_ai_update.rs` |
| `PKT-338` | `LaserUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 21,343 | `crates/zh_logic/src/modules/update/laser_update.rs` |
| `PKT-339` | `LifetimeUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 8,216 | `crates/zh_logic/src/modules/update/lifetime_update.rs` |
| `PKT-340` | `MissileAIUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 38,618 | `crates/zh_logic/src/modules/update/missile_ai_update.rs` |
| `PKT-341` | `MissileLauncherBuildingUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 19,292 | `crates/zh_logic/src/modules/update/missile_launcher_building_update.rs` |
| `PKT-342` | `MobMemberSlavedUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 20,659 | `crates/zh_logic/src/modules/update/mob_member_slaved_update.rs` |
| `PKT-343` | `NeutronBlastBehavior` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 1 | 3,405 | `crates/zh_logic/src/modules/update/neutron_blast_behavior.rs` |
| `PKT-344` | `NeutronMissileSlowDeathUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 34,105 | `crates/zh_logic/src/modules/update/neutron_missile_slow_death_update.rs` |
| `PKT-345` | `NeutronMissileUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 26,518 | `crates/zh_logic/src/modules/update/neutron_missile_update.rs` |
| `PKT-346` | `OCLUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 14,839 | `crates/zh_logic/src/modules/update/ocl_update.rs` |
| `PKT-347` | `ParticleUplinkCannonUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 68,175 | `crates/zh_logic/src/modules/update/particle_uplink_cannon_update.rs` |
| `PKT-348` | `PhysicsUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 77,122 | `crates/zh_logic/src/modules/update/physics_update.rs` |
| `PKT-349` | `PilotFindVehicleUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 10,626 | `crates/zh_logic/src/modules/update/pilot_find_vehicle_update.rs` |
| `PKT-350` | `PointDefenseLaserUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 16,970 | `crates/zh_logic/src/modules/update/point_defense_laser_update.rs` |
| `PKT-351` | `PowerPlantUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 9,271 | `crates/zh_logic/src/modules/update/power_plant_update.rs` |
| `PKT-352` | `POWTruckAIUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 35,920 | `crates/zh_logic/src/modules/update/pow_truck_ai_update.rs` |
| `PKT-353` | `PrisonDockUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 6,879 | `crates/zh_logic/src/modules/update/prison_dock_update.rs` |
| `PKT-354` | `ProductionUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 60,927 | `crates/zh_logic/src/modules/update/production_update.rs` |
| `PKT-355` | `ProjectileStreamUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 11,658 | `crates/zh_logic/src/modules/update/projectile_stream_update.rs` |
| `PKT-356` | `ProneUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 8,688 | `crates/zh_logic/src/modules/update/prone_update.rs` |
| `PKT-357` | `QueueProductionExitUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 16,553 | `crates/zh_logic/src/modules/update/queue_production_exit_update.rs` |
| `PKT-358` | `RadarUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 9,117 | `crates/zh_logic/src/modules/update/radar_update.rs` |
| `PKT-359` | `RadiusDecalUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 8,084 | `crates/zh_logic/src/modules/update/radius_decal_update.rs` |
| `PKT-360` | `RailedTransportAIUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 17,344 | `crates/zh_logic/src/modules/update/railed_transport_ai_update.rs` |
| `PKT-361` | `RailedTransportDockUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 24,095 | `crates/zh_logic/src/modules/update/railed_transport_dock_update.rs` |
| `PKT-362` | `RailroadGuideAIUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 56,062 | `crates/zh_logic/src/modules/update/railroad_guide_ai_update.rs` |
| `PKT-363` | `RepairDockUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 9,941 | `crates/zh_logic/src/modules/update/repair_dock_update.rs` |
| `PKT-364` | `SabotageFakeBuildingCrateCollide` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 1 | 3,153 | `crates/zh_logic/src/modules/update/sabotage_fake_building_crate_collide.rs` |
| `PKT-365` | `SlavedUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 34,735 | `crates/zh_logic/src/modules/update/slaved_update.rs` |
| `PKT-366` | `SmartBombTargetHomingUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 8,280 | `crates/zh_logic/src/modules/update/smart_bomb_target_homing_update.rs` |
| `PKT-367` | `SpawnPointProductionExitUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 13,341 | `crates/zh_logic/src/modules/update/spawn_point_production_exit_update.rs` |
| `PKT-368` | `SpecialAbilityUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 85,607 | `crates/zh_logic/src/modules/update/special_ability_update.rs` |
| `PKT-369` | `SpecialPowerUpdateModule` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 7,626 | `crates/zh_logic/src/modules/update/special_power_update_module.rs` |
| `PKT-370` | `SpectreGunshipDeploymentUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 18,180 | `crates/zh_logic/src/modules/update/spectre_gunship_deployment_update.rs` |
| `PKT-371` | `SpectreGunshipUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 37,343 | `crates/zh_logic/src/modules/update/spectre_gunship_update.rs` |
| `PKT-372` | `SpyVisionUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 15,247 | `crates/zh_logic/src/modules/update/spy_vision_update.rs` |
| `PKT-373` | `StealthDetectorUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 21,447 | `crates/zh_logic/src/modules/update/stealth_detector_update.rs` |
| `PKT-374` | `StealthUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 46,745 | `crates/zh_logic/src/modules/update/stealth_update.rs` |
| `PKT-375` | `StickyBombUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 16,546 | `crates/zh_logic/src/modules/update/sticky_bomb_update.rs` |
| `PKT-376` | `StructureCollapseUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 20,293 | `crates/zh_logic/src/modules/update/structure_collapse_update.rs` |
| `PKT-377` | `StructureToppleUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 31,223 | `crates/zh_logic/src/modules/update/structure_topple_update.rs` |
| `PKT-378` | `SupplyCenterDockUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 11,395 | `crates/zh_logic/src/modules/update/supply_center_dock_update.rs` |
| `PKT-379` | `SupplyCenterProductionExitUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 13,279 | `crates/zh_logic/src/modules/update/supply_center_production_exit_update.rs` |
| `PKT-380` | `SupplyTruckAIUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 40,352 | `crates/zh_logic/src/modules/update/supply_truck_ai_update.rs` |
| `PKT-381` | `SupplyWarehouseDockUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 11,137 | `crates/zh_logic/src/modules/update/supply_warehouse_dock_update.rs` |
| `PKT-382` | `TensileFormationUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 16,588 | `crates/zh_logic/src/modules/update/tensile_formation_update.rs` |
| `PKT-383` | `ToppleUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 22,132 | `crates/zh_logic/src/modules/update/topple_update.rs` |
| `PKT-384` | `TransportAIUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 11,159 | `crates/zh_logic/src/modules/update/transport_ai_update.rs` |
| `PKT-385` | `UpdateModule` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 21,186 | `crates/zh_logic/src/modules/update/update_module.rs` |
| `PKT-386` | `WanderAIUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 5,836 | `crates/zh_logic/src/modules/update/wander_ai_update.rs` |
| `PKT-387` | `WaveGuideUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 35,976 | `crates/zh_logic/src/modules/update/wave_guide_update.rs` |
| `PKT-388` | `WeaponBonusUpdate` | `WP-5.4` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 13,156 | `crates/zh_logic/src/modules/update/weapon_bonus_update.rs` |
| `PKT-389` | `WorkerAIUpdate` | `WP-5.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Pattern 3: RustBehaviorModuleAdapter | 2 | 69,258 | `crates/zh_logic/src/modules/update/worker_ai_update.rs` |
| `PKT-390` | `Object` | `WP-5.5` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 256,283 | `crates/zh_logic/src/object/object.rs` |
| `PKT-391` | `ObjectCreationList` | `WP-5.5` | `C3` | `tier:2-engineer` | `env:hybrid-ci` | Core Spine | 2 | 71,400 | `crates/zh_logic/src/object/object_creation_list.rs` |
| `PKT-392` | `ObjectScriptStatusBits` | `WP-5.5` | `C3` | `tier:2-engineer` | `env:hybrid-ci` | Core Spine | 1 | 2,040 | `crates/zh_logic/src/object/object_script_status_bits.rs` |
| `PKT-393` | `ObjectTypes` | `WP-5.5` | `C3` | `tier:2-engineer` | `env:hybrid-ci` | Core Spine | 2 | 8,869 | `crates/zh_logic/src/object/object_types.rs` |
| `PKT-394` | `Overridable` | `WP-5.5` | `C3` | `tier:2-engineer` | `env:hybrid-ci` | Core Spine | 1 | 3,682 | `crates/zh_logic/src/object/overridable.rs` |
| `PKT-395` | `Override` | `WP-5.5` | `C3` | `tier:2-engineer` | `env:hybrid-ci` | Core Spine | 1 | 4,884 | `crates/zh_logic/src/object/override.rs` |
| `PKT-396` | `Thing` | `WP-5.5` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 21,053 | `crates/zh_logic/src/object/thing.rs` |
| `PKT-397` | `ThingFactory` | `WP-5.5` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 23,836 | `crates/zh_logic/src/object/thing_factory.rs` |
| `PKT-398` | `ThingTemplate` | `WP-5.5` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 105,317 | `crates/zh_logic/src/object/thing_template.rs` |
| `PKT-399` | `Weapon` | `WP-5.5` | `C3` | `tier:2-engineer` | `env:hybrid-ci` | Core Spine | 2 | 175,845 | `crates/zh_logic/src/object/weapon.rs` |
| `PKT-400` | `WeaponSet` | `WP-5.5` | `C3` | `tier:2-engineer` | `env:hybrid-ci` | Core Spine | 2 | 54,411 | `crates/zh_logic/src/object/weapon_set.rs` |
| `PKT-401` | `ActionManager` | `WP-6.1` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Core Spine | 2 | 74,128 | `crates/zh_logic/src/player/action_manager.rs` |
| `PKT-402` | `BorderColors` | `WP-6.1` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Core Spine | 1 | 1,214 | `crates/zh_logic/src/player/border_colors.rs` |
| `PKT-403` | `BuildAssistant` | `WP-6.1` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Core Spine | 2 | 70,294 | `crates/zh_logic/src/player/build_assistant.rs` |
| `PKT-404` | `Player` | `WP-6.1` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 178,929 | `crates/zh_logic/src/player/player.rs` |
| `PKT-405` | `PlayerList` | `WP-6.1` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Core Spine | 2 | 21,978 | `crates/zh_logic/src/player/player_list.rs` |
| `PKT-406` | `Radar` | `WP-6.1` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Core Spine | 2 | 64,965 | `crates/zh_logic/src/player/radar.rs` |
| `PKT-407` | `ResourceGatheringManager` | `WP-6.1` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Core Spine | 2 | 12,263 | `crates/zh_logic/src/player/resource_gathering_manager.rs` |
| `PKT-408` | `SparseMatchFinder` | `WP-6.1` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Core Spine | 1 | 7,597 | `crates/zh_logic/src/player/sparse_match_finder.rs` |
| `PKT-409` | `Squad` | `WP-6.1` | `C2` | `tier:1b-fast` | `env:ephemeral-web` | Core Spine | 2 | 12,536 | `crates/zh_logic/src/player/squad.rs` |
| `PKT-410` | `Team` | `WP-6.1` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 106,151 | `crates/zh_logic/src/player/team.rs` |
| `PKT-411` | `AIPathfind` | `WP-6.2` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 250,535 | `crates/zh_logic/src/pathfinder/ai_pathfind.rs` |
| `PKT-412` | `Path` | `WP-6.2` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 30,020 | `crates/zh_logic/src/pathfinder/path.rs` |
| `PKT-413` | `PathfindCell` | `WP-6.2` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 36,625 | `crates/zh_logic/src/pathfinder/pathfind_cell.rs` |
| `PKT-414` | `PathfindCellInfo` | `WP-6.2` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 6,074 | `crates/zh_logic/src/pathfinder/pathfind_cell_info.rs` |
| `PKT-415` | `PathfindCellList` | `WP-6.2` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 2,496 | `crates/zh_logic/src/pathfinder/pathfind_cell_list.rs` |
| `PKT-416` | `PathfindConstants` | `WP-6.2` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 3,170 | `crates/zh_logic/src/pathfinder/pathfind_constants.rs` |
| `PKT-417` | `PathfindLayer` | `WP-6.2` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 22,334 | `crates/zh_logic/src/pathfinder/pathfind_layer.rs` |
| `PKT-418` | `PathfindZoneManager` | `WP-6.2` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 30,313 | `crates/zh_logic/src/pathfinder/pathfind_zone_manager.rs` |
| `PKT-419` | `PathNode` | `WP-6.2` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 6,516 | `crates/zh_logic/src/pathfinder/path_node.rs` |
| `PKT-420` | `ZoneBlock` | `WP-6.2` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 13,594 | `crates/zh_logic/src/pathfinder/zone_block.rs` |
| `PKT-421` | `AI` | `WP-6.3` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 71,535 | `crates/zh_logic/src/ai/ai.rs` |
| `PKT-422` | `AIDock` | `WP-6.3` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 34,935 | `crates/zh_logic/src/ai/ai_dock.rs` |
| `PKT-423` | `AIGroup` | `WP-6.3` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 1 | 94,338 | `crates/zh_logic/src/ai/ai_group.rs` |
| `PKT-424` | `AIGuard` | `WP-6.3` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 42,842 | `crates/zh_logic/src/ai/ai_guard.rs` |
| `PKT-425` | `AIGuardRetaliate` | `WP-6.3` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 41,207 | `crates/zh_logic/src/ai/ai_guard_retaliate.rs` |
| `PKT-426` | `AIPlayer` | `WP-6.3` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 144,116 | `crates/zh_logic/src/ai/ai_player.rs` |
| `PKT-427` | `AISkirmishPlayer` | `WP-6.3` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 43,785 | `crates/zh_logic/src/ai/ai_skirmish_player.rs` |
| `PKT-428` | `AIStateMachine` | `WP-6.3` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 1 | 49,700 | `crates/zh_logic/src/ai/ai_state_machine.rs` |
| `PKT-429` | `AIStates` | `WP-6.3` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 1 | 258,265 | `crates/zh_logic/src/ai/ai_states.rs` |
| `PKT-430` | `AITNGuard` | `WP-6.3` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 41,631 | `crates/zh_logic/src/ai/aitn_guard.rs` |
| `PKT-431` | `ScriptActions` | `WP-6.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Core Spine | 2 | 294,167 | `crates/zh_logic/src/scripts/script_actions.rs` |
| `PKT-432` | `ScriptConditions` | `WP-6.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Core Spine | 2 | 122,652 | `crates/zh_logic/src/scripts/script_conditions.rs` |
| `PKT-433` | `ScriptEngine` | `WP-6.4` | `C4` | `tier:3-architect` | `env:ephemeral-web` | Core Spine | 2 | 435,234 | `crates/zh_logic/src/scripts/script_engine.rs` |
| `PKT-434` | `Scripts` | `WP-6.4` | `C4` | `tier:3-architect` | `env:ephemeral-web` | Core Spine | 2 | 139,784 | `crates/zh_logic/src/scripts/scripts.rs` |
| `PKT-435` | `VictoryConditions` | `WP-6.4` | `C3` | `tier:2-engineer` | `env:ephemeral-web` | Core Spine | 2 | 15,864 | `crates/zh_logic/src/scripts/victory_conditions.rs` |
| `PKT-436` | `CommandLine` | `WP-7.1` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 41,563 | `crates/zh_logic/src/gamelogic/command_line.rs` |
| `PKT-437` | `FramePacer` | `WP-7.1` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 11,494 | `crates/zh_logic/src/gamelogic/frame_pacer.rs` |
| `PKT-438` | `FrameRateLimit` | `WP-7.1` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 8,839 | `crates/zh_logic/src/gamelogic/frame_rate_limit.rs` |
| `PKT-439` | `GameEngine` | `WP-7.1` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 56,687 | `crates/zh_logic/src/gamelogic/game_engine.rs` |
| `PKT-440` | `GameLogic` | `WP-7.1` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 203,575 | `crates/zh_logic/src/gamelogic/game_logic.rs` |
| `PKT-441` | `GameMain` | `WP-7.1` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 1 | 2,016 | `crates/zh_logic/src/gamelogic/game_main.rs` |
| `PKT-442` | `GlobalData` | `WP-7.1` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 119,548 | `crates/zh_logic/src/gamelogic/global_data.rs` |
| `PKT-443` | `LatchRestore` | `WP-7.1` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 1 | 2,449 | `crates/zh_logic/src/gamelogic/latch_restore.rs` |
| `PKT-444` | `MessageStream` | `WP-7.1` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 85,238 | `crates/zh_logic/src/gamelogic/message_stream.rs` |
| `PKT-445` | `ReplaySimulation` | `WP-7.1` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 11,461 | `crates/zh_logic/src/gamelogic/replay_simulation.rs` |
| `PKT-446` | `SubsystemInterface` | `WP-7.1` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 14,741 | `crates/zh_logic/src/gamelogic/subsystem_interface.rs` |
| `PKT-447` | `WorkerProcess` | `WP-7.1` | `C4` | `tier:3-architect` | `env:hybrid-ci` | Core Spine | 2 | 9,376 | `crates/zh_logic/src/gamelogic/worker_process.rs` |
| `PKT-448` | `ArchiveFile` | `WP-7.2` | `C3` | `tier:2-engineer` | `env:local-full` | Core Spine | 2 | 13,360 | `crates/zh_common/src/io/archive_file.rs` |
| `PKT-449` | `ArchiveFileSystem` | `WP-7.2` | `C3` | `tier:2-engineer` | `env:local-full` | Core Spine | 2 | 24,226 | `crates/zh_common/src/io/archive_file_system.rs` |
| `PKT-450` | `BIGFileSystem` | `WP-7.2` | `C3` | `tier:2-engineer` | `env:local-full` | Core Spine | 2 | 35,025 | `crates/zh_common/src/io/big_file_system.rs` |
| `PKT-451` | `DataChunk` | `WP-7.2` | `C3` | `tier:2-engineer` | `env:local-full` | Core Spine | 2 | 33,783 | `crates/zh_common/src/io/data_chunk.rs` |
| `PKT-452` | `Directory` | `WP-7.2` | `C3` | `tier:2-engineer` | `env:local-full` | Core Spine | 2 | 6,430 | `crates/zh_common/src/io/directory.rs` |
| `PKT-453` | `encrypt` | `WP-7.2` | `C3` | `tier:2-engineer` | `env:local-full` | Core Spine | 2 | 5,154 | `crates/zh_common/src/io/encrypt.rs` |
| `PKT-454` | `file` | `WP-7.2` | `C3` | `tier:2-engineer` | `env:local-full` | Core Spine | 3 | 27,678 | `crates/zh_common/src/io/file.rs` |
| `PKT-455` | `FileSystem` | `WP-7.2` | `C3` | `tier:2-engineer` | `env:local-full` | Core Spine | 2 | 27,436 | `crates/zh_common/src/io/file_system.rs` |
| `PKT-456` | `LocalFile` | `WP-7.2` | `C3` | `tier:2-engineer` | `env:local-full` | Core Spine | 2 | 23,027 | `crates/zh_common/src/io/local_file.rs` |
| `PKT-457` | `LocalFileSystem` | `WP-7.2` | `C3` | `tier:2-engineer` | `env:local-full` | Core Spine | 2 | 6,042 | `crates/zh_common/src/io/local_file_system.rs` |
| `PKT-458` | `MapObject` | `WP-7.2` | `C3` | `tier:2-engineer` | `env:local-full` | Core Spine | 1 | 7,675 | `crates/zh_common/src/io/map_object.rs` |
| `PKT-459` | `MapReaderWriterInfo` | `WP-7.2` | `C3` | `tier:2-engineer` | `env:local-full` | Core Spine | 1 | 5,280 | `crates/zh_common/src/io/map_reader_writer_info.rs` |
| `PKT-460` | `RAMFile` | `WP-7.2` | `C3` | `tier:2-engineer` | `env:local-full` | Core Spine | 2 | 19,331 | `crates/zh_common/src/io/ram_file.rs` |
| `PKT-461` | `Recorder` | `WP-7.2` | `C3` | `tier:2-engineer` | `env:local-full` | Core Spine | 2 | 66,383 | `crates/zh_common/src/io/recorder.rs` |
| `PKT-462` | `StreamingArchiveFile` | `WP-7.2` | `C3` | `tier:2-engineer` | `env:local-full` | Core Spine | 2 | 13,116 | `crates/zh_common/src/io/streaming_archive_file.rs` |
| `PKT-463` | `WorkingDirectory` | `WP-7.2` | `C3` | `tier:2-engineer` | `env:local-full` | Core Spine | 2 | 5,179 | `crates/zh_common/src/io/working_directory.rs` |
