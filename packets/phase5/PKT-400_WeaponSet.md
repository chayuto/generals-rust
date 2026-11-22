# Translation Packet `PKT-400`: `WeaponSet`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-400`
- **Work Package:** `WP-5.5` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:hybrid-ci`
- **FFI Seam Pattern:** Core Spine
- **Allowed Rust Output File:** `crates/zh_logic/src/object/weapon_set.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (24):**
  - `Common/BitFlagsIO.h`
  - `Common/GameType.h`
  - `Common/INI.h`
  - `Common/KindOf.h`
  - `Common/ModelState.h`
  - `Common/Player.h`
  - `Common/PlayerList.h`
  - `Common/Snapshot.h`
  - `Common/SparseMatchFinder.h`
  - `Common/ThingFactory.h`
  - `Common/ThingTemplate.h`
  - `GameLogic/AI.h`
  - `GameLogic/Damage.h`
  - `GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/ContainModule.h`
  - `GameLogic/Module/SpawnBehavior.h`
  - `GameLogic/Module/StealthUpdate.h`
  - `GameLogic/Object.h`
  - `GameLogic/Weapon.h`
  - `GameLogic/WeaponSet.h`
  - `GameLogic/WeaponSetFlags.h`
  - `GameLogic/WeaponSetType.h`
  - `Lib/BaseType.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (6):**
  - `"Conditions" => WeaponSetFlags::parseFromINI`
  - `"Weapon" => WeaponTemplateSet::parseWeapon`
  - `"AutoChooseSources" => WeaponTemplateSet::parseAutoChoose`
  - `"PreferredAgainst" => WeaponTemplateSet::parsePreferredAgainst`
  - `"ShareWeaponReloadTime" => INI::parseBool`
  - `"WeaponLockSharedAcrossSets" => INI::parseBool`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (12 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferAsciiString(&ttName)`
  3. `xferAsciiString(&ttName)`
  4. `xferBool(&hasWeaponInSlot)`
  5. `xferSnapshot(m_weapons[i])`
  6. `xferUser(&m_curWeapon, sizeof(m_curWeapon))`
  7. `xferUser(&m_curWeaponLockedStatus, sizeof(m_curWeaponLockedStatus))`
  8. `xferUnsignedInt(&m_filledWeaponSlotMask)`
  9. `xferInt(&m_totalAntiMask)`
  10. `xferUnsignedInt(&totalDamageTypeMask)`
  11. `xferBool(version >= 3 ? &m_hasPitchLimit : &m_hasDamageWeapon)`
  12. `xferBool(&m_hasDamageWeapon)`
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/WeaponSet.h`
```cpp
// WeaponSet.h

#pragma once

#include "Lib/BaseType.h"
#include "Common/GameType.h"
#include "Common/KindOf.h"
#include "Common/ModelState.h"
#include "Common/SparseMatchFinder.h"
#include "Common/Snapshot.h"
#include "GameLogic/Damage.h"

//-------------------------------------------------------------------------------------------------
class INI;
class Object;
class Weapon;
class WeaponTemplate;

enum CommandSourceType CPP_11(: Int);
enum DamageType CPP_11(: Int);

// for WeaponSetType. Part of detangling.
#include "GameLogic/WeaponSetType.h"

//-------------------------------------------------------------------------------------------------
// for WeaponSetFlags. Part of detangling.
#include "GameLogic/WeaponSetFlags.h"

#ifdef DEFINE_WEAPONSLOTTYPE_NAMES
static const char *const TheWeaponSlotTypeNames[] =
{
	"PRIMARY",
	"SECONDARY",
	"TERTIARY",

	nullptr
};
static_assert(ARRAY_SIZE(TheWeaponSlotTypeNames) == WEAPONSLOT_COUNT + 1, "Incorrect array size");

static const LookupListRec TheWeaponSlotTypeNamesLookupList[] =
{
	{ "PRIMARY",		PRIMARY_WEAPON },
	{ "SECONDARY",	SECONDARY_WEAPON },
	{ "TERTIARY",		TERTIARY_WEAPON },

	{ nullptr, 0	}
};
static_assert(ARRAY_SIZE(TheWeaponSlotTypeNamesLookupList) == WEAPONSLOT_COUNT + 1, "Incorrect array size");

#endif

//-------------------------------------------------------------------------------------------------
#ifdef DEFINE_WEAPONCONDITIONMAP

//Kris: I did not write this code, but I am adding comments to clarify it.
//When I ran into this, I discovered that it was grossly out of date. It wasn't
//clearly identified as a "lookup table", but hopefully now it makes sense. I've
//updated it as of May 2003 when I added RIDER1-8 conditions.

//Purpose: Whenever you change a weaponset, the model condition state associated with it
//will be properly set exclusively.
static const ModelConditionFlagType TheWeaponSetTypeToModelConditionTypeMap[] =
{
	/*WEAPONSET_VETERAN*/								MODELCONDITION_WEAPONSET_VETERAN,
	/*WEAPONSET_ELITE*/									MODELCONDITION_WEAPONSET_ELITE,
	/*WEAPONSET_HERO*/									MODELCONDITION_WEAPONSET_HERO,
	/*WEAPONSET_PLAYER_UPGRADE*/				MODELCONDITION_WEAPONSET_PLAYER_UPGRADE,
	/*WEAPONSET_CRATEUPGRADE_ONE*/			MODELCONDITION_WEAPONSET_CRATEUPGRADE_ONE,
	/*WEAPONSET_CRATEUPGRADE_TWO*/			MODELCONDITION_WEAPONSET_CRATEUPGRADE_TWO,
	/*WEAPONSET_VEHICLE_HIJACK*/				MODELCONDITION_INVALID,
	/*WEAPONSET_CARBOMB*/								MODELCONDITION_INVALID,
	/*WEAPONSET_MINE_CLEARING_DETAIL*/	MODELCONDITION_INVALID,
	/*WEAPONSET_RIDER1*/								MODELCONDITION_RIDER1,	//Added these for different riders, but feel free to use these for anything.
	/*WEAPONSET_RIDER2*/								MODELCONDITION_RIDER2,
	/*WEAPONSET_RIDER3*/								MODELCONDITION_RIDER3,
	/*WEAPONSET_RIDER4*/								MODELCONDITION_RIDER4,
	/*WEAPONSET_RIDER5*/								MODELCONDITION_RIDER5,
	/*WEAPONSET_RIDER6*/								MODELCONDITION_RIDER6,
	/*WEAPONSET_RIDER7*/								MODELCONDITION_RIDER7,
	/*WEAPONSET_RIDER8*/								MODELCONDITION_RIDER8,
};
static_assert(ARRAY_SIZE(TheWeaponSetTypeToModelConditionTypeMap) == WEAPONSET_COUNT, "Incorrect array size");
#endif

//-------------------------------------------------------------------------------------------------
enum WeaponSetConditionType CPP_11(: Int)
{
	WSF_INVALID = -1,

	WSF_NONE = 0,
	WSF_FIRING,
	WSF_BETWEEN,
	WSF_RELOADING,
	WSF_PREATTACK,

	WSF_COUNT
};

//-------------------------------------------------------------------------------------------------
class WeaponTemplateSet
{
private:
	const ThingTemplate*		m_thingTemplate;	// needed for save/load
	WeaponSetFlags					m_types;
	const WeaponTemplate*		m_template[WEAPONSLOT_COUNT];
	UnsignedInt							m_autoChooseMask[WEAPONSLOT_COUNT];
	KindOfMaskType					m_preferredAgainst[WEAPONSLOT_COUNT];
	Bool										m_isReloadTimeShared;
	Bool										m_isWeaponLockSharedAcrossSets; ///< A weapon set so similar that it is safe to hold locks across

