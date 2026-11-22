# Multi-Agent Token Optimization, Compartmentalized V&V, and Public CI Offload Architecture

This document defines the operational engineering system for translating ~330,000 lines of C++ (`871` files across `Core/GameEngine` and `GeneralsMD/Code/GameEngine`) to Rust with minimal token burn, strict compartmentalization, automated Verification & Validation (V&V), and near-zero local Mac CPU/disk load via free GitHub Actions CI.

---

## 1. Token Optimization: Context Slicing & 4-Tier Agent Routing

### 1.1 Why Naive Agent Translation Burns 10x Too Many Tokens
In the C++ codebase, every `.cpp` file includes `PreRTS.h`, `Object.h` (40 KB), `GameLogic.h` (24 KB), `ThingTemplate.h` (45 KB), `GlobalData.h` (35 KB), and `INI.h` (21 KB).
- If an agent uses file-search/read tools to explore headers before translating each of the ~450 `.cpp` implementation files, it consumes **40,000–70,000 input tokens per file** ($\approx 25\text{M}–35\text{M}$ input tokens just on repeated header reads, plus multi-turn tool-call overhead accumulating context history).

### 1.2 Tier 0: Zero-Token Deterministic Context Slicer (`scripts/prep_packet.py`)
Before any LLM agent is invoked on a translation unit, a deterministic local Python/Clang script prepares a self-contained **Translation Packet (`packet_<module>.md`, ~3,000–4,500 tokens total)** containing:
1. **Stripped Target Source (`target.h` + `target.cpp`):** License header blocks, `#include` boilerplate, and dead `#if 0` blocks removed.
2. **AST-Extracted Dependency Signatures (`deps_context.rs`):** Using `compile_commands.json` from CMake (`clang++ -Xclang -ast-dump=json`), the script extracts *only* the exact external methods, enum variants, and struct fields actually referenced by `target.cpp`, mapped to their Rust equivalents in `zh_common` / `zh_logic`.
3. **Pre-Generated Boilerplate Skeleton (`skeleton.rs`):**
   - `ModuleData` struct definition and `INI` field-parse table extracted mechanically from `buildFieldParse(MultiIniFieldParse& p)`.
   - `Snapshot::crc` and `Snapshot::xfer` field traversal order extracted from `cls::crc` / `cls::xfer`.
   - `FacetMask` constants and `extern "C"` FFI entry points pre-populated.
4. **Compact Determinism Cheat Sheet (`RUST_CONVENTIONS_CHEATSHEET.md`, ~800 tokens):** The 10 determinism rules reduced to concrete code idioms (`wrapping_add`, `gm_*f`, `BTreeMap`, float-to-int truncation helper `cpp_f32_mul_to_i32(i, f)`).

**Token Reduction Impact:**
- Per-module input prompt drops from **~50,000 tokens (multi-turn exploration)** to **~3,800 tokens (single-turn, zero-tool-call transformation)** — an **~88% reduction in token consumption**.

---

### 1.3 Four-Tier Agent Routing Matrix (Complexity vs. Model Cost)

We classify every work unit into four complexity classes (`C1`..`C4`) and route each to the lowest-cost agent tier capable of >90% first-pass accuracy:

```mermaid
flowchart TD
    Prep["Tier 0: scripts/prep_packet.py\n(0 LLM Tokens: AST Slice + Skeleton)"]
    Prep --> Router{"Complexity Class\nRouter"}

    Router -->|"Class C1 (~35% LOC)\nLeaf data, INI, Upgrades, Create, Die"| T1A["Tier 1A: Fast/Lite Worker\n(Single-Turn, No Search Tools)"]
    Router -->|"Class C2 (~40% LOC)\nStandard Update, Behavior, Collide, SpecialPower"| T1B["Tier 1B: Fast Worker\n(Single-Turn + 1 Compiler Retry)"]
    Router -->|"Class C3 (~18% LOC)\nContain, Vehicle/Air AIUpdates, Locomotor, Weapon, Scripts"| T2["Tier 2: Mid-Tier Engineer\n(Multi-Turn with cargo test & V1/V2 Loop)"]
    Router -->|"Class C4 (~7% LOC)\nSimContext, ObjectArena, SleepyHeap, Pathfinder, Core AI"| T3["Tier 3: Lead Architect Model\n(Full Architecture & Desync Root-Cause)"]

    T1A --> V1["Gate V1: Static Determinism Linter\n+ cargo check"]
    T1B --> V1
    V1 -->|"Compile Error (1 retry)"| T2
    V1 -->|"Pass"| V2["Gate V2: In-Process C++ vs Rust\nDifferential Micro-Harness"]
    T2 --> V2
    V2 -->|"Diff Failure"| T3
    V2 -->|"Pass"| V3["Gate V3/V4: Shadow-Verify &\nCI Headless Replay Matrix"]
    T3 --> V3
```

