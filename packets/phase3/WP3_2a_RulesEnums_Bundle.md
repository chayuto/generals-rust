# Bundled Translation Packet: `WP-3.2a` — Phase 3 Core Enum & BitFlags Registry (`12` Packets)

- **Included Packets:** `PKT-125` (`DisabledTypes`), `PKT-126` (`KindOf`), `PKT-127` (`ModelState`), `PKT-129` (`ObjectStatusTypes`), `PKT-135` (`SpecialPowerMaskType`), `PKT-136` (`SpecialPowerType`), `PKT-139` (`ArmorSet`), `PKT-140` (`Damage` enums/flags), `PKT-144` (`LocomotorSet` surface types), `PKT-146` (`WeaponBonusConditionFlags`), `PKT-147` (`WeaponSetFlags`), `PKT-148` (`WeaponSetType`), `PKT-149` (`WeaponStatus`)
- **Build Configuration:** **Zero Hour (`-DRTS_ZEROHOUR=1 -DRTS_GENERALS=0`, `ALLOW_SURRENDER` undefined, `ALLOW_DEMORALIZE` undefined)**
- **Target Rust Files:** `crates/zh_logic/src/rules/{mod.rs, disabled_types.rs, kind_of.rs, model_state.rs, object_status_types.rs, special_power_types.rs, combat_types.rs}`
- **1,000-Seed C++ Golden Vector Oracle:** `docs/golden_vectors/WP3_2a_RulesEnums.json`
- **Gate V2-Golden Test Suite:** `crates/zh_logic/tests/golden_wp3_2a_rules.rs`

## 1. Mandatory Rust Conventions

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

## 2. C++ Source Code (`GeneralsX @ 84a7a20`)

### PKT-125: `DisabledTypes.h` (`GeneralsMD/Code/GameEngine/Include/Common/DisabledTypes.h`)
```cpp
// FILE: DisabledTypes.h //////////////////////////////////////////////////////////////////////////
// Author: Kris Morness, September 2002
// Desc:  Defines all the types of disabled statii any given object can have.
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/BitFlags.h"
#include "Common/BitFlagsIO.h"

//-------------------------------------------------------------------------------------------------
/** Kind of flags for determining groups of things that belong together
	* NOTE: You *MUST* keep this in the same order as the DisabledNames[] below */
//-------------------------------------------------------------------------------------------------
enum DisabledType CPP_11(: Int)
{
	DISABLED_DEFAULT,     //Typical disable -- like systems, things that don't need to run.
	DISABLED_HACKED,      //This unit has been hacked
	DISABLED_EMP,         //This unit has been disabled via electro-magnetic-pulse.
	DISABLED_HELD,	      //Special case -- held means it can fire and isHeld checks to make sure ONLY held is set!
	DISABLED_PARALYZED,   //Battle plans have changed, and unit is confused/paralyzed
	DISABLED_UNMANNED,		//Vehicle is unmanned
	DISABLED_UNDERPOWERED,//Separate from ScriptUnderpowered, the owning player has insufficient power.  Energy status controls this
	DISABLED_FREEFALL,    //This unit has been disabled via being in free fall

  DISABLED_AWESTRUCK,
  DISABLED_BRAINWASHED,
	DISABLED_SUBDUED,			///< Temporarily shut down by Subdual damage
	//These ones are specifically for scripts to enable/reenable!
	DISABLED_SCRIPT_DISABLED,
	DISABLED_SCRIPT_UNDERPOWERED,

	DISABLED_COUNT,

	DISABLED_ANY = 65535		///< Do not use this value for setting disabled types (read-only)
};

typedef BitFlags<DISABLED_COUNT, struct DisabledMaskTypeTag>	DisabledMaskType;

#define MAKE_DISABLED_MASK(k) DisabledMaskType(DisabledMaskType::kInit, (k))
#define MAKE_DISABLED_MASK2(k,a) DisabledMaskType(DisabledMaskType::kInit, (k), (a))
#define MAKE_DISABLED_MASK3(k,a,b) DisabledMaskType(DisabledMaskType::kInit, (k), (a), (b))
#define MAKE_DISABLED_MASK4(k,a,b,c) DisabledMaskType(DisabledMaskType::kInit, (k), (a), (b), (c))
#define MAKE_DISABLED_MASK5(k,a,b,c,d) DisabledMaskType(DisabledMaskType::kInit, (k), (a), (b), (c), (d))

inline Bool TEST_DISABLEDMASK(const DisabledMaskType& m, DisabledType t)
{
	return m.test(t);
}

inline Bool TEST_DISABLEDMASK_ANY(const DisabledMaskType& m, const DisabledMaskType& mask)
{
	return m.anyIntersectionWith(mask);
}

inline Bool TEST_DISABLEDMASK_MULTI(const DisabledMaskType& m, const DisabledMaskType& mustBeSet, const DisabledMaskType& mustBeClear)
{
	return m.testSetAndClear(mustBeSet, mustBeClear);
}

inline Bool DISABLEDMASK_ANY_SET(const DisabledMaskType& m)
{
	return m.any();
}

inline void CLEAR_DISABLEDMASK(DisabledMaskType& m)
{
	m.clear();
}

inline void SET_ALL_DISABLEDMASK_BITS(DisabledMaskType& m)
{
	m.clear();
	m.flip();
}

inline void FLIP_DISABLEDMASK(DisabledMaskType& m)
{
	m.flip();
}



// defined in Common/System/DisabledTypes.cpp
extern DisabledMaskType DISABLEDMASK_NONE;	// inits to all zeroes
extern DisabledMaskType DISABLEDMASK_ALL;		// inits to all bits set.
```

### PKT-125: `DisabledTypes.cpp` (`GeneralsMD/Code/GameEngine/Source/Common/System/DisabledTypes.cpp`)
```cpp
#include "PreRTS.h"

#include "Common/DisabledTypes.h"
#include "Common/BitFlagsIO.h"

template<>
const char* const DisabledMaskType::s_bitNameList[] =
{
	"DEFAULT",
	"DISABLED_HACKED",
	"DISABLED_EMP",
	"DISABLED_HELD",
	"DISABLED_PARALYZED",
	"DISABLED_UNMANNED",
	"DISABLED_UNDERPOWERED",
	"DISABLED_FREEFALL",

  "DISABLED_AWESTRUCK",
  "DISABLED_BRAINWASHED",
	"DISABLED_SUBDUED",

	"DISABLED_SCRIPT_DISABLED",
	"DISABLED_SCRIPT_UNDERPOWERED",

	nullptr
};
static_assert(ARRAY_SIZE(DisabledMaskType::s_bitNameList) == DisabledMaskType::NumBits + 1, "Incorrect array size");

DisabledMaskType DISABLEDMASK_NONE;	// inits to all zeroes
DisabledMaskType DISABLEDMASK_ALL(DisabledMaskType::kInitSetAll);
```

