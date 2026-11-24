# Translation Packet `PKT-240`: `SalvageCrateCollide`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-240`
- **Work Package:** `WP-5.2` (Phase 5)
- **Complexity & Assigned Tier:** Class `C1` -> `tier:1a-lite`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/collide/salvage_crate_collide.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (16):**
  - `Common/AudioEventRTS.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/AudioEventRTS.h`
  - `Common/KindOf.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/KindOf.h`
  - `Common/MiscAudio.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/MiscAudio.h`
  - `Common/Module.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Module.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/RandomValue.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/RandomValue.h`
  - `Common/STLTypedefs.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/STLTypedefs.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/GameText.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/GameText.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameLogic/ExperienceTracker.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ExperienceTracker.h`
  - `GameLogic/Module/CrateCollide.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CrateCollide.h`
  - `GameLogic/Module/SalvageCrateCollide.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SalvageCrateCollide.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (5):**
  - `"WeaponChance" => INI::parsePercentToReal`
  - `"LevelChance" => INI::parsePercentToReal`
  - `"MoneyChance" => INI::parsePercentToReal`
  - `"MinMoney" => INI::parseInt`
  - `"MaxMoney" => INI::parseInt`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (1 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
- **PRNG Calls (3 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValueReal( 0, 1 )`
  - `GameLogicRandomValueReal( 0, 1 )`
  - `GameLogicRandomValue( md->m_minimumMoney, md->m_maximumMoney )`

