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
   - Read retail game data in place from `~/Games/generalszh` via `-assetPath ~/Games/generalszh` (`CNC_GENERALS_ZH_PATH`). Download and layout: [docs/LOCAL_ASSETS.md](docs/LOCAL_ASSETS.md).
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

### 4.4 Three-Tier Resource Cost Hierarchy & Operational Mode

> **Dated Decision (`2026-11-22` — Active):** All **`env:ephemeral-web` remote agent assignments are PAUSED** until further notice (GitHub Actions monthly runner quota nearly exhausted). Until explicitly re-enabled by the owner, execute all work **locally (`env:local-full`)**, prioritize `$0` local deterministic compute before committing, batch pushes to `origin/main` cleanly to minimize CI runs, and do **not** open `status:ready-for-agent` remote-offload Issues or commit `#[ignore]` remote stubs.

**Local Multi-Agent & Read-Only Chat Execution Rules (`2026-11-22` — [docs/TOKENOMICS_AUDIT.md](docs/TOKENOMICS_AUDIT.md) §6):**
- Use a thin `Tier 3` Local Coordinator + 2–4 parallel `Tier 1A/1B/2` worktree workers (`share`) on separate higher-remaining-quota pools with fresh contexts and disjoint file ownership when quota headroom allows (avoid monolithic single-thread waves that suffer $O(T^2)$ context growth).
- Enforce a hard `2–3` read-turn cap before first write in worker briefs (C++ oracle by Turn 4, Rust by Turn 6), never run full `./scripts/bootstrap_repo.sh` inside a worktree, pre-slice C++/Rust references via `scripts/prep_packet.py`, and run `1` worker serially (instead of parallel fan-out) whenever worker pool headroom is `< 30%`.
- For multi-packet local waves (`WP-5.2+`), cap waves at `10–12` packets, pre-pack a deduplicated bundle via `python3 scripts/pack_wave.py` (`< 100k` worker peak context target — [docs/TOKENOMICS_AUDIT.md](docs/TOKENOMICS_AUDIT.md) §6.7), ban full-file reads of `sim/mod.rs`/`module_factory.rs`/`golden_*.rs`/`ARCHITECTURE_SPEC.md`, and always pass `-k "bt 15" -k "quit" -o "quit"` to `lldb --batch`.
- Prioritize `WP-5.0` force-multiplier contracts (unlocking the `225` `WP-5.1`–`WP-5.4` `BehaviorModule` packets) and targeted worker `cargo test` runs before a single coordinator integration, `./scripts/bootstrap_repo.sh`, and batched `git push`.
- Restrict read-only public-repo deep-think chat agents (no VM) to compact `C4` architecture/contract specs, deep determinism audits/desync triage, or direct-to-disk drafts verified by local workers.
- Never commit throwaway helper files for a read-only chat agent, and never paste long chat outputs or verbose instructions back into paid local coordinator sessions.

**Empirical Cost Principle ([docs/TOKENOMICS_AUDIT.md](docs/TOKENOMICS_AUDIT.md)):**
1. **Local Deterministic Compute = `$0` / Cheap (`0` LLM Tokens, `0` CI Minutes):**
   - Running local scripts, compilers, `scripts/build_oracle.py`, `scripts/prep_packet.py`, `scripts/port_map.py`, `./scripts/bootstrap_repo.sh`, `cargo check`/`cargo test`, `-O0`/`-O2`/UBSan builds, and one-shot Python filters/extractors costs **zero tokens**. Always replace repetitive LLM reading/writing with local deterministic scripts, pre-filter large files/JSONs with local Python before pulling snippets into context, and run full local V&V before pushing.
2. **`env:ephemeral-web` Remote LLM Tokens = `$0` FREE, *Bounded by GitHub Actions Runner Minutes* (`3,000` min/month):**
   - Each remote cloud sandbox session costs `$0` local LLM tokens, but its managed cloud worker runner bills ~5–20 Actions minutes against the monthly account quota even on a public repository. Maximize verified LOC per session by targeting large packets (`2,000–7,000+` C++ LOC) or macro-batched leaf waves (`10–30` modules per issue), and never spin up a second remote retry session to fix minor 1–2 seed edge cases that can be diagnosed and patched locally with cheap compute.
3. **`env:local-full` Local LLM Tokens = EXPENSIVE:**
   - Optimizing total tokens across both environments is wrong — optimize **verified Rust progress per local token** (while respecting the monthly cloud runner budget). Never spend local tokens implementing medium or large Rust modules that can be verified via a lean Golden Vector oracle and offloaded to free remote workers.
   - Invest expensive local tokens strictly in **force multipliers**: (1) deterministic local scripts/code generators that convert future token work into `0`-token local compute (`scripts/build_oracle.py`, `tests/cpp_harness/oracle_common.h`, `WP-5.0` batch generator), (2) `< 300–600`-LOC lean seeds that unlock `2,000–10,000+`-LOC free remote PRs, (3) `C4` architectural spines (`SimContext`, `ObjectArena`, `ModuleArena`, `SleepyUpdateHeap`, `Pathfinder`, `GameLogic::update()`), and (4) `Gate 0` / `Gate V3–V4` replay verification & desync triage.

Apply this **Ultra-Lean Seeding & Local-Compute-First Protocol** strictly before opening any `env:ephemeral-web` Issue:

1. **Zero-Stub C++ Oracle Compilation (`scripts/build_oracle.py`):**
   - **NEVER create** new per-packet `stubs_pktXXX/` trees. Compile compact (`~150–550` LOC) C++ oracle drivers directly against `vendor/GeneralsX/build/macos-vulkan` (`libz_gameengine.a`) via `python3 scripts/build_oracle.py`.