	static void parseWeapon(INI* ini, void *instance, void *store, const void* userData);
	static void parseAutoChoose(INI* ini, void *instance, void *store, const void* userData);
	static void parsePreferredAgainst(INI* ini, void *instance, void *store, const void* userData);

public:
	inline WeaponTemplateSet()
	{
		clear();
	}

	const ThingTemplate* friend_getThingTemplate() const { return m_thingTemplate; }	// only for WeaponSet::xfer
	const WeaponSetFlags& friend_getWeaponSetFlags() const { return m_types; }	// only for WeaponSet::xfer

	void clear();
	void parseWeaponTemplateSet( INI* ini, const ThingTemplate* tt );
	Bool testWeaponSetFlag( WeaponSetType wst ) const;
	Bool isSharedReloadTime() const { return m_isReloadTimeShared; }
	Bool isWeaponLockSharedAcrossSets() const {return m_isWeaponLockSharedAcrossSets; }

	Bool hasAnyWeapons() const;
	inline const WeaponTemplate* getNth(WeaponSlotType n) const { return m_template[n]; }
	inline UnsignedInt getNthCommandSourceMask(WeaponSlotType n) const { return m_autoChooseMask[n]; }
	inline const KindOfMaskType& getNthPreferredAgainstMask(WeaponSlotType n) const { return m_preferredAgainst[n]; }

	inline Int getConditionsYesCount() const { return 1; }
	inline const WeaponSetFlags& getNthConditionsYes(Int i) const { return m_types; }
#if defined(RTS_DEBUG)
	inline AsciiString getDescription() const { return "ArmorTemplateSet"; }
#endif
};

//-------------------------------------------------------------------------------------------------
typedef std::vector<WeaponTemplateSet> WeaponTemplateSetVector;

//-------------------------------------------------------------------------------------------------
enum WeaponChoiceCriteria CPP_11(: Int)
{
	PREFER_MOST_DAMAGE,		///< choose the weapon that will do the most damage
	PREFER_LONGEST_RANGE	///< choose the weapon with the longest range (that will do nonzero damage)
};

//-------------------------------------------------------------------------------------------------
enum WeaponLockType CPP_11(: Int)
{
	NOT_LOCKED,							///< Weapon is not locked
	LOCKED_TEMPORARILY,			///< Weapon is locked until clip is empty, or current "attack" state exits
	LOCKED_PERMANENTLY			///< Weapon is locked until explicitly unlocked or lock is changed to another weapon
};

//-------------------------------------------------------------------------------------------------
enum CanAttackResult CPP_11(: Int)
{
	//Worst scenario to best scenario -- These must be done this way now!
	ATTACKRESULT_NOT_POSSIBLE,					//Can't possibly attack target.
	ATTACKRESULT_INVALID_SHOT,					//Not a clear shot
	ATTACKRESULT_POSSIBLE_AFTER_MOVING, //I can attack, but after moving closer.
	ATTACKRESULT_POSSIBLE,							//I can attack now.
};

//-------------------------------------------------------------------------------------------------
class WeaponSet : public Snapshot
{
private:
	const WeaponTemplateSet*	m_curWeaponTemplateSet;
	Weapon*										m_weapons[WEAPONSLOT_COUNT];
	WeaponSlotType						m_curWeapon;
	WeaponLockType						m_curWeaponLockedStatus;
	UnsignedInt								m_filledWeaponSlotMask;
	Int												m_totalAntiMask;						///< anti mask of all current weapons
	DamageTypeFlags						m_totalDamageTypeMask;			///< damagetype mask of all current weapons
	Bool											m_hasPitchLimit;
	Bool											m_hasDamageWeapon;

	Bool isAnyWithinTargetPitch(const Object* obj, const Object* victim) const;

protected:
	// snapshot methods
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

public:

	WeaponSet();
	~WeaponSet();

	void updateWeaponSet(const Object* obj);
	void reloadAllAmmo(const Object *obj, Bool now);
	Bool isOutOfAmmo() const;
	Bool hasAnyWeapon() const { return m_filledWeaponSlotMask != 0; }
	Bool hasAnyDamageWeapon() const { return m_hasDamageWeapon; }
	Bool hasWeaponToDealDamageType(DamageType typeToDeal) const { return m_totalDamageTypeMask.test(typeToDeal); }
	Bool hasSingleDamageType(DamageType typeToDeal) const { return (m_totalDamageTypeMask.test(typeToDeal) && (m_totalDamageTypeMask.count() == 1) ); }
	Bool isCurWeaponLocked() const { return m_curWeaponLockedStatus != NOT_LOCKED; }
	Weapon* getCurWeapon() { return m_weapons[m_curWeapon]; }
	const Weapon* getCurWeapon() const { return m_weapons[m_curWeapon]; }
	WeaponSlotType getCurWeaponSlot() const { return m_curWeapon; }
	Weapon* findWaypointFollowingCapableWeapon();
	const Weapon* findAmmoPipShowingWeapon() const;
	void weaponSetOnWeaponBonusChange(const Object *source);
	UnsignedInt getMostPercentReadyToFireAnyWeapon() const;
	inline UnsignedInt getNthCommandSourceMask( WeaponSlotType n ) const { return m_curWeaponTemplateSet ? m_curWeaponTemplateSet->getNthCommandSourceMask( n ) : 0; }

	Bool setWeaponLock( WeaponSlotType weaponSlot, WeaponLockType lockType );
	void releaseWeaponLock(WeaponLockType lockType);
	Bool isSharedReloadTime() const;

	//When an AIAttackState is over, it needs to clean up any weapons that might be in leech range mode
	//or else those weapons will have unlimited range!
	void clearLeechRangeModeForAllWeapons();

	/**
		Determines if the unit has any weapon that could conceivably
		harm the victim. this does not take range, ammo, etc. into
		account, but immutable weapon properties, such as "can you
		target airborne victims".
	*/
	CanAttackResult getAbleToAttackSpecificObject( AbleToAttackType t, const Object* obj, const Object* victim, CommandSourceType commandSource, WeaponSlotType specificSlot = (WeaponSlotType)-1 ) const;

	//When calling this function, all conditions must be validated to the point where we have decided that we wish to attack the object (faction checks, etc).
	//Now, we are determining if the attack itself is able to be performed!
	CanAttackResult getAbleToUseWeaponAgainstTarget( AbleToAttackType attackType, const Object *source, const Object *victim, const Coord3D *pos, CommandSourceType commandSource, WeaponSlotType specificSlot = (WeaponSlotType)-1 ) const;

	/**
		Selects the best weapon for the given target, and sets it as the current weapon.
		If there is no weapon that can damage the target, false is returned (and the current-weapon is unchanged).
		Note that this DOES take weapon attack range into account.
	*/
	Bool chooseBestWeaponForTarget(const Object* obj, const Object* victim, WeaponChoiceCriteria criteria, CommandSourceType cmdSource);

	Weapon* getWeaponInWeaponSlot(WeaponSlotType wslot) const;

	static ModelConditionFlags getModelConditionForWeaponSlot(WeaponSlotType wslot, WeaponSetConditionType a);
};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/WeaponSet.cpp`
```cpp
// FILE: WeaponSet.cpp ///////////////////////////////////////////////////////////////////////////////
// Author: Steven Johnson, March 2002
// Desc:   Weapon descriptions
///////////////////////////////////////////////////////////////////////////////////////////////////


// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#define DEFINE_WEAPONSLOTTYPE_NAMES
#define DEFINE_COMMANDSOURCEMASK_NAMES

#include "GameLogic/WeaponSet.h"

#include "Common/INI.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/BitFlagsIO.h"

#include "GameLogic/AI.h"
#include "GameLogic/Object.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/StealthUpdate.h"
#include "GameLogic/Module/SpawnBehavior.h"

#include "GameLogic/Weapon.h"



///////////////////////////////////////////////////////////////////////////////////////////////////
// PUBLIC DATA ////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

template<>
const char* const WeaponSetFlags::s_bitNameList[] =
{
	"VETERAN",
	"ELITE",
	"HERO",
	"PLAYER_UPGRADE",
	"CRATEUPGRADE_ONE",
	"CRATEUPGRADE_TWO",
	"VEHICLE_HIJACK",
	"CARBOMB",
	"MINE_CLEARING_DETAIL",
	"WEAPON_RIDER1", //Kris: Added these for different combat-bike riders.
	"WEAPON_RIDER2",
	"WEAPON_RIDER3",
	"WEAPON_RIDER4",
	"WEAPON_RIDER5",
	"WEAPON_RIDER6",
	"WEAPON_RIDER7",
	"WEAPON_RIDER8",

	nullptr
};
static_assert(ARRAY_SIZE(WeaponSetFlags::s_bitNameList) == WeaponSetFlags::NumBits + 1, "Incorrect array size");

///////////////////////////////////////////////////////////////////////////////////////////////////
// PRIVATE DATA ///////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS ///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------------------
void WeaponTemplateSet::clear()
{
	m_isReloadTimeShared = false;
	m_isWeaponLockSharedAcrossSets = FALSE;
	m_types.clear();
	for (int i = 0; i < WEAPONSLOT_COUNT; ++i)
	{
		m_template[i] = nullptr;
		m_autoChooseMask[i] = 0xffffffff;					// by default, allow autochoosing from any CommandSource
		CLEAR_KINDOFMASK(m_preferredAgainst[i]);	// by default, weapon isn't preferred against anything in particular
	}
}

//-------------------------------------------------------------------------------------------------
Bool WeaponTemplateSet::hasAnyWeapons() const
{
	for (int i = 0; i < WEAPONSLOT_COUNT; ++i)
	{
		if (m_template[i])
			return true;
	}
	return false;
}

//-------------------------------------------------------------------------------------------------
void WeaponTemplateSet::parseWeapon(INI* ini, void *instance, void * /*store*/, const void* userData)
{
	WeaponTemplateSet* self = (WeaponTemplateSet*)instance;
	WeaponSlotType wslot = (WeaponSlotType)INI::scanIndexList(ini->getNextToken(), TheWeaponSlotTypeNames);
	INI::parseWeaponTemplate(ini, instance, &self->m_template[wslot], nullptr);
}

//-------------------------------------------------------------------------------------------------
void WeaponTemplateSet::parseAutoChoose(INI* ini, void *instance, void * /*store*/, const void* userData)
{
	WeaponTemplateSet* self = (WeaponTemplateSet*)instance;
	WeaponSlotType wslot = (WeaponSlotType)INI::scanIndexList(ini->getNextToken(), TheWeaponSlotTypeNames);
	INI::parseBitString32(ini, instance, &self->m_autoChooseMask[wslot], TheCommandSourceMaskNames);
}

//-------------------------------------------------------------------------------------------------
void WeaponTemplateSet::parsePreferredAgainst(INI* ini, void *instance, void * /*store*/, const void* userData)
{
	WeaponTemplateSet* self = (WeaponTemplateSet*)instance;
	WeaponSlotType wslot = (WeaponSlotType)INI::scanIndexList(ini->getNextToken(), TheWeaponSlotTypeNames);
	KindOfMaskType::parseFromINI(ini, instance, &self->m_preferredAgainst[wslot], nullptr);
}

//-------------------------------------------------------------------------------------------------
void WeaponTemplateSet::parseWeaponTemplateSet( INI* ini, const ThingTemplate* tt )
{
	static const FieldParse myFieldParse[] =
	{
		{ "Conditions", WeaponSetFlags::parseFromINI, nullptr, offsetof( WeaponTemplateSet, m_types ) },
		{ "Weapon",	WeaponTemplateSet::parseWeapon,	nullptr, 0 },
		{ "AutoChooseSources",	WeaponTemplateSet::parseAutoChoose, nullptr, 0 },
		{ "PreferredAgainst", WeaponTemplateSet::parsePreferredAgainst, nullptr, 0 },
		{ "ShareWeaponReloadTime", INI::parseBool, nullptr, offsetof( WeaponTemplateSet, m_isReloadTimeShared ) },
		{ "WeaponLockSharedAcrossSets", INI::parseBool, nullptr, offsetof( WeaponTemplateSet, m_isWeaponLockSharedAcrossSets ) },
		{ nullptr, nullptr, nullptr, 0 }
	};

	ini->initFromINI(this, myFieldParse);
	this->m_thingTemplate = tt;
}

