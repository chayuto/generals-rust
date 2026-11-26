# Translation Packet `PKT-336`: `HordeUpdate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-336`
- **Work Package:** `WP-5.4` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/update/horde_update.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (13):**
  - `Common/BitFlagsIO.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/BitFlagsIO.h`
  - `Common/KindOf.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/KindOf.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Upgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Upgrade.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/HordeUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/HordeUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `GameLogic/Weapon.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h`
  - `PreRTS.h`
- **Implemented Module Facets (1):**
  - `HordeUpdateInterface* getHordeUpdateInterface()`
- **INI `FieldParse` Table Entries (10):**
  - `"UpdateRate" => INI::parseDurationUnsignedInt`
  - `"KindOf" => KindOfMaskType::parseFromINI`
  - `"Count" => INI::parseInt`
  - `"Radius" => INI::parseReal`
  - `"RubOffRadius" => INI::parseReal`
  - `"AlliesOnly" => INI::parseBool`
  - `"ExactMatch" => INI::parseBool`
  - `"Action" => INI::parseIndexList`
  - `"FlagSubObjectNames" => INI::parseAsciiStringVector`
  - `"AllowedNationalism" => INI::parseBool`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (3 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferBool( &m_inHorde )`
  3. `xferBool( &m_hasFlag )`
- **PRNG Calls (1 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValue(1, delay)))`
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if !RETAIL_COMPATIBLE_CRC`
  - `#if RETAIL_COMPATIBLE_CRC || PRESERVE_PERPETUAL_HORDE_BONUS`

## 2.5 Existing Rust Symbol Index (Use These Exact Paths)
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
- `zh_logic::modules::update::smart_bomb_target_homing_update`: `SmartBombTargetHomingUpdateModuleData`, `SmartBombTargetHomingUpdate`
- `zh_logic::modules::update::spawn_point_production_exit_update`: `MAX_SPAWN_POINTS`, `SpawnPointProductionExitUpdateModuleData`, `SpawnPointProductionExitUpdate`
- `zh_logic::modules::update::spy_vision_update`: `SpyVisionUpdateModuleData`, `SpyVisionUpdate`, `fn`
- `zh_logic::modules::update::stealth_detector_update`: `StealthDetectorUpdateModuleData`, `StealthDetectorUpdate`
- `zh_logic::modules::update::stealth_update`: `STEALTH_NOT_WHILE_ATTACKING`, `STEALTH_NOT_WHILE_MOVING`, `STEALTH_NOT_WHILE_USING_ABILITY`, `STEALTH_NOT_WHILE_FIRING_PRIMARY`, `STEALTH_NOT_WHILE_FIRING_SECONDARY`, `STEALTH_NOT_WHILE_FIRING_TERTIARY`, `STEALTH_ONLY_WITH_BLACK_MARKET`, `STEALTH_NOT_WHILE_TAKING_DAMAGE`, `STEALTH_NOT_WHILE_RIDERS_ATTACKING`, `STEALTH_NOT_WHILE_FIRING_WEAPON`, `STEALTH_LEVEL_NAMES`, `StealthUpdateModuleData`, `StealthUpdate`
- `zh_logic::modules::update::supply_center_dock_update`: `SupplyCenterDockUpdateModuleData`, `SupplyCenterDockUpdate`
- `zh_logic::modules::update::supply_center_production_exit_update`: `SupplyCenterProductionExitUpdateModuleData`, `SupplyCenterProductionExitUpdate`
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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/HordeUpdate.h`
```cpp
// FILE: HordeUpdate.h //////////////////////////////////////////////////////////////////////////
// Author: Steven Johnson, Feb 2002
// Desc:   Horde update module
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/UpdateModule.h"
#include "Common/KindOf.h"

class SimpleObjectIterator;
class UpgradeTemplate;

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------

// TheSuperHackers @bugfix xezon 15/09/2025 Adds a new horde action to select a fixed implementation.
//
// TheSuperHackers @todo If we want more control over the horde setup, then we need to implement
// filters for separate weapon bonuses and required upgrades. But adding more behavior modules will
// be a performance concern. Example INI setup:
//
// Behavior = HordeUpdate ModuleTag
//   Action = NATIONALISM
//   ;ApplyWeaponBonus = NATIONALISM
//   UpgradeRequired = Upgrade_Nationalism
// End
//
enum HordeActionType CPP_11(: Int)
{
	HORDEACTION_HORDE, ///< Classic action, applies the Horde bonus correctly, but Nationalism and Fanaticism bonuses are not removed after leaving horde.
	HORDEACTION_HORDE_FIXED, ///< Applies the Horde, Nationalism and Fanaticism bonuses correctly.

