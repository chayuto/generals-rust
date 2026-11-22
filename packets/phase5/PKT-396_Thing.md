# Translation Packet `PKT-396`: `Thing`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-396`
- **Work Package:** `WP-5.5` (Phase 5)
- **Complexity & Assigned Tier:** Class `C4` -> `tier:3-architect`
- **Environment Access:** `env:hybrid-ci`
- **FFI Seam Pattern:** Core Spine
- **Allowed Rust Output File:** `crates/zh_logic/src/object/thing.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (16):**
  - `Common/GameMemory.h`
  - `Common/GlobalData.h`
  - `Common/KindOf.h`
  - `Common/NameKeyGenerator.h`
  - `Common/Override.h`
  - `Common/PerfTimer.h`
  - `Common/Player.h`
  - `Common/PlayerList.h`
  - `Common/Team.h`
  - `Common/Thing.h`
  - `Common/ThingFactory.h`
  - `Common/ThingTemplate.h`
  - `GameLogic/TerrainLogic.h`
  - `Lib/trig.h`
  - `PreRTS.h`
  - `WWMath/matrix3d.h`
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

### `GeneralsMD/Code/GameEngine/Include/Common/Thing.h`
```cpp
// FILE: Thing.h //////////////////////////////////////////////////////////////
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
// File name:  Thing.h
//
// Created:    Colin Day, May 2001
//
// Desc:       Things are the base class for objects and drawables, objects
//						 are logic side representations while drawables are client
//						 side.  Common data will be held in the Thing defined here
//						 and systems that need to work with both of them will work with
//						 "Things"
//
//-----------------------------------------------------------------------------

#pragma once

//-----------------------------------------------------------------------------

//-----------------------------------------------------------------------------
//           Includes
//-----------------------------------------------------------------------------
#include "Common/GameMemory.h"
#include "Common/KindOf.h"
#include "Common/Override.h"
#include "WWMath/matrix3d.h"							///< @todo Decide if we're keeping the WWMath libs (MSB)

//-----------------------------------------------------------------------------
//           Forward References
//-----------------------------------------------------------------------------

class Object;
class AIObject;
class Drawable;
class Team;
class ThingTemplate;

//-----------------------------------------------------------------------------
//           Type Defines
//-----------------------------------------------------------------------------

//=====================================
// class Thing
//=====================================
/** A thing is the common base class for objects and drawables.  It will
	* hold common information to both and systems that need to work with both
	* objects and drawables should work with things instead.  You can not
	* instantiate things, they are purely virtual */
//=====================================
class Thing : public MemoryPoolObject
{
	// note, it is explicitly OK to pass null for 'thing' here;
	// they will check for null and return null in these cases.
	friend Object *AsObject(Thing *thing) { return thing ? thing->asObjectMeth() : nullptr; }
	friend Drawable *AsDrawable(Thing *thing) { return thing ? thing->asDrawableMeth() : nullptr; }
	friend const Object *AsObject(const Thing *thing) { return thing ? thing->asObjectMeth() : nullptr; }
	friend const Drawable *AsDrawable(const Thing *thing) { return thing ? thing->asDrawableMeth() : nullptr; }

	MEMORY_POOL_GLUE_ABC(Thing)

public:

	Thing( const ThingTemplate *thingTemplate );

	/** Return the final override of this thing's template. */
	const ThingTemplate *getTemplate() const;

	// convenience method for patching isKindOf thru to template.
	Bool isKindOf(KindOfType t) const;
	Bool isKindOfMulti(const KindOfMaskType& mustBeSet, const KindOfMaskType& mustBeClear) const;
	Bool isAnyKindOf(const KindOfMaskType& anyKindOf) const;

	// physical properties
	void setPosition( const Coord3D *pos );

	/// the nice thing about this is that we don't have to recalc out cached terrain stuff.
	void setPositionZ( Real z );

	// note that calling this always orients the object straight up on the Z-axis,
	// or aligned with the terrain if we have the magic flag set.
	// don't want this behavior? then call setTransformMatrix instead.
	void setOrientation( Real angle );

	const Coord3D *getPosition() const { return &m_cachedPos; }
	Real getOrientation() const { return m_cachedAngle; }

	Bool isPositioned() const;

