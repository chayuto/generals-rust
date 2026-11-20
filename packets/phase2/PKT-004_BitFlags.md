# Translation Packet `PKT-004`: `BitFlags`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-004`
- **Work Package:** `WP-2.1` (Phase 2)
- **Complexity & Assigned Tier:** Class `C1` -> `tier:1a-lite`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 1: C-ABI Leaf
- **Allowed Rust Output File:** `crates/zh_common/src/xfer/bit_flags.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_common/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (6):**
  - `Common/BitFlags.h`
  - `Common/BitFlagsIO.h`
  - `Common/ModelState.h`
  - `Common/STLTypedefs.h`
  - `GameLogic/ArmorSet.h`
  - `PreRTS.h`
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

### `GeneralsMD/Code/GameEngine/Include/Common/BitFlags.h`
```cpp
// FILE: BitFlags.h /////////////////////////////////////////////////////////////////////////
// Author: Steven Johnson, March 2002
// Desc:
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/STLTypedefs.h"

class INI;
class Xfer;
class AsciiString;

//-------------------------------------------------------------------------------------------------
/*
	BitFlags is a wrapper class that exists primarily because of a flaw in std::bitset<>.
	Although quite useful, it has horribly non-useful constructor, which (1) don't let
	us initialize stuff in useful ways, and (2) provide a default constructor that implicitly
	converts ints into bitsets in a "wrong" way (ie, it treats the int as a mask, not an index).
	So we wrap to correct this, but leave the bitset "exposed" so that we can use all the non-ctor
	functions on it directly (since it doesn't overload operator= to do the "wrong" thing, strangely enough)
*/
template <size_t NUMBITS, typename TAG>
class BitFlags
{
private:
	std::bitset<NUMBITS>				m_bits;

public:
	CPP_11(static constexpr size_t NumBits = NUMBITS);
	static const char* const s_bitNameList[];

	/*
		just a little syntactic sugar so that there is no "foo = 0" compatible constructor
	*/
	enum BogusInitType { kInit };
	enum InitSetAllType { kInitSetAll };

	BitFlags()
	{
	}

	// This constructor sets all bits to 1
	BitFlags(InitSetAllType)
	{
		m_bits.set();
	}

	BitFlags(UnsignedInt value)
		: m_bits(static_cast<unsigned long>(value))
	{
	}

	// TheSuperHackers @todo Replace with variadic template

	BitFlags(BogusInitType k, Int idx1)
	{
		m_bits.set(idx1);
	}

	BitFlags(BogusInitType k, Int idx1, Int idx2)
	{
		m_bits.set(idx1);
		m_bits.set(idx2);
	}

	BitFlags(BogusInitType k, Int idx1, Int idx2, Int idx3)
	{
		m_bits.set(idx1);
		m_bits.set(idx2);
		m_bits.set(idx3);
	}

	BitFlags(BogusInitType k, Int idx1, Int idx2, Int idx3, Int idx4)
	{
		m_bits.set(idx1);
		m_bits.set(idx2);
		m_bits.set(idx3);
		m_bits.set(idx4);
	}

	BitFlags(BogusInitType k, Int idx1, Int idx2, Int idx3, Int idx4, Int idx5)
	{
		m_bits.set(idx1);
		m_bits.set(idx2);
		m_bits.set(idx3);
		m_bits.set(idx4);
		m_bits.set(idx5);
	}

	// Set all given indices in the array.
	BitFlags(BogusInitType, const Int* idxs, Int count)
	{
		const Int* idx = idxs;
		const Int* end = idxs + count;
		for (; idx < end; ++idx)
		{
			m_bits.set(*idx);
		}
	}

	Bool operator==(const BitFlags& that) const
	{
		return m_bits == that.m_bits;
	}

	Bool operator!=(const BitFlags& that) const
	{
		return m_bits != that.m_bits;
	}

	void set(Int i, Int val = 1)
	{
		m_bits.set(i, val);
	}

	Bool test(Int i) const
	{
		return m_bits.test(i);
	}

	//Tests for any bits that are set in both.
	Bool testForAny( const BitFlags& that ) const
	{
		return (m_bits & that.m_bits).any();
	}

	//All argument bits must be set in our bits too in order to return TRUE
	Bool testForAll( const BitFlags& that ) const
	{
		return (m_bits & that.m_bits) == that.m_bits;
	}

	//None of the argument bits must be set in our bits in order to return TRUE
	Bool testForNone( const BitFlags& that ) const
	{
		return (m_bits & that.m_bits).none();
	}

	Int size() const
	{
		return m_bits.size();
	}

	Int count() const
	{
		return m_bits.count();
	}

	Bool any() const
	{
		return m_bits.any();
	}

	void flip()
	{
		m_bits.flip();
	}

	void clear()
	{
		m_bits.reset();
	}

	Int countIntersection(const BitFlags& that) const
	{
		return (m_bits & that.m_bits).count();
	}

	Int countInverseIntersection(const BitFlags& that) const
	{
		return (~m_bits & that.m_bits).count();
	}

