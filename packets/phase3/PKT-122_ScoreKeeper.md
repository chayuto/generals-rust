# Translation Packet `PKT-122`: `ScoreKeeper`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-122`
- **Work Package:** `WP-3.1` (Phase 3)
- **Complexity & Assigned Tier:** Class `C1` -> `tier:1a-lite`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 2: Handle Component
- **Allowed Rust Output File:** `crates/zh_logic/src/economy/score_keeper.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (11):**
  - `Common/GameState.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/GameState.h`
  - `Common/KindOf.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/KindOf.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/ScoreKeeper.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ScoreKeeper.h`
  - `Common/Snapshot.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Snapshot.h`
  - `Common/ThingFactory.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ThingFactory.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `PreRTS.h`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (20 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUnsignedShort( &mapSize )`
  3. `xferAsciiString( &thingTemplateName )`
  4. `xferInt( &count )`
  5. `xferAsciiString( &thingTemplateName )`
  6. `xferInt( &count )`
  7. `xferVersion( &version, currentVersion )`
  8. `xferInt( &m_totalMoneyEarned )`
  9. `xferInt( &m_totalMoneySpent )`
  10. `xferUser( m_totalUnitsDestroyed, sizeof( Int ) * MAX_PLAYER_COUNT )`
  11. `xferInt( &m_totalUnitsBuilt )`
  12. `xferInt( &m_totalUnitsLost )`
  13. `xferUser( m_totalBuildingsDestroyed, sizeof( Int ) * MAX_PLAYER_COUNT )`
  14. `xferInt( &m_totalBuildingsBuilt )`
  15. `xferInt( &m_totalBuildingsLost )`
  16. `xferInt( &m_totalTechBuildingsCaptured )`
  17. `xferInt( &m_totalFactionBuildingsCaptured )`
  18. `xferInt( &m_currentScore )`
  19. `xferInt( &m_myPlayerIdx )`
  20. `xferUnsignedShort( &destroyedArraySize )`

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

### `GeneralsMD/Code/GameEngine/Include/Common/ScoreKeeper.h`
```cpp
// FILE: ScoreKeeper.h /////////////////////////////////////////////////
//-----------------------------------------------------------------------------
//
//                       Electronic Arts Pacific.
//
//                       Confidential Information
//                Copyright (C) 2002 - All Rights Reserved
//
//-----------------------------------------------------------------------------
//
//	created:	Jun 2002
//
//	Filename: 	ScoreKeeper.h
//
//	author:		Chris Huybregts
//
//	purpose:	Header file for the scorekeeper class
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
#include "Common/Snapshot.h"

//-----------------------------------------------------------------------------
// FORWARD REFERENCES /////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------
class Object;
class ThingTemplate;
//-----------------------------------------------------------------------------
// TYPE DEFINES ///////////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------
class ScoreKeeper : public Snapshot
{
public:
	ScoreKeeper();
	~ScoreKeeper();

	void reset( Int playerIdx );			///< Zero out the variables
	Int calculateScore();				///< Performs the equation to calculate the score

	void addMoneySpent( Int money );								///< Adds money to the amount spent
	void addMoneyEarned( Int money );								///< Adds money to the earned amount

	void addObjectBuilt( const Object *o );
	void addObjectDestroyed( const Object *o );
	void addObjectLost( const Object *o );
	void addObjectCaptured( const Object *o );

	void removeObjectBuilt( const Object *o );

	Int getTotalMoneyEarned() { return m_totalMoneyEarned; }
	Int getTotalMoneySpent() { return m_totalMoneySpent; }
	Int getTotalUnitsDestroyed();
	Int getTotalUnitsBuilt() { return m_totalUnitsBuilt; }
	Int getTotalUnitsLost() { return m_totalUnitsLost; }
	Int getTotalBuildingsDestroyed();
	Int getTotalBuildingsBuilt() { return m_totalBuildingsBuilt; }
	Int getTotalBuildingsLost() { return m_totalBuildingsLost; }
	Int getTotalTechBuildingsCaptured() { return m_totalTechBuildingsCaptured; }
	Int getTotalFactionBuildingsCaptured() { return m_totalFactionBuildingsCaptured; }
	Int getTotalObjectsBuilt( const ThingTemplate *pTemplate ); // get a count of objects built matching a specific thing template

	// for battle honor calculation.  done once at the end of each online game
	Int getTotalUnitsBuilt( KindOfMaskType validMask, KindOfMaskType invalidMask );

protected:

	// snapshot methods
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

private:

	Int m_totalMoneyEarned;						///< The total money that was harvested, refined, received in crates
	Int m_totalMoneySpent;						///< The total money spent on units, buildings, repairs
	Int m_totalUnitsDestroyed[MAX_PLAYER_COUNT];				///< The total number of enemies that we've killed
	Int m_totalUnitsBuilt;						///< The total number of units we've created (created meaning that we built from a building)
	Int m_totalUnitsLost;							///< The total number of our units lost
	Int m_totalBuildingsDestroyed[MAX_PLAYER_COUNT];		///< The total number of Buildings we've destroyed
	Int m_totalBuildingsBuilt;				///< The total number of buildings we've constructed
	Int m_totalBuildingsLost;					///< The total number of our buildings lost
	Int m_totalTechBuildingsCaptured;	///< The total number of tech buildings captured
	Int m_totalFactionBuildingsCaptured;	///< The total number of faction buildings captured
	Int m_currentScore;								///< Based off the stats, this is the current score

	Int m_myPlayerIdx;								///< We need to not score kills on ourselves... so we need to know who we are

	typedef std::map<const ThingTemplate *, Int> ObjectCountMap;
	typedef ObjectCountMap::iterator ObjectCountMapIt;
	ObjectCountMap m_objectsBuilt;			///< How many and what kinds of objects did we build
	ObjectCountMap m_objectsDestroyed[MAX_PLAYER_COUNT];		///< How many and what kinds and who's did we kill
	ObjectCountMap m_objectsLost;				///< how many and what kinds of objects did we loose
	ObjectCountMap m_objectsCaptured;
	void xferObjectCountMap( Xfer *xfer, ObjectCountMap *map );

};

//-----------------------------------------------------------------------------
// INLINING ///////////////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------
inline void ScoreKeeper::addMoneySpent( Int money )							{	m_totalMoneySpent += money; }
inline void ScoreKeeper::addMoneyEarned( Int money )						{	m_totalMoneyEarned += money; }

//-----------------------------------------------------------------------------
// EXTERNALS //////////////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------
```

### `GeneralsMD/Code/GameEngine/Source/Common/RTS/ScoreKeeper.cpp`
```cpp
// FILE: ScoreKeeper.cpp /////////////////////////////////////////////////
//-----------------------------------------------------------------------------
//
//                       Electronic Arts Pacific.
//
//                       Confidential Information
//                Copyright (C) 2002 - All Rights Reserved
//
//-----------------------------------------------------------------------------
//
//	created:	Jun 2002
//
//	Filename: 	ScoreKeeper.cpp
//
//	author:		Chris Huybregts
//
//	purpose:	Score Keeper class will be an object attached to each player
//						that will maintain accurate counts for the various stats we
//						want to show on the score screen.  The information in here
//						could also be used for the observer screen
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

#include "Common/GameState.h"
#include "Common/KindOf.h"
#include "Common/Player.h"
#include "Common/ScoreKeeper.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"

#include "GameLogic/Object.h"
#include "GameLogic/GameLogic.h"

//-----------------------------------------------------------------------------
// DEFINES ////////////////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------

//-----------------------------------------------------------------------------
// PUBLIC FUNCTIONS ///////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------

ScoreKeeper::ScoreKeeper()
{
	reset(0);
}

ScoreKeeper::~ScoreKeeper()
{

}

static KindOfMaskType scoringBuildingMask;
static KindOfMaskType scoringBuildingDestroyMask;
static KindOfMaskType scoringBuildingCreateMask;

void ScoreKeeper::reset( Int playerIdx )
{
	scoringBuildingMask.set(KINDOF_STRUCTURE);
	scoringBuildingMask.set(KINDOF_SCORE);

	scoringBuildingCreateMask.set(KINDOF_STRUCTURE);
	scoringBuildingCreateMask.set(KINDOF_SCORE_CREATE);

	scoringBuildingDestroyMask.set(KINDOF_STRUCTURE);
	scoringBuildingDestroyMask.set(KINDOF_SCORE_DESTROY);

	m_totalMoneyEarned = m_totalMoneySpent = 0;
	m_totalUnitsLost = m_totalUnitsBuilt = 0;
	m_totalBuildingsLost = m_totalBuildingsBuilt = 0;
	m_totalFactionBuildingsCaptured = m_totalTechBuildingsCaptured = 0;
	m_currentScore = 0;
	m_objectsBuilt.clear();
	m_objectsCaptured.clear();
	m_objectsLost.clear();
	for(int i = 0; i < MAX_PLAYER_COUNT; ++i)
	{
		m_objectsDestroyed[i].clear();
		m_totalBuildingsDestroyed[i] = m_totalUnitsDestroyed[i] = 0;
	}
	m_myPlayerIdx	= playerIdx;
}

void ScoreKeeper::addObjectBuilt( const Object *o)
{
	Bool addToCount = FALSE;

	if (TheGameLogic->isScoringEnabled() == FALSE) {
		return;
	}

	if(o->getTemplate()->isKindOfMulti(scoringBuildingMask, KINDOFMASK_NONE))
	{
		++m_totalBuildingsBuilt;
		addToCount = TRUE;
	}
	else if (o->getTemplate()->isKindOfMulti(scoringBuildingCreateMask, KINDOFMASK_NONE))
	{
		++m_totalBuildingsBuilt;
		addToCount = TRUE;
	}
	else if(o->getTemplate()->isKindOf(KINDOF_INFANTRY) || o->getTemplate()->isKindOf(KINDOF_VEHICLE))
	{
		if (o->getTemplate()->isKindOf(KINDOF_SCORE) || o->getTemplate()->isKindOf(KINDOF_SCORE_CREATE))
		{
			++m_totalUnitsBuilt;
			addToCount = TRUE;
		}
	}

	if(addToCount)
	{
		Int existingCount = 0;
		ObjectCountMapIt it = m_objectsBuilt.find(o->getTemplate());
		if (it != m_objectsBuilt.end())
			existingCount = it->second;
		m_objectsBuilt[o->getTemplate()] = existingCount + 1;
	}
}

Int ScoreKeeper::getTotalUnitsBuilt( KindOfMaskType validMask, KindOfMaskType invalidMask )
{
	Int count = 0;
	for (ObjectCountMapIt it = m_objectsBuilt.begin(); it != m_objectsBuilt.end(); ++it)
	{
		const ThingTemplate *theTemplate = it->first;
		Int numBuilt = it->second;
		if (theTemplate && theTemplate->isKindOfMulti(validMask, invalidMask))
			count += numBuilt;
	}
	return count;
}

Int ScoreKeeper::getTotalObjectsBuilt( const ThingTemplate *pTemplate )
{
	Int count = 0;
	for (ObjectCountMapIt it = m_objectsBuilt.begin(); it != m_objectsBuilt.end(); ++it)
	{
		const ThingTemplate *theTemplate = it->first;
		if (theTemplate && theTemplate->isEquivalentTo(pTemplate))
			++count;
	}
	return count;
}


void ScoreKeeper::removeObjectBuilt( const Object *o)
{
	if (TheGameLogic->isScoringEnabled() == FALSE) {
		return;
	}

	Bool removeFromCount = FALSE;
	if (o->getTemplate()->isKindOfMulti(scoringBuildingMask, KINDOFMASK_NONE))
	{
		--m_totalBuildingsBuilt;
		removeFromCount = TRUE;
	}
	else if (o->getTemplate()->isKindOfMulti(scoringBuildingCreateMask, KINDOFMASK_NONE))
	{
		--m_totalBuildingsBuilt;
		removeFromCount = TRUE;
	}
	else if (o->getTemplate()->isKindOf(KINDOF_INFANTRY) || o->getTemplate()->isKindOf(KINDOF_VEHICLE))
	{
		if (o->getTemplate()->isKindOf(KINDOF_SCORE) || o->getTemplate()->isKindOf(KINDOF_SCORE_CREATE))
		{
			--m_totalUnitsBuilt;
			removeFromCount = TRUE;
		}
	}

	if (removeFromCount)
	{
		Int existingCount = 0;
		ObjectCountMapIt it = m_objectsBuilt.find(o->getTemplate());
		if (it != m_objectsBuilt.end())
			existingCount = it->second;
		m_objectsBuilt[o->getTemplate()] = existingCount - 1;
	}
}

void ScoreKeeper::addObjectCaptured( const Object *o )
{
	if (TheGameLogic->isScoringEnabled() == FALSE) {
		return;
	}

	Bool addToCount = FALSE;
	if(o->getTemplate()->isKindOf(KINDOF_STRUCTURE))
	{
		if (o->getTemplate()->isKindOf(KINDOF_SCORE))
		{
			++m_totalFactionBuildingsCaptured;
		}
		else
		{
			++m_totalTechBuildingsCaptured;
		}
		addToCount = TRUE;
	}

	if(addToCount)
	{
		Int existingCount = 0;
		ObjectCountMapIt it = m_objectsCaptured.find(o->getTemplate());
		if (it != m_objectsCaptured.end())
			existingCount = it->second;
		m_objectsCaptured[o->getTemplate()] = existingCount + 1;
	}
}



void ScoreKeeper::addObjectDestroyed( const Object *o)
{

	if (TheGameLogic->isScoringEnabled() == FALSE) {
		return;
	}

	Int playerIdx = o->getControllingPlayer()->getPlayerIndex();

	Bool addToCount = FALSE;
	if(o->getTemplate()->isKindOfMulti(scoringBuildingMask, KINDOFMASK_NONE))
	{
		if (!(o->testStatus(OBJECT_STATUS_UNDER_CONSTRUCTION))) {
			++m_totalBuildingsDestroyed[playerIdx];
			addToCount = TRUE;
		}
	}
	else if (o->getTemplate()->isKindOfMulti(scoringBuildingDestroyMask, KINDOFMASK_NONE))
	{
		if (!(o->testStatus(OBJECT_STATUS_UNDER_CONSTRUCTION))) {
			++m_totalBuildingsDestroyed[playerIdx];
			addToCount = TRUE;
		}
	}
	else if(o->getTemplate()->isKindOf(KINDOF_INFANTRY) || o->getTemplate()->isKindOf(KINDOF_VEHICLE))
	{
		if (o->getTemplate()->isKindOf(KINDOF_SCORE) || o->getTemplate()->isKindOf(KINDOF_SCORE_DESTROY))
		{
			if (!(o->testStatus(OBJECT_STATUS_UNDER_CONSTRUCTION))) {
				m_totalUnitsDestroyed[playerIdx]++;
				addToCount = TRUE;
			}
		}
	}

	if(addToCount)
	{
		Int existingCount = 0;
		ObjectCountMapIt it = m_objectsDestroyed[playerIdx].find(o->getTemplate());
		if (it != m_objectsDestroyed[playerIdx].end())
			existingCount = it->second;
		m_objectsDestroyed[playerIdx][o->getTemplate()] = existingCount + 1;
	}
}

void ScoreKeeper::addObjectLost( const Object *o )
{
	if (TheGameLogic->isScoringEnabled() == FALSE) {
		return;
	}

	Bool addToCount = FALSE;
	if(o->getTemplate()->isKindOfMulti(scoringBuildingMask, KINDOFMASK_NONE))
	{
		if (!(o->testStatus(OBJECT_STATUS_UNDER_CONSTRUCTION))) {
			++m_totalBuildingsLost;
			addToCount = TRUE;
		}
	}
	else if (o->getTemplate()->isKindOfMulti(scoringBuildingDestroyMask, KINDOFMASK_NONE))
	{
		if (!(o->testStatus(OBJECT_STATUS_UNDER_CONSTRUCTION))) {
			++m_totalBuildingsLost;
			addToCount = TRUE;
		}
	}
	else if(o->getTemplate()->isKindOf(KINDOF_INFANTRY) || o->getTemplate()->isKindOf(KINDOF_VEHICLE))
	{
		if (o->getTemplate()->isKindOf(KINDOF_SCORE) || o->getTemplate()->isKindOf(KINDOF_SCORE_DESTROY))
		{
			if (!(o->testStatus(OBJECT_STATUS_UNDER_CONSTRUCTION))) {
				++m_totalUnitsLost;
				addToCount = TRUE;
			}
		}
	}

	if(addToCount)
	{
		Int existingCount = 0;
		ObjectCountMapIt it = m_objectsLost.find(o->getTemplate());
		if (it != m_objectsLost.end())
			existingCount = it->second;
		m_objectsLost[o->getTemplate()] = existingCount + 1;
	}
}

Int ScoreKeeper::calculateScore()
{
	Int score = 0;
	score += m_totalUnitsBuilt * 100;
	score += m_totalMoneyEarned;
	score += m_totalBuildingsBuilt * 100;
	for (Int i = 0; i < MAX_PLAYER_COUNT; ++i)
	{
		if(i == m_myPlayerIdx)
			continue;
		score += m_totalUnitsDestroyed[i] * 100;
		score += m_totalBuildingsDestroyed[i] * 100;
	}

	m_currentScore = score;
	return m_currentScore;

}

//-----------------------------------------------------------------------------
// PRIVATE FUNCTIONS //////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------

Int ScoreKeeper::getTotalBuildingsDestroyed()
{
	int count = 0;
	for (int i = 0; i< MAX_PLAYER_COUNT; ++i)
	{
	// Design change, display even if we killed our own
//		if(i == m_myPlayerIdx)
//			continue;
		count += m_totalBuildingsDestroyed[i];
		//for (ObjectCountMapIt it = m_objectsDestroyed[i].begin(); it != m_objectsDestroyed[i].end(); ++it)
		//		{
		//
		//			count += it->second;
		//		}

	}
	return count;
}
Int ScoreKeeper::getTotalUnitsDestroyed()
{
	int count = 0;
	for (int i = 0; i< MAX_PLAYER_COUNT; ++i)
	{
	// Design change, display even if we killed our own
//		if(i == m_myPlayerIdx)
//			continue;
		count += m_totalUnitsDestroyed[i];
//		for (ObjectCountMapIt it = m_objectsDestroyed[i].begin(); it != m_objectsDestroyed[i].end(); ++it)
//		{
//		}
	}
	return count;
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void ScoreKeeper::crc( Xfer *xfer )
{

}

// ------------------------------------------------------------------------------------------------
/** Xfer of an object count map
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void ScoreKeeper::xferObjectCountMap( Xfer *xfer, ObjectCountMap *map )
{

	// sanity
	if( map == nullptr )
	{

		DEBUG_CRASH(( "xferObjectCountMap - Invalid map parameter" ));
		throw SC_INVALID_DATA;

	}

	// version info
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// size of the map
	UnsignedShort mapSize = map->size();
	xfer->xferUnsignedShort( &mapSize );

	// map data
	Int count;
	const ThingTemplate *thingTemplate;
	AsciiString thingTemplateName;
	if( xfer->getXferMode() == XFER_SAVE )
	{
		ObjectCountMapIt it;

		// save all entries
		for( it = map->begin(); it != map->end(); ++it )
		{

			// thing template
			thingTemplate = it->first;
			thingTemplateName = thingTemplate->getName();
			xfer->xferAsciiString( &thingTemplateName );

			// the count
			count = it->second;
			xfer->xferInt( &count );

		}

	}
	else
	{

		// read all entries
		for( UnsignedShort i = 0; i < mapSize; ++i )
		{

			// read thing template name
			xfer->xferAsciiString( &thingTemplateName );
			thingTemplate = TheThingFactory->findTemplate( thingTemplateName );
			if( thingTemplate == nullptr )
			{

				DEBUG_CRASH(( "xferObjectCountMap - Unknown thing template '%s'", thingTemplateName.str() ));
				throw SC_INVALID_DATA;

			}

			// read count
			xfer->xferInt( &count );

			// add to map
			(*map)[ thingTemplate ] = count;

		}

	}

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void ScoreKeeper::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// money earned
	xfer->xferInt( &m_totalMoneyEarned );

	// money spent
	xfer->xferInt( &m_totalMoneySpent );

	// units destroyed
	xfer->xferUser( m_totalUnitsDestroyed, sizeof( Int ) * MAX_PLAYER_COUNT );

	// units built
	xfer->xferInt( &m_totalUnitsBuilt );

	// units lost
	xfer->xferInt( &m_totalUnitsLost );

	// buildings destroyed
	xfer->xferUser( m_totalBuildingsDestroyed, sizeof( Int ) * MAX_PLAYER_COUNT );

	// buildings built
	xfer->xferInt( &m_totalBuildingsBuilt );

	// buildings lost
	xfer->xferInt( &m_totalBuildingsLost );

	// tech buildings captured
	xfer->xferInt( &m_totalTechBuildingsCaptured );

	// faction buildings captured
	xfer->xferInt( &m_totalFactionBuildingsCaptured );

	// current score
	xfer->xferInt( &m_currentScore );

	// player index
	xfer->xferInt( &m_myPlayerIdx );

	// objects built
	xferObjectCountMap( xfer, &m_objectsBuilt );

	// objects destroyed
	UnsignedShort destroyedArraySize = MAX_PLAYER_COUNT;
	xfer->xferUnsignedShort( &destroyedArraySize );
	if( destroyedArraySize != MAX_PLAYER_COUNT )
	{

		DEBUG_CRASH(( "ScoreKeeper::xfer - size of objects destroyed array has changed" ));
		throw SC_INVALID_DATA;

	}
	for( UnsignedShort i = 0; i < destroyedArraySize; ++i )
	{

		// xfer map data
		xferObjectCountMap( xfer, &m_objectsDestroyed[ i ] );

	}

	// objects lost
	xferObjectCountMap( xfer, &m_objectsLost );

	// objects captured
	xferObjectCountMap( xfer, &m_objectsCaptured );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void ScoreKeeper::loadPostProcess()
{

}
```