	HORDEACTION_COUNT,

#if RETAIL_COMPATIBLE_CRC || PRESERVE_PERPETUAL_HORDE_BONUS
	HORDEACTION_DEFAULT = HORDEACTION_HORDE,
#else
	HORDEACTION_DEFAULT = HORDEACTION_HORDE_FIXED, ///< Does not change unmodified retail game behavior, because all its horde update modules explicitly set Action = HORDE.
#endif
};

#ifdef DEFINE_HORDEACTION_NAMES
static const char *const TheHordeActionTypeNames[] =
{
	"HORDE",
	"HORDE_FIXED",

	nullptr
};
static_assert(ARRAY_SIZE(TheHordeActionTypeNames) == HORDEACTION_COUNT + 1, "Incorrect array size");
#endif

//-------------------------------------------------------------------------------------------------
class HordeUpdateModuleData : public ModuleData
{
public:
  UnsignedInt								m_updateRate;   ///< how often to recheck our horde status
	KindOfMaskType						m_kindof;				///< the kind(s) of units that count towards hordeness
	Int												m_minCount;		  ///< min count to get "horde" status
  Real											m_minDist;      ///< min dist to contribute to hordeness
	Bool											m_alliesOnly;		///< if true, only allied units count towards hordeness
	Bool											m_exactMatch;		///< if true, only exact same type of units count towards hordeness
	Real											m_rubOffRadius;///< If I am this close to another guy who is a true hordesman, it'll rub off on me
	HordeActionType						m_action;				///< what to do if we get hordeness
	Bool											m_allowedNationalism; ///< Nationalism is hard coded.  Yeah!  Add to the goodness with this flag instead of rewriting after Alpha.
	std::vector<AsciiString>	m_flagSubObjNames;		///< name(s) of the flag sub obj

	HordeUpdateModuleData();
	static void buildFieldParse(MultiIniFieldParse& p);

private:

};

// ------------------------------------------------------------------------------------------------
class HordeUpdateInterface
{
public:
	virtual Bool isInHorde() const = 0;
	virtual Bool hasFlag() const = 0;
	virtual Bool isTrueHordeMember() const = 0;
	virtual Bool isAllowedNationalism() const = 0;
	virtual HordeActionType getHordeActionType() const = 0;
};

