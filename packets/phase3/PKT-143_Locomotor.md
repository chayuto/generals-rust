# Translation Packet `PKT-143`: `Locomotor`

## 0. Lean Port & Oracle Contract (`env:ephemeral-web`, Zero Pre-Created Skeletons)
- **Zero-Skeleton Handoff:** No Rust files exist under `crates/zh_logic/src/combat/` for `Locomotor` or `LocomotorSet` yet. Create and export:
  1. `crates/zh_logic/src/combat/locomotor.rs` (`PKT-143`: `LocomotorAppearance`, `LocomotorPriority`, `LocomotorBehaviorZ`, `LocomotorTemplate`, `LocomotorStore`, `Locomotor`)
  2. `crates/zh_logic/src/combat/locomotor_set.rs` (`PKT-144`: `LocomotorSet`)
  3. Wire `pub mod locomotor;` and `pub mod locomotor_set;` in `crates/zh_logic/src/combat/mod.rs` (and re-export key types).
  4. Replace the `#[ignore]` body in `crates/zh_logic/tests/golden_pkt143_locomotor.rs` with the full 1,000-seed differential loop against `docs/golden_vectors/PKT-143.json`, remove `#[ignore]`, and run `python3 scripts/port_map.py --write` to update `docs/PORT_MAP.md`.
- **Reuse Existing Crate Types (Do Not Duplicate):**
  - `zh_logic::rules::combat_types::{LocomotorSurfaceType, LocomotorSurfaceTypeMask, LOCOMOTOR_SURFACE_TYPE_NAMES, NO_SURFACES, ALL_SURFACES}`
  - `zh_logic::gamelogic::global_data::BodyDamageType` (and `MovementPenaltyDamageState` comparison `is_condition_better(a, b)`)
  - `zh_common::ini` scanners, `zh_common::random_value`, `zh_common::math::Coord3D`, `zh_common::xfer::{Snapshot, Xfer, XferSave, XferLoad}`
- **Oracle Build Invariant (`PKT-143.json`):**
  - Header `use_retail_physics_forward_speed_average: 0`: in the C++ reference build (`macos-vulkan`), `USE_RETAIL_PHYSICS_FORWARD_SPEED_AVERAGE()` evaluates to `0`, so `LocomotorTemplate::scale_speed(speed)` returns `speed` unchanged regardless of `legacy_fwd` (`PhysicsBehavior::useLegacyForwardSpeed`).
  - `Locomotor::new` draws exact `GameLogic` RNG sequence when constructed:
    1. `angle_offset = get_game_logic_random_value_real(-PI / 6.0, PI / 6.0)`
    2. `offset_increment = (PI / 40.0) * (get_game_logic_random_value_real(0.8, 1.2) / wander_length_factor)`
    3. `OFFSET_INCREASING` flag `= get_game_logic_random_value(0, 1) != 0`
    4. `donut_timer = current_frame + ((2.5 * 30.0) as u32)` (matching C++ truncation `(UnsignedInt)(frame + 2.5f * 30.0f)`).
- **JSON Vector Contract (`docs/golden_vectors/PKT-143.json`):**
  - Inputs per seed: `ini` (INI block text for `Loco_<seed>`), `ovr` (optional override block applied via `INI_LOAD_CREATE_OVERRIDES`), `bad` + `bad_threw` (must error on invalid `THRUST` with lift/Z-behavior), `rand_seed` + `frame0` (before `Locomotor::new`), `set_rand_seed` + `frame0` (before `LocomotorSet::add_locomotor` sequence; each added `Locomotor` initializes `maintain_pos` to `(0, 0, 0)`), `start_frame`, `move_pen` (`BodyDamageType`), `legacy_fwd`, `mpos` (`[u32; 3]` f32 bits), `ov` (`[ov_spd_bits, ov_turn_bits, ov_acc_bits, ov_lift_bits, ov_brk_bits, close_enough_bits, pref_h_bits, allow_inv, close3d, precise_z, no_slow, ultra]`, where `99999.0_f32` omits the override), `set_names`, `cur_name`.
  - Expected outputs per seed: `rand_crc` (`get_game_logic_random_seed_crc()`), `spd`/`trn`/`acc`/`lft`/`rad`/`tim` (`[u32; 4]` f32 bits across `BodyDamageType` 0..3), `brk`, `lflags`, `lxlen`, `lxcrc`, `set_surf`, `set_dh`, `qhits` (`[&str; 4]` for `GROUND`, `WATER`, `AIR`, `CLIFF|RUBBLE`), `sxlen`, `sxcrc`, plus Seed-0 checkpoints (`s0_tmpl`, `s0_loco` including `l_xhex`, `s0_set` including `s_xhex`).

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-143`
- **Work Package:** `WP-3.3` (Phase 3)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 2: Handle Component
- **Allowed Rust Output File:** `crates/zh_logic/src/combat/locomotor.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (16):**
  - `Common/INI.h`
  - `Common/NameKeyGenerator.h`
  - `Common/Override.h`
  - `Common/Snapshot.h`
  - `GameLogic/AI.h`
  - `GameLogic/AIPathfind.h`
  - `GameLogic/Damage.h`
  - `GameLogic/GameLogic.h`
  - `GameLogic/Locomotor.h`
  - `GameLogic/LocomotorSet.h`
  - `GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/PhysicsUpdate.h`
  - `GameLogic/Object.h`
  - `GameLogic/PartitionManager.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (60):**
  - `"Surfaces" => INI::parseBitString32`
  - `"Speed" => INI::parseVelocityReal`
  - `"SpeedDamaged" => INI::parseVelocityReal`
  - `"TurnRate" => INI::parseAngularVelocityReal`
  - `"TurnRateDamaged" => INI::parseAngularVelocityReal`
  - `"Acceleration" => INI::parseAccelerationReal`
  - `"AccelerationDamaged" => INI::parseAccelerationReal`
  - `"Lift" => INI::parseAccelerationReal`
  - `"LiftDamaged" => INI::parseAccelerationReal`
  - `"Braking" => INI::parseAccelerationReal`
  - `"MinSpeed" => INI::parseVelocityReal`
  - `"MinTurnSpeed" => INI::parseVelocityReal`
  - `"PreferredHeight" => INI::parseReal`
  - `"PreferredHeightDamping" => INI::parseReal`
  - `"CirclingRadius" => INI::parseReal`
  - `"Extra2DFriction" => parseFrictionPerSec`
  - `"SpeedLimitZ" => INI::parseVelocityReal`
  - `"MaxThrustAngle" => INI::parseAngleReal`
  - `"ZAxisBehavior" => INI::parseIndexList`
  - `"Appearance" => INI::parseIndexList`
  - `"GroupMovementPriority" => INI::parseIndexList`
  - `"AccelerationPitchLimit" => INI::parseAngleReal`
  - `"DecelerationPitchLimit" => INI::parseAngleReal`
  - `"BounceAmount" => INI::parseAngularVelocityReal`
  - `"PitchStiffness" => INI::parseReal`
  - `"RollStiffness" => INI::parseReal`
  - `"PitchDamping" => INI::parseReal`
  - `"RollDamping" => INI::parseReal`
  - `"ThrustRoll" => INI::parseReal`
  - `"ThrustWobbleRate" => INI::parseReal`
  - `"ThrustMinWobble" => INI::parseReal`
  - `"ThrustMaxWobble" => INI::parseReal`
  - `"PitchInDirectionOfZVelFactor" => INI::parseReal`
  - `"ForwardVelocityPitchFactor" => INI::parseReal`
  - `"LateralVelocityRollFactor" => INI::parseReal`
  - `"ForwardAccelerationPitchFactor" => INI::parseReal`
  - `"LateralAccelerationRollFactor" => INI::parseReal`
  - `"UniformAxialDamping" => INI::parseReal`
  - `"TurnPivotOffset" => INI::parseReal`
  - `"Apply2DFrictionWhenAirborne" => INI::parseBool`
  - `"DownhillOnly" => INI::parseBool`
  - `"AllowAirborneMotiveForce" => INI::parseBool`
  - `"LocomotorWorksWhenDead" => INI::parseBool`
  - `"AirborneTargetingHeight" => INI::parseInt`
  - `"StickToGround" => INI::parseBool`
  - `"CanMoveBackwards" => INI::parseBool`
  - `"HasSuspension" => INI::parseBool`
  - `"FrontWheelTurnAngle" => INI::parseAngleReal`
  - `"MaximumWheelExtension" => INI::parseReal`
  - `"MaximumWheelCompression" => INI::parseReal`
  - `"CloseEnoughDist" => INI::parseReal`
  - `"CloseEnoughDist3D" => INI::parseBool`
  - `"SlideIntoPlaceTime" => INI::parseDurationReal`
  - `"WanderWidthFactor" => INI::parseReal`
  - `"WanderLengthFactor" => INI::parseReal`
  - `"WanderAboutPointRadius" => INI::parseReal`
  - `"RudderCorrectionDegree" => INI::parseReal`
  - `"RudderCorrectionRate" => INI::parseReal`
  - `"ElevatorCorrectionDegree" => INI::parseReal`
  - `"ElevatorCorrectionRate" => INI::parseReal`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (26 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUnsignedInt(&m_donutTimer)`
  3. `xferCoord3D(&m_maintainPos)`
  4. `xferReal(&m_brakingFactor)`
  5. `xferReal(&m_maxLift)`
  6. `xferReal(&m_maxSpeed)`
  7. `xferReal(&m_maxAccel)`
  8. `xferReal(&m_maxBraking)`
  9. `xferReal(&m_maxTurnRate)`
  10. `xferReal(&m_closeEnoughDist)`
  11. `xferUnsignedInt(&m_flags)`
  12. `xferReal(&m_preferredHeight)`
  13. `xferReal(&m_preferredHeightDamping)`
  14. `xferReal(&m_angleOffset)`
  15. `xferReal(&m_offsetIncrement)`
  16. `xferVersion( &version, currentVersion )`
  17. `xferUnsignedShort( &count )`
  18. `xferAsciiString(&name)`
  19. `xferSnapshot(loco)`
  20. `xferAsciiString(&name)`
  21. `xferSnapshot(loco)`
  22. `xferInt(&m_validLocomotorSurfaces)`
  23. `xferBool(&m_downhillOnly)`
  24. `xferSnapshot(this)`
  25. `xferAsciiString(&name)`
  26. `xferAsciiString(&name)`
- **PRNG Calls (3 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValueReal(-PI/6, PI/6)`
  - `GameLogicRandomValueReal(0.8f, 1.2f)/m_template->m_wanderLengthFactor)`
  - `GameLogicRandomValue(0,1))`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Locomotor.h`
