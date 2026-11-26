# Translation Packet `PKT-354`: `ProductionUpdate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-354`
- **Work Package:** `WP-5.4` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/update/production_update.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (26):**
  - `Common/BitFlagsIO.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/BitFlagsIO.h`
  - `Common/BuildAssistant.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/BuildAssistant.h`
  - `Common/CRCDebug.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/CRCDebug.h`
  - `Common/GameState.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/GameState.h`
  - `Common/ModelState.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ModelState.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/Radar.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Radar.h`
  - `Common/ThingFactory.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ThingFactory.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Upgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Upgrade.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `Common/XferCRC.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/XferCRC.h`
  - `GameClient/ControlBar.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/ControlBar.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/Eva.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Eva.h`
  - `GameClient/GameText.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/GameText.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/CreateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CreateModule.h`
  - `GameLogic/Module/DieModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DieModule.h`
  - `GameLogic/Module/ParkingPlaceBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ParkingPlaceBehavior.h`
  - `GameLogic/Module/ProductionUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ProductionUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/ScriptEngine.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ScriptEngine.h`
  - `PreRTS.h`
- **Implemented Module Facets (2):**
  - `DieModuleInterface* getDie()`
  - `ProductionUpdateInterface* getProductionUpdateInterface()`
- **INI `FieldParse` Table Entries (8):**
  - `"MaxQueueEntries" => INI::parseInt`
  - `"NumDoorAnimations" => INI::parseInt`
  - `"DoorOpeningTime" => INI::parseDurationUnsignedInt`
  - `"DoorWaitOpenTime" => INI::parseDurationUnsignedInt`
  - `"DoorCloseTime" => INI::parseDurationUnsignedInt`
  - `"ConstructionCompleteDuration" => INI::parseDurationUnsignedInt`
  - `"QuantityModifier" => parseAppendQuantityModifier`
  - `"DisabledTypesToProcess" => DisabledMaskType::parseFromINI`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (23 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUnsignedShort( &productionCount )`
  3. `xferUser( &production->m_type, sizeof( ProductionType ) )`
  4. `xferAsciiString( &name )`
  5. `xferUser( &production->m_productionID, sizeof( ProductionID ) )`
  6. `xferReal( &production->m_percentComplete )`
  7. `xferInt( &production->m_framesUnderConstruction )`
  8. `xferInt( &production->m_productionQuantityTotal )`
  9. `xferInt( &production->m_productionQuantityProduced )`
  10. `xferInt( (Int*)&production->m_exitDoor )`
  11. `xferUser( &production->m_type, sizeof( ProductionType ) )`
  12. `xferAsciiString( &name )`
  13. `xferUser( &production->m_productionID, sizeof( ProductionID ) )`
  14. `xferReal( &production->m_percentComplete )`
  15. `xferInt( &production->m_framesUnderConstruction )`
  16. `xferInt( &production->m_productionQuantityTotal )`
  17. `xferInt( &production->m_productionQuantityProduced )`
  18. `xferInt( (Int*)&production->m_exitDoor )`
  19. `xferUser( &m_uniqueID, sizeof( ProductionID ) )`
  20. `xferUnsignedInt( &m_productionCount )`
  21. `xferUnsignedInt( &m_constructionCompleteFrame )`
  22. `xferUser( &m_doors, sizeof( DoorInfo ) * DOOR_COUNT_MAX )`
  23. `xferBool( &m_flagsDirty )`
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if !RETAIL_COMPATIBLE_CRC`
  - `#if RETAIL_COMPATIBLE_CRC`

## 2.5 Existing Rust Symbol Index (Use These Exact Paths)
- `zh_logic::modules::update::base_renerate_update`: `BaseRegenerateUpdateModuleData`, `BaseRegenerateUpdate`
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
- `zh_logic::modules::update::production_update`: `PRODUCTION_INVALID`, `PRODUCTION_UNIT`, `PRODUCTION_UPGRADE`, `PRODUCTION_ID_INVALID`, `QuantityModifier`, `ProductionUpdateModuleData`, `FACET_MASK`, `ProductionEntry`, `DoorInfo`, `ProductionUpdate`
- `zh_logic::modules::update::projectile_stream_update`: `MAX_PROJECTILE_STREAM`, `ProjectileStreamUpdateModuleData`, `ProjectileStreamUpdate`
- `zh_logic::modules::update::prone_update`: `ProneUpdateModuleData`, `ProneUpdate`
- `zh_logic::modules::update::radar_update`: `RadarUpdateModuleData`, `RadarUpdate`
- `zh_logic::modules::update::radius_decal_update`: `RadiusDecalUpdateModuleData`, `RadiusDecalUpdate`
- `zh_logic::modules::update::repair_dock_update`: `RepairDockUpdateModuleData`, `RepairDockUpdate`
- `zh_logic::modules::update::smart_bomb_target_homing_update`: `SmartBombTargetHomingUpdateModuleData`, `SmartBombTargetHomingUpdate`
- `zh_logic::modules::update::spy_vision_update`: `SpyVisionUpdateModuleData`, `SpyVisionUpdate`, `fn`
- `zh_logic::modules::update::supply_center_dock_update`: `SupplyCenterDockUpdateModuleData`, `SupplyCenterDockUpdate`
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
- `zh_logic::modules::behavior::instant_death_behavior`: `InstantDeathBehaviorModuleData`, `InstantDeathBehavior`
- `zh_logic::modules::behavior::jet_slow_death_behavior`: `JetSlowDeathBehaviorModuleData`, `JetSlowDeathBehavior`
- `zh_logic::modules::behavior::neuton_blast_behavior`: `NeutronBlastBehaviorModuleData`, `NeutronBlastBehavior`
- `zh_logic::modules::behavior::overcharge_behavior`: `OverchargeBehaviorModuleData`, `OverchargeBehavior`
- `zh_logic::modules::behavior::poisoned_behavior`: `PoisonedBehaviorModuleData`, `PoisonedBehavior`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ProductionUpdate.h`
```cpp
// FILE: ProductionUpdate.h ///////////////////////////////////////////////////////////////////////
// Author: Colin Day, March 2002
// Desc:   This module allows things to be "constructed" from a building
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "Common/ModelState.h"
#include "GameLogic/Module/DieModule.h"
#include "GameLogic/Module/UpdateModule.h"

// FORWARD REFERENCES //////////////////////////////////////////////////////////////////////////////
class ProductionEntry;
class ThingTemplate;
class UpgradeTemplate;

///////////////////////////////////////////////////////////////////////////////////////////////////
enum ProductionID CPP_11(: Int)
{
	PRODUCTIONID_INVALID = 0
};

enum ProductionType CPP_11(: Int)
{
	PRODUCTION_INVALID = 0,
	PRODUCTION_UNIT,
	PRODUCTION_UPGRADE
};

//-------------------------------------------------------------------------------------------------
/** A ProductionEntry is a single entry representing something that we are supposed to
	* produce */
//-------------------------------------------------------------------------------------------------
class ProductionEntry : public MemoryPoolObject
{

friend class ProductionUpdate;

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( ProductionEntry, "ProductionEntry" )

public:

	ProductionEntry();
	// virtual destructor provided by memory pool object

	/// query what kind of thing is being produced by this entry
	const ThingTemplate *getProductionObject() const { return m_objectToProduce; }

	/// query what kind of upgrade is being produced by this entry
	const UpgradeTemplate *getProductionUpgrade() const { return m_upgradeToResearch; }

	/// query the production type
	ProductionType getProductionType() const { return m_type; }

	/// how much progress is done on this entry
	Real getPercentComplete() const { return m_percentComplete; }

	/// get the unique (to the producer object) production ID
	ProductionID getProductionID() const { return m_productionID; }

