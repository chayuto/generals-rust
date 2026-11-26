# Translation Packet `PKT-305`: `AutoDepositUpdate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-305`
- **Work Package:** `WP-5.4` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/update/auto_deposit_update.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (17):**
  - `Common/BuildAssistant.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/BuildAssistant.h`
  - `Common/INI.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/INI.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/RandomValue.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/RandomValue.h`
  - `Common/Thing.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Thing.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Color.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/Color.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/GameText.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/GameText.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/AutoDepositUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AutoDepositUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (5):**
  - `"DepositTiming" => INI::parseDurationUnsignedInt`
  - `"DepositAmount" => INI::parseInt`
  - `"InitialCaptureBonus" => INI::parseInt`
  - `"ActualMoney" => INI::parseBool`
  - `"UpgradedBoost" => parseUpgradePair`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (4 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUnsignedInt( &m_depositOnFrame )`
  3. `xferBool( &m_awardInitialCaptureBonus )`
  4. `xferBool(&m_initialized)`
- **PRNG Calls (2 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameClientRandomValue(-width,width)`
  - `GameClientRandomValue(-depth,depth)`

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
- `zh_logic::modules::behavior::auto_heal_behavior`: `AutoHealBehaviorModuleData`, `AutoHealBehavior`
- `zh_logic::modules::behavior::battle_bus_slow_death_behavior`: `BattleBusSlowDeathBehaviorModuleData`, `BattleBusSlowDeathBehavior`
- `zh_logic::modules::behavior::behavior_module`: `BehaviorModuleData`, `fn`, `UpdateScheduleTracker`
- `zh_logic::modules::behavior::bridge_scaffold_behavior`: `BridgeScaffoldBehaviorModuleData`, `BridgeScaffoldBehavior`
- `zh_logic::modules::behavior::bridge_tower_behavior`: `BridgeTowerBehaviorModuleData`, `BridgeTowerBehavior`
- `zh_logic::modules::behavior::bunker_buster_behavior`: `BunkerBusterBehaviorModuleData`, `BunkerBusterBehavior`
- `zh_logic::modules::behavior::fire_weapon_when_damaged_behavior`: `FireWeaponWhenDamagedBehaviorModuleData`, `FireWeaponWhenDamagedBehavior`
- `zh_logic::modules::behavior::fire_weapon_when_dead_behavior`: `FireWeaponWhenDeadBehaviorModuleData`, `FireWeaponWhenDeadBehavior`
- `zh_logic::modules::behavior::grant_stealth_behavior`: `GrantStealthBehaviorModuleData`, `GrantStealthBehavior`
- `zh_logic::modules::behavior::instant_death_behavior`: `InstantDeathBehaviorModuleData`, `InstantDeathBehavior`
- `zh_logic::modules::behavior::jet_slow_death_behavior`: `JetSlowDeathBehaviorModuleData`, `JetSlowDeathBehavior`
- `zh_logic::modules::behavior::neutron_blast_behavior`: `NeutronBlastBehaviorModuleData`, `NeutronBlastBehavior`
- `zh_logic::modules::behavior::overcharge_behavior`: `OverchargeBehaviorModuleData`, `OverchargeBehavior`
- `zh_logic::modules::behavior::poisoned_behavior`: `PoisonedBehaviorModuleData`, `PoisonedBehavior`
- `zh_logic::modules::behavior::prison_behavior`: `PrisonBehaviorModuleData`, `PrisonVisual`, `PrisonBehavior`
- `zh_logic::modules::behavior::propaganda_tower_behavior`: `PropagandaTowerBehaviorModuleData`, `PropagandaTowerBehavior`
- `zh_logic::modules::behavior::slow_death_behavior`: `SLOW_DEATH_PHASE_NAMES`, `SlowDeathPhaseType`, `COUNT`, `SD_FLAG_SLOW_DEATH_ACTIVATED`, `SD_FLAG_MIDPOINT_EXECUTED`, `SD_FLAG_FLUNG_INTO_AIR`, `SD_FLAG_BOUNCED`, `SD_EFFECT_HAS_FX`, `SD_EFFECT_HAS_OCL`, `SD_EFFECT_HAS_WEAPON`, `SlowDeathBehaviorModuleData`, `fn`, `SlowDeathCore`, `SlowDeathBehavior`
- `zh_logic::modules::behavior::supply_warehouse_crippling_behavior`: `SupplyWarehouseCripplingBehaviorModuleData`, `SupplyWarehouseCripplingBehavior`
- `zh_logic::modules::behavior::tech_building_behavior`: `TechBuildingBehaviorModuleData`, `TechBuildingBehavior`
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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AutoDepositUpdate.h`
```cpp
// FILE: AutoDepositUpdate.h /////////////////////////////////////////////////
//-----------------------------------------------------------------------------
//
//                       Electronic Arts Pacific.
//
//                       Confidential Information
//                Copyright (C) 2002 - All Rights Reserved
//
//-----------------------------------------------------------------------------
//
//	created:	Aug 2002
//
//	Filename: 	AutoDepositUpdate.h
//
//	author:		Chris Huybregts
//
//	purpose:	Auto Deposit Update Module
//
//-----------------------------------------------------------------------------
///////////////////////////////////////////////////////////////////////////////

#pragma once

//-----------------------------------------------------------------------------
// SYSTEM INCLUDES ////////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------

//-----------------------------------------------------------------------------
// USER INCLUDES //////////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------
#include "GameLogic/Module/UpdateModule.h"
//-----------------------------------------------------------------------------
// FORWARD REFERENCES /////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------
class Player;
class Thing;
void parseUpgradePair( INI *ini, void *instance, void *store, const void *userData );
struct upgradePair
{
	std::string type;
	Int         amount;
};

//-----------------------------------------------------------------------------
// TYPE DEFINES ///////////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------
class AutoDepositUpdateModuleData : public UpdateModuleData
{
public:

	UnsignedInt m_depositFrame;
	Int m_depositAmount;
	Int m_initialCaptureBonus;
	Bool m_isActualMoney;
	std::list<upgradePair> m_upgradeBoost;

	AutoDepositUpdateModuleData()
	{
		m_depositFrame = 0;
		m_depositAmount = 0;
		m_initialCaptureBonus = 0;
		m_isActualMoney = TRUE;
		m_upgradeBoost.clear();
	}

	static void buildFieldParse(MultiIniFieldParse& p)
	{
    UpdateModuleData::buildFieldParse(p);
		static const FieldParse dataFieldParse[] =
		{
			{ "DepositTiming",					INI::parseDurationUnsignedInt,		nullptr, offsetof( AutoDepositUpdateModuleData, m_depositFrame ) },
			{ "DepositAmount",					INI::parseInt,		nullptr, offsetof( AutoDepositUpdateModuleData, m_depositAmount ) },
			{ "InitialCaptureBonus",		INI::parseInt,		nullptr, offsetof( AutoDepositUpdateModuleData, m_initialCaptureBonus ) },
			{ "ActualMoney",						INI::parseBool,		nullptr, offsetof( AutoDepositUpdateModuleData, m_isActualMoney ) },
			{ "UpgradedBoost",					parseUpgradePair,		nullptr, offsetof( AutoDepositUpdateModuleData, m_upgradeBoost ) },
			{ nullptr, nullptr, nullptr, 0 }
		};
    p.add(dataFieldParse);
	}
};


//-------------------------------------------------------------------------------------------------
class AutoDepositUpdate : public UpdateModule
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( AutoDepositUpdate, "AutoDepositUpdate" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( AutoDepositUpdate, AutoDepositUpdateModuleData )

public:

	AutoDepositUpdate( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	void awardInitialCaptureBonus( Player *player );	// Test and award the initial capture bonus
	virtual UpdateSleepTime update() override;

protected:

	Int getUpgradedSupplyBoost() const;

	UnsignedInt m_depositOnFrame;
	Bool m_awardInitialCaptureBonus;
	Bool m_initialized;

};


//-----------------------------------------------------------------------------
// INLINING ///////////////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------

//-----------------------------------------------------------------------------
// EXTERNALS //////////////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/AutoDepositUpdate.cpp`
```cpp
// FILE: AutoDepositUpdate.cpp /////////////////////////////////////////////////
//-----------------------------------------------------------------------------
//
//                       Electronic Arts Pacific.
//
//                       Confidential Information
//                Copyright (C) 2002 - All Rights Reserved
//
//-----------------------------------------------------------------------------
//
//	created:	Aug 2002
//
//	Filename: 	AutoDepositUpdate.cpp
//
//	author:		Chris Huybregts
//
//	purpose:	The meat of the auto deposit update module
//
//-----------------------------------------------------------------------------
///////////////////////////////////////////////////////////////////////////////

//-----------------------------------------------------------------------------
// SYSTEM INCLUDES ////////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------

//-----------------------------------------------------------------------------
// USER INCLUDES //////////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/BuildAssistant.h"
#include "Common/Thing.h"
#include "Common/ThingTemplate.h"
#include "Common/INI.h"
#include "Common/RandomValue.h"
#include "Common/Player.h"
#include "Common/Xfer.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/AutoDepositUpdate.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Object.h"
#include "GameClient/Drawable.h"
#include "GameClient/InGameUI.h"
#include "GameClient/Color.h"
#include "GameClient/GameText.h"

//-------------------------------------------------------------------------------------------------
void parseUpgradePair( INI *ini, void *instance, void *store, const void *userData )
{
	upgradePair info;
	info.amount = 0;

	const char *token = ini->getNextToken( ini->getSepsColon() );
	if ( stricmp(token, "UpgradeType") == 0 )
	{
		token = ini->getNextToken( ini->getSepsColon() );
		info.type = token;
	}
	else
		throw INI_INVALID_DATA;


	token = ini->getNextToken( ini->getSepsColon() );
	if ( stricmp(token, "Boost") == 0 )
		info.amount = INI::scanInt(ini->getNextToken( ini->getSepsColon() ));
	else
		throw INI_INVALID_DATA;

	// Insert the info into the upgrade list
	std::list<upgradePair> * theList = (std::list<upgradePair>*)store;
	theList->push_back(info);

}

//-----------------------------------------------------------------------------
// PUBLIC FUNCTIONS ///////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------
AutoDepositUpdate::AutoDepositUpdate( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{
	m_depositOnFrame = TheGameLogic->getFrame() + getAutoDepositUpdateModuleData()->m_depositFrame;
	m_awardInitialCaptureBonus = FALSE;
	m_initialized = FALSE;
}
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
AutoDepositUpdate::~AutoDepositUpdate()
{

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void AutoDepositUpdate::awardInitialCaptureBonus( Player *player )
{
	m_depositOnFrame = TheGameLogic->getFrame() + getAutoDepositUpdateModuleData()->m_depositFrame;
	if(!player || !m_awardInitialCaptureBonus || getAutoDepositUpdateModuleData()->m_initialCaptureBonus <= 0)
		return;

	player->getMoney()->deposit( getAutoDepositUpdateModuleData()->m_initialCaptureBonus );
	player->getScoreKeeper()->addMoneyEarned( getAutoDepositUpdateModuleData()->m_initialCaptureBonus );

	//Display cash income floating over the blacklotus
	if(getAutoDepositUpdateModuleData()->m_initialCaptureBonus > 0)
	{
		UnicodeString moneyString;
		moneyString.format( TheGameText->fetch( "GUI:AddCash" ), getAutoDepositUpdateModuleData()->m_initialCaptureBonus );
		Coord3D pos;
		pos.set( *getObject()->getPosition() );
		pos.z += 10.0f; //add a little z to make it show up above the unit.
		Color color = player->getPlayerColor() | GameMakeColor( 0, 0, 0, 230 );
		TheInGameUI->addFloatingText( moneyString, &pos, color );
	}

	m_awardInitialCaptureBonus = FALSE;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
UpdateSleepTime AutoDepositUpdate::update()
{
	const AutoDepositUpdateModuleData *modData = getAutoDepositUpdateModuleData();
/// @todo srj use SLEEPY_UPDATE here
	if( TheGameLogic->getFrame() >= m_depositOnFrame)
	{
		if (!m_initialized) {
			// Note - we have to set these in update, because during load the team is set,
			// and we don't want to award initial bonus on load.  jba :)
			m_awardInitialCaptureBonus = TRUE;
			m_initialized = TRUE;
		}
		m_depositOnFrame = TheGameLogic->getFrame() + modData->m_depositFrame;

		if(getObject()->isNeutralControlled() || modData->m_depositAmount <= 0 )
			return UPDATE_SLEEP_NONE;

		// makes sure that buildings under construction do not get a bonus CCB
		if( getObject()->getConstructionPercent() != CONSTRUCTION_COMPLETE )
			return UPDATE_SLEEP_NONE;

		int moneyAmount = modData->m_depositAmount + getUpgradedSupplyBoost();

		if( modData->m_isActualMoney )
		{
			getObject()->getControllingPlayer()->getMoney()->deposit( moneyAmount );
			getObject()->getControllingPlayer()->getScoreKeeper()->addMoneyEarned( modData->m_depositAmount);
		}

		if (moneyAmount > 0 && getObject()->isLogicallyVisible())
		{

      const Object *owner = getObject();
      if ( owner )
      {

			  // OY LOOK!  I AM USING LOCAL PLAYER.  Do not put anything other than TheInGameUI->addFloatingText in the block this controls!!!
			  UnicodeString moneyString;
			  moneyString.format( TheGameText->fetch( "GUI:AddCash" ), moneyAmount );
			  Coord3D pos;
			  pos.set( *getObject()->getPosition() );
			  pos.z += 10.0f; //add a little z to make it show up above the unit.

        if ( owner->isKindOf( KINDOF_STRUCTURE ) )
        {
          Real width = owner->getGeometryInfo().getMajorRadius() * 0.3f;
          Real depth = owner->getGeometryInfo().getMinorRadius() * 0.3f;
          pos.x += GameClientRandomValue(-width,width);
          pos.y += GameClientRandomValue(-depth,depth);
        }


        Color color = getObject()->getControllingPlayer()->getPlayerColor() | GameMakeColor( 0, 0, 0, 230 );
			  TheInGameUI->addFloatingText( moneyString, &pos, color );
      }
		}
	}

	return UPDATE_SLEEP_NONE;
}

//------------------------------------------------------------------------------------------------
Int AutoDepositUpdate::getUpgradedSupplyBoost() const
{
	Player *player = getObject()->getControllingPlayer();
	if (!player) return 0;

	// Loop through the upgrade pairs and see if an upgrade is present that adds supply boost
	std::list<upgradePair>::const_iterator it = getAutoDepositUpdateModuleData()->m_upgradeBoost.begin();
	while (it != getAutoDepositUpdateModuleData()->m_upgradeBoost.end())
	{
		upgradePair info = *it;

		// Check if the player has the desired upgrade. If so return the boost
		static const UpgradeTemplate *upgradeTemplate = TheUpgradeCenter->findUpgrade( info.type.c_str() );
		if (player && upgradeTemplate && player->hasUpgradeComplete(upgradeTemplate))
		{
			return info.amount;
		}

		// check next
		++it;
	}

	return 0;
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void AutoDepositUpdate::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void AutoDepositUpdate::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 2;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// deposit on frame
	xfer->xferUnsignedInt( &m_depositOnFrame );

	// award initial capture bonus
	xfer->xferBool( &m_awardInitialCaptureBonus );
	if (version>1) {
		xfer->xferBool(&m_initialized);
	}

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void AutoDepositUpdate::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
