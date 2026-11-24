# Translation Packet `PKT-261`: `BridgeScaffoldBehavior`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-261`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/behavior/bridge_scaffold_behavior.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (8):**
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/BridgeScaffoldBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BridgeScaffoldBehavior.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `PreRTS.h`
- **Implemented Module Facets (1):**
  - `BridgeScaffoldBehaviorInterface* getBridgeScaffoldBehaviorInterface()`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (8 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUser( &m_targetMotion, sizeof( ScaffoldTargetMotion ) )`
  3. `xferCoord3D( &m_createPos )`
  4. `xferCoord3D( &m_riseToPos )`
  5. `xferCoord3D( &m_buildPos )`
  6. `xferReal( &m_lateralSpeed )`
  7. `xferReal( &m_verticalSpeed )`
  8. `xferCoord3D( &m_targetPos )`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BridgeScaffoldBehavior.h`
```cpp
// FILE: BridgeScaffoldBehavior.h /////////////////////////////////////////////////////////////////
// Author: Colin Day, September 2002
// Desc:   Bridge scaffold
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/UpdateModule.h"

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
enum ScaffoldTargetMotion CPP_11(: Int)
{
	STM_STILL,
	STM_RISE,
	STM_BUILD_ACROSS,
	STM_TEAR_DOWN_ACROSS,
	STM_SINK,
};

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
class BridgeScaffoldBehaviorInterface
{

public:

	virtual void setPositions( const Coord3D *createPos,
														 const Coord3D *riseToPos,
														 const Coord3D *buildPos ) = 0;
	virtual void setMotion( ScaffoldTargetMotion targetMotion ) = 0;
	virtual ScaffoldTargetMotion getCurrentMotion() = 0;
	virtual void reverseMotion() = 0;
	virtual void setLateralSpeed( Real lateralSpeed ) = 0;
	virtual void setVerticalSpeed( Real verticalSpeed ) = 0;

};

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
class BridgeScaffoldBehavior : public UpdateModule,
															 public BridgeScaffoldBehaviorInterface
{

	MAKE_STANDARD_MODULE_MACRO( BridgeScaffoldBehavior );
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( BridgeScaffoldBehavior, "BridgeScaffoldBehavior" )

public:

	BridgeScaffoldBehavior( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	// behavior module methods
	virtual BridgeScaffoldBehaviorInterface* getBridgeScaffoldBehaviorInterface() override { return this; }

	// update methods
	virtual UpdateSleepTime update() override;

	// bridge scaffold interface methods
	virtual void setPositions( const Coord3D *createPos,
														 const Coord3D *riseToPos,
														 const Coord3D *buildPos ) override;
	virtual void setMotion( ScaffoldTargetMotion targetMotion ) override;
	virtual ScaffoldTargetMotion getCurrentMotion() override { return m_targetMotion; }
	virtual void reverseMotion() override;
	virtual void setLateralSpeed( Real lateralSpeed ) override { m_lateralSpeed = lateralSpeed; }
	virtual void setVerticalSpeed( Real verticalSpeed ) override { m_verticalSpeed = verticalSpeed; }

	// public interface acquisition
	static BridgeScaffoldBehaviorInterface *getBridgeScaffoldBehaviorInterfaceFromObject( Object *obj );

protected:

	void doVerticalMotion();				///< do rise/sink vertical motion
	void doLateralmotion();					///< do lateral motion

	ScaffoldTargetMotion m_targetMotion;	///< which way our motion should be going (build up, still, tear down etc)
	Coord3D m_createPos;									///< initial position of object creation (in ground)
	Coord3D m_riseToPos;									///< position we "rise to" out of the ground
	Coord3D m_buildPos;										///< position we move to and stop at on the bridge surface
	Real m_lateralSpeed;									///< speed for lateral motions
	Real m_verticalSpeed;									///< speed for vertical motions
	Coord3D m_targetPos;									///< current target position for our motion type

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Behavior/BridgeScaffoldBehavior.cpp`
```cpp
// FILE: BridgeScaffoldBehavior.cpp ///////////////////////////////////////////////////////////////
// Author: Colin Day, September 2002
// Desc:   Bridge scaffold
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"
#include "Common/Xfer.h"
#include "GameClient/InGameUI.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/Module/BridgeScaffoldBehavior.h"

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
BridgeScaffoldBehavior::BridgeScaffoldBehavior( Thing *thing, const ModuleData *moduleData )
											: UpdateModule( thing, moduleData )
{

	m_targetMotion = STM_STILL;
	m_createPos.zero();
	m_riseToPos.zero();
	m_buildPos.zero();
	m_targetPos.zero();
	m_lateralSpeed = 1.0f;
	m_verticalSpeed = 1.0f;

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
BridgeScaffoldBehavior::~BridgeScaffoldBehavior()
{

}

// ------------------------------------------------------------------------------------------------
/** Set all of the target positions that we're care about as a moving scaffold object */
// ------------------------------------------------------------------------------------------------
void BridgeScaffoldBehavior::setPositions( const Coord3D *createPos,
																					 const Coord3D *riseToPos,
																					 const Coord3D *buildPos )
{

	m_createPos = *createPos;
	m_riseToPos = *riseToPos;
	m_buildPos = *buildPos;

}

// ------------------------------------------------------------------------------------------------
/** Set us moving to the right target position for the requested motion type */
// ------------------------------------------------------------------------------------------------
void BridgeScaffoldBehavior::setMotion( ScaffoldTargetMotion targetMotion )
{

	// save the target motion type
	m_targetMotion = targetMotion;

	// given the target motion, pick a destination target
	switch( m_targetMotion )
	{

		// --------------------------------------------------------------------------------------------
		case STM_RISE:
		case STM_TEAR_DOWN_ACROSS:
			m_targetPos = m_riseToPos;
			break;

		// --------------------------------------------------------------------------------------------
		case STM_BUILD_ACROSS:
			m_targetPos = m_buildPos;
			break;

		// --------------------------------------------------------------------------------------------
		case STM_SINK:
			m_targetPos = m_createPos;
			break;

	}

}

// ------------------------------------------------------------------------------------------------
/** Whatever our current state of motion is, reverse it */
// ------------------------------------------------------------------------------------------------
void BridgeScaffoldBehavior::reverseMotion()
{

	switch( m_targetMotion )
	{

		case STM_STILL:
			setMotion( STM_TEAR_DOWN_ACROSS );
			break;

		case STM_RISE:
			setMotion( STM_SINK );
			break;

		case STM_BUILD_ACROSS:
			setMotion( STM_TEAR_DOWN_ACROSS );
			break;

		case STM_TEAR_DOWN_ACROSS:
			setMotion( STM_BUILD_ACROSS );
			break;

		case STM_SINK:
			setMotion( STM_RISE );
			break;

	}

}

// ------------------------------------------------------------------------------------------------
/** The update method */
// ------------------------------------------------------------------------------------------------
UpdateSleepTime BridgeScaffoldBehavior::update()
{

	// do nothing if we're not in motion
	if( m_targetMotion == STM_STILL )
		return UPDATE_SLEEP_NONE;

	// get our info
	Object *us = getObject();
	const Coord3D *ourPos = us->getPosition();

	// compute direction vector from our position to the target position
	Coord3D dirV;
	dirV.x = m_targetPos.x - ourPos->x;
	dirV.y = m_targetPos.y - ourPos->y;
	dirV.z = m_targetPos.z - ourPos->z;

	// use normalized direction vector "v" to do the pulling movement
	Coord3D v = dirV;
	v.normalize();

	// depending on our motion type, we move at different speeds
	Real topSpeed = 1.0f;
	Coord3D *start, *end;
	switch( m_targetMotion )
	{

		case STM_RISE:
			topSpeed = m_verticalSpeed;
			start = &m_createPos;
			end = &m_riseToPos;
			break;

		case STM_SINK:
			topSpeed = m_verticalSpeed;
			start = &m_riseToPos;
			end = &m_createPos;
			break;

		case STM_BUILD_ACROSS:
			topSpeed = m_lateralSpeed;
			start = &m_riseToPos;
			end = &m_buildPos;
			break;

		case STM_TEAR_DOWN_ACROSS:
			topSpeed = m_lateralSpeed;
			start = &m_buildPos;
			end = &m_riseToPos;
			break;

		default:
			DEBUG_CRASH(("Unhandled case in BridgeScaffoldBehavior::update()"));
			return UPDATE_SLEEP_NONE;

	}

	// adjust speed so it's slower at the end of motion
	Coord3D speedVector;
	speedVector.x = end->x - start->x;
	speedVector.y = end->y - start->y;
	speedVector.z = end->z - start->z;
	Real totalDistance = speedVector.length() * 0.25f;
	speedVector.x = end->x - ourPos->x;
	speedVector.y = end->y - ourPos->y;
	speedVector.z = end->z - ourPos->z;
	Real ourDistance = speedVector.length();
	Real speed = (ourDistance / totalDistance) * topSpeed;
	Real minSpeed = topSpeed * 0.08f;
	if( speed < minSpeed )
		speed = minSpeed;
	if( speed > topSpeed )
		speed = topSpeed;

	//
	// make sure that speed can't get so incredibly small that we never finish our
	// movement no matter what the speed and distance are
	//
	if( speed < 0.001f )
		speed = 0.001f;

	// compute the new position given the speed
	Coord3D newPos;
	newPos.x = v.x * speed + ourPos->x;
	newPos.y = v.y * speed + ourPos->y;
	newPos.z = v.z * speed + ourPos->z;

	//
	// will this new position push us beyond our target destination, we will take the vector
	// from the new position to the destination and the vector from our current present position
	// tot he destination and dot them together ... if the result is < 0 then we have will
	// overshoot the distance if we use the new position
	//
	Coord3D tooFarVector;
	tooFarVector.x = m_targetPos.x - newPos.x;
	tooFarVector.y = m_targetPos.y - newPos.y;
	tooFarVector.z = m_targetPos.z - newPos.z;
	if( tooFarVector.x * dirV.x + tooFarVector.y * dirV.y + tooFarVector.z * dirV.z <= 0.0f )
	{

		// use the destination position
		newPos = m_targetPos;

		//
		// we have reached our target position, switch motion to the next position in
		// the chain (which may be stay still and don't move anymore)
		//
		switch( m_targetMotion )
		{

			case STM_RISE: setMotion( STM_BUILD_ACROSS ); break;
			case STM_BUILD_ACROSS: setMotion( STM_STILL ); break;
			case STM_TEAR_DOWN_ACROSS: setMotion( STM_SINK ); break;

			case STM_SINK:
			{

				// we are done with a sinking motion, destroy the scaffold object as our job is done
				TheGameLogic->destroyObject( us );
				break;

			}

		}

	}

	// set the new position
	us->setPosition( &newPos );

	// do not sleep
	return UPDATE_SLEEP_NONE;

}

// ------------------------------------------------------------------------------------------------
/** STATIC MEMBER:
	* Helper function to retrieve a bridge scaffold interface from an object if one is present */
// ------------------------------------------------------------------------------------------------
BridgeScaffoldBehaviorInterface *BridgeScaffoldBehavior::getBridgeScaffoldBehaviorInterfaceFromObject( Object *obj )
{

	// sanity
	if( obj == nullptr )
		return nullptr;

	// get the bridge tower behavior interface
	BridgeScaffoldBehaviorInterface *bridgeScaffoldInterface = nullptr;
	BehaviorModule **bmi;
	for( bmi = obj->getBehaviorModules(); *bmi; ++bmi )
	{

		bridgeScaffoldInterface = (*bmi)->getBridgeScaffoldBehaviorInterface();
		if( bridgeScaffoldInterface )
			return bridgeScaffoldInterface;

	}

	// interface not found
	return nullptr;

}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void BridgeScaffoldBehavior::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void BridgeScaffoldBehavior::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// target motion
	xfer->xferUser( &m_targetMotion, sizeof( ScaffoldTargetMotion ) );

	// create pos
	xfer->xferCoord3D( &m_createPos );

	// rise to pos
	xfer->xferCoord3D( &m_riseToPos );

	// build pos
	xfer->xferCoord3D( &m_buildPos );

	// lateral speed
	xfer->xferReal( &m_lateralSpeed );

	// vertical speed
	xfer->xferReal( &m_verticalSpeed );

	// current target pos
	xfer->xferCoord3D( &m_targetPos );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void BridgeScaffoldBehavior::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
