# Translation Packet `PKT-266`: `CountermeasuresBehavior`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-266`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/behavior/countermeasures_behavior.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (21):**
  - `Common/BitFlagsIO.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/BitFlagsIO.h`
  - `Common/INI.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/INI.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/Thing.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Thing.h`
  - `Common/ThingFactory.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ThingFactory.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Anim2D.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/Anim2D.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameClient/ParticleSys.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/ParticleSys.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/CountermeasuresBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CountermeasuresBehavior.h`
  - `GameLogic/Module/DamageModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DamageModule.h`
  - `GameLogic/Module/PhysicsUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PhysicsUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Module/UpgradeModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpgradeModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `PreRTS.h`
- **Implemented Module Facets (2):**
  - `CountermeasuresBehaviorInterface* getCountermeasuresBehaviorInterface()`
  - `UpgradeModuleInterface* getUpgrade()`
- **INI `FieldParse` Table Entries (12):**
  - `"FlareTemplateName" => INI::parseAsciiString`
  - `"FlareBoneBaseName" => INI::parseAsciiString`
  - `"VolleySize" => INI::parseUnsignedInt`
  - `"VolleyArcAngle" => INI::parseAngleReal`
  - `"VolleyVelocityFactor" => INI::parseReal`
  - `"DelayBetweenVolleys" => INI::parseDurationUnsignedInt`
  - `"NumberOfVolleys" => INI::parseUnsignedInt`
  - `"ReloadTime" => INI::parseDurationUnsignedInt`
  - `"EvasionRate" => INI::parsePercentToReal`
  - `"MustReloadAtAirfield" => INI::parseBool`
  - `"MissileDecoyDelay" => INI::parseDurationUnsignedInt`
  - `"ReactionLaunchLatency" => INI::parseDurationUnsignedInt`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (8 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferSTLObjectIDVector( &m_counterMeasures )`
  3. `xferUnsignedInt( &m_availableCountermeasures )`
  4. `xferUnsignedInt( &m_activeCountermeasures )`
  5. `xferUnsignedInt( &m_divertedMissiles )`
  6. `xferUnsignedInt( &m_incomingMissiles )`
  7. `xferUnsignedInt( &m_reactionFrame )`
  8. `xferUnsignedInt( &m_nextVolleyFrame )`
