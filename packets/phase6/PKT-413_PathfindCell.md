# Translation Packet `PKT-413`: `PathfindCell`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-413` *(bundle includes `PKT-414` `PathfindCellInfo`, `PKT-415` `PathfindCellList`, and `PKT-416` `PathfindConstants`)*
- **Work Package:** `WP-6.2` (Phase 6)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web` *(seeded with C++ Release oracle `docs/golden_vectors/PKT-413.json` & Rust skeletons)*
- **FFI Seam Pattern:** Core Spine (safe index-based `PathfindCellGrid` + `PathfindCellInfoPool`)
- **Allowed Rust Output Files:**
  - `crates/zh_logic/src/pathfinder/pathfind_cell_info.rs` *(remove `todo!` stubs)*
  - `crates/zh_logic/src/pathfinder/pathfind_cell_list.rs` *(remove `todo!` stubs)*
  - `crates/zh_logic/src/pathfinder/pathfind_cell.rs` *(remove `todo!` stubs)*
  - `crates/zh_logic/tests/golden_pkt413_pathfind_cell.rs` *(remove both `#[ignore]` attributes once implemented)*
  - `docs/PORT_MAP.md` *(regenerate with `python3 scripts/port_map.py --write`)*
- **Mandatory V&V Gates:** `V1, V2-Golden`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test -p zh_logic --test golden_pkt413_pathfind_cell && python3 scripts/port_map.py --write && cargo test --workspace && cargo check --workspace --all-targets
  ```

## Port Contract (read first)

1. **Compile Config & Runtime Mode Switch (`PathfindMode`):**
   - The C++ oracle (`docs/golden_vectors/PKT-413.json`) was generated with `RETAIL_COMPATIBLE_PATHFINDING=1`, `RETAIL_COMPATIBLE_PATHFINDING_ALLOCATION=1`, `USE_DETERMINISTIC_MATH=1`, `RETAIL_COMPATIBLE_CRC=0`, and `-URTS_DEBUG`.
   - Do **not** compile away fixed vs retail mode behind `#[cfg(feature = "retail-crc")]` — keep `PathfindMode::use_fixed_pathfinding: bool` (`false` = retail-compatible default, `true` = fixed mode) matching runtime `s_useFixedPathfinding` / `s_forceCleanCells` in `PathfindConstants.h/.cpp`!
   - `crates/zh_logic/src/pathfinder/pathfind_constants.rs` (`PKT-416`) already defines and tests all constants (`test_pkt413_oracle_structure_and_constants` is active and passes).

2. **`PathfindCellInfoPool` (`PKT-414`):**
   - Allocates `CELL_INFOS_TO_ALLOCATE` (`30_000`) entries linked via `next_free` (`first_free = Some(InfoIdx(0))`, `free_count = 30_000`, `is_free = true`).
   - `get_a_cell_info(cell, pos)` pops `first_free`, sets `is_free = false`, `cell = Some(cell)`, `pos`, and zeros every field (`next_open = None`, `prev_open = None`, `path_parent = None`, costs = `0`, IDs = `ObjectId::INVALID`, bool flags = `false`).
   - `release_a_cell_info(idx)` sets `is_free = true` and pushes onto `first_free` (`free_count += 1`).
   - `force_clean_pathfind_cell_infos()` loops `0..CELL_INFOS_TO_ALLOCATE - 1` (exact C++ `< CELL_INFOS_TO_ALLOCATE - 1` upper bound) clearing `next_open = None`, `prev_open = None`, `open = false`, `closed = false`.
   - When `PathfindCell::release_info()` runs: in `RETAIL_COMPATIBLE_PATHFINDING`, ALWAYS clear `info.path_parent = None` first (even if early-returning for `Obstacle` / `flags != NoUnits` / `aircraft_goal`!). Only free `info` if `cell_type != Obstacle && flags == NoUnits && !aircraft_goal && prev_open.is_none() && next_open.is_none() && !open && !closed`.

3. **Retail (`use_fixed_pathfinding = false`) vs Fixed (`use_fixed_pathfinding = true`) State Routing:**
   - **Obstacle state (`set_type_as_obstacle`, `remove_obstacle`, `set_type`, `obstacle_id`, `is_obstacle_*`, `is_obstruction_invalid`, `clear_obstruction`):**
     - `set_type_as_obstacle`: if `cell_type` is neither `Clear` nor `Impassable`, return `false`. If `obstacle.is_rubble`, set `cell_type = Rubble`, clear `cell` obstacle fields, and in retail mode set `info.obstacle_id = Invalid` and call `release_info()`, returning `true`. For non-rubble: set `cell.cell_type = Obstacle` and write `cell.obstacle_*`; in retail mode (`!use_fixed_pathfinding`), allocate `info` if missing (`return false` if pool exhausted) and ALSO mirror `id`, `is_fence`, `is_transparent` onto `info`!
     - `remove_obstacle`: always turns `Rubble -> Clear` first. In fixed mode checks `cell.obstacle_id == obstacle_id`; in retail mode requires `info` present with `info.obstacle_id == obstacle_id`, sets `info.obstacle_id = Invalid`, calls `release_info()`, THEN clears `cell.obstacle_*` (`cell_type = Clear`).
     - `set_type(t)`: in fixed mode refuses change (forces `Obstacle`) when `cell.obstacle_id != Invalid`; in retail mode refuses when `info.is_some_and(|i| i.obstacle_id != Invalid)`.
     - `clear_obstruction()`: clears `cell_type = Clear`, `cell.obstacle_id = Invalid`, `obstacle_is_fence = false`, `obstacle_is_transparent = false`.
     - `is_blocked_by_ally` / `set_blocked_by_ally`: in fixed mode reads/writes `cell.blocked_by_ally` (`Some(...)`); in retail mode reads/writes `info.blocked_by_ally` (`None` when `info` is absent!).

4. **Bitwise & Arithmetic Semantic Traps:**
   - **`water_ground` bitwise quirk (`PathfindCell.cpp:969`):** Literally mirrors `((target == Clear) && ((src as i32 & Water as i32) != 0)) || ((src == Clear) && ((target as i32 & Water as i32) != 0))`. Because `Water = 1`, `Rubble = 3`, and `BridgeImpassable = 5` also have bit `1` set and return `true` against `Clear`! Exact parity with `predicate_matrix`.
   - **`terrain` predicate:** Normalizes `Obstacle -> Clear` on both arguments before comparing.
   - **Widths & masks:** `set_zone(z)` masks `z & 0x3FFF`; `set_total_cost(c)` and `set_cost_so_far(c)` truncate `c as UnsignedShort` (`u16`); `get_total_cost_difference` returns `u32::MAX` if either cell lacks `info`, else `(a_total as i32 - b_total as i32).unsigned_abs()`.
   - **`start_pathfind(goal)`:** Zeros `next_open`, `prev_open`, `path_parent`, `cost_so_far = 0`, `total_cost = goal.map_or(0, |g| cost_to_goal(idx, g) as UnsignedShort)`, `closed = false`, and sets `open = !mode.use_fixed_pathfinding`.
   - **`cost_to_goal`:** Manhattan+half octile heuristic (`COST_ORTHOGONAL = 10`): `10 * max + (10 * min) / 2` (integer division!).
   - **`cost_to_hier_goal`:** If `!self.has_info()`, return `100_000`; otherwise compute `let dist_sq = (dx * dx + dy * dy) as f32;` and `zh_common::math::real_to_int_floor(10.0_f32 * zh_common::math::gm_sqrtf(dist_sq) + 0.5_f32) as UnsignedInt` (keep explicit `+ 0.5_f32`, NO `mul_add`!).
   - **`cost_so_far(parent)`:** `None => 0`. Orthogonal parent (`dx == 0 || dy == 0`) adds `COST_ORTHOGONAL` (`10`), diagonal adds `COST_DIAGONAL` (`14`); `+ 14` if `self.pinched`; then if `parent` has a `path_parent` with valid info, compare `dir = grand_pos - parent_pos` vs `prev_dir = parent_pos - self_pos` and add `4` (`dot > 0`), `8` (`dot == 0`), or `16` (`dot < 0`) on direction mismatch.
   - **`set_pos_unit` early return (`PathfindCell.cpp:328`):** When `unit_id != INVALID_ID` and `info.goal_unit_id != INVALID_ID && info.goal_unit_id == info.pos_unit_id`, return early WITHOUT modifying `pos_unit_id` or `flags`!

