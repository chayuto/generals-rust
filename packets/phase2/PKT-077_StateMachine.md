# Translation Packet `PKT-077`: `StateMachine`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-077`
- **Work Package:** `WP-2.3` (Phase 2)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 1: C-ABI Leaf
- **Allowed Rust Output File:** `crates/zh_common/src/core/state_machine.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_common/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (15):**
  - `Common/Errors.h`
  - `Common/GameMemory.h`
  - `Common/GameState.h`
  - `Common/GameType.h`
  - `Common/GlobalData.h`
  - `Common/ModelState.h`
  - `Common/PerfMetrics.h`
  - `Common/PerfTimer.h`
  - `Common/Snapshot.h`
  - `Common/StateMachine.h`
  - `Common/ThingTemplate.h`
  - `Common/Xfer.h`
  - `GameLogic/GameLogic.h`
  - `GameLogic/Object.h`
  - `PreRTS.h`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (18 calls — preserve order identically):**
  1. `xferVersion( &v, cv )`
  2. `xferVersion( &v, cv )`
  3. `xferVersion( &v, cv )`
  4. `xferVersion( &v, cv )`
  5. `xferVersion( &version, currentVersion )`
  6. `xferUnsignedInt(&m_sleepTill)`
  7. `xferUnsignedInt(&m_defaultStateID)`
  8. `xferUnsignedInt(&curStateID)`
  9. `xferBool(&snapshotAllStates)`
  10. `xferInt(&saveCount)`
  11. `xferUnsignedInt(&id)`
  12. `xferUnsignedInt(&id)`
  13. `xferSnapshot(state)`
  14. `xferSnapshot(m_currentState)`
  15. `xferObjectID(&m_goalObjectID)`
  16. `xferCoord3D(&m_goalPosition)`
  17. `xferBool(&m_locked)`
  18. `xferBool(&m_defaultStateInited)`

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

### `GeneralsMD/Code/GameEngine/Include/Common/StateMachine.h`
```cpp
// StateMachine.h
// Finite state machine encapsulation
// Author: Michael S. Booth, January 2002

#pragma once

#include "Common/GameMemory.h"
#include "Common/GameType.h"
#include "Common/ModelState.h"
#include "Common/Snapshot.h"
#include "Common/Xfer.h"

//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------

class State;
class StateMachine;
class Object;

//#undef STATE_MACHINE_DEBUG
#if defined(RTS_DEBUG)
	#define STATE_MACHINE_DEBUG
#endif

//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------

enum { MACHINE_DONE_STATE_ID = 999998, INVALID_STATE_ID = 999999 };
typedef UnsignedInt StateID;									///< used to denote individual states
typedef Bool (*StateTransFuncPtr)( State *state, void* userData );

/**
 * State return codes
 */
enum StateReturnType CPP_11(: Int)
{
	// note that all positive values are reserved for STATE_SLEEP!

	STATE_CONTINUE	= 0,						///< stay in this state (only for update method)
	STATE_SUCCESS		= -1,						///< state finished successfully, go to next state
	STATE_FAILURE		= -2,						///< state finished abnormally, go to next state
};

#define STATE_SLEEP(numFrames)				((StateReturnType)(numFrames))
#define IS_STATE_SLEEP(ret)						((Int)(ret) > 0)
#define GET_STATE_SLEEP_FRAMES(ret)		((UnsignedInt)(ret))

// (we use 0x3fffffff so that we can add offsets and not overflow...
//		at 30fps that's around ~414 days!)
#define STATE_SLEEP_FOREVER		STATE_SLEEP(0x3fffffff)

// this is mainly useful for states that enclose other state machines...
// where, even if the enclosed machine is sleeping, the encloser still needs
// to run every frame.
inline StateReturnType CONVERT_SLEEP_TO_CONTINUE(StateReturnType s)
{
	return IS_STATE_SLEEP(s) ? STATE_CONTINUE : s;
}

// this is mainly useful for states that enclose other state machines...
// where the encloser and enclosee both might sleep. we need to choose the min
// sleep time that satisfies both.
inline StateReturnType MIN_SLEEP(UnsignedInt encloserSleep, StateReturnType encloseeResult)
{
	if (IS_STATE_SLEEP(encloseeResult))
	{
		UnsignedInt encloseeSleep = GET_STATE_SLEEP_FRAMES(encloseeResult);
		return STATE_SLEEP(min(encloserSleep, encloseeSleep));
	}
	else
	{
		// if enclosee needs to stay awake, we better do so, regardless
		return encloseeResult;
	}
}


/**
 * Special argument for onCondition. It means when the given condition
 * becomes true, the state machine will exit and return the given status.
 */
enum
{
	EXIT_MACHINE_WITH_SUCCESS = 9998,
	EXIT_MACHINE_WITH_FAILURE = 9999
};

/**
 * Parameters for onExit().
 */
enum StateExitType CPP_11(: Int)
{
	EXIT_NORMAL,							///< state exited due to normal state transitioning
	EXIT_RESET								///< state exited due to state machine reset
};

struct StateConditionInfo
{
	StateTransFuncPtr	test;
	StateID						toStateID;
	void*							userData;

	StateConditionInfo(StateTransFuncPtr t, StateID id, void* ud) : test(t), toStateID(id), userData(ud) { }
};

//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
/**
 * An abstraction of a machine's "state".
 */
class State : public MemoryPoolObject, public Snapshot
{
	MEMORY_POOL_GLUE_ABC( State )						///< this abstract class needs memory pool hooks
public:
	State( StateMachine *machine, AsciiString name);
	// already defined by MPO.
	//virtual ~State() { }

	virtual StateReturnType onEnter() { return STATE_CONTINUE; }	///< executed once when entering state
	virtual void onExit( StateExitType status ) { }											///< executed once when leaving state
	virtual StateReturnType update() = 0;	///< implements this state's behavior, decides when to change state

	virtual Bool isIdle() const { return false; }
	virtual Bool isAttack() const { return false; }
	virtual Bool isGuardIdle() const { return FALSE; }
	//Definition of busy -- when explicitly in the busy state. Moving or attacking is not considered busy!
	virtual Bool isBusy() const { return false; }

	StateMachine* getMachine() { return m_machine; }		///< return the machine this state is part of
	StateID getID() const { return m_ID; }			///< get this state's id

