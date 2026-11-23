# Translation Packet `PKT-166`: `BodyModule`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-166`
- **Work Package:** `WP-5.1` (Phase 5)
- **Complexity & Assigned Tier:** Class `C1` -> `tier:1a-lite`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/body/body_module.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (7):**
  - `Common/Module.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Module.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameLogic/ArmorSet.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ArmorSet.h`
  - `GameLogic/Damage.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/Damage.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `PreRTS.h`
- **Implemented Module Facets (1):**
  - `BodyModuleInterface* getBody()`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (2 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferReal( &m_damageScalar )`
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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
```cpp
// FILE: BodyModule.h /////////////////////////////////////////////////////////////////////////////////
// Author: Colin Day, September 2001
// Desc:
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

#include "Common/Module.h"
#include "GameLogic/Damage.h"
#include "GameLogic/ArmorSet.h"
#include "GameLogic/Module/BehaviorModule.h"

//-------------------------------------------------------------------------------------------------
/** OBJECT BODY MODULE base class */
//-------------------------------------------------------------------------------------------------

class WeaponTemplate;

//-------------------------------------------------------------------------------------------------
/** Damage states for structures
	*
	* NOTE: the macros below for IS_CONDITION_WORSE and IS_CONDITION_BETTER depend on this
	* enumeration being in sequential order
	*/
//-------------------------------------------------------------------------------------------------
enum BodyDamageType CPP_11(: Int)
{
	BODY_PRISTINE,				///< unit should appear in pristine condition
	BODY_DAMAGED,					///< unit has been damaged
	BODY_REALLYDAMAGED,		///< unit is extremely damaged / nearly destroyed
	BODY_RUBBLE,					///< unit has been reduced to rubble/corpse/exploded-hulk, etc

	BODYDAMAGETYPE_COUNT
};

#ifdef DEFINE_BODYDAMAGETYPE_NAMES
static const char* const TheBodyDamageTypeNames[] =
{
	"PRISTINE",
	"DAMAGED",
	"REALLYDAMAGED",
	"RUBBLE",

	nullptr
};
static_assert(ARRAY_SIZE(TheBodyDamageTypeNames) == BODYDAMAGETYPE_COUNT + 1, "Incorrect array size");
#endif

enum MaxHealthChangeType CPP_11(: Int)
{
	SAME_CURRENTHEALTH,
	PRESERVE_RATIO,
	ADD_CURRENT_HEALTH_TOO,
	FULLY_HEAL,

	MAX_HEALTH_CHANGE_COUNT
};

#ifdef DEFINE_MAXHEALTHCHANGETYPE_NAMES
static const char* const TheMaxHealthChangeTypeNames[] =
{
	"SAME_CURRENTHEALTH",
	"PRESERVE_RATIO",
	"ADD_CURRENT_HEALTH_TOO",
	"FULLY_HEAL",
	nullptr
};
static_assert(ARRAY_SIZE(TheMaxHealthChangeTypeNames) == MAX_HEALTH_CHANGE_COUNT + 1, "Incorrect array size");
#endif


//
// is condition A worse than condition B  ... NOTE: this assumes the conditions
// in BodyDamageType are in sequential order
//
// is a worse than b
#define IS_CONDITION_WORSE( a, b ) ( a > b )
// is a better than b
#define IS_CONDITION_BETTER( a, b ) ( a < b )

//-------------------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------------------
class BodyModuleData : public BehaviorModuleData
{
public:
	BodyModuleData()
	{
	}

	static void buildFieldParse(MultiIniFieldParse& p)
	{
    BehaviorModuleData::buildFieldParse(p);
	}
};

//-------------------------------------------------------------------------------------------------
class BodyModuleInterface
{
public:
	/**
		Try to damage this Object. The module's Armor
		will be taken into account, so the actual damage done may vary
		considerably from what you requested. Also note that (if damage is done)
		the DamageFX will be invoked to provide a/v fx as appropriate.
	*/
	virtual void attemptDamage( DamageInfo *damageInfo ) = 0;

	/**
		Instead of having negative damage count as healing, or allowing access to the private
		changeHealth Method, we will use this parallel to attemptDamage to do healing without hack.
	*/
	virtual void attemptHealing( DamageInfo *healingInfo ) = 0;

	/**
		Estimate the (unclipped) damage that would be done to this object
		by the given damage (taking bonuses, armor, etc into account),
		but DO NOT alter the body in any way. (This is used by the AI system
		to choose weapons.)
	*/
	virtual Real estimateDamage( DamageInfoInput& damageInfo ) const = 0;

	virtual Real getHealth() const = 0;													///< get current health

	virtual Real getMaxHealth() const = 0;

	virtual Real getInitialHealth() const = 0;

	virtual Real getPreviousHealth() const = 0;

	virtual UnsignedInt getSubdualDamageHealRate() const = 0;
	virtual Real getSubdualDamageHealAmount() const = 0;
	virtual Bool hasAnySubdualDamage() const = 0;
	virtual Real getCurrentSubdualDamageAmount() const = 0;

	virtual BodyDamageType getDamageState() const = 0;
	virtual void setDamageState( BodyDamageType newState ) = 0;	///< control damage state directly.  Will adjust hitpoints.
	virtual void setAflame( Bool setting ) = 0;///< This is a major change like a damage state.

	virtual void onVeterancyLevelChanged( VeterancyLevel oldLevel, VeterancyLevel newLevel, Bool provideFeedback ) = 0;	///< I just achieved this level right this moment

	virtual void setArmorSetFlag(ArmorSetType ast) = 0;
	virtual void clearArmorSetFlag(ArmorSetType ast) = 0;
	virtual Bool testArmorSetFlag(ArmorSetType ast) = 0;

	virtual const DamageInfo *getLastDamageInfo() const = 0;
	virtual const UnsignedInt *getLastDamageTimestamp() const = 0;
	virtual const UnsignedInt *getLastHealingTimestamp() const = 0;
	virtual ObjectID getClearableLastAttacker() const = 0;
	virtual void clearLastAttacker() = 0;
	virtual Bool getFrontCrushed() const = 0;
	virtual Bool getBackCrushed() const = 0;

	virtual void setInitialHealth(Int initialPercent)  = 0;
	virtual void setMaxHealth( Real maxHealth, MaxHealthChangeType healthChangeType = SAME_CURRENTHEALTH )  = 0;

	virtual void setFrontCrushed(Bool v) = 0;
	virtual void setBackCrushed(Bool v) = 0;

	virtual void applyDamageScalar( Real scalar ) = 0;
	virtual Real getDamageScalar() const = 0;

	/**
		Change the module's health by the given delta. Note that
		the module's DamageFX and Armor are NOT taken into
		account, so you should think about what you're bypassing when you
		call this directly (especially when when decreasing health, since
		you probably want "attemptDamage" or "attemptHealing")
	*/
	virtual void internalChangeHealth( Real delta ) = 0;

	virtual void setIndestructible( Bool indestructible ) = 0;
	virtual Bool isIndestructible() const = 0;

	virtual void evaluateVisualCondition() = 0;
	virtual void updateBodyParticleSystems() = 0; // made public for topple and building collapse updates -ML

};

//-------------------------------------------------------------------------------------------------
class BodyModule : public BehaviorModule, public BodyModuleInterface
{

	MEMORY_POOL_GLUE_ABC( BodyModule )

public:

	BodyModule( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype defined by MemoryPoolObject

	static Int getInterfaceMask() { return MODULEINTERFACE_BODY; }

	// BehaviorModule
	virtual BodyModuleInterface* getBody() override { return this; }

	virtual Real getMaxHealth() const override {return 0.0f;}  ///< return max health
	virtual Real getPreviousHealth() const override { return 0.0f; } ///< return previous health

	virtual UnsignedInt getSubdualDamageHealRate() const override {return 0;}
	virtual Real getSubdualDamageHealAmount() const override {return 0.0f;}
	virtual Bool hasAnySubdualDamage() const override {return FALSE;}
	virtual Real getCurrentSubdualDamageAmount() const override { return 0.0f; }

	virtual Real getInitialHealth() const override {return 0.0f;}  // return initial health

	virtual const DamageInfo *getLastDamageInfo() const override { return nullptr; }	///< return info on last damage dealt to this object
	virtual const UnsignedInt *getLastDamageTimestamp() const override	///< return frame of last damage, or nullptr if none
	{
#if RETAIL_COMPATIBLE_CRC
		static const UnsignedInt timestamp = 0;
		return &timestamp;
#else
		return nullptr;
#endif
	}
	virtual const UnsignedInt *getLastHealingTimestamp() const override	///< return frame of last healing, or nullptr if none
	{
#if RETAIL_COMPATIBLE_CRC
		static const UnsignedInt timestamp = 0;
		return &timestamp;
#else
		return nullptr;
#endif
	}
	virtual ObjectID getClearableLastAttacker() const override { return INVALID_ID; }
	virtual void clearLastAttacker() override { }
	virtual Bool getFrontCrushed() const override { return false; }
	virtual Bool getBackCrushed() const override { return false; }

	virtual void setInitialHealth(Int initialPercent) override  {  } ///< Sets the initial load health %.
	virtual void setMaxHealth(Real maxHealth, MaxHealthChangeType healthChangeType = SAME_CURRENTHEALTH ) override  {  } ///< Sets the max health.

	virtual void setFrontCrushed(Bool v) override { DEBUG_CRASH(("you should never call this for generic Bodys")); }
	virtual void setBackCrushed(Bool v) override { DEBUG_CRASH(("you should never call this for generic Bodys")); }


	virtual void setIndestructible( Bool indestructible ) override { }
	virtual Bool isIndestructible() const override { return TRUE; }

	//Allows outside systems to apply defensive bonuses or penalties (they all stack as a multiplier!)
	virtual void applyDamageScalar( Real scalar ) override { m_damageScalar *= scalar; }
	virtual Real getDamageScalar() const override { return m_damageScalar; }

	virtual void evaluateVisualCondition() override { }
	virtual void updateBodyParticleSystems() override { };// made public for topple anf building collapse updates -ML

protected:

	// snapshot methods
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

	Real	m_damageScalar;

};
inline BodyModule::BodyModule( Thing *thing, const ModuleData* moduleData ) : BehaviorModule( thing, moduleData ), m_damageScalar(1.0f) { }
inline BodyModule::~BodyModule() { }
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Body/BodyModule.cpp`
```cpp
// FILE: BodyModule.cpp ///////////////////////////////////////////////////////////////////////////
// Author: Colin Day, September 2002
// Desc:   BodyModule base class
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"
#include "Common/Xfer.h"
#include "GameLogic/Module/BodyModule.h"

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void BodyModule::crc( Xfer *xfer )
{

	// call base class
	BehaviorModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer Method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void BodyModule::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// call base class
	BehaviorModule::xfer( xfer );

	// damage scalar
	xfer->xferReal( &m_damageScalar );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void BodyModule::loadPostProcess()
{

	// call base class
	BehaviorModule::loadPostProcess();

}
```
