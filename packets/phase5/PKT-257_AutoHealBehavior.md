# Translation Packet `PKT-257`: `AutoHealBehavior`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-257`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/behavior/auto_heal_behavior.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (19):**
  - `Common/BitFlagsIO.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/BitFlagsIO.h`
  - `Common/INI.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/INI.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/Thing.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Thing.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Anim2D.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/Anim2D.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameClient/ParticleSys.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/ParticleSys.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/AutoHealBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AutoHealBehavior.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/DamageModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DamageModule.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Module/UpgradeModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpgradeModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `PreRTS.h`
- **Implemented Module Facets (2):**
  - `DamageModuleInterface* getDamage()`
  - `UpgradeModuleInterface* getUpgrade()`
- **INI `FieldParse` Table Entries (12):**
  - `"StartsActive" => INI::parseBool`
  - `"SingleBurst" => INI::parseBool`
  - `"HealingAmount" => INI::parseInt`
  - `"HealingDelay" => INI::parseDurationUnsignedInt`
  - `"Radius" => INI::parseReal`
  - `"KindOf" => KindOfMaskType::parseFromINI`
  - `"ForbiddenKindOf" => KindOfMaskType::parseFromINI`
  - `"RadiusParticleSystemName" => INI::parseParticleSystemTemplate`
  - `"UnitHealPulseParticleSystemName" => INI::parseParticleSystemTemplate`
  - `"StartHealingDelay" => INI::parseDurationUnsignedInt`
  - `"AffectsWholePlayer" => INI::parseBool`
  - `"SkipSelfForHealing" => INI::parseBool`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (4 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUser( &m_radiusParticleSystemID, sizeof( ParticleSystemID ) )`
  3. `xferUnsignedInt( &m_soonestHealFrame )`
  4. `xferBool( &m_stopped )`
- **PRNG Calls (1 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValue(1, delay)))`
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if RETAIL_COMPATIBLE_CRC`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AutoHealBehavior.h`
```cpp
// FILE: AutoHealBehavior.h /////////////////////////////////////////////////////////////////////////
// Author: Colin Day, December 2001
// Desc:   Update that heals itself
//------------------------------------------
// Modified by Kris Morness, September 2002
// Kris: Added the ability to add effects, radius healing, and restricting the type of objects
//       subjected to the heal (or repair).
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameClient/ParticleSys.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/UpgradeModule.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Module/DamageModule.h"
#include "Common/BitFlagsIO.h"

class ParticleSystem;
class ParticleSystemTemplate;

//-------------------------------------------------------------------------------------------------
class AutoHealBehaviorModuleData : public UpdateModuleData
{
public:
	UpgradeMuxData				m_upgradeMuxData;
	Bool									m_initiallyActive;
	Bool									m_singleBurst;
	Int										m_healingAmount;
	UnsignedInt						m_healingDelay;
	UnsignedInt						m_startHealingDelay;	///< how long since our last damage till autoheal starts.
	Real									m_radius; //If non-zero, then it becomes a area effect.
	Bool									m_affectsWholePlayer; ///< I have more than a range, I try to affect everything the player owns
	Bool									m_skipSelfForHealing; ///< Don't heal myself.
	KindOfMaskType				m_kindOf;	//Only these types can heal -- defaults to everything.
	KindOfMaskType				m_forbiddenKindOf;	//Only these types can heal -- defaults to everything.
	const ParticleSystemTemplate*				m_radiusParticleSystemTmpl;					//Optional particle system meant to apply to entire effect for entire duration.
	const ParticleSystemTemplate*				m_unitHealPulseParticleSystemTmpl;	//Optional particle system applying to each object getting healed each heal pulse.

	AutoHealBehaviorModuleData()
	{
		m_initiallyActive = false;
		m_singleBurst = FALSE;
		m_healingAmount = 0;
		m_healingDelay = UINT_MAX;
		m_startHealingDelay = 0;
		m_radius = 0.0f;
		m_radiusParticleSystemTmpl = nullptr;
		m_unitHealPulseParticleSystemTmpl = nullptr;
		m_affectsWholePlayer = FALSE;
		m_skipSelfForHealing = FALSE;
		SET_ALL_KINDOFMASK_BITS( m_kindOf );
		m_forbiddenKindOf.clear();
	}

