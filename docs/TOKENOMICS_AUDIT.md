# Tokenomics Audit: Manual `env:ephemeral-web` Seeding Workflow

This document records the empirical tokenomics and code-volume audit of the first three manually seeded `env:ephemeral-web` translation tickets (`#71` `PKT-116`, `#70` `PKT-434`, and `#74` `PKT-442`), identifies the root causes of token overhead, and establishes updated engineering rules for future work packages.

---

## 1. Summary & Verdict

Manual per-packet seeding of single standalone modules for `env:ephemeral-web` workers has proven **token-negative in total tokens** and heavily unfavorable in local `env:local-full` effort: across the three seeded tickets, local hand-authored scaffolding (`gen_pkt*.cpp` + `stubs_pkt*/` + `todo!()` Rust skeleton + `golden_pkt*.rs` test harness) required **~3.1× to ~5.4+× more hand-written LOC** than the remote worker ultimately wrote in implementation code.

To evaluate the workflow accurately, local effort must be split into two distinct buckets:

1. **Mandatory V&V Cost (Always Required):** Under `AGENTS.md §1`, every translated module requires an automated C++ Golden Vector oracle and a Rust verification suite regardless of whether an `env:local-full` or `env:ephemeral-web` agent writes the Rust implementation.
2. **Pure Delegation Tax (Avoidable Overhead):**
   - Per-packet C++ stub header trees (`stubs_pktXXX/`) manually stubbing transitive engine headers.
   - Duplicated synthetic fuzzers and state hashers implemented once in C++ and again in Rust.
   - Multi-hundred-line compiling `todo!()` skeletons and exhaustive `Port Contract` prose written upfront locally.
   - Double CI runs, issue bookkeeping, and local PR rebase/bugfix/parity-audit cycles (e.g., PR `#82`).

Under the `AGENTS.md §4.4` break-even rule, hand-seeding single medium modules fails break-even on total tokens. Even when optimizing solely for offloading wall-clock work onto separate `env:ephemeral-web` quota, the current seeding pattern spends several times more `env:local-full` tokens per ticket than necessary.

---

## 2. Empirical Measurements (`#71`, `#70`, `#74`)

All counts below are measured directly from git commits `d57b56a`, `8cac324`, `5e9a6ef`, `c1369d2`, and `tests/cpp_harness/stubs*`:

| Ticket / Packet | Local Hand-Authored Seed Breakdown | Packet Markdown & Oracle JSON | Remote Worker Delta (`env:ephemeral-web`) | Local Seed : Remote Impl Ratio | Extra Local Coordination Cost |
| :--- | :--- | :--- | :--- | :---: | :--- |
| **`#71` / `PKT-116`** (`AcademyStats`, `d57b56a` -> PR `#83` `8cac324`) | **`1,794` LOC** (`gen_pkt116_academy_stats.cpp` `494` + `stubs_pkt116/` `652` across `28` files + `academy_stats.rs` skeleton `298` + `golden_pkt116_academy_stats.rs` `350`) | `1,705` lines (`PKT-116_AcademyStats.md`) | **`+455 / -18` LOC** in `crates/zh_logic/src/economy/academy_stats.rs` (`+464 / -29` total across 3 files) | **`~3.9 : 1`** | PR review & merge |
| **`#70` / `PKT-434`** (`Scripts`, `d57b56a` -> PR `#82` `5e9a6ef`) | **`3,252` LOC** (`gen_pkt434_scripts.cpp` `880` + `stubs_pkt434/` `851` across `28` files + `scripts.rs` skeleton `800` + `golden_pkt434_scripts.rs` `721`) | `3,945` lines (`PKT-434_Scripts.md`) | **`+1,042 / -40` LOC** in `crates/zh_logic/src/scripts/scripts.rs` (`+1,065 / -47` total across 3 files) | **`~3.1 : 1`** | Local rebase, seed-8 bugfix, and parity audit pass on PR `#82` |
| **`#74` / `PKT-442`** (`GlobalData`, `c1369d2`, assigned) | **`5,425` LOC** (`gen_pkt442_global_data.cpp` `1,669` + `stubs_pkt442/` `1,199` across `50` files + `global_data.rs` skeleton `1,062` + `golden_pkt442_global_data.rs` `1,495`) | `2,744` lines (`PKT-442_GlobalData.md`) + `1,018` lines (`PKT-442.json`) | *~800–1,000 LOC est.* of method bodies in `crates/zh_logic/src/gamelogic/global_data.rs` | **`~5.4 : 1` to `6.8 : 1`** *(est.)* | Pending PR review & merge |

### Repository-Wide Stub Sprawl

Across `tests/cpp_harness/`, there are currently **`21` separate `stubs*` directories** totaling **`479` files and `11,312` LOC** of duplicated C++ stub headers — despite `vendor/GeneralsX/build/macos-vulkan/GeneralsMD/Code/GameEngine/libz_gameengine.a` and `GeneralsXZH` already being built locally.

---

## 3. Four Root Causes of Token Waste

