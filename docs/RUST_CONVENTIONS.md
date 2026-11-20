# Rust Determinism & Translation Conventions (`generals-rust`)

Every line of simulation logic translated from C++ (`Core/GameEngine` and `GeneralsMD/Code/GameEngine`) to Rust (`crates/zh_common`, `crates/zh_logic`, `crates/zh_ffi`) must produce bit-identical state checksums (`XferCRC` / `GameLogic::getCRC()`) across all frames of all replays.

This document specifies the exact C++-to-Rust mapping rules enforced by `scripts/lint_determinism.py` (**Gate V1**) and verified by the differential harness (**Gate V2**) and replay suite (**Gate V3/V4**).

---

<!-- BEGIN WORKER CHEATSHEET -->
## Compact Worker Cheat Sheet (Embedded in All Translation Packets)

### 1. Primitive Type Mapping
| C++ Type | Rust Type | Size | Notes |
|---|---|---:|---|
| `Int` | `i32` | 4 | Two's-complement 32-bit signed |
| `UnsignedInt` | `u32` | 4 | 32-bit unsigned |
| `Short` | `i16` | 2 | 16-bit signed |
| `UnsignedShort` | `u16` | 2 | 16-bit unsigned |
| `Byte` / `char` | `i8` | 1 | Signed 8-bit (`char` is signed on x86/macOS) |
| `UnsignedByte` | `u8` | 1 | Unsigned 8-bit |
| `Bool` | `u8` (`Bool`) | 1 | `FALSE = 0`, `TRUE = 1` (1 byte in `Xfer`!) |
| `Real` | `f32` | 4 | IEEE-754 binary32, round-to-nearest-even, no FMA |
| `ObjectID` / `TeamID` / `DrawableID` | `ObjectId(pub i32)` etc. | 4 | `INVALID_ID = 0` |
| `NameKeyType` | `NameKeyType(pub i32)` | 4 | `NAMEKEY_INVALID = 0` |
| `UpdateSleepTime` | `UpdateSleepTime(pub u32)` | 4 | `NONE = 1`, `FOREVER = 0x3fff_ffff` |

### 2. Ten Non-Negotiable Determinism Rules
1. **Wrapping Integer Arithmetic:** Never use bare `+`, `-`, `*`, `+=`, `-=`, `*=` on `i32`/`u32` where C++ can overflow or wrap. Use `.wrapping_add()`, `.wrapping_sub()`, `.wrapping_mul()`, `.wrapping_neg()`.
2. **Unfused IEEE-754 `f32` Arithmetic:** Never call `f32::mul_add` (FMA). Keep exact C++ parenthesization and left-to-right operator grouping: `(a * b) * c + d` must not be rewritten as `a * (b * c) + d`.
3. **Deterministic Transcendentals via `GameMath`:** Never call standard `f32::sin`, `cos`, `tan`, `asin`, `acos`, `atan2`, `sqrt`, `floor`, `ceil`, `powf`, `exp`, or `ln`. Always call `zh_common::gamemath::{gm_sinf, gm_cosf, gm_tanf, gm_asinf, gm_acosf, gm_atan2f, gm_sqrtf, gm_floorf, gm_ceilf, gm_powf, gm_expf, gm_logf}`.
4. **Mixed `Int * Real` Promotion & Float-to-Int Truncation:**
   - In C++, `Int a; Real s; a *= s;` evaluates `a = static_cast<Int>(static_cast<Real>(a) * s)`.
   - In Rust, always write: `a = ((a as f32) * s) as i32;` (or `zh_common::math::cpp_mul_i32_f32(a, s)`).
   - For `REAL_TO_INT_FLOOR(x)` and `REAL_TO_INT_CEIL(x)`, call `zh_common::math::{real_to_int_floor, real_to_int_ceil}` (which route to `gm_floorf(x) as i32` / `gm_ceilf(x) as i32` in default deterministic mode, and `fast_float_floor`/`ceil` under `#[cfg(feature = "retail-crc")]`).
5. **No `HashMap` or `HashSet`:** Never import `std::collections::{HashMap, HashSet}`. Always use `std::collections::{BTreeMap, BTreeSet}` keyed by a deterministic integer ID (`ObjectId`, `NameKeyType`, `TeamId`, `u32`), never by raw pointer address.
6. **No `BinaryHeap` for Sleepy Updates:** Use `zh_logic::SleepyUpdateHeap` (strict `>` comparator, left-child tie-breaking, in-place root sifting, reverse-heap-index removal).
7. **Evaluation Order of Side-Effecting Arguments:** When a C++ function call has 2+ arguments that call `GameLogicRandomValue*` or mutate state, evaluate them into explicit `let arg0 = ...; let arg1 = ...;` bindings in left-to-right order before the call.
8. **Exact PRNG Call Parity:** `GameLogicRandomValue(lo, hi)` and `GameLogicRandomValueReal(lo, hi)` return `hi` **without advancing the seed** when `lo >= hi` (under default `RETAIL_COMPATIBLE_CRC == 0`). Never reorder, hoist, or short-circuit a branch that calls `GameLogicRandomValue*`.
9. **Exact `Xfer` & `XferCRC` Field Order and Widths:**
   - Serialize fields in the **exact** C++ `crc(Xfer*)` and `xfer(Xfer*)` order.
   - Never serialize `usize` or `isize`; cast lengths to `u16`, `u32`, or `i32` matching the C++ local variable type (`UnsignedShort`, `UnsignedInt`, `Int`).
   - Remember `Bool` is `1 byte` (`u8`), and enum values serialized via `xferUser(&e, sizeof(Enum))` are `4 bytes` (`i32`).