	Bool anyIntersectionWith(const BitFlags& that) const
	{
		return (m_bits & that.m_bits).any();
	}

	void clear(const BitFlags& that)
	{
		m_bits &= ~that.m_bits;
	}

	void set(const BitFlags& that)
	{
		m_bits |= that.m_bits;
	}

	void clearAndSet(const BitFlags& flagsToClear, const BitFlags& flagsToSet)
	{
		clear(flagsToClear);
		set(flagsToSet);
	}

	Bool testSetAndClear(const BitFlags& mustBeSet, const BitFlags& mustBeClear) const
	{
		return testForNone(mustBeClear) && testForAll(mustBeSet);
	}

	// TheSuperHackers @info Function for rare use cases where we must access the flags as an integer.
	// Not using to_ulong because that can throw. Truncates all bits above 32.
	UnsignedInt toUnsignedInt() const noexcept
	{
		UnsignedInt val = 0;
		const UnsignedInt count = min(m_bits.size(), sizeof(val) * 8);
		for (UnsignedInt i = 0; i < count; ++i)
			val |= m_bits.test(i) * (1u << i);
		return val;
	}

  static const char* const* getBitNames()
  {
    return s_bitNameList;
  }

  static const char* getNameFromSingleBit(Int i)
  {
    return (i >= 0 && i < NUMBITS) ? s_bitNameList[i] : nullptr;
  }

  static Int getSingleBitFromName(const char* token)
  {
    Int i = 0;
	  for(const char* const* name = s_bitNameList; *name; ++name, ++i )
	  {
		  if( stricmp( *name, token ) == 0 )
		  {
        return i;
		  }
	  }
		return -1;
  }

  const char* getBitNameIfSet(Int i) const
  {
    return test(i) ? s_bitNameList[i] : nullptr;
  }

  Bool setBitByName(const char* token)
  {
    Int i = getSingleBitFromName(token);
		if (i >= 0)
		{
      set(i);
			return true;
		}
		else
		{
	    return false;
		}
  }

	void parse(INI* ini, AsciiString* str);
	void parseSingleBit(INI* ini, AsciiString* str);
	void xfer(Xfer* xfer);
	static void parseFromINI(INI* ini, void* /*instance*/, void *store, const void* /*userData*/); ///< Returns a BitFlag
	static void parseSingleBitFromINI(INI* ini, void* /*instance*/, void *store, const void* /*userData*/); ///< Returns an int, the Index of the one bit

	void buildDescription( AsciiString* str ) const
	{
		if ( str == nullptr )
			return;//sanity

		for( Int i = 0; i < size(); ++i )
		{
			const char* bitName = getBitNameIfSet(i);

			if (bitName != nullptr)
			{
				str->concat( bitName );
				str->concat( ",\n");
			}
		}
	}

	// TheSuperHackers @feature Stubbjax 23/01/2026 Add function for outputting debug data.
	AsciiString toHexString() const
	{
		constexpr const int numChunks = (NUMBITS + 63) / 64;
		char chunkBuf[32]; // Enough for 16 hex digits + null terminator
		AsciiString result;
		bool printedAny = false;

		for (int chunk = numChunks - 1; chunk >= 0; --chunk)
		{
			UnsignedInt64 val = 0;
			for (int bit = 0; bit < 64 && (chunk * 64 + bit) < NUMBITS; ++bit)
			{
				if (m_bits.test(chunk * 64 + bit))
					val |= (UnsignedInt64)(1) << bit;
			}

			if (val != 0 || chunk == 0 || printedAny)
			{
#ifdef _MSC_VER
				if (printedAny)
					snprintf(chunkBuf, sizeof(chunkBuf), "%016I64X", val);
				else
					snprintf(chunkBuf, sizeof(chunkBuf), "%I64X", val);
#else
				if (printedAny)
					snprintf(chunkBuf, sizeof(chunkBuf), "%016llX", (unsigned long long)val);
				else
					snprintf(chunkBuf, sizeof(chunkBuf), "%llX", (unsigned long long)val);
#endif

				result.concat(chunkBuf);
				printedAny = true;
			}
		}

		if (!printedAny)
			result = "0";

		return result;
	}
};
```

### `GeneralsMD/Code/GameEngine/Source/Common/BitFlags.cpp`
```cpp
// FILE: BitFlags.cpp ///////////////////////////////////////////////////////////
//
// Used to set detail levels of various game systems.
//  Steven Johnson, Sept 2002
//
//
///////////////////////////////////////////////////////////////////////////////

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/BitFlags.h"
#include "Common/BitFlagsIO.h"
#include "Common/ModelState.h"
#include "GameLogic/ArmorSet.h"

