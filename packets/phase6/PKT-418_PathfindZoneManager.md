# Translation Packet `PKT-418` + `PKT-420`: `PathfindZoneManager` & `ZoneBlock`

## 1. Compartmentalization & Routing Metadata
- **Packet IDs:** `PKT-418` (`PathfindZoneManager`), `PKT-420` (`ZoneBlock`)
- **Work Package:** `WP-6.2` (Phase 6)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer` *(zone equivalency arrays & 10x10 hierarchical blocks)*
- **Environment Access:** `env:ephemeral-web` *(seeded with C++ Release oracle `docs/golden_vectors/PKT-418.json` & compiling Rust skeletons)*
- **FFI Seam Pattern:** Core Spine
- **Allowed Rust Output Files:**
  - `crates/zh_logic/src/pathfinder/zone_block.rs` *(implement all `todo!("PKT-420")` methods)*
  - `crates/zh_logic/src/pathfinder/pathfind_zone_manager.rs` *(implement all `todo!("PKT-418")` methods)*
  - `crates/zh_logic/tests/golden_pkt418_zone_manager.rs` *(remove both `#[ignore]` attributes once implemented)*
  - `docs/PORT_MAP.md` *(regenerate via `python3 scripts/port_map.py --write`)*
- **Mandatory V&V Gates:** `V1, V2-Golden`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test -p zh_logic --test golden_pkt418_zone_manager && python3 scripts/port_map.py --write && python3 scripts/port_map.py --check && cargo test --workspace && cargo check --workspace --all-targets
  ```

## Port Contract (read first)

1. **Zero Hour (`RTS_ZEROHOUR=1, RTS_GENERALS=0`) `#else` Branch in `calculate_zones`:**
   - Equal cell types (`cell.cell_type == neighbor.cell_type`) ONLY call `ZoneBlock::apply_zone` on `hierarchical_zones` and skip all other equivalency tables.
   - Differing cell types check `PathfindCell::terrain` (`terrain_zones`) and `PathfindCell::crusher_ground` (`crusher_zones`), setting `not_terrain_or_crusher = false` if either matches; only when `not_terrain_or_crusher` stays `true` do they check `water_ground` -> `else if ground_rubble` -> `else if ground_cliff`.
   - When calling `block_calculate_zones` per 10x10 block, `ZoneBlock` checks all 4 terrain equivalencies independently (`water_ground`, `ground_rubble`, `ground_cliff`, `crusher_ground`). Note C++ `update_zones_for_modify` uses `map[i][j]` (`bounds.hi.x += 1; bounds.hi.y += 1;` clamped to `global_bounds.hi`) and runs BOTH forward (`lo..=hi`) and backward (`hi..=lo`) propagation sweeps with side + diagonal `types_match` checks.

2. **`ZoneLayerSlot` (`0..=15`) & Bridge Endpoints (`LAYER_LAST = 15`):**
   - `calculate_zones` zeros `layers[0..=15].zone = 0` before Pass 1, runs collapse over `[1..total_zones)` (`collapsed[0] = 0`), then for each layer `0..=15`: looks up `zone = collapsed[layer.zone]`; if `zone == 0`, allocates `zone = max_zone; max_zone += 1`; calls `layer.zone = zone; layer.apply_zone();` (`applied_zone = zone`), and if `!layer.unused && !layer.destroyed`, calls `self.set_bridge(layer.start_cell.x, layer.start_cell.y, TRUE)` and `self.set_bridge(layer.end_cell.x, layer.end_cell.y, TRUE)`.
   - After zone table allocation, any cell with `cell.connect_layer() > PathfindLayerEnum::Ground` AND `cell.cell_type == CellType::Clear` joins `cell.zone` with `layers[cell.connect_layer() as usize].zone` in `hierarchical_zones`.

3. **Dirty Scheduling Lifecycle (`need_to_calculate_zones` / `mark_zones_dirty`):**
   - Initial `next_frame_to_calculate_zones = 0`; `need_to_calculate_zones(frame)` returns `TRUE` iff `next_frame_to_calculate_zones <= frame`.
   - `mark_zones_dirty(frame)`: if `frame < 2` sets `next = 2`, else `next = min(next, frame.wrapping_add(ZONE_UPDATE_FREQUENCY))` (`ZONE_UPDATE_FREQUENCY = 300`; note C++ wraps on `u32`).
   - `calculate_zones` resets `next_frame_to_calculate_zones = u32::MAX` at exit.

4. **Exact Range & Mask Checks (`ZoneBlock` vs `PathfindZoneManager`):**
   - `ZoneBlock::get_effective_zone`: `UNINITIALIZED_ZONE (0)` -> `0`; `(acceptable_surfaces & Air) != 0` or all three `(Ground | Water | Cliff)` set -> `1`; `num_zones < 2` -> `first_zone`; `zone < first_zone || zone >= first_zone + num_zones` -> `first_zone`; applies `crusher` remapping, then `Ground|Cliff`, `Ground|Water`, `Ground|Rubble`, falling back to `zone + first_zone`.
   - `PathfindZoneManager::get_effective_zone`: **`zone > max_zone`** -> `0` (`zone == max_zone` does NOT fail because `zones_allocated` is a power of 2 strictly greater than `max_zone` initialized to identity `i`); handles `Air`, `Ground|Water|Cliff`, `crusher`, `Ground|Cliff`, `Ground|Water`, `Ground|Rubble`, then `hierarchical_zones[zone]`. (`Cliff|Water` falls through to `hierarchical_zones[zone]` in Release.)
   - `PathfindZoneManager::get_block_zone`: looks up `ZoneBlock::get_effective_zone` and rejects **`zone >= max_zone`** -> `UNINITIALIZED_ZONE (0)`.

5. **`ZoneBlock::flatten_zones`, `resolve_zones`, & `resolve_block_zones`:**
   - `resolve_zones(src, target, zone_eq)` keeps the smaller of `zone_eq[src]` and `zone_eq[target]` and updates all matching elements in `zone_eq`.
   - `resolve_block_zones(src, target, zone_eq)` keeps `min(src, target)` over `zone_eq` directly.
   - `flatten_zones(zone_array, zone_hier)` executes a 2-hop lookup for every index `i` (`zone_array[i] = zone_hier[zone_array[zone_hier[zone_array[i]]]]`), then iterates `i` again calling `resolve_zones(zone_array[i], zone_hier[i], zone_array)` whenever they differ.

6. **Passability & Block Coordinates (`cell_x / ZONE_BLOCK_SIZE`):**
   - C++ uses signed truncating division `cell_x / 10` and `cell_y / 10`; `set_passable`, `is_passable`, `clip_is_passable`, and `interacts_with_bridge` reject `block_x < 0 || block_x >= extent.x || block_y < 0 || block_y >= extent.y` (note `is_passable` / `interacts_with_bridge` return `FALSE` out-of-bounds cleanly in Rust; `set_bridge` silently returns out-of-bounds).

7. **Map Data Layout & Column-Major Indexing:**
   - `PathfindZoneMap` stores `(width, height)` cells (`map.cell(x, y)`, `map.cell_mut(x, y)`), `pool`, and `mode`. Use `global_bounds.lo..=global_bounds.hi` loops directly as in C++ (`for y in lo.y..=hi.y { for x in lo.x..=hi.x ... }`).

8. **Determinism & 3-Attempt Escalation Rule:** Zero external crates, no `unsafe`, no `HashMap`/`HashSet`/`BinaryHeap`, no `Rc`/`RefCell`, no std float math. If 3 consecutive fix attempts fail to shrink failing vectors, isolate the minimal failing seed or escalate (`tier:3-architect`).

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (9):**
  - `GameLogic/GameLogic.h`
  - `GameLogic/LocomotorSet.h`
  - `GameLogic/Pathfinder/PathfindCell.h`
  - `GameLogic/Pathfinder/PathfindConstants.h`
  - `GameLogic/Pathfinder/PathfindLayer.h`
  - `GameLogic/Pathfinder/PathfindZoneManager.h`
  - `GameLogic/Pathfinder/ZoneBlock.h`
  - `GameLogic/TerrainLogic.h`
  - `PreRTS.h`