5. **Open & Closed List Ordering (`PathfindCellList` + `PathfindCell`):**
   - **Retail open sort (`!use_fixed_pathfinding`):** `forwardInsertionSortRetailCompatible` walks from `list.head` with `<=` (at most `PATHFIND_CELLS_PER_FRAME` steps), updates ONLY `list.head` and `next_open`/`prev_open`, and leaves `list.tail = None`! Equal costs preserve FIFO order.
   - **Fixed open sort (`use_fixed_pathfinding`):** Checks `list.can_reverse_sort` (`head` vs `tail` via strict `total_cost_difference(head, cur) > total_cost_difference(tail, cur)`); `forwardInsertionSort` uses `<=` on `next_open`, `reverseInsertionSort` uses `>` on `prev_open` — BOTH maintain `list.head` & `list.tail` and preserve FIFO ties.
   - **`put_on_closed_list`:** Prepends at `list.head` only and does NOT touch `list.tail`.

6. **Determinism & 3-Attempt Stall Escalation Rule:** Zero external crates, zero `unsafe`, zero `HashMap`/`HashSet`/`BinaryHeap`, zero std float math; must pass `python3 scripts/lint_determinism.py`. If 3 consecutive fix attempts on failing golden vectors fail to shrink the failure count, isolate the failing seed, re-read the C++ source below, or escalate (`tier:3-architect`).

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (8):**
  - `GameLogic/Module/BodyModule.h`
  - `GameLogic/Object.h`
  - `GameLogic/Pathfinder/PathfindCell.h`
  - `GameLogic/Pathfinder/PathfindCellInfo.h`
  - `GameLogic/Pathfinder/PathfindCellList.h`
  - `GameLogic/Pathfinder/PathfindConstants.h`
  - `PathfindCellInfo.h`
  - `WWMath/wwmath.h`
- **`Snapshot::crc` / `Snapshot::xfer` Calls:** None in module.
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if !RETAIL_COMPATIBLE_PATHFINDING`
  - `#if RETAIL_COMPATIBLE_PATHFINDING`
  - `#if RETAIL_COMPATIBLE_PATHFINDING_ALLOCATION`

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

### `Core/GameEngine/Include/GameLogic/Pathfinder/PathfindCell.h`
```cpp
#pragma once

#include "PathfindCellInfo.h"

class Object;
class PathfindCellList;

typedef UnsignedShort zoneStorageType;

#if !RETAIL_COMPATIBLE_PATHFINDING
#undef RETAIL_COMPATIBLE_PATHFINDING_ALLOCATION
#endif

/**
 * This represents one cell in the pathfinding grid.
 * These cells categorize the world into idealized cellular states,
 * and are also used for efficient A* pathfinding.
 * @todo Optimize memory usage of pathfind grid.
 */
class PathfindCell
{
public:

	enum CellType
	{
		CELL_CLEAR		= 0x00,									///< clear, unobstructed ground
		CELL_WATER		= 0x01,									///< water area
		CELL_CLIFF		= 0x02,									///< steep altitude change
		CELL_RUBBLE		= 0x03,									///< Cell is occupied by rubble.
		CELL_OBSTACLE	= 0x04,									///< Occupied by a structure
		CELL_BRIDGE_IMPASSABLE = 0x05,				///< Piece of a bridge that is impassable.
		CELL_IMPASSABLE = 0x06								///< Just plain impassable except for aircraft.
	};

	enum CellFlags
	{
		NO_UNITS		= 0x00,						///< No units in this cell.
		UNIT_GOAL		= 0x01,						///< A unit is heading to this cell.
		UNIT_PRESENT_MOVING	= 0x02,		///< A unit is moving through this cell.
		UNIT_PRESENT_FIXED	= 0x03,		///< A unit is stationary in this cell.
		UNIT_GOAL_OTHER_MOVING	= 0x05		///< A unit is moving through this cell, and another unit has this as it's goal.
	};

	/// reset the cell
	void reset();

	PathfindCell();
	~PathfindCell();

	Bool setTypeAsObstacle( Object *obstacle, Bool isFence, const ICoord2D &pos );				///< flag this cell as an obstacle, from the given one
	Bool removeObstacle( Object *obstacle );				///< unflag this cell as an obstacle, from the given one
	void setType( CellType type );	///< set the cell type
	CellType getType() const { return (CellType)m_type; }				///< get the cell type
	CellFlags getFlags() const { return (CellFlags)m_flags; }				///< get the cell type
	Bool isAircraftGoal() const {return m_aircraftGoal != 0;}

	Bool isObstaclePresent( ObjectID objID ) const;					///< return true if the given object ID is registered as an obstacle in this cell
#if RETAIL_COMPATIBLE_PATHFINDING_ALLOCATION
	// TheSuperHackers @info isObstructionInvalid() and clearObstruction() only used during retail compatible pathfinding failover cleanup
	Bool isObstructionInvalid() const { return m_obstacleID != INVALID_ID && m_info == nullptr && (m_type == CELL_OBSTACLE || m_type == CELL_IMPASSABLE); }
	void clearObstruction() { m_type = CELL_CLEAR; m_obstacleID = INVALID_ID; m_obstacleIsFence = false; m_obstacleIsTransparent = false; }
#endif

	Bool isObstacleTransparent() const;
	Bool isObstacleFence() const;

	/// Return estimated cost from given cell to reach goal cell
	UnsignedInt costToGoal( PathfindCell *goal );

	UnsignedInt costToHierGoal( PathfindCell *goal );

	UnsignedInt costSoFar( PathfindCell *parent );

#if RETAIL_COMPATIBLE_PATHFINDING
	// Forward insertion sort that is 100% retail compatible
	void forwardInsertionSortRetailCompatible(PathfindCellList& list);
#endif

	// Forward insertion sort, in ascending cost order
	void forwardInsertionSort(PathfindCellList& list);

	// Reverse insertion sort, in ascending cost order
	void reverseInsertionSort(PathfindCellList& list);

	/// put self on "open" list in ascending cost order
	void putOnSortedOpenList( PathfindCellList &list );

	/// remove self from "open" list
	void removeFromOpenList( PathfindCellList &list );

	/// put self on "closed" list, return new list
	void putOnClosedList( PathfindCellList &list );

	/// remove self from "closed" list
	void removeFromClosedList( PathfindCellList &list );

	/// remove all cells from closed list.
	static Int releaseClosedList( PathfindCellList &list );

	/// remove all cells from closed list.
	static Int releaseOpenList( PathfindCellList &list );

	PathfindCell *getNextOpen() {return m_info->m_nextOpen?m_info->m_nextOpen->m_cell: nullptr;}
	PathfindCell *getPrevOpen() {return m_info->m_prevOpen?m_info->m_prevOpen->m_cell: nullptr;}

	UnsignedShort getXIndex() const {return m_info->m_pos.x;}
	UnsignedShort getYIndex() const {return m_info->m_pos.y;}

	Bool isBlockedByAlly() const;
	void setBlockedByAlly(Bool blocked);

	Bool getOpen() const {return m_info->m_open;}
	Bool getClosed() const {return m_info->m_closed;}
	UnsignedInt getCostSoFar() const {return m_info->m_costSoFar;}
	UnsignedInt getTotalCost() const {return m_info->m_totalCost;}

	UnsignedInt getTotalCostDifference(PathfindCell& other) const;

	void setCostSoFar(UnsignedInt cost) { if( m_info ) m_info->m_costSoFar = cost;}
	void setTotalCost(UnsignedInt cost) { if( m_info ) m_info->m_totalCost = cost;}

	void setParentCell(PathfindCell* parent);
	void clearParentCell();
	void setParentCellHierarchical(PathfindCell* parent);
	PathfindCell* getParentCell() const {return m_info ? m_info->m_pathParent ? m_info->m_pathParent->m_cell : nullptr : nullptr;}

	Bool startPathfind( PathfindCell *goalCell );
	Bool getPinched() const {return m_pinched;}
	void setPinched(Bool pinch) {m_pinched = pinch;	}

	Bool allocateInfo(const ICoord2D &pos);
	void releaseInfo();
	Bool hasInfo() const {return m_info!=nullptr;}
	zoneStorageType getZone() const {return m_zone;}
	void setZone(zoneStorageType zone) {m_zone = zone;}
	void setGoalUnit(ObjectID unit, const ICoord2D &pos );
	void setGoalAircraft(ObjectID unit, const ICoord2D &pos );
	void setPosUnit(ObjectID unit, const ICoord2D &pos );
	ObjectID getGoalUnit() const {ObjectID id = m_info?m_info->m_goalUnitID:INVALID_ID; return id;}
	ObjectID getGoalAircraft() const {ObjectID id = m_info?m_info->m_goalAircraftID:INVALID_ID; return id;}
	ObjectID getPosUnit() const {ObjectID id = m_info?m_info->m_posUnitID:INVALID_ID; return id;}

	ObjectID getObstacleID() const;

	void setLayer( PathfindLayerEnum layer ) { m_layer = layer; }	///< set the cell layer
	PathfindLayerEnum getLayer() const { return (PathfindLayerEnum)m_layer; }				///< get the cell layer

	void setConnectLayer( PathfindLayerEnum layer ) { m_connectsToLayer = layer; }	///< set the cell layer	connect id
	PathfindLayerEnum getConnectLayer() const { return (PathfindLayerEnum)m_connectsToLayer; }				///< get the cell layer connect id

	static Bool typesMatch(const PathfindCell& targetCell, const PathfindCell& sourceCell);
	static Bool waterGround(const PathfindCell& targetCell, const PathfindCell& sourceCell);
	static Bool groundRubble(const PathfindCell& targetCell, const PathfindCell& sourceCell);
	static Bool terrain(const PathfindCell& targetCell, const PathfindCell& sourceCell);
	static Bool crusherGround(const PathfindCell& targetCell, const PathfindCell& sourceCell);
	static Bool groundCliff(const PathfindCell& targetCell, const PathfindCell& sourceCell);

private:
	PathfindCellInfo *m_info;
	ObjectID m_obstacleID;	                  ///< the object ID who overlaps this cell
	UnsignedInt m_blockedByAlly : 1;          ///< True if this cell is blocked by an allied unit.
	UnsignedInt m_obstacleIsFence : 1;        ///< True if occupied by a fence.
	UnsignedInt m_obstacleIsTransparent : 1;  ///< True if obstacle is transparent (undefined if obstacleid is invalid)

	zoneStorageType m_zone : 14;              ///< Zone. Each zone is a set of adjacent terrain type.  If from & to in the same zone, you can successfully pathfind.  If not,
	                                          /// you still may be able to if you can cross multiple terrain types.
	UnsignedShort m_aircraftGoal : 1;         ///< This is an aircraft goal cell.
	UnsignedShort m_pinched : 1;              ///< This cell is surrounded by obstacle cells.
	UnsignedByte m_type : 4;                  ///< what type of cell terrain this is.
	UnsignedByte m_flags : 4;                 ///< what type of units are in or moving through this cell.
	UnsignedByte m_connectsToLayer : 4;       ///< This cell can pathfind onto this layer, if > LAYER_TOP.
	UnsignedByte m_layer : 4;                 ///< Layer of this cell.
};

typedef PathfindCell *PathfindCellP;
```