	Object* getMachineOwner();
	const Object* getMachineOwner() const;
	Object* getMachineGoalObject();
	const Object* getMachineGoalObject() const;
	const Coord3D* getMachineGoalPosition() const;

#ifdef STATE_MACHINE_DEBUG
	virtual AsciiString getName() const {return m_name;}
	std::vector<StateID> *getTransitions();
#endif

	// for internal use by the StateMachine class ---------------------------------------------------------
	void friend_setID( StateID id ) { m_ID = id; }			///< define this state's id (for use only by StateMachine class)
	void friend_onSuccess( StateID toStateID ) { m_successStateID = toStateID; }	///< define which state to move to after successful completion
	void friend_onFailure( StateID toStateID ) { m_failureStateID = toStateID; }	///< define which state to move to after failure
	void friend_onCondition( StateTransFuncPtr test, StateID toStateID, void* userData, const char* description = nullptr );	///< define when to change state
	StateReturnType friend_checkForTransitions( StateReturnType status );	///< given a return code, handle state transitions
	StateReturnType friend_checkForSleepTransitions( StateReturnType status );	///< given a return code, handle state transitions

protected:

#ifdef STATE_MACHINE_DEBUG
	inline void setName(AsciiString n) { m_name = n; }
#endif

private:

	struct TransitionInfo
	{
		StateTransFuncPtr		test;											///< the condition evaluation function
		StateID							toStateID;								///< the state to transition to
		void*								userData;									///< data passed to transFuncPtr.
#ifdef STATE_MACHINE_DEBUG
		const char*					description;							///< description (for debugging purposes)
#endif

		TransitionInfo(StateTransFuncPtr t, StateID id, void* ud, const char* desc) :
			test(t),
			toStateID(id),
			userData(ud)
#ifdef STATE_MACHINE_DEBUG
			, description(desc)
#endif
		{ }
	};

	StateID m_ID;																///< this state's ID
	StateID m_successStateID;										///< state to move to upon success
	StateID m_failureStateID;										///< state to move to upon failure
	std::vector<TransitionInfo> m_transitions;	///< possible transitions from this state

	StateMachine *m_machine;										///< the state machine this state is part of
protected:
#ifdef STATE_MACHINE_DEBUG
	AsciiString m_name;													///< Human readable name of this state - for debugging.  jba.
#endif
};
inline State::~State() { }

//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------

/**
 * A finite state machine.
 */
class StateMachine : public MemoryPoolObject, public Snapshot
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( StateMachine, "StateMachinePool" );

public:
	/**
	 * All of the states used by this machine should be
	 * instantiated and defined via defineState() in the
	 * machine's constructor.
	 */
	StateMachine( Object *owner, AsciiString name );
	// virtual destructor defined by MemoryPool

	virtual StateReturnType updateStateMachine();				///< run one step of the machine

	virtual void clear();																///< clear the machine's internals to a known, initialized state

	virtual StateReturnType resetToDefaultState();			///< clear the machine's internals and set to the default state

	/** must be called to jump the StateMachine into the default state... this may fail
		(return a failure code), so it can't be done via the ctor. you MUST call this before
		using the state machine. */
	virtual StateReturnType initDefaultState();

	virtual StateReturnType setState( StateID newStateID );			///< change the current state of the machine (which may cause further state changes, due to onEnter)

	void Add_Ref() const { m_refCount.Add_Ref(); }
	void Release_Ref() const { m_refCount.Release_Ref(MemoryPoolObject::deleteInstanceInternal, this); }
	UnsignedByte Num_Refs() const { return m_refCount.Num_Refs(); }

	StateID getCurrentStateID() const { return m_currentState ? m_currentState->getID() : INVALID_STATE_ID; }	///< return the id of the current state of the machine
	Bool isInIdleState() const { return m_currentState ? m_currentState->isIdle() : true; }	// stateless things are considered 'idle'
	Bool isInAttackState() const { return m_currentState ? m_currentState->isAttack() : true; }	// stateless things are considered 'idle'
	Bool isInForceAttackState() const { return m_currentState ? m_currentState->isIdle() : true; }	// stateless things are considered 'idle'
	Bool isInGuardIdleState() const { return m_currentState ? m_currentState->isGuardIdle() : FALSE; } // stateless things aren't guard idle.

	//Definition of busy -- when explicitly in the busy state. Moving or attacking is not considered busy!
	Bool isInBusyState() const { return m_currentState ? m_currentState->isBusy() : false; }	// stateless things are not considered 'busy'

	// no, this is now deprecated. you should strive to avoid having to get the current state.
	// try to make do with getCurrentStateID() or isInIdleState() instead. (srj)
	// State *getCurrentState() { return m_currentState; }	///< return the current state of the machine

	/**
	 * Lock/unlock this state machine.
	 * If a machine is locked, it cannot be reset, or given external setStates(), etc.
	 */
	void lock(const char* msg)
	{
		m_locked = true;
#ifdef STATE_MACHINE_DEBUG
		m_lockedby = msg;
#endif
	}

	void unlock()
	{
		m_locked = false;
#ifdef STATE_MACHINE_DEBUG
		m_lockedby = nullptr;
#endif
	}

	Bool isLocked() const { return m_locked; }

	/**
	 * Get the object that "owns" this machine.
	 * There need not be an object with a machine, but it is so common
	 * that it is useful to have this method in the generic state machine.
	 */
	Object *getOwner() { return m_owner; }
	const Object *getOwner() const { return m_owner; }

	// common parameters for state machines
	void setGoalObject( const Object *obj );
	Object *getGoalObject();
	const Object *getGoalObject() const;
	void setGoalPosition( const Coord3D *pos );
	const Coord3D *getGoalPosition() const { return &m_goalPosition; }
	Bool isGoalObjectDestroyed() const;  ///< Returns true if we had a goal object, but it has been destroyed.

	virtual void halt(); ///< Stops the state machine & disables it in preparation for deleting it.

	//
	// The following methods are for internal use by the State class
	//
	StateReturnType internalSetState( StateID newStateID );	///< for internal use only - change the current state of the machine

#if defined(RTS_DEBUG)
	UnsignedInt peekSleepTill() const { return m_sleepTill; }
#endif

