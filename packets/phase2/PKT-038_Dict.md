# Translation Packet `PKT-038`: `Dict`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-038`
- **Work Package:** `WP-2.3` (Phase 2)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 1: C-ABI Leaf
- **Allowed Rust Output File:** `crates/zh_common/src/core/dict.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_common/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (5):**
  - `Common/Dict.h`
  - `Common/Errors.h`
  - `Common/GameMemory.h`
  - `Common/NameKeyGenerator.h`
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

### `Core/GameEngine/Include/Common/Dict.h`
```cpp
// FILE: Dict.h
//-----------------------------------------------------------------------------
//
//                       Westwood Studios Pacific.
//
//                       Confidential Information
//                Copyright (C) 2001 - All Rights Reserved
//
//-----------------------------------------------------------------------------
//
// Project:    RTS3
//
// File name:  Dict.h
//
// Created:    Steven Johnson, November 2001
//
// Desc:       General-purpose dictionary class
//
//-----------------------------------------------------------------------------
///////////////////////////////////////////////////////////////////////////////

#pragma once

#include "Common/Errors.h"
#include "Common/NameKeyGenerator.h"

// -----------------------------------------------------
/**
	Dict provides a general utility class for maintaining
	a sorted key-value pair list. Keys are currently required
	to be of type NameKeyType, and data may be Bool, int, real,
	or string.

	Current implementation keeps the pairs sorted by key, and
	does a binary search for lookups; this might change in the future,
	depending on usage and performance (e.g., to a hashtable). So
	don't rely on the quirks of this implementation.
*/
class Dict
{
public:
	enum
	{
		MAX_LEN = 32767							///< max total len of any Dict, in Pairs
	};

	enum DataType
	{
		DICT_NONE = -1,	// this is returned by getType and getNthType to indicate "invalid key/index"
		DICT_BOOL = 0,	// note, we rely on the fact that this constant is zero in the code. so don't change it.
		DICT_INT,
		DICT_REAL,
		DICT_ASCIISTRING,
		DICT_UNICODESTRING,

		DICT_TYPE_COUNT
	};

	/**
		Default constructor -- construct a new, empty Dict.
	*/
	Dict(Int numPairsToPreAllocate = 0);

	/**
		Copy constructor -- make this Dict identical to the
		other Dict. (This is actually quite efficient, because
		they will simply share the same data and increment the
		refcount.)
	*/
	Dict(const Dict& src);

	/**
		Destructor. Not too exciting... clean up the works and such.
	*/
	~Dict();

	/**
	*/
	Dict& operator=(const Dict& src);

	/**
		remove all pairs.
	*/
	void clear();

	/**
		return the number of key-value pairs in the dict.
	*/
	Int getPairCount() const;

	/**
		Return the key for the nth pair (0-based index).
		return NAMEKEY_INVALID if n is out of range.
	*/
	NameKeyType getNthKey(Int n) const;

	/**
		Return the datatype for the pair with the given key.
		return DICT_NONE if no pair with the key exists.
	*/
	DataType getType(NameKeyType key) const;

	/**
		Return there is a pair with the given key and datatype, return true.
	*/
	Bool known(NameKeyType key, DataType d) const
	{
		return getType(key) == d;
	}

	/**
		Return the datatype for the nth pair (0-based index).
		return DICT_NONE if n is out of range.
	*/
	DataType getNthType(Int n) const;

	/**
		return the value for the pair with the given key.
		if there is no pair with the given key, or the value is
		not of the correct type, 0 is returned.
	*/
	Bool getBool(NameKeyType key, Bool* exists = nullptr) const;
	/**
		return the value for the pair with the given key.
		if there is no pair with the given key, or the value is
		not of the correct type, 0 is returned.
	*/
	Int getInt(NameKeyType key, Bool* exists = nullptr) const;
	/**
		return the value for the pair with the given key.
		if there is no pair with the given key, or the value is
		not of the correct type, 0 is returned.
	*/
	Real getReal(NameKeyType key, Bool* exists = nullptr) const;
	/**
		return the value for the pair with the given key.
		if there is no pair with the given key, or the value is
		not of the correct type, "" is returned.
	*/
	AsciiString getAsciiString(NameKeyType key, Bool* exists = nullptr) const;
	/**
		return the value for the pair with the given key.
		if there is no pair with the given key, or the value is
		not of the correct type, "" is returned.
	*/
	UnicodeString getUnicodeString(NameKeyType key, Bool* exists = nullptr) const;

