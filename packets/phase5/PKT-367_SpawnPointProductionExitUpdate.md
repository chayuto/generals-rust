# Translation Packet `PKT-367`: `SpawnPointProductionExitUpdate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-367`
- **Work Package:** `WP-5.4` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/update/spawn_point_production_exit_update.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (13):**
  - `Common/INI.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/INI.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameLogic/AI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/AI.h`
  - `GameLogic/AIPathfind.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/AIPathfind.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/SpawnPointProductionExitUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpawnPointProductionExitUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/TerrainLogic.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/TerrainLogic.h`
  - `Lib/BaseType.h`
  - `PreRTS.h`
  - `WWMath/matrix3d.h`
- **Implemented Module Facets (1):**
  - `ExitInterface* getUpdateExitInterface()`
- **INI `FieldParse` Table Entries (1):**
  - `"SpawnPointBoneName" => INI::parseAsciiString`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (2 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUser( &m_spawnPointOccupier, sizeof( ObjectID ) * MAX_SPAWN_POINTS )`

## 2.5 Existing Rust Symbol Index (Use These Exact Paths)
- `zh_logic::modules::update::base_renerate_update`: `BaseRegenerateUpdateModuleData`, `BaseRegenerateUpdate`
- `zh_logic::modules::update::deletion_update`: `DeletionUpdateModuleData`, `DeletionUpdate`
- `zh_logic::modules::update::dock_update`: `DockUpdateModuleData`, `FACET_MASK`, `DockUpdateState`
- `zh_logic::modules::update::dynamic_geometry_info_update`: `DIRECTION_FORWARD`, `DynamicGeometryInfoUpdateModuleData`, `DynamicGeometryInfoUpdate`
- `zh_logic::modules::update::dynamic_shroud_clearing_range_update`: `DynamicShroudClearingRangeUpdateModuleData`, `DscruState`, `DynamicShroudClearingRangeUpdate`
- `zh_logic::modules::update::fire_spread_update`: `FireSpreadUpdateModuleData`, `FireSpreadUpdate`
- `zh_logic::modules::update::fire_weapon_update`: `FireWeaponUpdateModuleData`, `FireWeaponUpdate`
- `zh_logic::modules::update::flammable_update`: `FlammableUpdateModuleData`, `FlammableUpdate`
- `zh_logic::modules::update::float_update`: `FloatUpdateModuleData`, `FloatUpdate`
- `zh_logic::modules::update::height_die_update`: `HeightDieUpdateModuleData`, `HeightDieUpdate`
- `zh_logic::modules::update::lifetime_update`: `LifetimeUpdateModuleData`, `LifetimeUpdate`
- `zh_logic::modules::update::physics_update`: `STICK_TO_GROUND`, `ALLOW_BOUNCE`, `APPLY_FRICTION2D_WHEN_AIRBORNE`, `UPDATE_EVER_RUN`, `WAS_AIRBORNE_LAST_FRAME`, `ALLOW_COLLIDE_FORCE`, `ALLOW_TO_FALL`, `HAS_PITCHROLLYAW`, `IMMUNE_TO_FALLING_DAMAGE`, `IS_IN_FREEFALL`, `IS_IN_UPDATE`, `IS_STUNNED`, `TURN_NEGATIVE`, `TURN_NONE`, ... (+4)
- `zh_logic::modules::update::power_plant_update`: `PowerPlantUpdateModuleData`, `PowerPlantUpdate`
- `zh_logic::modules::update::projectile_stream_update`: `MAX_PROJECTILE_STREAM`, `ProjectileStreamUpdateModuleData`, `ProjectileStreamUpdate`
- `zh_logic::modules::update::prone_update`: `ProneUpdateModuleData`, `ProneUpdate`
- `zh_logic::modules::update::radar_update`: `RadarUpdateModuleData`, `RadarUpdate`
- `zh_logic::modules::update::radius_decal_update`: `RadiusDecalUpdateModuleData`, `RadiusDecalUpdate`
- `zh_logic::modules::update::repair_dock_update`: `RepairDockUpdateModuleData`, `RepairDockUpdate`
- `zh_logic::modules::update::smart_bomb_target_homing_update`: `SmartBombTargetHomingUpdateModuleData`, `SmartBombTargetHomingUpdate`
- `zh_logic::modules::update::spy_vision_update`: `SpyVisionUpdateModuleData`, `SpyVisionUpdate`, `fn`
- `zh_logic::modules::update::supply_center_dock_update`: `SupplyCenterDockUpdateModuleData`, `SupplyCenterDockUpdate`
- `zh_logic::modules::update::supply_warehouse_dock_update`: `SupplyWarehouseDockUpdateModuleData`, `SupplyWarehouseDockUpdate`
- `zh_logic::modules::update::weapon_bonus_update`: `WeaponBonusUpdateModuleData`, `WeaponBonusUpdate`
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
- `zh_logic::modules::behavior::auto_heal_behavior`: `AutoHealBehaviorModuleData`, `AutoHealBehavior`
- `zh_logic::modules::behavior::battle_bus_slow_death_behavior`: `BattleBusSlowDeathBehaviorModuleData`, `BattleBusSlowDeathBehavior`
- `zh_logic::modules::behavior::behavior_module`: `BehaviorModuleData`, `fn`, `UpdateScheduleTracker`
- `zh_logic::modules::behavior::bridge_scaffold_behavior`: `BridgeScaffoldBehaviorModuleData`, `BridgeScaffoldBehavior`
- `zh_logic::modules::behavior::bridge_tower_behavior`: `BridgeTowerBehaviorModuleData`, `BridgeTowerBehavior`
- `zh_logic::modules::behavior::bunker_buster_behavior`: `BunkerBusterBehaviorModuleData`, `BunkerBusterBehavior`
- `zh_logic::modules::behavior::fire_weapon_when_damaged_behavior`: `FireWeaponWhenDamagedBehaviorModuleData`, `FireWeaponWhenDamagedBehavior`
- `zh_logic::modules::behavior::fire_weapon_when_dead_behavior`: `FireWeaponWhenDeadBehaviorModuleData`, `FireWeaponWhenDeadBehavior`
- `zh_logic::modules::behavior::grant_stealth_behavior`: `GrantStealthBehaviorModuleData`, `GrantStealthBehavior`
- `zh_logic::modules::behavior::instant_death_behavior`: `InstantDeathBehaviorModuleData`, `InstantDeathBehavior`
- `zh_logic::modules::behavior::jet_slow_death_behavior`: `JetSlowDeathBehaviorModuleData`, `JetSlowDeathBehavior`
- `zh_logic::modules::behavior::neuton_blast_behavior`: `NeutronBlastBehaviorModuleData`, `NeutronBlastBehavior`
- `zh_logic::modules::behavior::overcharge_behavior`: `OverchargeBehaviorModuleData`, `OverchargeBehavior`
- `zh_logic::modules::behavior::poisoned_behavior`: `PoisonedBehaviorModuleData`, `PoisonedBehavior`
- `zh_logic::modules::behavior::prison_behavior`: `PrisonBehaviorModuleData`, `PrisonVisual`, `PrisonBehavior`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpawnPointProductionExitUpdate.h`
```cpp
// FILE: SpawnPointProductionExitUpdate.h /////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, April, 2002
// Desc:		Hand off produced Units to me so I can Exit them into the world with my specific style
//					This instance puts guys at named bones.
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

#include "GameLogic/Module/UpdateModule.h"
#include "Common/INI.h"
#include "Lib/BaseType.h"

class Object;

enum
{
	MAX_SPAWN_POINTS = 10	///< Size the array that holds the bone positions
};

//-------------------------------------------------------------------------------------------------
class SpawnPointProductionExitUpdateModuleData : public UpdateModuleData
{
public:
	AsciiString m_spawnPointBoneNameData;

	static void buildFieldParse(MultiIniFieldParse& p)
	{
    UpdateModuleData::buildFieldParse(p);
		static const FieldParse dataFieldParse[] =
		{
			{ "SpawnPointBoneName",		INI::parseAsciiString,		nullptr, offsetof( SpawnPointProductionExitUpdateModuleData, m_spawnPointBoneNameData ) },
			{ nullptr, nullptr, nullptr, 0 }
		};
    p.add(dataFieldParse);
	}
};

//-------------------------------------------------------------------------------------------------
class SpawnPointProductionExitUpdate : public UpdateModule, public ExitInterface
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( SpawnPointProductionExitUpdate, "SpawnPointProductionExitUpdate" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( SpawnPointProductionExitUpdate, SpawnPointProductionExitUpdateModuleData )

public:

	virtual ExitInterface* getUpdateExitInterface() override { return this; }

	SpawnPointProductionExitUpdate( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	// Required funcs to fulfill interface requirements
	virtual Bool isExitBusy() const override {return FALSE;}	///< Contain style exiters are getting the ability to space out exits, so ask this before reserveDoor as a kind of no-commitment check.
	virtual ExitDoorType reserveDoorForExit( const ThingTemplate* objType, Object *specificObject ) override;
	virtual void exitObjectViaDoor( Object *newObj, ExitDoorType exitDoor ) override;
	virtual void unreserveDoorForExit( ExitDoorType exitDoor ) override;
	virtual void setRallyPoint( const Coord3D * ) override {}
	virtual const Coord3D *getRallyPoint() const override { return nullptr; }
	virtual void exitObjectByBudding( Object *newObj, Object *budHost ) override { return; }

	virtual UpdateSleepTime update() override										{ return UPDATE_SLEEP_FOREVER; }

protected:
	Bool m_bonesInitialized;													///< To prevent creation bugs, only init the World coords when first asked for one
	Int m_spawnPointCount;														///< How many in the array are actually live and valid
	Coord3D m_worldCoordSpawnPoints[MAX_SPAWN_POINTS];///< Where my little friends will be created
	Real m_worldAngleSpawnPoints[MAX_SPAWN_POINTS];		///< And what direction they should face
	ObjectID m_spawnPointOccupier[MAX_SPAWN_POINTS];	///< Who I think is in each spot.  I can validate their existence to see if I am free to exit something.

	// Required func to fulfill Module requirement

private:
	void initializeBonePositions();	///< Look up the bone positions and store them in world space coords
	void revalidateOccupiers();			///< Do a lookup on all our ID's and clear the dead ones.
};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/ProductionExitUpdate/SpawnPointProductionExitUpdate.cpp`
```cpp
// FILE: SpawnPointProductionExitUpdate.cpp /////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, April 2002
// Desc:		Hand off produced Units to me so I can Exit them into the world with my specific style
//					This instance puts guys at named bones.
///////////////////////////////////////////////////////////////////////////////////////////////////

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/Xfer.h"
#include "GameClient/Drawable.h"
#include "GameLogic/AI.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/TerrainLogic.h"
#include "GameLogic/Module/SpawnPointProductionExitUpdate.h"

#include "WWMath/matrix3d.h"		///< @todo Replace with our own matrix library

//-------------------------------------------------------------------------------------------------
SpawnPointProductionExitUpdate::SpawnPointProductionExitUpdate( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{
	m_bonesInitialized = FALSE;
	m_spawnPointCount = 0;
	for( Int positionIndex = 0; positionIndex < MAX_SPAWN_POINTS; positionIndex++ )
	{
		m_worldCoordSpawnPoints[positionIndex].zero();
		m_worldAngleSpawnPoints[positionIndex] = 0.0f;
		m_spawnPointOccupier[positionIndex] = INVALID_ID;
	}
	setWakeFrame(getObject(), UPDATE_SLEEP_FOREVER);
}

//-------------------------------------------------------------------------------------------------
SpawnPointProductionExitUpdate::~SpawnPointProductionExitUpdate()
{
}

//-------------------------------------------------------------------------------------------------
void SpawnPointProductionExitUpdate::exitObjectViaDoor( Object *newObj, ExitDoorType exitDoor )
{
	DEBUG_ASSERTCRASH(exitDoor == DOOR_1, ("multiple exit doors not supported here"));

	if( !m_bonesInitialized )
		initializeBonePositions();

	Object *creationObject = getObject();
	if (creationObject)
	{
		Int positionIndex = 0;
		for( ; positionIndex < m_spawnPointCount; positionIndex++ )
		{
			if( m_spawnPointOccupier[positionIndex] == INVALID_ID )
				break;
		}
		if( positionIndex == m_spawnPointCount )
		{
			DEBUG_CRASH( ("A SpawnPoint exit thought it had room but then failed") );
			return;
		}

		Coord3D createPoint;
		Real createAngle;

		// Get the location from our bone array
		createPoint = m_worldCoordSpawnPoints[positionIndex];

		// make sure the point is on the terrain
		createPoint.z = TheTerrainLogic ? TheTerrainLogic->getLayerHeight( createPoint.x, createPoint.y, creationObject->getLayer()) : 0.0f;

		// get the angle
		createAngle = m_worldAngleSpawnPoints[positionIndex];

		// record him as taking this spot
		m_spawnPointOccupier[positionIndex] = newObj->getID();

		// put him there
		newObj->setPosition( &createPoint );
		newObj->setOrientation( createAngle );
		newObj->setLayer(creationObject->getLayer());

		/** @todo This really should be automatically wrapped up in an activation sequence
		for objects in general */
		// tell the AI about it
		TheAI->pathfinder()->addObjectToPathfindMap( newObj );

		// You are stuck here, little man.
		newObj->setDisabled( DISABLED_HELD );
	}
}

//-------------------------------------------------------------------------------------------------
ExitDoorType SpawnPointProductionExitUpdate::reserveDoorForExit( const ThingTemplate* objType, Object *specificObject )
{
	if( !m_bonesInitialized )
		initializeBonePositions();

	if( !m_bonesInitialized )
		return DOOR_NONE_AVAILABLE; // Init failure

	revalidateOccupiers();

	for( Int positionIndex = 0; positionIndex < m_spawnPointCount; positionIndex++ )
	{
		if( m_spawnPointOccupier[positionIndex] == INVALID_ID )
			return DOOR_1;
	}

	return DOOR_NONE_AVAILABLE;
}

//-------------------------------------------------------------------------------------------------
void SpawnPointProductionExitUpdate::unreserveDoorForExit( ExitDoorType exitDoor )
{
	/* nothing */
}

//-------------------------------------------------------------------------------------------------
void SpawnPointProductionExitUpdate::initializeBonePositions()
{
	Object *me = getObject();
	Drawable *myDrawable = me->getDrawable();

	// This fundamental failure will result in this never ever thinking it is free
	if( myDrawable == nullptr )
		return;

	Matrix3D boneTransforms[MAX_SPAWN_POINTS];
	Int matrixIndex = 0;
	for( ; matrixIndex < MAX_SPAWN_POINTS; matrixIndex++ )
		boneTransforms[matrixIndex].Make_Identity();

	// Get all the bones of the right name
	const SpawnPointProductionExitUpdateModuleData* md = getSpawnPointProductionExitUpdateModuleData();
	m_spawnPointCount = myDrawable->getPristineBonePositions( md->m_spawnPointBoneNameData.str(), 1, nullptr, boneTransforms, MAX_SPAWN_POINTS );

	for( matrixIndex = 0; matrixIndex < m_spawnPointCount; matrixIndex++ )
	{
		Matrix3D *currentTransform = &(boneTransforms[matrixIndex]);
		// Convert their matrix one by one
		me->convertBonePosToWorldPos( nullptr, currentTransform, nullptr, currentTransform );

		// Then save the world coord and angle
		m_worldCoordSpawnPoints[matrixIndex].x = currentTransform->Get_X_Translation();
		m_worldCoordSpawnPoints[matrixIndex].y = currentTransform->Get_Y_Translation();
		m_worldCoordSpawnPoints[matrixIndex].z = 0; //set at creation time

		m_worldAngleSpawnPoints[matrixIndex] = currentTransform->Get_Z_Rotation();
	}

	m_bonesInitialized = TRUE;
}

//-------------------------------------------------------------------------------------------------
void SpawnPointProductionExitUpdate::revalidateOccupiers()
{
	for( Int positionIndex = 0; positionIndex < m_spawnPointCount; positionIndex++ )
	{
		if( m_spawnPointOccupier[positionIndex] == INVALID_ID )
			continue;

		if( TheGameLogic->findObjectByID( m_spawnPointOccupier[positionIndex] ) == nullptr )
			m_spawnPointOccupier[positionIndex] = INVALID_ID;
	}
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void SpawnPointProductionExitUpdate::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void SpawnPointProductionExitUpdate::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	//
	// we can ignore all the data with the bone position arrays and spawn point counts
	// because this module will load them lazily
	//
	//	Bool m_bonesInitialized;													///< To prevent creation bugs, only init the World coords when first asked for one
	//	Int m_spawnPointCount;														///< How many in the array are actually live and valid
	//	Coord3D m_worldCoordSpawnPoints[MAX_SPAWN_POINTS];///< Where my little friends will be created
	//	Real m_worldAngleSpawnPoints[MAX_SPAWN_POINTS];		///< And what direction they should face

	// spawn point occupants
	xfer->xferUser( &m_spawnPointOccupier, sizeof( ObjectID ) * MAX_SPAWN_POINTS );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void SpawnPointProductionExitUpdate::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
