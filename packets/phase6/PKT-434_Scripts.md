# Translation Packet `PKT-434`: `Scripts`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-434`
- **Work Package:** `WP-6.4` (Phase 6)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer` *(self-contained `ScriptList`/`ScriptGroup`/`Script`/`OrCondition`/`Condition`/`ScriptAction`/`Parameter` AST, `DataChunk` reader/writer, and `Snapshot` serialization)*
- **Environment Access:** `env:ephemeral-web` *(seeded with 1,000-seed C++ Release oracle `docs/golden_vectors/PKT-434.json` & compiling Rust skeleton)*
- **FFI Seam Pattern:** Core Spine
- **Allowed Rust Output Files:**
  - `crates/zh_logic/src/scripts/scripts.rs` *(implement all `todo!("PKT-434")` methods)*
  - `crates/zh_logic/tests/golden_pkt434_scripts.rs` *(remove both `#[ignore]` attributes once implemented)*
  - `docs/PORT_MAP.md` *(regenerate via `python3 scripts/port_map.py --write`)*
- **Mandatory V&V Gates:** `V1, V2-Golden`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test -p zh_logic --test golden_pkt434_scripts && python3 scripts/port_map.py --write && python3 scripts/port_map.py --check && cargo test --workspace && cargo check --workspace --all-targets
  ```

## Port Contract (read first)

1. **Vec-Backed Hierarchy Replacing Intrusive Linked Lists:**
   - `ScriptList` stores `groups: Vec<ScriptGroup>` and `scripts: Vec<Script>`.
   - `ScriptGroup` stores `scripts: Vec<Script>`.
   - `Script` stores `or_conditions: Vec<OrCondition>`, `actions: Vec<ScriptAction>`, and `actions_false: Vec<ScriptAction>`.
   - `OrCondition` stores `conditions: Vec<Condition>`.
   - `Condition` and `ScriptAction` store `parms: Vec<Parameter>` (`<= MAX_PARMS = 12`).
   - `add_script(scr, ndx)` and `add_group(grp, ndx)`: mimic C++ `while (ndx && pCur) { ... ndx--; }` (`ndx <= 0` inserts at index `0`; `ndx as usize >= len` appends at tail; `0 < ndx < len` inserts at `ndx as usize`).

2. **`Parameter::write_parameter` & `Parameter::read_parameter` Legacy Migrations (`Scripts.cpp` lines 2103–2257):**
   - `write_parameter`: writes `param_type.as_i32()`. If `param_type == ParameterType::KindOfParam`, sets `self.string_val = KindOfMaskType::get_name_from_single_bit(&KINDOF_NAMES, self.int_val).to_string()`. If `param_type == ParameterType::Coord3D`, writes `coord.x`, `coord.y`, `coord.z` (`write_real`); else writes `int_val`, `real_val`, `string_val`.
   - `read_parameter`: reads `param_type = ParameterType::from_i32(input.read_int()?)`, sets `initialized = TRUE`. If `Coord3D`, reads 3 `f32`s (`coord.x, coord.y, coord.z`); else reads `int_val`, `real_val`, `string_val`.
   - Then runs the 4 legacy migrations on `read_parameter`:
     1. `ObjectType`: if `string_val.starts_with("Fundamentalist")`, replaces prefix `"Fundamentalist"` with `"GLA"`.
     2. `Upgrade`: if `string_val` is `"Upgrade_AmericaRangerCaptureBuilding"`, `"Upgrade_ChinaRedguardCaptureBuilding"`, or `"Upgrade_GLARebelCaptureBuilding"`, replaces with `"Upgrade_InfantryCaptureBuilding"`.
     3. `ObjectStatus`: iterates `0..OBJECT_STATUS_COUNT`; if `string_val.eq_ignore_ascii_case(OBJECT_STATUS_NAMES[i])`, sets bit `i` on `self.object_status` (`self.object_status.set(i as Int, TRUE)`) and breaks.
     4. `KindOfParam`: if `!string_val.is_empty()`, scans `KINDOF_NAMES` (`0..KINDOF_COUNT`):
        - If `string_val.eq_ignore_ascii_case(name)` -> `self.int_val = i as i32; break;`
        - Else if `string_val.eq_ignore_ascii_case("CRUSHER")` or `"CRUSHABLE"` or `"OVERLAPPABLE"` -> `self.int_val = i as i32; break;` (note C++ checks these inside the `i` loop on `i == 0`, so they set `int_val = 0`!).
        - Else if `string_val.eq_ignore_ascii_case("MISSILE")` -> sets `string_val = "SMALL_MISSILE"`, finds `"SMALL_MISSILE"` in `KINDOF_NAMES`, sets `int_val` to its index.
        - If `string_val.is_empty()`, sets `string_val = KindOfMaskType::get_name_from_single_bit(&KINDOF_NAMES, self.int_val).to_string()`.

3. **`Parameter::qualify` & `Parameter::get_ui_text` Exact Formatting (`Scripts.cpp` lines 1812–2095):**
   - `qualify(qualifier, player_template_name, new_player_name)`:
     - `ParameterType::Side`: let `tmp = format!("{}{}", self.string_val, qualifier)`; if `tmp == player_template_name`, sets `self.string_val = new_player_name.to_string()`.
     - `ParameterType::Team`: if `self.string_val == THIS_TEAM` (`"<This Team>"`), do nothing; else append `qualifier` (`self.string_val.push_str(qualifier)`).
     - `Script | Counter | Flag | ScriptSubroutine`: append `qualifier` (`self.string_val.push_str(qualifier)`).
   - `get_ui_text()`: let `ui_string = if self.string_val.is_empty() { "???" } else { &self.string_val }`. Match every `ParameterType` arm in `Scripts.cpp` lines 1846–2093 verbatim:
     - Note `Angle` formats `format!("{:.2} degrees", self.real_val * 180.0 / PI)` where `PI = 3.14159265359_f32`!
     - `Percent` formats `format!("{:.2}%", self.real_val * 100.0)`.
     - `Coord3D` formats `format!("({:.2},{:.2},{:.2})", pos.x, pos.y, pos.z)`.
     - `Color` formats `format!(" R:{} G:{} B:{} ", (self.int_val & 0x00ff0000) >> 16, (self.int_val & 0x0000ff00) >> 8, self.int_val & 0x000000ff)`.
     - `Boundary` uses `BORDER_COLORS[(self.int_val.rem_euclid(BORDER_COLORS_SIZE as i32)) as usize].color_name` (for non-negative `int_val`, `(self.int_val as usize) % BORDER_COLORS_SIZE`).
     - `ObjectStatus` checks `self.string_val.is_empty()` -> `"Object Status is ???"`, else `format!("Object Status is '{}'", self.string_val)`.

4. **`Condition` & `ScriptAction` DataChunk Write/Parse & Legacy File Healing (`Scripts.cpp` lines 1647–1758, 2403–2687):**
   - `Condition` writes `"Condition"` chunk v`4` (`K_SCRIPT_CONDITION_VERSION_4`): `condition_type`, `internal_name_key` via `chunk_writer.write_name_key(ct.internal_name_key, nkg)` (or `"Bogus"` if missing), `parms.len() as i32`, and each parameter.
   - `Condition` parse: reads `condition_type`; if `version >= 4`, reads `NameKeyType` (`read_name_key(nkg)`), rematches across `0..CONDITION_NUM_ITEMS` if `ct.internal_name_key != key`, or falls back to `CONDITION_FALSE (0)` with 0 parms; reads `num_parms` and `parms`; if `version < 2` and `condition_type` is one of the 7 `ParameterChangesVer2` types (`7, 17, 18, 39, 40, 41, 42`), appends `Parameter::new(ParameterType::SurfacesAllowed, 3)` and sets `num_parms = 3`; if `condition_type == 84` (`SKIRMISH_SPECIAL_POWER_READY`) and `parms.len() == 1`, shifts `parms[0]` to `parms[1]` and inserts `Parameter::new(ParameterType::Side, 0)` with `string_val = THIS_PLAYER.to_string()` at `parms[0]`; finally if `ct.num_parameters != parms.len() as i32`, resets `condition_type = 0` (`CONDITION_FALSE`) and clears `parms`.
   - `ScriptAction` writes `"ScriptAction"` (or `"ScriptActionFalse"`) chunk v`2` (`K_SCRIPT_ACTION_VERSION_2`): `action_type`, `internal_name_key`, `parms.len() as i32`, and each parameter.
   - `ScriptAction` parse: reads `action_type`; if `version >= 2`, reads `NameKeyType` and rematches across `0..ACTION_NUM_ITEMS` or sets `NO_OP (5)`; reads `num_parms` and `parms`; runs the 10 legacy healing cases in `Scripts.cpp` lines 2485–2590; if `at.num_parameters != parms.len() as i32`, resets `action_type = 5` (`NO_OP`) and clears `parms`.

5. **`ScriptList`, `ScriptGroup`, & `Script` `Snapshot::xfer` (`RETAIL_COMPATIBLE_XFER_SAVE = 1`):**
   - `ScriptList::xfer`: version `1`, `script_count: u16` (`self.scripts.len() as u16`), `xfer.xfer_snapshot(script)` for each top-level script, `group_count: u16` (`self.groups.len() as u16`), `xfer.xfer_snapshot(group)` for each group.
   - `ScriptGroup::xfer`: version `2`, `xfer.xfer_bool(&mut self.is_group_active)` (when `version >= 2`), `script_count: u16`, `xfer.xfer_snapshot(script)` for each group script.
   - `Script::xfer`: under default retail-compatible save format (`current_version = 1`), `version = xfer.xfer_version(1, 1)?`, `xfer.xfer_bool(&mut self.is_active)?`, and if `version >= 2`, `xfer.xfer_u32(&mut self.frame_to_evaluate_at)?`.

6. **Determinism & 3-Attempt Escalation Rule:** Zero external crates, no `unsafe`, no `HashMap`/`HashSet`/`BinaryHeap`, no `Rc`/`RefCell`, no std float math. If 3 consecutive fix attempts fail to shrink failing vectors, isolate the minimal failing seed or escalate (`tier:3-architect`).

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (19):**
  - `Common/BorderColors.h`
  - `Common/DataChunk.h`
  - `Common/GameState.h`
  - `Common/KindOf.h`
  - `Common/ObjectStatusTypes.h`
  - `Common/Player.h`
  - `Common/Radar.h`
  - `Common/Snapshot.h`
  - `Common/ThingTemplate.h`
  - `Common/Xfer.h`
  - `GameClient/ShellHooks.h`
  - `GameClient/View.h`
  - `GameLogic/AI.h`
  - `GameLogic/Module/ContainModule.h`
  - `GameLogic/Object.h`
  - `GameLogic/ScriptEngine.h`
  - `GameLogic/SidesList.h`
  - `GameNetwork/NetworkDefs.h`
  - `PreRTS.h`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (15 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUnsignedShort( &scriptCount )`
  3. `xferSnapshot( script )`
  4. `xferSnapshot(s_mtScript)`
  5. `xferUnsignedShort( &scriptGroupCount )`
  6. `xferSnapshot( scriptGroup )`
  7. `xferSnapshot(s_mtGroup)`
  8. `xferVersion( &version, currentVersion )`
  9. `xferBool(&m_isGroupActive)`
  10. `xferUnsignedShort( &scriptCount )`
  11. `xferSnapshot( script )`
  12. `xferSnapshot(s_mtScript)`
  13. `xferVersion( &version, currentVersion )`
  14. `xferBool( &active )`
  15. `xferUnsignedInt( &m_frameToEvaluateAt )`
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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Scripts.h`
```cpp
// Scripts.h
// Script data structures.
// Author: John Ahlquist, November 2001

#pragma once

#include "Common/Snapshot.h"
#include "GameNetwork/NetworkDefs.h"
#include "Common/ObjectStatusTypes.h"

#define THIS_TEAM "<This Team>"
#define ANY_TEAM "<Any Team>"

#define THIS_OBJECT "<This Object>"
#define ANY_OBJECT "<Any Object>"

#define THIS_PLAYER "<This Player>"
#define LOCAL_PLAYER "<Local Player>"

#define THE_PLAYER "ThePlayer"
#define TEAM_THE_PLAYER "teamThePlayer"

#define THIS_PLAYER_ENEMY "<This Player's Enemy>"

#define WATER_GRID "Water Grid"

// Skirmish waypoint names
#define SKIRMISH_CENTER "Center"
#define SKIRMISH_FLANK "Flank"
#define SKIRMISH_BACKDOOR "Backdoor"
#define SKIRMISH_SPECIAL "Special"

// Skirmish Player Areas
#define SKIRMISH_AREA_HOME_BASE				"Home Base"
#define SKIRMISH_AREA_ANY_SUPPLYDEPOT	"Any Supply Depot"

// Skirmish trigger names.
#define MY_INNER_PERIMETER		"[Skirmish]MyInnerPerimeter"
#define MY_OUTER_PERIMETER		"[Skirmish]MyOuterPerimeter"
#define ENEMY_OUTER_PERIMETER "[Skirmish]EnemyOuterPerimeter"
#define ENEMY_INNER_PERIMETER "[Skirmish]EnemyInnerPerimeter"

#define INNER_PERIMETER "InnerPerimeter"
#define OUTER_PERIMETER "OuterPerimeter"

class Parameter;
class Script;
class OrCondition;
class Condition;
class DataChunkInput;
struct DataChunkInfo;
class DataChunkOutput;

#define NO_MORE_COMPLEX_SKIRMISH_SCRIPTS
#ifndef NO_MORE_COMPLEX_SKIRMISH_SCRIPTS
// For now, we only actually allow the simple stuff to be chosen. However, as soon as I can get more
// time on the schedule, we need to extend these scripts to be more useful. To that end, I've
// created the parameters and such necessary for more complex behavior, but for now only the
// rudimentary support is enabled. jkmcd
	#define MORE_COMPLEX_SKIRMISH_SCRIPTS
#endif


// There is undoubtedly a more sophisticated way to do this, but for some reason my brain hurts.
// The corresponding UnsignedInt values that we pass around are:
// 1 means ground
// 2 means air
// 3 means air or ground
extern const char *Surfaces[];

// Shake intensities. Must be kept in sync with View::CameraShakeType
extern const char *ShakeIntensities[];

//-------------------------------------------------------------------------------------------------
// ******************************** class ScriptGroup ***********************************************
//-------------------------------------------------------------------------------------------------
class ScriptGroup : public MemoryPoolObject, public Snapshot
// This is a list of scripts that are grouped and named, with some
// additional properties.
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(ScriptGroup, "ScriptGroup")
protected:

	// snapshot methods
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

protected:
	Script			*m_firstScript;
	AsciiString m_groupName;
	Bool				m_isGroupActive;
	Bool				m_isGroupSubroutine;
	ScriptGroup	*m_nextGroup;
	Bool				m_hasWarnings; ///< Runtime flag used by the editor only.

public:
	ScriptGroup();
	//~ScriptGroup();

public:

	ScriptGroup *duplicate() const;		// note, duplicates just this node, not the full list.
	ScriptGroup *duplicateAndQualify(const AsciiString& qualifier,
			const AsciiString& playerTemplateName, const AsciiString& newPlayerName) const;		// note, duplicates just this node, not the full list.

	void setName(AsciiString name) { m_groupName = name;}
	void setActive(Bool active) { m_isGroupActive = active;}
	void setSubroutine(Bool subr) { m_isGroupSubroutine = subr;}
	void setWarnings(Bool warnings) { m_hasWarnings = warnings;}
	void setNextGroup(ScriptGroup *pGr) {m_nextGroup = pGr;}

	const AsciiString& getName() const { return m_groupName;}
	Bool isActive() const { return m_isGroupActive;}
	Bool isSubroutine() const { return m_isGroupSubroutine;}
	Bool hasWarnings() const { return m_hasWarnings;}
	ScriptGroup *getNext() const {return m_nextGroup;};
	Script *getScript() {return m_firstScript;};

	void addScript(Script *pScr, Int ndx);
	void deleteScript(Script *pScr);

	static void WriteGroupDataChunk(DataChunkOutput &chunkWriter, ScriptGroup *pGroup);
	static Bool ParseGroupDataChunk(DataChunkInput &file, DataChunkInfo *info, void *userData);
};

//-------------------------------------------------------------------------------------------------
// ******************************** class OrCondition ***********************************************
//-------------------------------------------------------------------------------------------------
/// This is a linked list of or clauses, each containing a list of and clauses.
class OrCondition : public MemoryPoolObject
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(OrCondition, "OrCondition")
protected:
	OrCondition *m_nextOr;   // Next or clause.
	Condition *m_firstAnd;	 // These are Anded.

public:
	OrCondition():m_nextOr(nullptr),m_firstAnd(nullptr){};
	//~OrCondition();
	/// Duplicate creates a "deep" copy.  If it is head of a linked list, duplicates the entire list.
	OrCondition *duplicate() const;
	OrCondition *duplicateAndQualify(const AsciiString& qualifier,
			const AsciiString& playerTemplateName, const AsciiString& newPlayerName) const;

public:
	void setNextOrCondition(OrCondition *pOr) {m_nextOr = pOr;}
	void setFirstAndCondition(Condition *pAnd) {m_firstAnd = pAnd;}

	OrCondition *getNextOrCondition() {return m_nextOr;}
	Condition *getFirstAndCondition() {return m_firstAnd;}

	Condition *removeCondition(Condition *pCond);
	void deleteCondition(Condition *pCond);
	static void WriteOrConditionDataChunk(DataChunkOutput &chunkWriter, OrCondition *pCondition);
	static Bool ParseOrConditionDataChunk(DataChunkInput &file, DataChunkInfo *info, void *userData);

	// Utility for moving scripts upwards
	Condition *findPreviousCondition( Condition *curCond );
};

//-------------------------------------------------------------------------------------------------
// ******************************** class ScriptAction ***********************************************
//-------------------------------------------------------------------------------------------------
/** This is an action that can be executed by a script.  The Action Types
are defined by ActionTemplates created in the ScriptEngine::init method. */
enum {MAX_PARMS=12};