	/**
		return the value for the pair with the given index.
		if the index is out of range, or the value is
		not of the correct type, 0 is returned.
	*/
	Bool getNthBool(Int n) const;
	/**
		return the value for the pair with the given index.
		if the index is out of range, or the value is
		not of the correct type, 0 is returned.
	*/
	Int getNthInt(Int n) const;
	/**
		return the value for the pair with the given index.
		if the index is out of range, or the value is
		not of the correct type, 0 is returned.
	*/
	Real getNthReal(Int n) const;
	/**
		return the value for the pair with the given index.
		if the index is out of range, or the value is
		not of the correct type, "" is returned.
	*/
	AsciiString getNthAsciiString(Int n) const;
	/**
		return the value for the pair with the given index.
		if the index is out of range, or the value is
		not of the correct type, "" is returned.
	*/
	UnicodeString getNthUnicodeString(Int n) const;

	/**
		set the value for the pair with the given key.
		if no such pair exists, it is created.
		if such a pair exists, it is replaced.
		note that when replacing a pair, the new and old
		data types need not match.
	*/
	void setBool(NameKeyType key, Bool value);
	/**
		set the value for the pair with the given key.
		if no such pair exists, it is created.
		if such a pair exists, it is replaced.
		note that when replacing a pair, the new and old
		data types need not match.
	*/
	void setInt(NameKeyType key, Int value);
	/**
		set the value for the pair with the given key.
		if no such pair exists, it is created.
		if such a pair exists, it is replaced.
		note that when replacing a pair, the new and old
		data types need not match.
	*/
	void setReal(NameKeyType key, Real value);
	/**
		set the value for the pair with the given key.
		if no such pair exists, it is created.
		if such a pair exists, it is replaced.
		note that when replacing a pair, the new and old
		data types need not match.
	*/
	void setAsciiString(NameKeyType key, const AsciiString& value);
	/**
		set the value for the pair with the given key.
		if no such pair exists, it is created.
		if such a pair exists, it is replaced.
		note that when replacing a pair, the new and old
		data types need not match.
	*/
	void setUnicodeString(NameKeyType key, const UnicodeString& value);

	/**
		remove the pair with the given key. if such a pair existed, return true.
		if no such pair existed, return false.
	*/
	Bool remove(NameKeyType key);

	/**
		copy the pair with the given key from 'that', replacing any such pair in 'this'.
		if no such pair exists in 'that', any pair with that key will be removed from 'this'.
	*/
	void copyPairFrom(const Dict& that, NameKeyType key);

private:

	struct DictPair;
	struct DictPairData;

	DictPairData* m_data;   // pointer to ref counted Pair data

	void sortPairs();
	Dict::DictPair *setPrep(NameKeyType key, Dict::DataType type);
	DictPair* findPairByKey(NameKeyType key) const;
	void releaseData();
	DictPair *ensureUnique(int numPairsNeeded, Bool preserveData, DictPair *pairToTranslate);

	enum DictPairKeyType
	{
		DICTPAIRKEY_ILLEGAL = 0
	};

	// danger... this is Plain Old Data and allocated in a skanky way;
	// and thus the ctor/dtor for DictPair will never be called. so don't
	// bother writing one.
	struct DictPair
	{
	private:
		DictPairKeyType		m_key;
		void*							m_value;

		static DictPairKeyType createKey(NameKeyType keyVal, DataType nt)
		{
			return (DictPairKeyType)((((UnsignedInt)(keyVal)) << 8) | ((UnsignedInt)nt));
		}

		static DataType getTypeFromKey(DictPairKeyType nk)
		{
			return (DataType)(((UnsignedInt)nk) & 0xff);
		}

