# Translation Packet `PKT-286`: `ParkingPlaceBehavior`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-286`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/behavior/parking_place_behavior.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (18):**
  - `Common/CRCDebug.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/CRCDebug.h`
  - `Common/Team.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Team.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameLogic/AI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/AI.h`
  - `GameLogic/AIPathfind.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/AIPathfind.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/DieModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DieModule.h`
  - `GameLogic/Module/JetAIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/JetAIUpdate.h`
  - `GameLogic/Module/ParkingPlaceBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ParkingPlaceBehavior.h`
  - `GameLogic/Module/ProductionUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ProductionUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/TerrainLogic.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/TerrainLogic.h`
  - `PreRTS.h`
- **Implemented Module Facets (3):**
  - `DieModuleInterface* getDie()`
  - `ExitInterface* getUpdateExitInterface()`
  - `ParkingPlaceBehaviorInterface* getParkingPlaceBehaviorInterface()`
- **INI `FieldParse` Table Entries (9):**
  - `"NumRows" => INI::parseInt`
  - `"NumCols" => INI::parseInt`
  - `"ApproachHeight" => INI::parseReal`
  - `"LandingDeckHeightOffset" => INI::parseReal`
  - `"HasRunways" => INI::parseBool`
  - `"ParkInHangars" => INI::parseBool`
  - `"HealAmountPerSecond" => INI::parseReal`
  - `"ExtraHealAmount4Helicopters" => INI::parseReal`
  - `"TimeForFullHeal" => INI::parseDurationUnsignedInt`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (21 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUnsignedByte( &spacesCount )`
  3. `xferObjectID( &((*it).m_objectInSpace) )`
  4. `xferBool( &((*it).m_reservedForExit) )`
  5. `xferObjectID( &objectID )`
  6. `xferBool( &reservedForExit )`
  7. `xferUnsignedByte( &runwaysCount )`
  8. `xferObjectID( &((*it).m_inUseBy) )`
  9. `xferObjectID( &((*it).m_nextInLineForTakeoff) )`
  10. `xferBool( &((*it).m_wasInLine) )`
  11. `xferObjectID( &inUseBy )`
  12. `xferObjectID( &nextInLineForTakeoff )`
  13. `xferBool( &wasInLine )`
  14. `xferUnsignedByte( &healCount )`
  15. `xferObjectID( &((*it).m_gettingHealedID) )`
  16. `xferUnsignedInt( &((*it).m_healStartFrame) )`
  17. `xferObjectID( &info.m_gettingHealedID )`
  18. `xferUnsignedInt( &info.m_healStartFrame )`
  19. `xferCoord3D(&m_heliRallyPoint)`
  20. `xferBool(&m_heliRallyPointExists)`
  21. `xferUnsignedInt(&m_nextHealFrame)`
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if !RETAIL_COMPATIBLE_CRC`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ParkingPlaceBehavior.h`
```cpp
// FILE: ParkingPlaceBehavior.h /////////////////////////////////////////////////////////////////////////
// Author: Steven Johnson, June 2002
// Desc:
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/DieModule.h"
#include "GameLogic/Module/UpdateModule.h"

//-------------------------------------------------------------------------------------------------
class ParkingPlaceBehaviorModuleData : public UpdateModuleData
{
public:
	//UnsignedInt		m_framesForFullHeal;
	Real					m_healAmount;
//	Real					m_extraHealAmount4Helicopters;
	Int						m_numRows;
	Int						m_numCols;
	Real					m_approachHeight;
	Real					m_landingDeckHeightOffset;
	Bool					m_hasRunways;			// if true, each col has a runway in front of it
	Bool					m_parkInHangars;	// if true, park at the hangar production spot, not the "real" parking place

	ParkingPlaceBehaviorModuleData()
	{
		//m_framesForFullHeal = 0;
		m_healAmount = 0;
//    m_extraHealAmount4Helicopters = 0;
		m_numRows = 0;
		m_numCols = 0;
		m_approachHeight = 0.0f;
		m_landingDeckHeightOffset = 0.0f;
		m_hasRunways = false;
		m_parkInHangars = false;
	}

	static void buildFieldParse(MultiIniFieldParse& p)
	{
		UpdateModuleData::buildFieldParse(p);

		static const FieldParse dataFieldParse[] =
		{
			{ "NumRows",						     INI::parseInt,	 nullptr, offsetof( ParkingPlaceBehaviorModuleData, m_numRows ) },
			{ "NumCols",						     INI::parseInt,	 nullptr, offsetof( ParkingPlaceBehaviorModuleData, m_numCols ) },
			{ "ApproachHeight",			     INI::parseReal, nullptr, offsetof( ParkingPlaceBehaviorModuleData, m_approachHeight ) },
			{ "LandingDeckHeightOffset", INI::parseReal, nullptr, offsetof( ParkingPlaceBehaviorModuleData, m_landingDeckHeightOffset ) },
			{ "HasRunways",					     INI::parseBool, nullptr, offsetof( ParkingPlaceBehaviorModuleData, m_hasRunways ) },
			{ "ParkInHangars",			     INI::parseBool, nullptr, offsetof( ParkingPlaceBehaviorModuleData, m_parkInHangars ) },
			{ "HealAmountPerSecond",     INI::parseReal, nullptr, offsetof( ParkingPlaceBehaviorModuleData, m_healAmount ) },
//			{ "ExtraHealAmount4Helicopters",  INI::parseReal, nullptr, offsetof( ParkingPlaceBehaviorModuleData, m_extraHealAmount4Helicopters ) },



			//{ "TimeForFullHeal",	INI::parseDurationUnsignedInt,	nullptr, offsetof( ParkingPlaceBehaviorModuleData, m_framesForFullHeal ) },
			{ nullptr, nullptr, nullptr, 0 }
		};
		p.add(dataFieldParse);
	}

private:

};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class ParkingPlaceBehavior : public UpdateModule,
														 public DieModuleInterface,
														 public ParkingPlaceBehaviorInterface,
														 public ExitInterface
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( ParkingPlaceBehavior, "ParkingPlaceBehavior" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( ParkingPlaceBehavior, ParkingPlaceBehaviorModuleData )

public:

	ParkingPlaceBehavior( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	static Int getInterfaceMask() { return UpdateModule::getInterfaceMask() | (MODULEINTERFACE_DIE); }

	// BehaviorModule
	virtual DieModuleInterface *getDie() override { return this; }
	virtual ParkingPlaceBehaviorInterface* getParkingPlaceBehaviorInterface() override { return this; }
	virtual ExitInterface* getUpdateExitInterface() override { return this; }

	// ExitInterface
	virtual Bool isExitBusy() const override {return FALSE;}	///< Contain style exiters are getting the ability to space out exits, so ask this before reserveDoor as a kind of no-commitment check.
	virtual ExitDoorType reserveDoorForExit( const ThingTemplate* objType, Object *specificObject ) override;
	virtual void exitObjectViaDoor( Object *newObj, ExitDoorType exitDoor ) override;
	virtual void unreserveDoorForExit( ExitDoorType exitDoor ) override;
	virtual void exitObjectByBudding( Object *newObj, Object *budHost ) override { return; }

	virtual Bool getExitPosition( Coord3D& rallyPoint ) const override;
	virtual Bool getNaturalRallyPoint( Coord3D& rallyPoint, Bool offset = TRUE ) const override;
	virtual void setRallyPoint( const Coord3D *pos ) override;			///< define a "rally point" for units to move towards
	virtual const Coord3D *getRallyPoint() const override;			///< define a "rally point" for units to move towards

	// UpdateModule
	virtual UpdateSleepTime update() override;

	// DieModule
	virtual void onDie( const DamageInfo *damageInfo ) override;

	// ParkingPlaceBehaviorInterface
	virtual Bool shouldReserveDoorWhenQueued(const ThingTemplate* thing) const override;
	virtual Bool hasAvailableSpaceFor(const ThingTemplate* thing) const override;
	virtual Bool hasReservedSpace(ObjectID id) const override;
	virtual Int  getSpaceIndex( ObjectID id ) const override;
	virtual Bool reserveSpace(ObjectID id, Real parkingOffset, PPInfo* info) override;
	virtual void releaseSpace(ObjectID id) override;
	virtual Bool reserveRunway(ObjectID id, Bool forLanding) override;
	virtual void releaseRunway(ObjectID id) override;
	virtual void calcPPInfo( ObjectID id, PPInfo *info ) override;
	virtual Int getRunwayIndex(ObjectID id) override;
	virtual Int getRunwayCount() const override { return m_runways.size(); }
	virtual ObjectID getRunwayReservation( Int r, RunwayReservationType type ) override;
	virtual void transferRunwayReservationToNextInLineForTakeoff(ObjectID id) override;
	virtual Real getApproachHeight() const override { return getParkingPlaceBehaviorModuleData()->m_approachHeight; }
	virtual Real getLandingDeckHeightOffset() const override { return getParkingPlaceBehaviorModuleData()->m_landingDeckHeightOffset; }
	virtual void setHealee(Object* healee, Bool add) override;
	virtual void killAllParkedUnits() override;
	virtual void defectAllParkedUnits(Team* newTeam, UnsignedInt detectionTime) override;
	virtual Bool calcBestParkingAssignment( ObjectID id, Coord3D *pos, Int *oldIndex = nullptr, Int *newIndex = nullptr ) override { return FALSE; }
	virtual const std::vector<Coord3D>* getTaxiLocations( ObjectID id ) const override { return nullptr; }
	virtual const std::vector<Coord3D>* getCreationLocations( ObjectID id ) const override { return nullptr; }

private:

	struct ParkingPlaceInfo
	{
		Coord3D      m_hangarStart;
		Real         m_hangarStartOrient;
		Coord3D      m_location;
		Coord3D      m_prep;
		Real         m_orientation;
		Int          m_runway;
		ExitDoorType m_door;
		ObjectID     m_objectInSpace;
		Bool         m_reservedForExit;
		Bool         m_postponedRunwayReservationForTakeoff;

		ParkingPlaceInfo()
		{
			m_hangarStart.zero();
			m_hangarStartOrient = 0;
			m_location.zero();
			m_prep.zero();
			m_orientation = 0;
			m_runway = 0;
			m_door = DOOR_NONE_AVAILABLE;
			m_objectInSpace = INVALID_ID;
			m_reservedForExit = false;
			m_postponedRunwayReservationForTakeoff = false;
		}
	};

	struct RunwayInfo
	{
		Coord3D		m_start;
		Coord3D		m_end;
		ObjectID	m_inUseBy;
		ObjectID	m_nextInLineForTakeoff;
		Bool			m_wasInLine;
	};

	struct HealingInfo
	{
		ObjectID		m_gettingHealedID;
		UnsignedInt	m_healStartFrame;
	};

	std::vector<ParkingPlaceInfo>	m_spaces;
	std::vector<RunwayInfo>				m_runways;
	std::list<HealingInfo>				m_healing;	// note, this list can vary in size, and be larger than the parking space count
	UnsignedInt										m_nextHealFrame;
	Bool													m_gotInfo;

	Bool postponeRunwayReservation(UnsignedInt spaceIndex, Bool forLanding);
	void buildInfo();
	void purgeDead();
	void resetWakeFrame();

	ParkingPlaceInfo* findPPI(ObjectID id);
	ParkingPlaceInfo* findEmptyPPI();

	Coord3D m_heliRallyPoint;
	Bool m_heliRallyPointExists;				///< Only move to the rally point if this is true
};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Behavior/ParkingPlaceBehavior.cpp`
```cpp
// FILE: ParkingPlaceBehavior.cpp ///////////////////////////////////////////////////////////////////////
// Author:	Steven Johnson, June 2002
// Desc:
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine
#include "Common/CRCDebug.h"
#include "Common/Xfer.h"
#include "Common/ThingTemplate.h"
#include "GameClient/Drawable.h"
#include "GameLogic/AI.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/ParkingPlaceBehavior.h"
#include "GameLogic/Module/ProductionUpdate.h"
#include "GameLogic/Module/JetAIUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/TerrainLogic.h"
#include "Common/Team.h"



//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ParkingPlaceBehavior::ParkingPlaceBehavior( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{
	m_gotInfo = false;
	m_heliRallyPoint.zero();
	m_heliRallyPointExists = FALSE;
	m_nextHealFrame = FOREVER;
	setWakeFrame(getObject(), UPDATE_SLEEP_NONE);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ParkingPlaceBehavior::~ParkingPlaceBehavior()
{
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void ParkingPlaceBehavior::buildInfo()
{
	if (m_gotInfo)
		return;

	if (getObject()->testStatus(OBJECT_STATUS_UNDER_CONSTRUCTION) ||
			getObject()->testStatus(OBJECT_STATUS_SOLD))
		return;

	const ParkingPlaceBehaviorModuleData* d = getParkingPlaceBehaviorModuleData();
	ProductionUpdateInterface* pu = getObject()->getProductionUpdateInterface();

	m_spaces.reserve(d->m_numRows * d->m_numCols);

	Int door = 0;
	{
		ParkingPlaceInfo info;
		for (Int row = 0; row < d->m_numRows; ++row)
		{
			for (Int col = 0; col < d->m_numCols; ++col)
			{
				AsciiString tmp;
				Matrix3D mtx;

				tmp.format("Runway%dPark%dHan",col+1,row+1);
				getObject()->getSingleLogicalBonePosition(tmp.str(), &info.m_hangarStart, &mtx);
				info.m_hangarStartOrient = mtx.Get_Z_Rotation();

				tmp.format("Runway%dParking%d",col+1,row+1);
				getObject()->getSingleLogicalBonePosition(tmp.str(), &info.m_location, &mtx);
				info.m_orientation = mtx.Get_Z_Rotation();

				tmp.format("Runway%dPrep%d",col+1,row+1);
				getObject()->getSingleLogicalBonePosition(tmp.str(), &info.m_prep, nullptr);

				info.m_runway = col;
				info.m_door = (ExitDoorType)door++;
				info.m_objectInSpace = INVALID_ID;
				info.m_reservedForExit = false;
				if (pu)
					pu->setHoldDoorOpen(info.m_door, false);

				m_spaces.push_back(info);
			}
		}
	}

	if (d->m_hasRunways)
	{
		RunwayInfo info;
		m_runways.reserve(d->m_numCols);
		for (Int col = 0; col < d->m_numCols; ++col)
		{
			AsciiString tmp;

			tmp.format("RunwayStart%d",col+1);
			getObject()->getSingleLogicalBonePosition(tmp.str(), &info.m_start, nullptr);

			tmp.format("RunwayEnd%d",col+1);
			getObject()->getSingleLogicalBonePosition(tmp.str(), &info.m_end, nullptr);

			info.m_inUseBy = INVALID_ID;
			info.m_nextInLineForTakeoff = INVALID_ID;
			info.m_wasInLine = false;
			m_runways.push_back(info);
		}
	}

	m_gotInfo = true;
}

//-------------------------------------------------------------------------------------------------
void ParkingPlaceBehavior::purgeDead()
{
	buildInfo();

	ProductionUpdateInterface* pu = getObject()->getProductionUpdateInterface();
	{
		for (std::vector<ParkingPlaceInfo>::iterator it = m_spaces.begin(); it != m_spaces.end(); ++it)
		{
			if (it->m_objectInSpace != INVALID_ID)
			{
				Object* obj = TheGameLogic->findObjectByID(it->m_objectInSpace);
				if (obj == nullptr || obj->isEffectivelyDead())
				{
					it->m_objectInSpace = INVALID_ID;
					it->m_reservedForExit = false;
					it->m_postponedRunwayReservationForTakeoff = false;
					if (pu)
						pu->setHoldDoorOpen(it->m_door, false);
				}
			}
		}
	}

	{
		for (std::vector<RunwayInfo>::iterator it = m_runways.begin(); it != m_runways.end(); ++it)
		{
			if (it->m_inUseBy != INVALID_ID)
			{
				Object* obj = TheGameLogic->findObjectByID(it->m_inUseBy);
				if (obj == nullptr || obj->isEffectivelyDead())
				{
					it->m_inUseBy = INVALID_ID;
					it->m_wasInLine = false;
				}
			}
			if (it->m_nextInLineForTakeoff != INVALID_ID)
			{
				Object* obj = TheGameLogic->findObjectByID(it->m_nextInLineForTakeoff);
				if (obj == nullptr || obj->isEffectivelyDead())
				{
					it->m_nextInLineForTakeoff = INVALID_ID;
				}
			}
		}
	}

	{
		Bool anythingPurged = false;
		for (std::list<HealingInfo>::iterator it = m_healing.begin(); it != m_healing.end(); /*++it*/)
		{
			if (it->m_gettingHealedID != INVALID_ID)
			{
				Object* objToHeal = TheGameLogic->findObjectByID(it->m_gettingHealedID);
				if (objToHeal == nullptr || objToHeal->isEffectivelyDead())
				{
					it = m_healing.erase(it);
					anythingPurged = true;
				}
				else
				{
					++it;
				}
			}
		}
		if (anythingPurged)
			resetWakeFrame();
	}
}

//-------------------------------------------------------------------------------------------------
// note: called from client, so MUST NOT modify self in any way, or desyncs will occur
Bool ParkingPlaceBehavior::hasReservedSpace(ObjectID id) const
{
	if (!m_gotInfo)
		return false;

	if (id == INVALID_ID)	// shouldn't call this way, but Weapon mistakenly does sometimes, so check for it
		return false;

	for (std::vector<ParkingPlaceInfo>::const_iterator it = m_spaces.begin(); it != m_spaces.end(); ++it)
	{
		if (it->m_objectInSpace == id)
			return true;
	}
	return false;
}

//-------------------------------------------------------------------------------------------------
Int ParkingPlaceBehavior::getSpaceIndex( ObjectID id ) const
{
	if( id == INVALID_ID )
	{
		return -1;
	}

	Int index = 0;
	for( std::vector<ParkingPlaceInfo>::const_iterator it = m_spaces.begin(); it != m_spaces.end(); it++, index++ )
	{
		if (it->m_objectInSpace == id)
		{
			return index;
		}
	}
	return -1;
}

//-------------------------------------------------------------------------------------------------
ParkingPlaceBehavior::ParkingPlaceInfo* ParkingPlaceBehavior::findPPI(ObjectID id)
{
	DEBUG_ASSERTCRASH(id != INVALID_ID, ("call findEmptyPPI instead"));

	if (!m_gotInfo || id == INVALID_ID)
		return nullptr;

	for (std::vector<ParkingPlaceInfo>::iterator it = m_spaces.begin(); it != m_spaces.end(); ++it)
	{
		if (it->m_objectInSpace == id)
			return &(*it);
	}

	return nullptr;
}

//-------------------------------------------------------------------------------------------------
ParkingPlaceBehavior::ParkingPlaceInfo* ParkingPlaceBehavior::findEmptyPPI()
{
	if (!m_gotInfo)
		return nullptr;

	for (std::vector<ParkingPlaceInfo>::iterator it = m_spaces.begin(); it != m_spaces.end(); ++it)
	{
		if (it->m_objectInSpace == INVALID_ID && it->m_reservedForExit == false)
			return &(*it);
	}

	return nullptr;
}

//-------------------------------------------------------------------------------------------------
// note: called from client, so MUST NOT modify self in any way, or desyncs will occur
Bool ParkingPlaceBehavior::shouldReserveDoorWhenQueued(const ThingTemplate* thing) const
{
	if (thing->isKindOf(KINDOF_PRODUCED_AT_HELIPAD))
		return false;

	return true;
}

//-------------------------------------------------------------------------------------------------
// note: called from client, so MUST NOT modify self in any way, or desyncs will occur
Bool ParkingPlaceBehavior::hasAvailableSpaceFor(const ThingTemplate* thing) const
{
	if (!m_gotInfo)	// degenerate case, shouldn't happen, but just in case...
		return false;

	if (thing->isKindOf(KINDOF_PRODUCED_AT_HELIPAD))
		return true;

	for (std::vector<ParkingPlaceInfo>::const_iterator it = m_spaces.begin(); it != m_spaces.end(); ++it)
	{
		ObjectID id = it->m_objectInSpace;

		// since this is const, and we can't purge the dead safely, just peek and see if we have a dead thing.
		if (id != INVALID_ID)
		{
			Object* obj = TheGameLogic->findObjectByID(id);
			if (obj == nullptr || obj->isEffectivelyDead())
			{
				id = INVALID_ID;
			}
		}

		if (id == INVALID_ID && it->m_reservedForExit == false)
		{
			return true;
		}
	}

	return false;
}

//-------------------------------------------------------------------------------------------------
Bool ParkingPlaceBehavior::reserveSpace(ObjectID id, Real parkingOffset, ParkingPlaceBehaviorInterface::PPInfo* info)
{
	buildInfo();
	purgeDead();

	const ParkingPlaceBehaviorModuleData* d = getParkingPlaceBehaviorModuleData();

	ParkingPlaceInfo* ppi = findPPI(id);
	if (ppi == nullptr)
	{
		ppi = findEmptyPPI();
		if (ppi == nullptr)
		{
			DEBUG_CRASH(("No parking places!"));
			return false;	// nothing available
		}
	}

	ppi->m_objectInSpace = id;
	ppi->m_reservedForExit = false;

	if( d->m_landingDeckHeightOffset )
	{
		Object *obj = TheGameLogic->findObjectByID( id );
		if( obj )
		{
			obj->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_DECK_HEIGHT_OFFSET ) );
		}
	}

	if (info)
	{
		calcPPInfo( id, info );
		if (parkingOffset != 0.0f)
		{
			info->parkingSpace.x += parkingOffset * Cos(ppi->m_orientation);
			info->parkingSpace.y += parkingOffset * Sin(ppi->m_orientation);
		}
	}

	ProductionUpdateInterface* pu = getObject()->getProductionUpdateInterface();
	if (pu)
		pu->setHoldDoorOpen(ppi->m_door, true);

	return true;
}

//-------------------------------------------------------------------------------------------------
void ParkingPlaceBehavior::calcPPInfo( ObjectID id, PPInfo *info )
{

	ParkingPlaceInfo* ppi = findPPI( id );
	if( !ppi )
	{
		//Utter failure.
		return;
	}
	const RunwayInfo& rr = m_runways[ ppi->m_runway ];

	if( info )
	{
		const ParkingPlaceBehaviorModuleData* d = getParkingPlaceBehaviorModuleData();
		info->parkingSpace = d->m_parkInHangars ? ppi->m_hangarStart : ppi->m_location;
		info->runwayPrep = ppi->m_prep;
		info->parkingOrientation = d->m_parkInHangars ? ppi->m_hangarStartOrient : ppi->m_orientation;
		info->runwayStart = rr.m_start;
		info->runwayEnd = rr.m_end;
		info->runwayApproach = rr.m_end;
		//const Real APPROACH_DIST = 0.5f;	// no, too short, planes may not have enough space to drop altitude
		const Real APPROACH_DIST = 0.75f;
		info->runwayApproach.x += (rr.m_end.x - rr.m_start.x) * APPROACH_DIST;
		info->runwayApproach.y += (rr.m_end.y - rr.m_start.y) * APPROACH_DIST;
		info->runwayApproach.z = rr.m_end.z + d->m_approachHeight + d->m_landingDeckHeightOffset;
		info->runwayExit = info->runwayApproach;
		info->hangarInternal = ppi->m_hangarStart;
		info->hangarInternalOrient = ppi->m_hangarStartOrient;

		//Cache the runway's takeoff distance used by JetAIUpdate for calculating lift.
		Coord3D vector = info->runwayStart;
		vector.sub( info->runwayEnd );
		info->runwayTakeoffDist = vector.length();

		for (std::vector<RunwayInfo>::iterator it = m_runways.begin(); it != m_runways.end(); ++it)
		{
			if (it->m_inUseBy == id)
			{
				if (it->m_wasInLine)
				{
					info->runwayStart = info->runwayPrep;
				}
				break;
			}
		}
	}
}

//-------------------------------------------------------------------------------------------------
void ParkingPlaceBehavior::releaseSpace(ObjectID id)
{
	buildInfo();
	purgeDead();

	ProductionUpdateInterface* pu = getObject()->getProductionUpdateInterface();
	for (std::vector<ParkingPlaceInfo>::iterator it = m_spaces.begin(); it != m_spaces.end(); ++it)
	{
		if (it->m_objectInSpace == id)
		{
			it->m_objectInSpace = INVALID_ID;
			it->m_reservedForExit = false;
			it->m_postponedRunwayReservationForTakeoff = false;
			if (pu)
				pu->setHoldDoorOpen(it->m_door, false);
			break;
		}
	}

	Object *obj = TheGameLogic->findObjectByID( id );
	if( obj )
	{
		obj->clearStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_DECK_HEIGHT_OFFSET ) );
	}

}

