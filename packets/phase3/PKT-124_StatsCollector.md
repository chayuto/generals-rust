# Translation Packet `PKT-124`: `StatsCollector`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-124`
- **Work Package:** `WP-3.1` (Phase 3)
- **Complexity & Assigned Tier:** Class `C1` -> `tier:1a-lite`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 2: Handle Component
- **Allowed Rust Output File:** `crates/zh_logic/src/economy/stats_collector.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (12):**
  - `Common/FileSystem.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/FileSystem.h`
  - `Common/GlobalData.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/GlobalData.h`
  - `Common/Money.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Money.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/PlayerList.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/PlayerList.h`
  - `Common/StatsCollector.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/StatsCollector.h`
  - `GameClient/MapUtil.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/MapUtil.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameNetwork/LANAPICallbacks.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameNetwork/LANAPICallbacks.h`
  - `GameNetwork/networkutil.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameNetwork/networkutil.h`
  - `PreRTS.h`
- **`Snapshot::crc` / `Snapshot::xfer` Calls:** None in module.

## 2.5 Existing Rust Symbol Index (Use These Exact Paths)
- `zh_logic::economy`: `MAX_PLAYER_COUNT`
- `zh_logic::economy::academy_stats`: `MAX_ADVICE_TIPS`, `FRAMES_BETWEEN_UPDATES`, `DEFAULT_SUPPLY_CENTER_COST`, `KINDOF_DOZER`, `KINDOF_FS_SUPPLY_CENTER`, `KINDOF_COMMANDCENTER`, `KINDOF_INFANTRY`, `KINDOF_VEHICLE`, `KINDOF_HARVESTER`, `KINDOF_HERO`, `KINDOF_FS_STRATEGY_CENTER`, `KINDOF_MONEY_HACKER`, `KINDOF_FS_BLACK_MARKET`, `KINDOF_FS_SUPPLY_DROPZONE`, ... (+9)
- `zh_logic::economy::energy`: `EnergyOwnerCallbacks`, `NoopEnergyCallbacks`, `EnergyObjectInfo`, `Energy`
- `zh_logic::economy::handicap`: `HANDICAP_TYPE_COUNT`, `THING_TYPE_COUNT`, `HandicapType`, `ThingType`, `Handicap`, `fn`
- `zh_logic::economy::mission_stats`: `MissionStats`
- `zh_logic::economy::money`: `INCOME_BUCKET_COUNT`, `LOGICFRAMES_PER_SECOND`, `MoneyCallbacks`, `NoopMoneyCallbacks`, `Money`
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
- `zh_logic::arena`: `ObjectSlot`, `INVALID`, `fn`, `ModuleSlot`, `ModuleDataId`, `ThingTemplateId`, `FacetMask`, `NONE`, `UPDATE`, `DIE`, `DAMAGE`, `CREATE`, `COLLIDE`, `BODY`, ... (+34)
- `zh_logic::modules::module`: `ModuleData`, `StubModuleData`, `ModuleSnapshotHeader`, `BehaviorModule`, `UpdateFacet`, `BodyFacet`, `DieFacet`, `DamageFacet`, `CreateFacet`, `CollideFacet`, `ContainFacet`, `UpgradeFacet`, `SpecialPowerFacet`, `DestroyFacet`, ... (+3)
- `zh_logic::modules::module_factory`: `ModuleDataParseFn`, `ModuleCreateFn`, `BuiltinModuleEntry`, `BUILTIN_ZH_MODULES`, `W3D_DRAW_STUB_NAMES`, `ModuleDataStore`, `ModuleDescriptor`, `ModuleRegistry`
- `zh_logic::object::experience_tracker`: `ExperienceParentAccess`, `ExperienceTrackerState`, `fn`
- `zh_logic::object::object_script_status_bits`: `ObjectScriptStatusBit`, `fn`, `OBJECT_STATUS_SCRIPT_DISABLED`, `OBJECT_STATUS_SCRIPT_UNPOWERED`, `OBJECT_STATUS_SCRIPT_UNSELLABLE`, `OBJECT_STATUS_SCRIPT_UNSTEALTHED`, `OBJECT_STATUS_SCRIPT_TARGETABLE`
- `zh_logic::object::object_types`: `ObjectTypes`
- `zh_logic::object::overridable`: `OverridableEntry`, `fn`
- `zh_logic::object::override`: `OverrideStore`, `Override`, `fn`
- `zh_logic::object::thing`: `INITIAL_THING_POS_X`, `INITIAL_THING_POS_Y`, `ThingTerrainAccess`, `ThingTransformReactor`, `NoopTransformReactor`, `Thing`, `fn`
- `zh_logic::object::thing_factory`: `ThingFactory`, `fn`
- `zh_logic::object::thing_template`: `LEVEL_COUNT`, `MAX_UPGRADE_CAMEO_UPGRADES`, `USE_EXP_VALUE_FOR_SKILL_VALUE`, `RADAR_PRIORITY_NAMES`, `MODULEINTERFACE_UPDATE`, `MODULEINTERFACE_DIE`, `MODULEINTERFACE_DAMAGE`, `MODULEINTERFACE_CREATE`, `MODULEINTERFACE_COLLIDE`, `MODULEINTERFACE_BODY`, `MODULEINTERFACE_CONTAIN`, `MODULEINTERFACE_UPGRADE`, `MODULEINTERFACE_SPECIAL_POWER`, `MODULEINTERFACE_DESTROY`, ... (+27)
- `zh_logic::object::weapon`: `NO_MAX_SHOTS_LIMIT`, `PATHFIND_CELL_SIZE_F`, `WEAPON_UNDERSIZE`, `WeaponReloadType`, `fn`, `WeaponPrefireType`, `WEAPON_RELOAD_NAMES`, `WEAPON_PREFIRE_NAMES`, `WEAPON_ANTI_AIRBORNE_VEHICLE`, `WEAPON_ANTI_GROUND`, `WEAPON_ANTI_PROJECTILE`, `WEAPON_ANTI_SMALL_MISSILE`, `WEAPON_ANTI_MINE`, `WEAPON_ANTI_AIRBORNE_INFANTRY`, ... (+22)
- `zh_logic::object::weapon_set`: `WEAPONSLOT_COUNT`, `WeaponSlotType`, `fn`, `WeaponSetConditionType`, `WeaponChoiceCriteria`, `WeaponLockType`, `CanAttackResult`, `WeaponTemplateSet`, `WEAPON_SET_TYPE_TO_MODEL_CONDITION_TYPE_MAP`, `WeaponSet`
- `zh_logic::rules`: `AcademyClassificationType`, `fn`, `ACADEMY_CLASSIFICATION_TYPE_NAMES`
- `zh_logic::rules::combat_types`: `ARMORSET_COUNT`, `ArmorSetFlags`, `ARMOR_SET_NAMES`, `ArmorSetType`, `fn`, `DAMAGE_NUM_TYPES`, `DamageTypeFlags`, `DAMAGE_TYPE_NAMES`, `DamageType`, `DEATH_NUM_TYPES`, `DeathTypeFlags`, `DEATH_TYPE_FLAGS_ALL`, `DEATH_TYPE_FLAGS_NONE`, `DEATH_TYPE_NAMES`, ... (+14)
- `zh_logic::rules::disabled_types`: `DISABLED_COUNT`, `DISABLED_ANY_SENTINEL`, `DisabledMaskType`, `DisabledType`, `fn`, `DISABLED_TYPE_NAMES`
- `zh_logic::rules::kind_of`: `KINDOF_COUNT`, `KindOfMaskType`, `KindOfType`, `fn`, `KINDOF_NAMES`
- `zh_logic::rules::model_state`: `MODELCONDITION_COUNT`, `ModelConditionFlags`, `MODEL_CONDITION_NAMES`, `ModelConditionFlagType`, `fn`
- `zh_logic::rules::multiplayer_settings`: `Color`, `PLAYERTEMPLATE_RANDOM`, `PLAYERTEMPLATE_OBSERVER`, `PLAYERTEMPLATE_MIN`, `GAME_COLOR_UNDEFINED`, `MultiplayerColorDefinition`, `MultiplayerColorList`, `MultiplayerStartingMoneyList`, `MultiplayerSettings`
- `zh_logic::rules::object_status_types`: `OBJECT_STATUS_COUNT`, `ObjectStatusMaskType`, `ObjectStatusType`, `fn`, `OBJECT_STATUS_NAMES`
- `zh_logic::rules::player_template`: `MAX_MP_STARTING_UNITS`, `VETERANCY_NAMES`, `ProductionChangeMap`, `ProductionVeterancyMap`, `PlayerTemplateImageLookup`, `PlayerTemplate`, `PlayerTemplateStore`
- `zh_logic::rules::production_prerequisite`: `UNIT_OR_WITH_PREV`, `MAX_PREREQ`, `ThingTemplateRef`, `ThingTemplateRegistry`, `PrereqPlayerView`, `PrereqTextLookup`, `ProductionPrerequisite`
- `zh_logic::rules::rank_info`: `RankInfoData`, `RankInfoStore`
- `zh_logic::rules::science`: `ScienceType`, `SCIENCE_INVALID`, `ScienceVec`, `SciencePlayerAccess`, `ScienceTextLookup`, `ScienceInfoData`, `ScienceStore`
- `zh_logic::rules::special_power`: `DEFAULT_DEFECTION_DETECTION_PROTECTION_TIME_LIMIT`, `SpecialPowerObjectAccess`, `SpecialPowerTemplateData`, `SpecialPowerTemplate`, `fn`, `SpecialPowerStore`
- `zh_logic::rules::special_power_types`: `SPECIALPOWER_COUNT`, `SpecialPowerMaskType`, `SpecialPowerType`, `fn`, `SPECIAL_POWER_NAMES`
- `zh_logic::rules::upgrade`: `UPGRADE_MAX_COUNT`, `UpgradeMaskType`, `UpgradeStatusType`, `fn`, `UpgradeType`, `UPGRADE_TYPE_NAMES`, `UpgradePlayerAccess`, `Upgrade`, `UpgradeTemplate`, `UpgradeCenter`
- `zh_logic::scheduler::sleepy_heap`: `UPDATE_SLEEP_INVALID`, `UPDATE_SLEEP_NONE`, `UPDATE_SLEEP_FOREVER`, `INVALID_LOGIC_INDEX`, `MAX_SLEEPY_UPDATES_PER_OBJECT`, `SleepyUpdatePhase`, `fn`, `UpdateModuleScheduleState`, `ScheduleStore`, `SleepyUpdateHeap`
- `zh_logic::sim`: `ClientSink`, `NoopClientSink`, `FlatTerrain`, `RulesRegistry`, `ObjectRecord`, `SimView`, `SimApi`, `SimWorld`, `SimContext`, `MockSimApi`
- `zh_logic::combat::armor`: `ArmorTemplate`, `Armor`, `fn`, `ArmorStore`
- `zh_logic::combat::armor_set`: `ArmorTemplateSet`
- `zh_logic::combat::damage`: `PlayerMaskType`, `HUGE_DAMAGE_AMOUNT`, `DamageInfoInput`, `DamageInfoOutput`, `DamageInfo`
- `zh_logic::combat::locomotor`: `BIGNUM`, `LocomotorAppearance`, `fn`, `LOCOMOTOR_APPEARANCE_NAMES`, `LocomotorPriority`, `LOCOMOTOR_PRIORITY_NAMES`, `LocomotorBehaviorZ`, `LOCOMOTOR_BEHAVIOR_Z_NAMES`, `LocomotorTemplate`, `LocomotorStore`, `Locomotor`

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

### `GeneralsMD/Code/GameEngine/Include/Common/StatsCollector.h`
```cpp
// FILE: StatsCollector.h /////////////////////////////////////////////////
//-----------------------------------------------------------------------------
//
//                       Electronic Arts Pacific.
//
//                       Confidential Information
//                Copyright (C) 2002 - All Rights Reserved
//
//-----------------------------------------------------------------------------
//
//	created:	Jul 2002
//
//	Filename: 	StatsCollector.h
//
//	author:		Chris Huybregts
//
//	purpose:	Convenience class to help with collecting stats.
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

