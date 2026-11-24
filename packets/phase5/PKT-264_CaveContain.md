# Translation Packet `PKT-264`: `CaveContain`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-264`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/contain/cave_contain.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (16):**
  - `Common/GameState.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/GameState.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/PlayerList.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/PlayerList.h`
  - `Common/Team.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Team.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/TunnelTracker.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/TunnelTracker.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameLogic/CaveSystem.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/CaveSystem.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/CaveContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CaveContain.h`
  - `GameLogic/Module/CreateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CreateModule.h`
  - `GameLogic/Module/OpenContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/OpenContain.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `PreRTS.h`
- **Implemented Module Facets (2):**
  - `CaveInterface* getCaveInterface()`
  - `CreateModuleInterface* getCreate()`
- **INI `FieldParse` Table Entries (1):**
  - `"CaveIndex" => INI::parseInt`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (4 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferBool( &m_needToRunOnBuildComplete )`
  3. `xferInt( &m_caveIndex )`
  4. `xferUser( &teamID, sizeof( TeamID ) )`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CaveContain.h`
```cpp
// FILE: CaveContain.h ////////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, July 2002
// Desc:   A version of OpenContain that overrides where the passengers are stored: one of CaveManager's
//					entries. Changing entry is a script or ini command.  All queries about capacity and
//					contents are also redirected.
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/CreateModule.h"
#include "GameLogic/Module/OpenContain.h"

struct Sound;
class Team;

//-------------------------------------------------------------------------------------------------
class CaveContainModuleData : public OpenContainModuleData
{
public:
	Int m_caveIndexData;

	CaveContainModuleData()
	{
		m_caveIndexData = 0;// By default, all Caves will be grouped together as number 0
	}

	static void buildFieldParse(MultiIniFieldParse& p)
	{
    OpenContainModuleData::buildFieldParse(p);

		static const FieldParse dataFieldParse[] =
		{
			{ "CaveIndex", INI::parseInt, nullptr, offsetof( CaveContainModuleData, m_caveIndexData ) },
			{ nullptr, nullptr, nullptr, 0 }
		};
    p.add(dataFieldParse);
	}
};

