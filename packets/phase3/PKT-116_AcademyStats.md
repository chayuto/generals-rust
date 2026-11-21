# Translation Packet `PKT-116`: `AcademyStats`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-116`
- **Work Package:** `WP-3.1` (Phase 3)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast` *(35-counter telemetry tracker, 3-tier `GameClientRandomValue` advice selector, and 44-field `Xfer` snapshot)*
- **Environment Access:** `env:ephemeral-web` *(seeded with 1,000-seed C++ Release oracle `docs/golden_vectors/PKT-116.json` & compiling Rust skeleton)*
- **FFI Seam Pattern:** Pattern 2: Handle Component
- **Allowed Rust Output Files:**
  - `crates/zh_logic/src/economy/academy_stats.rs` *(implement all `todo!("PKT-116")` methods)*
  - `crates/zh_logic/tests/golden_pkt116_academy_stats.rs` *(remove both `#[ignore]` attributes once implemented)*
  - `docs/PORT_MAP.md` *(regenerate via `python3 scripts/port_map.py --write`)*
- **Mandatory V&V Gates:** `V1, V2-Golden`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test -p zh_logic --test golden_pkt116_academy_stats && python3 scripts/port_map.py --write && python3 scripts/port_map.py --check && cargo test --workspace && cargo check --workspace --all-targets
  ```

## Port Contract (read first)

1. **`AcademyStats::init(&mut self, ctx: &AcademyPlayerContext<'_>)`:**
   - Sets `next_update_frame = ctx.current_frame.wrapping_add(FRAMES_BETWEEN_UPDATES)` (`30`), `first_update = TRUE`, `unknown_side = FALSE`.
   - Checks `ctx.base_side`: if `None` OR its ASCII case-insensitive comparison (`eq_ignore_ascii_case`) is NOT `"USA"`, `"China"`, or `"GLA"`, sets `unknown_side = TRUE`.
   - Sets `spent_cash_before_building_supply_center = FALSE`, `supply_centers_built = 0`, `supply_center_cost = ctx.dozer_supply_center_cost.unwrap_or(1000)` (note C++ `findDozerCommandSet` passes `m_dozerCommandSet` by value so `m_dozerCommandSet` stays null in default init and `m_supplyCenterCost` defaults to `1000`).
   - Zeros all other Tier 1, Tier 2, and Tier 3 counters and sets all boolean flags (`researched_radar`, `had_power_last_check`, `had_a_strategy_center`, `chose_a_strategy_for_center`, `had_a_tunnel_network`, `built_barracks_within_five_minutes`, `built_war_factory_within_ten_minutes`, `built_tech_structure_within_fifteen_minutes`) to `FALSE` (`0`).

2. **`AcademyStats::update(&mut self, ctx: &AcademyPlayerContext<'_>)` Exact C++ Quirk (`m_nextUpdateFrame >= now`):**
   - If `self.unknown_side != FALSE`, return immediately.
   - Let `now = ctx.current_frame`. Note C++ writes **`if (m_nextUpdateFrame >= now)`** (not `<=`!):
     - Sets `self.next_update_frame = now.wrapping_add(FRAMES_BETWEEN_UPDATES)`.
     - Iterates `for obj in ctx.objects`: if `self.is_first_update() != FALSE`, calls `self.record_production(obj, now)`.
     - Item 2 check: if `self.supply_centers_built == 0 && self.spent_cash_before_building_supply_center == FALSE`, if `ctx.current_money < self.supply_center_cost`, sets `self.spent_cash_before_building_supply_center = TRUE`.
     - Item 12 check: let `has_power = ctx.has_sufficient_power`; if `has_power != self.had_power_last_check`:
       - If `has_power == FALSE`: sets `self.oldest_power_out_frame = now`.
       - Else: let `frames = now.wrapping_sub(self.oldest_power_out_frame)`; if `frames > self.power_out_max_frames`, sets `self.power_out_max_frames = frames`.
       - Sets `self.had_power_last_check = has_power`.
     - If `self.is_first_update() != FALSE`, calls `self.set_first_update(FALSE)`.

3. **`record_production`, `record_upgrade`, `record_special_power_used`, & `record_income` (`m_lastIncomeFrame - now` Quirk!):**
   - `record_production(obj, current_frame)` checks each `academy_kind_bits::has_kind(obj.kind_mask, ...)` independently (not `else if`!) in exact C++ order:
     - `KINDOF_FS_SUPPLY_CENTER` -> `supply_centers_built += 1`
     - `KINDOF_DOZER` -> `peons_built += 1`
     - `(KINDOF_INFANTRY || KINDOF_VEHICLE) && !KINDOF_DOZER && !KINDOF_HARVESTER` -> `idle = current_frame.wrapping_sub(last_unit_built_frame)`; if `idle > idle_building_units_max_frames`, updates max; sets `last_unit_built_frame = current_frame`
     - `KINDOF_HARVESTER` -> `gatherers_built += 1`
     - `KINDOF_HERO` -> `heroes_built += 1`
     - `KINDOF_FS_STRATEGY_CENTER` -> `had_a_strategy_center = TRUE`
     - `obj.is_tunnel_contain != FALSE` -> `had_a_tunnel_network = TRUE`
     - `KINDOF_MONEY_HACKER || KINDOF_FS_BLACK_MARKET || KINDOF_FS_SUPPLY_DROPZONE` -> `secondary_income_units_built += 1`
     - `KINDOF_FS_BARRACKS` and `current_frame <= 300 * 30` (`9000`) -> `built_barracks_within_five_minutes = TRUE`
     - `KINDOF_FS_WARFACTORY` and `current_frame <= 600 * 30` (`18000`) -> `built_war_factory_within_ten_minutes = TRUE`
     - `KINDOF_FS_ADVANCED_TECH` and `current_frame <= 900 * 30` (`27000`) -> `built_tech_structure_within_fifteen_minutes = TRUE`
     - `KINDOF_DISGUISER` -> `disguisable_vehicles_built += 1`
   - `record_upgrade(classification, granted)`: if `classification == AcademyClassificationType::UpgradeRadar`, sets `researched_radar = TRUE`; if `granted == FALSE`, increments `upgrades_purchased`.
   - `record_special_power_used(classification)`: if `classification == AcademyClassificationType::Superpower`, increments `special_powers_used`.
   - `record_income(current_frame)`: C++ writes **`UnsignedInt delta = m_lastIncomeFrame - now;`** (`self.last_income_frame.wrapping_sub(current_frame)`!). If `delta > self.max_frames_between_income`, updates `self.max_frames_between_income = delta`; then sets `self.last_income_frame = current_frame`.

4. **`calculate_academy_advice(&mut self, info, ctx, prng)` & 2-Pass `GameClientRandomValue` Draw Parity:**
   - If `self.unknown_side != FALSE`, returns `FALSE` without touching `info` or `prng`.
   - Initializes `info.num_tips = 0` and `info.advice[0] = "\n\n".to_string()`.
   - Evaluates Tier 1 (`evaluate_tier1_advice(info, -1)`), then if `info.num_tips < MAX_ADVICE_TIPS as u32` evaluates Tier 2 (`evaluate_tier2_advice(info, -1)`), and if still `< MAX_ADVICE_TIPS as u32` evaluates Tier 3 (`evaluate_tier3_advice(info, -1)`).
   - **CRITICAL PRNG & SIDE-EFFECT RULE:** Each tier runs a counting pass (`num_available_tips = -1`, `choosing = false`), and if `available_tips > 0`, recursively runs a choosing pass (`num_available_tips = available_tips`, `choosing = true`).
     - In **BOTH** the counting pass (`num_available_tips == -1`) and the choosing pass, every matching condition **ALWAYS calls `prng.get_game_client_random_value(0, num_available_tips - 1)`** and decrements `num_available_tips -= 1`! (When `num_available_tips <= 1`, `get_game_client_random_value(0, hi)` with `hi <= 0` returns `hi` without advancing `client_seed`, whereas `hi >= 1` advances `client_seed`!)
     - In Tier 1: Item 9 updates `self.idle_building_units_max_frames = max(self.idle_building_units_max_frames, now.wrapping_sub(self.last_unit_built_frame))` before checking `> 300 * 30 || self.last_unit_built_frame == 0`. Item 12 (when `self.had_power_last_check == FALSE`) updates `self.power_out_max_frames = max(self.power_out_max_frames, now.wrapping_sub(self.oldest_power_out_frame))` before checking `> 600 * 30`.
     - In Tier 2: Item 20 checks `ctx.base_side.is_some_and(|s| s.eq_ignore_ascii_case("GLA"))` and `self.salvage_collected == 0`.
     - In Tier 3: Item 25 checks `ctx.use_alternate_mouse == FALSE`. Item 30 updates `self.max_frames_between_income = max(self.max_frames_between_income, self.last_income_frame.wrapping_sub(now))` before checking `> 30 * 120 || self.last_income_frame == 0`. Item 31 checks `ctx.local_player_mines > 0 && self.mines_cleared == 0` (and appends `"ACADEMY:NoIncome"`, matching C++ line 984!). Item 32 checks `ctx.neutral_player_vehicles_sniped > 0 && self.vehicles_recovered == 0`. Item 35 checks `ctx.base_side.is_some_and(|s| s.eq_ignore_ascii_case("China")) && self.firestorms_created == 0`.
     - When `choosing && rand < (MAX_ADVICE_TIPS as i32 - info.num_tips as i32)`, appends the `"ACADEMY:..."` key string to `info.advice[info.num_tips as usize]` and increments `info.num_tips += 1`.

5. **`Snapshot::xfer` Exact 44-Call Sequence:**
   - `version = xfer.xfer_version(1, 1)?`, followed by the 43 fields in the exact order listed in Section 2 (`next_update_frame`, `first_update`, `unknown_side`, ..., `firestorms_created`). `crc` and `load_post_process` are no-ops (`Ok(())`).

6. **Determinism & 3-Attempt Escalation Rule:** Zero external crates, no `unsafe`, no `HashMap`/`HashSet`/`BinaryHeap`, no `Rc`/`RefCell`, no std float math. If 3 consecutive fix attempts fail to shrink failing vectors, isolate the minimal failing seed or escalate (`tier:3-architect`).

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (17):**
  - `Common/AcademyStats.h`
  - `Common/Debug.h`
  - `Common/Energy.h`
  - `Common/Player.h`
  - `Common/PlayerList.h`
  - `Common/PlayerTemplate.h`
  - `Common/Snapshot.h`
  - `Common/SpecialPower.h`
  - `Common/ThingTemplate.h`
  - `Common/UnicodeString.h`
  - `Common/Xfer.h`
  - `GameClient/ControlBar.h`
  - `GameClient/GameText.h`
  - `GameLogic/GameLogic.h`
  - `GameLogic/Module/ContainModule.h`
  - `GameLogic/Object.h`
  - `PreRTS.h`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (44 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUnsignedInt( &m_nextUpdateFrame )`
  3. `xferBool( &m_firstUpdate )`
  4. `xferBool( &m_unknownSide )`
  5. `xferBool( &m_spentCashBeforeBuildingSupplyCenter )`
  6. `xferUnsignedInt( &m_supplyCentersBuilt )`
  7. `xferUnsignedInt( &m_supplyCenterCost )`
  8. `xferBool( &m_researchedRadar )`
  9. `xferUnsignedInt( &m_peonsBuilt )`
  10. `xferUnsignedInt( &m_structuresCaptured )`
  11. `xferUnsignedInt( &m_generalsPointsSpent )`
  12. `xferUnsignedInt( &m_specialPowersUsed )`
  13. `xferUnsignedInt( &m_structuresGarrisoned )`
  14. `xferUnsignedInt( &m_idleBuildingUnitsMaxFrames )`
  15. `xferUnsignedInt( &m_lastUnitBuiltFrame )`
  16. `xferUnsignedInt( &m_dragSelectUnits )`
  17. `xferUnsignedInt( &m_upgradesPurchased )`
  18. `xferUnsignedInt( &m_powerOutMaxFrames )`
  19. `xferUnsignedInt( &m_oldestPowerOutFrame )`
  20. `xferBool ( &m_hadPowerLastCheck )`
  21. `xferUnsignedInt( &m_gatherersBuilt )`
  22. `xferUnsignedInt( &m_heroesBuilt )`
  23. `xferBool( &m_hadAStrategyCenter )`
  24. `xferBool( &m_choseAStrategyForCenter )`
  25. `xferUnsignedInt( &m_unitsEnteredTunnelNetwork )`
  26. `xferBool( &m_hadATunnelNetwork )`
  27. `xferUnsignedInt( &m_controlGroupsUsed )`
  28. `xferUnsignedInt( &m_secondaryIncomeUnitsBuilt )`
  29. `xferUnsignedInt( &m_clearedGarrisonedBuildings )`
  30. `xferUnsignedInt( &m_salvageCollected )`
  31. `xferUnsignedInt( &m_guardAbilityUsedCount )`
  32. `xferUnsignedInt( &m_doubleClickAttackMoveOrdersGiven )`
  33. `xferBool( &m_builtBarracksWithinFiveMinutes )`
  34. `xferBool( &m_builtWarFactoryWithinTenMinutes )`
  35. `xferBool( &m_builtTechStructureWithinFifteenMinutes )`
  36. `xferUnsignedInt( &m_lastIncomeFrame )`
  37. `xferUnsignedInt( &m_maxFramesBetweenIncome )`
  38. `xferUnsignedInt( &m_mines )`
  39. `xferUnsignedInt( &m_minesCleared )`
  40. `xferUnsignedInt( &m_vehiclesRecovered )`
  41. `xferUnsignedInt( &m_vehiclesSniped )`
  42. `xferUnsignedInt( &m_disguisableVehiclesBuilt )`
  43. `xferUnsignedInt( &m_vehiclesDisguised )`
  44. `xferUnsignedInt( &m_firestormsCreated )`
- **PRNG Calls (31 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`
  - `GameClientRandomValue( 0, numAvailableTips - 1 )`

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

### `Core/GameEngine/Include/Common/AcademyStats.h`
```cpp
// FILE: AcademyStats.h ////////////////////////////////////////////////////////////
//-----------------------------------------------------------------------------
//
//                       Electronic Arts Los Angeles
//
//                       Confidential Information
//                Copyright (C) 2003 - All Rights Reserved
//
//-----------------------------------------------------------------------------
//
// Project:    RTS3
//
// File name:  AcademyStats.h
//
// Created:    Kris Morness, July 2003
//
// Desc:			 Keeps track of various statistics in order to provide advice to
//             the player about how to improve playing.
//
//-----------------------------------------------------------------------------

#pragma once

#include "Common/Debug.h"
#include "Common/Snapshot.h"
#include "Common/UnicodeString.h"

class Object;
class Player;
class SpecialPowerTemplate;
class UpgradeTemplate;
class CommandSet;
class ThingTemplate;

#define MAX_ADVICE_TIPS 1

struct AcademyAdviceInfo
{
	UnicodeString advice[ MAX_ADVICE_TIPS ];
	UnsignedInt numTips;
};

enum AcademyClassificationType CPP_11(: Int)
{
	//Don't forget to update the strings too!
	ACT_NONE,
	ACT_UPGRADE_RADAR,
	ACT_SUPERPOWER,

	ACT_COUNT
};
extern const char *const TheAcademyClassificationTypeNames[]; //Change above, change this!


// ----------------------------------------------------------------------------------------------
class AcademyStats : public Snapshot
{

public:

	AcademyStats();

	void init( const Player *player );
	void update();

	Bool isFirstUpdate() const { return m_firstUpdate; }
	void setFirstUpdate( Bool set ) { m_firstUpdate = set; }

	void recordProduction( const Object *obj, const Object *constructor );
	void recordUpgrade( const UpgradeTemplate *upgrade, Bool granted );
	void recordSpecialPowerUsed( const SpecialPowerTemplate *spTemplate );
	void recordIncome();

	void recordBuildingCapture() { m_structuresCaptured++; }
	void recordGeneralsPointsSpent( Int points ) { m_generalsPointsSpent += points; }
	void recordBuildingGarrisoned() { m_structuresGarrisoned++; }
	void recordDragSelection() { m_dragSelectUnits++; }
	void recordStrategyCenter() { m_hadAStrategyCenter = TRUE; }
	void recordBattlePlanSelected() { m_choseAStrategyForCenter = TRUE; }
	void recordUnitEnteredTunnelNetwork() { m_unitsEnteredTunnelNetwork++; }
	void recordControlGroupsUsed() { m_controlGroupsUsed++; }
	void recordClearedGarrisonedBuilding() { m_clearedGarrisonedBuildings++; }
	void recordVehicleDisguised() { m_vehiclesDisguised++; }
	void recordFirestormCreated() { m_firestormsCreated++; }
	void recordGuardAbilityUsed() { m_guardAbilityUsedCount++; }
	void recordSalvageCollected() { m_salvageCollected++;}
	void recordDoubleClickAttackMoveOrderGiven() { m_doubleClickAttackMoveOrdersGiven++; }
	void recordMineCleared() { m_minesCleared++; }

	//Returns the natural command center template (you may capture others in a game...)
	const ThingTemplate* getCommandCenterTemplate() const { return m_commandCenterTemplate; }

	//Use these functions for the Neutral player only!
	void recordVehicleSniped() { m_vehiclesSniped++; }
	UnsignedInt getVehiclesSniped() const { return m_vehiclesSniped; }
	void recordMine() { m_mines++; }
	UnsignedInt getMines() const { return m_mines; }

	const Player *getPlayer() { return m_player; }
	Bool hadASupplyCenter() const { return m_supplyCentersBuilt > 0; }

	Bool calculateAcademyAdvice( AcademyAdviceInfo *info );

protected:
	// snapshot methods
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

private:

	void evaluateTier1Advice( AcademyAdviceInfo *info, Int numAvailableTips = -1 );
	void evaluateTier2Advice( AcademyAdviceInfo *info, Int numAvailableTips = -1 );
	void evaluateTier3Advice( AcademyAdviceInfo *info, Int numAvailableTips = -1 );

	const Player *m_player;
	UnsignedInt m_nextUpdateFrame;
	Bool m_firstUpdate;
	const CommandSet *m_dozerCommandSet;
	Bool m_unknownSide;
	const ThingTemplate *m_commandCenterTemplate;

	//+-----------------------+
	//| Tier 1 (Basic advice) |
	//+-----------------------+

	//1) Did player build at least one of each structure type available?
	//CUT!!!

	//2) Did player run out of money before building a supply center?
	Bool m_spentCashBeforeBuildingSupplyCenter;
	UnsignedInt m_supplyCentersBuilt;
	const ThingTemplate *m_supplyCenterTemplate;
	UnsignedInt m_supplyCenterCost;

	//3) Did player build radar (if applicable)?
	Bool m_researchedRadar;

	//4) Did player build any dozers/workers?
	UnsignedInt m_peonsBuilt;

	//5) Did player ever capture a structure?
	UnsignedInt m_structuresCaptured;

	//6) Did player spend any generals points?
	UnsignedInt m_generalsPointsSpent;

	//7) Did player ever use a generals power or superweapon?
	UnsignedInt m_specialPowersUsed;

	//8) Did player garrison any structures?
	UnsignedInt m_structuresGarrisoned;

	//9) How idle was the player in building military units?
	UnsignedInt m_idleBuildingUnitsMaxFrames;
	UnsignedInt m_lastUnitBuiltFrame;

	//10) Did player drag select units?
	UnsignedInt m_dragSelectUnits;

	//11) Did player upgrade anything?
	UnsignedInt m_upgradesPurchased;

	//12) Was player out of power for more than 10 minutes?
	UnsignedInt m_powerOutMaxFrames;
	UnsignedInt m_oldestPowerOutFrame;
	Bool				m_hadPowerLastCheck;

	//13) Extra gathers built?
	UnsignedInt m_gatherersBuilt;

	//14) Heroes built?
	UnsignedInt m_heroesBuilt;

	//+------------------------------+
	//| Tier 2 (Intermediate advice) |
	//+------------------------------+

	//15) Selected a strategy center battle plan?
	Bool m_hadAStrategyCenter;
	Bool m_choseAStrategyForCenter;

	//16) Placed units inside tunnel network?
	UnsignedInt m_unitsEnteredTunnelNetwork;
	Bool m_hadATunnelNetwork;

	//17) Player used control groups?
	UnsignedInt m_controlGroupsUsed;

	//18) Built secondary income unit (hacker, dropzone, blackmarket)?
	UnsignedInt m_secondaryIncomeUnitsBuilt;

	//19) Cleared out garrisoned buildings?
	UnsignedInt m_clearedGarrisonedBuildings;

	//20) Did the Player pick up salvage (as GLA)?
	UnsignedInt m_salvageCollected;

	//21) Did the player ever use the "Guard" ability?
	UnsignedInt m_guardAbilityUsedCount;

	//22) Did the player build more than one Supply Center (that is, did he expand out)?
	//Uses m_supplyCentersBuilt!

	//23) Did the player ever garrison a vehicle?
	//CUT!!!

	//24) Did the player ever use the hotkey to grab all of one unit type (change to use any hotkeys)?
	//CUT!!!

	//+--------------------------+
	//| Tier 3 (Advanced advice) |
	//+--------------------------+

	//25) Did the player use the new alternate interface in the options?
	//Uses TheGlobalData->m_useAlternateMouse

	//26) Player did not use the new "double click location attack move/guard"
	UnsignedInt m_doubleClickAttackMoveOrdersGiven;

  //27) Built barracks within 5 minutes?
	Bool m_builtBarracksWithinFiveMinutes;

	//28) Built war factory within 10 minutes?
	Bool m_builtWarFactoryWithinTenMinutes;

	//29) Built tech structure within 15 minutes?
	Bool m_builtTechStructureWithinFifteenMinutes;

	//30) No income for 2 minutes?
	UnsignedInt m_lastIncomeFrame;
	UnsignedInt m_maxFramesBetweenIncome;

	//31) Did the Player ever use Dozers/Workers to clear out traps/mines/booby traps?
	UnsignedInt m_mines; //Neutral player stat
	UnsignedInt m_minesCleared;

	//32) Captured any sniped vehicles?
	UnsignedInt m_vehiclesRecovered;
	UnsignedInt m_vehiclesSniped;

	//33) Did the player ever build a "disguisable" unit and never used the disguise ability?
	UnsignedInt m_disguisableVehiclesBuilt;
	UnsignedInt m_vehiclesDisguised;

	//34) Did the player never build a "stealth" upgrade?
	//CUT!!!

	//35) Did the player ever create a "Firestorm" with his MiGs or Inferno Cannons?
	UnsignedInt m_firestormsCreated;
};
```

### `Core/GameEngine/Source/Common/RTS/AcademyStats.cpp`
```cpp
// FILE: AcademyStats.cpp //////////////////////////////////////////////////////
//-----------------------------------------------------------------------------
//
//                       Electronic Arts Los Angeles
//
//                       Confidential Information
//                Copyright (C) 2003 - All Rights Reserved
//
//-----------------------------------------------------------------------------
//
// Project:    RTS3
//
// File name:  AcademyStats.cpp
//
// Created:    Kris Morness, July 2003
//
// Desc:			 Keeps track of various statistics in order to provide advice to
//             the player about how to improve playing.
//
//-----------------------------------------------------------------------------

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/AcademyStats.h"
#include "Common/Energy.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/PlayerTemplate.h"
#include "Common/SpecialPower.h"
#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"

#include "GameClient/ControlBar.h"
#include "GameClient/GameText.h"

#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"

#include "GameLogic/Module/ContainModule.h"


const char *const TheAcademyClassificationTypeNames[] =
{
	"ACT_NONE",
	"ACT_UPGRADE_RADAR",
	"ACT_SUPERPOWER",
	nullptr
};
static_assert(ARRAY_SIZE(TheAcademyClassificationTypeNames) == ACT_COUNT + 1, "Incorrect array size");

#define FRAMES_BETWEEN_UPDATES 30


//------------------------------------------------------------------------------------------------
AcademyStats::AcademyStats()
{
}

//------------------------------------------------------------------------------------------------
void findDozerCommandSet( Object *object, void *userData )
{
	const CommandSet *dozerCommandSet = (const CommandSet*)userData;
	if( dozerCommandSet )
	{
		return;
	}
	if( object->isKindOf( KINDOF_DOZER ) )
	{
		dozerCommandSet = TheControlBar->findCommandSet( object->getCommandSetString() );
	}
}

//------------------------------------------------------------------------------------------------
void AcademyStats::init( const Player *player )
{
	if( !TheGameLogic )
	{
		return; // GUIEdit crashes on this, so bail
	}

	m_nextUpdateFrame = TheGameLogic->getFrame() + FRAMES_BETWEEN_UPDATES;
	m_firstUpdate = TRUE;
	m_unknownSide = FALSE;

	//Init the player and cache some important information.
	m_player = player;
	const PlayerTemplate *plyrTemplate = player->getPlayerTemplate();

	if( !plyrTemplate ||
			( plyrTemplate->getBaseSide().compareNoCase( "USA" ) &&
			plyrTemplate->getBaseSide().compareNoCase( "China" ) &&
			plyrTemplate->getBaseSide().compareNoCase( "GLA" ) ) )
	{
		//Admittedly, this is a massive violation of data driven design. Shame on me!
		//Unknown side... don't provide ANY advice. Simplicity reasons.
		m_unknownSide = TRUE;
	}

	//Find the command set for our dozer... so we can extract information about things
	//we can build.
	m_dozerCommandSet = nullptr;
	player->iterateObjects( findDozerCommandSet, (void*)m_dozerCommandSet );

	m_commandCenterTemplate = nullptr;
	m_supplyCenterTemplate = nullptr;

	if( m_dozerCommandSet )
	{
		for( int i = 0; i < MAX_COMMANDS_PER_SET; i++ )
		{
			const CommandButton *button = m_dozerCommandSet->getCommandButton( i );
			if( button )
			{
				const ThingTemplate *thing = button->getThingTemplate();
				if( thing )
				{
					if( thing->isKindOf( KINDOF_FS_SUPPLY_CENTER ) )
					{
						m_supplyCenterTemplate = thing;
						m_supplyCenterCost = m_supplyCenterTemplate->calcCostToBuild( player );
					}
					else if( thing->isKindOf( KINDOF_COMMANDCENTER ) )
					{
						//Wow, this is bogus... but we need this template pointer in order to fire special powers from the
						//shortcut... ugh.
						m_commandCenterTemplate = thing;
					}
				}
			}
		}
	}

	//+-----------------------+
	//| Tier 1 (Basic advice) |
	//+-----------------------+

	//1) Did player build at least one of each structure type available?
	//CUT!!!

	//2) Did player run out of money before building a supply center?
	m_spentCashBeforeBuildingSupplyCenter = FALSE;
	m_supplyCentersBuilt = 0;
	if( !m_supplyCenterTemplate )
	{
		m_supplyCenterCost = 1000;
	}

	//3) Did player build radar (if applicable)?
	m_researchedRadar = FALSE;

	//4) Did player build any dozers/workers?
	m_peonsBuilt = 0;

	//5) Did player ever capture a structure?
	m_structuresCaptured = 0;

	//6) Did player spend any generals points?
	m_generalsPointsSpent = 0;

	//7) Did player ever use a generals power or superweapon?
	m_specialPowersUsed = 0;

	//8) Did player garrison any structures?
	m_structuresGarrisoned = 0;

	//9) How idle was the player in building military units?
	m_idleBuildingUnitsMaxFrames = 0;
	m_lastUnitBuiltFrame = 0;

	//10) Did player drag select units?
	m_dragSelectUnits = 0;

	//11) Did player upgrade anything?
	m_upgradesPurchased = 0;

	//12) Was player out of power for more than 10 minutes?
	m_powerOutMaxFrames = 0;
	m_oldestPowerOutFrame = 0;
	m_hadPowerLastCheck = 0;

	//13) Extra gatherers built?
	m_gatherersBuilt = 0;

	//14) Heroes built?
	m_heroesBuilt = 0;

	//+------------------------------+
	//| Tier 2 (Intermediate advice) |
	//+------------------------------+

	//15) Selected a strategy center battle plan?
	m_hadAStrategyCenter = FALSE;
	m_choseAStrategyForCenter = FALSE;

	//16) Placed units inside tunnel network?
	m_hadATunnelNetwork = FALSE;
	m_unitsEnteredTunnelNetwork = 0;

	//17) Player used control groups?
	m_controlGroupsUsed = 0;

	//18) Built secondary income building?
	m_secondaryIncomeUnitsBuilt = 0;

	//19) Cleared out garrisoned buildings?
	m_clearedGarrisonedBuildings = 0;

	//20) Did the Player pick up salvage (as GLA)?
	m_salvageCollected = 0;

	//21) Did the player ever use the "Guard" ability?
	m_guardAbilityUsedCount = 0;

	//22) Did the player build more than one Supply Center (that is, did he expand out)?
	//Uses m_supplyCentersBuilt!

	//+--------------------------+
	//| Tier 3 (Advanced advice) |
	//+--------------------------+

	//25) Did the player use the new alternate interface in the options?
	//Uses TheGlobalData->m_useAlternateMouse

	//26) Player did not use the new "double click location attack move/guard"
	m_doubleClickAttackMoveOrdersGiven = 0;

  //27) Built barracks within 5 minutes?
	m_builtBarracksWithinFiveMinutes = FALSE;

	//28) Built war factory within 10 minutes?
	m_builtWarFactoryWithinTenMinutes = FALSE;

	//29) Built tech structure within 15 minutes?
	m_builtTechStructureWithinFifteenMinutes = FALSE;

	//30) No income for 2 minutes?
	m_lastIncomeFrame = 0;
	m_maxFramesBetweenIncome = 0;

	//31) Did the Player ever use Dozers/Workers to clear out traps/mines/booby traps?
	m_mines = 0;
	m_minesCleared = 0;

	//32) Captured any sniped vehicles?
	m_vehiclesRecovered = 0;
	m_vehiclesSniped = 0; //Neutral team stat

	//33) Did the player ever build a "disguisable" unit and never used the disguise ability?
	m_disguisableVehiclesBuilt = 0;
	m_vehiclesDisguised = 0;

	//34) Did the player never build a "stealth" upgrade?
	//CUT!!!

	//35) Did the player ever create a "Firestorm" with his MiGs or Inferno Cannons?
	m_firestormsCreated = 0;
}

//------------------------------------------------------------------------------------------------
static void updateAcademyStats( Object *obj, void *userData )
{
	AcademyStats *academy = (AcademyStats*)userData;

	if( academy->isFirstUpdate() )
	{
		academy->recordProduction( obj, nullptr );
	}
}

//------------------------------------------------------------------------------------------------
void AcademyStats::update()
{
	if( m_unknownSide )
	{
		return;
	}

	UnsignedInt now = TheGameLogic->getFrame();

	if( m_nextUpdateFrame >= now )
	{
		m_nextUpdateFrame = now + FRAMES_BETWEEN_UPDATES;
		m_player->iterateObjects( updateAcademyStats, (void*)this );

		//2) Did player run out of money before building a supply center?
		if( !m_supplyCentersBuilt && !m_spentCashBeforeBuildingSupplyCenter )
		{
			const Money *money = m_player->getMoney();
			if( money )
			{
				UnsignedInt amount = money->countMoney();
				if( amount < m_supplyCenterCost )
				{
					m_spentCashBeforeBuildingSupplyCenter = TRUE;
				}
			}
		}

		//12) Was player out of power for more than 10 minutes?
		const Energy *power = m_player->getEnergy();
		if( power )
		{
			Bool hasPower = power->hasSufficientPower();

			if( hasPower != m_hadPowerLastCheck )
			{
				if( !hasPower )
				{
					//We just lost power
					m_oldestPowerOutFrame = now;
				}
				else
				{
					//We just gained power... see how long we were without.
					UnsignedInt frames = now - m_oldestPowerOutFrame;
					if( frames > m_powerOutMaxFrames )
					{
						m_powerOutMaxFrames = frames;
					}
				}
				m_hadPowerLastCheck = hasPower;
			}
		}

		if( isFirstUpdate() )
		{
			setFirstUpdate( FALSE );
		}

	}
}

//------------------------------------------------------------------------------------------------
void AcademyStats::recordProduction( const Object *obj, const Object *constructor )
{
	UnsignedInt now = TheGameLogic->getFrame();

	//2) Did player run out of money before building a supply center?
	//22) Did the player build more than one Supply Center (that is, did he expand out)?
	if( obj->isKindOf( KINDOF_FS_SUPPLY_CENTER ) )
	{
		m_supplyCentersBuilt++;
	}

	//4) Did player build any dozers/workers?
	if( obj->isKindOf( KINDOF_DOZER ) )
	{
		m_peonsBuilt++;
	}

	//9) How idle was the player in building military units?
	if( obj->isKindOf( KINDOF_INFANTRY ) || obj->isKindOf( KINDOF_VEHICLE ) )
	{
		if( !obj->isKindOf( KINDOF_DOZER ) && !obj->isKindOf( KINDOF_HARVESTER ) )
		{
			//How long has it been since we built our last unit?
			UnsignedInt idleFrames = now - m_lastUnitBuiltFrame;

			//If it was longer than our max time... then record it.
			if( idleFrames > m_idleBuildingUnitsMaxFrames )
			{
				m_idleBuildingUnitsMaxFrames = idleFrames;
			}

			//Record the frame we built our unit.
			m_lastUnitBuiltFrame = now;
		}
	}

	//13) Extra gatherers built?
	if( obj->isKindOf( KINDOF_HARVESTER ) )
	{
		m_gatherersBuilt++;
	}

	//14) Heroes built?
	if( obj->isKindOf( KINDOF_HERO ) )
	{
		m_heroesBuilt++;
	}

	//15) Selected a strategy center battle plan?
	if( obj->isKindOf( KINDOF_FS_STRATEGY_CENTER ) )
	{
		m_hadAStrategyCenter = TRUE;
	}

	//16) Placed units inside tunnel network?
	if( obj->getContain() && obj->getContain()->isTunnelContain() )
	{
		m_hadATunnelNetwork = TRUE;
	}

	//18) Built secondary income building?
	if( obj->isKindOf( KINDOF_MONEY_HACKER ) || obj->isKindOf( KINDOF_FS_BLACK_MARKET ) || obj->isKindOf( KINDOF_FS_SUPPLY_DROPZONE ) )
	{
		m_secondaryIncomeUnitsBuilt++;
	}

	//27) Built tech structure within 15 minutes?
	if( obj->isKindOf( KINDOF_FS_BARRACKS ) )
	{
		if( TheGameLogic->getFrame() <= 300 * LOGICFRAMES_PER_SECOND )
		{
			m_builtBarracksWithinFiveMinutes = TRUE;
		}
	}

	//28) Built tech structure within 15 minutes?
	if( obj->isKindOf( KINDOF_FS_WARFACTORY ) )
	{
		if( TheGameLogic->getFrame() <= 600 * LOGICFRAMES_PER_SECOND )
		{
			m_builtWarFactoryWithinTenMinutes = TRUE;
		}
	}

	//29) Built tech structure within 15 minutes?
	if( obj->isKindOf( KINDOF_FS_ADVANCED_TECH ) )
	{
		if( TheGameLogic->getFrame() <= 900 * LOGICFRAMES_PER_SECOND )
		{
			m_builtTechStructureWithinFifteenMinutes = TRUE;
		}
	}

	//33) Did the player ever build a "disguisable" unit and never used the disguise ability?
	if( obj->isKindOf( KINDOF_DISGUISER ) )
	{
		m_disguisableVehiclesBuilt++;
	}

}

//------------------------------------------------------------------------------------------------
void AcademyStats::recordUpgrade( const UpgradeTemplate *upgrade, Bool granted )
{
	//3) Did player build radar (if applicable)?
	if( upgrade->getAcademyClassificationType() == ACT_UPGRADE_RADAR )
	{
		m_researchedRadar = TRUE;
	}

	//11) Did player upgrade anything?
	if( !granted )
	{
		m_upgradesPurchased++;
	}
}

//------------------------------------------------------------------------------------------------
void AcademyStats::recordSpecialPowerUsed( const SpecialPowerTemplate *spTemplate )
{
	if( spTemplate->getAcademyClassificationType() == ACT_SUPERPOWER )
	{
		m_specialPowersUsed++;
	}
}

//------------------------------------------------------------------------------------------------
void AcademyStats::recordIncome()
{
	UnsignedInt now = TheGameLogic->getFrame();

	UnsignedInt delta = m_lastIncomeFrame - now;
	if( delta > m_maxFramesBetweenIncome )
	{
		m_maxFramesBetweenIncome = delta;
	}

	m_lastIncomeFrame = now;
}

//------------------------------------------------------------------------------------------------
void AcademyStats::evaluateTier1Advice( AcademyAdviceInfo *info, Int numAvailableTips )
{
	UnsignedInt maxAdviceTips = MAX_ADVICE_TIPS;

	//-allAdvice feature
	//if( !TheGlobalData->m_allAdvice )
	//{
	//	maxAdviceTips = 1;
	//}

	UnsignedInt now = TheGameLogic->getFrame();
	//numAvailableTips is used to determine if we are going to randomly choose a tip
	//or determine if a tip is available (if -1)
	Bool choosing = numAvailableTips != -1;
	Int availableTips = 0;

	//1) Did player build at least one of each structure type available?
	//CUT!!!

	//2) Did player run out of money before building a supply center?
	if( m_spentCashBeforeBuildingSupplyCenter )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:BuildSupplyCenterEarlier" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//3) Did player build radar (if applicable)?
	if( !m_researchedRadar )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:TryBuildingRadar" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//4) Did player build any dozers/workers?
	if( m_peonsBuilt < 2 )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:BuildMorePeons" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//5) Did player ever capture a structure?
	if( !m_structuresCaptured )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:TryCapturingStructures" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//6) Did player spend any generals points?
	if( !m_generalsPointsSpent )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:SpendGeneralsPoints" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//7) Did player ever use a generals power or superweapon?
	if( !m_specialPowersUsed )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:TryUsingSuperweapons" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//8) Did player garrison any structures?
	if( !m_structuresGarrisoned )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:TryGarrisoningAStructure" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//9) How idle was the player in building military units?
	UnsignedInt idleFrames = now - m_lastUnitBuiltFrame;
	if( idleFrames > m_idleBuildingUnitsMaxFrames )
	{
		m_idleBuildingUnitsMaxFrames = idleFrames;
	}
	if( m_idleBuildingUnitsMaxFrames > 300 * LOGICFRAMES_PER_SECOND || !m_lastUnitBuiltFrame )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:IdleBuildingUnits" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//10) Did player drag select units?
	if( !m_dragSelectUnits )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:TryDragSelectingUnits" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//11) Did player upgrade anything?
	if( !m_upgradesPurchased )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:ResearchUpgrades" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//12) Was player out of power for more than 10 minutes?

	if( !m_hadPowerLastCheck )
	{
		UnsignedInt frames = now - m_oldestPowerOutFrame;
		if( frames > m_powerOutMaxFrames )
		{
			m_powerOutMaxFrames = frames;
		}
	}
	if( m_powerOutMaxFrames > 600 * LOGICFRAMES_PER_SECOND )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:RanOutOfPower" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//13) Extra gatherers built?
	if( !m_gatherersBuilt )
	{
		//Don't count free ones that come with supply centers!
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:BuildMoreGatherers" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//14) Heroes built?
	if( !m_heroesBuilt )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:BuildAHero" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	if( !choosing && availableTips > 0 )
	{
		//We're done counting available tips, so now that we know the
		//number, we can randomly choose.
		evaluateTier1Advice( info, availableTips );
	}
}

//------------------------------------------------------------------------------------------------
void AcademyStats::evaluateTier2Advice( AcademyAdviceInfo *info, Int numAvailableTips )
{
	UnsignedInt maxAdviceTips = MAX_ADVICE_TIPS;

	//-allAdvice feature
	//if( !TheGlobalData->m_allAdvice )
	//{
	//	maxAdviceTips = 1;
	//}

	//numAvailableTips is used to determine if we are going to randomly choose a tip
	//or determine if a tip is available (if -1)
	Bool choosing = numAvailableTips != -1;
	Int availableTips = 0;

	//15) Selected a strategy center battle plan?
	if( m_hadAStrategyCenter && !m_choseAStrategyForCenter )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:PickStrategyCenterPlan" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//16) Placed units inside tunnel network?
	if( m_hadATunnelNetwork && !m_unitsEnteredTunnelNetwork )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:UseTunnelNetwork" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//17) Player used control groups?
	if( !m_controlGroupsUsed )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:UseControlGroups" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//18) Built secondary income building?
	if( !m_secondaryIncomeUnitsBuilt )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:UseSecondaryIncomeMethods" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//19) Cleared out garrisoned buildings?
	if( !m_clearedGarrisonedBuildings )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:ClearBuildings" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//20) Did the Player pick up salvage (as GLA)?
	if( m_player->getPlayerTemplate() && !m_player->getPlayerTemplate()->getBaseSide().compareNoCase( "GLA" ) )
	{
		if( !m_salvageCollected )
		{
			availableTips++;
			Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
			Int limit = maxAdviceTips - info->numTips;
			if( choosing &&  rand < limit )
			{
				info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:PickUpSalvage" ) );
				info->numTips++;
			}
			numAvailableTips--;
		}
	}

	//21) Did the player ever use the "Guard" ability?
	if( !m_guardAbilityUsedCount )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:UseGuardAbility" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//22) Did the player build more than one Supply Center (that is, did he expand out)?
	if( m_supplyCentersBuilt < 2 )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:MultipleSupplyCenters" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//23) Did the player ever garrison a vehicle?
	//CUT!!!

	//24) Did the player ever use the hotkey to grab all of one unit type (change to use any hotkeys)?
	//CUT!!!

	if( !choosing && availableTips > 0 )
	{
		//We're done counting available tips, so now that we know the
		//number, we can randomly choose.
		evaluateTier2Advice( info, availableTips );
	}

}

//------------------------------------------------------------------------------------------------
void AcademyStats::evaluateTier3Advice( AcademyAdviceInfo *info, Int numAvailableTips )
{
	UnsignedInt maxAdviceTips = MAX_ADVICE_TIPS;

	//-allAdvice feature
	//if( !TheGlobalData->m_allAdvice )
	//{
	//	maxAdviceTips = 1;
	//}

	UnsignedInt now = TheGameLogic->getFrame();

	//numAvailableTips is used to determine if we are going to randomly choose a tip
	//or determine if a tip is available (if -1)
	Bool choosing = numAvailableTips != -1;
	Int availableTips = 0;

	//25) Did the player use the new alternate interface in the options?
	if( !TheGlobalData->m_useAlternateMouse )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:AlternateMouseInterface" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//26) Player did not use the new "double click location attack move/guard"
	if( !m_doubleClickAttackMoveOrdersGiven )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:DoubleClickAttackMoveGuard" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

  //27) Built barracks within 5 minutes?
	if( !m_builtBarracksWithinFiveMinutes )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:BuildBarracksSooner" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//28) Built warfactory within 10 minutes?
	if( !m_builtWarFactoryWithinTenMinutes )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:BuildWarFactorySooner" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//29) Built tech structure within 15 minutes?
	if( !m_builtTechStructureWithinFifteenMinutes )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:BuildTechStructureSooner" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//30) No income for 2 minutes?
	UnsignedInt delta = m_lastIncomeFrame - now;
	if( delta > m_maxFramesBetweenIncome )
	{
		m_maxFramesBetweenIncome = delta;
	}
	if( m_maxFramesBetweenIncome > LOGICFRAMES_PER_SECOND * 120 || !m_lastIncomeFrame )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:NoIncome" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//31) Did the Player ever use Dozers/Workers to clear out traps/mines/booby traps?
	if( ThePlayerList->getLocalPlayer()->getAcademyStats()->getMines() > 0 && !m_minesCleared )
	{
		availableTips++;
		Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
		Int limit = maxAdviceTips - info->numTips;
		if( choosing &&  rand < limit )
		{
			info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:NoIncome" ) );
			info->numTips++;
		}
		numAvailableTips--;
	}

	//32) Captured any sniped vehicles?
	if( ThePlayerList->getNeutralPlayer()->getAcademyStats()->getVehiclesSniped() > 0 )
	{
		if( !m_vehiclesRecovered )
		{
			availableTips++;
			Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
			Int limit = maxAdviceTips - info->numTips;
			if( choosing &&  rand < limit )
			{
				info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:UnmannedVehicles" ) );
				info->numTips++;
			}
			numAvailableTips--;
		}
	}

	//33) Did the player ever build a "disguisable" unit and never used the disguise ability?
	if( m_disguisableVehiclesBuilt )
	{
		if( !m_vehiclesDisguised )
		{
			availableTips++;
			Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
			Int limit = maxAdviceTips - info->numTips;
			if( choosing &&  rand < limit )
			{
				info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:DisguisedUnits" ) );
				info->numTips++;
			}
			numAvailableTips--;
		}
	}

	//35) Did the player ever create a "Firestorm" with his MiGs or Inferno Cannons?
	if( m_player->getPlayerTemplate() && !m_player->getPlayerTemplate()->getBaseSide().compareNoCase( "China" ) )
	{
		if( !m_firestormsCreated )
		{
			availableTips++;
			Int rand = GameClientRandomValue( 0, numAvailableTips - 1 );
			Int limit = maxAdviceTips - info->numTips;
			if( choosing &&  rand < limit )
			{
				info->advice[ info->numTips ].concat( TheGameText->fetch( "ACADEMY:Firestorm" ) );
				info->numTips++;
			}
			numAvailableTips--;
		}
	}

	if( !choosing && availableTips > 0 )
	{
		//We're done counting available tips, so now that we know the
		//number, we can randomly choose.
		evaluateTier3Advice( info, availableTips );
	}
}

//------------------------------------------------------------------------------------------------
Bool AcademyStats::calculateAcademyAdvice( AcademyAdviceInfo *info )
{
	UnsignedInt maxAdviceTips = MAX_ADVICE_TIPS;

	//-allAdvice feature
	//if( !TheGlobalData->m_allAdvice )
	//{
	//	maxAdviceTips = 1;
	//}

	if( m_unknownSide )
	{
		return FALSE;
	}

	//Sanity
	if( !info )
	{
		DEBUG_CRASH( ("AcademyStats::calculateAcademyAdvice() was passed in null AcademyAdviceInfo.") );
		return FALSE;
	}

	//It's possible to play a perfect game and not need any advice.
	info->numTips = 0;

	//Build the header for each string.
	for( UnsignedInt i = 0; i < maxAdviceTips; i++ )
	{
		info->advice[ i ].format( L"\n\n" );
	}

	//First look at tier 1 basic advice and pick any advice we could benefit from.
	evaluateTier1Advice( info );

	//Now look at tier 2 intermediate advice.
	if( info->numTips < maxAdviceTips )
	{
		evaluateTier2Advice( info );

		if( info->numTips < maxAdviceTips )
		{
			//Finally look at tier 3 advanced advice.
			evaluateTier3Advice( info );
		}
	}

	//Do we have any advice to give?
	return info->numTips != 0;
}

//------------------------------------------------------------------------------------------------
// CRC
//------------------------------------------------------------------------------------------------
void AcademyStats::crc( Xfer *xfer )
{

}

//------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
//------------------------------------------------------------------------------------------------
void AcademyStats::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	xfer->xferUnsignedInt( &m_nextUpdateFrame );
	xfer->xferBool( &m_firstUpdate );
	xfer->xferBool( &m_unknownSide );

	//+-----------------------+
	//| Tier 1 (Basic advice) |
	//+-----------------------+

	//1) Did player build at least one of each structure type available?
	//CUT!!!

	//2) Did player run out of money before building a supply center?
	xfer->xferBool( &m_spentCashBeforeBuildingSupplyCenter );
	xfer->xferUnsignedInt( &m_supplyCentersBuilt );
	xfer->xferUnsignedInt( &m_supplyCenterCost );

	//3) Did player build radar (if applicable)?
	xfer->xferBool( &m_researchedRadar );

	//4) Did player build any dozers/workers?
	xfer->xferUnsignedInt( &m_peonsBuilt );

	//5) Did player ever capture a structure?
	xfer->xferUnsignedInt( &m_structuresCaptured );

	//6) Did player spend any generals points?
	xfer->xferUnsignedInt( &m_generalsPointsSpent );

	//7) Did player ever use a generals power or superweapon?
	xfer->xferUnsignedInt( &m_specialPowersUsed );

	//8) Did player garrison any structures?
	xfer->xferUnsignedInt( &m_structuresGarrisoned );

	//9) How idle was the player in building military units?
	xfer->xferUnsignedInt( &m_idleBuildingUnitsMaxFrames );
	xfer->xferUnsignedInt( &m_lastUnitBuiltFrame );

	//10) Did player drag select units?
	xfer->xferUnsignedInt( &m_dragSelectUnits );

	//11) Did player upgrade anything?
	xfer->xferUnsignedInt( &m_upgradesPurchased );

	//12) Was player out of power for more than 10 minutes?
	xfer->xferUnsignedInt( &m_powerOutMaxFrames );
	xfer->xferUnsignedInt( &m_oldestPowerOutFrame );
	xfer->xferBool			(	&m_hadPowerLastCheck );

	//13) Extra gathers built?
	xfer->xferUnsignedInt( &m_gatherersBuilt );

	//14) Heroes built?
	xfer->xferUnsignedInt( &m_heroesBuilt );

	//+------------------------------+
	//| Tier 2 (Intermediate advice) |
	//+------------------------------+

	//15) Selected a strategy center battle plan?
	xfer->xferBool( &m_hadAStrategyCenter );
	xfer->xferBool( &m_choseAStrategyForCenter );

	//16) Placed units inside tunnel network?
	xfer->xferUnsignedInt( &m_unitsEnteredTunnelNetwork );
	xfer->xferBool( &m_hadATunnelNetwork );

	//17) Player used control groups?
	xfer->xferUnsignedInt( &m_controlGroupsUsed );

	//18) Built secondary income unit (hacker, dropzone, blackmarket)?
	xfer->xferUnsignedInt( &m_secondaryIncomeUnitsBuilt );

	//19) Cleared out garrisoned buildings?
	xfer->xferUnsignedInt( &m_clearedGarrisonedBuildings );

	//20) Did the Player pick up salvage (as GLA)?
	xfer->xferUnsignedInt( &m_salvageCollected );

	//21) Did the player ever use the "Guard" ability?
	xfer->xferUnsignedInt( &m_guardAbilityUsedCount );

	//22) Did the player build more than one Supply Center (that is, did he expand out)?
	//Uses m_supplyCentersBuilt!

	//23) Did the player ever garrison a vehicle?
	//CUT!!!

	//24) Did the player ever use the hotkey to grab all of one unit type (change to use any hotkeys)?
	//CUT!!!

	//+--------------------------+
	//| Tier 3 (Advanced advice) |
	//+--------------------------+

	//25) Did the player use the new alternate interface in the options?
	//Uses TheGlobalData->m_useAlternateMouse

	//26) Player did not use the new "double click location attack move/guard"
	xfer->xferUnsignedInt( &m_doubleClickAttackMoveOrdersGiven );

  //27) Built barracks within 5 minutes?
	xfer->xferBool( &m_builtBarracksWithinFiveMinutes );

	//28) Built war factory within 10 minutes?
	xfer->xferBool( &m_builtWarFactoryWithinTenMinutes );

	//29) Built tech structure within 15 minutes?
	xfer->xferBool( &m_builtTechStructureWithinFifteenMinutes );

	//30) No income for 2 minutes?
	xfer->xferUnsignedInt( &m_lastIncomeFrame );
	xfer->xferUnsignedInt( &m_maxFramesBetweenIncome );

	//31) Did the Player ever use Dozers/Workers to clear out traps/mines/booby traps?
	xfer->xferUnsignedInt( &m_mines ); //Neutral player stat
	xfer->xferUnsignedInt( &m_minesCleared );

	//32) Captured any sniped vehicles?
	xfer->xferUnsignedInt( &m_vehiclesRecovered );
	xfer->xferUnsignedInt( &m_vehiclesSniped );

	//33) Did the player ever build a "disguisable" unit and never used the disguise ability?
	xfer->xferUnsignedInt( &m_disguisableVehiclesBuilt );
	xfer->xferUnsignedInt( &m_vehiclesDisguised );

	//34) Did the player never build a "stealth" upgrade?
	//CUT!!!

	//35) Did the player ever create a "Firestorm" with his MiGs or Inferno Cannons?
	xfer->xferUnsignedInt( &m_firestormsCreated );

}

//------------------------------------------------------------------------------------------------
// Load post process
//------------------------------------------------------------------------------------------------
void AcademyStats::loadPostProcess()
{

}
```
