# Translation Packet `PKT-444`: `MessageStream`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-444`
- **Work Package:** `WP-7.1` (Phase 7)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web` (seeded with C++ Release oracle `docs/golden_vectors/PKT-444.json` & Rust skeleton)
- **FFI Seam Pattern:** Core Spine
- **Allowed Rust Output Files:**
  - `crates/zh_logic/src/gamelogic/message_stream.rs`
  - `crates/zh_logic/tests/golden_pkt444_messagestream.rs` *(remove `#[ignore]` attributes once implemented)*
  - `docs/PORT_MAP.md` *(regenerate with `python3 scripts/port_map.py --write`)*
- **Mandatory V&V Gates:** `V1, V2-Golden`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test -p zh_logic --test golden_pkt444_messagestream && python3 scripts/port_map.py --write && cargo test --workspace
  ```

## Port Contract (read first)

1. **Release-Aligned Enum Ordinals (`GameMessageType`).** The oracle (`docs/golden_vectors/PKT-444.json`) is compiled with **all** debug/cheat guards **undefined** (`-URTS_DEBUG -U_ALLOW_DEBUG_CHEATS_IN_RELEASE -UALLOW_SURRENDER -UDUMP_PERF_STATS`). Under this configuration:
   - `MSG_BEGIN_NETWORK_MESSAGES == 1000`
   - `MSG_BEGIN_DEBUG_NETWORK_MESSAGES == 1900`
   - `MSG_END_NETWORK_MESSAGES == 1999`
   - `MSG_COUNT == 2006`
   Do **not** shift discriminants by adding `RTS_DEBUG` / `_ALLOW_DEBUG_CHEATS_IN_RELEASE` / `ALLOW_SURRENDER` variants; keep the exact discriminants already verified in `message_stream.rs`.
2. **`get_command_type_as_string_raw(raw: Int) -> &'static str`:**
   - If `raw >= MSG_COUNT` (`2006`), or `raw < 0` if treated as unsigned? Check C++: `GameMessage::Type` is an enum, `(t >= GameMessage::MSG_COUNT) ? "Invalid command" : "UnknownMessage"` (`t < 0` in C++ signed enum returns `"UnknownMessage"`). Match every named enum value to its exact C `MSG_*` string (`"MSG_INVALID"`, `"MSG_FRAME_TICK"`, etc.); return `"Invalid command"` for `raw >= MSG_COUNT`, and `"UnknownMessage"` for unnamed gaps (such as `182..999`, `1098..1899`, `1901..1998`, or negative values).
3. **`GameMessage` arguments & out-of-range fallback:**
   - Each appended argument uses the matching `GameMessageArgumentDataType` variant (`Integer..=WideChar`). `get_argument_count()` returns `UnsignedByte` (`(len as UnsignedByte)`).
   - `get_argument(arg_index)` returns a zeroed `GameMessageArgumentValue` default (and `get_argument_data_type(arg_index)` returns `GameMessageArgumentDataType::Unknown`) whenever `arg_index < 0` or `arg_index as usize >= args.len()` (never panic on out-of-range!).
4. **`GameMessageList` / `CommandList` ordering & insertion:**
   - Preserve C++ doubly-linked ordering: `append_message` adds at tail; `insert_message_after(msg, after_id)` inserts immediately after `after_id` (returning `None` cleanly if `after_id` is absent); `remove_message(id)` unlinks; `contains_message_of_type(t)` scans front-to-back; `CommandList::append_message_list` moves all messages from the stream into the end of `CommandList`.
5. **Translator priority ordering & `propagate_messages`:**
   - `MessageStream::new()` initializes `next_translator_id = 1`.
   - `attach_translator` inserts BEFORE the first existing translator whose `priority > new_priority` (strict `>`!), preserving **FIFO order among equal priorities**.
   - `propagate_messages(&mut self, command_list: &mut CommandList)` iterates translators in priority order (outer loop) and active stream messages (inner loop). Note that if a translator returns `GameMessageDisposition::DestroyMessage`, that message is destroyed immediately and not passed to subsequent translators.
   - Surviving messages are moved onto `command_list`, and then `command_list` is pruned with `is_redundant_network_message`:
     - `MsgDestroySelectedGroup` is redundant if it has an immediate next message `next` in `command_list` and EITHER:
       - `next` is `MsgCreateSelectedGroup` or `MsgCreateSelectedGroupNoSound` AND `next.get_argument_count() >= 1` AND `next.get_argument(0).as_bool() != FALSE`, OR
       - `next` is `MsgDestroySelectedGroup`, OR
       - `next` is `MsgSelectTeam0`..=`MsgSelectTeam9`.
     - `MsgSetReplayCamera` is redundant if **any** later message anywhere after it in `command_list` is `MsgSetReplayCamera`.
6. **`build_region(anchor, dest)`:** normalizes `lo = min(anchor, dest)` and `hi = max(anchor, dest)` component-wise.
7. **Determinism & 3-attempt stall rule:** Zero external crates; no `HashMap`/`HashSet`/`BinaryHeap`; pass `python3 scripts/lint_determinism.py`. If 3 consecutive fix attempts on failing golden vectors do not reduce the failure count, isolate the minimal failing seed, re-read the C++ source in section 3 below, or escalate (`tier:3-architect`).

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (11):**
  - `Common/GameCommon.h`
  - `Common/GameMemory.h`
  - `Common/MessageStream.h`
  - `Common/Player.h`
  - `Common/PlayerList.h`
  - `Common/Recorder.h`
  - `Common/SubsystemInterface.h`
  - `GameClient/InGameUI.h`
  - `GameLogic/GameLogic.h`
  - `Lib/BaseType.h`
  - `PreRTS.h`
- **`Snapshot::crc` / `Snapshot::xfer` Calls:** None in module.

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

