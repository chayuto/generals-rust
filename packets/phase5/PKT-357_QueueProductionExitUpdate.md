# Translation Packet `PKT-357`: `QueueProductionExitUpdate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-357`
- **Work Package:** `WP-5.4` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/update/queue_production_exit_update.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (13):**
  - `Common/INI.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/INI.h`
  - `Common/RandomValue.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/RandomValue.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameLogic/AI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/AI.h`
  - `GameLogic/AIPathfind.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/AIPathfind.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/PhysicsUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PhysicsUpdate.h`
  - `GameLogic/Module/QueueProductionExitUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/QueueProductionExitUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `Lib/BaseType.h`
  - `PreRTS.h`
- **Implemented Module Facets (1):**
  - `ExitInterface* getUpdateExitInterface()`
- **INI `FieldParse` Table Entries (5):**
  - `"UnitCreatePoint" => INI::parseCoord3D`
  - `"NaturalRallyPoint" => INI::parseCoord3D`
  - `"ExitDelay" => INI::parseDurationUnsignedInt`
  - `"AllowAirborneCreation" => INI::parseBool`
  - `"InitialBurst" => INI::parseUnsignedInt`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (6 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUnsignedInt( &m_currentDelay )`
  3. `xferCoord3D( &m_rallyPoint )`
  4. `xferBool( &m_rallyPointExists )`
  5. `xferReal( &m_creationClearDistance )`
  6. `xferUnsignedInt( &m_currentBurstCount )`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/QueueProductionExitUpdate.h`
```cpp
// FILE: QueueProductionExitUpdate.h /////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, January, 2002
// Desc:		Hand off produced Units to me so I can Exit them into the world with my specific style
//					This instance refuses to spit a second out until the first is clear
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

#include "GameLogic/Module/UpdateModule.h"
#include "Common/INI.h"
#include "Lib/BaseType.h"

class Object;

//-------------------------------------------------------------------------------------------------
class QueueProductionExitUpdateModuleData : public UpdateModuleData
{
public:
	Coord3D m_unitCreatePoint;
	Coord3D m_naturalRallyPoint;
	UnsignedInt m_exitDelayData;
	Bool m_allowAirborneCreationData;
	UnsignedInt m_initialBurst;

	QueueProductionExitUpdateModuleData()
	{
		m_unitCreatePoint.zero();
		m_naturalRallyPoint.zero();
		m_exitDelayData = 0;
		m_allowAirborneCreationData = FALSE;
		m_initialBurst = 0;
	}

	static void buildFieldParse(MultiIniFieldParse& p)
	{
    UpdateModuleData::buildFieldParse(p);
		static const FieldParse dataFieldParse[] =
		{
			{ "UnitCreatePoint",				INI::parseCoord3D,							nullptr, offsetof( QueueProductionExitUpdateModuleData, m_unitCreatePoint ) },
			{ "NaturalRallyPoint",			INI::parseCoord3D,							nullptr, offsetof( QueueProductionExitUpdateModuleData, m_naturalRallyPoint ) },
			{ "ExitDelay",							INI::parseDurationUnsignedInt,	nullptr, offsetof( QueueProductionExitUpdateModuleData, m_exitDelayData ) },
			{ "AllowAirborneCreation",	INI::parseBool,									nullptr, offsetof( QueueProductionExitUpdateModuleData, m_allowAirborneCreationData ) },
			{ "InitialBurst",						INI::parseUnsignedInt,					nullptr, offsetof( QueueProductionExitUpdateModuleData, m_initialBurst ) },
			{ nullptr, nullptr, nullptr, 0 }
		};
    p.add(dataFieldParse);
	}
};

