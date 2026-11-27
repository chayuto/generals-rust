# Translation Packet `PKT-428`: `AIStateMachine`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-428`
- **Work Package:** `WP-6.3` (Phase 6)
- **Complexity & Assigned Tier:** Class `C4` -> `tier:3-architect`
- **Environment Access:** `env:hybrid-ci`
- **FFI Seam Pattern:** Core Spine
- **Allowed Rust Output File:** `crates/zh_logic/src/ai/ai_state_machine.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (5):**
  - `Common/AudioEventRTS.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/AudioEventRTS.h`
  - `Common/GameMemory.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameMemory.h`
  - `Common/StateMachine.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/StateMachine.h`
  - `GameLogic/TerrainLogic.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/TerrainLogic.h`
  - `Lib/BaseType.h`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (4 calls — preserve order identically):**
  1. `xferVersion( &v, cv )`
  2. `xferVersion( &v, cv )`
  3. `xferVersion( &v, cv )`
  4. `xferVersion( &v, cv )`

## 2.5 Existing Rust Symbol Index (Use These Exact Paths)
- `zh_logic::ai`: `ResidentEntries`
- `zh_logic::ai::ai_state_machine`: `IDLE`, `MOVE_TO`, `FOLLOW_WAYPOINT_PATH_AS_TEAM`, `FOLLOW_WAYPOINT_PATH_AS_INDIVIDUALS`, `FOLLOW_WAYPOINT_PATH_AS_TEAM_EXACT`, `FOLLOW_WAYPOINT_PATH_AS_INDIVIDUALS_EXACT`, `FOLLOW_PATH`, `FOLLOW_EXITPRODUCTION_PATH`, `WAIT`, `ATTACK_POSITION`, `ATTACK_OBJECT`, `FORCE_ATTACK_OBJECT`, `ATTACK_AND_FOLLOW_OBJECT`, `DEAD`, ... (+31)
- `zh_logic::ai::ai_states`: `IDLE_COUNTDOWN_DELAY`, `IdleState`, `fn`
- `zh_logic::ai::seams`: `PathServices`, `NoPathServices`, `PathUnitQueries`, `WaypointInfo`, `MapTerrain`, `AiActionQuery`, `AiSeams`, `NoAiSeams`, `SharedAiSeams`, `SeamsGuard`
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
- `zh_logic::modules::behavior::dumb_projectile_behavior`: `DumbProjectileBehaviorModuleData`, `DumbProjectileBehavior`
- `zh_logic::modules::behavior::fire_weapon_when_damaged_behavior`: `FireWeaponWhenDamagedBehaviorModuleData`, `FireWeaponWhenDamagedBehavior`
- `zh_logic::modules::behavior::fire_weapon_when_dead_behavior`: `FireWeaponWhenDeadBehaviorModuleData`, `FireWeaponWhenDeadBehavior`
- `zh_logic::modules::behavior::generate_minefield_behavior`: `GenerateMinefieldBehaviorModuleData`, `GenerateMinefieldBehavior`
- `zh_logic::modules::behavior::grant_stealth_behavior`: `GrantStealthBehaviorModuleData`, `GrantStealthBehavior`
- `zh_logic::modules::behavior::instant_death_behavior`: `InstantDeathBehaviorModuleData`, `InstantDeathBehavior`
- `zh_logic::modules::behavior::jet_slow_death_behavior`: `JetSlowDeathBehaviorModuleData`, `JetSlowDeathBehavior`
- `zh_logic::modules::behavior::minefield_behavior`: `MinefieldBehaviorModuleData`, `ImmuneInfo`, `MinefieldBehavior`
- `zh_logic::modules::behavior::neuton_blast_behavior`: `NeutronBlastBehaviorModuleData`, `NeutronBlastBehavior`
- `zh_logic::modules::behavior::overcharge_behavior`: `OverchargeBehaviorModuleData`, `OverchargeBehavior`
- `zh_logic::modules::behavior::parking_place_behavior`: `ParkingPlaceBehaviorModuleData`, `ParkingSpace`, `Runway`, `Healing`, `ParkingPlaceBehavior`
- `zh_logic::modules::behavior::poisoned_behavior`: `PoisonedBehaviorModuleData`, `PoisonedBehavior`
- `zh_logic::modules::behavior::prison_behavior`: `PrisonBehaviorModuleData`, `PrisonVisual`, `PrisonBehavior`
- `zh_logic::modules::behavior::propaganda_tower_behavior`: `PropagandaTowerBehaviorModuleData`, `PropagandaTowerBehavior`
- `zh_logic::modules::behavior::slow_death_behavior`: `SLOW_DEATH_PHASE_NAMES`, `SlowDeathPhaseType`, `COUNT`, `SD_FLAG_SLOW_DEATH_ACTIVATED`, `SD_FLAG_MIDPOINT_EXECUTED`, `SD_FLAG_FLUNG_INTO_AIR`, `SD_FLAG_BOUNCED`, `SD_EFFECT_HAS_FX`, `SD_EFFECT_HAS_OCL`, `SD_EFFECT_HAS_WEAPON`, `SlowDeathBehaviorModuleData`, `fn`, `SlowDeathCore`, `SlowDeathBehavior`
- `zh_logic::modules::behavior::spawn_behavior`: `SpawnBehaviorModuleData`, `SpawnBehavior`
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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/AIStateMachine.h`
```cpp
// AIStateMachine.h
// Finite state machine encapsulation
// Author: Michael S. Booth, January 2002

#pragma once

#include "Lib/BaseType.h"

#include "Common/AudioEventRTS.h"
#include "Common/GameMemory.h"
#include "Common/StateMachine.h"

#include "GameLogic/TerrainLogic.h"

// FORWARD DECLARATIONS ///////////////////////////////////////////////////////////////////////////
class AIGuardMachine;
class AIGuardRetaliateMachine;
class AITNGuardMachine;
class Weapon;
class Team;
class AIAttackState;
class AttackStateMachine;
class AIGroup;
class Squad;

//-----------------------------------------------------------------------------------------------------------


/**
 * The AI state IDs.
 * Each of these constants will be associated with an instance of a State class
 * in a given StateMachine.
 */
enum AIStateType CPP_11(: Int)
{
	AI_IDLE,
	AI_MOVE_TO,																///< move to the GoalObject or GoalPosition
	AI_FOLLOW_WAYPOINT_PATH_AS_TEAM,					///< follow a waypoint path as a team
	AI_FOLLOW_WAYPOINT_PATH_AS_INDIVIDUALS,		///< follow a waypoint path
	AI_FOLLOW_WAYPOINT_PATH_AS_TEAM_EXACT,		///< follow a waypoint path as a team
	AI_FOLLOW_WAYPOINT_PATH_AS_INDIVIDUALS_EXACT,///< follow a waypoint path
	AI_FOLLOW_PATH,														///< follow a simple list of points
	AI_FOLLOW_EXITPRODUCTION_PATH,						///< the same, but only when exiting production facility
	AI_WAIT,
	AI_ATTACK_POSITION,												///< attempt to kill GoalPosition
	AI_ATTACK_OBJECT,													///< attempt to kill GoalObject
	AI_FORCE_ATTACK_OBJECT,										///< attempt to kill GoalObject, force fire on it.
	AI_ATTACK_AND_FOLLOW_OBJECT,							///< attempt to kill GoalObject, following it if necessary (and possible)
	AI_DEAD,
	AI_DOCK,																	///< dock with GoalObject, if GoalObject has a DockUpdate module
	AI_ENTER,																	///< move to GoalObject and "enter" it when close
	AI_GUARD,																	///< guard your current location
	AI_HUNT,																	///< seek and destroy behavior
	AI_WANDER,																///< Wander around following a waypoint path.
	AI_PANIC,																	///< Run around screaming following a waypoint path.
	AI_ATTACK_SQUAD,													///< Set the unit to attempt to kill all objects in goalSquad.
	AI_GUARD_TUNNEL_NETWORK,									///< Guard from inside a tunnel network.
	AI_GET_REPAIRED,													///< Get repaired at a repair depot
	AI_MOVE_OUT_OF_THE_WAY,										///< Move out of the way of another unit.
	AI_MOVE_AND_TIGHTEN,											///< Move in order to tighten up a formation.
	AI_MOVE_AND_EVACUATE,											///< Move to, then empty transport.
	AI_MOVE_AND_EVACUATE_AND_EXIT,						///< Move to, then empty transport.
	AI_MOVE_AND_DELETE,												///< Move to, then delete self.
	AI_ATTACK_AREA,														///< Attack units in an area.
	AI_HACK_INTERNET,													///< Hack internet for free money (no target required).
	AI_ATTACK_MOVE_TO,												///< Attack-move to a location
	AI_ATTACKFOLLOW_WAYPOINT_PATH_AS_INDIVIDUALS,	///< Attack-Follow down a waypoint path as individuals
	AI_ATTACKFOLLOW_WAYPOINT_PATH_AS_TEAM,				///< Attack-Follow down a waypoint path as a team
	AI_FACE_OBJECT,
	AI_FACE_POSITION,
	AI_RAPPEL_INTO,														/**< rappel from current pos down to target object.
																								if target is building, will enter and kill lots of folks.
																								if target is null, will rappel to ground. */
	AI_COMBATDROP,														/**< attempt to send AI_RAPPEL_INTO to contents. */
	AI_EXIT,																	///< exit the obj, waiting if necessary
	AI_PICK_UP_CRATE,													///< Pick up a crate created by a kill.  jba.
	AI_MOVE_AWAY_FROM_REPULSORS,							///< Civilians are running away from repulsors. (enemies or dead civs, usually) jba
	AI_WANDER_IN_PLACE,												///< Civilians just wander around a spot, rather than along a path.
	AI_BUSY,																	///< This is a state that things will be in when they are busy doing random stuff that doesn't require AI interaction.
	AI_EXIT_INSTANTLY,												///< exit this obj, without waiting -- do it in the onEnter! This frame!
	AI_GUARD_RETALIATE,												///< attacks attacker but with restrictions (hybrid of attack and guard).
};

//-----------------------------------------------------------------------------------------------------------
// generically state transition conditions
extern Bool outOfWeaponRangeObject( State *thisState, void* userData );
extern Bool outOfWeaponRangePosition( State *thisState, void* userData );
extern Bool wantToSquishTarget( State *thisState, void* userData );

//-----------------------------------------------------------------------------------------------------------
/**
  The AI state machine.  This is used by AIUpdate to implement all of the
  commands in the AICommandInterface.

	NOTE NOTE NOTE NOTE NOTE

	Do NOT subclass this unless you want ALL of the states this machine possesses.
	If you only want SOME of the states, please make a new StateMachine, descended
	from StateMachine, NOT AIStateMachine. Thank you. (srj)

	NOTE NOTE NOTE NOTE NOTE

 */
class AIStateMachine : public StateMachine
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( AIStateMachine, "AIStateMachine" );

public:
	/**
	 * The implementation of this constructor defines the states
	 * used by this machine.
	 */
	AIStateMachine( Object *owner, AsciiString name );

	virtual void clear() override;
	virtual StateReturnType resetToDefaultState() override;
	virtual StateReturnType setState( StateID newStateID ) override;

	/// @todo Rethink state parameter passing. Continuing in this fashion will have a pile of params in the machine (MSB)
	void setGoalPath( std::vector<Coord3D>* path );
	void addToGoalPath( const Coord3D *pathPoint );
	const Coord3D *getGoalPathPosition( Int i ) const;		///< return path position at index "i"
	Int getGoalPathSize() const { return m_goalPath.size(); }


	void setGoalWaypoint( const Waypoint *way );		///< move toward this waypoint, continue if connected
	const Waypoint *getGoalWaypoint();

	// All of these wind up storing a Squad, as it is the only one that can always be safely created.
	void setGoalTeam( const Team *team );
	void setGoalSquad( const Squad *squad );
	void setGoalAIGroup( const AIGroup *group );

	Squad *getGoalSquad();

	StateReturnType setTemporaryState( StateID newStateID, Int frameLimitCoount );			///< change the temporary state of the machine, and number of frames limit.
	StateID getTemporaryState() const {return m_temporaryState?m_temporaryState->getID():INVALID_STATE_ID;}

public:	// overrides.
	virtual StateReturnType updateStateMachine() override;				///< run one step of the machine
#ifdef STATE_MACHINE_DEBUG
	virtual AsciiString getCurrentStateName() const override;
#endif

protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

private:
	std::vector<Coord3D>	m_goalPath;					///< defines a simple path to follow
	const Waypoint *			m_goalWaypoint;
	Squad *								m_goalSquad;

	/** A temporary state to run for a while (usually AI_MOVE_OUT_OF_THE_WAY).
	Doesn't clear or reset the state machine, so it goes back to doing what it was doing.  jba. */
	State									*m_temporaryState;
	UnsignedInt						m_temporaryStateFramEnd; ///< Last frame to run m_temporaryState.
};


//-----------------------------------------------------------------------------------------------------------
class AttackStateMachine : public StateMachine
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( AttackStateMachine, "AttackStateMachine" );

public:
// Attack states.
enum StateType CPP_11(: Int)
	{
		CHASE_TARGET,													///< Chase a moving target (optionally following it)
		APPROACH_TARGET,											///< Approach a non-moving target.
		AIM_AT_TARGET,												///< rotate to face GoalObject or GoalPosition
		FIRE_WEAPON,													///< fire the machine owner's current weapon
	};
	AttackStateMachine( Object *owner, AIAttackState* att, AsciiString name, Bool follow, Bool attackingObject, Bool forceAttacking );

protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;
};

//-----------------------------------------------------------------------------------------------------------
class AIIdleState : public State
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIIdleState, "AIIdleState")
private:

	UnsignedShort m_initialSleepOffset;
	Bool m_shouldLookForTargets;
	Bool m_inited;

	void doInitIdleState();

public:
	enum AIIdleTargetingType
	{
		LOOK_FOR_TARGETS,
		DO_NOT_LOOK_FOR_TARGETS
	};
	virtual Bool isIdle() const override { return true; }
	AIIdleState( StateMachine *machine, AIIdleTargetingType shouldLookForTargets);
	virtual StateReturnType onEnter() override;
	virtual StateReturnType update() override;

protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

};
EMPTY_DTOR(AIIdleState)

//-----------------------------------------------------------------------------------------------------------
/**
 * Basic pathfinding and moving to a goal position.
 * Not for direct use (hence no associated state ID), but for deriving from.
 */
class AIInternalMoveToState : public State
{
	MEMORY_POOL_GLUE_ABC(AIInternalMoveToState)
public:
	AIInternalMoveToState( StateMachine *machine, AsciiString name ) : State( machine, name )
	{
		m_goalPosition.zero();
		m_pathGoalPosition.zero();
		m_pathTimestamp = 0;
		m_blockedRepathTimestamp = 0;
		m_adjustDestinations = true;
		m_goalLayer = LAYER_INVALID;
	}

	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;

protected:

	void setAdjustsDestination(Bool b) { m_adjustDestinations = b; }
	Bool getAdjustsDestination() const;

	void setGoalPos(const Coord3D* pos) { m_goalPosition = *pos; }
	const Coord3D* getGoalPos() const { return &m_goalPosition; }

	virtual Bool computePath();												///< compute the path

	void forceRepath()
	{
		m_pathGoalPosition.x = m_pathGoalPosition.y = m_pathGoalPosition.z = -100.0f;
		m_pathTimestamp = -MIN_REPATH_TIME;
	}

private:
	void startMoveSound();

protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

private:
	enum { MIN_REPATH_TIME = 10 };										///< minimum # of frames must elapse before re-pathing
protected:
	Coord3D							m_goalPosition;											///< the goal position to move to
	PathfindLayerEnum		m_goalLayer;										///< The layer we are moving towards.
	Coord3D				m_pathGoalPosition;									///< the position our current path leads to
private:
	AudioHandle		m_ambientPlayingHandle;							///< Audio handle for the looping sound that we may play.
	UnsignedInt		m_pathTimestamp;										///< time of last pathfind
	UnsignedInt		m_blockedRepathTimestamp;						///< time of last blocked pathfind
	Bool					m_adjustDestinations;								///< Adjust destinations to avoid stacking units on top of each other.  Normally true, but
																										//   occasionally false for things like car bombs.
protected:
	Bool					m_waitingForPath;										///< If we are waiting for a path.
	Bool					m_tryOneMoreRepath;									///< If true, after we complete movement do another compute path.
};
EMPTY_DTOR(AIInternalMoveToState)

//-----------------------------------------------------------------------------------------------------------
/**
 * Move to the GoalPosition, or GoalObject.
 */
class AIRappelState : public State
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIRappelState, "AIRappelState")
protected:
	Real m_rappelRate;
	Real m_destZ;
	Bool m_targetIsBldg;
protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

public:
	AIRappelState( StateMachine *machine );
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;
};
EMPTY_DTOR(AIRappelState)

//-----------------------------------------------------------------------------------------------------------
/**
 * Spin. Busy-like
 */
class AIBusyState : public State
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIBusyState, "AIBusyState")
public:
	AIBusyState( StateMachine *machine ) : State( machine, "AIBusyState" ) { }
	virtual StateReturnType onEnter() override { return STATE_CONTINUE; }
	virtual void onExit( StateExitType status ) override { }
	virtual StateReturnType update() override { return STATE_CONTINUE; }
	virtual Bool isBusy() const override { return true; }
protected:
	// snapshot interface STUBBED.
	virtual void crc( Xfer *xfer ) override {};
	virtual void xfer( Xfer *xfer ) override {XferVersion cv = 1;	XferVersion v = cv; xfer->xferVersion( &v, cv );}
	virtual void loadPostProcess() override {};

};
EMPTY_DTOR(AIBusyState)

//-----------------------------------------------------------------------------------------------------------
/**
 * Move to the GoalPosition, or GoalObject.
 */
class AIMoveToState : public AIInternalMoveToState
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIMoveToState, "AIMoveToState")
protected:
	Bool m_isMoveTo;
public:
	AIMoveToState( StateMachine *machine );
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;
};
EMPTY_DTOR(AIMoveToState)

//-----------------------------------------------------------------------------------------------------------
/**
 * Moves out of the way of another object.
 */
class AIMoveOutOfTheWayState : public AIInternalMoveToState
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIMoveOutOfTheWayState, "AIMoveOutOfTheWayState")
public:
	AIMoveOutOfTheWayState( StateMachine *machine ) : AIInternalMoveToState( machine, "AIMoveOutOfTheWayState" ) { }
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;
protected:
	virtual Bool computePath() override;												///< compute the path

};
EMPTY_DTOR(AIMoveOutOfTheWayState)

//-----------------------------------------------------------------------------------------------------------
/**
 * Moves toward goal pos to tighen up a formation.
 */
class AIMoveAndTightenState : public AIInternalMoveToState
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIMoveAndTightenState, "AIMoveAndTightenState")
public:
	AIMoveAndTightenState( StateMachine *machine ) : AIInternalMoveToState( machine, "AIMoveAndTightenState" )
	{
		m_checkForPath = false;
		m_okToRepathTimes = 0;
	}
	AIMoveAndTightenState( StateMachine *machine, const char *name ) : AIInternalMoveToState( machine, name )
	{
		m_checkForPath = false;
		m_okToRepathTimes = 0;
	}
	virtual StateReturnType onEnter() override;
	virtual StateReturnType update() override;
protected:
	virtual Bool computePath() override;												///< compute the path
	Int m_okToRepathTimes;
	Bool m_checkForPath;
protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;
};
EMPTY_DTOR(AIMoveAndTightenState)

//-----------------------------------------------------------------------------------------------------------
/**
 * Moves toward goal pos to get away from an enemy.
 */
class AIMoveAwayFromRepulsorsState : public AIMoveAndTightenState
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIMoveAwayFromRepulsorsState, "AIMoveAwayFromRepulsorsState")
public:
	AIMoveAwayFromRepulsorsState( StateMachine *machine ) : AIMoveAndTightenState( machine, "AIMoveAwayFromRepulsors" ) { }
	virtual StateReturnType onEnter() override;
	virtual StateReturnType update() override;
	virtual void onExit( StateExitType status ) override;
protected:
	virtual Bool computePath() override;												///< compute the path

};
EMPTY_DTOR(AIMoveAwayFromRepulsorsState)

//-----------------------------------------------------------------------------------------------------------
/**
 * Compute a valid spot to fire on GoalObject, and move there.
 */
class AIAttackApproachTargetState : public AIInternalMoveToState
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIAttackApproachTargetState, "AIAttackApproachTargetState")
public:
	AIAttackApproachTargetState( StateMachine *machine, Bool follow, Bool attackingObject, Bool forceAttacking ) :
		AIInternalMoveToState( machine, "AIAttackApproachTargetState" ),
		m_follow(follow),
		m_isAttackingObject(attackingObject),
		m_isInitialApproach(true),
		m_isForceAttacking(forceAttacking)
	{
		// we're setting m_isInitialApproach to true in the constructor because we want the first pass
		// through this state to allow a unit to attack incidental targets (if it is turreted)
	}
	virtual Bool isAttack() const override { return TRUE; }
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;
protected:
	virtual Bool computePath() override;												///< compute the path

protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

private:

	enum { MIN_RECOMPUTE_TIME = 10 };

	StateReturnType updateInternal();

	Coord3D				m_prevVictimPos;									///< Where we think our victim is
	UnsignedInt		m_approachTimestamp;							///< When we last computed an approach goal
	Bool					m_follow;													///< If true, follow object until it dies
	Bool					m_isAttackingObject;							///< If false, attacking position
	Bool					m_stopIfInRange;									///< If true, we check and stop as soon we can fire.  Used when we have to path to the object instead of to a firing position.
	Bool					m_isInitialApproach;							///< If true, we can attack other units along the way. We will check for them every N frames (N specified in AI.ini)
	Bool					m_isForceAttacking;
};
EMPTY_DTOR(AIAttackApproachTargetState)

//-----------------------------------------------------------------------------------------------------------
/**
 * Chases a moving GoalObject.  If goal is not object, or not moving, or attacker lacks turret, exits with success.
 */
class AIAttackPursueTargetState : public AIInternalMoveToState
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIAttackPursueTargetState, "AIAttackPursueTargetState")
public:
	AIAttackPursueTargetState( StateMachine *machine, Bool follow, Bool attackingObject, Bool forceAttacking ) :
		AIInternalMoveToState( machine, "AIAttackPursueTargetState" ),
		m_follow(follow),
		m_isAttackingObject(attackingObject),
		m_isInitialApproach(true),
		m_isForceAttacking(forceAttacking),
		m_approachTimestamp(0),
		m_stopIfInRange(false)
	{
		// we're setting m_isInitialApproach to true in the constructor because we want the first pass
		// through this state to allow a unit to attack incidental targets (if it is turreted)
	}
	virtual Bool isAttack() const override { return TRUE; }
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;
protected:
	virtual Bool computePath() override;												///< compute the path

protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

private:

	enum { MIN_RECOMPUTE_TIME = 10 };

	StateReturnType updateInternal();

	Coord3D				m_prevVictimPos;									///< Where we think our victim is
	UnsignedInt		m_approachTimestamp;							///< When we last computed an approach goal
	Bool					m_follow;													///< If true, follow object until it dies
	Bool					m_isAttackingObject;							///< If false, attacking position
	Bool					m_stopIfInRange;									///< If true, we check and stop as soon we can fire.  Used when we have to path to the object instead of to a firing position.
	Bool					m_isInitialApproach;							///< If true, we can attack other units along the way. We will check for them every N frames (N specified in AI.ini)
	Bool					m_isForceAttacking;
};
EMPTY_DTOR(AIAttackPursueTargetState)

//-----------------------------------------------------------------------------------------------------------
/**
 * Pick up an upgrade crate after a kill.
 */
class AIPickUpCrateState : public AIInternalMoveToState
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIPickUpCrateState, "AIPickUpCrateState")
public:
	AIPickUpCrateState( StateMachine *machine ) :
		AIInternalMoveToState( machine, "AIAttackPickUpCrateState" )
	{
		// we're setting m_isInitialApproach to true in the constructor because we want the first pass
		// through this state to allow a unit to attack incidental targets (if it is turreted)
	}
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;
protected:
	virtual Bool computePath() override;												///< compute the path

private:
	Int			m_delayCounter;
protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;
};
EMPTY_DTOR(AIPickUpCrateState)

//-------------------------------------------------------------------------------------------------
/**
 * Execute an Attack-Move
 */
 class AIAttackMoveToState : public AIMoveToState
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIAttackMoveToState, "AIAttackMoveToState")
public:
	AIAttackMoveToState( StateMachine *machine );
	//virtual ~AIAttackMoveToState();

	virtual Bool isAttack() const override { return m_attackMoveMachine ? m_attackMoveMachine->isInAttackState() : FALSE; }
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;
#ifdef STATE_MACHINE_DEBUG
	virtual AsciiString getName() const override;
#endif

protected:
	enum {ATTACK_RETRY_COUNT=5};
	enum {ATTACK_CLOSE_ENOUGH_CELLS=8};
	CommandSourceType	m_commandSrc;		// Original command source.  We switch to CMD_FROM_AI when auto-acquiring.
	StateMachine *m_attackMoveMachine;
	UnsignedInt		m_frameToSleepUntil;
	Int						m_retryCount;
protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;
};

//-----------------------------------------------------------------------------------------------------------
/**
 * Follow a waypoint path
 */
class AIFollowWaypointPathState : public AIInternalMoveToState
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIFollowWaypointPathState, "AIFollowWaypointPathState")
public:
	AIFollowWaypointPathState( StateMachine *machine, Bool asGroup ) : m_isFollowWaypointPathState(true),
		m_moveAsGroup(asGroup), AIInternalMoveToState( machine, "AIFollowWaypointPathState" )
	{
		m_angle = 0.0f;
	}
	AIFollowWaypointPathState( StateMachine *machine, Bool asGroup, Bool isFollow) :
		m_isFollowWaypointPathState(isFollow), m_moveAsGroup(asGroup), AIInternalMoveToState( machine,
																"AIFollowWaypointPathState" )
	{
		m_angle = 0.0f;
	}
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;

protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

protected:
	Coord2D m_groupOffset;
	Real		m_angle;
	Int  m_framesSleeping;
	const Waypoint *m_currentWaypoint;
	const Waypoint *m_priorWaypoint;
	Bool m_appendGoalPosition;

	const Bool m_moveAsGroup;
	// this is necessary because we derive from FollowWaypointPathState, but we do not want to incur the
	// expense of checking RTTI to determine whether we are actually a FollowWaypointPathState or not
	const Bool m_isFollowWaypointPathState;	// derived classes should set this false.

protected:
	void computeGoal(Bool useGroupOffsets);
	Real calcExtraPathDistance();
	const Waypoint *getNextWaypoint();
	Bool hasNextWaypoint();
};
EMPTY_DTOR(AIFollowWaypointPathState)

//-----------------------------------------------------------------------------------------------------------
/**
 * Follow a waypoint path exactly
 */
class AIFollowWaypointPathExactState : public AIInternalMoveToState
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIFollowWaypointPathExactState, "AIFollowWaypointPathExactState")
public:
	AIFollowWaypointPathExactState( StateMachine *machine, Bool asGroup ) : m_moveAsGroup(asGroup),
		m_lastWaypoint(nullptr),
		AIInternalMoveToState( machine, "AIFollowWaypointPathExactState" ) { }
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;

protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

protected:
	const Waypoint *m_lastWaypoint;
	const Bool m_moveAsGroup;
};
EMPTY_DTOR(AIFollowWaypointPathExactState)

//-----------------------------------------------------------------------------------------------------------
/**
 * Follow a waypoint path, attacking targets encountered along the way
 */
class AIAttackFollowWaypointPathState : public AIFollowWaypointPathState
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIAttackFollowWaypointPathState, "AIAttackFollowWaypointPathState")
public:
	AIAttackFollowWaypointPathState( StateMachine *machine, Bool asGroup );
	//virtual ~AIAttackFollowWaypointPathState();

	virtual Bool isAttack() const override { return m_attackFollowMachine ? m_attackFollowMachine->isInAttackState() : FALSE; }

	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;
#ifdef STATE_MACHINE_DEBUG
	virtual AsciiString getName() const override;
#endif

protected:
	// We can (and want to) use the attack-move-to logic for when to attack and when to do a normal attack.
	StateMachine *m_attackFollowMachine;
protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;
};

//-----------------------------------------------------------------------------------------------------------
/**
 * Wanders around following a waypoint path.
 */
class AIWanderState : public AIFollowWaypointPathState
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIWanderState, "AIWanderState")
public:
	AIWanderState( StateMachine *machine ) : AIFollowWaypointPathState( machine, false )
	{
#ifdef STATE_MACHINE_DEBUG
		setName("AIWanderState");
#endif
		m_timer = 0;
		m_waitFrames = 0;
	}
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;

protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

protected:
	Int		m_waitFrames;
	Int		m_timer;
};
EMPTY_DTOR(AIWanderState)

//-----------------------------------------------------------------------------------------------------------
/**
 * Wanders around a point.
 */
class AIWanderInPlaceState : public AIInternalMoveToState
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIWanderInPlaceState, "AIWanderInPlaceState")
public:
	AIWanderInPlaceState( StateMachine *machine ) : AIInternalMoveToState( machine, "AIWanderInPlaceState" )
	{
		m_origin.zero();
		m_waitFrames = 0;
		m_timer = 0;
	}
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;

protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

protected:
	void computeWanderGoal();
	Coord3D m_origin;									///< The point we're wandering around.
	Int		m_waitFrames;
	Int		m_timer;
};
EMPTY_DTOR(AIWanderInPlaceState)

//-----------------------------------------------------------------------------------------------------------
/**
 * Runs around screaming following a waypoint path.
 */
class AIPanicState : public AIFollowWaypointPathState
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIPanicState, "AIPanicState")
public:
	AIPanicState( StateMachine *machine ) : AIFollowWaypointPathState( machine, false)
	{
#ifdef STATE_MACHINE_DEBUG
		setName("AIPanicState");
#endif
	}
	virtual StateReturnType onEnter() override;
	virtual StateReturnType update() override;
	virtual void onExit( StateExitType status ) override;
protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

protected:
	Int		m_waitFrames;
	Int		m_timer;
};
EMPTY_DTOR(AIPanicState)


//-----------------------------------------------------------------------------------------------------------
/**
 * Follow simple list of points.
 */
class AIFollowPathState : public AIInternalMoveToState
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIFollowPathState, "AIFollowPathState")
public:
	AIFollowPathState( StateMachine *machine, AsciiString name = "AIFollowPathState" ) :
		m_adjustFinal(true),
		m_adjustFinalOverride(false),
		m_index(0),
		m_retryCount(10),
		AIInternalMoveToState( machine, name ) { }
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;

protected:

	Int getCurPathIndex() const { return m_index; }
	void setAdjustFinalDestination(Bool b) { m_adjustFinal = b; }
	void setAdjustFinalDestinationEvenIfNotDoingGroundMovement(Bool b) { m_adjustFinalOverride = b; }

protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

private:
	Int m_index;																		///< current path index
	Bool m_adjustFinal;
	Bool m_adjustFinalOverride;
	Int m_retryCount;
};
EMPTY_DTOR(AIFollowPathState)

//-----------------------------------------------------------------------------------------------------------
/**
 * Move to and unload transport.
 */
class AIMoveAndEvacuateState : public AIInternalMoveToState
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIMoveAndEvacuateState, "AIMoveAndEvacuateState")
public:
	AIMoveAndEvacuateState( StateMachine *machine ) : AIInternalMoveToState( machine, "AIMoveAndEvacuateState" )
	{
		m_origin.zero();
	}
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;
protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

private:
	Coord3D m_origin;													///< current position - set as goal on exit in case we follow with MoveToAndDelete.
};
EMPTY_DTOR(AIMoveAndEvacuateState)

//-----------------------------------------------------------------------------------------------------------
/**
 * Move to and delete self.  Used for removing reinforcement transports.
 */
class AIMoveAndDeleteState : public AIInternalMoveToState
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIMoveAndDeleteState, "AIMoveAndDeleteState")
public:
	AIMoveAndDeleteState( StateMachine *machine ) : AIInternalMoveToState( machine, "AIMoveAndDeleteState" )
	{
		m_appendGoalPosition = FALSE;
	}
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;
protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

protected:
	Bool m_appendGoalPosition;
};
EMPTY_DTOR(AIMoveAndDeleteState)

//-----------------------------------------------------------------------------------------------------------
class AIAttackAimAtTargetState : public State
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIAttackAimAtTargetState, "AIAttackAimAtTargetState")
public:
	AIAttackAimAtTargetState( StateMachine *machine, Bool attackingObject, Bool forceAttacking ) :
		State( machine, "AIAttackAimAtTargetState" ),
		m_isAttackingObject(attackingObject),
		m_canTurnInPlace(false),
		m_isForceAttacking(forceAttacking),
		m_setLocomotor(false)
	{
	}
	virtual Bool isAttack() const override { return TRUE; }
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;
protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;
private:
	const Bool m_isAttackingObject;
	Bool m_canTurnInPlace;
	Bool m_setLocomotor;
	Bool m_isForceAttacking;
};
EMPTY_DTOR(AIAttackAimAtTargetState)

//-----------------------------------------------------------------------------------------------------------
class AIWaitState : public State
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIWaitState, "AIWaitState")
public:
	AIWaitState( StateMachine *machine ) : State( machine,"AIWaitState" ) { }
	virtual StateReturnType update() override;
protected:
	// snapshot interface STUBBED.
	virtual void crc( Xfer *xfer ) override {};
	virtual void xfer( Xfer *xfer ) override {XferVersion cv = 1;	XferVersion v = cv; xfer->xferVersion( &v, cv );}
	virtual void loadPostProcess() override {};
};
EMPTY_DTOR(AIWaitState)

//-----------------------------------------------------------------------------------------------------------
class NotifyWeaponFiredInterface	// an ABC
{
public:
	virtual void notifyFired() = 0;
	virtual void notifyNewVictimChosen(Object* victim) = 0;
	virtual Bool isWeaponSlotOkToFire(WeaponSlotType wslot) const = 0;
	virtual Bool isAttackingObject() const = 0;
	virtual const Coord3D* getOriginalVictimPos() const = 0;
};

//-----------------------------------------------------------------------------------------------------------
class AIAttackFireWeaponState : public State
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIAttackFireWeaponState, "AIAttackFireWeaponState")
public:
	AIAttackFireWeaponState( StateMachine *machine, NotifyWeaponFiredInterface* att ) :
		State( machine, "AIAttackFireWeaponState" ),
		m_att(att)
	{
	}
	virtual Bool isAttack() const override { return TRUE; }
	virtual StateReturnType update() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType onEnter() override;
protected:
	// snapshot interface STUBBED.
	virtual void crc( Xfer *xfer ) override {};
	virtual void xfer( Xfer *xfer ) override {XferVersion cv = 1;	XferVersion v = cv; xfer->xferVersion( &v, cv );}
	virtual void loadPostProcess() override {};
private:
	NotifyWeaponFiredInterface *const m_att;		// this is NOT owned by us and should not be freed
};
EMPTY_DTOR(AIAttackFireWeaponState)

//-----------------------------------------------------------------------------------------------------------
class AttackExitConditionsInterface
{
public:
	virtual Bool shouldExit(const StateMachine* machine) const = 0;
};

//-----------------------------------------------------------------------------------------------------------
class AIAttackState : public State, public NotifyWeaponFiredInterface
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIAttackState, "AIAttackState")
public:

	AIAttackState( StateMachine *machine, Bool follow, Bool attackingObject, Bool forceAttacking, AttackExitConditionsInterface* attackParameters);
	//~AIAttackState();

	virtual Bool isAttack() const override { return TRUE; }
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;

	virtual void notifyFired() override { /* nothing */ }
	virtual void notifyNewVictimChosen(Object* victim) override;
	virtual const Coord3D* getOriginalVictimPos() const override { return &m_originalVictimPos; }
	virtual Bool isWeaponSlotOkToFire(WeaponSlotType wslot) const override { return true; }
	virtual Bool isAttackingObject() const override { return m_isAttackingObject; }
	virtual Bool isForceAttacking() const { return m_isForceAttacking; }
#ifdef STATE_MACHINE_DEBUG
	virtual AsciiString getName() const override;
#endif

protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

private:

	Bool chooseWeapon();

	AttackStateMachine*							m_attackMachine;						///< state sub-machine for attack behavior
	AttackExitConditionsInterface*	m_attackParameters;					///< these are not owned by this, and will not be deleted on destruction
	Team*														m_victimTeam;								///< recorded onEnter because if it changes during attack , it may no longer be a valid target.
	Coord3D													m_originalVictimPos;				///< position of first obj/pos attacked... used for ContinueAttackRange.
	const Weapon*						m_lockedWeaponOnEnter;
	const Bool							m_follow;
	const Bool							m_isAttackingObject;								// if false, attacking position
	const Bool							m_isForceAttacking;
};

//-----------------------------------------------------------------------------------------------------------
class AIAttackSquadState : public State
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIAttackSquadState, "AIAttackSquadState")
public:
	AIAttackSquadState( StateMachine *machine, AttackExitConditionsInterface *attackParameters = nullptr) :
			State( machine , "AIAttackSquadState") {	}
	//~AIAttackSquadState();

	virtual Bool isAttack() const override { return m_attackSquadMachine ? m_attackSquadMachine->isInAttackState() : FALSE; }
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;
	Object *chooseVictim();
#ifdef STATE_MACHINE_DEBUG
	virtual AsciiString getName() const override;
#endif


protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

private:
	StateMachine *m_attackSquadMachine;						///< state sub-machine for attack behavior
};

//-----------------------------------------------------------------------------------------------------------
class AIDeadState : public State
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIDeadState, "AIDeadState")
public:
	AIDeadState( StateMachine *machine ) : State( machine, "AIDeadState" ) { }
	virtual StateReturnType onEnter() override;
	virtual StateReturnType update() override;
	virtual void onExit( StateExitType status ) override;
protected:
	// snapshot interface STUBBED.
	virtual void crc( Xfer *xfer ) override {};
	virtual void xfer( Xfer *xfer ) override {XferVersion cv = 1;	XferVersion v = cv; xfer->xferVersion( &v, cv );}
	virtual void loadPostProcess() override {};
};
EMPTY_DTOR(AIDeadState)

//-----------------------------------------------------------------------------------------------------------
class AIDockState : public State
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIDockState, "AIDockState")
public:
	AIDockState( StateMachine *machine ) : State( machine, "AIDockState" ), m_dockMachine(nullptr), m_usingPrecisionMovement(FALSE) { }
	//~AIDockState();
	virtual Bool isAttack() const override { return m_dockMachine ? m_dockMachine->isInAttackState() : FALSE; }
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;
#ifdef STATE_MACHINE_DEBUG
	virtual AsciiString getName() const override;
#endif

protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

private:
	StateMachine *m_dockMachine;				///< state sub-machine for docking behavior
	Bool m_usingPrecisionMovement;			///< keep a record of precision movement so we can restore the right state on exit
};
//-----------------------------------------------------------------------------------------------------------
/**
 * Move close to GoalObject and enter it.
 */
class AIEnterState : public AIInternalMoveToState
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIEnterState, "AIEnterState")
protected:
	ObjectID m_entryToClear;
protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;
public:
	AIEnterState( StateMachine *machine ) : AIInternalMoveToState( machine, "AIEnterState" ) { }
	virtual StateReturnType onEnter() override;
	virtual StateReturnType update() override;
	virtual void onExit( StateExitType status ) override;
};
EMPTY_DTOR(AIEnterState)

//-----------------------------------------------------------------------------------------------------------
class AIExitState : public State
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIExitState, "AIExitState")
protected:
	ObjectID m_entryToClear;
protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;
public:
	AIExitState( StateMachine *machine ) : State( machine, "AIExitState" ) { }
	virtual StateReturnType onEnter() override;
	virtual StateReturnType update() override;
	virtual void onExit( StateExitType status ) override;
};
EMPTY_DTOR(AIExitState)

//-----------------------------------------------------------------------------------------------------------
class AIExitInstantlyState : public State
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIExitInstantlyState, "AIExitInstantlyState")
protected:
	ObjectID m_entryToClear;
protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;
public:
	AIExitInstantlyState( StateMachine *machine ) : State( machine, "AIExitInstantlyState" ) { }
	virtual StateReturnType onEnter() override;
	virtual StateReturnType update() override;
	virtual void onExit( StateExitType status ) override;
};
EMPTY_DTOR(AIExitInstantlyState)


//-----------------------------------------------------------------------------------------------------------
/**
 * Guard location
 */
class AIGuardState : public State
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIGuardState, "AIGuardState")
public:
	AIGuardState( StateMachine *machine ) : State( machine, "AIGuardState" ), m_guardMachine(nullptr)
	{
		m_guardMachine = nullptr;
	}
	//~AIGuardState();
	virtual Bool isAttack() const override;
	virtual Bool isGuardIdle() const override;
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;
#ifdef STATE_MACHINE_DEBUG
	virtual AsciiString getName() const override;
#endif
protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

private:
	AIGuardMachine *m_guardMachine;					///< state sub-machine for guard behavior
};

//-----------------------------------------------------------------------------------------------------------
/**
 * Guard retaliate against object (hybrid attack / guard)
 */
class AIGuardRetaliateState : public State
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIGuardRetaliateState, "AIGuardRetaliateState")
public:
	AIGuardRetaliateState( StateMachine *machine ) : State( machine, "AIGuardRetaliateState" ), m_guardRetaliateMachine(nullptr) {}
	//~AIGuardRetaliateState();
	virtual Bool isAttack() const override;
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;
#ifdef STATE_MACHINE_DEBUG
	virtual AsciiString getName() const override;
#endif
protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

private:
	AIGuardRetaliateMachine *m_guardRetaliateMachine;					///< state sub-machine for retaliate behavior
};

//-----------------------------------------------------------------------------------------------------------
/**
 * Guard from inside a tunnel network.
 */
class AITunnelNetworkGuardState : public State
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AITunnelNetworkGuardState, "AITunnelNetworkGuardState")
public:
	AITunnelNetworkGuardState( StateMachine *machine ) : State( machine, "AITunnelNetworkGuardState" ), m_guardMachine(nullptr)
	{
		m_guardMachine = nullptr;
	}
	//~AIGuardState();
	virtual Bool isAttack() const override;
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;
#ifdef STATE_MACHINE_DEBUG
	virtual AsciiString getName() const override;
#endif
protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

private:
	AITNGuardMachine *m_guardMachine;					///< state sub-machine for guard behavior
};

//-----------------------------------------------------------------------------------------------------------
/**
 * Seek and destroy behavior
 */
class AIHuntState : public State
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIHuntState, "AIHuntState")
public:
	AIHuntState( StateMachine *machine ) : State( machine, "AIHuntState" ), m_huntMachine(nullptr)
	{
		m_nextEnemyScanTime = 0;
	}
	//~AIHuntState();
	virtual Bool isAttack() const override;
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;
#ifdef STATE_MACHINE_DEBUG
	virtual AsciiString getName() const override;
#endif

protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

private:
	StateMachine* m_huntMachine;					///< state sub-machine for hunt behavior
	UnsignedInt m_nextEnemyScanTime;			///< time of last enemy scan
	enum { ENEMY_SCAN_RATE = LOGICFRAMES_PER_SECOND };
};

//-----------------------------------------------------------------------------------------------------------
/**
 * Seek and destroy in area
 */
class AIAttackAreaState : public State
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIAttackAreaState, "AIAttackAreaState")
public:
	AIAttackAreaState( StateMachine *machine ) : State( machine, "AIAttackAreaState" ), m_attackMachine(nullptr),
		m_nextEnemyScanTime(0) { }
	//~AIAttackAreaState();
	virtual Bool isAttack() const override { return m_attackMachine ? m_attackMachine->isInAttackState() : FALSE; }
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;
#ifdef STATE_MACHINE_DEBUG
	virtual AsciiString getName() const override;
#endif

protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

private:
	StateMachine *m_attackMachine;					///< state sub-machine for attack behavior
	UnsignedInt m_nextEnemyScanTime;			///< time of last enemy scan
	enum { ENEMY_SCAN_RATE = LOGICFRAMES_PER_SECOND };
};

//-----------------------------------------------------------------------------------------------------------
// Faces a position or an object (depending on goal)
//-----------------------------------------------------------------------------------------------------------
class AIFaceState : public State
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AIFaceState, "AIFaceState")
public:
	AIFaceState( StateMachine *machine, Bool obj ) : State( machine, "AIFaceState" ), m_canTurnInPlace(false), m_obj(obj) { }
	virtual StateReturnType onEnter() override;
	virtual void onExit( StateExitType status ) override;
	virtual StateReturnType update() override;
protected:
	// snapshot interface .
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

protected:
	const Bool m_obj;
	Bool m_canTurnInPlace;
};
EMPTY_DTOR(AIFaceState)
```