//-----------------------------------------------------------------------------
// FORWARD REFERENCES /////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------
class GameMessage;


//-----------------------------------------------------------------------------
// TYPE DEFINES ///////////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------
class StatsCollector
{
public:
	StatsCollector();
	~StatsCollector();

	void reset();							///< Resets all values and writes the file header

	void collectMsgStats( const GameMessage *msg );			///< collects Msg Stats if
	void collectUnitCountStats();									///< cycle through all units and takes count
	void incrementScrollMoveCount();
	void incrementBuildCount();
	void incrementAttackCount();
	void incrementMoveCount();
	void startScrollTime();		///< Start our logging on the amount of time we're scrolling
	void endScrollTime();			///< end our logging on the amount of time we're scrolling

	void update();						///< called once a frame to see if we should poll this frame

	void writeFileEnd();				///< Write the end of the file
private:

	void createFileName();		///< Create a snazzy filename
	AsciiString m_statsFileName;		///< store the snazzy filename

	void writeInitialFileInfo();		///< write the header file info
	void writeStatInfo();					///< write the stats we're keeping track of

	void zeroOutStats();			///< zero out the stats
	UnsignedInt m_buildCommands;		///< count of the build commands the local player issued
	UnsignedInt m_moveCommands;			///< count of the move commands
	UnsignedInt m_attackCommands;		///< attack commands
	UnsignedInt m_scrollMapCommands;///< scroll map commands
	UnsignedInt m_AIUnits;					///< tally of all the AI Units
	UnsignedInt m_playerUnits;			///< tally of all the player Units

