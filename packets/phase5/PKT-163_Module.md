# Translation Packet `PKT-163`: `Module`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-163`
- **Work Package:** `WP-5.0` (Phase 5)
- **Complexity & Assigned Tier:** Class `C4` -> `tier:3-architect`
- **Environment Access:** `env:local-full`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/module.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (19):**
  - `Common/GameMemory.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameMemory.h`
  - `Common/INI.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/INI.h`
  - `Common/Module.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Module.h`
  - `Common/NameKeyGenerator.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/NameKeyGenerator.h`
  - `Common/Snapshot.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Snapshot.h`
  - `Common/Thing.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Thing.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Upgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Upgrade.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/CollideModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CollideModule.h`
  - `GameLogic/Module/ContainModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ContainModule.h`
  - `GameLogic/Module/DamageModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DamageModule.h`
  - `GameLogic/Module/DieModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DieModule.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Module/UpgradeModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpgradeModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `PreRTS.h`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (3 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferVersion( &version, currentVersion )`
  3. `xferVersion( &version, currentVersion )`

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
- `zh_logic::arena`: `ObjectSlot`, `INVALID`, `fn`, `ModuleSlot`, `ModuleDataId`, `ThingTemplateId`, `FacetMask`, `NONE`, `UPDATE`, `DIE`, `DAMAGE`, `CREATE`, `COLLIDE`, `BODY`, ... (+22)
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
- `zh_logic::scheduler::sleepy_heap`: `UPDATE_SLEEP_INVALID`, `UPDATE_SLEEP_NONE`, `UPDATE_SLEEP_FOREVER`, `INVALID_LOGIC_INDEX`, `SleepyUpdatePhase`, `fn`, `UpdateModuleScheduleState`, `ScheduleStore`, `SleepyUpdateHeap`
- `zh_logic::combat::armor`: `ArmorTemplate`, `Armor`, `fn`, `ArmorStore`
- `zh_logic::combat::armor_set`: `ArmorTemplateSet`
- `zh_logic::combat::damage`: `PlayerMaskType`, `HUGE_DAMAGE_AMOUNT`, `DamageInfoInput`, `DamageInfoOutput`, `DamageInfo`
- `zh_logic::combat::locomotor`: `BIGNUM`, `LocomotorAppearance`, `fn`, `LOCOMOTOR_APPEARANCE_NAMES`, `LocomotorPriority`, `LOCOMOTOR_PRIORITY_NAMES`, `LocomotorBehaviorZ`, `LOCOMOTOR_BEHAVIOR_Z_NAMES`, `LocomotorTemplate`, `LocomotorStore`, `Locomotor`
- `zh_logic::combat::locomotor_set`: `LocomotorSet`
- `zh_logic::economy`: `MAX_PLAYER_COUNT`
- `zh_logic::economy::academy_stats`: `MAX_ADVICE_TIPS`, `FRAMES_BETWEEN_UPDATES`, `DEFAULT_SUPPLY_CENTER_COST`, `KINDOF_DOZER`, `KINDOF_FS_SUPPLY_CENTER`, `KINDOF_COMMANDCENTER`, `KINDOF_INFANTRY`, `KINDOF_VEHICLE`, `KINDOF_HARVESTER`, `KINDOF_HERO`, `KINDOF_FS_STRATEGY_CENTER`, `KINDOF_MONEY_HACKER`, `KINDOF_FS_BLACK_MARKET`, `KINDOF_FS_SUPPLY_DROPZONE`, ... (+9)
- `zh_logic::economy::energy`: `EnergyOwnerCallbacks`, `NoopEnergyCallbacks`, `EnergyObjectInfo`, `Energy`
- `zh_logic::economy::handicap`: `HANDICAP_TYPE_COUNT`, `THING_TYPE_COUNT`, `HandicapType`, `ThingType`, `Handicap`, `fn`
- `zh_logic::economy::mission_stats`: `MissionStats`
- `zh_logic::economy::money`: `INCOME_BUCKET_COUNT`, `LOGICFRAMES_PER_SECOND`, `MoneyCallbacks`, `NoopMoneyCallbacks`, `Money`
- `zh_logic::gamelogic::global_data`: `MAX_GLOBAL_LIGHTS`, `MAX_WATER_GRID_SETTINGS`, `SIMULATE_REPLAYS_SEQUENTIAL`, `DEFAULT_DISPLAY_WIDTH`, `DEFAULT_DISPLAY_HEIGHT`, `DEFAULT_JPEG_QUALITY`, `DEFAULT_DOUBLE_CLICK_TIME_MS`, `PLAYERTYPE_COUNT`, `DIFFICULTY_COUNT`, `LEVEL_COUNT`, `MULTISAMPLE_MODE_NONE`, `TEXTURE_FILTER_BILINEAR`, `TEXTURE_FILTER_ANISOTROPIC_2X`, `FIREWALL_TYPE_UNKNOWN`, ... (+30)
- `zh_logic::gamelogic::message_stream`: `TranslatorId`, `TRANSLATOR_ID_INVALID`, `MSG_COUNT_VALUE`, `GameMessageArgumentDataType`, `GameMessageArgumentValue`, `GameMessageArgument`, `GameMessageType`, `fn`, `MessageId`, `GameMessage`, `GameMessageList`, `CommandList`, `GameMessageDisposition`, `GameMessageTranslator`, ... (+1)

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