class ScriptAction : public MemoryPoolObject // This is the action class.
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(ScriptAction, "ScriptAction")
// friend bad for MPOs. (srj)
//friend class EditAction;
public:
	enum ScriptActionType
	{
		DEBUG_MESSAGE_BOX,							///< Show a message box.
		SET_FLAG,													///< Set a flag true of false.
		SET_COUNTER,											///< Set a counter to an integer.
		VICTORY,													///< Announce victory.
		DEFEAT,														///< Announce defeat.
		NO_OP, 														///< No op.
		SET_TIMER,												///< Start a frame timer.
		PLAY_SOUND_EFFECT,								///< Play sound.
		ENABLE_SCRIPT,										///< Enable script.
		DISABLE_SCRIPT,										///< Disable script.
		CALL_SUBROUTINE,									///< Call a subroutine.
		PLAY_SOUND_EFFECT_AT,							///< Play sound at location.
		DAMAGE_MEMBERS_OF_TEAM,						///< Damage team members (boom.)
		MOVE_TEAM_TO,											///< Move to.
		MOVE_CAMERA_TO,										///< Move camera to.
		INCREMENT_COUNTER,								///< Add to a counter.
		DECREMENT_COUNTER,								///< subtract from a counter.
		MOVE_CAMERA_ALONG_WAYPOINT_PATH,	///< Move camera along path.
		ROTATE_CAMERA,										///< Rotate camera.
		RESET_CAMERA,											///< Reset camera.
		SET_MILLISECOND_TIMER,						///< Start a time based timer.
		CAMERA_MOD_FREEZE_TIME,						///< Freeze time during a camera movement.
		SET_VISUAL_SPEED_MULTIPLIER,			///< Drop frames so that time "goes" faster..
		CREATE_OBJECT,										///< Create an object
		SUSPEND_BACKGROUND_SOUNDS,				///< Stop backgropund music & sounds.
		RESUME_BACKGROUND_SOUNDS,					///< Resume backgropund music & sounds.
		CAMERA_MOD_SET_FINAL_ZOOM,				///< Sets the final zoom for a camera movement
		CAMERA_MOD_SET_FINAL_PITCH,				///< Sets the final pitch for a camera movement
		CAMERA_MOD_FREEZE_ANGLE,					///< Freeze camera angle during a camera movement.
		CAMERA_MOD_SET_FINAL_SPEED_MULTIPLIER,///< Sets the final time multiplier for a camera movement.
		CAMERA_MOD_SET_ROLLING_AVERAGE,		///< Sets the number of frames to average changes (angle, position) to smooth out a camera movement.
		CAMERA_MOD_FINAL_LOOK_TOWARD,			///< Sets the look toward point for the end of a camera movement.
		CAMERA_MOD_LOOK_TOWARD,						///< Sets the look toward point during a camera movement.
		TEAM_ATTACK_TEAM,									///< Tell team to attack other team
		CREATE_REINFORCEMENT_TEAM,				///< Create a team of reinforcements.
		MOVE_CAMERA_TO_SELECTION,					///< Move the target to the selected item.
		TEAM_FOLLOW_WAYPOINTS,						///< Tell the team to begin following a path, starting at given waypoint
		TEAM_SET_STATE,										///< Set a team to a particular state.
		MOVE_NAMED_UNIT_TO,								///< Move a named unit to a position
		NAMED_ATTACK_NAMED,								///< Make a named unit attack another named unit
		CREATE_NAMED_ON_TEAM_AT_WAYPOINT,	///< Create a named unit on a team at a waypoint
		CREATE_UNNAMED_ON_TEAM_AT_WAYPOINT,	///< Create a unnamed unit on a team at a waypoint
		NAMED_APPLY_ATTACK_PRIORITY_SET,	///< Applies an attack priority set to a named unit
		TEAM_APPLY_ATTACK_PRIORITY_SET,		///< Applies an attack priority set to a team
		SET_BASE_CONSTRUCTION_SPEED,			///< Impl JKMCD
		NAMED_SET_ATTITUDE,								///< Impl JKMCD
		TEAM_SET_ATTITUDE,								///< Impl JKMCD
		NAMED_ATTACK_AREA,								///< Set a unit to attack an area
		NAMED_ATTACK_TEAM,								///< Impl JKMCD
		TEAM_ATTACK_AREA,									///< Set a team to attack an area
		TEAM_ATTACK_NAMED,								///< Set a team to attack a unit
		TEAM_LOAD_TRANSPORTS,							///< Set a team to auto-load transports
		NAMED_ENTER_NAMED,								///< Set a unit to enter a transport
		TEAM_ENTER_NAMED,									///< Set a team to enter a transport
		NAMED_EXIT_ALL,										///< Set a unit to exit its transport
		TEAM_EXIT_ALL,										///< Set a team to exit all transports
		NAMED_FOLLOW_WAYPOINTS,						///< Set a unit to follow a waypoint path
		NAMED_GUARD,											///< Set a unit to guard it's current location.
		TEAM_GUARD,												///< Set a team to guard it's current location.
		NAMED_HUNT,												///< Set a unit to hunt
		TEAM_HUNT,												///< Set a team to hunt
		PLAYER_SELL_EVERYTHING,						///< Set a player to sell everything
		PLAYER_DISABLE_BASE_CONSTRUCTION,	///< Disallow a player from building anymore base buildings
		PLAYER_DISABLE_FACTORIES,					///< Disallow a player from building from factories
		PLAYER_DISABLE_UNIT_CONSTRUCTION,	///< Disallow a player from building units
		PLAYER_ENABLE_BASE_CONSTRUCTION,	///< Allow a player to build bases
		PLAYER_ENABLE_FACTORIES,					///< Allow a player to build from his factories
		PLAYER_ENABLE_UNIT_CONSTRUCTION,	///< Allow a player to build units
		CAMERA_MOVE_HOME,									///< Impl JKMCD
		BUILD_TEAM,												///< Build a team from a teamtemplate
		NAMED_DAMAGE,											///< Damage a unit
		NAMED_DELETE,											///< Delete a unit
		TEAM_DELETE,											///< Delete a team
		NAMED_KILL,												///< Kill a unit
		TEAM_KILL,												///< Kill a team
		PLAYER_KILL,											///< Kill a player
		DISPLAY_TEXT,											///< Display a localized string
		CAMEO_FLASH,											///< Flashes a command button
		NAMED_FLASH,											///< Flashes a specific unit
		TEAM_FLASH,												///< Flashes a specific team
		MOVIE_PLAY_FULLSCREEN,						///< Play a movie in fullscreen
		MOVIE_PLAY_RADAR,									///< Play a movie in the radar
		SOUND_PLAY_NAMED,									///< Play a sound as though it comes from a unit
		SPEECH_PLAY,											///< Play a speech
		PLAYER_TRANSFER_OWNERSHIP_PLAYER,	///< Transfer ownership of all units to another player
		NAMED_TRANSFER_OWNERSHIP_PLAYER,	///< Transfer ownership of a single unit to another player
		PLAYER_RELATES_PLAYER,						///< Update player-player relations
		RADAR_CREATE_EVENT,								///< Impl JKMCD
		RADAR_DISABLE,										///< Disable the radar
		RADAR_ENABLE,											///< Enable the radar
		MAP_REVEAL_AT_WAYPOINT,						///< Reveal the map within a radius of waypoint.
		TEAM_AVAILABLE_FOR_RECRUITMENT,		///< Impl JKMCD
		TEAM_COLLECT_NEARBY_FOR_TEAM,			///< Impl JKMCD
		TEAM_MERGE_INTO_TEAM,							///< Merge a team into another team
		DISABLE_INPUT,										///< Disable user input
		ENABLE_INPUT,											///< Re-enable user input
		PLAYER_HUNT,											///< Set an entire player's team to hunt
		SOUND_AMBIENT_PAUSE,							///< Pause all ambient sounds
		SOUND_AMBIENT_RESUME,							///< Resume all ambient sounds
		MUSIC_SET_TRACK,									///< Set the current music track, allowing for fade-in and fade-out
		SET_TREE_SWAY,										///< Set the trees' sway
		DEBUG_STRING,											///< Output a debug string without stopping.
		MAP_REVEAL_ALL,										///< Reveal the entire map
		TEAM_GARRISON_SPECIFIC_BUILDING,	///< Garrison a specific building
		EXIT_SPECIFIC_BUILDING,						///< Empty a specific building
		TEAM_GARRISON_NEAREST_BUILDING,		///< Garrison a nearby building
		TEAM_EXIT_ALL_BUILDINGS,					///< Empty all buildings of units
		NAMED_GARRISON_SPECIFIC_BUILDING,	///< Send a unit to garrison a specific building
		NAMED_GARRISON_NEAREST_BUILDING,		///< Send a unit to garrison the nearest building
		NAMED_EXIT_BUILDING,								///< Tell a unit to exit the building its currently in
		PLAYER_GARRISON_ALL_BUILDINGS,		///< Garrison as many buildings as possible (filling each one up)
		PLAYER_EXIT_ALL_BUILDINGS,				///< Tell all buildings to empty themselves for this player.
		TEAM_WANDER,											///< Tell a team to wander.
		TEAM_PANIC,												///< Tell a team to panic.
		SETUP_CAMERA,											///< Set up an initial camera position.
		CAMERA_LETTERBOX_BEGIN,						///< Set the camera into letterbox mode
		CAMERA_LETTERBOX_END,							///< End the Letterbox mode.
		ZOOM_CAMERA,											///< Set camera zoom.
		PITCH_CAMERA,											///< Set camera pitch.
		CAMERA_FOLLOW_NAMED,							///< Camera follow named.
		OVERSIZE_TERRAIN,									// Oversize the drawn terrain for effect.
		CAMERA_FADE_ADD,									///< Fade using an add blend
		CAMERA_FADE_SUBTRACT,							///< Fade using a subtract blend
		CAMERA_FADE_SATURATE,							///< Fade using a saturate blend
		CAMERA_FADE_MULTIPLY,							///< Fade using a multiply blend
		CAMERA_BW_MODE_BEGIN,							///< switch camera to black & white mode.
		CAMERA_BW_MODE_END,								///< switch camera back to color mode.
		DRAW_SKYBOX_BEGIN,								///< draw skybox
		DRAW_SKYBOX_END,									///< stop skybox
		SET_ATTACK_PRIORITY_THING,				///< Sets the attack priority for a single thing in a set.
		SET_ATTACK_PRIORITY_KIND_OF,			///< Sets the attack priority for all of a kind	in a set.
		SET_DEFAULT_ATTACK_PRIORITY,			///< Sets the default attack priority for a set.
		CAMERA_STOP_FOLLOW,								///< Ends a camera follow named.
		CAMERA_MOTION_BLUR,								///< Starts a camera motion blur zoom
		CAMERA_MOTION_BLUR_JUMP,					///< Starts a camera motion blur in & out with jump.
		CAMERA_MOTION_BLUR_FOLLOW,				///< Starts a motion blur follow scrolling effect.
		CAMERA_MOTION_BLUR_END_FOLLOW,		///< Ends a motion blur follow scrolling effect.
		FREEZE_TIME,											///< Freeze time.
		UNFREEZE_TIME,										///< Unfreeze time.
		SHOW_MILITARY_CAPTION,						///< Display military-style briefing text
		CAMERA_SET_AUDIBLE_DISTANCE,			///< Sets the range for the audible distance
		SET_STOPPING_DISTANCE,						///< Sets the stopping distance for a team
		NAMED_SET_STOPPING_DISTANCE,			///< Sets the stopping distance for a named object
		SET_FPS_LIMIT,										///< Sets the max frames-per-second
		MUSIC_SET_VOLUME,									///< Adjust the track volume
		MAP_SHROUD_AT_WAYPOINT,						///< Shroud the map at this waypoint
		MAP_SHROUD_ALL,										///< Shroud the entire map
		SET_RANDOM_TIMER,									///< Start a random frame based timer.
		SET_RANDOM_MSEC_TIMER,						///< Start a random time based timer.
		STOP_TIMER,												///< Stop a timer. Remembers time stopped.
		RESTART_TIMER,										///< Restarts a stopped timer. Remembers time stopped.
		ADD_TO_MSEC_TIMER,								///< Adds time to a timer.
		SUB_FROM_MSEC_TIMER,							///< Subtracts time from a timer.
		TEAM_TRANSFER_TO_PLAYER,					///< Transfers a team to a player, maintaining team-ness
		PLAYER_SET_MONEY,									///< Set a player's cash reserves to a specific value.
		PLAYER_GIVE_MONEY,								///< // Add/subtract cash from a player's reserves.
		DISABLE_SPECIAL_POWER_DISPLAY,		///< hides the special power countdown display
		ENABLE_SPECIAL_POWER_DISPLAY,			///< shows the special power countdown display
		NAMED_HIDE_SPECIAL_POWER_DISPLAY,	///< hides the special power countdown display for an object
		NAMED_SHOW_SPECIAL_POWER_DISPLAY,	///< shows the special power countdown display for an object
		DISPLAY_COUNTDOWN_TIMER,					///< shows a countdown timer
		HIDE_COUNTDOWN_TIMER,							///< hides a countdown timer
		ENABLE_COUNTDOWN_TIMER_DISPLAY,		///< enables countdown timer display
		DISABLE_COUNTDOWN_TIMER_DISPLAY,	///< disables countdown timer display
		NAMED_STOP_SPECIAL_POWER_COUNTDOWN,	///< pauses the countdown for a special power
		NAMED_START_SPECIAL_POWER_COUNTDOWN,///< resumes the countdown for a special power
		NAMED_SET_SPECIAL_POWER_COUNTDOWN,	///< set special power countdown for an object
		NAMED_ADD_SPECIAL_POWER_COUNTDOWN,	///< add/subtract time from a special power countdown
		NAMED_FIRE_SPECIAL_POWER_AT_WAYPOINT,	///< fire a special power
		NAMED_FIRE_SPECIAL_POWER_AT_NAMED,	///< fire a special power
		REFRESH_RADAR,										///< Force the radar to recalculate the terrain
		CAMERA_TETHER_NAMED,							///< Tether the camera location to a named unit
		CAMERA_STOP_TETHER_NAMED,					///< Break tether
		CAMERA_SET_DEFAULT,								///< Set default angle, pitch, zoom, etc for camera
		NAMED_STOP,												///< Make a named unit stop (go idle)
		TEAM_STOP,												///< Make a team stop (go idle)
		TEAM_STOP_AND_DISBAND,						///< Make a team stop and be merged into the default player team. Also, mark the whole team recruitable
		RECRUIT_TEAM,	  									///< Recruit a team?
		TEAM_SET_OVERRIDE_RELATION_TO_TEAM,					///< override a team's relation to another team
		TEAM_REMOVE_OVERRIDE_RELATION_TO_TEAM,			///< remove that override
		TEAM_REMOVE_ALL_OVERRIDE_RELATIONS,///< remove all overrides
		CAMERA_LOOK_TOWARD_OBJECT,				///< Rotate the camera to be pointing toward a unit
		NAMED_FIRE_WEAPON_FOLLOWING_WAYPOINT_PATH,    ///< Fires a specific weapon that follows waypoint path and detonates at the final waypoint.
		TEAM_SET_OVERRIDE_RELATION_TO_PLAYER,					///< override a team's relation to another player
		TEAM_REMOVE_OVERRIDE_RELATION_TO_PLAYER,			///< remove that override
		PLAYER_SET_OVERRIDE_RELATION_TO_TEAM,					///< override a Player's relation to another team
		PLAYER_REMOVE_OVERRIDE_RELATION_TO_TEAM,			///< remove that override
		UNIT_EXECUTE_SEQUENTIAL_SCRIPT,		///< Make a unit execute a script sequentially
		UNIT_EXECUTE_SEQUENTIAL_SCRIPT_LOOPING,		///< Make a unit execute a script sequentially (forever)
		UNIT_STOP_SEQUENTIAL_SCRIPT,			///< Make a unit stop executing its sequential scripts
		TEAM_EXECUTE_SEQUENTIAL_SCRIPT,		///< Make a team execute a script sequentially
		TEAM_EXECUTE_SEQUENTIAL_SCRIPT_LOOPING,		///< Make a team execute a script sequentially (forever)
		TEAM_STOP_SEQUENTIAL_SCRIPT,			///< Make a team stop executing its sequential scripts
		UNIT_GUARD_FOR_FRAMECOUNT,				///< Make a specific unit guard for a framecount
		UNIT_IDLE_FOR_FRAMECOUNT,					///< Make a specific unit stop idle for a framecount
		TEAM_GUARD_FOR_FRAMECOUNT,				///< Make a team guard for a framecount
		TEAM_IDLE_FOR_FRAMECOUNT,					///< Make a team stop idle for a framecount
		WATER_CHANGE_HEIGHT,							///< Adjust the water level
		NAMED_USE_COMMANDBUTTON_ABILITY_ON_NAMED,				///< Make a unit use an ability on another unit.
		NAMED_USE_COMMANDBUTTON_ABILITY_AT_WAYPOINT,		///< Make a unit use an ability on a waypoint.
		WATER_CHANGE_HEIGHT_OVER_TIME,						///< Adjust the water level over a period of time.
		MAP_SWITCH_BORDER,												///< Switch to border color <Blah>
		TEAM_GUARD_POSITION,
		TEAM_GUARD_OBJECT,
		TEAM_GUARD_AREA,
		OBJECT_FORCE_SELECT,											///< Force selection of the first object type on the player's team
		CAMERA_LOOK_TOWARD_WAYPOINT,							///< Rotate the camera to be pointing toward a waypoint
		UNIT_DESTROY_ALL_CONTAINED,								///< Destroy all units contained by the named unit
		RADAR_FORCE_ENABLE,												///< Force the radar to be on regardless of player status
		RADAR_REVERT_TO_NORMAL,										///< Undo forcing the radar to be on
		SCREEN_SHAKE,															///< Shake the entire screen
		TECHTREE_MODIFY_BUILDABILITY_OBJECT,			///< Modify how "buildable" something is.
		WAREHOUSE_SET_VALUE,											///< Set a Warehouse to have this much money.
		OBJECT_CREATE_RADAR_EVENT,								///< Create a radar event at this object
		TEAM_CREATE_RADAR_EVENT,									///< Create a radar event at this team
		DISPLAY_CINEMATIC_TEXT,									  ///< Displays a string in the bottom letterbox
		DEBUG_CRASH_BOX,													///< Debug/Internal builds only: do a crash box
		SOUND_DISABLE_TYPE,
		SOUND_ENABLE_TYPE,
		SOUND_ENABLE_ALL,
		AUDIO_OVERRIDE_VOLUME_TYPE,
		AUDIO_RESTORE_VOLUME_TYPE,
		AUDIO_RESTORE_VOLUME_ALL_TYPE,
		INGAME_POPUP_MESSAGE,											///< Popup an ingame popup message box
		SET_CAVE_INDEX,														///< Set the index of which set of caves a cave is connected to.
		NAMED_SET_HELD,														///< Sets a named object to be "held" in place or not
		NAMED_SET_TOPPLE_DIRECTION,								///< Set the topple direction of the specified direction
		UNIT_MOVE_TOWARDS_NEAREST_OBJECT_TYPE,		///< Move a named unit towards the nearest object of type type
		TEAM_MOVE_TOWARDS_NEAREST_OBJECT_TYPE,		///< For each member of the team, move towards the nearest object of type
		MAP_REVEAL_ALL_PERM,
		MAP_REVEAL_ALL_UNDO_PERM,
		NAMED_SET_REPULSOR,												///< Set a unit as a repulsor (scares) civilians.
		TEAM_SET_REPULSOR,												///< Set a team as a repulsor (scares) civilians.
		TEAM_WANDER_IN_PLACE,											///< Set a team to wander around their current position.
		TEAM_INCREASE_PRIORITY,										///< Increment a team's ai priority by it's success amount.
		TEAM_DECREASE_PRIORITY,										///< Decrease a team's ai prioiryt by it's failure amount.
		DISPLAY_COUNTER,													///< shows a counter
		HIDE_COUNTER,															///< hides a counter
		TEAM_USE_COMMANDBUTTON_ABILITY_ON_NAMED,	///< Make a team use an ability on another unit.
		TEAM_USE_COMMANDBUTTON_ABILITY_AT_WAYPOINT,			///< Make a team use an ability on a waypoint.
		NAMED_USE_COMMANDBUTTON_ABILITY,					///< Make a unit use an ability.
		TEAM_USE_COMMANDBUTTON_ABILITY,						///< Make a team use an ability.
		NAMED_FLASH_WHITE,												///< Flashes a specific unit
		TEAM_FLASH_WHITE,													///< Flashes a specific team
		SKIRMISH_BUILD_BUILDING,									///< Start building a building.
		SKIRMISH_FOLLOW_APPROACH_PATH,						///< Follow a particular route to the enemy.
		IDLE_ALL_UNITS,														///< cancel any current actions for one player (or all players)
		RESUME_SUPPLY_TRUCKING,										///< Re-enable supply trucking for *idle* trucks for one player (or all players)
		NAMED_CUSTOM_COLOR,												///< set a specific unit to use a special indicator color (not his controlling player's)
		SKIRMISH_MOVE_TO_APPROACH_PATH,						///< Follow a particular route to the enemy.
		SKIRMISH_BUILD_BASE_DEFENSE_FRONT,				///< Build some base defense on the front.
		SKIRMISH_FIRE_SPECIAL_POWER_AT_MOST_COST,	///< Fire a power or superweapon at the enemy's most costly concentration of structures & units..
		NAMED_RECEIVE_UPGRADE,										///< Unit receives an upgrade
		PLAYER_REPAIR_NAMED_STRUCTURE,						///< Send a dozer to repair a structure.
		SKIRMISH_BUILD_BASE_DEFENSE_FLANK,				///< Build some base defense on the flank.
		SKIRMISH_BUILD_STRUCTURE_FRONT,						///< Build some structure on the front.
		SKIRMISH_BUILD_STRUCTURE_FLANK,						///< Build some structure on the flank.
		SKIRMISH_ATTACK_NEAREST_GROUP_WITH_VALUE,	///< Send a team to attack the nearest group with value comparison
		SKIRMISH_PERFORM_COMMANDBUTTON_ON_MOST_VALUABLE_OBJECT,	///< Send a single unit/ all available to perform a command button action on the most valuable target
		SKIRMISH_WAIT_FOR_COMMANDBUTTON_AVAILABLE_ALL,		///< In a sequential script, stay on this step until the specified command button is ready for all units.
		SKIRMISH_WAIT_FOR_COMMANDBUTTON_AVAILABLE_PARTIAL,///< In a sequential script, stay on this step until the specified command button is ready for one unit on the team.
		TEAM_SPIN_FOR_FRAMECOUNT,									///< Tell the team to continue its current task without advancing for this number of frames
		TEAM_ALL_USE_COMMANDBUTTON_ON_NAMED,			///< Tell the team to use the commandbutton on a named enemy
		TEAM_ALL_USE_COMMANDBUTTON_ON_NEAREST_ENEMY_UNIT,	///< Tell the team to use the commandbutton on the nearest enemy unit
		TEAM_ALL_USE_COMMANDBUTTON_ON_NEAREST_GARRISONED_BUILDING,	///< Tell the team to use the commandbutton on the nearest enemy garrisoned building
		TEAM_ALL_USE_COMMANDBUTTON_ON_NEAREST_KINDOF,	///< Tell the team to use the commandbutton on the nearest enemy that has kindof set
		TEAM_ALL_USE_COMMANDBUTTON_ON_NEAREST_ENEMY_BUILDING,	///< Tell the team to use the commandbutton on the nearest enemy building
		TEAM_ALL_USE_COMMANDBUTTON_ON_NEAREST_ENEMY_BUILDING_CLASS,	///< Tell the team to use the commandbutton on the nearest enemy building class
		TEAM_ALL_USE_COMMANDBUTTON_ON_NEAREST_OBJECTTYPE,	///< Tell the team to use the commandbutton on the nearest enemy object of type
		TEAM_PARTIAL_USE_COMMANDBUTTON,						///< Tell some percentage of the team to use the commandbutton on
		TEAM_CAPTURE_NEAREST_UNOWNED_FACTION_UNIT,///< Tell the team to capture the nearest unowned faction unit (unit whose pilot has been killed by Jarmen Kell)
		PLAYER_CREATE_TEAM_FROM_CAPTURED_UNITS,		///< Tell the player to create a new named team from all the units who have been captured.
		PLAYER_ADD_SKILLPOINTS,										///< add/subtract skill points to the player.
		PLAYER_ADD_RANKLEVEL,											///< add/subtract ranklevel for the player.
		PLAYER_SET_RANKLEVEL,											///< set the player's ranklevel to an explicit level.
		PLAYER_SET_RANKLEVELLIMIT,								///< set the RankLevelLimit for the current map.
		PLAYER_GRANT_SCIENCE,											///< grant a Science to the player, regardless of prereqs, etc
		PLAYER_PURCHASE_SCIENCE,									///< the Player purchases the specified Science (if he has enough points, prereqs, etc)
		TEAM_HUNT_WITH_COMMAND_BUTTON,						///< Hunt using a command button ability.
		TEAM_WAIT_FOR_NOT_CONTAINED_ALL,					///< In a sequential script, stay on this step until the entire team is no longer contained
		TEAM_WAIT_FOR_NOT_CONTAINED_PARTIAL,			///< In a sequential script, stay on this step until at least one member of the team is no longer contained
		TEAM_FOLLOW_WAYPOINTS_EXACT,							///< Follows waypoints exactly.
		NAMED_FOLLOW_WAYPOINTS_EXACT,							///< Follows waypoints exactly.
		TEAM_SET_EMOTICON,												///< Adds specified emoticon to each unit in team (-1 permanent, 0 get rid of, 1+ duration in ms)
		NAMED_SET_EMOTICON,												///< Adds specified emoticon to named unit (-1 permanent, 0 get rid of, 1+ duration in ms)
		AI_PLAYER_BUILD_SUPPLY_CENTER,						///< Tell the ai player to build a supply center.
		AI_PLAYER_BUILD_UPGRADE,									///< Tell the ai player to build an upgrade.
		OBJECTLIST_ADDOBJECTTYPE,									///< Add the associated object to the named list
		OBJECTLIST_REMOVEOBJECTTYPE,							///< Remove the associated object to the named list
		MAP_REVEAL_PERMANENTLY_AT_WAYPOINT,				///< Reveal an area at a waypoint
		MAP_UNDO_REVEAL_PERMANENTLY_AT_WAYPOINT,	///< Undo a previous permanent reveal
		NAMED_SET_STEALTH_ENABLED,								///< Enables/disables stealth capability for a specific unit.
		TEAM_SET_STEALTH_ENABLED,									///< Enables/disables stealth capability for a team.
		EVA_SET_ENABLED_DISABLED,									///< Sets EVA to continue to evaluate messages or not.
		OPTIONS_SET_OCCLUSION_MODE,								///< Enables behind building functionality.
		LOCALDEFEAT,															///< Announce local player's defeat.
		OPTIONS_SET_DRAWICON_UI_MODE,							///< Enables/disables all emoticons.
		OPTIONS_SET_PARTICLE_CAP_MODE,						///< Enables/disables particle cap.
		PLAYER_SCIENCE_AVAILABILITY,							///< Determines the availability of a specific science.
		UNIT_AFFECT_OBJECT_PANEL_FLAGS,						///< Affects the various flags on the object panel for an individual unit
		TEAM_AFFECT_OBJECT_PANEL_FLAGS,						///< Affects the various flags on the object panel for all units on a team
		PLAYER_SELECT_SKILLSET,										///< Select the skillset for an AI player.
		SCRIPTING_OVERRIDE_HULK_LIFETIME,					///< Overrides the lifetime of hulks (for cinematic purposes).
		NAMED_FACE_NAMED,													///< Orders unit to face the position of the other named object.
		NAMED_FACE_WAYPOINT,											///< Orders unit to face a waypoint.
		TEAM_FACE_NAMED,													///< Orders all members of a team to face a specific unit.
		TEAM_FACE_WAYPOINT,												///< Orders all members of a team to face a waypoint.
		COMMANDBAR_REMOVE_BUTTON_OBJECTTYPE,			///< Remove a button from a command bar for an objecttype
		COMMANDBAR_ADD_BUTTON_OBJECTTYPE_SLOT,		///< Add a button to the command bar for an objecttype, in a specific slot
		UNIT_SPAWN_NAMED_LOCATION_ORIENTATION,		///< Create a named unit at the specified location, altitude, and orientation
		PLAYER_AFFECT_RECEIVING_EXPERIENCE,				///< Adjust whether or not a player is receiving experience for kills
		PLAYER_EXCLUDE_FROM_SCORE_SCREEN,					///< This player should be listed in the score screen.  Should only be used in campaign games.
		TEAM_GUARD_SUPPLY_CENTER,									///< Have an ai team guard the nearest available supply center..
		ENABLE_SCORING,														///< Turn on scoring of kills, units destroyed, etc.
		DISABLE_SCORING,													///< Turn off scoring of kills, units destroyed, etc.
		SOUND_SET_VOLUME,													///< Adjust the volume of the sounds
		SPEECH_SET_VOLUME,												///< Adjust the volume of the speech
		DISABLE_BORDER_SHROUD,										///< Clear the shroud that draws off the map border
		ENABLE_BORDER_SHROUD,											///< Restore the shroud that draws off the map border (normally on)
		OBJECT_ALLOW_BONUSES,											///< Change whether or not the bonuses are applied to objects.
		SOUND_REMOVE_ALL_DISABLED,								///< Remove any and all disabled sounds from the sound channels.
		SOUND_REMOVE_TYPE,												///< Remove sound of type.
		TEAM_GUARD_IN_TUNNEL_NETWORK,							///< Team enters nearest tunnel network & guards.
		QUICKVICTORY,															///< End map in victory immediately.
		SET_INFANTRY_LIGHTING_OVERRIDE,						///< Overrides the natural lighting on infantry for closeups
		RESET_INFANTRY_LIGHTING_OVERRIDE,					///< And then goes back to the GlobalData defaults
		TEAM_DELETE_LIVING,												///< Delete only the living members of this team
		RESIZE_VIEW_GUARDBAND,										///< Allow bigger objects to be perceived as onscreen near the edge
		DELETE_ALL_UNMANNED,											///< Delete all unmanned (sniped) vehicles
		CHOOSE_VICTIM_ALWAYS_USES_NORMAL,					///< choose victim always uses normal AI behavior, ignoring game difficulty
		CAMERA_ENABLE_SLAVE_MODE,
		CAMERA_DISABLE_SLAVE_MODE,
		CAMERA_ADD_SHAKER_AT,			              	///< WST added 10.12.2002 (MBL)
		SET_TRAIN_HELD,				                    ///< LORENZEN -- Forbids trains from departing stations while true
    NAMED_SET_EVAC_LEFT_OR_RIGHT,	            ///< LORENZEN -- Which side of the garrisoned unit (LIKELY A TRAIN) do you want units to evacuate from?
    ENABLE_OBJECT_SOUND,                      ///< Enables the ambient sound on an object or fire the one-shot "ambient" sound on an object
    DISABLE_OBJECT_SOUND,                     ///< Disable the ambient sound on an object or kill the one-shot "ambient" sound on an object
		NAMED_USE_COMMANDBUTTON_ABILITY_USING_WAYPOINT_PATH, ///< Added for particle cannon to have beam follow waypoint path.
		NAMED_SET_UNMANNED_STATUS,								///< Make unit unmanned or manned.
		TEAM_SET_UNMANNED_STATUS,									///< Make all units on team unmanned or manned.
		NAMED_SET_BOOBYTRAPPED,										///< Add boobytrap to unit.
		TEAM_SET_BOOBYTRAPPED,										///< Add boobytrap to all units on team.
		SHOW_WEATHER,															///< show map defined weather.
		AI_PLAYER_BUILD_TYPE_NEAREST_TEAM,				///< Tell the ai player to build an object nearest team.
		// add new items here, please
		NUM_ITEMS
	};
	ScriptAction(ScriptActionType type);
	//~ScriptAction();

	ScriptAction *duplicate() const;
	ScriptAction *duplicateAndQualify(const AsciiString& qualifier,
			const AsciiString& playerTemplateName, const AsciiString& newPlayerName) const;