//-------------------------------------------------------------------------------------------------
class QueueProductionExitUpdate : public UpdateModule, public ExitInterface
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( QueueProductionExitUpdate, "QueueProductionExitUpdate" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( QueueProductionExitUpdate, QueueProductionExitUpdateModuleData )

public:

	virtual ExitInterface* getUpdateExitInterface() override { return this; }

	QueueProductionExitUpdate( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	// Required funcs to fulfill interface requirements
	virtual Bool isExitBusy() const override {return FALSE;}	///< Contain style exiters are getting the ability to space out exits, so ask this before reserveDoor as a kind of no-commitment check.
	virtual ExitDoorType reserveDoorForExit( const ThingTemplate* objType, Object *specificObject ) override;
	virtual void exitObjectViaDoor( Object *newObj, ExitDoorType exitDoor ) override;
	virtual void exitObjectByBudding( Object *newObj, Object *budHost ) override;
	virtual void unreserveDoorForExit( ExitDoorType exitDoor ) override;

	virtual void setRallyPoint( const Coord3D *pos ) override;			///< define a "rally point" for units to move towards
	virtual const Coord3D *getRallyPoint() const override;			///< define a "rally point" for units to move towards
	virtual Bool getExitPosition( Coord3D& exitPosition ) const override;					///< access to the "Door" position of the production object
	virtual Bool getNaturalRallyPoint( Coord3D& rallyPoint, Bool offset = TRUE ) const override;			///< get the natural "rally point" for units to move towards

	virtual UpdateSleepTime update() override;

protected:

	UnsignedInt m_currentDelay;							///< When this hits zero, I can exit again
	Coord3D			m_rallyPoint;								///< Where units should move to after they have reached the "natural" rally point
	Bool				m_rallyPointExists;					///< Only move to the rally point if this is true
	Real				m_creationClearDistance;		///< I can think of myself as ready when the previous guy is this far away.
	UnsignedInt m_currentBurstCount;				///< how many times must I still override the delay timer

	Bool isFreeToExit() const;
};

inline void QueueProductionExitUpdate::setRallyPoint( const Coord3D *pos )
{
	m_rallyPoint = *pos;
	m_rallyPointExists = true;
}

inline const Coord3D *QueueProductionExitUpdate::getRallyPoint()  const
{
	if (m_rallyPointExists)
		return &m_rallyPoint;

	return nullptr;
}
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/ProductionExitUpdate/QueueProductionExitUpdate.cpp`
```cpp
// FILE: QueueProductionExitUpdate.cpp /////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, January, 2002
// Desc:		Hand off produced Units to me so I can Exit them into the world with my specific style
//					This instance refuses to spit a second out until the first is clear
///////////////////////////////////////////////////////////////////////////////////////////////////

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/RandomValue.h"
#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"
#include "GameLogic/AI.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/Object.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Module/QueueProductionExitUpdate.h"

//-------------------------------------------------------------------------------------------------
QueueProductionExitUpdate::QueueProductionExitUpdate( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{
	m_currentDelay = 0;

	m_creationClearDistance = 0;
	m_rallyPoint.zero();
	// no rally point has been set
	m_rallyPointExists = false;
	m_currentBurstCount = 0;
	const QueueProductionExitUpdateModuleData* md = getQueueProductionExitUpdateModuleData();
	if (md) //sanity
	{
		m_currentBurstCount = md->m_initialBurst;
	}
}

//-------------------------------------------------------------------------------------------------
QueueProductionExitUpdate::~QueueProductionExitUpdate()
{
}

//-------------------------------------------------------------------------------------------------
void QueueProductionExitUpdate::exitObjectViaDoor( Object *newObj, ExitDoorType exitDoor )
{
	DEBUG_ASSERTCRASH(exitDoor == DOOR_1, ("multiple exit doors not supported here"));

	Object *creationObject = getObject();
	if (creationObject)
	{
		const QueueProductionExitUpdateModuleData* md = getQueueProductionExitUpdateModuleData();

		Real exitAngle = creationObject->getOrientation();
		const Matrix3D *transform = creationObject->getTransformMatrix();
		Vector3 loc;
		Coord3D createPoint;

		//
		// calculate the position to create the object at, we take the coord specified
		// in INI which is in model space, rotate it to match the building angle
		// and translate for building location via a transform call
		//
		loc.Set( md->m_unitCreatePoint.x, md->m_unitCreatePoint.y, md->m_unitCreatePoint.z );
		transform->Transform_Vector( *transform, loc, &loc );

		Bool creationInAir = FALSE;
		if( TheTerrainLogic )
		{
			if( loc.Z != TheTerrainLogic->getGroundHeight( loc.X, loc.Y ) )
				creationInAir = TRUE;
		}
		// make sure the point is on the terrain
		if( creationInAir && !md->m_allowAirborneCreationData )
			loc.Z = TheTerrainLogic ? TheTerrainLogic->getGroundHeight( loc.X, loc.Y ) : 0.0f;

		// we need it in Coord3D form
		createPoint.x = loc.X;
		createPoint.y = loc.Y;
		createPoint.z = loc.Z;

		newObj->setPosition( &createPoint );
		newObj->setOrientation( exitAngle );

		//
		// Objects that are created in the air from producers that are in the air get
		// a kick that will make their starting speed equal mine.
		//
		PhysicsBehavior *newObjectPhysics = newObj->getPhysics();
		PhysicsBehavior *myPhysics = creationObject->getPhysics();

		if( (myPhysics != nullptr) && creationInAir && (newObjectPhysics != nullptr) )
		{
			Coord3D startingForce = *myPhysics->getVelocity();
			startingForce.x *= newObjectPhysics->getMass();
			startingForce.y *= newObjectPhysics->getMass();
			startingForce.z *= newObjectPhysics->getMass();
			newObjectPhysics->applyMotiveForce( &startingForce );

			const Real STARTING_PITCH_COEFF = 0.04f;
			Real pitchRate = newObjectPhysics->getCenterOfMassOffset() * STARTING_PITCH_COEFF;
			newObjectPhysics->setPitchRate( pitchRate );
		}


		/** @todo This really should be automatically wrapped up in an activation sequence
		for objects in general */
		// tell the AI about it
		TheAI->pathfinder()->addObjectToPathfindMap( newObj );
		Coord3D tmp;
		getNaturalRallyPoint(tmp);
		// Grid it.
		TheAI->pathfinder()->snapPosition(newObj, &tmp);
		std::vector<Coord3D> exitPath;
		exitPath.push_back(tmp);

		AIUpdateInterface  *ai = newObj->getAIUpdateInterface();
		if (m_rallyPointExists)
		{
			tmp = m_rallyPoint;
			if (ai && ai->isDoingGroundMovement())
			{
				if (TheAI->pathfinder()->adjustDestination(newObj, ai->getLocomotorSet(), &tmp))
					exitPath.push_back(tmp);

			}
		} else {
			// Double the destination to keep redguards from stacking.
			exitPath.push_back(tmp);
		}
		if (ai) {
			ai->aiFollowExitProductionPath( &exitPath, creationObject, CMD_FROM_AI );
		}
		m_currentDelay = md->m_exitDelayData;

		if (m_currentBurstCount)
			m_currentBurstCount--; // fewer and fewer units to burst

	}

}


Bool QueueProductionExitUpdate::getExitPosition( Coord3D& exitPosition ) const
{
	const Object *obj = getObject();
	if (!obj)
		return FALSE;

	const Matrix3D *transform = obj->getTransformMatrix();

	const QueueProductionExitUpdateModuleData *md = getQueueProductionExitUpdateModuleData();

	Vector3 loc;
	loc.Set( md->m_unitCreatePoint.x, md->m_unitCreatePoint.y, md->m_unitCreatePoint.z );
	transform->Transform_Vector( *transform, loc, &loc );

	exitPosition.x = loc.X;
	exitPosition.y = loc.Y;
	exitPosition.z = loc.Z;

	return TRUE;

}


//-------------------------------------------------------------------------------------------------
ExitDoorType QueueProductionExitUpdate::reserveDoorForExit( const ThingTemplate* objType, Object *specificObject )
{
	return isFreeToExit() ? DOOR_1 : DOOR_NONE_AVAILABLE;
}

//-------------------------------------------------------------------------------------------------
void QueueProductionExitUpdate::unreserveDoorForExit( ExitDoorType exitDoor )
{
	/* nothing */
}

//-------------------------------------------------------------------------------------------------
Bool QueueProductionExitUpdate::isFreeToExit() const
{
	Bool stillBursting = (m_currentBurstCount > 0);
	Bool stillDelaying = (m_currentDelay != 0);

	if ( stillBursting )
	{
		return TRUE;
	}

	return ( ! stillDelaying );
}

//-------------------------------------------------------------------------------------------------
UpdateSleepTime QueueProductionExitUpdate::update()
{
	// This update needs to decide when we are again free to exit something
	if (isFreeToExit())
	{
		m_currentDelay = 0;
		return UPDATE_SLEEP_NONE;
	}

	m_currentDelay--;
	return UPDATE_SLEEP_NONE;
}

//-------------------------------------------------------------------------------------------------
void QueueProductionExitUpdate::exitObjectByBudding( Object *newObj, Object *budHost )
{
	if (budHost) // object has provided a host for us
	{
		const Coord3D *newPos = budHost->getPosition();
		Real     newAngle = budHost->getOrientation();
		PathfindLayerEnum newLayer = budHost->getLayer();

		newObj->setPosition( newPos );
		newObj->setOrientation( newAngle );
		newObj->setLayer(newLayer);

	}
	else // we need to just copy the host object
	{
		newObj->setPosition( getObject()->getPosition() );
		newObj->setOrientation( getObject()->getOrientation() );
	}

	//This causes the formation to break like billard balls
	AIUpdateInterface *ai = newObj->getAI();
	if (ai)
	{
		ai->aiMoveToPosition( newObj->getPosition(), CMD_FROM_AI );// IOW, I cant stay here, if somebody else is here
	}

	const QueueProductionExitUpdateModuleData* data = getQueueProductionExitUpdateModuleData();
	m_currentDelay = data->m_exitDelayData;

	if (m_currentBurstCount)
		m_currentBurstCount--; // fewer and fewer units to burst

}





Bool QueueProductionExitUpdate::getNaturalRallyPoint( Coord3D& rallyPoint, Bool offset )  const
{
	const QueueProductionExitUpdateModuleData *data = getQueueProductionExitUpdateModuleData();
	Vector3 p;

	//
	// get the natural rally point from the INI definition, this coord is in model space relative
	// to the model (0,0,0)
	//
	p.X = data->m_naturalRallyPoint.x;
	p.Y = data->m_naturalRallyPoint.y;
	p.Z = data->m_naturalRallyPoint.z;

	if ( offset )
	{
		Vector3 offset = p;
		offset.Normalize();
		offset *= (2*PATHFIND_CELL_SIZE_F);
		p+=offset;
	}

	// transform the point into world space
	const Matrix3D *transform = getObject()->getTransformMatrix();
	transform->Transform_Vector( *transform, p, &p );
	rallyPoint.x = p.X; rallyPoint.y = p.Y; rallyPoint.z = p.Z;
	return TRUE;
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void QueueProductionExitUpdate::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void QueueProductionExitUpdate::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// current delay
	xfer->xferUnsignedInt( &m_currentDelay );

	// rally point
	xfer->xferCoord3D( &m_rallyPoint );

	// rally point exists
	xfer->xferBool( &m_rallyPointExists );

	// creation clear distance
	xfer->xferReal( &m_creationClearDistance );

	// current burst count
	xfer->xferUnsignedInt( &m_currentBurstCount );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void QueueProductionExitUpdate::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
