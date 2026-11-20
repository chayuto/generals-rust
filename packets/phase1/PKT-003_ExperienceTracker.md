# Translation Packet `PKT-003`: `ExperienceTracker`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-003`
- **Work Package:** `WP-1.2b` (Phase 1)
- **Complexity & Assigned Tier:** Class `C1` -> `tier:1a-lite`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 2: Handle Component
- **Allowed Rust Output File:** `crates/zh_logic/src/object/experience_tracker.rs` *(Do not modify files outside this boundary)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (10):**
  - `Common/GameCommon.h`
  - `Common/GameMemory.h`
  - `Common/GameType.h`
  - `Common/Snapshot.h`
  - `Common/ThingTemplate.h`
  - `Common/Xfer.h`
  - `GameLogic/ExperienceTracker.h`
  - `GameLogic/GameLogic.h`
  - `GameLogic/Object.h`
  - `PreRTS.h`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (9 calls — preserve order identically):**
  1. `xferInt( &m_currentExperience )`
  2. `xferUser( &m_currentLevel, sizeof( VeterancyLevel ) )`
  3. `xferBool(&m_isTrainable)`
  4. `xferVersion( &version, currentVersion )`
  5. `xferUser( &m_currentLevel, sizeof( VeterancyLevel ) )`
  6. `xferInt( &m_currentExperience )`
  7. `xferObjectID( &m_experienceSink )`
  8. `xferReal( &m_experienceScalar )`
  9. `xferBool(&m_isTrainable)`
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if !RETAIL_COMPATIBLE_CRC`
  - `#if RETAIL_COMPATIBLE_XFER_SAVE`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/ExperienceTracker.h`
