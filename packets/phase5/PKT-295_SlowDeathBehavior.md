# Translation Packet `PKT-295`: `SlowDeathBehavior`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-295`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/behavior/slow_death_behavior.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (22):**
  - `Common/GameLOD.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/GameLOD.h`
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
  - `GameLogic/Module/DieModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DieModule.h`
  - `GameLogic/Module/PhysicsUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PhysicsUpdate.h`
  - `GameLogic/Module/SlavedUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SlavedUpdate.h`
  - `GameLogic/Module/SlowDeathBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SlowDeathBehavior.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/ObjectCreationList.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ObjectCreationList.h`
  - `GameLogic/Weapon.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h`
  - `PreRTS.h`
- **Implemented Module Facets (2):**
  - `DieModuleInterface* getDie()`
  - `SlowDeathBehaviorInterface* getSlowDeathBehaviorInterface()`
- **INI `FieldParse` Table Entries (15):**
  - `"SinkRate" => INI::parseVelocityReal`
  - `"ProbabilityModifier" => INI::parseInt`
  - `"ModifierBonusPerOverkillPercent" => INI::parsePercentToReal`
  - `"SinkDelay" => INI::parseDurationUnsignedInt`
  - `"SinkDelayVariance" => INI::parseDurationUnsignedInt`
  - `"DestructionDelay" => INI::parseDurationUnsignedInt`
  - `"DestructionDelayVariance" => INI::parseDurationUnsignedInt`
  - `"DestructionAltitude" => INI::parseReal`
  - `"FX" => parseFX`
  - `"OCL" => parseOCL`
  - `"Weapon" => parseWeapon`
  - `"FlingForce" => INI::parseReal`
  - `"FlingForceVariance" => INI::parseReal`
  - `"FlingPitch" => INI::parseAngleReal`
  - `"FlingPitchVariance" => INI::parseAngleReal`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (6 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUnsignedInt( &m_sinkFrame )`
  3. `xferUnsignedInt( &m_midpointFrame )`
  4. `xferUnsignedInt( &m_destructionFrame )`
  5. `xferReal( &m_acceleratedTimeScale )`
  6. `xferUnsignedInt( &m_flags )`
- **PRNG Calls (10 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValueReal(-PI, PI)`
  - `GameLogicRandomValueReal(minPitch, maxPitch)`
  - `GameLogicRandomValueReal(minMag, maxMag)`
  - `GameLogicRandomValue(0, d->m_sinkDelayVariance))`
  - `GameLogicRandomValue(0, d->m_destructionDelayVariance))`
  - `GameLogicRandomValue( BEGIN_MIDPOINT_RATIO * m_destructionFrame, END_MIDPOINT_RATIO * m_destructionFrame )`
  - `GameLogicRandomValue(0, listSize-1)`
  - `GameLogicRandomValue(0, listSize-1)`
  - `GameLogicRandomValue(0, listSize-1)`
  - `GameLogicRandomValue(1, total)`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SlowDeathBehavior.h`
```cpp
// FILE: SlowDeathBehavior.h /////////////////////////////////////////////////////////////////////////
// Author: Colin Day, December 2001
// Desc:   Update that will count down a lifetime and destroy object when it reaches zero
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/DieModule.h"
#include "GameLogic/Module/UpdateModule.h"

class FXList;
class ObjectCreationList;
class WeaponTemplate;
class DamageInfo;

//-------------------------------------------------------------------------------------------------
typedef std::vector<const FXList*> FXListVec;
typedef std::vector<const ObjectCreationList*> OCLVec;
typedef std::vector<const WeaponTemplate*> WeaponTemplateVec;

//-------------------------------------------------------------------------------------------------
enum SlowDeathPhaseType CPP_11(: Int)
{
	SDPHASE_INITIAL = 0,
	SDPHASE_MIDPOINT,
	SDPHASE_FINAL,

	SD_PHASE_COUNT
};

#ifdef DEFINE_SLOWDEATHPHASE_NAMES
static const char *const TheSlowDeathPhaseNames[] =
{
	"INITIAL",
	"MIDPOINT",
	"FINAL",

	nullptr
};
static_assert(ARRAY_SIZE(TheSlowDeathPhaseNames) == SD_PHASE_COUNT + 1, "Incorrect array size");
#endif


//-------------------------------------------------------------------------------------------------
class SlowDeathBehaviorModuleData : public UpdateModuleData
{
public:
	DieMuxData				m_dieMuxData;
	Real							m_sinkRate;
	Int								m_probabilityModifier;
	Real							m_modifierBonusPerOverkillPercent;
	UnsignedInt				m_sinkDelay;
	UnsignedInt				m_sinkDelayVariance;
	Real							m_destructionAltitude;
	UnsignedInt				m_destructionDelay;
	UnsignedInt				m_destructionDelayVariance;
	FXListVec					m_fx[SD_PHASE_COUNT];
	OCLVec						m_ocls[SD_PHASE_COUNT];
	WeaponTemplateVec	m_weapons[SD_PHASE_COUNT];
	Real							m_flingForce;
	Real							m_flingForceVariance;
	Real							m_flingPitch;
	Real							m_flingPitchVariance;

	enum
	{
		//flags used by m_maskOfLoadedEffects
		HAS_FX							= 1,
		HAS_OCL							= 2,
		HAS_WEAPON					= 4,
		HAS_NON_LOD_EFFECTS = (HAS_OCL | HAS_WEAPON)
	};
	Byte							m_maskOfLoadedEffects;	///<used to keep track of whether m_fx,m_ocls,m_weapons actually contain anything.

	SlowDeathBehaviorModuleData();
	static void buildFieldParse(MultiIniFieldParse& p);
	Bool hasNonLodEffects() const
	{
		return (m_maskOfLoadedEffects & SlowDeathBehaviorModuleData::HAS_NON_LOD_EFFECTS) != 0;
	}

private:

};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class SlowDeathBehaviorInterface
{
public:
	virtual void beginSlowDeath( const DamageInfo *damageInfo ) = 0;
	virtual Int getProbabilityModifier( const DamageInfo *damageInfo ) const = 0;
	virtual Bool isDieApplicable(const DamageInfo *damageInfo) const = 0;
};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class SlowDeathBehavior : public UpdateModule,
													public DieModuleInterface,
													public SlowDeathBehaviorInterface
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( SlowDeathBehavior, "SlowDeathBehavior" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( SlowDeathBehavior, SlowDeathBehaviorModuleData )

public:

	SlowDeathBehavior( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	static Int getInterfaceMask() { return UpdateModule::getInterfaceMask() | (MODULEINTERFACE_DIE); }

	// BehaviorModule
	virtual DieModuleInterface* getDie() override { return this; }

	// UpdateModuleInterface
	virtual UpdateSleepTime update() override;
	virtual SlowDeathBehaviorInterface* getSlowDeathBehaviorInterface() override { return this; }
	// Disabled conditions to process -- all
	virtual DisabledMaskType getDisabledTypesToProcess() const override { return DISABLEDMASK_ALL; }

	// DieModuleInterface
	virtual void onDie( const DamageInfo *damageInfo ) override;

	// SlowDeathBehaviorInterface
	virtual void beginSlowDeath( const DamageInfo *damageInfo ) override;
	virtual Int getProbabilityModifier( const DamageInfo *damageInfo ) const override;
	virtual Bool isDieApplicable(const DamageInfo *damageInfo) const override { return getSlowDeathBehaviorModuleData()->m_dieMuxData.isDieApplicable(getObject(), damageInfo); }

protected:

	void doPhaseStuff(SlowDeathPhaseType sdphase);
	Bool isSlowDeathActivated() const { return (m_flags & (1<<SLOW_DEATH_ACTIVATED)) != 0; }
	UnsignedInt getDestructionFrame() const { return m_destructionFrame; }

private:

	enum
	{
		SLOW_DEATH_ACTIVATED,
		MIDPOINT_EXECUTED,
		FLUNG_INTO_AIR,
		BOUNCED
	};

	UnsignedInt m_sinkFrame;							///< Frame to be sunken into the ground on
	UnsignedInt m_midpointFrame;					///< The midpoint is between .25 through life and .75 through life (eg)
	UnsignedInt m_destructionFrame;
	Real				m_acceleratedTimeScale;		///<used to speedup deaths when needed to improve game performance.
	UnsignedInt	m_flags;
};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Behavior/SlowDeathBehavior.cpp`
```cpp
// FILE: SlowDeathBehavior.cpp ///////////////////////////////////////////////////////////////////////
// Author:
// Desc:
///////////////////////////////////////////////////////////////////////////////////////////////////


// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine
#define DEFINE_SLOWDEATHPHASE_NAMES
#include "Common/GameLOD.h"
#include "Common/INI.h"
#include "Common/RandomValue.h"
#include "Common/Thing.h"
#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"
#include "GameClient/Drawable.h"
#include "GameClient/FXList.h"
#include "GameClient/InGameUI.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Module/SlowDeathBehavior.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/SlavedUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectCreationList.h"
#include "GameLogic/Weapon.h"


const Real BEGIN_MIDPOINT_RATIO = 0.35f;
const Real END_MIDPOINT_RATIO = 0.65f;

//-------------------------------------------------------------------------------------------------
SlowDeathBehaviorModuleData::SlowDeathBehaviorModuleData()
{
	m_sinkRate = 0;
	m_probabilityModifier = 10;
	m_modifierBonusPerOverkillPercent = 0;
	m_sinkDelay = 0;
	m_sinkDelayVariance = 0;
	m_destructionDelay = 0;
	m_destructionDelayVariance = 0;
	m_destructionAltitude = -10;
	m_maskOfLoadedEffects = 0; //assume no ocl, fx, or weapons.
	m_flingForce = 0;
	m_flingForceVariance = 0;
	m_flingPitch = 0;
	m_flingPitchVariance = 0;
	// redundant.
	//m_fx.clear();
	//m_ocls.clear();
	//m_weapons.clear();
}

//-------------------------------------------------------------------------------------------------
static void parseFX( INI* ini, void *instance, void * /*store*/, const void* /*userData*/ )
{
	SlowDeathBehaviorModuleData* self = (SlowDeathBehaviorModuleData*)instance;
	SlowDeathPhaseType sdphase = (SlowDeathPhaseType)INI::scanIndexList(ini->getNextToken(), TheSlowDeathPhaseNames);
	for (const char* token = ini->getNextToken(); token; token = ini->getNextTokenOrNull())
	{
		const FXList *fxl = TheFXListStore->findFXList((token));	// could be null! this is OK!
		self->m_fx[sdphase].push_back(fxl);
		if (fxl)
			self->m_maskOfLoadedEffects |= SlowDeathBehaviorModuleData::HAS_FX;
	}
}

//-------------------------------------------------------------------------------------------------
static void parseOCL( INI* ini, void *instance, void * /*store*/, const void* /*userData*/ )
{
	SlowDeathBehaviorModuleData* self = (SlowDeathBehaviorModuleData*)instance;
	SlowDeathPhaseType sdphase = (SlowDeathPhaseType)INI::scanIndexList(ini->getNextToken(), TheSlowDeathPhaseNames);
	for (const char* token = ini->getNextToken(); token; token = ini->getNextTokenOrNull())
	{
		const ObjectCreationList *ocl = TheObjectCreationListStore->findObjectCreationList(token);	// could be null! this is OK!
		self->m_ocls[sdphase].push_back(ocl);
		if (ocl)
			self->m_maskOfLoadedEffects |= SlowDeathBehaviorModuleData::HAS_OCL;
	}
}

//-------------------------------------------------------------------------------------------------
static void parseWeapon( INI* ini, void *instance, void * /*store*/, const void* /*userData*/ )
{
	SlowDeathBehaviorModuleData* self = (SlowDeathBehaviorModuleData*)instance;
	SlowDeathPhaseType sdphase = (SlowDeathPhaseType)INI::scanIndexList(ini->getNextToken(), TheSlowDeathPhaseNames);
	for (const char* token = ini->getNextToken(); token; token = ini->getNextTokenOrNull())
	{
		const WeaponTemplate *wt = TheWeaponStore->findWeaponTemplate(token);	// could be null! this is OK!
		self->m_weapons[sdphase].push_back(wt);
		if (wt)
			self->m_maskOfLoadedEffects |= SlowDeathBehaviorModuleData::HAS_WEAPON;
	}
}

//-------------------------------------------------------------------------------------------------
/*static*/ void SlowDeathBehaviorModuleData::buildFieldParse(MultiIniFieldParse& p)
{
  UpdateModuleData::buildFieldParse(p);

	static const FieldParse dataFieldParse[] =
	{
		{ "SinkRate",													INI::parseVelocityReal,						nullptr, offsetof( SlowDeathBehaviorModuleData, m_sinkRate ) },
		{ "ProbabilityModifier",							INI::parseInt,										nullptr, offsetof( SlowDeathBehaviorModuleData, m_probabilityModifier ) },
		{ "ModifierBonusPerOverkillPercent",	INI::parsePercentToReal,					nullptr, offsetof( SlowDeathBehaviorModuleData, m_modifierBonusPerOverkillPercent ) },
		{ "SinkDelay",												INI::parseDurationUnsignedInt,		nullptr, offsetof( SlowDeathBehaviorModuleData, m_sinkDelay ) },
		{ "SinkDelayVariance",								INI::parseDurationUnsignedInt,		nullptr, offsetof( SlowDeathBehaviorModuleData, m_sinkDelayVariance ) },
		{ "DestructionDelay",									INI::parseDurationUnsignedInt,		nullptr, offsetof( SlowDeathBehaviorModuleData, m_destructionDelay ) },
		{ "DestructionDelayVariance",					INI::parseDurationUnsignedInt,		nullptr, offsetof( SlowDeathBehaviorModuleData, m_destructionDelayVariance ) },
		{ "DestructionAltitude",							INI::parseReal,										nullptr, offsetof( SlowDeathBehaviorModuleData, m_destructionAltitude ) },
		{ "FX",																parseFX,													nullptr, 0 },
		{ "OCL",															parseOCL,													nullptr, 0 },
		{ "Weapon",														parseWeapon,											nullptr, 0 },
		{ "FlingForce",												INI::parseReal,										nullptr, offsetof( SlowDeathBehaviorModuleData, m_flingForce) },
		{ "FlingForceVariance",								INI::parseReal,										nullptr, offsetof( SlowDeathBehaviorModuleData, m_flingForceVariance) },
		{ "FlingPitch",												INI::parseAngleReal,							nullptr, offsetof( SlowDeathBehaviorModuleData, m_flingPitch) },
		{ "FlingPitchVariance",								INI::parseAngleReal,							nullptr, offsetof( SlowDeathBehaviorModuleData, m_flingPitchVariance) },
		{ nullptr, nullptr, nullptr, 0 }
	};
  p.add(dataFieldParse);
	p.add(DieMuxData::getFieldParse(), offsetof( SlowDeathBehaviorModuleData, m_dieMuxData ));
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
SlowDeathBehavior::SlowDeathBehavior( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{
	m_flags = 0;
	m_sinkFrame = 0;
	m_midpointFrame = 0;
	m_destructionFrame = 0;
	m_acceleratedTimeScale = 1.0f;

	if (getSlowDeathBehaviorModuleData()->m_probabilityModifier < 1)
	{
		DEBUG_CRASH(("ProbabilityModifier must be >= 1."));
		throw INI_INVALID_DATA;
	}

	setWakeFrame(getObject(), UPDATE_SLEEP_FOREVER);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
SlowDeathBehavior::~SlowDeathBehavior()
{
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Int SlowDeathBehavior::getProbabilityModifier( const DamageInfo *damageInfo ) const
{
	// Calculating how far past dead we were allows us to pick more spectacular deaths when
	// severely killed, and more sedate ones when only slightly killed.
	// eg ( 200 hp max, had 10 left, took 50 damage, 40 overkill, (40/200) * 100 = 20 overkill %)
	Int overkillDamage = damageInfo->out.m_actualDamageDealt - damageInfo->out.m_actualDamageClipped;
	Real overkillPercent = WWMath::Div_Safe((float)overkillDamage, (float)getObject()->getBodyModule()->getMaxHealth());
	Int overkillModifier = overkillPercent * getSlowDeathBehaviorModuleData()->m_modifierBonusPerOverkillPercent;

	return max( getSlowDeathBehaviorModuleData()->m_probabilityModifier + overkillModifier, 1 );
}

//-------------------------------------------------------------------------------------------------
static void calcRandomForce(Real minMag, Real maxMag, Real minPitch, Real maxPitch, Coord3D& force)
{
	Real angle = GameLogicRandomValueReal(-PI, PI);
	Real pitch = GameLogicRandomValueReal(minPitch, maxPitch);
	Real mag = GameLogicRandomValueReal(minMag, maxMag);

	Matrix3D mtx(1);
	mtx.Scale(mag);
	mtx.Rotate_Z(angle);
	mtx.Rotate_Y(-pitch);

	Vector3 v = mtx.Get_X_Vector();

	force.x = v.X;
	force.y = v.Y;
	force.z = v.Z;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void SlowDeathBehavior::beginSlowDeath(const DamageInfo *damageInfo)
{
	if (!isSlowDeathActivated())
	{
		const SlowDeathBehaviorModuleData* d = getSlowDeathBehaviorModuleData();
		Object* obj = getObject();

		if (d->m_sinkRate && obj->isKindOf(KINDOF_INFANTRY))
		{

			Drawable *draw = getObject()->getDrawable();
			if ( draw )
			{
				// this object sinks slowly after it dies so don't draw a
				// floating shadow decal on the ground above it.
				obj->getDrawable()->setShadowsEnabled(false);
				draw->setTerrainDecalFadeTarget( 0.0f, -0.2f );
			}



		}


		// Ask game detail manager if we need to speedup all deaths to improve performance
		Real timeScale = TheGameLODManager->getSlowDeathScale();
		m_acceleratedTimeScale = 1.0f;	// assume normal death speed.

		if (timeScale == 0.0f && !d->hasNonLodEffects())
		{
			// Deaths happen instantly so just delete the object and return
			TheGameLogic->destroyObject(obj);
			return;
		}
		else
		{
			// timescale is some non-zero value so we may need to speed up death
			if( getObject()->isKindOf( KINDOF_HULK ) && TheGameLogic->getHulkMaxLifetimeOverride() != -1 )
			{
				//Scripts don't want hulks around, so start sinking immediately!
				m_sinkFrame = 1;
				m_midpointFrame = (LOGICFRAMES_PER_SECOND/2) + 1;
				m_destructionFrame = LOGICFRAMES_PER_SECOND + 1;
				m_acceleratedTimeScale = 1.0f;
			}
			else
			{
				m_sinkFrame = timeScale * (d->m_sinkDelay + GameLogicRandomValue(0, d->m_sinkDelayVariance));
				m_destructionFrame = timeScale * (d->m_destructionDelay + GameLogicRandomValue(0, d->m_destructionDelayVariance));
				m_midpointFrame = GameLogicRandomValue( BEGIN_MIDPOINT_RATIO * m_destructionFrame, END_MIDPOINT_RATIO * m_destructionFrame );
				m_acceleratedTimeScale = timeScale;
			}
		}

		UnsignedInt now = TheGameLogic->getFrame();

		if (d->m_flingForce > 0)
		{


			//Just in case this is a stingersoldier or other HELD object, lets set them free so they will fly
			// with their own physics during slow death
			if( obj->isDisabledByType( DISABLED_HELD ) )
			{
				static NameKeyType key_SlavedUpdate = NAMEKEY( "SlavedUpdate" );
				SlavedUpdate* slave = (SlavedUpdate*)obj->findUpdateModule( key_SlavedUpdate );
				if( slave )
				{
					slave->onSlaverDie( nullptr );
				}
			}

			PhysicsBehavior* physics = obj->getPhysics();
			if (physics)
			{
				// make sure we are at least a bit above the ground
				const Real MIN_ALTITUDE = 1.0f;
				Real altitude = obj->getHeightAboveTerrain();
				if (altitude < MIN_ALTITUDE)
				{
					Coord3D pos = *obj->getPosition();
					pos.z += MIN_ALTITUDE;
					obj->setPosition(&pos);
				}

				Coord3D force;
				calcRandomForce(d->m_flingForce, d->m_flingForce + d->m_flingForceVariance,
												d->m_flingPitch, d->m_flingPitch + d->m_flingPitchVariance, force);
				physics->setAllowToFall(true);
				physics->applyForce(&force);
				physics->setExtraBounciness(-1.0);					// we don't want this guy to bounce at all
				physics->setExtraFriction(-3 * SECONDS_PER_LOGICFRAME_REAL);							// reduce his ground friction a bit
				physics->setAllowBouncing(true);
				Real orientation = WWMath::Atan2(force.y, force.x);
				physics->setAngles(orientation, 0, 0);
				obj->getDrawable()->setModelConditionState(MODELCONDITION_EXPLODED_FLAILING);
				m_flags |= (1<<FLUNG_INTO_AIR);
			}
			setWakeFrame(obj, UPDATE_SLEEP_NONE);
		}
		else
		{
			// we don't need to wake up immediately, but only when the first of these
			// counters wants to trigger....
			Int whenToWakeTime = m_sinkFrame;
			if (whenToWakeTime > m_destructionFrame)
				whenToWakeTime = m_destructionFrame;
			if (whenToWakeTime > m_midpointFrame)
				whenToWakeTime = m_midpointFrame;
			setWakeFrame(obj, UPDATE_SLEEP(whenToWakeTime));
		}
		m_sinkFrame += now;
		m_destructionFrame += now;
		m_midpointFrame += now;

		m_flags |= (1<<SLOW_DEATH_ACTIVATED);

		doPhaseStuff(SDPHASE_INITIAL);

	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void SlowDeathBehavior::doPhaseStuff(SlowDeathPhaseType sdphase)
{
	const SlowDeathBehaviorModuleData* d = getSlowDeathBehaviorModuleData();
	Int idx, listSize;

	if (!d->m_maskOfLoadedEffects)
		return;	//has no ocl, fx, or weapons.

	listSize = d->m_fx[sdphase].size();
	if (listSize > 0)
	{
		idx = GameLogicRandomValue(0, listSize-1);
		const FXListVec& v = d->m_fx[sdphase];
		DEBUG_ASSERTCRASH(idx>=0&&idx<v.size(),("bad idx"));
		const FXList* fxl = v[idx];
		FXList::doFXObj(fxl, getObject(), nullptr);
	}

	listSize = d->m_ocls[sdphase].size();
	if (listSize > 0)
	{
		idx = GameLogicRandomValue(0, listSize-1);
		const OCLVec& v = d->m_ocls[sdphase];
		DEBUG_ASSERTCRASH(idx>=0&&idx<v.size(),("bad idx"));
		const ObjectCreationList* ocl = v[idx];
		ObjectCreationList::create(ocl, getObject(), nullptr);
	}

	listSize = d->m_weapons[sdphase].size();
	if (listSize > 0)
	{
		idx = GameLogicRandomValue(0, listSize-1);
		const WeaponTemplateVec& v = d->m_weapons[sdphase];
		DEBUG_ASSERTCRASH(idx>=0&&idx<v.size(),("bad idx"));
		const WeaponTemplate* wt = v[idx];
		if (wt)
		{
			TheWeaponStore->createAndFireTempWeapon(wt, getObject(), getObject()->getPosition());
		}
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
UpdateSleepTime SlowDeathBehavior::update()
{
	//DEBUG_LOG(("updating SlowDeathBehavior %08lx",this));
	DEBUG_ASSERTCRASH(isSlowDeathActivated(), ("hmm, this should not be possible"));

	const SlowDeathBehaviorModuleData* d = getSlowDeathBehaviorModuleData();
	Object* obj = getObject();

	Real timeScale = TheGameLODManager->getSlowDeathScale();

	// Check if we have normal time scale but LODManager is requesting acceleration
	if (timeScale != 1.0f && m_acceleratedTimeScale == 1.0f && !d->hasNonLodEffects())
	{
		// speed of deaths has been increased since beginning of death
		// so adjust it to current levels.
		if (timeScale == 0)
		{
			// instant death
			TheGameLogic->destroyObject(obj);
			return UPDATE_SLEEP_NONE;
		}

		m_sinkFrame = (Real)m_sinkFrame * timeScale;
		m_midpointFrame = (Real)m_midpointFrame * timeScale;
		m_destructionFrame = (Real)m_destructionFrame * timeScale;
		m_acceleratedTimeScale = timeScale;
	};

	UnsignedInt now = TheGameLogic->getFrame();


	if ((m_flags & (1<<FLUNG_INTO_AIR)) != 0)
	{
		if ((m_flags & (1<<BOUNCED)) == 0)
		{
			++m_sinkFrame;
			++m_midpointFrame;
			++m_destructionFrame;
			if (!obj->isAboveTerrain())
			{
				obj->clearAndSetModelConditionFlags(MAKE_MODELCONDITION_MASK(MODELCONDITION_EXPLODED_FLAILING),
																						MAKE_MODELCONDITION_MASK(MODELCONDITION_EXPLODED_BOUNCING));
				m_flags |= (1<<BOUNCED);
			}

			// Here we want to make sure we die if we collide with a tree on the way down
			PhysicsBehavior *phys = obj->getPhysics();
			if ( phys )
			{
				ObjectID treeID = phys->getLastCollidee();
				Object *tree = TheGameLogic->findObjectByID( treeID );
				if ( tree )
				{
					if (tree->isKindOf( KINDOF_SHRUBBERY ) )
					{
						obj->setDisabled( DISABLED_HELD );
						obj->clearModelConditionFlags( MAKE_MODELCONDITION_MASK(MODELCONDITION_EXPLODED_FLAILING) );
						obj->clearModelConditionFlags( MAKE_MODELCONDITION_MASK(MODELCONDITION_EXPLODED_BOUNCING) );
						obj->setModelConditionFlags(   MAKE_MODELCONDITION_MASK(MODELCONDITION_PARACHUTING) ); //looks like he is snagged in a tree
						obj->setPositionZ( obj->getPosition()->z - (d->m_sinkRate * 50.0f) );// make him sink faster
						if ( !obj->isAboveTerrain() )
							TheGameLogic->destroyObject(obj);

					}
				}
			}



		}
	}

	if ( (now >= m_sinkFrame && d->m_sinkRate > 0.0f) )
	{
		// disable Physics (if any) so that we can control the sink...
		obj->setDisabled( DISABLED_HELD );
		Coord3D pos = *obj->getPosition();
		pos.z -= d->m_sinkRate / m_acceleratedTimeScale;
		obj->setPosition( &pos );
	}

	if( now >= m_midpointFrame && (m_flags & (1<<MIDPOINT_EXECUTED)) == 0 )
	{
		doPhaseStuff(SDPHASE_MIDPOINT);
		m_flags |= (1<<MIDPOINT_EXECUTED);
	}

	if (now >= m_destructionFrame)
	{
		doPhaseStuff(SDPHASE_FINAL);
		TheGameLogic->destroyObject(obj);
	}

	return UPDATE_SLEEP_NONE;
}

//-------------------------------------------------------------------------------------------------
void SlowDeathBehavior::onDie( const DamageInfo *damageInfo )
{
	Object *obj = getObject();

	if (!isDieApplicable(damageInfo))
		return;

	AIUpdateInterface *ai = obj->getAIUpdateInterface();
	if (ai)
	{
		// has another AI already handled us. (hopefully another SlowDeathBehavior)
		if (ai->isAiInDeadState())
			return;
		ai->markAsDead();
	}

	// deselect this unit for all players.
	TheGameLogic->deselectObject(obj, PLAYERMASK_ALL, TRUE);

	Int total = 0;
	BehaviorModule** update = obj->getBehaviorModules();
	for (; *update; ++update)
	{
		SlowDeathBehaviorInterface* sdu = (*update)->getSlowDeathBehaviorInterface();
		if (sdu != nullptr && sdu->isDieApplicable(damageInfo))
		{
			total += sdu->getProbabilityModifier( damageInfo );
		}
	}
	DEBUG_ASSERTCRASH(total > 0, ("Hmm, this is wrong"));


	// this returns a value from 1...total, inclusive
	Int roll = GameLogicRandomValue(1, total);

	for (/* UpdateModuleInterface** */ update = obj->getBehaviorModules(); *update; ++update)
	{
		SlowDeathBehaviorInterface* sdu = (*update)->getSlowDeathBehaviorInterface();
		if (sdu != nullptr && sdu->isDieApplicable(damageInfo))
		{
			roll -= sdu->getProbabilityModifier( damageInfo );
			if (roll <= 0)
			{
				sdu->beginSlowDeath(damageInfo);
				return;
			}
		}
	}

	DEBUG_CRASH(("We should never get here"));
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void SlowDeathBehavior::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void SlowDeathBehavior::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// sink frame
	xfer->xferUnsignedInt( &m_sinkFrame );

	// midpoint frame
	xfer->xferUnsignedInt( &m_midpointFrame );

	// destruction frame
	xfer->xferUnsignedInt( &m_destructionFrame );

	// accelerated time scale
	xfer->xferReal( &m_acceleratedTimeScale );

	// flags
	xfer->xferUnsignedInt( &m_flags );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void SlowDeathBehavior::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