| Complexity Class | Codebase Share | Target Files & Modules | Assigned Agent Tier | Execution Mode |
|---|---:|---|---|---|
| **Class C1: Leaf & Data Modules** | **~35%** (~115k LOC, ~310 files) | `RandomValue`, `crc`, `XferCRC`, `BitFlags`, `Money`, `Energy`, `ScoreKeeper`, `Handicap`, `Science`, `RankInfo`, `Armor`, `FiringTracker`, `ExperienceTracker`, `Common/INI/*` (28 files), `Object/Destroy/*` (1), `Object/Damage/*` (3), `Object/Create/*` (8), `Object/Helper/*` (8), `Object/Die/*` (12), `Object/Upgrade/*` (21), `Object/SpecialPower/*` (11), `Object/Collide/*` (20), simple `Object/Body/*` (6). | **Tier 1A (Fastest / Lite Model)** | **Single-turn, zero-tool sandbox.** Reads `packet_<mod>.md`, outputs `<mod>.rs`. Zero codebase search allowed. |
| **Class C2: Standard Stateful Modules** | **~40%** (~130k LOC, ~380 files) | ~60 of 86 `Object/Update/*` files (timers, detectors, stealth, weapon triggers, dock/exit queues, EMP, radar, power plant, OCL updates), ~22 of 30 `Object/Behavior/*` files (`AutoHeal`, `Minefield`, `Overcharge`, `Poisoned`, `TechBuilding`, `Countermeasures`, `DumbProjectile`, `InstantDeath`, `GrantStealth`), `ActiveBody`, `StructureBody`, `TeamFactory`, `CrateSystem`, `CaveSystem`, `SidesList`, `PolygonTrigger`, `VictoryConditions`. | **Tier 1B (Fast Model)** | **Single-turn + 1 compiler-error feedback loop.** If `cargo check` or `Gate V1` fails after 1 retry, escalates automatically to Tier 2. |
| **Class C3: Complex Multi-Facet & Domain Subsystems** | **~18%** (~60k LOC, ~130 files) | `Object/Contain/*` (13 files: `OpenContain`, `TransportContain`, `GarrisonContain`, `OverlordContain`, `HelixContain`, `TunnelContain`, `InternetHackContain`), complex `Object/Behavior/*` (8 files: `BridgeBehavior`, `ParkingPlaceBehavior`, `PropagandaTowerBehavior`, `RebuildHoleBehavior`, `SlowDeathBehavior`, `SpawnBehavior`, `PrisonBehavior`, `POWTruckBehavior`), complex `Object/Update/*` (26 files: `PhysicsUpdate`, `MissileAIUpdate`, `LaserUpdate`, `JetAIUpdate`, `ChinookAIUpdate`, `SupplyTruckAIUpdate`, `WorkerAIUpdate`, `DozerAIUpdate`, `ProductionUpdate`, `HordeUpdate`, `SlavedUpdate`), `Weapon.cpp`, `WeaponSet.cpp`, `Locomotor.cpp`, `TurretAI.cpp`, `ScriptConditions.cpp` (split into 10 function-group packets), `ScriptActions.cpp` (split into 15 function-group packets). | **Tier 2 (Mid-Tier Engineer Model)** | **Bounded tool loop (`cargo test` + `Gate V1/V2`).** Translates packet and runs in-process differential test harness until `Gate V2` passes. |
| **Class C4: Architectural Spine & Desync Triage** | **~7%** (~25k LOC, ~51 files) | `SimContext`, `ObjectArena`, `ModuleArena`, `SleepyUpdateHeap` (`m_sleepyUpdates`), `RustBehaviorModuleAdapter` (universal C++ proxy), `Object.cpp`, `PartitionManager.cpp`, `TerrainLogic.cpp`, `Pathfinder` (`Pathfind*.cpp`, `AIPathfind.cpp`), `AI.cpp`, `AIGroup.cpp`, `AIPlayer.cpp`, `AISkirmishPlayer.cpp`, `AIUpdate.cpp`, `AIStateMachine.cpp`, `AIStates.cpp`, `Player.cpp`, `Team.cpp`, `ScriptEngine.cpp`, `GameLogic.cpp`, `Recorder.cpp`. | **Tier 3 (Lead Architect / Frontier Model)** | **Full architectural authority.** Designs seams, reviews interface contracts, implements C4 spine, and resolves any `Gate V2/V3/V4` desync escalations. |