//-------------------------------------------------------------------------------------------------
Int ParkingPlaceBehavior::getRunwayIndex(ObjectID id)
{
	purgeDead();

	for (std::vector<ParkingPlaceInfo>::iterator it = m_spaces.begin(); it != m_spaces.end(); ++it)
	{
		if (it->m_objectInSpace == id)
		{
			return it->m_runway;
		}
	}
	return InvalidRunway;
}

//-------------------------------------------------------------------------------------------------
ObjectID ParkingPlaceBehavior::getRunwayReservation( Int runway, RunwayReservationType type )
{
	//Note: We don't care about type because these runways share the runway for taking off and landing.
	buildInfo();
	purgeDead();
	return m_runways[runway].m_inUseBy;
}

//-------------------------------------------------------------------------------------------------
void ParkingPlaceBehavior::transferRunwayReservationToNextInLineForTakeoff(ObjectID id)
{
	buildInfo();
	purgeDead();
	for (std::vector<RunwayInfo>::iterator it = m_runways.begin(); it != m_runways.end(); ++it)
	{
		if (it->m_inUseBy == id)
		{
			if (it->m_nextInLineForTakeoff != INVALID_ID)
			{
				it->m_inUseBy = it->m_nextInLineForTakeoff;
				it->m_wasInLine = true;
				it->m_nextInLineForTakeoff = INVALID_ID;
			}
			break;
		}
	}
}

