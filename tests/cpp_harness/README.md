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

## Shared world fixture (`oracle_wp51_world.h`)

These are headless-setup gaps, not reference-code quirks. Each one surfaced as a segfault.

- **Neutral player without a default team.** `Player::init` leaves `m_defaultTeam` null, and `GarrisonContain::onObjectCreated` spawns its `InitialRoster` into `getControllingPlayer()->getDefaultTeam()`. `init_with_neutral_player()` now creates `TeamNeutral` and aborts if the player still has no default team.
- **UI messages.** Damaging a structure reaches `Radar::tryUnderAttackEvent`, then `InGameUI::message`, which needs `TheGlobalLanguageData`. `NullInGameUI` overrides the message calls as no-ops.

After changing the fixture, rebuild every oracle that uses the changed function and `cmp` its output with the committed JSON.

## Registering a module changes what existing templates do

A `Behavior = X` line in a test template is inert until `X` is registered. Once a wave registers it, the object gets a real module that can claim `contain_slot` and similar slots ahead of a test's stand-in. `WP-5.3-A` broke `golden_wp52_special_power` this way, and a targeted test run missed it. Run `cargo test --workspace --no-fail-fast` before committing any wave that registers modules.