protected:
		ScriptAction();  ///< Protected constructor for read.

protected:
	ScriptActionType	m_actionType;
	Int 							m_numParms;
	Parameter *				m_parms[MAX_PARMS];
	ScriptAction *		m_nextAction;
	Bool							m_hasWarnings; ///< Runtime flag used by the editor only.

public:
	void setActionType(ScriptActionType type);

public:
	void setNextAction(ScriptAction *pAct) {m_nextAction = pAct;}
	void setWarnings(Bool warnings) { m_hasWarnings = warnings;}
	ScriptActionType getActionType() {return m_actionType;}
	ScriptAction * getNext() {return m_nextAction;}
	AsciiString getUiText();
	Parameter *getParameter(Int ndx)
	{
		if (ndx>=0 && ndx<m_numParms)
			return m_parms[ndx];

		return nullptr;
	}
	Bool hasWarnings() const { return m_hasWarnings;}
	Int getNumParameters() {return m_numParms;}
	Int getUiStrings(AsciiString strings[MAX_PARMS]);

	static void WriteActionDataChunk(DataChunkOutput &chunkWriter, ScriptAction *pAct);
	static Bool ParseActionDataChunk(DataChunkInput &file, DataChunkInfo *info, void *userData);
	static void WriteActionFalseDataChunk(DataChunkOutput &chunkWriter, ScriptAction *pAct);
	static Bool ParseActionFalseDataChunk(DataChunkInput &file, DataChunkInfo *info, void *userData);

protected:
	static ScriptAction *ParseAction(DataChunkInput &file, DataChunkInfo *info, void *userData);

};

//-------------------------------------------------------------------------------------------------
// ******************************** class Script ***********************************************
//-------------------------------------------------------------------------------------------------
/** This is a script, in a linked list of scripts.  The m_condition list is evaluated, and if one of the
or clauses is true, the m_action list is executed.  . */
class Script : public MemoryPoolObject, public Snapshot
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(Script, "Script")
protected:	// Note - If you add any member vars, you must take them into account in duplicate() and updateFrom(), as well as file read/write.

	// snapshot methods
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

	AsciiString	m_scriptName;   ///<Short name.
	AsciiString m_comment;			///< Long comment.
	AsciiString m_conditionComment;			///< Long comment about conditions.
	AsciiString m_actionComment;			///< Long comment about actions.
	Int					m_delayEvaluationSeconds; ///< Number of seconds to delay evaluation.
	Bool				m_isActive;			 ///< If true, script is enabled.  If false, disabled.
	Bool				m_isOneShot;    ///< If true, script is made not active after executing the m_action actions.
	Bool				m_isSubroutine; ///< If true, script is a subroutine.
	Bool				m_easy;					///< Applies to easy campaign.
	Bool				m_normal;				///< Applies to normal campaign.
	Bool				m_hard;					///< Applies to Hard campaign.

	OrCondition *m_condition;		///< First in a list of ORed clauses.
	ScriptAction *m_action;			///< First in a list of actions executed if the conditions are true.
	ScriptAction *m_actionFalse;///< First in a list of actions executed if the conditions are false.
	Script			*m_nextScript;	///< Next in the list of scripts.
	UnsignedInt m_frameToEvaluateAt; ///< When to evaluate the conditions next, if m_delayEvaluationSeconds>0.

// Runtime fields - not saved or read.
	Bool				m_hasWarnings; ///< Runtime flag used by the editor only.
	AsciiString	m_conditionTeamName; ///< Runtime name used by ScriptEngine only.
	Real				m_conditionTime;		///< Amount of time (cum) to evaluate conditions.
	Real				m_curTime;		///< Amount of time (cum) to evaluate conditions.
	Int					m_conditionExecutedCount; ///< Number of times conditions evaluated.

public:
	Script();
	//~Script();
	Script *duplicate() const;	// note, duplicates just this node, not the full list.
	Script *duplicateAndQualify(const AsciiString& qualifier,
			const AsciiString& playerTemplateName, const AsciiString& newPlayerName) const;

public:
	void setName(AsciiString name) { m_scriptName = name;}
	void setWarnings(Bool warnings) { m_hasWarnings = warnings;}
	void setComment(AsciiString comment) { m_comment = comment;}
	void setActionComment(AsciiString comment) { m_actionComment = comment;}
	void setConditionComment(AsciiString comment) { m_conditionComment = comment;}
	void setActive(Bool active) { m_isActive = active;}
	void setOneShot(Bool one) { m_isOneShot = one;}
	void setEasy(Bool easy) { m_easy = easy;}
	void setNormal(Bool normal) { m_normal = normal;}
	void setHard(Bool hard) { m_hard = hard;}
	void setSubroutine(Bool subr) { m_isSubroutine = subr;}
	void setNextScript(Script *pScr) {m_nextScript = pScr;}
	void setOrCondition(OrCondition *pCond) {m_condition = pCond;}
	void setAction(ScriptAction *pAction) {m_action = pAction;}
	void setFalseAction(ScriptAction *pAction) {m_actionFalse = pAction;}
	void updateFrom(Script *pSrc); ///< Updates this from pSrc.  pSrc IS MODIFIED - it's guts are removed.  jba.
	void setFrameToEvaluate(UnsignedInt frame) {m_frameToEvaluateAt=frame;}
	void incrementConditionCount() {m_conditionExecutedCount++;}
	void addToConditionTime(Real time) {m_conditionTime += time;}
	void setCurTime(Real time) {m_curTime	= time;}
	void setDelayEvalSeconds(Int delay) {m_delayEvaluationSeconds = delay;}

	UnsignedInt getFrameToEvaluate() {return m_frameToEvaluateAt;}
	Int getConditionCount() {return m_conditionExecutedCount;}
	Real getConditionTime() {return m_conditionTime;}
	Real getCurTime() {return m_curTime;}
	Int getDelayEvalSeconds() {return m_delayEvaluationSeconds;}

	const AsciiString& getName() const { return m_scriptName;}
	AsciiString getComment() const {return m_comment;}
	AsciiString getActionComment() const {return m_actionComment;}
	AsciiString getConditionComment() const {return m_conditionComment;}
	Bool hasWarnings() const { return m_hasWarnings;}
	Bool isActive() const { return m_isActive;}
	Bool isOneShot() const { return m_isOneShot;}
	Bool isEasy() const { return m_easy;}
	Bool isNormal() const { return m_normal;}
	Bool isHard() const { return m_hard;}
	Bool isSubroutine() const { return m_isSubroutine;}
	Script *getNext() const {return m_nextScript;}
	OrCondition *getOrCondition() const {return m_condition;}
	ScriptAction *getAction() const	{return m_action;}
	ScriptAction *getFalseAction() const {return m_actionFalse;}
	AsciiString getUiText();

	void deleteOrCondition(OrCondition *pCond);
	void deleteAction(ScriptAction *pAct);
	void deleteFalseAction(ScriptAction *pAct);

	static void WriteScriptDataChunk(DataChunkOutput &chunkWriter, Script *pScript);
	static Script *ParseScript(DataChunkInput &file, unsigned short version);
	static Bool ParseScriptFromListDataChunk(DataChunkInput &file, DataChunkInfo *info, void *userData);
	static Bool ParseScriptFromGroupDataChunk(DataChunkInput &file, DataChunkInfo *info, void *userData);

	// Support routines for moving conditions around
	OrCondition *findPreviousOrCondition( OrCondition *curOr );

	// Support routines for ScriptEngine -
	const AsciiString& getConditionTeamName() const {return m_conditionTeamName;}
	void setConditionTeamName(AsciiString teamName) {m_conditionTeamName = teamName;}
};

//-------------------------------------------------------------------------------------------------
// ******************************** class Parameter ***********************************************
//-------------------------------------------------------------------------------------------------
/** This is a parameter, in either a condition or an action. */
class Parameter : public MemoryPoolObject
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(Parameter, "Parameter")
// friend bad for MPOs. (srj)
//friend class EditParameter;
//friend class EditCoordParameter;
//friend class EditObjectParameter;
//friend class ScriptEngine;
//friend class ScriptConditions;

public:

	enum ParameterType
	{
		INT=0,							// Int.
		REAL,								// Real.
		SCRIPT,							// String.	Any script.
		TEAM,								// String.
		COUNTER,						// String, int index into counters array at runtime.
		FLAG,								// String, int index into flags array at runtime.
		COMPARISON,					// Int, see comparison types below.
		WAYPOINT,						// String.
		BOOLEAN,						// Int 0 or 1 for flags.
		TRIGGER_AREA,				// String.
		TEXT_STRING,				// String.
		SIDE,								// String.
		SOUND,							// String.
		SCRIPT_SUBROUTINE,	// String.  Script that is a subroutine.
		UNIT,								// String.  Name of a unit.
		OBJECT_TYPE,				// String.  Name of a thing template.
		COORD3D,						// String.  (x,y,z), ie (2.0,7,5).	Use getCoord()
		ANGLE,							// Real.  Radians.
		TEAM_STATE,					// String.  Name of a state that a team is in.
		RELATION,						// Int. Specifies the relation towards a team
		AI_MOOD,						// Int. Specifies the aggressiveness towards a team.
		DIALOG,							// String, specifies string usable with getSpeech(...)
		MUSIC,							// String, specifies string usable with getTrack(...)
		MOVIE,							// String, specifies string usable with open(...) or load(...)
		WAYPOINT_PATH,			// String, waypoint path label.
		LOCALIZED_TEXT,			// String, must be looked up from the localized text file.
		BRIDGE,							// String, Named bridge.
		KIND_OF_PARAM,			// Integer, KindOfType enum.
		ATTACK_PRIORITY_SET,// String, name of an attack priority set.
		RADAR_EVENT_TYPE,		// Int. Specifies the radar Event type
		SPECIAL_POWER,			// String, specifies superweapon types
		SCIENCE,						// String, specifies a science/general/faction/buzzword-of-the-week
		UPGRADE,						// String, specifies a type of upgrade
		COMMANDBUTTON_ABILITY, // Int, refers to the appropriate 1-12 commandbutton that is used for an object.
		BOUNDARY,						// Int, specifies a boundary index.
		BUILDABLE,					// Int, specifies buildability of a specific object.
		SURFACES_ALLOWED,		// Int, specifies whether the check should consider Air, Ground, or Air and Ground
		SHAKE_INTENSITY,		// Int, specifies which Intensity to use (Intensity in 10 cities!)
		COMMAND_BUTTON,  		// String
		FONT_NAME,       		// String, the name of the desired font
		OBJECT_STATUS,			// String, specifies ObjectStatusTypes name. However, translated to an ObjectStatusMaskType on read
		COMMANDBUTTON_ALL_ABILITIES, // String, refers to all command buttons
		SKIRMISH_WAYPOINT_PATH, // String, name of a predefined skirmish waypoint path.
		COLOR,							// color (as int) in ARGB format.
		EMOTICON,						// List of all possible emoticons.
		OBJECT_PANEL_FLAG,	// String, specifies a flag name available on the options panel.
		FACTION_NAME,				// The name of the faction (GLA, America, Munkee, China, etc)
		OBJECT_TYPE_LIST,		// String, Special case of Object Type.
		REVEALNAME,					// String, the name of the look taking place.
		SCIENCE_AVAILABILITY, // String, the name of the different science availabilities.
    LEFT_OR_RIGHT,        // 1=left, 2=right, okay?
		PERCENT,						// Real.  A percentage.
		NUM_ITEMS
	};

	enum {
		// Comparison types.  Stored in Int value.
		LESS_THAN = 0,
		LESS_EQUAL,
		EQUAL,
		GREATER_EQUAL,
		GREATER,
		NOT_EQUAL
	};

	enum {
		// Relation types. Stored in Int value. (Values from GameCommon.h)
		REL_ENEMY			= ENEMIES,
		REL_NEUTRAL		= NEUTRAL,
		REL_FRIEND		= ALLIES
	};

	Parameter(ParameterType type, int val = 0) :
		m_initialized(false),
		m_paramType(type),
		m_int(val),
		m_real(0)
	{
		m_coord.x=0;m_coord.y=0;m_coord.z=0;
	}

private:
	ParameterType	m_paramType;
	Bool					m_initialized;
	Int						m_int;
	Real					m_real;
	AsciiString		m_string;
	Coord3D				m_coord;
	ObjectStatusMaskType m_objectStatus;

protected:
	void setInt(Int i) {m_int = i;}
	void setReal(Real r) {m_real = r;}
	void setCoord3D(const Coord3D *pLoc);
	void setString(AsciiString s) {m_string = s;}
	void setStatus( ObjectStatusMaskType objectStatus ) { m_objectStatus.set( objectStatus ); }

public:
	Int getInt() const {return m_int;}
	Real getReal() const {return m_real;}
	void getCoord3D(Coord3D *pLoc) const;
	ParameterType getParameterType() const {return m_paramType;}
	ObjectStatusMaskType getStatus() const { return m_objectStatus; }

	void friend_setInt(Int i) {m_int = i;}
	void friend_setReal(Real r) {m_real = r;}
	void friend_setCoord3D(const Coord3D *pLoc) { setCoord3D(pLoc); }
	void friend_setString(AsciiString s) {m_string = s;}

	void qualify(const AsciiString& qualifier,const AsciiString& playerTemplateName,const AsciiString& newPlayerName);

	const AsciiString& getString() const {return m_string;}
	AsciiString getUiText() const;

	void WriteParameter(DataChunkOutput &chunkWriter);
	static Parameter *ReadParameter(DataChunkInput &file);

};
EMPTY_DTOR(Parameter)

extern const char* const TheObjectFlagsNames[];

//-------------------------------------------------------------------------------------------------
// ******************************** class Condition ***********************************************
//-------------------------------------------------------------------------------------------------
/** This is a condition.  The parameters for a ConditionType are set up in
ConditionTemplates created in ScriptEngine::init.

// SPECIAL NOTE ABOUT Skirmish Scripts: Please note that ALL Skirmish conditions should first pass a pSkirmishPlayerParm to
// prevent the necessity of having to write additional scripts for other players / skirmish types later.
*/

