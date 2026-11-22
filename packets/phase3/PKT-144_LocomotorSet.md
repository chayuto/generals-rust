# Translation Packet `PKT-144`: `LocomotorSet`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-144`
- **Work Package:** `WP-3.3` (Phase 3)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 2: Handle Component
- **Allowed Rust Output File:** `crates/zh_logic/src/combat/locomotor_set.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (4):**
  - `Common/GameCommon.h`
  - `Common/STLTypedefs.h`
  - `Common/Snapshot.h`
  - `GameLogic/Locomotor.h`
- **`Snapshot::crc` / `Snapshot::xfer` Calls:** None in module.

---

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

---

## 3. Reference C++ Source (`fbraz3/GeneralsX` at `84a7a20`)

### `GeneralsMD/Code/GameEngine/Include/GameLogic/LocomotorSet.h`
```cpp
// FILE: LocomotorSet.h /////////////////////////////////////////////////////////////////////////////////
// Author: Steven Johnson, Feb 2002
// Desc: Locomotor Descriptions
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// no, please do NOT include this.
//#include "GameLogic/Locomotor.h"
#include "Common/GameCommon.h"
#include "Common/STLTypedefs.h"
#include "Common/Snapshot.h"

class Locomotor;
class LocomotorTemplate;

//-------------------------------------------------------------------------------------------------
//
// Note: these values are saved in save files, so you MUST NOT REMOVE OR CHANGE
// existing values!
//
enum LocomotorSurfaceType CPP_11(: Int)
{
	LOCOMOTORSURFACE_GROUND			= (1 << 0),									///< clear, unobstructed ground
	LOCOMOTORSURFACE_WATER			= (1 << 1),									///< water area
	LOCOMOTORSURFACE_CLIFF			= (1 << 2),									///< steep altitude change
	LOCOMOTORSURFACE_AIR				= (1 << 3),									///< airborne
	LOCOMOTORSURFACE_RUBBLE			= (1 << 4),									///< building rubble

	LOCOMOTOR_SURFACE_TYPE_END // keep after the last named flag
};

typedef Int LocomotorSurfaceTypeMask;

const LocomotorSurfaceTypeMask NO_SURFACES = (LocomotorSurfaceTypeMask)0x0000;
const LocomotorSurfaceTypeMask ALL_SURFACES = (LocomotorSurfaceTypeMask)0xffff;

#ifdef DEFINE_SURFACECATEGORY_NAMES
static const char *const TheLocomotorSurfaceTypeNames[] =
{
	"GROUND",
	"WATER",
	"CLIFF",
	"AIR",
	"RUBBLE",

	nullptr
};
static_assert(equalsEnumBitFlagsCount(ARRAY_SIZE(TheLocomotorSurfaceTypeNames) - 1, LOCOMOTOR_SURFACE_TYPE_END), "Incorrect array size");
#endif

//-------------------------------------------------------------------------------------------------
typedef std::vector< Locomotor* > LocomotorVector;

//-------------------------------------------------------------------------------------------------
class LocomotorSet : public Snapshot
{
private:
	LocomotorVector						m_locomotors;
	LocomotorSurfaceTypeMask	m_validLocomotorSurfaces;
	Bool											m_downhillOnly;

	LocomotorSet(const LocomotorSet& that);
	LocomotorSet& operator=(const LocomotorSet& that);

protected:
	// snapshot methods
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

public:

	LocomotorSet();
	~LocomotorSet();

	void clear();

	void addLocomotor(const LocomotorTemplate* lt);

	Locomotor* findLocomotor(LocomotorSurfaceTypeMask t);

	void xferSelfAndCurLocoPtr(Xfer *xfer, Locomotor** loco);

	LocomotorSurfaceTypeMask getValidSurfaces() const { return m_validLocomotorSurfaces; }
	Bool isDownhillOnly() const { return m_downhillOnly; };

};
```