#ifdef STATE_MACHINE_DEBUG
	Bool getWantsDebugOutput() const;
	void setDebugOutput( Bool output ) { m_debugOutput = output; }
	void setName( AsciiString name) {m_name = name;}
	inline AsciiString getName() const {return m_name;}
	virtual AsciiString getCurrentStateName() const { return m_currentState ? m_currentState->getName() : AsciiString::TheEmptyString;}
#else
	Bool getWantsDebugOutput() const { return false; }
	AsciiString getCurrentStateName() const { return AsciiString::TheEmptyString;}
#endif

protected:
	// snapshot interface
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

protected:

	/**
	 * Given a unique (to this machine) integer ID representing a state, and an instance
	 * of that state, the machine records this as a possible state, and
	 * internally maps the given integer ID to the state instance.
	 * These state id's are used to change the machine's state via setState().
	 */
	void defineState( StateID id, State *state,
										StateID successID,
										StateID failureID,
										const StateConditionInfo* conditions = nullptr);

	State* internalGetState( StateID id );

private:

	void internalClear();
	void internalSetGoalObject( const Object *obj );
	void internalSetGoalPosition( const Coord3D *pos);


	std::map<StateID, State *>	m_stateMap;			///< the mapping of ids to states
	Object*											m_owner;				///< object that "owns" this machine

	UnsignedInt		m_sleepTill;									///< if nonzero, we are sleeping 'till this frame

	StateID				m_defaultStateID;									///< the default state of the machine
	State*				m_currentState;

	ObjectID			m_goalObjectID;										///< the object of interest for this state
	Coord3D				m_goalPosition;										///< the position of interest for this state

	Bool					m_locked;													///< whether this machine is locked or not
	Bool					m_defaultStateInited;							///< if initDefaultState has been called

	RefCountValue<UnsignedByte> m_refCount;

#ifdef STATE_MACHINE_DEBUG
	Bool					m_debugOutput;
	AsciiString		m_name;													///< Human readable name of this state - for debugging.  jba.
	const char*		m_lockedby;
#endif
};

//-----------------------------------------------------------------------------

inline Object* State::getMachineOwner() { return m_machine->getOwner(); }
inline const Object* State::getMachineOwner() const { return m_machine->getOwner(); }

inline Object* State::getMachineGoalObject() { return m_machine->getGoalObject(); }		///< return the machine this state is part of
inline const Object* State::getMachineGoalObject() const { return m_machine->getGoalObject(); }		///< return the machine this state is part of

inline const Coord3D* State::getMachineGoalPosition() const { return m_machine->getGoalPosition(); }		///< return the machine this state is part of

//-----------------------------------------------------------------------------------------------------------
/**
	A utility state that immediately succeeds.
*/
class SuccessState : public State
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(SuccessState, "SuccessState")
public:
	SuccessState( StateMachine *machine ) : State( machine, "SuccessState") { }
	virtual StateReturnType onEnter() override { return STATE_SUCCESS; }
	virtual StateReturnType update() override { return STATE_SUCCESS; }
protected:
	// snapshot interface STUBBED.
	virtual void crc( Xfer *xfer ) override {};
	virtual void xfer( Xfer *xfer ) override {XferVersion cv = 1;	XferVersion v = cv; xfer->xferVersion( &v, cv );}
	virtual void loadPostProcess() override {};

};
EMPTY_DTOR(SuccessState)

//-----------------------------------------------------------------------------------------------------------
/**
	A utility state that immediately fails.
*/
class FailureState : public State
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(FailureState, "FailureState")
public:
	FailureState( StateMachine *machine ) : State( machine, "FailureState") { }
	virtual StateReturnType onEnter() override { return STATE_FAILURE; }
	virtual StateReturnType update() override { return STATE_FAILURE; }
protected:
	// snapshot interface STUBBED.
	virtual void crc( Xfer *xfer ) override {};
	virtual void xfer( Xfer *xfer ) override {XferVersion cv = 1;	XferVersion v = cv; xfer->xferVersion( &v, cv );}
	virtual void loadPostProcess() override {};
};
EMPTY_DTOR(FailureState)


//-----------------------------------------------------------------------------------------------------------
/**
	A utility state that never exits (except due to conditions).
*/
class ContinueState :  public State
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(ContinueState, "ContinueState")
public:
	ContinueState( StateMachine *machine ) : State( machine, "ContinueState" ) { }
	virtual StateReturnType onEnter() override { return STATE_CONTINUE; }
	virtual StateReturnType update() override { return STATE_CONTINUE; }
protected:
	// snapshot interface STUBBED.
	virtual void crc( Xfer *xfer ) override {};
	virtual void xfer( Xfer *xfer ) override {XferVersion cv = 1;	XferVersion v = cv; xfer->xferVersion( &v, cv );}
	virtual void loadPostProcess() override {};
};
EMPTY_DTOR(ContinueState)


//-----------------------------------------------------------------------------------------------------------
/**
	A utility state that sleeps forever.
*/
class SleepState :  public State
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(SleepState, "SleepState")
public:
	SleepState( StateMachine *machine ) : State( machine, "SleepState" ) { }
	virtual StateReturnType onEnter() override { return STATE_SLEEP_FOREVER; }
	virtual StateReturnType update() override { return STATE_SLEEP_FOREVER; }
protected:
	// snapshot interface STUBBED.
	virtual void crc( Xfer *xfer ) override {};
	virtual void xfer( Xfer *xfer ) override {XferVersion cv = 1;	XferVersion v = cv; xfer->xferVersion( &v, cv );}
	virtual void loadPostProcess() override {};
};
EMPTY_DTOR(SleepState)


//-----------------------------------------------------------------------------------------------------------
// TheSuperHackers @info Misappropriates deleteInstance to call Release_Ref to keep the StateMachine fix small.
// @todo Replace calls to deleteInstance with RefCountPtr<StateMachine> when so appropriate.
inline void deleteInstance(StateMachine* machine)
{
	if (machine != nullptr)
		machine->Release_Ref();
}
```

### `GeneralsMD/Code/GameEngine/Source/Common/StateMachine.cpp`
```cpp
// StateMachine.cpp
// Implementation of basic state machine
// Author: Michael S. Booth, January 2002

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/Errors.h"
#include "Common/StateMachine.h"
#include "Common/ThingTemplate.h"
#include "Common/GameState.h"
#include "Common/GlobalData.h"
#include "Common/Xfer.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"