class Condition : public MemoryPoolObject  // This is the conditional class.
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(Condition, "Condition")
// friend bad for MPOs. (srj)
//friend class EditCondition;
public:
	enum ConditionType
	{
		CONDITION_FALSE,										// Always evaluates to false.
		COUNTER,															// COUNTER, COMPARISON, INT
		FLAG,																	// FLAG BOOLEAN compares flag to value.
		CONDITION_TRUE,												// Always evaluates to true.
		TIMER_EXPIRED,												// If a timer expired.
		PLAYER_ALL_DESTROYED,									// If a side (player) is totally destroyed.
		PLAYER_ALL_BUILDFACILITIES_DESTROYED, // If a side (player) has all build facilities destroyed.
		TEAM_INSIDE_AREA_PARTIALLY,						// If a player has units from a team entering an area.
		TEAM_DESTROYED,												// If a team has been destroyed.
		CAMERA_MOVEMENT_FINISHED,							// If the camera has completed it's current movement.
		TEAM_HAS_UNITS,												// If a team has units.
		TEAM_STATE_IS,												// If a team is in a particular state.
		TEAM_STATE_IS_NOT,										// If a team is not in a particular state.
		NAMED_INSIDE_AREA,										// If a player has units inside an area.
		NAMED_OUTSIDE_AREA,										// If a player has units outside an area.
		NAMED_DESTROYED,											// If a unit is destroyed
		NAMED_NOT_DESTROYED,									// If a unit is destroyed
		TEAM_INSIDE_AREA_ENTIRELY,						// If a team is entirely inside an area.
		TEAM_OUTSIDE_AREA_ENTIRELY,						// If a team is entirely outside an area.
		NAMED_ATTACKED_BY_OBJECTTYPE,					// If a unit has been attacked by an object of type objecttype
		TEAM_ATTACKED_BY_OBJECTTYPE,					// If a team has been attacked by an object of type objecttype
		NAMED_ATTACKED_BY_PLAYER,							// If a unit has been attacked by a unit controlled by player
		TEAM_ATTACKED_BY_PLAYER,							// If a team has been attacked by a unit controlled by player
		BUILT_BY_PLAYER,											// If a unit of type objecttype has been built by player
		NAMED_CREATED,												// If a unit is currently in existence
		TEAM_CREATED,													// If a team is currently in existence
		PLAYER_HAS_CREDITS,										// If X [Less Than | Equal | More than] Players credits
		NAMED_DISCOVERED,											// If a named unit is currently "seen" by a player.
		TEAM_DISCOVERED,											// If a unit in a team is currently "seen" by a player.
		MISSION_ATTEMPTS,							// Impl JKMCD
		NAMED_OWNED_BY_PLAYER,								// If a unit is currently owned by a player
		TEAM_OWNED_BY_PLAYER,									// If a team is currently owned by a player
		PLAYER_HAS_N_OR_FEWER_BUILDINGS,			// If a player has count or fewer buildings
		PLAYER_HAS_POWER,											// If a player currently has power to his facilities
		NAMED_REACHED_WAYPOINTS_END,					// If a unit has reached the end of the path that contains Waypoint
		TEAM_REACHED_WAYPOINTS_END,						// If a team has reached the end of the path that contains Waypoint
		NAMED_SELECTED,												// If a unit is selected
		NAMED_ENTERED_AREA,										// If a unit enters an area
		NAMED_EXITED_AREA,										// If a unit exits an area
		TEAM_ENTERED_AREA_ENTIRELY,						// If a team has entirely entered an area
		TEAM_ENTERED_AREA_PARTIALLY,					// If a team has exactly one unit entering an area
		TEAM_EXITED_AREA_ENTIRELY,						// If a team has entirely exited an area
		TEAM_EXITED_AREA_PARTIALLY,						// If a team has exactly one unit exiting an area
		MULTIPLAYER_ALLIED_VICTORY,						// If a single group of allies has vanquished everyone else
		MULTIPLAYER_ALLIED_DEFEAT,						// If a single group of allies has been vanquished
		MULTIPLAYER_PLAYER_DEFEAT,						// If a player (but not necessarily his allies) has been defeated
		PLAYER_HAS_NO_POWER,									// If a player doesn't have power
		HAS_FINISHED_VIDEO,										// If a video has completed playing
		HAS_FINISHED_SPEECH,									// If a speech has completed playing
		HAS_FINISHED_AUDIO,										// If an audio piece has completed playing
		BUILDING_ENTERED_BY_PLAYER,						// If a player has garrisoned a building.
		ENEMY_SIGHTED,												// If a unit sees any type of unit/building belonging to a player.
		UNIT_HEALTH,													// Compares the unit health
		BRIDGE_REPAIRED,											// If a bridge has been repaired.
		BRIDGE_BROKEN,												// If a bridge has been broken.
		NAMED_DYING,													// A unit has been killed, but is in the process of dying (using slow death state)
		NAMED_TOTALLY_DEAD,										// A unit is done dying.
		PLAYER_HAS_OBJECT_COMPARISON,					// Does a player have more/less than X number of units of type object
		OBSOLETE_SCRIPT_1,										// Obsolete!
		OBSOLETE_SCRIPT_2,										// Obsolete!
		PLAYER_TRIGGERED_SPECIAL_POWER,				// has a player triggered a specific special power type
		PLAYER_COMPLETED_SPECIAL_POWER,				// has a player's special power completed?
		PLAYER_MIDWAY_SPECIAL_POWER,					// If the special power is midway through activation
		PLAYER_TRIGGERED_SPECIAL_POWER_FROM_NAMED,	// has a player triggered a specific special power type
		PLAYER_COMPLETED_SPECIAL_POWER_FROM_NAMED,	// has a player's special power completed?
		PLAYER_MIDWAY_SPECIAL_POWER_FROM_NAMED,			// If the special power is midway through activation
		DEFUNCT_PLAYER_SELECTED_GENERAL,							// no longer in use!
		DEFUNCT_PLAYER_SELECTED_GENERAL_FROM_NAMED,		// no longer in use!
		PLAYER_BUILT_UPGRADE,									// If a player built a specific upgrade
		PLAYER_BUILT_UPGRADE_FROM_NAMED,			// If a player built a specific upgrade from a specific building
		PLAYER_DESTROYED_N_BUILDINGS_PLAYER,	// If a player has destroyed N or more buildings of a specific player
		UNIT_COMPLETED_SEQUENTIAL_EXECUTION,	// If a unit has completed sequential execution
		TEAM_COMPLETED_SEQUENTIAL_EXECUTION,	// If a team has completed sequential execution
		PLAYER_HAS_COMPARISON_UNIT_TYPE_IN_TRIGGER_AREA, // If player has a specific unit or building type...
		PLAYER_HAS_COMPARISON_UNIT_KIND_IN_TRIGGER_AREA, // If player has a kindof in a trigger area.
		UNIT_EMPTIED,													// True if a unit has just emptied its cargo
		TYPE_SIGHTED,														// True if a specific unit has just seen a type of unit belonging to a specific player
		NAMED_BUILDING_IS_EMPTY,								// True if a specific building is empty
		PLAYER_HAS_N_OR_FEWER_FACTION_BUILDINGS,			// If a player has count or fewer faction buildings
		UNIT_HAS_OBJECT_STATUS,									// True if an object has that object status bit set to true
		TEAM_ALL_HAS_OBJECT_STATUS,							// True if all objects on a team have that object status bit set to true
		TEAM_SOME_HAVE_OBJECT_STATUS,						// True if an object on the team has that object status bit set to true
		PLAYER_POWER_COMPARE_PERCENT,						// True if a player's power ratio compares to a percentage.
		PLAYER_EXCESS_POWER_COMPARE_VALUE,			// True if a player has comparison  power in kilowatts.
		SKIRMISH_SPECIAL_POWER_READY,						// True if the special power is ready to fire.
		SKIRMISH_VALUE_IN_AREA,									// True if the units in an area are worth comparison to a value
		SKIRMISH_PLAYER_FACTION,								// True if player is a specific faction
		SKIRMISH_SUPPLIES_VALUE_WITHIN_DISTANCE,// True if there are supplies within distance of the perimeter
		SKIRMISH_TECH_BUILDING_WITHIN_DISTANCE,	// True if there is a tech building within distance of the perimeter
		SKIRMISH_COMMAND_BUTTON_READY_ALL,			// True when ???
		SKIRMISH_COMMAND_BUTTON_READY_PARTIAL,	// True when ???
		SKIRMISH_UNOWNED_FACTION_UNIT_EXISTS,		// True if there are comparison to unowned faction unit on the map.
		SKIRMISH_PLAYER_HAS_PREREQUISITE_TO_BUILD,	// True if we can build this thing (we have prerequisites)
		SKIRMISH_PLAYER_HAS_COMPARISON_GARRISONED,	// True if there are comparison to garrisoned buildings (by player) on the map
		SKIRMISH_PLAYER_HAS_COMPARISON_CAPTURED_UNITS,	// True if there are comparison to captured units
		SKIRMISH_NAMED_AREA_EXIST,							// True if a named area exists
		SKIRMISH_PLAYER_HAS_UNITS_IN_AREA,			// True if a player has units in an area
		SKIRMISH_PLAYER_HAS_BEEN_ATTACKED_BY_PLAYER,	// True if the player has been attacked by a player
		SKIRMISH_PLAYER_IS_OUTSIDE_AREA,				// True if player doesn't have any units in an area
		SKIRMISH_PLAYER_HAS_DISCOVERED_PLAYER,	// True if a player has discovered another player

		PLAYER_ACQUIRED_SCIENCE,								// True if the player has acquired the specified Science.
		PLAYER_HAS_SCIENCEPURCHASEPOINTS,				// True if the player has at least N SciencePurchasePoints.
		PLAYER_CAN_PURCHASE_SCIENCE,						// True is player can purchase the given science (has all prereqs & points needed)
		MUSIC_TRACK_HAS_COMPLETED,							// True if the specified track has completed at least N times
		PLAYER_LOST_OBJECT_TYPE,								// True if a player has lost a specific unit type

		SUPPLY_SOURCE_SAFE,											// True if the nearest available supply source is not under enemy influence.
		SUPPLY_SOURCE_ATTACKED,									// True if our supply depot or dozer near depot was attacked.
		START_POSITION_IS,											// True if our start position matches.
		NAMED_HAS_FREE_CONTAINER_SLOTS,					///< Kris -- Checks if any given container has any free slots.

		NUM_ITEMS		 // Always the last condition.
	};

	Condition(enum ConditionType type);
	//~Condition();

	Condition *duplicate() const;
	Condition *duplicateAndQualify(const AsciiString& qualifier,
			const AsciiString& playerTemplateName, const AsciiString& newPlayerName) const;

protected:
	Condition();  ///< Protected constructor for read.

private:
	enum ConditionType	m_conditionType;
	Int 		m_numParms;
	Parameter *m_parms[MAX_PARMS];
	Condition *m_nextAndCondition;

	Int				m_hasWarnings; ///< Runtime flag used by the editor only.
	Int				m_customData;  ///< Custom data for cacheing.
	UnsignedInt m_customFrame; ///< Custom frame count for cacheing.

public:
	void setConditionType(enum ConditionType type);

public:
	void setNextCondition(Condition *pScr) {m_nextAndCondition = pScr;}
	void setWarnings(Bool warnings) { m_hasWarnings = warnings;}
	enum ConditionType getConditionType() {return m_conditionType;}
	Condition * getNext() {return m_nextAndCondition;}
	AsciiString getUiText();
	Parameter *getParameter(Int ndx)
	{
		if (ndx>=0 && ndx<m_numParms)
			return m_parms[ndx];

		return nullptr;
	}

	Int getNumParameters() {return m_numParms;}
	Int getUiStrings(AsciiString strings[MAX_PARMS]);
	Bool hasWarnings() const { return m_hasWarnings;}
	Int getCustomData() const {return m_customData;}
	void setCustomData(Int val) { m_customData = val;}

	Int getCustomFrame() const {return m_customFrame;}
	void setCustomFrame(Int val) { m_customFrame = val;}

	static void WriteConditionDataChunk(DataChunkOutput &chunkWriter, Condition *pCond);
	static Bool ParseConditionDataChunk(DataChunkInput &file, DataChunkInfo *info, void *userData);

};

#define dontCOUNT_SCRIPT_USAGE

//-------------------------------------------------------------------------------------------------
// ******************************** class Template ***********************************************
//-------------------------------------------------------------------------------------------------
/** The template defines the parameters and ui strings used to display a
condition or action. */
class Template : public MemoryPoolObject
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(Template, "Template")
// friend bad for MPOs. (srj)
//friend class ScriptEngine;
public:
	AsciiString m_uiName;
	AsciiString m_uiName2;
	AsciiString m_internalName;
	NameKeyType m_internalNameKey; // matches internal name.jba [3/20/2003]
	Int					m_numUiStrings;
	AsciiString m_uiStrings[MAX_PARMS];
	Int					m_numParameters;
	enum Parameter::ParameterType m_parameters[MAX_PARMS];
	AsciiString m_helpText;
#ifdef COUNT_SCRIPT_USAGE
	mutable Int					m_numTimesUsed;
	mutable AsciiString m_firstMapUsed;
#endif

public:
	Template();

public:
	AsciiString getName() const {return m_uiName;}
	AsciiString getName2() const {return m_uiName2;}
	Int getUiStrings(AsciiString strings[MAX_PARMS]) const;
	Int getNumParameters() const {return m_numParameters;}
	enum Parameter::ParameterType getParameterType(Int ndx) const;
};
EMPTY_DTOR(Template)

//-------------------------------------------------------------------------------------------------
// ******************************** class ConditionTemplate ***********************************************
//-------------------------------------------------------------------------------------------------
/// Template for condition.
class ConditionTemplate : public Template{};

//-------------------------------------------------------------------------------------------------
// ******************************** class ActionTemplate ***********************************************
//-------------------------------------------------------------------------------------------------
/// Template for action.
class ActionTemplate : public Template{};

//-------------------------------------------------------------------------------------------------
// ******************************** class ScriptList ***********************************************
//-------------------------------------------------------------------------------------------------
/// List of scripts.  May have one level of hierarchy in the ScriptGroup list.
class ScriptList : public MemoryPoolObject, public Snapshot
// A script list belongs to a Player.  It contains the heads of the
// the top level scripts, and the script groups.
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(ScriptList, "ScriptList")
protected:

	// snapshot methods
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

	ScriptGroup		*m_firstGroup;
	Script				*m_firstScript;
	static Int		m_curId;

	static ScriptList *s_readLists[MAX_PLAYER_COUNT];
	static Int				s_numInReadList;

public:
	ScriptList();
	//~ScriptList();

public:
	static void updateDefaults();
	static void reset();
	static Int getNextID() {m_curId++; return m_curId;};

public:
	ScriptGroup *getScriptGroup() {return m_firstGroup;};
	Script *getScript() {return m_firstScript;};
	void WriteScriptListDataChunk(DataChunkOutput &chunkWriter);
	static Bool ParseScriptListDataChunk(DataChunkInput &file, DataChunkInfo *info, void *userData);

	void addGroup(ScriptGroup *pGrp, Int ndx);
	void addScript(Script *pScr, Int ndx);
	void deleteScript(Script *pScr);
	void deleteGroup(ScriptGroup *pGrp);

	void discard();

	ScriptList *duplicate() const;
	ScriptList *duplicateAndQualify(const AsciiString& qualifier,
			const AsciiString& playerTemplateName, const AsciiString& newPlayerName ) const;
	/// Reads a set of scripts into m_readScripts.  Use getReadScripts to access.
	static Bool ParseScriptsDataChunk(DataChunkInput &file, DataChunkInfo *info, void *userData);
	/// Writes sides (including build list info.)
	static void WriteScriptsDataChunk(DataChunkOutput &chunkWriter, ScriptList *scriptLists[], Int numLists);

	/// Returns array of script list pointers.  This can only be called once after scripts
	/// are read, and the caller is responsible for deleting the scripts.
	static Int getReadScripts(ScriptList *scriptLists[MAX_PLAYER_COUNT]);
};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/ScriptEngine/Scripts.cpp`
```cpp
// FILE: Scripts.cpp /////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------
//
//                       Westwood Studios Pacific.
//
//                       Confidential Information
//                Copyright (C) 2001 - All Rights Reserved
//
//-----------------------------------------------------------------------------
//
// Project:   Generals
//
// File name: Scripts.cpp
//
// Created:   John Ahlquist, Nov 2001
//
// Desc:      Contains the information describing scripts.
//
//-----------------------------------------------------------------------------
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#define DEFINE_BUILDABLE_STATUS_NAMES
#define DEFINE_OBJECT_STATUS_NAMES
#define DEFINE_SCIENCE_AVAILABILITY_NAMES

#include "Common/BorderColors.h"
#include "Common/DataChunk.h"
#include "Common/GameState.h"
#include "Common/KindOf.h"
#include "Common/Radar.h"
#include "Common/ThingTemplate.h"
#include "Common/Player.h"
#include "Common/Xfer.h"

#include "GameClient/ShellHooks.h"
#include "GameClient/View.h"

#include "GameLogic/AI.h"
#include "GameLogic/Object.h"
#include "GameLogic/ScriptEngine.h"
#include "GameLogic/SidesList.h"
#include "GameLogic/Module/ContainModule.h"




static Script *s_mtScript = nullptr;
static ScriptGroup *s_mtGroup = nullptr;

//
// These strings must be in the same order as they are in their definitions
// (See SHELL_SCRIPT_HOOK_* )
//
const char *const TheShellHookNames[]=
{
	"ShellMainMenuCampaignPushed", //SHELL_SCRIPT_HOOK_MAIN_MENU_CAMPAIGN_SELECTED,
	"ShellMainMenuCampaignHighlighted", //SHELL_SCRIPT_HOOK_MAIN_MENU_CAMPAIGN_HIGHLIGHTED,
	"ShellMainMenuCampaignUnhighlighted", //SHELL_SCRIPT_HOOK_MAIN_MENU_CAMPAIGN_UNHIGHLIGHTED,

	"ShellMainMenuSkirmishPushed", //SHELL_SCRIPT_HOOK_MAIN_MENU_SKIRMISH_SELECTED,
	"ShellMainMenuSkirmishHighlighted", //SHELL_SCRIPT_HOOK_MAIN_MENU_SKIRMISH_HIGHLIGHTED,
	"ShellMainMenuSkirmishUnhighlighted", //SHELL_SCRIPT_HOOK_MAIN_MENU_SKIRMISH_UNHIGHLIGHTED,

	"ShellMainMenuOptionsPushed", //SHELL_SCRIPT_HOOK_MAIN_MENU_OPTIONS_SELECTED,
	"ShellMainMenuOptionsHighlighted", //SHELL_SCRIPT_HOOK_MAIN_MENU_OPTIONS_HIGHLIGHTED,
	"ShellMainMenuOptionsUnhighlighted", //SHELL_SCRIPT_HOOK_MAIN_MENU_OPTIONS_UNHIGHLIGHTED,

	"ShellMainMenuOnlinePushed", //SHELL_SCRIPT_HOOK_MAIN_MENU_ONLINE_SELECTED,
	"ShellMainMenuOnlineHighlighted", //SHELL_SCRIPT_HOOK_MAIN_MENU_ONLINE_HIGHLIGHTED,
	"ShellMainMenuOnlineUnhighlighted", //SHELL_SCRIPT_HOOK_MAIN_MENU_ONLINE_UNHIGHLIGHTED,

	"ShellMainMenuNetworkPushed", //SHELL_SCRIPT_HOOK_MAIN_MENU_NETWORK_SELECTED,
	"ShellMainMenuNetworkHighlighted", //SHELL_SCRIPT_HOOK_MAIN_MENU_NETWORK_HIGHLIGHTED,
	"ShellMainMenuNetworkUnhighlighted", //SHELL_SCRIPT_HOOK_MAIN_MENU_NETWORK_UNHIGHLIGHTED,

	"ShellMainMenuExitPushed", //SHELL_SCRIPT_HOOK_MAIN_MENU_EXIT_SELECTED,
	"ShellMainMenuExitHighlighted", //SHELL_SCRIPT_HOOK_MAIN_MENU_EXIT_HIGHLIGHTED,
	"ShellMainMenuExitUnhighlighted", //SHELL_SCRIPT_HOOK_MAIN_MENU_EXIT_UNHIGHLIGHTED,

	"ShellGeneralsOnlineLogin", //SHELL_SCRIPT_HOOK_GENERALS_ONLINE_LOGIN,
	"ShellGeneralsOnlineLogout", //SHELL_SCRIPT_HOOK_GENERALS_ONLINE_LOGOUT,
	"ShellGeneralsOnlineEnteredFromGame", //SHELL_SCRIPT_HOOK_GENERALS_ONLINE_ENTERED_FROM_GAME,

	"ShellOptionsOpened", //SHELL_SCRIPT_HOOK_OPTIONS_OPENED,
	"ShellOptionsClosed", //SHELL_SCRIPT_HOOK_OPTIONS_CLOSED,

	"ShellSkirmishOpened", //SHELL_SCRIPT_HOOK_SKIRMISH_OPENED,
	"ShellSkirmishClosed", //SHELL_SCRIPT_HOOK_SKIRMISH_CLOSED,
	"ShellSkirmishEnteredFromGame", //SHELL_SCRIPT_HOOK_SKIRMISH_ENTERED_FROM_GAME,

	"ShellLANOpened", //SHELL_SCRIPT_HOOK_LAN_OPENED,
	"ShellLANClosed", //SHELL_SCRIPT_HOOK_LAN_CLOSED,
	"ShellLANEnteredFromGame", //SHELL_SCRIPT_HOOK_LAN_ENTERED_FROM_GAME,
};
static_assert(ARRAY_SIZE(TheShellHookNames) == SHELL_SCRIPT_HOOK_TOTAL, "Incorrect array size");