### `Core/GameEngine/Include/Common/MessageStream.h`
```cpp
// FILE: MessageStream.h //////////////////////////////////////////////////////
// The message stream propagates all messages thru a series of "translators"
// Author: Michael S. Booth, February 2001

#pragma once

#include "Common/GameCommon.h"	// ensure we get DUMP_PERF_STATS, or not
#include "Common/SubsystemInterface.h"
#include "Lib/BaseType.h"
#include "Common/GameMemory.h"


enum { TRANSLATOR_ID_INVALID = -1 };

// how far the the cursor moves before a click becomes a drag


typedef UnsignedInt TranslatorID;								///< Unique identifiers for message stream translators

class Drawable;
class GameMessageList;
enum ObjectID CPP_11(: Int);
enum DrawableID CPP_11(: Int);

union GameMessageArgumentType														///< Union of possible data for given message type
{
	Int							integer;
	Real 						real;
	Bool						boolean;
	ObjectID				objectID;
	DrawableID			drawableID;
	UnsignedInt			teamID;
	UnsignedInt			squadID;
	Coord3D					location;
	ICoord2D				pixel;
	IRegion2D				pixelRegion;
	UnsignedInt			timestamp;
	WideChar				wChar;
};

enum GameMessageArgumentDataType CPP_11(: Int)
{
	ARGUMENTDATATYPE_INTEGER,
	ARGUMENTDATATYPE_REAL,
	ARGUMENTDATATYPE_BOOLEAN,
	ARGUMENTDATATYPE_OBJECTID,
	ARGUMENTDATATYPE_DRAWABLEID,
	ARGUMENTDATATYPE_TEAMID,
	ARGUMENTDATATYPE_LOCATION,
	ARGUMENTDATATYPE_PIXEL,
	ARGUMENTDATATYPE_PIXELREGION,
	ARGUMENTDATATYPE_TIMESTAMP,
	ARGUMENTDATATYPE_WIDECHAR,
	ARGUMENTDATATYPE_UNKNOWN
};

class GameMessageArgument : public MemoryPoolObject
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(GameMessageArgument, "GameMessageArgument")
public:
	GameMessageArgumentType			m_data;									///< The data storage of an argument
	GameMessageArgumentDataType	m_type;									///< The type of the argument.
};
EMPTY_DTOR(GameMessageArgument)

/**
 * A game message that either lives on TheMessageStream or TheCommandList.
 * Messages consist of a type, defining what the message is, and zero or more arguments
 * of various data types.  The user of a message must know how many and what type of
 * arguments are valid for a given message type.
 */
class GameMessage : public MemoryPoolObject
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(GameMessage, "GameMessage")

public:

	/// The various messages which can be sent in a MessageStream
	/// @todo Replace this hardcoded enum with a generalized system that can be easily changed and updated
	/** @todo Because the Client will run faster than Logic, we'll need "superseding" messages for events
						such as mouse movements so we only send the latest one over the net */
	/**	@todo Create two classes of message: raw input messages, and command messages. Raw input messages
						will be destroyed when they reach the end of the stream, whereas command messages will be
						transferred to TheCommandList */
	enum Type
	{
		MSG_INVALID,																///< (none) this msg should never actually occur

		MSG_FRAME_TICK,															///< (timestamp) once each frame this message is sent thru the stream

		// Client to Server messages
		// Note: Please keep MSG_RAW_MOUSE between MSG_RAW_MOUSE_BEGIN and MSG_RAW_MOUSE_END
		MSG_RAW_MOUSE_BEGIN,
		MSG_RAW_MOUSE_POSITION,											///< (pixel) the cursor's position
		MSG_RAW_MOUSE_LEFT_BUTTON_DOWN,							///< (pixel, modifiers, time)
		MSG_RAW_MOUSE_LEFT_DOUBLE_CLICK,						///< (pixel, modifiers, time)
		MSG_RAW_MOUSE_LEFT_BUTTON_UP,								///< (pixel, modifiers, time)
		MSG_RAW_MOUSE_LEFT_CLICK,										///< (pixel, modifiers, time)
		MSG_RAW_MOUSE_LEFT_DRAG,										///< drag of the mouse with a button held down
		MSG_RAW_MOUSE_MIDDLE_BUTTON_DOWN,						///< (pixel, modifiers, time)
		MSG_RAW_MOUSE_MIDDLE_DOUBLE_CLICK,					///< (pixel, modifiers, time)
		MSG_RAW_MOUSE_MIDDLE_BUTTON_UP,							///< (pixel, modifiers, time)
		MSG_RAW_MOUSE_MIDDLE_DRAG,									///< drag of the mouse with a button held down
		MSG_RAW_MOUSE_RIGHT_BUTTON_DOWN,						///< (pixel, modifiers, time)
		MSG_RAW_MOUSE_RIGHT_DOUBLE_CLICK,						///< (pixel, modifiers, time)
		MSG_RAW_MOUSE_RIGHT_BUTTON_UP,							///< (pixel, modifiers, time)
		MSG_RAW_MOUSE_RIGHT_DRAG,										///< drag of the mouse with a button held down
		MSG_RAW_MOUSE_WHEEL,												///< (Real spin, + is away, - is toward user)
		MSG_RAW_MOUSE_END,

		MSG_RAW_KEY_DOWN,														///< (KeyDefType) the given key was pressed (uses Microsoft VK_ codes)
		MSG_RAW_KEY_UP,															///< (KeyDefType) the given key was released

		// Refined Mouse messages
		// NOTE: All processing should attempt to use these refined mouse messages, rather than the
		// RAW_* variants. (Please.) :-) jkmcd
		MSG_MOUSE_LEFT_CLICK,												///< (pixelRegion, 0 sized means its a point), (Int, modifier keys)
		MSG_MOUSE_LEFT_DOUBLE_CLICK,								///< (pixelRegion, 0 sized means its a point), (Int, modifier keys)
		MSG_MOUSE_MIDDLE_CLICK,											///< (pixelRegion, 0 sized means its a point), (Int, modifier keys)
		MSG_MOUSE_MIDDLE_DOUBLE_CLICK,							///< (pixelRegion, 0 sized means its a point), (Int, modifier keys)
		MSG_MOUSE_RIGHT_CLICK,											///< (pixelRegion, 0 sized means its a point), (Int, modifier keys)
		MSG_MOUSE_RIGHT_DOUBLE_CLICK,								///< (pixelRegion, 0 sized means its a point), (Int, modifier keys)
		// End Refined Mouse Messages

		MSG_CLEAR_GAME_DATA,												///< Clear all game data in memory
		MSG_NEW_GAME,																///< Start a new game

		MSG_LOGIC_CRC_PLAYBACK,                     ///< TheSuperHackers @tweak CRC from the logic used internally for replay playback.

		// "meta" messages should be thought of as "virtual keystrokes" -- they exist
		// solely to provide an abstraction layer useful for keyboard/mouse remapping.
		// they should NEVER be sent over the network.
		MSG_BEGIN_META_MESSAGES,										///< Marker to delineate "meta" messages

		MSG_META_SAVE_VIEW1,												///< save current view as the given user-defined view
		MSG_META_SAVE_VIEW2,												///< save current view as the given user-defined view
		MSG_META_SAVE_VIEW3,												///< save current view as the given user-defined view
		MSG_META_SAVE_VIEW4,												///< save current view as the given user-defined view
		MSG_META_SAVE_VIEW5,												///< save current view as the given user-defined view
		MSG_META_SAVE_VIEW6,												///< save current view as the given user-defined view
		MSG_META_SAVE_VIEW7,												///< save current view as the given user-defined view
		MSG_META_SAVE_VIEW8,												///< save current view as the given user-defined view
		MSG_META_VIEW_VIEW1,												///< center view on the given user-defined view
		MSG_META_VIEW_VIEW2,												///< center view on the given user-defined view
		MSG_META_VIEW_VIEW3,												///< center view on the given user-defined view
		MSG_META_VIEW_VIEW4,												///< center view on the given user-defined view
		MSG_META_VIEW_VIEW5,												///< center view on the given user-defined view
		MSG_META_VIEW_VIEW6,												///< center view on the given user-defined view
		MSG_META_VIEW_VIEW7,												///< center view on the given user-defined view
		MSG_META_VIEW_VIEW8,												///< center view on the given user-defined view
		MSG_META_CREATE_TEAM0,											///< create user-defined team from the selected objects
		MSG_META_CREATE_TEAM1,											///< create user-defined team from the selected objects
		MSG_META_CREATE_TEAM2,											///< create user-defined team from the selected objects
		MSG_META_CREATE_TEAM3,											///< create user-defined team from the selected objects
		MSG_META_CREATE_TEAM4,											///< create user-defined team from the selected objects
		MSG_META_CREATE_TEAM5,											///< create user-defined team from the selected objects
		MSG_META_CREATE_TEAM6,											///< create user-defined team from the selected objects
		MSG_META_CREATE_TEAM7,											///< create user-defined team from the selected objects
		MSG_META_CREATE_TEAM8,											///< create user-defined team from the selected objects
		MSG_META_CREATE_TEAM9,											///< create user-defined team from the selected objects
		MSG_META_SELECT_TEAM0,											///< deselect all, then select all units in the given user-defined team
		MSG_META_SELECT_TEAM1,											///< deselect all, then select all units in the given user-defined team
		MSG_META_SELECT_TEAM2,											///< deselect all, then select all units in the given user-defined team
		MSG_META_SELECT_TEAM3,											///< deselect all, then select all units in the given user-defined team
		MSG_META_SELECT_TEAM4,											///< deselect all, then select all units in the given user-defined team
		MSG_META_SELECT_TEAM5,											///< deselect all, then select all units in the given user-defined team
		MSG_META_SELECT_TEAM6,											///< deselect all, then select all units in the given user-defined team
		MSG_META_SELECT_TEAM7,											///< deselect all, then select all units in the given user-defined team
		MSG_META_SELECT_TEAM8,											///< deselect all, then select all units in the given user-defined team
		MSG_META_SELECT_TEAM9,											///< deselect all, then select all units in the given user-defined team
		MSG_META_ADD_TEAM0,													///< add the user-defined team to the current selection
		MSG_META_ADD_TEAM1,													///< add the user-defined team to the current selection
		MSG_META_ADD_TEAM2,													///< add the user-defined team to the current selection
		MSG_META_ADD_TEAM3,													///< add the user-defined team to the current selection
		MSG_META_ADD_TEAM4,													///< add the user-defined team to the current selection
		MSG_META_ADD_TEAM5,													///< add the user-defined team to the current selection
		MSG_META_ADD_TEAM6,													///< add the user-defined team to the current selection
		MSG_META_ADD_TEAM7,													///< add the user-defined team to the current selection
		MSG_META_ADD_TEAM8,													///< add the user-defined team to the current selection
		MSG_META_ADD_TEAM9,													///< add the user-defined team to the current selection
		MSG_META_VIEW_TEAM0,												///< center view on given user-defined team (but do not affect selection)
		MSG_META_VIEW_TEAM1,												///< center view on given user-defined team (but do not affect selection)
		MSG_META_VIEW_TEAM2,												///< center view on given user-defined team (but do not affect selection)
		MSG_META_VIEW_TEAM3,												///< center view on given user-defined team (but do not affect selection)
		MSG_META_VIEW_TEAM4,												///< center view on given user-defined team (but do not affect selection)
		MSG_META_VIEW_TEAM5,												///< center view on given user-defined team (but do not affect selection)
		MSG_META_VIEW_TEAM6,												///< center view on given user-defined team (but do not affect selection)
		MSG_META_VIEW_TEAM7,												///< center view on given user-defined team (but do not affect selection)
		MSG_META_VIEW_TEAM8,												///< center view on given user-defined team (but do not affect selection)
		MSG_META_VIEW_TEAM9,												///< center view on given user-defined team (but do not affect selection)

		MSG_META_SELECT_MATCHING_UNITS,              ///< selects matching units, used for both on screen and across map
		MSG_META_SELECT_NEXT_UNIT,									///< select 'next' unit
		MSG_META_SELECT_PREV_UNIT,									///< select 'prev' unit
		MSG_META_SELECT_NEXT_WORKER,                ///< select 'next' worker
		MSG_META_SELECT_PREV_WORKER,                ///< select 'prev' worker
		MSG_META_SELECT_NEXT_IDLE_WORKER,           ///< TheSuperHackers @feature L3-M 03/08/2025 select next idle worker
		MSG_META_VIEW_COMMAND_CENTER,								///< center view on command center
		MSG_META_VIEW_LAST_RADAR_EVENT,							///< center view on last radar event
		MSG_META_SELECT_HERO,                       ///< selects player's hero character, if exists...
		MSG_META_SELECT_ALL,                        ///< selects all units across screen
		MSG_META_SELECT_ALL_AIRCRAFT,								///< selects all air units just like select all, Added in Zero Hour
		MSG_META_SCATTER,														///< selected units scatter
		MSG_META_STOP,															///< selected units stop
		MSG_META_DEPLOY,														///< selected units 'deploy'
		MSG_META_CREATE_FORMATION,									///< selected units become a formation
		MSG_META_FOLLOW,														///< selected units 'follow'
		MSG_META_CHAT_PLAYERS,											///< send chat msg to all players
		MSG_META_CHAT_ALLIES,												///< send chat msg to allied players
		MSG_META_CHAT_EVERYONE,											///< send chat msg to everyone (incl. observers)
		MSG_META_DIPLOMACY,													///< bring up diplomacy screen
		MSG_META_OPTIONS,														///< bring up options screen
#if defined(RTS_DEBUG)
		MSG_META_HELP,															///< bring up help screen
#endif

		MSG_META_INCREASE_MAX_RENDER_FPS,						///< TheSuperHackers @feature Increase the max render fps
		MSG_META_DECREASE_MAX_RENDER_FPS,						///< TheSuperHackers @feature Decrease the max render fps
		MSG_META_INCREASE_LOGIC_TIME_SCALE,					///< TheSuperHackers @feature Increase the logic time scale
		MSG_META_DECREASE_LOGIC_TIME_SCALE,					///< TheSuperHackers @feature Decrease the logic time scale
		MSG_META_TOGGLE_LOWER_DETAILS,							///< toggles graphics options to crappy mode instantly
		MSG_META_TOGGLE_CONTROL_BAR,								///< show/hide controlbar
		MSG_META_TOGGLE_PLAYER_OBSERVER,						///< TheSuperHackers @feature Toggle the player observer view in game

		MSG_META_BEGIN_PATH_BUILD,									///< enter path-building mode
		MSG_META_END_PATH_BUILD,										///< exit path-building mode
		MSG_META_BEGIN_FORCEATTACK,									///< enter force-attack mode
		MSG_META_END_FORCEATTACK,										///< exit force-attack mode
		MSG_META_BEGIN_FORCEMOVE,										///< enter force-move mode
		MSG_META_END_FORCEMOVE,											///< exit force-move mode
		MSG_META_BEGIN_WAYPOINTS,										///< enter waypoint mode
		MSG_META_END_WAYPOINTS,											///< exit waypoint mode
		MSG_META_BEGIN_PREFER_SELECTION,						///< The Shift key has been depressed alone
		MSG_META_END_PREFER_SELECTION,							///< The Shift key has been released.

		MSG_META_TAKE_SCREENSHOT,										///< take JPEG screenshot
		MSG_META_TAKE_SCREENSHOT_PNG,							///< TheSuperHackers @feature Take lossless PNG screenshot
		MSG_META_ALL_CHEER,													///< Yay! :)
		MSG_META_TOGGLE_ATTACKMOVE,									///< enter attack-move mode

		MSG_META_BEGIN_CAMERA_ROTATE_LEFT,
		MSG_META_END_CAMERA_ROTATE_LEFT,
		MSG_META_ALT_CAMERA_ROTATE_LEFT,						///< TheSuperHackers @feature Rotate camera in 45 degree increments
		MSG_META_BEGIN_CAMERA_ROTATE_RIGHT,
		MSG_META_END_CAMERA_ROTATE_RIGHT,
		MSG_META_ALT_CAMERA_ROTATE_RIGHT,						///< TheSuperHackers @feature Rotate camera in 45 degree increments
		MSG_META_BEGIN_CAMERA_ZOOM_IN,
		MSG_META_END_CAMERA_ZOOM_IN,
		MSG_META_BEGIN_CAMERA_ZOOM_OUT,
		MSG_META_END_CAMERA_ZOOM_OUT,
		MSG_META_CAMERA_RESET,
    MSG_META_TOGGLE_CAMERA_TRACKING_DRAWABLE, // Added in Zero Hour
		MSG_META_TOGGLE_FAST_FORWARD_REPLAY,				///< Toggle the fast forward feature, // Added in Zero Hour
		MSG_META_TOGGLE_PAUSE,											///< TheSuperHackers @feature Toggle game pause
		MSG_META_TOGGLE_PAUSE_ALT,									///< TheSuperHackers @feature Toggle game pause (alternative mapping)
		MSG_META_STEP_FRAME,												///< TheSuperHackers @feature Step one frame
		MSG_META_STEP_FRAME_ALT,										///< TheSuperHackers @feature Step one frame (alternative mapping)
		MSG_META_DEMO_INSTANT_QUIT,									///< bail out of game immediately


#if defined(_ALLOW_DEBUG_CHEATS_IN_RELEASE) // All MSG_CHEAT messages added in Zero Hour
    MSG_CHEAT_RUNSCRIPT1,										///< run script named "KEY_F1"
		MSG_CHEAT_RUNSCRIPT2,										///< run script named "KEY_F2"
		MSG_CHEAT_RUNSCRIPT3,										///< run script named "KEY_F3"
		MSG_CHEAT_RUNSCRIPT4,										///< run script named "KEY_F4"
		MSG_CHEAT_RUNSCRIPT5,										///< run script named "KEY_F5"
		MSG_CHEAT_RUNSCRIPT6,										///< run script named "KEY_F6"
		MSG_CHEAT_RUNSCRIPT7,										///< run script named "KEY_F7"
		MSG_CHEAT_RUNSCRIPT8,										///< run script named "KEY_F8"
		MSG_CHEAT_RUNSCRIPT9,										///< run script named "KEY_F9"
		MSG_CHEAT_TOGGLE_SPECIAL_POWER_DELAYS,	///< Toggle special power delays on/off
		MSG_CHEAT_SWITCH_TEAMS,									///< switch local control to another team
		MSG_CHEAT_KILL_SELECTION,								///< kill the selected units (yeah!)
		MSG_CHEAT_TOGGLE_HAND_OF_GOD_MODE,			///< do 100% damage to the selected units (w00t!)
		MSG_CHEAT_INSTANT_BUILD,								///< All building is with a timer of 1
		MSG_CHEAT_DESHROUD,											///< de-shroud the world for the local player
		MSG_CHEAT_ADD_CASH,											///< adds 10000 cash to the player
		MSG_CHEAT_GIVE_ALL_SCIENCES,						///< grant all grantable sciences
		MSG_CHEAT_GIVE_SCIENCEPURCHASEPOINTS,		///< give yourself an SPP (but no rank change)
		MSG_CHEAT_SHOW_HEALTH,									///< show object health
    MSG_CHEAT_TOGGLE_MESSAGE_TEXT,          ///< hides/shows the onscreen messages

#endif

		// META items that are really for debug/demo/development use only...
		// They do not get built into RELEASE builds.
#if defined(RTS_DEBUG)
		MSG_META_DEMO_TOGGLE_BEHIND_BUILDINGS,			///< Toggles showing units behind buildings or not
		MSG_META_DEMO_TOGGLE_LETTERBOX,							///< enable/disable letterbox mode
		MSG_META_DEMO_TOGGLE_MESSAGE_TEXT,					///< toggle the text from the UI messages
		MSG_META_DEMO_LOD_DECREASE,                 ///< decrease LOD by 1
		MSG_META_DEMO_LOD_INCREASE,                 ///< increase LOD by 1
		MSG_META_DEMO_TOGGLE_ZOOM_LOCK,							///< Toggle the camera zoom lock on/off
		MSG_META_DEMO_PLAY_CAMEO_MOVIE,							///< Play a movie in the cameo spot
		MSG_META_DEMO_TOGGLE_SPECIAL_POWER_DELAYS,	///< Toggle special power delays on/off
		MSG_META_DEMO_BATTLE_CRY,										///< battle cry
		MSG_META_DEMO_SWITCH_TEAMS,									///< switch local control to another team
		MSG_META_DEMO_SWITCH_TEAMS_BETWEEN_CHINA_USA, ///< switch the local player between china and usa
		MSG_META_DEMO_TOGGLE_PARTICLEDEBUG,					///< show/hide the particle system debug info
		MSG_META_DEMO_TOGGLE_SHADOW_VOLUMES,				///< show/hide shadow volumes
		MSG_META_DEMO_TOGGLE_FOGOFWAR,
		MSG_META_DEMO_KILL_ALL_ENEMIES,							///< kill ALL ENEMIES! (yeah!)
		MSG_META_DEMO_KILL_SELECTION,								///< kill the selected units (yeah!)
		MSG_META_DEMO_TOGGLE_HURT_ME_MODE,					///< do 10% damage to the selected units (yeah!)
		MSG_META_DEMO_TOGGLE_HAND_OF_GOD_MODE,			///< do 100% damage to the selected units (w00t!)
		MSG_META_DEMO_DEBUG_SELECTION,							///< select a given unit for state-machine debugging
		MSG_META_DEMO_LOCK_CAMERA_TO_SELECTION,			///< lock the view camera to the selected object
		MSG_META_DEMO_TOGGLE_SOUND,									///< toggle sound/video on/off
		MSG_META_DEMO_TOGGLE_TRACKMARKS,						///< toggle tank tread marks on/off
		MSG_META_DEMO_TOGGLE_WATERPLANE,						///< toggle waterplane on/off
		MSG_META_DEMO_TIME_OF_DAY,									///< change time-of-day lighting
		MSG_META_DEMO_TOGGLE_MUSIC,									///< turn background music on/off
		MSG_META_DEMO_MUSIC_NEXT_TRACK,							///< play next track
		MSG_META_DEMO_MUSIC_PREV_TRACK,							///< play prev track
		MSG_META_DEMO_NEXT_OBJECTIVE_MOVIE,					///< play next "Objective" movie
		MSG_META_DEMO_PLAY_OBJECTIVE_MOVIE1,				///< play specific "Objective" movie
		MSG_META_DEMO_PLAY_OBJECTIVE_MOVIE2,				///< play specific "Objective" movie
		MSG_META_DEMO_PLAY_OBJECTIVE_MOVIE3,				///< play specific "Objective" movie
		MSG_META_DEMO_PLAY_OBJECTIVE_MOVIE4,				///< play specific "Objective" movie
		MSG_META_DEMO_PLAY_OBJECTIVE_MOVIE5,				///< play specific "Objective" movie
		MSG_META_DEMO_PLAY_OBJECTIVE_MOVIE6,				///< play specific "Objective" movie
		MSG_META_DEMO_BEGIN_ADJUST_PITCH,						///< enter adjust-pitch mode
		MSG_META_DEMO_END_ADJUST_PITCH,							///< exit adjust-pitch mode
		MSG_META_DEMO_BEGIN_ADJUST_DEFAULTPITCH,		///< TheSuperHackers @feature Enter adjust-default-pitch mode
		MSG_META_DEMO_END_ADJUST_DEFAULTPITCH,			///< TheSuperHackers @feature Exit adjust-default-pitch mode
		MSG_META_DEMO_BEGIN_ADJUST_FOV,							///< enter adjust-FOV mode
		MSG_META_DEMO_END_ADJUST_FOV,								///< exit adjust-FOV mode
		MSG_META_DEMO_LOCK_CAMERA_TO_PLANES,				///< lock camera to airborne thingies
		MSG_META_DEMO_REMOVE_PREREQ,								///< Turn off Prerequisite checks in building legality, works in MULTIPLAYER for all humans
		MSG_META_DEMO_INSTANT_BUILD,								///< All building is with a timer of 1, works in MULTIPLAYER for all humans
		MSG_META_DEMO_FREE_BUILD,										///< All building is for 0 money, works in MULTIPLAYER for all humans
		MSG_META_DEMO_RUNSCRIPT1,										///< run script named "KEY_F1"
		MSG_META_DEMO_RUNSCRIPT2,										///< run script named "KEY_F2"
		MSG_META_DEMO_RUNSCRIPT3,										///< run script named "KEY_F3"
		MSG_META_DEMO_RUNSCRIPT4,										///< run script named "KEY_F4"
		MSG_META_DEMO_RUNSCRIPT5,										///< run script named "KEY_F5"
		MSG_META_DEMO_RUNSCRIPT6,										///< run script named "KEY_F6"
		MSG_META_DEMO_RUNSCRIPT7,										///< run script named "KEY_F7"
		MSG_META_DEMO_RUNSCRIPT8,										///< run script named "KEY_F8"
		MSG_META_DEMO_RUNSCRIPT9,										///< run script named "KEY_F9"
		MSG_META_DEMO_ENSHROUD,											///< re-shroud the world for the local player
		MSG_META_DEMO_DESHROUD,											///< de-shroud the world for the local player
		MSG_META_DEBUG_SHOW_EXTENTS,								///< show object extents
    MSG_META_DEBUG_SHOW_AUDIO_LOCATIONS,	  		///< show audio objects and radii, Added in Zero Hour
    MSG_META_DEBUG_SHOW_HEALTH,									///< show object health
		MSG_META_DEBUG_GIVE_VETERANCY,							///< give a veterancy level to selected objects
		MSG_META_DEBUG_TAKE_VETERANCY,							///< take a veterancy level from selected objects
		MSG_META_DEMO_TOGGLE_AI_DEBUG,							///< show/hide the ai debug stats
		MSG_META_DEMO_TOGGLE_SUPPLY_CENTER_PLACEMENT, ///<start/stop dumping to file all thoughts about placing SupplyCenters, Added in Zero Hour
		MSG_META_DEMO_TOGGLE_CAMERA_DEBUG,					///< show/hide the camera debug stats
		MSG_META_DEMO_TOGGLE_AVI,										///< start capturing video
		MSG_META_DEMO_TOGGLE_BW_VIEW,								///< enable/disable black & white camera mode
		MSG_META_DEMO_TOGGLE_RED_VIEW,							///< enable/disable red tinted view
		MSG_META_DEMO_TOGGLE_GREEN_VIEW,						///< enable/disable green view
		MSG_META_DEMO_TOGGLE_MOTION_BLUR_ZOOM,			///< enable/disable green view
		MSG_META_DEMO_TOGGLE_MILITARY_SUBTITLES,		///< enable/disable military subtitles
		MSG_META_DEMO_ADD_CASH,											///< adds 10000 cash to the player
#ifdef ALLOW_SURRENDER
		MSG_META_DEMO_TEST_SURRENDER,							  ///< Test key to show surrender animation in game.
#endif
		MSG_META_DEMO_TOGGLE_RENDER,								///< toggle rendering on/off
		MSG_META_DEMO_KILL_AREA_SELECTION,					///< (teamID, objectID1, objectID2, ... objectIDN)
		MSG_META_DEMO_CYCLE_LOD_LEVEL,						///< cycles through dynamic game detail levels.

		MSG_META_DEBUG_INCR_ANIM_SKATE_SPEED,				///< for debugging anim skate speeds
		MSG_META_DEBUG_DECR_ANIM_SKATE_SPEED,				///< for debugging anim skate speeds
		MSG_META_DEBUG_CYCLE_EXTENT_TYPE,						///< change extent
		MSG_META_DEBUG_INCREASE_EXTENT_MAJOR,				///< change extent
		MSG_META_DEBUG_INCREASE_EXTENT_MAJOR_BIG,		///< change extent
		MSG_META_DEBUG_DECREASE_EXTENT_MAJOR,				///< change extent
		MSG_META_DEBUG_DECREASE_EXTENT_MAJOR_BIG,		///< change extent
		MSG_META_DEBUG_INCREASE_EXTENT_MINOR,				///< change extent
		MSG_META_DEBUG_INCREASE_EXTENT_MINOR_BIG,		///< change extent
		MSG_META_DEBUG_DECREASE_EXTENT_MINOR,				///< change extent
		MSG_META_DEBUG_DECREASE_EXTENT_MINOR_BIG,		///< change extent
		MSG_META_DEBUG_INCREASE_EXTENT_HEIGHT,			///< change extent
		MSG_META_DEBUG_INCREASE_EXTENT_HEIGHT_BIG,	///< change extent
		MSG_META_DEBUG_DECREASE_EXTENT_HEIGHT,			///< change extent
		MSG_META_DEBUG_DECREASE_EXTENT_HEIGHT_BIG,	///< change extent
		MSG_META_DEBUG_VTUNE_ON,										///< turn on/off Vtune
		MSG_META_DEBUG_VTUNE_OFF,										///< turn on/off Vtune
		MSG_META_DEBUG_TOGGLE_FEATHER_WATER, 			///< toggle lorenzen's feather water

		MSG_META_DEBUG_DUMP_ASSETS,						///< dumps currently used map assets to a file.

		MSG_NO_DRAW,																///< show/hide all objects to test Drawing code
		MSG_META_DEMO_TOGGLE_METRICS,								///< Toggle the metrics on/off
		MSG_META_DEMO_TOGGLE_PROJECTILEDEBUG,				///< Toggles bezier curves on projectiles on/off
		MSG_META_DEMO_TOGGLE_VISIONDEBUG,						///< Toggles vision debug circles on/off
		MSG_META_DEMO_TOGGLE_THREATDEBUG,						///< Toggle the threat debugger on/off
		MSG_META_DEMO_TOGGLE_CASHMAPDEBUG,					///< Toggle the cash map debugger on/off
		MSG_META_DEMO_TOGGLE_GRAPHICALFRAMERATEBAR,	///< Toggle the graphical framerate bar on/off
		MSG_META_DEMO_GIVE_ALL_SCIENCES,						///< grant all available sciences, works in MULTIPLAYER for all humans
		MSG_META_DEMO_GIVE_RANKLEVEL,								///< up one rank level
		MSG_META_DEMO_TAKE_RANKLEVEL,								///< up one rank level
		MSG_META_DEMO_GIVE_SCIENCEPURCHASEPOINTS,		///< give a science purchase point (but no rank change)
		MSG_META_DEBUG_TOGGLE_NETWORK,							///< toggle between having and not having network traffic.
		MSG_META_DEBUG_DUMP_PLAYER_OBJECTS,					///< Dump numbers of objects owned by each player to the script debug window
		MSG_META_DEBUG_DUMP_ALL_PLAYER_OBJECTS,			///< Dump numbers of objects owned by each player to the script debug window, and additional object info
		MSG_META_DEBUG_OBJECT_ID_PERFORMANCE,				///< Run a mess of ObjectID lookups to see performance, Added in Zero Hour
		MSG_META_DEBUG_DRAWABLE_ID_PERFORMANCE,			///< Run a mess of DrawableID lookups to see performance, Added in Zero Hour
		MSG_META_DEBUG_SLEEPY_UPDATE_PERFORMANCE,		///< Peek at the size of the sleepy update vector, Added in Zero Hour

		MSG_META_DEBUG_WIN,													///< Instant Win
		MSG_META_DEMO_TOGGLE_DEBUG_STATS,						///< show/hide the debug stats
		/// @todo END section to REMOVE (not disable) for release
#endif // defined(RTS_DEBUG)

#if defined(RTS_DEBUG)
		MSG_META_DEMO_TOGGLE_AUDIODEBUG,						///< show/hide the audio debug info
#endif//defined(RTS_DEBUG)
#ifdef DUMP_PERF_STATS
		MSG_META_DEMO_PERFORM_STATISTICAL_DUMP,			///< dump performance stats for this frame to StatisticsDump.txt
#endif//DUMP_PERF_STATS

		MSG_META_PLACE_BEACON,
		MSG_META_REMOVE_BEACON,

		MSG_END_META_MESSAGES,											///< Marker to delineate "meta" messages

		MSG_MOUSEOVER_DRAWABLE_HINT,								///< (drawableid) the given drawable is under the mouse, regardless of button states
		MSG_MOUSEOVER_LOCATION_HINT,								///< (location) The cursor is not over a drawable, but is here.
		MSG_VALID_GUICOMMAND_HINT,									///< posted when the gui command is valid if the user clicked to execute it.
		MSG_INVALID_GUICOMMAND_HINT,								///< posted when the gui command is not valid if the user were to click to attempt to execute it.
		MSG_BEGIN_AREA_SELECTION_HINT,							///< (pixelRegion) rectangular selection area under construction, not confirmed
		MSG_END_AREA_SELECTION_HINT,                ///< (pixelRegion) rectangular selection area finish construction

		//Command hints
		MSG_DO_ATTACK_OBJECT_HINT,									///< (victim objectID) If clicked, an attack would be ordered, "Current Selection" is assumed
		MSG_IMPOSSIBLE_ATTACK_HINT,									///< we can't do anything, and target is out of range.
		MSG_DO_FORCE_ATTACK_OBJECT_HINT,						///< (victim objectID) If clicked, an attack would be ordered, "Current Selection" is assumed
		MSG_DO_FORCE_ATTACK_GROUND_HINT,						///< (victim objectID) If clicked, an attack would be ordered, "Current Selection" is assumed
		MSG_GET_REPAIRED_HINT,											///< If clicked, selected unit will go get repaired at clicked object
		MSG_GET_HEALED_HINT,												///< If clicked, selected unit will go get healed at clicked object
		MSG_DO_REPAIR_HINT,													///< if clicked, dozer will go repair the clicked target
		MSG_RESUME_CONSTRUCTION_HINT,								///< if clicked, dozer will go construct a partially constructed building
		MSG_ENTER_HINT,															///< if clicked, selected unit(s) will attempt to enter clicked object
		MSG_DOCK_HINT,															///< If clicked, selected unit(s) will dock
		MSG_DO_MOVETO_HINT,													///< (location) If clicked, a move would be ordered, "Current Selection" is assumed
		MSG_DO_ATTACKMOVETO_HINT,										///< (location) If clicked, a move would be ordered, "Current Selection" is assumed
		MSG_ADD_WAYPOINT_HINT,											///< (location) If clicked, a waypoint will be added for currently selected units.
		//Context command hints
		MSG_HIJACK_HINT,								///< if clicked, selected unit(s) will attempt to take over vehicle.
		MSG_SABOTAGE_HINT, // Added in Zero Hour
		MSG_FIREBOMB_HINT,								///< throw a molotov cocktail
		MSG_CONVERT_TO_CARBOMB_HINT,								///< if clicked, selected unit(s) will attempt to convert clicked object into a carbomb.
		MSG_CAPTUREBUILDING_HINT,
#ifdef ALLOW_SURRENDER
		MSG_PICK_UP_PRISONER_HINT,
#endif
		MSG_SNIPE_VEHICLE_HINT,
		MSG_DEFECTOR_HINT,
		MSG_SET_RALLY_POINT_HINT,										///< (location) if clicked, we will place a rally point here.
		MSG_DO_SPECIAL_POWER_OVERRIDE_DESTINATION_HINT,
		MSG_DO_SALVAGE_HINT,
		MSG_DO_INVALID_HINT,												///< Display invalid cursor because no real command can be done in this context.
		MSG_DO_ATTACK_OBJECT_AFTER_MOVING_HINT,
		MSG_HACK_HINT,

//*********************************************************************************************************
//*********************************************************************************************************
		/*
			Note that we start this at a fixed value, so that (regardless of ifdefs upstream from us) we have the same
			numeric values for all enums that are saved in replay files. This helps improve compatibility between
			replay files created in internal vs. release builds (though they will still generate sync errors in
			some situations due to other reasons, e.g., DEBUG_LOG using the FPU and thus changing its state).

			there should be NO IFDEFS OF ANY KIND FROM HERE UNTIL MSG_END_NETWORK_MESSAGES.

			there should be NO IFDEFS OF ANY KIND FROM HERE UNTIL MSG_END_NETWORK_MESSAGES.

			there should be NO IFDEFS OF ANY KIND FROM HERE UNTIL MSG_END_NETWORK_MESSAGES.

		*/
		MSG_BEGIN_NETWORK_MESSAGES = 1000,					///< MARKER TO DELINEATE MESSAGES THAT GO OVER THE NETWORK
//*********************************************************************************************************
		MSG_CREATE_SELECTED_GROUP,									/**< (Bool createNewGroup, objectID1, objectID2, ... objectIDN)
																									* The selected team is created/augmented with the given team members
																									*/
		MSG_CREATE_SELECTED_GROUP_NO_SOUND,					/**< (Bool createNewGroup, objectID1, objectID2, ... objectIDN)
																									* The selected team is created/augmented with the given team members.
																									* Do not play their selection sounds.
																									*/
		MSG_DESTROY_SELECTED_GROUP,									///< deselect currently selected objects (which can be none)
		MSG_REMOVE_FROM_SELECTED_GROUP,							/**< (objectID1, objectID2, ... objectIDN)
																									* Remove these units from the selected group. (N should almost always be 1)
																									*/
		MSG_SELECTED_GROUP_COMMAND,									///< (teamID) the NEXT COMMAND acts upon all members of this team
		MSG_CREATE_TEAM0,														///< Creates a hotkey squad from the currently selected units.
		MSG_CREATE_TEAM1,
		MSG_CREATE_TEAM2,
		MSG_CREATE_TEAM3,
		MSG_CREATE_TEAM4,
		MSG_CREATE_TEAM5,
		MSG_CREATE_TEAM6,
		MSG_CREATE_TEAM7,
		MSG_CREATE_TEAM8,
		MSG_CREATE_TEAM9,
		MSG_SELECT_TEAM0,														///< Set a hotkey squad to be the currently selected units.
		MSG_SELECT_TEAM1,														///< Set a hotkey squad to be the currently selected units.
		MSG_SELECT_TEAM2,														///< Set a hotkey squad to be the currently selected units.
		MSG_SELECT_TEAM3,														///< Set a hotkey squad to be the currently selected units.
		MSG_SELECT_TEAM4,														///< Set a hotkey squad to be the currently selected units.
		MSG_SELECT_TEAM5,														///< Set a hotkey squad to be the currently selected units.
		MSG_SELECT_TEAM6,														///< Set a hotkey squad to be the currently selected units.
		MSG_SELECT_TEAM7,														///< Set a hotkey squad to be the currently selected units.
		MSG_SELECT_TEAM8,														///< Set a hotkey squad to be the currently selected units.
		MSG_SELECT_TEAM9,														///< Set a hotkey squad to be the currently selected units.
		MSG_ADD_TEAM0,															///< Add hotkey squad to the currently selected units.
		MSG_ADD_TEAM1,															///< Add hotkey squad to the currently selected units.
		MSG_ADD_TEAM2,															///< Add hotkey squad to the currently selected units.
		MSG_ADD_TEAM3,															///< Add hotkey squad to the currently selected units.
		MSG_ADD_TEAM4,															///< Add hotkey squad to the currently selected units.
		MSG_ADD_TEAM5,															///< Add hotkey squad to the currently selected units.
		MSG_ADD_TEAM6,															///< Add hotkey squad to the currently selected units.
		MSG_ADD_TEAM7,															///< Add hotkey squad to the currently selected units.
		MSG_ADD_TEAM8,															///< Add hotkey squad to the currently selected units.
		MSG_ADD_TEAM9,															///< Add hotkey squad to the currently selected units.
		MSG_DO_ATTACKSQUAD,													///< (numObjects) (numObjects * objectID)
		MSG_DO_WEAPON,															///< fire specific weapon
		MSG_DO_WEAPON_AT_LOCATION,									///< fire a specific weapon at location
		MSG_DO_WEAPON_AT_OBJECT,										///< fire a specific weapon at a target object
		MSG_DO_SPECIAL_POWER,												///< do special
		MSG_DO_SPECIAL_POWER_AT_LOCATION,						///< do special with target location
		MSG_DO_SPECIAL_POWER_AT_OBJECT,							///< do special at with target object
		MSG_SET_RALLY_POINT,												///< (objectID, location)
		MSG_PURCHASE_SCIENCE,												///< purchase a science
		MSG_QUEUE_UPGRADE,													///< queue the "research" of an upgrade
		MSG_CANCEL_UPGRADE,													///< cancel the "research" of an upgrade
		MSG_QUEUE_UNIT_CREATE,											///< clicked on a button to queue the production of a unit
		MSG_CANCEL_UNIT_CREATE,											///< clicked on UI button to cancel production of a unit
		MSG_DOZER_CONSTRUCT,												/**< building things requires clicking on a dozer
																										 selecting what to build, selecting where to
																										 build it ... this construct message will
																										 start the actual build process */
		MSG_DOZER_CONSTRUCT_LINE,										///< Like MSG_CONSTRUCT, but for build processes that occur in a line (like walls)
		MSG_DOZER_CANCEL_CONSTRUCT,									///< cancel construction of a building
		MSG_SELL,																		///< sell a structure
		MSG_EXIT,																		///< WE want to exit from whatever WE are inside of
		MSG_EVACUATE,																///< Dump out all of OUR contained objects
		MSG_EXECUTE_RAILED_TRANSPORT,								///< Execute railed transport sequence
		MSG_COMBATDROP_AT_LOCATION,									///< dump out all rappellers
		MSG_COMBATDROP_AT_OBJECT,										///< dump out all rappellers
		MSG_AREA_SELECTION_DEPRECATED,              ///< TheSuperHackers @tweak former MSG_AREA_SELECTION is deprecated as network message.
		MSG_DO_ATTACK_OBJECT,												///< (objectID, victim objectID)
		MSG_DO_FORCE_ATTACK_OBJECT,									///< force attack the given object if picked
		MSG_DO_FORCE_ATTACK_GROUND,									///< (locationID) bombard the given location if picked
		MSG_GET_REPAIRED,														///< selected unit will go get repaired at clicked object
		MSG_GET_HEALED,															///< selected unit will go get healed at clicked object
		MSG_DO_REPAIR,															///< dozer will go repair the clicked target
		MSG_RESUME_CONSTRUCTION,										///< resume construction on a structure
		MSG_ENTER,																	///< Enter object
		MSG_DOCK,																		///< Dock with this object
		MSG_DO_MOVETO,															///< location
		MSG_DO_ATTACKMOVETO,												///< location
		MSG_DO_FORCEMOVETO,													///< location
		MSG_ADD_WAYPOINT,														///< location
		MSG_DO_GUARD_POSITION,											///< Guard with the currently selected group
		MSG_DO_GUARD_OBJECT,												///< Guard with the currently selected group
		MSG_DO_STOP,																///< Stop with the currently selected group
		MSG_DO_SCATTER,															///< Scatter the currently selected group
		MSG_INTERNET_HACK,													///< Begin a persistent internet hack (free slow income)
		MSG_DO_CHEER,																///< Orders selected units to play cheer animation (if possible)
		MSG_TOGGLE_OVERCHARGE,											///< Toggle overcharge status of a power plant
		MSG_SWITCH_WEAPONS,													///< Switches which weapon slot to use for an object
		MSG_CONVERT_TO_CARBOMB,
		MSG_CAPTUREBUILDING,
		MSG_DISABLEVEHICLE_HACK,
		MSG_STEALCASH_HACK,
		MSG_DISABLEBUILDING_HACK,
		MSG_SNIPE_VEHICLE,
		MSG_DO_SPECIAL_POWER_OVERRIDE_DESTINATION,
		MSG_DO_SALVAGE,
		MSG_CLEAR_INGAME_POPUP_MESSAGE,							///< If we want a replay to work with the popup messages then we need it to be passed
		MSG_PLACE_BEACON,
		MSG_REMOVE_BEACON,
		MSG_SET_BEACON_TEXT,
		MSG_SET_REPLAY_CAMERA,											///< Track camera pos for replays
		MSG_SELF_DESTRUCT,													///< Destroys a player's units (for copy protection or to quit to observer)
		MSG_CREATE_FORMATION,												///< Creates a formation.
		MSG_LOGIC_CRC,															///< CRC from the logic passed around in a network game
		MSG_SET_MINE_CLEARING_DETAIL,								///< Order selected unit(s) to clear mines
		MSG_ENABLE_RETALIATION_MODE,								///< Turn retaliation mode on or off. Added in Zero Hour

		MSG_BEGIN_DEBUG_NETWORK_MESSAGES = 1900,		///< network messages that exist only in debug/internal builds. all grouped separately.

#if defined(RTS_DEBUG) || defined(_ALLOW_DEBUG_CHEATS_IN_RELEASE)
		// all debug/internal-only messages must go here.
		MSG_DEBUG_KILL_SELECTION,
		MSG_DEBUG_HURT_OBJECT,
		MSG_DEBUG_KILL_OBJECT,
#endif



//*********************************************************************************************************
		MSG_END_NETWORK_MESSAGES = 1999,						///< MARKER TO DELINEATE MESSAGES THAT GO OVER THE NETWORK
//*********************************************************************************************************
//*********************************************************************************************************


		// Server to Client messages
		MSG_TIMESTAMP,															///< The current frame number
		MSG_OBJECT_CREATED,													///< (objectID, Int type) Cause a drawable to be created and bound to this ID
		MSG_OBJECT_DESTROYED,												///< (objectID) Free bound drawable
		MSG_OBJECT_POSITION,												///< (objectID, location) New position of object
		MSG_OBJECT_ORIENTATION,											///< (objectID, angle) New orientation of object
		MSG_OBJECT_JOINED_TEAM,											///< (objectID) New team affiliation of object

		MSG_COUNT
	};

	GameMessage( Type type );

	GameMessage *next() { return m_next; }             ///< Return next message in the stream
	const GameMessage *next() const { return m_next; } ///< Return next message in the stream
	GameMessage *prev() { return m_prev; }             ///< Return prev message in the stream
	const GameMessage *prev() const { return m_prev; } ///< Return prev message in the stream

	Type getType() const { return m_type; }					///< Return the message type

	const char *getCommandAsString() const; ///< returns a string representation of the command type.
	static const char *getCommandTypeAsString(GameMessage::Type t);

	Int getPlayerIndex() const { return m_playerIndex; }		///< Return the originating player

	// access methods for GameMessageArgumentType enum
	void appendIntegerArgument( Int arg );
	void appendRealArgument( Real arg );
	void appendBooleanArgument( Bool arg );
	void appendDrawableIDArgument( DrawableID arg );
	void appendObjectIDArgument( ObjectID arg );
	void appendTeamIDArgument( UnsignedInt arg );
	void appendLocationArgument( const Coord3D& arg );
	void appendPixelArgument( const ICoord2D& arg );
	void appendPixelRegionArgument( const IRegion2D& arg );
	void appendWideCharArgument( const WideChar& arg );

	void appendTimestampArgument( UnsignedInt arg );

	/**
	 * Return the given argument union.
	 */
	UnsignedByte getArgumentCount() const { return static_cast<UnsignedByte>(m_argList.size()); }
	const GameMessageArgumentType *getArgument( Int argIndex ) const;
	GameMessageArgumentDataType getArgumentDataType( Int argIndex ) const;

	void friend_setNext(GameMessage* m) { m_next = m; }
	void friend_setPrev(GameMessage* m) { m_prev = m; }
	void friend_setList(GameMessageList* m) { m_list = m; }
	void friend_setPlayerIndex(Int i) { m_playerIndex = i; }

private:
	// friend classes are bad. don't use them. no, really.
	// if for no other reason than the fact that they subvert MemoryPoolObject. (srj)

	GameMessage *m_next, *m_prev;								///< List links for message list
	GameMessageList *m_list;										///< The list this message is on

	Type m_type;										///< The type of this message

	Int m_playerIndex;													///< The Player who issued the command

	std::vector<GameMessageArgument*> m_argList;						///< This message's arguments

	/// allocate a new argument, add it to list, return pointer to its data
	GameMessageArgument *allocArg();

};


/**
 * The GameMessageList class encapsulates the manipulation of lists of GameMessages.
 * Both MessageStream and CommandList derive from this class.
 */
class GameMessageList : public SubsystemInterface
{

public:

	GameMessageList();
	virtual ~GameMessageList() override;

	virtual void init() override { };			///< Initialize system
	virtual void reset() override { };			///< Reset system
	virtual void update() override { };		///< Update system

	GameMessage *getFirstMessage() { return m_firstMessage; }	///< Return the first message

	virtual void appendMessage( GameMessage *msg );			///< Add message to end of the list
	virtual void insertMessage( GameMessage *msg, GameMessage *messageToInsertAfter );	// Insert message after messageToInsertAfter.
	virtual void removeMessage( GameMessage *msg );			///< Remove message from the list
	virtual Bool containsMessageOfType( GameMessage::Type type );	///< Return true if a message of type is in the message stream



protected:
	GameMessage *m_firstMessage;								///< The first message on the list
	GameMessage *m_lastMessage;									///< The last message on the list
};

/**
	What to do with a GameMessage after a translator has handled it.
	Use a custom enum (rather than a Bool) to make the code more obvious.
*/
enum GameMessageDisposition CPP_11(: Int)
{
	KEEP_MESSAGE,			///< continue processing this message thru other translators
	DESTROY_MESSAGE		///< destroy this message immediately and don't hand it to any other translators
};

class GameMessageTranslator
{
public:
	virtual GameMessageDisposition translateGameMessage(const GameMessage *msg) = 0;
	virtual ~GameMessageTranslator() { }
};

/**
 * A MessageStream contains an ordered list of messages which can have one or more
 * prioritized message handler functions ("translators") attached to it.
 */
class MessageStream : public GameMessageList
{

public:

	MessageStream();
	virtual ~MessageStream() override;

	// Inherited Methods ----------------------------------------------------------------------------
	virtual void init() override;
	virtual void reset() override;
	virtual void update() override;

	virtual GameMessage *appendMessage( GameMessage::Type type );		///< Append a message to the end of the stream
	virtual GameMessage *insertMessage( GameMessage::Type type, GameMessage *messageToInsertAfter );	// Insert message after messageToInsertAfter.
	virtual Bool isReadyForMessages() const;											///< Return true if a local player is present and ready to accept messages

	// Methods NOT Inherited ------------------------------------------------------------------------
	void propagateMessages();													///< Propagate messages through attached translators

	/**
		Attach a translator function to the stream at a priority value. Lower priorities are executed first.
		Note that MessageStream assumes ownership of the translator, and is responsible for freeing it!
	*/
	TranslatorID attachTranslator( GameMessageTranslator *translator, UnsignedInt priority);
	GameMessageTranslator* findTranslator( TranslatorID id );
	void removeTranslator( TranslatorID );				///< Remove a previously attached translator

protected:

	Bool isRedundantNetworkMessage(const GameMessage *msg) const;

	struct TranslatorData
	{
		TranslatorData *m_next, *m_prev;						///< List links for list of translators
		TranslatorID m_id;													///< The unique ID of this translator
		GameMessageTranslator *m_translator;					///< The translator's interface function
		UnsignedInt m_priority;											///< The priority level of this translator

		TranslatorData() : m_next(0), m_prev(0), m_id(0), m_translator(0), m_priority(0)
		{
		}

		~TranslatorData()
		{
			delete m_translator;
		}
	};

	TranslatorData *m_firstTranslator;						///< List of registered translators, in order of priority
	TranslatorData *m_lastTranslator;
	TranslatorID m_nextTranslatorID;							///< For issuing unique translator ID's

};


/**
 * The CommandList is the final set of messages that have made their way through
 * all of the Translators of the MessageStream, and reached the end.
 * This set of commands will be executed by the GameLogic on its next iteration.
 */
class CommandList : public GameMessageList
{
public:
	CommandList();
	virtual ~CommandList() override;

	virtual void init() override;			///< Init command list
	virtual void reset() override;			///< Destroy all messages and reset list to empty
	virtual void update() override;		///< Update hook

	void appendMessageList( GameMessage *list );			///< Adds messages to the end of the command list

protected:

	void destroyAllMessages();		///< The meat of a reset and a shutdown

};

//
// The message stream that filters client input into game commands
//
extern MessageStream *TheMessageStream;

//
// The list of commands awaiting execution by the GameLogic
//
extern CommandList *TheCommandList;


//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
// Functions used in multiple translators should go here.
//

/**
 * Given an "anchor" point and the current mouse position (dest),
 * construct a valid 2D bounding region.
 */
extern void buildRegion( const ICoord2D *anchor, const ICoord2D *dest, IRegion2D *region );
```

