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
| Experience and veterancy (members, `crc` and `xfer` stay in C++) | `GeneralsMD/.../GameLogic/Object/ExperienceTracker.cpp` | `PKT-003` |
| `LifetimeUpdate`, a behavior module (`m_dieFrame`, `crc` and `xfer` stay in C++) | `GeneralsMD/.../GameLogic/Object/Update/LifetimeUpdate.cpp` | `PKT-339` |

Nothing else. `zh_ffi` also exports the byte CRC, but the engine does not call it.

A swapped function that tells the engine about a change must store the change first and write nothing afterwards, because C++ reads the object's state inside that call. `zh_ffi`'s tracker got this wrong until it was wired in; two tests in `crates/zh_ffi/src/lib.rs` now hold it.

## Running a ported module in the engine

Behavior modules are written against the `SimView`/`SimApi` traits. `zh_ffi::engine_sim::EngineSim` implements both over C++ callbacks, bound to one object for one call. `scripts/gen_engine_sim.py` writes the impl; a method it does not connect aborts the game with its name. To connect one, add it to `WIRED` in the script, write `engine_<name>` in `engine_sim.rs`, extend `ZhEngineSimCallbacks` and rerun the script. The trait methods that return references (`object`, `rules` and so on) cannot be connected this way; see `OPEN_DECISIONS.md` item 10.

## The fault test

A check that passes proves nothing until a wrong build fails it. Each swap has a variant with a deliberate fault:

```bash
python3 scripts/build_rust_engine.py --mutate random       # one extra logic draw
python3 scripts/build_rust_engine.py --mutate experience   # one experience point too many
python3 scripts/build_rust_engine.py --mutate lifetime     # every lifetime one frame too long
python3 scripts/check_replays.py --check --expect-rust \
  --binary vendor/GeneralsX/build/macos-vulkan/GeneralsMD/GeneralsXZH-rust-mutant-<swap>   # must FAIL
```

| Fault | Replays that fail |
|---|---|
| `random` | all six, at frame 100 |
| `experience` | `macos_2p_custom_map` (frame 14700) and `macos_6p_custom_map` (frame 17600) only |
| `lifetime` | `linux_2p_custom_map` (frame 2000) only |

So the experience swap is exercised by two replays, and the other four say nothing about it. The lifetime swap is exercised by one. One-off faults showed that replay also reaches the other two entry points: removing the kill in `update` diverges at frame 1800, and an extra random draw after `setLifetimeRange` diverges at frame 2000. A first generator fault that only changed one returned value passed, because the CRC covers the generator state and whatever the value goes on to change. Add a fault for every new swap and record which replays catch it.

## How the swap is built

`scripts/build_rust_engine.py` edits nothing in `vendor/GeneralsX`. It patches a copy of each source file under `target/`, compiles it with that file's own flags, and places the objects ahead of `libz_gameengine.a` on the game's link line. The library stays stock, which matters because the golden-vector oracles link against it.

## Playing it

`GeneralsXZH-rust` sits next to the stock binary and takes the same arguments. Only the headless replay path above has been run. Windowed play, menus, input and sound have **not** been tested.