```cpp
// FILE: Locomotor.h /////////////////////////////////////////////////////////////////////////////////
// Author: Steven Johnson, Feb 2002
// Desc: Locomotor Descriptions
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/NameKeyGenerator.h"
#include "Common/Override.h"
#include "Common/Snapshot.h"
#include "GameLogic/Damage.h"
#include "GameLogic/LocomotorSet.h"

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
class Locomotor;
class LocomotorTemplate;
class INI;
class PhysicsBehavior;
enum BodyDamageType CPP_11(: Int);
enum PhysicsTurningType CPP_11(: Int);

// if we ever re-enable jets circling for landing, we need this. so keep in around just in case. (srj)
#define NO_CIRCLE_FOR_LANDING

//-------------------------------------------------------------------------------------------------
enum LocomotorAppearance CPP_11(: Int)
{
	LOCO_LEGS_TWO,
	LOCO_WHEELS_FOUR,
	LOCO_TREADS,
	LOCO_HOVER,
	LOCO_THRUST,
	LOCO_WINGS,
	LOCO_CLIMBER,			// human climber - backs down cliffs.
	LOCO_OTHER,
	LOCO_MOTORCYCLE, // Added in Zero Hour

	LOCOMOTOR_APPEARANCE_COUNT
};

enum LocomotorPriority CPP_11(: Int)
{
	LOCO_MOVES_BACK,				// In a group, this one moves toward the back
	LOCO_MOVES_MIDDLE,			// In a group, this one stays in the middle
	LOCO_MOVES_FRONT,				// In a group, this one moves toward the front of the group

	LOCOMOTOR_PRIORITY_COUNT
};

#ifdef DEFINE_LOCO_APPEARANCE_NAMES
static const char *const TheLocomotorAppearanceNames[] =
{
	"TWO_LEGS",
	"FOUR_WHEELS",
	"TREADS",
	"HOVER",
	"THRUST",
	"WINGS",
	"CLIMBER",
	"OTHER",
	"MOTORCYCLE",

	nullptr
};
static_assert(ARRAY_SIZE(TheLocomotorAppearanceNames) == LOCOMOTOR_APPEARANCE_COUNT + 1, "Incorrect array size");
#endif

//-------------------------------------------------------------------------------------------------
enum LocomotorBehaviorZ CPP_11(: Int)
{
	Z_NO_Z_MOTIVE_FORCE,							// does whatever physics tells it, but has no z-force of its own.
	Z_SEA_LEVEL,											// keep at surface-of-water level
	Z_SURFACE_RELATIVE_HEIGHT,				// try to follow a specific height relative to terrain/water height
	Z_ABSOLUTE_HEIGHT,								// try follow a specific height regardless of terrain/water height
	Z_FIXED_SURFACE_RELATIVE_HEIGHT,		// stays fixed at surface-rel height, regardless of physics
	Z_FIXED_ABSOLUTE_HEIGHT,						// stays fixed at absolute height, regardless of physics
	Z_RELATIVE_TO_GROUND_AND_BUILDINGS,	// stays fixed at surface-rel height including buildings, regardless of physics
	Z_SMOOTH_RELATIVE_TO_HIGHEST_LAYER,	// try to follow a height relative to the highest layer.

	LOCOMOTOR_BEHAVIOR_Z_COUNT
};

#ifdef DEFINE_LOCO_Z_NAMES
static const char *const TheLocomotorBehaviorZNames[] =
{
	"NO_Z_MOTIVE_FORCE",
	"SEA_LEVEL",
	"SURFACE_RELATIVE_HEIGHT",
	"ABSOLUTE_HEIGHT",
	"FIXED_SURFACE_RELATIVE_HEIGHT",
	"FIXED_ABSOLUTE_HEIGHT",
	"FIXED_RELATIVE_TO_GROUND_AND_BUILDINGS",
	"RELATIVE_TO_HIGHEST_LAYER",

	nullptr
};
static_assert(ARRAY_SIZE(TheLocomotorBehaviorZNames) == LOCOMOTOR_BEHAVIOR_Z_COUNT + 1, "Incorrect array size");
#endif

//-------------------------------------------------------------------------------------------------
class LocomotorTemplate : public Overridable
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( LocomotorTemplate, "LocomotorTemplate"  )
	friend class Locomotor;

public:

	LocomotorTemplate();

	/// field table for loading the values from an INI
	const FieldParse* getFieldParse() const;

	void friend_setName(const AsciiString& n) { m_name = n; }

	void validate();

private:

	/// TheSuperHackers @info Speeds authored in INI are understated by the forward speed the Locomotor measures
	/// itself with, which is what made objects move faster on diagonal headings than on axis aligned ones.
	/// These accessors hand out the authored speed compensated for that, in world distance per logic frame.
	/// They all return the authored value unchanged in retail compatible builds.
	Real getActualMaxSpeed() const;
	Real getActualMaxSpeedDamaged() const;
	Real getActualMinSpeed() const;
	Real getActualMinTurnSpeed() const;

	/// Compensate an authored speed. Returns it unchanged when the mover is commanded with retail speeds.
	Real scaleSpeed(Real speed) const;

	/**
		Units check:

		-- Velocity:			dist/frame
		-- Acceleration:	dist/(frame*frame)
		-- Forces:				(mass*dist)/(frame*frame)
	*/
	AsciiString								m_name;
	LocomotorSurfaceTypeMask	m_surfaces;							///< flags indicating the kinds of surfaces we can use
	Real											m_maxSpeed;							///< max speed
	Real											m_maxSpeedDamaged;			///< max speed when "damaged"
	Real											m_minSpeed;							///< we should never brake past this
	Real											m_maxTurnRate;					///< max rate at which we can turn, in rads/frame
	Real											m_maxTurnRateDamaged;		///< max turn rate when "damaged"
	Real											m_acceleration;					///< max acceleration
	Real											m_accelerationDamaged;	///< max acceleration when damaged
	Real											m_lift;									///< max lifting acceleration (flying objects only)
	Real											m_liftDamaged;					///< max lift when damaged
	Real											m_braking;							///< max braking (deceleration)
	Real											m_minTurnSpeed;					///< we must be going >= this speed in order to turn
#if USE_RETAIL_PHYSICS_FORWARD_SPEED_AVERAGE()
	Real											m_speedScale;						///< factor that turns an authored speed into its compensated speed
#endif
	Real											m_preferredHeight;			///< our preferred height (if flying)
	Real											m_preferredHeightDamping;		///< how aggressively to adjust to preferred height: 1.0 = very much so, 0.1 = gradually, etc
	Real											m_circlingRadius;				///< for flying things, the radius at which they circle their "maintain" destination. (pos = cw, neg = ccw, 0 = smallest possible)
	Real											m_speedLimitZ;					///< try to avoid going up or down at more than this speed, if possible
	Real											m_extra2DFriction;			///< extra 2dfriction to apply (via Physics)
	Real											m_maxThrustAngle;				///< THRUST locos only: how much we deflect our thrust angle
	LocomotorBehaviorZ				m_behaviorZ;						///< z-axis behavior
	LocomotorAppearance				m_appearance;						///< how we should diddle the Drawable to imitate this motion
	LocomotorPriority					m_movePriority;					///< Where we move - front, middle, back.

	Real											m_accelPitchLimit;			///< Maximum amount we will pitch up  under acceleration (including recoil.)
	Real											m_decelPitchLimit;			///< Maximum amount we will pitch down under deceleration (including recoil.)
	Real											m_bounceKick;						///< How much simulating rough terrain "bounces" a wheel up.
	Real											m_pitchStiffness;				///< How stiff the springs are forward & back.
	Real											m_rollStiffness;				///< How stiff the springs are side to side.
	Real											m_pitchDamping;					///< How good the shock absorbers are.
	Real											m_rollDamping;					///< How good the shock absorbers are.
	Real											m_pitchByZVelCoef;			///< How much we pitch in response to z-speed.
	Real											m_thrustRoll;						///< Thrust roll around X axis
	Real											m_wobbleRate;						///< how fast thrust things "wobble"
	Real											m_minWobble;						///< how much thrust things "wobble"
	Real											m_maxWobble;						///< how much thrust things "wobble"
	Real											m_forwardVelCoef;				///< How much we pitch in response to speed.
	Real											m_lateralVelCoef;				///< How much we roll in response to speed.
	Real											m_forwardAccelCoef;			///< How much we pitch in response to acceleration.
	Real											m_lateralAccelCoef;			///< How much we roll in response to acceleration.
	Real											m_uniformAxialDamping;	///< For Attenuating the pitch and roll rates
	Real											m_turnPivotOffset;			///< should we pivot around noncenter? (-1.0 = rear, 0.0 = center, 1.0 = front)
	Int												m_airborneTargetingHeight;	///< The height transition at witch I should mark myself as a AA target.

	Real											m_closeEnoughDist;			///< How close we have to approach the end of a path before stopping
	Bool											m_isCloseEnoughDist3D;	///< And is that calculation 3D, for very rare cases that need to move straight down.
	Real											m_ultraAccurateSlideIntoPlaceFactor;			///< how much we can fudge turning when ultra-accurate

	Bool											m_locomotorWorksWhenDead;	///< should locomotor continue working even when object is "dead"?
	Bool											m_allowMotiveForceWhileAirborne;	///< can we apply motive when airborne?
	Bool											m_apply2DFrictionWhenAirborne;	// apply "2d friction" even when airborne... useful for realistic-looking movement
	Bool											m_downhillOnly;	// pinewood derby, moves only by gravity pulling downhill
	Bool											m_stickToGround;				// if true, can't leave ground
	Bool											m_canMoveBackward;				// if true, can move backwards.
	Bool											m_hasSuspension;				///< If true, calculate 4 wheel independent suspension values.
	Real											m_maximumWheelExtension; ///< Maximum distance wheels can move down.  (negative value)
	Real											m_maximumWheelCompression; ///< Maximum distance wheels can move up.  (positive value)
	Real											m_wheelTurnAngle;				///< How far the front wheels can turn.

	// Fields for wander locomotor
	Real											m_wanderWidthFactor;
	Real											m_wanderLengthFactor;
	Real											m_wanderAboutPointRadius;


	Real											m_rudderCorrectionDegree;
	Real											m_rudderCorrectionRate;
	Real											m_elevatorCorrectionDegree;
	Real											m_elevatorCorrectionRate;
};

typedef OVERRIDE<LocomotorTemplate> LocomotorTemplateOverride;

// ---------------------------------------------------------
class Locomotor : public MemoryPoolObject, public Snapshot
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(Locomotor, "Locomotor" )

	friend class LocomotorStore;

public:

	void setPhysicsOptions(Object* obj);

	void locoUpdate_moveTowardsPosition(Object* obj, const Coord3D& goalPos,
		Real onPathDistToGoal, Real desiredSpeed, Bool *blocked);
	void locoUpdate_moveTowardsAngle(Object* obj, Real angle);
	/**
		Kill any current (2D) velocity (but stay at current position, or as close as possible)

		return true if we can maintain the position without being called every frame (eg, we are
		resting on the ground), false if not (eg, we are hovering or circling)
	*/
	Bool locoUpdate_maintainCurrentPosition(Object* obj);

	Real getMaxSpeedForCondition(BodyDamageType condition) const;  ///< get max speed given condition
	Real getMaxTurnRate(BodyDamageType condition) const;  ///< get max turning rate given condition
	Real getMaxAcceleration(BodyDamageType condition) const;  ///< get acceleration given condition
	Real getMaxLift(BodyDamageType condition) const;  ///< get acceleration given condition
	Real getBraking() const;  ///< get braking given condition

	Real getPreferredHeight() const { return m_preferredHeight;} ///< Just return preferredheight, no damage consideration
	void restorePreferredHeightFromTemplate() { m_preferredHeight = m_template->m_preferredHeight; };
	Real getPreferredHeightDamping() const { return m_preferredHeightDamping;}
	LocomotorAppearance getAppearance() const { return m_template->m_appearance; }
	LocomotorPriority getMovePriority() const { return m_template->m_movePriority; }
	LocomotorSurfaceTypeMask getLegalSurfaces() const { return m_template->m_surfaces; }

	AsciiString getTemplateName() const { return m_template->m_name;}
	Real getMinSpeed() const;
	Real getMinTurnSpeed() const;
	Real getAccelPitchLimit() const { return m_template->m_accelPitchLimit;}	///< Maximum amount we will pitch up or down under acceleration (including recoil.)
	Real getDecelPitchLimit() const { return m_template->m_decelPitchLimit;}	///< Maximum amount we will pitch down under deceleration (including recoil.)
	Real getBounceKick() const { return m_template->m_bounceKick;}						///< How much simulating rough terrain "bounces" a wheel up.
	Real getPitchStiffness() const { return m_template->m_pitchStiffness;}			///< How stiff the springs are forward & back.
	Real getRollStiffness() const { return m_template->m_rollStiffness;}				///< How stiff the springs are side to side.
	Real getPitchDamping() const { return m_template->m_pitchDamping;}	///< How good the shock absorbers are.
	Real getRollDamping() const { return m_template->m_rollDamping;}	///< How good the shock absorbers are.
	Real getPitchByZVelCoef() const { return m_template->m_pitchByZVelCoef;}		///< How much we pitch in response to speed.
	Real getThrustRoll() const { return m_template->m_thrustRoll; }  ///< Thrust roll
	Real getWobbleRate() const { return m_template->m_wobbleRate; }  ///< how fast thrust things "wobble"
	Real getMaxWobble() const { return m_template->m_maxWobble; }  ///< how much thrust things "wobble"
	Real getMinWobble() const { return m_template->m_minWobble; }  ///< how much thrust things "wobble"

	Real getForwardVelCoef() const { return m_template->m_forwardVelCoef;}		///< How much we pitch in response to speed.
	Real getLateralVelCoef() const { return m_template->m_lateralVelCoef;}			///< How much we roll in response to speed.
	Real getForwardAccelCoef() const { return m_template->m_forwardAccelCoef;}		///< How much we pitch in response to acceleration.
	Real getLateralAccelCoef() const { return m_template->m_lateralAccelCoef;}			///< How much we roll in response to acceleration.
	Real getUniformAxialDamping() const { return m_template->m_uniformAxialDamping;}			///< How much we roll in response to acceleration.
	Real getTurnPivotOffset() const { return m_template->m_turnPivotOffset;}
	Bool getApply2DFrictionWhenAirborne() const { return m_template->m_apply2DFrictionWhenAirborne; }
	Bool getIsDownhillOnly() const { return m_template->m_downhillOnly; }
	Bool getAllowMotiveForceWhileAirborne() const { return m_template->m_allowMotiveForceWhileAirborne; }
	Int getAirborneTargetingHeight() const { return m_template->m_airborneTargetingHeight; }
	Bool getLocomotorWorksWhenDead() const { return m_template->m_locomotorWorksWhenDead; }
	Bool getStickToGround() const { return m_template->m_stickToGround; }
	Real getCloseEnoughDist() const { return m_closeEnoughDist; }
	Bool isCloseEnoughDist3D() const { return getFlag(IS_CLOSE_ENOUGH_DIST_3D); }
	Bool hasSuspension() const {return m_template->m_hasSuspension;}
	Bool canMoveBackwards() const {return m_template->m_canMoveBackward;}
	Real getMaxWheelExtension() const {return m_template->m_maximumWheelExtension;}
	Real getMaxWheelCompression() const {return m_template->m_maximumWheelCompression;}
	Real getWheelTurnAngle() const {return m_template->m_wheelTurnAngle;}


	Real getRudderCorrectionDegree()	  const { return m_template->m_rudderCorrectionDegree;}			///< How much we roll in response to acceleration.
	Real getRudderCorrectionRate()	    const { return m_template->m_rudderCorrectionRate;}			///< How much we roll in response to acceleration.
	Real getElevatorCorrectionDegree() const { return m_template->m_elevatorCorrectionDegree;}			///< How much we roll in response to acceleration.
	Real getElevatorCorrectionRate()	  const { return m_template->m_elevatorCorrectionRate;}			///< How much we roll in response to acceleration.


	Real getWanderWidthFactor() const {return m_template->m_wanderWidthFactor;}
	Real getWanderAboutPointRadius() const {return m_template->m_wanderAboutPointRadius;}

	Real calcMinTurnRadius(BodyDamageType condition, Real* timeToTravelThatDist) const;

	/// this is handy for doing things like forcing helicopters to crash realistically: cut their lift.
	void setMaxLift(Real lift) { m_maxLift = lift; }
	void setMaxSpeed(Real speed)
	{
		DEBUG_ASSERTCRASH(!(speed <= 0.0f && m_template->m_appearance == LOCO_THRUST), ("THRUST locos may not have zero speeds!"));
		m_maxSpeed = speed;
	}
	void setMaxSpeedToMinSpeed() { setMaxSpeed(m_template->m_minSpeed); }
	void setMaxAcceleration(Real accel) { m_maxAccel = accel; }
	void setMaxBraking(Real braking) { m_maxBraking = braking; }
	void setMaxTurnRate(Real turn) { m_maxTurnRate = turn; }
	void setAllowInvalidPosition(Bool allow) { setFlag(ALLOW_INVALID_POSITION, allow); }
	void setCloseEnoughDist( Real dist ) { m_closeEnoughDist = dist; }
	void setCloseEnoughDist3D( Bool setting ) { setFlag(IS_CLOSE_ENOUGH_DIST_3D, setting); }
	Bool isInvalidPositionAllowed() const { return getFlag( ALLOW_INVALID_POSITION ); }

	void setPreferredHeight( Real height ) { m_preferredHeight = height; }

#ifdef CIRCLE_FOR_LANDING
	/**
		if we are climbing/diving more than this, circle as needed rather
		than just diving or climbing directly. (only useful for Winged things)
	*/
	inline void setAltitudeChangeThresholdForCircling(Real a) { m_circleThresh = a; }
#endif

	/**
		when off (the default), things get to adjust their z-pos as their
		loco says (in particular, airborne things tend to try to fly at a preferred height).

		when on, they do their best to reach the specified zpos, even if it's not at their preferred height.
		this is used mainly for force missiles to swoop in on their target, and to force airplane takeoff/landing
		to go smoothly.
	*/
	void setUsePreciseZPos(Bool u) { setFlag(PRECISE_Z_POS, u); }

	/**
    when off (the default), units slow down as they approach their target.

    when on, units go full speed till the end, and may overshoot their target.
    this is useful mainly in some weird, temporary situations where we know we are
    going to follow this move with another one...	or for carbombs.
	*/
	void setNoSlowDownAsApproachingDest(Bool u) { setFlag(NO_SLOW_DOWN_AS_APPROACHING_DEST, u); }

	/**
    when off (the default), units do their normal stuff.

    when on, we cheat and make very precise motion, regardless of loco settings.
    this is accomplished by cranking up the unit's turning rate, friction, lift (for airborne things),
    and possibly other things. This is useful mainly when doing maneuvers where precision
    is VITAL, such as airplane takeoff/landing.

		For ground units, it also allows units to have a destination off of a pathfing grid.

	*/
	void setUltraAccurate(Bool u) { setFlag(ULTRA_ACCURATE, u); }
	Bool isUltraAccurate() const { return getFlag(ULTRA_ACCURATE); }

	Bool isMovingBackwards() const {return getFlag(MOVING_BACKWARDS);}

	void startMove(); ///< Indicates that a move is starting, primarily to reset the donut timer. jba.

protected:
	Real getMaxSpeedOverride() const;

	void moveTowardsPositionLegs(Object* obj, PhysicsBehavior *physics, const Coord3D& goalPos, Real onPathDistToGoal, Real desiredSpeed);
	void moveTowardsPositionClimb(Object* obj, PhysicsBehavior *physics, const Coord3D& goalPos, Real onPathDistToGoal, Real desiredSpeed);
	void moveTowardsPositionWheels(Object* obj, PhysicsBehavior *physics, const Coord3D& goalPos, Real onPathDistToGoal, Real desiredSpeed);
	void moveTowardsPositionTreads(Object* obj, PhysicsBehavior *physics, const Coord3D& goalPos, Real onPathDistToGoal, Real desiredSpeed);
	void moveTowardsPositionOther(Object* obj, PhysicsBehavior *physics, const Coord3D& goalPos, Real onPathDistToGoal, Real desiredSpeed);
	void moveTowardsPositionHover(Object* obj, PhysicsBehavior *physics, const Coord3D& goalPos, Real onPathDistToGoal, Real desiredSpeed);
	void moveTowardsPositionThrust(Object* obj, PhysicsBehavior *physics, const Coord3D& goalPos, Real onPathDistToGoal, Real desiredSpeed);
	void moveTowardsPositionWings(Object* obj, PhysicsBehavior *physics, const Coord3D& goalPos, Real onPathDistToGoal, Real desiredSpeed);

	void maintainCurrentPositionThrust(Object* obj, PhysicsBehavior *physics);
	void maintainCurrentPositionOther(Object* obj, PhysicsBehavior *physics);
	void maintainCurrentPositionLegs(Object* obj, PhysicsBehavior *physics) { maintainCurrentPositionOther(obj, physics); }
	void maintainCurrentPositionWheels(Object* obj, PhysicsBehavior *physics) { maintainCurrentPositionOther(obj, physics); }
	void maintainCurrentPositionTreads(Object* obj, PhysicsBehavior *physics) { maintainCurrentPositionOther(obj, physics); }
	void maintainCurrentPositionHover(Object* obj, PhysicsBehavior *physics);
	void maintainCurrentPositionWings(Object* obj, PhysicsBehavior *physics);

	PhysicsTurningType rotateTowardsPosition(Object* obj, const Coord3D& goalPos, Real *relAngle=nullptr);

	/*
		return true if we can maintain the position without being called every frame (eg, we are
		resting on the ground), false if not (eg, we are hovering or circling)
	*/
	Bool handleBehaviorZ(Object* obj, PhysicsBehavior *physics, const Coord3D& goalPos);
	PhysicsTurningType rotateObjAroundLocoPivot(Object* obj, const Coord3D& goalPos, Real maxTurnRate, Real *relAngle = nullptr);

	Real getSurfaceHtAtPt(Real x, Real y);
	Real calcLiftToUseAtPt(Object* obj, PhysicsBehavior *physics, Real curZ, Real surfaceAtPt, Real preferredHeight);

	Bool fixInvalidPosition(Object* obj, PhysicsBehavior *physics);

protected:
	// snapshot methods
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

protected:

	Locomotor(const LocomotorTemplate* tmpl);

	// Note, "Law of the Big Three" applies here
	//Locomotor(); -- nope, we don't have a default ctor. (srj)
	Locomotor(const Locomotor& that);
	Locomotor& operator=(const Locomotor& that);
	//~Locomotor();

private:

	//
	// Note: these values are saved in save files, so you MUST NOT REMOVE OR CHANGE
	// existing values!
	//
	enum LocoFlag
	{
		IS_BRAKING = 0,
		ALLOW_INVALID_POSITION,
		MAINTAIN_POS_IS_VALID,
		PRECISE_Z_POS,
		NO_SLOW_DOWN_AS_APPROACHING_DEST,
		OVER_WATER,					// To allow things to move slower/faster over water and do special effects
		ULTRA_ACCURATE,
		MOVING_BACKWARDS,				// If we are moving backwards.
		DOING_THREE_POINT_TURN,	// If we are doing a 3 pt turn.
		CLIMBING,								// If we are in the process of climbing.
		IS_CLOSE_ENOUGH_DIST_3D,
		OFFSET_INCREASING
	};

	Bool getFlag(LocoFlag f) const { return (m_flags & (1 << f)) != 0; }
	void setFlag(LocoFlag f, Bool b) { if (b) m_flags |= (1<<f); else m_flags &= ~(1<<f); }

	LocomotorTemplateOverride m_template;		///< the kind of Locomotor this is
	Coord3D			m_maintainPos;
	Real				m_brakingFactor;
	Real				m_maxLift;
	Real				m_maxSpeed;
	Real				m_maxAccel;
	Real				m_maxBraking;
	Real				m_maxTurnRate;
	Real				m_closeEnoughDist;
#ifdef CIRCLE_FOR_LANDING
	Real				m_circleThresh;
#endif
	UnsignedInt	m_flags;
	Real				m_preferredHeight;
	Real				m_preferredHeightDamping;

	Real				m_angleOffset;
	Real				m_offsetIncrement;
	UnsignedInt m_donutTimer;				///< Frame time to keep units from doing the donut. jba.


};

//-------------------------------------------------------------------------------------------------
/**
	The "store" used to hold all the LocomotorTemplates in existence. This is usually used when creating
	an Object, but can be used at any time after that. (It is explicitly
	OK to swap an Object's Locomotor out at any given time.)
*/
//-------------------------------------------------------------------------------------------------
class LocomotorStore : public SubsystemInterface
{
public:

	LocomotorStore();
	virtual ~LocomotorStore() override;

	virtual void init() override { };
	virtual void reset() override;
	virtual void update() override;

	/**
		Find the LocomotorTemplate with the given name. If no such LocomotorTemplate exists, return null.
	*/
	const LocomotorTemplate* findLocomotorTemplate(NameKeyType namekey) const;
	LocomotorTemplate* findLocomotorTemplate(NameKeyType namekey);

	Locomotor* newLocomotor(const LocomotorTemplate *tmpl) const
	{
		return newInstance(Locomotor)(tmpl);	// my, that was easy
	}

	// locoTemplate is who we're overriding
	LocomotorTemplate *newOverride(LocomotorTemplate *locoTemplate);


	static void parseLocomotorTemplateDefinition(INI* ini);

protected:

private:

	typedef std::map< NameKeyType, LocomotorTemplate*, std::less<NameKeyType>/**/> LocomotorTemplateMap;

	LocomotorTemplateMap m_locomotorTemplates;

};

// EXTERNALS //////////////////////////////////////////////////////////////////////////////////////
extern LocomotorStore *TheLocomotorStore;
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Locomotor.cpp`
```cpp
// FILE: Locomotor.cpp ///////////////////////////////////////////////////////////////////////////////
// Author: Steven Johnson, Feb 2002
// Desc:   Locomotor descriptions
///////////////////////////////////////////////////////////////////////////////////////////////////


// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include <climits>	// For INT_MAX on Linux
#define DEFINE_SURFACECATEGORY_NAMES
#define DEFINE_LOCO_Z_NAMES
#define DEFINE_LOCO_APPEARANCE_NAMES

#include "Common/INI.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Locomotor.h"
#include "GameLogic/Object.h"
#include "GameLogic/AI.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/AIUpdate.h"


static const Real DONUT_TIME_DELAY_SECONDS=2.5f;
static const Real DONUT_DISTANCE=4.0*PATHFIND_CELL_SIZE_F;


#define MAX_BRAKING_FACTOR 5.0f
///////////////////////////////////////////////////////////////////////////////////////////////////
// PUBLIC DATA ////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
LocomotorStore *TheLocomotorStore = nullptr;					///< the Locomotor store definition

const Real BIGNUM = 99999.0f;

static const char *const TheLocomotorPriorityNames[] =
{
	"MOVES_BACK",
	"MOVES_MIDDLE",
	"MOVES_FRONT",

	nullptr
};
static_assert(ARRAY_SIZE(TheLocomotorPriorityNames) == LOCOMOTOR_PRIORITY_COUNT + 1, "Incorrect array size");

///////////////////////////////////////////////////////////////////////////////////////////////////
// PRIVATE DATA ///////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTIONS //////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

#if USE_RETAIL_PHYSICS_FORWARD_SPEED_AVERAGE()

// TheSuperHackers @bugfix xezon 30/07/2026 The compensation that equalizes straight and diagonal
// movement speeds at their harmonic mean.
//
// Retail measured forward speed as sqrt(sum of (vi * di)^2) rather than the true projection of the
// velocity onto the heading, sum of (vi * di). For a unit heading d that understates the speed by
// sqrt(sum of di^4), which is 1 on an axis aligned heading and falls to 1/sqrt(2) on a 2d diagonal
// and 1/sqrt(3) on a 3d body diagonal. The 2d movers compare that measurement directly against
// their goal speed, so they kept accelerating until the real speed was between 1x and sqrt(2)x the
// authored speed, decided by nothing but which way the object happened to face.
//
// getForwardSpeed2D and getForwardSpeed3D now report the true projection, so the real speed equals
// the commanded speed on every heading. The speeds commanded by the 2d movers are scaled by a single
// constant, which picks where inside the old range that now uniform speed sits.
//
// The constant is the harmonic mean of the two ends of the old range, 1 and sqrt(2). Axis aligned
// headings become exactly as much faster as diagonal headings become slower, 17.16% each, so no
// heading changes its speed by more than that, whatever mix of headings a map produces.
//
// Averaging the old factor over all headings instead would preserve the average pace of the game,
// but only for an even mix of headings, and the result depends on whether that mix is even over
// distance travelled (harmonic mean, 1.1630) or over time spent (arithmetic mean, 1.1803). The
// constant sits between the two, within 0.75% of both. The midpoint of the old range, 1.2071, lies
// above all of them and would quietly speed the whole game up.
//
// THRUST, the only mover that measures itself with getForwardSpeed3D, is deliberately not scaled.
// Its thrust is damped by the velocity times acceleration over max speed, which cancels the thrust
// at exactly the max speed on every heading, so the understated measurement never raised its speed.
// Scaling it would make missiles faster than retail.

constexpr const Real DiagonalCompensation2D = 1.17157288f; // 2*sqrt(2) / (1 + sqrt(2)) = 4 - 2*sqrt(2)

#endif

//-------------------------------------------------------------------------------------------------
static Real calcSlowDownDist(Real curSpeed, Real desiredSpeed, Real maxBraking)
{
	Real delta = curSpeed - desiredSpeed;
	if (delta <= 0)
		return 0.0f;

	Real dist = (sqr(delta) / WWMath::Fabs(maxBraking)) * 0.5f;

	// use a little fudge so that things can stop "on a dime" more easily...
	const Real FUDGE = 1.05f;
	return dist * FUDGE;
}

//-----------------------------------------------------------------------------
inline Bool isNearlyZero(Real a)
{
	const Real TINY_EPSILON = 0.001f;
	return WWMath::Fabs(a) < TINY_EPSILON;
}

//-----------------------------------------------------------------------------
inline Bool isNearly(Real a, Real val)
{
	const Real TINY_EPSILON = 0.001f;
	return WWMath::Fabs(a - val) < TINY_EPSILON;
}

//-----------------------------------------------------------------------------
// return the angle delta (in 3-space) we turned.
static Real tryToRotateVector3D(
	Real maxAngle,						// if negative, it's a percent (0...1) of the dist to rotate 'em
	const Vector3& inCurDir,
	const Vector3& inGoalDir,
	Vector3& actualDir
)
{
	if (isNearlyZero(maxAngle))
	{
		actualDir = inCurDir;
		return 0.0f;
	}

	Vector3 curDir = inCurDir;
	curDir.Normalize();

	Vector3 goalDir = inGoalDir;
	goalDir.Normalize();

	// dot of two unit vectors is cos of angle between them.
	Real cosine = Vector3::Dot_Product(curDir, goalDir);
	// bound it in case of numerical error
	Real angleBetween = (Real)ACos(clamp(-1.0f, cosine, 1.0f));

	if (maxAngle < 0)
	{
		maxAngle = -maxAngle * angleBetween;
		if (isNearlyZero(maxAngle))
		{
			actualDir = inCurDir;
			return 0.0f;
		}
	}

	if (WWMath::Fabs(angleBetween) <= maxAngle)
	{
		// close enough
		actualDir = goalDir;
	}
	else
	{
		// nah, try as much as we can in the right dir.
		// we need to rotate around the axis perpendicular to these two vecs.
		// but: cross of two vectors is the perpendicular axis!
#ifdef ALLOW_TEMPORARIES
		Vector3 objCrossGoal = Vector3::Cross_Product(curDir, goalDir);
		objCrossGoal.Normalize();
#else
		Vector3 objCrossGoal;
		Vector3::Normalized_Cross_Product(curDir, goalDir, &objCrossGoal);
#endif

		angleBetween = maxAngle;
		Matrix3D rotMtx(objCrossGoal, angleBetween);
		actualDir = rotMtx.Rotate_Vector(curDir);
	}

	return angleBetween;
}

//-------------------------------------------------------------------------------------------------
static Real tryToOrientInThisDirection3D(Object* obj, Real maxTurnRate, const Vector3& desiredDir)
{
	Vector3 actualDir;
	Real relAngle = tryToRotateVector3D(maxTurnRate, obj->getTransformMatrix()->Get_X_Vector(), desiredDir, actualDir);
	if (relAngle != 0.0f)
	{
		Vector3 objPos(obj->getPosition()->x, obj->getPosition()->y, obj->getPosition()->z);

		Matrix3D newXform;
		newXform.buildTransformMatrix( objPos, actualDir );

		obj->setTransformMatrix( &newXform );
	}
	return relAngle;
}

//-------------------------------------------------------------------------------------------------
inline Real tryToOrientInThisDirection3D(Object* obj, Real maxTurnRate, const Coord3D* dir)
{
	return tryToOrientInThisDirection3D(obj, maxTurnRate, Vector3(dir->x, dir->y, dir->z));
}

//-----------------------------------------------------------------------------
static void calcDirectionToApplyThrust(
	const Object* obj,
	const PhysicsBehavior* physics,
	const Coord3D& ingoalPos,
	Real maxAccel,
	Vector3& goalDir
)
{
	/*
		our meta-goal here is to calculate the direction we should apply our motive force
		in order to minimize the angle between (our velocity) and (direction towards goalpos).

		this is complicated by the fact that we generally have an intrinsic velocity already,
		that must be accounted for, and by the fact that we can only apply force in our
		forward-x-direction (with a thrust-angle-range), and (due to limited range) might not
		be able to apply the force in the optimal direction!
	*/

	// convert to Vector3, to use all its handy stuff
	Vector3 objPos(obj->getPosition()->x, obj->getPosition()->y, obj->getPosition()->z);
	Vector3 goalPos(ingoalPos.x, ingoalPos.y, ingoalPos.z);

	Vector3 vecToGoal = goalPos - objPos;
	if (isNearlyZero(vecToGoal.Length2()))
	{
		// goal pos is essentially same as current pos, so just stay the same & return
		goalDir = obj->getTransformMatrix()->Get_X_Vector();
		return;
	}

	/*
		get our cur vel into a useful Vector3 form
	*/
	Vector3 curVel(physics->getVelocity()->x, physics->getVelocity()->y, physics->getVelocity()->z);

	// add gravity to our vel so that we account for it in our calcs
	curVel.Z += TheGlobalData->m_gravity;

	Bool foundSolution = false;
	Real distToGoalSqr = vecToGoal.Length2();
	Real distToGoal = WWMath::Sqrt(distToGoalSqr);
	Real curVelMagSqr = curVel.Length2();
	Real curVelMag = WWMath::Sqrt(curVelMagSqr);
	Real maxAccelSqr = sqr(maxAccel);

	Real denom = curVelMagSqr - maxAccelSqr;
	if (!isNearlyZero(denom))
	{
		// solve the (greatly simplified) quadratic...
		Real t = (distToGoal * (curVelMag + maxAccel)) / denom;
		Real t2 = (distToGoal * (curVelMag - maxAccel)) / denom;
		if (t >= 0 || t2 >= 0)
		{
			// choose the smallest positive t.
			if (t < 0 || (t2 >= 0 && t2 < t))
				t = t2;

			// plug it in.
			if (!isNearlyZero(t))
			{
				goalDir.X = (vecToGoal.X / t) - curVel.X;
				goalDir.Y = (vecToGoal.Y / t) - curVel.Y;
				goalDir.Z = (vecToGoal.Z / t) - curVel.Z;
				goalDir.Normalize();
				foundSolution = true;
			}
		}
	}
	if (!foundSolution)
	{
		// Doh... no (useful) solution. revert to dumb.
		goalDir = vecToGoal;
		goalDir.Normalize();
	}

}

///////////////////////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS ///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
LocomotorTemplate::LocomotorTemplate()
{
	// these values mean "make the same as undamaged if not explicitly specified"
	m_maxSpeedDamaged = -1.0f;
	m_maxTurnRateDamaged = -1.0f;
	m_accelerationDamaged = -1.0f;
	m_liftDamaged = -1.0f;

	m_surfaces = 0;
	m_maxSpeed = 0.0f;
	m_maxTurnRate = 0.0f;
	m_acceleration = 0.0f;
	m_lift = 0.0f;
	m_braking = BIGNUM;
	m_minSpeed = 0.0f;
	m_minTurnSpeed = BIGNUM;
#if USE_RETAIL_PHYSICS_FORWARD_SPEED_AVERAGE()
	m_speedScale = DiagonalCompensation2D;
#endif
	m_behaviorZ = Z_NO_Z_MOTIVE_FORCE;
	m_appearance = LOCO_OTHER;
	m_movePriority = LOCO_MOVES_MIDDLE;
	m_preferredHeight = 0;
	m_preferredHeightDamping = 1.0f;
	m_circlingRadius = 0;

	m_maxThrustAngle = 0;
	m_speedLimitZ = 999999.0f;
	m_extra2DFriction = 0.0f;

	m_accelPitchLimit = 0;
	m_decelPitchLimit = 0;
	m_bounceKick = 0;

//	m_pitchStiffness = 0;
//	m_rollStiffness = 0;
//	m_pitchDamping = 0;
//	m_rollDamping = 0;
// it's highly unlikely you want zero for the defaults for stiffness and damping... (srj)
// for stiffness: stiffness of the "springs" in the suspension 0 = no stiffness, 1 = totally stiff (huh huh, he said "stiff")
// for damping: 0=perfect spring, bounces forever.  1=glued to terrain.
	m_pitchStiffness = 0.1f;
	m_rollStiffness = 0.1f;
	m_pitchDamping = 0.9f;
	m_rollDamping = 0.9f;
	m_forwardVelCoef = 0;
	m_pitchByZVelCoef = 0;
	m_thrustRoll = 0.0f;
	m_wobbleRate = 0.0f;
	m_minWobble = 0.0f;
	m_maxWobble = 0.0f;
	m_lateralVelCoef = 0;
	m_forwardAccelCoef = 0;
	m_lateralAccelCoef = 0;
	m_uniformAxialDamping = 1.0f;
	m_turnPivotOffset = 0;
	m_apply2DFrictionWhenAirborne = false;
	m_downhillOnly = false;
	m_allowMotiveForceWhileAirborne = false;
	m_locomotorWorksWhenDead = false;
	m_airborneTargetingHeight = INT_MAX;
	m_stickToGround = false;
	m_canMoveBackward = false;
	m_hasSuspension = false;
	m_wheelTurnAngle = 0;
	m_maximumWheelExtension = 0;
	m_maximumWheelCompression = 0;
	m_closeEnoughDist = 1.0f;
	m_isCloseEnoughDist3D = FALSE;
	m_ultraAccurateSlideIntoPlaceFactor = 0.0f;

	m_wanderWidthFactor = 0.0f;
	m_wanderLengthFactor = 1.0f;
	m_wanderAboutPointRadius = 0.0f;

	m_rudderCorrectionDegree    = 0.0f;
	m_rudderCorrectionRate      = 0.0f;
	m_elevatorCorrectionDegree  = 0.0f;
	m_elevatorCorrectionRate    = 0.0f;

}

//-------------------------------------------------------------------------------------------------
LocomotorTemplate::~LocomotorTemplate()
{

}

//-------------------------------------------------------------------------------------------------
void LocomotorTemplate::validate()
{
	// this is ok; parachutes need it!
	//DEBUG_ASSERTCRASH(m_lift == 0.0f || m_lift > fabs(TheGlobalData->m_gravity), ("Lift is too low to counteract gravity!"));
	//DEBUG_ASSERTCRASH(m_liftDamaged == 0.0f || m_liftDamaged > fabs(TheGlobalData->m_gravity), ("LiftDamaged is too low to counteract gravity!"));
	//DEBUG_ASSERTCRASH(m_preferredHeight == 0.0f || (m_behaviorZ == Z_SURFACE_RELATIVE_HEIGHT || m_behaviorZ == Z_ABSOLUTE_HEIGHT || m_appearance == LOCO_THRUST),
	//	("You must use Z_SURFACE_RELATIVE_HEIGHT or Z_ABSOLUTE_HEIGHT (or THRUST) to use preferredHeight"));

	// for 'damaged' stuff that was omitted, set 'em to be the same as 'undamaged'...
	if (m_maxSpeedDamaged < 0.0f)
		m_maxSpeedDamaged = m_maxSpeed;

	if (m_maxTurnRateDamaged < 0.0f)
		m_maxTurnRateDamaged = m_maxTurnRate;

	if (m_accelerationDamaged < 0.0f)
		m_accelerationDamaged = m_acceleration;

	if (m_liftDamaged < 0.0f)
		m_liftDamaged = m_lift;

	if (m_appearance == LOCO_WINGS)
	{
		if (m_minSpeed <= 0.0f)
		{
			DEBUG_CRASH(("WINGS should always have positive minSpeeds (otherwise, they hover)"));
			m_minSpeed = 0.01f;
		}
		if (m_minTurnSpeed <= 0.0f)
		{
			DEBUG_CRASH(("WINGS should always have positive minTurnSpeed"));
			m_minTurnSpeed = 0.01f;
		}
	}

	if (m_appearance == LOCO_THRUST)
	{
		if (m_behaviorZ != Z_NO_Z_MOTIVE_FORCE ||
				m_lift != 0.0f ||
				m_liftDamaged != 0.0f)
		{
			DEBUG_CRASH(("THRUST locos may not use ZAxisBehavior or lift!"));
			throw INI_INVALID_DATA;
		}
		if (m_maxSpeed <= 0.0f)
		{
			// if one of these was omitted, it defaults to zero... just quietly heal it here, rather than crashing
			DEBUG_LOG(("THRUST locos may not have zero m_maxSpeed; healing..."));
			m_maxSpeed = 0.01f;
		}
		if (m_maxSpeedDamaged <= 0.0f)
		{
			// if one of these was omitted, it defaults to zero... just quietly heal it here, rather than crashing
			DEBUG_LOG(("THRUST locos may not have zero m_maxSpeedDamaged; healing..."));
			m_maxSpeedDamaged = 0.01f;
		}
		if (m_minSpeed <= 0.0f)
		{
			// if one of these was omitted, it defaults to zero... just quietly heal it here, rather than crashing
			DEBUG_LOG(("THRUST locos may not have zero m_minSpeed; healing..."));
			m_minSpeed = 0.01f;
		}
	}

#if RTS_GENERALS
	// TheSuperHackers @info DecelerationPitchLimit was just added in Zero Hour and is therefore defaulted to
	// AccelerationPitchLimit when zero to preserve the original behavior of Generals.
	if (m_decelPitchLimit == 0.0f)
		m_decelPitchLimit = m_accelPitchLimit;
#endif

#if USE_RETAIL_PHYSICS_FORWARD_SPEED_AVERAGE()
	// TheSuperHackers @info THRUST is not scaled, see DiagonalCompensation2D.
	m_speedScale = (m_appearance == LOCO_THRUST) ? 1.0f : DiagonalCompensation2D;
#endif
}

//-------------------------------------------------------------------------------------------------
Real LocomotorTemplate::getActualMaxSpeed() const
{
	return scaleSpeed(m_maxSpeed);
}

//-------------------------------------------------------------------------------------------------
Real LocomotorTemplate::getActualMaxSpeedDamaged() const
{
	return scaleSpeed(m_maxSpeedDamaged);
}

//-------------------------------------------------------------------------------------------------
Real LocomotorTemplate::getActualMinSpeed() const
{
	return scaleSpeed(m_minSpeed);
}

//-------------------------------------------------------------------------------------------------
Real LocomotorTemplate::getActualMinTurnSpeed() const
{
	return scaleSpeed(m_minTurnSpeed);
}

//-------------------------------------------------------------------------------------------------
Real LocomotorTemplate::scaleSpeed(Real speed) const
{
#if USE_RETAIL_PHYSICS_FORWARD_SPEED_AVERAGE()
	if (!PhysicsBehavior::useLegacyForwardSpeed())
		return speed * m_speedScale;
#endif
	return speed;
}

//-------------------------------------------------------------------------------------------------
static void parseFrictionPerSec( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	Real fricPerSec = INI::scanReal(ini->getNextToken());
	Real fricPerFrame = fricPerSec * SECONDS_PER_LOGICFRAME_REAL;
	*(Real *)store = fricPerFrame;
}

//-------------------------------------------------------------------------------------------------
const FieldParse* LocomotorTemplate::getFieldParse() const
{
	static const FieldParse TheFieldParse[] =
	{
		{ "Surfaces", INI::parseBitString32, TheLocomotorSurfaceTypeNames, offsetof(LocomotorTemplate, m_surfaces) },
		{ "Speed", INI::parseVelocityReal, nullptr, offsetof(LocomotorTemplate, m_maxSpeed) },
		{ "SpeedDamaged", INI::parseVelocityReal, nullptr, offsetof( LocomotorTemplate, m_maxSpeedDamaged ) },
		{ "TurnRate", INI::parseAngularVelocityReal, nullptr, offsetof(LocomotorTemplate, m_maxTurnRate) },
		{ "TurnRateDamaged", INI::parseAngularVelocityReal, nullptr, offsetof( LocomotorTemplate, m_maxTurnRateDamaged ) },
		{ "Acceleration", INI::parseAccelerationReal, nullptr, offsetof(LocomotorTemplate, m_acceleration) },
		{ "AccelerationDamaged", INI::parseAccelerationReal, nullptr, offsetof( LocomotorTemplate, m_accelerationDamaged ) },
		{ "Lift", INI::parseAccelerationReal, nullptr, offsetof(LocomotorTemplate, m_lift) },
		{ "LiftDamaged", INI::parseAccelerationReal, nullptr, offsetof( LocomotorTemplate, m_liftDamaged ) },
		{ "Braking", INI::parseAccelerationReal, nullptr, offsetof(LocomotorTemplate, m_braking) },
		{ "MinSpeed", INI::parseVelocityReal, nullptr, offsetof(LocomotorTemplate, m_minSpeed) },
		{ "MinTurnSpeed", INI::parseVelocityReal, nullptr, offsetof(LocomotorTemplate, m_minTurnSpeed) },
		{ "PreferredHeight", INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_preferredHeight) },
		{ "PreferredHeightDamping", INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_preferredHeightDamping) },
		{ "CirclingRadius", INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_circlingRadius) },
		{ "Extra2DFriction", parseFrictionPerSec, nullptr, offsetof(LocomotorTemplate, m_extra2DFriction) },
		{ "SpeedLimitZ", INI::parseVelocityReal, nullptr, offsetof(LocomotorTemplate, m_speedLimitZ) },
		{ "MaxThrustAngle", INI::parseAngleReal, nullptr, offsetof(LocomotorTemplate, m_maxThrustAngle) },		// yes, angle, not angular-vel
		{ "ZAxisBehavior", INI::parseIndexList, TheLocomotorBehaviorZNames, offsetof(LocomotorTemplate, m_behaviorZ) },
		{ "Appearance", INI::parseIndexList, TheLocomotorAppearanceNames, offsetof(LocomotorTemplate, m_appearance) },		\
		{ "GroupMovementPriority", INI::parseIndexList, TheLocomotorPriorityNames, offsetof(LocomotorTemplate, m_movePriority) },		\

		{ "AccelerationPitchLimit", INI::parseAngleReal, nullptr, offsetof(LocomotorTemplate, m_accelPitchLimit) },
		{ "DecelerationPitchLimit", INI::parseAngleReal, nullptr, offsetof(LocomotorTemplate, m_decelPitchLimit) }, // Added in Zero Hour
		{ "BounceAmount", INI::parseAngularVelocityReal, nullptr, offsetof(LocomotorTemplate, m_bounceKick) },
		{ "PitchStiffness", INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_pitchStiffness) },
		{ "RollStiffness", INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_rollStiffness) },
		{ "PitchDamping", INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_pitchDamping) },
		{ "RollDamping", INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_rollDamping) },
		{ "ThrustRoll", INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_thrustRoll) },
		{ "ThrustWobbleRate",	INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_wobbleRate) },
		{ "ThrustMinWobble",	INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_minWobble) },
		{ "ThrustMaxWobble",	INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_maxWobble) },
		{ "PitchInDirectionOfZVelFactor", INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_pitchByZVelCoef) },
		{ "ForwardVelocityPitchFactor", INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_forwardVelCoef) },
		{ "LateralVelocityRollFactor", INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_lateralVelCoef) },
		{ "ForwardAccelerationPitchFactor", INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_forwardAccelCoef) },
		{ "LateralAccelerationRollFactor", INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_lateralAccelCoef) },
		{ "UniformAxialDamping", INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_uniformAxialDamping) },
		{ "TurnPivotOffset", INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_turnPivotOffset) },
		{ "Apply2DFrictionWhenAirborne", INI::parseBool, nullptr, offsetof(LocomotorTemplate, m_apply2DFrictionWhenAirborne) },
		{ "DownhillOnly", INI::parseBool, nullptr, offsetof(LocomotorTemplate, m_downhillOnly) },
		{ "AllowAirborneMotiveForce", INI::parseBool, nullptr, offsetof(LocomotorTemplate, m_allowMotiveForceWhileAirborne) },
		{ "LocomotorWorksWhenDead", INI::parseBool, nullptr, offsetof(LocomotorTemplate, m_locomotorWorksWhenDead) },
		{ "AirborneTargetingHeight", INI::parseInt, nullptr, offsetof( LocomotorTemplate, m_airborneTargetingHeight ) },
		{ "StickToGround",				INI::parseBool,			nullptr,	offsetof(LocomotorTemplate, m_stickToGround) },
		{ "CanMoveBackwards",				INI::parseBool,			nullptr,	offsetof(LocomotorTemplate, m_canMoveBackward) },
		{ "HasSuspension",				INI::parseBool,			nullptr,	offsetof(LocomotorTemplate, m_hasSuspension) },
		{ "FrontWheelTurnAngle", INI::parseAngleReal, nullptr, offsetof(LocomotorTemplate, m_wheelTurnAngle) },
		{ "MaximumWheelExtension", INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_maximumWheelExtension) },
		{ "MaximumWheelCompression", INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_maximumWheelCompression) },
		{ "CloseEnoughDist",				 INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_closeEnoughDist) },
		{ "CloseEnoughDist3D",			 INI::parseBool, nullptr, offsetof(LocomotorTemplate, m_isCloseEnoughDist3D) },
		{ "SlideIntoPlaceTime",		INI::parseDurationReal, nullptr, offsetof(LocomotorTemplate, m_ultraAccurateSlideIntoPlaceFactor) },

		{ "WanderWidthFactor", INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_wanderWidthFactor) },
		{ "WanderLengthFactor",				 INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_wanderLengthFactor) },
		{ "WanderAboutPointRadius",				 INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_wanderAboutPointRadius) },

		{ "RudderCorrectionDegree",		 INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_rudderCorrectionDegree) }, // Added in Zero Hour
		{ "RudderCorrectionRate",			 INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_rudderCorrectionRate) }, // Added in Zero Hour
		{ "ElevatorCorrectionDegree",	 INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_elevatorCorrectionDegree) }, // Added in Zero Hour
		{ "ElevatorCorrectionRate",		 INI::parseReal, nullptr, offsetof(LocomotorTemplate, m_elevatorCorrectionRate) }, // Added in Zero Hour
		{ nullptr, nullptr, nullptr, 0 }

	};
	return TheFieldParse;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
LocomotorStore::LocomotorStore()
{
}

//-------------------------------------------------------------------------------------------------
LocomotorStore::~LocomotorStore()
{
	// delete all the templates, then clear out the table.
	LocomotorTemplateMap::iterator it;
	for (it = m_locomotorTemplates.begin(); it != m_locomotorTemplates.end(); ++it) {
		deleteInstance(it->second);
	}

	m_locomotorTemplates.clear();
}

//-------------------------------------------------------------------------------------------------
LocomotorTemplate* LocomotorStore::findLocomotorTemplate(NameKeyType namekey)
{
	if (namekey == NAMEKEY_INVALID)
		return nullptr;

  LocomotorTemplateMap::iterator it = m_locomotorTemplates.find(namekey);
  if (it == m_locomotorTemplates.end())
		return nullptr;
	else
		return (*it).second;
}

//-------------------------------------------------------------------------------------------------
const LocomotorTemplate* LocomotorStore::findLocomotorTemplate(NameKeyType namekey) const
{
	if (namekey == NAMEKEY_INVALID)
		return nullptr;

  LocomotorTemplateMap::const_iterator it = m_locomotorTemplates.find(namekey);
  if (it == m_locomotorTemplates.end())
	{
		return nullptr;
	}
	else
	{
		return (*it).second;
	}
}

//-------------------------------------------------------------------------------------------------
void LocomotorStore::update()
{
}

//-------------------------------------------------------------------------------------------------
void LocomotorStore::reset()
{
	// cleanup overrides.
	LocomotorTemplateMap::iterator it;
	for (it = m_locomotorTemplates.begin(); it != m_locomotorTemplates.end(); ) {
		Overridable *locoTemp = it->second->deleteOverrides();
		if (!locoTemp)
		{
			m_locomotorTemplates.erase(it++);
		}
		else
		{
			++it;
		}
	}
}

//-------------------------------------------------------------------------------------------------
LocomotorTemplate *LocomotorStore::newOverride( LocomotorTemplate *locoTemplate )
{
	if (locoTemplate == nullptr)
		return nullptr;

	// allocate new template
	LocomotorTemplate *newTemplate = newInstance(LocomotorTemplate);

	// copy data from final override to 'newTemplate' as a set of initial default values
	*newTemplate = *locoTemplate;
	locoTemplate->setNextOverride(newTemplate);

	newTemplate->markAsOverride();

	// return the newly created override for us to set values with etc
	return newTemplate;

}

//-------------------------------------------------------------------------------------------------
/*static*/ void LocomotorStore::parseLocomotorTemplateDefinition(INI* ini)
{
	if (!TheLocomotorStore)
		throw INI_INVALID_DATA;

	Bool isOverride = false;
	// read the Locomotor name
	const char* token = ini->getNextToken();
	NameKeyType namekey = NAMEKEY(token);

	LocomotorTemplate *loco = TheLocomotorStore->findLocomotorTemplate(namekey);
	if (loco) {
		if (ini->getLoadType() == INI_LOAD_CREATE_OVERRIDES) {
			loco = TheLocomotorStore->newOverride((LocomotorTemplate*) loco->friend_getFinalOverride());
		}
		isOverride = true;
	} else {
		loco = newInstance(LocomotorTemplate);
		if (ini->getLoadType() == INI_LOAD_CREATE_OVERRIDES) {
			loco->markAsOverride();
		}
	}

	loco->friend_setName(token);
	ini->initFromINI(loco, loco->getFieldParse());
	loco->validate();

	// if this is an override, then we want the pointer on the existing named locomotor to point us
	// to the override, so don't add it to the map.
	if (!isOverride)
		TheLocomotorStore->m_locomotorTemplates[namekey] = loco;
}

//-------------------------------------------------------------------------------------------------
/*static*/ void INI::parseLocomotorTemplateDefinition( INI* ini )
{
	LocomotorStore::parseLocomotorTemplateDefinition(ini);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------------------
Locomotor::Locomotor(const LocomotorTemplate* tmpl)
{
	m_template = tmpl;
	m_brakingFactor = 1.0f;
	m_maxLift = BIGNUM;
	m_maxSpeed = BIGNUM;
	m_maxAccel = BIGNUM;
	m_maxBraking = BIGNUM;
	m_maxTurnRate = BIGNUM;
	m_flags = 0;
	m_closeEnoughDist = m_template->m_closeEnoughDist;
	setFlag(IS_CLOSE_ENOUGH_DIST_3D, m_template->m_isCloseEnoughDist3D);
#ifdef CIRCLE_FOR_LANDING
	m_circleThresh = 0.0f;
#endif
	m_preferredHeight = m_template->m_preferredHeight;
	m_preferredHeightDamping = m_template->m_preferredHeightDamping;

	m_angleOffset = GameLogicRandomValueReal(-PI/6, PI/6);
	m_offsetIncrement = (PI/40) * (GameLogicRandomValueReal(0.8f, 1.2f)/m_template->m_wanderLengthFactor);
	setFlag(OFFSET_INCREASING, GameLogicRandomValue(0,1));
	m_donutTimer = TheGameLogic->getFrame()+DONUT_TIME_DELAY_SECONDS*static_cast<float>(LOGICFRAMES_PER_SECOND);
}

//-------------------------------------------------------------------------------------------------
Locomotor::Locomotor(const Locomotor& that)
{
	m_angleOffset = 0.0f;
	m_maintainPos.zero();
	m_template = that.m_template;
	m_brakingFactor = that.m_brakingFactor;
	m_maxLift = that.m_maxLift;
	m_maxSpeed = that.m_maxSpeed;
	m_maxAccel = that.m_maxAccel;
	m_maxBraking = that.m_maxBraking;
	m_maxTurnRate = that.m_maxTurnRate;
	m_flags = that.m_flags;
	m_closeEnoughDist = that.m_closeEnoughDist;
#ifdef CIRCLE_FOR_LANDING
	m_circleThresh = that.m_circleThresh;
#endif
	m_preferredHeight = that.m_preferredHeight;
	m_preferredHeightDamping = that.m_preferredHeightDamping;
	m_angleOffset = that.m_angleOffset;
	m_offsetIncrement = that.m_offsetIncrement;
}

//-------------------------------------------------------------------------------------------------
Locomotor& Locomotor::operator=(const Locomotor& that)
{
	if (this != &that)
	{
		m_template = that.m_template;
		m_brakingFactor = that.m_brakingFactor;
		m_maxLift = that.m_maxLift;
		m_maxSpeed = that.m_maxSpeed;
		m_maxAccel = that.m_maxAccel;
		m_maxBraking = that.m_maxBraking;
		m_maxTurnRate = that.m_maxTurnRate;
		m_flags = that.m_flags;
		m_closeEnoughDist = that.m_closeEnoughDist;
#ifdef CIRCLE_FOR_LANDING
		m_circleThresh = that.m_circleThresh;
#endif
		m_preferredHeight = that.m_preferredHeight;
		m_preferredHeightDamping = that.m_preferredHeightDamping;
	}
	return *this;
}

//-------------------------------------------------------------------------------------------------
Locomotor::~Locomotor()
{
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void Locomotor::crc( Xfer *xfer )
{

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void Locomotor::xfer( Xfer *xfer )
{
	// version
	const XferVersion currentVersion = 2;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	if (version>=2) {
		xfer->xferUnsignedInt(&m_donutTimer);
	}

	xfer->xferCoord3D(&m_maintainPos);
	xfer->xferReal(&m_brakingFactor);
	xfer->xferReal(&m_maxLift);
	xfer->xferReal(&m_maxSpeed);
	xfer->xferReal(&m_maxAccel);
	xfer->xferReal(&m_maxBraking);
	xfer->xferReal(&m_maxTurnRate);
	xfer->xferReal(&m_closeEnoughDist);
#ifdef CIRCLE_FOR_LANDING
	DEBUG_CRASH(("not supported, must fix me"));
#endif
	xfer->xferUnsignedInt(&m_flags);
	xfer->xferReal(&m_preferredHeight);
	xfer->xferReal(&m_preferredHeightDamping);
	xfer->xferReal(&m_angleOffset);
	xfer->xferReal(&m_offsetIncrement);

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void Locomotor::loadPostProcess()
{

}

//-------------------------------------------------------------------------------------------------
void Locomotor::startMove()
{
	// Reset the donut timer.
	m_donutTimer = TheGameLogic->getFrame()+DONUT_TIME_DELAY_SECONDS*static_cast<float>(LOGICFRAMES_PER_SECOND);
}

//-------------------------------------------------------------------------------------------------
Real Locomotor::getMaxSpeedForCondition(BodyDamageType condition) const
{
	Real speed;

	if( IS_CONDITION_BETTER( condition, TheGlobalData->m_movementPenaltyDamageState ) )
		speed = m_template->getActualMaxSpeed();
	else
		speed = m_template->getActualMaxSpeedDamaged();

	Real maxSpeed = getMaxSpeedOverride();
	if (speed > maxSpeed)
		speed = maxSpeed;

	return speed;
}

//-------------------------------------------------------------------------------------------------
Real Locomotor::getMaxSpeedOverride() const
{
	return m_template->scaleSpeed(m_maxSpeed);
}

//-------------------------------------------------------------------------------------------------
Real Locomotor::getMinSpeed() const
{
	return m_template->getActualMinSpeed();
}

//-------------------------------------------------------------------------------------------------
Real Locomotor::getMinTurnSpeed() const
{
	return m_template->getActualMinTurnSpeed();
}

//-------------------------------------------------------------------------------------------------
Real Locomotor::getMaxTurnRate(BodyDamageType condition) const
{
	Real turn;

	if( IS_CONDITION_BETTER( condition, TheGlobalData->m_movementPenaltyDamageState ) )
		turn = m_template->m_maxTurnRate;
	else
		turn = m_template->m_maxTurnRateDamaged;

	if (turn > m_maxTurnRate)
		turn = m_maxTurnRate;

	const Real TURN_FACTOR = 2;
	if (getFlag(ULTRA_ACCURATE))
		turn *= TURN_FACTOR;	// monster turning ability

	return turn;
}

//-------------------------------------------------------------------------------------------------
Real Locomotor::getMaxAcceleration(BodyDamageType condition) const
{
	Real accel;

	if( IS_CONDITION_BETTER( condition, TheGlobalData->m_movementPenaltyDamageState ) )
		accel = m_template->m_acceleration;
	else
		accel = m_template->m_accelerationDamaged;

	if (accel > m_maxAccel)
		accel = m_maxAccel;

	return accel;
}

//-------------------------------------------------------------------------------------------------
Real Locomotor::getBraking() const
{
	Real braking = m_template->m_braking;

	if (braking > m_maxBraking)
		braking = m_maxBraking;

	return braking;
}

//-------------------------------------------------------------------------------------------------
Real Locomotor::getMaxLift(BodyDamageType condition) const
{
	Real lift;

	if( IS_CONDITION_BETTER( condition, TheGlobalData->m_movementPenaltyDamageState ) )
		lift = m_template->m_lift;
	else
		lift = m_template->m_liftDamaged;

	if (lift > m_maxLift)
		lift = m_maxLift;

	return lift;
}

//-------------------------------------------------------------------------------------------------
void Locomotor::locoUpdate_moveTowardsAngle(Object* obj, Real goalAngle)
{
	setFlag(MAINTAIN_POS_IS_VALID, false);

	if (obj == nullptr || m_template == nullptr)
		return;

	PhysicsBehavior *physics = obj->getPhysics();
	if (physics == nullptr)
	{
		DEBUG_CRASH(("you can only apply Locomotors to objects with Physics"));
		return;
	}

	// Skip moveTowardsAngle if physics say you're stunned
	if(physics->getIsStunned())
	{
		return;
	}

#ifdef DEBUG_OBJECT_ID_EXISTS
//	DEBUG_ASSERTLOG(obj->getID() != TheObjectIDToDebug, ("locoUpdate_moveTowardsAngle %f (%f deg), spd %f (%f)",goalAngle,goalAngle*180/PI,physics->getSpeed(),physics->getForwardSpeed2D()));
#endif

	Real minSpeed = getMinSpeed();
	if (minSpeed > 0)
	{
		// can't stay in one place; move in the desired direction at min speed.
		Coord3D desiredPos = *obj->getPosition();
		desiredPos.x += Cos(goalAngle) * minSpeed * 2;
		desiredPos.y += Sin(goalAngle) * minSpeed * 2;
		// pass a huge num for "dist to goal", so that we don't think we're nearing
		// our destination and thus slow down...
		const Real onPathDistToGoal = 99999.0f;
		Bool blocked = false;
		locoUpdate_moveTowardsPosition(obj, desiredPos, onPathDistToGoal, minSpeed, &blocked);

		// don't need to call handleBehaviorZ() here, since locoUpdate_moveTowardsPosition() will do so
		return;
	}
	else
	{
		DEBUG_ASSERTCRASH(m_template->m_appearance != LOCO_THRUST, ("THRUST should always have minspeeds!"));
		Coord3D desiredPos = *obj->getPosition();
		desiredPos.x += Cos(goalAngle) * 1000.0f;
		desiredPos.y += Sin(goalAngle) * 1000.0f;
		PhysicsTurningType rotating = rotateTowardsPosition(obj, desiredPos);
		physics->setTurning(rotating);
		handleBehaviorZ(obj, physics, *obj->getPosition());
	}

}

//-------------------------------------------------------------------------------------------------
PhysicsTurningType Locomotor::rotateTowardsPosition(Object* obj, const Coord3D& goalPos, Real *relAngle)
{
	BodyDamageType bdt = obj->getBodyModule()->getDamageState();
	Real turnRate = getMaxTurnRate(bdt);

	PhysicsTurningType rotating = rotateObjAroundLocoPivot(obj, goalPos, turnRate, relAngle);
	return rotating;
}

//-------------------------------------------------------------------------------------------------
void Locomotor::setPhysicsOptions(Object* obj)
{
	PhysicsBehavior *physics = obj->getPhysics();
	if (physics == nullptr)
	{
		DEBUG_CRASH(("you can only apply Locomotors to objects with Physics"));
		return;
	}

	// crank up the friction in ultra-accurate mode to increase movement precision.
	const Real EXTRA_FRIC = 0.5f;
	Real extraExtraFriction = getFlag(ULTRA_ACCURATE) ? EXTRA_FRIC : 0.0f;
	physics->setExtraFriction(m_template->m_extra2DFriction + extraExtraFriction);
	physics->setAllowAirborneFriction(getApply2DFrictionWhenAirborne());	// you'd think we wouldn't want friction in the air, but it's needed for realistic behavior.
	physics->setStickToGround(getStickToGround()); // walking guys aren't allowed to catch huge (or even small) air.
}

//-------------------------------------------------------------------------------------------------
void Locomotor::locoUpdate_moveTowardsPosition(Object* obj, const Coord3D& goalPos,
																							 Real onPathDistToGoal, Real desiredSpeed, Bool *blocked)
{
	setFlag(MAINTAIN_POS_IS_VALID, false);

	BodyDamageType bdt = obj->getBodyModule()->getDamageState();
	Real maxSpeed = getMaxSpeedForCondition(bdt);

	// sanity, we cannot use desired speed that is greater than our max speed we are capable of moving at
	if( desiredSpeed > maxSpeed )
		desiredSpeed = maxSpeed;

	Real distToStopAtMaxSpeed = (maxSpeed/getBraking()) * (maxSpeed)/2.0f;
	if (onPathDistToGoal>PATHFIND_CELL_SIZE_F && onPathDistToGoal > distToStopAtMaxSpeed)
	{
		setFlag(IS_BRAKING, false);
		m_brakingFactor = 1.0f;
	}
	PhysicsBehavior *physics = obj->getPhysics();
	if (physics == nullptr)
	{
		DEBUG_CRASH(("you can only apply Locomotors to objects with Physics"));
		return;
	}

	// Skip moveTowardsPosition if physics say you're stunned
	if(physics->getIsStunned())
	{
		return;
	}

#ifdef DEBUG_OBJECT_ID_EXISTS
//	DEBUG_ASSERTLOG(obj->getID() != TheObjectIDToDebug, ("locoUpdate_moveTowardsPosition %f %f %f (dtg %f, spd %f), speed %f (%f)",goalPos.x,goalPos.y,goalPos.z,onPathDistToGoal,desiredSpeed,physics->getSpeed(),physics->getForwardSpeed2D()));
#endif

	//
	// do not allow for invalid positions that the pathfinder cannot handle ... for airborne
	// objects we don't need the pathfinder so we'll ignore this
	//
	if( BitIsSet( m_template->m_surfaces, LOCOMOTORSURFACE_AIR ) == false &&
			!TheAI->pathfinder()->validMovementTerrain(obj->getLayer(), this, obj->getPosition()) &&
			!getFlag(ALLOW_INVALID_POSITION))
	{
		// Somehow, we have gotten to an invalid location.
		if (fixInvalidPosition(obj, physics))
		{
			// the we adjusted us toward a legal position, so just return.
			return;
		}
	}

	// If the actual distance is farther, then use the actual distance so we get there.
	Real dx = goalPos.x - obj->getPosition()->x;
	Real dy = goalPos.y - obj->getPosition()->y;
	Real dz = goalPos.z - obj->getPosition()->z;
	Real dist = WWMath::Sqrt(dx*dx+dy*dy);
	if (dist>onPathDistToGoal)
	{
		if (!obj->isKindOf(KINDOF_PROJECTILE) && dist>2*onPathDistToGoal)
		{
			setFlag(IS_BRAKING, true);
		}
		onPathDistToGoal = dist;
	}

	Coord3D nullAccel;

	Bool treatAsAirborne = false;
	Coord3D pos = *obj->getPosition();
	Real heightAboveSurface = pos.z - TheTerrainLogic->getLayerHeight(pos.x, pos.y, obj->getLayer());

	if( obj->getStatusBits().test( OBJECT_STATUS_DECK_HEIGHT_OFFSET ) )
	{
		heightAboveSurface -= obj->getCarrierDeckHeight();
	}

	if (heightAboveSurface > -(3*3)*TheGlobalData->m_gravity)
	{
		// If we get high enough to stay up for 3 frames, then we left the ground.
		treatAsAirborne = true;
	}
	// We apply a zero acceleration to all units, as the call to
	// applyMotiveForce flags an object as being "driven" by a locomotor, rather
	// than being pushed around by objects bumping it.
	nullAccel.x = nullAccel.y = nullAccel.z = 0;
	physics->applyMotiveForce(&nullAccel);

	if (*blocked)
	{
		if (desiredSpeed > physics->getVelocityMagnitude())
		{
			*blocked = false;
		}
		if (treatAsAirborne && BitIsSet( m_template->m_surfaces, LOCOMOTORSURFACE_AIR ) )
		{
			// Airborne flying objects don't collide for now.  jba.
			*blocked = false;
		}
	}

	if (*blocked)
	{
		physics->scrubVelocity2D(desiredSpeed); // stop if we are about to run into the blocking object.
		Real turnRate = getMaxTurnRate(obj->getBodyModule()->getDamageState());
		if (m_template->m_wanderWidthFactor == 0.0f)
		{
			*blocked = (TURN_NONE != rotateObjAroundLocoPivot(obj, goalPos, turnRate));
		}

		// it is very important to be sure to call this in all situations, even if not moving in 2d space.
		handleBehaviorZ(obj, physics, goalPos);
		return;
	}

	if (
// srj sez: I don't know why we didn't want HOVERs to allow to "brake".
// we actually really want them to, because it allows much more precise destination positioning.
//			m_template->m_appearance == LOCO_HOVER ||
				m_template->m_appearance == LOCO_WINGS)
	{
		setFlag(IS_BRAKING, false);
	}

	Bool wasBraking = obj->getStatusBits().test( OBJECT_STATUS_BRAKING );

	physics->setTurning(TURN_NONE);
	if (getAllowMotiveForceWhileAirborne() || !treatAsAirborne)
	{
		switch (m_template->m_appearance)
		{
			case LOCO_LEGS_TWO:
					moveTowardsPositionLegs(obj, physics, goalPos, onPathDistToGoal, desiredSpeed);
					break;
			case LOCO_CLIMBER:
					moveTowardsPositionClimb(obj, physics, goalPos, onPathDistToGoal, desiredSpeed);
					break;
			case LOCO_WHEELS_FOUR:
			case LOCO_MOTORCYCLE:
					moveTowardsPositionWheels(obj, physics, goalPos, onPathDistToGoal, desiredSpeed);
					break;
			case LOCO_TREADS:
					moveTowardsPositionTreads(obj, physics, goalPos, onPathDistToGoal, desiredSpeed);
					break;
			case LOCO_HOVER:
					moveTowardsPositionHover(obj, physics, goalPos, onPathDistToGoal, desiredSpeed);
					break;
			case LOCO_WINGS:
					moveTowardsPositionWings(obj, physics, goalPos, onPathDistToGoal, desiredSpeed);
					break;
			case LOCO_THRUST:
					moveTowardsPositionThrust(obj, physics, goalPos, onPathDistToGoal, desiredSpeed);
					break;
			case LOCO_OTHER:
			default:
					moveTowardsPositionOther(obj, physics, goalPos, onPathDistToGoal, desiredSpeed);
					break;
		}
	}

	handleBehaviorZ(obj, physics, goalPos);
	// Objects that are braking don't follow the normal physics, so they end up at their destination exactly.
	obj->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_BRAKING ), getFlag(IS_BRAKING) );

	if (wasBraking)
	{
	#define MIN_VEL (PATHFIND_CELL_SIZE_F/(static_cast<float>(LOGICFRAMES_PER_SECOND)))

		Coord3D pos = *obj->getPosition();
		if (obj->isKindOf(KINDOF_PROJECTILE))
		{
			// Projectiles never stop braking once they start.  jba.
			obj->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_BRAKING ) );
			// Projectiles cheat in 3 dimensions.
			dist = WWMath::Sqrt(dx*dx+dy*dy+dz*dz);
			Real vel = physics->getVelocityMagnitude();
			if (vel < MIN_VEL)
				vel = MIN_VEL;
			if (vel > dist)
				vel = dist;	// do not overcompensate!
			// Normalize.
			if (dist > 0.001f)
			{
				dist = 1.0f / dist;
				dx *= dist;
				dy *= dist;
				dz *= dist;
				pos.x += dx * vel;
				pos.y += dy * vel;
				pos.z += dz * vel;
			}
		}
		else
		{
			// not projectiles only cheat in x & y.
			// Normalize.
			if (dist > 0.001f)
			{
				Real vel = WWMath::Fabs(physics->getForwardSpeed2D());
				if (vel < MIN_VEL)
					vel = MIN_VEL;
				if (vel > dist)
					vel = dist;	// do not overcompensate!
				dist = 1.0f / dist;
				dx *= dist;
				dy *= dist;
				pos.x += dx * vel;
				pos.y += dy * vel;
			}
		}
		obj->setPosition(&pos);
	}

}

//-------------------------------------------------------------------------------------------------
void Locomotor::moveTowardsPositionTreads(Object* obj, PhysicsBehavior *physics, const Coord3D& goalPos, Real onPathDistToGoal, Real desiredSpeed)
{

	// sanity, we cannot use desired speed that is greater than our max speed we are capable of moving at
	BodyDamageType bdt = obj->getBodyModule()->getDamageState();
	Real maxSpeed = getMaxSpeedForCondition(bdt);
	if( desiredSpeed > maxSpeed )
		desiredSpeed = maxSpeed;

	Real maxAcceleration = getMaxAcceleration(bdt);

	// Locomotion for treaded vehicles, ie tanks.

	//
	// Orient toward goal position
	//
//	Real angle = obj->getOrientation();
//	Real relAngle = ThePartitionManager->getRelativeAngle2D( obj, &goalPos );
//	Real desiredAngle = angle + relAngle;
	Real relAngle ;
	PhysicsTurningType rotating = rotateTowardsPosition(obj, goalPos, &relAngle);
	physics->setTurning(rotating);

	//
	// Modulate speed according to turning. The more we have to turn, the slower we go
	//
	const Real QUAETERPI = PI / 4.0f;
	Real angleCoeff = (Real)WWMath::Fabs( relAngle ) / QUAETERPI;
	if (angleCoeff > 1.0f)
		angleCoeff = 1.0;


	Real dx = obj->getPosition()->x - goalPos.x;
	Real dy = obj->getPosition()->y - goalPos.y;


	Real goalSpeed = (1.0f - angleCoeff) * desiredSpeed;


//	if (speed < m_minTurnSpeed)
//		speed = m_minTurnSpeed;

	Real actualSpeed = physics->getForwardSpeed2D();
	Real slowDownTime = actualSpeed / getBraking();
	Real slowDownDist = (actualSpeed/1.50f) * slowDownTime;

	if (sqr(dx)+sqr(dy)<sqr(2*PATHFIND_CELL_SIZE_F) && angleCoeff > 0.05) {
		goalSpeed = actualSpeed*0.6f;
	}

 	if (onPathDistToGoal < slowDownDist && !getFlag(IS_BRAKING) && !getFlag(NO_SLOW_DOWN_AS_APPROACHING_DEST))
	{
		setFlag(IS_BRAKING, true);
		m_brakingFactor = 1.1f;
	}

	if (onPathDistToGoal>PATHFIND_CELL_SIZE_F && onPathDistToGoal > 2.0*slowDownDist)
	{
		setFlag(IS_BRAKING, false);
	}

	if (getFlag(IS_BRAKING))
	{
		m_brakingFactor = slowDownDist/onPathDistToGoal;
		m_brakingFactor *= m_brakingFactor;
		if (m_brakingFactor>MAX_BRAKING_FACTOR) {
			m_brakingFactor = MAX_BRAKING_FACTOR;
		}
		if (slowDownDist>onPathDistToGoal) {
			goalSpeed = actualSpeed-getBraking();
			if (goalSpeed<0.0f) goalSpeed= 0.0f;
		} else if (slowDownDist>onPathDistToGoal*0.75f) {
			goalSpeed = actualSpeed-getBraking()/2.0f;
			if (goalSpeed<0.0f) goalSpeed = 0.0f;
		} else {
			goalSpeed = actualSpeed;
		}
	}


	//DEBUG_LOG(("Actual speed %f, Braking factor %f, slowDownDist %f, Pathdist %f, goalSpeed %f",
	//	actualSpeed, m_brakingFactor, slowDownDist, onPathDistToGoal, goalSpeed));

	//
	// Maintain goal speed
	//
	Real speedDelta = goalSpeed - actualSpeed;
	if (speedDelta != 0.0f)
	{
		Real mass = physics->getMass();
		Real acceleration = (speedDelta > 0.0f) ? maxAcceleration : -m_brakingFactor*getBraking();
		Real accelForce = mass * acceleration;

		/*
			don't accelerate/brake more than necessary. do a quick calc to
			see how much force we really need to achieve our goal speed...
		*/
		Real maxForceNeeded = mass * speedDelta;
		if (WWMath::Fabs(accelForce) > WWMath::Fabs(maxForceNeeded))
			accelForce = maxForceNeeded;

		const Coord3D *dir = obj->getUnitDirectionVector2D();

		Coord3D force;
		force.x = accelForce * dir->x;
		force.y = accelForce * dir->y;
		force.z = 0.0f;

		// apply forces to object
		physics->applyMotiveForce( &force );
	}
}

//-------------------------------------------------------------------------------------------------
void Locomotor::moveTowardsPositionWheels(Object* obj, PhysicsBehavior *physics, const Coord3D& goalPos, Real onPathDistToGoal, Real desiredSpeed)
{
	BodyDamageType bdt = obj->getBodyModule()->getDamageState();
	Real maxSpeed = getMaxSpeedForCondition(bdt);
	Real maxTurnRate = getMaxTurnRate(bdt);
	Real maxAcceleration = getMaxAcceleration(bdt);

	// sanity, we cannot use desired speed that is greater than our max speed we are capable of moving at
	if( desiredSpeed > maxSpeed )
		desiredSpeed = maxSpeed;

	// Locomotion for wheeled vehicles, ie trucks.
	//
	// See if we are turning.  If so, use the min turn speed.
	//
	Real turnSpeed = getMinTurnSpeed();
	Real angle = obj->getOrientation();
//	Real relAngle = ThePartitionManager->getRelativeAngle2D( obj, &goalPos );
//	Real desiredAngle = angle + relAngle;
	Real desiredAngle = WWMath::Atan2(goalPos.y - obj->getPosition()->y, goalPos.x - obj->getPosition()->x);
	Real relAngle = stdAngleDiff(desiredAngle, angle);

	Bool moveBackwards = false;

	// Wheeled vehicles can only turn while moving, so make sure the turn speed is reasonable.
	if (turnSpeed < maxSpeed/4.0f)
	{
		turnSpeed = maxSpeed/4.0f;
	}


	Real actualSpeed = physics->getForwardSpeed2D();
	Bool do3pointTurn = false;
#if 1
	if (actualSpeed==0.0f) {
		setFlag(MOVING_BACKWARDS, false);
		if (m_template->m_canMoveBackward && WWMath::Fabs(relAngle) > PI/2) {
			setFlag(MOVING_BACKWARDS, true );
			setFlag(DOING_THREE_POINT_TURN, onPathDistToGoal>5*obj->getGeometryInfo().getMajorRadius());
		}

	}
	if (getFlag(MOVING_BACKWARDS)) {
		if (WWMath::Fabs(relAngle) < PI/2) {
			moveBackwards = false;
			setFlag(MOVING_BACKWARDS, false);
		} else {
			moveBackwards = true;
			setFlag(DOING_THREE_POINT_TURN, onPathDistToGoal>5*obj->getGeometryInfo().getMajorRadius());
			do3pointTurn = getFlag(DOING_THREE_POINT_TURN);
			if (!do3pointTurn) {
				desiredAngle = stdAngleDiff(desiredAngle, PI);
				relAngle = stdAngleDiff(desiredAngle, angle);
			}
		}
	}
#endif

	const Real SMALL_TURN = PI / 20.0f;
	if ((Real)WWMath::Fabs( relAngle ) > SMALL_TURN)
	{
		if (desiredSpeed>turnSpeed)
		{
			desiredSpeed = turnSpeed;
		}
	}

	Real goalSpeed = desiredSpeed;
	if (moveBackwards) {
		actualSpeed = -actualSpeed;
	}



	Real slowDownTime = actualSpeed / getBraking() + 1.0f;
	Real slowDownDist = (actualSpeed/1.5f) * slowDownTime + actualSpeed;
	Real effectiveSlowDownDist = slowDownDist;
	if (effectiveSlowDownDist < 1*PATHFIND_CELL_SIZE) {
		effectiveSlowDownDist = 1*PATHFIND_CELL_SIZE;
	}


	const Real FIFTEEN_DEGREES = PI / 12.0f;
	const Real PROJECT_FRAMES = LOGICFRAMES_PER_SECOND/2; // Project out 1/2 second.
	if (WWMath::Fabs( relAngle ) > FIFTEEN_DEGREES)
	{
		// If we're turning more than 10 degrees, check & see if we're moving into "impassable territory"
		Real distance = PROJECT_FRAMES * (goalSpeed+actualSpeed)/2.0f;
		Real targetAngle = obj->getOrientation();
		Real turnFactor = ((goalSpeed+actualSpeed)/2.0f)/turnSpeed;
		if (turnFactor > 1.0f)
			turnFactor = 1.0f;
		Real turnAmount = PROJECT_FRAMES*turnFactor*maxTurnRate/4.0f;
		if (relAngle < 0)
		{
			targetAngle -= turnAmount;
		}
		else
		{
			targetAngle += turnAmount;
		}
		Coord3D offset;
		offset.x = Cos(targetAngle)*distance;
		offset.y = Sin(targetAngle)*distance;
		offset.z = 0;

		const Coord3D* pos = obj->getPosition();

		Coord3D nextPos;
		nextPos.x = pos->x+offset.x;
		nextPos.y = pos->y+offset.y;
		nextPos.z = pos->z;

		pos = obj->getPosition();

		Coord3D halfPos;
		halfPos.x = pos->x+offset.x/2;
		halfPos.y = pos->y+offset.y/2;
		halfPos.z = pos->z;

		if (!TheAI->pathfinder()->validMovementTerrain(obj->getLayer(), this, &halfPos) ||
			!TheAI->pathfinder()->validMovementTerrain(obj->getLayer(), this, &nextPos))
		{
			PhysicsTurningType rotating = rotateTowardsPosition(obj, goalPos);
			physics->setTurning(rotating);

			// apply a zero force to object so that it acts "driven"
			Coord3D force;
			force.zero();
			physics->applyMotiveForce( &force );
			return;
		}

	}

	if (onPathDistToGoal < effectiveSlowDownDist && !getFlag(IS_BRAKING) && !getFlag(NO_SLOW_DOWN_AS_APPROACHING_DEST))
	{
		setFlag(IS_BRAKING, true);
		m_brakingFactor = 1.1f;
	}


	if (onPathDistToGoal>PATHFIND_CELL_SIZE_F && onPathDistToGoal > 2.0*slowDownDist)
	{
		setFlag(IS_BRAKING, false);
	}

 	if (onPathDistToGoal > DONUT_DISTANCE) {
		m_donutTimer = TheGameLogic->getFrame()+DONUT_TIME_DELAY_SECONDS*static_cast<float>(LOGICFRAMES_PER_SECOND);
	} else {
		if (m_donutTimer < TheGameLogic->getFrame()) {
			setFlag(IS_BRAKING, true);
		}
	}

	if (getFlag(IS_BRAKING))
	{
		m_brakingFactor = slowDownDist/onPathDistToGoal;
		m_brakingFactor *= m_brakingFactor;
		if (m_brakingFactor>MAX_BRAKING_FACTOR) {
			m_brakingFactor = MAX_BRAKING_FACTOR;
		}
		m_brakingFactor = 1.0f;
		if (slowDownDist>onPathDistToGoal) {
			goalSpeed = actualSpeed-getBraking();
			if (goalSpeed<0.0f) goalSpeed= 0.0f;
		} else if (slowDownDist>onPathDistToGoal*0.75f) {
			goalSpeed = actualSpeed-getBraking()/2.0f;
			if (goalSpeed<0.0f) goalSpeed = 0.0f;
		} else {
			goalSpeed = actualSpeed;
		}
	}


	//DEBUG_LOG(("Actual speed %f, Braking factor %f, slowDownDist %f, Pathdist %f, goalSpeed %f",
	//	actualSpeed, m_brakingFactor, slowDownDist, onPathDistToGoal, goalSpeed));


	// Wheeled can only turn while moving.
	Real turnFactor = actualSpeed/turnSpeed;
	if (turnFactor<0) {
		turnFactor = -turnFactor; // in case we're sliding backwards in a 3 pt turn.
	}
	if (turnFactor > 1.0f)
		turnFactor = 1.0f;
	Real turnAmount = turnFactor*maxTurnRate;

	PhysicsTurningType rotating;
	if (moveBackwards && !do3pointTurn) {
		Coord3D backwardPos = *obj->getPosition();
		backwardPos.x += -(goalPos.x - obj->getPosition()->x);
		backwardPos.y += -(goalPos.y - obj->getPosition()->y);
		rotating = rotateObjAroundLocoPivot(obj, backwardPos, turnAmount);
	} else {
		rotating = rotateObjAroundLocoPivot(obj, goalPos, turnAmount);
	}

	physics->setTurning(rotating);

	//
	// Maintain goal speed
	//
	Real speedDelta = goalSpeed - actualSpeed;
	if (moveBackwards) {
		speedDelta = -goalSpeed+actualSpeed;
	}
	if (speedDelta != 0.0f)
	{
		Real mass = physics->getMass();
		Real acceleration;
		if (moveBackwards) {
			acceleration = (speedDelta < 0.0f) ? -maxAcceleration : m_brakingFactor*getBraking();
		}	else {
			acceleration = (speedDelta > 0.0f) ? maxAcceleration : -m_brakingFactor*getBraking();
		}
		Real accelForce = mass * acceleration;

		/*
			don't accelerate/brake more than necessary. do a quick calc to
			see how much force we really need to achieve our goal speed...
		*/
		Real maxForceNeeded = mass * speedDelta;
		if (WWMath::Fabs(accelForce) > WWMath::Fabs(maxForceNeeded))
			accelForce = maxForceNeeded;

		//DEBUG_LOG(("Braking %d, actualSpeed %f, goalSpeed %f, delta %f, accel %f", getFlag(IS_BRAKING),
			//actualSpeed, goalSpeed, speedDelta, accelForce));

		const Coord3D *dir = obj->getUnitDirectionVector2D();

		Coord3D force;
		force.x = accelForce * dir->x;
		force.y = accelForce * dir->y;
		force.z = 0.0f;

		// apply forces to object
		physics->applyMotiveForce( &force );
	}

}
//-------------------------------------------------------------------------------------------------
Bool Locomotor::fixInvalidPosition(Object* obj, PhysicsBehavior *physics)
{
	if (obj->isKindOf(KINDOF_DOZER)) {
		// don't fix him.
		return false;
	}
#define no_IGNORE_INVALID
#ifdef IGNORE_INVALID
	// Right now we ignore invalid positions, so when units clip the edge of a building or cliff
	// they don't get stuck.  jba. 12SEPT02
	return false;
#else
	Int dx = 0;
	Int dy = 0;
	Int i, j;
	for (j=-1; j<2; j++) {
		for (i=-1; i<2; i++) {
			Coord3D thePos = *obj->getPosition();
			thePos.x += i*PATHFIND_CELL_SIZE_F;
			thePos.y += j*PATHFIND_CELL_SIZE_F;
			if (!TheAI->pathfinder()->validMovementTerrain(obj->getLayer(), this, &thePos)) {
				if (i<0) dx += 1;
				if (i>0) dx -= 1;
				if (j<0) dy += 1;
				if (j>0) dy -= 1;
			}
		}
	}
	if (dx || dy) {

		Coord3D correction;
		correction.x = dx*physics->getMass()/5;
		correction.y = dy*physics->getMass()/5;
		correction.z = 0;

		Coord3D correctionNormalized = correction;
		correctionNormalized.normalize();

		Coord3D velocity;
		// Kill current velocity	in the direction of the correction.
		velocity = *physics->getVelocity();
		Real dot = (velocity.x*correctionNormalized.x) + (velocity.y*correctionNormalized.y);
		if (dot>.25f) {
			// It was already leaving.
			return false;
		}


		// Kill current accel
		//physics->clearAcceleration();

		if (dot<0) {
			dot = WWMath::Sqrt(-dot);
			correctionNormalized.x *= dot*physics->getMass();
			correctionNormalized.y *= dot*physics->getMass();
			physics->applyMotiveForce(&correctionNormalized);
		}

		// apply correction.
		physics->applyMotiveForce(&correction);
		return true;
	}
	return false;
#endif
}

//-------------------------------------------------------------------------------------------------
Real Locomotor::calcMinTurnRadius(BodyDamageType condition, Real* timeToTravelThatDist) const
{
	Real minSpeed = getMinSpeed();								// in dist/frame
	Real maxTurnRate = getMaxTurnRate(condition);	// in rads/frame

	/*
		our minimum circumference will be like so:

		Real minTurnCircum = maxSpeed * (2*PI / maxTurnRate);

		so therefore our minimum turn radius is:

		Real minTurnRadius = minTurnCircum / 2*PI;

		so we just eliminate the middleman:
	*/
	// if we can't turn, return a huge-but-finite radius rather than NAN...
	Real minTurnRadius = (maxTurnRate > 0.0f) ? minSpeed / maxTurnRate : BIGNUM;

	if (timeToTravelThatDist)
		*timeToTravelThatDist = (minSpeed > 0.0f) ? (minTurnRadius / minSpeed) : 0.0f;

	return minTurnRadius;
}


//-------------------------------------------------------------------------------------------------
void Locomotor::moveTowardsPositionLegs(Object* obj, PhysicsBehavior *physics, const Coord3D& goalPos, Real onPathDistToGoal, Real desiredSpeed)
{
	if (getIsDownhillOnly() && obj->getPosition()->z < goalPos.z)
	{
		return;
	}

	Real maxAcceleration = getMaxAcceleration( obj->getBodyModule()->getDamageState() );

	// sanity, we cannot use desired speed that is greater than our max speed we are capable of moving at
	Real maxSpeed = getMaxSpeedForCondition( obj->getBodyModule()->getDamageState() );
	if( desiredSpeed > maxSpeed )
		desiredSpeed = maxSpeed;

	// Locomotion for infantry.
	//
	// Orient toward goal position
	//
	Real actualSpeed = physics->getForwardSpeed2D();
	Real angle = obj->getOrientation();
//	Real relAngle = ThePartitionManager->getRelativeAngle2D( obj, &goalPos );
//	Real desiredAngle = angle + relAngle;
	Real desiredAngle = WWMath::Atan2(goalPos.y - obj->getPosition()->y, goalPos.x - obj->getPosition()->x);

	if (m_template->m_wanderWidthFactor != 0.0f) {
		Real angleLimit = PI/8 * m_template->m_wanderWidthFactor;
		// This is the wander offline code - it forces the desired angle away from the goal, so we wander back & forth.  jba.
		if (getFlag(OFFSET_INCREASING)) {
			m_angleOffset += m_offsetIncrement*actualSpeed;
			if (m_angleOffset > angleLimit) {
				setFlag(OFFSET_INCREASING, false);
			}
		} else {
			m_angleOffset -= m_offsetIncrement*actualSpeed;
			if (m_angleOffset<-angleLimit) {
				setFlag(OFFSET_INCREASING, true);
			}
		}
		desiredAngle = normalizeAngle(desiredAngle+m_angleOffset);
	}

	Real relAngle = stdAngleDiff(desiredAngle, angle);
	locoUpdate_moveTowardsAngle(obj, desiredAngle);

	//
	// Modulate speed according to turning. The more we have to turn, the slower we go
	//
	const Real QUARTERPI = PI / 4.0f;
	Real angleCoeff = (Real)WWMath::Fabs( relAngle ) / (QUARTERPI);
	if (angleCoeff > 1.0f)
		angleCoeff = 1.0;

	Real goalSpeed = (1.0f - angleCoeff) * desiredSpeed;

	//Real slowDownDist = (actualSpeed - m_template->m_minSpeed) / getBraking();
	Real slowDownDist = calcSlowDownDist(actualSpeed, getMinSpeed(), getBraking());
	if (onPathDistToGoal < slowDownDist && !getFlag(NO_SLOW_DOWN_AS_APPROACHING_DEST))
	{
		goalSpeed = getMinSpeed();
	}



	//
	// Maintain goal speed
	//
	Real speedDelta = goalSpeed - actualSpeed;
	if (speedDelta != 0.0f)
	{
		Real mass = physics->getMass();
		Real acceleration = (speedDelta > 0.0f) ? maxAcceleration : -getBraking();
		Real accelForce = mass * acceleration;

		/*
			don't accelerate/brake more than necessary. do a quick calc to
			see how much force we really need to achieve our goal speed...
		*/
		Real maxForceNeeded = mass * speedDelta;
		if (WWMath::Fabs(accelForce) > WWMath::Fabs(maxForceNeeded))
			accelForce = maxForceNeeded;

		const Coord3D *dir = obj->getUnitDirectionVector2D();

		Coord3D force;
		force.x = accelForce * dir->x;
		force.y = accelForce * dir->y;
		force.z = 0.0f;



		// apply forces to object
		physics->applyMotiveForce( &force );
	}
}

//-------------------------------------------------------------------------------------------------
void Locomotor::moveTowardsPositionClimb(Object* obj, PhysicsBehavior *physics, const Coord3D& goalPos, Real onPathDistToGoal, Real desiredSpeed)
{
	Real maxAcceleration = getMaxAcceleration( obj->getBodyModule()->getDamageState() );

	// sanity, we cannot use desired speed that is greater than our max speed we are capable of moving at
	Real maxSpeed = getMaxSpeedForCondition( obj->getBodyModule()->getDamageState() );
	if( desiredSpeed > maxSpeed )
		desiredSpeed = maxSpeed;

	// Locomotion for climbing infantry.


	Bool moveBackwards = false;

	Real dx, dy, dz;

	Coord3D pos = *obj->getPosition();

	dx = pos.x - goalPos.x;
	dy = pos.y - goalPos.y;
	dz = pos.z - goalPos.z;
	if (dz*dz > sqr(PATHFIND_CELL_SIZE_F)) {
		setFlag(CLIMBING, true);
	}
	if (WWMath::Fabs(dz)<1) {
		setFlag(CLIMBING, false);
	}


	//setFlag(CLIMBING, true);

	if (getFlag(CLIMBING)) {
		Coord3D delta = goalPos;
		delta.x -= pos.x;
		delta.y -= pos.y;
		delta.z = 0;
		delta.normalize();
		delta.x += pos.x;
		delta.y += pos.y;
		delta.z = TheTerrainLogic->getGroundHeight(delta.x, delta.y);
		if (delta.z < pos.z-0.1) {
			moveBackwards = true;
		}

		Real groundSlope = WWMath::Fabs(delta.z - pos.z);
		if (groundSlope<1.0f) groundSlope = 1.0f;

		if (groundSlope>1.0f) {
			desiredSpeed /= groundSlope*4;
		}
	}
	setFlag(MOVING_BACKWARDS, moveBackwards);

	//
	// Orient toward goal position
	//
	Real angle = obj->getOrientation();
//	Real relAngle = ThePartitionManager->getRelativeAngle2D( obj, &goalPos );
//	Real desiredAngle = angle + relAngle;
	Real desiredAngle = WWMath::Atan2(goalPos.y - obj->getPosition()->y, goalPos.x - obj->getPosition()->x);
	Real relAngle = stdAngleDiff(desiredAngle, angle);

	if (moveBackwards) {
		desiredAngle = stdAngleDiff(desiredAngle, PI);
		relAngle = stdAngleDiff(desiredAngle, angle);
	}

	locoUpdate_moveTowardsAngle(obj, desiredAngle);

	//
	// Modulate speed according to turning. The more we have to turn, the slower we go
	//
	const Real QUARTERPI = PI / 4.0f;
	Real angleCoeff = (Real)WWMath::Fabs( relAngle ) / (QUARTERPI);
	if (angleCoeff > 1.0f)
		angleCoeff = 1.0;

	Real goalSpeed = (1.0f - angleCoeff) * desiredSpeed;

	Real actualSpeed = physics->getForwardSpeed2D();

	if (moveBackwards) {
		actualSpeed = -actualSpeed;
	}

	//Real slowDownDist = (actualSpeed - m_template->m_minSpeed) / getBraking();
	Real slowDownDist = calcSlowDownDist(actualSpeed, getMinSpeed(), getBraking());
	if (onPathDistToGoal < slowDownDist && !getFlag(NO_SLOW_DOWN_AS_APPROACHING_DEST))
	{
		goalSpeed = getMinSpeed();
	}

	//
	// Maintain goal speed
	//
	Real speedDelta = goalSpeed - actualSpeed;
	if (moveBackwards) {
		speedDelta = -goalSpeed+actualSpeed;
	}
	if (speedDelta != 0.0f)
	{
		Real mass = physics->getMass();
		Real acceleration;
		if (moveBackwards) {
			acceleration = (speedDelta < 0.0f) ? -maxAcceleration : getBraking();
		}	else {
			acceleration = (speedDelta > 0.0f) ? maxAcceleration : -getBraking();
		}
		Real accelForce = mass * acceleration;

		/*
			don't accelerate/brake more than necessary. do a quick calc to
			see how much force we really need to achieve our goal speed...
		*/
		Real maxForceNeeded = mass * speedDelta;
		if (WWMath::Fabs(accelForce) > WWMath::Fabs(maxForceNeeded))
			accelForce = maxForceNeeded;

		const Coord3D *dir = obj->getUnitDirectionVector2D();

		Coord3D force;
		force.x = accelForce * dir->x;
		force.y = accelForce * dir->y;
		force.z = 0.0f;

		// apply forces to object
		physics->applyMotiveForce( &force );
	}
}

//-------------------------------------------------------------------------------------------------
void Locomotor::moveTowardsPositionWings(Object* obj, PhysicsBehavior *physics, const Coord3D& goalPos, Real onPathDistToGoal, Real desiredSpeed)
{
#ifdef CIRCLE_FOR_LANDING
	if (m_circleThresh > 0.0f)
	{
		// if we are going a mostly-vertical maneuver, circle in order to
		// gain/lose altitude, then resume course...
		const Coord3D* pos = obj->getPosition();
		Real dx = goalPos.x - pos->x;
		Real dy = goalPos.y - pos->y;
		Real dz = goalPos.z - pos->z;
		if (WWMath::Fabs(dz) > m_circleThresh)
		{
			// aim for the spot on the opposite side of the circle.

			// find the direction towards our goal pos
			Real angleTowardPos =
					(isNearlyZero(dx) && isNearlyZero(dy)) ?
					obj->getOrientation() :
					WWMath::Atan2(dy, dx);

			Real aimDir = (PI - PI/8);
			angleTowardPos += aimDir;

			BodyDamageType bdt = obj->getBodyModule()->getDamageState();
			Real turnRadius = calcMinTurnRadius(bdt, nullptr) * 4;

			// project a spot "radius" dist away from it, in that dir
			Coord3D desiredPos = goalPos;
			desiredPos.x += Cos(angleTowardPos) * turnRadius;
			desiredPos.y += Sin(angleTowardPos) * turnRadius;
			moveTowardsPositionOther(obj, physics, desiredPos, 0, desiredSpeed);
			return;
		}
	}
#endif

	// handle the 2D component.
	moveTowardsPositionOther(obj, physics, goalPos, onPathDistToGoal, desiredSpeed);
}

//-------------------------------------------------------------------------------------------------
void Locomotor::moveTowardsPositionHover(Object* obj, PhysicsBehavior *physics, const Coord3D& goalPos, Real onPathDistToGoal, Real desiredSpeed)
{
	// handle the 2D component.
	moveTowardsPositionOther(obj, physics, goalPos, onPathDistToGoal, desiredSpeed);

	// Only hover locomotors care about their OverWater special effects.  (OverWater also affects speed, so this is not a client thing)
	Coord3D newPosition = *obj->getPosition();
	if( TheTerrainLogic->isUnderwater( newPosition.x, newPosition.y ) )
	{
		if( ! getFlag( OVER_WATER ) )
		{
			// Change my model condition because I used to not be over water, but now I am
			setFlag( OVER_WATER, TRUE );
			obj->setModelConditionState( MODELCONDITION_OVER_WATER );
		}
	}
	else
	{
		if( getFlag( OVER_WATER ) )
		{
			// Here, I was, but now I'm not
			setFlag( OVER_WATER, FALSE );
			obj->clearModelConditionState( MODELCONDITION_OVER_WATER );
		}
	}
}

//-------------------------------------------------------------------------------------------------
void Locomotor::moveTowardsPositionThrust(Object* obj, PhysicsBehavior *physics, const Coord3D& goalPos, Real onPathDistToGoal, Real desiredSpeed)
{
	BodyDamageType bdt = obj->getBodyModule()->getDamageState();

	Real maxForwardSpeed = getMaxSpeedForCondition(bdt);
	desiredSpeed = clamp(getMinSpeed(), desiredSpeed, maxForwardSpeed);
	Real actualForwardSpeed = physics->getForwardSpeed3D();

	if (getBraking() > 0)
	{
		//Real slowDownDist = (actualForwardSpeed - m_template->m_minSpeed) / getBraking();
		Real slowDownDist = calcSlowDownDist(actualForwardSpeed, getMinSpeed(), getBraking());
		if (onPathDistToGoal < slowDownDist && !getFlag(NO_SLOW_DOWN_AS_APPROACHING_DEST))
			desiredSpeed = getMinSpeed();
	}

	Coord3D localGoalPos = goalPos;
#ifdef USE_ZDIR_DAMPING
	Real zDirDamping = 0.0f;
#endif

	//out of the handleBehaviorZ() function
	Coord3D pos = *obj->getPosition();
	if( m_preferredHeight != 0.0f && !getFlag(PRECISE_Z_POS) )
	{
		// If we have a preferred flight height, and we haven't been told explicitly to ignore it...
		Real surfaceHt =  getSurfaceHtAtPt(pos.x, pos.y);
		localGoalPos.z = m_preferredHeight + surfaceHt;
//		localGoalPos.z = goalPos.z;
		Real delta = localGoalPos.z - pos.z;
		delta *= getPreferredHeightDamping();
		localGoalPos.z = pos.z + delta;

#ifdef USE_ZDIR_DAMPING
		// closer we get to the preferred height, less we adjust z-thrust,
		// so we tend to "level out" at that height. we don't use this till
		// below, but go ahead and calc it now...
		Real MAX_VERTICAL_DAMP_RANGE = m_preferredHeight * 0.5;
		delta = WWMath::Fabs(delta);
		if (delta > MAX_VERTICAL_DAMP_RANGE)
			delta = MAX_VERTICAL_DAMP_RANGE;
		zDirDamping = 1.0f - (delta / MAX_VERTICAL_DAMP_RANGE);
#endif
	}

	Vector3 forwardDir = obj->getTransformMatrix()->Get_X_Vector();

	// Maintain goal speed
	Real forwardSpeedDelta = desiredSpeed - actualForwardSpeed;
	Real maxAccel = (forwardSpeedDelta > 0.0f || getBraking() == 0) ? getMaxAcceleration(bdt) : -getBraking();
	Real maxTurnRate = getMaxTurnRate(bdt);

	// what direction do we need to thrust in, in order to reach the goalpos?
	Vector3 desiredThrustDir;
	calcDirectionToApplyThrust(obj, physics, localGoalPos, maxAccel, desiredThrustDir);

	// we might not be able to thrust in that dir, so thrust as closely as we can
	Real maxThrustAngle =	(maxTurnRate > 0) ? (m_template->m_maxThrustAngle) : 0;
	Vector3 thrustDir;
	Real thrustAngle = tryToRotateVector3D(maxThrustAngle, forwardDir, desiredThrustDir, thrustDir);

	// note that we are trying to orient in the direction of our vel, not the dir of our thrust.
	if (!isNearlyZero(physics->getVelocityMagnitude()))
	{
		const Coord3D* veltmp = physics->getVelocity();
		Vector3 vel(veltmp->x, veltmp->y, veltmp->z);
		Bool adjust = true;
		if( obj->getStatusBits().test( OBJECT_STATUS_BRAKING ) )
		{
			// align to target, cause that's where we're going anyway.

			vel.Set(goalPos.x - pos.x, goalPos.y-pos.y, goalPos.z-pos.z);
			if (isNearlyZero(sqr(vel.X)+sqr(vel.Y)+sqr(vel.Z))) {
				// we are at target.
				adjust = false;
			}
			maxTurnRate = 3*maxTurnRate;
		}
#ifdef USE_ZDIR_DAMPING
		if (zDirDamping != 0.0f)
		{
			Vector3 vel2D(veltmp->x, veltmp->y, 0);
			// no need to normalize -- this call does that internally
			tryToRotateVector3D(-zDirDamping, vel, vel2D, vel);
		}
#endif
		if (adjust) {
			/*Real orient =*/ tryToOrientInThisDirection3D(obj, maxTurnRate, vel);
		}
	}

	if (forwardSpeedDelta != 0.0f || thrustAngle != 0.0f)
	{
		if (maxForwardSpeed <= 0.0f)
		{
			maxForwardSpeed = 0.01f; // In some cases, this is 0, hack for now.  jba.
		}
		Real damping = clamp(0.0f, maxAccel / maxForwardSpeed, 1.0f);
		Vector3 curVel(physics->getVelocity()->x, physics->getVelocity()->y, physics->getVelocity()->z);

		Vector3 accelVec = thrustDir * maxAccel - curVel * damping;
		//DEBUG_LOG(("accel %f (max %f) vel %f (max %f) damping %f",accelVec.Length(),maxAccel,curVel.Length(),maxForwardSpeed,damping));

		Real mass = physics->getMass();

		Coord3D force;
		force.x = mass * accelVec.X;
		force.y = mass * accelVec.Y;
		force.z = mass * accelVec.Z;

		// apply forces to object
		physics->applyMotiveForce( &force );
	}
}

//-------------------------------------------------------------------------------------------------
Real Locomotor::getSurfaceHtAtPt(Real x, Real y)
{
	Real ht = 0;

	Real z,waterZ;
	if (TheTerrainLogic->isUnderwater(x, y, &waterZ, &z)) {
		ht += waterZ;
	} else {
		ht += z;
	}

	return ht;
}

//-------------------------------------------------------------------------------------------------
Real Locomotor::calcLiftToUseAtPt(Object* obj, PhysicsBehavior *physics, Real curZ, Real surfaceAtPt, Real preferredHeight)
{
	/*
		take the classic equation:

			x = x0 + v*t + 0.5*a*t^2

		and solve for acceleration.
	*/
	BodyDamageType bdt = obj->getBodyModule()->getDamageState();
	Real maxGrossLift = getMaxLift(bdt);
	Real maxNetLift = maxGrossLift + TheGlobalData->m_gravity;	// note that gravity is always negative.
	if (maxNetLift < 0)
		maxNetLift = 0;
	Real curVelZ = physics->getVelocity()->z;
	// going down, braking is limited by net lift; going up, braking is limited by gravity
	Real maxAccel;
	if (getFlag(ULTRA_ACCURATE))
		maxAccel = (curVelZ < 0) ? 2*maxNetLift : -2*maxNetLift;
	else
		maxAccel = (curVelZ < 0) ? maxNetLift : TheGlobalData->m_gravity;
	// see how far we need to slow to dead stop, given max braking
	Real desiredAccel;
	const Real TINY_ACCEL = 0.001f;
	if (WWMath::Fabs(maxAccel) > TINY_ACCEL)
	{
		Real deltaZ = preferredHeight - curZ;
		// calc how far it will take for us to go from cur speed to zero speed, at max accel.
		// Real brakeDist = calcSlowDownDist(curVelZ, 0, maxAccel);
		// in theory, the above is the correct calculation, but in practice,
		// doesn't work in some situations (eg, opening of USA01 map). Why, I dunno.
		// But for now I have gone back to the old, looks-incorrect-to-me-but-works calc. (srj)
		Real brakeDist = (sqr(curVelZ) / WWMath::Fabs(maxAccel));
		if (WWMath::Fabs(brakeDist) > WWMath::Fabs(deltaZ))
		{
			// if the dist-to-accel (or dist-to-brake) is further than the dist-to-go,
			// use the max accel.
			desiredAccel = maxAccel;
		}
		else if (WWMath::Fabs(curVelZ) > m_template->m_speedLimitZ)
		{
			// or, if we're going too fast, limit it here.
			desiredAccel = m_template->m_speedLimitZ - curVelZ;
		}
		else
		{
			// ok, figure out the correct accel to use to get us there at zero.
			//
			// dz = v t + 0.5 a t^2
			//	thus
			// a = 2(dz - v t)/t^2
			//	and
			// t = (-v +- sqrt(v*v + 2*a*dz))/a
			//
			// but if we assume t=1, then
			//	a=2(dz-v)
			// then, plug it back in and see if t is really 1...
			desiredAccel = 2.0f * (deltaZ - curVelZ);
		}
	}
	else
	{
		desiredAccel = 0.0f;
	}
	Real liftToUse = desiredAccel - TheGlobalData->m_gravity;
	if (getFlag(ULTRA_ACCURATE))
	{
		// in ultra-accurate mode, we allow cheating.
		const Real UP_FACTOR = 3.0f;
		if (liftToUse > UP_FACTOR*maxGrossLift)
			liftToUse = UP_FACTOR*maxGrossLift;
		// srj sez: we used to clip lift to zero here (not allowing neg lift).
		// however, I now think that allowing neg lift in ultra-accurate mode is
		// a good and desirable thing; in particular, it enables jets to complete
		// "short" landings more accurately (previously they sometimes would "float"
		// down, which sucked.) if you need to bump this back to zero, check it carefully...
		else if (liftToUse < -maxGrossLift)
			liftToUse = -maxGrossLift;
	}
	else
	{
		if (liftToUse > maxGrossLift)
			liftToUse = maxGrossLift;
		else if (liftToUse < 0.0f)
			liftToUse = 0.0f;
	}

	return liftToUse;
}

//-------------------------------------------------------------------------------------------------
PhysicsTurningType Locomotor::rotateObjAroundLocoPivot(Object* obj, const Coord3D& goalPos,
																											 Real maxTurnRate, Real *relAngle)
{
	Real angle = obj->getOrientation();
	Real offset = getTurnPivotOffset();

	PhysicsTurningType turn = TURN_NONE;

	if (getFlag(IS_BRAKING)) offset = 0.0f; // When braking we do exact movement towards goal, instead of physics.
	//Rotating about pivot moves the object, and can make us miss our goal, so it is disabled.  jba.
	if (offset != 0.0f)
	{
		Real radius = obj->getGeometryInfo().getBoundingCircleRadius();
		Real turnPointOffset = offset * radius;

		Coord3D turnPos = *obj->getPosition();
		const Coord3D* dir = obj->getUnitDirectionVector2D();
		turnPos.x += dir->x * turnPointOffset;
		turnPos.y += dir->y * turnPointOffset;
		Real dx =goalPos.x - turnPos.x;
		Real dy = goalPos.y - turnPos.y;
		// If we are very close to the goal, we twitch due to rounding error.  So just return. jba.
		if (WWMath::Fabs(dx)<0.1f && WWMath::Fabs(dy)<0.1f) return TURN_NONE;
		Real desiredAngle = WWMath::Atan2(dy, dx);
		Real amount = stdAngleDiff(desiredAngle, angle);
		if (relAngle) *relAngle = amount;
		if (amount>maxTurnRate) {
			amount = maxTurnRate;
			turn = TURN_POSITIVE;
		} else if (amount < -maxTurnRate) {
			amount = -maxTurnRate;
			turn = TURN_NEGATIVE;
		} else {
			turn = TURN_NONE;
		}

#if 0
		Coord3D desiredPos = *obj->getPosition();	// well, desired Dir, anyway
		desiredPos.x += Cos(angle + amount) * radius;
		desiredPos.y += Sin(angle + amount) * radius;


		// so, the thing is, we want to rotate ourselves so that our *center* is rotated
		// by the given amount, but the rotation must be around turnPos. so do a little
		// back-calculation.
		Real angleDesiredForTurnPos = WWMath::Atan2(desiredPos.y - turnPos.y, desiredPos.x - turnPos.x);
		amount = angleDesiredForTurnPos - angle;
#endif
		/// @todo srj -- there's probably a more efficient & more direct way to do this. find it.
		Matrix3D mtx;
		Matrix3D tmp(1);
		tmp.Translate(turnPos.x, turnPos.y, 0);
		tmp.In_Place_Pre_Rotate_Z(amount);
		tmp.Translate(-turnPos.x, -turnPos.y, 0);

		mtx.mul(tmp, *obj->getTransformMatrix());

		obj->setTransformMatrix(&mtx);
	}
	else
	{
		Real desiredAngle = WWMath::Atan2(goalPos.y - obj->getPosition()->y, goalPos.x - obj->getPosition()->x);
		Real amount = stdAngleDiff(desiredAngle, angle);
		if (relAngle) *relAngle = amount;
		if (amount>maxTurnRate) {
			amount = maxTurnRate;
			turn = TURN_POSITIVE;
		} else if (amount < -maxTurnRate) {
			amount = -maxTurnRate;
			turn = TURN_NEGATIVE;
		} else {
			turn = TURN_NONE;
		}
		obj->setOrientation( normalizeAngle(angle + amount) );
	}
	return turn;
}

//-------------------------------------------------------------------------------------------------
/*
	return true if we can maintain the position without being called every frame (eg, we are
	resting on the ground), false if not (eg, we are hovering or circling)
*/
Bool Locomotor::handleBehaviorZ(Object* obj, PhysicsBehavior *physics, const Coord3D& goalPos)
{
	Bool requiresConstantCalling = TRUE;

	// keep the agent aligned on the terrain
	switch(m_template->m_behaviorZ)
	{
		case Z_NO_Z_MOTIVE_FORCE:
			// nothing to do.
			requiresConstantCalling = FALSE;
			break;

		case Z_SEA_LEVEL:
			requiresConstantCalling = TRUE;
			if( !obj->isDisabledByType( DISABLED_HELD ) )
			{
				Coord3D pos = *obj->getPosition();
				Real waterZ;
				if (TheTerrainLogic->isUnderwater(pos.x, pos.y, &waterZ)) {
					pos.z = waterZ;
				} else {
					pos.z = TheTerrainLogic->getLayerHeight(pos.x, pos.y, obj->getLayer());
				}
				obj->setPosition(&pos);
			}
			break;

		case Z_FIXED_SURFACE_RELATIVE_HEIGHT:
		case Z_FIXED_ABSOLUTE_HEIGHT:
			requiresConstantCalling = TRUE;
			{
				Coord3D pos = *obj->getPosition();
				Bool surfaceRel = (m_template->m_behaviorZ == Z_FIXED_SURFACE_RELATIVE_HEIGHT);
				Real surfaceHt = surfaceRel ? getSurfaceHtAtPt(pos.x, pos.y) : 0.0f;
				pos.z = m_preferredHeight + (surfaceRel ? surfaceHt : 0);
				obj->setPosition(&pos);
			}
			break;

		case Z_RELATIVE_TO_GROUND_AND_BUILDINGS:
			requiresConstantCalling = TRUE;
			{
				// srj sez: use getGroundOrStructureHeight(), because someday it will cache building heights...
				Coord3D pos = *obj->getPosition();
				Real surfaceHt = ThePartitionManager->getGroundOrStructureHeight(pos.x, pos.y);

					pos.z = m_preferredHeight + surfaceHt;

					obj->setPosition(&pos);

			}
			break;
		case Z_SMOOTH_RELATIVE_TO_HIGHEST_LAYER:
			requiresConstantCalling = TRUE;
			{
				if (m_preferredHeight != 0.0f || getFlag(PRECISE_Z_POS))
				{
					Coord3D pos = *obj->getPosition();

					// srj sez: if we aren't on the ground, never find the ground layer
					PathfindLayerEnum layerAtDest = obj->getLayer();
					if (layerAtDest == LAYER_GROUND)
						layerAtDest = TheTerrainLogic->getHighestLayerForDestination( &pos );

					Real surfaceHt;
					Coord3D normal;
					const Bool clip = false;	// return the height, even if off the edge of the bridge proper.
					surfaceHt = TheTerrainLogic->getLayerHeight( pos.x, pos.y, layerAtDest, &normal, clip );

					Real preferredHeight = m_preferredHeight + surfaceHt;
					if (getFlag(PRECISE_Z_POS))
						preferredHeight = goalPos.z;

					Real delta = preferredHeight - pos.z;
					delta *= getPreferredHeightDamping();
					preferredHeight = pos.z + delta;

					Real liftToUse = calcLiftToUseAtPt(obj, physics, pos.z, surfaceHt, preferredHeight);

					//DEBUG_LOG(("HandleBZ %d LiftToUse %f",TheGameLogic->getFrame(),liftToUse));
					if (liftToUse != 0.0f)
					{
						Coord3D force;
						force.x = 0.0f;
						force.y = 0.0f;
						force.z = liftToUse * physics->getMass();
						physics->applyMotiveForce(&force);
					}
				}
			}
			break;

		case Z_SURFACE_RELATIVE_HEIGHT:
		case Z_ABSOLUTE_HEIGHT:
			requiresConstantCalling = TRUE;
			{
				if (m_preferredHeight != 0.0f || getFlag(PRECISE_Z_POS))
				{
					Coord3D pos = *obj->getPosition();

					Bool surfaceRel = (m_template->m_behaviorZ == Z_SURFACE_RELATIVE_HEIGHT);
					Real surfaceHt = surfaceRel ? getSurfaceHtAtPt(pos.x, pos.y) : 0.0f;
					Real preferredHeight = m_preferredHeight + (surfaceRel ? surfaceHt : 0);
					if (getFlag(PRECISE_Z_POS))
						preferredHeight = goalPos.z;

					Real delta = preferredHeight - pos.z;
					delta *= getPreferredHeightDamping();
					preferredHeight = pos.z + delta;

					Real liftToUse = calcLiftToUseAtPt(obj, physics, pos.z, surfaceHt, preferredHeight);

					//DEBUG_LOG(("HandleBZ %d LiftToUse %f",TheGameLogic->getFrame(),liftToUse));
					if (liftToUse != 0.0f)
					{
						Coord3D force;
						force.x = 0.0f;
						force.y = 0.0f;
						force.z = liftToUse * physics->getMass();
						physics->applyMotiveForce(&force);
					}
				}
			}
			break;
	}

	return requiresConstantCalling;
}

//-------------------------------------------------------------------------------------------------
void Locomotor::moveTowardsPositionOther(Object* obj, PhysicsBehavior *physics, const Coord3D& goalPos, Real onPathDistToGoal, Real desiredSpeed)
{
	BodyDamageType bdt = obj->getBodyModule()->getDamageState();
	Real maxAcceleration = getMaxAcceleration(bdt);

	// sanity, we cannot use desired speed that is greater than our max speed we are capable of moving at
	Real maxSpeed = getMaxSpeedForCondition(bdt);
	if( desiredSpeed > maxSpeed )
		desiredSpeed = maxSpeed;

	Real goalSpeed = desiredSpeed;
	Real actualSpeed = physics->getForwardSpeed2D();

	// Locomotion for other things, ie don't know what it is jba :)
	//
	// Orient toward goal position
	// exception: if very close (ie, we could get there in 2 frames or less),\
	// and ULTRA_ACCURATE, just slide into place
	//
	const Coord3D* pos =  obj->getPosition();
	Coord3D dirToApplyForce = *obj->getUnitDirectionVector2D();

//DEBUG_ASSERTLOG(!getFlag(ULTRA_ACCURATE),("thresh %f %f (%f %f)",
//fabs(goalPos.y - pos->y),fabs(goalPos.x - pos->x),
//fabs(goalPos.y - pos->y)/goalSpeed,fabs(goalPos.x - pos->x)/goalSpeed));
	if (getFlag(ULTRA_ACCURATE) &&
				WWMath::Fabs(goalPos.y - pos->y) <= goalSpeed * m_template->m_ultraAccurateSlideIntoPlaceFactor &&
				WWMath::Fabs(goalPos.x - pos->x) <= goalSpeed * m_template->m_ultraAccurateSlideIntoPlaceFactor)
	{
		// don't turn, just slide in the right direction
		physics->setTurning(TURN_NONE);
		dirToApplyForce.x = goalPos.x - pos->x;
		dirToApplyForce.y = goalPos.y - pos->y;
		dirToApplyForce.z = 0.0f;
		dirToApplyForce.normalize();
	}
	else
	{
		PhysicsTurningType rotating = rotateTowardsPosition(obj, goalPos);
		physics->setTurning(rotating);
	}

	if (!getFlag(NO_SLOW_DOWN_AS_APPROACHING_DEST))
	{
		Real slowDownDist = calcSlowDownDist(actualSpeed, getMinSpeed(), getBraking());
		if (onPathDistToGoal < slowDownDist)
		{
			goalSpeed = getMinSpeed();
		}
	}

	//
	// Maintain goal speed
	//
	Real speedDelta = goalSpeed - actualSpeed;
	if (speedDelta != 0.0f)
	{
		Real mass = physics->getMass();
		Real acceleration = (speedDelta > 0.0f) ? maxAcceleration : -getBraking();
		Real accelForce = mass * acceleration;

		/*
			don't accelerate/brake more than necessary. do a quick calc to
			see how much force we really need to achieve our goal speed...
		*/
		Real maxForceNeeded = mass * speedDelta;
		if (WWMath::Fabs(accelForce) > WWMath::Fabs(maxForceNeeded))
			accelForce = maxForceNeeded;

		Coord3D force;
		force.x = accelForce * dirToApplyForce.x;
		force.y = accelForce * dirToApplyForce.y;
		force.z = 0.0f;

		// apply forces to object
		physics->applyMotiveForce( &force );
	}

}


//-------------------------------------------------------------------------------------------------
/*
	return true if we can maintain the position without being called every frame (eg, we are
	resting on the ground), false if not (eg, we are hovering or circling)
*/
Bool Locomotor::locoUpdate_maintainCurrentPosition(Object* obj)
{
	if (!getFlag(MAINTAIN_POS_IS_VALID))
	{
		m_maintainPos = *obj->getPosition();
		setFlag(MAINTAIN_POS_IS_VALID, true);
	}

	m_donutTimer = TheGameLogic->getFrame()+DONUT_TIME_DELAY_SECONDS*static_cast<float>(LOGICFRAMES_PER_SECOND);
	setFlag(IS_BRAKING, false);
	PhysicsBehavior *physics = obj->getPhysics();
	if (physics == nullptr)
	{
		DEBUG_CRASH(("you can only apply Locomotors to objects with Physics"));
		return TRUE;
	}

#ifdef DEBUG_OBJECT_ID_EXISTS
//	DEBUG_ASSERTLOG(obj->getID() != TheObjectIDToDebug, ("locoUpdate_maintainCurrentPosition %f %f %f, speed %f (%f)",m_maintainPos.x,m_maintainPos.y,m_maintainPos.z,physics->getSpeed(),physics->getForwardSpeed2D()));
#endif

	Bool requiresConstantCalling = TRUE;	// assume the worst.
	switch (m_template->m_appearance)
	{
		case LOCO_THRUST:
			maintainCurrentPositionThrust(obj, physics);
			requiresConstantCalling = TRUE;
			break;
		case LOCO_LEGS_TWO:
			maintainCurrentPositionLegs(obj, physics);
			requiresConstantCalling = FALSE;
			break;
		case LOCO_CLIMBER:
			maintainCurrentPositionLegs(obj, physics);
			requiresConstantCalling = FALSE;
			break;
		case LOCO_WHEELS_FOUR:
		case LOCO_MOTORCYCLE:
			maintainCurrentPositionWheels(obj, physics);
			requiresConstantCalling = FALSE;
			break;
		case LOCO_TREADS:
			maintainCurrentPositionTreads(obj, physics);
			requiresConstantCalling = FALSE;
			break;
		case LOCO_HOVER:
			maintainCurrentPositionHover(obj, physics);
			requiresConstantCalling = TRUE;
			break;
		case LOCO_WINGS:
			maintainCurrentPositionWings(obj, physics);
			requiresConstantCalling = TRUE;
			break;
		case LOCO_OTHER:
		default:
			maintainCurrentPositionOther(obj, physics);
			requiresConstantCalling = TRUE;
			break;
	}

	// but we do need to do this even if not moving, for hovering/Thrusting things.
	if (handleBehaviorZ(obj, physics, m_maintainPos))
		requiresConstantCalling = TRUE;

	return requiresConstantCalling;
}

//-------------------------------------------------------------------------------------------------
void Locomotor::maintainCurrentPositionThrust(Object* obj, PhysicsBehavior *physics)
{
	DEBUG_ASSERTCRASH(getFlag(MAINTAIN_POS_IS_VALID), ("invalid maintain pos"));
	/// @todo srj -- should these also use the "circling radius" stuff, like wings?
	moveTowardsPositionThrust(obj, physics, m_maintainPos, 0, getMinSpeed());
}

//-------------------------------------------------------------------------------------------------
void Locomotor::maintainCurrentPositionWings(Object* obj, PhysicsBehavior *physics)
{
	DEBUG_ASSERTCRASH(getFlag(MAINTAIN_POS_IS_VALID), ("invalid maintain pos"));
	physics->setTurning(TURN_NONE);
	if (physics->isMotive() && obj->isAboveTerrain())	// no need to stop something that isn't moving (or is just sitting on the ground)
	{

			// aim for the spot on the opposite side of the circle.
		BodyDamageType bdt = obj->getBodyModule()->getDamageState();
		Real turnRadius = m_template->m_circlingRadius;
		if (turnRadius == 0.0f)
			turnRadius = calcMinTurnRadius(bdt, nullptr);

		// find the direction towards our "maintain pos"
		const Coord3D* pos = obj->getPosition();
		Real dx = m_maintainPos.x - pos->x;
		Real dy = m_maintainPos.y - pos->y;
		Real angleTowardMaintainPos =
				(isNearlyZero(dx) && isNearlyZero(dy)) ?
				obj->getOrientation() :
				WWMath::Atan2(dy, dx);

		Real aimDir = (PI - PI/8);
		if (turnRadius < 0)
		{
			turnRadius = -turnRadius;
			aimDir = -aimDir;
		}
		angleTowardMaintainPos += aimDir;

		// project a spot "radius" dist away from it, in that dir
		Coord3D desiredPos = m_maintainPos;
		desiredPos.x += Cos(angleTowardMaintainPos) * turnRadius;
		desiredPos.y += Sin(angleTowardMaintainPos) * turnRadius;
		moveTowardsPositionWings(obj, physics, desiredPos, 0, getMinSpeed());
	}
}

//-------------------------------------------------------------------------------------------------
void Locomotor::maintainCurrentPositionHover(Object* obj, PhysicsBehavior *physics)
{
	physics->setTurning(TURN_NONE);
	if (physics->isMotive())	// no need to stop something that isn't moving.
	{
		DEBUG_ASSERTCRASH(m_template->m_minSpeed == 0.0f, ("HOVER should always have zero minSpeeds (otherwise, they WING)"));

		BodyDamageType bdt = obj->getBodyModule()->getDamageState();
		Real maxAcceleration = getMaxAcceleration(bdt);
		Real actualSpeed = physics->getForwardSpeed2D();
		//
		// Stop
		//
		Real minSpeed = max( 1.0E-10f, getMinSpeed() );
		Real speedDelta = minSpeed - actualSpeed;
		if (WWMath::Fabs(speedDelta) > minSpeed)
		{
			Real mass = physics->getMass();
			Real acceleration = (speedDelta > 0.0f) ? maxAcceleration : -getBraking();
			Real accelForce = mass * acceleration;

			/*
				don't accelerate/brake more than necessary. do a quick calc to
				see how much force we really need to achieve our goal speed...
			*/
			Real maxForceNeeded = mass * speedDelta;
			if (WWMath::Fabs(accelForce) > WWMath::Fabs(maxForceNeeded))
				accelForce = maxForceNeeded;

			const Coord3D *dir = obj->getUnitDirectionVector2D();

			Coord3D force;
			force.x = accelForce * dir->x;
			force.y = accelForce * dir->y;
			force.z = 0.0f;

			// apply forces to object
			physics->applyMotiveForce( &force );
		}
	}

}

//-------------------------------------------------------------------------------------------------
void Locomotor::maintainCurrentPositionOther(Object* obj, PhysicsBehavior *physics)
{

	physics->setTurning(TURN_NONE);
	if (physics->isMotive())	// no need to stop something that isn't moving.
	{
		physics->scrubVelocity2D(0); // stop.
	}

}


//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------------------
LocomotorSet::LocomotorSet()
{
	m_locomotors.clear();
	m_validLocomotorSurfaces = 0;
	m_downhillOnly = FALSE;

}

//-------------------------------------------------------------------------------------------------
LocomotorSet::LocomotorSet(const LocomotorSet& that)
{
	DEBUG_CRASH(("unimplemented"));
}

//-------------------------------------------------------------------------------------------------
LocomotorSet& LocomotorSet::operator=(const LocomotorSet& that)
{
	if (this != &that)
	{
		DEBUG_CRASH(("unimplemented"));
	}
	return *this;
}

//-------------------------------------------------------------------------------------------------
LocomotorSet::~LocomotorSet()
{
	clear();
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void LocomotorSet::crc( Xfer *xfer )
{

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void LocomotorSet::xfer( Xfer *xfer )
{
	// version
	const XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// count of vector
	UnsignedShort count = m_locomotors.size();
	xfer->xferUnsignedShort( &count );

	// data
	if (xfer->getXferMode() == XFER_SAVE)
	{
		for (LocomotorVector::iterator it = m_locomotors.begin(); it != m_locomotors.end(); ++it)
		{
			Locomotor* loco = *it;
			AsciiString name = loco->getTemplateName();
			xfer->xferAsciiString(&name);
			xfer->xferSnapshot(loco);
		}
	}
	else if (xfer->getXferMode() == XFER_LOAD)
	{
		// vector should be empty at this point
		if (m_locomotors.empty() == FALSE)
		{
			DEBUG_CRASH(( "LocomotorSet::xfer - vector is not empty, but should be" ));
			throw XFER_LIST_NOT_EMPTY;
		}

		for (UnsignedShort i = 0; i < count; ++i)
		{
			AsciiString name;
			xfer->xferAsciiString(&name);

			const LocomotorTemplate* lt = TheLocomotorStore->findLocomotorTemplate(NAMEKEY(name));
			if (lt == nullptr)
			{
				DEBUG_CRASH(( "LocomotorSet::xfer - template %s not found", name.str() ));
				throw XFER_UNKNOWN_STRING;
			}

			Locomotor* loco = TheLocomotorStore->newLocomotor(lt);
			xfer->xferSnapshot(loco);
			m_locomotors.push_back(loco);
		}
	}

	xfer->xferInt(&m_validLocomotorSurfaces);
	xfer->xferBool(&m_downhillOnly);

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void LocomotorSet::loadPostProcess()
{

}

//-------------------------------------------------------------------------------------------------
void LocomotorSet::xferSelfAndCurLocoPtr(Xfer *xfer, Locomotor** loco)
{
	xfer->xferSnapshot(this);

	if (xfer->getXferMode() == XFER_SAVE)
	{
		AsciiString name;
		if (*loco)
			name = (*loco)->getTemplateName();
		xfer->xferAsciiString(&name);
	}
	else if (xfer->getXferMode() == XFER_LOAD)
	{
		AsciiString name;
		xfer->xferAsciiString(&name);

		if (name.isEmpty())
		{
			*loco = nullptr;
		}
		else
		{
			for (size_t i = 0; i < m_locomotors.size(); ++i)
			{
				if (m_locomotors[i]->getTemplateName() == name)
				{
					*loco = m_locomotors[i];
					return;
				}
			}

			DEBUG_CRASH(( "LocomotorSet::xfer - template %s not found", name.str() ));
			throw XFER_UNKNOWN_STRING;
		}
	}
}

//-------------------------------------------------------------------------------------------------
void LocomotorSet::clear()
{
	for (size_t i = 0; i < m_locomotors.size(); ++i)
	{
		deleteInstance(m_locomotors[i]);
	}
	m_locomotors.clear();
	m_validLocomotorSurfaces = 0;
	m_downhillOnly = FALSE;
}

//-------------------------------------------------------------------------------------------------
void LocomotorSet::addLocomotor(const LocomotorTemplate* lt)
{
	Locomotor* loco = TheLocomotorStore->newLocomotor(lt);
	if (loco)
	{
		m_locomotors.push_back(loco);
		m_validLocomotorSurfaces |= loco->getLegalSurfaces();
		if (loco->getIsDownhillOnly())
		{
			m_downhillOnly = TRUE;
		}
		else // Previous locos were gravity only, but this one isn't!
		{
			DEBUG_ASSERTCRASH(!m_downhillOnly,("LocomotorSet, YOU CAN NOT MIX DOWNHILL-ONLY LOCOMOTORS WITH NON-DOWNHILL-ONLY ONES."));
		}

	}
}

//-------------------------------------------------------------------------------------------------
Locomotor* LocomotorSet::findLocomotor(LocomotorSurfaceTypeMask t)
{
	for (LocomotorVector::iterator it = m_locomotors.begin(); it != m_locomotors.end(); ++it)
	{
		Locomotor* curLocomotor = *it;
		if (curLocomotor && (curLocomotor->getLegalSurfaces() & t))
			return curLocomotor;
	}
	return nullptr;
}
```
