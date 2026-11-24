# Translation Packet `PKT-265`: `ContainModule`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-265`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/contain/contain_module.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (3):**
  - `Common/Module.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Module.h`
  - `GameLogic/Damage.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/Damage.h`
  - `GameLogic/WeaponBonusConditionFlags.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/WeaponBonusConditionFlags.h`
- **Implemented Module Facets (1):**
  - `ExitInterface* getContainExitInterface()`
- **`Snapshot::crc` / `Snapshot::xfer` Calls:** None in module.

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ContainModule.h`
```cpp
// FILE: ContainModule.h /////////////////////////////////////////////////////////////////////////////////
// Author: Colin Day, September 2001
// Desc:
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

#include "Common/Module.h"
#include "GameLogic/WeaponBonusConditionFlags.h" // Can't forward a typedef.  This should me made a BitFlags class.
#include "GameLogic/Damage.h"

//-------------------------------------------------------------------------------------------------
class OpenContain;
class Player;
class ExitInterface;
class Matrix3D;
class Weapon;
enum CommandSourceType CPP_11(: Int);

//-------------------------------------------------------------------------------------------------
enum ObjectEnterExitType CPP_11(: Int)
{
	WANTS_TO_ENTER,
	WANTS_TO_EXIT,
	WANTS_NEITHER
};


enum EvacDisposition CPP_11(: Int)
{
  EVAC_INVALID = 0,
  EVAC_TO_LEFT,
  EVAC_TO_RIGHT,
  EVAC_BURST_FROM_CENTER,

};

//-------------------------------------------------------------------------------------------------

typedef std::list<Object*> ContainedItemsList;

//-------------------------------------------------------------------------------------------------
typedef ModuleData ContainModuleData;

//-------------------------------------------------------------------------------------------------

typedef void (*ContainIterateFunc)( Object *obj, void *userData );		///< callback type for contain iterate