	UnsignedInt m_scrollBeginTime;	///< Begin time in frames
	UnsignedInt m_scrollTime;				///< our totals for the scrolltime
	Bool m_isScrolling;							///< flag to make sure we are scrolling

	Int m_timeCount;								///< the current timeframe we're on
	Int m_lastUpdate;								///< last time we updated
	Int m_startFrame;								///< frame we started on
};


//-----------------------------------------------------------------------------
// INLINING ///////////////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------

//-----------------------------------------------------------------------------
// EXTERNALS //////////////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------
extern StatsCollector* TheStatsCollector;			///< we need a singleton
```

### `GeneralsMD/Code/GameEngine/Source/Common/StatsCollector.cpp`
```cpp
// FILE: StatsCollector.cpp /////////////////////////////////////////////////
//-----------------------------------------------------------------------------
//
//                       Electronic Arts Pacific.
//
//                       Confidential Information
//                Copyright (C) 2002 - All Rights Reserved
//
//-----------------------------------------------------------------------------
//
//	created:	Jul 2002
//
//	Filename: 	StatsCollector.cpp
//
//	author:		Chris Huybregts
//
//	purpose:	Convenience class to gather player stats
//
//-----------------------------------------------------------------------------
///////////////////////////////////////////////////////////////////////////////

