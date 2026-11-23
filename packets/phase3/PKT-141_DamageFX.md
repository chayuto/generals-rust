# Translation Packet `PKT-141`: `DamageFX`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-141`
- **Work Package:** `WP-3.3` (Phase 3)
- **Complexity & Assigned Tier:** Class `C1` -> `tier:1a-lite`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 2: Handle Component
- **Allowed Rust Output File:** `crates/zh_logic/src/combat/damage_fx.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (15):**
  - `Common/DamageFX.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/DamageFX.h`
  - `Common/GameAudio.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameAudio.h`
  - `Common/GameCommon.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameCommon.h`
  - `Common/INI.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/INI.h`
  - `Common/NameKeyGenerator.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/NameKeyGenerator.h`
  - `Common/STLTypedefs.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/STLTypedefs.h`
  - `Common/ThingFactory.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ThingFactory.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `GameClient/FXList.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/FXList.h`
  - `GameClient/GameClient.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/GameClient.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameLogic/Damage.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/Damage.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (8):**
  - `"AmountForMajorFX" => parseAmount`
  - `"MajorFX" => parseMajorFXList`
  - `"MinorFX" => parseMinorFXList`
  - `"ThrottleTime" => parseTime`
  - `"VeterancyAmountForMajorFX" => parseAmount`
  - `"VeterancyMajorFX" => parseMajorFXList`
  - `"VeterancyMinorFX" => parseMinorFXList`
  - `"VeterancyThrottleTime" => parseTime`
- **`Snapshot::crc` / `Snapshot::xfer` Calls:** None in module.

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

### `Core/GameEngine/Include/Common/DamageFX.h`
```cpp
// FILE: DamageFX.h /////////////////////////////////////////////////////////////////////////////////
// Author: Steven Johnson, November 2001
// Desc:   Damage Effects Descriptions
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/GameCommon.h"
#include "Common/NameKeyGenerator.h"
#include "Common/STLTypedefs.h"
#include "GameLogic/Damage.h"

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
class DamageFXStore;
class FXList;
class INI;

//-------------------------------------------------------------------------------------------------
typedef const FXList* ConstFXListPtr;

//-------------------------------------------------------------------------------------------------
/**
	A DamageFX is an object used to describe how an object reacts to taking a particular type
	of damage. (Note that "reacts" here implies only audio-video effects, not logic effects,
	damage modifiers, etc.)

	Conceptually speaking, every unit with a Body module has a DamageFX object. When it receives damage,
	it asks its DamageFX module to produce an appropriate a/v effect, which can vary by type of damage
	(eg, explosion, armor-piercing, flame, etc) and amount ("minor" or "major").

	Notes:

	-- Every particular damage-type within a DamageFX can have a "minor" and/or "major" effect;
	basically, if the damage done exceeds a specified threshold (or if there isn't a "minor"
	effect), the major effect is used.

	-- DamageFX is shared between multiple units; there should generally only be one instance
	of any particular class. The implication is that it should not require private data storage
	to do what it needs to do, aside from stuff initialized at object instantiation time. To help
	enforce this, all it's methods are declared 'const'. If you can't implement the damage you
	need within this framework, please *don't* simply de-const things, because it could lead to very
	strange side-effects. Instead, the system will have to be enhanced to allow for multiple instances
	of each DamageFX.
*/
//-------------------------------------------------------------------------------------------------
class DamageFX
{
public:

	DamageFX();

	void clear();

	/**
		This is the main public access point to the system: when you want to execute the
		a/v fx used for a specific damage type, call this method. (It is OK for source and/or
		victim to be null, but you should pass them when available.)
	*/
	void doDamageFX(DamageType t, Real damageAmount, const Object* source, const Object* victim) const;

	UnsignedInt getDamageFXThrottleTime(DamageType t, const Object* source) const;

	const FieldParse* getFieldParse() const;

private:

	ConstFXListPtr getDamageFXList(DamageType t, Real damageAmount, const Object* source) const;

