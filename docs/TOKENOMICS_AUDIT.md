# Tokenomics Audit: Asymmetric Cost Model (`Remote = $0 FREE`, `Local = EXPENSIVE`)

This document records the empirical tokenomics and code-volume audit of the `env:ephemeral-web` delegation workflow (`#71` `PKT-116`, `#70` `PKT-434`, and `#74` `PKT-442`), analyzes why manual per-packet seeding was expensive locally, and defines the **Asymmetric Cost Law** governing all remaining `411` unported packets.

---

## 1. The Asymmetric Cost Law

Engineering in `generals-rust` operates across two cost tiers that are **not** symmetric:

- **Local `env:local-full` tokens are EXPENSIVE.** Every local turn that writes Rust skeletons, prose contracts, stub headers, or medium/large Rust module implementations directly consumes high-value local quota.
- **Remote `env:ephemeral-web` tokens are `$0` / FREE** (subject only to automated Gate V1/V2 verification before merge).

Therefore, optimizing for **minimum total tokens across both environments is the wrong objective**. The real objective is maximizing **verified Rust lines landed per local `env:local-full` token**, treating local tokens strictly as **force multipliers** (shared oracle tooling, architectural `C4` spines, `WP-5.0` fan-out automation, and ultra-lean `< 300`-LOC remote seeds).

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

---

## 5. Whole-Project Routing Table (`#72`–`#81` & Remaining `411` Packets)

Under the **Asymmetric Cost Law**, **every** isolated or semi-isolated ticket stays on `env:ephemeral-web` using Rules A–C (`< 300` local LOC per seed), NOT ported locally:

| Scope | Local Action (`env:local-full`) | Remote Action (`env:ephemeral-web`, `$0`) |
| :--- | :--- | :--- |
| **Lean-Seedable Targets (`#77` `Radar`, `#72` `TurretAI`, `#73` `Locomotor`, `#75` `ActionManager`, `#76` `BuildAssistant`, `#78` `ObjectCreationList`, `#79` `Weapon`/`WeaponSet`, `#80` `AIDock`/`AIGuard*`, `#81` `ScriptConditions`/`ScriptActions`)** | `prep_packet.py` (`0` tokens) + `~200` LOC `build_oracle.py` C++ driver + `~50` LOC guard test + Seed-0 diagnostics in JSON. **No Rust skeleton.** | Worker writes full Rust module, wires `mod.rs`, implements 1,000-seed assertion loop, passes `V1` + 3-OS CI (`V2-Golden`). |
| **Batch Leaf Bundles (Remaining Phase 2–3 C1/C2 modules)** | One shared `build_oracle.py` multi-unit oracle (`~200` LOC) + `prep_packet.py` per target -> fan out issues in parallel. | Remote workers clear 10–30 files concurrently on `$0` quota. |
| **`C4` Architectural Spines, FFI Seams & Automation (`WP-5.0`, `Object`, `GameLogic::update()`, `Pathfinder`, `Gate 0` replays)** | Owned directly by `env:local-full` (`Tier 3` Architect) as the force multiplier that unlocks the `221` Phase 5 modules and replay matrix. | `221` Phase 5 modules (`WP-5.1`–`WP-5.4`) and `29` Script packets (`WP-6.4`) auto-generated once `WP-5.0` lands. |
