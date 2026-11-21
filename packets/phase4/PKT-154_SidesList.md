# Translation Packet `PKT-154`: `SidesList`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-154`
- **Work Package:** `WP-4.1` (Phase 4)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer` *(stateful `DataChunk`/`Xfer`/`SkirmishScripts` orchestration)*
- **Environment Access:** `env:ephemeral-web` *(seeded with C++ Release oracle `docs/golden_vectors/PKT-154.json` & compiling Rust skeleton)*
- **FFI Seam Pattern:** Pattern 2: Handle Component
- **Allowed Rust Output Files:**
  - `crates/zh_logic/src/spatial/sides_list.rs` *(implement all `todo!()` methods)*
  - `crates/zh_logic/tests/golden_pkt154_sides_list.rs` *(remove both `#[ignore]` attributes once implemented)*
  - `docs/PORT_MAP.md` *(regenerate via `python3 scripts/port_map.py --write`)*
- **Mandatory V&V Gates:** `V1, V2-Golden`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test -p zh_logic --test golden_pkt154_sides_list && python3 scripts/port_map.py --write && cargo test --workspace && cargo check --workspace --all-targets
  ```

## Port Contract (read first)

1. **`WellKnownSidesKeys` Fixed Intern Order:**
   - The C++ oracle (`docs/golden_vectors/PKT-154.json`) interns keys via `WellKnownSidesKeys::intern` in the exact order: `playerName`, `playerIsHuman`, `playerIsSkirmish`, `playerDisplayName`, `playerFaction`, `playerAllies`, `playerEnemies`, `teamName`, `teamOwner`, `teamIsSingleton`.
   - `Dict` sorts pairs by `NameKeyType` integer key (`Dict.cpp`) and `DataChunkOutput::write_dict` emits TOC entries in LIFO order of pair key allocation, so preserving this exact intern order guarantees byte-identical `sides_chunk_hex` and `sides_xfer_hex`.

2. **`SidesScriptList` Deterministic Codec (Already Implemented):**
   - `crates/zh_logic/src/spatial/sides_list.rs` already implements `SidesScriptList`, `Snapshot`, `write_scripts_data_chunk`, and `parse_scripts_data_chunk` matching the C++ oracle `"PlayerScriptsList"` chunk (`version = 1`, `Int count`, per-slot `Byte present`, `Byte has_script`, `Byte has_group`, `Int tag`) and `xfer` (`version = 1`, `Byte has_script`, `Byte has_group`, `Int tag`). Do not change its wire format.

3. **`SidesList` / `SidesInfo` / `TeamsInfoRec` Capacity & Refusal Guards:**
   - `MAX_PLAYER_COUNT = 16`, `MAX_TEAM_DEPTH = 3`, `MAX_SKIRMISH_SIDES = 64` (`#335` expanded skirmish capacity).
   - `remove_side(i)` and `TeamsInfoRec::remove_team(i)` MUST refuse removal if `len <= 1` (`if i < 0 || i >= len || len <= 1 { return; }`).
   - `add_side` silently ignores additions once `sides.len() >= MAX_PLAYER_COUNT`; `add_skirmish_side` silently ignores additions once `skirmish_sides.len() >= MAX_SKIRMISH_SIDES`.
   - Out-of-range `side_info(i)`, `skirmish_side_info(i)`, and `team_info(i)` return `Err(SidesError::BadArg)`.

4. **`SidesInfo` Build List Ordering (`Vec<BuildListInfo>` replacing intrusive linked list):**
   - `add_to_build_list(item, position)`: mimics C++ walk (`position <= 0` prepends at index `0`; `position >= len` appends at tail; `0 < position < len` inserts at `position`).
   - `remove_from_build_list(index)`: removes item at `index` (returning `index as Int`; `0` if out of bounds).
   - `reorder_in_build_list(index, new_position)`: removes at `index`, then calls `add_to_build_list(item, new_position)`.

5. **`BuildListInfo` Defaults, INI & Xfer:**
   - Defaults (`BuildListInfo::new()`): `health = 100`, `whiner = TRUE`, `repairable = TRUE`, `automatically_build = TRUE`, `object_id = ObjectId::INVALID`, `resource_gatherers = [INVALID; 10]`, all other fields `0`/`FALSE`/empty.
   - `decrement_num_rebuilds` & `increment_num_rebuilds`: leave `UNLIMITED_REBUILDS` (`0xFFFF_FFFF`) untouched; `is_buildable()` is `TRUE` iff `num_rebuilds > 0 || num_rebuilds == UNLIMITED_REBUILDS`.
   - `parse_structure(ini)`: consumes `template_name` token first (`ini.next_token(SEPS_DEFAULT)`), then calls `ini.init_from_ini` with `Name`, `Location` (`parse_coord2d` into `location.x`/`location.y`, keeping `location.z = 0.0`), `Rebuilds`, `Angle` (`parse_angle_real`), `InitiallyBuilt`, `RallyPointOffset` (`parse_coord2d`), `AutomaticallyBuild`.
   - `Snapshot::xfer`: version `2` (`currentVersion = 2`): xfers `building_name`, `template_name`, `location` (`Coord3D`), `rally_point_offset` (`Coord2D`), `angle` (`Real`), `is_initially_built`, `num_rebuilds` (`UnsignedInt`), `script`, `health`, `whiner`, `unsellable`, `repairable`, `automatically_build`, `object_id`, `object_timestamp`, `under_construction`, **`resource_gatherers` as 40 raw LE bytes (`xfer_bytes`, matching C++ `xferUser(m_resourceGatherers, sizeof(ObjectID) * 10)`)**, `is_supply_building`, `desired_gatherers`, `priority_build`, and if `version >= 2`, `current_gatherers`.

6. **`SidesList::validate_sides` Exact Sequence:**
   - Ensure at least one neutral side (`playerName == ""`): if none, call `add_player_by_template("", keys)` and set `modified = TRUE`. (`add_player_by_template`: `""` -> `playerName=""`, `display="Neutral"`, `isHuman=FALSE`; non-empty `"FactionX"` strips `"Faction"` -> `"PlyrX"`; else `"Plyr" + template`; `PlyrCivilian` forces `isHuman=FALSE`; adds default `"team" + playerName` owned by `playerName`, `teamIsSingleton = TRUE`.)
   - For each side `0..num_sides`, ensure default team `"team" + pname` exists (repairing `teamOwner = pname` and `teamIsSingleton = TRUE` if mismatched, or adding if missing), and prune `playerAllies` & `playerEnemies` via whitespace tokenization (`" \n\r\t"`), removing self (`token == pname`) and unknown players (`find_side_info(token).is_none()`), joining kept tokens with single spaces `" "`.
   - Loop from start (`validate_team_names` restart loop): remove any team whose `teamName` matches any `side`'s `playerName`.
   - For remaining teams, if `teamOwner` is not found in sides OR `teamOwner == teamName`, reparent `teamOwner` to `""`.

