# Translation Packet `PKT-165`: `ActiveBody`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-165`
- **Work Package:** `WP-5.1` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/body/active_body.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (33):**
  - `Common/BitFlagsIO.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/BitFlagsIO.h`
  - `Common/CRCDebug.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/CRCDebug.h`
  - `Common/DamageFX.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/DamageFX.h`
  - `Common/GameState.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/GameState.h`
  - `Common/GlobalData.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/GlobalData.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/PlayerList.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/PlayerList.h`
  - `Common/Team.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Team.h`
  - `Common/Thing.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Thing.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/ControlBar.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/ControlBar.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameClient/ParticleSys.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/ParticleSys.h`
  - `GameLogic/AI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/AI.h`
  - `GameLogic/AIPathfind.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/AIPathfind.h`
  - `GameLogic/Armor.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Armor.h`
  - `GameLogic/ArmorSet.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ArmorSet.h`
  - `GameLogic/Damage.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/Damage.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/ActiveBody.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ActiveBody.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/BridgeBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BridgeBehavior.h`
  - `GameLogic/Module/ContainModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ContainModule.h`
  - `GameLogic/Module/DamageModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DamageModule.h`
  - `GameLogic/Module/DieModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DieModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `GameLogic/TerrainLogic.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/TerrainLogic.h`
  - `GameLogic/Weapon.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (5):**
  - `"MaxHealth" => INI::parseReal`
  - `"InitialHealth" => INI::parseReal`
  - `"SubdualDamageCap" => INI::parseReal`
  - `"SubdualDamageHealRate" => INI::parseDurationUnsignedInt`
  - `"SubdualDamageHealAmount" => INI::parseReal`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (19 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferReal( &m_currentHealth )`
  3. `xferReal( &m_currentSubdualDamage )`
  4. `xferReal( &m_prevHealth )`
  5. `xferReal( &m_maxHealth )`
  6. `xferReal( &m_initialHealth )`
  7. `xferUser( &m_curDamageState, sizeof( BodyDamageType ) )`
  8. `xferUnsignedInt( &m_nextDamageFXTime )`
  9. `xferUser( &m_lastDamageFXDone, sizeof( DamageType ) )`
  10. `xferSnapshot( &m_lastDamageInfo )`
  11. `xferUnsignedInt( &m_lastDamageTimestamp )`
  12. `xferUnsignedInt( &m_lastHealingTimestamp )`
  13. `xferBool( &m_frontCrushed )`
  14. `xferBool( &m_backCrushed )`
  15. `xferBool( &m_lastDamageCleared )`
  16. `xferBool( &m_indestructible )`
  17. `xferUnsignedShort( &particleSystemCount )`
  18. `xferUser( &system->m_particleSystemID, sizeof( ParticleSystemID ) )`
  19. `xferUser( &particleSystemID, sizeof( ParticleSystemID ) )`
- **PRNG Calls (2 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValue(0, 99) < 25) { AudioEventRTS fearSound = *obj->getTemplate()->getVoiceFear()`
  - `GameClientRandomValue( 0, maxSystems - i - 1 )`
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if RETAIL_COMPATIBLE_CRC`

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
- `zh_logic::combat::armor`: `ArmorTemplate`, `Armor`, `fn`, `ArmorStore`
- `zh_logic::combat::armor_set`: `ArmorTemplateSet`
- `zh_logic::combat::damage`: `PlayerMaskType`, `HUGE_DAMAGE_AMOUNT`, `DamageInfoInput`, `DamageInfoOutput`, `DamageInfo`
- `zh_logic::combat::damage_fx`: `DamageFxSlot`, `fn`, `DamageFX`, `DamageFXStore`
- `zh_logic::combat::firing_tracker`: `AHSV_NO_SOUND`, `FiringTrackerModuleData`, `FiredWeaponInfo`, `FiringTrackerHost`, `FiringTracker`, `fn`
- `zh_logic::combat::locomotor`: `BIGNUM`, `LocomotorAppearance`, `fn`, `LOCOMOTOR_APPEARANCE_NAMES`, `LocomotorPriority`, `LOCOMOTOR_PRIORITY_NAMES`, `LocomotorBehaviorZ`, `LOCOMOTOR_BEHAVIOR_Z_NAMES`, `LocomotorTemplate`, `LocomotorStore`, `Locomotor`
- `zh_logic::combat::locomotor_set`: `LOCOMOTORSET_COUNT`, `LocomotorSetType`, `fn`, `LOCOMOTOR_SET_NAMES`, `LocomotorSet`
- `zh_logic::combat::turret_ai`: `DEFAULT_TURN_RATE`, `DEFAULT_PITCH_RATE`, `WAIT_INDEFINITELY`, `WhichTurretType`, `MAX_TURRETS`, `fn`, `TurretStateType`, `NUM_STATES`, `TurretTargetType`, `TurretAIError`, `FireStateStep`, `TurretWorldAccess`, `TurretAIData`, `TurretStateMachine`, ... (+1)
- `zh_logic::economy`: `MAX_PLAYER_COUNT`
- `zh_logic::economy::academy_stats`: `MAX_ADVICE_TIPS`, `FRAMES_BETWEEN_UPDATES`, `DEFAULT_SUPPLY_CENTER_COST`, `KINDOF_DOZER`, `KINDOF_FS_SUPPLY_CENTER`, `KINDOF_COMMANDCENTER`, `KINDOF_INFANTRY`, `KINDOF_VEHICLE`, `KINDOF_HARVESTER`, `KINDOF_HERO`, `KINDOF_FS_STRATEGY_CENTER`, `KINDOF_MONEY_HACKER`, `KINDOF_FS_BLACK_MARKET`, `KINDOF_FS_SUPPLY_DROPZONE`, ... (+9)

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ActiveBody.h`
```cpp
// FILE: ActiveBody.h /////////////////////////////////////////////////////////////////////////////
// Author: Colin Day, November 2001
// Desc:   Active bodies have health, they can die and are affected by health
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/DamageFX.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Damage.h"
#include "GameLogic/Armor.h"
#include "GameLogic/ArmorSet.h"

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
class BodyParticleSystem;
class ParticleSystemTemplate;

//-------------------------------------------------------------------------------------------------
/** Active body module */
//-------------------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------------------
class ActiveBodyModuleData : public BodyModuleData
{
public:
	Real m_maxHealth;
	Real m_initialHealth;

	Real m_subdualDamageCap;								///< Subdual damage will never accumulate past this
	UnsignedInt m_subdualDamageHealRate;		///< Every this often, we drop subdual damage...
	Real m_subdualDamageHealAmount;					///< by this much.

	ActiveBodyModuleData();

	static void buildFieldParse(MultiIniFieldParse& p);
};

//-------------------------------------------------------------------------------------------------
class ActiveBody : public BodyModule
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( ActiveBody, "ActiveBody" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( ActiveBody, ActiveBodyModuleData )

public:

	ActiveBody( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	virtual void onDelete() override;

	virtual void attemptDamage( DamageInfo *damageInfo ) override;		///< try to damage this object
	virtual Real estimateDamage( DamageInfoInput& damageInfo ) const override;
	virtual void attemptHealing( DamageInfo *damageInfo ) override;		///< try to heal this object
	virtual Real getHealth() const override;													///< get current health
	virtual BodyDamageType getDamageState() const override;
	virtual void setDamageState( BodyDamageType newState ) override;	///< control damage state directly.  Will adjust hitpoints.
	virtual void setAflame( Bool setting ) override;///< This is a major change like a damage state.
	virtual UnsignedInt getSubdualDamageHealRate() const override;
	virtual Real getSubdualDamageHealAmount() const override;
	virtual Bool hasAnySubdualDamage() const override;
	virtual Real getCurrentSubdualDamageAmount() const override { return m_currentSubdualDamage; }

	virtual const DamageInfo *getLastDamageInfo() const override { return &m_lastDamageInfo; }	///< return info on last damage dealt to this object
	virtual const UnsignedInt *getLastDamageTimestamp() const override;	///< return frame of last damage, or nullptr if none
	virtual const UnsignedInt *getLastHealingTimestamp() const override;	///< return frame of last healing, or nullptr if none
	virtual ObjectID getClearableLastAttacker() const override { return (m_lastDamageCleared ? INVALID_ID : m_lastDamageInfo.in.m_sourceID); }
	virtual void clearLastAttacker() override { m_lastDamageCleared = true; }

	virtual void onVeterancyLevelChanged( VeterancyLevel oldLevel, VeterancyLevel newLevel, Bool provideFeedback = TRUE ) override;

	virtual void setArmorSetFlag(ArmorSetType ast) override { m_curArmorSetFlags.set(ast, 1); }
	virtual void clearArmorSetFlag(ArmorSetType ast) override { m_curArmorSetFlags.set(ast, 0); }
	virtual Bool testArmorSetFlag(ArmorSetType ast) override { return m_curArmorSetFlags.test(ast); }

	virtual void setInitialHealth(Int initialPercent) override; ///< Sets the initial load health %.
	virtual void setMaxHealth( Real maxHealth, MaxHealthChangeType healthChangeType = SAME_CURRENTHEALTH ) override; ///< Sets the initial max health

	virtual Bool getFrontCrushed() const override { return m_frontCrushed; }
	virtual Bool getBackCrushed() const override { return m_backCrushed; }

	virtual void setFrontCrushed(Bool v) override { m_frontCrushed = v; }
	virtual void setBackCrushed(Bool v) override { m_backCrushed = v; }

	virtual Real getMaxHealth() const override;  ///< return max health
	virtual Real getInitialHealth() const override;  // return initial health

	virtual Real getPreviousHealth() const override { return m_prevHealth; }

	virtual void setIndestructible( Bool indestructible ) override;
	virtual Bool isIndestructible() const override { return m_indestructible; }

	virtual void internalChangeHealth( Real delta ) override;								///< change health

	virtual void evaluateVisualCondition() override;
	virtual void updateBodyParticleSystems() override;// made public for topple anf building collapse updates -ML

	// Subdual Damage
	virtual Bool isSubdued() const;
	virtual Bool canBeSubdued() const;
	virtual void onSubdualChange( Bool isNowSubdued );///< Override this if you want a totally different effect than DISABLED_SUBDUED

protected:

	void validateArmorAndDamageFX() const;
	void doDamageFX( const DamageInfo *damageInfo );

	void createParticleSystems( const AsciiString &boneBaseName,
															const ParticleSystemTemplate *systemTemplate,
															Int maxSystems );
	void deleteAllParticleSystems();
	void setCorrectDamageState();

	Bool shouldRetaliate(Object *obj);
	Bool shouldRetaliateAgainstAggressor(Object *obj, Object *damager);

	virtual void internalAddSubdualDamage( Real delta );								///< change health

private:

	Real									m_currentHealth;				///< health of the object
	Real									m_prevHealth;						///< previous health value before current health change op
  Real									m_maxHealth;						///< max health this object can have
  Real									m_initialHealth;				///< starting health for this object
	Real									m_currentSubdualDamage;	///< Starts at zero and goes up.  Inherited modules will do something when "subdued".

	BodyDamageType				m_curDamageState;				///< last known damage state
	UnsignedInt						m_nextDamageFXTime;
	DamageType						m_lastDamageFXDone;
	DamageInfo						m_lastDamageInfo;				///< store the last DamageInfo object that we received
	UnsignedInt						m_lastDamageTimestamp; 	///< frame of last damage dealt
	UnsignedInt						m_lastHealingTimestamp; ///< frame of last healing dealt
	Bool									m_frontCrushed;
	Bool									m_backCrushed;
	Bool									m_lastDamageCleared;
	Bool									m_indestructible;				///< is this object indestructible?

	BodyParticleSystem *m_particleSystems;				///< particle systems created and attached to this object

	/*
		Note, you MUST call validateArmorAndDamageFX() before accessing these fields.
	*/
	ArmorSetFlags											m_curArmorSetFlags;
	mutable const ArmorTemplateSet*		m_curArmorSet;
	mutable Armor											m_curArmor;
	mutable const DamageFX*						m_curDamageFX;

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Body/ActiveBody.cpp`
```cpp
// FILE: ActiveBody.cpp ///////////////////////////////////////////////////////////////////////////
// Author: Colin Day, November 2001
// Desc:   Active bodies have health, they can die and are affected by health
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine
#include "Common/BitFlagsIO.h"
#include "Common/CRCDebug.h"
#include "Common/DamageFX.h"
#include "Common/Player.h"
#include "Common/GameState.h"
#include "Common/GlobalData.h"
#include "Common/PlayerList.h"
#include "Common/Team.h"
#include "Common/Thing.h"
#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"
#include "GameClient/ControlBar.h"
#include "GameClient/Drawable.h"
#include "GameClient/InGameUI.h"
#include "GameClient/ParticleSys.h"
#include "GameLogic/AI.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/Armor.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/Damage.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/TerrainLogic.h"
#include "GameLogic/Weapon.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/ActiveBody.h"
#include "GameLogic/Module/BridgeBehavior.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/DamageModule.h"
#include "GameLogic/Module/DieModule.h"



#define YELLOW_DAMAGE_PERCENT (0.25f)

constexpr const UnsignedInt InvalidBodyTimestamp = ~0u; ///< damage or healing timestamp that has not been recorded yet

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////

// ------------------------------------------------------------------------------------------------
/** Body particle systems are particle systems that are automatically created and attached
	* to an object as the damage state changes for that object.  We keep a list of these
	* so that when we transition from one state to another we can kill any old particle
	* systems that we need to before we create new ones */
// ------------------------------------------------------------------------------------------------
class BodyParticleSystem : public MemoryPoolObject
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( BodyParticleSystem, "BodyParticleSystem" )

public:

	ParticleSystemID m_particleSystemID;		///< the particle system ID
	BodyParticleSystem *m_next;							///< next particle system in this body module

};

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
BodyParticleSystem::~BodyParticleSystem()
{

}

///////////////////////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS ///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
static BodyDamageType calcDamageState(Real health, Real maxHealth)
{
	if (!TheGlobalData)
		return BODY_PRISTINE;

	Real ratio = health / maxHealth;

	if (ratio > TheGlobalData->m_unitDamagedThresh)
	{
		return BODY_PRISTINE;
	}
	else if (ratio > TheGlobalData->m_unitReallyDamagedThresh)
	{
		return BODY_DAMAGED;
	}
	else if (ratio > 0.0f)
	{
		return BODY_REALLYDAMAGED;
	}
	else
	{
		return BODY_RUBBLE;
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ActiveBodyModuleData::ActiveBodyModuleData()
{
	m_maxHealth = 0;
	m_initialHealth = 0;
	m_subdualDamageCap = 0;
	m_subdualDamageHealRate = 0;
	m_subdualDamageHealAmount = 0;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void ActiveBodyModuleData::buildFieldParse(MultiIniFieldParse& p)
{
  ModuleData::buildFieldParse(p);

	static const FieldParse dataFieldParse[] =
	{
		{ "MaxHealth",						INI::parseReal,						nullptr,		offsetof( ActiveBodyModuleData, m_maxHealth ) },
		{ "InitialHealth",				INI::parseReal,						nullptr,		offsetof( ActiveBodyModuleData, m_initialHealth ) },

		{ "SubdualDamageCap",					INI::parseReal,									nullptr,		offsetof( ActiveBodyModuleData, m_subdualDamageCap ) },
		{ "SubdualDamageHealRate",		INI::parseDurationUnsignedInt,	nullptr,		offsetof( ActiveBodyModuleData, m_subdualDamageHealRate ) },
		{ "SubdualDamageHealAmount",	INI::parseReal,									nullptr,		offsetof( ActiveBodyModuleData, m_subdualDamageHealAmount ) },
		{ nullptr, nullptr, nullptr, 0 }
	};
  p.add(dataFieldParse);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ActiveBody::ActiveBody( Thing *thing, const ModuleData* moduleData ) :
	BodyModule(thing, moduleData),
	m_curDamageFX(nullptr),
	m_curArmorSet(nullptr),
	m_frontCrushed(false),
	m_backCrushed(false),
	m_lastDamageTimestamp(InvalidBodyTimestamp),// So we don't think we just got damaged on the first frame
	m_lastHealingTimestamp(InvalidBodyTimestamp),// So we don't think we just got healed on the first frame
	m_curDamageState(BODY_PRISTINE),
	m_nextDamageFXTime(0),
	m_lastDamageFXDone((DamageType)-1),
	m_lastDamageCleared(false),
	m_particleSystems(nullptr),
	m_currentSubdualDamage(0),
	m_indestructible(false)
{
	m_currentHealth = getActiveBodyModuleData()->m_initialHealth;
	m_prevHealth = getActiveBodyModuleData()->m_initialHealth;
	m_maxHealth = getActiveBodyModuleData()->m_maxHealth;
	m_initialHealth = getActiveBodyModuleData()->m_initialHealth;

	// force an initially-valid armor setup
	validateArmorAndDamageFX();
	// start us in the right state
	setCorrectDamageState();

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ActiveBody::~ActiveBody()
{
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void ActiveBody::onDelete()
{

	// delete all particle systems
	deleteAllParticleSystems();

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void ActiveBody::setCorrectDamageState()
{
	m_curDamageState = calcDamageState(m_currentHealth, m_maxHealth);

	/// @todo srj -- bleah, this is an icky way to do it. oh well.
	if (m_curDamageState == BODY_RUBBLE && getObject()->isKindOf(KINDOF_STRUCTURE))
	{
		Real rubbleHeight = getObject()->getTemplate()->getStructureRubbleHeight();

		if (rubbleHeight <= 0.0f)
			rubbleHeight = TheGlobalData->m_defaultStructureRubbleHeight;

		/** @todo I had to change this to a Z only version to keep it from disappearing from the
			PartitionManager for a frame.  That didn't used to happen.
		*/
		getObject()->setGeometryInfoZ(rubbleHeight);

		// Have to tell pathfind as well, as rubble pathfinds differently.
		TheAI->pathfinder()->removeObjectFromPathfindMap(getObject());
		TheAI->pathfinder()->addObjectToPathfindMap(getObject());


		// here we make sure nobody collides with us, ever again...			//Lorenzen
		//THis allows projectiles shot from infantry that are inside rubble to get out of said rubble safely
		getObject()->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_NO_COLLISIONS ) );


	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void ActiveBody::setDamageState( BodyDamageType newState )
{
	Real ratio = 1.0f;
	if( newState == BODY_PRISTINE )
	{
		ratio = 1.0f;
	}
	else if( newState == BODY_DAMAGED )
	{
		ratio = TheGlobalData->m_unitDamagedThresh;
	}
	else if( newState == BODY_REALLYDAMAGED )
	{
		ratio = TheGlobalData->m_unitReallyDamagedThresh;
	}
	else if( newState == BODY_RUBBLE )
	{
		ratio = 0.0f;
	}
	Real desiredHealth = m_maxHealth * ratio - 1;// -1 because < not <= in calcState
	desiredHealth = max( desiredHealth, 0.0f );
	internalChangeHealth( desiredHealth - m_currentHealth );
	setCorrectDamageState();
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void ActiveBody::validateArmorAndDamageFX() const
{
	const ArmorTemplateSet* set = getObject()->getTemplate()->findArmorTemplateSet(m_curArmorSetFlags);
	DEBUG_ASSERTCRASH(set, ("findArmorSet should never return null"));
	if (set && set != m_curArmorSet)
	{
		if (set->getArmorTemplate())
		{
			m_curArmor = TheArmorStore->makeArmor(set->getArmorTemplate());
		}
		else
		{
			m_curArmor.clear();
		}
		m_curDamageFX = set->getDamageFX();
		m_curArmorSet = set;
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Real ActiveBody::estimateDamage( DamageInfoInput& damageInfo ) const
{
	validateArmorAndDamageFX();

	//Subdual damage can't affect you if you can't be subdued
	if( IsSubdualDamage(damageInfo.m_damageType)  &&  !canBeSubdued() )
		return 0.0f;

	if( damageInfo.m_damageType == DAMAGE_KILL_GARRISONED )
	{
		ContainModuleInterface* contain = getObject()->getContain();
		if( contain && contain->getContainCount() > 0 && contain->isGarrisonable() && !contain->isImmuneToClearBuildingAttacks() )
			return 1.0f;
		else
			return 0.0f;
	}

	if( damageInfo.m_damageType == DAMAGE_SNIPER )
	{
		if( getObject()->isKindOf( KINDOF_STRUCTURE ) && getObject()->testStatus( OBJECT_STATUS_UNDER_CONSTRUCTION ) )
		{
			//If we're a pathfinder shooting a stinger site under construction... don't. Special case code.
			return 0.0f;
		}
	}

	Real amount = m_curArmor.adjustDamage(damageInfo.m_damageType, damageInfo.m_amount);

	return amount;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void ActiveBody::doDamageFX( const DamageInfo *damageInfo )
{
	DamageType damageTypeToUse = damageInfo->in.m_damageType;
	if (damageInfo->in.m_damageFXOverride != DAMAGE_UNRESISTABLE )
	{
		// Just the visual aspect of damage can be overridden in some cases.
		// Unresistable is the default to mean no override, as we are out of bits.
		damageTypeToUse = damageInfo->in.m_damageFXOverride;
	}

	if (m_curDamageFX)
	{
		UnsignedInt now = TheGameLogic->getFrame();
		if (damageTypeToUse == m_lastDamageFXDone && m_nextDamageFXTime > now)
			return;
		Object *source = TheGameLogic->findObjectByID(damageInfo->in.m_sourceID);	// might be null, I guess
		m_lastDamageFXDone = damageTypeToUse;
		m_nextDamageFXTime = now + m_curDamageFX->getDamageFXThrottleTime(damageTypeToUse, source);
		m_curDamageFX->doDamageFX(damageTypeToUse, damageInfo->out.m_actualDamageDealt, source, getObject());
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void ActiveBody::attemptDamage( DamageInfo *damageInfo )
{
	validateArmorAndDamageFX();

	// sanity
	if( damageInfo == nullptr )
		return;

	if ( m_indestructible )
		return;

	// initialize these, just in case we bail out early
	damageInfo->out.m_actualDamageDealt = 0.0f;
	damageInfo->out.m_actualDamageClipped = 0.0f;

	// we cannot damage again objects that are already dead
	Object* obj = getObject();
	if( obj->isEffectivelyDead() )
		return;

	Object *damager = TheGameLogic->findObjectByID( damageInfo->in.m_sourceID );
	if( damager )
	{
		//Store the template so later if the attacking object dies, we use script conditions to look at the
		//damager's template inside evaluateTeamAttackedByType or evaluateNameAttackedByType.
		damageInfo->in.m_sourceTemplate = damager->getTemplate();
	}

	Bool alreadyHandled = FALSE;
	Bool allowModifier = TRUE;
	Real amount = m_curArmor.adjustDamage(damageInfo->in.m_damageType, damageInfo->in.m_amount);

	switch( damageInfo->in.m_damageType )
	{
		case DAMAGE_HEALING:
		{
			if( !damageInfo->in.m_kill )
			{
				// Healing and Damage are separate, so this shouldn't happen
				attemptHealing( damageInfo );
			}
			return;
		}

		case DAMAGE_KILLPILOT:
		{
			// This type of damage doesn't actually damage the unit, but it does kill it's
			// pilot, in the case of a vehicle.
			if( obj->isKindOf( KINDOF_VEHICLE ) )
			{
				//Handle special case for combat bike. We actually will kill the bike by
				//forcing the rider to leave the bike. That way the bike will automatically
				//scuttle and be unusable.
				ContainModuleInterface *contain = obj->getContain();
				if( contain && contain->isRiderChangeContain() )
				{

					AIUpdateInterface *ai = obj->getAI();

					if( ai->isMoving() )
					{
						//Bike is moving, so just blow it up instead.
						if (damager)
							damager->scoreTheKill( obj );
						obj->kill();
					}
					else
					{
						// TheSuperHackers @bugfix Caball009 04/09/2025 Check whether a bike still has a rider.
						// A rider may dismount or be sniped off a bike when it's disabled, resulting in a bike object with an empty contain list.
						if ( !contain->getContainedItemsList()->empty() )
						{
							//Removing the rider will scuttle the bike.
							Object* rider = *(contain->getContainedItemsList()->begin());
							ai->aiEvacuateInstantly(TRUE, CMD_FROM_AI);

							//Kill the rider.
							if (damager)
								damager->scoreTheKill(rider);
							rider->kill();
						}
					}
				}
				else
				{
					// Make it unmanned, so units can easily check the ability to "take control of it"
					obj->setDisabled( DISABLED_UNMANNED );
					TheGameLogic->deselectObject(obj, PLAYERMASK_ALL, TRUE);

          if ( obj->getAI() )
            obj->getAI()->aiIdle( CMD_FROM_AI );

					// Convert it to the neutral team so it renders gray giving visual representation that it is unmanned.
					obj->setTeam( ThePlayerList->getNeutralPlayer()->getDefaultTeam() );
				}

				//We don't care which team sniped the vehicle... we use this information to flag whether or not
				//we captured a vehicle.
	      ThePlayerList->getNeutralPlayer()->getAcademyStats()->recordVehicleSniped();
			}
			alreadyHandled = TRUE;
			allowModifier = FALSE;
			break;
		}

		case DAMAGE_KILL_GARRISONED:
		{
			// KRIS: READ THIS!!!
			// This code is very misleading (but in a good way). One would think this is
			// an excellent place to add the hook to kill garrisoned troops. And that is
			// a correct assumption. Unfortunately, the vast majority of garrison slayings
			// are performed in DumbProjectileBehavior::projectileHandleCollision(), so my
			// hope is that this message will save you some research time!

			Int killsToMake = REAL_TO_INT_FLOOR(damageInfo->in.m_amount);
			ContainModuleInterface* contain = obj->getContain();
			if( contain && contain->getContainCount() > 0 && contain->isGarrisonable() && !contain->isImmuneToClearBuildingAttacks() )
			{
				Int numKilled = 0;

				// garrisonable buildings subvert the normal process here.
				const ContainedItemsList* items = contain->getContainedItemsList();
				if (items)
				{
					for( ContainedItemsList::const_iterator it = items->begin(); (it != items->end()) && (numKilled < killsToMake); it++ )
					{
						Object* thingToKill = *it;
						if (!thingToKill->isEffectivelyDead() )
						{
							if (damager)
								damager->scoreTheKill( thingToKill );
							thingToKill->kill();
							++numKilled;
							thingToKill->getControllingPlayer()->getAcademyStats()->recordClearedGarrisonedBuilding();
						}
					}

				}
			}
			alreadyHandled = TRUE;
			allowModifier = FALSE;
			break;
		}

		case DAMAGE_STATUS:
		{
			// Damage amount is msec time we set the status given in damageStatusType
			Real realFramesToStatusFor = ConvertDurationFromMsecsToFrames(amount);
			obj->doStatusDamage( damageInfo->in.m_damageStatusType , REAL_TO_INT_CEIL(realFramesToStatusFor) );
			alreadyHandled = TRUE;
			allowModifier = FALSE;
			break;
		}
	}

	if( IsSubdualDamage(damageInfo->in.m_damageType) )
	{
		if( !canBeSubdued() )
			return;

		// TheSuperHackers @bugfix Stubbjax 20/09/2025 The isSubdued() function now directly checks status instead
		// of health to prevent indefinite subdue status when internally shifting health across the threshold.
		Bool wasSubdued = isSubdued();
		internalAddSubdualDamage(amount);
		Bool nowSubdued = m_maxHealth <= m_currentSubdualDamage;
		alreadyHandled = TRUE;
		allowModifier = FALSE;

		if( wasSubdued != nowSubdued )
		{
			onSubdualChange(nowSubdued);
		}

		getObject()->notifySubdualDamage(amount);
	}

	if (allowModifier)
	{
		if( damageInfo->in.m_damageType != DAMAGE_UNRESISTABLE )
		{
			// Apply the damage scalar (extra bonuses -- like strategy center defensive battle plan)
			// And remember not to adjust unresistable damage, just like the armor code can't.
			amount *= m_damageScalar;
		}
	}

	// sanity check the damage value -- can't apply negative damage
	if( amount > 0.0f || damageInfo->in.m_kill )
	{
		BodyDamageType oldState = m_curDamageState;

		//If the object is going to die, make sure we damage all remaining health.
		if( damageInfo->in.m_kill )
		{
			amount = m_currentHealth;
		}

		if (!alreadyHandled)
		{
			// do the damage simplistic damage subtraction
			internalChangeHealth( -amount );
		}

#ifdef ALLOW_SURRENDER
//*****************************************************************************************
//*****************************************************************************************
//THIS CODE HAS BEEN DISABLED FOR THE MULTIPLAYER PLAY TEST!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!**
//*****************************************************************************************
//		// if we were "killed" by surrender damage...
//		if (damageInfo->in.m_damageType == DAMAGE_SURRENDER && m_currentHealth <= 0.0f && obj->isKindOf(KINDOF_CAN_SURRENDER))
//		{
//			AIUpdateInterface* ai = obj->getAIUpdateInterface();
//			if (ai)
//			{
//				// do no damage, but make it surrender instead.
//				m_currentHealth = m_prevHealth;
//				const Object* killer = TheGameLogic->findObjectByID( damageInfo->in.m_sourceID );
//				ai->setSurrendered(killer, true);
//				return;
//			}
//		}
//*****************************************************************************************
//*****************************************************************************************
#endif

		// record the actual damage done from this, and when it happened
		damageInfo->out.m_actualDamageDealt = amount;
		damageInfo->out.m_actualDamageClipped = m_prevHealth - m_currentHealth;

		// then copy the whole DamageInfo struct for easy lookup
		// (object pointer loses scope as soon as atteptdamage's caller ends)
		// m_lastDamageTimestamp is initialized to FFFFFFFFFF, so doing a < compare is problematic.
		// jba.
		if (m_lastDamageTimestamp!=TheGameLogic->getFrame() && m_lastDamageTimestamp != TheGameLogic->getFrame()-1) {
			m_lastDamageInfo = *damageInfo;
			m_lastDamageCleared = false;
			m_lastDamageTimestamp = TheGameLogic->getFrame();
		} else {
			// Multiple damages applied in one/next frame.  We prefer the one that tells who the attacker is.
			Object *srcObj1 = TheGameLogic->findObjectByID(m_lastDamageInfo.in.m_sourceID);
			Object *srcObj2 = TheGameLogic->findObjectByID(damageInfo->in.m_sourceID);
			if (srcObj2) {
				if (srcObj1) {
					if (srcObj2->isKindOf(KINDOF_VEHICLE) || srcObj2->isKindOf(KINDOF_INFANTRY) ||
						srcObj2->isFactionStructure()) {
							m_lastDamageInfo = *damageInfo;
							m_lastDamageCleared = false;
							m_lastDamageTimestamp = TheGameLogic->getFrame();
						}
				} else {
					m_lastDamageInfo = *damageInfo;
					m_lastDamageCleared = false;
					m_lastDamageTimestamp = TheGameLogic->getFrame();
				}

			}	else {
				// no change.
			}
		}

		// Notify the player that they have been attacked by this player
		if (m_lastDamageInfo.in.m_sourceID != INVALID_ID)
		{
			Object *srcObj = TheGameLogic->findObjectByID(m_lastDamageInfo.in.m_sourceID);
			if (srcObj)
			{
				Player *srcPlayer = srcObj->getControllingPlayer();
				obj->getControllingPlayer()->setAttackedBy(srcPlayer->getPlayerIndex());
			}
		}

		// if our health has gone down then do run the damage module callback
		if( m_currentHealth < m_prevHealth )
		{
			for (BehaviorModule** m = obj->getBehaviorModules(); *m; ++m)
			{
				DamageModuleInterface* d = (*m)->getDamage();
				if (!d)
					continue;

				d->onDamage( damageInfo );
			}
		}

		if (m_curDamageState != oldState)
		{
			for (BehaviorModule** m = obj->getBehaviorModules(); *m; ++m)
			{
				DamageModuleInterface* d = (*m)->getDamage();
				if (!d)
					continue;

				d->onBodyDamageStateChange( damageInfo, oldState, m_curDamageState );
			}

			// @todo: This really feels like it should be in the TransitionFX lists.
			if (m_curDamageState == BODY_DAMAGED)
			{
				AudioEventRTS damaged = *obj->getTemplate()->getSoundOnDamaged();
				damaged.setObjectID(obj->getID());
				TheAudio->addAudioEvent(&damaged);
			}
			else if (m_curDamageState == BODY_REALLYDAMAGED)
			{
				AudioEventRTS reallyDamaged = *obj->getTemplate()->getSoundOnReallyDamaged();
				reallyDamaged.setObjectID(obj->getID());
				TheAudio->addAudioEvent(&reallyDamaged);
			}

		}

		// Should we play our fear sound?
		if( (m_prevHealth / m_maxHealth) > YELLOW_DAMAGE_PERCENT &&
				(m_currentHealth / m_maxHealth) < YELLOW_DAMAGE_PERCENT &&
				(m_currentHealth > 0) )
		{
			// 25% chance to play
			if (GameLogicRandomValue(0, 99) < 25)
			{
				AudioEventRTS fearSound = *obj->getTemplate()->getVoiceFear();
				fearSound.setPosition( obj->getPosition() );
				fearSound.setPlayerIndex( obj->getControllingPlayer()->getPlayerIndex() );
				TheAudio->addAudioEvent(&fearSound);
			}
		}

		// check to see if we died
		if( m_currentHealth <= 0 && m_prevHealth > 0 )
		{
			// Give our killer credit for killing us, if there is one.
			if( damager )
			{
				damager->scoreTheKill( obj );
			}

			obj->onDie( damageInfo );
		}
	}

	doDamageFX(damageInfo);

	// Damaged repulsable civilians scare (repulse) other civs.	jba.
	if( TheAI->getAiData()->m_enableRepulsors )
	{
		if( obj->isKindOf( KINDOF_CAN_BE_REPULSED ) )
		{
			obj->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_REPULSOR ) );
		}
	}

	//Retaliate, even if I'm dead -- we'll still get my nearby friends to get revenge!!!
	//Also only retaliate if we're controlled by a human player and the thing that attacked me
	//is an enemy.
	Player *controllingPlayer = obj->getControllingPlayer();
	if( controllingPlayer && controllingPlayer->isLogicalRetaliationModeEnabled() && controllingPlayer->getPlayerType() == PLAYER_HUMAN )
	{
		if( shouldRetaliateAgainstAggressor(obj, damager))
		{
			PartitionFilterPlayerAffiliation f1( controllingPlayer, ALLOW_ALLIES, true );
			PartitionFilterOnMap filterMapStatus;
			PartitionFilter *filters[] = { &f1, &filterMapStatus, nullptr };


			Real distance = TheAI->getAiData()->m_retaliateFriendsRadius + obj->getGeometryInfo().getBoundingCircleRadius();
			SimpleObjectIterator *iter = ThePartitionManager->iterateObjectsInRange( obj->getPosition(), distance, FROM_CENTER_2D, filters, ITER_FASTEST );
			MemoryPoolObjectHolder hold( iter );
			for( Object *them = iter->first(); them; them = iter->next() )
			{
				if (!shouldRetaliate(them)) {
					continue;
				}
				AIUpdateInterface *ai = them->getAI();
				if (ai==nullptr) {
					continue;
				}
				//If we have AI and we're mobile, then assist!
				if( !them->isKindOf( KINDOF_IMMOBILE ))
				{
					//But only if we can attack it!
					CanAttackResult result = them->getAbleToAttackSpecificObject( ATTACK_NEW_TARGET, damager, CMD_FROM_AI );
					if( result == ATTACKRESULT_POSSIBLE_AFTER_MOVING || result == ATTACKRESULT_POSSIBLE )
					{
						ai->aiGuardRetaliate( damager, them->getPosition(), NO_MAX_SHOTS_LIMIT, CMD_FROM_AI );
					}
				}
			}
		}
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Bool ActiveBody::shouldRetaliateAgainstAggressor(Object *obj, Object *damager)
{
	/* This considers whether obj should invoke his friends to retaliate against damager.
		 Note that obj could be a structure, so we don't actually check whether obj will
		 retaliate, as in many cases he wouldn't. */
	if (damager==nullptr) {
		return false;
	}
	if (damager->isAirborneTarget()) {
		return false; // Don't retaliate against aircraft. [8/25/2003]
	}
	if (damager->getRelationship( obj ) != ENEMIES) {
		return false; // only retaliate against enemies.
	}
	Real distSqr = ThePartitionManager->getDistanceSquared(obj, damager, FROM_BOUNDINGSPHERE_2D);
	if (distSqr > sqr(TheAI->getAiData()->m_maxRetaliateDistance)) {
		return false;
	}
	// Only human players retaliate. [8/25/2003]
	if (obj->getControllingPlayer()->getPlayerType() != PLAYER_HUMAN) {
		return false;
	}
	// Drones never retaliate. [8/25/2003]
	if (obj->isKindOf(KINDOF_DRONE)) {
		return false;
	}
	return true;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Bool ActiveBody::shouldRetaliate(Object *obj)
{
	// Cannot retaliate objects dont. [8/25/2003]
	if (obj->isKindOf(KINDOF_CANNOT_RETALIATE)) {
		return false;
	}
	if (obj->isKindOf( KINDOF_IMMOBILE )) {
		return false;
	}
	// Drones never retaliate. [8/25/2003]
	if (obj->isKindOf(KINDOF_DRONE)) {
		return false;
	}
	// Any unit that isn't idle won't retaliate. [8/25/2003]
	if (obj->getAI()) {
		if (!obj->getAI()->isIdle()) {
			return false;
		}
	} else {
		return false; // Non-ai can't retaliate. [8/26/2003]
	}
	// Stealthed units don't retaliate unless they're detected. [8/25/2003]
	if ( obj->getStatusBits().test( OBJECT_STATUS_STEALTHED ) &&
		!obj->getStatusBits().test( OBJECT_STATUS_DETECTED ) ) {
		return false;
	}
	// If we're using an ability, don't stop. [8/25/2003]
	if (obj->testStatus(OBJECT_STATUS_IS_USING_ABILITY)) {
		return false;
	}
	return true;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void ActiveBody::attemptHealing( DamageInfo *damageInfo )
{
	validateArmorAndDamageFX();

	// sanity
	if( damageInfo == nullptr )
		return;

	if( damageInfo->in.m_damageType != DAMAGE_HEALING )
	{
		// Healing and Damage are separate, so this shouldn't happen
		attemptDamage( damageInfo );
		return;
	}

	Object* obj = getObject();

	// srj sez: sorry, once yer dead, yer dead.
	// Special case for bridges, cause the system now things they're dead
	///@todo we need to figure out what has changed so we don't have to hack this (CBD 11-1-2002)
	if( obj->isKindOf( KINDOF_BRIDGE ) == FALSE &&
			obj->isKindOf( KINDOF_BRIDGE_TOWER ) == FALSE &&
			obj->isEffectivelyDead())
		return;

	// initialize these, just in case we bail out early
	damageInfo->out.m_actualDamageDealt = 0.0f;
	damageInfo->out.m_actualDamageClipped = 0.0f;

	Real amount = m_curArmor.adjustDamage(damageInfo->in.m_damageType, damageInfo->in.m_amount);

	// sanity check the damage value -- can't apply negative healing
	if( amount > 0.0f )
	{
		BodyDamageType oldState = m_curDamageState;

		// do the damage simplistic damage ADDITION
		internalChangeHealth( amount );

		// record the actual damage done from this, and when it happened
		damageInfo->out.m_actualDamageDealt = amount;
		damageInfo->out.m_actualDamageClipped = m_prevHealth - m_currentHealth;

		//then copy the whole DamageInfo struct for easy lookup
		//(object pointer loses scope as soon as atteptdamage's caller ends)
		m_lastDamageInfo = *damageInfo;
		m_lastDamageCleared = false;
#if PRESERVE_STRUCTURE_STEALTH_DURING_REPAIR
		m_lastDamageTimestamp = TheGameLogic->getFrame();
#endif
		m_lastHealingTimestamp = TheGameLogic->getFrame();

		// if our health has gone UP then do run the damage module callback
		if( m_currentHealth > m_prevHealth )
		{
			for (BehaviorModule** m = obj->getBehaviorModules(); *m; ++m)
			{
				DamageModuleInterface* d = (*m)->getDamage();
				if (!d)
					continue;

				d->onHealing( damageInfo );
			}
		}

		if (m_curDamageState != oldState)
		{
			for (BehaviorModule** m = obj->getBehaviorModules(); *m; ++m)
			{
				DamageModuleInterface* d = (*m)->getDamage();
				if (!d)
					continue;

				d->onBodyDamageStateChange( damageInfo, oldState, m_curDamageState );
			}
		}
	}

	doDamageFX(damageInfo);
}

//-------------------------------------------------------------------------------------------------
/** Simple setting of the health value, it does *NOT* track any transition
	* states for the event of "damage" or the event of "death".  */
//-------------------------------------------------------------------------------------------------
void ActiveBody::setInitialHealth(Int initialPercent)
{

	// save the current health as the previous health
	m_prevHealth = m_currentHealth;

	Real factor = initialPercent/100.0f;
	Real newHealth = factor * m_initialHealth;

	// change the health to the requested percentage.
	internalChangeHealth(newHealth - m_currentHealth);

}

//-------------------------------------------------------------------------------------------------
/** Simple setting of the health value, it does *NOT* track any transition
	* states for the event of "damage" or the event of "death".  */
//-------------------------------------------------------------------------------------------------
void ActiveBody::setMaxHealth( Real maxHealth, MaxHealthChangeType healthChangeType )
{
	Real prevMaxHealth = m_maxHealth;
	m_maxHealth = maxHealth;
	m_initialHealth = maxHealth;

	switch( healthChangeType )
	{
		case PRESERVE_RATIO:
		{
			//400/500 (80%) + 100 becomes 480/600 (80%)
			//200/500 (40%) - 100 becomes 160/400 (40%)
			Real ratio = WWMath::Div_Safe(m_currentHealth, prevMaxHealth);
			Real newHealth = maxHealth * ratio;
			internalChangeHealth( newHealth - m_currentHealth );
			break;
		}
		case ADD_CURRENT_HEALTH_TOO:
		{
			//Add the same amount that we are adding to the max health.
			//This could kill you if max health is reduced (if we ever have that ability to add buffer health like in D&D)
			//400/500 (80%) + 100 becomes 500/600 (83%)
			//200/500 (40%) - 100 becomes 100/400 (25%)
			internalChangeHealth( maxHealth - prevMaxHealth );
			break;
		}
		case SAME_CURRENTHEALTH:
			//do nothing
			break;

		case FULLY_HEAL:
		{
			// Set current to the new Max.
			//400/500 (80%) + 100 becomes 600/600 (100%)
			//200/500 (40%) - 100 becomes 400/400 (100%)
			internalChangeHealth(m_maxHealth - m_currentHealth);
			break;
		}
	}

	//
	// when max health is getting clipped to a lower value, if our current health
	// value is now outside of the max health range we will set it back down to the
	// new cap.  Note that we are *NOT* going through any healing or damage methods here
	// and are doing a direct set
	//
	if( m_currentHealth > maxHealth )
	{
		internalChangeHealth( maxHealth - m_currentHealth );
	}

}

// ------------------------------------------------------------------------------------------------
/** Given the current damage state of the object, evaluate the visual model conditions
	* that have a visual impact on the object */
// ------------------------------------------------------------------------------------------------
void ActiveBody::evaluateVisualCondition()
{

	Drawable* draw = getObject()->getDrawable();
	if (draw)
	{
		draw->reactToBodyDamageStateChange(m_curDamageState);
	}

	//
	// destroy any particle systems that were attached to our body for the old state
	// and create new particle systems for the new state
	//
	updateBodyParticleSystems();

}

// ------------------------------------------------------------------------------------------------
/** Create up to maxSystems particle systems of type particleSystemName and attach to bones
	* specified by the bone base name.  If there are more bones than maxSystems then the
	* bones will be randomly selected */
// ------------------------------------------------------------------------------------------------
void ActiveBody::createParticleSystems( const AsciiString &boneBaseName,
																				const ParticleSystemTemplate *systemTemplate,
																				Int maxSystems )
{
	Object *us = getObject();

	// sanity
	if( systemTemplate == nullptr )
		return;

	// get the bones
	enum { MAX_BONES = 16 };
	Coord3D bonePositions[ MAX_BONES ];
	Int numBones = us->getMultiLogicalBonePosition( boneBaseName.str(),
																									MAX_BONES,
																									bonePositions,
																									nullptr,
																									FALSE );

	// if no bones found nothing else to do
	if( numBones == 0 )
		return;

	//
	// if we don't have enough bones to go up to maxSystems, we will change maxSystems to be
	// the number of bones we actually have (we don't want systems doubling up on bones)
	//
	if( numBones < maxSystems )
		maxSystems = numBones;

	//
	// create an array that we'll use to mark which bone positions have already been used,
	// this is necessary when we have more bones than particle systems we're going to
	// create, in which case we place the particle systems at random bone locations
	// but don't want to repeat any
	//
	Bool usedBoneIndices[ MAX_BONES ] = { FALSE };

	// create the particle systems
	const Coord3D *pos;
	for( Int i = 0; i < maxSystems; ++i )
	{

		// pick a bone index to place this particle system at
		// MDC: moving to GameLogicRandomValue.  This does not need to be synced, but having it so makes searches *so* much nicer.
		// DTEH: Moved back to GameClientRandomValue because of desync problems. July 27th 2003.
		Int boneIndex = GameClientRandomValue( 0, maxSystems - i - 1 );

		// find the actual bone location to use and mark that bone index as used
		Int count = 0;
		Int j = 0;
		for( ; j < numBones; j++ )
		{

			// ignore bone positions that have already been used
			if( usedBoneIndices[ j ] == TRUE )
				continue;

			// this spot is available, if count == boneIndex then use this index
			if( count == boneIndex )
			{

				pos = &bonePositions[ j ];
				usedBoneIndices[ j ] = TRUE;
				break;  // exit for j

			}
			else
			{

				// we won't use this index, increment count until we find a suitable index to use
				++count;

			}

		}

		// sanity
		DEBUG_ASSERTCRASH( j != numBones,
											 ("ActiveBody::createParticleSystems, Unable to select particle system index") );

		// create particle system here
		ParticleSystem *particleSystem = TheParticleSystemManager->createParticleSystem( systemTemplate );
		if( particleSystem )
		{

			// set the position of the particle system in local object space
			particleSystem->setPosition( pos );

			// attach particle system to object
			particleSystem->attachToObject( us );

			// create a new body particle system entry and keep this particle system in it
			BodyParticleSystem *newEntry = newInstance(BodyParticleSystem);
			newEntry->m_particleSystemID = particleSystem->getSystemID();
			newEntry->m_next = m_particleSystems;
			m_particleSystems = newEntry;

		}

	}

}

// ------------------------------------------------------------------------------------------------
/** Delete all the body particle systems */
// ------------------------------------------------------------------------------------------------
void ActiveBody::deleteAllParticleSystems()
{
	BodyParticleSystem *nextBodySystem;
	ParticleSystem *particleSystem;

	while( m_particleSystems )
	{

		// get this particle system
		particleSystem = TheParticleSystemManager->findParticleSystem( m_particleSystems->m_particleSystemID );
		if( particleSystem )
			particleSystem->destroy();

		// get next system in the body
		nextBodySystem = m_particleSystems->m_next;

		// destroy this entry
		deleteInstance(m_particleSystems);

		// set the body systems head to the next
		m_particleSystems = nextBodySystem;

	}

}

// ------------------------------------------------------------------------------------------------
/* 	This function is called on state changes only.  Body Type or Aflameness. */
// ------------------------------------------------------------------------------------------------
void ActiveBody::updateBodyParticleSystems()
{
	static const ParticleSystemTemplate *fireSmallTemplate   = TheParticleSystemManager->findTemplate( TheGlobalData->m_autoFireParticleSmallSystem );
	static const ParticleSystemTemplate *fireMediumTemplate  = TheParticleSystemManager->findTemplate( TheGlobalData->m_autoFireParticleMediumSystem );
	static const ParticleSystemTemplate *fireLargeTemplate   = TheParticleSystemManager->findTemplate( TheGlobalData->m_autoFireParticleLargeSystem );
	static const ParticleSystemTemplate *smokeSmallTemplate  = TheParticleSystemManager->findTemplate( TheGlobalData->m_autoSmokeParticleSmallSystem );
	static const ParticleSystemTemplate *smokeMediumTemplate = TheParticleSystemManager->findTemplate( TheGlobalData->m_autoSmokeParticleMediumSystem );
	static const ParticleSystemTemplate *smokeLargeTemplate  = TheParticleSystemManager->findTemplate( TheGlobalData->m_autoSmokeParticleLargeSystem );
	static const ParticleSystemTemplate *aflameTemplate			 = TheParticleSystemManager->findTemplate( TheGlobalData->m_autoAflameParticleSystem );
	Int countModifier;
	const ParticleSystemTemplate *fireSmall;
	const ParticleSystemTemplate *fireMedium;
	const ParticleSystemTemplate *fireLarge;
	const ParticleSystemTemplate *smokeSmall;
	const ParticleSystemTemplate *smokeMedium;
	const ParticleSystemTemplate *smokeLarge;

	//
	// when we're aflame, we use a slightly different set of particle systems that are
	// auto created that lends itself to more fire and bigger fire
	//
	if( getObject()->testStatus( OBJECT_STATUS_AFLAME ) )
	{

		fireSmall		= fireMediumTemplate;		// small fire becomes medium fire
		fireMedium	= fireLargeTemplate;		// medium fire becomes large fire
		fireLarge		= fireLargeTemplate;		// large fire stays large
		smokeSmall	= fireSmallTemplate;		// small smoke becomes small fire
		smokeMedium = fireSmallTemplate;		// medium smoke becomes small fire
		smokeLarge	= fireSmallTemplate;		// large smoke becomes small fire

		// we get to make more of them all too
		countModifier = 2;

	}
	else
	{

		// use regular templates
		fireSmall		= fireSmallTemplate;
		fireMedium	= fireMediumTemplate;
		fireLarge		= fireLargeTemplate;
		smokeSmall	= smokeSmallTemplate;
		smokeMedium = smokeMediumTemplate;
		smokeLarge	= smokeLargeTemplate;

		// we make just the normal amount of these
		countModifier = 1;

	}

	//
	// remove any particle systems we have currently in the list in favor of any new ones
	// that we're going to autopopulate ourselves with
	//
	deleteAllParticleSystems();

	//
	// create particle systems for the new body state
	//

	// small fire bones
	createParticleSystems( TheGlobalData->m_autoFireParticleSmallPrefix,
												 fireSmall, TheGlobalData->m_autoFireParticleSmallMax * countModifier );

	// medium fire bones
	createParticleSystems( TheGlobalData->m_autoFireParticleMediumPrefix,
												 fireMedium, TheGlobalData->m_autoFireParticleMediumMax * countModifier );

	// large fire bones
	createParticleSystems( TheGlobalData->m_autoFireParticleLargePrefix,
												 fireLarge, TheGlobalData->m_autoFireParticleLargeMax * countModifier );

	// small smoke bones
	createParticleSystems( TheGlobalData->m_autoSmokeParticleSmallPrefix,
												 smokeSmall, TheGlobalData->m_autoSmokeParticleSmallMax * countModifier );

	// medium smoke bones
	createParticleSystems( TheGlobalData->m_autoSmokeParticleMediumPrefix,
												 smokeMedium, TheGlobalData->m_autoSmokeParticleMediumMax * countModifier );

	// large smoke bones
	createParticleSystems( TheGlobalData->m_autoSmokeParticleLargePrefix,
												 smokeLarge, TheGlobalData->m_autoSmokeParticleLargeMax * countModifier );

	// actively on fire
	if( getObject()->testStatus( OBJECT_STATUS_AFLAME ) )
		createParticleSystems( TheGlobalData->m_autoAflameParticlePrefix,
													 aflameTemplate, TheGlobalData->m_autoAflameParticleMax * countModifier );

}

//-------------------------------------------------------------------------------------------------
/** Simple changing of the health value, it does *NOT* track any transition
	* states for the event of "damage" or the event of "death".  If you
	* with to kill an object and give these modules a chance to react
	* to that event use the proper damage method calls.
	* No game logic should go in here.  This is the low level math and flag maintenance.
	* Game stuff goes in attemptDamage and attemptHealing.
*/
//-------------------------------------------------------------------------------------------------
void ActiveBody::internalChangeHealth( Real delta )
{
	// save the current health as the previous health
	m_prevHealth = m_currentHealth;

	// change the health by the delta, it can be positive or negative
	m_currentHealth += delta;

	// high end cap
	Real maxHealth = m_maxHealth;
	if( m_currentHealth > maxHealth )
		m_currentHealth = maxHealth;

	// low end cap
	const Real lowEndCap = 0.0f;  // low end cap for health, don't go below this
	if( m_currentHealth < lowEndCap )
		m_currentHealth = lowEndCap;

	// recalc the damage state
	BodyDamageType oldState = m_curDamageState;
	setCorrectDamageState();

	// if our state has changed
	if( m_curDamageState != oldState )
	{

		//
		// show a visual change in the model for the damage state, we do not show visual changes
		// for damage states when things are under construction because we just don't have
		// all the art states for that during buildup animation
		//
		if( !getObject()->getStatusBits().test( OBJECT_STATUS_UNDER_CONSTRUCTION ) )
			evaluateVisualCondition();

	}

	// mark the bit according to our health. (if our AI is dead but our health improves, it will
	// still re-flag this bit in the AIDeadState every frame.)
	getObject()->setEffectivelyDead(m_currentHealth <= 0);

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void ActiveBody::internalAddSubdualDamage( Real delta )
{
	const ActiveBodyModuleData *data = getActiveBodyModuleData();

	m_currentSubdualDamage += delta;
#if RETAIL_COMPATIBLE_CRC
	m_currentSubdualDamage = min(m_currentSubdualDamage, data->m_subdualDamageCap);
#else
	// TheSuperHackers @bugfix Stubbjax 25/01/2026 Subdual damage can no longer go negative, which
	// stops weak subdual damage + rapid healing from negatively stacking subdual damage over time.
	m_currentSubdualDamage = clamp(0.0f, m_currentSubdualDamage, data->m_subdualDamageCap);
#endif
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Bool ActiveBody::canBeSubdued() const
{
	// Any body with subdue listings can be subdued.
	return getActiveBodyModuleData()->m_subdualDamageCap > 0;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void ActiveBody::onSubdualChange( Bool isNowSubdued )
{
	if( !getObject()->isKindOf(KINDOF_PROJECTILE) )
	{
		Object *me = getObject();

		if( isNowSubdued )
		{
			me->setDisabled(DISABLED_SUBDUED);

      ContainModuleInterface *contain = me->getContain();
      if ( contain )
        contain->orderAllPassengersToIdle( CMD_FROM_AI );

		}
		else
		{
			me->clearDisabled(DISABLED_SUBDUED);

			if( me->isKindOf( KINDOF_FS_INTERNET_CENTER ) )
			{
				//Kris: October 20, 2003 - Patch 1.01
				//Any unit inside an internet center is a hacker! Order
				//them to start hacking again.
				ContainModuleInterface *contain = me->getContain();
				if ( contain )
					contain->orderAllPassengersToHackInternet( CMD_FROM_AI );
			}
		}
	}
	else if( isNowSubdued )// There is no coming back from being jammed, and projectiles can't even heal, but this makes it clear.
	{
		ProjectileUpdateInterface *pui = getObject()->getProjectileUpdateInterface();
		if( pui )
		{
			pui->projectileNowJammed();
		}
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Bool ActiveBody::isSubdued() const
{
#if RETAIL_COMPATIBLE_CRC
	return m_maxHealth <= m_currentSubdualDamage;
#else
  // TheSuperHackers @info Projectiles don't receive the DISABLED_SUBDUED flag (or any flag for
	// that matter) when jammed, so we have to check their subdual damage directly.
	if (getObject()->isKindOf(KINDOF_PROJECTILE))
		return m_maxHealth <= m_currentSubdualDamage;

	return getObject()->isDisabledByType(DISABLED_SUBDUED);
#endif
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
const UnsignedInt *ActiveBody::getLastDamageTimestamp() const
{
#if RETAIL_COMPATIBLE_CRC
	return &m_lastDamageTimestamp;
#else
	// TheSuperHackers @fix WebbontheWeb 03/10/2026 Avoid treating never-damaged sentinels as an attack timestamp.
	return m_lastDamageTimestamp != InvalidBodyTimestamp ? &m_lastDamageTimestamp : nullptr;
#endif
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
const UnsignedInt *ActiveBody::getLastHealingTimestamp() const
{
#if RETAIL_COMPATIBLE_CRC
	return &m_lastHealingTimestamp;
#else
	// TheSuperHackers @fix WebbontheWeb 04/10/2026 Avoid treating never-healed sentinels as a healing timestamp.
	return m_lastHealingTimestamp != InvalidBodyTimestamp ? &m_lastHealingTimestamp : nullptr;
#endif
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Real ActiveBody::getHealth() const
{
	return m_currentHealth;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
BodyDamageType ActiveBody::getDamageState() const
{
	return m_curDamageState;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Real ActiveBody::getMaxHealth() const
{
	return m_maxHealth;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
UnsignedInt ActiveBody::getSubdualDamageHealRate() const
{
	return getActiveBodyModuleData()->m_subdualDamageHealRate;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Real ActiveBody::getSubdualDamageHealAmount() const
{
	return getActiveBodyModuleData()->m_subdualDamageHealAmount;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Bool ActiveBody::hasAnySubdualDamage() const
{
	return m_currentSubdualDamage > 0;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Real ActiveBody::getInitialHealth() const
{
	return m_initialHealth;
}


// ------------------------------------------------------------------------------------------------
/** Set or unset the overridable indestructible flag in the body */
// ------------------------------------------------------------------------------------------------
void ActiveBody::setIndestructible( Bool indestructible )
{

	m_indestructible = indestructible;

	// for bridges, we mirror this state on its towers
	Object *us = getObject();
	if( us->isKindOf( KINDOF_BRIDGE ) )
	{
		BridgeBehaviorInterface *bbi = BridgeBehavior::getBridgeBehaviorInterfaceFromObject( us );
		if( bbi )
		{
			Object *tower;

			// get tower
			for( Int i = 0; i < BRIDGE_MAX_TOWERS; ++i )
			{

				tower = TheGameLogic->findObjectByID( bbi->getTowerID( BridgeTowerType(i) ) );
				if( tower )
				{
					BodyModuleInterface *body = tower->getBodyModule();

					if( body )
						body->setIndestructible( indestructible );

				}

			}

		}

	}

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void ActiveBody::onVeterancyLevelChanged( VeterancyLevel oldLevel, VeterancyLevel newLevel, Bool provideFeedback )
{
	if (oldLevel == newLevel)
		return;

	if (oldLevel < newLevel)
	{
		if( provideFeedback )
		{
			AudioEventRTS veterancyChanged;
			switch (newLevel)
			{
				case LEVEL_VETERAN:
					veterancyChanged = *getObject()->getTemplate()->getSoundPromotedVeteran();
					break;
				case LEVEL_ELITE:
					veterancyChanged = *getObject()->getTemplate()->getSoundPromotedElite();
					break;
				case LEVEL_HEROIC:
					veterancyChanged = *getObject()->getTemplate()->getSoundPromotedHero();
					break;
			}

			veterancyChanged.setObjectID(getObject()->getID());
			TheAudio->addAudioEvent(&veterancyChanged);
		}

		//Also mark the UI dirty -- incase the object is selected or contained.
		Object *obj = getObject();
		Drawable *draw = TheInGameUI->getFirstSelectedDrawable();
		if( draw )
		{
			Object *checkOwner = draw->getObject();
			if( checkOwner == obj )
			{
				//Our selected object has been promoted!
				TheControlBar->markUIDirty();
			}
			else
			{
				const Object *containedBy = obj->getContainedBy();
				if( containedBy && TheInGameUI->getSelectCount() == 1 )
				{
					Object *checkOwner = draw->getObject();
					if( checkOwner == containedBy )
					{
						//But only if the contained by object is containing me!
						TheControlBar->markUIDirty();
					}
				}
			}
		}
	}

	Real oldBonus = TheGlobalData->m_healthBonus[oldLevel];
	Real newBonus = TheGlobalData->m_healthBonus[newLevel];
	Real mult = newBonus / oldBonus;

	// get this before calling setMaxHealth, since it can clip curHealth
	//Real newHealth = m_currentHealth * mult;

	// change the max
	setMaxHealth(m_maxHealth * mult, PRESERVE_RATIO );

	// now change the cur (setMaxHealth now handles it)
	//internalChangeHealth( newHealth - m_currentHealth );

	switch (newLevel)
	{
		case LEVEL_REGULAR:
			clearArmorSetFlag(ARMORSET_VETERAN);
			clearArmorSetFlag(ARMORSET_ELITE);
			clearArmorSetFlag(ARMORSET_HERO);
			break;
		case LEVEL_VETERAN:
			setArmorSetFlag(ARMORSET_VETERAN);
			clearArmorSetFlag(ARMORSET_ELITE);
			clearArmorSetFlag(ARMORSET_HERO);
			break;
		case LEVEL_ELITE:
			clearArmorSetFlag(ARMORSET_VETERAN);
			setArmorSetFlag(ARMORSET_ELITE);
			clearArmorSetFlag(ARMORSET_HERO);
			break;
		case LEVEL_HEROIC:
			clearArmorSetFlag(ARMORSET_VETERAN);
			clearArmorSetFlag(ARMORSET_ELITE);
			setArmorSetFlag(ARMORSET_HERO);
			break;
	}
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void ActiveBody::setAflame( Bool )
{

	//
	// All this does now is act like a major body state change. It is called after Aflame has been
	// set or cleared as an Object Status
	//
	updateBodyParticleSystems();

}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void ActiveBody::crc( Xfer *xfer )
{

  // extend base class
	BodyModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void ActiveBody::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// base class
	BodyModule::xfer( xfer );

	// current health
	xfer->xferReal( &m_currentHealth );

	xfer->xferReal( &m_currentSubdualDamage );

	// previous health
	xfer->xferReal( &m_prevHealth );

	// max health
	xfer->xferReal( &m_maxHealth );

	// initial health
	xfer->xferReal( &m_initialHealth );

	// current damage state
	xfer->xferUser( &m_curDamageState, sizeof( BodyDamageType ) );

	// next damage fx time
	xfer->xferUnsignedInt( &m_nextDamageFXTime );

	// last damage fx done
	xfer->xferUser( &m_lastDamageFXDone, sizeof( DamageType ) );

	// last damage info
	xfer->xferSnapshot( &m_lastDamageInfo );

	// last damage timestamp
	xfer->xferUnsignedInt( &m_lastDamageTimestamp );

	// last damage timestamp
	xfer->xferUnsignedInt( &m_lastHealingTimestamp );

	// front crushed
	xfer->xferBool( &m_frontCrushed );

	// back crushed
	xfer->xferBool( &m_backCrushed );

	// last damaged cleared
	xfer->xferBool( &m_lastDamageCleared );

	// indestructible
	xfer->xferBool( &m_indestructible );

	// particle system count
	BodyParticleSystem *system;
	UnsignedShort particleSystemCount = 0;
	for( system = m_particleSystems; system; system = system->m_next )
		particleSystemCount++;
	xfer->xferUnsignedShort( &particleSystemCount );

	// particle systems
	if( xfer->getXferMode() == XFER_SAVE )
	{

		// walk the particle systems
		for( system = m_particleSystems; system; system = system->m_next )
		{

			// write particle system ID
			xfer->xferUser( &system->m_particleSystemID, sizeof( ParticleSystemID ) );

		}

	}
	else
	{
		ParticleSystemID particleSystemID;

		// the list should be empty at this time
		if( m_particleSystems != nullptr )
		{

			DEBUG_CRASH(( "ActiveBody::xfer - m_particleSystems should be empty, but is not" ));
			throw SC_INVALID_DATA;

		}

		// read all data elements
		BodyParticleSystem *newEntry;
		for( UnsignedShort i = 0; i < particleSystemCount; ++i )
		{

			// read particle system ID
			xfer->xferUser( &particleSystemID, sizeof( ParticleSystemID ) );

			// allocate entry and add to list
			newEntry = newInstance(BodyParticleSystem);
			newEntry->m_particleSystemID = particleSystemID;
			newEntry->m_next = m_particleSystems;  // the list will be reversed, but we don't care
			m_particleSystems = newEntry;

		}

	}

	// armor set flags
	m_curArmorSetFlags.xfer( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void ActiveBody::loadPostProcess()
{

	// extend base class
	BodyModule::loadPostProcess();

}
```
