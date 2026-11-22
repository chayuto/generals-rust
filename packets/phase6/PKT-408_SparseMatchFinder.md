# Translation Packet `PKT-408`: `SparseMatchFinder`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-408`
- **Work Package:** `WP-6.1` (Phase 6)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Core Spine
- **Allowed Rust Output File:** `crates/zh_logic/src/player/sparse_match_finder.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (2):**
  - `Common/BitFlags.h`
  - `Common/STLTypedefs.h`
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

### `GeneralsMD/Code/GameEngine/Include/Common/SparseMatchFinder.h`
```cpp
// FILE: SparseMatchFinder.h /////////////////////////////////////////////////////////////////////////
// Author: Steven Johnson, March 2002
// Desc:
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/BitFlags.h"
#include "Common/STLTypedefs.h"

#if defined(RTS_DEBUG)
	#define SPARSEMATCH_DEBUG
#else
	#undef SPARSEMATCH_DEBUG
#endif

typedef UnsignedInt SparseMatchFinderFlags;
enum SparseMatchFinderFlags_ CPP_11(: SparseMatchFinderFlags)
{
	SparseMatchFinderFlags_NoCopy = 1<<0,
};

//-------------------------------------------------------------------------------------------------
template<class MATCHABLE, class BITSET, SparseMatchFinderFlags FLAGS = 0>
class SparseMatchFinder
{
private:

	//-------------------------------------------------------------------------------------------------
	// TYPEDEFS
	//-------------------------------------------------------------------------------------------------

	struct HashMapHelper
	{
		size_t operator()(const BITSET& p) const
		{
			/// @todo srj -- provide a better hash function for BITSET
			size_t result = 0;

			for (int i = 0; i < p.size(); ++i)
				if (p.test(i))
					result += (i + 1);

			return result;
		}

		Bool operator()(const BITSET& a, const BITSET& b) const
		{
			return (a == b);
		}
	};

	struct MapHelper
	{
		bool operator()(const BITSET& a, const BITSET& b) const
		{
			int i;
			if (a.size() < b.size()) {
				return true;
			}
			for (i = 0; i < a.size(); ++i) {
				bool aVal = a.test(i);
				bool bVal = b.test(i);
				if (aVal && bVal) continue;
				if (!aVal && !bVal) continue;
				if (!aVal) return true;
				return false;
			}
			return false; // all bits match.
		}
	};

	//-------------------------------------------------------------------------------------------------
	//typedef std::hash_map< BITSET, const MATCHABLE*, HashMapHelper, HashMapHelper > HashMatchMap;
	typedef std::map< const BITSET, const MATCHABLE*, MapHelper> MatchMap;

	//-------------------------------------------------------------------------------------------------
	// MEMBER VARS
	//-------------------------------------------------------------------------------------------------

	mutable MatchMap m_bestMatches;
	//mutable HashMatchMap m_bestHashMatches;

	//-------------------------------------------------------------------------------------------------
	// METHODS
	//-------------------------------------------------------------------------------------------------

	//-------------------------------------------------------------------------------------------------
	static Int countConditionIntersection(const BITSET& a, const BITSET& b)
	{
		return a.countIntersection(b);
	}

	//-------------------------------------------------------------------------------------------------
	static Int countConditionInverseIntersection(const BITSET& a, const BITSET& b)
	{
		return a.countInverseIntersection(b);
	}

	//-------------------------------------------------------------------------------------------------
	const MATCHABLE* findBestInfoSlow(const std::vector<MATCHABLE>& v, const BITSET& bits) const
	{
		const MATCHABLE* result = nullptr;
		Int bestYesMatch = 0;							// want to maximize this
		Int bestYesExtraneousBits = 999;	// want to minimize this

	#ifdef SPARSEMATCH_DEBUG
		Int numDupMatches = 0;
		AsciiString curBestMatchStr, dupMatchStr;
	#endif

		for (typename std::vector<MATCHABLE>::const_iterator it = v.begin(); it != v.end(); ++it)
		{
			for (Int i = it->getConditionsYesCount()-1; i >= 0; --i)
			{
				const BITSET& yesFlags = it->getNthConditionsYes(i);

				// the best match has the most "yes" matches and the smallest number of "no" matches.
				// if there are ties, then prefer the model with the smaller number of irrelevant 'yes' bits.
				// (example of why tiebreaker is necessary: if we want to match FRONTCRUSHED,
				// it would tie with both FRONTCRUSHED and FRONTCRUSHED|BACKCRUSHED, since they
				// both match a single YES bit.)
				Int yesMatch = countConditionIntersection(bits, yesFlags);
				Int yesExtraneousBits = countConditionInverseIntersection(bits, yesFlags);

	#ifdef SPARSEMATCH_DEBUG
				if (yesMatch == bestYesMatch &&
						yesExtraneousBits == bestYesExtraneousBits)
				{
					++numDupMatches;
					dupMatchStr = it->getDescription();
				}
	#endif

				if ((yesMatch > bestYesMatch) ||
						(yesMatch >= bestYesMatch && yesExtraneousBits < bestYesExtraneousBits))
				{
					result = &(*it);
					bestYesMatch = yesMatch;
					bestYesExtraneousBits = yesExtraneousBits;
	#ifdef SPARSEMATCH_DEBUG
					numDupMatches = 0;
					curBestMatchStr = it->getDescription();
	#endif
				}
			}

		}

#ifdef SPARSEMATCH_DEBUG
		if (numDupMatches > 0)
		{
			AsciiString curConditionStr;
			bits.buildDescription(&curConditionStr);
			DEBUG_CRASH(("ambiguous model match in findBestInfoSlow \n\nbetween \n(%s)\n<and>\n(%s)\n\n(%d extra matches found)\n\ncurrent bits are (\n%s)",
					curBestMatchStr.str(),
					dupMatchStr.str(),
					numDupMatches,
					curConditionStr.str()));
		}
#endif

		return result;
	}

public:


	//-------------------------------------------------------------------------------------------------
	SparseMatchFinder() {}
	SparseMatchFinder(const SparseMatchFinder& other)
	{
		*this = other;
	}

	//-------------------------------------------------------------------------------------------------
	SparseMatchFinder& operator=(const SparseMatchFinder& other)
	{
		if constexpr ((FLAGS & SparseMatchFinderFlags_NoCopy) == 0)
		{
			if (this != &other)
			{
				m_bestMatches = other.m_bestMatches;
			}
		}
		return *this;
	}

	//-------------------------------------------------------------------------------------------------
	void clear()
	{
		m_bestMatches.clear();
	}

	//-------------------------------------------------------------------------------------------------
	const MATCHABLE* findBestInfo(const std::vector<MATCHABLE>& v, const BITSET& bits) const
	{
		typename MatchMap::const_iterator it = m_bestMatches.find(bits);

		const MATCHABLE *first = nullptr;
		if (it != m_bestMatches.end())
		{
			first = (*it).second;
		}
		if (first != nullptr) {
			return first;
		}

		const MATCHABLE* info = findBestInfoSlow(v, bits);

		DEBUG_ASSERTCRASH(info != nullptr, ("no suitable match for criteria was found!"));
		if (info != nullptr) {
			m_bestMatches[bits] = info;
		}

		return info;
	}

};
```