- **`Snapshot::crc` / `Snapshot::xfer` Calls:** None in module.
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if RTS_GENERALS && RETAIL_COMPATIBLE_PATHFINDING`

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

### `Core/GameEngine/Include/GameLogic/Pathfinder/PathfindZoneManager.h`
```cpp
#pragma once

#include "GameLogic/GameLogic.h"
#include "GameLogic/LocomotorSet.h"

class PathfindCell;
class PathfindLayer;
class ZoneBlock;

typedef UnsignedShort zoneStorageType;

/**
 * This class manages the zones in the map.  A zone is an area in the map that
 * is one contiguous type of terrain (clear, cliff, water, building).  If
 * a unit is in a zone, and wants to move to another location, the destination
 * zone has to be the same, or it can't get there.
 * There are equivalency tables for meta-zones.  For example, an amphibious craft can
 * travel through water and clear cells.
 */
class PathfindZoneManager
{
public:
	enum {INITIAL_ZONES = 256};
	enum {ZONE_BLOCK_SIZE = 10};	// Zones are calculated in blocks of 20x20.  This way, the raw zone numbers can be used to
	enum {UNINITIALIZED_ZONE = 0};
																// compute hierarchically between the 20x20 blocks of cells. jba.
	PathfindZoneManager();
	~PathfindZoneManager();

	void reset();

	Bool needToCalculateZones() const {return m_nextFrameToCalculateZones <= TheGameLogic->getFrame() ;} ///< Returns true if the zones need to be recalculated.
	void markZonesDirty() ; ///< Called when the zones need to be recalculated.
	void updateZonesForModify( PathfindCell **map,  PathfindLayer layers[], const IRegion2D &structureBounds, const IRegion2D &globalBounds ) ; ///< Called to recalculate an area when a structure has been removed.
	void calculateZones(	PathfindCell **map, PathfindLayer layers[], const IRegion2D &bounds);	///< Does zone calculations.
	zoneStorageType getEffectiveZone(LocomotorSurfaceTypeMask acceptableSurfaces, Bool crusher, zoneStorageType zone) const;
	zoneStorageType getEffectiveTerrainZone(zoneStorageType zone) const;

	void getExtent(ICoord2D &extent) const {extent = m_zoneBlockExtent;}

	/// return zone relative the the block zone that this cell resides in.
	zoneStorageType getBlockZone(LocomotorSurfaceTypeMask acceptableSurfaces, Bool crusher, Int cellX, Int cellY, PathfindCell **map) const;
	void allocateBlocks(const IRegion2D &globalBounds);

	void clearPassableFlags();
	Bool isPassable(Int cellX, Int cellY) const;
	Bool clipIsPassable(Int cellX, Int cellY) const;
	void setPassable(Int cellX, Int cellY, Bool passable);

	void setAllPassable();

	void setBridge(Int cellX, Int cellY, Bool bridge);
	Bool interactsWithBridge(Int cellX, Int cellY) const;

private:
	void allocateZones();
	void freeZones();
	void freeBlocks();

private:
	ZoneBlock			*m_blockOfZoneBlocks;			///< Zone blocks - Info for hierarchical pathfinding at a "blocky" level.
	ZoneBlock			**m_zoneBlocks;						///< Zone blocks as a matrix - contains matrix indexing into the map.
	ICoord2D			m_zoneBlockExtent;				///< Zone block extents. Not the same scale as the pathfind extents.

	UnsignedShort m_maxZone;								///< Max zone used.
	UnsignedInt		m_nextFrameToCalculateZones;		///< When should I recalculate, next?.
	UnsignedShort m_zonesAllocated;
	zoneStorageType *m_groundCliffZones;
	zoneStorageType *m_groundWaterZones;
	zoneStorageType *m_groundRubbleZones;
	zoneStorageType *m_terrainZones;
	zoneStorageType *m_crusherZones;
	zoneStorageType *m_hierarchicalZones;
};
```

### `Core/GameEngine/Source/GameLogic/AI/Pathfinder/PathfindZoneManager.cpp`
```cpp
#include "PreRTS.h"
#include "GameLogic/Pathfinder/PathfindCell.h"
#include "GameLogic/Pathfinder/PathfindConstants.h"
#include "GameLogic/Pathfinder/PathfindLayer.h"
#include "GameLogic/Pathfinder/PathfindZoneManager.h"
#include "GameLogic/Pathfinder/ZoneBlock.h"
#include "GameLogic/TerrainLogic.h"

PathfindZoneManager::PathfindZoneManager() : m_maxZone(0),
m_nextFrameToCalculateZones(0),
m_groundCliffZones(nullptr),
m_groundWaterZones(nullptr),
m_groundRubbleZones(nullptr),
m_terrainZones(nullptr),
m_crusherZones(nullptr),
m_hierarchicalZones(nullptr),
m_blockOfZoneBlocks(nullptr),
m_zoneBlocks(nullptr),
m_zonesAllocated(0)
{
	m_zoneBlockExtent.x = 0;
	m_zoneBlockExtent.y = 0;
}

PathfindZoneManager::~PathfindZoneManager()
{
	freeZones();
	freeBlocks();
}

void PathfindZoneManager::freeZones()
{
	delete [] m_groundCliffZones;
	m_groundCliffZones = nullptr;

	delete [] m_groundWaterZones;
	m_groundWaterZones = nullptr;

	delete [] m_groundRubbleZones;
	m_groundRubbleZones = nullptr;

	delete [] m_terrainZones;
	m_terrainZones = nullptr;

	delete [] m_crusherZones;
	m_crusherZones = nullptr;

	delete [] m_hierarchicalZones;
	m_hierarchicalZones = nullptr;

	m_zonesAllocated = 0;
}

void PathfindZoneManager::freeBlocks()
{
	delete [] m_blockOfZoneBlocks;
	m_blockOfZoneBlocks = nullptr;

	delete [] m_zoneBlocks;
	m_zoneBlocks = nullptr;

	m_zoneBlockExtent.x = 0;
	m_zoneBlockExtent.y = 0;
}

/* Allocate zone equivalency arrays large enough to hold m_maxZone entries.  If the arrays are already
large enough, just return. */
void PathfindZoneManager::allocateZones()
{
	if (m_zonesAllocated>m_maxZone && m_groundCliffZones!=nullptr) {
		return;
	}
	freeZones();

	if (m_zonesAllocated == 0) {
		m_zonesAllocated = INITIAL_ZONES;
	}
	while (m_zonesAllocated <= m_maxZone) {
		m_zonesAllocated *= 2;
	}
	DEBUG_LOG(("Allocating zone tables of size %d", m_zonesAllocated));
	// pool[]ify
	m_groundCliffZones = MSGNEW("PathfindZoneInfo") zoneStorageType[m_zonesAllocated];
	m_groundWaterZones = MSGNEW("PathfindZoneInfo") zoneStorageType[m_zonesAllocated];
	m_groundRubbleZones = MSGNEW("PathfindZoneInfo") zoneStorageType[m_zonesAllocated];
	m_terrainZones = MSGNEW("PathfindZoneInfo") zoneStorageType[m_zonesAllocated];
	m_crusherZones = MSGNEW("PathfindZoneInfo") zoneStorageType[m_zonesAllocated];
	m_hierarchicalZones = MSGNEW("PathfindZoneInfo") zoneStorageType[m_zonesAllocated];
}

