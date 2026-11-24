# Translation Packet `PKT-291`: `PropagandaTowerBehavior`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-291`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/behavior/propaganda_tower_behavior.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (19):**
  - `Common/GameState.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/GameState.h`
  - `Common/GameUtility.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameUtility.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/PlayerList.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/PlayerList.h`
  - `Common/Upgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Upgrade.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/FXList.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/FXList.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/ContainModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ContainModule.h`
  - `GameLogic/Module/DieModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DieModule.h`
  - `GameLogic/Module/PropagandaTowerBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PropagandaTowerBehavior.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `GameLogic/Weapon.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h`
  - `PreRTS.h`
- **Implemented Module Facets (1):**
  - `DieModuleInterface* getDie()`
- **INI `FieldParse` Table Entries (8):**
  - `"Radius" => INI::parseReal`
  - `"DelayBetweenUpdates" => INI::parseDurationUnsignedInt`
  - `"HealPercentEachSecond" => INI::parsePercentToReal`
  - `"UpgradedHealPercentEachSecond" => INI::parsePercentToReal`
  - `"PulseFX" => INI::parseFXList`
  - `"UpgradeRequired" => INI::parseAsciiString`
  - `"UpgradedPulseFX" => INI::parseFXList`
  - `"AffectsSelf" => INI::parseBool`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (5 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUnsignedInt( &m_lastScanFrame )`
  3. `xferUnsignedShort( &insideCount )`
  4. `xferObjectID( &trackerEntry->objectID )`
  5. `xferObjectID( &trackerEntry->objectID )`
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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PropagandaTowerBehavior.h`
```cpp
// FILE: PropagandaTowerBehavior.h ////////////////////////////////////////////////////////////////
// Author: Colin Day, August 2002
// Desc:   Behavior module for PropagandaTower
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/PropagandaTowerBehavior.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Module/DieModule.h"

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
class ObjectTracker;
class FXList;
class UpgradeTemplate;

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
class PropagandaTowerBehaviorModuleData: public UpdateModuleData
{

public:

	PropagandaTowerBehaviorModuleData();

	static void buildFieldParse( MultiIniFieldParse &p );

