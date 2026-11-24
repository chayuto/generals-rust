# Translation Packet `PKT-296`: `SpawnBehavior`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-296`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/behavior/spawn_behavior.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (21):**
  - `Common/GameState.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/GameState.h`
  - `Common/INI.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/INI.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/ThingFactory.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ThingFactory.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameLogic/ExperienceTracker.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ExperienceTracker.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/DamageModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DamageModule.h`
  - `GameLogic/Module/DieModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DieModule.h`
  - `GameLogic/Module/SpawnBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpawnBehavior.h`
  - `GameLogic/Module/StealthUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/StealthUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `PreRTS.h`
- **Implemented Module Facets (4):**
  - `DamageModuleInterface* getDamage()`
  - `DieModuleInterface* getDie()`
  - `SpawnBehaviorInterface* getSpawnBehaviorInterface()`
  - `UpdateModuleInterface* getUpdate()`
- **INI `FieldParse` Table Entries (11):**
  - `"SpawnNumber" => INI::parseInt`
  - `"SpawnReplaceDelay" => INI::parseDurationUnsignedInt`
  - `"OneShot" => INI::parseBool`
  - `"CanReclaimOrphans" => INI::parseBool`
  - `"AggregateHealth" => INI::parseBool`
  - `"ExitByBudding" => INI::parseBool`
  - `"SpawnTemplateName" => INI::parseAsciiStringVectorAppend`
  - `"SpawnedRequireSpawner" => INI::parseBool`
  - `"PropagateDamageTypesToSlavesWhenExisting" => INI::parseDamageTypeFlags`
  - `"InitialBurst" => INI::parseInt`
  - `"SlavesHaveFreeWill" => INI::parseBool`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (13 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferBool(&m_initialBurstTimesInited)`
  3. `xferUnsignedInt(&m_initialBurstCountdown)`
  4. `xferAsciiString( &name )`
  5. `xferInt( &m_oneShotCountdown )`
  6. `xferInt( &m_framesToWait )`
  7. `xferInt( &m_firstBatchCount )`
  8. `xferSTLIntList( &m_replacementTimes )`
  9. `xferSTLObjectIDList( &m_spawnIDs )`
  10. `xferBool( &m_active )`
  11. `xferBool( &m_aggregateHealth )`
  12. `xferInt( &m_spawnCount )`
  13. `xferUnsignedInt( &m_selfTaskingSpawnCount )`
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpawnBehavior.h`
```cpp
// FILE: SpawnBehavior.h //////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, January 2002
//				 Colin Day, October 2002
// Desc:	 Behavior will create and monitor a group of spawned units and replace as needed
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

const Int SPAWN_UPDATE_RATE = LOGICFRAMES_PER_SECOND/2; ///< This is a low priority module that only needs to be called every this many frames

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/INI.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Module/DieModule.h"
#include "GameLogic/Module/DamageModule.h"

//-------------------------------------------------------------------------------------------------
class ThingTemplate;
enum CanAttackResult CPP_11(: Int);

//-------------------------------------------------------------------------------------------------
class SpawnBehaviorModuleData : public BehaviorModuleData
{
public:
	Int m_spawnNumberData;								///< How many spawn I maintain
	Int m_spawnStartNumberData;				  	///< How many spawn I start with
	Int m_spawnReplaceDelayData;					///< After this many frames, I can replace one
	Int m_initialBurst;						        ///< How many should I make immediately, ignoring the delay?
	Bool m_isOneShotData;									///< Do I just spawn once and go dormant?
	Bool m_canReclaimOrphans;							///< Can I reclaim orphans for purposes of spawning
	Bool m_aggregateHealth;								///< should I calc an offset for the healthbox, averaging all my spawn
	Bool m_exitByBudding;									///< do I create each new spawn atop an existing one?
	Bool m_spawnedRequireSpawner;					///< Spawned objects can only exist while the spawner (us) is alive and present
	Bool m_slavesHaveFreeWill;						///< Slaves with free will don't attack when parent attacks.
	DamageTypeFlags m_damageTypesToPropagateToSlaves;
	std::vector<AsciiString> m_spawnTemplateNameData;
	DieMuxData m_dieMuxData;

	SpawnBehaviorModuleData()
	{
		m_spawnNumberData = 0;
		m_spawnReplaceDelayData = 0;
		m_spawnStartNumberData = 0;
		m_initialBurst = 0;
		m_isOneShotData = FALSE;
		m_canReclaimOrphans = FALSE;
		m_aggregateHealth = FALSE;
		m_exitByBudding = FALSE;
		m_spawnTemplateNameData.clear();
		m_spawnedRequireSpawner = FALSE;
		m_slavesHaveFreeWill = FALSE;
	}

	static void buildFieldParse(MultiIniFieldParse& p)
	{
    BehaviorModuleData::buildFieldParse(p);
		static const FieldParse dataFieldParse[] =
		{
			{ "SpawnNumber",							INI::parseInt,										nullptr, offsetof( SpawnBehaviorModuleData, m_spawnNumberData ) },
			{ "SpawnReplaceDelay",				INI::parseDurationUnsignedInt,		nullptr, offsetof( SpawnBehaviorModuleData, m_spawnReplaceDelayData ) },
			{ "OneShot",									INI::parseBool,										nullptr, offsetof( SpawnBehaviorModuleData, m_isOneShotData ) },
			{ "CanReclaimOrphans",				INI::parseBool,										nullptr,	offsetof( SpawnBehaviorModuleData, m_canReclaimOrphans ) },
			{ "AggregateHealth",  				INI::parseBool,										nullptr, offsetof( SpawnBehaviorModuleData, m_aggregateHealth ) },
			{ "ExitByBudding",    				INI::parseBool,										nullptr, offsetof( SpawnBehaviorModuleData, m_exitByBudding ) },
			{ "SpawnTemplateName",				INI::parseAsciiStringVectorAppend,nullptr, offsetof( SpawnBehaviorModuleData, m_spawnTemplateNameData ) },
			{ "SpawnedRequireSpawner",		INI::parseBool,										nullptr,	offsetof( SpawnBehaviorModuleData, m_spawnedRequireSpawner ) },
			{ "PropagateDamageTypesToSlavesWhenExisting",   INI::parseDamageTypeFlags, nullptr, offsetof( SpawnBehaviorModuleData, m_damageTypesToPropagateToSlaves ) },
			{ "InitialBurst",				      INI::parseInt,						        nullptr, offsetof( SpawnBehaviorModuleData, m_initialBurst ) },
			{ "SlavesHaveFreeWill",				INI::parseBool,										nullptr, offsetof( SpawnBehaviorModuleData, m_slavesHaveFreeWill ) },
			{ nullptr, nullptr, nullptr, 0 }

		};
    p.add(dataFieldParse);
		p.add(DieMuxData::getFieldParse(), offsetof( SpawnBehaviorModuleData, m_dieMuxData ));
	}
};

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
class SpawnBehaviorInterface
{

public:

