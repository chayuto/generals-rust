# Translation Packet `PKT-002`: `RandomValue`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-002`
- **Work Package:** `WP-1.2a` (Phase 1)
- **Complexity & Assigned Tier:** Class `C1` -> `tier:1a-lite`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 1: C-ABI Leaf
- **Allowed Rust Output File:** `crates/zh_common/src/random_value.rs` *(Do not modify files outside this boundary)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (5):**
  - `Common/Debug.h`
  - `Common/RandomValue.h`
  - `Common/crc.h`
  - `GameLogic/GameLogic.h`
  - `PreRTS.h`
- **`Snapshot::crc` / `Snapshot::xfer` Calls:** None in module.
- **PRNG Calls (10 — preserve call order & `lo >= hi` no-advance guard):**
  - `GetGameAudioRandomValue( int lo, int hi, const char *file, int line ) { if (lo >= hi)`
  - `GetGameAudioRandomValueReal( Real lo, Real hi, const char *file, int line ) { if (lo >= hi)`
  - `GetGameClientRandomValue( int lo, int hi, const char *file, int line ) { if (lo >= hi)`
  - `GetGameClientRandomValueReal( Real lo, Real hi, const char *file, int line ) { if (lo >= hi)`
  - `GetGameLogicRandomValue( int lo, int hi, const char *file, int line )`
  - `GetGameLogicRandomValueReal( Real lo, Real hi, const char *file, int line )`
  - `GetGameLogicRandomValueUnchanged( int lo, int hi, const char *file, int line ) { #if RETAIL_COMPATIBLE_CRC return GetGameLogicRandomValue(lo, hi, file, line)`
  - `GetGameLogicRandomValueRealUnchanged( Real lo, Real hi, const char *file, int line ) { #if RETAIL_COMPATIBLE_CRC return GetGameLogicRandomValueReal(lo, hi, file, line)`
  - `GameClientRandomValueReal( m_low, m_high )`
  - `GameLogicRandomValueReal( m_low, m_high )`
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if RETAIL_COMPATIBLE_CRC`

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

### `Core/GameEngine/Include/Common/RandomValue.h`
```cpp
// RandomValue.h
// Random number generation system
// Author: Michael S. Booth, January 1998

#pragma once

extern void InitRandom();
extern void InitRandom( UnsignedInt seed );
extern UnsignedInt GetGameLogicRandomSeed();   ///< Get the seed (used for replays)
extern UnsignedInt GetGameLogicRandomSeedCRC();///< Get the seed (used for CRCs)

//--------------------------------------------------------------------------------------------------------------
```

### `Core/GameEngine/Source/Common/RandomValue.cpp`
```cpp
// RandomValue.cpp
// Pseudo-random number generators
// Author: Michael S. Booth, January 1998

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/RandomValue.h"
#include "Common/crc.h"
#include "Common/Debug.h"
#include "GameLogic/GameLogic.h"

#undef DEBUG_RANDOM_AUDIO
#undef DEBUG_RANDOM_CLIENT
#undef DEBUG_RANDOM_LOGIC
#undef DETERMINISTIC

//#define DEBUG_RANDOM_AUDIO
//#define DEBUG_RANDOM_CLIENT
//#define DEBUG_RANDOM_LOGIC
//#define DETERMINISTIC       // to allow repetition for debugging

static const Real theMultFactor = 1.0f / static_cast<float>(UINT_MAX);

// Initial seed values.
static UnsignedInt theGameAudioSeed[6] =
{
	0xf22d0e56L, 0x883126e9L, 0xc624dd2fL, 0x702c49cL, 0x9e353f7dL, 0x6fdf3b64L
};

static UnsignedInt theGameClientSeed[6] =
{
	0xf22d0e56L, 0x883126e9L, 0xc624dd2fL, 0x702c49cL, 0x9e353f7dL, 0x6fdf3b64L
};

static UnsignedInt theGameLogicSeed[6] =
{
	0xf22d0e56L, 0x883126e9L, 0xc624dd2fL, 0x702c49cL, 0x9e353f7dL, 0x6fdf3b64L
};

static UnsignedInt theGameLogicBaseSeed = 0;

UnsignedInt GetGameLogicRandomSeed()
{
	return theGameLogicBaseSeed;
}

UnsignedInt GetGameLogicRandomSeedCRC()
{
	CRC c;
	c.computeCRC(theGameLogicSeed, sizeof(theGameLogicSeed));
	return c.get();
}

