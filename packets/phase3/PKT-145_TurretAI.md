# Translation Packet `PKT-145`: `TurretAI`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-145`
- **Work Package:** `WP-3.3` (Phase 3)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 2: Handle Component
- **Allowed Rust Output File:** `crates/zh_logic/src/combat/turret_ai.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (16):**
  - `Common/GameAudio.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameAudio.h`
  - `Common/GameMemory.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameMemory.h`
  - `Common/PerfTimer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/PerfTimer.h`
  - `Common/RandomValue.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/RandomValue.h`
  - `Common/StateMachine.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/StateMachine.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `GameLogic/TerrainLogic.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/TerrainLogic.h`
  - `GameLogic/TurretAI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/TurretAI.h`
  - `GameLogic/Weapon.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h`
  - `GameLogic/WeaponSet.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/WeaponSet.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (19):**
  - `"TurretTurnRate" => INI::parseAngularVelocityReal`
  - `"TurretPitchRate" => INI::parseAngularVelocityReal`
  - `"NaturalTurretAngle" => INI::parseAngleReal`
  - `"NaturalTurretPitch" => INI::parseAngleReal`
  - `"FirePitch" => INI::parseAngleReal`
  - `"MinPhysicalPitch" => INI::parseAngleReal`
  - `"GroundUnitPitch" => INI::parseAngleReal`
  - `"TurretFireAngleSweep" => TurretAIData::parseTurretSweep`
  - `"TurretSweepSpeedModifier" => TurretAIData::parseTurretSweepSpeed`
  - `"ControlledWeaponSlots" => parseTWS`
  - `"AllowsPitch" => INI::parseBool`
  - `"InterTurretDelay" => INI::parseDurationUnsignedInt`
  - `"MinIdleScanAngle" => INI::parseAngleReal`
  - `"MaxIdleScanAngle" => INI::parseAngleReal`
  - `"MinIdleScanInterval" => INI::parseDurationUnsignedInt`
  - `"MaxIdleScanInterval" => INI::parseDurationUnsignedInt`
  - `"RecenterTime" => INI::parseDurationUnsignedInt`
  - `"InitiallyDisabled" => INI::parseBool`
  - `"FiresWhileTurning" => INI::parseBool`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (18 calls — preserve order identically):**
  1. `xferVersion( &v, cv )`
  2. `xferVersion( &v, cv )`
  3. `xferVersion( &version, currentVersion )`
  4. `xferVersion( &version, currentVersion )`
  5. `xferSnapshot(m_turretStateMachine)`
  6. `xferReal(&m_angle)`
  7. `xferReal(&m_pitch)`
  8. `xferUnsignedInt(&m_enableSweepUntil)`
  9. `xferUser(&m_target, sizeof(m_target))`
  10. `xferUnsignedInt(&m_continuousFireExpirationFrame)`
  11. `xferBool(&tmpBool)`
  12. `xferUnsignedInt(&m_sleepUntil)`
  13. `xferVersion( &version, currentVersion )`
  14. `xferUnsignedInt(&m_nextIdleScan)`
  15. `xferVersion( &version, currentVersion )`
  16. `xferReal(&m_desiredAngle)`
  17. `xferVersion( &version, currentVersion )`
  18. `xferUnsignedInt(&m_timestamp)`
- **PRNG Calls (3 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValue(getTurretAI()->getMinIdleScanInterval(), getTurretAI()->getMaxIdleScanInterval())`
  - `GameLogicRandomValueReal(0, maxA - minA)`
  - `GameLogicRandomValue( 0, 1 ) == 0)`
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if RETAIL_COMPATIBLE_XFER_SAVE`

## 2.5 Existing Rust Symbol Index (Use These Exact Paths)
- `zh_logic::combat::armor`: `ArmorTemplate`, `Armor`, `fn`, `ArmorStore`
- `zh_logic::combat::armor_set`: `ArmorTemplateSet`
- `zh_logic::combat::damage`: `PlayerMaskType`, `HUGE_DAMAGE_AMOUNT`, `DamageInfoInput`, `DamageInfoOutput`, `DamageInfo`
- `zh_logic::combat::locomotor`: `BIGNUM`, `LocomotorAppearance`, `fn`, `LOCOMOTOR_APPEARANCE_NAMES`, `LocomotorPriority`, `LOCOMOTOR_PRIORITY_NAMES`, `LocomotorBehaviorZ`, `LOCOMOTOR_BEHAVIOR_Z_NAMES`, `LocomotorTemplate`, `LocomotorStore`, `Locomotor`
- `zh_logic::combat::locomotor_set`: `LOCOMOTORSET_COUNT`, `LocomotorSetType`, `fn`, `LOCOMOTOR_SET_NAMES`, `LocomotorSet`
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
- `zh_logic::arena`: `ObjectSlot`, `INVALID`, `fn`, `ModuleSlot`, `ModuleDataId`, `ThingTemplateId`, `FacetMask`, `NONE`, `UPDATE`, `DIE`, `DAMAGE`, `CREATE`, `COLLIDE`, `BODY`, ... (+34)
- `zh_logic::modules::module`: `ModuleData`, `StubModuleData`, `ModuleSnapshotHeader`, `BehaviorModule`, `UpdateFacet`, `BodyFacet`, `DieFacet`, `DamageFacet`, `CreateFacet`, `CollideFacet`, `ContainFacet`, `UpgradeFacet`, `SpecialPowerFacet`, `DestroyFacet`, ... (+3)
- `zh_logic::modules::module_factory`: `ModuleDataParseFn`, `ModuleCreateFn`, `BuiltinModuleEntry`, `BUILTIN_ZH_MODULES`, `W3D_DRAW_STUB_NAMES`, `ModuleDataStore`, `ModuleDescriptor`, `ModuleRegistry`
- `zh_logic::object::experience_tracker`: `ExperienceParentAccess`, `ExperienceTrackerState`, `fn`
- `zh_logic::object::object_script_status_bits`: `ObjectScriptStatusBit`, `fn`, `OBJECT_STATUS_SCRIPT_DISABLED`, `OBJECT_STATUS_SCRIPT_UNPOWERED`, `OBJECT_STATUS_SCRIPT_UNSELLABLE`, `OBJECT_STATUS_SCRIPT_UNSTEALTHED`, `OBJECT_STATUS_SCRIPT_TARGETABLE`
- `zh_logic::object::object_types`: `ObjectTypes`
- `zh_logic::object::overridable`: `OverridableEntry`, `fn`
- `zh_logic::object::override`: `OverrideStore`, `Override`, `fn`
- `zh_logic::object::thing`: `INITIAL_THING_POS_X`, `INITIAL_THING_POS_Y`, `ThingTerrainAccess`, `ThingTransformReactor`, `NoopTransformReactor`, `Thing`, `fn`
- `zh_logic::object::thing_factory`: `ThingFactory`, `fn`
- `zh_logic::object::thing_template`: `LEVEL_COUNT`, `MAX_UPGRADE_CAMEO_UPGRADES`, `USE_EXP_VALUE_FOR_SKILL_VALUE`, `RADAR_PRIORITY_NAMES`, `MODULEINTERFACE_UPDATE`, `MODULEINTERFACE_DIE`, `MODULEINTERFACE_DAMAGE`, `MODULEINTERFACE_CREATE`, `MODULEINTERFACE_COLLIDE`, `MODULEINTERFACE_BODY`, `MODULEINTERFACE_CONTAIN`, `MODULEINTERFACE_UPGRADE`, `MODULEINTERFACE_SPECIAL_POWER`, `MODULEINTERFACE_DESTROY`, ... (+27)
- `zh_logic::object::weapon`: `NO_MAX_SHOTS_LIMIT`, `PATHFIND_CELL_SIZE_F`, `WEAPON_UNDERSIZE`, `WeaponReloadType`, `fn`, `WeaponPrefireType`, `WEAPON_RELOAD_NAMES`, `WEAPON_PREFIRE_NAMES`, `WEAPON_ANTI_AIRBORNE_VEHICLE`, `WEAPON_ANTI_GROUND`, `WEAPON_ANTI_PROJECTILE`, `WEAPON_ANTI_SMALL_MISSILE`, `WEAPON_ANTI_MINE`, `WEAPON_ANTI_AIRBORNE_INFANTRY`, ... (+22)
- `zh_logic::object::weapon_set`: `WEAPONSLOT_COUNT`, `WeaponSlotType`, `fn`, `WeaponSetConditionType`, `WeaponChoiceCriteria`, `WeaponLockType`, `CanAttackResult`, `WeaponTemplateSet`, `WEAPON_SET_TYPE_TO_MODEL_CONDITION_TYPE_MAP`, `WeaponSet`
- `zh_logic::rules`: `AcademyClassificationType`, `fn`, `ACADEMY_CLASSIFICATION_TYPE_NAMES`
- `zh_logic::rules::combat_types`: `ARMORSET_COUNT`, `ArmorSetFlags`, `ARMOR_SET_NAMES`, `ArmorSetType`, `fn`, `DAMAGE_NUM_TYPES`, `DamageTypeFlags`, `DAMAGE_TYPE_NAMES`, `DamageType`, `DEATH_NUM_TYPES`, `DeathTypeFlags`, `DEATH_TYPE_FLAGS_ALL`, `DEATH_TYPE_FLAGS_NONE`, `DEATH_TYPE_NAMES`, ... (+14)
- `zh_logic::rules::disabled_types`: `DISABLED_COUNT`, `DISABLED_ANY_SENTINEL`, `DisabledMaskType`, `DisabledType`, `fn`, `DISABLED_TYPE_NAMES`
- `zh_logic::rules::kind_of`: `KINDOF_COUNT`, `KindOfMaskType`, `KindOfType`, `fn`, `KINDOF_NAMES`
- `zh_logic::rules::model_state`: `MODELCONDITION_COUNT`, `ModelConditionFlags`, `MODEL_CONDITION_NAMES`, `ModelConditionFlagType`, `fn`
- `zh_logic::rules::multiplayer_settings`: `Color`, `PLAYERTEMPLATE_RANDOM`, `PLAYERTEMPLATE_OBSERVER`, `PLAYERTEMPLATE_MIN`, `GAME_COLOR_UNDEFINED`, `MultiplayerColorDefinition`, `MultiplayerColorList`, `MultiplayerStartingMoneyList`, `MultiplayerSettings`
- `zh_logic::rules::object_status_types`: `OBJECT_STATUS_COUNT`, `ObjectStatusMaskType`, `ObjectStatusType`, `fn`, `OBJECT_STATUS_NAMES`
- `zh_logic::rules::player_template`: `MAX_MP_STARTING_UNITS`, `VETERANCY_NAMES`, `ProductionChangeMap`, `ProductionVeterancyMap`, `PlayerTemplateImageLookup`, `PlayerTemplate`, `PlayerTemplateStore`
- `zh_logic::rules::production_prerequisite`: `UNIT_OR_WITH_PREV`, `MAX_PREREQ`, `ThingTemplateRef`, `ThingTemplateRegistry`, `PrereqPlayerView`, `PrereqTextLookup`, `ProductionPrerequisite`
- `zh_logic::rules::rank_info`: `RankInfoData`, `RankInfoStore`
- `zh_logic::rules::science`: `ScienceType`, `SCIENCE_INVALID`, `ScienceVec`, `SciencePlayerAccess`, `ScienceTextLookup`, `ScienceInfoData`, `ScienceStore`
- `zh_logic::rules::special_power`: `DEFAULT_DEFECTION_DETECTION_PROTECTION_TIME_LIMIT`, `SpecialPowerObjectAccess`, `SpecialPowerTemplateData`, `SpecialPowerTemplate`, `fn`, `SpecialPowerStore`
- `zh_logic::rules::special_power_types`: `SPECIALPOWER_COUNT`, `SpecialPowerMaskType`, `SpecialPowerType`, `fn`, `SPECIAL_POWER_NAMES`
- `zh_logic::rules::upgrade`: `UPGRADE_MAX_COUNT`, `UpgradeMaskType`, `UpgradeStatusType`, `fn`, `UpgradeType`, `UPGRADE_TYPE_NAMES`, `UpgradePlayerAccess`, `Upgrade`, `UpgradeTemplate`, `UpgradeCenter`
- `zh_logic::scheduler::sleepy_heap`: `UPDATE_SLEEP_INVALID`, `UPDATE_SLEEP_NONE`, `UPDATE_SLEEP_FOREVER`, `INVALID_LOGIC_INDEX`, `MAX_SLEEPY_UPDATES_PER_OBJECT`, `SleepyUpdatePhase`, `fn`, `UpdateModuleScheduleState`, `ScheduleStore`, `SleepyUpdateHeap`
- `zh_logic::sim`: `ClientSink`, `NoopClientSink`, `FlatTerrain`, `RulesRegistry`, `ObjectRecord`, `SimView`, `SimApi`, `SimWorld`, `SimContext`, `MockSimApi`
- `zh_logic::economy`: `MAX_PLAYER_COUNT`
- `zh_logic::economy::academy_stats`: `MAX_ADVICE_TIPS`, `FRAMES_BETWEEN_UPDATES`, `DEFAULT_SUPPLY_CENTER_COST`, `KINDOF_DOZER`, `KINDOF_FS_SUPPLY_CENTER`, `KINDOF_COMMANDCENTER`, `KINDOF_INFANTRY`, `KINDOF_VEHICLE`, `KINDOF_HARVESTER`, `KINDOF_HERO`, `KINDOF_FS_STRATEGY_CENTER`, `KINDOF_MONEY_HACKER`, `KINDOF_FS_BLACK_MARKET`, `KINDOF_FS_SUPPLY_DROPZONE`, ... (+9)
- `zh_logic::economy::energy`: `EnergyOwnerCallbacks`, `NoopEnergyCallbacks`, `EnergyObjectInfo`, `Energy`
- `zh_logic::economy::handicap`: `HANDICAP_TYPE_COUNT`, `THING_TYPE_COUNT`, `HandicapType`, `ThingType`, `Handicap`, `fn`
- `zh_logic::economy::mission_stats`: `MissionStats`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/TurretAI.h`
```cpp
// TurretAI.h
// Author: Steven Johnson, April 2002
// Simple turret behavior

#pragma once

#include "Common/StateMachine.h"
#include "Common/GameMemory.h"

const Real DEFAULT_TURN_RATE = 0.01f;
const Real DEFAULT_PITCH_RATE = 0.01f;

/**
 * The TurretAI state IDs.
 * Each of these constants will be associated with an instance of a State class
 * in a given StateMachine.
 */
enum TurretStateType CPP_11(: Int)
{
	TURRETAI_IDLE,
	TURRETAI_IDLESCAN,
	TURRETAI_AIM,															///< aim turret at GoalObject
	TURRETAI_FIRE,														///< fire turret at GoalObject
	TURRETAI_RECENTER,												///< rotate turret back to default position
	TURRETAI_HOLD,														///< hold turret position for a bit before recenter

	NUM_TURRETAI_STATES
};

// FORWARD DECLARATIONS ///////////////////////////////////////////////////////////////////////////
class Weapon;
class TurretAI;

//-----------------------------------------------------------------------------------------------------------
/**
 * The AI state machine.  This is used by AIUpdate to implement all of the
 * commands in the AICommandInterface.
 */
class TurretStateMachine : public StateMachine
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( TurretStateMachine, "TurretStateMachine" );

public:
	/**
	 * The implementation of this constructor defines the states
	 * used by this machine.
	 */
	TurretStateMachine( TurretAI* tai, Object* owner, AsciiString name );

	TurretAI* getTurretAI() const { return m_turretAI; }

	virtual void clear() override;
	virtual StateReturnType resetToDefaultState() override;
	virtual StateReturnType setState( StateID newStateID ) override;

private:
	TurretAI* m_turretAI;

protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;
};

//-----------------------------------------------------------------------------------------------------------
class TurretState : public State
{
	MEMORY_POOL_GLUE_ABC(TurretState)
protected:
	TurretState( TurretStateMachine* machine, AsciiString name ) : State( machine, name) { }
	TurretAI* getTurretAI() { return ((TurretStateMachine*)getMachine())->getTurretAI(); }
};
EMPTY_DTOR(TurretState)

//-----------------------------------------------------------------------------------------------------------
class TurretAIIdleState : public TurretState
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(TurretAIIdleState, "TurretAIIdleState")
public:
	TurretAIIdleState( TurretStateMachine* machine ) : TurretState( machine, "TurretAIIdleState"), m_nextIdleScan(0) { }
	virtual StateReturnType onEnter() override;
	virtual StateReturnType update() override;

protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;
private:
	void resetIdleScan();
	UnsignedInt m_nextIdleScan;
};
EMPTY_DTOR(TurretAIIdleState)

//-----------------------------------------------------------------------------------------------------------
class TurretAIIdleScanState : public TurretState
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(TurretAIIdleScanState, "TurretAIIdleScanState")
public:
	TurretAIIdleScanState( TurretStateMachine* machine ) : TurretState( machine, "TurretAIIdleScanState"), m_desiredAngle(0) { }
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;

protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;
private:
	Real m_desiredAngle;
};
EMPTY_DTOR(TurretAIIdleScanState)

//-----------------------------------------------------------------------------------------------------------
/**
 * Aim a turret at GoalObject
 */
class TurretAIAimTurretState : public TurretState
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(TurretAIAimTurretState, "TurretAIAimTurretState")
private:
#ifdef INTER_TURRET_DELAY
	UnsignedInt m_extraDelay;
#endif
public:
	TurretAIAimTurretState( TurretStateMachine* machine ) : TurretState( machine, "TurretAIAimTurretState" )
	{

	}
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;
protected:
	// snapshot interface	STUBBED - no member vars to save. jba.
	virtual void crc( Xfer *xfer ) override {};
	virtual void xfer( Xfer *xfer ) override {XferVersion cv = 1;	XferVersion v = cv; xfer->xferVersion( &v, cv );}
	virtual void loadPostProcess() override {};
};
EMPTY_DTOR(TurretAIAimTurretState)

//-----------------------------------------------------------------------------------------------------------
/**
 * Rotate a turret back to its default position
 */
class TurretAIRecenterTurretState : public TurretState
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(TurretAIRecenterTurretState, "TurretAIRecenterTurretState")
public:
	TurretAIRecenterTurretState( TurretStateMachine* machine ) : TurretState( machine, "TurretAIRecenterTurretState" ) { }
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;
protected:
	// snapshot interface	STUBBED - no member vars to save. jba.
	virtual void crc( Xfer *xfer ) override {};
	virtual void xfer( Xfer *xfer ) override {XferVersion cv = 1;	XferVersion v = cv; xfer->xferVersion( &v, cv );}
	virtual void loadPostProcess() override {};
};
EMPTY_DTOR(TurretAIRecenterTurretState)

//-----------------------------------------------------------------------------------------------------------
/**
 * Hold turret position for a bit before recenter
 */
class TurretAIHoldTurretState : public TurretState
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(TurretAIHoldTurretState, "TurretAIHoldTurretState")
private:
	UnsignedInt m_timestamp;										///< frame this state was last entered
public:
	TurretAIHoldTurretState( TurretStateMachine* machine ) : TurretState( machine , "AIHoldTurretState")
	{
		m_timestamp = 0;
	}
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;
protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;
};
EMPTY_DTOR(TurretAIHoldTurretState)

//-------------------------------------------------------------------------------------------------
class TurretAIData : public MemoryPoolObject
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(TurretAIData, "TurretAIData")
public:
	Real						m_turnRate;
	Real						m_pitchRate;
	Real						m_naturalTurretAngle;
	Real						m_naturalTurretPitch;
	Real						m_turretFireAngleSweep[WEAPONSLOT_COUNT];	///< if nonzero, sweep within +/- this angle range while firing
	Real						m_turretSweepSpeedModifier[WEAPONSLOT_COUNT];	///< While sweeping, change your speed by this
	Real						m_firePitch;						///< if nonzero, I am on target at this fixed pitch, not when pointing at target
	Real						m_minPitch;							///< dependent on allowspitch. defaults to 0 (horizontal). The lowest pitch allowed (negative to allow pointing down of a high turret)
	Real						m_groundUnitPitch;			///< dependent on allowspitch. defaults to 0 (horizontal). The lowest pitch allowed when firing at ground units to give the weapon an arc.  jba
	UnsignedInt			m_turretWeaponSlots;		///< which WeaponSlots are controlled by this turret
#ifdef INTER_TURRET_DELAY
	UnsignedInt			m_interTurretDelay;			///< special-case for multiturret battleships
#endif
	Real						m_minIdleScanAngle;			///< max angle the turret can turn while idling
	Real						m_maxIdleScanAngle;			///< max angle the turret can turn while idling
	UnsignedInt			m_minIdleScanInterval;	///< min interval between idle scans
	UnsignedInt			m_maxIdleScanInterval;	///< max interval between idle scans
	UnsignedInt			m_recenterTime;					///< time to wait before recentering turret
	Bool						m_initiallyDisabled;		///< manually controlled and disabled.
	Bool						m_firesWhileTurning;    ///< so the firing state does not instantly expire the turning state
	Bool						m_isAllowsPitch;				///< This type of turret can pitch up and down as well as spin

	TurretAIData();
	static void buildFieldParse(MultiIniFieldParse& p);

	static void parseTurretSweep(INI* ini, void *instance, void *store, const void* userData);
	static void parseTurretSweepSpeed(INI* ini, void *instance, void *store, const void* userData);
};
EMPTY_DTOR(TurretAIData)

//-----------------------------------------------------------------------------------------------------
enum TurretTargetType CPP_11(: Int)
{
	TARGET_NONE,
	TARGET_OBJECT,
	TARGET_POSITION
};

//-----------------------------------------------------------------------------------------------------
/**
 * Turret behavior implementation.
 */
class TurretAI : public MemoryPoolObject, public Snapshot, public NotifyWeaponFiredInterface
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( TurretAI, "TurretAI" )

public:
	TurretAI(Object* owner, const TurretAIData* data, WhichTurretType tur);
	// virtual destructor prototype provided by memory pool declaration

	Real getTurretAngle() const { return m_angle; }
	Real getTurretPitch() const { return m_pitch; }
	Real getMinPitch() const { return m_data->m_minPitch; }
	Bool isAllowsPitch() const { return m_data->m_isAllowsPitch; }
	Real getTurnRate() const { return m_data->m_turnRate; }
	Real getNaturalTurretAngle() const { return m_data->m_naturalTurretAngle; }
	Real getPitchRate() const { return m_data->m_pitchRate; }
	Real getFirePitch() const { return m_data->m_firePitch; }
	Real getGroundUnitPitch() const { return m_data->m_groundUnitPitch; }
	Real getNaturalTurretPitch() const { return m_data->m_naturalTurretPitch;	}
	Real getTurretFireAngleSweepForWeaponSlot( WeaponSlotType slot ) const;
	Real getTurretSweepSpeedModifierForWeaponSlot( WeaponSlotType slot ) const;
	Real getMinIdleScanAngle() const { return m_data->m_minIdleScanAngle;	}
	Real getMaxIdleScanAngle() const { return m_data->m_maxIdleScanAngle;	}
	UnsignedInt getMinIdleScanInterval() const { return m_data->m_minIdleScanInterval;	}
	UnsignedInt getMaxIdleScanInterval() const { return m_data->m_maxIdleScanInterval;	}
	UnsignedInt getRecenterTime() const { return m_data->m_recenterTime;	}
	Object* getOwner() { return m_owner; }
	const Object* getOwner() const { return m_owner; }

	Bool isOwnersCurWeaponOnTurret() const;
	Bool isWeaponSlotOnTurret(WeaponSlotType wslot) const;
	virtual Bool isAttackingObject() const override { return m_target == TARGET_OBJECT; }
	Bool isForceAttacking() const { return m_isForceAttacking; }

	// this will cause the turret to continuously track the given victim.
	// passing null will allow the turret to revert back to "normal" position.
	void setTurretTargetObject(Object* o, Bool forceAttacking);
	void setTurretTargetPosition(const Coord3D* pos);
	void recenterTurret();
	Bool isTurretInNaturalPosition() const;

	void setTurretEnabled( Bool enabled );
	Bool isTurretEnabled() const { return m_enabled; }

	/**
		return true iff the turret is trying to aim at the victim, BUT not yet pointing in
		the right dir.
	*/
	Bool isTryingToAimAtTarget(const Object* victim) const;

	UpdateSleepTime updateTurretAI();			///< implement this module's behavior

	virtual void notifyFired() override;
	virtual void notifyNewVictimChosen(Object* victim) override;
	virtual const Coord3D* getOriginalVictimPos() const override { return nullptr; }	// yes, we return nullptr here
	virtual Bool isWeaponSlotOkToFire(WeaponSlotType wslot) const override;

	// these are only for use by the state machines... don't call them otherwise, please
	Bool friend_turnTowardsAngle(Real desiredAngle, Real rateModifier, Real relThresh);
	Bool friend_turnTowardsPitch(Real pitch, Real rateModifier);
	Bool friend_getPositiveSweep() const { return m_positiveSweep; }
	void friend_setPositiveSweep(Bool b) { m_positiveSweep = b; }
	Bool friend_isSweepEnabled() const;
	WhichTurretType friend_getWhichTurret() const { return m_whichTurret; }
#ifdef INTER_TURRET_DELAY
	UnsignedInt friend_getInterTurretDelay();
#endif
	Bool friend_isAnyWeaponInRangeOf(const Object* o) const;
	TurretTargetType friend_getTurretTarget( Object*& obj, Coord3D& pos, Bool clearDeadTargets = TRUE ) const;
	Bool friend_getTargetWasSetByIdleMood() const { return m_targetWasSetByIdleMood; }
	const Team* friend_getVictimInitialTeam() const { return m_victimInitialTeam; }
	void friend_checkForIdleMoodTarget();
	UnsignedInt friend_getNextIdleMoodTargetFrame() const;
	void friend_notifyStateMachineChanged();

protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

private:


	void startRotOrPitchSound();					///< start turret rotation sound
	void stopRotOrPitchSound();						///< stop turret rotation sound
	void removeSelfAsTargeter();

#ifdef INTER_TURRET_DELAY
	void getOtherTurretWeaponInfo(Int& numSelf, Int& numSelfReloading, Int& numSelfReady, Int& numOther, Int& numOtherReloading, Int& numOtherReady) const;
#endif

	const TurretAIData*				m_data;
	const WhichTurretType			m_whichTurret;
	Object*										m_owner;
	TurretStateMachine*				m_turretStateMachine;							///< the state machine
	Real											m_angle;										///< angle of the turret
	Real											m_pitch;										///< pitch angle, if this supports it
	AudioEventRTS							m_turretRotOrPitchSound;		///< Sound of turret rotation
	UnsignedInt								m_enableSweepUntil;
	Team*											m_victimInitialTeam; // The team of the victim at the BEGINNING of the attack. If it changes, we may need to stop the attack.
	mutable TurretTargetType	m_target;
	UnsignedInt								m_continuousFireExpirationFrame;
	UnsignedInt								m_sleepUntil;

	Bool										m_playRotSound : 1;
	Bool										m_playPitchSound : 1;
	Bool										m_positiveSweep : 1;
	Bool										m_didFire : 1;
	Bool										m_enabled : 1;
	Bool										m_firesWhileTurning : 1;
	Bool										m_isForceAttacking : 1;
	mutable Bool						m_targetWasSetByIdleMood : 1;

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/AI/TurretAI.cpp`
```cpp
// TurretAI.cpp
// Turret behavior implementation
// Author: Steven Johnson, April 2002

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#define DEFINE_WEAPONSLOTTYPE_NAMES

#include "Common/GameAudio.h"
#include "Common/PerfTimer.h"
#include "Common/RandomValue.h"
#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"

#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/TerrainLogic.h"
#include "GameLogic/TurretAI.h"
#include "GameLogic/Weapon.h"
#include "GameLogic/WeaponSet.h"

const UnsignedInt WAIT_INDEFINITELY = 0xffffffff;


// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
static StateReturnType frameToSleepTime(
	UnsignedInt frame1,
	UnsignedInt frame2 = FOREVER,
	UnsignedInt frame3 = FOREVER,
	UnsignedInt frame4 = FOREVER
)
{
	if (frame1 > frame2) frame1 = frame2;
	if (frame1 > frame3) frame1 = frame3;
	if (frame1 > frame4) frame1 = frame4;
	UnsignedInt now = TheGameLogic->getFrame();
	if (frame1 > now)
	{
		return STATE_SLEEP(frame1 - now);
	}
	else
	{
		// ignore times that are in the past, since this can frequently happen
		return STATE_CONTINUE;
	}
}

//----------------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------------

//----------------------------------------------------------------------------------------------------------
/**
 * Create a TurretAI state machine. Define all of the states the machine
 * can possibly be in, and set the initial (default) state.
 */
TurretStateMachine::TurretStateMachine( TurretAI* tai, Object *obj, AsciiString name ) : m_turretAI(tai), StateMachine( obj, name )
{
	static const StateConditionInfo fireConditions[] =
	{
		StateConditionInfo(outOfWeaponRangeObject, TURRETAI_AIM, nullptr),
		StateConditionInfo(nullptr, INVALID_STATE_ID, nullptr)
	};

	// order matters: first state is the default state.
	defineState( TURRETAI_IDLE,					newInstance(TurretAIIdleState)( this ), TURRETAI_IDLE, TURRETAI_IDLESCAN );
	defineState( TURRETAI_IDLESCAN,			newInstance(TurretAIIdleScanState)( this ), TURRETAI_HOLD, TURRETAI_HOLD );
	defineState( TURRETAI_AIM,					newInstance(TurretAIAimTurretState)( this ), TURRETAI_FIRE, TURRETAI_HOLD );
	defineState( TURRETAI_FIRE,					newInstance(AIAttackFireWeaponState)( this, tai ), TURRETAI_AIM, TURRETAI_AIM, fireConditions );
	defineState( TURRETAI_RECENTER,			newInstance(TurretAIRecenterTurretState)( this ), TURRETAI_IDLE, TURRETAI_IDLE );
	defineState( TURRETAI_HOLD,					newInstance(TurretAIHoldTurretState)( this ), TURRETAI_RECENTER, TURRETAI_RECENTER );
}

//----------------------------------------------------------------------------------------------------------
TurretStateMachine::~TurretStateMachine()
{
}

//----------------------------------------------------------------------------------------------------------
void TurretStateMachine::clear()
{
	StateMachine::clear();

	TurretAI* turret = getTurretAI();
	if (turret)
		turret->friend_notifyStateMachineChanged();
}

//----------------------------------------------------------------------------------------------------------
StateReturnType TurretStateMachine::resetToDefaultState()
{
	StateReturnType tmp = StateMachine::resetToDefaultState();

	TurretAI* turret = getTurretAI();
	if (turret)
		turret->friend_notifyStateMachineChanged();

	return tmp;
}

//----------------------------------------------------------------------------------------------------------
StateReturnType TurretStateMachine::setState(StateID newStateID)
{
	StateID oldID = getCurrentStateID();
	StateReturnType tmp = StateMachine::setState(newStateID);

	TurretAI* turret = getTurretAI();
	if (turret && oldID != newStateID)
		turret->friend_notifyStateMachineChanged();

	return tmp;
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void TurretStateMachine::crc( Xfer *xfer )
{
}

// ------------------------------------------------------------------------------------------------
/** Xfer Method
	* Version Info:
	* 1: Initial version
	* 2: TheSuperHackers @bugfix bobtista 19/07/2026 Serialize the base StateMachine state.
	*    Without this the turret reverts to its default state on load.
	*/
// ------------------------------------------------------------------------------------------------
void TurretStateMachine::xfer( Xfer *xfer )
{
	// version
#if RETAIL_COMPATIBLE_XFER_SAVE
	XferVersion currentVersion = 1;
#else
	XferVersion currentVersion = 2;
#endif
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	if (version >= 2)
	{
		StateMachine::xfer(xfer);
	}
}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void TurretStateMachine::loadPostProcess()
{
	StateMachine::loadPostProcess();
}

//----------------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------------

//----------------------------------------------------------------------------------------------------------
TurretAIData::TurretAIData()
{
	m_turnRate = DEFAULT_TURN_RATE;
	m_pitchRate = DEFAULT_PITCH_RATE;
	m_naturalTurretAngle = 0.0f;
	m_naturalTurretPitch = 0.0f;
	for( Int slotIndex = 0; slotIndex < WEAPONSLOT_COUNT; ++slotIndex )
	{
		m_turretFireAngleSweep[slotIndex] = 0.0f;
		m_turretSweepSpeedModifier[slotIndex] = 1.0f;
	}
	m_firePitch = 0.0f;
	m_minPitch = 0.0f;
	m_groundUnitPitch = 0;
	m_turretWeaponSlots = 0;
#ifdef INTER_TURRET_DELAY
	m_interTurretDelay = 0;
#endif
	m_minIdleScanAngle = 0.0f;
	m_maxIdleScanAngle = 0.0f;
	m_groundUnitPitch = 0.0f;

	m_minIdleScanInterval = 9999999;
	m_maxIdleScanInterval = 9999999;
	m_recenterTime = 2*LOGICFRAMES_PER_SECOND;
	m_initiallyDisabled = false;
	m_firesWhileTurning = FALSE;
	m_isAllowsPitch = false;
}

//-------------------------------------------------------------------------------------------------
static void parseTWS(INI* ini, void * /*instance*/, void * store, const void* /*userData*/)
{
	UnsignedInt* tws = (UnsignedInt*)store;
	for (const char* token = ini->getNextToken(); token; token = ini->getNextTokenOrNull())
	{
		WeaponSlotType wslot = (WeaponSlotType)INI::scanIndexList(token, TheWeaponSlotTypeNames);
		*tws |= (1 << wslot);
	}
}

//-------------------------------------------------------------------------------------------------
void TurretAIData::parseTurretSweep(INI* ini, void *instance, void * /*store*/, const void* userData)
{
	TurretAIData* self = (TurretAIData*)instance;
	WeaponSlotType wslot = (WeaponSlotType)INI::scanIndexList(ini->getNextToken(), TheWeaponSlotTypeNames);
	INI::parseAngleReal( ini, instance, &self->m_turretFireAngleSweep[wslot], nullptr );
}

//-------------------------------------------------------------------------------------------------
void TurretAIData::parseTurretSweepSpeed(INI* ini, void *instance, void * /*store*/, const void* userData)
{
	TurretAIData* self = (TurretAIData*)instance;
	WeaponSlotType wslot = (WeaponSlotType)INI::scanIndexList(ini->getNextToken(), TheWeaponSlotTypeNames);
	INI::parseReal( ini, instance, &self->m_turretSweepSpeedModifier[wslot], nullptr );
}

//----------------------------------------------------------------------------------------------------------
void TurretAIData::buildFieldParse(MultiIniFieldParse& p)
{
	static const FieldParse dataFieldParse[] =
	{
		{ "TurretTurnRate",					INI::parseAngularVelocityReal,				nullptr, offsetof( TurretAIData, m_turnRate ) },
		{ "TurretPitchRate",				INI::parseAngularVelocityReal,				nullptr, offsetof( TurretAIData, m_pitchRate ) },
		{ "NaturalTurretAngle",			INI::parseAngleReal,									nullptr, offsetof( TurretAIData, m_naturalTurretAngle ) },
		{ "NaturalTurretPitch",			INI::parseAngleReal,									nullptr, offsetof( TurretAIData, m_naturalTurretPitch ) },
		{ "FirePitch",							INI::parseAngleReal,									nullptr, offsetof( TurretAIData, m_firePitch ) },
		{ "MinPhysicalPitch",				INI::parseAngleReal,									nullptr, offsetof( TurretAIData, m_minPitch ) },
		{ "GroundUnitPitch",				INI::parseAngleReal,									nullptr, offsetof( TurretAIData, m_groundUnitPitch ) },
		{ "TurretFireAngleSweep",		TurretAIData::parseTurretSweep,				nullptr, 0 },
		{ "TurretSweepSpeedModifier",TurretAIData::parseTurretSweepSpeed,	nullptr, 0 },
		{ "ControlledWeaponSlots",	parseTWS,															nullptr, offsetof( TurretAIData, m_turretWeaponSlots ) },
		{ "AllowsPitch",						INI::parseBool,												nullptr, offsetof( TurretAIData, m_isAllowsPitch ) },
#ifdef INTER_TURRET_DELAY
		{ "InterTurretDelay",				INI::parseDurationUnsignedInt,				nullptr, offsetof( TurretAIData, m_interTurretDelay ) },
#endif
		{ "MinIdleScanAngle",				INI::parseAngleReal,									nullptr, offsetof( TurretAIData, m_minIdleScanAngle ) },
		{ "MaxIdleScanAngle",				INI::parseAngleReal,									nullptr, offsetof( TurretAIData, m_maxIdleScanAngle ) },
		{ "MinIdleScanInterval",		INI::parseDurationUnsignedInt,				nullptr, offsetof( TurretAIData, m_minIdleScanInterval ) },
		{ "MaxIdleScanInterval",		INI::parseDurationUnsignedInt,				nullptr, offsetof( TurretAIData, m_maxIdleScanInterval ) },
		{ "RecenterTime",						INI::parseDurationUnsignedInt,				nullptr, offsetof( TurretAIData, m_recenterTime ) },
		{ "InitiallyDisabled",			INI::parseBool,												nullptr, offsetof( TurretAIData, m_initiallyDisabled ) },
		{ "FiresWhileTurning",			INI::parseBool,												nullptr, offsetof( TurretAIData, m_firesWhileTurning ) },
		{ nullptr, nullptr, nullptr, 0 }
	};
  p.add(dataFieldParse);

}

//----------------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------------

//----------------------------------------------------------------------------------------------------------
TurretAI::TurretAI(Object* owner, const TurretAIData* data, WhichTurretType tur) :
	m_owner(owner),
	m_whichTurret(tur),
	m_data(data),
	m_turretStateMachine(nullptr),
	m_playRotSound(false),
	m_playPitchSound(false),
	m_positiveSweep(true),
	m_enableSweepUntil(0),
	m_sleepUntil(0),
	m_didFire(false),
	m_target(TARGET_NONE),
	m_targetWasSetByIdleMood(false),
	m_enabled(!data->m_initiallyDisabled),
	m_firesWhileTurning(data->m_firesWhileTurning),
	m_isForceAttacking(false),
	m_victimInitialTeam(nullptr)
{
	m_continuousFireExpirationFrame = -1;
	if (!m_data)
	{
		DEBUG_CRASH(("TurretAI MUST have ModuleData"));
		throw INI_INVALID_DATA;
	}

	if (m_data->m_turretWeaponSlots == 0)
	{
		DEBUG_CRASH(("TurretAI MUST specify controlled weapon slots!"));
		throw INI_INVALID_DATA;
	}
	m_angle = getNaturalTurretAngle();
	m_pitch = getNaturalTurretPitch();

#ifdef RTS_DEBUG
	char smbuf[256];
	sprintf(smbuf, "TurretStateMachine for tur %08lx slot %d",this,tur);
	const char* smname = smbuf;
#else
	const char* smname = "TurretStateMachine";
#endif
	m_turretStateMachine = newInstance(TurretStateMachine)(this, m_owner, smname);
	m_turretStateMachine->initDefaultState();

	m_turretRotOrPitchSound = *(m_owner->getTemplate()->getPerUnitSound("TurretMoveLoop"));
}

//----------------------------------------------------------------------------------------------------------
TurretAI::~TurretAI()
{
	stopRotOrPitchSound();

	deleteInstance(m_turretStateMachine);
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void TurretAI::crc( Xfer *xfer )
{
}

// ------------------------------------------------------------------------------------------------
/** Xfer Method */
// ------------------------------------------------------------------------------------------------
void TurretAI::xfer( Xfer *xfer )
{
  // version
  const XferVersion currentVersion = 2;
  XferVersion version = currentVersion;
  xfer->xferVersion( &version, currentVersion );

/* These 4 are loaded on creation, and don't change. jba.
	const TurretAIData*				m_data;
	WhichTurretType						m_whichTurret;
	Object*										m_owner;
	AudioEventRTS							m_turretRotOrPitchSound;		///< Sound of turret rotation
	*/
	xfer->xferSnapshot(m_turretStateMachine);

	xfer->xferReal(&m_angle);
	xfer->xferReal(&m_pitch);
	xfer->xferUnsignedInt(&m_enableSweepUntil);

	xfer->xferUser(&m_target, sizeof(m_target));
	xfer->xferUnsignedInt(&m_continuousFireExpirationFrame);
	Bool tmpBool;
#define UNPACK_AND_XFER(val) {tmpBool = val; xfer->xferBool(&tmpBool); val = tmpBool;}
	UNPACK_AND_XFER(m_playRotSound);
	UNPACK_AND_XFER(m_playPitchSound);
	UNPACK_AND_XFER(m_positiveSweep);
	UNPACK_AND_XFER(m_didFire);
	UNPACK_AND_XFER(m_enabled);
	UNPACK_AND_XFER(m_firesWhileTurning);
	UNPACK_AND_XFER(m_targetWasSetByIdleMood);
#undef UNPACK_AND_XFER

	if (version >= 2)
		xfer->xferUnsignedInt(&m_sleepUntil);

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void TurretAI::loadPostProcess()
{
	Object *victim = m_turretStateMachine->getGoalObject();
	if (victim) {
		m_victimInitialTeam = victim->getTeam();
	}
}

//----------------------------------------------------------------------------------------------------------
Bool TurretAI::friend_turnTowardsAngle(Real desiredAngle, Real rateModifier, Real relThresh)
{
	desiredAngle = normalizeAngle(desiredAngle);

	// rotate turret back to zero angle
	Real origAngle = getTurretAngle();
	Real actualAngle = origAngle;
	Real turnRate = getTurnRate() * rateModifier;
	Real angleDiff = normalizeAngle(desiredAngle - actualAngle);

	// Are we close enough to the desired angle to just snap there?
	if (WWMath::Fabs(angleDiff) < turnRate)
	{
		// we are centered
		actualAngle = desiredAngle;

		getOwner()->clearModelConditionState(MODELCONDITION_TURRET_ROTATE);
	}
	else
	{
		if (angleDiff > 0)
			actualAngle += turnRate;
		else
			actualAngle -= turnRate;

		getOwner()->setModelConditionState(MODELCONDITION_TURRET_ROTATE);
		m_playRotSound = true;
	}

	m_angle = normalizeAngle(actualAngle);

	if( m_angle != origAngle )
		getOwner()->reactToTurretChange( m_whichTurret, origAngle, m_pitch );

	Bool aligned = WWMath::Fabs(m_angle - desiredAngle) <= relThresh;

	return aligned;
}

//----------------------------------------------------------------------------------------------------------
Bool TurretAI::friend_turnTowardsPitch(Real desiredPitch, Real rateModifier)
{
	if (!isAllowsPitch())
		return true;

	desiredPitch = normalizeAngle(desiredPitch);

	// rotate turret back to zero angle
	Real actualPitch = getTurretPitch();
	Real pitchRate = getPitchRate() * rateModifier;
	Real pitchDiff = normalizeAngle(desiredPitch - actualPitch);

	if (WWMath::Fabs(pitchDiff) < pitchRate)
	{
		// we are centered
		actualPitch = desiredPitch;
	}
	else
	{
		if (pitchDiff > 0)
			actualPitch += pitchRate;
		else
			actualPitch -= pitchRate;
		m_playPitchSound = true;
	}

	m_pitch = normalizeAngle(actualPitch);

	return (m_pitch == desiredPitch);
}

//----------------------------------------------------------------------------------------------------------
Bool TurretAI::isWeaponSlotOkToFire(WeaponSlotType wslot) const
{
  // If we turrets are linked, ai wants us to fire together, regardless of slot
  if( getOwner()->getAI()->areTurretsLinked() )
    return TRUE;

	return isWeaponSlotOnTurret(wslot);
}

//----------------------------------------------------------------------------------------------------------
Real TurretAI::getTurretFireAngleSweepForWeaponSlot( WeaponSlotType slot ) const
{
	return m_data->m_turretFireAngleSweep[slot];
}

//----------------------------------------------------------------------------------------------------------
Real TurretAI::getTurretSweepSpeedModifierForWeaponSlot( WeaponSlotType slot ) const
{
	return m_data->m_turretSweepSpeedModifier[slot];
}

//----------------------------------------------------------------------------------------------------------
void TurretAI::notifyFired()
{
	m_didFire = true;
}

//----------------------------------------------------------------------------------------------------------
void TurretAI::notifyNewVictimChosen(Object* victim)
{
	setTurretTargetObject(victim, FALSE);
}

//----------------------------------------------------------------------------------------------------------
Bool TurretAI::isTryingToAimAtTarget(const Object* victim) const
{
	StateID sid = m_turretStateMachine->getCurrentStateID();

	Object* obj;
	Coord3D pos;
	return (sid == TURRETAI_AIM && friend_getTurretTarget(obj, pos) == TARGET_OBJECT && obj == victim);
}

//----------------------------------------------------------------------------------------------------------
Bool TurretAI::isOwnersCurWeaponOnTurret() const
{
	WeaponSlotType wslot;
	Weapon* w = m_owner->getCurrentWeapon(&wslot);
	return w != nullptr && isWeaponSlotOnTurret(wslot);
}

//----------------------------------------------------------------------------------------------------------
Bool TurretAI::isWeaponSlotOnTurret(WeaponSlotType wslot) const
{
	return (m_data->m_turretWeaponSlots & (1 << wslot)) != 0;
}

//----------------------------------------------------------------------------------------------------------
TurretTargetType TurretAI::friend_getTurretTarget( Object*& obj, Coord3D& pos, Bool clearDeadTargets ) const
{
	obj = nullptr;
	pos.zero();

	if (m_target == TARGET_OBJECT)
	{
		obj = m_turretStateMachine->getGoalObject();
		// clear it explicitly if null -- could be deceased but holding the
		// old (bogus) objectid internally
		if( clearDeadTargets )
		{
			if (obj == nullptr || obj->isEffectivelyDead())
			{
				m_turretStateMachine->setGoalObject(nullptr);
				m_target = TARGET_NONE;
				m_targetWasSetByIdleMood = false;
			}
		}
	}
	else if (m_target == TARGET_POSITION)
	{
		obj = nullptr;
		pos = *m_turretStateMachine->getGoalPosition();
	}

	return m_target;
}

//----------------------------------------------------------------------------------------------------------
void TurretAI::removeSelfAsTargeter()
{
	// be paranoid, in case we are called from dtors, etc.
	if (m_target == TARGET_OBJECT && m_turretStateMachine != nullptr)
	{
		Object* self = m_owner;
		Object* target = m_turretStateMachine->getGoalObject();
		if (self != nullptr && target != nullptr)
		{
			AIUpdateInterface* targetAI = target->getAI();
			if (targetAI)
			{
				targetAI->addTargeter(self->getID(), false);
			}
		}
	}
}

//----------------------------------------------------------------------------------------------------------
void TurretAI::setTurretTargetObject( Object *victim, Bool forceAttacking )
{
	if( !victim || victim->isEffectivelyDead() ||	!isOwnersCurWeaponOnTurret() )
	{
		if( !getOwner()->getAI()->areTurretsLinked() )
		{
			victim = nullptr;
		}
	}

	if (victim == nullptr)
	{
		// if nuking the victim, remove self as targeter before doing anything else.
		// (note that we never ADD self as targeter here; that is done in the aim state)
		removeSelfAsTargeter();
	}

	m_turretStateMachine->setGoalObject( victim );	// could be null! this is OK!
	m_target = victim ? TARGET_OBJECT : TARGET_NONE;
	m_targetWasSetByIdleMood = false;
	m_isForceAttacking = forceAttacking;

	StateID sid = m_turretStateMachine->getCurrentStateID();
	if (victim != nullptr)
	{
		// if we're already in the aim state, don't call setState, since
		// it would go thru the exit/enter stuff, which we don't really want
		// to do...
		if (sid != TURRETAI_AIM && sid != TURRETAI_FIRE)
			m_turretStateMachine->setState( TURRETAI_AIM );
		m_victimInitialTeam = victim->getTeam();
	}
	else
	{
		// only change states if we are aiming.
		if (sid == TURRETAI_AIM || sid == TURRETAI_FIRE)
			m_turretStateMachine->setState(TURRETAI_HOLD);
		m_victimInitialTeam = nullptr;
	}
}

//----------------------------------------------------------------------------------------------------------
void TurretAI::setTurretTargetPosition( const Coord3D* pos )
{
	if (!pos ||	!isOwnersCurWeaponOnTurret())
	{
		if( !getOwner()->getAI()->areTurretsLinked() )
		{
			pos = nullptr;
		}
	}

	// remove self as targeter before doing anything else.
	// (note that we never ADD self as targeter here; that is done in the aim state)
	removeSelfAsTargeter();

	m_turretStateMachine->setGoalObject( nullptr );
	if (pos)
		m_turretStateMachine->setGoalPosition( pos );
	m_target = pos ? TARGET_POSITION : TARGET_NONE;
	m_targetWasSetByIdleMood = false;

	StateID sid = m_turretStateMachine->getCurrentStateID();
	if (pos != nullptr)
	{
		// if we're already in the aim state, don't call setState, since
		// it would go thru the exit/enter stuff, which we don't really want
		// to do...
		if (sid != TURRETAI_AIM && sid != TURRETAI_FIRE)
			m_turretStateMachine->setState( TURRETAI_AIM );
		m_victimInitialTeam = nullptr;
	}
	else
	{
		// only change states if we are aiming.
		if (sid == TURRETAI_AIM || sid == TURRETAI_FIRE)
			m_turretStateMachine->setState(TURRETAI_HOLD);
		m_victimInitialTeam = nullptr;
	}
}

//----------------------------------------------------------------------------------------------------------
void TurretAI::recenterTurret()
{
	m_turretStateMachine->setState( TURRETAI_RECENTER );
}

//----------------------------------------------------------------------------------------------------------
Bool TurretAI::isTurretInNaturalPosition() const
{

  if( this->getOwner()->testStatus( OBJECT_STATUS_UNDER_CONSTRUCTION))
    return true;//ML so that under-construction base-defenses do not re-center while under construction



	if( getNaturalTurretAngle() == getTurretAngle() &&
			getNaturalTurretPitch() == getTurretPitch() )
	{
		return true;
	}
	return false;
}

//----------------------------------------------------------------------------------------------------------
void TurretAI::friend_notifyStateMachineChanged()
{
	m_sleepUntil = TheGameLogic->getFrame();
}

//----------------------------------------------------------------------------------------------------------
/*
	Note that TurretAI isn't a behavior at all, so doesn't really "sleep"; however, it does return
	this information so that its AIUpdate owner can decide how long it (the AIUpdate) should sleep.
	So it behooves you to maximize "sleep" potential here! (srj)
*/
DECLARE_PERF_TIMER(TurretAI)
UpdateSleepTime TurretAI::updateTurretAI()
{
	USE_PERF_TIMER(TurretAI)

#if defined(RTS_DEBUG)
	DEBUG_ASSERTCRASH(!m_enabled ||
							m_turretStateMachine->peekSleepTill() == 0 ||
							m_turretStateMachine->peekSleepTill() >= m_sleepUntil, ("Turret Machine is less sleepy than turret"));
#endif

	UnsignedInt now = TheGameLogic->getFrame();
	if (m_sleepUntil != 0 && now < m_sleepUntil)
	{
		return UPDATE_SLEEP(m_sleepUntil - now);
	}

	//DEBUG_LOG(("updateTurretAI frame %d: %08lx",TheGameLogic->getFrame(),getOwner()));
	UpdateSleepTime subMachineSleep = UPDATE_SLEEP_FOREVER;	// assume the best!

	// either we don't care about continuous fire stuff, or we care, but time has elapsed
	if ((!m_firesWhileTurning) || (m_continuousFireExpirationFrame <= now))
	{
		m_playRotSound = false;
		m_playPitchSound = false;
	}

	if (m_enabled || m_turretStateMachine->getCurrentStateID() == TURRETAI_RECENTER)
	{
		m_didFire = false;

		// run the behavior state machine BEFORE doing sound check
		StateReturnType stRet = m_turretStateMachine->updateStateMachine();

		if (m_didFire)
		{
			// if we fired, enable sweeping for a few frames.
			const Int ENABLE_SWEEP_FRAME_COUNT = 3;
			m_enableSweepUntil = now + ENABLE_SWEEP_FRAME_COUNT;
			m_continuousFireExpirationFrame = now + ENABLE_SWEEP_FRAME_COUNT;// so the recent firing will not interrupt the moving sound
		}

		if (m_playRotSound || m_playPitchSound)
			startRotOrPitchSound();
		else
			stopRotOrPitchSound();

		if (IS_STATE_SLEEP(stRet))
		{
			Int frames = GET_STATE_SLEEP_FRAMES(stRet);
			if (frames < subMachineSleep)
				subMachineSleep = UPDATE_SLEEP(frames);
		}
		else
		{
			// it's STATE_CONTINUE, STATE_SUCCESS, or STATE_FAILURE,
			// any of which will probably require next frame
			subMachineSleep = UPDATE_SLEEP_NONE;
		}

	}

	m_sleepUntil = now + subMachineSleep;

#if defined(RTS_DEBUG)
	DEBUG_ASSERTCRASH(!m_enabled ||
							m_turretStateMachine->peekSleepTill() == 0 ||
							m_turretStateMachine->peekSleepTill() >= m_sleepUntil, ("Turret Machine is less sleepy than turret"));
#endif

	return subMachineSleep;
}

//-------------------------------------------------------------------------------------------------
void TurretAI::setTurretEnabled( Bool enabled )
{
	if (enabled && !m_enabled)
	{
		// be sure we wake up!
		m_sleepUntil = TheGameLogic->getFrame();
	}
	m_enabled = enabled;
}

//-------------------------------------------------------------------------------------------------
/**
 * Start turret rotation sound
 */
void TurretAI::startRotOrPitchSound()
{
	if (!m_turretRotOrPitchSound.isCurrentlyPlaying())
	{
		m_turretRotOrPitchSound.setObjectID(m_owner->getID());
		m_turretRotOrPitchSound.setPlayingHandle(TheAudio->addAudioEvent(&m_turretRotOrPitchSound));
	}
}

//-------------------------------------------------------------------------------------------------
/**
 * Stop turret rotation sound
 */
void TurretAI::stopRotOrPitchSound()
{
	if (m_turretRotOrPitchSound.isCurrentlyPlaying())
	{
		TheAudio->removeAudioEvent(m_turretRotOrPitchSound.getPlayingHandle());
	}
}

#ifdef INTER_TURRET_DELAY
//----------------------------------------------------------------------------------------------------------
void TurretAI::getOtherTurretWeaponInfo(Int& numSelf, Int& numSelfReloading, Int& numSelfReady, Int& numOther, Int& numOtherReloading, Int& numOtherReady) const
{
	numSelf = 0;
	numSelfReady = 0;
	numSelfReloading = 0;
	numOther = 0;
	numOtherReady = 0;
	numOtherReloading = 0;

	for( Int i = 0; i < WEAPONSLOT_COUNT;	i++ )
	{
		// ignore empty slots.
		const Weapon* w = getOwner()->getWeaponInWeaponSlot((WeaponSlotType)i);
		if (w == nullptr)
			continue;

		// ignore the weapons on this turret.
		if (isWeaponSlotOnTurret((WeaponSlotType)i))
		{
			++numSelf;
			if (w->getStatus() == RELOADING_CLIP)
				++numSelfReloading;
			else if (w->getStatus() == READY_TO_FIRE)
				++numSelfReady;
		}
		else
		{
			++numOther;
			if (w->getStatus() == RELOADING_CLIP)
				++numOtherReloading;
			else if (w->getStatus() == READY_TO_FIRE)
				++numOtherReady;
		}
	}
}
#endif

//----------------------------------------------------------------------------------------------------------
Bool TurretAI::friend_isAnyWeaponInRangeOf(const Object* o) const
{
	// see if we've moved out of range, and if so, nuke the target.
	for (Int i = 0; i < WEAPONSLOT_COUNT;	i++ )
	{
		// ignore empty slots.
		const Weapon* w = getOwner()->getWeaponInWeaponSlot((WeaponSlotType)i);
		if (w == nullptr || !isWeaponSlotOnTurret((WeaponSlotType)i))
			continue;

		if (w->isWithinAttackRange(getOwner(), o)
				// srj sez: not sure if we want to do this or not.
				// jba sez: no, don't do this.  isWithinAttackRange checks terrain los now, and
				// some weapons (like tomahawk) can fire beyond los, so this check is problematical here.
				//  && w->isClearFiringLineOfSightTerrain(getOwner(), o)
			)
		{
			return true;
		}
	}

	return false;
}

//----------------------------------------------------------------------------------------------------------
Bool TurretAI::friend_isSweepEnabled() const
{
	if (m_enableSweepUntil != 0 && m_enableSweepUntil > TheGameLogic->getFrame())
		return true;

	return false;
}

//----------------------------------------------------------------------------------------------------------
UnsignedInt TurretAI::friend_getNextIdleMoodTargetFrame() const
{
	const Object* obj = getOwner();
	const AIUpdateInterface *ai = obj->getAIUpdateInterface();
	// ai can be null during object construction.
	return ai ? ai->getNextMoodCheckTime() : TheGameLogic->getFrame();
}

//----------------------------------------------------------------------------------------------------------
void TurretAI::friend_checkForIdleMoodTarget()
{
	Object* obj = getOwner();
	AIUpdateInterface *ai = obj->getAIUpdateInterface();

	// This state is used internally some places, so we don't necessarily want to be looking for targets
	// Places that use AI_IDLE internally should set this to false in the constructor. jkmcd
	UnsignedInt moodAdjust = ai->getMoodMatrixActionAdjustment(MM_Action_Idle);
	if (moodAdjust & MAA_Affect_Range_IgnoreAll)
		return;

	// If we're supposed to attack based on mood, etc, then we will do so.
	Object* enemy = ai->getNextMoodTarget( true, true );
	if (enemy)
	{
		setTurretTargetObject(enemy, FALSE);
		obj->chooseBestWeaponForTarget(enemy, PREFER_MOST_DAMAGE, CMD_FROM_AI);
		// set this magic flag to indicate that this was an independent attack-of-opportunity,
		// and we should check to see if out-of-range, and if so, nuke it
		m_targetWasSetByIdleMood = true;
	}
}

#ifdef INTER_TURRET_DELAY
//----------------------------------------------------------------------------------------------------------
UnsignedInt TurretAI::friend_getInterTurretDelay()
{
	if (m_data->m_interTurretDelay == 0)
		return 0;

	Int numSelf;
	Int numSelfReloading;
	Int numSelfReady;
	Int numOther;
	Int numOtherReloading;
	Int numOtherReady;
	getOtherTurretWeaponInfo(numSelf, numSelfReloading, numSelfReady, numOther, numOtherReloading, numOtherReady);

	AIUpdateInterface* ai = getOwner()->getAIUpdateInterface();
	WhichTurretType t = ai->friend_getTurretSync();
	if (t == TURRET_INVALID || t == m_whichTurret)
	{
		if (numSelf == numSelfReady)
		{
			ai->friend_setTurretSync(m_whichTurret);
			// if we just got the flag, use our delay, otherwise go immediately
			if (t == TURRET_INVALID)
				return m_data->m_interTurretDelay;
			else
				return 0;
		}
		else if (numSelf == numSelfReloading)
		{
			// we're done... relinquish the flag
			ai->friend_setTurretSync(TURRET_INVALID);
			return WAIT_INDEFINITELY;
		}
		else
		{
			// we're not ready, but not reloading either, so retain the flag and wait
			return WAIT_INDEFINITELY;
		}
	}
	else
	{
		// someone else has it, so just wait.
		return WAIT_INDEFINITELY;
	}
}
#endif

//----------------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------------

//----------------------------------------------------------------------------------------------------------
StateReturnType TurretAIAimTurretState::onEnter()
{
	// don't set the "turret-rotate" bit here; wait and see if we really turn or not.
#ifdef INTER_TURRET_DELAY
	m_extraDelay = 0;
#endif
	return STATE_CONTINUE;
}

//-------------------------------------------------------------------------------------------------
/**
 * Rotate our turret to point at the machine's goal
 */
StateReturnType TurretAIAimTurretState::update()
{
	//DEBUG_LOG(("TurretAIAimTurretState frame %d: %08lx",TheGameLogic->getFrame(),getTurretAI()->getOwner()));

	TurretAI* turret = getTurretAI();
	Object* obj = turret->getOwner();
	AIUpdateInterface* ai = obj->getAIUpdateInterface();
	if( !ai )
	{
		return STATE_FAILURE;
	}

	Object* enemy;
	AIUpdateInterface* enemyAI=nullptr;
	Coord3D enemyPosition;
	Bool preventing = false;
	TurretTargetType targetType =  turret->friend_getTurretTarget(enemy, enemyPosition);
	Object *enemyForDistanceCheckOnly = enemy;	// Note: Do not use this anywhere except for the range check.

	Bool nothingInRange = false;
	switch (targetType)
	{
		case TARGET_NONE:
		{
			return STATE_FAILURE;
		}

		case TARGET_OBJECT:
		{
			Bool isPrimaryEnemy = (enemy && enemy == ai->getGoalObject());
			// if the enemy is gone, or we're out of range, or it changed teams, the attack is over
			Bool ableToAttackTarget = obj->isAbleToAttack();
			if (ableToAttackTarget)
			{
				// srj sez: since we have already acquired this target, we should use
				// the CONTINUED attack tests, not the new ones.
				CanAttackResult result = obj->getAbleToAttackSpecificObject(
							turret->isForceAttacking() ? ATTACK_CONTINUED_TARGET_FORCED : ATTACK_CONTINUED_TARGET,
							enemy,
							ai->getLastCommandSource()
						);
				ableToAttackTarget = result == ATTACKRESULT_POSSIBLE || result == ATTACKRESULT_POSSIBLE_AFTER_MOVING;
			}

			nothingInRange = !turret->friend_isAnyWeaponInRangeOf(enemy);
			if (enemy == nullptr || !ableToAttackTarget ||
					(!isPrimaryEnemy && nothingInRange) ||
					enemy->getTeam() != turret->friend_getVictimInitialTeam()
			)
			{
				if (turret->friend_getTargetWasSetByIdleMood())
				{
					turret->setTurretTargetObject(nullptr, FALSE);
				}
				return STATE_FAILURE;
			}

			// aim turret towards enemy (turret angle is relative to its parent object)
			if (enemy->isKindOf(KINDOF_BRIDGE))
			{
				// Special case - bridges have two attackable points at either end.
				TBridgeAttackInfo info;
				TheTerrainLogic->getBridgeAttackPoints(enemy, &info);
				Real distSqr = ThePartitionManager->getDistanceSquared( obj, &info.attackPoint1, FROM_BOUNDINGSPHERE_3D );
				if (distSqr > ThePartitionManager->getDistanceSquared( obj, &info.attackPoint2, FROM_BOUNDINGSPHERE_3D ) )
				{
					enemyPosition = info.attackPoint2;
				}
				else
				{
					enemyPosition = info.attackPoint1;
				}
			}
			else
			{
				enemyPosition = *enemy->getPosition();
			}

			enemyAI = enemy ? enemy->getAI() : nullptr;

			// add ourself as a targeter BEFORE calling isTemporarilyPreventingAimSuccess().
			// we do this every time thru, just in case we get into a squabble with our ai
			// over whether or not we are a targeter... (srj)
			if (enemyAI)
				enemyAI->addTargeter(obj->getID(), true);

			preventing = enemyAI && enemyAI->isTemporarilyPreventingAimSuccess();

			// don't use 'enemy' after this point, just the position. to help
			// enforce this, we'll null it out.
			enemy = nullptr;
			break;
		}

		case TARGET_POSITION:
		{
			// nothing, just break.
			break;
		}
	}

	WeaponSlotType slot;
	Weapon *curWeapon = obj->getCurrentWeapon( &slot );
	if (!curWeapon)
	{
		DEBUG_CRASH(("TurretAIAimTurretState::update - curWeapon is null."));
		return STATE_FAILURE;
	}

	Real turnSpeedModifier = 1.0f;// Just like how recentering turns you half speed, sweeping can change your turn speed

	Real relAngle = ThePartitionManager->getRelativeAngle2D( obj, &enemyPosition );

	Real aimAngle = relAngle;
	Real sweep = turret->getTurretFireAngleSweepForWeaponSlot( slot );
	if (sweep > 0.0f && turret->friend_isSweepEnabled())
	{
		if (turret->friend_getPositiveSweep())
			aimAngle += sweep;
		else
			aimAngle -= sweep;

		turnSpeedModifier = turret->getTurretSweepSpeedModifierForWeaponSlot( slot );
	}

	const Real REL_THRESH = 0.035f;	// about 2 degrees. (getRelativeAngle2D is current only accurate to about 1.25 degrees)
	Bool turnAlignedToNemesis = turret->friend_turnTowardsAngle(aimAngle, turnSpeedModifier, REL_THRESH);

	// this section we do even if sweep is "disabled", so that we can start firing
	// once we get into sweep "range"
	if (sweep > 0.0f)
	{
		if (turnAlignedToNemesis)
			turret->friend_setPositiveSweep(!turret->friend_getPositiveSweep());

		Real angleDiff = normalizeAngle(relAngle - turret->getTurretAngle());
		turnAlignedToNemesis = (WWMath::Fabs(angleDiff) < sweep);
	}

	Bool pitchAlignedToNemesis = true;

	// Now do pitch
	if( turret->isAllowsPitch() )
	{
		Real desiredPitch = 0;

		// Some turrets want to fire at a set pitch, and some want to fire straight to the target
		if( turret->getFirePitch() > 0 )
		{
			desiredPitch = turret->getFirePitch();
		}
		else
		{
			// Find the pitch to the target, but unlike Turn, we can't go 360 so bind at 0;
			Coord3D v;
			ThePartitionManager->getVectorTo(obj, &enemyPosition, FROM_CENTER_3D, v);

			//GetVectorTo only takes Object as the first, but we want the angle from our Weapon to the
			// target, not us to the target.  Raise our side to get the line to make sense.
			v.z -= obj->getGeometryInfo().getMaxHeightAbovePosition() / 2; // I kinda hate our logic/client split.
			//The point to fire from should be intrinsic to the turret, but in reality it is very slow to look it up.

 			Real actualPitch;
 			if( v.length() > 0 )
 				actualPitch = ASin( v.z / v.length() );
 			else
 				actualPitch = 0;// Don't point at NAN, just point at 0 if they are right on us

			desiredPitch = actualPitch;
			if( desiredPitch < turret->getMinPitch() )
			{
				desiredPitch = turret->getMinPitch();
			}
			if (turret->getGroundUnitPitch() > 0) {
				Bool adjust = false;
				if (!enemy) {
					adjust = true; // adjust for ground targets.
				}
				if (enemy && enemy->isKindOf(KINDOF_IMMOBILE)) {
					adjust = true;
				}
				if (enemyAI && enemyAI->isDoingGroundMovement()) {
					adjust = true;
				}
				if (adjust) {
					Real range = curWeapon->getAttackRange(obj);
					Real dist = v.length();
					if (range<1) range = 1; // paranoia. jba.
					// As the unit gets closer, reduce the pitch so we don't shoot over him.
					Real groundPitch = turret->getGroundUnitPitch() * (dist/range);
					desiredPitch = actualPitch+groundPitch;
					if (desiredPitch < turret->getMinPitch()) {
						desiredPitch = turret->getMinPitch();
					}
				}
			}

		}

		pitchAlignedToNemesis = turret->friend_turnTowardsPitch(desiredPitch, 1.0f);
	}

	// For now, we require that we're within range before we can successfully exit the AIM state,
	// and move into the FIRE state.
	if (turnAlignedToNemesis && pitchAlignedToNemesis &&
		((enemyForDistanceCheckOnly && curWeapon->isWithinAttackRange(obj, enemyForDistanceCheckOnly)) ||
		 (!enemyForDistanceCheckOnly && curWeapon->isWithinAttackRange(obj, &enemyPosition))))
	{
#ifdef INTER_TURRET_DELAY
		if (m_extraDelay > 0)
		{
			--m_extraDelay;
		}
		else
		{
			m_extraDelay = turret->friend_getInterTurretDelay();
			if (m_extraDelay == WAIT_INDEFINITELY)
			{
				m_extraDelay = 0;
				return STATE_CONTINUE;
			}
		}
		if (m_extraDelay != 0)
		{
			return STATE_CONTINUE;
		}
#endif
		if (preventing || nothingInRange)
		{
			return STATE_CONTINUE;
		}
		return STATE_SUCCESS;
	}

	// stay in aiming state until directed to do something else
	return STATE_CONTINUE;
}

//-------------------------------------------------------------------------------------------------
void TurretAIAimTurretState::onExit( StateExitType status )
{
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------

//----------------------------------------------------------------------------------------------------------
StateReturnType TurretAIRecenterTurretState::onEnter()
{
	return STATE_CONTINUE;
}

//-------------------------------------------------------------------------------------------------
/**
 * Rotate the owner's turret to its home orientation.
 */

StateReturnType TurretAIRecenterTurretState::update()
{
	//DEBUG_LOG(("TurretAIRecenterTurretState frame %d: %08lx",TheGameLogic->getFrame(),getTurretAI()->getOwner()));


  if( getMachineOwner()->testStatus( OBJECT_STATUS_UNDER_CONSTRUCTION))
    return STATE_CONTINUE;//ML so that under-construction base-defenses do not re-center while under construction


	TurretAI* turret = getTurretAI();
	Bool angleAligned = turret->friend_turnTowardsAngle(turret->getNaturalTurretAngle(), 0.5f, 0.0f);
	Bool pitchAligned = turret->friend_turnTowardsPitch(turret->getNaturalTurretPitch(), 0.5f);

	if( angleAligned && pitchAligned )
		return STATE_SUCCESS;

	return STATE_CONTINUE;
}

//-------------------------------------------------------------------------------------------------
/**
 * Stop rotation sound
 */
void TurretAIRecenterTurretState::onExit( StateExitType status )
{
}

//----------------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------------

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void TurretAIIdleState::crc( Xfer *xfer )
{
}

// ------------------------------------------------------------------------------------------------
/** Xfer Method */
// ------------------------------------------------------------------------------------------------
void TurretAIIdleState::xfer( Xfer *xfer )
{
  // version
  XferVersion currentVersion = 1;
  XferVersion version = currentVersion;
  xfer->xferVersion( &version, currentVersion );

	xfer->xferUnsignedInt(&m_nextIdleScan);
}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void TurretAIIdleState::loadPostProcess()
{
}

//----------------------------------------------------------------------------------------------------------
void TurretAIIdleState::resetIdleScan()
{
	UnsignedInt now = TheGameLogic->getFrame();
	UnsignedInt delay = GameLogicRandomValue(getTurretAI()->getMinIdleScanInterval(), getTurretAI()->getMaxIdleScanInterval());
	m_nextIdleScan = now + delay;
}

//----------------------------------------------------------------------------------------------------------
StateReturnType TurretAIIdleState::onEnter()
{
	AIUpdateInterface *ai = getMachineOwner()->getAIUpdateInterface();
	if (ai)
	{
		ai->resetNextMoodCheckTime();
		if (ai->friend_getTurretSync() == getTurretAI()->friend_getWhichTurret())
			ai->friend_setTurretSync(TURRET_INVALID);
	}

	resetIdleScan();

	TurretAI* turret = getTurretAI();
	return frameToSleepTime(turret->friend_getNextIdleMoodTargetFrame(), m_nextIdleScan);
}

//----------------------------------------------------------------------------------------------------------
StateReturnType TurretAIIdleState::update()
{
	//DEBUG_LOG(("TurretAIIdleState frame %d: %08lx",TheGameLogic->getFrame(),getTurretAI()->getOwner()));

	UnsignedInt now = TheGameLogic->getFrame();
	if (now >= m_nextIdleScan)
	{
		// this is redundant, since we're exiting the state, and will reset
		// it again in onEnter next time (srj)
		// resetIdleScan();

		return STATE_FAILURE;
	}

	TurretAI* turret = getTurretAI();
	turret->friend_checkForIdleMoodTarget();

	return frameToSleepTime(turret->friend_getNextIdleMoodTargetFrame(), m_nextIdleScan);
}

//----------------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------------

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void TurretAIIdleScanState::crc( Xfer *xfer )
{
}

// ------------------------------------------------------------------------------------------------
/** Xfer Method */
// ------------------------------------------------------------------------------------------------
void TurretAIIdleScanState::xfer( Xfer *xfer )
{
  // version
  XferVersion currentVersion = 1;
  XferVersion version = currentVersion;
  xfer->xferVersion( &version, currentVersion );

	xfer->xferReal(&m_desiredAngle);
}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void TurretAIIdleScanState::loadPostProcess()
{
}

//----------------------------------------------------------------------------------------------------------
StateReturnType TurretAIIdleScanState::onEnter()
{
	Real minA = getTurretAI()->getMinIdleScanAngle();
	Real maxA = getTurretAI()->getMaxIdleScanAngle();
	if (minA == 0.0f && maxA == 0.0f)
		return STATE_SUCCESS;

	m_desiredAngle = minA + GameLogicRandomValueReal(0, maxA - minA);
	if (GameLogicRandomValue( 0, 1 ) == 0)
		m_desiredAngle = -m_desiredAngle;

	return STATE_CONTINUE;
}

//-------------------------------------------------------------------------------------------------
/**
 * Rotate the owner's turret to its scan orientation.
 */

StateReturnType TurretAIIdleScanState::update()
{
	//DEBUG_LOG(("TurretAIIdleScanState frame %d: %08lx",TheGameLogic->getFrame(),getTurretAI()->getOwner()));

  if( getMachineOwner()->testStatus( OBJECT_STATUS_UNDER_CONSTRUCTION))
    return STATE_CONTINUE;//ML so that under-construction base-defenses do not idle-scan while under construction

	Bool angleAligned = getTurretAI()->friend_turnTowardsAngle(getTurretAI()->getNaturalTurretAngle() + m_desiredAngle, 0.5f, 0.0f);
	Bool pitchAligned = getTurretAI()->friend_turnTowardsPitch(getTurretAI()->getNaturalTurretPitch(), 0.5f);

	if( angleAligned && pitchAligned )
		return STATE_SUCCESS;

	return STATE_CONTINUE;
}

//-------------------------------------------------------------------------------------------------
/**
 * Stop rotation sound
 */
void TurretAIIdleScanState::onExit( StateExitType status )
{
}

//----------------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------------

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void TurretAIHoldTurretState::crc( Xfer *xfer )
{
}

// ------------------------------------------------------------------------------------------------
/** Xfer Method */
// ------------------------------------------------------------------------------------------------
void TurretAIHoldTurretState::xfer( Xfer *xfer )
{
  // version
  XferVersion currentVersion = 1;
  XferVersion version = currentVersion;
  xfer->xferVersion( &version, currentVersion );

	xfer->xferUnsignedInt(&m_timestamp);
}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void TurretAIHoldTurretState::loadPostProcess()
{
}

//----------------------------------------------------------------------------------------------------------
StateReturnType TurretAIHoldTurretState::onEnter()
{
	m_timestamp = TheGameLogic->getFrame() + getTurretAI()->getRecenterTime();

	TurretAI* turret = getTurretAI();
	return frameToSleepTime(turret->friend_getNextIdleMoodTargetFrame(), m_timestamp);
}

//-------------------------------------------------------------------------------------------------
void TurretAIHoldTurretState::onExit( StateExitType status )
{
}

//----------------------------------------------------------------------------------------------------------
/**
 * Hold the turret's position for a short time before returning to center.
 */

StateReturnType TurretAIHoldTurretState::update()
{
	//DEBUG_LOG(("TurretAIHoldTurretState frame %d: %08lx",TheGameLogic->getFrame(),getTurretAI()->getOwner()));

	if (TheGameLogic->getFrame() >= m_timestamp)
		return STATE_SUCCESS;

	TurretAI* turret = getTurretAI();
	turret->friend_checkForIdleMoodTarget();

	return frameToSleepTime(turret->friend_getNextIdleMoodTargetFrame(), m_timestamp);
}
```