## 2.5 Existing Rust Symbol Index (Use These Exact Paths)
- `zh_common::core::dict`: `DICT_MAX_LEN`, `DictDataType`, `fn`, `DictValue`, `Dict`
- `zh_common::core::name_key_generator`: `SOCKET_COUNT`, `NAMEKEY_MAX`, `NameKeyGenerator`
- `zh_common::core::state_machine`: `StateId`, `MACHINE_DONE_STATE_ID`, `INVALID_STATE_ID`, `EXIT_MACHINE_WITH_SUCCESS`, `EXIT_MACHINE_WITH_FAILURE`, `STATE_CONTINUE_CODE`, `STATE_SUCCESS_CODE`, `STATE_FAILURE_CODE`, `STATE_SLEEP_FOREVER_FRAMES`, `MAX_TRANSITION_DEPTH`, `StateReturnType`, `fn`, `StateExitType`, `StateWorldView`, ... (+8)
- `zh_common::crc`: `Crc`, `fn`
- `zh_common::ini`: `INI_MAX_CHARS_PER_LINE`, `LOGICFRAMES_PER_SECOND`, `MSEC_PER_SECOND`, `LOGICFRAMES_PER_MSEC_REAL`, `MSEC_PER_LOGICFRAME_REAL`, `LOGICFRAMES_PER_SECONDS_REAL`, `SECONDS_PER_LOGICFRAME_REAL`, `RADS_PER_DEGREE`, `SEPS_DEFAULT`, `SEPS_PERCENT`, `SEPS_COLON`, `SEPS_QUOTE`, `END_TOKEN`, `Color`, ... (+22)
- `zh_common::math`: `fn`, `Coord2D`, `Coord3D`, `ICoord2D`, `ICoord3D`, `Region2D`, `Region3D`, `IRegion2D`, `IRegion3D`, `RealRange`, `RGBColor`, `RGBAColorReal`, `RGBAColorInt`, `Matrix3D`
- `zh_common::math::bez_fwd_iterator`: `BezFwdIterator`
- `zh_common::math::bezier_segment`: `USUAL_TOLERANCE`, `BEZ_BASIS_MATRIX`, `BezierMath`, `BezierSegment`
- `zh_common::math::discrete_circle`: `HorzLine`, `DiscreteCircle`
- `zh_common::math::gamemath`: `WWMATH_PI`, `WWMATH_TWO_PI`, `PI`, `TWO_PI`, `WWMATH_EPSILON`
- `zh_common::math::geometry`: `GeometryType`, `fn`, `GEOMETRY_NAMES`, `GeometryInfo`
- `zh_common::math::partition_solver`: `PairObjectIdAndUInt`, `PairObjectId`, `EntriesVec`, `SpacesVec`, `SolutionVec`, `SolutionType`, `PartitionSolver`
- `zh_common::random_value`: `INITIAL_SEED_WORDS`, `THE_MULT_FACTOR`, `DistributionType`, `PrngState`, `fn`
- `zh_common::types`: `Int`, `UnsignedInt`, `Short`, `UnsignedShort`, `Byte`, `UnsignedByte`, `Bool`, `Real`, `XferVersion`, `FALSE`, `TRUE`, `ObjectId`, `INVALID`, `TeamId`, ... (+11)
- `zh_common::xfer`: `XferMode`, `XferError`, `Xfer`, `Snapshot`, `XferCrc`, `fn`, `XferDeepCrc`, `XferSave`, `XferLoad`
- `zh_common::xfer::bit_flags`: `ARMOR_SET_BIT_NAMES`, `MODEL_CONDITION_BIT_NAMES`, `ArmorSetFlags`, `ModelConditionFlags`, `BitFlags`, `NUM_BITS`, `fn`
- `zh_logic::arena`: `ObjectSlot`, `INVALID`, `fn`, `ModuleSlot`, `ModuleDataId`, `ThingTemplateId`, `FacetMask`, `NONE`, `UPDATE`, `DIE`, `DAMAGE`, `CREATE`, `COLLIDE`, `BODY`, ... (+34)
- `zh_logic::modules::body::active_body`: `ActiveBodyModuleData`, `fn`, `ActiveBody`
- `zh_logic::modules::body::body_module`: `BodyModuleData`, `BodyModuleBase`, `fn`, `BodyModule`
- `zh_logic::modules::body::highlander_body`: `HighlanderBody`
- `zh_logic::modules::body::hive_structure_body`: `HiveStructureBodyModuleData`, `fn`, `HiveStructureBody`
- `zh_logic::modules::body::immortal_body`: `ImmortalBody`
- `zh_logic::modules::body::inactive_body`: `InactiveBody`, `fn`
- `zh_logic::modules::body::structure_body`: `StructureBody`
- `zh_logic::modules::body::undead_body`: `UndeadBodyModuleData`, `fn`, `UndeadBody`
- `zh_logic::modules::create::create_module`: `CreateModuleData`, `CreateModuleBase`, `fn`, `CreateModule`
- `zh_logic::modules::create::grant_upgrade_create`: `GrantUpgradeCreateModuleData`, `GrantUpgradeCreate`
- `zh_logic::modules::create::lock_weapon_create`: `LockWeaponCreateModuleData`, `LockWeaponCreate`
- `zh_logic::modules::create::preorder_create`: `PreorderCreate`
- `zh_logic::modules::create::special_power_create`: `SpecialPowerCreate`
- `zh_logic::modules::create::supply_center_create`: `SupplyCenterCreate`
- `zh_logic::modules::create::supply_warehouse_create`: `SupplyWarehouseCreate`
- `zh_logic::modules::create::veterancy_gain_create`: `VeterancyGainCreateModuleData`, `VeterancyGainCreate`
- `zh_logic::modules::damage::bone_fx_damage`: `BoneFXDamage`, `fn`
- `zh_logic::modules::damage::damage_module`: `DamageModuleData`, `DamageModuleBase`, `fn`, `DamageModule`
- `zh_logic::modules::damage::transition_damage_fx`: `DAMAGE_MODULE_MAX_FX`, `FxDamageLocType`, `FxLocInfo`, `fn`, `FxDamageFxListInfo`, `FxDamageOclInfo`, `FxDamageParticleSystemInfo`, `TransitionDamageFxModuleData`, `TransitionDamageFX`
- `zh_logic::modules::destroy::destroy_module`: `DestroyModuleData`, `DestroyModuleBase`, `fn`, `DestroyModule`
- `zh_logic::modules::helper::object_defection_helper`: `DEFECTION_DETECTION_TIME_MAX`, `ObjectDefectionHelper`
- `zh_logic::modules::helper::object_helper`: `ObjectHelperBase`, `ObjectHelper`
- `zh_logic::modules::helper::object_repulsor_helper`: `ObjectRepulsorHelper`
- `zh_logic::modules::helper::object_smc_helper`: `ObjectSMCHelper`
- `zh_logic::modules::helper::object_weapon_status_helper`: `ObjectWeaponStatusHelper`
- `zh_logic::modules::helper::status_damage_helper`: `StatusDamageHelper`
- `zh_logic::modules::helper::subdual_damage_helper`: `SubdualDamageHelper`
- `zh_logic::modules::helper::temp_weapon_bonus_helper`: `WEAPON_BONUS_CONDITION_INVALID`, `TempWeaponBonusHelper`
- `zh_logic::modules::module`: `INVALID_BODY_TIMESTAMP`, `MaxHealthChangeType`, `fn`, `MAX_HEALTH_CHANGE_NAMES`, `ModuleData`, `StubModuleData`, `ModuleSnapshotHeader`, `BehaviorModule`, `UpdateFacet`, `BodyFacet`, `DieFacet`, `DamageFacet`, `CreateFacet`, `CollideFacet`, ... (+9)
- `zh_logic::modules::module_factory`: `ModuleParseContext`, `ModuleDataParseFn`, `ModuleCreateFn`, `BuiltinModuleEntry`, `BUILTIN_ZH_MODULES`, `W3D_DRAW_STUB_NAMES`, `ModuleDataStore`, `ModuleDescriptor`, `ModuleRegistry`
- `zh_logic::object::experience_tracker`: `ExperienceParentAccess`, `ExperienceTrackerState`, `fn`
- `zh_logic::object::object_script_status_bits`: `ObjectScriptStatusBit`, `fn`, `OBJECT_STATUS_SCRIPT_DISABLED`, `OBJECT_STATUS_SCRIPT_UNPOWERED`, `OBJECT_STATUS_SCRIPT_UNSELLABLE`, `OBJECT_STATUS_SCRIPT_UNSTEALTHED`, `OBJECT_STATUS_SCRIPT_TARGETABLE`
- `zh_logic::object::object_types`: `ObjectTypes`
- `zh_logic::object::overridable`: `OverridableEntry`, `fn`
- `zh_logic::object::override`: `OverrideStore`, `Override`, `fn`
- `zh_logic::object::thing`: `INITIAL_THING_POS_X`, `INITIAL_THING_POS_Y`, `ThingTerrainAccess`, `ThingTransformReactor`, `NoopTransformReactor`, `Thing`, `fn`
- `zh_logic::object::thing_factory`: `ThingFactory`, `fn`
- `zh_logic::object::thing_template`: `LEVEL_COUNT`, `MAX_UPGRADE_CAMEO_UPGRADES`, `USE_EXP_VALUE_FOR_SKILL_VALUE`, `RADAR_PRIORITY_NAMES`, `MODULEINTERFACE_UPDATE`, `MODULEINTERFACE_DIE`, `MODULEINTERFACE_DAMAGE`, `MODULEINTERFACE_CREATE`, `MODULEINTERFACE_COLLIDE`, `MODULEINTERFACE_BODY`, `MODULEINTERFACE_CONTAIN`, `MODULEINTERFACE_UPGRADE`, `MODULEINTERFACE_SPECIAL_POWER`, `MODULEINTERFACE_DESTROY`, ... (+27)

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SalvageCrateCollide.h`
```cpp
// FILE: SalvageCrateCollide.h /////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, March 2002
// Desc:   The Savlage system can give a Weaponset bonus, a level, or money.  Salvagers create them
//					by killing marked units, and only WeaponSalvagers can get the WeaponSet bonus
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/Module.h"
#include "Common/STLTypedefs.h"
#include "GameLogic/Module/CrateCollide.h"

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
class Thing;