---

### 1.4 Two-Dimensional Task Classification: Agent Tier $\times$ Execution Environment (`env:local-full` vs. `env:ephemeral-web`)

Agents working on `generals-rust` operate in one of two distinct physical environments with different access limitations:

1. **`env:local-full` — Full Local Workstation Agents (Retail Assets + Local Comparative C++ Build):**
   - **Access:** Has read-only access to retail `.big` archives at `~/Games/generalszh` (`-assetPath ~/Games/generalszh`), the local `vendor/GeneralsX` C++ build (`GeneralsXZH`, `compile_commands.json`, `libz_gameengine.a`, `z_googletest`), and the local replay corpus (`~/Library/Application Support/GeneralsX/GeneralsZH/`).
   - **Exclusive Tasks (`env:local-full`):**
     - `WP-0.1..0.4`: Retail asset download (`steamcmd`), local `GeneralsXZH` build, replay corpus assembly, running `scripts/prep_packet.py` against `compile_commands.json`.
     - `WP-1.1` & `WP-5.0`: C++ `CMakeLists.txt` FFI seam bring-up, `RustBehaviorModuleAdapter` C++ compilation, and generating **In-Repo Golden Test Vectors** from the C++ reference engine.
     - `WP-2.4` (retail `INI.big` `m_iniCRC` check), `WP-7.2` (retail `.big`/`.map`/`.csf` parser verification), `WP-7.3` (local standalone `zh-sim` benchmark), and interactive `-SaveDebugCRCPerFrame` desync debugging.
2. **`env:ephemeral-web` — Ephemeral Web / Cloud Sandbox Agents (Codebase-Only Access):**
   - **Limitations:** Has access **only** to the `chayuto/generals-rust` git repository and a standard Rust/Python toolchain. It has **no access** to `~/Games/generalszh` (`.big` files are never in git) and **no access** to a local `vendor/GeneralsX` C++ binary.
   - **How We Enable `env:ephemeral-web` Agents to Execute & Verify ~85% of the Project (295+ of 337 Packets):**
     - **Step 1 (Producer Step by `env:local-full`):** Before marking a translation packet `status:ready-for-agent` + `env:ephemeral-web`, an `env:local-full` agent runs `scripts/prep_packet.py` and commits into `chayuto/generals-rust`:
       1. `packets/<phase>/<module>.md`: The self-contained GPLv3 C++ `.h`/`.cpp` source slice from `fbraz3/GeneralsX` (`84a7a20`), the AST-sliced dependency signatures, and the Rust struct/INI/Xfer skeleton.
       2. `crates/*/tests/golden_<module>.rs` (or `tests/golden/<module>.json`): **1,000-seed Golden Test Vectors** (inputs -> outputs, `XferSave` snapshot bytes, `XferCRC`, and `GetGameLogicRandomSeedCRC()`) recorded directly from the C++ reference build.
     - **Step 2 (Consumer Step by `env:ephemeral-web`):** The ephemeral web agent claims an issue labeled `status:ready-for-agent` + `env:ephemeral-web`, reads `packets/<phase>/<module>.md` directly from the repo, writes the single isolated `.rs` module file, and runs its local V&V oracle (`python3 scripts/lint_determinism.py && cargo test` against the committed Golden Vectors) entirely inside its codebase-only container!
     - **Step 3 (Public CI Verification):** When the ephemeral web agent pushes its branch/PR to public `chayuto/generals-rust`, GitHub Actions CI automatically runs the full C++ FFI link (`Gate V2`) and the 6-way headless replay matrix (`Gate V3/V4`).
3. **`env:hybrid-ci` — Two-Stage Work Packages:**
   - Used on multi-packet work packages where an `env:local-full` agent seeds the C++ FFI harness + Golden Vectors first, unlocking parallel `env:ephemeral-web` translation across all child packets.

---

### 1.5 Asymmetric Cost Model & Ultra-Lean Remote Offload (`Remote = $0 FREE`, `Local = EXPENSIVE`)

