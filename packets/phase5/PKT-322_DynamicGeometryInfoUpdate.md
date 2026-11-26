# Translation Packet `PKT-322`: `DynamicGeometryInfoUpdate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-322`
- **Work Package:** `WP-5.4` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/update/dynamic_geometry_info_update.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (6):**
  - `Common/Geometry.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Geometry.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameLogic/Module/DynamicGeometryInfoUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DynamicGeometryInfoUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (9):**
  - `"InitialDelay" => INI::parseDurationUnsignedInt`
  - `"InitialHeight" => INI::parseReal`
  - `"InitialMajorRadius" => INI::parseReal`
  - `"InitialMinorRadius" => INI::parseReal`
  - `"FinalHeight" => INI::parseReal`
  - `"FinalMajorRadius" => INI::parseReal`
  - `"FinalMinorRadius" => INI::parseReal`
  - `"TransitionTime" => INI::parseDurationUnsignedInt`
  - `"ReverseAtTransitionTime" => INI::parseBool`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (14 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUnsignedInt( &m_startingDelayCountdown )`
  3. `xferUnsignedInt( &m_timeActive )`
  4. `xferBool( &m_started )`
  5. `xferBool( &m_finished )`
  6. `xferBool( &m_reverseAtTransitionTime )`
  7. `xferUser( &m_direction, sizeof( DynamicGeometryDirection ) )`
  8. `xferBool( &m_switchedDirections )`
  9. `xferReal( &m_initialHeight )`
  10. `xferReal( &m_initialMajorRadius )`
  11. `xferReal( &m_initialMinorRadius )`
  12. `xferReal( &m_finalHeight )`
  13. `xferReal( &m_finalMajorRadius )`
  14. `xferReal( &m_finalMinorRadius )`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DynamicGeometryInfoUpdate.h`
```cpp
// FILE: DynamicGeometryInfoUpdate.h //////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, April 2002
// Desc:   Update module that changes the object's GeometryInfo
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/Geometry.h"
#include "GameLogic/Module/UpdateModule.h"

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
class DynamicGeometryInfoUpdateModuleData : public ModuleData
{
public:

	UnsignedInt m_initialDelay;

	Real m_initialHeight;
	Real m_initialMajorRadius;
	Real m_initialMinorRadius;

	Real m_finalHeight;
	Real m_finalMajorRadius;
	Real m_finalMinorRadius;

	UnsignedInt m_transitionTime;

	Bool m_reverseAtTransitionTime;		///< reverse directions once transition time is reached

	// I will go from initial to final in transitionTime frames, smoothly.
	// I won't change type until that is actually needed as a task.

	DynamicGeometryInfoUpdateModuleData();
	static void buildFieldParse(MultiIniFieldParse& p);

private:

};

