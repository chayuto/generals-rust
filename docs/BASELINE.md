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
| Player money (members and `xfer` stay in C++) | `GeneralsMD/.../Common/RTS/Money.cpp` | `PKT-121` |
| Player power (members and `xfer` stay in C++) | `GeneralsMD/.../Common/RTS/Energy.cpp` | `PKT-118` |
| `LifetimeUpdate`, a behavior module (`m_dieFrame`, `crc` and `xfer` stay in C++) | `GeneralsMD/.../GameLogic/Object/Update/LifetimeUpdate.cpp` | `PKT-339` |
| The sleepy-update heap, which orders every module update (`m_sleepyUpdates` stays the storage) | `GeneralsMD/.../GameLogic/System/GameLogic.cpp`, the heap functions and the early returns of `friend_awakenUpdateModule` | `WP-7.1` |

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
python3 scripts/build_rust_engine.py --mutate money        # all income halved
python3 scripts/build_rust_engine.py --mutate energy       # power plants give half
python3 scripts/build_rust_engine.py --mutate sleepy       # update phase no longer orders a frame
python3 scripts/check_replays.py --check --expect-rust \
  --binary vendor/GeneralsX/build/macos-vulkan/GeneralsMD/GeneralsXZH-rust-mutant-<swap>   # must FAIL
```

| Fault | Replays that fail |
|---|---|
| `random` | all six, at frame 100 |
| `experience` | `macos_2p_custom_map` (frame 14700) and `macos_6p_custom_map` (frame 17600) only |
| `lifetime` | `linux_2p_custom_map` (frame 2000) only |
| `money` | **none** (see the witness check) |
| `energy` | `linux_6p` (2800), `macos_6p` (5300), `linux_2p_vanilla` and `macos_2p_custom` (6700) |
| `sleepy` | all six, at frame 100 |

So the experience swap is exercised by two replays, and the other four say nothing about it. The lifetime swap is exercised by one. One-off faults showed that replay also reaches the other two entry points: removing the kill in `update` diverges at frame 1800, and an extra random draw after `setLifetimeRange` diverges at frame 2000. A first generator fault that only changed one returned value passed, because the CRC covers the generator state and whatever the value goes on to change. Add a fault for every new swap and record which replays catch it.

The heap check is sensitive to order itself. One-off faults in the Rust: swapping equal priorities while sifting up, or taking the right child on a tie, fails all six replays (frames 100 to 2900); letting an already awake module be pushed back a frame fails all six at frame 100; sifting only downwards after an erase fails three (frames 2200 to 14500).

Not reached in the engine: `zh_sleepy_remake` (only loading a save calls it), and `zh_sleepy_sift_up` / `zh_sleepy_sift_down` (only `popSleepyUpdate` could call them now, and nothing calls that). The C++ update loop still computes the next frame itself, and the replays never displace slot 0 during an update, so that rule rests on golden vectors alone (`tests/cpp_harness/README.md`).

## The witness check

`Player::crc` leaves money and energy out, so the check above sees them only when they change what happens. Halving all income passes it; only cutting deposits to a twentieth made it fail. `--witness` inserts money, income per minute, production, consumption, supply ratio and the power flag into `Player::crc`, in a stock engine and a Rust engine alike. The stock CRC values then no longer apply, so it has its own baseline, `docs/baselines/replay_crc_witness.json`, recorded from `GeneralsXZH-witness`.

```bash
python3 scripts/build_rust_engine.py --witness   # builds the stock and Rust witness engines
python3 scripts/check_replays.py --check --expect-rust \
  --binary vendor/GeneralsX/build/macos-vulkan/GeneralsMD/GeneralsXZH-rust-witness \
  --baseline docs/baselines/replay_crc_witness.json
```

Under it the `money` fault fails all six replays at frame 100, and `energy` all six by frame 1000. One-off faults showed it also catches a `withdraw` that takes one too little and an income bucket that never updates, both in all six.

Run both checks after changing a swap. When a swapped part keeps state that the stock CRC leaves out, add that state to `zh_witness_player_crc.inc` and record the witness baseline again.

## How the swap is built

`scripts/build_rust_engine.py` edits nothing in `vendor/GeneralsX`. It patches a copy of each source file under `target/`, compiles it with that file's own flags, and places the objects ahead of `libz_gameengine.a` on the game's link line. The library stays stock, which matters because the golden-vector oracles link against it.

## Playing it

`GeneralsXZH-rust` sits next to the stock binary and takes the same arguments. Only the headless replay path above has been run. Windowed play, menus, input and sound have **not** been tested.