//-------------------------------------------------------------------------------------------------
Bool WeaponTemplateSet::testWeaponSetFlag( WeaponSetType wst ) const
{
	return m_types.test( wst );
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------------------
WeaponSet::WeaponSet()
{
	m_curWeapon = PRIMARY_WEAPON;
	m_curWeaponLockedStatus = NOT_LOCKED;
	m_curWeaponTemplateSet = nullptr;
	m_filledWeaponSlotMask = 0;
	m_totalAntiMask = 0;
	m_totalDamageTypeMask.clear();
	m_hasPitchLimit = false;
	m_hasDamageWeapon = false;
	for (Int i = 0; i < WEAPONSLOT_COUNT; ++i)
		m_weapons[i] = nullptr;
}

//-------------------------------------------------------------------------------------------------
WeaponSet::~WeaponSet()
{
	for (Int i = 0; i < WEAPONSLOT_COUNT; ++i)
		deleteInstance(m_weapons[i]);
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void WeaponSet::crc( Xfer *xfer )
{

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version
	* 2: TheSuperHackers @tweak Upgrade damage type flags from integer to BitFlags for Generals.
	*    Zero Hour already had this at version 1.
	* 3: TheSuperHackers @bugfix bobtista 14/08/2026 Now serialize m_hasPitchLimit instead of m_hasDamageWeapon twice
	*/
// ------------------------------------------------------------------------------------------------
void WeaponSet::xfer( Xfer *xfer )
{
	// version
#if RETAIL_COMPATIBLE_XFER_SAVE
	const XferVersion currentVersion = 1;
#else
	const XferVersion currentVersion = 3;
#endif
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	if (xfer->getXferMode() == XFER_LOAD)
	{
		AsciiString ttName;
		WeaponSetFlags wsFlags;

		xfer->xferAsciiString(&ttName);
		wsFlags.xfer( xfer );

		if (ttName.isEmpty())
		{
			m_curWeaponTemplateSet = nullptr;
		}
		else
		{
			const ThingTemplate* tt = TheThingFactory->findTemplate(ttName);
			if (tt == nullptr)
				throw INI_INVALID_DATA;

			// TheSuperHackers @fix bobtista 27/01/2026 Use the same final override as Object.
			tt = static_cast<const ThingTemplate*>(tt->getFinalOverride());

			m_curWeaponTemplateSet = tt->findWeaponTemplateSet(wsFlags);
			if (m_curWeaponTemplateSet == nullptr)
				throw INI_INVALID_DATA;
		}
	}
	else if (xfer->getXferMode() == XFER_SAVE)
	{
		AsciiString ttName;				// leave 'em empty in case we're null
		WeaponSetFlags wsFlags;
		if (m_curWeaponTemplateSet != nullptr)
		{
			const ThingTemplate* tt = m_curWeaponTemplateSet->friend_getThingTemplate();
			if (tt == nullptr)
				throw INI_INVALID_DATA;

			ttName = tt->getName();
			wsFlags = m_curWeaponTemplateSet->friend_getWeaponSetFlags();
		}
		xfer->xferAsciiString(&ttName);
		wsFlags.xfer( xfer );
	}

	for (Int i = 0; i < WEAPONSLOT_COUNT; ++i)
	{
		Bool hasWeaponInSlot = (m_weapons[i] != nullptr);
		xfer->xferBool(&hasWeaponInSlot);
		if (hasWeaponInSlot)
		{
			if (xfer->getXferMode() == XFER_LOAD && m_weapons[i] == nullptr)
			{
				const WeaponTemplate* wt = m_curWeaponTemplateSet->getNth((WeaponSlotType)i);
				if (wt==nullptr) {
					DEBUG_CRASH(("xfer backwards compatibility code - old save file??? jba."));
					wt = m_curWeaponTemplateSet->getNth((WeaponSlotType)0);
				}
				m_weapons[i] = TheWeaponStore->allocateNewWeapon(wt, (WeaponSlotType)i);
			}
			xfer->xferSnapshot(m_weapons[i]);
		}
	}
	xfer->xferUser(&m_curWeapon, sizeof(m_curWeapon));
	xfer->xferUser(&m_curWeaponLockedStatus, sizeof(m_curWeaponLockedStatus));
	xfer->xferUnsignedInt(&m_filledWeaponSlotMask);
	xfer->xferInt(&m_totalAntiMask);

#if RTS_GENERALS
	if (version < 2)
	{
		UnsignedInt totalDamageTypeMask = m_totalDamageTypeMask.toUnsignedInt();
		xfer->xferUnsignedInt(&totalDamageTypeMask);
		m_totalDamageTypeMask = DamageTypeFlags(totalDamageTypeMask);
	}
#endif

	xfer->xferBool(version >= 3 ? &m_hasPitchLimit : &m_hasDamageWeapon);
	xfer->xferBool(&m_hasDamageWeapon);

#if RTS_GENERALS
	if (version >= 2)
#endif
	{
		m_totalDamageTypeMask.xfer(xfer);// BitSet has built in xfer
	}
}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void WeaponSet::loadPostProcess()
{

}

//-------------------------------------------------------------------------------------------------
void WeaponSet::updateWeaponSet(const Object* obj)
{
	const WeaponTemplateSet* set = obj->getTemplate()->findWeaponTemplateSet(obj->getWeaponSetFlags());
	DEBUG_ASSERTCRASH(set, ("findWeaponSet should never return null"));
	if (set && set != m_curWeaponTemplateSet)
	{
		if( ! set->isWeaponLockSharedAcrossSets() )
		{
			DEBUG_ASSERTLOG(!isCurWeaponLocked(), ("changing WeaponSet while Weapon is Locked... implicit unlock occurring!"));
			releaseWeaponLock(LOCKED_PERMANENTLY);	// release all locks. sorry!
			m_curWeapon = PRIMARY_WEAPON;
		}
		m_filledWeaponSlotMask = 0;
		m_totalAntiMask = 0;
		m_totalDamageTypeMask.clear();
		m_hasPitchLimit = false;
		m_hasDamageWeapon = false;
		for (Int i = WEAPONSLOT_COUNT - 1; i >= PRIMARY_WEAPON ; --i)
		{
			deleteInstance(m_weapons[i]);
			m_weapons[i] = nullptr;

			if (set->getNth((WeaponSlotType)i))
			{
				m_weapons[i] = TheWeaponStore->allocateNewWeapon(set->getNth((WeaponSlotType)i), (WeaponSlotType)i);
				m_weapons[i]->loadAmmoNow(obj);	// start 'em all with full clips.
				m_filledWeaponSlotMask |= (1 << i);
				m_totalAntiMask |= m_weapons[i]->getAntiMask();
				m_totalDamageTypeMask.set(m_weapons[i]->getDamageType());
				if (m_weapons[i]->isPitchLimited())
					m_hasPitchLimit = true;
				if (m_weapons[i]->isDamageWeapon())
					m_hasDamageWeapon = true;

				// no, do NOT do this; always start with the cur weapon being primary, even if there is no primary
				// weapon. this is by design, to allow us to have units that have only "spell" weapons and no
				// "normal" weapons. (srj)
				// m_curWeapon = (WeaponSlotType)i;
			}
		}
		m_curWeaponTemplateSet = set;
		//DEBUG_LOG(("WeaponSet::updateWeaponSet -- changed curweapon to %s",getCurWeapon()->getName().str()));
	}
}

//-------------------------------------------------------------------------------------------------
/*static*/ ModelConditionFlags WeaponSet::getModelConditionForWeaponSlot(WeaponSlotType wslot, WeaponSetConditionType a)
{
	static const ModelConditionFlagType Nothing[] = {
		MODELCONDITION_INVALID,
		MODELCONDITION_INVALID,
		MODELCONDITION_INVALID
	};
	static const ModelConditionFlagType Firing[] = {
		MODELCONDITION_FIRING_A,
		MODELCONDITION_FIRING_B,
		MODELCONDITION_FIRING_C
	};
	static const ModelConditionFlagType Betweening[] = {
		MODELCONDITION_BETWEEN_FIRING_SHOTS_A,
		MODELCONDITION_BETWEEN_FIRING_SHOTS_B,
		MODELCONDITION_BETWEEN_FIRING_SHOTS_C
	};
	static const ModelConditionFlagType Reloading[] = {
		MODELCONDITION_RELOADING_A,
		MODELCONDITION_RELOADING_B,
		MODELCONDITION_RELOADING_C
	};
	static const ModelConditionFlagType PreAttack[] = {
		MODELCONDITION_PREATTACK_A,
		MODELCONDITION_PREATTACK_B,
		MODELCONDITION_PREATTACK_C
	};
	static const ModelConditionFlagType* Lookup[] = {
		Nothing,
		Firing,
		Betweening,
		Reloading,
		PreAttack
	};
	static_assert(ARRAY_SIZE(Nothing) == WEAPONSLOT_COUNT, "Incorrect array size");
	static_assert(ARRAY_SIZE(Firing) == WEAPONSLOT_COUNT, "Incorrect array size");
	static_assert(ARRAY_SIZE(Betweening) == WEAPONSLOT_COUNT, "Incorrect array size");
	static_assert(ARRAY_SIZE(Reloading) == WEAPONSLOT_COUNT, "Incorrect array size");
	static_assert(ARRAY_SIZE(PreAttack) == WEAPONSLOT_COUNT, "Incorrect array size");
	static_assert(ARRAY_SIZE(Lookup) == WSF_COUNT, "Incorrect array size");

	ModelConditionFlags flags;	// defaults to all clear

	ModelConditionFlagType f = Lookup[a][wslot];
	if (f != MODELCONDITION_INVALID)
		flags.set(f);

	static const ModelConditionFlagType Using[] = {
		MODELCONDITION_USING_WEAPON_A,
		MODELCONDITION_USING_WEAPON_B,
		MODELCONDITION_USING_WEAPON_C
	};
	static_assert(ARRAY_SIZE(Using) == WEAPONSLOT_COUNT, "Incorrect array size");
	if (a != WSF_NONE)
		flags.set(Using[wslot]);

	return flags;
}

//-------------------------------------------------------------------------------------------------
static Int getVictimAntiMask(const Object* victim)
{
	if( victim->isKindOf( KINDOF_SMALL_MISSILE ) )
	{
		//All missiles are also projectiles!
		return WEAPON_ANTI_SMALL_MISSILE;
	}
	else if( victim->isKindOf( KINDOF_BALLISTIC_MISSILE ) )
	{
		return WEAPON_ANTI_BALLISTIC_MISSILE;
	}
	else if( victim->isKindOf( KINDOF_PROJECTILE ) )
	{
		return WEAPON_ANTI_PROJECTILE;
	}
	else if( victim->isKindOf( KINDOF_MINE ) || victim->isKindOf( KINDOF_DEMOTRAP ) )
	{
		return WEAPON_ANTI_MINE | WEAPON_ANTI_GROUND;
	}
	else if( victim->isAirborneTarget() )
	{
		if( victim->isKindOf( KINDOF_VEHICLE ) )
		{
			return WEAPON_ANTI_AIRBORNE_VEHICLE;
		}
		else if( victim->isKindOf( KINDOF_INFANTRY ) )
		{
			return WEAPON_ANTI_AIRBORNE_INFANTRY;
		}
		else if( victim->isKindOf( KINDOF_PARACHUTE ) )
		{
			return WEAPON_ANTI_PARACHUTE;
		}
		else if( !victim->isKindOf( KINDOF_UNATTACKABLE ) )
		{
			DEBUG_CRASH( ("Object %s is being targeted as airborne, but is not infantry, nor vehicle. Is this legit? -- tell Kris", victim->getTemplate()->getName().str() ) );
		}
		return 0;
	}
	else
	{
		return WEAPON_ANTI_GROUND;
	}
}

//-------------------------------------------------------------------------------------------------
void WeaponSet::weaponSetOnWeaponBonusChange(const Object *source)
{
	for( Int i = 0; i < WEAPONSLOT_COUNT;	i++ )
	{
		Weapon *weapon = m_weapons[ i ];
		if( weapon )
		{
			weapon->onWeaponBonusChange(source);
		}
	}
}

//-------------------------------------------------------------------------------------------------
Bool WeaponSet::isAnyWithinTargetPitch(const Object* obj, const Object* victim) const
{
	if (!m_hasPitchLimit)
		return true;

	for( Int i = 0; i < WEAPONSLOT_COUNT;	i++ )
	{
		const Weapon* weapon = m_weapons[ i ];
		if (weapon && weapon->isWithinTargetPitch(obj, victim))
		{
			return true;
		}
	}
	return false;
}

//-------------------------------------------------------------------------------------------------
CanAttackResult WeaponSet::getAbleToAttackSpecificObject( AbleToAttackType attackType, const Object* source, const Object* victim, CommandSourceType commandSource, WeaponSlotType specificSlot ) const
{

	// basic sanity checks.
	if (!source ||
			!victim ||
			source->isEffectivelyDead() ||
			victim->isEffectivelyDead() ||
			source->isDestroyed() ||
			victim->isDestroyed() ||
			victim == source)
		return ATTACKRESULT_NOT_POSSIBLE;

	Bool sameOwnerForceAttack = ((source->getControllingPlayer() == victim->getControllingPlayer()) && isForcedAttack(attackType));

// ----- examine VICTIM to determine legality of attack

	//
	// an "OBJECT_STATUS_MASKED" object is not really a valid target for attacking, this masked
	// status is an "override" that can be applied to objects that are otherwise totally
	// valid targets when we really really really don't want anything in the world
	// being able to attack stuff
	//
	if (victim->testStatus(OBJECT_STATUS_MASKED))
		return ATTACKRESULT_NOT_POSSIBLE;

	// And this is the KINDOF version, for things that aren't really game objects, but need to have "life"
	// so they can have lifetimeupdates and vision
	if (victim->isKindOf(KINDOF_UNATTACKABLE))
		return ATTACKRESULT_NOT_POSSIBLE;

	// this object is not currently auto-acquireable
	if (victim->testStatus(OBJECT_STATUS_NO_ATTACK_FROM_AI) && commandSource == CMD_FROM_AI)
		return ATTACKRESULT_NOT_POSSIBLE;

  Bool allowStealthToPreventAttacks = TRUE;
	if (source->testStatus(OBJECT_STATUS_IGNORING_STEALTH) || sameOwnerForceAttack)
		allowStealthToPreventAttacks = FALSE;
  if( isForcedAttack( attackType ) && victim->isKindOf( KINDOF_DISGUISER ) )
  {
    // force-attack allows you to attack disguised things, which also happen to be stealthed.
    // since we normally disallow attacking stealthed things (even via force-fire), we check
    // for disguised and explicitly ignore stealth in that case
		if( victim->testStatus( OBJECT_STATUS_DISGUISED ) )
  	  allowStealthToPreventAttacks = FALSE;
  }

	// If an object is stealthed and hasn't been detected yet, then it is not a valid target to fire
	// on.
	if (allowStealthToPreventAttacks &&
				victim->testStatus(OBJECT_STATUS_STEALTHED) &&
				!victim->testStatus(OBJECT_STATUS_DETECTED))
	{
		if( !victim->isKindOf( KINDOF_DISGUISER ) )
		{
			return ATTACKRESULT_NOT_POSSIBLE;
		}
		else
		{
			//Exception case -- don't return false if we are a bomb truck disguised as an enemy vehicle.
			StealthUpdate *update = victim->getStealth();
			if( update && update->isDisguised() )
			{
				Player *ourPlayer = source->getControllingPlayer();
				Player *otherPlayer = ThePlayerList->getNthPlayer( update->getDisguisedPlayerIndex() );
				if( ourPlayer && otherPlayer )
				{
					if( ourPlayer->getRelationship( otherPlayer->getDefaultTeam() ) != ENEMIES )
					{
						//Our stealthed & undetected object is disguised as a unit not perceived to be our enemy.
						return ATTACKRESULT_NOT_POSSIBLE;
					}
				}
			}
		}
	}

	// If the victim is fully fogged or fully shrouded, then we cannot attack them
	// GS -- Shroud only applies in decision making to Player owned objects and when the source is not from script
	// SRJ -- ignore shroud checks if mode is CONTINUED and the target is immobile, since presumably
	// we know where it is (and it's not going anywhere); this prevents wonky behavior if you have planes attack
	// a deshrouded thing, which becomes reshrouded before they get there (basically the planes circle once in order
	// to deshroud it again, and get nailed by AA fire in the meantime)

	// GS -- Fog no longer prevents attacking at all.  it prevents targeting in findClosestEnemy.  Leaving this
	// here commented out to show that shroud should not be checked.
//  const Player* sourceController = source->getControllingPlayer();
//	if( sourceController
 //   && sourceController->getPlayerType() == PLAYER_HUMAN
//		&& commandSource != CMD_FROM_SCRIPT
//		&& !(isContinuedAttack(attackType) && victim->isKindOf(KINDOF_IMMOBILE))
//		&&  victim->getShroudedStatus(sourceController->getPlayerIndex()) >= OBJECTSHROUD_FOGGED
//		)
//	{
//		return ATTACKRESULT_NOT_POSSIBLE;
//	}

	//
	// At least one of the following must be true:
	//
	// -- victim is our enemy
	// -- victim is a Bridge Tower (they are always neutral)
	// -- victim is a non-allied Mine
	// -- attack mode is force attack
	//
	// if none of the above is true, we can't attack. sorry!
	//
	Relationship r = source->getRelationship(victim);
	if (r != ENEMIES &&
			!isForcedAttack(attackType) &&
// GS			!victim->isKindOf( KINDOF_BRIDGE_TOWER ) && // Repairing bridges cut 12/12/02, so attacking is too
			!(victim->isKindOf( KINDOF_MINE ) && r != ALLIES))
	{
		//Only reject this if the command is from a player. If scripts or AI order attacks, they don't
		//care about relationships (and fixes broken scripts).
		if( commandSource == CMD_FROM_PLAYER && (!victim->testScriptStatusBit( OBJECT_STATUS_SCRIPT_TARGETABLE ) || r == ALLIES) )
		{
			//Unless the object has a map property that sets it to be targetable (and not allied), then give up.
			return ATTACKRESULT_NOT_POSSIBLE;
		}
	}

	// if the victim is contained within an enclosing container, it cannot be attacked directly
	const Object* victimsContainer = victim->getContainedBy();
	if (victimsContainer != nullptr && victimsContainer->getContain()->isEnclosingContainerFor(victim) == TRUE)
	{
		return ATTACKRESULT_NOT_POSSIBLE;
	}

	// check if the container has an apparent controller, that controller must be an enemy
	// (unless this is a force-attack)
	if (!isForcedAttack(attackType))
	{
		const ContainModuleInterface* victimContain = victim->getContain();
		if (victimContain)
		{
			const Player* victimApparentController = victimContain->getApparentControllingPlayer(source->getControllingPlayer());
			if (victimApparentController && source->getTeam()->getRelationship(victimApparentController->getDefaultTeam()) != ENEMIES)
			{
				//Only reject this if the command is from a player. If scripts or AI order attacks, they don't
				//care about relationships (and fixes broken scripts).
				if( commandSource == CMD_FROM_PLAYER && (!victim->testScriptStatusBit( OBJECT_STATUS_SCRIPT_TARGETABLE ) || r == ALLIES) )
				{
					//Unless the object has a map property that sets it to be targetable (and not allied), then give up.
					return ATTACKRESULT_NOT_POSSIBLE;
				}
			}
		}
	}

	//Check if the shot itself is valid!
	return getAbleToUseWeaponAgainstTarget( attackType, source, victim, victim->getPosition(), commandSource, specificSlot );
}

//-------------------------------------------------------------------------------------------------
//This is formerly the 2nd half of getAbleToAttackSpecificObject
//This function is responsible for determining if our object is physically capable of attacking the target and it
//supports both victim or position.
CanAttackResult WeaponSet::getAbleToUseWeaponAgainstTarget( AbleToAttackType attackType, const Object *source, const Object *victim, const Coord3D *pos, CommandSourceType commandSource, WeaponSlotType specificSlot ) const
{

	//First determine if we are attacking an object or the ground and get the
	//appropriate weapon anti mask.
	WeaponAntiMaskType targetAntiMask;
	if( victim )
	{
		//Attacking a specific object -- get the victim's anti weapon mask!
		targetAntiMask = (WeaponAntiMaskType)getVictimAntiMask( victim );

		//Make sure that the pos matches the victim!
		pos = victim->getPosition();
	}
	else
	{
		//Attacking the ground so this is obvious.
		targetAntiMask = WEAPON_ANTI_GROUND;
	}


	// Special Case test for turreted weapons on buildings... since they cannot move, they must be within range of target
	//Kris: Actually -- let's just do this for all immobile objects. If we can give it orders to attack, then we must check
	// the range anyways. This will also fix stinger soldiers.
	//Kris: ***NOTE*** KINDOF_SPAWNS_ARE_THE_WEAPONS was added here because initially I tried to get this to work by
	//      making stinger soldiers immobile. However, doing so prevents them from attacking because they can't turn without
	//      a locomotor! In anycase, the stinger site when ordered to attack will pass the spawn member closest to the target
	//      to this function for evaluation in it's place. So to get that code into this area, giving it the kindof bit will
	//      accomplish that. This is fine because only these two units have this set. Should that change in the future, this
	//      will need re-evaluation (can be solved with a new kindof).
	//
	// srj sez: contained soldiers are a lot like turrets in this sense. they should only attempt to acquire targets
	// within range, even if their container is mobile. otherwise they end up resetting their mood-check
	// times in odd ways in some cases. hence the getContainedBy() check.
	//Kris: Dec 28 -- Now we're testing this for ALL objects, but we only abort if the object is immobile. We'll use the
	//		  information later to determine if the unit must move before firing (cursor reasons).
	Bool withinAttackRange = FALSE;
	const Object *containedBy = source->getContainedBy();
	Bool hasAWeaponInRange = FALSE;
	Bool hasAWeapon				 = FALSE;
	for (Int slot = 0; slot < WEAPONSLOT_COUNT - 1; ++slot)
	{
		Weapon *weaponToTestForRange = m_weapons[ m_curWeapon ];
		if ( weaponToTestForRange )
		{
			hasAWeapon = TRUE;
			if ((m_totalAntiMask & targetAntiMask) == 0)//we don't care to check for this weapon
				continue;

			Bool handled = FALSE;
			ContainModuleInterface *contain = containedBy ? containedBy->getContain() : nullptr;
			if( contain && contain->isGarrisonable() && contain->isEnclosingContainerFor( source ))
			{
				// non enclosing garrison containers do not use firepoints. Lorenzen, 6/11/03
				//For contained things, we need to fake-move objects to the best garrison point in order
				//to get precise range checks.
				Coord3D targetPos = *pos;
				Coord3D goalPos;
				if( contain->calcBestGarrisonPosition( &goalPos, &targetPos) )
				{
					withinAttackRange = weaponToTestForRange->isSourceObjectWithGoalPositionWithinAttackRange( source, &goalPos, victim, &targetPos );
					handled = TRUE;
				}
			}
			else if( victim )
				withinAttackRange = weaponToTestForRange->isWithinAttackRange( source, victim );
			else
				withinAttackRange = weaponToTestForRange->isWithinAttackRange( source, pos );
			if( withinAttackRange )
			{
				hasAWeaponInRange = TRUE;
				break;
			}
		}
	}

	if( source->isKindOf( KINDOF_IMMOBILE ) || source->isKindOf( KINDOF_SPAWNS_ARE_THE_WEAPONS ) || containedBy )
	{
		if ( hasAWeapon && !hasAWeaponInRange && attackType != ATTACK_TUNNEL_NETWORK_GUARD )
			return ATTACKRESULT_INVALID_SHOT;

	}

	CanAttackResult okResult = withinAttackRange ? ATTACKRESULT_POSSIBLE : ATTACKRESULT_POSSIBLE_AFTER_MOVING;


// ----- things we examine about SOURCE to determine legality of attack

	if (hasAnyDamageWeapon())
	{
		if ((m_totalAntiMask & targetAntiMask) == 0)
			return ATTACKRESULT_INVALID_SHOT;

		//If we don't have a victim, we are force attacking a position. Because
		//of the weapon mask check preceding this, the attack is possible!
		if( !victim )
			return okResult;

		if (!isAnyWithinTargetPitch(source, victim))
			return ATTACKRESULT_INVALID_SHOT;

		Int first, last;
		if( isCurWeaponLocked() )
		{
			first = m_curWeapon;
			last = m_curWeapon;
		}
		else if( specificSlot != (WeaponSlotType)-1 )
		{
			first = specificSlot;
			last = specificSlot;
		}
		else
		{
			first = WEAPONSLOT_COUNT - 1;
			last = PRIMARY_WEAPON;
		}

		//Check each weapon, until we find one that can do damage (or fail)
		for( Int i = first; i >= last ; --i )
		{
			Weapon *weapon = m_weapons[ i ];
			if (weapon && weapon->estimateWeaponDamage( source, victim ))
			{
				//Kris: Aug 22, 2003
				//Surgical fix so Jarmen Kell doesn't get a targeting cursor on enemy vehicles unless he is in snipe mode.
				if( weapon->getDamageType() == DAMAGE_KILLPILOT && source->isKindOf( KINDOF_HERO ) && m_curWeapon == PRIMARY_WEAPON && specificSlot == (WeaponSlotType)-1 )
				{
					continue;
				}

				return okResult;
			}
		}
	}

	// Do a check to see if we have an occupied container (garrisoned building, transport that allows passengers to fire).
	ContainModuleInterface *contain = source->getContain();
	if (contain && contain->isPassengerAllowedToFire())
	{
		// Loop through each member and if just one of them can attack the specific target, then
		// we are good to go!
		const ContainedItemsList* items = contain->getContainedItemsList();
		if (items)
		{
			for (ContainedItemsList::const_iterator it = items->begin(); it != items->end(); ++it)
			{
				Object* garrisonedMember = *it;
				if( garrisonedMember->isAbleToAttack() )
				{
					CanAttackResult result = garrisonedMember->getAbleToUseWeaponAgainstTarget( attackType, victim, pos, commandSource );
					if( result == ATTACKRESULT_POSSIBLE || result == ATTACKRESULT_POSSIBLE_AFTER_MOVING )
					{
						return result;
					}
				}
			}
		}
	}


	// Do a check to see if we have a hive object that has slaved objects.
	SpawnBehaviorInterface* spawnInterface = source->getSpawnBehaviorInterface();
	if( spawnInterface &&
		spawnInterface->getCanAnySlavesUseWeaponAgainstTarget( attackType, victim, pos, commandSource ) == ATTACKRESULT_POSSIBLE )
	{

// srj sez: this bit is intended to fix the situation where you have a stinger site
// selected and get the "attack if I move" cursor against an enemy. since the stinger site can't
// move or fire, you shouldn't really EVER get this cursor for it. and since we just verified above
// that our slaves (the soldiers) can attack correctly, just nork it.
		if (source->isKindOf( KINDOF_IMMOBILE ) &&
				source->isKindOf( KINDOF_SPAWNS_ARE_THE_WEAPONS ) &&
				okResult == ATTACKRESULT_POSSIBLE_AFTER_MOVING)
			okResult = ATTACKRESULT_POSSIBLE;

		return okResult;
	}

	// Oh well... guess not!
	return ATTACKRESULT_INVALID_SHOT;
}

//-------------------------------------------------------------------------------------------------
Bool WeaponSet::chooseBestWeaponForTarget(const Object* obj, const Object* victim, WeaponChoiceCriteria criteria, CommandSourceType cmdSource)
{
	/*
		1) The first criteria is weapon fitness.  If the object has two weapons that can fire concurrently,
			find the set of weapons that can hit the given target.  If two weapons can hit the given target,
		2) Figure potential damage. If both weapons have the same potential damage (should never happen) then,
		3) Pick one.

		Another consideration is reloading.  A weapon that is not ready to fire right this moment should
		never be returned if there is another weapon that could be fired.  Readiness trumps Criteria.

		Note that this will never result in being locked into a wrong choice (ie "If only I had waited one
		more frame, I would have gotten the better weapon") because the AI is recomputing this choice
		every frame.  Picture a tank with a slow reloading missile and a fast low damage gun.  This function
		will choose the missile, then choose the gun, then want to choose the missile until the gun reloads,
		then choose the gun and go back to wanting to choose the missile, etc
	*/

	if( isCurWeaponLocked() )
		return TRUE; // I have been forced into choosing a specific weapon, so it is right until someone says otherwise

	if (victim == nullptr)
	{
		// Weapon lock is checked first for specific attack- ground powers.  Otherwise, we will reproduce the old behavior
		// and make only Primary attack the ground.
		m_curWeapon = PRIMARY_WEAPON;
		return TRUE;
	}

	Bool found = FALSE;				// A Ready weapon has been found
	Bool foundBackup = FALSE;	// An unready, but valid weapon has been found

	Real longestRange = 0.0f;
	Real bestDamage = 0.0f;
	Real longestRangeBackup = 0.0f;
	Real bestDamageBackup = 0.0f;

	WeaponSlotType currentDecision = PRIMARY_WEAPON;
	WeaponSlotType currentDecisionBackup = PRIMARY_WEAPON;

	// go backwards, so that in the event of ties, the primary weapon is preferred
	for (Int i = WEAPONSLOT_COUNT - 1; i >= PRIMARY_WEAPON ; --i)
	{
		/*
			First: eliminate the weapons that cannot be used.
		*/

		// no weapon in this slot.
		if (!m_weapons[i])
			continue;

		// weapon not allowed to be specified via this command source.
		CommandSourceMask okSrcs = m_curWeaponTemplateSet->getNthCommandSourceMask((WeaponSlotType)i);
		if( ( okSrcs & (1 << cmdSource) ) == 0 )
		{
			if( !( okSrcs & CMD_DEFAULT_SWITCH_WEAPON ) )
			{
				continue;
			}
		}

		Weapon* weapon = m_weapons[i];
		if (weapon == nullptr)
			continue;

		// No bad wrong!  Being out of range does not mean this weapon can not affect the target!
		// weapon out of range.
//		if (!weapon->isWithinAttackRange(obj, victim))
//			continue;

		// weapon out of ammo.
		if (weapon->getStatus() == OUT_OF_AMMO && !weapon->getAutoReloadsClip())
			continue;

		// weapon not allowed to target this kind of thing.
		if (!(weapon->getAntiMask() & getVictimAntiMask(victim)))
			continue;

		if (!weapon->isWithinTargetPitch(obj, victim))
			continue;

		Real damage = weapon->estimateWeaponDamage(obj, victim);
		Real attackRange = weapon->getAttackRange(obj);
		Bool weaponIsReady = (weapon->getStatus() == READY_TO_FIRE);

		const AIUpdateInterface* ai = obj->getAI();
		if (ai && ai->isWeaponSlotOnTurretAndAimingAtTarget((WeaponSlotType)i, victim))
			weaponIsReady = false;

		// weapon would do no damage against this target.
		// exception: 'unresistable' weapons are allowed to do zero damage.
		if (damage <= 0.0f && weapon->getDamageType() != DAMAGE_UNRESISTABLE)
			continue;

		/*
			now that we've eliminated the impossible ones, let's decide which
			one we will prefer.
		*/

		/*
			if a weapon matches the magic "preferred" bits, it's ALWAYS preferred over other weapons, regardless
			of actual damage/range. (eg, the Comanche cannon, which is always preferred against infantry, rather
			than its antitank missiles.) we accomplish this cheesily, by magnifying the damage/range, and by declaring
			the weapon "ready" in a more liberal manner.
		*/
		const KindOfMaskType& preferredAgainst = m_curWeaponTemplateSet->getNthPreferredAgainstMask((WeaponSlotType)i);
		if (KINDOFMASK_ANY_SET(preferredAgainst) && victim->isKindOfMulti(preferredAgainst, KINDOFMASK_NONE))
		{
			const Real HUGE_DAMAGE = 1e10;		// wow, that's a lot of damage.
			const Real HUGE_RANGE = 1e10;
			damage = HUGE_DAMAGE;
			attackRange = HUGE_RANGE;
			// preferred weapons are also kept if they are merely reloading. (if out of ammo, we can punt.)
			weaponIsReady = (weapon->getStatus() != OUT_OF_AMMO);
		}

		switch (criteria)
		{
			case PREFER_MOST_DAMAGE:
				if( !weaponIsReady )
				{
					// If this weapon is not ready, the best it can do is qualify as the Best Backup choice
					if (damage >= bestDamageBackup)
					{
						bestDamageBackup = damage;
						currentDecisionBackup = (WeaponSlotType)i;
						foundBackup = true;
					}
				}
				else
				{
					if (damage >= bestDamage)
					{
						bestDamage = damage;
						currentDecision = (WeaponSlotType)i;
						found = true;
					}
				}
				break;
			case PREFER_LONGEST_RANGE:
				{
					if( !weaponIsReady )
					{
						if (attackRange > longestRangeBackup)
						{
							longestRangeBackup = attackRange;
							currentDecisionBackup = (WeaponSlotType)i;
							foundBackup = true;
						}
					}
					else
					{
						if (attackRange > longestRange)
						{
							longestRange = attackRange;
							currentDecision = (WeaponSlotType)i;
							found = true;
						}
					}
				}
				break;
		}
	}

	if ( found )
	{
		// If we found a good best one, then just use it
		m_curWeapon = currentDecision;
	}
	else if ( foundBackup )
	{
		// No ready weapon was found, so return the most suitable unready one.
		m_curWeapon = currentDecisionBackup;
		found = TRUE;
	}
	else
	{
		// No weapon at all was found, so we go back to primary.
		m_curWeapon = PRIMARY_WEAPON;
	}

	//DEBUG_LOG(("WeaponSet::chooseBestWeaponForTarget -- changed curweapon to %s",getCurWeapon()->getName().str()));

	return found;
}


//-------------------------------------------------------------------------------------------------
void WeaponSet::reloadAllAmmo(const Object *obj, Bool now)
{
	for( Int i = 0; i < WEAPONSLOT_COUNT;	i++ )
	{
		Weapon* weapon = m_weapons[i];
		if (weapon != nullptr)
		{
			if (now)
				weapon->loadAmmoNow(obj);
			else
				weapon->reloadAmmo(obj);
		}
	}
}

//-------------------------------------------------------------------------------------------------
Bool WeaponSet::isOutOfAmmo() const
{
	for( Int i = 0; i < WEAPONSLOT_COUNT;	i++ )
	{
		const Weapon* weapon = m_weapons[i];
		if (weapon == nullptr)
			continue;
		if (weapon->getStatus() != OUT_OF_AMMO)
		{
			return false;
		}
	}
	return true;
}

//-------------------------------------------------------------------------------------------------
const Weapon* WeaponSet::findAmmoPipShowingWeapon() const
{
	for( Int i = 0; i < WEAPONSLOT_COUNT;	i++ )
	{
		const Weapon *weapon = m_weapons[ i ];
		if (weapon && weapon->isShowsAmmoPips())
		{
			return weapon;
		}
	}
	return nullptr;
}

//-------------------------------------------------------------------------------------------------
Weapon* WeaponSet::findWaypointFollowingCapableWeapon()
{
	for( Int i = WEAPONSLOT_COUNT - 1; i >= PRIMARY_WEAPON; i-- )
	{
		if( m_weapons[i] && m_weapons[i]->isCapableOfFollowingWaypoint() )
		{
			return m_weapons[i];
		}
	}
	return nullptr;
}

//-------------------------------------------------------------------------------------------------
UnsignedInt WeaponSet::getMostPercentReadyToFireAnyWeapon() const
{
	UnsignedInt mostReady = 0;
	for( Int i = 0; i < WEAPONSLOT_COUNT;	i++ )
	{
		if( m_weapons[ i ] )
		{
			UnsignedInt percentage = (UnsignedInt)(m_weapons[ i ]->getPercentReadyToFire() * 100.0f);
			if( percentage > mostReady )
			{
				mostReady = percentage;
			}
			if( mostReady >= 100 )
			{
				return mostReady;
			}
		}
	}
	return mostReady;
}

//-------------------------------------------------------------------------------------------------
// A special type of command demands that you use this (normally unchooseable) weapon
// until told otherwise.
Bool WeaponSet::setWeaponLock( WeaponSlotType weaponSlot, WeaponLockType lockType )
{
	if (lockType == NOT_LOCKED)
	{
		DEBUG_CRASH(("calling setWeaponLock with NOT_LOCKED, so I am doing nothing... did you mean to use releaseWeaponLock()?"));
		return false;
	}

	if (static_cast<UnsignedInt>(weaponSlot) >= ARRAY_SIZE(m_weapons))
	{
		return false;
	}

	// Verify the asked for weapon exists , choose it, and then lock it as chosen until unlocked
	// the old code was just plain wrong. (look at it in perforce and you'll see...)
	if (m_weapons[weaponSlot] != nullptr)
	{
		if( lockType == LOCKED_PERMANENTLY )
		{
			m_curWeapon = weaponSlot;
			m_curWeaponLockedStatus = lockType;
			//DEBUG_LOG(("WeaponSet::setWeaponLock permanently -- changed curweapon to %s",getCurWeapon()->getName().str()));
		}
		else if( lockType == LOCKED_TEMPORARILY && m_curWeaponLockedStatus != LOCKED_PERMANENTLY )
		{
			m_curWeapon = weaponSlot;
			m_curWeaponLockedStatus = lockType;
			//DEBUG_LOG(("WeaponSet::setWeaponLock temporarily -- changed curweapon to %s",getCurWeapon()->getName().str()));
		}

		return true;
	}

	DEBUG_CRASH(("setWeaponLock: weapon %d not found (missing an upgrade?)", (Int)weaponSlot));
	return false;
}

//-------------------------------------------------------------------------------------------------
// Either we have successfully fired a full clip of our special attack, or we have switched
// weaponsets entirely, or any Player issued command besides special attack has been given.
void WeaponSet::releaseWeaponLock(WeaponLockType lockType)
{
	if( m_curWeaponLockedStatus == NOT_LOCKED )
		return;// Nothing to do

	if (lockType == LOCKED_PERMANENTLY)
	{
		// all locks released.
		m_curWeaponLockedStatus = NOT_LOCKED;
	}
	else if (lockType == LOCKED_TEMPORARILY)
	{
		// only unlocked if the current lock is temporary.
		if (m_curWeaponLockedStatus == LOCKED_TEMPORARILY)
			m_curWeaponLockedStatus = NOT_LOCKED;
	}
	else
	{
		DEBUG_CRASH(("calling releaseWeaponLock with NOT_LOCKED makes no sense. why did you do this?"));
	}
}

//-------------------------------------------------------------------------------------------------
Weapon* WeaponSet::getWeaponInWeaponSlot(WeaponSlotType wslot) const
{
	return m_weapons[wslot];
}

//-------------------------------------------------------------------------------------------------
	//When an AIAttackState is over, it needs to clean up any weapons that might be in leech range mode
	//or else those weapons will have unlimited range!
void WeaponSet::clearLeechRangeModeForAllWeapons()
{
	for( Int i = 0; i < WEAPONSLOT_COUNT;	i++ )
	{
		Weapon* weapon = m_weapons[ i ];
		if( weapon )
		{
			//Just clear the leech range active flag.
			weapon->setLeechRangeActive( FALSE );
		}
	}
}

//-------------------------------------------------------------------------------------------------
Bool WeaponSet::isSharedReloadTime() const
{
	if (m_curWeaponTemplateSet)
		return m_curWeaponTemplateSet->isSharedReloadTime();
	return false;
}
```
