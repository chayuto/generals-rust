# Translation Packet `PKT-022`: `Geometry`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-022`
- **Work Package:** `WP-2.2` (Phase 2)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 1: C-ABI Leaf
- **Allowed Rust Output File:** `crates/zh_common/src/math/geometry.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_common/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (8):**
  - `Common/AsciiString.h`
  - `Common/Geometry.h`
  - `Common/INI.h`
  - `Common/RandomValue.h`
  - `Common/Snapshot.h`
  - `Common/Xfer.h`
  - `Lib/BaseType.h`
  - `PreRTS.h`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (8 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUser( &m_type, sizeof( GeometryType ) )`
  3. `xferBool( &m_isSmall )`
  4. `xferReal( &m_height )`
  5. `xferReal( &m_majorRadius )`
  6. `xferReal( &m_minorRadius )`
  7. `xferReal( &m_boundingCircleRadius )`
  8. `xferReal( &m_boundingSphereRadius )`
- **PRNG Calls (10 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValueReal(-m_majorRadius, m_majorRadius)`
  - `GameLogicRandomValueReal(-m_majorRadius, m_majorRadius)`
  - `GameLogicRandomValueReal(0.0f, m_boundingCircleRadius)`
  - `GameLogicRandomValueReal(-PI, PI)`
  - `GameLogicRandomValueReal(-m_majorRadius, m_majorRadius)`
  - `GameLogicRandomValueReal(-m_minorRadius, m_minorRadius)`
  - `GameLogicRandomValueReal( 0.0f, 1.0f ) < 0.5f ) { //Pick random point on x axis. pt.x = GameLogicRandomValueReal(-m_majorRadius, m_majorRadius)`
  - `GameLogicRandomValueReal( 0.0f, 1.0f ) < 0.5f )`
  - `GameLogicRandomValueReal(-m_minorRadius, m_minorRadius)`
  - `GameLogicRandomValueReal( 0.0f, 1.0f ) < 0.5f )`

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

### `GeneralsMD/Code/GameEngine/Include/Common/Geometry.h`
```cpp
// FILE: Geometry.h ///////////////////////////////////////////////////////////////////////////////
// Author: Colin Day, November 2001
// Desc:	 Geometry descriptions
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

#include "Lib/BaseType.h"
#include "Common/AsciiString.h"
#include "Common/Snapshot.h"

class INI;

//-------------------------------------------------------------------------------------------------
/** Geometry type descriptions, keep this in the same order as GeometryNames[] below
	*
	* NOTE: Do *NOT* change the order of these defines unless you update the
	* partition manager ... in particular theCollideTestProcs depend on the
	* order of this geometry and the fact that the values start at 1
	*/
//-------------------------------------------------------------------------------------------------
enum GeometryType CPP_11(: Int)
{
	GEOMETRY_SPHERE,			///< partition/collision testing as sphere. (majorRadius = radius)
	GEOMETRY_CYLINDER,		///< partition/collision testing as cylinder. (majorRadius = radius, height = height)
	GEOMETRY_BOX,					///< partition/collision testing as rectangular box (majorRadius = half len in forward dir; minorRadius = half len in side dir; height = height)

	GEOMETRY_NUM_TYPES,
	GEOMETRY_FIRST = 0
};

#ifdef DEFINE_GEOMETRY_NAMES
static const char *const GeometryNames[] =
{
	"SPHERE",
	"CYLINDER",
	"BOX",
	nullptr
};
static_assert(ARRAY_SIZE(GeometryNames) == GEOMETRY_NUM_TYPES + 1, "Incorrect array size");
#endif  // end DEFINE_GEOMETRY_NAMES

//-------------------------------------------------------------------------------------------------
#if defined(RTS_DEBUG)
enum ExtentModType CPP_11(: Int)
{
	EXTENTMOD_INVALID = 0,
	EXTENTMOD_TYPE = 1,
	EXTENTMOD_MAJOR = 2,
	EXTENTMOD_MINOR = 3,
	EXTENTMOD_HEIGHT = 4,
};
static const Real EXTENT_BIG_CHANGE = 10.0f;
#endif

//-------------------------------------------------------------------------------------------------
/** Geometry information */
//-------------------------------------------------------------------------------------------------
class GeometryInfo : public Snapshot
{
private:
	GeometryType m_type;
	Bool m_isSmall;						///< if true, geometry is assumed to fit in a single partition cell
	Real m_height;
	Real m_majorRadius;
	Real m_minorRadius;

	Real m_boundingCircleRadius;	///< not in INI file -- size of bounding circle (2d)
	Real m_boundingSphereRadius;	///< not in INI -- size of bounding sphere (3d)

	void calcBoundingStuff();

protected:

	// snapshot methods
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

public:

	static void parseGeometryType( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ );
	static void parseGeometryIsSmall( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ );
	static void parseGeometryHeight( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ );
	static void parseGeometryMajorRadius( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ );
	static void parseGeometryMinorRadius( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ );

	GeometryInfo(GeometryType type, Bool isSmall, Real height, Real majorRadius, Real minorRadius)
	{
		m_boundingCircleRadius = 0.0f;
		m_boundingSphereRadius = 0.0f;
		set(type, isSmall, height, majorRadius, minorRadius);
	}

	void set(GeometryType type, Bool isSmall, Real height, Real majorRadius, Real minorRadius);

	// bleah, icky but needed for legacy code
	void setMajorRadius(Real majorRadius)
	{
		m_majorRadius = majorRadius;
		calcBoundingStuff();
	}

	// bleah, icky but needed for legacy code
	void setMinorRadius(Real minorRadius)
	{
		m_minorRadius = minorRadius;
		calcBoundingStuff();
	}

	GeometryType getGeomType() const { return m_type; }
	Bool getIsSmall() const { return m_isSmall; }
	Real getMajorRadius() const { return m_majorRadius; }	// x-axis
	Real getMinorRadius() const { return m_minorRadius; }	// y-axis

	// this has been removed and should never need to be called...
	// you should generally call getMaxHeightAbovePosition() instead. (srj)
	//inline Real getGeomHeight() const { return m_height; }				// z-axis

	Real getBoundingCircleRadius() const { return m_boundingCircleRadius; }
	Real getBoundingSphereRadius() const { return m_boundingSphereRadius; }

	Bool isIntersectedByLineSegment(const Coord3D& loc, const Coord3D& from, const Coord3D& to) const;

	Real getFootprintArea() const;

	// given an object with this geom, how far above the object's canonical position does its max z extend?
	Real getMaxHeightAbovePosition() const;

	// given an object with this geom, how far below the object's canonical position does its max z extend?
	Real getMaxHeightBelowPosition() const;

	// given an object with this geom, how far above/below the object's canonical position is its center?
	Real getZDeltaToCenterPosition() const;

	// given an object with this geom, located at 'pos', where is the "center" of the geometry?
	void getCenterPosition(const Coord3D& pos, Coord3D& center) const;

	// given an object with this geom, located at 'pos', and another obj with the given
	// pos & geom, calc the min and max pitches from this to that.
	void calcPitches(const Coord3D& thisPos, const GeometryInfo& that, const Coord3D& thatPos,
		Real& minPitch, Real& maxPitch) const;

	/// expand the 2d footprint
	void expandFootprint(Real radius);

	/// get the 2d bounding box
	void get2DBounds(const Coord3D& geomCenter, Real angle, Region2D& bounds	) const;

	/// note that the pt is generated using game logic random, not game client random!
	void makeRandomOffsetWithinFootprint(Coord3D& pt) const;
	void makeRandomOffsetOnPerimeter(Coord3D& pt) const; //Chooses a random point on the extent border.

	void clipPointToFootprint(const Coord3D& geomCenter, Coord3D& ptToClip) const;

	Bool isPointInFootprint(const Coord3D& geomCenter, const Coord3D& pt) const;

	// given an object with this geom, SET how far above the object's canonical position its max z should extend.
	void setMaxHeightAbovePosition(Real z);

#if defined(RTS_DEBUG)
	void tweakExtents(ExtentModType extentModType, Real extentModAmount);
	AsciiString getDescriptiveString() const;
#endif

};
```

### `GeneralsMD/Code/GameEngine/Source/Common/System/Geometry.cpp`
```cpp
// FILE: Geometry.cpp /////////////////////////////////////////////////////////////////////////////
// Author: Steven Johnson, Aug 2002
// Desc:
///////////////////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#define DEFINE_GEOMETRY_NAMES

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "Common/Geometry.h"
#include "Common/INI.h"
#include "Common/RandomValue.h"
#include "Common/Xfer.h"


///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

//=============================================================================
/*static*/ void GeometryInfo::parseGeometryType( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	((GeometryInfo*)store)->m_type = (GeometryType)INI::scanIndexList(ini->getNextToken(), GeometryNames);
	((GeometryInfo*)store)->calcBoundingStuff();
}

//=============================================================================
/*static*/ void GeometryInfo::parseGeometryIsSmall( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	((GeometryInfo*)store)->m_isSmall = INI::scanBool(ini->getNextToken());
	((GeometryInfo*)store)->calcBoundingStuff();
}

//=============================================================================
/*static*/ void GeometryInfo::parseGeometryHeight( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	((GeometryInfo*)store)->m_height = INI::scanReal(ini->getNextToken());
	((GeometryInfo*)store)->calcBoundingStuff();
}

//=============================================================================
/*static*/ void GeometryInfo::parseGeometryMajorRadius( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	((GeometryInfo*)store)->m_majorRadius = INI::scanReal(ini->getNextToken());
	((GeometryInfo*)store)->calcBoundingStuff();
}

//=============================================================================
/*static*/ void GeometryInfo::parseGeometryMinorRadius( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	((GeometryInfo*)store)->m_minorRadius = INI::scanReal(ini->getNextToken());
	((GeometryInfo*)store)->calcBoundingStuff();
}

//-----------------------------------------------------------------------------
void GeometryInfo::set(GeometryType type, Bool isSmall, Real height, Real majorRadius, Real minorRadius)
{
	m_type = type;
	m_isSmall = isSmall;

	switch(m_type)
	{
		case GEOMETRY_SPHERE:
			m_majorRadius = majorRadius;
			m_minorRadius = majorRadius;
			m_height = majorRadius;
			break;

		case GEOMETRY_CYLINDER:
			m_majorRadius = majorRadius;
			m_minorRadius = majorRadius;
			m_height = height;
			break;

		case GEOMETRY_BOX:
			m_majorRadius = majorRadius;
			m_minorRadius = minorRadius;
			m_height = height;
			break;
	};

	calcBoundingStuff();
}

//-----------------------------------------------------------------------------
static Real calcDotProduct(const Coord3D& a, const Coord3D& b)
{
	return a.x*b.x + a.y*b.y + a.z*b.z;
}

//-----------------------------------------------------------------------------
static Real calcDistSquared(const Coord3D& a, const Coord3D& b)
{
	return sqr(a.x - b.x) + sqr(a.y - b.y) + sqr(a.z - b.z);
}

//-----------------------------------------------------------------------------
static Real calcPointToLineDistSquared(const Coord3D& pt, const Coord3D& lineStart, const Coord3D& lineEnd)
{
	Coord3D line, lineToPt, closest;

	line.x = lineEnd.x - lineStart.x;
	line.y = lineEnd.y - lineStart.y;
	line.z = lineEnd.z - lineStart.z;

	lineToPt.x = pt.x - lineStart.x;
	lineToPt.y = pt.y - lineStart.y;
	lineToPt.z = pt.z - lineStart.z;

	Real dot = calcDotProduct(lineToPt, line);
	if (dot <= 0.0f)
	{
		// angle is obtuse
		return calcDistSquared(pt, lineStart);
	}

	Real lineLenSqr = calcDistSquared(lineStart, lineEnd);
	DEBUG_ASSERTCRASH(lineLenSqr==calcDotProduct(line,line),("hmm"));
	if (lineLenSqr <= dot)
	{
		return calcDistSquared(pt, lineEnd);
	}

  Real tmp = dot / lineLenSqr;

	closest.x = lineStart.x + tmp * line.x;
	closest.y = lineStart.y + tmp * line.y;
	closest.z = lineStart.z + tmp * line.z;

  return calcDistSquared(pt, closest);
}

//=============================================================================
Bool GeometryInfo::isIntersectedByLineSegment(const Coord3D& loc, const Coord3D& from, const Coord3D& to) const
{
	DEBUG_CRASH(("this call does not work properly for nonspheres yet. use with caution."));

	/// @todo srj -- treats everything as a sphere for now. fix.
	Real distSquared = calcPointToLineDistSquared(loc, from, to);
	return distSquared <= sqr(getBoundingSphereRadius());
}

//=============================================================================
// given an object with this geom, located at 'pos', and another obj with the given
// pos & geom, calc the min and max pitches from this to that.
void GeometryInfo::calcPitches(const Coord3D& thisPos, const GeometryInfo& that, const Coord3D& thatPos,
	Real& minPitch, Real& maxPitch) const
{
	Coord3D thisCenter;
	getCenterPosition(thisPos, thisCenter);

	Real dxy = WWMath::Sqrt(sqr(thatPos.x - thisCenter.x) + sqr(thatPos.y - thisCenter.y));

	Real dz;

	/** @todo srj -- this could be better, by calcing it for all the corners, not just top-center
		and bottom-center... oh well */
	dz = (thatPos.z + that.getMaxHeightAbovePosition()) - thisCenter.z;
	maxPitch = WWMath::Atan2(dz, dxy);

	dz = (thatPos.z - that.getMaxHeightBelowPosition()) - thisCenter.z;
	minPitch = WWMath::Atan2(dz, dxy);
}

//=============================================================================
// given an object with this geom, SET how far above the object's canonical position its max z should extend.
void GeometryInfo::setMaxHeightAbovePosition(Real z)
{
	switch(m_type)
	{
		case GEOMETRY_SPHERE:
			m_majorRadius = z;
			break;

		case GEOMETRY_BOX:
		case GEOMETRY_CYLINDER:
			m_height = z;
			break;
	};

	calcBoundingStuff();
}

//=============================================================================
// given an object with this geom, how far above the object's canonical position does its max z extend?
Real GeometryInfo::getMaxHeightAbovePosition() const
{
	switch(m_type)
	{
		case GEOMETRY_SPHERE:
			return m_majorRadius;

		case GEOMETRY_BOX:
		case GEOMETRY_CYLINDER:
			return m_height;
	};

	return 0.0f;
}

//=============================================================================
// given an object with this geom, how far below the object's canonical position does its max z extend?
Real GeometryInfo::getMaxHeightBelowPosition() const
{
	switch(m_type)
	{
		case GEOMETRY_SPHERE:
			return m_majorRadius;

		case GEOMETRY_BOX:
		case GEOMETRY_CYLINDER:
			return 0.0f;
	};

	return 0.0f;
}

//=============================================================================
// given an object with this geom, located at 'pos', where is the "center" of the geometry?
Real GeometryInfo::getZDeltaToCenterPosition() const
{
	return (m_type == GEOMETRY_SPHERE) ? 0.0f : (m_height * 0.5f);
}

//=============================================================================
// given an object with this geom, located at 'pos', where is the "center" of the geometry?
void GeometryInfo::getCenterPosition(const Coord3D& pos, Coord3D& center) const
{
	center = pos;
	center.z += getZDeltaToCenterPosition();
}

//=============================================================================
void GeometryInfo::expandFootprint(Real radius)
{
	m_majorRadius += radius;
	m_minorRadius += radius;
	calcBoundingStuff();
}

//=============================================================================
void GeometryInfo::get2DBounds(const Coord3D& geomCenter, Real angle, Region2D& bounds) const
{
	switch(m_type)
	{
		case GEOMETRY_SPHERE:
		case GEOMETRY_CYLINDER:
		{
			bounds.lo.x = geomCenter.x - m_majorRadius;
			bounds.lo.y = geomCenter.y - m_majorRadius;
			bounds.hi.x = geomCenter.x + m_majorRadius;
			bounds.hi.y = geomCenter.y + m_majorRadius;
			break;
		}

		case GEOMETRY_BOX:
		{
			Real c = (Real)WWMath::Cos(angle);
			Real s = (Real)WWMath::Sin(angle);
			Real exc = m_majorRadius*c;
			Real eyc = m_minorRadius*c;
			Real exs = m_majorRadius*s;
			Real eys = m_minorRadius*s;
			Real x,y;
			x = geomCenter.x - exc - eys;
			y = geomCenter.y + eyc - exs;
			bounds.lo.x = x;
			bounds.lo.y = y;
			bounds.hi.x = x;
			bounds.hi.y = y;

			x = geomCenter.x + exc - eys;
			y = geomCenter.y + eyc + exs;
			if (bounds.lo.x > x) bounds.lo.x = x;
			if (bounds.lo.y > y) bounds.lo.y = y;
			if (bounds.hi.x < x) bounds.hi.x = x;
			if (bounds.hi.y < y) bounds.hi.y = y;

			x = geomCenter.x + exc + eys;
			y = geomCenter.y - eyc + exs;
			if (bounds.lo.x > x) bounds.lo.x = x;
			if (bounds.lo.y > y) bounds.lo.y = y;
			if (bounds.hi.x < x) bounds.hi.x = x;
			if (bounds.hi.y < y) bounds.hi.y = y;

			x = geomCenter.x - exc + eys;
			y = geomCenter.y - eyc - exs;
 			if (bounds.lo.x > x) bounds.lo.x = x;
			if (bounds.lo.y > y) bounds.lo.y = y;
			if (bounds.hi.x < x) bounds.hi.x = x;
			if (bounds.hi.y < y) bounds.hi.y = y;

			break;
		}
	}
}

//=============================================================================
void GeometryInfo::clipPointToFootprint(const Coord3D& geomCenter, Coord3D& ptToClip) const
{
	switch(m_type)
	{
		case GEOMETRY_SPHERE:
		case GEOMETRY_CYLINDER:
		{
			Real dx = ptToClip.x - geomCenter.x;
			Real dy = ptToClip.y - geomCenter.y;
			Real radius = WWMath::Sqrt(sqr(dx) + sqr(dy));
			if (radius > m_majorRadius)
			{
				Real ratio = m_majorRadius / radius;
				ptToClip.x = geomCenter.x + dx * ratio;
				ptToClip.y = geomCenter.y + dy * ratio;
			}
			break;
		}

		case GEOMETRY_BOX:
		{
			ptToClip.x = clamp(geomCenter.x - m_majorRadius, ptToClip.x, geomCenter.x + m_majorRadius);
			ptToClip.y = clamp(geomCenter.y - m_minorRadius, ptToClip.y, geomCenter.y + m_minorRadius);
			break;
		}
	};
}

//=============================================================================
inline Bool isWithin(Real a, Real b, Real c) { return a<=b && b<=c; }

//=============================================================================
Bool GeometryInfo::isPointInFootprint(const Coord3D& geomCenter, const Coord3D& pt) const
{
	switch(m_type)
	{
		case GEOMETRY_SPHERE:
		case GEOMETRY_CYLINDER:
		{
			Real dx = pt.x - geomCenter.x;
			Real dy = pt.y - geomCenter.y;
			Real radius = WWMath::Sqrt(sqr(dx) + sqr(dy));
			return (radius <= m_majorRadius);
			break;
		}

		case GEOMETRY_BOX:
		{
			return isWithin(geomCenter.x - m_majorRadius, pt.x, geomCenter.x + m_majorRadius) &&
							isWithin(geomCenter.y - m_minorRadius, pt.y, geomCenter.y + m_minorRadius);
		}
	};
	return false;
}

//=============================================================================
void GeometryInfo::makeRandomOffsetWithinFootprint(Coord3D& pt) const
{
	switch(m_type)
	{
		case GEOMETRY_SPHERE:
		case GEOMETRY_CYLINDER:
		{
#if 1
			// this is a better technique than the more obvious radius-and-angle
			// one, below, because the latter tends to clump more towards the center.
			Real maxDistSqr = sqr(m_majorRadius);
			Real distSqr;
			do
			{
				pt.x = GameLogicRandomValueReal(-m_majorRadius, m_majorRadius);
				pt.y = GameLogicRandomValueReal(-m_majorRadius, m_majorRadius);
				pt.z = 0.0f;
				distSqr = sqr(pt.x) + sqr(pt.y);
			} while (distSqr > maxDistSqr);
#else
			Real radius = GameLogicRandomValueReal(0.0f, m_boundingCircleRadius);
			Real angle = GameLogicRandomValueReal(-PI, PI);
			pt.x = radius * Cos(angle);
			pt.y = radius * Sin(angle);
			pt.z = 0.0f;
#endif
			break;
		}

		case GEOMETRY_BOX:
		{
			pt.x = GameLogicRandomValueReal(-m_majorRadius, m_majorRadius);
			pt.y = GameLogicRandomValueReal(-m_minorRadius, m_minorRadius);
			pt.z = 0.0f;
			break;
		}
	};
}

//=============================================================================
void GeometryInfo::makeRandomOffsetOnPerimeter(Coord3D& pt) const
{
	switch(m_type)
	{
		case GEOMETRY_SPHERE:
		case GEOMETRY_CYLINDER:
		{
			DEBUG_CRASH( ("GeometryInfo::makeRandomOffsetOnPerimeter() not implemented for SPHERE or CYLINDER extents. Using position.") );

			//Kris: Did not have time nor need to support non-box extents. I added this feature for script placement
			//      of boobytraps.
			pt.x = 0.0f;
			pt.y = 0.0f;
			break;
		}

		case GEOMETRY_BOX:
		{
			if( GameLogicRandomValueReal( 0.0f, 1.0f ) < 0.5f )
			{
				//Pick random point on x axis.
				pt.x = GameLogicRandomValueReal(-m_majorRadius, m_majorRadius);

				//Min or max the y axis value
				if( GameLogicRandomValueReal( 0.0f, 1.0f ) < 0.5f )
					pt.y = -m_minorRadius;
				else
					pt.y = m_minorRadius;
			}
			else
			{
				//Pick random point on y axis.
				pt.y = GameLogicRandomValueReal(-m_minorRadius, m_minorRadius);

				//Min or max the x axis value
				if( GameLogicRandomValueReal( 0.0f, 1.0f ) < 0.5f )
					pt.x = -m_majorRadius;
				else
					pt.x = m_majorRadius;
			}
			pt.z = 0.0f;
			break;
		}
	};
}

//=============================================================================
Real GeometryInfo::getFootprintArea() const
{
	switch(m_type)
	{
		case GEOMETRY_SPHERE:
		case GEOMETRY_CYLINDER:
		{
			return PI * sqr(m_boundingCircleRadius);
		}

		case GEOMETRY_BOX:
		{
			return 4.0f * m_majorRadius * m_minorRadius;
		}
	};

	DEBUG_CRASH(("should never get here"));
	return 0.0f;
}

//=============================================================================
void GeometryInfo::calcBoundingStuff()
{
	switch(m_type)
	{
		case GEOMETRY_SPHERE:
		{
			m_boundingSphereRadius = m_majorRadius;
			m_boundingCircleRadius = m_majorRadius;
			break;
		}
		case GEOMETRY_CYLINDER:
		{
			m_boundingCircleRadius = m_majorRadius;

			m_boundingSphereRadius = m_height*0.5;
			if (m_boundingSphereRadius < m_majorRadius)
				m_boundingSphereRadius = m_majorRadius;
			break;
		}

		case GEOMETRY_BOX:
		{
			m_boundingCircleRadius = WWMath::Sqrt(sqr(m_majorRadius) + sqr(m_minorRadius));
			m_boundingSphereRadius = WWMath::Sqrt(sqr(m_majorRadius) + sqr(m_minorRadius) + sqr(m_height*0.5));
			break;
		}
	};
}

#if defined(RTS_DEBUG)
//=============================================================================
void GeometryInfo::tweakExtents(ExtentModType extentModType, Real extentModAmount)
{
	switch(extentModType)
	{
		case EXTENTMOD_HEIGHT:
			m_height += extentModAmount;
			break;
		case EXTENTMOD_MAJOR:
			m_majorRadius += extentModAmount;
			break;
		case EXTENTMOD_MINOR:
			m_minorRadius += extentModAmount;
			break;
		case EXTENTMOD_TYPE:
			m_type = (GeometryType)((m_type + ((extentModType == EXTENTMOD_TYPE)?1:0)) % GEOMETRY_NUM_TYPES);
			break;
	}
	m_isSmall = false;
	calcBoundingStuff();
}
#endif

#if defined(RTS_DEBUG)
//=============================================================================
AsciiString GeometryInfo::getDescriptiveString() const
{
	AsciiString msg;
	msg.format("%d/%d(%g %g %g)", (Int)m_type, (Int)m_isSmall, m_majorRadius, m_minorRadius, m_height);
	return msg;
}
#endif

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void GeometryInfo::crc( Xfer *xfer )
{

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void GeometryInfo::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// type
	xfer->xferUser( &m_type, sizeof( GeometryType ) );

	// is small
	xfer->xferBool( &m_isSmall );

	// height
	xfer->xferReal( &m_height );

	// major radius
	xfer->xferReal( &m_majorRadius );

	// minor radius
	xfer->xferReal( &m_minorRadius );

	// bouncing circle radius
	xfer->xferReal( &m_boundingCircleRadius );

	// bounding sphere radius
	xfer->xferReal( &m_boundingSphereRadius );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void GeometryInfo::loadPostProcess()
{

}
```

---

## 4. Architect Notes & Existing Rust Helpers

1. **Exact Imports Available in `zh_common`:**
   - `use crate::math::{gm_atan2f, gm_cosf, gm_sinf, gm_sqrt, gm_sqrtf, clamp_f32, Coord3D, Region2D, PI};`
   - `use crate::random_value::PrngState;` (`rng.get_game_logic_random_value_real(lo, hi)`)
   - `use crate::types::{Bool, Int, Real, XferVersion, FALSE, TRUE};`
   - `use crate::xfer::{Snapshot, Xfer, XferError};`
2. **Critical C++ Floating-Point & Constant Details (`Geometry.cpp`):**
   - **Exact `PI` constant (`BaseMacros.h:34`):** Use `crate::math::PI` (`3.14159265359f32`), **not** `std::f32::consts::PI` or `WWMATH_PI`.
   - **Implicit `f64` promotion in `calcBoundingStuff()` (`Geometry.cpp:501, 510`):**
     - `GEOMETRY_CYLINDER` line 501 writes `m_boundingSphereRadius = m_height*0.5;`, where `0.5` is a `double` (`f64`), so `let half_h = (self.height as f64) * 0.5; self.bounding_sphere_radius = half_h as Real;`.
     - `GEOMETRY_BOX` line 510 writes `WWMath::Sqrt(sqr(m_majorRadius) + sqr(m_minorRadius) + sqr(m_height*0.5))`. Because `m_height * 0.5` is `f64`, C++ promotes the sum to `f64` and invokes **`WWMath::Sqrt(double)` (`gm_sqrt(f64)`)**, then narrows to `f32`:
       `let sum_d = ((self.major_radius * self.major_radius + self.minor_radius * self.minor_radius) as f64) + half_h * half_h; self.bounding_sphere_radius = gm_sqrt(sum_d) as Real;`
     - All other `WWMath::Sqrt(...)` calls (`calcPitches`, `clipPointToFootprint`, `isPointInFootprint`, and `GEOMETRY_BOX` `boundingCircleRadius`) take `f32` and call **`gm_sqrtf(f32)`**.
   - **No `mul_add` / no `std` float methods:** Use only `gm_*` functions to stay clean under Gate V1 (`DET003`, `DET004`).
   - **PRNG parameter:** `make_random_offset_within_footprint` and `make_random_offset_on_perimeter` accept `rng: &mut PrngState` so tests remain deterministic and parallel-safe.
   - **Binary `Snapshot` / `Xfer`:** `GeometryInfo::crc` is empty; `GeometryInfo::xfer` transfers `version (1)`, `m_type` via `xfer_enum_i32`, `m_isSmall`, `m_height`, `m_majorRadius`, `m_minorRadius`, `m_boundingCircleRadius`, `m_boundingSphereRadius`.

