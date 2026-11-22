# Translation Packet `PKT-406`: `Radar`

## 0. Lean Port & Oracle Contract (Read First)
- **Zero-Skeleton Target:** Create `crates/zh_logic/src/player/radar.rs` and `crates/zh_logic/src/player/mod.rs` (`pub mod radar;`), export `pub mod player;` in `crates/zh_logic/src/lib.rs`, implement the full 1,000-seed parity assertions in `crates/zh_logic/tests/golden_pkt406_radar.rs` (removing `#[ignore]`), and regenerate `docs/PORT_MAP.md` via `python3 scripts/port_map.py --write`. Do **not** modify unrelated modules or files.
- **Headless Simulation Scope (No W3D / Display Buffers):**
  - **Port in full:** Constants (`RADAR_CELL_WIDTH = 128`, `RADAR_CELL_HEIGHT = 128`, `MAX_RADAR_EVENTS = 64`, `MAX_PLAYER_COUNT = 16`, `RADAR_QUEUE_TERRAIN_REFRESH_DELAY = 90.0f`), enums (`RadarEventType` `0..=10`, `RadarPriorityType` `0..=4`), `RadarObject` (metadata, color, `Xfer`), `RadarEvent`, `Radar` (`world_to_radar`, `radar_to_world_2d`, `find_draw_positions`, `is_priority_visible`, `hide`/`is_radar_hidden`, `force_on`/`is_radar_forced`, `create_event`, `create_player_event`, `try_event`, `update`, `reset`, `clear_all_events`, `get_last_event_loc`, `queue_terrain_refresh`, `refresh_terrain`, `Radar::xfer` version `1` (`RETAIL_COMPATIBLE_XFER_SAVE = 1`) + version `2` load support, and priority-sorted object list ordering helpers).
  - **Omit / keep abstract:** Direct `W3D` texture surface blitting and `GameWindow*` pixel UI callbacks (`draw`, `clear_shroud`, `set_shroud_level`, raw window handle pointer). Expose `Radar::configure_extent(ext: &Region3D)` setting `m_xSample = ext.width() / 128.0` and `m_ySample = ext.height() / 128.0` cleanly without requiring `GameWindowManager`.
- **Exact C++ Numerical & Logic Quirks (Required for 1,000-Seed Bit Parity):**
  1. **Coordinate conversion (`worldToRadar`, `radarToWorld2D`):** `radar.x = (world.x / m_xSample) as Int` (truncation toward zero), clamped to `0..RADAR_CELL_WIDTH` (`0..=127`). `radarToWorld2D` clamps input `x, y` to `0..=127`, then `world.x = (x as Real) * m_xSample`, `world.y = (y as Real) * m_ySample`.
  2. **`findDrawPositions` truncation (`Radar.cpp:771-798`):** Note C++ writes `ul.x = 0` (`ratio_w >= ratio_h`), `ul.y = ((height as Real - radar.y) / 2.0f) as Int` (float-to-`i32` truncation toward zero!), `lr.x = radar.x as Int`, `lr.y = height - ul.y` (`else` branch mirrors `ul.x`/`lr.x` similarly), then adds `start_x`/`start_y` to both `ul` and `lr`. Preserve the exact single-precision `f32` operation order (`m_mapExtent.width() / (width as f32 * 1.0f)`, etc.).
  3. **`createPlayerEvent` darkening (`Radar.cpp:948-967`):** `color[0]` unpacks ARGB `Color` (`GameGetColorComponents`: `r = (c >> 16) & 0xff`, `g = (c >> 8) & 0xff`, `b = c & 0xff`, `a = (c >> 24) & 0xff`). `color[1]` subtracts `REAL_TO_INT(color[0].red * 0.75f)` (`((channel as f32) * 0.75_f32) as Int`, clamped at `>= 0`). Check exact RGBA byte order against Seed-0 `s0_ev0`!
  4. **`internalCreateEvent` & `tryEvent` (`Radar.cpp:976-1013, 1166-1218`):** `die_frame = (frame as f32 + 30.0f * seconds_to_live) as UnsignedInt`; `fade_frame = (die_frame as f32 - 30.0f * 0.5f) as UnsignedInt`. `RADAR_EVENT_BEACON_PULSE` does **not** update `m_lastRadarEvent`. `PRESERVE_RADAR_WARNING_SUPPRESSION = 1` makes `RADAR_EVENT_UNDER_ATTACK` suppress map-wide whenever `(current_frame.wrapping_sub(ev.create_frame)) < 300`; all other valid types (`1..=10`, excluding `0` and `>= 11`) require `dist_sq <= 250.0f * 250.0f`. Note `try_event` checks **all** 64 slots matching `ev.type` (even `active == FALSE`, since historical `createFrame` persists!).
  5. **`update()` & `reset()` (`Radar.cpp:256-305`):** Expires active events when `ev.create_frame != 0 && this_frame > ev.die_frame`; triggers `refreshTerrain` when `m_queueTerrainRefreshFrame != 0 && (this_frame - m_queueTerrainRefreshFrame) as f32 > 90.0f`. `reset()` clears lists, calls `clear_all_events()`, resets `m_radarForceOn`, and intentionally leaves `m_radarHidden` unchanged (`Radar.cpp:265`).
  6. **`Radar::xfer` (`Radar.cpp:1360-1438`):** Generated oracle uses `RETAIL_COMPATIBLE_XFER_SAVE = 1` (`version = 1`), writing `m_radarHidden[local_player]`, `m_radarForceOn[local_player]`, both `RadarObject` list versions + `u16` counts, `u16(64)` event count, `64` events (`type` `i32` LE, `active` `Bool`, `createFrame` `u32`, `dieFrame` `u32`, `fadeFrame` `u32`, `color1`/`color2` `RGBAColorInt`, `worldLoc` `Coord3D`, `radarLoc` `ICoord2D`, `soundPlayed` `Bool`), `next_free` `i32`, `last_event` `i32` (`4499` bytes total with empty object lists). Call `radar.xfer(&mut xfer_crc)` directly to match `"xcrc"` (`XferCrc` uses default `not(feature = "retail-crc")`).