### `Core/GameEngine/Source/GameLogic/AI/Pathfinder/PathfindCell.cpp`
```cpp
#include "GameLogic/Object.h"
#include "GameLogic/Pathfinder/PathfindCell.h"
#include "GameLogic/Pathfinder/PathfindCellInfo.h"
#include "GameLogic/Pathfinder/PathfindCellList.h"
#include "GameLogic/Pathfinder/PathfindConstants.h"
#include "GameLogic/Module/BodyModule.h"
#include "WWMath/wwmath.h"

/**
 * Constructor
 */
PathfindCell::PathfindCell() :m_info(nullptr)
{
	reset();
}

/**
 * Destructor
 */
PathfindCell::~PathfindCell()
{
	if (m_info) PathfindCellInfo::releaseACellInfo(m_info);
	m_info = nullptr;
	static Bool warn = true;
	if (warn) {
		warn = false;
		DEBUG_LOG( ("PathfindCell::~PathfindCell m_info Allocated."));
	}
}

/**
 * Reset the cell to default values
 */
void PathfindCell::reset()
{
	m_type = CELL_CLEAR;
	m_flags = NO_UNITS;
	m_zone = 0;
	m_aircraftGoal = false;
	m_pinched = false;
	if (m_info) {
		m_info->m_obstacleID = INVALID_ID;
		PathfindCellInfo::releaseACellInfo(m_info);
		m_info = nullptr;
	}
	m_obstacleID = INVALID_ID;
	m_blockedByAlly = false;
	m_obstacleIsFence = false;
	m_obstacleIsTransparent = false;

	m_connectsToLayer = LAYER_INVALID;
	m_layer = LAYER_GROUND;

}

/**
 * Reset the pathfinding values in the cell.
 */
Bool PathfindCell::startPathfind( PathfindCell *goalCell  )
{
	DEBUG_ASSERTCRASH(m_info, ("Has to have info."));
	m_info->m_nextOpen = nullptr;
	m_info->m_prevOpen = nullptr;
	m_info->m_pathParent = nullptr;
	m_info->m_costSoFar = 0;		// start node, no cost to get here
	m_info->m_totalCost = 0;
	if (goalCell) {
		m_info->m_totalCost = costToGoal( goalCell );
	}
#if RETAIL_COMPATIBLE_PATHFINDING
	if (!s_useFixedPathfinding) {
		m_info->m_open = TRUE;
	} else
#endif
	{
		m_info->m_open = FALSE;
	}
	m_info->m_closed = FALSE;
	return true;
}

/**
 * Set the blocked by ally flag on the pathfind cell info.
 */
Bool PathfindCell::isBlockedByAlly() const
{
#if RETAIL_COMPATIBLE_PATHFINDING_ALLOCATION
	if (s_useFixedPathfinding) {
		return m_blockedByAlly;
	}

	return m_info->m_blockedByAlly;
#else
	return m_blockedByAlly;
#endif
}

void PathfindCell::setBlockedByAlly(Bool blocked)
{
#if RETAIL_COMPATIBLE_PATHFINDING_ALLOCATION
	if (s_useFixedPathfinding) {
		m_blockedByAlly = (blocked != 0);
		return;
	}

	m_info->m_blockedByAlly = (blocked != 0);
#else
	m_blockedByAlly = (blocked != 0);
#endif
}

/**
 * Determine absolute total path cost difference between two cells.
 * Returns UINT_MAX if used with an uninitialised cell, so will be sorted as maximally dissimilar.
 */
UnsignedInt PathfindCell::getTotalCostDifference(PathfindCell& other) const
{
	if (m_info && other.m_info)
		return abs((Int)m_info->m_totalCost - (Int)other.m_info->m_totalCost);

	return UINT_MAX;
}

/**
 * Set the parent pointer.
 */
void PathfindCell::setParentCell( PathfindCell* parent  )
{
	DEBUG_ASSERTCRASH(m_info, ("Has to have info."));
	m_info->m_pathParent = parent->m_info;
	Int dx = m_info->m_pos.x - parent->m_info->m_pos.x;
	Int dy = m_info->m_pos.y - parent->m_info->m_pos.y;
	if (dx<-1 || dx>1 || dy<-1 || dy>1) {
		DEBUG_CRASH(("Invalid parent index."));
	}
}

/**
 * Set the parent pointer.
 */
void PathfindCell::setParentCellHierarchical( PathfindCell* parent  )
{
	DEBUG_ASSERTCRASH(m_info, ("Has to have info."));
	m_info->m_pathParent = parent->m_info;
}

/**
 * Reset the parent cell.
 */
void PathfindCell::clearParentCell(  )
{
	DEBUG_ASSERTCRASH(m_info, ("Has to have info."));
	m_info->m_pathParent = nullptr;
}


/**
 * Allocates an info record for a cell.
 */
Bool PathfindCell::allocateInfo( const ICoord2D &pos )
{
	if (!m_info) {
		m_info = PathfindCellInfo::getACellInfo(this, pos);
		return (m_info != nullptr);
	}
	return true;
}

/**
 * Releases an info record for a cell.
 */
void PathfindCell::releaseInfo()
{
	// TheSuperHackers @bugfix Mauller/SkyAero 05/06/2025 Parent cell links need clearing to prevent dangling pointers on starting points that can link them to an invalid parent cell.
	// Parent cells are only cleared within Pathfinder::prependCells, so cells that do not make it onto the final path do not get their parent cell cleared.
	// Cells with a special flags also do not get their PathfindCellInfo cleared and therefore can leave a parent cell set on a starting cell.
#if RETAIL_COMPATIBLE_PATHFINDING
	if (s_useFixedPathfinding)
#endif
	{
		if (m_info) {
			m_info->m_pathParent = nullptr;
		}
	}

	if (m_type == CELL_OBSTACLE || m_flags != NO_UNITS || m_aircraftGoal) {
		return;
	}

	if (!m_info) {
		return;
	}

	DEBUG_ASSERTCRASH(m_info->m_prevOpen==nullptr && m_info->m_nextOpen==nullptr, ("Shouldn't be linked."));
	DEBUG_ASSERTCRASH(m_info->m_open==0 && m_info->m_closed==0, ("Shouldn't be linked."));
	DEBUG_ASSERTCRASH(m_info->m_goalUnitID==INVALID_ID && m_info->m_posUnitID==INVALID_ID, ("Shouldn't be occupied."));
	DEBUG_ASSERTCRASH(m_info->m_goalAircraftID==INVALID_ID , ("Shouldn't be occupied by aircraft."));
	if (m_info->m_prevOpen || m_info->m_nextOpen || m_info->m_open || m_info->m_closed) {
		// Bad release.  Skip for now, better leak than crash.  jba.
		return;
	}

	PathfindCellInfo::releaseACellInfo(m_info);
	m_info = nullptr;

}

/**
 * Sets the goal unit into the info record for a cell.
 */
void PathfindCell::setGoalUnit(ObjectID unitID, const ICoord2D &pos )
{
	if (unitID==INVALID_ID) {
		// removing goal.
		if (m_info) {
			m_info->m_goalUnitID = INVALID_ID;
			if (m_info->m_posUnitID == INVALID_ID) {
				// No units here.
				DEBUG_ASSERTCRASH(m_flags==UNIT_GOAL, ("Bad flags."));
				m_flags = NO_UNITS;
				releaseInfo();
			} else{
				m_flags = UNIT_PRESENT_MOVING;
			}
		}	else {
			DEBUG_ASSERTCRASH(m_flags == NO_UNITS, ("Bad flags."));
		}
	} else {
		// adding goal.
		if (!m_info) {
			DEBUG_ASSERTCRASH(m_flags == NO_UNITS, ("Bad flags."));
			allocateInfo(pos);
		}
		if (!m_info) {
			DEBUG_CRASH(("Ran out of pathfind cells - fatal error!!!!! jba."));
			return;
		}
		m_info->m_goalUnitID = unitID;
		if (unitID==m_info->m_posUnitID) {
			m_flags = UNIT_PRESENT_FIXED;
		} else if (m_info->m_posUnitID==INVALID_ID) {
			m_flags = UNIT_GOAL;
		}	else {
			m_flags = UNIT_GOAL_OTHER_MOVING;
		}
	}
}


/**
 * Sets the goal aircraft into the info record for a cell.
 */
void PathfindCell::setGoalAircraft(ObjectID unitID, const ICoord2D &pos )
{
	if (unitID==INVALID_ID) {
		// removing goal.
		if (m_info) {
			m_info->m_goalAircraftID = INVALID_ID;
			m_aircraftGoal = false;
			releaseInfo();
		}	else {
			DEBUG_ASSERTCRASH(m_aircraftGoal==false, ("Bad flags."));
		}
	} else {
		// adding goal.
		if (!m_info) {
			DEBUG_ASSERTCRASH(m_aircraftGoal==false, ("Bad flags."));
			allocateInfo(pos);
		}
		if (!m_info) {
			DEBUG_CRASH(("Ran out of pathfind cells - fatal error!!!!! jba."));
			return;
		}
		m_info->m_goalAircraftID = unitID;
		m_aircraftGoal = true;
	}
}


/**
 * Sets the position unit into the info record for a cell.
 */
void PathfindCell::setPosUnit(ObjectID unitID, const ICoord2D &pos )
{
	if (unitID==INVALID_ID) {
		// removing position.
		if (m_info) {
			m_info->m_posUnitID = INVALID_ID;
			if (m_info->m_goalUnitID == INVALID_ID) {
				// No units here.
				DEBUG_ASSERTCRASH(m_flags==UNIT_PRESENT_MOVING, ("Bad flags."));
				m_flags = NO_UNITS;
				releaseInfo();
			}	else {
				m_flags = UNIT_GOAL;
			}
		}	else {
			DEBUG_ASSERTCRASH(m_flags == NO_UNITS, ("Bad flags."));
		}
	} else {
		// adding goal.
		if (!m_info) {
			DEBUG_ASSERTCRASH(m_flags == NO_UNITS, ("Bad flags."));
			allocateInfo(pos);
		}
		if (!m_info) {
			DEBUG_CRASH(("Ran out of pathfind cells - fatal error!!!!! jba."));
			return;
		}
		if (m_info->m_goalUnitID!=INVALID_ID && (m_info->m_goalUnitID==m_info->m_posUnitID)) {
			// A unit is already occupying this cell.
			return;
		}
		m_info->m_posUnitID = unitID;
		if (unitID==m_info->m_goalUnitID) {
			m_flags = UNIT_PRESENT_FIXED;
		} else if (m_info->m_goalUnitID==INVALID_ID) {
			m_flags = UNIT_PRESENT_MOVING;
		}	else {
			m_flags = UNIT_GOAL_OTHER_MOVING;
		}
	}
}


/**
 * Return the relevant obstacle ID.
 */
ObjectID PathfindCell::getObstacleID() const
{
#if RETAIL_COMPATIBLE_PATHFINDING_ALLOCATION
	if (s_useFixedPathfinding) {
		return m_obstacleID;
	}

	return m_info ? m_info->m_obstacleID : INVALID_ID;
#else
	return m_obstacleID;
#endif
}


/**
 * Flag this cell as an obstacle, from the given one.
 * Return true if cell was flagged.
 */
Bool PathfindCell::setTypeAsObstacle( Object *obstacle, Bool isFence, const ICoord2D &pos )
{
	if (m_type!=CELL_CLEAR && m_type != CELL_IMPASSABLE) {
		return false;
	}

	Bool isRubble = false;
	if (obstacle->getBodyModule() && obstacle->getBodyModule()->getDamageState() == BODY_RUBBLE)
	{
		isRubble = true;
	}

	if (isRubble) {
		m_type = CELL_RUBBLE;
		m_obstacleID = INVALID_ID;
		m_obstacleIsFence = false;
		m_obstacleIsTransparent = false;
#if RETAIL_COMPATIBLE_PATHFINDING_ALLOCATION
		if (s_useFixedPathfinding) {
			return true;
		}

		if (m_info) {
			m_info->m_obstacleID = INVALID_ID;
			releaseInfo();
		}
#endif
		return true;
	}

	m_type = CELL_OBSTACLE;
	m_obstacleID = obstacle->getID();
	m_obstacleIsFence = isFence;
	m_obstacleIsTransparent = obstacle->isKindOf(KINDOF_CAN_SEE_THROUGH_STRUCTURE);
#if RETAIL_COMPATIBLE_PATHFINDING_ALLOCATION
	// TheSuperHackers @info In retail mode we need to track orphaned cells set as obstacles so we can cleanup and failover properly
	// So we always make sure to set and clear the local obstacle data on the PathfindCell regardless of retail compat or not
	if (s_useFixedPathfinding) {
		return true;
	}

	if (!m_info) {
		m_info = PathfindCellInfo::getACellInfo(this, pos);
		if (!m_info) {
			DEBUG_CRASH(("Not enough PathFindCellInfos in pool."));
			return false;
		}
	}
	m_info->m_obstacleID = obstacle->getID();
	m_info->m_obstacleIsFence = isFence;
	m_info->m_obstacleIsTransparent = obstacle->isKindOf(KINDOF_CAN_SEE_THROUGH_STRUCTURE);
#endif
	return true;
}

/**
 * Flag this cell as given type.
 */
void PathfindCell::setType( CellType type )
{
#if RETAIL_COMPATIBLE_PATHFINDING_ALLOCATION
	if (s_useFixedPathfinding) {
		if (m_obstacleID != INVALID_ID) {
			DEBUG_ASSERTCRASH(type == CELL_OBSTACLE, ("Wrong type."));
			m_type = CELL_OBSTACLE;
			return;
		}
	}

	if (m_info && (m_info->m_obstacleID != INVALID_ID)) {
		DEBUG_ASSERTCRASH(type==CELL_OBSTACLE, ("Wrong type."));
		m_type = CELL_OBSTACLE;
		return;
	}
#else
	if (m_obstacleID != INVALID_ID) {
		DEBUG_ASSERTCRASH(type == CELL_OBSTACLE, ("Wrong type."));
		m_type = CELL_OBSTACLE;
		return;
	}
#endif
	m_type = type;
}

/**
 * Unflag this cell as an obstacle, from the given one.
 * Return true if this cell was previously flagged as an obstacle by this object.
 */
Bool PathfindCell::removeObstacle( Object *obstacle )
{
	if (m_type == CELL_RUBBLE) {
		m_type = CELL_CLEAR;
	}
#if RETAIL_COMPATIBLE_PATHFINDING_ALLOCATION
	if (s_useFixedPathfinding) {
		if (m_obstacleID != obstacle->getID()) return false;
		m_type = CELL_CLEAR;
		m_obstacleID = INVALID_ID;
		m_obstacleIsFence = false;
		m_obstacleIsTransparent = false;
		return true;
	}

	if (!m_info) return false;
	if (m_info->m_obstacleID != obstacle->getID()) return false;
	m_type = CELL_CLEAR;
	m_info->m_obstacleID = INVALID_ID;
	releaseInfo();

#else
	if (m_obstacleID != obstacle->getID()) return false;
	m_type = CELL_CLEAR;
#endif
	m_obstacleID = INVALID_ID;
	m_obstacleIsFence = false;
	m_obstacleIsTransparent = false;
	return true;
}

#if RETAIL_COMPATIBLE_PATHFINDING
// Retail compatible insertion sort
void PathfindCell::forwardInsertionSortRetailCompatible(PathfindCellList& list)
{
	DEBUG_ASSERTCRASH(m_info, ("Has to have info."));
	DEBUG_ASSERTCRASH(m_info->m_closed == FALSE && m_info->m_open == FALSE, ("Serious error - Invalid flags. jba"));

	// mark the newCell as being on the open list
	m_info->m_open = true;
	m_info->m_closed = false;

	if (list.m_head == nullptr)
	{
		list.m_head = this;
		m_info->m_prevOpen = nullptr;
		m_info->m_nextOpen = nullptr;
		return;
	}

	// insertion sort
	PathfindCell* currentCell = list.m_head;
	PathfindCell* previousCell = nullptr;
	UnsignedInt cellCount = 0;
	while (currentCell && cellCount < PATHFIND_CELLS_PER_FRAME && currentCell->m_info->m_totalCost <= m_info->m_totalCost)
	{
		// Prevent a retail crash where a pathfindCell has an m_info with a dangling nextOpen pointer
		if (currentCell->m_info->m_nextOpen && !currentCell->m_info->m_nextOpen->m_cell->m_info)
		{
			currentCell->m_info->m_nextOpen->m_cell = nullptr;
			currentCell->m_info->m_nextOpen = nullptr;
		}

		cellCount++;
		previousCell = currentCell;
		currentCell = currentCell->getNextOpen();
	}

	if (currentCell)
	{
		// insert just before "currentCell"
		if (currentCell->m_info->m_prevOpen)
			currentCell->m_info->m_prevOpen->m_nextOpen = this->m_info;
		else
			list.m_head = this;

		m_info->m_prevOpen = currentCell->m_info->m_prevOpen;
		currentCell->m_info->m_prevOpen = this->m_info;

		m_info->m_nextOpen = currentCell->m_info;

	}
	else
	{
		// append after "previousCell" - we are at the end of the list
		previousCell->m_info->m_nextOpen = this->m_info;
		m_info->m_prevOpen = previousCell->m_info;
		m_info->m_nextOpen = nullptr;
	}
}
#endif

// Forward insertion sort, returns early if the list is being initialized or we are prepending the list
void PathfindCell::forwardInsertionSort(PathfindCellList& list)
{
	DEBUG_ASSERTCRASH(m_info, ("Has to have info."));
	DEBUG_ASSERTCRASH(m_info->m_closed == FALSE && m_info->m_open == FALSE, ("Serious error - Invalid flags. jba"));

	// mark the new cell as being on the open list
	m_info->m_open = true;
	m_info->m_closed = false;

	if (list.m_head == nullptr) {
		m_info->m_prevOpen = nullptr;
		m_info->m_nextOpen = nullptr;
		list.m_head = this;
		list.m_tail = this;
		return;
	}

	// If the node needs inserting before the current list head
	if (m_info->m_totalCost < list.m_head->m_info->m_totalCost) {
		m_info->m_prevOpen = nullptr;
		list.m_head->m_info->m_prevOpen = this->m_info;
		m_info->m_nextOpen = list.m_head->m_info;
		list.m_head = this;
		return;
	}

	// Traverse the list to find correct position
	PathfindCell* current = list.m_head;
	while (current->m_info->m_nextOpen && current->m_info->m_nextOpen->m_totalCost <= m_info->m_totalCost) {
		current = current->getNextOpen();
	}

	// Insert the new node in the correct position
	m_info->m_nextOpen = current->m_info->m_nextOpen;
	if (current->m_info->m_nextOpen != nullptr) {
		current->m_info->m_nextOpen->m_prevOpen = this->m_info;
	}
	else {
		list.m_tail = this;
	}

	current->m_info->m_nextOpen = this->m_info;
	m_info->m_prevOpen = current->m_info;
}

// Reverse insertion sort, returns early if the list is being initialized or we are appending the list
void PathfindCell::reverseInsertionSort(PathfindCellList& list)
{
	DEBUG_ASSERTCRASH(m_info, ("Has to have info."));
	DEBUG_ASSERTCRASH(m_info->m_closed == FALSE && m_info->m_open == FALSE, ("Serious error - Invalid flags. jba"));

	// mark the new cell as being on the open list
	m_info->m_open = true;
	m_info->m_closed = false;

	if (list.m_tail == nullptr) {
		m_info->m_prevOpen = nullptr;
		m_info->m_nextOpen = nullptr;
		list.m_tail = this;
		list.m_head = this;
		return;
	}

	// If the node needs inserting after the current list tail
	if (m_info->m_totalCost >= list.m_tail->m_info->m_totalCost) {
		m_info->m_prevOpen = list.m_tail->m_info;
		list.m_tail->m_info->m_nextOpen = this->m_info;
		m_info->m_nextOpen = nullptr;
		list.m_tail = this;
		return;
	}

	// Traverse the list to find correct position
	PathfindCell* current = list.m_tail;
	while (current->m_info->m_prevOpen && current->m_info->m_prevOpen->m_totalCost > m_info->m_totalCost) {
		current = current->getPrevOpen();
	}

	// Insert the new node in the correct position
	m_info->m_prevOpen = current->m_info->m_prevOpen;
	if (current->m_info->m_prevOpen != nullptr) {
		current->m_info->m_prevOpen->m_nextOpen = this->m_info;
	}
	else {
		list.m_head = this;
	}

	current->m_info->m_prevOpen = this->m_info;
	m_info->m_nextOpen = current->m_info;
}

/// put self on "open" list in ascending cost order, return new list
void PathfindCell::putOnSortedOpenList( PathfindCellList &list )
{
#if RETAIL_COMPATIBLE_PATHFINDING
	if (!s_useFixedPathfinding) {
		forwardInsertionSortRetailCompatible(list);
		return;
	}
#endif

	// TheSuperHackers @performance Mauller 20/03/2026 Implement reverse insertion sorting.
	// Long and complex paths often append PathfindCell's, with high total path costs, to the open list.
	// Appending and reverse traversal allow faster insertion of these cells, reducing pathfinding overhead by 50 - 66%.
	if (list.canReverseSort(*this)) {
		reverseInsertionSort(list);
	}
	else {
		forwardInsertionSort(list);
	}
}

/// remove self from "open" list
void PathfindCell::removeFromOpenList( PathfindCellList &list )
{
	DEBUG_ASSERTCRASH(m_info, ("Has to have info."));
	DEBUG_ASSERTCRASH(m_info->m_closed==FALSE && m_info->m_open==TRUE, ("Serious error - Invalid flags. jba"));
	if (m_info->m_nextOpen)
		m_info->m_nextOpen->m_prevOpen = m_info->m_prevOpen;
	else {
		list.m_tail = getPrevOpen();
	}

	if (m_info->m_prevOpen)
		m_info->m_prevOpen->m_nextOpen = m_info->m_nextOpen;
	else
		list.m_head = getNextOpen();

	m_info->m_open = false;
	m_info->m_nextOpen = nullptr;
	m_info->m_prevOpen = nullptr;

}

/// remove all cells from "open" list
Int PathfindCell::releaseOpenList( PathfindCellList &list )
{
	Int count = 0;
	while (list.m_head) {
		count++;
		DEBUG_ASSERTCRASH(list.m_head->m_info, ("Has to have info."));
		DEBUG_ASSERTCRASH(list.m_head->m_info->m_closed==FALSE && list.m_head->m_info->m_open==TRUE, ("Serious error - Invalid flags. jba"));
		PathfindCell *cur = list.m_head;
		PathfindCellInfo *curInfo = list.m_head->m_info;

#if RETAIL_COMPATIBLE_PATHFINDING
		// TheSuperHackers @info This is only here to catch a crash point in the retail compatible pathfinding
		// One crash mode is where a cell has no PathfindCellInfo, resulting in a nullptr access and a crash.
		// Therefore we signal that we need to clean the maps cells and the PathfindCellInfos
		if(!curInfo && !s_useFixedPathfinding) {
			s_useFixedPathfinding = true;
			s_forceCleanCells = true;
			return count;
		}
#endif

		if (curInfo->m_nextOpen) {
			list.m_head = curInfo->m_nextOpen->m_cell;
		} else {
			list.reset();
		}
		DEBUG_ASSERTCRASH(cur == curInfo->m_cell, ("Bad backpointer in PathfindCellInfo"));
		curInfo->m_nextOpen = nullptr;
		curInfo->m_prevOpen = nullptr;
		curInfo->m_open = FALSE;
		cur->releaseInfo();
	}
	return count;
}

/// remove all cells from "closed" list
Int PathfindCell::releaseClosedList( PathfindCellList &list )
{
	Int count = 0;
	while (list.m_head) {
		count++;
		DEBUG_ASSERTCRASH(list.m_head->m_info, ("Has to have info."));
		DEBUG_ASSERTCRASH(list.m_head->m_info->m_closed==TRUE && list.m_head->m_info->m_open==FALSE, ("Serious error - Invalid flags. jba"));
		PathfindCell *cur = list.m_head;
		PathfindCellInfo *curInfo = list.m_head->m_info;
#if RETAIL_COMPATIBLE_PATHFINDING
		// TheSuperHackers @info This is only here to catch a crash point in the retail compatible pathfinding
		// One crash mode is where a cell has no PathfindCellInfo, resulting in a nullptr access and a crash.
		// Therefore we signal that we need to clean the maps cells and the PathfindCellInfos
		if(!curInfo && !s_useFixedPathfinding) {
			s_useFixedPathfinding = true;
			s_forceCleanCells = true;
			return count;
		}
#endif

		if (curInfo->m_nextOpen) {
			list.m_head = curInfo->m_nextOpen->m_cell;
		} else {
			list.reset();
		}
		DEBUG_ASSERTCRASH(cur == curInfo->m_cell, ("Bad backpointer in PathfindCellInfo"));
		curInfo->m_nextOpen = nullptr;
		curInfo->m_prevOpen = nullptr;
		curInfo->m_closed = FALSE;
		cur->releaseInfo();
	}
	return count;
}

/// put self on "closed" list, return new list
void PathfindCell::putOnClosedList( PathfindCellList &list )
{
	DEBUG_ASSERTCRASH(m_info, ("Has to have info."));
	DEBUG_ASSERTCRASH(m_info->m_closed==FALSE && m_info->m_open==FALSE, ("Serious error - Invalid flags. jba"));
	// only put on list if not already on it
	if (m_info->m_closed == FALSE)
	{
		m_info->m_closed = FALSE;
		m_info->m_closed = TRUE;

		m_info->m_prevOpen = nullptr;
		m_info->m_nextOpen = list.m_head ? list.m_head->m_info : nullptr;
		if (list.m_head)
#if RETAIL_COMPATIBLE_PATHFINDING
		// TheSuperHackers @info This is only here to catch a crash point in the retail compatible pathfinding
		// This crash mode occurs due to the closed list head not having an m_info associated with it
		// A node cannot be put onto the closed list without an m_info under normal conditions
		{
			if (list.m_head->m_info)
			{
				list.m_head->m_info->m_prevOpen = this->m_info;
			}
		}
#else
			list.m_head->m_info->m_prevOpen = this->m_info;
#endif

		list.m_head = this;
	}

}

/// remove self from "closed" list
void PathfindCell::removeFromClosedList( PathfindCellList &list )
{
	DEBUG_ASSERTCRASH(m_info, ("Has to have info."));
	DEBUG_ASSERTCRASH(m_info->m_closed==TRUE && m_info->m_open==FALSE, ("Serious error - Invalid flags. jba"));
	if (m_info->m_nextOpen)
		m_info->m_nextOpen->m_prevOpen = m_info->m_prevOpen;

	if (m_info->m_prevOpen)
		m_info->m_prevOpen->m_nextOpen = m_info->m_nextOpen;
	else
		list.m_head = getNextOpen();

	m_info->m_closed = false;
	m_info->m_nextOpen = nullptr;
	m_info->m_prevOpen = nullptr;

}

/**
 * Return true if the given object ID is registered as an obstacle in this cell
 */
Bool PathfindCell::isObstaclePresent(ObjectID objID) const
{
	if (objID != INVALID_ID && (getType() == CELL_OBSTACLE))
	{
#if RETAIL_COMPATIBLE_PATHFINDING_ALLOCATION
		if (s_useFixedPathfinding) {
			return m_obstacleID == objID;
		}

		DEBUG_ASSERTCRASH(m_info, ("Should have info to be obstacle."));
		return (m_info && m_info->m_obstacleID == objID);
#else
		return m_obstacleID == objID;
#endif
	}

	return false;
}


/**
 * return true if the obstacle in the cell is KINDOF_CAN_SEE_THROUGHT_STRUCTURE
 */
Bool PathfindCell::isObstacleTransparent() const
{
#if RETAIL_COMPATIBLE_PATHFINDING_ALLOCATION
	if (s_useFixedPathfinding) {
		return m_obstacleIsTransparent;
	}

	return m_info ? m_info->m_obstacleIsTransparent : false;
#else
	return m_obstacleIsTransparent;
#endif
}

/**
 * return true if the given obstacle in the cell is a fence.
 */
Bool PathfindCell::isObstacleFence() const
{
#if RETAIL_COMPATIBLE_PATHFINDING_ALLOCATION
	if (s_useFixedPathfinding) {
		return m_obstacleIsFence;
	}

	return m_info ? m_info->m_obstacleIsFence : false;
#else
	return m_obstacleIsFence;
#endif
}

UnsignedInt PathfindCell::costToGoal( PathfindCell *goal )
{
	DEBUG_ASSERTCRASH(m_info, ("Has to have info."));
	Int dx = m_info->m_pos.x - goal->getXIndex();
	Int dy = m_info->m_pos.y - goal->getYIndex();
#define NO_REAL_DIST
#ifdef REAL_DIST
	Int cost = COST_ORTHOGONAL*WWMath::Sqrt((float)(dx*dx + dy*dy));
#else
	if (dx<0) dx = -dx;
	if (dy<0) dy = -dy;
	Int cost;
	if (dx>dy) {
		cost= COST_ORTHOGONAL*dx + (COST_ORTHOGONAL*dy)/2;
	}	else {
		cost= COST_ORTHOGONAL*dy + (COST_ORTHOGONAL*dx)/2;
	}

#endif


	return cost;
}

UnsignedInt PathfindCell::costToHierGoal( PathfindCell *goal )
{
	if( !m_info )
	{
		DEBUG_CRASH( ("Has to have info.") );
		return 100000; //...patch hack 1.01
	}
	Int dx = m_info->m_pos.x - goal->getXIndex();
	Int dy = m_info->m_pos.y - goal->getYIndex();
	Int cost = REAL_TO_INT_FLOOR(COST_ORTHOGONAL*WWMath::Sqrt((float)(dx*dx + dy*dy)) + 0.5f);
	return cost;
}

UnsignedInt PathfindCell::costSoFar( PathfindCell *parent )
{
	DEBUG_ASSERTCRASH(m_info, ("Has to have info."));
	// very first node in path - no turns, no cost
	if (parent == nullptr)
		return 0;

	// add in number of turns in path so far
	ICoord2D prevDir;
	Int cost;

	prevDir.x = parent->getXIndex() - m_info->m_pos.x;
	prevDir.y = parent->getYIndex() - m_info->m_pos.y;

	// diagonal moves cost a bit more than orthogonal ones
	if (prevDir.x == 0 || prevDir.y == 0)
		cost = parent->getCostSoFar() + COST_ORTHOGONAL;
	else
		cost = parent->getCostSoFar() + COST_DIAGONAL;
	if (getPinched()) {
		cost += 1*COST_DIAGONAL;
	}

#if 1
	// Increase cost of turns.
	Int numTurns = 0;
	PathfindCell *prevCell = parent->getParentCell();
	if (prevCell) {

#if RETAIL_COMPATIBLE_PATHFINDING
		// TheSuperHackers @info this is a possible crash point in the retail pathfinding, we just prevent the crash at this point
		// External code should catch the issue in another block and cleanup the pathfinding before switching to the fixed pathfinding.
		if (!prevCell->hasInfo())
		{
			return cost;
		}
#endif

		ICoord2D dir;
		dir.x = prevCell->getXIndex() - parent->getXIndex();
		dir.y = prevCell->getYIndex() - parent->getYIndex();

		// count number of direction changes
		if (dir.x != prevDir.x || dir.y != prevDir.y)
		{
			Int dot = dir.x * prevDir.x + dir.y * prevDir.y;
			if (dot > 0)
				numTurns=4;				// 45 degree turn
			else if (dot == 0)
				numTurns = 8;		// 90 degree turn
			else
				numTurns = 16;		// 135 degree turn
		}
	}

	return cost + numTurns;
#else
	return cost;
#endif

}

Bool PathfindCell::typesMatch(const PathfindCell& targetCell, const PathfindCell& sourceCell)
{
	CellType targetType = targetCell.getType();
	CellType srcType = sourceCell.getType();
	if (targetType == srcType)
	{
		return true;
	}

	return false;
}

Bool PathfindCell::waterGround(const PathfindCell& targetCell, const PathfindCell& sourceCell)
{
	CellType targetType = targetCell.getType();
	CellType srcType = sourceCell.getType();
	if ((targetType == CELL_CLEAR &&
	     (srcType & CELL_WATER)))
	{
		return true;
	}
	if ((srcType == CELL_CLEAR &&
	     (targetType & CELL_WATER)))
	{
		return true;
	}

	return false;
}

Bool PathfindCell::groundRubble(const PathfindCell& targetCell, const PathfindCell& sourceCell)
{
	CellType targetType = targetCell.getType();
	CellType srcType = sourceCell.getType();
	if ((targetType == CELL_CLEAR &&
	     (srcType == CELL_RUBBLE)))
	{
		return true;
	}
	if ((srcType == CELL_CLEAR &&
	     (targetType == CELL_RUBBLE)))
	{
		return true;
	}

	return false;
}

Bool PathfindCell::terrain(const PathfindCell& targetCell, const PathfindCell& sourceCell)
{
	Int targetType = targetCell.getType();
	Int srcType = sourceCell.getType();
	if (targetType == CELL_OBSTACLE)
	{
		targetType = CELL_CLEAR;
	}
	if (srcType == CELL_OBSTACLE)
	{
		srcType = CELL_CLEAR;
	}
	if (targetType == srcType)
	{
		return true;
	}
	return false;
}

Bool PathfindCell::crusherGround(const PathfindCell& targetCell, const PathfindCell& sourceCell)
{
	Int targetType = targetCell.getType();
	Int srcType = sourceCell.getType();
	if (targetType == CELL_OBSTACLE)
	{
		if (targetCell.isObstacleFence())
		{
			if (srcType == CELL_CLEAR)
			{
				return true;
			}
		}
	}
	if (srcType == CELL_OBSTACLE)
	{
		if (sourceCell.isObstacleFence())
		{
			if (targetType == CELL_CLEAR)
			{
				return true;
			}
		}
	}
	return false;
}

Bool PathfindCell::groundCliff(const PathfindCell& targetCell, const PathfindCell& sourceCell)
{
	CellType targetType = targetCell.getType();
	CellType srcType = sourceCell.getType();

	if ((targetType == CELL_CLIFF) &&
	    (srcType == CELL_CLEAR))
	{
		return true;
	}
	if ((targetType == CELL_CLEAR) &&
	    (srcType == CELL_CLIFF))
	{
		return true;
	}
	return false;
}
```