template<>
const char* const ModelConditionFlags::s_bitNameList[] =
{
	"TOPPLED",
	"FRONTCRUSHED",
	"BACKCRUSHED",
	"DAMAGED",
	"REALLYDAMAGED",
	"RUBBLE",
	"SPECIAL_DAMAGED",
	"NIGHT",
	"SNOW",
	"PARACHUTING",
	"GARRISONED",
	"ENEMYNEAR",
	"WEAPONSET_VETERAN",
	"WEAPONSET_ELITE",
	"WEAPONSET_HERO",
	"WEAPONSET_CRATEUPGRADE_ONE",
	"WEAPONSET_CRATEUPGRADE_TWO",
	"WEAPONSET_PLAYER_UPGRADE",
	"DOOR_1_OPENING",
	"DOOR_1_CLOSING",
	"DOOR_1_WAITING_OPEN",
	"DOOR_1_WAITING_TO_CLOSE",
	"DOOR_2_OPENING",
	"DOOR_2_CLOSING",
	"DOOR_2_WAITING_OPEN",
	"DOOR_2_WAITING_TO_CLOSE",
	"DOOR_3_OPENING",
	"DOOR_3_CLOSING",
	"DOOR_3_WAITING_OPEN",
	"DOOR_3_WAITING_TO_CLOSE",
	"DOOR_4_OPENING",
	"DOOR_4_CLOSING",
	"DOOR_4_WAITING_OPEN",
	"DOOR_4_WAITING_TO_CLOSE",
	"ATTACKING",
	"PREATTACK_A",
	"FIRING_A",
	"BETWEEN_FIRING_SHOTS_A",
	"RELOADING_A",
	"PREATTACK_B",
	"FIRING_B",
	"BETWEEN_FIRING_SHOTS_B",
	"RELOADING_B",
	"PREATTACK_C",
	"FIRING_C",
	"BETWEEN_FIRING_SHOTS_C",
	"RELOADING_C",
	"TURRET_ROTATE",
	"POST_COLLAPSE",
	"MOVING",
	"DYING",
	"AWAITING_CONSTRUCTION",
	"PARTIALLY_CONSTRUCTED",
	"ACTIVELY_BEING_CONSTRUCTED",
	"PRONE",
	"FREEFALL",
	"ACTIVELY_CONSTRUCTING",
	"CONSTRUCTION_COMPLETE",
	"RADAR_EXTENDING",
	"RADAR_UPGRADED",
	"PANICKING",	// yes, it's spelled with a "k". look it up.
	"AFLAME",
	"SMOLDERING",
	"BURNED",
	"DOCKING",
	"DOCKING_BEGINNING",
	"DOCKING_ACTIVE",
	"DOCKING_ENDING",
	"CARRYING",
	"FLOODED",
	"LOADED",
	"JETAFTERBURNER",
	"JETEXHAUST",
	"PACKING",
	"UNPACKING",
	"DEPLOYED",
	"OVER_WATER",
	"POWER_PLANT_UPGRADED",
	"CLIMBING",
	"SOLD",
#ifdef ALLOW_SURRENDER
	"SURRENDER",
#endif
	"RAPPELLING",
	"ARMED",
	"POWER_PLANT_UPGRADING",

	"SPECIAL_CHEERING",

	"CONTINUOUS_FIRE_SLOW",
	"CONTINUOUS_FIRE_MEAN",
	"CONTINUOUS_FIRE_FAST",

	"RAISING_FLAG",
	"CAPTURED",

	"EXPLODED_FLAILING",
	"EXPLODED_BOUNCING",
	"SPLATTED",

	"USING_WEAPON_A",
	"USING_WEAPON_B",
	"USING_WEAPON_C",

	"PREORDER",

	"CENTER_TO_LEFT",
	"LEFT_TO_CENTER",
	"CENTER_TO_RIGHT",
	"RIGHT_TO_CENTER",

	"RIDER1",	//Kris: Added these for different combat-bike riders, but feel free to use these for anything.
	"RIDER2",
	"RIDER3",
	"RIDER4",
	"RIDER5",
	"RIDER6",
	"RIDER7",
	"RIDER8",

  "STUNNED_FLAILING", // Daniel Teh's idea, added by Lorenzen, 5/28/03
	"STUNNED",
	"SECOND_LIFE",
	"JAMMED",
	"ARMORSET_CRATEUPGRADE_ONE",
	"ARMORSET_CRATEUPGRADE_TWO",

	"USER_1",
	"USER_2",

	"DISGUISED",

	nullptr
};
static_assert(ARRAY_SIZE(ModelConditionFlags::s_bitNameList) == ModelConditionFlags::NumBits + 1, "Incorrect array size");

template<>
const char* const ArmorSetFlags::s_bitNameList[] =
{
	"VETERAN",
	"ELITE",
	"HERO",
	"PLAYER_UPGRADE",
	"WEAK_VERSUS_BASEDEFENSES",
	"SECOND_LIFE",
	"CRATE_UPGRADE_ONE",
	"CRATE_UPGRADE_TWO",

	nullptr
};
static_assert(ARRAY_SIZE(ArmorSetFlags::s_bitNameList) == ArmorSetFlags::NumBits + 1, "Incorrect array size");
```
