# Translation Packet `PKT-273`: `GrantStealthBehavior`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-273`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/behavior/grant_stealth_behavior.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (21):**
  - `Common/BitFlagsIO.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/BitFlagsIO.h`
  - `Common/INI.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/INI.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/Thing.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Thing.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Anim2D.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/Anim2D.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameClient/ParticleSys.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/ParticleSys.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/ContainModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ContainModule.h`
  - `GameLogic/Module/DamageModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DamageModule.h`
  - `GameLogic/Module/GrantStealthBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/GrantStealthBehavior.h`
  - `GameLogic/Module/StealthUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/StealthUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Module/UpgradeModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpgradeModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (5):**
  - `"StartRadius" => INI::parseReal`
  - `"FinalRadius" => INI::parseReal`
  - `"RadiusGrowRate" => INI::parseReal`
  - `"KindOf" => KindOfMaskType::parseFromINI`
  - `"RadiusParticleSystemName" => INI::parseParticleSystemTemplate`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (3 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUser( &m_radiusParticleSystemID, sizeof( ParticleSystemID ) )`
  3. `xferReal( &m_currentScanRadius )`

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
- `zh_logic::arena`: `ObjectSlot`, `INVALID`, `fn`, `ModuleSlot`, `ModuleDataId`, `ThingTemplateId`, `FacetMask`, `NONE`, `UPDATE`, `DIE`, `DAMAGE`, `CREATE`, `COLLIDE`, `BODY`, ... (+42)
- `zh_logic::modules::body::active_body`: `ActiveBodyModuleData`, `fn`, `ActiveBody`
- `zh_logic::modules::body::body_module`: `BodyModuleData`, `BodyModuleBase`, `fn`, `BodyModule`
- `zh_logic::modules::body::highlander_body`: `HighlanderBody`
- `zh_logic::modules::body::hive_structure_body`: `HiveStructureBodyModuleData`, `fn`, `HiveStructureBody`
- `zh_logic::modules::body::immortal_body`: `ImmortalBody`
- `zh_logic::modules::body::inactive_body`: `InactiveBody`, `fn`
- `zh_logic::modules::body::structure_body`: `StructureBody`
- `zh_logic::modules::body::undead_body`: `UndeadBodyModuleData`, `fn`, `UndeadBody`
- `zh_logic::modules::collide::collide_module`: `CollideModuleData`, `CollideModule`, `fn`, `CollideBehaviorImpl`
- `zh_logic::modules::collide::convert_to_car_bomb_crate_collide`: `ConvertToCarBombCrateCollideModuleData`, `ConvertToCarBombCrateCollide`
- `zh_logic::modules::collide::convert_to_hijacked_vehicle_crate_collide`: `ConvertToHijackedVehicleCrateCollideModuleData`, `ConvertToHijackedVehicleCrateCollide`
- `zh_logic::modules::collide::crate_collide`: `PRESERVE_MULTI_CRATE_PICKUP`, `SabotageVictimType`, `CrateCollideData`, `CrateCollide`, `CrateCollideImpl`
- `zh_logic::modules::collide::fire_weapon_collide`: `FireWeaponCollideModuleData`, `FireWeaponCollide`
- `zh_logic::modules::collide::heal_crate_collide`: `HealCrateCollideModuleData`, `HealCrateCollide`
- `zh_logic::modules::collide::money_crate_collide`: `UpgradePair`, `MoneyCrateCollideModuleData`, `MoneyCrateCollide`
- `zh_logic::modules::collide::sabotage_command_center_crate_collide`: `SabotageCommandCenterCrateCollideModuleData`, `SabotageCommandCenterCrateCollide`
- `zh_logic::modules::collide::sabotage_fake_building`: `SabotageFakeBuildingCrateCollideModuleData`, `SabotageFakeBuildingCrateCollide`
- `zh_logic::modules::collide::sabotage_internet_center_crate_collide`: `SabotageInternetCenterCrateCollideModuleData`, `SabotageInternetCenterCrateCollide`
- `zh_logic::modules::collide::sabotage_military_factory_crate_collide`: `SabotageMilitaryFactoryCrateCollideModuleData`, `SabotageMilitaryFactoryCrateCollide`
- `zh_logic::modules::collide::sabotage_power_plant_crate_collide`: `SabotagePowerPlantCrateCollideModuleData`, `SabotagePowerPlantCrateCollide`
- `zh_logic::modules::collide::sabotage_superweapon_crate_collide`: `SabotageSuperweaponCrateCollideModuleData`, `SabotageSuperweaponCrateCollide`
- `zh_logic::modules::collide::sabotage_supply_center_crate_collide`: `SabotageSupplyCenterCrateCollideModuleData`, `SabotageSupplyCenterCrateCollide`
- `zh_logic::modules::collide::sabotage_supply_dropzone_crate_collide`: `SabotageSupplyDropzoneCrateCollideModuleData`, `SabotageSupplyDropzoneCrateCollide`
- `zh_logic::modules::collide::salvage_crate_collide`: `SalvageCrateCollideModuleData`, `SalvageCrateCollide`
- `zh_logic::modules::collide::shroud_crate_collide`: `ShroudCrateCollideModuleData`, `ShroudCrateCollide`
- `zh_logic::modules::collide::squish_collide`: `SquishCollideModuleData`, `SquishCollide`
- `zh_logic::modules::collide::unit_crate_collide`: `UnitCrateCollideModuleData`, `UnitCrateCollide`
- `zh_logic::modules::collide::veterancy_crate_collide`: `VeterancyCrateCollideModuleData`, `VeterancyCrateCollide`
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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/GrantStealthBehavior.h`
```cpp
// FILE: GrantStealthBehavior.h /////////////////////////////////////////////////////////////////////////
// Author: Mark Lorenzen, June 2003
// Desc:   Update that grants permanent stealth to those whose stealth is permanently grantable
//------------------------------------------

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameClient/ParticleSys.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/UpgradeModule.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Module/DamageModule.h"
#include "Common/BitFlagsIO.h"

