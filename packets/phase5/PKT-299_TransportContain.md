# Translation Packet `PKT-299`: `TransportContain`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-299`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/contain/transport_contain.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (17):**
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/ThingFactory.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ThingFactory.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameLogic/AI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/AI.h`
  - `GameLogic/AIPathfind.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/AIPathfind.h`
  - `GameLogic/Locomotor.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Locomotor.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/OpenContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/OpenContain.h`
  - `GameLogic/Module/PhysicsUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PhysicsUpdate.h`
  - `GameLogic/Module/StealthUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/StealthUpdate.h`
  - `GameLogic/Module/TransportContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/TransportContain.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/Weapon.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (14):**
  - `"Slots" => INI::parseInt`
  - `"ScatterNearbyOnExit" => INI::parseBool`
  - `"OrientLikeContainerOnExit" => INI::parseBool`
  - `"KeepContainerVelocityOnExit" => INI::parseBool`
  - `"GoAggressiveOnExit" => INI::parseBool`
  - `"ResetMoodCheckTimeOnExit" => INI::parseBool`
  - `"DestroyRidersWhoAreNotFreeToExit" => INI::parseBool`
  - `"ExitBone" => INI::parseAsciiString`
  - `"ExitPitchRate" => INI::parseAngularVelocityReal`
  - `"InitialPayload" => parseInitialPayload`
  - `"HealthRegen%PerSec" => INI::parseReal`
  - `"ExitDelay" => INI::parseDurationUnsignedInt`
  - `"ArmedRidersUpgradeMyWeaponSet" => INI::parseBool`
  - `"DelayExitInAir" => INI::parseBool`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (4 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferBool( &m_payloadCreated )`
  3. `xferInt( &m_extraSlotsInUse )`
  4. `xferUnsignedInt( &m_frameExitNotBusy )`
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if !RETAIL_COMPATIBLE_CRC`
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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/TransportContain.h`
```cpp
// FILE: TransportContain.h ////////////////////////////////////////////////////////////////////////
// Author: Steven Johnson, March 2002
// Desc:   Contain module for transport units.
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/OpenContain.h"

//-------------------------------------------------------------------------------------------------
class TransportContainModuleData : public OpenContainModuleData
{
public:
	struct InitialPayload
	{
		AsciiString name;
		Int count;
	};

	Int								m_slotCapacity;								///< max units that can be inside us
	Real							m_exitPitchRate;
	AsciiString				m_exitBone;
	InitialPayload		m_initialPayload;
	Real							m_healthRegen;
	UnsignedInt				m_exitDelay;
	Bool							m_scatterNearbyOnExit;
	Bool							m_orientLikeContainerOnExit;
	Bool							m_keepContainerVelocityOnExit;
	Bool							m_goAggressiveOnExit;
	Bool							m_armedRidersUpgradeWeaponSet;
	Bool							m_resetMoodCheckTimeOnExit;
	Bool							m_destroyRidersWhoAreNotFreeToExit;
	Bool							m_isDelayExitInAir;

	TransportContainModuleData();

	static void buildFieldParse(MultiIniFieldParse& p);
	static void parseInitialPayload( INI* ini, void *instance, void *store, const void* /*userData*/ );

};

//-------------------------------------------------------------------------------------------------
class TransportContain : public OpenContain
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( TransportContain, "TransportContain" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( TransportContain, TransportContainModuleData )

public:

	TransportContain( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	virtual Bool isValidContainerFor( const Object* obj, Bool checkCapacity) const override;

	virtual void onCapture( Player *oldOwner, Player *newOwner ) override; // have to kick everyone out on capture.
	virtual void onContaining( Object *obj, Bool wasSelected ) override;		///< object now contains 'obj'
	virtual void onRemoving( Object *obj ) override;			///< object no longer contains 'obj'
	virtual UpdateSleepTime update() override;							///< called once per frame

	virtual Bool isRiderChangeContain() const override { return FALSE; }
  virtual Bool isSpecialOverlordStyleContainer() const override {return FALSE;}

	virtual Int getContainMax() const override;

	virtual Int getExtraSlotsInUse() override { return m_extraSlotsInUse; }///< Transports have the ability to carry guys how take up more than spot.

	virtual Bool isExitBusy() const override;	///< Contain style exiters are getting the ability to space out exits, so ask this before reserveDoor as a kind of no-commitment check.
	virtual ExitDoorType reserveDoorForExit( const ThingTemplate* objType, Object *specificObject ) override;
	virtual void unreserveDoorForExit( ExitDoorType exitDoor ) override;
	virtual Bool isDisplayedOnControlBar() const override {return TRUE;}///< Does this container display its contents on the ControlBar?

protected:

	// exists primarily for TransportContain to override
	virtual void killRidersWhoAreNotFreeToExit() override;
	virtual Bool isSpecificRiderFreeToExit(Object* obj);
	virtual Bool isPassengerAllowedToFire( ObjectID id = INVALID_ID ) const override;	///< Hey, can I shoot out of this container?

	virtual void createPayload();
	void letRidersUpgradeWeaponSet();

	Bool m_payloadCreated;

private:

	Int m_extraSlotsInUse;
	UnsignedInt m_frameExitNotBusy;

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Contain/TransportContain.cpp`
```cpp
// FILE: TransportContain.cpp //////////////////////////////////////////////////////////////////////
// Author: Steven Johnson, March 2002
// Desc:   Contain module for transport units.
///////////////////////////////////////////////////////////////////////////////////////////////////

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/Player.h"
#include "Common/ThingTemplate.h"
#include "Common/ThingFactory.h"
#include "Common/Xfer.h"
#include "GameClient/Drawable.h"
#include "GameLogic/AI.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/Locomotor.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Module/StealthUpdate.h"
#include "GameLogic/Module/TransportContain.h"
#include "GameLogic/Object.h"
#include "GameLogic/Weapon.h"


// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
TransportContainModuleData::TransportContainModuleData()
{

	m_slotCapacity = 0;
	m_scatterNearbyOnExit = true;
	m_orientLikeContainerOnExit = false;
	m_keepContainerVelocityOnExit = false;
	m_goAggressiveOnExit = FALSE;
  m_armedRidersUpgradeWeaponSet = FALSE;
	m_resetMoodCheckTimeOnExit = true;
	m_destroyRidersWhoAreNotFreeToExit = false;
	m_exitPitchRate = 0.0f;
	m_initialPayload.count = 0;
	m_healthRegen = 0.0f;
	m_exitDelay = 0;
	m_isDelayExitInAir = FALSE;

	//
	// by default we say that transports can have infantry inside them, this will be totally
	// overwritten by any data provided from the INI entry tho
	//
	m_allowInsideKindOf = MAKE_KINDOF_MASK( KINDOF_INFANTRY );

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void TransportContainModuleData::parseInitialPayload( INI* ini, void *instance, void *store, const void* /*userData*/ )
{
	TransportContainModuleData* self = (TransportContainModuleData*)instance;
	const char* name = ini->getNextToken();
	const char* countStr = ini->getNextTokenOrNull();
	Int count = countStr ? INI::scanInt(countStr) : 1;

	self->m_initialPayload.name.set(name);
	self->m_initialPayload.count = count;
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void TransportContainModuleData::buildFieldParse(MultiIniFieldParse& p)
{
  OpenContainModuleData::buildFieldParse(p);

	static const FieldParse dataFieldParse[] =
	{
		{ "Slots",	INI::parseInt,		nullptr, offsetof( TransportContainModuleData, m_slotCapacity ) },
		{ "ScatterNearbyOnExit",	INI::parseBool,		nullptr, offsetof( TransportContainModuleData, m_scatterNearbyOnExit ) },
		{ "OrientLikeContainerOnExit",	INI::parseBool,		nullptr, offsetof( TransportContainModuleData, m_orientLikeContainerOnExit ) },
		{ "KeepContainerVelocityOnExit",	INI::parseBool,		nullptr, offsetof( TransportContainModuleData, m_keepContainerVelocityOnExit ) },
		{ "GoAggressiveOnExit",	INI::parseBool,		nullptr, offsetof( TransportContainModuleData, m_goAggressiveOnExit ) },
		{ "ResetMoodCheckTimeOnExit",	INI::parseBool,		nullptr, offsetof( TransportContainModuleData, m_resetMoodCheckTimeOnExit ) },
		{ "DestroyRidersWhoAreNotFreeToExit",	INI::parseBool,		nullptr, offsetof( TransportContainModuleData, m_destroyRidersWhoAreNotFreeToExit ) },
		{ "ExitBone",	INI::parseAsciiString,		nullptr, offsetof( TransportContainModuleData, m_exitBone ) },
		{ "ExitPitchRate",	INI::parseAngularVelocityReal,		nullptr, offsetof( TransportContainModuleData, m_exitPitchRate ) },
		{ "InitialPayload", parseInitialPayload, nullptr, 0 },
		{ "HealthRegen%PerSec", INI::parseReal, nullptr, offsetof( TransportContainModuleData, m_healthRegen ) },
		{ "ExitDelay",	INI::parseDurationUnsignedInt,		nullptr, offsetof( TransportContainModuleData, m_exitDelay ) },
		{ "ArmedRidersUpgradeMyWeaponSet",	INI::parseBool,		nullptr, offsetof( TransportContainModuleData, m_armedRidersUpgradeWeaponSet ) },
		{ "DelayExitInAir",	INI::parseBool,		nullptr, offsetof( TransportContainModuleData, m_isDelayExitInAir ) },

    { nullptr, nullptr, nullptr, 0 }
  };
  p.add(dataFieldParse);
}



// PRIVATE ////////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Int TransportContain::getContainMax() const
{
	if (getTransportContainModuleData())
		return getTransportContainModuleData()->m_slotCapacity;

	return 0;
}

// PUBLIC /////////////////////////////////////////////////////////////////////////////////////////
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
TransportContain::TransportContain( Thing *thing, const ModuleData *moduleData ) :
								 OpenContain( thing, moduleData )
{
	m_extraSlotsInUse = 0;
	m_frameExitNotBusy = 0;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
TransportContain::~TransportContain()
{

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
/**
	can this container contain this kind of object?
	and, if checkCapacity is TRUE, does this container have enough space left to hold the given unit?
*/
Bool TransportContain::isValidContainerFor(const Object* rider, Bool checkCapacity) const
{

	// sanity
	if (!rider)
		return false;

	// The point of this new code is to determine when something is a "fake" container, to
	// look at the object inside of it to use that as the valid check. There is a case, when a
	// paratrooper (an infantry contained in a parachute). In this case, when we pass this object
	// to contain in a transport plane, we want to check the infantry, not the parachute.
	if (rider->getContain() && rider->getContain()->isSpecialZeroSlotContainer())
	{
		// Report the first thing inside it!
		const ContainedItemsList *items = rider->getContain()->getContainedItemsList();
		if (items && !items->empty())
		{
			if (items->front())
			{
				// Replace the object we are checking with the *first* object contained within it.
				rider = items->front();
			}
		}
	}


	// extend functionality
	if( OpenContain::isValidContainerFor( rider, checkCapacity ) == false )
		return false;

//	// only allied objects can be transported.
//	// order matters: we want to know if I consider it to be an ally, not vice versa
//	if (getObject()->getRelationship(rider) != ALLIES)
//		return false;

// no... actually, only OUR OWN units can be transported.
	if (rider->getControllingPlayer() != getObject()->getControllingPlayer())
		return false;

	Int transportSlotCount = rider->getTransportSlotCount();

	// if 0, this object isn't transportable.
	if (transportSlotCount == 0)
		return false;

	if (checkCapacity)
	{
    Int containMax = getContainMax();
    Int containCount = getContainCount();

		return (m_extraSlotsInUse + containCount + transportSlotCount <= containMax);

	}
	else
	{
		return true;
	}
}



//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void TransportContain::letRidersUpgradeWeaponSet()
{

  const TransportContainModuleData * d = getTransportContainModuleData();

  if ( ! d->m_armedRidersUpgradeWeaponSet )
    return;

  Object *self = getObject();
  if ( self == nullptr )
    return;

  Bool anyRiderHasViableWeapon = FALSE;

	const ContainedItemsList* riderList = getContainedItemsList();
	if( riderList )
	{
		ContainedItemsList::const_iterator it;
		it = riderList->begin();

		while( it != riderList->end() )
		{
			Object *rider = *it;

			//Advance to the next iterator
			it++;

      if ( rider )
      {
		if(rider->isKindOf(KINDOF_INFANTRY) == false)
			continue;
        Weapon *weapon = nullptr;
        for ( Int w = PRIMARY_WEAPON; w < WEAPONSLOT_COUNT; ++ w )
        {
          weapon = rider->getWeaponInWeaponSlot( (WeaponSlotType)w );
          if ( weapon )
          {
            if ( weapon->getTemplate()->isContactWeapon() == FALSE && weapon->isDamageWeapon() == TRUE ) // THIS MAY NEED TO CHECK MORE WEAPON ATTRIBUTES TO WORK BEST
            {
              anyRiderHasViableWeapon = TRUE;
              break;
            }
          }
        }
      }

    }
  }

  if ( anyRiderHasViableWeapon )
  	self->setWeaponSetFlag( WEAPONSET_PLAYER_UPGRADE );
  else
  	self->clearWeaponSetFlag( WEAPONSET_PLAYER_UPGRADE );

}



//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void TransportContain::onContaining( Object *rider, Bool wasSelected )
{
	OpenContain::onContaining( rider, wasSelected );

	// objects inside a transport are held
	rider->setDisabled( DISABLED_HELD );

	Int transportSlotCount = rider->getTransportSlotCount();

	DEBUG_ASSERTCRASH(transportSlotCount > 0, ("Hmm, this object isnt transportable"));
	m_extraSlotsInUse += transportSlotCount - 1;
	DEBUG_ASSERTCRASH(m_extraSlotsInUse >= 0 && m_extraSlotsInUse + getContainCount() <= getContainMax(), ("Hmm, bad slot count"));

	//
	// when we go from holding nothing to holding something we have a model condition
	// to visually show the change
	//
	if( getContainCount() == 1 )
	{
		Drawable *draw = getObject()->getDrawable();

		if( draw )
			draw->setModelConditionState( MODELCONDITION_LOADED );

	}

  if ( getTransportContainModuleData()->m_armedRidersUpgradeWeaponSet )
    letRidersUpgradeWeaponSet();

	//Kris: October 20, 2003 - Patch 1.01
	//Force Jarmen Kell to transfer weapon timer for snipe to and from the combat bike.
	if( getObject()->isKindOf( KINDOF_CLIFF_JUMPER ) && rider->isKindOf( KINDOF_HERO ) && rider->isKindOf( KINDOF_SALVAGER ) )
	{
		//Admittedly brutal hack, but considering the state of the game (post-ship), this is a particularly surgical fix.
		Weapon *bikeWeapon = getObject()->getWeaponInWeaponSlot( SECONDARY_WEAPON );
		Weapon *riderWeapon = rider->getWeaponInWeaponSlot( SECONDARY_WEAPON );
		if( bikeWeapon && riderWeapon )
		{
			//Transfer the reload time from the rider to the bike
			bikeWeapon->transferNextShotStatsFrom( *riderWeapon );
		}
	}

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void TransportContain::onRemoving( Object *rider )
{
	OpenContain::onRemoving(rider);

	// object is no longer held inside a transport
	rider->clearDisabled( DISABLED_HELD );

	const TransportContainModuleData* d = getTransportContainModuleData();

	if (!d->m_exitBone.isEmpty())
	{
		Drawable* draw = getObject()->getDrawable();
		if (draw)
		{
			Coord3D bonePos, worldPos;
			if (draw->getPristineBonePositions(d->m_exitBone.str(), 0, &bonePos, nullptr, 1) == 1)
			{
				getObject()->convertBonePosToWorldPos(&bonePos, nullptr, &worldPos, nullptr);
				rider->setPosition(&worldPos);
			}
		}
	}

	if (d->m_orientLikeContainerOnExit)
	{
		rider->setOrientation(getObject()->getOrientation());
	}

	if (d->m_keepContainerVelocityOnExit)
	{
		PhysicsBehavior* parent = getObject()->getPhysics();
		PhysicsBehavior* child = rider->getPhysics();
		if (parent && child)
		{
			Coord3D startingForce = *parent->getVelocity();
			Real mass = child->getMass();
			startingForce.x *= mass;
			startingForce.y *= mass;
			startingForce.z *= mass;
			child->applyMotiveForce( &startingForce );

			Real pitchRate = child->getCenterOfMassOffset() * d->m_exitPitchRate;
			child->setPitchRate( pitchRate );
		}
	}

	Int transportSlotCount = rider->getTransportSlotCount();
	DEBUG_ASSERTCRASH(transportSlotCount > 0, ("Hmm, this object isnt transportable"));
	m_extraSlotsInUse -= transportSlotCount - 1;
	DEBUG_ASSERTCRASH(m_extraSlotsInUse >= 0 && m_extraSlotsInUse + getContainCount() <= getContainMax(), ("Hmm, bad slot count"));

	// when we are empty again, clear the model condition for loaded
	if( getContainCount() == 0 )
	{
		Drawable *draw = getObject()->getDrawable();

		if( draw )
			draw->clearModelConditionState( MODELCONDITION_LOADED );

	}

	if (getObject()->isAboveTerrain())
	{
		// temporarily mark the guy as being allowed to fall
		// (overriding his locomotor's stick-to-ground attribute).
		// this will be reset (by PhysicsBehavior) when he touches the ground.
		PhysicsBehavior* physics = rider->getPhysics();
		if (physics)
			physics->setAllowToFall(true);
	}

	// AI might need help using this transport in a good way.  Make the passengers aggressive.
	//There is no computer player check since Aggressive only means something for computer players anyway
	if( d->m_goAggressiveOnExit && rider->getAI() )
	{
		rider->getAI()->setAttitude( ATTITUDE_AGGRESSIVE );
	}
	if (getObject()->isEffectivelyDead()) {
		scatterToNearbyPosition(rider);
	}
	if( d->m_resetMoodCheckTimeOnExit && rider->getAI() )
	{
		rider->getAI()->wakeUpAndAttemptToTarget();
	}

	m_frameExitNotBusy = TheGameLogic->getFrame() + d->m_exitDelay;


  if ( d->m_armedRidersUpgradeWeaponSet )
    letRidersUpgradeWeaponSet();

	//Kris: October 20, 2003 - Patch 1.01
	//Force Jarmen Kell to transfer weapon timer for snipe to and from the combat bike.
	if( getObject()->isKindOf( KINDOF_CLIFF_JUMPER ) && rider->isKindOf( KINDOF_HERO ) && rider->isKindOf( KINDOF_SALVAGER ) )
	{
		//Admittedly brutal hack, but considering the state of the game (post-ship), this is a particularly surgical fix.
		Weapon *bikeWeapon = getObject()->getWeaponInWeaponSlot( SECONDARY_WEAPON );
		Weapon *riderWeapon = rider->getWeaponInWeaponSlot( SECONDARY_WEAPON );
		if( bikeWeapon && riderWeapon )
		{
			//Transfer the reload time from the bike to the rider
			riderWeapon->transferNextShotStatsFrom( *bikeWeapon );
		}
	}

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void TransportContain::createPayload()
{
	TransportContainModuleData* self = (TransportContainModuleData*)getTransportContainModuleData();

	Int count = self->m_initialPayload.count;
	const ThingTemplate* payloadTemplate = TheThingFactory->findTemplate( self->m_initialPayload.name );
	Object* object = getObject();
	ContainModuleInterface *contain = object->getContain();

	if( contain )
	{
		contain->enableLoadSounds( FALSE );
		for( int i = 0; i < count; i++ )
		{
			//We are creating a transport that comes with a initial payload, so add it now!
			Object* payload = TheThingFactory->newObject( payloadTemplate, object->getControllingPlayer()->getDefaultTeam() );
			if( contain->isValidContainerFor( payload, true ) )
			{
				contain->addToContain( payload );
			}
			else
			{
				DEBUG_CRASH( ( "DeliverPayload: PutInContainer %s is full, or not valid for the payload %s!", object->getName().str(), self->m_initialPayload.name.str() ) );
			}
		}
		contain->enableLoadSounds( TRUE );
	}

	m_payloadCreated = TRUE;

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
UpdateSleepTime TransportContain::update()
{
	const TransportContainModuleData *moduleData = getTransportContainModuleData();

	if (m_payloadCreated == FALSE)
	{
#if RETAIL_COMPATIBLE_CRC
		createPayload();
#else
		// TheSuperHackers @bugfix Caball009 25/05/2026 Don't create payload
		// for destroyed object to avoid leaving the payload in an invalid state.
		if (!getObject()->isDestroyed())
		{
			createPayload();
		}
#endif
	}

	if( moduleData && moduleData->m_healthRegen )
	{
		ContainModuleInterface *contain = getObject()->getContain();
		if( contain )
		{
			//This transport has a health regeneration value, so go through and heal all inside.
			const ContainedItemsList* items = contain->getContainedItemsList();
			if( items )
			{
				ContainedItemsList::const_iterator it;
				it = items->begin();

				while( it != items->end() )
				{
					Object *object = *it;

					//Advance to the next iterator
					it++;

					//Determine if we need healing or not.
					BodyModuleInterface *body = object->getBodyModule();
					if( body->getHealth() < body->getMaxHealth() )
					{
						//Calculate the health to be regenerated on each unit.
						Real regen = body->getMaxHealth() * moduleData->m_healthRegen / 100.0f * SECONDS_PER_LOGICFRAME_REAL;

						//Perform the actual healing for this frame.
//						DamageInfo damageInfo;
//						damageInfo.in.m_damageType = DAMAGE_HEALING;
//						damageInfo.in.m_deathType = DEATH_NONE;
//						damageInfo.in.m_sourceID = getObject()->getID();
//						damageInfo.in.m_amount = regen;
//						object->attemptDamage( &damageInfo );
						object->attemptHealing( regen, getObject() );
					}
				}
			}
		}
	}
	return OpenContain::update(); //extend
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void TransportContain::unreserveDoorForExit( ExitDoorType exitDoor )
{
	/* nothing */
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void TransportContain::killRidersWhoAreNotFreeToExit()
{
	const TransportContainModuleData* d = getTransportContainModuleData();
	ContainedItemsList::const_iterator it = getContainList().begin();
	while( it != getContainList().end() )
	{
		// get the object
		Object *obj = *it;
		// increment the iterator, since death will pull this guy from the list somewhere else
		++it;

		if (!isSpecificRiderFreeToExit(obj)) // only TransportContain has a meaningful failure for isFreeToExit
		{
			// If we cannot exit this guy legally, kill the bastard before removeAllContained forces him out.
			if (d->m_destroyRidersWhoAreNotFreeToExit)
				TheGameLogic->destroyObject(obj);
			else
#if RETAIL_COMPATIBLE_CRC
				obj->kill();
#else
				// TheSuperHackers @info Burned death prevents infantry corpses dropping out of the container.
				obj->kill(DAMAGE_UNRESISTABLE, d->m_isBurnedDeathToUnits ? DEATH_BURNED : DEATH_NORMAL);
#endif
		}
	}
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
Bool TransportContain::isSpecificRiderFreeToExit(Object* specificObject)
{
	if( specificObject == nullptr )
		return TRUE;	// I can, in general, exit people.

	// This is a override, not an extend.  I will check for game legality for
	// okaying the call to exitObjectViaDoor.
  const Object* me = getObject();

	// this is present solely for some transports to override, so that they can land before
	// allowing people to exit...
	const AIUpdateInterface* ai = me->getAIUpdateInterface();
	if (ai && ai->getAiFreeToExit(specificObject) != FREE_TO_EXIT)
		return FALSE;

#if !RETAIL_COMPATIBLE_CRC
	// TheSuperHackers @bugfix Stubbjax/bobtista 01/08/2026 If our container is itself contained,
	// then we are not free to exit.
	// GeneralsX @bugfix UnicodeApocalypse 12/09/2026 Except for special Overlord-style containers (e.g. Battle Bunker).
	if (me->isContained())
	{
		const ContainModuleInterface* outerContain = me->getContainedBy()->getContain();
		if (outerContain == nullptr || !outerContain->isSpecialOverlordStyleContainer())
			return FALSE;
	}
#endif

  // I can always kick people out if I am in the air, I know what I'm doing
  if (me->isUsingAirborneLocomotor())
   	return TRUE;

  const Coord3D *myPosition = me->getPosition();
 	if (!specificObject->getAIUpdateInterface())
		return FALSE;

	const Locomotor *hisLocomotor = specificObject->getAIUpdateInterface()->getCurLocomotor();
	if( hisLocomotor == nullptr )
   	return FALSE;

  // He can't get to this spot naturally, so I can't force him there.  (amphib transport)
  if (!TheAI->pathfinder()->validMovementTerrain(me->getLayer(), hisLocomotor, myPosition))
   	return FALSE;

  return TRUE;
}

Bool TransportContain::isPassengerAllowedToFire( ObjectID id ) const
{
	Object *passenger = TheGameLogic->findObjectByID(id);

	if( passenger != nullptr )
	{
		//only allow infantry, and turrets and such.  no vehicles.
		if( passenger->isKindOf(KINDOF_INFANTRY) == FALSE )
			return FALSE;
	}

  if ( ! getObject() )
    return FALSE;

  // but wait! I may be riding on an Overlord
  // This code detects the case of whether the contained passenger is in a bunker riding on an overlord, inside a helix!
  // Oh  my  God.
  const Object *heWhoContainsMe = getObject()->getContainedBy();
  if ( heWhoContainsMe)
  {
    ContainModuleInterface *hisContain = heWhoContainsMe->getContain();
    DEBUG_ASSERTCRASH( hisContain,("TransportContain::isPassengerAllowedToFire()... CONTAINER WITHOUT A CONTAIN! AARRGH!") );
    if ( hisContain && hisContain->isSpecialOverlordStyleContainer() )
      return hisContain->isPassengerAllowedToFire( id );
  }



	return OpenContain::isPassengerAllowedToFire();
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
ExitDoorType TransportContain::reserveDoorForExit( const ThingTemplate* objType, Object *specificObject )
{
	return isSpecificRiderFreeToExit(specificObject) ? DOOR_1 : DOOR_NONE_AVAILABLE;
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
Bool TransportContain::isExitBusy() const	///< Contain style exiters are getting the ability to space out exits, so ask this before reserveDoor as a kind of no-commitment check.
{
	const TransportContainModuleData *data = getTransportContainModuleData();
	if( data->m_isDelayExitInAir && getObject()->isAboveTerrain() )
		return TRUE;

	return TheGameLogic->getFrame() < m_frameExitNotBusy;
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void TransportContain::onCapture( Player *oldOwner, Player *newOwner )
{
	if( oldOwner != newOwner )
	{
		if( getObject()->isDisabledByType( DISABLED_UNMANNED ) )
		{
			//If this vehicle was sniped, then instantly eject everyone (otherwise, the next guy to eject will recapture it).
			removeAllContained();
		}
		else
		{
			//Use standard
#if RETAIL_COMPATIBLE_CRC
			orderAllPassengersToExit( CMD_FROM_AI, FALSE );
#else
      // TheSuperHackers @bugfix Stubbjax 20/11/2025 Only eject passengers if the new owner is not allied with the old owner.
			if (oldOwner->getRelationship(newOwner->getDefaultTeam()) != ALLIES)
				orderAllPassengersToExit(CMD_FROM_AI, FALSE);
#endif
		}
	}
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void TransportContain::crc( Xfer *xfer )
{

	// extend base class
	OpenContain::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void TransportContain::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	OpenContain::xfer( xfer );

	// payload created
	xfer->xferBool( &m_payloadCreated );

	// extra slots in use
	xfer->xferInt( &m_extraSlotsInUse );

	// frame exit not busy
	xfer->xferUnsignedInt( &m_frameExitNotBusy );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void TransportContain::loadPostProcess()
{

	// extend base class
	OpenContain::loadPostProcess();

}
```