//-----------------------------------------------------------------------------
// SYSTEM INCLUDES ////////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

//-----------------------------------------------------------------------------
// USER INCLUDES //////////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------
#include "Common/StatsCollector.h"
#include "Common/FileSystem.h"
#include "Common/PlayerList.h"
#include "Common/Player.h"
#include "Common/GlobalData.h"
#include "Common/Money.h"
#include "GameLogic/Object.h"
#include "GameLogic/GameLogic.h"
#include "GameClient/MapUtil.h"
#include "GameNetwork/networkutil.h"
#include "GameNetwork/LANAPICallbacks.h"
//-----------------------------------------------------------------------------
// DEFINES ////////////////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------
StatsCollector *TheStatsCollector = nullptr;

static char statsDir[255] = "Stats\\";
//-----------------------------------------------------------------------------
// PUBLIC FUNCTIONS ///////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------

// init all
//=============================================================================
StatsCollector::StatsCollector()
{
	m_isScrolling = FALSE;
	m_scrollBeginTime = 0;
	m_scrollTime = 0;
	m_timeCount = 0;
	m_buildCommands = 0;
	m_moveCommands = 0;
	m_attackCommands = 0;
	m_scrollMapCommands = 0;
	m_AIUnits = 0;
	m_playerUnits = 0;
	m_lastUpdate = 0;
	m_startFrame = TheGameLogic->getFrame();
}
//Destructor
//=============================================================================
StatsCollector::~StatsCollector()
{

}

// Reset and create the file header
//=============================================================================
void StatsCollector::reset()
{

	// make sure we have a stats Dir.
#if defined(RTS_DEBUG)
	if (TheGlobalData->m_saveStats)
	{
		AsciiString playtestDir = TheGlobalData->m_baseStatsDir;
		playtestDir.concat(statsDir);
		if (TheNetwork)
		{
			if (TheLAN)
			{
				TheFileSystem->createDirectory(playtestDir);
			}
		}
	}
#endif
	TheFileSystem->createDirectory(AsciiString(statsDir));
	createFileName();
	writeInitialFileInfo();

	// zero out
	zeroOutStats();

	m_lastUpdate = TheGameLogic->getFrame(); // timeGetTime();
}

// Msgs pass through here so we can track whichever ones we want
//=============================================================================
void StatsCollector::collectMsgStats( const GameMessage *msg )
{
	// We only care about our own messages.
	if(ThePlayerList->getLocalPlayer()->getPlayerIndex() != msg->getPlayerIndex())
		return;

	switch (msg->getType())
	{
		case GameMessage::MSG_QUEUE_UNIT_CREATE:
		case GameMessage::MSG_DOZER_CONSTRUCT:
		case GameMessage::MSG_DOZER_CONSTRUCT_LINE:
			{
				++m_buildCommands;
				break;
			}
	}

}

