# Translation Packet `PKT-204`: `CrateCollide`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-204`
- **Work Package:** `WP-5.2` (Phase 5)
- **Complexity & Assigned Tier:** Class `C1` -> `tier:1a-lite`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/collide/crate_collide.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (14):**
  - `Common/BitFlagsIO.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/BitFlagsIO.h`
  - `Common/GameAudio.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameAudio.h`
  - `Common/MiscAudio.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/MiscAudio.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Anim2D.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/Anim2D.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/FXList.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/FXList.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/CollideModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CollideModule.h`
  - `GameLogic/Module/CrateCollide.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CrateCollide.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (12):**
  - `"RequiredKindOf" => KindOfMaskType::parseFromINI`
  - `"ForbiddenKindOf" => KindOfMaskType::parseFromINI`
  - `"ForbidOwnerPlayer" => INI::parseBool`
  - `"BuildingPickup" => INI::parseBool`
  - `"HumanOnly" => INI::parseBool`
  - `"AllowMultiPickup" => INI::parseBool`
  - `"PickupScience" => INI::parseScience`
  - `"ExecuteFX" => INI::parseFXList`
  - `"ExecuteAnimation" => INI::parseAsciiString`
  - `"ExecuteAnimationTime" => INI::parseReal`
  - `"ExecuteAnimationZRise" => INI::parseReal`
  - `"ExecuteAnimationFades" => INI::parseBool`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (1 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if !RETAIL_COMPATIBLE_CRC`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CrateCollide.h`
```cpp
// FILE: CrateCollide.h /////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, March 2002
// Desc:   Abstract base Class Crate Collide
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/CollideModule.h"

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
class Thing;
class Anim2DTemplate;
class FXList;
enum ScienceType CPP_11(: Int);

//-------------------------------------------------------------------------------------------------
class CrateCollideModuleData : public CollideModuleData
{
public:
	KindOfMaskType	m_kindof;				///< the kind(s) of units that can be collided with
	KindOfMaskType	m_kindofnot;		///< the kind(s) of units that CANNOT be collided with
	Bool m_isForbidOwnerPlayer;			///< This crate cannot be picked up by the player of the dead thing that made it.
	Bool m_isBuildingPickup;			///< This crate can be picked up by a Building (bypassing AI requirement)
	Bool m_isHumanOnlyPickup;				///< Can this crate only be picked up by a human player?  (Mission thing)
	Bool m_allowMultiPickup;				///< Can this crate be picked up by multiple objects on the same frame?
	ScienceType m_pickupScience;		///< Can only be picked up by a unit whose player has this science
	FXList *m_executeFX;						///< FXList to play when activated

	AsciiString m_executionAnimationTemplate;				///< Anim2D to play at crate location
	Real m_executeAnimationDisplayTimeInSeconds;		///< time to play animation for
	Real m_executeAnimationZRisePerSecond;					///< rise animation up while playing
	Bool m_executeAnimationFades;										///< animation fades out

	CrateCollideModuleData();
	static void buildFieldParse(MultiIniFieldParse& p);
};

//-------------------------------------------------------------------------------------------------
class CrateCollide : public CollideModule
{

	MEMORY_POOL_GLUE_ABC( CrateCollide )
	MAKE_STANDARD_MODULE_MACRO_ABC( CrateCollide )
	MAKE_STANDARD_MODULE_DATA_MACRO_ABC( CrateCollide, CrateCollideModuleData )

public:

enum SabotageVictimType CPP_11(: Int)
{
	SAB_VICTIM_GENERIC = 0,
	SAB_VICTIM_COMMAND_CENTER,
	SAB_VICTIM_FAKE_BUILDING,
	SAB_VICTIM_INTERNET_CENTER,
	SAB_VICTIM_MILITARY_FACTORY,
	SAB_VICTIM_POWER_PLANT,
	SAB_VICTIM_SUPERWEAPON,
	SAB_VICTIM_SUPPLY_CENTER,
	SAB_VICTIM_DROP_ZONE,
};

