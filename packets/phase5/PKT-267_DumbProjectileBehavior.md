# Translation Packet `PKT-267`: `DumbProjectileBehavior`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-267`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/behavior/dumb_projectile_behavior.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (27):**
  - `Common/BezierSegment.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/BezierSegment.h`
  - `Common/GameCommon.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameCommon.h`
  - `Common/GameState.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/GameState.h`
  - `Common/GameType.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameType.h`
  - `Common/GlobalData.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/GlobalData.h`
  - `Common/INI.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/INI.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/RandomValue.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/RandomValue.h`
  - `Common/STLTypedefs.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/STLTypedefs.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/FXList.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/FXList.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/CollideModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CollideModule.h`
  - `GameLogic/Module/ContainModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ContainModule.h`
  - `GameLogic/Module/DumbProjectileBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DumbProjectileBehavior.h`
  - `GameLogic/Module/MissileAIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/MissileAIUpdate.h`
  - `GameLogic/Module/PhysicsUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PhysicsUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `GameLogic/Weapon.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h`
  - `GameLogic/WeaponBonusConditionFlags.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/WeaponBonusConditionFlags.h`
  - `PreRTS.h`
  - `WWMath/matrix3d.h`
- **Implemented Module Facets (1):**
  - `ProjectileUpdateInterface* getProjectileUpdateInterface()`
- **INI `FieldParse` Table Entries (13):**
  - `"MaxLifespan" => INI::parseDurationUnsignedInt`
  - `"TumbleRandomly" => INI::parseBool`
  - `"DetonateCallsKill" => INI::parseBool`
  - `"OrientToFlightPath" => INI::parseBool`
  - `"FirstHeight" => INI::parseReal`
  - `"SecondHeight" => INI::parseReal`
  - `"FirstPercentIndent" => INI::parsePercentToReal`
  - `"SecondPercentIndent" => INI::parsePercentToReal`
  - `"GarrisonHitKillRequiredKindOf" => KindOfMaskType::parseFromINI`
  - `"GarrisonHitKillForbiddenKindOf" => KindOfMaskType::parseFromINI`
  - `"GarrisonHitKillCount" => INI::parseUnsignedInt`
  - `"GarrisonHitKillFX" => INI::parseFXList`
  - `"FlightPathAdjustDistPerSecond" => INI::parseVelocityReal`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (10 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferObjectID( &m_launcherID )`
  3. `xferObjectID( &m_victimID )`
  4. `xferInt( &m_flightPathSegments )`
  5. `xferReal( &m_flightPathSpeed )`
  6. `xferCoord3D( &m_flightPathStart )`
  7. `xferCoord3D( &m_flightPathEnd )`
  8. `xferAsciiString( &weaponTemplateName )`
  9. `xferUnsignedInt( &m_lifespanFrame )`
  10. `xferInt( &m_currentFlightPathStep )`