7. **`DataChunk`, `prepare_for_mp_or_skirmish`, and `SidesList::xfer`:**
   - `write_sides_data_chunk`: opens `"SidesList"` v`3` (`SIDES_DATA_VERSION_3`), writes `num_sides`, each side's `Dict` + build list (`count` then each `BuildListInfo` including v3 fields), `num_teams` + each team `Dict`, `"PlayerScriptsList"` via `SidesScriptList::write_scripts_data_chunk`, closes chunk, then runs `validate_sides(keys)`.
   - `parse_sides_chunk_stream`: reads `"SidesList"` chunk (`loc.z = 0.0` forced on load!), optional v3 build-list fields if `version >= 3`, v2 teams if `version >= 2`, nested `"PlayerScriptsList"`, discards excess sides past `MAX_PLAYER_COUNT` and excess scripts, runs `validate_sides(keys)`.
   - `prepare_for_mp_or_skirmish`: snapshots `team_rec` -> `skirmish_team_rec` and clears `team_rec`; clears `skirmish_sides` and copies `sides` -> `skirmish_sides`, removing non-`"FactionCivilian"` sides from `sides` unless `sides.len() == 1` (`remove_side`); checks `got_scripts` across non-`"FactionCivilian"` skirmish sides with non-empty (`!is_empty()`) `SidesScriptList`; if `!got_scripts` and `skirmish_scripts_scb` is `Some(bytes)`, clears `skirmish_team_rec` and parses `"ScriptsPlayers"` (v1/v2, `#335` mod-side import when `!player_name.is_empty()` and not already in `skirmish_sides`, setting `playerName` if dict's `playerName` is empty), `"ScriptTeams"` (skipping if owner missing in `skirmish_sides`), and `"PlayerScriptsList"` (matching player names from `ScriptsPlayers` in order to `skirmish_sides`!).
   - `SidesList::xfer`: version `1`, xfers `side_count` (must equal `self.num_sides()` else `Err(XferError::InvalidData)`), then per side xfers `script_list_present: Bool` and `SidesScriptList` snapshot.

8. **Determinism & 3-Attempt Stall Escalation Rule:** Zero external crates, no `unsafe`, no `HashMap`/`HashSet`/`BinaryHeap`, no `Rc`/`RefCell`, no std float math. If 3 consecutive fix attempts fail to shrink failing vectors, isolate the minimal failing seed or escalate (`tier:3-architect`).

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (15):**
  - `Common/DataChunk.h`
  - `Common/Dict.h`
  - `Common/Errors.h`
  - `Common/GameMemory.h`
  - `Common/GameState.h`
  - `Common/GameType.h`
  - `Common/PlayerTemplate.h`
  - `Common/STLTypedefs.h`
  - `Common/Snapshot.h`
  - `Common/WellKnownKeys.h`
  - `Common/Xfer.h`
  - `GameLogic/AI.h`
  - `GameLogic/Scripts.h`
  - `GameLogic/SidesList.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (7):**
  - `"Name" => INI::parseAsciiString`
  - `"Location" => INI::parseCoord2D`
  - `"Rebuilds" => INI::parseInt`
  - `"Angle" => INI::parseAngleReal`
  - `"InitiallyBuilt" => INI::parseBool`
  - `"RallyPointOffset" => INI::parseCoord2D`
  - `"AutomaticallyBuild" => INI::parseBool`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (26 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferInt( &sideCount )`
  3. `xferBool( &scriptListPresent )`
  4. `xferSnapshot( scriptList )`
  5. `xferVersion( &version, currentVersion )`
  6. `xferAsciiString( &m_buildingName )`
  7. `xferAsciiString( &m_templateName )`
  8. `xferCoord3D( &m_location )`
  9. `xferCoord2D( &m_rallyPointOffset )`
  10. `xferReal( &m_angle )`
  11. `xferBool( &m_isInitiallyBuilt )`
  12. `xferUnsignedInt( &m_numRebuilds )`
  13. `xferAsciiString( &m_script )`
  14. `xferInt( &m_health )`
  15. `xferBool( &m_whiner )`
  16. `xferBool( &m_unsellable )`
  17. `xferBool( &m_repairable )`
  18. `xferBool( &m_automaticallyBuild )`
  19. `xferObjectID( &m_objectID )`
  20. `xferUnsignedInt( &m_objectTimestamp )`
  21. `xferBool( &m_underConstruction )`
  22. `xferUser( m_resourceGatherers, sizeof( ObjectID ) * MAX_RESOURCE_GATHERERS )`
  23. `xferBool( &m_isSupplyBuilding )`
  24. `xferInt( &m_desiredGatherers )`
  25. `xferBool( &m_priorityBuild )`
  26. `xferInt(&m_currentGatherers)`

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

### `Core/GameEngine/Include/GameLogic/SidesList.h`
```cpp
// SidesList.h
// Class to encapsulate Sides and Build Lists for maps.
// Author: John Ahlquist, November 2001

#pragma once

#include "Common/Dict.h"
#include "Common/Errors.h"
#include "Common/GameType.h"
#include "Common/Snapshot.h"
#include "Common/GameMemory.h"
#include "Common/STLTypedefs.h"

class DataChunkInput;
struct DataChunkInfo;
class DataChunkOutput;
class BuildListInfo;
class RenderObjClass;
class ScriptList;
class Shadow;

// ----------------------------------------------------------------------------------------------
/**
	This is a class that describes a Side, including build list.
	Note that a side corresponds to a Player in the game.  The lightweight
	Side is used to give the WB Editor somewhere to hang build lists.
*/
class SidesInfo
{
protected:
	BuildListInfo*	m_pBuildList;		///< linked list.
	Dict						m_dict;					///< general player dict.
	ScriptList			*m_scripts;			///< linked list.

public:
	SidesInfo();
	SidesInfo(const SidesInfo& thatref);
	~SidesInfo();
	void init(const Dict* d);
	void clear() { init(nullptr); }
	Dict* getDict() { return &m_dict; }
	void addToBuildList(BuildListInfo *pBuildList, Int position);
	Int removeFromBuildList(BuildListInfo *pBuildList);
	void reorderInBuildList(BuildListInfo *pBuildList, Int newPosition);
	BuildListInfo* getBuildList() {return m_pBuildList;} ///< Gets the build list.
	void releaseBuildList()  {m_pBuildList=nullptr;} ///< Used when the build list is passed to class Player.
	ScriptList *getScriptList() {return(m_scripts);};
	void setScriptList(ScriptList *pScriptList) {m_scripts = pScriptList;};

	// ug, I hate having to overload stuff, but this makes it a lot easier to make copies safely
	SidesInfo& operator=(const SidesInfo& that);
};

// ----------------------------------------------------------------------------------------------
class TeamsInfo
{
private:
	Dict m_dict;
public:
	Dict* getDict() { return &m_dict; }
	void init(const Dict* d) { m_dict.clear(); if (d) m_dict = *d; }
	void clear() { init(nullptr); }
};


// ----------------------------------------------------------------------------------------------
// a wrapper class to make this a little cleaner.
class TeamsInfoRec
{
private:
	Int					m_numTeams;
	Int					m_numTeamsAllocated;
	TeamsInfo*	m_teams;

public:
	TeamsInfoRec();
	TeamsInfoRec(const TeamsInfoRec& thatref);
	~TeamsInfoRec();
	TeamsInfoRec& operator=(const TeamsInfoRec& thatref);
	void clear();
	TeamsInfo *findTeamInfo(AsciiString name, Int* index);
	void addTeam(const Dict* d);
	void removeTeam(Int i);
	Int getNumTeams() const { return m_numTeams; }
	TeamsInfo * getTeamInfo(Int team)
	{
		if (team>=0&&team<m_numTeams)
		{
			return(&m_teams[team]);
		}
		DEBUG_CRASH(("Out of range."));
		throw ERROR_BAD_ARG;
		return nullptr;
	}
};

// ----------------------------------------------------------------------------------------------
/**
	This is a singleton class that maintains the list of Sides.
	In RTS it is associated with the Player singleton.
	Note that a side corresponds to a Player in the game.  The lightweight
	Side is used to give the WB Editor somewhere to hang build lists, as
	it doesn't instantiate the Player subsystem

	jba.
*/
class SidesList : public SubsystemInterface,
									public Snapshot
{

public:

	SidesList();
	virtual ~SidesList() override;

	virtual void init() override { }
	virtual void update() override { }
	virtual void reset() override;

	/// Reads sides (including build list info && player dicts.)
	static Bool ParseSidesDataChunk(DataChunkInput &file, DataChunkInfo *info, void *userData);
	/// Writes sides (including build list info.)
	static void WriteSidesDataChunk(DataChunkOutput &chunkWriter);

	Int getNumSides() { return m_numSides; }
	void emptySides();
	void addSide(const Dict* d);
	void removeSide(Int i);
	inline SidesInfo *getSideInfo(Int side);
	inline SidesInfo *getSkirmishSideInfo(Int side);
	Int getNumSkirmishSides() { return m_numSkirmishSides; }
	// GeneralsX @bugfix fbraz3 05/10/2026 Allow importing missing skirmish player definitions from SkirmishScripts chunk (#335).
	void addSkirmishSide(const Dict* d);
	SidesInfo *findSideInfo(AsciiString name, Int* index = nullptr);
	SidesInfo *findSkirmishSideInfo(AsciiString name, Int* index = nullptr);

	void prepareForMP_or_Skirmish(); // After a map is loaded, save & clear any players.

	Int getNumTeams() { return m_teamrec.getNumTeams(); }
	void emptyTeams();
	void addTeam(const Dict* d);
	void removeTeam(Int i);

	inline TeamsInfo *getTeamInfo(Int team);
	inline TeamsInfo *getSkirmishTeamInfo(Int team);
	Int getNumSkirmishTeams() { return m_skirmishTeamrec.getNumTeams(); }
 	void addSkirmishTeam(const Dict* d);

	TeamsInfo *findTeamInfo(AsciiString name, Int* index = nullptr);

	Bool isPlayerDefaultTeam(TeamsInfo *t);

	void clear();
	Bool validateSides();

	void addPlayerByTemplate(AsciiString playerTemplateName);

	enum
	{
		MAX_TEAM_DEPTH = 3,
		// GeneralsX @bugfix fbraz3 05/10/2026 Expand skirmish sides capacity to accommodate mod rosters on custom maps (#335).
		MAX_SKIRMISH_SIDES = 64
	};

protected:

	// snapshot methods
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

	Int					m_numSides;
	SidesInfo		m_sides[MAX_PLAYER_COUNT];

	Int					m_numSkirmishSides;
	// GeneralsX @bugfix fbraz3 05/10/2026 Size skirmish sides array with MAX_SKIRMISH_SIDES to prevent buffer overflow (#335).
	SidesInfo		m_skirmishSides[MAX_SKIRMISH_SIDES];

	TeamsInfoRec	m_teamrec;
	TeamsInfoRec	m_skirmishTeamrec;

	Bool validateAllyEnemyList(const AsciiString& tname, AsciiString& allies);
};


inline TeamsInfo * SidesList::getTeamInfo(Int team)
{
	return m_teamrec.getTeamInfo(team);
}

inline TeamsInfo * SidesList::getSkirmishTeamInfo(Int team)
{
	return m_skirmishTeamrec.getTeamInfo(team);
}

inline SidesInfo * SidesList::getSideInfo(Int side)
{
	if (side>=0&&side<m_numSides)
	{
		return(&m_sides[side]);
	}
	DEBUG_CRASH(("Out of range."));
	throw ERROR_BAD_ARG;
	return nullptr;
}

inline SidesInfo * SidesList::getSkirmishSideInfo(Int side)
{
	if (side>=0&&side<m_numSkirmishSides)
	{
		return(&m_skirmishSides[side]);
	}
	DEBUG_CRASH(("Out of range."));
	throw ERROR_BAD_ARG;
	return nullptr;
}


// ----------------------------------------------------------------------------------------------
extern SidesList *TheSidesList;	 ///< singleton instance of SidesList


// ----------------------------------------------------------------------------------------------
/**
	This is a class that describes an entry in the build list.
*/
class BuildListInfo : public MemoryPoolObject, public Snapshot
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(BuildListInfo, "BuildListInfo")

// friend doesn't play well with MPO... it reveals delete, which shouldn't be visible, even to friends (srj)
//friend class SidesInfo;  // This is essentially a component of the SidesInfo data structure.
//friend class SidesList;  // This is essentially a component of the SidesInfo data structure.
//friend class BuildList;  // This is the class in WB that edits build lists.
//friend class BuildListTool; // Other class in wb that edits build lists.
//friend class WBDocUndoable; // Other class in wb that edits build lists.
//friend class Player;
//friend class AISideBuildList;
//friend class AISkirmishPlayer;

public:
	enum {UNLIMITED_REBUILDS=0xFFFFFFFF};
	enum {MAX_RESOURCE_GATHERERS = 10};
	BuildListInfo();
	//~BuildListInfo();								///< Note that deleting the head of a list deletes all linked objects in the list.

	static void parseStructure(INI *ini, void *instance, void* /*store*/, const void* /*userData*/);

protected:

	// snapshot methods
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

	AsciiString			m_buildingName;			///< The name of this building.
	AsciiString			m_templateName;			///< The thing template name for this model's info.
	Coord3D					m_location;					///< The location of the object.
	Coord2D					m_rallyPointOffset; ///< Offset to the natural rally point.
	Real						m_angle;						///< Initial orientation of the building.
	Bool						m_isInitiallyBuilt; ///< Whether the building is built at the start of the game.
	UnsignedInt			m_numRebuilds;			///< Number of rebuilds allowed.
	BuildListInfo*	m_nextBuildList;		///< linked list.
	AsciiString		m_script;
	Int				m_health;
	Bool			m_whiner;
	Bool			m_unsellable;
	Bool			m_repairable;
	Bool			m_automaticallyBuild;			///< If true, the ai will build.  If false, script has to enable this.


	// For WorldBuilder use only:
	RenderObjClass	*m_renderObj;				///< object that renders in the 3d scene.
	Shadow			*m_shadowObj;				///< object that renders shadows in the 3d scane.
	Bool				m_selected;					///< True if the obj is selected in the editor.

	// For solo AI use only:
	ObjectID m_objectID;								///< the object on the map instantiated by this info
	UnsignedInt m_objectTimestamp;			///< frame when object was built
	Bool		 m_underConstruction;				///< True if being constructed by dozer.
	ObjectID m_resourceGatherers[MAX_RESOURCE_GATHERERS];	///< gatherers for this supply center type building.
	Bool		 m_isSupplyBuilding;				///< Uses gatherers to get supplies.
	Int			 m_desiredGatherers;				///< Number of gatherers desired.
	Int			 m_currentGatherers;				///< Number of gatherers available.
	Bool		 m_priorityBuild;						///< Is priority build.

public:
	// srj sez: naughty public access to avoid 'friend' -- should be friend for JUST THESE, but hey
	void setNumRebuilds(UnsignedInt numRebuilds) {m_numRebuilds = numRebuilds;}
	void setNextBuildList(BuildListInfo *pNext) {m_nextBuildList = pNext;}
	void setLocation(Coord3D loc) {m_location = loc;}
	void setAngle(Real angle) {m_angle = angle;}
	void setInitiallyBuilt(Bool built) {m_isInitiallyBuilt = built;}
	void setBuildingName(AsciiString name) {m_buildingName = name;}
	void setScript(AsciiString script) {m_script = script;}
	void setHealth(Int health) {m_health = health;}
	void setWhiner(Bool whiner) {m_whiner = whiner;}
	void setUnsellable(Bool unsellable) {m_unsellable = unsellable;}
	void setRepairable(Bool repairable) {m_repairable = repairable;}

public:
	BuildListInfo *getNext() const {return m_nextBuildList;}
	AsciiString getBuildingName() const {return m_buildingName;} ///< Gets the name.
	AsciiString getTemplateName() const {return m_templateName;} ///< Gets the name.
	void setTemplateName(AsciiString name) {m_templateName = name;}
	Int getNumRebuilds() {return m_numRebuilds;}
	void decrementNumRebuilds();
	void incrementNumRebuilds();
	const Coord3D *getLocation() const {return &m_location;}
	const Coord2D *getRallyOffset() const {return &m_rallyPointOffset;}
	Real getAngle() const {return m_angle;}
	Bool isInitiallyBuilt() {return m_isInitiallyBuilt;}
	const AsciiString& getScript() const {return m_script;}
	Int getHealth() {return m_health;}
	Bool getWhiner() {return m_whiner;}
	Bool getUnsellable() {return m_unsellable;}
	Bool getRepairable() {return m_repairable;}

	void setRenderObj(RenderObjClass *pObj) {m_renderObj = pObj;}
	RenderObjClass *getRenderObj() {return m_renderObj;}
	void setShadowObj(Shadow *pObj) {m_shadowObj = pObj;}
	Shadow *getShadowObj() {return m_shadowObj;}

	void setSelected(Bool sel) {m_selected = sel;}
	Bool isSelected() {return m_selected;}

	// used by the solo AI
	void setObjectID( ObjectID objID ) { m_objectID = objID; }
	ObjectID getObjectID() const { return m_objectID; }
	void setObjectTimestamp( UnsignedInt frame ) { m_objectTimestamp = frame; }
	UnsignedInt getObjectTimestamp() const { return m_objectTimestamp; }
	Bool isBuildable();															///< returns true if has enough rebuilds left to build again
	Bool isUnderConstruction() {return m_underConstruction;}
	void setUnderConstruction(Bool construction) { m_underConstruction=construction;}
	void markPriorityBuild() {m_priorityBuild = true; }
	Bool isPriorityBuild() {return m_priorityBuild;}
	Bool isAutomaticBuild() {return m_automaticallyBuild;}

	Bool isSupplyBuilding() {return m_isSupplyBuilding;}
	void setSupplyBuilding(Bool isSupply) {m_isSupplyBuilding = isSupply;}
	ObjectID getGathererID(Int ndx) {if (ndx>=0 && ndx < MAX_RESOURCE_GATHERERS) return m_resourceGatherers[ndx]; return INVALID_ID;}
	void setGathererID(Int ndx, ObjectID id)  {if (ndx>=0 && ndx < MAX_RESOURCE_GATHERERS) m_resourceGatherers[ndx] = id;}
	Int getDesiredGatherers() {return m_desiredGatherers;};
	void setDesiredGatherers(Int desired) {m_desiredGatherers = desired;}
	Int getCurrentGatherers() {return m_currentGatherers;};
	void setCurrentGatherers(Int cur) {m_currentGatherers = cur;}

	BuildListInfo *duplicate();
};

inline void BuildListInfo::decrementNumRebuilds()
{
	if (m_numRebuilds > 0 && m_numRebuilds != UNLIMITED_REBUILDS)
		m_numRebuilds--;
}

inline void BuildListInfo::incrementNumRebuilds()
{
	if (m_numRebuilds != UNLIMITED_REBUILDS)
		m_numRebuilds++;
}

inline Bool BuildListInfo::isBuildable()
{
	if (getNumRebuilds() > 0 || getNumRebuilds() == BuildListInfo::UNLIMITED_REBUILDS)
		return true;

	return false;
}
```

### `Core/GameEngine/Source/GameLogic/Map/SidesList.cpp`
```cpp
// FILE: SidesList.cpp /////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------
//
//                       Westwood Studios Pacific.
//
//                       Confidential Information
//                Copyright (C) 2001 - All Rights Reserved
//
//-----------------------------------------------------------------------------
//
// Project:   RTS3
//
// File name: SidesList.cpp
//
// Created:   John Ahlquist, Nov 2001
//
// Desc:      Contains the information describing Sides (player, ai, neutral etc.)
//						in a scenario, including build lists for non-player sides.
//
//-----------------------------------------------------------------------------

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/DataChunk.h"
#include "Common/GameState.h"
#include "Common/PlayerTemplate.h"
#include "Common/WellKnownKeys.h"
#include "Common/Xfer.h"
#include "GameLogic/AI.h"
#include "GameLogic/Scripts.h"
#include "GameLogic/SidesList.h"

static const Int K_SIDES_DATA_VERSION_1 = 1;
static const Int K_SIDES_DATA_VERSION_2 = 2;	// includes Team list.
static const Int K_SIDES_DATA_VERSION_3 = 3;	// includes Team list.

/* ********* SidesInfo class ****************************/
/**
 SidesInfo - Constructor.
*/
SidesInfo::SidesInfo() :
	m_pBuildList(nullptr),
	m_scripts(nullptr)
{
}

SidesInfo::SidesInfo(const SidesInfo& thatref) :
	m_pBuildList(nullptr),
	m_scripts(nullptr)
{
	*this = thatref;
}

/**
 SidesInfo - Destructor -
*/
SidesInfo::~SidesInfo()
{
	clear();
}

void SidesInfo::init(const Dict* d)
{
	deleteInstance(m_pBuildList);
	m_pBuildList = nullptr;

	m_dict.clear();

	deleteInstance(m_scripts);
	m_scripts = nullptr;

	if (d)
		m_dict = *d;
}

// ug, I hate having to overload stuff, but this makes it a lot easier to make copies safely
SidesInfo& SidesInfo::operator=(const SidesInfo& that)
{
	if (this != &that)
	{
		this->clear();
		this->m_dict = that.m_dict;

		BuildListInfo* thisBLTail = nullptr;
		for (BuildListInfo* thatBL = that.m_pBuildList; thatBL; thatBL = thatBL->getNext())
		{
			BuildListInfo* thisBL = newInstance( BuildListInfo );
			*thisBL = *thatBL;
			thisBL->setNextBuildList(nullptr);

			if (thisBLTail)
				thisBLTail->setNextBuildList(thisBL);
			else
				this->m_pBuildList = thisBL;

			thisBLTail = thisBL;
		}

		if (that.m_scripts)
			this->m_scripts = that.m_scripts->duplicate();
		else
			this->m_scripts = nullptr;
	}
	return *this;
}

/**
* SidesInfo::addToBuildList - Adds a build list entry as the nth entry.
*
*/
void SidesInfo::addToBuildList(BuildListInfo *pBuildList, Int position)
{
	DEBUG_ASSERTLOG(pBuildList->getNext()==nullptr, ("WARNING***Adding already linked element."));
	BuildListInfo *pCur = nullptr;
	while (position) {
		position--;
		if (pCur==nullptr) {
			pCur = m_pBuildList;
		} else {
			if (pCur->getNext()) {
				pCur = pCur->getNext();
			}	else {
				break; // at end of list.
			}
		}
	}
	if (pCur==nullptr) {
		// add to front of list.
		pBuildList->setNextBuildList(m_pBuildList);
		m_pBuildList = pBuildList;
	}	else {
		pBuildList->setNextBuildList(pCur->getNext());
		pCur->setNextBuildList(pBuildList);
	}
}

/**
* SidesInfo::reorderInBuildList - Reorders a build list entry as the nth entry.
*
*/
void SidesInfo::reorderInBuildList(BuildListInfo *pBuildList, Int newPosition)
{
	/*Int oldPos =*/ removeFromBuildList(pBuildList);
	addToBuildList(pBuildList, newPosition);
}

/**
* SidesInfo::removeFromBuildList - Removes a build list entry.
* Returns the position in the list that the item occupied.
*
*/
Int SidesInfo::removeFromBuildList(BuildListInfo *pBuildList)
{
	DEBUG_ASSERTCRASH(pBuildList, ("Removing null list."));
	if (pBuildList==nullptr) return 0;

	Int position = 0;

	if (pBuildList == m_pBuildList) {
		// First item in list, so update head.
		m_pBuildList = pBuildList->getNext();
	} else {
		position = 1;
		// Not the first item, so find the preceding list element.
		BuildListInfo *pPrev = m_pBuildList;
		while (pPrev && (pPrev->getNext()!=pBuildList) ) {
			pPrev = pPrev->getNext();
			position++;
		}
		DEBUG_ASSERTCRASH(pPrev, ("Removing item not in list."));
		if (pPrev) {
			pPrev->setNextBuildList(pBuildList->getNext());
		}
	}
	pBuildList->setNextBuildList(nullptr);
	return position;
}

/* ********* SidesList class ****************************/
/*extern*/ SidesList *TheSidesList = nullptr;	 ///< singleton instance of SidesList
/**
 SidesList - Constructor.
*/
SidesList::SidesList() : m_numSides(0), m_numSkirmishSides(0)
{
}

/**
 SidesList - Destructor -
*/
SidesList::~SidesList()
{
}

/**
 SidesList - reset -
*/
void SidesList::reset()
{
	clear();
}

/**
 SidesList - clear -
*/
void SidesList::clear()
{
	emptySides();
	emptyTeams();
}



/**
* SidesList::ParseSidesDataChunk - read a Sides chunk.
* Format is the newer CHUNKY format.
*	See SidesList::WriteSidesDataChunk for the writer.
*	Input: DataChunkInput
*
*/
Bool SidesList::ParseSidesDataChunk(DataChunkInput &file, DataChunkInfo *info, void *userData)
{
	DEBUG_ASSERTCRASH(TheSidesList, ("TheSidesList is null"));

	if (TheSidesList==nullptr)
		return false;

	TheSidesList->clear();
	Int count = file.readInt();
	Int i, j;
	TheSidesList->emptySides();
	for (i=0; i<count; i++)
	{
		if (i >= MAX_PLAYER_COUNT) break;
		Dict d =  file.readDict();
		TheSidesList->addSide(&d);
		BuildListInfo* pBuildList;
		Int count = file.readInt();
		for (j=0; j<count; j++)
		{
			pBuildList = newInstance( BuildListInfo );
			pBuildList->setBuildingName(file.readAsciiString());
			pBuildList->setTemplateName(file.readAsciiString());
			Coord3D loc;
			loc.x = file.readReal();
			loc.y = file.readReal();
			loc.z = file.readReal();
			loc.z = 0; // force to ground level
			pBuildList->setLocation(loc);
			pBuildList->setAngle(file.readReal());
			pBuildList->setInitiallyBuilt(file.readByte());
			pBuildList->setNumRebuilds(file.readInt());
			if (info->version >= K_SIDES_DATA_VERSION_3)
			{
				pBuildList->setScript(file.readAsciiString());
				pBuildList->setHealth(file.readInt());
				pBuildList->setWhiner(file.readByte());
				pBuildList->setUnsellable(file.readByte());
				pBuildList->setRepairable(file.readByte());
			}
			TheSidesList->getSideInfo(i)->addToBuildList(pBuildList, j);
		}
	}
	if (info->version >= K_SIDES_DATA_VERSION_2)
	{
		count = file.readInt();
		TheSidesList->emptyTeams();
		for (i=0; i<count; i++)
		{
			Dict d =  file.readDict();
			TheSidesList->addTeam(&d);
		}
	}

	file.registerParser( "PlayerScriptsList", info->label, ScriptList::ParseScriptsDataChunk );
	if (!file.parse(nullptr)) {
		throw(ERROR_CORRUPT_FILE_FORMAT);
	}
	ScriptList *scripts[MAX_PLAYER_COUNT];
	count = ScriptList::getReadScripts(scripts);
	for (i=0; i<count; i++) {
		if (i<TheSidesList->getNumSides()) {
			deleteInstance(TheSidesList->getSideInfo(i)->getScriptList());
			TheSidesList->getSideInfo(i)->setScriptList(scripts[i]);
			scripts[i] = nullptr;
		} else {
			// Read in more players worth than we have.
			deleteInstance(scripts[i]);
			scripts[i] = nullptr;
		}
	}
	TheSidesList->validateSides();

	DEBUG_ASSERTCRASH(file.atEndOfChunk(), ("Incorrect data file length."));
	return true;
}


/**
* SidesList::WriteSidesDataChunk - Writes a Sides chunk.
* Format is the newer CHUNKY format.
*	See SidesList::ParseSidesDataChunk for the reader.
*	Input: DataChunkInput
*
*/
void SidesList::WriteSidesDataChunk(DataChunkOutput &chunkWriter)
{
	DEBUG_ASSERTCRASH(TheSidesList, ("TheSidesList is null"));
	if (TheSidesList==nullptr)
		return;
	/**********HEIGHT MAP DATA ***********************/
	chunkWriter.openDataChunk("SidesList", K_SIDES_DATA_VERSION_3);

		chunkWriter.writeInt(TheSidesList->getNumSides());
		Int i;
		for (i=0; i<TheSidesList->getNumSides(); i++) {
			chunkWriter.writeDict(*TheSidesList->getSideInfo(i)->getDict());
			BuildListInfo* pBuildList = TheSidesList->getSideInfo(i)->getBuildList();
			Int count = 0;
			while (pBuildList) {
				count++;
				pBuildList = pBuildList->getNext();
			}
			chunkWriter.writeInt(count);
			pBuildList = TheSidesList->getSideInfo(i)->getBuildList();
			while (pBuildList) {
				chunkWriter.writeAsciiString(pBuildList->getBuildingName());
				chunkWriter.writeAsciiString(pBuildList->getTemplateName());
				chunkWriter.writeReal(pBuildList->getLocation()->x);
				chunkWriter.writeReal(pBuildList->getLocation()->y);
				chunkWriter.writeReal(pBuildList->getLocation()->z);
				chunkWriter.writeReal(pBuildList->getAngle());
				chunkWriter.writeByte(pBuildList->isInitiallyBuilt());
				chunkWriter.writeInt(pBuildList->getNumRebuilds());
				// BEGIN stuff new to K_SIDES_DATA_VERSION_3
				chunkWriter.writeAsciiString(pBuildList->getScript());
				chunkWriter.writeInt(pBuildList->getHealth());
				chunkWriter.writeByte(pBuildList->getWhiner());
				chunkWriter.writeByte(pBuildList->getUnsellable());
				chunkWriter.writeByte(pBuildList->getRepairable());
				// END stuff new to K_SIDES_DATA_VERSION_3

				pBuildList = pBuildList->getNext();
			}
		}

		// BEGIN stuff new to K_SIDES_DATA_VERSION_2
		chunkWriter.writeInt(TheSidesList->getNumTeams());
		for (i=0; i<TheSidesList->getNumTeams(); i++) {
			chunkWriter.writeDict(*TheSidesList->getTeamInfo(i)->getDict());
		}
		// END stuff new to K_SIDES_DATA_VERSION_2

		ScriptList *scripts[MAX_PLAYER_COUNT];
		for (i=0; i<TheSidesList->getNumSides(); i++) {
			scripts[i] = TheSidesList->getSideInfo(i)->getScriptList();
		}
		ScriptList::WriteScriptsDataChunk(chunkWriter, scripts, TheSidesList->getNumSides());
	chunkWriter.closeDataChunk();

	Bool modified = TheSidesList->validateSides();
	DEBUG_ASSERTLOG(!modified, ("*** had to clean up sideslist on read"));
	modified = false;	// silence compiler warnings in release build

}

TeamsInfo *SidesList::findTeamInfo(AsciiString name, Int* index /*= nullptr*/)
{
	return m_teamrec.findTeamInfo(name, index);
}

SidesInfo *SidesList::findSideInfo(AsciiString name, Int* index /*= nullptr*/)
{
	for (int i = 0; i < m_numSides; i++)
	{
		if (m_sides[i].getDict()->getAsciiString(TheKey_playerName) == name)
		{
			if (index)
				*index = i;
			return &m_sides[i];
		}
	}
	return nullptr;
}

SidesInfo *SidesList::findSkirmishSideInfo(AsciiString name, Int* index /*= nullptr*/)
{
	for (int i = 0; i < m_numSkirmishSides; i++)
	{
		if (m_skirmishSides[i].getDict()->getAsciiString(TheKey_playerName) == name)
		{
			if (index)
				*index = i;
			return &m_skirmishSides[i];
		}
	}
	return nullptr;
}

static AsciiString static_readPlayerNames[SidesList::MAX_SKIRMISH_SIDES];

/**
* ParsePlayersDataChunk - read players names data chunk.
* Format is the newer CHUNKY format.
*	Input: DataChunkInput
*
*/
#define K_PLAYERS_NAMES_FOR_SCRIPTS_VERSION_1 1
#define K_PLAYERS_NAMES_FOR_SCRIPTS_VERSION_2 2 // Added in Zero Hour

static Bool ParsePlayersDataChunk(DataChunkInput &file, DataChunkInfo *info, void *userData)
{
	SidesList *sides = (SidesList *)userData;
	Int readDicts = 0;
	if (info->version >= K_PLAYERS_NAMES_FOR_SCRIPTS_VERSION_2) {
		readDicts = file.readInt();
	}
	Int numNames = file.readInt();
	Int i;
	for (i=0; i<numNames; i++) {
		AsciiString playerName = file.readAsciiString();
		if (i < SidesList::MAX_SKIRMISH_SIDES) {
			static_readPlayerNames[i] = playerName;
		}
		if (readDicts) {
			Dict sideDict = file.readDict();
			// GeneralsX @bugfix fbraz3/gpherai 05/10/2026 Import missing skirmish player definitions
			// from SkirmishScripts chunk to dynamically support mod factions on custom/retail maps (#335).
			if (sides && !playerName.isEmpty() && sides->findSkirmishSideInfo(playerName) == nullptr) {
				if (sideDict.getAsciiString(TheKey_playerName).isEmpty()) {
					sideDict.setAsciiString(TheKey_playerName, playerName);
				}
				sides->addSkirmishSide(&sideDict);
#ifdef ALLOW_DEBUG_UTILS
				fprintf(stderr, "[SKIRMISH_DIAG] Imported missing skirmish side '%s' from SkirmishScripts\n", playerName.str());
				fflush(stderr);
#endif
			}
		}
	}
	DEBUG_ASSERTCRASH(file.atEndOfChunk(), ("Unexpected data left over."));
	return true;
}

/**
* ParseTeamsDataChunk - read teams data chunk.
* Format is the newer CHUNKY format.
*	Input: DataChunkInput
*
*/
static Bool ParseTeamsDataChunk(DataChunkInput &file, DataChunkInfo *info, void *userData)
{
	SidesList *sides = (SidesList *)userData;
	while (!file.atEndOfChunk()) {
		Dict teamDict = file.readDict();
		AsciiString teamName = teamDict.getAsciiString(TheKey_teamName);
		AsciiString player = teamDict.getAsciiString(TheKey_teamOwner);
		if (sides->findSkirmishSideInfo(player)) {
			// player exists, so just add it.
			sides->addSkirmishTeam(&teamDict);
			//DEBUG_LOG(("Adding team %s", teamName.str()));
		} else {
			//DEBUG_LOG(("Couldn't add team %s, no player %s", teamName.str(), player.str()));
		}
	}
	DEBUG_ASSERTCRASH(file.atEndOfChunk(), ("Unexpected data left over."));
	return true;
}

void SidesList::prepareForMP_or_Skirmish()
{
	m_skirmishTeamrec.clear();
	Int i;
	for (i = 0; i < getNumTeams(); i++)
	{
		Dict *tdict = getTeamInfo(i)->getDict();
		m_skirmishTeamrec.addTeam(tdict);
	}
	m_teamrec.clear();

	for (i = 0; i < MAX_SKIRMISH_SIDES; i++) {
		m_skirmishSides[i].clear();
	}
	m_numSkirmishSides = 0;

	for (i = 0; i < m_numSides; i++)
	{
		m_skirmishSides[m_numSkirmishSides] = m_sides[i];
		m_numSkirmishSides++;
		if (m_sides[i].getDict()->getAsciiString(TheKey_playerFaction) == "FactionCivilian") {
			// Don't remove FactionCivilian.
			continue;
		}
		if (m_numSides == 1) break;	// can't remove the last side.
		removeSide(i);
		i--;
	}
	Bool gotScripts = false;
	for (i=0; i<m_numSkirmishSides; i++) {
		if (m_skirmishSides[i].getDict()->getAsciiString(TheKey_playerFaction) == "FactionCivilian") {
			// Don't consider FactionCivilian.
			continue;
		}
		if (m_skirmishSides[i].getScriptList()==nullptr) continue;
		if (m_skirmishSides[i].getScriptList()->getScript() != nullptr ||
			m_skirmishSides[i].getScriptList()->getScriptGroup()!=nullptr) {
			gotScripts = true;
		}
	}
	if (!gotScripts) {
		// GeneralsX @bugfix [upstream] 22/03/2026 Load default skirmish scripts relative to the configured asset root.
		AsciiString path = "Data\\Scripts\\SkirmishScripts.scb";
#ifdef ALLOW_DEBUG_UTILS
		// GeneralsX @bugfix [upstream] 22/03/2026 Emit runtime diagnostics for SkirmishScripts loading on Linux.
		fprintf(stderr, "[SKIRMISH_DIAG] SidesList::prepareForMP_or_Skirmish gotScripts=false path='%s'\n", path.str());
		fflush(stderr);
#endif
		DEBUG_LOG(("Skirmish map using standard scripts"));
		m_skirmishTeamrec.clear();
		CachedFileInputStream theInputStream;
		if (theInputStream.open(path)) {
#ifdef ALLOW_DEBUG_UTILS
				fprintf(stderr, "[SKIRMISH_DIAG] Opened SkirmishScripts.scb successfully\n");
				fflush(stderr);
#endif
				ChunkInputStream *pStrm = &theInputStream;
				DataChunkInput file( pStrm );
				file.registerParser( "PlayerScriptsList", AsciiString::TheEmptyString, ScriptList::ParseScriptsDataChunk );
				file.registerParser( "ScriptsPlayers", AsciiString::TheEmptyString, ParsePlayersDataChunk );
				file.registerParser( "ScriptTeams", AsciiString::TheEmptyString, ParseTeamsDataChunk );
				if (!file.parse(this)) {
#ifdef ALLOW_DEBUG_UTILS
					fprintf(stderr, "[SKIRMISH_DIAG] ERROR parsing SkirmishScripts.scb\n");
					fflush(stderr);
#endif
					DEBUG_LOG(("ERROR - Unable to read in skirmish scripts."));
					return;
				}
				ScriptList *scripts[MAX_PLAYER_COUNT];
				Int count = ScriptList::getReadScripts(scripts);
#ifdef ALLOW_DEBUG_UTILS
				fprintf(stderr, "[SKIRMISH_DIAG] Parsed SkirmishScripts.scb scriptCount=%d\n", count);
				fflush(stderr);
#endif
				Int i;
				for (i=0; i<count; i++) {
					Int curSide = -1;
					Int j;
					for (j=0; j<m_numSkirmishSides; j++) {
 						AsciiString name = getSkirmishSideInfo(j)->getDict()->getAsciiString(TheKey_playerName);
						if (name == static_readPlayerNames[i]) {
							curSide = j;
							break;
						}
					}
					if (curSide == -1)
					{
						deleteInstance(scripts[i]);
						scripts[i] = nullptr;
						continue;
					}

					deleteInstance(getSkirmishSideInfo(curSide)->getScriptList());
					getSkirmishSideInfo(curSide)->setScriptList(scripts[i]);
					scripts[i] = nullptr;
				}
				for (i=0; i<MAX_SKIRMISH_SIDES; i++) {
					static_readPlayerNames[i].clear();
				}
		} else {
#ifdef ALLOW_DEBUG_UTILS
			fprintf(stderr, "[SKIRMISH_DIAG] FAILED to open SkirmishScripts.scb path='%s'\n", path.str());
			fflush(stderr);
#endif
		}


	}
}


Bool SidesList::isPlayerDefaultTeam(TeamsInfo *t)
{
	// if our name is "teamfoo" and there is a player named "foo", we are a player-default team.
	AsciiString tname = t->getDict()->getAsciiString(TheKey_teamName);
	if (tname.startsWith("team"))
	{
		const char* rest = tname.str() + 4;
		for (int j = 0; j < m_numSides; j++)
		{
			AsciiString pname = m_sides[j].getDict()->getAsciiString(TheKey_playerName);
			if (strcmp(pname.str(), rest) == 0)
			{
				return true;
			}
		}
	}
	return false;
}

void SidesList::emptySides()
{
	Int i;

	m_numSides = 0;
	m_numSkirmishSides = 0;
	for (i = 0; i < MAX_PLAYER_COUNT; i++) {
		m_sides[i].clear();
	}
	for (i = 0; i < MAX_SKIRMISH_SIDES; i++) {
		m_skirmishSides[i].clear();
	}
}

void SidesList::emptyTeams()
{
	m_teamrec.clear();
	m_skirmishTeamrec.clear();
}

void SidesList::addSide(const Dict* d)
{
	DEBUG_ASSERTCRASH(m_numSides < MAX_PLAYER_COUNT, ("too many players"));
	if (m_numSides < MAX_PLAYER_COUNT)
		m_sides[m_numSides++].init(d);
}

// GeneralsX @bugfix fbraz3 05/10/2026 Add skirmish side definition imported from SkirmishScripts chunk (#335).
void SidesList::addSkirmishSide(const Dict* d)
{
	DEBUG_ASSERTCRASH(m_numSkirmishSides < MAX_SKIRMISH_SIDES, ("too many skirmish players"));
	if (m_numSkirmishSides < MAX_SKIRMISH_SIDES)
		m_skirmishSides[m_numSkirmishSides++].init(d);
}

void SidesList::addTeam(const Dict* d)
{
	m_teamrec.addTeam(d);
}

void SidesList::addSkirmishTeam(const Dict* d)
{
	m_skirmishTeamrec.addTeam(d);
}

void SidesList::removeSide(Int i)
{
	if (i < 0 || i >= m_numSides || m_numSides <= 1)
		return;

	for ( ; i < m_numSides-1; i++)
		m_sides[i] = m_sides[i+1];

	for ( ; i < MAX_PLAYER_COUNT; i++)
		m_sides[i].clear();

	--m_numSides;
}

void SidesList::removeTeam(Int i)
{
	m_teamrec.removeTeam(i);
}

Bool SidesList::validateAllyEnemyList(const AsciiString& tname, AsciiString& allies)
{
		// owners/allies/enemies must be players.

	Bool modified = false;

	AsciiString str, newstr, token;

	str = allies;
	newstr.clear();
	while (str.nextToken(&token))
	{
		if (token == tname)
		{
			modified = true;
			continue;	// no allies/enemies with self
		}

		SidesInfo *si = findSideInfo(token);
		if (!si)
		{
			modified = true;
			continue;	// player not found.
		}

		if (!newstr.isEmpty())
			newstr.concat(" ");
		newstr.concat(token);
	}

	allies = newstr;
	return modified;
}

void SidesList::addPlayerByTemplate(AsciiString playerTemplateName)
{
	AsciiString playerName;
	UnicodeString playerDisplayName;
	Bool isHuman = false;

	if (playerTemplateName.isEmpty())
	{
		playerName.set("");	// magic code for "neutral"
		playerDisplayName = L"Neutral";
		isHuman = false;
	}
	else
	{
		playerName.set("Plyr");
		if (playerTemplateName.startsWith("Faction"))
		{
			playerName.concat(playerTemplateName.str() + 7);
		}
		else
		{
			playerName.concat(playerTemplateName);
		}
		playerDisplayName.translate(playerName);
		isHuman = true;
		// special-case "civilian"...
		if (playerName == "PlyrCivilian")
			isHuman = false;
	}

	Dict d;

	d.clear();
	d.setAsciiString(TheKey_playerName, playerName);
	d.setBool(TheKey_playerIsHuman, isHuman);
	d.setUnicodeString(TheKey_playerDisplayName, playerDisplayName);
	d.setAsciiString(TheKey_playerFaction, playerTemplateName);
	d.setAsciiString(TheKey_playerAllies, AsciiString::TheEmptyString);
	d.setAsciiString(TheKey_playerEnemies, AsciiString::TheEmptyString);

	addSide(&d);

	AsciiString playerTeamName;
	playerTeamName.set("team");
	playerTeamName.concat(playerName);

	d.clear();
	d.setAsciiString(TheKey_teamName, playerTeamName);
	d.setAsciiString(TheKey_teamOwner, playerName);
	d.setBool(TheKey_teamIsSingleton, true);
	addTeam(&d);
}

Bool SidesList::validateSides()
{
	Bool modified = false;

	// ensure we have at least one player, and at least one neutral player.
	Int i;
	Int neutral = -1;
	Int num = getNumSides();
	for (i = 0; i < num; i++)
	{
		if (getSideInfo(i)->getDict()->getAsciiString(TheKey_playerName).isEmpty())
		{
			neutral = i;
			break;
		}
	}
	if (neutral == -1)
	{
		addPlayerByTemplate(AsciiString::TheEmptyString);
		modified = true;
	}

	// now ensure that every player has a proper "default team"
	for (i = 0; i < getNumSides(); i++)
	{
		Dict *pdict = getSideInfo(i)->getDict();
		AsciiString pname = pdict->getAsciiString(TheKey_playerName);
		AsciiString tname("team");
		tname.concat(pname);
		TeamsInfo *ti = findTeamInfo(tname);
		if (ti)
		{
			// make sure the team owner points back to the player.
			if (ti->getDict()->getAsciiString(TheKey_teamOwner) != pname)
			{
				DEBUG_CRASH(("hmm, team owner mismatch (%s) (%s), this should not normally be possible",ti->getDict()->getAsciiString(TheKey_teamOwner).str(), pname.str()));
				ti->getDict()->setAsciiString(TheKey_teamOwner, pname);
				modified = true;
			}
			// default teams are always singletons.
			if (!ti->getDict()->getBool(TheKey_teamIsSingleton))
			{
				DEBUG_CRASH(("hmm, this should not normally be possible"));
				ti->getDict()->setBool(TheKey_teamIsSingleton, true);
				modified = true;
			}
		}
		else
		{
			DEBUG_LOG(("*** default team for player %s missing (should not be possible), adding it...",tname.str()));
			Dict d;
			d.setAsciiString(TheKey_teamName, tname);
			d.setAsciiString(TheKey_teamOwner, pname);
			d.setBool(TheKey_teamIsSingleton, true);
			addTeam(&d);
			modified = true;
		}

		AsciiString allies = pdict->getAsciiString(TheKey_playerAllies);
		AsciiString enemies = pdict->getAsciiString(TheKey_playerEnemies);

		// ensure all teams have valid allies & enemies.
		// (note that owners can be teams or players, but allies/enemies can only be teams.)
		if (validateAllyEnemyList(pname, allies))
		{
			DEBUG_LOG(("bad allies..."));
			pdict->setAsciiString(TheKey_playerAllies, allies);
			modified = true;
		}

		if (validateAllyEnemyList(pname, enemies))
		{
			DEBUG_LOG(("bad enemies..."));
			pdict->setAsciiString(TheKey_playerEnemies, enemies);
			modified = true;
		}
	}

	// ensure there's no overlap between team names and player names.
	// (if there is, the player wins and the team is whacked.)
validate_team_names:
	for (i = 0; i < getNumTeams(); i++)
	{
		Dict *tdict = getTeamInfo(i)->getDict();
		AsciiString tname = tdict->getAsciiString(TheKey_teamName);
		if (findSideInfo(tname))
		{
			DEBUG_CRASH(("name %s is duplicate between player and team, removing...",tname.str()));
			removeTeam(i);
			modified = true;
			goto validate_team_names;
		}
	}

	for (i = 0; i < getNumTeams(); i++)
	{
		Dict *tdict = getTeamInfo(i)->getDict();
		AsciiString tname = tdict->getAsciiString(TheKey_teamName);
		AsciiString towner = tdict->getAsciiString(TheKey_teamOwner);
		SidesInfo* si = findSideInfo(towner);
		if (si == nullptr || towner == tname)
		{
			DEBUG_LOG(("bad owner %s; reparenting to neutral...",towner.str()));
			tdict->setAsciiString(TheKey_teamOwner, AsciiString::TheEmptyString);
			modified = true;
		}
//		if (tdict->getType(NAMEKEY("teamAllies")) != Dict::DICT_NONE)
//			tdict->remove(NAMEKEY("teamAllies"));
//		if (tdict->getType(NAMEKEY("teamEnemies")) != Dict::DICT_NONE)
//			tdict->remove(NAMEKEY("teamEnemies"));

	}

	return modified;
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void SidesList::crc( Xfer *xfer )
{

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void SidesList::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// xfer num sides
	Int sideCount = getNumSides();
	xfer->xferInt( &sideCount );
	if( sideCount != getNumSides() )
	{

		DEBUG_CRASH(( "SidesList::xfer - The sides list size has changed, this was not supposed to happen, you must version this method and figure out how to translate between old and new versions now" ));
		throw SC_INVALID_DATA;

	}

	// side data
	ScriptList *scriptList;
	Bool scriptListPresent;
	for( Int i = 0; i < sideCount; ++i )
	{

		// xfer script list data that can change
		scriptList = getSideInfo( i )->getScriptList();
		scriptListPresent = scriptList ? TRUE : FALSE;
		xfer->xferBool( &scriptListPresent );
		if( (scriptList == nullptr && scriptListPresent == TRUE) ||
				(scriptList != nullptr && scriptListPresent == FALSE) )
		{

			DEBUG_CRASH(( "SidesList::xfer - script list missing/present mismatch" ));
			throw SC_INVALID_DATA;

		}
		if( scriptListPresent )
			xfer->xferSnapshot( scriptList );

	}

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void SidesList::loadPostProcess()
{

}

/* ********* BuildListInfo class ****************************/
/**
 BuildListInfo - Constructor.
*/
BuildListInfo::BuildListInfo() :
m_nextBuildList(nullptr),
m_renderObj(nullptr),
m_shadowObj(nullptr),
m_isInitiallyBuilt(false),
m_numRebuilds(0),
m_angle(0),
m_script(AsciiString::TheEmptyString),
m_health(100),
m_whiner(true),
m_unsellable(false),
m_repairable(true),
m_objectID(INVALID_ID),
m_objectTimestamp(0),
m_underConstruction(false),
m_isSupplyBuilding(false),
m_desiredGatherers(0),
m_currentGatherers(0),
m_automaticallyBuild(true),
m_priorityBuild(false),
m_buildingName(AsciiString::TheEmptyString)
{
	m_location.zero();
	m_rallyPointOffset.x = 0.0f;
	m_rallyPointOffset.y = 0.0f;
	m_selected = FALSE;

	Int i;
	for (i=0; i<MAX_RESOURCE_GATHERERS; i++)
	{
		m_resourceGatherers[i] = INVALID_ID;
	}
}


/**
 BuildListInfo - Destructor - note - if linked, deletes linked items.
*/
BuildListInfo::~BuildListInfo()
{
	if (m_nextBuildList) {
		BuildListInfo *cur = m_nextBuildList;
		BuildListInfo *next;
		while (cur) {
			next = cur->getNext();
			cur->setNextBuildList(nullptr); // prevents recursion.
			deleteInstance(cur);
			cur = next;
		}
	}
}

void BuildListInfo::parseStructure(INI *ini, void *instance, void* /*store*/, const void* /*userData*/)
{
	const char* c = ini->getNextToken();
	AsciiString tTemplateName(c);

	static const FieldParse myFieldParse[] =
		{
			{ "Name",				INI::parseAsciiString,		nullptr, offsetof( BuildListInfo, m_buildingName	 ) },
			{ "Location",		INI::parseCoord2D,				nullptr, offsetof( BuildListInfo, m_location ) },
      { "Rebuilds",		INI::parseInt,						nullptr, offsetof( BuildListInfo, m_numRebuilds ) },
      { "Angle",			INI::parseAngleReal,			nullptr, offsetof( BuildListInfo, m_angle ) },
      { "InitiallyBuilt",			INI::parseBool,		nullptr, offsetof( BuildListInfo, m_isInitiallyBuilt ) },
      { "RallyPointOffset",			INI::parseCoord2D,		nullptr, offsetof( BuildListInfo, m_rallyPointOffset ) },
      { "AutomaticallyBuild",			INI::parseBool,	nullptr, offsetof( BuildListInfo, m_automaticallyBuild ) },
			{ nullptr,							nullptr,											nullptr, 0 }
		};

	BuildListInfo *buildInfo = newInstance( BuildListInfo );
	buildInfo->setTemplateName(tTemplateName);
	ini->initFromINI(buildInfo, myFieldParse);
	((AISideBuildList*)instance)->addInfo(buildInfo);
}


/**
 BuildListInfo - Duplicate - note - if linked, duplicates linked items.
*/
BuildListInfo *BuildListInfo::duplicate()
{
	BuildListInfo *first = newInstance( BuildListInfo );
	*first = *this;
	first->m_nextBuildList = nullptr;
	BuildListInfo *next = this->m_nextBuildList;
	BuildListInfo *cur = first;
	while (next) {
		BuildListInfo *link = newInstance( BuildListInfo );
		*link = *next;
		link->m_nextBuildList = nullptr;
		cur->m_nextBuildList = link;
		cur = link;
		next = next->m_nextBuildList;
	}
	return first;
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void BuildListInfo::crc( Xfer *xfer )
{

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void BuildListInfo::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 2;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	xfer->xferAsciiString( &m_buildingName );
	xfer->xferAsciiString( &m_templateName );
	xfer->xferCoord3D( &m_location );
	xfer->xferCoord2D( &m_rallyPointOffset );
	xfer->xferReal( &m_angle );
	xfer->xferBool( &m_isInitiallyBuilt );
	xfer->xferUnsignedInt( &m_numRebuilds );
	xfer->xferAsciiString( &m_script );
	xfer->xferInt( &m_health );
	xfer->xferBool( &m_whiner );
	xfer->xferBool( &m_unsellable );
	xfer->xferBool( &m_repairable );
	xfer->xferBool( &m_automaticallyBuild );
	// m_renderObj we don't need to xfer this, its for the editor only
	// m_shadowObj we don't need to xfer this, its for the editor only
	// m_selected we don't need to xfer this, its for the editor only
	xfer->xferObjectID( &m_objectID );
	xfer->xferUnsignedInt( &m_objectTimestamp );
	xfer->xferBool( &m_underConstruction );
	xfer->xferUser( m_resourceGatherers, sizeof( ObjectID ) * MAX_RESOURCE_GATHERERS );
	xfer->xferBool( &m_isSupplyBuilding );
	xfer->xferInt( &m_desiredGatherers );
	xfer->xferBool( &m_priorityBuild );
	if (version>=2) {
		xfer->xferInt(&m_currentGatherers);
	}

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void BuildListInfo::loadPostProcess()
{

}

/* ********* TeamsInfoRec class ****************************/
TeamsInfoRec::TeamsInfoRec() :
	m_numTeams(0), m_numTeamsAllocated(0), m_teams(nullptr)
{
}

TeamsInfoRec::TeamsInfoRec(const TeamsInfoRec& thatref) :
	m_numTeams(0), m_numTeamsAllocated(0), m_teams(nullptr)
{
	*this = thatref;
}

TeamsInfoRec::~TeamsInfoRec()
{
	clear();
}

// ug, I hate having to overload stuff, but this makes it a lot easier to make copies safely
TeamsInfoRec& TeamsInfoRec::operator=(const TeamsInfoRec& thatref)
{
	const TeamsInfoRec* that = &thatref;
	if (this != that)
	{
		this->clear();
		for (int i = 0; i < that->m_numTeams; i++)
		{
			this->addTeam(that->m_teams[i].getDict());
		}
	}
	return *this;
}

void TeamsInfoRec::clear()
{
	Int i;

	for (i = 0; i < m_numTeams; ++i)
		m_teams[i].clear();

	m_numTeams = 0;
	m_numTeamsAllocated = 0;
	delete [] m_teams;
	m_teams = nullptr;
}

TeamsInfo *TeamsInfoRec::findTeamInfo(AsciiString name, Int* index /*= nullptr*/)
{
	for (int i = 0; i < m_numTeams; ++i)
	{
		if (m_teams[i].getDict()->getAsciiString(TheKey_teamName) == name)
		{
			if (index)
				*index = i;
			return &m_teams[i];
		}
	}
	return nullptr;
}

void TeamsInfoRec::addTeam(const Dict* d)
{
	enum
	{
		TEAM_ALLOC_CHUNK = 8	///< how many teams to alloc at a time
	};

	DEBUG_ASSERTCRASH(m_numTeams < 2048, ("%d teams have been allocated (so far). This seems excessive.", m_numTeams ));
	if (m_numTeams >= m_numTeamsAllocated)
	{
		// pool[]ify
		const Int newNumTeamsAllocated = m_numTeams + TEAM_ALLOC_CHUNK;
		TeamsInfo* nti = NEW TeamsInfo[newNumTeamsAllocated];
		Int i;

		for (i = 0; i < m_numTeams; ++i)
			nti[i] = m_teams[i];

		delete [] m_teams;
		m_teams = nti;
		m_numTeamsAllocated = newNumTeamsAllocated;
	}

	m_teams[m_numTeams].init(d);

	++m_numTeams;
}

void TeamsInfoRec::removeTeam(Int i)
{
	if (i < 0 || i >= m_numTeams || m_numTeams <= 1)
		return;

	--m_numTeams;

	for ( ; i < m_numTeams; ++i)
		m_teams[i] = m_teams[i+1];

	m_teams[m_numTeams].clear();
}
```
