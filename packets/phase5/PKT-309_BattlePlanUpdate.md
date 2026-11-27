# Translation Packet `PKT-309`: `BattlePlanUpdate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-309`
- **Work Package:** `WP-5.4` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/update/battle_plan_update.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (29):**
  - `Common/BitFlagsIO.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/BitFlagsIO.h`
  - `Common/KindOf.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/KindOf.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/PlayerList.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/PlayerList.h`
  - `Common/Radar.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Radar.h`
  - `Common/ThingFactory.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ThingFactory.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/ControlBar.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/ControlBar.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/FXList.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/FXList.h`
  - `GameClient/GameClient.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/GameClient.h`
  - `GameClient/GameText.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/GameText.h`
  - `GameClient/ParticleSys.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/ParticleSys.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/ActiveBody.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ActiveBody.h`
  - `GameLogic/Module/BattlePlanUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BattlePlanUpdate.h`
  - `GameLogic/Module/PhysicsUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PhysicsUpdate.h`
  - `GameLogic/Module/SpecialPowerModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpecialPowerModule.h`
  - `GameLogic/Module/SpecialPowerUpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpecialPowerUpdateModule.h`
  - `GameLogic/Module/StealthDetectorUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/StealthDetectorUpdate.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/ObjectIter.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ObjectIter.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `GameLogic/TerrainLogic.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/TerrainLogic.h`
  - `GameLogic/Weapon.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h`
  - `GameLogic/WeaponSet.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/WeaponSet.h`
  - `PreRTS.h`
- **Implemented Module Facets (1):**
  - `SpecialPowerUpdateInterface* getSpecialPowerUpdateInterface()`
- **INI `FieldParse` Table Entries (28):**
  - `"SpecialPowerTemplate" => INI::parseSpecialPowerTemplate`
  - `"BombardmentPlanAnimationTime" => INI::parseDurationUnsignedInt`
  - `"HoldTheLinePlanAnimationTime" => INI::parseDurationUnsignedInt`
  - `"SearchAndDestroyPlanAnimationTime" => INI::parseDurationUnsignedInt`
  - `"TransitionIdleTime" => INI::parseDurationUnsignedInt`
  - `"BombardmentPlanUnpackSoundName" => INI::parseAsciiString`
  - `"BombardmentPlanPackSoundName" => INI::parseAsciiString`
  - `"BombardmentMessageLabel" => INI::parseAsciiString`
  - `"BombardmentAnnouncementName" => INI::parseAsciiString`
  - `"SearchAndDestroyPlanUnpackSoundName" => INI::parseAsciiString`
  - `"SearchAndDestroyPlanIdleLoopSoundName" => INI::parseAsciiString`
  - `"SearchAndDestroyPlanPackSoundName" => INI::parseAsciiString`
  - `"SearchAndDestroyMessageLabel" => INI::parseAsciiString`
  - `"SearchAndDestroyAnnouncementName" => INI::parseAsciiString`
  - `"HoldTheLinePlanUnpackSoundName" => INI::parseAsciiString`
  - `"HoldTheLinePlanPackSoundName" => INI::parseAsciiString`
  - `"HoldTheLineMessageLabel" => INI::parseAsciiString`
  - `"HoldTheLineAnnouncementName" => INI::parseAsciiString`
  - `"ValidMemberKindOf" => KindOfMaskType::parseFromINI`
  - `"InvalidMemberKindOf" => KindOfMaskType::parseFromINI`
  - `"BattlePlanChangeParalyzeTime" => INI::parseDurationUnsignedInt`
  - `"HoldTheLinePlanArmorDamageScalar" => INI::parseReal`
  - `"SearchAndDestroyPlanSightRangeScalar" => INI::parseReal`
  - `"StrategyCenterSearchAndDestroySightRangeScalar" => INI::parseReal`
  - `"StrategyCenterSearchAndDestroyDetectsStealth" => INI::parseBool`
  - `"StrategyCenterHoldTheLineMaxHealthScalar" => INI::parseReal`
  - `"StrategyCenterHoldTheLineMaxHealthChangeType" => INI::parseIndexList`
  - `"VisionObjectName" => INI::parseAsciiString`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (14 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUser( &m_currentPlan, sizeof( BattlePlanStatus ) )`
  3. `xferUser( &m_desiredPlan, sizeof( BattlePlanStatus ) )`
  4. `xferUser( &m_planAffectingArmy, sizeof( BattlePlanStatus ) )`
  5. `xferUser( &m_status, sizeof( TransitionStatus ) )`
  6. `xferUnsignedInt( &m_nextReadyFrame )`
  7. `xferBool( &m_invalidSettings )`
  8. `xferBool( &m_centeringTurret )`
  9. `xferReal( &m_bonuses->m_armorScalar )`
  10. `xferInt( &m_bonuses->m_bombardment )`
  11. `xferInt( &m_bonuses->m_searchAndDestroy )`
  12. `xferInt( &m_bonuses->m_holdTheLine )`
  13. `xferReal( &m_bonuses->m_sightRangeScalar )`
  14. `xferObjectID( &m_visionObjectID )`
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if !RETAIL_COMPATIBLE_CRC`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BattlePlanUpdate.h`
```cpp
// FILE: BattlePlanUpdate.h //////////////////////////////////////////////////////////////////////////
// Author: Kris Morness, September 2002
// Desc:   Update module to handle building states and battle plan execution & changes
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/KindOf.h"
#include "GameLogic/Module/SpecialPowerUpdateModule.h"

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
class SpecialPowerModule;
class ParticleSystem;
class FXList;
class AudioEventRTS;
enum  MaxHealthChangeType CPP_11(: Int);
enum  CommandOption CPP_11(: Int);

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class BattlePlanUpdateModuleData : public ModuleData
{
public:
	SpecialPowerTemplate *m_specialPowerTemplate;

  UnsignedInt m_bombardmentPlanAnimationFrames;
  UnsignedInt m_holdTheLinePlanAnimationFrames;
  UnsignedInt m_searchAndDestroyPlanAnimationFrames;
	UnsignedInt m_transitionIdleFrames;

	AsciiString	m_bombardmentUnpackName;
	AsciiString	m_bombardmentPackName;
	AsciiString m_bombardmentMessageLabel;
	AsciiString m_bombardmentAnnouncementName;
	AsciiString	m_searchAndDestroyUnpackName;
	AsciiString m_searchAndDestroyIdleName;
	AsciiString	m_searchAndDestroyPackName;
	AsciiString m_searchAndDestroyMessageLabel;
	AsciiString m_searchAndDestroyAnnouncementName;
	AsciiString m_holdTheLineUnpackName;
	AsciiString m_holdTheLinePackName;
	AsciiString m_holdTheLineMessageLabel;
	AsciiString m_holdTheLineAnnouncementName;

	UnsignedInt m_battlePlanParalyzeFrames;
	KindOfMaskType m_validMemberKindOf;
	KindOfMaskType m_invalidMemberKindOf;

	Real m_holdTheLineArmorDamageScalar;
	Real m_searchAndDestroySightRangeScalar;
	Real m_strategyCenterSearchAndDestroySightRangeScalar;
	Bool m_strategyCenterSearchAndDestroyDetectsStealth;
	Real m_strategyCenterHoldTheLineMaxHealthScalar;
	MaxHealthChangeType m_strategyCenterHoldTheLineMaxHealthChangeType;

	AsciiString m_visionObjectName;		///< name of object to create to reveal shroud to all players

	BattlePlanUpdateModuleData();
	static void buildFieldParse(MultiIniFieldParse& p);

private:

};

enum TransitionStatus CPP_11(: Int)
{
	TRANSITIONSTATUS_IDLE,
	TRANSITIONSTATUS_UNPACKING,
	TRANSITIONSTATUS_ACTIVE,
	TRANSITIONSTATUS_PACKING,
};

enum BattlePlanStatus CPP_11(: Int)
{
	PLANSTATUS_NONE,
	PLANSTATUS_BOMBARDMENT,
	PLANSTATUS_HOLDTHELINE,
	PLANSTATUS_SEARCHANDDESTROY,
};

struct BattlePlanBonusesData
{
	BattlePlanBonusesData() //Default the bonuses to no change.
		: m_armorScalar(1.0f)
		, m_bombardment(0)
		, m_searchAndDestroy(0)
		, m_holdTheLine(0)
		, m_sightRangeScalar(1.0f)
	{
	}

	Real						m_armorScalar;
	Int							m_bombardment;				//Represents having weapon bonuses for bombardment plan
	Int							m_searchAndDestroy;		//Represents having weapon bonuses for searchAndDestroy plan
	Int							m_holdTheLine;				//Represents having weapon bonuses for holdTheLine plan
	Real						m_sightRangeScalar;
	KindOfMaskType	m_validKindOf;
	KindOfMaskType	m_invalidKindOf;
};

class BattlePlanBonuses : public BattlePlanBonusesData, public MemoryPoolObject
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(BattlePlanBonuses, "BattlePlanBonuses")
};
EMPTY_DTOR(BattlePlanBonuses)

#define ALL_PLANS	1000000		//Used when stacking or removing plans -- we only remove the bonuses when it's 0 or negative.

//-------------------------------------------------------------------------------------------------
/** The default	update module */
//-------------------------------------------------------------------------------------------------
class BattlePlanUpdate : public SpecialPowerUpdateModule
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( BattlePlanUpdate, "BattlePlanUpdate" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( BattlePlanUpdate, BattlePlanUpdateModuleData );

public:

	BattlePlanUpdate( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	// SpecialPowerUpdateInterface
	virtual Bool initiateIntentToDoSpecialPower(const SpecialPowerTemplate *specialPowerTemplate, const Object *targetObj, const Coord3D *targetPos, const Waypoint *way, UnsignedInt commandOptions ) override;
	virtual Bool isSpecialAbility() const override { return false; }
	virtual Bool isSpecialPower() const override { return true; }
	virtual Bool isActive() const override {return m_status != TRANSITIONSTATUS_IDLE;}
	virtual SpecialPowerUpdateInterface* getSpecialPowerUpdateInterface() override { return this; }
	virtual Bool doesSpecialPowerHaveOverridableDestinationActive() const override { return false; } //Is it active now?
	virtual Bool doesSpecialPowerHaveOverridableDestination() const override { return false; }	//Does it have it, even if it's not active?
	virtual void setSpecialPowerOverridableDestination( const Coord3D *loc ) override {}
	virtual Bool isPowerCurrentlyInUse( const CommandButton *command = nullptr ) const override;

	//Returns the currently active battle plan -- unpacked and ready... returns PLANSTATUS_NONE if in transition!
	BattlePlanStatus getActiveBattlePlan() const;

	virtual void onObjectCreated() override;
	virtual void onDelete() override;
	virtual UpdateSleepTime update() override;
	virtual void onCapture(Player* oldOwner, Player* newOwner) override;

	virtual CommandOption getCommandOption() const override;
protected:

	void setStatus( TransitionStatus status );
	void enableTurret( Bool enable );
	void recenterTurret();
	Bool isTurretInNaturalPosition();
	void setBattlePlan( BattlePlanStatus plan );
	void createVisionObject();

	BattlePlanStatus m_currentPlan;	//The current battle plan displayed by the building (includes packing & unpacking)
	BattlePlanStatus m_desiredPlan; //The user desired battle plan
	BattlePlanStatus m_planAffectingArmy; //The current battle plan that is affecting troops!
	TransitionStatus m_status;

	UnsignedInt m_nextReadyFrame;
	SpecialPowerModuleInterface *m_specialPowerModule;
	Bool				m_invalidSettings;
	Bool				m_centeringTurret;
	BattlePlanBonuses* m_bonuses;

	AudioEventRTS		m_bombardmentUnpack;
	AudioEventRTS		m_bombardmentPack;
	AudioEventRTS   m_bombardmentAnnouncement;
	AudioEventRTS		m_searchAndDestroyUnpack;
	AudioEventRTS   m_searchAndDestroyIdle;
	AudioEventRTS		m_searchAndDestroyPack;
	AudioEventRTS   m_searchAndDestroyAnnouncement;
	AudioEventRTS   m_holdTheLineUnpack;
	AudioEventRTS   m_holdTheLinePack;
	AudioEventRTS   m_holdTheLineAnnouncement;

	// vision object - hang on to this so we can delete it on destruction
	ObjectID m_visionObjectID;

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/BattlePlanUpdate.cpp`
```cpp
// FILE: BattlePlanUpdate.cpp //////////////////////////////////////////////////////////////////////////
// Author: Kris Morness, September 2002
// Desc:   Update module to handle building states and battle plan execution & changes
///////////////////////////////////////////////////////////////////////////////////////////////////

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#define DEFINE_MAXHEALTHCHANGETYPE_NAMES						// for TheMaxHealthChangeTypeNames[]

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/BitFlagsIO.h"
#include "Common/Radar.h"
#include "Common/PlayerList.h"
#include "Common/ThingTemplate.h"
#include "Common/ThingFactory.h"
#include "Common/Player.h"
#include "Common/Xfer.h"

#include "GameClient/GameClient.h"
#include "GameClient/Drawable.h"
#include "GameClient/GameText.h"
#include "GameClient/ParticleSys.h"
#include "GameClient/FXList.h"
#include "GameClient/ControlBar.h"

#include "GameLogic/GameLogic.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectIter.h"
#include "GameLogic/WeaponSet.h"
#include "GameLogic/Weapon.h"
#include "GameLogic/TerrainLogic.h"
#include "GameLogic/Module/SpecialPowerModule.h"
#include "GameLogic/Module/BattlePlanUpdate.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Module/ActiveBody.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/StealthDetectorUpdate.h"


//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
BattlePlanUpdateModuleData::BattlePlanUpdateModuleData()
{
	m_specialPowerTemplate								= nullptr;
	m_bombardmentPlanAnimationFrames			= 0;
	m_holdTheLinePlanAnimationFrames			= 0;
	m_searchAndDestroyPlanAnimationFrames = 0;
	m_battlePlanParalyzeFrames						= 0;


	m_holdTheLineArmorDamageScalar				= 1.0f;
	m_searchAndDestroySightRangeScalar		= 1.0f;
	m_strategyCenterSearchAndDestroySightRangeScalar = 1.0f;
	m_strategyCenterSearchAndDestroyDetectsStealth = true;
	m_strategyCenterHoldTheLineMaxHealthScalar = 1.0f;
	m_strategyCenterHoldTheLineMaxHealthChangeType = PRESERVE_RATIO;

}

//-------------------------------------------------------------------------------------------------
/*static*/ void BattlePlanUpdateModuleData::buildFieldParse(MultiIniFieldParse& p)
{
	ModuleData::buildFieldParse(p);

	static const FieldParse dataFieldParse[] =
	{
		{ "SpecialPowerTemplate",									INI::parseSpecialPowerTemplate,	nullptr, offsetof( BattlePlanUpdateModuleData, m_specialPowerTemplate ) },

    { "BombardmentPlanAnimationTime",					INI::parseDurationUnsignedInt,  nullptr, offsetof( BattlePlanUpdateModuleData, m_bombardmentPlanAnimationFrames ) },
    { "HoldTheLinePlanAnimationTime",					INI::parseDurationUnsignedInt,  nullptr, offsetof( BattlePlanUpdateModuleData, m_holdTheLinePlanAnimationFrames ) },
    { "SearchAndDestroyPlanAnimationTime",		INI::parseDurationUnsignedInt,  nullptr, offsetof( BattlePlanUpdateModuleData, m_searchAndDestroyPlanAnimationFrames ) },
		{ "TransitionIdleTime",										INI::parseDurationUnsignedInt,  nullptr, offsetof( BattlePlanUpdateModuleData, m_transitionIdleFrames ) },

		{ "BombardmentPlanUnpackSoundName",				INI::parseAsciiString,					nullptr, offsetof( BattlePlanUpdateModuleData, m_bombardmentUnpackName ) },
		{ "BombardmentPlanPackSoundName",					INI::parseAsciiString,					nullptr, offsetof( BattlePlanUpdateModuleData, m_bombardmentPackName ) },
		{ "BombardmentMessageLabel",							INI::parseAsciiString,					nullptr,	offsetof( BattlePlanUpdateModuleData, m_bombardmentMessageLabel ) },
		{ "BombardmentAnnouncementName",					INI::parseAsciiString,					nullptr, offsetof( BattlePlanUpdateModuleData, m_bombardmentAnnouncementName ) },
		{ "SearchAndDestroyPlanUnpackSoundName",	INI::parseAsciiString,					nullptr, offsetof( BattlePlanUpdateModuleData, m_searchAndDestroyUnpackName ) },
		{ "SearchAndDestroyPlanIdleLoopSoundName",INI::parseAsciiString,					nullptr, offsetof( BattlePlanUpdateModuleData, m_searchAndDestroyIdleName ) },
		{ "SearchAndDestroyPlanPackSoundName",		INI::parseAsciiString,					nullptr, offsetof( BattlePlanUpdateModuleData, m_searchAndDestroyPackName ) },
		{ "SearchAndDestroyMessageLabel",					INI::parseAsciiString,					nullptr,	offsetof( BattlePlanUpdateModuleData, m_searchAndDestroyMessageLabel ) },
		{ "SearchAndDestroyAnnouncementName",			INI::parseAsciiString,					nullptr, offsetof( BattlePlanUpdateModuleData, m_searchAndDestroyAnnouncementName ) },
		{ "HoldTheLinePlanUnpackSoundName",				INI::parseAsciiString,					nullptr, offsetof( BattlePlanUpdateModuleData, m_holdTheLineUnpackName ) },
		{ "HoldTheLinePlanPackSoundName",					INI::parseAsciiString,					nullptr, offsetof( BattlePlanUpdateModuleData, m_holdTheLinePackName ) },
		{ "HoldTheLineMessageLabel",							INI::parseAsciiString,					nullptr,	offsetof( BattlePlanUpdateModuleData, m_holdTheLineMessageLabel ) },
		{ "HoldTheLineAnnouncementName",					INI::parseAsciiString,					nullptr, offsetof( BattlePlanUpdateModuleData, m_holdTheLineAnnouncementName ) },

		{ "ValidMemberKindOf",										KindOfMaskType::parseFromINI,								nullptr, offsetof( BattlePlanUpdateModuleData, m_validMemberKindOf ) },
		{ "InvalidMemberKindOf",									KindOfMaskType::parseFromINI,								nullptr, offsetof( BattlePlanUpdateModuleData, m_invalidMemberKindOf ) },
		{ "BattlePlanChangeParalyzeTime",					INI::parseDurationUnsignedInt,  nullptr, offsetof( BattlePlanUpdateModuleData, m_battlePlanParalyzeFrames ) },
		{ "HoldTheLinePlanArmorDamageScalar",			INI::parseReal,									nullptr, offsetof( BattlePlanUpdateModuleData, m_holdTheLineArmorDamageScalar ) },
		{ "SearchAndDestroyPlanSightRangeScalar",	INI::parseReal,									nullptr, offsetof( BattlePlanUpdateModuleData, m_searchAndDestroySightRangeScalar ) },

		{ "StrategyCenterSearchAndDestroySightRangeScalar", INI::parseReal,				nullptr, offsetof( BattlePlanUpdateModuleData, m_strategyCenterSearchAndDestroySightRangeScalar ) },
		{ "StrategyCenterSearchAndDestroyDetectsStealth",   INI::parseBool,				nullptr, offsetof( BattlePlanUpdateModuleData, m_strategyCenterSearchAndDestroyDetectsStealth ) },
		{ "StrategyCenterHoldTheLineMaxHealthScalar",				INI::parseReal,				nullptr, offsetof( BattlePlanUpdateModuleData, m_strategyCenterHoldTheLineMaxHealthScalar ) },
    { "StrategyCenterHoldTheLineMaxHealthChangeType",		INI::parseIndexList,  TheMaxHealthChangeTypeNames, offsetof( BattlePlanUpdateModuleData, m_strategyCenterHoldTheLineMaxHealthChangeType ) },

		{ "VisionObjectName",											INI::parseAsciiString,					nullptr, offsetof( BattlePlanUpdateModuleData, m_visionObjectName ) },

		{ nullptr, nullptr, nullptr, 0 }
	};
	p.add(dataFieldParse);
}

//-------------------------------------------------------------------------------------------------
BattlePlanUpdate::BattlePlanUpdate( Thing *thing, const ModuleData* moduleData ) :
	SpecialPowerUpdateModule( thing, moduleData ),
	m_bonuses(nullptr)
{
	const BattlePlanUpdateModuleData *data = getBattlePlanUpdateModuleData();

	m_status								= TRANSITIONSTATUS_IDLE;
	m_currentPlan						= PLANSTATUS_NONE;
	m_desiredPlan						= PLANSTATUS_NONE;
	m_planAffectingArmy			= PLANSTATUS_NONE;
	m_nextReadyFrame				= 0;
	m_invalidSettings				= false;
	m_centeringTurret				= false;

	m_bonuses = newInstance(BattlePlanBonuses);
	m_bonuses->m_validKindOf = data->m_validMemberKindOf;
	m_bonuses->m_invalidKindOf = data->m_invalidMemberKindOf;

	m_visionObjectID = INVALID_ID;

	m_specialPowerModule   = nullptr;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
BattlePlanUpdate::~BattlePlanUpdate()
{
	TheAudio->removeAudioEvent( m_bombardmentUnpack.getPlayingHandle() );
	TheAudio->removeAudioEvent( m_bombardmentPack.getPlayingHandle() );
	TheAudio->removeAudioEvent( m_searchAndDestroyUnpack.getPlayingHandle() );
	TheAudio->removeAudioEvent( m_searchAndDestroyIdle.getPlayingHandle() );
	TheAudio->removeAudioEvent( m_searchAndDestroyPack.getPlayingHandle() );
	TheAudio->removeAudioEvent( m_holdTheLineUnpack.getPlayingHandle() );
	TheAudio->removeAudioEvent( m_holdTheLinePack.getPlayingHandle() );

	deleteInstance(m_bonuses);
}

// ------------------------------------------------------------------------------------------------
/** On delete */
// ------------------------------------------------------------------------------------------------
void BattlePlanUpdate::onDelete()
{

	// extend base class
	UpdateModule::onDelete();

	// delete our vision object, if it exists
	Object *obj;
	if( m_visionObjectID != INVALID_ID )
	{
		obj = TheGameLogic->findObjectByID( m_visionObjectID );
		if( obj )
			TheGameLogic->destroyObject( obj );
	}

	// If we get destroyed, then make sure we remove our bonus!
	// srj sez: we can't do this in the dtor because our team
	// (and thus controlling player) has already been nulled by then...
	Player* player = getObject()->getControllingPlayer();
	// however, player CAN legitimately be null during game reset cycles
	// (and which point it doesn't really matter if we can remove the bonus or not)
	//DEBUG_ASSERTCRASH(player != nullptr, ("Hmm, controller is null"));
	if( player && m_planAffectingArmy != PLANSTATUS_NONE )
	{
		player->changeBattlePlan( m_planAffectingArmy, -1, m_bonuses );
	}

}

//-------------------------------------------------------------------------------------------------
// Validate that we have the necessary data from the ini file.
//-------------------------------------------------------------------------------------------------
void BattlePlanUpdate::onObjectCreated()
{
	const BattlePlanUpdateModuleData *data = getBattlePlanUpdateModuleData();
	Object *obj = getObject();

	if( !data->m_specialPowerTemplate )
	{
		DEBUG_CRASH( ("%s object's BattlePlanUpdate lacks access to the SpecialPowerTemplate. Needs to be specified in ini.", obj->getTemplate()->getName().str() ) );
		m_invalidSettings = true;
		return;
	}

	m_specialPowerModule = obj->getSpecialPowerModule( data->m_specialPowerTemplate );

	//Create instances of the sounds required.
	m_bombardmentUnpack.setEventName( data->m_bombardmentUnpackName );
	m_bombardmentPack.setEventName(	data->m_bombardmentPackName );
	m_bombardmentAnnouncement.setEventName( data->m_bombardmentAnnouncementName );
	m_searchAndDestroyUnpack.setEventName( data->m_searchAndDestroyUnpackName );
	m_searchAndDestroyIdle.setEventName( data->m_searchAndDestroyIdleName );
	m_searchAndDestroyPack.setEventName( data->m_searchAndDestroyPackName );
	m_searchAndDestroyAnnouncement.setEventName( data->m_searchAndDestroyAnnouncementName );
	m_holdTheLineUnpack.setEventName( data->m_holdTheLineUnpackName );
	m_holdTheLinePack.setEventName(	data->m_holdTheLinePackName );
	m_holdTheLineAnnouncement.setEventName( data->m_holdTheLineAnnouncementName );
	TheAudio->getInfoForAudioEvent( &m_bombardmentUnpack );
	TheAudio->getInfoForAudioEvent( &m_bombardmentPack );
	TheAudio->getInfoForAudioEvent( &m_bombardmentAnnouncement );
	TheAudio->getInfoForAudioEvent( &m_searchAndDestroyUnpack );
	TheAudio->getInfoForAudioEvent( &m_searchAndDestroyIdle );
	TheAudio->getInfoForAudioEvent( &m_searchAndDestroyPack );
	TheAudio->getInfoForAudioEvent( &m_searchAndDestroyAnnouncement );
	TheAudio->getInfoForAudioEvent( &m_holdTheLineUnpack );
	TheAudio->getInfoForAudioEvent( &m_holdTheLinePack );
	TheAudio->getInfoForAudioEvent( &m_holdTheLineAnnouncement );

	getObject()->setWeaponSetFlag( WEAPONSET_VETERAN );
	AIUpdateInterface *ai = obj->getAI();
	if( ai )
	{
		// lock it just till the weapon is empty or the attack is "done"
		obj->setWeaponLock( PRIMARY_WEAPON, LOCKED_TEMPORARILY );
	}
	enableTurret( false );
}

//-------------------------------------------------------------------------------------------------
void BattlePlanUpdate::onCapture(Player* oldOwner, Player* newOwner)
{
#if !RETAIL_COMPATIBLE_CRC
	// TheSuperHackers @bugfix Stubbjax 04/11/2025 Transfer battle plan bonuses on capture.
	oldOwner->changeBattlePlan(m_planAffectingArmy, -1, m_bonuses);
	newOwner->changeBattlePlan(m_planAffectingArmy, 1, m_bonuses);
#endif
}

//-------------------------------------------------------------------------------------------------
Bool BattlePlanUpdate::initiateIntentToDoSpecialPower(const SpecialPowerTemplate *specialPowerTemplate, const Object *targetObj, const Coord3D *targetPos, const Waypoint *way, UnsignedInt commandOptions )
{
	if( m_specialPowerModule->getSpecialPowerTemplate() != specialPowerTemplate )
	{
		//Check to make sure our modules are connected.
		return FALSE;
	}

	//Set the desired status based on the command button option!
	if( BitIsSet( commandOptions, OPTION_ONE ) )
	{
		m_desiredPlan = PLANSTATUS_BOMBARDMENT;
	}
	else if( BitIsSet( commandOptions, OPTION_TWO ) )
	{
		m_desiredPlan = PLANSTATUS_HOLDTHELINE;
	}
	else if( BitIsSet( commandOptions, OPTION_THREE ) )
	{
		m_desiredPlan = PLANSTATUS_SEARCHANDDESTROY;
	}
	else
	{
		DEBUG_CRASH( ("Selected an unsupported strategy for strategy center.") );
		return FALSE;
	}

	getObject()->getControllingPlayer()->getAcademyStats()->recordBattlePlanSelected();
	return TRUE;
}

Bool BattlePlanUpdate::isPowerCurrentlyInUse( const CommandButton *command ) const
{
	//@todo -- perhaps we may need this one day...
	return false;
}

//-------------------------------------------------------------------------------------------------
CommandOption BattlePlanUpdate::getCommandOption() const
{
	switch( m_desiredPlan )
	{
		case PLANSTATUS_BOMBARDMENT:
			return OPTION_ONE;
		case PLANSTATUS_HOLDTHELINE:
			return OPTION_TWO;
		case PLANSTATUS_SEARCHANDDESTROY:
			return OPTION_THREE;
	}
	return (CommandOption)0;
}

//-------------------------------------------------------------------------------------------------
/** The update callback. */
//-------------------------------------------------------------------------------------------------
UpdateSleepTime BattlePlanUpdate::update()
{

	if( m_invalidSettings )
	{
		// can't return UPDATE_SLEEP_FOREVER unless we are sleepy...
		return UPDATE_SLEEP_NONE;
		///return UPDATE_SLEEP_FOREVER;
	}

	//const BattlePlanUpdateModuleData *data = getBattlePlanUpdateModuleData();
	//Object *obj = getObject();

	UnsignedInt now = TheGameLogic->getFrame();

	if( m_nextReadyFrame <= now )
	{
		switch( m_status )
		{
			case TRANSITIONSTATUS_IDLE:
				//There's only two cases where we are in an idle status -- upon initialization
				//when no plan has yet been selected. The other case is after we've finished
				//packing the previous plan up and are waiting to unpack the new state.
				if( m_desiredPlan != PLANSTATUS_NONE )
				{
					m_currentPlan = m_desiredPlan;
					setStatus( TRANSITIONSTATUS_UNPACKING );
				}
				break;
			case TRANSITIONSTATUS_UNPACKING:
				//If we're unpacking, we are forcing the user to wait until the plan is unpacked
				//before allowing him to select a new plan. The plan doesn't become active until
				//we're finished unpacking.
				setStatus( TRANSITIONSTATUS_ACTIVE );
				if( m_currentPlan == PLANSTATUS_BOMBARDMENT )
				{
					enableTurret( true );
				}
				break;
			case TRANSITIONSTATUS_ACTIVE:
				//If we're active and the user has selected a different plan, then we need to
				//pack up.
				if( m_currentPlan != m_desiredPlan )
				{
					if( m_currentPlan == PLANSTATUS_BOMBARDMENT )
					{
						//Special case situation -- in bombardment status, we need to center
						//the turret prior to packing up, so handle it here.
						AIUpdateInterface *ai = getObject()->getAI();
						if( ai )
						{
							if( isTurretInNaturalPosition() )
							{
								//It's centered, so pack
								setStatus( TRANSITIONSTATUS_PACKING );
								m_centeringTurret = false;
								enableTurret( false );
							}
							else if( !m_centeringTurret )
							{
								//It's not centered, and not trying to center, so order it to center.
								ai->aiIdle( CMD_FROM_AI );
#if !RETAIL_COMPATIBLE_CRC
								// TheSuperHackers @bugfix Stubbjax 19/02/2026 Prevent using the turret while it recenters.
								enableTurret(false);
#endif
								recenterTurret();
								m_centeringTurret = true;
							}
						}
					}
					else
					{
						setStatus( TRANSITIONSTATUS_PACKING );
					}
				}
				break;
			case TRANSITIONSTATUS_PACKING:
				//If we finished packing, then go idle until we can switch to our new plan.
				setStatus( TRANSITIONSTATUS_IDLE );
				break;
		}
	}

	return UPDATE_SLEEP_NONE;
}

// ------------------------------------------------------------------------------------------------
/** Create vision objects for all players revealing this building to all */
// ------------------------------------------------------------------------------------------------
void BattlePlanUpdate::createVisionObject()
{
	if (m_visionObjectID != INVALID_ID) // don't want two.
		return;

	const BattlePlanUpdateModuleData *data = getBattlePlanUpdateModuleData();
	Object *obj = getObject();

	// get template of object to create
	const ThingTemplate *tt = TheThingFactory->findTemplate( data->m_visionObjectName );
	DEBUG_ASSERTCRASH( tt, ("BattlePlanUpdate::setStatus - Invalid vision object name '%s'",
													data->m_visionObjectName.str()) );

	if (!tt)
		return;

	Player *pPlayer = ThePlayerList->getNeutralPlayer();
	// sanity
	if(!pPlayer)
		return;

	Object *visionObject;

	// create object for this player
	visionObject = TheThingFactory->newObject( tt, pPlayer->getDefaultTeam() );
	if( visionObject )
	{

		// record we have an object
		m_visionObjectID = visionObject->getID();

		// set position
		visionObject->setPosition( obj->getPosition() );

		// set the shroud clearing range
		visionObject->setShroudClearingRange( obj->getGeometryInfo().getBoundingSphereRadius() );

	}

}

//-------------------------------------------------------------------------------------------------
void BattlePlanUpdate::setStatus( TransitionStatus newStatus )
{
	const BattlePlanUpdateModuleData *data = getBattlePlanUpdateModuleData();
	Object *obj = getObject();

	if( m_status == newStatus )
	{
		return;
	}

	TransitionStatus oldStatus = m_status;

	//Turn off old defining states and sounds
	switch( oldStatus )
	{
		case TRANSITIONSTATUS_IDLE:
			break;

		case TRANSITIONSTATUS_UNPACKING:
			switch( m_currentPlan )
			{
				case PLANSTATUS_BOMBARDMENT:
					obj->clearModelConditionState( MODELCONDITION_DOOR_1_OPENING );
					TheAudio->removeAudioEvent( m_bombardmentUnpack.getPlayingHandle() );
					break;
				case PLANSTATUS_HOLDTHELINE:
					obj->clearModelConditionState( MODELCONDITION_DOOR_2_OPENING );
					TheAudio->removeAudioEvent( m_holdTheLineUnpack.getPlayingHandle() );
					break;
				case PLANSTATUS_SEARCHANDDESTROY:
					obj->clearModelConditionState( MODELCONDITION_DOOR_3_OPENING );
					TheAudio->removeAudioEvent( m_searchAndDestroyUnpack.getPlayingHandle() );
					break;
			}
			break;

		case TRANSITIONSTATUS_ACTIVE:
			switch( m_currentPlan )
			{
				case PLANSTATUS_BOMBARDMENT:
					obj->clearModelConditionState( MODELCONDITION_DOOR_1_WAITING_TO_CLOSE );
					break;
				case PLANSTATUS_HOLDTHELINE:
					obj->clearModelConditionState( MODELCONDITION_DOOR_2_WAITING_TO_CLOSE );
					break;
				case PLANSTATUS_SEARCHANDDESTROY:
					obj->clearModelConditionState( MODELCONDITION_DOOR_3_WAITING_TO_CLOSE );
					TheAudio->removeAudioEvent( m_searchAndDestroyIdle.getPlayingHandle() );
					break;
			}
			break;

		case TRANSITIONSTATUS_PACKING:
			switch( m_currentPlan )
			{
				case PLANSTATUS_BOMBARDMENT:
					obj->clearModelConditionState( MODELCONDITION_DOOR_1_CLOSING );
					TheAudio->removeAudioEvent( m_bombardmentPack.getPlayingHandle() );
					break;
				case PLANSTATUS_HOLDTHELINE:
					obj->clearModelConditionState( MODELCONDITION_DOOR_2_CLOSING );
					TheAudio->removeAudioEvent( m_holdTheLinePack.getPlayingHandle() );
					break;
				case PLANSTATUS_SEARCHANDDESTROY:
					obj->clearModelConditionState( MODELCONDITION_DOOR_3_CLOSING );
					TheAudio->removeAudioEvent( m_searchAndDestroyPack.getPlayingHandle() );
					break;
			}
			break;
	}

	UnsignedInt now = TheGameLogic->getFrame();

	//Handle entering new status
	switch( newStatus )
	{
		case TRANSITIONSTATUS_IDLE:
		{
			m_currentPlan = PLANSTATUS_NONE;
			m_nextReadyFrame = now + data->m_transitionIdleFrames;
			break;
		}

		case TRANSITIONSTATUS_UNPACKING:
		{

			// play a radar blip showing the battle plan change in the color of the
			TheRadar->createPlayerEvent( obj->getControllingPlayer(),
																	 obj->getPosition(),
																	 RADAR_EVENT_BATTLE_PLAN );

			//this is now handled with ShroudRevealToAllRange in thingTemplate
			//createVisionObject();

			switch( m_currentPlan )
			{
				case PLANSTATUS_BOMBARDMENT:
					obj->setModelConditionState( MODELCONDITION_DOOR_1_OPENING );
					obj->getDrawable()->setAnimationLoopDuration( data->m_bombardmentPlanAnimationFrames );
					m_nextReadyFrame = now + data->m_bombardmentPlanAnimationFrames;
					if( m_bombardmentUnpack.getEventName().isNotEmpty() )
					{
						m_bombardmentUnpack.setObjectID( obj->getID() );
						m_bombardmentUnpack.setPlayingHandle( TheAudio->addAudioEvent( &m_bombardmentUnpack ) );
					}

					// display a message to *all* users
					TheInGameUI->message( TheGameText->fetch( data->m_bombardmentMessageLabel ) );
					if( m_bombardmentAnnouncement.getEventName().isEmpty() == FALSE )
					{
						m_bombardmentAnnouncement.setPosition( obj->getPosition() );
						TheAudio->addAudioEvent( &m_bombardmentAnnouncement );
					}
					break;

				case PLANSTATUS_HOLDTHELINE:
					obj->setModelConditionState( MODELCONDITION_DOOR_2_OPENING );
					obj->getDrawable()->setAnimationLoopDuration( data->m_holdTheLinePlanAnimationFrames );
					m_nextReadyFrame = now + data->m_holdTheLinePlanAnimationFrames;
					if( m_holdTheLineUnpack.getEventName().isNotEmpty() )
					{
						m_holdTheLineUnpack.setObjectID( obj->getID() );
						m_holdTheLineUnpack.setPlayingHandle( TheAudio->addAudioEvent( &m_holdTheLineUnpack ) );
					}

					// display a message to *all* users
					TheInGameUI->message( TheGameText->fetch( data->m_holdTheLineMessageLabel ) );
					if( m_holdTheLineAnnouncement.getEventName().isEmpty() == FALSE )
					{
						m_holdTheLineAnnouncement.setPosition( obj->getPosition() );
						TheAudio->addAudioEvent( &m_holdTheLineAnnouncement );
					}
					break;

				case PLANSTATUS_SEARCHANDDESTROY:
					obj->setModelConditionState( MODELCONDITION_DOOR_3_OPENING );
					obj->getDrawable()->setAnimationLoopDuration( data->m_searchAndDestroyPlanAnimationFrames );
					m_nextReadyFrame = now + data->m_searchAndDestroyPlanAnimationFrames;
					if( m_searchAndDestroyUnpack.getEventName().isNotEmpty() )
					{
						m_searchAndDestroyUnpack.setObjectID( obj->getID() );
						m_searchAndDestroyUnpack.setPlayingHandle( TheAudio->addAudioEvent( &m_searchAndDestroyUnpack ) );
					}

					// display a message to *all* users
					TheInGameUI->message( TheGameText->fetch( data->m_searchAndDestroyMessageLabel ) );
					if( m_searchAndDestroyAnnouncement.getEventName().isEmpty() == FALSE )
					{
						m_searchAndDestroyAnnouncement.setPosition( obj->getPosition() );
						TheAudio->addAudioEvent( &m_searchAndDestroyAnnouncement );
					}
					break;

			}

			break;

		}

		case TRANSITIONSTATUS_ACTIVE:

			setBattlePlan( m_currentPlan );

			switch( m_currentPlan )
			{
				case PLANSTATUS_BOMBARDMENT:
					obj->setModelConditionState( MODELCONDITION_DOOR_1_WAITING_TO_CLOSE );
					break;
				case PLANSTATUS_HOLDTHELINE:
					obj->setModelConditionState( MODELCONDITION_DOOR_2_WAITING_TO_CLOSE );
					break;
				case PLANSTATUS_SEARCHANDDESTROY:
					obj->setModelConditionState( MODELCONDITION_DOOR_3_WAITING_TO_CLOSE );
					if( m_searchAndDestroyIdle.getEventName().isNotEmpty() )
					{
						m_searchAndDestroyIdle.setObjectID( obj->getID() );
						m_searchAndDestroyIdle.setPlayingHandle( TheAudio->addAudioEvent( &m_searchAndDestroyIdle ) );
					}
					break;
			}
			break;

		case TRANSITIONSTATUS_PACKING:

			setBattlePlan( PLANSTATUS_NONE );

			switch( m_currentPlan )
			{
				case PLANSTATUS_BOMBARDMENT:
					obj->setModelConditionState( MODELCONDITION_DOOR_1_CLOSING );
					obj->getDrawable()->setAnimationLoopDuration( data->m_bombardmentPlanAnimationFrames );
					m_nextReadyFrame = now + data->m_bombardmentPlanAnimationFrames;
					if( m_bombardmentUnpack.getEventName().isNotEmpty() )
					{
						m_bombardmentPack.setObjectID( obj->getID() );
						m_bombardmentPack.setPlayingHandle( TheAudio->addAudioEvent( &m_bombardmentPack ) );
					}
					break;
				case PLANSTATUS_HOLDTHELINE:
					obj->setModelConditionState( MODELCONDITION_DOOR_2_CLOSING );
					obj->getDrawable()->setAnimationLoopDuration( data->m_holdTheLinePlanAnimationFrames );
					m_nextReadyFrame = now + data->m_holdTheLinePlanAnimationFrames;
					if( m_holdTheLineUnpack.getEventName().isNotEmpty() )
					{
						m_holdTheLinePack.setObjectID( obj->getID() );
						m_holdTheLinePack.setPlayingHandle( TheAudio->addAudioEvent( &m_holdTheLinePack ) );
					}
					break;
				case PLANSTATUS_SEARCHANDDESTROY:
					obj->setModelConditionState( MODELCONDITION_DOOR_3_CLOSING );
					obj->getDrawable()->setAnimationLoopDuration( data->m_searchAndDestroyPlanAnimationFrames );
					m_nextReadyFrame = now + data->m_searchAndDestroyPlanAnimationFrames;
					if( m_searchAndDestroyUnpack.getEventName().isNotEmpty() )
					{
						m_searchAndDestroyPack.setObjectID( obj->getID() );
						m_searchAndDestroyPack.setPlayingHandle( TheAudio->addAudioEvent( &m_searchAndDestroyPack ) );
					}
					break;
			}
			break;
	}

	//Set new status.
	m_status = newStatus;
}

//------------------------------------------------------------------------------------------------
void BattlePlanUpdate::enableTurret( Bool enable )
{
	AIUpdateInterface *ai = getObject()->getAI();
	if( ai )
	{
		WhichTurretType tur = ai->getWhichTurretForCurWeapon();
		if( tur != TURRET_INVALID )
		{
			ai->setTurretEnabled( tur, enable );
		}
	}
}

//------------------------------------------------------------------------------------------------
void BattlePlanUpdate::recenterTurret()
{
	AIUpdateInterface *ai = getObject()->getAI();
	if( ai )
	{
		WhichTurretType tur = ai->getWhichTurretForCurWeapon();
		if( tur != TURRET_INVALID )
		{
			ai->recenterTurret( tur );
		}
	}
}

//------------------------------------------------------------------------------------------------
Bool BattlePlanUpdate::isTurretInNaturalPosition()
{
	AIUpdateInterface *ai = getObject()->getAI();
	if( ai )
	{
		WhichTurretType tur = ai->getWhichTurretForCurWeapon();
		if( tur != TURRET_INVALID )
		{
			return ai->isTurretInNaturalPosition( tur );
		}
	}
	return false;
}

//------------------------------------------------------------------------------------------------
static void paralyzeTroop( Object *obj, void *userData )
{
	const BattlePlanUpdateModuleData *data = (BattlePlanUpdateModuleData*)userData;
	if( obj->isAnyKindOf( data->m_validMemberKindOf ) )
	{
		if( !obj->isAnyKindOf( data->m_invalidMemberKindOf ) )
		{
			obj->setDisabledUntil( DISABLED_PARALYZED, TheGameLogic->getFrame() + data->m_battlePlanParalyzeFrames );
		}
	}
}

//------------------------------------------------------------------------------------------------
void BattlePlanUpdate::setBattlePlan( BattlePlanStatus plan )
{
	const BattlePlanUpdateModuleData *data = getBattlePlanUpdateModuleData();
	Object *obj = getObject();
	Player *player = obj->getControllingPlayer();
	if( player )
	{
		switch( m_planAffectingArmy )
		{
			case PLANSTATUS_BOMBARDMENT:
			{
				//Remove the previous plan!
				player->changeBattlePlan( PLANSTATUS_BOMBARDMENT, -1, m_bonuses );

				//The only building bonus is actually the turret, and that's already handled!
				break;
			}
			case PLANSTATUS_HOLDTHELINE:
			{
				//Remove the previous plan!
				player->changeBattlePlan( PLANSTATUS_HOLDTHELINE, -1, m_bonuses );

				//Remove building health bonuses
				if( data->m_strategyCenterHoldTheLineMaxHealthScalar != 1.0f )
				{
					BodyModuleInterface *body = obj->getBodyModule();
					body->setMaxHealth( body->getMaxHealth() * 1.0f / data->m_strategyCenterHoldTheLineMaxHealthScalar, data->m_strategyCenterHoldTheLineMaxHealthChangeType );
				}
				break;
			}
			case PLANSTATUS_SEARCHANDDESTROY:
			{
				//Remove the previous plan!
				player->changeBattlePlan( PLANSTATUS_SEARCHANDDESTROY, -1, m_bonuses );

				//Remove sight range bonus
				if( data->m_strategyCenterSearchAndDestroySightRangeScalar != 1.0f )
				{
					obj->setVisionRange( obj->getVisionRange() * 1.0f / data->m_strategyCenterSearchAndDestroySightRangeScalar );
					obj->setShroudClearingRange( obj->getShroudClearingRange() * 1.0f / data->m_strategyCenterSearchAndDestroySightRangeScalar );
				}

				//Remove stealth detection
				if( data->m_strategyCenterSearchAndDestroyDetectsStealth )
				{
					static NameKeyType key_StealthDetectorUpdate = NAMEKEY( "StealthDetectorUpdate" );
					StealthDetectorUpdate *update = (StealthDetectorUpdate*)obj->findUpdateModule( key_StealthDetectorUpdate );
					if( update )
					{
						update->setSDEnabled( false );
					}
				}

				break;
			}
		}

		//Revert to default no-bonuses!
		m_bonuses->m_armorScalar					= 1.0f;
		m_bonuses->m_sightRangeScalar		= 1.0f;
		m_bonuses->m_bombardment					= 0;
		m_bonuses->m_searchAndDestroy		= 0;
		m_bonuses->m_holdTheLine					= 0;

		//Add new bonuses!
		switch( plan )
		{
			case PLANSTATUS_NONE:
				//Paralyze troops!
				player->iterateObjects( paralyzeTroop, (void*)data );
				break;

			case PLANSTATUS_BOMBARDMENT:
				//Set the bombardment bonuses
				m_bonuses->m_bombardment	= 1; //for weapon bonuses

				//Add the new plan!
				player->changeBattlePlan( PLANSTATUS_BOMBARDMENT, 1, m_bonuses );
				break;

			case PLANSTATUS_HOLDTHELINE:
				//Add building health bonuses
				if( data->m_strategyCenterHoldTheLineMaxHealthScalar )
				{
					BodyModuleInterface *body = obj->getBodyModule();
					body->setMaxHealth( body->getMaxHealth() * data->m_strategyCenterHoldTheLineMaxHealthScalar, data->m_strategyCenterHoldTheLineMaxHealthChangeType );
				}

				//Set the hold-the-line bonuses
				m_bonuses->m_armorScalar = data->m_holdTheLineArmorDamageScalar;
				m_bonuses->m_holdTheLine	= 1; //for weapon bonuses

				//Add the new plan!
				player->changeBattlePlan( PLANSTATUS_HOLDTHELINE, 1, m_bonuses );
				break;
			case PLANSTATUS_SEARCHANDDESTROY:
				//Add sight range bonus
				if( data->m_strategyCenterSearchAndDestroySightRangeScalar != 1.0f )
				{
					obj->setVisionRange( obj->getVisionRange() * data->m_strategyCenterSearchAndDestroySightRangeScalar );
					obj->setShroudClearingRange( obj->getShroudClearingRange() * data->m_strategyCenterSearchAndDestroySightRangeScalar );
				}

				//Enable stealth detection
				if( data->m_strategyCenterSearchAndDestroyDetectsStealth )
				{
					static NameKeyType key_StealthDetectorUpdate = NAMEKEY( "StealthDetectorUpdate" );
					StealthDetectorUpdate *update = (StealthDetectorUpdate*)obj->findUpdateModule( key_StealthDetectorUpdate );
					if( update )
					{
						update->setSDEnabled( true );
					}
				}

				//Set the search-and-destroy bonuses
				m_bonuses->m_searchAndDestroy	= 1; //for weapon bonuses
				m_bonuses->m_sightRangeScalar	= data->m_searchAndDestroySightRangeScalar;

				//Add the new plan!
				player->changeBattlePlan( PLANSTATUS_SEARCHANDDESTROY, 1, m_bonuses );
				break;
		}
		m_planAffectingArmy = plan;
	}
}

//------------------------------------------------------------------------------------------------
//Returns the currently active battle plan -- unpacked and ready... returns PLANSTATUS_NONE if in
//transition!
//------------------------------------------------------------------------------------------------
BattlePlanStatus BattlePlanUpdate::getActiveBattlePlan() const
{
	if( m_status == TRANSITIONSTATUS_ACTIVE )
	{
		return m_planAffectingArmy;
	}
	return PLANSTATUS_NONE;
}

//------------------------------------------------------------------------------------------------
void BattlePlanUpdate::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

//------------------------------------------------------------------------------------------------
// Xfer method
//	Version Info:
//	1: Initial version
//------------------------------------------------------------------------------------------------
void BattlePlanUpdate::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// current plan
	xfer->xferUser( &m_currentPlan, sizeof( BattlePlanStatus ) );

	// desired plan
	xfer->xferUser( &m_desiredPlan, sizeof( BattlePlanStatus ) );

	// plan affecting army
	xfer->xferUser( &m_planAffectingArmy, sizeof( BattlePlanStatus ) );

	// status
	xfer->xferUser( &m_status, sizeof( TransitionStatus ) );

	// next ready frame
	xfer->xferUnsignedInt( &m_nextReadyFrame );

	// don't need to save this interface, it's retrieved on object creation
	// SpecialPowerModuleInterface *m_specialPowerModule;

	// invalid settings
	xfer->xferBool( &m_invalidSettings );

	// centering turret
	xfer->xferBool( &m_centeringTurret );

	// bonuses
	xfer->xferReal( &m_bonuses->m_armorScalar );
	xfer->xferInt( &m_bonuses->m_bombardment );
	xfer->xferInt( &m_bonuses->m_searchAndDestroy );
	xfer->xferInt( &m_bonuses->m_holdTheLine );
	xfer->xferReal( &m_bonuses->m_sightRangeScalar );
	m_bonuses->m_validKindOf.xfer( xfer );
	m_bonuses->m_invalidKindOf.xfer( xfer );

	// vision object data
	xfer->xferObjectID( &m_visionObjectID );

}

//------------------------------------------------------------------------------------------------
void BattlePlanUpdate::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