//-------------------------------------------------------------------------------------------------
class HordeUpdate : public UpdateModule, public HordeUpdateInterface
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( HordeUpdate, "HordeUpdate" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( HordeUpdate, HordeUpdateModuleData )

public:
	HordeUpdate( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	virtual HordeUpdateInterface *getHordeUpdateInterface() override { return this; }

	virtual void onDrawableBoundToObject() override;
	virtual UpdateSleepTime update() override;	///< update this object's AI

	virtual Bool isInHorde() const override { return m_inHorde; }
	virtual Bool hasFlag() const override { return m_hasFlag; }
	virtual Bool isTrueHordeMember() const override { return m_trueHordeMember && m_inHorde; }
	virtual Bool isAllowedNationalism() const override;
	virtual HordeActionType getHordeActionType() const override { return getHordeUpdateModuleData()->m_action; };

protected:

	void showHideFlag(Bool show);
	void joinOrLeaveHorde(SimpleObjectIterator *iter, Bool join);

private:
	UnsignedInt m_lastHordeRefreshFrame; //Just like it sounds
	Bool				m_inHorde;				 //I may be a true member, or I may merely inherit hordehood from a neighbor who is
	Bool				m_trueHordeMember; //meaning, I have enough hordesmen near me to qualify
	Bool				m_hasFlag;

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/HordeUpdate.cpp`
```cpp
// FILE: HordeUpdate.cpp ////////////////////////////////////////////////////////////////////////
// Author: Steven Johnson, Feb 2002
// Desc:   HordeUpdate module
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#define DEFINE_HORDEACTION_NAMES
#include "Common/Player.h"
#include "Common/ThingTemplate.h"
#include "Common/Upgrade.h"
#include "Common/Xfer.h"
#include "Common/BitFlagsIO.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Weapon.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/HordeUpdate.h"
#include "GameClient/Drawable.h"

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------




static HordeUpdateInterface* getHUI(Object* obj)
{
	for( BehaviorModule** u = obj->getBehaviorModules(); *u; ++u )
	{
		HordeUpdateInterface* hui = (*u)->getHordeUpdateInterface();
		if( hui )
			return hui;
	}
	return nullptr;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------

class PartitionFilterHordeMember : public PartitionFilter
{
private:
	Object* m_obj;
	const HordeUpdateModuleData* m_data;

public:

	PartitionFilterHordeMember(Object* obj, const HordeUpdateModuleData* data) : m_obj(obj), m_data(data) { }

#if defined(RTS_DEBUG)
	virtual const char* debugGetName() override { return "PartitionFilterHordeMember"; }
#endif

	virtual Bool allow(Object *objOther) override
	{
#if !RETAIL_COMPATIBLE_CRC
		// TheSuperHackers @bugfix Stubbjax 07/08/2026 Prevent dead units from being considered for horde membership.
		if (objOther->isEffectivelyDead())
			return false;
#endif
		
		// must be exact same type as us (well, maybe)
		if (m_data->m_exactMatch && m_obj->getTemplate() != objOther->getTemplate())
			return false;

		// can only horde with other horde-able things
		HordeUpdateInterface *hui = getHUI(objOther);
		if( !hui )
			return false;

		// must match the kindof flags (if any)
		if (!objOther->isKindOfMulti(m_data->m_kindof, KINDOFMASK_NONE))
			return false;

		// must be our ally (well, maybe)
		if (m_data->m_alliesOnly && m_obj->getRelationship(objOther) != ALLIES)
			return false;

		// doh
		if (m_obj->isOffMap() != objOther->isOffMap())
			return false;

		// guess it's horde-worthy!
		return true;
	}
};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------

const Int DEFAULT_UPDATE_RATE = LOGICFRAMES_PER_SECOND;

//-------------------------------------------------------------------------------------------------
HordeUpdateModuleData::HordeUpdateModuleData()
	: m_updateRate(DEFAULT_UPDATE_RATE)
	, m_minCount(0)
	, m_minDist(0.0f)
	, m_rubOffRadius(20.0f)
	, m_alliesOnly(true)
	, m_exactMatch(false)
	, m_allowedNationalism(TRUE)
	, m_action(HORDEACTION_DEFAULT)
{
}

//-------------------------------------------------------------------------------------------------
/*static*/ void HordeUpdateModuleData::buildFieldParse(MultiIniFieldParse& p)
{
	ModuleData::buildFieldParse(p);

	static const FieldParse dataFieldParse[] =
	{
		{ "UpdateRate", INI::parseDurationUnsignedInt, nullptr, offsetof(HordeUpdateModuleData, m_updateRate) },
		{ "KindOf", KindOfMaskType::parseFromINI, nullptr, offsetof(HordeUpdateModuleData, m_kindof) },
		{ "Count", INI::parseInt, nullptr, offsetof(HordeUpdateModuleData, m_minCount) },
		{ "Radius", INI::parseReal, nullptr, offsetof(HordeUpdateModuleData, m_minDist) },
		{ "RubOffRadius", INI::parseReal, nullptr, offsetof(HordeUpdateModuleData, m_rubOffRadius) },
		{ "AlliesOnly", INI::parseBool, nullptr, offsetof(HordeUpdateModuleData, m_alliesOnly) },
		{ "ExactMatch", INI::parseBool, nullptr, offsetof(HordeUpdateModuleData, m_exactMatch) },
		{ "Action", INI::parseIndexList, TheHordeActionTypeNames, offsetof(HordeUpdateModuleData, m_action) },
		{ "FlagSubObjectNames", INI::parseAsciiStringVector, nullptr, offsetof(HordeUpdateModuleData, m_flagSubObjNames) },
		{ "AllowedNationalism", INI::parseBool, nullptr, offsetof(HordeUpdateModuleData, m_allowedNationalism) },
		{ nullptr, nullptr, nullptr, 0 }
	};
	p.add(dataFieldParse);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------------------
HordeUpdate::HordeUpdate( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{
	m_inHorde = FALSE;
	m_hasFlag = FALSE;
	m_lastHordeRefreshFrame = TheGameLogic->getFrame();
	m_trueHordeMember = FALSE;

	UnsignedInt delay = getHordeUpdateModuleData()->m_updateRate;
	setWakeFrame(getObject(), UPDATE_SLEEP(GameLogicRandomValue(1, delay)));
}

//-------------------------------------------------------------------------------------------------
HordeUpdate::~HordeUpdate()
{

}

//-------------------------------------------------------------------------------------------------
Bool HordeUpdate::isAllowedNationalism() const
{
	const HordeUpdateModuleData *data = getHordeUpdateModuleData();
	return data->m_allowedNationalism;
}

//-------------------------------------------------------------------------------------------------
/** @todo I think we should model the horde list ... so we can do all this without doing
  * all this scanning, plus we can give exactly 1 flag to the right person in the
	* center of the horde which I think would look better (CBD)
	*
	* Redesign occurred 10/15, where the flags have been removed replaced by a teraindecal for each horde member
	* Thank You for reading, MLorenzen
*/
// ------------------------------------------------------------------------------------------------
void HordeUpdate::joinOrLeaveHorde(SimpleObjectIterator *iter, Bool join)
{
	// give/remove bonus effects
	if( m_inHorde != join )
	{
		m_inHorde = join;

		if( AIUpdateInterface *ai = getObject()->getAIUpdateInterface() )
		{
			const HordeUpdateModuleData* md = getHordeUpdateModuleData();
			ai->evaluateMoraleBonus(m_inHorde, md->m_allowedNationalism, md->m_action);
		}
		else
		{
			DEBUG_CRASH(( "HordeUpdate::joinOrLeaveHorde - We (%s) must have an AI to benefit from horde",
										getObject()->getTemplate()->getName().str() ));
		}
	}
}

//-------------------------------------------------------------------------------------------------
void HordeUpdate::showHideFlag(Bool show)
{
	const HordeUpdateModuleData* d = getHordeUpdateModuleData();
	if (!d->m_flagSubObjNames.empty())
	{
		Drawable* draw = getObject()->getDrawable();
		Object *obj = getObject();
		if (draw && obj)
		{
			for (std::vector<AsciiString>::const_iterator it = d->m_flagSubObjNames.begin(); it != d->m_flagSubObjNames.end(); ++it)
				draw->showSubObject(*it, show);
			draw->updateSubObjects();
		}
	}
}

//-------------------------------------------------------------------------------------------------
/**
	this is called whenever a drawable is bound to the object.
	drawable is NOT guaranteed to be non-null.
*/
void HordeUpdate::onDrawableBoundToObject()
{
	/////////////showHideFlag(m_hasFlag);
	showHideFlag(FALSE);//hide the flag
}

//-------------------------------------------------------------------------------------------------
UpdateSleepTime HordeUpdate::update()
{


	//This code handles decals and ONLY decals!
	Object *obj = getObject();

	if ( ! obj )
		return UPDATE_SLEEP_FOREVER;

	const HordeUpdateModuleData *md = getHordeUpdateModuleData();

	Bool wasInHorde = m_inHorde;

	// This is a sticky situation, where refreshing the model state (like from default to damaged, for example)
	// will rebuild the terrain decal and set its size to the default size.... since Vehicles have a special size,
	// we want to keep it fresh, here, but not do the hording test every frame...
	Bool isInfantry = ( obj->isKindOf(KINDOF_INFANTRY) );
	if ( isInfantry || (TheGameLogic->getFrame() > m_lastHordeRefreshFrame + md->m_updateRate) )
	{
		m_lastHordeRefreshFrame = TheGameLogic->getFrame();

		PartitionFilterHordeMember hmFilter(getObject(), md);
		PartitionFilter *filters[] = { &hmFilter, nullptr };
		SimpleObjectIterator *iter = ThePartitionManager->iterateObjectsInRange(getObject(), md->m_minDist, FROM_BOUNDINGSPHERE_3D, filters);
		MemoryPoolObjectHolder hold(iter);

		if ((iter->getCount() >= md->m_minCount - 1) )//we really are in the thick part of the horde
		{
			m_inHorde = TRUE;
			m_trueHordeMember = TRUE;
		}
		else
		{
			m_inHorde = FALSE;
			m_trueHordeMember = FALSE;/// unless...

			Real rubOffRadiusSq = sqr(md->m_rubOffRadius);
			for (Object* other = iter->first(); other; other = iter->next())
			{
				HordeUpdateInterface* hui = getHUI(other);
				if ( hui != nullptr && hui->isTrueHordeMember() )
				{
					Real dist = ThePartitionManager->getDistanceSquared(getObject(), other, FROM_CENTER_2D);

					if (dist <= rubOffRadiusSq )
					{
						m_inHorde = TRUE;
						break;
					}

				}
			}
		}

		AIUpdateInterface *ai = getObject()->getAIUpdateInterface();
		if( ai )
			ai->evaluateMoraleBonus(m_inHorde, md->m_allowedNationalism, md->m_action);

	}






	// This is a sticky situation, where refreshing the model state (like from default to damaged, for example)
	// will rebuild the terrain decal and set its size to the default size.... since Vehicles have a special size,
	// we want to keep it fresh, here, but not do the hording test every frame...
	// This is a weak solution, in that It causes this update to fight the default behavior of the model state methods,
	// But in the interest of not breaking the model state changing logic for five hundred other units a week before golden,
	// this is my solution... If anyone gets this note on the next project... please please please, fix the resetting of the
	// shadows/terrain decals in W3DModelDraw. Thanks, ML

	Drawable* draw = getObject()->getDrawable();
	if ( draw && ! obj->isEffectivelyDead() )
	{
		if( TheGameLogic->getDrawIconUI() )
		{
			if(m_inHorde && !obj->isKindOf( KINDOF_PORTABLE_STRUCTURE ) )// this not is a ride-on for overlord
			{

				TerrainDecalType nuType;// uninitialized

				if ( isInfantry )
				{
					if( obj->testWeaponBonusCondition( WEAPONBONUSCONDITION_NATIONALISM ) == TRUE )
          {
            if ( obj->testWeaponBonusCondition( WEAPONBONUSCONDITION_FANATICISM ) == TRUE )
  						nuType = ( TERRAIN_DECAL_HORDE_WITH_FANATICISM_UPGRADE );
            else
  						nuType = ( TERRAIN_DECAL_HORDE_WITH_NATIONALISM_UPGRADE );
          }
					else
						nuType =(TERRAIN_DECAL_HORDE);

					if (nuType != draw->getTerrainDecalType())
						draw->setTerrainDecal(nuType);
				}
				else
				{
					Real size = 3.5f * obj->getGeometryInfo().getMajorRadius();
					draw->setTerrainDecalSize( size, size );

					if( obj->testWeaponBonusCondition( WEAPONBONUSCONDITION_NATIONALISM ) == TRUE )
          {
            if ( obj->testWeaponBonusCondition( WEAPONBONUSCONDITION_FANATICISM ) == TRUE )
  						nuType = ( TERRAIN_DECAL_HORDE_WITH_FANATICISM_UPGRADE );
            else
  						nuType = ( TERRAIN_DECAL_HORDE_WITH_NATIONALISM_UPGRADE );
          }
					else
						nuType = (TERRAIN_DECAL_HORDE_VEHICLE);

					if (nuType != draw->getTerrainDecalType())
						draw->setTerrainDecal(nuType);
				}
			}


		}
		else
		{
			//Scripts have disabled icons so don't draw these!
			draw->setTerrainDecal( TERRAIN_DECAL_NONE );
		}

		if ( ! wasInHorde && m_inHorde  )
				draw->setTerrainDecalFadeTarget(1.0f, 0.03f);
		else if ( wasInHorde && ! m_inHorde )
				draw->setTerrainDecalFadeTarget(0.0f, -0.03f);
	}


	return isInfantry ? UPDATE_SLEEP(md->m_updateRate) : UPDATE_SLEEP_NONE;

}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void HordeUpdate::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void HordeUpdate::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	xfer->xferBool( &m_inHorde );
	xfer->xferBool( &m_hasFlag );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void HordeUpdate::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
