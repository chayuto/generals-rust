# Translation Packet `PKT-370`: `SpectreGunshipDeploymentUpdate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-370`
- **Work Package:** `WP-5.4` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/update/spectre_gunship_deployment_update.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (29):**
  - `Common/ClientUpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ClientUpdateModule.h`
  - `Common/KindOf.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/KindOf.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/PlayerList.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/PlayerList.h`
  - `Common/ThingFactory.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ThingFactory.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/ControlBar.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/ControlBar.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/FXList.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/FXList.h`
  - `GameClient/GameClient.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/GameClient.h`
  - `GameClient/ParticleSys.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/ParticleSys.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Locomotor.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Locomotor.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/ActiveBody.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ActiveBody.h`
  - `GameLogic/Module/ContainModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ContainModule.h`
  - `GameLogic/Module/LaserUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/LaserUpdate.h`
  - `GameLogic/Module/PhysicsUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PhysicsUpdate.h`
  - `GameLogic/Module/SpecialPowerModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpecialPowerModule.h`
  - `GameLogic/Module/SpecialPowerUpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpecialPowerUpdateModule.h`
  - `GameLogic/Module/SpectreGunshipDeploymentUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpectreGunshipDeploymentUpdate.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/ObjectIter.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ObjectIter.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `GameLogic/TerrainLogic.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/TerrainLogic.h`
  - `GameLogic/Weapon.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h`
  - `GameLogic/WeaponSet.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/WeaponSet.h`
  - `PreRTS.h`
- **Implemented Module Facets (1):**
  - `SpecialPowerUpdateInterface* getSpecialPowerUpdateInterface()`
- **INI `FieldParse` Table Entries (5):**
  - `"GunshipTemplateName" => INI::parseAsciiString`
  - `"RequiredScience" => INI::parseScience`
  - `"SpecialPowerTemplate" => INI::parseSpecialPowerTemplate`
  - `"AttackAreaRadius" => INI::parseReal`
  - `"CreateLocation" => INI::parseIndexList`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (2 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferObjectID( &m_gunshipID )`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpectreGunshipDeploymentUpdate.h`
```cpp
// FILE: SpectreGunshipDeploymentUpdate.h //////////////////////////////////////////////////////////////////////////
// Author: Mark Lorenzen, April 2003
// Desc:   Update module to handle deployment of the SpectreGunship Generals special power.from command center
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
enum ParticleSystemID CPP_11(: Int);
enum ScienceType CPP_11(: Int);

//#define MAX_OUTER_NODES 16

//#define PUCK


enum GunshipCreateLocType CPP_11(: Int)
{
	CREATE_GUNSHIP_AT_EDGE_NEAR_SOURCE,
  CREATE_GUNSHIP_AT_EDGE_FARTHEST_FROM_SOURCE,
	CREATE_GUNSHIP_AT_EDGE_NEAR_TARGET,
	CREATE_GUNSHIP_AT_EDGE_FARTHEST_FROM_TARGET,

	GUNSHIP_CREATE_LOC_COUNT
};



//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class SpectreGunshipDeploymentUpdateModuleData : public ModuleData
{
public:
	SpecialPowerTemplate *m_specialPowerTemplate;
	ScienceType						m_extraRequiredScience;		///< science required (if any) to actually execute this power
	WeaponTemplate	      *m_howitzerWeaponTemplate;
  AsciiString           m_gunshipTemplateName;
  AsciiString           m_gattlingTemplateName;
//  AsciiString           m_howitzerTemplateName;
  RadiusDecalTemplate   m_attackAreaDecalTemplate;
  RadiusDecalTemplate   m_targetingReticleDecalTemplate;
  UnsignedInt           m_orbitFrames;
  Real                  m_attackAreaRadius;
  Real                  m_targetingReticleRadius;
  Real                  m_gunshipOrbitRadius;
	GunshipCreateLocType	m_createLoc;


	const ParticleSystemTemplate * m_gattlingStrafeFXParticleSystem;

	SpectreGunshipDeploymentUpdateModuleData();
	static void buildFieldParse(MultiIniFieldParse& p);

private:

};

enum GunshipDeployStatus CPP_11(: Int)
{
   GUNSHIPDEPLOY_STATUS_INSERTING,
   GUNSHIPDEPLOY_STATUS_ORBITING,
   GUNSHIPDEPLOY_STATUS_DEPARTING,
   GUNSHIPDEPLOY_STATUS_IDLE,
};


