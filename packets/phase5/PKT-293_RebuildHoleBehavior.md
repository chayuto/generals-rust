# Translation Packet `PKT-293`: `RebuildHoleBehavior`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-293`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/behavior/rebuild_hole_behavior.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (16):**
  - `Common/GameState.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/GameState.h`
  - `Common/ThingFactory.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ThingFactory.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/DieModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DieModule.h`
  - `GameLogic/Module/RebuildHoleBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/RebuildHoleBehavior.h`
  - `GameLogic/Module/StickyBombUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/StickyBombUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/ScriptEngine.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ScriptEngine.h`
  - `PreRTS.h`
- **Implemented Module Facets (2):**
  - `DieModuleInterface* getDie()`
  - `RebuildHoleBehaviorInterface* getRebuildHoleBehaviorInterface()`
- **INI `FieldParse` Table Entries (3):**
  - `"WorkerObjectName" => INI::parseAsciiString`
  - `"WorkerRespawnDelay" => INI::parseDurationReal`
  - `"HoleHealthRegen%PerSecond" => INI::parsePercentToReal`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (7 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferObjectID( &m_workerID )`
  3. `xferObjectID( &m_reconstructingID )`
  4. `xferObjectID( &m_spawnerObjectID )`
  5. `xferUnsignedInt( &m_workerWaitCounter )`
  6. `xferAsciiString( &workerName )`
  7. `xferAsciiString( &rebuildName )`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/RebuildHoleBehavior.h`
```cpp
// FILE: RebuildHoleBehavior.h ////////////////////////////////////////////////////////////////////
// Author: Colin Day, June 2002
// Desc:   GLA Hole behavior that reconstructs a building after death
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/DieModule.h"
#include "GameLogic/Module/UpdateModule.h"

//-------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
class RebuildHoleBehaviorModuleData : public UpdateModuleData
{

public:

	RebuildHoleBehaviorModuleData();

	static void buildFieldParse( MultiIniFieldParse &p );

	Real m_workerRespawnDelay;							///< delay in frames from death of object till respawn of worker
	Real m_holeHealthRegenPercentPerSecond; ///< the hole recovers this % of the max hit points per second
	AsciiString m_workerTemplateName;				///< name of worker object

private:

};

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
class RebuildHoleBehaviorInterface
{

public:

	virtual void startRebuildProcess( const ThingTemplate *rebuild, ObjectID spawnerID ) = 0;
	virtual ObjectID getSpawnerID() = 0;
	virtual ObjectID getReconstructedBuildingID() = 0;
	virtual const ThingTemplate* getRebuildTemplate() const = 0;

};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class RebuildHoleBehavior : public UpdateModule,
														public DieModuleInterface,
														public RebuildHoleBehaviorInterface
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( RebuildHoleBehavior, "RebuildHoleBehavior" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( RebuildHoleBehavior, RebuildHoleBehaviorModuleData )

public:

	RebuildHoleBehavior( Thing *thing, const ModuleData *moduleData );
	// virtual destructor prototype provided by memory pool declaration

	virtual RebuildHoleBehaviorInterface* getRebuildHoleBehaviorInterface() override { return this; }

	static Int getInterfaceMask() { return UpdateModule::getInterfaceMask() | (MODULEINTERFACE_DIE); }

	// BehaviorModule
	virtual DieModuleInterface* getDie() override { return this; }

	// UpdateModuleInterface
	virtual UpdateSleepTime update() override;

	// DieModuleInterface
	virtual void onDie( const DamageInfo *damageInfo ) override;

	// RebuildHole specific methods
	virtual void startRebuildProcess( const ThingTemplate *rebuild, ObjectID spawnerID ) override;
	virtual ObjectID getSpawnerID() override { return m_spawnerObjectID; }
	virtual ObjectID getReconstructedBuildingID() override { return m_reconstructingID; }
	virtual const ThingTemplate* getRebuildTemplate() const override { return m_rebuildTemplate; }
	void transferBombs( Object *reconstruction );

	// interface acquisition
	static RebuildHoleBehaviorInterface* getRebuildHoleBehaviorInterfaceFromObject( Object *obj );

protected:

	void newWorkerRespawnProcess( Object *existingWorker );		///< start the worker respawn process (again if existingWorker is non nullptr)

	ObjectID m_workerID;										///< id of the worker that will rebuild us
	ObjectID m_reconstructingID;						///< ID of the object we're reconstructing
	ObjectID m_spawnerObjectID;							///< Object that was killed and therefore the object that created this hole
	UnsignedInt m_workerWaitCounter;				///< when this reaches zero we spawn a worker after death
	const ThingTemplate *m_workerTemplate;  ///< template of the worker to make
	const ThingTemplate *m_rebuildTemplate;	///< what we are rebuilding

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Behavior/RebuildHoleBehavior.cpp`
```cpp
// FILE: RebuildHoleBehavior.cpp //////////////////////////////////////////////////////////////////
// Author: Colin Day, June 2002
// Desc:   GLA Hole behavior that reconstructs a building after death
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine
#include "Common/GameState.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"
#include "GameClient/Drawable.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/ScriptEngine.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/RebuildHoleBehavior.h"
#include "GameLogic/Module/StickyBombUpdate.h"


//-------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
RebuildHoleBehaviorModuleData::RebuildHoleBehaviorModuleData()
{

	m_workerRespawnDelay = 0.0f;
	m_holeHealthRegenPercentPerSecond = 0.1f;

}

//-------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
/*static*/ void RebuildHoleBehaviorModuleData::buildFieldParse( MultiIniFieldParse &p )
{

  UpdateModuleData::buildFieldParse( p );

	static const FieldParse dataFieldParse[] =
	{
	  { "WorkerObjectName", INI::parseAsciiString, nullptr, offsetof( RebuildHoleBehaviorModuleData, m_workerTemplateName ) },
		{ "WorkerRespawnDelay", INI::parseDurationReal,	nullptr, offsetof( RebuildHoleBehaviorModuleData, m_workerRespawnDelay ) },
		{ "HoleHealthRegen%PerSecond", INI::parsePercentToReal, nullptr, offsetof( RebuildHoleBehaviorModuleData, m_holeHealthRegenPercentPerSecond ) },
		{ nullptr, nullptr, nullptr, 0 }
	};

  p.add( dataFieldParse );

}

///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
RebuildHoleBehavior::RebuildHoleBehavior( Thing *thing, const ModuleData* moduleData )
									 : UpdateModule( thing, moduleData )
{

	m_workerID = INVALID_ID;
	m_reconstructingID = INVALID_ID;
	m_spawnerObjectID = INVALID_ID;
	m_workerWaitCounter = 0;
	m_workerTemplate = nullptr;
	m_rebuildTemplate = nullptr;

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
RebuildHoleBehavior::~RebuildHoleBehavior()
{
	// ensure that our generated worker is destroyed,
	// just in case someone decides to destroy (not kill) us...
	if( m_workerID != INVALID_ID )
	{
		Object *worker = TheGameLogic->findObjectByID(m_workerID);
		if( worker )
		{
			TheGameLogic->destroyObject(worker);
			m_workerID = INVALID_ID;
		}
	}

}

// ------------------------------------------------------------------------------------------------
/** we need to start all the timers and ID ties to make a new worker at the correct time */
// ------------------------------------------------------------------------------------------------
void RebuildHoleBehavior::newWorkerRespawnProcess( Object *existingWorker )
{
	const RebuildHoleBehaviorModuleData *modData = getRebuildHoleBehaviorModuleData();

	// if we have an existing worker, get rid of it
	if( existingWorker )
	{
		DEBUG_ASSERTCRASH(existingWorker->getID() == m_workerID, ("m_workerID mismatch in RebuildHole"));
		TheGameLogic->destroyObject( existingWorker );
	}
	m_workerID = INVALID_ID;

	// set the timer for the next worker respawn
	m_workerWaitCounter = modData->m_workerRespawnDelay;

	//
	// this method is called when a worker needs to be respawned from the hole.  One of those
	// situations is where the building was killed.  Since during building reconstruction
	// we made the hole "effectively not here" we will always want to make the hole
	// "here again" (able to be selected, targeted by the AI etc) because it's the
	// "focus" of this small area again
	//
	getObject()->maskObject( FALSE );

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void RebuildHoleBehavior::startRebuildProcess( const ThingTemplate *rebuild, ObjectID spawnerID )
{

	// save what we're gonna do
	m_rebuildTemplate = rebuild;

	// store the object that spawned this hole (even though it's likely being destroyed)
	m_spawnerObjectID = spawnerID;

	// start the spawning process for a worker
	Object *worker = TheGameLogic->findObjectByID(m_workerID);
	newWorkerRespawnProcess( worker ); //Kill the worker if we have one.

}


//----------------------------------------------------------------------------------------------
void RebuildHoleBehavior::transferBombs( Object *reconstruction )
{

	Object *self = getObject();

	Object *obj = TheGameLogic->getFirstObject();
	while( obj )
	{
		if( obj->isKindOf( KINDOF_MINE ) )
		{
			static NameKeyType key_StickyBombUpdate = NAMEKEY( "StickyBombUpdate" );
			StickyBombUpdate *update = (StickyBombUpdate*)obj->findUpdateModule( key_StickyBombUpdate );
			if( update && update->getTargetObject() == self )
			{
				update->setTargetObject( reconstruction );
			}
		}
		obj = obj->getNextObject();
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
UpdateSleepTime RebuildHoleBehavior::update()
{
	const RebuildHoleBehaviorModuleData *modData = getRebuildHoleBehaviorModuleData();
	Object *hole = getObject();
	Object *reconstructing = nullptr;
	Object *worker = nullptr;

	// get the worker object if we have one
	if( m_workerID != 0 )
	{

		// get the worker
		worker = TheGameLogic->findObjectByID( m_workerID );

		// if the worker is no longer there, start the respawning process for a worker again
		if( worker == nullptr )
			newWorkerRespawnProcess( nullptr );

	}

	// if we have a reconstructing object built, get the actual object pointer
	if( m_reconstructingID != 0 )
	{

		// get object pointer
		reconstructing = TheGameLogic->findObjectByID( m_reconstructingID );

		//
		// if that object does not exist anymore, we need to kill a worker if we have one
		// and start the spawning process over again
		//
		if( reconstructing == nullptr )
		{
			newWorkerRespawnProcess( worker );
			m_reconstructingID = INVALID_ID;
		}

	}

	// see if it's time for us to spawn a worker
	if( worker == nullptr && m_workerWaitCounter > 0 )
	{

		// decrement counter and respawn if it's time
		if( --m_workerWaitCounter == 0 )
		{

			// resolve the worker template pointer if necessary
			if( m_workerTemplate == nullptr )
				m_workerTemplate = TheThingFactory->findTemplate( modData->m_workerTemplateName );

			// create a worker
			worker = TheThingFactory->newObject( m_workerTemplate, hole->getTeam() );
			if( worker )
			{

				// set the position of the worker to that of the hole
				worker->setPosition( hole->getPosition() );

				// save the ID of the worker spawned
				m_workerID = worker->getID();

				//
				// tell the worker to begin construction of a building if one does not
				// exist yet.  If one does, have construction resume
				//
				AIUpdateInterface *ai = worker->getAIUpdateInterface();
				if( ai )
				{

					if( reconstructing == nullptr )
						reconstructing = ai->construct( m_rebuildTemplate,
																						hole->getPosition(),
																						hole->getOrientation(),
																						hole->getControllingPlayer(),
																						TRUE );
					else
						ai->aiResumeConstruction( reconstructing, CMD_FROM_AI );

					for ( Object *obj = TheGameLogic->getFirstObject(); obj; obj = obj->getNextObject() )
					{
						// Just like the building transfers attackers to the hole when it creates us, we need to transfer
						// attackers to our replacement building before we mask ourselves.
						AIUpdateInterface* ai = obj->getAI();
						if (!ai)
							continue;

						ai->transferAttack(hole->getID(), reconstructing->getID());
					}

					// save the id of what we are reconstructing
					m_reconstructingID = reconstructing->getID();

					//Kris: Hacking the building to set the hole as the producer... so if the site dies, we
					//can transfer the attack back to the hole. The object has OBJECT_STATUS_RECONSTRUCTING
					//which we check when the object dies.
					reconstructing->setProducer( hole );

					// we want to prevent the player from selecting and doing things with this worker
					worker->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_UNSELECTABLE ) );

					//
					// we want to prevent the player and the AI from selecting or targeting the hole
					// cause the focus in this area (while reconstruction is happening) is the
					// actual reconstructing building
					//
					hole->maskObject( TRUE );

					transferBombs( reconstructing );

				}

			}

		}

	}

	// holes get auto-healed when they're sitting around
	BodyModuleInterface *body = hole->getBodyModule();
	if( body->getHealth() < body->getMaxHealth() )
	{
		DamageInfo healingInfo;

		// do some healing
		healingInfo.in.m_amount = (modData->m_holeHealthRegenPercentPerSecond / static_cast<float>(LOGICFRAMES_PER_SECOND)) *
															body->getMaxHealth();
		healingInfo.in.m_sourceID = hole->getID();
		healingInfo.in.m_damageType = DAMAGE_HEALING;
		healingInfo.in.m_deathType = DEATH_NONE;
		body->attemptHealing( &healingInfo );

	}

	// when re-construction is complete, we remove this hole and worker
	if( reconstructing && !reconstructing->getStatusBits().test( OBJECT_STATUS_UNDER_CONSTRUCTION ) )
	{
		// Transfer hole name to new building
		TheScriptEngine->transferObjectName( hole->getName(), reconstructing );

		// make the worker go away
		if( worker )
			TheGameLogic->destroyObject( worker );

		// make the hole go away
		TheGameLogic->destroyObject( hole );

	}

	return UPDATE_SLEEP_NONE;

}

// ------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void RebuildHoleBehavior::onDie( const DamageInfo *damageInfo )
{
	if( m_workerID != INVALID_ID )
	{
		// Our rebuilding building and us the hole can be killed in the same frame, which means we may not have
		// deleted our generated worker since we do that in our update.
		Object *worker = TheGameLogic->findObjectByID(m_workerID);
		if( worker )
		{
			TheGameLogic->destroyObject(worker);
			m_workerID = INVALID_ID;
		}
	}

	Object *obj = getObject();

	// destroy us
	TheGameLogic->destroyObject( obj );

}

// ------------------------------------------------------------------------------------------------
/** Helper method to get interface given an object */
// ------------------------------------------------------------------------------------------------
/*static*/ RebuildHoleBehaviorInterface* RebuildHoleBehavior::getRebuildHoleBehaviorInterfaceFromObject( Object *obj )
{
	RebuildHoleBehaviorInterface *rhbi = nullptr;

	if( obj )
	{

		for( BehaviorModule **i = obj->getBehaviorModules(); *i; ++i )
		{

			rhbi = (*i)->getRebuildHoleBehaviorInterface();
			if( rhbi )
				break;  // exit for

		}

	}

	return rhbi;

}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void RebuildHoleBehavior::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version,
	* 2: Added spawner id */
// ------------------------------------------------------------------------------------------------
void RebuildHoleBehavior::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 2;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// worker ID
	xfer->xferObjectID( &m_workerID );

	// reconstructing id
	xfer->xferObjectID( &m_reconstructingID );

	// spawner ID
	if( version >= 2 )
		xfer->xferObjectID( &m_spawnerObjectID );

	// worker wait counter
	xfer->xferUnsignedInt( &m_workerWaitCounter );

	// worker template
	AsciiString workerName = m_workerTemplate ? m_workerTemplate->getName() : AsciiString::TheEmptyString;
	xfer->xferAsciiString( &workerName );
	if( xfer->getXferMode() == XFER_LOAD )
	{

		if( workerName != AsciiString::TheEmptyString )
		{

			m_workerTemplate = TheThingFactory->findTemplate( workerName );
			if( m_workerTemplate == nullptr )
			{

				DEBUG_CRASH(( "RebuildHoleBehavior::xfer - Unable to find template '%s'",
											workerName.str() ));
				throw SC_INVALID_DATA;

			}

		}
		else
			m_workerTemplate = nullptr;

	}

	// rebuild template
	AsciiString rebuildName = m_rebuildTemplate ? m_rebuildTemplate->getName() : AsciiString::TheEmptyString;
	xfer->xferAsciiString( &rebuildName );
	if( xfer->getXferMode() == XFER_LOAD )
	{

		if( rebuildName != AsciiString::TheEmptyString )
		{

			m_rebuildTemplate = TheThingFactory->findTemplate( rebuildName );
			if( m_rebuildTemplate == nullptr )
			{

				DEBUG_CRASH(( "RebuildHoleBehavior::xfer - Unable to find template '%s'",
											rebuildName.str() ));
				throw SC_INVALID_DATA;

			}

		}
		else
			m_rebuildTemplate = nullptr;

	}

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void RebuildHoleBehavior::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