	virtual Bool maySpawnSelfTaskAI( Real maxSelfTaskersRatio ) = 0;
	virtual void onSpawnDeath( ObjectID deadSpawn, DamageInfo *damageInfo ) = 0;
	virtual Object* getClosestSlave( const Coord3D *pos ) = 0;
	virtual void orderSlavesToAttackTarget( Object *target, Int maxShotsToFire, CommandSourceType cmdSource ) = 0;
	virtual void orderSlavesToAttackPosition( const Coord3D *pos, Int maxShotsToFire, CommandSourceType cmdSource ) = 0;
	virtual CanAttackResult getCanAnySlavesAttackSpecificTarget( AbleToAttackType attackType, const Object *target, CommandSourceType cmdSource ) = 0;
	virtual CanAttackResult getCanAnySlavesUseWeaponAgainstTarget( AbleToAttackType attackType, const Object *victim, const Coord3D *pos, CommandSourceType cmdSource ) = 0;
	virtual Bool canAnySlavesAttack() = 0;
	virtual void orderSlavesToGoIdle( CommandSourceType cmdSource ) = 0;
	virtual void orderSlavesDisabledUntil( DisabledType type, UnsignedInt frame ) = 0;
	virtual void orderSlavesToClearDisabled( DisabledType type ) = 0;
	virtual void giveSlavesStealthUpgrade( Bool grantStealth ) = 0;
	virtual Bool areAllSlavesStealthed() const = 0;
	virtual void revealSlaves() = 0;
	virtual Bool doSlavesHaveFreedom() const = 0;
	virtual Int getSlaveCount() const = 0;
};

