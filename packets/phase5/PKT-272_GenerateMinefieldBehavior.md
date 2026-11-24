# Translation Packet `PKT-272`: `GenerateMinefieldBehavior`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-272`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/behavior/generate_minefield_behavior.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (25):**
  - `Common/GlobalData.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/GlobalData.h`
  - `Common/INI.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/INI.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/RandomValue.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/RandomValue.h`
  - `Common/Thing.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Thing.h`
  - `Common/ThingFactory.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ThingFactory.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/FXList.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/FXList.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/DieModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DieModule.h`
  - `GameLogic/Module/GenerateMinefieldBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/GenerateMinefieldBehavior.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Module/UpgradeModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpgradeModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/ObjectCreationList.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ObjectCreationList.h`
  - `GameLogic/ObjectIter.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ObjectIter.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `GameLogic/Weapon.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h`
  - `PreRTS.h`
- **Implemented Module Facets (2):**
  - `DieModuleInterface* getDie()`
  - `UpgradeModuleInterface* getUpgrade()`
- **INI `FieldParse` Table Entries (14):**
  - `"MineName" => INI::parseAsciiString`
  - `"UpgradedMineName" => INI::parseAsciiString`
  - `"UpgradedTriggeredBy" => INI::parseAsciiString`
  - `"GenerationFX" => INI::parseFXList`
  - `"DistanceAroundObject" => INI::parseReal`
  - `"MinesPerSquareFoot" => INI::parseReal`
  - `"GenerateOnlyOnDeath" => INI::parseBool`
  - `"BorderOnly" => INI::parseBool`
  - `"SmartBorder" => INI::parseBool`
  - `"SmartBorderSkipInterior" => INI::parseBool`
  - `"AlwaysCircular" => INI::parseBool`
  - `"Upgradable" => INI::parseBool`
  - `"RandomJitter" => INI::parsePercentToReal`
  - `"SkipIfThisMuchUnderStructure" => INI::parsePercentToReal`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (8 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferBool( &m_generated )`
  3. `xferBool( &m_hasTarget )`
  4. `xferBool( &m_upgraded )`
  5. `xferCoord3D( &m_target )`
  6. `xferUnsignedByte( &spacesCount )`
  7. `xferObjectID( &(*it) )`
  8. `xferObjectID( &objectID )`