//------------------------------------------------------------------------------ Performance Timers
//#include "Common/PerfMetrics.h"
//#include "Common/PerfTimer.h"

//static PerfTimer s_stateMachineTimer("StateMachine::update", false, PERFMETRICS_LOGIC_STARTFRAME, PERFMETRICS_LOGIC_STOPFRAME);
//-------------------------------------------------------------------------------------------------
//-----------------------------------------------------------------------------
/**
 * Constructor
 */
State::State( StateMachine *machine, AsciiString name )
#ifdef STATE_MACHINE_DEBUG
: m_name(name)
#endif
{
	m_ID = INVALID_STATE_ID;
	m_successStateID = INVALID_STATE_ID;
	m_failureStateID = INVALID_STATE_ID;
	m_machine = machine;
}

//-----------------------------------------------------------------------------
/**
 * Add another state transition condition for this state
 */
void State::friend_onCondition( StateTransFuncPtr test, StateID toStateID, void* userData, const char* description )
{
	m_transitions.push_back(TransitionInfo(test, toStateID, userData, description));
}


//-----------------------------------------------------------------------------
class StIncrementer
{
private:
	Int& num;
public:
	StIncrementer(Int& n) : num(n)
	{
		++num;
	}
	~StIncrementer()
	{
		--num;
	}
};
#ifdef STATE_MACHINE_DEBUG
//-----------------------------------------------------------------------------
std::vector<StateID> * State::getTransitions()
{
	std::vector<StateID> *ids = new std::vector<StateID>;
	ids->push_back(m_successStateID);
	ids->push_back(m_failureStateID);
	// check transition condition list
	if (!m_transitions.empty())
	{
		for(std::vector<TransitionInfo>::const_iterator it = m_transitions.begin(); it != m_transitions.end(); ++it)
		{
			ids->push_back(it->toStateID);
		}
	}
	return ids;
}
#endif

//-----------------------------------------------------------------------------
/**
 * Given a return code, handle state transitions
 */
StateReturnType State::friend_checkForTransitions( StateReturnType status )
{
	static Int checkfortransitionsnum = 0;

	StIncrementer inc(checkfortransitionsnum);
	if (checkfortransitionsnum >= 20)
	{
		DEBUG_CRASH(("checkfortransitionsnum is > 20"));
		return STATE_FAILURE;
	}

	DEBUG_ASSERTCRASH(!IS_STATE_SLEEP(status), ("Please handle sleep states prior to this"));

	// handle transitions
	switch( status )
	{
		case STATE_SUCCESS:
			// check if machine should exit
			if (m_successStateID == EXIT_MACHINE_WITH_SUCCESS)
			{
				getMachine()->internalSetState( MACHINE_DONE_STATE_ID );
				return STATE_SUCCESS;
			}
			else if (m_successStateID == EXIT_MACHINE_WITH_FAILURE)
			{
				getMachine()->internalSetState( MACHINE_DONE_STATE_ID );
				return STATE_FAILURE;
			}

			// move to new state
			return getMachine()->internalSetState( m_successStateID );

		case STATE_FAILURE:
			// check if machine should exit
			if (m_failureStateID == EXIT_MACHINE_WITH_SUCCESS)
			{
				getMachine()->internalSetState( MACHINE_DONE_STATE_ID );
				return STATE_SUCCESS;
			}
			else if (m_failureStateID == EXIT_MACHINE_WITH_FAILURE)
			{
				getMachine()->internalSetState( MACHINE_DONE_STATE_ID );
				return STATE_FAILURE;
			}

			// move to new state
			return getMachine()->internalSetState( m_failureStateID );

		case STATE_CONTINUE:

			// check transition condition list
			if (!m_transitions.empty())
			{
				for(std::vector<TransitionInfo>::const_iterator it = m_transitions.begin(); it != m_transitions.end(); ++it)
				{
					if (it->test( this, it->userData ))
					{
						// test returned true, change to associated state

	#ifdef STATE_MACHINE_DEBUG
						if (getMachine()->getWantsDebugOutput())
						{
							DEBUG_LOG(("%d '%s' -- '%s' condition '%s' returned true!", TheGameLogic->getFrame(), getMachineOwner()->getTemplate()->getName().str(),
											getMachine()->getName().str(), it->description ? it->description : "[no description]"));
						}
	#endif

						// check if machine should exit
						if (it->toStateID == EXIT_MACHINE_WITH_SUCCESS)
						{
							return STATE_SUCCESS;
						}
						else if (it->toStateID == EXIT_MACHINE_WITH_FAILURE)
						{
							return STATE_FAILURE;//Lorenzen wants to know why...
						}

						// move to new state
						return getMachine()->internalSetState( it->toStateID );
					}
				}
			}
			break;
	}

	// the machine keeps running
	return STATE_CONTINUE;
}

//-----------------------------------------------------------------------------
/**
 * Given a return code, handle state transitions
 */
StateReturnType State::friend_checkForSleepTransitions( StateReturnType status )
{
	static Int checkfortransitionsnum = 0;

	StIncrementer inc(checkfortransitionsnum);
	if (checkfortransitionsnum >= 20)
	{
		DEBUG_CRASH(("checkforsleeptransitionsnum is > 20"));
		return STATE_FAILURE;
	}

	DEBUG_ASSERTCRASH(IS_STATE_SLEEP(status), ("Please only pass sleep states here"));

	// check transition condition list
	if (m_transitions.empty())
		return status;

	for(std::vector<TransitionInfo>::const_iterator it = m_transitions.begin(); it != m_transitions.end(); ++it)
	{
		if (!it->test( this, it->userData ))
			continue;

		// test returned true, change to associated state

#ifdef STATE_MACHINE_DEBUG
		if (getMachine()->getWantsDebugOutput())
		{
			DEBUG_LOG(("%d '%s' -- '%s' condition '%s' returned true!", TheGameLogic->getFrame(), getMachineOwner()->getTemplate()->getName().str(),
							getMachine()->getName().str(), it->description ? it->description : "[no description]"));
		}
#endif

		// check if machine should exit
		if (it->toStateID == EXIT_MACHINE_WITH_SUCCESS)
		{
			return STATE_SUCCESS;
		}
		else if (it->toStateID == EXIT_MACHINE_WITH_FAILURE)
		{
			return STATE_FAILURE;
		}
		else
		{
			// move to new state
			return getMachine()->internalSetState( it->toStateID );
		}
	}

	return status;
}
//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------