### PKT-126: `KindOf.h` (`GeneralsMD/Code/GameEngine/Include/Common/KindOf.h`)
```cpp
// FILE: KindOf.h //////////////////////////////////////////////////////////////////////////
// Author: Steven Johnson, Dec 2001
// Desc:
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/BitFlags.h"
#include "Common/BitFlagsIO.h"

//-------------------------------------------------------------------------------------------------
/** Kind of flags for determining groups of things that belong together
	* NOTE: You *MUST* keep this in the same order as the KindOfNames[] below */
//-------------------------------------------------------------------------------------------------
enum KindOfType CPP_11(: Int)
{
	KINDOF_INVALID = -1,

	KINDOF_OBSTACLE,								///< an obstacle to land-based pathfinders
	KINDOF_SELECTABLE,							///< Actually means MOUSE-INTERACTABLE (doesn't mean you can select it!)
	KINDOF_IMMOBILE,								///< fixed in location
	KINDOF_CAN_ATTACK,							///< can attack
	KINDOF_STICK_TO_TERRAIN_SLOPE,	///< should be stuck at ground level, aligned to terrain slope. requires that IMMOBILE bit is also set.
	KINDOF_CAN_CAST_REFLECTIONS,		///< can cast reflections in water
	KINDOF_SHRUBBERY,								///< tree, bush, etc.
	KINDOF_STRUCTURE,								///< structure of some sort (buildable or not)
	KINDOF_INFANTRY,								///< unit like soldier etc
	KINDOF_VEHICLE,									///< unit like tank, jeep, plane, helicopter, etc.
	KINDOF_AIRCRAFT,								///< unit like plane, helicopter, etc., that is predominantly a flyer. (hovercraft are NOT aircraft)
	KINDOF_HUGE_VEHICLE,						///< unit that is, technically, a vehicle, but WAY larger than normal (eg, Overlord)
	KINDOF_DOZER,										///< a dozer
	KINDOF_HARVESTER,               ///< a harvester
	KINDOF_COMMANDCENTER,						///< a command center
#ifdef ALLOW_SURRENDER
	KINDOF_PRISON,									///< a prison detention center kind of thing
	KINDOF_COLLECTS_PRISON_BOUNTY,	///< when prisoners are delivered to these, the player gets money
	KINDOF_POW_TRUCK,								///< a pow truck can pick up and return prisoners
#endif
	KINDOF_LINEBUILD,								///< wall-type thing that is built in a line
	KINDOF_SALVAGER,								///< something that can create and use Salvage Crates
	KINDOF_WEAPON_SALVAGER,					///< subset of salvager that can get weapon upgrades from salvage
	KINDOF_TRANSPORT,								///< a true transport (has TransportContain)
	KINDOF_BRIDGE,									///< a Bridge. (special structure)
	KINDOF_LANDMARK_BRIDGE,					///< a landmark bridge (special bridge that isn't resizable)
	KINDOF_BRIDGE_TOWER,						///< a bridge tower that we can target for bridge destruction
	KINDOF_PROJECTILE,							///< Instead of being a ground or air unit, this object is special
	KINDOF_PRELOAD,									///< all model data will be preloaded even if not on map
	KINDOF_NO_GARRISON,							///< unit may not garrison bldgs, even if infantry bit is set
	KINDOF_WAVEGUIDE,								///< water wave object
	KINDOF_WAVE_EFFECT,							///< wave effect point
	KINDOF_NO_COLLIDE,							///< Never collide with or be collided with
	KINDOF_REPAIR_PAD,							///< is a repair pad object that can repair other machines
	KINDOF_HEAL_PAD,								///< is a heal pad object that can heal flesh and bone units
	KINDOF_STEALTH_GARRISON,				/** enemy teams can't tell that unit is in building.. and if they
																		garrison that building, they stealth unit will eject. */
	KINDOF_CASH_GENERATOR,					///< used to check if the unit generates cash... checked by cash hackers and whatever else comes up
#if RTS_GENERALS
	KINDOF_AIRFIELD,								///< unit has a runway that planes can takeoff/land on
#endif
	KINDOF_DRAWABLE_ONLY,						///< template is used only to create drawables (not Objects)
	KINDOF_MP_COUNT_FOR_VICTORY,		///< If a player loses all his buildings that have this kindof in a multiplayer game, he loses.
	KINDOF_REBUILD_HOLE,						///< a GLA rebuild hole
	KINDOF_SCORE,										///< Object counts for Multiplayer scores, and short-game calculations (for buildings)
	KINDOF_SCORE_CREATE,						///< Object only counts for multiplayer score for creation.
	KINDOF_SCORE_DESTROY,						///< Object only counts for multiplayer score for destruction.
	KINDOF_NO_HEAL_ICON,						///< do not ever display healing icons on these objects
	KINDOF_CAN_RAPPEL,							///< can rappel. duh.
	KINDOF_PARACHUTABLE,						///< parachutable object
#ifdef ALLOW_SURRENDER
	KINDOF_CAN_SURRENDER,						///< object that can surrender
#endif
	KINDOF_CAN_BE_REPULSED,					///< object that runs away from a repulsor object.
	KINDOF_MOB_NEXUS,					      ///< object that coordinates the members of a mob (i.e. GLAInfantryAngryMob)
	KINDOF_IGNORED_IN_GUI,					///< object that is the members of a mob (i.e. GLAInfantryAngryMob)
	KINDOF_CRATE,										///< a bonus crate
	KINDOF_CAPTURABLE,							///< is "capturable" even if not an enemy (should generally be used only for structures, eg, Tech bldgs)
	KINDOF_CLEARED_BY_BUILD,				///< is auto-cleared from the map when built over via construction
	KINDOF_SMALL_MISSILE,						///< Missile object: ONLY USED FOR ANTI-MISSILE TARGETING PURPOSES! Keep using PROJECTILE!
	KINDOF_ALWAYS_VISIBLE,					///< is never obscured by fog of war or shroud.  mostly for UI feedback objects.
	KINDOF_UNATTACKABLE,						///< You cannot target this thing, it probably doesn't really exist
	KINDOF_MINE,										///< a landmine. (possibly also extend to Col. Burton timed charges?)
	KINDOF_CLEANUP_HAZARD,					///< radiation and bio-poison are samples of area conditions that can be cleaned up (or avoided)
	KINDOF_PORTABLE_STRUCTURE,				///< Flag to identify building like subobjects an Overlord is allowed to Contain.
	KINDOF_ALWAYS_SELECTABLE,				///< is never unselectable (even if effectively dead).  mostly for UI feedback objects.
	KINDOF_ATTACK_NEEDS_LINE_OF_SIGHT,				///< Unit has to have clear line of sight (los) to attack.
	KINDOF_WALK_ON_TOP_OF_WALL, 		///< Units can walk on top of a wall made of these kind of objects.
	KINDOF_DEFENSIVE_WALL,					///< wall can't be driven thru, even if crusher, so pathfinder must path around it
	KINDOF_FS_POWER,								///< Faction structure power building
	KINDOF_FS_FACTORY,							///< Faction structure power building
	KINDOF_FS_BASE_DEFENSE,					///< Faction structure base defense
	KINDOF_FS_TECHNOLOGY,						///< Faction structure technology building
	KINDOF_AIRCRAFT_PATH_AROUND,		///< Tall enough that aircraft need to path around this.
	KINDOF_LOW_OVERLAPPABLE,				///< When overlapped, things always overlap at a 'low' height rather than our object geom
	KINDOF_FORCEATTACKABLE,					///< unit is always attackable via force-attack, even if not selectable
	KINDOF_AUTO_RALLYPOINT,					///< When immobile-structure-object is selected, left clicking on ground will set new rally point without requiring command button.
	KINDOF_TECH_BUILDING,						///< Neutral tech building - Oil derrick, Hospital, Radio Station, Refinery.
	KINDOF_POWERED,									///< This object gets the Underpowered disabled condition when its owning player has power consumption exceed supply
	KINDOF_PRODUCED_AT_HELIPAD,			///< ugh... hacky fix for comanche. (srj)
	KINDOF_DRONE,										///< Object drone type -- used for filtering them out of battle plan bonuses, making un-snipable, and whatever else may come up.
	KINDOF_CAN_SEE_THROUGH_STRUCTURE,///< Structure does not block line of sight.
	KINDOF_BALLISTIC_MISSILE,				///< Large ballistic missiles that are specifically large enough to be targetted by base defenses.
	KINDOF_CLICK_THROUGH,						///< Objects with this will never be picked by mouse interactions!
	KINDOF_SUPPLY_SOURCE_ON_PREVIEW,///< Any thing that we can get "supplies" from that we want to show up on the map preview
	KINDOF_PARACHUTE,								///< it's a parachute
	KINDOF_GARRISONABLE_UNTIL_DESTROYED, ///< Object is capable of garrisoning troops until completely destroyed.
	KINDOF_BOAT,										///< It's a boat!
	KINDOF_IMMUNE_TO_CAPTURE,				///< Under no circumstances can this building ever be captured.
	KINDOF_HULK,										///< Hulk types so we can do special things to them via scripts or other things that may come up.
	KINDOF_SHOW_PORTRAIT_WHEN_CONTROLLED,	///< Only shows portraits when controlled.
	KINDOF_SPAWNS_ARE_THE_WEAPONS,	///< Evaluate the spawn slaves as this object's weapons.
	KINDOF_CANNOT_BUILD_NEAR_SUPPLIES,	///< you can't be built "too close" to anything that provides supplies
	KINDOF_SUPPLY_SOURCE,						///< this object provides supplies
	KINDOF_REVEAL_TO_ALL,						///< this object reveals shroud for all players
	KINDOF_DISGUISER,								///< This object has the ability to disguise.
	KINDOF_INERT,										///< this object shouldn't be considered for any sort of interaction with any player.
	KINDOF_HERO,										///< Any of the single-instance infantry, JarmenKell, BlackLotus, ColonelBurton
	KINDOF_IGNORES_SELECT_ALL,			///< Too late to figure out intelligently if something should respond to a Select All command
	KINDOF_DONT_AUTO_CRUSH_INFANTRY,					///< These units don't try to crush the infantry if ai.
	// TheSuperHackers @info Added in Zero Hour:
	KINDOF_CLIFF_JUMPER,						///< Can't climb cliffs, but can jump off of them.
	KINDOF_FS_SUPPLY_DROPZONE,						///< A supply dropzone.
	KINDOF_FS_SUPERWEAPON,					///< A superweapon structure like a nuke silo, particle uplink cannon, scudstorm.
	KINDOF_FS_BLACK_MARKET,					///< Is this object a black market?
	KINDOF_FS_SUPPLY_CENTER,				///< Is this object a supply center?
	KINDOF_FS_STRATEGY_CENTER,			///< Is this object a strategy center?
	KINDOF_MONEY_HACKER,	///< Unit that generates money from air.  Needed for things that directly power them up.
	KINDOF_ARMOR_SALVAGER,					///< subset of salvager that can get armor upgrades from salvage
  KINDOF_REVEALS_ENEMY_PATHS,     ///< like the listening outpost... when selected, any enemy drawable will draw show paths when moused over
	KINDOF_BOOBY_TRAP,							///< A sticky bomb that gets set off by 5 random and unrelated events.
	KINDOF_FS_FAKE,									///< Fake structure!
	KINDOF_FS_INTERNET_CENTER,			///< Internet Center.
  KINDOF_BLAST_CRATER,            ///< deeply gouges out the terrain under object footprint
	KINDOF_PROP,										///< A prop, visual only, doesn't interact with other objects (rock, street sign, inert fire hydrant)
	KINDOF_OPTIMIZED_TREE,					///< An optimized, client side only tree.  (The only good kind of tree. jba)
	KINDOF_FS_ADVANCED_TECH,				///< Represents each faction's advanced techtree building -- strategy center, propaganda center, and palace.
	KINDOF_FS_BARRACKS,							///< A barracks
	KINDOF_FS_WARFACTORY,						///< A war factory or arms dealer.
	KINDOF_FS_AIRFIELD,							///< An airfield.
	KINDOF_AIRCRAFT_CARRIER,				///< An aircraft carrier.
	KINDOF_NO_SELECT,								///< Can't select it but you can mouse over it to see it's health (drones!)
	KINDOF_REJECT_UNMANNED,					///< Unit cannot enter an unmanned vehicle.
	KINDOF_CANNOT_RETALIATE,				///< Unit will not retaliate if asked.
	KINDOF_TECH_BASE_DEFENSE,				///< Tech Building that acts as base defence when captured
  KINDOF_EMP_HARDENED,            ///< Like a delivery plane (B52, B3, CargoPlane,etc.)  or a SpectreGunship, which sort-of IS the weapon...
	KINDOF_DEMOTRAP,								///< Added strictly only for disarming purposes. They don't act like mines which have rendering and selection implications!
	KINDOF_CONSERVATIVE_BUILDING,		///< Conservative structures aren't considered part of your base for sneak attack boundary calculations...
	KINDOF_IGNORE_DOCKING_BONES,		///< Structure will not look up docking bones. Patch 1.03 hack.

	// TheSuperHackers @info New kinds for Mods

	KINDOF_NO_ATTACK_WARNING,				///< does not trigger the under attack radar/EVA warning when taking damage

	KINDOF_COUNT,										// total number of kindofs
	KINDOF_FIRST = 0,
};

typedef BitFlags<KINDOF_COUNT, struct KindOfMaskTypeTag>	KindOfMaskType;

#define MAKE_KINDOF_MASK(k) KindOfMaskType(KindOfMaskType::kInit, (k))

inline Bool TEST_KINDOFMASK(const KindOfMaskType& m, KindOfType t)
{
	return m.test(t);
}

inline Bool TEST_KINDOFMASK_ANY(const KindOfMaskType& m, const KindOfMaskType& mask)
{
	return m.anyIntersectionWith(mask);
}

inline Bool TEST_KINDOFMASK_MULTI(const KindOfMaskType& m, const KindOfMaskType& mustBeSet, const KindOfMaskType& mustBeClear)
{
	return m.testSetAndClear(mustBeSet, mustBeClear);
}

inline Bool KINDOFMASK_ANY_SET(const KindOfMaskType& m)
{
	return m.any();
}

inline void CLEAR_KINDOFMASK(KindOfMaskType& m)
{
	m.clear();
}

inline void SET_ALL_KINDOFMASK_BITS(KindOfMaskType& m)
{
	m.clear();
	m.flip();
}

inline void FLIP_KINDOFMASK(KindOfMaskType& m)
{
	m.flip();
}

// defined in Common/System/Kindof.cpp
extern KindOfMaskType KINDOFMASK_NONE;	// inits to all zeroes
extern KindOfMaskType KINDOFMASK_FS;		// Initializes all FS types for faction structures.
```