	Real m_scanRadius;													///< radius of our scan
	UnsignedInt m_scanDelayInFrames;						///< how frequently we do an update scan
	Real m_autoHealPercentPerSecond;						///< how much % of max health we heal per second
	const FXList *m_pulseFX;										///< FXList to play when scan is updated
	AsciiString m_upgradeRequired;							///< Upgrade required to use the upgraded pulse FX
	Real m_upgradedAutoHealPercentPerSecond;		///< Different percent to use for healing if upgraded too
	const FXList *m_upgradedPulseFX;						///< FXList to play for pulse when upgraded
	Bool m_affectsSelf;													///< Allow effect to affect ourselves

};

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
class PropagandaTowerBehavior : public UpdateModule,
																public DieModuleInterface
{

	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( PropagandaTowerBehavior, PropagandaTowerBehaviorModuleData );
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( PropagandaTowerBehavior, "PropagandaTowerBehavior" )

public:

	PropagandaTowerBehavior( Thing *thing, const ModuleData *modData );
	// virtual destructor prototype provided by MemoryPoolObject

	// module methods
	static Int getInterfaceMask() { return UpdateModule::getInterfaceMask() | (MODULEINTERFACE_DIE); }
	virtual void onDelete() override;
	virtual void onObjectCreated() override;

	// update module methods
	virtual UpdateSleepTime update() override;

	// die module methods
	virtual DieModuleInterface *getDie() override { return this; }
	virtual void onDie( const DamageInfo *damageInfo ) override;
	virtual void onCapture( Player *oldOwner, Player *newOwner ) override;

	// Disabled conditions to process. Need to process when disabled, because our update needs to actively let go
	// of our effect on people.  We don't say "Be affected for n frames", we toggle people.  We need to process
	// so we can toggle everyone off.
	virtual DisabledMaskType getDisabledTypesToProcess() const override { return DISABLEDMASK_ALL; }

	// our own public module methods

protected:

	virtual void removeAllInfluence();			///< remove any influence we had on all objects we've affected
	virtual void doScan();									///< do a scan
	virtual void effectLogic( Object *obj, Bool giving,
														const PropagandaTowerBehaviorModuleData *modData);///< give/remove effect on object

	UnsignedInt m_lastScanFrame;									///< last frame we did a scan on

	ObjectTracker *m_insideList;									///< objects that are inside our area of influence
	const UpgradeTemplate *m_upgradeRequired;			///< Upgrade required to use the upgraded pulse FX

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Behavior/PropagandaTowerBehavior.cpp`
```cpp
// FILE: PropagandaTowerBehavior.cpp //////////////////////////////////////////////////////////////
// Author: Colin Day, August 2002
// Desc:   Behavior module for PropagandaTower
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"
#include "Common/GameState.h"
#include "Common/GameUtility.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/Upgrade.h"
#include "Common/Xfer.h"
#include "GameClient/Drawable.h"
#include "GameClient/FXList.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Weapon.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/PropagandaTowerBehavior.h"
#include "GameLogic/Module/BodyModule.h"




// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
enum ObjectID CPP_11(: Int);

// ------------------------------------------------------------------------------------------------
/** This class is used to track objects as they exit our area of influence */
// ------------------------------------------------------------------------------------------------
class ObjectTracker : public MemoryPoolObject
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( ObjectTracker, "ObjectTracker" );

public:

	ObjectTracker() { objectID = INVALID_ID; next = nullptr; }

	ObjectID objectID;
	ObjectTracker *next;

};
ObjectTracker::~ObjectTracker() { }