// ------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class SpawnBehavior : public UpdateModule,
											public SpawnBehaviorInterface,
											public DieModuleInterface,
											public DamageModuleInterface
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( SpawnBehavior, "SpawnBehavior" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( SpawnBehavior, SpawnBehaviorModuleData )

public:

	SpawnBehavior( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	// module methods
	static Int getInterfaceMask() { return (MODULEINTERFACE_UPDATE) | (MODULEINTERFACE_DIE) | (MODULEINTERFACE_DAMAGE); }
	virtual void onDelete() override;
	virtual UpdateModuleInterface *getUpdate() override { return this; }
	virtual DieModuleInterface *getDie() override { return this; }
	virtual DamageModuleInterface *getDamage() override { return this; }
	virtual SpawnBehaviorInterface* getSpawnBehaviorInterface() override { return this; }

	// update methods
	virtual UpdateSleepTime update() override;

	// die methods
	virtual void onDie( const DamageInfo *damageInfo ) override;

	// damage methods
	virtual void onDamage( DamageInfo *damageInfo ) override;
	virtual void onHealing( DamageInfo *damageInfo ) override { }
	virtual void onBodyDamageStateChange( const DamageInfo* damageInfo,
																				BodyDamageType oldState,
																				BodyDamageType newState) override { }

	// SpawnBehaviorInterface methods
	virtual Bool maySpawnSelfTaskAI( Real maxSelfTaskersRatio ) override;
	virtual void onSpawnDeath( ObjectID deadSpawn, DamageInfo *damageInfo ) override;	///< Something we spawned and set up to tell us it died just died.
	virtual Object* getClosestSlave( const Coord3D *pos ) override;
	virtual void orderSlavesToAttackTarget( Object *target, Int maxShotsToFire, CommandSourceType cmdSource ) override;
	virtual void orderSlavesToAttackPosition( const Coord3D *pos, Int maxShotsToFire, CommandSourceType cmdSource ) override;
	virtual CanAttackResult getCanAnySlavesAttackSpecificTarget( AbleToAttackType attackType, const Object *target, CommandSourceType cmdSource ) override;
	virtual CanAttackResult getCanAnySlavesUseWeaponAgainstTarget( AbleToAttackType attackType, const Object *victim, const Coord3D *pos, CommandSourceType cmdSource ) override;
	virtual Bool canAnySlavesAttack() override;
	virtual void orderSlavesToGoIdle( CommandSourceType cmdSource ) override;
	virtual void orderSlavesDisabledUntil( DisabledType type, UnsignedInt frame ) override;
	virtual void orderSlavesToClearDisabled( DisabledType type ) override;
	virtual void giveSlavesStealthUpgrade( Bool grantStealth ) override;
	virtual Bool areAllSlavesStealthed() const override;
	virtual void revealSlaves() override;
	virtual Bool doSlavesHaveFreedom() const override { return getSpawnBehaviorModuleData()->m_slavesHaveFreeWill; }
	virtual Int getSlaveCount() const override { return m_spawnCount; }

	// **********************************************************************************************
	// our own methods
	void stopSpawning();	///< Whoever owns this module may want to turn it off
	void startSpawning();	///< Whoever owns this module may want to turn it on

	void computeAggregateStates();
//	void notifySelfTasking( Bool isSelfTasking );

private:

	Bool shouldTryToSpawn(); ///< For my own use, should I even think of spawning
	Bool createSpawn();										///< Actual work of creating a guy

	const ThingTemplate* m_spawnTemplate;	///< What it is I spawn
	Int m_oneShotCountdown;						///< and if so, this is what "once" entails
	Int m_framesToWait;
	Int m_firstBatchCount;   ///<how many to start off with on the first Update();

	/// @todo Make sure the allocator for std::list<> is a good one.  Otherwise override it.
	typedef std::list<Int> intList;
	typedef std::list<Int>::iterator intListIterator;
	typedef std::list<Int>::reverse_iterator intListReverseIterator;
	typedef std::list<ObjectID> objectIDList;
	typedef std::list<ObjectID>::iterator objectIDListIterator;
	typedef std::list<ObjectID>::reverse_iterator objectIDListReverseIterator;

	intList m_replacementTimes;			///< A list of frame times that I need to create new spawns
	objectIDList m_spawnIDs;				///< My darling little spawns.  I need to keep track of them explicitly for the Slave type stuff
	Bool m_active;									///< Am I currently turned on


	Object *reclaimOrphanSpawn();		///< find existing orphaned spawn object if present

	Bool m_aggregateHealth;			///< should I calc an offset for the healthbox, averaging all my spawn
	Bool m_initialBurstTimesInited;
	Int m_spawnCount;						///< so I can track for zero = kill; (aggregate)
	UnsignedInt m_selfTaskingSpawnCount;		///< How many of my spawn have I authorized to do their own thing?

	UnsignedInt m_initialBurstCountdown;

	std::vector<AsciiString>::const_iterator m_templateNameIterator;

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Behavior/SpawnBehavior.cpp`
```cpp
// FILE: SpawnBehavior.cpp ////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, January 2002
// Desc:   Update will create and monitor a group of spawned units and replace as needed
///////////////////////////////////////////////////////////////////////////////////////////////////

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/GameState.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/Player.h"
#include "Common/Xfer.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/SpawnBehavior.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameClient/Drawable.h" //selection logic
#include "GameClient/InGameUI.h" // selection logic
#include "GameLogic/ExperienceTracker.h" //veterancy logic
#include "GameLogic/Module/StealthUpdate.h"


#define NONE_SPAWNED_YET (0xffffffff)





#define SPAWN_DELAY_MIN_FRAMES (16) // about as rapidly as you'd expect people to successively exit through the same door
//-------------------------------------------------------------------------------------------------
SpawnBehavior::SpawnBehavior( Thing *thing, const ModuleData* moduleData )
						 : UpdateModule( thing, moduleData )
{
	const SpawnBehaviorModuleData* md = getSpawnBehaviorModuleData();

	// GEE, THIS IS NEW...
	// NOW, WE CAN HAVE A LIST OF TEMPLATE NAMES
	m_templateNameIterator = md->m_spawnTemplateNameData.begin();
	m_spawnTemplate = TheThingFactory->findTemplate( *m_templateNameIterator );
	//each time m_spawn template is used, it will increment m_templateNameIterator,
	//thus scanning through the ordered list of template names
	//looping back to the beginning

	m_framesToWait = 0;
	m_firstBatchCount = 0;
	if( md->m_isOneShotData )
		m_oneShotCountdown = md->m_spawnNumberData;
	else
		m_oneShotCountdown = -1;

	m_active = TRUE;

	m_replacementTimes.clear();
	// The initializing of the initial bursters is handled in the first update @todo invent an object::postConstructionProcess() some day
	m_initialBurstCountdown = md->m_initialBurst;
	m_initialBurstTimesInited = FALSE;



	m_aggregateHealth = md->m_aggregateHealth;

	m_spawnCount = NONE_SPAWNED_YET;
	m_active = TRUE;
	m_selfTaskingSpawnCount = 0;
}

//-------------------------------------------------------------------------------------------------
SpawnBehavior::~SpawnBehavior()
{
	m_replacementTimes.clear();
}

// ------------------------------------------------------------------------------------------------
void SpawnBehavior::onDelete()
{
	const SpawnBehaviorModuleData *modData = getSpawnBehaviorModuleData();

	// destroy anything that we have spawned that is not already dead
	if( modData->m_spawnedRequireSpawner )
	{
		Object *obj;

		for( objectIDListIterator it = m_spawnIDs.begin(); it != m_spawnIDs.end(); /*empty*/ )
		{

			// get object
			obj = TheGameLogic->findObjectByID( *it );

			// increment iterator incase the id list is alterd because what we're about to do
			++it;

			//
			// destroy this if it's alive, in the usual case this object "dies" and is
			// not "destroyed".  on *death* we *KILL* our spawned things, but this is here
			// and will *DESTROY* our spawned things if we ourselves are destroyed and they
			// are still alive (such a case would be calling destroy object on us directly)
			//
			if( obj && obj->isEffectivelyDead() == FALSE )
				TheGameLogic->destroyObject( obj );

		}

	}

}

// ------------------------------------------------------------------------------------------------
void SpawnBehavior::onDie( const DamageInfo *damageInfo )
{
	const SpawnBehaviorModuleData *modData = getSpawnBehaviorModuleData();

	///@todo isDieApplicable should be called outside of the onDie call
	if( modData->m_dieMuxData.isDieApplicable( getObject(), damageInfo ) == FALSE )
		return;

	for( objectIDListIterator iter = m_spawnIDs.begin();
				iter != m_spawnIDs.end();
				iter++
					)
	{
		Object *currentSpawn = TheGameLogic->findObjectByID( (*iter) );
		if( currentSpawn )
		{
			// Go through all my spawns and see if they have a SlavedUpdate I can tell I was killed to
			for (BehaviorModule** update = currentSpawn->getBehaviorModules(); *update; ++update)
			{
				SlavedUpdateInterface* sdu = (*update)->getSlavedUpdateInterface();
				if (sdu != nullptr)
				{
					sdu->onSlaverDie( damageInfo );
					break;
				}
			}

			// our spawner has died, we must invalidate the ID now in the spawned object
			currentSpawn->setProducer( nullptr );

		}
	}

	// kill anything that we have spawned if our module data directs us to do so
	if( modData->m_spawnedRequireSpawner )
	{
		Object *obj;

		for( objectIDListIterator it = m_spawnIDs.begin(); it != m_spawnIDs.end(); /*emtpy*/ )
		{

			// get object
			obj = TheGameLogic->findObjectByID( *it );

			// increment iterator incase the id list is alterd because what we're about to do
			++it;

			// kill it if not already dead
			if( obj && obj->isEffectivelyDead() == FALSE )
				obj->kill();

		}

	}

}

//-------------------------------------------------------------------------------------------------
UpdateSleepTime SpawnBehavior::update()
{
/// @todo srj use SLEEPY_UPDATE here

	//EVERY FRAME
	if ( m_aggregateHealth )
	{
		computeAggregateStates();
	}

	const SpawnBehaviorModuleData* md = getSpawnBehaviorModuleData();
	if ( ! m_initialBurstTimesInited )
	{
		m_initialBurstTimesInited = TRUE;

		Bool runtimeProduced = getObject()->getProducerID()!=INVALID_ID; //this was produced by a production module, rather than a script or worldbuilder
		Int now = TheGameLogic->getFrame();
		Int burstInitCount = m_initialBurstCountdown;
		for( int listIndex = 0; listIndex < md->m_spawnNumberData; listIndex++ )
		{
			if ( md->m_initialBurst > 0 )
			{
				UnsignedInt birthFrame = now;
				if ( runtimeProduced && burstInitCount > 0)
				{
					--burstInitCount;
					birthFrame += (listIndex*SPAWN_DELAY_MIN_FRAMES);
				}
				m_replacementTimes.push_back( runtimeProduced  );// Set all spawns to be created in rapid succession
			}
			else
				m_replacementTimes.push_back( listIndex );// Set all spawns to be created as soon as possible
		}
	}



	//SPARSELY SOLVED
	if (--m_framesToWait > 0)
	{
		return UPDATE_SLEEP_NONE;
	}

	m_framesToWait = SPAWN_UPDATE_RATE;

	//Go through the list and make a spawn for each number that is less than now's frame
	if( shouldTryToSpawn() )
	{
		// Due to STL's list vomiting when you erase in a for loop,
		// Start at the beginning, and if you erase one, use the return
		// value to manually step to the next.  Otherwise, step by yourself.
		// begin == end means empty list, so no special check is needed.
		intListIterator iterator = m_replacementTimes.begin();
		while( iterator != m_replacementTimes.end() )
		{
			Int replacementTime = *iterator;
			UnsignedInt currentTime = TheGameLogic->getFrame();
			if( currentTime > replacementTime )
			{
				//If you create one, you pop the number off the list
				if( createSpawn() )
					iterator = m_replacementTimes.erase( iterator );
				else
					iterator++;
			}
			else
				iterator++;
		}

		if( md->m_isOneShotData  &&  (m_oneShotCountdown <= 0) )
			stopSpawning(); //I only trigger one batch.  ie on Creation.
	}
	return UPDATE_SLEEP_NONE;
}

// ------------------------------------------------------------------------------------------------
Bool SpawnBehavior::maySpawnSelfTaskAI( Real maxSelfTaskersRatio )
{
	if ( m_spawnCount == 0)
		return FALSE;
	if ( maxSelfTaskersRatio == 0)
		return FALSE;


	//if my last attack command was from player or script, I need to forbid my spawn from disobeying that command
	//otherwise (since my attack state was autoacquired my ny own ai), let them deviate by the ratio specified.
	Object* obj = getObject();
	if ( ! obj )
		return FALSE;
	AIUpdateInterface *ai = obj->getAI();
	if ( ! ai )
		return FALSE;

	CommandSourceType lastAttackCommandSource = ai->getLastCommandSource();


	if ( lastAttackCommandSource != CMD_FROM_AI )
		return FALSE;


	Real curSelfTaskersRatio = (Real)m_selfTaskingSpawnCount / (Real)m_spawnCount;

	return ( curSelfTaskersRatio < maxSelfTaskersRatio );
}

// ------------------------------------------------------------------------------------------------
Object* SpawnBehavior::getClosestSlave( const Coord3D *pos )
{
	Object *closest = nullptr;
	Real closestDistance;
	for( objectIDListIterator it = m_spawnIDs.begin(); it != m_spawnIDs.end(); ++it )
	{
		Object *obj = TheGameLogic->findObjectByID( *it );
		if( obj )
		{
			Real distance = ThePartitionManager->getDistanceSquared( obj, pos, FROM_CENTER_2D );

			if( !closest || closestDistance > distance )
			{
				closest = obj;
				closestDistance = distance;
			}
		}
	}
	return closest; //Could be null!
}

// ------------------------------------------------------------------------------------------------
void SpawnBehavior::orderSlavesToAttackTarget( Object *target, Int maxShotsToFire, CommandSourceType cmdSource )
{
	for( objectIDListIterator it = m_spawnIDs.begin(); it != m_spawnIDs.end(); ++it )
	{
		Object *obj = TheGameLogic->findObjectByID( *it );
		if( obj )
		{
			AIUpdateInterface *ai = obj->getAI();
			if( ai )
			{
				ai->aiForceAttackObject( target, maxShotsToFire, cmdSource );
			}
		}
	}
}

// ------------------------------------------------------------------------------------------------
void SpawnBehavior::orderSlavesToAttackPosition( const Coord3D *pos, Int maxShotsToFire, CommandSourceType cmdSource )
{
	for( objectIDListIterator it = m_spawnIDs.begin(); it != m_spawnIDs.end(); ++it )
	{
		Object *obj = TheGameLogic->findObjectByID( *it );
		if( obj )
		{
			AIUpdateInterface *ai = obj->getAI();
			if( ai )
			{
				ai->aiAttackPosition( pos, maxShotsToFire, cmdSource );
			}
		}
	}
}

// ------------------------------------------------------------------------------------------------
void SpawnBehavior::orderSlavesToGoIdle( CommandSourceType cmdSource )
{
	for( objectIDListIterator it = m_spawnIDs.begin(); it != m_spawnIDs.end(); ++it )
	{
		Object *obj = TheGameLogic->findObjectByID( *it );
		if( obj )
		{
			AIUpdateInterface *ai = obj->getAI();
			if( ai )
			{
				ai->aiIdle( cmdSource );
			}
		}
	}
}

// ------------------------------------------------------------------------------------------------
void SpawnBehavior::orderSlavesDisabledUntil( DisabledType type, UnsignedInt frame )
{
	for( objectIDListIterator it = m_spawnIDs.begin(); it != m_spawnIDs.end(); ++it )
	{
		Object *obj = TheGameLogic->findObjectByID( *it );
		if( obj )
		{
			AIUpdateInterface *ai = obj->getAI();
			if( ai )
			{
				ai->aiIdle( CMD_FROM_AI );
			}
			obj->setDisabledUntil( type, frame );
		}
	}
}

// ------------------------------------------------------------------------------------------------
void SpawnBehavior::orderSlavesToClearDisabled( DisabledType type )
{
	for( objectIDListIterator it = m_spawnIDs.begin(); it != m_spawnIDs.end(); ++it )
	{
		Object *obj = TheGameLogic->findObjectByID( *it );
		if( obj )
		{
			obj->clearDisabled( type );
		}
	}
}

// ------------------------------------------------------------------------------------------------
CanAttackResult SpawnBehavior::getCanAnySlavesAttackSpecificTarget( AbleToAttackType attackType, const Object *target, CommandSourceType cmdSource )
{
	Bool invalidShot = FALSE;
	for( objectIDListIterator it = m_spawnIDs.begin(); it != m_spawnIDs.end(); ++it )
	{
		Object *obj = TheGameLogic->findObjectByID( *it );
		if( obj )
		{
			CanAttackResult result = obj->getAbleToAttackSpecificObject( attackType, target, cmdSource );

			switch( result )
			{
				case ATTACKRESULT_POSSIBLE:
				case ATTACKRESULT_POSSIBLE_AFTER_MOVING:
					return result;

				case ATTACKRESULT_NOT_POSSIBLE:
					break;

				case ATTACKRESULT_INVALID_SHOT:
					invalidShot = TRUE;
					break;

				default:
					DEBUG_CRASH( ("SpawnBehavior::getCanAnySlavesAttackSpecificTarget encountered unhandled CanAttackResult of %d. Treating as not possible...", result) );
					break;
			}
		}
	}
	//Prioritize the reasonings!
	if( invalidShot )
	{
		return ATTACKRESULT_INVALID_SHOT;
	}
	return ATTACKRESULT_NOT_POSSIBLE;
}

// ------------------------------------------------------------------------------------------------
CanAttackResult SpawnBehavior::getCanAnySlavesUseWeaponAgainstTarget( AbleToAttackType attackType, const Object *victim, const Coord3D *pos, CommandSourceType cmdSource )
{
	Bool invalidShot = FALSE;
	for( objectIDListIterator it = m_spawnIDs.begin(); it != m_spawnIDs.end(); ++it )
	{
		Object *obj = TheGameLogic->findObjectByID( *it );
		if( obj )
		{
			CanAttackResult result = obj->getAbleToUseWeaponAgainstTarget( attackType, victim, pos, cmdSource );

			switch( result )
			{
				case ATTACKRESULT_POSSIBLE:
				case ATTACKRESULT_POSSIBLE_AFTER_MOVING:
					return result;

				case ATTACKRESULT_NOT_POSSIBLE:
					break;

				case ATTACKRESULT_INVALID_SHOT:
					invalidShot = TRUE;
					break;

				default:
					DEBUG_CRASH( ("SpawnBehavior::getCanAnySlavesUseWeaponAgainstTarget encountered unhandled CanAttackResult of %d. Treating as not possible...", result) );
					break;
			}
		}
	}
	//Prioritize the reasonings!
	if( invalidShot )
	{
		return ATTACKRESULT_INVALID_SHOT;
	}
	return ATTACKRESULT_NOT_POSSIBLE;
}

// ------------------------------------------------------------------------------------------------
void SpawnBehavior::giveSlavesStealthUpgrade( Bool grantStealth )
{
	for( objectIDListIterator it = m_spawnIDs.begin(); it != m_spawnIDs.end(); ++it )
	{
		Object *obj = TheGameLogic->findObjectByID( *it );
		if( obj )
		{
			obj->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_CAN_STEALTH ), grantStealth );
		}
	}
}