### PKT-126: `KindOf.cpp` (`GeneralsMD/Code/GameEngine/Source/Common/System/KindOf.cpp`)
```cpp
#include "PreRTS.h"

#include "Common/KindOf.h"
#include "Common/BitFlagsIO.h"

template<>
const char* const KindOfMaskType::s_bitNameList[] =
{
	"OBSTACLE",
	"SELECTABLE",
	"IMMOBILE",
	"CAN_ATTACK",
	"STICK_TO_TERRAIN_SLOPE",
	"CAN_CAST_REFLECTIONS",
	"SHRUBBERY",
	"STRUCTURE",
	"INFANTRY",
	"VEHICLE",
	"AIRCRAFT",
	"HUGE_VEHICLE",
	"DOZER",
	"HARVESTER",
	"COMMANDCENTER",
#ifdef ALLOW_SURRENDER
	"PRISON",
	"COLLECTS_PRISON_BOUNTY",
	"POW_TRUCK",
#endif
	"LINEBUILD",
	"SALVAGER",
	"WEAPON_SALVAGER",
	"TRANSPORT",
	"BRIDGE",
	"LANDMARK_BRIDGE",
	"BRIDGE_TOWER",
	"PROJECTILE",
	"PRELOAD",
	"NO_GARRISON",
	"WAVEGUIDE",
	"WAVE_EFFECT",
	"NO_COLLIDE",
	"REPAIR_PAD",
	"HEAL_PAD",
	"STEALTH_GARRISON",
	"CASH_GENERATOR",
#if RTS_GENERALS
	"AIRFIELD",
#endif
	"DRAWABLE_ONLY",
	"MP_COUNT_FOR_VICTORY",
	"REBUILD_HOLE",
	"SCORE",
	"SCORE_CREATE",
	"SCORE_DESTROY",
	"NO_HEAL_ICON",
	"CAN_RAPPEL",
	"PARACHUTABLE",
#ifdef ALLOW_SURRENDER
	"CAN_SURRENDER",
#endif
	"CAN_BE_REPULSED",
	"MOB_NEXUS",
	"IGNORED_IN_GUI",
	"CRATE",
	"CAPTURABLE",
	"CLEARED_BY_BUILD",
	"SMALL_MISSILE",
	"ALWAYS_VISIBLE",
	"UNATTACKABLE",
	"MINE",
	"CLEANUP_HAZARD",
	"PORTABLE_STRUCTURE",
	"ALWAYS_SELECTABLE",
	"ATTACK_NEEDS_LINE_OF_SIGHT",
	"WALK_ON_TOP_OF_WALL",
	"DEFENSIVE_WALL",
	"FS_POWER",
	"FS_FACTORY",
	"FS_BASE_DEFENSE",
	"FS_TECHNOLOGY",
	"AIRCRAFT_PATH_AROUND",
	"LOW_OVERLAPPABLE",
	"FORCEATTACKABLE",
	"AUTO_RALLYPOINT",
	"TECH_BUILDING",
	"POWERED",
	"PRODUCED_AT_HELIPAD",
	"DRONE",
	"CAN_SEE_THROUGH_STRUCTURE",
	"BALLISTIC_MISSILE",
	"CLICK_THROUGH",
	"SUPPLY_SOURCE_ON_PREVIEW",
	"PARACHUTE",
	"GARRISONABLE_UNTIL_DESTROYED",
	"BOAT",
	"IMMUNE_TO_CAPTURE",
	"HULK",
	"SHOW_PORTRAIT_WHEN_CONTROLLED",
	"SPAWNS_ARE_THE_WEAPONS",
	"CANNOT_BUILD_NEAR_SUPPLIES",
	"SUPPLY_SOURCE",
	"REVEAL_TO_ALL",
	"DISGUISER",
	"INERT",
	"HERO",
	"IGNORES_SELECT_ALL",
	"DONT_AUTO_CRUSH_INFANTRY",
	"CLIFF_JUMPER",
	"FS_SUPPLY_DROPZONE",
	"FS_SUPERWEAPON",
	"FS_BLACK_MARKET",
	"FS_SUPPLY_CENTER",
	"FS_STRATEGY_CENTER",
	"MONEY_HACKER",
	"ARMOR_SALVAGER",
  "REVEALS_ENEMY_PATHS",
	"BOOBY_TRAP",
	"FS_FAKE",
	"FS_INTERNET_CENTER",
  "BLAST_CRATER",
	"PROP",
	"OPTIMIZED_TREE",
	"FS_ADVANCED_TECH",
	"FS_BARRACKS",
	"FS_WARFACTORY",
	"FS_AIRFIELD",
	"AIRCRAFT_CARRIER",
	"NO_SELECT",
	"REJECT_UNMANNED",
	"CANNOT_RETALIATE",
	"TECH_BASE_DEFENSE",
	"EMP_HARDENED",
	"DEMOTRAP",
	"CONSERVATIVE_BUILDING",
	"IGNORE_DOCKING_BONES",
	"NO_ATTACK_WARNING",

	nullptr
};
static_assert(ARRAY_SIZE(KindOfMaskType::s_bitNameList) == KindOfMaskType::NumBits + 1, "Incorrect array size");

static const Int fsList[] = {
	KINDOF_FS_FACTORY,
	KINDOF_FS_BASE_DEFENSE,
	KINDOF_FS_TECHNOLOGY,
	KINDOF_FS_SUPPLY_DROPZONE,
	KINDOF_FS_SUPERWEAPON,
	KINDOF_FS_BLACK_MARKET,
	KINDOF_FS_SUPPLY_CENTER,
	KINDOF_FS_STRATEGY_CENTER,
	KINDOF_FS_FAKE,
	KINDOF_FS_INTERNET_CENTER,
	KINDOF_FS_ADVANCED_TECH,
	KINDOF_FS_BARRACKS,
	KINDOF_FS_WARFACTORY,
	KINDOF_FS_AIRFIELD,
};

KindOfMaskType KINDOFMASK_NONE;	// inits to all zeroes
KindOfMaskType KINDOFMASK_FS(KindOfMaskType::kInit, fsList, ARRAY_SIZE(fsList));
```

### PKT-127: `ModelState.h` (`GeneralsMD/Code/GameEngine/Include/Common/ModelState.h`)
```cpp
#pragma once

#include "Common/INI.h"
#include "Common/BitFlags.h"
#include "Common/BitFlagsIO.h"

//-------------------------------------------------------------------------------------------------

/**
	THE PROBLEM
	-----------

	-- there are lots of different states. (consider that structures can be: day/night, snow/nosnow,
	powered/not, garrisoned/empty, damaged/not... you do the math.)

	-- some states are mutually exclusive (idle vs. moving), others are not (snow/nosnow, day/night).
	generally, humanoid units have mostly the former, while structure units have mostly the latter.
	The current ModelState system really only supports the mutually-exclusive states well.

	-- we'd rather not have to specify every state in the INI files, BUT we do want to be able
	to intelligently choose the best model for a given state, whether or not a "real" state exists for it.

	-- it would be desirable to have a unified way of representing "ModelState" so that we don't
	have multiple similar-yet-different systems.

	YUCK, WHAT NOW
	--------------

	Let's represent the Model State with two distinct pieces:

	-- an "ActionState" piece, representing the mutually-exclusive states, which are almost
		always an action of some sort

	-- and a "ConditionState" piece, which is a set of bitflags to indicate the static "condition"
		of the model.

	Note that these are usually set independently in code, but they are lumped together in order to
	determine the actual model to be used.

	(Let's require all objects would be required to have an "Idle" ActionState, which is the
	normal, just-sitting there condition.)

	From a code point of view, this becomes an issue of requesting a certain state, and
	finding the best-fit match for it. So, what are the rules for finding a good match?

	-- Action states must match exactly. If the desired action state is not found, then the
	IDLE state is substituted (but this should generally be considered an error condition).

	-- Condition states choose the match with the closest match among the "Condition" bits in the
	INI file, based on satisfying the most of the "required" conditions and the fewest of the
	"forbidden" conditions.

*/

#define NUM_MODELCONDITION_DOOR_STATES 4

//-------------------------------------------------------------------------------------------------
// IMPORTANT NOTE: you should endeavor to set up states such that the most "normal"
// state is defined by the bit being off. That is, the typical "normal" condition
// has all condition flags set to zero.
enum ModelConditionFlagType CPP_11(: Int)
{
	MODELCONDITION_INVALID = -1,

//
// Note: these values are saved in save files, so you MUST NOT REMOVE OR CHANGE
// existing values!
//
	MODELCONDITION_TOPPLED,
	MODELCONDITION_FRONTCRUSHED,
	MODELCONDITION_BACKCRUSHED,
	MODELCONDITION_DAMAGED,
	MODELCONDITION_REALLY_DAMAGED,
	MODELCONDITION_RUBBLE,
	MODELCONDITION_SPECIAL_DAMAGED,
	MODELCONDITION_NIGHT,
	MODELCONDITION_SNOW,
	MODELCONDITION_PARACHUTING,
	MODELCONDITION_GARRISONED,
	MODELCONDITION_ENEMYNEAR,
	MODELCONDITION_WEAPONSET_VETERAN,
	MODELCONDITION_WEAPONSET_ELITE,
	MODELCONDITION_WEAPONSET_HERO,
	MODELCONDITION_WEAPONSET_CRATEUPGRADE_ONE,
	MODELCONDITION_WEAPONSET_CRATEUPGRADE_TWO,
	MODELCONDITION_WEAPONSET_PLAYER_UPGRADE,
	MODELCONDITION_DOOR_1_OPENING,
	MODELCONDITION_DOOR_1_CLOSING,
	MODELCONDITION_DOOR_1_WAITING_OPEN,
	MODELCONDITION_DOOR_1_WAITING_TO_CLOSE,
	MODELCONDITION_DOOR_2_OPENING,
	MODELCONDITION_DOOR_2_CLOSING,
	MODELCONDITION_DOOR_2_WAITING_OPEN,
	MODELCONDITION_DOOR_2_WAITING_TO_CLOSE,
	MODELCONDITION_DOOR_3_OPENING,
	MODELCONDITION_DOOR_3_CLOSING,
	MODELCONDITION_DOOR_3_WAITING_OPEN,
	MODELCONDITION_DOOR_3_WAITING_TO_CLOSE,
	MODELCONDITION_DOOR_4_OPENING,
	MODELCONDITION_DOOR_4_CLOSING,
	MODELCONDITION_DOOR_4_WAITING_OPEN,
	MODELCONDITION_DOOR_4_WAITING_TO_CLOSE,
	MODELCONDITION_ATTACKING,		//Simply set when a unit is fighting -- terrorist moving with a target will flail arms like a psycho.
	MODELCONDITION_PREATTACK_A,		//Use for pre-attack animations (like aiming, pulling out a knife, or detonating explosives).
	MODELCONDITION_FIRING_A,
	MODELCONDITION_BETWEEN_FIRING_SHOTS_A,
	MODELCONDITION_RELOADING_A,
	MODELCONDITION_PREATTACK_B,		//Use for pre-attack animations (like aiming, pulling out a knife, or detonating explosives).
	MODELCONDITION_FIRING_B,
	MODELCONDITION_BETWEEN_FIRING_SHOTS_B,
	MODELCONDITION_RELOADING_B,
	MODELCONDITION_PREATTACK_C,		//Use for pre-attack animations (like aiming, pulling out a knife, or detonating explosives).
	MODELCONDITION_FIRING_C,
	MODELCONDITION_BETWEEN_FIRING_SHOTS_C,
	MODELCONDITION_RELOADING_C,
	MODELCONDITION_TURRET_ROTATE,
	MODELCONDITION_POST_COLLAPSE,
	MODELCONDITION_MOVING,
	MODELCONDITION_DYING,
	MODELCONDITION_AWAITING_CONSTRUCTION,
	MODELCONDITION_PARTIALLY_CONSTRUCTED,
	MODELCONDITION_ACTIVELY_BEING_CONSTRUCTED,
	MODELCONDITION_PRONE,
	MODELCONDITION_FREEFALL,
	MODELCONDITION_ACTIVELY_CONSTRUCTING,
	MODELCONDITION_CONSTRUCTION_COMPLETE,
	MODELCONDITION_RADAR_EXTENDING,
	MODELCONDITION_RADAR_UPGRADED,
	MODELCONDITION_PANICKING,	// yes, it's spelled with a "k". look it up.
	MODELCONDITION_AFLAME,
	MODELCONDITION_SMOLDERING,
	MODELCONDITION_BURNED,
	MODELCONDITION_DOCKING,						///< This encloses the whole time you are Entering, Actioning, and Exiting a dock
	MODELCONDITION_DOCKING_BEGINNING,	///< From Enter to Action
	MODELCONDITION_DOCKING_ACTIVE,		///< From Action to Exit
	MODELCONDITION_DOCKING_ENDING,		///< Exit all the way to next enter (use only animations that end with this)
	MODELCONDITION_CARRYING,
	MODELCONDITION_FLOODED,
	MODELCONDITION_LOADED,				// loaded woot! ... like a transport is loaded
	MODELCONDITION_JETAFTERBURNER,// shows "flames" for extra motive force (eg, when taking off)
	MODELCONDITION_JETEXHAUST,		// shows "exhaust" for motive force
	MODELCONDITION_PACKING,				// packs an object
	MODELCONDITION_UNPACKING,			// unpacks an object
	MODELCONDITION_DEPLOYED,			// a deployed object state
	MODELCONDITION_OVER_WATER,		// Units that can go over water want cool effects for doing so
	MODELCONDITION_POWER_PLANT_UPGRADED,	// to show special control rods on the cold fusion plant
	MODELCONDITION_CLIMBING,	//For units climbing up or down cliffs.
	MODELCONDITION_SOLD,					// object is being sold
#ifdef ALLOW_SURRENDER
	MODELCONDITION_SURRENDER,			//When units surrender...
#endif
	MODELCONDITION_RAPPELLING,
	MODELCONDITION_ARMED,					// armed like a mine or bomb is armed (not like a human is armed)
	MODELCONDITION_POWER_PLANT_UPGRADING,	// while special control rods on the cold fusion plant are extending

	//Special model conditions work as following:
	//Something turns it on... but a timer in the object will turn them off after a given
	//amount of time. If you add any more special animations, then you'll need to add the
	//code to turn off the state.
	MODELCONDITION_SPECIAL_CHEERING,	//When units do a victory cheer (or player initiated cheer).

	MODELCONDITION_CONTINUOUS_FIRE_SLOW,
	MODELCONDITION_CONTINUOUS_FIRE_MEAN,
	MODELCONDITION_CONTINUOUS_FIRE_FAST,

	MODELCONDITION_RAISING_FLAG,
	MODELCONDITION_CAPTURED,

	MODELCONDITION_EXPLODED_FLAILING,
	MODELCONDITION_EXPLODED_BOUNCING,
	MODELCONDITION_SPLATTED,

	// this is an easier-to-use variant on the whole FIRING_A deal...
	// these bits are set if firing, reloading, between shots, or preattack.
	MODELCONDITION_USING_WEAPON_A,
	MODELCONDITION_USING_WEAPON_B,
	MODELCONDITION_USING_WEAPON_C,

	MODELCONDITION_PREORDER,

	MODELCONDITION_CENTER_TO_LEFT,
	MODELCONDITION_LEFT_TO_CENTER,
	MODELCONDITION_CENTER_TO_RIGHT,
	MODELCONDITION_RIGHT_TO_CENTER,

	MODELCONDITION_RIDER1,	//Added these for different riders
	MODELCONDITION_RIDER2,
	MODELCONDITION_RIDER3,
	MODELCONDITION_RIDER4,
	MODELCONDITION_RIDER5,
	MODELCONDITION_RIDER6,
	MODELCONDITION_RIDER7,
	MODELCONDITION_RIDER8,

	MODELCONDITION_STUNNED_FLAILING, // Daniel Teh's idea, added by Lorenzen, 5/28/03
	MODELCONDITION_STUNNED,
	MODELCONDITION_SECOND_LIFE,
	MODELCONDITION_JAMMED,	///< Jammed as in missile jammed by ECM
	MODELCONDITION_ARMORSET_CRATEUPGRADE_ONE,
	MODELCONDITION_ARMORSET_CRATEUPGRADE_TWO,

	MODELCONDITION_USER_1,		///< Wildcard flag to use with upgrade modules or other random little things
	MODELCONDITION_USER_2,

	MODELCONDITION_DISGUISED,
//
// Note: these values are saved in save files, so you MUST NOT REMOVE OR CHANGE
// existing values!
//

	MODELCONDITION_COUNT,
	MODELCONDITION_FIRST = 0,
};

//-------------------------------------------------------------------------------------------------

typedef BitFlags<MODELCONDITION_COUNT, struct ModelConditionFlagsTag> ModelConditionFlags;

#define MAKE_MODELCONDITION_MASK(k) ModelConditionFlags(ModelConditionFlags::kInit, (k))
#define MAKE_MODELCONDITION_MASK2(k,a) ModelConditionFlags(ModelConditionFlags::kInit, (k), (a))
#define MAKE_MODELCONDITION_MASK3(k,a,b) ModelConditionFlags(ModelConditionFlags::kInit, (k), (a), (b))
#define MAKE_MODELCONDITION_MASK4(k,a,b,c) ModelConditionFlags(ModelConditionFlags::kInit, (k), (a), (b), (c))
#define MAKE_MODELCONDITION_MASK5(k,a,b,c,d) ModelConditionFlags(ModelConditionFlags::kInit, (k), (a), (b), (c), (d))

//-------------------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------------------
```

