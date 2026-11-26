# Translation Packet `PKT-365`: `SlavedUpdate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-365`
- **Work Package:** `WP-5.4` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/update/slaved_update.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (20):**
  - `Common/INI.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/INI.h`
  - `Common/MiscAudio.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/MiscAudio.h`
  - `Common/RandomValue.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/RandomValue.h`
  - `Common/Team.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Team.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/ParticleSys.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/ParticleSys.h`
  - `GameLogic/AIPathfind.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/AIPathfind.h`
  - `GameLogic/Damage.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/Damage.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Locomotor.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Locomotor.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/SlavedUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SlavedUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `GameLogic/TerrainLogic.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/TerrainLogic.h`
  - `GameLogic/Weapon.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h`
  - `PreRTS.h`
- **Implemented Module Facets (1):**
  - `SlavedUpdateInterface* getSlavedUpdateInterface()`
- **INI `FieldParse` Table Entries (19):**
  - `"GuardMaxRange" => INI::parseInt`
  - `"GuardWanderRange" => INI::parseInt`
  - `"AttackRange" => INI::parseInt`
  - `"AttackWanderRange" => INI::parseInt`
  - `"ScoutRange" => INI::parseInt`
  - `"ScoutWanderRange" => INI::parseInt`
  - `"RepairRange" => INI::parseInt`
  - `"RepairMinAltitude" => INI::parseReal`
  - `"RepairMaxAltitude" => INI::parseReal`
  - `"DistToTargetToGrantRangeBonus" => INI::parseInt`
  - `"RepairRatePerSecond" => INI::parseReal`
  - `"RepairWhenBelowHealth%" => INI::parseInt`
  - `"RepairMinReadyTime" => INI::parseDurationUnsignedInt`
  - `"RepairMaxReadyTime" => INI::parseDurationUnsignedInt`
  - `"RepairMinWeldTime" => INI::parseDurationUnsignedInt`
  - `"RepairMaxWeldTime" => INI::parseDurationUnsignedInt`
  - `"RepairWeldingSys" => INI::parseAsciiString`
  - `"RepairWeldingFXBone" => INI::parseAsciiString`
  - `"StayOnSameLayerAsMaster" => INI::parseBool`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (6 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferObjectID( &m_slaver )`
  3. `xferCoord3D( &m_guardPointOffset)`
  4. `xferInt( &m_framesToWait )`
  5. `xferUser( &m_repairState, sizeof( RepairStates ) )`
  6. `xferBool( &m_repairing )`