	const Coord3D *getUnitDirectionVector2D() const;
	void getUnitDirectionVector2D(Coord3D& dir) const;
	void getUnitDirectionVector3D(Coord3D& dir) const;

	Real getHeightAboveTerrain() const;
	Real getHeightAboveTerrainOrWater() const;

	Bool isAboveTerrain() const { return getHeightAboveTerrain() > 0.0f; }
	Bool isAboveTerrainOrWater() const { return getHeightAboveTerrainOrWater() > 0.0f; }

 	/** Ground vehicles moving down a slope will get slightly above the terrain.
	If we treat this as airborne, then they slide down slopes.  This checks whether
	they are high enough that we should let them act like they're flying. jba. */
	Bool isSignificantlyAboveTerrain() const ;

	void convertBonePosToWorldPos(const Coord3D* bonePos, const Matrix3D* boneTransform, Coord3D* worldPos, Matrix3D* worldTransform) const;

	void setTransformMatrix( const Matrix3D *mx );												///< set the world transformation matrix
	const Matrix3D* getTransformMatrix() const { return &m_transform; }		///< return the world transformation matrix

	void transformPoint( const Coord3D *in, Coord3D *out );								///< transform this point using the m_transform matrix of this thing

protected:

	// Virtual method since objects can be on bridges and need to calculate height above terrain differently.
	virtual Real calculateHeightAboveTerrain() const;		// Calculates the actual height above terrain.  Doesn't use cache.

	virtual Object *asObjectMeth() { return nullptr; }
	virtual Drawable *asDrawableMeth() { return nullptr; }
	virtual const Object *asObjectMeth() const { return nullptr; }
	virtual const Drawable *asDrawableMeth() const { return nullptr; }

	virtual void reactToTransformChange(const Matrix3D* oldMtx, const Coord3D* oldPos, Real oldAngle) = 0;

private:

	// note that it is declared 'const' -- the assumption being that
	// since ThingTemplates are shared between many, many Things, the Thing
	// should never be able to change it.
	OVERRIDE<ThingTemplate> m_template;	///< reference back to template database
#if defined(RTS_DEBUG)
	AsciiString m_templateName;
#endif
	/*
		yes, private; it's important that some of these only be modified
		by going thru the right methods, thus we make 'em private to enforce this.

		note that "m_transform" is the true description of location; the other fields
		(pos, angle, etc) are all simply cached values used for efficiency and convenience.
		you should NEVER modify them directly, because that won't change anything!
	*/
	Matrix3D m_transform;									///< the 3D orientation and position of this Thing

	enum
	{
		VALID_DIRVECTOR = 0x01,
		VALID_ALTITUDE_TERRAIN = 0x02,
		VALID_ALTITUDE_SEALEVEL = 0x04
	};

	mutable Coord3D		m_cachedPos;												///< position of thing
	mutable Real			m_cachedAngle;											///< orientation of thing
	mutable Coord3D		m_cachedDirVector;									///< unit direction vector
	mutable Real			m_cachedAltitudeAboveTerrain;
	mutable Real			m_cachedAltitudeAboveTerrainOrWater;
	mutable Int				m_cacheFlags;

};


//-----------------------------------------------------------------------------
//           Externals
//-----------------------------------------------------------------------------
```

### `GeneralsMD/Code/GameEngine/Source/Common/Thing/Thing.cpp`
```cpp
// FILE: Thing.cpp ////////////////////////////////////////////////////////////
// Created:   Colin Day, May 2001
//
// Desc:      Things are the base class for objects and drawables, objects
//						are logic side representations while drawables are client
//						side.  Common data will be held in the Thing defined here
//						and systems that need to work with both of them will work with
//						"Things"
//
//-----------------------------------------------------------------------------
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/PerfTimer.h"
#include "Common/Thing.h"
#include "Common/ThingTemplate.h"
#include "Common/ThingFactory.h"
#include "Common/GlobalData.h"
#include "Common/NameKeyGenerator.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/Team.h"
#include "Lib/trig.h"
#include "GameLogic/TerrainLogic.h"


static constexpr const Real InitialThingPosX = 0.0f;
static constexpr const Real InitialThingPosY = 0.0f;