### PKT-127 & PKT-139: `BitFlags.cpp` (`ModelConditionFlags` & `ArmorSetFlags`) (`GeneralsMD/Code/GameEngine/Source/Common/BitFlags.cpp`)
```cpp
// FILE: BitFlags.cpp ///////////////////////////////////////////////////////////
//
// Used to set detail levels of various game systems.
//  Steven Johnson, Sept 2002
//
//
///////////////////////////////////////////////////////////////////////////////

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/BitFlags.h"
#include "Common/BitFlagsIO.h"
#include "Common/ModelState.h"
#include "GameLogic/ArmorSet.h"

template<>
const char* const ModelConditionFlags::s_bitNameList[] =
{
	"TOPPLED",
	"FRONTCRUSHED",
	"BACKCRUSHED",
	"DAMAGED",
	"REALLYDAMAGED",
	"RUBBLE",
	"SPECIAL_DAMAGED",
	"NIGHT",
	"SNOW",
	"PARACHUTING",
	"GARRISONED",
	"ENEMYNEAR",
	"WEAPONSET_VETERAN",
	"WEAPONSET_ELITE",
	"WEAPONSET_HERO",
	"WEAPONSET_CRATEUPGRADE_ONE",
	"WEAPONSET_CRATEUPGRADE_TWO",
	"WEAPONSET_PLAYER_UPGRADE",
	"DOOR_1_OPENING",
	"DOOR_1_CLOSING",
	"DOOR_1_WAITING_OPEN",
	"DOOR_1_WAITING_TO_CLOSE",
	"DOOR_2_OPENING",
	"DOOR_2_CLOSING",
	"DOOR_2_WAITING_OPEN",
	"DOOR_2_WAITING_TO_CLOSE",
	"DOOR_3_OPENING",
	"DOOR_3_CLOSING",
	"DOOR_3_WAITING_OPEN",
	"DOOR_3_WAITING_TO_CLOSE",
	"DOOR_4_OPENING",
	"DOOR_4_CLOSING",
	"DOOR_4_WAITING_OPEN",
	"DOOR_4_WAITING_TO_CLOSE",
	"ATTACKING",
	"PREATTACK_A",
	"FIRING_A",
	"BETWEEN_FIRING_SHOTS_A",
	"RELOADING_A",
	"PREATTACK_B",
	"FIRING_B",
	"BETWEEN_FIRING_SHOTS_B",
	"RELOADING_B",
	"PREATTACK_C",
	"FIRING_C",
	"BETWEEN_FIRING_SHOTS_C",
	"RELOADING_C",
	"TURRET_ROTATE",
	"POST_COLLAPSE",
	"MOVING",
	"DYING",
	"AWAITING_CONSTRUCTION",
	"PARTIALLY_CONSTRUCTED",
	"ACTIVELY_BEING_CONSTRUCTED",
	"PRONE",
	"FREEFALL",
	"ACTIVELY_CONSTRUCTING",
	"CONSTRUCTION_COMPLETE",
	"RADAR_EXTENDING",
	"RADAR_UPGRADED",
	"PANICKING",	// yes, it's spelled with a "k". look it up.
	"AFLAME",
	"SMOLDERING",
	"BURNED",
	"DOCKING",
	"DOCKING_BEGINNING",
	"DOCKING_ACTIVE",
	"DOCKING_ENDING",
	"CARRYING",
	"FLOODED",
	"LOADED",
	"JETAFTERBURNER",
	"JETEXHAUST",
	"PACKING",
	"UNPACKING",
	"DEPLOYED",
	"OVER_WATER",
	"POWER_PLANT_UPGRADED",
	"CLIMBING",
	"SOLD",
#ifdef ALLOW_SURRENDER
	"SURRENDER",
#endif
	"RAPPELLING",
	"ARMED",
	"POWER_PLANT_UPGRADING",

	"SPECIAL_CHEERING",

	"CONTINUOUS_FIRE_SLOW",
	"CONTINUOUS_FIRE_MEAN",
	"CONTINUOUS_FIRE_FAST",

	"RAISING_FLAG",
	"CAPTURED",

	"EXPLODED_FLAILING",
	"EXPLODED_BOUNCING",
	"SPLATTED",

	"USING_WEAPON_A",
	"USING_WEAPON_B",
	"USING_WEAPON_C",

	"PREORDER",

	"CENTER_TO_LEFT",
	"LEFT_TO_CENTER",
	"CENTER_TO_RIGHT",
	"RIGHT_TO_CENTER",

	"RIDER1",	//Kris: Added these for different combat-bike riders, but feel free to use these for anything.
	"RIDER2",
	"RIDER3",
	"RIDER4",
	"RIDER5",
	"RIDER6",
	"RIDER7",
	"RIDER8",

  "STUNNED_FLAILING", // Daniel Teh's idea, added by Lorenzen, 5/28/03
	"STUNNED",
	"SECOND_LIFE",
	"JAMMED",
	"ARMORSET_CRATEUPGRADE_ONE",
	"ARMORSET_CRATEUPGRADE_TWO",

	"USER_1",
	"USER_2",

	"DISGUISED",

	nullptr
};
static_assert(ARRAY_SIZE(ModelConditionFlags::s_bitNameList) == ModelConditionFlags::NumBits + 1, "Incorrect array size");

template<>
const char* const ArmorSetFlags::s_bitNameList[] =
{
	"VETERAN",
	"ELITE",
	"HERO",
	"PLAYER_UPGRADE",
	"WEAK_VERSUS_BASEDEFENSES",
	"SECOND_LIFE",
	"CRATE_UPGRADE_ONE",
	"CRATE_UPGRADE_TWO",

	nullptr
};
static_assert(ARRAY_SIZE(ArmorSetFlags::s_bitNameList) == ArmorSetFlags::NumBits + 1, "Incorrect array size");
```