### `Core/GameEngine/Include/GameLogic/Pathfinder/PathfindCellInfo.h`
```cpp
#pragma once

class PathfindCell;

class PathfindCellInfo
{
	friend class PathfindCell;
public:
#if RETAIL_COMPATIBLE_PATHFINDING
	static void forceCleanPathFindCellInfos();
#endif
	static void allocateCellInfos();
	static void releaseCellInfos();

	static PathfindCellInfo * getACellInfo(PathfindCell *cell, const ICoord2D &pos);
	static void releaseACellInfo(PathfindCellInfo *theInfo);

protected:
	static PathfindCellInfo *s_infoArray;
	static PathfindCellInfo *s_firstFree;							///<


	PathfindCellInfo *m_nextOpen, *m_prevOpen;						///< for A* "open" list, shared by closed list

	PathfindCellInfo *m_pathParent;												///< "parent" cell from pathfinder
	PathfindCell *m_cell;															///< Cell this info belongs to currently.

	UnsignedShort m_totalCost, m_costSoFar;	///< cost estimates for A* search

	/// have to include cell's coordinates, since cells are often accessed via pointer only
	ICoord2D m_pos;

	ObjectID m_goalUnitID; ///< The objectID of the ground unit whose goal this is.
	ObjectID m_posUnitID;  ///< The objectID of the ground unit that is occupying this cell.
	ObjectID m_goalAircraftID; ///< The objectID of the aircraft whose goal this is.

	ObjectID m_obstacleID;	///< the object ID who overlaps this cell

	UnsignedInt m_isFree:1;
	UnsignedInt m_blockedByAlly:1;///< True if this cell is blocked by an allied unit.
	UnsignedInt m_obstacleIsFence:1;///< True if occupied by a fence.
	UnsignedInt m_obstacleIsTransparent:1;///< True if obstacle is transparent (undefined if obstacleid is invalid)
	/// @todo Do we need both mark values in this cell?  Can't store a single value and compare it?
	UnsignedInt m_open:1;													///< place for marking this cell as on the open list
	UnsignedInt m_closed:1;												///< place for marking this cell as on the closed list
};
```