**Governing Economic Rule (`AGENTS.md` §4.4):** **Remote `env:ephemeral-web` worker tokens are `$0` FREE; local `env:local-full` tokens are EXPENSIVE.** Optimize **verified Rust lines landed per local token**, NOT total tokens. Never spend local tokens implementing medium/large Rust modules that can be lean-seeded in `< 300` local LOC and offloaded to free remote workers:

| Target Shape | Examples | Decision | Rationale |
| :--- | :--- | :--- | :--- |
| **Standalone & Domain Modules (`C1–C3`, any medium/large LOC)** | `PKT-406` (`Radar`), `PKT-145` (`TurretAI`), `PKT-143` (`Locomotor`), `PKT-401` (`ActionManager`), `PKT-403` (`BuildAssistant`), `PKT-391` (`ObjectCreationList`), `PKT-399` (`Weapon`), `PKT-431`/`PKT-432` (`Scripts`) | **Lean-Seed Locally (`< 300` LOC) -> Offload Remotely (`$0`)** | `scripts/build_oracle.py` (`libz_gameengine.a`) + `scripts/prep_packet.py` + `~50`-LOC guard test delivers `500–2,000+` free remote LOC without stubs or Rust skeletons. |
| **Small Leaves (`C1`/`C2`, Phase 2–3)** | Remaining `WP-2.1`..`WP-2.4` & `WP-3.x` leaves | **Batch via Shared Oracle -> Offload Remotely (`$0`)** | One shared C++ driver covers a family; remote workers execute files concurrently at `$0` local implementation cost. |
| **Phase 5–6 Waves (`WP-5.1`–`5.4`, `WP-6.4`)** | `221` `BehaviorModule` packets + `29` Script packets | **Auto-Generate via `WP-5.0` -> Offload Remotely (`$0`)** | One local generator pass produces all packets, oracles, and issues with `O(1)` marginal local spend. |
| **Core Spines, Seams & Verification (`C4` / `env:local-full`)** | `scripts/build_oracle.py`, `Gate 0` replays, `SimContext`/`ObjectArena`/`ModuleArena`, `WP-5.0` adapter, `Pathfinder`, `GameLogic::update()`, desync triage | **Build Locally (`Force Multiplier`)** | Deep repo/CMake/replay work that unlocks parallel free remote execution across the remaining codebase. |

Mandatory **Ultra-Lean Seeding Rules** (calibrated by [TOKENOMICS_AUDIT.md](TOKENOMICS_AUDIT.md)):
1. **Zero new `stubs_pktXXX/` trees:** Compile C++ oracle drivers via `python3 scripts/build_oracle.py` directly against `vendor/GeneralsX/build/macos-vulkan` (`libz_gameengine.a`).
2. **Zero local Rust skeletons:** Run `python3 scripts/prep_packet.py <PKT>` (`0` local LLM tokens). Do **not** write compiling `todo!()` skeletons locally; let free remote workers create `crates/.../<module>.rs` and wire `mod.rs`.
3. **Input-carrying + Seed-0 self-diagnosing oracle JSON + thin `~40–60` LOC test guard:** Store explicit inputs + expected outputs + Seed-0 per-section/per-field checkpoints in `docs/golden_vectors/<PKT>.json` so remote agents self-diagnose field mismatches (avoiding opaque checksum stalls seen on PR `#84`). Commit only a thin structural integrity test + `#[ignore]` 1,000-seed contract stub.
4. **Squash & Merge remote PRs:** Keep `main` clean with conventional commit messages linking closed packet issues.

**Empirical Calibration (`#70`, `#71`, `#74`):** Legacy manual seeding burned `1,794` (`PKT-116`, `~3.9:1`), `3,252` (`PKT-434`, `~3.1:1`), and `5,425` (`PKT-442`, `+1,923/-103` PR `#84`, `~2.8:1`) local LOC because of per-packet stub trees (`479` files / `11,312` LOC across `21` `stubs*` dirs), duplicated C++/Rust fuzzers, and upfront skeletons. Adopting `scripts/build_oracle.py` + zero-skeleton seeding cuts local cost per ticket by **~20×** while keeping implementation on free `$0` remote quota. See [TOKENOMICS_AUDIT.md](TOKENOMICS_AUDIT.md).

---

## 2. Compartmentalization with Strict V&V Checkpoints