//-------------------------------------------------------------------------------------------------
Bool ParkingPlaceBehavior::postponeRunwayReservation(UnsignedInt spaceIndex, Bool forLanding)
{
	// TheSuperHackers @tweak Block the first attempt to reserve a runway for 'upper' space indices.
	// This allows 'lower' space indices to reserve a runway first to ensure deterministic takeoff ordering.

	if (m_spaces.size() > m_runways.size() && spaceIndex >= m_runways.size())
	{
		Bool& postponed = m_spaces[spaceIndex].m_postponedRunwayReservationForTakeoff;
		if (forLanding)
		{
			postponed = false;
		}
		else if (!postponed)
		{
			postponed = true;
			return true;
		}
	}

	return false;
}

//-------------------------------------------------------------------------------------------------
Bool ParkingPlaceBehavior::reserveRunway(ObjectID id, Bool forLanding)
{
	buildInfo();
	purgeDead();

	Int runway = -1;
	for (UnsignedInt i = 0; i < m_spaces.size(); ++i)
	{
		if (m_spaces[i].m_objectInSpace == id)
		{
#if !RETAIL_COMPATIBLE_CRC
			if (postponeRunwayReservation(i, forLanding))
				return false;
#endif

			runway = m_spaces[i].m_runway;
			break;
		}
	}

	if (runway == -1)
	{
		DEBUG_CRASH(("only planes with reserved spaces can reserve runways"));
		return false;
	}

	RunwayInfo& info = m_runways[runway];
	if (info.m_inUseBy == id)
	{
		return true;
	}
	else if (info.m_inUseBy == INVALID_ID)
	{
		info.m_inUseBy = id;

		if (info.m_nextInLineForTakeoff == id)
		{
			info.m_nextInLineForTakeoff = INVALID_ID;
			info.m_wasInLine = true;
		}
		else
		{
			info.m_wasInLine = false;
		}

		return true;
	}
	else if (!forLanding && info.m_nextInLineForTakeoff == INVALID_ID)
	{
		info.m_nextInLineForTakeoff = id;
		return false;	// yes, that's right
	}

	return false;
}