/* Allocate zone blocks for hierarchical pathfinding.   */
void PathfindZoneManager::allocateBlocks(const IRegion2D &globalBounds)
{
	freeBlocks();

	m_zoneBlockExtent.x = (globalBounds.hi.x-globalBounds.lo.x+1+ZONE_BLOCK_SIZE-1)/ZONE_BLOCK_SIZE;
	m_zoneBlockExtent.y = (globalBounds.hi.y-globalBounds.lo.y+1+ZONE_BLOCK_SIZE-1)/ZONE_BLOCK_SIZE;

	m_blockOfZoneBlocks = MSGNEW("PathfindZoneBlocks") ZoneBlock[(m_zoneBlockExtent.x)*(m_zoneBlockExtent.y)];
	m_zoneBlocks = MSGNEW("PathfindZoneBlocks") ZoneBlockP[m_zoneBlockExtent.x];
	Int i;
	for (i=0; i<m_zoneBlockExtent.x; i++) {
		m_zoneBlocks[i] = &m_blockOfZoneBlocks[i*(m_zoneBlockExtent.y)];
	}
}

void PathfindZoneManager::reset()  ///< Called when the map is reset.
{
	freeZones();
	freeBlocks();
}


void PathfindZoneManager::markZonesDirty()  ///< Called when the zones need to be recalculated.
{
#if RTS_GENERALS && RETAIL_COMPATIBLE_PATHFINDING
	m_nextFrameToCalculateZones = TheGameLogic->getFrame();
#else
	if (TheGameLogic->getFrame()<2) {
		m_nextFrameToCalculateZones = 2;
		return;
	}
	m_nextFrameToCalculateZones = MIN( m_nextFrameToCalculateZones, TheGameLogic->getFrame() + ZONE_UPDATE_FREQUENCY );
#endif
}

/**
 * Calculate zones.  A zone is an area of the same terrain - clear, water or cliff.
 * The utility of zones is that if current location and destination are in the same zone,
 * you can successfully pathfind.
 * If you are a multiple terrain vehicle, like amphibious transport, the lookup is a little more
 * complicated.
 */