		static NameKeyType getNameFromKey(DictPairKeyType nk)
		{
			return (NameKeyType)(((UnsignedInt)nk) >> 8);
		}


	public:
		void clear();
		void copyFrom(DictPair* that);
		void setNameAndType(NameKeyType key, DataType type);
		DataType getType() const { return getTypeFromKey(m_key); }
		NameKeyType getName() const { return getNameFromKey(m_key); }
		Bool* asBool() { return (Bool*)&m_value; }
		Int* asInt() { return (Int*)&m_value; }
		Real* asReal() { return (Real*)&m_value; }
		AsciiString* asAsciiString() { return (AsciiString*)&m_value; }
		UnicodeString* asUnicodeString() { return (UnicodeString*)&m_value; }
	};

	struct DictPairData
	{
		unsigned short	m_refCount;						// reference count
		unsigned short	m_numPairsAllocated;  // length of data allocated
		unsigned short	m_numPairsUsed;				// length of data allocated
		//DictPair m_pairs[];

		DictPair* peek() { return (DictPair*)(this+1); }
	};

	#ifdef RTS_DEBUG
	void validate() const;
	#else
	void validate() const { }
	#endif

};

// -----------------------------------------------------
inline Dict::Dict(const Dict& src) : m_data(src.m_data)
{
	if (m_data)
		++m_data->m_refCount;
}

// -----------------------------------------------------
inline Dict::~Dict()
{
	releaseData();
}

// -----------------------------------------------------
inline Int Dict::getPairCount() const
{
	return m_data ? m_data->m_numPairsUsed : 0;
}

// -----------------------------------------------------
inline NameKeyType Dict::getNthKey(Int n) const
{
	if (!m_data || n < 0 || n >= m_data->m_numPairsUsed)
		return NAMEKEY_INVALID;
	return m_data->peek()[n].getName();
}

// -----------------------------------------------------
inline Dict::DataType Dict::getNthType(Int n) const
{
	if (!m_data || n < 0 || n >= m_data->m_numPairsUsed)
		return DICT_NONE;
	return m_data->peek()[n].getType();
}
```

### `Core/GameEngine/Source/Common/Dict.cpp`
```cpp
// FILE: Dict.cpp
//-----------------------------------------------------------------------------
//
//                       Westwood Studios Pacific.
//
//                       Confidential Information
//                Copyright (C) 2001 - All Rights Reserved
//
//-----------------------------------------------------------------------------
//
// Project:    RTS3
//
// File name:  Dict.cpp
//
// Created:    Steven Johnson, November 2001
//
// Desc:       General-purpose dictionary class
//
//-----------------------------------------------------------------------------
///////////////////////////////////////////////////////////////////////////////

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/Dict.h"
#include "Common/GameMemory.h"

// -----------------------------------------------------
void Dict::DictPair::copyFrom(DictPair* that)
{
	Dict::DataType curType = this->getType();
	Dict::DataType newType = that->getType();
	if (curType != newType)
	{
		clear();
	}

	switch(newType)
	{
		case DICT_BOOL:
		case DICT_INT:
		case DICT_REAL:
			*this = *that;
			break;
		case DICT_ASCIISTRING:
			this->m_key = that->m_key;
			*this->asAsciiString() = *that->asAsciiString();
			break;
		case DICT_UNICODESTRING:
			this->m_key = that->m_key;
			*this->asUnicodeString() = *that->asUnicodeString();
			break;
	}
}

// -----------------------------------------------------
void Dict::DictPair::clear()
{
	switch (getType())
	{
		case DICT_BOOL:
		case DICT_INT:
		case DICT_REAL:
			m_value = nullptr;
			break;
		case DICT_ASCIISTRING:
			asAsciiString()->clear();
			break;
		case DICT_UNICODESTRING:
			asUnicodeString()->clear();
			break;
	}
}

// -----------------------------------------------------
void Dict::DictPair::setNameAndType(NameKeyType key, Dict::DataType type)
{
	Dict::DataType curType = getType();
	if (curType != type)
	{
		clear();
	}
	m_key = createKey(key, type);
}