### `Core/GameEngine/Source/GameLogic/AI/Pathfinder/PathfindCellInfo.cpp`
```cpp
#include "PreRTS.h"
#include "GameLogic/Pathfinder/PathfindCellInfo.h"

constexpr const UnsignedInt CELL_INFOS_TO_ALLOCATE = 30000;

PathfindCellInfo *PathfindCellInfo::s_infoArray = nullptr;
PathfindCellInfo *PathfindCellInfo::s_firstFree = nullptr;

#if RETAIL_COMPATIBLE_PATHFINDING
void PathfindCellInfo::forceCleanPathFindCellInfos()
{
	for (Int i = 0; i < CELL_INFOS_TO_ALLOCATE - 1; i++) {
		s_infoArray[i].m_nextOpen = nullptr;
		s_infoArray[i].m_prevOpen = nullptr;
		s_infoArray[i].m_open = FALSE;
		s_infoArray[i].m_closed = FALSE;
	}
}
#endif

/**
 * Allocates a pool of pathfind cell infos.
 */
void PathfindCellInfo::allocateCellInfos()
{
	releaseCellInfos();
	s_infoArray = MSGNEW("PathfindCellInfo") PathfindCellInfo[CELL_INFOS_TO_ALLOCATE];	// pool[]ify
	s_infoArray[CELL_INFOS_TO_ALLOCATE-1].m_pathParent = nullptr;
	s_infoArray[CELL_INFOS_TO_ALLOCATE-1].m_isFree = true;
	s_firstFree = s_infoArray;
	for (Int i=0; i<CELL_INFOS_TO_ALLOCATE-1; i++) {
		s_infoArray[i].m_pathParent = &s_infoArray[i+1];
		s_infoArray[i].m_isFree = true;
	}
}

/**
 * Releases a pool of pathfind cell infos.
 */
void PathfindCellInfo::releaseCellInfos()
{
	if (s_infoArray==nullptr) {
		return; // haven't allocated any yet.
	}
	Int count=0;
	while (s_firstFree) {
		count++;
		DEBUG_ASSERTCRASH(s_firstFree->m_isFree, ("Should be freed."));
		s_firstFree = s_firstFree->m_pathParent;
	}
	DEBUG_ASSERTCRASH(count==CELL_INFOS_TO_ALLOCATE, ("Error - Allocated cellinfos."));
	delete[] s_infoArray;
	s_infoArray = nullptr;
	s_firstFree = nullptr;
}

/**
 * Gets a pathfindcellinfo.
 */
PathfindCellInfo *PathfindCellInfo::getACellInfo(PathfindCell *cell,const ICoord2D &pos)
{
	PathfindCellInfo *info = s_firstFree;
	if (s_firstFree) {
		DEBUG_ASSERTCRASH(s_firstFree->m_isFree, ("Should be freed."));
		s_firstFree = s_firstFree->m_pathParent;
		info->m_isFree = false;  // Just allocated it.
		info->m_cell = cell;
		info->m_pos = pos;

		info->m_nextOpen = nullptr;
		info->m_prevOpen = nullptr;
		info->m_pathParent = nullptr;
		info->m_costSoFar = 0;
		info->m_totalCost = 0;
		info->m_open = 0;
		info->m_closed = 0;
		info->m_obstacleID = INVALID_ID;
		info->m_goalUnitID = INVALID_ID;
		info->m_posUnitID = INVALID_ID;
		info->m_goalAircraftID = INVALID_ID;
		info->m_obstacleIsFence = false;
		info->m_obstacleIsTransparent = false;
		info->m_blockedByAlly = false;
	}
	return info;
}

/**
 * Returns a pathfindcellinfo.
 */
void PathfindCellInfo::releaseACellInfo(PathfindCellInfo *theInfo)
{
	DEBUG_ASSERTCRASH(!theInfo->m_isFree, ("Shouldn't be free."));
	//@ todo -fix this assert on usa04.  jba.
	//DEBUG_ASSERTCRASH(theInfo->m_obstacleID==0, ("Shouldn't be obstacle."));
	theInfo->m_pathParent = s_firstFree;
	s_firstFree = theInfo;
	s_firstFree->m_isFree = true;
}
```

