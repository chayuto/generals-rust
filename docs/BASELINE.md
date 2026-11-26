# Local replay baseline (`env:local-full` only)

This is the check that Rust code swapped into the real engine behaves like the code it replaces. It is not Gate 0 as `PLAN.md` defines it.

## What it proves and what it does not

- **Proves:** on this machine, a build with Rust linked in produces the same game-logic CRC as the stock C++ build, at every checkpoint (one per 100 frames) of six replays, 996 checkpoints in total.
- **Does not prove** agreement with the original recordings. The public `GeneralsXReplays` files mismatch from frame 100 against the local Steam data (issue #7), so they only serve as a source of player commands. Gate 0 is still open.
- Code that these replays never reach is not covered.

## Commands

```bash
python3 scripts/check_replays.py --check                  # stock engine still matches
python3 scripts/build_rust_engine.py                      # builds GeneralsXZH-rust
python3 scripts/check_replays.py --check --expect-rust \
  --binary vendor/GeneralsX/build/macos-vulkan/GeneralsMD/GeneralsXZH-rust
```

A full check takes about 10 seconds. The baseline is `docs/baselines/replay_crc.json`; re-record it (`--write`) only from the stock build, and only when the reference commit, the retail data or the replays change.

`--expect-rust` rejects a binary with no `zh_` symbols, so a stock build cannot pass as a Rust one. It does not show that a particular function is swapped; the fault test below does that.

## What runs on Rust in the game

| Part | Source replaced | Packet |
|---|---|---|
| Random number generator (logic, client, audio) | `Core/GameEngine/Source/Common/RandomValue.cpp` | `PKT-002` |

Nothing else. `zh_ffi` also exports the byte CRC and the experience tracker, but the engine does not call them yet.

## The fault test

```bash
python3 scripts/build_rust_engine.py --mutate
python3 scripts/check_replays.py --check --expect-rust \
  --binary vendor/GeneralsX/build/macos-vulkan/GeneralsMD/GeneralsXZH-rust-mutant   # must FAIL
```

The mutant makes one extra logic draw on the 100th call after seeding; all six replays diverge at frame 100. A first version that only changed one returned value passed, because the check sees the generator state and whatever the value goes on to change. Run the fault test again after adding a new swapped part, with a fault that the part's own state or output would carry into the CRC.

## How the swap is built

`scripts/build_rust_engine.py` edits nothing in `vendor/GeneralsX`. It patches a copy of the source file under `target/`, compiles it with that file's own flags, and places the object ahead of `libz_gameengine.a` on the game's link line. The library stays stock, which matters because the golden-vector oracles link against it.

## Playing it

`GeneralsXZH-rust` sits next to the stock binary and takes the same arguments. Only the headless replay path above has been run. Windowed play, menus, input and sound have **not** been tested.