//-------------------------------------------------------------------------------------------------
void ParkingPlaceBehavior::releaseRunway(ObjectID id)
{
	buildInfo();
	purgeDead();

	for (std::vector<RunwayInfo>::iterator it = m_runways.begin(); it != m_runways.end(); ++it)
	{
		if (it->m_inUseBy == id)
		{
			it->m_inUseBy = INVALID_ID;
			it->m_wasInLine = false;
		}
		if (it->m_nextInLineForTakeoff == id)
		{
			it->m_nextInLineForTakeoff = INVALID_ID;
		}
	}
}

//-------------------------------------------------------------------------------------------------
// don't really need to autoheal every frame....
const Int HEAL_RATE_FRAMES = LOGICFRAMES_PER_SECOND / 5;

//-------------------------------------------------------------------------------------------------
void ParkingPlaceBehavior::resetWakeFrame()
{
	if (m_healing.empty())
	{
		m_nextHealFrame = FOREVER;
	}
	else
	{
		m_nextHealFrame = TheGameLogic->getFrame() + HEAL_RATE_FRAMES;
	}
}

//-------------------------------------------------------------------------------------------------
void ParkingPlaceBehavior::setHealee(Object* healee, Bool add)
{
	if (add)
	{
		for (std::list<HealingInfo>::const_iterator it = m_healing.begin(); it != m_healing.end(); ++it)
		{
			if (it->m_gettingHealedID == healee->getID())
				return;
		}
		HealingInfo info;
		info.m_gettingHealedID = healee->getID();
		info.m_healStartFrame = TheGameLogic->getFrame();
		m_healing.push_back(info);
		resetWakeFrame();
	}
	else
	{
		for (std::list<HealingInfo>::iterator it = m_healing.begin(); it != m_healing.end(); /*++it*/)
		{
			if (it->m_gettingHealedID == healee->getID())
			{
				it = m_healing.erase(it);
				resetWakeFrame();
			}
			else
			{
				++it;
			}
		}
	}
}