// -----------------------------------------------------
#ifdef RTS_DEBUG
void Dict::validate() const
{
	if (!m_data) return;
	DEBUG_ASSERTCRASH(m_data->m_refCount > 0, ("m_refCount is zero"));
	DEBUG_ASSERTCRASH(m_data->m_refCount < 32000, ("m_refCount is suspiciously large"));
	DEBUG_ASSERTCRASH(m_data->m_numPairsAllocated > 0, ("m_numPairsAllocated is zero"));
	DEBUG_ASSERTCRASH(m_data->m_numPairsUsed >= 0, ("m_numPairsUsed is neg"));
	DEBUG_ASSERTCRASH(m_data->m_numPairsAllocated >= m_data->m_numPairsUsed, ("m_numPairsAllocated too small"));
	DEBUG_ASSERTCRASH(m_data->m_numPairsAllocated < 1024, ("m_numPairsAllocated suspiciously large"));
}
#endif

// -----------------------------------------------------
Dict::DictPair* Dict::findPairByKey(NameKeyType key) const
{
	DEBUG_ASSERTCRASH(key != NAMEKEY_INVALID, ("invalid namekey!"));
	DEBUG_ASSERTCRASH((UnsignedInt)key < (1L<<23), ("namekey too large!"));
	if (!m_data)
		return nullptr;
	DictPair* base = m_data->peek();
	Int minIdx = 0;
	Int maxIdx = m_data->m_numPairsUsed;
	while (minIdx < maxIdx)
	{
		Int midIdx = (((minIdx + maxIdx) - 1) >> 1);
		DictPair* mid = base + midIdx;
		NameKeyType midKey = mid->getName();
		if (key > midKey)
			minIdx = midIdx + 1;
		else if (key < midKey)
			maxIdx = midIdx;
		else
			return mid;
	}

	return nullptr;
}

// -----------------------------------------------------
Dict::DictPair *Dict::ensureUnique(int numPairsNeeded, Bool preserveData, DictPair *pairToTranslate)
{
	if (m_data &&
			m_data->m_refCount == 1 &&
			m_data->m_numPairsAllocated >= numPairsNeeded)
	{
		// no buffer manhandling is needed (it's already large enough, and unique to us)
		return pairToTranslate;
	}

	Dict::DictPairData* newData = nullptr;
	if (numPairsNeeded > 0)
	{
		DEBUG_ASSERTCRASH(TheDynamicMemoryAllocator != nullptr, ("Cannot use dynamic memory allocator before its initialization. Check static initialization order."));
		DEBUG_ASSERTCRASH(numPairsNeeded <= MAX_LEN, ("Dict::ensureUnique exceeds max pairs length %d with requested length %d", MAX_LEN, numPairsNeeded));
		int minBytes = sizeof(Dict::DictPairData) + numPairsNeeded*sizeof(Dict::DictPair);
		int actualBytes = TheDynamicMemoryAllocator->getActualAllocationSize(minBytes);
		// note: be certain to alloc with zero; we'll take advantage of the fact that all-zero
		// is a bit-pattern that happens to init all our pairs to legal values:
		// type BOOL, key INVALID, value FALSE.
		newData = (Dict::DictPairData*)TheDynamicMemoryAllocator->allocateBytes(actualBytes, "Dict::ensureUnique");
		newData->m_refCount = 1;
		newData->m_numPairsAllocated = (actualBytes - sizeof(Dict::DictPairData))/sizeof(Dict::DictPair);
		newData->m_numPairsUsed = 0;

		if (preserveData && m_data)
		{
			Dict::DictPair* src = m_data->peek();
			Dict::DictPair* dst = newData->peek();
			for (Int i = 0; i < m_data->m_numPairsUsed; ++i, ++src, ++dst)
				dst->copyFrom(src);
			newData->m_numPairsUsed = m_data->m_numPairsUsed;
		}
	}

	Int delta = 0;
	if (pairToTranslate && m_data)
		delta = pairToTranslate - m_data->peek();

	releaseData();
	m_data = newData;

	if (pairToTranslate && m_data)
		pairToTranslate = m_data->peek() + delta;

	return pairToTranslate;
}