### PKT-129: `ObjectStatusTypes.h` (`Core/GameEngine/Include/Common/ObjectStatusTypes.h`)
```cpp
// FILE: ObjectStatusTypes.h /////////////////////////////////////////////////////////////////////////
// Author: Kris, May 2003
// Desc:   Object status types that are stackable using the BitSet system. Used to be ObjectStatusBits
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/BitFlags.h"
#include "Common/BitFlagsIO.h"

//-------------------------------------------------------------------------------------------------
/** Object status types */
//-------------------------------------------------------------------------------------------------
enum ObjectStatusTypes CPP_11(: Int)
{
	//These are saved. Do not insert or remove any!

	OBJECT_STATUS_NONE,									///< no status bit
	OBJECT_STATUS_DESTROYED,						///< has been destroyed, pending delete
	OBJECT_STATUS_CAN_ATTACK,						///< used by garrissoned buildings, is OR'ed with KINDOF_CAN_ATTACK in isAbleToAttack()
	OBJECT_STATUS_UNDER_CONSTRUCTION,		///< object is being constructed and is not yet complete
	OBJECT_STATUS_UNSELECTABLE,					///< This is a negative condition since these statuses are overrides.  ie their presence forces the condition, but their absence means nothing
	OBJECT_STATUS_NO_COLLISIONS,				///< object should be ignored for object-object collisions (but not object-ground); used for thing like collapsing parachutes that are intangible
	OBJECT_STATUS_NO_ATTACK,						///< Absolute override to being able to attack
	OBJECT_STATUS_AIRBORNE_TARGET,			///< InTheAir as far as AntiAir weapons are concerned only.
	OBJECT_STATUS_PARACHUTING,					///< object is on a parachute
	OBJECT_STATUS_REPULSOR,							///< object repulses "KINDOF_CAN_BE_REPULSED" objects.
	OBJECT_STATUS_HIJACKED,  						///< unit is in the possesion of an enemy criminal, call the authorities
	OBJECT_STATUS_AFLAME,								///< This object is on fire.
	OBJECT_STATUS_BURNED,								///< This object has already burned as much as it can.
	OBJECT_STATUS_WET,									///< object has been soaked with water
	OBJECT_STATUS_IS_FIRING_WEAPON,			///< Object is firing a weapon, now. Not true for special attacks. --Lorenzen
	OBJECT_STATUS_BRAKING,							///< Object is braking, and subverts the physics.
	OBJECT_STATUS_STEALTHED,						///< Object is currently "stealthed"
	OBJECT_STATUS_DETECTED,							///< Object is in range of a stealth-detector unit (meaningless if STEALTHED not set)
	OBJECT_STATUS_CAN_STEALTH,					///< Object has ability to stealth allowing the stealth update module to run.
	OBJECT_STATUS_SOLD,									///< Object is being sold
	OBJECT_STATUS_UNDERGOING_REPAIR,		///< Object is awaiting/undergoing a repair order that has been issued
	OBJECT_STATUS_RECONSTRUCTING,				///< Reconstructing
	OBJECT_STATUS_MASKED,								///< Masked objects are not selectable and targetable by players or AI
	OBJECT_STATUS_IS_ATTACKING,					///< Object is in the general Attack state (incl. aim, approach, etc.). Note that IS_FIRING_WEAPON and IS_AIMING_WEAPON is a subset of this!
	OBJECT_STATUS_IS_USING_ABILITY,			///< Object is in the process of preparing or firing a special ability.
	OBJECT_STATUS_IS_AIMING_WEAPON,			///< Object is aiming a weapon, now. Not true for special attacks.
	OBJECT_STATUS_NO_ATTACK_FROM_AI,		///< attacking this object may not be done from commandSource == CMD_FROM_AI
	OBJECT_STATUS_IGNORING_STEALTH,			///< temporarily ignoring all stealth bits. (used only for some special-case mine clearing stuff.)
	OBJECT_STATUS_IS_CARBOMB,						///< Object is now a carbomb.

	// TheSuperHackers @info New statuses added in Zero Hour
	// Note: Loading old save games that do not track these flags in objects will not recover them. Expect logic bugs.
	OBJECT_STATUS_DECK_HEIGHT_OFFSET,		///< Object factors deck height on top of ground altitude.
	OBJECT_STATUS_RIDER1,
	OBJECT_STATUS_RIDER2,
	OBJECT_STATUS_RIDER3,
	OBJECT_STATUS_RIDER4,
	OBJECT_STATUS_RIDER5,
	OBJECT_STATUS_RIDER6,
	OBJECT_STATUS_RIDER7,
	OBJECT_STATUS_RIDER8,
	OBJECT_STATUS_FAERIE_FIRE,			///< Anyone shooting at you shoots faster than normal
  OBJECT_STATUS_MISSILE_KILLING_SELF, ///< Object (likely a missile or bomb) is *BUSTING* its way through the *BUNKER*, building or ground, awaiting death at the bottom.
	OBJECT_STATUS_REASSIGN_PARKING,			///< Jet is trying to get a better parking assignment.
	OBJECT_STATUS_BOOBY_TRAPPED,				///< We need to know we have a booby trap on us so we can detonate it from many different code segments
	OBJECT_STATUS_IMMOBILE,							///< Do not move!
	OBJECT_STATUS_DISGUISED,						///< Object is disguised (a type of stealth)
	OBJECT_STATUS_DEPLOYED,							///< Object is deployed.
	// add more status types here and don't forget to add to the string table ObjectStatusMaskType::s_bitNameList[]

	OBJECT_STATUS_COUNT

};

typedef BitFlags<OBJECT_STATUS_COUNT, struct ObjectStatusMaskTypeTag>	ObjectStatusMaskType;

#define MAKE_OBJECT_STATUS_MASK(k) ObjectStatusMaskType(ObjectStatusMaskType::kInit, (k))
#define MAKE_OBJECT_STATUS_MASK2(k,a) ObjectStatusMaskType(ObjectStatusMaskType::kInit, (k), (a))
#define MAKE_OBJECT_STATUS_MASK3(k,a,b) ObjectStatusMaskType(ObjectStatusMaskType::kInit, (k), (a), (b))
#define MAKE_OBJECT_STATUS_MASK4(k,a,b,c) ObjectStatusMaskType(ObjectStatusMaskType::kInit, (k), (a), (b), (c))
#define MAKE_OBJECT_STATUS_MASK5(k,a,b,c,d) ObjectStatusMaskType(ObjectStatusMaskType::kInit, (k), (a), (b), (c), (d))

inline Bool TEST_OBJECT_STATUS_MASK( const ObjectStatusMaskType& m, ObjectStatusTypes t )
{
	return m.test( t );
}

inline Bool TEST_OBJECT_STATUS_MASK_ANY( const ObjectStatusMaskType& m, const ObjectStatusMaskType& mask )
{
	return m.anyIntersectionWith( mask );
}

inline Bool TEST_OBJECT_STATUS_MASK_MULTI( const ObjectStatusMaskType& m, const ObjectStatusMaskType& mustBeSet, const ObjectStatusMaskType& mustBeClear )
{
	return m.testSetAndClear( mustBeSet, mustBeClear );
}

inline Bool OBJECT_STATUS_MASK_ANY_SET( const ObjectStatusMaskType& m)
{
	return m.any();
}

inline void CLEAR_OBJECT_STATUS_MASK( ObjectStatusMaskType& m )
{
	m.clear();
}

inline void SET_ALL_OBJECT_STATUS_MASK_BITS( ObjectStatusMaskType& m )
{
	m.clear( );
	m.flip( );
}

inline void FLIP_OBJECT_STATUS_MASK( ObjectStatusMaskType& m )
{
	m.flip();
}

// defined in Common/System/ObjectStatusTypes.cpp
extern ObjectStatusMaskType OBJECT_STATUS_MASK_NONE;	// inits to all zeroes
```

### PKT-129: `ObjectStatusTypes.cpp` (`Core/GameEngine/Source/Common/System/ObjectStatusTypes.cpp`)
```cpp
// FILE: ObjectStatusTypes.cpp ////////////////////////////////////////////////////////////////////
// Author: Kris, May 2003
// Desc:   Object status types that are stackable using the BitSet system. Used to be ObjectStatusBits
///////////////////////////////////////////////////////////////////////////////////////////////////

#include "PreRTS.h"

#include "Common/ObjectStatusTypes.h"
#include "Common/BitFlagsIO.h"

template<>
const char* const ObjectStatusMaskType::s_bitNameList[] =
{
	"NONE",
	"DESTROYED",
	"CAN_ATTACK",
	"UNDER_CONSTRUCTION",
	"UNSELECTABLE",
	"NO_COLLISIONS",
	"NO_ATTACK",
	"AIRBORNE_TARGET",
	"PARACHUTING",
	"REPULSOR",
	"HIJACKED",
	"AFLAME",
	"BURNED",
	"WET",
	"IS_FIRING_WEAPON",
	"IS_BRAKING",
	"STEALTHED",
	"DETECTED",
	"CAN_STEALTH",
	"SOLD",
	"UNDERGOING_REPAIR",
	"RECONSTRUCTING",
	"MASKED",
	"IS_ATTACKING",
	"USING_ABILITY",
	"IS_AIMING_WEAPON",
	"NO_ATTACK_FROM_AI",
	"IGNORING_STEALTH",
	"IS_CARBOMB",
	// TheSuperHackers @info New statuses added in Zero Hour
	"DECK_HEIGHT_OFFSET",
	"STATUS_RIDER1",
	"STATUS_RIDER2",
	"STATUS_RIDER3",
	"STATUS_RIDER4",
	"STATUS_RIDER5",
	"STATUS_RIDER6",
	"STATUS_RIDER7",
	"STATUS_RIDER8",
	"FAERIE_FIRE",
  "KILLING_SELF",
	"REASSIGN_PARKING",
	"BOOBY_TRAPPED",
	"IMMOBILE",
	"DISGUISED",
	"DEPLOYED",
	nullptr
};
static_assert(ARRAY_SIZE(ObjectStatusMaskType::s_bitNameList) == ObjectStatusMaskType::NumBits + 1, "Incorrect array size");

ObjectStatusMaskType OBJECT_STATUS_MASK_NONE;	// inits to all zeroes
```

### PKT-135 & PKT-136: `SpecialPowerMaskType.h` (`GeneralsMD/Code/GameEngine/Include/Common/SpecialPowerMaskType.h`)
```cpp
#pragma once

#include "Common/BitFlags.h"
#include "Common/BitFlagsIO.h"
#include "Common/SpecialPowerType.h"

typedef BitFlags<SPECIALPOWER_COUNT, struct SpecialPowerMaskTypeTag>	SpecialPowerMaskType;
```