	static void buildFieldParse(MultiIniFieldParse& p)
	{
		static const FieldParse dataFieldParse[] =
		{
			{ "StartsActive",	INI::parseBool, nullptr, offsetof( AutoHealBehaviorModuleData, m_initiallyActive ) },
			{ "SingleBurst",	INI::parseBool, nullptr, offsetof( AutoHealBehaviorModuleData, m_singleBurst ) },
			{ "HealingAmount",		INI::parseInt,												nullptr, offsetof( AutoHealBehaviorModuleData, m_healingAmount ) },
			{ "HealingDelay",			INI::parseDurationUnsignedInt,				nullptr, offsetof( AutoHealBehaviorModuleData, m_healingDelay ) },
			{ "Radius",						INI::parseReal,												nullptr, offsetof( AutoHealBehaviorModuleData, m_radius ) },
			{ "KindOf",						KindOfMaskType::parseFromINI,											nullptr, offsetof( AutoHealBehaviorModuleData, m_kindOf ) },
			{ "ForbiddenKindOf",	KindOfMaskType::parseFromINI,											nullptr, offsetof( AutoHealBehaviorModuleData, m_forbiddenKindOf ) },
			{ "RadiusParticleSystemName",					INI::parseParticleSystemTemplate,	nullptr, offsetof( AutoHealBehaviorModuleData, m_radiusParticleSystemTmpl ) },
			{ "UnitHealPulseParticleSystemName",	INI::parseParticleSystemTemplate,	nullptr, offsetof( AutoHealBehaviorModuleData, m_unitHealPulseParticleSystemTmpl ) },
			{ "StartHealingDelay",			INI::parseDurationUnsignedInt,				nullptr, offsetof( AutoHealBehaviorModuleData, m_startHealingDelay ) },
			{ "AffectsWholePlayer",			INI::parseBool,												nullptr, offsetof( AutoHealBehaviorModuleData, m_affectsWholePlayer ) },
			{ "SkipSelfForHealing",			INI::parseBool,												nullptr, offsetof( AutoHealBehaviorModuleData, m_skipSelfForHealing ) },
			{ nullptr, nullptr, nullptr, 0 }
		};

		UpdateModuleData::buildFieldParse(p);
		p.add(dataFieldParse);
		p.add(UpgradeMuxData::getFieldParse(), offsetof( AutoHealBehaviorModuleData, m_upgradeMuxData ));
	}

};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class AutoHealBehavior : public UpdateModule,
												 public UpgradeMux,
												 public DamageModuleInterface
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( AutoHealBehavior, "AutoHealBehavior" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( AutoHealBehavior, AutoHealBehaviorModuleData )

public:

	AutoHealBehavior( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	// module methods
	static Int getInterfaceMask() { return UpdateModule::getInterfaceMask() | MODULEINTERFACE_UPGRADE | MODULEINTERFACE_DAMAGE; }

	// BehaviorModule
	virtual UpgradeModuleInterface* getUpgrade() override { return this; }
	virtual DamageModuleInterface* getDamage() override { return this; }

	// DamageModuleInterface
	virtual void onDamage( DamageInfo *damageInfo ) override;
	virtual void onHealing( DamageInfo *damageInfo ) override { }
	virtual void onBodyDamageStateChange(const DamageInfo* damageInfo, BodyDamageType oldState, BodyDamageType newState) override { }

	// UpdateModuleInterface
	virtual UpdateSleepTime update() override;
	virtual DisabledMaskType getDisabledTypesToProcess() const override { return MAKE_DISABLED_MASK( DISABLED_HELD ); }

	void stopHealing();
	void undoUpgrade(); ///<pretend like we have not been activated yet, so we can be reactivated later

protected:

	virtual void upgradeImplementation() override
	{
		setWakeFrame(getObject(), UPDATE_SLEEP_NONE);
	}

	virtual void getUpgradeActivationMasks(UpgradeMaskType& activation, UpgradeMaskType& conflicting) const override
	{
		getAutoHealBehaviorModuleData()->m_upgradeMuxData.getUpgradeActivationMasks(activation, conflicting);
	}

	virtual void performUpgradeFX() override
	{
		getAutoHealBehaviorModuleData()->m_upgradeMuxData.performUpgradeFX(getObject());
	}

	virtual void processUpgradeRemoval() override
	{
		// I can't take it any more.  Let the record show that I think the UpgradeMux multiple inheritance is CRAP.
		getAutoHealBehaviorModuleData()->m_upgradeMuxData.muxDataProcessUpgradeRemoval(getObject());
	}

	virtual Bool requiresAllActivationUpgrades() const override
	{
		return getAutoHealBehaviorModuleData()->m_upgradeMuxData.m_requiresAllTriggers;
	}

	Bool isUpgradeActive() const { return isAlreadyUpgraded(); }

	virtual Bool isSubObjectsUpgrade() override { return false; }


private:

	void pulseHealObject( Object *obj );
	void createEmitters();

	ParticleSystemID m_radiusParticleSystemID;

	UnsignedInt m_soonestHealFrame;/** I need to record this, because with multiple wake up sources,
																		I can't rely solely on my sleeping.  So this will guard onDamage's wake up.
																		I could guard the act of healing, but that would defeat the gain of being
																		a sleepy module.  I never want to run update unless I am going to heal.
																 */

	Bool m_stopped;

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Behavior/AutoHealBehavior.cpp`
```cpp
// FILE: AutoHealBehavior.cpp ///////////////////////////////////////////////////////////////////////
// Author:
// Desc:
///////////////////////////////////////////////////////////////////////////////////////////////////


// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine
#include "Common/Thing.h"
#include "Common/ThingTemplate.h"
#include "Common/INI.h"
#include "Common/Player.h"
#include "Common/Xfer.h"
#include "GameClient/ParticleSys.h"
#include "GameClient/Anim2D.h"
#include "GameClient/InGameUI.h"
#include "GameLogic/Module/AutoHealBehavior.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"


//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
struct AutoHealPlayerScanHelper
{
	KindOfMaskType m_kindOfToTest;
	KindOfMaskType m_forbiddenKindOf;
	Object *m_theHealer;
	ObjectPointerList *m_objectList;
	Bool m_skipSelfForHealing;
};

static void checkForAutoHeal( Object *testObj, void *userData )
{
	AutoHealPlayerScanHelper *helper = (AutoHealPlayerScanHelper*)userData;
	ObjectPointerList *listToAddTo = helper->m_objectList;

	if( testObj->isEffectivelyDead() )
		return;

	if( testObj->getControllingPlayer() != helper->m_theHealer->getControllingPlayer() )
		return;

	if( testObj->isOffMap() )
		return;

	if( helper->m_skipSelfForHealing && testObj == helper->m_theHealer )
		return;

	if( !testObj->isAnyKindOf(helper->m_kindOfToTest) )
		return;

	if( testObj->isAnyKindOf( helper->m_forbiddenKindOf ) )
		return;

	if( testObj->getBodyModule()->getHealth() >= testObj->getBodyModule()->getMaxHealth() )
		return;

	listToAddTo->push_back(testObj);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
AutoHealBehavior::AutoHealBehavior( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{
	const AutoHealBehaviorModuleData *d = getAutoHealBehaviorModuleData();

	m_radiusParticleSystemID = INVALID_PARTICLE_SYSTEM_ID;
	m_soonestHealFrame = 0;
	m_stopped = false;

	if (d->m_initiallyActive)
	{
		giveSelfUpgrade();
		// start these guys with random phasings so that we don't
		// have all of 'em check on the same frame.
		UnsignedInt delay = getAutoHealBehaviorModuleData()->m_healingDelay;
		setWakeFrame(getObject(), UPDATE_SLEEP(GameLogicRandomValue(1, delay)));
	}
	else
	{
		setWakeFrame(getObject(), UPDATE_SLEEP_FOREVER);
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
AutoHealBehavior::~AutoHealBehavior()
{

	if( m_radiusParticleSystemID != INVALID_PARTICLE_SYSTEM_ID )
		TheParticleSystemManager->destroyParticleSystemByID( m_radiusParticleSystemID );

}

//-------------------------------------------------------------------------------------------------
void AutoHealBehavior::stopHealing()
{
	m_stopped = true;
	m_soonestHealFrame = FOREVER;
	setWakeFrame(getObject(), UPDATE_SLEEP_FOREVER);
}

//-------------------------------------------------------------------------------------------------
void AutoHealBehavior::undoUpgrade()
{
	m_soonestHealFrame = 0;
	setUpgradeExecuted( FALSE );
}

//-------------------------------------------------------------------------------------------------
/** Damage has been dealt, this is an opportunity to reach to that damage */
//-------------------------------------------------------------------------------------------------
void AutoHealBehavior::onDamage( DamageInfo *damageInfo )
{
	if (m_stopped)
		return;

	const AutoHealBehaviorModuleData *d = getAutoHealBehaviorModuleData();
	if (isUpgradeActive() && d->m_radius == 0.0f)
	{
		// if this is nonzero, getting damaged resets our healing process. so go to
		// sleep for this long.
		if (d->m_startHealingDelay > 0)
		{
			setWakeFrame(getObject(), UPDATE_SLEEP(d->m_startHealingDelay));
		}
		else if( TheGameLogic->getFrame() > m_soonestHealFrame )
		{
			// We can only force an immediate wake if we are ready to heal.  Otherwise we will
			// heal on a timer AND at every damage input.
			setWakeFrame(getObject(), UPDATE_SLEEP_NONE);
		}
	}
}

//-------------------------------------------------------------------------------------------------
/** The update callback. */
//-------------------------------------------------------------------------------------------------
UpdateSleepTime AutoHealBehavior::update()
{
	if (m_stopped)
		return UPDATE_SLEEP_FOREVER;

	Object *obj = getObject();
	const AutoHealBehaviorModuleData *d = getAutoHealBehaviorModuleData();

	// do not heal if our status bit is not on.
	// do not heal if our status is effectively dead.  There ain't no coming back, man!
	if (!isUpgradeActive() || obj->isEffectivelyDead())
	{
		DEBUG_ASSERTCRASH(isUpgradeActive(), ("hmm, this should not be possible"));
		return UPDATE_SLEEP_FOREVER;
	}

	// TheSuperHackers @bugfix stephanmeesters 18/04/2026 Delay emitter creation until update, to ensure that the particle
	// systems are not created before ParticleManager has xfer-loaded.
	createEmitters();

//DEBUG_LOG(("doing auto heal %d",TheGameLogic->getFrame()));

	if( d->m_affectsWholePlayer )
	{
		// Even newer system, I can ignore radius and iterate objects on the owning player.  Faster than scanning range 10,000,000
		ObjectPointerList objectsToHeal;
		Player *owningPlayer = getObject()->getControllingPlayer();
		if( owningPlayer )
		{
			AutoHealPlayerScanHelper helper;
			helper.m_kindOfToTest = d->m_kindOf;
			helper.m_forbiddenKindOf = d->m_forbiddenKindOf;
			helper.m_objectList = &objectsToHeal;
			helper.m_theHealer = getObject();
			helper.m_skipSelfForHealing = d->m_skipSelfForHealing;

			// Smack all objects with this function, and we will end up with a list of Objects deserving of pulseHealObject
			owningPlayer->iterateObjects( checkForAutoHeal, &helper );

			for( ObjectPointerListIterator iter = objectsToHeal.begin(); iter != objectsToHeal.end(); ++iter )
			{
				pulseHealObject(*iter);
			}
			objectsToHeal.clear();
		}
		return UPDATE_SLEEP(d->m_healingDelay);
	}
	else if( d->m_radius == 0.0f )
	{
		//ORIGINAL SYSTEM -- JUST HEAL SELF!

		// do not heal if we are at max health already
		BodyModuleInterface *body = obj->getBodyModule();
		if( body->getHealth() < body->getMaxHealth() )
		{
  		pulseHealObject( obj );
			return UPDATE_SLEEP(d->m_healingDelay);
		}
		else
		{
			// go to sleep forever -- we'll wake back up when we are damaged again
			return UPDATE_SLEEP_FOREVER;
		}
	}
	else
	{
		//EXPANDED SYSTEM -- HEAL FRIENDLIES IN RADIUS
		// setup scan filters
		PartitionFilterRelationship relationship( obj, PartitionFilterRelationship::ALLOW_ALLIES );
		PartitionFilterSameMapStatus filterMapStatus(obj);
		PartitionFilterAlive filterAlive;
		PartitionFilter *filters[] = { &relationship, &filterAlive, &filterMapStatus, nullptr };

		// scan objects in our region
		ObjectIterator *iter = ThePartitionManager->iterateObjectsInRange( obj->getPosition(), d->m_radius, FROM_CENTER_2D, filters );
		MemoryPoolObjectHolder hold( iter );
		for( obj = iter->first(); obj; obj = iter->next() )
		{
			// do not heal if we are at max health already
			BodyModuleInterface *body = obj->getBodyModule();
			if( body->getHealth() < body->getMaxHealth() )
			{
				if( obj->isAnyKindOf( d->m_kindOf ) && !obj->isAnyKindOf( d->m_forbiddenKindOf ) )
				{
					if( !d->m_skipSelfForHealing || obj != getObject() )
					{
						pulseHealObject( obj );

						if( d->m_singleBurst && TheGameLogic->getDrawIconUI() )
						{
							if( TheAnim2DCollection && TheGlobalData->m_getHealedAnimationName.isEmpty() == FALSE )
							{
								Anim2DTemplate *animTemplate = TheAnim2DCollection->findTemplate( TheGlobalData->m_getHealedAnimationName );

								if ( animTemplate )
								{
									Coord3D iconPosition;
									iconPosition.set(obj->getPosition()->x,
																	 obj->getPosition()->y,
																	 obj->getPosition()->z + obj->getGeometryInfo().getMaxHeightAbovePosition() );
									TheInGameUI->addWorldAnimation( animTemplate,	&iconPosition, WORLD_ANIM_FADE_ON_EXPIRE,
																									TheGlobalData->m_getHealedAnimationDisplayTimeInSeconds,
																									TheGlobalData->m_getHealedAnimationZRisePerSecond);
								}
							}
						}
					}
				}
			}
		}

		return UPDATE_SLEEP( d->m_singleBurst ? UPDATE_SLEEP_FOREVER : d->m_healingDelay );
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void AutoHealBehavior::pulseHealObject( Object *obj )
{
	if (m_stopped)
		return;

	const AutoHealBehaviorModuleData *data = getAutoHealBehaviorModuleData();


#if RETAIL_COMPATIBLE_CRC
	if ( data->m_radius == 0.0f )
#else
	// TheSuperHackers @bugfix arcticdolphin 25/10/2026 Allow instant repairs while an aura is active.
	if ( data->m_radius == 0.0f || data->m_singleBurst )
#endif
		obj->attemptHealing(data->m_healingAmount, getObject());
	else
		obj->attemptHealingFromSoleBenefactor( data->m_healingAmount, getObject(), data->m_healingDelay );


	if( data->m_unitHealPulseParticleSystemTmpl )
	{
		ParticleSystem *system = TheParticleSystemManager->createParticleSystem( data->m_unitHealPulseParticleSystemTmpl );
		if( system )
		{
			system->setPosition( obj->getPosition() );
		}
	}

	m_soonestHealFrame = TheGameLogic->getFrame() + data->m_healingDelay;// In case onDamage tries to wake us up early
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void AutoHealBehavior::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

	// extend base class
	UpgradeMux::upgradeMuxCRC( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void AutoHealBehavior::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// extend base class
	UpgradeMux::upgradeMuxXfer( xfer );

	// particle system id
	xfer->xferUser( &m_radiusParticleSystemID, sizeof( ParticleSystemID ) );

	// Timer safety
	xfer->xferUnsignedInt( &m_soonestHealFrame );

	// stopped
	xfer->xferBool( &m_stopped );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void AutoHealBehavior::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

	// extend base class
	UpgradeMux::upgradeMuxLoadPostProcess();

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void AutoHealBehavior::createEmitters()
{
	if( m_radiusParticleSystemID == INVALID_PARTICLE_SYSTEM_ID )
	{
		const AutoHealBehaviorModuleData *d = getAutoHealBehaviorModuleData();
		ParticleSystem *particleSystem = TheParticleSystemManager->createParticleSystem(d->m_radiusParticleSystemTmpl);
		if( particleSystem )
		{
			particleSystem->setPosition( getObject()->getPosition() );
			m_radiusParticleSystemID = particleSystem->getSystemID();
		}
	}
}
```
