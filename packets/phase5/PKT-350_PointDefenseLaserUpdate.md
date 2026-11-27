# Translation Packet `PKT-350`: `PointDefenseLaserUpdate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-350`
- **Work Package:** `WP-5.4` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/update/point_defense_laser_update.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (15):**
  - `Common/BitFlagsIO.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/BitFlagsIO.h`
  - `Common/KindOf.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/KindOf.h`
  - `Common/RandomValue.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/RandomValue.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/PhysicsUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PhysicsUpdate.h`
  - `GameLogic/Module/PointDefenseLaserUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PointDefenseLaserUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/ObjectIter.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ObjectIter.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `GameLogic/Weapon.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (6):**
  - `"WeaponTemplate" => INI::parseWeaponTemplate`
  - `"PrimaryTargetTypes" => KindOfMaskType::parseFromINI`
  - `"SecondaryTargetTypes" => KindOfMaskType::parseFromINI`
  - `"ScanRate" => INI::parseDurationUnsignedInt`
  - `"ScanRange" => INI::parseReal`
  - `"PredictTargetVelocityFactor" => INI::parseReal`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (5 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferObjectID( &m_bestTargetID )`
  3. `xferBool( &m_inRange )`
  4. `xferInt( &m_nextScanFrames )`
  5. `xferInt( &m_nextShotAvailableInFrames )`
- **PRNG Calls (2 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValue( 0, 3 )`
  - `GameLogicRandomValue( 0, 3 )`

## 2.5 Existing Rust Symbol Index (Use These Exact Paths)
- `zh_logic::modules::update::assisted_targeting_update`: `AssistedTargetingUpdateModuleData`, `AssistedTargetingUpdate`
- `zh_logic::modules::update::auto_deposit_update`: `AutoDepositUpdateModuleData`, `AutoDepositUpdate`
- `zh_logic::modules::update::auto_find_healing_update`: `AutoFindHealingUpdateModuleData`, `AutoFindHealingUpdate`
- `zh_logic::modules::update::base_renerate_update`: `BaseRegenerateUpdateModuleData`, `BaseRegenerateUpdate`
- `zh_logic::modules::update::battle_plan_update`: `BattlePlanStatus`, `TransitionStatus`, `BattlePlanBonuses`, `PlayerBattlePlans`, `BattlePlanUpdateModuleData`, `BattlePlanUpdate`
- `zh_logic::modules::update::command_button_hunt_update`: `CommandButtonHuntUpdateModuleData`, `CommandButtonHuntUpdate`
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
- `zh_logic::modules::update::helicopter_slow_death_update`: `HelicopterSlowDeathBehaviorModuleData`, `HelicopterSlowDeathBehavior`
- `zh_logic::modules::update::horde_update`: `HordeActionType`, `TerrainDecalType`, `HordeUpdateModuleData`, `HordeUpdate`
- `zh_logic::modules::update::lifetime_update`: `LifetimeUpdateModuleData`, `LifetimeUpdate`
- `zh_logic::modules::update::missile_launcher_building_update`: `DoorState`, `MissileLauncherBuildingUpdateModuleData`, `MissileLauncherBuildingUpdate`
- `zh_logic::modules::update::ocl_update`: `FactionOclInfo`, `OclUpdateModuleData`, `OclUpdate`
- `zh_logic::modules::update::physics_update`: `STICK_TO_GROUND`, `ALLOW_BOUNCE`, `APPLY_FRICTION2D_WHEN_AIRBORNE`, `UPDATE_EVER_RUN`, `WAS_AIRBORNE_LAST_FRAME`, `ALLOW_COLLIDE_FORCE`, `ALLOW_TO_FALL`, `HAS_PITCHROLLYAW`, `IMMUNE_TO_FALLING_DAMAGE`, `IS_IN_FREEFALL`, `IS_IN_UPDATE`, `IS_STUNNED`, `TURN_NEGATIVE`, `TURN_NONE`, ... (+4)
- `zh_logic::modules::update::point_defense_laser_update`: `PointDefenseLaserUpdateModuleData`, `PointDefenseLaserUpdate`
- `zh_logic::modules::update::power_plant_update`: `PowerPlantUpdateModuleData`, `PowerPlantUpdate`
- `zh_logic::modules::update::production_update`: `PRODUCTION_INVALID`, `PRODUCTION_UNIT`, `PRODUCTION_UPGRADE`, `PRODUCTION_ID_INVALID`, `QuantityModifier`, `ProductionUpdateModuleData`, `FACET_MASK`, `ProductionEntry`, `DoorInfo`, `ProductionUpdate`
- `zh_logic::modules::update::projectile_stream_update`: `MAX_PROJECTILE_STREAM`, `ProjectileStreamUpdateModuleData`, `ProjectileStreamUpdate`
- `zh_logic::modules::update::prone_update`: `ProneUpdateModuleData`, `ProneUpdate`
- `zh_logic::modules::update::queue_production_exit_update`: `QueueProductionExitUpdateModuleData`, `QueueProductionExitUpdate`
- `zh_logic::modules::update::radar_update`: `RadarUpdateModuleData`, `RadarUpdate`
- `zh_logic::modules::update::radius_decal_update`: `RadiusDecalUpdateModuleData`, `RadiusDecalUpdate`
- `zh_logic::modules::update::repair_dock_update`: `RepairDockUpdateModuleData`, `RepairDockUpdate`
- `zh_logic::modules::update::slaved_update`: `REPAIRSTATE_NONE`, `REPAIRSTATE_UNPACKING`, `REPAIRSTATE_PACKING`, `REPAIRSTATE_READY`, `REPAIRSTATE_EXTENDING`, `REPAIRSTATE_RETRACTING`, `REPAIRSTATE_WELDING`, `SlavedUpdateModuleData`, `SlavedUpdate`
- `zh_logic::modules::update::smart_bomb_target_homing_update`: `SmartBombTargetHomingUpdateModuleData`, `SmartBombTargetHomingUpdate`
- `zh_logic::modules::update::spawn_point_production_exit_update`: `MAX_SPAWN_POINTS`, `SpawnPointProductionExitUpdateModuleData`, `SpawnPointProductionExitUpdate`
- `zh_logic::modules::update::special_ability_update`: `SPECIAL_ABILITY_HUGE_DISTANCE`, `STATE_NONE`, `STATE_PACKING`, `STATE_UNPACKING`, `STATE_PACKED`, `STATE_UNPACKED`, `AbilityFeedback`, `SpecialAbilityUpdateModuleData`, `SpecialAbilityUpdate`
- `zh_logic::modules::update::spectre_gunship_deployment_update`: `COMMAND_FIRED_BY_SCRIPT`, `GunshipCreateLocType`, `SpectreGunshipDeploymentUpdateModuleData`, `SpectreGunshipDeploymentUpdate`
- `zh_logic::modules::update::spy_vision_update`: `SpyVisionUpdateModuleData`, `SpyVisionUpdate`, `fn`
- `zh_logic::modules::update::stealth_detector_update`: `StealthDetectorUpdateModuleData`, `StealthDetectorUpdate`
- `zh_logic::modules::update::stealth_update`: `STEALTH_NOT_WHILE_ATTACKING`, `STEALTH_NOT_WHILE_MOVING`, `STEALTH_NOT_WHILE_USING_ABILITY`, `STEALTH_NOT_WHILE_FIRING_PRIMARY`, `STEALTH_NOT_WHILE_FIRING_SECONDARY`, `STEALTH_NOT_WHILE_FIRING_TERTIARY`, `STEALTH_ONLY_WITH_BLACK_MARKET`, `STEALTH_NOT_WHILE_TAKING_DAMAGE`, `STEALTH_NOT_WHILE_RIDERS_ATTACKING`, `STEALTH_NOT_WHILE_FIRING_WEAPON`, `STEALTH_LEVEL_NAMES`, `StealthUpdateModuleData`, `StealthUpdate`
- `zh_logic::modules::update::structure_topple_update`: `STPHASE_INITIAL`, `STPHASE_DELAY`, `STPHASE_FINAL`, `ST_PHASE_COUNT`, `TOPPLESTATE_STANDING`, `TOPPLESTATE_WAITINGFORTOPPLESTART`, `TOPPLESTATE_TOPPLING`, `TOPPLESTATE_WAITINGFORDONE`, `TOPPLESTATE_DONE`, `StructureToppleUpdateModuleData`, `StructureToppleUpdate`
- `zh_logic::modules::update::supply_center_dock_update`: `SupplyCenterDockUpdateModuleData`, `SupplyCenterDockUpdate`
- `zh_logic::modules::update::supply_center_production_exit_update`: `SupplyCenterProductionExitUpdateModuleData`, `SupplyCenterProductionExitUpdate`
- `zh_logic::modules::update::supply_warehouse_dock_update`: `SupplyWarehouseDockUpdateModuleData`, `SupplyWarehouseDockUpdate`
- `zh_logic::modules::update::tensile_formation_update`: `TensileFormationUpdateModuleData`, `TensileLink`, `TensileFormationUpdate`
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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PointDefenseLaserUpdate.h`
```cpp
// FILE: PointDefenseLaserUpdate.cpp //////////////////////////////////////////////////////////////////////////
// Author: Kris Morness, August 2002
// Desc:   Update module to handle independent targeting of point defense laser.
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/KindOf.h"
#include "GameLogic/Module/UpdateModule.h"

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
class ThingTemplate;
class WeaponTemplate;


//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class PointDefenseLaserUpdateModuleData : public ModuleData
{
public:
	WeaponTemplate	*m_weaponTemplate;
	KindOfMaskType	m_primaryTargetKindOf;
	KindOfMaskType  m_secondaryTargetKindOf;
	UnsignedInt			m_scanFrames;
	Real						m_scanRange;
	Real						m_velocityFactor;

	PointDefenseLaserUpdateModuleData();
	static void buildFieldParse(MultiIniFieldParse& p);

private:

};

//-------------------------------------------------------------------------------------------------
/** The default	update module */
//-------------------------------------------------------------------------------------------------
class PointDefenseLaserUpdate : public UpdateModule
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( PointDefenseLaserUpdate, "PointDefenseLaserUpdate" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( PointDefenseLaserUpdate, PointDefenseLaserUpdateModuleData );

public:

	PointDefenseLaserUpdate( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	virtual void onObjectCreated() override;
	virtual UpdateSleepTime update() override;

	Object* scanClosestTarget();
	void fireWhenReady();

protected:

	ObjectID m_bestTargetID;
	Bool m_inRange;
	Int m_nextScanFrames;
	Int m_nextShotAvailableInFrames;
};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/PointDefenseLaserUpdate.cpp`
```cpp
// FILE: PointDefenseLaserUpdate.cpp //////////////////////////////////////////////////////////////////////////
// Author: Kris Morness, August 2002
// Desc:   Update module to handle independent targeting of point defense laser.
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/BitFlagsIO.h"
#include "Common/RandomValue.h"
#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"

#include "GameClient/Drawable.h"

#include "GameLogic/GameLogic.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectIter.h"
#include "GameLogic/Module/PointDefenseLaserUpdate.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Weapon.h"



//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
PointDefenseLaserUpdateModuleData::PointDefenseLaserUpdateModuleData()
{
	m_weaponTemplate		= nullptr;
	m_scanFrames				= 0;
	m_scanRange					= 0.0f;
	m_velocityFactor		= 0.0f;
}

//-------------------------------------------------------------------------------------------------
/*static*/ void PointDefenseLaserUpdateModuleData::buildFieldParse(MultiIniFieldParse& p)
{
	ModuleData::buildFieldParse(p);

	static const FieldParse dataFieldParse[] =
	{
		{ "WeaponTemplate",				INI::parseWeaponTemplate,				nullptr, offsetof( PointDefenseLaserUpdateModuleData, m_weaponTemplate ) },
		{ "PrimaryTargetTypes",		KindOfMaskType::parseFromINI,								nullptr, offsetof( PointDefenseLaserUpdateModuleData, m_primaryTargetKindOf ) },
		{ "SecondaryTargetTypes",	KindOfMaskType::parseFromINI,								nullptr, offsetof( PointDefenseLaserUpdateModuleData, m_secondaryTargetKindOf ) },
		{ "ScanRate",							INI::parseDurationUnsignedInt,	nullptr, offsetof( PointDefenseLaserUpdateModuleData, m_scanFrames ) },
		{ "ScanRange",						INI::parseReal,									nullptr, offsetof( PointDefenseLaserUpdateModuleData, m_scanRange ) },
		{ "PredictTargetVelocityFactor", INI::parseReal,					nullptr, offsetof( PointDefenseLaserUpdateModuleData, m_velocityFactor ) },
		{ nullptr, nullptr, nullptr, 0 }
	};
	p.add(dataFieldParse);
}

//-------------------------------------------------------------------------------------------------
PointDefenseLaserUpdate::PointDefenseLaserUpdate( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{
	m_bestTargetID = INVALID_ID;
	m_nextScanFrames = 0;
	m_nextShotAvailableInFrames = 0;
	m_inRange  					= false;
	setWakeFrame(getObject(), UPDATE_SLEEP_NONE);// No starting sleep, but we want to sleep later.
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
PointDefenseLaserUpdate::~PointDefenseLaserUpdate()
{

}


//-------------------------------------------------------------------------------------------------
void PointDefenseLaserUpdate::onObjectCreated()
{
	const PointDefenseLaserUpdateModuleData *data = getPointDefenseLaserUpdateModuleData();

	//Make sure we have a weapon template
	if( !data->m_weaponTemplate )
	{
		DEBUG_CRASH( ("PointDefenseLaserUpdate for %s doesn't have a valid weapon template",
			getObject()->getTemplate()->getName().str() ) );
		return;
	}

	//Make sure our firing range is smaller than the scan range.
	WeaponBonus bonus;
	bonus.clear();
	Real attackRange = data->m_weaponTemplate->getAttackRange( bonus );
	if( data->m_scanRange <= attackRange )
	{
		DEBUG_CRASH( ("PointDefenseLaserUpdate for %s requires the scan range (%.1f) being larger than the firing range (%.1f)",
			getObject()->getTemplate()->getName().str(), data->m_scanRange, attackRange ) );
	}
}

//-------------------------------------------------------------------------------------------------
/** The update callback. */
//-------------------------------------------------------------------------------------------------
UpdateSleepTime PointDefenseLaserUpdate::update()
{
/// @todo srj use SLEEPY_UPDATE here
	//*** HERE'S THE UPDATE PHILOSOPHY ***
	//The point defense laser typically has short range, high rate of fire, and shoots at incoming projectiles
	//that move fast. This amounts to a potentially very expensive system. Instead of frantically scanning for
	//targets, we will scan less frequently (data->m_scanFrames) in a larger radius (data->m_scanRange). When
	//this occurs, we'll store the "best" target, and track only that target until the next update or if it is
	//killed.

	Object *me = getObject();
	if( me->isEffectivelyDead() )
		return UPDATE_SLEEP_FOREVER;//No more laser fo you.

	const PointDefenseLaserUpdateModuleData *data = getPointDefenseLaserUpdateModuleData();

	//Optimized firing at acquired target
	if( m_nextScanFrames > 0 )
	{
		m_nextScanFrames--;
		fireWhenReady(); //Only happens if something is tracked.
		return UPDATE_SLEEP_NONE;
	}
	m_nextScanFrames = data->m_scanFrames;

	//Periodic scanning (expensive)
	if( scanClosestTarget() )
	{
		//1 frame can make a big difference so fire ASAP!
		fireWhenReady();
	}
	return UPDATE_SLEEP_NONE;
}

//-------------------------------------------------------------------------------------------------
void PointDefenseLaserUpdate::fireWhenReady()
{
	const PointDefenseLaserUpdateModuleData *data = getPointDefenseLaserUpdateModuleData();

	//Track our target
	Object *target = TheGameLogic->findObjectByID( m_bestTargetID );
	if( target )
	{
		WeaponBonus bonus;
		bonus.clear();
		Real fireRange = data->m_weaponTemplate->getAttackRange( bonus );
		Object *me = getObject();
		Real fDist = WWMath::Sqrt( ThePartitionManager->getDistanceSquared( me, target, FROM_CENTER_2D ) );
		if( fDist < fireRange )
		{
			//We are currently in range!
			m_inRange = true;
		}
		else
		{
			if( m_inRange )
			{
				//We were in range last frame, but the target has moved out of firing range, so
				//re-evaluate by forcing a new scan.
				m_nextScanFrames = GameLogicRandomValue( 0, 3 );
				m_bestTargetID = INVALID_ID;
				if( !m_nextScanFrames )
				{
					scanClosestTarget();
					m_nextScanFrames = data->m_scanFrames;
					target = nullptr; //Set target to nullptr so we don't shoot at it (might be out of range)
				}
			}
			else
			{
				//Not in range
				m_inRange = false;
			}
		}
	}

	if( m_nextShotAvailableInFrames > 0 )
	{
		//We can't fire this frame.
		m_nextShotAvailableInFrames--;
		return;
	}

	WeaponTemplate *wt = data->m_weaponTemplate;
	if( wt )
	{
		WeaponBonus bonus;

		//Fire control!
		if( target && m_inRange )
		{
			if( !target->isEffectivelyDead() )
			{
				Weapon* w = TheWeaponStore->allocateNewWeapon( wt, TERTIARY_WEAPON );
				w->loadAmmoNow( getObject() );
				w->fireWeapon( getObject(), target );
				deleteInstance(w);

				// And now that we have shot, set our internal reload timer.
				m_nextShotAvailableInFrames = wt->getDelayBetweenShots( bonus );
			}

			if( target->isEffectivelyDead() )
			{
				m_nextScanFrames = GameLogicRandomValue( 0, 3 );
				m_bestTargetID = INVALID_ID;
				if( !m_nextScanFrames )
				{
					scanClosestTarget();
					m_nextScanFrames = data->m_scanFrames;
				}
			}
		}
	}

}

//-------------------------------------------------------------------------------------------------
Object* PointDefenseLaserUpdate::scanClosestTarget()
{
	const PointDefenseLaserUpdateModuleData *data = getPointDefenseLaserUpdateModuleData();
	Object *me = getObject();
	Object *bestTargetOutOfRange[2] = { nullptr, nullptr };
	Object *bestTargetInRange[2] = { nullptr, nullptr };
	Real closestDist[2];
	Real closestOutsideRange[2];
	Int index;
	WeaponBonus bonus;
	bonus.clear();
	Real fireRange = data->m_weaponTemplate->getAttackRange( bonus );

	ObjectIterator *iter = ThePartitionManager->iterateObjectsInRange( me->getPosition(), data->m_scanRange, FROM_CENTER_2D );
	MemoryPoolObjectHolder hold(iter);

	for( Object *other = iter->first(); other; other = iter->next() )
	{
		if( other->isAnyKindOf( data->m_primaryTargetKindOf ) )
		{
			//Primary target type
			index = 0;
		}
		else if( other->isAnyKindOf( data->m_secondaryTargetKindOf ) )
		{
			//Secondary target type (use only if we can't find any primary targets)
			index = 1;
		}
		else
		{
			//Not a valid target.
			continue;
		}

		// Since we don't have an actual weapon or a weapon set, we lose all of the automatic checks.
		// "Borrow" the check for being an AA only laser to stop from shooting planes on airports.
		if( !other->isAirborneTarget() && !(data->m_weaponTemplate->getAntiMask() & WEAPON_ANTI_GROUND) )
			continue;

			// order matters: we want to know if I consider it to be an enemy, not vice versa
		if( getObject()->getRelationship( other ) != ENEMIES )
		{
			//Don't kill our friends!
			continue;
		}

		if( other->testStatus( OBJECT_STATUS_STEALTHED ) && !other->testStatus( OBJECT_STATUS_DETECTED ) && !other->testStatus( OBJECT_STATUS_DISGUISED ) )
		{
			//We can't see it.
			continue;
		}

		Real fDist = WWMath::Sqrt( ThePartitionManager->getDistanceSquared( me, other, FROM_CENTER_2D ) );

		if( fDist <= fireRange )
		{
			//Inside fire range (which one is closest?)
			if( !bestTargetInRange[index] || fDist < closestDist[index] )
			{
				closestDist[index] = fDist;
				bestTargetInRange[index] = other;
			}
		}
		else if( !bestTargetInRange[index] )
		{
			//Outside fire range.

			//Determine where the target will be based on current velocity using (m_velocityFactor * frames)
			if( data->m_velocityFactor != 0.0f && !other->isKindOf( KINDOF_IMMOBILE ) )
			{
				Coord3D pos;
				PhysicsBehavior *physics = other->getPhysics();
				if( physics )
				{
					pos.set( *physics->getVelocity() );
					pos.scale( data->m_velocityFactor );
					pos.add( *other->getPosition() );

					//Recalculate the distance.
					fDist = WWMath::Sqrt( ThePartitionManager->getDistanceSquared( me, other, FROM_CENTER_2D ) );
				}
			}

			//Now calculate the best outside range target.
			if( !bestTargetOutOfRange[index] || fDist < closestOutsideRange[index] )
			{
				closestOutsideRange[index] = fDist;
				bestTargetOutOfRange[index] = other;
			}
		}
	}

	if( bestTargetInRange[ 0 ] )
	{
		//This is the best primary target in range.
		m_bestTargetID = bestTargetInRange[ 0 ]->getID();
		m_inRange = true;
		return bestTargetInRange[ 0 ];
	}

	if( bestTargetInRange[ 1 ] )
	{
		//This is the best secondary target in range.
		m_bestTargetID = bestTargetInRange[ 1 ]->getID();
		m_inRange = true;
		return bestTargetInRange[ 1 ];
	}

	if( bestTargetOutOfRange[ 0 ] )
	{
		//This is the best primary target out of range.
		m_bestTargetID = bestTargetOutOfRange[ 0 ]->getID();
		m_inRange = false;
		return bestTargetOutOfRange[ 0 ];
	}

	if( bestTargetOutOfRange[ 1 ] )
	{
		//This is the best secondary target out of range.
		m_bestTargetID = bestTargetOutOfRange[ 1 ]->getID();
		m_inRange = false;
		return bestTargetOutOfRange[ 1 ];
	}

	//Utter failure -- nothing on the scope.
	m_bestTargetID = INVALID_ID;
	m_inRange = false;
	return nullptr;
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void PointDefenseLaserUpdate::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void PointDefenseLaserUpdate::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// best target ID
	xfer->xferObjectID( &m_bestTargetID );

	// in range
	xfer->xferBool( &m_inRange );

	// next scan frames
	xfer->xferInt( &m_nextScanFrames );

	// next shot available in frames
	xfer->xferInt( &m_nextShotAvailableInFrames );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void PointDefenseLaserUpdate::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