### PKT-136: `SpecialPowerType.h` (`GeneralsMD/Code/GameEngine/Include/Common/SpecialPowerType.h`)
```cpp
#pragma once

// ------------------------------------------------------------------------------------------------
// don't forget to add new strings to SpecialPowerMaskType::s_bitNameList[]
// ------------------------------------------------------------------------------------------------
//
// Note: these values are saved in save files, so you MUST NOT REMOVE OR CHANGE
// existing values!
//
enum SpecialPowerType CPP_11(: Int)
{
	SPECIAL_INVALID,
	// don't forget to add new strings to SpecialPowerMaskType::s_bitNameList[]

	//Superweapons
	SPECIAL_DAISY_CUTTER,
	SPECIAL_PARADROP_AMERICA,
	SPECIAL_CARPET_BOMB,
	SPECIAL_CLUSTER_MINES,
	SPECIAL_EMP_PULSE,
	SPECIAL_NAPALM_STRIKE,
	SPECIAL_CASH_HACK,
	SPECIAL_NEUTRON_MISSILE,
	SPECIAL_SPY_SATELLITE,
	SPECIAL_DEFECTOR,
	SPECIAL_TERROR_CELL,
	SPECIAL_AMBUSH,
	SPECIAL_BLACK_MARKET_NUKE,
	SPECIAL_ANTHRAX_BOMB,
	SPECIAL_SCUD_STORM,
#ifdef ALLOW_DEMORALIZE
	SPECIAL_DEMORALIZE,
#else
	SPECIAL_DEMORALIZE_OBSOLETE,
#endif
	SPECIAL_CRATE_DROP,
	SPECIAL_A10_THUNDERBOLT_STRIKE,
	SPECIAL_DETONATE_DIRTY_NUKE,
	SPECIAL_ARTILLERY_BARRAGE,
	// don't forget to add new strings to SpecialPowerMaskType::s_bitNameList[]

	//Special abilities
	SPECIAL_MISSILE_DEFENDER_LASER_GUIDED_MISSILES,
	SPECIAL_REMOTE_CHARGES,
	SPECIAL_TIMED_CHARGES,
	SPECIAL_HELIX_NAPALM_BOMB,
	SPECIAL_HACKER_DISABLE_BUILDING,
	SPECIAL_TANKHUNTER_TNT_ATTACK,
	SPECIAL_BLACKLOTUS_CAPTURE_BUILDING,
	SPECIAL_BLACKLOTUS_DISABLE_VEHICLE_HACK,
	SPECIAL_BLACKLOTUS_STEAL_CASH_HACK,
	SPECIAL_INFANTRY_CAPTURE_BUILDING,
	SPECIAL_RADAR_VAN_SCAN,
	SPECIAL_SPY_DRONE,
	SPECIAL_DISGUISE_AS_VEHICLE,
	SPECIAL_BOOBY_TRAP,
	// don't forget to add new strings to SpecialPowerMaskType::s_bitNameList[]
	SPECIAL_REPAIR_VEHICLES,
	SPECIAL_PARTICLE_UPLINK_CANNON,
	SPECIAL_CASH_BOUNTY,
	SPECIAL_CHANGE_BATTLE_PLANS,
	SPECIAL_CIA_INTELLIGENCE,
	SPECIAL_CLEANUP_AREA,
	// don't forget to add new strings to SpecialPowerMaskType::s_bitNameList[]
	SPECIAL_LAUNCH_BAIKONUR_ROCKET,

  SPECIAL_SPECTRE_GUNSHIP,
  SPECIAL_GPS_SCRAMBLER,

	SPECIAL_FRENZY,
	SPECIAL_SNEAK_ATTACK,

	//Ack, this is ass. These enums fix a bug where new enums were missing for
	//shortcut powers... but the real clincher was that if you were say USA and
	//captured a Tank China command center, your US paradrop would be assigned
	//to the china tank drop and when you tried to fire it from the shortcut
	//it could pick the china one and not fire it because it didn't have
	//complete connection... ugh!!!
	SPECIAL_CHINA_CARPET_BOMB,
	EARLY_SPECIAL_CHINA_CARPET_BOMB,
	SPECIAL_LEAFLET_DROP,
	EARLY_SPECIAL_LEAFLET_DROP,
	EARLY_SPECIAL_FRENZY,
	SPECIAL_COMMUNICATIONS_DOWNLOAD,
	EARLY_SPECIAL_REPAIR_VEHICLES,
	SPECIAL_TANK_PARADROP,
	SUPW_SPECIAL_PARTICLE_UPLINK_CANNON,
	AIRF_SPECIAL_DAISY_CUTTER,
	NUKE_SPECIAL_CLUSTER_MINES,
	NUKE_SPECIAL_NEUTRON_MISSILE,
	AIRF_SPECIAL_A10_THUNDERBOLT_STRIKE,
	AIRF_SPECIAL_SPECTRE_GUNSHIP,
	INFA_SPECIAL_PARADROP_AMERICA,
	SLTH_SPECIAL_GPS_SCRAMBLER,
	AIRF_SPECIAL_CARPET_BOMB,
	SUPR_SPECIAL_CRUISE_MISSILE,
	LAZR_SPECIAL_PARTICLE_UPLINK_CANNON,
	SUPW_SPECIAL_NEUTRON_MISSILE,

	SPECIAL_BATTLESHIP_BOMBARDMENT,

	SPECIALPOWER_COUNT,
	// don't forget to add new strings to SpecialPowerMaskType::s_bitNameList[]
};

	// Definition of these names is located in SpecialPower.cpp
```

### PKT-135: `SpecialPower.cpp` (`SpecialPowerMaskType::s_bitNameList`) (`GeneralsMD/Code/GameEngine/Source/Common/RTS/SpecialPower.cpp`)
```cpp
template<>
const char* const SpecialPowerMaskType::s_bitNameList[] =
{
	"SPECIAL_INVALID",

	//Superweapons
	"SPECIAL_DAISY_CUTTER",
	"SPECIAL_PARADROP_AMERICA",
	"SPECIAL_CARPET_BOMB",
	"SPECIAL_CLUSTER_MINES",
	"SPECIAL_EMP_PULSE",
	"SPECIAL_NAPALM_STRIKE",
	"SPECIAL_CASH_HACK",
	"SPECIAL_NEUTRON_MISSILE",
	"SPECIAL_SPY_SATELLITE",
	"SPECIAL_DEFECTOR",
	"SPECIAL_TERROR_CELL",
	"SPECIAL_AMBUSH",
	"SPECIAL_BLACK_MARKET_NUKE",
	"SPECIAL_ANTHRAX_BOMB",
	"SPECIAL_SCUD_STORM",
#ifdef ALLOW_DEMORALIZE
	"SPECIAL_DEMORALIZE",
#else
	"SPECIAL_DEMORALIZE_OBSOLETE",
#endif
	"SPECIAL_CRATE_DROP",
	"SPECIAL_A10_THUNDERBOLT_STRIKE",
	"SPECIAL_DETONATE_DIRTY_NUKE",
	"SPECIAL_ARTILLERY_BARRAGE",

	//Special abilities
	"SPECIAL_MISSILE_DEFENDER_LASER_GUIDED_MISSILES",
	"SPECIAL_REMOTE_CHARGES",
	"SPECIAL_TIMED_CHARGES",
	"SPECIAL_HELIX_NAPALM_BOMB",
	"SPECIAL_HACKER_DISABLE_BUILDING",
	"SPECIAL_TANKHUNTER_TNT_ATTACK",
	"SPECIAL_BLACKLOTUS_CAPTURE_BUILDING",
	"SPECIAL_BLACKLOTUS_DISABLE_VEHICLE_HACK",
	"SPECIAL_BLACKLOTUS_STEAL_CASH_HACK",
	"SPECIAL_INFANTRY_CAPTURE_BUILDING",
	"SPECIAL_RADAR_VAN_SCAN",
	"SPECIAL_SPY_DRONE",
	"SPECIAL_DISGUISE_AS_VEHICLE",
	"SPECIAL_BOOBY_TRAP",
	"SPECIAL_REPAIR_VEHICLES",
	"SPECIAL_PARTICLE_UPLINK_CANNON",
	"SPECIAL_CASH_BOUNTY",
	"SPECIAL_CHANGE_BATTLE_PLANS",
	"SPECIAL_CIA_INTELLIGENCE",
	"SPECIAL_CLEANUP_AREA",
	"SPECIAL_LAUNCH_BAIKONUR_ROCKET",
  "SPECIAL_SPECTRE_GUNSHIP",
  "SPECIAL_GPS_SCRAMBLER",
	"SPECIAL_FRENZY",
	"SPECIAL_SNEAK_ATTACK",

	"SPECIAL_CHINA_CARPET_BOMB",
	"EARLY_SPECIAL_CHINA_CARPET_BOMB",
	"SPECIAL_LEAFLET_DROP",
	"EARLY_SPECIAL_LEAFLET_DROP",
	"EARLY_SPECIAL_FRENZY",
	"SPECIAL_COMMUNICATIONS_DOWNLOAD",
	"EARLY_SPECIAL_REPAIR_VEHICLES",
	"SPECIAL_TANK_PARADROP",
	"SUPW_SPECIAL_PARTICLE_UPLINK_CANNON",
	"AIRF_SPECIAL_DAISY_CUTTER",
	"NUKE_SPECIAL_CLUSTER_MINES",
	"NUKE_SPECIAL_NEUTRON_MISSILE",
	"AIRF_SPECIAL_A10_THUNDERBOLT_STRIKE",
	"AIRF_SPECIAL_SPECTRE_GUNSHIP",
	"INFA_SPECIAL_PARADROP_AMERICA",
	"SLTH_SPECIAL_GPS_SCRAMBLER",
	"AIRF_SPECIAL_CARPET_BOMB",
	"SUPR_SPECIAL_CRUISE_MISSILE",
	"LAZR_SPECIAL_PARTICLE_UPLINK_CANNON",
	"SUPW_SPECIAL_NEUTRON_MISSILE",

	"SPECIAL_BATTLESHIP_BOMBARDMENT",

	nullptr
};
static_assert(ARRAY_SIZE(SpecialPowerMaskType::s_bitNameList) == SpecialPowerMaskType::NumBits + 1, "Incorrect array size");
```

### PKT-139: `ArmorSet.h` (`GeneralsMD/Code/GameEngine/Include/GameLogic/ArmorSet.h`)
```cpp
#pragma once

#include "Common/GameType.h"
#include "Common/SparseMatchFinder.h"

//-------------------------------------------------------------------------------------------------
class ArmorTemplate;
class DamageFX;
class INI;

//-------------------------------------------------------------------------------------------------
// IMPORTANT NOTE: you should endeavor to set up states such that the most "normal"
// state is defined by the bit being off. That is, the typical "normal" condition
// has all condition flags set to zero.
enum ArmorSetType CPP_11(: Int)
{
	// The access and use of this enum has the bit shifting built in, so this is a 0,1,2,3,4,5 enum
	ARMORSET_VETERAN		= 0,
	ARMORSET_ELITE			= 1,
	ARMORSET_HERO				= 2,
	ARMORSET_PLAYER_UPGRADE = 3,
	ARMORSET_WEAK_VERSUS_BASEDEFENSES = 4,
	ARMORSET_SECOND_LIFE = 5,	///< Body Module has marked us as on our second life
	ARMORSET_CRATE_UPGRADE_ONE, ///< Just like weaponset type from salvage.
	ARMORSET_CRATE_UPGRADE_TWO,

	ARMORSET_COUNT
};

//-------------------------------------------------------------------------------------------------
typedef BitFlags<ARMORSET_COUNT, struct ArmorSetFlagsTag> ArmorSetFlags;

//-------------------------------------------------------------------------------------------------
class ArmorTemplateSet
{
private:
	ArmorSetFlags m_types;
	const ArmorTemplate* m_template;
	const DamageFX* m_fx;

public:
	ArmorTemplateSet()
	{
		clear();
	}

	void clear()
	{
		m_types.clear();
		m_template = nullptr;
		m_fx = nullptr;
	}

	const ArmorTemplate* getArmorTemplate() const { return m_template; }
	const DamageFX* getDamageFX() const { return m_fx; }

	Int getConditionsYesCount() const { return 1; }
	const ArmorSetFlags& getNthConditionsYes(Int i) const { return m_types; }
#if defined(RTS_DEBUG)
	inline AsciiString getDescription() const { return "ArmorTemplateSet"; }
#endif

	void parseArmorTemplateSet( INI* ini );
};

//-------------------------------------------------------------------------------------------------
typedef std::vector<ArmorTemplateSet> ArmorTemplateSetVector;
```

