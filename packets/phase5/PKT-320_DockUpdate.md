# Translation Packet `PKT-320`: `DockUpdate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-320`
- **Work Package:** `WP-5.4` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/update/dock_update.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (12):**
  - `Common/Debug.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Debug.h`
  - `Common/GameMemory.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameMemory.h`
  - `Common/INI.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/INI.h`
  - `Common/STLTypedefs.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/STLTypedefs.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/DockUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DockUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (2):**
  - `"NumberApproachPositions" => INI::parseInt`
  - `"AllowsPassthrough" => INI::parseBool`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (16 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferCoord3D( &m_enterPosition )`
  3. `xferCoord3D( &m_dockPosition )`
  4. `xferCoord3D( &m_exitPosition )`
  5. `xferInt( &m_numberApproachPositions )`
  6. `xferBool( &m_positionsLoaded )`
  7. `xferInt( &vectorSize )`
  8. `xferCoord3D( &m_approachPositions[vectorIndex] )`
  9. `xferInt( &vectorSize )`
  10. `xferObjectID( &m_approachPositionOwners[vectorIndex] )`
  11. `xferInt( &vectorSize )`
  12. `xferBool( &unpack )`
  13. `xferObjectID( &m_activeDocker )`
  14. `xferBool( &m_dockerInside )`
  15. `xferBool( &m_dockCrippled )`
  16. `xferBool( &m_dockOpen )`

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
- `zh_logic::modules::behavior::propaganda_tower_behavior`: `PropagandaTowerBehaviorModuleData`, `PropagandaTowerBehavior`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DockUpdate.h`
```cpp
// FILE: DockUpdate.h /////////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood Feb 2002
// Desc:   Behavior common to all DockUpdates is here.  Everything but action()
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/INI.h"
#include "Common/GameMemory.h"
#include "Common/STLTypedefs.h"
#include "GameLogic/Module/UpdateModule.h"

enum
{
	DEFAULT_APPROACH_VECTOR_SIZE = 10
};

enum
{
	DYNAMIC_APPROACH_VECTOR_FLAG = -1
};

//-------------------------------------------------------------------------------------------------
class DockUpdateModuleData : public UpdateModuleData
{
public:
	Int m_numberApproachPositionsData; // A positive number is an absolute, DYNAMIC_APPROACH_VECTOR_FLAG means dynamic vector
	Bool m_isAllowPassthrough;

	DockUpdateModuleData();
	static void buildFieldParse(MultiIniFieldParse& p);
};

//-------------------------------------------------------------------------------------------------
class DockUpdate : public UpdateModule , public DockUpdateInterface
{

	MEMORY_POOL_GLUE_ABC( DockUpdate )
	MAKE_STANDARD_MODULE_MACRO_ABC( DockUpdate )
	MAKE_STANDARD_MODULE_DATA_MACRO_ABC( DockUpdate, DockUpdateModuleData )

public:

	DockUpdate( Thing *thing, const ModuleData* moduleData );
	// virtual destructor provided by memory pool object

	/** Returns true if it is okay for the docker to approach and prepare to dock.
			False could mean the queue is full, for example.
	*/
	virtual Bool isClearToApproach( Object const* docker ) const override;

	/** Give me a Queue point to drive to, and record that that point is taken.
			Returning null means there are none free
	*/
	virtual Bool reserveApproachPosition( Object* docker, Coord3D *position, Int *index ) override;

	/** Give me the next Queue point to drive to, and record that that point is taken.
	*/
	virtual Bool advanceApproachPosition( Object* docker, Coord3D *position, Int *index ) override;

	/** Return true when it is OK for docker to begin entering the dock
			The Dock will lift the restriction on one particular docker on its own,
			so you must continually ask.
	*/
	virtual Bool isClearToEnter( Object const* docker ) const override;

	/** Return true when it is OK for docker to request a new Approach position.  The dock is in
			charge of keeping track of holes in the line, but the docker will remind us of their spot.
	*/
	virtual Bool isClearToAdvance( Object const* docker, Int dockerIndex ) const override;

