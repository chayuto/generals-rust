# Translation Packet `PKT-270`: `FlightDeckBehavior`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-270`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/behavior/flight_deck_behavior.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (22):**
  - `Common/CRCDebug.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/CRCDebug.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/Team.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Team.h`
  - `Common/ThingFactory.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ThingFactory.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/ParticleSys.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/ParticleSys.h`
  - `GameLogic/AI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/AI.h`
  - `GameLogic/AIPathfind.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/AIPathfind.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/DieModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DieModule.h`
  - `GameLogic/Module/FlightDeckBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/FlightDeckBehavior.h`
  - `GameLogic/Module/JetAIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/JetAIUpdate.h`
  - `GameLogic/Module/ProductionUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ProductionUpdate.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `GameLogic/TerrainLogic.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/TerrainLogic.h`
  - `GameLogic/Weapon.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h`
  - `PreRTS.h`
- **Implemented Module Facets (3):**
  - `DieModuleInterface* getDie()`
  - `ExitInterface* getUpdateExitInterface()`
  - `ParkingPlaceBehaviorInterface* getParkingPlaceBehaviorInterface()`
- **INI `FieldParse` Table Entries (26):**
  - `"NumRunways" => INI::parseInt`
  - `"NumSpacesPerRunway" => INI::parseInt`
  - `"Runway1Spaces" => INI::parseAsciiStringVector`
  - `"Runway1Takeoff" => parseRunwayStrip`
  - `"Runway1Landing" => parseRunwayStrip`
  - `"Runway1Taxi" => INI::parseAsciiStringVector`
  - `"Runway1Creation" => INI::parseAsciiStringVector`
  - `"Runway1CatapultSystem" => INI::parseParticleSystemTemplate`
  - `"Runway2Spaces" => INI::parseAsciiStringVector`
  - `"Runway2Takeoff" => parseRunwayStrip`
  - `"Runway2Landing" => parseRunwayStrip`
  - `"Runway2Taxi" => INI::parseAsciiStringVector`
  - `"Runway2Creation" => INI::parseAsciiStringVector`
  - `"Runway2CatapultSystem" => INI::parseParticleSystemTemplate`
  - `"ApproachHeight" => INI::parseReal`
  - `"LandingDeckHeightOffset" => INI::parseReal`
  - `"HealAmountPerSecond" => INI::parseReal`
  - `"ParkingCleanupPeriod" => INI::parseDurationUnsignedInt`
  - `"HumanFollowPeriod" => INI::parseDurationUnsignedInt`
  - `"PayloadTemplate" => INI::parseAsciiString`
  - `"ReplacementDelay" => INI::parseDurationUnsignedInt`
  - `"DockAnimationDelay" => INI::parseDurationUnsignedInt`
  - `"LaunchWaveDelay" => INI::parseDurationUnsignedInt`
  - `"LaunchRampDelay" => INI::parseDurationUnsignedInt`
  - `"LowerRampDelay" => INI::parseDurationUnsignedInt`
  - `"CatapultFireDelay" => INI::parseDurationUnsignedInt`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (33 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUnsignedByte( &spacesCount )`
  3. `xferObjectID( &((*it).m_objectInSpace) )`
  4. `xferObjectID( &objectID )`
  5. `xferUnsignedByte( &runwaysCount )`
  6. `xferObjectID( &((*it).m_inUseByForTakeoff ) )`
  7. `xferObjectID( &((*it).m_inUseByForLanding ) )`
  8. `xferObjectID( &inUseByForTakeoff )`
  9. `xferObjectID( &inUseByForLanding )`
  10. `xferBool( &wasInLine )`
  11. `xferUnsignedByte( &healCount )`
  12. `xferObjectID( &((*it).m_gettingHealedID) )`
  13. `xferUnsignedInt( &((*it).m_healStartFrame) )`
  14. `xferObjectID( &info.m_gettingHealedID )`
  15. `xferUnsignedInt( &info.m_healStartFrame )`
  16. `xferUnsignedInt( &m_nextHealFrame )`
  17. `xferUnsignedInt( &m_nextCleanupFrame )`
  18. `xferUnsignedInt( &m_startedProductionFrame )`
  19. `xferUnsignedInt( &m_nextAllowedProductionFrame )`
  20. `xferObjectID( &m_designatedTarget )`
  21. `xferInt( &commandType )`
  22. `xferCoord3D( &m_designatedPosition )`
  23. `xferUnsignedInt( &maxRunways )`
  24. `xferUnsignedInt( &m_nextLaunchWaveFrame[ i ] )`
  25. `xferUnsignedInt( &m_rampUpFrame[ i ] )`
  26. `xferUnsignedInt( &m_catapultSystemFrame[ i ] )`
  27. `xferUnsignedInt( &m_lowerRampFrame[ i ] )`
  28. `xferBool( &m_rampUp[ i ] )`
  29. `xferUnsignedInt( &dummyInt )`
  30. `xferUnsignedInt( &dummyInt )`
  31. `xferUnsignedInt( &dummyInt )`
  32. `xferUnsignedInt( &dummyInt )`
  33. `xferBool( &dummyBool )`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/FlightDeckBehavior.h`
```cpp
// FILE: FlightDeckBehavior.h /////////////////////////////////////////////////////////////////////
// Author: Kris Morness, May 2003
// Desc: Handles aircraft movement and parking behavior for aircraft carriers.
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/DieModule.h"
#include "GameLogic/Module/AIUpdate.h"


#define MAX_RUNWAYS			2 //***NOTE: If you change this, make sure you update the parsing section!
													//And also do a search for MAX_RUNWAYS and evaluate any special case comments!

enum
{
	RUNWAY_START_BONE,
	RUNWAY_END_BONE,
	NUM_RUNWAY_BONES,
};

struct RunwayDefinition
{
	RunwayDefinition()
	{
		m_catapultParticleSystem = nullptr;
	}

	std::vector<AsciiString> m_spacesBoneNames;
	std::vector<AsciiString> m_taxiBoneNames;
	std::vector<AsciiString> m_creationBoneNames;
	AsciiString m_takeoffBoneNames[ NUM_RUNWAY_BONES ];
	AsciiString m_landingBoneNames[ NUM_RUNWAY_BONES ];
	const ParticleSystemTemplate *m_catapultParticleSystem;
};

//-------------------------------------------------------------------------------------------------
class FlightDeckBehaviorModuleData : public AIUpdateModuleData
{
public:
	RunwayDefinition m_runwayInfo[ MAX_RUNWAYS ];
	AsciiString   m_thingTemplateName;
	Real					m_healAmount;
	Real					m_approachHeight;
	Real					m_landingDeckHeightOffset;
	Int						m_numRows;
	Int						m_numCols;
	UnsignedInt		m_cleanupFrames;
	UnsignedInt		m_humanFollowFrames;
	UnsignedInt		m_replacementFrames;
	UnsignedInt		m_dockAnimationFrames;
	UnsignedInt		m_launchWaveFrames;
	UnsignedInt   m_launchRampFrames;
	UnsignedInt   m_lowerRampFrames;
	UnsignedInt		m_catapultFireFrames;

	FlightDeckBehaviorModuleData();
	static void buildFieldParse( MultiIniFieldParse& p );
	static void parseRunwayStrip( INI* ini, void *instance, void *store, const void* /*userData*/ );
};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class FlightDeckBehavior : public AIUpdateInterface,
													 public ParkingPlaceBehaviorInterface,
													 public DieModuleInterface,
													 public ExitInterface