### PKT-140 (Enums): `Damage.h` (`DamageType`, `DeathType`) (`Core/GameEngine/Include/GameLogic/Damage.h`)
```cpp
//-------------------------------------------------------------------------------------------------
/** Damage types, keep this in sync with DamageTypeFlags::s_bitNameList[] */
//-------------------------------------------------------------------------------------------------
enum DamageType CPP_11(: Int)
{
	DAMAGE_EXPLOSION							= 0,
	DAMAGE_CRUSH									= 1,
	DAMAGE_ARMOR_PIERCING					= 2,
	DAMAGE_SMALL_ARMS							= 3,
	DAMAGE_GATTLING								= 4,
	DAMAGE_RADIATION							= 5,
	DAMAGE_FLAME									= 6,
	DAMAGE_LASER									= 7,
	DAMAGE_SNIPER									= 8,
	DAMAGE_POISON									= 9,
	DAMAGE_HEALING								= 10,
	DAMAGE_UNRESISTABLE						= 11,		// this is for scripting to cause 'armorproof' damage
	DAMAGE_WATER									= 12,
	DAMAGE_DEPLOY									= 13,					// for transports to deploy units and order them to all attack.
	DAMAGE_SURRENDER							= 14,				// if something "dies" to surrender damage, they surrender.... duh!
	DAMAGE_HACK										= 15,
	DAMAGE_KILLPILOT							= 16,				// special snipe attack that kills the pilot and renders a vehicle unmanned.
	DAMAGE_PENALTY								= 17,					// from game penalty (you won't receive radar warnings BTW)
	DAMAGE_FALLING								= 18,
	DAMAGE_MELEE									= 19,						// Blades, clubs...
	DAMAGE_DISARM									= 20,	// "special" damage type used for disarming mines, bombs, etc (NOT for "disarming" an opponent!)
	DAMAGE_HAZARD_CLEANUP					= 21,	// special damage type for cleaning up hazards like radiation or bio-poison.
	DAMAGE_PARTICLE_BEAM					= 22,	// Incinerates virtually everything (insanely powerful orbital beam)
	DAMAGE_TOPPLING								= 23,	// damage from getting toppled.
	DAMAGE_INFANTRY_MISSILE				= 24,
	DAMAGE_AURORA_BOMB						= 25,
	DAMAGE_LAND_MINE							= 26,
	DAMAGE_JET_MISSILES						= 27,
	DAMAGE_STEALTHJET_MISSILES		= 28,
	DAMAGE_MOLOTOV_COCKTAIL				= 29,
	DAMAGE_COMANCHE_VULCAN				= 30,
#if RTS_GENERALS
	DAMAGE_FLESHY_SNIPER					= 31,		// like DAMAGE_SNIPER, but (generally) does no damage to vehicles.
#endif
	DAMAGE_SUBDUAL_MISSILE				/*= 31*/,	///< Damage that does not kill you, but produces some special effect based on your Body Module. Separate HP from normal damage.
	DAMAGE_SUBDUAL_VEHICLE				/*= 32*/,
	DAMAGE_SUBDUAL_BUILDING				/*= 33*/,
	DAMAGE_SUBDUAL_UNRESISTABLE		/*= 34*/,
	DAMAGE_MICROWAVE							/*= 35*/, ///< Radiation that only affects infantry
	DAMAGE_KILL_GARRISONED				/*= 36*/, ///< Kills Passengers up to the number specified in Damage
	DAMAGE_STATUS									/*= 37*/, ///< Damage that gives a status condition, not that does hitpoint damage

	// Please note: There is a string array DamageTypeFlags::s_bitNameList[]

	DAMAGE_NUM_TYPES
};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------

typedef BitFlags<DAMAGE_NUM_TYPES, struct DamageTypeFlagsTag> DamageTypeFlags;

inline Bool getDamageTypeFlag(DamageTypeFlags flags, DamageType dt)
{
	return flags.test(dt);
}

inline DamageTypeFlags setDamageTypeFlag(DamageTypeFlags flags, DamageType dt)
{
	flags.set(dt, TRUE);
	return flags;
}

inline DamageTypeFlags clearDamageTypeFlag(DamageTypeFlags flags, DamageType dt)
{
	flags.set(dt, FALSE);
	return flags;
}

// Instead of checking against a single damage type, gather the question here so we can have more than one
inline Bool IsSubdualDamage( DamageType type )
{
	switch( type )
	{
		case DAMAGE_SUBDUAL_MISSILE:
		case DAMAGE_SUBDUAL_VEHICLE:
		case DAMAGE_SUBDUAL_BUILDING:
		case DAMAGE_SUBDUAL_UNRESISTABLE:
			return TRUE;
	}

	return FALSE;
}

/// Does this type of damage go to internalChangeHealth?
inline Bool IsHealthDamagingDamage( DamageType type )
{
	// The need for this function brought to you by "Have the guy with no game experience write the weapon code" Foundation.
	// Health Damage should be one type of WeaponEffect.  Thinking "Weapons can only do damage" is why AoE is boring.
	switch( type )
	{
		case DAMAGE_STATUS:
		case DAMAGE_SUBDUAL_MISSILE:
		case DAMAGE_SUBDUAL_VEHICLE:
		case DAMAGE_SUBDUAL_BUILDING:
		case DAMAGE_SUBDUAL_UNRESISTABLE:
		case DAMAGE_KILLPILOT:
		case DAMAGE_KILL_GARRISONED:
			return FALSE;
	}

	return TRUE;
}

inline void SET_ALL_DAMAGE_TYPE_BITS(DamageTypeFlags& m)
{
	m.clear();
	m.flip();
}

extern DamageTypeFlags DAMAGE_TYPE_FLAGS_NONE;
extern DamageTypeFlags DAMAGE_TYPE_FLAGS_ALL;


//-------------------------------------------------------------------------------------------------
/** Death types, keep this in sync with TheDeathNames[] */
//-------------------------------------------------------------------------------------------------
enum DeathType CPP_11(: Int)
{
	// note that these DELIBERATELY have (slightly) different names from the damage names,
	// since there isn't necessarily a one-to-one correspondence. e.g., DEATH_BURNED
	// can come from DAMAGE_FLAME but also from DAMAGE_PARTICLE_BEAM.
	DEATH_NORMAL		= 0,
	DEATH_NONE			= 1,					///< this is a "special case" that can't normally cause death
  DEATH_CRUSHED		= 2,
  DEATH_BURNED		= 3,
  DEATH_EXPLODED	= 4,
	DEATH_POISONED	= 5,
	DEATH_TOPPLED		= 6,
	DEATH_FLOODED		= 7,
	DEATH_SUICIDED	= 8,
	DEATH_LASERED		= 9,
	DEATH_DETONATED	= 10,		/**< this is the "death" that occurs when a missile/warhead/etc detonates normally,
														as opposed to being shot down, etc */
	DEATH_SPLATTED	= 11,		/**< the death that results from DAMAGE_FALLING */
	DEATH_POISONED_BETA	= 12,

	// these are the "extra" types for yet-to-be-defined stuff. Don't bother renaming or adding
	// or removing these; they are reserved for modders :-)
	DEATH_EXTRA_2		= 13,
	DEATH_EXTRA_3		= 14,
	DEATH_EXTRA_4		= 15,
	DEATH_EXTRA_5		= 16,
	DEATH_EXTRA_6		= 17,
	DEATH_EXTRA_7		= 18,
	DEATH_EXTRA_8		= 19,
	DEATH_POISONED_GAMMA = 20,

	DEATH_NUM_TYPES
};

#ifdef DEFINE_DEATH_NAMES
static const char *const TheDeathNames[] =
{
	"NORMAL",
	"NONE",
	"CRUSHED",
	"BURNED",
	"EXPLODED",
	"POISONED",
	"TOPPLED",
	"FLOODED",
	"SUICIDED",
	"LASERED",
	"DETONATED",
	"SPLATTED",
	"POISONED_BETA",
	"EXTRA_2",
	"EXTRA_3",
	"EXTRA_4",
	"EXTRA_5",
	"EXTRA_6",
	"EXTRA_7",
	"EXTRA_8",
	"POISONED_GAMMA",

	nullptr
};
static_assert(ARRAY_SIZE(TheDeathNames) == DEATH_NUM_TYPES + 1, "Incorrect array size");
#endif // end DEFINE_DEATH_NAMES


//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------

typedef UnsignedInt DeathTypeFlags;

const DeathTypeFlags DEATH_TYPE_FLAGS_ALL = 0xffffffff;
const DeathTypeFlags DEATH_TYPE_FLAGS_NONE = 0x00000000;

static_assert(DEATH_NUM_TYPES <= 32, "DeathTypeFlags has 32 bits");

inline Bool getDeathTypeFlag(DeathTypeFlags flags, DeathType dt)
{
	return (flags & (1UL << dt)) != 0;
}

inline DeathTypeFlags setDeathTypeFlag(DeathTypeFlags flags, DeathType dt)
{
	return (flags | (1UL << dt));
}

inline DeathTypeFlags clearDeathTypeFlag(DeathTypeFlags flags, DeathType dt)
{
	return (flags & ~(1UL << dt));
}

```

### PKT-140 (Names): `Damage.cpp` (`DamageTypeFlags::s_bitNameList`) (`Core/GameEngine/Source/GameLogic/System/Damage.cpp`)
```cpp
template<>
const char* const DamageTypeFlags::s_bitNameList[] =
{
	"EXPLOSION",
	"CRUSH",
	"ARMOR_PIERCING",
	"SMALL_ARMS",
	"GATTLING",
	"RADIATION",
	"FLAME",
	"LASER",
	"SNIPER",
	"POISON",
	"HEALING",
	"UNRESISTABLE",
	"WATER",
	"DEPLOY",
	"SURRENDER",
	"HACK",
	"KILL_PILOT",
	"PENALTY",
	"FALLING",
	"MELEE",
	"DISARM",
	"HAZARD_CLEANUP",
	"PARTICLE_BEAM",
	"TOPPLING",
	"INFANTRY_MISSILE",
	"AURORA_BOMB",
	"LAND_MINE",
	"JET_MISSILES",
	"STEALTHJET_MISSILES",
	"MOLOTOV_COCKTAIL",
	"COMANCHE_VULCAN",
#if RTS_GENERALS
	"FLESHY_SNIPER",
#endif
	"SUBDUAL_MISSILE",
	"SUBDUAL_VEHICLE",
	"SUBDUAL_BUILDING",
	"SUBDUAL_UNRESISTABLE",
	"MICROWAVE",
	"KILL_GARRISONED",
	"STATUS",

	nullptr
};
static_assert(ARRAY_SIZE(DamageTypeFlags::s_bitNameList) == DamageTypeFlags::NumBits + 1, "Incorrect array size");

DamageTypeFlags DAMAGE_TYPE_FLAGS_NONE; 	// inits to all zeroes
DamageTypeFlags DAMAGE_TYPE_FLAGS_ALL(DamageTypeFlags::kInitSetAll);
```