1. **Per-Packet Stub-Header Re-Invention (`stubs_pktXXX/`):**
   Hand-writing 28–50 minimal stub headers per packet kept harnesses isolated for tiny Phase 1 leaf utilities (`GameMath`, `CRC`, `Xfer`), but scales poorly once target files include core engine headers (`GlobalData`, `Locomotor`, `Weapon`, `ScriptActions`).

2. **Duplicating Fuzzers and State Hashers Across C++ and Rust:**
   On `PKT-442`, `tests/cpp_harness/gen_pkt442_global_data.cpp` (`1,669` LOC) and `crates/zh_logic/tests/golden_pkt442_global_data.rs` (`1,495` LOC) re-implemented the same 1,000-seed synthetic INI generator and 250-field state hasher side by side — **`3,164` LOC** of mirrored test scaffolding before counting any production Rust logic.

3. **Front-Loading ~80% of Design Effort into `todo!()` Skeletons and Prose Contracts:**
   In `PKT-442`, seeding already required auditing all C++ edge cases (`m_infantryLightScale[0]` uninitialized slot, `DrawSkyBox`/`DebugAI` `INI::parseBool` byte-0 overwrites, `newOverride()` shallow `m_weaponBonusSet` pointer, `generateExeCRC()` 1,001-chunk loop bound), writing a compiling `1,062`-LOC skeleton, and authoring a detailed `Port Contract`. The remote worker then had to consume `2,744` lines of packet markdown + `1,062` lines of skeleton + `1,495` lines of test harness just to fill in the remaining method bodies. Finishing `global_data.rs` directly in the local pass would have added only ~15–25% incremental tokens while eliminating the skeleton, contract prose, and PR turnaround entirely.

4. **Two-Pass Coordination & PR Fix Overhead:**
   Splitting seeding from implementation introduces an intermediate seed commit + 3-OS CI run, issue state management, and follow-up local engineering whenever the remote PR needs rebasing or edge-case parity fixes (as occurred on PR `#82`).

---

## 4. Mandatory Remedies (4 Architectural Changes)

1. **Rule 1 — Finish Single Small/Medium Modules Locally While Context Is Hot:**
   Once an `env:local-full` agent has read the C++ target and built its Golden Vector oracle, completing the Rust module in the same pass is strictly cheaper in total tokens than writing a compiling `todo!()` skeleton, prose `Port Contract`, and reviewing a separate PR.

2. **Rule 2 — Shared CMake C++ Oracle Target Linking `libz_gameengine.a`:**
   Stop creating new per-packet `stubs_pktXXX/` trees. Add a shared CMake oracle harness target linked against the already-built `vendor/GeneralsX/build/macos-vulkan` headers and static archives (`libz_gameengine.a`), shrinking C++ oracle drivers to ~100–200 LOC while testing real vendor headers directly (preserving the `__APPLE__` / `Byte` signedness guards documented in [tests/cpp_harness/README.md](../tests/cpp_harness/README.md)).

3. **Rule 3 — Input-Carrying Oracle Fixtures & Minimal Public API Contracts:**
   - Have the C++ oracle serialize compact test inputs alongside expected outputs in `docs/golden_vectors/*.json` rather than mirroring complex synthetic generators in Rust. This reduces `crates/*/tests/golden_*.rs` runners from ~700–1,500 LOC to ~80–150 LOC.
   - When delegating a true heavyweight to `env:ephemeral-web`, define only a compact (~50–100 LOC) public trait/type API contract rather than a 1,000-LOC field-for-field `todo!()` skeleton; let the remote worker author internal structs, enums, and method bodies directly from the stripped C++ source packet.

4. **Rule 4 — Reserve `env:ephemeral-web` Strictly for `O(1)`-Setup / `O(N)`-Offload Work:**
   - **Programmatic batch generation (`WP-5.0` / `WP-6.4`):** Build automated oracle, packet, and adapter generators once locally to fan out uniform batches (`221` `BehaviorModule` packets, `29` `Script` action/condition packets) at near-zero marginal local token cost per ticket.
   - **Ultra-dense standalone targets (`#81` `ScriptConditions`/`ScriptActions`, `#79` `Weapon`/`WeaponSet`):** Only delegate these after Rules 2 and 3 are in place so local seeding drops from ~3,000–5,400 LOC to ~300–500 LOC.

---

## 5. Routing Guidance for Remaining `status:blocked-on-vv` Tickets (`#72`–`#81`)

- **Do NOT hand-seed under the old `stubs_pktXXX/` + full-skeleton pattern:**
  - `#72` (`TurretAI`), `#73` (`Locomotor`), `#75` (`ActionManager`), `#76` (`BuildAssistant`), `#77` (`Radar`), `#78` (`ObjectCreationList`), `#80` (`AIDock` / `AIGuard*`) — either port directly locally alongside a shared CMake oracle or defer until the Phase 4/5 Rust entity/world seams exist.
- **Hold for post-Rule-2/3 delegation:**
  - `#79` (`Weapon` / `WeaponSet`) and `#81` (`ScriptConditions` / `ScriptActions`) remain viable `env:ephemeral-web` offload candidates once shared CMake oracle linkage and input-carrying fixtures replace per-packet stub trees and mirrored fuzzers.