	static void parseAmount( INI* ini, void *instance, void*, const void* );
	static void parseMajorFXList( INI* ini, void *instance, void*, const void* );
	static void parseMinorFXList( INI* ini, void *instance, void*, const void* );
	static void parseTime( INI* ini, void *instance, void*, const void* );

	/*
		this isn't terribly efficient since this is pretty sparsely populated
		and with lots of redundancy, but since we allocate very few of these,
		it's not worth more effort at this time...
	*/
	struct DFX
	{
		Real									m_amountForMajorFX;				///< if damage done is >= this, use major fx
		ConstFXListPtr				m_majorDamageFXList;			///< fx to make
		ConstFXListPtr				m_minorDamageFXList;			///< fx to make
		UnsignedInt						m_damageFXThrottleTime;

		DFX()
		{
			clear();
		}

		void clear()
		{
			m_amountForMajorFX = 0.0f;
			m_majorDamageFXList = nullptr;
			m_minorDamageFXList = nullptr;
			m_damageFXThrottleTime = 0;
		}
	};

	DFX m_dfx[DAMAGE_NUM_TYPES][LEVEL_COUNT];
};

//-------------------------------------------------------------------------------------------------
/**
	The "store" used to hold all the DamageFXs in existence. This is usually used when creating
	an Object (actually, a Body module), but can be used at any time after that. (It is explicitly
	OK to swap an Object's DamageFX out at any given time.)
*/
//-------------------------------------------------------------------------------------------------
class DamageFXStore : public SubsystemInterface
{

public:

	DamageFXStore();
	virtual ~DamageFXStore() override;

	virtual void init() override;
	virtual void reset() override;
	virtual void update() override;

	/**
		Find the DamageFX with the given name. If no such DamageFX exists, return null.
	*/
	const DamageFX *findDamageFX( NameKeyType namekey ) const;
	const DamageFX *findDamageFX( const AsciiString& name ) const;
	const DamageFX *findDamageFX( const char* name ) const;

	static void parseDamageFXDefinition(INI* ini);


private:

	typedef std::hash_map< NameKeyType, DamageFX, rts::hash<NameKeyType>, rts::equal_to<NameKeyType>/**/> DamageFXMap;
	DamageFXMap m_dfxmap;

};

// EXTERNALS //////////////////////////////////////////////////////////////////////////////////////
extern DamageFXStore *TheDamageFXStore;
```

### `Core/GameEngine/Source/Common/DamageFX.cpp`
```cpp
// FILE: DamageFX.cpp ///////////////////////////////////////////////////////////////////////////////
// Author: Steven Johnson, November 2001
// Desc:   DamageFX descriptions
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/INI.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/DamageFX.h"
#include "Common/GameAudio.h"

#include "GameClient/FXList.h"
#include "GameLogic/Damage.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameClient/GameClient.h"
#include "GameClient/InGameUI.h"

///////////////////////////////////////////////////////////////////////////////////////////////////
// PUBLIC DATA ////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

DamageFXStore *TheDamageFXStore = nullptr;					///< the DamageFX store definition

///////////////////////////////////////////////////////////////////////////////////////////////////
// PRIVATE DATA ///////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
// PRIVATE CLASSES ///////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS ///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
DamageFX::DamageFX()
{
	// not necessary.
	//clear();
}

//-------------------------------------------------------------------------------------------------
void DamageFX::clear()
{
	for (Int dt = 0; dt < DAMAGE_NUM_TYPES; ++dt)
	{
		for (Int v = LEVEL_FIRST; v <= LEVEL_LAST; ++v)
		{
			m_dfx[dt][v].clear();
		}
	}
}

//-------------------------------------------------------------------------------------------------
UnsignedInt DamageFX::getDamageFXThrottleTime(DamageType t, const Object* source) const
{
	return m_dfx[t][source ? source->getVeterancyLevel() : LEVEL_REGULAR].m_damageFXThrottleTime;
}

