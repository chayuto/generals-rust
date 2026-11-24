# Translation Packet `PKT-280`: `MobNexusContain`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-280`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/contain/mob_nexus_contain.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (16):**
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/ThingFactory.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ThingFactory.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameLogic/AI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/AI.h`
  - `GameLogic/AIPathfind.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/AIPathfind.h`
  - `GameLogic/Locomotor.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Locomotor.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/MobNexusContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/MobNexusContain.h`
  - `GameLogic/Module/OpenContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/OpenContain.h`
  - `GameLogic/Module/PhysicsUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PhysicsUpdate.h`
  - `GameLogic/Module/StealthUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/StealthUpdate.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `PreRTS.h`
- **Implemented Module Facets (1):**
  - `TransportPassengerInterface* getTransportPassengerInterface()`
- **INI `FieldParse` Table Entries (8):**
  - `"Slots" => INI::parseInt`
  - `"ScatterNearbyOnExit" => INI::parseBool`
  - `"OrientLikeContainerOnExit" => INI::parseBool`
  - `"KeepContainerVelocityOnExit" => INI::parseBool`
  - `"ExitBone" => INI::parseAsciiString`
  - `"ExitPitchRate" => INI::parseAngularVelocityReal`
  - `"InitialPayload" => parseInitialPayload`
  - `"HealthRegen%PerSec" => INI::parseReal`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (2 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferInt( &m_extraSlotsInUse )`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/MobNexusContain.h`
```cpp
// FILE: MobNexusContain.h ////////////////////////////////////////////////////////////////////////
// Author: Mark Lorenzen, August 2002
// Desc:   Contain module for mob units. Acts as proxy for UI and AI
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/OpenContain.h"

//-------------------------------------------------------------------------------------------------
class MobNexusContainModuleData : public OpenContainModuleData
{
public:
	struct InitialPayload
	{
		AsciiString name;
		Int count;
	};

	Int								m_slotCapacity;								///< max units that can be inside us
	Real							m_exitPitchRate;
	Bool							m_scatterNearbyOnExit;
	Bool							m_orientLikeContainerOnExit;
	Bool							m_keepContainerVelocityOnExit;
	AsciiString				m_exitBone;
	InitialPayload		m_initialPayload;
	Real							m_healthRegen;

	MobNexusContainModuleData();

	static void buildFieldParse(MultiIniFieldParse& p);
	static void parseInitialPayload( INI* ini, void *instance, void *store, const void* /*userData*/ );

};

//-------------------------------------------------------------------------------------------------
class MobNexusContain : public OpenContain,
												public TransportPassengerInterface // lorenzen add a MobMemberInterface
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( MobNexusContain, "MobNexusContain" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( MobNexusContain, MobNexusContainModuleData )

public:

	MobNexusContain( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration
																																								// lorenzen add a MobMemberInterface
																																								// lorenzen add a MobMemberInterface
																																								// lorenzen add a MobMemberInterface
	virtual TransportPassengerInterface* getTransportPassengerInterface() override { return this; }// lorenzen add a MobMemberInterface
																																								// lorenzen add a MobMemberInterface
																																								// lorenzen add a MobMemberInterface
																																								// lorenzen add a MobMemberInterface
	virtual Bool isValidContainerFor( const Object* obj, Bool checkCapacity) const override;

	virtual void onContaining( Object *obj, Bool wasSelected ) override;		///< object now contains 'obj'
	virtual void onRemoving( Object *obj ) override;			///< object no longer contains 'obj'
	virtual UpdateSleepTime update() override;							///< called once per frame

	virtual Int getContainMax() const override;

	virtual void onObjectCreated() override;
	virtual Int getExtraSlotsInUse() override { return m_extraSlotsInUse; }

	virtual ExitDoorType reserveDoorForExit( const ThingTemplate* objType, Object *specificObject ) override;					///< All types can answer if they are free to exit or not, and you can ask about a specific guy or just exit anything in general
	virtual void unreserveDoorForExit( ExitDoorType exitDoor ) override;

	virtual Bool tryToEvacuate( Bool exposeStealthedUnits ) override; ///< Will try to kick everybody out with game checks, and will return whether anyone made it

protected:

private:

	Int m_extraSlotsInUse;

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Contain/MobNexusContain.cpp`
```cpp
// FILE: MobNexusContain.cpp //////////////////////////////////////////////////////////////////////
// Author: Mark Lorenzen, August 2002
// Desc:   Contain module for mob units.
///////////////////////////////////////////////////////////////////////////////////////////////////

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine
#include "Common/Player.h"
#include "Common/ThingTemplate.h"
#include "Common/ThingFactory.h"
#include "Common/Xfer.h"
#include "GameClient/Drawable.h"
#include "GameLogic/AI.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/Locomotor.h"
#include "GameLogic/Module/StealthUpdate.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/MobNexusContain.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Object.h"

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
MobNexusContainModuleData::MobNexusContainModuleData()
{

	m_slotCapacity = 0;
	m_scatterNearbyOnExit = true;
	m_orientLikeContainerOnExit = false;
	m_keepContainerVelocityOnExit = false;
	m_exitPitchRate = 0.0f;
	m_initialPayload.count = 0;
	m_healthRegen = 0.0f;

	//
	// by default we say that MobNexae can have infantry inside them, this will be totally
	// overwritten by any data provided from the INI entry tho
	//
	m_allowInsideKindOf = MAKE_KINDOF_MASK( KINDOF_INFANTRY );

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void MobNexusContainModuleData::parseInitialPayload( INI* ini, void *instance, void *store, const void* /*userData*/ )
{
	MobNexusContainModuleData* self = (MobNexusContainModuleData*)instance;
	const char* name = ini->getNextToken();
	const char* countStr = ini->getNextTokenOrNull();
	Int count = countStr ? INI::scanInt(countStr) : 1;

	self->m_initialPayload.name.set(name);
	self->m_initialPayload.count = count;
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void MobNexusContainModuleData::buildFieldParse(MultiIniFieldParse& p)
{
  OpenContainModuleData::buildFieldParse(p);

	static const FieldParse dataFieldParse[] =
	{
		{ "Slots",	INI::parseInt,		nullptr, offsetof( MobNexusContainModuleData, m_slotCapacity ) },
		{ "ScatterNearbyOnExit",	INI::parseBool,		nullptr, offsetof( MobNexusContainModuleData, m_scatterNearbyOnExit ) },
		{ "OrientLikeContainerOnExit",	INI::parseBool,		nullptr, offsetof( MobNexusContainModuleData, m_orientLikeContainerOnExit ) },
		{ "KeepContainerVelocityOnExit",	INI::parseBool,		nullptr, offsetof( MobNexusContainModuleData, m_keepContainerVelocityOnExit ) },
		{ "ExitBone",	INI::parseAsciiString,		nullptr, offsetof( MobNexusContainModuleData, m_exitBone ) },
		{ "ExitPitchRate",	INI::parseAngularVelocityReal,		nullptr, offsetof( MobNexusContainModuleData, m_exitPitchRate ) },
		{ "InitialPayload", parseInitialPayload, nullptr, 0 },
		{ "HealthRegen%PerSec", INI::parseReal, nullptr, offsetof( MobNexusContainModuleData, m_healthRegen ) },
		{ nullptr, nullptr, nullptr, 0 }
	};
  p.add(dataFieldParse);
}



// PRIVATE ////////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Int MobNexusContain::getContainMax() const
{
	if (getMobNexusContainModuleData())
		return getMobNexusContainModuleData()->m_slotCapacity;

	return 0;
}

// PUBLIC /////////////////////////////////////////////////////////////////////////////////////////
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
MobNexusContain::MobNexusContain( Thing *thing, const ModuleData *moduleData ) :
								 OpenContain( thing, moduleData )
{
	m_extraSlotsInUse = 0;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
MobNexusContain::~MobNexusContain()
{

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
/**
	can this container contain this kind of object?
	and, if checkCapacity is TRUE, does this container have enough space left to hold the given unit?
*/
Bool MobNexusContain::isValidContainerFor(const Object* rider, Bool checkCapacity) const
{

	// sanity
	if (!rider)
		return false;

	//The point of this new code is to determine when something has a negative 1 contain max, to
	//look at the object inside of it to use that as the valid check. There is a case, when a
	//paratrooper (an infantry contained in a parachute). In this case, when we pass this object
	//to contain in a --- plane, we want to check the infantry, not the parachute.
//	const MobNexusContainModuleData *modData = getMobNexusContainModuleData();

	//Check if we are a fake container, and if so, get an object inside it to see what kind this object *is*.
	if( rider->getContain() && rider->getContain()->isSpecialZeroSlotContainer() )
	{
		//Report the first thing inside it!
		const ContainedItemsList *items = rider->getContain()->getContainedItemsList();
		if( items )
		{
			ContainedItemsList::const_iterator it;
			it = items->begin();
			if( *it )
			{
				//Replace the object we are checking with the *first* object contained within it.
				rider = *it;
			}
		}
	}
	else
	{
		//blech! This case may or may not occur... in which case, just use the supplied object.
	}


	// extend functionality
	if( OpenContain::isValidContainerFor( rider, checkCapacity ) == false )
		return false;

	// only allied objects can be MobNexused.
	// order matters: we want to know if IT considers ME to be an ally (a reversal of the usual situation)
	if (rider->getRelationship(getObject()) != ALLIES)
		return false;

	Int mobNexusSlotCount = rider->getTransportSlotCount();

	// if 0, this object isn't MobNexusable.
	if (mobNexusSlotCount == 0)
		return false;

	if (checkCapacity)
	{
		return (m_extraSlotsInUse + getContainCount() + mobNexusSlotCount <= getContainMax());
	}
	else
	{
		return true;
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void MobNexusContain::onContaining( Object *rider, Bool wasSelected )
{
	OpenContain::onContaining( rider, wasSelected );

	// objects inside a MobNexus are held
	rider->setDisabled( DISABLED_HELD );

	Int mobNexusSlotCount = rider->getTransportSlotCount();

	DEBUG_ASSERTCRASH(mobNexusSlotCount > 0, ("Hmm, this object isnt MobNexusable"));
	m_extraSlotsInUse += mobNexusSlotCount - 1;
	DEBUG_ASSERTCRASH(m_extraSlotsInUse >= 0 && m_extraSlotsInUse + getContainCount() <= getContainMax(), ("Hmm, bad slot count"));

	//
	// when we go from holding nothing to holding something we have a model condition
	// to visually show the change
	//
	if( getContainCount() == 1 )
	{
		Drawable *draw = getObject()->getDrawable();

		if( draw )
			draw->setModelConditionState( MODELCONDITION_LOADED );

	}

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void MobNexusContain::onRemoving( Object *rider )
{
	OpenContain::onRemoving(rider);

	// object is no longer held inside a MobNexus
	rider->clearDisabled( DISABLED_HELD );

	const MobNexusContainModuleData* d = getMobNexusContainModuleData();

	if (!d->m_exitBone.isEmpty())
	{
		Drawable* draw = getObject()->getDrawable();
		if (draw)
		{
			Coord3D bonePos, worldPos;
			if (draw->getPristineBonePositions(d->m_exitBone.str(), 0, &bonePos, nullptr, 1) == 1)
			{
				getObject()->convertBonePosToWorldPos(&bonePos, nullptr, &worldPos, nullptr);
				rider->setPosition(&worldPos);
			}
		}
	}

	if (d->m_orientLikeContainerOnExit)
	{
		rider->setOrientation(getObject()->getOrientation());
	}

	if (d->m_keepContainerVelocityOnExit)
	{
		PhysicsBehavior* parent = getObject()->getPhysics();
		PhysicsBehavior* child = rider->getPhysics();
		if (parent && child)
		{
			Coord3D startingForce = *parent->getVelocity();
			Real mass = child->getMass();
			startingForce.x *= mass;
			startingForce.y *= mass;
			startingForce.z *= mass;
			child->applyMotiveForce( &startingForce );

			Real pitchRate = child->getCenterOfMassOffset() * d->m_exitPitchRate;
			child->setPitchRate( pitchRate );
		}
	}

	if (d->m_scatterNearbyOnExit)
		scatterToNearbyPosition(rider);

	Int mobNexusSlotCount = rider->getTransportSlotCount();
	DEBUG_ASSERTCRASH(mobNexusSlotCount > 0, ("This object isnt MobNexusable"));
	m_extraSlotsInUse -= mobNexusSlotCount - 1;
	DEBUG_ASSERTCRASH(m_extraSlotsInUse >= 0 && m_extraSlotsInUse + getContainCount() <= getContainMax(), ("Bad slot count, MobNexus"));

	// when we are empty again, clear the model condition for loaded
	if( getContainCount() == 0 )
	{
		Drawable *draw = getObject()->getDrawable();

		if( draw )
			draw->clearModelConditionState( MODELCONDITION_LOADED );

	}

	if (getObject()->isAboveTerrain())
	{
		// temporarily mark the guy as being allowed to fall
		// (overriding his locomotor's stick-to-ground attribute).
		// this will be reset (by PhysicsBehavior) when he touches the ground.
		PhysicsBehavior* physics = rider->getPhysics();
		if (physics)
			physics->setAllowToFall(true);
	}

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void MobNexusContain::onObjectCreated()
{
	MobNexusContainModuleData* self = (MobNexusContainModuleData*)getMobNexusContainModuleData();

	Int count = self->m_initialPayload.count;
	const ThingTemplate* payloadTemplate = TheThingFactory->findTemplate( self->m_initialPayload.name );
	Object* object = getObject();

	for( int i = 0; i < count; i++ )
	{
		//We are creating a MobNexus that comes with a initial payload, so add it now!

		Object* payload = TheThingFactory->newObject( payloadTemplate, object->getControllingPlayer()->getDefaultTeam() );
		if( object->getContain() && object->getContain()->isValidContainerFor( payload, true ) )
		{
			object->getContain()->addToContain( payload );
		}
		else
		{
			DEBUG_CRASH( ( "DeliverPayload: PutInContainer %s is full, or not valid for the payload %s!", object->getName().str(), self->m_initialPayload.name.str() ) );
		}
	}
}


// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
UpdateSleepTime MobNexusContain::update()
{
	MobNexusContainModuleData *moduleData = (MobNexusContainModuleData*)getModuleData();

	if( moduleData && moduleData->m_healthRegen )
	{
		ContainModuleInterface *contain = getObject()->getContain();
		if( contain )
		{
			//This MobNexus has a health regeneration value, so go through and heal all inside.
			const ContainedItemsList* items = contain->getContainedItemsList();
			if( items )
			{
				ContainedItemsList::const_iterator it;
				it = items->begin();

				while( it != items->end() )
				{
					Object *object = *it;

					//Advance to the next iterator
					it++;

					//Determine if we need healing or not.
					BodyModuleInterface *body = object->getBodyModule();
					if( body->getHealth() < body->getMaxHealth() )
					{
						//Calculate the health to be regenerated on each unit.
						Real regen = body->getMaxHealth() * moduleData->m_healthRegen / 100.0f * SECONDS_PER_LOGICFRAME_REAL;

						//Perform the actual healing for this frame.
//						DamageInfo damageInfo;
//						damageInfo.in.m_damageType = DAMAGE_HEALING;
//						damageInfo.in.m_deathType = DEATH_NONE;
//						damageInfo.in.m_sourceID = getObject()->getID();
//						damageInfo.in.m_amount = regen;
//						object->attemptDamage( &damageInfo );
						object->attemptHealing( regen, getObject() );
					}
				}
			}
		}
	}

	return OpenContain::update(); //extend
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
ExitDoorType MobNexusContain::reserveDoorForExit( const ThingTemplate* objType, Object *specificObject )
{
	if( specificObject == nullptr )
		return DOOR_1;// I can, in general, exit people.

	// This is an override, not an extend.  I will check for game legality for
	// okaying the call to exitObjectViaDoor.
  Object *me = getObject();

	// this is present solely for some MobNexuss to override, so that they can land before
	// allowing people to exit...
	AIUpdateInterface* ai = me->getAIUpdateInterface();
	if (ai && ai->getAiFreeToExit(me) != FREE_TO_EXIT)
		return DOOR_NONE_AVAILABLE;

  // I can always kick people out if I am in the air, I know what I'm doing
  if( me->isUsingAirborneLocomotor() )
   	return DOOR_1;

  const Coord3D *myPosition = me->getPosition();
 	if( !specificObject->getAIUpdateInterface() )
	{
		return DOOR_NONE_AVAILABLE;
	}
	const Locomotor *hisLocomotor = specificObject->getAIUpdateInterface()->getCurLocomotor();
  // He can't get to this spot naturally, so I can't force him there.  (amphib MobNexus)
  if( ! TheAI->pathfinder()->validMovementTerrain(me->getLayer(), hisLocomotor, myPosition ) )
   	return DOOR_NONE_AVAILABLE;

  return DOOR_1;
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void MobNexusContain::unreserveDoorForExit( ExitDoorType exitDoor )
{
	/* nothing */
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
Bool MobNexusContain::tryToEvacuate( Bool exposeStealthedUnits )
{
	Bool exitedAnyone = false;
	ContainedItemsList::const_iterator it = getContainList().begin();
	while( it != getContainList().end() )
	{
		// get the object
		Object *obj = *it;
		// increment the iterator, since removal will pull this guy from the list somewhere else
		// and we might not actually kick anyone so we don't want to loop forever.
		++it;

		ExitDoorType exitDoor = reserveDoorForExit(obj->getTemplate(), obj);
		if(exitDoor != DOOR_NONE_AVAILABLE)
		{
			exitObjectViaDoor( obj, exitDoor );
			exitedAnyone = true;

			if( obj->isKindOf( KINDOF_STEALTH_GARRISON ) && exposeStealthedUnits )
			{
				StealthUpdate* stealth = obj->getStealth();
				if( stealth )
				{
					stealth->markAsDetected();
				}
			}
		}
	}
	return exitedAnyone;
}

// ------------------------------------------------------------------------------------------------
/** CRC - I like the word nexus */
// ------------------------------------------------------------------------------------------------
void MobNexusContain::crc( Xfer *xfer )
{

	// extend base class
	OpenContain::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void MobNexusContain::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	OpenContain::xfer( xfer );

	// extra slots in use
	xfer->xferInt( &m_extraSlotsInUse );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void MobNexusContain::loadPostProcess()
{

	// extend base class
	OpenContain::loadPostProcess();

}
```