//-------------------------------------------------------------------------------------------------
/** The default	update module */
//-------------------------------------------------------------------------------------------------
class SpectreGunshipDeploymentUpdate : public SpecialPowerUpdateModule
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( SpectreGunshipDeploymentUpdate, "SpectreGunshipDeploymentUpdate" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( SpectreGunshipDeploymentUpdate, SpectreGunshipDeploymentUpdateModuleData );

public:

	SpectreGunshipDeploymentUpdate( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	// SpecialPowerUpdateInterface
	virtual Bool initiateIntentToDoSpecialPower(const SpecialPowerTemplate *specialPowerTemplate, const Object *targetObj, const Coord3D *targetPos, const Waypoint *way, UnsignedInt commandOptions ) override;
	virtual Bool isSpecialAbility() const override { return false; }
	virtual Bool isSpecialPower() const override { return true; }
	virtual Bool isActive() const override {return FALSE;}
	virtual SpecialPowerUpdateInterface* getSpecialPowerUpdateInterface() override { return this; }
	virtual CommandOption getCommandOption() const override { return (CommandOption)0; }
  virtual Bool isPowerCurrentlyInUse( const CommandButton *command = nullptr ) const override { return FALSE; };
	virtual ScienceType getExtraRequiredScience() const override { return getSpectreGunshipDeploymentUpdateModuleData()->m_extraRequiredScience; } //Does this object have more than one special power module with the same spTemplate?

	virtual void onObjectCreated() override;
	virtual UpdateSleepTime update() override;

	void cleanUp();



  virtual Bool doesSpecialPowerHaveOverridableDestinationActive() const override { return FALSE; };
	virtual Bool doesSpecialPowerHaveOverridableDestination() const override { return FALSE; }	//Does it have it, even if it's not active?
  virtual void setSpecialPowerOverridableDestination( const Coord3D *loc ) override {};

	// Disabled conditions to process (termination conditions!)
	virtual DisabledMaskType getDisabledTypesToProcess() const override { return MAKE_DISABLED_MASK4( DISABLED_SUBDUED, DISABLED_UNDERPOWERED, DISABLED_EMP, DISABLED_HACKED ); }

protected:



	SpecialPowerModuleInterface* m_specialPowerModule;
  Coord3D				m_initialTargetPosition;
  ObjectID        m_gunshipID;


};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/SpectreGunshipDeploymentUpdate.cpp`
```cpp
// FILE: SpectreGunshipDeploymentUpdate.cpp //////////////////////////////////////////////////////////////////////////
// Author: Mark Lorenzen, April 2003
// Desc:   Update module to handle deployment of the SpectreGunship Generals special power.from command center
///////////////////////////////////////////////////////////////////////////////////////////////////

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#define DEFINE_DEATH_NAMES

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/ThingTemplate.h"
#include "Common/ThingFactory.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/Xfer.h"
#include "Common/ClientUpdateModule.h"

#include "GameClient/ControlBar.h"
#include "GameClient/GameClient.h"
#include "GameClient/Drawable.h"
#include "GameClient/ParticleSys.h"
#include "GameClient/FXList.h"

#include "GameLogic/Locomotor.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectIter.h"
#include "GameLogic/WeaponSet.h"
#include "GameLogic/Weapon.h"
#include "GameLogic/TerrainLogic.h"
#include "GameLogic/Module/SpecialPowerModule.h"
#include "GameLogic/Module/SpectreGunshipDeploymentUpdate.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Module/LaserUpdate.h"
#include "GameLogic/Module/ActiveBody.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/ContainModule.h"




//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
SpectreGunshipDeploymentUpdateModuleData::SpectreGunshipDeploymentUpdateModuleData()
{
	m_specialPowerTemplate			   = nullptr;
	m_extraRequiredScience				 = SCIENCE_INVALID;
/******BOTH*******//*BOTH*//******BOTH*******//******BOTH*******/  m_attackAreaRadius             = 200.0f;
	m_createLoc = CREATE_GUNSHIP_AT_EDGE_FARTHEST_FROM_TARGET;

}


static const char* const TheGunshipCreateLocTypeNames[] =
{
	"CREATE_AT_EDGE_NEAR_SOURCE",
  "CREATE_AT_EDGE_FARTHEST_FROM_SOURCE",
	"CREATE_AT_EDGE_NEAR_TARGET",
	"CREATE_AT_EDGE_FARTHEST_FROM_TARGET",
	nullptr
};
static_assert(ARRAY_SIZE(TheGunshipCreateLocTypeNames) == GUNSHIP_CREATE_LOC_COUNT + 1, "Incorrect array size");


static Real zero = 0.0f;
//-------------------------------------------------------------------------------------------------
/*static*/ void SpectreGunshipDeploymentUpdateModuleData::buildFieldParse(MultiIniFieldParse& p)
{
	ModuleData::buildFieldParse(p);

	static const FieldParse dataFieldParse[] =
	{
		{ "GunshipTemplateName",	    INI::parseAsciiString,				    nullptr, offsetof( SpectreGunshipDeploymentUpdateModuleData, m_gunshipTemplateName ) },
		{ "RequiredScience",					INI::parseScience,								nullptr, offsetof( SpectreGunshipDeploymentUpdateModuleData, m_extraRequiredScience ) },
/******BOTH*******/   { "SpecialPowerTemplate",     INI::parseSpecialPowerTemplate,   nullptr, offsetof( SpectreGunshipDeploymentUpdateModuleData, m_specialPowerTemplate ) },
/*******BOTH******/		{ "AttackAreaRadius",	        INI::parseReal,				            nullptr, offsetof( SpectreGunshipDeploymentUpdateModuleData, m_attackAreaRadius ) },
		{ "CreateLocation", INI::parseIndexList, TheGunshipCreateLocTypeNames, offsetof( SpectreGunshipDeploymentUpdateModuleData, m_createLoc ) },

    { nullptr, nullptr, nullptr, 0 }
	};
	p.add(dataFieldParse);
}

//-------------------------------------------------------------------------------------------------
SpectreGunshipDeploymentUpdate::SpectreGunshipDeploymentUpdate( Thing *thing, const ModuleData* moduleData ) : SpecialPowerUpdateModule( thing, moduleData )
{
	m_specialPowerModule = nullptr;
  m_gunshipID  = INVALID_ID;
}

//-------------------------------------------------------------------------------------------------
SpectreGunshipDeploymentUpdate::~SpectreGunshipDeploymentUpdate()
{
}

//-------------------------------------------------------------------------------------------------
// Validate that we have the necessary data from the ini file.
//-------------------------------------------------------------------------------------------------
void SpectreGunshipDeploymentUpdate::onObjectCreated()
{
	const SpectreGunshipDeploymentUpdateModuleData *data = getSpectreGunshipDeploymentUpdateModuleData();
	Object *obj = getObject();

	if( !data->m_specialPowerTemplate )
	{
		DEBUG_CRASH( ("%s object's SpectreGunshipDeploymentUpdate lacks access to the SpecialPowerTemplate. Needs to be specified in ini.", obj->getTemplate()->getName().str() ) );
		return;
	}

	m_specialPowerModule = obj->getSpecialPowerModule( data->m_specialPowerTemplate );
}

//-------------------------------------------------------------------------------------------------
Bool SpectreGunshipDeploymentUpdate::initiateIntentToDoSpecialPower(const SpecialPowerTemplate *specialPowerTemplate, const Object *targetObj, const Coord3D *targetPos, const Waypoint *way, UnsignedInt commandOptions )
{
	const SpectreGunshipDeploymentUpdateModuleData *data = getSpectreGunshipDeploymentUpdateModuleData();

	if( m_specialPowerModule->getSpecialPowerTemplate() != specialPowerTemplate )
	{
		//Check to make sure our modules are connected.
		return FALSE;
	}

//	getObject()->getControllingPlayer()->getAcademyStats()->recordSpecialPowerUsed( specialPowerTemplate );

	if( !BitIsSet( commandOptions, COMMAND_FIRED_BY_SCRIPT ) )
	{
/******CHANGE*******/		m_initialTargetPosition.set( *targetPos );
	}
	else
	{
		UnsignedInt now = TheGameLogic->getFrame();
		m_specialPowerModule->setReadyFrame( now );
/******CHANGE*******/   	m_initialTargetPosition.set( *targetPos );
//		setLogicalStatus( GUNSHIPDEPLOY_STATUS_INSERTING );
	}

   Object *newGunship = TheGameLogic->findObjectByID( m_gunshipID );
	const ThingTemplate *gunshipTemplate = TheThingFactory->findTemplate( data->m_gunshipTemplateName );
	if( newGunship != nullptr )
  {
//    disengageAndDepartAO( newGunship );
    m_gunshipID = INVALID_ID;
    newGunship = nullptr;
  }


  // Lets make a gunship, since we have none.
	{
		newGunship = TheThingFactory->newObject( gunshipTemplate, getObject()->getTeam() );
  }

  DEBUG_ASSERTCRASH( newGunship, ("SpecterGunshipUpdate failed to find or create a gunship object"));
  if ( newGunship )
  {
    //PRODUCER
    newGunship->setProducer( getObject() );

    //POSITION
    Coord3D creationCoord;
	  switch (data->m_createLoc)
	  {
		  case CREATE_GUNSHIP_AT_EDGE_NEAR_SOURCE:
			  creationCoord = TheTerrainLogic->findClosestEdgePoint( getObject()->getPosition() );
			  break;
		  case CREATE_GUNSHIP_AT_EDGE_FARTHEST_FROM_SOURCE:
			  creationCoord = TheTerrainLogic->findFarthestEdgePoint( getObject()->getPosition() );
			  break;
		  case CREATE_GUNSHIP_AT_EDGE_NEAR_TARGET:
			  creationCoord = TheTerrainLogic->findClosestEdgePoint(targetPos);
			  break;
		  case CREATE_GUNSHIP_AT_EDGE_FARTHEST_FROM_TARGET:
      default:
			  creationCoord = TheTerrainLogic->findFarthestEdgePoint(targetPos);
			  break;
	  }




      // HERE WE NEED TO CREATE THE POINT FURTHER OFF THE MAP SO WE CANT SEE THE LAME HOVER AND ACCELERATE BEHAVIOR
    Coord3D deltaToCreationPoint = m_initialTargetPosition;
    deltaToCreationPoint.sub( creationCoord );
    Real distanceFromTarget = deltaToCreationPoint.length();
    deltaToCreationPoint.normalize();
    deltaToCreationPoint.x *= ( distanceFromTarget + data->m_gunshipOrbitRadius );
    deltaToCreationPoint.y *= ( distanceFromTarget + data->m_gunshipOrbitRadius );
    creationCoord.x = m_initialTargetPosition.x - deltaToCreationPoint.x;
    creationCoord.y = m_initialTargetPosition.y - deltaToCreationPoint.y;

    Real preferredElevation = newGunship->getAI()->getCurLocomotor()->getPreferredHeight();
    creationCoord.z = preferredElevation;
    newGunship->setPosition( &creationCoord );

    //ORIENTATION
		Real orient = WWMath::Atan2( m_initialTargetPosition.y - creationCoord.y, m_initialTargetPosition.x - creationCoord.x);
    newGunship->setOrientation( orient );

    // ID
    m_gunshipID = newGunship->getID();

    // FIRE THE SPECIAL POWER OF THE GUNSHIP
	  SpecialPowerModuleInterface *shipSPMInterface = newGunship->getSpecialPowerModule( specialPowerTemplate );
	  if( shipSPMInterface )
	  {
		  SpecialPowerModule *spModule = (SpecialPowerModule*)shipSPMInterface;
		  spModule->markSpecialPowerTriggered( &m_initialTargetPosition );
      spModule->doSpecialPowerAtLocation( &m_initialTargetPosition, INVALID_ANGLE, commandOptions );

	  }

    // MAKE THE GUNSHIP SELECTED
    // TheSuperHackers @bugfix arcticdolphin 04/03/2026 Only select the gunship on the local client that controls the unit.
    TheGameLogic->selectObject( newGunship, TRUE, getObject()->getControllingPlayer()->getPlayerMask(), newGunship->isLocallyControlled() );


  }


	SpecialPowerModuleInterface *spmInterface = getObject()->getSpecialPowerModule( specialPowerTemplate );
	if( spmInterface )
	{
		SpecialPowerModule *spModule = (SpecialPowerModule*)spmInterface;
		spModule->markSpecialPowerTriggered( &m_initialTargetPosition );
	}

  return TRUE;
}




//-------------------------------------------------------------------------------------------------
/** The update callback. */
//-------------------------------------------------------------------------------------------------
UpdateSleepTime SpectreGunshipDeploymentUpdate::update()
{
//	const SpectreGunshipDeploymentUpdateModuleData *data = getSpectreGunshipDeploymentUpdateModuleData();


	Object *me = getObject();
	// Abort conditions.
	if( me->testStatus(OBJECT_STATUS_SOLD) ||
    me->testStatus(OBJECT_STATUS_UNDER_CONSTRUCTION) ||
    me->isEffectivelyDead() )
	{

		return UPDATE_SLEEP_FOREVER;
	}

	return UPDATE_SLEEP_NONE;

}










// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void SpectreGunshipDeploymentUpdate::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void SpectreGunshipDeploymentUpdate::xfer( Xfer *xfer )
{
//	const SpectreGunshipDeploymentUpdateModuleData *data = getSpectreGunshipDeploymentUpdateModuleData();

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );
  xfer->xferObjectID( &m_gunshipID );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void SpectreGunshipDeploymentUpdate::loadPostProcess()
{
	// extend base class
	UpdateModule::loadPostProcess();

}
```
