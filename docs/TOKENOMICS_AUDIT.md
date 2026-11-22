# Tokenomics Audit: Three-Tier Resource Cost Hierarchy (`Local Compute = $0`, `Remote Tokens = $0 / Bounded Runner Quota`, `Local Tokens = EXPENSIVE`)

> **Dated Operational Decision (`2026-11-22` — Active):** All **`env:ephemeral-web` remote agent assignments are PAUSED** until further notice by owner instruction so GitHub Actions runner minutes are conserved. Execute translations, C++ oracle verification, and V&V locally (`env:local-full`), maximizing `$0` local compute (`python3`, `build_oracle.py`, `prep_packet.py`, `port_map.py`, `./scripts/bootstrap_repo.sh`) and batching pushes into clean verified stages.

This document records the empirical tokenomics and code-volume audit of both our legacy `env:ephemeral-web` delegation workflow (`#71` `PKT-116`, `#70` `PKT-434`, and `#74` `PKT-442`) and our post-refactor ultra-lean workflow (`#77` `PKT-406`, `#73` `PKT-143`, and `#79` `PKT-399`), analyzes seeding leverage and GitHub Actions runner-minute usage, and defines the **Three-Tier Resource Cost Hierarchy** governing the remaining engine packets.

---

## 1. The Three-Tier Resource Cost Hierarchy

Engineering in `generals-rust` operates across three distinct resource tiers that must never be conflated:

1. **Tier 1 (Cheapest): Local Deterministic Compute (`$0`, `0` LLM Tokens, `0` CI Minutes):**
   - Local CPU scripts, `python3`, `clang++`, `cargo`, `scripts/build_oracle.py`, `scripts/prep_packet.py`, `scripts/port_map.py`, `./scripts/bootstrap_repo.sh`, `-O0`/`-O2`/UBSan determinism cross-checks, and one-shot Python log/JSON slicers consume **zero tokens** and **zero CI runner minutes**.
   - Offload every mechanical, repetitive, formatting, filtering, and verification task to local scripts before pulling compact diagnostics into LLM context.
2. **Tier 2 (Free Tokens, Finite Runner Quota): Remote `env:ephemeral-web` Workers (`$0` LLM Tokens, `~5–20` Actions min/session):**
   - Remote cloud sandbox workers cost `$0` local LLM tokens, **but** their managed cloud worker runners (`ubuntu-cca-*`) bill `~5–20` minutes per session against the personal `3,000` min/month Actions quota — even on a public repository.
   - Maximize verified Rust LOC delivered per cloud session by targeting large domains (`2,000–7,000+` C++ LOC) or macro-batched leaf waves (`10–30` modules per issue), and patch minor PR edge cases locally with `$0` local compute instead of triggering remote retry loops.
3. **Tier 3 (Most Expensive): Local `env:local-full` LLM Tokens:**
   - Strictly reserved for high-leverage **force multipliers**: writing `0`-token local automation scripts/code generators (`oracle_common.h`, `WP-5.0` batch generator), ultra-lean remote seeds (`< 300–600` LOC), `C4` architectural spines (`SimContext`, `ObjectArena`, `ModuleArena`, `SleepyUpdateHeap`, `Pathfinder`, `GameLogic::update()`), and replay desync triage.

Therefore, optimizing for **minimum total tokens across both environments is the wrong objective**, and treating remote sessions as infinite without tracking runner minutes is also wrong. The real objective is maximizing **verified Rust lines landed per local `env:local-full` token *and* per cloud runner minute**, using `$0` local compute heavily for generation, filtering, and verification.

### Why PR `#84` Validated Remote Offload — and Exposed Our Seeding Tax

On Issue `#74` (`PKT-442` `GlobalData`, merged via PR `#84` `70fe662`), the remote worker produced **`+1,923 / -103` LOC** at **`$0` local implementation cost**, nearly reaching full 1,000-seed parity on its first pass. However, two seeding flaws reduced leverage:

1. **Massive Local Seeding Tax (`5,425` local LOC):** Hand-writing `50` stub headers (`1,199` LOC), a `1,669`-LOC C++ generator, a `1,062`-LOC compiling Rust skeleton, and a `1,495`-LOC mirrored Rust test harness burned far too many local tokens before the remote worker even started.
2. **Opaque Aggregate Checksum Without Seed-0 Diagnostics:** `defaults_crc` collapsed 300+ constructor fields into a single `u32` hash, hiding a single float default mismatch (`shroud_color.red` `0.0` vs `1.0`) so the remote worker could not self-diagnose without local intervention.

---

## 2. Empirical Measurements (`#71`, `#70`, `#74`)

All counts below are measured from git commits `d57b56a`, `8cac324`, `5e9a6ef`, `c1369d2`, `70fe662`, and `tests/cpp_harness/stubs*`:

| Ticket / Packet | Legacy Local Seed Breakdown | Packet Markdown & Oracle JSON | Remote Worker Delta (`env:ephemeral-web`, `$0`) | Local Seed : Remote Impl Ratio | Extra Local Intervention |
| :--- | :--- | :--- | :--- | :---: | :--- |
| **`#71` / `PKT-116`** (`AcademyStats`, `d57b56a` -> PR `#83` `8cac324`) | **`1,794` LOC** (`gen_pkt116_academy_stats.cpp` `494` + `stubs_pkt116/` `652` across `28` files + `academy_stats.rs` skeleton `298` + `golden_pkt116_academy_stats.rs` `350`) | `1,705` lines (`PKT-116_AcademyStats.md`) | **`+455 / -18` LOC** in `academy_stats.rs` (`+464 / -29` total) | **`~3.9 : 1`** | PR review & merge |
| **`#70` / `PKT-434`** (`Scripts`, `d57b56a` -> PR `#82` `5e9a6ef`) | **`3,252` LOC** (`gen_pkt434_scripts.cpp` `880` + `stubs_pkt434/` `851` across `28` files + `scripts.rs` skeleton `800` + `golden_pkt434_scripts.rs` `721`) | `3,945` lines (`PKT-434_Scripts.md`) | **`+1,042 / -40` LOC** in `scripts.rs` (`+1,065 / -47` total) | **`~3.1 : 1`** | Local rebase, seed-8 fix & audit |
| **`#74` / `PKT-442`** (`GlobalData`, `c1369d2` -> PR `#84` `70fe662`) | **`5,425` LOC** (`gen_pkt442_global_data.cpp` `1,669` + `stubs_pkt442/` `1,199` across `50` files + `global_data.rs` skeleton `1,062` + `golden_pkt442_global_data.rs` `1,495`) | `2,744` lines (`PKT-442_GlobalData.md`) + `1,018` lines (`PKT-442.json`) | **`+1,923 / -103` LOC** total (`+1,731` net diff in `global_data.rs`) | **`~2.8 : 1`** | Fixed `shroud_color.red`, 2 INI tokens, `Firewall*`, 11 getters & squash-merged |

### Post-Refactor Ultra-Lean Calibration (`#77`, `#73`, `#79`)

Following adoption of zero-stub `build_oracle.py`, shared `oracle_common.h`, zero local Rust skeletons, and Seed-0 field diagnostics:

| Ticket / Packet | Ultra-Lean Local Seed (`0` Stubs, `0` Rust Skeleton) | Remote Worker Delta (`env:ephemeral-web`, `$0` Tokens) | Local Seed : Remote Impl Ratio | Cloud Actions & Local Follow-Up |
| :--- | :--- | :--- | :---: | :--- |
| **`#77` / `PKT-406`** (`Radar`, `2,030` C++ LOC, `5e246f9` -> PR `#85` `fc59e73`) | **`398` LOC** (`gen_pkt406_radar.cpp` `356` + `golden_pkt406_radar.rs` `42`) | **`764` LOC** `radar.rs` + **`+277` LOC** assertions (`+1,041` Rust LOC) | **`~0.38 : 1`** | Fixed 4-line stack-zeroing & `clear_all_events` alpha quirk locally (`0` extra CI/cloud min) and squash-merged. |
| **`#73` / `PKT-143` + `PKT-144`** (`Locomotor` + `LocomotorSet`, `4,149` C++ LOC, `f7ffff7` -> PR `#86` `3dcb39c`) | **`506` per-packet LOC** (`gen_pkt143_locomotor.cpp` `456` + `golden_pkt143_locomotor.rs` `50`; plus `389`-LOC shared `oracle_common.h`) | **`+892 / -6` LOC** (`locomotor.rs` `551` + `locomotor_set.rs` `122` + `golden_pkt143_locomotor.rs` `+207`) | **`~0.57 : 1`** | **Zero local intervention:** merged cleanly on first remote pass with `1,000/1,000` vectors green. |
| **`#79` / `PKT-399` + `PKT-400`** (`Weapon` + `WeaponSet`, `7,195` C++ LOC, `e8b8ab4`) | **`600` LOC** (`gen_pkt399_weapon.cpp` `548` + `golden_pkt399_weapon.rs` `52`) | Pending (`7,195` C++ LOC queued as 1 high-ROI session) | — | Seeded via `oracle_common.h` + `prep_packet.py` (`0` tokens) + `0`-token `-O0`/`-O2`/UBSan build verification. |

### Repository-Wide Legacy Stub Sprawl

Across `tests/cpp_harness/`, there are **`21` legacy `stubs*` directories** totaling **`479` files and `11,312` LOC** of hand-written C++ stub headers — despite `vendor/GeneralsX/build/macos-vulkan/GeneralsMD/Code/GameEngine/libz_gameengine.a` and `GeneralsXZH` already existing locally.

---

## 3. Four Root Causes of Local Seeding Waste (Solved Below)

1. **Per-Packet Stub-Header Re-Invention (`stubs_pktXXX/`):** Hand-stubbing 28–50 transitive headers per ticket burns huge local context and introduces subtle stub drift.
2. **Duplicating Fuzzers and State Hashers in Both C++ and Rust:** Mirroring complex input generators across C++ (`1,669` LOC) and Rust (`1,495` LOC) on `PKT-442` doubled local test-authoring cost.
3. **Writing Compiling `todo!()` Skeletons & Line-by-Line Prose Locally:** Authoring `300–1,062` lines of compiling `todo!()` Rust skeletons locally steals work the free remote agent can generate from `packets/<phase>/<PKT>.md` directly.
4. **Opaque Single-Integer CRCs Without Seed-0 Diagnostics:** Returning one aggregate hash per seed leaves remote agents unable to isolate a single mismatched field when CI fails.

---

## 4. The Ultra-Lean Remote Offload Playbook (`< 300` Local LOC per Ticket, ~20× Reduction)

To maximize **free remote throughput** while keeping **expensive local spend minimal**:

1. **Never Implement Medium or Large Modules Locally:**
   If a module can be cleanly verified against a Golden Vector oracle, offload its Rust implementation to `env:ephemeral-web` — do not write the Rust body locally.
2. **Rule A — Zero-Stub C++ Compilation (`scripts/build_oracle.py`):**
   Compile every new `gen_pkt*.cpp` directly against `vendor/GeneralsX/build/macos-vulkan` (`libz_gameengine.a` + friends) via `python3 scripts/build_oracle.py`. Create **zero** new `stubs_pktXXX/` directories. C++ oracle drivers shrink to **`~150–250` LOC**.
3. **Rule B — Zero Local Rust Skeletons (`0` Local Production Rust LOC):**
   Generate `packets/<phase>/<PKT>_<Module>.md` using deterministic `python3 scripts/prep_packet.py <PKT>` (`0` local LLM tokens). Add only a brief top-level scope/API note if headless scoping applies. **Do not create `crates/.../<module>.rs` skeleton files locally**; let the `env:ephemeral-web` worker create the module and wire its `mod.rs` export from the packet.