//-------------------------------------------------------------------------------------------------
class CaveContain : public OpenContain, public CreateModuleInterface, public CaveInterface
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( CaveContain, "CaveContain" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( CaveContain, CaveContainModuleData )

public:

	CaveContain( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	virtual CreateModuleInterface* getCreate() override { return this; }
	virtual CaveInterface* getCaveInterface() override { return this; }
	static Int getInterfaceMask() { return OpenContain::getInterfaceMask() | (MODULEINTERFACE_CREATE); }

	virtual OpenContain *asOpenContain() override { return this; }  ///< treat as open container
	virtual Bool isGarrisonable() const override { return false; }	///< can this unit be Garrisoned? (ick)
	virtual Bool isBustable() const override { return TRUE; }	///< can this container get busted by a bunkerbuster
	virtual Bool isHealContain() const override { return false; } ///< true when container only contains units while healing (not a transport!)

	virtual void onContaining( Object *obj, Bool wasSelected ) override;		///< object now contains 'obj'
	virtual void onRemoving( Object *obj ) override;			///< object no longer contains 'obj'

	virtual Bool isValidContainerFor(const Object* obj, Bool checkCapacity) const override;
	virtual void addToContainList( Object *obj ) override;		///< The part of AddToContain that inheritors can override (Can't do whole thing because of all the private stuff involved)
	virtual void removeFromContain( Object *obj, Bool exposeStealthUnits = FALSE ) override;	///< remove 'obj' from contain list
	virtual void removeAllContained( Bool exposeStealthUnits = FALSE ) override;				///< remove all objects on contain list

	/**
		return the player that *appears* to control this unit. if null, use getObject()->getControllingPlayer() instead.
	*/
	virtual void recalcApparentControllingPlayer() override;

	// contain list access
	virtual void iterateContained( ContainIterateFunc func, void *userData, Bool reverse ) override;
	virtual UnsignedInt getContainCount() const override;
	virtual Int getContainMax() const override;
	virtual const ContainedItemsList* getContainedItemsList() const override;
	virtual Bool isContained( const Object *obj ) const override;
	virtual Bool isKickOutOnCapture() override { return FALSE; }///< Caves and Tunnels don't kick out on capture.

	// override the onDie we inherit from OpenContain
	virtual void onDie( const DamageInfo *damageInfo ) override;  ///< the die callback

	virtual void onCreate() override;
	virtual void onBuildComplete() override;	///< This is called when you are a finished game object
	virtual Bool shouldDoOnBuildComplete() const override { return m_needToRunOnBuildComplete; }

	// Unique to Cave Contain
	virtual void tryToSetCaveIndex( Int newIndex ) override;	///< Called by script as an alternative to instancing separate objects.  'Try', because can fail.
	virtual void setOriginalTeam( Team *oldTeam ) override;	///< This is a distributed Garrison in terms of capturing, so when one node triggers the change, he needs to tell everyone, so anyone can do the un-change.

protected:

	void changeTeamOnAllConnectedCaves( Team *newTeam, Bool setOriginalTeams );	///< When one gets captured, all connected ones get captured.  DistributedGarrison.

	Bool m_needToRunOnBuildComplete;
	Int m_caveIndex;

	Team *m_originalTeam;												///< our original team before we were garrisoned

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Contain/CaveContain.cpp`
```cpp
// FILE: CaveContain.cpp ////////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, July 2002
// Desc:   A version of OpenContain that overrides where the passengers are stored: one of CaveSystem's
//					entries. Changing entry is a script or ini command.  All queries about capacity and
//					contents are also redirected.  They change sides like Garrison too.
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine
#include "Common/GameState.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/Team.h"
#include "Common/ThingTemplate.h"
#include "Common/TunnelTracker.h"
#include "Common/Xfer.h"
#include "GameClient/Drawable.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/CaveContain.h"
#include "GameLogic/CaveSystem.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"

///////////////////////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS ///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
CaveContain::CaveContain( Thing *thing, const ModuleData* moduleData ) : OpenContain( thing, moduleData )
{
	m_needToRunOnBuildComplete = true;
	m_caveIndex = 0;
	m_originalTeam = nullptr;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
CaveContain::~CaveContain()
{
}

void CaveContain::addToContainList( Object *obj )
{
	TunnelTracker *myTracker = TheCaveSystem->getTunnelTrackerForCaveIndex( m_caveIndex );
	myTracker->addToContainList( obj );
}

//-------------------------------------------------------------------------------------------------
/** Remove 'obj' from the m_containList of objects in this module.
	* This will trigger an onRemoving event for the object that this module
	* is a part of and an onRemovedFrom event for the object being removed */
//-------------------------------------------------------------------------------------------------
void CaveContain::removeFromContain( Object *obj, Bool exposeStealthUnits )
{

	// sanity
	if( obj == nullptr )
		return;

	//
	// we can only remove this object from the contains list of this module if
	// it is actually contained by this module
	//
	TunnelTracker *myTracker = TheCaveSystem->getTunnelTrackerForCaveIndex( m_caveIndex );

	if( ! myTracker->isInContainer( obj ) )
	{
		return;
	}

	// This must come before the onRemov*, because CaveContain's version has a edge-0 triggered event.
	// If that were to go first, the number would still be 1 at that time.  Noone else cares about
	// order.
	myTracker->removeFromContain( obj, exposeStealthUnits );

	// trigger an onRemoving event for 'm_object' no longer containing 'itemToRemove->m_object'
	if (getObject()->getContain())
		getObject()->getContain()->onRemoving( obj );

	// trigger an onRemovedFrom event for 'remove'
	obj->onRemovedFrom( getObject() );

}

//-------------------------------------------------------------------------------------------------
/** Remove all contained objects from the contained list */
//-------------------------------------------------------------------------------------------------
void CaveContain::removeAllContained( Bool exposeStealthUnits )
{
	TunnelTracker *myTracker = TheCaveSystem->getTunnelTrackerForCaveIndex( m_caveIndex );
	const ContainedItemsList *fullList = myTracker->getContainedItemsList();

	Object *obj;
	ContainedItemsList::const_iterator it;
	it = (*fullList).begin();
	while( it != (*fullList).end() )
	{
		obj = *it;
		it++;
		removeFromContain( obj, exposeStealthUnits );
	}
}

//-------------------------------------------------------------------------------------------------
/** Iterate the contained list and call the callback on each of the objects */
//-------------------------------------------------------------------------------------------------
void CaveContain::iterateContained( ContainIterateFunc func, void *userData, Bool reverse )
{
	TunnelTracker *myTracker = TheCaveSystem->getTunnelTrackerForCaveIndex( m_caveIndex );
	myTracker->iterateContained( func, userData, reverse );
}

//-------------------------------------------------------------------------------------------------
void CaveContain::onContaining( Object *obj, Bool wasSelected )
{
	OpenContain::onContaining( obj, wasSelected );
	// objects inside a building are held
	obj->setDisabled( DISABLED_HELD );

	//
	// the team of the building is now the same as those that have garrisoned it, be sure
	// to save our original team tho so that we can revert back to it when all the
	// occupants are gone
	//
	recalcApparentControllingPlayer();

}

//-------------------------------------------------------------------------------------------------
// TheSuperHackers @info A whole network shares one passenger list, so a passenger is contained by
// the endpoint it entered and not by the one that was ordered to unload.
Bool CaveContain::isContained( const Object *obj ) const
{
	if (OpenContain::isContained(obj))
		return TRUE;

	const ContainedItemsList *items = getContainedItemsList();
	return items != nullptr && std::find(items->begin(), items->end(), obj) != items->end();
}

//-------------------------------------------------------------------------------------------------
void CaveContain::onRemoving( Object *obj )
{
	OpenContain::onRemoving(obj);
	// object is no longer held inside a garrisoned building
	obj->clearDisabled( DISABLED_HELD );

	/// place the object in the world at position of the container m_object
	ThePartitionManager->registerObject( obj );
	obj->setPosition( getObject()->getPosition() );
	if( obj->getDrawable() )
	{
		obj->getDrawable()->setDrawableHidden( false );
	}

	doUnloadSound();

	if( getContainCount() == 0 )
	{

		// put us back on our original team
		// (hokey exception: if our team is null, don't bother -- this
		// usually means we are being called during game-teardown and
		// the teams are no longer valid...)
		if (getObject()->getTeam() != nullptr)
		{
			changeTeamOnAllConnectedCaves( m_originalTeam, FALSE );
			m_originalTeam = nullptr;
		}

		// change the state back from garrisoned
		Drawable *draw = getObject()->getDrawable();
		if( draw )
		{
			draw->clearModelConditionState( MODELCONDITION_GARRISONED );
		}

	}
}

Bool CaveContain::isValidContainerFor(const Object* obj, Bool checkCapacity) const
{
	TunnelTracker *myTracker = TheCaveSystem->getTunnelTrackerForCaveIndex( m_caveIndex );
	return myTracker->isValidContainerFor( obj, checkCapacity );
}

UnsignedInt CaveContain::getContainCount() const
{
	TunnelTracker *myTracker = TheCaveSystem->getTunnelTrackerForCaveIndex( m_caveIndex );
	return myTracker->getContainCount();
}

Int CaveContain::getContainMax() const
{
	TunnelTracker *myTracker = TheCaveSystem->getTunnelTrackerForCaveIndex( m_caveIndex );
	return myTracker->getContainMax();
}

const ContainedItemsList* CaveContain::getContainedItemsList() const
{
	TunnelTracker *myTracker = TheCaveSystem->getTunnelTrackerForCaveIndex( m_caveIndex );
	return myTracker->getContainedItemsList();
}

//-------------------------------------------------------------------------------------------------
/** The die callback. */
//-------------------------------------------------------------------------------------------------
void CaveContain::onDie( const DamageInfo * damageInfo )
{
	// override the onDie we inherit from OpenContain. no super call.
	if (!getCaveContainModuleData()->m_dieMuxData.isDieApplicable(getObject(), damageInfo))
		return;

	if( getObject()->getStatusBits().test( OBJECT_STATUS_UNDER_CONSTRUCTION ) )
		return;//it never registered itself as a tunnel

	TunnelTracker *myTracker = TheCaveSystem->getTunnelTrackerForCaveIndex( m_caveIndex );

	TheCaveSystem->unregisterCave( m_caveIndex );

	myTracker->onTunnelDestroyed( getObject() );
}


//-------------------------------------------------------------------------------------------------
void CaveContain::onCreate()
{
	m_caveIndex = getCaveContainModuleData()->m_caveIndexData;
}

//-------------------------------------------------------------------------------------------------
void CaveContain::onBuildComplete()
{
	if( ! shouldDoOnBuildComplete() )
		return;

	m_needToRunOnBuildComplete = false;

	TheCaveSystem->registerNewCave( m_caveIndex );

	TunnelTracker *myTracker = TheCaveSystem->getTunnelTrackerForCaveIndex( m_caveIndex );

	myTracker->onTunnelCreated( getObject() );
}

//-------------------------------------------------------------------------------------------------
void CaveContain::tryToSetCaveIndex( Int newIndex )
{
	if( TheCaveSystem->canSwitchIndexToIndex( m_caveIndex, newIndex ) )
	{
		TunnelTracker *myOldTracker = TheCaveSystem->getTunnelTrackerForCaveIndex( m_caveIndex );
		TheCaveSystem->unregisterCave( m_caveIndex );
		myOldTracker->onTunnelDestroyed( getObject() );

		m_caveIndex = newIndex;

		TheCaveSystem->registerNewCave( m_caveIndex );
		TunnelTracker *myNewTracker = TheCaveSystem->getTunnelTrackerForCaveIndex( m_caveIndex );
		myNewTracker->onTunnelCreated( getObject() );
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void CaveContain::recalcApparentControllingPlayer()
{
	//Record original team first time through.
	if( m_originalTeam == nullptr )
	{
		m_originalTeam = getObject()->getTeam();
	}

	// (hokey trick: if our team is null, nuke originalTeam -- this
	// usually means we are being called during game-teardown and
	// the teams are no longer valid...)
	if (getObject()->getTeam() == nullptr)
		m_originalTeam = nullptr;

	// This is called from onContaining, so a one is the edge trigger to do capture stuff
	if( getContainCount() == 1 )
	{
		ContainedItemsList::const_iterator it = getContainedItemsList()->begin();
		Object *rider = *it;

		// This also gets called during reset from the PlayerList, so we might not actually have players.
		// Check like the hokey trick mentioned above
		if( rider->getControllingPlayer() )
			changeTeamOnAllConnectedCaves( rider->getControllingPlayer()->getDefaultTeam(), TRUE );

	}
	else if( getContainCount() == 0 )
	{
		// And a 0 is the edge trigger to do uncapture stuff
		changeTeamOnAllConnectedCaves( m_originalTeam, FALSE );
	}

	// Handle the team color that is rendered
	const Player* controller = getApparentControllingPlayer(ThePlayerList->getLocalPlayer());
	if (controller)
	{
		if (TheGlobalData->m_timeOfDay == TIME_OF_DAY_NIGHT)
			getObject()->getDrawable()->setIndicatorColor( controller->getPlayerNightColor() );
		else
			getObject()->getDrawable()->setIndicatorColor( controller->getPlayerColor() );
	}
}

/////////////////////////////////////////////////////////////////////////////////////
static CaveInterface* findCave(Object* obj)
{
	for (BehaviorModule** i = obj->getBehaviorModules(); *i; ++i)
	{
		CaveInterface* c = (*i)->getCaveInterface();
		if (c != nullptr)
			return c;
	}
	return nullptr;
}

/////////////////////////////////////////////////////////////////////////////////////
void CaveContain::changeTeamOnAllConnectedCaves( Team *newTeam, Bool setOriginalTeams )
{
	TunnelTracker *myTracker = TheCaveSystem->getTunnelTrackerForCaveIndex( m_caveIndex );
	const std::list<ObjectID> *allCaves = myTracker->getContainerList();
	for( std::list<ObjectID>::const_iterator iter = allCaves->begin(); iter != allCaves->end(); iter++ )
	{
		// For each ID, look it up and change its team.  We all get captured together.
		Object *currentCave = TheGameLogic->findObjectByID( *iter );
		if( currentCave )
		{
			// This is a distributed Garrison in terms of capturing, so when one node
			// triggers the change, he needs to tell everyone, so anyone can do the un-change.
			CaveInterface *caveModule = findCave(currentCave);
			if( caveModule == nullptr )
				continue;
			if( setOriginalTeams )
				caveModule->setOriginalTeam( currentCave->getTeam() );
			else
				caveModule->setOriginalTeam( nullptr );

			// Now do the actual switch for this one.

			currentCave->defect( newTeam, 0 );
//			currentCave->setTeam( newTeam );
		}
	}
}

/////////////////////////////////////////////////////////////////////////////////////
void CaveContain::setOriginalTeam( Team *oldTeam )
{
	m_originalTeam = oldTeam;
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void CaveContain::crc( Xfer *xfer )
{

	// extend base class
	OpenContain::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void CaveContain::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	OpenContain::xfer( xfer );

	// need to run on build complete
	xfer->xferBool( &m_needToRunOnBuildComplete );

	// cave index
	xfer->xferInt( &m_caveIndex );

	// original team
	TeamID teamID = m_originalTeam ? m_originalTeam->getID() : TEAM_ID_INVALID;
	xfer->xferUser( &teamID, sizeof( TeamID ) );
	if( xfer->getXferMode() == XFER_LOAD )
	{

		if( teamID != TEAM_ID_INVALID )
		{

			m_originalTeam = TheTeamFactory->findTeamByID( teamID );
			if( m_originalTeam == nullptr )
			{

				DEBUG_CRASH(( "CaveContain::xfer - Unable to find original team by id" ));
				throw SC_INVALID_DATA;

			}

		}
		else
			m_originalTeam = nullptr;

	}

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void CaveContain::loadPostProcess()
{

	// extend base class
	OpenContain::loadPostProcess();

}
```
