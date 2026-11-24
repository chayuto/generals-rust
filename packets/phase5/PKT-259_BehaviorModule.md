# Translation Packet `PKT-259`: `BehaviorModule`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-259`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/behavior/behavior_module.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (5):**
  - `Common/GameType.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameType.h`
  - `Common/Module.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Module.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `PreRTS.h`
- **Implemented Module Facets (33):**
  - `AIUpdateInterface* getAIUpdateInterface()`
  - `BodyModuleInterface* getBody()`
  - `BridgeBehaviorInterface* getBridgeBehaviorInterface()`
  - `BridgeScaffoldBehaviorInterface* getBridgeScaffoldBehaviorInterface()`
  - `BridgeTowerBehaviorInterface* getBridgeTowerBehaviorInterface()`
  - `CaveInterface* getCaveInterface()`
  - `CollideModuleInterface* getCollide()`
  - `ContainModuleInterface* getContain()`
  - `CountermeasuresBehaviorInterface* getCountermeasuresBehaviorInterface()`
  - `CreateModuleInterface* getCreate()`
  - `DamageModuleInterface* getDamage()`
  - `DestroyModuleInterface* getDestroy()`
  - `DieModuleInterface* getDie()`
  - `DieModuleInterface* getEjectPilotDieInterface()`
  - `DockUpdateInterface* getDockUpdateInterface()`
  - `ExitInterface* getUpdateExitInterface()`
  - `HordeUpdateInterface* getHordeUpdateInterface()`
  - `LandMineInterface* getLandMineInterface()`
  - `OverchargeBehaviorInterface* getOverchargeBehaviorInterface()`
  - `ParkingPlaceBehaviorInterface* getParkingPlaceBehaviorInterface()`
  - `PowerPlantUpdateInterface* getPowerPlantUpdateInterface()`
  - `ProductionUpdateInterface* getProductionUpdateInterface()`
  - `ProjectileUpdateInterface* getProjectileUpdateInterface()`
  - `RailedTransportDockUpdateInterface* getRailedTransportDockUpdateInterface()`
  - `RebuildHoleBehaviorInterface* getRebuildHoleBehaviorInterface()`
  - `SlavedUpdateInterface* getSlavedUpdateInterface()`
  - `SlowDeathBehaviorInterface* getSlowDeathBehaviorInterface()`
  - `SpawnBehaviorInterface* getSpawnBehaviorInterface()`
  - `SpecialPowerModuleInterface* getSpecialPower()`
  - `SpecialPowerUpdateInterface* getSpecialPowerUpdateInterface()`
  - `TransportPassengerInterface* getTransportPassengerInterface()`
  - `UpdateModuleInterface* getUpdate()`
  - `UpgradeModuleInterface* getUpgrade()`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (1 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
```cpp
// FILE: BehaviorModule.h /////////////////////////////////////////////////////////////////////////////////
// Author: Steven Johnson
// Desc:
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

#include "Common/GameType.h"
#include "Common/Module.h"

//-------------------------------------------------------------------------------------------------
class Team;
class ThingTemplate;

//-------------------------------------------------------------------------------------------------
class BodyModuleInterface;
class CollideModuleInterface;
class ContainModuleInterface;
class CreateModuleInterface;
class DamageModuleInterface;
class DestroyModuleInterface;
class DieModuleInterface;
class SpecialPowerModuleInterface;
class UpdateModuleInterface;
class UpgradeModuleInterface;

//-------------------------------------------------------------------------------------------------
class ParkingPlaceBehaviorInterface;
class RebuildHoleBehaviorInterface;
class BridgeBehaviorInterface;
class BridgeTowerBehaviorInterface;
class BridgeScaffoldBehaviorInterface;
class OverchargeBehaviorInterface;
class TransportPassengerInterface;
class CaveInterface;
class LandMineInterface;