### `Core/GameEngine/Include/GameLogic/Pathfinder/PathfindCellList.h`
```cpp
#pragma once

class PathfindCell;

// TheSuperHackers @info The PathfindCellList class acts as a new management class for the pathfindcell open and closed lists
class PathfindCellList
{
	friend class PathfindCell;

public:
	PathfindCellList() : m_head(nullptr), m_tail(nullptr) {}

#if RETAIL_COMPATIBLE_PATHFINDING
	void reset(PathfindCell* newHead = nullptr) { m_head = newHead; m_tail = nullptr; }
#else
	void reset() { m_head = nullptr; m_tail = nullptr; }
#endif

	PathfindCell* getHead() const { return m_head; }

	Bool empty() const { return m_head == nullptr; }

	Bool canReverseSort(PathfindCell& currentCell) const;

private:
	PathfindCell* m_head;
	PathfindCell* m_tail;
};
```

### `Core/GameEngine/Source/GameLogic/AI/Pathfinder/PathfindCellList.cpp`
```cpp
#include "PreRTS.h"
#include "GameLogic/Pathfinder/PathfindCell.h"
#include "GameLogic/Pathfinder/PathfindCellList.h"

Bool PathfindCellList::canReverseSort(PathfindCell& currentCell) const
{
	if (m_head && m_tail)
		return m_head->getTotalCostDifference(currentCell) > m_tail->getTotalCostDifference(currentCell);

	return false;
}
```

