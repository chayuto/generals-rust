# Translation Packet `PKT-341`: `MissileLauncherBuildingUpdate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-341`
- **Work Package:** `WP-5.4` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/update/missile_launcher_building_update.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (16):**
  - `Common/AudioEventRTS.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/AudioEventRTS.h`
  - `Common/GameAudio.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameAudio.h`
  - `Common/GlobalData.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/GlobalData.h`
  - `Common/INI.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/INI.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/SpecialPower.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/SpecialPower.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/FXList.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/FXList.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/MissileLauncherBuildingUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/MissileLauncherBuildingUpdate.h`
  - `GameLogic/Module/SpecialPowerModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpecialPowerModule.h`
  - `GameLogic/Module/SpecialPowerUpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpecialPowerUpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `PreRTS.h`
- **Implemented Module Facets (1):**
  - `SpecialPowerUpdateInterface* getSpecialPowerUpdateInterface()`
- **INI `FieldParse` Table Entries (10):**
  - `"SpecialPowerTemplate" => INI::parseSpecialPowerTemplate`
  - `"DoorOpenTime" => INI::parseDurationUnsignedInt`
  - `"DoorWaitOpenTime" => INI::parseDurationUnsignedInt`
  - `"DoorCloseTime" => INI::parseDurationUnsignedInt`
  - `"DoorOpeningFX" => INI::parseFXList`
  - `"DoorOpenFX" => INI::parseFXList`
  - `"DoorWaitingToCloseFX" => INI::parseFXList`
  - `"DoorClosingFX" => INI::parseFXList`
  - `"DoorClosedFX" => INI::parseFXList`
  - `"DoorOpenIdleAudio" => INI::parseAudioEventRTS`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (4 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUser( &m_doorState, sizeof( DoorStateType ) )`
  3. `xferUser( &m_timeoutState, sizeof( DoorStateType ) )`
  4. `xferUnsignedInt( &m_timeoutFrame )`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/MissileLauncherBuildingUpdate.h`
```cpp
// FILE: MissileLauncherBuildingUpdate.h /////////////////////////////////////////////////////////////////////////
// Author: Matthew D. Campbell, April 2002
// Desc:   Update will change model state conditions based on special power state
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/AudioEventRTS.h"
#include "Common/INI.h"
#include "GameLogic/Module/SpecialPowerUpdateModule.h"
class DamageInfo;
class SpecialPowerTemplate;
class SpecialPowerModule;
class FXList;

//-------------------------------------------------------------------------------------------------
class MissileLauncherBuildingUpdateModuleData : public UpdateModuleData
{
public:
	SpecialPowerTemplate *m_specialPowerTemplate;		///< pointer to the special power template
	UnsignedInt m_doorOpenTime;							///< in frames, time we should take to open the door
	UnsignedInt m_doorWaitOpenTime;					///< in frames, time we should leave the door open after firing the superweapon
	UnsignedInt m_doorClosingTime;					///< in frames, time it takes to close the door

	const FXList *m_openingFX;
	const FXList *m_openFX;
	const FXList *m_waitingToCloseFX;
	const FXList *m_closingFX;
	const FXList *m_closedFX;
	AudioEventRTS m_openIdleAudio;

	MissileLauncherBuildingUpdateModuleData()
	{
		m_specialPowerTemplate = nullptr;
		m_doorOpenTime = 0;
		m_doorWaitOpenTime = 0;
		m_doorClosingTime = 0;

		m_openingFX = m_openFX = m_waitingToCloseFX = m_closingFX = m_closedFX = nullptr;
	}