//-----------------------------------------------------------------------------
/**
 * Constructor
 */
StateMachine::StateMachine( Object *owner, AsciiString name )
{
	m_owner = owner;
	m_sleepTill = 0;
	m_defaultStateID = INVALID_STATE_ID;
	m_defaultStateInited = false;
	m_currentState = nullptr;
	m_locked = false;
#ifdef STATE_MACHINE_DEBUG
	m_name = name;
	m_debugOutput = false;
	m_lockedby = nullptr;
#endif
	internalClear();
}

//-----------------------------------------------------------------------------
/**
 * Destructor.  Destroy any states attached to this machine.
 */
StateMachine::~StateMachine()
{

	// do not allow current state to exit
	if (m_currentState)
		m_currentState->onExit( EXIT_RESET );

	std::map<StateID, State *>::iterator i;

	// delete all states in the mapping
	for( i = m_stateMap.begin(); i != m_stateMap.end(); ++i )
	{
		deleteInstance((*i).second);
	}
}

//-----------------------------------------------------------------------------
#ifdef STATE_MACHINE_DEBUG
Bool StateMachine::getWantsDebugOutput() const
{
	if (m_debugOutput)
	{
		return true;
	}

	if (TheGlobalData->m_stateMachineDebug)
	{
		return true;
	}

#ifdef DEBUG_OBJECT_ID_EXISTS
	if (TheObjectIDToDebug != 0 && getOwner() != nullptr && getOwner()->getID() == TheObjectIDToDebug)
	{
		return true;
	}
#endif

	return false;
}
#endif

//-----------------------------------------------------------------------------
/**
 * Clear the internal variables of state machine to known values.
 */
void StateMachine::internalClear()
{
	m_goalObjectID = INVALID_ID;
	m_goalPosition.x = 0.0f;
	m_goalPosition.y = 0.0f;
	m_goalPosition.z = 0.0f;
#ifdef STATE_MACHINE_DEBUG
	if (getWantsDebugOutput())
	{
		DEBUG_LOG(("%d '%s'%x -- '%s' %x internalClear()", TheGameLogic->getFrame(), m_owner->getTemplate()->getName().str(), m_owner, m_name.str(), this));
	}
#endif
}

//-----------------------------------------------------------------------------
/**
 * Clear the machine
 */
void StateMachine::clear()
{
	// if the machine is locked, it cannot be cleared
	if (m_locked)
	{
#ifdef STATE_MACHINE_DEBUG
		if (m_currentState) DEBUG_LOG((" cur state '%s'", m_currentState->getName().str()));
		DEBUG_LOG(("machine is locked (by %s), cannot be cleared (Please don't ignore; this generally indicates a potential logic flaw)",m_lockedby));
#endif
		return;
	}

	// invoke the old state's onExit()
	if (m_currentState)
		m_currentState->onExit( EXIT_RESET );

	m_currentState = nullptr;

	internalClear();
}

//-----------------------------------------------------------------------------
/**
 * Reset the machine to its default state
 */
StateReturnType StateMachine::resetToDefaultState()
{
	// if the machine is locked, it cannot be reset
	if (m_locked)
	{
#ifdef STATE_MACHINE_DEBUG
		if (m_currentState) DEBUG_LOG((" cur state '%s'", m_currentState->getName().str()));
		DEBUG_LOG(("machine is locked (by %s), cannot be cleared (Please don't ignore; this generally indicates a potential logic flaw)",m_lockedby));
#endif
		return STATE_FAILURE;
	}

	if (!m_defaultStateInited)
	{
		DEBUG_CRASH(("you may not call resetToDefaultState before initDefaultState"));
		return STATE_FAILURE;
	}

	// allow current state to exit with EXIT_RESET if present
	if (m_currentState)
		m_currentState->onExit( EXIT_RESET );
	m_currentState = nullptr;

	//
	// the current state has done an onExit, clear the internal guts before we set
	// the new state, to clear it after the new state is set might be overwriting
	// things the new state transition causes to happen
	//
	internalClear();

	// change to the default state
	StateReturnType status = internalSetState( m_defaultStateID );

	DEBUG_ASSERTCRASH( status != STATE_FAILURE, ( "StateMachine::resetToDefaultState() Error setting default state" ) );

	return status;

}

//-----------------------------------------------------------------------------
/**
 * Run one step of the machine
 */
StateReturnType StateMachine::updateStateMachine()
{
	UnsignedInt now = TheGameLogic->getFrame();
	if (m_sleepTill != 0 && now < m_sleepTill)
	{
		if( m_currentState == nullptr )
		{
			return STATE_FAILURE;
		}
		return m_currentState->friend_checkForSleepTransitions( STATE_SLEEP(m_sleepTill - now) );
	}

	// not sleeping anymore
	m_sleepTill = 0;

	if (m_currentState)
	{
		// TheSuperHackers @bugfix xezon 20/05/2025 Defer the deletion of this state machine.
		// Calling m_currentState->update() can release this state machine in certain circumstances,
		// for example if something kills the entity of this state machine as a result of this state update.
		// See https://github.com/TheSuperHackers/GeneralsGameCode/issues/212
		RefCountPtr<StateMachine> refThis(this);

		// update() can change m_currentState, so save it for a moment...
		State* stateBeforeUpdate = m_currentState;

		// execute this state
		StateReturnType status = m_currentState->update();

		// it is possible that the state's update() method clears the state machine.
		if (m_currentState == nullptr)
		{
			return STATE_FAILURE;
		}

		// here's the scenario:
		// -- State A calls foo() and then says "sleep for 2000 frames".
		// -- however, foo() called setState() to State B. thus our current state is not the same.
		// -- thus, if the state changed, we must ignore any sleep result and pretend we got STATE_CONTINUE,
		// so that the new state will be called immediately.
		if (stateBeforeUpdate != m_currentState)
		{
			status = STATE_CONTINUE;
		}

		if (IS_STATE_SLEEP(status))
		{
			// hey, we're sleepy!
			m_sleepTill = now + GET_STATE_SLEEP_FRAMES(status);
			return m_currentState->friend_checkForSleepTransitions( STATE_SLEEP(m_sleepTill - now) );
		}
		else
		{
			// check for state transitions, possibly exiting this machine
			return m_currentState->friend_checkForTransitions( status );
		}
	}
	else
	{
		DEBUG_CRASH(("State machine has no current state -- did you remember to call initDefaultState?"));
		return STATE_FAILURE;
	}
}