- **PRNG Calls (1 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValueReal( 0.0f, 1.0f ) < data->m_evasionRate )`
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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CountermeasuresBehavior.h`
```cpp
// FILE: CountermeasuresBehavior.h /////////////////////////////////////////////////////////////////////////
// Author: Kris Morness, April 2003
// Desc: Handles countermeasure firing when under missile threat, and responsible
//       for diverting missiles to the flares.
//------------------------------------------

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
class CountermeasuresBehaviorModuleData : public UpdateModuleData
{
public:
	UpgradeMuxData				m_upgradeMuxData;
	AsciiString						m_flareTemplateName;
	AsciiString						m_flareBoneBaseName;
  Real									m_evasionRate;
  UnsignedInt						m_volleySize;
  Real									m_volleyArcAngle;
	Real									m_volleyVelocityFactor;
  UnsignedInt						m_framesBetweenVolleys;
	UnsignedInt						m_numberOfVolleys;
  UnsignedInt						m_reloadFrames;
	UnsignedInt						m_missileDecoyFrames;
	UnsignedInt						m_countermeasureReactionFrames;
	Bool									m_mustReloadAtAirfield;

	CountermeasuresBehaviorModuleData()
	{
    m_volleySize            = 0;
		m_volleyArcAngle				= 0.0f;
    m_framesBetweenVolleys  = 0;
		m_numberOfVolleys       = 0;
    m_reloadFrames          = 0;
    m_evasionRate           = 0.0f;
		m_mustReloadAtAirfield	= FALSE;
		m_missileDecoyFrames		= 0;
		m_volleyVelocityFactor  = 1.0f;
	}

	static void buildFieldParse(MultiIniFieldParse& p)
	{
		static const FieldParse dataFieldParse[] =
		{
			{ "FlareTemplateName",			INI::parseAsciiString,					nullptr, offsetof( CountermeasuresBehaviorModuleData, m_flareTemplateName ) },
			{ "FlareBoneBaseName",			INI::parseAsciiString,					nullptr, offsetof( CountermeasuresBehaviorModuleData, m_flareBoneBaseName ) },
			{ "VolleySize",							INI::parseUnsignedInt,					nullptr, offsetof( CountermeasuresBehaviorModuleData, m_volleySize ) },
			{ "VolleyArcAngle",					INI::parseAngleReal,						nullptr, offsetof( CountermeasuresBehaviorModuleData, m_volleyArcAngle ) },
			{ "VolleyVelocityFactor",		INI::parseReal,						nullptr, offsetof( CountermeasuresBehaviorModuleData, m_volleyVelocityFactor ) },
			{ "DelayBetweenVolleys",		INI::parseDurationUnsignedInt,  nullptr, offsetof( CountermeasuresBehaviorModuleData, m_framesBetweenVolleys ) },
			{ "NumberOfVolleys",				INI::parseUnsignedInt,					nullptr, offsetof( CountermeasuresBehaviorModuleData, m_numberOfVolleys ) },
			{ "ReloadTime",							INI::parseDurationUnsignedInt,  nullptr, offsetof( CountermeasuresBehaviorModuleData, m_reloadFrames ) },
			{ "EvasionRate",						INI::parsePercentToReal,				nullptr, offsetof( CountermeasuresBehaviorModuleData, m_evasionRate ) },
			{ "MustReloadAtAirfield",		INI::parseBool,									nullptr, offsetof( CountermeasuresBehaviorModuleData, m_mustReloadAtAirfield ) },
			{ "MissileDecoyDelay",			INI::parseDurationUnsignedInt,	nullptr, offsetof( CountermeasuresBehaviorModuleData, m_missileDecoyFrames ) },
			{ "ReactionLaunchLatency",	INI::parseDurationUnsignedInt,	nullptr, offsetof( CountermeasuresBehaviorModuleData, m_countermeasureReactionFrames ) },
			{ nullptr, nullptr, nullptr, 0 }
		};

		UpdateModuleData::buildFieldParse(p);
		p.add(dataFieldParse);
		p.add(UpgradeMuxData::getFieldParse(), offsetof( CountermeasuresBehaviorModuleData, m_upgradeMuxData ));
	}

};


// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
class CountermeasuresBehaviorInterface
{
public:
	virtual void reportMissileForCountermeasures( Object *missile ) = 0;
	virtual ObjectID calculateCountermeasureToDivertTo( const Object& victim ) = 0;
	virtual void reloadCountermeasures() = 0;
	virtual Bool isActive() const = 0;
};


typedef std::vector<ObjectID> CountermeasuresVec;

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class CountermeasuresBehavior : public UpdateModule, public UpgradeMux, public CountermeasuresBehaviorInterface
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( CountermeasuresBehavior, "CountermeasuresBehavior" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( CountermeasuresBehavior, CountermeasuresBehaviorModuleData )

public:

	CountermeasuresBehavior( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	// module methods
	static Int getInterfaceMask() { return UpdateModule::getInterfaceMask() | MODULEINTERFACE_UPGRADE; }

	// BehaviorModule
	virtual UpgradeModuleInterface* getUpgrade() override { return this; }
	virtual CountermeasuresBehaviorInterface* getCountermeasuresBehaviorInterface() override { return this; }
	virtual const CountermeasuresBehaviorInterface* getCountermeasuresBehaviorInterface() const override { return this; }

	// UpdateModuleInterface
	virtual UpdateSleepTime update() override;
	virtual DisabledMaskType getDisabledTypesToProcess() const override { return MAKE_DISABLED_MASK( DISABLED_HELD ); }

	// CountermeasuresBehaviorInterface
	virtual void reportMissileForCountermeasures( Object *missile ) override;
	virtual ObjectID calculateCountermeasureToDivertTo( const Object& victim ) override;
	virtual void reloadCountermeasures() override;
	virtual Bool isActive() const override;


protected:

	virtual void upgradeImplementation() override
	{
		setWakeFrame(getObject(), UPDATE_SLEEP_NONE);
	}

	virtual void getUpgradeActivationMasks(UpgradeMaskType& activation, UpgradeMaskType& conflicting) const override
	{
		getCountermeasuresBehaviorModuleData()->m_upgradeMuxData.getUpgradeActivationMasks(activation, conflicting);
	}

	virtual void performUpgradeFX() override
	{
		getCountermeasuresBehaviorModuleData()->m_upgradeMuxData.performUpgradeFX(getObject());
	}

	virtual void processUpgradeRemoval() override
	{
		// I can't take it any more.  Let the record show that I think the UpgradeMux multiple inheritance is CRAP.
		getCountermeasuresBehaviorModuleData()->m_upgradeMuxData.muxDataProcessUpgradeRemoval(getObject());
	}

	virtual Bool requiresAllActivationUpgrades() const override
	{
		return getCountermeasuresBehaviorModuleData()->m_upgradeMuxData.m_requiresAllTriggers;
	}

	Bool isUpgradeActive() const { return isAlreadyUpgraded(); }

	virtual Bool isSubObjectsUpgrade() override { return false; }

	void launchVolley();

private:
	CountermeasuresVec m_counterMeasures;		//vector of countermeasures in the world.
	UnsignedInt m_availableCountermeasures;	//number of countermeasures that can be launched to divert missiles.
	UnsignedInt m_activeCountermeasures;		//number of countermeasures currently able to divert missiles.
	UnsignedInt m_divertedMissiles;					//number of missiles that have been diverted to countermeasures.
	UnsignedInt m_incomingMissiles;					//grand total of all missiles that were ever fired at me.
	UnsignedInt m_reactionFrame;						//The frame countermeasures will be launched after initial hostile act.
	UnsignedInt m_nextVolleyFrame;					//Frame the next volley is fired.
	UnsignedInt m_reloadFrame;							//The frame countermeasures will be ready to use again.
};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Behavior/CountermeasuresBehavior.cpp`
```cpp
// FILE: CountermeasuresBehavior.cpp //////////////////////////////////////////////////////////////
// Author: Kris Morness, April 2003
// Desc: Handles countermeasure firing when under missile threat, and responsible
//       for diverting missiles to the flares.
///////////////////////////////////////////////////////////////////////////////////////////////////


// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine
#include "Common/Thing.h"
#include "Common/ThingTemplate.h"
#include "Common/INI.h"
#include "Common/Player.h"
#include "Common/ThingFactory.h"
#include "Common/Xfer.h"
#include "GameClient/ParticleSys.h"
#include "GameClient/Anim2D.h"
#include "GameClient/InGameUI.h"
#include "GameLogic/Module/CountermeasuresBehavior.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"


//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
struct CountermeasuresPlayerScanHelper
{
	KindOfMaskType m_kindOfToTest;
	Object *m_theHealer;
	ObjectPointerList *m_objectList;
};

static void checkForCountermeasures( Object *testObj, void *userData )
{
	CountermeasuresPlayerScanHelper *helper = (CountermeasuresPlayerScanHelper*)userData;
	ObjectPointerList *listToAddTo = helper->m_objectList;

	if( testObj->isEffectivelyDead() )
		return;

	if( testObj->getControllingPlayer() != helper->m_theHealer->getControllingPlayer() )
		return;

	if( testObj->isOffMap() )
		return;

	if( !testObj->isAnyKindOf(helper->m_kindOfToTest) )
		return;

	if( testObj->getBodyModule()->getHealth() >= testObj->getBodyModule()->getMaxHealth() )
		return;

	listToAddTo->push_back(testObj);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
CountermeasuresBehavior::CountermeasuresBehavior( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{
	const CountermeasuresBehaviorModuleData *data = getCountermeasuresBehaviorModuleData();
	m_availableCountermeasures = data->m_numberOfVolleys * data->m_volleySize;
	m_reactionFrame = 0;
	m_activeCountermeasures = 0;
	m_divertedMissiles = 0;
	m_incomingMissiles = 0;
	m_nextVolleyFrame = 0;

	setWakeFrame( getObject(), UPDATE_SLEEP_NONE );
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
CountermeasuresBehavior::~CountermeasuresBehavior()
{
}

// ------------------------------------------------------------------------------------------------
void CountermeasuresBehavior::reportMissileForCountermeasures( Object *missile )
{
	if( !missile )
	{
		return;
	}

	//Record the number of missiles that have been fired at us
	m_incomingMissiles++;

  if( m_availableCountermeasures + m_activeCountermeasures > 0 )
	{
		//We have countermeasures we can use. Determine now whether or not the incoming missile will
		//be diverted.
		const CountermeasuresBehaviorModuleData *data = getCountermeasuresBehaviorModuleData();

		if( GameLogicRandomValueReal( 0.0f, 1.0f ) < data->m_evasionRate )
		{
			//This missile will be diverted!
			ProjectileUpdateInterface* pui = nullptr;
			for( BehaviorModule** u = missile->getBehaviorModules(); *u; ++u )
			{
				if( (pui = (*u)->getProjectileUpdateInterface()) != nullptr )
				{
					//Make sure the missile diverts after a delay. The delay needs to be larger than
					//the countermeasure reaction time or else the missile won't have a countermeasure to divert to!
					DEBUG_ASSERTCRASH( data->m_countermeasureReactionFrames < data->m_missileDecoyFrames,
						("MissileDecoyDelay needs to be less than CountermeasureReactionTime in order to function properly.") );
					pui->setFramesTillCountermeasureDiversionOccurs( data->m_missileDecoyFrames );
					m_divertedMissiles++;

					if( m_activeCountermeasures == 0 && m_reactionFrame == 0 )
					{
						//We need to launch our first volley of countermeasures, but we can't do it now. If we
						//do, it'll look too artificial. Instead, we need to set up a timer to fake a reaction
						//delay.
						m_reactionFrame = TheGameLogic->getFrame() + data->m_countermeasureReactionFrames;
					}
					break;
				}
			}
		}
	}
}

//-------------------------------------------------------------------------------------------------
ObjectID CountermeasuresBehavior::calculateCountermeasureToDivertTo( const Object& victim )
{
	const CountermeasuresBehaviorModuleData *data = getCountermeasuresBehaviorModuleData();

	// TheSuperHackers @bugfix Mauller/Stubbjax 27/07/2025 Fix unsafe iterator handling and correct the countermeasures behavior
	// The code now iterates through all flares in the volley and selects the nearest one to the weapon as intended in the original EA code
	// This can slightly change behavior but does not significantly impact the overall survivability of the aircraft

	Real closestFlareDist = 1e15f;
	Object *closestFlare = nullptr;

	const UnsignedInt volleySize = max(data->m_volleySize, 1u);
	UnsignedInt volleyFlaresCounted = 0;

	//Flares are pushed to the front of the list, but we only want to acquire the "newest" of the flares, therefore
	//Start at the end of the list and go towards the beginning.
	CountermeasuresVec::reverse_iterator it = m_counterMeasures.rbegin();
	//stop iterating after we've reached size of a single volley.
	while( it != m_counterMeasures.rend() && volleyFlaresCounted < volleySize )
	{
		Object *obj = TheGameLogic->findObjectByID( *it++ );
		if( obj )
		{
			Real weaponToFlareDist = ThePartitionManager->getDistanceSquared( obj, getObject(), FROM_CENTER_2D );
			if( weaponToFlareDist < closestFlareDist )
			{
				closestFlareDist = weaponToFlareDist;
				closestFlare = obj;
			}
#if RETAIL_COMPATIBLE_CRC
			// TheSuperHackers @info Original EA code did not work as intended and stopped on the first retrieved flare
			// The non retail behaviour corrects the code to iterate through all flares in the volley
			break;
#else
			++volleyFlaresCounted;
#endif
		}

#if RETAIL_COMPATIBLE_CRC
		++volleyFlaresCounted;
#endif
	}

	if( closestFlare )
	{
		return closestFlare->getID();
	}
	return INVALID_ID;
}

//-------------------------------------------------------------------------------------------------
Bool CountermeasuresBehavior::isActive() const
{
	return isUpgradeActive();
}

//-------------------------------------------------------------------------------------------------
/** The update callback. */
//-------------------------------------------------------------------------------------------------
UpdateSleepTime CountermeasuresBehavior::update()
{
	UnsignedInt now = TheGameLogic->getFrame();
	const CountermeasuresBehaviorModuleData *data = getCountermeasuresBehaviorModuleData();
	Object *obj = getObject();

	if( obj->isEffectivelyDead() )
	{
		return UPDATE_SLEEP_FOREVER;
	}
	if( !isUpgradeActive()  )
	{
		return UPDATE_SLEEP_FOREVER;
	}

	//Validate all existing flares, and clean them up as needed.
	for (CountermeasuresVec::iterator it = m_counterMeasures.begin(); it != m_counterMeasures.end(); /*nothing*/ )
	{
		Object *obj = TheGameLogic->findObjectByID( *it );
		if( !obj )
		{
			it = m_counterMeasures.erase( it );
			m_activeCountermeasures--;
		}
		else
		{
			++it;
		}
	}

	if( obj->isAirborneTarget() )
	{

		//Handle flare volley launching (initial reaction, and continuation firing).
		if( m_availableCountermeasures )
		{
			//Deal with the initial volley, but wait until we are permitted to react.
			if( m_reactionFrame )
			{
				if( m_reactionFrame == now )
				{
					//We have been shot at and now that the reaction timer has expired, fire a full volley of
					//countermeasures.
					launchVolley();
					m_nextVolleyFrame = now + data->m_framesBetweenVolleys;
					m_reactionFrame = 0;
				}
			}

			//Handle subsequent volley launching.
			if( m_nextVolleyFrame == now )
			{
				launchVolley();
				m_nextVolleyFrame = now + data->m_framesBetweenVolleys;
			}
		}
	}

	//Handle auto-reloading (data->m_reloadFrames of zero means it's not possible to auto-reload).
	//Aircraft that don't auto-reload require landing at an airfield for resupply.
	if( !m_availableCountermeasures && data->m_reloadFrames )
	{
		if( m_reloadFrame != 0 )
		{
			if( m_reloadFrame <= now )
			{
				//We've successfully reloaded automatically.
				reloadCountermeasures();
			}
		}
		else
		{
			//We just started reloading, so set the frame it'll be ready.
			m_reloadFrame = now + data->m_reloadFrames;
		}
	}

	return UPDATE_SLEEP( UPDATE_SLEEP_NONE );
}

//-------------------------------------------------------------------------------------------------
void CountermeasuresBehavior::reloadCountermeasures()
{
	const CountermeasuresBehaviorModuleData *data = getCountermeasuresBehaviorModuleData();
	m_availableCountermeasures = data->m_numberOfVolleys * data->m_volleySize;
	m_reloadFrame = 0;
}

//-------------------------------------------------------------------------------------------------
void CountermeasuresBehavior::launchVolley()
{
	const CountermeasuresBehaviorModuleData *data = getCountermeasuresBehaviorModuleData();
	Object *obj = getObject();

	Real volleySize = (Real)data->m_volleySize;
	for( UnsignedInt i = 0; i < data->m_volleySize; i++ )
	{
		//Each flare in a volley will calculate a different vector to fly out. We have a +/- angle to
		//spread out equally. With only one flare, it'll come straight out the back. Two flares will
		//launch at the extreme positive and negative angle. Three flares will launch at extreme angles
		//plus straight back. Four or more will divvy it up equally.
		Real currentVolley = (Real)i;
		Real ratio = 0.0f;
		if( volleySize != 1.0f )
		{
			//ratio between -1.0 and +1.0f
			ratio = currentVolley / (volleySize - 1.0f) * 2.0f - 1.0f;
		}
		//Now calculate the angle. Simply multiply it by the ratio!
		Real angle = ratio * data->m_volleyArcAngle;

		Coord3D vel;
		PhysicsBehavior *physics = obj->getPhysics();

		//Calculate the angle to fire the flare by taking the facing angle and rotating it
		//and then scaling it by it's velocity (if it's moving).
		obj->getUnitDirectionVector3D( vel );
		Vector2 flareVector;
		flareVector.X = vel.x;
		flareVector.Y = vel.y;
		flareVector.Normalize();
		flareVector.Rotate( angle );
		//Give it back to the Coord3D
		vel.x = flareVector.X;
		vel.y = flareVector.Y;
		vel.z = 0.0f;

		Real velocity = physics->getVelocityMagnitude();
		if( velocity < 1.0f )
		{
			velocity = -10.0f;
		}
		vel.scale( velocity * data->m_volleyVelocityFactor );

		const ThingTemplate *thing = TheThingFactory->findTemplate( data->m_flareTemplateName );
		if( thing )
		{
			Object *flare = TheThingFactory->newObject( thing, obj->getControllingPlayer()->getDefaultTeam() );
			flare->setPosition( obj->getPosition() );
			flare->setOrientation( obj->getOrientation() );
			physics->transferVelocityTo( flare->getPhysics() );
			flare->getPhysics()->applyMotiveForce( &vel );
			m_activeCountermeasures++;
			m_availableCountermeasures--;
			m_counterMeasures.push_back( flare->getID() );
		}
	}
}


//------------------------------------------------------------------------------------------------
/** CRC */
//------------------------------------------------------------------------------------------------
void CountermeasuresBehavior::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

	// extend base class
	UpgradeMux::upgradeMuxCRC( xfer );

}

//------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
//------------------------------------------------------------------------------------------------
void CountermeasuresBehavior::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 2;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// extend base class
	UpgradeMux::upgradeMuxXfer( xfer );

	if( version >= 2 )
	{
		xfer->xferSTLObjectIDVector( &m_counterMeasures );
		xfer->xferUnsignedInt( &m_availableCountermeasures );
		xfer->xferUnsignedInt( &m_activeCountermeasures );
		xfer->xferUnsignedInt( &m_divertedMissiles );
		xfer->xferUnsignedInt( &m_incomingMissiles );
		xfer->xferUnsignedInt( &m_reactionFrame );
		xfer->xferUnsignedInt( &m_nextVolleyFrame );
	}

}

//------------------------------------------------------------------------------------------------
/** Load post process */
//------------------------------------------------------------------------------------------------
void CountermeasuresBehavior::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

	// extend base class
	UpgradeMux::upgradeMuxLoadPostProcess();

}
```