void PathfindZoneManager::calculateZones( PathfindCell **map, PathfindLayer layers[], const IRegion2D &globalBounds )
{
#ifdef DEBUG_QPF
#if defined(DEBUG_LOGGING)
	__int64 startTime64;
	static double timeToUpdate = 0.0f;
	static double averageTimeToUpdate = 0.0f;
	static Int updateSamples = 0;
	__int64 endTime64,freq64;
	QueryPerformanceFrequency((LARGE_INTEGER *)&freq64);
	QueryPerformanceCounter((LARGE_INTEGER *)&startTime64);
#endif
#endif

	m_maxZone = 1;	// we start using zone 0 as a flag.
	const Int maxZones=24000;
	zoneStorageType zoneEquivalency[maxZones];
	Int i, j;
	for (i=0; i<maxZones; i++) {
		zoneEquivalency[i] = i;
	}
	for (i=0; i<=LAYER_LAST; i++) {
		layers[i].setZone(0);
	}

	Int xCount = (globalBounds.hi.x-globalBounds.lo.x+1+ZONE_BLOCK_SIZE-1)/ZONE_BLOCK_SIZE;
	Int yCount = (globalBounds.hi.y-globalBounds.lo.y+1+ZONE_BLOCK_SIZE-1)/ZONE_BLOCK_SIZE;

	Int xBlock, yBlock;
	for (xBlock = 0; xBlock<xCount; xBlock++) {
		for (yBlock=0; yBlock<yCount; yBlock++) {
			IRegion2D bounds;
			bounds.lo.x = globalBounds.lo.x + xBlock*ZONE_BLOCK_SIZE;
			bounds.lo.y = globalBounds.lo.y + yBlock*ZONE_BLOCK_SIZE;
			bounds.hi.x = bounds.lo.x + ZONE_BLOCK_SIZE - 1; // bounds are inclusive.
			bounds.hi.y = bounds.lo.y + ZONE_BLOCK_SIZE - 1; // bounds are inclusive.
			bounds.hi.updateMin(globalBounds.hi);
#if RTS_GENERALS && RETAIL_COMPATIBLE_PATHFINDING
			if (bounds.lo.x>bounds.hi.x || bounds.lo.y>bounds.hi.y) {
				DEBUG_CRASH(("Incorrect bounds calculation. Logic error, fix me. jba."));
				continue;
			}
#endif
			m_zoneBlocks[xBlock][yBlock].setInteractsWithBridge(false);
			for( j=bounds.lo.y; j<=bounds.hi.y; j++ )	{
				for( i=bounds.lo.x; i<=bounds.hi.x; i++ )	{
					PathfindCell *cell = &map[i][j];
					cell->setZone(0);

					if (i>bounds.lo.x) {
						if (map[i][j].getType() == map[i-1][j].getType()) {
							ZoneBlock::applyZone(map[i][j], map[i-1][j], zoneEquivalency, m_maxZone);
						}
					}
					if (j>bounds.lo.y) {
						if (map[i][j].getType() == map[i][j-1].getType()) {
							ZoneBlock::applyZone(map[i][j], map[i][j - 1], zoneEquivalency, m_maxZone);
						}
					}
					if (cell->getZone()==0) {
						cell->setZone(m_maxZone);
						m_maxZone++;
#if RTS_GENERALS && RETAIL_COMPATIBLE_PATHFINDING
						if (m_maxZone>= maxZones) {
							DEBUG_CRASH(("Ran out of pathfind zones.  SERIOUS ERROR! jba."));
							break;
						}
#endif
					}
					if (cell->getConnectLayer() > LAYER_GROUND) {
 						m_zoneBlocks[xBlock][yBlock].setInteractsWithBridge(true);
					}

				}
			}
 		}
	}

	Int totalZones = m_maxZone;

	// Collapse the zones into a 1,2,3... sequence, removing collapsed zones.
	m_maxZone = 1;
	Int collapsedZones[maxZones];
	collapsedZones[0] = 0;
	for (i=1; i<totalZones; i++) {
		Int zone = zoneEquivalency[i];
		if (zone == i) {
			collapsedZones[i] = m_maxZone;
			++m_maxZone;
		} else {
			collapsedZones[i] = collapsedZones[zone];
		}
	}

	// Now map the zones in the map back into the collapsed zones.
	for( j=globalBounds.lo.y; j<=globalBounds.hi.y; j++ ) {
		for( i=globalBounds.lo.x; i<=globalBounds.hi.x; i++ ) {
			PathfindCell &cell = map[i][j];
			cell.setZone(collapsedZones[cell.getZone()]);
		}
	}
	
	for (i=0; i<=LAYER_LAST; i++) {
		PathfindLayer &r_thisLayer = layers[i];

		Int zone = collapsedZones[r_thisLayer.getZone()];
		if (zone == 0) {
			zone = m_maxZone;
			m_maxZone++;
		}

		r_thisLayer.setZone( zone );
		r_thisLayer.applyZone();

		if (!r_thisLayer.isUnused() && !r_thisLayer.isDestroyed()) {
			ICoord2D ndx;
			r_thisLayer.getStartCellIndex(&ndx);
			setBridge(ndx.x, ndx.y, true);
			r_thisLayer.getEndCellIndex(&ndx);
			setBridge(ndx.x, ndx.y, true);
		}
	}

	allocateZones();

	for (xBlock=0; xBlock<xCount; xBlock++) {
		for (yBlock=0; yBlock<yCount; yBlock++) {
			IRegion2D bounds;
			bounds.lo.x = globalBounds.lo.x + xBlock*ZONE_BLOCK_SIZE;
			bounds.lo.y = globalBounds.lo.y + yBlock*ZONE_BLOCK_SIZE;
			bounds.hi.x = bounds.lo.x + ZONE_BLOCK_SIZE - 1; // bounds are inclusive.
			bounds.hi.y = bounds.lo.y + ZONE_BLOCK_SIZE - 1; // bounds are inclusive.

			bounds.hi.updateMin(globalBounds.hi);
#if RTS_GENERALS && RETAIL_COMPATIBLE_PATHFINDING
			if (bounds.lo.x>bounds.hi.x || bounds.lo.y>bounds.hi.y) {
				DEBUG_CRASH(("Incorrect bounds calculation. Logic error, fix me. jba."));
				continue;
			}
#endif
			m_zoneBlocks[xBlock][yBlock].blockCalculateZones(map, layers, bounds);
		}
	}

	// Determine water/ground equivalent zones, and ground/cliff equivalent zones.
	for (i=0; i<m_zonesAllocated; i++) {
		m_groundCliffZones[i] = i;
		m_groundWaterZones[i] = i;
		m_groundRubbleZones[i] = i;
		m_terrainZones[i] = i;
		m_crusherZones[i] = i;
		m_hierarchicalZones[i] = i;
	}

	for( j=globalBounds.lo.y; j<=globalBounds.hi.y; j++ ) {
		for( i=globalBounds.lo.x; i<=globalBounds.hi.x; i++ ) {
			PathfindCell &r_thisCell = map[i][j];

			if ( (r_thisCell.getConnectLayer() > LAYER_GROUND) &&
				(r_thisCell.getType() == PathfindCell::CELL_CLEAR) ) {
				PathfindLayer *layer = layers + r_thisCell.getConnectLayer();
				ZoneBlock::resolveZones(r_thisCell.getZone(), layer->getZone(), m_hierarchicalZones, m_maxZone);
			}

			if ( i > globalBounds.lo.x && r_thisCell.getZone() != map[i-1][j].getZone() ) {
				const PathfindCell &r_leftCell = map[i-1][j];

#if RTS_GENERALS && RETAIL_COMPATIBLE_PATHFINDING
				if (r_thisCell.getType() == r_leftCell.getType()) {
					ZoneBlock::applyZone(r_thisCell, r_leftCell, m_hierarchicalZones, m_maxZone);
				}
				if (PathfindCell::waterGround(r_thisCell, r_leftCell)) {
					ZoneBlock::applyZone(r_thisCell, r_leftCell, m_groundWaterZones, m_maxZone);
				}
				if (PathfindCell::groundRubble(r_thisCell, r_leftCell)) {
					ZoneBlock::applyZone(r_thisCell, r_leftCell, m_groundRubbleZones, m_maxZone);
				}
				if (PathfindCell::groundCliff(r_thisCell, r_leftCell)) {
					ZoneBlock::applyZone(r_thisCell, r_leftCell, m_groundCliffZones, m_maxZone);
				}
				if (PathfindCell::terrain(r_thisCell, r_leftCell)) {
					ZoneBlock::applyZone(r_thisCell, r_leftCell, m_terrainZones, m_maxZone);
				}
				if (PathfindCell::crusherGround(r_thisCell, r_leftCell)) {
					ZoneBlock::applyZone(r_thisCell, r_leftCell, m_crusherZones, m_maxZone);
				}
#else
				//if this is true, skip all the ones below
				if (r_thisCell.getType() == r_leftCell.getType())
					ZoneBlock::applyZone(r_thisCell, r_leftCell, m_hierarchicalZones, m_maxZone);
				else {
					Bool notTerrainOrCrusher = TRUE; // if this is false, skip the if-else-ladder below

					if (PathfindCell::terrain(r_thisCell, r_leftCell)) {
						ZoneBlock::applyZone(r_thisCell, r_leftCell, m_terrainZones, m_maxZone);
						notTerrainOrCrusher = FALSE;
					}

					if (PathfindCell::crusherGround(r_thisCell, r_leftCell)) {
						ZoneBlock::applyZone(r_thisCell, r_leftCell, m_crusherZones, m_maxZone);
						notTerrainOrCrusher = FALSE;
					}

					if ( notTerrainOrCrusher ) {
						if (PathfindCell::waterGround(r_thisCell, r_leftCell))
							ZoneBlock::applyZone(r_thisCell, r_leftCell, m_groundWaterZones, m_maxZone);
						else if (PathfindCell::groundRubble(r_thisCell, r_leftCell))
							ZoneBlock::applyZone(r_thisCell, r_leftCell, m_groundRubbleZones, m_maxZone);
						else if (PathfindCell::groundCliff(r_thisCell, r_leftCell))
							ZoneBlock::applyZone(r_thisCell, r_leftCell, m_groundCliffZones, m_maxZone);
					}

				}
#endif

			}

			if (j>globalBounds.lo.y && r_thisCell.getZone()!=map[i][j-1].getZone()) {
				const PathfindCell &r_topCell = map[i][j-1];

#if RTS_GENERALS && RETAIL_COMPATIBLE_PATHFINDING
				if (r_thisCell.getType() == r_topCell.getType()) {
					ZoneBlock::applyZone(r_thisCell, r_topCell, m_hierarchicalZones, m_maxZone);
				}
				if (PathfindCell::waterGround(r_thisCell, r_topCell)) {
					ZoneBlock::applyZone(r_thisCell, r_topCell, m_groundWaterZones, m_maxZone);
				}
				if (PathfindCell::groundRubble(r_thisCell, r_topCell)) {
					ZoneBlock::applyZone(r_thisCell, r_topCell, m_groundRubbleZones, m_maxZone);
				}
				if (PathfindCell::groundCliff(r_thisCell, r_topCell)) {
					ZoneBlock::applyZone(r_thisCell, r_topCell, m_groundCliffZones, m_maxZone);
				}
				if (PathfindCell::terrain(r_thisCell, r_topCell)) {
					ZoneBlock::applyZone(r_thisCell, r_topCell, m_terrainZones, m_maxZone);
				}
				if (PathfindCell::crusherGround(r_thisCell, r_topCell)) {
					ZoneBlock::applyZone(r_thisCell, r_topCell, m_crusherZones, m_maxZone);
				}
#else
				//if this is true, skip all the ones below
				if (r_thisCell.getType() == r_topCell.getType())
					ZoneBlock::applyZone(r_thisCell, r_topCell, m_hierarchicalZones, m_maxZone);
				else {
					Bool notTerrainOrCrusher = TRUE; // if this is false, skip the if-else-ladder below

					if (PathfindCell::terrain(r_thisCell, r_topCell)) {
						ZoneBlock::applyZone(r_thisCell, r_topCell, m_terrainZones, m_maxZone);
						notTerrainOrCrusher = FALSE;
					}

					if (PathfindCell::crusherGround(r_thisCell, r_topCell)) {
						ZoneBlock::applyZone(r_thisCell, r_topCell, m_crusherZones, m_maxZone);
						notTerrainOrCrusher = FALSE;
					}

					if (notTerrainOrCrusher) {
						if (PathfindCell::waterGround(r_thisCell, r_topCell))
							ZoneBlock::applyZone(r_thisCell, r_topCell, m_groundWaterZones, m_maxZone);
						else if (PathfindCell::groundRubble(r_thisCell, r_topCell))
							ZoneBlock::applyZone(r_thisCell, r_topCell, m_groundRubbleZones, m_maxZone);
						else if (PathfindCell::groundCliff(r_thisCell, r_topCell))
							ZoneBlock::applyZone(r_thisCell, r_topCell, m_groundCliffZones, m_maxZone);
					}

				}
#endif

			}

		}
	}

	//FLATTEN HIERARCHICAL ZONES
	for (i=1; i<m_maxZone; i++) {
		Int zone = m_hierarchicalZones[i];
		m_hierarchicalZones[i] = m_hierarchicalZones[zone];
	}

	//THIS BLOCK IS 20%
	ZoneBlock::flattenZones(m_groundCliffZones, m_hierarchicalZones, m_maxZone);
	ZoneBlock::flattenZones(m_groundWaterZones, m_hierarchicalZones, m_maxZone);
	ZoneBlock::flattenZones(m_groundRubbleZones, m_hierarchicalZones, m_maxZone);
	ZoneBlock::flattenZones(m_terrainZones, m_hierarchicalZones, m_maxZone);
	ZoneBlock::flattenZones(m_crusherZones, m_hierarchicalZones, m_maxZone);

#ifdef DEBUG_QPF
#if defined(DEBUG_LOGGING)
	QueryPerformanceCounter((LARGE_INTEGER *)&endTime64);
	timeToUpdate = ((double)(endTime64-startTime64) / (double)(freq64));

	if ( updateSamples < 400 ) {
		averageTimeToUpdate = ((averageTimeToUpdate * updateSamples) + timeToUpdate) / (updateSamples + 1.0f);
		updateSamples++;
		DEBUG_LOG(("computing...: %f", averageTimeToUpdate));
	}
	else if ( updateSamples == 400 ) {
		DEBUG_LOG((" =============DONE============= Average time to calculate zones: %f", averageTimeToUpdate));
		DEBUG_LOG(("                                           Percent of baseline : %f", averageTimeToUpdate/0.003335f));
		updateSamples = 777;
	}

#endif
#endif
#if defined(RTS_DEBUG)
	if (TheGlobalData->m_debugAI == AI_DEBUG_ZONES)
	{
		extern void addIcon(const Coord3D *pos, Real width, Int numFramesDuration, RGBColor color);
		RGBColor color;
		memset(&color, 0, sizeof(Color));
		addIcon(nullptr, 0, 0, color);
		for( j=0; j<globalBounds.hi.y; j++ )	{
			for( i=0; i<globalBounds.hi.x; i++ )	{
				Int zone = map[i][j].getZone();
				zone = m_hierarchicalZones[zone];

				color.blue = (zone%3) * 0.5f;
				zone = zone/3;
				color.green = (zone%3) * 0.5f;
				zone = zone/3;
				color.red = (zone%3) * 0.5;
				Coord3D pos;
				pos.x = ((Real)i + 0.5f) * PATHFIND_CELL_SIZE_F;
				pos.y = ((Real)j + 0.5f) * PATHFIND_CELL_SIZE_F;
				pos.z = TheTerrainLogic->getLayerHeight( pos.x, pos.y, map[i][j].getLayer() ) + 0.5f;
				addIcon(&pos, PATHFIND_CELL_SIZE_F*0.8f, 500, color);
			}
		}
	}
#endif
	m_nextFrameToCalculateZones = 0xffffffff;
}