//=============================================================================
/** Constructor */
//=============================================================================
Thing::Thing( const ThingTemplate *thingTemplate )
{
	// sanity
	if( thingTemplate == nullptr )
	{

		// cannot create thing without template
		DEBUG_CRASH(( "no template" ));
		return;

	}

	m_template = thingTemplate;
#if defined(RTS_DEBUG)
	m_templateName = thingTemplate->getName();
#endif
	m_transform.Make_Identity();
	m_cachedPos.x = InitialThingPosX;
	m_cachedPos.y = InitialThingPosY;
	m_cachedPos.z = 0.0f;
	m_cachedAngle = 0.0f;
	m_cachedDirVector.zero();
	m_cachedAltitudeAboveTerrain = 0;
	m_cachedAltitudeAboveTerrainOrWater = 0;
	m_cacheFlags = 0;

}

//=============================================================================
/** Destructor */
//=============================================================================
Thing::~Thing()
{
}

//DECLARE_PERF_TIMER(ThingMatrixStuff)

//=============================================================================
const ThingTemplate *Thing::getTemplate() const
{
	return m_template;
}

//=============================================================================
Bool Thing::isPositioned() const
{
	return m_cachedPos.x != InitialThingPosX || m_cachedPos.y != InitialThingPosY;
}

//=============================================================================
const Coord3D* Thing::getUnitDirectionVector2D() const
{
	//USE_PERF_TIMER(ThingMatrixStuff)
	if (!(m_cacheFlags & VALID_DIRVECTOR))
	{
		Real angle = getOrientation();
		m_cachedDirVector.x = Cos( angle );
		m_cachedDirVector.y = Sin( angle );
		m_cachedDirVector.z = 0;
		m_cacheFlags |= VALID_DIRVECTOR;
	}

	return &m_cachedDirVector;
}

//=============================================================================
void Thing::getUnitDirectionVector2D(Coord3D& dir) const
{
	dir = *getUnitDirectionVector2D();
}

//=============================================================================
void Thing::getUnitDirectionVector3D(Coord3D& dir) const
{
	Vector3 vdir = m_transform.Get_X_Vector();
	vdir.Normalize();
	dir.x = vdir.X;
	dir.y = vdir.Y;
	dir.z = vdir.Z;
}

//=============================================================================
// the nice thing about this is that we don't have to recalc out cached terrain stuff.
void Thing::setPositionZ( Real z )
{
	//USE_PERF_TIMER(ThingMatrixStuff)
	if( !m_template->isKindOf( KINDOF_STICK_TO_TERRAIN_SLOPE) )
	{
		Real oldAngle = m_cachedAngle;
		Coord3D oldPos = m_cachedPos;
		Matrix3D oldMtx = m_transform;

		m_transform.Set_Z_Translation( z );
		m_cachedPos.z = z;

		if (m_cacheFlags & VALID_ALTITUDE_TERRAIN)
		{
			m_cachedAltitudeAboveTerrain += (z - oldPos.z);
		}
		if (m_cacheFlags & VALID_ALTITUDE_SEALEVEL)
		{
			m_cachedAltitudeAboveTerrainOrWater += (z - oldPos.z);
		}

		reactToTransformChange(&oldMtx, &oldPos, oldAngle);
	}
	else
	{
		Matrix3D mtx;
		const Bool stickToGround = true;	// yes, set the "z" pos
		Coord3D pos = m_cachedPos;
		pos.z = z;
		TheTerrainLogic->alignOnTerrain(getOrientation(), pos, stickToGround, mtx );
		setTransformMatrix(&mtx);
	}
	DEBUG_ASSERTCRASH(!(_isnan(getPosition()->x) || _isnan(getPosition()->y) || _isnan(getPosition()->z)), ("Drawable/Object position NAN! '%s'", m_template->getName().str() ));
}