void SignalUIInteraction(Int interaction)
{
	if (TheScriptEngine)
		TheScriptEngine->signalUIInteract(TheShellHookNames[interaction]);
}


// Changing the order or meaning of either of these will require you to update the maps
// in a meaningful way. If there are new entries, add them to the end, rather than the middle.
const char *Surfaces[] = {
	"Ground",
	"Air",
	"Ground or Air"
};
const char *ShakeIntensities[] = {
	"Subtle",
	"Normal",
	"Strong",
	"Severe",
	"Cine_Extreme",
	"Cine_Insane"
};
static_assert(ARRAY_SIZE(ShakeIntensities) == View::SHAKE_COUNT, "Incorrect array size");

enum { K_SCRIPT_LIST_DATA_VERSION_1 = 1,
			K_SCRIPT_GROUP_DATA_VERSION_1 = 1,
			K_SCRIPT_GROUP_DATA_VERSION_2 = 2,
			K_SCRIPT_DATA_VERSION_1 = 1,
			K_SCRIPT_DATA_VERSION_2 = 2,
			K_SCRIPT_OR_CONDITION_DATA_VERSION_1=1,
			K_SCRIPT_ACTION_VERSION_1 = 1,
			K_SCRIPT_ACTION_VERSION_2 = 2,
			K_SCRIPT_CONDITION_VERSION_1 = 1,
			K_SCRIPT_CONDITION_VERSION_2 = 2,
			K_SCRIPT_CONDITION_VERSION_3 = 3,
			K_SCRIPT_CONDITION_VERSION_4 = 4,
			K_SCRIPTS_DATA_VERSION_1,
			end_of_the_enumeration
};

static Condition::ConditionType ParameterChangesVer2[] =
{
	// Seven Changed from version 1.
	Condition::TEAM_INSIDE_AREA_PARTIALLY,
	Condition::TEAM_INSIDE_AREA_ENTIRELY,
	Condition::TEAM_OUTSIDE_AREA_ENTIRELY,
	Condition::TEAM_ENTERED_AREA_PARTIALLY,
	Condition::TEAM_ENTERED_AREA_ENTIRELY,
	Condition::TEAM_EXITED_AREA_ENTIRELY,
	Condition::TEAM_EXITED_AREA_PARTIALLY,
	(Condition::ConditionType) -1,
};

enum { AT_END = 0x00FFFFFF };

//-------------------------------------------------------------------------------------------------
// ******************************** class  ScriptList *********************************************
//-------------------------------------------------------------------------------------------------
// Statics ///////////////////////////////////////////////////////////////////////////////////////
ScriptList *ScriptList::s_readLists[MAX_PLAYER_COUNT] = {0};
Int					ScriptList::s_numInReadList = 0;

Int ScriptList::m_curId = 0;

/**
 ScriptList::updateDefaults -  checks for empty script lists, and adds some default stuff
 so you don't get a totally blank screen in the editor.
*/
void ScriptList::updateDefaults()
{
	Int i;
	for (i=0; i<TheSidesList->getNumSides(); i++)
	{
		ScriptList* pList = TheSidesList->getSideInfo(i)->getScriptList();
		if (pList == nullptr) {
			pList = newInstance(ScriptList);
			TheSidesList->getSideInfo(i)->setScriptList(pList);
		}
	}
}

/**
  Deletes any script lists attached to sides.  Used for editor cleanup.
*/
void ScriptList::reset()
{
	Int i;
	if (TheSidesList == nullptr) return; /// @todo - move this code into sides list.
	for (i=0; i<TheSidesList->getNumSides(); i++)
	{
		ScriptList* pList = TheSidesList->getSideInfo(i)->getScriptList();
		TheSidesList->getSideInfo(i)->setScriptList(nullptr);
		deleteInstance(pList);
	}
}



/**
  Ctor.
*/
ScriptList::ScriptList() :
m_firstGroup(nullptr),
m_firstScript(nullptr)
{
}