```cpp
// FILE: ExperienceTracker.h //////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, February 2002
// Desc:   Keeps track of experience points so Veterance levels can be gained
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

#include "Common/GameCommon.h"
#include "Common/GameType.h"
#include "Common/GameMemory.h"
#include "Common/Snapshot.h"

class Object;

class ExperienceTracker : public MemoryPoolObject, public Snapshot
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(ExperienceTracker, "ExperienceTrackerPool" )
public:
	ExperienceTracker(Object *parent);

	VeterancyLevel getVeterancyLevel() const { return m_currentLevel; }			///< What level am I?
	Int getExperienceValue( const Object* killer ) const;										///< How much do give for being killed
	Int getCurrentExperience() const { return m_currentExperience; };	///< How much experience do I have at the moment?
	Bool isTrainable() const;																						///< Can I gain experience?
	void setTrainable(Bool trainable);																	///< Set whether I can gain experience
	void resetTrainable();																							///< Set to default trainable state from template
	Bool isAcceptingExperiencePoints() const;														///< Either I am trainable, or I have a Sink set up

	void setVeterancyLevel( VeterancyLevel newLevel, Bool provideFeedback = TRUE );						///< Set Level to this
	void setMinVeterancyLevel( VeterancyLevel newLevel, Bool provideFeedback = TRUE );					///< Set Level to AT LEAST this... if we are already >= this level, do nothing.
	void addExperiencePoints( Int experienceGain, Bool canScaleForBonus = TRUE );	///< Gain this many exp.
	Bool gainExpForLevel(Int levelsToGain, Bool canScaleForBonus = TRUE );			  ///< Gain enough exp to gain a level. return false if can't gain a level.
	Bool canGainExpForLevel(Int levelsToGain) const;															///< return same value as gainExpForLevel, but don't change anything
	void setExperienceAndLevel(Int experienceIn, Bool provideFeedback = TRUE );
	void setExperienceSink( ObjectID sink );											///< My experience actually goes to this person (loose couple)
	ObjectID getExperienceSink() const;

	Real getExperienceScalar() const { return m_experienceScalar; }
	void setExperienceScalar( Real scalar ) { m_experienceScalar = scalar; }

	// --------------- inherited from Snapshot interface --------------
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

private:
	Object*						m_parent;														///< Object I am owned by
	VeterancyLevel		m_currentLevel;											///< Level of experience
	Int								m_currentExperience;								///< Number of experience points
	ObjectID					m_experienceSink;										///< ID of object I have pledged my experience point gains to
	Real							m_experienceScalar;									///< Scales any experience gained by this multiplier.
	Bool							m_isTrainable;											///< Can I gain experience?
};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/ExperienceTracker.cpp`
```cpp
// FILE: ExperienceTracker.cpp //////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, February 2002
// Desc:   Keeps track of experience points so Veterance levels can be gained
///////////////////////////////////////////////////////////////////////////////////////////////////

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/Xfer.h"
#include "Common/ThingTemplate.h"
#include "GameLogic/ExperienceTracker.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"



//-------------------------------------------------------------------------------------------------
ExperienceTracker::ExperienceTracker(Object *parent) :
	m_parent(parent),
	m_currentLevel(LEVEL_REGULAR),
	m_experienceSink(INVALID_ID),
	m_experienceScalar( 1.0f ),
	m_currentExperience(0)
{
	resetTrainable();
}

//-------------------------------------------------------------------------------------------------
ExperienceTracker::~ExperienceTracker()
{
}

//-------------------------------------------------------------------------------------------------
Int ExperienceTracker::getExperienceValue( const Object* killer ) const
{
	// No experience for killing an ally, cheater.
	if( killer->getRelationship( m_parent ) == ALLIES )
		return 0;

	return m_parent->getTemplate()->getExperienceValue(m_currentLevel);
}

//-------------------------------------------------------------------------------------------------
Bool ExperienceTracker::isTrainable() const
{
	return m_isTrainable;
}

//-------------------------------------------------------------------------------------------------
void ExperienceTracker::setTrainable(Bool trainable)
{
	m_isTrainable = trainable;
}

//-------------------------------------------------------------------------------------------------
void ExperienceTracker::resetTrainable()
{
	m_isTrainable = m_parent->getTemplate()->isTrainable();
}

//-------------------------------------------------------------------------------------------------
Bool ExperienceTracker::isAcceptingExperiencePoints() const
{
	return isTrainable() || (m_experienceSink != INVALID_ID);
}

//-------------------------------------------------------------------------------------------------
void ExperienceTracker::setExperienceSink( ObjectID sink )
{
	m_experienceSink = sink;
}

//-------------------------------------------------------------------------------------------------
ObjectID ExperienceTracker::getExperienceSink() const
{
	return m_experienceSink;
}

//-------------------------------------------------------------------------------------------------
// Set Level to AT LEAST this... if we are already >= this level, do nothing.
void ExperienceTracker::setMinVeterancyLevel( VeterancyLevel newLevel, Bool provideFeedback )
{
	// This does not check for IsTrainable, because this function is for explicit setting,
	// so the setter is assumed to know what they are doing.  The game function
	// of addExperiencePoints cares about Trainability.
	if (m_currentLevel < newLevel)
	{
		VeterancyLevel oldLevel = m_currentLevel;
		m_currentLevel = newLevel;
		m_currentExperience = m_parent->getTemplate()->getExperienceRequired(m_currentLevel); //Minimum for this level
		if (m_parent)
			m_parent->onVeterancyLevelChanged( oldLevel, newLevel, provideFeedback );
	}
}

//-------------------------------------------------------------------------------------------------
void ExperienceTracker::setVeterancyLevel( VeterancyLevel newLevel, Bool provideFeedback )
{
	// This does not check for IsTrainable, because this function is for explicit setting,
	// so the setter is assumed to know what they are doing.  The game function
	// of addExperiencePoints cares about Trainability, if flagged thus.
	if (m_currentLevel != newLevel)
	{
		VeterancyLevel oldLevel = m_currentLevel;
		m_currentLevel = newLevel;
		m_currentExperience = m_parent->getTemplate()->getExperienceRequired(m_currentLevel); //Minimum for this level
		if (m_parent)
			m_parent->onVeterancyLevelChanged( oldLevel, newLevel, provideFeedback );
	}
}

//-------------------------------------------------------------------------------------------------
Bool ExperienceTracker::gainExpForLevel(Int levelsToGain, Bool canScaleForBonus)
{
	Int newLevel = (Int)m_currentLevel + levelsToGain;
	if (newLevel > LEVEL_LAST)
		newLevel = LEVEL_LAST;
	// gain what levels we can, even if we can't use 'em all
	if (newLevel > m_currentLevel)
	{
		Int experienceNeeded = m_parent->getTemplate()->getExperienceRequired(newLevel) - m_currentExperience;
		addExperiencePoints( experienceNeeded, canScaleForBonus );
		return true;
	}
	return false;
}

//-------------------------------------------------------------------------------------------------
Bool ExperienceTracker::canGainExpForLevel(Int levelsToGain) const
{
	Int newLevel = (Int)m_currentLevel + levelsToGain;
	// return true if we can gain levels, even if we can't gain ALL the levels requested
	if (newLevel > LEVEL_LAST)
		newLevel = LEVEL_LAST;
	return (newLevel > m_currentLevel);
}

//-------------------------------------------------------------------------------------------------
void ExperienceTracker::addExperiencePoints( Int experienceGain, Bool canScaleForBonus)
{
	if( m_experienceSink != INVALID_ID )
	{
		// I have been set up to give my experience to someone else
		Object *sinkPointer = TheGameLogic->findObjectByID( m_experienceSink );
		if( sinkPointer )
		{
			// Not a fatal failure if not valid, he died when I was in the air.
			sinkPointer->getExperienceTracker()->addExperiencePoints( experienceGain * m_experienceScalar, canScaleForBonus );
			return;
		}
	}

	if( !isTrainable() )
		return; //safety

	VeterancyLevel oldLevel = m_currentLevel;

	Int amountToGain = experienceGain;
	if ( canScaleForBonus )
		amountToGain *= m_experienceScalar;


	m_currentExperience += amountToGain;

	Int levelIndex = 0;
	while( ( (levelIndex + 1) < LEVEL_COUNT) &&
		m_currentExperience >= m_parent->getTemplate()->getExperienceRequired(levelIndex + 1)
		)
	{
		// If there is a higher level to qualify for, and I qualify for it, advance the index
		levelIndex++;
	}

	m_currentLevel = (VeterancyLevel)levelIndex;

	if( oldLevel != m_currentLevel )
	{
		// Edge trigger special level gain effects.
		m_parent->onVeterancyLevelChanged( oldLevel, m_currentLevel );
	}

}
//-------------------------------------------------------------------------------------------------
void ExperienceTracker::setExperienceAndLevel( Int experienceIn, Bool provideFeedback )
{
	if( m_experienceSink != INVALID_ID )
	{
		// I have been set up to give my experience to someone else
		Object *sinkPointer = TheGameLogic->findObjectByID( m_experienceSink );
		if( sinkPointer )
		{
			// Not a fatal failure if not valid, he died when I was in the air.
			sinkPointer->getExperienceTracker()->setExperienceAndLevel( experienceIn, provideFeedback );
			return;
		}
	}

	if( !isTrainable() )
		return; //safety

	VeterancyLevel oldLevel = m_currentLevel;

	m_currentExperience = experienceIn;

	Int levelIndex = 0;
	while( ( (levelIndex + 1) < LEVEL_COUNT) &&
		m_currentExperience >= m_parent->getTemplate()->getExperienceRequired(levelIndex + 1)
		)
	{
		// If there is a level to qualify for, and I qualify for it, advance the index
		levelIndex++;
	}

	m_currentLevel = (VeterancyLevel)levelIndex;

	if( oldLevel != m_currentLevel )
	{
		// Edge trigger special level gain effects.
		m_parent->onVeterancyLevelChanged( oldLevel, m_currentLevel, provideFeedback ); //<<== paradox! this may be a level lost!
	}

}

//-----------------------------------------------------------------------------
void ExperienceTracker::crc( Xfer *xfer )
{
	xfer->xferInt( &m_currentExperience );
	xfer->xferUser( &m_currentLevel, sizeof( VeterancyLevel ) );
#if !RETAIL_COMPATIBLE_CRC
	xfer->xferBool(&m_isTrainable);
#endif
}

//-----------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version
	* 2: TheSuperHackers @tweak Serialize m_isTrainable
	*/
// ----------------------------------------------------------------------------
void ExperienceTracker::xfer( Xfer *xfer )
{

	// version
#if RETAIL_COMPATIBLE_XFER_SAVE
	XferVersion currentVersion = 1;
#else
	XferVersion currentVersion = 2;
#endif
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// no need to save the m_parent pointer, it is connected on allocation time
	// m_parent

	// current level
	xfer->xferUser( &m_currentLevel, sizeof( VeterancyLevel ) );

	// current experience
	xfer->xferInt( &m_currentExperience );

	// experience sink
	xfer->xferObjectID( &m_experienceSink );

	// experience scalar
	xfer->xferReal( &m_experienceScalar );

	if (version >= 2)
		xfer->xferBool(&m_isTrainable);
}

//-----------------------------------------------------------------------------
void ExperienceTracker::loadPostProcess()
{

}
```