class ProjectileUpdateInterface;
class AIUpdateInterface;
class ExitInterface;
class DockUpdateInterface;
class RailedTransportDockUpdateInterface;
class SpecialPowerUpdateInterface;
class SlavedUpdateInterface;
class SpawnBehaviorInterface;
class CountermeasuresBehaviorInterface;
class SlowDeathBehaviorInterface;
class PowerPlantUpdateInterface;
class ProductionUpdateInterface;
class HordeUpdateInterface;
class SpecialPowerTemplate;
class WeaponTemplate;
class DamageInfo;
class ParticleSystemTemplate;
class StealthUpdate;
class SpyVisionUpdate;


//-------------------------------------------------------------------------------------------------
class BehaviorModuleData : public ModuleData
{
public:
	BehaviorModuleData()
	{
	}

	static void buildFieldParse(MultiIniFieldParse& p)
	{
    ModuleData::buildFieldParse(p);
	}
};

//-------------------------------------------------------------------------------------------------
class BehaviorModuleInterface
{
public:

	virtual BodyModuleInterface* getBody() = 0;
	virtual CollideModuleInterface* getCollide() = 0;
	virtual ContainModuleInterface* getContain() = 0;
	virtual CreateModuleInterface* getCreate() = 0;
	virtual DamageModuleInterface* getDamage() = 0;
	virtual DestroyModuleInterface* getDestroy() = 0;
	virtual DieModuleInterface* getDie() = 0;
	virtual SpecialPowerModuleInterface* getSpecialPower() = 0;
	virtual UpdateModuleInterface* getUpdate() = 0;
	virtual UpgradeModuleInterface* getUpgrade() = 0;

	// interface acquisition
	virtual ParkingPlaceBehaviorInterface* getParkingPlaceBehaviorInterface() = 0;
	virtual RebuildHoleBehaviorInterface* getRebuildHoleBehaviorInterface() = 0;
	virtual BridgeBehaviorInterface* getBridgeBehaviorInterface() = 0;
	virtual BridgeTowerBehaviorInterface* getBridgeTowerBehaviorInterface() = 0;
	virtual BridgeScaffoldBehaviorInterface* getBridgeScaffoldBehaviorInterface() = 0;
	virtual OverchargeBehaviorInterface* getOverchargeBehaviorInterface() = 0;
	virtual TransportPassengerInterface* getTransportPassengerInterface() = 0;
	virtual CaveInterface* getCaveInterface() = 0;
	virtual LandMineInterface* getLandMineInterface() = 0;
	virtual DieModuleInterface* getEjectPilotDieInterface() = 0;
	// move from UpdateModuleInterface (srj)
	virtual ProjectileUpdateInterface* getProjectileUpdateInterface() = 0;
	virtual AIUpdateInterface* getAIUpdateInterface() = 0;
	virtual ExitInterface* getUpdateExitInterface() = 0;
	virtual DockUpdateInterface* getDockUpdateInterface() = 0;
	virtual RailedTransportDockUpdateInterface *getRailedTransportDockUpdateInterface() = 0;
	virtual SlowDeathBehaviorInterface* getSlowDeathBehaviorInterface() = 0;
	virtual SpecialPowerUpdateInterface* getSpecialPowerUpdateInterface() = 0;
	virtual SlavedUpdateInterface* getSlavedUpdateInterface() = 0;
	virtual ProductionUpdateInterface* getProductionUpdateInterface() = 0;
	virtual HordeUpdateInterface* getHordeUpdateInterface() = 0;
	virtual PowerPlantUpdateInterface* getPowerPlantUpdateInterface() = 0;
	virtual SpawnBehaviorInterface* getSpawnBehaviorInterface() = 0;
	virtual CountermeasuresBehaviorInterface* getCountermeasuresBehaviorInterface() = 0;
	virtual const CountermeasuresBehaviorInterface* getCountermeasuresBehaviorInterface() const = 0;

};

//-------------------------------------------------------------------------------------------------
class BehaviorModule : public ObjectModule, public BehaviorModuleInterface
{

	MEMORY_POOL_GLUE_ABC( BehaviorModule )

public:

	BehaviorModule( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype defined by MemoryPoolObject

	static Int getInterfaceMask() { return 0; }
	static ModuleType getModuleType() { return MODULETYPE_BEHAVIOR; }

	virtual BodyModuleInterface* getBody() override { return nullptr; }
	virtual CollideModuleInterface* getCollide() override { return nullptr; }
	virtual ContainModuleInterface* getContain() override { return nullptr; }
	virtual CreateModuleInterface* getCreate() override { return nullptr; }
	virtual DamageModuleInterface* getDamage() override { return nullptr; }
	virtual DestroyModuleInterface* getDestroy() override { return nullptr; }
	virtual DieModuleInterface* getDie() override { return nullptr; }
	virtual SpecialPowerModuleInterface* getSpecialPower() override { return nullptr; }
	virtual UpdateModuleInterface* getUpdate() override { return nullptr; }
	virtual UpgradeModuleInterface* getUpgrade() override { return nullptr; }
  virtual StealthUpdate* getStealth() { return nullptr; }
	virtual SpyVisionUpdate* getSpyVisionUpdate() { return nullptr; }

	virtual ParkingPlaceBehaviorInterface* getParkingPlaceBehaviorInterface() override { return nullptr; }
	virtual RebuildHoleBehaviorInterface* getRebuildHoleBehaviorInterface() override { return nullptr; }
	virtual BridgeBehaviorInterface* getBridgeBehaviorInterface() override { return nullptr; }
	virtual BridgeTowerBehaviorInterface* getBridgeTowerBehaviorInterface() override { return nullptr; }
	virtual BridgeScaffoldBehaviorInterface* getBridgeScaffoldBehaviorInterface() override { return nullptr; }
	virtual OverchargeBehaviorInterface* getOverchargeBehaviorInterface() override { return nullptr; }
	virtual TransportPassengerInterface* getTransportPassengerInterface() override { return nullptr; }
	virtual CaveInterface* getCaveInterface() override { return nullptr; }
	virtual LandMineInterface* getLandMineInterface() override { return nullptr; }
	virtual DieModuleInterface* getEjectPilotDieInterface() override { return nullptr; }
	// interface acquisition (moved from UpdateModule)
	virtual ProjectileUpdateInterface* getProjectileUpdateInterface() override { return nullptr; }
	virtual AIUpdateInterface* getAIUpdateInterface() override { return nullptr; }
	virtual ExitInterface* getUpdateExitInterface() override { return nullptr; }
	virtual DockUpdateInterface* getDockUpdateInterface() override { return nullptr; }
	virtual RailedTransportDockUpdateInterface *getRailedTransportDockUpdateInterface() override { return nullptr; }
	virtual SlowDeathBehaviorInterface* getSlowDeathBehaviorInterface() override { return nullptr; }
	virtual SpecialPowerUpdateInterface* getSpecialPowerUpdateInterface() override { return nullptr; }
	virtual SlavedUpdateInterface* getSlavedUpdateInterface() override { return nullptr; }
	virtual ProductionUpdateInterface* getProductionUpdateInterface() override { return nullptr; }
	virtual HordeUpdateInterface* getHordeUpdateInterface() override { return nullptr; }
	virtual PowerPlantUpdateInterface* getPowerPlantUpdateInterface() override { return nullptr; }
	virtual SpawnBehaviorInterface* getSpawnBehaviorInterface() override { return nullptr; }
	virtual CountermeasuresBehaviorInterface* getCountermeasuresBehaviorInterface() override { return nullptr; }
	virtual const CountermeasuresBehaviorInterface* getCountermeasuresBehaviorInterface() const override { return nullptr; }

protected:

	// snapshot methods
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

};
inline BehaviorModule::BehaviorModule( Thing *thing, const ModuleData* moduleData ) : ObjectModule( thing, moduleData ) { }
inline BehaviorModule::~BehaviorModule() { }


enum RunwayReservationType CPP_11(: Int)
{
	RESERVATION_TAKEOFF,
	RESERVATION_LANDING,
};

enum
{
	InvalidRunway = -1,
};

//-------------------------------------------------------------------------------------------------
class ParkingPlaceBehaviorInterface
{
public:
	struct PPInfo
	{
		Coord3D		parkingSpace;
		Real			parkingOrientation;
		Coord3D		runwayPrep;
		Coord3D		runwayStart;
		Coord3D		runwayEnd;
		Coord3D		runwayExit;
		Coord3D	  runwayLandingStart;
		Coord3D	  runwayLandingEnd;
		Coord3D		runwayApproach;
		Coord3D		hangarInternal;
		Real			runwayTakeoffDist;
		Real			hangarInternalOrient;
	};
	virtual Bool shouldReserveDoorWhenQueued(const ThingTemplate* thing) const = 0;
	virtual Bool hasAvailableSpaceFor(const ThingTemplate* thing) const = 0;
	virtual Bool hasReservedSpace(ObjectID id) const = 0;
	virtual Int  getSpaceIndex( ObjectID id ) const = 0;
	virtual Bool reserveSpace(ObjectID id, Real parkingOffset, PPInfo* info) = 0;
	virtual void releaseSpace(ObjectID id) = 0;
	virtual Bool reserveRunway(ObjectID id, Bool forLanding) = 0;
	virtual void calcPPInfo( ObjectID id, PPInfo *info ) = 0;
	virtual void releaseRunway(ObjectID id) = 0;
	virtual Int getRunwayIndex(ObjectID id) = 0;
	virtual Int getRunwayCount() const = 0;
	virtual ObjectID getRunwayReservation( Int r, RunwayReservationType type = RESERVATION_TAKEOFF ) = 0;
	virtual void transferRunwayReservationToNextInLineForTakeoff(ObjectID id) = 0;
	virtual Real getApproachHeight() const = 0;
	virtual Real getLandingDeckHeightOffset() const = 0;
	virtual void setHealee(Object* healee, Bool add) = 0;
	virtual void killAllParkedUnits() = 0;
	virtual void defectAllParkedUnits(Team* newTeam, UnsignedInt detectionTime) = 0;
	virtual Bool calcBestParkingAssignment( ObjectID id, Coord3D *pos, Int *oldIndex = nullptr, Int *newIndex = nullptr ) = 0;

