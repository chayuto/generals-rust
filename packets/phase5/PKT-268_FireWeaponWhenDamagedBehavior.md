# Translation Packet `PKT-268`: `FireWeaponWhenDamagedBehavior`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-268`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/behavior/fire_weapon_when_damaged_behavior.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (20):**
  - `Common/INI.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/INI.h`
  - `Common/RandomValue.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/RandomValue.h`
  - `Common/Thing.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Thing.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/FXList.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/FXList.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/DamageModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DamageModule.h`
  - `GameLogic/Module/FireWeaponWhenDamagedBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/FireWeaponWhenDamagedBehavior.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Module/UpgradeModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpgradeModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/ObjectCreationList.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ObjectCreationList.h`
  - `GameLogic/Weapon.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h`
  - `PreRTS.h`
- **Implemented Module Facets (2):**
  - `DamageModuleInterface* getDamage()`
  - `UpgradeModuleInterface* getUpgrade()`
- **INI `FieldParse` Table Entries (11):**
  - `"StartsActive" => INI::parseBool`
  - `"ReactionWeaponPristine" => INI::parseWeaponTemplate`
  - `"ReactionWeaponDamaged" => INI::parseWeaponTemplate`
  - `"ReactionWeaponReallyDamaged" => INI::parseWeaponTemplate`
  - `"ReactionWeaponRubble" => INI::parseWeaponTemplate`
  - `"ContinuousWeaponPristine" => INI::parseWeaponTemplate`
  - `"ContinuousWeaponDamaged" => INI::parseWeaponTemplate`
  - `"ContinuousWeaponReallyDamaged" => INI::parseWeaponTemplate`
  - `"ContinuousWeaponRubble" => INI::parseWeaponTemplate`
  - `"DamageTypes" => INI::parseDamageTypeFlags`
  - `"DamageAmount" => INI::parseReal`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (17 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferBool( &weaponPresent )`
  3. `xferSnapshot( m_reactionWeaponPristine )`
  4. `xferBool( &weaponPresent )`
  5. `xferSnapshot( m_reactionWeaponDamaged )`
  6. `xferBool( &weaponPresent )`
  7. `xferSnapshot( m_reactionWeaponReallyDamaged )`
  8. `xferBool( &weaponPresent )`
  9. `xferSnapshot( m_reactionWeaponRubble )`
  10. `xferBool( &weaponPresent )`
  11. `xferSnapshot( m_continuousWeaponPristine )`
  12. `xferBool( &weaponPresent )`
  13. `xferSnapshot( m_continuousWeaponDamaged )`
  14. `xferBool( &weaponPresent )`
  15. `xferSnapshot( m_continuousWeaponReallyDamaged )`
  16. `xferBool( &weaponPresent )`
  17. `xferSnapshot( m_continuousWeaponRubble )`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/FireWeaponWhenDamagedBehavior.h`
```cpp
// FILE: FireWeaponWhenDamagedBehavior.h /////////////////////////////////////////////////////////////////////////
// Author: Steven Johnson, June 2002
// Desc:   Update that will count down a lifetime and destroy object when it reaches zero
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/UpgradeModule.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Module/DamageModule.h"
#include "GameLogic/Weapon.h"

//-------------------------------------------------------------------------------------------------
class FireWeaponWhenDamagedBehaviorModuleData : public UpdateModuleData
{
public:
	UpgradeMuxData				m_upgradeMuxData;
	Bool									m_initiallyActive;
	DamageTypeFlags				m_damageTypes;
	Real									m_damageAmount;
	const WeaponTemplate* m_reactionWeaponPristine;///< fire these weapons only when damage is received
	const WeaponTemplate* m_reactionWeaponDamaged;
	const WeaponTemplate* m_reactionWeaponReallyDamaged;
	const WeaponTemplate* m_reactionWeaponRubble;
	const WeaponTemplate*	m_continuousWeaponPristine;///< fire these weapons continuously, versus just onDamage
	const WeaponTemplate*	m_continuousWeaponDamaged;
	const WeaponTemplate*	m_continuousWeaponReallyDamaged;
	const WeaponTemplate*	m_continuousWeaponRubble;

	FireWeaponWhenDamagedBehaviorModuleData()
	{
		m_initiallyActive = false;
		m_reactionWeaponPristine = nullptr;
		m_reactionWeaponDamaged = nullptr;
		m_reactionWeaponReallyDamaged = nullptr;
		m_reactionWeaponRubble = nullptr;
		m_continuousWeaponPristine = nullptr;
		m_continuousWeaponDamaged = nullptr;
		m_continuousWeaponReallyDamaged = nullptr;
		m_continuousWeaponRubble = nullptr;
		m_damageTypes = DAMAGE_TYPE_FLAGS_ALL;
		m_damageAmount = 0;
	}


	static void buildFieldParse(MultiIniFieldParse& p)
	{
		static const FieldParse dataFieldParse[] =
		{
			{ "StartsActive",	INI::parseBool, nullptr, offsetof( FireWeaponWhenDamagedBehaviorModuleData, m_initiallyActive ) },
			{ "ReactionWeaponPristine", INI::parseWeaponTemplate, nullptr, offsetof(FireWeaponWhenDamagedBehaviorModuleData,				m_reactionWeaponPristine) },
			{ "ReactionWeaponDamaged", INI::parseWeaponTemplate, nullptr, offsetof(FireWeaponWhenDamagedBehaviorModuleData,				m_reactionWeaponDamaged) },
			{ "ReactionWeaponReallyDamaged", INI::parseWeaponTemplate, nullptr, offsetof(FireWeaponWhenDamagedBehaviorModuleData,	m_reactionWeaponReallyDamaged) },
			{ "ReactionWeaponRubble", INI::parseWeaponTemplate, nullptr, offsetof(FireWeaponWhenDamagedBehaviorModuleData,					m_reactionWeaponRubble) },
			{ "ContinuousWeaponPristine", INI::parseWeaponTemplate, nullptr, offsetof(FireWeaponWhenDamagedBehaviorModuleData,			m_continuousWeaponPristine) },
			{ "ContinuousWeaponDamaged", INI::parseWeaponTemplate, nullptr, offsetof(FireWeaponWhenDamagedBehaviorModuleData,			m_continuousWeaponDamaged) },
			{ "ContinuousWeaponReallyDamaged", INI::parseWeaponTemplate, nullptr, offsetof(FireWeaponWhenDamagedBehaviorModuleData,m_continuousWeaponReallyDamaged) },
			{ "ContinuousWeaponRubble", INI::parseWeaponTemplate, nullptr, offsetof(FireWeaponWhenDamagedBehaviorModuleData,				m_continuousWeaponRubble) },
			{ "DamageTypes", INI::parseDamageTypeFlags, nullptr, offsetof( FireWeaponWhenDamagedBehaviorModuleData, m_damageTypes ) },
			{ "DamageAmount", INI::parseReal, nullptr, offsetof( FireWeaponWhenDamagedBehaviorModuleData, m_damageAmount ) },
			{ nullptr, nullptr, nullptr, 0 }
		};

		UpdateModuleData::buildFieldParse(p);
		p.add(dataFieldParse);
		p.add(UpgradeMuxData::getFieldParse(), offsetof( FireWeaponWhenDamagedBehaviorModuleData, m_upgradeMuxData ));
	}


private:

};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class FireWeaponWhenDamagedBehavior : public UpdateModule,
																			public UpgradeMux,
																			public DamageModuleInterface
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( FireWeaponWhenDamagedBehavior, "FireWeaponWhenDamagedBehavior" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( FireWeaponWhenDamagedBehavior, FireWeaponWhenDamagedBehaviorModuleData )

public:

	FireWeaponWhenDamagedBehavior( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	// module methods
	static Int getInterfaceMask() { return UpdateModule::getInterfaceMask() | (MODULEINTERFACE_UPGRADE) | (MODULEINTERFACE_DAMAGE); }

	// BehaviorModule
	virtual UpgradeModuleInterface* getUpgrade() override { return this; }
	virtual DamageModuleInterface* getDamage() override { return this; }

	// DamageModuleInterface
	virtual void onDamage( DamageInfo *damageInfo ) override;
	virtual void onHealing( DamageInfo *damageInfo ) override { }
	virtual void onBodyDamageStateChange(const DamageInfo* damageInfo, BodyDamageType oldState, BodyDamageType newState) override { }

	// UpdateModuleInterface
	virtual UpdateSleepTime update() override;

protected:

	virtual void upgradeImplementation() override
	{
		setWakeFrame(getObject(), UPDATE_SLEEP_NONE);
	}

	virtual void getUpgradeActivationMasks(UpgradeMaskType& activation, UpgradeMaskType& conflicting) const override
	{
		getFireWeaponWhenDamagedBehaviorModuleData()->m_upgradeMuxData.getUpgradeActivationMasks(activation, conflicting);
	}

	virtual void performUpgradeFX() override
	{
		getFireWeaponWhenDamagedBehaviorModuleData()->m_upgradeMuxData.performUpgradeFX(getObject());
	}

	virtual void processUpgradeRemoval() override
	{
		// I can't take it any more.  Let the record show that I think the UpgradeMux multiple inheritance is CRAP.
		getFireWeaponWhenDamagedBehaviorModuleData()->m_upgradeMuxData.muxDataProcessUpgradeRemoval(getObject());
	}

	virtual Bool requiresAllActivationUpgrades() const override
	{
		return getFireWeaponWhenDamagedBehaviorModuleData()->m_upgradeMuxData.m_requiresAllTriggers;
	}

	Bool isUpgradeActive() const { return isAlreadyUpgraded(); }

	virtual Bool isSubObjectsUpgrade() override { return false; }

private:
	Weapon *m_reactionWeaponPristine;
	Weapon *m_reactionWeaponDamaged;
	Weapon *m_reactionWeaponReallyDamaged;
	Weapon *m_reactionWeaponRubble;
	Weapon *m_continuousWeaponPristine;
	Weapon *m_continuousWeaponDamaged;
	Weapon *m_continuousWeaponReallyDamaged;
	Weapon *m_continuousWeaponRubble;

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Behavior/FireWeaponWhenDamagedBehavior.cpp`
```cpp
// FILE: FireWeaponWhenDamagedBehavior.cpp ///////////////////////////////////////////////////////////////////////
// Author:
// Desc:
///////////////////////////////////////////////////////////////////////////////////////////////////


// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine
#define DEFINE_SLOWDEATHPHASE_NAMES

#include "Common/Thing.h"
#include "Common/ThingTemplate.h"
#include "Common/INI.h"
#include "Common/RandomValue.h"
#include "Common/Xfer.h"
#include "GameClient/Drawable.h"
#include "GameClient/FXList.h"
#include "GameClient/InGameUI.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/FireWeaponWhenDamagedBehavior.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectCreationList.h"
#include "GameLogic/Weapon.h"

const Int MAX_IDX = 32;

const Real BEGIN_MIDPOINT_RATIO = 0.35f;
const Real END_MIDPOINT_RATIO = 0.65f;

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
FireWeaponWhenDamagedBehavior::FireWeaponWhenDamagedBehavior( Thing *thing, const ModuleData* moduleData ) :
	UpdateModule( thing, moduleData ),
	m_reactionWeaponPristine( nullptr ),
	m_reactionWeaponDamaged( nullptr ),
	m_reactionWeaponReallyDamaged( nullptr ),
	m_reactionWeaponRubble( nullptr ),
	m_continuousWeaponPristine( nullptr ),
	m_continuousWeaponDamaged( nullptr ),
	m_continuousWeaponReallyDamaged( nullptr ),
	m_continuousWeaponRubble( nullptr )
{

	const FireWeaponWhenDamagedBehaviorModuleData *d = getFireWeaponWhenDamagedBehaviorModuleData();
	const Object* obj = getObject();

	if ( d->m_reactionWeaponPristine )
	{
		m_reactionWeaponPristine				= TheWeaponStore->allocateNewWeapon(
			d->m_reactionWeaponPristine,					PRIMARY_WEAPON);
		m_reactionWeaponPristine->reloadAmmo( obj );
	}
	if ( d->m_reactionWeaponDamaged )
	{
		m_reactionWeaponDamaged					= TheWeaponStore->allocateNewWeapon(
			d->m_reactionWeaponDamaged,					PRIMARY_WEAPON);
		m_reactionWeaponDamaged->reloadAmmo( obj );
	}
	if ( d->m_reactionWeaponReallyDamaged )
	{
		m_reactionWeaponReallyDamaged		= TheWeaponStore->allocateNewWeapon(
			d->m_reactionWeaponReallyDamaged,		PRIMARY_WEAPON);
		m_reactionWeaponReallyDamaged->reloadAmmo( obj );
	}
	if ( d->m_reactionWeaponRubble )
	{
		m_reactionWeaponRubble					= TheWeaponStore->allocateNewWeapon(
			d->m_reactionWeaponRubble,						PRIMARY_WEAPON);
		m_reactionWeaponRubble->reloadAmmo( obj );
	}


	if ( d->m_continuousWeaponPristine )
	{
		m_continuousWeaponPristine			= TheWeaponStore->allocateNewWeapon(
			d->m_continuousWeaponPristine,				PRIMARY_WEAPON);
		m_continuousWeaponPristine->reloadAmmo( obj );
	}
	if ( d->m_continuousWeaponDamaged )
	{
		m_continuousWeaponDamaged				= TheWeaponStore->allocateNewWeapon(
			d->m_continuousWeaponDamaged,				PRIMARY_WEAPON);
		m_continuousWeaponDamaged->reloadAmmo( obj );
	}
	if ( d->m_continuousWeaponReallyDamaged )
	{
		m_continuousWeaponReallyDamaged = TheWeaponStore->allocateNewWeapon(
			d->m_continuousWeaponReallyDamaged,	PRIMARY_WEAPON);
		m_continuousWeaponReallyDamaged->reloadAmmo( obj );
	}
	if ( d->m_continuousWeaponRubble )
	{
		m_continuousWeaponRubble				= TheWeaponStore->allocateNewWeapon(
			d->m_continuousWeaponRubble,					PRIMARY_WEAPON);
		m_continuousWeaponRubble->reloadAmmo( obj );
	}

	if (d->m_initiallyActive)
	{
		giveSelfUpgrade();
	}

	if (isUpgradeActive() &&
			(d->m_continuousWeaponPristine != nullptr ||
			d->m_continuousWeaponDamaged != nullptr ||
			d->m_continuousWeaponReallyDamaged != nullptr ||
			d->m_continuousWeaponRubble != nullptr))
	{
		setWakeFrame(getObject(), UPDATE_SLEEP_NONE);
	}
	else
	{
		setWakeFrame(getObject(), UPDATE_SLEEP_FOREVER);
	}

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
FireWeaponWhenDamagedBehavior::~FireWeaponWhenDamagedBehavior()
{
	deleteInstance(m_reactionWeaponPristine);
	deleteInstance(m_reactionWeaponDamaged);
	deleteInstance(m_reactionWeaponReallyDamaged);
	deleteInstance(m_reactionWeaponRubble);

	deleteInstance(m_continuousWeaponPristine);
	deleteInstance(m_continuousWeaponDamaged);
	deleteInstance(m_continuousWeaponReallyDamaged);
	deleteInstance(m_continuousWeaponRubble);
}

//-------------------------------------------------------------------------------------------------
/** Damage has been dealt, this is an opportunity to reach to that damage */
//-------------------------------------------------------------------------------------------------
void FireWeaponWhenDamagedBehavior::onDamage( DamageInfo *damageInfo )
{
	if (!isUpgradeActive())
		return;

	const FireWeaponWhenDamagedBehaviorModuleData* d = getFireWeaponWhenDamagedBehaviorModuleData();

	// right type?
	if (!getDamageTypeFlag(d->m_damageTypes, damageInfo->in.m_damageType))
		return;

	// right amount? (use actual [post-armor] damage dealt)
	if (damageInfo->out.m_actualDamageDealt < d->m_damageAmount)
		return;

	const Object *obj = getObject();
	BodyDamageType bdt = obj->getBodyModule()->getDamageState();

	if ( bdt == BODY_RUBBLE )
	{
		if( m_reactionWeaponRubble && m_reactionWeaponRubble->getStatus() == READY_TO_FIRE )
		{
			m_reactionWeaponRubble->forceFireWeapon( obj, obj->getPosition() );
		}

	}
	else if ( bdt == BODY_REALLYDAMAGED )
	{
		if( m_reactionWeaponReallyDamaged && m_reactionWeaponReallyDamaged->getStatus() == READY_TO_FIRE )
		{
			m_reactionWeaponReallyDamaged->forceFireWeapon( obj, obj->getPosition() );
		}
	}
	else if ( bdt == BODY_DAMAGED )
	{
		if( m_reactionWeaponDamaged && m_reactionWeaponDamaged->getStatus() == READY_TO_FIRE )
		{
			m_reactionWeaponDamaged->forceFireWeapon( obj, obj->getPosition() );
		}
	}
	else // not damaged yet
	{
		if( m_reactionWeaponPristine && m_reactionWeaponPristine->getStatus() == READY_TO_FIRE )
		{
			m_reactionWeaponPristine->forceFireWeapon( obj, obj->getPosition() );
		}
	}

}

//-------------------------------------------------------------------------------------------------
/** if object fires weapon constantly, figure out which one and do it */
//-------------------------------------------------------------------------------------------------
UpdateSleepTime FireWeaponWhenDamagedBehavior::update()
{
	if (!isUpgradeActive())
	{
		DEBUG_ASSERTCRASH(isUpgradeActive(), ("hmm, this should not be possible"));
		return UPDATE_SLEEP_FOREVER;
	}

	const Object *obj = getObject();
	BodyDamageType bdt = obj->getBodyModule()->getDamageState();

	if ( bdt == BODY_RUBBLE )
	{
		if( m_continuousWeaponRubble && m_continuousWeaponRubble->getStatus() == READY_TO_FIRE )
		{
			m_continuousWeaponRubble->forceFireWeapon( obj, obj->getPosition() );
		}

	}
	else if ( bdt == BODY_REALLYDAMAGED )
	{
		if( m_continuousWeaponReallyDamaged && m_continuousWeaponReallyDamaged->getStatus() == READY_TO_FIRE )
		{
			m_continuousWeaponReallyDamaged->forceFireWeapon( obj, obj->getPosition() );
		}
	}
	else if ( bdt == BODY_DAMAGED )
	{
		if( m_continuousWeaponDamaged && m_continuousWeaponDamaged->getStatus() == READY_TO_FIRE )
		{
			m_continuousWeaponDamaged->forceFireWeapon( obj, obj->getPosition() );
		}
	}
	else // not damaged yet
	{
		if( m_continuousWeaponPristine && m_continuousWeaponPristine->getStatus() == READY_TO_FIRE )
		{
			m_continuousWeaponPristine->forceFireWeapon( obj, obj->getPosition() );
		}
	}

	return UPDATE_SLEEP_NONE;
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void FireWeaponWhenDamagedBehavior::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

	// extend upgrade mux
	UpgradeMux::upgradeMuxCRC( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void FireWeaponWhenDamagedBehavior::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// extend upgrade mux
	UpgradeMux::upgradeMuxXfer( xfer );

	Bool weaponPresent;

	// reaction pristine
	weaponPresent = m_reactionWeaponPristine ? TRUE : FALSE;
	xfer->xferBool( &weaponPresent );
	if( weaponPresent )
		xfer->xferSnapshot( m_reactionWeaponPristine );

	// reaction damaged
	weaponPresent = m_reactionWeaponDamaged ? TRUE : FALSE;
	xfer->xferBool( &weaponPresent );
	if( weaponPresent )
		xfer->xferSnapshot( m_reactionWeaponDamaged );

	// reaction really damaged
	weaponPresent = m_reactionWeaponReallyDamaged ? TRUE : FALSE;
	xfer->xferBool( &weaponPresent );
	if( weaponPresent )
		xfer->xferSnapshot( m_reactionWeaponReallyDamaged );

	// reaction rubble
	weaponPresent = m_reactionWeaponRubble ? TRUE : FALSE;
	xfer->xferBool( &weaponPresent );
	if( weaponPresent )
		xfer->xferSnapshot( m_reactionWeaponRubble );

	// continuous pristine
	weaponPresent = m_continuousWeaponPristine ? TRUE : FALSE;
	xfer->xferBool( &weaponPresent );
	if( weaponPresent )
		xfer->xferSnapshot( m_continuousWeaponPristine );

	// continuous damaged
	weaponPresent = m_continuousWeaponDamaged ? TRUE : FALSE;
	xfer->xferBool( &weaponPresent );
	if( weaponPresent )
		xfer->xferSnapshot( m_continuousWeaponDamaged );

	// continuous really damaged
	weaponPresent = m_continuousWeaponReallyDamaged ? TRUE : FALSE;
	xfer->xferBool( &weaponPresent );
	if( weaponPresent )
		xfer->xferSnapshot( m_continuousWeaponReallyDamaged );

	// continuous rubble
	weaponPresent = m_continuousWeaponRubble ? TRUE : FALSE;
	xfer->xferBool( &weaponPresent );
	if( weaponPresent )
		xfer->xferSnapshot( m_continuousWeaponRubble );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void FireWeaponWhenDamagedBehavior::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

	// extend upgrade mux
	UpgradeMux::upgradeMuxLoadPostProcess();

}
```