/**
 * Update zones where a structure has been added or removed.
 * This can be done by just updating the equivalency arrays, without rezoning the map..
 */
void PathfindZoneManager::updateZonesForModify(PathfindCell **map, PathfindLayer layers[], const IRegion2D &structureBounds, const IRegion2D &globalBounds )
{

#ifdef DEBUG_QPF
#if defined(DEBUG_LOGGING)
	__int64 startTime64;
	double timeToUpdate=0.0f;
	__int64 endTime64,freq64;
	QueryPerformanceFrequency((LARGE_INTEGER *)&freq64);
	QueryPerformanceCounter((LARGE_INTEGER *)&startTime64);
#endif
#endif
	IRegion2D bounds = structureBounds;
	bounds.hi.x++;
	bounds.hi.y++;
	bounds.hi.updateMin(globalBounds.hi);

	Int xBlock, yBlock;
	for (xBlock = 0; xBlock<m_zoneBlockExtent.x; xBlock++) {
		for (yBlock=0; yBlock<m_zoneBlockExtent.y; yBlock++) {
			IRegion2D blockBounds;
			blockBounds.lo.x = globalBounds.lo.x + xBlock*ZONE_BLOCK_SIZE;
			blockBounds.lo.y = globalBounds.lo.y + yBlock*ZONE_BLOCK_SIZE;
			blockBounds.hi.x = blockBounds.lo.x + ZONE_BLOCK_SIZE - 1; // blockBounds are inclusive.
			blockBounds.hi.y = blockBounds.lo.y + ZONE_BLOCK_SIZE - 1; // blockBounds are inclusive.
			blockBounds.intersectWith(bounds);
			if (blockBounds.lo.x>blockBounds.hi.x || blockBounds.lo.y>blockBounds.hi.y) {
				continue;
			}
			m_zoneBlocks[xBlock][yBlock].setInteractsWithBridge(false);
			Int i, j;
			for( j=blockBounds.lo.y; j<=blockBounds.hi.y; j++ )	{
				for( i=blockBounds.lo.x; i<=blockBounds.hi.x; i++ )	{
					PathfindCell *cell = &map[i][j];
					if (cell->getZone()!=UNINITIALIZED_ZONE) continue;

					if (i>blockBounds.lo.x) {
						if (map[i][j].getType() == map[i-1][j].getType()) {
							cell->setZone(map[i-1][j].getZone());
							if (cell->getZone()!=UNINITIALIZED_ZONE) continue;
						}
					}
					if (j>blockBounds.lo.y) {
						if (cell->getType() == map[i][j-1].getType()) {
							cell->setZone(map[i][j-1].getZone());
							if (cell->getZone()!=UNINITIALIZED_ZONE) continue;
						}
						if (i<blockBounds.hi.x) {
							if (PathfindCell::typesMatch(*cell, map[i+1][j-1]) &&
									PathfindCell::typesMatch(*cell, map[i+1][j])) {
								cell->setZone(map[i+1][j-1].getZone());
								if (cell->getZone()!=UNINITIALIZED_ZONE) continue;
							}
						}
					}
				}
			}
			for( j=blockBounds.hi.y; j>=blockBounds.lo.y; j-- )	{
				for( i=blockBounds.hi.x; i>=blockBounds.lo.x; i-- )	{
					PathfindCell *cell = &map[i][j];
					if (cell->getZone()!=UNINITIALIZED_ZONE) continue;
					if (i<blockBounds.hi.x) {
						if (map[i][j].getType() == map[i+1][j].getType()) {
							cell->setZone(map[i+1][j].getZone());
							if (cell->getZone()!=UNINITIALIZED_ZONE) continue;
						}
					}
					if (j<blockBounds.hi.y) {
						if (cell->getType() == map[i][j+1].getType()) {
							cell->setZone(map[i][j+1].getZone());
							if (cell->getZone()!=UNINITIALIZED_ZONE) continue;
						}
						if (i<blockBounds.hi.x) {
							if (PathfindCell::typesMatch(*cell, map[i+1][j+1]) &&
									PathfindCell::typesMatch(*cell, map[i+1][j])) {
								cell->setZone(map[i+1][j+1].getZone());
								if (cell->getZone()!=UNINITIALIZED_ZONE) continue;
							}
						}
					}
				}
			}
		}
	}
#ifdef DEBUG_QPF
#if defined(DEBUG_LOGGING)
	QueryPerformanceCounter((LARGE_INTEGER *)&endTime64);
	timeToUpdate = ((double)(endTime64-startTime64) / (double)(freq64));
#endif
#endif
#if defined(RTS_DEBUG)
	if (TheGlobalData->m_debugAI==AI_DEBUG_ZONES)
	{
		extern void addIcon(const Coord3D *pos, Real width, Int numFramesDuration, RGBColor color);
		RGBColor color;
		memset(&color, 0, sizeof(Color));
		addIcon(nullptr, 0, 0, color);
		Int i, j;
		for( j=0; j<globalBounds.hi.y; j++ )	{
			for( i=0; i<globalBounds.hi.x; i++ )	{
				Int zone = map[i][j].getZone();
				//zone = m_terrainZones[zone];
				zone = m_hierarchicalZones[zone];

				color.blue = (zone%3) * 0.5f;
				zone = zone/3;
				color.green = (zone%3) * 0.5f;
				zone = zone/3;
				color.red = (zone%3) * 0.5;
				Coord3D pos;
				pos.x = ((Real)i + 0.5f) * PATHFIND_CELL_SIZE_F;
				pos.y = ((Real)j + 0.5f) * PATHFIND_CELL_SIZE_F;
				pos.z = TheTerrainLogic->getLayerHeight( pos.x, pos.y, map[i][j].getLayer() ) + 0.5f;
				addIcon(&pos, PATHFIND_CELL_SIZE_F*0.8f, 200, color);
			}
		}
	}
#endif

}

//
// Clear the passable flags.
//
void PathfindZoneManager::clearPassableFlags()
{	Int blockX;
	Int blockY;
	for (blockX = 0; blockX<m_zoneBlockExtent.x; blockX++) {
		for (blockY = 0; blockY<m_zoneBlockExtent.y; blockY++) {
			m_zoneBlocks[blockX][blockY].setPassable(false);
		}
	}
}