	static void buildFieldParse(MultiIniFieldParse& p)
	{
    UpdateModuleData::buildFieldParse(p);

		static const FieldParse dataFieldParse[] =
		{
			{ "SpecialPowerTemplate",	INI::parseSpecialPowerTemplate,					nullptr, offsetof( MissileLauncherBuildingUpdateModuleData, m_specialPowerTemplate ) },
			{ "DoorOpenTime",					INI::parseDurationUnsignedInt,	nullptr, offsetof( MissileLauncherBuildingUpdateModuleData, m_doorOpenTime ) },
			{ "DoorWaitOpenTime",			INI::parseDurationUnsignedInt,	nullptr, offsetof( MissileLauncherBuildingUpdateModuleData, m_doorWaitOpenTime ) },
			{ "DoorCloseTime",				INI::parseDurationUnsignedInt,	nullptr, offsetof( MissileLauncherBuildingUpdateModuleData, m_doorClosingTime ) },
			{ "DoorOpeningFX",				INI::parseFXList,								nullptr, offsetof( MissileLauncherBuildingUpdateModuleData, m_openingFX ) },
			{ "DoorOpenFX",						INI::parseFXList,								nullptr, offsetof( MissileLauncherBuildingUpdateModuleData, m_openFX ) },
			{ "DoorWaitingToCloseFX",	INI::parseFXList,								nullptr, offsetof( MissileLauncherBuildingUpdateModuleData, m_waitingToCloseFX ) },
			{ "DoorClosingFX",				INI::parseFXList,								nullptr, offsetof( MissileLauncherBuildingUpdateModuleData, m_closingFX ) },
			{ "DoorClosedFX",					INI::parseFXList,								nullptr, offsetof( MissileLauncherBuildingUpdateModuleData, m_closedFX ) },
			{ "DoorOpenIdleAudio",		INI::parseAudioEventRTS,				nullptr,	offsetof( MissileLauncherBuildingUpdateModuleData, m_openIdleAudio ) },
			{ nullptr, nullptr, nullptr, 0 }
		};
    p.add(dataFieldParse);
	}
};