	Int getProductionQuantity() const { return m_productionQuantityTotal; } //How many I try to make
	Int getProductionQuantityRemaining() const { return m_productionQuantityTotal - m_productionQuantityProduced; }//How many I have made

	void oneProductionSuccessful() { ++m_productionQuantityProduced; m_exitDoor = DOOR_NONE_AVAILABLE; }//increment, and mark door to re-reserve

	ExitDoorType getExitDoor() const { return m_exitDoor; }
	void setExitDoor(ExitDoorType exitDoor) { m_exitDoor = exitDoor; }

protected:

	ProductionType m_type;														///< production type
	union
	{
		const ThingTemplate *m_objectToProduce;					///< what we're going to produce
		const UpgradeTemplate *m_upgradeToResearch;			///< what upgrade we're researching
	};
	ProductionID m_productionID;											///< our very own production ID!
	Real m_percentComplete;														///< percent our construction is complete
	Int m_framesUnderConstruction;										///< counter for how many frames we've been under construction (incremented once per update)
	Int m_productionQuantityTotal;										///< it is now possible to construct multiple units simultaneously.
	Int m_productionQuantityProduced;									///< And we need to allow pausing within an entry, so we keep track of number of sub-successes
	ExitDoorType m_exitDoor;

	ProductionEntry *m_next;													///< next in list
	ProductionEntry *m_prev;													///< prev in list

};

//-------------------------------------------------------------------------------------------------
struct QuantityModifier
{
	AsciiString m_templateName;
	Int					m_quantity;
};

//-------------------------------------------------------------------------------------------------
class ProductionUpdateModuleData : public UpdateModuleData
{

public:
  Int														m_numDoorAnimations;						///< has a door animation(s) upon unit built and exit
	UnsignedInt										m_doorOpeningTime;							///< in frames, time it takes the door to open
	UnsignedInt										m_doorWaitOpenTime;							///< in frames, time we should leave the door open
	UnsignedInt										m_doorClosingTime;							///< in frames, time it takes to close the door
	UnsignedInt										m_constructionCompleteDuration; ///< in frames, how long we state in "construction complete" condition after making something
	std::vector<QuantityModifier>	m_quantityModifiers;						///< Quantity modifiers modify the number of specified object to created whenever produced.
  Int														m_maxQueueEntries;							///< max things that can be queued at once.
	DisabledMaskType							m_disabledTypesToProcess;

	ProductionUpdateModuleData();
	static void buildFieldParse(MultiIniFieldParse& p);
	static void parseAppendQuantityModifier( INI* ini, void *instance, void *store, const void *userData );
};

//-------------------------------------------------------------------------------------------------
enum CanMakeType CPP_11(: Int);

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
class ProductionUpdateInterface
{

public:

	virtual CanMakeType canQueueCreateUnit( const ThingTemplate *unitType ) const = 0;
	virtual CanMakeType canQueueUpgrade( const UpgradeTemplate *upgrade ) const = 0;

	virtual ProductionID requestUniqueUnitID() = 0;

	virtual Bool queueUpgrade( const UpgradeTemplate *upgrade ) = 0;
	virtual Bool cancelUpgrade( const UpgradeTemplate *upgrade ) = 0;
	virtual Bool isUpgradeInQueue( const UpgradeTemplate *upgrade ) const = 0;
	virtual UnsignedInt countUnitTypeInQueue( const ThingTemplate *unitType ) const = 0;

	virtual Bool queueCreateUnit( const ThingTemplate *unitType, ProductionID productionID ) = 0;
	virtual Bool cancelUnitCreate( ProductionID productionID ) = 0;
	virtual void cancelAllUnitsOfType( const ThingTemplate *unitType) = 0;

	virtual void cancelAndRefundAllProduction() = 0;

	virtual UnsignedInt getProductionCount() const = 0;

	virtual const ProductionEntry *firstProduction() const = 0;
	virtual const ProductionEntry *nextProduction( const ProductionEntry *p ) const = 0;

	virtual void setHoldDoorOpen(ExitDoorType exitDoor, Bool holdIt) = 0;

	//These functions keep track of the special power construction of a new building via a special power instead of standard production interface.
	//This was added for the sneak attack building functionality.
	virtual const CommandButton* getSpecialPowerConstructionCommandButton() const = 0;
	virtual void setSpecialPowerConstructionCommandButton( const CommandButton *commandButton ) = 0;

};

