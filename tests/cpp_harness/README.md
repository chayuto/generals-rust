# C++ golden-vector harnesses

Each `gen_*.cpp` here compiles unmodified `vendor/GeneralsX` sources, with minimal engine headers from a sibling `stubs_<wave>/` directory, and writes a `docs/golden_vectors/*.json` oracle that a `crates/*/tests/golden_*.rs` suite embeds. The oracle is the specification the Rust port is held to, so a defect in the reference code or in the harness becomes a defect in the port. This file collects the rules that prevent that, and the reference-code quirks found so far.

## Rules for a new harness

1. **Record the exact build and run commands** in the harness's header comment, including every `-include`, `-D` and `-O` flag.
2. **Write the output atomically.** Write `<out.json>.tmp` and rename it only after every vector has been produced and checked. Opening the real output path first means a crash leaves an empty or truncated file over the committed oracle. This happened to the first `WP-7.2b` attempt.
3. **Make the reference prove each vector.** Where the C++ can check itself (for example, every encoder output must decode back to its input), do it for every vector, and exit non-zero on any failure. Also check at startup any build-configuration invariant the vectors depend on (see `check_byte_order()` in `gen_wave4_compression.cpp`).
4. **Run under sanitizers and at several optimisation levels.** Build once with `-fsanitize=address,undefined` and at `-O0` and `-O2`; all outputs must be byte-identical. A difference means the reference has undefined behaviour or uninitialised reads, which the vectors would otherwise bake in.
5. **Grep for platform branches before you compile.** The reference workstation is an Apple Silicon Mac, so every `#if defined(__APPLE__)` branch in the code under test is the one the harness runs, while retail data and Windows/Linux builds take the other one. Run `grep -n __APPLE__` on every vendor file the harness compiles, and on the headers those files include, and decide for each hit which side the oracle must follow (usually the non-Apple side). See the watch list below.
6. **Add a non-ignored Rust test of the oracle's structure** (counts, header bytes, payload regeneration), so a bad regeneration fails CI even before the port exists. `golden_wp7_2b_compression.rs` has two such tests.
7. **Stamp vendor source SHA-256 fingerprints** (`"source_sha256"`) in every new oracle JSON header so pin drift or unrecorded local edits to vendor sources are detected immediately.
8. **Include explicit error-path and semantic-trap seeds** (`seed 0..N`) alongside randomized fuzz vectors — e.g. `-0.0` IEEE 754 bit sign preservation, `0`/`-1`/`MIN`/`MAX` boundaries, unknown/malformed enum or INI tokens, and truncated inputs (with bounds checks tested on the Rust side when C++ lacks bounds guards).
9. **Side-by-side C/C++ shadow compilation (`-Dfn=fn__original`)** for future Phase 4/5 in-process differential Gate V2-FFI / Gate V3 harnesses: compile the reference symbol renamed via `-Dfoo=foo__original` alongside the Rust `#[no_mangle] pub extern "C" fn foo` symbol so a single shadow binary can invoke both and assert bit-exact parity per call.
10. **Zero-stub C++ oracle compilation (`scripts/build_oracle.py`):** Legacy waves left `21` `stubs*` directories (`479` files, `11,312` LOC). Never create another `stubs_pktXXX/` directory. Compile all new C++ oracle drivers directly against `vendor/GeneralsX/build/macos-vulkan` (`libz_gameengine.a`) via `python3 scripts/build_oracle.py` (`--opt O0`, `O1`, `O2`, and `--sanitize` UBSan; keep the `__APPLE__` / `Byte` signedness awareness documented below for standalone compiler invocations). Serialize explicit test inputs directly inside `docs/golden_vectors/*.json` alongside expected outputs rather than mirroring synthetic fuzzers in Rust.
11. **Self-diagnosing Seed-0 checkpoints:** Never emit only a single opaque checksum (`defaults_crc` / `state_crc`) over dozens or hundreds of fields without granular Seed-0 breakdown fields in `docs/golden_vectors/<PKT>.json`. Always include per-section or per-field Seed-0 diagnostics so `env:ephemeral-web` workers can isolate any field discrepancy on pass 1 without local debugging.

## Known reference-code quirks

