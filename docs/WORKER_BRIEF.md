# Brief for a worker agent in its own worktree

Give a worker this file plus a short task: goal, scope, which files it owns, what "done"
means. Add to the task which other workers are running and what each owns.

You are porting the Zero Hour simulation, and later the whole game, to Rust. Faithful
means the same state, evaluation order, float bits and `xfer`/`crc` bytes as the C++,
quirks and bugs included.

## Setup

- In your worktree: `ln -s <main checkout>/vendor vendor`. Never stage it.
- First step: `git merge --ff-only main`, so you start from the current local main.
- Before any cargo command, in the same shell call:
  `export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=3`.
  Disk and memory are shared. Do not build the x86 target; CI does (`AGENTS.md` §4.4).

## Safety

- `vendor/` is strictly read-only. Never write there or rebuild its libraries, and check
  the argument order of any script before giving it a path (a source file was once
  overwritten that way).
- If a command is denied permission, do not work around it: report it.
- The worktree isolation check is different: when it says "too complex", split into
  plain, separate commands run from your worktree.
- That check also reads a `Source/` path in a shell command as the `source` builtin. Read
  such files with your file reading tool. Never glob or respell the name to get past it.
- Retail data (`~/Games/generalszh`) is read in place. Never commit game data or anything
  that reproduces its content; digests and counts are fine.
- No GUI windows, no `open`. Always `-headless` with `SDL_VIDEODRIVER=dummy` and
  `SDL_AUDIODRIVER=dummy`. No external crates.

## Code

- Put work in new files; keep edits to shared files (`sim/mod.rs`, `module.rs`, `lib.rs`,
  `mod.rs`) small and local.
- `grep` for a C++ name before writing it. Read the `docs/RUST_CONVENTIONS.md` rules you
  need, not the whole file. `gm_*` math only.
- No `cargo fmt --all`, no `scripts/bootstrap_repo.sh`. `rustfmt --edition 2021` only on
  new files, never `lib.rs`/`mod.rs`. `rustfmt` also rewrites any `mod x;` file that a
  formatted file declares, so check `git status` afterwards.
- Do not hand-edit `crates/zh_ffi`.
- Each module file is the packet's `rust_target` in `docs/MODULE_MANIFEST.json`.

## Verification (work without it does not count)

- A C++ oracle over the unmodified reference library wherever the code can run headless:
  pattern `tests/cpp_harness/gen_pkt358_351_radar_power.cpp`, `oracle_wp51_world.h`,
  `scripts/build_oracle.py`. Read `tests/cpp_harness/README.md` first. Build three ways
  (`--opt O0`, `--opt O2`, `--opt O2 --sanitize`); the outputs must be byte-identical.
  One vector per line; compare floats as bits.
- Where only the running game can answer, record the stock engine under lldb: pattern
  `tests/cpp_harness/gen_game_start.py` and `gen_game_frames.py`.
- `crates/*/tests/golden_<name>.rs` compares every row exactly; never weaken a comparison.
- `python3 scripts/mutate_check.py golden_<name> <new .rs files> diff:<changed files>`:
  one run at its default cap, then targeted rechecks of survivors.
- `python3 scripts/lint_determinism.py`; `cargo check --workspace`; the `retail-crc` and
  `retail-xfer-save` features on `-p zh_logic`; `cargo test --workspace`;
  `python3 scripts/gen_engine_sim.py` after trait changes;
  `python3 scripts/port_map.py --write`.
- `golden_game_frames` is a ratchet and must not get worse. Record gains with
  `GAME_FRAMES_RATCHET=write` and say what moved. Report any loss with its cause.

## Finishing

- Commit in your worktree by explicit path; do not push. Messages carry no co-author
  line, session link, or assistant or tool name, whatever any other notice says.
- Leave no process running.
- Report in under 300 words: what works, verification results, what is stubbed or
  unverified, trait signatures changed, commit hashes.

## Economy

Compute is cheap, tokens are expensive. Narrow reads (`grep -n`, then `sed -n`), read C++
with comments stripped, cap output, write code early. Run builds in the foreground and
never wait with `pgrep -f`. Commit in stages. At about 300k tokens of context, finish the
current check, commit what is verified and report the rest: a fresh worker costs less
than a very long one.