//-----------------------------------------------------------------------------
/**
 * Given a unique (for this machine) id number, and an instance of the
 * State class, the machine records this as a possible state, and
 * retains the id mapping.
 * These state id's are used to change the machine's state via setState().
 */
void StateMachine::defineState( StateID id, State *state, StateID successID, StateID failureID, const StateConditionInfo* conditions )
{
#ifdef STATE_MACHINE_DEBUG
	DEBUG_ASSERTCRASH(m_stateMap.find( id ) == m_stateMap.end(), ("duplicate state ID in statemachine %s",m_name.str()));
#endif

	// map the ID to the state
	m_stateMap.insert( std::map<StateID, State *>::value_type( id, state ) );

	// store the ID in the state itself, as well
	state->friend_setID( id );

	state->friend_onSuccess(successID);
	state->friend_onFailure(failureID);

	while (conditions && conditions->test != nullptr)
	{
		state->friend_onCondition(conditions->test, conditions->toStateID, conditions->userData);
		++conditions;
	}

	if (m_defaultStateID == INVALID_STATE_ID)
		m_defaultStateID = id;
}

//-----------------------------------------------------------------------------
/**
 * Given a state ID, return the state instance
 */
State *StateMachine::internalGetState( StateID id )
{
	// locate the actual state associated with the given ID
	std::map<StateID, State *>::iterator i;
	i = m_stateMap.find( id );

	if (i == m_stateMap.end())
	{
		DEBUG_CRASH( ("StateMachine::internalGetState(): Invalid state for object %s using state %d", m_owner->getTemplate()->getName().str(), id) );
		DEBUG_LOG(("Transitioning to state %d", (Int)id));
		DEBUG_LOG(("Attempting to recover - locating default state..."));
		i = m_stateMap.find(m_defaultStateID);
		if (i == m_stateMap.end()) {
			DEBUG_LOG(("Failed to located default state.  Aborting..."));
			throw ERROR_BAD_ARG;
		} else {
			DEBUG_LOG(("Located default state to recover."));
		}
	}

	return (*i).second;
}

//-----------------------------------------------------------------------------
/**
 * Change the current state of the machine.
 * This causes the old state's onExit() method to be invoked,
 * and the new state's onEnter() method to be invoked.
 */
StateReturnType StateMachine::setState( StateID newStateID )
{
	// if the machine is locked, it cannot change state via external events
	if (m_locked)
	{
#ifdef STATE_MACHINE_DEBUG
		if (m_currentState) DEBUG_LOG((" cur state '%s'", m_currentState->getName().str()));
		DEBUG_LOG(("machine is locked (by %s), cannot be cleared (Please don't ignore; this generally indicates a potential logic flaw)",m_lockedby));
#endif
		return STATE_CONTINUE;
	}

	return internalSetState( newStateID );
}

//-----------------------------------------------------------------------------
/**
 * Change the current state of the machine.
 * This causes the old state's onExit() method to be invoked,
 * and the new state's onEnter() method to be invoked.
 */
StateReturnType StateMachine::internalSetState( StateID newStateID )
{
	State *newState = nullptr;

	// anytime the state changes, stop sleeping
	m_sleepTill = 0;

	// if we're not setting the "done" state ID we will continue with the actual transition
	if( newStateID != MACHINE_DONE_STATE_ID )
	{

		// if incoming state is invalid, go to the machine's default state
		if (newStateID == INVALID_STATE_ID)
		{
			newStateID = m_defaultStateID;
			if (newStateID == INVALID_STATE_ID)
			{
				DEBUG_CRASH(("you may NEVER set the current state to an invalid state id."));
				return STATE_FAILURE;
			}
		}

		// extract the state associated with the given ID
		newState = internalGetState( newStateID );
#ifdef STATE_MACHINE_DEBUG
		if (getWantsDebugOutput())
		{
			StateID curState = INVALID_STATE_ID;
			if (m_currentState) {
				curState = m_currentState->getID();
			}
			DEBUG_LOG_RAW(("%d '%s'%x -- '%s' %x exit ", TheGameLogic->getFrame(), m_owner->getTemplate()->getName().str(), m_owner, m_name.str(), this));
			if (m_currentState) {
				DEBUG_LOG_RAW((" '%s' ", m_currentState->getName().str()));
			} else {
				DEBUG_LOG_RAW((" INVALID_STATE_ID "));
			}
			if (newState) {
				DEBUG_LOG(("enter '%s'", newState->getName().str()));
			} else {
				DEBUG_LOG(("to INVALID_STATE"));
			}
		}
#endif
	}

	// invoke the old state's onExit()
	if (m_currentState)
		m_currentState->onExit( EXIT_NORMAL );

	// set the new state
	m_currentState = newState;

	// invoke the new state's onEnter()
	/// @todo It might be useful to pass the old state in... (MSB)
	if( m_currentState )
	{
		// onEnter() could conceivably change m_currentState, so save it for a moment...
		State* stateBeforeEnter = m_currentState;

		StateReturnType status = m_currentState->onEnter();

		// it is possible that the state's onEnter() method may cause the state to be destroyed
		if (m_currentState == nullptr)
		{
			return STATE_FAILURE;
		}

		// here's the scenario:
		// -- State A calls foo() and then says "sleep for 2000 frames".
		// -- however, foo() called setState() to State B. thus our current state is not the same.
		// -- thus, if the state changed, we must ignore any sleep result and pretend we got STATE_CONTINUE,
		// so that the new state will be called immediately.
		if (stateBeforeEnter != m_currentState)
		{
			status = STATE_CONTINUE;
		}

		if (IS_STATE_SLEEP(status))
		{
			// hey, we're sleepy!
			UnsignedInt now = TheGameLogic->getFrame();
			m_sleepTill = now + GET_STATE_SLEEP_FRAMES(status);
			return m_currentState->friend_checkForSleepTransitions( STATE_SLEEP(m_sleepTill - now) );
		}
		else
		{
			// check for state transitions, possibly exiting this machine
			return m_currentState->friend_checkForTransitions( status );
		}
	}
	else
	{
		return STATE_CONTINUE;  // irrelevant return code, but we must return something
	}
}