- **PRNG Calls (3 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValueReal(-maxAmt, maxAmt)`
  - `GameLogicRandomValueReal(-maxAmt, maxAmt)`
  - `GameLogicRandomValueReal(-PI, PI)`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/GenerateMinefieldBehavior.h`
```cpp
// FILE: GenerateMinefieldBehavior.h /////////////////////////////////////////////////////////////////////////
// Author: Colin Day, December 2001
// Desc:   Update that will count down a lifetime and destroy object when it reaches zero
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/DieModule.h"
#include "GameLogic/Module/UpgradeModule.h"
#include "GameLogic/Module/UpdateModule.h"

//-------------------------------------------------------------------------------------------------
class GenerateMinefieldBehaviorModuleData : public BehaviorModuleData
{
public:
	UpgradeMuxData				m_upgradeMuxData;
	AsciiString						m_mineName;
	AsciiString						m_mineNameUpgraded;
	AsciiString						m_mineUpgradeTrigger;
	const FXList*					m_genFX;
	Real									m_distanceAroundObject;
	Real									m_minesPerSquareFoot;
	Real									m_randomJitter;
	Real									m_skipIfThisMuchUnderStructure;
	Bool									m_onDeath;
	Bool									m_borderOnly;
	Bool									m_alwaysCircular;
	Bool									m_upgradable;
	Bool									m_smartBorder;
	Bool									m_smartBorderSkipInterior;

	GenerateMinefieldBehaviorModuleData();

	static void buildFieldParse(MultiIniFieldParse& p);

private:

};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class GenerateMinefieldBehavior : public UpdateModule,
																	public DieModuleInterface,
																	public UpgradeMux
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( GenerateMinefieldBehavior, "GenerateMinefieldBehavior" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( GenerateMinefieldBehavior, GenerateMinefieldBehaviorModuleData )

public:

	GenerateMinefieldBehavior( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	// module methods
	static Int getInterfaceMask() { return UpdateModule::getInterfaceMask() | (MODULEINTERFACE_DIE) | (MODULEINTERFACE_UPGRADE); }

	// BehaviorModule
	virtual DieModuleInterface* getDie() override { return this; }
	virtual UpgradeModuleInterface* getUpgrade() override { return this; }
	virtual UpdateSleepTime update() override;

	// DamageModuleInterface
	virtual void onDie( const DamageInfo *damageInfo ) override;

	void setMinefieldTarget(const Coord3D* pos);

protected:

	virtual void upgradeImplementation() override;
	virtual Bool isSubObjectsUpgrade() override { return false; }

	virtual void getUpgradeActivationMasks(UpgradeMaskType& activation, UpgradeMaskType& conflicting) const override
	{
		getGenerateMinefieldBehaviorModuleData()->m_upgradeMuxData.getUpgradeActivationMasks(activation, conflicting);
	}
	virtual void performUpgradeFX() override
	{
		getGenerateMinefieldBehaviorModuleData()->m_upgradeMuxData.performUpgradeFX(getObject());
	}
	virtual void processUpgradeRemoval() override
	{
		// I can't take it any more.  Let the record show that I think the UpgradeMux multiple inheritance is CRAP.
		getGenerateMinefieldBehaviorModuleData()->m_upgradeMuxData.muxDataProcessUpgradeRemoval(getObject());
	}

	virtual Bool requiresAllActivationUpgrades() const override
	{
		return getGenerateMinefieldBehaviorModuleData()->m_upgradeMuxData.m_requiresAllTriggers;
	}


private:
	Coord3D							m_target;
	Bool								m_hasTarget;
	Bool								m_generated;
	Bool								m_upgraded;
	std::list<ObjectID> m_mineList;

	const Coord3D* getMinefieldTarget() const;
	void placeMines();
	void placeMinesInFootprint(const GeometryInfo& geom, const ThingTemplate* mineTemplate);
	void placeMinesAroundCircle(const Coord3D& pos, Real radius, const ThingTemplate* mineTemplate);
	void placeMinesAlongLine(const Coord3D& posStart, const Coord3D& posEnd, const ThingTemplate* mineTemplate, Bool skipOneAtStart);
	void placeMinesAroundRect(const Coord3D& pos, Real majorRadius, Real minorRadius, const ThingTemplate* mineTemplate);
	Object* placeMineAt(const Coord3D& pt, const ThingTemplate* mineTemplate, Team* team, const Object* producer);
};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Behavior/GenerateMinefieldBehavior.cpp`
```cpp
// FILE: GenerateMinefieldBehavior.cpp ///////////////////////////////////////////////////////////////////////
// Author:
// Desc:
///////////////////////////////////////////////////////////////////////////////////////////////////


// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine
#define DEFINE_SLOWDEATHPHASE_NAMES

#include "Common/GlobalData.h"
#include "Common/Thing.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/INI.h"
#include "Common/Player.h"
#include "Common/RandomValue.h"
#include "Common/Xfer.h"
#include "GameClient/Drawable.h"
#include "GameClient/FXList.h"
#include "GameClient/InGameUI.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/GenerateMinefieldBehavior.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectIter.h"
#include "GameLogic/ObjectCreationList.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Weapon.h"


//-------------------------------------------------------------------------------------------------
GenerateMinefieldBehaviorModuleData::GenerateMinefieldBehaviorModuleData()
{
	m_mineName.clear();
	m_mineNameUpgraded.clear();
	m_mineUpgradeTrigger.clear();

	m_genFX = nullptr;
	m_distanceAroundObject = TheGlobalData->m_standardMinefieldDistance;
	m_minesPerSquareFoot = TheGlobalData->m_standardMinefieldDensity;
	m_onDeath = false;
	m_borderOnly = true;
	m_alwaysCircular = false;
	m_upgradable = false;
	m_smartBorder = false;
	m_smartBorderSkipInterior = true;
	m_randomJitter = 0.0f;
	m_skipIfThisMuchUnderStructure = 0.33f;
}

//-------------------------------------------------------------------------------------------------
/*static*/ void GenerateMinefieldBehaviorModuleData::buildFieldParse(MultiIniFieldParse& p)
{

	static const FieldParse dataFieldParse[] =
	{
		{ "MineName", INI::parseAsciiString,	nullptr, offsetof( GenerateMinefieldBehaviorModuleData, m_mineName ) },
		{ "UpgradedMineName", INI::parseAsciiString,	nullptr, offsetof( GenerateMinefieldBehaviorModuleData, m_mineNameUpgraded ) },
		{ "UpgradedTriggeredBy", INI::parseAsciiString,	nullptr, offsetof( GenerateMinefieldBehaviorModuleData, m_mineUpgradeTrigger ) },
		{ "GenerationFX", INI::parseFXList,	nullptr, offsetof( GenerateMinefieldBehaviorModuleData, m_genFX ) },
		{ "DistanceAroundObject", INI::parseReal, nullptr, offsetof( GenerateMinefieldBehaviorModuleData, m_distanceAroundObject ) },
		{ "MinesPerSquareFoot", INI::parseReal, nullptr, offsetof( GenerateMinefieldBehaviorModuleData, m_minesPerSquareFoot ) },
		{ "GenerateOnlyOnDeath", INI::parseBool, nullptr, offsetof(GenerateMinefieldBehaviorModuleData, m_onDeath) },
		{ "BorderOnly", INI::parseBool, nullptr, offsetof(GenerateMinefieldBehaviorModuleData, m_borderOnly) },
		{ "SmartBorder", INI::parseBool, nullptr, offsetof(GenerateMinefieldBehaviorModuleData, m_smartBorder) },
		{ "SmartBorderSkipInterior", INI::parseBool, nullptr, offsetof(GenerateMinefieldBehaviorModuleData, m_smartBorderSkipInterior) },
		{ "AlwaysCircular", INI::parseBool, nullptr, offsetof(GenerateMinefieldBehaviorModuleData, m_alwaysCircular) },
		{ "Upgradable", INI::parseBool, nullptr, offsetof(GenerateMinefieldBehaviorModuleData, m_upgradable) },
		{ "RandomJitter", INI::parsePercentToReal, nullptr, offsetof(GenerateMinefieldBehaviorModuleData, m_randomJitter) },
		{ "SkipIfThisMuchUnderStructure", INI::parsePercentToReal, nullptr, offsetof(GenerateMinefieldBehaviorModuleData, m_skipIfThisMuchUnderStructure) },
		{ nullptr, nullptr, nullptr, 0 }
	};

  BehaviorModuleData::buildFieldParse(p);
  p.add(dataFieldParse);
  p.add(UpgradeMuxData::getFieldParse(), offsetof( GenerateMinefieldBehaviorModuleData, m_upgradeMuxData ));
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
GenerateMinefieldBehavior::GenerateMinefieldBehavior( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{
	m_target.zero();
	m_generated = false;
	m_hasTarget = false;
	m_upgraded = false;
	m_mineList.clear();
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
GenerateMinefieldBehavior::~GenerateMinefieldBehavior()
{
	m_mineList.clear();
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void GenerateMinefieldBehavior::upgradeImplementation()
{
	placeMines();
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void GenerateMinefieldBehavior::onDie( const DamageInfo *damageInfo )
{
	const GenerateMinefieldBehaviorModuleData* d = getGenerateMinefieldBehaviorModuleData();

	if (d->m_onDeath)
	{
		placeMines();
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void GenerateMinefieldBehavior::setMinefieldTarget(const Coord3D* pos)
{
	if (pos)
	{
		m_hasTarget = true;
		m_target = *pos;
	}
	else
	{
		m_hasTarget = false;
		m_target.zero();
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
const Coord3D* GenerateMinefieldBehavior::getMinefieldTarget() const
{
	return m_hasTarget ? &m_target : getObject()->getPosition();
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
static Bool isAnythingTooClose2D(const std::vector<Object*>& v, const Coord3D& pos, Real minDistSqr)
{
	for (std::vector<Object*>::const_iterator it = v.begin(); it != v.end(); ++it)
	{
		const Coord3D* p = (*it)->getPosition();
		Real distSqr = sqr(p->x - pos.x) + sqr(p->y - pos.y);
		if (distSqr < minDistSqr)
			return true;
	}
	return false;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
static void offsetBySmallRandomAmount(Coord3D& pt, Real maxAmt)
{
	pt.x += GameLogicRandomValueReal(-maxAmt, maxAmt);
	pt.y += GameLogicRandomValueReal(-maxAmt, maxAmt);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Object* GenerateMinefieldBehavior::placeMineAt(const Coord3D& pt, const ThingTemplate* mineTemplate, Team* team, const Object* producer)
{
	Coord3D tmp = pt;
	tmp.z = 99999.0f;
	PathfindLayerEnum layer = TheTerrainLogic->getHighestLayerForDestination(&tmp);

	if (layer == LAYER_GROUND && TheTerrainLogic->isUnderwater(pt.x, pt.y))
		return nullptr;

	if (layer == LAYER_GROUND && TheTerrainLogic->isCliffCell(pt.x, pt.y))
		return nullptr;

	Real orient = GameLogicRandomValueReal(-PI, PI);

	// if the mine will be "mostly" under a structure, don't place it.
	// for now, "mostly" means "central third of radius would overlap"
	const GenerateMinefieldBehaviorModuleData* d = getGenerateMinefieldBehaviorModuleData();
	GeometryInfo geom = mineTemplate->getTemplateGeometryInfo();
	Real mineRadius = mineTemplate->getTemplateGeometryInfo().getBoundingCircleRadius();
	geom.expandFootprint(mineRadius * -(1.0f - d->m_skipIfThisMuchUnderStructure));
	ObjectIterator *iter = ThePartitionManager->iteratePotentialCollisions( &pt, geom, orient );
	MemoryPoolObjectHolder hold(iter);
	for (Object* them = iter->first(); them; them = iter->next())
	{
		if (them->isKindOf(KINDOF_STRUCTURE))
			return nullptr;
	}

	Object* mine = TheThingFactory->newObject(mineTemplate, team);
	mine->setPosition(&pt);
	mine->setOrientation(orient);
	mine->setProducer(producer);

	for (BehaviorModule** bmi = mine->getBehaviorModules(); *bmi; ++bmi)
	{
		LandMineInterface* lmi = (*bmi)->getLandMineInterface();
		if (lmi)
		{
			lmi->setScootParms(*producer->getPosition(), pt);
			break;
		}
	}

	// Keep track of the mines
	if (mine && d->m_upgradable)
	{
		m_mineList.push_back(mine->getID());
	}

	return mine;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void GenerateMinefieldBehavior::placeMinesAlongLine(const Coord3D& posStart, const Coord3D& posEnd, const ThingTemplate* mineTemplate, Bool skipOneAtStart)
{
	const Object* obj = getObject();
	const GenerateMinefieldBehaviorModuleData* d = getGenerateMinefieldBehaviorModuleData();
	Team* team = obj->getControllingPlayer()->getDefaultTeam();

	Real dx = posEnd.x - posStart.x;
	Real dy = posEnd.y - posStart.y;
	Real len = WWMath::Sqrt(sqr(dx) + sqr(dy));
	Real mineRadius = mineTemplate->getTemplateGeometryInfo().getBoundingCircleRadius();
	Real mineDiameter = mineRadius * 2.0f;
	Real mineJitter = mineRadius*d->m_randomJitter;
	Int numMines = REAL_TO_INT_CEIL(WWMath::Div_Safe(len, mineDiameter, 1.0f));
	if (numMines < 1)
		numMines = 1;
	Real inc = len/numMines;
	for (Real place = skipOneAtStart ? inc : 0; place <= len; place += inc)
	{
		Coord3D pt;
		pt.x = posStart.x + place * dx / len;
		pt.y = posStart.y + place * dy / len;
		pt.z = TheTerrainLogic->getGroundHeight( pt.x, pt.y );
		offsetBySmallRandomAmount(pt, mineJitter);
		placeMineAt(pt, mineTemplate, team, obj);
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
static void makeCorner(const Coord3D& pos, Real majorRadius, Real minorRadius, const Matrix3D& mtx, Coord3D& corner)
{
	Vector3 tmp;
	tmp.X = majorRadius;
	tmp.Y = minorRadius;
	tmp.Z = 0;
	Matrix3D::Transform_Vector(mtx, tmp, &tmp);
	corner.x = tmp.X;
	corner.y = tmp.Y;
	corner.z = tmp.Z;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void GenerateMinefieldBehavior::placeMinesAroundRect(const Coord3D& pos, Real majorRadius, Real minorRadius, const ThingTemplate* mineTemplate)
{
	const Object* obj = getObject();
	const Matrix3D* mtx = obj->getTransformMatrix();

	Coord3D pt[4];
	makeCorner(pos,  majorRadius,  minorRadius, *mtx, pt[0]);
	makeCorner(pos, -majorRadius,  minorRadius, *mtx, pt[1]);
	makeCorner(pos, -majorRadius, -minorRadius, *mtx, pt[2]);
	makeCorner(pos,  majorRadius, -minorRadius, *mtx, pt[3]);

	placeMinesAlongLine(pt[0], pt[1], mineTemplate, true);
	placeMinesAlongLine(pt[1], pt[2], mineTemplate, true);
	placeMinesAlongLine(pt[2], pt[3], mineTemplate, true);
	placeMinesAlongLine(pt[3], pt[0], mineTemplate, true);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void GenerateMinefieldBehavior::placeMinesAroundCircle(const Coord3D& pos, Real radius, const ThingTemplate* mineTemplate)
{
	const Object* obj = getObject();
	const GenerateMinefieldBehaviorModuleData* d = getGenerateMinefieldBehaviorModuleData();
	Team* team = obj->getControllingPlayer()->getDefaultTeam();

	Real circum = 2.0f * PI * radius;
	Real mineRadius = mineTemplate->getTemplateGeometryInfo().getBoundingCircleRadius();
	Real mineDiameter = mineRadius * 2.0f;
	Real mineJitter = mineRadius*d->m_randomJitter;
	Int numMines = REAL_TO_INT_CEIL(WWMath::Div_Safe(circum, mineDiameter, 1.0f));
	if (numMines < 1)
		numMines = 1;
	Real angleInc = (2*PI)/numMines;
	Real angleLim = (2*PI) - angleInc*0.5f;
	for (Real angle = 0; angle < angleLim; angle += angleInc)
	{
		Coord3D pt;
		pt.x = pos.x + radius * Cos(angle);
		pt.y = pos.y + radius * Sin(angle);
		pt.z = TheTerrainLogic->getGroundHeight( pt.x, pt.y );
		offsetBySmallRandomAmount(pt, mineJitter);
		placeMineAt(pt, mineTemplate, team, obj);
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void GenerateMinefieldBehavior::placeMinesInFootprint(const GeometryInfo& geom, const ThingTemplate* mineTemplate)
{
	const Object* obj = getObject();
	const GenerateMinefieldBehaviorModuleData* d = getGenerateMinefieldBehaviorModuleData();
	Team* team = obj->getControllingPlayer()->getDefaultTeam();

	Real area = geom.getFootprintArea();
	Int numMines = REAL_TO_INT_CEIL(d->m_minesPerSquareFoot * area);
	if (numMines < 1)
		numMines = 1;

	const Coord3D* target = getMinefieldTarget();
	std::vector<Object*> minesCreatedSoFar;
	Real minDistSqr = sqr(mineTemplate->getTemplateGeometryInfo().getBoundingCircleRadius() * 2.0f);
	for (int i = 0; i < numMines; ++i)
	{
		Coord3D pt;
		Int maxRetry = 100;
		do
		{
			geom.makeRandomOffsetWithinFootprint(pt);
			pt.x += target->x;
			pt.y += target->y;
			pt.z += target->z;
			--maxRetry;
		} while (isAnythingTooClose2D(minesCreatedSoFar, pt, minDistSqr) && maxRetry > 0);
		DEBUG_ASSERTCRASH(maxRetry>0,("ran out of retries %f",minDistSqr));

		if (getObject()->getGeometryInfo().isPointInFootprint(*target, pt))
			continue;

		Object* mine = placeMineAt(pt, mineTemplate, team, obj);	// can return null.
		if (mine)
			minesCreatedSoFar.push_back(mine);
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void GenerateMinefieldBehavior::placeMines()
{
	if (m_generated)
		return;

	m_generated = true;

	const Object* obj = getObject();
	const GenerateMinefieldBehaviorModuleData* d = getGenerateMinefieldBehaviorModuleData();
	const ThingTemplate* mineTemplate = nullptr;

	if (m_upgraded)
		mineTemplate = TheThingFactory->findTemplate(d->m_mineNameUpgraded);
	else
		mineTemplate = TheThingFactory->findTemplate(d->m_mineName);

	if (!mineTemplate)
	{
		DEBUG_CRASH(("mine %s not found",d->m_mineName.str()));
		return;
	}

	const Coord3D* target = getMinefieldTarget();
	if (d->m_smartBorder)
	{
		GeometryInfo geom = obj->getGeometryInfo();

		if (!d->m_smartBorderSkipInterior)
		{
			geom = mineTemplate->getTemplateGeometryInfo();
			placeMineAt(*target, mineTemplate, obj->getControllingPlayer()->getDefaultTeam(), obj);
		}

		if (d->m_alwaysCircular)
			geom.set(GEOMETRY_CYLINDER, false, 1, geom.getBoundingCircleRadius(), geom.getBoundingCircleRadius());

		Real mineRadius = mineTemplate->getTemplateGeometryInfo().getBoundingCircleRadius();
		Real mineDiameter = mineRadius * 2.0f;
		geom.expandFootprint(mineRadius);
		do
		{
			if (geom.getGeomType() == GEOMETRY_BOX && !d->m_alwaysCircular)
			{
				placeMinesAroundRect(*target, geom.getMajorRadius(), geom.getMinorRadius(), mineTemplate);
			}
			else
			{
				placeMinesAroundCircle(*target, geom.getMajorRadius(), mineTemplate);
			}
			geom.expandFootprint(mineDiameter);
		} while (geom.getBoundingCircleRadius() < d->m_distanceAroundObject);
	}
	else if (d->m_borderOnly)
	{
		GeometryInfo geom = obj->getGeometryInfo();
		geom.expandFootprint(d->m_distanceAroundObject);

		if (geom.getGeomType() == GEOMETRY_BOX && !d->m_alwaysCircular)
		{
			placeMinesAroundRect(*target, geom.getMajorRadius(), geom.getMinorRadius(), mineTemplate);
		}
		else
		{
			placeMinesAroundCircle(*target, geom.getMajorRadius(), mineTemplate);
		}
	}
	else
	{
		GeometryInfo geom = obj->getGeometryInfo();
		geom.expandFootprint(d->m_distanceAroundObject);

		if (d->m_alwaysCircular)
			geom.set(GEOMETRY_CYLINDER, false, 1, geom.getBoundingCircleRadius(), geom.getBoundingCircleRadius());

		placeMinesInFootprint(geom, mineTemplate);
	}

	FXList::doFXObj(d->m_genFX, obj);

}

// ------------------------------------------------------------------------------------------------

UpdateSleepTime GenerateMinefieldBehavior::update()
{
	// Test to see if we need to replace the current mines with upgraded ones
	if (!m_upgraded && getGenerateMinefieldBehaviorModuleData()->m_upgradable)
	{
		if (m_generated)
		{
			// Upgraded minefield to next level for China Player
			const UpgradeTemplate *upgradeTemplate = TheUpgradeCenter->findUpgrade( "Upgrade_ChinaEMPMines" );

			if (upgradeTemplate)
			{
				const UpgradeMaskType& upgradeMask = upgradeTemplate->getUpgradeMask();
				const UpgradeMaskType& objMask = getObject()->getObjectCompletedUpgradeMask();
				if (objMask.testForAny(upgradeMask))
				{
					m_upgraded = TRUE;

					// Remove all old mine objects if present
					for (std::list<ObjectID>::iterator it = m_mineList.begin(); it != m_mineList.end(); ++it)
					{
						ObjectID objID = *it;
						Object *obj = TheGameLogic->findObjectByID(objID);
						if (obj)
						{
							TheGameLogic->destroyObject(obj);
						}
					}
					m_mineList.clear();

					// Place new mines down (Replace old ones that we removed
					m_generated = false;
					placeMines();
				}
			}
		}

		return UPDATE_SLEEP_NONE;
	}

	return UPDATE_SLEEP_FOREVER;
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void GenerateMinefieldBehavior::crc( Xfer *xfer )
{

	// extend base class
	BehaviorModule::crc( xfer );

	// extend base class
	UpgradeMux::upgradeMuxCRC( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void GenerateMinefieldBehavior::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// base class
	BehaviorModule::xfer( xfer );

	// mux "base class"
	UpgradeMux::upgradeMuxXfer( xfer );

	// generated
	xfer->xferBool( &m_generated );
	xfer->xferBool( &m_hasTarget );
	xfer->xferBool( &m_upgraded );

	xfer->xferCoord3D( &m_target );

		// spaces info count and objectID data
	UnsignedByte spacesCount = m_mineList.size();
	xfer->xferUnsignedByte( &spacesCount );
	if( xfer->getXferMode() == XFER_SAVE )
	{
		// save all elements
		std::list<ObjectID>::iterator it;
		for( it = m_mineList.begin(); it != m_mineList.end(); ++it )
		{
			// object in this space
			xfer->xferObjectID( &(*it) );
		}

	}
	else if( xfer->getXferMode() == XFER_LOAD )
	{
		ObjectID objectID;
		m_mineList.clear();

		// read all elements
		std::list<ObjectID>::iterator it;
		it = m_mineList.begin();
		for(int i = 0; i < spacesCount; ++i )
		{
			// read object id
			xfer->xferObjectID( &objectID );

			m_mineList.push_back(objectID);
		}
	}

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void GenerateMinefieldBehavior::loadPostProcess()
{

	// extend base class
	BehaviorModule::loadPostProcess();

	// extend base class
	UpgradeMux::upgradeMuxLoadPostProcess();

}
```
