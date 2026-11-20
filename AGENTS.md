# Agent Constitution & Shared Workspace Protocol (`generals-rust`)

Every agent or contributor working in this repository **must** read and follow this document before running commands, claiming tasks, or writing code.

---

## 1. Golden Rule of Verification & Validation (V&V)

> **NEVER BUILD SOMETHING YOU CANNOT VERIFY.**
> Do not write or merge any port/translation code unless its automated verification oracle already exists, is runnable, and can prove the code right or wrong.

1. **Before Phase 1+ port code:** Phase 0 (`WP-0.1`..`WP-0.4`) must be complete and green — meaning `GeneralsXZH` builds at pinned commit `84a7a20769aa5749f388b1f49bd542c79b3c33e8` and passes 100% of the headless replay corpus (`Gate 0`).
2. **Before merging any translated module:** The module must have and pass:
   - **Gate V1 (`scripts/lint_determinism.py` + `cargo check`):** Zero static determinism violations (`HashMap`, `BinaryHeap`, std `f32` transcendentals, `mul_add`, pointer ordering, `usize` in `Xfer`).
   - **Gate V2 (In-Process C++ vs. Rust Differential Micro-Harness):** 1,000-seed side-by-side fuzz & boundary test comparing C++ and Rust return values, `XferSave` snapshot bytes, `XferCRC`, and `GetGameLogicRandomSeedCRC()`.
   - **Gate V3 (Lockstep Shadow-Verify Smoke Replay):** Single fast replay (`macos_2p_vanilla_map.rep`) with `shadow-verify` asserting per-call C++/Rust parity.
   - **Gate V4 (Full Replay Corpus CRC Gate):** All cross-platform replays exit `0` with zero CRC mismatches.
3. **Before merging any script, tool, or harness:** Every script in `scripts/` must have an automated self-test (e.g. `python3 scripts/test_lint_determinism.py`, `scripts/bootstrap_repo.sh --self-test`) executed and passing.

---

## 2. Mandatory Repository Bootstrap (Run First in Every Session)

At the start of every session, run:

```bash
./scripts/bootstrap_repo.sh
```

This verifies that:
- `git config core.hooksPath` is set to `.githooks` and hooks are executable;
- `git config user.email` is set to `5305832+chayuto@users.noreply.github.com`;
- Free disk space on `/` is above the safety threshold;
- Required local tools (`git`, `gh`, `python3`, `cargo`, `cmake`, `ninja`) are available.

---

## 3. Non-Negotiable Governance & Hygiene Rules

1. **No Attribution or Assistant Names Anywhere:**
   - Never include co-author trailers, generator notices, internal session links, or assistant/model brand names in commit messages, code comments, documentation, PRs, GitHub Issues, or GitHub Discussions.
   - When referring to agent tiers in Issues or Discussions, use the neutral project tier names: **`Tier 0 (AST Slicer)`**, **`Tier 1A (Lite Worker)`**, **`Tier 1B (Fast Worker)`**, **`Tier 2 (Engineer)`**, **`Tier 3 (Architect)`**.
2. **No Proprietary Game Assets in Git:**
   - Never stage or commit `.big`, `.bik`, `.map`, `.wak`, `.tga`, `.dds`, `.w3d`, `.csf`, `.str`, `.mp3`, `.wav`, `.exe`, `.dll`, `.crc`, or retail archives.
   - Read retail game data in place from `~/Games/generalszh` via `-assetPath ~/Games/generalszh` (`CNC_GENERALS_ZH_PATH`).
3. **Complete Independence from Other User Repositories:**
   - Do not read, modify, import crates from, or link against any other repository on this machine (`cnc-gen-rig`, `cnc-gen-engine`, `rust-ra2-arena`, `ra2-arena-engine`, `civrig`, `rignarok`).
4. **No Upstream Contact:**
   - Never open PRs, issues, or comments on `fbraz3/GeneralsX` or `TheSuperHackers/GeneralsGameCode`.
5. **No Window Disruption on the Host Mac:**
   - The owner works actively on this Mac. Never run GUI game windows unless explicitly asked, never bring windows to the foreground, never resize windows, and never run `open`. Always pass `-headless` with `SDL_VIDEODRIVER=dummy` and `SDL_AUDIODRIVER=dummy`.