4. **Rule C — Input-Carrying + Self-Diagnosing Golden Vector JSON + Thin Guard Test (`~40–60` LOC):**
   - Store **explicit test inputs** alongside expected outputs in `docs/golden_vectors/<PKT>.json` so there is zero duplicated synthetic input generator logic to maintain.
   - Always include **Seed-0 per-section / per-field diagnostic checkpoints** inside the JSON header/vector 0 so remote workers can pinpoint any constructor, field, or state discrepancy without local assistance.
   - Commit only a **`~40–60`-LOC** `crates/*/tests/golden_<pkt>.rs` containing an active oracle-integrity test + an `#[ignore]` 1,000-seed contract stub; let the remote worker implement the 1,000-seed assertion loop and remove `#[ignore]`.
5. **Rule D — Whole-Project Parallel Pipeline (Batching & Force Multipliers):**
   Spend expensive local tokens strictly on high-leverage force multipliers:
   - Feeding lean `< 300`-LOC tickets continuously to free `env:ephemeral-web` workers (`#72`–`#81` cover **40,000+ C++ LOC**).
   - Shared Phase 2/3 CMake oracles (batching clean leaf families so 1 oracle unlocks multiple remote tickets or one remote worker clears multiple files).
   - **Phase 0 (`Gate 0` replay baseline)** and **`WP-5.0` (`SimContext` / `ObjectArena` / `ModuleArena` / `ModuleTrait` + automated batch generator)**, which unlocks automated generation across **`221` Phase 5 `BehaviorModule` packets** (~58% of all unported files) and **`WP-6.4` Script packets** with `O(1)` local token cost.
6. **Rule E — Local-Compute-First & CI Actions Minute Discipline:**
   - **Prefer `$0` local deterministic compute over LLM tokens or cloud minutes:** Write/run local Python extractors, JSON pre-filters, `-O0`/`-O2`/UBSan builds, and `./scripts/bootstrap_repo.sh` / `cargo test` (`0` tokens, `0` CI minutes) before pulling raw logs into LLM context or pushing commits.
   - **Conserve the `3,000` min/month GitHub Actions quota:** Managed cloud worker runners (`env:ephemeral-web`) consume `~5–20` Actions minutes per session even on a public repository. Macro-batch small C1/C2 leaves into large multi-module waves (`10–30` modules per issue) rather than 1-issue-per-tiny-leaf; keep `macos-15` (10×) and `windows-latest` (2×) `workflow_dispatch`-only in `.github/workflows/ci-guardrails.yml` (`6fe772a`); and fix minor 1–2 seed edge cases in near-passing remote PRs locally rather than triggering remote retry runs.

---

## 5. Whole-Project Routing Table (`#72`–`#81` & Remaining Packets)

Under the **Three-Tier Resource Cost Hierarchy**, **every** isolated or semi-isolated ticket runs on `env:ephemeral-web` using Rules A–E (`0`-token local prep scripts + lean `< 300–600`-LOC seed -> high-ROI cloud sessions), while `$0` local compute handles verification and minor PR fixups:

| Scope | Local Action (`env:local-full`, Local Compute First) | Remote Action (`env:ephemeral-web`, `$0` Tokens) |
| :--- | :--- | :--- |
| **Lean-Seedable Large Targets (`#72` `TurretAI`, `#75` `ActionManager`, `#76` `BuildAssistant`, `#78` `ObjectCreationList`, `#79` `Weapon`/`WeaponSet`, `#80` `AIDock`/`AIGuard*`, `#81` `ScriptConditions`/`ScriptActions`)** | `prep_packet.py` (`0` tokens) + `build_oracle.py` (`libz_gameengine.a` + `oracle_common.h`) + `~50`-LOC guard test + Seed-0 diagnostics in JSON. **No Rust skeleton.** Patch minor PR edge cases locally if needed. | Worker writes full Rust module, wires `mod.rs`, implements 1,000-seed assertion loop, passes `V1` + Linux CI (`V2-Golden`). |
| **Batch Leaf Bundles (Remaining Phase 2–3 C1/C2 modules)** | One shared `build_oracle.py` multi-unit oracle + scripted `prep_packet.py` (`0` tokens) batched into `10–30`-module waves so one cloud session clears many files. | Remote workers clear 10–30 files per session on `$0` LLM quota while conserving Actions minutes. |
| **`C4` Architectural Spines, FFI Seams & Automation (`WP-5.0`, `Object`, `GameLogic::update()`, `Pathfinder`, `Gate 0` replays)** | Owned directly by `env:local-full` (`Tier 3` Architect) as the force multiplier that converts Phase 5–6 codegen into `0`-token local script execution. | `221` Phase 5 modules (`WP-5.1`–`WP-5.4`) and `29` Script packets (`WP-6.4`) auto-generated once `WP-5.0` lands. |