### `Core/GameEngine/Include/GameLogic/Pathfinder/PathfindConstants.h`
```cpp
#pragma once

constexpr const Int MAX_WALL_PIECES = 128;
constexpr const Int PATHFIND_QUEUE_LEN = 512;
constexpr const Int MOVE_ALLIES_MAX_DEPTH = 2;

// how close a unit has to be in z to interact with the layer.
constexpr const Real LAYER_Z_CLOSE_ENOUGH_F = 10.0f;

constexpr const UnsignedInt PATHFIND_CELL_SIZE = 10;

#if RETAIL_COMPATIBLE_CRC
// TheSuperHackers @info Avoid mismatches by using a macro to retain
// VC6's floating-point optimizations (e.g. 10.0f division -> 0.1f multiplication).
#define PATHFIND_CELL_SIZE_F 10.0f
#else
constexpr const Real PATHFIND_CELL_SIZE_F = 10.0f;
#endif

constexpr const UnsignedInt ZONE_UPDATE_FREQUENCY = 300;
constexpr const UnsignedInt MAX_CELL_COUNT = 500;
constexpr const UnsignedInt MAX_ADJUSTMENT_CELL_COUNT = 400;
constexpr const UnsignedInt MAX_SAFE_PATH_CELL_COUNT = 2000;

// Number of cells we will search pathfinding per frame.
constexpr const UnsignedInt PATHFIND_CELLS_PER_FRAME = 5000;

constexpr const Int COST_ORTHOGONAL = 10;
constexpr const Int COST_DIAGONAL = 14;
constexpr const Real COST_TO_DISTANCE_FACTOR = 1.0f / 10.0f;
constexpr const Real COST_TO_DISTANCE_FACTOR_SQR = COST_TO_DISTANCE_FACTOR * COST_TO_DISTANCE_FACTOR;

#if RETAIL_COMPATIBLE_PATHFINDING
// TheSuperHackers @info This variable is here so the code will run down the retail compatible path till a failure mode is hit
// The pathfinding will then switch over to the corrected pathfinding code for SH clients
extern Bool s_useFixedPathfinding;
extern Bool s_forceCleanCells;
#endif
```

### `Core/GameEngine/Source/GameLogic/AI/Pathfinder/PathfindConstants.cpp`
```cpp
#include "PreRTS.h"
#include "GameLogic/Pathfinder/PathfindConstants.h"

#if RETAIL_COMPATIBLE_PATHFINDING
Bool s_useFixedPathfinding = false;
Bool s_forceCleanCells = false;
#endif
```