---

## 4. Shared Multi-Agent Workspace: GitHub Issues & Discussions Protocol

We use **GitHub Issues** and **GitHub Discussions** on `chayuto/generals-rust` as our persistent shared coordination board across agents and sessions.

### 4.1 Where Things Live on GitHub
- **GitHub Discussions (Architecture, Specs & Cross-Agent Broadcasts):**
  - **Architectural Decisions & RFCs:** Any change to `SimContext`, `ObjectArena`, `ModuleArena`, `SleepyUpdateHeap`, FFI bridge patterns, or determinism conventions is posted and updated in GitHub Discussions.
  - **Phase Execution Boards & Handoffs:** Session handoff summaries, Phase 1 spike telemetry tables, and CI cache status are posted to the active Phase Discussion thread.
- **GitHub Issues (Executable Work Packages & Atomic Translation Packets):**
  - Every Work Package (`WP-0.1`..`WP-7.3`) has a tracking Issue under its Phase Milestone (`Phase 0`..`Phase 7`).
  - Every task is classified across **two orthogonal axes**:
    1. **Model Complexity Tier (`tier:*`):**
       - `tier:1a-lite` (Class `C1` leaf modules)
       - `tier:1b-fast` (Class `C2` standard modules)
       - `tier:2-engineer` (Class `C3` complex/multi-facet modules)
       - `tier:3-architect` (Class `C4` spine, FFI seams, & desync triage)
    2. **Execution Environment & Access Capability (`env:*`):**
       - `env:local-full` — **Full Local Workstation Agent:** Has access to retail `.big` game assets at `~/Games/generalszh`, the local `vendor/GeneralsX` C++ reference clone/build (`GeneralsXZH`, `compile_commands.json`, `z_googletest`), and local replay directories. Required for Phase 0 setup, C++ CMake/FFI seam bring-up (`WP-1.1`, `WP-5.0`), generating Tier 0 AST packets + C++ Golden Test Vectors (`scripts/prep_packet.py`), retail `.big`/`.map` reader verification (`WP-2.4`, `WP-7.2`, `WP-7.3`), and interactive CRC desync debugging.
       - `env:ephemeral-web` — **Ephemeral Web / Cloud Sandbox Agent (Codebase-Only Access):** Has access **only** to this git repository (`chayuto/generals-rust`) and a standard Rust/Python toolchain (no local `~/Games/generalszh` `.big` files and no local `vendor/GeneralsX` C++ binary). Translates modules from committed Tier 0 packets (`packets/<phase>/<module>.md`), runs **Gate V1** (`scripts/lint_determinism.py` + `cargo check`) and **Gate V2-Golden** (`cargo test` against committed C++-generated Golden Test Vectors) locally in-container, and pushes branches/PRs to trigger **Gate V2-FFI / Gate V3 / Gate V4** on public GitHub Actions CI!
       - `env:hybrid-ci` — Work packages where an `env:local-full` agent seeds the FFI seam and Golden Vectors first, unlocking parallel `env:ephemeral-web` execution for all child module packets.

### 4.2 Producer–Consumer Contract Between `env:local-full` and `env:ephemeral-web` Agents
Because an `env:ephemeral-web` agent cannot read `~/Games/generalszh` or a local `vendor/GeneralsX` build tree, **an issue may only be marked `status:ready-for-agent` + `env:ephemeral-web` after an `env:local-full` agent (or CI job) has committed:**
1. **The Self-Contained Translation Packet (`packets/<phase>/<module>.md`):** Contains the stripped GPLv3 C++ `.h`/`.cpp` source from `fbraz3/GeneralsX` (`84a7a20`), the AST-extracted dependency signatures, and the Rust `ModuleData`/`Snapshot` skeleton.
2. **The In-Repo Golden Vector Oracle (`crates/*/tests/golden_<module>.rs` or `tests/golden/<module>.json`):** 1,000-seed input/output + `XferSave` bytes + `XferCRC` + `GetGameLogicRandomSeedCRC()` test vectors recorded from the C++ reference build. This guarantees the `env:ephemeral-web` agent has a runnable `cargo test` V&V oracle inside its codebase-only sandbox before pushing to GitHub Actions CI.