// -----------------------------------------------------
void Dict::clear()
{
	releaseData();
	m_data = nullptr;
}

// -----------------------------------------------------
void Dict::releaseData()
{
	if (m_data)
	{
		if (--m_data->m_refCount == 0)
		{
			Dict::DictPair* src = m_data->peek();
			for (Int i = 0; i < m_data->m_numPairsUsed; ++i, ++src)
				src->clear();
			TheDynamicMemoryAllocator->freeBytes(m_data);
		}
		m_data = nullptr;
	}
}

// -----------------------------------------------------
Dict::Dict(Int numPairsToPreAllocate) : m_data(nullptr)
{

	/*
		This class plays some skanky games, in the name of memory and code
		efficiency; it assumes all the data types will fit into a pointer.
		This is currently true, but if that assumption ever changes, all hell
		will break loose. So we do a quick check to assure this...
	*/
	DEBUG_ASSERTCRASH(sizeof(Bool) <= sizeof(void*) &&
										sizeof(Int) <= sizeof(void*) &&
										sizeof(Real) <= sizeof(void*) &&
										sizeof(AsciiString) <= sizeof(void*) &&
										sizeof(UnicodeString) <= sizeof(void*), ("oops, this code needs attention"));

	if (numPairsToPreAllocate)
		ensureUnique(numPairsToPreAllocate, false, nullptr);	// will throw on error
}

// -----------------------------------------------------
Dict& Dict::operator=(const Dict& src)
{
	validate();
	if (&src != this)
	{
		releaseData();
		m_data = src.m_data;
		if (m_data)
			++m_data->m_refCount;
	}
	validate();
	return *this;
}

// -----------------------------------------------------
Dict::DataType Dict::getType(NameKeyType key) const
{
	validate();
	DictPair* pair = findPairByKey(key);
	if (pair)
		return pair->getType();
	return DICT_NONE;
}

// -----------------------------------------------------
Bool Dict::getBool(NameKeyType key, Bool *exists/*=nullptr*/) const
{
	validate();
	DictPair* pair = findPairByKey(key);
	if (pair && pair->getType() == DICT_BOOL)
	{
		if (exists) *exists = true;
		return *pair->asBool();
	}
	DEBUG_ASSERTCRASH(exists != nullptr, ("dict key missing, or of wrong type"));	// only assert if they didn't check result
	if (exists) *exists = false;
	return false;
}

// -----------------------------------------------------
Int Dict::getInt(NameKeyType key, Bool *exists/*=nullptr*/) const
{
	validate();
	DictPair* pair = findPairByKey(key);
	if (pair && pair->getType() == DICT_INT)
	{
		if (exists) *exists = true;
		return *pair->asInt();
	}
	DEBUG_ASSERTCRASH(exists != nullptr,("dict key missing, or of wrong type"));	// only assert if they didn't check result
	if (exists) *exists = false;
	return 0;
}

// -----------------------------------------------------
Real Dict::getReal(NameKeyType key, Bool *exists/*=nullptr*/) const
{
	validate();
	DictPair* pair = findPairByKey(key);
	if (pair && pair->getType() == DICT_REAL)
	{
		if (exists) *exists = true;
		return *pair->asReal();
	}
	DEBUG_ASSERTCRASH(exists != nullptr,("dict key missing, or of wrong type"));	// only assert if they didn't check result
	if (exists) *exists = false;
	return 0.0f;
}

// -----------------------------------------------------
AsciiString Dict::getAsciiString(NameKeyType key, Bool *exists/*=nullptr*/) const
{
	validate();
	DictPair* pair = findPairByKey(key);
	if (pair && pair->getType() == DICT_ASCIISTRING)
	{
		if (exists) *exists = true;
		return *pair->asAsciiString();
	}
	DEBUG_ASSERTCRASH(exists != nullptr,("dict key missing, or of wrong type"));	// only assert if they didn't check result
	if (exists) *exists = false;
	return AsciiString::TheEmptyString;
}