- **PRNG Calls (9 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValue( 0, 2*PI )`
  - `GameLogicRandomValue( 0, 2*PI )`
  - `GameLogicRandomValue( 0, 2*PI )`
  - `GameLogicRandomValueReal( data->m_repairMinAltitude, data->m_repairMaxAltitude )`
  - `GameLogicRandomValue( data->m_minReadyFrames, data->m_maxReadyFrames )`
  - `GameLogicRandomValue( data->m_minWeldFrames, data->m_maxWeldFrames )`
  - `GameLogicRandomValue( 0, 2*PI )`
  - `GameLogicRandomValueReal( data->m_repairMinAltitude, data->m_repairMaxAltitude )`
  - `GameLogicRandomValue( 0, 2*PI )`

## 2.5 Existing Rust Symbol Index (Use These Exact Paths)
- `zh_logic::modules::update::auto_deposit_update`: `AutoDepositUpdateModuleData`, `AutoDepositUpdate`
- `zh_logic::modules::update::base_renerate_update`: `BaseRegenerateUpdateModuleData`, `BaseRegenerateUpdate`
- `zh_logic::modules::update::default_production_exit_update`: `Producer`, `DefaultProductionExitUpdateModuleData`, `DefaultProductionExitUpdate`
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
- `zh_logic::modules::update::queue_production_exit_update`: `QueueProductionExitUpdateModuleData`, `QueueProductionExitUpdate`
- `zh_logic::modules::update::radar_update`: `RadarUpdateModuleData`, `RadarUpdate`
- `zh_logic::modules::update::radius_decal_update`: `RadiusDecalUpdateModuleData`, `RadiusDecalUpdate`
- `zh_logic::modules::update::repair_dock_update`: `RepairDockUpdateModuleData`, `RepairDockUpdate`
- `zh_logic::modules::update::slaved_update`: `REPAIRSTATE_NONE`, `REPAIRSTATE_UNPACKING`, `REPAIRSTATE_PACKING`, `REPAIRSTATE_READY`, `REPAIRSTATE_EXTENDING`, `REPAIRSTATE_RETRACTING`, `REPAIRSTATE_WELDING`, `SlavedUpdateModuleData`, `SlavedUpdate`
- `zh_logic::modules::update::smart_bomb_target_homing_update`: `SmartBombTargetHomingUpdateModuleData`, `SmartBombTargetHomingUpdate`
- `zh_logic::modules::update::spawn_point_production_exit_update`: `MAX_SPAWN_POINTS`, `SpawnPointProductionExitUpdateModuleData`, `SpawnPointProductionExitUpdate`
- `zh_logic::modules::update::spy_vision_update`: `SpyVisionUpdateModuleData`, `SpyVisionUpdate`, `fn`
- `zh_logic::modules::update::stealth_detector_update`: `StealthDetectorUpdateModuleData`, `StealthDetectorUpdate`
- `zh_logic::modules::update::stealth_update`: `STEALTH_NOT_WHILE_ATTACKING`, `STEALTH_NOT_WHILE_MOVING`, `STEALTH_NOT_WHILE_USING_ABILITY`, `STEALTH_NOT_WHILE_FIRING_PRIMARY`, `STEALTH_NOT_WHILE_FIRING_SECONDARY`, `STEALTH_NOT_WHILE_FIRING_TERTIARY`, `STEALTH_ONLY_WITH_BLACK_MARKET`, `STEALTH_NOT_WHILE_TAKING_DAMAGE`, `STEALTH_NOT_WHILE_RIDERS_ATTACKING`, `STEALTH_NOT_WHILE_FIRING_WEAPON`, `STEALTH_LEVEL_NAMES`, `StealthUpdateModuleData`, `StealthUpdate`
- `zh_logic::modules::update::structure_topple_update`: `STPHASE_INITIAL`, `STPHASE_DELAY`, `STPHASE_FINAL`, `ST_PHASE_COUNT`, `TOPPLESTATE_STANDING`, `TOPPLESTATE_WAITINGFORTOPPLESTART`, `TOPPLESTATE_TOPPLING`, `TOPPLESTATE_WAITINGFORDONE`, `TOPPLESTATE_DONE`, `StructureToppleUpdateModuleData`, `StructureToppleUpdate`
- `zh_logic::modules::update::supply_center_dock_update`: `SupplyCenterDockUpdateModuleData`, `SupplyCenterDockUpdate`
- `zh_logic::modules::update::supply_center_production_exit_update`: `SupplyCenterProductionExitUpdateModuleData`, `SupplyCenterProductionExitUpdate`
- `zh_logic::modules::update::supply_warehouse_dock_update`: `SupplyWarehouseDockUpdateModuleData`, `SupplyWarehouseDockUpdate`
- `zh_logic::modules::update::topple_update`: `TOPPLE_OPTIONS_NONE`, `TOPPLE_OPTIONS_NO_BOUNCE`, `TOPPLE_OPTIONS_NO_FX`, `TOPPLE_UPRIGHT`, `TOPPLE_FALLING`, `TOPPLE_DOWN`, `ToppleUpdateModuleData`, `ToppleUpdate`
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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SlavedUpdate.h`
```cpp
// FILE: SlavedUpdate.cpp /////////////////////////////////////////////////////////////////////////
// Author: Matt Campbell, March 2002
// Updated: Kris Morness, July 2002 -- Add support for advanced scout drone abilities
// Desc:  Slaved unit(s) remain close to their master. Used by scout drones, and used by stinger
//        soldiers that are close to a stinger site. It's important to note that any slaved units
//				can use any or all features, some of which are specialized.
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

const Int SLAVED_UPDATE_RATE = LOGICFRAMES_PER_SECOND/4; ///< This is a low priority module that only needs to be called every this many frames

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/INI.h"
#include "GameLogic/Module/UpdateModule.h"
class DamageInfo;
enum ModelConditionFlagType CPP_11(: Int);

//-------------------------------------------------------------------------------------------------
class SlavedUpdateModuleData : public UpdateModuleData
{
public:
	//Example: Currently used by scout drones owned by rangers AND stinger soldiers owned by stinger sites.
	Int m_guardMaxRange;		//Distance from master I'm allowed when he's idle. If I go too far away, I'll come back.
	Int m_guardWanderRange;	//Allowable wander distance from master while guarding master.

	//Example: Below are only currently used by various drones owned by rangers/vehicles
	Int m_attackRange;			//When master attacks a target, I'll go to the target -- how far am I allowed to go in this case.
	Int m_attackWanderRange;//If I'm at the target point, how far can I wander from it.
	Int m_scoutRange;				//If master is moving somewhere, I'll scout ahead -- how far am I allowed to go?
	Int m_scoutWanderRange;	//If I'm at the scout point, how far can I wander from it.
	Int m_distToTargetToGrantRangeBonus;	//How close I have to be to the master's target in order to grant master a range bonus.

	//Example: Below are used by battle drones
	Int m_repairRange;
	Real m_repairMinAltitude;
	Real m_repairMaxAltitude;

	Real m_repairRatePerSecond;	//How fast I can repair
	Int m_repairWhenHealthBelowPercentage; //When should I prioritize repairing my master.
	Int m_minReadyFrames;
	Int m_maxReadyFrames;
	Int m_minWeldFrames;
	Int m_maxWeldFrames;
	AsciiString m_weldingSysName;
	AsciiString m_weldingFXBone;

	Bool m_stayOnSameLayerAsMaster;

	SlavedUpdateModuleData()
	{
		m_guardMaxRange = 0;
		m_guardWanderRange = 0;
		m_attackRange = 0;
		m_attackWanderRange = 0;
		m_scoutRange = 0;
		m_scoutWanderRange = 0;
		m_distToTargetToGrantRangeBonus = 0;
		m_repairRatePerSecond = 0.0f;
		m_repairWhenHealthBelowPercentage = 0;
		m_repairMinAltitude = 0.0f;
		m_repairMaxAltitude = 0.0f;
		m_minWeldFrames = 0;
		m_maxWeldFrames = 0;
		m_minReadyFrames = 0;
		m_maxReadyFrames = 0;
		m_stayOnSameLayerAsMaster = false;
	}

	static void buildFieldParse(MultiIniFieldParse& p)
	{
    UpdateModuleData::buildFieldParse(p);
		static const FieldParse dataFieldParse[] =
		{
			{ "GuardMaxRange",			INI::parseInt,	nullptr, offsetof( SlavedUpdateModuleData, m_guardMaxRange ) },
			{ "GuardWanderRange",		INI::parseInt,	nullptr, offsetof( SlavedUpdateModuleData, m_guardWanderRange ) },
			{ "AttackRange",				INI::parseInt,	nullptr, offsetof( SlavedUpdateModuleData, m_attackRange ) },
			{ "AttackWanderRange",	INI::parseInt,	nullptr, offsetof( SlavedUpdateModuleData, m_attackWanderRange ) },
			{ "ScoutRange",					INI::parseInt,	nullptr, offsetof( SlavedUpdateModuleData, m_scoutRange ) },
			{ "ScoutWanderRange",		INI::parseInt,	nullptr, offsetof( SlavedUpdateModuleData, m_scoutWanderRange ) },
			{ "RepairRange",				INI::parseInt,	nullptr, offsetof( SlavedUpdateModuleData, m_repairRange ) },
			{ "RepairMinAltitude",		  INI::parseReal, nullptr, offsetof( SlavedUpdateModuleData, m_repairMinAltitude ) },
			{ "RepairMaxAltitude",		  INI::parseReal, nullptr, offsetof( SlavedUpdateModuleData, m_repairMaxAltitude ) },
			{ "DistToTargetToGrantRangeBonus", INI::parseInt, nullptr, offsetof( SlavedUpdateModuleData, m_distToTargetToGrantRangeBonus ) },
			{ "RepairRatePerSecond", INI::parseReal, nullptr, offsetof( SlavedUpdateModuleData, m_repairRatePerSecond ) },
			{ "RepairWhenBelowHealth%", INI::parseInt, nullptr, offsetof( SlavedUpdateModuleData, m_repairWhenHealthBelowPercentage ) },
			{ "RepairMinReadyTime", INI::parseDurationUnsignedInt, nullptr, offsetof( SlavedUpdateModuleData, m_minReadyFrames ) },
			{ "RepairMaxReadyTime", INI::parseDurationUnsignedInt, nullptr, offsetof( SlavedUpdateModuleData, m_maxReadyFrames ) },
			{ "RepairMinWeldTime",  INI::parseDurationUnsignedInt, nullptr, offsetof( SlavedUpdateModuleData, m_minWeldFrames ) },
			{ "RepairMaxWeldTime",  INI::parseDurationUnsignedInt, nullptr, offsetof( SlavedUpdateModuleData, m_maxWeldFrames ) },
			{ "RepairWeldingSys",		INI::parseAsciiString,	nullptr, offsetof( SlavedUpdateModuleData, m_weldingSysName ) },
			{ "RepairWeldingFXBone", INI::parseAsciiString, nullptr, offsetof( SlavedUpdateModuleData, m_weldingFXBone ) },
			{ "StayOnSameLayerAsMaster", INI::parseBool, nullptr, offsetof( SlavedUpdateModuleData, m_stayOnSameLayerAsMaster ) },
			{ nullptr, nullptr, nullptr, 0 }
		};
    p.add(dataFieldParse);
	}
};

enum RepairStates CPP_11(: Int)
{
	REPAIRSTATE_NONE,
	REPAIRSTATE_UNPACKING,
	REPAIRSTATE_PACKING,
	REPAIRSTATE_READY,
	REPAIRSTATE_EXTENDING,
	REPAIRSTATE_RETRACTING,
	REPAIRSTATE_WELDING,
};

//-------------------------------------------------------------------------------------------------
class SlavedUpdate : public UpdateModule, public SlavedUpdateInterface
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( SlavedUpdate, "SlavedUpdate" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( SlavedUpdate, SlavedUpdateModuleData )

public:

	SlavedUpdate( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	virtual SlavedUpdateInterface* getSlavedUpdateInterface() override { return this; }

	virtual ObjectID getSlaverID() const override { return m_slaver; }
	virtual void onEnslave( const Object *slaver ) override;
	virtual void onSlaverDie( const DamageInfo *info ) override;
	virtual void onSlaverDamage( const DamageInfo *info ) override;
	virtual void onObjectCreated() override;
	virtual Bool isSelfTasking() const override { return FALSE; };


	void doScoutLogic( const Coord3D *mastersDestination );
	void doAttackLogic( const Object *target );
	void doGuardLogic( Coord3D *pinnedPosition );
	void doRepairLogic();
	void endRepair();
	void setRepairState( RepairStates repairState );
	void setRepairModelConditionStates( ModelConditionFlagType flag );
	void moveToNewRepairSpot();

	virtual UpdateSleepTime update() override;	///< Deciding whether or not to make new guys

private:
	void startSlavedEffects( const Object *slaver );	///< We have been marked as Slaved, so we can't be selected or move too far or other stuff
	void stopSlavedEffects();		///< We are no longer slaved.

	ObjectID m_slaver;			///< To whom we are enslaved
	Coord3D m_guardPointOffset;	///< Where we should go when not busy and still enslaved
	Int m_framesToWait;
	RepairStates m_repairState;
	Bool m_repairing;
};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/SlavedUpdate.cpp`
```cpp
// FILE: SlavedUpdate.cpp /////////////////////////////////////////////////////////////////////////
// Author: Matt Campbell, March 2002
// Updated: Kris Morness, July 2002 -- Add support for advanced scout drone abilities
// Desc:  Slaved unit(s) remain close to their master. Used by scout drones, and used by stinger
//        soldiers that are close to a stinger site. It's important to note that any slaved units
//				can use any or all features, some of which are specialized.
///////////////////////////////////////////////////////////////////////////////////////////////////

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/RandomValue.h"
#include "Common/Xfer.h"
#include "Common/Team.h"
#include "Common/MiscAudio.h"
#include "GameClient/Drawable.h"
#include "GameClient/ParticleSys.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/Damage.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Locomotor.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/TerrainLogic.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/SlavedUpdate.h"
#include "GameLogic/Weapon.h"


#define STRAY_MULTIPLIER 2.0f // Multiplier from stating diestance from tunnel, to max distance from
const Real CLOSE_ENOUGH = 15;				// Our moveTo commands and pathfinding can't handle people in the way, so quit trying to hump someone on your spot
const Real CLOSE_ENOUGH_SQR = (CLOSE_ENOUGH * CLOSE_ENOUGH);

//-------------------------------------------------------------------------------------------------
SlavedUpdate::SlavedUpdate( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{
//	const SlavedUpdateModuleData* data = getSlavedUpdateModuleData();

	m_slaver = INVALID_ID;
	m_guardPointOffset.zero();
	m_framesToWait = 0;
	m_repairState = REPAIRSTATE_NONE;
	m_repairing = false;
}

//-------------------------------------------------------------------------------------------------
SlavedUpdate::~SlavedUpdate()
{
}

//-------------------------------------------------------------------------------------------------
void SlavedUpdate::onObjectCreated()
{
	const SlavedUpdateModuleData* data = getSlavedUpdateModuleData();

	if( data->m_repairRatePerSecond > 0.0f )
	{
		//If this object can repair, pack it up at init.
		getObject()->setModelConditionState( MODELCONDITION_PACKING );
	}
}

//-------------------------------------------------------------------------------------------------
void SlavedUpdate::onEnslave( const Object *slaver )
{
	startSlavedEffects( slaver );
}

//-------------------------------------------------------------------------------------------------
void SlavedUpdate::onSlaverDie( const DamageInfo *info )
{
	stopSlavedEffects();
}

//-------------------------------------------------------------------------------------------------
void SlavedUpdate::onSlaverDamage( const DamageInfo *info )
{
	// Only slaves with a ProneUpdate will even care.
	AIUpdateInterface *ai = getObject()->getAIUpdateInterface();
	if( ai )
		ai->aiGoProne( info, CMD_FROM_AI );
}


//-------------------------------------------------------------------------------------------------
UpdateSleepTime SlavedUpdate::update()
{
/// @todo srj use SLEEPY_UPDATE here
	if( m_framesToWait > 0 )
	{
		m_framesToWait--;
	}
	if( m_repairState == REPAIRSTATE_NONE )
	{
		if( m_framesToWait > 0 )
		{
			return UPDATE_SLEEP_NONE;
		}
		m_framesToWait = SLAVED_UPDATE_RATE;
	}

	if( m_slaver == INVALID_ID )
		return UPDATE_SLEEP_NONE;

	const SlavedUpdateModuleData* data = getSlavedUpdateModuleData();
	Object *me = getObject();
	if( !me )
	{
		return UPDATE_SLEEP_NONE;
	}
	AIUpdateInterface *myAI = me->getAIUpdateInterface();
	if( !myAI )
	{
		return UPDATE_SLEEP_NONE;
	}
	Locomotor *locomotor = myAI->getCurLocomotor();
	if( !locomotor )
	{
		return UPDATE_SLEEP_NONE;
	}

	Object *master = TheGameLogic->findObjectByID( m_slaver );
	if( !master || master->isEffectivelyDead() || master->isDisabledByType( DISABLED_UNMANNED ) )
	{
		stopSlavedEffects();

		//Killing is lame.
		//me->kill();

		//Let's disable the drone so it crashes instead!
		//Added special case code in physics falling to ensure death.
		me->setDisabled( DISABLED_UNMANNED );
    if ( me->getAI() )
      me->getAI()->aiIdle( CMD_FROM_AI);

		return UPDATE_SLEEP_NONE;
	}
	else
	{
		Team *masterTeam = master->getTeam();
		Team *myTeam     = me->getTeam();
		if ( masterTeam->getRelationship( myTeam ) != ALLIES )
		{
			//slaver must have been hijacked or something..	// we will join his team
			me->defect( masterTeam, 0 );
		}




	}

	if (data->m_stayOnSameLayerAsMaster)
		me->setLayer(master->getLayer());

	//Clear the drone spotting bonus to the master. Up to the drone
	//to satisfy the conditions to set it again for the next update.
	master->clearWeaponBonusCondition( WEAPONBONUSCONDITION_DRONE_SPOTTING );

	//Get my master's AI. If he is attacking something, grant him a range bonus,
	//and I'll fly over the target.
	Object *target = nullptr;
	AIUpdateInterface *masterAI = master->getAIUpdateInterface();
	if( masterAI )
	{
		target = masterAI->getCurrentVictim();
	}

	//Calculate the health percentage of the master -- there are two places we care.
	//NOTE: Health percentage will always be 100 should the drone be incapable of
	//repairing.
	Int healthPercentage = 100;
	if( data->m_repairRatePerSecond > 0.0f )
	{
		BodyModuleInterface *body = master->getBodyModule();
		if (body)
		{
			Real health = body->getHealth();
			Real maxHealth = body->getMaxHealth();
			healthPercentage = (Int)(WWMath::Div_Safe(health, maxHealth) * 100.0f);
		}
	}

	//Determine whether or not we need to go back to the master to repair him
	if( healthPercentage <= data->m_repairWhenHealthBelowPercentage )
	{
		//1ST PRIORITY: Go to the master's position to repair him because he needs it.
		doRepairLogic();
		return UPDATE_SLEEP_NONE;
	}

	if( data->m_attackRange )
	{
		//2ND PRIORITY: Go to the master's current victim (as close as wander distance allows)
		if( target )
		{
			//At this point, we officially are in an attack mode! Now, simply
			endRepair();
			doAttackLogic( target );
			return UPDATE_SLEEP_NONE;
		}
	}

	if( data->m_scoutRange )
	{
		//3RD PRIORITY: Hover above master's current move destination (as close as wander distance
		//allows).
		if( masterAI->getPath() )
		{
			const Coord3D *masterDest = masterAI->getPath()->getLastNode()->getPosition();

			//Check to see if master is close to the goal position.
			Real distSqr = ThePartitionManager->getDistanceSquared( master, masterDest, FROM_BOUNDINGSPHERE_2D );
			if( distSqr > (data->m_guardMaxRange * 0.5f) * (data->m_guardMaxRange * 0.5f) )
			{
				//If the master's distance to destination is more than half of the guarding range of the slave,
				//then order the slave to scout it.
				endRepair();
				doScoutLogic( masterDest );
				return UPDATE_SLEEP_NONE;
			}
		}
	}

	//NOTE: Health percentage will always be 100 should the drone be incapable of
	//repairing.
	if( healthPercentage < 100 )
	{
		//3RD PRIORITY: Go to the master's position to repair him (because we're idle)
		doRepairLogic();
		return UPDATE_SLEEP_NONE;
	}

	// update our "pinned" location based on where our master is
	Coord3D pinnedPosition = *master->getPosition();
	pinnedPosition.x += m_guardPointOffset.x;
	pinnedPosition.y += m_guardPointOffset.y;
	m_guardPointOffset.z = TheTerrainLogic->getGroundHeight( pinnedPosition.x, pinnedPosition.y );

	if( data->m_guardMaxRange )
	{
		//3RD PRIORITY: Guard the master's area.
		if( myAI->isIdle() && ThePartitionManager->getDistanceSquared(me, &pinnedPosition, FROM_CENTER_3D) > CLOSE_ENOUGH_SQR )
		{
			//I'm idle and too far away.
			endRepair();
			doGuardLogic( &pinnedPosition );
		}
		else if( ThePartitionManager->getDistanceSquared( me, master, FROM_CENTER_3D ) > sqr(STRAY_MULTIPLIER * data->m_guardMaxRange ) )
		{
			//I'm too far away, no matter what I'm doing.
			endRepair();
			doGuardLogic( &pinnedPosition );
		}
	}
	return UPDATE_SLEEP_NONE;
}

//-------------------------------------------------------------------------------------------------
// We are ordered to attempt to get as close as possible to my master's target.
//-------------------------------------------------------------------------------------------------
void SlavedUpdate::doAttackLogic( const Object *target )
{
	const SlavedUpdateModuleData* data = getSlavedUpdateModuleData();
	Object *me = getObject();
	Object *master = TheGameLogic->findObjectByID( m_slaver );
	Coord3D attackPosition;

	//First, determine the attack position. If the target is too far away, then we'll
	//calculate the closest allowable position.
	const Coord3D *targetPos = target->getPosition();
	Real dist = ThePartitionManager->getDistanceSquared( me, targetPos, FROM_BOUNDINGSPHERE_2D );
	if( dist > sqr( data->m_attackRange ) )
	{
		//The distance is too far, so calculate the best allowable position.
		Coord3D vector;
		vector.set( *targetPos );
		vector.sub( *master->getPosition() );
		vector.normalize();
		vector.scale( data->m_attackRange );

		//Now that we have calculated the vector relative to me, add it to my position to get my goal.
		attackPosition.set( *master->getPosition() );
		attackPosition.add( vector );
	}
	else
	{
		//We are close enough, so use the target position -- easy!
		attackPosition.set( *targetPos );
	}

	//Finally, if we have a wander distance, then randomly select a point within
	//the wander range radius of the pinned position, and we'll go there.
	if( data->m_attackWanderRange )
	{
		//Allow me to wander away from the pinnedPosition.
		Real randomDirection = GameLogicRandomValue( 0, 2*PI );
		m_guardPointOffset.zero();
		m_guardPointOffset.x += data->m_attackWanderRange * Cos( randomDirection );
		m_guardPointOffset.y += data->m_attackWanderRange * Sin( randomDirection );

		//Offset our pinned position by our random offset.
		attackPosition.x += m_guardPointOffset.x;
		attackPosition.y += m_guardPointOffset.y;
		m_guardPointOffset.z = TheTerrainLogic->getGroundHeight( attackPosition.x, attackPosition.y );
	}

	//Move to the updated position!
	AIUpdateInterface *ai = me->getAIUpdateInterface();
	if( ai )
	{
		ai->aiMoveToPosition( &attackPosition, CMD_FROM_AI );
	}

	if( dist < sqr( data->m_distToTargetToGrantRangeBonus ) )
	{
		//Finally, seeing we are close enough to the target, grant our
		//master extended weapon range!
		master->setWeaponBonusCondition( WEAPONBONUSCONDITION_DRONE_SPOTTING );
	}
}

//-------------------------------------------------------------------------------------------------
// We are ordered to attempt to get as close as possible to my master's movement destination point.
//-------------------------------------------------------------------------------------------------
void SlavedUpdate::doScoutLogic( const Coord3D *mastersDestination )
{
	const SlavedUpdateModuleData* data = getSlavedUpdateModuleData();
	Object *me = getObject();
	Object *master = TheGameLogic->findObjectByID( m_slaver );
	Coord3D scoutPosition;

	//First, determine the scout position. If our master's destination is too far away, then we'll
	//calculate the closest allowable position.
	Real dist = ThePartitionManager->getDistanceSquared( me, mastersDestination, FROM_BOUNDINGSPHERE_2D );
	if( dist > sqr( data->m_scoutRange ) )
	{
		//The distance is too far, so calculate the best allowable position.
		Coord3D vector;
		vector.set( *mastersDestination );
		vector.sub( *master->getPosition() );
		vector.normalize();
		vector.scale( data->m_scoutRange );

		//Now that we have calculated the vector relative to me, add it to my position to get my goal.
		scoutPosition.set( *master->getPosition() );
		scoutPosition.add( vector );
	}
	else
	{
		//We are close enough, so use the target position -- easy!
		scoutPosition.set( *mastersDestination );
	}

	//Finally, if we have a wander distance, then randomly select a point within
	//the wander range radius of the pinned position, and we'll go there.
	if( data->m_scoutWanderRange )
	{
		//Allow me to wander away from the pinnedPosition.
		Real randomDirection = GameLogicRandomValue( 0, 2*PI );
		m_guardPointOffset.zero();
		m_guardPointOffset.x += data->m_scoutWanderRange * Cos( randomDirection );
		m_guardPointOffset.y += data->m_scoutWanderRange * Sin( randomDirection );

		//Offset our pinned position by our random offset.
		scoutPosition.x += m_guardPointOffset.x;
		scoutPosition.y += m_guardPointOffset.y;
		m_guardPointOffset.z = TheTerrainLogic->getGroundHeight( scoutPosition.x, scoutPosition.y );
	}

	//Move to the updated position!
	AIUpdateInterface *ai = me->getAIUpdateInterface();
	if( ai )
	{
		ai->aiMoveToPosition( &scoutPosition, CMD_FROM_AI );
	}
}

//-------------------------------------------------------------------------------------------------
// We are ordered to attempt to get as close as possible to my master's position.
//-------------------------------------------------------------------------------------------------
void SlavedUpdate::doGuardLogic( Coord3D *pinnedPosition )
{
	const SlavedUpdateModuleData* data = getSlavedUpdateModuleData();
	Object *me = getObject();

	if( data->m_guardWanderRange )
	{
		// recalc where we want to be if we wander around
		Real randomDirection = GameLogicRandomValue( 0, 2*PI );
		m_guardPointOffset.zero();
		m_guardPointOffset.x += data->m_guardMaxRange * Cos( randomDirection );
		m_guardPointOffset.y += data->m_guardMaxRange * Sin( randomDirection );

		pinnedPosition->x += m_guardPointOffset.x;
		pinnedPosition->y += m_guardPointOffset.y;
		m_guardPointOffset.z = TheTerrainLogic->getGroundHeight( pinnedPosition->x, pinnedPosition->y );
	}
	AIUpdateInterface *ai = me->getAIUpdateInterface();
	if( ai )
	{
		ai->aiMoveToPosition( pinnedPosition, CMD_FROM_AI );
	}
}

//-------------------------------------------------------------------------------------------------
// We are ordered to repair our master
//-------------------------------------------------------------------------------------------------
void SlavedUpdate::doRepairLogic()
{
	Object *me = getObject();
	Object *master = TheGameLogic->findObjectByID( m_slaver );
	const SlavedUpdateModuleData* data = getSlavedUpdateModuleData();
	AIUpdateInterface *ai = me->getAIUpdateInterface();
	if( !ai )
	{
		return;
	}

	//There are two major things... either move closer or repair.
	Real distSqr = ThePartitionManager->getDistanceSquared( me, master, FROM_BOUNDINGSPHERE_2D );
	Bool closeEnough = distSqr < 12.0f * 12.0f;

	//We're going to do different things based on the repair state.
	if( closeEnough )
	{
		//Now drive the repairing states.
		switch( m_repairState )
		{
			case REPAIRSTATE_NONE:
				setRepairState( REPAIRSTATE_READY );
				break;
			case REPAIRSTATE_READY:
			case REPAIRSTATE_EXTENDING:
				if( m_framesToWait == 0 )
				{
					setRepairState( REPAIRSTATE_WELDING );
				}
				break;
			case REPAIRSTATE_UNPACKING:
			case REPAIRSTATE_WELDING:
			case REPAIRSTATE_RETRACTING:
				if( m_framesToWait == 0 )
				{
					setRepairState( REPAIRSTATE_READY );
				}
				break;
		}
	}
	else
	{
		m_repairing = false;

		Bool closeEnoughForZPrecision = distSqr < sqr(master->getGeometryInfo().getBoundingSphereRadius() * 2);

		//We're too far away to repair, so get closer.
		Locomotor *locomotor = ai->getCurLocomotor();
		if( locomotor )
		{
			locomotor->setUsePreciseZPos( closeEnoughForZPrecision );
		}
		Coord3D pos;
		pos.set( *master->getPosition() );
		Real altitude = GameLogicRandomValueReal( data->m_repairMinAltitude, data->m_repairMaxAltitude );
		pos.z += altitude;
		ai->aiMoveToPosition( &pos, CMD_FROM_AI );

		//Also speed things up by retracting the repair arm (so viewers
		//will see it's intentions)
		if( m_framesToWait == 0 )
		{
			setRepairState( REPAIRSTATE_READY );
		}
	}

	if( closeEnough && m_repairing )
	{
		//We're close enough to repair.
		BodyModuleInterface *body = master->getBodyModule();
		if( body )
		{
			//Calculate the repair rate per frame.
			Real repairAmount = data->m_repairRatePerSecond / static_cast<float>(LOGICFRAMES_PER_SECOND);

			DamageInfo healingInfo;
			healingInfo.in.m_amount = repairAmount;
			healingInfo.in.m_damageType = DAMAGE_HEALING;
			healingInfo.in.m_deathType = DEATH_NONE;
			body->attemptHealing( &healingInfo );
		}
	}
}

//-------------------------------------------------------------------------------------------------
void SlavedUpdate::endRepair()
{
	if( m_repairState != REPAIRSTATE_NONE )
	{
		//Remove the state so we can start using the update timer again.
		m_repairState = REPAIRSTATE_NONE;
		m_framesToWait = SLAVED_UPDATE_RATE;
		m_repairing = false;

		//Set the packing state and clear any other states (to force the arm to fully retract)
		setRepairModelConditionStates( MODELCONDITION_PACKING );
	}

	AIUpdateInterface* ai = getObject()->getAIUpdateInterface();
	if( ai )
	{
		ai->chooseLocomotorSet(LOCOMOTORSET_NORMAL);
		Locomotor* locomotor = ai->getCurLocomotor();
		if (locomotor)
		{
			locomotor->setUltraAccurate(false);
			locomotor->setUsePreciseZPos(false);
		}
	}
}

//-------------------------------------------------------------------------------------------------
void SlavedUpdate::setRepairModelConditionStates( ModelConditionFlagType flag )
{
	Object *obj = getObject();
	obj->clearModelConditionState( MODELCONDITION_PACKING );
	obj->clearModelConditionState( MODELCONDITION_UNPACKING );
	obj->clearModelConditionState( MODELCONDITION_FIRING_B );
	obj->clearModelConditionState( MODELCONDITION_FIRING_C );
	obj->clearModelConditionState( MODELCONDITION_BETWEEN_FIRING_SHOTS_B );
	obj->clearModelConditionState( MODELCONDITION_BETWEEN_FIRING_SHOTS_C );
	obj->clearModelConditionState( MODELCONDITION_RELOADING_B );
	obj->clearModelConditionState( MODELCONDITION_RELOADING_C );
	obj->setModelConditionState( flag );
}

//-------------------------------------------------------------------------------------------------
void SlavedUpdate::setRepairState( RepairStates repairState )
{
	Object *obj = getObject();
	Drawable *draw = obj->getDrawable();
	//Object *master = TheGameLogic->findObjectByID( m_slaver );
	const SlavedUpdateModuleData* data = getSlavedUpdateModuleData();

	if( repairState == m_repairState )
	{
		return;
	}

	switch( repairState )
	{
		case REPAIRSTATE_UNPACKING:
			setRepairModelConditionStates( MODELCONDITION_UNPACKING );
			m_framesToWait = 15;
			break;
		case REPAIRSTATE_PACKING:
			setRepairModelConditionStates( MODELCONDITION_PACKING );
			m_framesToWait = 15;
			break;
		case REPAIRSTATE_READY:
		{
			switch( m_repairState )
			{
				case REPAIRSTATE_NONE:
					//We're not in a repair state -- so assume we need to unpack
					setRepairModelConditionStates( MODELCONDITION_UNPACKING );
					m_repairState = REPAIRSTATE_UNPACKING;
					m_framesToWait = 15;
					break;
				case REPAIRSTATE_WELDING:
					//We are welding so retract before going into ready state.
					m_repairState = REPAIRSTATE_RETRACTING;
					m_framesToWait = 5;
					setRepairModelConditionStates( MODELCONDITION_FIRING_C );
					moveToNewRepairSpot();
					break;
				default:
					m_repairState = REPAIRSTATE_READY;
					m_framesToWait = GameLogicRandomValue( data->m_minReadyFrames, data->m_maxReadyFrames );
					break;
			}
			break;
		}
		case REPAIRSTATE_WELDING:
		{
			if( m_repairState == REPAIRSTATE_READY )
			{
				m_repairState = REPAIRSTATE_EXTENDING;
				m_framesToWait = 5;
				setRepairModelConditionStates( MODELCONDITION_FIRING_B );
				break;
			}
			else
			{
				m_repairState = REPAIRSTATE_WELDING;
				m_framesToWait = GameLogicRandomValue( data->m_minWeldFrames, data->m_maxWeldFrames );

				//Make sparks!
				if( !data->m_weldingSysName.isEmpty() )
				{
					const ParticleSystemTemplate *tmp = TheParticleSystemManager->findTemplate( data->m_weldingSysName );
					if( tmp )
					{
						ParticleSystem *weldingSys = TheParticleSystemManager->createParticleSystem(tmp);
						if( weldingSys )
						{
							Coord3D pos;
							//Get the bone position
							if( draw->getPristineBonePositions( data->m_weldingFXBone.str(), 0, &pos, nullptr, 1 ) )
							{
								pos.add( *obj->getPosition() );
							}
							else
							{
								pos.set( *obj->getPosition() );
							}

							weldingSys->setPosition( &pos );
							Real time = (Real)(m_framesToWait * static_cast<float>(LOGICFRAMES_PER_SECOND));
							weldingSys->setLifetimeRange( time, time );

							AudioEventRTS soundToPlay = TheAudio->getMiscAudio()->m_repairSparks;
							soundToPlay.setPosition( &pos );
							TheAudio->addAudioEvent( &soundToPlay );
						}
					}
				}

				if( !m_repairing )
				{
					//Don't actually start healing until we fire our first sparks
					m_repairing = true;
					AIUpdateInterface *ai = obj->getAIUpdateInterface();
					if( ai )
					{
						//ai->aiFaceObject( master, CMD_FROM_AI );
					}
				}
				break;
			}
			break;
		}
	}
}

//-------------------------------------------------------------------------------------------------
void SlavedUpdate::moveToNewRepairSpot()
{
	const SlavedUpdateModuleData* data = getSlavedUpdateModuleData();
	Object *me = getObject();
	Object *master = TheGameLogic->findObjectByID( m_slaver );

	//Finally, if we have a wander distance, then randomly select a point within
	//the wander range radius of the pinned position, and we'll go there.
	if( data->m_repairRange )
	{
		//Allow me to wander away from the pinnedPosition.
		Real randomDirection = GameLogicRandomValue( 0, 2*PI );
		m_guardPointOffset.set( *master->getPosition() );
		m_guardPointOffset.x += data->m_repairRange * Cos( randomDirection );
		m_guardPointOffset.y += data->m_repairRange * Sin( randomDirection );
		m_guardPointOffset.z = TheTerrainLogic->getGroundHeight( m_guardPointOffset.x, m_guardPointOffset.y );
		Real altitude = GameLogicRandomValueReal( data->m_repairMinAltitude, data->m_repairMaxAltitude );
		m_guardPointOffset.z += altitude;

		AIUpdateInterface *ai = me->getAIUpdateInterface();
		if( ai )
		{
			ai->chooseLocomotorSet( LOCOMOTORSET_PANIC );
			ai->getCurLocomotor()->setUltraAccurate( true );
			ai->aiMoveToPosition( &m_guardPointOffset, CMD_FROM_AI );

			Locomotor *locomotor = ai->getCurLocomotor();
			if( locomotor )
			{
				locomotor->setUsePreciseZPos( true );
			}
		}
	}
}

//-------------------------------------------------------------------------------------------------
void SlavedUpdate::startSlavedEffects( const Object *slaver )
{
	if( slaver == nullptr )
		return;

	m_slaver = slaver->getID();
	const SlavedUpdateModuleData* data = getSlavedUpdateModuleData();

	// Decide where our pinned stray point is
	Real randomDirection = GameLogicRandomValue( 0, 2*PI );
	m_guardPointOffset.zero();
	m_guardPointOffset.x += data->m_guardMaxRange * Cos( randomDirection );
	m_guardPointOffset.y += data->m_guardMaxRange * Sin( randomDirection );

	// mark selves as not selectable
	getObject()->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_UNSELECTABLE ) );


  if ( slaver->testStatus( OBJECT_STATUS_STEALTHED ) )
  {
    StealthUpdate *myStealth = getObject()->getStealth();
    if ( myStealth )
    {
      myStealth->receiveGrant( true );
      // note to anyone... once stealth is granted to this drone(or such)
      // let its own stealthupdate govern the allowedtostealth cases
    }
  }


}

//-------------------------------------------------------------------------------------------------
void SlavedUpdate::stopSlavedEffects()
{
	m_slaver = INVALID_ID;
	m_guardPointOffset.zero();

	/// @todo Just a thought.  Our Status bits on objects really need to be reference counts so you don't clear someone else's flag
	getObject()->clearStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_UNSELECTABLE ) );
	getObject()->clearDisabled( DISABLED_HELD );
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void SlavedUpdate::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void SlavedUpdate::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// slaver
	xfer->xferObjectID( &m_slaver );

	// guard point offset
	xfer->xferCoord3D( &m_guardPointOffset);

	// frames to wait
	xfer->xferInt( &m_framesToWait );

	// repair state
	xfer->xferUser( &m_repairState, sizeof( RepairStates ) );

	// repairing
	xfer->xferBool( &m_repairing );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void SlavedUpdate::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