	CrateCollide( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	/// This collide method gets called when collision occur
	virtual void onCollide( Object *other, const Coord3D *loc, const Coord3D *normal ) override;

	virtual Bool wouldLikeToCollideWith(const Object* other) const override { return isValidToExecute(other); }

	virtual Bool isRailroad() const override { return FALSE;};
 	virtual Bool isCarBombCrateCollide() const override { return FALSE; }
	virtual Bool isHijackedVehicleCrateCollide() const override { return FALSE; }
	virtual Bool isSabotageBuildingCrateCollide() const override { return FALSE; }

  void doSabotageFeedbackFX( const Object *other, SabotageVictimType type = SAB_VICTIM_GENERIC );

protected:

	/// This is the game logic execution function that all real CrateCollides will implement
	virtual Bool executeCrateBehavior( Object *other ) = 0;

	/// This allows specific vetoes to certain types of crates and their data
	virtual Bool isValidToExecute( const Object *other ) const;

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Collide/CrateCollide/CrateCollide.cpp`
```cpp
// FILE: CrateCollide.cpp ///////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, March 2002
// Desc:   Abstract base Class Crate Collide
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine
#include "Common/BitFlagsIO.h"
#include "Common/Player.h"
#include "Common/Xfer.h"
#include "Common/GameAudio.h"
#include "Common/MiscAudio.h"
#include "GameClient/Anim2D.h"
#include "GameClient/FXList.h"
#include "GameClient/InGameUI.h"
#include "GameClient/Drawable.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/Module/CrateCollide.h"


//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
CrateCollideModuleData::CrateCollideModuleData()
{
	m_isForbidOwnerPlayer = FALSE;
	m_executeAnimationDisplayTimeInSeconds = 0.0f;
	m_executeAnimationZRisePerSecond = 0.0f;
	m_executeAnimationFades = TRUE;
	m_isBuildingPickup = FALSE;
	m_isHumanOnlyPickup = FALSE;
	m_allowMultiPickup = (PRESERVE_MULTI_CRATE_PICKUP != 0);
	m_executeFX = nullptr;
	m_pickupScience = SCIENCE_INVALID;
	m_executionAnimationTemplate = AsciiString::TheEmptyString;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void CrateCollideModuleData::buildFieldParse(MultiIniFieldParse& p)
{
  ModuleData::buildFieldParse(p);

	static const FieldParse dataFieldParse[] =
	{
		{ "RequiredKindOf", KindOfMaskType::parseFromINI, nullptr, offsetof( CrateCollideModuleData, m_kindof ) },
		{ "ForbiddenKindOf", KindOfMaskType::parseFromINI, nullptr, offsetof( CrateCollideModuleData, m_kindofnot ) },
		{ "ForbidOwnerPlayer", INI::parseBool,	nullptr,	offsetof( CrateCollideModuleData, m_isForbidOwnerPlayer ) },
		{ "BuildingPickup", INI::parseBool,	nullptr,	offsetof( CrateCollideModuleData, m_isBuildingPickup ) },
		{ "HumanOnly", INI::parseBool,	nullptr,	offsetof( CrateCollideModuleData, m_isHumanOnlyPickup ) },
		{ "AllowMultiPickup", INI::parseBool,	nullptr,	offsetof( CrateCollideModuleData, m_allowMultiPickup ) },
		{ "PickupScience", INI::parseScience,	nullptr,	offsetof( CrateCollideModuleData, m_pickupScience ) },
		{ "ExecuteFX", INI::parseFXList, nullptr, offsetof( CrateCollideModuleData, m_executeFX ) },
		{ "ExecuteAnimation", INI::parseAsciiString, nullptr, offsetof( CrateCollideModuleData, m_executionAnimationTemplate ) },
		{ "ExecuteAnimationTime", INI::parseReal, nullptr, offsetof( CrateCollideModuleData, m_executeAnimationDisplayTimeInSeconds ) },
		{ "ExecuteAnimationZRise", INI::parseReal, nullptr, offsetof( CrateCollideModuleData, m_executeAnimationZRisePerSecond ) },
		{ "ExecuteAnimationFades", INI::parseBool, nullptr, offsetof( CrateCollideModuleData, m_executeAnimationFades ) },

		{ nullptr, nullptr, nullptr, 0 }
	};
  p.add(dataFieldParse);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
CrateCollide::CrateCollide( Thing *thing, const ModuleData* moduleData ) : CollideModule( thing, moduleData )
{

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
CrateCollide::~CrateCollide()
{

}

//-------------------------------------------------------------------------------------------------
/** The collide event.
	* Note that when other is nullptr it means "collide with ground" */
//-------------------------------------------------------------------------------------------------
void CrateCollide::onCollide( Object *other, const Coord3D *, const Coord3D * )
{
	const CrateCollideModuleData *modData = getCrateCollideModuleData();
	// If the crate can be picked up, perform the game logic and destroy the crate.
	if( isValidToExecute( other ) )
	{
		if( executeCrateBehavior( other ) )
		{
			if( modData->m_executeFX != nullptr )
			{
				// Note: We pass in other here, because the crate is owned by the neutral player, and
				// we want to do things that only the other person can see.
				FXList::doFXObj( modData->m_executeFX, other );
			}

			TheGameLogic->destroyObject( getObject() );
		}

		// play animation in the world at this spot if there is one
		if( TheAnim2DCollection && modData->m_executionAnimationTemplate.isEmpty() == FALSE && TheGameLogic->getDrawIconUI() )
		{
			Anim2DTemplate *animTemplate = TheAnim2DCollection->findTemplate( modData->m_executionAnimationTemplate );

			TheInGameUI->addWorldAnimation( animTemplate,
																			getObject()->getPosition(),
																			WORLD_ANIM_FADE_ON_EXPIRE,
																			modData->m_executeAnimationDisplayTimeInSeconds,
																			modData->m_executeAnimationZRisePerSecond );

		}

	}

}

//-------------------------------------------------------------------------------------------------
Bool CrateCollide::isValidToExecute( const Object *other ) const
{
	//The ground never picks up a crate
	if( other == nullptr )
		return FALSE;

	//Nothing Neutral can pick up any type of crate
	if( other->isNeutralControlled() )
		return FALSE;

	const CrateCollideModuleData* md = getCrateCollideModuleData();
	Bool validBuildingAttempt = md->m_isBuildingPickup && other->isKindOf( KINDOF_STRUCTURE );

	// Must be a "Unit" type thing.  Real Game Object, not just Object
	if( other->getAIUpdateInterface() == nullptr  &&  !validBuildingAttempt )// Building exception flag for Drop Zone
		return FALSE;

	// must match our kindof flags (if any)
	if ( !other->isKindOfMulti(md->m_kindof, md->m_kindofnot) )
		return FALSE;

	if( other->isEffectivelyDead() )
		return FALSE;

	// crates cannot be claimed while in the air, except by buildings
	if( getObject()->isAboveTerrain() && !validBuildingAttempt )
		return FALSE;

	// TheSuperHackers @bugfix Stubbjax 09/02/2026 Prevent the crate from being collected multiple times in a single frame.
#if !RETAIL_COMPATIBLE_CRC
	if (getObject()->isDestroyed() && !md->m_allowMultiPickup)
		return FALSE;
#endif

	if( md->m_isForbidOwnerPlayer  &&  (getObject()->getControllingPlayer() == other->getControllingPlayer()) )
		return FALSE; // Design has decreed this to not be picked up by the dead guy's team.

	if( md->m_isHumanOnlyPickup  &&  other->getControllingPlayer() && (other->getControllingPlayer()->getPlayerType() != PLAYER_HUMAN) )
		return FALSE; // Human only mission crate

	if( (md->m_pickupScience != SCIENCE_INVALID)  &&  other->getControllingPlayer()  &&  !other->getControllingPlayer()->hasScience(md->m_pickupScience) )
		return FALSE; // Science required to pick this up

	if( other->isKindOf( KINDOF_PARACHUTE ) )
		return FALSE;

	return TRUE;
}





void CrateCollide::doSabotageFeedbackFX( const Object *other, SabotageVictimType type )
{

  if ( ! getObject() )
    return;
  if ( ! other )
    return;

	AudioEventRTS soundToPlay;
  switch ( type )
  {
    case  CrateCollide::SAB_VICTIM_FAKE_BUILDING:
    {
      return; // THIS NEEDS NO ADD'L FEEDBACK
    }
    case 	CrateCollide::SAB_VICTIM_COMMAND_CENTER:
    case 	CrateCollide::SAB_VICTIM_SUPERWEAPON:
    {
      soundToPlay = TheAudio->getMiscAudio()->m_sabotageResetTimerBuilding;
      break;
    }
    case 	CrateCollide::SAB_VICTIM_DROP_ZONE:
    case 	CrateCollide::SAB_VICTIM_SUPPLY_CENTER:
    {
      soundToPlay = TheAudio->getMiscAudio()->m_moneyWithdrawSound;
      break;
    }
    case 	CrateCollide::SAB_VICTIM_INTERNET_CENTER:
    case 	CrateCollide::SAB_VICTIM_MILITARY_FACTORY:
    case 	CrateCollide::SAB_VICTIM_POWER_PLANT:
    default:
    {
      soundToPlay = TheAudio->getMiscAudio()->m_sabotageShutDownBuilding;
      break;
    }
  }

	soundToPlay.setPosition( other->getPosition() );
	TheAudio->addAudioEvent( &soundToPlay );

  Drawable *draw = other->getDrawable();
  if ( draw )
    draw->flashAsSelected();

}



// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void CrateCollide::crc( Xfer *xfer )
{

	// extend base class
	CollideModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer Method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void CrateCollide::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	CollideModule::xfer( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void CrateCollide::loadPostProcess()
{

	// extend base class
	CollideModule::loadPostProcess();

}
```