	virtual const std::vector<Coord3D>* getTaxiLocations( ObjectID id ) const = 0;
	virtual const std::vector<Coord3D>* getCreationLocations( ObjectID id ) const = 0;
};

//-------------------------------------------------------------------------------------------------
class TransportPassengerInterface
{
public:
	virtual Bool tryToEvacuate( Bool exposeStealthedUnits ) = 0; ///< Will try to kick everybody out with game checks, and will return whether anyone made it
};

//-------------------------------------------------------------------------------------------------
class CaveInterface
{
public:
	virtual void tryToSetCaveIndex( Int newIndex ) = 0;	///< Called by script as an alternative to instancing separate objects.  'Try', because can fail.
	virtual void setOriginalTeam( Team *oldTeam ) = 0;	///< This is a distributed Garrison in terms of capturing, so when one node triggers the change, he needs to tell everyone, so anyone can do the un-change.
};

//-------------------------------------------------------------------------------------------------
class LandMineInterface
{
public:
	virtual void setScootParms(const Coord3D& start, const Coord3D& end) = 0;
	virtual void disarm() = 0;
};

//-------------------------------------------------------------------------------------------------
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Behavior/BehaviorModule.cpp`
```cpp
// FILE: BehaviorModule.cpp ///////////////////////////////////////////////////////////////////////
// Author: Colin Day, September 2002
// Desc:   Implementation for anything in the base BehaviorModule
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"
#include "Common/Xfer.h"
#include "GameLogic/Module/BehaviorModule.h"

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void BehaviorModule::crc( Xfer *xfer )
{

	// call base class
	ObjectModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer Method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void BehaviorModule::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// call base class
	ObjectModule::xfer( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void BehaviorModule::loadPostProcess()
{

	// call base class
	ObjectModule::loadPostProcess();

}
```