//-------------------------------------------------------------------------------------------------
void ParkingPlaceBehavior::defectAllParkedUnits(Team* newTeam, UnsignedInt detectionTime)
{
	buildInfo();
	purgeDead();

	for (std::vector<ParkingPlaceInfo>::iterator it = m_spaces.begin(); it != m_spaces.end(); ++it)
	{
		if (it->m_objectInSpace != INVALID_ID)
		{
			Object* obj = TheGameLogic->findObjectByID(it->m_objectInSpace);
			if (obj == nullptr || obj->isEffectivelyDead())
				continue;

			// srj sez: evil. fix better someday.
			static NameKeyType jetKey = TheNameKeyGenerator->nameToKey("JetAIUpdate");
			JetAIUpdate* ju = (JetAIUpdate *)obj->findUpdateModule(jetKey);
			Bool takeoffOrLanding = ju ? ju->friend_isTakeoffOrLandingInProgress() : false;

			if (obj->isAboveTerrain() && !takeoffOrLanding)
			{
				// if the new team is a different controlling player, this guys loses his space.
				if (newTeam->getControllingPlayer() != obj->getControllingPlayer())
				{
					releaseSpace(obj->getID());
					if (obj->getProducerID() == getObject()->getID())
						obj->setProducer(nullptr);
				}
			}
			else
			{
				obj->defect(newTeam, detectionTime);
			}
		}
	}

	purgeDead();
}