//-------------------------------------------------------------------------------------------------
class SalvageCrateCollideModuleData : public CrateCollideModuleData
{
public:
	Real m_weaponChance;	///< Chance to get a weapon upgrade, if possible
	Real m_levelChance;		///< Chance to get a level, if weaponChance fails
	Real m_moneyChance;		///< Chance to get money, if weaponChance fails
	Int m_minimumMoney;		///< How much, if we get money
	Int m_maximumMoney;		///< How much, if we get money

	SalvageCrateCollideModuleData()
	{
		m_weaponChance = 1.0f;
		m_levelChance = .25f;
		m_moneyChance = .75f;
		m_minimumMoney = 25;
		m_maximumMoney = 75;
	}

	static void buildFieldParse(MultiIniFieldParse& p)
	{
    CrateCollideModuleData::buildFieldParse(p);

		static const FieldParse dataFieldParse[] =
		{
			{ "WeaponChance",	INI::parsePercentToReal,	nullptr, offsetof( SalvageCrateCollideModuleData, m_weaponChance ) },
			{ "LevelChance",	INI::parsePercentToReal,	nullptr, offsetof( SalvageCrateCollideModuleData, m_levelChance ) },
			{ "MoneyChance",	INI::parsePercentToReal,	nullptr, offsetof( SalvageCrateCollideModuleData, m_moneyChance ) },
			{ "MinMoney",			INI::parseInt,						nullptr, offsetof( SalvageCrateCollideModuleData, m_minimumMoney ) },
			{ "MaxMoney",			INI::parseInt,						nullptr, offsetof( SalvageCrateCollideModuleData, m_maximumMoney ) },
			{ nullptr, nullptr, nullptr, 0 }
		};
    p.add(dataFieldParse);

	}
};