### `Core/GameEngine/Source/Common/MessageStream.cpp`
```cpp
// MessageStream.cpp
// Implementation of the message stream
// Author: Michael S. Booth, February 2001

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/MessageStream.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/Recorder.h"

#include "GameClient/InGameUI.h"
#include "GameLogic/GameLogic.h"

/// The singleton message stream for messages going to TheGameLogic
MessageStream *TheMessageStream = nullptr;
CommandList *TheCommandList = nullptr;






//------------------------------------------------------------------------------------------------
// GameMessage
//

/**
 * Constructor
 */
GameMessage::GameMessage( GameMessage::Type type )
{
	m_playerIndex = ThePlayerList->getLocalPlayer()->getPlayerIndex();
	m_type = type;
	m_list = nullptr;
}


/**
 * Destructor
 */
GameMessage::~GameMessage()
{
	// free all arguments
	for( size_t i = 0; i < m_argList.size(); ++i ) {
		deleteInstance(m_argList[i]);
	}

	// detach message from list
	if (m_list)
		m_list->removeMessage( this );
}

/**
 * Return the given argument union.
 */
const GameMessageArgumentType *GameMessage::getArgument( Int argIndex ) const
{
	if (static_cast<size_t>(argIndex) < m_argList.size())
		return &m_argList[argIndex]->m_data;

	DEBUG_CRASH(("argument not found"));
	static const GameMessageArgumentType zero = { 0 };
	return &zero;
}

/**
 * Return the given argument data type
 */
GameMessageArgumentDataType GameMessage::getArgumentDataType( Int argIndex ) const
{
	if (static_cast<size_t>(argIndex) < m_argList.size())
		return m_argList[argIndex]->m_type;

	return ARGUMENTDATATYPE_UNKNOWN;
}

/**
 * Allocate a new argument, add it to the argument list, and increment the total arg count
 */
GameMessageArgument *GameMessage::allocArg()
{
	// allocate a new argument
	GameMessageArgument *arg = newInstance(GameMessageArgument);
	m_argList.push_back(arg);

	DEBUG_ASSERTCRASH(
		m_argList.size() <= 255,
		("If a GameMessage needs more than 255 arguments, it needs to be split up into multiple GameMessage's.")
	); 
	return arg;
}

/**
 * Append an integer argument
 */
void GameMessage::appendIntegerArgument( Int arg )
{
	GameMessageArgument *a = allocArg();
	a->m_data.integer = arg;
	a->m_type = ARGUMENTDATATYPE_INTEGER;
}

void GameMessage::appendRealArgument( Real arg )
{
	GameMessageArgument *a = allocArg();
	a->m_data.real = arg;
	a->m_type = ARGUMENTDATATYPE_REAL;
}

void GameMessage::appendBooleanArgument( Bool arg )
{
	GameMessageArgument *a = allocArg();
	a->m_data.boolean = arg;
	a->m_type = ARGUMENTDATATYPE_BOOLEAN;
}

void GameMessage::appendObjectIDArgument( ObjectID arg )
{
	GameMessageArgument *a = allocArg();
	a->m_data.objectID = arg;
	a->m_type = ARGUMENTDATATYPE_OBJECTID;
}

void GameMessage::appendDrawableIDArgument( DrawableID arg )
{
	GameMessageArgument *a = allocArg();
	a->m_data.drawableID = arg;
	a->m_type = ARGUMENTDATATYPE_DRAWABLEID;
}

void GameMessage::appendTeamIDArgument( UnsignedInt arg )
{
	GameMessageArgument *a = allocArg();
	a->m_data.teamID = arg;
	a->m_type = ARGUMENTDATATYPE_TEAMID;
}

void GameMessage::appendLocationArgument( const Coord3D& arg )
{
	GameMessageArgument *a = allocArg();
	a->m_data.location = arg;
	a->m_type = ARGUMENTDATATYPE_LOCATION;
}

void GameMessage::appendPixelArgument( const ICoord2D& arg )
{
	GameMessageArgument *a = allocArg();
	a->m_data.pixel = arg;
	a->m_type = ARGUMENTDATATYPE_PIXEL;
}

void GameMessage::appendPixelRegionArgument( const IRegion2D& arg )
{
	GameMessageArgument *a = allocArg();
	a->m_data.pixelRegion = arg;
	a->m_type = ARGUMENTDATATYPE_PIXELREGION;
}

void GameMessage::appendTimestampArgument( UnsignedInt arg )
{
	GameMessageArgument *a = allocArg();
	a->m_data.timestamp = arg;
	a->m_type = ARGUMENTDATATYPE_TIMESTAMP;
}

void GameMessage::appendWideCharArgument( const WideChar& arg )
{
	GameMessageArgument *a = allocArg();
	a->m_data.wChar = arg;
	a->m_type = ARGUMENTDATATYPE_WIDECHAR;
}

const char *GameMessage::getCommandAsString() const
{
	return getCommandTypeAsString(m_type);
}

const char *GameMessage::getCommandTypeAsString(GameMessage::Type t)
{
#define CASE_LABEL(x) case x: return #x;

	switch (t) {
	default: return (t >= GameMessage::MSG_COUNT) ? "Invalid command" : "UnknownMessage";

	CASE_LABEL(MSG_INVALID)
	CASE_LABEL(MSG_FRAME_TICK)
	CASE_LABEL(MSG_RAW_MOUSE_BEGIN)
	CASE_LABEL(MSG_RAW_MOUSE_POSITION)
	CASE_LABEL(MSG_RAW_MOUSE_LEFT_BUTTON_DOWN)
	CASE_LABEL(MSG_RAW_MOUSE_LEFT_DOUBLE_CLICK)
	CASE_LABEL(MSG_RAW_MOUSE_LEFT_BUTTON_UP)
	CASE_LABEL(MSG_RAW_MOUSE_LEFT_CLICK)
	CASE_LABEL(MSG_RAW_MOUSE_LEFT_DRAG)
	CASE_LABEL(MSG_RAW_MOUSE_MIDDLE_BUTTON_DOWN)
	CASE_LABEL(MSG_RAW_MOUSE_MIDDLE_DOUBLE_CLICK)
	CASE_LABEL(MSG_RAW_MOUSE_MIDDLE_BUTTON_UP)
	CASE_LABEL(MSG_RAW_MOUSE_MIDDLE_DRAG)
	CASE_LABEL(MSG_RAW_MOUSE_RIGHT_BUTTON_DOWN)
	CASE_LABEL(MSG_RAW_MOUSE_RIGHT_DOUBLE_CLICK)
	CASE_LABEL(MSG_RAW_MOUSE_RIGHT_BUTTON_UP)
	CASE_LABEL(MSG_RAW_MOUSE_RIGHT_DRAG)
	CASE_LABEL(MSG_RAW_MOUSE_WHEEL)
	CASE_LABEL(MSG_RAW_MOUSE_END)
	CASE_LABEL(MSG_RAW_KEY_DOWN)
	CASE_LABEL(MSG_RAW_KEY_UP)
	CASE_LABEL(MSG_MOUSE_LEFT_CLICK)
	CASE_LABEL(MSG_MOUSE_LEFT_DOUBLE_CLICK)
	CASE_LABEL(MSG_MOUSE_MIDDLE_CLICK)
	CASE_LABEL(MSG_MOUSE_MIDDLE_DOUBLE_CLICK)
	CASE_LABEL(MSG_MOUSE_RIGHT_CLICK)
	CASE_LABEL(MSG_MOUSE_RIGHT_DOUBLE_CLICK)
	CASE_LABEL(MSG_CLEAR_GAME_DATA)
	CASE_LABEL(MSG_NEW_GAME)
	CASE_LABEL(MSG_LOGIC_CRC_PLAYBACK)
	CASE_LABEL(MSG_BEGIN_META_MESSAGES)
	CASE_LABEL(MSG_META_SAVE_VIEW1)
	CASE_LABEL(MSG_META_SAVE_VIEW2)
	CASE_LABEL(MSG_META_SAVE_VIEW3)
	CASE_LABEL(MSG_META_SAVE_VIEW4)
	CASE_LABEL(MSG_META_SAVE_VIEW5)
	CASE_LABEL(MSG_META_SAVE_VIEW6)
	CASE_LABEL(MSG_META_SAVE_VIEW7)
	CASE_LABEL(MSG_META_SAVE_VIEW8)
	CASE_LABEL(MSG_META_VIEW_VIEW1)
	CASE_LABEL(MSG_META_VIEW_VIEW2)
	CASE_LABEL(MSG_META_VIEW_VIEW3)
	CASE_LABEL(MSG_META_VIEW_VIEW4)
	CASE_LABEL(MSG_META_VIEW_VIEW5)
	CASE_LABEL(MSG_META_VIEW_VIEW6)
	CASE_LABEL(MSG_META_VIEW_VIEW7)
	CASE_LABEL(MSG_META_VIEW_VIEW8)
	CASE_LABEL(MSG_META_CREATE_TEAM0)
	CASE_LABEL(MSG_META_CREATE_TEAM1)
	CASE_LABEL(MSG_META_CREATE_TEAM2)
	CASE_LABEL(MSG_META_CREATE_TEAM3)
	CASE_LABEL(MSG_META_CREATE_TEAM4)
	CASE_LABEL(MSG_META_CREATE_TEAM5)
	CASE_LABEL(MSG_META_CREATE_TEAM6)
	CASE_LABEL(MSG_META_CREATE_TEAM7)
	CASE_LABEL(MSG_META_CREATE_TEAM8)
	CASE_LABEL(MSG_META_CREATE_TEAM9)
	CASE_LABEL(MSG_META_SELECT_TEAM0)
	CASE_LABEL(MSG_META_SELECT_TEAM1)
	CASE_LABEL(MSG_META_SELECT_TEAM2)
	CASE_LABEL(MSG_META_SELECT_TEAM3)
	CASE_LABEL(MSG_META_SELECT_TEAM4)
	CASE_LABEL(MSG_META_SELECT_TEAM5)
	CASE_LABEL(MSG_META_SELECT_TEAM6)
	CASE_LABEL(MSG_META_SELECT_TEAM7)
	CASE_LABEL(MSG_META_SELECT_TEAM8)
	CASE_LABEL(MSG_META_SELECT_TEAM9)
	CASE_LABEL(MSG_META_ADD_TEAM0)
	CASE_LABEL(MSG_META_ADD_TEAM1)
	CASE_LABEL(MSG_META_ADD_TEAM2)
	CASE_LABEL(MSG_META_ADD_TEAM3)
	CASE_LABEL(MSG_META_ADD_TEAM4)
	CASE_LABEL(MSG_META_ADD_TEAM5)
	CASE_LABEL(MSG_META_ADD_TEAM6)
	CASE_LABEL(MSG_META_ADD_TEAM7)
	CASE_LABEL(MSG_META_ADD_TEAM8)
	CASE_LABEL(MSG_META_ADD_TEAM9)
	CASE_LABEL(MSG_META_VIEW_TEAM0)
	CASE_LABEL(MSG_META_VIEW_TEAM1)
	CASE_LABEL(MSG_META_VIEW_TEAM2)
	CASE_LABEL(MSG_META_VIEW_TEAM3)
	CASE_LABEL(MSG_META_VIEW_TEAM4)
	CASE_LABEL(MSG_META_VIEW_TEAM5)
	CASE_LABEL(MSG_META_VIEW_TEAM6)
	CASE_LABEL(MSG_META_VIEW_TEAM7)
	CASE_LABEL(MSG_META_VIEW_TEAM8)
	CASE_LABEL(MSG_META_VIEW_TEAM9)
	CASE_LABEL(MSG_META_SELECT_MATCHING_UNITS)
	CASE_LABEL(MSG_META_SELECT_NEXT_UNIT)
	CASE_LABEL(MSG_META_SELECT_PREV_UNIT)
	CASE_LABEL(MSG_META_SELECT_NEXT_WORKER)
	CASE_LABEL(MSG_META_SELECT_PREV_WORKER)
	CASE_LABEL(MSG_META_SELECT_NEXT_IDLE_WORKER)
	CASE_LABEL(MSG_META_VIEW_COMMAND_CENTER)
	CASE_LABEL(MSG_META_VIEW_LAST_RADAR_EVENT)
	CASE_LABEL(MSG_META_SELECT_HERO)
	CASE_LABEL(MSG_META_SELECT_ALL)
	CASE_LABEL(MSG_META_SELECT_ALL_AIRCRAFT)
	CASE_LABEL(MSG_META_SCATTER)
	CASE_LABEL(MSG_META_STOP)
	CASE_LABEL(MSG_META_DEPLOY)
	CASE_LABEL(MSG_META_CREATE_FORMATION)
	CASE_LABEL(MSG_META_FOLLOW)
	CASE_LABEL(MSG_META_CHAT_PLAYERS)
	CASE_LABEL(MSG_META_CHAT_ALLIES)
	CASE_LABEL(MSG_META_CHAT_EVERYONE)
	CASE_LABEL(MSG_META_DIPLOMACY)
	CASE_LABEL(MSG_META_OPTIONS)

#if defined(RTS_DEBUG)
	CASE_LABEL(MSG_META_HELP)
#endif

	CASE_LABEL(MSG_META_INCREASE_MAX_RENDER_FPS)
	CASE_LABEL(MSG_META_DECREASE_MAX_RENDER_FPS)
	CASE_LABEL(MSG_META_INCREASE_LOGIC_TIME_SCALE)
	CASE_LABEL(MSG_META_DECREASE_LOGIC_TIME_SCALE)
	CASE_LABEL(MSG_META_TOGGLE_LOWER_DETAILS)
	CASE_LABEL(MSG_META_TOGGLE_CONTROL_BAR)
	CASE_LABEL(MSG_META_TOGGLE_PLAYER_OBSERVER)
	CASE_LABEL(MSG_META_BEGIN_PATH_BUILD)
	CASE_LABEL(MSG_META_END_PATH_BUILD)
	CASE_LABEL(MSG_META_BEGIN_FORCEATTACK)
	CASE_LABEL(MSG_META_END_FORCEATTACK)
	CASE_LABEL(MSG_META_BEGIN_FORCEMOVE)
	CASE_LABEL(MSG_META_END_FORCEMOVE)
	CASE_LABEL(MSG_META_BEGIN_WAYPOINTS)
	CASE_LABEL(MSG_META_END_WAYPOINTS)
	CASE_LABEL(MSG_META_BEGIN_PREFER_SELECTION)
	CASE_LABEL(MSG_META_END_PREFER_SELECTION)
	CASE_LABEL(MSG_META_TAKE_SCREENSHOT)
	CASE_LABEL(MSG_META_TAKE_SCREENSHOT_PNG)
	CASE_LABEL(MSG_META_ALL_CHEER)
	CASE_LABEL(MSG_META_TOGGLE_ATTACKMOVE)
	CASE_LABEL(MSG_META_BEGIN_CAMERA_ROTATE_LEFT)
	CASE_LABEL(MSG_META_END_CAMERA_ROTATE_LEFT)
	CASE_LABEL(MSG_META_ALT_CAMERA_ROTATE_LEFT)
	CASE_LABEL(MSG_META_BEGIN_CAMERA_ROTATE_RIGHT)
	CASE_LABEL(MSG_META_END_CAMERA_ROTATE_RIGHT)
	CASE_LABEL(MSG_META_ALT_CAMERA_ROTATE_RIGHT)
	CASE_LABEL(MSG_META_BEGIN_CAMERA_ZOOM_IN)
	CASE_LABEL(MSG_META_END_CAMERA_ZOOM_IN)
	CASE_LABEL(MSG_META_BEGIN_CAMERA_ZOOM_OUT)
	CASE_LABEL(MSG_META_END_CAMERA_ZOOM_OUT)
	CASE_LABEL(MSG_META_CAMERA_RESET)
	CASE_LABEL(MSG_META_TOGGLE_CAMERA_TRACKING_DRAWABLE)
	CASE_LABEL(MSG_META_TOGGLE_FAST_FORWARD_REPLAY)
	CASE_LABEL(MSG_META_TOGGLE_PAUSE)
	CASE_LABEL(MSG_META_TOGGLE_PAUSE_ALT)
	CASE_LABEL(MSG_META_STEP_FRAME)
	CASE_LABEL(MSG_META_STEP_FRAME_ALT)
	CASE_LABEL(MSG_META_DEMO_INSTANT_QUIT)

#if defined(_ALLOW_DEBUG_CHEATS_IN_RELEASE)
	CASE_LABEL(MSG_CHEAT_RUNSCRIPT1)
	CASE_LABEL(MSG_CHEAT_RUNSCRIPT2)
	CASE_LABEL(MSG_CHEAT_RUNSCRIPT3)
	CASE_LABEL(MSG_CHEAT_RUNSCRIPT4)
	CASE_LABEL(MSG_CHEAT_RUNSCRIPT5)
	CASE_LABEL(MSG_CHEAT_RUNSCRIPT6)
	CASE_LABEL(MSG_CHEAT_RUNSCRIPT7)
	CASE_LABEL(MSG_CHEAT_RUNSCRIPT8)
	CASE_LABEL(MSG_CHEAT_RUNSCRIPT9)
	CASE_LABEL(MSG_CHEAT_TOGGLE_SPECIAL_POWER_DELAYS)
	CASE_LABEL(MSG_CHEAT_SWITCH_TEAMS)
	CASE_LABEL(MSG_CHEAT_KILL_SELECTION)
	CASE_LABEL(MSG_CHEAT_TOGGLE_HAND_OF_GOD_MODE)
	CASE_LABEL(MSG_CHEAT_INSTANT_BUILD)
	CASE_LABEL(MSG_CHEAT_DESHROUD)
	CASE_LABEL(MSG_CHEAT_ADD_CASH)
	CASE_LABEL(MSG_CHEAT_GIVE_ALL_SCIENCES)
	CASE_LABEL(MSG_CHEAT_GIVE_SCIENCEPURCHASEPOINTS)
	CASE_LABEL(MSG_CHEAT_SHOW_HEALTH)
	CASE_LABEL(MSG_CHEAT_TOGGLE_MESSAGE_TEXT)
#endif

#if defined(RTS_DEBUG)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_BEHIND_BUILDINGS)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_LETTERBOX)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_MESSAGE_TEXT)
	CASE_LABEL(MSG_META_DEMO_LOD_DECREASE)
	CASE_LABEL(MSG_META_DEMO_LOD_INCREASE)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_ZOOM_LOCK)
	CASE_LABEL(MSG_META_DEMO_PLAY_CAMEO_MOVIE)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_SPECIAL_POWER_DELAYS)
	CASE_LABEL(MSG_META_DEMO_BATTLE_CRY)
	CASE_LABEL(MSG_META_DEMO_SWITCH_TEAMS)
	CASE_LABEL(MSG_META_DEMO_SWITCH_TEAMS_BETWEEN_CHINA_USA)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_PARTICLEDEBUG)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_SHADOW_VOLUMES)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_FOGOFWAR)
	CASE_LABEL(MSG_META_DEMO_KILL_ALL_ENEMIES)
	CASE_LABEL(MSG_META_DEMO_KILL_SELECTION)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_HURT_ME_MODE)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_HAND_OF_GOD_MODE)
	CASE_LABEL(MSG_META_DEMO_DEBUG_SELECTION)
	CASE_LABEL(MSG_META_DEMO_LOCK_CAMERA_TO_SELECTION)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_SOUND)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_TRACKMARKS)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_WATERPLANE)
	CASE_LABEL(MSG_META_DEMO_TIME_OF_DAY)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_MUSIC)
	CASE_LABEL(MSG_META_DEMO_MUSIC_NEXT_TRACK)
	CASE_LABEL(MSG_META_DEMO_MUSIC_PREV_TRACK)
	CASE_LABEL(MSG_META_DEMO_NEXT_OBJECTIVE_MOVIE)
	CASE_LABEL(MSG_META_DEMO_PLAY_OBJECTIVE_MOVIE1)
	CASE_LABEL(MSG_META_DEMO_PLAY_OBJECTIVE_MOVIE2)
	CASE_LABEL(MSG_META_DEMO_PLAY_OBJECTIVE_MOVIE3)
	CASE_LABEL(MSG_META_DEMO_PLAY_OBJECTIVE_MOVIE4)
	CASE_LABEL(MSG_META_DEMO_PLAY_OBJECTIVE_MOVIE5)
	CASE_LABEL(MSG_META_DEMO_PLAY_OBJECTIVE_MOVIE6)
	CASE_LABEL(MSG_META_DEMO_BEGIN_ADJUST_PITCH)
	CASE_LABEL(MSG_META_DEMO_END_ADJUST_PITCH)
	CASE_LABEL(MSG_META_DEMO_BEGIN_ADJUST_DEFAULTPITCH)
	CASE_LABEL(MSG_META_DEMO_END_ADJUST_DEFAULTPITCH)
	CASE_LABEL(MSG_META_DEMO_BEGIN_ADJUST_FOV)
	CASE_LABEL(MSG_META_DEMO_END_ADJUST_FOV)
	CASE_LABEL(MSG_META_DEMO_LOCK_CAMERA_TO_PLANES)
	CASE_LABEL(MSG_META_DEMO_REMOVE_PREREQ)
	CASE_LABEL(MSG_META_DEMO_INSTANT_BUILD)
	CASE_LABEL(MSG_META_DEMO_FREE_BUILD)
	CASE_LABEL(MSG_META_DEMO_RUNSCRIPT1)
	CASE_LABEL(MSG_META_DEMO_RUNSCRIPT2)
	CASE_LABEL(MSG_META_DEMO_RUNSCRIPT3)
	CASE_LABEL(MSG_META_DEMO_RUNSCRIPT4)
	CASE_LABEL(MSG_META_DEMO_RUNSCRIPT5)
	CASE_LABEL(MSG_META_DEMO_RUNSCRIPT6)
	CASE_LABEL(MSG_META_DEMO_RUNSCRIPT7)
	CASE_LABEL(MSG_META_DEMO_RUNSCRIPT8)
	CASE_LABEL(MSG_META_DEMO_RUNSCRIPT9)
	CASE_LABEL(MSG_META_DEMO_ENSHROUD)
	CASE_LABEL(MSG_META_DEMO_DESHROUD)
	CASE_LABEL(MSG_META_DEBUG_SHOW_EXTENTS)
	CASE_LABEL(MSG_META_DEBUG_SHOW_AUDIO_LOCATIONS)
	CASE_LABEL(MSG_META_DEBUG_SHOW_HEALTH)
	CASE_LABEL(MSG_META_DEBUG_GIVE_VETERANCY)
	CASE_LABEL(MSG_META_DEBUG_TAKE_VETERANCY)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_AI_DEBUG)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_SUPPLY_CENTER_PLACEMENT)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_CAMERA_DEBUG)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_AVI)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_BW_VIEW)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_RED_VIEW)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_GREEN_VIEW)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_MOTION_BLUR_ZOOM)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_MILITARY_SUBTITLES)
	CASE_LABEL(MSG_META_DEMO_ADD_CASH)

#ifdef ALLOW_SURRENDER
	CASE_LABEL(MSG_META_DEMO_TEST_SURRENDER)
#endif

	CASE_LABEL(MSG_META_DEMO_TOGGLE_RENDER)
	CASE_LABEL(MSG_META_DEMO_KILL_AREA_SELECTION)
	CASE_LABEL(MSG_META_DEMO_CYCLE_LOD_LEVEL)
	CASE_LABEL(MSG_META_DEBUG_INCR_ANIM_SKATE_SPEED)
	CASE_LABEL(MSG_META_DEBUG_DECR_ANIM_SKATE_SPEED)
	CASE_LABEL(MSG_META_DEBUG_CYCLE_EXTENT_TYPE)
	CASE_LABEL(MSG_META_DEBUG_INCREASE_EXTENT_MAJOR)
	CASE_LABEL(MSG_META_DEBUG_INCREASE_EXTENT_MAJOR_BIG)
	CASE_LABEL(MSG_META_DEBUG_DECREASE_EXTENT_MAJOR)
	CASE_LABEL(MSG_META_DEBUG_DECREASE_EXTENT_MAJOR_BIG)
	CASE_LABEL(MSG_META_DEBUG_INCREASE_EXTENT_MINOR)
	CASE_LABEL(MSG_META_DEBUG_INCREASE_EXTENT_MINOR_BIG)
	CASE_LABEL(MSG_META_DEBUG_DECREASE_EXTENT_MINOR)
	CASE_LABEL(MSG_META_DEBUG_DECREASE_EXTENT_MINOR_BIG)
	CASE_LABEL(MSG_META_DEBUG_INCREASE_EXTENT_HEIGHT)
	CASE_LABEL(MSG_META_DEBUG_INCREASE_EXTENT_HEIGHT_BIG)
	CASE_LABEL(MSG_META_DEBUG_DECREASE_EXTENT_HEIGHT)
	CASE_LABEL(MSG_META_DEBUG_DECREASE_EXTENT_HEIGHT_BIG)
	CASE_LABEL(MSG_META_DEBUG_VTUNE_ON)
	CASE_LABEL(MSG_META_DEBUG_VTUNE_OFF)
	CASE_LABEL(MSG_META_DEBUG_TOGGLE_FEATHER_WATER)
	CASE_LABEL(MSG_META_DEBUG_DUMP_ASSETS)
	CASE_LABEL(MSG_NO_DRAW)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_METRICS)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_PROJECTILEDEBUG)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_VISIONDEBUG)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_THREATDEBUG)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_CASHMAPDEBUG)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_GRAPHICALFRAMERATEBAR)
	CASE_LABEL(MSG_META_DEMO_GIVE_ALL_SCIENCES)
	CASE_LABEL(MSG_META_DEMO_GIVE_RANKLEVEL)
	CASE_LABEL(MSG_META_DEMO_TAKE_RANKLEVEL)
	CASE_LABEL(MSG_META_DEMO_GIVE_SCIENCEPURCHASEPOINTS)
	CASE_LABEL(MSG_META_DEBUG_TOGGLE_NETWORK)
	CASE_LABEL(MSG_META_DEBUG_DUMP_PLAYER_OBJECTS)
	CASE_LABEL(MSG_META_DEBUG_DUMP_ALL_PLAYER_OBJECTS)
	CASE_LABEL(MSG_META_DEBUG_OBJECT_ID_PERFORMANCE)
	CASE_LABEL(MSG_META_DEBUG_DRAWABLE_ID_PERFORMANCE)
	CASE_LABEL(MSG_META_DEBUG_SLEEPY_UPDATE_PERFORMANCE)
	CASE_LABEL(MSG_META_DEBUG_WIN)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_DEBUG_STATS)
#endif // defined(RTS_DEBUG)

#if defined(RTS_DEBUG)
	CASE_LABEL(MSG_META_DEMO_TOGGLE_AUDIODEBUG)
#endif

#ifdef DUMP_PERF_STATS
	CASE_LABEL(MSG_META_DEMO_PERFORM_STATISTICAL_DUMP)
#endif

	CASE_LABEL(MSG_META_PLACE_BEACON)
	CASE_LABEL(MSG_META_REMOVE_BEACON)
	CASE_LABEL(MSG_END_META_MESSAGES)
	CASE_LABEL(MSG_MOUSEOVER_DRAWABLE_HINT)
	CASE_LABEL(MSG_MOUSEOVER_LOCATION_HINT)
	CASE_LABEL(MSG_VALID_GUICOMMAND_HINT)
	CASE_LABEL(MSG_INVALID_GUICOMMAND_HINT)
	CASE_LABEL(MSG_BEGIN_AREA_SELECTION_HINT)
	CASE_LABEL(MSG_END_AREA_SELECTION_HINT)
	CASE_LABEL(MSG_DO_ATTACK_OBJECT_HINT)
	CASE_LABEL(MSG_IMPOSSIBLE_ATTACK_HINT)
	CASE_LABEL(MSG_DO_FORCE_ATTACK_OBJECT_HINT)
	CASE_LABEL(MSG_DO_FORCE_ATTACK_GROUND_HINT)
	CASE_LABEL(MSG_GET_REPAIRED_HINT)
	CASE_LABEL(MSG_GET_HEALED_HINT)
	CASE_LABEL(MSG_DO_REPAIR_HINT)
	CASE_LABEL(MSG_RESUME_CONSTRUCTION_HINT)
	CASE_LABEL(MSG_ENTER_HINT)
	CASE_LABEL(MSG_DOCK_HINT)
	CASE_LABEL(MSG_DO_MOVETO_HINT)
	CASE_LABEL(MSG_DO_ATTACKMOVETO_HINT)
	CASE_LABEL(MSG_ADD_WAYPOINT_HINT)
	CASE_LABEL(MSG_HIJACK_HINT)
	CASE_LABEL(MSG_SABOTAGE_HINT)
	CASE_LABEL(MSG_FIREBOMB_HINT)
	CASE_LABEL(MSG_CONVERT_TO_CARBOMB_HINT)
	CASE_LABEL(MSG_CAPTUREBUILDING_HINT)

#ifdef ALLOW_SURRENDER
	CASE_LABEL(MSG_PICK_UP_PRISONER_HINT)
#endif

	CASE_LABEL(MSG_SNIPE_VEHICLE_HINT)
	CASE_LABEL(MSG_DEFECTOR_HINT)
	CASE_LABEL(MSG_SET_RALLY_POINT_HINT)
	CASE_LABEL(MSG_DO_SPECIAL_POWER_OVERRIDE_DESTINATION_HINT)
	CASE_LABEL(MSG_DO_SALVAGE_HINT)
	CASE_LABEL(MSG_DO_INVALID_HINT)
	CASE_LABEL(MSG_DO_ATTACK_OBJECT_AFTER_MOVING_HINT)
	CASE_LABEL(MSG_HACK_HINT)
	CASE_LABEL(MSG_BEGIN_NETWORK_MESSAGES)
	CASE_LABEL(MSG_CREATE_SELECTED_GROUP)
	CASE_LABEL(MSG_CREATE_SELECTED_GROUP_NO_SOUND)
	CASE_LABEL(MSG_DESTROY_SELECTED_GROUP)
	CASE_LABEL(MSG_REMOVE_FROM_SELECTED_GROUP)
	CASE_LABEL(MSG_SELECTED_GROUP_COMMAND)
	CASE_LABEL(MSG_CREATE_TEAM0)
	CASE_LABEL(MSG_CREATE_TEAM1)
	CASE_LABEL(MSG_CREATE_TEAM2)
	CASE_LABEL(MSG_CREATE_TEAM3)
	CASE_LABEL(MSG_CREATE_TEAM4)
	CASE_LABEL(MSG_CREATE_TEAM5)
	CASE_LABEL(MSG_CREATE_TEAM6)
	CASE_LABEL(MSG_CREATE_TEAM7)
	CASE_LABEL(MSG_CREATE_TEAM8)
	CASE_LABEL(MSG_CREATE_TEAM9)
	CASE_LABEL(MSG_SELECT_TEAM0)
	CASE_LABEL(MSG_SELECT_TEAM1)
	CASE_LABEL(MSG_SELECT_TEAM2)
	CASE_LABEL(MSG_SELECT_TEAM3)
	CASE_LABEL(MSG_SELECT_TEAM4)
	CASE_LABEL(MSG_SELECT_TEAM5)
	CASE_LABEL(MSG_SELECT_TEAM6)
	CASE_LABEL(MSG_SELECT_TEAM7)
	CASE_LABEL(MSG_SELECT_TEAM8)
	CASE_LABEL(MSG_SELECT_TEAM9)
	CASE_LABEL(MSG_ADD_TEAM0)
	CASE_LABEL(MSG_ADD_TEAM1)
	CASE_LABEL(MSG_ADD_TEAM2)
	CASE_LABEL(MSG_ADD_TEAM3)
	CASE_LABEL(MSG_ADD_TEAM4)
	CASE_LABEL(MSG_ADD_TEAM5)
	CASE_LABEL(MSG_ADD_TEAM6)
	CASE_LABEL(MSG_ADD_TEAM7)
	CASE_LABEL(MSG_ADD_TEAM8)
	CASE_LABEL(MSG_ADD_TEAM9)
	CASE_LABEL(MSG_DO_ATTACKSQUAD)
	CASE_LABEL(MSG_DO_WEAPON)
	CASE_LABEL(MSG_DO_WEAPON_AT_LOCATION)
	CASE_LABEL(MSG_DO_WEAPON_AT_OBJECT)
	CASE_LABEL(MSG_DO_SPECIAL_POWER)
	CASE_LABEL(MSG_DO_SPECIAL_POWER_AT_LOCATION)
	CASE_LABEL(MSG_DO_SPECIAL_POWER_AT_OBJECT)
	CASE_LABEL(MSG_SET_RALLY_POINT)
	CASE_LABEL(MSG_PURCHASE_SCIENCE)
	CASE_LABEL(MSG_QUEUE_UPGRADE)
	CASE_LABEL(MSG_CANCEL_UPGRADE)
	CASE_LABEL(MSG_QUEUE_UNIT_CREATE)
	CASE_LABEL(MSG_CANCEL_UNIT_CREATE)
	CASE_LABEL(MSG_DOZER_CONSTRUCT)
	CASE_LABEL(MSG_DOZER_CONSTRUCT_LINE)
	CASE_LABEL(MSG_DOZER_CANCEL_CONSTRUCT)
	CASE_LABEL(MSG_SELL)
	CASE_LABEL(MSG_EXIT)
	CASE_LABEL(MSG_EVACUATE)
	CASE_LABEL(MSG_EXECUTE_RAILED_TRANSPORT)
	CASE_LABEL(MSG_COMBATDROP_AT_LOCATION)
	CASE_LABEL(MSG_COMBATDROP_AT_OBJECT)
	CASE_LABEL(MSG_AREA_SELECTION_DEPRECATED)
	CASE_LABEL(MSG_DO_ATTACK_OBJECT)
	CASE_LABEL(MSG_DO_FORCE_ATTACK_OBJECT)
	CASE_LABEL(MSG_DO_FORCE_ATTACK_GROUND)
	CASE_LABEL(MSG_GET_REPAIRED)
	CASE_LABEL(MSG_GET_HEALED)
	CASE_LABEL(MSG_DO_REPAIR)
	CASE_LABEL(MSG_RESUME_CONSTRUCTION)
	CASE_LABEL(MSG_ENTER)
	CASE_LABEL(MSG_DOCK)
	CASE_LABEL(MSG_DO_MOVETO)
	CASE_LABEL(MSG_DO_ATTACKMOVETO)
	CASE_LABEL(MSG_DO_FORCEMOVETO)
	CASE_LABEL(MSG_ADD_WAYPOINT)
	CASE_LABEL(MSG_DO_GUARD_POSITION)
	CASE_LABEL(MSG_DO_GUARD_OBJECT)
	CASE_LABEL(MSG_DO_STOP)
	CASE_LABEL(MSG_DO_SCATTER)
	CASE_LABEL(MSG_INTERNET_HACK)
	CASE_LABEL(MSG_DO_CHEER)
	CASE_LABEL(MSG_TOGGLE_OVERCHARGE)
	CASE_LABEL(MSG_SWITCH_WEAPONS)
	CASE_LABEL(MSG_CONVERT_TO_CARBOMB)
	CASE_LABEL(MSG_CAPTUREBUILDING)
	CASE_LABEL(MSG_DISABLEVEHICLE_HACK)
	CASE_LABEL(MSG_STEALCASH_HACK)
	CASE_LABEL(MSG_DISABLEBUILDING_HACK)
	CASE_LABEL(MSG_SNIPE_VEHICLE)
	CASE_LABEL(MSG_DO_SPECIAL_POWER_OVERRIDE_DESTINATION)
	CASE_LABEL(MSG_DO_SALVAGE)
	CASE_LABEL(MSG_CLEAR_INGAME_POPUP_MESSAGE)
	CASE_LABEL(MSG_PLACE_BEACON)
	CASE_LABEL(MSG_REMOVE_BEACON)
	CASE_LABEL(MSG_SET_BEACON_TEXT)
	CASE_LABEL(MSG_SET_REPLAY_CAMERA)
	CASE_LABEL(MSG_SELF_DESTRUCT)
	CASE_LABEL(MSG_CREATE_FORMATION)
	CASE_LABEL(MSG_LOGIC_CRC)
	CASE_LABEL(MSG_SET_MINE_CLEARING_DETAIL)
	CASE_LABEL(MSG_ENABLE_RETALIATION_MODE)
	CASE_LABEL(MSG_BEGIN_DEBUG_NETWORK_MESSAGES)

#if defined(RTS_DEBUG) || defined(_ALLOW_DEBUG_CHEATS_IN_RELEASE)
	CASE_LABEL(MSG_DEBUG_KILL_SELECTION)
	CASE_LABEL(MSG_DEBUG_HURT_OBJECT)
	CASE_LABEL(MSG_DEBUG_KILL_OBJECT)
#endif

	CASE_LABEL(MSG_END_NETWORK_MESSAGES)
	CASE_LABEL(MSG_TIMESTAMP)
	CASE_LABEL(MSG_OBJECT_CREATED)
	CASE_LABEL(MSG_OBJECT_DESTROYED)
	CASE_LABEL(MSG_OBJECT_POSITION)
	CASE_LABEL(MSG_OBJECT_ORIENTATION)
	CASE_LABEL(MSG_OBJECT_JOINED_TEAM)

#ifdef ALLOW_SURRENDER
	CASE_LABEL(MSG_DO_SURRENDER)
	CASE_LABEL(MSG_RETURN_TO_PRISON)
	CASE_LABEL(MSG_PICK_UP_PRISONER)
#endif
	}

#undef CASE_LABEL
}


//------------------------------------------------------------------------------------------------
// GameMessageList
//

/**
 * Constructor
 */
GameMessageList::GameMessageList()
{
	m_firstMessage = nullptr;
	m_lastMessage = nullptr;
}

/**
 * Destructor
 */
GameMessageList::~GameMessageList()
{
	// destroy all messages currently on the list
	GameMessage *msg, *nextMsg;
	for( msg = m_firstMessage; msg; msg = nextMsg )
	{
		nextMsg = msg->next();
		// set list ptr to null to avoid it trying to remove itself from the list
		// that we are in the process of nuking...
		msg->friend_setList(nullptr);
		deleteInstance(msg);
	}
}

/**
 * Append message to end of message list
 */
void GameMessageList::appendMessage( GameMessage *msg )
{
	msg->friend_setNext(nullptr);

	if (m_lastMessage)
	{
		m_lastMessage->friend_setNext(msg);
		msg->friend_setPrev(m_lastMessage);
		m_lastMessage = msg;
	}
	else
	{
		// first message
		m_firstMessage = msg;
		m_lastMessage = msg;
		msg->friend_setPrev(nullptr);
	}

	// note containment within message itself
	msg->friend_setList(this);
}

/**
 * Inserts the msg after messageToInsertAfter.
 */
void GameMessageList::insertMessage( GameMessage *msg, GameMessage *messageToInsertAfter )
{
	// First, set msg's next to be messageToInsertAfter's next.
	msg->friend_setNext(messageToInsertAfter->next());

	// Next, set msg's prev to be messageToInsertAfter
	msg->friend_setPrev(messageToInsertAfter);

	// Now update the next message's prev to be msg
	if (msg->next())
		msg->next()->friend_setPrev(msg);
	else	// if the friend wasn't there, then messageToInsertAfter is the last message. Update it to be msg.
		m_lastMessage = msg;

	// Finally, update the messageToInsertAfter's next to be msg
	messageToInsertAfter->friend_setNext(msg);

	// note containment within the message itself
	msg->friend_setList(this);
}

/**
 * Remove given message from the list.
 */
void GameMessageList::removeMessage( GameMessage *msg )
{
	if (msg->next())
		msg->next()->friend_setPrev(msg->prev());
	else
		m_lastMessage = msg->prev();

	if (msg->prev())
		msg->prev()->friend_setNext(msg->next());
	else
		m_firstMessage = msg->next();

	msg->friend_setList(nullptr);
}

/**
 * Return whether or not a message of the given type is in the message list
 */
Bool GameMessageList::containsMessageOfType( GameMessage::Type type )
{
	GameMessage *msg = getFirstMessage();
	while (msg) {
		if (msg->getType() == type) {
			return true;
		}
		msg = msg->next();
	}
	return false;
}

//------------------------------------------------------------------------------------------------
// MessageStream
//


/**
 * Constructor
 */
MessageStream::MessageStream()
{
	m_firstTranslator = nullptr;
	m_nextTranslatorID = 1;
}

/**
 * Destructor
 */
MessageStream::~MessageStream()
{
	// destroy all translators
	TranslatorData *trans, *nextTrans;
	for( trans=m_firstTranslator; trans; trans=nextTrans )
	{
		nextTrans = trans->m_next;
		delete trans;
	}
}

/**
	* Init
	*/
void MessageStream::init()
{
	// extend
	GameMessageList::init();
}

/**
	* Reset
	*/
void MessageStream::reset()
{

	/// @todo Reset the MessageStream

	// extend
	GameMessageList::reset();

}

/**
	* Update
	*/
void MessageStream::update()
{
	// extend
	GameMessageList::update();

}

Bool MessageStream::isReadyForMessages() const
{
	return (ThePlayerList != nullptr);
}

/**
 * Create a new message of the given message type and append it
 * to this message stream.  Return the message such that any data
 * associated with this message can be attached to it.
 */
GameMessage *MessageStream::appendMessage( GameMessage::Type type )
{
	GameMessage *msg = newInstance(GameMessage)( type );

	// add message to list
	GameMessageList::appendMessage( msg );

	return msg;
}

/**
 * Create a new message of the given message type and insert it
 * in the stream after messageToInsertAfter, which must not be nullptr.
 */
GameMessage *MessageStream::insertMessage( GameMessage::Type type, GameMessage *messageToInsertAfter )
{
	GameMessage *msg = newInstance(GameMessage)(type);

	GameMessageList::insertMessage(msg, messageToInsertAfter);

	return msg;
}

/**
 * Attach the given Translator to the message stream, and return a
 * unique TranslatorID identifying it.
 * Translators are placed on a list, sorted by priority order.  If two
 * Translators share a priority, they are kept in the same order they
 * were attached.
 */
TranslatorID MessageStream::attachTranslator( GameMessageTranslator *translator,
																							UnsignedInt priority)
{
	MessageStream::TranslatorData *newSS = NEW MessageStream::TranslatorData;
	MessageStream::TranslatorData *ss;

	newSS->m_translator = translator;
	newSS->m_priority = priority;
	newSS->m_id = m_nextTranslatorID++;

	if (m_firstTranslator == nullptr)
	{
		// first Translator to be attached
		newSS->m_prev = nullptr;
		newSS->m_next = nullptr;
		m_firstTranslator = newSS;
		m_lastTranslator = newSS;
		return newSS->m_id;
	}

	// search the Translator list for our priority location
	for( ss=m_firstTranslator; ss; ss=ss->m_next )
		if (ss->m_priority > newSS->m_priority)
			break;

	if (ss)
	{
		// insert new Translator just BEFORE this one,
		// therefore, m_lastTranslator cannot be affected
		if (ss->m_prev)
		{
			ss->m_prev->m_next = newSS;
			newSS->m_prev = ss->m_prev;
			newSS->m_next = ss;
			ss->m_prev = newSS;
		}
		else
		{
			// insert at head of list
			newSS->m_prev = nullptr;
			newSS->m_next = m_firstTranslator;
			m_firstTranslator->m_prev = newSS;
			m_firstTranslator = newSS;
		}
	}
	else
	{
		// append Translator to end of list
		m_lastTranslator->m_next = newSS;
		newSS->m_prev = m_lastTranslator;
		newSS->m_next = nullptr;
		m_lastTranslator = newSS;
	}

	return newSS->m_id;
}

/**
	* Find a translator attached to this message stream given the ID
	*/
GameMessageTranslator* MessageStream::findTranslator( TranslatorID id )
{
	MessageStream::TranslatorData *translatorData;

	for( translatorData = m_firstTranslator; translatorData; translatorData = translatorData->m_next )
	{

		if( translatorData->m_id == id )
			return translatorData->m_translator;

	}

	return nullptr;

}

/**
 * Remove a previously attached translator.
 */
void MessageStream::removeTranslator( TranslatorID id )
{
	MessageStream::TranslatorData *ss;

	for( ss=m_firstTranslator; ss; ss=ss->m_next )
		if (ss->m_id == id)
		{
			// found the translator - remove it
			if (ss->m_prev)
				ss->m_prev->m_next = ss->m_next;
			else
				m_firstTranslator = ss->m_next;

			if (ss->m_next)
				ss->m_next->m_prev = ss->m_prev;
			else
				m_lastTranslator = ss->m_prev;

			// delete the translator data
			delete ss;

			break;
		}
}


// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
#if defined(RTS_DEBUG)

static Bool isInvalidDebugCommand( GameMessage::Type t )
{
	// see if this is something that should be prevented in multiplayer games
	// Don't reject this stuff in skirmish games.
	if (TheGameLogic && !TheGameLogic->isInSkirmishGame() &&
			(TheRecorder && TheRecorder->isMultiplayer() && TheRecorder->getMode() == RECORDERMODETYPE_RECORD))
	{
		switch (t)
		{
		case GameMessage::MSG_META_DEMO_REMOVE_PREREQ:
		case GameMessage::MSG_META_DEMO_INSTANT_BUILD:
		case GameMessage::MSG_META_DEMO_FREE_BUILD:
		case GameMessage::MSG_META_DEMO_GIVE_ALL_SCIENCES:
			// TheSuperHackers @tweak Debug cheats are now multiplayer compatible. Happy cheating Munkees :)
			return false;

		case GameMessage::MSG_META_DEMO_SWITCH_TEAMS:
		case GameMessage::MSG_META_DEMO_SWITCH_TEAMS_BETWEEN_CHINA_USA:
		case GameMessage::MSG_META_DEMO_KILL_ALL_ENEMIES:
		case GameMessage::MSG_META_DEMO_KILL_SELECTION:
		case GameMessage::MSG_META_DEMO_TOGGLE_HURT_ME_MODE:
		case GameMessage::MSG_META_DEMO_TOGGLE_HAND_OF_GOD_MODE:
		case GameMessage::MSG_META_DEMO_TOGGLE_SPECIAL_POWER_DELAYS:
		case GameMessage::MSG_META_DEMO_TIME_OF_DAY:
		case GameMessage::MSG_META_DEMO_LOCK_CAMERA_TO_PLANES:
		case GameMessage::MSG_META_DEMO_RUNSCRIPT1:
		case GameMessage::MSG_META_DEMO_RUNSCRIPT2:
		case GameMessage::MSG_META_DEMO_RUNSCRIPT3:
		case GameMessage::MSG_META_DEMO_RUNSCRIPT4:
		case GameMessage::MSG_META_DEMO_RUNSCRIPT5:
		case GameMessage::MSG_META_DEMO_RUNSCRIPT6:
		case GameMessage::MSG_META_DEMO_RUNSCRIPT7:
		case GameMessage::MSG_META_DEMO_RUNSCRIPT8:
		case GameMessage::MSG_META_DEMO_RUNSCRIPT9:
		case GameMessage::MSG_META_DEMO_ENSHROUD:
		case GameMessage::MSG_META_DEMO_DESHROUD:
		case GameMessage::MSG_META_DEBUG_GIVE_VETERANCY:
		case GameMessage::MSG_META_DEBUG_TAKE_VETERANCY:
		case GameMessage::MSG_META_DEMO_ADD_CASH:
		case GameMessage::MSG_META_DEBUG_INCR_ANIM_SKATE_SPEED:
		case GameMessage::MSG_META_DEBUG_DECR_ANIM_SKATE_SPEED:
		case GameMessage::MSG_META_DEBUG_CYCLE_EXTENT_TYPE:
		case GameMessage::MSG_META_DEMO_TOGGLE_RENDER:
		case GameMessage::MSG_META_DEBUG_INCREASE_EXTENT_MAJOR:
		case GameMessage::MSG_META_DEBUG_INCREASE_EXTENT_MAJOR_BIG:
		case GameMessage::MSG_META_DEBUG_DECREASE_EXTENT_MAJOR:
		case GameMessage::MSG_META_DEBUG_DECREASE_EXTENT_MAJOR_BIG:
		case GameMessage::MSG_META_DEBUG_INCREASE_EXTENT_MINOR:
		case GameMessage::MSG_META_DEBUG_INCREASE_EXTENT_MINOR_BIG:
		case GameMessage::MSG_META_DEBUG_DECREASE_EXTENT_MINOR:
		case GameMessage::MSG_META_DEBUG_DECREASE_EXTENT_MINOR_BIG:
		case GameMessage::MSG_META_DEBUG_INCREASE_EXTENT_HEIGHT:
		case GameMessage::MSG_META_DEBUG_INCREASE_EXTENT_HEIGHT_BIG:
		case GameMessage::MSG_META_DEBUG_DECREASE_EXTENT_HEIGHT:
		case GameMessage::MSG_META_DEBUG_DECREASE_EXTENT_HEIGHT_BIG:
		case GameMessage::MSG_META_DEMO_KILL_AREA_SELECTION:
		case GameMessage::MSG_DEBUG_KILL_SELECTION:
		case GameMessage::MSG_DEBUG_HURT_OBJECT:
		case GameMessage::MSG_DEBUG_KILL_OBJECT:
		case GameMessage::MSG_META_DEMO_GIVE_SCIENCEPURCHASEPOINTS:
		case GameMessage::MSG_META_DEMO_GIVE_RANKLEVEL:
		case GameMessage::MSG_META_DEMO_TAKE_RANKLEVEL:
		case GameMessage::MSG_META_DEBUG_WIN:

			return true;
		}
	}
	return false;
}
#endif

/**
 * Propagate messages thru attached Translators, invoking each Translator's
 * callback for each message in the stream.
 * Once all Translators have evaluated the message stream, all messages
 * in the stream are destroyed.
 */
void MessageStream::propagateMessages()
{
	MessageStream::TranslatorData *ss;
	GameMessage *msg, *next;

	// process each Translator
	for( ss=m_firstTranslator; ss; ss=ss->m_next )
	{
		for( msg=m_firstMessage; msg; msg=next )
		{
			if (ss->m_translator
#if defined(RTS_DEBUG)
				&& !isInvalidDebugCommand(msg->getType())
#endif
				)
			{
				GameMessageDisposition disp = ss->m_translator->translateGameMessage(msg);
				if (disp == DESTROY_MESSAGE)
				{
					next = msg->next();
					deleteInstance(msg);
					continue;
				}
			}

			next = msg->next();
		}
	}

	// Transfer all messages that reached the end of the stream to TheCommandList.
	// Note that the client translators above can also create game messages.
	TheCommandList->appendMessageList( m_firstMessage );

	// TheSuperHackers @tweak Delete network messages that are redundant. This can reduce network traffic.
	for( msg=TheCommandList->getFirstMessage(); msg; msg=next )
	{
		next = msg->next();
		if (isRedundantNetworkMessage(msg))
		{
			deleteInstance(msg);
		}
	}

	// clear the stream
	m_firstMessage = nullptr;
	m_lastMessage = nullptr;

}

Bool MessageStream::isRedundantNetworkMessage(const GameMessage *msg) const
{
	switch (msg->getType())
	{
	case GameMessage::MSG_DESTROY_SELECTED_GROUP:
	{
		const GameMessage* msgNext = msg->next();
		if (!msgNext)
			break;

		switch (msgNext->getType())
		{
		case GameMessage::MSG_CREATE_SELECTED_GROUP:
		case GameMessage::MSG_CREATE_SELECTED_GROUP_NO_SOUND:
			if (msgNext->getArgumentCount() >= 1)
			{
				const Bool newGroup = msgNext->getArgument(0)->boolean;
				if (newGroup)
					return true;
			}
			break;
		case GameMessage::MSG_DESTROY_SELECTED_GROUP:
			return true;
		case GameMessage::MSG_SELECT_TEAM0:
		case GameMessage::MSG_SELECT_TEAM1:
		case GameMessage::MSG_SELECT_TEAM2:
		case GameMessage::MSG_SELECT_TEAM3:
		case GameMessage::MSG_SELECT_TEAM4:
		case GameMessage::MSG_SELECT_TEAM5:
		case GameMessage::MSG_SELECT_TEAM6:
		case GameMessage::MSG_SELECT_TEAM7:
		case GameMessage::MSG_SELECT_TEAM8:
		case GameMessage::MSG_SELECT_TEAM9:
			return true;
		}
		break;
	}
	case GameMessage::MSG_SET_REPLAY_CAMERA:
	{
		for (const GameMessage* msgNext = msg->next(); msgNext; msgNext = msgNext->next())
		{
			if (msgNext->getType() == GameMessage::MSG_SET_REPLAY_CAMERA)
				return true;
		}
		break;
	}
	}

	return false;
}

//------------------------------------------------------------------------------------------------
// CommandList
//

/**
 * Constructor
 */
CommandList::CommandList()
{
}

/**
 * Destructor
 */
CommandList::~CommandList()
{
	destroyAllMessages();
}

/**
	* Init
	*/
void CommandList::init()
{

	// extend
	GameMessageList::init();

}

/**
	* Destroy all messages on the list, and reset list to empty
	*/
void CommandList::reset()
{

	// extend
	GameMessageList::reset();

	// destroy all messages
	destroyAllMessages();

}

/**
	* Update
	*/
void CommandList::update()
{

	// extend
	GameMessageList::update();

}

/**
	* Destroy all messages on the command list, this will get called from the
	* destructor and reset methods, DO NOT throw exceptions
	*/
void CommandList::destroyAllMessages()
{
	GameMessage *msg, *next;

	for( msg=m_firstMessage; msg; msg=next )
	{
		next = msg->next();
		deleteInstance(msg);
	}

	m_firstMessage = nullptr;
	m_lastMessage = nullptr;

}

/**
 * Adds messages to the end of TheCommandList.
 * Primarily used by TheMessageStream to put the final messages that reach the end of the
 * stream on TheCommandList. Since TheGameClient will update faster than TheNetwork
 * and TheGameLogic, messages will accumulate on this list.
 */
void CommandList::appendMessageList( GameMessage *list )
{
	GameMessage *msg, *next;

	for( msg = list; msg; msg = next )
	{
		next = msg->next();
		appendMessage( msg );
	}
}

//-----------------------------------------------------------------------------
/**
 * Given an "anchor" point and the current mouse position (dest),
 * construct a valid 2D bounding region.
 */
void buildRegion( const ICoord2D *anchor, const ICoord2D *dest, IRegion2D *region )
{
	// build rectangular region defined by the drag selection
	if (anchor->x < dest->x)
	{
		region->lo.x = anchor->x;
		region->hi.x = dest->x;
	}
	else
	{
		region->lo.x = dest->x;
		region->hi.x = anchor->x;
	}

	if (anchor->y < dest->y)
	{
		region->lo.y = anchor->y;
		region->hi.y = dest->y;
	}
	else
	{
		region->lo.y = dest->y;
		region->hi.y = anchor->y;
	}
}
```