### `gimex.h` byte order on Apple Silicon (EA compression codecs)

- **Where:** `Core/Libraries/Source/Compression/EAC/gimex.h`, `ggetm()` (line 377) and `gputm()` (line 423), under `#if defined(__APPLE__) || (defined(__MWERKS__) && defined(__PPCGEKKO__))`.
- **What goes wrong:** that branch does native-endian 2- and 4-byte loads and stores. It was written for big-endian PowerPC Macs. On little-endian Apple Silicon the RefPack, BTree and Huffman encoders write byte-swapped stream headers (`fb 10 05 00 00` instead of `10 fb 00 00 05`). `REF_decode` parses part of the header by hand, misreads the swapped bytes, and crashes; ASan reports a heap over-read.
- **How the oracle handles it:** compile every EAC `.cpp` with `-include tests/cpp_harness/stubs_wave4_eac/eac_portable_byteorder.h`. That prelude includes the system headers and then undefines `__APPLE__`, so `gimex.h` takes the portable big-endian path, which is the format of retail data and Windows/Linux builds. Any future harness that compiles the EAC sources needs the same flag. That includes `PKT-451` (`DataChunk`), because retail `.map` files are RefPack blobs.
- **Guards:**
  - `check_byte_order()` in `gen_wave4_compression.cpp` exits with status 3 before writing anything if the objects were built without the prelude.
  - `test_wp7_2b_oracle_streams_are_big_endian` fails CI if a byte-swapped JSON is ever committed.

### Huffman decoder reads past the end of the stream

- **Where:** `Core/Libraries/Source/Compression/EAC/huffdecode.cpp`, the `GET16BITS` / `SQgetbits` refill macros.
- **What goes wrong:** the bit buffer is refilled 16 bits at a time, so decoding a valid stream reads up to 3 bytes past its end. The engine always decodes from a larger file buffer, so this never faults in the game, but it means the decoder's result could in principle depend on bytes that are not part of the stream.
- **How the oracle handles it:** `verify_decode()` decodes every stream from a copy followed by 64 bytes of `0x00`, and again followed by `0xff`. Both decodes must reproduce the source exactly. That proves the extra bytes never reach the output.
- **Port contract:** the Rust decoder may treat up to 4 bytes past the end as `0x00`; any further read is `CodecError::Truncated` (packet §1, rule 4).

### Huffman cannot decode its own empty-input stream

- **What goes wrong:** `HUFF_decode` never terminates on the stream `HUFF_encode` produces for a zero-length input. It keeps reading past the buffer; unsanitized builds die with `SIGBUS`.
- **How the oracle handles it:** the encoder output is recorded, but that one decode has no reference result and is skipped (see `compress_like_manager()`).
- **Port contract:** the Rust decoder only has to return, with `Ok` or `Err` (packet §1, rule 7). `test_wp7_2b_compression_1000_small_golden_vectors` decodes that stream, and the CI job timeouts (`timeout-minutes` of 3 on Linux and 5 cross-OS in `ci-guardrails.yml`) stop a hang from running for long.

### `min` is a macro, so NaN takes the second argument

- **Where:** `HighlanderBody::attemptDamage`, `min(m_amount, getHealth() - 1)`.
- **What happens:** a NaN damage amount becomes `health - 1` and is dealt. `if cap < amount` in Rust (`std::min` order) keeps the NaN.
- **Guard:** `golden_pkt330_327_fire` feeds NaN damage to every body type it uses.

### A module's own damage callback runs at its place in the list

- **Where:** `FlammableUpdate::doAflameDamage` damages its owner from inside `update()`. With two `FlammableUpdate` modules and the AFLAME status cleared from outside, the second ignites after the first one's `onDamage` has already run.
- **Port contract:** a callback replayed after the damage returns must see the object status of its turn (`attempt_damage_watching`), not the status afterwards.
- **Guard:** `golden_pkt330_327_fire` (one vector of 1,000 shows the difference).

### `snapshot_save_hex`'s CRC is not the module's CRC

- **Where:** `oracle_wp51::World::snapshot_save_hex(s, &crc)` returns a CRC of the saved bytes. `snapshot_crc(s)` runs the module's own `crc()`.
- **What happens:** most update modules add nothing in `crc()`, so the two differ.
- **Guard:** `golden_pkt358_351_radar_power` compares the Rust `crc()` with `snapshot_crc`.