---

## 6. Comparative Tokenomic Designs & Local Multi-Agent Orchestration (`2026-11-22`)

With `env:ephemeral-web` cloud workers paused to protect our monthly GitHub Actions quota, empirical execution on `WP-5.5` (`f1c25a4`) showed that falling back to a single monolithic local conversation is also suboptimal. This section compares all five tokenomic architectures evaluated for `generals-rust` and locks in our active local orchestration and read-only chat-agent rules.

### 6.1 Comparison of Tokenomic Approaches

| Tokenomic Design | Local LLM Token Cost | CI / Cloud Quota Cost | Throughput & Parallelism | Best Fit | Current Status |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **1. Legacy Heavy-Seed -> Remote Offload** | Very High (`~2.8–3.9:1` local seed LOC tax from `stubs_pktXXX/`, `todo!()` skeletons, mirrored fuzzers) | `~5–20` Actions min/session (`3,000` min/mo pool) | Parallel remote PRs, high pre-work | Superseded calibration baseline (`#71`, `#70`, `#74`) | **Obsolete** |
| **2. Ultra-Lean Seed -> Remote Cloud Sandbox (`env:ephemeral-web`)** | Low (`~0.38–0.57:1` local LOC via `build_oracle.py` + Seed-0 diagnostics) | `~5–20` Actions min/session against `3,000` min/mo limit | High (parallel cloud sessions per batch) | Isolated `C1`–`C3` targets & `10–30`-leaf macro-batches | **Paused (`2026-11-22`)** |
| **3. Monolithic Single-Thread Local Execution** | Highest ($O(T^2)$ context-replay growth per turn + drains tightest `Tier 3` pool on `C1`/`C2` leaves) | `0` if batched push | Serial `1x`; one long `WP-5.5` session (`10` packets + `1,296` C++ oracle + `1,143` test LOC) | Avoid for parallelizable leaf/module waves | **Anti-Pattern for Waves** |
| **4. Local Coordinator + Parallel Worktree Swarm** | Controlled (`Tier 3` scarce coordinator thin; `Tier 1A/1B/2` higher-quota worker pools; fresh context per worker) | `0` local verification (`1` batched push per verified wave) | `2–4x` concurrent in isolated git worktrees (`share`) | Day-to-day local translation, multi-module waves, oracle + Rust pairing | **Active Default (`2026-11-22`)** |
| **5. Read-Only Public-Repo Deep-Think Chat Agent (No VM)** | `$0` local quota | `0` CI minutes | Asynchronous single-shot deep design/audit | Compact `C4` contracts, determinism audits, desync root-cause triage | **Selective Supplement** |

### 6.2 Local Multi-Agent Orchestration Rules

1. **Quota-Pool Tiering (Protect the Scarce Coordinator Pool):**
   - Keep the scarce `Tier 3` local coordinator strictly focused on thin planning turns, `C4` architecture and seam contracts, branch integration, and CRC desync triage.
   - Route `C1` (`Tier 1A`), `C2` (`Tier 1B`), and `C3` (`Tier 2`) module translation, oracle wiring, and Golden Vector test verification to separate higher-remaining-quota local worker pools.
   - When a worker's quota pool runs low or hits capacity, switch the existing worker in-place to an available higher-quota worker pool (preserving its worktree and context) rather than tearing down worktrees unnecessarily.