	/** Give me the point that is the start of your docking path
			Returning null means there is none free
			All functions take docker as arg so we could have multiple docks on a building.
			Docker is not assumed, it is recorded and checked.
	*/
	virtual void getEnterPosition( Object* docker, Coord3D *position ) override;

	/** Give me the middle point of the dock process where the action() happens */
	virtual void getDockPosition( Object* docker, Coord3D *position ) override;

	/** Give me the point to drive to when I am done */
	virtual void getExitPosition( Object* docker, Coord3D *position ) override;

	virtual void onApproachReached( Object* docker ) override;		///< I have reached the Approach Point.
	virtual void onEnterReached( Object* docker ) override;			///< I have reached the Enter Point.
	virtual void onDockReached( Object* docker ) override;				///< I have reached the Dock point
	virtual void onExitReached( Object* docker ) override;				///< I have reached the exit.  You are no longer busy

	//The fact that action() is not here is intentional.  This object cannot exist.  You must
	//derive off it and implement action().

	virtual void cancelDock( Object* docker ) override;	///< Clear me from any reserved points, and if I was the reason you were Busy, you aren't anymore.

	virtual Bool isDockOpen() override { return m_dockOpen; }				///< Is the dock open to accepting dockers
	virtual void setDockOpen( Bool open ) override { m_dockOpen = open; }	///< Open/Close the dock

	virtual Bool isAllowPassthroughType() override;	///< Not all docks allow you to path through them in your AIDock machine

	virtual Bool isRallyPointAfterDockType() override {return FALSE;} ///< A minority of docks want to give you a final command to their rally point

	virtual void setDockCrippled( Bool setting ) override; ///< Game Logic can set me as inoperative.  I get to decide what that means.

	virtual UpdateSleepTime update() override;	///< In charge of lifting dock restriction for one registered as Approached if all is ready

protected:

	// These are const data read from the drawable when first asked for them
	Coord3D m_enterPosition;
	Coord3D m_dockPosition;
	Coord3D m_exitPosition;
	Int m_numberApproachPositions;
	Int m_numberApproachPositionBones; ///< Recomputed after loading; does not need serialization.

	// These are real variables local to my specific needs here in DockUpdate
	Bool m_positionsLoaded;	///< FALSE until we have loaded all the docking positions

	VecCoord3D m_approachPositions;
	ObjectIDVector m_approachPositionOwners;	///< Who is in or at least reserved each spot
	BoolVector m_approachPositionReached;			///< Which positions have actually been reached

	ObjectID m_activeDocker;	///< we could expand this to multiple dock paths since we always get docker in our methods
	Bool m_dockerInside; ///< This is true while our active docker is between Enter and Exit. This is shorter than activeDocker's lifetime as it doesn't include approach to enter
	Bool m_dockCrippled; ///< Has game logic set me as crippled?

	Bool m_dockOpen;  ///< Is the dock open for dockers

	void loadDockPositions();  ///< load all the dock positions
	Coord3D computeApproachPosition( Int positionIndex, Object *forWhom ); ///< Do a smart lookup of this bone position
};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/DockUpdate/DockUpdate.cpp`
```cpp
// FILE: DockUpdate.cpp /////////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood Feb 2002
// Desc:   Behavior common to all DockUpdates is here.  Everything but action()
///////////////////////////////////////////////////////////////////////////////////////////////////

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine
#include "Common/Debug.h"
#include "Common/Xfer.h"
#include "GameClient/Drawable.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Module/DockUpdate.h"

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
DockUpdateModuleData::DockUpdateModuleData()
{
	m_numberApproachPositionsData = 0;
	m_isAllowPassthrough = TRUE;
}

/*static*/ void DockUpdateModuleData::buildFieldParse(MultiIniFieldParse& p)
{

	UpdateModuleData::buildFieldParse( p );

	static const FieldParse dataFieldParse[] =
	{
		{ "NumberApproachPositions"	,INI::parseInt,		nullptr, offsetof( DockUpdateModuleData, m_numberApproachPositionsData ) },
		{ "AllowsPassthrough"				,INI::parseBool,	nullptr, offsetof( DockUpdateModuleData, m_isAllowPassthrough ) },
		{ nullptr, nullptr, nullptr, 0 }

	};

  p.add(dataFieldParse);

}