### Oversized integer INI tokens differ by platform

`scanType` takes the `__APPLE__` branch for integers too: `MinSpreadDelay = 100000000000` saturates with a warning on the reference Mac and is rejected elsewhere. Keep fixture tokens in range.

### `__APPLE__` watch list (not yet handled by an oracle)

These are the other `__APPLE__` branches in the vendor sources at the pinned commit that can change data, not just file paths. Check them before writing a harness that compiles these files.

| Location | Apple behaviour | Other platforms | Status |
| :--- | :--- | :--- | :--- |
| `Core/Libraries/Source/Lib/Include/Precompiled/BaseTypes.h:35` | `Byte` is `unsigned char` | `Byte` is `char` (signed on x86 Windows/Linux) | `stubs_wave3_ini/PreRTS.h` defines `Byte` as `int8_t`, matching Windows. Any harness that includes the real `BaseTypes.h` gets the unsigned type, which changes sign extension (for example `INI::parseBitString8` reads `*(Byte*)store` into an `UnsignedInt`). |
| `Core/GameEngine/Source/Common/INI/INI.cpp:1686` (`scanType`, when `USE_STD_FROM_CHARS_PARSING` is 1, i.e. C++17 or later) | floats are parsed with `std::strtod`, and out-of-range values saturate to `±max` | floats are parsed with `std::from_chars` | Not yet compared. `gen_wave3_ini.cpp` compiles `INI.cpp` on macOS but does not record its `-std` flag; if it was C++17 or later, the `PKT-087` oracle follows the Apple side for float tokens. The two agree on correctly rounded in-range decimals, but may differ on out-of-range or unusual tokens. |
| `Core/Libraries/Source/WWVegas/WWLib/bittype.h:71` | `DWORD`/`ULONG` are `uint32_t` | same on Linux; Win32 `unsigned long` is 32-bit | Benign: chosen to match Win32. |

The other hits only choose include files, find the executable or user-data directory, or load platform libraries: `PlatformPaths.h`, `BIGFileSystem.cpp`, `GlobalData.cpp`, `ReplaySimulation.cpp`, `UnicodeString.cpp`, `GameMemoryNull.cpp`, `WWLib` (`FastAllocator.h`, `ini.cpp`, `registryini.cpp`, `TARGA.cpp`), `WW3D2`, and the GameSpy and GeneralsOnline networking code.

### `PartitionData::doSmallFill` writes cells without a capacity check

A small object normally spans at most two cells per axis, and its cell array has at least four slots. Float rounding of `(x - r)` and `(x + r)` can make the span three cells wide; with a four-slot array the C++ then writes past the end. `gen_partition_grid.cpp` tests each pending small fill before `update()` (`small_fill_overflows()`), discards a script that would overrun, and records the count as `small_overflow_dropped`. Any harness that places small objects on cell boundaries needs the same check.

### The sleepy-update loop rebalances slot 0, not the module that ran

