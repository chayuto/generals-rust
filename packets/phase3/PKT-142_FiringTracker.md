# Translation Packet `PKT-142`: `FiringTracker`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-142`
- **Work Package:** `WP-3.3` (Phase 3)
- **Complexity & Assigned Tier:** Class `C1` -> `tier:1a-lite`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 2: Handle Component
- **Allowed Rust Output File:** `crates/zh_logic/src/combat/firing_tracker.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (15):**
  - `Common/AudioEventRTS.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/AudioEventRTS.h`
  - `Common/AudioHandleSpecialValues.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/AudioHandleSpecialValues.h`
  - `Common/GameAudio.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameAudio.h`
  - `Common/GameMemory.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameMemory.h`
  - `Common/GameType.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameType.h`
  - `Common/PerfTimer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/PerfTimer.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameLogic/FiringTracker.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/FiringTracker.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/ObjectHelper.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ObjectHelper.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/Weapon.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h`
  - `PreRTS.h`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (6 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferInt( &m_consecutiveShots )`
  3. `xferObjectID( &m_victimID )`
  4. `xferUnsignedInt( &m_frameToStartCooldown )`
  5. `xferUnsignedInt( &m_frameToForceReload )`
  6. `xferUnsignedInt( &m_frameToStopLoopingSound )`
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if RETAIL_COMPATIBLE_XFER_SAVE`

## 2.5 Existing Rust Symbol Index (Use These Exact Paths)
- `zh_logic::combat::armor`: `ArmorTemplate`, `Armor`, `fn`, `ArmorStore`
- `zh_logic::combat::armor_set`: `ArmorTemplateSet`
- `zh_logic::combat::damage`: `PlayerMaskType`, `HUGE_DAMAGE_AMOUNT`, `DamageInfoInput`, `DamageInfoOutput`, `DamageInfo`
- `zh_logic::combat::locomotor`: `BIGNUM`, `LocomotorAppearance`, `fn`, `LOCOMOTOR_APPEARANCE_NAMES`, `LocomotorPriority`, `LOCOMOTOR_PRIORITY_NAMES`, `LocomotorBehaviorZ`, `LOCOMOTOR_BEHAVIOR_Z_NAMES`, `LocomotorTemplate`, `LocomotorStore`, `Locomotor`
- `zh_logic::combat::locomotor_set`: `LOCOMOTORSET_COUNT`, `LocomotorSetType`, `fn`, `LOCOMOTOR_SET_NAMES`, `LocomotorSet`
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
- `zh_logic::economy`: `MAX_PLAYER_COUNT`
- `zh_logic::economy::academy_stats`: `MAX_ADVICE_TIPS`, `FRAMES_BETWEEN_UPDATES`, `DEFAULT_SUPPLY_CENTER_COST`, `KINDOF_DOZER`, `KINDOF_FS_SUPPLY_CENTER`, `KINDOF_COMMANDCENTER`, `KINDOF_INFANTRY`, `KINDOF_VEHICLE`, `KINDOF_HARVESTER`, `KINDOF_HERO`, `KINDOF_FS_STRATEGY_CENTER`, `KINDOF_MONEY_HACKER`, `KINDOF_FS_BLACK_MARKET`, `KINDOF_FS_SUPPLY_DROPZONE`, ... (+9)
- `zh_logic::economy::energy`: `EnergyOwnerCallbacks`, `NoopEnergyCallbacks`, `EnergyObjectInfo`, `Energy`
- `zh_logic::economy::handicap`: `HANDICAP_TYPE_COUNT`, `THING_TYPE_COUNT`, `HandicapType`, `ThingType`, `Handicap`, `fn`
- `zh_logic::economy::mission_stats`: `MissionStats`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/FiringTracker.h`
```cpp
// FILE: FiringTracker.h //////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, February 2002
// Desc:   Keeps track of shots fired and people targeted for weapons that want a history of such a thing
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

#include "Common/GameType.h"
#include "Common/GameMemory.h"
#include "Common/AudioEventRTS.h"
#include "GameLogic/Module/UpdateModule.h"

class Object;
class Weapon;

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
class FiringTrackerModuleData : public ModuleData
{

};

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
class FiringTracker : public UpdateModule
{
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( FiringTracker, FiringTrackerModuleData )
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(FiringTracker, "FiringTrackerPool" )
public:
	FiringTracker(Thing *thing, const ModuleData *modData);
	void shotFired(const Weapon* weaponFired, ObjectID victimID );			///< Owner just fired this weapon at this Object
	ObjectID getLastShotVictim() const { return m_victimID; }						///< get the last victim ID that was shot at
	Int getNumConsecutiveShotsAtVictim( const Object *victim ) const;

	/// this is never disabled, since we want disabled things to continue to slowly "spin down"... (srj)
	virtual DisabledMaskType getDisabledTypesToProcess() const override { return DISABLEDMASK_ALL; }

	virtual UpdateSleepTime update() override;	///< See if spin down is needed because we haven't shot in a while

protected:

	/*
		The firingtracker needs to have its update run after all "normal"
		user update modules, so it redefines this. Please don't redefine this
		for other modules without very careful deliberation. (srj)
	*/
	virtual SleepyUpdatePhase getUpdatePhase() const override
	{
		return PHASE_FINAL;
	}

private:

	void speedUp();		///< I've qualified for an increase in my Object flag status
	void coolDown();	///< I need to slow down because it has been too long since I fired.
	UpdateSleepTime calcTimeToSleep();

private:
	Int							m_consecutiveShots;					///< How many times I have shot at the same thing
	ObjectID				m_victimID;									///< The thing I have shot so many times
	UnsignedInt			m_frameToStartCooldown;			///< This is the frame I should cool down at, and is pushed back every time a shot is fired
 	UnsignedInt			m_frameToForceReload;				///< Even more than AutoReload, this means it will pre-emptively reload instead of event triggering a delay after the last shot

	UnsignedInt			m_frameToStopLoopingSound;	///< if sound is looping, frame to stop looping it (or zero if not looping)
	AudioHandle			m_audioHandle;
};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/FiringTracker.cpp`
```cpp
// FILE: FiringTracker.cpp //////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, February 2002
// Desc:   Keeps track of shots fired and people targeted for weapons that want a history of such a thing
///////////////////////////////////////////////////////////////////////////////////////////////////

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/AudioHandleSpecialValues.h"
#include "Common/GameType.h"
#include "Common/GameAudio.h"
#include "Common/PerfTimer.h"
#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"

#include "GameLogic/FiringTracker.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/ObjectHelper.h"
#include "GameLogic/Object.h"
#include "GameLogic/Weapon.h"


//-------------------------------------------------------------------------------------------------
FiringTracker::FiringTracker(Thing* thing, const ModuleData *modData) : UpdateModule( thing, modData )
{
	m_consecutiveShots = 0;
	m_victimID = INVALID_ID;
	m_frameToStartCooldown = 0;
 	m_frameToForceReload = 0;
	m_frameToStopLoopingSound = 0;
	m_audioHandle = AHSV_NoSound;
	setWakeFrame(getObject(), UPDATE_SLEEP_FOREVER);
}

//-------------------------------------------------------------------------------------------------
FiringTracker::~FiringTracker()
{
	// no need to protect this.
	TheAudio->removeAudioEvent( m_audioHandle );
	m_audioHandle = AHSV_NoSound;
}

//-------------------------------------------------------------------------------------------------
Int FiringTracker::getNumConsecutiveShotsAtVictim( const Object *victim ) const
{
	if( victim == nullptr )
		return 0;// safety, this function is for asking about shots at a victim

	if( victim->getID() != m_victimID )
		return 0;// nope, not shooting at him

	return m_consecutiveShots;// this is how any times I have shot at this hoser right now
}

//-------------------------------------------------------------------------------------------------
void FiringTracker::shotFired(const Weapon* weaponFired, ObjectID victimID)
{
	UnsignedInt now = TheGameLogic->getFrame();
	Object *me = getObject();
	const Object *victim = TheGameLogic->findObjectByID(victimID); // May be null for ground shot

	if( victim && victim->testStatus(OBJECT_STATUS_FAERIE_FIRE) )
	{
		if( !me->testWeaponBonusCondition(WEAPONBONUSCONDITION_TARGET_FAERIE_FIRE) )
		{
			// We shoot faster at guys marked thusly
			me->setWeaponBonusCondition(WEAPONBONUSCONDITION_TARGET_FAERIE_FIRE);
		}
	}
	else
	{
		// A ground shot or the lack of the status on the target will clear this
		if( me->testWeaponBonusCondition(WEAPONBONUSCONDITION_TARGET_FAERIE_FIRE) )
		{
			me->clearWeaponBonusCondition(WEAPONBONUSCONDITION_TARGET_FAERIE_FIRE);
		}
	}

	if( victimID == m_victimID )
	{
		// Shooting at the same guy
		++m_consecutiveShots;
	}
	else if( now < m_frameToStartCooldown )
	{
		// Switching targets within the coast time is valid, and we will not spin down
		++m_consecutiveShots;
		m_victimID = victimID;
	}
	else
	{
		// Start the count over for the new guy
		m_consecutiveShots = 1;
		m_victimID = victimID;
	}

 	// Push back the time that we should force a reload with each shot
 	UnsignedInt autoReloadDelay = weaponFired->getAutoReloadWhenIdleFrames();
 	if( autoReloadDelay > 0 )
 		m_frameToForceReload = now + autoReloadDelay;

	UnsignedInt coast = weaponFired->getContinuousFireCoastFrames();
	if (coast)
		m_frameToStartCooldown = weaponFired->getPossibleNextShotFrame() + coast;
	else
		m_frameToStartCooldown = 0;

	Int shotsNeededOne = weaponFired->getContinuousFireOneShotsNeeded();
	Int shotsNeededTwo = weaponFired->getContinuousFireTwoShotsNeeded();

	if( me->testWeaponBonusCondition( WEAPONBONUSCONDITION_CONTINUOUS_FIRE_MEAN ) )
	{
		// Can either go up or down from here.
		if( m_consecutiveShots < shotsNeededOne )
			coolDown();
		else if( m_consecutiveShots > shotsNeededTwo )
			speedUp();
	}
	else if( me->testWeaponBonusCondition( WEAPONBONUSCONDITION_CONTINUOUS_FIRE_FAST ) )
	{
		// Only place I can go here from here is all the way down
		if( m_consecutiveShots < shotsNeededTwo )
		{
			coolDown();
		}
	}
	else
	{
		// Check to go up
		if( m_consecutiveShots > shotsNeededOne )
			speedUp();
	}

	UnsignedInt fireSoundLoopTime = weaponFired->getFireSoundLoopTime();
	if (fireSoundLoopTime != 0)
	{
		// If the sound has stopped playing, then we need to re-add it.
		if (m_frameToStopLoopingSound == 0 || !TheAudio->isCurrentlyPlaying(m_audioHandle))
		{
			AudioEventRTS audio = weaponFired->getFireSound();
			audio.setObjectID(getObject()->getID());
			m_audioHandle = TheAudio->addAudioEvent( &audio );
		}
		m_frameToStopLoopingSound = now + fireSoundLoopTime;
	}
	else
	{
		AudioEventRTS fireAndForgetSound = weaponFired->getFireSound();
		fireAndForgetSound.setObjectID(getObject()->getID());
		TheAudio->addAudioEvent(&fireAndForgetSound);
		m_frameToStopLoopingSound = 0;
	}


	setWakeFrame(me, calcTimeToSleep());
}

//-------------------------------------------------------------------------------------------------
UpdateSleepTime FiringTracker::update()
{
	//DEBUG_ASSERTCRASH(m_frameToStartCooldown != 0 || m_frameToStopLoopingSound != 0, ("hmm, should be asleep"));

	UnsignedInt now = TheGameLogic->getFrame();

 	// I have been idle long enough that I should reload, so I do not hang around with a near empty clip forever.
 	if( m_frameToForceReload != 0  &&  now >= m_frameToForceReload )
 	{
 		getObject()->reloadAllAmmo(TRUE);
 		m_frameToForceReload = 0;
 	}

	// If it has been too long since I fired.  I have to start over.
	// (don't call if we don't need to cool down... it's expensive!)

	if (m_frameToStopLoopingSound != 0)
	{
		if (now >= m_frameToStopLoopingSound)
		{
			TheAudio->removeAudioEvent( m_audioHandle );
			m_audioHandle = AHSV_NoSound;
			m_frameToStopLoopingSound = 0;
		}
	}

	if( m_frameToStartCooldown != 0 && now > m_frameToStartCooldown )
	{
		m_frameToStartCooldown = now + LOGICFRAMES_PER_SECOND;
		coolDown();// if this is the coolest call to cooldown, it will set m_frameToStartCooldown to zero
		return UPDATE_SLEEP(LOGICFRAMES_PER_SECOND);
	}

	UpdateSleepTime sleepTime = calcTimeToSleep();

	return sleepTime;
}

//-------------------------------------------------------------------------------------------------
UpdateSleepTime FiringTracker::calcTimeToSleep()
{
 	// Figure out the longest amount of time we can sleep as unneeded

 	// If all the timers are off, then we aren't needed at all
 	if (m_frameToStopLoopingSound == 0 && m_frameToStartCooldown == 0 && m_frameToForceReload == 0)
   		return UPDATE_SLEEP_FOREVER;

 	// Otherwise, we need to wake up to service the shortest timer
   	UnsignedInt now = TheGameLogic->getFrame();
 	UnsignedInt sleepTime = UPDATE_SLEEP_FOREVER;
 	if( m_frameToStopLoopingSound != 0 )
 	{
 		if( m_frameToStopLoopingSound <= now )
 			sleepTime = UPDATE_SLEEP_NONE;
 		else if( (m_frameToStopLoopingSound - now) < sleepTime )
 			sleepTime = m_frameToStopLoopingSound - now;
 	}
 	if( m_frameToStartCooldown != 0 )
 	{
 		if( m_frameToStartCooldown <= now )
 			sleepTime = UPDATE_SLEEP_NONE;
 		else if( (m_frameToStartCooldown - now) < sleepTime )
 			sleepTime = m_frameToStartCooldown - now;
 	}
 	if( m_frameToForceReload != 0 )
 	{
 		if( m_frameToForceReload <= now )
 			sleepTime = UPDATE_SLEEP_NONE;
 		else if( (m_frameToForceReload - now) < sleepTime )
 			sleepTime = m_frameToForceReload - now;
 	}

 	return UPDATE_SLEEP(sleepTime);
}

//-------------------------------------------------------------------------------------------------
void FiringTracker::speedUp()
{
	ModelConditionFlags clr, set;
	Object *self = getObject();

	if( self->testWeaponBonusCondition( WEAPONBONUSCONDITION_CONTINUOUS_FIRE_FAST ) )
	{
		//self->clearWeaponBonusCondition( WEAPONBONUSCONDITION_CONTINUOUS_FIRE_MEAN );
		//self->clearModelConditionState( MODELCONDITION_CONTINUOUS_FIRE_MEAN );
		//self->clearModelConditionState( MODELCONDITION_CONTINUOUS_FIRE_SLOW );
	}
	else if(self->testWeaponBonusCondition( WEAPONBONUSCONDITION_CONTINUOUS_FIRE_MEAN ) )
	{
		const AudioEventRTS *soundToPlayPtr = self->getTemplate()->getPerUnitSound( "VoiceRapidFire" );
		AudioEventRTS soundToPlay = *soundToPlayPtr;
		soundToPlay.setObjectID( self->getID() );
		TheAudio->addAudioEvent( &soundToPlay );

		// These flags are exclusive, not cumulative
		self->setWeaponBonusCondition( WEAPONBONUSCONDITION_CONTINUOUS_FIRE_FAST );
		set.set(MODELCONDITION_CONTINUOUS_FIRE_FAST);

		// These flags are exclusive, not cumulative
		self->clearWeaponBonusCondition( WEAPONBONUSCONDITION_CONTINUOUS_FIRE_MEAN );
		clr.set(MODELCONDITION_CONTINUOUS_FIRE_MEAN);
		clr.set(MODELCONDITION_CONTINUOUS_FIRE_SLOW);


	}
	else
	{

		self->setWeaponBonusCondition( WEAPONBONUSCONDITION_CONTINUOUS_FIRE_MEAN );
		set.set(MODELCONDITION_CONTINUOUS_FIRE_MEAN);

		// These flags are exclusive, not cumulative
		self->clearWeaponBonusCondition( WEAPONBONUSCONDITION_CONTINUOUS_FIRE_FAST );
		clr.set(MODELCONDITION_CONTINUOUS_FIRE_FAST);
		clr.set(MODELCONDITION_CONTINUOUS_FIRE_SLOW);

	}

	self->clearAndSetModelConditionFlags(clr, set);

}

//-------------------------------------------------------------------------------------------------
void FiringTracker::coolDown()
{
	ModelConditionFlags clr, set;

	if( getObject()->testWeaponBonusCondition( WEAPONBONUSCONDITION_CONTINUOUS_FIRE_FAST ) ||
	 getObject()->testWeaponBonusCondition( WEAPONBONUSCONDITION_CONTINUOUS_FIRE_MEAN ))
	{

		// Straight to zero from wherever it is
		set.set(MODELCONDITION_CONTINUOUS_FIRE_SLOW);

		getObject()->clearWeaponBonusCondition( WEAPONBONUSCONDITION_CONTINUOUS_FIRE_FAST );
		getObject()->clearWeaponBonusCondition( WEAPONBONUSCONDITION_CONTINUOUS_FIRE_MEAN );
		clr.set(MODELCONDITION_CONTINUOUS_FIRE_FAST);
		clr.set(MODELCONDITION_CONTINUOUS_FIRE_MEAN);


	}
	else // we stop, now
	{

		getObject()->clearWeaponBonusCondition( WEAPONBONUSCONDITION_CONTINUOUS_FIRE_FAST );
		getObject()->clearWeaponBonusCondition( WEAPONBONUSCONDITION_CONTINUOUS_FIRE_MEAN );
		// Just chillin, nothing to change
		clr.set(MODELCONDITION_CONTINUOUS_FIRE_FAST);
		clr.set(MODELCONDITION_CONTINUOUS_FIRE_MEAN);
		clr.set(MODELCONDITION_CONTINUOUS_FIRE_SLOW);

		m_frameToStartCooldown = 0;
	}

	getObject()->clearAndSetModelConditionFlags(clr, set);

	// Start everything over
	m_consecutiveShots = 0;
	m_victimID = INVALID_ID;
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void FiringTracker::crc( Xfer *xfer )
{

	// object helper base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version
	* 2: TheSuperHackers @bugfix bobtista 11/10/2026 Save m_frameToForceReload and m_frameToStopLoopingSound
	*/
// ------------------------------------------------------------------------------------------------
void FiringTracker::xfer( Xfer *xfer )
{

	// version
#if RETAIL_COMPATIBLE_XFER_SAVE
	XferVersion currentVersion = 1;
#else
	XferVersion currentVersion = 2;
#endif
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// object helper base class
	UpdateModule::xfer( xfer );

	// consecutive shots
	xfer->xferInt( &m_consecutiveShots );

	// victim id
	xfer->xferObjectID( &m_victimID );

	// frame to start cooldown
	xfer->xferUnsignedInt( &m_frameToStartCooldown );

	if( version >= 2 )
	{
		// frame to force reload
		xfer->xferUnsignedInt( &m_frameToForceReload );

		// frame to stop looping sound
		xfer->xferUnsignedInt( &m_frameToStopLoopingSound );
	}

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void FiringTracker::loadPostProcess()
{

	// object helper back class
	UpdateModule::loadPostProcess();

}
```