//-------------------------------------------------------------------------------------------------
class MissileLauncherBuildingUpdate : public SpecialPowerUpdateModule
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( MissileLauncherBuildingUpdate, "MissileLauncherBuildingUpdate" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( MissileLauncherBuildingUpdate, MissileLauncherBuildingUpdateModuleData )

public:

	MissileLauncherBuildingUpdate( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	//SpecialPowerUpdateInterface pure virtual implementations
	virtual Bool initiateIntentToDoSpecialPower(const SpecialPowerTemplate *specialPowerTemplate, const Object *targetObj, const Coord3D *targetPos, const Waypoint *way, UnsignedInt commandOptions ) override;
	virtual Bool isSpecialAbility() const override { return false; }
	virtual Bool isSpecialPower() const override { return true; }
	virtual Bool isActive() const override { return m_doorState != m_timeoutState; }
	SpecialPowerTemplate* getTemplate() const;
	virtual Bool doesSpecialPowerHaveOverridableDestinationActive() const override { return false; } //Is it active now?
	virtual Bool doesSpecialPowerHaveOverridableDestination() const override { return false; }	//Does it have it, even if it's not active?
	virtual void setSpecialPowerOverridableDestination( const Coord3D *loc ) override {}

	virtual SpecialPowerUpdateInterface* getSpecialPowerUpdateInterface() override { return this; }
	virtual CommandOption getCommandOption() const override { return (CommandOption)0; }

	virtual UpdateSleepTime update() override;	///< Deciding whether or not to make new guys
	virtual Bool isPowerCurrentlyInUse( const CommandButton *command = nullptr ) const override;

private:
	enum DoorStateType
	{
		DOOR_CLOSED,
		DOOR_OPENING,
		DOOR_OPEN,
		DOOR_WAITING_TO_CLOSE,
		DOOR_CLOSING,
	};

	void switchToState(DoorStateType dst);

	const SpecialPowerModuleInterface*	m_specialPowerModule;
	DoorStateType												m_doorState;
	DoorStateType												m_timeoutState;
	UnsignedInt													m_timeoutFrame;
	AudioEventRTS												m_openIdleAudio;
};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/MissileLauncherBuildingUpdate.cpp`
```cpp
// FILE: MissileLauncherBuildingUpdate.cpp /////////////////////////////////////////////////////////////////////////
// Author: Matthew D. Campbell, April 2002
// Desc:   Update will change model state conditions based on special power state
///////////////////////////////////////////////////////////////////////////////////////////////////

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/GameAudio.h"
#include "Common/GlobalData.h"
#include "Common/Player.h"
#include "Common/SpecialPower.h"
#include "Common/Xfer.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/MissileLauncherBuildingUpdate.h"
#include "GameLogic/Module/SpecialPowerModule.h"
#include "GameClient/Drawable.h"
#include "GameClient/FXList.h"


//-------------------------------------------------------------------------------------------------
MissileLauncherBuildingUpdate::MissileLauncherBuildingUpdate( Thing *thing, const ModuleData* moduleData ) : SpecialPowerUpdateModule( thing, moduleData )
{
	m_doorState = DOOR_CLOSED;
	m_timeoutState = DOOR_CLOSED;
	m_timeoutFrame = 0;
	m_openIdleAudio = getMissileLauncherBuildingUpdateModuleData()->m_openIdleAudio;
	m_openIdleAudio.setObjectID(getObject()->getID());
}

//-------------------------------------------------------------------------------------------------
MissileLauncherBuildingUpdate::~MissileLauncherBuildingUpdate()
{
	if (m_openIdleAudio.isCurrentlyPlaying())
	{
		TheAudio->removeAudioEvent(m_openIdleAudio.getPlayingHandle());
	}
}

//-------------------------------------------------------------------------------------------------
void MissileLauncherBuildingUpdate::switchToState(DoorStateType dst)
{
	if (m_doorState == dst)
		return;

	const MissileLauncherBuildingUpdateModuleData* d = getMissileLauncherBuildingUpdateModuleData();
	ModelConditionFlags clr, set;
	UnsignedInt now = TheGameLogic->getFrame();
	switch (dst)
	{
		case DOOR_CLOSED:
			DEBUG_LOG(("switch to state DOOR_CLOSED at %d",now));
			/// @todo srj -- for now, this assumes at most one door
			clr.set(MODELCONDITION_DOOR_1_WAITING_TO_CLOSE);
			clr.set(MODELCONDITION_DOOR_1_CLOSING);
			clr.set(MODELCONDITION_DOOR_1_OPENING);
			clr.set(MODELCONDITION_DOOR_1_WAITING_OPEN);
			m_timeoutFrame = 0;
			m_timeoutState = DOOR_CLOSED;
			if (d->m_closedFX)
			{
				const Coord3D *pos = getObject()->getPosition();
				FXList::doFXPos(d->m_closedFX, pos);
			}
			if (m_openIdleAudio.isCurrentlyPlaying())
			{
				TheAudio->removeAudioEvent(m_openIdleAudio.getPlayingHandle());
			}
			break;

		case DOOR_OPENING:
			DEBUG_LOG(("switch to state DOOR_OPENING at %d",now));
			/// @todo srj -- for now, this assumes at most one door
			clr.set(MODELCONDITION_DOOR_1_WAITING_TO_CLOSE);
			clr.set(MODELCONDITION_DOOR_1_CLOSING);
			clr.set(MODELCONDITION_DOOR_1_WAITING_OPEN);
			set.set(MODELCONDITION_DOOR_1_OPENING);
			// we want this to be done BEFORE the power is ready, so
			// end it one frame ahead.
			m_timeoutFrame = m_specialPowerModule->getReadyFrame() - 1;
			m_timeoutState = DOOR_OPEN;
			if (d->m_openingFX)
			{
				const Coord3D *pos = getObject()->getPosition();
				FXList::doFXPos(d->m_openingFX, pos);
			}
			if (m_openIdleAudio.isCurrentlyPlaying())
			{
				TheAudio->removeAudioEvent(m_openIdleAudio.getPlayingHandle());
			}
			break;

		case DOOR_OPEN:
			DEBUG_LOG(("switch to state DOOR_OPEN at %d",now));
			/// @todo srj -- for now, this assumes at most one door
			clr.set(MODELCONDITION_DOOR_1_WAITING_TO_CLOSE);
			clr.set(MODELCONDITION_DOOR_1_CLOSING);
			clr.set(MODELCONDITION_DOOR_1_OPENING);
			set.set(MODELCONDITION_DOOR_1_WAITING_OPEN);
			m_timeoutFrame = 0;
			m_timeoutState = DOOR_OPEN;
			if (d->m_openFX)
			{
				const Coord3D *pos = getObject()->getPosition();
				FXList::doFXPos(d->m_openFX, pos);
			}
			if (!m_openIdleAudio.isCurrentlyPlaying())
			{
				m_openIdleAudio.setPlayingHandle(TheAudio->addAudioEvent(&m_openIdleAudio));
			}
			break;

		case DOOR_WAITING_TO_CLOSE:
			DEBUG_LOG(("switch to state DOOR_WAITING_TO_CLOSE at %d",now));
			/// @todo srj -- for now, this assumes at most one door
			clr.set(MODELCONDITION_DOOR_1_CLOSING);
			clr.set(MODELCONDITION_DOOR_1_OPENING);
			clr.set(MODELCONDITION_DOOR_1_WAITING_OPEN);
			set.set(MODELCONDITION_DOOR_1_WAITING_TO_CLOSE);
			m_timeoutFrame = now + d->m_doorWaitOpenTime;
			m_timeoutState = DOOR_CLOSING;
			if (d->m_waitingToCloseFX)
			{
				const Coord3D *pos = getObject()->getPosition();
				FXList::doFXPos(d->m_waitingToCloseFX, pos);
			}
			if (m_openIdleAudio.isCurrentlyPlaying())
			{
				TheAudio->removeAudioEvent(m_openIdleAudio.getPlayingHandle());
			}
			break;

		case DOOR_CLOSING:
			DEBUG_LOG(("switch to state DOOR_CLOSING at %d",now));
			/// @todo srj -- for now, this assumes at most one door
			clr.set(MODELCONDITION_DOOR_1_WAITING_TO_CLOSE);
			clr.set(MODELCONDITION_DOOR_1_WAITING_OPEN);
			clr.set(MODELCONDITION_DOOR_1_OPENING);
			set.set(MODELCONDITION_DOOR_1_CLOSING);
			m_timeoutFrame = now + d->m_doorClosingTime;

			{
				Int delta = m_specialPowerModule->getReadyFrame() - now;
				if (m_timeoutFrame > now + delta/2)
					m_timeoutFrame = now + delta/2;
			}

			m_timeoutState = DOOR_CLOSED;
			if (d->m_closingFX)
			{
				const Coord3D *pos = getObject()->getPosition();
				FXList::doFXPos(d->m_closingFX, pos);
			}
			if (m_openIdleAudio.isCurrentlyPlaying())
			{
				TheAudio->removeAudioEvent(m_openIdleAudio.getPlayingHandle());
			}
			break;
#ifdef RTS_DEBUG
		default:
			DEBUG_CRASH(("unknown state"));
			break;
#endif
	}
	m_doorState = dst;
	getObject()->clearAndSetModelConditionFlags(clr, set);

	if (m_timeoutFrame > 0)
	{
		Drawable* draw = getObject()->getDrawable();
		if (draw)
			draw->setAnimationLoopDuration(m_timeoutFrame - now);
	}
}

//-------------------------------------------------------------------------------------------------
Bool MissileLauncherBuildingUpdate::initiateIntentToDoSpecialPower( const SpecialPowerTemplate *specialPowerTemplate, const Object *targetObj, const Coord3D *targetPos, const Waypoint *way, UnsignedInt commandOptions )
{
	const MissileLauncherBuildingUpdateModuleData* data = getMissileLauncherBuildingUpdateModuleData();
	if (data->m_specialPowerTemplate != specialPowerTemplate)
		return FALSE;

	// GeneralsX @bugfix Copilot 06/09/2026 Abort launch requests made before the special-power module is ready.
	// Upstream reference: Mauller, PR #1218
	// https://github.com/TheSuperHackers/GeneralsGameCode/pull/1218
	Object* us = getObject();
	if (us->testStatus(OBJECT_STATUS_UNDER_CONSTRUCTION)) {
		SpecialPowerModuleInterface* specialPowerModule = m_specialPowerModule
			? const_cast<SpecialPowerModuleInterface*>(m_specialPowerModule)
			: us->getSpecialPowerModule(data->m_specialPowerTemplate);
		DEBUG_ASSERTCRASH(specialPowerModule, ("Missing special power"));
		if (specialPowerModule)
			specialPowerModule->setReadyFrame(0xFFFFFFFF);
		return FALSE;
	}

	if (!m_specialPowerModule) {
		// The cached pointer is intentionally not serialized. Restore it if a loaded, completed
		// launcher receives a scripted command before its first update.
		m_specialPowerModule = us->getSpecialPowerModule(data->m_specialPowerTemplate);
		if (!m_specialPowerModule)
			return FALSE;
	}

	if( m_specialPowerModule->getSpecialPowerTemplate() != specialPowerTemplate )
	{
		return FALSE;
	}

#if defined(RTS_DEBUG) || defined(_ALLOW_DEBUG_CHEATS_IN_RELEASE)
	DEBUG_ASSERTCRASH(!TheGlobalData->m_specialPowerUsesDelay || m_doorState == DOOR_OPEN, ("door is not fully open when specialpower is fired!"));
#endif

	switchToState(DOOR_WAITING_TO_CLOSE);
//	getObject()->getControllingPlayer()->getAcademyStats()->recordSpecialPowerUsed( specialPowerTemplate );
	return TRUE;
}

//-------------------------------------------------------------------------------------------------
Bool MissileLauncherBuildingUpdate::isPowerCurrentlyInUse( const CommandButton *command ) const
{
	//@todo -- Implement me in such a way that it still works with Ctrl+s cheat key...
	//In fact, this code doesn't really care because in legit mode, the game will
	//never require this check to disable the cameo button.
	return false;
}

//-------------------------------------------------------------------------------------------------
UpdateSleepTime MissileLauncherBuildingUpdate::update()
{
	const MissileLauncherBuildingUpdateModuleData* d = getMissileLauncherBuildingUpdateModuleData();

	UnsignedInt now = TheGameLogic->getFrame();

	// If we are under construction, any decision we make about door status could be wrong.
	// Our special power module is randomly going to be initialized or not (which would result
	// in him reporting a 0 frame ready, which means we will start open).
	if( getObject()->testStatus(OBJECT_STATUS_UNDER_CONSTRUCTION) )
		return UPDATE_SLEEP_NONE;

	if (!m_specialPowerModule)
	{
		m_specialPowerModule = getObject()->getSpecialPowerModule(d->m_specialPowerTemplate);
		DEBUG_ASSERTCRASH(m_specialPowerModule, ("Missing special power"));
	}

	if (m_specialPowerModule)
	{
		UnsignedInt readyFrame = m_specialPowerModule->getReadyFrame();
		UnsignedInt whenToStartOpening = (readyFrame >= d->m_doorOpenTime) ? (readyFrame - d->m_doorOpenTime) : 0;

		if (m_timeoutFrame != 0 && now > m_timeoutFrame)
		{
			switchToState(m_timeoutState);
		}

#if defined(RTS_DEBUG) || defined(_ALLOW_DEBUG_CHEATS_IN_RELEASE)
		DEBUG_ASSERTCRASH(!TheGlobalData->m_specialPowerUsesDelay || !(m_specialPowerModule->isReady() && m_doorState != DOOR_OPEN), ("door is not fully open when specialpower is ready!"));
#endif

		if (m_doorState != DOOR_OPEN && m_specialPowerModule->isReady())
		{
			DEBUG_LOG(("*** had to POP the door open!"));
			switchToState(DOOR_OPEN);
		}
		else if (m_doorState == DOOR_CLOSED && now >= whenToStartOpening)
		{
			switchToState(DOOR_OPENING);
		}
	}
	return UPDATE_SLEEP_NONE;
}

//-------------------------------------------------------------------------------------------------
SpecialPowerTemplate* MissileLauncherBuildingUpdate::getTemplate() const
{
	const MissileLauncherBuildingUpdateModuleData* data = getMissileLauncherBuildingUpdateModuleData();
	return data->m_specialPowerTemplate;
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void MissileLauncherBuildingUpdate::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void MissileLauncherBuildingUpdate::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// do not need to tie the m_specialPowerModule pointer cause it gets tied
	// SpecialPowerModuleInterface *m_specialPowerModule;

	// door state
	xfer->xferUser( &m_doorState, sizeof( DoorStateType ) );

	// timeout state
	xfer->xferUser( &m_timeoutState, sizeof( DoorStateType ) );

	// timeout frame
	xfer->xferUnsignedInt( &m_timeoutFrame );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void MissileLauncherBuildingUpdate::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