//-------------------------------------------------------------------------------------------------
void DamageFX::doDamageFX(DamageType t, Real damageAmount, const Object* source, const Object* victim) const
{
	ConstFXListPtr fx = getDamageFXList(t, damageAmount, source);
	// since the victim is receiving the damage, it's the "primary" object.
	// the source is the "secondary" object -- unused by most fx, but could be
	// useful in some cases.
	FXList::doFXObj(fx, victim, source);
}

//-------------------------------------------------------------------------------------------------
ConstFXListPtr DamageFX::getDamageFXList(DamageType t, Real damageAmount, const Object* source) const
{
	/*
		if damage is zero, never do damage fx. this is by design, since "zero" damage can happen
		with some special weapons, like the battleship, which is a "faux" weapon that never does damage.
		if you really need to change this for some reason, consider carefully... (srj)
	*/
	if (damageAmount == 0.0f)
		return nullptr;

	const DFX& dfx = m_dfx[t][source ? source->getVeterancyLevel() : LEVEL_REGULAR];
	ConstFXListPtr fx =
		damageAmount >= dfx.m_amountForMajorFX ?
		dfx.m_majorDamageFXList :
		dfx.m_minorDamageFXList;

	return fx;
}

//-------------------------------------------------------------------------------------------------
const FieldParse* DamageFX::getFieldParse() const
{
	static const FieldParse myFieldParse[] =
	{
		{ "AmountForMajorFX",						parseAmount,			nullptr, 0 },
		{ "MajorFX",										parseMajorFXList,	nullptr, 0 },
		{ "MinorFX",										parseMinorFXList,	nullptr, 0 },
		{ "ThrottleTime",								parseTime,				nullptr, 0 },
		{ "VeterancyAmountForMajorFX",	parseAmount,			TheVeterancyNames, 0 },
		{ "VeterancyMajorFX",						parseMajorFXList,	TheVeterancyNames, 0 },
		{ "VeterancyMinorFX",						parseMinorFXList,	TheVeterancyNames, 0 },
		{ "VeterancyThrottleTime",			parseTime,				TheVeterancyNames, 0 },
		{ nullptr, nullptr, nullptr,0 }
	};
	return myFieldParse;
}

//-------------------------------------------------------------------------------------------Static
static void parseCommonStuff(
	INI* ini,
	ConstCharPtrArray names,
	VeterancyLevel& vetFirst,
	VeterancyLevel& vetLast,
	DamageType& damageFirst,
	DamageType& damageLast
)
{
	if (names)
	{
		vetFirst = (VeterancyLevel)INI::scanIndexList(ini->getNextToken(), names);
		vetLast = vetFirst;
	}
	else
	{
		vetFirst = LEVEL_FIRST;
		vetLast = LEVEL_LAST;
	}

	const char* damageName = ini->getNextToken();
	if (stricmp(damageName, "Default") == 0)
	{
		damageFirst = (DamageType)0;
		damageLast = (DamageType)(DAMAGE_NUM_TYPES - 1);
	}
	else
	{
		damageFirst = (DamageType)INI::scanIndexList(damageName, DamageTypeFlags::getBitNames());
		damageLast = damageFirst;
	}
}

//-------------------------------------------------------------------------------------------Static
/*static*/ void DamageFX::parseAmount( INI* ini, void* instance, void* /*store*/, const void* userData )
{
	DamageFX* self = (DamageFX*)instance;
	ConstCharPtrArray names = (ConstCharPtrArray)userData;

	VeterancyLevel vetFirst, vetLast;
	DamageType damageFirst, damageLast;
	parseCommonStuff(ini, names, vetFirst, vetLast, damageFirst, damageLast);

	Real amt = INI::scanReal(ini->getNextToken());

	for (Int dt = damageFirst; dt <= damageLast; ++dt)
	{
		for (Int v = vetFirst; v <= vetLast; ++v)
		{
			self->m_dfx[dt][v].m_amountForMajorFX = amt;
		}
	}
}