// -----------------------------------------------------
UnicodeString Dict::getUnicodeString(NameKeyType key, Bool *exists/*=nullptr*/) const
{
	validate();
	DictPair* pair = findPairByKey(key);
	if (pair && pair->getType() == DICT_UNICODESTRING)
	{
		if (exists) *exists = true;
		return *pair->asUnicodeString();
	}
	DEBUG_ASSERTCRASH(exists != nullptr,("dict key missing, or of wrong type"));	// only assert if they didn't check result
	if (exists) *exists = false;
	return UnicodeString::TheEmptyString;
}

// -----------------------------------------------------
Bool Dict::getNthBool(Int n) const
{
	validate();
	DEBUG_ASSERTCRASH(n >= 0 && n < getPairCount(), ("n out of range"));
	if (m_data)
	{
		DictPair* pair = &m_data->peek()[n];
		if (pair && pair->getType() == DICT_BOOL)
			return *pair->asBool();
	}
	DEBUG_CRASH(("dict key missing, or of wrong type"));
	return false;
}

// -----------------------------------------------------
Int Dict::getNthInt(Int n) const
{
	validate();
	DEBUG_ASSERTCRASH(n >= 0 && n < getPairCount(), ("n out of range"));
	if (m_data)
	{
		DictPair* pair = &m_data->peek()[n];
		if (pair && pair->getType() == DICT_INT)
			return *pair->asInt();
	}
	DEBUG_CRASH(("dict key missing, or of wrong type"));
	return 0;
}

// -----------------------------------------------------
Real Dict::getNthReal(Int n) const
{
	validate();
	DEBUG_ASSERTCRASH(n >= 0 && n < getPairCount(), ("n out of range"));
	if (m_data)
	{
		DictPair* pair = &m_data->peek()[n];
		if (pair && pair->getType() == DICT_REAL)
			return *pair->asReal();
	}
	DEBUG_CRASH(("dict key missing, or of wrong type"));
	return 0.0f;
}

// -----------------------------------------------------
AsciiString Dict::getNthAsciiString(Int n) const
{
	validate();
	DEBUG_ASSERTCRASH(n >= 0 && n < getPairCount(), ("n out of range"));
	if (m_data)
	{
		DictPair* pair = &m_data->peek()[n];
		if (pair && pair->getType() == DICT_ASCIISTRING)
			return *pair->asAsciiString();
	}
	DEBUG_CRASH(("dict key missing, or of wrong type"));
	return AsciiString::TheEmptyString;
}

// -----------------------------------------------------
UnicodeString Dict::getNthUnicodeString(Int n) const
{
	validate();
	DEBUG_ASSERTCRASH(n >= 0 && n < getPairCount(), ("n out of range"));
	if (m_data)
	{
		DictPair* pair = &m_data->peek()[n];
		if (pair && pair->getType() == DICT_UNICODESTRING)
			return *pair->asUnicodeString();
	}
	DEBUG_CRASH(("dict key missing, or of wrong type"));
	return UnicodeString::TheEmptyString;
}

// -----------------------------------------------------
Dict::DictPair *Dict::setPrep(NameKeyType key, Dict::DataType type)
{
	DictPair* pair = findPairByKey(key);
	Int pairsNeeded = getPairCount();
	if (!pair)
		++pairsNeeded;
	pair = ensureUnique(pairsNeeded, true, pair);
	if (!pair)
	{
		pair = &m_data->peek()[m_data->m_numPairsUsed++];
	}
	pair->setNameAndType(key, type);
	DEBUG_ASSERTCRASH(pair, ("pair must not be null here"));
	return pair;
}

// -----------------------------------------------------
void Dict::sortPairs()
{
	if (!m_data)
		return;

	// yer basic shellsort.
	for (Int gap = m_data->m_numPairsUsed >> 1; gap > 0; gap >>= 1)
	{
		for (Int i = gap; i < m_data->m_numPairsUsed; i++)
		{
			for (Int j = i - gap; j >= 0; j -= gap)
			{
				DictPair* a = m_data->peek() + j;
				DictPair* b = m_data->peek() + j + gap;
				if (a->getName() > b->getName())
				{
					DictPair tmp = *a;
					*a = *b;
					*b = tmp;
				}
				else
				{
					break;
				}
			}
		}
	}
}