- **Oracle Fixture Schema (`docs/golden_vectors/PKT-406.json`, 1,000 seeds):**
  - Inputs per line: `ext` `[lo_x_bits, lo_y_bits, hi_x_bits, hi_y_bits]`, `wq` `[wx_bits, wy_bits]`, `rq` `[rx, ry]`, `dr` `[sx, sy, dw, dh]`, `lp` `local_player_index`, `pcol` `Color` (`i32`), `hm` `u16` `hide` bitmask (`0..16`), `fm` `u16` `forceOn` bitmask (`0..16`), `ops` flat 5-`i32` tuples `[kind, a0, a1, a2, a3, ...]` where `0` = set `frame = a0` & `update()`, `1` = `createEvent(type=a0, wx=bits(a1), wy=bits(a2), ttl=bits(a3))`, `2` = `createPlayerEvent(local_player, ...)`, `3` = `tryEvent(type=a0, wx=bits(a1), wy=bits(a2))` pushing `0|1` to `try_rets`, `4` = `queueTerrainRefresh()`, `5` = `reset()`.
  - Outputs per line: `w2r`, `r2w` (bit patterns), `ul_lr`, `pvis` (bit `b = p + 1` for `p in -1..=6`), `try_rets`, `last_ok`, `last_loc`, `next_free`, `last_idx`, `act` (active count), `qframe`, `h_out`, `f_out`, `xlen`, `xbcrc` (`zh_common::crc::Crc` over `XferSave` bytes), `xcrc` (`XferCrc::get_crc()` after `Radar::xfer`).
  - Seed-0 self-diagnosing fields (`s0_samp`, `s0_ev0`, `s0_xhex`) pinpoint exact sample scaling, first event fields, and raw 4,499-byte `XferSave` hex on Seed 0.

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-406`
- **Work Package:** `WP-6.1` (Phase 6)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Core Spine
- **Allowed Rust Output File:** `crates/zh_logic/src/player/radar.rs` *(Plus `crates/zh_logic/src/player/mod.rs`, `pub mod player;` in `crates/zh_logic/src/lib.rs`, completing `crates/zh_logic/tests/golden_pkt406_radar.rs` + removing `#[ignore]`, and running `python3 scripts/port_map.py --write`)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (27):**
  - `Common/GameAudio.h`
  - `Common/GameMemory.h`
  - `Common/GameState.h`
  - `Common/GameUtility.h`
  - `Common/GlobalData.h`
  - `Common/MiscAudio.h`
  - `Common/Player.h`
  - `Common/PlayerList.h`
  - `Common/Radar.h`
  - `Common/SubsystemInterface.h`
  - `Common/ThingTemplate.h`
  - `Common/Xfer.h`
  - `GameClient/Color.h`
  - `GameClient/ControlBar.h`
  - `GameClient/Display.h`
  - `GameClient/Drawable.h`
  - `GameClient/Eva.h`
  - `GameClient/GameWindowManager.h`
  - `GameClient/InGameUI.h`
  - `GameLogic/GameLogic.h`
  - `GameLogic/Module/ContainModule.h`
  - `GameLogic/Module/StealthUpdate.h`
  - `GameLogic/Object.h`
  - `GameLogic/PartitionManager.h`
  - `GameLogic/TerrainLogic.h`
  - `Lib/BaseType.h`
  - `PreRTS.h`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (25 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferObjectID( &objectID )`
  3. `xferColor( &m_color )`
  4. `xferVersion( &version, currentVersion )`
  5. `xferUnsignedShort( &count )`
  6. `xferSnapshot( radarObject )`
  7. `xferSnapshot( radarObject )`
  8. `xferVersion( &version, currentVersion )`
  9. `xferBool( &value )`
  10. `xferBool( &value )`
  11. `xferUser(&m_radarHidden, sizeof(m_radarHidden))`
  12. `xferUser(&m_radarForceOn, sizeof(m_radarForceOn))`
  13. `xferUnsignedShort( &eventCount )`
  14. `xferUser( &m_event[ i ].type, sizeof( RadarEventType ) )`
  15. `xferBool( &m_event[ i ].active )`
  16. `xferUnsignedInt( &m_event[ i ].createFrame )`
  17. `xferUnsignedInt( &m_event[ i ].dieFrame )`
  18. `xferUnsignedInt( &m_event[ i ].fadeFrame )`
  19. `xferRGBAColorInt( &m_event[ i ].color1 )`
  20. `xferRGBAColorInt( &m_event[ i ].color2 )`
  21. `xferCoord3D( &m_event[ i ].worldLoc )`
  22. `xferICoord2D( &m_event[ i ].radarLoc )`
  23. `xferBool( &m_event[ i ].soundPlayed )`
  24. `xferInt( &m_nextFreeRadarEvent )`
  25. `xferInt( &m_lastRadarEvent )`
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if RETAIL_COMPATIBLE_XFER_SAVE`

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

### `Core/GameEngine/Include/Common/Radar.h`
```cpp
// FILE: Radar.h //////////////////////////////////////////////////////////////////////////////////
// Author: Colin Day, January 2002
// Desc:   Logical radar implementation
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Lib/BaseType.h"
#include "Common/SubsystemInterface.h"
#include "Common/GameMemory.h"
#include "GameClient/Display.h"	// for ShroudLevel
#include "GameClient/Color.h"

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
class GameWindow;
class Object;
class Player;
class TerrainLogic;

// GLOBAL /////////////////////////////////////////////////////////////////////////////////////////
//
// the following is used for the resolution of the radar "cells" ... this is how accurate
// the radar is and also reflects directly the size of the image we build ... which with
// WW3D must be a square power of two as well
//
enum
{
	RADAR_CELL_WIDTH  = 128,	// radar created at this horz resolution
	RADAR_CELL_HEIGHT = 128   // radar created at this vert resolution
};

//-------------------------------------------------------------------------------------------------
/** These event types determine the colors radar events happen in to make it easier for us
	* to play events with a consistent color scheme */
//-------------------------------------------------------------------------------------------------
enum RadarEventType CPP_11(: Int)
{
	RADAR_EVENT_INVALID = 0,
	RADAR_EVENT_CONSTRUCTION,
	RADAR_EVENT_UPGRADE,
	RADAR_EVENT_UNDER_ATTACK,
	RADAR_EVENT_INFORMATION,
	RADAR_EVENT_BEACON_PULSE,
	RADAR_EVENT_INFILTRATION, //for defection, hijacking, hacking, carbombing, and other sneaks
	RADAR_EVENT_BATTLE_PLAN,
	RADAR_EVENT_STEALTH_DISCOVERED,		// we discovered a stealth unit
	RADAR_EVENT_STEALTH_NEUTRALIZED,	// our stealth unit has been revealed
	RADAR_EVENT_FAKE,					//Internally creates a radar event, but doesn't notify the player (unit lost
														//for example, so we can use the spacebar to jump to the event).

	RADAR_EVENT_NUM_EVENTS

};

// PROTOTYPES /////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
/** Radar objects are objects that are on the radar, go figure :) */
//-------------------------------------------------------------------------------------------------
class RadarObject : public MemoryPoolObject,
										public Snapshot
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( RadarObject, "RadarObject" )

public:

	RadarObject();
	// destructor prototype defined by memory pool glue

	// color management
	void setColor( Color c ) { m_color = c; }
	Color getColor() const { return m_color; }

	void friend_setObject( Object *obj ) { m_object = obj; }
	Object *friend_getObject() { return m_object; }
	const Object *friend_getObject() const { return m_object; }

	void friend_setNext( RadarObject *next ) { m_next = next; }
	RadarObject *friend_getNext() { return m_next; }
	const RadarObject *friend_getNext() const { return m_next; }

	Bool isTemporarilyHidden() const;

protected:

	// snapshot methods
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

	Object *m_object;				///< the object
	RadarObject *m_next;		///< next radar object
	Color m_color;					///< color to draw for this object on the radar

};

//-------------------------------------------------------------------------------------------------
/** Radar priorities.  Keep this in sync with the priority names list below */
//-------------------------------------------------------------------------------------------------
enum RadarPriorityType CPP_11(: Int)
{
	RADAR_PRIORITY_INVALID,					// a priority that has not been set (in general it won't show up on the radar)
	RADAR_PRIORITY_NOT_ON_RADAR,		// object specifically forbidden from being on the radar
	RADAR_PRIORITY_STRUCTURE,				// structure level drawing priority
	RADAR_PRIORITY_UNIT,						// unit level drawing priority
	RADAR_PRIORITY_LOCAL_UNIT_ONLY,	// unit priority, but only on the radar if controlled by the local player

	RADAR_PRIORITY_NUM_PRIORITIES
};
#ifdef DEFINE_RADAR_PRIORITY_NAMES
static const char *const RadarPriorityNames[] =
{
	"INVALID",											// a priority that has not been set (in general it won't show up on the radar)
	"NOT_ON_RADAR",									// object specifically forbidden from being on the radar
	"STRUCTURE",										// structure level drawing priority
	"UNIT",													// unit level drawing priority
	"LOCAL_UNIT_ONLY",							// unit priority, but only on the radar if controlled by the local player

	nullptr
};
static_assert(ARRAY_SIZE(RadarPriorityNames) == RADAR_PRIORITY_NUM_PRIORITIES + 1, "Incorrect array size");
#endif  // DEFINE_RADAR_PRIOTITY_NAMES

//-------------------------------------------------------------------------------------------------
/** Interface for the radar */
//-------------------------------------------------------------------------------------------------
class Radar : public Snapshot,
							public SubsystemInterface
{

public:

	Radar();
	virtual ~Radar() override;

	virtual void init() override { }														///< subsystem initialization
	virtual void reset() override;															///< subsystem reset
	virtual void update() override;														///< subsystem per frame update

	// is the game window parameter the radar window
	Bool isRadarWindow( GameWindow *window ) { return (m_radarWindow == window) && (m_radarWindow != nullptr); }

	Bool radarToWorld( const ICoord2D *radar, Coord3D *world );		///< radar point to world point on terrain
	Bool radarToWorld2D( const ICoord2D *radar, Coord3D *world );		///< radar point to world point (x,y only!)
	Bool worldToRadar( const Coord3D *world, ICoord2D *radar );		///< translate world point to radar (x,y)
	Bool localPixelToRadar( const ICoord2D *pixel, ICoord2D *radar );	///< translate pixel (with UL of radar being (0,0)) to logical radar coordinates
	Bool screenPixelToWorld( const ICoord2D *pixel, Coord3D *world ); ///< translate pixel (with UL of the screen being (0,0)) to world position in the world
	Object *objectUnderRadarPixel( const ICoord2D *pixel );				///< return the object (if any) represented by the pixel coordinates passed in
	void findDrawPositions( Int startX, Int startY, Int width, Int height,
													ICoord2D *ul, ICoord2D *lr );					///< make translation for screen area of radar square to scaled aspect ratio preserving points inside the radar area

	// priority inquiry
	static Bool isPriorityVisible( RadarPriorityType priority );		///< is the priority passed in a "visible" one on the radar

	// radar events
	void createEvent( const Coord3D *world, RadarEventType type, Real secondsToLive = 4.0f );	///< create radar event at location in world
	void createPlayerEvent( Player *player, const Coord3D *world, RadarEventType type, Real secondsToLive = 4.0f );  ///< create radar event using player colors

	Bool getLastEventLoc( Coord3D *eventPos );							///< get last event loc (if any)
	void tryUnderAttackEvent( const Object *obj );					///< try to make an "under attack" event if it's the proper time
	void tryInfiltrationEvent( const Object *obj );					///< try to make an "infiltration" event if it's the proper time
 	Bool tryEvent( RadarEventType event, const Coord3D *pos );	///< try to make a "stealth" event

	// adding and removing objects from the radar
	virtual Bool addObject( Object *obj ); ///< add object to radar
	virtual Bool removeObject( Object *obj ); ///< remove object from radar

	// radar options
	void hide( Int playerIndex, Bool hide ) { m_radarHidden[playerIndex] = hide; } ///< hide/show the radar
	Bool isRadarHidden( Int playerIndex ) { return m_radarHidden[playerIndex]; } ///< is radar hidden
	// other radar option methods here like the ability to show a certain
	// team, show buildings, show units at all, etc

	// forcing the radar on/off regardless of player situation
	void forceOn( Int playerIndex, Bool force ) { m_radarForceOn[playerIndex] = force; } ///< force the radar to be on
	Bool isRadarForced( Int playerIndex ) { return m_radarForceOn[playerIndex]; } ///< is radar forced on?

	/// refresh the water values for the radar
	virtual void refreshTerrain( TerrainLogic *terrain );

	/// refresh the radar when the state of world objects changes drastically
	virtual void refreshObjects() {};

	/// queue a refresh of the terrain at the next available time
	virtual void queueTerrainRefresh();

	virtual void newMap( TerrainLogic *terrain );	///< reset radar for new map

	virtual void draw( Int pixelX, Int pixelY, Int width, Int height ) = 0;	///< draw the radar

	/// empty the entire shroud
	virtual void clearShroud() = 0;

	/// TheSuperHackers @performance xezon 20/12/2025 Provides beginSetShroudLevel and endSetShroudLevel to improve performance.
	/// Calling setShroudLevel many times is very expensive because it will lock a render resource on every call.
	virtual void setShroudLevel( Int x, Int y, CellShroudStatus setting ) = 0; ///< set the shroud level at shroud cell x,y
	virtual void beginSetShroudLevel() {} ///< call this once before multiple calls to setShroudLevel for better performance
	virtual void endSetShroudLevel() {} ///< call this once after beginSetShroudLevel and setShroudLevel

	virtual void notifyViewChanged() {} ///< signals that the camera view has changed

protected:

	// snapshot methods
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

	/// internal method for creating a radar event with specific colors
	void internalCreateEvent( const Coord3D *world, RadarEventType type, Real secondsToLive,
														const RGBAColorInt *color1, const RGBAColorInt *color2 );

	void deleteList( RadarObject **list );
	void deleteListResources();			///< delete list radar resources used
	Bool deleteFromList( Object *obj, RadarObject **list );	///< try to remove object from specific list

	Real getTerrainAverageZ() const { return m_terrainAverageZ; }
	Real getWaterAverageZ() const { return m_waterAverageZ; }

	void clearAllEvents();					///< remove all radar events in progress

	// search the object list for an object that maps to the given logical radar coordinates
	Object *searchListForRadarLocationMatch( RadarObject *listHead, ICoord2D *radarMatch );

	void linkRadarObject( RadarObject *newObj, RadarObject **list );
	void assignObjectColorToRadarObject( RadarObject *radarObj, Object *obj );

	Bool m_radarHidden[MAX_PLAYER_COUNT]; ///< true when radar is not visible
	Bool m_radarForceOn[MAX_PLAYER_COUNT]; ///< true when radar is forced to be on

	RadarObject *m_objectList;						///< list of objects in the radar
	RadarObject *m_localObjectList;				/** list of objects for the local player, sorted
																					* in exactly the same priority as the regular
																					* object list for all other objects */

	Real m_terrainAverageZ;								///< average Z for terrain samples
	Real m_waterAverageZ;									///< average Z for water samples

	//
	// when dealing with world sampling we will sample at these intervals so that
	// the whole map can be accounted for within our RADAR_CELL_WIDTH and
	// RADAR_CELL_HEIGHT resolutions
	//
	Real m_xSample;
	Real m_ySample;

	enum { MAX_RADAR_EVENTS = 64 };
	struct RadarEvent
	{
		RadarEventType type;								///< type of this radar event
		Bool active;												///< TRUE when event is "active", otherwise it's just historical information in the event array to look through
		UnsignedInt createFrame;						///< frame event was created on
		UnsignedInt dieFrame;								///< frame the event will go away on
		UnsignedInt fadeFrame;							///< start fading out on this frame
		RGBAColorInt color1;								///< color 1 for drawing
		RGBAColorInt color2;								///< color 2 for drawing
		Coord3D worldLoc;										///< location of event in the world
		ICoord2D radarLoc;									///< 2D radar location of the event
		Bool soundPlayed;										///< TRUE when we have played the radar sound for this
	};
	RadarEvent m_event[ MAX_RADAR_EVENTS ];///< our radar events
	Int m_nextFreeRadarEvent;							///< index into m_event for where to store the next event
	Int m_lastRadarEvent;									///< index of the most recent radar event

	GameWindow *m_radarWindow;						///< window we display the radar in

	Region3D m_mapExtent;									///< extents of the current map

	UnsignedInt m_queueTerrainRefreshFrame;  ///< frame we requested the last terrain refresh on

};

// EXTERNALS //////////////////////////////////////////////////////////////////////////////////////
extern Radar *TheRadar;  ///< the radar singleton extern

// TheSuperHackers @feature helmutbuhler 10/04/2025
// Radar that does nothing. Used for Headless Mode.
class RadarDummy : public Radar
{
public:
	virtual void draw(Int pixelX, Int pixelY, Int width, Int height) override { }
	virtual void clearShroud() override { }
	virtual void setShroudLevel(Int x, Int y, CellShroudStatus setting) override { }
};
```

### `Core/GameEngine/Source/Common/System/Radar.cpp`
```cpp
// FILE: Radar.cpp ////////////////////////////////////////////////////////////////////////////////
// Author: Colin Day, January 2002
// Desc:   Radar logic implementation
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/GameAudio.h"
#include "Common/GameState.h"
#include "Common/GameUtility.h"
#include "Common/MiscAudio.h"
#include "Common/Radar.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/ThingTemplate.h"
#include "Common/GlobalData.h"
#include "Common/Xfer.h"

#include "GameClient/Drawable.h"
#include "GameClient/Eva.h"
#include "GameClient/GameWindowManager.h"
#include "GameClient/InGameUI.h"
#include "GameClient/ControlBar.h"

#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/TerrainLogic.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/StealthUpdate.h"


// GLOBALS ////////////////////////////////////////////////////////////////////////////////////////
Radar *TheRadar = nullptr;  ///< the radar global singleton

// PRIVATE ////////////////////////////////////////////////////////////////////////////////////////
#define RADAR_QUEUE_TERRAIN_REFRESH_DELAY (static_cast<float>(LOGICFRAMES_PER_SECOND) * 3.0f)

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void Radar::deleteList( RadarObject **list )
{
	while( *list )
	{
		RadarObject *nextObject = (*list)->friend_getNext();
		(*list)->friend_getObject()->friend_setRadarData( nullptr );
		deleteInstance(*list);
		*list = nextObject;
	}
}

//-------------------------------------------------------------------------------------------------
/** Delete list resources used by the radar and return them to the memory pools */
//-------------------------------------------------------------------------------------------------
void Radar::deleteListResources()
{
	deleteList(&m_objectList);
	deleteList(&m_localObjectList);

#ifdef DEBUG_CRASHING
	for( Object *obj = TheGameLogic->getFirstObject(); obj; obj = obj->getNextObject() )
	{
		DEBUG_ASSERTCRASH( obj->friend_getRadarData() == nullptr,
			("Radar::deleteListResources: Unexpectedly an object still has radar data assigned") );
	}
#endif
}

// PUBLIC METHODS /////////////////////////////////////////////////////////////////////////////////
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
RadarObject::RadarObject()
{

	m_object = nullptr;
	m_next = nullptr;
	m_color = GameMakeColor( 255, 255, 255, 255 );

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
RadarObject::~RadarObject()
{

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Bool RadarObject::isTemporarilyHidden() const
{
	Drawable* draw = m_object->getDrawable();

	if (draw == nullptr || draw->getStealthLook() == STEALTHLOOK_INVISIBLE || draw->isDrawableEffectivelyHidden())
		return true;

	return false;
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void RadarObject::crc( Xfer *xfer )
{

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void RadarObject::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// object id
	ObjectID objectID = m_object ? m_object->getID() : INVALID_ID;
	xfer->xferObjectID( &objectID );
	if( xfer->getXferMode() == XFER_LOAD )
	{

		// find the object and save
		m_object = TheGameLogic->findObjectByID( objectID );
		if( m_object == nullptr )
		{

			DEBUG_CRASH(( "RadarObject::xfer - Unable to find object for radar data" ));
			throw SC_INVALID_DATA;

		}

		// tell the object we now have some radar data
		m_object->friend_setRadarData( this );

	}

	// color
	xfer->xferColor( &m_color );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void RadarObject::loadPostProcess()
{

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Radar::Radar()
{

	m_radarWindow = nullptr;
	m_objectList = nullptr;
	m_localObjectList = nullptr;
	std::fill(m_radarHidden, m_radarHidden + ARRAY_SIZE(m_radarHidden), false);
	std::fill(m_radarForceOn, m_radarForceOn + ARRAY_SIZE(m_radarForceOn), false);
	m_terrainAverageZ = 0.0f;
	m_waterAverageZ = 0.0f;
	m_xSample = 0.0f;
	m_ySample = 0.0f;
	m_mapExtent.lo.x = 0.0f;
	m_mapExtent.lo.y = 0.0f;
	m_mapExtent.lo.z = 0.0f;
	m_mapExtent.hi.x = 0.0f;
	m_mapExtent.hi.y = 0.0f;
	m_mapExtent.hi.z = 0.0f;
	m_queueTerrainRefreshFrame = 0;

	// clear the radar events
	clearAllEvents();

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Radar::~Radar()
{

	// delete list resources
	deleteListResources();

}

//-------------------------------------------------------------------------------------------------
/** Clear all radar events */
//-------------------------------------------------------------------------------------------------
void Radar::clearAllEvents()
{

	// set next free index to the first one
	m_nextFreeRadarEvent = 0;
	m_lastRadarEvent = -1;

	// zero out all data
	for( Int i = 0; i < MAX_RADAR_EVENTS; ++i )
	{

		m_event[ i ].type = RADAR_EVENT_INVALID;
		m_event[ i ].active = FALSE;
		m_event[ i ].createFrame = 0;
		m_event[ i ].dieFrame = 0;
		m_event[ i ].fadeFrame = 0;
		m_event[ i ].color1.red = 0;
		m_event[ i ].color1.green = 0;
		m_event[ i ].color1.blue = 0;
		m_event[ i ].color2.red = 0;
		m_event[ i ].color2.green = 0;
		m_event[ i ].color2.blue = 0;
		m_event[ i ].worldLoc.x = 0.0f;
		m_event[ i ].worldLoc.y = 0.0f;
		m_event[ i ].worldLoc.z = 0.0f;
		m_event[ i ].radarLoc.x = 0;
		m_event[ i ].radarLoc.y = 0;
		m_event[ i ].soundPlayed = FALSE;

	}

}

//-------------------------------------------------------------------------------------------------
/** Reset radar data */
//-------------------------------------------------------------------------------------------------
void Radar::reset()
{

	// delete list resources
	deleteListResources();

	// clear all events
	clearAllEvents();

	// TheSuperHackers @todo Reset m_radarHidden?
	//std::fill(m_radarHidden, m_radarHidden + ARRAY_SIZE(m_radarHidden), false);

	// stop forcing the radar on
	std::fill(m_radarForceOn, m_radarForceOn + ARRAY_SIZE(m_radarForceOn), false);

}

//-------------------------------------------------------------------------------------------------
/** Radar per frame update */
//-------------------------------------------------------------------------------------------------
void Radar::update()
{
	Int i;
	UnsignedInt thisFrame = TheGameLogic->getFrame();

	//
	// traverse the radar event list, if an event has a creationFrame it means that it
	// exists ... check to see if it's time for that event to die and if so, just clear
	// out the active status
	//
	for( i = 0; i < MAX_RADAR_EVENTS; i++ )
	{

		if( m_event[ i ].active == TRUE && m_event[ i ].createFrame &&
				thisFrame > m_event[ i ].dieFrame )
			m_event[ i ].active = FALSE;

	}

	// see if we should refresh the terrain
	if( m_queueTerrainRefreshFrame != 0 &&
			TheGameLogic->getFrame() - m_queueTerrainRefreshFrame > RADAR_QUEUE_TERRAIN_REFRESH_DELAY )
	{

		// refresh the terrain
		refreshTerrain( TheTerrainLogic );

	}

}

//-------------------------------------------------------------------------------------------------
/** Reset the radar for the new map data being given to it */
//-------------------------------------------------------------------------------------------------
void Radar::newMap( TerrainLogic *terrain )
{

	// keep a pointer for our radar window
	Int id = NAMEKEY( "ControlBar.wnd:LeftHUD" );
	m_radarWindow = TheWindowManager->winGetWindowFromId( nullptr, id );
	DEBUG_ASSERTCRASH( m_radarWindow, ("Radar::newMap - Unable to find radar game window") );

	// reset all the data in the radar
	reset();

	// get the extents of the new map
	terrain->getExtent( &m_mapExtent );

	// we will sample at these intervals across the map
	m_xSample = m_mapExtent.width() / static_cast<float>(RADAR_CELL_WIDTH);
	m_ySample = m_mapExtent.height() / static_cast<float>(RADAR_CELL_HEIGHT);

	// find the "middle" height for the terrain (most used value) and water table
	Int x, y;
	Int terrainSamples = 0, waterSamples = 0;

	m_terrainAverageZ = 0.0f;
	m_waterAverageZ = 0.0f;
	Coord3D worldPoint;

	// since we're averaging let's skip every second sample...
	worldPoint.y=0;
	for( y = 0; y < RADAR_CELL_HEIGHT; y+=2, worldPoint.y+=2.0*m_ySample )
	{
		worldPoint.x=0;
		for( x = 0; x < RADAR_CELL_WIDTH; x+=2, worldPoint.x+=2.0*m_xSample )
		{
			// don't use this, we don't really need the
			// Z position by this function... radarToWorld( &radarPoint, &worldPoint );
			// and this is done by isUnderwater anyway: z = terrain->getGroundHeight( worldPoint.x, worldPoint.y );
			Real z,waterZ;
			if( terrain->isUnderwater( worldPoint.x, worldPoint.y, &waterZ, &z ) )
			{
				m_waterAverageZ += z;
				waterSamples++;
			}
			else
			{
				m_terrainAverageZ += z;
				terrainSamples++;
			}
		}
	}

	// avoid divide by zeros
	if( terrainSamples == 0 )
		terrainSamples = 1;
	if( waterSamples == 0 )
		waterSamples = 1;

	// compute averages
	m_terrainAverageZ = m_terrainAverageZ / INT_TO_REAL( terrainSamples );
	m_waterAverageZ = m_waterAverageZ / INT_TO_REAL( waterSamples );

}

//-------------------------------------------------------------------------------------------------
/** Add an object to the radar list.  The object will be sorted in the list to be grouped
	* using it's radar priority */
//-------------------------------------------------------------------------------------------------
Bool Radar::addObject( Object *obj )
{

	// get the radar priority for this object
	RadarPriorityType newPriority = obj->getRadarPriority();
	if( isPriorityVisible( newPriority ) == FALSE )
		return FALSE;

	// if this object is on the radar, remove it in favor of the new add
	RadarObject **list;
	RadarObject *newObj;

	// sanity
	DEBUG_ASSERTCRASH( obj->friend_getRadarData() == nullptr,
										 ("Radar: addObject - non null radar data for '%s'",
										 obj->getTemplate()->getName().str()) );

	// allocate a new object
	newObj = newInstance(RadarObject);

	// set the object data
	newObj->friend_setObject( obj );

	// set color for this object on the radar
	assignObjectColorToRadarObject( newObj, obj );

	// set a chunk of radar data in the object
	obj->friend_setRadarData( newObj );

	//
	// we will put this on either the local object list for objects that belong to the
	// local player, or on the regular object list for all other objects
	//
	if( obj->isLocallyControlled() )
	{
		list = &m_localObjectList;
	}
	else
	{
		list = &m_objectList;
	}

	// link object to master list at the head of it's priority section
	linkRadarObject(newObj, list);

	return TRUE;
}

//-------------------------------------------------------------------------------------------------
/** Try to delete an object from a specific list */
//-------------------------------------------------------------------------------------------------
Bool Radar::deleteFromList( Object *obj, RadarObject **list )
{
	RadarObject *radarObject, *prevObject = nullptr;

	// find the object in list
	for( radarObject = *list; radarObject; radarObject = radarObject->friend_getNext() )
	{

		if( radarObject->friend_getObject() == obj )
		{

			// unlink the object from list
			if( prevObject == nullptr )
				*list = radarObject->friend_getNext();  // removing head of list
			else
				prevObject->friend_setNext( radarObject->friend_getNext() );

			// set the object radar data to null
			obj->friend_setRadarData( nullptr );

			// delete the object instance
			deleteInstance(radarObject);

			// all done, object found and deleted
			return TRUE;

		}

		// save this object as previous one encountered in the list
		prevObject = radarObject;

	}

	// object was not found in this list
	return FALSE;

}

//-------------------------------------------------------------------------------------------------
/** Remove an object from the radar, the object may reside in any list */
//-------------------------------------------------------------------------------------------------
Bool Radar::removeObject( Object *obj )
{

	// sanity
	if( obj->friend_getRadarData() == nullptr )
		return FALSE;

	if( deleteFromList( obj, &m_localObjectList ) == TRUE )
		return TRUE;
	else if( deleteFromList( obj, &m_objectList ) == TRUE )
		return TRUE;
	else
	{
		DEBUG_CRASH( ("Radar: Tried to remove object '%s' which was not found",
											 obj->getTemplate()->getName().str()) );
		return FALSE;
	}

}

//-------------------------------------------------------------------------------------------------
/** Translate a 2D spot on the radar (from (0,0) to (RADAR_CELL_WIDTH,RADAR_CELL_HEIGHT)
	* to a 3D spot in the world. Does not determine Z value!
	* Return TRUE if the radar points translates to a valid world position
	* Return FALSE if the radar point is not a valid world position */
//-------------------------------------------------------------------------------------------------
Bool Radar::radarToWorld2D( const ICoord2D *radar, Coord3D *world )
{
	Int x, y;

	// sanity
	if( radar == nullptr || world == nullptr )
		return FALSE;

	// get the coords
	x = radar->x;
	y = radar->y;

	// more sanity
	if( x < 0 )
		x = 0;
	if( x >= RADAR_CELL_WIDTH )
		x = RADAR_CELL_WIDTH - 1;
	if( y < 0 )
		y = 0;
	if( y >= RADAR_CELL_HEIGHT )
		y = RADAR_CELL_HEIGHT - 1;

	// translate to world
	world->x = x * m_xSample;
	world->y = y * m_ySample;
  return TRUE;
}

//-------------------------------------------------------------------------------------------------
/** Translate a 2D spot on the radar (from (0,0) to (RADAR_CELL_WIDTH,RADAR_CELL_HEIGHT)
	* to a 3D spot in the world on the terrain
	* Return TRUE if the radar points translates to a valid world position
	* Return FALSE if the radar point is not a valid world position */
//-------------------------------------------------------------------------------------------------
Bool Radar::radarToWorld( const ICoord2D *radar, Coord3D *world )
{
  if (!radarToWorld2D(radar,world))
    return FALSE;

	// find the terrain height here
	world->z = TheTerrainLogic->getGroundHeight( world->x, world->y );

	return TRUE;  // valid translation

}

//-------------------------------------------------------------------------------------------------
/** Translate a point in the world to the 2D radar (x,y)
	* Return TRUE if the world point successfully translates to a radar point
	* Return FALSE if world point is a bogus off the map position */
//-------------------------------------------------------------------------------------------------
Bool Radar::worldToRadar( const Coord3D *world, ICoord2D *radar )
{

	// sanity
	if( world == nullptr || radar == nullptr )
		return FALSE;

	// sanity check the world position
//	if( world->x < m_mapExtent.lo.x || world->x > m_mapExtent.hi.x ||
//			world->y < m_mapExtent.lo.y || world->y > m_mapExtent.hi.y )
//		return FALSE;
	// This is actually an insanity check.  Nobody uses the return value, so this just leaves garbage in the
	// return pointer.  The reason the question gets asked is there are 60 partition cells to 128 radar cells
	// (for example), and the radar wants to draw a horizontal line.  This line ends up three pixels long
	// at the right side, so the radar gives up and doesn't draw the middle one.
	// We bind to on radar anyway, and we only ask if we are on radar, so don't intentionally add edge weirdness.

	// convert
	radar->x = world->x / m_xSample;
	radar->y = world->y / m_ySample;

	// keep it in bounds
	if( radar->x < 0 )
		radar->x = 0;
	if( radar->x >= RADAR_CELL_WIDTH )
		radar->x = RADAR_CELL_WIDTH - 1;
	if( radar->y < 0 )
		radar->y = 0;
	if( radar->y >= RADAR_CELL_HEIGHT )
		radar->y = RADAR_CELL_HEIGHT - 1;

	return TRUE;  // valid translation

}

// ------------------------------------------------------------------------------------------------
/** Translate an actual pixel location (relative pixel with (0,0) being the top left of
	* the radar area) to the "logical" radar coords that would cover the entire area of display
	* on the screen.  This is needed because some maps are "long" or "tall" and need a translation
	* to any radar image that has been scaled to preserve the map aspect ratio */
// ------------------------------------------------------------------------------------------------
Bool Radar::localPixelToRadar( const ICoord2D *pixel, ICoord2D *radar )
{

	// sanity
	if( pixel == nullptr || radar == nullptr )
		return FALSE;

	// get window size of the radar
	ICoord2D size;
	m_radarWindow->winGetSize( &size.x, &size.y );

	//
	// act like we're going to draw and find the aspect ratio adjusted points of the
	// terrain radar positions
	//
	ICoord2D start = { 0, 0 };
	ICoord2D ul, lr;
	findDrawPositions( start.x, start.y, size.x, size.y, &ul, &lr );

	// get the scaled width and height
	Int scaledWidth = lr.x - ul.x;
	Int scaledHeight = lr.y - ul.y;

	// if the pixel is outsize of the adjusted radar area there are no logical coords
	if( pixel->x < ul.x || pixel->x > lr.x ||
			pixel->y < ul.y || pixel->y > lr.y )
		return FALSE;

	if( scaledWidth >= scaledHeight )
	{

		// just normal conversion from full stretched to radar cells
		radar->x = (pixel->x - ul.x)* RADAR_CELL_WIDTH / scaledWidth;

		// conversion for scaled Y direction in map
		radar->y = REAL_TO_INT( ((pixel->y - ul.y) / INT_TO_REAL( scaledHeight )) * size.y );

		//
		// radar->y now refers to a point that was "as if" the map was square, translate to radar
		// note that y is inverted to have the radar align with the world (+x = right, -y = down)
		//
		radar->y = (size.y - radar->y) * RADAR_CELL_HEIGHT / size.y;


	}
	else
	{

		// conversion for scaled Y direction in map
		radar->x = REAL_TO_INT( ((pixel->x - ul.x) / INT_TO_REAL( scaledWidth )) * size.x );

		// radar->x now refers to a point that was "as if" the map was square, translate to radar
		radar->x = radar->x * RADAR_CELL_WIDTH / size.x;

		//
		// just normal conversion from full stretched to radar cells, note that y is inverted
		// to have the radar align with the world (+x = right, -y = down)
		//
		radar->y = (size.y - pixel->y) * RADAR_CELL_HEIGHT / size.y;

	}

	return TRUE;

}

// ------------------------------------------------------------------------------------------------
/** Translate a screen mouse position to world coords if the screen position is within
	* the radar window and that spot in the radar corresponds to a point in the world */
// ------------------------------------------------------------------------------------------------
Bool Radar::screenPixelToWorld( const ICoord2D *pixel, Coord3D *world )
{

	// sanity
	if( pixel == nullptr || world == nullptr )
		return FALSE;

	// if we have no radar window can't do the conversion
	if( m_radarWindow == nullptr )
		return FALSE;

	// translate pixel coords to local pixel coords relative to the radar window
	ICoord2D localPixel;
	ICoord2D screenPos;
	m_radarWindow->winGetScreenPosition( &screenPos.x, &screenPos.y );
	localPixel.x = pixel->x - screenPos.x;
	localPixel.y = pixel->y - screenPos.y;

	// translate local pixel to radar
	ICoord2D radar;
	if( localPixelToRadar( &localPixel, &radar ) == FALSE )
		return FALSE;

	// translate radar to world
	return radarToWorld( &radar, world );

}

// ------------------------------------------------------------------------------------------------
/** Given the pixel coordinates, see if there is an object that is exactly in this
	* spot represented on the radar */
// ------------------------------------------------------------------------------------------------
Object *Radar::objectUnderRadarPixel( const ICoord2D *pixel )
{

	// sanity
	if( pixel == nullptr )
		return nullptr;

	// convert pixel location to radar logical radar location
	ICoord2D radar;
	if( localPixelToRadar( pixel, &radar ) == FALSE )
		return nullptr;

	// object we will return
	Object *obj = nullptr;

	//
	// scan the objects on the radar list and return any object that maps its world location
	// to the radar location
	//

	// search the local object list
	obj = searchListForRadarLocationMatch( m_localObjectList, &radar );

	// search all other objects if not found
	if( obj == nullptr )
		obj = searchListForRadarLocationMatch( m_objectList, &radar );

	// return the object found (if any)
	return obj;

}

// ------------------------------------------------------------------------------------------------
/** Search the object list for an object that maps to the given logical radar coords */
// ------------------------------------------------------------------------------------------------
Object *Radar::searchListForRadarLocationMatch( RadarObject *listHead, ICoord2D *radarMatch )
{

	// sanity
	if( listHead == nullptr || radarMatch == nullptr )
		return nullptr;

	// scan the list
	RadarObject *radarObject;
	ICoord2D radar;
	for( radarObject = listHead; radarObject; radarObject = radarObject->friend_getNext() )
	{

		// get object
		Object *obj = radarObject->friend_getObject();

		// sanity
		if( obj == nullptr )
		{

			DEBUG_CRASH(( "Radar::searchListForRadarLocationMatch - null object encountered in list" ));
			continue;

		}

		// convert object position to logical radar
		worldToRadar( obj->getPosition(), &radar );

		// see if this matches our match radar location
		if( radar.x >= radarMatch->x - 1 &&
				radar.x <= radarMatch->x + 1 &&
				radar.y >= radarMatch->y - 1 &&
				radar.y <= radarMatch->y + 1 )
			return obj;

	}

	// no match found
	return nullptr;

}

// ------------------------------------------------------------------------------------------------
/** Given the RELATIVE SCREEN start X and Y, the width and height of the area to draw the whole
	* radar in, compute what the upper left (ul) and lower right (lr) local coordinates are
	* that represent the actual terrain image part of the radar that will preserve the
	* aspect ratio of the map */
// ------------------------------------------------------------------------------------------------
void Radar::findDrawPositions( Int startX, Int startY, Int width, Int height,
															 ICoord2D *ul, ICoord2D *lr )
{

	Real ratioWidth;
	Real ratioHeight;
	Coord2D radar;
	ratioWidth = m_mapExtent.width()/(width * 1.0f);
	ratioHeight = m_mapExtent.height()/(height* 1.0f);

	if( ratioWidth >= ratioHeight)
	{
		radar.x = m_mapExtent.width() / ratioWidth;
		radar.y = m_mapExtent.height()/ ratioWidth;
		ul->x = 0;
		ul->y = (height - radar.y) / 2.0f;
		lr->x = radar.x;
		lr->y = height - ul->y;
	}
	else
	{
		radar.x = m_mapExtent.width() / ratioHeight;
		radar.y = m_mapExtent.height()/ ratioHeight;
		ul->x = (width - radar.x ) / 2.0f;
		ul->y = 0;
		lr->x = width - ul->x;
		lr->y = radar.y;
	}
/*

	if( m_mapExtent.width() > m_mapExtent.height() )
	{

		//
		// +---------------+
		// |               |
		// |               |
		// +---------------+
		// |   map area    |
		// +---------------+
		// |               |
		// |               |
		// +---------------+
		//
		ul->x = 0;
		ul->y = (height - (m_mapExtent.height() / m_mapExtent.width() * height)) / 2.0f;
		lr->x = width;
		lr->y = height - ul->y;

	}
	else if(  m_mapExtent.height() > m_mapExtent.width() )
	{

		// +-----+---+-----+
		// |     | m |     |
		// |     | a |     |
		// |     | p |     |
		// |     |   |     |
		// |     | a |     |
		// |     | r |     |
		// |     | e |     |
		// |     | a |     |
		// +-----+---+-----+
		//

		ul->x = (width - (m_mapExtent.width() / m_mapExtent.height() * width)) / 2.0f;
		ul->y = 0;
		lr->x = width - ul->x;
		lr->y = height;

	}
	else
	{

		ul->x = 0;
		ul->y = 0;
		lr->x = width;
		lr->y = height;

	}
*/

	// make them pixel positions
	ul->x += startX;
	ul->y += startY;
	lr->x += startX;
	lr->y += startY;

}

//-------------------------------------------------------------------------------------------------
/** Radar color lookup table */
//-------------------------------------------------------------------------------------------------
struct RadarColorLookup
{
	RadarEventType event;
	RGBAColorInt color1;
	RGBAColorInt color2;
};
static RadarColorLookup radarColorLookupTable[] =
{
	/*      Radar Event													Color 1									 Color 2       */
	{ RADAR_EVENT_CONSTRUCTION,					{ 128, 128, 255, 255 },  {  128, 255, 255, 255 } },
	{ RADAR_EVENT_UPGRADE,							{ 128,   0,  64, 255 },  {  255, 185, 220, 255 } },
	{ RADAR_EVENT_UNDER_ATTACK,					{ 255,   0,   0, 255 },  {  255, 128, 128, 255 } },
	{ RADAR_EVENT_INFORMATION,					{ 255, 255,   0, 255 },  {  255, 255, 128, 255 } },
	{ RADAR_EVENT_BEACON_PULSE,					{ 255, 255,   0, 255 },  {  255, 255, 128, 255 } },
	{ RADAR_EVENT_INFILTRATION,					{   0, 255, 255, 255 },  {  128, 255, 255, 255 } },
	{ RADAR_EVENT_BATTLE_PLAN,					{ 255, 255, 255, 255 },	 {  255, 255, 255, 255 } },
	{ RADAR_EVENT_STEALTH_DISCOVERED,		{   0, 255,   0, 255 },	 {    0, 128,   0, 255 } },
	{ RADAR_EVENT_STEALTH_NEUTRALIZED,	{   0, 255,   0, 255 },	 {    0, 128,   0, 255 } },
	{ RADAR_EVENT_FAKE,									{   0,	 0,   0,	 0 },	 {    0,	 0,   0,   0 } },
	{ RADAR_EVENT_INVALID,							{   0,   0,   0,   0 },  {    0,   0,   0,   0 } }
};
static_assert(ARRAY_SIZE(radarColorLookupTable) == RADAR_EVENT_NUM_EVENTS, "Incorrect array size");

//-------------------------------------------------------------------------------------------------
/** Create a new radar event */
//-------------------------------------------------------------------------------------------------
void Radar::createEvent( const Coord3D *world, RadarEventType type, Real secondsToLive )
{

	// sanity
	if( world == nullptr )
		return;

	// lookup the colors we are to used based on the event
	RGBAColorInt color[ 2 ];
	Int i = 0;
	for( ; radarColorLookupTable[ i ].event != RADAR_EVENT_INVALID; ++i )
	{

		if( radarColorLookupTable[ i ].event == type )
		{

			color[ 0 ] = radarColorLookupTable[ i ].color1;
			color[ 1 ] = radarColorLookupTable[ i ].color2;
			break;

		}

	}

	// check for no match found in color table
	if( radarColorLookupTable[ i ].event == RADAR_EVENT_INVALID )
	{
		static RGBAColorInt color1 = { 255, 255, 255, 255 };
		static RGBAColorInt color2 = { 255, 255, 255, 255 };

		DEBUG_CRASH(( "Radar::createEvent - Event not found in color table, using default colors" ));
		color[ 0 ] = color1;
		color[ 1 ] = color2;

	}

	// call the internal method to create the event with these colors
	internalCreateEvent( world, type, secondsToLive, &color[ 0 ], &color[ 1 ] );

}

// ------------------------------------------------------------------------------------------------
/** Create radar event using a specific colors from the player */
// ------------------------------------------------------------------------------------------------
void Radar::createPlayerEvent( Player *player, const Coord3D *world,
															 RadarEventType type, Real secondsToLive )
{

	// sanity
	if( player == nullptr || world == nullptr )
		return;

	// figure out the two colors we should use
	Color c;
	UnsignedByte r, g, b, a;
	RGBAColorInt color[ 2 ];

	// color 1
	c = player->getPlayerColor();
	GameGetColorComponents( c, &r, &g, &b, &a );
	color[ 0 ].red = r;
	color[ 0 ].green = g;
	color[ 0 ].blue = b;
	color[ 0 ].alpha = a;

	// the second will be a darker color 1
	Real darkScale = 0.75f;
	color[ 1 ] = color[ 0 ];
	color[ 1 ].red -= REAL_TO_INT( color[ 0 ].red * darkScale );
	if( color[ 1 ].red < 0 )
		color[ 1 ].red = 0;
	color[ 1 ].green -= REAL_TO_INT( color[ 0 ].green * darkScale );
	if( color[ 1 ].green < 0 )
		color[ 1 ].green = 0;
	color[ 1 ].blue -= REAL_TO_INT( color[ 0 ].blue * darkScale );
	if( color[ 1 ].blue < 0 )
		color[ 1 ].blue = 0;

	// create the events using these colors
	internalCreateEvent( world, type, secondsToLive, &color[ 0 ], &color[ 1 ] );

}

//-------------------------------------------------------------------------------------------------
/** Create a new radar event */
//-------------------------------------------------------------------------------------------------
void Radar::internalCreateEvent( const Coord3D *world, RadarEventType type, Real secondsToLive,
																 const RGBAColorInt *color1, const RGBAColorInt *color2 )
{
	static Real secondsBeforeDieToFade = 0.5f;  ///< this many seconds before we hit the die frame we start to fade away

	// sanity
	if( world == nullptr || color1 == nullptr || color2 == nullptr )
		return;

	// translate the world coord to radar coords
	ICoord2D radar;
	worldToRadar( world, &radar );

	// add to the list of radar events
	m_event[ m_nextFreeRadarEvent ].type = type;
	m_event[ m_nextFreeRadarEvent ].active = TRUE;
	m_event[ m_nextFreeRadarEvent ].createFrame = TheGameLogic->getFrame();
	m_event[ m_nextFreeRadarEvent ].dieFrame = TheGameLogic->getFrame() + static_cast<float>(LOGICFRAMES_PER_SECOND) * secondsToLive;
	m_event[ m_nextFreeRadarEvent ].fadeFrame = m_event[ m_nextFreeRadarEvent ].dieFrame - static_cast<float>(LOGICFRAMES_PER_SECOND) * secondsBeforeDieToFade;
	m_event[ m_nextFreeRadarEvent ].color1 = *color1;
	m_event[ m_nextFreeRadarEvent ].color2 = *color2;
	m_event[ m_nextFreeRadarEvent ].worldLoc = *world;
	m_event[ m_nextFreeRadarEvent ].radarLoc = radar;
	m_event[ m_nextFreeRadarEvent ].soundPlayed = FALSE;

	// record the index of this, our "last" radar event.
	if ( type != RADAR_EVENT_BEACON_PULSE )
		m_lastRadarEvent = m_nextFreeRadarEvent;

	//
	// increment the next radar event index, wrapping to the beginning.  If we ever have so many
	// events that they fill up the buffer the oldest ones will just drop off, eh ... should be fine.
	//
	m_nextFreeRadarEvent++;
	if( m_nextFreeRadarEvent >= MAX_RADAR_EVENTS )
		m_nextFreeRadarEvent = 0;

}

//-------------------------------------------------------------------------------------------------
/** Get the last event position, if any.
	* Returns TRUE if event was found
	* Returns FALSE if there is no "last event" */
//-------------------------------------------------------------------------------------------------
Bool Radar::getLastEventLoc( Coord3D *eventPos )
{

	// if we have an index for the last event, one was present
	if( m_lastRadarEvent != -1 )
	{

		if( eventPos )
			*eventPos = m_event[ m_lastRadarEvent ].worldLoc;
		return TRUE;

	}

	return FALSE;  // no last event

}

// ------------------------------------------------------------------------------------------------
/** Try to create a radar event for "we're under attack".  This will be called every time
	* actual damage is dealt to an object that the player owns that shows up on the radar.
	* We don't want to create under attack events every time we are damaged and we also want
	* to limit them based on time and local area of other recent attack events */
// ------------------------------------------------------------------------------------------------
void Radar::tryUnderAttackEvent( const Object *obj )
{

	// sanity
	if( obj == nullptr )
		return;

	// try to create the event
	Bool eventCreated = tryEvent( RADAR_EVENT_UNDER_ATTACK, obj->getPosition() );

	// if event created, do some more feedback
	if( eventCreated )
	{

		TheControlBar->triggerRadarAttackGlow();
		//
		///@todo Should make an INI data driven table for radar event strings, and audio events
		//
		// UI feedback for being under attack (note that we display these messages and audio
		// queues even if we don't have a radar)
		//
		Player *player = rts::getObservedOrLocalPlayer();

		// create a message for the attack event
		if( obj->isKindOf( KINDOF_INFANTRY ) || obj->isKindOf( KINDOF_VEHICLE ) )
		{
			AudioEventRTS unitAttackSound;
			if( obj->isKindOf(KINDOF_HARVESTER) )
			{
				// display special message
				TheInGameUI->message( "RADAR:HarvesterUnderAttack" );

				// play special audio event
				unitAttackSound = TheAudio->getMiscAudio()->m_radarHarvesterUnderAttackSound;
			}
			else
			{
				// display message
				TheInGameUI->message( "RADAR:UnitUnderAttack" );

				// play audio event
				unitAttackSound = TheAudio->getMiscAudio()->m_radarStructureUnderAttackSound;
			}
			unitAttackSound.setPlayerIndex(player->getPlayerIndex());
			TheAudio->addAudioEvent( &unitAttackSound );

		}
		else if( obj->isKindOf( KINDOF_STRUCTURE ) && obj->isKindOf( KINDOF_MP_COUNT_FOR_VICTORY ) )
		{
			// play EVA. If its our object, play Base under attack.
			if (obj->getControllingPlayer()->isLocalPlayer())
				TheEva->setShouldPlay(EVA_BaseUnderAttack);
			else if (player->getRelationship(obj->getTeam()) == ALLIES)
				TheEva->setShouldPlay(EVA_AllyUnderAttack);

			// display message
			TheInGameUI->message( "RADAR:StructureUnderAttack" );

			// play audio event
			static AudioEventRTS structureAttackSound = TheAudio->getMiscAudio()->m_radarStructureUnderAttackSound;
			structureAttackSound.setPlayerIndex(player->getPlayerIndex());
			TheAudio->addAudioEvent( &structureAttackSound );

		}
		else
		{

			// display message
			TheInGameUI->message( "RADAR:UnderAttack" );

			// play audio event
			static AudioEventRTS underAttackSound = TheAudio->getMiscAudio()->m_radarStructureUnderAttackSound;
			underAttackSound.setPlayerIndex(player->getPlayerIndex());
			TheAudio->addAudioEvent( &underAttackSound );

		}

	}

}

// ------------------------------------------------------------------------------------------------
/** Try to create a radar event for "infiltration".
		This happens whenever a unit is hijacked, defected, converted to carbomb, hacked, or
		otherwise snuck into */
// ------------------------------------------------------------------------------------------------
void Radar::tryInfiltrationEvent( const Object *obj )
{

	//Sanity!
	if( !obj )
	{
		return;
	}

	Player *player = rts::getObservedOrLocalPlayer();

	// We should only be warned against infiltrations that are taking place against us.
	if( obj->getControllingPlayer() != player )
		return;

	// create the radar event
	createEvent( obj->getPosition(), RADAR_EVENT_INFILTRATION );

	//
	///@todo Should make an INI data driven table for radar event strings, and audio events
	//
	// UI feedback for being under attack (note that we display these messages and audio
	// queues even if we don't have a radar)
	//

	// display message
	TheInGameUI->message( "RADAR:Infiltration" );

	// play audio event
	static AudioEventRTS infiltrationWarningSound = TheAudio->getMiscAudio()->m_radarInfiltrationSound;
	infiltrationWarningSound.setPlayerIndex(player->getPlayerIndex());
	TheAudio->addAudioEvent( &infiltrationWarningSound );

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
Bool Radar::tryEvent( RadarEventType event, const Coord3D *pos )
{

	// sanity
	if( event <= RADAR_EVENT_INVALID || event >= RADAR_EVENT_NUM_EVENTS || pos == nullptr )
		return FALSE;

	// see if there was an event of this type within the given range within the given time
	UnsignedInt currentFrame = TheGameLogic->getFrame();
	const Real closeEnoughDistanceSq = 250.0f * 250.0f;
	const UnsignedInt framesBetweenEvents = LOGICFRAMES_PER_SECOND * 10;
	//

	// see if there was any matching radar event within range of this one
	// that wasn't too long ago, if there was we won't create another and get outta here
	//
	for( Int i = 0; i < MAX_RADAR_EVENTS; ++i )
	{

		// only pay attention to under attack events
		if( m_event[ i ].type == event )
		{

			// get distance from our new event location to this event location in 2D
			const Real distSquared = sqr(m_event[ i ].worldLoc.x - pos->x) + sqr(m_event[ i ].worldLoc.y - pos->y);

			Bool isClose = distSquared <= closeEnoughDistanceSq;
			#if PRESERVE_RADAR_WARNING_SUPPRESSION
				// TheSuperHackers @tweak Preserve retail map-wide suppression for under attack events
				// because otherwise they trigger way too frequent from cargo planes.
				isClose |= (event == RADAR_EVENT_UNDER_ATTACK);
			#endif

			if( isClose )
			{

				// finally only reject making a new event of this existing one is "recent enough"
				if( currentFrame - m_event[ i ].createFrame < framesBetweenEvents )
					return FALSE;  // reject it

			}

		}

	}

	// if we got here then we want to create a new event
	createEvent( pos, event );

	// return TRUE for successfully created event
	return TRUE;

}


// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void Radar::refreshTerrain( TerrainLogic *terrain )
{

	// no future queue is valid now
	m_queueTerrainRefreshFrame = 0;

}

// ------------------------------------------------------------------------------------------------
/** Queue a refresh of the radar terrain, we have this so that if there is code that
	* rapidly needs to refresh the radar, it should use this so we aren't continually
	* rebuilding the radar graphic because that process is slow.  If you need to update
	* the terrain on the radar immediately use refreshTerrain() */
// ------------------------------------------------------------------------------------------------
void Radar::queueTerrainRefresh()
{

	//
	// we just simply overwrite the frame we have recorded for a radar refresh.  If there was
	// already one there, it's simply just forgotten and whatever changes we wanted to see
	// with that refresh will have to wait until enough time has passed to show these
	// changes as well.  why you ask ... well, because if we're calling this in close enough
	// proximity for us to overwrite something, we're changing the terrain features
	// quite often and can't afford the expense of rebuilding the radar visual
	//
	m_queueTerrainRefreshFrame = TheGameLogic->getFrame();

}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void Radar::crc( Xfer *xfer )
{

}

// ------------------------------------------------------------------------------------------------
/** Xfer a radar object list given the head pointer as a parameter
	* Version Info;
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
static void xferRadarObjectList( Xfer *xfer, RadarObject **head )
{
	RadarObject *radarObject;

	// sanity
	DEBUG_ASSERTCRASH( head != nullptr, ("xferRadarObjectList - Invalid parameters" ));

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// write could of objects in list
	UnsignedShort count = 0;
	for( radarObject = *head; radarObject; radarObject = radarObject->friend_getNext() )
		count++;
	xfer->xferUnsignedShort( &count );

	// xfer the list data
	if( xfer->getXferMode() == XFER_SAVE )
	{

		// save each object
		for( radarObject = *head; radarObject; radarObject = radarObject->friend_getNext() )
		{

			// save this object
			xfer->xferSnapshot( radarObject );

		}

	}
	else
	{

		// the list should be empty at this point as we are loading it as a whole
		if( *head != nullptr )
		{
#if 1
			// srj sez: yeah, it SHOULD be, but a few rogue things come into existence (via their ctor) preloaded
			// with stuff (eg, "CabooseFullOfTerrorists"). we immediately destroy 'em, but they don't go away just yet.
			// so just ignore 'em if possible.
			for( radarObject = *head; radarObject; radarObject = radarObject->friend_getNext() )
			{
				if (!radarObject->friend_getObject()->isDestroyed())
				{
					DEBUG_CRASH(( "xferRadarObjectList - List head should be null, or contain only destroyed objects" ));
					throw SC_INVALID_DATA;
				}
			}
#else
			DEBUG_CRASH(( "xferRadarObjectList - List head should be null, but isn't" ));
			throw SC_INVALID_DATA;
#endif
		}

		// read each element
		for( UnsignedShort i = 0; i < count; ++i )
		{

			// alloate a new radar object
			radarObject = newInstance(RadarObject);

			// link to the end of the list
			if( *head == nullptr )
				*head = radarObject;
			else
			{

				RadarObject *other;
				for( other = *head; other->friend_getNext() != nullptr; other = other->friend_getNext() )
				{
				}

				// set the end of the list to point to the new object
				other->friend_setNext( radarObject );

			}

			// load the data
			xfer->xferSnapshot( radarObject );

		}

	}

}

// ------------------------------------------------------------------------------------------------
/** Xfer Method
	* Version Info:
	* 1: Initial version
	* 2: TheSuperHackers @tweak Serialize m_radarHidden, m_radarForceOn for each player
	*/
// ------------------------------------------------------------------------------------------------
void Radar::xfer( Xfer *xfer )
{

	// version
#if RETAIL_COMPATIBLE_XFER_SAVE
	XferVersion currentVersion = 1;
#else
	XferVersion currentVersion = 2;
#endif
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );


	if (version <= 1)
	{
		const Int localPlayerIndex = ThePlayerList->getLocalPlayer()->getPlayerIndex();
		Bool value;

		// radar hidden
		value = m_radarHidden[localPlayerIndex];
		xfer->xferBool( &value );
		m_radarHidden[localPlayerIndex] = value;

		// radar force on
		value = m_radarForceOn[localPlayerIndex];
		xfer->xferBool( &value );
		m_radarForceOn[localPlayerIndex] = value;
	}
	else
	{
		static_assert(sizeof(m_radarHidden) == 16, "Increase version if size changes");
		xfer->xferUser(&m_radarHidden, sizeof(m_radarHidden));

		static_assert(sizeof(m_radarForceOn) == 16, "Increase version if size changes");
		xfer->xferUser(&m_radarForceOn, sizeof(m_radarForceOn));
	}

	// save our local object list
	xferRadarObjectList( xfer, &m_localObjectList );

	// save the regular object list
	xferRadarObjectList( xfer, &m_objectList );

	// save the radar event count and data
	UnsignedShort eventCountVerify = MAX_RADAR_EVENTS;
	UnsignedShort eventCount = eventCountVerify;
	xfer->xferUnsignedShort( &eventCount );
	if( eventCount != eventCountVerify )
	{

		DEBUG_CRASH(( "Radar::xfer - size of MAX_RADAR_EVENTS has changed, you must version this xfer method to accommodate the new array size.  Was '%d' and is now '%d'",
									eventCount, eventCountVerify ));
		throw SC_INVALID_DATA;

	}
	for( UnsignedShort i = 0; i < eventCount; ++i )
	{

		// xfer event data
		xfer->xferUser( &m_event[ i ].type, sizeof( RadarEventType ) );
		xfer->xferBool( &m_event[ i ].active );
		xfer->xferUnsignedInt( &m_event[ i ].createFrame );
		xfer->xferUnsignedInt( &m_event[ i ].dieFrame );
		xfer->xferUnsignedInt( &m_event[ i ].fadeFrame );
		xfer->xferRGBAColorInt( &m_event[ i ].color1 );
		xfer->xferRGBAColorInt( &m_event[ i ].color2 );
		xfer->xferCoord3D( &m_event[ i ].worldLoc );
		xfer->xferICoord2D( &m_event[ i ].radarLoc );
		xfer->xferBool( &m_event[ i ].soundPlayed );

	}

	// next event index
	xfer->xferInt( &m_nextFreeRadarEvent );

	// last event index
	xfer->xferInt( &m_lastRadarEvent );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void Radar::loadPostProcess()
{

	//
	// refresh the radar texture now that all the objects (specifically bridges) have
	// been loaded with their correct damage states from save game file
	//
	refreshTerrain( TheTerrainLogic );

}

// ------------------------------------------------------------------------------------------------
/** Is the priority type passed in a "visible" one that can show up on the radar */
// ------------------------------------------------------------------------------------------------
Bool Radar::isPriorityVisible( RadarPriorityType priority )
{

	switch( priority )
	{

		case RADAR_PRIORITY_INVALID:
		case RADAR_PRIORITY_NOT_ON_RADAR:
			return FALSE;

		default:
			return TRUE;

	}

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void Radar::linkRadarObject( RadarObject *newObj, RadarObject **list )
{
	if( *list == nullptr )
	{
		// trivial case, an empty list
		*list = newObj;
		return;
	}

	RadarPriorityType newPriority = newObj->friend_getObject()->getRadarPriority();
	RadarPriorityType prevPriority;
	RadarPriorityType currPriority;
	RadarObject *currObject;
	RadarObject *prevObject;
	RadarObject *nextObject;

	DEBUG_ASSERTCRASH(newObj->friend_getNext() == nullptr, ("newObj->friend_getNext is not null"));

	prevObject = nullptr;
	prevPriority = RADAR_PRIORITY_INVALID;
	for( currObject = *list; currObject; currObject = nextObject )
	{
		// get the next object
		nextObject = currObject->friend_getNext();

		// get the priority of this entry in the list (currPriority)
		currPriority = currObject->friend_getObject()->getRadarPriority();

		//
		// if there is no previous object, or the previous priority is less than the
		// our new priority, and the current object in the list has a priority
		// higher than our equal to our own we need to be inserted here
		//
		if( (prevObject == nullptr || prevPriority < newPriority ) && (currPriority >= newPriority) )
		{
			// insert into the list just ahead of currObject
			if( prevObject )
			{
				// the new entry next points to what the previous one used to point to
				newObj->friend_setNext( prevObject->friend_getNext() );

				// the previous one next now points to the new entry
				prevObject->friend_setNext( newObj );
			}
			else
			{
				// the new object next points to the current object
				newObj->friend_setNext( currObject );

				// new list head is now newObj
				*list = newObj;
			}
			break;
		}
		else if( nextObject == nullptr )
		{
			// at the end of the list, put object here
			currObject->friend_setNext( newObj );
		}

		// our current object is now the previous object
		prevObject = currObject;
		prevPriority = currPriority;
	}
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void Radar::assignObjectColorToRadarObject( RadarObject *radarObj, Object *obj )
{
	const Player *player = obj->getControllingPlayer();
	Player *clientPlayer = rts::getObservedOrLocalPlayer();
	Bool useIndicatorColor = true;

	if( obj->isKindOf( KINDOF_DISGUISER ) )
	{
		//Because we have support for disguised units pretending to be units from another
		//team, we need to intercept it here and make sure it's rendered appropriately
		//based on which client is rendering it.
		StealthUpdate *update = obj->getStealth();
		if( update )
		{
			if( update->isDisguised() )
			{
				Player *disguisedPlayer = ThePlayerList->getNthPlayer( update->getDisguisedPlayerIndex() );
				if( player->getRelationship( clientPlayer->getDefaultTeam() ) != ALLIES && clientPlayer->isPlayerActive() )
				{
					//Neutrals and enemies will see this disguised unit as the team it's disguised as.
					player = disguisedPlayer;
					if( player )
						useIndicatorColor = false;
				}
				//Otherwise, the color will show up as the team it really belongs to (already set above).
			}
		}
	}

	if( obj->getContain() )
	{
		// To handle Stealth garrison, ask containers what color they are drawing with to the local player.
		// Local is okay because radar display is not synced.
		player = obj->getContain()->getApparentControllingPlayer( clientPlayer );
		if( player )
			useIndicatorColor = false;
	}

	if( useIndicatorColor || (player == nullptr) )
	{
		radarObj->setColor( obj->getIndicatorColor() );
	}
	else
	{
		radarObj->setColor( player->getPlayerColor() );
	}
}
```