///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
PropagandaTowerBehaviorModuleData::PropagandaTowerBehaviorModuleData()
{

	m_scanRadius = 1.0f;
	m_scanDelayInFrames = 100;
	m_autoHealPercentPerSecond = 0.01f;
	m_upgradedAutoHealPercentPerSecond = 0.02f;
	m_pulseFX = nullptr;
	m_upgradeRequired = nullptr;
	m_upgradedPulseFX = nullptr;
	m_affectsSelf = FALSE;

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
/*static*/ void PropagandaTowerBehaviorModuleData::buildFieldParse( MultiIniFieldParse &p )
{
  UpdateModuleData::buildFieldParse( p );

	static const FieldParse dataFieldParse[] =
	{
		{ "Radius",									INI::parseReal,									nullptr,	offsetof( PropagandaTowerBehaviorModuleData, m_scanRadius ) },
		{ "DelayBetweenUpdates",		INI::parseDurationUnsignedInt,	nullptr,	offsetof( PropagandaTowerBehaviorModuleData, m_scanDelayInFrames ) },
		{ "HealPercentEachSecond",	INI::parsePercentToReal,				nullptr,	offsetof( PropagandaTowerBehaviorModuleData, m_autoHealPercentPerSecond ) },
		{ "UpgradedHealPercentEachSecond",	INI::parsePercentToReal,nullptr,	offsetof( PropagandaTowerBehaviorModuleData, m_upgradedAutoHealPercentPerSecond ) },
		{ "PulseFX",								INI::parseFXList,								nullptr,	offsetof( PropagandaTowerBehaviorModuleData, m_pulseFX ) },
		{ "UpgradeRequired",				INI::parseAsciiString,					nullptr, offsetof( PropagandaTowerBehaviorModuleData, m_upgradeRequired ) },
		{ "UpgradedPulseFX",				INI::parseFXList,								nullptr, offsetof( PropagandaTowerBehaviorModuleData, m_upgradedPulseFX ) },
		{ "AffectsSelf",						INI::parseBool,									nullptr, offsetof( PropagandaTowerBehaviorModuleData, m_affectsSelf ) },
		{ nullptr, nullptr, nullptr, 0 }
	};

  p.add( dataFieldParse );

}

///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
PropagandaTowerBehavior::PropagandaTowerBehavior( Thing *thing, const ModuleData *modData )
											 : UpdateModule( thing, modData )
{
	m_lastScanFrame = 0;
	m_insideList = nullptr;
	setWakeFrame( getObject(), UPDATE_SLEEP_NONE );

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
PropagandaTowerBehavior::~PropagandaTowerBehavior()
{

}

// ------------------------------------------------------------------------------------------------
/** Module is being deleted */
// ------------------------------------------------------------------------------------------------
void PropagandaTowerBehavior::onDelete()
{

	// remove any benefits from anybody in our area of influence
	removeAllInfluence();

}

// ------------------------------------------------------------------------------------------------
/** Resolve */
// ------------------------------------------------------------------------------------------------
void PropagandaTowerBehavior::onObjectCreated()
{
	const PropagandaTowerBehaviorModuleData *modData = getPropagandaTowerBehaviorModuleData();

	// convert module upgrade name to a pointer
	m_upgradeRequired = TheUpgradeCenter->findUpgrade( modData->m_upgradeRequired );

}

// ------------------------------------------------------------------------------------------------
void PropagandaTowerBehavior::onCapture( Player *oldOwner, Player *newOwner )
{
	// We don't function for the neutral player.
	if( newOwner == ThePlayerList->getNeutralPlayer() )
	{
		removeAllInfluence();
		setWakeFrame( getObject(), UPDATE_SLEEP_FOREVER );
	}
	else
		setWakeFrame( getObject(), UPDATE_SLEEP_NONE );
}

// ------------------------------------------------------------------------------------------------
/** The update callback */
// ------------------------------------------------------------------------------------------------
UpdateSleepTime PropagandaTowerBehavior::update()
{
/// @todo srj use SLEEPY_UPDATE here
	const PropagandaTowerBehaviorModuleData *modData = getPropagandaTowerBehaviorModuleData();

	//Sep 27, 2002 (Kris): Added this code to prevent the tower from working while under construction.
	Object *self = getObject();
	if( self->getStatusBits().test( OBJECT_STATUS_UNDER_CONSTRUCTION ) )
		return UPDATE_SLEEP_NONE;

	if( self->testStatus(OBJECT_STATUS_SOLD) )
	{
		removeAllInfluence();
		return UPDATE_SLEEP_FOREVER;
	}

	if( self->isEffectivelyDead() )
		return UPDATE_SLEEP_FOREVER;

	if( self->isDisabled() )
	{
		// We need to let go of everyone if we are EMPd or underpowered or yadda, but not if we are only held

		DisabledMaskType allButHeld = MAKE_DISABLED_MASK( DISABLED_HELD );
		FLIP_DISABLEDMASK(allButHeld);

		if( TEST_DISABLEDMASK_ANY(self->getDisabledFlags(), allButHeld) )
		{
			removeAllInfluence();
			return UPDATE_SLEEP_NONE;
		}
	}

#if RETAIL_COMPATIBLE_CRC
	if (self->getContainedBy() && self->getContainedBy()->getContainedBy())
#else
	// TheSuperHackers @bugfix If our container or any parent containers are enclosing, we turn the heck off.
	if (self->getEnclosingContainedBy())
#endif
	{
		removeAllInfluence();
		return UPDATE_SLEEP_NONE;
	}

	// if it's not time to scan, nothing to do
	UnsignedInt currentFrame = TheGameLogic->getFrame();
	if( currentFrame - m_lastScanFrame >= modData->m_scanDelayInFrames )
	{

		// do a scan
		doScan();
		m_lastScanFrame = currentFrame;

	}

	// go through any objects in our area of influence and do the effect logic on them
	Object *obj;
	ObjectTracker *curr = nullptr, *prev = nullptr, *next = nullptr;
	for( curr = m_insideList; curr; curr = next )
	{

		// get the next link
		next = curr->next;

		// find this object
		obj = TheGameLogic->findObjectByID( curr->objectID );
		if ((obj) &&
			 (obj->isKindOf(KINDOF_SCORE) || obj->isKindOf(KINDOF_SCORE_CREATE) || obj->isKindOf(KINDOF_SCORE_DESTROY) || obj->isKindOf(KINDOF_MP_COUNT_FOR_VICTORY)))
		{

			// give any bonus to this object
			effectLogic( obj, TRUE, getPropagandaTowerBehaviorModuleData() );

			// record this element as the previous one found in the list
			prev = curr;

		}
		else
		{

			//
			// actual object wasn't found, remove this entry from our inside list so we don't
			// have to search through it again
			//
			if( prev )
				prev->next = curr->next;
			else
				m_insideList = curr->next;
			deleteInstance(curr);

		}

	}

	return UPDATE_SLEEP_NONE;

}

// ------------------------------------------------------------------------------------------------
/** The death callback */
// ------------------------------------------------------------------------------------------------
void PropagandaTowerBehavior::onDie( const DamageInfo *damageInfo )
{

	// remove any benefits from anybody in our area of influence
	removeAllInfluence();

}

///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

// ------------------------------------------------------------------------------------------------
/** Grant or remove effect to this object */
// ------------------------------------------------------------------------------------------------
void PropagandaTowerBehavior::effectLogic( Object *obj, Bool giving,
																					 const PropagandaTowerBehaviorModuleData *modData )
{
	Bool effectUpgraded = getObject()->getControllingPlayer()->hasUpgradeComplete( m_upgradeRequired );

	// if giving the effect
	if( giving )
	{
		if ( obj->hasAnyDamageWeapon() == TRUE )
		{
			if( obj->testWeaponBonusCondition( WEAPONBONUSCONDITION_ENTHUSIASTIC ) == FALSE )
				obj->setWeaponBonusCondition( WEAPONBONUSCONDITION_ENTHUSIASTIC );


			if (effectUpgraded)
			{
				if (obj->testWeaponBonusCondition( WEAPONBONUSCONDITION_SUBLIMINAL ) == FALSE)
					obj->setWeaponBonusCondition( WEAPONBONUSCONDITION_SUBLIMINAL );
			}

		}


		// grant health to this object as well
		BodyModuleInterface *body = obj->getBodyModule();
		if( body )
		{
			Real healthPercent;
			if(effectUpgraded)
				healthPercent = modData->m_upgradedAutoHealPercentPerSecond;
			else
				healthPercent = modData->m_autoHealPercentPerSecond;

			Real amount = healthPercent / static_cast<float>(LOGICFRAMES_PER_SECOND) * body->getMaxHealth();

	// Dustin wants the healing effect not to stack from multiple propaganda towers...
	// To accomplish this, I'll give every object a single healing-sender (ID)
	// Any given healing recipient (object) can only receive healing from one particular healing sender
	// and cannot change healing senders until the previous one expires (its scandelay)

//		obj->attemptHealing(amount, getObject()); // the regular way to give healing...
			obj->attemptHealingFromSoleBenefactor( amount, getObject(), modData->m_scanDelayInFrames );//the non-stacking way

		}

	}
	else
	{

		// taking effect away
		obj->clearWeaponBonusCondition( WEAPONBONUSCONDITION_ENTHUSIASTIC );
		obj->clearWeaponBonusCondition( WEAPONBONUSCONDITION_SUBLIMINAL );

	}

}

// ------------------------------------------------------------------------------------------------
/** Remove all influence from objects we've given bonuses to */
// ------------------------------------------------------------------------------------------------
void PropagandaTowerBehavior::removeAllInfluence()
{
	ObjectTracker *o;

	// go through all objects we've given bonuses to and remove them
	Object *obj;
	for( o = m_insideList; o; o = o->next )
	{

		obj = TheGameLogic->findObjectByID( o->objectID );
		if( obj )
			effectLogic( obj, FALSE, getPropagandaTowerBehaviorModuleData() );

	}

	// delete the list of objects under our influence
	while( m_insideList )
	{

		o = m_insideList->next;
		deleteInstance(m_insideList);
		m_insideList = o;

	}

}

// ------------------------------------------------------------------------------------------------
/** Do a scan */
// ------------------------------------------------------------------------------------------------
void PropagandaTowerBehavior::doScan()
{
	const PropagandaTowerBehaviorModuleData *modData = getPropagandaTowerBehaviorModuleData();
	Object *us = getObject();
	ObjectTracker *newInsideList = nullptr;

	// The act of scanning is when we play our effect
	Bool upgradePresent = FALSE;
	if( m_upgradeRequired )
	{

		// see if we have the upgrade
		switch( m_upgradeRequired->getUpgradeType() )
		{

			// ------------------------------------------------------------------------------------------
			case UPGRADE_TYPE_PLAYER:
			{
				Player *player = us->getControllingPlayer();

				upgradePresent = player->hasUpgradeComplete( m_upgradeRequired );
				break;

			}

			// ------------------------------------------------------------------------------------------
			case UPGRADE_TYPE_OBJECT:
			{

				upgradePresent = us->hasUpgrade( m_upgradeRequired );
				break;

			}

			// ------------------------------------------------------------------------------------------
			default:
			{

				DEBUG_CRASH(( "PropagandaTowerBehavior::doScan - Unknown upgrade type '%d'",
											m_upgradeRequired->getUpgradeType() ));
				break;

			}

		}

	}


  Bool doFX = TRUE;
    // if it is us, or if it is he and we do see him
  Object *overlord = us->getContainedBy();
  if ( overlord )
  {
	  if ( us->getControllingPlayer() != rts::getObservedOrLocalPlayer() )// dealing with someone else's tower
    {
      if ( overlord->testStatus( OBJECT_STATUS_STEALTHED ) && !overlord->testStatus( OBJECT_STATUS_DETECTED ) )
        doFX = FALSE;// so they don't give up their position

    }

    // Sorry, this is a lot of hard coded mishmash, but it must be done... ship it!
    // Propaganda towers (proper) never get contained, so they don't care about this code
    // The PropTowers that ride on the backs of overlord tanks want to suppress their fx if the overlord is stealthed to the local player
    // The PropTowers that ride on the bellies of Helixes also suppress their fx when hiding stealthed (never happens?)
    // The PropTowers must do nothing, scanning-or-FX-wise when they are inside something... there are two cases
    //   1) When the prop tower is on an Overlord that is in a helicopter (likely a Helix II)
    //   2) When the prop tower is the EmperorTank, which is in a helicopter (likely a Helix II).

    if ( us->isKindOf( KINDOF_VEHICLE ) && !us->isKindOf( KINDOF_PORTABLE_STRUCTURE ) )//craptacular!
      // oh my dear Lord... I am not a propaganda tower, I am an emperorTank! that means the overlord is a helix or something!
      return; // proptowers that are riding IN things cannot do their scan!

    //okay this is wacky, but this proptower may be on an overlord thank that is riding in a helix... oy!
    Object *helix = overlord->getContainedBy();
    if ( helix )
      return; // proptowers that are riding ON things that are in-turn riding IN things cannot do their scan!

  }

	if ( us->getControllingPlayer() != rts::getObservedOrLocalPlayer() )// dealing with someone else's tower
	{
		if ( us->testStatus( OBJECT_STATUS_STEALTHED ) && !us->testStatus( OBJECT_STATUS_DETECTED ) )
		{
			doFX = FALSE;// Certainly don't play if we ourselves are stealthed.
		}
	}

  if ( doFX )
  {
    // play the right pulse
	  if( upgradePresent == TRUE )
		  FXList::doFXObj( modData->m_upgradedPulseFX, us );
	  else
		  FXList::doFXObj( modData->m_pulseFX, us );
  }

	// setup scan filters
	PartitionFilterRelationship relationship( us, PartitionFilterRelationship::ALLOW_ALLIES );
	PartitionFilterAlive filterAlive;
	PartitionFilterSameMapStatus filterMapStatus(us);
	PartitionFilterAcceptByKindOf filterOutBuildings(KINDOFMASK_NONE, MAKE_KINDOF_MASK(KINDOF_STRUCTURE));
	PartitionFilter *filters[] = {	&relationship,
																	&filterAlive,
																	&filterMapStatus,
																	&filterOutBuildings,
																	nullptr
																};

	// scan objects in our region
	ObjectIterator *iter = ThePartitionManager->iterateObjectsInRange( us->getPosition(),
																																		 modData->m_scanRadius,
																																		 FROM_CENTER_2D,
																																		 filters );
	MemoryPoolObjectHolder hold( iter );
	Object *obj;
	ObjectTracker *newEntry;
	for( obj = iter->first(); obj; obj = iter->next() )
	{

		// ignore ourselves, unless Design wants us to affect ourselves
		if( obj == us  &&  !modData->m_affectsSelf)
			continue;

		// record this object as being in the new "in list"
		newEntry = newInstance(ObjectTracker);
		newEntry->objectID = obj->getID();
		newEntry->next = newInsideList;
		newInsideList = newEntry;

	}

	//
	// now that we have a list of objects that are in our area of influence, look through
	// the objects that were last recorded as in our area of influence and remove any
	// bonus we've given them (if they are within the area of effect of another tower it's
	// OK, they'll get the bonus back again when that tower does a scan which won't be too long
	//
	for( ObjectTracker *curr = m_insideList; curr; curr = curr->next )
	{

		// find this entry in the new list
		ObjectTracker *o = nullptr;
		for( o = newInsideList; o; o = o->next )
			if( o->objectID == curr->objectID )
				break;

		// if entry wasn't there, remove the bonus from this object
		if( o == nullptr )
		{

			obj = TheGameLogic->findObjectByID( curr->objectID );
			if( obj )
				effectLogic( obj, FALSE, modData );

		}

	}

	// delete the inside list we have recorded
	ObjectTracker *next;
	while( m_insideList )
	{

		next = m_insideList->next;
		deleteInstance(m_insideList);
		m_insideList = next;

	}

	// set the new inside list to the one we're recording
	m_insideList = newInsideList;

}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void PropagandaTowerBehavior::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void PropagandaTowerBehavior::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// last scan frame
	xfer->xferUnsignedInt( &m_lastScanFrame );

	// inside list tracking
	ObjectTracker *trackerEntry;
	UnsignedShort insideCount = 0;
	for( trackerEntry = m_insideList; trackerEntry; trackerEntry = trackerEntry->next )
		insideCount++;
	xfer->xferUnsignedShort( &insideCount );
	if( xfer->getXferMode() == XFER_SAVE )
	{

		// write all entries
		for( trackerEntry = m_insideList; trackerEntry; trackerEntry = trackerEntry->next )
		{

			// object id
			xfer->xferObjectID( &trackerEntry->objectID );

		}

	}
	else
	{

		// sanity
		if( m_insideList != nullptr )
		{

			DEBUG_CRASH(( "PropagandaTowerBehavior::xfer - m_insideList should be empty but is not" ));
			throw SC_INVALID_DATA;

		}

		// read all entries
		for( UnsignedShort i = 0; i < insideCount; ++i )
		{

			// allocate new tracker entry and tie to list
			trackerEntry = newInstance(ObjectTracker);
			trackerEntry->next = m_insideList;
			m_insideList = trackerEntry;

			// read object id
			xfer->xferObjectID( &trackerEntry->objectID );

		}

	}

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void PropagandaTowerBehavior::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