// ------------------------------------------------------------------------------------------------
Bool SpawnBehavior::canAnySlavesAttack()
{
	for( objectIDListIterator it = m_spawnIDs.begin(); it != m_spawnIDs.end(); ++it )
	{
		Object *obj = TheGameLogic->findObjectByID( *it );
		if( obj )
		{
			if( obj->isAbleToAttack() )
			{
				return true;
			}
		}
	}
	return false;
}


// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
class OrphanData
{

public:

	OrphanData();

	const ThingTemplate *m_matchTemplate;
	Object *m_source;
	Object *m_closest;
	Real m_closestDistSq;

};

#define BIG_DISTANCE 99999999.9f
// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
OrphanData::OrphanData()
{

	m_matchTemplate = nullptr;
	m_source = nullptr;
	m_closest = nullptr;
	m_closestDistSq = BIG_DISTANCE;

}

// ------------------------------------------------------------------------------------------------
static void findClosestOrphan( Object *obj, void *userData )
{
	OrphanData *orphanData = (OrphanData *)userData;

	// if template doesn't match do nothing
	if( obj->getTemplate()->isEquivalentTo( orphanData->m_matchTemplate ) == FALSE )
		return;

	// this object must be orphaned
	if( obj->getProducerID() != INVALID_ID )
		return;

	// is this the closest one so far
	Real distSq = ThePartitionManager->getDistanceSquared( orphanData->m_source, obj, FROM_CENTER_2D );
	if( distSq < orphanData->m_closestDistSq )
	{

		orphanData->m_closest = obj;
		orphanData->m_closestDistSq = distSq;

	}

}

