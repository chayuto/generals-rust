# Translation Packet `PKT-294`: `RiderChangeContain`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-294`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/contain/rider_change_contain.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (20):**
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/PlayerList.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/PlayerList.h`
  - `Common/ThingFactory.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ThingFactory.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/ControlBar.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/ControlBar.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameLogic/AI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/AI.h`
  - `GameLogic/AIPathfind.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/AIPathfind.h`
  - `GameLogic/ExperienceTracker.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ExperienceTracker.h`
  - `GameLogic/Locomotor.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Locomotor.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/PhysicsUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PhysicsUpdate.h`
  - `GameLogic/Module/RiderChangeContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/RiderChangeContain.h`
  - `GameLogic/Module/StealthUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/StealthUpdate.h`
  - `GameLogic/Module/TransportContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/TransportContain.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (10):**
  - `"Rider1" => parseRiderInfo`
  - `"Rider2" => parseRiderInfo`
  - `"Rider3" => parseRiderInfo`
  - `"Rider4" => parseRiderInfo`
  - `"Rider5" => parseRiderInfo`
  - `"Rider6" => parseRiderInfo`
  - `"Rider7" => parseRiderInfo`
  - `"Rider8" => parseRiderInfo`
  - `"ScuttleDelay" => INI::parseDurationUnsignedInt`
  - `"ScuttleStatus" => INI::parseIndexList`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (4 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferBool( &m_payloadCreated )`
  3. `xferInt( &m_extraSlotsInUse )`
  4. `xferUnsignedInt( &m_frameExitNotBusy )`
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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/RiderChangeContain.h`
```cpp
// FILE: RiderChangeContain.cpp //////////////////////////////////////////////////////////////////////
// Author: Kris Morness, May 2003
// Desc:   Contain module for the combat bike (transport that switches units).
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/TransportContain.h"

#define MAX_RIDERS 8 //***NOTE: If you change this, make sure you update the parsing section!

enum WeaponSetType CPP_11(: Int);
enum ObjectStatusType CPP_11(: Int);
enum LocomotorSetType CPP_11(: Int);

struct RiderInfo
{
	AsciiString m_templateName;
	WeaponSetType m_weaponSetFlag;
	ModelConditionFlagType m_modelConditionFlagType;
	ObjectStatusType m_objectStatusType;
	AsciiString m_commandSet;
	LocomotorSetType m_locomotorSetType;
};

//-------------------------------------------------------------------------------------------------
class RiderChangeContainModuleData : public TransportContainModuleData
{
public:

	RiderInfo m_riders[ MAX_RIDERS ];
	UnsignedInt m_scuttleFrames;
	ModelConditionFlagType m_scuttleState;

	RiderChangeContainModuleData();

	static void buildFieldParse(MultiIniFieldParse& p);
	static void parseRiderInfo( INI* ini, void *instance, void *store, const void* /*userData*/ );

};

//-------------------------------------------------------------------------------------------------
class RiderChangeContain : public TransportContain
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( RiderChangeContain, "RiderChangeContain" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( RiderChangeContain, RiderChangeContainModuleData )

public:

	RiderChangeContain( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	virtual Bool isValidContainerFor( const Object* obj, Bool checkCapacity) const override;

	virtual void onCapture( Player *oldOwner, Player *newOwner ) override; // have to kick everyone out on capture.
	virtual void onContaining( Object *obj, Bool wasSelected ) override;		///< object now contains 'obj'
	virtual void onRemoving( Object *obj ) override;			///< object no longer contains 'obj'
	virtual UpdateSleepTime update() override;							///< called once per frame

	virtual Bool isRiderChangeContain() const override { return TRUE; }
	virtual const Object *friend_getRider() const override;

	virtual Int getContainMax() const override;

	virtual Int getExtraSlotsInUse() override { return m_extraSlotsInUse; }///< Transports have the ability to carry guys how take up more than spot.

	virtual Bool isExitBusy() const override;	///< Contain style exiters are getting the ability to space out exits, so ask this before reserveDoor as a kind of no-commitment check.
	virtual ExitDoorType reserveDoorForExit( const ThingTemplate* objType, Object *specificObject ) override;
	virtual void unreserveDoorForExit( ExitDoorType exitDoor ) override;
	virtual Bool isDisplayedOnControlBar() const override {return TRUE;}///< Does this container display its contents on the ControlBar?

	virtual Bool getContainerPipsToShow( Int& numTotal, Int& numFull ) override;

protected:

	// exists primarily for RiderChangeContain to override
	virtual void killRidersWhoAreNotFreeToExit() override;
	virtual Bool isSpecificRiderFreeToExit(Object* obj) override;
	virtual void createPayload() override;

private:

	Int m_extraSlotsInUse;
	UnsignedInt m_frameExitNotBusy;
	UnsignedInt m_scuttledOnFrame;

	Bool m_containing; //doesn't require xfer.

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Contain/RiderChangeContain.cpp`
```cpp
// FILE: RiderChangeContain.cpp //////////////////////////////////////////////////////////////////////
// Author: Kris Morness, May 2003
// Desc:   Contain module for the combat bike (transport that switches units).
///////////////////////////////////////////////////////////////////////////////////////////////////

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#define DEFINE_LOCOMOTORSET_NAMES //Gain access to TheLocomotorSetNames[]

#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/ThingTemplate.h"
#include "Common/ThingFactory.h"
#include "Common/Xfer.h"

#include "GameClient/ControlBar.h"
#include "GameClient/Drawable.h"
#include "GameClient/InGameUI.h"

#include "GameLogic/AI.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/ExperienceTracker.h"
#include "GameLogic/Object.h"
#include "GameLogic/Locomotor.h"

#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Module/StealthUpdate.h"
#include "GameLogic/Module/RiderChangeContain.h"



// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
RiderChangeContainModuleData::RiderChangeContainModuleData()
{
	m_scuttleFrames = 0;
	m_scuttleState = MODELCONDITION_TOPPLED;
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void RiderChangeContainModuleData::parseRiderInfo( INI* ini, void *instance, void *store, const void* /*userData*/ )
{
	RiderInfo* rider = (RiderInfo*)store;
	const char* name = ini->getNextToken();

	//Template name
	rider->m_templateName.format( name );

	//Model condition state
	INI::parseIndexList( ini, instance, &(rider->m_modelConditionFlagType), ModelConditionFlags::getBitNames() );

	//Weaponset
	INI::parseIndexList( ini, instance, &(rider->m_weaponSetFlag), WeaponSetFlags::getBitNames() );

	//Object status
	INI::parseIndexList( ini, instance, &(rider->m_objectStatusType), ObjectStatusMaskType::getBitNames() );

	//Command set override
	name = ini->getNextToken();
	rider->m_commandSet.format( name );

	//Locomotor set type
	rider->m_locomotorSetType = (LocomotorSetType)INI::scanIndexList( ini->getNextToken(), TheLocomotorSetNames );
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void RiderChangeContainModuleData::buildFieldParse(MultiIniFieldParse& p)
{
  TransportContainModuleData::buildFieldParse(p);

	static const FieldParse dataFieldParse[] =
	{
		{ "Rider1",					parseRiderInfo,					nullptr, offsetof( RiderChangeContainModuleData, m_riders[0] ) },
		{ "Rider2",					parseRiderInfo,					nullptr, offsetof( RiderChangeContainModuleData, m_riders[1] ) },
		{ "Rider3",					parseRiderInfo,					nullptr, offsetof( RiderChangeContainModuleData, m_riders[2] ) },
		{ "Rider4",					parseRiderInfo,					nullptr, offsetof( RiderChangeContainModuleData, m_riders[3] ) },
		{ "Rider5",					parseRiderInfo,					nullptr, offsetof( RiderChangeContainModuleData, m_riders[4] ) },
		{ "Rider6",					parseRiderInfo,					nullptr, offsetof( RiderChangeContainModuleData, m_riders[5] ) },
		{ "Rider7",					parseRiderInfo,					nullptr, offsetof( RiderChangeContainModuleData, m_riders[6] ) },
		{ "Rider8",					parseRiderInfo,					nullptr, offsetof( RiderChangeContainModuleData, m_riders[7] ) },
    { "ScuttleDelay",   INI::parseDurationUnsignedInt,	nullptr, offsetof( RiderChangeContainModuleData, m_scuttleFrames ) },
    { "ScuttleStatus",  INI::parseIndexList,		ModelConditionFlags::getBitNames(), offsetof( RiderChangeContainModuleData, m_scuttleState ) },
		{ nullptr, nullptr, nullptr, 0 }
	};
  p.add(dataFieldParse);
}



// PRIVATE ////////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Int RiderChangeContain::getContainMax() const
{
	if (getRiderChangeContainModuleData())
		return getRiderChangeContainModuleData()->m_slotCapacity;

	return 0;
}

// PUBLIC /////////////////////////////////////////////////////////////////////////////////////////
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
RiderChangeContain::RiderChangeContain( Thing *thing, const ModuleData *moduleData ) :
								 TransportContain( thing, moduleData )
{
	m_extraSlotsInUse = 0;
	m_frameExitNotBusy = 0;
	m_containing = FALSE;
	m_scuttledOnFrame = 0;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
RiderChangeContain::~RiderChangeContain()
{

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
/**
	can this container contain this kind of object?
	and, if checkCapacity is TRUE, does this container have enough space left to hold the given unit?
*/
Bool RiderChangeContain::isValidContainerFor(const Object* rider, Bool checkCapacity) const
{
	//Don't check capacity because our rider will kick the other rider out!
	if( TransportContain::isValidContainerFor( rider, FALSE ) )
	{
		if( m_scuttledOnFrame != 0 )
		{
			//Scuttled... too late!
			return FALSE;
		}

		//We can enter this bike... but now we need to extend the base functionality by limiting
		//which infantry can enter.
		const RiderChangeContainModuleData *data = getRiderChangeContainModuleData();
		for( int i = 0; i < MAX_RIDERS; i++ )
		{
			const ThingTemplate *thing = TheThingFactory->findTemplate( data->m_riders[ i ].m_templateName );
			if( thing && thing->isEquivalentTo( rider->getTemplate() ) )
			{
				//We found a valid rider, so return success.
				return TRUE;
			}
		}
	}
	return FALSE;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void RiderChangeContain::onContaining( Object *rider, Bool wasSelected )
{
	Object *obj = getObject();
	m_containing = TRUE;
	//Remove our existing rider
	if( m_payloadCreated )
	{
		obj->getAI()->aiEvacuateInstantly( TRUE, CMD_FROM_AI );
	}

	//If the rider is currently selected, transfer selection to the container and preserve other units
	//that may be already selected. Note that containing the rider will automatically cause it to be
	//deselected, so all we have to do is select the container (if not already selected)!
	Drawable *containDraw = getObject()->getDrawable();
	if( containDraw && wasSelected && !containDraw->isSelected() )
	{
		//Create the selection message
		GameMessage *teamMsg = TheMessageStream->appendMessage( GameMessage::MSG_CREATE_SELECTED_GROUP );
		teamMsg->appendBooleanArgument( FALSE );// not creating new team so pass false
		teamMsg->appendObjectIDArgument( getObject()->getID() );
		TheInGameUI->selectDrawable( containDraw );
		TheInGameUI->setDisplayedMaxWarning( FALSE );
	}

	//Find the rider in the list and set the appropriate model condition
	const RiderChangeContainModuleData *data = getRiderChangeContainModuleData();
	for( int i = 0; i < MAX_RIDERS; i++ )
	{
		const ThingTemplate *thing = TheThingFactory->findTemplate( data->m_riders[ i ].m_templateName );
		if( thing && thing->isEquivalentTo( rider->getTemplate() ) )
		{

			//This is our rider, so set the correct model condition.
			obj->setModelConditionState( data->m_riders[ i ].m_modelConditionFlagType );

			//Also set the correct weaponset flag
			obj->setWeaponSetFlag( data->m_riders[ i ].m_weaponSetFlag );

			//Also set the object status
			obj->setStatus( MAKE_OBJECT_STATUS_MASK( data->m_riders[ i ].m_objectStatusType ) );

			//Set the new commandset override
			obj->setCommandSetStringOverride( data->m_riders[ i ].m_commandSet );
			TheControlBar->markUIDirty();	// Refresh the UI in case we are selected

			//Change the locomotor.
			AIUpdateInterface* ai = obj->getAI();
			if( ai )
			{
				ai->chooseLocomotorSet( data->m_riders[ i ].m_locomotorSetType );
			}

			if( obj->getStatusBits().test( OBJECT_STATUS_STEALTHED ) )
			{
				StealthUpdate* stealth = obj->getStealth();
				if( stealth )
				{
					stealth->markAsDetected();
				}
			}

			//Transfer experience from the rider to the bike.
			ExperienceTracker *riderTracker = rider->getExperienceTracker();
			ExperienceTracker *bikeTracker = obj->getExperienceTracker();
#if !RETAIL_COMPATIBLE_CRC
			// TheSuperHackers @bugfix Stubbjax 15/12/2025 Copy trainable flag from the rider to prevent
			// Workers and other untrainable riders from ranking up via the bike's experience tracker.
			bikeTracker->setTrainable(riderTracker->isTrainable());
#endif
			bikeTracker->setVeterancyLevel( riderTracker->getVeterancyLevel(), FALSE );
			riderTracker->setExperienceAndLevel( 0, FALSE );

			break;
		}
	}

	//Extend base class
	TransportContain::onContaining( rider, wasSelected );

	m_containing = FALSE;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void RiderChangeContain::onRemoving( Object *rider )
{
	Object *bike = getObject();
	//Note if the bike dies, the rider dies too.
	if( bike->isEffectivelyDead() )
	{
		TheGameLogic->destroyObject( rider );
		return;
	}

	if( m_payloadCreated )
	{
		//Extend base class
		TransportContain::onRemoving( rider );
	}

	//Find the rider in the list and clear various data.
	const RiderChangeContainModuleData *data = getRiderChangeContainModuleData();
	for( int i = 0; i < MAX_RIDERS; i++ )
	{
		const ThingTemplate *thing = TheThingFactory->findTemplate( data->m_riders[ i ].m_templateName );
		if( thing && thing->isEquivalentTo( rider->getTemplate() ) )
		{
			//This is our rider, so clear the current model condition.
			bike->clearModelConditionFlags( MAKE_MODELCONDITION_MASK2( data->m_riders[ i ].m_modelConditionFlagType, MODELCONDITION_DOOR_1_CLOSING ) );

			//Also clear the current weaponset flag
			bike->clearWeaponSetFlag( data->m_riders[ i ].m_weaponSetFlag );

			//Also clear the object status
			bike->clearStatus( MAKE_OBJECT_STATUS_MASK( data->m_riders[ i ].m_objectStatusType ) );

			if( rider->getControllingPlayer() != nullptr )
			{
				//Wow, completely unforseeable game teardown order crash.  SetVeterancyLevel results in a call to player
				//about upgrade masks.  So if we have a null player, it is game teardown, so don't worry about transferring exp.

				//Transfer experience from the bike to the rider.
				ExperienceTracker *riderTracker = rider->getExperienceTracker();
				ExperienceTracker *bikeTracker = bike->getExperienceTracker();
				bikeTracker->resetTrainable();
				riderTracker->setVeterancyLevel( bikeTracker->getVeterancyLevel(), FALSE );
				bikeTracker->setExperienceAndLevel( 0, FALSE );
			}

			break;
		}
	}

	//If we're not replacing the rider, then if the cycle is selected, transfer selection
	//to the rider getting off (because the bike is gonna blow).
	if( !m_containing )
	{
		Drawable *containDraw = bike->getDrawable();
		Drawable *riderDraw = rider->getDrawable();
		if( containDraw && riderDraw )
		{
			//Create the selection message for the rider if it's ours and SELECTED!
			if( bike->isLocallyControlled() && containDraw->isSelected() )
			{
				GameMessage *teamMsg = TheMessageStream->appendMessage( GameMessage::MSG_CREATE_SELECTED_GROUP );
				teamMsg->appendBooleanArgument( FALSE );// not creating new team so pass false
				teamMsg->appendObjectIDArgument( rider->getID() );
				TheInGameUI->selectDrawable( riderDraw );
				TheInGameUI->setDisplayedMaxWarning( FALSE );

				//Create the de-selection message for the container
				teamMsg = TheMessageStream->appendMessage( GameMessage::MSG_REMOVE_FROM_SELECTED_GROUP );
				teamMsg->appendObjectIDArgument( bike->getID() );
				TheInGameUI->deselectDrawable( containDraw );
			}

			//Finally, scuttle the bike so nobody else can use it! <Design Spec>
			m_scuttledOnFrame = TheGameLogic->getFrame();
			bike->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_UNSELECTABLE ) );
			bike->setModelConditionState( data->m_scuttleState );
			if( !bike->getAI()->isMoving() )
			{
				bike->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_IMMOBILE ) );
			}
		}
	}
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void RiderChangeContain::createPayload()
{
	// extend base class
	TransportContain::createPayload();
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
UpdateSleepTime RiderChangeContain::update()
{
	if( m_scuttledOnFrame != 0 )
	{
		//Bike in the process of getting scuttled.
		const RiderChangeContainModuleData *data = getRiderChangeContainModuleData();
		UnsignedInt now = TheGameLogic->getFrame();
		if( m_scuttledOnFrame + data->m_scuttleFrames <= now )
		{
			//We have scuttled the bike (at least as far as tipping it over via scuttle animation. Now
			//kill the bike in a way that will cause it to sink into the ground without any real destruction.
			getObject()->kill( DAMAGE_UNRESISTABLE, DEATH_TOPPLED ); //Sneaky, eh? Toppled heheh.
		}
	}
	// extend base class
	return TransportContain::update(); //extend
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void RiderChangeContain::unreserveDoorForExit( ExitDoorType exitDoor )
{
	/* nothing */
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void RiderChangeContain::killRidersWhoAreNotFreeToExit()
{
	// extend base class
	TransportContain::killRidersWhoAreNotFreeToExit();
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
Bool RiderChangeContain::isSpecificRiderFreeToExit(Object* specificObject)
{
	// extend base class
	return TransportContain::isSpecificRiderFreeToExit( specificObject );
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
ExitDoorType RiderChangeContain::reserveDoorForExit( const ThingTemplate* objType, Object *specificObject )
{
	// extend base class
	return TransportContain::reserveDoorForExit( objType, specificObject );
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
Bool RiderChangeContain::isExitBusy() const	///< Contain style exiters are getting the ability to space out exits, so ask this before reserveDoor as a kind of no-commitment check.
{
	// extend base class
	return FALSE; //return TransportContain::isExitBusy();
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void RiderChangeContain::onCapture( Player *oldOwner, Player *newOwner )
{
	// extend base class
	TransportContain::onCapture( oldOwner, newOwner );
}

//-------------------------------------------------------------------------------------------------
Bool RiderChangeContain::getContainerPipsToShow(Int& numTotal, Int& numFull)
{
	//Don't show any pips for motorcycles as they always have one rider unless dead!
	numTotal = 0;
	numFull = 0;
	return false;
}

//-------------------------------------------------------------------------------------------------
const Object *RiderChangeContain::friend_getRider() const
{
 	if( m_containListSize > 0 ) // Yes, this does assume that infantry never ride double on the bike
 		return m_containList.front();

	return nullptr;
}




// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void RiderChangeContain::crc( Xfer *xfer )
{

	// extend base class
	TransportContain::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void RiderChangeContain::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	TransportContain::xfer( xfer );

	// payload created
	xfer->xferBool( &m_payloadCreated );

	// extra slots in use
	xfer->xferInt( &m_extraSlotsInUse );

	// frame exit not busy
	xfer->xferUnsignedInt( &m_frameExitNotBusy );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void RiderChangeContain::loadPostProcess()
{

	// extend base class
	TransportContain::loadPostProcess();

}
```
