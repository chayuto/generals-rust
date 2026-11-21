# CLAUDE.md

This file gives coding agents guidance for working with the code in this repository.

@AGENTS.md

AGENTS.md (imported above) is the binding project constitution. It covers the V&V gates, governance and hygiene rules, the GitHub Issue claiming protocol, and local-vs-remote task routing. This file only adds what AGENTS.md does not cover.

## Shared working tree

Several agents use this folder at different times. Uncommitted or untracked files may be another agent's unfinished work. Don't revert, reformat, stage or commit them unless the user says so. Before you touch a file you didn't create, check `git status` and its modification time (`stat -f %Sm <file>`). Stage files by explicit path, never with `git add -A` or `git add .`.

The GitHub issue board is the coordination point, not this tree. `env:ephemeral-web` agents see only what has been pushed. A packet counts as ready only when its packet doc, golden JSON, skeleton and `#[ignore]`d golden test are on `origin/main` and an issue carries `status:ready-for-agent` + `env:ephemeral-web`.

## Commands

```bash
./scripts/bootstrap_repo.sh                    # session start; also runs everything CI's Linux gate runs
python3 scripts/lint_determinism.py            # Gate V1 linter (defaults to crates/; pass paths to narrow)
cargo check --workspace                        # Gate V1 typecheck
cargo test --workspace                         # Gate V2-Golden: every golden-vector suite
cargo test -p zh_common --test golden_pkt087   # one suite (suites are crates/*/tests/golden_*.rs)
cargo test -p zh_logic --test golden_sleepy_heap <name_filter>
```

Script self-tests (AGENTS.md §1.3 requires one per script in `scripts/`): `python3 scripts/test_lint_determinism.py`, `scripts/test_prep_packet.py`, `scripts/test_apply_ffi_seam.py`, `scripts/validate_manifest.py`.

