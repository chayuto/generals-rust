# Translation Packet `PKT-282`: `OpenContain`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-282`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/contain/open_contain.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (32):**
  - `Common/AudioEventRTS.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/AudioEventRTS.h`
  - `Common/BitFlagsIO.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/BitFlagsIO.h`
  - `Common/GameAudio.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameAudio.h`
  - `Common/GameMemory.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameMemory.h`
  - `Common/GameState.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/GameState.h`
  - `Common/KindOf.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/KindOf.h`
  - `Common/ModelState.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ModelState.h`
  - `Common/Module.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Module.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/RandomValue.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/RandomValue.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/ControlBar.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/ControlBar.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameLogic/AIPathfind.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/AIPathfind.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/CollideModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CollideModule.h`
  - `GameLogic/Module/ContainModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ContainModule.h`
  - `GameLogic/Module/DamageModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DamageModule.h`
  - `GameLogic/Module/DieModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DieModule.h`
  - `GameLogic/Module/OpenContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/OpenContain.h`
  - `GameLogic/Module/PhysicsUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PhysicsUpdate.h`
  - `GameLogic/Module/StealthUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/StealthUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `GameLogic/Weapon.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h`
  - `PreRTS.h`
- **Implemented Module Facets (5):**
  - `CollideModuleInterface* getCollide()`
  - `ContainModuleInterface* getContain()`
  - `DamageModuleInterface* getDamage()`
  - `DieModuleInterface* getDie()`
  - `ExitInterface* getContainExitInterface()`
- **INI `FieldParse` Table Entries (15):**
  - `"ContainMax" => INI::parseInt`
  - `"EnterSound" => INI::parseAudioEventRTS`
  - `"ExitSound" => INI::parseAudioEventRTS`
  - `"DamagePercentToUnits" => INI::parsePercentToReal`
  - `"BurnedDeathToUnits" => INI::parseBool`
  - `"AllowInsideKindOf" => KindOfMaskType::parseFromINI`
  - `"ForbidInsideKindOf" => KindOfMaskType::parseFromINI`
  - `"PassengersAllowedToFire" => INI::parseBool`
  - `"PassengersInTurret" => INI::parseBool`
  - `"NumberOfExitPaths" => INI::parseInt`
  - `"DoorOpenTime" => INI::parseDurationUnsignedInt`
  - `"WeaponBonusPassedToPassengers" => INI::parseBool`
  - `"AllowAlliesInside" => INI::parseBool`
  - `"AllowEnemiesInside" => INI::parseBool`
  - `"AllowNeutralInside" => INI::parseBool`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (24 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUnsignedInt( &m_containListSize )`
  3. `xferObjectID( &objectID )`
  4. `xferUnsignedInt( &m_containListSize )`
  5. `xferObjectID( &objectID )`
  6. `xferUser( &m_playerEnteredMask, sizeof( PlayerMaskType ) )`
  7. `xferUnsignedInt( &m_lastUnloadSoundFrame )`
  8. `xferUnsignedInt( &m_lastLoadSoundFrame )`
  9. `xferUnsignedInt( &m_stealthUnitsContained )`
  10. `xferUnsignedInt( &m_doorCloseCountdown )`
  11. `xferUser( &m_firePoints, sizeof( Matrix3D ) * MAX_FIRE_POINTS )`
  12. `xferInt( &m_firePointStart )`
  13. `xferInt( &m_firePointNext )`
  14. `xferInt( &m_firePointSize )`
  15. `xferBool( &m_noFirePointsInArt )`
  16. `xferCoord3D( &m_rallyPoint )`
  17. `xferBool( &m_rallyPointExists )`
  18. `xferUnsignedShort( &enterExitCount )`
  19. `xferObjectID( &objectID )`
  20. `xferUser( &enterExitType, sizeof( ObjectEnterExitType ) )`
  21. `xferObjectID( &objectID )`
  22. `xferUser( &enterExitType, sizeof( ObjectEnterExitType ) )`
  23. `xferInt( &m_whichExitPath )`
  24. `xferBool( &m_passengerAllowedToFire )`
