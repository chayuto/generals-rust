# Translation Packet `PKT-333`: `HeightDieUpdate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-333`
- **Work Package:** `WP-5.4` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/update/height_die_update.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (13):**
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameClient/ParticleSys.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/ParticleSys.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/ContainModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ContainModule.h`
  - `GameLogic/Module/HeightDieUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/HeightDieUpdate.h`
  - `GameLogic/Module/PhysicsUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PhysicsUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `GameLogic/TerrainLogic.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/TerrainLogic.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (6):**
  - `"TargetHeight" => INI::parseReal`
  - `"TargetHeightIncludesStructures" => INI::parseBool`
  - `"OnlyWhenMovingDown" => INI::parseBool`
  - `"DestroyAttachedParticlesAtHeight" => INI::parseReal`
  - `"SnapToGroundOnDeath" => INI::parseBool`
  - `"InitialDelay" => INI::parseDurationUnsignedInt`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (5 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferBool( &m_hasDied )`
  3. `xferBool( &m_particlesDestroyed )`
  4. `xferCoord3D( &m_lastPosition )`
  5. `xferUnsignedInt( &m_earliestDeathFrame )`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/HeightDieUpdate.h`
```cpp
// FILE: HeightDieUpdate.h ////////////////////////////////////////////////////////////////////////
// Author: Objects that will die when the are a certain height above the terrain or objects
// Desc:   Colin Day, April 2002
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/UpdateModule.h"

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class HeightDieUpdateModuleData: public UpdateModuleData
{

public:

	HeightDieUpdateModuleData();

	static void buildFieldParse(MultiIniFieldParse& p);

	Real m_targetHeightAboveTerrain;				///< die at this height above terrain
	Bool m_targetHeightIncludesStructures;	///< target height considers terrain AND structure height underneath us
	Bool m_onlyWhenMovingDown;							///< don't detonate unless moving in downward z dir
	Real m_destroyAttachedParticlesAtHeight;  ///< HACK, destroy any attached particle system of object when below this height
	Bool m_snapToGroundOnDeath;							///< snap to the ground when killed
	UnsignedInt m_initialDelay;							///< Don't explode before this time

};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class HeightDieUpdate : public UpdateModule
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( HeightDieUpdate, "HeightDieUpdate" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( HeightDieUpdate, HeightDieUpdateModuleData )

public:

	HeightDieUpdate( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	virtual UpdateSleepTime update() override;

protected:


	Bool m_hasDied;							///< TRUE once we have triggered death
	Bool m_particlesDestroyed;	///< TRUE once we destroy attached systems (so we do it only once)
	Coord3D m_lastPosition;			///< we record our last position for logic that needs to know our direction of travel
	UnsignedInt m_earliestDeathFrame; ///< Earliest we are allowed to think about dying

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/HeightDieUpdate.cpp`
```cpp
// FILE: HeightDieUpdate.cpp //////////////////////////////////////////////////////////////////////
// Author: Objects that will die when the are a certain height above the terrain or objects
// Desc:   Colin Day, April 2002
///////////////////////////////////////////////////////////////////////////////////////////////////

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"
#include "GameClient/InGameUI.h"
#include "GameClient/ParticleSys.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/TerrainLogic.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/HeightDieUpdate.h"
#include "GameLogic/Module/PhysicsUpdate.h"


//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
HeightDieUpdateModuleData::HeightDieUpdateModuleData()
{

	m_targetHeightAboveTerrain = 0.0f;
	m_targetHeightIncludesStructures = FALSE;
	m_onlyWhenMovingDown = FALSE;
	m_destroyAttachedParticlesAtHeight = -1.0f;
	m_snapToGroundOnDeath = FALSE;
	m_initialDelay = 0;

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void HeightDieUpdateModuleData::buildFieldParse(MultiIniFieldParse& p)
{

  UpdateModuleData::buildFieldParse( p );

	static const FieldParse dataFieldParse[] =
	{
		{ "TargetHeight", INI::parseReal, nullptr, offsetof( HeightDieUpdateModuleData, m_targetHeightAboveTerrain ) },
		{ "TargetHeightIncludesStructures", INI::parseBool, nullptr, offsetof( HeightDieUpdateModuleData, m_targetHeightIncludesStructures ) },
		{ "OnlyWhenMovingDown", INI::parseBool, nullptr, offsetof( HeightDieUpdateModuleData, m_onlyWhenMovingDown ) },
		{ "DestroyAttachedParticlesAtHeight", INI::parseReal, nullptr, offsetof( HeightDieUpdateModuleData, m_destroyAttachedParticlesAtHeight ) },
		{ "SnapToGroundOnDeath", INI::parseBool, nullptr, offsetof( HeightDieUpdateModuleData, m_snapToGroundOnDeath ) },
		{ "InitialDelay", INI::parseDurationUnsignedInt, nullptr, offsetof( HeightDieUpdateModuleData, m_initialDelay ) },
		{ nullptr, nullptr, nullptr, 0 }

	};

  p.add(dataFieldParse);

}

///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
HeightDieUpdate::HeightDieUpdate( Thing *thing, const ModuleData* moduleData )
																: UpdateModule( thing, moduleData )
{
	m_hasDied = FALSE;
	m_particlesDestroyed = FALSE;
	m_lastPosition.x = -1.0f;
	m_lastPosition.y = -1.0f;
	m_lastPosition.z = -1.0f;
	m_earliestDeathFrame = UINT_MAX;
	// m_lastPosition = *thing->getPosition();

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
HeightDieUpdate::~HeightDieUpdate()
{

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
UpdateSleepTime HeightDieUpdate::update()
{
	UnsignedInt now = TheGameLogic->getFrame();
	if( m_earliestDeathFrame == UINT_MAX )
		m_earliestDeathFrame = now + getHeightDieUpdateModuleData()->m_initialDelay;

	// If at least a one frame delay has been set, then stop for a while
	if( m_earliestDeathFrame > now )
		return UPDATE_SLEEP_NONE;

	// do nothing if we're contained within other objects ... like a transport
	if( getObject()->getContainedBy() != nullptr )
	{

		// keep track of our last position even though we're not doing anything yet
		m_lastPosition = *getObject()->getPosition();

		// get outta here
		return UPDATE_SLEEP_NONE;

	}

	// get the module data
	const HeightDieUpdateModuleData *modData = getHeightDieUpdateModuleData();

	// get our current position
	const Coord3D *pos = getObject()->getPosition();

	Bool directionOK = TRUE;
	if( m_hasDied == FALSE )
	{

		if( modData->m_onlyWhenMovingDown )
		{

			if( pos->z >= m_lastPosition.z )
				directionOK = FALSE;

		}

		// get the terrain height
		Real terrainHeightAtPos = TheTerrainLogic->getGroundHeight( pos->x, pos->y );

		// if including structures, check for bridges
		if (modData->m_targetHeightIncludesStructures)
		{
			PathfindLayerEnum layer = TheTerrainLogic->getHighestLayerForDestination(pos);
			if (layer != LAYER_GROUND)
			{
				Real layerHeight = TheTerrainLogic->getLayerHeight(pos->x, pos->y, layer);
				if (layerHeight > terrainHeightAtPos)
					terrainHeightAtPos = layerHeight;
			}
		}

		//
		// our target height to die at is by default the height specified in the INI entry above
		// the terrain ... we may change our target height if we care about dying above
		// objects under us (see below)
		//
		Real targetHeight = terrainHeightAtPos + modData->m_targetHeightAboveTerrain;

		//
		// if we consider objects under us ... we will die when we are the specified distance above
		// those objects
		//
		if( modData->m_targetHeightIncludesStructures == TRUE )
		{

			// scan all objects in the radius of our extent and find the tallest height among them
			PartitionFilterAcceptByKindOf filter1( MAKE_KINDOF_MASK( KINDOF_STRUCTURE ),KINDOFMASK_NONE );
			PartitionFilter *filters[] = { &filter1, nullptr };
			Real range = getObject()->getGeometryInfo().getBoundingCircleRadius();
			ObjectIterator *iter = ThePartitionManager->iterateObjectsInRange( getObject(),
																																				 range,
																																				 FROM_BOUNDINGSPHERE_3D,
																																				 filters );
			MemoryPoolObjectHolder hold( iter );
			Object *obj;

			Real tallestHeight = 0.0f;
			Real thisHeight;
			for( obj = iter->first(); obj; obj = iter->next() )
			{

				// ignore ourselves
				if( obj == getObject() )
					continue;

				// store the height of the tallest object under us
				thisHeight = obj->getGeometryInfo().getMaxHeightAbovePosition();

				if( thisHeight > tallestHeight )
					tallestHeight = thisHeight;

			}

			//
			// our target height is either the height above the terrain as specified by the INI
			// entry for the object that has this update ... or it is the building height of the
			// tallest thing under us
			//
			if( tallestHeight > modData->m_targetHeightAboveTerrain )
				targetHeight = tallestHeight + terrainHeightAtPos;

		}

		// if we are below the target height ... DIE!
		if( pos->z < targetHeight && directionOK )
		{

			// if we're supposed to snap us to the ground on death do so
			// AND: even if we're not snapping to ground, be sure we don't go BELOW ground
			if( modData->m_snapToGroundOnDeath || pos->z < terrainHeightAtPos )
			{
				Coord3D ground;

				ground.x = pos->x;
				ground.y = pos->y;
				ground.z = terrainHeightAtPos;
				getObject()->setPosition( &ground );

			}

			// kill the object
			getObject()->kill();

			// we have died ... don't do this again
			m_hasDied = TRUE;

		}

	}

	//
	// if our height is below the destroy attached particles height above the terrain, clean
	// them up from the particle system
	//
	if( m_particlesDestroyed == FALSE && pos->z < modData->m_destroyAttachedParticlesAtHeight && (m_hasDied || directionOK) )
	{

		// destroy them
		TheParticleSystemManager->destroyAttachedSystems( getObject() );

		// don't do this again
		m_particlesDestroyed = TRUE;

	}

	// save our current position as the last position we monitored
	m_lastPosition = *pos;

	return UPDATE_SLEEP_NONE;

}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void HeightDieUpdate::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version
	* 2: m_earliestDeathFrame
*/
// ------------------------------------------------------------------------------------------------
void HeightDieUpdate::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 2;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// has died
	xfer->xferBool( &m_hasDied );

	// particles destroyed
	xfer->xferBool( &m_particlesDestroyed );

	// last position
	xfer->xferCoord3D( &m_lastPosition );

	if( version >= 2 )
		xfer->xferUnsignedInt( &m_earliestDeathFrame );
	else
		m_earliestDeathFrame = 0;

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void HeightDieUpdate::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