//-------------------------------------------------------------------------------------------------
/** Production modules do actual unit construction */
//-------------------------------------------------------------------------------------------------
class ProductionUpdate : public UpdateModule, public ProductionUpdateInterface, public DieModuleInterface
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( ProductionUpdate, "ProductionUpdate" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( ProductionUpdate, ProductionUpdateModuleData )

public:

	ProductionUpdate( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by MemoryPoolObject

	static Int getInterfaceMask() { return UpdateModule::getInterfaceMask() | (MODULEINTERFACE_DIE); }

	// Disabled conditions to process (AI will still process held status)
	virtual DisabledMaskType getDisabledTypesToProcess() const override { return getProductionUpdateModuleData()->m_disabledTypesToProcess; }

	virtual ProductionUpdateInterface* getProductionUpdateInterface() override { return this; }
	virtual DieModuleInterface* getDie() override { return this; }
	static ProductionUpdateInterface *getProductionUpdateInterfaceFromObject( Object *obj );

	virtual CanMakeType canQueueCreateUnit( const ThingTemplate *unitType ) const override;
	virtual CanMakeType canQueueUpgrade( const UpgradeTemplate *upgrade ) const override;

	/** this method is used to request a unique ID to assign to the production of a single
	unit.  It is unique to all units that can be created from this source object, but is
	not unique among multiple source objects */
	virtual ProductionID requestUniqueUnitID() override { ProductionID tmp = m_uniqueID; m_uniqueID = (ProductionID)(m_uniqueID+1); return tmp; }

	virtual Bool queueUpgrade( const UpgradeTemplate *upgrade ) override;				///< queue upgrade "research"
	virtual Bool cancelUpgrade( const UpgradeTemplate *upgrade ) override;				///< cancel upgrade "research"
	virtual Bool isUpgradeInQueue( const UpgradeTemplate *upgrade ) const override;		///< is the upgrade in our production queue already
	virtual UnsignedInt countUnitTypeInQueue( const ThingTemplate *unitType ) const override;  ///< count number of units with matching unit type in the production queue

	virtual Bool queueCreateUnit( const ThingTemplate *unitType, ProductionID productionID ) override;					///< queue unit to be produced
	virtual Bool cancelUnitCreate( ProductionID productionID ) override;		      ///< cancel construction of unit with matching production ID
	virtual void cancelAllUnitsOfType( const ThingTemplate *unitType) override;	///< cancel all production of type unitType

	virtual void cancelAndRefundAllProduction() override;									///< cancel and refund anything in the production queue

	virtual UnsignedInt getProductionCount() const override { return m_productionCount; }    ///< return # of things in the production queue

	// walking the production list from outside
	virtual const ProductionEntry *firstProduction() const override { return m_productionQueue; }
	virtual const ProductionEntry *nextProduction( const ProductionEntry *p ) const override { return p ? p->m_next : nullptr; }

	virtual void setHoldDoorOpen(ExitDoorType exitDoor, Bool holdIt) override;

	virtual UpdateSleepTime update() override;					///< the update

	//These functions keep track of the special power construction of a new building via a special power instead of standard production interface.
	//This was added for the sneak attack building functionality.
	virtual const CommandButton* getSpecialPowerConstructionCommandButton() const override { return m_specialPowerConstructionCommandButton; }
	virtual void setSpecialPowerConstructionCommandButton( const CommandButton *commandButton ) override { m_specialPowerConstructionCommandButton = commandButton; }

	// DieModuleInterface
	virtual void onDie( const DamageInfo *damageInfo ) override;

protected:

	void addToProductionQueue( ProductionEntry *production );				///< add to *END* of production queue list
	void removeFromProductionQueue( ProductionEntry *production );	///< remove production from the queue list

	Bool cancelUpgrade( ProductionEntry *production );
	Bool cancelUnitCreate( ProductionEntry *production );

	void updateDoors();														///< update the door behavior

	struct DoorInfo
	{
		UnsignedInt m_doorOpenedFrame;											///< for producer objects that have a door open/close animation when they make something
		UnsignedInt m_doorWaitOpenFrame;										///< frame we entered into wait open state
		UnsignedInt m_doorClosedFrame;											///< frame any door was closed on
		Bool				m_holdOpen;															///< if T, don't allow door to close
	};

	const CommandButton *m_specialPowerConstructionCommandButton; ///< In a mode to construct a specific building via a special power. (NO NEED TO SAVE DATA ON THIS FIELD)
	ProductionEntry*		m_productionQueue;							///< queue of things we want to build
	ProductionEntry*		m_productionQueueTail;					///< tail pointer for m_productionQueue
	ProductionID				m_uniqueID;											///< unique ID counter for producing units
	UnsignedInt					m_productionCount;							///< # of things in the production queue
	UnsignedInt					m_constructionCompleteFrame;		///< frame construction was complete on
	DoorInfo						m_doors[DOOR_COUNT_MAX];
	ModelConditionFlags m_clearFlags;										///< flags to clear from model
	ModelConditionFlags m_setFlags;											///< flags to set in model
	Bool								m_flagsDirty;										///< clearFlags/setFlags needs to be set into the model

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/ProductionUpdate.cpp`
```cpp
// FILE: ProductionUpdate.cpp /////////////////////////////////////////////////////////////////////
// Author: Colin Day, March 2002
// Desc:   This module allows things to be "constructed" from a building
///////////////////////////////////////////////////////////////////////////////////////////////////

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/BitFlagsIO.h"
#include "Common/BuildAssistant.h"
#include "Common/CRCDebug.h"
#include "Common/GameState.h"
#include "Common/Player.h"
#include "Common/Radar.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/Upgrade.h"
#include "Common/Xfer.h"
#include "Common/XferCRC.h"

#include "GameClient/ControlBar.h"
#include "GameClient/Drawable.h"
#include "GameClient/Eva.h"
#include "GameClient/GameText.h"
#include "GameClient/InGameUI.h"

#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/CreateModule.h"
#include "GameLogic/Module/ParkingPlaceBehavior.h"
#include "GameLogic/Module/ProductionUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/ScriptEngine.h"


// PUBLIC /////////////////////////////////////////////////////////////////////////////////////////

static const ModelConditionFlagType theOpeningFlags[] =
{
	MODELCONDITION_DOOR_1_OPENING,
	MODELCONDITION_DOOR_2_OPENING,
	MODELCONDITION_DOOR_3_OPENING,
	MODELCONDITION_DOOR_4_OPENING
};
static_assert(ARRAY_SIZE(theOpeningFlags) == DOOR_COUNT_MAX, "Incorrect array size");

static const ModelConditionFlagType theClosingFlags[] =
{
	MODELCONDITION_DOOR_1_CLOSING,
	MODELCONDITION_DOOR_2_CLOSING,
	MODELCONDITION_DOOR_3_CLOSING,
	MODELCONDITION_DOOR_4_CLOSING
};
static_assert(ARRAY_SIZE(theClosingFlags) == DOOR_COUNT_MAX, "Incorrect array size");

static const ModelConditionFlagType theWaitingOpenFlags[] =
{
	MODELCONDITION_DOOR_1_WAITING_OPEN,
	MODELCONDITION_DOOR_2_WAITING_OPEN,
	MODELCONDITION_DOOR_3_WAITING_OPEN,
	MODELCONDITION_DOOR_4_WAITING_OPEN
};
static_assert(ARRAY_SIZE(theWaitingOpenFlags) == DOOR_COUNT_MAX, "Incorrect array size");

static const ModelConditionFlagType theWaitingToCloseFlags[] =
{
	MODELCONDITION_DOOR_1_WAITING_TO_CLOSE,
	MODELCONDITION_DOOR_2_WAITING_TO_CLOSE,
	MODELCONDITION_DOOR_3_WAITING_TO_CLOSE,
	MODELCONDITION_DOOR_4_WAITING_TO_CLOSE
};
static_assert(ARRAY_SIZE(theWaitingToCloseFlags) == DOOR_COUNT_MAX, "Incorrect array size");

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ProductionUpdateModuleData::ProductionUpdateModuleData()
{
	// someday, might need separate times for each door. but not yet.
	m_numDoorAnimations = 0;
	m_doorOpeningTime = 0;
	m_doorWaitOpenTime = 0;
	m_doorClosingTime = 0;
	m_constructionCompleteDuration = 0;
	m_quantityModifiers.clear();
	m_maxQueueEntries = 9;
	m_disabledTypesToProcess = MAKE_DISABLED_MASK(DISABLED_HELD);
}

//-------------------------------------------------------------------------------------------------
/*static*/ void ProductionUpdateModuleData::parseAppendQuantityModifier( INI* ini, void *instance, void *store, const void* /*userData*/ )
{
	ProductionUpdateModuleData* data = (ProductionUpdateModuleData*)instance;
	const char* name = ini->getNextToken();
	const char* countStr = ini->getNextTokenOrNull();
	Int count = countStr ? INI::scanInt(countStr) : 1;

	QuantityModifier qm;
	qm.m_quantity = count;
	qm.m_templateName.set( name );
	data->m_quantityModifiers.push_back( qm );
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
/*static*/ void ProductionUpdateModuleData::buildFieldParse(MultiIniFieldParse& p)
{
  UpdateModuleData::buildFieldParse( p );

	static const FieldParse dataFieldParse[] =
	{
		{ "MaxQueueEntries",	INI::parseInt, nullptr, offsetof( ProductionUpdateModuleData, m_maxQueueEntries ) },
		{ "NumDoorAnimations",	INI::parseInt, nullptr, offsetof( ProductionUpdateModuleData, m_numDoorAnimations ) },
		{ "DoorOpeningTime", INI::parseDurationUnsignedInt, nullptr, offsetof( ProductionUpdateModuleData, m_doorOpeningTime ) },
		{ "DoorWaitOpenTime", INI::parseDurationUnsignedInt, nullptr, offsetof( ProductionUpdateModuleData, m_doorWaitOpenTime ) },
		{ "DoorCloseTime", INI::parseDurationUnsignedInt, nullptr, offsetof( ProductionUpdateModuleData, m_doorClosingTime ) },
		{ "ConstructionCompleteDuration", INI::parseDurationUnsignedInt, nullptr, offsetof( ProductionUpdateModuleData, m_constructionCompleteDuration ) },
		{ "QuantityModifier",	parseAppendQuantityModifier, nullptr, offsetof( ProductionUpdateModuleData, m_quantityModifiers ) },
		{ "DisabledTypesToProcess",	DisabledMaskType::parseFromINI, nullptr, offsetof( ProductionUpdateModuleData, m_disabledTypesToProcess ) },
		{ nullptr, nullptr, nullptr, 0 }
	};
  p.add(dataFieldParse);

}

///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ProductionEntry::ProductionEntry()
{

	m_type = PRODUCTION_INVALID;
	m_objectToProduce = nullptr;
	m_upgradeToResearch = nullptr;
	m_productionID = (ProductionID)1;
	m_percentComplete = 0.0f;
	m_framesUnderConstruction = 0;
	m_next = nullptr;
	m_prev = nullptr;
	m_productionQuantityProduced = 0;
	m_productionQuantityTotal = 0;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ProductionEntry::~ProductionEntry()
{

}

///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ProductionUpdate::ProductionUpdate( Thing *thing, const ModuleData* moduleData ) :
									UpdateModule( thing, moduleData )
{

	m_productionQueue = nullptr;
	m_productionQueueTail = nullptr;
	m_productionCount = 0;
	m_uniqueID = (ProductionID)1;
	for (Int i = 0; i < DOOR_COUNT_MAX; ++i)
	{
		m_doors[i].m_doorOpenedFrame = 0;
		m_doors[i].m_doorWaitOpenFrame = 0;
		m_doors[i].m_doorClosedFrame = 0;
		m_doors[i].m_holdOpen = false;
	}
	m_constructionCompleteFrame = 0;
	m_clearFlags.clear();
	m_setFlags.clear();
	m_flagsDirty = FALSE;
	m_specialPowerConstructionCommandButton = nullptr;

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ProductionUpdate::~ProductionUpdate()
{

	// destroy any queued productions
	ProductionEntry *production;
	while( m_productionQueue )
	{

		production = m_productionQueue;
		removeFromProductionQueue( production );
		// TheSuperHackers @fix Mauller 13/04/2025 Delete instance of production item
		deleteInstance(production);

	}

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
CanMakeType ProductionUpdate::canQueueUpgrade( const UpgradeTemplate *upgrade ) const
{
	if (m_productionCount >= getProductionUpdateModuleData()->m_maxQueueEntries)
		return CANMAKE_QUEUE_FULL;

	return CANMAKE_OK;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
CanMakeType ProductionUpdate::canQueueCreateUnit( const ThingTemplate *unitType ) const
{
	/// @todo srj -- this is horrible, but the "right" way to do it is to move
	// ProductionUpdate to be part of ParkingPlaceBehavior, which I don't currently
	// have time for...
	ParkingPlaceBehaviorInterface* pp = nullptr;
	for (BehaviorModule** i = getObject()->getBehaviorModules(); *i; ++i)
	{
		if ((pp = (*i)->getParkingPlaceBehaviorInterface()) != nullptr)
		{
			if (pp->shouldReserveDoorWhenQueued(unitType) && !pp->hasAvailableSpaceFor(unitType))
				return CANMAKE_PARKING_PLACES_FULL;
		}
	}

	if (m_productionCount >= getProductionUpdateModuleData()->m_maxQueueEntries)
		return CANMAKE_QUEUE_FULL;

	return CANMAKE_OK;

}

//-------------------------------------------------------------------------------------------------
/** Queue an upgrade to be produced */
//-------------------------------------------------------------------------------------------------
Bool ProductionUpdate::queueUpgrade( const UpgradeTemplate *upgrade )
{

	// sanity
	if( upgrade == nullptr )
		return FALSE;

	// get the player
	Player *player = getObject()->getControllingPlayer();

	// sanity check to make sure we can build this upgrade
	if( upgrade->getUpgradeType() == UPGRADE_TYPE_PLAYER &&
			TheUpgradeCenter->canAffordUpgrade( player, upgrade ) == FALSE )
		return FALSE;
	else if( upgrade->getUpgradeType() == UPGRADE_TYPE_OBJECT &&
					 (getObject()->hasUpgrade( upgrade ) == TRUE ||
					  getObject()->affectedByUpgrade( upgrade ) == FALSE) )
		return FALSE;

	// you cannot queue the production of an upgrade twice in this queue
	if( isUpgradeInQueue( upgrade ) == TRUE )
		return FALSE;

	// STOP cheaters by making sure they can actually build this
	if( !getObject()->canProduceUpgrade(upgrade) )
		return FALSE;

	//
	// you cannot queue a player upgrade production if you are producing one already somewhere else
	// (or that somewhere else could even possibly be here)
	//
	if( upgrade->getUpgradeType() == UPGRADE_TYPE_PLAYER &&
      (player->hasUpgradeComplete( upgrade ) || player->hasUpgradeInProduction( upgrade )) )
		return FALSE;

	if (m_productionCount >= getProductionUpdateModuleData()->m_maxQueueEntries)
	{
		DEBUG_CRASH(("Production Queue is full... how did we get here?"));
		return FALSE;
	}

	// take the cost for the build away from the player
	Money *money = player->getMoney();
	money->withdraw( upgrade->calcCostToBuild( player ) );

	// allocate a new production entry
	ProductionEntry *production = newInstance(ProductionEntry);

	// assign production entry data
	production->m_type = PRODUCTION_UPGRADE;
	production->m_upgradeToResearch = upgrade;
	production->m_productionID = PRODUCTIONID_INVALID;  // not needed for upgrades, you can only have one of
																	 // this type in the queue

	// tie to the end of the production queue
	addToProductionQueue( production );

	// add this upgrade as in progress in the player
	player->addUpgrade( upgrade, UPGRADE_STATUS_IN_PRODUCTION );



	return TRUE;  // queued

}

//-------------------------------------------------------------------------------------------------
/** Cancel an upgrade being produced here */
//-------------------------------------------------------------------------------------------------
Bool ProductionUpdate::cancelUpgrade( const UpgradeTemplate *upgrade )
{
	// sanity
	if( upgrade == nullptr )
		return FALSE;

	//
	// find the production entry for this upgrade in the queue here, there can only be one
	// of this type in the queue
	//
	ProductionEntry *production;
	for( production = m_productionQueue; production; production = production->m_next )
	{
		if( production->m_type == PRODUCTION_UPGRADE &&
				production->m_upgradeToResearch == upgrade )
		{
			return cancelUpgrade( production );
		}

	}

	return FALSE;
}

//-------------------------------------------------------------------------------------------------
/** Queue the production of a unit.  Returns TRUE if unit was added to queue, FALSE if it
	* was not */
//-------------------------------------------------------------------------------------------------
Bool ProductionUpdate::queueCreateUnit( const ThingTemplate *unitType, ProductionID productionID )
{
	const ProductionUpdateModuleData *data = getProductionUpdateModuleData();

	// if we can't create the unit do nothing
	if( TheBuildAssistant->canMakeUnit( getObject(), unitType ) != CANMAKE_OK )
		return FALSE;

	ExitDoorType exitDoor = DOOR_NONE_AVAILABLE;

	/// @todo srj -- this is horrible, but the "right" way to do it is to move
	// ProductionUpdate to be part of ParkingPlaceBehavior, which I don't currently
	// have time for...
	ParkingPlaceBehaviorInterface* pp = nullptr;
	for (BehaviorModule** i = getObject()->getBehaviorModules(); *i; ++i)
	{
		if ((pp = (*i)->getParkingPlaceBehaviorInterface()) != nullptr)
		{
			if (pp->shouldReserveDoorWhenQueued(unitType))
			{
				ExitInterface* exitInterface = getObject()->getObjectExitInterface();
				if (exitInterface)
				{
					exitDoor = exitInterface->reserveDoorForExit(unitType, nullptr);
				}
				if (exitDoor == DOOR_NONE_AVAILABLE)
				{
					return FALSE;
				}
				break;
			}
		}
	}

	if (m_productionCount >= getProductionUpdateModuleData()->m_maxQueueEntries)
	{
		DEBUG_CRASH(("Production Queue is full... how did we get here?"));
		return FALSE;
	}

	// take the cost for the build away from the player
	Player *player = getObject()->getControllingPlayer();
	Money *money = player->getMoney();
	money->withdraw( unitType->calcCostToBuild( player ) );

	// allocate a new production entry
	ProductionEntry *production = newInstance(ProductionEntry);

	// Check to see if there is a quantity modifier entry. What this means is if we are building a particular unit
	// and it has a modifier, we build that number of objects instead of just one. A good example is the Chinese
	// barracks building redguards -- they are built 4 at a time, but the user or script only pays for one and
	// builds four for that price!
	production->m_productionQuantityTotal = 1;
	production->m_productionQuantityProduced = 0;
	for( std::vector<QuantityModifier>::const_iterator it = data->m_quantityModifiers.begin(); it != data->m_quantityModifiers.end(); ++it )
  {
		const ThingTemplate* productionTemplate = TheThingFactory->findTemplate( it->m_templateName );
		if( productionTemplate && productionTemplate->isEquivalentTo( unitType ) )
		{
			production->m_productionQuantityTotal = it->m_quantity;
			break;
		}
	}

	// assign production entry data
	production->m_type = PRODUCTION_UNIT;
	production->m_objectToProduce = unitType;
	production->m_productionID = productionID;
	production->m_exitDoor = exitDoor;

	// tie to the end of the production queue
	addToProductionQueue( production );

	return TRUE;  // unit queued

}

//-------------------------------------------------------------------------------------------------
/** Cancel the construction of the unit with the matching production ID */
//-------------------------------------------------------------------------------------------------
Bool ProductionUpdate::cancelUnitCreate( ProductionID productionID )
{

	// search for the production entry in our queue
	ProductionEntry *production;
	for( production = m_productionQueue; production; production = production->m_next )
	{
		// are we at the one we want get rid of it
		if( production->m_productionID == productionID )
		{
			return cancelUnitCreate( production );
		}
	}

	return FALSE;

}

//-------------------------------------------------------------------------------------------------
/** Cancel all production of type unitType */
//-------------------------------------------------------------------------------------------------
void ProductionUpdate::cancelAllUnitsOfType( const ThingTemplate *unitType)
{

	// search for the production entry in our queue
	ProductionEntry *production;
	for( production = m_productionQueue; production; )
	{

		// are we at the one we want get rid of it
		if( production->getProductionType() == PRODUCTION_UNIT &&
		    production->getProductionObject() == unitType )
		{
			ProductionEntry *temp = production->m_next;

			// cancel the production
			cancelUnitCreate( production );

			// advance
			production = temp;

		}
		else
		{

			// advance
			production = production->m_next;

		}

	}

}

//-------------------------------------------------------------------------------------------------
/** Update the door behavior */
//-------------------------------------------------------------------------------------------------
void ProductionUpdate::updateDoors()
{
	const ProductionUpdateModuleData *d = getProductionUpdateModuleData();

	UnsignedInt now = TheGameLogic->getFrame();
	for (Int i = 0; i < DOOR_COUNT_MAX; ++i)
	{
		//
		// if we have an open door, see if enough time has passed for us to close it ... likewise
		// if we were closing a door we will clear the closing condition after an amount of time
		// has passed
		//
		if( m_doors[i].m_doorOpenedFrame )
		{

			if( now - m_doors[i].m_doorOpenedFrame > d->m_doorOpeningTime )
			{

				// set our frame markers for door states
				m_doors[i].m_doorOpenedFrame = 0;
				m_doors[i].m_doorWaitOpenFrame = now;

				// set the flags that will show the difference in the model
				m_clearFlags.set( theOpeningFlags[i], true );
				m_setFlags.set( theOpeningFlags[i], false );

				m_setFlags.set( theWaitingOpenFlags[i] );
				m_flagsDirty = TRUE;

			}

		}
		else if( m_doors[i].m_doorWaitOpenFrame )
		{

			if( now - m_doors[i].m_doorWaitOpenFrame > d->m_doorWaitOpenTime && !m_doors[i].m_holdOpen )
			{

				// set our frame marker for closing
				m_doors[i].m_doorWaitOpenFrame = 0;
				m_doors[i].m_doorClosedFrame = now;

				// set the flags that show the difference in the art
				m_clearFlags.set( theWaitingOpenFlags[i], true );
				m_setFlags.set( theWaitingOpenFlags[i], false );

				m_setFlags.set( theClosingFlags[i] );
				m_flagsDirty = TRUE;

			}

		}
		else if( m_doors[i].m_doorClosedFrame && !m_doors[i].m_holdOpen )
		{

			if( now - m_doors[i].m_doorClosedFrame > d->m_doorClosingTime )
			{

				// set our frame marker for the closed door frame
				m_doors[i].m_doorClosedFrame = 0;

				// set the flags that will show the difference in the model
				m_clearFlags.set( theClosingFlags[i], true );
				m_setFlags.set( theClosingFlags[i], false );
				m_flagsDirty = TRUE;

			}

		}
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
UpdateSleepTime ProductionUpdate::update()
{
/// @todo srj use SLEEPY_UPDATE here
	ProductionEntry *production = m_productionQueue;
	const ProductionUpdateModuleData *d = getProductionUpdateModuleData();
	UnsignedInt now = TheGameLogic->getFrame();
	Object *us = getObject();

	// update the door behaviors
	if( d->m_numDoorAnimations > 0 )
		updateDoors();

	//
	// if we're in the construction complete state ... we need to check frame
	// we're in it and get out of it when the time is up
	//
	if( m_constructionCompleteFrame )
	{

		if( now - m_constructionCompleteFrame > d->m_constructionCompleteDuration )
		{

			// set our frame marker to be out of construction complete state
			m_constructionCompleteFrame = 0;

			// set the flags that show the difference in the model
			m_clearFlags.set( MODELCONDITION_CONSTRUCTION_COMPLETE, true );
			m_setFlags.set( MODELCONDITION_CONSTRUCTION_COMPLETE, false );
			m_flagsDirty = TRUE;

		}

	}

	// if we have dirty bits that need to be set and cleared, do it here all at once
	if( m_flagsDirty == TRUE )
	{
		Drawable *draw = us->getDrawable();

		if( draw )
			draw->clearAndSetModelConditionFlags( m_clearFlags, m_setFlags );

		// clear the things we just set and turn off our dirty flag
		m_clearFlags.clear();
		m_setFlags.clear();
		m_flagsDirty = FALSE;

	}

	// if nothing in the queue get outta here
	if( production == nullptr )
		return UPDATE_SLEEP_NONE;

	//
	// if we've become OBJECT_STATUS_SOLD, halt all production ... leave things in the
	// queue because if the sell process completes we get money back for them, for now we
	// will be just frozen in time
	// Actually, there will usually be nothing in the queue since everything gets cancel/refunded
	// at the start of sell, but we still don't want to do anything here.
	//
	if( us->getStatusBits().test( OBJECT_STATUS_SOLD ) )
		return UPDATE_SLEEP_NONE;

	// get the player that is building this thing
	Player *player = us->getControllingPlayer();

	// sanity
	if( player == nullptr )
	{

		// remove from queue list
		removeFromProductionQueue( production );

		// delete the production entry
		deleteInstance(production);

		return UPDATE_SLEEP_NONE;

	}

	//
	// you can disallow types of units on the fly in scripts, so if there is something
	// in the queue that suddenly can't be build then get rid of it
	//
	if( production->getProductionType() == PRODUCTION_UNIT &&
			player->allowedToBuild( production->getProductionObject() ) == FALSE )
	{

		// Don't cancel dozers in the queue.  jba.
		if (!production->getProductionObject()->isKindOf(KINDOF_DOZER))
		{
			if( cancelUnitCreate(production) )
				return UPDATE_SLEEP_NONE;
		}

	}

	// increase the frames we've been under production for
	production->m_framesUnderConstruction++;

	// how many total logic frames does it take to produce this unit
	Int totalProductionFrames;
	if( production->m_type == PRODUCTION_UNIT )
		totalProductionFrames = production->m_objectToProduce->calcTimeToBuild( player );
	else
		totalProductionFrames = production->m_upgradeToResearch->calcTimeToBuild( player );

	// figure out our percent complete
	production->m_percentComplete = INT_TO_REAL( production->m_framesUnderConstruction ) /
																	INT_TO_REAL( totalProductionFrames ) *
																	100.0f;

	// if we've reached 100% or more we're done, tada!
	if( production->m_percentComplete >= 100.0f )
	{

		// handle unit production items
		if( production->m_type == PRODUCTION_UNIT )
		{
			Object *creationBuilding = us;

			// create this unit

			//if I've got a ExitModule
			ExitInterface *exitInterface = creationBuilding->getObjectExitInterface();
			if( exitInterface )
			{

				//
				// only exit if the exit interface will let us ... if we can't we'll just keep this
				// unit in the production and keep "overbuilding" it, every frame we will check to see
				// if we can exit and then move onto production of the next one
				//
				Int numberToTry = production->getProductionQuantityRemaining();// this will change midloop, so it can't be the For clause
				for( int i = 0; i < numberToTry; i++ )
				{
					ExitDoorType exitDoor = production->getExitDoor();
					if (exitDoor == DOOR_NONE_AVAILABLE)
					{
						exitDoor = exitInterface->reserveDoorForExit(production->m_objectToProduce, nullptr);
						production->setExitDoor(exitDoor);
					}

					if (exitDoor != DOOR_NONE_AVAILABLE)
					{
						// note, could be DOOR_NONE_NEEDED! so door could be null. (srj)
						DoorInfo* door = (exitDoor >= 0 && exitDoor < DOOR_COUNT_MAX) ? &m_doors[exitDoor] : nullptr;

						//
						// if the producing structure has a door opening animation we will set the condition
						// bits to open the door (note that we must also take care to remove any condition
						// that had us previously closing a door)
						//
						const ProductionUpdateModuleData *d = getProductionUpdateModuleData();
						if( d->m_numDoorAnimations > 0 && door != nullptr )
						{

							// if the door is closed, open it
							if( door->m_doorOpenedFrame == 0 && door->m_doorWaitOpenFrame == 0 && door->m_doorClosedFrame == 0 )
							{

								door->m_doorOpenedFrame = now;
								m_setFlags.set( theOpeningFlags[exitDoor] );
								m_flagsDirty = TRUE;

							}
							// if the door is waiting-open, keep it there
							else if( door->m_doorWaitOpenFrame != 0 )
							{

								door->m_doorWaitOpenFrame = now;

							}
							// if the door is closing, for now, pop it to waiting open
							else if( door->m_doorClosedFrame != 0 )
							{

								door->m_doorWaitOpenFrame = now;

								m_clearFlags.set( theOpeningFlags[exitDoor], true );
								m_clearFlags.set( theClosingFlags[exitDoor], true );
								m_setFlags.set( theOpeningFlags[exitDoor], false );
								m_setFlags.set( theClosingFlags[exitDoor], false );

								m_setFlags.set( theWaitingOpenFlags[exitDoor] );
								m_flagsDirty = TRUE;

							}

						}

						// we now go into the construction complete condition for a while
						if( m_constructionCompleteFrame == 0 )
						{

							m_constructionCompleteFrame = now;
							m_setFlags.set( MODELCONDITION_CONSTRUCTION_COMPLETE );
							m_flagsDirty = TRUE;

						}

						//
						// make a new object, note that for production buildings that have door
						// animations we will not make the object until the door has been totally
						// opened
						//
						if( d->m_numDoorAnimations == 0 || door == nullptr || door->m_doorWaitOpenFrame != 0 )
						{
							Object *newObj = TheThingFactory->newObject( production->m_objectToProduce,
																	creationBuilding->getControllingPlayer()->getDefaultTeam() );

							newObj->setProducer(creationBuilding);

							// call the exit interface to do the rally point and position stuff
							exitInterface->exitObjectViaDoor( newObj, exitDoor );

							// since we successfully exited via this door, we should NOT call unreserveDoorForExit
							// when we toss this production entry. so set it back to an innocuous value.
							production->setExitDoor(DOOR_NONE_AVAILABLE);

							// Now, play the sound associated with the new object's production.
							AudioEventRTS voiceCreate = *newObj->getTemplate()->getVoiceCreated();
							voiceCreate.setObjectID(newObj->getID());
							TheAudio->addAudioEvent(&voiceCreate);

							// call the onUnitCreated for the player
							creationBuilding->getControllingPlayer()->onUnitCreated( creationBuilding, newObj );

							// onCreates have been called on newObj, and after that the owner was set,
							// so now is the time to call the game side of CreateModules
							for (BehaviorModule** m = newObj->getBehaviorModules(); *m; ++m)
							{
								CreateModuleInterface* create = (*m)->getCreate();
								if (!create)
									continue;
								create->onBuildComplete();
							}

							if( production->getProductionQuantity() == production->getProductionQuantityRemaining() )
							{
								//Call the voice created for the 1st object -- because it's possible to create multiple objects like redguards!
								AudioEventRTS sound = *newObj->getTemplate()->getPerUnitSound( "VoiceCreate" );
								sound.setObjectID( newObj->getID() );
								TheAudio->addAudioEvent( &sound );
							}

							creationBuilding->getControllingPlayer()->getAcademyStats()->recordProduction( newObj, creationBuilding );

							//We created one guy, but we may want to do more so we should stay in this node of production.
							// This is last so the voice check can easily check for "first" guy
							production->oneProductionSuccessful();

						}

					}

				}

				if( production->getProductionQuantityRemaining() == 0 )
				{
					// remove this production entry so we can go on to the next if we are totally finished
					removeFromProductionQueue( production );

					// delete the production entry
					deleteInstance(production);
				}

			}
			else
			{

				// there is no exit interface, this is an error
				DEBUG_CRASH( ("Cannot create '%s', there is no ExitUpdate interface defined for producer object '%s'",
															production->m_objectToProduce->getName().str(),
															creationBuilding->getTemplate()->getName().str()) );

				// remove this item from the production queue
				removeFromProductionQueue( production );

				// delete the production entry
				deleteInstance(production);

			}

		}
		else if( production->m_type == PRODUCTION_UPGRADE )
		{
			const UpgradeTemplate *upgrade = production->m_upgradeToResearch;

			// we finished an upgrade, lets add that money spent on it to the scorekeeper
			player->getScoreKeeper()->addMoneySpent(upgrade->calcCostToBuild(player));

			// notify the script engine
			TheScriptEngine->notifyOfCompletedUpgrade(
				us->getControllingPlayer()->getPlayerIndex(),
				upgrade->getUpgradeName(),
				us->getID());

			// print a message to the local player, if it wants one
			if( us->isLocallyViewed() && !upgrade->getDisplayNameLabel().isEmpty() )
			{
				UnicodeString msg;
				UnicodeString format = TheGameText->fetch( "UPGRADE:UpgradeComplete" );
				UnicodeString upgradeName = TheGameText->fetch( upgrade->getDisplayNameLabel().str() );

				msg.format( format.str(), upgradeName.str() );
				TheInGameUI->message( msg );

				// upgrades are a more rare event, play a nifty radar event thingy
				TheRadar->createEvent( us->getPosition(), RADAR_EVENT_UPGRADE );

				//Play the sound for the upgrade, because we just built it!
				AudioEventRTS sound = *upgrade->getResearchCompleteSound();
				if( TheAudio->isValidAudioEvent( &sound ) )
				{
					//We have a custom upgrade complete sound.
					sound.setObjectID( us->getID() );
					TheAudio->addAudioEvent( &sound );
				}
				else
				{
					//Use a generic EVA event.
					TheEva->setShouldPlay(EVA_UpgradeComplete);
				}

				//Play any available unit specific sound for upgrade.
				sound = *upgrade->getUnitSpecificSound();
				sound.setObjectID( us->getID() );
				TheAudio->addAudioEvent( &sound );

			}

			// update the upgrade status in the player or give the upgrade to the object
			if( upgrade->getUpgradeType() == UPGRADE_TYPE_PLAYER )
			{
				player->addUpgrade( upgrade, UPGRADE_STATUS_COMPLETE );
			}
			else
			{
				us->giveUpgrade( upgrade );
			}

			player->getAcademyStats()->recordUpgrade( upgrade, FALSE );

			//Also mark the UI dirty -- incase object with upgrade cameo is selected.
			Drawable *draw = TheInGameUI->getFirstSelectedDrawable();
			Object *selectedObject = draw ? draw->getObject() : nullptr;
			if( selectedObject )
			{
				const ThingTemplate *thing = selectedObject->getTemplate();
				for( Int i = 0; i < MAX_UPGRADE_CAMEO_UPGRADES; i++ )
				{
					AsciiString upgradeName = thing->getUpgradeCameoName( i );
					const UpgradeTemplate *testUpgrade = TheUpgradeCenter->findUpgrade( upgradeName );
					if( testUpgrade == upgrade )
					{
						//Our selected object has the upgrade
						TheControlBar->markUIDirty();
						break;
					}
				}
			}

			// remove this production entry so we can go on to the next
			removeFromProductionQueue( production );

			// delete the production entry
			deleteInstance(production);

		}

	}

	return UPDATE_SLEEP_NONE;
}

//-------------------------------------------------------------------------------------------------
/** Add the production entry to the *END* of the production queue list */
//-------------------------------------------------------------------------------------------------
void ProductionUpdate::addToProductionQueue( ProductionEntry *production )
{

	// check for empty list
	if( m_productionQueue == nullptr )
		m_productionQueue = production;

	// make any existing tail pointer now point to us, and we point back to them
	if( m_productionQueueTail )
	{

		m_productionQueueTail->m_next = production;
		production->m_prev = m_productionQueueTail;

	}

	// this production entry is now the new tail at the end of the list
	m_productionQueueTail = production;

	// we now have one more production item
	++m_productionCount;

	// when something is in the queue ... we are in the actively constructing state
	Drawable *draw = getObject()->getDrawable();
	if( draw )
	{
		ModelConditionFlags condition = draw->getModelConditionFlags();

		if( condition.test( MODELCONDITION_ACTIVELY_CONSTRUCTING ) == FALSE )
		{

			m_setFlags.set( MODELCONDITION_ACTIVELY_CONSTRUCTING );
			m_flagsDirty = TRUE;

		}

	}

}

//-------------------------------------------------------------------------------------------------
/** Remove the production entry from the production queue list */
//-------------------------------------------------------------------------------------------------
void ProductionUpdate::removeFromProductionQueue( ProductionEntry *production )
{
	if (production->m_type == PRODUCTION_UNIT &&
			production->m_exitDoor != DOOR_NONE_AVAILABLE)
	{
		ExitInterface* exitInterface = getObject()->getObjectExitInterface();
		if (exitInterface)
		{
			exitInterface->unreserveDoorForExit(production->m_exitDoor);
		}
	}

	// detach prev pointer, keep head pointer to the whole queue in tact
	if( production->m_prev )
		production->m_prev->m_next = production->m_next;
	else
		m_productionQueue = production->m_next;

	// detach next pointer, keep tail pointer to the whole queue in tact
	if( production->m_next )
		production->m_next->m_prev = production->m_prev;
	else
		m_productionQueueTail = production->m_prev;

	// we now have one less production item
	--m_productionCount;

	// when we go back to zero things in our queue, we clear the constructing state
	Drawable *draw = getObject()->getDrawable();
	if( draw )
	{
		ModelConditionFlags condition = draw->getModelConditionFlags();

		if( m_productionCount == 0 && condition.test( MODELCONDITION_ACTIVELY_CONSTRUCTING ) == TRUE )
		{

			m_clearFlags.set( MODELCONDITION_ACTIVELY_CONSTRUCTING, true );
			m_setFlags.set( MODELCONDITION_ACTIVELY_CONSTRUCTING, false );
			m_flagsDirty = TRUE;

		}

	}

/*
	// Debugging
	UnicodeString msg;
	if( production->getProductionType() == ProductionEntry::PRODUCTION_UNIT )
	{
		msg.format( L"Removed unit '%S'(%d)", production->getProductionObject()->getName().str(),
																					production->getProductionID() );
		TheInGameUI->message( msg );
	}
	else
	{
		msg.format( L"Remove upgrade '%S'", production->getProductionUpgrade()->getName().str() );
		TheInGameUI->message( msg );
	}
*/

}

//-------------------------------------------------------------------------------------------------
Bool ProductionUpdate::cancelUpgrade( ProductionEntry *production )
{
	DEBUG_ASSERTCRASH(production->getProductionType() == PRODUCTION_UPGRADE,
		("ProductionUpdate::cancelUpgrade - called with wrong production type"));

	Player *player = getObject()->getControllingPlayer();
	const UpgradeTemplate *upgrade = production->getProductionUpgrade();

	// sanity, you can't cancel it if the player isn't actually building one
	if( upgrade->getUpgradeType() == UPGRADE_TYPE_PLAYER && player->hasUpgradeInProduction( upgrade ) == FALSE )
		return FALSE;

	// refund money back to the player
	Money *money = player->getMoney();
	money->deposit( production->m_upgradeToResearch->calcCostToBuild( player ), TRUE, FALSE );

	// remove this production from the queue
	removeFromProductionQueue( production );

	// delete production instance
	deleteInstance(production);

	//
	// remove the IN_PRODUCTION status of this upgrade from the player, object upgrades don't
	// have any other IN_PRODUCTION status other than their existence in the build queue
	//
	if( upgrade->getUpgradeType() == UPGRADE_TYPE_PLAYER )
		player->removeUpgrade( upgrade );

	return TRUE;
}

//-------------------------------------------------------------------------------------------------
Bool ProductionUpdate::cancelUnitCreate( ProductionEntry *production )
{
	DEBUG_ASSERTCRASH(production->getProductionType() == PRODUCTION_UNIT,
		("ProductionUpdate::cancelUnitCreate - called with wrong production type"));

	Player *player = getObject()->getControllingPlayer();

#if !RETAIL_COMPATIBLE_CRC
	// TheSuperHackers @bugfix arcticdolphin 07/09/2026 No cancel once the batch has started producing.
	if( production->getProductionQuantityRemaining() < production->getProductionQuantity() )
		return FALSE;
#endif

	// give the player the cost of the object back
	Money *money = player->getMoney();
	money->deposit( production->m_objectToProduce->calcCostToBuild( player ), TRUE, FALSE );

	// remove from queue list
	removeFromProductionQueue( production );

	// delete the production entry
	deleteInstance(production);

	return TRUE;
}

//-------------------------------------------------------------------------------------------------
/** Is the upgrade already in the production queue.  Note that you can only have one
	* production entry for any given upgrade in the queue */
//-------------------------------------------------------------------------------------------------
Bool ProductionUpdate::isUpgradeInQueue( const UpgradeTemplate *upgrade ) const
{
	const ProductionEntry *production;

	for( production = firstProduction(); production; production = nextProduction( production ) )
		if( production->getProductionType() == PRODUCTION_UPGRADE &&
				production->getProductionUpgrade() == upgrade )
			return TRUE;

	return FALSE;  // not in queue

}

// ------------------------------------------------------------------------------------------------
/** count number of units with matching unit type in the production queue */
// ------------------------------------------------------------------------------------------------
UnsignedInt ProductionUpdate::countUnitTypeInQueue( const ThingTemplate *unitType ) const
{
	UnsignedInt count = 0;
	const ProductionEntry *production;

	for( production = firstProduction(); production; production = nextProduction( production ) )
		if( production->getProductionType() == PRODUCTION_UNIT &&
				production->getProductionObject() == unitType )
			count++;

	return count;

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void ProductionUpdate::onDie( const DamageInfo *damageInfo )
{
	// we need to cancel all of our production on death
	cancelAndRefundAllProduction();
}

// ------------------------------------------------------------------------------------------------
/** Cancel each of the production items in the queue.  By going through the actual cancel
	* methods, the cost of each of those items will be refunded to the player */
// ------------------------------------------------------------------------------------------------
void ProductionUpdate::cancelAndRefundAllProduction()
{
	// Empirically, in release the code can loop forever.  So we limit to 100 passes. jba. [8/31/2003]
	const Int productionLimit = 100;// With luck, we never queue up 100 units. [8/31/2003]
	Int i;
	for(i = 0; i < productionLimit && m_productionQueue; ++i)
	{
		ProductionEntry *production = m_productionQueue;

		if( production->getProductionType() == PRODUCTION_UNIT )
		{
			if( !cancelUnitCreate( production ) )
			{
#if RETAIL_COMPATIBLE_CRC
				DEBUG_CRASH(("This code path cannot be reached"));
#endif
				// Unable to cancel for a refund... delete anyway.
				removeFromProductionQueue( production );
				deleteInstance( production );
			}
		}
		else if( production->getProductionType() == PRODUCTION_UPGRADE )
		{
			if( !cancelUpgrade( production ) )
			{
#if RETAIL_COMPATIBLE_CRC
				// Cannot cancel the head production... this loop is stuck now and can quit.
				return;
#else
				// Unable to cancel for a refund... delete anyway.
				removeFromProductionQueue( production );
				deleteInstance( production );
#endif
			}
		}
		else
		{
			// unknown production type
			DEBUG_CRASH(( "ProductionUpdate::cancelAndRefundAllProduction - Unknown production type '%d'", production->getProductionType() ));
			return;
		}
	}
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void ProductionUpdate::setHoldDoorOpen(ExitDoorType exitDoor, Bool holdIt)
{
	if (exitDoor >= DOOR_1 && exitDoor < DOOR_COUNT_MAX)
	{
		DoorInfo& door = m_doors[exitDoor];
		door.m_holdOpen = holdIt;
		if (holdIt && door.m_doorOpenedFrame == 0 && door.m_doorWaitOpenFrame == 0 && door.m_doorClosedFrame == 0)
		{
			door.m_doorOpenedFrame = TheGameLogic->getFrame();
			m_setFlags.set( theOpeningFlags[exitDoor] );
			m_flagsDirty = TRUE;
		}
	}
}

// ------------------------------------------------------------------------------------------------
/** Helper method to retrieve a production update interface from an object if one is present */
// ------------------------------------------------------------------------------------------------
/*static*/ ProductionUpdateInterface *ProductionUpdate::getProductionUpdateInterfaceFromObject( Object *obj )
{

	// sanity
	if( obj == nullptr )
		return nullptr;

	BehaviorModule **bmi;
	ProductionUpdateInterface *pui = nullptr;
	for( bmi = obj->getBehaviorModules(); *bmi; ++bmi )
	{

		pui = (*bmi)->getProductionUpdateInterface();
		if( pui )
			return pui;

	}

	// interface not found
	return nullptr;


}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void ProductionUpdate::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void ProductionUpdate::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// production queue count
	ProductionEntry *production;
	UnsignedShort productionCount = 0;
	for( production = m_productionQueue; production; production = production->m_next )
		productionCount++;
	xfer->xferUnsignedShort( &productionCount );

	// production queue data
	if( xfer->getXferMode() == XFER_SAVE )
	{
		AsciiString name;

		// write all queue data
		for( production = m_productionQueue; production; production = production->m_next )
		{

			// type
			xfer->xferUser( &production->m_type, sizeof( ProductionType ) );

			// thing/upgrade template name
			if( production->m_type == PRODUCTION_UNIT )
				name = production->m_objectToProduce->getName();
			else
				name = production->m_upgradeToResearch->getUpgradeName();
			xfer->xferAsciiString( &name );

			// production ID
			xfer->xferUser( &production->m_productionID, sizeof( ProductionID ) );

			// percent complete
			xfer->xferReal( &production->m_percentComplete );

			// frames under construction
			xfer->xferInt( &production->m_framesUnderConstruction );

			// production quantity
			xfer->xferInt( &production->m_productionQuantityTotal );

			// production quantity in progress
			xfer->xferInt( &production->m_productionQuantityProduced );

			// exit door
			xfer->xferInt( (Int*)&production->m_exitDoor );

		}

	}
	else
	{
		AsciiString name;

		// the queue should be empty now
		if( m_productionQueue != nullptr )
		{

			DEBUG_CRASH(( "ProductionUpdate::xfer - m_productionQueue is not empty, but should be" ));
			throw SC_INVALID_DATA;

		}

		// read each element
		for( UnsignedShort i = 0; i < productionCount; ++i )
		{

			// allocate new production entry
			production = newInstance(ProductionEntry);

			// tie to list at end
			if( m_productionQueue == nullptr )
				m_productionQueue = production;

			// make any existing tail pointer now point to us, and we point back to them
			if( m_productionQueueTail )
			{

				m_productionQueueTail->m_next = production;
				production->m_prev = m_productionQueueTail;

			}

			// this production entry is now the new tail at the end of the list
			m_productionQueueTail = production;

			// type
			xfer->xferUser( &production->m_type, sizeof( ProductionType ) );

			// thing/upgrade template name
			xfer->xferAsciiString( &name );
			if( production->m_type == PRODUCTION_UNIT )
			{

				production->m_objectToProduce = TheThingFactory->findTemplate( name );
				if( production->m_objectToProduce == nullptr )
				{

					DEBUG_CRASH(( "ProductionUpdate::xfer - Cannot find template '%s'", name.str() ));
					throw SC_INVALID_DATA;

				}

			}
			else
			{

				production->m_upgradeToResearch = TheUpgradeCenter->findUpgrade( name );
				if( production->m_upgradeToResearch == nullptr )
				{

					DEBUG_CRASH(( "ProductionUpdate::xfer - Cannot find upgrade '%s'", name.str() ));
					throw SC_INVALID_DATA;

				}

			}

			// production ID
			xfer->xferUser( &production->m_productionID, sizeof( ProductionID ) );

			// percent complete
			xfer->xferReal( &production->m_percentComplete );

			// frames under construction
			xfer->xferInt( &production->m_framesUnderConstruction );

			// production quantity
			xfer->xferInt( &production->m_productionQuantityTotal );

			// production quantity in progress
			xfer->xferInt( &production->m_productionQuantityProduced );

			// exit door
			xfer->xferInt( (Int*)&production->m_exitDoor );

		}

	}

	// unique id
	xfer->xferUser( &m_uniqueID, sizeof( ProductionID ) );

	// production count
	xfer->xferUnsignedInt( &m_productionCount );

	// construction complete frame
	xfer->xferUnsignedInt( &m_constructionCompleteFrame );

	// door info
	xfer->xferUser( &m_doors, sizeof( DoorInfo ) * DOOR_COUNT_MAX );

	// clear flags
	m_clearFlags.xfer( xfer );

	// set flags
	m_setFlags.xfer( xfer );

	// flags dirty
	xfer->xferBool( &m_flagsDirty );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void ProductionUpdate::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