// ------------------------------------------------------------------------------------------------
Object *SpawnBehavior::reclaimOrphanSpawn()
{
	Player *player = getObject()->getControllingPlayer();
	const SpawnBehaviorModuleData *md = getSpawnBehaviorModuleData();

	//
	// iterate all the objects my controlling player has and look for any orphaned things
	// that we would normally spawn, if found we'll just make it our own
	//
	// EVEN MORE NEW AND DIFFERENT
	// This block scans the list for matchTemplates
	//

	OrphanData orphanData;
	AsciiString prevName;
	for (std::vector<AsciiString>::const_iterator tempName = md->m_spawnTemplateNameData.begin();
			tempName != md->m_spawnTemplateNameData.end();
			++tempName)
	{
		if (prevName.compare(*tempName)) // the list may have redundancy, this will skip some of it
			continue;
		orphanData.m_matchTemplate = TheThingFactory->findTemplate( *tempName );
		orphanData.m_source = getObject();
		orphanData.m_closest = nullptr;
		orphanData.m_closestDistSq = BIG_DISTANCE;
		player->iterateObjects( findClosestOrphan, &orphanData );
		prevName = *tempName;
	}

	return orphanData.m_closest;
}

//-------------------------------------------------------------------------------------------------
Bool SpawnBehavior::createSpawn()
{
	Object *parent = getObject();
	const SpawnBehaviorModuleData *md = getSpawnBehaviorModuleData();

	ExitInterface* exitInterface = parent->getObjectExitInterface();
	if( exitInterface == nullptr )
	{
		DEBUG_ASSERTCRASH( exitInterface != nullptr, ("Something cannot have SpawnBehavior without an exit interface") );
		return FALSE;
	}

	ExitDoorType exitDoor = exitInterface->reserveDoorForExit(nullptr, nullptr);
	if (exitDoor == DOOR_NONE_AVAILABLE)
		return FALSE;

	Object *newSpawn = nullptr;

	// try to reclaim orphaned objects if possible
	Bool reclaimedOrphan = FALSE;
	if( md->m_canReclaimOrphans && md->m_isOneShotData == FALSE )
	{
		newSpawn = reclaimOrphanSpawn();
		if( newSpawn )
			reclaimedOrphan = TRUE;
	}

													// This assures that an orphan has not just been reclaimed,
	if( newSpawn == nullptr )	// and that we really want a new spawn, here.
	{
		m_spawnTemplate = TheThingFactory->findTemplate( *m_templateNameIterator );

		//newSpawn = TheThingFactory->newObject( m_spawnTemplate, parent->getControllingPlayer()->getDefaultTeam() );
		newSpawn = TheThingFactory->newObject( m_spawnTemplate, parent->getTeam() ); // just a little worried about this...

		// Count this unit towards our score.
#if RTS_GENERALS && defined(_MSC_VER) && _MSC_VER < 1300
		// TheSuperHackers @info Set m_onUnitProducedZeroInit to true so it can be used in AIPlayer::onUnitProduced.
		TheGameLogic->m_onUnitProducedZeroInit = true;
		newSpawn->getControllingPlayer()->onUnitCreated(parent, newSpawn);
		TheGameLogic->m_onUnitProducedZeroInit = false;
#else
		newSpawn->getControllingPlayer()->onUnitCreated(parent, newSpawn);
#endif

		// Gee, this is new...
		// Now, we can have a list of template names.
		// Each time m_spawn template is used, it will increment m_templateNameIterator,
		// thus scanning through the ordered list of template names, and
		// looping back to the beginning

		++m_templateNameIterator;
		if ( m_templateNameIterator == md->m_spawnTemplateNameData.end())
		{
			m_templateNameIterator = md->m_spawnTemplateNameData.begin();
		}
	}


	newSpawn->setProducer(parent);

	// If they have a SlavedUpdate, then I have to tell them who their daddy is from now on.
	for (BehaviorModule** update = newSpawn->getBehaviorModules(); *update; ++update)
	{
		SlavedUpdateInterface* sdu = (*update)->getSlavedUpdateInterface();
		if (sdu != nullptr)
		{
			sdu->onEnslave( parent );
			break;
		}
	}

	m_spawnIDs.push_back( newSpawn->getID() );

	if( reclaimedOrphan == FALSE )
	{
		if ( md->m_exitByBudding )
		{

			Bool barracksExitSuccess = FALSE;

			if ( m_initialBurstCountdown > 0 )
			{
				Object *barracks = TheGameLogic->findObjectByID( parent->getProducerID() );
				if ( barracks && barracks->isKindOf( KINDOF_STRUCTURE ) )
				{
					ExitInterface* barracksExitInterface = barracks->getObjectExitInterface();
					if ( barracksExitInterface )
					{
						ExitDoorType barracksDoor = barracksExitInterface->reserveDoorForExit(nullptr, nullptr);
						barracksExitInterface->exitObjectViaDoor( newSpawn, barracksDoor );
						newSpawn->setProducer(parent);//let parents producer exit him, but he thinks it was me
						--m_initialBurstCountdown;
						barracksExitSuccess = TRUE;
					}

				}

			}

			if ( ! barracksExitSuccess )
			{
				// find the closest spawn to the nexus...
				//there is probably a more elegant way to choose the budHost, but oh well
				Object *budHost = nullptr;
				Object *curSpawn = nullptr;
				Real tapeMeasure = 99999;
				Real closest = 999999.9f; // 1000 * 1000
				objectIDListIterator iter;
				for( iter = m_spawnIDs.begin(); iter != m_spawnIDs.end(); iter++)
				{
					curSpawn = TheGameLogic->findObjectByID( *iter );
					if ( curSpawn )
					{
						if (curSpawn == newSpawn )
							continue;

						tapeMeasure = ThePartitionManager->getDistanceSquared( curSpawn, parent, FROM_CENTER_2D );
						if ( tapeMeasure < closest )
						{
							closest = tapeMeasure;
							budHost = curSpawn;
						}
					}
				}
				exitInterface->exitObjectByBudding( newSpawn, budHost );// also handles the nullptr pointer okay

			}

		}
		else
		{
			exitInterface->exitObjectViaDoor( newSpawn, exitDoor );
		}

	}
	else
		exitInterface->unreserveDoorForExit( exitDoor );

	if( md->m_isOneShotData )
		m_oneShotCountdown--;


	if ( m_spawnCount == NONE_SPAWNED_YET )
	{
		m_spawnCount = 1;
	}
	else
	{
		++ m_spawnCount;
	}


	return TRUE;
}

