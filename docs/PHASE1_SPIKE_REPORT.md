# Phase 1 Translation Spike Telemetry & Go/No-Go Report (`[WP-1.3]`)

**Milestone:** Phase 1 — Translation Spike (Go/No-Go)  
**Tracking Issues:** `#8` (`WP-1.1`), `#9` (`WP-1.2a`), `#10` (`WP-1.2b`), `#11` (`WP-1.3`)  
**Reference C++ Commit:** `fbraz3/GeneralsX` @ `84a7a20769aa5749f388b1f49bd542c79b3c33e8`  
**Decision:** **GO FOR PHASE 2** (`100%` first-pass compile rate, `100%` bit-exact across `3,000` Golden Vectors and `3,000` in-process C++/Rust FFI differential seeds on Linux, macOS, and Windows).

---

## 1. Executive Summary

Phase 1 validated our multi-tier C++-to-Rust translation pipeline, our zero-external-dependency Cargo workspace architecture (`zh_common`, `zh_logic`, `zh_ffi`), our CMake static-library FFI seam into `GeneralsXZH`, and our 4-Gate Verification & Validation (V&V) funnel across all three target operating systems (`ubuntu-latest` x86_64, `macos-15` aarch64, and `windows-latest` x86_64 MSVC).

Three representative modules spanning `Common` and `GameLogic` were sliced by the **Tier 0 (AST Slicer)** (`scripts/prep_packet.py`), paired with **3,000 C++ Golden Test Vectors** generated directly from the unmodified C++ reference implementation, and translated by isolated **Tier 1A (Lite Worker)** agents in a single turn with zero codebase search:

1. **`PKT-001` (`crc.cpp` / `crc.h`):** Engine-wide rotate-left-add `CRC` accumulator (`crates/zh_common/src/crc.rs`).
2. **`PKT-002` (`RandomValue.cpp` / `RandomValue.h`):** 6-word additive-carry PRNG (`GameLogicRandomValue`, `GameLogicRandomValueReal`, `GameClientRandomValue`, `GameAudioRandomValue`) and all 7 `GameLogicRandomVariable::DistributionType` distributions (`CONSTANT`, `UNIFORM`, `GAUSSIAN`, `TRIANGULAR`, `LOW_BIAS`, `HIGH_BIAS`) (`crates/zh_common/src/random_value.rs`).
3. **`PKT-003` (`ExperienceTracker.cpp` / `ExperienceTracker.h`):** Unit veterancy state machine (`VeterancyLevel`, `m_currentLevel`, `m_currentExperience`, `m_experienceSink`, `m_experienceScalar`, `addExperiencePoints`, `setVeterancyLevel`, `setMinVeterancyLevel`, `gainExpForLevel`, `canGainExpForLevel`) and binary `Snapshot` (`XferSave`, `XferLoad`, `XferCRC`) (`crates/zh_logic/src/object/experience_tracker.rs`).

---

## 2. Spike Telemetry Matrix

| Metric | `PKT-001` (`crc`) | `PKT-002` (`RandomValue`) | `PKT-003` (`ExperienceTracker`) | Phase 1 Total / Average |
| :--- | :---: | :---: | :---: | :---: |
| **Assigned Agent Tier** | `Tier 1A (Lite Worker)` | `Tier 1A (Lite Worker)` | `Tier 1A (Lite Worker)` | `100% Tier 1A` |
| **Tier 0 Packet Size (Bytes / Lines)** | `7,841 B` (`236` lines) | `19,180 B` (`535` lines) | `17,654 B` (`413` lines) | `44,675 B` (`1,184` lines) |
| **Tier 0 Packet Input Tokens (est.)** | `~1,960` tokens | `~4,795` tokens | `~4,413` tokens | `~3,723` tokens / packet |
| **Rust Output Size (Bytes / Lines)** | `1,339 B` (`53` lines) | `10,884 B` (`361` lines) | `9,402 B` (`271` lines) | `21,625 B` (`685` lines) |
| **Rust Output Tokens (est.)** | `~335` tokens | `~2,720` tokens | `~2,350` tokens | `~1,802` tokens / packet |
| **Codebase Search Tool Calls by Worker** | `0` | `0` | `0` | **`0` (Zero-Search Verified)** |
| **First-Pass `cargo check` Compile Rate** | **Pass (Retry 0)** | **Pass (Retry 0)** | **Pass (Retry 0)** | **`100%` (`3 / 3`)** |
| **Gate V1 Static Determinism Violations** | `0` | `0` | `0` | **`0` Violations** |
| **Gate V2-Golden Vector Parity (`1,000` seeds)** | `1,000 / 1,000` (`100%`) | `1,000 / 1,000` (`100%`) | `1,000 / 1,000` (`100%`) | **`3,000 / 3,000` (`100%`)** |
| **Gate V2-FFI In-Process C++/Rust Parity** | `1,000 / 1,000` (`100%`) | `1,000 / 1,000` (`100%`) | `1,000 / 1,000` (`100%`) | **`3,000 / 3,000` (`100%`)** |
| **Tier 2 / Tier 3 Post-Translation Fixes** | `0` | `0` | `0` | **`0` Escalations** |