//-----------------------------------------------------------------------------
/**
 * Define the default state of the machine, and
 * set the machine's state to it.
 */
StateReturnType StateMachine::initDefaultState()
{
#ifdef DEBUG_LOGGING
#ifdef STATE_MACHINE_DEBUG
#define REALLY_VERBOSE_LOG(x) /* DEBUG_LOG_RAW(x) */
	// Run through all the transitions and make sure there aren't any transitions to undefined states. jba. [8/18/2003]
	std::map<StateID, State *>::iterator i;
	REALLY_VERBOSE_LOG(("SM_BEGIN\n"));
	for( i = m_stateMap.begin(); i != m_stateMap.end(); ++i ) {
		State *state = (*i).second;
		StateID id = state->getID();
		// Check transitions. [8/18/2003]
		std::vector<StateID> *ids = state->getTransitions();
		// check transitions
		REALLY_VERBOSE_LOG(("State %s(%d) : ", state->getName().str(), id));
		if (!ids->empty())
		{
			for(std::vector<StateID>::const_iterator it = ids->begin(); it != ids->end(); ++it)
			{
				StateID curID = *it;
				REALLY_VERBOSE_LOG(("%d('", curID));
				if (curID == INVALID_STATE_ID) {
					REALLY_VERBOSE_LOG(("INVALID_STATE_ID', "));
					continue;
				}
				if (curID == EXIT_MACHINE_WITH_SUCCESS) {
					REALLY_VERBOSE_LOG(("EXIT_MACHINE_WITH_SUCCESS', "));
					continue;
				}
				if (curID == EXIT_MACHINE_WITH_FAILURE) {
					REALLY_VERBOSE_LOG(("EXIT_MACHINE_WITH_FAILURE', "));
					continue;
				}
				// locate the actual state associated with the given ID
				std::map<StateID, State *>::iterator i;
				i = m_stateMap.find( curID );

				if (i == m_stateMap.end()) {
					DEBUG_LOG(("\nState %s(%d) : Transition %d not found", state->getName().str(), id, curID));
					DEBUG_LOG(("This MUST BE FIXED!!!jba"));
					DEBUG_CRASH(("Invalid transition."));
				} else {
					State *st = (*i).second;
					if (st->getName().isNotEmpty()) {
						REALLY_VERBOSE_LOG(("%s') ", st->getName().str()));
					}
				}
			}
		}
		REALLY_VERBOSE_LOG(("\n"));
		delete ids;
		ids = nullptr;
	}
	REALLY_VERBOSE_LOG(("SM_END\n\n"));
#endif
#endif
	DEBUG_ASSERTCRASH(!m_locked, ("Machine is locked here, but probably should not be"));
	if (m_defaultStateInited)
	{
		DEBUG_CRASH(("you may not call initDefaultState twice for the same StateMachine"));
		return STATE_FAILURE;
	}
	else
	{
		m_defaultStateInited = true;
		return internalSetState( m_defaultStateID );
	}
}

//-----------------------------------------------------------------------------
void StateMachine::setGoalObject( const Object *obj )
{
	if (m_locked)
		return;

	internalSetGoalObject( obj );
}

//-----------------------------------------------------------------------------
Bool StateMachine::isGoalObjectDestroyed() const
{
	if (m_goalObjectID == 0)
	{
		return false; // never had a goal object
	}
	return getGoalObject() == nullptr;
}

//-----------------------------------------------------------------------------
void StateMachine::halt()
{
	m_locked = true;
	m_currentState = nullptr; // don't exit current state, just clear it.
#ifdef STATE_MACHINE_DEBUG
	if (getWantsDebugOutput())
	{
		DEBUG_LOG(("%d '%s' -- '%s' %x halt()", TheGameLogic->getFrame(), m_owner->getTemplate()->getName().str(), m_name.str(), this));
	}
#endif
}

//-----------------------------------------------------------------------------
void StateMachine::internalSetGoalObject( const Object *obj )
{
	if (obj) {
		m_goalObjectID = obj->getID();
		internalSetGoalPosition(obj->getPosition());
	}
	else {
		m_goalObjectID = INVALID_ID;
	}
}

//-----------------------------------------------------------------------------
Object *StateMachine::getGoalObject()
{
	return TheGameLogic->findObjectByID( m_goalObjectID );
}

//-----------------------------------------------------------------------------
const Object *StateMachine::getGoalObject() const
{
	return TheGameLogic->findObjectByID( m_goalObjectID );
}

//-----------------------------------------------------------------------------
void StateMachine::setGoalPosition( const Coord3D *pos )
{
	if (m_locked)
		return;

	internalSetGoalPosition( pos );
}

