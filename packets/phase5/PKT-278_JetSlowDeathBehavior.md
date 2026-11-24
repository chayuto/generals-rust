# Translation Packet `PKT-278`: `JetSlowDeathBehavior`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-278`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/behavior/jet_slow_death_behavior.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (16):**
  - `Common/AudioEventRTS.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/AudioEventRTS.h`
  - `Common/GlobalData.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/GlobalData.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/FXList.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/FXList.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Locomotor.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Locomotor.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/JetSlowDeathBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/JetSlowDeathBehavior.h`
  - `GameLogic/Module/PhysicsUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PhysicsUpdate.h`
  - `GameLogic/Module/SlowDeathBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SlowDeathBehavior.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/ObjectCreationList.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ObjectCreationList.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (17):**
  - `"FXOnGroundDeath" => INI::parseFXList`
  - `"OCLOnGroundDeath" => INI::parseObjectCreationList`
  - `"FXInitialDeath" => INI::parseFXList`
  - `"OCLInitialDeath" => INI::parseObjectCreationList`
  - `"DelaySecondaryFromInitialDeath" => INI::parseDurationUnsignedInt`
  - `"FXSecondary" => INI::parseFXList`
  - `"OCLSecondary" => INI::parseObjectCreationList`
  - `"FXHitGround" => INI::parseFXList`
  - `"OCLHitGround" => INI::parseObjectCreationList`
  - `"DelayFinalBlowUpFromHitGround" => INI::parseDurationUnsignedInt`
  - `"FXFinalBlowUp" => INI::parseFXList`
  - `"OCLFinalBlowUp" => INI::parseObjectCreationList`
  - `"DeathLoopSound" => INI::parseAudioEventRTS`
  - `"RollRate" => INI::parseReal`
  - `"RollRateDelta" => INI::parsePercentToReal`
  - `"PitchRate" => INI::parseReal`
  - `"FallHowFast" => INI::parsePercentToReal`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (4 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUnsignedInt( &m_timerDeathFrame )`
  3. `xferUnsignedInt( &m_timerOnGroundFrame )`
  4. `xferReal( &m_rollRate )`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/JetSlowDeathBehavior.h`
```cpp
// FILE: JetSlowDeathBehavior.h ///////////////////////////////////////////////////////////////////
// Author: Colin Day
// Desc:   Death sequence for jets
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "Common/AudioEventRTS.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/SlowDeathBehavior.h"

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
class FXList;

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class JetSlowDeathBehaviorModuleData : public SlowDeathBehaviorModuleData
{

public:

	JetSlowDeathBehaviorModuleData();

	static void buildFieldParse( MultiIniFieldParse &p );

	const FXList *m_fxOnGroundDeath;							///< fx list executed on death when destroyed on ground
	const ObjectCreationList *m_oclOnGroundDeath;	///< ocl list executed on death when destroyed on ground

	const FXList *m_fxInitialDeath;								///< FXList for initial death
	const ObjectCreationList *m_oclInitialDeath;	///< OCL for initial death

	UnsignedInt m_delaySecondaryFromInitialDeath;	///< delay (in frames) from initial death, to the secondary event
	const FXList *m_fxSecondary;									///< FXList for secondary event
	const ObjectCreationList *m_oclSecondary;			///< OCL for secondary event

	const FXList *m_fxHitGround;									///< FXList for hit ground
	const ObjectCreationList *m_oclHitGround;			///< OCL for hit ground

	UnsignedInt m_delayFinalBlowUpFromHitGround;	///< delay (in frames) from hit ground, to final explosion
	const FXList *m_fxFinalBlowUp;								///< FxList for final blow up
	const ObjectCreationList *m_oclFinalBlowUp;		///< OCL for final blow up

	Real m_rollRate;															///< our initial roll rate
	Real m_rollRateDelta;													///< how our roll rate changes over time
	Real m_pitchRate;															///< spin speed on another axis after hitting the ground
	Real m_fallHowFast;														///< a fraction of gravity we use to modify the jet locmotor lift

	AudioEventRTS m_deathLoopSound;								///< looping death sound

};

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
class JetSlowDeathBehavior : public SlowDeathBehavior
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( JetSlowDeathBehavior, "JetSlowDeathBehavior" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( JetSlowDeathBehavior, JetSlowDeathBehaviorModuleData )

public:

	JetSlowDeathBehavior( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	// slow death methods
	virtual void onDie( const DamageInfo *damageInfo ) override;
	virtual void beginSlowDeath( const DamageInfo *damageInfo ) override;
	virtual UpdateSleepTime update() override;

protected:

	UnsignedInt m_timerDeathFrame;					///< fame we died on
	UnsignedInt m_timerOnGroundFrame;				///< frame we landed on the ground on

	Real m_rollRate;												///< our roll rate

	AudioEventRTS m_deathLoopSound;					///< death loop sound

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Behavior/JetSlowDeathBehavior.cpp`
```cpp
// FILE: JetSlowDeathBehavior.cpp /////////////////////////////////////////////////////////////////
// Author: Colin Day
// Desc:   Death sequence for jets
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/GlobalData.h"
#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"
#include "GameClient/FXList.h"
#include "GameClient/InGameUI.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Locomotor.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/JetSlowDeathBehavior.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectCreationList.h"


///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
JetSlowDeathBehaviorModuleData::JetSlowDeathBehaviorModuleData()
{

	m_fxOnGroundDeath = nullptr;
	m_oclOnGroundDeath = nullptr;

	m_fxInitialDeath = nullptr;
	m_oclInitialDeath = nullptr;

	m_delaySecondaryFromInitialDeath = 0;
	m_fxSecondary = nullptr;
	m_oclSecondary = nullptr;

	m_fxHitGround = nullptr;
	m_oclHitGround = nullptr;

	m_delayFinalBlowUpFromHitGround = 0;
	m_fxFinalBlowUp = nullptr;
	m_oclFinalBlowUp = nullptr;

	m_rollRate = 0.0f;
	m_rollRateDelta = 1.0f;
	m_pitchRate = 0.0f;
	m_fallHowFast = 0.0f;

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
/*static*/ void JetSlowDeathBehaviorModuleData::buildFieldParse( MultiIniFieldParse &p )
{
  SlowDeathBehaviorModuleData::buildFieldParse( p );

	static const FieldParse dataFieldParse[] =
	{

		{ "FXOnGroundDeath",	INI::parseFXList,	nullptr, offsetof( JetSlowDeathBehaviorModuleData, m_fxOnGroundDeath ) },
		{ "OCLOnGroundDeath", INI::parseObjectCreationList, nullptr, offsetof( JetSlowDeathBehaviorModuleData, m_oclOnGroundDeath ) },

		{ "FXInitialDeath",	INI::parseFXList, nullptr, offsetof( JetSlowDeathBehaviorModuleData, m_fxInitialDeath ) },
		{ "OCLInitialDeath", INI::parseObjectCreationList, nullptr, offsetof( JetSlowDeathBehaviorModuleData, m_oclInitialDeath ) },

		{ "DelaySecondaryFromInitialDeath",	INI::parseDurationUnsignedInt, nullptr, offsetof( JetSlowDeathBehaviorModuleData, m_delaySecondaryFromInitialDeath ) },
		{ "FXSecondary",	INI::parseFXList, nullptr, offsetof( JetSlowDeathBehaviorModuleData, m_fxSecondary ) },
		{ "OCLSecondary", INI::parseObjectCreationList, nullptr, offsetof( JetSlowDeathBehaviorModuleData, m_oclSecondary ) },

		{ "FXHitGround", INI::parseFXList, nullptr, offsetof( JetSlowDeathBehaviorModuleData, m_fxHitGround ) },
		{ "OCLHitGround", INI::parseObjectCreationList, nullptr, offsetof( JetSlowDeathBehaviorModuleData, m_oclHitGround ) },

		{ "DelayFinalBlowUpFromHitGround", INI::parseDurationUnsignedInt, nullptr, offsetof( JetSlowDeathBehaviorModuleData, m_delayFinalBlowUpFromHitGround ) },
		{ "FXFinalBlowUp", INI::parseFXList, nullptr, offsetof( JetSlowDeathBehaviorModuleData, m_fxFinalBlowUp ) },
		{ "OCLFinalBlowUp", INI::parseObjectCreationList, nullptr, offsetof( JetSlowDeathBehaviorModuleData, m_oclFinalBlowUp ) },

		{ "DeathLoopSound", INI::parseAudioEventRTS, nullptr, offsetof( JetSlowDeathBehaviorModuleData, m_deathLoopSound ) },

// @todo srj -- RollRate and RollRateDelta and PitchRate should use parseAngularVelocityReal
		{ "RollRate",	INI::parseReal, nullptr, offsetof( JetSlowDeathBehaviorModuleData, m_rollRate ) },
		{ "RollRateDelta", INI::parsePercentToReal, nullptr, offsetof( JetSlowDeathBehaviorModuleData, m_rollRateDelta ) },
		{ "PitchRate", INI::parseReal, nullptr, offsetof( JetSlowDeathBehaviorModuleData, m_pitchRate ) },
		{ "FallHowFast", INI::parsePercentToReal, nullptr, offsetof( JetSlowDeathBehaviorModuleData, m_fallHowFast ) },

		{ nullptr, nullptr, nullptr, 0 }

	};

  p.add( dataFieldParse );

}

///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
JetSlowDeathBehavior::JetSlowDeathBehavior( Thing *thing, const ModuleData *moduleData )
										: SlowDeathBehavior( thing, moduleData )
{

	m_timerDeathFrame = 0;
	m_timerOnGroundFrame = 0;
	m_rollRate = 0.0f;

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
JetSlowDeathBehavior::~JetSlowDeathBehavior()
{

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void JetSlowDeathBehavior::onDie( const DamageInfo *damageInfo )
{
	Object *us = getObject();

	// if the jet is on the ground we do just our ground fx death
	if( us->isSignificantlyAboveTerrain() == FALSE || us->getStatusBits().test( OBJECT_STATUS_DECK_HEIGHT_OFFSET ) )
	{
		const JetSlowDeathBehaviorModuleData *modData = getJetSlowDeathBehaviorModuleData();

		// execute fx
		FXList::doFXObj( modData->m_fxOnGroundDeath, us );

		// execute ocl
		ObjectCreationList::create( modData->m_oclOnGroundDeath, us, nullptr );

		// destroy object
		TheGameLogic->destroyObject( us );

	}
	else
	{

		// extend base class for slow death and begin the slow death behavior
		SlowDeathBehavior::onDie( damageInfo );

	}

	getObject()->clearStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_DECK_HEIGHT_OFFSET ) );

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void JetSlowDeathBehavior::beginSlowDeath( const DamageInfo *damageInfo )
{

	// extend functionality
	SlowDeathBehavior::beginSlowDeath( damageInfo );

	// get our info
	Object *us = getObject();
	const JetSlowDeathBehaviorModuleData *modData = getJetSlowDeathBehaviorModuleData();

	// record the frame we died on
	m_timerDeathFrame = TheGameLogic->getFrame();

	// do some effects
	FXList::doFXObj( modData->m_fxInitialDeath, us );
	ObjectCreationList::create( modData->m_oclInitialDeath, us, nullptr );

	// start audio loop playing
	m_deathLoopSound = modData->m_deathLoopSound;
	if( m_deathLoopSound.getEventName().isEmpty() == FALSE )
	{

		m_deathLoopSound.setObjectID( us->getID() );
		m_deathLoopSound.setPlayingHandle( TheAudio->addAudioEvent( &m_deathLoopSound ) );

	}

	// initialize our roll rate to that defined as the initial value in the module data
	m_rollRate = modData->m_rollRate;

	// set the locomotor so that the plane starts falling
	Locomotor *locomotor = us->getAIUpdateInterface()->getCurLocomotor();
	locomotor->setMaxLift( -TheGlobalData->m_gravity * (1.0f - modData->m_fallHowFast) );

	// do not allow the jet to turn anymore
	locomotor->setMaxTurnRate( 0.0f );

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
UpdateSleepTime JetSlowDeathBehavior::update()
{

	// extend functionality of base class
	SlowDeathBehavior::update();

	// if the death is not activated, do nothing else
	if( isSlowDeathActivated() == FALSE )
		return UPDATE_SLEEP_NONE;

	// get object info
	Object *us = getObject();
	const JetSlowDeathBehaviorModuleData *modData = getJetSlowDeathBehaviorModuleData();

	// roll us around in the air
	PhysicsBehavior *physics = us->getPhysics();
	DEBUG_ASSERTCRASH( physics, ("JetSlowDeathBehavior::beginSlowDeath - '%s' has no physics",
															us->getTemplate()->getName().str()) );
	if( physics )
		physics->setRollRate( m_rollRate );

	// adjust the roll rate over time
	m_rollRate *= modData->m_rollRateDelta;

	// do effects for death while in the air
	if( m_timerOnGroundFrame == 0 )
	{

		PathfindLayerEnum layer = TheTerrainLogic->getLayerForDestination(us->getPosition());
		us->setLayer(layer);
		Real height;
		if (layer == LAYER_GROUND)
		{
			// (this is more efficient than getGroundHeight because the info is cached)
			height = us->getHeightAboveTerrain();
		}
		else
		{
			Real layerHeight = TheTerrainLogic->getLayerHeight( us->getPosition()->x, us->getPosition()->y, layer );
			height = us->getPosition()->z - layerHeight;
			// slop a little bit for bridges, since we tend to end up fractionally
			// above 'em, and it's easier to just slop it here
			if (height >= 0.0f && height <= 1.0f)
				height = 0.0f;
		}



		Bool hitATree = FALSE;
		// Here we want to make sure we crash if we collide with a tree on the way down
		PhysicsBehavior *phys = us->getPhysics();
		if ( m_timerOnGroundFrame == 0 && phys )
		{
			ObjectID treeID = phys->getLastCollidee();
			Object *tree = TheGameLogic->findObjectByID( treeID );
			if ( tree )
			{
				if (tree->isKindOf( KINDOF_SHRUBBERY ) )
				hitATree = TRUE;
			}
		}



		// when we've hit the ground, we're totally done
		if( height <= 0.0f || hitATree )
		{

			// stop the death looping sound at the right time
			TheAudio->removeAudioEvent( m_deathLoopSound.getPlayingHandle() );

			// do some effects
			FXList::doFXObj( modData->m_fxHitGround, us );
			ObjectCreationList::create( modData->m_oclHitGround, us, nullptr );

			// we are now on the ground
			m_timerOnGroundFrame = TheGameLogic->getFrame();

			// start us rolling on another axis too
			if( physics )
				physics->setPitchRate( modData->m_pitchRate );

		}

		// timers for the secondary effect
		if( m_timerDeathFrame != 0 &&
				TheGameLogic->getFrame() - m_timerDeathFrame >= modData->m_delaySecondaryFromInitialDeath )
		{

			// do some effects
			FXList::doFXObj( modData->m_fxSecondary, us );
			ObjectCreationList::create( modData->m_oclSecondary, us, nullptr );

			// clear the death frame timer since we've already executed the event now
			m_timerDeathFrame = 0;

		}

	}
	else
	{
		// we are on the ground, pay attention to the final explosion timers
		if( TheGameLogic->getFrame() - m_timerOnGroundFrame >= modData->m_delayFinalBlowUpFromHitGround )
		{

			// do some effects
			FXList::doFXObj( modData->m_fxFinalBlowUp, us );
			ObjectCreationList::create( modData->m_oclFinalBlowUp, us, nullptr );

			// we're all done now
			TheGameLogic->destroyObject( us );

		}

	}

	return UPDATE_SLEEP_NONE;

}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void JetSlowDeathBehavior::crc( Xfer *xfer )
{

	// extend base class
	SlowDeathBehavior::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void JetSlowDeathBehavior::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	SlowDeathBehavior::xfer( xfer );

	// timer death frame
	xfer->xferUnsignedInt( &m_timerDeathFrame );

	// on ground frame
	xfer->xferUnsignedInt( &m_timerOnGroundFrame );

	// roll rate
	xfer->xferReal( &m_rollRate );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void JetSlowDeathBehavior::loadPostProcess()
{

	// extend base class
	SlowDeathBehavior::loadPostProcess();

}
```