//-------------------------------------------------------------------------------------------------
/** The default	update module */
//-------------------------------------------------------------------------------------------------
class DynamicGeometryInfoUpdate : public UpdateModule
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( DynamicGeometryInfoUpdate, "DynamicGeometryInfoUpdate" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( DynamicGeometryInfoUpdate, DynamicGeometryInfoUpdateModuleData );

public:

	DynamicGeometryInfoUpdate( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	virtual UpdateSleepTime update() override;

protected:

	UnsignedInt m_startingDelayCountdown;
	UnsignedInt m_timeActive;

	Bool m_started;
	Bool m_finished;

	Bool m_reverseAtTransitionTime;						///< do a reverse at transition time
	enum DynamicGeometryDirection { FORWARD = 1, BACKWARD = -1 };
	DynamicGeometryDirection m_direction;			///< direction we're growing/shrinking
	Bool m_switchedDirections;								///< TRUE once we've switched directions

	Real m_initialHeight;
	Real m_initialMajorRadius;
	Real m_initialMinorRadius;
	Real m_finalHeight;
	Real m_finalMajorRadius;
	Real m_finalMinorRadius;

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/DynamicGeometryInfoUpdate.cpp`
```cpp
// FILE: DynamicGeometryInfoUpdate.cpp //////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, April 2002
// Desc:   Update module that changes the object's GeometryInfo
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/Xfer.h"
#include "GameLogic/Module/DynamicGeometryInfoUpdate.h"
#include "GameLogic/Object.h"

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
DynamicGeometryInfoUpdateModuleData::DynamicGeometryInfoUpdateModuleData()
{

	m_initialDelay = 0;

	m_initialHeight = 0;
	m_initialMajorRadius = 0;
	m_initialMinorRadius = 0;

	m_finalHeight = 0;
	m_finalMajorRadius = 0;
	m_finalMinorRadius = 0;

	m_transitionTime = 1;
	m_reverseAtTransitionTime = FALSE;

}

//-------------------------------------------------------------------------------------------------
/*static*/ void DynamicGeometryInfoUpdateModuleData::buildFieldParse(MultiIniFieldParse& p)
{
	ModuleData::buildFieldParse(p);

	static const FieldParse dataFieldParse[] =
	{

		{ "InitialDelay",					INI::parseDurationUnsignedInt,		nullptr, offsetof(DynamicGeometryInfoUpdateModuleData, m_initialDelay) },

		{ "InitialHeight",				INI::parseReal,										nullptr, offsetof(DynamicGeometryInfoUpdateModuleData, m_initialHeight) },
		{ "InitialMajorRadius",		INI::parseReal,										nullptr, offsetof(DynamicGeometryInfoUpdateModuleData, m_initialMajorRadius) },
		{ "InitialMinorRadius",		INI::parseReal,										nullptr, offsetof(DynamicGeometryInfoUpdateModuleData, m_initialMinorRadius) },

		{ "FinalHeight",					INI::parseReal, nullptr, offsetof(DynamicGeometryInfoUpdateModuleData, m_finalHeight) },
		{ "FinalMajorRadius",			INI::parseReal, nullptr, offsetof(DynamicGeometryInfoUpdateModuleData, m_finalMajorRadius) },
		{ "FinalMinorRadius",			INI::parseReal, nullptr, offsetof(DynamicGeometryInfoUpdateModuleData, m_finalMinorRadius) },

		{ "TransitionTime",				INI::parseDurationUnsignedInt,		nullptr, offsetof(DynamicGeometryInfoUpdateModuleData, m_transitionTime) },
		{ "ReverseAtTransitionTime", INI::parseBool,	 nullptr, offsetof( DynamicGeometryInfoUpdateModuleData, m_reverseAtTransitionTime ) },

		{ nullptr, nullptr, nullptr, 0 }

	};
	p.add(dataFieldParse);
}

//-------------------------------------------------------------------------------------------------
DynamicGeometryInfoUpdate::DynamicGeometryInfoUpdate( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{
	DynamicGeometryInfoUpdateModuleData *modData = (DynamicGeometryInfoUpdateModuleData *)moduleData;
	m_startingDelayCountdown = modData->m_initialDelay;
	m_startingDelayCountdown = max( m_startingDelayCountdown, 1u );
	m_timeActive = 0;

	m_started = FALSE;
	m_finished = FALSE;

	m_reverseAtTransitionTime = modData->m_reverseAtTransitionTime;
	m_switchedDirections = FALSE;

	// record in our instance what initial and final height are
	m_initialHeight = modData->m_initialHeight;
	m_initialMajorRadius = modData->m_initialMajorRadius;
	m_initialMinorRadius = modData->m_initialMinorRadius;
	m_finalHeight = modData->m_finalHeight;
	m_finalMajorRadius = modData->m_finalMajorRadius;
	m_finalMinorRadius = modData->m_finalMinorRadius;

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
DynamicGeometryInfoUpdate::~DynamicGeometryInfoUpdate()
{
}

//-------------------------------------------------------------------------------------------------
/** The update callback. */
//-------------------------------------------------------------------------------------------------
UpdateSleepTime DynamicGeometryInfoUpdate::update()
{
/// @todo srj use SLEEPY_UPDATE here
	if( m_finished )
		return UPDATE_SLEEP_NONE;

	if( !m_started )
	{

		m_startingDelayCountdown--;
		if( m_startingDelayCountdown > 0 )
			return UPDATE_SLEEP_NONE;

		m_started = TRUE;

	}

	// Either we've been running, or we just started right now.  Doesn't matter.
	const DynamicGeometryInfoUpdateModuleData *data = getDynamicGeometryInfoUpdateModuleData();
	Object *me = getObject();
	Real newHeight, newMajor, newMinor;

	Real ratio = 1.0f;
	if (data->m_transitionTime > 0)
	{
		ratio = (float)m_timeActive / (float)data->m_transitionTime;
	}

	newHeight = m_initialHeight + (ratio * (m_finalHeight - m_initialHeight));
	newMajor = m_initialMajorRadius + (ratio * (m_finalMajorRadius - m_initialMajorRadius));
	newMinor = m_initialMinorRadius + (ratio * (m_finalMinorRadius - m_initialMinorRadius));

	// make a new geometry info with the new values
	const GeometryInfo oldGeom = me->getGeometryInfo();
	GeometryInfo newGeom( oldGeom.getGeomType(), oldGeom.getIsSmall(), newHeight, newMajor, newMinor );
	me->setGeometryInfo( newGeom );

	// we've not been active another frame .. increment out counter
	m_timeActive++;

	// greater, so an update that is exactly right will be done
	if( m_timeActive > data->m_transitionTime )
	{

		// if we're supposed to reverse at transition time do so now if we haven't already
		if( m_reverseAtTransitionTime == TRUE )
		{

			// switch directions, reset time active, and turn off the switch at transition time
			m_switchedDirections = TRUE;
			m_timeActive = 0;
			m_reverseAtTransitionTime = FALSE;

			// swap the initial and final values
			m_initialHeight = data->m_finalHeight;
			m_initialMajorRadius = data->m_finalMajorRadius;
			m_initialMinorRadius = data->m_finalMinorRadius;
			m_finalHeight = data->m_initialHeight;
			m_finalMajorRadius = data->m_initialMajorRadius;
			m_finalMinorRadius = data->m_initialMinorRadius;

		}
		else
		{

			// no switch needed ... we're all done
			m_finished = TRUE;

		}

	}

	return UPDATE_SLEEP_NONE;
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void DynamicGeometryInfoUpdate::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void DynamicGeometryInfoUpdate::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// starting delay countdown
	xfer->xferUnsignedInt( &m_startingDelayCountdown );

	// time active
	xfer->xferUnsignedInt( &m_timeActive );

	// started
	xfer->xferBool( &m_started );

	// finished
	xfer->xferBool( &m_finished );

	// reverse at transition time
	xfer->xferBool( &m_reverseAtTransitionTime );

	// direction
	xfer->xferUser( &m_direction, sizeof( DynamicGeometryDirection ) );

	// switched directions
	xfer->xferBool( &m_switchedDirections );

	// initial height
	xfer->xferReal( &m_initialHeight );

	// initial major radius
	xfer->xferReal( &m_initialMajorRadius );

	// initial minor radius
	xfer->xferReal( &m_initialMinorRadius );

	// final height
	xfer->xferReal( &m_finalHeight );

	// final major radius
	xfer->xferReal( &m_finalMajorRadius );

	// final minor radius
	xfer->xferReal( &m_finalMinorRadius );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void DynamicGeometryInfoUpdate::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
