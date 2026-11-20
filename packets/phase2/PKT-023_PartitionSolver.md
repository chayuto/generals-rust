# Translation Packet `PKT-023`: `PartitionSolver`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-023`
- **Work Package:** `WP-2.2` (Phase 2)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 1: C-ABI Leaf
- **Allowed Rust Output File:** `crates/zh_common/src/math/partition_solver.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_common/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (2):**
  - `Common/PartitionSolver.h`
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

### `GeneralsMD/Code/GameEngine/Include/Common/PartitionSolver.h`
```cpp
// FILE: Statistics.h
/*---------------------------------------------------------------------------*/
/* EA Pacific                                                                */
/* Confidential Information	                                                 */
/* Copyright (C) 2001 - All Rights Reserved                                  */
/* DO NOT DISTRIBUTE                                                         */
/*---------------------------------------------------------------------------*/
/* Project:    RTS3                                                          */
/* File name:  PartitionSolver.h                                             */
/* Created:    John K. McDonald, Jr., 4/2/2002                               */
/* Desc:       General purpose partition problem-solver											 */
/* Revision History:                                                         */
/*		4/12/2002 : Initial creation                                           */
/*---------------------------------------------------------------------------*/

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////

// DEFINES ////////////////////////////////////////////////////////////////////
// TYPE DEFINES ///////////////////////////////////////////////////////////////
// FORWARD DECLARATIONS ///////////////////////////////////////////////////////

typedef std::pair<ObjectID, UnsignedInt> PairObjectIDAndUInt;
typedef std::pair<ObjectID, ObjectID> PairObjectID;

// the first unsigned int is the identifying nugget, while the second is the size of the nugget
typedef std::vector<PairObjectIDAndUInt> EntriesVec;

// the first unsigned int is the identifying nugget, while the second is the size of the hole we get to fill
typedef std::vector<PairObjectIDAndUInt> SpacesVec;

// the first ObjectID is the id of the entry, while the second is the id of the hole
typedef std::vector<PairObjectID> SolutionVec;

enum SolutionType CPP_11(: Int)
{
	PREFER_FAST_SOLUTION = 0,
	PREFER_CORRECT_SOLUTION = 0x7FFFFFFF
};

class PartitionSolver
{
	protected:
		SolutionType m_howToSolve;
		EntriesVec m_data;
		SpacesVec m_spacesForData;

		SolutionVec m_currentSolution;
		UnsignedInt m_currentSolutionLeftovers;
		SolutionVec m_bestSolution;

	public:
		PartitionSolver(const EntriesVec& elements, const SpacesVec& spaces, SolutionType solveHow);

		// Solve could potentially take a LONG TIME (as in NEVER complete). This problem is NP-complete.
		void solve();
		const SolutionVec& getSolution() const;
};
```

### `GeneralsMD/Code/GameEngine/Source/Common/PartitionSolver.cpp`
```cpp
// FILE: PartitionSolver.cpp //////////////////////////////////////////////////////////////////////
/*---------------------------------------------------------------------------*/
/* EA Pacific                                                                */
/* Confidential Information	                                                 */
/* Copyright (C) 2001 - All Rights Reserved                                  */
/* DO NOT DISTRIBUTE                                                         */
/*---------------------------------------------------------------------------*/
/* Project:    RTS3                                                          */
/* File name:  PartitionSolver.cpp                                           */
/* Created:    John K. McDonald, Jr., 4/2/2002                               */
/* Desc:       This contains a general-purpose Partition solver							 */
/* Revision History:                                                         */
/*		4/12/2002 : Initial creation                                           */
/*---------------------------------------------------------------------------*/
/**************************************************************************************************
Some info about partitioning problems:

	This problem is contained in a very interesting class of problems known as NP complete. The
	basic problem is that there is no way to tell whether you have an optimal solution or not.
	Worst case, you try out every possible solution and still don't find the optimal solution:
	this takes 2^n time to find, where N is the number of elements you are attempting to place.
	For this reason, a value near PREFER_FAST_SOLUTION should almost always be chosen. We will use
	a flat multiply to determine how many solutions to attempt before giving up and returning our
	best attempt. If you want more info, this site contains info on the problem:
	http://odysseus.nat.uni-magdeburg.de/~mertens/npp/index.shtml
**************************************************************************************************/

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/PartitionSolver.h"

static Bool greater_than(PairObjectIDAndUInt a, PairObjectIDAndUInt b)
{
	return a.second > b.second;
}

PartitionSolver::PartitionSolver(const EntriesVec& elements, const SpacesVec& spaces, SolutionType solveHow)
{
	m_data = elements;
	m_spacesForData = spaces;
	m_howToSolve = solveHow;
	m_currentSolutionLeftovers = 0;
}

void PartitionSolver::solve()
{
	m_bestSolution.clear();
	m_currentSolution.clear();
	m_currentSolutionLeftovers = 0x7fffffff;

	Int minSizeForAllData = 0;
	Int slotsAllotted = 0;
	size_t i, j;

	// first, determine whether there is an actual solution, or we're going to have to fudge it.
	for (i = 0; i < m_data.size(); ++i) {
		minSizeForAllData += m_data[i].second;
	}

	for (i = 0; i < m_spacesForData.size(); ++i) {
		slotsAllotted += m_spacesForData[i].second;
	}

	// we want to attempt to place the largest things first. This allows us to throw
	// out whole classes of solutions

	std::sort(m_data.begin(), m_data.end(), greater_than);

	// Also make the largest partition first.
	std::sort(m_spacesForData.begin(), m_spacesForData.end(), greater_than);

	// work in our temporary vector.
	SpacesVec spacesStillAvailable = m_spacesForData;

	if (m_howToSolve == PREFER_FAST_SOLUTION)
	{
		// we prefer the fast, but not necessarily correct solution
		// simply start placing the stuff. Skip things you can't place.
		for (i = 0; i < m_data.size(); ++i)
		{
			for (j = 0; j < spacesStillAvailable.size(); ++j)
			{
				if (m_data[i].second <= spacesStillAvailable[j].second)
				{
					spacesStillAvailable[j].second -= m_data[i].second;
					m_bestSolution.push_back(std::make_pair(m_data[i].first, spacesStillAvailable[j].first));
					break;
				}
			}
		}
	} else {
		DEBUG_CRASH(("PREFER_CORRECT_SOLUTION @todo impl"));
	}
}

const SolutionVec& PartitionSolver::getSolution() const
{
	return m_bestSolution;
}
```