10. **Preserve `#if !RETAIL_COMPATIBLE_CRC` Guards:** Translate `#if RETAIL_COMPATIBLE_CRC` / `#if !RETAIL_COMPATIBLE_CRC` blocks using `#[cfg(feature = "retail-crc")]` and `#[cfg(not(feature = "retail-crc"))]`.
<!-- END WORKER CHEATSHEET -->

---

## 3. Detailed C++-to-Rust Translation Patterns (Before & After)

### 3.1 Wrapping Integer Arithmetic & Carry Chains (`RandomValue.cpp`)

**C++ Reference:**
```cpp
#define ADC(SUM, A, B, C) SUM = (A) + (B) + (C); C = ((SUM < (A)) || (SUM < (B)))
ADC(ax, seed[5], seed[4], c);
seed[4] = ax;
```

**Rust Translation:**
```rust
#[inline(always)]
fn adc(a: u32, b: u32, c: &mut u32) -> u32 {
    let sum = a.wrapping_add(b).wrapping_add(*c);
    *c = u32::from(sum < a || sum < b);
    sum
}

let mut c: u32 = 0;
let mut ax = adc(seed[5], seed[4], &mut c);
seed[4] = ax;
```

---

### 3.2 Mixed `Int` and `Real` Compound Assignment (`ExperienceTracker.cpp`)

**C++ Reference:**
```cpp
if (m_experienceSink != INVALID_ID) {
    Object *sinkPointer = TheGameLogic->findObjectByID(m_experienceSink);
    if (sinkPointer) {
        sinkPointer->getExperienceTracker()->addExperiencePoints(
            experienceGain * m_experienceScalar, canScaleForBonus);
        return;
    }
}
Int amountToGain = experienceGain;
if (canScaleForBonus)
    amountToGain *= m_experienceScalar;
m_currentExperience += amountToGain;
```

**Rust Translation:**
```rust
if self.experience_sink != ObjectId::INVALID {
    if let Some(sink_slot) = world.find_object_slot(self.experience_sink) {
        let scaled_gain = ((experience_gain as f32) * self.experience_scalar) as i32;
        world.add_object_experience_points(sink_slot, scaled_gain, can_scale_for_bonus);
        return;
    }
}
let mut amount_to_gain: i32 = experience_gain;
if can_scale_for_bonus {
    amount_to_gain = ((amount_to_gain as f32) * self.experience_scalar) as i32;
}
self.current_experience = self.current_experience.wrapping_add(amount_to_gain);
```

---

### 3.3 `Snapshot::crc` and `Snapshot::xfer` Implementation (`ExperienceTracker.cpp`)

**C++ Reference:**
```cpp
void ExperienceTracker::crc( Xfer *xfer )
{
    xfer->xferInt( &m_currentExperience );
    xfer->xferUser( &m_currentLevel, sizeof( VeterancyLevel ) );
#if !RETAIL_COMPATIBLE_CRC
    xfer->xferBool(&m_isTrainable);
#endif
}
```

**Rust Translation:**
```rust
impl Snapshot for ExperienceTracker {
    fn crc(&self, xfer: &mut dyn Xfer) {
        let mut exp = self.current_experience;
        xfer.xfer_int(&mut exp);
        let mut lvl = self.current_level as i32;
        xfer.xfer_int(&mut lvl);
        #[cfg(not(feature = "retail-crc"))]
        {
            let mut trainable = u8::from(self.is_trainable);
            xfer.xfer_bool(&mut trainable);
        }
    }
    // ...
}
```

---

### 3.4 Evaluating Multiple RNG Calls in Function Arguments

**C++ Reference:**
```cpp
Coord3D offset;
offset.set(
    GameLogicRandomValueReal(-radius, radius),
    GameLogicRandomValueReal(-radius, radius),
    0.0f
);
```

**Rust Translation (Explicit Left-to-Right `let` Bindings):**
```rust
let dx = world.rng.logic_random_real(-radius, radius);
let dy = world.rng.logic_random_real(-radius, radius);
let offset = Coord3D::new(dx, dy, 0.0f32);
```