{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( FlightDeckBehavior, "FlightDeckBehavior" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( FlightDeckBehavior, FlightDeckBehaviorModuleData )

public:

	FlightDeckBehavior( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	static Int getInterfaceMask() { return UpdateModule::getInterfaceMask() | (MODULEINTERFACE_DIE); }

	// BehaviorModule
	virtual ParkingPlaceBehaviorInterface* getParkingPlaceBehaviorInterface() override { return this; }
	virtual ExitInterface* getUpdateExitInterface() override { return this; }
	virtual DieModuleInterface* getDie() override { return this; }

	// ExitInterface
	virtual Bool isExitBusy() const override {return FALSE;}	///< Contain style exiters are getting the ability to space out exits, so ask this before reserveDoor as a kind of no-commitment check.
	virtual ExitDoorType reserveDoorForExit( const ThingTemplate* objType, Object *specificObject ) override;
	virtual void exitObjectViaDoor( Object *newObj, ExitDoorType exitDoor ) override;
	virtual void unreserveDoorForExit( ExitDoorType exitDoor ) override;
	virtual void exitObjectByBudding( Object *newObj, Object *budHost ) override { return; }
	virtual Bool getExitPosition( Coord3D& rallyPoint ) const override { return FALSE; }
	virtual Bool getNaturalRallyPoint( Coord3D& rallyPoint, Bool offset = TRUE ) const override { return FALSE; }
	virtual void setRallyPoint( const Coord3D *pos ) override {}
	virtual const Coord3D *getRallyPoint() const override { return nullptr;}

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
	virtual Real getApproachHeight() const override { return getFlightDeckBehaviorModuleData()->m_approachHeight; }
	virtual Real getLandingDeckHeightOffset() const override { return getFlightDeckBehaviorModuleData()->m_landingDeckHeightOffset; }
	virtual void setHealee(Object* healee, Bool add) override;
	virtual void killAllParkedUnits() override;
	virtual void defectAllParkedUnits(Team* newTeam, UnsignedInt detectionTime) override;
	virtual Bool calcBestParkingAssignment( ObjectID id, Coord3D *pos, Int *oldIndex = nullptr, Int *newIndex = nullptr ) override;

	// AIUpdateInterface
	virtual void aiDoCommand(const AICommandParms* parms) override;

	virtual const std::vector<Coord3D>* getTaxiLocations( ObjectID id ) const override;
	virtual const std::vector<Coord3D>* getCreationLocations( ObjectID id ) const override;

private:

	Bool isAbleToGiveUpParkingSpace( Object *jet );
	Bool isAbleToMoveForward( const Object &jet ) const;
	Bool isInPositionToTakeoff( const Object &jet ) const;
	void validateAssignments();
	void propagateOrdersToPlanes();
	void propagateOrderToSpecificPlane( Object *jet );
	Bool hasTakeoffOrders();

	struct FlightDeckInfo
	{
		Coord3D				m_prep;
		Real					m_orientation;
		Int						m_runway;
		ObjectID			m_objectInSpace;

		FlightDeckInfo()
		{
			m_prep.zero();
			m_orientation = 0;
			m_runway = 0;
			m_objectInSpace = INVALID_ID;
		}
	};

	struct RunwayInfo
	{
		Coord3D		m_start;
		Matrix3D  m_startTransform;
		Coord3D		m_end;
		Coord3D		m_landingStart;
		Coord3D		m_landingEnd;
		std::vector<Coord3D> m_taxi;
		std::vector<Coord3D> m_creation;
		Real			m_startOrient;
		ObjectID	m_inUseByForTakeoff;
		ObjectID	m_inUseByForLanding;
	};

	struct HealingInfo
	{
		ObjectID		m_gettingHealedID;
		UnsignedInt	m_healStartFrame;
	};

	void buildInfo( Bool createUnits = TRUE);
	void purgeDead();
	void resetWakeFrame();
	FlightDeckInfo* findPPI(ObjectID id);
	FlightDeckInfo* findEmptyPPI();

  const ThingTemplate *m_thingTemplate;


	std::vector<FlightDeckInfo>		m_spaces;
	std::vector<RunwayInfo>				m_runways;
	std::list<HealingInfo>				m_healing;	// note, this list can vary in size, and be larger than the parking space count
	UnsignedInt										m_nextHealFrame;
	UnsignedInt										m_nextCleanupFrame;
	UnsignedInt										m_startedProductionFrame;
	UnsignedInt										m_nextAllowedProductionFrame;
	UnsignedInt										m_nextLaunchWaveFrame[ MAX_RUNWAYS ];
	UnsignedInt										m_rampUpFrame[ MAX_RUNWAYS ];		 //The frame the ramp has completed raising.
	UnsignedInt										m_catapultSystemFrame[ MAX_RUNWAYS ]; //The frame the catapult effect created.
	UnsignedInt										m_lowerRampFrame[ MAX_RUNWAYS ]; //The frame the ramp begins to lower.
	ObjectID											m_designatedTarget;
	AICommandType									m_designatedCommand;
	Coord3D												m_designatedPosition;
	Bool													m_gotInfo;
	Bool													m_rampUp[ MAX_RUNWAYS ];

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Behavior/FlightDeckBehavior.cpp`
```cpp
// FILE: FlightDeckBehavior.cpp ///////////////////////////////////////////////////////////////////
// Author: Kris Morness, May 2003
// Desc: Handles aircraft movement and parking behavior for aircraft carriers.
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/CRCDebug.h"
#include "Common/Player.h"
#include "Common/Team.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"

#include "GameClient/Drawable.h"
#include "GameClient/ParticleSys.h"

#include "GameLogic/AI.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/TerrainLogic.h"
#include "GameLogic/Weapon.h"

#include "GameLogic/Module/FlightDeckBehavior.h"
#include "GameLogic/Module/JetAIUpdate.h"
#include "GameLogic/Module/ProductionUpdate.h"


FlightDeckBehaviorModuleData::FlightDeckBehaviorModuleData()
{
	//m_framesForFullHeal = 0;
	m_healAmount = 0;
	m_numRows = 0;
	m_numCols = 0;
	m_approachHeight = 0.0f;
	m_landingDeckHeightOffset = 0.0f;
	m_dockAnimationFrames = 0;
	m_catapultFireFrames = 0;
}

//-------------------------------------------------------------------------------------------------
void FlightDeckBehaviorModuleData::parseRunwayStrip( INI* ini, void *instance, void *store, const void* /*userData*/ )
{
	AsciiString *runwayNames = (AsciiString*)store;

	const char *token = ini->getNextTokenOrNull();
	if( token )
	{
		runwayNames[ RUNWAY_START_BONE ].format( token );
		token = ini->getNextTokenOrNull();
		if( token )
		{
			runwayNames[ RUNWAY_END_BONE ].format( token );
		}
	}
}

//-------------------------------------------------------------------------------------------------
void FlightDeckBehaviorModuleData::buildFieldParse(MultiIniFieldParse& p)
{
	AIUpdateModuleData::buildFieldParse(p);

	static const FieldParse dataFieldParse[] =
	{
		{ "NumRunways",							INI::parseInt,										nullptr, offsetof( FlightDeckBehaviorModuleData, m_numCols ) },
		{ "NumSpacesPerRunway",			INI::parseInt,										nullptr, offsetof( FlightDeckBehaviorModuleData, m_numRows ) },

		{ "Runway1Spaces",					INI::parseAsciiStringVector,			nullptr, offsetof( FlightDeckBehaviorModuleData, m_runwayInfo[ 0 ].m_spacesBoneNames ) },
		{ "Runway1Takeoff",					parseRunwayStrip,									nullptr, offsetof( FlightDeckBehaviorModuleData, m_runwayInfo[ 0 ].m_takeoffBoneNames ) },
		{ "Runway1Landing",					parseRunwayStrip,									nullptr, offsetof( FlightDeckBehaviorModuleData, m_runwayInfo[ 0 ].m_landingBoneNames ) },
		{ "Runway1Taxi",						INI::parseAsciiStringVector,			nullptr, offsetof( FlightDeckBehaviorModuleData, m_runwayInfo[ 0 ].m_taxiBoneNames ) },
		{ "Runway1Creation",				INI::parseAsciiStringVector,			nullptr, offsetof( FlightDeckBehaviorModuleData, m_runwayInfo[ 0 ].m_creationBoneNames ) },
		{ "Runway1CatapultSystem",	INI::parseParticleSystemTemplate,	nullptr, offsetof( FlightDeckBehaviorModuleData, m_runwayInfo[ 0 ].m_catapultParticleSystem ) },

		{ "Runway2Spaces",					INI::parseAsciiStringVector,			nullptr, offsetof( FlightDeckBehaviorModuleData, m_runwayInfo[ 1 ].m_spacesBoneNames ) },
		{ "Runway2Takeoff",					parseRunwayStrip,									nullptr, offsetof( FlightDeckBehaviorModuleData, m_runwayInfo[ 1 ].m_takeoffBoneNames ) },
		{ "Runway2Landing",					parseRunwayStrip,									nullptr, offsetof( FlightDeckBehaviorModuleData, m_runwayInfo[ 1 ].m_landingBoneNames ) },
		{ "Runway2Taxi",						INI::parseAsciiStringVector,			nullptr, offsetof( FlightDeckBehaviorModuleData, m_runwayInfo[ 1 ].m_taxiBoneNames ) },
		{ "Runway2Creation",				INI::parseAsciiStringVector,			nullptr, offsetof( FlightDeckBehaviorModuleData, m_runwayInfo[ 1 ].m_creationBoneNames ) },
		{ "Runway2CatapultSystem",	INI::parseParticleSystemTemplate,	nullptr, offsetof( FlightDeckBehaviorModuleData, m_runwayInfo[ 1 ].m_catapultParticleSystem ) },

		{ "ApproachHeight",					INI::parseReal,										nullptr, offsetof( FlightDeckBehaviorModuleData, m_approachHeight ) },
		{ "LandingDeckHeightOffset",INI::parseReal,										nullptr, offsetof( FlightDeckBehaviorModuleData, m_landingDeckHeightOffset ) },
		{ "HealAmountPerSecond",		INI::parseReal,										nullptr, offsetof( FlightDeckBehaviorModuleData, m_healAmount ) },
		{ "ParkingCleanupPeriod",		INI::parseDurationUnsignedInt,		nullptr, offsetof( FlightDeckBehaviorModuleData, m_cleanupFrames ) },
		{ "HumanFollowPeriod",			INI::parseDurationUnsignedInt,		nullptr, offsetof( FlightDeckBehaviorModuleData, m_humanFollowFrames ) },
		{ "PayloadTemplate",				INI::parseAsciiString,				  	nullptr, offsetof( FlightDeckBehaviorModuleData, m_thingTemplateName ) },
		{ "ReplacementDelay",				INI::parseDurationUnsignedInt, 		nullptr, offsetof( FlightDeckBehaviorModuleData, m_replacementFrames ) },
		{ "DockAnimationDelay",			INI::parseDurationUnsignedInt,		nullptr, offsetof( FlightDeckBehaviorModuleData, m_dockAnimationFrames ) },
		{ "LaunchWaveDelay",				INI::parseDurationUnsignedInt,		nullptr, offsetof( FlightDeckBehaviorModuleData, m_launchWaveFrames ) },
		{ "LaunchRampDelay",				INI::parseDurationUnsignedInt,		nullptr, offsetof( FlightDeckBehaviorModuleData, m_launchRampFrames ) },
		{ "LowerRampDelay",					INI::parseDurationUnsignedInt,		nullptr, offsetof( FlightDeckBehaviorModuleData, m_lowerRampFrames ) },
		{ "CatapultFireDelay",			INI::parseDurationUnsignedInt,		nullptr, offsetof( FlightDeckBehaviorModuleData, m_catapultFireFrames ) },

		{ nullptr, nullptr, nullptr, 0 }
	};
	p.add(dataFieldParse);
}


//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
FlightDeckBehavior::FlightDeckBehavior( Thing *thing, const ModuleData* moduleData ) : AIUpdateInterface( thing, moduleData )
{
	m_gotInfo = false;

	m_nextHealFrame = FOREVER;
	setWakeFrame(getObject(), UPDATE_SLEEP_NONE);
	m_nextCleanupFrame = 0;
	m_startedProductionFrame = FOREVER;
	m_nextAllowedProductionFrame = 0;
	m_designatedTarget = INVALID_ID;
	m_designatedCommand = AICMD_NO_COMMAND;
	m_designatedPosition.zero();

	for( int i = 0; i < MAX_RUNWAYS; i++ )
	{
		m_nextLaunchWaveFrame[ i ] = 0;
		m_rampUpFrame[ i ] = 0;
		m_catapultSystemFrame[ i ] = 0;
		m_lowerRampFrame[ i ] = 0;
		m_rampUp[ i ] = FALSE;
	}

  m_thingTemplate = nullptr;

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
FlightDeckBehavior::~FlightDeckBehavior()
{
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void FlightDeckBehavior::buildInfo(Bool createUnits)
{
	if (m_gotInfo)
		return;

	const FlightDeckBehaviorModuleData* data = getFlightDeckBehaviorModuleData();

	m_thingTemplate = TheThingFactory->findTemplate( data->m_thingTemplateName );

	if (getObject()->testStatus(OBJECT_STATUS_UNDER_CONSTRUCTION) ||
			getObject()->testStatus(OBJECT_STATUS_SOLD))
		return;

	//ProductionUpdateInterface* pu = getObject()->getProductionUpdateInterface();

	m_spaces.reserve( data->m_numRows * data->m_numCols );

	//Initialize the spaces that planes will eventually be assigned to for parking purposes
	FlightDeckInfo flightDeckInfo;

	//We want to sort the spaces so that we have runway 1 space 1, runway 2 space 1, R1S2, R2S2, R1S3...
	for( Int row = 0; row < data->m_numRows; row++ )
	{
		for( Int col = 0; col < data->m_numCols; col++ )
		{
			std::vector<AsciiString> spaces = data->m_runwayInfo[ col ].m_spacesBoneNames;
			std::vector<AsciiString>::const_iterator it;

			Int counter = 0;
			for( it = spaces.begin(); it != spaces.end() && counter < row; it++, counter++ )
			{
				//just iterate to the spaces.
			}

			AsciiString tmp;
			Matrix3D mtx;

			//Convert the module data bone names into coordinates that we can use
			getObject()->getSingleLogicalBonePosition( it->str(), &flightDeckInfo.m_prep, &mtx );
			flightDeckInfo.m_orientation = mtx.Get_Z_Rotation();

			//Init basic runway stuff
			flightDeckInfo.m_runway = col;
			flightDeckInfo.m_objectInSpace = INVALID_ID;

			//Create the payload
			Object *jet = nullptr;
			if( m_thingTemplate && createUnits )
			{
				jet = TheThingFactory->newObject( m_thingTemplate, getObject()->getControllingPlayer()->getDefaultTeam() );
				if( jet )
				{
					jet->setProducer( getObject() );
					jet->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_DECK_HEIGHT_OFFSET ) );

					//Init positioning.
					jet->setPosition( &flightDeckInfo.m_prep );
					jet->setOrientation( flightDeckInfo.m_orientation );

					//Assign jet to space
					flightDeckInfo.m_objectInSpace = jet->getID();
					//validateAssignments();
				}
			}

			m_spaces.push_back( flightDeckInfo );
		}
	}

	//Now initialize the runway take-off and landing information.
	RunwayInfo info;
	m_runways.reserve(data->m_numCols);
	for( Int col = 0; col < data->m_numCols; ++col )
	{
		AsciiString tmp;

		getObject()->getSingleLogicalBonePosition( data->m_runwayInfo[ col ].m_takeoffBoneNames[ RUNWAY_START_BONE ].str(), &info.m_start, nullptr);
		getObject()->getSingleLogicalBonePosition( data->m_runwayInfo[ col ].m_takeoffBoneNames[ RUNWAY_END_BONE ].str(), &info.m_end, nullptr);
		getObject()->getSingleLogicalBonePosition( data->m_runwayInfo[ col ].m_landingBoneNames[ RUNWAY_START_BONE ].str(), &info.m_landingStart, nullptr);
		getObject()->getSingleLogicalBonePosition( data->m_runwayInfo[ col ].m_landingBoneNames[ RUNWAY_END_BONE ].str(), &info.m_landingEnd, nullptr);

		info.m_inUseByForTakeoff = INVALID_ID;
		info.m_inUseByForLanding = INVALID_ID;

		//Get the taxi bones and store them as well (possible to have none!)
		std::vector<AsciiString> locations = data->m_runwayInfo[ col ].m_taxiBoneNames;
		std::vector<AsciiString>::const_iterator it;
		info.m_taxi.clear();
		for( it = locations.begin(); it != locations.end(); it++ )
		{
			Coord3D taxiPos;
			Matrix3D mtx;

			//Get the position of the taxi bone.
			getObject()->getSingleLogicalBonePosition( it->str(), &taxiPos, &mtx );

			//Add it to the taxi vector
			info.m_taxi.push_back( taxiPos );
		}

		//Get the creation bones and store them as well
		locations = data->m_runwayInfo[ col ].m_creationBoneNames;
		info.m_creation.clear();
		Bool firstTime = TRUE;
		for( it = locations.begin(); it != locations.end(); it++ )
		{
			Coord3D pos;
			Matrix3D mtx;

			//Get the position of the creation bone.
			getObject()->getSingleLogicalBonePosition( it->str(), &pos, &mtx );

			if( firstTime )
			{
				firstTime = FALSE;
				info.m_startOrient = mtx.Get_Z_Rotation();
				info.m_startTransform = mtx;
			}

			//Add it to the taxi vector
			info.m_creation.push_back( pos );
		}

		m_runways.push_back(info);
	}

	m_gotInfo = true;
}

//-------------------------------------------------------------------------------------------------
void FlightDeckBehavior::purgeDead()
{
	buildInfo();

	for (std::vector<FlightDeckInfo>::iterator it = m_spaces.begin(); it != m_spaces.end(); ++it)
	{
		if (it->m_objectInSpace != INVALID_ID)
		{
			Object* obj = TheGameLogic->findObjectByID(it->m_objectInSpace);
			if (obj == nullptr || obj->isEffectivelyDead())
			{
				it->m_objectInSpace = INVALID_ID;
			}
		}
	}

	{
		for (std::vector<RunwayInfo>::iterator it = m_runways.begin(); it != m_runways.end(); ++it)
		{
			if (it->m_inUseByForTakeoff != INVALID_ID)
			{
				Object* obj = TheGameLogic->findObjectByID(it->m_inUseByForTakeoff);
				if (obj == nullptr || obj->isEffectivelyDead())
				{
					it->m_inUseByForTakeoff = INVALID_ID;
				}
			}
			if (it->m_inUseByForLanding != INVALID_ID)
			{
				Object* obj = TheGameLogic->findObjectByID(it->m_inUseByForLanding);
				if (obj == nullptr || obj->isEffectivelyDead())
				{
					it->m_inUseByForLanding = INVALID_ID;
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
Bool FlightDeckBehavior::hasReservedSpace(ObjectID id) const
{
	if (!m_gotInfo)
		return false;

	if (id == INVALID_ID)	// shouldn't call this way, but Weapon mistakenly does sometimes, so check for it
		return false;

	for (std::vector<FlightDeckInfo>::const_iterator it = m_spaces.begin(); it != m_spaces.end(); ++it)
	{
		if (it->m_objectInSpace == id)
			return true;
	}
	return false;
}

//-------------------------------------------------------------------------------------------------
Int FlightDeckBehavior::getSpaceIndex( ObjectID id ) const
{
	if( id == INVALID_ID )
	{
		return -1;
	}

	Int index = 0;
	for( std::vector<FlightDeckInfo>::const_iterator it = m_spaces.begin(); it != m_spaces.end(); it++, index++ )
	{
		if (it->m_objectInSpace == id)
		{
			return index;
		}
	}
	return -1;
}

//-------------------------------------------------------------------------------------------------
FlightDeckBehavior::FlightDeckInfo* FlightDeckBehavior::findPPI(ObjectID id)
{
	DEBUG_ASSERTCRASH(id != INVALID_ID, ("call findEmptyPPI instead"));

	if (!m_gotInfo || id == INVALID_ID)
		return nullptr;

	for (std::vector<FlightDeckInfo>::iterator it = m_spaces.begin(); it != m_spaces.end(); ++it)
	{
		if (it->m_objectInSpace == id)
			return &(*it);
	}

	return nullptr;
}

//-------------------------------------------------------------------------------------------------
FlightDeckBehavior::FlightDeckInfo* FlightDeckBehavior::findEmptyPPI()
{
	if (!m_gotInfo)
		return nullptr;

	for (std::vector<FlightDeckInfo>::iterator it = m_spaces.begin(); it != m_spaces.end(); ++it)
	{
		if( it->m_objectInSpace == INVALID_ID )
			return &(*it);
	}

	return nullptr;
}

//-------------------------------------------------------------------------------------------------
// note: called from client, so MUST NOT modify self in any way, or desyncs will occur
Bool FlightDeckBehavior::shouldReserveDoorWhenQueued(const ThingTemplate* thing) const
{
	return true;
}

//-------------------------------------------------------------------------------------------------
// note: called from client, so MUST NOT modify self in any way, or desyncs will occur
Bool FlightDeckBehavior::hasAvailableSpaceFor(const ThingTemplate* thing) const
{
	if (!m_gotInfo)	// degenerate case, shouldn't happen, but just in case...
		return false;

	for (std::vector<FlightDeckInfo>::const_iterator it = m_spaces.begin(); it != m_spaces.end(); ++it)
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

		if( id == INVALID_ID )
		{
			return true;
		}
	}

	return false;
}

//-------------------------------------------------------------------------------------------------
Bool FlightDeckBehavior::reserveSpace(ObjectID id, Real parkingOffset, ParkingPlaceBehaviorInterface::PPInfo* info)
{
	buildInfo();
	purgeDead();

	const FlightDeckBehaviorModuleData* d = getFlightDeckBehaviorModuleData();

	FlightDeckInfo* ppi = findPPI(id);
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
	//validateAssignments();

	if( d->m_landingDeckHeightOffset )
	{
		Object *obj = TheGameLogic->findObjectByID( id );
		if( obj )
		{
			obj->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_DECK_HEIGHT_OFFSET ) );
		}
	}

	if( info )
	{
		calcPPInfo( id, info );
		if (parkingOffset != 0.0f)
		{
			info->parkingSpace.x += parkingOffset * Cos(ppi->m_orientation);
			info->parkingSpace.y += parkingOffset * Sin(ppi->m_orientation);
		}
	}

	return true;
}

//-------------------------------------------------------------------------------------------------
void FlightDeckBehavior::validateAssignments()
{
	Int index = 0, index2 = 0;
	for( std::vector<FlightDeckInfo>::const_iterator it = m_spaces.begin(); it != m_spaces.end(); it++, index++ )
	{
		ObjectID id = it->m_objectInSpace;
		if( id != INVALID_ID )
		{
			for( std::vector<FlightDeckInfo>::const_iterator it2 = it; it2 != m_spaces.end(); it2++, index2++ )
			{
				if( it == it2 )
				{
					continue;
				}
				ObjectID id2 = it2->m_objectInSpace;
				if( id == id2 )
				{
					DEBUG_CRASH( ("Aircraft %d assigned to multiple spaces %d and %d", id, index, index2 ) );
				}
			}
		}
	}
}

//-------------------------------------------------------------------------------------------------
void FlightDeckBehavior::calcPPInfo( ObjectID id, PPInfo *info )
{
	FlightDeckInfo *ppi = findPPI( id );
	if( !ppi )
	{
		//Utter failure.
		return;
	}
	const RunwayInfo& rr = m_runways[ ppi->m_runway ];

	if( info )
	{
		const FlightDeckBehaviorModuleData* d = getFlightDeckBehaviorModuleData();
		const Real APPROACH_DIST = 0.75f;
		info->parkingSpace = ppi->m_prep;
		info->runwayPrep = ppi->m_prep;
		info->parkingOrientation = ppi->m_orientation;
		info->runwayStart = rr.m_start;
		info->runwayEnd = rr.m_end;
		info->runwayExit = rr.m_end;
		info->runwayExit.x += (rr.m_end.x - rr.m_start.x) * APPROACH_DIST;
		info->runwayExit.y += (rr.m_end.y - rr.m_start.y) * APPROACH_DIST;
		info->runwayExit.z = rr.m_end.z + d->m_approachHeight + d->m_landingDeckHeightOffset;

		info->runwayLandingStart = rr.m_landingStart;
		info->runwayLandingEnd = rr.m_landingEnd;
		info->runwayApproach = rr.m_landingStart;
		info->runwayApproach.x += (rr.m_landingStart.x - rr.m_landingEnd.x) * APPROACH_DIST;
		info->runwayApproach.y += (rr.m_landingStart.y - rr.m_landingEnd.y) * APPROACH_DIST;
		info->runwayApproach.z = rr.m_landingStart.z + d->m_approachHeight + d->m_landingDeckHeightOffset;

		//Cache the runway's takeoff distance used by JetAIUpdate for calculating lift.
		Coord3D vector = info->runwayStart;
		vector.sub( info->runwayEnd );
		info->runwayTakeoffDist = vector.length();

		for (std::vector<RunwayInfo>::iterator it = m_runways.begin(); it != m_runways.end(); ++it)
		{
			if (it->m_inUseByForTakeoff == id )
			{
				info->runwayStart = info->runwayPrep;
				break;
			}
		}
	}
}


//-------------------------------------------------------------------------------------------------
void FlightDeckBehavior::releaseSpace(ObjectID id)
{
	buildInfo();
	purgeDead();

	for (std::vector<FlightDeckInfo>::iterator it = m_spaces.begin(); it != m_spaces.end(); ++it)
	{
		if (it->m_objectInSpace == id)
		{
			it->m_objectInSpace = INVALID_ID;
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
Int FlightDeckBehavior::getRunwayIndex(ObjectID id)
{
	purgeDead();

	for (std::vector<FlightDeckInfo>::iterator it = m_spaces.begin(); it != m_spaces.end(); ++it)
	{
		if (it->m_objectInSpace == id)
		{
			return it->m_runway;
		}
	}
	return InvalidRunway;
}

//-------------------------------------------------------------------------------------------------
ObjectID FlightDeckBehavior::getRunwayReservation( Int runway, RunwayReservationType type )
{
	buildInfo();
	purgeDead();
	switch( type )
	{
		case RESERVATION_TAKEOFF:
			return m_runways[runway].m_inUseByForTakeoff;
		case RESERVATION_LANDING:
			return m_runways[runway].m_inUseByForLanding;
		default:
			return INVALID_ID;
	}
}

//-------------------------------------------------------------------------------------------------
void FlightDeckBehavior::transferRunwayReservationToNextInLineForTakeoff(ObjectID id)
{
	//Aircraft carrier controls this functionality with an iron fist.
}

//-------------------------------------------------------------------------------------------------
Bool FlightDeckBehavior::reserveRunway(ObjectID id, Bool forLanding)
{
	buildInfo();
	purgeDead();

	Int runway = -1;

	if( !forLanding )
	{
		//Only look at the front spaces for takeoff. You can't take off from the back!
		const FlightDeckBehaviorModuleData *data = getFlightDeckBehaviorModuleData();
		for( Int i = 0; i < data->m_numCols; i++ )
		{
			if( m_spaces[ i ].m_objectInSpace == id )
			{
				runway = m_spaces[ i ].m_runway;
				break;
			}
		}
	}
	else
	{
		//Look at all spaces for landing.
		for (std::vector<FlightDeckInfo>::iterator it = m_spaces.begin(); it != m_spaces.end(); ++it)
		{
			if (it->m_objectInSpace == id)
			{
				runway = it->m_runway;
				break;
			}
		}
	}
	if (runway == -1)
	{
		if( forLanding )
		{
			DEBUG_CRASH(("only planes with reserved spaces can reserve runways"));
		}
		return false;
	}

	RunwayInfo& info = m_runways[runway];
	if( (info.m_inUseByForTakeoff == id && !forLanding) || (info.m_inUseByForLanding == id && forLanding) )
	{
		return true;
	}
	else if( (info.m_inUseByForTakeoff == INVALID_ID && !forLanding) || (info.m_inUseByForLanding == INVALID_ID && forLanding) )
	{
		if( forLanding )
		{
			info.m_inUseByForLanding = id;
		}
		else
		{
			info.m_inUseByForTakeoff = id;
		}

		return true;
	}

	return false;
}


//-------------------------------------------------------------------------------------------------
void FlightDeckBehavior::releaseRunway(ObjectID id)
{
	buildInfo();
	purgeDead();

	for (std::vector<RunwayInfo>::iterator it = m_runways.begin(); it != m_runways.end(); ++it)
	{
		if( it->m_inUseByForTakeoff == id )
		{
			it->m_inUseByForTakeoff = INVALID_ID;
		}
		if( it->m_inUseByForLanding == id )
		{
			it->m_inUseByForLanding = INVALID_ID;
		}
	}
}

//-------------------------------------------------------------------------------------------------
const std::vector<Coord3D>* FlightDeckBehavior::getTaxiLocations( ObjectID id ) const
{
	//Find the runway the object is assigned to.
	Int runway = -1;
	for( std::vector<FlightDeckInfo>::const_iterator it = m_spaces.begin(); it != m_spaces.end(); it++ )
	{
		if( it->m_objectInSpace == id )
		{
			runway = it->m_runway;
			break;
		}
	}

	if( runway == -1 )
	{
		DEBUG_CRASH(("only planes with reserved spaces can reserve runways"));
		return nullptr;
	}

	//Now get the runway we're assigned to and return it's taxi vector
	return &(m_runways[ runway ].m_taxi);
}

//-------------------------------------------------------------------------------------------------
const std::vector<Coord3D>* FlightDeckBehavior::getCreationLocations( ObjectID id ) const
{
	//Find the runway the object is assigned to.
	Int runway = -1;
	for( std::vector<FlightDeckInfo>::const_iterator it = m_spaces.begin(); it != m_spaces.end(); it++ )
	{
		if( it->m_objectInSpace == id )
		{
			runway = it->m_runway;
			break;
		}
	}

	if( runway == -1 )
	{
		DEBUG_CRASH(("only planes with reserved spaces can reserve runways"));
		return nullptr;
	}

	//Now get the runway we're assigned to and return it's creation vector
	return &(m_runways[ runway ].m_creation);
}

//-------------------------------------------------------------------------------------------------
Bool FlightDeckBehavior::isAbleToGiveUpParkingSpace( Object *jet )
{
	//If we're airborne or non-existent, someone else can have my spot if they need it.
	if( !jet || jet->isAirborneTarget() )
	{
		return TRUE;
	}

	JetAIUpdate *ai = (JetAIUpdate*)jet->getAI();
	if( ai )
	{
		//If I'm not idle, I'm certainly not parked. But I could be on route to park in my space!
		if( !ai->isIdle() && !ai->isTaxiingToParking() )
		{
			//We're definitely not airborne, so if our pending command is to land at the carrier, nuke the pending
			//command.
			AICommandType command = ai->friend_getPendingCommandType();
			if( command == AICMD_ENTER || m_designatedCommand == AICMD_IDLE )
			{
				ai->friend_purgePendingCommand();
				return FALSE;
			}

			//But make sure we're not waiting to takeoff!
			std::vector< RunwayInfo >::const_iterator it;
			for( it = m_runways.begin(); it != m_runways.end(); it++ )
			{
				if( it->m_inUseByForTakeoff == jet->getID() )
				{
					return FALSE;
				}
			}
			return TRUE;
		}
	}
	return FALSE;
}

//-------------------------------------------------------------------------------------------------
Bool FlightDeckBehavior::isInPositionToTakeoff( const Object &jet ) const
{
	const AIUpdateInterface *ai = jet.getAI();
	const FlightDeckBehaviorModuleData *data = getFlightDeckBehaviorModuleData();
	if( ai )
	{
		for( int i = 0; i < data->m_numCols; i++ )
		{
			if( m_spaces[ i ].m_objectInSpace == jet.getID() )
			{
				//This code fixes a problem where there is a one frame lag between the
				//time a jet gets assigned to the front spot, and the time it's AI is able to
				//order it to taxi into the position. When this happens, the ramp triggers its
				//animation, and the jet drives through it. So to counter that, simply check
				//the distance between the jet and the space.
				Real distanceSqr = ThePartitionManager->getDistanceSquared( &jet, &m_spaces[ i ].m_prep, FROM_CENTER_2D );
				if( distanceSqr < 10.0f )
				{
					return TRUE;
				}
				return FALSE;
			}
		}
	}
	return FALSE;
}


//-------------------------------------------------------------------------------------------------
Bool FlightDeckBehavior::isAbleToMoveForward( const Object &jet ) const
{
	JetAIUpdate *jetAI = (JetAIUpdate*)jet.getAI();
	if( jetAI && !jet.isAirborneTarget() )
	{
		//We're not airborne.
		if( jetAI->isIdle() )
		{
			//We're idle (not moving)
			return TRUE;
		}

		//We might be rearming... if so, allow it.
		if( jetAI->isReloading() )
		{
			return TRUE;
		}
	}
	return FALSE;
}


//-------------------------------------------------------------------------------------------------
Bool FlightDeckBehavior::calcBestParkingAssignment( ObjectID id, Coord3D *pos, Int *oldIndex, Int *newIndex )
{
	//Find the runway the object is assigned to.
	Int runway = -1;
	Int myIndex = 0;
	std::vector<FlightDeckInfo>::iterator myIt = m_spaces.begin();
	for( ; myIt != m_spaces.end(); myIt++, myIndex++ )
	{
		if( myIt->m_objectInSpace == id )
		{
			runway = myIt->m_runway;

			if( pos )
			{
				pos->set( myIt->m_prep );
			}
			break;
		}
	}

	if( oldIndex )
	{
		*oldIndex = myIndex;
	}

	//Now iterate the runway again in search of an empty spot in front of it. If we reach the same
	//plane again, then we're done because there is no better spot!

	//Search for the front-most available space that doesn't have any planes blocking it. So start from
	//the back and keep looking at empty spaces until we find one with a plane blocking.

	Bool checkForPlaneInWay = FALSE;
	std::vector<FlightDeckInfo>::iterator bestIt = m_spaces.end();
	Object *bestJet = nullptr;
	Int bestIndex = 0, index = 0;
	for( std::vector<FlightDeckInfo>::iterator thatIt = m_spaces.begin(); thatIt != m_spaces.end(); thatIt++, index++ )
	{
		Object *nonIdleJet = TheGameLogic->findObjectByID( thatIt->m_objectInSpace );
		if( myIt == thatIt )
		{
			//Done, don't look at my spot, nor spots behind me.
			if( bestIt != m_spaces.end() )
			{
				myIt->m_objectInSpace = bestJet ? bestJet->getID() : INVALID_ID;
				bestIt->m_objectInSpace = id;
				//validateAssignments();

				/*if( bestJet )
				{
					JetAIUpdate *jetAI = (JetAIUpdate*)bestJet->getAI();
					reserveSpace( bestJet->getID(), jetAI->friend_getParkingOffset(), nullptr );
				}
				if( nonIdleJet )
				{
					JetAIUpdate *jetAI = (JetAIUpdate*)nonIdleJet->getAI();
					reserveSpace( nonIdleJet->getID(), jetAI->friend_getParkingOffset(), nullptr );
				}*/

				if( newIndex )
				{
					*newIndex = bestIndex;
				}

				//Promoted forward.
				return TRUE;
			}
			//Did not get promoted forward.
			return FALSE;
		}
		if( thatIt->m_runway == runway )
		{
			if( !nonIdleJet || isAbleToGiveUpParkingSpace( nonIdleJet ) )
			{
				if( !checkForPlaneInWay )
				{
					//We can take this spot! But first find the flight deck info entry for it.Now handle assignment swap.
					bestIt = thatIt;
					bestJet = nonIdleJet;
					bestIndex = index;
					checkForPlaneInWay = TRUE;
					if( pos )
					{
						pos->set( thatIt->m_prep );
					}
				}
			}
			else if( checkForPlaneInWay  )
			{
				//Ugh, there's a plane parked between us and the best spot!
 				checkForPlaneInWay = FALSE;
				if( pos )
				{
					pos->set( myIt->m_prep ); //reset the original position.
					bestIt = m_spaces.end();
				}
			}
		}
	}
	//Did not get promoted forward
	return FALSE;
}


//-------------------------------------------------------------------------------------------------
// don't really need to autoheal every frame....
const Int HEAL_RATE_FRAMES = LOGICFRAMES_PER_SECOND / 5;

//-------------------------------------------------------------------------------------------------
void FlightDeckBehavior::resetWakeFrame()
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
void FlightDeckBehavior::setHealee(Object* healee, Bool add)
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
void FlightDeckBehavior::defectAllParkedUnits(Team* newTeam, UnsignedInt detectionTime)
{
	buildInfo();
	purgeDead();

	for (std::vector<FlightDeckInfo>::iterator it = m_spaces.begin(); it != m_spaces.end(); ++it)
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
void FlightDeckBehavior::killAllParkedUnits()
{
	buildInfo();
	purgeDead();

	for (std::vector<FlightDeckInfo>::iterator it = m_spaces.begin(); it != m_spaces.end(); ++it)
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
void FlightDeckBehavior::onDie( const DamageInfo *damageInfo )
{
	killAllParkedUnits();
}

//-------------------------------------------------------------------------------------------------
UpdateSleepTime FlightDeckBehavior::update()
{
	// alas, we need to keep the buildInfo and dead-purged stuff pretty much up to date, for
	// the client to be able to peek at. at this late date, the most expedient way is to ensure
	// our update is run every frame, and do this manually. the extra cost should be trivial, since
	// there are generally at most only a few airfields at any given time.
	buildInfo();
	purgeDead();

	const FlightDeckBehaviorModuleData* data = getFlightDeckBehaviorModuleData();
	UnsignedInt now = TheGameLogic->getFrame();
	if( now >= m_nextHealFrame )
	{
		m_nextHealFrame = now + HEAL_RATE_FRAMES;
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
					healInfo.in.m_amount = HEAL_RATE_FRAMES * data->m_healAmount * SECONDS_PER_LOGICFRAME_REAL;
					BodyModuleInterface *body = objToHeal->getBodyModule();
					body->attemptHealing( &healInfo );
					++it;
				}
			}
		}
	}

	//Periodically, we want to look at all the aircraft assignments and keep them near the front
	//of the carrier at all times. We do this with the following steps:
	//  1) Detect non-idle plane assigned to frontmost spaces (implying spot is empty)
	//  2) Promote next idle plane behind it to move up and take it's spot. The original plane will
	//     get assigned to it's old spot (thus bubblesorted eventually to the rear).
	if( now >= m_nextCleanupFrame )
	{
		m_nextCleanupFrame = now + data->m_cleanupFrames;
		std::vector<FlightDeckInfo>::iterator tempIt;
		Int spaceID = 0;

		Bool complete[ MAX_RUNWAYS ] = { FALSE, FALSE };
		for( std::vector<FlightDeckInfo>::iterator it = m_spaces.begin(); it != m_spaces.end(); it++, spaceID++ )
		{
			Object *nonIdleJet = TheGameLogic->findObjectByID( it->m_objectInSpace );
			if( !nonIdleJet || isAbleToGiveUpParkingSpace( nonIdleJet ) )
			{
				//Either we don't have a jet, or the jet is busy (meaning it's not parked there). When a jet
				//isn't in his spot, we will look for jets behind him to move up to take up his spot.
				Int runwayCount = data->m_numCols;
				Int tempID = spaceID;
				for( tempIt = it; tempIt != m_spaces.end(); tempIt++, tempID++ )
				{
					if( runwayCount > 0 )
					{
						//Because the spaces are sorted Runway 1 Space 1, R2S1, R1S2, R2S2 etc, we simply iterate
						//every other space. If there were 3 or 4 runways, then we would jump the number of runways
						//before each check so we don't get aircraft from a different runway getting moved up!
						runwayCount--;
						continue;
					}

					if( complete[ it->m_runway ] )
					{
						continue;
					}

					//Now we have the correct runway. Check if that spot has an idle plane in it!
					Object *parkedJet = TheGameLogic->findObjectByID( tempIt->m_objectInSpace );
					if( parkedJet && isAbleToMoveForward( *parkedJet ) )
					{
						//We have found the best candidate to replace our empty space. Now handle assignment swap.
						it->m_objectInSpace = parkedJet->getID();
						tempIt->m_objectInSpace = nonIdleJet ? nonIdleJet->getID() : INVALID_ID;
						//validateAssignments();

						//Give the parkedJet a move order to taxi over to the new spot.
						//However, we need to set a status bit to tell the AI the difference between taxiing from
						//a hangar and reassigning a parking space.
						parkedJet->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_REASSIGN_PARKING ) );

						//Doesn't matter what we push in now, the JetOrHeliTaxiState::onEnter() will nuke it
						//and calculate the taxi point to move through.
						std::vector<Coord3D> exitPath;
						exitPath.push_back( it->m_prep );
						parkedJet->getAI()->aiFollowExitProductionPath( &exitPath, getObject(), CMD_FROM_AI );

						//Determine if the first plane in each row has completed a move. We want to lag the
						//additional planes instead of moving them all simultaneously. So this is the exit
						//condition if one plane per runway has been bumped.
						complete[ it->m_runway ] = TRUE;
						m_nextCleanupFrame = now + data->m_humanFollowFrames;
					}
					//Break through and advance to the next spot!
					break;
				}
			}
		}
	}

	//Handle automatic production of lost aircraft.

	//Reset timer if we just finished building an aircraft.
	if( m_nextAllowedProductionFrame <= now )
	{
		m_startedProductionFrame = FOREVER;
	}

	std::vector<FlightDeckInfo>::iterator it = m_spaces.begin();
	for( ; it != m_spaces.end(); it++ )
	{
		//Unassigned space?... so we can build a replacement.
		if( it->m_objectInSpace == INVALID_ID )
		{
			//But are we already building one?
			ProductionUpdateInterface *pu = getObject()->getProductionUpdateInterface();
			if( pu == nullptr )
			{

				DEBUG_CRASH( ("MSG_QUEUE_UNIT_CREATE: Producer '%s' doesn't have a unit production interface", getObject()->getTemplate()->getName().str()) );
				break;
			}
			DEBUG_ASSERTCRASH( m_thingTemplate != nullptr, ("flightdeck has a null thingtemplate... no jets for you!") );
			if( !pu->getProductionCount() && now >= m_nextAllowedProductionFrame && m_thingTemplate != nullptr )
			{
				//Queue the build
				pu->queueCreateUnit( m_thingTemplate, pu->requestUniqueUnitID() );

				m_startedProductionFrame = now;
				m_nextAllowedProductionFrame = now + data->m_replacementFrames + data->m_dockAnimationFrames;
			}

			break;
		}
	}

	//If the carrier has at least one aircraft, then allow it to attack.
	Bool hasAircraft = FALSE;
	for( it = m_spaces.begin(); it != m_spaces.end(); it++ )
	{
		if( it->m_objectInSpace != INVALID_ID )
		{
			hasAircraft = TRUE;
			break;
		}
	}
	getObject()->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_NO_ATTACK ), !hasAircraft );
	Drawable *draw = getObject()->getDrawable();

	//Check for timer expiry -- are we allowed to launch the next wave yet?
	for( int i = 0; i < data->m_numCols; i++ )
	{
		Object *jet = TheGameLogic->findObjectByID( m_spaces[ i ].m_objectInSpace );
		if( jet && !isAbleToGiveUpParkingSpace( jet ) && isInPositionToTakeoff( *jet ) && hasTakeoffOrders() )
		{
			if( m_nextLaunchWaveFrame[ i ] <= now )
			{
				//Handle making the ramp going up and holding the launch until it's completely up.
				if( !m_rampUp[ i ] )
				{
					m_rampUp[ i ] = TRUE;
					m_rampUpFrame[ i ] = now + data->m_launchRampFrames;
					m_lowerRampFrame[ i ] = FOREVER;
					//****MAX_RUNWAYS***** if defined beyond 3, then this code will need to be rewritten or more modelcondition flags will
					//                     need to be added.
					ModelConditionFlagType opening = (ModelConditionFlagType)(MODELCONDITION_DOOR_2_OPENING + i * NUM_MODELCONDITION_DOOR_STATES);
					ModelConditionFlagType closing = (ModelConditionFlagType)(MODELCONDITION_DOOR_2_CLOSING + i * NUM_MODELCONDITION_DOOR_STATES);
					draw->clearAndSetModelConditionState( closing, opening );
				}

				//Handle launching the wave of fighters.
				if( m_rampUp[ i ] && m_rampUpFrame[ i ] <= now )
				{
					AIUpdateInterface *jetAI = jet->getAI();
					if( jetAI )
					{
						propagateOrderToSpecificPlane( jet );
						m_nextLaunchWaveFrame[ i ] = now + data->m_launchWaveFrames;
						m_catapultSystemFrame[ i ] = now + data->m_catapultFireFrames;
						m_lowerRampFrame[ i ] = now + data->m_lowerRampFrames;
					}
				}
			}
		}

		//Handle firing of the catapult steam effect upon launching planes.
		if( m_catapultSystemFrame[ i ] <= now && data->m_runwayInfo[ i ].m_catapultParticleSystem )
		{
			ParticleSystem *ps = TheParticleSystemManager->createParticleSystem( data->m_runwayInfo[ i ].m_catapultParticleSystem );
			m_catapultSystemFrame[ i ] = FOREVER;
			if( ps )
			{
				ps->setLocalTransform( &m_runways[ i ].m_startTransform );
				ps->setPosition( &m_runways[ i ].m_start );
			}
		}

		//Handle lowering the ramp after the fighter has been launched.
		if( m_rampUp[ i ] && m_lowerRampFrame[ i ] <= now )
		{
			m_rampUp[ i ] = FALSE;
			//****MAX_RUNWAYS***** if defined beyond 3, then this code will need to be rewritten or more modelcondition flags will
			//                     need to be added.
			ModelConditionFlagType opening = (ModelConditionFlagType)(MODELCONDITION_DOOR_2_OPENING + i * NUM_MODELCONDITION_DOOR_STATES);
			ModelConditionFlagType closing = (ModelConditionFlagType)(MODELCONDITION_DOOR_2_CLOSING + i * NUM_MODELCONDITION_DOOR_STATES);
			draw->clearAndSetModelConditionState( opening, closing );
		}
	}
	return UPDATE_SLEEP_NONE;
}

//-------------------------------------------------------------------------------------------------
ExitDoorType FlightDeckBehavior::reserveDoorForExit( const ThingTemplate* objType, Object *specificObject )
{
	//Uses the same door for all production.
	return DOOR_1;
}

//-------------------------------------------------------------------------------------------------
void FlightDeckBehavior::exitObjectViaDoor( Object *newObj, ExitDoorType exitDoor ) ///< Here is the thing I want you to exit
{
	FlightDeckInfo* ppi = nullptr;
	if (exitDoor != DOOR_NONE_NEEDED)
	{
		for (std::vector<FlightDeckInfo>::iterator it = m_spaces.begin(); it != m_spaces.end(); ++it)
		{
			if( it->m_objectInSpace == INVALID_ID )
			{
				ppi = &(*it);
				break;
			}
		}
	}

	if (!ppi)
	{
		DEBUG_CRASH(("could not find the space. what?"));
		return;
	}

	ppi->m_objectInSpace = newObj->getID();
	//validateAssignments();

	/// @todo srj -- this is evil. fix.
	static NameKeyType jetKey = TheNameKeyGenerator->nameToKey( "JetAIUpdate" );
	JetAIUpdate* ju = (JetAIUpdate *)newObj->findUpdateModule( jetKey );
	Real parkingOffset = ju ? ju->friend_getParkingOffset() : 0.0f;

	PPInfo ppinfo;
	Matrix3D mtx;

	DUMPMATRIX3D(getObject()->getTransformMatrix());
	DUMPCOORD3D(getObject()->getPosition());
	CRCDEBUG_LOG(("Produced at hangar (door = %d)", exitDoor));
	DEBUG_ASSERTCRASH(exitDoor != DOOR_NONE_NEEDED, ("Hmm, unlikely"));
	if (!reserveSpace(newObj->getID(), parkingOffset, &ppinfo)) //&loc, &orient, nullptr, nullptr, nullptr, nullptr, &hangarInternal, &hangOrient))
	{
		DEBUG_CRASH(("no spaces available, how did we get here?"));
		ppinfo.parkingSpace = *getObject()->getPosition();
		ppinfo.parkingOrientation = getObject()->getOrientation();
	}

	const std::vector<Coord3D> *pCreationLocations = getCreationLocations( newObj->getID() );
	if( !pCreationLocations )
	{
		DEBUG_CRASH( ("No creation locations specified for runway for FlightDeckBehavior (Kris).") );
		return;
	}

	newObj->setPosition( &(*pCreationLocations)[0] );
	newObj->setOrientation( m_runways[ ppi->m_runway ].m_startOrient );
	TheAI->pathfinder()->addObjectToPathfindMap( newObj );

	AIUpdateInterface  *ai = newObj->getAIUpdateInterface();
	if( ai )
	{
		std::vector<Coord3D> exitPath;
		//Doesn't matter what we push in now, the JetOrHeliTaxiState::onEnter() will nuke it
		//and calculate the taxi points to move through.
		exitPath.push_back( ppi->m_prep );
		ai->aiFollowExitProductionPath( &exitPath, getObject(), CMD_FROM_AI );
	}

}

//-------------------------------------------------------------------------------------------------
void FlightDeckBehavior::unreserveDoorForExit( ExitDoorType exitDoor )
{
	//Aircraft carrier doesn't use the door reservation system.
}

//-------------------------------------------------------------------------------------------------
void FlightDeckBehavior::aiDoCommand(const AICommandParms* parms)
{
	//Inspect the command and reset everything when necessary.
	if( parms->m_cmdSource != CMD_FROM_AI )
	{
		//Now the only time we care about anything is if we were ordered to attack something or attack move.
		switch( parms->m_cmd )
		{
			case AICMD_GUARD_POSITION:
				m_designatedTarget = INVALID_ID;
				m_designatedPosition.set( parms->m_pos );
				m_designatedCommand = parms->m_cmd;
				propagateOrdersToPlanes();
				break;
			case AICMD_ATTACK_POSITION:
				m_designatedTarget = INVALID_ID;
				m_designatedPosition.set( parms->m_pos );
				m_designatedCommand = parms->m_cmd;
				propagateOrdersToPlanes();
				break;
			case AICMD_FORCE_ATTACK_OBJECT:
			case AICMD_ATTACK_OBJECT:
				m_designatedTarget = parms->m_obj ? parms->m_obj->getID() : INVALID_ID;
				m_designatedPosition.zero();
				m_designatedCommand = parms->m_cmd;
				propagateOrdersToPlanes();
				break;
			case AICMD_ATTACKMOVE_TO_POSITION:
				m_designatedTarget = INVALID_ID;
				m_designatedPosition.set( parms->m_pos );
				m_designatedCommand = parms->m_cmd;
				propagateOrdersToPlanes();
				break;
			case AICMD_IDLE:
				m_designatedTarget = INVALID_ID;
				m_designatedPosition.zero();
				m_designatedCommand = parms->m_cmd;
				propagateOrdersToPlanes();
				break;
			default:
				m_designatedCommand = AICMD_NO_COMMAND;
				break;
		}
	}

	//Do NOT extend the base class AI. The carrier is a dumb stump that only propagates orders
	//to it's aircraft.
	//AIUpdateInterface::aiDoCommand( parms );
}

//-------------------------------------------------------------------------------------------------
void FlightDeckBehavior::propagateOrdersToPlanes()
{
	//We just ordered the carrier to stop, so order all the planes that are out to return!
	for( std::vector<FlightDeckInfo>::iterator it = m_spaces.begin(); it != m_spaces.end(); it++ )
	{
		if( it->m_objectInSpace != INVALID_ID )
		{
			Object *jet = TheGameLogic->findObjectByID( it->m_objectInSpace );
			if( jet && isAbleToGiveUpParkingSpace( jet ) )
			{
				propagateOrderToSpecificPlane( jet );
			}
		}
	}
}

//-------------------------------------------------------------------------------------------------
Bool FlightDeckBehavior::hasTakeoffOrders()
{
	Object *target = TheGameLogic->findObjectByID( m_designatedTarget );
	switch( m_designatedCommand )
	{
		case AICMD_GUARD_POSITION:
			return TRUE;

		case AICMD_ATTACK_POSITION:
			return TRUE;

		case AICMD_ATTACKMOVE_TO_POSITION:
			return TRUE;

		case AICMD_FORCE_ATTACK_OBJECT:
		case AICMD_ATTACK_OBJECT:
			if( target )
			{
				return TRUE;
			}
			m_designatedCommand = AICMD_NO_COMMAND;
			m_designatedTarget = INVALID_ID;
			return FALSE;

		case AICMD_IDLE:
			return FALSE;

		default:
			return FALSE;
	}
}

//-------------------------------------------------------------------------------------------------
void FlightDeckBehavior::propagateOrderToSpecificPlane( Object *jet )
{
	if( jet )
	{
		AIUpdateInterface *ai = jet->getAI();
		if( ai )
		{
			Object *target = TheGameLogic->findObjectByID( m_designatedTarget );
			switch( m_designatedCommand )
			{
				case AICMD_GUARD_POSITION:
					ai->aiGuardPosition( &m_designatedPosition, GUARDMODE_NORMAL, CMD_FROM_AI );
					break;
				case AICMD_ATTACK_POSITION:
					ai->aiAttackPosition( &m_designatedPosition, NO_MAX_SHOTS_LIMIT, CMD_FROM_AI );
					break;
				case AICMD_FORCE_ATTACK_OBJECT:
				case AICMD_ATTACK_OBJECT:
					ai->aiForceAttackObject( target, NO_MAX_SHOTS_LIMIT, CMD_FROM_PLAYER );
					break;
				case AICMD_ATTACKMOVE_TO_POSITION:
					ai->aiAttackMoveToPosition( &m_designatedPosition, NO_MAX_SHOTS_LIMIT, CMD_FROM_AI );
					break;
				case AICMD_IDLE:
					ai->aiEnter( getObject(), CMD_FROM_AI );
					break;
			}
		}
	}
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void FlightDeckBehavior::crc( Xfer *xfer )
{

	// extend base class
	AIUpdateInterface::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void FlightDeckBehavior::xfer( Xfer *xfer )
{
	Int i;

	// version
	const XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	AIUpdateInterface::xfer( xfer );

	if( xfer->getXferMode() == XFER_LOAD )
	{
		// first, build our info, so it won't be overwritten later.
		buildInfo(FALSE); // False, because the planes are going to save themselves.  We don't re-create them
	}

	// spaces info count and data
	UnsignedByte spacesCount = m_spaces.size();
	xfer->xferUnsignedByte( &spacesCount );
	if( xfer->getXferMode() == XFER_SAVE )
	{

		// save all elements
		std::vector< FlightDeckInfo >::iterator it;
		for( it = m_spaces.begin(); it != m_spaces.end(); ++it )
		{
			// object in this space
			xfer->xferObjectID( &((*it).m_objectInSpace) ); // This is the one thing not regenerated by buildInfo
		}

	}
	else if( xfer->getXferMode() == XFER_LOAD )
	{
		ObjectID objectID;

		// read all elements
		std::vector< FlightDeckInfo >::iterator it;
		it = m_spaces.begin();
		for( i = 0; i < spacesCount; ++i )
		{

			// read object id
			xfer->xferObjectID( &objectID );

			// store in vector if the vector does indeed still have room for this entry
			if( it != m_spaces.end() )
			{
				(*it).m_objectInSpace = objectID;
				//validateAssignments();
				++it;
			}

		}

	}

	// runways count and info
	UnsignedByte runwaysCount = m_runways.size();
	xfer->xferUnsignedByte( &runwaysCount );
	if( xfer->getXferMode() == XFER_SAVE )
	{

		// save all elements
		std::vector< RunwayInfo >::iterator it;
		for( it = m_runways.begin(); it != m_runways.end(); ++it )
		{

			// save object ID
			xfer->xferObjectID( &((*it).m_inUseByForTakeoff ) );
			xfer->xferObjectID( &((*it).m_inUseByForLanding ) );

		}

	}
	else if( xfer->getXferMode() == XFER_LOAD )
	{
		// read all elements
		std::vector< RunwayInfo >::iterator it;
		it = m_runways.begin();
		for( i = 0; i < runwaysCount; ++i )
		{

			ObjectID inUseByForTakeoff, inUseByForLanding;
// Old?			Bool wasInLine;

			// read object ID
			xfer->xferObjectID( &inUseByForTakeoff );
			xfer->xferObjectID( &inUseByForLanding );
// Old?			xfer->xferBool( &wasInLine );

			// store in vector if the vector does indeed still have room for this entry
			if( it != m_runways.end() )
			{

				(*it).m_inUseByForTakeoff = inUseByForTakeoff;
				(*it).m_inUseByForLanding = inUseByForLanding;
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

	xfer->xferUnsignedInt( &m_nextHealFrame );
	xfer->xferUnsignedInt( &m_nextCleanupFrame );
	xfer->xferUnsignedInt( &m_startedProductionFrame );
	xfer->xferUnsignedInt( &m_nextAllowedProductionFrame );
	xfer->xferObjectID( &m_designatedTarget );
	Int commandType = (Int)m_designatedCommand;
	xfer->xferInt( &commandType );
	m_designatedCommand = (AICommandType)commandType;
	xfer->xferCoord3D( &m_designatedPosition );

	UnsignedInt maxRunways = MAX_RUNWAYS;
	xfer->xferUnsignedInt( &maxRunways ); //If we're loading, we'll overwrite it. If we're saving, we save it!

	for( i = 0; i < MAX_RUNWAYS; i++ )
	{
		if( maxRunways <= MAX_RUNWAYS )
		{
			//Save and load everything.
			xfer->xferUnsignedInt( &m_nextLaunchWaveFrame[ i ] );
			xfer->xferUnsignedInt( &m_rampUpFrame[ i ] );
			xfer->xferUnsignedInt( &m_catapultSystemFrame[ i ] );
			xfer->xferUnsignedInt( &m_lowerRampFrame[ i ] );
			// TheSuperHackers @bugfix bobtista 22/07/2026 Index the ramp state by runway instead of reading and writing one element past the array.
			xfer->xferBool( &m_rampUp[ i ] );
		}
		else
		{
			//We must be loading... so load filler but don't assign it.
			UnsignedInt dummyInt;
			Bool dummyBool;
			xfer->xferUnsignedInt( &dummyInt );
			xfer->xferUnsignedInt( &dummyInt );
			xfer->xferUnsignedInt( &dummyInt );
			xfer->xferUnsignedInt( &dummyInt );
			xfer->xferBool( &dummyBool );
		}
	}

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void FlightDeckBehavior::loadPostProcess()
{


	const FlightDeckBehaviorModuleData* data = getFlightDeckBehaviorModuleData();
	m_thingTemplate = TheThingFactory->findTemplate( data->m_thingTemplateName );


	// extend base class
	AIUpdateInterface::loadPostProcess();

	// no, this is bad.. it is NOT SAFE to call setWakeFrame from the xfer system. crap. (srj)
	// make sure we are awake... old save games let us sleep
	//setWakeFrame(getObject(), UPDATE_SLEEP_NONE);

}
```
