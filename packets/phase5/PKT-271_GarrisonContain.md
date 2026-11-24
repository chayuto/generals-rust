# Translation Packet `PKT-271`: `GarrisonContain`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-271`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/contain/garrison_contain.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (24):**
  - `Common/GameState.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/GameState.h`
  - `Common/ModelState.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ModelState.h`
  - `Common/PerfTimer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/PerfTimer.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/PlayerList.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/PlayerList.h`
  - `Common/RandomValue.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/RandomValue.h`
  - `Common/Team.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Team.h`
  - `Common/ThingFactory.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ThingFactory.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/GameClient.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/GameClient.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameClient/View.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/View.h`
  - `GameLogic/AIPathfind.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/AIPathfind.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/GarrisonContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/GarrisonContain.h`
  - `GameLogic/Module/OpenContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/OpenContain.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `GameLogic/Weapon.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (6):**
  - `"MobileGarrison" => INI::parseBool`
  - `"HealObjects" => INI::parseBool`
  - `"TimeForFullHeal" => INI::parseDurationReal`
  - `"InitialRoster" => parseInitialRoster`
  - `"ImmuneToClearBuildingAttacks" => INI::parseBool`
  - `"IsEnclosingContainer" => INI::parseBool`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (19 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUser( &teamID, sizeof( TeamID ) )`
  3. `xferBool( &m_hideGarrisonedStateFromNonallies )`
  4. `xferUnsignedShort( &pointDataCount )`
  5. `xferObjectID( &objectID )`
  6. `xferObjectID( &m_garrisonPointData[ i ].targetID )`
  7. `xferUnsignedInt( &m_garrisonPointData[ i ].placeFrame )`
  8. `xferUnsignedInt( &m_garrisonPointData[ i ].lastEffectFrame )`
  9. `xferDrawableID( &drawableID )`
  10. `xferObjectID( &objectID )`
  11. `xferObjectID( &targetID )`
  12. `xferUnsignedInt( &placeFrame )`
  13. `xferUnsignedInt( &lastEffectFrame )`
  14. `xferDrawableID( &drawableID )`
  15. `xferInt( &m_garrisonPointsInUse )`
  16. `xferUser( m_garrisonPoint, sizeof( Coord3D ) * MAX_GARRISON_POINT_CONDITIONS * MAX_GARRISON_POINTS )`
  17. `xferBool( &m_garrisonPointsInitialized )`
  18. `xferBool( &m_rallyValid )`
  19. `xferCoord3D( &m_exitRallyPoint )`
- **PRNG Calls (3 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValueReal( -containerHalfLength/4, containerHalfLength/4 )`
  - `GameLogicRandomValueReal( containerHalfWidth/2, containerHalfWidth * 2)`
  - `GameLogicRandomValueReal( -containerHalfLength, containerHalfLength )`
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if defined(RTS_DEBUG) || !RETAIL_COMPATIBLE_CRC`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/GarrisonContain.h`
```cpp
// FILE: GarrisonContain.h ////////////////////////////////////////////////////////////////////////
// Author: Colin Day, February 2002
// Desc:   Contain module for structures that can be garrisoned
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/OpenContain.h"
#include "Common/ModelState.h"

//-------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------


class GarrisonContainModuleData : public OpenContainModuleData
{
public:


	struct InitialRoster
	{
		AsciiString templateName;
		Int count;
	};


	Bool m_doIHealObjects;
	Real m_framesForFullHeal;
	Bool m_mobileGarrison;
	Bool m_immuneToClearBuildingAttacks;
  Bool m_isEnclosingContainer;

	InitialRoster		m_initialRoster;

	GarrisonContainModuleData();

	static void buildFieldParse(MultiIniFieldParse& p)
	{
    OpenContainModuleData::buildFieldParse(p);

		static const FieldParse dataFieldParse[] =
		{
			{ "MobileGarrison", INI::parseBool, nullptr, offsetof( GarrisonContainModuleData, m_mobileGarrison ) },
			{ "HealObjects", INI::parseBool, nullptr, offsetof( GarrisonContainModuleData, m_doIHealObjects ) },
			{ "TimeForFullHeal", INI::parseDurationReal, nullptr, offsetof( GarrisonContainModuleData, m_framesForFullHeal ) },
			{ "InitialRoster", parseInitialRoster, nullptr, 0 },
			{ "ImmuneToClearBuildingAttacks", INI::parseBool, nullptr, offsetof( GarrisonContainModuleData, m_immuneToClearBuildingAttacks ) },
      { "IsEnclosingContainer", INI::parseBool, nullptr, offsetof( GarrisonContainModuleData, m_isEnclosingContainer ) },

			{ nullptr, nullptr, nullptr, 0 }
		};
    p.add(dataFieldParse);
	};

	static void parseInitialRoster( INI* ini, void *instance, void *store, const void* )
	{
		GarrisonContainModuleData* self = (GarrisonContainModuleData*)instance;
		const char* name = ini->getNextToken();
		const char* countStr = ini->getNextTokenOrNull();
		Int count = countStr ? INI::scanInt(countStr) : 1;

		self->m_initialRoster.templateName.set(name);
		self->m_initialRoster.count = count;
	};


};

//-------------------------------------------------------------------------------------------------
/** A GarrisonContain is used for objects that can be garrisoned, heh, go figure */
//-------------------------------------------------------------------------------------------------
class GarrisonContain : public OpenContain
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( GarrisonContain, "GarrisonContain" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( GarrisonContain, GarrisonContainModuleData )

public:

	GarrisonContain( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	virtual UpdateSleepTime update() override;						///< called once per frame

	virtual Bool isValidContainerFor( const Object* obj, Bool checkCapacity) const override; // Garrison has an extra check forbidding any containment if ReallyDamaged
	virtual Bool isGarrisonable() const override { return true; }	///< can this unit be Garrisoned? (ick)
	virtual Bool isBustable() const override { return TRUE; }	///< can this container get busted by a bunkerbuster
	virtual Bool isImmuneToClearBuildingAttacks() const override { return getGarrisonContainModuleData()->m_immuneToClearBuildingAttacks; }
	virtual Bool isHealContain() const override { return false; } ///< true when container only contains units while healing (not a transport!)
	virtual Bool isTunnelContain() const override { return FALSE; }
	virtual Bool isPassengerAllowedToFire(  ObjectID id = INVALID_ID  ) const override;	///< Hey, can I shoot out of this container?
  virtual Bool isEnclosingContainerFor( const Object *obj ) const override { return getGarrisonContainModuleData()->m_isEnclosingContainer; }
  virtual Bool isSpecialOverlordStyleContainer() const override {return FALSE;}

	virtual void removeAllContained( Bool exposeStealthUnits ) override;	///< remove all contents of this open container

	virtual void exitObjectViaDoor( Object *exitObj, ExitDoorType exitDoor ) override;	///< exit one of our content items from us
	virtual void exitObjectByBudding( Object *newObj, Object *budHost ) override { return; };
	virtual void onContaining( Object *obj, Bool wasSelected ) override;				///< object now contains 'obj'
	virtual void onRemoving( Object *obj ) override;					///< object no longer contains 'obj'
  virtual void onSelling() override;


	// A Garrison Contain must eject all passengers when it crosses the ReallyDamaged threshold.
	virtual void onBodyDamageStateChange( const DamageInfo* damageInfo,
																				BodyDamageType oldState,
																				BodyDamageType newState) override;  ///< Die Interface state change callback

	/**
		return the player that *appears* to control this unit, given an observing player.
		if null, use getObject()->getControllingPlayer() instead.
	*/
	virtual const Player* getApparentControllingPlayer( const Player* observingPlayer ) const override;
	virtual void recalcApparentControllingPlayer() override;
	virtual Bool isDisplayedOnControlBar() const override {return TRUE;}///< Does this container display its contents on the ControlBar?

  virtual void onDamage( DamageInfo *info ) override;

  virtual void setEvacDisposition( EvacDisposition disp ) override { m_evacDisposition = disp; };

protected:

	virtual void redeployOccupants() override;				///< redeploy the occupants of us at all available garrison points
	virtual void onObjectCreated() override;

	void validateRallyPoint();							///< validate (if necessary) and pick (if possible) an exit rally point

	virtual Bool calcBestGarrisonPosition( Coord3D *sourcePos, const Coord3D *targetPos ) override;
	virtual Bool attemptBestFirePointPosition( Object *source, Weapon *weapon, Object *victim ) override;
	virtual Bool attemptBestFirePointPosition( Object *source, Weapon *weapon, const Coord3D *targetPos ) override;

	void updateEffects();										///< do any effects needed per frame
	void loadGarrisonPoints();							///< load garrison point position data and save for later
	void putObjectAtBestGarrisonPoint( Object *obj, Object *target, const Coord3D *targetPos );	///< place object at position of the best garrison point to use for its target
	void putObjectAtGarrisonPoint( Object *obj, ObjectID targetID, Int conditionIndex, Int index );					///< place object at the specified garrison point index
	enum { SEARCH_FOR_REMOVE = -1 };
	void removeObjectFromGarrisonPoint( Object *obj, Int index = SEARCH_FOR_REMOVE );///< remove object from the garrison point placement
	void addValidObjectsToGarrisonPoints();			///< add any objects with targets to a garrison point
	void removeInvalidObjectsFromGarrisonPoints();	///< remove objects with invalid targets from valid points
	void trackTargets();										///< keep attackers at the closest garrison point to their active target

  void matchObjectsToGarrisonPoints();                ///< Every frame, and whenever anyone enters or leaves
  void positionObjectsAtStationGarrisonPoints();  ///< enforce that everybody stays at their pre-assigned space
  void loadStationGarrisonPoints();
  Bool pickAStationForMe( const Object *pbj );
  void removeObjectFromStationPoint( const Object *obj );

	enum { GARRISON_INDEX_INVALID = -1 };
	Int findConditionIndex();										///< find the condition index to use given the current object body state
	Int getObjectGarrisonPointIndex( Object *obj );		///< get the garrison point index object is at (if present)
	Int findClosestFreeGarrisonPointIndex( Int conditionIndex,
																				 const Coord3D *targetPos );  ///< find closest free garrison point to the target location

	void healObjects();	///< heal all the objects within me
	void healSingleObject( Object *obj, Real frames );	///< heal just one of the objects within me
	void moveObjectsWithMe();				///< translates all the garrisoned object to this->getObject()->getPosition()

private:

	enum { MAX_GARRISON_POINTS = 40 };


	//
	// The max units inside any garrisoned structure is 10.  Since the units will "move around"
	// the inside of the structure to be close to their targets, we need a max of 10 garrison points
	// on each side of the building to accommodate everybody inside
	//
	// ----------------------------------------------------------------------------------------------
	struct GarrisonPointData
	{
		union
		{
			Object *		object;							///< object at this garrison point
			ObjectID		objectID;						///< for loading
		};
		ObjectID			targetID;						///< object ID that is our current target
		UnsignedInt		placeFrame;					///< frame we were placed at this garrison point
		UnsignedInt		lastEffectFrame;		///< last frame we fired our effects on
		union
		{
			Drawable *	effect;							///< effect object for showing gun barrels and muzzle flash fire
			DrawableID	effectID;						///< for loading
		};
	};


  struct StationPointData
  {
    ObjectID  occupantID;
    Coord3D   position;
  };

	// ----------------------------------------------------------------------------------------------
	enum
	{
		GARRISON_POINT_PRISTINE,
		GARRISON_POINT_DAMAGED,
		GARRISON_POINT_REALLY_DAMAGED,

		MAX_GARRISON_POINT_CONDITIONS
	};

	Team *							m_originalTeam;																///< our original team before we were garrisoned
	GarrisonPointData		m_garrisonPointData[ MAX_GARRISON_POINTS ];		///< the garrison point placement data
	Int									m_garrisonPointsInUse;
	Coord3D							m_garrisonPoint[ MAX_GARRISON_POINT_CONDITIONS ][ MAX_GARRISON_POINTS ];		///< the garrison point positions (in world coords) for pristine, damaged, and really damaged
	Coord3D							m_exitRallyPoint;												///< Point to rally at when exiting structure (if possible)

  std::vector<StationPointData> m_stationPointList;

	Bool		m_stationGarrisonPointsInitialized;	///< DO NOT XFER THIS!!!   TRUE once we have loaded the pre-assigned garrison point positions from the art
	Bool		m_garrisonPointsInitialized;							///< TRUE once we have loaded the garrison point positions from the art
	Bool		m_hideGarrisonedStateFromNonallies;								///< if T, don't appear to be garrisoned (all stealthy)
	Bool		m_rallyValid;															///< TRUE when m_exitRallyPoint is valid

  EvacDisposition m_evacDisposition;

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Contain/GarrisonContain.cpp`
```cpp
// FILE: GarrisonContain.cpp //////////////////////////////////////////////////////////////////////
// Author: Colin Day, February 2002
// Desc:   Contain module for structures that can be garrisoned
///////////////////////////////////////////////////////////////////////////////////////////////////

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/GameState.h"
#include "Common/PerfTimer.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/RandomValue.h"
#include "Common/Team.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"

#include "GameLogic/AIPathfind.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/GarrisonContain.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Weapon.h"

#include "GameClient/Drawable.h"
#include "GameClient/GameClient.h"
#include "GameClient/InGameUI.h"
#include "GameClient/View.h"


///////////////////////////////////////////////////////////////////////////////////////////////////
enum { MUZZLE_FLASH_LIFETIME = LOGICFRAMES_PER_SECOND / 7 };

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
GarrisonContainModuleData::GarrisonContainModuleData()
{

	//
	// by default we say that transports can have infantry inside them, this will be totally
	// overwritten by any data provided from the INI entry tho
	//
	m_allowInsideKindOf = MAKE_KINDOF_MASK( KINDOF_INFANTRY );

	m_mobileGarrison = FALSE;
	m_doIHealObjects = false;			///< if T, then I heal objects that are inside of me
	m_framesForFullHeal = 1.0f;		///< the number of frames something inside of me takes to heal
	m_immuneToClearBuildingAttacks = false;
  m_isEnclosingContainer = TRUE; ///< a sensible default for a garrison container... few exceptions, firebase is one

	m_initialRoster.count = 0;
}

//-----------------------------------------------------------------------------
inline Real calcDistSqr(const Coord3D& a, const Coord3D& b)
{
	return sqr(a.x - b.x) + sqr(a.y - b.y) + sqr(a.z - b.z);
}

// ------------------------------------------------------------------------------------------------
/** Given the target position, find the garrison point that is closest to it */
// ------------------------------------------------------------------------------------------------
Int GarrisonContain::findClosestFreeGarrisonPointIndex( Int conditionIndex,
																												const Coord3D *targetPos )
{
// TheSuperHackers @info helmutbuhler 05/05/2025 This debug mutates the code to become CRC incompatible
#if defined(RTS_DEBUG) || !RETAIL_COMPATIBLE_CRC
	DEBUG_ASSERTCRASH(m_garrisonPointsInitialized, ("garrisonPoints are not inited"));

	DEBUG_ASSERTCRASH(conditionIndex >= 0 && conditionIndex < MAX_GARRISON_POINT_CONDITIONS,
		("GarrisonContain::findClosestFreeGarrisonPointIndex - Invalid condition index '%d'", conditionIndex));
#endif

	// sanity
	if( targetPos == nullptr || m_garrisonPointsInUse == MAX_GARRISON_POINTS )
		return GARRISON_INDEX_INVALID;

	Int closestIndex = GARRISON_INDEX_INVALID;
	Real closestDistSq = -1.0f;
	Real distSq;
	for( Int i = 0; i < MAX_GARRISON_POINTS; ++i )
	{

		// only consider free garrison points
		if( m_garrisonPointData[ i ].object == nullptr )
		{

			// compute the squared distance between these two points
			distSq = calcDistSqr(*targetPos, m_garrisonPoint[ conditionIndex ][ i ]);

			if( distSq < closestDistSq || closestDistSq == -1.0f )
			{

				closestDistSq = distSq;
				closestIndex = i;

			}

		}

	}

	return closestIndex;

}

// ------------------------------------------------------------------------------------------------
/** Given the object, return the garrison point index the object is placed at ... if any */
// ------------------------------------------------------------------------------------------------
Int GarrisonContain::getObjectGarrisonPointIndex( Object *obj )
{

	// sanity
	if( obj == nullptr )
		return GARRISON_INDEX_INVALID;

	for( Int i = 0; i < MAX_GARRISON_POINTS; ++i )
		if( m_garrisonPointData[ i ].object == obj )
			return i;

	return GARRISON_INDEX_INVALID;

}

// ------------------------------------------------------------------------------------------------
/** Put the object at the specified garrison point by index */
// ------------------------------------------------------------------------------------------------
void GarrisonContain::putObjectAtGarrisonPoint( Object *obj,
																								ObjectID targetID,
																								Int conditionIndex,
																								Int pointIndex )
{
	DEBUG_ASSERTCRASH(m_garrisonPointsInitialized, ("garrisonPoints are not inited"));

	// sanity
	if( obj == nullptr || pointIndex < 0 || pointIndex >= MAX_GARRISON_POINTS ||
			conditionIndex < 0 || conditionIndex >= MAX_GARRISON_POINT_CONDITIONS )
	{

		DEBUG_CRASH(( "GarrisionContain::putObjectAtGarrisionPoint - Invalid arguments" ));
		return;

	}

	// make sure this point is empty
	if( m_garrisonPointData[ pointIndex ].object != nullptr )
	{

		DEBUG_CRASH(( "GarrisonContain::putObjectAtGarrisonPoint - Garrison Point '%d' is not empty",
									pointIndex ));
		return;

	}

	// get the position we're going to use
	Coord3D pos = m_garrisonPoint[ conditionIndex ][ pointIndex ];

	// set the object position
	obj->setPosition( &pos );

	// save the data for being place at this point
	m_garrisonPointData[ pointIndex	].object = obj;
	m_garrisonPointData[ pointIndex	].targetID = targetID;
	m_garrisonPointData[ pointIndex	].placeFrame = TheGameLogic->getFrame();
	++m_garrisonPointsInUse;

	//
	// create a drawable that has a gun barrel which will show there is an object at this
	// garrison point ready to shoot
	//
	static const ThingTemplate *muzzle = TheThingFactory->findTemplate( "GarrisonGun" );
	DEBUG_ASSERTCRASH( muzzle, ("Warning, Object 'GarrisonGun' not found and is need for Garrison gun effects") );
	if( muzzle && isEnclosingContainerFor( obj ) )// If we are showing the contained, we need no gun barrel drawable added
	{
		Drawable *draw = TheThingFactory->newDrawable( muzzle );
		if( draw )
		{

			// set position of the drawable at the garrison fire point
			draw->setPosition( &pos );

			// record the drawable in our data array
			m_garrisonPointData[ pointIndex ].effect = draw;
			m_garrisonPointData[ pointIndex ].lastEffectFrame = 0;

			//Copy shroud status from our container.
			Drawable *containerDrawable=getObject()->getDrawable();
			if (containerDrawable)
				draw->setFullyObscuredByShroud(containerDrawable->getFullyObscuredByShroud());

		}

	}

/*
UnicodeString msg;
msg.format( L"Added object '%S'(%d) to point '%d'",
						obj->getTemplate()->getName().str(),
						obj->getID(),
						pointIndex );
TheInGameUI->message( msg );
*/

}

// ------------------------------------------------------------------------------------------------
/** Given the current state of the structure, return the condition index we are to use
	* from the garrison point position arrays */
// ------------------------------------------------------------------------------------------------
Int GarrisonContain::findConditionIndex()
{
	BodyModuleInterface *body = getObject()->getBodyModule();
	BodyDamageType bodyDamage = body->getDamageState();
	Int index = GARRISON_INDEX_INVALID;

	switch( bodyDamage )
	{

		// --------------------------------------------------------------------------------------------
		case BODY_PRISTINE:

			index = GARRISON_POINT_PRISTINE;
			break;

		// --------------------------------------------------------------------------------------------
		case BODY_DAMAGED:

			index = GARRISON_POINT_DAMAGED;
			break;

		// --------------------------------------------------------------------------------------------
		case BODY_REALLYDAMAGED:
		case BODY_RUBBLE:

			index = GARRISON_POINT_REALLY_DAMAGED;
			break;

		// --------------------------------------------------------------------------------------------
		default:

			DEBUG_CRASH(( "GarrisonContain::findConditionIndex - Unknown body damage type '%d'",
										bodyDamage ));
			break;

	}

	return index;

}

//-------------------------------------------------------------------------------------------------
//The weapon system would like to perform a range check assuming the object is placed in the
//best possible available garrison position. If found, we change sourcePos to that position.
//-------------------------------------------------------------------------------------------------
Bool GarrisonContain::calcBestGarrisonPosition( Coord3D *sourcePos, const Coord3D *targetPos )
{
	// sanity
	if( !sourcePos || !targetPos )
		return FALSE;

#ifdef DEBUG_CRASHING
  const GarrisonContainModuleData *modData = getGarrisonContainModuleData();
  DEBUG_ASSERTCRASH(modData->m_isEnclosingContainer, ("calcBestGarrisonPosition... SHOULD NOT GET HERE, since this container is non-enclosing") );
#endif



	// find which garrison point position array we will used based on body condition
	Int conditionIndex = findConditionIndex();

	// get the index of the garrison point that is closest to the target position
	Int placeIndex = findClosestFreeGarrisonPointIndex( conditionIndex, targetPos );
	if( placeIndex == GARRISON_INDEX_INVALID )
	{
		DEBUG_CRASH( ("GarrisonContain::calcBestGarrisonPosition - Unable to find suitable garrison point.") );
		return FALSE;
	}

	sourcePos->set( m_garrisonPoint[ conditionIndex ][ placeIndex ] );
	return TRUE;
}

//-------------------------------------------------------------------------------------------------
//The AI is entering the aim state and would like to move the unit to the best position, perform
//a range check, and if it succeeds, leave him there -- otherwise, remove him immediately.
//-------------------------------------------------------------------------------------------------
Bool GarrisonContain::attemptBestFirePointPosition( Object *source, Weapon *weapon, Object *victim )
{
	//Sanity
	if( !source || !victim || !weapon )
	{
		return FALSE;
	}

#ifdef DEBUG_CRASHING
  const GarrisonContainModuleData *modData = getGarrisonContainModuleData();
  DEBUG_ASSERTCRASH(modData->m_isEnclosingContainer, ("calcBestGarrisonPosition... SHOULD NOT GET HERE, since this container is non-enclosing") );
#endif
	//If this object is already at a garrison point, remove him.
	Int existingIndex = getObjectGarrisonPointIndex( source );
	if( existingIndex != GARRISON_INDEX_INVALID )
	{
		removeObjectFromGarrisonPoint( source, existingIndex );
	}

	putObjectAtBestGarrisonPoint( source, victim, nullptr );

	//Okay, now we have positioned the object in the best position for the victim.
	//Now check if we are able to fire on our victim.
	if( weapon->isWithinAttackRange( source, victim ) )
	{
		return TRUE;
	}

	//Crap, we failed... so remove the object from the garrison point.
	existingIndex = getObjectGarrisonPointIndex( source );
	if( existingIndex != GARRISON_INDEX_INVALID )
	{
		removeObjectFromGarrisonPoint( source, existingIndex );
	}
	return FALSE;
}

//-------------------------------------------------------------------------------------------------
//The AI is entering the aim state and would like to move the unit to the best position, perform
//a range check, and if it succeeds, leave him there -- otherwise, remove him immediately.
//-------------------------------------------------------------------------------------------------
Bool GarrisonContain::attemptBestFirePointPosition( Object *source, Weapon *weapon, const Coord3D *targetPos )
{
	//Sanity
	if( !source || !targetPos || !weapon )
	{
		return FALSE;
	}
#ifdef DEBUG_CRASHING
  const GarrisonContainModuleData *modData = getGarrisonContainModuleData();
  DEBUG_ASSERTCRASH(modData->m_isEnclosingContainer, ("calcBestGarrisonPosition... SHOULD NOT GET HERE, since this container is non-enclosing") );
#endif

	//If this object is already at a garrison point, remove him.
	Int existingIndex = getObjectGarrisonPointIndex( source );
	if( existingIndex != GARRISON_INDEX_INVALID )
	{
		removeObjectFromGarrisonPoint( source, existingIndex );
	}

	putObjectAtBestGarrisonPoint( source, nullptr, targetPos );

	//Okay, now we have positioned the object in the best position for the targetPos.
	//Now check if we are able to fire on our targetPos.
	if( weapon->isWithinAttackRange( source, targetPos ) )
	{
		return TRUE;
	}

	//Crap, we failed... so remove the object from the garrison point.
	existingIndex = getObjectGarrisonPointIndex( source );
	if( existingIndex != GARRISON_INDEX_INVALID )
	{
		removeObjectFromGarrisonPoint( source, existingIndex );
	}
	return FALSE;
}

//-------------------------------------------------------------------------------------------------
/** Place the object at the "best" garrison point position so it's on the same "side" of
	* the structure that its target is */
//-------------------------------------------------------------------------------------------------
void GarrisonContain::putObjectAtBestGarrisonPoint( Object *obj, Object *target, const Coord3D *targetPos )
{

	// sanity
	if( obj == nullptr || (target == nullptr && targetPos == nullptr) )
		return;

#ifdef DEBUG_CRASHING
  const GarrisonContainModuleData *modData = getGarrisonContainModuleData();
  DEBUG_ASSERTCRASH(modData->m_isEnclosingContainer, ("calcBestGarrisonPosition... SHOULD NOT GET HERE, since this container is non-enclosing") );
#endif
	// if obj target, override pos
	if (target != nullptr)
		targetPos = target->getPosition();

	// if this object is already at a garrison point do nothing
	if( getObjectGarrisonPointIndex( obj ) != GARRISON_INDEX_INVALID )
		return;

	// find which garrison point position array we will used based on body condition
	Int conditionIndex = findConditionIndex();

	// get the index of the garrison point that is closest to the target position
	Int placeIndex = findClosestFreeGarrisonPointIndex( conditionIndex, targetPos );
	DEBUG_ASSERTCRASH( placeIndex != GARRISON_INDEX_INVALID,
										 ("GarrisonContain::putObjectAtBestGarrisonPoint - Unable to find suitable garrison point for '%s'",
										 obj->getTemplate()->getName().str()) );

	// put it here
	putObjectAtGarrisonPoint( obj, target ? target->getID() : INVALID_ID, conditionIndex, placeIndex );

}

// ------------------------------------------------------------------------------------------------
/** Remove the object from the garrison point position and replace at the center of the building */
// ------------------------------------------------------------------------------------------------
void GarrisonContain::removeObjectFromGarrisonPoint( Object *obj, Int index )
{

  if ( ! isEnclosingContainerFor(obj) )
    return;// since I am not enclosed, I am not at a garrison point!

#ifdef DEBUG_CRASHING
  const GarrisonContainModuleData *modData = getGarrisonContainModuleData();
  DEBUG_ASSERTCRASH(modData->m_isEnclosingContainer, ("calcBestGarrisonPosition... SHOULD NOT GET HERE, since this container is non-enclosing") );
#endif

	// sanity
	if( obj == nullptr )
		return;

	// search for the object in the garrison point data, if found, remove it
	Int removeIndex = index;
	if( removeIndex == SEARCH_FOR_REMOVE )
	{

		for( Int i = 0; i < MAX_GARRISON_POINTS; ++i )
		{

			if( m_garrisonPointData[ i ].object == obj )
			{

				removeIndex = i;
				break;

			}

		}

	}

	// validate the index slot to remove
	if( removeIndex < 0 || removeIndex >= MAX_GARRISON_POINTS )
	{

		//
		// this is not an error, if a search was ordered, we may very well not find the
		// object at a garrison point and therefore can't remove it
		//
		return;

	}

	// remove from this spot
	m_garrisonPointData[ removeIndex ].object = nullptr;
	m_garrisonPointData[ removeIndex ].targetID = INVALID_ID;
	m_garrisonPointData[ removeIndex ].placeFrame = 0;
	m_garrisonPointData[ removeIndex ].lastEffectFrame = 0;
	--m_garrisonPointsInUse;

	// destroy drawable for gun barrel and effects if present
	if( m_garrisonPointData[ removeIndex ].effect )
		TheGameClient->destroyDrawable( m_garrisonPointData[ removeIndex ].effect );
	m_garrisonPointData[ removeIndex ].effect = nullptr;

	// set the position of the object to back to the center of the garrisoned building
	obj->setPosition( getObject()->getPosition() );

/*
UnicodeString msg;
msg.format( L"Removed object '%S'(%d) from point '%d'",
						obj->getTemplate()->getName().str(),
						obj->getID(),
						removeIndex );
TheInGameUI->message( msg );
*/

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
GarrisonContain::GarrisonContain( Thing *thing, const ModuleData *moduleData ) :
								 OpenContain( thing, moduleData )
{
	Int i, j;

	m_originalTeam = nullptr;
	m_hideGarrisonedStateFromNonallies = FALSE;
	m_garrisonPointsInUse = 0;
	m_garrisonPointsInitialized = FALSE;
  m_stationGarrisonPointsInitialized = FALSE;

	for( i = 0; i < MAX_GARRISON_POINTS; i++ )
	{

		m_garrisonPointData[ i ].object = nullptr;
		m_garrisonPointData[ i ].targetID = INVALID_ID;
		m_garrisonPointData[ i ].placeFrame = 0;
		m_garrisonPointData[ i ].lastEffectFrame = 0;
		m_garrisonPointData[ i ].effect = nullptr;

		for( j = 0; j < MAX_GARRISON_POINT_CONDITIONS; ++j )
			m_garrisonPoint[ j ][ i ].zero();

	}

	m_rallyValid = FALSE;
	m_exitRallyPoint.zero();


  m_evacDisposition = EVAC_BURST_FROM_CENTER; // default, anyway


  m_stationPointList.clear();

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
GarrisonContain::~GarrisonContain()
{

}

//-------------------------------------------------------------------------------------------------
/** can this container contain this kind of object?
	and, if checkCapacity is TRUE, does this container have enough space
	left to hold the given unit? */
// ------------------------------------------------------------------------------------------------
Bool GarrisonContain::isValidContainerFor(const Object* obj, Bool checkCapacity) const
{

	// extend functionality // this just tests kindof masks
	if( OpenContain::isValidContainerFor( obj, checkCapacity ) == false )
		return false;

	// zero-health buildings are not garrisonable.
	if (getObject()->getBodyModule()->getHealth() <= 0.0f)
		return false;

	// ReallyDamaged buildings are not garrisonable as well.
	if( getObject()->getBodyModule()->getDamageState() == BODY_REALLYDAMAGED && !getObject()->isKindOf( KINDOF_GARRISONABLE_UNTIL_DESTROYED ) )
		return false;

	if (obj && !obj->isKindOf(KINDOF_NO_GARRISON))
	{
		if (checkCapacity)
		{
			Int garrisonMax = getContainMax();
			Int containCount = getContainCount();
			return ( containCount < garrisonMax );
		}
		else
		{
			return true;
		}
	}
	return false;
}

// ------------------------------------------------------------------------------------------------
/** Any objects that are sitting at the garrison points which no longer have targets need
	* to be moved to the center of the building and taken off the garrison point */
// ------------------------------------------------------------------------------------------------
void GarrisonContain::removeInvalidObjectsFromGarrisonPoints()
{
#ifdef DEBUG_CRASHING
  const GarrisonContainModuleData *modData = getGarrisonContainModuleData();
  DEBUG_ASSERTCRASH(modData->m_isEnclosingContainer, ("removeinvalidobjFromGarrisonPoint... SHOULD NOT GET HERE, since this container is non-enclosing") );
#endif
	Object *obj;

	if (m_garrisonPointsInUse == 0)
		return;	// my, that was easy

	for( Int i = 0; i < MAX_GARRISON_POINTS; ++i )
	{

		obj = m_garrisonPointData[ i ].object;
		if( obj )
		{
			AIUpdateInterface *ai = obj->getAIUpdateInterface();

			Bool targetIsValid = true;	// assume true for now...
			Object *goalObject = ai->getGoalObject();
			if( goalObject )
			{
				Weapon *weapon = obj->getCurrentWeapon();
				if( !weapon || !weapon->isWithinAttackRange( obj, goalObject ) )
				{
					//As a garrisoned member, if our target is out of range,
					//then get out of the space, because someone else might
					//be able to shoot it.
					targetIsValid = false;
				}
			}

			// note that we can be attacking a position, rather than an object...
			if( !obj->testStatus(OBJECT_STATUS_IS_ATTACKING) || !targetIsValid )
			{
				removeObjectFromGarrisonPoint( obj, i );
			}

		}

	}

}

// ------------------------------------------------------------------------------------------------
/** Are there any objects in the center that have now obtained targets and need to move to
	* a garrison point */
	// ------------------------------------------------------------------------------------------------
void GarrisonContain::addValidObjectsToGarrisonPoints()
{


#ifdef DEBUG_CRASHING
  const GarrisonContainModuleData *modData = getGarrisonContainModuleData();
  DEBUG_ASSERTCRASH(modData->m_isEnclosingContainer, ("addvalidobjtoGarrisonPoint... SHOULD NOT GET HERE, since this container is non-enclosing") );
#endif


	const ContainedItemsList& containList = getContainList();

	if (containList.empty())
		return;

	for( ContainedItemsList::const_iterator it = containList.begin(); it != containList.end(); ++it )
	{
		Object* obj = *it;
		AIUpdateInterface* ai = obj->getAIUpdateInterface();
		if( ai )
		{
			Object *victim = ai->getCurrentVictim();
			const Coord3D *victimPos = ai->getCurrentVictimPos();

			//
			// add this object to the garrison point that is closest to its target if it's not
			// already in there
			//
			if( victim )
				putObjectAtBestGarrisonPoint( obj, victim, nullptr );
			else if( victimPos )
				putObjectAtBestGarrisonPoint( obj, nullptr, victimPos );

		}

	}

}

// ------------------------------------------------------------------------------------------------
/** Every frame this method is called.  It keeps any of the attacking units at any of the
	* fire points closest to their active target and shuffles them around to any open garrison
	* points that are available if they are closer.  We will also track our targets position
	* and orient any effect stuff we need to (gun barrel / muzzle flash) */
// ------------------------------------------------------------------------------------------------
void GarrisonContain::trackTargets()
{


  if ( ! isEnclosingContainerFor( nullptr ) )
    return; // since ina non-enclosing container, objects fire from their station points, instead of being juggled around between garrison firepoints



	Int conditionIndex = findConditionIndex();
	const ContainedItemsList& containList = getContainList();
	AIUpdateInterface *ai;
	Object *obj;

	DEBUG_ASSERTCRASH(m_garrisonPointsInitialized || containList.empty(), ("garrisonPoints are not inited"));

	for( ContainedItemsList::const_iterator it = containList.begin(); it != containList.end(); ++it )
	{
		// get the object
		obj = *it;

		// only consider objects that are actually at garrison points for re-shuffling
		Int ourIndex = getObjectGarrisonPointIndex( obj );
		if( ourIndex != GARRISON_INDEX_INVALID )
		{

			// does this object have a target?
			ai = obj->getAIUpdateInterface();
			if( ai )
			{
				Object *victim = ai->getCurrentVictim();
				// even though the target position can't change in some cases, still must do this code at least once.
				const Coord3D *victimPos = ai->getCurrentVictimPos();

				if( victim || victimPos )
				{
					if (victim)
						victimPos = victim->getPosition();
					const Coord3D *ourPos = obj->getPosition();

					// find the closest free (of all remaining) garrison points to our target
					Int newIndex = findClosestFreeGarrisonPointIndex( conditionIndex,
																														victimPos );

					// if unable to find another garrison point, don't bother
					if( newIndex != GARRISON_INDEX_INVALID )
					{

						// get the distance from our current index to the target
						Real currentDistSq = calcDistSqr(*victimPos, *ourPos );

						// get the distance from the newly chosen index
						Real newDistSq = calcDistSqr(*victimPos, m_garrisonPoint[ conditionIndex ][ newIndex ] );

						// if the newly chosen index is closer than our current index, switch
						if( newDistSq < currentDistSq )
						{

							// remove from the old index
							removeObjectFromGarrisonPoint( obj, ourIndex );

							// place at the new index
							putObjectAtGarrisonPoint( obj, victim ? victim->getID() : INVALID_ID, conditionIndex, newIndex );

						}

					}

					//
					// we are now either at a new garrison fire point, or we have remained at our
					// existing point still tracking our target.  Orient the effect drawable which
					// shows the gun barrel and muzzle flash towards our target position
					//
					if( m_garrisonPointData[ ourIndex ].effect )
					{
						Coord2D v;
						v.x = victimPos->x - ourPos->x;
						v.y = victimPos->y - ourPos->y;
//					v.z = victomPos->z - ourPos.z;

						// orient the effect object towards the victim position
						m_garrisonPointData[ ourIndex ].effect->setOrientation( v.toAngle() );

					}

				}

			}

		}

	}

}

// ------------------------------------------------------------------------------------------------
/** Remove all the objects at garrison points back to the center and redeploy them among the
	* garrison points.  NOTE that we are preserving the frame in which the object was put
	* at the garrison point originally as this method is used when the model condition changes
	* which could shuffle the garrison point positions but that shouldn't logically change
	* when an object was placed at the point */
// ------------------------------------------------------------------------------------------------
void GarrisonContain::redeployOccupants()
{
	GarrisonPointData garrisonPointDataCopy[ MAX_GARRISON_POINTS ];
	Int i;

	// copy the current set of garrison point data sets
	for( i = 0; i < MAX_GARRISON_POINTS; ++i )
		garrisonPointDataCopy[ i ] = m_garrisonPointData[ i ];

// Lorenzen changed, 6/11/03, so that garrisoncontains that are not enclosing will keep units at their assigned stations,
// rather than Bamphing them all over the building as they fire.
//	// remove the occupants
//	removeInvalidObjectsFromGarrisonPoints();
//	// redeploy them
//	addValidObjectsToGarrisonPoints();


  // ATTENTION... setting this false allows each redeployOccupants() call to create fresh station points, based on the new transform
  // if anything weird ever happens, like rotating buildings and such, we will need a way of transforming the points without clearing the
  // list (and thus forgetting where everyone contained was stationed)... just a handy reminder.
  m_stationGarrisonPointsInitialized = FALSE;


  matchObjectsToGarrisonPoints();

	// restore the frame markers that things were recorded as entering their point
	Int index;
	for( i = 0; i < MAX_GARRISON_POINTS; ++i )
	{

		if( garrisonPointDataCopy[ i ].object )
		{

			// where was this object redeployed
			index = getObjectGarrisonPointIndex( garrisonPointDataCopy[ i ].object );
			if( index != GARRISON_INDEX_INVALID )
				m_garrisonPointData[ index ].placeFrame = garrisonPointDataCopy[ i ].placeFrame;

		}

	}

}

// ------------------------------------------------------------------------------------------------
/** Do any effects during an update cycle that we need to */
// ------------------------------------------------------------------------------------------------
void GarrisonContain::updateEffects()
{


#ifdef DEBUG_CRASHING
  const GarrisonContainModuleData *modData = getGarrisonContainModuleData();
  DEBUG_ASSERTCRASH(modData->m_isEnclosingContainer, ("updateeffects... SHOULD NOT GET HERE, since this container is non-enclosing") );
#endif


	UnsignedInt currentFrame = TheGameLogic->getFrame();
	const ContainedItemsList& containList = getContainList();

	for( ContainedItemsList::const_iterator it = containList.begin(); it != containList.end(); ++it )
	{
		Object *obj;

		// get the object
		obj = *it;

		//
		// did the object fire last frame, if so make a muzzle flash if needed at the
		// garrison point
		//
		if( obj->getLastShotFiredFrame() == currentFrame - 1 )
		{
			Int garrisonIndex = getObjectGarrisonPointIndex( obj );

			// only consider doing muzzle flash logic if the object is actually at a garrison point
			if( garrisonIndex != GARRISON_INDEX_INVALID )
			{

				// set the model condition for the effect object to show the muzzle flash
				Drawable *effect = m_garrisonPointData[ garrisonIndex ].effect;
				if( effect )
				{
					const Weapon *passengerWeapon = obj->getCurrentWeapon();
					if( passengerWeapon && passengerWeapon->getDamageType() != DAMAGE_POISON )// No muzzle flash with poison weapon
					{
						// set the model condition
						effect->setModelConditionState( MODELCONDITION_FIRING_A );

						// mark this "fire frame" so we can turn it off in a little while
						m_garrisonPointData[ garrisonIndex ].lastEffectFrame = currentFrame;

					}
				}

			}

		}

	}

	// remove any firing effects for time that has passed
	for( Int i = 0; i < MAX_GARRISON_POINTS; ++i )
	{

		if( m_garrisonPointData[ i ].effect &&
				m_garrisonPointData[ i ].lastEffectFrame != 0 &&
				currentFrame - m_garrisonPointData[ i ].lastEffectFrame > MUZZLE_FLASH_LIFETIME  )
		{

			// clear the model condition
			m_garrisonPointData[ i ].effect->clearModelConditionState( MODELCONDITION_FIRING_A );

			// clear the last effect frame
			m_garrisonPointData[ i ].lastEffectFrame = 0;

		}

	}

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
UpdateSleepTime GarrisonContain::update()
{
	const GarrisonContainModuleData *modData = getGarrisonContainModuleData();

	// extend functionality
	UpdateSleepTime result;
	result = OpenContain::update();

	// remove effectively dead objects from this garrison container
	const ContainedItemsList& containList = getContainList();
	Object *contained;
	for( ContainedItemsList::const_iterator it = containList.begin(); it != containList.end(); /*empty*/ )
	{

		// get object
		contained = *it;

		// increment iterator, we may delete the object
		++it;

		// remove if dead
		if( contained->isEffectivelyDead() )
		{
			// remove from container
			removeFromContain( contained );

			// set the safe occlusion frame to way way way in the future so we never see it during death
			#define HUGE_FRAME_IN_FUTURE (LOGICFRAMES_PER_SECOND * 1000)
			contained->setSafeOcclusionFrame( TheGameLogic->getFrame() + HUGE_FRAME_IN_FUTURE );

		}

	}

// Lorenzen changed, 6/11/03, so that garrisoncontains that are not enclosing will keep units at their assigned stations,
// rather than Bamphing them all over the building as they fire.
//	// are there any objects at the garrison points who now need to go back to the center of the structure
//	removeInvalidObjectsFromGarrisonPoints();
//
//	//
//	// are there any objects in the center that have now obtained targets and need to move to
//	// a garrison point
//	//
//	addValidObjectsToGarrisonPoints();
  matchObjectsToGarrisonPoints();

	healObjects();

	if (modData->m_mobileGarrison && (getObject()->isMobile() == TRUE) )
	{
		moveObjectsWithMe();
	}
	else
	{
		// sanity information
		DEBUG_ASSERTCRASH( getObject()->isMobile() == FALSE,
		 ("GarrisonContain::update - Objects with garrison contain can be spec'd as 'mobile' in the INI. Do you really want to do this?") );
	}

	return UPDATE_SLEEP_NONE;
}




//-------------------------------------------------------------------------------------------------
/** Every frame, and whenever anyone enters or leaves */
// ------------------------------------------------------------------------------------------------
void GarrisonContain::matchObjectsToGarrisonPoints()
{
  if ( isEnclosingContainerFor( nullptr ) == FALSE )
  {
    // enforce that everybody stays at their pre-assigned space
    positionObjectsAtStationGarrisonPoints();
  }
  else
  {
	  // are there any objects at the garrison points who now need to go back to the center of the structure
	  removeInvalidObjectsFromGarrisonPoints();
	  // are there any objects in the center that have now obtained targets and need to move to
	  // a garrison point
	  addValidObjectsToGarrisonPoints();
	  // any units that have just fired need to have a muzzle flash display out of the fire point
	  updateEffects();
	  // given all the objects that are at the garrison points shooting at something, if their
	  // target moves around the structure and closer to another open garrison point we want
	  // to shuffle our object to the new closest garrison point.  We'll also track the target
	  // here and set orientation for any effects we need to
	  trackTargets();
  }

}



//-------------------------------------------------------------------------------------------------
/** enforce that everybody stays at their pre-assigned space */
// ------------------------------------------------------------------------------------------------
void GarrisonContain::positionObjectsAtStationGarrisonPoints()
{
  if ( ! m_stationGarrisonPointsInitialized )
  {
    loadStationGarrisonPoints();
  }

	const ContainedItemsList& containList = getContainList();
	Object *contained;
	for( ContainedItemsList::const_iterator it = containList.begin(); it != containList.end(); ++it )
	{
		contained = *it;
    Bool foundHisSpot = FALSE;

    // now lets find him in our station point list, and make sure he stays put there.
    for( std::vector<StationPointData>::const_iterator pt = m_stationPointList.begin();
    pt != m_stationPointList.end();
    ++pt)
    {
      const StationPointData *spd = &*pt;

      if( spd->occupantID == contained->getID() )
      {
        contained->setPosition( &spd->position );
        foundHisSpot = TRUE;
        break;
      }

    }

    if ( ! foundHisSpot && ! pickAStationForMe( contained ))
    {
      DEBUG_ASSERTCRASH( foundHisSpot, ("GarrisonContain::positionObjectsAtStationGarrisonPoints found something terribly wrong... \nthere is either a station point shortage, or some other bug."));
    }

	}

}



//-------------------------------------------------------------------------------------------------
/** When a new guy enters a non-enclosing garrison container */
// ------------------------------------------------------------------------------------------------
Bool GarrisonContain::pickAStationForMe( const Object *obj )
{
  Bool foundVacancy = FALSE;
  for( std::vector<StationPointData>::iterator pt = m_stationPointList.begin(); pt != m_stationPointList.end(); ++pt)
  {
    StationPointData *spd = &*pt; // non const
    if ( spd->occupantID  == INVALID_ID ) // found a vacancy
    {
      spd->occupantID = obj->getID();
      foundVacancy = TRUE;
      return TRUE;
    }
  }

  DEBUG_ASSERTCRASH(foundVacancy, ("GarrisonContain::pickAStationForMe is all kinds of bad... \n there was no vacancy found for a newly contained object."));

  return FALSE;

}

void GarrisonContain::removeObjectFromStationPoint( const Object *obj )
{

  //sanity
  if ( obj == nullptr )
    return;

  Bool foundOccupant = FALSE;
  for( std::vector<StationPointData>::iterator pt = m_stationPointList.begin(); pt != m_stationPointList.end(); ++pt)
  {
    StationPointData *spd = &*pt; // non const
    if ( spd->occupantID  == obj->getID() ) // found him sitting there
    {
      spd->occupantID = INVALID_ID;// give up your space
      foundOccupant = TRUE;
      return;
    }
  }


  DEBUG_ASSERTCRASH(foundOccupant, ("GarrisonContain::removeObjectFromStationPoint is all kinds of bad... \n the contained object was not found in station point list."));


}


//-------------------------------------------------------------------------------------------------
/** When I become damaged */
// ------------------------------------------------------------------------------------------------
void GarrisonContain::onDamage( DamageInfo * /*info*/ )
{



//	const ContainedItemsList& containList = getContainList();
//	for( ContainedItemsList::const_iterator it = containList.begin(); it != containList.end(); ++it )
//	{
//		Object *obj;
//
//		// get the object
//		obj = *it;
//
//		healSingleObject(obj, modData->m_framesForFullHeal);
//	}



}




//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void GarrisonContain::healObjects()
{
	const GarrisonContainModuleData *modData = getGarrisonContainModuleData();

	if (!modData->m_doIHealObjects)
		return;

	const ContainedItemsList& containList = getContainList();
	for( ContainedItemsList::const_iterator it = containList.begin(); it != containList.end(); ++it )
	{
		Object *obj;

		// get the object
		obj = *it;

		healSingleObject(obj, modData->m_framesForFullHeal);
	}
}





//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void GarrisonContain::healSingleObject( Object *obj, Real framesForFullHeal)
{
	// setup the healing damageInfo structure with all but the amount
	DamageInfo healInfo;
	healInfo.in.m_damageType = DAMAGE_HEALING;
	healInfo.in.m_deathType = DEATH_NONE;
	//healInfo.in.m_sourceID = getObject()->getID();

	// get body module of the thing to heal
	BodyModuleInterface *body = obj->getBodyModule();

	// if we've been in here long enough ... set our health to max
	if( TheGameLogic->getFrame() - obj->getContainedByFrame() >= framesForFullHeal )
	{

		// set the amount to max just to be sure we're at the top
		healInfo.in.m_amount = body->getMaxHealth();

		// set max health
		body->attemptHealing( &healInfo );

	}
	else
	{
		//
		// given the *whole* time it would take to heal this object, lets pretend that the
		// object is at zero health ... and give it a sliver of health as if it were at 0 health
		// and would be fully healed at 'framesForFullHeal'
		//
		healInfo.in.m_amount = body->getMaxHealth() / framesForFullHeal;

		// do the healing
		body->attemptHealing( &healInfo );

	}
}

//-------------------------------------------------------------------------------------------------
/** return the player that *appears* to control this unit. if null,
		use getObject()->getControllingPlayer() instead. */
// ------------------------------------------------------------------------------------------------
const Player* GarrisonContain::getApparentControllingPlayer( const Player* observingPlayer ) const
{
	const Player* myPlayer = getObject()->getControllingPlayer();

	if ( m_hideGarrisonedStateFromNonallies && m_originalTeam && myPlayer && observingPlayer )
	{
		Relationship r = myPlayer->getRelationship(observingPlayer->getDefaultTeam());
		if (r != ALLIES)
			return m_originalTeam->getControllingPlayer();
	}
	return myPlayer;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void GarrisonContain::recalcApparentControllingPlayer()
{
	//Record original team first time through.
	if( m_originalTeam == nullptr )
	{
		m_originalTeam = getObject()->getTeam();
	}

	// (hokey trick: if our team is null, nuke originalTeam -- this
	// usually means we are being called during game-teardown and
	// the teams are no longer valid...)
	if (getObject()->getTeam() == nullptr)
		m_originalTeam = nullptr;
	// Check to see if we have any units contained in our object
	if( getContainCount() > 0 )
	{
		ContainedItemsList::const_iterator it = getContainList().begin();
		Object *rider = *it;

		// Check to see if all the contained units are stealthy.  Need to set this flag before the capture,
		// since the Radar refresh in setTeam will want to use it to decide our color.
		Bool detected = rider->getStatusBits().test( OBJECT_STATUS_DETECTED );
		m_hideGarrisonedStateFromNonallies = ( !detected && ( getStealthUnitsContained() == getContainCount() ) );

		Player* controller = rider->getControllingPlayer();
		Team *team = controller ? controller->getDefaultTeam() : nullptr;
		if( team )
		{
			getObject()->setTeam( team );
		}
	}
	else
	{
		//Nothing in object, so set team to original team.
		getObject()->setTeam( m_originalTeam );
		m_hideGarrisonedStateFromNonallies = false;
	}

	//Only allow the garrison state to be set if the client team knows that it is garrisoned.
	Drawable *draw = getObject()->getDrawable();
	if( draw )
	{
		Bool setModelGarrisoned = FALSE;


		if ( getContainCount() > 0 )
		{
			ContainedItemsList::const_iterator it = getContainList().begin();
			Object *occupant = *it;
			Bool detected = occupant->getStatusBits().test( OBJECT_STATUS_DETECTED );

			if( detected || (getApparentControllingPlayer(ThePlayerList->getLocalPlayer()) == getObject()->getControllingPlayer()) )
			{
				setModelGarrisoned = TRUE;
			}

		}


		if ( setModelGarrisoned )
			draw->setModelConditionState( MODELCONDITION_GARRISONED );
		else
			draw->clearModelConditionState( MODELCONDITION_GARRISONED );


		// Handle the team color that is rendered
		const Player* controller = getApparentControllingPlayer(ThePlayerList->getLocalPlayer());
		if (controller)
		{
			if (TheGlobalData->m_timeOfDay == TIME_OF_DAY_NIGHT)
				draw->setIndicatorColor( controller->getPlayerNightColor() );
			else
				draw->setIndicatorColor( controller->getPlayerColor() );
		}

		// now that we have an object inside us, we need to get all the garrison point positions
		// if we don't already have them.
		if( getContainCount() > 0 )
    {
      if ( isEnclosingContainerFor( nullptr ) )
      {
        if ( m_garrisonPointsInitialized == FALSE )
		    {
			    loadGarrisonPoints();
		    }
      }
      else // must need station points instead
      {
        if ( m_stationGarrisonPointsInitialized == FALSE )
        {
          loadStationGarrisonPoints();
        }
      }
    }
	}
}

// ------------------------------------------------------------------------------------------------
/** Load the garrison point position data and save for use later */
// ------------------------------------------------------------------------------------------------
void GarrisonContain::loadGarrisonPoints()
{



	const GarrisonContainModuleData *modData = getGarrisonContainModuleData();

  DEBUG_ASSERTCRASH(modData->m_isEnclosingContainer, ("loadGarrisonPoints... SHOULD NOT GET HERE, since this container is non-enclosing") );

  Object *structure = getObject();
	Int i, j;
	Bool gBonesFound = FALSE;

	//
	// initialize all the garrison points to the center of the object, this assumes that we
	// will never move (these points are cached)
	//
	for( i = 0; i < MAX_GARRISON_POINT_CONDITIONS; ++i )
		for( j = 0; j < MAX_GARRISON_POINTS; ++j )
			m_garrisonPoint[ i ][ j ] = *(structure->getPosition());

	//
	// in order to get all the garrison point positions we will actually switch the model
	// condition through the garrisoned pristine, damaged, and really damaged states.  This
	// is especially important because at the time or "garrisoning" a model condition may
	// not actually switch to a garrisoned state because of units that garrison things
	// all stealthy.  By caching all the bone positions once we will have access to
	// any of the points for any condition state at any time
	//
	{
		Int conditionIndex;
		Int count = 0;

		// save the original parameters for the model condition
		Drawable* draw = structure->getDrawable();
		const ModelConditionFlags originalFlags = draw->getModelConditionFlags();
		ModelConditionFlags clearFlags;
		ModelConditionFlags setFlags;

		// pristine garrisoned
		clearFlags.clear();
		setFlags.clear();
		clearFlags.set( MODELCONDITION_REALLY_DAMAGED );
		clearFlags.set( MODELCONDITION_RUBBLE );
		clearFlags.set( MODELCONDITION_SPECIAL_DAMAGED );
		clearFlags.set( MODELCONDITION_DAMAGED );
		setFlags.set( MODELCONDITION_GARRISONED );
		structure->clearAndSetModelConditionFlags( clearFlags, setFlags );
		conditionIndex = GARRISON_POINT_PRISTINE;

		count = structure->getMultiLogicalBonePosition("FIREPOINT", MAX_GARRISON_POINTS, m_garrisonPoint[ conditionIndex ], nullptr);

		if ( count > 0) gBonesFound = TRUE;

		// damaged garrisoned
		clearFlags.clear();
		setFlags.clear();
		clearFlags.set( MODELCONDITION_REALLY_DAMAGED );
		clearFlags.set( MODELCONDITION_RUBBLE );
		clearFlags.set( MODELCONDITION_SPECIAL_DAMAGED );
		setFlags.set( MODELCONDITION_DAMAGED );
		structure->clearAndSetModelConditionFlags( clearFlags, setFlags );
		conditionIndex = GARRISON_POINT_DAMAGED;

		count = structure->getMultiLogicalBonePosition("FIREPOINT", MAX_GARRISON_POINTS, m_garrisonPoint[ conditionIndex ], nullptr);

		if ( count > 0) gBonesFound = TRUE;

		// really damaged garrisoned
		clearFlags.clear();
		setFlags.clear();
		clearFlags.set( MODELCONDITION_RUBBLE );
		clearFlags.set( MODELCONDITION_SPECIAL_DAMAGED );
		clearFlags.set( MODELCONDITION_DAMAGED );
		setFlags.set( MODELCONDITION_REALLY_DAMAGED );
		structure->clearAndSetModelConditionFlags( clearFlags, setFlags );
		conditionIndex = GARRISON_POINT_REALLY_DAMAGED;

		count = structure->getMultiLogicalBonePosition("FIREPOINT", MAX_GARRISON_POINTS, m_garrisonPoint[ conditionIndex ], nullptr);

		if ( count > 0) gBonesFound = TRUE;

		// restore the original condition flags
		draw->replaceModelConditionFlags( originalFlags );

	}

	// garrison points are now initialized
	m_garrisonPointsInitialized = TRUE;



	if (gBonesFound && modData->m_mobileGarrison && (getObject()->isMobile() == TRUE) )
	{
		DEBUG_ASSERTCRASH( getObject()->isMobile() == FALSE,
		 ("GarrisonContain::update - You have specified this garrisonContain as mobile,\n yet you want garrison point placement bones... \n what are you thinking?") );
	}



}

// ------------------------------------------------------------------------------------------------
/** Validate any exit rally point that has been chosen (if any).  If it's not valid,
	* try to find a new one */
// ------------------------------------------------------------------------------------------------
void GarrisonContain::validateRallyPoint()
{

	// if we have a rally point already picked, make sure it's valid
	if( m_rallyValid == TRUE )
	{
		Coord3D result;
		FindPositionOptions options;

		// ask for a valid position exactly at the rally point
		options.flags = FPF_IGNORE_ALLY_OR_NEUTRAL_UNITS;
		options.minRadius = 0.0f;
		options.maxRadius = 0.0f;
		options.ignoreObject = getObject();
		options.relationshipObject = getObject();
		if( ThePartitionManager->findPositionAround( &m_exitRallyPoint, &options, &result ) == FALSE )
			m_rallyValid = FALSE;

	}

	// if no rally point is present, try to find one
	if( m_rallyValid == FALSE )
	{
		FindPositionOptions options;

		// pick a location for everybody to rally at
		options.flags = FPF_IGNORE_ALLY_OR_NEUTRAL_UNITS;
		options.minRadius = getObject()->getGeometryInfo().getBoundingCircleRadius();
		options.maxRadius = options.minRadius * 1.8f;  // arbitrary max distance away, change as needed
		options.ignoreObject = getObject();
		options.relationshipObject = getObject();
		m_rallyValid = ThePartitionManager->findPositionAround( getObject()->getPosition(),
																													  &options,
																													  &m_exitRallyPoint );
	}

}




//-------------------------------------------------------------------------------------------------
void GarrisonContain::onSelling()
{
  removeAllContained( FALSE );
  OpenContain::onSelling();
}



// ------------------------------------------------------------------------------------------------
/** Remove all contents of this container.  We will try to do so with intelligent garrison
	* logic, but if all else fails no matter, we need to get all things out after this
	* call is complete */
// ------------------------------------------------------------------------------------------------
void GarrisonContain::removeAllContained( Bool exposeStealthUnits )
{

	//
	// we will call this when we are destroying the object (either normally or through a game
	// reset/exit etc) *and* also when we have received an "evacuate" command from the player.
	// We will attempt to find a spot for all the contents to scatter to that is around the
	// structure, but if such a spot is not found, we'll just scatter in a random direction
	// from the building.
	//

	// only even bother doing this if we have contents inside us just cause it's a waste
	if( getContainCount() > 0 )
	{

		// validate the current rally point is still a good one, or pick a new one
		validateRallyPoint();

	}

	// call the base class to extend functionality and do the actual removal
	OpenContain::removeAllContained( exposeStealthUnits );

	recalcApparentControllingPlayer();

}

// ------------------------------------------------------------------------------------------------
/** 'exitObj' is one of the things we contain, it needs to 'exit' us */
// ------------------------------------------------------------------------------------------------
void GarrisonContain::exitObjectViaDoor( Object *exitObj, ExitDoorType exitDoor )
{
	DEBUG_ASSERTCRASH(exitDoor == DOOR_1, ("multiple exit doors not supported here"));

	// We don't use the ExitPath system of the general OpenContain, we just send people out.  The
	// direction of outing has been picked by Design to be the Screen Down at the default camera angle.
	removeFromContain( exitObj );

	Coord3D startPosition;
	Coord3D endPosition;

	Real exitAngle = getObject()->getOrientation();

  // Garrison doesn't have reserveDoor or exitDelay, so if we do nothing, everyone will appear on top
	// of each other and get stuck inside each others' extent (except for the first guy).  So we'll
	// scatter the start point around a little to make it better.
	startPosition = *getObject()->getPosition();
	// In the case of cliff bunkers, the units start in a cliff.  So we want to adjust.
	AIUpdateInterface  *ai = exitObj->getAI();
	if (ai) {
		Locomotor *loco = ai->getCurLocomotor();
		if (loco && !TheAI->pathfinder()->validMovementTerrain( LAYER_GROUND, loco, &startPosition)) {
			// try front & back.
			Real offset = getObject()->getGeometryInfo().getMajorRadius();
			startPosition.x -= offset*Cos(exitAngle);
			startPosition.y -= offset*Sin(exitAngle);
			if (!TheAI->pathfinder()->validMovementTerrain(LAYER_GROUND, loco, &startPosition)) {
				startPosition.x += 2*offset*Cos(exitAngle);
				startPosition.y += 2*offset*Sin(exitAngle);
				if (!TheAI->pathfinder()->validMovementTerrain(LAYER_GROUND, loco, &startPosition)) {
					startPosition = *getObject()->getPosition();
				}
			}
		}
	}




  if ( m_evacDisposition == EVAC_TO_LEFT || m_evacDisposition == EVAC_TO_RIGHT  )
  {

    Real EVAC__SCALAR = ( m_evacDisposition == EVAC_TO_LEFT ? 1.0f : -1.0f );

    Real containerHalfLength = getObject()->getGeometryInfo().getMajorRadius() ;
    Real containerHalfWidth = getObject()->getGeometryInfo().getMinorRadius() ;

    Vector3 doorPosition;
    doorPosition.X = GameLogicRandomValueReal( -containerHalfLength/4, containerHalfLength/4 );// a rectangular pocket to act as the "doorway"
    doorPosition.Y = GameLogicRandomValueReal( containerHalfWidth/2, containerHalfWidth * 2) * EVAC__SCALAR;
    doorPosition.Z = 0;
    Vector3 walkToPosition;
    walkToPosition.X = GameLogicRandomValueReal( -containerHalfLength, containerHalfLength );
    walkToPosition.Y = containerHalfWidth * 10 * EVAC__SCALAR;// spread-out!
    walkToPosition.Z = 0;

    const Matrix3D *mtx = getObject()->getTransformMatrix();
    mtx->Transform_Vector( *mtx, doorPosition, &doorPosition );
    startPosition.x = doorPosition.X;
    startPosition.y = doorPosition.Y;
    startPosition.z = doorPosition.Z;

    mtx->Transform_Vector( *mtx, walkToPosition, &walkToPosition );
    endPosition.x = walkToPosition.X;
    endPosition.y = walkToPosition.Y;
    endPosition.z = walkToPosition.Z;

	  exitObj->setPosition( &startPosition );
	  exitObj->setOrientation( exitAngle );

	  ///< @todo This really should be automatically wrapped up in an activation sequence	for objects in general
	  // tell the AI about it
	  TheAI->pathfinder()->addObjectToPathfindMap( exitObj );
	  if( ai )
	  {
		  TheAI->pathfinder()->adjustToPossibleDestination(exitObj, ai->getLocomotorSet(), &endPosition);
		  std::vector<Coord3D> exitPath;
		  exitPath.push_back(endPosition);

		  ai->aiFollowPath( &exitPath, getObject(), CMD_FROM_AI );
		  TheAI->pathfinder()->updateGoal(exitObj, &endPosition, TheTerrainLogic->getLayerForDestination(&endPosition));
	  }

  }
  else // must be EVAC_BURST_FROM_CENTER. then!
  {
    // if we are not enclosed, then just walk away from where we "are."
  	if ( isEnclosingContainerFor( exitObj ))
    {
      exitObj->setPosition( &startPosition ); // correct for non-ground-level station points
      exitObj->setPositionZ( TheTerrainLogic->getGroundHeight( startPosition.x, startPosition.y ) );
    }

    exitObj->setOrientation( exitAngle );
	  ///< @todo This really should be automatically wrapped up in an activation sequence	for objects in general
	  // tell the AI about it
	  TheAI->pathfinder()->addObjectToPathfindMap( exitObj );
	  endPosition = startPosition;
	  if( ai )
	  {
		  TheAI->pathfinder()->adjustToPossibleDestination(exitObj, ai->getLocomotorSet(), &endPosition);
		  std::vector<Coord3D> exitPath;
		  exitPath.push_back(endPosition);

		  ai->aiFollowPath( &exitPath, getObject(), CMD_FROM_AI );
		  TheAI->pathfinder()->updateGoal(exitObj, &endPosition, TheTerrainLogic->getLayerForDestination(&endPosition));
	  }
  }



	recalcApparentControllingPlayer();
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void GarrisonContain::onContaining( Object *obj, Bool wasSelected )
{

	// extend base class
	OpenContain::onContaining( obj, wasSelected );

	// get the structure object
	Object *structure = getObject();

	// objects inside a building are held
	obj->setDisabled( DISABLED_HELD );

	// the building can now attack, since it has soldiers inside of it
	structure->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_CAN_ATTACK ) );

	// give the object a garrisoned version of its weapon
	obj->setWeaponBonusCondition( WEAPONBONUSCONDITION_GARRISONED );

	// put the object in the center of the building
  if (isEnclosingContainerFor( obj ))
	  obj->setPosition( structure->getPosition() );

	obj->getControllingPlayer()->getAcademyStats()->recordBuildingGarrisoned();

	//
	// the team of the building is now the same as those that have garrisoned it, be sure
	// to save our original team tho so that we can revert back to it when all the
	// occupants are gone
	//
	recalcApparentControllingPlayer();

  Drawable *draw = obj->getDrawable();
  if ( draw && draw->isSelected() )
    TheInGameUI->deselectDrawable( draw );


}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void GarrisonContain::onRemoving( Object *obj )
{
	OpenContain::onRemoving(obj);


  if (isEnclosingContainerFor( obj ))
	  // first remove the object from any garrison fire point if it's at one
  	removeObjectFromGarrisonPoint( obj );
  else
  {
    removeObjectFromStationPoint( obj );
		//Kris: Patch 1.01 -- Passing in correct argument for Y (instead of X) fixes cases where selling firebases
		//were dropping contained infantry to incorrect altitudes.
    obj->setPositionZ( TheTerrainLogic->getGroundHeight( obj->getPosition()->x, obj->getPosition()->y ) );
  }
	// give the object back a regular weapon
	obj->clearWeaponBonusCondition( WEAPONBONUSCONDITION_GARRISONED );

	// object is no longer held inside a garrisoned building
	obj->clearDisabled( DISABLED_HELD );

	// if we have nothing left inside of us then we are once again back to our original team
	if( getContainCount() == 0 )
	{

		// put us back on our original team
		// (hokey exception: if our team is null, don't bother -- this
		// usually means we are being called during game-teardown and
		// the teams are no longer valid...)
		if (getObject()->getTeam() != nullptr)
		{
			getObject()->setTeam( m_originalTeam );
			m_originalTeam = nullptr;
		}

		// we also lose our transient attack ability
		getObject()->clearStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_CAN_ATTACK ) );
		m_hideGarrisonedStateFromNonallies = false;

		// change the state back from garrisoned
		Drawable *draw = getObject()->getDrawable();
		if( draw )
		{
			draw->clearModelConditionState( MODELCONDITION_GARRISONED );
		}

	}
	else if( getStealthUnitsContained() != getContainCount() )
	{
		m_hideGarrisonedStateFromNonallies = false;
	}

	// disable occlusion while the unit walks out of the building
	///@todo: we should probably not draw the unit to begin with and just have them pop out.
	obj->setSafeOcclusionFrame(TheGameLogic->getFrame()+obj->getTemplate()->getOcclusionDelay());

	recalcApparentControllingPlayer();
}

// ------------------------------------------------------------------------------------------------
/** A GarrisonContain always lets people shoot out */
// ------------------------------------------------------------------------------------------------
Bool GarrisonContain::isPassengerAllowedToFire( ObjectID id ) const
{

  const Object *self = getObject();
  if ( self && self->isDisabledByType( DISABLED_SUBDUED ) )
    return FALSE;

	return TRUE;

}

// ------------------------------------------------------------------------------------------------
/** A Mobile garrison keeps its occupants with it when it moves */
//-------------------------------------------------------------------------------------------------
void GarrisonContain::moveObjectsWithMe()
{
	const GarrisonContainModuleData *modData = getGarrisonContainModuleData();

	if (!modData->m_mobileGarrison)
		return;

	const ContainedItemsList& containList = getContainList();
	for( ContainedItemsList::const_iterator it = containList.begin(); it != containList.end(); ++it )
	{
		Object *obj;
		// get the object
		obj = *it;
		obj->setPosition(getObject()->getPosition());
	}
}

//-------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void GarrisonContain::onBodyDamageStateChange( const DamageInfo* , BodyDamageType , BodyDamageType newState )
{
	// This is an event triggered on edge, so we just need to look at the newState. We know a change happened.
	// And we don't need to check Rubble, since death exiting is already handled. This is Garrison specific.
	if( newState == BODY_REALLYDAMAGED && !getObject()->isKindOf( KINDOF_GARRISONABLE_UNTIL_DESTROYED ) )
	{
		if( getContainCount() > 0 )
			orderAllPassengersToExit( CMD_FROM_AI, FALSE );
	}
}

//-------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void GarrisonContain::onObjectCreated()
{
	GarrisonContainModuleData* self = (GarrisonContainModuleData*)getGarrisonContainModuleData();

	Int count = self->m_initialRoster.count;
	const ThingTemplate* rosterTemplate = TheThingFactory->findTemplate( self->m_initialRoster.templateName );
	Object* object = getObject();

	for( int i = 0; i < count; i++ )
	{
		//We are creating a garrison that comes with an initial roster, so add it now!
		Object* payload = TheThingFactory->newObject( rosterTemplate, object->getControllingPlayer()->getDefaultTeam() );
		if( object->getContain() && object->getContain()->isValidContainerFor( payload, true ) )
		{
			object->getContain()->addToContain( payload );
		}
		else
		{
			DEBUG_CRASH( ( "DeliverPayload: PutInContainer %s is full, or not valid for the payload %s!",
				object->getName().str(), self->m_initialRoster.templateName.str() ) );
		}
	}
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void GarrisonContain::crc( Xfer *xfer )
{

	// extend base class
	OpenContain::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void GarrisonContain::xfer( Xfer *xfer )
{
	Int i;

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	OpenContain::xfer( xfer );

	// original team
	TeamID teamID = m_originalTeam ? m_originalTeam->getID() : TEAM_ID_INVALID;
	xfer->xferUser( &teamID, sizeof( TeamID ) );
	if( xfer->getXferMode() == XFER_LOAD )
	{

		if( teamID != TEAM_ID_INVALID )
		{

			m_originalTeam = TheTeamFactory->findTeamByID( teamID );
			if( m_originalTeam == nullptr )
			{

				DEBUG_CRASH(( "GarrisonContain::xfer - Unable to find original team by id" ));
				throw SC_INVALID_DATA;

			}

		}
		else
			m_originalTeam = nullptr;

	}

	xfer->xferBool( &m_hideGarrisonedStateFromNonallies );

	// garrison point data
	UnsignedShort pointDataCount = MAX_GARRISON_POINTS;
	xfer->xferUnsignedShort( &pointDataCount );
	for( i = 0; i < pointDataCount; ++i )
	{

		if( xfer->getXferMode() == XFER_SAVE )
		{

			// object at this point
			Object *obj = m_garrisonPointData[ i ].object;
			ObjectID objectID = obj ? obj->getID() : INVALID_ID;
			xfer->xferObjectID( &objectID );

			// target
			xfer->xferObjectID( &m_garrisonPointData[ i ].targetID );

			// placement frame
			xfer->xferUnsignedInt( &m_garrisonPointData[ i ].placeFrame );

			// last effect frame
			xfer->xferUnsignedInt( &m_garrisonPointData[ i ].lastEffectFrame );

			// effect drawable id
			Drawable *draw = m_garrisonPointData[ i ].effect;
			DrawableID drawableID = draw ? draw->getID() : INVALID_DRAWABLE_ID;
			xfer->xferDrawableID( &drawableID );

		}
		else
		{

			// objectID
			ObjectID objectID;
			xfer->xferObjectID( &objectID );

			// target
			ObjectID targetID;
			xfer->xferObjectID( &targetID );

			// placement frame
			UnsignedInt placeFrame;
			xfer->xferUnsignedInt( &placeFrame );

			// last effect frame
			UnsignedInt lastEffectFrame;
			xfer->xferUnsignedInt( &lastEffectFrame );

			// effect drawable id
			DrawableID drawableID;
			xfer->xferDrawableID( &drawableID );

			// store
			if( i < MAX_GARRISON_POINTS )
			{

				m_garrisonPointData[ i ].objectID = objectID;
				m_garrisonPointData[ i ].targetID = targetID;
				m_garrisonPointData[ i ].placeFrame = placeFrame;
				m_garrisonPointData[ i ].lastEffectFrame = lastEffectFrame;
				m_garrisonPointData[ i ].effectID = drawableID;

			}

		}

	}

	// garrison points in use
	xfer->xferInt( &m_garrisonPointsInUse );

	// garrison points
	xfer->xferUser( m_garrisonPoint, sizeof( Coord3D ) * MAX_GARRISON_POINT_CONDITIONS * MAX_GARRISON_POINTS );

	// garrison points initialized
	xfer->xferBool( &m_garrisonPointsInitialized );

	// rally valid
	xfer->xferBool( &m_rallyValid );

	// exit rally point
	xfer->xferCoord3D( &m_exitRallyPoint );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void GarrisonContain::loadPostProcess()
{

	// extend base class
	OpenContain::loadPostProcess();

	// connect up pointers needed
	for( Int i = 0; i < MAX_GARRISON_POINTS; i++ )
	{

		// object pointer
		if( m_garrisonPointData[ i ].objectID != INVALID_ID )
		{

			m_garrisonPointData[ i ].object = TheGameLogic->findObjectByID( m_garrisonPointData[ i ].objectID );
			if( m_garrisonPointData[ i ].object == nullptr )
			{

				DEBUG_CRASH(( "GarrisonContain::loadPostProcess - Unable to find object for point data" ));
				throw SC_INVALID_DATA;

			}

		}
		else
			m_garrisonPointData[ i ].object = nullptr;

		// drawable effect pointer
		if( m_garrisonPointData[ i ].effectID != INVALID_ID )
		{

			m_garrisonPointData[ i ].effect = TheGameClient->findDrawableByID( m_garrisonPointData[ i ].effectID );
			if( m_garrisonPointData[ i ].effect == nullptr )
			{

				DEBUG_CRASH(( "GarrisonContain::loadPostProcess - Unable to find effect for point data" ));
				throw SC_INVALID_DATA;

			}

		}
		else
			m_garrisonPointData[ i ].effect = nullptr;

	}

}





// ------------------------------------------------------------------------------------------------
/** Load the loadStationGarrisonPoints data and save for use later */
// ------------------------------------------------------------------------------------------------
void GarrisonContain::loadStationGarrisonPoints()
{
	const GarrisonContainModuleData *modData = getGarrisonContainModuleData();

	Object *structure = getObject();
	Bool stationBonesFound = FALSE;


	//
	// in order to get all the station point positions we will actually switch the model
	// condition to garrisoned pristine, and use these for any modelcondition
	{
		Int conditionIndex;
		Int count = 0;

		// save the original parameters for the model condition
		Drawable* draw = structure->getDrawable();
		const ModelConditionFlags originalFlags = draw->getModelConditionFlags();
		ModelConditionFlags clearFlags;
		ModelConditionFlags setFlags;

		// pristine garrisoned
		clearFlags.clear();
		setFlags.clear();
		clearFlags.set( MODELCONDITION_REALLY_DAMAGED );
		clearFlags.set( MODELCONDITION_RUBBLE );
		clearFlags.set( MODELCONDITION_SPECIAL_DAMAGED );
		clearFlags.set( MODELCONDITION_DAMAGED );
		setFlags.set( MODELCONDITION_GARRISONED );
		structure->clearAndSetModelConditionFlags( clearFlags, setFlags );
		conditionIndex = GARRISON_POINT_PRISTINE;


    Coord3D tempBuffer[MAX_GARRISON_POINTS];
		int t = 0;
		for( ; t < MAX_GARRISON_POINTS; ++t )
		  tempBuffer[ t ] = *(structure->getPosition());

		count = structure->getMultiLogicalBonePosition("STATION", modData->m_containMax, tempBuffer, nullptr);
		if ( count > 0) stationBonesFound = TRUE;


    m_stationPointList.clear();// we are starting over... forget everything

    for( t = 0; t < count; ++t )
    {
      StationPointData tempStationPointData;
      tempStationPointData.position = tempBuffer[ t ];
      tempStationPointData.occupantID = INVALID_ID;
      m_stationPointList.push_back( tempStationPointData ); // store for later use
    }
		// restore the original condition flags
		draw->replaceModelConditionFlags( originalFlags );

    //tempBuffer pops

	}

	// garrison points are now initialized
	m_stationGarrisonPointsInitialized = TRUE;



	if (stationBonesFound && modData->m_mobileGarrison && (getObject()->isMobile() == TRUE) )
	{
		DEBUG_ASSERTCRASH( getObject()->isMobile() == FALSE,
		 ("GarrisonContain::update - You have specified this garrisonContain as mobile,\n yet you want station garrison point placement bones... \n what are you thinking?") );
	}

}
```