2. **Eliminate $O(T^2)$ Context Growth via Isolated Git Worktrees:**
   - Spawn 2–4 concurrent workers with **fresh contexts** and **disjoint file ownership** using shared git worktrees (`share`). Do not fork huge coordinator histories into workers.
   - Workers edit only their scoped module/oracle/test files — never shared `mod.rs` files or `docs/PORT_MAP.md` — and verify locally with targeted runs (`python3 scripts/lint_determinism.py && cargo test -p <crate> --test golden_<pkt>`).
   - Once workers report green, the `Tier 3` Coordinator integrates the verified slices, updates `mod.rs`, regenerates `docs/PORT_MAP.md`, runs `./scripts/bootstrap_repo.sh` once across the whole wave, and issues a single batched `git push` to conserve CI minutes.
3. **Force-Multiplier Sequencing (Manifest Reality):**
   - Out of `396 / 463` unported packets (`302,384` estimated C++ LOC), **`291` packets (`73%`) are Class `C1`/`C2`** (`193` `C1` / `66,309` LOC + `98` `C2` / `58,580` LOC), `66` are `C3` (`77,029` LOC), and `39` are `C4` (`100,466` LOC).
   - Phase 5 `BehaviorModule` subclasses (`WP-5.1`–`WP-5.4`: **`225` packets**, `57%` of all remaining files, `136,639` est. LOC) and Phase 2 leaf clusters (`WP-2.3` + `WP-2.4`: `87` packets, `33,870` est. LOC) dominate the remaining file count.
   - Prioritize **`WP-5.0`** (`SimContext` / `ObjectArena` / `ModuleArena` / `ModuleTrait` + a reusable `BehaviorModule` oracle harness) and `Tier 0` (`$0` CPU) generators in `scripts/` so `WP-5.1`–`WP-5.4` execute as uniform, small-context parallel worker batches rather than bespoke hand-crafted waves.

### 6.3 Read-Only Public-Repo Chat Agent Protocol (Zero-Overhead Constraints)

A read-only chat agent can inspect committed files in public `chayuto/generals-rust` and reason deeply in long single-shot outputs, **but has no VM, no compiler, no `~/Games/generalszh` access, and no `vendor/GeneralsX` access**. To remain net-positive on both tokens and engineering time, enforce strict zero-overhead boundaries:

1. **Three Valid High-ROI Uses:**
   - **`C4` Architecture & Contract Design (`WP-5.0`, `Object`, `GameLogic::update()`):** Have it read already-committed `PLAN.md`, `docs/ARCHITECTURE_SPEC.md`, `docs/RESEARCH.md`, and `crates/` and output a compact trait layout, borrow-splitting plan, and trap checklist before local workers write code.
   - **Direct-to-Disk Single-File Drafts (Only When Source Is Already Accessible Without Extra Commits):** Save a generated file straight to disk and pass **only the 1-line file path** to a local compiler-equipped `Tier 1B/2` worker to run `lint_determinism.py` + `cargo test` and fix residual compile or Seed-0 mismatches.
   - **Deep Determinism Audits & Stubborn Desync Root-Causing:** Supply a short Seed-0 / frame-CRC diff alongside existing repo paths to obtain a concise root-cause hypothesis.
2. **Three Explicit Anti-Patterns (Hidden Double-Spend Traps):**
   - **No Throwaway Commits for a Chat Agent:** Never commit extra repo-index dumps, unpacker glue, or speculative packets solely to feed a read-only chat agent.
   - **No Pasting Large Outputs or Verbose Instructions Into Paid Local Coordinator Sessions:** Pasting hundreds or thousands of generated lines back into the local coordinator burns input and re-processing tokens twice; keep chat outputs compact or direct-to-disk.
   - **No Compile-Fix Ping-Pong Without a VM:** Never shuttle `rustc` or borrow-checker diagnostics manually between a no-VM chat agent and the local repo; let local compiler-equipped workers iterate directly against `cargo check` / `cargo test`.
