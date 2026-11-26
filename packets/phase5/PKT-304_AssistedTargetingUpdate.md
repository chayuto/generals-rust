# Translation Packet `PKT-304`: `AssistedTargetingUpdate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-304`
- **Work Package:** `WP-5.4` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/update/assisted_targeting_update.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (11):**
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/ThingFactory.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ThingFactory.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/AssistedTargetingUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AssistedTargetingUpdate.h`
  - `GameLogic/Module/LaserUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/LaserUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/Weapon.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h`
  - `GameLogic/WeaponSet.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/WeaponSet.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (4):**
  - `"AssistingClipSize" => INI::parseInt`
  - `"AssistingWeaponSlot" => INI::parseLookupList`
  - `"LaserFromAssisted" => INI::parseAsciiString`
  - `"LaserToTarget" => INI::parseAsciiString`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (1 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if RTS_GENERALS || !RETAIL_COMPATIBLE_CRC`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AssistedTargetingUpdate.h`
```cpp
// FILE: AssistedTargetingUpdate.h /////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, September 2002
// Desc:   Outside influences can tell me to attack something out of my normal targeting range
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/UpdateModule.h"

//-------------------------------------------------------------------------------------------------
class AssistedTargetingUpdateModuleData : public UpdateModuleData
{
public:
	Int m_clipSize;
	WeaponSlotType m_weaponSlot;

	AsciiString m_laserFromAssistedName;
	AsciiString m_laserToTargetName;

	AssistedTargetingUpdateModuleData()
	{
		m_clipSize = 1;
		m_weaponSlot = PRIMARY_WEAPON;
	}

	static void buildFieldParse(MultiIniFieldParse& p);
};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class AssistedTargetingUpdate : public UpdateModule
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( AssistedTargetingUpdate, "AssistedTargetingUpdate" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( AssistedTargetingUpdate, AssistedTargetingUpdateModuleData )

public:

	AssistedTargetingUpdate( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	virtual UpdateSleepTime update() override;

	Bool isFreeToAssist() const;
	void assistAttack( const Object *requestingObject, Object *victimObject );

private:
	void makeFeedbackLaser( const ThingTemplate *laserTemplate, const Object *from, const Object *to );

	const ThingTemplate *m_laserFromAssisted;
	const ThingTemplate *m_laserToTarget;
};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/AssistedTargetingUpdate.cpp`
```cpp
// FILE: AssistedTargetingUpdate.cpp /////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, September 2002
// Desc:   Outside influences can tell me to attack something out of my normal targeting range
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#define DEFINE_WEAPONSLOTTYPE_NAMES
#include "Common/Player.h"
#include "Common/ThingFactory.h"
#include "Common/Xfer.h"
#include "GameLogic/Object.h"
#include "GameLogic/Weapon.h"
#include "GameLogic/WeaponSet.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/AssistedTargetingUpdate.h"
#include "GameLogic/Module/LaserUpdate.h"





//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void AssistedTargetingUpdateModuleData::buildFieldParse(MultiIniFieldParse& p)
{
	UpdateModuleData::buildFieldParse(p);
	static const FieldParse dataFieldParse[] =
	{
		{ "AssistingClipSize",		INI::parseInt,		nullptr, offsetof( AssistedTargetingUpdateModuleData, m_clipSize ) },
		{ "AssistingWeaponSlot",	INI::parseLookupList,	TheWeaponSlotTypeNamesLookupList, offsetof( AssistedTargetingUpdateModuleData, m_weaponSlot ) },
		{ "LaserFromAssisted",		INI::parseAsciiString,				nullptr, offsetof( AssistedTargetingUpdateModuleData, m_laserFromAssistedName ) },
		{ "LaserToTarget",				INI::parseAsciiString,				nullptr, offsetof( AssistedTargetingUpdateModuleData, m_laserToTargetName ) },
		{ nullptr, nullptr, nullptr, 0 }
	};
	p.add(dataFieldParse);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
AssistedTargetingUpdate::AssistedTargetingUpdate( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{
	const AssistedTargetingUpdateModuleData* d = getAssistedTargetingUpdateModuleData();

	m_laserFromAssisted = TheThingFactory->findTemplate(d->m_laserFromAssistedName);
	m_laserToTarget = TheThingFactory->findTemplate(d->m_laserToTargetName);
#if RTS_GENERALS || !RETAIL_COMPATIBLE_CRC
	// TheSuperHackers @info Zero Hour needs the update to run once to avoid mismatches with retail.
	setWakeFrame(getObject(), UPDATE_SLEEP_FOREVER);
#endif
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
AssistedTargetingUpdate::~AssistedTargetingUpdate()
{
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Bool AssistedTargetingUpdate::isFreeToAssist() const
{
	// The reload times of my two weapons are tied together, so Ready is indicative of either.
	const Object *me = getObject();
	if( !me->isAbleToAttack() )
		return FALSE;// This will cover under construction among other things

	Bool ready = me->getCurrentWeapon() && me->getCurrentWeapon()->getStatus() == READY_TO_FIRE;
	return ready;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void AssistedTargetingUpdate::assistAttack( const Object *requestingObject, Object *victimObject )
{
	const AssistedTargetingUpdateModuleData *md = getAssistedTargetingUpdateModuleData();
	Object *me = getObject();
	if( !me->getAI() )
		return;

	// lock it just till the weapon is empty or the attack is "done"
	me->setWeaponLock( md->m_weaponSlot, LOCKED_TEMPORARILY );
	me->getAI()->aiAttackObject( victimObject, md->m_clipSize, CMD_FROM_AI );

	if( m_laserFromAssisted )
		makeFeedbackLaser( m_laserFromAssisted, requestingObject, me );
	if( m_laserToTarget )
		makeFeedbackLaser( m_laserToTarget, me, victimObject );
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void AssistedTargetingUpdate::makeFeedbackLaser( const ThingTemplate *laserTemplate, const Object *from, const Object *to )
{
	if( !getObject()->getControllingPlayer() )
		return;

	Team *laserTeam = getObject()->getControllingPlayer()->getDefaultTeam();
	Object *laser = TheThingFactory->newObject( laserTemplate, laserTeam );
	if( !laser )
		return;

	Drawable *draw = laser->getDrawable();
	static const NameKeyType key_LaserUpdate = NAMEKEY( "LaserUpdate" );
	LaserUpdate *update = (LaserUpdate*)draw->findClientUpdateModule( key_LaserUpdate );
	if( !update )
	{
		TheGameLogic->destroyObject( laser );
		return;
	}

	// Give it a good basis in reality to ensure it can draw when on screen.
	laser->setPosition(from->getPosition());

	update->initLaser( getObject(), to, from->getPosition(), to->getPosition(), "" );
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
UpdateSleepTime AssistedTargetingUpdate::update()
{
	return UPDATE_SLEEP_FOREVER;
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void AssistedTargetingUpdate::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void AssistedTargetingUpdate::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void AssistedTargetingUpdate::loadPostProcess()
{
	// extend base class
	UpdateModule::loadPostProcess();
}
```