- **Where:** `GameLogic.cpp`, the sleepy loop in `GameLogic::update()`: `u->friend_setNextCallFrame(now + sleepLen); rebalanceSleepyUpdate(0);`.
- **What goes wrong:** `update()` can put another module in slot 0, by creating an object whose module runs in an earlier phase, or by waking one for the current frame. C++ still rebalances slot 0, so the module that ran keeps its old slot with a new, later priority and the heap order stays broken until something moves it. Later modules run in a different order than a repaired heap would give.
- **How it was missed:** `gen_sleepy_heap_vectors.cpp` only woke other modules for `now + 1`, and the Rust port repaired the heap by rebalancing the module's own slot. 1,000 vectors passed.
- **Guard:** `WP5_0_SleepyHeap_displaced.json` (the harness's second output) and `test_sleepy_heap_displaced_root_golden_vectors`. The six local replays never displace slot 0 (about 40 million updates, measured 2026-11-27), so a replay check cannot see this.
- **Lesson for new harnesses:** inputs chosen to be "safe" for the reference are inputs the port is never held to.

### Tunnel passenger lists can keep pointers to destroyed objects

- **Where:** `TunnelContain.cpp` and `TunnelTracker.cpp`.
- **What goes wrong:** the reference reads freed memory when the destroy list is flushed (a) while a passenger list still holds a destroyed object, which happens after a unit is exited through another player's tunnel and then destroyed, or (b) while an object's container has been destroyed, which happens after a unit enters a sold or killed tunnel that is later destroyed.
- **How the oracle handles it:** `gen_pkt300_tunnel.cpp` skips the flush in those states only; the operations themselves still run. The port stores object ids, so it has no matching undefined behaviour, and no vector says what it should do there.

### A cave never leaves its network when it is deleted

- **Where:** `CaveContain.cpp`. It overrides `onDie` but not `onDelete`, unlike `TunnelContain`.
- **What goes wrong:** a cave destroyed without dying stays on its tracker's list of entrances. If it becomes the front entrance after a flush, `TunnelTracker::onTunnelDestroyed` calls `onContainedBy(nullptr)`: the riders stay listed but are contained by nothing, and can be listed a second time. A cave whose index is switched before `onBuildComplete` is listed twice.
- **How the oracle handles it:** `gen_pkt264_cave.cpp` runs all of this.

### A cave can end up without a team

- **Where:** `CaveContain::onRemoving` and `recalcApparentControllingPlayer`.
- **What goes wrong:** a cave built while its network is occupied has no `m_originalTeam`. If the network empties through it, every cave of the index is defected to `nullptr`. `ActiveBody::attemptDamage` then dereferences the null player when a hit on or from such a cave names a source.
- **How the oracle handles it:** `gen_pkt264_cave.cpp` keeps the teamless caves and sends those hits without a source.

### `CaveSystem` crashes or spins on indices it was not built for

- Contain calls, queries and `onDie` on a cave whose index no cave has registered dereference a null tracker.
- `registerNewCave` with a negative index appends slots until memory runs out.
- Saving a system with an unused index below a used one throws.

### Shared passenger lists ignore "expose stealthed units"

`TunnelTracker::removeFromContain` takes `exposeStealthUnits` and never reads it, so leaving a tunnel or cave detects nobody. Mutating that argument in the port changes nothing, which is correct. `golden_pkt264_cave` forces stealthed passengers out and records that none is detected.

### Loading a live update module drops it from the sleepy heap

- **Where:** `UpdateModule::xfer` in load mode (`snapshot_load_bytes`).
- **What happens:** the module forgets its heap index. A later `processDestroyList` erases index -1 and segfaults.
- **How the oracle handles it:** `gen_pkt348_physics.cpp` saves `friend_getIndexInLogic()` before a load and restores it afterwards.

### `PhysicsBehavior` reads members it never assigns

- **Where:** `m_originalAllowBounce` (never written) and `m_turning` (not set by the constructor).
- **What happens:** both rely on the memory pool zeroing the block. So once a bounce dies out, `ALLOW_BOUNCE` is cleared for good, whatever the INI says.
- **Port contract:** both start at zero. `golden_pkt348_physics` covers it.

### A weapon's `FireSound` is not played by the logic

- **What happens:** firing a weapon does not reach `TheAudio`, so an audio spy sees nothing. `gen_pkt348_physics.cpp` counts fires through `WeaponTemplate::m_historicDamage` (`HistoricBonusCount` set far out of reach).
- **Related:** above frame 2^24, `frame + reloadTime` is computed in `float` and can round up, so a freshly loaded temporary weapon refuses to fire. The Rust `fire_weapon_template` stand-in does not model this, and that harness keeps its frames below 2^24.

### Host `std::cos` in a harness differs between optimisation levels

- **What happens:** constant folding gives other bits than the run-time call, so `-O0` and `-O2` outputs differ although the module is fine.
- **Rule:** build harness inputs with `WWMath::Cosf` / `Sinf`.
### A rider without `StealthUpdate` leaves locals uninitialised

`StealthUpdate::update` and `markAsDetected` declare `stealthDelay` (and `orderIdlesToAttack`) without a value and assign them only if the stealth owner is the object itself or has a `StealthUpdate`. With `UseRiderStealth` and a plain first passenger the result is whatever was in the register. The port uses the container's own data; no vector covers it.

### Disguise change dereferences physics and needs transition frames

- `changeVisualDisguise` calls `getPhysics()->resetDynamicPhysics()` unchecked, and `disguiseAsObject` does not require `DisguisesAsTeam`, so a harness must never disguise an object without `PhysicsBehavior`.
- With `DisguiseTransitionTime = 0` the disguise template is recorded but never applied; with `DisguiseRevealTransitionTime = 0` a disguised unit never loses `DISGUISED`. The port keeps both.

### The black market is only checked now and then

`allowedToStealth` looks for a black market only when `m_nextBlackMarketCheckFrame < now`; every other frame passes without one. The frame is not saved, and it advances from its old value rather than from `now`, so a module first checked late draws once per frame until it catches up.

### Loading into a live update module loses its scheduler index

`UpdateModule::xfer` sets `m_indexInLogic = -1` on load, expecting `GameLogic` to rebuild the heap. A harness that round-trips a registered module must restore the index, or the next `setWakeFrame` or destroy release-crashes with "sleepy update module illegal index".

### Going under construction destroys mines

`Object::setStatus` with `OBJECT_STATUS_UNDER_CONSTRUCTION` also detonates mines it finds. The port does not do this, so vectors with mines must not toggle it.

## Shared world fixture (`oracle_wp51_world.h`)

These are headless-setup gaps, not reference-code quirks. Each one surfaced as a segfault.

- **Neutral player without a default team.** `Player::init` leaves `m_defaultTeam` null, and `GarrisonContain::onObjectCreated` spawns its `InitialRoster` into `getControllingPlayer()->getDefaultTeam()`. `init_with_neutral_player()` now creates `TeamNeutral` and aborts if the player still has no default team.
- **UI messages.** Damaging a structure reaches `Radar::tryUnderAttackEvent`, then `InGameUI::message`, which needs `TheGlobalLanguageData`. `NullInGameUI` overrides the message calls as no-ops.

- **Shroud edges.** An object whose looking changes a cell for the local player reaches `TheDisplay`, which is null. A harness that moves looking objects between teams must first reveal the map to those players, each while another player is local (`gen_pkt264_cave.cpp`).

After changing the fixture, rebuild every oracle that uses the changed function and `cmp` its output with the committed JSON.

## Registering a module changes what existing templates do

A `Behavior = X` line in a test template is inert until `X` is registered. Once a wave registers it, the object gets a real module that can claim `contain_slot` and similar slots ahead of a test's stand-in. `WP-5.3-A` broke `golden_wp52_special_power` this way, and a targeted test run missed it. Run `cargo test --workspace --no-fail-fast` before committing any wave that registers modules.

### The warehouse jostle is a whole-number draw

- **Where:** `SupplyWarehouseDockUpdate::action`, `newPos.x += GameLogicRandomValue(-range, range)` with a `Real` range of 4.
- **What happens:** `GameLogicRandomValue` is the integer draw, so the docker moves by a whole number in -4..4. A real draw would give a different position and a different stream.
- **Guard:** `golden_pkt320_381_378_363_docks` compares the docker position and the seed CRC after every action.

### Dock queries that look at the wrong member

- **Where:** `DockUpdate::getDockPosition` and `getExitPosition`.
- **What happens:** both test `m_enterPosition` against zero, not the position they return.
- **Guard:** `golden_pkt320_381_378_363_docks` writes bone positions with a zero enter position and non-zero dock and exit positions.

### A repair keeps the rate of whoever started it

- **Where:** `RepairDockUpdate::action`.
- **What happens:** the healing per frame is fixed while `m_lastRepair` is zero, which is only restored when a docker is found at full health. A docker that leaves early passes its rate to the next one. `TimeForFullHeal = 0` gives a NaN or infinite rate that is saved.
- **Guard:** `golden_pkt320_381_378_363_docks`, scripted vectors.

### `SupplyCenterDockUpdate` needs the floating text switched off

- **What happens:** `action` ends in `InGameUI::addFloatingText`, which needs fonts and a display and crashes headless.
- **How the oracle handles it:** `TheGameLogic->setDrawIconUI(FALSE)` in `gen_pkt320_381_378_363_docks.cpp`.