To allow multiple worker agents to translate Class C1/C2/C3 modules in parallel without stepping on each other or introducing hidden replay desyncs, we enforce **strict physical compartmentalization** and a **4-Gate V&V Funnel**.

### 2.1 File & Crate Compartmentalization Rules
1. **One File Per Module (`crates/zh_logic/src/modules/<category>/<module_name>.rs`):**
   - Each of the 222 `BehaviorModule` classes maps to its own isolated `.rs` file (e.g., `crates/zh_logic/src/modules/behavior/auto_heal.rs`).
   - Worker agents are granted write permission **only** to their assigned `<module_name>.rs` file and its paired unit test file. They cannot edit `ObjectArena`, `ModuleArena`, `SimContext`, or `CMakeLists.txt`.
2. **Pre-Locked Trait/Enum Signatures:**
   - The Tier 3 Architect pre-defines the `ModuleTrait` signature and the `SimWorld` query/mutation API (`world.damage_object(...)`, `world.logic_random_i32(lo, hi)`, `world.find_object(id)`).
   - Worker agents code strictly against this frozen API contract, meaning 10 worker agents can translate 10 modules concurrently with **zero merge conflicts**.
3. **Chunking Mega-Files (`ScriptActions.cpp`, `ScriptConditions.cpp`, `Object.cpp`, `GameLogic.cpp`):**
   - Files over 1,500 lines are never handed to a worker agent whole:
     - `ScriptConditions.cpp` (~170 KB, ~85 condition evaluators) is split by condition enum category into 10 files (`conditions/player.rs`, `conditions/team.rs`, `conditions/unit.rs`, `conditions/timer.rs`, etc.) of ~17 KB each.
     - `ScriptActions.cpp` (~270 KB, ~160 action executors) is split into 15 files (`actions/spawn.rs`, `actions/camera.rs`, `actions/ai.rs`, `actions/team.rs`, etc.) of ~18 KB each.
     - `GameLogic.cpp` (~180 KB) is split into `gamelogic/sleepy_heap.rs`, `gamelogic/commands.rs` (55 `on*` handlers), `gamelogic/crc.rs`, `gamelogic/destroy.rs`, and `gamelogic/update.rs`.

---

### 2.2 The 4-Gate Automated V&V Funnel

No translated module is enabled in the main build until it passes all four automated gates in sequence:

```
┌────────────────────────────────────────────────────────────────────────────┐
│ GATE V1: Static Determinism Linter & Typecheck (< 1s, Local / CI, 0 tokens)│
│  • scripts/lint_determinism.py blocks:                                     │
│    - HashMap / HashSet / BinaryHeap                                        │
│    - std f32::sin/cos/tan/asin/acos/atan2/sqrt/floor/ceil/powf/exp/ln      │
│    - f32::mul_add (unintended FMA contraction)                             │
│    - raw pointer < / > comparisons or pointer-keyed BTreeMap               │
│    - usize / isize in Xfer serialization calls                             │
│  • cargo check --workspace --all-targets                                   │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│ GATE V2: In-Process C++ vs Rust Differential Micro-Harness (~2s, Zero .big)│
│  • Links libzh_ffi.a + z_gameengine into a headless GTest/Cargo test runner│
│  • Instantiates C++ Module and Rust Module side-by-side in memory          │
│  • Runs 1,000 seeded fuzz & boundary steps (lo>=hi, negative floats,       │
│    disabled masks, level promotions, damage deaths, xfer save/load)        │
│  • Asserts 100% bit-exact match on:                                        │
│    1. Return values (UpdateSleepTime, damage adjusted, bools)              │
│    2. XferSave binary snapshot bytes (byte-for-byte state equivalence)     │
│    3. XferCRC state hash                                                   │
│    4. GetGameLogicRandomSeedCRC() (verifies exact PRNG call count & args)  │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│ GATE V3: Lockstep Shadow-Verify Smoke Replay (~15s, Single Fast Replay)    │
│  • Runs GeneralsXZH -headless -replay macos_2p_vanilla_map.rep with        │
│    #[cfg(feature = "shadow-verify")] active for the newly ported batch     │
│  • Both C++ and Rust implementations execute on every live frame call      │
│  • Panics with exact frame, ObjectID, call stack, and field diff on the    │
│    very first divergence (without waiting for the 100-frame CRC interval)  │
└─────────────────────────────────────┬──────────────────────────────────────┘
                                      ▼
┌────────────────────────────────────────────────────────────────────────────┐
│ GATE V4: Full Corpus Replay Matrix (~45s in GitHub Actions 6-Way Matrix)   │
│  • Runs all 6 GeneralsXReplays (3 Linux x86_64 + 3 macOS arm64) + custom   │
│    skirmish replays with shadow-verify OFF (Rust running solo in the seam) │
│  • Must exit 0 with zero CRC mismatches across all frames                  │
└────────────────────────────────────────────────────────────────────────────┘
```