//-------------------------------------------------------------------------------------------------
void ParkingPlaceBehavior::killAllParkedUnits()
{
	buildInfo();
	purgeDead();

	for (std::vector<ParkingPlaceInfo>::iterator it = m_spaces.begin(); it != m_spaces.end(); ++it)
	{
		if (it->m_objectInSpace != INVALID_ID)
		{
			Object* obj = TheGameLogic->findObjectByID(it->m_objectInSpace);
			if (obj == nullptr || obj->isEffectivelyDead())
				continue;

			// srj sez: evil. fix better someday.
			static NameKeyType jetKey = TheNameKeyGenerator->nameToKey("JetAIUpdate");
			JetAIUpdate* ju = (JetAIUpdate *)obj->findUpdateModule(jetKey);
			Bool takeoffOrLanding = ju ? ju->friend_isTakeoffOrLandingInProgress() : false;

			if (obj->isAboveTerrain() && !takeoffOrLanding)
				continue;

			obj->kill();
		}
	}

	purgeDead();
}

//-------------------------------------------------------------------------------------------------
void ParkingPlaceBehavior::onDie( const DamageInfo *damageInfo )
{
	killAllParkedUnits();
}

//-------------------------------------------------------------------------------------------------
UpdateSleepTime ParkingPlaceBehavior::update()
{
	// alas, we need to keep the buildInfo and dead-purged stuff pretty much up to date, for
	// the client to be able to peek at. at this late date, the most expedient way is to ensure
	// our update is run every frame, and do this manually. the extra cost should be trivial, since
	// there are generally at most only a few airfields at any given time.
	buildInfo();
	purgeDead();

	UnsignedInt now = TheGameLogic->getFrame();
	if (now >= m_nextHealFrame)
	{
		m_nextHealFrame = now + HEAL_RATE_FRAMES;
		const ParkingPlaceBehaviorModuleData* d = getParkingPlaceBehaviorModuleData();
		for (std::list<HealingInfo>::iterator it = m_healing.begin(); it != m_healing.end(); /*++it*/)
		{
			if (it->m_gettingHealedID != INVALID_ID)
			{
				Object* objToHeal = TheGameLogic->findObjectByID(it->m_gettingHealedID);
				if (objToHeal == nullptr || objToHeal->isEffectivelyDead())
				{
					it = m_healing.erase(it);
				}
				else
				{
					DamageInfo healInfo;
					healInfo.in.m_damageType = DAMAGE_HEALING;
					healInfo.in.m_deathType = DEATH_NONE;
					healInfo.in.m_sourceID = getObject()->getID();
  				healInfo.in.m_amount = HEAL_RATE_FRAMES * d->m_healAmount * SECONDS_PER_LOGICFRAME_REAL;

//          if ( objToHeal->isKindOf( KINDOF_PRODUCED_AT_HELIPAD ) )
//            healInfo.in.m_amount += HEAL_RATE_FRAMES * d->m_extraHealAmount4Helicopters * SECONDS_PER_LOGICFRAME_REAL;

					BodyModuleInterface *body = objToHeal->getBodyModule();
					body->attemptHealing( &healInfo );
					++it;
				}
			}
		}
	}

	return UPDATE_SLEEP_NONE;
}