### `GeneralsMD/Code/GameEngine/Include/Common/Module.h`
```cpp
// FILE: Module.h /////////////////////////////////////////////////////////////////////////////////
// Author: Colin Day, September 2001
// Desc:	 Object and drawable modules and actions.  These are simply just class
//				 instances that we can assign to objects, drawables, and things to contain
//				 data and code for specific events, or just to hold data
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/INI.h"
#include "Common/GameMemory.h"
#include "Common/NameKeyGenerator.h"
#include "Common/Snapshot.h"

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
enum TimeOfDay CPP_11(: Int);
enum StaticGameLODLevel CPP_11(: Int);
class Drawable;
class Object;
class Player;
class Thing;
class W3DModelDrawModuleData;	// ugh, hack (srj)
class W3DTreeDrawModuleData; // ugh, hack (srj)
struct FieldParse;

// TYPES //////////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
enum ModuleType CPP_11(: Int)
{
	MODULETYPE_BEHAVIOR = 0,

	//
	// drawable module types - you should *NOT* remove drawable module types, we write
	// modules into save game files in buckets of module types ... if you do remove one
	// here you will have to update the xfer code for a drawable.
	//
	// ALSO note that new drawable module types should go at the end of the existing drawable modules
	//
	MODULETYPE_DRAW = 1,
	MODULETYPE_CLIENT_UPDATE = 2,
	// put new drawable module types here

	NUM_MODULE_TYPES,

	FIRST_DRAWABLE_MODULE_TYPE = MODULETYPE_DRAW,
	LAST_DRAWABLE_MODULE_TYPE = MODULETYPE_CLIENT_UPDATE,
	NUM_DRAWABLE_MODULE_TYPES = (LAST_DRAWABLE_MODULE_TYPE - FIRST_DRAWABLE_MODULE_TYPE + 1)

};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
enum ModuleInterfaceType CPP_11(: Int)
{
	MODULEINTERFACE_UPDATE					= 0x00000001,
	MODULEINTERFACE_DIE							= 0x00000002,
	MODULEINTERFACE_DAMAGE					= 0x00000004,
	MODULEINTERFACE_CREATE					= 0x00000008,
	MODULEINTERFACE_COLLIDE					= 0x00000010,
	MODULEINTERFACE_BODY						= 0x00000020,
	MODULEINTERFACE_CONTAIN					= 0x00000040,
	MODULEINTERFACE_UPGRADE					= 0x00000080,
	MODULEINTERFACE_SPECIAL_POWER		= 0x00000100,
	MODULEINTERFACE_DESTROY					= 0x00000200,
	MODULEINTERFACE_DRAW						= 0x00000400,
	MODULEINTERFACE_CLIENT_UPDATE		= 0x00000800
};

//-------------------------------------------------------------------------------------------------
/** Base class for data-read-from-INI for modules. */
//-------------------------------------------------------------------------------------------------
/// @todo srj -- make ModuleData be MemoryPool based
class ModuleData : public Snapshot
{
public:
	ModuleData() { }
	virtual ~ModuleData() { }

	void setModuleTagNameKey( NameKeyType key ) { m_moduleTagNameKey = key; }
	NameKeyType getModuleTagNameKey() const { return m_moduleTagNameKey; }

	virtual Bool isAiModuleData() const { return false; }

	// ugh, hack
	virtual const W3DModelDrawModuleData* getAsW3DModelDrawModuleData() const { return nullptr; }
	// ugh, hack
	virtual const W3DTreeDrawModuleData* getAsW3DTreeDrawModuleData() const { return nullptr; }
	virtual StaticGameLODLevel getMinimumRequiredGameLOD() const { return (StaticGameLODLevel)0;}

	static void buildFieldParse(MultiIniFieldParse& p)
	{
		// nothing
	}

public:
	virtual void crc( Xfer *xfer ) override {}
	virtual void xfer( Xfer *xfer ) override {}
	virtual void loadPostProcess() override {}

private:
	NameKeyType m_moduleTagNameKey;		///< module tag key, unique among all modules for an object instance
};

//-------------------------------------------------------------------------------------------------
// This macro is to assist in the creation of new modules, contains the common
// things that all module definitions must have in order to work with
// the module creation factory
//-------------------------------------------------------------------------------------------------
#define MAKE_STANDARD_MODULE_MACRO( cls ) \
public: \
	static Module* friend_newModuleInstance( Thing *thing, const ModuleData* moduleData ) { return newInstance( cls )( thing, moduleData ); } \
	virtual NameKeyType getModuleNameKey() const override { static NameKeyType nk = NAMEKEY(#cls); return nk; } \
protected: \
	virtual void crc( Xfer *xfer ) override; \
	virtual void xfer( Xfer *xfer ) override; \
	virtual void loadPostProcess() override;

// ------------------------------------------------------------------------------------------------
// For the creation of abstract module classes
// ------------------------------------------------------------------------------------------------
#define MAKE_STANDARD_MODULE_MACRO_ABC( cls ) \
protected: \
	virtual void crc( Xfer *xfer ) override; \
	virtual void xfer( Xfer *xfer ) override; \
	virtual void loadPostProcess() override;

//-------------------------------------------------------------------------------------------------
// only use this macro for an ABC. for a real class, use MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA.
#define MAKE_STANDARD_MODULE_DATA_MACRO_ABC( cls, clsmd ) \
private: \
	const clsmd* get##clsmd() const { return (clsmd*)getModuleData(); } \
public: \
	static ModuleData* friend_newModuleData(INI* ini) \
	{ \
		clsmd* data = MSGNEW( "AllModuleData" ) clsmd; \
		if (ini) ini->initFromINIMultiProc(data, clsmd::buildFieldParse); \
		return data; \
	}

//-------------------------------------------------------------------------------------------------
#define MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( cls, clsmd ) \
	MAKE_STANDARD_MODULE_MACRO(cls) \
	MAKE_STANDARD_MODULE_DATA_MACRO_ABC(cls, clsmd)

//-------------------------------------------------------------------------------------------------
/** Common interface for thing modules, we want a single common base class
	* for all the modules (either object or drawable) so that we can use
	* a single module factory to handle instancing them ... it's just
	* convenient this way */
//-------------------------------------------------------------------------------------------------
class Module : public MemoryPoolObject,
							 public Snapshot
{

	MEMORY_POOL_GLUE_ABC( Module )						///< this abstract class needs memory pool hooks

public:

	Module(const ModuleData* moduleData) : m_moduleData(moduleData) { }
	// virtual destructor prototype defined by MemoryPoolObject

	// this method should NEVER be overridden by user code, only via the MAKE_STANDARD_MODULE_xxx macros!
	// it should also NEVER be called directly; it's only for use by ModuleFactory!
	static ModuleData* friend_newModuleData(INI* ini);

	virtual NameKeyType getModuleNameKey() const = 0;

	NameKeyType getModuleTagNameKey() const { return getModuleData()->getModuleTagNameKey(); }

	/** this is called after all the Modules for a given Thing are created; it
		allows Modules to resolve any inter-Module dependencies.
	*/
	virtual void onObjectCreated() { }

	/**
		this is called whenever a drawable is bound to the object.
		drawable is NOT guaranteed to be non-null.
	*/
	virtual void onDrawableBoundToObject() { }

	/// preload any assets we might have for this time of day
	virtual void preloadAssets( TimeOfDay timeOfDay ) { }

	/** onDelete() will be called on all modules contained by an object or drawable before
	the actual deletion of each of those modules happens */
	virtual void onDelete() { }

protected:

	const ModuleData* getModuleData() const { return m_moduleData; }

	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

private:
	const ModuleData* m_moduleData;

};
//-------------------------------------------------------------------------------------------------


//=================================================================================================
//														OBJECT Module interface and modules
//=================================================================================================


//-------------------------------------------------------------------------------------------------
/** Module interface specific for Objects, this is really just to make a clear distinction
	* between modules intended for use in objects and modules intended for use
	* in drawables */
//-------------------------------------------------------------------------------------------------
class ObjectModule : public Module
{

	MEMORY_POOL_GLUE_ABC( ObjectModule )			///< this abstract class needs memory pool hooks

public:

	ObjectModule( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype defined by MemoryPoolObject

	virtual void onCapture( Player *oldOwner, Player *newOwner ) { }
	virtual void onDisabledEdge( Bool nowDisabled ) { }

protected:

	Object *getObject() { return m_object; }
	const Object *getObject() const { return m_object; }

	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

private:

	// it shouldn't be legal for subclasses to ever modify this, only to look at it;
	// so, we'll enforce this by making it private and providing a protected access method.
	Object *m_object;													///< the object this module is a part of

};
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------






//=================================================================================================
//													DRAWABLE module interface and modules
//=================================================================================================

//-------------------------------------------------------------------------------------------------
/** Module interface specific for Drawables, this is really just to make a clear distinction
	* between modules intended for use in objects and modules intended for use
	* in drawables */
//-------------------------------------------------------------------------------------------------
class DrawableModule : public Module
{

	MEMORY_POOL_GLUE_ABC( DrawableModule )		///< this abstract class needs memory pool hooks

public:

	DrawableModule( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype defined by MemoryPoolObject

protected:

	Drawable *getDrawable() { return m_drawable; }
	const Drawable *getDrawable() const { return m_drawable; }

	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

private:

	// it shouldn't be legal for subclasses to ever modify this, only to look at it;
	// so, we'll enforce this by making it private and providing a protected access method.
	Drawable *m_drawable;											///< the drawble this module is a part of

};
//-------------------------------------------------------------------------------------------------


//-------------------------------------------------------------------------------------------------
/** VARIOUS MODULE INTERFACES */
//-------------------------------------------------------------------------------------------------
```