**Why Gate V2 Is the Secret Weapon for Compartmentalized V&V:**
Gate V2 tests a single module (e.g., `ExperienceTracker`, `Energy`, `AutoHealBehavior`, `Locomotor`) against the original C++ implementation in-memory across 1,000 randomized edge-case inputs in **2 seconds**, **without needing retail `.big` files or running a full 20,000-frame replay**. If a cheap Tier 1 agent misses an `Int * Real` float truncation or an early return before a `GameLogicRandomValue` call, Gate V2 catches it in 2 seconds and pinpoints the exact input seed that diverged!

---

## 3. Leveraging Free GitHub Actions CI (Public Repo Strategy) to Offload Local Workload

### 3.1 Why Public GitHub Repo CI Changes the Economics
- **Local Mac Constraints:** Your Apple Silicon Mac currently has **~13 GiB free disk space** and is your active daily workstation. Building 600k lines of C++ (`GeneralsX` + `vcpkg` + `MoltenVK` + `FFmpeg` + debug symbols) and running multi-process replay suites locally consumes 6–10 GiB of disk and saturates local CPU cores.
- **GitHub Hosted Runners for Public Repositories ($0 Unlimited Free Hours):**
  - Because `chayuto/generals-rust` is a **public repository**, standard GitHub-hosted runners (`ubuntu-latest`, `ubuntu-24.04-arm`, `windows-latest`, `macos-15`) are **100% free with unlimited minutes** (GitHub's 2x Windows and 10x macOS minute multipliers apply *only* to private repositories' monthly quota).
  - **Strict Cost Rule:** Never use GitHub Larger Runners (`ubuntu-latest-8-cores`, etc.) or paid third-party runners. Stick exclusively to the standard free labels (`ubuntu-latest`, `macos-15`, `windows-latest`).
- **Concurrency & Runtime Optimization Rules (Avoiding Queue Bottlenecks):**
  - Even though standard public runners cost $0, GitHub enforces a **20-job total concurrency cap** and a **5-job macOS concurrency cap** across your entire account, and Windows/macOS VMs take 25–45 seconds longer to boot than Linux.
  - To keep wall-clock runtimes fast (~10 seconds per push) and never saturate the 5-job macOS pool when multiple agents push branches simultaneously, all workflows enforce four rules:
    1. **Two-Tier OS Matrix Gating (`ubuntu-latest` First, Native `macos-15`/`windows-latest` Only on PR/`main`):**
       - Every intermediate push to worker branches (`packet/**`, `wp/**`) runs **only on `ubuntu-latest`** (`fast-linux-gate`, ~10s), using Linux cross-target checking (`cargo check` for Linux, Windows, and macOS triples) to catch OS-specific type errors without booting Windows or macOS VMs.
       - The native `macos-15` + `windows-latest` matrix (`cross-os-parity`) is gated behind `needs: fast-linux-gate` and `if: github.ref == 'refs/heads/main' || github.event_name == 'pull_request' || github.event_name == 'workflow_dispatch'`.
    2. **Aggressive In-Flight Cancelation (`cancel-in-progress: true`):** Every workflow declares a `concurrency` group keyed by branch/PR so pushing a new commit immediately cancels any still-running older run on that branch.
    3. **Zero Toolchain Re-Download Overhead:** Use the runner image's pre-installed `python3`, `cargo`, `cmake`, and `ninja` (plus `actions/cache` for Cargo/ccache/vcpkg) rather than downloading new Python/toolchain distributions on every job.
    4. **Tight Job Timeouts:** Every job sets `timeout-minutes: 3` (fast gate), `5` (OS parity), or `15` (hybrid C++/Rust build & replays) instead of GitHub's default 360-minute timeout.
- **Public Readiness of `chayuto/generals-rust`:**
  - Because `generals-rust` has zero EA trademarks in its repo/crate/binary names, includes the full `GPL-3.0-or-later` + EA Section 7 `LICENSE.md` with Notice of Modification, and enforces `.gitignore` + `.githooks/pre-commit` blocking all proprietary assets and attribution trailers, **the repository is safe to make public at any time** to unlock unlimited free CI.