---

## 3. Cross-Platform 3-OS CI Matrix Verification

Both **Gate V1 + Gate V2-Golden** (`cargo test --workspace`) and **Gate V2-FFI** (`tests/cpp_harness/gen_and_verify_phase1.cpp` linked directly against `libzh_ffi.a` / `zh_ffi.lib`) were executed across all 3 operating systems on public GitHub Actions runners:

| Platform / Runner | Architecture & Toolchain | Gate V1 + V2-Golden (`3,000` vectors) | Gate V2-FFI (`3,000` live C++ vs. Rust seeds) | Total CI Job Wall Time |
| :--- | :--- | :---: | :---: | :---: |
| **`ubuntu-latest`** | Linux `x86_64` (`gcc` / `rustc`) | **PASS** (`0` diffs) | **PASS** (`0` diffs) | **`21s` – `29s`** |
| **`macos-15`** | macOS `aarch64` (`clang` / `rustc`) | **PASS** (`0` diffs) | **PASS** (`0` diffs) | **`24s` – `36s`** |
| **`windows-latest`** | Windows `x86_64` (`MSVC cl.exe` / `rustc`) | **PASS** (`0` diffs) | **PASS** (`0` diffs) | **`51s` – `59s`** |

*Note on CI Budget Efficiency:* Because the Cargo workspace has **zero external crates.io dependencies**, `cargo test --workspace` compiles from scratch and verifies thousands of Golden Test Vectors in **`< 1.5 seconds`** locally and under **`30 seconds`** on cold Linux CI runners.

---

## 4. Key Architectural & Determinism Findings from Phase 1

1. **C++ Mixed-Type Compound Assignment (`Int *= Real`):**
   - In `ExperienceTracker::addExperiencePoints` (`ExperienceTracker.cpp:178`), C++ executes `amountToGain *= m_experienceScalar;` where `amountToGain` is `Int` (`i32`) and `m_experienceScalar` is `Real` (`f32`).
   - Standard C++ promotes `amountToGain` to `float` (which can round `i32` values with `>24` significant bits to the nearest even `f32`), multiplies in IEEE-754 `binary32`, and truncates back to `i32`.
   - We codified `zh_common::math::cpp_mul_i32_f32(a: i32, s: f32) -> i32` (`((a as f32) * s) as i32`) and documented this rule in `docs/RUST_CONVENTIONS.md`.
2. **`INT_TO_REAL` Macro Semantics (`BaseMacros.h:80`):**
   - `#define INT_TO_REAL(x) ((Real)(x))` is a direct `(Real)` cast with **no** integer intermediate cast. When applied to a `Real` expression (such as in `BezierSegment::getApproximateLength`), it preserves fractional bits.
3. **`XferCRC` 32-Bit Word & Trailing-Byte Asymmetry (`XferCRC.cpp:101-164`):**
   - `XferCRC::xferImplementation` processes 4-byte chunks via `addCRC(uint32_t)` (which swaps endianness when `RETAIL_COMPATIBLE_CRC` is enabled), whereas trailing `1..3` bytes (`dataSize & 3`) are packed little-endian (`c[0] + (c[1] << 8) + (c[2] << 16)`) and rotated/added into `m_crc` **without** `htobe(val)`.
   - Our `zh_common::xfer::XferCrc` implementation and `PKT-003` golden vectors verify both 4-byte words and 1-byte/2-byte trailing fields (`XferVersion`, `Bool`, `VeterancyLevel`).
4. **SunPro `fdlibm` Pure-Rust Portability (`GameMath`):**
   - Inspection of `vendor/GeneralsX/build/macos-vulkan/_deps/gamemath-src/float/` confirmed that `GameMath` (`gm_sqrtf`, `gm_floorf`, `gm_ceilf`, `gm_sinf`, `gm_cosf`, `gm_tanf`, `gm_asinf`, `gm_acosf`, `gm_atan2f`) is pure integer-bit-manipulation `fdlibm` C code. Translating `GameMath` directly into pure Rust in `crates/zh_common/src/math/` eliminates C FFI call overhead while guaranteeing 100% bit-exact IEEE-754 parity across x86_64 and ARM64.

---

## 5. Phase 2 Go/No-Go Decision

All Phase 1 exit criteria for Track A (`Gate V1`, `Gate V2-Golden`, `Gate V2-FFI` across 3 OSes, `< 8k` token packet slicing, and `100%` Tier 1A first-pass compilation) have been met with zero escalations.

**Recommendation:** **PROCEED (GO)** with Phase 2 (`zh_common` Foundation, Math, Serialization & INI) and parallel `env:ephemeral-web` worker packet execution.