static void seedRandom(UnsignedInt SEED, UnsignedInt (&seed)[6])
{
	UnsignedInt ax;

	ax = SEED;                      /* mov     eax,SEED                     */
	ax += 0xf22d0e56;               /* add     eax,0f22d0e56h               */
	seed[0] = ax;                   /* mov     seed,eax                     */
	ax += 0x883126e9 - 0xf22d0e56;  /* add     eax,0883126e9h-0f22d0e56h    */
	seed[1] = ax;                   /* mov     seed+4,eax                   */
	ax += 0xc624dd2f - 0x883126e9;  /* add     eax,0c624dd2fh-0883126e9h    */
	seed[2] = ax;                   /* mov     seed+8,eax                   */
	ax += 0x0702c49c - 0xc624dd2f;  /* add     eax,00702c49ch-0c624dd2fh    */
	seed[3] = ax;                   /* mov     seed+12,eax                  */
	ax += 0x9e353f7d - 0x0702c49c;  /* add     eax,09e353f7dh-00702c49ch    */
	seed[4] = ax;                   /* mov     seed+16,eax                  */
	ax += 0x6fdf3b64 - 0x9e353f7d;  /* add     eax,06fdf3b64h-09e353f7dh    */
	seed[5] = ax;                   /* mov     seed+20,eax                  */
}

void InitRandom()
{
#ifdef DETERMINISTIC
	// needs to be the same every time
	seedRandom(0, theGameAudioSeed);
	seedRandom(0, theGameClientSeed);
	seedRandom(0, theGameLogicSeed);
	theGameLogicBaseSeed = 0;
#else
	const time_t seconds = time( nullptr );

	seedRandom(seconds, theGameAudioSeed);
	seedRandom(seconds, theGameClientSeed);
	seedRandom(seconds, theGameLogicSeed);
	theGameLogicBaseSeed = seconds;
#endif
}

void InitRandom( UnsignedInt seed )
{
#ifdef DETERMINISTIC
	seed = 0;
#endif

	seedRandom(seed, theGameAudioSeed);
	seedRandom(seed, theGameClientSeed);
	seedRandom(seed, theGameLogicSeed);
	theGameLogicBaseSeed = seed;

#ifdef DEBUG_RANDOM_LOGIC
	DEBUG_LOG(("InitRandom %08lx", seed));
#endif
}

// Add with carry. SUM is replaced with A + B + C, C is replaced with 1 if there was a carry, 0 if there wasn't.
// A carry occurred if the sum is less than one of the inputs. This is addition, so carry can never be more than one.
#define ADC(SUM, A, B, C) SUM = (A) + (B) + (C); C = ((SUM < (A)) || (SUM < (B)))

static UnsignedInt randomValue(UnsignedInt (&seed)[6])
{
	UnsignedInt ax;
	UnsignedInt c = 0;

	ADC(ax, seed[5], seed[4], c);   /*  mov     ax,seed+20  */
	/*  add     ax,seed+16  */
	seed[4] = ax;                   /*  mov     seed+8,ax   */

	ADC(ax, ax, seed[3], c);        /*  adc     ax,seed+12  */
	seed[3] = ax;                   /*  mov     seed+12,ax  */

	ADC(ax, ax, seed[2], c);        /*  adc     ax,seed+8   */
	seed[2] = ax;                   /*  mov     seed+8,ax   */

	ADC(ax, ax, seed[1], c);        /*  adc     ax,seed+4   */
	seed[1] = ax;                   /*  mov     seed+4,ax   */

	ADC(ax, ax, seed[0], c);        /*  adc     ax,seed+0   */
	seed[0] = ax;                   /*  mov     seed+0,ax   */

	/* Increment seed array, bubbling up the carries. */
	if (!++seed[5])
	{
		if (!++seed[4])
		{
			if (!++seed[3])
			{
				if (!++seed[2])
				{
					if (!++seed[1])
					{
						++seed[0];
						++ax;
					}
				}
			}
		}
	}

	return ax;
}

//
// It is necessary to separate the GameClient and GameLogic usage of random
// values to ensure that the GameLogic remains deterministic, regardless
// of the effects displayed on the GameClient.
//

//
// Integer random value
//
Int GetGameAudioRandomValue( int lo, int hi, const char *file, int line )
{
	if (lo >= hi)
		return hi;

	const UnsignedInt delta = hi - lo + 1;
	const Int rval = ((Int)(randomValue(theGameAudioSeed) % delta)) + lo;

#ifdef DEBUG_RANDOM_AUDIO
	DEBUG_LOG(( "%d: GetGameAudioRandomValue = %d (%d - %d), %s line %d",
		TheGameLogic->getFrame(), rval, lo, hi, file, line ));
#endif

	DEBUG_ASSERTCRASH(rval >= lo && rval <= hi, ("Bad random val"));
	return rval;
}