//-------------------------------------------------------------------------------------------------
class SalvageCrateCollide : public CrateCollide
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( SalvageCrateCollide, "SalvageCrateCollide" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( SalvageCrateCollide, SalvageCrateCollideModuleData );

public:

	SalvageCrateCollide( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	virtual Bool isSalvageCrateCollide() const override { return true; }

protected:

	/// This allows specific vetoes to certain types of crates and their data
	virtual Bool isValidToExecute( const Object *other ) const override;

	/// This is the game logic execution function that all real CrateCollides will implement
	virtual Bool executeCrateBehavior( Object *other ) override;

private:
	Bool eligibleForWeaponSet( Object *other );
	Bool eligibleForArmorSet( Object *other );
	Bool eligibleForLevel( Object *other );
	Bool testWeaponChance();
	Bool testLevelChance();

	void doWeaponSet( Object *other );
	void doArmorSet( Object *other );
	void doLevelGain( Object *other );
	void doMoney( Object *other );

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Collide/CrateCollide/SalvageCrateCollide.cpp`
```cpp
// FILE: SalvageCrateCollide.cpp ///////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, March 2002
// Desc:   The Savlage system can give a Weaponset bonus, a level, or money.  Salvagers create them
//					by killing marked units, and only WeaponSalvagers can get the WeaponSet bonus
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine
#include "GameLogic/Module/SalvageCrateCollide.h"

#include "Common/AudioEventRTS.h"
#include "Common/MiscAudio.h"
#include "Common/KindOf.h"
#include "Common/RandomValue.h"
#include "Common/Player.h"
#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"
#include "GameClient/GameText.h"
#include "GameClient/InGameUI.h"
#include "GameLogic/ExperienceTracker.h"
#include "GameLogic/Object.h"

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
SalvageCrateCollide::SalvageCrateCollide( Thing *thing, const ModuleData* moduleData ) : CrateCollide( thing, moduleData )
{

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
SalvageCrateCollide::~SalvageCrateCollide()
{

}

//-------------------------------------------------------------------------------------------------
Bool SalvageCrateCollide::isValidToExecute( const Object *other ) const
{
	if( ! CrateCollide::isValidToExecute( other ) )
		return FALSE;

	// Only salvage units can pick up a Salvage crate
	if( ! other->getTemplate()->isKindOf( KINDOF_SALVAGER ) )
		return FALSE;

	return TRUE;
}

//-------------------------------------------------------------------------------------------------
Bool SalvageCrateCollide::executeCrateBehavior( Object *other )
{
	if( eligibleForArmorSet(other) )// No percent chance on this one, if you can get it, you get it.
	{
		doArmorSet(other);

		//Play the salvage installation crate pickup sound.
		AudioEventRTS soundToPlay = TheAudio->getMiscAudio()->m_crateSalvage;
		soundToPlay.setObjectID( other->getID() );
		TheAudio->addAudioEvent( &soundToPlay );
	}
	else if( eligibleForWeaponSet( other ) && testWeaponChance() )
	{
		doWeaponSet( other );

		//Play the salvage installation crate pickup sound.
		AudioEventRTS soundToPlay = TheAudio->getMiscAudio()->m_crateSalvage;
		soundToPlay.setObjectID( other->getID() );
		TheAudio->addAudioEvent( &soundToPlay );

		//Play the unit voice acknowledgement for upgrading weapons.
		//Already handled by the "move order"
		//const AudioEventRTS *soundToPlayPtr = other->getTemplate()->getPerUnitSound( "VoiceSalvage" );
		//soundToPlay = *soundToPlayPtr;
		//soundToPlay.setObjectID( other->getID() );
		//TheAudio->addAudioEvent( &soundToPlay );
	}
	else if( eligibleForLevel( other ) && testLevelChance() )
	{
		doLevelGain( other );

		//Sound will play in
		//soundToPlay = TheAudio->getMiscAudio()->m_unitPromoted;
	}
	else // just assume the testMoneyChance
	{
		doMoney( other );
		AudioEventRTS soundToPlay = TheAudio->getMiscAudio()->m_crateMoney;
		soundToPlay.setObjectID( other->getID() );
		TheAudio->addAudioEvent(&soundToPlay);
	}

	other->getControllingPlayer()->getAcademyStats()->recordSalvageCollected();

	return TRUE;
}

// ------------------------------------------------------------------------------------------------
Bool SalvageCrateCollide::eligibleForWeaponSet( Object *other )
{
	if( other == nullptr )
		return FALSE;

	// A kindof marks eligibility, and you must not be fully upgraded
	if( !other->isKindOf(KINDOF_WEAPON_SALVAGER) )
		return FALSE;
	if( other->testWeaponSetFlag(WEAPONSET_CRATEUPGRADE_TWO) )
		return FALSE;

	return TRUE;
}

// ------------------------------------------------------------------------------------------------
Bool SalvageCrateCollide::eligibleForArmorSet( Object *other )
{
	if( other == nullptr )
		return FALSE;

	// A kindof marks eligibility, and you must not be fully upgraded
	if( !other->isKindOf(KINDOF_ARMOR_SALVAGER) )
		return FALSE;
	if( other->testArmorSetFlag(ARMORSET_CRATE_UPGRADE_TWO) )
		return FALSE;

	return TRUE;
}

// ------------------------------------------------------------------------------------------------
Bool SalvageCrateCollide::eligibleForLevel( Object *other )
{
	if( other == nullptr )
		return FALSE;

	// Sorry, you are max level
	if( other->getExperienceTracker()->getVeterancyLevel() == LEVEL_HEROIC )
		return FALSE;

	// Sorry, you can't gain levels
	if( !other->getExperienceTracker()->isTrainable() )
		return FALSE;

	return TRUE;
}

// ------------------------------------------------------------------------------------------------
Bool SalvageCrateCollide::testWeaponChance()
{
	const SalvageCrateCollideModuleData *md = getSalvageCrateCollideModuleData();
	if( md->m_weaponChance == 1.0f )
		return TRUE; // don't waste a random number for a 100%

	Real randomNumber = GameLogicRandomValueReal( 0, 1 );
	if( randomNumber < md->m_weaponChance )
		return TRUE;

	return FALSE;
}

// ------------------------------------------------------------------------------------------------
Bool SalvageCrateCollide::testLevelChance()
{
	const SalvageCrateCollideModuleData *md = getSalvageCrateCollideModuleData();
	if( md->m_levelChance == 1.0f )
		return TRUE; // don't waste a random number for a 100%

	Real randomNumber = GameLogicRandomValueReal( 0, 1 );
	if( randomNumber < md->m_levelChance )
		return TRUE;

	return FALSE;
}

// ------------------------------------------------------------------------------------------------
void SalvageCrateCollide::doWeaponSet( Object *other )
{
	if( other->testWeaponSetFlag( WEAPONSET_CRATEUPGRADE_ONE ) )
	{
		other->clearWeaponSetFlag( WEAPONSET_CRATEUPGRADE_ONE );
		other->setWeaponSetFlag( WEAPONSET_CRATEUPGRADE_TWO );
	}
	else
	{
		other->setWeaponSetFlag( WEAPONSET_CRATEUPGRADE_ONE );
	}
}

// ------------------------------------------------------------------------------------------------
void SalvageCrateCollide::doArmorSet( Object *other )
{
	if( other->testArmorSetFlag( ARMORSET_CRATE_UPGRADE_ONE ) )
	{
		other->clearArmorSetFlag( ARMORSET_CRATE_UPGRADE_ONE );
		other->setArmorSetFlag( ARMORSET_CRATE_UPGRADE_TWO );

		other->clearAndSetModelConditionState(MODELCONDITION_ARMORSET_CRATEUPGRADE_ONE, MODELCONDITION_ARMORSET_CRATEUPGRADE_TWO);
	}
	else
	{
		other->setArmorSetFlag( ARMORSET_CRATE_UPGRADE_ONE );

		other->setModelConditionState(MODELCONDITION_ARMORSET_CRATEUPGRADE_ONE);
	}
}

// ------------------------------------------------------------------------------------------------
void SalvageCrateCollide::doLevelGain( Object *other )
{
	other->getExperienceTracker()->gainExpForLevel( 1 );
}

// ------------------------------------------------------------------------------------------------
void SalvageCrateCollide::doMoney( Object *other )
{
	const SalvageCrateCollideModuleData *md = getSalvageCrateCollideModuleData();

	Int money;
	if( md->m_minimumMoney != md->m_maximumMoney )// Random value doesn't like to get a constant range
		money = GameLogicRandomValue( md->m_minimumMoney, md->m_maximumMoney );
	else
		money = md->m_minimumMoney;

	if( money > 0 )
	{
		other->getControllingPlayer()->getMoney()->deposit( money );
		other->getControllingPlayer()->getScoreKeeper()->addMoneyEarned( money );

		//Display cash income floating over the crate.  Position is me, everything else is them.
		UnicodeString moneyString;
		moneyString.format( TheGameText->fetch( "GUI:AddCash" ), money );
		Coord3D pos;
		pos.set( *getObject()->getPosition() );
		pos.z += 10.0f; //add a little z to make it show up above the unit.
		Color color = other->getControllingPlayer()->getPlayerColor() | GameMakeColor( 0, 0, 0, 230 );
		TheInGameUI->addFloatingText( moneyString, &pos, color );
	}
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void SalvageCrateCollide::crc( Xfer *xfer )
{

	// extend base class
	CrateCollide::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void SalvageCrateCollide::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	CrateCollide::xfer( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void SalvageCrateCollide::loadPostProcess()
{

	// extend base class
	CrateCollide::loadPostProcess();

}
```