- **PRNG Calls (3 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValueReal( 0.0f, 2.0f * PI )`
  - `GameLogicRandomValueReal( PI / 3.0f, 2.0f * (PI / 3.0F) )`
  - `GameLogicRandomValueReal( minRadius, maxRadius )`
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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/OpenContain.h`
```cpp
// FILE: OpenContain.h ////////////////////////////////////////////////////////////////////////////
// Author: Colin Day, November 2001
// Desc:   The OpenContainer ContainModule allows objects to be contained inside of other
//				 objects.  There is a set of functionality that will be common to
//				 all container modules that provides the actual containment
//				 implementations, those implementations are found here
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/CollideModule.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Module/DieModule.h"
#include "GameLogic/Module/DamageModule.h"
#include "Common/AudioEventRTS.h"
#include "Common/KindOf.h"
#include "Common/GameMemory.h"
#include "Common/ModelState.h"

// ------------------------------------------------------------------------------------------------
enum { CONTAIN_MAX_UNKNOWN = -1 };  // means we don't care, infinite, unassigned, whatever

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class OpenContainModuleData : public UpdateModuleData
{

public:
	DieMuxData m_dieMuxData;
	Int m_containMax;								///< how many things we can have inside (-1 = "I don't care")
	AudioEventRTS m_enterSound;			///< sound to play on entering
	AudioEventRTS m_exitSound;			///< sound to play on exiting
	Bool m_passengersAllowedToFire;	///< Can the passengers shoot out of us?
	Bool m_passengersInTurret;			///< The Firepoint bones are in our turret, not our chassis
	Int m_numberOfExitPaths;				///< Will alternate through ExitStart/End paths as we exit people.
	Real m_damagePercentageToUnits;
	Bool m_isBurnedDeathToUnits;		///< Turn off the hardcoded burn death when killing guys in transport
	UnsignedInt m_doorOpenTime;
	KindOfMaskType m_allowInsideKindOf;			///< objects must have at least one of these kind of bits set to be contained by us
	KindOfMaskType m_forbidInsideKindOf;		///< objects must have NONE of these kind of bits set to be contained by us
	Bool m_weaponBonusPassedToPassengers;		///< Do our passengers get to use our weapon bonuses?
 	Bool m_allowAlliesInside;				///< allow allies inside us
 	Bool m_allowEnemiesInside;			///< allow enemies inside us
 	Bool m_allowNeutralInside;			///< allow neutral inside us

	OpenContainModuleData();
	static void buildFieldParse(MultiIniFieldParse& p);
};

//-------------------------------------------------------------------------------------------------
/** An open container can actually contain other objects */
//-------------------------------------------------------------------------------------------------
class OpenContain : public UpdateModule,
										public ContainModuleInterface,
										public CollideModuleInterface,
										public DieModuleInterface,
										public DamageModuleInterface,
										public ExitInterface
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( OpenContain, "OpenContain" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( OpenContain, OpenContainModuleData )

public:

	OpenContain( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	virtual ContainModuleInterface* getContain() override { return this; }
	virtual CollideModuleInterface* getCollide() override { return this; }
	virtual DieModuleInterface* getDie() override { return this; }
	virtual DamageModuleInterface* getDamage() override { return this; }
	static Int getInterfaceMask() { return UpdateModule::getInterfaceMask() | (MODULEINTERFACE_CONTAIN) | (MODULEINTERFACE_COLLIDE) | (MODULEINTERFACE_DIE) | (MODULEINTERFACE_DAMAGE); }

	virtual void onDie( const DamageInfo *damageInfo ) override;  ///< the die callback
	virtual void onDelete() override;	///< Last possible moment cleanup
	virtual void onCapture( Player *oldOwner, Player *newOwner ) override {}

	// CollideModuleInterface
	virtual void onCollide( Object *other, const Coord3D *loc, const Coord3D *normal ) override;
	virtual Bool wouldLikeToCollideWith(const Object* other) const override { return false; }
	virtual Bool isCarBombCrateCollide() const override { return false; }
	virtual Bool isHijackedVehicleCrateCollide() const override { return false; }
	virtual Bool isRailroad() const override { return false;}
	virtual Bool isSalvageCrateCollide() const override { return false; }
	virtual Bool isSabotageBuildingCrateCollide() const override { return FALSE; }

	// UpdateModule
	virtual UpdateSleepTime update() override;				///< called once per frame

	// ContainModuleInterface
	virtual OpenContain *asOpenContain() override { return this; }  ///< treat as open container

	// DamageModuleInterface
	virtual void onDamage( DamageInfo *damageInfo ) override {};	///< damage callback
	virtual void onHealing( DamageInfo *damageInfo ) override {};	///< healing callback
	virtual void onBodyDamageStateChange( const DamageInfo* damageInfo,
																				BodyDamageType oldState,
																				BodyDamageType newState) override {};  ///< state change callback


	// our object changed position... react as appropriate.
	virtual void containReactToTransformChange() override;

	virtual Bool calcBestGarrisonPosition( Coord3D *sourcePos, const Coord3D *targetPos ) override { return FALSE; }
	virtual Bool attemptBestFirePointPosition( Object *source, Weapon *weapon, Object *victim ) override { return FALSE; }
	virtual Bool attemptBestFirePointPosition( Object *source, Weapon *weapon, const Coord3D *targetPos ) override { return FALSE; }

	///< if my object gets selected, then my visible passengers should, too
	///< this gets called from
	virtual void clientVisibleContainedFlashAsSelected() override {};

	virtual const Player* getApparentControllingPlayer(const Player* observingPlayer) const override { return nullptr; }
	virtual void recalcApparentControllingPlayer() override { }

	virtual void onContaining( Object *obj, Bool wasSelected ) override;		///< object now contains 'obj'
	virtual void onRemoving( Object *obj ) override;			///< object no longer contains 'obj'
	virtual void onSelling() override;///< Container is being sold.  Open responds by kicking people out

	virtual void orderAllPassengersToExit( CommandSourceType commandSource, Bool instantly ) override; ///< All of the smarts of exiting are in the passenger's AIExit. removeAllFrommContain is a last ditch system call, this is the game Evacuate
	virtual void orderAllPassengersToIdle( CommandSourceType commandSource ) override; ///< Just like it sounds
	virtual void orderAllPassengersToHackInternet( CommandSourceType ) override; ///< Just like it sounds
	virtual void markAllPassengersDetected() override;										///< Cool game stuff got added to the system calls since this layer didn't exist, so this regains that functionality

	// default OpenContain has unlimited capacity...!
	virtual Bool isValidContainerFor(const Object* obj, Bool checkCapacity) const override;
	virtual void addToContain( Object *obj ) override;				///< add 'obj' to contain list
	virtual void addToContainList( Object *obj ) override;		///< The part of AddToContain that inheritors can override (Can't do whole thing because of all the private stuff involved)
	virtual void removeFromContain( Object *obj, Bool exposeStealthUnits = FALSE ) override;	///< remove 'obj' from contain list
	virtual void removeAllContained( Bool exposeStealthUnits = FALSE ) override;				///< remove all objects on contain list
	virtual void killAllContained() override;				///< kill all objects on contain list
  virtual void harmAndForceExitAllContained( DamageInfo *info ) override; // apply canned damage against those contains
	virtual Bool isEnclosingContainerFor( const Object *obj ) const override;	///< Does this type of Contain Visibly enclose its contents?
	virtual Bool isPassengerAllowedToFire( ObjectID id = INVALID_ID ) const override;	///< Hey, can I shoot out of this container?

  virtual void setPassengerAllowedToFire( Bool permission = TRUE ) override { m_passengerAllowedToFire = permission; }	///< Hey, can I shoot out of this container?

  virtual void setOverrideDestination( const Coord3D * ) override {} ///< Instead of falling peacefully towards a clear spot, I will now aim here
	virtual Bool isDisplayedOnControlBar() const override {return FALSE;}///< Does this container display its contents on the ControlBar?
	virtual Int getExtraSlotsInUse() override { return 0; }
	virtual Bool isKickOutOnCapture() override { return TRUE; }///< By default, yes, all contain modules kick passengers out on capture

	// contain list access
	virtual void iterateContained( ContainIterateFunc func, void *userData, Bool reverse ) override;
	virtual UnsignedInt getContainCount() const override { return m_containListSize; }
	virtual const ContainedItemsList* getContainedItemsList() const override { return &m_containList; }
	virtual Bool isContained( const Object *obj ) const override;
	virtual const Object *friend_getRider() const override {return nullptr;} ///< Damn.  The draw order dependency bug for riders means that our draw module needs to cheat to get around it.
	virtual Real getContainedItemsMass() const override;
	virtual UnsignedInt getStealthUnitsContained() const override { return m_stealthUnitsContained; }
	virtual UnsignedInt getHeroUnitsContained() const override { return m_heroUnitsContained; }

	virtual PlayerMaskType getPlayerWhoEntered() const override { return m_playerEnteredMask; }

	virtual Int getContainMax() const override;

	// ExitInterface
	virtual Bool isExitBusy() const override {return FALSE;}	///< Contain style exiters are getting the ability to space out exits, so ask this before reserveDoor as a kind of no-commitment check.
	virtual ExitDoorType reserveDoorForExit( const ThingTemplate* objType, Object *specificObject ) override { return DOOR_1; }
	virtual void exitObjectViaDoor( Object *newObj, ExitDoorType exitDoor ) override;
	virtual void exitObjectInAHurry( Object *newObj ) override;


	virtual void unreserveDoorForExit( ExitDoorType exitDoor ) override { /*nothing*/ }
	virtual void exitObjectByBudding( Object *newObj, Object *budHost ) override { return; };

	virtual void setRallyPoint( const Coord3D *pos ) override;				///< define a "rally point" for units to move towards
	virtual const Coord3D *getRallyPoint() const override;			///< define a "rally point" for units to move towards
	virtual Bool getExitPosition(Coord3D& exitPosition ) const override { return FALSE; };					///< access to the "Door" position of the production object
	virtual Bool getNaturalRallyPoint( Coord3D& rallyPoint, Bool offset = TRUE ) const override;			///< get the natural "rally point" for units to move towards

	virtual ExitInterface* getContainExitInterface() override { return this; }

	virtual Bool isGarrisonable() const override { return false; }		///< can this unit be Garrisoned? (ick)
	virtual Bool isBustable() const override { return false; }		///< can this container get busted by a bunkerbuster
	virtual Bool isHealContain() const override { return false; } ///< true when container only contains units while healing (not a transport!)
	virtual Bool isTunnelContain() const override { return FALSE; }
	virtual Bool isRiderChangeContain() const override { return FALSE; }
	virtual Bool isSpecialZeroSlotContainer() const override { return false; }
	virtual Bool isImmuneToClearBuildingAttacks() const override { return true; }
  virtual Bool isSpecialOverlordStyleContainer() const override { return false; }
  virtual Bool isAnyRiderAttacking() const override;

	/**
		this is used for containers that must do something to allow people to enter or exit...
		eg, land (for Chinook), open door (whatever)... it's called with wants=WANTS_TO_ENTER
		when something is in the enter state, and wants=ENTS_NOTHING when the unit has
		either entered, or given up...
	*/
	virtual void onObjectWantsToEnterOrExit(Object* obj, ObjectEnterExitType wants) override;

	// returns true iff there are objects currently waiting to enter.
	virtual Bool hasObjectsWantingToEnterOrExit() const override;

	virtual void processDamageToContained(Real percentDamage) override; ///< Do our % damage to units now.
#if RETAIL_COMPATIBLE_CRC
	void processDamageToContainedInternal(Object* const* objects, size_t size, Real percentDamage);
#endif

	virtual Bool isWeaponBonusPassedToPassengers() const override;
	virtual WeaponBonusConditionFlags getWeaponBonusPassedToPassengers() const override;

	virtual void enableLoadSounds( Bool enable ) override { m_loadSoundsEnabled = enable; }

  Real getDamagePercentageToUnits();
  virtual Object* getClosestRider ( const Coord3D *pos ) override;

  virtual void setEvacDisposition( EvacDisposition disp ) override {};
protected:

	virtual void monitorConditionChanges();				///< check to see if we need to update our occupant positions from a model change or anything else
	virtual void putObjAtNextFirePoint( Object *obj );	///< place object at position of the next fire point to use
	virtual void redeployOccupants();							///< redeploy any objects at firepoints due to a model condition change

	const ContainedItemsList& getContainList() const { return m_containList; }

	void scatterToNearbyPosition(Object* obj);
	void removeFromContainViaIterator( ContainedItemsList::iterator it, Bool exposeStealthUnits = FALSE );  ///< remove item from contain list
	void removeFromPassengerViaIterator( ContainedItemsList::iterator it );///< remove item from passenger list

	virtual void doLoadSound();
	virtual void doUnloadSound();
	virtual void positionContainedObjectsRelativeToContainer(){}

	virtual void addOrRemoveObjFromWorld(Object* obj, Bool add);

	// exists primarily for TransportContain to override
	virtual void killRidersWhoAreNotFreeToExit() { }

	void pruneDeadWanters();

	ContainedItemsList	m_containList;						///< the list of contained objects
	UnsignedInt					m_containListSize;							///< size of contained list
private:

	typedef std::map< ObjectID, ObjectEnterExitType, std::less<ObjectID>/**/> ObjectEnterExitMap;

	ObjectEnterExitMap	m_objectEnterExitInfo;
	UnsignedInt					m_stealthUnitsContained;				///< number of stealth units that can't be seen by enemy players.
	UnsignedInt					m_heroUnitsContained;						///< cached hero count
	Int									m_whichExitPath; ///< Cycles from 1 to n and is used only in modules whose data has numberOfExitPaths > 1.
	UnsignedInt					m_doorCloseCountdown;						///< When should I shut my door.

	std::list<ObjectID>	m_xferContainIDList;		///< for loading m_containList from a save game
	PlayerMaskType			m_playerEnteredMask;					///< Mask of player that entered last, if any.
	UnsignedInt					m_lastUnloadSoundFrame;					///< last frame we did an un-loading sound
	UnsignedInt					m_lastLoadSoundFrame;						///< last frame we did a loading sound

/// @todo srj -- move this to a lazily-allocated subobject
	enum { MAX_FIRE_POINTS = 32 };
	ModelConditionFlags	m_conditionState;				///< The Drawables current behavior state
	Matrix3D						m_firePoints[ MAX_FIRE_POINTS ];
	Int									m_firePointStart;												///< start firepoint index to use when building becomes occupied
	Int									m_firePointNext;												///< next index to place objects at
	Int									m_firePointSize;												///< how many entries in m_firePoint are valid
	Bool								m_noFirePointsInArt;										///< TRUE when no fire point bones exist in the art

	Coord3D							m_rallyPoint;												///< Where units should move to after they have reached the "natural" rally point
	Bool								m_rallyPointExists;										///< Only move to the rally point if this is true
	Bool								m_loadSoundsEnabled;								///< Don't serialize -- used for disabling sounds during payload creation.
  Bool                m_passengerAllowedToFire;      ///< Newly promoted from the template data to the module for upgrade overriding access
};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Contain/OpenContain.cpp`
```cpp
// FILE: OpenContain.cpp //////////////////////////////////////////////////////////////////////////
// Author: Colin Day, November 2001
// Desc:   The OpenContain ContainModule allows objects to be contained inside of other
//				 objects.  There is a set of functionality that will be common to
//				 all container modules that provides the actual containment
//				 implementations, those implementations are found here
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/BitFlagsIO.h"
#include "Common/GameAudio.h"
#include "Common/GameState.h"
#include "Common/Module.h"
#include "Common/Player.h"
#include "Common/RandomValue.h"
#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"

#include "GameClient/Drawable.h"
#include "GameClient/InGameUI.h"
#include "GameClient/ControlBar.h"

#include "GameLogic/AIPathfind.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/OpenContain.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Module/StealthUpdate.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Weapon.h"


///////////////////////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS ///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
OpenContainModuleData::OpenContainModuleData()
{

	m_containMax = CONTAIN_MAX_UNKNOWN;  // means we don't care, infinite, unassigned, whatever
	m_passengersAllowedToFire = FALSE;
	m_passengersInTurret = FALSE;
	m_numberOfExitPaths = 1;
	m_damagePercentageToUnits = 0;
	m_isBurnedDeathToUnits = TRUE;
	m_doorOpenTime = 1;
	m_allowInsideKindOf.clear(); m_allowInsideKindOf.flip();		// everything is allowed
	m_forbidInsideKindOf.clear();	// nothing is forbidden
	m_weaponBonusPassedToPassengers = FALSE;
 	m_allowAlliesInside = TRUE;
 	m_allowEnemiesInside = TRUE;
 	m_allowNeutralInside = TRUE;
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
/*static*/ void OpenContainModuleData::buildFieldParse(MultiIniFieldParse& p)
{
  UpdateModuleData::buildFieldParse(p);

	static const FieldParse dataFieldParse[] =
	{
		{ "ContainMax",								INI::parseInt, nullptr, offsetof( OpenContainModuleData, m_containMax ) },
		{ "EnterSound",								INI::parseAudioEventRTS,		nullptr, offsetof( OpenContainModuleData, m_enterSound ) },
		{ "ExitSound",								INI::parseAudioEventRTS,		nullptr, offsetof( OpenContainModuleData, m_exitSound ) },
		{ "DamagePercentToUnits",			INI::parsePercentToReal,		nullptr, offsetof( OpenContainModuleData, m_damagePercentageToUnits ) },
		{ "BurnedDeathToUnits",				INI::parseBool,							nullptr, offsetof( OpenContainModuleData, m_isBurnedDeathToUnits ) },
		{ "AllowInsideKindOf",				KindOfMaskType::parseFromINI, nullptr, offsetof( OpenContainModuleData, m_allowInsideKindOf ) },
		{ "ForbidInsideKindOf",				KindOfMaskType::parseFromINI, nullptr, offsetof( OpenContainModuleData, m_forbidInsideKindOf ) },
		{ "PassengersAllowedToFire",	INI::parseBool, nullptr, offsetof( OpenContainModuleData, m_passengersAllowedToFire ) },
		{ "PassengersInTurret",				INI::parseBool, nullptr, offsetof( OpenContainModuleData, m_passengersInTurret ) },
		{ "NumberOfExitPaths",				INI::parseInt, nullptr, offsetof( OpenContainModuleData, m_numberOfExitPaths ) },
		{ "DoorOpenTime",							INI::parseDurationUnsignedInt, nullptr, offsetof( OpenContainModuleData, m_doorOpenTime ) },
 		{ "WeaponBonusPassedToPassengers", INI::parseBool,	nullptr, offsetof( OpenContainModuleData, m_weaponBonusPassedToPassengers ) },
 		{ "AllowAlliesInside",				INI::parseBool,	nullptr, offsetof( OpenContainModuleData, m_allowAlliesInside ) },
 		{ "AllowEnemiesInside",				INI::parseBool,	nullptr, offsetof( OpenContainModuleData, m_allowEnemiesInside ) },
 		{ "AllowNeutralInside",				INI::parseBool,	nullptr, offsetof( OpenContainModuleData, m_allowNeutralInside ) },
		{ nullptr, nullptr, nullptr, 0 }
	};
  p.add(dataFieldParse);
	p.add(DieMuxData::getFieldParse(), offsetof( OpenContainModuleData, m_dieMuxData ));

}

///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
OpenContain::OpenContain( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{

	// initialize our lists
	m_containList.clear();
	m_objectEnterExitInfo.clear();
	m_playerEnteredMask = 0;
	m_lastUnloadSoundFrame = 0;
	m_lastLoadSoundFrame = 0;
	m_containListSize = 0;
	m_stealthUnitsContained = 0;
	m_heroUnitsContained = 0;
	m_doorCloseCountdown = 0;

	m_rallyPoint.zero();
	m_rallyPointExists = FALSE;
	m_conditionState.clear();
	m_firePointStart = -1;
	m_firePointNext = 0;
	m_firePointSize = 0;
	m_noFirePointsInArt = false;
	m_whichExitPath = 1;
	m_loadSoundsEnabled = TRUE;

  m_passengerAllowedToFire = getOpenContainModuleData()->m_passengersAllowedToFire;
  // overridable by setPass...()  in the parent interface (for use by upgrade module)

	for( Int i = 0; i < MAX_FIRE_POINTS; i++ )
	{
		m_firePoints[ i ].Make_Identity();
	}

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
Int OpenContain::getContainMax() const
{
	const OpenContainModuleData *modData = getOpenContainModuleData();

	return modData->m_containMax;

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
OpenContain::~OpenContain()
{

	// sanity, the system should be cleaning these up itself if all is going well
	DEBUG_ASSERTCRASH( m_containList.empty(),
										 ("OpenContain %s: destroying a container that still has items in it!",
										  getObject()->getTemplate()->getName().str() ) );

	// sanity
	DEBUG_ASSERTCRASH( m_xferContainIDList.empty(),
										 ("OpenContain %s: m_xferContainIDList is not empty but should be",
											getObject()->getTemplate()->getName().str() ) );

#if RETAIL_COMPATIBLE_CRC
	// TheSuperHackers @bugfix Caball009 18/08/2026 Due to a potential use-after-free bug that cannot be fixed
	// with retail compatibility, it's desirable to be able to check if the contain list is empty after its destruction.
	// Clear the list explicitly to reset the list size.
	m_containList.clear();
#endif
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
// our object changed position... react as appropriate.
void OpenContain::containReactToTransformChange()
{
	// Our transform changed, which means our bones moved, and we keep people positioned on bones.
	redeployOccupants();
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
UpdateSleepTime OpenContain::update()
{
	m_playerEnteredMask = 0;

	// we need to monitor changes in the art and position
	monitorConditionChanges();

	if( m_doorCloseCountdown )
	{
		/// @todo srj -- for now, OpenContain assumes at most one door
		--m_doorCloseCountdown;
		if( m_doorCloseCountdown == 0 )
			getObject()->clearAndSetModelConditionState( MODELCONDITION_DOOR_1_OPENING, MODELCONDITION_DOOR_1_CLOSING );
	}

	if (!m_objectEnterExitInfo.empty())
		pruneDeadWanters();

	return UPDATE_SLEEP_NONE;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void OpenContain::addOrRemoveObjFromWorld(Object* obj, Bool add)
{

	//
	// it's expensive to add and remove structures from the pathfinder, but we really shouldn't
	// be doing it anyway as it doesn't make much sense to contain a structure, but we'll
	// check for it here and print a warning
	//
	if( obj->isKindOf( KINDOF_STRUCTURE ) )
		DEBUG_LOG(( "WARNING: Containing/Removing structures like '%s' is potentially a very expensive and slow operation",
								obj->getTemplate()->getName().str() ));


	if (add)
	{
		ThePartitionManager->registerObject( obj );

		if( obj->getDrawable() )
		{
			obj->getDrawable()->setDrawableHidden( false );
		}

		// add object to pathfind map
		TheAI->pathfinder()->addObjectToPathfindMap( obj );

	}
	else
	{
		DEBUG_ASSERTCRASH(!getObject()->isEffectivelyDead() && !getObject()->isDestroyed(),
			("object shouldn't become an occupant of a dead or destroyed container object"));

		// remove object from its group (if any)
		obj->leaveGroup();

		// remove rider from partition manager
		ThePartitionManager->unRegisterObject( obj );

		// hide the drawable associated with rider
		if( obj->getDrawable() )
			obj->getDrawable()->setDrawableHidden( true );

		// remove object from pathfind map
		TheAI->pathfinder()->removeObjectFromPathfindMap( obj );

	}

	// if we're a non-enclosing container (eg, parachute), and we're inside an
	// enclosing container (eg, b52), we need to be able to show/hide the contained riders...
	if( obj->getContain() )
	{
		const ContainedItemsList* items = obj->getContain()->getContainedItemsList();
		if (items)
		{
			for(ContainedItemsList::const_iterator it = items->begin(); it != items->end(); ++it)
			{
				if( !obj->getContain()->isEnclosingContainerFor(*it) )
					addOrRemoveObjFromWorld(*it, add);
			}
		}
	}
}

//-------------------------------------------------------------------------------------------------
/** Add 'rider' to the m_containList of objects in this module.
	* This will trigger an onContaining event for the object that this module
	* is a part of and an onContainedBy event for the object being contained */
//-------------------------------------------------------------------------------------------------
void OpenContain::addToContain( Object *rider )
{
	if( getObject()->checkAndDetonateBoobyTrap(rider) )
	{
		// Whoops, I was mined.  Cancel if I (or they) am now dead.
		if( getObject()->isEffectivelyDead() || rider->isEffectivelyDead() )
		{
			return;
		}
	}

	// sanity
	if( rider == nullptr )
		return;

#if !RETAIL_COMPATIBLE_CRC
	// TheSuperHackers @bugfix Caball009 25/05/2026 Ensure the occupant is only added to a non-destroyed
	// container to avoid an invalid state and use-after-free bugs when accessing the contained by pointer.
	if (getObject()->isDestroyed())
	{
		DEBUG_CRASH(("'%s' is about to be added to '%s', which is destroyed",
			rider->getTemplate()->getName().str(), getObject()->getTemplate()->getName().str()));
		return;
	}
#endif

	// TheSuperHackers @bugfix Stubbjax 06/02/2026 Ensure the rider is not destroyed to prevent a
	// likely crash if it enters the container on the same frame. If this occurs with an unpatched
	// client present in a match, the game has a small chance to mismatch.
	if (rider->isDestroyed())
	{
		DEBUG_CRASH(("'%s', which is destroyed, is about to be added to '%s'",
			rider->getTemplate()->getName().str(), getObject()->getTemplate()->getName().str()));
		return;
	}

	Drawable *riderDraw = rider->getDrawable();
	Bool wasSelected = FALSE;
	if( riderDraw && riderDraw->isSelected() )
	{
		wasSelected = TRUE;
	}

#if defined(RTS_DEBUG)
	if( !isValidContainerFor( rider, false ) )
	{
		Object *reportObject = rider;
		if( rider->getContain() )
		{
			//Report the first thing inside it!
			const ContainedItemsList *items = rider->getContain()->getContainedItemsList();
			if( items )
			{
				reportObject = *items->begin();
			}
		}
		DEBUG_CRASH( ("OpenContain::addToContain() - Object %s not valid for container %s!", reportObject?reportObject->getTemplate()->getName().str():"null", getObject()->getTemplate()->getName().str() ) );
	}
#endif

	// this object cannot be contained by this module if it is already contained in something
	if( rider->getContainedBy() != nullptr )
	{

		DEBUG_LOG(( "'%s' is trying to contain '%s', but '%s' is already contained by '%s'",
								getObject()->getTemplate()->getName().str(),
								rider->getTemplate()->getName().str(),
								rider->getTemplate()->getName().str(),
								rider->getContainedBy()->getTemplate()->getName().str() ));
		return;

	}

	// Which list to physically add to needs to be overridable
	addToContainList(rider);

	m_playerEnteredMask = rider->getControllingPlayer()->getPlayerMask();

	if (isEnclosingContainerFor( rider ))
	{
		addOrRemoveObjFromWorld(rider, false);
	}

#if RETAIL_COMPATIBLE_CRC
	// ensure our occupants are positioned correctly.
	// TheSuperHackers @info Moving this call elsewhere will cause retail mismatch.
	redeployOccupants();
#endif

	// trigger an onContaining event for the object that just "ate" something
	if( getObject()->getContain() )
	{
		getObject()->getContain()->onContaining( rider, wasSelected );
	}

	// ensure our occupants are positioned correctly.
	// TheSuperHackers @fix Skyaero 10/07/2025 Now (re)deploys the occupants after the garrison points
	// had a chance to initialize with prior call to onContaining(). No user facing bug was observed.
#if !RETAIL_COMPATIBLE_CRC
	redeployOccupants();
#endif

	// trigger an onContainedBy event for the object that just got "eaten" by us
	rider->onContainedBy( getObject() );

	doLoadSound();

}

//-------------------------------------------------------------------------------------------------
Bool OpenContain::isContained( const Object *obj ) const
{
	return obj->getContainedBy() == getObject();
}

//-------------------------------------------------------------------------------------------------
void OpenContain::addToContainList( Object *rider )
{
	m_containList.push_back(rider);
	m_containListSize++;
	if( rider->isKindOf( KINDOF_STEALTH_GARRISON ) )
	{
		m_stealthUnitsContained++;
	}
	if( rider->isKindOf( KINDOF_HERO ) )
	{
		m_heroUnitsContained++;
	}
}

//-------------------------------------------------------------------------------------------------
/** Remove 'rider' from the m_containList of objects in this module.
	* This will trigger an onRemoving event for the object that this module
	* is a part of and an onRemovedFrom event for the object being removed */
//-------------------------------------------------------------------------------------------------
void OpenContain::removeFromContain( Object *rider, Bool exposeStealthUnits )
{

	// sanity
	if( rider == nullptr )
		return;

	//
	// we can only remove this object from the contains list of this module if
	// it is actually contained by this module
	//
	Object *containedBy = rider->getContainedBy();
	if( containedBy != getObject() )
	{

		DEBUG_LOG(( "'%s' is trying to un-contain '%s', but '%s' is really contained by '%s'",
								getObject()->getTemplate()->getName().str(),
								rider->getTemplate()->getName().str(),
								rider->getTemplate()->getName().str(),
								containedBy ? containedBy->getTemplate()->getName().str() : "Nothing" ));
		return;

	}

#if RETAIL_COMPATIBLE_CRC
	// TheSuperHackers @bugfix Caball009 18/08/2026 Due to a potential use-after-free bug that cannot be fixed
	// with retail compatibility, the 'contained by' pointer of this object may point to an already destroyed object.
	// Check the list size before executing the find operation below, otherwise the game crashes if the list
	// was already destructed.
	if (m_containList.empty())
		return;
#endif

	ContainedItemsList::iterator it = std::find(m_containList.begin(), m_containList.end(), rider);
	if (it != m_containList.end())
	{
		// note that this invalidates the iterator!
		removeFromContainViaIterator( it, exposeStealthUnits );
	}

}

//-------------------------------------------------------------------------------------------------
/** Remove all contained objects from the contained list */
//-------------------------------------------------------------------------------------------------
void OpenContain::removeAllContained( Bool exposeStealthUnits )
{
	ContainedItemsList::iterator it;

 	while ((it = m_containList.begin()) != m_containList.end())
	{

 		// note that this invalidates the iterator!
 		removeFromContainViaIterator( it, exposeStealthUnits );

	}

}

//-------------------------------------------------------------------------------------------------
/** Kill all contained objects in the contained list */
//-------------------------------------------------------------------------------------------------
void OpenContain::killAllContained()
{
	// TheSuperHackers @bugfix Caball009 11/03/2026 The contain list must be updated while iterating over it,
	// because e.g. garrisoned infantry relies on that behavior for the team ownership of civilian buildings.

	ContainedItemsList::iterator it = m_containList.begin();
	while ( it != m_containList.end() )
	{
		Object *rider = *it;

		DEBUG_ASSERTCRASH( rider, ("Contain list must not contain null element"));
		if ( rider )
		{
			m_containList.erase(it);
			--m_containListSize;

			onRemoving( rider );
			rider->onRemovedFrom( getObject() );
			rider->kill();

			// After kill, the iterator may or may not be invalidated and the list may or may not be empty.
			// Set the iterator to the beginning of the list.
			it = m_containList.begin();
		}
		else
		{
			++it;
		}
	}

	DEBUG_ASSERTCRASH(m_containList.empty(), ("killAllContained should have emptied the contain list"));

	m_containList.clear();
	m_containListSize = 0;
}

//--------------------------------------------------------------------------------------------------------
/** Force all contained objects in the contained list to exit, and kick them in the pants on the way out*/
//--------------------------------------------------------------------------------------------------------
void OpenContain::harmAndForceExitAllContained( DamageInfo *info )
{
	ContainedItemsList::iterator it = m_containList.begin();

 	while ( it != m_containList.end() )
	{
		Object *rider = *it;

		DEBUG_ASSERTCRASH( rider, ("Contain list must not contain null element"));
		if ( rider )
		{
		  removeFromContain( rider, true );
		  rider->attemptDamage( info );
		}

		//Kris: Patch 1.03 -- Crash fix when neutral bunker on Alpine Assault is occupied with 10 demo general
		//infantry units with the suicide upgrade and US stealth fighters with bunker busters kill the guys inside.
		//Causes recursive damage where a bunker buster destroys an infantry, the infantry explodes and blows up
		//another missile which kills everyone inside while the first missile is killing everyone. And the game blows up.
		//Fix is to reset the list.
		it = m_containList.begin();

	}


  DEBUG_ASSERTCRASH( m_containListSize == 0, ("harmAndForceExitAllContained just made a booboo, list size != zero.") );

}


//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void OpenContain::doLoadSound()
{
	//
	// play a sound for loading someone into a building
	//
	if( m_loadSoundsEnabled )
	{
		UnsignedInt now = TheGameLogic->getFrame();
		if( now != m_lastLoadSoundFrame )
		{
			if (getOpenContainModuleData())
			{
				AudioEventRTS enterSound(getOpenContainModuleData()->m_enterSound);
				enterSound.setObjectID(getObject()->getID());
				TheAudio->addAudioEvent(&enterSound);
			}
			// save this frame as the last time we did this sound
			m_lastLoadSoundFrame = now;
		}
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void OpenContain::doUnloadSound()
{
	//
	// play a sound for unloading someone from a building ... but don't play more
	// than one per frame (we can do meta unload all commands)
	//
	UnsignedInt now = TheGameLogic->getFrame();
	if( now != m_lastUnloadSoundFrame )
	{
		if (getOpenContainModuleData())
		{
			AudioEventRTS exitSound(getOpenContainModuleData()->m_exitSound);
			exitSound.setObjectID(getObject()->getID());

			TheAudio->addAudioEvent(&exitSound);
		}
		// save this frame as the last time we did this sound
		m_lastUnloadSoundFrame = now;
	}
}

//-------------------------------------------------------------------------------------------------
/** Iterate the contained list and call the callback on each of the objects */
//-------------------------------------------------------------------------------------------------
void OpenContain::iterateContained( ContainIterateFunc func, void *userData, Bool reverse )
{
	if (reverse)
	{
		// note that this has to be smart enough to handle items in the list being deleted
		// via the callback function.
		for(ContainedItemsList::reverse_iterator it = m_containList.rbegin(); it != m_containList.rend(); )
		{
			// save the rider...
			Object* rider = *it;

			// incr the iterator BEFORE calling the func (if the func removes the rider,
			// the iterator becomes invalid)
			++it;

			// call it
			(*func)( rider, userData );
		}
	}
	else
	{
		// note that this has to be smart enough to handle items in the list being deleted
		// via the callback function.
		for(ContainedItemsList::iterator it = m_containList.begin(); it != m_containList.end(); )
		{
			// save the rider...
			Object* rider = *it;

			// incr the iterator BEFORE calling the func (if the func removes the rider,
			// the iterator becomes invalid)
			++it;

			// call it
			(*func)( rider, userData );
		}
  }
}

//-------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
Object* OpenContain::getClosestRider( const Coord3D *pos )
{
	Object *closest = nullptr;
	Real closestDistance;

	for(ContainedItemsList::const_iterator it = m_containList.begin(); it != m_containList.end(); ++it)
	{
    Object *rider = *it;

    if (rider)
    {
      Real distance = ThePartitionManager->getDistanceSquared( rider, pos, FROM_CENTER_2D );
	    if( !closest || closestDistance > distance )
	    {
		    closest = rider;
		    closestDistance = distance;
	    }
    }

  }

   return closest; //Could be null!
}





//-------------------------------------------------------------------------------------------------
struct DropData
{
	Real minRadius;
	Real maxRadius;
	Object *container;
};

///////////////////////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTIONS //////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
/** Remove an object from the containment of this module given the item
	* to remove and trigger the proper callback events */
//-------------------------------------------------------------------------------------------------
void OpenContain::removeFromContainViaIterator( ContainedItemsList::iterator it, Bool exposeStealthUnits )
{

/*
	#ifdef RTS_DEBUG
		TheInGameUI->message( L"'%S(%d)' no longer contains '%S(%d)'",
													getObject()->getTemplate()->getName().str(),
													getObject()->getID(),
													itemToRemove->m_object->getTemplate()->getName().str(),
													itemToRemove->m_object->getID() );
	#endif
*/
	Object *rider = *it;

	// remove item from list
	m_containList.erase(it);
	m_containListSize--;
	if( rider->isKindOf( KINDOF_STEALTH_GARRISON ) )
	{
		DEBUG_ASSERTCRASH( m_stealthUnitsContained > 0, ("OpenContain::removeFromContainViaIterator - Removing stealth unit but stealth count is %d", m_stealthUnitsContained) );
		m_stealthUnitsContained--;
		if( exposeStealthUnits )
		{
			StealthUpdate* stealth = rider->getStealth();
			if( stealth )
			{
				stealth->markAsDetected();
			}
		}
	}
	if( rider->isKindOf( KINDOF_HERO ) )
	{
		DEBUG_ASSERTCRASH( m_heroUnitsContained > 0, ("OpenContain::removeFromContainViaIterator - Removing hero but hero count is %d", m_heroUnitsContained) );
		m_heroUnitsContained--;
	}


	if (isEnclosingContainerFor( rider ))
	{
		addOrRemoveObjFromWorld(rider, true);
  	rider->setPosition( getObject()->getPosition() );
        // if we are not enclosed, then just walk away from where we "are."

	}

	/// place the object in the world at position of the container m_object
	rider->setLayer( getObject()->getLayer() );

	doUnloadSound();

	// trigger an onRemoving event for 'm_object' no longer containing 'itemToRemove->m_object'
	DEBUG_ASSERTCRASH(getObject()->getContain() == this, ("hmm, wrong container"));
	if( getObject()->getContain() )
	{
		getObject()->getContain()->onRemoving( rider );
	}

	// trigger an onRemovedFrom event for 'remove'
	DEBUG_ASSERTCRASH(getObject()->getContain() == this, ("hmm, wrong container 2"));
	rider->onRemovedFrom( getObject() );

}

//-------------------------------------------------------------------------------------------------
void OpenContain::scatterToNearbyPosition(Object* rider)
{
	Object *theContainer = getObject();

	//
	// for now we will just set the position of the object that is being removed from us
	// at a random angle away from our center out some distance
	//

	//
	// pick an angle that is in the view of the current camera position so that
	// the thing will come out "toward" the player and they can see it
	// NOPE, can't do that ... all players screen angles will be different, unless
	// we maintain the angle of each players screen in the player structure or something
	//
	Real angle = GameLogicRandomValueReal( 0.0f, 2.0f * PI );
//	angle = TheTacticalView->getAngle();
//	angle -= GameLogicRandomValueReal( PI / 3.0f, 2.0f * (PI / 3.0F) );

	Real minRadius = theContainer->getGeometryInfo().getBoundingCircleRadius();
	Real maxRadius = minRadius + minRadius / 2.0f;
	const Coord3D *containerPos = theContainer->getPosition();
	Real dist = GameLogicRandomValueReal( minRadius, maxRadius );

	Coord3D pos;
	pos.x = dist * Cos( angle ) + containerPos->x;
	pos.y = dist * Sin( angle ) + containerPos->y;
	pos.z = TheTerrainLogic->getLayerHeight( pos.x, pos.y, theContainer->getLayer() );

	// set orientation
	rider->setOrientation( angle );

	AIUpdateInterface *ai = rider->getAI();
	if( ai )
	{
		// set position of the object at center of building and move them toward pos
		rider->setPosition( theContainer->getPosition() );
		ai->ignoreObstacle(theContainer);
#if !RETAIL_COMPATIBLE_CRC
		ai->friend_setGoalObject(nullptr);
#endif
		ai->aiMoveToPosition( &pos, CMD_FROM_AI );

	}
	else
	{

		// no ai, just set position at the target pos
		rider->setPosition( &pos );

	}
}

//-------------------------------------------------------------------------------------------------
void OpenContain::onContaining( Object *rider, Bool wasSelected )
{
	// Play audio
	if( m_loadSoundsEnabled )
	{
		AudioEventRTS enterSound = *getObject()->getTemplate()->getSoundEnter();
		enterSound.setObjectID(getObject()->getID());
		TheAudio->addAudioEvent(&enterSound);
	}
}

//-------------------------------------------------------------------------------------------------
void OpenContain::onRemoving( Object *rider)
{
	// Play audio
	AudioEventRTS exitSound = *getObject()->getTemplate()->getSoundExit();
	exitSound.setObjectID(getObject()->getID());
	TheAudio->addAudioEvent(&exitSound);

	if (rider) {
		// This is a misnomer, but it makes it clearer for the user.
		AudioEventRTS fallingSound = *rider->getTemplate()->getSoundFalling();
		fallingSound.setObjectID(rider->getID());
		TheAudio->addAudioEvent(&fallingSound);
	}
}

//-------------------------------------------------------------------------------------------------
Real OpenContain::getContainedItemsMass() const
{
	/// @todo srj -- may want to cache this information.
	Real mass = 0;
	for(ContainedItemsList::const_iterator it = m_containList.begin(); it != m_containList.end(); ++it)
	{
		PhysicsBehavior* phys = (*it)->getPhysics();
		if (phys)
			mass += phys->getMass();
	}
	return mass;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void OpenContain::onCollide( Object *other, const Coord3D *loc, const Coord3D *normal )
{
	// colliding with nothing? we don't care.
	if( other == nullptr )
		return;

	// ok, step two: only contain stuff that wants us to contain it.
	// (it would suck to have accidental collisions contain things...)

	// must be an AI object....
	AIUpdateInterface *ai = other->getAI();
	if (ai == nullptr)
		return;

	// ...and must be trying to enter our object.
	// (huh huh, he said "enter")
	if (ai->getEnterTarget() != getObject())
		return;

	// last-minute change: don't allow units from multiple (different) players to occupy the same
	// unit. so eject everyone else if they aren't controlled by the same player. (srj)
	for( ContainedItemsList::iterator it = m_containList.begin(); it != m_containList.end(); )
	{
		// save the rider...
		Object* rider = *it;

		// incr the iterator BEFORE calling the func (if the func removes the rider,
		// the iterator becomes invalid)
		++it;

		// call it
		if( rider->getControllingPlayer() != other->getControllingPlayer() )
		{
			if( rider->getAI() )
			{
				if( rider->isKindOf( KINDOF_STEALTH_GARRISON ) )
				{
					// aiExit is needed to walk away from the building well, but it doesn't take the Unstealth flag
          StealthUpdate* stealth = rider->getStealth();
         	if( stealth )
					{
						stealth->markAsDetected();
					}
				}
				rider->getAI()->aiExit( getObject(), CMD_FROM_AI );
			}
			else
				removeFromContain( rider, TRUE );
		}
	}


	// finally, we must have space to contain it.
	if( !isValidContainerFor( other, TRUE ) )
		return;

	addToContain(other);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void OpenContain::onDelete()	///< Last possible moment cleanup
{
	// This uses my literal list, and not the gettor, because we don't want to get redirected some place fancy.
	for(ContainedItemsList::iterator it = m_containList.begin(); it != m_containList.end(); )
	{
		Object* rider = *it;
		++it;
		TheGameLogic->destroyObject( rider );
	}
}

//-------------------------------------------------------------------------------------------------
/** The die callback. */
//-------------------------------------------------------------------------------------------------
void OpenContain::onDie( const DamageInfo * damageInfo )
{
	if (!getOpenContainModuleData()->m_dieMuxData.isDieApplicable(getObject(), damageInfo))
		return;

#if !RETAIL_COMPATIBLE_CRC
	killRidersWhoAreNotFreeToExit();
#endif

	//Check to see if we are going to inflict damage on contained units.
	if( getDamagePercentageToUnits() > 0 )
	{
		//Cycle through the units and apply damage to them!
		processDamageToContained(getDamagePercentageToUnits());
	}

#if RETAIL_COMPATIBLE_CRC
	killRidersWhoAreNotFreeToExit();
#endif

	// Leaving this commented out to show it can't work.  We are about to die, so they will have zero
	// chance to hit an exitState::Update.  At least we would clean them up in onDelete.
//	orderAllPassengersToExit( CMD_FROM_AI, FALSE );
	removeAllContained();
}

// ------------------------------------------------------------------------------------------------
/** Check to see if we are a valid container for 'obj' */
// ------------------------------------------------------------------------------------------------
Bool OpenContain::isValidContainerFor(const Object* obj, Bool checkCapacity) const
{
	const Object *us = getObject();
	const OpenContainModuleData *modData = getOpenContainModuleData();

	// if we have any kind of masks set then we must make that check
	if (obj->isAnyKindOf( modData->m_allowInsideKindOf ) == FALSE ||
			obj->isAnyKindOf( modData->m_forbidInsideKindOf ) == TRUE)
	{
		return false;
	}

 	//
 	// check relationship, note that this behavior is defined as the relation between
 	// 'obj' and the container 'us', and not the reverse
 	//
 	Bool relationshipRestricted = FALSE;
 	Relationship r = obj->getRelationship( us );
 	switch( r )
 	{
 		case ALLIES:
 			if( modData->m_allowAlliesInside == FALSE )
 				relationshipRestricted = TRUE;
 			break;

 		case ENEMIES:
 			if( modData->m_allowEnemiesInside == FALSE )
 				relationshipRestricted = TRUE;
 			break;

 		case NEUTRAL:
 			if( modData->m_allowNeutralInside == FALSE )
 				relationshipRestricted = TRUE;
 			break;

 		default:
 			DEBUG_CRASH(( "isValidContainerFor: Undefined relationship (%d) between '%s' and '%s'",
 										r, getObject()->getTemplate()->getName().str(),
 										obj->getTemplate()->getName().str() ));
 			return FALSE;

 	}
 	if( relationshipRestricted == TRUE )
 		return FALSE;

	// all is well
	return true;

}

// ------------------------------------------------------------------------------------------------
/**
	This new call is going to replace all game callings of removeFromContain.
	Consider rFC a 'system' call, while this is a 'game' call.  rFC will pull
	from the contained list and just plop the guy in the world, this function
	will then override that location with cool game stuff.

	Exit and Evacuate can fail, removeAllContain and removeFromContain cannot.
*/
void OpenContain::exitObjectViaDoor( Object *exitObj, ExitDoorType exitDoor )
{
	DEBUG_ASSERTCRASH(exitDoor == DOOR_1, ("multiple exit doors not supported here"));

	removeFromContain( exitObj );

	Object *me = getObject();

	m_doorCloseCountdown = getOpenContainModuleData()->m_doorOpenTime;
	if (m_doorCloseCountdown)
	{
		// srj sez: only diddle the doors if this countdown is nonzero.
		// this allows us to prevent this module from messing with the doors
		// at all, which is required for DeliverPayloadAIUpdate.
		/// @todo srj -- for now, OpenContain assumes at most one door
		me->clearAndSetModelConditionState( MODELCONDITION_DOOR_1_CLOSING, MODELCONDITION_DOOR_1_OPENING );
	}

	Int numberExits = getOpenContainModuleData()->m_numberOfExitPaths;
	if( numberExits > 0 )
	{
		// We have ExitStart/End specified in art to use when we kick people out.  If >1, then
		// we have many and we need to decide which one to use.
		AsciiString startBone("ExitStart");
		AsciiString endBone("ExitEnd");
		Coord3D startPosition;
		Coord3D endPosition;
		if( numberExits > 1 )
		{
			char suffix[8];
			itoa(m_whichExitPath, suffix, 10);
			if( m_whichExitPath < 10 )
			{
				startBone.concat('0');
				endBone.concat('0');
			}

			m_whichExitPath = (m_whichExitPath % numberExits) + 1;// To cycle from 1 to n

			startBone.concat(suffix);
			endBone.concat(suffix);
		}
		me->getSingleLogicalBonePosition( startBone.str(), &startPosition, nullptr );
		me->getSingleLogicalBonePosition( endBone.str(), &endPosition, nullptr );

		//startPosition.x = startPosition.y = 0;
		Real exitAngle = me->getOrientation();
		PhysicsBehavior *physics = exitObj->getPhysics();
		// Can fall problem - When units exit from planes, they are airborne for a while as they drop.
		// Airborne units dont' do pathfinding, and this is fine.
		// However, amphibious transports unload 3 feet off the ground.  This is enough to trigger the
		// airborne flag, and they don't pathfind.  So they all unload at the same exact point, and life
		// is ugly.  So temporarily, we turn off the allowToFall flag, so they aren't considered airborne,
		// and their final destination (adjustToPossibleDestination) considers them as pathfinding, so they
		// don't stack up.  Then we turn the flag back, so if they are parachuting, they can fly down
		//  on their parachute.  jba.
		Bool canFall = false;
		if (physics)
		{
			canFall = physics->getAllowToFall();
		}
		exitObj->setPosition( &startPosition );
		exitObj->setOrientation( exitAngle );

		// Per JohnA: We need to set our layer to match our transports layer, or we'll try to pick a spot
		// on the ground.
		exitObj->setLayer( me->getLayer() );
		///< @todo This really should be automatically wrapped up in an activation sequence	for objects in general
		// tell the AI about it
		AIUpdateInterface *ai = exitObj->getAI();
		AIUpdateInterface *myAi = me->getAI();
		TheAI->pathfinder()->addObjectToPathfindMap( exitObj );
		if (ai)
		{
			if (myAi && myAi->isIdle() && me->isKindOf(KINDOF_VEHICLE)) {
				TheAI->pathfinder()->removeUnitFromPathfindMap(me);
				TheAI->pathfinder()->updatePos(me, me->getPosition());
				TheAI->pathfinder()->updateGoal(me, me->getPosition(), TheTerrainLogic->getLayerForDestination(me->getPosition()));
			}
			ai->ignoreObstacle(nullptr);
			// The units often come out at the same position, and need to ignore collisions briefly
			// as they move out.  jba.
			ai->setIgnoreCollisionTime(LOGICFRAMES_PER_SECOND);
			TheAI->pathfinder()->adjustToPossibleDestination(exitObj, ai->getLocomotorSet(), &endPosition);
		}
		std::vector<Coord3D> exitPath;
		exitPath.push_back(endPosition);
		exitPath.push_back(endPosition); // Do it twice, in case units stack up due to brief flying.  jba.
		if (const Coord3D *rallyPoint = getRallyPoint()) {
			exitPath.push_back(*rallyPoint);
		}

		if( ai )
		{
			if (physics)
			{
				physics->setAllowToFall(false);
			}

			ai->aiFollowPath( &exitPath, me, CMD_FROM_AI );

			TheAI->pathfinder()->updateGoal(exitObj, &endPosition, TheTerrainLogic->getLayerForDestination(&endPosition));
		}
		if (physics)
		{
			physics->setAllowToFall(canFall);
		}
	}
	else
	{
		///< @todo This really should be automatically wrapped up in an activation sequence	for objects in general
		// tell the AI about it
		TheAI->pathfinder()->addObjectToPathfindMap( exitObj );
	}
}
// ------------------------------------------------------------------------------------------------
/**
	This call is used to evacuate units from a tunnel network via a persistent state (guard tunnel networ).
	The main difference between this and exitObjectViaDoor is that this exit doesn't change the current
	ai state, so the guard state doesn't get blown away. jba.
*/
void OpenContain::exitObjectInAHurry( Object *exitObj )
{
	removeFromContain( exitObj );

	Object *me = getObject();

	m_doorCloseCountdown = getOpenContainModuleData()->m_doorOpenTime;
	if (m_doorCloseCountdown)
	{
		// srj sez: only diddle the doors if this countdown is nonzero.
		// this allows us to prevent this module from messing with the doors
		// at all, which is required for DeliverPayloadAIUpdate.
		/// @todo srj -- for now, OpenContain assumes at most one door
		me->clearAndSetModelConditionState( MODELCONDITION_DOOR_1_CLOSING, MODELCONDITION_DOOR_1_OPENING );
	}

	Int numberExits = getOpenContainModuleData()->m_numberOfExitPaths;
	if( numberExits > 0 )
	{
		// We have ExitStart/End specified in art to use when we kick people out.  If >1, then
		// we have many and we need to decide which one to use.
		AsciiString startBone("ExitStart");
		AsciiString endBone("ExitEnd");
		Coord3D startPosition;
		Coord3D endPosition;
		if( numberExits > 1 )
		{
			char suffix[8];
			itoa(m_whichExitPath, suffix, 10);
			if( m_whichExitPath < 10 )
			{
				startBone.concat('0');
				endBone.concat('0');
			}

			m_whichExitPath = (m_whichExitPath % numberExits) + 1;// To cycle from 1 to n

			startBone.concat(suffix);
			endBone.concat(suffix);
		}
		me->getSingleLogicalBonePosition( startBone.str(), &startPosition, nullptr );
		me->getSingleLogicalBonePosition( endBone.str(), &endPosition, nullptr );

		//startPosition.x = startPosition.y = 0;
		Real exitAngle = me->getOrientation();
		exitObj->setPosition( &startPosition );
		exitObj->setOrientation( exitAngle );

		// Per JohnA: We need to set our layer to match our transports layer, or we'll try to pick a spot
		// on the ground.
		exitObj->setLayer( me->getLayer() );
		std::vector<Coord3D> exitPath;
		exitPath.push_back(endPosition);
		AIUpdateInterface *ai = exitObj->getAI();
		AIUpdateInterface *myAi = me->getAI();
		TheAI->pathfinder()->addObjectToPathfindMap( exitObj );
		if (ai)
		{
			if (myAi && myAi->isIdle() && me->isKindOf(KINDOF_VEHICLE)) {
				TheAI->pathfinder()->removeUnitFromPathfindMap(me);
				TheAI->pathfinder()->updatePos(me, me->getPosition());
				TheAI->pathfinder()->updateGoal(me, me->getPosition(), TheTerrainLogic->getLayerForDestination(me->getPosition()));
			}
			ai->ignoreObstacle(nullptr);
			// The units often come out at the same position, and need to ignore collisions briefly
			// as they move out.  jba.
			ai->setIgnoreCollisionTime(LOGICFRAMES_PER_SECOND);
			TheAI->pathfinder()->adjustToPossibleDestination(exitObj, ai->getLocomotorSet(), &endPosition);
		}
		exitPath.push_back(endPosition);
		if (m_rallyPointExists) {
			exitPath.push_back(m_rallyPoint);
		}

		if( ai )
		{
			ai->doQuickExit( &exitPath );

			TheAI->pathfinder()->updateGoal(exitObj, &endPosition, TheTerrainLogic->getLayerForDestination(&endPosition));
		}
	}
	else
	{
		///< @todo This really should be automatically wrapped up in an activation sequence	for objects in general
		// tell the AI about it
		TheAI->pathfinder()->addObjectToPathfindMap( exitObj );
	}
}


//-------------------------------------------------------------------------------------------------
Bool OpenContain::isPassengerAllowedToFire( ObjectID id ) const
{
//	const OpenContainModuleData *modData = getOpenContainModuleData();
  //this flag is owned by opencontain, now, so that the upgrade can override the template data
  //M Lorenzen, 5/6/03
	if( ! m_passengerAllowedToFire )
		return FALSE;// Just no, no matter what.

	// If we are ourselves contained, our passengers need to check with them if they get past us
	if( getObject()->getContainedBy() != nullptr )
		return getObject()->getContainedBy()->getContain()->isPassengerAllowedToFire();

	return TRUE;// We say yes, and we are not inside something.
}

//-------------------------------------------------------------------------------------------------
/** Check to see if our internal knowledge of the artwork matches the current state of
	* the model being displayed.  When changes occur (from changing the time of day or going
	* to a new damage state, we will want to redeploy all our occupants to be at new fire
	* points that are reflected in the new artwork */
//-------------------------------------------------------------------------------------------------
void OpenContain::monitorConditionChanges()
{
	Drawable *draw = getObject()->getDrawable();
	Bool stateChanged = false;
	ModelConditionFlags currCondition;

	if( draw )
	{
		currCondition = draw->getModelConditionFlags();
		if(currCondition != m_conditionState )
			stateChanged = TRUE;
	}

	// on a state change we should redeploy our occupants
	if( stateChanged )
	{
		// just yank all objects from the fire points and redeploy
		redeployOccupants();
		// record this as our current and up to date state
		m_conditionState = currCondition;
	}

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void OpenContain::redeployOccupants()
{

	//
	// because the state has changed, we will must give the deploy logic the opportunity
	// to look for a new set of bones in the art ... so if we had them flagged as not
	// having any before (for optimization reasons) we want to check at least one more
	// time again
	//
	m_noFirePointsInArt = false;
	m_firePointStart = -1;
	m_firePointNext = 0;
	m_firePointSize = 0;				// 0 here will cause a re-lookup in the new art

	//
	// redeploy the occupants, note that we're starting at the last occupant and going
	// backwards to the start ... that reflects the order in which they actually went
	// into the building since all list building constructs at the head
	//
	for (ContainedItemsList::const_reverse_iterator it = getContainList().rbegin(); it != getContainList().rend(); ++it)
	{
		putObjAtNextFirePoint( *it );
	}

}

//-------------------------------------------------------------------------------------------------
/** Place the object at the 3D position of the next fire point to use */
//-------------------------------------------------------------------------------------------------
void OpenContain::putObjAtNextFirePoint( Object *obj )
{

	//
	// first, if we need to load the 3D point data from the art do so, if we've already
	// determined there is no firepoints in the current art we will avoid searching for
	// it all the time
	//
	if( m_firePointSize == 0 && m_noFirePointsInArt == false )
	{

		m_firePointSize = getObject()->getMultiLogicalBonePosition("FIREPOINT", MAX_FIRE_POINTS, nullptr, m_firePoints, TRUE );

		//
		// if there is still no firepoints in the art, we'll set a flag so that we don't
		// ever go through the art stuff again
		//
		if( m_firePointSize == 0 )
			m_noFirePointsInArt = TRUE;

	}

	//
	// if there are no fire points in the art we just put the object at the center
	// of the object
	//
	if( m_noFirePointsInArt == TRUE )
	{
		const Coord3D *pos = getObject()->getPosition();

		obj->setPosition( pos );
		return;

	}

	// get the position
	Matrix3D matrix;
	if( getOpenContainModuleData()->m_passengersInTurret )
	{
		// If our passengers are in our turret, we need to recompute the Matrix.
		AsciiString firepoint("FIREPOINT");
		char suffix[8];
		itoa( m_firePointNext + 1, suffix, 10 );//+1 from bone names starting at 1, not zero like my array
		if( m_firePointNext < 10 )
		{
			firepoint.concat('0');
		}
		firepoint.concat(suffix);

		getObject()->getSingleLogicalBonePositionOnTurret(TURRET_MAIN, firepoint.str(), nullptr, &matrix );
	}
	else
	{
		matrix = m_firePoints[ m_firePointNext ];
	}

	Vector3 vectorPos = matrix.Get_Translation();
	Coord3D pos;
	pos.set( vectorPos.X, vectorPos.Y, vectorPos.Z );

	// set the object position
	if( isEnclosingContainerFor( obj ) )
		obj->setPosition( &pos );
	else
		obj->setTransformMatrix( &matrix );//Only do everything if it matters

	// increment the next firepoint to use ... make sure to wrap if we need to
	m_firePointNext++;
	if( m_firePointNext >= m_firePointSize )
		m_firePointNext = 0;

}

//-------------------------------------------------------------------------------------------------
/**
	this is used for containers that must do something to allow people to enter or exit...
	eg, land (for Chinook), open door (whatever)... it's called with wants=WANTS_TO_ENTER
	when something is in the enter state, and wants=WANTS_NOTHING when the unit has
	either entered, or given up...
*/
void OpenContain::onObjectWantsToEnterOrExit(Object* obj, ObjectEnterExitType wants)
{
	if (obj == nullptr)
		return;

	ObjectID id = obj->getID();
	if (wants == WANTS_NEITHER)
	{
		ObjectEnterExitMap::iterator it = m_objectEnterExitInfo.find(id);
		if (it != m_objectEnterExitInfo.end())
			m_objectEnterExitInfo.erase(it);
	}
	else
	{
		m_objectEnterExitInfo[id] = wants;
	}
}

//-------------------------------------------------------------------------------------------------
void OpenContain::pruneDeadWanters()
{
	for (ObjectEnterExitMap::iterator it = m_objectEnterExitInfo.begin(); it != m_objectEnterExitInfo.end(); /*++it*/)
	{
		ObjectID id = (*it).first;
		Object* obj = TheGameLogic->findObjectByID(id);
		if (obj == nullptr || obj->isEffectivelyDead())
		{
			ObjectEnterExitMap::iterator tmp = it;
			++it;
			m_objectEnterExitInfo.erase(tmp);
		}
		else
		{
			++it;
		}
	}
}

//-------------------------------------------------------------------------------------------------
void OpenContain::markAllPassengersDetected()
{
	for( ContainedItemsList::iterator it = m_containList.begin(); it != m_containList.end(); )
	{
		// save the rider...
		Object* rider = *it;

		// incr the iterator BEFORE calling the func (if the func removes the rider,
		// the iterator becomes invalid)
		++it;

		// call it
		if( rider->isKindOf( KINDOF_STEALTH_GARRISON ) )
		{
			StealthUpdate* stealth = rider->getStealth();
			if( stealth )
			{
				stealth->markAsDetected();
			}
		}
	}
}

//-------------------------------------------------------------------------------------------------
void OpenContain::onSelling()
{
	// An OpenContain tells everyone to leave.
	orderAllPassengersToExit( CMD_FROM_AI, FALSE );
}

//-------------------------------------------------------------------------------------------------
void OpenContain::orderAllPassengersToExit( CommandSourceType commandSource, Bool instantly )
{
	for( ContainedItemsList::const_iterator it = getContainedItemsList()->begin(); it != getContainedItemsList()->end(); )
	{
		// save the rider...
		Object* rider = *it;

		// incr the iterator BEFORE calling the func (if the func removes the rider,
		// the iterator becomes invalid)
		++it;

		// call it
		if( rider->getAI() )
		{
			if( instantly )
			{
				rider->getAI()->aiExitInstantly( getObject(), commandSource );
			}
			else
			{
				rider->getAI()->aiExit( getObject(), commandSource );
			}
		}
	}
}


//-------------------------------------------------------------------------------------------------
void OpenContain::orderAllPassengersToIdle( CommandSourceType commandSource )
{
	for( ContainedItemsList::const_iterator it = getContainedItemsList()->begin(); it != getContainedItemsList()->end(); )
	{
		Object* rider = *it;
		++it;

		if( rider->getAI() )
		{
			rider->getAI()->aiIdle( commandSource );
		}
	}
}

//-------------------------------------------------------------------------------------------------
void OpenContain::orderAllPassengersToHackInternet( CommandSourceType commandSource )
{
	for( ContainedItemsList::const_iterator it = getContainedItemsList()->begin(); it != getContainedItemsList()->end(); )
	{
		Object* rider = *it;
		++it;

		if( rider->isKindOf( KINDOF_MONEY_HACKER ) )
		{
			AIUpdateInterface *ai = rider->getAI();
			if( ai )
			{
				rider->getAI()->aiHackInternet( commandSource );
			}
		}
	}
}

#if RETAIL_COMPATIBLE_CRC

//-------------------------------------------------------------------------------------------------
void OpenContain::processDamageToContainedInternal(Object* const* objects, size_t size, Real percentDamage)
{
	const DeathType deathType = getOpenContainModuleData()->m_isBurnedDeathToUnits ? DEATH_BURNED : DEATH_NORMAL;
	const bool killContained = percentDamage == 1.0f;

	for (size_t i = 0; i < size; ++i)
	{
		Object* object = objects[i];

		// Calculate the damage to be inflicted on each unit.
		Real damage = object->getBodyModule()->getMaxHealth() * percentDamage;

		DamageInfo damageInfo;
		damageInfo.in.m_damageType = DAMAGE_UNRESISTABLE;
		damageInfo.in.m_deathType = deathType;
		damageInfo.in.m_sourceID = getObject()->getID();
		damageInfo.in.m_amount = damage;
		object->attemptDamage( &damageInfo );

		if( !object->isEffectivelyDead() && killContained )
			object->kill(); // in case we are carrying flame proof troops we have been asked to kill

		// TheSuperHackers @info Calls to Object::attemptDamage and Object::kill may not remove
		// the occupant from the host container straight away. Instead it would be removed when the
		// Object deletion is finalized in a Game Logic update. This will lead to strange behavior
		// where the occupant will be removed after death with a delay. This behavior cannot be
		// changed without breaking retail compatibility.
	}
}

#endif // RETAIL_COMPATIBLE_CRC

//-------------------------------------------------------------------------------------------------
void OpenContain::processDamageToContained(Real percentDamage)
{
#if RETAIL_COMPATIBLE_CRC

	DEBUG_ASSERTCRASH(m_containListSize == m_containList.size(), ("contain list size doesn't match size of container"));

	// TheSuperHackers @bugfix Caball009 11/03/2026 Use a temporary copy of the contain list to iterate over,
	// because causing damage to the occupants may remove some or all elements from the list
	// while iterating over it, which may be unsafe.

	constexpr const UnsignedInt smallContainerSize = 16;
	const size_t containCount = m_containList.size();
	if (containCount < smallContainerSize)
	{
		// GeneralsX @bugfix Bender 01/04/2026 Use copied element count to avoid invalid reads in non-assert builds.
		Object* containCopy[smallContainerSize];
		std::copy(m_containList.begin(), m_containList.end(), containCopy);

		processDamageToContainedInternal(containCopy, containCount, percentDamage);
	}
	else
	{
		const std::vector<Object*> containCopy(m_containList.begin(), m_containList.end());

		processDamageToContainedInternal(&containCopy[0], containCopy.size(), percentDamage);
	}

#else

	// TheSuperHackers @bugfix xezon 05/06/2025 Temporarily empty the m_containList
	// because causing damage to the occupants may remove some or all elements from the list
	// while iterating over it, which may be unsafe.

	// Caveat: While the m_containList is empty, it will not be possible to apply damage
	// on death of a unit to another unit in the host container. If this functionality
	// is desired, then this implementation needs to be revisited.

	const DeathType deathType = getOpenContainModuleData()->m_isBurnedDeathToUnits ? DEATH_BURNED : DEATH_NORMAL;
	const bool killContained = percentDamage == 1.0f;

	ContainedItemsList list;
	m_containList.swap(list);
	m_containListSize = 0;

	ContainedItemsList::iterator it = list.begin();

	while ( it != list.end() )
	{
		Object *object = *it;

		DEBUG_ASSERTCRASH( object, ("Contain list must not contain null element") );

		// Calculate the damage to be inflicted on each unit.
		Real damage = object->getBodyModule()->getMaxHealth() * percentDamage;

		DamageInfo damageInfo;
		damageInfo.in.m_damageType = DAMAGE_UNRESISTABLE;
		damageInfo.in.m_deathType = deathType;
		damageInfo.in.m_sourceID = getObject()->getID();
		damageInfo.in.m_amount = damage;
		object->attemptDamage( &damageInfo );

		if( !object->isEffectivelyDead() && killContained )
			object->kill(); // in case we are carrying flame proof troops we have been asked to kill

		// GeneralsX @bugfix mreza0100 23/07/2026 A destroyed occupant cannot remove itself
		// while the contain list is temporarily detached. Do not restore a pending dangling pointer.
		if ( object->isEffectivelyDead() || object->isDestroyed() )
		{
			onRemoving( object );
			object->onRemovedFrom( getObject() );
			it = list.erase( it );
		}
		else
		{
			++it;
		}
	}

	// Swap the list back where it belongs.
	m_containList.swap(list);
	m_containListSize = (UnsignedInt)m_containList.size();

#endif // RETAIL_COMPATIBLE_CRC
}

//-------------------------------------------------------------------------------------------------
Bool OpenContain::isWeaponBonusPassedToPassengers() const
{
	return getOpenContainModuleData()->m_weaponBonusPassedToPassengers;
}

//-------------------------------------------------------------------------------------------------
WeaponBonusConditionFlags OpenContain::getWeaponBonusPassedToPassengers() const
{
	// Our entire weapon bonus flag set is passed on.  Maybe that could be limited in the future.
	return getObject()->getWeaponBonusCondition();
}

//-------------------------------------------------------------------------------------------------
Real OpenContain::getDamagePercentageToUnits()
{
	return getOpenContainModuleData()->m_damagePercentageToUnits;
}

//-------------------------------------------------------------------------------------------------
Bool OpenContain::isEnclosingContainerFor( const Object * ) const
{
	return TRUE;
}

//-------------------------------------------------------------------------------------------------
Bool OpenContain::hasObjectsWantingToEnterOrExit() const
{
	return !m_objectEnterExitInfo.empty();
}


//-------------------------------------------------------------------------------------------------
void OpenContain::setRallyPoint( const Coord3D *pos )
{
	m_rallyPoint = *pos;
	m_rallyPointExists = true;
}

//-------------------------------------------------------------------------------------------------
const Coord3D *OpenContain::getRallyPoint() const
{
	if (m_rallyPointExists)
		return &m_rallyPoint;

#if !RETAIL_COMPATIBLE_CRC
	// TheSuperHackers @bugfix arcticdolphin 02/03/2026 Use primary exit interface rally point if available.
	if (getObject())
	{
		ExitInterface *primaryExit = getObject()->getObjectExitInterface();
		if (primaryExit && primaryExit != static_cast<const ExitInterface *>(this))
		{
			return primaryExit->getRallyPoint();
		}
	}
#endif

	return nullptr;
}

//-------------------------------------------------------------------------------------------------
Bool OpenContain::getNaturalRallyPoint( Coord3D& rallyPoint, Bool offset )  const
{
	Int numberExits = getOpenContainModuleData()->m_numberOfExitPaths;
	if( numberExits > 0 )
	{
		AsciiString endBone("ExitEnd");
		if( numberExits > 1 )
		{
				endBone.concat("01");
		}
		getObject()->getSingleLogicalBonePosition( endBone.str(), &rallyPoint, nullptr );
	}
	else
	{
		rallyPoint = *getObject()->getPosition();
	}
	return TRUE;
}







//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void testForAttackingProc( Object *obj, void *userData )
{
	Bool *info = (Bool*)userData;
  if ( *info == TRUE )
    return;

  *info = ( obj->testStatus( OBJECT_STATUS_IS_ATTACKING ) );

}

//-------------------------------------------------------------------------------------------------
Bool OpenContain::isAnyRiderAttacking() const
{
  Bool wellIsHe = FALSE;

	((ContainModuleInterface*)this)->iterateContained(testForAttackingProc, &wellIsHe, FALSE );

  return wellIsHe;
}











// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void OpenContain::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version
	* 2: Added m_passengerAllowedToFire
	*/
// ------------------------------------------------------------------------------------------------
void OpenContain::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 2;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// contain list data
	ObjectID objectID;
	if( xfer->getXferMode() == XFER_SAVE )
	{

		// contain list size
		xfer->xferUnsignedInt( &m_containListSize );

		for( ContainedItemsList::const_iterator it = m_containList.begin(); it != m_containList.end(); ++it )
		{

			objectID = (*it)->getID();
			xfer->xferObjectID( &objectID );

		}

	}
	else
	{

		// the containment list should be empty at this time
		if( m_containList.empty() == FALSE )
		{
#if 1
			// srj sez: yeah, it SHOULD be, but a few rogue things come into existence (via their ctor) preloaded
			// with stuff (eg, "CabooseFullOfTerrorists"). so just nuke 'em here. sigh...
			m_containListSize = 0;
			for (ContainedItemsList::iterator it = m_containList.begin(); it != m_containList.end(); /*++it*/ )
			{
				// clear as go along, otherwise the obj will try to remove itself, munging things
				Object* tmp = *it;
				it = m_containList.erase(it);
				TheGameLogic->destroyObject(tmp);
			}
			m_containList.clear();
#else
			DEBUG_CRASH(( "OpenContain::xfer - Contain list should be empty before load but is not" ));
			throw SC_INVALID_DATA;
#endif

		}

		// contain list size
		xfer->xferUnsignedInt( &m_containListSize );

		// read all contained items
		for( UnsignedInt i = 0; i < m_containListSize; ++i )
		{

			// read id
			xfer->xferObjectID( &objectID );

			// put on list for later processing once objects are loaded
			m_xferContainIDList.push_back( objectID );

		}

	}

	// player entered mask
	xfer->xferUser( &m_playerEnteredMask, sizeof( PlayerMaskType ) );

	// last unload sound frame
	xfer->xferUnsignedInt( &m_lastUnloadSoundFrame );

	// last load sound frame
	xfer->xferUnsignedInt( &m_lastLoadSoundFrame );

	// stealth units contained
	xfer->xferUnsignedInt( &m_stealthUnitsContained ); // TheSuperHackers @todo This is redundant information in xfer. Remove it.

	// door close countdown
	xfer->xferUnsignedInt( &m_doorCloseCountdown );

	// condition state
	m_conditionState.xfer( xfer );

	// fire points
	xfer->xferUser( &m_firePoints, sizeof( Matrix3D ) * MAX_FIRE_POINTS );

	// fire point start
	xfer->xferInt( &m_firePointStart );

	// fire point next
	xfer->xferInt( &m_firePointNext );

	// fire point size
	xfer->xferInt( &m_firePointSize );

	// no fire points in art
	xfer->xferBool( &m_noFirePointsInArt );

	// rally point
	xfer->xferCoord3D( &m_rallyPoint );

	// rally point exists
	xfer->xferBool( &m_rallyPointExists );

	// enter exit map info
	UnsignedShort enterExitCount = m_objectEnterExitInfo.size();
	xfer->xferUnsignedShort( &enterExitCount );
	ObjectEnterExitType enterExitType;
	if( xfer->getXferMode() == XFER_SAVE )
	{

		ObjectEnterExitMap::const_iterator it;
		for( it = m_objectEnterExitInfo.begin(); it != m_objectEnterExitInfo.end(); ++it )
		{

			// object id
			objectID = (*it).first;
			xfer->xferObjectID( &objectID );

			// enter exit type
			enterExitType = (*it).second;
			xfer->xferUser( &enterExitType, sizeof( ObjectEnterExitType ) );

		}

	}
	else
	{

		// the map should be empty now
		if( m_objectEnterExitInfo.empty() == FALSE )
		{

			DEBUG_CRASH(( "OpenContain::xfer - m_objectEnterExitInfo should be empty, but is not" ));
			throw SC_INVALID_DATA;

		}

		// read all data items
		for( UnsignedShort i = 0; i < enterExitCount; ++i )
		{

			// object id
			xfer->xferObjectID( &objectID );

			// enter exit type
			xfer->xferUser( &enterExitType, sizeof( ObjectEnterExitType ) );

			// assign
			m_objectEnterExitInfo[ objectID ] = enterExitType;

		}

	}

	// which exit path
	xfer->xferInt( &m_whichExitPath );


  if ( version >= 2 )
  {
    xfer->xferBool( &m_passengerAllowedToFire );
  }


}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void OpenContain::loadPostProcess()
{
	Object *us = getObject();

	// extend base class
	UpdateModule::loadPostProcess();

	// the containment list should be empty at this time
	if( m_containList.empty() == FALSE )
	{

		DEBUG_CRASH(( "OpenContain::loadPostProcess - Contain list should be empty before load but is not" ));
		throw SC_INVALID_DATA;

	}

	// turn the contained id list into actual object pointers in the contain list
	m_containListSize = 0;
	m_stealthUnitsContained = 0;
	m_heroUnitsContained = 0;
	Object *obj;
	std::list<ObjectID>::const_iterator idIt;
	for( idIt = m_xferContainIDList.begin(); idIt != m_xferContainIDList.end(); ++idIt )
	{

		// find this object
		obj = TheGameLogic->findObjectByID( *idIt );

		// sanity
		if( obj == nullptr )
		{

			DEBUG_CRASH(( "OpenContain::loadPostProcess - Unable to find object to put on contain list" ));
			throw SC_INVALID_DATA;

		}

		// put object on list
		addToContainList( obj );

		// remove this object from the world if we need to
		if( isEnclosingContainerFor( obj ) )
			addOrRemoveObjFromWorld( obj, FALSE );

		// record in the object who we are contained by
		obj->friend_setContainedBy( us );

	}

	// sanity
	DEBUG_ASSERTCRASH( m_containListSize == m_containList.size(),
										 ("OpenContain::loadPostProcess - contain list count mismatch") );

	// clear the list as we don't need it anymore
	m_xferContainIDList.clear();

}
```