//=============================================================================
void Thing::setPosition( const Coord3D *pos )
{
	//USE_PERF_TIMER(ThingMatrixStuff)
	if( !m_template->isKindOf( KINDOF_STICK_TO_TERRAIN_SLOPE) )
	{
		Real oldAngle = m_cachedAngle;
		Coord3D oldPos = m_cachedPos;
		Matrix3D oldMtx = m_transform;

		//DEBUG_ASSERTCRASH(!(_isnan(pos->x) || _isnan(pos->y) || _isnan(pos->z)), ("Drawable/Object position NAN! '%s'", m_template->getName().str() ));
		m_transform.Set_X_Translation( pos->x );
		m_transform.Set_Y_Translation( pos->y );
		m_transform.Set_Z_Translation( pos->z );
		m_cachedPos = *pos;
		m_cacheFlags &= ~(VALID_ALTITUDE_TERRAIN | VALID_ALTITUDE_SEALEVEL);	// but don't clear the dir flags.

		reactToTransformChange(&oldMtx, &oldPos, oldAngle);
	}
	else
	{
		Matrix3D mtx;
		const Bool stickToGround = true;	// yes, set the "z" pos
		TheTerrainLogic->alignOnTerrain(getOrientation(), *pos, stickToGround, mtx );
		setTransformMatrix(&mtx);
	}
	DEBUG_ASSERTCRASH(!(_isnan(getPosition()->x) || _isnan(getPosition()->y) || _isnan(getPosition()->z)), ("Drawable/Object position NAN! '%s'", m_template->getName().str() ));
}

//=============================================================================
void Thing::setOrientation( Real angle )
{
	//USE_PERF_TIMER(ThingMatrixStuff)
	Coord3D u, x, y, z, pos;

	// setOrientation always forces us straight up in the Z axis,
	// or aligned with the terrain if we have the magic flag set.
	// don't want this? call setTransformMatrix instead.

	Real oldAngle = m_cachedAngle;
	Coord3D oldPos = m_cachedPos;
	Matrix3D oldMtx = m_transform;

	pos.x = m_transform.Get_X_Translation();
	pos.y = m_transform.Get_Y_Translation();
	pos.z = m_transform.Get_Z_Translation();
	if( m_template->isKindOf( KINDOF_STICK_TO_TERRAIN_SLOPE) )
	{
		Matrix3D mtx;
		const Bool stickToGround = true;	// yes, set the "z" pos
		TheTerrainLogic->alignOnTerrain(angle, pos, stickToGround, m_transform );
	}
	else
	{
		z.x = 0.0f;
		z.y = 0.0f;
		z.z = 1.0f;

		u.x = Cos(angle);
		u.y = Sin(angle);
		u.z = 0.0f;

		y.crossProduct( z, u, y );
		x.crossProduct( y, z, x );

		m_transform.Set(  x.x, y.x, z.x, pos.x,
											x.y, y.y, z.y, pos.y,
											x.z, y.z, z.z, pos.z );
	}

	//DEBUG_ASSERTCRASH(-PI <= angle && angle <= PI, ("Please pass only normalized (-PI..PI) angles to setOrientation (%f).", angle));
	m_cachedAngle = normalizeAngle(angle);
	m_cachedPos = pos;
	m_cacheFlags &= ~VALID_DIRVECTOR;	// but don't clear the altitude flags.

	reactToTransformChange(&oldMtx, &oldPos, oldAngle);
	DEBUG_ASSERTCRASH(!(_isnan(getPosition()->x) || _isnan(getPosition()->y) || _isnan(getPosition()->z)), ("Drawable/Object position NAN! '%s'", m_template->getName().str() ));
}

//=============================================================================
/** Set the world transformation matrix */
//=============================================================================
void Thing::setTransformMatrix( const Matrix3D *mx )
{
	//USE_PERF_TIMER(ThingMatrixStuff)
	Real oldAngle = m_cachedAngle;
	Coord3D oldPos = m_cachedPos;
	Matrix3D oldMtx = m_transform;

	m_transform = *mx;
	m_cachedPos.x = m_transform.Get_X_Translation();
	m_cachedPos.y = m_transform.Get_Y_Translation();
	m_cachedPos.z = m_transform.Get_Z_Translation();
	m_cachedAngle = m_transform.Get_Z_Rotation();
	m_cacheFlags = 0;

	reactToTransformChange(&oldMtx, &oldPos, oldAngle);
	DEBUG_ASSERTCRASH(!(_isnan(getPosition()->x) || _isnan(getPosition()->y) || _isnan(getPosition()->z)), ("Drawable/Object position NAN! '%s'", m_template->getName().str() ));
}

//-------------------------------------------------------------------------------------------------
Bool Thing::isKindOf(KindOfType t) const
{
	return getTemplate()->isKindOf(t);
}