2. **Zero Local Rust Skeletons (`0` Local Production Rust LOC):**
   - Generate `packets/<phase>/<PKT>_<Module>.md` deterministically with `python3 scripts/prep_packet.py <PKT>` (`0` local LLM tokens).
   - **Do NOT write** compiling `todo!()` skeletons in `crates/*/src/`. Let the free `env:ephemeral-web` worker create `<module>.rs`, wire `mod.rs`, and update `docs/PORT_MAP.md`.
3. **Input-Carrying + Self-Diagnosing Seed-0 Golden Vector JSON (`~40–60` LOC Guard Test):**
   - Store **explicit compact inputs** alongside expected outputs in `docs/golden_vectors/<PKT>.json` — never duplicate complex C++ input synthesizers in Rust.
   - Always include **Seed-0 per-section / per-field diagnostic checkpoints** inside the JSON oracle (post-PR `#84` rule: never emit only a single opaque checksum over dozens/hundreds of fields, so remote agents self-diagnose mismatches without local help).
   - Commit only a compact `~40–60`-LOC `crates/*/tests/golden_<pkt>.rs` (active JSON structural check + `#[ignore]` 1,000-seed contract stub); let the remote worker write the full 1,000-seed assertion loop and remove `#[ignore]`.
4. **Target Selection Across Phases 2–7:**
   - **Lean-Seed & Offload (`env:ephemeral-web`, `$0` Tokens, High-ROI Cloud Sessions):** All large/medium C1–C3 modules (`#72`–`#81`, macro-batched Phase 2–3 leaf waves of `10–30` modules, and auto-generated Phase 5–6 `WP-5.0` / `WP-6.4` waves) once verified against `build_oracle.py` or a shared CMake oracle. Do **not** spend an individual cloud runner session on a tiny standalone leaf unless batched with sibling leaves.
   - **Keep Local (`env:local-full` Force Multipliers):** Phase 0 (`Gate 0` replay corpus), `C4` architectural spines & FFI seams (`GameMath`, `Xfer`, `ModuleArena`, `ObjectArena`, `SleepyUpdateHeap`, `WP-5.0` generator, `Pathfinder`, `GameLogic::update()`), retail `.big`/`.map`/`.rep` checks, and CRC desync triage.
5. **PR Merge Hygiene & Escalation Triage:**
   - Prefer **Squash & Merge** with clean conventional commit messages on remote PRs. When reviewing a failing remote PR, check Seed-0 diagnostics and patch minor edge cases locally with zero-token local compute rather than burning another remote cloud runner session.
6. **Phase 5–6 Fan-Out Automation:**
   - Build `WP-5.0` (`SimContext`/`ModuleArena`/`ModuleTrait` + automated C++/Rust oracle & packet generator) once locally (`0`-token local script generation once written) to unlock all `221` `BehaviorModule` packets (`WP-5.1`–`WP-5.4`) and `29` Script packets (`WP-6.4`) on free remote quota with near-zero marginal local token cost.
7. **Local-Compute-First & CI Budget Discipline:**
   - Run `./scripts/bootstrap_repo.sh` and `cargo test` locally (`0` tokens, `0` CI minutes) before pushing; keep `macos-15` (10×) and `windows-latest` (2×) `workflow_dispatch`-only in `.github/workflows/ci-guardrails.yml`; fix minor near-passing remote PRs locally rather than consuming another cloud runner session.

---

## 5. Repository Map

- [PLAN.md](PLAN.md) — Master architectural blueprint, `SimContext`/`ObjectArena`/`ModuleArena`/`SleepyUpdateHeap` design, 3 FFI strangler patterns, and 337-packet roadmap across Phases 0–7.
- [docs/MULTI_AGENT_ROADMAP.md](docs/MULTI_AGENT_ROADMAP.md) — Complete task/subtask dependency DAG (`Issues #4..#33` & `PKT-001..PKT-463`), 6 master unblockers, and Milestones `M0`–`M7`.
- [docs/RESEARCH.md](docs/RESEARCH.md) — Verified technical reference for `fbraz3/GeneralsX` at `84a7a20` (48-singleton init table, 16-step `GameLogic::update()` pipeline, `m_sleepyUpdates` heap rules, `Object` + 222 `BehaviorModule` layout, `XferCRC` / `GameLogic::getCRC()` specification).
- [docs/AGENT_WORKFLOW_AND_CI.md](docs/AGENT_WORKFLOW_AND_CI.md) — Tier 0 AST Context Slicer design, 4-Tier Agent Routing matrix, 4-Gate V&V Funnel specification, and Free Public GitHub Actions CI Offload Pipeline.
- [docs/TOKENOMICS_AUDIT.md](docs/TOKENOMICS_AUDIT.md) — Empirical tokenomics audit of manual `env:ephemeral-web` seeding (`#70`, `#71`, `#74`), stub sprawl analysis, and updated seeding/CMake oracle rules.
- [docs/LOCAL_ASSETS.md](docs/LOCAL_ASSETS.md) — Where the retail Zero Hour assets live locally, how to (re)download them via steamcmd, and usage rules.
- [docs/RUST_CONVENTIONS.md](docs/RUST_CONVENTIONS.md) — Mandatory C++-to-Rust determinism translation rules, type mappings, and worker cheat sheet.
- `scripts/bootstrap_repo.sh` — Idempotent environment, git hook, and self-test verifier.
- `scripts/lint_determinism.py` & `scripts/test_lint_determinism.py` — Gate V1 static determinism linter and its automated V&V unit test suite.