//-------------------------------------------------------------------------------------------------
void SpawnBehavior::onSpawnDeath( ObjectID deadSpawn, DamageInfo *damageInfo )
{
	objectIDListIterator it = std::find(m_spawnIDs.begin(), m_spawnIDs.end(), deadSpawn);

	// If the iterator is at the end, we didn't find deadSpawn, so bail out.
	// Otherwise, bad crash stuff will happen.
	if (it == m_spawnIDs.end())
		return;

	//When one dies, you push (now + delay) as the time a new one should be made
	const SpawnBehaviorModuleData* md = getSpawnBehaviorModuleData();

	Int replacementTime = md->m_spawnReplaceDelayData + TheGameLogic->getFrame();
	m_replacementTimes.push_back( replacementTime );

	m_spawnIDs.erase( it );

	--m_spawnCount;

	if ( (m_spawnCount == 0) && m_aggregateHealth) // I'm dead without my spawn
	{
		Object *killer = TheGameLogic->findObjectByID(damageInfo->in.m_sourceID);
		if (killer != nullptr) {
			killer->scoreTheKill(getObject());
		}
		TheGameLogic->destroyObject(getObject());
		//getObject()->kill();
		return;
	}
}

//-------------------------------------------------------------------------------------------------
void SpawnBehavior::stopSpawning()
{
	m_active = FALSE;
}