//-------------------------------------------------------------------------------------------Static
/*static*/ void DamageFX::parseMajorFXList( INI* ini, void* instance, void* /*store*/, const void* userData )
{
	DamageFX* self = (DamageFX*)instance;
	ConstCharPtrArray names = (ConstCharPtrArray)userData;

	VeterancyLevel vetFirst, vetLast;
	DamageType damageFirst, damageLast;
	parseCommonStuff(ini, names, vetFirst, vetLast, damageFirst, damageLast);

	ConstFXListPtr fx;
	INI::parseFXList(ini, nullptr, &fx, nullptr);

	for (Int dt = damageFirst; dt <= damageLast; ++dt)
	{
		for (Int v = vetFirst; v <= vetLast; ++v)
		{
			self->m_dfx[dt][v].m_majorDamageFXList = fx;
		}
	}
}

//-------------------------------------------------------------------------------------------Static
/*static*/ void DamageFX::parseMinorFXList( INI* ini, void* instance, void* /*store*/, const void* userData )
{
	DamageFX* self = (DamageFX*)instance;
	ConstCharPtrArray names = (ConstCharPtrArray)userData;

	VeterancyLevel vetFirst, vetLast;
	DamageType damageFirst, damageLast;
	parseCommonStuff(ini, names, vetFirst, vetLast, damageFirst, damageLast);

	ConstFXListPtr fx;
	INI::parseFXList(ini, nullptr, &fx, nullptr);

	for (Int dt = damageFirst; dt <= damageLast; ++dt)
	{
		for (Int v = vetFirst; v <= vetLast; ++v)
		{
			self->m_dfx[dt][v].m_minorDamageFXList = fx;
		}
	}
}

//-------------------------------------------------------------------------------------------Static
/*static*/ void DamageFX::parseTime( INI* ini, void* instance, void* /*store*/, const void* userData )
{
	DamageFX* self = (DamageFX*)instance;
	ConstCharPtrArray names = (ConstCharPtrArray)userData;

	VeterancyLevel vetFirst, vetLast;
	DamageType damageFirst, damageLast;
	parseCommonStuff(ini, names, vetFirst, vetLast, damageFirst, damageLast);

	UnsignedInt t;
	INI::parseDurationUnsignedInt(ini, nullptr, &t, nullptr);

	for (Int dt = damageFirst; dt <= damageLast; ++dt)
	{
		for (Int v = vetFirst; v <= vetLast; ++v)
		{
			self->m_dfx[dt][v].m_damageFXThrottleTime = t;
		}
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
DamageFXStore::DamageFXStore()
{
	m_dfxmap.clear();
}

//-------------------------------------------------------------------------------------------------
DamageFXStore::~DamageFXStore()
{
	m_dfxmap.clear();
}

//-------------------------------------------------------------------------------------------------
const DamageFX *DamageFXStore::findDamageFX(NameKeyType namekey) const
{
	DamageFXMap::const_iterator it = m_dfxmap.find(namekey);
	if (it == m_dfxmap.end())
	{
		return nullptr;
	}
	else
	{
		return &(*it).second;
	}
}

//-------------------------------------------------------------------------------------------------
const DamageFX *DamageFXStore::findDamageFX(const AsciiString& name) const
{
	return findDamageFX(TheNameKeyGenerator->nameToKey(name));
}

//-------------------------------------------------------------------------------------------------
const DamageFX *DamageFXStore::findDamageFX(const char* name) const
{
	return findDamageFX(TheNameKeyGenerator->nameToKey(name));
}

//-------------------------------------------------------------------------------------------------
void DamageFXStore::init()
{
}

//-------------------------------------------------------------------------------------------------
void DamageFXStore::reset()
{
}

//-------------------------------------------------------------------------------------------------
void DamageFXStore::update()
{
}

//-------------------------------------------------------------------------------------------------
/*static */ void DamageFXStore::parseDamageFXDefinition(INI* ini)
{

	const char *c = ini->getNextToken();
	NameKeyType key = TheNameKeyGenerator->nameToKey(c);
	DamageFX& dfx = TheDamageFXStore->m_dfxmap[key];
	dfx.clear();
	ini->initFromINI(&dfx, dfx.getFieldParse());
}
```
