# Translation Packet `PKT-355`: `ProjectileStreamUpdate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-355`
- **Work Package:** `WP-5.4` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/update/projectile_stream_update.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (7):**
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/ProjectileStreamUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ProjectileStreamUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `PreRTS.h`
  - `WWMath/vector3.h`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (7 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUser( m_projectileIDs, sizeof( ObjectID ) * MAX_PROJECTILE_STREAM )`
  3. `xferInt( &m_nextFreeIndex )`
  4. `xferInt( &m_firstValidIndex )`
  5. `xferObjectID( &m_owningObject )`
  6. `xferObjectID( &m_targetObject )`
  7. `xferCoord3D( &m_targetPosition )`

## 2.5 Existing Rust Symbol Index (Use These Exact Paths)
- `zh_logic::modules::update::base_renerate_update`: `BaseRegenerateUpdateModuleData`, `BaseRegenerateUpdate`
- `zh_logic::modules::update::deletion_update`: `DeletionUpdateModuleData`, `DeletionUpdate`
- `zh_logic::modules::update::fire_weapon_update`: `FireWeaponUpdateModuleData`, `FireWeaponUpdate`
- `zh_logic::modules::update::float_update`: `FloatUpdateModuleData`, `FloatUpdate`
- `zh_logic::modules::update::lifetime_update`: `LifetimeUpdateModuleData`, `LifetimeUpdate`
- `zh_logic::modules::update::prone_update`: `ProneUpdateModuleData`, `ProneUpdate`
- `zh_logic::modules::update::smart_bomb_target_homing_update`: `SmartBombTargetHomingUpdateModuleData`, `SmartBombTargetHomingUpdate`
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
- `zh_logic::modules::behavior::slow_death_behavior`: `SLOW_DEATH_PHASE_NAMES`, `SlowDeathPhaseType`, `COUNT`, `SD_FLAG_SLOW_DEATH_ACTIVATED`, `SD_FLAG_MIDPOINT_EXECUTED`, `SD_FLAG_FLUNG_INTO_AIR`, `SD_FLAG_BOUNCED`, `SD_EFFECT_HAS_FX`, `SD_EFFECT_HAS_OCL`, `SD_EFFECT_HAS_WEAPON`, `SlowDeathBehaviorModuleData`, `fn`, `SlowDeathCore`, `SlowDeathBehavior`
- `zh_logic::modules::behavior::supply_warehouse_crippling_behavior`: `SupplyWarehouseCripplingBehaviorModuleData`, `SupplyWarehouseCripplingBehavior`
- `zh_logic::modules::behavior::tech_building_behavior`: `TechBuildingBehaviorModuleData`, `TechBuildingBehavior`
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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ProjectileStreamUpdate.h`
```cpp
// FILE: ProjectileStreamUpdate.h //////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, May 2002
// Desc:   Tracks all projectiles fired so they can be drawn as a stream
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/UpdateModule.h"
// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
class Thing;
class Vector3;

enum
{
	MAX_PROJECTILE_STREAM = 20
};

//-------------------------------------------------------------------------------------------------
/** The default	update module */
//-------------------------------------------------------------------------------------------------
class ProjectileStreamUpdate : public UpdateModule
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( ProjectileStreamUpdate, "ProjectileStreamUpdate" )
	MAKE_STANDARD_MODULE_MACRO( ProjectileStreamUpdate );

public:

	ProjectileStreamUpdate( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	void addProjectile( ObjectID sourceID, ObjectID newID, ObjectID victimID, const Coord3D *victimPos );	///< This projectile was just shot, so keep track of it.
	void getAllPoints( Vector3 *points, Int *count );					///< unroll circular array and write down all projectile positions
	void setPosition( const Coord3D *newPosition );						///< I need to exist at the place I want to draw since only (near) on screen Drawables get updated

	virtual UpdateSleepTime update() override;

protected:


	void cullFrontOfList();
	Bool considerDying();

	ObjectID m_projectileIDs[MAX_PROJECTILE_STREAM];
	Int m_nextFreeIndex;
	Int m_firstValidIndex;
	ObjectID m_owningObject;

	ObjectID m_targetObject;///< Need to insert a hole if target changes, so track target ID and target position
	Coord3D m_targetPosition;
};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/ProjectileStreamUpdate.cpp`
```cpp
// FILE: ProjectileStreamUpdate.cpp //////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, May 2002
// Desc:   Tracks all projectiles fired so they can be drawn as a stream
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/Xfer.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/Module/ProjectileStreamUpdate.h"
#include "WWMath/vector3.h"


//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ProjectileStreamUpdate::ProjectileStreamUpdate( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{
	std::fill(m_projectileIDs, m_projectileIDs + ARRAY_SIZE(m_projectileIDs), INVALID_ID);
	m_nextFreeIndex = 0;
	m_firstValidIndex = 0;
	m_owningObject = INVALID_ID;

	m_targetObject = INVALID_ID;
	m_targetPosition.zero();
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ProjectileStreamUpdate::~ProjectileStreamUpdate()
{

}

//-------------------------------------------------------------------------------------------------
/** The update callback. */
//-------------------------------------------------------------------------------------------------
UpdateSleepTime ProjectileStreamUpdate::update()
{
	cullFrontOfList();

	// Update the draw module about our points (arrange array so they can read?)

	if( considerDying() )
		TheGameLogic->destroyObject( getObject() );

	return UPDATE_SLEEP_NONE;
}

void ProjectileStreamUpdate::addProjectile( ObjectID sourceID, ObjectID newID, ObjectID victimID, const Coord3D *victimPos )
{
	DEBUG_ASSERTCRASH( m_owningObject == INVALID_ID  ||  m_owningObject == sourceID, ("Two objects are trying to use the same Projectile stream.") );//Don't cross the streams!
	if( m_owningObject == INVALID_ID )
		m_owningObject = sourceID;

	// Keep track of target, and insert a hole if the target has changed
	if( victimID != INVALID_ID )
	{
		// Object shot
		if( victimID != m_targetObject )
		{
			// Changed targets, insert a hole to break the stream
			m_projectileIDs[ m_nextFreeIndex ] = INVALID_ID;
			m_nextFreeIndex = (m_nextFreeIndex + 1) % MAX_PROJECTILE_STREAM;

			// And mark this as our new target
			m_targetObject = victimID;
		}

		// Clear position so we know we are an object shot
		m_targetPosition.zero();
	}
	else if( victimPos != nullptr )
	{
		if( ! (m_targetPosition == (*victimPos)) )
		{
			// New position, so insert hole
			m_projectileIDs[ m_nextFreeIndex ] = INVALID_ID;
			m_nextFreeIndex = (m_nextFreeIndex + 1) % MAX_PROJECTILE_STREAM;

			// And mark this as our new target
			m_targetPosition = (*victimPos);
		}

		// Clear object so we know we are a position shot
		m_targetObject = INVALID_ID;
	}
	else
	{
		DEBUG_CRASH(("A projectile stream was fired at neither an object nor a position.  Probably bad."));
	}

	// Keep track of the id in a circular array
	m_projectileIDs[ m_nextFreeIndex ] = newID;
	m_nextFreeIndex = (m_nextFreeIndex + 1) % MAX_PROJECTILE_STREAM;
	DEBUG_ASSERTCRASH( m_nextFreeIndex != m_firstValidIndex, ("Need to increase the allowed number of simultaneous particles in ProjectileStreamUpdate.") );
}

void ProjectileStreamUpdate::cullFrontOfList()
{
	while( (m_firstValidIndex != m_nextFreeIndex)  &&  (TheGameLogic->findObjectByID( m_projectileIDs[m_firstValidIndex] ) == nullptr) )
	{
		// Chew off the front if they are gone.  Don't chew on the middle, as bad ones there are just a break in the chain
		m_firstValidIndex = (m_firstValidIndex + 1) % MAX_PROJECTILE_STREAM;
	}
}

Bool ProjectileStreamUpdate::considerDying()
{
	if( m_firstValidIndex == m_nextFreeIndex  &&  m_owningObject != INVALID_ID )
	{
		//If I have no projectiles to watch, and my master is dead, then yes, I want to die
		if( TheGameLogic->findObjectByID(m_owningObject) == nullptr )
			return TRUE;
	}

	return FALSE;
}

void ProjectileStreamUpdate::getAllPoints( Vector3 *points, Int *count )
{
	Int pointCount = 0;
	Int pointIndex = m_firstValidIndex;


	Object *obj = TheGameLogic->findObjectByID(m_owningObject);


	while( pointIndex != m_nextFreeIndex )
	{
		// Go through the array I think of as good.  Holes in the middle get 0,0,0.  I write
		// to pointCount because I am unrolling a circular array into the same sized flat one
		// since I am writing anyway.
		Object *projectile = TheGameLogic->findObjectByID( m_projectileIDs[pointIndex] );

		if( projectile )
		{
			Coord3D thisPoint = *projectile->getPosition();
			points[pointCount].X = thisPoint.x;
			points[pointCount].Y = thisPoint.y;
			points[pointCount].Z = thisPoint.z;


			if ( obj && obj->isKindOf( KINDOF_VEHICLE ) )				// this makes the stream skim along my roof, if I have a roof
			{
				const Coord3D *pos = obj->getPosition();
				Real myTop = obj->getGeometryInfo().getMaxHeightAbovePosition() + pos->z + 0.5f;
				Coord3D delta;
				delta.x = pos->x - points[pointCount].X;
				delta.y = pos->y - points[pointCount].Y;
				delta.z = 0.0f;
				if( delta.length() <= obj->getGeometryInfo().getMajorRadius() * 1.5f )
					points[pointCount].Z = MAX( points[pointCount].Z, myTop );
			}




		}
		else
		{
			points[pointCount].X = 0;
			points[pointCount].Y = 0;
			points[pointCount].Z = 0;
		}

		pointIndex = (pointIndex + 1) % MAX_PROJECTILE_STREAM;
		pointCount++;
	}

	*count = pointCount;
}

void ProjectileStreamUpdate::setPosition( const Coord3D *newPosition )
{
	Object *me = getObject();
	me->setPosition( newPosition );
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void ProjectileStreamUpdate::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version
	* 2: Target tracking for line breaking
*/
// ------------------------------------------------------------------------------------------------
void ProjectileStreamUpdate::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 2;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// projectile ids
	xfer->xferUser( m_projectileIDs, sizeof( ObjectID ) * MAX_PROJECTILE_STREAM );

	// next free index
	xfer->xferInt( &m_nextFreeIndex );

	// first valid index
	xfer->xferInt( &m_firstValidIndex );

	// owning object
	xfer->xferObjectID( &m_owningObject );

	if( version >= 2 )
	{
		xfer->xferObjectID( &m_targetObject );
		xfer->xferCoord3D( &m_targetPosition );
	}

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void ProjectileStreamUpdate::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