These need `vendor/GeneralsX` (`env:local-full` only):
- `python3 scripts/prep_packet.py PKT-077` (or a module name) runs the Tier 0 slicer. It writes `packets/phase<N>/<PKT>_<Module>.md` from `docs/MODULE_MANIFEST.json`, the vendor C++ and the worker cheat sheet in `docs/RUST_CONVENTIONS.md`.
- `cargo build --release -p zh_ffi` produces `target/release/libzh_ffi.a`. `python3 scripts/apply_ffi_seam.py` then links it into `vendor/GeneralsX/GeneralsMD/Code/GameEngine/CMakeLists.txt`, and running it again is safe.
- `cargo run --release -p zh_common --example exhaustive_gamemath -- sinf` prints a digest over all 2^32 inputs. Compare it with `tests/cpp_harness/exhaustive_gamemath.c` linked against vendor `libgm.a` (the build line is in that file's header). The reference build tree is `vendor/GeneralsX/build/macos-vulkan`.

## Architecture

This is a bit-exact port of the Zero Hour `GameLogic`/`Common` simulation from `vendor/GeneralsX`, a gitignored local clone at the commit pinned in AGENTS.md. The port is correct only if it matches the C++ `XferCRC` / `GameLogic::getCRC()` bit-for-bit. That's why the code is deliberately shaped like the C++: it uses the `Int`/`Real`/`Bool` aliases from `zh_common::types` and keeps the C++ field order and evaluation order.

**Crates.** The workspace has no external dependencies; keep it that way. `zh_common::golden_json` is a hand-rolled JSON reader for this reason.
- `zh_common` is the foundation:
  - `types`, `crc`
  - `random_value`: the logic, client and audio PRNG streams
  - `xfer`: the `Xfer` trait with Save/Load/Crc modes, `Snapshot` (`crc`/`xfer`/`load_post_process`), and `BitFlags`
  - `math`: fdlibm-exact `gm_*f` functions, coordinates, geometry
  - `core`: `Dict`, `NameKeyGenerator`, `StateMachine`
  - `ini`
- `zh_logic` is the simulation spine:
  - `arena`: `ObjectArena`/`ModuleArena` and the 64-bit `FacetMask`
  - `scheduler`: `SleepyUpdateHeap`
  - `economy`, `object`
  - `rules`: the enum/BitFlags registry
- `zh_ffi` is an `rlib` + `staticlib` C ABI that the C++ engine links. `include/zh_ffi.h` is written by hand, so change it in the same commit as `src/lib.rs`.

**Strangler-fig FFI** (PLAN.md §4, docs/ARCHITECTURE_SPEC.md §2) has three patterns:
- Pattern 1 replaces stateless or global functions (CRC, RNG).
- Pattern 2 is a Rust-owned component that calls back into a C++ parent that hasn't been ported yet. In Rust, that parent is a callback trait (`MoneyCallbacks`, `EnergyOwnerCallbacks`, `ExperienceParentAccess`), with `Noop*` implementations for tests.
- Pattern 3 (Phase 5) is a single C++ `RustBehaviorModuleAdapter` that fronts all 222 behavior modules.

**Target simulation shape** (PLAN.md §3, ARCHITECTURE_SPEC.md §1). `SimContext` has three layers: an immutable `RulesRegistry`, a mutable `SimWorld` (everything `getCRC()` hashes), and a write-only `ClientSink`. `SimWorld` doesn't exist yet. Module callbacks go through take-and-restore dispatch: the module's payload is moved out of the arena for the duration of the call, so the handler gets `&mut` access to the whole world without borrow conflicts. Use this instead of `RefCell` or `unsafe` aliasing. The spec describes it as a `SimWorld::with_module_dispatch` method. The actual implementation is a generic free function, `zh_logic::arena::with_module_dispatch(world, slot, arena_accessor, f)`.

**How a module gets verified:**
1. `docs/MODULE_MANIFEST.json` maps 833 C++ files to 463 packets (`PKT-001`..`PKT-463`), grouped into work packages (`WP-x.y`).
2. A C++ harness in `tests/cpp_harness/gen_*.cpp` `#include`s the unmodified vendor `.cpp` sources directly. A sibling `stubs_<wave>/` directory supplies minimal versions of the engine headers they pull in. The harness writes `docs/golden_vectors/<PKT|WP>.json`, usually to the path given as `argv[1]`. Most `.cpp` harnesses don't record their compile command; check the commit that added the harness.
3. Each `crates/*/tests/golden_*.rs` suite embeds its JSON with `include_str!`, parses it with the `zh_common::golden_json` helpers, and checks return values, `XferSave` bytes and `XferCRC`.
4. A suite for a packet that isn't implemented yet ships as a compiling `todo!()` skeleton with an `#[ignore]` test. Implementing the packet includes removing that `#[ignore]`.

Each module's doc comment names its packet ID, packet doc and golden test (see `crates/zh_common/src/ini/mod.rs`). Do the same in new modules.

## Gotchas

- The golden vectors were recorded with `USE_DETERMINISTIC_MATH=1, RETAIL_COMPATIBLE_CRC=0`, so they pass only with default features. Under the `retail-crc` / `retail-xfer-save` features the crates compile, but golden suites such as `golden_pkt001_002` fail, and that is expected. Still port `#if RETAIL_COMPATIBLE_CRC` branches behind these features (conventions rule 10).
- `docs/RUST_CONVENTIONS.md` says `zh_common::gamemath::gm_*`, but the real path is `zh_common::math::gm_*`. Most functions are re-exported from `math::gamemath`; `gm_floorf` and `gm_ceilf` are defined in `math/mod.rs`. `gm_powf`, `gm_expf` and `gm_logf` don't exist yet: port them with golden vectors first. Never fall back to std.
- The dev profile sets `overflow-checks = true`, so a missing `wrapping_*` shows up as a panic in tests. Fix the arithmetic, not the test.
- To exempt a single line from Gate V1, end it with `// determinism-ok: <reason>`. Use this sparingly.
- `.githooks/commit-msg` rejects any commit message that contains an assistant or tool brand name, case-insensitively (see its regex). That includes this file's own name, so a commit touching this file should say "agent guidance" instead.
- CI (`.github/workflows/ci-guardrails.yml`) runs on Linux for pushes to `main`, `packet/**` and `wp/**`: bootstrap plus `cargo test --workspace`. The macOS and Windows parity jobs run only for `main`, PRs and manual dispatch.
