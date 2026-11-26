# Translation Packet `PKT-314`: `CommandButtonHuntUpdate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-314`
- **Work Package:** `WP-5.4` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/update/command_button_hunt_update.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (21):**
  - `Common/ActionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ActionManager.h`
  - `Common/KindOf.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/KindOf.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/RandomValue.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/RandomValue.h`
  - `Common/SpecialPower.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/SpecialPower.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/ControlBar.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/ControlBar.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/CollideModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CollideModule.h`
  - `GameLogic/Module/CommandButtonHuntUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CommandButtonHuntUpdate.h`
  - `GameLogic/Module/SpecialAbilityUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpecialAbilityUpdate.h`
  - `GameLogic/Module/SpecialPowerModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpecialPowerModule.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/ObjectIter.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ObjectIter.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `GameLogic/ScriptEngine.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ScriptEngine.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (2):**
  - `"ScanRate" => INI::parseDurationUnsignedInt`
  - `"ScanRange" => INI::parseReal`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (2 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferAsciiString( &m_commandButtonName )`

## 2.5 Existing Rust Symbol Index (Use These Exact Paths)
- `zh_logic::modules::update::base_renerate_update`: `BaseRegenerateUpdateModuleData`, `BaseRegenerateUpdate`
- `zh_logic::modules::update::command_button_hunt_update`: `CommandButtonHuntUpdateModuleData`, `CommandButtonHuntUpdate`
- `zh_logic::modules::update::deletion_update`: `DeletionUpdateModuleData`, `DeletionUpdate`
- `zh_logic::modules::update::dock_update`: `DockUpdateModuleData`, `FACET_MASK`, `DockUpdateState`
- `zh_logic::modules::update::dynamic_geometry_info_update`: `DIRECTION_FORWARD`, `DynamicGeometryInfoUpdateModuleData`, `DynamicGeometryInfoUpdate`
- `zh_logic::modules::update::dynamic_shroud_clearing_range_update`: `DynamicShroudClearingRangeUpdateModuleData`, `DscruState`, `DynamicShroudClearingRangeUpdate`
- `zh_logic::modules::update::fire_spread_update`: `FireSpreadUpdateModuleData`, `FireSpreadUpdate`
- `zh_logic::modules::update::fire_weapon_update`: `FireWeaponUpdateModuleData`, `FireWeaponUpdate`
- `zh_logic::modules::update::flammable_update`: `FlammableUpdateModuleData`, `FlammableUpdate`
- `zh_logic::modules::update::float_update`: `FloatUpdateModuleData`, `FloatUpdate`
- `zh_logic::modules::update::height_die_update`: `HeightDieUpdateModuleData`, `HeightDieUpdate`
- `zh_logic::modules::update::lifetime_update`: `LifetimeUpdateModuleData`, `LifetimeUpdate`
- `zh_logic::modules::update::physics_update`: `STICK_TO_GROUND`, `ALLOW_BOUNCE`, `APPLY_FRICTION2D_WHEN_AIRBORNE`, `UPDATE_EVER_RUN`, `WAS_AIRBORNE_LAST_FRAME`, `ALLOW_COLLIDE_FORCE`, `ALLOW_TO_FALL`, `HAS_PITCHROLLYAW`, `IMMUNE_TO_FALLING_DAMAGE`, `IS_IN_FREEFALL`, `IS_IN_UPDATE`, `IS_STUNNED`, `TURN_NEGATIVE`, `TURN_NONE`, ... (+4)
- `zh_logic::modules::update::power_plant_update`: `PowerPlantUpdateModuleData`, `PowerPlantUpdate`
- `zh_logic::modules::update::projectile_stream_update`: `MAX_PROJECTILE_STREAM`, `ProjectileStreamUpdateModuleData`, `ProjectileStreamUpdate`
- `zh_logic::modules::update::prone_update`: `ProneUpdateModuleData`, `ProneUpdate`
- `zh_logic::modules::update::radar_update`: `RadarUpdateModuleData`, `RadarUpdate`
- `zh_logic::modules::update::radius_decal_update`: `RadiusDecalUpdateModuleData`, `RadiusDecalUpdate`
- `zh_logic::modules::update::repair_dock_update`: `RepairDockUpdateModuleData`, `RepairDockUpdate`
- `zh_logic::modules::update::smart_bomb_target_homing_update`: `SmartBombTargetHomingUpdateModuleData`, `SmartBombTargetHomingUpdate`
- `zh_logic::modules::update::special_ability_update`: `SPECIAL_ABILITY_HUGE_DISTANCE`, `STATE_NONE`, `STATE_PACKING`, `STATE_UNPACKING`, `STATE_PACKED`, `STATE_UNPACKED`, `AbilityFeedback`, `SpecialAbilityUpdateModuleData`, `SpecialAbilityUpdate`
- `zh_logic::modules::update::spy_vision_update`: `SpyVisionUpdateModuleData`, `SpyVisionUpdate`, `fn`
- `zh_logic::modules::update::stealth_detector_update`: `StealthDetectorUpdateModuleData`, `StealthDetectorUpdate`
- `zh_logic::modules::update::stealth_update`: `STEALTH_NOT_WHILE_ATTACKING`, `STEALTH_NOT_WHILE_MOVING`, `STEALTH_NOT_WHILE_USING_ABILITY`, `STEALTH_NOT_WHILE_FIRING_PRIMARY`, `STEALTH_NOT_WHILE_FIRING_SECONDARY`, `STEALTH_NOT_WHILE_FIRING_TERTIARY`, `STEALTH_ONLY_WITH_BLACK_MARKET`, `STEALTH_NOT_WHILE_TAKING_DAMAGE`, `STEALTH_NOT_WHILE_RIDERS_ATTACKING`, `STEALTH_NOT_WHILE_FIRING_WEAPON`, `STEALTH_LEVEL_NAMES`, `StealthUpdateModuleData`, `StealthUpdate`
- `zh_logic::modules::update::supply_center_dock_update`: `SupplyCenterDockUpdateModuleData`, `SupplyCenterDockUpdate`
- `zh_logic::modules::update::supply_warehouse_dock_update`: `SupplyWarehouseDockUpdateModuleData`, `SupplyWarehouseDockUpdate`
- `zh_logic::modules::update::weapon_bonus_update`: `WeaponBonusUpdateModuleData`, `WeaponBonusUpdate`
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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CommandButtonHuntUpdate.h`
```cpp
// FILE: CommandButtonHuntUpdate.cpp //////////////////////////////////////////////////////////////////////////
// Author: John Ahlquist, Sept. 2002
// Desc:   Update module to handle wounded idle infantry finding a heal unit or structure.
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/KindOf.h"
#include "GameLogic/Module/UpdateModule.h"

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
class ThingTemplate;
class WeaponTemplate;


//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class CommandButtonHuntUpdateModuleData : public ModuleData
{
public:
	UnsignedInt			m_scanFrames;
	Real						m_scanRange;

	CommandButtonHuntUpdateModuleData();
	static void buildFieldParse(MultiIniFieldParse& p);

private:

};

//-------------------------------------------------------------------------------------------------
/** The default	update module */
//-------------------------------------------------------------------------------------------------
class CommandButtonHuntUpdate : public UpdateModule
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( CommandButtonHuntUpdate, "CommandButtonHuntUpdate" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( CommandButtonHuntUpdate, CommandButtonHuntUpdateModuleData );

public:

	CommandButtonHuntUpdate( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	virtual void onObjectCreated() override;
	virtual UpdateSleepTime update() override;

	void setCommandButton(const AsciiString& buttonName);

protected:
	Object* scanClosestTarget();
	UpdateSleepTime huntSpecialPower(AIUpdateInterface *ai);
	UpdateSleepTime huntWeapon(AIUpdateInterface *ai);
	UpdateSleepTime huntEnter( AIUpdateInterface *ai );


protected:
	AsciiString		m_commandButtonName;
	const CommandButton *m_commandButton;
};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/CommandButtonHuntUpdate.cpp`
```cpp
// FILE: CommandButtonHuntUpdate.cpp //////////////////////////////////////////////////////////////////////////
// Author: John Ahlquist, Sept. 2002
// Desc:   Update module to handle "Hunting" using a special power.  If the unit is idle, and the
// power is not active, it targets a new unit with the power.	 Note that this is an update rather than
// an AI state because many of the special abilities use the ai to perform portions of the special
// ability.
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/ActionManager.h"
#include "Common/Player.h"
#include "Common/RandomValue.h"
#include "Common/SpecialPower.h"
#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"
#include "GameClient/ControlBar.h"
#include "GameClient/Drawable.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectIter.h"
#include "GameLogic/Module/CommandButtonHuntUpdate.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/CollideModule.h"
#include "GameLogic/Module/SpecialAbilityUpdate.h"
#include "GameLogic/Module/SpecialPowerModule.h"
#include "GameLogic/ScriptEngine.h"


//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
CommandButtonHuntUpdateModuleData::CommandButtonHuntUpdateModuleData()
{
	m_scanFrames				= LOGICFRAMES_PER_SECOND;
	m_scanRange					= 9999.0f;
}

//-------------------------------------------------------------------------------------------------
/*static*/ void CommandButtonHuntUpdateModuleData::buildFieldParse(MultiIniFieldParse& p)
{
	ModuleData::buildFieldParse(p);

	static const FieldParse dataFieldParse[] =
	{
		{ "ScanRate",							INI::parseDurationUnsignedInt,	nullptr, offsetof( CommandButtonHuntUpdateModuleData, m_scanFrames ) },
		{ "ScanRange",						INI::parseReal,									nullptr, offsetof( CommandButtonHuntUpdateModuleData, m_scanRange ) },
		{ nullptr, nullptr, nullptr, 0 }
	};
	p.add(dataFieldParse);
}

//-------------------------------------------------------------------------------------------------
CommandButtonHuntUpdate::CommandButtonHuntUpdate( Thing *thing, const ModuleData* moduleData ) :
UpdateModule( thing, moduleData ),
m_commandButton(nullptr)
{
	setWakeFrame(getObject(), UPDATE_SLEEP_FOREVER);
	m_commandButtonName = AsciiString::TheEmptyString;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
CommandButtonHuntUpdate::~CommandButtonHuntUpdate()
{

}


//-------------------------------------------------------------------------------------------------
void CommandButtonHuntUpdate::onObjectCreated()
{

}

//-------------------------------------------------------------------------------------------------
void CommandButtonHuntUpdate::setCommandButton(const AsciiString& buttonName)
{
	Object *obj = getObject();
	m_commandButtonName = buttonName;
	m_commandButton = nullptr;
	const CommandSet *commandSet = TheControlBar->findCommandSet( obj->getCommandSetString() );
	if( commandSet )
	{
		for( Int i = 0; i < MAX_COMMANDS_PER_SET; i++ )
		{
			//Get the command button.
			m_commandButton = commandSet->getCommandButton(i);

			if( m_commandButton )
			{
				if( !m_commandButton->getName().isEmpty() )
				{
					if( m_commandButton->getName() == m_commandButtonName )
					{
						break;
					}
				}
			}
			m_commandButton = nullptr;
		}
	}
	if (m_commandButton==nullptr) {
		return;
	}
	AIUpdateInterface *ai = obj->getAI();
	if (ai==nullptr ) return;

	// Stop whatever we're doing.
	ai->aiIdle(CMD_FROM_AI);
	update();
	setWakeFrame(obj, UPDATE_SLEEP_NONE); // wake up.
}

//-------------------------------------------------------------------------------------------------
/** The update callback. */
//-------------------------------------------------------------------------------------------------
UpdateSleepTime CommandButtonHuntUpdate::update()
{

	Object *obj = getObject();

	AIUpdateInterface *ai = obj->getAI();
	if (ai==nullptr || m_commandButton==nullptr) return UPDATE_SLEEP_FOREVER;

	if (ai->getLastCommandSource() != CMD_FROM_AI) {
		// If a script or the player (in this case should only be script, but either way)
		// we quit hunting.
		m_commandButton = nullptr;
		m_commandButtonName.clear();
		return UPDATE_SLEEP_FOREVER;
	}

	switch( m_commandButton->getCommandType() )
	{
		case GUI_COMMAND_SPECIAL_POWER:
			return huntSpecialPower(ai);
		case GUI_COMMAND_SWITCH_WEAPON:
		case GUI_COMMAND_FIRE_WEAPON:
			return huntWeapon(ai);
		case GUICOMMANDMODE_HIJACK_VEHICLE:
		case GUICOMMANDMODE_CONVERT_TO_CARBOMB:
		case GUICOMMANDMODE_SABOTAGE_BUILDING:
			return huntEnter( ai );
		default:
			return UPDATE_SLEEP_FOREVER;
	}
	return UPDATE_SLEEP_FOREVER;
}

//-------------------------------------------------------------------------------------------------
/** Do a special power weapon. */
//-------------------------------------------------------------------------------------------------
UpdateSleepTime CommandButtonHuntUpdate::huntWeapon(AIUpdateInterface *ai)
{
	Object *obj = getObject();

	if(  ai->isIdle() )
	{
		ai->aiHunt(CMD_FROM_AI);
	}
	WeaponSlotType weaponSlot = m_commandButton->getWeaponSlot();
	// lock it just till the weapon is empty or the attack is "done"
	obj->setWeaponLock( weaponSlot, LOCKED_TEMPORARILY );
	return UPDATE_SLEEP_NONE;
}

//-------------------------------------------------------------------------------------------------
/** Do a special power hunt. */
//-------------------------------------------------------------------------------------------------
UpdateSleepTime CommandButtonHuntUpdate::huntSpecialPower(AIUpdateInterface *ai)
{
	Object *obj = getObject();
	const CommandButtonHuntUpdateModuleData *data = getCommandButtonHuntUpdateModuleData();
	if(  !ai->isIdle() )
	{
		return UPDATE_SLEEP(data->m_scanFrames);
	}
	const SpecialPowerTemplate *spTemplate = m_commandButton->getSpecialPowerTemplate();
	if( spTemplate )
	{
		SpecialAbilityUpdate* spUpdate = obj->findSpecialAbilityUpdate( spTemplate->getSpecialPowerType() );
		if (spUpdate == nullptr) return UPDATE_SLEEP_FOREVER;
		if (spUpdate->isActive()) {
			return UPDATE_SLEEP(data->m_scanFrames);
		}
	}
	//Periodic scanning (expensive)
	Object *victim = scanClosestTarget();
	if(victim)
	{
		obj->doCommandButtonAtObject( m_commandButton, victim, CMD_FROM_AI );
	}
	return UPDATE_SLEEP(data->m_scanFrames);
}

//-------------------------------------------------------------------------------------------------
/** Do an enter hunt (sabotage, convert to carbomb, hijack). */
//-------------------------------------------------------------------------------------------------
UpdateSleepTime CommandButtonHuntUpdate::huntEnter( AIUpdateInterface *ai )
{
	Object *obj = getObject();
	const CommandButtonHuntUpdateModuleData *data = getCommandButtonHuntUpdateModuleData();
	if( !ai->isIdle() )
	{
		return UPDATE_SLEEP( data->m_scanFrames );
	}

	//Periodic scanning (expensive)
	Object *victim = scanClosestTarget();
	if( victim )
	{
		obj->doCommandButtonAtObject( m_commandButton, victim, CMD_FROM_AI );
	}
	return UPDATE_SLEEP( data->m_scanFrames );
}

//-------------------------------------------------------------------------------------------------
Object* CommandButtonHuntUpdate::scanClosestTarget()
{
	const CommandButtonHuntUpdateModuleData *data = getCommandButtonHuntUpdateModuleData();
	Object *me = getObject();

	PartitionFilterAlive aliveFilter;

	// only consider enemies (unless it's convert to carbomb).
	PartitionFilterRelationship::RelationshipAllowTypes relationship = PartitionFilterRelationship::ALLOW_ENEMIES;

	Bool isEnter = FALSE;
	switch( m_commandButton->getCommandType() )
	{
		case GUICOMMANDMODE_CONVERT_TO_CARBOMB:
			relationship = PartitionFilterRelationship::ALLOW_NEUTRAL; //Only neutrals.
			isEnter = TRUE;
			break;
		case GUICOMMANDMODE_HIJACK_VEHICLE:
		case GUICOMMANDMODE_SABOTAGE_BUILDING:
			isEnter = TRUE;
			break;
	}

	PartitionFilterSameMapStatus filterMapStatus(getObject());
	PartitionFilterRelationship	filterTeam(me, relationship );
	PartitionFilterStealthedAndUndetected filterStealthed( me, FALSE );

	PartitionFilter *filters[5];
	filters[0] = &aliveFilter;
	filters[1] = &filterMapStatus;
	filters[2] = &filterStealthed;
	filters[3] = &filterTeam;
	filters[4] = nullptr;

	Bool isBlackLotusVehicleHack = FALSE;
	Bool isCaptureBuilding = FALSE;
	Bool isPlaceExplosive = FALSE;
 	const SpecialPowerTemplate *spTemplate = m_commandButton->getSpecialPowerTemplate();
	if( !isEnter )
	{
		if( !spTemplate )
			return nullptr;  // isn't going to happen.
		isBlackLotusVehicleHack = 	(spTemplate->getSpecialPowerType() == SPECIAL_BLACKLOTUS_DISABLE_VEHICLE_HACK);
		isCaptureBuilding = 	(spTemplate->getSpecialPowerType() == SPECIAL_INFANTRY_CAPTURE_BUILDING);
		if (isCaptureBuilding)
		{
			filters[3] = nullptr;  // It's ok (in fact necessary for oil derricks) to capture special buildings.
			if (spTemplate->getSpecialPowerType() == SPECIAL_TIMED_CHARGES)
				isPlaceExplosive = true;
			if (spTemplate->getSpecialPowerType() == SPECIAL_TANKHUNTER_TNT_ATTACK)
				isPlaceExplosive = true;
		}
	}

	ObjectIterator *iter = ThePartitionManager->iterateObjectsInRange( me->getPosition(), data->m_scanRange,
		FROM_CENTER_2D, filters, ITER_SORTED_NEAR_TO_FAR );
	MemoryPoolObjectHolder hold(iter);

	Object *bestTarget = nullptr;
	Int			effectivePriority=0;
	Int			actualPriority=0;
	const AttackPriorityInfo *info=nullptr;
	if (me->getAI()) {
		info = me->getAI()->getAttackInfo();
	}

	SpecialPowerModuleInterface *mod = me->getSpecialPowerModule( spTemplate );
	if( mod )
	{
		for( Object *other = iter->first(); other; other = iter->next() )
		{
			if (other->isDisabled() && isBlackLotusVehicleHack) {
				// The hack disables the vehicle, so we don't want to do it again.
				continue;
			}
			if (isCaptureBuilding) {
				if (me->getControllingPlayer() == other->getControllingPlayer()) {
					continue; // no need to capture our own buildings.
				}
				if (me->getRelationship(other) == ALLIES) {
					continue; // It's not polite to capture allies buildings.
				}
			}
			if( TheActionManager->canDoSpecialPowerAtObject( me, other, CMD_FROM_AI, spTemplate, 0 ) )
			{
				if (isPlaceExplosive) {
					Real range = spTemplate->getViewObjectRange();
					// Don't target things near explosives... It's just not a good idea.
					PartitionFilterSamePlayer filterPlayer( me->getControllingPlayer() );	// Look for our own mines.
					PartitionFilterAcceptByKindOf filterKind(MAKE_KINDOF_MASK(KINDOF_MINE), KINDOFMASK_NONE);
					PartitionFilter *filters[] = { &filterKind, &filterPlayer, nullptr };
					Object *mine = ThePartitionManager->getClosestObject( other, range, FROM_BOUNDINGSPHERE_2D, filters );// could be null. this is ok.
					if (mine) {
						continue;
					}
				}
				Real distSqr = ThePartitionManager->getDistanceSquared(me, other, FROM_BOUNDINGSPHERE_2D);
				Real dist = WWMath::Sqrt(distSqr);
				Int curPriority = data->m_scanRange - dist;
				if (info) curPriority = info->getPriority(other->getTemplate());
				if (curPriority == 0)
					continue; // don't attack 0 priority targets.
				Int modifier = (Int)WWMath::Div_Safe(dist, TheAI->getAiData()->m_attackPriorityDistanceModifier);
				Int modPriority = curPriority-modifier;
				if (modPriority < 1)
					modPriority = 1;
				if (modPriority > effectivePriority)
				{
					effectivePriority = modPriority;
					actualPriority = curPriority;
					bestTarget = other;
				}
				if (modPriority == effectivePriority && curPriority > actualPriority)
				{
					effectivePriority = modPriority;
					actualPriority = curPriority;
					bestTarget = other;
				}

			}
		}
	}
	else if( isEnter )
	{
		Bool valid = FALSE;
		for( Object *other = iter->first(); other; other = iter->next() )
		{
			switch( m_commandButton->getCommandType() )
			{
				case GUICOMMANDMODE_HIJACK_VEHICLE:
					valid = TheActionManager->canHijackVehicle( me, other, CMD_FROM_AI );
					break;
				case GUICOMMANDMODE_SABOTAGE_BUILDING:
					valid = TheActionManager->canSabotageBuilding( me, other, CMD_FROM_AI );
					break;
				case GUICOMMANDMODE_CONVERT_TO_CARBOMB:
					valid = TheActionManager->canConvertObjectToCarBomb( me, other, CMD_FROM_AI );
					break;
			}
			if( valid )
			{
				return other;
			}
		}
	}


	return bestTarget;
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void CommandButtonHuntUpdate::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void CommandButtonHuntUpdate::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// command button name
	xfer->xferAsciiString( &m_commandButtonName );

	// command button pointer (on loading only)
	if( xfer->getXferMode() == XFER_LOAD )
	{

		// initialize to no command button
		m_commandButton = nullptr;

		// find command button pointer if name is present
		if( m_commandButtonName.isEmpty() == FALSE )
		{
			Object *us = getObject();
			const CommandSet *commandSet = TheControlBar->findCommandSet( us->getCommandSetString() );

			if( commandSet )
			{
				const CommandButton *button;

				for( Int i = 0; i < MAX_COMMANDS_PER_SET; i++ )
				{

					button = commandSet->getCommandButton(i);
					if( button && button->getName() == m_commandButtonName )
					{

						m_commandButton = button;
						break;  // exit for, i

					}

				}

			}

		}

	}

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void CommandButtonHuntUpdate::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