//Loop through all objects and count up the ones we want. (Very Slow!!!)
//=============================================================================
void StatsCollector::collectUnitCountStats()
{

	for(Object *obj =	TheGameLogic->getFirstObject(); obj; obj = obj->getNextObject())
	{

		if((!(obj->isKindOf(KINDOF_INFANTRY) || obj->isKindOf(KINDOF_VEHICLE))) || ( obj->isNeutralControlled()) ||(obj->getControllingPlayer()->getSide().compare("Civilian") == 0))
			continue;

		if(obj->getControllingPlayer()->isLocalPlayer())
		{
			++m_playerUnits;
		}
		else
		{
			++m_AIUnits;
		}
	}

}

// call every frame and only do stuff when our time is up
//=============================================================================
void StatsCollector::update()
{
	if(m_lastUpdate + (TheGlobalData->m_playStats * LOGICFRAMES_PER_SECOND) > TheGameLogic->getFrame())
		return;

	collectUnitCountStats();

	if(m_isScrolling)
	{
		m_scrollTime += TheGameLogic->getFrame() - m_scrollBeginTime;
		m_scrollBeginTime = TheGameLogic->getFrame();
	}

	m_timeCount += TheGlobalData->m_playStats;
	writeStatInfo();

	zeroOutStats();

	m_lastUpdate = TheGameLogic->getFrame(); //timeGetTime();

}

void StatsCollector::incrementScrollMoveCount()
{
	++m_scrollMapCommands;
}

void StatsCollector::incrementAttackCount()
{
	++m_attackCommands;
}

void StatsCollector::incrementBuildCount()
{
	++m_buildCommands;
}
void StatsCollector::incrementMoveCount()
{
	++m_moveCommands;
}

void StatsCollector::writeFileEnd()
{
	//open the file
	FILE *f = fopen(m_statsFileName.str(), "a");
	if(!f)
	{
		DEBUG_ASSERTCRASH(f, ("Unable to open file %s to write", m_statsFileName.str()));
		return;
	}

	m_timeCount += (TheGameLogic->getFrame() - m_lastUpdate) / LOGICFRAMES_PER_SECOND;
	writeStatInfo();
	fprintf(f, "---------------------------------------------------\n");

		// Time
	struct tm *newTime;
	time_t aclock;
  time( &aclock );
  newTime = localtime( &aclock );
	fprintf(f, "End Time:\t%s\n",asctime(newTime) );

	fprintf(f, "=KEY===============================================\n");
	fprintf(f, "Time* = The Time Interval\n");
	fprintf(f, "BC = Build Commands\n");
	fprintf(f, "MC = Move Commands\n");
	fprintf(f, "AC = Attack Commands\n");
	fprintf(f, "SMC = Scroll Map Commands\n");
	fprintf(f, "ST* = Scroll Time in Seconds\n");
	fprintf(f, "OC = Other Commands (N/A)\n");
	fprintf(f, "$$$ = Local Player's Cash Amount\n");
	fprintf(f, "#PU = # of Player's Units\n");
	fprintf(f, "#AIU = # of AI's Units\n");
	fprintf(f, "===================================================\n");
	fprintf(f, "* Times are in Game Seconds which are based off of frames. Current fps is set to %d\n", LOGICFRAMES_PER_SECOND);

#if defined(RTS_DEBUG)
	if (TheGlobalData->m_benchmarkTimer > 0)
	{
		fprintf(f, "\n*** BENCHMARK MODE STATS ***\n");
		fprintf(f, " Frames = %d\n", TheGameLogic->getFrame()-m_startFrame);
		fprintf(f, "Seconds = %d\n", TheGlobalData->m_benchmarkTimer);
		fprintf(f, "    FPS = %.2f\n", ((Real)TheGameLogic->getFrame()-(Real)m_startFrame)/(Real)TheGlobalData->m_benchmarkTimer);
	}
#endif

	fclose(f);

}

void StatsCollector::startScrollTime()
{
	m_isScrolling = TRUE;
	m_scrollBeginTime = TheGameLogic->getFrame();
	++m_scrollMapCommands;
}

void StatsCollector::endScrollTime()
{
	if(!m_isScrolling)
		return;

	m_isScrolling = FALSE;

	m_scrollTime += TheGameLogic->getFrame() - m_scrollBeginTime;
}

//-----------------------------------------------------------------------------
// PRIVATE FUNCTIONS //////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------