### 3.2 Three-Stage CI Offload Pipeline (`.github/workflows/`)

```mermaid
flowchart LR
    subgraph LocalMac["Local Mac (~400 MB Rust Build Only)"]
        A["Tier 0 Slicer +\nTier 1/2/3 Agents"] --> B["Gate V1 Lint +\ncargo check / test"]
    end

    B -->|"git push branch"| GH["GitHub Actions (Public Repo = Free Unlimited)"]

    subgraph GHA["GitHub Actions Parallel Pipeline"]
        J1["Job 1: Rust Lint & Unit Tests\n(ubuntu-latest, ~30s)"]
        J2["Job 2: C++/Rust Hybrid Build +\nGate V2 Differential Test Suite\n(macos-15 & ubuntu-latest, cached ccache/vcpkg, ~2m)"]
        J3["Job 3: 6-Way Parallel Headless Replay Matrix (Gate V3/V4)\n(macos-15 + ubuntu-latest x 6 replays, ~45s)"]
        J1 --> J2 --> J3
    end
```

1. **Workflow 1: `ci-fast-verify.yml` (Zero Retail Assets Required — Runs on Every Push/PR):**
   - **Step A (`rust-check`, ~30s):** Runs `scripts/lint_determinism.py` (`Gate V1`), `cargo fmt --check`, `cargo clippy`, and pure-Rust unit tests (`cargo test --workspace`).
   - **Step B (`hybrid-diff-tests`, ~2 min with `ccache` + `vcpkg-bincache`):**
     - Checks out `fbraz3/GeneralsX` at pinned commit `84a7a20769aa5749f388b1f49bd542c79b3c33e8`.
     - Restores `vcpkg-bincache` and `ccache` from `actions/cache` (so C++ files are almost 100% cache hits after the first run!).
     - Links `libzh_ffi.a` into `z_gameengine` and runs **Gate V2 (In-Process C++-vs-Rust Differential Tests)** + `z_googletest`.
     - Uploads the compiled `GeneralsXZH` `arm64` macOS bundle as a downloadable workflow artifact!
2. **Workflow 2: `ci-replay-matrix.yml` (6-Way Parallel Headless Replay Verification — `Gate V3/V4`):**
   - Mirrors `fbraz3/GeneralsX/.github/workflows/replay-tests.yml`:
     - Clones public `generalsx-project/GeneralsXReplays` (6 cross-platform Zero Hour replays + custom maps).
     - Restores the read-only headless `.big` test bundle from `actions/cache` (populated on cache miss from an encrypted archive decrypted via repository secret `secrets.ASSETS_KEY`, which is never exposed to public viewers or fork PRs).
     - Spawns a **6-job parallel matrix** (one job per replay: `macos_2p_vanilla_map.rep`, `macos_2p_custom_map.rep`, `macos_6p_custom_map.rep`, `linux_2p_vanilla_map.rep`, `linux_2p_custom_map.rep`, `linux_6p_custom_map.rep`).
     - Each job runs `./GeneralsXZH -headless -replay <rep>` with a 180-second timeout, and on any CRC mismatch automatically re-runs with `-DebugCRCFromFrame (F-2) -DebugCRCUntilFrame (F+1) -LogObjectCRCs -SaveDebugCRCPerFrame` and uploads the frame CRC diff artifact for instant Tier 3 diagnosis!

### 3.3 How This Lightens Your Local Mac Workload
With this CI pipeline active:
1. **No 8 GiB C++ Build Tree Needed Locally:** If you want to save local disk space, your Mac only needs:
   - `~/Games/generalszh` (~3.2 GiB, downloaded once via `steamcmd` App `2732960` for local spot-checks or recording replays),
   - A pre-built `GeneralsXZH` binary + `libz_gameengine.a` static archive downloaded from our CI artifact (~150 MB) or a single stripped Release build,
   - The local Rust `target/` directory (~350 MB).
   - **Total local disk footprint stays under ~3.8 GiB** (leaving >9 GiB free on your Mac).
2. **Zero Local CPU Contention During Batch Translation:** Agents run `Gate V1` (Python determinism linter + `cargo check`) locally in < 2 seconds, push a batch branch of 5–10 translated modules, and let GitHub's cloud runners do the C++ linking, `Gate V2` differential fuzzing, and 6-way parallel replay simulations!
