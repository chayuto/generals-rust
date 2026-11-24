# Translation Packet `PKT-258`: `BattleBusSlowDeathBehavior`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-258`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/behavior/battle_bus_slow_death_behavior.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (13):**
  - `Common/ModelState.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ModelState.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/FXList.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/FXList.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/BattleBusSlowDeathBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BattleBusSlowDeathBehavior.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/ContainModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ContainModule.h`
  - `GameLogic/Module/PhysicsUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PhysicsUpdate.h`
  - `GameLogic/Module/SlowDeathBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SlowDeathBehavior.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/ObjectCreationList.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ObjectCreationList.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (7):**
  - `"FXStartUndeath" => INI::parseFXList`
  - `"OCLStartUndeath" => INI::parseObjectCreationList`
  - `"FXHitGround" => INI::parseFXList`
  - `"OCLHitGround" => INI::parseObjectCreationList`
  - `"ThrowForce" => INI::parseReal`
  - `"PercentDamageToPassengers" => INI::parsePercentToReal`
  - `"EmptyHulkDestructionDelay" => INI::parseDurationUnsignedInt`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (5 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferBool( &m_isRealDeath )`
  3. `xferBool( &m_isInFirstDeath )`
  4. `xferUnsignedInt( &m_groundCheckFrame )`
  5. `xferUnsignedInt( &m_penaltyDeathFrame )`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BattleBusSlowDeathBehavior.h`
```cpp
// FILE: BattleBusSlowDeathBehavior.h ///////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, June 2003
// Desc:   Death for the Battle Bus.  Can do a fake slow death before the real one if triggered intentionally
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/SlowDeathBehavior.h"

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
class FXList;

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class BattleBusSlowDeathBehaviorModuleData : public SlowDeathBehaviorModuleData
{

public:

	BattleBusSlowDeathBehaviorModuleData();

	static void buildFieldParse( MultiIniFieldParse &p );

	const FXList *m_fxStartUndeath;								///< fx list executed on start of throw
	const ObjectCreationList *m_oclStartUndeath;	///< ocl list executed on start of throw

	const FXList *m_fxHitGround;									///< FXList for when we land
	const ObjectCreationList *m_oclHitGround;			///< OCL for when we land

	Real m_throwForce;														///< How hard we are thrown in to the air
	Real m_percentDamageToPassengers;							///< At the moment we throw up, how hard the people inside are hit
	UnsignedInt m_emptyHulkDestructionDelay;			///< Another reason this is a BattleBus module, and not a generic two stage death.  If non-zero, time empty before we kill ourselves.

};

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
class BattleBusSlowDeathBehavior : public SlowDeathBehavior
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( BattleBusSlowDeathBehavior, "BattleBusSlowDeathBehavior" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( BattleBusSlowDeathBehavior, BattleBusSlowDeathBehaviorModuleData )

public:

	BattleBusSlowDeathBehavior( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	// slow death methods
	virtual void onDie( const DamageInfo *damageInfo ) override;
	virtual void beginSlowDeath( const DamageInfo *damageInfo ) override;
	virtual UpdateSleepTime update() override;

protected:

	Bool m_isRealDeath;								///< Flag set by onDie that prevents the alternate processing caused by beginSlowDeath.
	Bool m_isInFirstDeath;						///< Flag that controls lifetime of alternate processing.  Prevented by RealDeath flag.
	UnsignedInt m_groundCheckFrame;		///< Don't check for bouncing until at least here.
	UnsignedInt m_penaltyDeathFrame;	///< If non zero, kill us with Penalty damage at this frame

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Behavior/BattleBusSlowDeathBehavior.cpp`
```cpp
// FILE: BattleBusSlowDeathBehavior.cpp /////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, June 2003
// Desc:   Death for the Battle Bus.  Can do a fake slow death before the real one if triggered intentionally
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine
#include "Common/Xfer.h"

#include "GameLogic/Module/BattleBusSlowDeathBehavior.h"

#include "Common/ModelState.h"

#include "GameClient/FXList.h"

#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectCreationList.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/PhysicsUpdate.h"


enum
{
	GROUND_CHECK_DELAY = 10,		///< Check for colliding with the ground only after this long, to prevent hitting on the way up
	EMPTY_HULK_CHECK_DELAY = 15 ///< Check for empty hulk every this often instead of every frame
};

///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
BattleBusSlowDeathBehaviorModuleData::BattleBusSlowDeathBehaviorModuleData()
{

	m_fxStartUndeath = nullptr;
	m_oclStartUndeath = nullptr;

	m_fxHitGround = nullptr;
	m_oclHitGround = nullptr;

	m_throwForce = 1.0f;
	m_percentDamageToPassengers = 0.0f;
	m_emptyHulkDestructionDelay = 0;

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
/*static*/ void BattleBusSlowDeathBehaviorModuleData::buildFieldParse( MultiIniFieldParse &p )
{
  SlowDeathBehaviorModuleData::buildFieldParse( p );

	static const FieldParse dataFieldParse[] =
	{

		{ "FXStartUndeath",	INI::parseFXList,	nullptr, offsetof( BattleBusSlowDeathBehaviorModuleData, m_fxStartUndeath ) },
		{ "OCLStartUndeath", INI::parseObjectCreationList, nullptr, offsetof( BattleBusSlowDeathBehaviorModuleData, m_oclStartUndeath ) },

		{ "FXHitGround",	INI::parseFXList,	nullptr, offsetof( BattleBusSlowDeathBehaviorModuleData, m_fxHitGround ) },
		{ "OCLHitGround", INI::parseObjectCreationList, nullptr, offsetof( BattleBusSlowDeathBehaviorModuleData, m_oclHitGround ) },

		{ "ThrowForce", INI::parseReal, nullptr, offsetof( BattleBusSlowDeathBehaviorModuleData, m_throwForce ) },
		{ "PercentDamageToPassengers", INI::parsePercentToReal, nullptr, offsetof( BattleBusSlowDeathBehaviorModuleData, m_percentDamageToPassengers ) },
		{ "EmptyHulkDestructionDelay", INI::parseDurationUnsignedInt, nullptr, offsetof( BattleBusSlowDeathBehaviorModuleData, m_emptyHulkDestructionDelay ) },

		{ nullptr, nullptr, nullptr, 0 }

	};

  p.add( dataFieldParse );

}

///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
BattleBusSlowDeathBehavior::BattleBusSlowDeathBehavior( Thing *thing, const ModuleData *moduleData )
										: SlowDeathBehavior( thing, moduleData )
{

	m_isRealDeath = FALSE;
	m_isInFirstDeath = FALSE;
	m_groundCheckFrame = 0;
	m_penaltyDeathFrame = 0;
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
BattleBusSlowDeathBehavior::~BattleBusSlowDeathBehavior()
{

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void BattleBusSlowDeathBehavior::onDie( const DamageInfo *damageInfo )
{
	m_isRealDeath = TRUE;// Set beforehand because onDie could result in picking us to beginSlowDeath
	m_isInFirstDeath = FALSE; // and clear this incase we died while in the alternate death

	SlowDeathBehavior::onDie(damageInfo);
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void BattleBusSlowDeathBehavior::beginSlowDeath( const DamageInfo *damageInfo )
{
	Object *me = getObject();
	const BattleBusSlowDeathBehaviorModuleData * data = getBattleBusSlowDeathBehaviorModuleData();
	if( !m_isRealDeath )
	{
		// We can intercept and do our extra slow death if this is not from a real onDie
		m_isInFirstDeath = TRUE;
		m_groundCheckFrame = TheGameLogic->getFrame() + GROUND_CHECK_DELAY;

		// First do the special effects
		FXList::doFXObj(data->m_fxStartUndeath, me );
		ObjectCreationList::create(data->m_oclStartUndeath, me, nullptr );

		if( me->getAI() )
		{
			// Then stop what we were doing
			me->getAI()->aiIdle(CMD_FROM_AI);
		}

		if( me->getPhysics() )
		{
			// Then stop physically
			me->getPhysics()->clearAcceleration();
			me->getPhysics()->scrubVelocity2D(0);

			// Then get chucked in the air
			Coord3D throwForce;
			throwForce.x = 0;
			throwForce.y = 0;
			throwForce.z = data->m_throwForce;
			me->getPhysics()->applyShock(&throwForce);
			me->getPhysics()->applyRandomRotation();
		}

		// And finally hit those inside for some damage
		if( me->getContain() )
			me->getContain()->processDamageToContained(data->m_percentDamageToPassengers);

		setWakeFrame(getObject(), UPDATE_SLEEP_NONE);
	}
	else
	{
		// If a real death, we aren't allowed to do anything
		SlowDeathBehavior::beginSlowDeath( damageInfo );
	}

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
UpdateSleepTime BattleBusSlowDeathBehavior::update()
{
	Object *me = getObject();
	const BattleBusSlowDeathBehaviorModuleData * data = getBattleBusSlowDeathBehaviorModuleData();
	if( m_isInFirstDeath )
	{
		// First death means we are doing our throw up in the air conversion

		if( (m_groundCheckFrame< TheGameLogic->getFrame() )  &&  !me->isAboveTerrain() )
		{
			// We're done since we hit the ground
			m_isInFirstDeath = FALSE;

			// Do the special FX
			FXList::doFXObj(data->m_fxHitGround, me );
			ObjectCreationList::create(data->m_oclHitGround, me, nullptr );
			me->setModelConditionState(MODELCONDITION_SECOND_LIFE);

			// And stop us forever
			if( me->getAI() )
			{
				// Stop doing anything again (could try to move while in air)
				me->getAI()->aiIdle(CMD_FROM_AI);
			}
			if( me->getPhysics() )
			{
				// And stop physically again
				me->getPhysics()->clearAcceleration();
				me->getPhysics()->scrubVelocity2D(0);
			}
			me->setDisabled(DISABLED_HELD);

			// We can only sleep if we don't have to watch out for being empty.
			if( data->m_emptyHulkDestructionDelay == 0 )
				return UPDATE_SLEEP_FOREVER;
			else
				return UPDATE_SLEEP_NONE; // None, so we check for empty as soon as possible
		}
		return UPDATE_SLEEP_NONE;// None, since we are waiting to be able to check for ground collision
	}
	else if( m_isRealDeath )
	{
		// If a real death, we aren't allowed to do anything

		return SlowDeathBehavior::update();
	}
	else
	{
		// If neither death is active, we must be awake to check for emptiness

		const ContainModuleInterface *contain = me->getContain();
		// Safety, no need to be awake if no special case to wait for
		if( contain == nullptr )
			return UPDATE_SLEEP_FOREVER;

		if( m_penaltyDeathFrame != 0 )
		{
			// We have been timed for death, see if we have re-filled first
			if( contain->getContainCount() > 0 )
			{
				m_penaltyDeathFrame = 0;
				return UPDATE_SLEEP(EMPTY_HULK_CHECK_DELAY);
			}
			else if( TheGameLogic->getFrame() > m_penaltyDeathFrame )
			{
				// No salvation, see if the timer is up to be killed.  Penalty prevents effects, Extra 4 prevents this die module from recalling.
				// Can't use Suicided death because a Terrorist actually inflicts Suicide deaths on others.
				me->kill(DAMAGE_PENALTY, DEATH_EXTRA_4);
				return UPDATE_SLEEP_FOREVER;// Forever, since we are dead and some other death module is in charge
			}
			else
			{
				return UPDATE_SLEEP(EMPTY_HULK_CHECK_DELAY);
			}
		}
		else
		{
			// We are not marked for death, so see if we should be
			if( contain->getContainCount() == 0 )
			{
				m_penaltyDeathFrame = TheGameLogic->getFrame() + data->m_emptyHulkDestructionDelay;
			}
			return UPDATE_SLEEP(EMPTY_HULK_CHECK_DELAY);// Stay awake regardless
		}
	}
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void BattleBusSlowDeathBehavior::crc( Xfer *xfer )
{

	// extend base class
	SlowDeathBehavior::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void BattleBusSlowDeathBehavior::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	SlowDeathBehavior::xfer( xfer );

	xfer->xferBool( &m_isRealDeath );

	xfer->xferBool( &m_isInFirstDeath );

	xfer->xferUnsignedInt( &m_groundCheckFrame );

	xfer->xferUnsignedInt( &m_penaltyDeathFrame );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void BattleBusSlowDeathBehavior::loadPostProcess()
{

	// extend base class
	SlowDeathBehavior::loadPostProcess();

}
```