//
// Real valued random value
//
Real GetGameAudioRandomValueReal( Real lo, Real hi, const char *file, int line )
{
	if (lo >= hi)
		return hi;

	const Real delta = hi - lo;
	const Real rval = ((Real)(randomValue(theGameAudioSeed)) * theMultFactor) * delta + lo;

#ifdef DEBUG_RANDOM_AUDIO
	DEBUG_LOG(( "%d: GetGameAudioRandomValueReal = %f, %s line %d",
		TheGameLogic->getFrame(), rval, file, line ));
#endif

	DEBUG_ASSERTCRASH(rval >= lo && rval <= hi, ("Bad random val"));
	return rval;
}

//
// Integer random value
//
Int GetGameClientRandomValue( int lo, int hi, const char *file, int line )
{
	if (lo >= hi)
		return hi;

	const UnsignedInt delta = hi - lo + 1;
	const Int rval = ((Int)(randomValue(theGameClientSeed) % delta)) + lo;

#ifdef DEBUG_RANDOM_CLIENT
	DEBUG_LOG(( "%d: GetGameClientRandomValue = %d (%d - %d), %s line %d",
		TheGameLogic ? TheGameLogic->getFrame() : -1, rval, lo, hi, file, line ));
#endif

	DEBUG_ASSERTCRASH(rval >= lo && rval <= hi, ("Bad random val"));
	return rval;
}

//
// Real valued random value
//
Real GetGameClientRandomValueReal( Real lo, Real hi, const char *file, int line )
{
	if (lo >= hi)
		return hi;

	const Real delta = hi - lo;
	const Real rval = ((Real)(randomValue(theGameClientSeed)) * theMultFactor) * delta + lo;

#ifdef DEBUG_RANDOM_CLIENT
	DEBUG_LOG(( "%d: GetGameClientRandomValueReal = %f, %s line %d",
		TheGameLogic->getFrame(), rval, file, line ));
#endif

	DEBUG_ASSERTCRASH(rval >= lo && rval <= hi, ("Bad random val"));
	return rval;
}

//
// Integer random value
//
Int GetGameLogicRandomValue( int lo, int hi, const char *file, int line )
{
#if RETAIL_COMPATIBLE_CRC
	const UnsignedInt delta = hi - lo + 1;
	if (delta == 0)
		return hi;
#else
	if (lo >= hi)
		return hi;

	const UnsignedInt delta = hi - lo + 1;
#endif

	const Int rval = ((Int)(randomValue(theGameLogicSeed) % delta)) + lo;

#ifdef DEBUG_RANDOM_LOGIC
	DEBUG_LOG(( "%d: GetGameLogicRandomValue = %d (%d - %d), %s line %d",
		TheGameLogic->getFrame(), rval, lo, hi, file, line ));
#endif

	DEBUG_ASSERTCRASH(rval >= lo && rval <= hi, ("Bad random val"));
	return rval;
}

//
// Real valued random value
//
Real GetGameLogicRandomValueReal( Real lo, Real hi, const char *file, int line )
{
#if RETAIL_COMPATIBLE_CRC
	const Real delta = hi - lo;
	if (delta <= 0.0f)
		return hi;
#else
	if (lo >= hi)
		return hi;

	const Real delta = hi - lo;
#endif

	const Real rval = ((Real)(randomValue(theGameLogicSeed)) * theMultFactor) * delta + lo;

#ifdef DEBUG_RANDOM_LOGIC
	DEBUG_LOG(( "%d: GetGameLogicRandomValueReal = %f, %s line %d",
		TheGameLogic->getFrame(), rval, file, line ));
#endif

	DEBUG_ASSERTCRASH(rval >= lo && rval <= hi, ("Bad random val"));
	return rval;
}

//
// TheSuperHackers @info This function does not change the seed values with retail compatibility disabled.
// Consecutive calls always return the same value for the same combination of min / max values, assuming the seed values haven't changed in between.
// The intended use case for this function are randomized values that are desirable to be synchronized across clients,
// but should not result in a mismatch if they aren't synchronized; e.g. for scripted audio events.
//
Int GetGameLogicRandomValueUnchanged( int lo, int hi, const char *file, int line )
{
#if RETAIL_COMPATIBLE_CRC
	return GetGameLogicRandomValue(lo, hi, file, line);
#endif

	if (lo >= hi)
		return hi;

	UnsignedInt seed[ARRAY_SIZE(theGameLogicSeed)];
	memcpy(&seed[0], &theGameLogicSeed[0], sizeof(seed));

	const UnsignedInt delta = hi - lo + 1;
	const Int rval = ((Int)(randomValue(seed) % delta)) + lo;

#ifdef DEBUG_RANDOM_LOGIC
	DEBUG_LOG(( "%d: GetGameLogicRandomValueUnchanged = %d (%d - %d), %s line %d",
		TheGameLogic->getFrame(), rval, lo, hi, file, line ));
#endif

	DEBUG_ASSERTCRASH(rval >= lo && rval <= hi, ("Bad random val"));
	return rval;
}

