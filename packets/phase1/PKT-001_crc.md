# Translation Packet `PKT-001`: `crc`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-001`
- **Work Package:** `WP-1.2a` (Phase 1)
- **Complexity & Assigned Tier:** Class `C1` -> `tier:1a-lite`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 1: C-ABI Leaf
- **Allowed Rust Output File:** `crates/zh_common/src/crc.rs` *(Do not modify files outside this boundary)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (3):**
  - `Common/Debug.h`
  - `Common/crc.h`
  - `PreRTS.h`
- **`Snapshot::crc` / `Snapshot::xfer` Calls:** None in module.
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if (defined(_MSC_VER) && _MSC_VER < 1300) && RETAIL_COMPATIBLE_CRC`

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

### `Core/GameEngine/Include/Common/crc.h`
```cpp
// CRC.h ///////////////////////////////////////////////////////////////
// A class encapsulating CRC calculation
// Author: Matthew D. Campbell, October 2001

#pragma once

#ifdef RTS_DEBUG

class CRC
{
public:
	CRC() { crc = 0; }

	void computeCRC( const void *buf, Int len );		///< Compute the CRC for a buffer, added into current CRC
	void clear() { crc = 0; }									///< Clears the CRC to 0
//	UnsignedInt get() { return htonl(crc); }	///< Get the combined CRC
	UnsignedInt get();

#if (defined(_MSC_VER) && _MSC_VER < 1300) && RETAIL_COMPATIBLE_CRC
  void set( UnsignedInt v )
  {
    crc = v;
  }
#endif

private:
	void addCRC( UnsignedByte val );									///< CRC a 4-byte block

	UnsignedInt crc;
};

#else

// optimized inline only version
class CRC
{
public:
	CRC() { crc=0; }

  /// Compute the CRC for a buffer, added into current CRC
	__forceinline void computeCRC( const void *buf, Int len )
  {
    if (!buf||len<1)
      return;

#if !(defined(_MSC_VER) && _MSC_VER < 1300)
    // C++ version left in for reference purposes
	  for (UnsignedByte *uintPtr=(UnsignedByte *)buf;len>0;len--,uintPtr++)
    {
    	int hibit;
    	if (crc & 0x80000000)
      {
		    hibit = 1;
	    }
      else
      {
		    hibit = 0;
	    }

	    crc <<= 1;
	    crc += *uintPtr;
	    crc += hibit;
    }
#else
    // ASM version, verified by comparing resulting data with C++ version data
    unsigned *crcPtr=&crc;
    __asm
    {
      mov esi,[buf]
      mov ecx,[len]
      dec ecx
      mov edi,[crcPtr]
      mov ebx,dword ptr [edi]
      xor eax,eax
    lp:
      mov al,byte ptr [esi]
      shl ebx,1
      inc esi
      adc ebx,eax
      dec ecx
      jns lp
      mov dword ptr [edi],ebx
    };
#endif
  }

  /// Clears the CRC to 0
	void clear()
  {
    crc = 0;
  }

  ///< Get the combined CRC
	UnsignedInt get() const
  {
    return crc;
  }

#if (defined(_MSC_VER) && _MSC_VER < 1300) && RETAIL_COMPATIBLE_CRC
  void set( UnsignedInt v )
  {
    crc = v;
  }
#endif

private:
	UnsignedInt crc;
};

#endif
```

### `Core/GameEngine/Source/Common/crc.cpp`
```cpp
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/crc.h"
#include "Common/Debug.h"

#ifdef RTS_DEBUG

void CRC::addCRC( UnsignedByte val )
{
	int hibit;

	//cout << "\t\t" << hex << val;
//	val = htonl(val);
	//cout << " / " << hex << val <<endl;


	if (crc & 0x80000000) {
		hibit = 1;
	} else {
		hibit = 0;
	}

	crc <<= 1;
	crc += val;
	crc += hibit;

	//cout << hex << (*crc) <<endl;
}


void CRC::computeCRC( const void *buf, Int len )
{
	if (!buf || len < 1)
	{
		return;
	}

	//crc = 0;

	UnsignedByte *uintPtr = (UnsignedByte *)buf;

	for (int i=0 ; i<len ; i++) {
		addCRC (*(uintPtr++));
	}
	//crc = htonl(crc);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
UnsignedInt CRC::get()
{

	UnsignedInt tcrc = crc;
	return tcrc;

}

#endif
```