DockUpdate::DockUpdate( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{

	m_dockOpen = TRUE;

	m_positionsLoaded = FALSE;
	m_numberApproachPositionBones = -1;
	m_activeDocker = INVALID_ID;
	m_dockerInside = FALSE;
	m_dockCrippled = FALSE;

	const DockUpdateModuleData *md = (const DockUpdateModuleData *)moduleData;

	m_exitPosition.zero();
	m_dockPosition.zero();
	m_enterPosition.zero();

	m_numberApproachPositions = md->m_numberApproachPositionsData;
	if( m_numberApproachPositions != DYNAMIC_APPROACH_VECTOR_FLAG )
	{
		// Not dynamic, so make this the size
		m_approachPositions.resize(m_numberApproachPositions);
		m_approachPositionOwners.resize(m_numberApproachPositions);
		m_approachPositionReached.resize(m_numberApproachPositions);
	}
	else
	{
		//Otherwise, make a default size, and plan on growing it later
		m_approachPositions.resize(DEFAULT_APPROACH_VECTOR_SIZE);
		m_approachPositionOwners.resize(DEFAULT_APPROACH_VECTOR_SIZE);
		m_approachPositionReached.resize(DEFAULT_APPROACH_VECTOR_SIZE);
	}

	for( size_t vectorIndex = 0; vectorIndex < m_approachPositions.size(); ++vectorIndex )
	{
		// Whatever size we are, init everything.
		m_approachPositions[vectorIndex].zero();
		m_approachPositionOwners[vectorIndex] = INVALID_ID;
		m_approachPositionReached[vectorIndex] = FALSE;
	}
}

DockUpdate::~DockUpdate()
{
}

Bool DockUpdate::isClearToApproach( Object const* docker ) const
{
	// If we allow infinite approaches, we don't even need to look up.  Just say yes.
	// The reserve code will handle appending a free spot to the end.
	if( m_numberApproachPositions == DYNAMIC_APPROACH_VECTOR_FLAG )
		return TRUE;

	ObjectID dockerID = docker->getID();

	for( size_t positionIndex = 0; positionIndex < m_approachPositionOwners.size(); ++positionIndex )
	{
		if( m_approachPositionOwners[positionIndex] == INVALID_ID )
		{
			return TRUE;
		}
		if( m_approachPositionOwners[positionIndex] == dockerID )
		{
			return TRUE;
		}
	}

	return FALSE;
}

Bool DockUpdate::reserveApproachPosition( Object* docker, Coord3D *position, Int *index )
{

	// load dock positions if not loaded yet
	if( m_positionsLoaded == FALSE )
		loadDockPositions();

	// sanity
	if( position == nullptr )
		return FALSE;

	ObjectID dockerID = docker->getID();

	Int positionIndex = 0;
	for( ; positionIndex < m_approachPositionOwners.size(); ++positionIndex )
	{
		if( m_approachPositionOwners[positionIndex] == dockerID )
		{
			*position = computeApproachPosition( positionIndex, docker );
			*index = positionIndex;
			return TRUE;
		}
		if( m_approachPositionOwners[positionIndex] == INVALID_ID )
		{
			m_approachPositionOwners[positionIndex] = dockerID;
			*position = computeApproachPosition( positionIndex, docker );
			*index = positionIndex;
			return TRUE;
		}
	}

	// If I make it out of the loop, I am full, so dynamic approach buildings should make a new entry instead of saying no
	if( m_numberApproachPositions == DYNAMIC_APPROACH_VECTOR_FLAG )
	{
		Coord3D zero;
		zero.zero();
		m_approachPositions.push_back( zero );
		m_approachPositionOwners.push_back( INVALID_ID );
		m_approachPositionReached.push_back( FALSE );

		loadDockPositions();// refresh this new one

		positionIndex = m_approachPositionOwners.size() - 1;// The new last spot
		m_approachPositionOwners[positionIndex] = dockerID;
		*position = computeApproachPosition( positionIndex, docker );
		*index = positionIndex;
		return TRUE;
	}

	return FALSE;
}

Bool DockUpdate::advanceApproachPosition( Object* docker, Coord3D *position, Int *index )
{
	// load dock positions if not loaded yet
	if( m_positionsLoaded == FALSE )
		loadDockPositions();

	// sanity
	if( position == nullptr )
		return FALSE;
	if( *index <= 0 )
		return FALSE;
	if( m_approachPositionOwners[(*index) - 1] != INVALID_ID )
		return FALSE;

	Int hisIndex = *index;
	Int previousIndex = hisIndex - 1;

	m_approachPositionOwners[previousIndex] = docker->getID();
	m_approachPositionReached[previousIndex] = FALSE;

	m_approachPositionOwners[hisIndex] = INVALID_ID;
	m_approachPositionReached[hisIndex] = FALSE;

	*position = computeApproachPosition( previousIndex, docker );
	*index = previousIndex;
	return TRUE;
}

Bool DockUpdate::isClearToEnter( Object const* docker ) const
{
	ObjectID dockerID = docker->getID();
	return dockerID == m_activeDocker;
}

Bool DockUpdate::isClearToAdvance( Object const* docker, Int dockerIndex ) const
{
	if( dockerIndex < 0 )
		return FALSE;

	ObjectID dockerID = docker->getID();
	Bool correctRequest = dockerID == m_approachPositionOwners[dockerIndex];
	Bool approachReached = m_approachPositionReached[dockerIndex];
	Bool nextSpotFree = (dockerIndex > 0)  &&  (m_approachPositionOwners[dockerIndex - 1] == INVALID_ID);

	return correctRequest && approachReached && nextSpotFree;
}

void DockUpdate::getEnterPosition( Object* docker, Coord3D *position )
{

	// load dock positions if not loaded yet
	if( m_positionsLoaded == FALSE )
		loadDockPositions();

	// sanity
	if( position == nullptr )
		return;

	// If I don't have a bone, you are fine where you are, unless you fly, in which case I should recenter you
	Coord3D zero;
	zero.zero();
	if( m_enterPosition == zero )
	{
		if( docker->isUsingAirborneLocomotor() )
		{
			*position = *getObject()->getPosition();
			return;
		}
		*position = *docker->getPosition();
		return;
	}

	// take local space position and convert to world space
	getObject()->convertBonePosToWorldPos( &m_enterPosition, nullptr, position, nullptr );

}

void DockUpdate::getDockPosition( Object* docker, Coord3D *position )
{

	// load dock positions if not loaded yet
	if( m_positionsLoaded == FALSE )
		loadDockPositions();

	// sanity
	if( position == nullptr )
		return;

	// If I don't have a bone, you are fine where you are.
	Coord3D zero;
	zero.zero();
	if( m_enterPosition == zero )
	{
		*position = *docker->getPosition();
		return;
	}

	// take local space position and convert to world space
	getObject()->convertBonePosToWorldPos( &m_dockPosition, nullptr, position, nullptr );

}

void DockUpdate::getExitPosition( Object* docker, Coord3D *position )
{

	// load dock positions if not loaded yet
	if( m_positionsLoaded == FALSE )
		loadDockPositions();

	// sanity
	if( position == nullptr )
		return;

	// If I don't have a bone, you are fine where you are.
	Coord3D zero;
	zero.zero();
	if( m_enterPosition == zero )
	{
		*position = *docker->getPosition();
		return;
	}

	// take local space position and convert to world space
	getObject()->convertBonePosToWorldPos( &m_exitPosition, nullptr, position, nullptr );

}

void DockUpdate::onApproachReached( Object* docker )
{
	ObjectID dockerID = docker->getID();
	for( size_t positionIndex = 0; positionIndex < m_approachPositionOwners.size(); ++positionIndex )
	{
		if( m_approachPositionOwners[positionIndex] == dockerID )
		{
			m_approachPositionReached[positionIndex] = TRUE;
			return;
		}
	}
}

void DockUpdate::onEnterReached( Object* docker )
{
	Object *me = getObject();
	me->clearAndSetModelConditionFlags( MAKE_MODELCONDITION_MASK(MODELCONDITION_DOCKING_ENDING),
																			MAKE_MODELCONDITION_MASK2(MODELCONDITION_DOCKING_BEGINNING, MODELCONDITION_DOCKING) );
	docker->clearAndSetModelConditionFlags( MAKE_MODELCONDITION_MASK(MODELCONDITION_DOCKING_ENDING),
																					MAKE_MODELCONDITION_MASK2(MODELCONDITION_DOCKING_BEGINNING, MODELCONDITION_DOCKING) );
	m_dockerInside = TRUE;

	ObjectID dockerID = docker->getID();
	for( size_t positionIndex = 0; positionIndex < m_approachPositionOwners.size(); ++positionIndex )
	{
		if( m_approachPositionOwners[positionIndex] == dockerID )
		{
			m_approachPositionOwners[positionIndex] = INVALID_ID;
			m_approachPositionReached[positionIndex] = FALSE;
			return;
		}
	}
}

void DockUpdate::onDockReached( Object* docker )
{
	Object *me = getObject();
	me->clearAndSetModelConditionState( MODELCONDITION_DOCKING_BEGINNING, MODELCONDITION_DOCKING_ACTIVE );
	docker->clearAndSetModelConditionState( MODELCONDITION_DOCKING_BEGINNING, MODELCONDITION_DOCKING_ACTIVE );
}

void DockUpdate::onExitReached( Object* docker )
{
	Object *me = getObject();
	me->clearAndSetModelConditionFlags( MAKE_MODELCONDITION_MASK2(MODELCONDITION_DOCKING_ACTIVE, MODELCONDITION_DOCKING),
																			MAKE_MODELCONDITION_MASK(MODELCONDITION_DOCKING_ENDING) );
	docker->clearAndSetModelConditionFlags( MAKE_MODELCONDITION_MASK2(MODELCONDITION_DOCKING_ACTIVE, MODELCONDITION_DOCKING),
																					MAKE_MODELCONDITION_MASK(MODELCONDITION_DOCKING_ENDING) );
	m_dockerInside = FALSE;

	ObjectID dockerID = docker->getID();
	if( dockerID == m_activeDocker )
		m_activeDocker = INVALID_ID;
	else
	{

		//
		// we only assert here if the dock is open, for closed docks it's OK to allow somebody
		// to continue moving to the exit position cause they are leaving after all
		//
		if( isDockOpen() )
			DEBUG_CRASH( ("Fiddle.  Someone said goodbye to a dock when the dock didn't think it was talking to that someone."));

	}
}

void DockUpdate::cancelDock( Object* docker )
{
	ObjectID dockerID = docker->getID();
	for( size_t positionIndex = 0; positionIndex < m_approachPositionOwners.size(); ++positionIndex )
	{
		if( m_approachPositionOwners[positionIndex] == dockerID )
		{
			m_approachPositionOwners[positionIndex] = INVALID_ID;
			m_approachPositionReached[positionIndex] = FALSE;
		}
	}
	if( m_activeDocker == dockerID )
	{
		Object *dockingObject = TheGameLogic->findObjectByID(m_activeDocker);
		m_activeDocker = INVALID_ID;
		m_dockerInside = FALSE;
		// clear any model conditions related to docking that may be set on us and them.
		// (Normal clear is part of each stage, but we won't get there.)
		ModelConditionFlags clear;
		clear.set( MODELCONDITION_DOCKING_ENDING );
		clear.set( MODELCONDITION_DOCKING_BEGINNING );
		clear.set( MODELCONDITION_DOCKING_ACTIVE );
		clear.set( MODELCONDITION_DOCKING );
		getObject()->clearModelConditionFlags( clear );
		if( dockingObject )
			dockingObject->clearModelConditionFlags( clear );
	}
}

void DockUpdate::setDockCrippled( Bool setting )
{
	// At this level, Crippling means I will accept Approach requests, but I will never grant Enter clearance.
	m_dockCrippled = setting;
}

UpdateSleepTime DockUpdate::update()
{
	if( m_activeDocker == INVALID_ID  &&  !m_dockCrippled )
	{
		// if setDockCrippled has been called, I will never give entrance permission.
		for( size_t positionIndex = 0; positionIndex < m_approachPositionReached.size(); ++positionIndex )
		{
			if( m_approachPositionReached[positionIndex] )
			{
				m_activeDocker = m_approachPositionOwners[positionIndex];
				return UPDATE_SLEEP_NONE;
			}
		}
	}
	else if ( getObject()->isKindOf( KINDOF_SUPPLY_SOURCE ) )
	{
		Object *docker = TheGameLogic->findObjectByID( m_activeDocker );
		if ( docker && docker->isKindOf( KINDOF_DOZER ) && docker->isKindOf( KINDOF_HARVESTER ))// a worker
		{
			ModelConditionFlags test;
			test.set( MODELCONDITION_DOCKING_BEGINNING );
			Drawable *dockerDraw = docker->getDrawable();
			if ( dockerDraw && dockerDraw->getModelConditionFlags().anyIntersectionWith( test ) )
				dockerDraw->clearModelConditionFlags( MAKE_MODELCONDITION_MASK(MODELCONDITION_MOVING) );
		}
	}


	return UPDATE_SLEEP_NONE;
}

Coord3D DockUpdate::computeApproachPosition( Int positionIndex, Object *forWhom )
{
	// load dock positions if not loaded yet
	if( m_positionsLoaded == FALSE )
		loadDockPositions();

	Coord3D bestPosition;// This answer is the best, as it includes findPositionAround
	Coord3D workingPosition;// But if findPositionAround fails, we need to say something.

	FindPositionOptions fpOptions;
	// Start with the pristine bone, then convert it to the world, then find a clean spot around it.

	Object *us = getObject();
	us->convertBonePosToWorldPos( &m_approachPositions[positionIndex], nullptr, &workingPosition, nullptr );

	if( m_numberApproachPositionBones == 0 )
	{
		Coord3D ourPosition = *us->getPosition();
		Coord3D theirPosition = *forWhom->getPosition();
		// A Boneless building wants to bias towards the caller for the arbitrary position
		Vector3 offset( theirPosition.x - ourPosition.x, theirPosition.y - ourPosition.y, theirPosition.z - ourPosition.z );
		offset.Normalize();
		offset = offset * (us->getGeometryInfo().getMajorRadius() / 2);

		workingPosition.x += offset.X;
		workingPosition.y += offset.Y;
		workingPosition.z += offset.Z;
	}

	fpOptions.minRadius = 0.0f;
	fpOptions.maxRadius = 100.0f;
	fpOptions.sourceToPathToDest = forWhom;// This makes it find a place forWhom can get to.
	if( forWhom->isUsingAirborneLocomotor() )
		fpOptions.ignoreObject = getObject();// Flyers can ignore us, so they can approach right over us if they want.

	Bool spotFound = ThePartitionManager->findPositionAround( &workingPosition, &fpOptions, &bestPosition );

	if( spotFound)
		return bestPosition;

	return workingPosition;
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void DockUpdate::loadDockPositions()
{
	Object *obj = getObject();
	Drawable *myDrawable = obj->getDrawable();

	if (myDrawable)
	{
		//Patch 1.03 - Kris - Jan 19, 2005
		//Some of the GLASupplyStash assets still have docking bones in them. When found, the docking positions must be
		//observed. This occurs when upgrading to fortified structures which still have the bones. This has the negative
		//side-effect of workers suddenly slowing down their gathering rate. The proper fix would be to remove the bones
		//from the assets, but the artists could not find the original max files, hence the code solution.
		if( !obj->isKindOf( KINDOF_IGNORE_DOCKING_BONES ) )
		{

			myDrawable->getPristineBonePositions( "DockStart", 0, &m_enterPosition, nullptr, 1);
			myDrawable->getPristineBonePositions( "DockAction", 0, &m_dockPosition, nullptr, 1);
			myDrawable->getPristineBonePositions( "DockEnd", 0, &m_exitPosition, nullptr, 1);
			if( m_numberApproachPositions != DYNAMIC_APPROACH_VECTOR_FLAG )
			{
				// Dynamic means no bones

				// TheSuperHackers @logic-client-separation helmutbuhler 11/04/2025
				// We shouldn't depend on bones of a drawable here!

				// TheSuperHackers @fix helmutbuhler 19/04/2025 Zero initialize array to prevent uninitialized memory reads.
				// Important: the entire target vector is used for serialization and crc and must not contain random data.
				Coord3D approachBones[DEFAULT_APPROACH_VECTOR_SIZE] = {0};
				m_numberApproachPositionBones = myDrawable->getPristineBonePositions( "DockWaiting", 1, approachBones, nullptr, m_numberApproachPositions);
				if( m_numberApproachPositions == m_approachPositions.size() )//safeguard: will always be true
				{
					for( Int copyIndex = 0; copyIndex < m_numberApproachPositions; ++copyIndex )
					{
						m_approachPositions[copyIndex] = approachBones[copyIndex];
					}
				}
			}
			else
				m_numberApproachPositionBones = 0;

			m_positionsLoaded = TRUE;
		}
		else
		{
			m_numberApproachPositionBones = 0;
			m_positionsLoaded = TRUE;
		}
	}
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
Bool DockUpdate::isAllowPassthroughType()
{
	return getDockUpdateModuleData()->m_isAllowPassthrough;
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void DockUpdate::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer Method
	* Version Info:
	* 1: Initial version
	*/
// ------------------------------------------------------------------------------------------------
void DockUpdate::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// call base class
	UpdateModule::xfer( xfer );

	// enter position
	xfer->xferCoord3D( &m_enterPosition );

	// dock position
	xfer->xferCoord3D( &m_dockPosition );

	// exit position
	xfer->xferCoord3D( &m_exitPosition );

	// # approach positions
	xfer->xferInt( &m_numberApproachPositions );

	// positions loaded
	xfer->xferBool( &m_positionsLoaded );

	// approach positions
	Int vectorSize = m_approachPositions.size();
	xfer->xferInt( &vectorSize );
	m_approachPositions.resize(vectorSize);
	Int vectorIndex = 0;
	for( ; vectorIndex < vectorSize; ++vectorIndex )
	{
		// Okay, this is cool.  On save, the size and a bunch of coords will be written.
		// on load, vectorSize will be at 0 from the .size, but will then get set
		// by the xfer, and properly control the number of Coords.
		xfer->xferCoord3D( &m_approachPositions[vectorIndex] );
	}

	// approach position owners
	vectorSize = m_approachPositionOwners.size();
	xfer->xferInt( &vectorSize );
	m_approachPositionOwners.resize(vectorSize);
	for( vectorIndex = 0; vectorIndex < vectorSize; ++vectorIndex )
	{
		xfer->xferObjectID( &m_approachPositionOwners[vectorIndex] );
	}

	// approach positions reached
	vectorSize = m_approachPositionReached.size();
	xfer->xferInt( &vectorSize );
	m_approachPositionReached.resize(vectorSize);
	for( vectorIndex = 0; vectorIndex < vectorSize; ++vectorIndex )
	{
		// Vector of Bool gets packed as bitfield internally
		Bool unpack = m_approachPositionReached[vectorIndex];
		xfer->xferBool( &unpack );
		m_approachPositionReached[vectorIndex] = unpack;
	}

	// active docker
	xfer->xferObjectID( &m_activeDocker );

	// docker inside
	xfer->xferBool( &m_dockerInside );

	// docker crippled
	xfer->xferBool( &m_dockCrippled );

	// dock open
	xfer->xferBool( &m_dockOpen );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void DockUpdate::loadPostProcess()
{

	// call base class
	UpdateModule::loadPostProcess();

	// TheSuperHackers @bugfix bobtista 30/09/2026 Restore the unsaved approach bone count.
	if( m_positionsLoaded )
	{
		loadDockPositions();
	}

}
```