- **PRNG Calls (3 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValueReal( -1.0f/PI, 1.0f/PI ) )`
  - `GameLogicRandomValueReal( -1.0f/PI, 1.0f/PI ) )`
  - `GameLogicRandomValueReal( -1.0f/PI, 1.0f/PI ) )`
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if RETAIL_COMPATIBLE_CRC`
  - `#if RETAIL_COMPATIBLE_XFER_SAVE`

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
- `zh_logic::arena`: `ObjectSlot`, `INVALID`, `fn`, `ModuleSlot`, `ModuleDataId`, `ThingTemplateId`, `FacetMask`, `NONE`, `UPDATE`, `DIE`, `DAMAGE`, `CREATE`, `COLLIDE`, `BODY`, ... (+42)
- `zh_logic::modules::body::active_body`: `ActiveBodyModuleData`, `fn`, `ActiveBody`
- `zh_logic::modules::body::body_module`: `BodyModuleData`, `BodyModuleBase`, `fn`, `BodyModule`
- `zh_logic::modules::body::highlander_body`: `HighlanderBody`
- `zh_logic::modules::body::hive_structure_body`: `HiveStructureBodyModuleData`, `fn`, `HiveStructureBody`
- `zh_logic::modules::body::immortal_body`: `ImmortalBody`
- `zh_logic::modules::body::inactive_body`: `InactiveBody`, `fn`
- `zh_logic::modules::body::structure_body`: `StructureBody`
- `zh_logic::modules::body::undead_body`: `UndeadBodyModuleData`, `fn`, `UndeadBody`
- `zh_logic::modules::collide::collide_module`: `CollideModuleData`, `CollideModule`, `fn`, `CollideBehaviorImpl`
- `zh_logic::modules::collide::convert_to_car_bomb_crate_collide`: `ConvertToCarBombCrateCollideModuleData`, `ConvertToCarBombCrateCollide`
- `zh_logic::modules::collide::convert_to_hijacked_vehicle_crate_collide`: `ConvertToHijackedVehicleCrateCollideModuleData`, `ConvertToHijackedVehicleCrateCollide`
- `zh_logic::modules::collide::crate_collide`: `PRESERVE_MULTI_CRATE_PICKUP`, `SabotageVictimType`, `CrateCollideData`, `CrateCollide`, `CrateCollideImpl`
- `zh_logic::modules::collide::fire_weapon_collide`: `FireWeaponCollideModuleData`, `FireWeaponCollide`
- `zh_logic::modules::collide::heal_crate_collide`: `HealCrateCollideModuleData`, `HealCrateCollide`
- `zh_logic::modules::collide::money_crate_collide`: `UpgradePair`, `MoneyCrateCollideModuleData`, `MoneyCrateCollide`
- `zh_logic::modules::collide::sabotage_command_center_crate_collide`: `SabotageCommandCenterCrateCollideModuleData`, `SabotageCommandCenterCrateCollide`
- `zh_logic::modules::collide::sabotage_fake_building`: `SabotageFakeBuildingCrateCollideModuleData`, `SabotageFakeBuildingCrateCollide`
- `zh_logic::modules::collide::sabotage_internet_center_crate_collide`: `SabotageInternetCenterCrateCollideModuleData`, `SabotageInternetCenterCrateCollide`
- `zh_logic::modules::collide::sabotage_military_factory_crate_collide`: `SabotageMilitaryFactoryCrateCollideModuleData`, `SabotageMilitaryFactoryCrateCollide`
- `zh_logic::modules::collide::sabotage_power_plant_crate_collide`: `SabotagePowerPlantCrateCollideModuleData`, `SabotagePowerPlantCrateCollide`
- `zh_logic::modules::collide::sabotage_superweapon_crate_collide`: `SabotageSuperweaponCrateCollideModuleData`, `SabotageSuperweaponCrateCollide`
- `zh_logic::modules::collide::sabotage_supply_center_crate_collide`: `SabotageSupplyCenterCrateCollideModuleData`, `SabotageSupplyCenterCrateCollide`
- `zh_logic::modules::collide::sabotage_supply_dropzone_crate_collide`: `SabotageSupplyDropzoneCrateCollideModuleData`, `SabotageSupplyDropzoneCrateCollide`
- `zh_logic::modules::collide::salvage_crate_collide`: `SalvageCrateCollideModuleData`, `SalvageCrateCollide`
- `zh_logic::modules::collide::shroud_crate_collide`: `ShroudCrateCollideModuleData`, `ShroudCrateCollide`
- `zh_logic::modules::collide::squish_collide`: `SquishCollideModuleData`, `SquishCollide`
- `zh_logic::modules::collide::unit_crate_collide`: `UnitCrateCollideModuleData`, `UnitCrateCollide`
- `zh_logic::modules::collide::veterancy_crate_collide`: `VeterancyCrateCollideModuleData`, `VeterancyCrateCollide`
- `zh_logic::modules::create::create_module`: `CreateModuleData`, `CreateModuleBase`, `fn`, `CreateModule`
- `zh_logic::modules::create::grant_upgrade_create`: `GrantUpgradeCreateModuleData`, `GrantUpgradeCreate`
- `zh_logic::modules::create::lock_weapon_create`: `LockWeaponCreateModuleData`, `LockWeaponCreate`
- `zh_logic::modules::create::preorder_create`: `PreorderCreate`
- `zh_logic::modules::create::special_power_create`: `SpecialPowerCreate`
- `zh_logic::modules::create::supply_center_create`: `SupplyCenterCreate`
- `zh_logic::modules::create::supply_warehouse_create`: `SupplyWarehouseCreate`
- `zh_logic::modules::create::veterancy_gain_create`: `VeterancyGainCreateModuleData`, `VeterancyGainCreate`
- `zh_logic::modules::damage::bone_fx_damage`: `BoneFXDamage`, `fn`
- `zh_logic::modules::damage::damage_module`: `DamageModuleData`, `DamageModuleBase`, `fn`, `DamageModule`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DumbProjectileBehavior.h`
```cpp
// FILE:		DumbProjectileBehavior.h
// Author:	Steven Johnson, July 2002
// Desc:

#pragma once

#include "Common/GameType.h"
#include "Common/GlobalData.h"
#include "Common/STLTypedefs.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/CollideModule.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/WeaponBonusConditionFlags.h"
#include "Common/INI.h"
#include "WWMath/matrix3d.h"

class ParticleSystem;
class FXList;


//-------------------------------------------------------------------------------------------------
class DumbProjectileBehaviorModuleData : public UpdateModuleData
{
public:
	/**
		These four data define a Bezier curve.  The first and last control points are the firer and victim.
	*/
	Real m_firstHeight;					///< The first airborne control point will be this high above the highest intervening terrain
	Real m_secondHeight;				///< And the second, this.
	Real m_firstPercentIndent;	///< The first point will be this percent along the target line
	Real m_secondPercentIndent;	///< And the second, this.
	UnsignedInt m_maxLifespan;
	Bool m_tumbleRandomly;
	Bool m_orientToFlightPath;
	Bool m_detonateCallsKill;
	Int							m_garrisonHitKillCount;
	KindOfMaskType	m_garrisonHitKillKindof;			///< the kind(s) of units that can be collided with
	KindOfMaskType	m_garrisonHitKillKindofNot;		///< the kind(s) of units that CANNOT be collided with
	const FXList*		m_garrisonHitKillFX;
	Real m_flightPathAdjustDistPerFrame;


	DumbProjectileBehaviorModuleData();

	static void buildFieldParse(MultiIniFieldParse& p);

};