//-------------------------------------------------------------------------------------------------
Bool Thing::isKindOfMulti(const KindOfMaskType& mustBeSet, const KindOfMaskType& mustBeClear) const
{
	return getTemplate()->isKindOfMulti(mustBeSet, mustBeClear);
}

// ------------------------------------------------------------------------------------------------
Bool Thing::isAnyKindOf( const KindOfMaskType& anyKindOf ) const
{
	return getTemplate()->isAnyKindOf( anyKindOf );
}

// ------------------------------------------------------------------------------------------------
Real Thing::calculateHeightAboveTerrain() const
{
	//USE_PERF_TIMER(ThingMatrixStuff)
	const Coord3D* pos = getPosition();
	Real terrainZ = TheTerrainLogic->getGroundHeight( pos->x, pos->y );
	Real myZ = pos->z;
	return myZ - terrainZ;
}

//-------------------------------------------------------------------------------------------------
Real Thing::getHeightAboveTerrain() const
{
	if (!(m_cacheFlags & VALID_ALTITUDE_TERRAIN))
	{
		m_cachedAltitudeAboveTerrain = calculateHeightAboveTerrain();
		m_cacheFlags |= VALID_ALTITUDE_TERRAIN;
	}
	return m_cachedAltitudeAboveTerrain;
}

//-------------------------------------------------------------------------------------------------
Real Thing::getHeightAboveTerrainOrWater() const
{
	//USE_PERF_TIMER(ThingMatrixStuff)
	if (!(m_cacheFlags & VALID_ALTITUDE_SEALEVEL))
	{
		const Coord3D* pos = getPosition();
		Real waterZ;
		if (TheTerrainLogic->isUnderwater(pos->x, pos->y, &waterZ))
		{
			m_cachedAltitudeAboveTerrainOrWater = pos->z - waterZ;
		}
		else
		{
			m_cachedAltitudeAboveTerrainOrWater = getHeightAboveTerrain();
		}
		m_cacheFlags |= VALID_ALTITUDE_SEALEVEL;
	}
	return m_cachedAltitudeAboveTerrainOrWater;
}

//=============================================================================
/** If we treat this as airborne, then they slide down slopes.  This checks whether
they are high enough that we should let them act like they're flying. jba. */
//=============================================================================
Bool Thing::isSignificantlyAboveTerrain() const
{
	// If it's high enough that it will take more than 3 frames to return to the ground,
	// then it's significantly airborne.  jba
	return (getHeightAboveTerrain() > -(3*3)*TheGlobalData->m_gravity);
}


//-------------------------------------------------------------------------------------------------
void Thing::convertBonePosToWorldPos(const Coord3D* bonePos, const Matrix3D* boneTransform, Coord3D* worldPos, Matrix3D* worldTransform) const
{
	if (worldTransform)
	{
#ifdef ALLOW_TEMPORARIES
		*worldTransform = m_transform * (*boneTransform);
#else
		worldTransform->mul(m_transform, *boneTransform);
#endif
	}
	if (worldPos)
	{
		Vector3 vector;
		vector.X = bonePos->x;
		vector.Y = bonePos->y;
		vector.Z = bonePos->z;
		m_transform.Transform_Vector(m_transform, vector, &vector);
		worldPos->x = vector.X;
		worldPos->y = vector.Y;
		worldPos->z = vector.Z;
	}
}

// ------------------------------------------------------------------------------------------------
/** Push the 'in' parameter through our transformation matrix and store in 'out' */
// ------------------------------------------------------------------------------------------------
void Thing::transformPoint( const Coord3D *in, Coord3D *out )
{

	// sanity
	if( in == nullptr || out == nullptr )
		return;

	// for conversion
	Vector3 vectorIn;
	Vector3 vectorOut;

	///@ todo this is dumb and we should not have to convert types
	// convert to Vector3 datatypes
	vectorIn.X = in->x;
	vectorIn.Y = in->y;
	vectorIn.Z = in->z;

	// do the transform
	m_transform.Transform_Vector( m_transform, vectorIn, &vectorOut );

	// store converted vector in 'out'
	out->x = vectorOut.X;
	out->y = vectorOut.Y;
	out->z = vectorOut.Z;

}
```