### PKT-144: `LocomotorSet.h` (`LocomotorSurfaceType` & `TheLocomotorSurfaceTypeNames`) (`GeneralsMD/Code/GameEngine/Include/GameLogic/LocomotorSet.h`)
```cpp
// FILE: LocomotorSet.h /////////////////////////////////////////////////////////////////////////////////
// Author: Steven Johnson, Feb 2002
// Desc: Locomotor Descriptions
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// no, please do NOT include this.
//#include "GameLogic/Locomotor.h"
#include "Common/GameCommon.h"
#include "Common/STLTypedefs.h"
#include "Common/Snapshot.h"

class Locomotor;
class LocomotorTemplate;

//-------------------------------------------------------------------------------------------------
//
// Note: these values are saved in save files, so you MUST NOT REMOVE OR CHANGE
// existing values!
//
enum LocomotorSurfaceType CPP_11(: Int)
{
	LOCOMOTORSURFACE_GROUND			= (1 << 0),									///< clear, unobstructed ground
	LOCOMOTORSURFACE_WATER			= (1 << 1),									///< water area
	LOCOMOTORSURFACE_CLIFF			= (1 << 2),									///< steep altitude change
	LOCOMOTORSURFACE_AIR				= (1 << 3),									///< airborne
	LOCOMOTORSURFACE_RUBBLE			= (1 << 4),									///< building rubble

	LOCOMOTOR_SURFACE_TYPE_END // keep after the last named flag
};

typedef Int LocomotorSurfaceTypeMask;

const LocomotorSurfaceTypeMask NO_SURFACES = (LocomotorSurfaceTypeMask)0x0000;
const LocomotorSurfaceTypeMask ALL_SURFACES = (LocomotorSurfaceTypeMask)0xffff;

#ifdef DEFINE_SURFACECATEGORY_NAMES
static const char *const TheLocomotorSurfaceTypeNames[] =
{
	"GROUND",
	"WATER",
	"CLIFF",
	"AIR",
	"RUBBLE",

	nullptr
};
static_assert(equalsEnumBitFlagsCount(ARRAY_SIZE(TheLocomotorSurfaceTypeNames) - 1, LOCOMOTOR_SURFACE_TYPE_END), "Incorrect array size");
#endif

//-------------------------------------------------------------------------------------------------
typedef std::vector< Locomotor* > LocomotorVector;

//-------------------------------------------------------------------------------------------------
class LocomotorSet : public Snapshot
{
private:
	LocomotorVector						m_locomotors;
	LocomotorSurfaceTypeMask	m_validLocomotorSurfaces;
	Bool											m_downhillOnly;

	LocomotorSet(const LocomotorSet& that);
	LocomotorSet& operator=(const LocomotorSet& that);

protected:
	// snapshot methods
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

public:

	LocomotorSet();
	~LocomotorSet();

	void clear();

	void addLocomotor(const LocomotorTemplate* lt);

	Locomotor* findLocomotor(LocomotorSurfaceTypeMask t);

	void xferSelfAndCurLocoPtr(Xfer *xfer, Locomotor** loco);

	LocomotorSurfaceTypeMask getValidSurfaces() const { return m_validLocomotorSurfaces; }
	Bool isDownhillOnly() const { return m_downhillOnly; };

};
```

### PKT-146: `WeaponBonusConditionFlags.h` (`GeneralsMD/Code/GameEngine/Include/GameLogic/WeaponBonusConditionFlags.h`)
```cpp
#pragma once

typedef UnsignedInt WeaponBonusConditionFlags;
```

### PKT-147: `WeaponSetFlags.h` (`GeneralsMD/Code/GameEngine/Include/GameLogic/WeaponSetFlags.h`)
```cpp
#pragma once

#include "GameLogic/WeaponSetType.h"

typedef BitFlags<WEAPONSET_COUNT, struct WeaponSetFlagsTag> WeaponSetFlags;
```

### PKT-148: `WeaponSetType.h` (`GeneralsMD/Code/GameEngine/Include/GameLogic/WeaponSetType.h`)
```cpp
#pragma once

//-------------------------------------------------------------------------------------------------
// IMPORTANT NOTE: you should endeavor to set up states such that the most "normal"
// state is defined by the bit being off. That is, the typical "normal" condition
// has all condition flags set to zero.
//
// IMPORTANT NOTE #2: if you add or modify this list, be sure to update TheWeaponSetNames,
// *and* TheWeaponSetTypeToModelConditionTypeMap!
//
enum WeaponSetType CPP_11(: Int)
{
	// The access and use of this enum has the bit shifting built in, so this is a 0,1,2,3,4,5 enum
	WEAPONSET_VETERAN		= 0,
	WEAPONSET_ELITE,
	WEAPONSET_HERO,
	WEAPONSET_PLAYER_UPGRADE,			// This weapon set flag comes from a purchased upgrade to the player
	WEAPONSET_CRATEUPGRADE_ONE,
	WEAPONSET_CRATEUPGRADE_TWO,
	WEAPONSET_VEHICLE_HIJACK,
	WEAPONSET_CARBOMB,
	WEAPONSET_MINE_CLEARING_DETAIL,
	WEAPONSET_RIDER1, //Kris: Added these for different combat-bike riders
	WEAPONSET_RIDER2,
	WEAPONSET_RIDER3,
	WEAPONSET_RIDER4,
	WEAPONSET_RIDER5,
	WEAPONSET_RIDER6,
	WEAPONSET_RIDER7,
	WEAPONSET_RIDER8,

	WEAPONSET_COUNT
};
```

### PKT-147: `WeaponSet.cpp` (`WeaponSetFlags::s_bitNameList`) (`GeneralsMD/Code/GameEngine/Source/GameLogic/Object/WeaponSet.cpp`)
```cpp
template<>
const char* const WeaponSetFlags::s_bitNameList[] =
{
	"VETERAN",
	"ELITE",
	"HERO",
	"PLAYER_UPGRADE",
	"CRATEUPGRADE_ONE",
	"CRATEUPGRADE_TWO",
	"VEHICLE_HIJACK",
	"CARBOMB",
	"MINE_CLEARING_DETAIL",
	"WEAPON_RIDER1", //Kris: Added these for different combat-bike riders.
	"WEAPON_RIDER2",
	"WEAPON_RIDER3",
	"WEAPON_RIDER4",
	"WEAPON_RIDER5",
	"WEAPON_RIDER6",
	"WEAPON_RIDER7",
	"WEAPON_RIDER8",

	nullptr
};
static_assert(ARRAY_SIZE(WeaponSetFlags::s_bitNameList) == WeaponSetFlags::NumBits + 1, "Incorrect array size");
```

### PKT-149: `WeaponStatus.h` (`GeneralsMD/Code/GameEngine/Include/GameLogic/WeaponStatus.h`)
```cpp
#pragma once

enum WeaponStatus CPP_11(: Int)
{
	READY_TO_FIRE,
	OUT_OF_AMMO,
	BETWEEN_FIRING_SHOTS,
	RELOADING_CLIP,
	PRE_ATTACK,

	WEAPON_STATUS_COUNT
};
```


## 3. Architect Notes & Exact Zero Hour Guard Rules

1. **Zero Hour vs Generals `#ifdef` Branches (`-DRTS_ZEROHOUR=1 -DRTS_GENERALS=0`):**
   - `ALLOW_SURRENDER` is **NOT** defined: omit `KINDOF_PRISON`, `KINDOF_COLLECTS_PRISON_BOUNTY`, `KINDOF_POW_TRUCK`, `KINDOF_CAN_SURRENDER`, and `MODELCONDITION_SURRENDER`.
   - `ALLOW_DEMORALIZE` is **NOT** defined: use `SPECIAL_DEMORALIZE_OBSOLETE` (`"SPECIAL_DEMORALIZE_OBSOLETE"`).
   - `RTS_GENERALS` is **0**: omit `KINDOF_AIRFIELD` (`KindOf.h:82`) and `DAMAGE_FLESHY_SNIPER` (`Damage.h:80`).
2. **Exact Verified Zero Hour Counts (`static_assert`-checked against C++):**
   - `DISABLED_COUNT = 13` (`DisabledType::Default..=ScriptUnderpowered`; `DisabledType::Any = 65535` is read-only sentinel)
   - `KINDOF_COUNT = 117` (`KindOfType::Invalid = -1`; `KindOfType::Obstacle..=NoAttackWarning`)
   - `MODELCONDITION_COUNT = 117` (`ModelConditionFlagType::Invalid = -1`; re-export/reuse `zh_common::xfer::bit_flags::MODEL_CONDITION_BIT_NAMES`)
   - `OBJECT_STATUS_COUNT = 45` (`ObjectStatusType::None..=Deployed`)
   - `SPECIALPOWER_COUNT = 67` (`SpecialPowerType::Invalid = 0` .. `BattleshipBombardment = 66`)
   - `ARMORSET_COUNT = 8` (`ArmorSetType::Veteran..=CrateUpgradeTwo`; reuse `zh_common::xfer::bit_flags::ARMOR_SET_BIT_NAMES`)
   - `DAMAGE_NUM_TYPES = 38` (`DamageType::Explosion..=Status`; also implement `is_subdual_damage` and `is_health_damaging_damage`)
   - `DEATH_NUM_TYPES = 21` (`DeathType::Normal..=PoisonedGamma`; `DEATH_TYPE_NAMES` has 21 entries)
   - `LOCOMOTOR_SURFACE_TYPE_END = 5` (`GROUND = 1 << 0`, `WATER = 1 << 1`, `CLIFF = 1 << 2`, `AIR = 1 << 3`, `RUBBLE = 1 << 4`; `LOCOMOTOR_SURFACE_TYPE_NAMES` has 5 entries)
   - `WEAPONSET_COUNT = 17` (`WeaponSetType::Veteran..=Rider8`)
   - `WEAPON_STATUS_COUNT = 5` (`WeaponStatus::ReadyToFire..=PreAttack`)
3. **Mask Helpers:**
   - Reuse `zh_common::xfer::bit_flags::BitFlags<N>` directly (all `make_mask`, `test`, `any_intersection_with`, `test_set_and_clear`, `get_name_from_single_bit`, `get_single_bit_from_name`, `to_hex_string`, `to_unsigned_int` are already implemented in `zh_common`!).
   - Provide `kindof_mask_fs()` matching `KindOf.cpp:170-188` (`FS_FACTORY`, `FS_BASE_DEFENSE`, `FS_TECHNOLOGY`, `FS_SUPPLY_DROPZONE`, `FS_SUPERWEAPON`, `FS_BLACK_MARKET`, `FS_SUPPLY_CENTER`, `FS_STRATEGY_CENTER`, `FS_FAKE`, `FS_INTERNET_CENTER`, `FS_ADVANCED_TECH`, `FS_BARRACKS`, `FS_WARFACTORY`, `FS_AIRFIELD`).
   - Do **not** pull `ArmorTemplate` (`INI`), `LocomotorTemplate` (`Locomotor`), or `DamageFX` runtime classes into this bundle — only the enums, bitmasks, name tables, and pure predicates listed above.