//-------------------------------------------------------------------------------------------------
ExitDoorType ParkingPlaceBehavior::reserveDoorForExit( const ThingTemplate* objType, Object *specificObject )
{
	buildInfo();
	purgeDead();

	if (objType->isKindOf(KINDOF_PRODUCED_AT_HELIPAD))
	{
		return DOOR_NONE_NEEDED;
	}

	// note that this never allocates a per-unit parking space, even if specificObject is null.
	// this is by design and is assumed by exitObjectViaDoor.
	ParkingPlaceInfo* ppi = findEmptyPPI();
	if (ppi)
	{
		ppi->m_objectInSpace = INVALID_ID;
		ppi->m_reservedForExit = true;
		return ppi->m_door;
	}

	return DOOR_NONE_AVAILABLE;
}

//-------------------------------------------------------------------------------------------------
void ParkingPlaceBehavior::exitObjectViaDoor( Object *newObj, ExitDoorType exitDoor ) ///< Here is the thing I want you to exit
{
	if (exitDoor != DOOR_NONE_NEEDED)
	{
		ParkingPlaceInfo* ppi = nullptr;
		for (std::vector<ParkingPlaceInfo>::iterator it = m_spaces.begin(); it != m_spaces.end(); ++it)
		{
			if (it->m_objectInSpace == INVALID_ID && it->m_reservedForExit == TRUE && it->m_door == exitDoor)
			{
				ppi = &(*it);
				break;
			}
		}

		if (!ppi)
		{
			DEBUG_CRASH(("could not find the space. what?"));
			return;
		}

		ppi->m_objectInSpace = newObj->getID();
		ppi->m_reservedForExit = false;
	}

	/// @todo srj -- this is evil. fix.
	static NameKeyType jetKey = TheNameKeyGenerator->nameToKey( "JetAIUpdate" );
	JetAIUpdate* ju = (JetAIUpdate *)newObj->findUpdateModule( jetKey );
	Real parkingOffset = ju ? ju->friend_getParkingOffset() : 0.0f;
	Bool producedAtHelipad = newObj->isKindOf(KINDOF_PRODUCED_AT_HELIPAD);

	PPInfo ppinfo;
	DUMPMATRIX3D(getObject()->getTransformMatrix());
	DUMPCOORD3D(getObject()->getPosition());
	if (producedAtHelipad)
	{
		CRCDEBUG_LOG(("Produced at helipad (door = %d)", exitDoor));
		DEBUG_ASSERTCRASH(exitDoor == DOOR_NONE_NEEDED, ("Hmm, unlikely"));
		Matrix3D mtx;
		MAYBE_UNUSED Bool boneOk = getObject()->getSingleLogicalBonePosition("HeliPark01", &ppinfo.hangarInternal, &mtx);
		(void)boneOk;

		DEBUG_ASSERTCRASH(boneOk, ("Could not get bone!"));
		ppinfo.hangarInternalOrient = mtx.Get_Z_Rotation();
		ppinfo.parkingSpace = ppinfo.hangarInternal;
		ppinfo.parkingOrientation = ppinfo.hangarInternalOrient;
	}
	else
	{
		CRCDEBUG_LOG(("Produced at hangar (door = %d)", exitDoor));
		DEBUG_ASSERTCRASH(exitDoor != DOOR_NONE_NEEDED, ("Hmm, unlikely"));
		if (!reserveSpace(newObj->getID(), parkingOffset, &ppinfo)) //&loc, &orient, nullptr, nullptr, nullptr, nullptr, &hangarInternal, &hangOrient))
		{
			DEBUG_CRASH(("no spaces available, how did we get here?"));
			ppinfo.parkingSpace = *getObject()->getPosition();
			ppinfo.parkingOrientation = getObject()->getOrientation();
		}
	}
	DUMPCOORD3D(&ppinfo.hangarInternal);
	DUMPREAL(ppinfo.hangarInternalOrient);
	DUMPCOORD3D(&ppinfo.parkingSpace);
	DUMPREAL(ppinfo.parkingOrientation);

	newObj->setPosition( &ppinfo.hangarInternal );
	newObj->setOrientation( ppinfo.hangarInternalOrient );
	TheAI->pathfinder()->addObjectToPathfindMap( newObj );

	AIUpdateInterface  *ai = newObj->getAIUpdateInterface();
	if( ai )
	{
		Bool movedToRallyPoint = FALSE;
		if( producedAtHelipad )
		{
			const Coord3D *rallyPoint = getRallyPoint();
			if( rallyPoint )
			{
				ai->aiMoveToPosition( rallyPoint, CMD_FROM_AI );
				movedToRallyPoint = TRUE;
			}
		}
		if( !movedToRallyPoint )
		{
			if( !newObj->isKindOf( KINDOF_PRODUCED_AT_HELIPAD ) )
			{
				std::vector<Coord3D> exitPath;
				exitPath.push_back(ppinfo.parkingSpace);
				ai->aiFollowExitProductionPath( &exitPath, getObject(), CMD_FROM_AI );
			}
			else
			{
				// Lorenzen sez: aiMoveToPosition has an added benefit.
				// It invokes the pathfinder to find a vacant destination.
	      ai->aiMoveToPosition( &ppinfo.parkingSpace, CMD_FROM_AI );
			}
		}
	}
}

//-------------------------------------------------------------------------------------------------
void ParkingPlaceBehavior::unreserveDoorForExit( ExitDoorType exitDoor )
{
	if (exitDoor != DOOR_NONE_NEEDED)
	{
		for (std::vector<ParkingPlaceInfo>::iterator it = m_spaces.begin(); it != m_spaces.end(); ++it)
		{
			if (it->m_door == exitDoor)
			{
				//DEBUG_ASSERTCRASH(it->m_reservedForExit, ("ParkingPlaceBehavior::unreserveDoorForExit: door %d was not reserved",exitDoor));
				it->m_objectInSpace = INVALID_ID;
				it->m_reservedForExit = false;
				return;
			}
		}

		DEBUG_CRASH(("ParkingPlaceBehavior::unreserveDoorForExit: door %d was not found",exitDoor));
	}
}