//-------------------------------------------------------------------------------------------------
void SpawnBehavior::startSpawning()
{
	m_active = TRUE;
}

//-------------------------------------------------------------------------------------------------
/** When I become damaged */
// ------------------------------------------------------------------------------------------------
void SpawnBehavior::onDamage( DamageInfo *info )
{
	for( objectIDListIterator iter = m_spawnIDs.begin();
				iter != m_spawnIDs.end();
				iter++
					)
	{
		Object *currentSpawn = TheGameLogic->findObjectByID( (*iter) );
		if( currentSpawn )
		{
			// Go through all my spawns and see if they have a SlavedUpdate I can tell I was hurt to
			for (BehaviorModule** update = currentSpawn->getBehaviorModules(); *update; ++update)
			{
				SlavedUpdateInterface* sdu = (*update)->getSlavedUpdateInterface();
				if (sdu != nullptr)
				{
					sdu->onSlaverDamage( info );
					break;
				}
			}
		}
	}
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
Bool SpawnBehavior::shouldTryToSpawn()
{
	const SpawnBehaviorModuleData *modData = getSpawnBehaviorModuleData();

	// Not if we are turned off
	if( !m_active )
		return FALSE;
	if( getObject()->getStatusBits().test( OBJECT_STATUS_RECONSTRUCTING ) && modData->m_isOneShotData )
	{
		// If we are a Hole rebuild, not only should we not, but we should never ask again.
		stopSpawning();
		return FALSE;
	}
	// Not if we are under construction or being sold
	if( getObject()->testStatus(OBJECT_STATUS_UNDER_CONSTRUCTION) || getObject()->testStatus(OBJECT_STATUS_SOLD) )
		return FALSE;
	// Not if we are civilian controlled
	if( getObject()->isNeutralControlled() )
		return FALSE;

	return TRUE;
}

//********************************************************************
//* This function allows various object states to be either inherited
//* from the spawner to the spawn, or vice versa
//* Selection is distributed across all spawn and spawner at once
//* Health for the spawner is calc'd by aggregating the sum of all
//*   spawn health, and dividing by optimal health of full population
//*   (that is, the max spawn, SpawnBehaviorModuleData::m_spawnNumberData)
//*   at full health.
//* Veterancy is sucked out of any unit that has any and put into the
//* Spawner, scaled by 1/SpawnBehaviorModuleData::m_spawnNumberData;
//* The HealthBoxPosition (maybe to include moodicon, vet icon) is calc'd
//* as an average position of all the spawn.
//********************************************************************

void SpawnBehavior::computeAggregateStates()
{
	if ( ! m_aggregateHealth ) // sanity
		return;

	Object* obj = getObject();
	const SpawnBehaviorModuleData* md = getSpawnBehaviorModuleData();

	Int spawnCount = 0;
	Int spawnCountMax = md->m_spawnNumberData;
	Coord3D avgSpawnPos;

	avgSpawnPos.set(0,0,0);
	Real acrHealth = 0.0f;
	Real avgHealthMax = 0.0f;

	ExperienceTracker *expTracker = obj->getExperienceTracker();
	VeterancyLevel vetLevel = expTracker->getVeterancyLevel();

	Bool SomebodyIsSelected = FALSE;
	Bool SomebodyIsNotSelected = FALSE;

	Drawable *spawnDraw = nullptr;
	Object *currentSpawn = nullptr;

	WeaponBonusConditionFlags spawnWeaponBonus;


	m_selfTaskingSpawnCount = 0;

	for( objectIDListIterator iter = m_spawnIDs.begin(); iter != m_spawnIDs.end(); iter++)
	{
		currentSpawn = TheGameLogic->findObjectByID( (*iter) );

		if( currentSpawn )
		{
			//m_selfTaskingSpawnCount += ( currentSpawn->isSelf);

			for (BehaviorModule** update = currentSpawn->getBehaviorModules(); *update; ++update)
			{
				SlavedUpdateInterface* sdu = (*update)->getSlavedUpdateInterface();
				if (sdu != nullptr)
				{
					m_selfTaskingSpawnCount += ( sdu->isSelfTasking());
					break;
				}
			}


			// VETERANCY LEVEL *************************************
			VeterancyLevel spawnVetLevel = currentSpawn->getVeterancyLevel();
			if ( spawnVetLevel > vetLevel) // whoever has more, set the other guy
			{
				expTracker->setVeterancyLevel(spawnVetLevel);
			}
			else if ( spawnVetLevel < vetLevel) // whoever has more, set the other guy
			{
				currentSpawn->getExperienceTracker()->setVeterancyLevel(vetLevel);
			}

			spawnWeaponBonus = currentSpawn->getWeaponBonusCondition();

			avgSpawnPos.add(*currentSpawn->getPosition());

			BodyModuleInterface *body = currentSpawn->getBodyModule();
			acrHealth    += body->getHealth();
			avgHealthMax += body->getMaxHealth();

			spawnDraw = currentSpawn->getDrawable();

			if ( spawnDraw->isSelected() )
			{
				SomebodyIsSelected = TRUE;
			}
			else
			{
				SomebodyIsNotSelected = TRUE;
			}

			spawnCount++;
		}
	}



	// SELECTION STATE *****************************************
	// THIS LOGIC IS SIMPLE
	// IF ANY ONE OF MY SPAWN ARE SELECTED RIGHT NOW,
	// I JUST SET THEM ALL SELECTED
	// THIS WAY WE ARE ALWAYS ONE BIG HAPPY MOB
	// BUT I ONLY PROCEED IF THERE IS ONE AMONG ALL OF US WHO IS NOT SELECTED

	if ( SomebodyIsSelected && ( ! obj->getDrawable()->isSelected() || SomebodyIsNotSelected ) )
	{
		GameMessage *teamMsg = TheMessageStream->appendMessage( GameMessage::MSG_CREATE_SELECTED_GROUP );
		teamMsg->appendBooleanArgument( FALSE );// not creating new team so pass false

		if ( SomebodyIsNotSelected ) // lets select everybody
		{
			for( objectIDListIterator iter = m_spawnIDs.begin(); iter != m_spawnIDs.end(); iter++)
			{
				currentSpawn = TheGameLogic->findObjectByID( (*iter) );

				if( currentSpawn )
				{
					spawnDraw = currentSpawn->getDrawable();


					if ( ! spawnDraw->isSelected())
					{
						TheInGameUI->selectDrawable( spawnDraw );
						TheInGameUI->setDisplayedMaxWarning( FALSE );
						teamMsg->appendBooleanArgument( FALSE );// not creating new team so pass false
						teamMsg->appendObjectIDArgument( currentSpawn->getID() );
					}
				}
			}
		}
		// if somebody is selected then I sure need to be!
		if ( ! obj->getDrawable()->isSelected())
		{
			TheInGameUI->selectDrawable( obj->getDrawable() );
			TheInGameUI->setDisplayedMaxWarning( FALSE );
			teamMsg->appendBooleanArgument( FALSE );// not creating new team so pass false
			teamMsg->appendObjectIDArgument( obj->getID() );
		}

	}



	// HEALTH BOX POSITION *****************************
	// pick a centered, average spot to draw the health box
	avgSpawnPos.scale(1.0f / spawnCount);
	avgSpawnPos.sub(*obj->getPosition());
	obj->setHealthBoxOffset(avgSpawnPos);



	// HEALTH STATE *************************************
	// make my health an aggregate of all my spawns' healths
	if ( spawnCount )
	{
		avgHealthMax /= spawnCount;
		Real perfectTotalHealth = avgHealthMax * spawnCountMax;
		Real actualHealth  = acrHealth / perfectTotalHealth;
		obj->getBodyModule()->setInitialHealth(100.0f * actualHealth);
	}
	else
	{
		obj->getBodyModule()->setInitialHealth(0);// I been sick <
	}


	// HOUSEKEEPING *************************************
	//make sure no enemies are shooting at the nexus, since it doesn't 'exist'
	obj->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_MASKED ) );

}

