# Translation Packet `PKT-395`: `Override`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-395`
- **Work Package:** `WP-5.5` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:hybrid-ci`
- **FFI Seam Pattern:** Core Spine
- **Allowed Rust Output File:** `crates/zh_logic/src/object/override.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (1):**
  - `Common/Overridable.h`
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

### `GeneralsMD/Code/GameEngine/Include/Common/Override.h`
```cpp
// Override.h /////////////////////////////////////////////////////////////////////////////////////
// Electronic Arts Pacific
// Do Not Distribute

#pragma once

#include "Common/Overridable.h"

/*
	An OVERRIDE is a replacement for a pointer of its contained type, ie, rather than containing
	a LocomotorTemplate*, you would contain an OVERRIDE<LocomotorTemplate>.

	OVERRIDE pretends in all ways (dereference via *, -> and casting to type*) to be a type*, so
	there should be very little code that needs to be rewritten to work with these.

	In order to make something overridable, these are the steps:
		1) Make the desired class derive from Overridable.
		2) Make the container class contain an instance of OVERRIDE<Type>
		3) Make the newOverride function (wherever an override is new'd) request the overridables lastOverride,
			to ensure that no leaks are created.

		See LocomotorTemplate for an example.
*/

template <class T> class OVERRIDE
{
	public:
		// Provide useful constructors to go from a T* to an OVERRIDE<T>
		OVERRIDE(const T *overridable = nullptr);
		// Copy constructor
		OVERRIDE(OVERRIDE<T> &overridable);
		// Operator= for copying from another OVERRIDE and T*
		__inline OVERRIDE &operator=( const OVERRIDE<T>& other );
		__inline OVERRIDE &operator=( const T* overridable );

		// these are the methods which we can use to access data in a pointer. (Dereference*, ->, and cast
		// to T*). They are all overloaded to recurse to the lowest override and use that.
		__inline const T *operator->() const;	// overload const ->
		__inline const T *operator*() const;		// overload const *(dereference operator)
		__inline operator const T*() const;	// overload casting to (const T*)

		// this is useful in case you want to get the pointer that this object is actually looking at.
		__inline const T *getNonOverloadedPointer() const;

	private:
		// Because OVERRIDE is meant to live on the object and not in the store, it currently contains
		// a constant pointer. We could change this if it seems weird.
		const T *m_overridable;
};

//-------------------------------------------------------------------------------------------------
template <class T>
OVERRIDE<T>::OVERRIDE(const T *overridable)
{
	m_overridable = overridable;
}

//-------------------------------------------------------------------------------------------------
template <class T>
OVERRIDE<T>::OVERRIDE(OVERRIDE<T> &overridable)
{
	m_overridable = overridable.m_overridable;
}

//-------------------------------------------------------------------------------------------------
template <class T>
OVERRIDE<T> &OVERRIDE<T>::operator=( const OVERRIDE<T>& other )
{
	m_overridable = other.m_overridable;
	return *this;
}

//-------------------------------------------------------------------------------------------------
template <class T>
OVERRIDE<T> &OVERRIDE<T>::operator=(const T* overridable)
{
	m_overridable = overridable;
	return *this;
}

//-------------------------------------------------------------------------------------------------
template <class T>
const T *OVERRIDE<T>::operator->() const
{
	if (!m_overridable)
		return nullptr;
	return (T*) m_overridable->getFinalOverride();
}

//-------------------------------------------------------------------------------------------------
template <class T>
const T *OVERRIDE<T>::operator*() const
{
	if (!m_overridable)
		return nullptr;
	return (T*) m_overridable->getFinalOverride();
}

//-------------------------------------------------------------------------------------------------
template <class T>
const T *OVERRIDE<T>::getNonOverloadedPointer() const
{
	return (T*) m_overridable;
}

//-------------------------------------------------------------------------------------------------
template <class T>
OVERRIDE<T>::operator const T*() const
{
	return operator*();
}
```