//-------------------------------------------------------------------------------------------------
void ParkingPlaceBehavior::setRallyPoint( const Coord3D *pos )
{
	m_heliRallyPointExists = TRUE;
	m_heliRallyPoint.set( *pos );
	// nothing
}

//-------------------------------------------------------------------------------------------------
const Coord3D* ParkingPlaceBehavior::getRallyPoint() const
{
	if( m_heliRallyPointExists )
	{
		return &m_heliRallyPoint;
	}
	return nullptr;
}

//-------------------------------------------------------------------------------------------------
//We only use this for the helipad -- therefore the exit position is the helipad creation point.
//-------------------------------------------------------------------------------------------------
Bool ParkingPlaceBehavior::getExitPosition( Coord3D& exitPosition ) const
{
	Matrix3D mtx;
	return getObject()->getSingleLogicalBonePosition("HeliPark01", &exitPosition, &mtx );
}

//-------------------------------------------------------------------------------------------------
Bool ParkingPlaceBehavior::getNaturalRallyPoint( Coord3D& rallyPoint, Bool offset ) const
{
	Matrix3D mtx;
	return getObject()->getSingleLogicalBonePosition("HeliPark01", &rallyPoint, &mtx );
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void ParkingPlaceBehavior::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void ParkingPlaceBehavior::xfer( Xfer *xfer )
{
	Int i;

	// version
	const XferVersion currentVersion = 3;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	if( xfer->getXferMode() == XFER_LOAD )
	{
		// first, build our info, so it won't be overwritten later.
		buildInfo();
	}

	// spaces info count and data
	UnsignedByte spacesCount = m_spaces.size();
	xfer->xferUnsignedByte( &spacesCount );
	if( xfer->getXferMode() == XFER_SAVE )
	{

		// save all elements
		std::vector< ParkingPlaceInfo >::iterator it;
		for( it = m_spaces.begin(); it != m_spaces.end(); ++it )
		{

			// object in this space
			xfer->xferObjectID( &((*it).m_objectInSpace) );

			// reserved for exit
			xfer->xferBool( &((*it).m_reservedForExit) );

		}

	}
	else if( xfer->getXferMode() == XFER_LOAD )
	{
		ObjectID objectID;
		Bool reservedForExit;

		// read all elements
		std::vector< ParkingPlaceInfo >::iterator it;
		it = m_spaces.begin();
		for( i = 0; i < spacesCount; ++i )
		{

			// read object id
			xfer->xferObjectID( &objectID );

			// read reserved flag
			xfer->xferBool( &reservedForExit );

			// store in vector if the vector does indeed still have room for this entry
			if( it != m_spaces.end() )
			{

				(*it).m_objectInSpace = objectID;
				(*it).m_reservedForExit = reservedForExit;
				++it;

			}

		}

	}

	// runways could and info
	UnsignedByte runwaysCount = m_runways.size();
	xfer->xferUnsignedByte( &runwaysCount );
	if( xfer->getXferMode() == XFER_SAVE )
	{

		// save all elements
		std::vector< RunwayInfo >::iterator it;
		for( it = m_runways.begin(); it != m_runways.end(); ++it )
		{

			// save object ID
			xfer->xferObjectID( &((*it).m_inUseBy) );
			xfer->xferObjectID( &((*it).m_nextInLineForTakeoff) );
			xfer->xferBool( &((*it).m_wasInLine) );

		}

	}
	else if( xfer->getXferMode() == XFER_LOAD )
	{
		// read all elements
		std::vector< RunwayInfo >::iterator it;
		it = m_runways.begin();
		for( i = 0; i < runwaysCount; ++i )
		{

			ObjectID inUseBy;
			ObjectID nextInLineForTakeoff;
			Bool wasInLine;

			// read object ID
			xfer->xferObjectID( &inUseBy );
			xfer->xferObjectID( &nextInLineForTakeoff );
			xfer->xferBool( &wasInLine );

			// store in vector if the vector does indeed still have room for this entry
			if( it != m_runways.end() )
			{

				(*it).m_inUseBy = inUseBy;
				(*it).m_nextInLineForTakeoff = nextInLineForTakeoff;
				(*it).m_wasInLine = wasInLine;
				++it;

			}

		}

	}

	// healees
	UnsignedByte healCount = m_healing.size();
	xfer->xferUnsignedByte( &healCount );
	if( xfer->getXferMode() == XFER_SAVE )
	{

		// save all elements
		std::list< HealingInfo >::iterator it;
		for( it = m_healing.begin(); it != m_healing.end(); ++it )
		{

			// save object ID
			xfer->xferObjectID( &((*it).m_gettingHealedID) );
			xfer->xferUnsignedInt( &((*it).m_healStartFrame) );

		}

	}
	else if( xfer->getXferMode() == XFER_LOAD )
	{
		// read all elements
		m_healing.clear();
		for( i = 0; i < healCount; ++i )
		{
			HealingInfo info;

			// read object ID
			xfer->xferObjectID( &info.m_gettingHealedID );
			xfer->xferUnsignedInt( &info.m_healStartFrame );
			m_healing.push_back(info);

		}

	}

	if (version >= 2)
	{
		xfer->xferCoord3D(&m_heliRallyPoint);
		xfer->xferBool(&m_heliRallyPointExists);
	}

	if (version >= 3)
	{
		xfer->xferUnsignedInt(&m_nextHealFrame);
	}
	else
	{
		// old file .. just wake it up right away, and it will fall into the correct pattern
		if (xfer->getXferMode() == XFER_LOAD)
		{
			m_nextHealFrame = 0;
		}
	}

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void ParkingPlaceBehavior::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

	// no, this is bad.. it is NOT SAFE to call setWakeFrame from the xfer system. crap. (srj)
	// make sure we are awake... old save games let us sleep
	//setWakeFrame(getObject(), UPDATE_SLEEP_NONE);

}
```