//
// Set the passable flags.
//
void PathfindZoneManager::setAllPassable()
{	Int blockX;
	Int blockY;
	for (blockX = 0; blockX<m_zoneBlockExtent.x; blockX++) {
		for (blockY = 0; blockY<m_zoneBlockExtent.y; blockY++) {
			m_zoneBlocks[blockX][blockY].setPassable(true);
		}
	}
}

//
// Set the passable flag for the block at this location.
//
void PathfindZoneManager::setPassable(Int cellX, Int cellY, Bool passable)
{
	Int blockX = cellX/ZONE_BLOCK_SIZE;
	Int blockY = cellY/ZONE_BLOCK_SIZE;

	if (blockX<0 || blockX>=m_zoneBlockExtent.x) {
		DEBUG_CRASH(("Invalid block."));
		return;
	}
	if (blockY<0 || blockY>=m_zoneBlockExtent.y) {
		DEBUG_CRASH(("Invalid block."));
		return;
	}
	m_zoneBlocks[blockX][blockY].setPassable(passable);
}

//
// Get the passable flag for the block at this location.
//
Bool PathfindZoneManager::isPassable(Int cellX, Int cellY) const
{
	Int blockX = cellX/ZONE_BLOCK_SIZE;
	Int blockY = cellY/ZONE_BLOCK_SIZE;

	if (blockX<0 || blockX>=m_zoneBlockExtent.x) {
		DEBUG_CRASH(("Invalid block."));
		return false;
	}
	if (blockY<0 || blockY>=m_zoneBlockExtent.y) {
		DEBUG_CRASH(("Invalid block."));
		return false;
	}
	return m_zoneBlocks[blockX][blockY].isPassable();
}

//
// Get the passable flag for the block at this location.
//
Bool PathfindZoneManager::clipIsPassable(Int cellX, Int cellY) const
{
	Int blockX = cellX/ZONE_BLOCK_SIZE;
	Int blockY = cellY/ZONE_BLOCK_SIZE;

	if (blockX<0 || blockX>=m_zoneBlockExtent.x) {
		return false;
	}
	if (blockY<0 || blockY>=m_zoneBlockExtent.y) {
		return false;
	}
	return m_zoneBlocks[blockX][blockY].isPassable();
}

//
// Set the bridge flag for the block at this location.
//
void PathfindZoneManager::setBridge(Int cellX, Int cellY, Bool bridge)
{
	Int blockX = cellX/ZONE_BLOCK_SIZE;
	Int blockY = cellY/ZONE_BLOCK_SIZE;

	if (blockX<0 || blockX>=m_zoneBlockExtent.x) {
		// DEBUG_CRASH(("Invalid block."));  Bridges can be off the playable grid, so don't crash. jba.
		return;
	}
	if (blockY<0 || blockY>=m_zoneBlockExtent.y) {
		// DEBUG_CRASH(("Invalid block."));  Bridges can be off the playable grid, so don't crash. jba.
		return;
	}
	m_zoneBlocks[blockX][blockY].setInteractsWithBridge(bridge);
}


//
// Set the bridge flag for the block at this location.
//
Bool PathfindZoneManager::interactsWithBridge(Int cellX, Int cellY) const
{
	Int blockX = cellX/ZONE_BLOCK_SIZE;
	Int blockY = cellY/ZONE_BLOCK_SIZE;

	if (blockX<0 || blockX>=m_zoneBlockExtent.x) {
		DEBUG_CRASH(("Invalid block."));
		return false;
	}
	if (blockY<0 || blockY>=m_zoneBlockExtent.y) {
		DEBUG_CRASH(("Invalid block."));
		return false;
	}
	return m_zoneBlocks[blockX][blockY].getInteractsWithBridge();
}