class ParticleSystem;
class ParticleSystemTemplate;

//-------------------------------------------------------------------------------------------------
class GrantStealthBehaviorModuleData : public UpdateModuleData
{
public:
	Bool									m_initiallyActive;
	Bool									m_singleBurst;
	Int										m_healingAmount;
	Real									m_startRadius;
	Real									m_finalRadius;
	Real									m_radiusGrowRate;
	KindOfMaskType				m_kindOf;	//Only these types can heal -- defaults to everything.
	const ParticleSystemTemplate*				m_radiusParticleSystemTmpl;					//Optional particle system meant to apply to entire effect for entire duration.

	GrantStealthBehaviorModuleData()
	{
		m_finalRadius = 200.0f;
		m_startRadius = 0.0f;
    m_radiusGrowRate = 10.0f;
		m_radiusParticleSystemTmpl = nullptr;
		SET_ALL_KINDOFMASK_BITS( m_kindOf );
	}

	static void buildFieldParse( MultiIniFieldParse& p )
	{
		UpdateModuleData::buildFieldParse( p );

		static const FieldParse dataFieldParse[] =
		{
			{ "StartRadius",						         INI::parseReal,									 nullptr, offsetof( GrantStealthBehaviorModuleData, m_startRadius ) },
			{ "FinalRadius",						         INI::parseReal,									 nullptr, offsetof( GrantStealthBehaviorModuleData, m_finalRadius ) },
			{ "RadiusGrowRate",						       INI::parseReal,									 nullptr, offsetof( GrantStealthBehaviorModuleData, m_radiusGrowRate ) },
			{ "KindOf",						    KindOfMaskType::parseFromINI,					       nullptr, offsetof( GrantStealthBehaviorModuleData, m_kindOf ) },
			{ "RadiusParticleSystemName",				 INI::parseParticleSystemTemplate, nullptr, offsetof( GrantStealthBehaviorModuleData, m_radiusParticleSystemTmpl ) },
			{ nullptr, nullptr, nullptr, 0 }
		};

		p.add(dataFieldParse);

  }

};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class GrantStealthBehavior : public UpdateModule
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( GrantStealthBehavior, "GrantStealthBehavior" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( GrantStealthBehavior, GrantStealthBehaviorModuleData )

public:

	GrantStealthBehavior( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	virtual UpdateSleepTime update() override;


private:

	void grantStealthToObject( Object *obj );
	void createEmitters();

	ParticleSystemID m_radiusParticleSystemID;
  Real m_currentScanRadius;
};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Behavior/GrantStealthBehavior.cpp`
```cpp
// FILE: GrantStealthBehavior.cpp ///////////////////////////////////////////////////////////////////////
// Author: Lorenzen
///////////////////////////////////////////////////////////////////////////////////////////////////


// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine
#include "Common/Thing.h"
#include "Common/ThingTemplate.h"
#include "Common/INI.h"
#include "Common/Player.h"
#include "Common/Xfer.h"
#include "GameClient/ParticleSys.h"
#include "GameClient/Anim2D.h"
#include "GameClient/InGameUI.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/GrantStealthBehavior.h"
#include "GameLogic/Module/StealthUpdate.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"



//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
struct GrantStealthPlayerScanHelper
{
	KindOfMaskType m_kindOfToTest;
	Object *m_theGrantor;
	ObjectPointerList *m_objectList;
};

static void checkForGrantStealth( Object *testObj, void *userData )
{
	GrantStealthPlayerScanHelper *helper = (GrantStealthPlayerScanHelper*)userData;
	ObjectPointerList *listToAddTo = helper->m_objectList;

	if( testObj->isEffectivelyDead() )
		return;

	if( testObj->getControllingPlayer() != helper->m_theGrantor->getControllingPlayer() )
		return;

	if( testObj->isOffMap() )
		return;

	if( !testObj->isAnyKindOf(helper->m_kindOfToTest) )
		return;

	listToAddTo->push_back(testObj);

	if( testObj->getContain() )
	{
		// have to tag visible riders too, or they will float around and look silly.
		Object *rider = (Object*)testObj->getContain()->friend_getRider();
		if( rider )
		{
			listToAddTo->push_back(rider);
		}
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
GrantStealthBehavior::GrantStealthBehavior( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{
	const GrantStealthBehaviorModuleData *d = getGrantStealthBehaviorModuleData();

	m_radiusParticleSystemID = INVALID_PARTICLE_SYSTEM_ID;

  m_currentScanRadius = d->m_startRadius;

	setWakeFrame( getObject(), UPDATE_SLEEP_NONE );
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
GrantStealthBehavior::~GrantStealthBehavior()
{

	if( m_radiusParticleSystemID != INVALID_PARTICLE_SYSTEM_ID )
		TheParticleSystemManager->destroyParticleSystemByID( m_radiusParticleSystemID );

}



//-------------------------------------------------------------------------------------------------
/** The update callback. */
//-------------------------------------------------------------------------------------------------
UpdateSleepTime GrantStealthBehavior::update()
{

	Object *self = getObject();

	if ( self->isEffectivelyDead())
		return UPDATE_SLEEP_FOREVER;

	// TheSuperHackers @bugfix stephanmeesters 18/04/2026 Delay emitter creation until update, to ensure that the particle
	// systems are not created before ParticleManager has xfer-loaded.
	createEmitters();

	const GrantStealthBehaviorModuleData *d = getGrantStealthBehaviorModuleData();
	// setup scan filters
	PartitionFilterRelationship relationship( self, PartitionFilterRelationship::ALLOW_ALLIES );
	PartitionFilterSameMapStatus filterMapStatus( self );
	PartitionFilterAlive filterAlive;
	PartitionFilter *filters[] = { &relationship, &filterAlive, &filterMapStatus, nullptr };


  m_currentScanRadius += d->m_radiusGrowRate;

  Bool thisIsFinalScan = FALSE;
  if ( m_currentScanRadius >=  d->m_finalRadius )
  {
    m_currentScanRadius = d->m_finalRadius;
    thisIsFinalScan = TRUE;
  }

	// scan objects in our region
	ObjectIterator *iter = ThePartitionManager->iterateObjectsInRange( self->getPosition(), m_currentScanRadius, FROM_CENTER_2D, filters );
	MemoryPoolObjectHolder hold( iter );
	// GRANT STEALTH TO FRIENDLIES IN RADIUS
	for( Object *obj = iter->first(); obj; obj = iter->next() )
    grantStealthToObject( obj );

  if ( thisIsFinalScan )
  {

    TheGameLogic->destroyObject( self );
    return UPDATE_SLEEP_FOREVER;
  }

  return UPDATE_SLEEP_NONE;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void GrantStealthBehavior::grantStealthToObject( Object *obj )
{

  if ( obj == getObject() )
    return;


	const GrantStealthBehaviorModuleData *d = getGrantStealthBehaviorModuleData();
  if ( ! obj->isAnyKindOf( d->m_kindOf ) )
    return;

  StealthUpdate* stealth = obj->getStealth();
	if( stealth )
	{
		stealth->receiveGrant();
    Drawable *draw = obj->getDrawable();
    if ( draw )
    {
      draw->flashAsSelected();
    }
	}


}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void GrantStealthBehavior::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );


}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void GrantStealthBehavior::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );


	// particle system id
	xfer->xferUser( &m_radiusParticleSystemID, sizeof( ParticleSystemID ) );

	// Timer safety
	xfer->xferReal( &m_currentScanRadius );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void GrantStealthBehavior::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void GrantStealthBehavior::createEmitters()
{
	if( m_radiusParticleSystemID == INVALID_PARTICLE_SYSTEM_ID )
	{
		const GrantStealthBehaviorModuleData *d = getGrantStealthBehaviorModuleData();
		ParticleSystem *particleSystem = TheParticleSystemManager->createParticleSystem(d->m_radiusParticleSystemTmpl);
		if( particleSystem )
		{
			particleSystem->setPosition( getObject()->getPosition() );
			m_radiusParticleSystemID = particleSystem->getSystemID();
		}
	}
}
```
