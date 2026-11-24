# Translation Packet `PKT-284`: `OverlordContain`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-284`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/contain/overlord_contain.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (14):**
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/ThingFactory.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ThingFactory.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/ControlBar.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/ControlBar.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameLogic/ExperienceTracker.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ExperienceTracker.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/OverlordContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/OverlordContain.h`
  - `GameLogic/Module/TransportContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/TransportContain.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `PreRTS.h`
- **Implemented Module Facets (1):**
  - `ExitInterface* getContainExitInterface()`
- **INI `FieldParse` Table Entries (2):**
  - `"PayloadTemplateName" => INI::parseAsciiStringVectorAppend`
  - `"ExperienceSinkForRider" => INI::parseBool`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (2 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferBool( &m_redirectionActivated )`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/OverlordContain.h`
```cpp
// FILE: OverlordContain.h ////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, September, 2002
// Desc:   Contain module that acts as transport normally, but when full it redirects queries to the first passenger
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/TransportContain.h"
#include "GameLogic/GameLogic.h"


typedef std::vector<AsciiString> TemplateNameList;
typedef std::vector<AsciiString>::const_iterator TemplateNameIterator;

//-------------------------------------------------------------------------------------------------
class OverlordContainModuleData : public TransportContainModuleData
{
public:

	OverlordContainModuleData();

	TemplateNameList m_payloadTemplateNameData;
	Bool m_experienceSinkForRider;


	static void buildFieldParse(MultiIniFieldParse& p);
	static void parseInitialPayload( INI* ini, void *instance, void *store, const void* /*userData*/ );
};

//-------------------------------------------------------------------------------------------------
class OverlordContain : public TransportContain
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( OverlordContain, "OverlordContain" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( OverlordContain, OverlordContainModuleData )

	virtual void onBodyDamageStateChange( const DamageInfo* damageInfo,
																				BodyDamageType oldState,
																				BodyDamageType newState) override;  ///< state change callback
public:

	OverlordContain( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	virtual OpenContain *asOpenContain() override { return this; }  ///< treat as open container
	virtual Bool isGarrisonable() const override;	///< can this unit be Garrisoned? (ick)
  virtual Bool isBustable() const override { return false;};	///< can this container get busted by bunkerbuster? (ick)
	virtual Bool isHealContain() const override { return false; } ///< true when container only contains units while healing (not a transport!)
	virtual Bool isTunnelContain() const override { return FALSE; }
	virtual Bool isImmuneToClearBuildingAttacks() const override { return true; }
  virtual Bool isSpecialOverlordStyleContainer() const override {return TRUE;}
	virtual Bool isPassengerAllowedToFire( ObjectID id = INVALID_ID ) const override;	///< Hey, can I shoot out of this container?
	virtual Bool isSpecificRiderFreeToExit(Object* obj) override;
	virtual void exitObjectViaDoor(Object* exitObj, ExitDoorType exitDoor) override;
	// GeneralsX @bugfix UnicodeApocalypse 12/09/2026 Redirects exit queries to the loaded portable.
	virtual ExitInterface* getContainExitInterface() override;


	virtual void onDie( const DamageInfo *damageInfo ) override;  ///< the die callback
	virtual void onDelete() override;	///< Last possible moment cleanup
	virtual void onCapture( Player *oldOwner, Player *newOwner ) override; // Our main guy goes with us, but our redirected contain needs to do his thing too
	virtual void onObjectCreated() override;

	// Contain stuff we need to override to redirect on a condition
	virtual void onContaining( Object *obj, Bool wasSelected ) override;		///< object now contains 'obj'
	virtual void onRemoving( Object *obj ) override;			///< object no longer contains 'obj'
	virtual UpdateSleepTime update() override;
	virtual void containReactToTransformChange() override;
	// GeneralsX @bugfix copilot 20/04/2026 Issue #95: skip redeployOccupants for PORTABLE_STRUCTURE riders
	virtual void redeployOccupants() override;

private:
	void syncPortablePosition();  ///< Sync portable rider position/orientation to the host Overlord.


	virtual Bool isValidContainerFor(const Object* obj, Bool checkCapacity) const override;
	// GeneralsX @bugfix UnicodeApocalypse 12/09/2026 Redirects containment checks to the loaded portable.
	virtual Bool isContained( const Object *obj ) const override;
	virtual void addToContain( Object *obj ) override;				///< add 'obj' to contain list
	virtual void addToContainList( Object *obj ) override;		///< The part of AddToContain that inheritors can override (Can't do whole thing because of all the private stuff involved)
	virtual void removeFromContain( Object *obj, Bool exposeStealthUnits = FALSE ) override;	///< remove 'obj' from contain list
	virtual void removeAllContained( Bool exposeStealthUnits = FALSE ) override;				///< remove all objects on contain list
	virtual Bool isEnclosingContainerFor( const Object *obj ) const override;	///< Does this type of Contain Visibly enclose its contents?
	virtual Bool isDisplayedOnControlBar() const override;///< Does this container display its contents on the ControlBar?
	virtual Bool isKickOutOnCapture() override;// The bunker may want to, but we certainly don't
	virtual void killAllContained() override;				///< kill all objects inside.  For us, this does not mean our rider

	// contain list access
	virtual void iterateContained( ContainIterateFunc func, void *userData, Bool reverse ) override;
	virtual UnsignedInt getContainCount() const override;
	virtual Int getContainMax() const override;
	virtual const ContainedItemsList* getContainedItemsList() const override;

	// Friend for our Draw module only.
	virtual const Object *friend_getRider() const override; ///< Damn.  The draw order dependency bug for riders means that our draw module needs to cheat to get around it.

	///< if my object gets selected, then my visible passengers should, too
	///< this gets called from
	virtual void clientVisibleContainedFlashAsSelected() override;

	virtual Bool getContainerPipsToShow(Int& numTotal, Int& numFull) override;
	virtual void createPayload() override;

private:
	/**< An empty overlord is a container, but a full one redirects calls to its passengers.  If this returns null,
	we are either empty or carrying a non container.
	*/
	ContainModuleInterface *getRedirectedContain() const; ///< And this gets what are redirecting to.
	void activateRedirectedContain();///< I need to shut this off since I can talk directly to my bunker, but he can never directly see me
	void deactivateRedirectedContain();
  void parseInitialPayload( INI* ini, void *instance, void *store, const void* /*userData*/ );

	Bool m_redirectionActivated;

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Contain/OverlordContain.cpp`
```cpp
// FILE: OverlordContain.cpp ////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, September, 2002
// Desc:   Contain module that acts as transport normally, but when full it redirects queries to the first passenger
// All of this redirection stuff makes it so that while I am normally a transport
// for Overlord subObjects, once I have a passenger, _I_ become a transport of their type.
// So, the answer to this question depends on if it is my passenger asking, or theirs.
// As always, I can't use convenience functions that get redirected on a ? like this.

///////////////////////////////////////////////////////////////////////////////////////////////////

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine
#include "Common/Player.h"
#include "Common/Xfer.h"
#include "Common/ThingTemplate.h"
#include "Common/ThingFactory.h"
#include "GameClient/ControlBar.h"
#include "GameClient/Drawable.h"
#include "GameLogic/ExperienceTracker.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/OverlordContain.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"

 

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
OverlordContainModuleData::OverlordContainModuleData()
{
//	m_initialPayload.count = 0;
	m_experienceSinkForRider = TRUE;
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void OverlordContainModuleData::buildFieldParse(MultiIniFieldParse& p)
{
  TransportContainModuleData::buildFieldParse(p);

	static const FieldParse dataFieldParse[] =
	{
    { "PayloadTemplateName",  INI::parseAsciiStringVectorAppend, nullptr, offsetof(OverlordContainModuleData, m_payloadTemplateNameData) },
    { "ExperienceSinkForRider",  INI::parseBool, nullptr, offsetof(OverlordContainModuleData, m_experienceSinkForRider) },

		{ nullptr, nullptr, nullptr, 0 }
	};
  p.add(dataFieldParse);
}
// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
//void OverlordContainModuleData::parseInitialPayload( INI* ini, void *instance, void *store, const void* /*userData*/ )
//{
//	OverlordContainModuleData* self = (OverlordContainModuleData*)instance;
//	const char* name = ini->getNextToken();
//	const char* countStr = ini->getNextTokenOrNull();
//	Int count = countStr ? INI::scanInt(countStr) : 1;

//	self->m_initialPayload.name.set(name);
//	self->m_initialPayload.count = count;
//}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
OverlordContain::OverlordContain( Thing *thing, const ModuleData *moduleData ) :
								 TransportContain( thing, moduleData )
{
	m_redirectionActivated = FALSE;

  m_payloadCreated = FALSE;

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
OverlordContain::~OverlordContain()
{

}


void OverlordContain::onObjectCreated()
{
  OverlordContain::createPayload();
}

void OverlordContain::syncPortablePosition()
{
	Object* portable = (m_containListSize > 0) ? m_containList.front() : nullptr;
	if (portable && portable->isKindOf(KINDOF_PORTABLE_STRUCTURE))
	{
		// GeneralsX @bugfix copilot 19/04/2026 Keep Overlord portable structures spatially synced with the host tank.
		portable->setPosition(getObject()->getPosition());
		portable->setOrientation(getObject()->getOrientation());
	}
}

UpdateSleepTime OverlordContain::update()
{
	syncPortablePosition();
	return TransportContain::update();
}

void OverlordContain::containReactToTransformChange()
{
	// Let the base class run redeployOccupants() (bone-based placement).
	OpenContain::containReactToTransformChange();
	// GeneralsX @bugfix copilot 19/04/2026 Immediately correct portable position after transform;
	// bone queries return wrong world coords on POSIX, so override with the host tank's position.
	syncPortablePosition();
}

// GeneralsX @bugfix copilot 19/04/2026 Prevent portable structures from ever exiting the Overlord.
// On Windows/VC6, DISABLED_HELD caused getCurLocomotor() to return null, blocking exit naturally.
// On POSIX the locomotor is still present even when HELD, so portables could exit when commanded
// (e.g., force-attack). Portables are permanent upgrades and must never leave the Overlord.
Bool OverlordContain::isSpecificRiderFreeToExit(Object* obj)
{
	if (obj && obj->isKindOf(KINDOF_PORTABLE_STRUCTURE))
		return FALSE;
	return TransportContain::isSpecificRiderFreeToExit(obj);
}

// GeneralsX @bugfix copilot 19/04/2026 Block instant-exit path for portable structures.
// AIExitInstantlyState::onEnter() calls exitObjectViaDoor(obj, DOOR_1) directly, bypassing
// reserveDoorForExit and isSpecificRiderFreeToExit entirely. The Gatling Cannon enters
// AI_EXIT_INSTANTLY because it has attack weapons and its attack state machine transitions
// through CHASE_TARGET (ContinueState) -> EXIT_MACHINE_WITH_FAILURE -> AI_EXIT_INSTANTLY
// when force-attacking empty terrain. Propaganda Tower never hits this path (no weapons).
// GeneralsX @bugfix copilot 20/04/2026 Issue #95: redeployOccupants calls putObjAtNextFirePoint on
// PORTABLE_STRUCTURE riders which triggers Object::reactToTransformChange and destroys them.
// The portable position is managed exclusively by syncPortablePosition().
// Non-portable occupants (if any) are still redeployed via the parent.
void OverlordContain::redeployOccupants()
{
	// In practice the Overlord's own contain list only ever has a single portable upgrade,
	// so this is effectively a no-op — but guarded correctly for safety.
	bool hasNonPortable = false;
	const ContainedItemsList& list = getContainList();
	for (ContainedItemsList::const_iterator it = list.begin(); it != list.end(); ++it)
	{
		Object* obj = *it;
		if (obj && !obj->isKindOf(KINDOF_PORTABLE_STRUCTURE))
		{
			hasNonPortable = true;
			break;
		}
	}
	if (hasNonPortable)
		TransportContain::redeployOccupants();
	// Portables are repositioned by syncPortablePosition() called from containReactToTransformChange().
}

void OverlordContain::exitObjectViaDoor(Object* exitObj, ExitDoorType exitDoor)
{
	if (exitObj && exitObj->isKindOf(KINDOF_PORTABLE_STRUCTURE))
		return;
	TransportContain::exitObjectViaDoor(exitObj, exitDoor);
}

// GeneralsX @bugfix UnicodeApocalypse 12/09/2026 Without this, exit/evacuate silently no-ops.
ExitInterface* OverlordContain::getContainExitInterface()
{
	if( getRedirectedContain() == nullptr )
		return OpenContain::getContainExitInterface();

	return getRedirectedContain()->getContainExitInterface();
}

//-------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void OverlordContain::createPayload()
{
	OverlordContainModuleData* self = (OverlordContainModuleData*)getOverlordContainModuleData();


  // Any number of different passengers can be loaded here at init time
	Object* object = getObject();
	ContainModuleInterface *contain = object->getContain();
	if( contain )
  {
		contain->enableLoadSounds( FALSE );

	  TemplateNameList list = self->m_payloadTemplateNameData;
	  TemplateNameIterator iter = list.begin();
	  while ( iter != list.end() )
	  {
		  const ThingTemplate* temp = TheThingFactory->findTemplate( *iter );
		  if (temp)
		  {
			  Object* payload = TheThingFactory->newObject( temp, object->getTeam() );

			  if( contain->isValidContainerFor( payload, true ) )
			  {
				  contain->addToContain( payload );
			  }
			  else
			  {
				  DEBUG_CRASH( ( "OverlordContain::createPayload: %s is full, or not valid for the payload %s!", object->getName().str(), self->m_initialPayload.name.str() ) );
			  }

      }

      ++iter;
    }

		contain->enableLoadSounds( TRUE );

  }

	m_payloadCreated = TRUE;

}

// ------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void OverlordContain::onBodyDamageStateChange( const DamageInfo* damageInfo,
																				BodyDamageType oldState,
																				BodyDamageType newState)  ///< state change callback
{
	// I can't use any convenience functions, as they will all get routed to the bunker I may carry.
	// I want just me.
	// Oh, and I don't want this function trying to do death.  That is more complicated and will be handled
	// on my death.
	if( newState != BODY_RUBBLE  &&  m_containListSize == 1 )
	{
		Object *myGuy = m_containList.front();
		myGuy->getBodyModule()->setDamageState( newState );
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ContainModuleInterface *OverlordContain::getRedirectedContain() const
{
	// Naturally, I can not use a redirectible convenience function
	// to answer if I am redirecting yet.

	// If I am empty, say no.
	if( m_containListSize < 1 )
		return nullptr;

	if( !m_redirectionActivated )
		return nullptr;// Shut off early to allow death to happen without my bunker having
	// trouble finding me to say goodbye as messages get sucked up the pipe to him.

	Object *myGuy = m_containList.front();
	if( myGuy )
		return myGuy->getContain();

	return nullptr;// Or say no if they have no contain.
}


//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------



//-------------------------------------------------------------------------------------------------
void OverlordContain::onDie( const DamageInfo *damageInfo )
{
	// Do you mean me the Overlord, or my behavior of passing stuff on to my passengers?
	if( getRedirectedContain() == nullptr )
	{
		TransportContain::onDie( damageInfo );
		return;
	}
	//Everything is fine if I am empty or carrying a regular guy.  If I have a redirected contain
	// set up, then I need to handle the order of death explicitly, or things will become confused
	// when I stop redirecting in the middle of the process.  Or I will get confused as my commands
	// get sucked up the pipe.

	// So this is an extend that lets me control the order of death.

	deactivateRedirectedContain();
	Object *myGuy = m_containList.front();
	myGuy->kill();

	TransportContain::onDie( damageInfo );
}

//-------------------------------------------------------------------------------------------------
void OverlordContain::onDelete()
{
	// Do you mean me the Overlord, or my behavior of passing stuff on to my passengers?
	if( getRedirectedContain() == nullptr )
	{
		TransportContain::onDelete();
		return;
	}

	// Without my throwing the redirect switch, teardown deletion will get confused and fire off a bunch of asserts
	getRedirectedContain()->removeAllContained();

	deactivateRedirectedContain();
	removeAllContained();

	TransportContain::onDelete();
}

// ------------------------------------------------------------------------------------------------
void OverlordContain::onCapture( Player *oldOwner, Player *newOwner )
{
	if( m_containListSize < 1 )
		return;

	// Need to capture our specific rider.  He will then kick passengers out if he is a Transport
	Object *myGuy = m_containList.front();
	myGuy->setTeam( newOwner->getDefaultTeam() );
}

//-------------------------------------------------------------------------------------------------
Bool OverlordContain::isGarrisonable() const
{
	if( getRedirectedContain() == nullptr )
		return FALSE;

	return getRedirectedContain()->isGarrisonable();
}

//-------------------------------------------------------------------------------------------------
Bool OverlordContain::isKickOutOnCapture()
{
	if( getRedirectedContain() == nullptr )
		return FALSE;// Me the Overlord doesn't want to

	return getRedirectedContain()->isKickOutOnCapture();
}

//-------------------------------------------------------------------------------------------------
void OverlordContain::addToContainList( Object *obj )
{
	// Do you mean me the Overlord, or my behavior of passing stuff on to my passengers?
	if( getRedirectedContain() == nullptr )
	{
		TransportContain::addToContainList( obj );
		return;
	}

	getRedirectedContain()->addToContainList( obj );
}

//-------------------------------------------------------------------------------------------------
void OverlordContain::addToContain( Object *obj )
{
	// Do you mean me the Overlord, or my behavior of passing stuff on to my passengers?
	if( getRedirectedContain() == nullptr )
	{
		TransportContain::addToContain( obj );
		return;
	}

	getRedirectedContain()->addToContain( obj );

}

//-------------------------------------------------------------------------------------------------
/** Remove 'obj' from the m_containList of objects in this module.
	* This will trigger an onRemoving event for the object that this module
	* is a part of and an onRemovedFrom event for the object being removed */
//-------------------------------------------------------------------------------------------------
void OverlordContain::removeFromContain( Object *obj, Bool exposeStealthUnits )
{
	// Do you mean me the Overlord, or my behavior of passing stuff on to my passengers?
	if( getRedirectedContain() == nullptr )
	{
		TransportContain::removeFromContain( obj, exposeStealthUnits );
		return;
	}

	getRedirectedContain()->removeFromContain( obj, exposeStealthUnits );

}

//-------------------------------------------------------------------------------------------------
/** Remove all contained objects from the contained list */
//-------------------------------------------------------------------------------------------------
void OverlordContain::removeAllContained( Bool exposeStealthUnits )
{
	// Do you mean me the Overlord, or my behavior of passing stuff on to my passengers?
	if( getRedirectedContain() == nullptr )
	{
		TransportContain::removeAllContained( exposeStealthUnits );
		return;
	}

	const ContainedItemsList *fullList = getRedirectedContain()->getContainedItemsList();

	Object *obj;
	ContainedItemsList::const_iterator it;
	it = (*fullList).begin();
	while( it != (*fullList).end() )
	{
		obj = *it;
		it++;
		removeFromContain( obj, exposeStealthUnits );
	}
}

//-------------------------------------------------------------------------------------------------
/** Iterate the contained list and call the callback on each of the objects */
//-------------------------------------------------------------------------------------------------
void OverlordContain::iterateContained( ContainIterateFunc func, void *userData, Bool reverse )
{
	// Do you mean me the Overlord, or my behavior of passing stuff on to my passengers?
	if( getRedirectedContain() == nullptr )
	{
		TransportContain::iterateContained( func, userData, reverse );
		return;
	}

	getRedirectedContain()->iterateContained( func, userData, reverse );
}

//-------------------------------------------------------------------------------------------------
void OverlordContain::onContaining( Object *obj, Bool wasSelected )
{
	// Do you mean me the Overlord, or my behavior of passing stuff on to my passengers?
	if( getRedirectedContain() == nullptr )
	{
		TransportContain::onContaining( obj, wasSelected );

    if ( obj->isKindOf( KINDOF_PORTABLE_STRUCTURE ) )
    {
  		activateRedirectedContain();//Am now carrying something

			// And this contain style explicitly sucks XP from our little friend.
			if( getOverlordContainModuleData()->m_experienceSinkForRider  &&  obj->getExperienceTracker() )
				obj->getExperienceTracker()->setExperienceSink(getObject()->getID());


      if ( obj->isKindOf( KINDOF_PORTABLE_STRUCTURE ) && getObject()->testStatus( OBJECT_STATUS_STEALTHED ) )
      {
        StealthUpdate *myStealth =  obj->getStealth();
        if ( myStealth )
        {
          myStealth->receiveGrant( true );
          // note to anyone... once stealth is granted to this gattlingcannon ( or such )
          // let its own stealthupdate govern the allowedtostealth cases
          // a portable structure never gets removed, so...
        }
      }




    }


    return;
	}

	OpenContain::onContaining( obj, wasSelected );

	getRedirectedContain()->onContaining( obj, wasSelected );

}

//-------------------------------------------------------------------------------------------------
void OverlordContain::killAllContained()
{
	// This is a game call meant to clear actual passengers.  We don't want it to kill our turret.  That'd be weird.
	if( getRedirectedContain() )
	{
		getRedirectedContain()->killAllContained();
	}
}

//-------------------------------------------------------------------------------------------------
void OverlordContain::onRemoving( Object *obj )
{
	// Do you mean me the Overlord, or my behavior of passing stuff on to my passengers?
	if( getRedirectedContain() == nullptr )
	{
		TransportContain::onRemoving( obj );
		return;
	}

	OpenContain::onRemoving(obj);

	getRedirectedContain()->onRemoving( obj );

}

//-------------------------------------------------------------------------------------------------
Bool OverlordContain::isValidContainerFor(const Object* obj, Bool checkCapacity) const
{
	// Do you mean me the Overlord, or my behavior of passing stuff on to my passengers?
	if( getRedirectedContain() == nullptr )
		return TransportContain::isValidContainerFor( obj, checkCapacity );

	return getRedirectedContain()->isValidContainerFor( obj, checkCapacity );
}

// GeneralsX @bugfix UnicodeApocalypse 12/09/2026 Without this, privateExit's isContained() guard blocks all exits.
Bool OverlordContain::isContained( const Object *obj ) const
{
	if( getRedirectedContain() == nullptr )
		return OpenContain::isContained( obj );

	return getRedirectedContain()->isContained( obj );
}

//-------------------------------------------------------------------------------------------------
UnsignedInt OverlordContain::getContainCount() const
{
	ContainModuleInterface* redir = getRedirectedContain();

	// Do you mean me the Overlord, or my behavior of passing stuff on to my passengers?
	if( redir == nullptr )
		return TransportContain::getContainCount();

	return redir->getContainCount();
}

//-------------------------------------------------------------------------------------------------
Bool OverlordContain::getContainerPipsToShow(Int& numTotal, Int& numFull)
{
	// Do you mean me the Overlord, or my behavior of passing stuff on to my passengers?
	if( getRedirectedContain() == nullptr )
	{
		numTotal = 0;
		numFull = 0;
		return false;
	}
	else
	{
		return getRedirectedContain()->getContainerPipsToShow(numTotal, numFull);
	}
}

//-------------------------------------------------------------------------------------------------
Int OverlordContain::getContainMax() const
{
	// Do you mean me the Overlord, or my behavior of passing stuff on to my passengers?
	if( getRedirectedContain() == nullptr )
		return TransportContain::getContainMax();

	return getRedirectedContain()->getContainMax();
}

//-------------------------------------------------------------------------------------------------
const ContainedItemsList* OverlordContain::getContainedItemsList() const
{
	// Do you mean me the Overlord, or my behavior of passing stuff on to my passengers?
	if( getRedirectedContain() == nullptr )
		return TransportContain::getContainedItemsList();

	return getRedirectedContain()->getContainedItemsList();
}

//-------------------------------------------------------------------------------------------------
Bool OverlordContain::isEnclosingContainerFor( const Object *obj ) const
{
	// All of this redirection stuff makes it so that while I am normally a transport
	// for Overlord subObjects, once I have a passenger, _I_ become a transport of their type.
	// So, the answer to this question depends on if it is my passenger asking, or theirs.
	// As always, I can't use convenience functions that get redirected on a ? like this.
	if( m_containListSize > 0  &&  obj ==  m_containList.front() )
		return FALSE;

	return TRUE;
}

//-------------------------------------------------------------------------------------------------
Bool OverlordContain::isDisplayedOnControlBar() const
{
	// Do you mean me the Overlord, or my behavior of passing stuff on to my passengers?
	if( getRedirectedContain() == nullptr )
		return FALSE;//No need to call up inheritance, this is a module based question, and I say no.

	return getRedirectedContain()->isDisplayedOnControlBar();
}

//-------------------------------------------------------------------------------------------------
const Object *OverlordContain::friend_getRider() const
{
// The draw order dependency bug for riders means that our draw module needs to cheat to get
	//around it.	So this is another function that knows it is getting around redirection to ask
	// an Overlord specific function.

 	if( m_containListSize > 0 )
 		return m_containList.front();


	return nullptr;
}

//-------------------------------------------------------------------------------------------------
void OverlordContain::activateRedirectedContain()
{
	m_redirectionActivated = TRUE;
}

//-------------------------------------------------------------------------------------------------
void OverlordContain::deactivateRedirectedContain()
{
	m_redirectionActivated = FALSE;
}


//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
// if my object gets selected, then my visible passengers should, too
// this gets called from
void OverlordContain::clientVisibleContainedFlashAsSelected()
{
	// THIS OVERRIDES GRAHAMS NASTY OVERRIDE THING
	// SO WE CAN FLASH THE PORTABLE BUNKER INSTEAD OF ITS OCCUPANTS
	const ContainedItemsList* items = TransportContain::getContainedItemsList();

	if( items )
	{
		ContainedItemsList::const_iterator it;
		it = items->begin();

		while( it != items->end() )
		{
			Object *object = *it;
			if ( object && object->isKindOf( KINDOF_PORTABLE_STRUCTURE ) )
			{
				Drawable *draw = object->getDrawable();
				if ( draw )
				{
					draw->flashAsSelected(); //WOW!
				}
			}

			++it;
		}
	}

}


Bool OverlordContain::isPassengerAllowedToFire( ObjectID id ) const
{
	Object *passenger = TheGameLogic->findObjectByID(id);

	if(passenger != nullptr)
	{
		//only allow infantry, and turrets and such.  no vehicles.
		if(passenger->isKindOf(KINDOF_INFANTRY) == FALSE && passenger->isKindOf(KINDOF_PORTABLE_STRUCTURE) == FALSE)
			return FALSE;
	}


  if ( getObject() && getObject()->getContainedBy() ) // nested containment voids firing, always
	{
    return FALSE;
	}

	return TransportContain::isPassengerAllowedToFire();
}



// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void OverlordContain::crc( Xfer *xfer )
{

	// extend base class
	TransportContain::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void OverlordContain::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	TransportContain::xfer( xfer );

	// redirection activated
	xfer->xferBool( &m_redirectionActivated );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void OverlordContain::loadPostProcess()
{

	// extend base class
	TransportContain::loadPostProcess();

}
```