### 4.3 Agent Task-Claiming & State-Transition Protocol
To prevent two agents from duplicating work on the same module:

1. **Discover Ready Work Matching Your Environment & Tier:**
   - If you are an **Ephemeral Web Agent (codebase-only access)**:
     ```bash
     gh issue list --repo chayuto/generals-rust --label "status:ready-for-agent" --label "env:ephemeral-web"
     ```
     *(Never claim an issue labeled only `env:local-full` from an ephemeral web container!)*
   - If you are a **Local Full-Access Agent (running on the host Mac with `~/Games/generalszh` and `vendor/GeneralsX`)**:
     ```bash
     gh issue list --repo chayuto/generals-rust --label "status:ready-for-agent"
     ```
   *(Never start an issue labeled `status:blocked-on-vv` until its verification harness issue is closed!)*
2. **Claim the Issue Atomically:**
   Swap the label from `status:ready-for-agent` to `status:in-progress` and post a short claim comment stating the target branch/commit and the V&V gates that will be run:
   ```bash
   gh issue edit <NUM> --repo chayuto/generals-rust \
     --remove-label "status:ready-for-agent" \
     --add-label "status:in-progress"
   gh issue comment <NUM> --repo chayuto/generals-rust \
     --body "Claimed by Tier <X> worker. Target V&V gates: V1, V2, V3, V4."
   ```
3. **Compartmentalized Execution:**
   - Touch **only** the file(s) scoped in the issue packet.
   - Follow [docs/RUST_CONVENTIONS.md](docs/RUST_CONVENTIONS.md) strictly.
4. **Verify Before Closing (Mandatory V&V Proof):**
   - Run the required V&V gates (`Gate V1` linter, `Gate V2` differential test, `Gate V3` shadow-verify, `Gate V4` replay check).
   - Post the exact command output / pass summary as a comment on the Issue, commit to git, push, and close the Issue (or transition to `status:vv-review` if waiting for CI matrix completion).
5. **Escalation Protocol on Failure:**
   - If a `Tier 1A` or `Tier 1B` worker fails `Gate V1` (`cargo check`) after 1 retry, it commits its work to a `wip/packet-<id>` branch, relabels the issue `tier:2-engineer`, posts the compiler error output on the issue, and stops.
   - If any worker fails `Gate V2` (differential test) or `Gate V3/V4` (replay CRC mismatch), it labels the issue `status:crc-desync` + `tier:3-architect` and attaches the mismatch frame / seed diff for Tier 3 diagnosis.

---

## 5. Repository Map

- [PLAN.md](PLAN.md) — Master architectural blueprint, `SimContext`/`ObjectArena`/`ModuleArena`/`SleepyUpdateHeap` design, 3 FFI strangler patterns, and 337-packet roadmap across Phases 0–7.
- [docs/MULTI_AGENT_ROADMAP.md](docs/MULTI_AGENT_ROADMAP.md) — Complete task/subtask dependency DAG (`Issues #4..#33` & `PKT-001..PKT-463`), 6 master unblockers, and Milestones `M0`–`M7`.
- [docs/RESEARCH.md](docs/RESEARCH.md) — Verified technical reference for `fbraz3/GeneralsX` at `84a7a20` (48-singleton init table, 16-step `GameLogic::update()` pipeline, `m_sleepyUpdates` heap rules, `Object` + 222 `BehaviorModule` layout, `XferCRC` / `GameLogic::getCRC()` specification).
- [docs/AGENT_WORKFLOW_AND_CI.md](docs/AGENT_WORKFLOW_AND_CI.md) — Tier 0 AST Context Slicer design, 4-Tier Agent Routing matrix, 4-Gate V&V Funnel specification, and Free Public GitHub Actions CI Offload Pipeline.
- [docs/RUST_CONVENTIONS.md](docs/RUST_CONVENTIONS.md) — Mandatory C++-to-Rust determinism translation rules, type mappings, and worker cheat sheet.
- `scripts/bootstrap_repo.sh` — Idempotent environment, git hook, and self-test verifier.
- `scripts/lint_determinism.py` & `scripts/test_lint_determinism.py` — Gate V1 static determinism linter and its automated V&V unit test suite.