//-------------------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class ContainModuleInterface
{
public:

	// we have a two basic container types that it is convenient to query and use
	virtual OpenContain *asOpenContain() = 0;

	// our object changed position... react as appropriate.
	virtual void containReactToTransformChange() = 0;


	// containment is the basis for many complex systems, it helps us to have a formal
	// place where we can monitor the outside world if we need to

	//===============================================================================================
	// Containment ==================================================================================
	//===============================================================================================

	virtual Bool isGarrisonable() const = 0;
	virtual Bool isBustable() const = 0;
	virtual Bool isSpecialZeroSlotContainer() const = 0;
	virtual Bool isHealContain() const = 0;
	virtual Bool isTunnelContain() const = 0;
	virtual Bool isRiderChangeContain() const = 0;
	virtual Bool isImmuneToClearBuildingAttacks() const = 0;
  virtual Bool isSpecialOverlordStyleContainer() const = 0;
  virtual Bool isAnyRiderAttacking() const = 0;

	///< if my object gets selected, then my visible passengers should, too
	///< this gets called from
	virtual void clientVisibleContainedFlashAsSelected() = 0;



	/**
		this is used for containers that must do something to allow people to enter or exit...
		eg, land (for Chinook), open door (whatever)... it's called with wants=WANTS_TO_ENTER
		when something is in the enter state, and wants=ENTS_NOTHING when the unit has
		either entered, or given up...
	*/
	virtual void onObjectWantsToEnterOrExit(Object* obj, ObjectEnterExitType wants) = 0;

	// returns true iff there are objects currently waiting to enter.
	virtual Bool hasObjectsWantingToEnterOrExit() const = 0;

	/**
		return the player that *appears* to control this unit, given an observing player.
		if null, use getObject()->getControllingPlayer() instead.
	*/
	virtual const Player* getApparentControllingPlayer(const Player* observingPlayer) const = 0;

	virtual void recalcApparentControllingPlayer() = 0;

	//
	// you will want to override onContaining() and onRemoving() if you need to
	// do special actions at those event times for your module
	//
	virtual void onContaining( Object *obj, Bool wasSelected ) = 0;		///< object now contains 'obj'
	virtual void onRemoving( Object *obj ) = 0;			///< object no longer contains 'obj'
	virtual void onCapture( Player *oldOwner, Player *newOwner ) = 0; // Very important to handle capture of container, don't want to differ in teams from passenger to us.
	virtual void onSelling() = 0;///< Container is being sold.  Most people respond by kicking everyone out, but not all.

	virtual Int getContainMax() const = 0; ///< The max needs to be virtual, but only two inheritors care.  -1 means "I don't care".

	virtual ExitInterface* getContainExitInterface() = 0;

	virtual void orderAllPassengersToExit( CommandSourceType, Bool instantly ) = 0; ///< All of the smarts of exiting are in the passenger's AIExit. removeAllFrommContain is a last ditch system call, this is the game Evacuate
	virtual void orderAllPassengersToIdle( CommandSourceType ) = 0; ///< Just like it sounds
	virtual void orderAllPassengersToHackInternet( CommandSourceType ) = 0; ///< Just like it sounds
	virtual void markAllPassengersDetected() = 0;										///< Cool game stuff got added to the system calls since this layer didn't exist, so this regains that functionality

	//
	// interface for containing objects inside of objects.  Objects that are
	// contained remove their drawable representations entirely from the client
	//
	/**
		can this container contain this kind of object?
		and, if checkCapacity is TRUE, does this container have enough space left to hold the given unit?
	*/
	virtual Bool isValidContainerFor(const Object* obj, Bool checkCapacity) const = 0;
	virtual void addToContain( Object *obj ) = 0;						///< add 'obj' to contain list
	virtual void addToContainList( Object *obj ) = 0;		///< The part of AddToContain that inheritors can override (Can't do whole thing because of all the private stuff involved)
	virtual void removeFromContain( Object *obj, Bool exposeStealthUnits = FALSE ) = 0;			///< remove 'obj' from contain list
	virtual void removeAllContained( Bool exposeStealthUnits = FALSE ) = 0;									///< remove all objects on contain list
	virtual void killAllContained() = 0;									///< kill all objects on contain list
  virtual void harmAndForceExitAllContained( DamageInfo *info ) = 0; // apply canned damage against those contains
	virtual Bool isEnclosingContainerFor( const Object *obj ) const = 0;	///< Does this type of Contain Visibly enclose its contents?
	virtual Bool isPassengerAllowedToFire( ObjectID id = INVALID_ID ) const = 0;	///< Hey, can I shoot out of this container?
	virtual void setPassengerAllowedToFire( Bool permission = TRUE ) = 0;	///< Hey, can I shoot out of this container?
	virtual void setOverrideDestination( const Coord3D * ) = 0; ///< Instead of falling peacefully towards a clear spot, I will now aim here
	virtual Bool isDisplayedOnControlBar() const = 0;///< Does this container display its contents on the ControlBar?
	virtual Int getExtraSlotsInUse() = 0;
	virtual Bool isKickOutOnCapture() = 0;///< Does this contain module kick people out when captured?

	// list access
	virtual void iterateContained( ContainIterateFunc func, void *userData, Bool reverse ) = 0;		///< iterate the contain list
	virtual UnsignedInt getContainCount() const = 0;											///< contained count
	virtual const ContainedItemsList* getContainedItemsList() const = 0;
	virtual Bool isContained( const Object *obj ) const = 0;	///< Return whether the object is contained in this module
	virtual const Object *friend_getRider() const = 0; ///< Damn.  The draw order dependency bug for riders means that our draw module needs to cheat to get around it.
	virtual Real getContainedItemsMass() const = 0;
	virtual UnsignedInt getStealthUnitsContained() const = 0;
	virtual UnsignedInt getHeroUnitsContained() const = 0;

	virtual Bool calcBestGarrisonPosition( Coord3D *sourcePos, const Coord3D *targetPos ) = 0;
	virtual Bool attemptBestFirePointPosition( Object *source, Weapon *weapon, Object *victim ) = 0;
	virtual Bool attemptBestFirePointPosition( Object *source, Weapon *weapon, const Coord3D *targetPos ) = 0;

	// Player Occupancy.
	virtual PlayerMaskType getPlayerWhoEntered() const = 0;

	virtual void processDamageToContained(Real percentDamage) = 0; ///< Do our % damage to units now.
  virtual Object* getClosestRider ( const Coord3D *pos ) = 0;

	virtual void enableLoadSounds( Bool enable ) = 0;

  virtual void setEvacDisposition( EvacDisposition disp ) = 0;

	virtual Bool isWeaponBonusPassedToPassengers() const = 0;
	virtual WeaponBonusConditionFlags getWeaponBonusPassedToPassengers() const = 0;


	// this exists really just so someone can override it to prevent pip showings...
	virtual Bool getContainerPipsToShow(Int& numTotal, Int& numFull)
	{
		numTotal = getContainMax();
		numFull = getContainCount() + getExtraSlotsInUse();
// srj sez: this makes the pips display in the same manner as the inventory control bar...
//		numTotal = getContainMax() - getExtraSlotsInUse();
//		numFull = getContainCount();

		return true;
	}
};
//-------------------------------------------------------------------------------------------------
```