//-------------------------------------------------------------------------------------------------
Bool SpawnBehavior::areAllSlavesStealthed() const
{
	Object *currentSpawn;

	for( std::list<ObjectID>::const_iterator iter = m_spawnIDs.begin(); iter != m_spawnIDs.end(); iter++)
	{
		currentSpawn = TheGameLogic->findObjectByID( (*iter) );
		if( currentSpawn )
		{
      const StealthUpdate *stealthUpdate = currentSpawn->getStealth();
			if( !stealthUpdate || !stealthUpdate->allowedToStealth( currentSpawn ) )
			{
				return FALSE;
			}
		}
	}

	return TRUE; //0 or more spawns are ALL stealthed... I suppose if you have NO spawns, then they are considered stealthed ;)
}

//-------------------------------------------------------------------------------------------------
void SpawnBehavior::revealSlaves()
{
	Object *currentSpawn;

	for( objectIDListIterator iter = m_spawnIDs.begin(); iter != m_spawnIDs.end(); iter++)
	{
		currentSpawn = TheGameLogic->findObjectByID( (*iter) );
		if( currentSpawn )
		{
			StealthUpdate *stealthUpdate = currentSpawn->getStealth();
			if( stealthUpdate )
			{
				stealthUpdate->markAsDetected();
			}
		}
	}
}


// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void SpawnBehavior::crc( Xfer *xfer )
{

	// extend base class
	BehaviorModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version
	* 2: Added m_initialBurstTimesInited to the save. jba.
	* 3: TheSuperHackers @bugfix Stubbjax 16/02/2026 Added m_initialBurstCountdown.
*/
// ------------------------------------------------------------------------------------------------
void SpawnBehavior::xfer( Xfer *xfer )
{
	AsciiString name;

	// version
#if RETAIL_COMPATIBLE_XFER_SAVE
	XferVersion currentVersion = 2;
#else
	XferVersion currentVersion = 3;
#endif
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	BehaviorModule::xfer( xfer );


	if (version >= 2) {
		xfer->xferBool(&m_initialBurstTimesInited);
	}

	if (version >= 3) {
		xfer->xferUnsignedInt(&m_initialBurstCountdown);
	}

	// spawn template
	name = m_spawnTemplate ? m_spawnTemplate->getName() : AsciiString::TheEmptyString;
	xfer->xferAsciiString( &name );
	if( xfer->getXferMode() == XFER_LOAD )
	{

		m_spawnTemplate = nullptr;
		if( name.isEmpty() == FALSE )
		{

			m_spawnTemplate = TheThingFactory->findTemplate( name );
			if( m_spawnTemplate == nullptr )
			{

				DEBUG_CRASH(( "SpawnBehavior::xfer - Unable to find template '%s'", name.str() ));
				throw SC_INVALID_DATA;

			}

		}

	}

	// one shot countdown
	xfer->xferInt( &m_oneShotCountdown );

	// frames to wait
	xfer->xferInt( &m_framesToWait );

	// first batch count
	xfer->xferInt( &m_firstBatchCount );

	// replacement times
	if( xfer->getXferMode() == XFER_LOAD )
		m_replacementTimes.clear();
	xfer->xferSTLIntList( &m_replacementTimes );

	// spawn ids
	xfer->xferSTLObjectIDList( &m_spawnIDs );

	// active
	xfer->xferBool( &m_active );

	// aggregate health
	xfer->xferBool( &m_aggregateHealth );

	// spawn count
	xfer->xferInt( &m_spawnCount );

	// self tasking spawn count
	xfer->xferUnsignedInt( &m_selfTaskingSpawnCount );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void SpawnBehavior::loadPostProcess()
{

	// extend base class
	BehaviorModule::loadPostProcess();

}
```