void StatsCollector::zeroOutStats()
{
	m_buildCommands = 0;
	m_moveCommands = 0;
	m_attackCommands = 0;
	m_scrollMapCommands = 0;
	m_AIUnits = 0;
	m_playerUnits = 0;
	m_scrollTime = 0;
}

// create the filename based off of map time and date
//=============================================================================
void StatsCollector::createFileName()
{
	m_statsFileName.clear();
	// Date and Time
	char datestr[256] = "";
	time_t longTime;
	struct tm *curtime;
	time(&longTime);
	curtime = localtime(&longTime);
	strftime(datestr, 256, "_%b%d_%I%M%p", curtime);
//	const MapMetaData *m =  TheMapCache->findMap(TheGlobalData->m_mapName);
	AsciiString name = getFileName(TheGlobalData->m_mapName.str());
	name.truncateBy(4); // ".map"
	m_statsFileName.clear();
#if defined(RTS_DEBUG)
	if (TheGlobalData->m_saveStats)
	{
		m_statsFileName.set(TheGlobalData->m_baseStatsDir);
		m_statsFileName.concat(statsDir);

		if (TheNetwork)
		{
			if (TheLAN)
			{
				GameInfo *game = TheLAN->GetMyGame();
				AsciiString players;
				AsciiString full;
				AsciiString fullPlusNum;
				for (Int i=0; i<MAX_SLOTS; ++i)
				{
					GameSlot *slot = game->getSlot(i);
					if (slot && slot->isHuman())
					{
						AsciiString player;
						player.format("%ls_", slot->getName().str());
						players.concat(player);
					}
				}
				full.format("%s%s_%d_%d", players.str(), name.str(), game->getSeed(), game->getLocalSlotNum());
				AsciiString testString;
				testString.format("%s%s.txt", m_statsFileName.str(), full.str());
				m_statsFileName = testString;
			}
		}
		else
		{
			m_statsFileName.format("%s%s%s.txt",statsDir, name.str(),datestr);
		}
	}
	else
#endif
	{
		m_statsFileName.format("%s%s%s.txt",statsDir, name.str(),datestr);
	}
}

// create the header of the file
//=============================================================================
void StatsCollector::writeInitialFileInfo()
{
	//open the file
	FILE *f = fopen(m_statsFileName.str(), "w");
	if(!f)
	{
		DEBUG_ASSERTCRASH(f, ("Unable to open file %s to write", m_statsFileName.str()));
		return;
	}

	fprintf(f, "---------------------------------------------------\n");
	// Time
	struct tm *newTime;
	time_t aclock;
  time( &aclock );
  newTime = localtime( &aclock );
	fprintf(f, "Date:\t%s",asctime(newTime) );

	// Map
	fprintf(f, "Map:\t%s\n", TheGlobalData->m_mapName.str());

	// Side
	fprintf(f, "Side:\t%s\n", ThePlayerList->getLocalPlayer()->getSide().str());
	fprintf(f, "---------------------------------------------------\n\n");

	fprintf(f, "Time*\tBC\tMC\tAC\tSMC\tST*\tOC\t$$$\t#PU\t#AIU\n");
	collectUnitCountStats();
	Money *m = ThePlayerList->getLocalPlayer()->getMoney();
	fprintf(f, "%d\t%d\t%d\t%d\t%d\t%d\t%d\t%d\t%d\t%d\n", 0, m_buildCommands, m_moveCommands, m_attackCommands,
					m_scrollMapCommands, 0 ,/*other commands*/0, m->countMoney(),
					m_playerUnits, m_AIUnits );
	// initial stats
	// we don't want a file pointer open for seconds on end... we'll open it each time.
	fclose(f);
}

// Write out the stats
//=============================================================================
void StatsCollector::writeStatInfo()
{
	//open the file
	FILE *f = fopen(m_statsFileName.str(), "a");
	if(!f)
	{
		DEBUG_ASSERTCRASH(f, ("Unable to open file %s to write", m_statsFileName.str()));
		return;
	}
	Money *m = ThePlayerList->getLocalPlayer()->getMoney();

	fprintf(f, "%d\t%d\t%d\t%d\t%d\t%d\t%d\t%d\t%d\t%d\n", m_timeCount, m_buildCommands, m_moveCommands, m_attackCommands,
					m_scrollMapCommands, m_scrollTime / LOGICFRAMES_PER_SECOND, /*other commands*/0,m->countMoney() ,
					m_playerUnits, m_AIUnits  );


	fclose(f);

}
```