//
// Return the zone at this location.
//
zoneStorageType PathfindZoneManager::getBlockZone(LocomotorSurfaceTypeMask acceptableSurfaces, Bool crusher,Int cellX, Int cellY, PathfindCell **map) const
{
	PathfindCell *cell = &(map[cellX][cellY]);
	Int blockX = cellX/ZONE_BLOCK_SIZE;
	Int blockY = cellY/ZONE_BLOCK_SIZE;

	if (blockX<0 || blockX>=m_zoneBlockExtent.x) {
		DEBUG_CRASH(("Invalid block."));
		return 0;
	}
	if (blockY<0 || blockY>=m_zoneBlockExtent.y) {
		DEBUG_CRASH(("Invalid block."));
		return 0;
	}
	zoneStorageType zone =  m_zoneBlocks[blockX][blockY].getEffectiveZone(acceptableSurfaces, crusher, cell->getZone());
#if RTS_GENERALS && RETAIL_COMPATIBLE_PATHFINDING
	if (zone > m_maxZone) {
#else
	if (zone >= m_maxZone) {
#endif
		DEBUG_CRASH(("Invalid zone."));
		return UNINITIALIZED_ZONE;
	}
	return zone;
}

//
// Return the zone at this location.
//
zoneStorageType PathfindZoneManager::getEffectiveTerrainZone(zoneStorageType zone) const
{
	return m_hierarchicalZones[m_terrainZones[zone]];
}

//
// Return the zone at this location.
//
zoneStorageType PathfindZoneManager::getEffectiveZone( LocomotorSurfaceTypeMask acceptableSurfaces,
																										Bool crusher, zoneStorageType zone) const
{
	//DEBUG_ASSERTCRASH(zone, ("Zone not set"));
	if (zone>m_maxZone) {
		DEBUG_CRASH(("Invalid zone"));
		return (0);
	}
	if (zone>m_maxZone) {
		DEBUG_CRASH(("Invalid zone"));
		return (0);
	}
	if (acceptableSurfaces&LOCOMOTORSURFACE_AIR) return 1; // air is all zone 1.

	if ( (acceptableSurfaces&LOCOMOTORSURFACE_GROUND) &&
			(acceptableSurfaces&LOCOMOTORSURFACE_WATER) &&
			(acceptableSurfaces&LOCOMOTORSURFACE_CLIFF)) {
		// Locomotors can go on ground, water & cliff, so all is zone 1.
		return 1;
	}

	if (crusher) {
		zone = m_crusherZones[zone];
	}

	if ( (acceptableSurfaces&LOCOMOTORSURFACE_GROUND) &&
			(acceptableSurfaces&LOCOMOTORSURFACE_CLIFF)) {
		// Locomotors can go on ground & cliff, so use the ground cliff combiner.
		zone = m_groundCliffZones[zone];
		return zone;
	}

	if ( (acceptableSurfaces&LOCOMOTORSURFACE_GROUND) &&
			(acceptableSurfaces&LOCOMOTORSURFACE_WATER)) {
		// Locomotors can go on ground & water, so use the ground water combiner.
		zone = m_groundWaterZones[zone];
		return zone;
	}

	if ( (acceptableSurfaces&LOCOMOTORSURFACE_GROUND) &&
			(acceptableSurfaces&LOCOMOTORSURFACE_RUBBLE)) {
		// Locomotors can go on ground & rubble, so use the ground rubble combiner.
		zone = m_groundRubbleZones[zone];
		return zone;
	}

	if ( (acceptableSurfaces&LOCOMOTORSURFACE_CLIFF) &&
			(acceptableSurfaces&LOCOMOTORSURFACE_WATER)) {
		// Locomotors can go on ground & cliff, so use the ground cliff combiner.
		DEBUG_CRASH(("Cliff water only locomotor sets not supported yet."));
	}
	zone = m_hierarchicalZones[zone];

	return zone;
}
```

### `Core/GameEngine/Include/GameLogic/Pathfinder/ZoneBlock.h`
```cpp
#pragma once

#include "GameLogic/LocomotorSet.h"

class PathfindCell;
class PathfindLayer;

typedef UnsignedShort zoneStorageType;

/**
 * This class is a helper class for zone manager.  It maintains information regarding the
 * LocomotorSurfaceTypeMask equivalencies within a ZONE_BLOCK_SIZE x ZONE_BLOCK_SIZE area of
 * cells.  This is used in hierarchical pathfinding to find the best coarse path at the
 * block level.
 */
class ZoneBlock
{
public:
	ZoneBlock();
	~ZoneBlock();  // not virtual, please don't override without making virtual.  jba.

	void blockCalculateZones(	PathfindCell **map, PathfindLayer layers[], const IRegion2D &bounds);	///< Does zone calculations.
	zoneStorageType getEffectiveZone(LocomotorSurfaceTypeMask acceptableSurfaces, Bool crusher, zoneStorageType zone) const;

	void clearMarkedPassable() {m_markedPassable = false;}
	Bool isPassable() {return m_markedPassable;}
	void setPassable(Bool pass) {m_markedPassable = pass;}

	Bool getInteractsWithBridge() const {return m_interactsWithBridge;}
	void setInteractsWithBridge(Bool interacts) {m_interactsWithBridge = interacts;}

	static void __fastcall resolveBlockZones(Int srcZone, Int targetZone, zoneStorageType* zoneEquivalency, Int sizeOfZE);
	static void __fastcall resolveZones(Int srcZone, Int targetZone, zoneStorageType* zoneEquivalency, Int sizeOfZE);
	static void flattenZones(zoneStorageType* zoneArray, zoneStorageType* zoneHierarchical, Int sizeOfZones);
	static void applyZone(PathfindCell& targetCell, const PathfindCell& sourceCell, zoneStorageType* zoneEquivalency, Int sizeOfZE);
	static void applyBlockZone(PathfindCell& targetCell, const PathfindCell& sourceCell, zoneStorageType* zoneEquivalency, Int firstZone, Int sizeOfZE);

protected:
	void allocateZones();
	void freeZones();

protected:
	ICoord2D		m_cellOrigin;

	zoneStorageType m_firstZone; // First zone in this block.
	UnsignedShort m_numZones;	 // Number of zones in this block.  If == 1, there is only one zone, and
														 // no zone equivalency arrays will be allocated.

	UnsignedShort m_zonesAllocated;
	zoneStorageType *m_groundCliffZones;
	zoneStorageType *m_groundWaterZones;
	zoneStorageType *m_groundRubbleZones;
	zoneStorageType *m_crusherZones;
	Bool					m_interactsWithBridge;
	Bool					m_markedPassable;
};
typedef ZoneBlock *ZoneBlockP;
```

### `Core/GameEngine/Source/GameLogic/AI/Pathfinder/ZoneBlock.cpp`
```cpp
#include "PreRTS.h"
#include "GameLogic/Pathfinder/PathfindCell.h"
#include "GameLogic/Pathfinder/PathfindZoneManager.h"
#include "GameLogic/Pathfinder/ZoneBlock.h"

ZoneBlock::ZoneBlock() : m_firstZone(0),
m_numZones(0),
m_groundCliffZones(nullptr),
m_groundWaterZones(nullptr),
m_groundRubbleZones(nullptr),
m_crusherZones(nullptr),
m_zonesAllocated(0),
m_interactsWithBridge(FALSE)
{
	m_cellOrigin.x = 0;
	m_cellOrigin.y = 0;
	m_firstZone = 0;
	m_markedPassable = TRUE;
}

ZoneBlock::~ZoneBlock()
{
	freeZones();
}

void ZoneBlock::freeZones()
{
	delete [] m_groundCliffZones;
	m_groundCliffZones = nullptr;

	delete [] m_groundWaterZones;
	m_groundWaterZones = nullptr;

	delete [] m_groundRubbleZones;
	m_groundRubbleZones = nullptr;

	delete [] m_crusherZones;
	m_crusherZones = nullptr;
}

/* Allocate zone equivalency arrays large enough to hold required entries.  If the arrays are already
large enough, reuse.  Then calculate terrain equivalencies. */
void ZoneBlock::blockCalculateZones(PathfindCell **map, PathfindLayer layers[], const IRegion2D &bounds)
{
	Int i, j;
	m_cellOrigin = bounds.lo;
	UnsignedInt minZone = map[bounds.lo.x][bounds.lo.y].getZone();
	UnsignedInt maxZone = minZone;

	for( j=bounds.lo.y; j<=bounds.hi.y; j++ )	{
		for( i=bounds.lo.x; i<=bounds.hi.x; i++ )	{
			PathfindCell *cell = &map[i][j];
			zoneStorageType zone = cell->getZone();
			if (minZone>zone) minZone=zone;
			if (maxZone<zone) maxZone=zone;
		}
	}
	m_firstZone = minZone;
	m_numZones = 1 + maxZone - minZone;

	allocateZones();

	if (m_numZones==1) return; // all zones are equivalent.

	// Determine water/ground equivalent zones, and ground/cliff equivalent zones.
	for (i=0; i<m_zonesAllocated; i++) {
		m_groundCliffZones[i] = i+m_firstZone;
		m_groundWaterZones[i] = i+m_firstZone;
		m_groundRubbleZones[i] = i+m_firstZone;
		m_crusherZones[i] = i+m_firstZone;
	}

	for( j=bounds.lo.y; j<=bounds.hi.y; j++ )	{
		for( i=bounds.lo.x; i<=bounds.hi.x; i++ )	{
			if (i>bounds.lo.x && map[i][j].getZone()!=map[i-1][j].getZone()) {

				if (PathfindCell::waterGround(map[i][j], map[i-1][j])) {
					applyBlockZone(map[i][j], map[i-1][j], m_groundWaterZones, m_firstZone, m_numZones);
				}
				if (PathfindCell::groundRubble(map[i][j], map[i-1][j])) {
					applyBlockZone(map[i][j], map[i-1][j], m_groundRubbleZones, m_firstZone, m_numZones);
				}
				if (PathfindCell::groundCliff(map[i][j], map[i-1][j])) {
					applyBlockZone(map[i][j], map[i-1][j], m_groundCliffZones, m_firstZone, m_numZones);
				}
				if (PathfindCell::crusherGround(map[i][j], map[i-1][j])) {
					applyBlockZone(map[i][j], map[i-1][j], m_crusherZones, m_firstZone, m_numZones);
				}
			}
			if (j>bounds.lo.y && map[i][j].getZone()!=map[i][j-1].getZone()) {
				if (PathfindCell::waterGround(map[i][j],map[i][j-1])) {
					applyBlockZone(map[i][j], map[i][j-1], m_groundWaterZones, m_firstZone, m_numZones);
				}
				if (PathfindCell::groundRubble(map[i][j], map[i][j-1])) {
					applyBlockZone(map[i][j], map[i][j-1], m_groundRubbleZones, m_firstZone, m_numZones);
				}
				if (PathfindCell::groundCliff(map[i][j],map[i][j-1])) {
					applyBlockZone(map[i][j], map[i][j-1], m_groundCliffZones, m_firstZone, m_numZones);
				}
				if (PathfindCell::crusherGround(map[i][j], map[i][j-1])) {
					applyBlockZone(map[i][j], map[i][j-1], m_crusherZones, m_firstZone, m_numZones);
				}
			}
			DEBUG_ASSERTCRASH(map[i][j].getZone() != 0, ("Cleared the zone."));
		}
	}

}

//
// Return the zone at this location.
//
zoneStorageType ZoneBlock::getEffectiveZone( LocomotorSurfaceTypeMask acceptableSurfaces,
																					 Bool crusher, zoneStorageType zone) const
{
#if !(RTS_GENERALS && RETAIL_COMPATIBLE_PATHFINDING)
	if (zone==PathfindZoneManager::UNINITIALIZED_ZONE) {
		return zone;
	}
#endif

	if (acceptableSurfaces&LOCOMOTORSURFACE_AIR) return 1; // air is all zone 1.

	if ( (acceptableSurfaces&LOCOMOTORSURFACE_GROUND) &&
			(acceptableSurfaces&LOCOMOTORSURFACE_WATER) &&
			(acceptableSurfaces&LOCOMOTORSURFACE_CLIFF)) {
		// Locomotors can go on ground, water & cliff, so all is zone 1.
		return 1;
	}
	if (m_numZones<2) {
		return m_firstZone; // if we only got 1 zone, it's all the same zone.
	}
	DEBUG_ASSERTCRASH(zone >=m_firstZone && zone < m_firstZone+m_numZones, ("Invalid range."));
	if (zone<m_firstZone || zone >= m_firstZone+m_numZones) {
		return m_firstZone;
	}
	zone -= m_firstZone;
	if (crusher) {
		zone = m_crusherZones[zone];
		DEBUG_ASSERTCRASH(zone >=m_firstZone && zone < m_firstZone+m_numZones, ("Invalid range."));
		zone -= m_firstZone;
	}

	if ( (acceptableSurfaces&LOCOMOTORSURFACE_GROUND) &&
			(acceptableSurfaces&LOCOMOTORSURFACE_CLIFF)) {
		// Locomotors can go on ground & cliff, so use the ground cliff combiner.
		zone = m_groundCliffZones[zone];
		DEBUG_ASSERTCRASH(zone >=m_firstZone && zone < m_firstZone+m_numZones, ("Invalid range."));
		return zone;
	}

	if ( (acceptableSurfaces&LOCOMOTORSURFACE_GROUND) &&
			(acceptableSurfaces&LOCOMOTORSURFACE_WATER)) {
		// Locomotors can go on ground & water, so use the ground water combiner.
		zone = m_groundWaterZones[zone];
		DEBUG_ASSERTCRASH(zone >=m_firstZone && zone < m_firstZone+m_numZones, ("Invalid range."));
		return zone;
	}

	if ( (acceptableSurfaces&LOCOMOTORSURFACE_GROUND) &&
			(acceptableSurfaces&LOCOMOTORSURFACE_RUBBLE)) {
		// Locomotors can go on ground & rubble, so use the ground rubble combiner.
		zone = m_groundRubbleZones[zone];
		return zone;
	}

	if ( (acceptableSurfaces&LOCOMOTORSURFACE_CLIFF) &&
			(acceptableSurfaces&LOCOMOTORSURFACE_WATER)) {
		// Locomotors can go on ground & cliff, so use the ground cliff combiner.
		DEBUG_CRASH(("Cliff water only locomotor sets not supported yet."));
	}

	return zone+m_firstZone;
}


/* Allocate zone equivalency arrays large enough to hold m_maxZone entries.  If the arrays are already
large enough, just return. */
void ZoneBlock::allocateZones()
{
	if (m_zonesAllocated>m_numZones && m_groundCliffZones!=nullptr) {
		return;
	}
	freeZones();

	if (m_numZones==1) {
		return; // we don't need any zone equivalency tables.
	}

	if (m_zonesAllocated == 0) {
		m_zonesAllocated = 4;
	}
	while (m_zonesAllocated <= m_numZones) {
		m_zonesAllocated *= 2;
	}
	// pool[]ify
	m_groundCliffZones = MSGNEW("PathfindZoneInfo") zoneStorageType [m_zonesAllocated];
	m_groundWaterZones = MSGNEW("PathfindZoneInfo") zoneStorageType[m_zonesAllocated];
	m_groundRubbleZones = MSGNEW("PathfindZoneInfo") zoneStorageType[m_zonesAllocated];
	m_crusherZones = MSGNEW("PathfindZoneInfo") zoneStorageType[m_zonesAllocated];
}

void __fastcall ZoneBlock::resolveBlockZones(Int srcZone, Int targetZone, zoneStorageType* zoneEquivalency, Int sizeOfZE)
{
	Int i;
	// We have two zones being combined now. Keep the lower zone.
	DEBUG_ASSERTCRASH(srcZone != 0 && targetZone != 0, ("Bad resolve zones	."));
	if (targetZone < srcZone)
	{
		for (i = 0; i < sizeOfZE; i++)
		{
			if (zoneEquivalency[i] == srcZone)
			{
				zoneEquivalency[i] = targetZone;
			}
		}
	}
	else
	{
		for (i = 0; i < sizeOfZE; i++)
		{
			if (zoneEquivalency[i] == targetZone)
			{
				zoneEquivalency[i] = srcZone;
			}
		}
	}
}

void __fastcall ZoneBlock::resolveZones(Int srcZone, Int targetZone, zoneStorageType* zoneEquivalency, Int sizeOfZE)
{
	Int i;
	// We have two zones being combined now. Keep the lower zone.
	DEBUG_ASSERTCRASH(srcZone != 0 && targetZone != 0, ("Bad resolve zones	."));
	DEBUG_ASSERTCRASH(srcZone < sizeOfZE && targetZone < sizeOfZE, ("Bad resolve zones	."));
	srcZone = zoneEquivalency[srcZone];
	targetZone = zoneEquivalency[targetZone];
	DEBUG_ASSERTCRASH(srcZone < sizeOfZE && targetZone < sizeOfZE, ("Bad resolve zones	."));
	zoneStorageType finalZone;
	if (targetZone < srcZone)
	{
		finalZone = zoneEquivalency[targetZone];
	}
	else
	{
		finalZone = zoneEquivalency[srcZone];
	}
	DEBUG_ASSERTCRASH(finalZone < sizeOfZE, ("Bad resolve zones	."));
	for (i = 0; i < sizeOfZE; i++)
	{
		zoneStorageType ze = zoneEquivalency[i];
		if (ze == targetZone || ze == srcZone)
		{
			zoneEquivalency[i] = finalZone;
		}
	}
}

void ZoneBlock::flattenZones(zoneStorageType* zoneArray, zoneStorageType* zoneHierarchical, Int sizeOfZones)
{
	Int i;
	for (i = 0; i < sizeOfZones; i++)
	{
		Int zone1 = zoneArray[i];
		Int zone2 = zoneHierarchical[zone1];
		zone1 = zoneArray[zone2];
		zone2 = zoneHierarchical[zone1];
		zoneArray[i] = zone2;
	}
#if 1

	for (i = 0; i < sizeOfZones; i++)
	{
		Int zone1 = zoneArray[i];
		Int zone2 = zoneHierarchical[i];
		if (zone1 != zone2)
		{
			resolveZones(zone1, zone2, zoneArray, sizeOfZones);
		}
	}
#endif
}

void ZoneBlock::applyZone(PathfindCell& targetCell, const PathfindCell& sourceCell, zoneStorageType* zoneEquivalency, Int sizeOfZE)
{
	DEBUG_ASSERTCRASH(sourceCell.getZone() != 0, ("Unset source zone."));
	Int srcZone = zoneEquivalency[sourceCell.getZone()];
	Int targetZone = zoneEquivalency[targetCell.getZone()];

	if (targetZone == 0)
	{
		targetCell.setZone(srcZone);
		return;
	}
	if (targetZone == srcZone)
	{
		return;    // already match.
	}
	resolveZones(srcZone, targetZone, zoneEquivalency, sizeOfZE);
}

void ZoneBlock::applyBlockZone(PathfindCell& targetCell, const PathfindCell& sourceCell, zoneStorageType* zoneEquivalency, Int firstZone, Int sizeOfZE)
{
	DEBUG_ASSERTCRASH(sourceCell.getZone() >= firstZone && sourceCell.getZone() < firstZone + sizeOfZE, ("Memory overrun - FATAL ERROR."));
	Int srcZone = zoneEquivalency[sourceCell.getZone() - firstZone];
	DEBUG_ASSERTCRASH(targetCell.getZone() >= firstZone && sourceCell.getZone() < firstZone + sizeOfZE, ("Memory overrun - FATAL ERROR."));
	Int targetZone = zoneEquivalency[targetCell.getZone() - firstZone];
	if (targetZone == srcZone)
	{
		return;    // already match.
	}
	resolveBlockZones(srcZone, targetZone, zoneEquivalency, sizeOfZE);
}
```
