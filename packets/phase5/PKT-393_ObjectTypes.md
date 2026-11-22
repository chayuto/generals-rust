# Translation Packet `PKT-393`: `ObjectTypes`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-393`
- **Work Package:** `WP-5.5` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:hybrid-ci`
- **FFI Seam Pattern:** Core Spine
- **Allowed Rust Output File:** `crates/zh_logic/src/object/object_types.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (9):**
  - `Common/AsciiString.h`
  - `Common/GameState.h`
  - `Common/Player.h`
  - `Common/Snapshot.h`
  - `Common/ThingFactory.h`
  - `Common/ThingTemplate.h`
  - `Common/Xfer.h`
  - `GameLogic/ObjectTypes.h`
  - `PreRTS.h`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (5 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferAsciiString( &m_listName )`
  3. `xferUnsignedShort( &objectTypesCount )`
  4. `xferAsciiString( &(*it) )`
  5. `xferAsciiString( &typeName )`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/ObjectTypes.h`
```cpp
// ObjectTypes.h //////////////////////////////////////////////////////////////////////////////////
// Author: John McDonald, Jr.
// September 2002
// DO NOT DISTRIBUTE

#pragma once

#include "Common/AsciiString.h"
#include "Common/Snapshot.h"

class Player;

class ObjectTypes : public MemoryPoolObject,
										public Snapshot
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(ObjectTypes, "ObjectTypes")
private:
	typedef std::vector<AsciiString> AsciiStringVec;
	typedef AsciiStringVec::iterator AsciiStringVecIt;

private:
	// Note, there is no direct access to m_objectTypes. Please endeavor to keep it this way, and
	// add any functions you might need to this class.
	AsciiString m_listName;
	AsciiStringVec m_objectTypes;

protected:
	// snapshot methods
	virtual void crc(Xfer *xfer) override;
	virtual void xfer(Xfer *xfer) override;
	virtual void loadPostProcess() override;

public:
	ObjectTypes();
	ObjectTypes(const AsciiString& listName);

public:
	// Maintenance
	void addObjectType(const AsciiString &objectType);
	void removeObjectType(const AsciiString &objectType);

	// Which list is this again?
	const AsciiString& getListName() const;
	void setListName(const AsciiString& listName);

	// Equivalence testing
	Bool isInSet(const AsciiString& objectType) const;
	Bool isInSet(const ThingTemplate* objectType) const;

	// Is the set empty?
	size_t getListSize() const { return m_objectTypes.size(); }

	// I'd like to loop through, please.
	AsciiString getNthInList( size_t index ) const { return (index < getListSize()) ? m_objectTypes[index] : AsciiString::TheEmptyString; }

	// Prep two arrays for usage with Player::countObjectsByThingTemplate
	Int prepForPlayerCounting( std::vector<const ThingTemplate *>& templates, std::vector<Int>& counts);

	// Can the player build any of the thing templates that we have names for?
	Bool canBuildAny(Player *player);
};
EMPTY_DTOR(ObjectTypes)
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/ObjectTypes.cpp`
```cpp
// FILE: ObjectTypes.cpp //////////////////////////////////////////////////////////////////////////
// Author: John McDonald, Jr.
// September 2002
///////////////////////////////////////////////////////////////////////////////////////////////////

#include "PreRTS.h"
#include "GameLogic/ObjectTypes.h"

#include "Common/GameState.h"
#include "Common/Player.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"

//-------------------------------------------------------------------------------------------------
ObjectTypes::ObjectTypes()
{
	// Nada
}

//-------------------------------------------------------------------------------------------------
ObjectTypes::ObjectTypes(const AsciiString& listName) : m_listName(listName)
{
	// Nada
}

//-------------------------------------------------------------------------------------------------
void ObjectTypes::addObjectType(const AsciiString &objectType)
{
	if (isInSet(objectType)) {
		return;
	}

	m_objectTypes.push_back(objectType);
}

//-------------------------------------------------------------------------------------------------
void ObjectTypes::removeObjectType(const AsciiString &objectType)
{
	if (!isInSet(objectType)) {
		DEBUG_CRASH(("Attempted to remove '%s' from '%s', but it wasn't there.", objectType.str(), m_listName.str()));
		return;
	}

	AsciiStringVec::iterator it = std::find(m_objectTypes.begin(), m_objectTypes.end(), objectType);

	m_objectTypes.erase(it);
}

//-------------------------------------------------------------------------------------------------
const AsciiString& ObjectTypes::getListName() const
{
	return m_listName;
}

//-------------------------------------------------------------------------------------------------
void ObjectTypes::setListName(const AsciiString& listName)
{
	m_listName = listName;
}

//-------------------------------------------------------------------------------------------------
Bool ObjectTypes::isInSet(const AsciiString& objectType) const
{
	return (std::find(m_objectTypes.begin(), m_objectTypes.end(), objectType) != m_objectTypes.end());

}

//-------------------------------------------------------------------------------------------------
Bool ObjectTypes::isInSet(const ThingTemplate* objectType) const
{
	if (!objectType) {
		return FALSE;
	}

	return isInSet(objectType->getName());
}

//-------------------------------------------------------------------------------------------------
Int ObjectTypes::prepForPlayerCounting( std::vector<const ThingTemplate *>& templates, std::vector<Int>& counts)
{
	AsciiStringVecIt it;
	for (it = m_objectTypes.begin(); it != m_objectTypes.end(); ++it) {
		const ThingTemplate *templ = TheThingFactory->findTemplate(*it);
		if (templ) {
			templates.push_back(templ);
		}
	}

	Int retVal = templates.size();
	counts.resize(retVal);

	return retVal;
}

//-------------------------------------------------------------------------------------------------
Bool ObjectTypes::canBuildAny(Player *player)
{
	AsciiStringVecIt it;
	for (it = m_objectTypes.begin(); it != m_objectTypes.end(); ++it) {
		const ThingTemplate *templ = TheThingFactory->findTemplate(*it);
		if (templ && player->canBuild(templ)) {
			return TRUE;
		}
	}

	return FALSE;
}

// ------------------------------------------------------------------------------------------------
void ObjectTypes::crc(Xfer *xfer)
{

}

// ------------------------------------------------------------------------------------------------
/** Version Info:
	* 1: Initial version
	*/
// ------------------------------------------------------------------------------------------------
void ObjectTypes::xfer(Xfer *xfer)
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// list name
	xfer->xferAsciiString( &m_listName );

	// size of object types vector
	UnsignedShort objectTypesCount = m_objectTypes.size();
	xfer->xferUnsignedShort( &objectTypesCount );

	// object types data
	if( xfer->getXferMode() == XFER_SAVE )
	{

		// iterate vector
		AsciiStringVecIt it;
		for( it = m_objectTypes.begin(); it != m_objectTypes.end(); ++it )
		{

			// write type name
			xfer->xferAsciiString( &(*it) );

		}

	}
	else
	{

		// sanity, the vector should be empty when loading
		if( m_objectTypes.empty() == FALSE )
		{

			DEBUG_CRASH(( "ObjectTypes::xfer - m_objectTypes vector should be empty but is not!" ));
			throw SC_INVALID_DATA;

		}

		// read all data
		AsciiString typeName;
		for( UnsignedShort i = 0; i < objectTypesCount; ++i )
		{

			// read name
			xfer->xferAsciiString( &typeName );

			// put on vector
			m_objectTypes.push_back( typeName );

		}

	}

}

// ------------------------------------------------------------------------------------------------
void ObjectTypes::loadPostProcess()
{

}
```