/**
  Dtor.  Deletes any script lists or group lists.  Note that dtors for groups and script lists
	delete the whole list, so don't need to traverse here.
*/
ScriptList::~ScriptList()
{
	deleteInstance(m_firstGroup);
	m_firstGroup = nullptr;

	deleteInstance(m_firstScript);
	m_firstScript = nullptr;
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void ScriptList::crc( Xfer *xfer )
{

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void ScriptList::xfer( Xfer *xfer )
{
	UnsignedShort countVerify;

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// count of scripts here
	Script *script;
	UnsignedShort scriptCount = 0;
	for( script = getScript(); script; script = script->getNext() )
		scriptCount++;
	countVerify = scriptCount;
	xfer->xferUnsignedShort( &scriptCount );
	if( countVerify != scriptCount )
	{

		DEBUG_CRASH(( "ScriptList::xfer - Script list count has changed, attempting to recover."));
		// throw SC_INVALID_DATA; try to recover. jba.

	}

	// all script data here
	for( script = getScript(); script; script = script->getNext() )	{
		xfer->xferSnapshot( script );
		scriptCount--;
		if (scriptCount==0) break;
	}
	if (scriptCount>0) {
		DEBUG_CRASH(("Stripping out extra scripts - Bad..."));
		if (s_mtScript==nullptr) s_mtScript = newInstance(Script);	// Yes it leaks, but this is unusual recovery only. jba.
		while (scriptCount) {
			xfer->xferSnapshot(s_mtScript);
			scriptCount--;
		}
	}

	// count of script groups
	ScriptGroup *scriptGroup;
	UnsignedShort scriptGroupCount = 0;
	for( scriptGroup = getScriptGroup(); scriptGroup; scriptGroup = scriptGroup->getNext() )
		scriptGroupCount++;
	countVerify = scriptGroupCount;
	xfer->xferUnsignedShort( &scriptGroupCount );
	if( countVerify != scriptGroupCount )
	{

		DEBUG_CRASH(( "ScriptList::xfer - Script group count has changed, attempting to recover."));

	}

	// all script group data
	for( scriptGroup = getScriptGroup(); scriptGroup; scriptGroup = scriptGroup->getNext() ) {
		xfer->xferSnapshot( scriptGroup );
		scriptGroupCount--;
		if (scriptGroupCount==0) break;
	}
	if (scriptGroupCount>0) {
		DEBUG_CRASH(("Stripping out extra groups. - Bad..."));
		if (s_mtGroup == nullptr) s_mtGroup = newInstance(ScriptGroup);	// Yes it leaks, but this is only for recovery.
		while (scriptGroupCount) {
			xfer->xferSnapshot(s_mtGroup);
			scriptGroupCount--;
		}
	}

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void ScriptList::loadPostProcess()
{

}

/**
  ScriptList::duplicate - Creates a full, "deep" copy of scriptlist.
*/
ScriptList *ScriptList::duplicate() const
{
	ScriptList *pNew = newInstance(ScriptList);

	{
		const ScriptGroup *src = this->m_firstGroup;
		ScriptGroup *dst = nullptr;
		while (src)
		{
			ScriptGroup *tmp = src->duplicate();

			if (dst)
				dst->setNextGroup(tmp);
			else
				pNew->m_firstGroup = tmp;

			src = src->getNext();
			dst = tmp;
		}
	}

	{
		const Script *src = this->m_firstScript;
		Script *dst = nullptr;
		while (src)
		{
			Script *tmp = src->duplicate();

			if (dst)
				dst->setNextScript(tmp);
			else
				pNew->m_firstScript = tmp;

			src = src->getNext();
			dst = tmp;
		}
	}

	return pNew;
}

/**
  ScriptList::duplicateAndQualify - Creates a full, "deep" copy of scriptlist,
	adding the qualifier to names.
*/
ScriptList *ScriptList::duplicateAndQualify(const AsciiString& qualifier,
			const AsciiString& playerTemplateName, const AsciiString& newPlayerName) const
{
	ScriptList *pNew = newInstance(ScriptList);

	{
		const ScriptGroup *src = this->m_firstGroup;
		ScriptGroup *dst = nullptr;
		while (src)
		{
			ScriptGroup *tmp = src->duplicateAndQualify( qualifier, playerTemplateName, newPlayerName);

			if (dst)
				dst->setNextGroup(tmp);
			else
				pNew->m_firstGroup = tmp;

			src = src->getNext();
			dst = tmp;
		}
	}

	{
		const Script *src = this->m_firstScript;
		Script *dst = nullptr;
		while (src)
		{
			Script *tmp = src->duplicateAndQualify(qualifier, playerTemplateName, newPlayerName);

			if (dst)
				dst->setNextScript(tmp);
			else
				pNew->m_firstScript = tmp;

			src = src->getNext();
			dst = tmp;
		}
	}

	return pNew;
}

/**
  ScriptList::discard - Deletes a script list, but not any children.
*/
void ScriptList::discard()
{
	m_firstGroup = nullptr;
	m_firstScript = nullptr;
	deleteInstance(this);
}

/**
  Add a script group to the current list of groups.  Offset to position ndx.
*/
void ScriptList::addGroup(ScriptGroup *pGrp, Int ndx)
{
	ScriptGroup *pPrev = nullptr;
	ScriptGroup *pCur = m_firstGroup;
	DEBUG_ASSERTCRASH(pGrp->getNext()==nullptr, ("Adding already linked group."));
	while (ndx && pCur) {
		pPrev = pCur;
		pCur = pCur->getNext();
		ndx--;
	}
	if (pPrev) {
		// Weave into prev link.
		pGrp->setNextGroup(pPrev->getNext());
		pPrev->setNextGroup(pGrp);
	} else {
		// Goes at head of list.
		pGrp->setNextGroup(m_firstGroup);
		m_firstGroup = pGrp;
	}
}

/**
  Add a script to the current list of scripts.  Offset to position ndx.
*/
void ScriptList::addScript(Script *pScr, Int ndx)
{
	Script *pPrev = nullptr;
	Script *pCur = m_firstScript;
	DEBUG_ASSERTCRASH(pScr->getNext()==nullptr, ("Adding already linked group."));
	while (ndx && pCur) {
		pPrev = pCur;
		pCur = pCur->getNext();
		ndx--;
	}
	if (pPrev) {
		pScr->setNextScript(pPrev->getNext());
		pPrev->setNextScript(pScr);
	} else {
		pScr->setNextScript(m_firstScript);
		m_firstScript = pScr;
	}
}

/**
  Delete a script from the current list of scripts.
*/
void ScriptList::deleteScript(Script *pScr)
{
	Script *pPrev = nullptr;
	Script *pCur = m_firstScript;
	while (pCur != pScr) {
		pPrev = pCur;
		pCur = pCur->getNext();
	}
	DEBUG_ASSERTCRASH(pCur, ("Couldn't find script."));
	if (pCur==nullptr) return;

	if (pPrev) {
		// unlink from previous script.
		pPrev->setNextScript(pCur->getNext());
	} else {
		// Unlink from head of list.
		m_firstScript = pCur->getNext();
	}
	// Clear the link & delete.
	pCur->setNextScript(nullptr);
	deleteInstance(pCur);
}

/**
  Delete a group from the current list of groups.
*/
void ScriptList::deleteGroup(ScriptGroup *pGrp)
{
	ScriptGroup *pPrev = nullptr;
	ScriptGroup *pCur = m_firstGroup;
	while (pCur != pGrp) {
		pPrev = pCur;
		pCur = pCur->getNext();
	}
	DEBUG_ASSERTCRASH(pCur, ("Couldn't find group."));
	if (pCur==nullptr) return;
	if (pPrev) {
		// unlink from previous group.
		pPrev->setNextGroup(pCur->getNext());
	} else {
		// Unlink from head of list.
		m_firstGroup = pCur->getNext();
	}
	// Clear the link & delete.
	pCur->setNextGroup(nullptr);
	deleteInstance(pCur);
}

/**
* ScriptList::ParseScriptsDataChunk - read a Scripts chunk.
* Format is the newer CHUNKY format.
*	See ScriptList::ScriptList for the writer.
*	Input: DataChunkInput
*
*/
Bool ScriptList::ParseScriptsDataChunk(DataChunkInput &file, DataChunkInfo *info, void *userData)
{
	Int i;
	file.registerParser( "ScriptList", info->label, ScriptList::ParseScriptListDataChunk );
	DEBUG_ASSERTCRASH(s_numInReadList==0, ("Leftover scripts floating around."));
	for (i=0; i<s_numInReadList; i++) {
		deleteInstance(s_readLists[i]);
		s_readLists[i] = nullptr;
	}
	TScriptListReadInfo readInfo;
	for (i=0; i<MAX_PLAYER_COUNT; i++) {
		readInfo.readLists[i] = nullptr;
	}
	readInfo.numLists = 0;
	if (file.parse(&readInfo)) {
		DEBUG_ASSERTCRASH(readInfo.numLists<MAX_PLAYER_COUNT, ("Read too many, overrun buffer."));
		s_numInReadList = readInfo.numLists;
		for (i=0; i<s_numInReadList; i++) {
			s_readLists[i] = readInfo.readLists[i];
		}
		return true;
	}
	return false;
}

/**
* ScriptList::getReadScripts - Gets the scripts read in from a file by .
* ScriptList::ParseScriptsDataChunk.
*
*/
Int ScriptList::getReadScripts(ScriptList *scriptLists[MAX_PLAYER_COUNT])
{
	Int i;
	Int count = s_numInReadList;
	s_numInReadList = 0;
	for (i=0; i<count; i++) {
		scriptLists[i] = s_readLists[i];
		s_readLists[i] = nullptr;
	}
	return count;
}

/**
* ScriptList::WriteScriptsDataChunk - Writes a Scripts chunk.
* Format is the newer CHUNKY format.
*	See ScriptEngine::ParseScriptsDataChunk for the reader.
*	Input: DataChunkInput
*
*/
void ScriptList::WriteScriptsDataChunk(DataChunkOutput &chunkWriter, ScriptList *scriptLists[], Int numLists )
{
	/**********SCRIPTS DATA ***********************/
	chunkWriter.openDataChunk("PlayerScriptsList", K_SCRIPTS_DATA_VERSION_1);
		Int i;
		for (i=0; i<numLists; i++) {
			chunkWriter.openDataChunk("ScriptList", K_SCRIPT_LIST_DATA_VERSION_1);
			if (scriptLists[i]) scriptLists[i]->WriteScriptListDataChunk(chunkWriter);
			chunkWriter.closeDataChunk();
		}
	chunkWriter.closeDataChunk();

}



/**
* ScriptList::WriteScriptListDataChunk - Writes a Scripts chunk.
* Format is the newer CHUNKY format.
*	Input: DataChunkInput
*
*/
void ScriptList::WriteScriptListDataChunk(DataChunkOutput &chunkWriter)
{
	/**********SCRIPTS DATA ***********************/
		if (m_firstScript) m_firstScript->WriteScriptDataChunk(chunkWriter, m_firstScript);
		if (m_firstGroup) m_firstGroup->WriteGroupDataChunk(chunkWriter, m_firstGroup);
}


/**
* ScriptList::ParseScriptListDataChunk - read a Scripts chunk.
* Format is the newer CHUNKY format.
*	See ScriptList::WriteScriptListDataChunk for the writer.
*	Input: DataChunkInput
*
*/
Bool ScriptList::ParseScriptListDataChunk(DataChunkInput &file, DataChunkInfo *info, void *userData)
{
	TScriptListReadInfo *pInfo = (TScriptListReadInfo*)userData;
	DEBUG_ASSERTCRASH(pInfo->numLists < MAX_PLAYER_COUNT, ("Too many."));
	if (pInfo->numLists >= MAX_PLAYER_COUNT) return false;
	pInfo->readLists[pInfo->numLists] = newInstance(ScriptList);
	Int cur = pInfo->numLists;
	pInfo->numLists++;
	file.registerParser( "Script", info->label, Script::ParseScriptFromListDataChunk );
	file.registerParser( "ScriptGroup", info->label, ScriptGroup::ParseGroupDataChunk );
	return file.parse(pInfo->readLists[cur]);

}


//-------------------------------------------------------------------------------------------------
// ******************************** class  ScriptGroup *********************************************
//-------------------------------------------------------------------------------------------------

/**
  Ctor - gives it a default name.
*/
ScriptGroup::ScriptGroup() :
m_firstScript(nullptr),
m_hasWarnings(false),
m_isGroupActive(true),
m_isGroupSubroutine(false),
m_nextGroup(nullptr)
{
	m_groupName.format("Script Group %d", ScriptList::getNextID());
}

/**
  Dtor - The script list deletes the rest of the list, but we have to loop & delete
	sll the script groups in out list.
*/
ScriptGroup::~ScriptGroup()
{
	// Delete the first script.  m_firstScript deletes the entire list.
	deleteInstance(m_firstScript);
	m_firstScript = nullptr;

	if (m_nextGroup) {
		// Delete all the subsequent groups in our list.
		ScriptGroup *cur = m_nextGroup;
		ScriptGroup *next;
		while (cur) {
			next = cur->getNext();
			cur->setNextGroup(nullptr); // prevents recursion.
			deleteInstance(cur);
			cur = next;
		}
	}
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void ScriptGroup::crc( Xfer *xfer )
{

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version
	* 2: m_isGroupActive, since it is twiddled by other scripts.  Only its initial state is determined by the map.
*/
// ------------------------------------------------------------------------------------------------
void ScriptGroup::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 2;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	if( version >= 2 )
		xfer->xferBool(&m_isGroupActive);

	// count of scripts here
	UnsignedShort scriptCount = 0;
	Script *script;
	for( script = getScript(); script; script = script->getNext() )
		scriptCount++;
	UnsignedShort countVerify = scriptCount;
	xfer->xferUnsignedShort( &scriptCount );
	if( countVerify != scriptCount )
	{

		DEBUG_CRASH(( "ScriptGroup::xfer - Script list count has changed, attempting to recover."));
		// throw SC_INVALID_DATA; try to recover. jba.

	}

	// xfer script data
	for( script = getScript(); script; script = script->getNext() )	{
		xfer->xferSnapshot( script );
		scriptCount--;
		if (scriptCount==0) break;
	}
	if (scriptCount>0) {
		DEBUG_CRASH(("Stripping out extra scripts - Bad..."));
		if (s_mtScript==nullptr) s_mtScript = newInstance(Script);	// Yes it leaks, but this is unusual recovery only. jba.
		while (scriptCount) {
			xfer->xferSnapshot(s_mtScript);
			scriptCount--;
		}
	}

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void ScriptGroup::loadPostProcess()
{

}

/**
  ScriptGroup::duplicate - Creates a full, "deep" copy of ScriptGroup.
	m_nextGroup is nullptr on the copy.
*/
ScriptGroup *ScriptGroup::duplicate() const
{
	ScriptGroup *pNew = newInstance(ScriptGroup);

	{
		Script *src = this->m_firstScript;
		Script *dst = nullptr;
		while (src)
		{
			Script *tmp = src->duplicate();

			if (dst)
				dst->setNextScript(tmp);
			else
				pNew->m_firstScript = tmp;

			src = src->getNext();
			dst = tmp;
		}
	}

	pNew->m_groupName = this->m_groupName;
	pNew->m_isGroupActive = this->m_isGroupActive;
	pNew->m_isGroupSubroutine = this->m_isGroupSubroutine;
	pNew->m_nextGroup = nullptr;

	return pNew;
}

/**
  ScriptGroup::duplicateAndQualify - Creates a full, "deep" copy of ScriptGroup,
	adding qualifier to names.
	m_nextGroup is nullptr on the copy.
*/
ScriptGroup *ScriptGroup::duplicateAndQualify(const AsciiString& qualifier,
			const AsciiString& playerTemplateName, const AsciiString& newPlayerName) const
{
	ScriptGroup *pNew = newInstance(ScriptGroup);

	{
		Script *src = this->m_firstScript;
		Script *dst = nullptr;
		while (src)
		{
			Script *tmp = src->duplicateAndQualify(qualifier, playerTemplateName, newPlayerName);

			if (dst)
				dst->setNextScript(tmp);
			else
				pNew->m_firstScript = tmp;

			src = src->getNext();
			dst = tmp;
		}
	}

	pNew->m_groupName = this->m_groupName;
	pNew->m_groupName.concat(qualifier);
	pNew->m_isGroupActive = this->m_isGroupActive;
	pNew->m_isGroupSubroutine = this->m_isGroupSubroutine;
	pNew->m_nextGroup = nullptr;

	return pNew;
}

/**
  Delete a script from the current list of scripts.
*/
void ScriptGroup::deleteScript(Script *pScr)
{
	Script *pPrev = nullptr;
	Script *pCur = m_firstScript;
	while (pScr != pCur) {
		pPrev = pCur;
		pCur = pCur->getNext();
	}
	DEBUG_ASSERTCRASH(pCur, ("Couldn't find script."));
	if (pCur==nullptr) return;
	if (pPrev) {
		pPrev->setNextScript(pCur->getNext());
	} else {
		m_firstScript = pCur->getNext();
	}
	// Clear link & delete.
	pCur->setNextScript(nullptr);
	deleteInstance(pCur);
}

/**
  Add a script to the current list of scripts.  Offset to position ndx.
*/
void ScriptGroup::addScript(Script *pScr, Int ndx)
{
	Script *pPrev = nullptr;
	Script *pCur = m_firstScript;
	DEBUG_ASSERTCRASH(pScr->getNext()==nullptr, ("Adding already linked group."));
	while (ndx && pCur) {
		pPrev = pCur;
		pCur = pCur->getNext();
		ndx--;
	}
	if (pPrev) {
		// link to pPrev
		pScr->setNextScript(pPrev->getNext());
		pPrev->setNextScript(pScr);
	} else {
		// add to head of list.
		pScr->setNextScript(m_firstScript);
		m_firstScript = pScr;
	}
}

/**
* ScriptGroup::WriteGroupDataChunk - Writes a Scripts chunk.
* Format is the newer CHUNKY format.
*	Input: DataChunkInput
*
*/
void ScriptGroup::WriteGroupDataChunk(DataChunkOutput &chunkWriter, ScriptGroup *pGroup)
{

	/**********SCRIPT GROUP DATA ***********************/
	while (pGroup) {
		chunkWriter.openDataChunk("ScriptGroup", K_SCRIPT_GROUP_DATA_VERSION_2);
			chunkWriter.writeAsciiString(pGroup->m_groupName);
			chunkWriter.writeByte(pGroup->m_isGroupActive);
			chunkWriter.writeByte(pGroup->m_isGroupSubroutine);
			if (pGroup->m_firstScript) Script::WriteScriptDataChunk(chunkWriter, pGroup->m_firstScript);
		chunkWriter.closeDataChunk();
		pGroup = pGroup->getNext();
	}

}

/**
* ScriptGroup::ParseGroupDataChunk - read a Group chunk.
* Format is the newer CHUNKY format.
*	See ScriptList::WriteScriptListDataChunk for the writer.
*	Input: DataChunkInput
*
*/
Bool ScriptGroup::ParseGroupDataChunk(DataChunkInput &file, DataChunkInfo *info, void *userData)
{
	ScriptList *pList = (ScriptList *)userData;
	ScriptGroup *pGroup = newInstance(ScriptGroup);

	pGroup->m_groupName = file.readAsciiString();
	pGroup->m_isGroupActive = file.readByte();
	if (info->version == K_SCRIPT_GROUP_DATA_VERSION_2) {
		pGroup->m_isGroupSubroutine= file.readByte();
	}
	pList->addGroup(pGroup, AT_END);
	file.registerParser( "Script", info->label, Script::ParseScriptFromGroupDataChunk );
	return file.parse(pGroup);

}

//-------------------------------------------------------------------------------------------------
// ******************************** class  Script *********************************************
//-------------------------------------------------------------------------------------------------
/**
  Ctor - initializes members.
*/
Script::Script() :
m_isActive(true),
m_isOneShot(true),
m_easy(true),
m_normal(true),
m_hard(true),
m_delayEvaluationSeconds(0),
m_conditionTime(0),
m_conditionExecutedCount(0),
m_frameToEvaluateAt(0),
m_isSubroutine(false),
m_hasWarnings(false),
m_nextScript(nullptr),
m_condition(nullptr),
m_action(nullptr),
m_actionFalse(nullptr),
m_curTime(0.0f)
{
}

/**
  Dtor - The condition and action deletes the rest of the list, but we have to loop & delete
	all the scripts in out list.
*/
Script::~Script()
{
	if (m_nextScript) {
		Script *cur = m_nextScript;
		Script *next;
		while (cur) {
			next = cur->getNext();
			cur->setNextScript(nullptr); // prevents recursion.
			deleteInstance(cur);
			cur = next;
		}
	}

	deleteInstance(m_condition);
	deleteInstance(m_action);
	deleteInstance(m_actionFalse);
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void Script::crc( Xfer *xfer )
{

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version
	* 2: TheSuperHackers @bugfix bobtista 13/08/2026 Save the next evaluation frame of delayed scripts
	*/
// ------------------------------------------------------------------------------------------------
void Script::xfer( Xfer *xfer )
{

	// version
#if RETAIL_COMPATIBLE_XFER_SAVE
	XferVersion currentVersion = 1;
#else
	XferVersion currentVersion = 2;
#endif
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// active
	Bool active = isActive();
	xfer->xferBool( &active );
	setActive( active );

	if( version >= 2 )
	{
		xfer->xferUnsignedInt( &m_frameToEvaluateAt );
	}

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void Script::loadPostProcess()
{

}

/**
  Script::duplicate - Creates a full, "deep" copy of script. Condition list and action
  list is duplicated as well.  Note - just the script, doesn't
	duplicate a list of scripts.  m_nextScript is nullptr on the copy.
*/
Script *Script::duplicate() const
{
	Script *pNew = newInstance(Script);

	pNew->m_scriptName = m_scriptName;
	pNew->m_comment = m_comment;
	pNew->m_conditionComment = m_conditionComment;
	pNew->m_actionComment = m_actionComment;
	pNew->m_isActive = m_isActive;
	pNew->m_isOneShot = m_isOneShot;
	pNew->m_isSubroutine = m_isSubroutine;
	pNew->m_easy = m_easy;
	pNew->m_normal = m_normal;
	pNew->m_hard = m_hard;
	pNew->m_delayEvaluationSeconds = m_delayEvaluationSeconds;

	if (m_condition) {
		pNew->m_condition = m_condition->duplicate();
	}
	if (m_action) {
		pNew->m_action = m_action->duplicate();
	}
	if (m_actionFalse) {
		pNew->m_actionFalse = m_actionFalse->duplicate();
	}
	return pNew;
}

/**
  Script::duplicate - Creates a full, "deep" copy of script, with qualifier
  added to names. Condition list and action
  list is duplicated as well.  Note - just the script, doesn't
	duplicate a list of scripts.  m_nextScript is nullptr on the copy.
*/
Script *Script::duplicateAndQualify(const AsciiString& qualifier,
			const AsciiString& playerTemplateName, const AsciiString& newPlayerName) const
{
	Script *pNew = newInstance(Script);

	pNew->m_scriptName = m_scriptName;
	pNew->m_scriptName.concat(qualifier);
	pNew->m_comment = m_comment;
	pNew->m_conditionComment = m_conditionComment;
	pNew->m_actionComment = m_actionComment;
	pNew->m_isActive = m_isActive;
	pNew->m_isOneShot = m_isOneShot;
	pNew->m_isSubroutine = m_isSubroutine;
	pNew->m_easy = m_easy;
	pNew->m_normal = m_normal;
	pNew->m_hard = m_hard;
	pNew->m_delayEvaluationSeconds = m_delayEvaluationSeconds;

	if (m_condition) {
		pNew->m_condition = m_condition->duplicateAndQualify(qualifier, playerTemplateName, newPlayerName);
	}
	if (m_action) {
		pNew->m_action = m_action->duplicateAndQualify(qualifier, playerTemplateName, newPlayerName);
	}
	if (m_actionFalse) {
		pNew->m_actionFalse = m_actionFalse->duplicateAndQualify(qualifier, playerTemplateName, newPlayerName);
	}
	return pNew;
}

/**
  Script::updateFrom - Copies all the data from pSrc into this.  Any data in this
	is deleted (conditions, actions).  Note that this guts pSrc, and removes it's conditions
	and actions.  Intended for use in an edit dialog, where pSrc is a copy edited, and if cancelled
	discarded, and if not cancelled, updated into the real script, then discarded.
*/
void Script::updateFrom(Script *pSrc)
{
	this->m_scriptName = pSrc->m_scriptName;
	this->m_comment = pSrc->m_comment;
	this->m_conditionComment = pSrc->m_conditionComment;
	this->m_actionComment = pSrc->m_actionComment;
	this->m_isActive = pSrc->m_isActive;
	this->m_isSubroutine = pSrc->m_isSubroutine;
	this->m_delayEvaluationSeconds = pSrc->m_delayEvaluationSeconds;
	this->m_isOneShot = pSrc->m_isOneShot;
	this->m_easy = pSrc->m_easy;
	this->m_normal = pSrc->m_normal;
	this->m_hard = pSrc->m_hard;

	deleteInstance(this->m_condition);
	this->m_condition = pSrc->m_condition;
	pSrc->m_condition = nullptr;

	deleteInstance(this->m_action);
	this->m_action = pSrc->m_action;
	pSrc->m_action = nullptr;

	deleteInstance(this->m_actionFalse);
	this->m_actionFalse = pSrc->m_actionFalse;
	pSrc->m_actionFalse = nullptr;
}

/**
  Script::deleteOrCondition - delete pCond from the or condition list.
*/
void Script::deleteOrCondition(OrCondition *pCond)
{
	OrCondition *pPrev = nullptr;
	OrCondition *pCur = m_condition;
	while (pCond != pCur) {
		pPrev = pCur;
		pCur = pCur->getNextOrCondition();
	}
	DEBUG_ASSERTCRASH(pCur, ("Couldn't find condition."));
	if (pCur==nullptr) return;
	if (pPrev) {
		pPrev->setNextOrCondition(pCur->getNextOrCondition());
	} else {
		m_condition = pCur->getNextOrCondition();
	}
	pCur->setNextOrCondition(nullptr);
	deleteInstance(pCur);
}


/**
  Script::deleteAction - delete pAct from the action list.
*/
void Script::deleteAction(ScriptAction *pAct)
{
	ScriptAction *pPrev = nullptr;
	ScriptAction *pCur = m_action;
	while (pAct != pCur) {
		pPrev = pCur;
		pCur = pCur->getNext();
	}
	DEBUG_ASSERTCRASH(pCur, ("Couldn't find action."));
	if (pCur==nullptr) return;
	if (pPrev) {
		pPrev->setNextAction(pCur->getNext());
	} else {
		m_action = pCur->getNext();
	}
	pCur->setNextAction(nullptr);
	deleteInstance(pCur);
}


/**
  Script::deleteFalseAction - delete pAct from the false action list.
*/
void Script::deleteFalseAction(ScriptAction *pAct)
{
	ScriptAction *pPrev = nullptr;
	ScriptAction *pCur = m_actionFalse;
	while (pAct != pCur) {
		pPrev = pCur;
		pCur = pCur->getNext();
	}
	DEBUG_ASSERTCRASH(pCur, ("Couldn't find action."));
	if (pCur==nullptr) return;
	if (pPrev) {
		pPrev->setNextAction(pCur->getNext());
	} else {
		m_actionFalse = pCur->getNext();
	}
	pCur->setNextAction(nullptr);
	deleteInstance(pCur);
}


/**
  Script::getUiText - Creates the string to display in the scripts dialog box.
*/
AsciiString Script::getUiText()
{
	AsciiString uiText("*** IF ***\r\n");
	OrCondition *pOr = m_condition;
	Int count=0;

	while (pOr) {
		Condition *pCond = pOr->getFirstAndCondition();
		if (count>0) uiText.concat("  *** OR ***\r\n");
		count = 0;
		while (pCond) {
			if (count>0) {
				uiText.concat("    *AND* ");
			} else {
				uiText.concat("    ");
			}
			uiText.concat(pCond->getUiText());
			uiText.concat("\r\n");
			pCond = pCond->getNext();
			count++;
		}
		pOr = pOr->getNextOrCondition();
	}
	uiText.concat("*** THEN ***\r\n");
	ScriptAction *pAction = m_action;
	while (pAction) {
		uiText.concat("  ");
		uiText.concat(pAction->getUiText());
		uiText.concat("\r\n");
		pAction = pAction->getNext();
	}
	pAction = m_actionFalse;
	if (pAction) {
		uiText.concat("*** ELSE ***\r\n");
		while (pAction) {
			uiText.concat("  ");
			uiText.concat(pAction->getUiText());
			uiText.concat("\r\n");
			pAction = pAction->getNext();
		}
	}
	return uiText;
}

/**
* Script::WriteScriptDataChunk - Writes a Scripts chunk.
* Format is the newer CHUNKY format.
*	Input: DataChunkInput
*
*/
void Script::WriteScriptDataChunk(DataChunkOutput &chunkWriter, Script *pScript)
{
	/**********SCRIPT  DATA ***********************/
	while (pScript) {
		chunkWriter.openDataChunk("Script", K_SCRIPT_DATA_VERSION_2);
			chunkWriter.writeAsciiString(pScript->m_scriptName);
			chunkWriter.writeAsciiString(pScript->m_comment);
			chunkWriter.writeAsciiString(pScript->m_conditionComment);
			chunkWriter.writeAsciiString(pScript->m_actionComment);

			chunkWriter.writeByte(pScript->m_isActive);
			chunkWriter.writeByte(pScript->m_isOneShot);
			chunkWriter.writeByte(pScript->m_easy);
			chunkWriter.writeByte(pScript->m_normal);
			chunkWriter.writeByte(pScript->m_hard);
			chunkWriter.writeByte(pScript->m_isSubroutine);
			chunkWriter.writeInt(pScript->m_delayEvaluationSeconds);
			if (pScript->m_condition) OrCondition::WriteOrConditionDataChunk(chunkWriter, pScript->m_condition);
			if (pScript->m_action) ScriptAction::WriteActionDataChunk(chunkWriter, pScript->m_action);
			if (pScript->m_actionFalse) ScriptAction::WriteActionFalseDataChunk(chunkWriter, pScript->m_actionFalse);
		chunkWriter.closeDataChunk();
		pScript = pScript->getNext();
	}
}
/**
* Script::ParseScript - read a script chunk.
* Format is the newer CHUNKY format.
*	See ScriptList::WriteScriptDataChunk for the writer.
*	Input: DataChunkInput
*
*/
Script *Script::ParseScript(DataChunkInput &file, unsigned short version)
{
	Script *pScript = newInstance(Script);

	pScript->m_scriptName = file.readAsciiString();
	pScript->m_comment = file.readAsciiString();
	pScript->m_conditionComment = file.readAsciiString();
	pScript->m_actionComment = file.readAsciiString();

	pScript->m_isActive = file.readByte();
	pScript->m_isOneShot = file.readByte();
	pScript->m_easy = file.readByte();
	pScript->m_normal = file.readByte();
	pScript->m_hard = file.readByte();
	pScript->m_isSubroutine = file.readByte();
	if (version>=K_SCRIPT_DATA_VERSION_2) {
		pScript->m_delayEvaluationSeconds = file.readInt();
	}
	file.registerParser( "OrCondition", "Script", OrCondition::ParseOrConditionDataChunk );
	file.registerParser( "ScriptAction",  "Script", ScriptAction::ParseActionDataChunk );
	file.registerParser( "ScriptActionFalse",  "Script", ScriptAction::ParseActionFalseDataChunk );
	if (! file.parse(pScript) )
	{
		return nullptr;
	}
	DEBUG_ASSERTCRASH(file.atEndOfChunk(), ("Unexpected data left over."));
	return pScript;
}

/**
* Script::ParseScriptFromListDataChunk - read a script chunk in a script list.
* Format is the newer CHUNKY format.
*	See ScriptList::WriteScriptListDataChunk for the writer.
*	Input: DataChunkInput
*
*/
Bool Script::ParseScriptFromListDataChunk(DataChunkInput &file, DataChunkInfo *info, void *userData)
{
	ScriptList *pList = (ScriptList *)userData;
	Script *pScript = ParseScript(file, info->version);
	pList->addScript(pScript, AT_END);
	DEBUG_ASSERTCRASH(file.atEndOfChunk(), ("Unexpected data left over."));
	return true;
}

/**
* Script::ParseScriptFromGroupDataChunk - read a script chunk in a script group.
* Format is the newer CHUNKY format.
*	See ScriptList::WriteScriptListDataChunk for the writer.
*	Input: DataChunkInput
*
*/
Bool Script::ParseScriptFromGroupDataChunk(DataChunkInput &file, DataChunkInfo *info, void *userData)
{
	ScriptGroup *pGroup = (ScriptGroup *)userData;
	Script *pScript = ParseScript(file, info->version);
	pGroup->addScript(pScript, AT_END);
	DEBUG_ASSERTCRASH(file.atEndOfChunk(), ("Unexpected data left over."));
	return true;
}


/**
* Script::findPreviousOrCondition - find the OrCondition that immediately proceeds curOr.
*	Input: OrCondition
*
*/
OrCondition *Script::findPreviousOrCondition( OrCondition *curOr )
{
	OrCondition *myConditions = getOrCondition();
	if ( myConditions == curOr ) {
		return nullptr;
	}

	while (myConditions) {
		if (myConditions->getNextOrCondition() == curOr) {
			return myConditions;
		}
		myConditions = myConditions->getNextOrCondition();
	}

	DEBUG_CRASH(("Tried to find an OrCondition that doesn't seem to exist (jkmcd)"));
	return nullptr;
}

//-------------------------------------------------------------------------------------------------
// ******************************** class  OrCondition *********************************************
//-------------------------------------------------------------------------------------------------
OrCondition::~OrCondition()
{
	deleteInstance(m_firstAnd);
	m_firstAnd = nullptr;

	if (m_nextOr) {
		OrCondition *cur = m_nextOr;
		OrCondition *next;
		while (cur) {
			next = cur->getNextOrCondition();
			cur->setNextOrCondition(nullptr); // prevents recursion.
			deleteInstance(cur);
			cur = next;
		}
	}
}

OrCondition *OrCondition::duplicate() const
{
	OrCondition *pNew = newInstance(OrCondition);
	if (m_firstAnd) {
		pNew->m_firstAnd = m_firstAnd->duplicate();
	}
	OrCondition *pLink = m_nextOr;
	OrCondition *pCur = pNew;
	while (pLink) {
		pCur->m_nextOr = newInstance(OrCondition);
		pCur = pCur->m_nextOr;
		if (pLink->m_firstAnd) {
			pCur->m_firstAnd = pLink->m_firstAnd->duplicate();
		}
		pLink = pLink->m_nextOr;
	}
	return pNew;
}

OrCondition *OrCondition::duplicateAndQualify(const AsciiString& qualifier,
			const AsciiString& playerTemplateName, const AsciiString& newPlayerName) const
{
	OrCondition *pNew = newInstance(OrCondition);
	if (m_firstAnd) {
		pNew->m_firstAnd = m_firstAnd->duplicateAndQualify(qualifier, playerTemplateName, newPlayerName);
	}
	OrCondition *pLink = m_nextOr;
	OrCondition *pCur = pNew;
	while (pLink) {
		pCur->m_nextOr = newInstance(OrCondition);
		pCur = pCur->m_nextOr;
		if (pLink->m_firstAnd) {
			pCur->m_firstAnd = pLink->m_firstAnd->duplicateAndQualify(qualifier, playerTemplateName, newPlayerName);
		}
		pLink = pLink->m_nextOr;
	}
	return pNew;
}

Condition *OrCondition::removeCondition(Condition *pCond)
{
	Condition *pPrev = nullptr;
	Condition *pCur = m_firstAnd;
	while (pCond != pCur) {
		pPrev = pCur;
		pCur = pCur->getNext();
	}

	DEBUG_ASSERTCRASH(pCur, ("Couldn't find condition."));
	if (pCur==nullptr)
		return nullptr;
	if (pPrev) {
		pPrev->setNextCondition(pCur->getNext());
	} else {
		m_firstAnd = pCur->getNext();
	}

	pCur->setNextCondition(nullptr);
	return pCur;
}

void OrCondition::deleteCondition(Condition *pCond)
{
	Condition *pCur = removeCondition(pCond);
	DEBUG_ASSERTCRASH(pCur, ("Couldn't find condition."));
	if (pCur==nullptr)
		return;
	deleteInstance(pCur);
}


/**
* OrCondition::WriteOrConditionDataChunk - Writes a Or condition chunk.
* Format is the newer CHUNKY format.
*	Input: DataChunkInput
*
*/
void OrCondition::WriteOrConditionDataChunk(DataChunkOutput &chunkWriter, OrCondition	*pOrCondition)
{
	/**********OR CONDITION DATA ***********************/
	while (pOrCondition) {
		chunkWriter.openDataChunk("OrCondition", K_SCRIPT_OR_CONDITION_DATA_VERSION_1);
		if (pOrCondition->m_firstAnd) Condition::WriteConditionDataChunk(chunkWriter, pOrCondition->m_firstAnd);
		chunkWriter.closeDataChunk();
		pOrCondition = pOrCondition->getNextOrCondition();
	}

}

/**
* OrCondition::ParseOrConditionDataChunk - read a Or condition chunk.
* Format is the newer CHUNKY format.
*	See OrCondition::WriteOrConditionDataChunk for the writer.
*	Input: DataChunkInput
*
*/
Bool OrCondition::ParseOrConditionDataChunk(DataChunkInput &file, DataChunkInfo *info, void *userData)
{
	Script *pScript = (Script *)userData;
	OrCondition *pOrCondition = newInstance(OrCondition);
	OrCondition *pFirst = pScript->getOrCondition();
	while (pFirst && pFirst->getNextOrCondition()) {
		pFirst = pFirst->getNextOrCondition();
	}
	if (pFirst) {
		pFirst->setNextOrCondition(pOrCondition);
	} else {
		pScript->setOrCondition(pOrCondition);
	}
	file.registerParser( "Condition", info->label, Condition::ParseConditionDataChunk );
	return file.parse(pOrCondition);

}

/**
* OrCondition::findPreviousCondition - find the condition that immediately proceeds curCond.
* Format is the newer CHUNKY format.
*	See OrCondition::WriteOrConditionDataChunk for the writer.
*	Input: DataChunkInput
*
*/
Condition *OrCondition::findPreviousCondition( Condition *curCond )
{
	Condition *myConditions = getFirstAndCondition();
	if (myConditions == curCond) {
		return nullptr;
	}

	while (myConditions) {
		if (myConditions->getNext() == curCond) {
			return myConditions;
		}
		myConditions = myConditions->getNext();
	}

	DEBUG_CRASH(("Searched for non-existent And Condition. (jkmcd)"));
	return nullptr;
}


//-------------------------------------------------------------------------------------------------
// ******************************** class  Condition *********************************************
//-------------------------------------------------------------------------------------------------
Condition::Condition():
m_conditionType(CONDITION_FALSE),
m_hasWarnings(false),
m_customData(0),
m_customFrame(0),
m_numParms(0),
m_nextAndCondition(nullptr)
{
	Int i;
	for (i = 0; i < MAX_PARMS; i++)
	{
		m_parms[i] = nullptr;
	}
}

Condition::Condition(enum ConditionType type):
m_conditionType(type),
m_hasWarnings(false),
m_customData(0),
m_customFrame(0),
m_numParms(0),
m_nextAndCondition(nullptr)
{
	Int i;
	for (i=0; i<MAX_PARMS; i++) {
		m_parms[i] = nullptr;
	}
	setConditionType(type);
}

void Condition::setConditionType(enum ConditionType type)
{
	Int i;
	for (i=0; i<m_numParms; i++) {
		deleteInstance(m_parms[i]);
		m_parms[i] = nullptr;
	}
	m_conditionType = type;
	const ConditionTemplate *pTemplate = TheScriptEngine->getConditionTemplate(m_conditionType);
	m_numParms = pTemplate->getNumParameters();
	for (i=0; i<m_numParms; i++) {
		m_parms[i] = newInstance(Parameter)(pTemplate->getParameterType(i));
	}
}

Condition *Condition::duplicate() const
{
	Condition *pNew = newInstance(Condition)(m_conditionType);
	Int i;
	for (i=0; i<m_numParms && i<pNew->m_numParms; i++) {
		*pNew->m_parms[i] = *m_parms[i];
	}
	Condition *pLink = m_nextAndCondition;
	Condition *pCur = pNew;
	while (pLink) {
		pCur->m_nextAndCondition = newInstance(Condition)(pLink->getConditionType());
		pCur = pCur->m_nextAndCondition;
		for (i=0; i<pLink->m_numParms; i++) {
			*pCur->m_parms[i] = *pLink->m_parms[i];
		}
		pLink = pLink->m_nextAndCondition;
	}
	return pNew;
}

Condition *Condition::duplicateAndQualify(const AsciiString& qualifier,
			const AsciiString& playerTemplateName, const AsciiString& newPlayerName) const
{
	Condition *pNew = newInstance(Condition)(m_conditionType);
	Int i;
	for (i=0; i<m_numParms && i<pNew->m_numParms; i++) {
		*pNew->m_parms[i] = *m_parms[i];
		pNew->m_parms[i]->qualify(qualifier, playerTemplateName, newPlayerName);
	}
	Condition *pLink = m_nextAndCondition;
	Condition *pCur = pNew;
	while (pLink) {
		pCur->m_nextAndCondition = newInstance(Condition)(pLink->getConditionType());
		pCur = pCur->m_nextAndCondition;
		for (i=0; i<pLink->m_numParms; i++) {
			*pCur->m_parms[i] = *pLink->m_parms[i];
			pCur->m_parms[i]->qualify(qualifier, playerTemplateName, newPlayerName);
		}
		pLink = pLink->m_nextAndCondition;
	}
	return pNew;
}

Condition::~Condition()
{
	Int i;
	for (i=0; i<m_numParms; i++) {
		deleteInstance(m_parms[i]);
		m_parms[i] = nullptr;
	}
	if (m_nextAndCondition) {
		Condition *cur = m_nextAndCondition;
		Condition *next;
		while (cur) {
			next = cur->getNext();
			cur->setNextCondition(nullptr); // prevents recursion.
			deleteInstance(cur);
			cur = next;
		}
	}
}



Int Condition::getUiStrings(AsciiString strings[MAX_PARMS])
{
	const ConditionTemplate *pTemplate = TheScriptEngine->getConditionTemplate(m_conditionType);
	return pTemplate->getUiStrings(strings);
}

AsciiString Condition::getUiText()
{
	AsciiString uiText;
	AsciiString strings[MAX_PARMS];
	Int numStrings = getUiStrings(strings);
	Int i;

	if (m_hasWarnings) {
		uiText = "[???]";
	}

	for (i=0; i<MAX_PARMS; i++) {
		if (i<numStrings) {
			uiText.concat(strings[i]);
		}
		if (i<m_numParms) {
			uiText.concat(m_parms[i]->getUiText());
		}
	}

	return uiText;
}


/**
* Condition::WriteConditionDataChunk - Writes a condition chunk.
* Format is the newer CHUNKY format.
*	Input: DataChunkInput
*
*/
void Condition::WriteConditionDataChunk(DataChunkOutput &chunkWriter, Condition	*pCondition)
{
	/**********Condition  DATA ***********************/
	while (pCondition) {
		chunkWriter.openDataChunk("Condition", K_SCRIPT_CONDITION_VERSION_4);
			chunkWriter.writeInt(pCondition->m_conditionType);
			const ConditionTemplate* ct = TheScriptEngine->getConditionTemplate(pCondition->m_conditionType);
			if (ct) {
				chunkWriter.writeNameKey(ct->m_internalNameKey);
			}	else {
				DEBUG_CRASH(("Invalid condition."));
				chunkWriter.writeNameKey(NAMEKEY("Bogus"));
			}
			chunkWriter.writeInt(pCondition->m_numParms);
			Int i;
			for (i=0; i<pCondition->m_numParms; i++) {
				pCondition->m_parms[i]->WriteParameter(chunkWriter);
			}
		chunkWriter.closeDataChunk();
		pCondition = pCondition->getNext();
	}
}
/**
* Condition::ParseConditionDataChunk - read a condition.
* Format is the newer CHUNKY format.
*	See Condition::WriteActionDataChunk for the writer.
*	Input: DataChunkInput
*
*/
Bool Condition::ParseConditionDataChunk(DataChunkInput &file, DataChunkInfo *info, void *userData)
{
	Condition	*pCondition = newInstance(Condition);
	OrCondition *pOr = (OrCondition *)userData;
	pCondition->m_conditionType = (enum ConditionType)file.readInt();
	const ConditionTemplate* ct = TheScriptEngine->getConditionTemplate(pCondition->m_conditionType);
	if (info->version >= K_SCRIPT_CONDITION_VERSION_4) {
		NameKeyType key = file.readNameKey();
		Bool match = false;
		if (ct && ct->m_internalNameKey == key) {
			match = TRUE; // All good. jba. [3/20/2003]
		}
		if (!match) {
			//  name and id don't match.  Find the name [3/20/2003]
			Int i;
			for (i=0; i<Condition::NUM_ITEMS; i++) {
				ct = TheScriptEngine->getConditionTemplate(i);
				if (key == ct->m_internalNameKey) {
					match = true;
					DEBUG_LOG(("Rematching script condition %s", KEYNAME(key).str()));
					pCondition->m_conditionType = (enum ConditionType)i;
					break;
				}
			}
		}
		if (!match) {
			// Invalid script [3/20/2003]
			DEBUG_CRASH(("Invalid script condition.  Making it false. jba."));
			pCondition->m_conditionType = CONDITION_FALSE;
			pCondition->m_numParms = 0;
		}
	}
	pCondition->m_numParms =file.readInt();
	Int i;
	for (i=0; i<pCondition->m_numParms; i++)
	{
		pCondition->m_parms[i] = Parameter::ReadParameter(file);
	}

	if (file.getChunkVersion() < K_SCRIPT_CONDITION_VERSION_2) {
		for (int j = 0; ParameterChangesVer2[j] != -1; ++j) {
			if (pCondition->m_conditionType == ParameterChangesVer2[j]) {
				pCondition->m_parms[pCondition->m_numParms] = newInstance(Parameter)(Parameter::SURFACES_ALLOWED, 3);
				pCondition->m_numParms = 3;
			}
		}
	}
	// heal old files.
	switch (pCondition->getConditionType())
	{
		case SKIRMISH_SPECIAL_POWER_READY:
			if (pCondition->m_numParms == 1)
			{
				pCondition->m_numParms = 2;
				pCondition->m_parms[1] = pCondition->m_parms[0];
				pCondition->m_parms[0] = newInstance(Parameter)(Parameter::SIDE, 0);
				pCondition->m_parms[0]->friend_setString(THIS_PLAYER);
			}
			break;
	}
#ifdef COUNT_SCRIPT_USAGE
	const ConditionTemplate* conT = TheScriptEngine->getConditionTemplate(pCondition->m_conditionType);
	conT->m_numTimesUsed++;
	conT->m_firstMapUsed = TheGlobalData->m_mapName;
#endif
	if (ct->getNumParameters() != pCondition->getNumParameters()) {
		// Invalid script [3/20/2003]
		DEBUG_CRASH(("Invalid script condition.  Making it false. jba."));
		pCondition->m_conditionType = ConditionType::CONDITION_FALSE;
		pCondition->m_numParms = 0;
	}
	Condition *pLast = pOr->getFirstAndCondition();
	while (pLast && pLast->getNext()) {
		pLast = pLast->getNext();
	}
	if (pLast) {
		pLast->setNextCondition(pCondition);
	} else {
		pOr->setFirstAndCondition(pCondition);
	}
	DEBUG_ASSERTCRASH(file.atEndOfChunk(), ("Unexpected data left over."));
	return true;
}


//-------------------------------------------------------------------------------------------------
// ******************************** class  Template *********************************************
//-------------------------------------------------------------------------------------------------
Template::Template() :
m_numUiStrings(0),
m_numParameters(0),
#ifdef COUNT_SCRIPT_USAGE
m_numTimesUsed(0),
#endif
m_uiName("UNUSED/(placeholder)/placeholder")
{
}

Int Template::getUiStrings(AsciiString strings[MAX_PARMS]) const
{
	Int i;
	for (i=0; i<m_numUiStrings; i++) {
		strings[i] = m_uiStrings[i];
	}
	return m_numUiStrings;
}

//-------------------------------------------------------------------------------------------------
// ******************************** class Parameter ***********************************************
//-------------------------------------------------------------------------------------------------
enum Parameter::ParameterType Template::getParameterType(Int ndx) const
{
	if (ndx >= 0 && ndx < m_numParameters) {
		return m_parameters[ndx];
	}
	DEBUG_CRASH(("Index out of range."));
	return Parameter::INT;
}

void Parameter::getCoord3D(Coord3D *pLoc) const
{
	DEBUG_ASSERTCRASH(m_paramType==COORD3D, ("Wrong parameter type."));
	pLoc->x = pLoc->y = pLoc->z = 0;
	if (m_paramType==COORD3D) {
		*pLoc = m_coord;
	}
}

void Parameter::setCoord3D(const Coord3D *pLoc)
{
	DEBUG_ASSERTCRASH(m_paramType==COORD3D, ("Wrong parameter type."));
	if (m_paramType==COORD3D) {
		m_coord= *pLoc ;
	}
}

void Parameter::qualify(const AsciiString& qualifier,
			const AsciiString& playerTemplateName, const AsciiString& newPlayerName)
{
	AsciiString tmpString;
	switch (m_paramType) {
		case SIDE:
			tmpString = m_string;
			tmpString.concat(qualifier);
			if (tmpString==playerTemplateName) {
				m_string = newPlayerName;
			}
			break;
		case TEAM:
			if (m_string == THIS_TEAM) {
				break;
			}
			FALLTHROUGH; /// otherwise drop down & qualify.
		case SCRIPT:
		case COUNTER:
		case FLAG:
		case SCRIPT_SUBROUTINE: m_string.concat(qualifier); break;
		default: break;
	}
}

AsciiString Parameter::getUiText() const
{
	AsciiString uiText;
	AsciiString uiString = m_string;
	if (uiString.isEmpty()) {
		uiString = "???";
	}

	Coord3D pos;
	switch (m_paramType)
	{
		default:
			DEBUG_CRASH(("Unknown parameter type."));
			break;
		case SOUND:
			uiText.format("Sound '%s'", uiString.str());
			break;
		case SCRIPT:
			uiText.format("Script '%s'", uiString.str());
			break;
		case TEAM_STATE:
			uiText.format("'%s'", uiString.str());
			break;
		case SCRIPT_SUBROUTINE:
			uiText.format("Subroutine '%s'", uiString.str());
			break;
		case ATTACK_PRIORITY_SET:
			uiText.format("Attack priority set '%s'", uiString.str());
			break;
		case WAYPOINT:
			uiText.format("Waypoint '%s'", uiString.str());
			break;
		case WAYPOINT_PATH:
			uiText.format("Waypoint Path '%s'", uiString.str());
			break;
		case TRIGGER_AREA:
			uiText.format(" area '%s'", uiString.str());
			break;
		case COMMAND_BUTTON:
			uiText.format("Command button: '%s'", uiString.str());
			break;
		case FONT_NAME:
			uiText.format("Font: '%s'", uiString.str());
			break;
		case LOCALIZED_TEXT:
			uiText.format("Localized String: '%s'", uiString.str());
			break;
		case TEXT_STRING:
			uiText.format("String: '%s'", uiString.str());
			break;
		case TEAM:
			uiText.format("Team '%s'", uiString.str());
			break;
		case UNIT:
			uiText.format("Unit '%s'", uiString.str());
			break;
		case BRIDGE:
			uiText.format("Bridge '%s'", uiString.str());
			break;
		case ANGLE:
			uiText.format("%.2f degrees", m_real*180/PI);
			break;
		case PERCENT:
			uiText.format("%.2f%%", m_real*100.0f);
			break;
		case COORD3D:
			getCoord3D(&pos);
			uiText.format("(%.2f,%.2f,%.2f)", pos.x,pos.y,pos.z);
			break;
		case OBJECT_TYPE:
			uiText.format("'%s'", uiString.str());
			break;
		case KIND_OF_PARAM:
			if (m_int >= KINDOF_FIRST && m_int < KINDOF_COUNT )
				uiText.format("Kind is '%s'", KindOfMaskType::getNameFromSingleBit(m_int));
			else
				uiText.format("Kind is ???");
			break;
		case SIDE:
			uiText.format("Player '%s'", uiString.str());
			break;
		case COUNTER:
			uiText.format("'%s'", uiString.str());
			break;
		case INT:
			uiText.format(" %d ", m_int);
			break;
		case BOOLEAN:
			uiText.concat(m_int?"TRUE":"FALSE");
			break;

		case REAL:
			uiText.format("%.2f", m_real);
			break;

		case FLAG:
			uiText.format("Flag named '%s'", uiString.str());
			break;
		case COMPARISON:
			switch (m_int) {
				case LESS_THAN: uiText.format("Less Than"); break;
				case LESS_EQUAL: uiText.format("Less Than or Equal"); break;
				case EQUAL: uiText.format("Equal To"); break;
				case GREATER_EQUAL: uiText.format("Greater Than or Equal To"); break;
				case GREATER: uiText.format("Greater Than"); break;
				case NOT_EQUAL: uiText.format("Not Equal To"); break;
				default : DEBUG_CRASH(("Unknown comparison type."));
			}
			break;

		case RELATION:
			switch (m_int) {
				case REL_ENEMY: uiText.format("Enemy"); break;
				case REL_NEUTRAL: uiText.format("Neutral"); break;
				case REL_FRIEND: uiText.format("Friend"); break;
				default : DEBUG_CRASH(("Unknown Relation type."));
			}
			break;

		case AI_MOOD:
			switch (m_int) {
				case ATTITUDE_SLEEP: uiText.format("Sleep"); break;
				case ATTITUDE_PASSIVE: uiText.format("Passive"); break;
				case ATTITUDE_NORMAL: uiText.format("Normal"); break;
				case ATTITUDE_ALERT: uiText.format("Alert"); break;
				case ATTITUDE_AGGRESSIVE: uiText.format("Aggressive"); break;
				default : DEBUG_CRASH(("Unknown AI Mood type."));
			}
			break;

		case RADAR_EVENT_TYPE:
			switch (m_int) {
				//case RADAR_EVENT_INVALID: ++m_int;	// continue to the next case.
				case RADAR_EVENT_INVALID: DEBUG_CRASH(("Invalid radar event")); uiText.format("Construction"); break;
				case RADAR_EVENT_CONSTRUCTION: uiText.format("Construction"); break;
				case RADAR_EVENT_UPGRADE: uiText.format("Upgrade"); break;
				case RADAR_EVENT_UNDER_ATTACK: uiText.format("Under Attack"); break;
				case RADAR_EVENT_INFORMATION: uiText.format("Information"); break;
				case RADAR_EVENT_INFILTRATION: uiText.format("Infiltration"); break;
				default : DEBUG_CRASH(("Unknown Radar event type."));
			}
			break;

    case LEFT_OR_RIGHT:
      switch (m_int)
      {
        case EVAC_BURST_FROM_CENTER: uiText.format("normal (burst from center)"); break;
        case EVAC_TO_LEFT: uiText.format("left"); break;
        case EVAC_TO_RIGHT: uiText.format("right"); break;
        default :  uiText.format("unspecified"); break;
      }
      break;



		case DIALOG:
			uiText.format("'%s'", uiString.str());
			break;

		case SKIRMISH_WAYPOINT_PATH:
			uiText.format("'%s'", uiString.str());
			break;

		case COLOR:
			uiText.format(" R:%d G:%d B:%d ", (m_int&0x00ff0000)>>16, (m_int&0x0000ff00)>>8, (m_int&0x000000ff) );
			break;

		case MUSIC:
			uiText.format("'%s'", uiString.str());
			break;

		case MOVIE:
			uiText.format("'%s'", uiString.str());
			break;

		case SPECIAL_POWER:
			uiText.format("Special power '%s'", uiString.str());
			break;

		case SCIENCE:
			uiText.format("Science '%s'", uiString.str());
			break;

		case SCIENCE_AVAILABILITY:
			uiText.format( "Science availability '%s'", uiString.str() );
			break;

		case UPGRADE:
			uiText.format("Upgrade '%s'", uiString.str());
			break;

		case COMMANDBUTTON_ABILITY:
		case COMMANDBUTTON_ALL_ABILITIES:
			uiText.format( "Ability '%s'", uiString.str() );
			break;

		case EMOTICON:
			uiText.format( "Emoticon '%s'", uiString.str() );
			break;

		case BOUNDARY:
			uiText.format("Boundary %s", BORDER_COLORS[m_int % BORDER_COLORS_SIZE].m_colorName);
			break;

		case BUILDABLE:
			if (m_int >= BSTATUS_YES && m_int < BSTATUS_NUM_TYPES )
				uiText.format("Buildable (%s)", BuildableStatusNames[m_int - BSTATUS_YES]);
			else
				uiText.format("Buildable ???");
			break;

		case SURFACES_ALLOWED:
		{
			if (m_int > 0 && m_int <= 3)
				uiText.format("Surfaces Allowed: %s", Surfaces[m_int - 1]);
			else
				uiText.format("Surfaces Allowed: ???");
			break;
		}

		case SHAKE_INTENSITY:
		{
			if (m_int > 0 && m_int < View::SHAKE_COUNT)
				uiText.format("Shake Intensity: %s", ShakeIntensities[m_int]);
			else
				uiText.format("Shake Intensity: ???");
			break;
		}

		case OBJECT_STATUS:
		{
			if (m_string.isEmpty()) {
				uiText.format("Object Status is ???");
			} else {
				uiText.format("Object Status is '%s'", m_string.str());
			}
			break;
		}

		case FACTION_NAME:
		{
			uiText.format("Faction Name: %s", uiString.str());
			break;
		}

		case OBJECT_TYPE_LIST:
			uiText.format("'%s'", uiString.str());
			break;

		case REVEALNAME:
			uiText.format("Reveal Name: %s", uiString.str());
			break;

		case OBJECT_PANEL_FLAG:
			uiText.format("Object Flag: %s", uiString.str());
			break;
	}
	return uiText;
}

/**
* Parameter::WriteParameter - Writes an Parameter.
* Format is the newer CHUNKY format.
*	Input: DataChunkInput
*
*/
void Parameter::WriteParameter(DataChunkOutput &chunkWriter)
{

	/**********Parameter  DATA ***********************/
	chunkWriter.writeInt(m_paramType);
	if (m_paramType == KIND_OF_PARAM) {
		// To get the proper kindof string stored.
		m_string = KindOfMaskType::getNameFromSingleBit(m_int);
	}
	if (m_paramType == COORD3D) {
		chunkWriter.writeReal(m_coord.x);
		chunkWriter.writeReal(m_coord.y);
		chunkWriter.writeReal(m_coord.z);
	} else {
		chunkWriter.writeInt(m_int);
		chunkWriter.writeReal(m_real);
		chunkWriter.writeAsciiString(m_string);
	}
}

/**
* Parameter::ReadParameter - read a parameter.
* Format is the newer CHUNKY format.
*	See Parameter::WriteParameter for the writer.
*	Input: DataChunkInput
*
*/
Parameter *Parameter::ReadParameter(DataChunkInput &file)
{
	Parameter *pParm = newInstance(Parameter)( (ParameterType)file.readInt());
	pParm->m_initialized = true;
	if (pParm->getParameterType() == COORD3D) {
		Coord3D pos;
		pos.x = file.readReal();
		pos.y = file.readReal();
		pos.z = file.readReal();
		pParm->setCoord3D(&pos);
	}
	else
	{
		pParm->m_int = file.readInt();
		pParm->m_real = file.readReal();
		pParm->m_string = file.readAsciiString();
	}

	if (pParm->getParameterType() == OBJECT_TYPE)
	{
		// quick hack to make loading models with "Fundamentalist" switch to "GLA"
		if (pParm->m_string.startsWith("Fundamentalist"))
		{
			const char* replacePrefix = "Fundamentalist";
			const size_t offset = pParm->m_string.startsWith(replacePrefix) ? strlen(replacePrefix) : 0u;
			char newName[256];
			strcpy(newName, "GLA");
			strlcat(newName, pParm->m_string.str() + offset, ARRAY_SIZE(newName));
			DEBUG_LOG(("Changing Script Ref from %s to %s", pParm->m_string.str(), newName));
			pParm->m_string.set(newName);
		}
	}

	if (pParm->getParameterType() == UPGRADE)
	{
		// quick hack to make obsolete capture building upgrades switch to the new one. jba.
		if (pParm->m_string == "Upgrade_AmericaRangerCaptureBuilding" ||
			pParm->m_string == "Upgrade_ChinaRedguardCaptureBuilding" ||
			pParm->m_string == "Upgrade_GLARebelCaptureBuilding")
		{
			pParm->m_string.set("Upgrade_InfantryCaptureBuilding");
		}
	}

	if (pParm->getParameterType() == OBJECT_STATUS)
	{
		// Need to change the string to an ObjectStatusMaskType
		for( int i = 0; i < OBJECT_STATUS_COUNT; ++i )
		{
			if( !pParm->m_string.compareNoCase( ObjectStatusMaskType::getBitNames()[i] ) )
			{
				pParm->setStatus( MAKE_OBJECT_STATUS_MASK( i ) );
				break;
			}
		}
	}

	if (pParm->getParameterType() == KIND_OF_PARAM)
  {
		// Need to change the string to an integer
		const char* const* kindofNames = KindOfMaskType::getBitNames();
		if (!pParm->m_string.isEmpty())
    {
			Bool found = false;
			for (int i = 0; kindofNames[i]; ++i)
			{
				if (pParm->m_string.compareNoCase(kindofNames[i]) == 0)
        {
					pParm->setInt(i);
					found = true;
					break;
				}
				if( !pParm->m_string.compareNoCase( "CRUSHER" ) )
				{
					//????
					pParm->setInt(i);
					found = true;
					DEBUG_CRASH(( "Kindof CRUSHER no longer exists -- in order to get your map to load, it has been switched to OBSTACLE, please call Kris (x36844).", pParm->m_string.str()));
					break;
				}
				else if( !pParm->m_string.compareNoCase( "CRUSHABLE" ) )
				{
					//????
					pParm->setInt(i);
					found = true;
					DEBUG_CRASH(( "Kindof CRUSHABLE no longer exists -- in order to get your map to load, it has been switched to OBSTACLE, please call Kris (x36844).", pParm->m_string.str()));
					break;
				}
				else if( !pParm->m_string.compareNoCase( "OVERLAPPABLE" ) )
				{
					//????
					pParm->setInt(i);
					found = true;
					DEBUG_CRASH(( "Kindof OVERLAPPABLE no longer exists -- in order to get your map to load, it has been switched to OBSTACLE, please call Kris (x36844).", pParm->m_string.str()));
					break;
				}
				else if( !pParm->m_string.compareNoCase( "MISSILE" ) )
				{
					//MISSILE was split into two kinds -- SMALL_MISSILE and BALLISTIC_MISSILE.
					pParm->m_string.format( "SMALL_MISSILE" );
					for( i = 0; kindofNames[i]; ++i )
					{
						if (pParm->m_string.compareNoCase("SMALL_MISSILE") == 0)
						{
							pParm->setInt(i);
							found = true;
							break;
						}
					}
					DEBUG_CRASH(("Unable to find Kindof SMALL_MISSILE', please call KrisM (x36844).", pParm->m_string.str()));
				}

			}
			if (!found)
      {
				DEBUG_CRASH(("Unable to find Kindof '%s', please call JKM (x36872).", pParm->m_string.str()));
				throw ERROR_BUG;
			}
		}
    else
    {
			// Seems weird, but this is so WB will load them into the proper format.
			pParm->m_string = kindofNames[pParm->m_int];
		}
	}

	return pParm;
}

//-------------------------------------------------------------------------------------------------
// ******************************** class ScriptAction ***********************************************
//-------------------------------------------------------------------------------------------------
ScriptAction::ScriptAction():
m_actionType(NO_OP),
m_hasWarnings(false),
m_numParms(0),
m_nextAction(nullptr)
{
}

ScriptAction::ScriptAction(enum ScriptActionType type):
m_actionType(type),
m_numParms(0)
{
	Int i;
	for (i=0; i<MAX_PARMS; i++) {
		m_parms[i] = nullptr;
	}
	setActionType(type);
}

void ScriptAction::setActionType(enum ScriptActionType type)
{
	Int i;
	for (i=0; i<m_numParms; i++) {
		deleteInstance(m_parms[i]);
		m_parms[i] = nullptr;
	}
	m_actionType = type;
	const ActionTemplate *pTemplate = TheScriptEngine->getActionTemplate(m_actionType);
	m_numParms = pTemplate->getNumParameters();
	for (i=0; i<m_numParms; i++) {
		m_parms[i] = newInstance(Parameter)(pTemplate->getParameterType(i));
	}
}

ScriptAction *ScriptAction::duplicate() const
{
	ScriptAction *pNew = newInstance(ScriptAction)(m_actionType);
	Int i;
	for (i=0; i<m_numParms; i++) {
		if (pNew->m_parms[i]) {
			*pNew->m_parms[i] = *m_parms[i];
		}
	}
	ScriptAction *pLink = m_nextAction;
	ScriptAction *pCur = pNew;
	while (pLink) {
		pCur->m_nextAction = newInstance(ScriptAction)(pLink->m_actionType);
		pCur = pCur->m_nextAction;
		for (i=0; i<pLink->m_numParms; i++) {
			if (pCur->m_parms[i] && pLink->m_parms[i]) {
				*pCur->m_parms[i] = *pLink->m_parms[i];
			}
		}
		pLink = pLink->m_nextAction;
	}
	return pNew;
}

ScriptAction *ScriptAction::duplicateAndQualify(const AsciiString& qualifier,
			const AsciiString& playerTemplateName, const AsciiString& newPlayerName) const
{
	ScriptAction *pNew = newInstance(ScriptAction)(m_actionType);
	Int i;
	for (i=0; i<m_numParms; i++) {
		if (pNew->m_parms[i]) {
			*pNew->m_parms[i] = *m_parms[i];
			pNew->m_parms[i]->qualify(qualifier, playerTemplateName, newPlayerName);
		}
	}
	ScriptAction *pLink = m_nextAction;
	ScriptAction *pCur = pNew;
	while (pLink) {
		pCur->m_nextAction = newInstance(ScriptAction)(pLink->m_actionType);
		pCur = pCur->m_nextAction;
		for (i=0; i<pLink->m_numParms; i++) {
			if (pCur->m_parms[i] && pLink->m_parms[i]) {
				*pCur->m_parms[i] = *pLink->m_parms[i];
				pCur->m_parms[i]->qualify(qualifier, playerTemplateName, newPlayerName);
			}
		}
		pLink = pLink->m_nextAction;
	}
	return pNew;
}

ScriptAction::~ScriptAction()
{
	Int i;
	for (i=0; i<m_numParms; i++) {
		deleteInstance(m_parms[i]);
		m_parms[i] = nullptr;
	}
	if (m_nextAction) {
		ScriptAction *cur = m_nextAction;
		ScriptAction *next;
		while (cur) {
			next = cur->getNext();
			cur->setNextAction(nullptr); // prevents recursion.
			deleteInstance(cur);
			cur = next;
		}
	}
}



Int ScriptAction::getUiStrings(AsciiString strings[MAX_PARMS])
{
	const ActionTemplate *pTemplate = TheScriptEngine->getActionTemplate(m_actionType);
	return pTemplate->getUiStrings(strings);
}

AsciiString ScriptAction::getUiText()
{
	AsciiString uiText;
	AsciiString strings[MAX_PARMS];
	Int numStrings = getUiStrings(strings);
	Int i;

	if (m_hasWarnings) {
		uiText = "[???]";
	}

	for (i=0; i<MAX_PARMS; i++) {
		if (i<numStrings) {
			uiText.concat(strings[i]);
		}
		if (i<m_numParms) {
			uiText.concat(m_parms[i]->getUiText());
		}
	}

	return uiText;
}

/**
* ScriptAction::WriteActionDataChunk - Writes an Action chunk.
* Format is the newer CHUNKY format.
*	Input: DataChunkInput
*
*/
void ScriptAction::WriteActionDataChunk(DataChunkOutput &chunkWriter, ScriptAction	*pScriptAction)
{
	/**********ACTION  DATA ***********************/
	while (pScriptAction) {
		chunkWriter.openDataChunk("ScriptAction", K_SCRIPT_ACTION_VERSION_2);
			chunkWriter.writeInt(pScriptAction->m_actionType);
			const ActionTemplate* at = TheScriptEngine->getActionTemplate(pScriptAction->m_actionType);
			if (at) {
				chunkWriter.writeNameKey(at->m_internalNameKey);
			}	else {
				DEBUG_CRASH(("Invalid action."));
				chunkWriter.writeNameKey(NAMEKEY("Bogus"));
			}
			chunkWriter.writeInt(pScriptAction->m_numParms);
			Int i;
			for (i=0; i<pScriptAction->m_numParms; i++) {
				pScriptAction->m_parms[i]->WriteParameter(chunkWriter);
			}
		chunkWriter.closeDataChunk();
		pScriptAction = pScriptAction->getNext();
	}
}

/**
* ScriptAction::ParseAction - read an action chunk in a script list.
* Format is the newer CHUNKY format.
*	See ScriptAction::WriteActionDataChunk for the writer.
*	Input: DataChunkInput
*
*/
ScriptAction *ScriptAction::ParseAction(DataChunkInput &file, DataChunkInfo *info, void *userData)
{
	ScriptAction	*pScriptAction = newInstance(ScriptAction);

	pScriptAction->m_actionType = (enum ScriptActionType)file.readInt();

	const ActionTemplate* at = TheScriptEngine->getActionTemplate(pScriptAction->m_actionType);
	if (info->version >= K_SCRIPT_ACTION_VERSION_2) {
		NameKeyType key = file.readNameKey();
		Bool match = false;
		if (at && at->m_internalNameKey == key) {
			match = TRUE; // All good. jba. [3/20/2003]
		}
		if (!match) {
			//  name and id don't match.  Find the name [3/20/2003]
			Int i;
			for (i=0; i<ScriptAction::NUM_ITEMS; i++) {
				at = TheScriptEngine->getActionTemplate(i);
				if (key == at->m_internalNameKey) {
					match = true;
					DEBUG_LOG(("Rematching script action %s", KEYNAME(key).str()));
					pScriptAction->m_actionType = (enum ScriptActionType)i;
					break;
				}
			}
			if (!match) {
				// Invalid script [3/20/2003]
				DEBUG_CRASH(("Invalid script action.  Making it noop. jba."));
				pScriptAction->m_actionType = ScriptAction::NO_OP;
				pScriptAction->m_numParms = 0;
			}
		}
	}
#ifdef DEBUG_CRASHING
	Script *pScript = (Script *)userData;
	if (at && (at->getName().isEmpty() || (at->getName().compareNoCase("(placeholder)") == 0))) {
		DEBUG_CRASH(("Invalid Script Action found in script '%s'", pScript->getName().str()));
	}
#endif
#ifdef COUNT_SCRIPT_USAGE
	const ActionTemplate* at2 = TheScriptEngine->getActionTemplate(pScriptAction->m_actionType);
	at2->m_numTimesUsed++;
	at2->m_firstMapUsed = TheGlobalData->m_mapName;
#endif
	pScriptAction->m_numParms =file.readInt();
	Int i;
	for (i=0; i<pScriptAction->m_numParms; i++)
	{
		pScriptAction->m_parms[i] = Parameter::ReadParameter(file);
	}

	// heal old files.
	switch (pScriptAction->getActionType())
	{
		case SKIRMISH_FIRE_SPECIAL_POWER_AT_MOST_COST:
			if (pScriptAction->m_numParms == 1)
			{
				pScriptAction->m_numParms = 2;
				pScriptAction->m_parms[1] = pScriptAction->m_parms[0];
				pScriptAction->m_parms[0] = newInstance(Parameter)(Parameter::SIDE, 0);
				pScriptAction->m_parms[0]->friend_setString(THIS_PLAYER);
			}
			break;
		case TEAM_FOLLOW_WAYPOINTS:
			if (pScriptAction->m_numParms == 2)
			{
				pScriptAction->m_numParms = 3;
				pScriptAction->m_parms[2] = newInstance(Parameter)(Parameter::BOOLEAN, 1);
			}
			break;
		case SKIRMISH_BUILD_BASE_DEFENSE_FRONT:
			if (pScriptAction->m_numParms == 1)
			{
				Bool flank = pScriptAction->m_parms[0]->getInt()!=0;
				deleteInstance(pScriptAction->m_parms[0]);
				pScriptAction->m_numParms = 0;
				if (flank) pScriptAction->m_actionType = SKIRMISH_BUILD_BASE_DEFENSE_FLANK;
			}
			break;
		case NAMED_SET_ATTITUDE:
		case TEAM_SET_ATTITUDE:
			if (pScriptAction->m_numParms >= 2 && pScriptAction->m_parms[1]->getParameterType() == Parameter::INT)
			{
				pScriptAction->m_parms[1] = newInstance(Parameter)(Parameter::AI_MOOD, pScriptAction->m_parms[1]->getInt());
			}
			break;
		case MAP_REVEAL_AT_WAYPOINT:
		case MAP_SHROUD_AT_WAYPOINT:
			if (pScriptAction->getNumParameters() == 2)
			{
				pScriptAction->m_numParms = 3;
				pScriptAction->m_parms[2] = newInstance(Parameter)(Parameter::SIDE);
			}
			break;
		case MAP_REVEAL_ALL:
		case MAP_REVEAL_ALL_PERM:
		case MAP_REVEAL_ALL_UNDO_PERM:
		case MAP_SHROUD_ALL:
			if (pScriptAction->getNumParameters() == 0)
			{
				pScriptAction->m_numParms = 1;
				pScriptAction->m_parms[0] = newInstance(Parameter)(Parameter::SIDE);
			}
			break;
		case SPEECH_PLAY:
			if (pScriptAction->getNumParameters() == 1)
			{
				pScriptAction->m_numParms = 2;
				// Default it to TRUE, as per conversation with JohnL
				pScriptAction->m_parms[1] = newInstance(Parameter)(Parameter::BOOLEAN, 1);
			}
			break;
		case CAMERA_MOD_SET_FINAL_ZOOM:
		case CAMERA_MOD_SET_FINAL_PITCH:
			if (pScriptAction->getNumParameters() == 1)
			{
				pScriptAction->m_numParms = 3;
				pScriptAction->m_parms[1] = newInstance(Parameter)(Parameter::PERCENT, 0.0f);
				pScriptAction->m_parms[2] = newInstance(Parameter)(Parameter::PERCENT, 0.0f);
			}
			break;
		case MOVE_CAMERA_TO:
		case MOVE_CAMERA_ALONG_WAYPOINT_PATH:
		case CAMERA_LOOK_TOWARD_OBJECT:
			if (pScriptAction->getNumParameters() == 3)
			{
				pScriptAction->m_numParms = 5;
				pScriptAction->m_parms[3] = newInstance(Parameter)(Parameter::REAL, 0.0f);
				pScriptAction->m_parms[4] = newInstance(Parameter)(Parameter::REAL, 0.0f);
			}
			break;
		case RESET_CAMERA:
		case ZOOM_CAMERA:
		case PITCH_CAMERA:
		case ROTATE_CAMERA:
			if (pScriptAction->getNumParameters() == 2)
			{
				pScriptAction->m_numParms = 4;
				pScriptAction->m_parms[2] = newInstance(Parameter)(Parameter::REAL, 0.0f);
				pScriptAction->m_parms[3] = newInstance(Parameter)(Parameter::REAL, 0.0f);
			}
			break;
		case CAMERA_LOOK_TOWARD_WAYPOINT:
			if (pScriptAction->getNumParameters() == 2)
			{
				pScriptAction->m_numParms = 5;
				pScriptAction->m_parms[2] = newInstance(Parameter)(Parameter::REAL, 0.0f);
				pScriptAction->m_parms[3] = newInstance(Parameter)(Parameter::REAL, 0.0f);
				pScriptAction->m_parms[4] = newInstance(Parameter)(Parameter::BOOLEAN, FALSE);
			}
			else if (pScriptAction->getNumParameters() == 4)
			{
				pScriptAction->m_numParms = 5;
				pScriptAction->m_parms[4] = newInstance(Parameter)(Parameter::BOOLEAN, FALSE);
			}

			break;
	}

	if (at->getNumParameters() != pScriptAction->getNumParameters()) {
		// Invalid script [3/20/2003]
		DEBUG_CRASH(("Invalid script action.  Making it noop. jba."));
		pScriptAction->m_actionType = ScriptAction::NO_OP;
		pScriptAction->m_numParms = 0;
	}
	DEBUG_ASSERTCRASH(file.atEndOfChunk(), ("Unexpected data left over."));
	return pScriptAction;
}

/**
* ScriptAction::ParseActionDataChunk - read an action chunk in a script list.
* Format is the newer CHUNKY format.
*	See ScriptAction::WriteActionDataChunk for the writer.
*	Input: DataChunkInput
*
*/
Bool ScriptAction::ParseActionDataChunk(DataChunkInput &file, DataChunkInfo *info, void *userData)
{
	Script *pScript = (Script *)userData;

	ScriptAction	*pScriptAction = ParseAction(file, info, userData);

	ScriptAction *pLast = pScript->getAction();
	while (pLast && pLast->getNext())
	{
		pLast = pLast->getNext();
	}

	if (pLast)
	{
		pLast->setNextAction(pScriptAction);
	}
	else
	{
		pScript->setAction(pScriptAction);
	}
	DEBUG_ASSERTCRASH(file.atEndOfChunk(), ("Unexpected data left over."));
	return true;
}


/**
* ScriptAction::WriteActionFalseDataChunk - Writes a false Action chunk.
* Format is the newer CHUNKY format.
*	Input: DataChunkInput
*
*/
void ScriptAction::WriteActionFalseDataChunk(DataChunkOutput &chunkWriter, ScriptAction	*pScriptAction)
{
	/**********ACTION  DATA ***********************/
	while (pScriptAction) {
		chunkWriter.openDataChunk("ScriptActionFalse", K_SCRIPT_ACTION_VERSION_2);
			chunkWriter.writeInt(pScriptAction->m_actionType);
			const ActionTemplate* at = TheScriptEngine->getActionTemplate(pScriptAction->m_actionType);
			if (at) {
				chunkWriter.writeNameKey(at->m_internalNameKey);
			}	else {
				DEBUG_CRASH(("Invalid action."));
				chunkWriter.writeNameKey(NAMEKEY("Bogus"));
			}
			chunkWriter.writeInt(pScriptAction->m_numParms);
			Int i;
			for (i=0; i<pScriptAction->m_numParms; i++) {
				pScriptAction->m_parms[i]->WriteParameter(chunkWriter);
			}
		chunkWriter.closeDataChunk();
		pScriptAction = pScriptAction->getNext();
	}
}

/**
* ScriptAction::ParseActionFalseDataChunk - read a false action chunk in a script list.
* Format is the newer CHUNKY format.
*	See ScriptAction::WriteActionDataChunk for the writer.
*	Input: DataChunkInput
*
*/
Bool ScriptAction::ParseActionFalseDataChunk(DataChunkInput &file, DataChunkInfo *info, void *userData)
{
	Script *pScript = (Script *)userData;

	ScriptAction	*pScriptAction = ParseAction(file, info, userData);

	ScriptAction *pLast = pScript->getFalseAction();
	while (pLast && pLast->getNext()) {
		pLast = pLast->getNext();
	}
	if (pLast) {
		pLast->setNextAction(pScriptAction);
	} else {
		pScript->setFalseAction(pScriptAction);
	}
	DEBUG_ASSERTCRASH(file.atEndOfChunk(), ("Unexpected data left over."));
	return true;
}

// NOTE: Changing these or adding to TheObjectFlagsNames requires changes to
// ScriptActions::changeObjectPanelFlagForSingleObject
// THEY SHOULD STAY IN SYNC.
const char* const TheObjectFlagsNames[] =
{
	"Enabled",
	"Powered",
	"Indestructible",
	"Unsellable",
	"Selectable",
	"AI Recruitable",
	"Player Targetable",
	nullptr,
};
```
