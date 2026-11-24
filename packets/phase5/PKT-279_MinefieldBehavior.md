# Translation Packet `PKT-279`: `MinefieldBehavior`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-279`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/behavior/minefield_behavior.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (20):**
  - `Common/GameState.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/GameState.h`
  - `Common/RandomValue.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/RandomValue.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/AutoHealBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AutoHealBehavior.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/CollideModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CollideModule.h`
  - `GameLogic/Module/DamageModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DamageModule.h`
  - `GameLogic/Module/DieModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DieModule.h`
  - `GameLogic/Module/MinefieldBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/MinefieldBehavior.h`
  - `GameLogic/Module/PhysicsUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PhysicsUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/ObjectCreationList.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ObjectCreationList.h`
  - `GameLogic/Weapon.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h`
  - `PreRTS.h`
- **Implemented Module Facets (4):**
  - `CollideModuleInterface* getCollide()`
  - `DamageModuleInterface* getDamage()`
  - `DieModuleInterface* getDie()`
  - `LandMineInterface* getLandMineInterface()`
- **INI `FieldParse` Table Entries (11):**
  - `"DetonationWeapon" => INI::parseWeaponTemplate`
  - `"DetonatedBy" => INI::parseBitString32`
  - `"StopsRegenAfterCreatorDies" => INI::parseBool`
  - `"Regenerates" => INI::parseBool`
  - `"WorkersDetonate" => INI::parseBool`
  - `"CreatorDeathCheckRate" => INI::parseDurationUnsignedInt`
  - `"ScootFromStartingPointTime" => INI::parseDurationUnsignedInt`
  - `"NumVirtualMines" => INI::parseUnsignedInt`
  - `"RepeatDetonateMoveThresh" => INI::parseReal`
  - `"DegenPercentPerSecondAfterCreatorDies" => INI::parsePercentToReal`
  - `"CreationList" => INI::parseObjectCreationList`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (12 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUnsignedInt( &m_virtualMinesRemaining )`
  3. `xferUnsignedInt( &m_nextDeathCheckFrame )`
  4. `xferUnsignedInt( &m_scootFramesLeft )`
  5. `xferCoord3D( &m_scootVel )`
  6. `xferCoord3D( &m_scootAccel )`
  7. `xferBool( &m_ignoreDamage )`
  8. `xferBool( &m_regenerates )`
  9. `xferBool( &m_draining )`
  10. `xferUnsignedByte( &maxImmunity )`
  11. `xferObjectID( &m_immunes[ i ].id )`
  12. `xferUnsignedInt( &m_immunes[ i ].collideTime )`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/MinefieldBehavior.h`
```cpp
// FILE: MinefieldBehavior.h ////////////////////////////////////////////////////////////////////
// Author: Steven Johnson, June 2002
// Desc:   Minefield behavior
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/CollideModule.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Module/DamageModule.h"
#include "GameLogic/Module/DieModule.h"
#include "GameLogic/ObjectCreationList.h"

//-------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
class MinefieldBehaviorModuleData : public UpdateModuleData
{

public:

	MinefieldBehaviorModuleData();

	static void buildFieldParse( MultiIniFieldParse &p );

	const WeaponTemplate*	m_detonationWeapon;								///< what happens when we detonate
	Int										m_detonatedBy;										///< can we be triggered by allies, etc?
	Bool									m_stopsRegenAfterCreatorDies;
	Bool									m_regenerates;										///< if t, can't be killed normally
	Bool									m_workersDetonate;								///< if f, workers don't detonate mines
	UnsignedInt						m_creatorDeathCheckRate;					///< if above is true, how often to check
	UnsignedInt						m_scootFromStartingPointTime;			///< if nonzero, gradually scoot to dest pt
	UnsignedInt						m_numVirtualMines;								///< num of "virtual" mines we have
	Real									m_repeatDetonateMoveThresh;
	Real									m_healthPercentToDrainPerSecond;
	const ObjectCreationList* m_ocl; ///< object creaton list to make

};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class MinefieldBehavior : public UpdateModule,
													public CollideModuleInterface,
													public DamageModuleInterface,
													public DieModuleInterface,
													public LandMineInterface
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( MinefieldBehavior, "MinefieldBehavior" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( MinefieldBehavior, MinefieldBehaviorModuleData )

public:

	MinefieldBehavior( Thing *thing, const ModuleData *moduleData );
	// virtual destructor prototype provided by memory pool declaration

	static Int getInterfaceMask() { return UpdateModule::getInterfaceMask() | (MODULEINTERFACE_COLLIDE) | (MODULEINTERFACE_DAMAGE) | (MODULEINTERFACE_DIE); }

	// BehaviorModule
	virtual CollideModuleInterface* getCollide() override { return this; }
	virtual LandMineInterface* getLandMineInterface() override { return this; }
	virtual DamageModuleInterface* getDamage() override { return this; }
	virtual DieModuleInterface* getDie() override { return this; }

	// DamageModuleInterface
	virtual void onDamage( DamageInfo *damageInfo ) override;
	virtual void onHealing( DamageInfo *damageInfo ) override;
	virtual void onBodyDamageStateChange(const DamageInfo* damageInfo, BodyDamageType oldState, BodyDamageType newState) override { }

	// DieModuleInterface
	virtual void onDie( const DamageInfo *damageInfo ) override;

	// UpdateModuleInterface
	virtual UpdateSleepTime update() override;

	// CollideModuleInterface
	virtual void onCollide( Object *other, const Coord3D *loc, const Coord3D *normal ) override;
	virtual Bool wouldLikeToCollideWith(const Object* other) const override { return false; }
	virtual Bool isHijackedVehicleCrateCollide() const override { return false; }
	virtual Bool isCarBombCrateCollide() const override { return false; }
	virtual Bool isRailroad() const override { return false;}
	virtual Bool isSalvageCrateCollide() const override { return false; }
	virtual Bool isSabotageBuildingCrateCollide() const override { return FALSE; }

	// Minefield specific methods
	virtual void setScootParms(const Coord3D& start, const Coord3D& end) override;
	virtual void disarm() override;

private:

	// mines are small, so we can get by with a small fixed number here
	enum { MAX_IMMUNITY = 3 };
	struct ImmuneInfo
	{
		ObjectID id;
		UnsignedInt collideTime;
	};

	struct DetonatorInfo
	{
		ObjectID id;
		Coord3D where;
	};

	UnsignedInt		m_nextDeathCheckFrame;
	UnsignedInt		m_scootFramesLeft;
	Coord3D				m_scootVel;
	Coord3D				m_scootAccel;
	UnsignedInt		m_virtualMinesRemaining;
	ImmuneInfo		m_immunes[MAX_IMMUNITY];
	std::vector<DetonatorInfo>	m_detonators;
	Bool					m_ignoreDamage;
	Bool					m_regenerates;
	Bool					m_draining;

	void detonateOnce(const Coord3D& position);
	UpdateSleepTime calcSleepTime();

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Behavior/MinefieldBehavior.cpp`
```cpp
// FILE: MinefieldBehavior.cpp //////////////////////////////////////////////////////////////////
// Author: Steven Johnson, June 2002
// Desc:   Minefield behavior
///////////////////////////////////////////////////////////////////////////////////////////////////


// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine
#define DEFINE_RELATIONSHIP_NAMES
#include "Common/GameState.h"
#include "Common/RandomValue.h"
#include "Common/Xfer.h"
#include "GameClient/Drawable.h"
#include "GameClient/InGameUI.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Module/MinefieldBehavior.h"
#include "GameLogic/Module/AutoHealBehavior.h"
#include "GameLogic/Weapon.h"


// detonation never puts our health below this, since we probably auto-regen
const Real MIN_HEALTH = 0.1f;

//-------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
MinefieldBehaviorModuleData::MinefieldBehaviorModuleData()
{
	m_detonationWeapon = nullptr;
	m_detonatedBy = (1 << ENEMIES) | (1 << NEUTRAL);
	m_stopsRegenAfterCreatorDies = true;
	m_regenerates = false;
	m_workersDetonate = false;
	m_creatorDeathCheckRate = static_cast<float>(LOGICFRAMES_PER_SECOND);
	m_scootFromStartingPointTime = 0;
	m_repeatDetonateMoveThresh = 1.0f;
	m_numVirtualMines = 1;
	m_healthPercentToDrainPerSecond = 0.0f;
	m_ocl = nullptr;
}

//-------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
/*static*/ void MinefieldBehaviorModuleData::buildFieldParse( MultiIniFieldParse &p )
{

  UpdateModuleData::buildFieldParse( p );

	static const FieldParse dataFieldParse[] =
	{
		{ "DetonationWeapon", INI::parseWeaponTemplate,	nullptr, offsetof( MinefieldBehaviorModuleData, m_detonationWeapon ) },
		{ "DetonatedBy", INI::parseBitString32, TheRelationshipNames, offsetof( MinefieldBehaviorModuleData, m_detonatedBy ) },
		{ "StopsRegenAfterCreatorDies", INI::parseBool, nullptr, offsetof( MinefieldBehaviorModuleData, m_stopsRegenAfterCreatorDies ) },
		{ "Regenerates", INI::parseBool, nullptr, offsetof( MinefieldBehaviorModuleData, m_regenerates ) },
		{ "WorkersDetonate", INI::parseBool, nullptr, offsetof( MinefieldBehaviorModuleData, m_workersDetonate ) },
		{ "CreatorDeathCheckRate", INI::parseDurationUnsignedInt, nullptr, offsetof( MinefieldBehaviorModuleData, m_creatorDeathCheckRate ) },
		{ "ScootFromStartingPointTime", INI::parseDurationUnsignedInt, nullptr, offsetof( MinefieldBehaviorModuleData, m_scootFromStartingPointTime ) },
		{ "NumVirtualMines", INI::parseUnsignedInt, nullptr, offsetof( MinefieldBehaviorModuleData, m_numVirtualMines ) },
		{ "RepeatDetonateMoveThresh", INI::parseReal, nullptr, offsetof( MinefieldBehaviorModuleData, m_repeatDetonateMoveThresh ) },
		{ "DegenPercentPerSecondAfterCreatorDies", INI::parsePercentToReal,	nullptr, offsetof( MinefieldBehaviorModuleData, m_healthPercentToDrainPerSecond ) },
		{ "CreationList",	INI::parseObjectCreationList,	nullptr,	offsetof( MinefieldBehaviorModuleData, m_ocl ) },
		{ nullptr, nullptr, nullptr, 0 }
	};

  p.add( dataFieldParse );

}

///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
MinefieldBehavior::MinefieldBehavior( Thing *thing, const ModuleData* moduleData )
								 : UpdateModule( thing, moduleData )
{
	const MinefieldBehaviorModuleData* d = getMinefieldBehaviorModuleData();
	m_nextDeathCheckFrame = 0;
	m_scootFramesLeft = 0;
	m_scootVel.zero();
	m_scootAccel.zero();
	m_detonators.clear();
	m_ignoreDamage = false;
	m_regenerates = d->m_regenerates;
	m_draining = false;
	m_virtualMinesRemaining = d->m_numVirtualMines;
	for (Int i = 0; i < MAX_IMMUNITY; ++i)
	{
		m_immunes[i].id = INVALID_ID;
		m_immunes[i].collideTime = 0;
	}

	// start off awake, and we will calcSleepTime from here on
	setWakeFrame( getObject(), UPDATE_SLEEP_NONE );

	// mines aren't auto-acquirable
	getObject()->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_NO_ATTACK_FROM_AI ) );
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
MinefieldBehavior::~MinefieldBehavior()
{
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
UpdateSleepTime MinefieldBehavior::calcSleepTime()
{
	const MinefieldBehaviorModuleData* d = getMinefieldBehaviorModuleData();

	// if we're draining we have to update every frame
	if (m_draining)
		return UPDATE_SLEEP_NONE;

	// if we're scooting we need to update every frame
	if( m_scootFramesLeft > 0 )
		return UPDATE_SLEEP_NONE;

	// if there is anybody in our immulity monitoring we need to update every frame
	for( Int i = 0; i < MAX_IMMUNITY; ++i )
		if( m_immunes[ i ].id != INVALID_ID )
			return UPDATE_SLEEP_NONE;

	UnsignedInt sleepTime = FOREVER;
	UnsignedInt now = TheGameLogic->getFrame();
	//
	// sleep until the next death check frame we already have figured outif we care
	// about it (that is, when our creator dies)
	//
	if (m_regenerates && d->m_stopsRegenAfterCreatorDies)
		sleepTime = min( sleepTime, m_nextDeathCheckFrame - now );

	// if we don't want to sleep forever, prevent 0 frame sleeps
	if( sleepTime == 0 )
		sleepTime = 1;

	// sleep forever
	return UPDATE_SLEEP( sleepTime );

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
UpdateSleepTime MinefieldBehavior::update()
{
	Object* obj = getObject();
	const MinefieldBehaviorModuleData* d = getMinefieldBehaviorModuleData();
	UnsignedInt now = TheGameLogic->getFrame();

	if (m_scootFramesLeft > 0)
	{
		Coord3D pt = *obj->getPosition();

		m_scootVel.x += m_scootAccel.x;
		m_scootVel.y += m_scootAccel.y;
		m_scootVel.z += m_scootAccel.z;

		pt.x += m_scootVel.x;
		pt.y += m_scootVel.y;
		pt.z += m_scootVel.z;

		// srj sez: scooting mines always go on the highest layer.
		Coord3D tmp = pt;
		tmp.z = 99999.0f;
		PathfindLayerEnum newLayer = TheTerrainLogic->getHighestLayerForDestination(&tmp);
		obj->setLayer(newLayer);

		Real ground = TheTerrainLogic->getLayerHeight( pt.x, pt.y, newLayer );

		if (newLayer != LAYER_GROUND)
		{
			// ensure we are slightly above the bridge, to account for fudge & sloppy art
			const Real FUDGE = 1.0f;
			ground += FUDGE;
		}

		if (pt.z < ground || m_scootFramesLeft <= 1)
			pt.z = ground;

		obj->setPosition(&pt);

		--m_scootFramesLeft;
	}

	// check for expired immunities.
	for (Int i = 0; i < MAX_IMMUNITY; ++i)
	{
		if (m_immunes[i].id == INVALID_ID)
			continue;

		if (TheGameLogic->findObjectByID(m_immunes[i].id) == nullptr ||
				now > m_immunes[i].collideTime + 2)
		{
			//DEBUG_LOG(("expiring an immunity %d",m_immunes[i].id));
			m_immunes[i].id = INVALID_ID;	// he's dead, jim.
			m_immunes[i].collideTime = 0;
		}
	}

	if (now >= m_nextDeathCheckFrame)
	{

		// check to see if there is an enemy building on me... since enemy buildings can be build on top of me

		// check to see if the building that made me is gone, and whether therefore, I should go
		if (m_regenerates && d->m_stopsRegenAfterCreatorDies)
		{
			m_nextDeathCheckFrame = now + d->m_creatorDeathCheckRate;
			ObjectID producerID = getObject()->getProducerID();
			if (producerID != INVALID_ID)
			{
				Object* producer = TheGameLogic->findObjectByID(producerID);
				if (producer == nullptr || producer->isEffectivelyDead())
				{
					m_regenerates = false;
					m_draining = true;
					static const NameKeyType key_AutoHealBehavior = NAMEKEY("AutoHealBehavior");
					AutoHealBehavior* ahb = (AutoHealBehavior*)obj->findUpdateModule( key_AutoHealBehavior );
					if (ahb)
						ahb->stopHealing();
				}
			}
		}
	}

	if (m_draining)
	{
		DamageInfo damageInfo;
		damageInfo.in.m_amount = (obj->getBodyModule()->getMaxHealth() * d->m_healthPercentToDrainPerSecond) / static_cast<float>(LOGICFRAMES_PER_SECOND);
		damageInfo.in.m_sourceID = obj->getID();
		damageInfo.in.m_damageType = DAMAGE_UNRESISTABLE;
		damageInfo.in.m_deathType = DEATH_NORMAL;
		obj->attemptDamage( &damageInfo );
	}

	return calcSleepTime();
}

// ------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void MinefieldBehavior::detonateOnce(const Coord3D& position)
{
	const MinefieldBehaviorModuleData* d = getMinefieldBehaviorModuleData();
	if (d->m_detonationWeapon)
	{
		Object* obj = getObject();
	  TheWeaponStore->createAndFireTempWeapon(d->m_detonationWeapon, obj, &position);
	}

	if (m_virtualMinesRemaining > 0)
		--m_virtualMinesRemaining;

	if (!m_regenerates && m_virtualMinesRemaining == 0)
	{
		TheGameLogic->destroyObject(getObject());
	}
	else
	{
		Real percent = (Real)m_virtualMinesRemaining / (Real)d->m_numVirtualMines;
		BodyModuleInterface* body = getObject()->getBodyModule();
		Real health = body->getHealth();
		Real desired = percent * body->getMaxHealth();
		if (desired < MIN_HEALTH)
			desired = MIN_HEALTH;
		Real amount = health - desired;

		if (amount > 0.0f)
		{
			m_ignoreDamage = true;

			//body->internalChangeHealth(desired - health);
			//can't use this, AutoHeal won't work unless we go thru normal damage stuff

			DamageInfo extraDamageInfo;
			extraDamageInfo.in.m_damageType = DAMAGE_UNRESISTABLE;
			extraDamageInfo.in.m_deathType = DEATH_NONE;
			extraDamageInfo.in.m_sourceID = getObject()->getID();
			extraDamageInfo.in.m_amount = amount;
			getObject()->attemptDamage(&extraDamageInfo);

			m_ignoreDamage = false;
		}
	}

	if (m_virtualMinesRemaining == 0)
	{
		getObject()->setModelConditionState(MODELCONDITION_RUBBLE);
		getObject()->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_MASKED ) );
	}
	else
	{
		getObject()->clearModelConditionState(MODELCONDITION_RUBBLE);
		getObject()->clearStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_MASKED ) );
	}


	if (d->m_ocl)
	{
		ObjectCreationList::create(d->m_ocl, getObject(), getObject());
	}
}

//-----------------------------------------------------------------------------
static Real calcDistSquared(const Coord3D& a, const Coord3D& b)
{
	return sqr(a.x - b.x) + sqr(a.y - b.y) + sqr(a.z - b.z);
}

// ------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void MinefieldBehavior::onCollide( Object *other, const Coord3D *loc, const Coord3D *normal )
{
	if (other == nullptr || other->isEffectivelyDead())
		return;

	if (m_virtualMinesRemaining == 0)
		return;

	Object* obj = getObject();
	const MinefieldBehaviorModuleData* d = getMinefieldBehaviorModuleData();
	UnsignedInt now = TheGameLogic->getFrame();

	// is this guy in our immune list?
	// NOTE NOTE NOTE, must always do this check FIRST so that 'collideTime' is updated...
	for (Int i = 0; i < MAX_IMMUNITY; ++i)
	{
		if (m_immunes[i].id == other->getID())
		{
			//DEBUG_LOG(("ignoring due to immunity %d",m_immunes[i].id));
			m_immunes[i].collideTime = now;
			return;
		}
	}

	if (!d->m_workersDetonate)
	{
		// infantry+dozer=worker.
		if (other->isKindOf(KINDOF_INFANTRY) && other->isKindOf(KINDOF_DOZER))
			return;
	}

	Int requiredMask = 0;
	Relationship r = obj->getRelationship(other);
	if (r == ALLIES) requiredMask = (1 << ALLIES);
	else if (r == ENEMIES) requiredMask = (1 << ENEMIES);
	else if (r == NEUTRAL) requiredMask = (1 << NEUTRAL);
	if ((d->m_detonatedBy & requiredMask) == 0)
		return;

	// are we active?
	if (m_scootFramesLeft > 0)
		return;

	// things that are in the process of clearing mines are immune to mine detonation,
	// even if we aren't the specific mine they are trying to clear. (however, they must
	// have a real mine they area trying to clear... it's possible they could be trying to
	// clear a position where there is no mine, in which case we grant them no immunity, muwahahaha)
	AIUpdateInterface* otherAI = other->getAI();
	if (otherAI && otherAI->isClearingMines() && otherAI->getGoalObject() != nullptr)
	{
		// mine-clearers are granted immunity to us for as long as they continuously
		// collide, even if no longer clearing mines. (this prevents the problem
		// of a guy who touches two close-together mines while clearing, then puts up his
		// detector and is blown to smithereens by the other one.)
		for (Int i = 0; i < MAX_IMMUNITY; ++i)
		{
			if (m_immunes[i].id == INVALID_ID || m_immunes[i].id == other->getID())
			{
				//DEBUG_LOG(("add/update immunity %d",m_immunes[i].id));
				m_immunes[i].id = other->getID();
				m_immunes[i].collideTime = now;

				// wake up
				setWakeFrame( obj, calcSleepTime() );

				break;
			}
		}
		return;
	}

	// if we detonated another one nearby, we have to move a little bit to detonate another one.
	Bool found = false;
	for (std::vector<DetonatorInfo>::iterator it = m_detonators.begin(); it != m_detonators.end(); ++it)
	{
		if (other->getID() == it->id)
		{
			found = TRUE;
			Real distSqr = calcDistSquared(*other->getPosition(), it->where);
			if (distSqr <= sqr(d->m_repeatDetonateMoveThresh))
			{
				// too close. punt for now.
				return;
			}
			else
			{
				// far enough. update the loc, then break out and blow up.
				it->where = *other->getPosition();
				break;
			}
		}
	}
	if (!found)
	{
		// add him to the list.
		DetonatorInfo detInfo;
		detInfo.id = other->getID();
		detInfo.where = *other->getPosition();
		m_detonators.push_back(detInfo);
	}

	Coord3D detPt = *other->getPosition();
	obj->getGeometryInfo().clipPointToFootprint(*obj->getPosition(), detPt);
	detonateOnce(detPt);
}

// ------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void MinefieldBehavior::onDamage( DamageInfo *damageInfo )
{
	if (m_ignoreDamage)
		return;

	const MinefieldBehaviorModuleData* d = getMinefieldBehaviorModuleData();

	// detonate as many times as necessary for our virtual mine count to match our health
	BodyModuleInterface* body = getObject()->getBodyModule();

	for (;;)
	{
		Real virtualMinesExpectedF = WWMath::Div_Safe((Real)d->m_numVirtualMines * body->getHealth(), body->getMaxHealth());
		Int virtualMinesExpected =
			damageInfo->in.m_damageType == DAMAGE_HEALING ?
			REAL_TO_INT_FLOOR(virtualMinesExpectedF) :
			REAL_TO_INT_CEIL(virtualMinesExpectedF);
		if (virtualMinesExpected > d->m_numVirtualMines)
			virtualMinesExpected = d->m_numVirtualMines;
		if (m_virtualMinesRemaining < virtualMinesExpected)
		{
			m_virtualMinesRemaining = virtualMinesExpected;
		}
		else if (m_virtualMinesRemaining > virtualMinesExpected)
		{
			if (m_draining &&
						damageInfo->in.m_sourceID == getObject()->getID() &&
						damageInfo->in.m_damageType == DAMAGE_UNRESISTABLE)
			{
				// don't detonate.... just ditch a mine
				--m_virtualMinesRemaining;
			}
			else
			{
				detonateOnce(*getObject()->getPosition());
			}
		}
		else
		{
			break;
		}
	}

	if (m_virtualMinesRemaining == 0)
	{
		// oops, if someone did weapon damage they may have nuked our health to zero,
		// which would be bad if we regen. prevent this. (srj)
		if (m_regenerates && body->getHealth() < MIN_HEALTH)
		{
			body->internalChangeHealth(MIN_HEALTH - body->getHealth());
		}

		getObject()->setModelConditionState(MODELCONDITION_RUBBLE);
		getObject()->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_MASKED ) );
	}
	else
	{
		getObject()->clearModelConditionState(MODELCONDITION_RUBBLE);
		getObject()->clearStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_MASKED ) );
	}
}

// ------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void MinefieldBehavior::onHealing( DamageInfo *damageInfo )
{
	onDamage(damageInfo);
}

// ------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void MinefieldBehavior::onDie( const DamageInfo *damageInfo )
{
	TheGameLogic->destroyObject(getObject());
}

// ------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void MinefieldBehavior::disarm()
{
	if (!m_regenerates)
	{
		TheGameLogic->destroyObject(getObject());
		return;
	}

	// detonation never puts our health below this, since we probably auto-regen
	const Real MIN_HEALTH = 0.1f;

	BodyModuleInterface* body = getObject()->getBodyModule();
	Real desired = MIN_HEALTH;
	Real amount = body->getHealth() - desired;

	m_ignoreDamage = true;

	//body->internalChangeHealth(desired - health);
	//can't use this, AutoHeal won't work unless we go thru normal damage stuff

	DamageInfo extraDamageInfo;
	extraDamageInfo.in.m_damageType = DAMAGE_UNRESISTABLE;
	extraDamageInfo.in.m_deathType = DEATH_NONE;
	extraDamageInfo.in.m_sourceID = getObject()->getID();
	extraDamageInfo.in.m_amount = amount;
	getObject()->attemptDamage(&extraDamageInfo);

	m_ignoreDamage = false;

	m_virtualMinesRemaining = 0;
	getObject()->setModelConditionState(MODELCONDITION_RUBBLE);
	getObject()->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_MASKED ) );
}

// ------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void MinefieldBehavior::setScootParms(const Coord3D& start, const Coord3D& end)
{
	Object* obj = getObject();
	const MinefieldBehaviorModuleData* d = getMinefieldBehaviorModuleData();
	UnsignedInt scootFromStartingPointTime = d->m_scootFromStartingPointTime;

	Coord3D endOnGround = end;
	endOnGround.z = TheTerrainLogic->getGroundHeight( endOnGround.x, endOnGround.y );
	if (start.z > endOnGround.z)
	{
		// figure out how long it will take to fall, and replace scoot time with that
		UnsignedInt fallingTime = REAL_TO_INT_CEIL(WWMath::Sqrtf(2.0f * (start.z - endOnGround.z) / WWMath::Fabs(TheGlobalData->m_gravity)));
		// we can scoot after we land, but don't want to stop scooting before we land
		if (scootFromStartingPointTime < fallingTime)
			scootFromStartingPointTime = fallingTime;
	}

	if (scootFromStartingPointTime == 0)
	{
		obj->setPosition(&endOnGround);
		m_scootFramesLeft = 0;
	}
	else
	{
		// x = x0 + vt + 0.5at^2
		// thus 2(dx - vt)/t^2 = a
		Real dx = endOnGround.x - start.x;
		Real dy = endOnGround.y - start.y;
		Real dz = endOnGround.z - start.z;
		Real dist = WWMath::Sqrt(sqr(dx) + sqr(dy));
		if (dist <= 0.1f && WWMath::Fabs(dz) <= 0.1f)
		{
			obj->setPosition(&endOnGround);
			m_scootFramesLeft = 0;
		}
		else
		{
			Real t = (Real)scootFromStartingPointTime;
			Real scootFromStartingPointSpeed = dist / t;
			Real accelMag = WWMath::Fabs(2.0f * (dist - scootFromStartingPointSpeed*t)/sqr(t));
			Real dxNorm = (dist <= 0.1f) ? 0.0f : (dx / dist);
			Real dyNorm = (dist <= 0.1f) ? 0.0f : (dy / dist);
			m_scootVel.x = dxNorm * scootFromStartingPointSpeed;
			m_scootVel.y = dyNorm * scootFromStartingPointSpeed;
			m_scootAccel.x = -dxNorm * accelMag;
			m_scootAccel.y = -dyNorm * accelMag;
			m_scootAccel.z = TheGlobalData->m_gravity;
			obj->setPosition(&start);
			m_scootFramesLeft = scootFromStartingPointTime;

			// we need to wake ourselves up because we could be lying here sleeping forever
			setWakeFrame( obj, calcSleepTime() );

		}
	}

}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void MinefieldBehavior::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void MinefieldBehavior::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// mines remaining
	/// @todo srj -- ensure health, appearance, etc are correct for save/reload! post-MP!
	xfer->xferUnsignedInt( &m_virtualMinesRemaining );

	// next death check frame
	xfer->xferUnsignedInt( &m_nextDeathCheckFrame );

	// scoot frames left
	xfer->xferUnsignedInt( &m_scootFramesLeft );

	// scoot velocity
	xfer->xferCoord3D( &m_scootVel );

	// scoot acceleration
	xfer->xferCoord3D( &m_scootAccel );

	xfer->xferBool( &m_ignoreDamage );
	xfer->xferBool( &m_regenerates );
	xfer->xferBool( &m_draining );

	// immunities
	UnsignedByte maxImmunity = MAX_IMMUNITY;
	xfer->xferUnsignedByte( &maxImmunity );
	if( maxImmunity != MAX_IMMUNITY )
	{

		DEBUG_CRASH(( "MinefieldBehavior::xfer - MAX_IMMUNITY has changed size, you must version this code and then you can remove this error message" ));
		throw SC_INVALID_DATA;

	}
	for( UnsignedByte i = 0; i < maxImmunity; ++i )
	{

		// object id
		xfer->xferObjectID( &m_immunes[ i ].id );

		// collide time
		xfer->xferUnsignedInt( &m_immunes[ i ].collideTime );

	}

	if( xfer->getXferMode() == XFER_LOAD )
		m_detonators.clear();

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void MinefieldBehavior::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