//
// TheSuperHackers @info This function does not change the seed values with retail compatibility disabled.
// Consecutive calls always return the same value for the same combination of min / max values, assuming the seed values haven't changed in between.
// The intended use case for this function are randomized values that are desirable to be synchronized across clients,
// but should not result in a mismatch if they aren't synchronized; e.g. for scripted audio events.
//
Real GetGameLogicRandomValueRealUnchanged( Real lo, Real hi, const char *file, int line )
{
#if RETAIL_COMPATIBLE_CRC
	return GetGameLogicRandomValueReal(lo, hi, file, line);
#endif

	if (lo >= hi)
		return hi;

	UnsignedInt seed[ARRAY_SIZE(theGameLogicSeed)];
	memcpy(&seed[0], &theGameLogicSeed[0], sizeof(seed));

	const Real delta = hi - lo;
	const Real rval = ((Real)(randomValue(seed)) * theMultFactor) * delta + lo;

#ifdef DEBUG_RANDOM_LOGIC
	DEBUG_LOG(( "%d: GetGameLogicRandomValueRealUnchanged = %f, %s line %d",
		TheGameLogic->getFrame(), rval, file, line ));
#endif

	DEBUG_ASSERTCRASH(rval >= lo && rval <= hi, ("Bad random val"));
	return rval;
}

//--------------------------------------------------------------------------------------------------------------
// GameClientRandomVariable
//

const char *const GameClientRandomVariable::DistributionTypeNames[] =
{
	"CONSTANT", "UNIFORM", "GAUSSIAN", "TRIANGULAR", "LOW_BIAS", "HIGH_BIAS", nullptr
};
static_assert(ARRAY_SIZE(GameClientRandomVariable::DistributionTypeNames) == GameClientRandomVariable::DISTRIBUTION_COUNT + 1, "Incorrect array size");

/**
	define the range of random values, and the distribution of values
*/
void GameClientRandomVariable::setRange( Real low, Real high, DistributionType type )
{
	DEBUG_ASSERTCRASH(!(m_type == CONSTANT && m_low != m_high), ("CONSTANT GameClientRandomVariables should have low == high"));
	m_low = low;
	m_high = high;
	m_type = type;
}

/**
 * Return a value from the random distribution
 */
Real GameClientRandomVariable::getValue() const
{
	switch( m_type )
	{
		case CONSTANT:
			DEBUG_ASSERTLOG(m_low == m_high, ("m_low != m_high for a CONSTANT GameClientRandomVariable"));
			if (m_low == m_high) {
				return m_low;
			}
			FALLTHROUGH;

		case UNIFORM:
			return GameClientRandomValueReal( m_low, m_high );

		default:
			/// @todo fill in support for nonuniform GameClientRandomVariables.
			DEBUG_CRASH(("unsupported DistributionType in GameClientRandomVariable::getValue"));
			return 0.0f;
	}
}

//--------------------------------------------------------------------------------------------------------------
// GameLogicRandomVariable
//

const char *const GameLogicRandomVariable::DistributionTypeNames[] =
{
	"CONSTANT", "UNIFORM", "GAUSSIAN", "TRIANGULAR", "LOW_BIAS", "HIGH_BIAS", nullptr
};
static_assert(ARRAY_SIZE(GameLogicRandomVariable::DistributionTypeNames) == GameLogicRandomVariable::DISTRIBUTION_COUNT + 1, "Incorrect array size");

/**
	define the range of random values, and the distribution of values
*/
void GameLogicRandomVariable::setRange( Real low, Real high, DistributionType type )
{
	DEBUG_ASSERTCRASH(!(m_type == CONSTANT && m_low != m_high), ("CONSTANT GameLogicRandomVariables should have low == high"));
	m_low = low;
	m_high = high;
	m_type = type;
}

/**
 * Return a value from the random distribution
 */
Real GameLogicRandomVariable::getValue() const
{
	switch( m_type )
	{
		case CONSTANT:
			DEBUG_ASSERTLOG(m_low == m_high, ("m_low != m_high for a CONSTANT GameLogicRandomVariable"));
			if (m_low == m_high) {
				return m_low;
			}
			FALLTHROUGH;

		case UNIFORM:
			return GameLogicRandomValueReal( m_low, m_high );

		default:
			/// @todo fill in support for nonuniform GameLogicRandomVariables.
			DEBUG_CRASH(("unsupported DistributionType in GameLogicRandomVariable::getValue"));
			return 0.0f;
	}
}
```