// -----------------------------------------------------
void Dict::setBool(NameKeyType key, Bool value)
{
	validate();
	DictPair* pair = setPrep(key, DICT_BOOL);
	*pair->asBool() = value;
	sortPairs();
	validate();
}

// -----------------------------------------------------
void Dict::setInt(NameKeyType key, Int value)
{
	validate();
	DictPair* pair = setPrep(key, DICT_INT);
	*pair->asInt() = value;
	sortPairs();
	validate();
}

// -----------------------------------------------------
void Dict::setReal(NameKeyType key, Real value)
{
	validate();
	DictPair* pair = setPrep(key, DICT_REAL);
	*pair->asReal() = value;
	sortPairs();
	validate();
}

// -----------------------------------------------------
void Dict::setAsciiString(NameKeyType key, const AsciiString& value)
{
	validate();
	DictPair* pair = setPrep(key, DICT_ASCIISTRING);
	*pair->asAsciiString() = value;
	sortPairs();
	validate();
}

// -----------------------------------------------------
void Dict::setUnicodeString(NameKeyType key, const UnicodeString& value)
{
	validate();
	DictPair* pair = setPrep(key, DICT_UNICODESTRING);
	*pair->asUnicodeString() = value;
	sortPairs();
	validate();
}

// -----------------------------------------------------
Bool Dict::remove(NameKeyType key)
{
	validate();
	DictPair* pair = findPairByKey(key);
	if (pair)
	{
		pair = ensureUnique(m_data->m_numPairsUsed, true, pair);
		pair->setNameAndType((NameKeyType)0x7fffffff, DICT_BOOL);
		sortPairs();
		--m_data->m_numPairsUsed;
		validate();
		return true;
	}
	DEBUG_CRASH(("dict key missing in remove"));
	return false;
}

// -----------------------------------------------------
void Dict::copyPairFrom(const Dict& that, NameKeyType key)
{
	this->validate();
	DictPair* thatPair = that.findPairByKey(key);
	if (thatPair)
	{
		DictPair* thisPair = this->setPrep(key, thatPair->getType());
		thisPair->copyFrom(thatPair);
		this->sortPairs();
	}
	else
	{
		if (this->findPairByKey(key))
			this->remove(key);
	}
	this->validate();
}
```

---

## 4. Architect Notes & Existing Rust Helpers

1. **Exact Imports Available in `zh_common`:**
   - `use crate::types::{Bool, Int, NameKeyType, Real, FALSE, TRUE};`
2. **Determinism & Semantics Notes (`Dict.h` / `Dict.cpp`):**
   - **Storage &Ordering (`DET001` / `DET005`):** Store pairs in a `Vec<(NameKeyType, DictValue)>` sorted ascending by `NameKeyType` (`key.0`) using binary search (`findPairByKey` / `binary_search_by_key`), **never** a `HashMap`.
   - **Empty / Missing Key Queries (`Dict::get_*`, `Dict::get_nth_*`, `Dict::remove`):**
     - `get_type` returns `DictDataType::None` (`-1`) if missing; `get_nth_type` / `get_nth_key` return `DictDataType::None` / `NameKeyType::INVALID` if out of range (`n < 0 || n >= len`).
     - `get_bool` / `get_int` / `get_real` / `get_ascii_string` / `get_unicode_string` set `*exists = TRUE/FALSE` when `Some(flag)` is passed and return type defaults (`FALSE`, `0`, `0.0`, `String::new()`, `Vec::new()`) on missing key or type mismatch.
     - `remove` returns `TRUE` (`1`) if removed, `FALSE` (`0`) if not present.
     - `copy_pair_from(that, key)` copies the key-value entry from `that` if present (overwriting any existing entry for `key`), or removes `key` from `self` if absent in `that`.
   - **Value / Clone Semantics:** `#[derive(Clone)]` on `Dict` naturally provides C++ copy-on-write independence (`let mut cloned = orig.clone(); cloned.set_int(...);` does not mutate `orig`).
   - **Wide Strings:** `UnicodeString` maps to `Vec<u16>` (canonical UTF-16 code units across Linux, macOS, and Windows).