### `GeneralsMD/Code/GameEngine/Source/Common/Thing/Module.cpp`
```cpp
// FILE: Module.cpp ///////////////////////////////////////////////////////////////////////////////
// Author: Colin Day, September 2001
// Desc:	 Object and drawable modules and actions.  These are simply just class
//				 instances that we can assign to objects, drawables, and things to contain
//				 data and code for specific events, or just to hold data
///////////////////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/Module.h"
#include "Common/Thing.h"
#include "Common/INI.h"
#include "Common/ThingTemplate.h"
#include "Common/Upgrade.h"
#include "Common/Xfer.h"
#include "GameLogic/Object.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/CollideModule.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/DamageModule.h"
#include "GameLogic/Module/DieModule.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Module/UpgradeModule.h"


///////////////////////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS ///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
// this method should NEVER be overridden by user code, only via the MAKE_STANDARD_MODULE_xxx macros!
// it should also NEVER be called directly; it's only for use by ModuleFactory!
/*static*/ ModuleData* Module::friend_newModuleData(INI* ini)
{
	ModuleData* data = MSGNEW("Module::friend_newModuleData") ModuleData;	// no need to memorypool these since we never allocate more than one of each
	if (ini)
		ini->initFromINI(data, nullptr);	// this is just so that an "end" token is required
	return data;
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
Module::~Module()
{

}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void Module::crc( Xfer *xfer )
{

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void Module::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

}

// ------------------------------------------------------------------------------------------------
/** load post process */
// ------------------------------------------------------------------------------------------------
void Module::loadPostProcess()
{

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ObjectModule::ObjectModule( Thing *thing, const ModuleData* moduleData ) : Module(moduleData)
{
	if (!moduleData)
	{
		DEBUG_CRASH(("module data may not be null"));
		throw INI_INVALID_DATA;
	}

	DEBUG_ASSERTCRASH( thing, ("Thing passed to ObjectModule is null!") );
	m_object = AsObject(thing);
	DEBUG_ASSERTCRASH( m_object, ("Thing passed to ObjectModule is not an Object!") );

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ObjectModule::~ObjectModule()
{

}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void ObjectModule::crc( Xfer *xfer )
{

	// extend base class
	Module::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void ObjectModule::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	Module::xfer( xfer );

}

// ------------------------------------------------------------------------------------------------
/** load post process */
// ------------------------------------------------------------------------------------------------
void ObjectModule::loadPostProcess()
{

	// extend base class
	Module::loadPostProcess();

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
DrawableModule::DrawableModule( Thing *thing, const ModuleData* moduleData ) : Module(moduleData)
{
	if (!moduleData)
	{
		DEBUG_CRASH(("module data may not be null"));
		throw INI_INVALID_DATA;
	}

	DEBUG_ASSERTCRASH( thing, ("Thing passed to DrawableModule is null!") );
	m_drawable = AsDrawable(thing);
	DEBUG_ASSERTCRASH( m_drawable, ("Thing passed to DrawableModule is not a Drawable!") );

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
DrawableModule::~DrawableModule()
{

}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void DrawableModule::crc( Xfer *xfer )
{

	// extend base class
	Module::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void DrawableModule::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	Module::xfer( xfer );

}

// ------------------------------------------------------------------------------------------------
/** load post process */
// ------------------------------------------------------------------------------------------------
void DrawableModule::loadPostProcess()
{

	// extend base class
	Module::loadPostProcess();

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------------------
void UpgradeMuxData::performUpgradeFX(Object* obj) const
{
	if (m_fxListUpgrade)
	{
		FXList::doFXObj(m_fxListUpgrade, obj);
	}
}

//-------------------------------------------------------------------------------------------------
void UpgradeMuxData::muxDataProcessUpgradeRemoval(Object* obj) const
{
	if( !m_removalUpgradeNames.empty() )
	{
		std::vector<AsciiString>::const_iterator it;
		for( it = m_removalUpgradeNames.begin();
					it != m_removalUpgradeNames.end();
					it++)
		{
			const UpgradeTemplate* theTemplate = TheUpgradeCenter->findUpgrade( *it );
			if( !theTemplate )
			{
				DEBUG_CRASH(("An upgrade module references '%s', which is not an Upgrade", it->str()));
				throw INI_INVALID_DATA;
			}

			obj->removeUpgrade(theTemplate);
		}
	}
}

//-------------------------------------------------------------------------------------------------
Bool UpgradeMuxData::isTriggeredBy(const std::string &upgrade) const
{
	std::vector<AsciiString>::const_iterator it;
	for( it = m_triggerUpgradeNames.begin(); it != m_triggerUpgradeNames.end();	++it)
	{
		const AsciiString &trigger = *it;
		if (stricmp(trigger.str(), upgrade.c_str()) == 0)
		{
			return TRUE;
		}
	}

	return FALSE;
}

//-------------------------------------------------------------------------------------------------
void UpgradeMuxData::getUpgradeActivationMasks(UpgradeMaskType& activation, UpgradeMaskType& conflicting) const
{
	// already computed.
	if (!m_activationUpgradeNames.empty() || !m_conflictingUpgradeNames.empty())
	{
		m_activationMask.clear();
		m_conflictingMask.clear();

		std::vector<AsciiString>::const_iterator it;
		for( it = m_activationUpgradeNames.begin();
					it != m_activationUpgradeNames.end();
					it++)
		{
			const UpgradeTemplate* theTemplate = TheUpgradeCenter->findUpgrade( *it );
			if( !theTemplate )
			{
				DEBUG_CRASH(("An upgrade module references '%s', which is not an Upgrade", it->str()));
				throw INI_INVALID_DATA;
			}

			m_activationMask.set( theTemplate->getUpgradeMask() );
		}

		for( it = m_conflictingUpgradeNames.begin();
					it != m_conflictingUpgradeNames.end();
					it++)
		{
			const UpgradeTemplate* theTemplate = TheUpgradeCenter->findUpgrade( *it );
			if( !theTemplate )
			{
				DEBUG_CRASH(("An upgrade module references '%s', which is not an Upgrade", it->str()));
				throw INI_INVALID_DATA;
			}

			m_conflictingMask.set( theTemplate->getUpgradeMask() );
		}

		// We set the trigger upgrade names with the activationUpgradeNames entries to be used later.
		// We have to do this because the activationUpgradeNames are toasted just below.
		m_triggerUpgradeNames = m_activationUpgradeNames;

		//Clear the names now that we've cached the values!
		m_activationUpgradeNames.clear();
		m_conflictingUpgradeNames.clear();
	}
	activation = m_activationMask;
	conflicting = m_conflictingMask;
}
```