//-------------------------------------------------------------------------------------------------
class DumbProjectileBehavior : public UpdateModule, public ProjectileUpdateInterface
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( DumbProjectileBehavior, "DumbProjectileBehavior" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( DumbProjectileBehavior, DumbProjectileBehaviorModuleData );

public:

	DumbProjectileBehavior( Thing *thing, const ModuleData* moduleData );
	// virtual destructor provided by memory pool object

	// UpdateModuleInterface
	virtual UpdateSleepTime update() override;
	virtual ProjectileUpdateInterface* getProjectileUpdateInterface() override { return this; }

	// ProjectileUpdateInterface
	virtual void projectileLaunchAtObjectOrPosition(const Object *victim, const Coord3D* victimPos, const Object *launcher, WeaponSlotType wslot, Int specificBarrelToUse, const WeaponTemplate* detWeap, const ParticleSystemTemplate* exhaustSysOverride) override;
	virtual void projectileFireAtObjectOrPosition( const Object *victim, const Coord3D *victimPos, const WeaponTemplate *detWeap, const ParticleSystemTemplate* exhaustSysOverride ) override;
	virtual Bool projectileHandleCollision( Object *other ) override;
	virtual Bool projectileIsArmed() const override { return true; }
	virtual ObjectID projectileGetLauncherID() const override { return m_launcherID; }
	virtual void setFramesTillCountermeasureDiversionOccurs( UnsignedInt frames ) override {}
	virtual void projectileNowJammed() override {}

protected:

	void positionForLaunch(const Object *launcher, WeaponSlotType wslot, Int specificBarrelToUse);
	void detonate();

private:

	ObjectID							m_launcherID;							///< ID of object that launched us (zero if not yet launched)
	ObjectID							m_victimID;								///< ID of object we are targeting (zero if not yet launched)
	const WeaponTemplate*	m_detonationWeaponTmpl;		///< weapon to fire at end (or null)
	UnsignedInt						m_lifespanFrame;					///< if we haven't collided by this frame, blow up anyway
	VecCoord3D						m_flightPath;							///< The frame by frame flight path in a Bezier curve
	Coord3D								m_flightPathStart;				///< where flight path started (in case we must regen it)
	Coord3D								m_flightPathEnd;					///< where flight path ends (in case we must regen it)
	Real									m_flightPathSpeed;				///< flight path speed (in case we must regen it)
	Int										m_flightPathSegments;			///< number of segments in the flightpath (in case we must regen it)
	Int										m_currentFlightPathStep;	///< Our current index in the flight path vector.  Quicker than popping off.
	WeaponBonusConditionFlags		m_extraBonusFlags;

  Bool                  m_hasDetonated;           ///<

	Bool calcFlightPath(Bool recalcNumSegments);
#if defined(RTS_DEBUG)
	void displayFlightPath();	///< Uses little debug icons in worldspace to show the path chosen when it is decided upon
#endif

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Behavior/DumbProjectileBehavior.cpp`
```cpp
// FILE: DumbProjectileBehavior.cpp
// Author: Steven Johnson, July 2002
// Desc:

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/BezierSegment.h"
#include "Common/GameCommon.h"
#include "Common/GameState.h"
#include "Common/Player.h"
#include "Common/ThingTemplate.h"
#include "Common/RandomValue.h"
#include "Common/Xfer.h"
#include "GameClient/Drawable.h"
#include "GameClient/FXList.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/DumbProjectileBehavior.h"
#include "GameLogic/Module/MissileAIUpdate.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Weapon.h"


//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------

//-----------------------------------------------------------------------------
const Int DEFAULT_MAX_LIFESPAN = 10 * LOGICFRAMES_PER_SECOND;

//-----------------------------------------------------------------------------
DumbProjectileBehaviorModuleData::DumbProjectileBehaviorModuleData() :
	m_maxLifespan(DEFAULT_MAX_LIFESPAN),
	m_detonateCallsKill(FALSE),
	m_orientToFlightPath(TRUE),
	m_tumbleRandomly(FALSE),
	m_firstHeight(0.0f),
	m_secondHeight(0.0f),
	m_firstPercentIndent(0.0f),
	m_secondPercentIndent(0.0f),
	m_garrisonHitKillCount(0),
	m_garrisonHitKillFX(nullptr),
	m_flightPathAdjustDistPerFrame(0.0f)
{
}

//-----------------------------------------------------------------------------
void DumbProjectileBehaviorModuleData::buildFieldParse(MultiIniFieldParse& p)
{
  UpdateModuleData::buildFieldParse(p);

	static const FieldParse dataFieldParse[] =
	{
		{ "MaxLifespan", INI::parseDurationUnsignedInt, nullptr, offsetof( DumbProjectileBehaviorModuleData, m_maxLifespan ) },
		{ "TumbleRandomly", INI::parseBool, nullptr, offsetof( DumbProjectileBehaviorModuleData, m_tumbleRandomly ) },
		{ "DetonateCallsKill", INI::parseBool, nullptr, offsetof( DumbProjectileBehaviorModuleData, m_detonateCallsKill ) },
		{ "OrientToFlightPath", INI::parseBool, nullptr, offsetof( DumbProjectileBehaviorModuleData, m_orientToFlightPath ) },

		{ "FirstHeight",					INI::parseReal,						nullptr, offsetof( DumbProjectileBehaviorModuleData, m_firstHeight ) },
		{ "SecondHeight",					INI::parseReal,						nullptr, offsetof( DumbProjectileBehaviorModuleData, m_secondHeight ) },
		{ "FirstPercentIndent",		INI::parsePercentToReal,	nullptr, offsetof( DumbProjectileBehaviorModuleData, m_firstPercentIndent ) },
		{ "SecondPercentIndent",	INI::parsePercentToReal,	nullptr, offsetof( DumbProjectileBehaviorModuleData, m_secondPercentIndent ) },

		{ "GarrisonHitKillRequiredKindOf", KindOfMaskType::parseFromINI, nullptr, offsetof( DumbProjectileBehaviorModuleData, m_garrisonHitKillKindof ) },
		{ "GarrisonHitKillForbiddenKindOf", KindOfMaskType::parseFromINI, nullptr, offsetof( DumbProjectileBehaviorModuleData, m_garrisonHitKillKindofNot ) },
		{ "GarrisonHitKillCount", INI::parseUnsignedInt, nullptr, offsetof( DumbProjectileBehaviorModuleData, m_garrisonHitKillCount ) },
		{ "GarrisonHitKillFX", INI::parseFXList, nullptr, offsetof( DumbProjectileBehaviorModuleData, m_garrisonHitKillFX ) },

		{ "FlightPathAdjustDistPerSecond", INI::parseVelocityReal, nullptr, offsetof( DumbProjectileBehaviorModuleData, m_flightPathAdjustDistPerFrame ) },


		{ nullptr, nullptr, nullptr, 0 }
	};

  p.add(dataFieldParse);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------------------
DumbProjectileBehavior::DumbProjectileBehavior( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{
	m_launcherID = INVALID_ID;
	m_victimID = INVALID_ID;
	m_detonationWeaponTmpl = nullptr;
	m_lifespanFrame = 0;
	m_flightPath.clear();
	m_flightPathSegments = 0;
	m_flightPathSpeed = 0;
	m_flightPathStart.zero();
	m_flightPathEnd.zero();
	m_currentFlightPathStep = 0;
	m_extraBonusFlags = 0;

  m_hasDetonated = FALSE;
}

//-------------------------------------------------------------------------------------------------
DumbProjectileBehavior::~DumbProjectileBehavior()
{
}

//-------------------------------------------------------------------------------------------------
inline Bool within(Real min, Real val, Real max)
{
	return min <= val && val <= max;
}

//-------------------------------------------------------------------------------------------------
inline Bool notWithin(Real min, Real val, Real max)
{
	return val < min || val > max;
}

#ifdef NOT_IN_USE

#define NO_ONLY_RETURN_CLIPPED_PITCHES

//-------------------------------------------------------------------------------------------------
static Bool calcTrajectory(
	const Coord3D& start,		// in: where the projectile starts
	const Coord3D& end,			// in: where the projectile wants to end up
	Real velocity,					// in: the initial speed of the projectile
	Real minPitch,					// in: min pitch (-PI/2)
	Real maxPitch,					// in: max pitch (-PI/2)
	Bool preferShortPitch,	// in: prefer the shorter or longer path?
	Real& angle,						// out: the angle to aim
	Real& pitch							// out: the pitch to aim for
)
{
	Bool exactTarget = false;

	angle = 0.0f;
	pitch = 0.0f;

	if (velocity <= 0.0f)
	{
		DEBUG_CRASH(("cant get there from here (1)"));
		return false;
	}

	Real dx = end.x - start.x;
	Real dy = end.y - start.y;
	Real dz = end.z - start.z;

	// calculating the angle is trivial.
	angle = WWMath::Atan2(dy, dx);

	// calculating the pitch requires a bit more effort.
	Real horizDistSqr = sqr(dx) + sqr(dy);
	Real horizDist = WWMath::Sqrt(horizDistSqr);

	// calc the two possible pitches that will cover the given horizontal range.
	// (this is actually only true if dz==0, but is a good first guess)
	Real gravity = WWMath::Fabs(TheGlobalData->m_gravity);
	Real gravityTwoDZ = gravity * 2.0f * dz;

	// let's start by aiming directly for it. we know this isn't right (unless gravity
	// is zero, which it's not) but is a good starting point...
	Real theta = WWMath::Atan2(dz, horizDist);
	// if the angle isn't pretty shallow, we can get a better initial guess by using
	// the code below...
	const Real SHALLOW_ANGLE = 0.5f * PI / 180.0f;
	if (WWMath::Fabs(theta) > SHALLOW_ANGLE)
	{
		Real t = horizDist / velocity;
		Real vz = (dz/t + 0.5f*gravity*t);
		Real sineOfAngle = clamp(-1.0f, vz / velocity, 1.0f);
		theta = ASin(sineOfAngle)*0.5f;
	}

/*
	this is, in theory, the "right" formula for dz==0, but I
	get better results with the stuff above:

	Real sineOfAngle = (gravity * horizDist) / sqr(velocity);
	if (sineOfAngle > 1.0f)
	{
		return false;
	}
	Real theta = ASin(sineOfAngle)*0.5f;
*/

	Real pitches[2];
	Real cosPitches[2];
	Real sinPitches[2];
	Real theta_min = max(minPitch, -PI/2);
	Real theta_max = min(maxPitch, PI/2);
	const Real MIN_ANGLE_DIFF = (PI/(180.0f*16.0f));	// 1/16th of a degree. yes, we need that accuracy.
//Int numLoops = 0;
	while (theta_max > theta_min + MIN_ANGLE_DIFF)
	{
//++numLoops;
		pitches[0] = theta;	// shallower angle
		pitches[1] = (theta >= 0.0) ? (PI/2 - theta) : (-PI/2 - theta);	// steeper angle

		DEBUG_ASSERTCRASH(pitches[0]<=PI/2&&pitches[0]>=-PI/2,("bad pitches[0] %f",rad2deg(pitches[0])));
		DEBUG_ASSERTCRASH(pitches[1]<=PI/2&&pitches[1]>=-PI/2,("bad pitches[1] %f",rad2deg(pitches[1])));

		// calc the horiz-speed & time for each.
		// note that time can only be negative for 90<angle<270, and since we
		// ruled those out above, we're gold.
		sinPitches[0] = Sin(pitches[0]);
		sinPitches[1] = Sin(pitches[1]);
		cosPitches[0] = Cos(pitches[0]);
		cosPitches[1] = Cos(pitches[1]);
		Real t0 = (horizDist / (velocity * cosPitches[0]));
		Real t1 = (horizDist / (velocity * cosPitches[1]));

		t0 = MAX(0,t0);
		t1 = MAX(0,t1);


		DEBUG_ASSERTCRASH(t0>=0&&t1>=0,("neg time"));

		Int preferred = ((t0 < t1) == (preferShortPitch)) ? 0 : 1;

		// ok, NOW... since dz is virtually NEVER zero, do a little approximation
		// to get a better guess. (solving the equations directly are hard and
		// it's simpler to approximate)
		Bool tooClose = false;
		Real vx, vz, root;

		vz = velocity*sinPitches[preferred];

		root = sqr(vz) - gravityTwoDZ;
		if (root < 0.0f)
		{
			// oops, no solution for our preferred pitch. try the other one.
			if (preferred == 0)
				tooClose = true;	// if this fails for the shallow case, it's 'cuz the result is too close
			preferred = 1 - preferred;
			vz = velocity*sinPitches[preferred];
			root = sqr(vz) - gravityTwoDZ;
			if (root < 0.0f)
			{
				DEBUG_CRASH(("cant get there from here (2)"));
				return false;
			}
		}

#ifdef ONLY_RETURN_CLIPPED_PITCHES
		// if we get this far, we have some plausible solution...
		// it actually may be off, but let's go ahead and set the result
		// (along with the "exact" flag) so that the caller will want to
		// try to use the result. (we'll probably iterate and overwrite
		// this result with a more precise one.)
		if (within(minPitch, pitches[preferred], maxPitch))
		{
			pitch = pitches[preferred];
			exactTarget = true;
		}
#else
		// if we get this far, we have some plausible solution...
		// it actually may be off, but let's go ahead and set the result
		// (along with the "exact" flag) so that the caller will want to
		// try to use the result. (we'll probably iterate and overwrite
		// this result with a more precise one.) NOTE however that we
		// don't validate it's within the proper range... caller must do that!
		pitch = pitches[preferred];
		exactTarget = true;
#endif

		vx = velocity*cosPitches[preferred];
		Real actualRange = (vx*(vz + WWMath::Sqrt(root)))/gravity;
		const Real CLOSE_ENOUGH_RANGE = 5.0f;
		if (tooClose || (actualRange < horizDist - CLOSE_ENOUGH_RANGE))
		{
			theta_min = theta;
			theta = (theta + theta_max) * 0.5f;
		}
		else if (actualRange > horizDist + CLOSE_ENOUGH_RANGE)
		{
			theta_max = theta;
			theta = (theta + theta_min) * 0.5f;
		}
		else
		{
			break;
		}
	}

//DEBUG_LOG(("took %d loops to find a match",numLoops));
	if (exactTarget)
		return true;

	return false;
}

#endif // NOT_IN_USE

//-------------------------------------------------------------------------------------------------
// Prepares the missile for launch via proper weapon-system channels.
//-------------------------------------------------------------------------------------------------
void DumbProjectileBehavior::projectileLaunchAtObjectOrPosition(
	const Object* victim,
	const Coord3D* victimPos,
	const Object* launcher,
	WeaponSlotType wslot,
	Int specificBarrelToUse,
	const WeaponTemplate* detWeap,
	const ParticleSystemTemplate* exhaustSysOverride
)
{
	const DumbProjectileBehaviorModuleData* d = getDumbProjectileBehaviorModuleData();

	DEBUG_ASSERTCRASH(specificBarrelToUse>=0, ("specificBarrelToUse must now be explicit"));

	m_launcherID = launcher ? launcher->getID() : INVALID_ID;
	m_extraBonusFlags = launcher ? launcher->getWeaponBonusCondition() : 0;
	m_victimID = victim ? victim->getID() : INVALID_ID;
	m_detonationWeaponTmpl = detWeap;
	m_lifespanFrame = TheGameLogic->getFrame() + d->m_maxLifespan;

	Object* projectile = getObject();

	Weapon::positionProjectileForLaunch(projectile, launcher, wslot, specificBarrelToUse);

	projectileFireAtObjectOrPosition( victim, victimPos, detWeap, exhaustSysOverride );
}

//-------------------------------------------------------------------------------------------------
// The actual firing of the missile once setup. Uses a Bezier curve with points parameterized in ini
//-------------------------------------------------------------------------------------------------
void DumbProjectileBehavior::projectileFireAtObjectOrPosition( const Object *victim, const Coord3D *victimPos, const WeaponTemplate *detWeap, const ParticleSystemTemplate* exhaustSysOverride )
{
	const DumbProjectileBehaviorModuleData* d = getDumbProjectileBehaviorModuleData();
	Object* projectile = getObject();
	Real weaponSpeed = detWeap ? detWeap->getWeaponSpeed() : 0.0f;
	Real minWeaponSpeed = detWeap ? detWeap->getMinWeaponSpeed() : 0.0f;

	// if an object, aim at the center, not the ground part
	Coord3D victimPosToUse;
	if (victim)
		victim->getGeometryInfo().getCenterPosition(*victim->getPosition(), victimPosToUse);
	else
		victimPosToUse = *victimPos;

	if( detWeap  &&  detWeap->isScaleWeaponSpeed() )
	{
		// Some weapons want to scale their start speed to the range
		Real minRange = detWeap->getMinimumAttackRange();
		Real maxRange = detWeap->getUnmodifiedAttackRange();
		Real range = WWMath::Sqrt(ThePartitionManager->getDistanceSquared( projectile, &victimPosToUse, FROM_CENTER_2D ) );
		Real rangeRatio = (range - minRange) / (maxRange - minRange);
		m_flightPathSpeed = (rangeRatio * (weaponSpeed - minWeaponSpeed)) + minWeaponSpeed;
	}
	else
	{
		m_flightPathSpeed = weaponSpeed;
	}

 	PhysicsBehavior* physics = projectile->getPhysics();
 	if ( d->m_tumbleRandomly && physics)
	{
		physics->setPitchRate( GameLogicRandomValueReal( -1.0f/PI, 1.0f/PI ) );
		physics->setYawRate( GameLogicRandomValueReal( -1.0f/PI, 1.0f/PI ) );
		physics->setRollRate( GameLogicRandomValueReal( -1.0f/PI, 1.0f/PI ) );
	}

	m_flightPathStart = *getObject()->getPosition();
	m_flightPathEnd = victimPosToUse;
	if (!calcFlightPath(true))
	{
		//Can only fail if wildly incorrect points
		TheGameLogic->destroyObject( projectile );
		return;
	}
	m_currentFlightPathStep = 0;// We are at the first point, because the launching put us there
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Bool DumbProjectileBehavior::calcFlightPath(Bool recalcNumSegments)
{
	const DumbProjectileBehaviorModuleData* d = getDumbProjectileBehaviorModuleData();

	Coord3D controlPoints[4];
	//First point is us, last point is them
	controlPoints[0] = m_flightPathStart;
	controlPoints[3] = m_flightPathEnd;

	Real highestInterveningTerrain;
	Bool onMap = ThePartitionManager->estimateTerrainExtremesAlongLine( controlPoints[0], controlPoints[3], nullptr, &highestInterveningTerrain, nullptr, nullptr );
	if( !onMap )
	{
		return false;
	}

	// X and Y for inner points are along the line between us, so normalize and scale a vector between us, but
	// only use the x and y of the result
	Vector3 targetVector;// 0 origin vector between me and him
	targetVector.X = controlPoints[3].x - controlPoints[0].x;
	targetVector.Y = controlPoints[3].y - controlPoints[0].y;
	targetVector.Z = controlPoints[3].z - controlPoints[0].z;

	Real targetDistance = targetVector.Length();
	targetVector.Normalize();
	Vector3 firstPointAlongLine = targetVector * (targetDistance * d->m_firstPercentIndent );
	Vector3 secondPointAlongLine = targetVector * (targetDistance * d->m_secondPercentIndent );

	controlPoints[1].x = firstPointAlongLine.X + controlPoints[0].x;// add world start to offset along the origin based vector
	controlPoints[1].y = firstPointAlongLine.Y + controlPoints[0].y;
	controlPoints[2].x = secondPointAlongLine.X + controlPoints[0].x;
	controlPoints[2].y = secondPointAlongLine.Y + controlPoints[0].y;

	// Z's are determined using the highest intervening height so they won't hit hills, low end bounded by current Zs
	highestInterveningTerrain = max( highestInterveningTerrain, controlPoints[0].z );
	highestInterveningTerrain = max( highestInterveningTerrain, controlPoints[3].z );
	controlPoints[1].z = highestInterveningTerrain + d->m_firstHeight;
	controlPoints[2].z = highestInterveningTerrain + d->m_secondHeight;

	// With four control points, we have a curve.  We will decide how many frames we want to take to get to the target,
	// and fill our vector with those curve points.
	BezierSegment flightCurve( controlPoints );
	if (recalcNumSegments)
	{
		Real flightDistance = flightCurve.getApproximateLength();
		m_flightPathSegments = (Int)WWMath::Ceil( WWMath::Div_Safe(flightDistance, m_flightPathSpeed, 1.0f) );
	}

	// TheSuperHackers @info The way flight paths are used requires at least two curve points.
	// DumbProjectileBehavior::update has been modified to handle cases where the flight path consists of one or zero curve points.
	flightCurve.getSegmentPoints(m_flightPathSegments, &m_flightPath);
	DEBUG_ASSERTCRASH(m_flightPathSegments == m_flightPath.size(), ("m_flightPathSegments mismatch"));

#if defined(RTS_DEBUG)
	if( TheGlobalData->m_debugProjectilePath )
		displayFlightPath();
#endif

	return true;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Bool DumbProjectileBehavior::projectileHandleCollision( Object *other )
{
	const DumbProjectileBehaviorModuleData* d = getDumbProjectileBehaviorModuleData();

	if (other != nullptr)
	{
		Object *projectileLauncher = TheGameLogic->findObjectByID( projectileGetLauncherID() );

			// if it's not the specific thing we were targeting, see if we should incidentally collide...
		if (!m_detonationWeaponTmpl->shouldProjectileCollideWith(projectileLauncher, getObject(), other, m_victimID))
		{
			//DEBUG_LOG(("ignoring projectile collision with %s at frame %d",other->getTemplate()->getName().str(),TheGameLogic->getFrame()));
			return true;
		}

		if (d->m_garrisonHitKillCount > 0)
		{
			ContainModuleInterface* contain = other->getContain();
			if( contain && contain->getContainCount() > 0 && contain->isGarrisonable() && !contain->isImmuneToClearBuildingAttacks() )
			{
				Int numKilled = 0;

				// garrisonable buildings subvert the normal process here.
				const ContainedItemsList* items = contain->getContainedItemsList();
				if (items)
				{
					for (ContainedItemsList::const_iterator it = items->begin(); it != items->end() && numKilled < d->m_garrisonHitKillCount; )
					{
						Object* thingToKill = *it++;
						if (!thingToKill->isEffectivelyDead() && thingToKill->isKindOfMulti(d->m_garrisonHitKillKindof, d->m_garrisonHitKillKindofNot))
						{
							//DEBUG_LOG(("Killed a garrisoned unit (%08lx %s) via Flash-Bang!",thingToKill,thingToKill->getTemplate()->getName().str()));
							if (projectileLauncher)
								projectileLauncher->scoreTheKill( thingToKill );
							thingToKill->kill();
							++numKilled;
						}
					}
				}

				if (numKilled > 0)
				{
					// note, fx is played at center of building, not at grenade's location
					FXList::doFXObj(d->m_garrisonHitKillFX, other, nullptr);

					getObject()->getControllingPlayer()->getAcademyStats()->recordClearedGarrisonedBuilding();

					// don't do the normal explosion; just destroy ourselves & return
					TheGameLogic->destroyObject(getObject());

					return true;
				}
			}
		}

	}

	// collided with something... blow'd up!
	detonate();

	// mark ourself as "no collisions" (since we might still exist in slow death mode)
	getObject()->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_NO_COLLISIONS ) );
	return true;
}

//-------------------------------------------------------------------------------------------------
void DumbProjectileBehavior::detonate()
{
  if ( m_hasDetonated )
    return;

	Object* obj = getObject();
	if (m_detonationWeaponTmpl)
	{
		TheWeaponStore->handleProjectileDetonation(m_detonationWeaponTmpl, obj, obj->getPosition(), m_extraBonusFlags);

		if ( getDumbProjectileBehaviorModuleData()->m_detonateCallsKill )
		{
			// don't call kill(); do it manually, so we can specify DEATH_DETONATED
			DamageInfo damageInfo;
			damageInfo.in.m_damageType = DAMAGE_UNRESISTABLE;
			damageInfo.in.m_deathType = DEATH_DETONATED;
			damageInfo.in.m_sourceID = INVALID_ID;
			damageInfo.in.m_amount = obj->getBodyModule()->getMaxHealth();
			obj->attemptDamage( &damageInfo );
		}
		else
		{
			TheGameLogic->destroyObject( obj );
		}

	}
	else
	{
		// don't call kill(); do it manually, so we can specify DEATH_DETONATED
		DamageInfo damageInfo;
		damageInfo.in.m_damageType = DAMAGE_UNRESISTABLE;
		damageInfo.in.m_deathType = DEATH_DETONATED;
		damageInfo.in.m_sourceID = INVALID_ID;
		damageInfo.in.m_amount = obj->getBodyModule()->getMaxHealth();
		obj->attemptDamage( &damageInfo );
	}

	if (obj->getDrawable())
		obj->getDrawable()->setDrawableHidden(true);

  m_hasDetonated = TRUE;

}

//-------------------------------------------------------------------------------------------------
/**
 * Simulate one frame of a missile's behavior
 */
UpdateSleepTime DumbProjectileBehavior::update()
{
	const DumbProjectileBehaviorModuleData* d = getDumbProjectileBehaviorModuleData();

	if (m_lifespanFrame != 0 && TheGameLogic->getFrame() >= m_lifespanFrame)
	{
		// lifetime demands detonation
		detonate();
		return UPDATE_SLEEP_NONE;
	}

	// TheSuperHackers @info This check also covers the case where the flight path consists of zero curve points.
	if( m_currentFlightPathStep >= m_flightPath.size() )
	{
		// No more steps to use. Would go out of bounds on vector, so have to do something.
		// We could allow physics to take over and make us fall, but the point of this whole task
		// is to guarantee where the shell explodes.  This way, it _will_ explode at the target point.
		detonate();
		return UPDATE_SLEEP_NONE;
	}

	if (m_victimID != INVALID_ID && d->m_flightPathAdjustDistPerFrame > 0.0f)
	{
		Object* victim = TheGameLogic->findObjectByID(m_victimID);
		if (victim)
		{
			Coord3D newVictimPos;
			victim->getGeometryInfo().getCenterPosition(*victim->getPosition(), newVictimPos);
			Coord3D delta;
			delta.x = newVictimPos.x - m_flightPathEnd.x;
			delta.y = newVictimPos.y - m_flightPathEnd.y;
			delta.z = newVictimPos.z - m_flightPathEnd.z;
			Real distVictimMovedSqr = sqr(delta.x) + sqr(delta.y) + sqr(delta.z);
			if (distVictimMovedSqr > 0.1f)
			{
				Real distVictimMoved = WWMath::Sqrtf(distVictimMovedSqr);
				if (distVictimMoved > d->m_flightPathAdjustDistPerFrame)
					distVictimMoved = d->m_flightPathAdjustDistPerFrame;
				delta.normalize();
				m_flightPathEnd.x += distVictimMoved * delta.x;
				m_flightPathEnd.y += distVictimMoved * delta.y;
				m_flightPathEnd.z += distVictimMoved * delta.z;
				if (!calcFlightPath(false))
				{
					DEBUG_CRASH(("Hmm, recalc of flight path returned false... should this happen?"));
					detonate();
					return UPDATE_SLEEP_NONE;
				}
			}
		}
	}

	//Otherwise, continue to force the flight path
	Coord3D flightStep = m_flightPath[m_currentFlightPathStep];

	if (d->m_orientToFlightPath && (!d->m_tumbleRandomly) )
  {
    if ( m_currentFlightPathStep > 0)
	  {
	  // this seems reasonable; however, if this object has a PhysicsBehavior on it, this calc will be wrong,
	  // since Physics is applying gravity, which we duly ignore, but the prevPos won't be what we expect.
	  // get it from the flight path instead. (srj)
	  //Coord3D prevPos = *getObject()->getPosition();

		  Coord3D prevPos = m_flightPath[m_currentFlightPathStep - 1];

		  Vector3 curDir(flightStep.x - prevPos.x, flightStep.y - prevPos.y, flightStep.z - prevPos.z);
		  curDir.Normalize();	// buildTransformMatrix wants it this way
      Matrix3D orientMtx;
		  orientMtx.buildTransformMatrix(Vector3(flightStep.x, flightStep.y, flightStep.z), curDir);
		  getObject()->setTransformMatrix(&orientMtx);
    }
    else // oops! how do we orient the projectile on the zeroeth frame? This didn't matter until we started using the
      //long, blurry projectile graphics which look badly oriented on step 0 of the flight path
      // so lets orient it the same as if it were on frame 1!
    {
			// TheSuperHackers @bugfix Caball009 10/01/2026 Check vector size before accessing the second element to prevent out of bounds access.
			// The non-deterministic behavior for retail clients cannot be fixed, so this will remain a source of potential mismatches in retail compatibility mode.
			// Use the flight path start and end coordinates if needed, so that the behavior is deterministic for patched clients.
			Coord3D prevPos;
			Coord3D curPos;

			if (m_flightPath.size() >= 2)
			{
				prevPos = m_flightPath[0];
				curPos = m_flightPath[1];
			}
			else
			{
#if RETAIL_COMPATIBLE_CRC
				DEBUG_CRASH(("A mismatch is likely to happen if this code path is used in a match with unpatched clients."
					" Vector is expected to contain two or more elements; check the weapon speed value."));
#endif

				prevPos = m_flightPathStart;
				curPos = m_flightPathEnd;
				flightStep = m_flightPathEnd;
			}

		  Vector3 curDir(curPos.x - prevPos.x, curPos.y - prevPos.y, curPos.z - prevPos.z);
		  curDir.Normalize();	// buildTransformMatrix wants it this way
      Matrix3D orientMtx;
		  orientMtx.buildTransformMatrix(Vector3(flightStep.x, flightStep.y, flightStep.z), curDir);
		  getObject()->setTransformMatrix(&orientMtx);
    }

	}
	else
	{
		getObject()->setPosition(&flightStep);
	}

	// note that we want to use getHighestLayerForDestination() here, so that anything even slightly
	// below the bridge translates into GROUND. (getLayerForDestination just does a "closest" check)
	PathfindLayerEnum oldLayer = getObject()->getLayer();
	PathfindLayerEnum newLayer = TheTerrainLogic->getHighestLayerForDestination(getObject()->getPosition());
	getObject()->setLayer(newLayer);

	if (oldLayer != LAYER_GROUND && newLayer == LAYER_GROUND)
	{
		// see if we' still in the bridge's xy area
		Coord3D tmp = *getObject()->getPosition();
		tmp.z = 9999.0f;
		PathfindLayerEnum testLayer = TheTerrainLogic->getHighestLayerForDestination(&tmp);
		if (testLayer == oldLayer)
		{
			// ensure we are slightly above the bridge, to account for fudge & sloppy art
			const Real FUDGE = 2.0f;
			tmp.z = TheTerrainLogic->getLayerHeight(tmp.x, tmp.y, testLayer) + FUDGE;
			getObject()->setPosition(&tmp);
			// blow'd up!
			detonate();
			return UPDATE_SLEEP_NONE;
		}
	}

	++m_currentFlightPathStep;

	return UPDATE_SLEEP_NONE;//This no longer flys with physics, so it needs to not sleep
}

// ------------------------------------------------------------------------------------------------
/** displayFlightPath for debugging */
// ------------------------------------------------------------------------------------------------
#if defined(RTS_DEBUG)
void DumbProjectileBehavior::displayFlightPath()
{
	extern void addIcon(const Coord3D *pos, Real width, Int numFramesDuration, RGBColor color);
	for( size_t pointIndex = 0; pointIndex < m_flightPath.size(); ++pointIndex )
	{
		addIcon(&m_flightPath[pointIndex], TheGlobalData->m_debugProjectileTileWidth,
										TheGlobalData->m_debugProjectileTileDuration,
										TheGlobalData->m_debugProjectileTileColor);
	}
}
#endif

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void DumbProjectileBehavior::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version
	* 2: TheSuperHackers @bugfix Added m_currentFlightPathStep for mid-flight save/load.
	*/
// ------------------------------------------------------------------------------------------------
void DumbProjectileBehavior::xfer( Xfer *xfer )
{

	// version
#if RETAIL_COMPATIBLE_XFER_SAVE
	XferVersion currentVersion = 1;
#else
	XferVersion currentVersion = 2;
#endif
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// launcher
	xfer->xferObjectID( &m_launcherID );

	// victim ID
	xfer->xferObjectID( &m_victimID );

	xfer->xferInt( &m_flightPathSegments );
	xfer->xferReal( &m_flightPathSpeed );
	xfer->xferCoord3D( &m_flightPathStart );
	xfer->xferCoord3D( &m_flightPathEnd );

	// weapon template
	AsciiString weaponTemplateName = AsciiString::TheEmptyString;
	if( m_detonationWeaponTmpl )
		weaponTemplateName = m_detonationWeaponTmpl->getName();
	xfer->xferAsciiString( &weaponTemplateName );
	if( xfer->getXferMode() == XFER_LOAD )
	{

		if( weaponTemplateName == AsciiString::TheEmptyString )
			m_detonationWeaponTmpl = nullptr;
		else
		{

			// find template
			m_detonationWeaponTmpl = TheWeaponStore->findWeaponTemplate( weaponTemplateName );

			// sanity
			if( m_detonationWeaponTmpl == nullptr )
			{

				DEBUG_CRASH(( "DumbProjectileBehavior::xfer - Unknown weapon template '%s'",
											weaponTemplateName.str() ));
				throw SC_INVALID_DATA;

			}

		}

	}

	// lifespan frame
	xfer->xferUnsignedInt( &m_lifespanFrame );

	if( version >= 2 )
	{
		xfer->xferInt( &m_currentFlightPathStep );
	}

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void DumbProjectileBehavior::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

	if( m_flightPathSegments > 0 )
	{
		calcFlightPath( false );
	}

}
```