//-----------------------------------------------------------------------------
void StateMachine::internalSetGoalPosition( const Coord3D *pos )
{
	if (pos) {
		m_goalPosition = *pos;
		// Don't clear the goal object, or everything breaks.  Like construction of buildings.
	}
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void StateMachine::crc( Xfer *xfer )
{

}

// ------------------------------------------------------------------------------------------------
/** Xfer Method
	* Version Info
	* 1: Initial version
	*/
// ------------------------------------------------------------------------------------------------
void StateMachine::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	xfer->xferUnsignedInt(&m_sleepTill);
	xfer->xferUnsignedInt(&m_defaultStateID);
	StateID curStateID = getCurrentStateID();
	xfer->xferUnsignedInt(&curStateID);
	if (xfer->getXferMode() == XFER_LOAD)	{
		// We are going to jump into the current state.	We don't call onEnter or onExit, because the
		// state was already active when we saved.
		m_currentState = internalGetState( curStateID );
	}

	Bool snapshotAllStates = false;
#ifdef RTS_DEBUG
	//snapshotAllStates = true;
#endif
	xfer->xferBool(&snapshotAllStates);
	if (snapshotAllStates) {
		std::map<StateID, State *>::iterator i;
		// count all states in the mapping
		Int count = 0;
		for( i = m_stateMap.begin(); i != m_stateMap.end(); ++i )
			count++;
		Int saveCount = count;
		xfer->xferInt(&saveCount);
		if (saveCount!=count) {
			DEBUG_CRASH(("State count mismatch - %d expected, %d read", count, saveCount));
			throw SC_INVALID_DATA;
		}
		for( i = m_stateMap.begin(); i != m_stateMap.end(); ++i ) {
			State *state = (*i).second;
			if( state != nullptr )
			{
				StateID id = state->getID();
				xfer->xferUnsignedInt(&id);
				if (id!=state->getID()) {
					DEBUG_CRASH(("State ID mismatch - %d expected, %d read", state->getID(), id));
					throw SC_INVALID_DATA;
				}
			}
			else
			{
				DEBUG_CRASH(("state was null on xfer, trying to heal..."));
				// Hmm... too late to find out why we are getting nullptr in our state, but if we let it go, we will Throw in xferSnapshot.
				state = internalGetState(m_defaultStateID);
				StateID id = state->getID();
				xfer->xferUnsignedInt(&id);
			}

			xfer->xferSnapshot(state);
		}

	}	else {
		if( m_currentState == nullptr )
		{
			DEBUG_ASSERTCRASH(m_currentState != nullptr, ("currentState was null on xfer, trying to heal..."));
			// Hmm... too late to find out why we are getting nullptr in our state, but if we let it go, we will Throw in xferSnapshot.
			m_currentState = internalGetState(m_defaultStateID);
		}
		xfer->xferSnapshot(m_currentState);
	}


	xfer->xferObjectID(&m_goalObjectID);
	xfer->xferCoord3D(&m_goalPosition);
	xfer->xferBool(&m_locked);
	xfer->xferBool(&m_defaultStateInited);
}


// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void StateMachine::loadPostProcess()
{

}
```


---

## 4. Architect Notes & Safe Re-Entrant Rust Design

1. **Disjoint-Borrow Safe Rust Architecture (Zero `RefCell`, Zero `HashMap`):**
   - Look at `StateMachine.cpp:439, 553, 622`: `State::onEnter()` and `State::update()` can invoke `getMachine()->setState(...)` **re-entrantly**, and then return a sleep code (`STATE_SLEEP(999)`) that `internalSetState` / `updateStateMachine` must discard (`if stateBefore != currentState { status = STATE_CONTINUE; }`).
   - Using [`StateCallbacks<Ctx>`](../../crates/zh_common/src/core/state_machine.rs) with `Copy` function pointers (`on_enter`, `on_exit`, `update`, `xfer_state`, `cond.test`) stored in a deterministic `BTreeMap<StateId, StateEntry<Ctx>>` (`DET001` compliant, no `HashMap` and no `RefCell`/`Rc` `DET010`) allows `StateMachine<Ctx>` to copy the function pointers out before calling `on_enter(machine, ctx)` / `update(machine, ctx)`, so re-entrant `machine.set_state(next, ctx)` compiles cleanly in 100% safe Rust.
   - `Ctx` implements `StateWorldView` (`current_frame()`, `object_exists(ObjectId)`), letting `update_state_machine(&mut self, ctx: &mut Ctx)` and `is_goal_object_destroyed(&self, world: &Ctx)` query the current frame and target object without globals.

2. **Exact Sentinel Constants (`StateMachine.h:54-112`):**
   - `MACHINE_DONE_STATE_ID: StateId = 999_998`
   - `INVALID_STATE_ID: StateId = 999_999`
   - `EXIT_MACHINE_WITH_SUCCESS: StateId = 9_998`
   - `EXIT_MACHINE_WITH_FAILURE: StateId = 9_999`
   - `STATE_CONTINUE: Int = 0`, `STATE_SUCCESS: Int = -1`, `STATE_FAILURE: Int = -2`
   - `STATE_SLEEP_FOREVER: Int = 0x3fff_ffff` (`StateReturnType::Sleep(0x3fff_ffff)` or raw `Int > 0`)
   - `MAX_TRANSITION_DEPTH: u32 = 20` (both normal transitions and sleep transitions track depth separately; depth `>= 20` immediately returns `STATE_FAILURE`).

3. **Subtle Edge Cases Verified by `PKT-077.json`:**
   - **Condition vs `success_id`/`failure_id` Exit Difference (`StateMachine.cpp:128-153` vs `178-185, 236-243`):**
     - When `success_id` / `failure_id` equals `EXIT_MACHINE_WITH_SUCCESS` / `EXIT_MACHINE_WITH_FAILURE`, C++ first calls `internal_set_state(MACHINE_DONE_STATE_ID)` (invoking old state's `on_exit(Normal)` and clearing `current_state = None`) before returning `Success` / `Failure`.
     - When a **condition** transition has `to_state_id == EXIT_MACHINE_WITH_SUCCESS` / `EXIT_MACHINE_WITH_FAILURE`, C++ returns `Success` / `Failure` directly *without* calling `internal_set_state(MACHINE_DONE_STATE_ID)`.
   - **`halt()` vs `clear()` (`StateMachine.cpp:344-363, 753-763`):** `clear()` invokes `on_exit(Reset)` if a state is active and clears goal object/position; `halt()` locks the machine and clears `current_state = None` *without* calling `on_exit`.
   - **Null Current State & Stateless Queries (`StateMachine.h:255-263`):** When `current_state` is `None`, `get_current_state_id()` is `INVALID_STATE_ID`, `is_in_idle_state()` / `is_in_attack_state()` / `is_in_force_attack_state()` return `TRUE`, while `is_in_guard_idle_state()` / `is_in_busy_state()` return `FALSE`.
   - **Binary `Snapshot::xfer` v1 Field Order (`StateMachine.cpp:821-894`):**
     1. `version` (`u8 = 1`) via `xfer_version`
     2. `sleep_till` (`u32`)
     3. `default_state_id` (`u32`)
     4. `cur_state_id` (`u32`; on `XferMode::Load`, calls `internal_get_state_id` falling back to `default_state_id` without invoking `on_enter`/`on_exit`)
     5. `snapshot_all_states` (`Bool = FALSE`)
     6. Current state snapshot (`xfer_state` on `current_state_id`, or if `current_state` is `None`, C++ heals `current_state` to `default_state_id` BEFORE calling `xfer_state`, leaving `cur_state_id` above as `INVALID_STATE_ID`!)
     7. `goal_object_id`, `goal_position`, `locked`, `default_state_inited`.
