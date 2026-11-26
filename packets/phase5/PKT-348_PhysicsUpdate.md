# Translation Packet `PKT-348`: `PhysicsUpdate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-348`
- **Work Package:** `WP-5.4` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/update/physics_update.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (21):**
  - `Common/AudioEventRTS.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/AudioEventRTS.h`
  - `Common/CRCDebug.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/CRCDebug.h`
  - `Common/GameAudio.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameAudio.h`
  - `Common/PerfTimer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/PerfTimer.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/LogicRandomValue.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/LogicRandomValue.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/CollideModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CollideModule.h`
  - `GameLogic/Module/ContainModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ContainModule.h`
  - `GameLogic/Module/CrushDie.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CrushDie.h`
  - `GameLogic/Module/PhysicsUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PhysicsUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/ScriptEngine.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ScriptEngine.h`
  - `GameLogic/TerrainLogic.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/TerrainLogic.h`
  - `GameLogic/Weapon.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h`
  - `PreRTS.h`
- **Implemented Module Facets (1):**
  - `CollideModuleInterface* getCollide()`
- **INI `FieldParse` Table Entries (18):**
  - `"Mass" => INI::parsePositiveNonZeroReal`
  - `"ShockResistance" => INI::parsePositiveNonZeroReal`
  - `"ShockMaxYaw" => INI::parsePositiveNonZeroReal`
  - `"ShockMaxPitch" => INI::parsePositiveNonZeroReal`
  - `"ShockMaxRoll" => INI::parsePositiveNonZeroReal`
  - `"ForwardFriction" => parseFrictionPerSec`
  - `"LateralFriction" => parseFrictionPerSec`
  - `"ZFriction" => parseFrictionPerSec`
  - `"AerodynamicFriction" => parseFrictionPerSec`
  - `"CenterOfMassOffset" => INI::parseReal`
  - `"AllowBouncing" => INI::parseBool`
  - `"AllowCollideForce" => INI::parseBool`
  - `"KillWhenRestingOnGround" => INI::parseBool`
  - `"MinFallHeightForDamage" => parseHeightToSpeed`
  - `"FallHeightDamageFactor" => INI::parseReal`
  - `"PitchRollYawFactor" => INI::parseReal`
  - `"VehicleCrashesIntoBuildingWeaponTemplate" => INI::parseWeaponTemplate`
  - `"VehicleCrashesIntoNonBuildingWeaponTemplate" => INI::parseWeaponTemplate`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (18 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferReal( &m_yawRate )`
  3. `xferReal( &m_rollRate )`
  4. `xferReal( &m_pitchRate )`
  5. `xferCoord3D( &m_accel )`
  6. `xferCoord3D( &m_prevAccel )`
  7. `xferCoord3D( &m_vel )`
  8. `xferCoord3D( &tmp )`
  9. `xferUser( &m_turning, sizeof( PhysicsTurningType ) )`
  10. `xferObjectID( &m_ignoreCollisionsWith )`
  11. `xferInt( &m_flags )`
  12. `xferReal( &m_mass )`
  13. `xferObjectID( &m_currentOverlap )`
  14. `xferObjectID( &m_previousOverlap )`
  15. `xferUnsignedInt( &m_motiveForceExpires )`
  16. `xferReal( &m_extraBounciness )`
  17. `xferReal( &m_extraFriction )`
  18. `xferReal( &m_velMag )`
- **PRNG Calls (3 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValue(-1.0f, 1.0f)`
  - `GameLogicRandomValue(-1.0f, 1.0f)`
  - `GameLogicRandomValue(-1.0f, 1.0f)`
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if defined(RTS_DEBUG) || !RETAIL_COMPATIBLE_CRC`

## 2.5 Existing Rust Symbol Index (Use These Exact Paths)
- `zh_logic::modules::update::base_renerate_update`: `BaseRegenerateUpdateModuleData`, `BaseRegenerateUpdate`
- `zh_logic::modules::update::deletion_update`: `DeletionUpdateModuleData`, `DeletionUpdate`
- `zh_logic::modules::update::dynamic_geometry_info_update`: `DIRECTION_FORWARD`, `DynamicGeometryInfoUpdateModuleData`, `DynamicGeometryInfoUpdate`
- `zh_logic::modules::update::dynamic_shroud_clearing_range_update`: `DynamicShroudClearingRangeUpdateModuleData`, `DscruState`, `DynamicShroudClearingRangeUpdate`
- `zh_logic::modules::update::fire_spread_update`: `FireSpreadUpdateModuleData`, `FireSpreadUpdate`
- `zh_logic::modules::update::fire_weapon_update`: `FireWeaponUpdateModuleData`, `FireWeaponUpdate`
- `zh_logic::modules::update::flammable_update`: `FlammableUpdateModuleData`, `FlammableUpdate`
- `zh_logic::modules::update::float_update`: `FloatUpdateModuleData`, `FloatUpdate`
- `zh_logic::modules::update::height_die_update`: `HeightDieUpdateModuleData`, `HeightDieUpdate`
- `zh_logic::modules::update::lifetime_update`: `LifetimeUpdateModuleData`, `LifetimeUpdate`
- `zh_logic::modules::update::physics_behavior`: `STICK_TO_GROUND`, `ALLOW_BOUNCE`, `APPLY_FRICTION2D_WHEN_AIRBORNE`, `UPDATE_EVER_RUN`, `WAS_AIRBORNE_LAST_FRAME`, `ALLOW_COLLIDE_FORCE`, `ALLOW_TO_FALL`, `HAS_PITCHROLLYAW`, `IMMUNE_TO_FALLING_DAMAGE`, `IS_IN_FREEFALL`, `IS_IN_UPDATE`, `IS_STUNNED`, `TURN_NEGATIVE`, `TURN_NONE`, ... (+4)
- `zh_logic::modules::update::power_plant_update`: `PowerPlantUpdateModuleData`, `PowerPlantUpdate`
- `zh_logic::modules::update::projectile_stream_update`: `MAX_PROJECTILE_STREAM`, `ProjectileStreamUpdateModuleData`, `ProjectileStreamUpdate`
- `zh_logic::modules::update::prone_update`: `ProneUpdateModuleData`, `ProneUpdate`
- `zh_logic::modules::update::radar_update`: `RadarUpdateModuleData`, `RadarUpdate`
- `zh_logic::modules::update::radius_decal_update`: `RadiusDecalUpdateModuleData`, `RadiusDecalUpdate`
- `zh_logic::modules::update::smart_bomb_target_homing_update`: `SmartBombTargetHomingUpdateModuleData`, `SmartBombTargetHomingUpdate`
- `zh_logic::modules::update::spy_vision_update`: `SpyVisionUpdateModuleData`, `SpyVisionUpdate`, `fn`
- `zh_logic::modules::update::weapon_bonus_update`: `WeaponBonusUpdateModuleData`, `WeaponBonusUpdate`
- `zh_common::core::dict`: `DICT_MAX_LEN`, `DictDataType`, `fn`, `DictValue`, `Dict`
- `zh_common::core::name_key_generator`: `SOCKET_COUNT`, `NAMEKEY_MAX`, `NameKeyGenerator`
- `zh_common::core::state_machine`: `StateId`, `MACHINE_DONE_STATE_ID`, `INVALID_STATE_ID`, `EXIT_MACHINE_WITH_SUCCESS`, `EXIT_MACHINE_WITH_FAILURE`, `STATE_CONTINUE_CODE`, `STATE_SUCCESS_CODE`, `STATE_FAILURE_CODE`, `STATE_SLEEP_FOREVER_FRAMES`, `MAX_TRANSITION_DEPTH`, `StateReturnType`, `fn`, `StateExitType`, `StateWorldView`, ... (+8)
- `zh_common::crc`: `Crc`, `fn`
- `zh_common::ini`: `INI_MAX_CHARS_PER_LINE`, `LOGICFRAMES_PER_SECOND`, `MSEC_PER_SECOND`, `LOGICFRAMES_PER_MSEC_REAL`, `MSEC_PER_LOGICFRAME_REAL`, `LOGICFRAMES_PER_SECONDS_REAL`, `SECONDS_PER_LOGICFRAME_REAL`, `RADS_PER_DEGREE`, `SEPS_DEFAULT`, `SEPS_PERCENT`, `SEPS_COLON`, `SEPS_QUOTE`, `END_TOKEN`, `Color`, ... (+22)
- `zh_common::math`: `fn`, `Coord2D`, `Coord3D`, `ICoord2D`, `ICoord3D`, `Region2D`, `Region3D`, `IRegion2D`, `IRegion3D`, `RealRange`, `RGBColor`, `RGBAColorReal`, `RGBAColorInt`, `Matrix3D`
- `zh_common::math::bez_fwd_iterator`: `BezFwdIterator`
- `zh_common::math::bezier_segment`: `USUAL_TOLERANCE`, `BEZ_BASIS_MATRIX`, `BezierMath`, `BezierSegment`
- `zh_common::math::discrete_circle`: `HorzLine`, `DiscreteCircle`
- `zh_common::math::gamemath`: `WWMATH_PI`, `WWMATH_TWO_PI`, `PI`, `TWO_PI`, `WWMATH_EPSILON`
- `zh_common::math::geometry`: `GeometryType`, `fn`, `GEOMETRY_NAMES`, `GeometryInfo`
- `zh_common::math::partition_solver`: `PairObjectIdAndUInt`, `PairObjectId`, `EntriesVec`, `SpacesVec`, `SolutionVec`, `SolutionType`, `PartitionSolver`
- `zh_common::random_value`: `INITIAL_SEED_WORDS`, `THE_MULT_FACTOR`, `DistributionType`, `PrngState`, `fn`
- `zh_common::types`: `Int`, `UnsignedInt`, `Short`, `UnsignedShort`, `Byte`, `UnsignedByte`, `Bool`, `Real`, `XferVersion`, `FALSE`, `TRUE`, `ObjectId`, `INVALID`, `TeamId`, ... (+11)
- `zh_common::xfer`: `XferMode`, `XferError`, `Xfer`, `Snapshot`, `XferCrc`, `fn`, `XferDeepCrc`, `XferSave`, `XferLoad`
- `zh_common::xfer::bit_flags`: `ARMOR_SET_BIT_NAMES`, `MODEL_CONDITION_BIT_NAMES`, `ArmorSetFlags`, `ModelConditionFlags`, `BitFlags`, `NUM_BITS`, `fn`
- `zh_logic::arena`: `ObjectSlot`, `INVALID`, `fn`, `ModuleSlot`, `ModuleDataId`, `ThingTemplateId`, `FacetMask`, `NONE`, `UPDATE`, `DIE`, `DAMAGE`, `CREATE`, `COLLIDE`, `BODY`, ... (+42)
- `zh_logic::modules::behavior::auto_heal_behavior`: `AutoHealBehaviorModuleData`, `AutoHealBehavior`
- `zh_logic::modules::behavior::battle_bus_slow_death_behavior`: `BattleBusSlowDeathBehaviorModuleData`, `BattleBusSlowDeathBehavior`
- `zh_logic::modules::behavior::behavior_module`: `BehaviorModuleData`, `fn`, `UpdateScheduleTracker`
- `zh_logic::modules::behavior::bridge_scaffold_behavior`: `BridgeScaffoldBehaviorModuleData`, `BridgeScaffoldBehavior`
- `zh_logic::modules::behavior::bridge_tower_behavior`: `BridgeTowerBehaviorModuleData`, `BridgeTowerBehavior`
- `zh_logic::modules::behavior::bunker_buster_behavior`: `BunkerBusterBehaviorModuleData`, `BunkerBusterBehavior`
- `zh_logic::modules::behavior::fire_weapon_when_damaged_behavior`: `FireWeaponWhenDamagedBehaviorModuleData`, `FireWeaponWhenDamagedBehavior`
- `zh_logic::modules::behavior::fire_weapon_when_dead_behavior`: `FireWeaponWhenDeadBehaviorModuleData`, `FireWeaponWhenDeadBehavior`
- `zh_logic::modules::behavior::grant_stealth_behavior`: `GrantStealthBehaviorModuleData`, `GrantStealthBehavior`
- `zh_logic::modules::behavior::instant_death_behavior`: `InstantDeathBehaviorModuleData`, `InstantDeathBehavior`
- `zh_logic::modules::behavior::jet_slow_death_behavior`: `JetSlowDeathBehaviorModuleData`, `JetSlowDeathBehavior`
- `zh_logic::modules::behavior::neuton_blast_behavior`: `NeutronBlastBehaviorModuleData`, `NeutronBlastBehavior`
- `zh_logic::modules::behavior::overcharge_behavior`: `OverchargeBehaviorModuleData`, `OverchargeBehavior`
- `zh_logic::modules::behavior::poisoned_behavior`: `PoisonedBehaviorModuleData`, `PoisonedBehavior`
- `zh_logic::modules::behavior::prison_behavior`: `PrisonBehaviorModuleData`, `PrisonVisual`, `PrisonBehavior`
- `zh_logic::modules::behavior::propaganda_tower_behavior`: `PropagandaTowerBehaviorModuleData`, `PropagandaTowerBehavior`
- `zh_logic::modules::behavior::slow_death_behavior`: `SLOW_DEATH_PHASE_NAMES`, `SlowDeathPhaseType`, `COUNT`, `SD_FLAG_SLOW_DEATH_ACTIVATED`, `SD_FLAG_MIDPOINT_EXECUTED`, `SD_FLAG_FLUNG_INTO_AIR`, `SD_FLAG_BOUNCED`, `SD_EFFECT_HAS_FX`, `SD_EFFECT_HAS_OCL`, `SD_EFFECT_HAS_WEAPON`, `SlowDeathBehaviorModuleData`, `fn`, `SlowDeathCore`, `SlowDeathBehavior`
- `zh_logic::modules::behavior::supply_warehouse_crippling_behavior`: `SupplyWarehouseCripplingBehaviorModuleData`, `SupplyWarehouseCripplingBehavior`
- `zh_logic::modules::behavior::tech_building_behavior`: `TechBuildingBehaviorModuleData`, `TechBuildingBehavior`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PhysicsUpdate.h`
```cpp
// PhysicsUpdate.h
// Simple rigid body physics
// Author: Michael S. Booth, November 2001

#pragma once

#include "Common/AudioEventRTS.h"
#include "Common/GameAudio.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Module/CollideModule.h"

enum ObjectID CPP_11(: Int);

enum PhysicsTurningType CPP_11(: Int)
{
	TURN_NEGATIVE = -1,
	TURN_NONE = 0,
	TURN_POSITIVE = 1
};

//-------------------------------------------------------------------------------------------------
class PhysicsBehaviorModuleData : public UpdateModuleData
{
public:
	Real	m_mass;
	Real	m_shockResistance;
	Real	m_shockMaxYaw;
	Real	m_shockMaxPitch;
	Real	m_shockMaxRoll;
	Real	m_forwardFriction;
	Real	m_lateralFriction;
	Real	m_ZFriction;
	Real	m_aerodynamicFriction;	// The percent of the wind resistance effect you suffer from
	Real	m_centerOfMassOffset;	// Distance the center of mass is from the center of geometry, to control pitch rate
	Bool	m_killWhenRestingOnGround;	// when airborne==false and vel==0, kill it.
	Bool	m_allowBouncing;
	Bool	m_allowCollideForce;
	Real	m_minFallSpeedForDamage;
	Real	m_fallHeightDamageFactor;
	Real	m_pitchRollYawFactor;

	const WeaponTemplate* m_vehicleCrashesIntoBuildingWeaponTemplate;
	const WeaponTemplate* m_vehicleCrashesIntoNonBuildingWeaponTemplate;

	PhysicsBehaviorModuleData();
	static void buildFieldParse(MultiIniFieldParse& p);
};

//-------------------------------------------------------------------------------------------------
/**
 * Simple rigid body physics update module
 */
class PhysicsBehavior : public UpdateModule,
												public CollideModuleInterface
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( PhysicsBehavior, "PhysicsBehavior" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( PhysicsBehavior, PhysicsBehaviorModuleData )

public:
	PhysicsBehavior( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype defined by MemoryPoolObject

	static Int getInterfaceMask() { return UpdateModule::getInterfaceMask() | (MODULEINTERFACE_COLLIDE); }

	virtual void onObjectCreated() override;

	// BehaviorModule
	virtual CollideModuleInterface* getCollide() override { return this; }

	// CollideModuleInterface
	virtual void onCollide( Object *other, const Coord3D *loc, const Coord3D *normal ) override;
	virtual Bool wouldLikeToCollideWith(const Object* other) const override { return false; }
	virtual Bool isCarBombCrateCollide() const override { return false; }
	virtual Bool isHijackedVehicleCrateCollide() const override { return false; }
	virtual Bool isRailroad() const override { return false;}
	virtual Bool isSalvageCrateCollide() const override { return false; }
	virtual Bool isSabotageBuildingCrateCollide() const override { return FALSE; }

	// UpdateModuleInterface
	virtual UpdateSleepTime update() override;
	// Disabled conditions to process -- all
	virtual DisabledMaskType getDisabledTypesToProcess() const override { return DISABLEDMASK_ALL; }

	void applyForce( const Coord3D *force );		///< apply a force at the object's CG
	void applyShock( const Coord3D *force );					///< apply a shockwave force against the object's CG
	void applyRandomRotation();											  ///< apply a random rotation at the object's CG
	void addVelocityTo(const Coord3D* vel) ;

	/**
		identical to applyForce, except that no forward friction will be applied to the physics
		this frame. (lateral and z-friction will still be applied.)
	*/
	void applyMotiveForce( const Coord3D *force );

	/** This is a force clear for when objects are going out of bounds, so the locomotor
		can push them back into legal space. */
	void clearAcceleration() { m_accel.zero(); }

	/**
		add the velocity of 'this' to 'that'... useful when a unit disgorges another unit
		and you want to maintain relative velocities
	*/
	void transferVelocityTo(PhysicsBehavior* that) const;

	/// @todo Rotations should be handled by this system as well (MSB)

	void setAngles( Real yaw, Real pitch, Real roll );
	Real getMass() const;
	void setMass( Real mass ) { m_mass = mass; }
	Real getCenterOfMassOffset() const { return getPhysicsBehaviorModuleData()->m_centerOfMassOffset; }

	const Coord3D *getAcceleration() const { return &m_prevAccel; }		///< get last frame's acceleration
	const Coord3D *getVelocity() const { return &m_vel; }			///< get current velocity
	Real getVelocityMagnitude() const;																		///< return velocity magnitude (speed)
	Real getForwardSpeed2D() const;															///< compute speed along object's 2d direction vector
	Real getForwardSpeed3D() const;															///< compute speed along object's 3d direction vector

#if USE_RETAIL_PHYSICS_FORWARD_SPEED_DISCREPANCY_IN_CINEMATICS()
	static Bool useLegacyForwardSpeedInCinematics();
#endif

	static Bool useLegacyForwardSpeed()
	{
#if USE_RETAIL_PHYSICS_FORWARD_SPEED_DISCREPANCY()
		return true;
#elif USE_RETAIL_PHYSICS_FORWARD_SPEED_DISCREPANCY_IN_CINEMATICS()
		return useLegacyForwardSpeedInCinematics();
#else
		return false;
#endif
	}

	ObjectID getCurrentOverlap() const;					///< return object(s) being overlapped
	ObjectID getPreviousOverlap() const;					///< return object(s) that were overlapped last frame
	ObjectID getLastCollidee() const;					///< return object that was last collided with... can be quite old
	Bool isCurrentlyOverlapped(Object *obj) const;
	Bool wasPreviouslyOverlapped(Object *obj) const;
	void addOverlap(Object *obj);

	Bool isMotive() const;

	PhysicsTurningType getTurning() const { return m_turning; }		///< 0 = not turning, -1 = turn negative, 1 = turn positive.
	void setTurning(PhysicsTurningType turning) { m_turning = turning; }

	/** This is a force scrub for velocity when ai objects are colliding. */
	void scrubVelocity2D( Real desiredVelocity );

	// note that, unlike scrubVelocity2D(), this is a signed limit
	void scrubVelocityZ( Real desiredVelocity );

	void setPitchRate(Real pitch);
	void setRollRate(Real roll);
	void setYawRate(Real yaw);

	/*
		stickToGround and allowToFall seem contradictory... here's the deal.

		if stickToGround is true, you stick to the ground, otherwise your z isn't messed with.

		HOWEVER, if allowToFall is true, stickToGround is ignored, and z is never messed with.
		Also, allowToFall is reset to "false" when you hit the ground, so you can set it with
		(relatively) little fear.
	*/
	void setAllowToFall(Bool allow) { setFlag(ALLOW_TO_FALL, allow); }
	void setStickToGround(Bool allow) { setFlag(STICK_TO_GROUND, allow); }
	void setAllowBouncing(Bool allow) { setFlag(ALLOW_BOUNCE, allow); }
	void setAllowCollideForce(Bool allow) { setFlag(ALLOW_COLLIDE_FORCE, allow); }
	void setAllowAirborneFriction(Bool allow) { setFlag(APPLY_FRICTION2D_WHEN_AIRBORNE, allow); }
	void setImmuneToFallingDamage(Bool allow) { setFlag(IMMUNE_TO_FALLING_DAMAGE, allow); }
	void setStunned(Bool allow) { setFlag(IS_STUNNED, allow); }

	Bool getAllowToFall() const { return getFlag(ALLOW_TO_FALL); }

	void setIsInFreeFall(Bool allow) { setFlag(IS_IN_FREEFALL, allow); }
	Bool getIsInFreeFall() const { return getFlag(IS_IN_FREEFALL); }

	Bool getIsStunned() const { return getFlag(IS_STUNNED); }

	void setExtraBounciness(Real b) { m_extraBounciness = b; }
	void setExtraFriction(Real b) { m_extraFriction = b; }

	void setBounceSound(const AudioEventRTS* bounceSound);
	const AudioEventRTS* getBounceSound() { return m_bounceSound ? m_bounceSound.Peek() : TheAudio->getValidSilentAudioEvent(); }

	/**
		Reset all values (vel, accel, etc) to starting values.
		You should ALMOST NEVER use this; it's intended for cases where you need
		to nuke everything, like when an existing object goes off the map and you
		want to bring it back on with all-new values, and want to start with a clean
		slate. Rule of thumb: if the object was in existence on the map in the previous
		frame, don't call this... apply forces instead!
	*/
	void resetDynamicPhysics();

	void setIgnoreCollisionsWith(const Object* obj);
	Bool isIgnoringCollisionsWith(ObjectID id) const;

	Bool getAllowCollideForce() const { return getFlag(ALLOW_COLLIDE_FORCE); }

protected:

	/*
		Physics runs in its own phase, after AI, but before all others.
		It's actually quite important that AI (the thing that drives Locomotors) and Physics
		run in the same order, relative to each other, for a given object; otherwise,
		interesting oscillations can occur in some situations, with friction being applied
		either before or after the locomotive force, making for huge stuttery messes. (srj)
	*/
	virtual SleepyUpdatePhase getUpdatePhase() const override { return PHASE_PHYSICS; }

	Real getAerodynamicFriction() const;
	Real getForwardFriction() const;
	Real getLateralFriction() const;
	Real getZFriction() const;

	void applyGravitationalForces();
	void applyFrictionalForces();
	Bool handleBounce(Real oldZ, Real newZ, Real groundZ, Coord3D* bounceForce);
	void applyYPRDamping(Real factor);
	UpdateSleepTime calcSleepTime() const;

	void doBounceSound(const Coord3D& prevPos);

	Bool checkForOverlapCollision(Object *other);

	void testStunnedUnitForDestruction();

private:

	enum PhysicsFlagsType
	{
		// Note - written out in save/load xfer; don't change these numbers.
		STICK_TO_GROUND									= 0x0001,
		ALLOW_BOUNCE										= 0x0002,
		APPLY_FRICTION2D_WHEN_AIRBORNE	= 0x0004,
		UPDATE_EVER_RUN									= 0x0008,
		WAS_AIRBORNE_LAST_FRAME					= 0x0010,
		ALLOW_COLLIDE_FORCE							= 0x0020,
		ALLOW_TO_FALL										= 0x0040,
		HAS_PITCHROLLYAW								= 0x0080,
		IMMUNE_TO_FALLING_DAMAGE				= 0x0100,
		IS_IN_FREEFALL									= 0x0200,
		IS_IN_UPDATE										= 0x0400,
		IS_STUNNED											= 0x0800, // Added in Zero Hour
	};

	/*
		Note: these are private because you should never manipulate these directly,
		even if you are a subclass... if you want to change the acceleration, you
		MUST call applyForce().
	*/
	Real												m_yawRate;								///< rate of rotation around up vector
	Real												m_rollRate;								///< rate of rotation around forward vector
	Real												m_pitchRate;							///< rate or rotation around side vector
	RefCountPtr<DynamicAudioEventRTS> m_bounceSound;			///< The sound for when this thing bounces, or nullptr
	Coord3D											m_accel;									///< current acceleration
	Coord3D											m_prevAccel;							///< last frame's acceleration
	Coord3D											m_vel;										///< current velocity
	PhysicsTurningType					m_turning;								///< 0 = not turning, -1 = turn negative, 1 = turn positive.
	ObjectID										m_ignoreCollisionsWith;
	Int													m_flags;
	Real												m_mass;
	ObjectID										m_currentOverlap;					///< object(s) being overlapped, if any
	ObjectID										m_previousOverlap;				///< last frame's object(s) being overlapped
	ObjectID										m_lastCollidee;						///< ID of the last object I collided with, can be quite old.
	UnsignedInt									m_motiveForceExpires;			///< frames at which "recent" applyMotiveForce is no longer considered
	Real												m_extraBounciness;				///< modifier to ground stiffness
	Real												m_extraFriction;					///< modifier to friction(s)
	ProjectileUpdateInterface*	m_pui;
	mutable Real								m_velMag;									///< magnitude of cur vel (recalced when m_vel changes)

	Bool												m_originalAllowBounce;		///< original state of allow bounce

	void setFlag(PhysicsFlagsType f, Bool set) { if (set) m_flags |= f; else m_flags &= ~f; }
	Bool getFlag(PhysicsFlagsType f) const { return (m_flags & f) != 0; }


};

//-------------------------------------------------------------------------------------------------
inline ObjectID PhysicsBehavior::getCurrentOverlap() const
{
	return m_currentOverlap;
}

//-------------------------------------------------------------------------------------------------
inline ObjectID PhysicsBehavior::getPreviousOverlap() const
{
	return m_previousOverlap;
}

//-------------------------------------------------------------------------------------------------
inline ObjectID PhysicsBehavior::getLastCollidee() const
{
	return m_lastCollidee;
}
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/PhysicsUpdate.cpp`
```cpp
// PhysicsBehavior.cpp
// Simple rigid body physics
// Author: Michael S. Booth, November 2001

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

// please talk to MDC (x36804) before taking this out
#define NO_DEBUG_CRC

#include <Utility/intrin_compat.h>	// For _isnan compatibility
#include "Common/PerfTimer.h"
#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/CrushDie.h"		// for CrushEnum
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/ScriptEngine.h"
#include "GameLogic/TerrainLogic.h"
#include "GameLogic/Weapon.h"
#include "GameLogic/LogicRandomValue.h"

const Real DEFAULT_MASS = 1.0f;

const Real DEFAULT_SHOCK_YAW = 0.05f;
const Real DEFAULT_SHOCK_PITCH = 0.025f;
const Real DEFAULT_SHOCK_ROLL = 0.025f;

const Real DEFAULT_FORWARD_FRICTION = 0.15f;
const Real DEFAULT_LATERAL_FRICTION = 0.15f;
const Real DEFAULT_Z_FRICTION = 0.8f;
const Real DEFAULT_AERO_FRICTION = 0.0f;

// Air friction used to be completely separate and unclamped, so this constant must be split.
// The fact it defaults to 0 shows it.
const Real MIN_AERO_FRICTION = 0.00f;
const Real MIN_NON_AERO_FRICTION = 0.01f;
const Real MAX_FRICTION = 0.99f;

const Real STUN_RELIEF_EPSILON = 0.5f;

#include "Common/CRCDebug.h"

const Int MOTIVE_FRAMES = static_cast<float>(LOGICFRAMES_PER_SECOND) / 3;

#define SLEEPY_PHYSICS


//-------------------------------------------------------------------------------------------------
static Real angleBetweenVectors(const Coord3D& inCurDir, const Coord3D& inGoalDir)
{
	Vector3 curDir;
	curDir.X = inCurDir.x;
	curDir.Y = inCurDir.y;
	curDir.Z = inCurDir.z;
	curDir.Normalize();

	Vector3 goalDir;
	goalDir.X = inGoalDir.x;
	goalDir.Y = inGoalDir.y;
	goalDir.Z = inGoalDir.z;
	goalDir.Normalize();

	// dot of two unit vectors is cos of angle between them.
	Real cosine = Vector3::Dot_Product(curDir, goalDir);

	// bound it in case of numerical error
	Real angleBetween = (Real)ACos(clamp(-1.0f, cosine, 1.0f));

	return angleBetween;
}

//-------------------------------------------------------------------------------------------------
static Real heightToSpeed(Real height)
{
	// don't bother trying to remember how far we've fallen; instead,
	// back-calc it from our speed & gravity... v = sqrt(2*g*h)
	return WWMath::Sqrt(WWMath::Fabs(2.0f * TheGlobalData->m_gravity * height));
}

//-------------------------------------------------------------------------------------------------
PhysicsBehaviorModuleData::PhysicsBehaviorModuleData()
{
	m_mass = DEFAULT_MASS;
	m_shockResistance = 0.0f;
	m_shockMaxYaw = DEFAULT_SHOCK_YAW;
	m_shockMaxPitch = DEFAULT_SHOCK_PITCH;
	m_shockMaxRoll = DEFAULT_SHOCK_ROLL;

	m_forwardFriction = DEFAULT_FORWARD_FRICTION;
	m_lateralFriction = DEFAULT_LATERAL_FRICTION;
	m_ZFriction = DEFAULT_Z_FRICTION;
	m_aerodynamicFriction = DEFAULT_AERO_FRICTION;
	m_centerOfMassOffset = 0.0f;
	m_allowBouncing = false;
	m_allowCollideForce = true;
	m_killWhenRestingOnGround = false;
	m_minFallSpeedForDamage = heightToSpeed(40.0f);
	m_fallHeightDamageFactor = 1.0f;	// was 10. now is 1.
	/*
		thru some bizarre editing mishap, we have been double-apply pitch/roll/yaw rates
		to objects for, well, a long time, it looks like. I have corrected that problem
		in the name of efficiency, but to maintain the same visual appearance without having
		to edit every freaking INI in the world at this point, I am just multiplying
		all the results by a factor so that the effect is the same (but with less execution time).
		I have put this factor into INI in the unlikely event we ever need to change it,
		but defaulting it to 2 is, in fact, the right thing for now... (srj)
	*/
	m_pitchRollYawFactor = 2.0f;
	m_vehicleCrashesIntoBuildingWeaponTemplate = TheWeaponStore->findWeaponTemplate("VehicleCrashesIntoBuildingWeapon");
	m_vehicleCrashesIntoNonBuildingWeaponTemplate = TheWeaponStore->findWeaponTemplate("VehicleCrashesIntoNonBuildingWeapon");

}

//-------------------------------------------------------------------------------------------------
static void parseHeightToSpeed( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	// don't bother trying to remember how far we've fallen; instead,
	// back-calc it from our speed & gravity... v = WWMath::Sqrt(2*g*h)
	Real height = INI::scanReal(ini->getNextToken());
	*(Real *)store = heightToSpeed(height);
}

//-------------------------------------------------------------------------------------------------
static void parseFrictionPerSec( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	Real fricPerSec = INI::scanReal(ini->getNextToken());
	Real fricPerFrame = fricPerSec * SECONDS_PER_LOGICFRAME_REAL;
	*(Real *)store = fricPerFrame;
}

//-------------------------------------------------------------------------------------------------
/*static*/ void PhysicsBehaviorModuleData::buildFieldParse(MultiIniFieldParse& p)
{
  UpdateModuleData::buildFieldParse(p);

	static const FieldParse dataFieldParse[] =
	{
		{ "Mass",								INI::parsePositiveNonZeroReal,		nullptr, offsetof( PhysicsBehaviorModuleData, m_mass ) },

		{ "ShockResistance",		INI::parsePositiveNonZeroReal,		nullptr, offsetof( PhysicsBehaviorModuleData, m_shockResistance ) },
		{ "ShockMaxYaw",				INI::parsePositiveNonZeroReal,		nullptr, offsetof( PhysicsBehaviorModuleData, m_shockMaxYaw ) },
		{ "ShockMaxPitch",			INI::parsePositiveNonZeroReal,		nullptr, offsetof( PhysicsBehaviorModuleData, m_shockMaxPitch ) },
		{ "ShockMaxRoll",				INI::parsePositiveNonZeroReal,		nullptr, offsetof( PhysicsBehaviorModuleData, m_shockMaxRoll ) },

		{ "ForwardFriction",			parseFrictionPerSec,		nullptr, offsetof( PhysicsBehaviorModuleData, m_forwardFriction ) },
		{ "LateralFriction",			parseFrictionPerSec,		nullptr, offsetof( PhysicsBehaviorModuleData, m_lateralFriction ) },
		{ "ZFriction",						parseFrictionPerSec,		nullptr, offsetof( PhysicsBehaviorModuleData, m_ZFriction ) },
		{ "AerodynamicFriction",	parseFrictionPerSec,		nullptr, offsetof( PhysicsBehaviorModuleData, m_aerodynamicFriction ) },

		{ "CenterOfMassOffset",	INI::parseReal,		nullptr, offsetof( PhysicsBehaviorModuleData, m_centerOfMassOffset ) },
		{ "AllowBouncing",			INI::parseBool,		nullptr, offsetof( PhysicsBehaviorModuleData, m_allowBouncing ) },
		{ "AllowCollideForce",	INI::parseBool,		nullptr, offsetof( PhysicsBehaviorModuleData, m_allowCollideForce ) },
		{ "KillWhenRestingOnGround", INI::parseBool, nullptr, offsetof( PhysicsBehaviorModuleData, m_killWhenRestingOnGround) },

		{ "MinFallHeightForDamage",			parseHeightToSpeed,		nullptr, offsetof( PhysicsBehaviorModuleData, m_minFallSpeedForDamage) },
		{ "FallHeightDamageFactor",			INI::parseReal,		nullptr, offsetof( PhysicsBehaviorModuleData, m_fallHeightDamageFactor) },
		{ "PitchRollYawFactor",			INI::parseReal,		nullptr, offsetof( PhysicsBehaviorModuleData, m_pitchRollYawFactor) },

		{ "VehicleCrashesIntoBuildingWeaponTemplate", INI::parseWeaponTemplate, nullptr, offsetof(PhysicsBehaviorModuleData, m_vehicleCrashesIntoBuildingWeaponTemplate) },
		{ "VehicleCrashesIntoNonBuildingWeaponTemplate", INI::parseWeaponTemplate, nullptr, offsetof(PhysicsBehaviorModuleData, m_vehicleCrashesIntoNonBuildingWeaponTemplate) },

		{ nullptr, nullptr, nullptr, 0 }
	};
  p.add(dataFieldParse);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------

const Real INVALID_VEL_MAG = -1.0f;

//-------------------------------------------------------------------------------------------------
PhysicsBehavior::PhysicsBehavior( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{
	m_accel.zero();
	m_prevAccel = m_accel;
	m_vel.zero();
	m_velMag = 0.0f;
	m_yawRate = 0.0f;
	m_rollRate = 0.0f;
	m_pitchRate = 0.0f;
	m_mass = getPhysicsBehaviorModuleData()->m_mass;
	m_motiveForceExpires = 0;

	m_flags = 0;
	m_extraBounciness = 0.0f;
	m_extraFriction = 0.0f;

	m_currentOverlap = INVALID_ID;
	m_previousOverlap = INVALID_ID;
	m_lastCollidee = INVALID_ID;

	m_ignoreCollisionsWith = INVALID_ID;

	setAllowBouncing(getPhysicsBehaviorModuleData()->m_allowBouncing);
	setAllowCollideForce(getPhysicsBehaviorModuleData()->m_allowCollideForce);

	m_pui = nullptr;
	m_bounceSound = nullptr;

#ifdef SLEEPY_PHYSICS
	setWakeFrame(getObject(), UPDATE_SLEEP_NONE);
#endif
}

//-------------------------------------------------------------------------------------------------
static ProjectileUpdateInterface* getPui(Object* obj)
{
	if (!obj->isKindOf(KINDOF_PROJECTILE))
		return nullptr;

	ProjectileUpdateInterface* objPui = nullptr;
	for (BehaviorModule** u = obj->getBehaviorModules(); *u; ++u)
	{
		if ((objPui = (*u)->getProjectileUpdateInterface()) != nullptr)
			return objPui;
	}
	return nullptr;
}

//-------------------------------------------------------------------------------------------------
void PhysicsBehavior::onObjectCreated()
{
	m_pui = getPui(getObject());
}

//-------------------------------------------------------------------------------------------------
PhysicsBehavior::~PhysicsBehavior()
{
}

//-------------------------------------------------------------------------------------------------
void PhysicsBehavior::setIgnoreCollisionsWith(const Object* obj)
{
	m_ignoreCollisionsWith = obj ? obj->getID() : INVALID_ID;
}

//-------------------------------------------------------------------------------------------------
Bool PhysicsBehavior::isIgnoringCollisionsWith(ObjectID id) const
{
	return id != INVALID_ID && id == m_ignoreCollisionsWith;
}

//-------------------------------------------------------------------------------------------------
Real PhysicsBehavior::getAerodynamicFriction() const
{
	Real f = getPhysicsBehaviorModuleData()->m_aerodynamicFriction + m_extraFriction;
	if (f < MIN_AERO_FRICTION) f = MIN_AERO_FRICTION;
	if (f > MAX_FRICTION) f = MAX_FRICTION;
	return f;
}

//-------------------------------------------------------------------------------------------------
Real PhysicsBehavior::getForwardFriction() const
{
	Real f = getPhysicsBehaviorModuleData()->m_forwardFriction + m_extraFriction;
	if (f < MIN_NON_AERO_FRICTION) f = MIN_NON_AERO_FRICTION;
	if (f > MAX_FRICTION) f = MAX_FRICTION;
	return f;
}

//-------------------------------------------------------------------------------------------------
Real PhysicsBehavior::getLateralFriction() const
{
	Real f = getPhysicsBehaviorModuleData()->m_lateralFriction + m_extraFriction;
	if (f < MIN_NON_AERO_FRICTION) f = MIN_NON_AERO_FRICTION;
	if (f > MAX_FRICTION) f = MAX_FRICTION;
	return f;
}

//-------------------------------------------------------------------------------------------------
Real PhysicsBehavior::getZFriction() const
{
	Real f = getPhysicsBehaviorModuleData()->m_ZFriction + m_extraFriction;
	if (f < MIN_NON_AERO_FRICTION) f = MIN_NON_AERO_FRICTION;
	if (f > MAX_FRICTION) f = MAX_FRICTION;
	return f;
}

//-------------------------------------------------------------------------------------------------
/**
 * Apply a force at the object's CG
 */
void PhysicsBehavior::applyForce( const Coord3D *force )
{
// TheSuperHackers @info helmutbuhler 06/05/2025 This debug mutates the code to become CRC incompatible
#if defined(RTS_DEBUG) || !RETAIL_COMPATIBLE_CRC
	DEBUG_ASSERTCRASH(!(_isnan(force->x) || _isnan(force->y) || _isnan(force->z)), ("PhysicsBehavior::applyForce force NAN!"));
#endif
	if (_isnan(force->x) || _isnan(force->y) || _isnan(force->z)) {
		return;
	}
	// F = ma  -->  a = F/m  (divide force by mass)
	Real mass = getMass();
	Coord3D modForce = *force;
	if (isMotive())
	{
		const Coord3D *dir = getObject()->getUnitDirectionVector2D();
		// Only accept the lateral acceleration.
		Real lateralDot = force->x * (-dir->y) + force->y * dir->x;
		modForce.x = lateralDot * -dir->y;
		modForce.y = lateralDot * dir->x;
	}

	Real massInv = 1.0f / mass;
	m_accel.x += modForce.x * massInv;
	m_accel.y += modForce.y * massInv;
	m_accel.z += modForce.z * massInv;

	//DEBUG_ASSERTCRASH(!(_isnan(m_accel.x) || _isnan(m_accel.y) || _isnan(m_accel.z)), ("PhysicsBehavior::applyForce accel NAN!"));
	//DEBUG_ASSERTCRASH(!(_isnan(m_vel.x) || _isnan(m_vel.y) || _isnan(m_vel.z)), ("PhysicsBehavior::applyForce vel NAN!"));
	//DEBUG_ASSERTCRASH(fabs(force->z) < 3, ("unlikely z-force"));
#ifdef SLEEPY_PHYSICS
	if (getFlag(IS_IN_UPDATE))
	{
		// we're applying a force from inside our own update (probably for a bounce or friction).
		// just do nothing, since update will calculate the correct sleep behavior at the end.
	}
	else
	{
		// when a force is applied by an external module, we must wake up, even if the force is zero.
		setWakeFrame(getObject(), UPDATE_SLEEP_NONE);
	}
#endif
}

//-------------------------------------------------------------------------------------------------
/**
 * Apply a shocwave force at the object's CG
 */
void PhysicsBehavior::applyShock( const Coord3D *force )
{
	Coord3D resistedForce = *force;
	resistedForce.scale( 1.0f - min( 1.0f, max( 0.0f, getPhysicsBehaviorModuleData()->m_shockResistance ) ) );

	// Apply the processed shock force to the object
	applyForce(&resistedForce);
}

//-------------------------------------------------------------------------------------------------
/**
 * Apply a random rotation at the object's CG
 */
void PhysicsBehavior::applyRandomRotation()
{
	// Ignore any pitch, roll & yaw rotation if behavior is stick to ground
	if (getFlag(STICK_TO_GROUND))	return;

	// Set bounce to true for a while until the unit is complete bouncing
	setAllowBouncing(true);

	Real randomModifier;

	randomModifier = GameLogicRandomValue(-1.0f, 1.0f);
	m_yawRate += getPhysicsBehaviorModuleData()->m_shockMaxYaw * randomModifier;

	randomModifier = GameLogicRandomValue(-1.0f, 1.0f);
	m_pitchRate += getPhysicsBehaviorModuleData()->m_shockMaxPitch * randomModifier;

	randomModifier = GameLogicRandomValue(-1.0f, 1.0f);
	m_rollRate += getPhysicsBehaviorModuleData()->m_shockMaxRoll * randomModifier;

#ifdef SLEEPY_PHYSICS
	if (getFlag(IS_IN_UPDATE))
	{
		// we're applying a force from inside our own update (probably for a bounce or friction).
		// just do nothing, since update will calculate the correct sleep behavior at the end.
	}
	else
	{
		// when a force is applied by an external module, we must wake up, even if the force is zero.
		setWakeFrame(getObject(), UPDATE_SLEEP_NONE);
	}
#endif
}

//-------------------------------------------------------------------------------------------------
Bool PhysicsBehavior::isMotive() const
{
	return m_motiveForceExpires > TheGameLogic->getFrame();
}

//-------------------------------------------------------------------------------------------------
void PhysicsBehavior::applyMotiveForce( const Coord3D *force )
{
	m_motiveForceExpires = 0; // make it accept this force unquestioningly :)
	applyForce(force);
	m_motiveForceExpires = TheGameLogic->getFrame() + MOTIVE_FRAMES;
}

//-------------------------------------------------------------------------------------------------
void PhysicsBehavior::resetDynamicPhysics()
{
	m_accel.zero();
	m_prevAccel.zero();
	m_vel.zero();
	m_velMag = 0.0f;
	m_turning = TURN_NONE;
	m_yawRate = 0;
	m_rollRate = 0;
	m_pitchRate = 0;
	setFlag(HAS_PITCHROLLYAW, false);
#ifdef SLEEPY_PHYSICS
	DEBUG_ASSERTCRASH(!getFlag(IS_IN_UPDATE), ("hmm, should not happen, may not work"));
	setWakeFrame(getObject(), calcSleepTime());
#endif
}

//-------------------------------------------------------------------------------------------------
void PhysicsBehavior::applyGravitationalForces()
{
	m_accel.z += TheGlobalData->m_gravity;
}

//-------------------------------------------------------------------------------------------------
void PhysicsBehavior::applyFrictionalForces()
{
	//Are we a plane that is taxiing on a deck with a height offset?
	Bool deckTaxiing = getObject()->testStatus( OBJECT_STATUS_DECK_HEIGHT_OFFSET ) &&
										 getObject()->getAI() &&
										 getObject()->getAI()->getCurLocomotorSetType() == LOCOMOTORSET_TAXIING;

	if (getFlag(APPLY_FRICTION2D_WHEN_AIRBORNE) || !getObject()->isSignificantlyAboveTerrain() || deckTaxiing )
	{
		applyYPRDamping(1.0f - DEFAULT_LATERAL_FRICTION);

		if (m_vel.x || m_vel.y)
		{
			const Coord3D *dir = getObject()->getUnitDirectionVector2D();
			Real mass = getMass();

			Real lateralDot = m_vel.x * (-dir->y) + m_vel.y * dir->x;
			Real lateralVel_x = lateralDot * -dir->y;
			Real lateralVel_y = lateralDot * dir->x;

			Real lf = mass * getLateralFriction();

			Coord3D accel;
			accel.x = -(lf * lateralVel_x);
			accel.y = -(lf * lateralVel_y);
			accel.z = 0.0f;

			if (!isMotive())
			{
				Real forwardDot = m_vel.x * dir->x + m_vel.y * dir->y;
				Real forwardVel_x = forwardDot * dir->x;
				Real forwardVel_y = forwardDot * dir->y;
				Real ff = mass * getForwardFriction();
				accel.x += -(ff * forwardVel_x);
				accel.y += -(ff * forwardVel_y);
			}
			applyForce(&accel);
		}
	}
	else
	{
		Real aerodynamics = -getAerodynamicFriction();	// negated!

		// Air resistance is proportional to velocity in the opposite direction
		m_accel.x += m_vel.x * aerodynamics;
		m_accel.y += m_vel.y * aerodynamics;
		m_accel.z += m_vel.z * aerodynamics;

		applyYPRDamping(1.0f + aerodynamics);	// since aero is negated, this results in 1.0-getAerodynamicFriction()
	}
}


//-------------------------------------------------------------------------------------------------
Bool PhysicsBehavior::handleBounce(Real oldZ, Real newZ, Real groundZ, Coord3D* bounceForce)
{
	if (getFlag(ALLOW_BOUNCE) && newZ <= groundZ)
	{
		const Real MIN_STIFF = 0.01f;
		const Real MAX_STIFF = 0.99f;
		Real stiffness = TheGlobalData->m_groundStiffness;
		if (stiffness < MIN_STIFF) stiffness = MIN_STIFF;
		if (stiffness > MAX_STIFF) stiffness = MAX_STIFF;

		Real desiredAccelZ = 0.0f;
		Real vz = getVelocity()->z;
		if (oldZ > groundZ && vz < 0.0f)
		{
			desiredAccelZ = WWMath::Fabs(vz) * stiffness;
		}

		bounceForce->x = 0.0f;
		bounceForce->y = 0.0f;
		bounceForce->z = getMass() * desiredAccelZ;

		const Real DAMPING = 0.7f;
		applyYPRDamping(DAMPING);

		if (vz < 0.0f)
		{
			Vector3 zvec = getObject()->getTransformMatrix()->Get_Z_Vector();
			const Real rollAngle = (zvec.Z > 0) ? 0 : PI;
			// don't flip both pitch and roll... we'll "flip" twice.
			const Real pitchAngle = 0;
			Real yawAngle = getObject()->getTransformMatrix()->Get_Z_Rotation();
			setAngles(yawAngle, pitchAngle, rollAngle);
		}

		if(bounceForce->z > 0.0f)
		{
			testStunnedUnitForDestruction();
			return true;
		}
		else
		{
			setAllowBouncing(m_originalAllowBounce);
			return false;
		}
	}
	else
	{
		bounceForce->zero();
		return false;
	}
}

//-------------------------------------------------------------------------------------------------
inline Bool isVerySmall3D(const Coord3D& v)
{
	const Real THRESH = 0.01f;
	return (WWMath::Fabs(v.x) < THRESH && WWMath::Fabs(v.y) < THRESH && WWMath::Fabs(v.z) < THRESH);
}

//-------------------------------------------------------------------------------------------------
inline Bool isZero3D(const Coord3D& v)
{
	return v.x == 0.0f && v.y == 0.0f && v.z == 0.0f;
}

//-------------------------------------------------------------------------------------------------
void PhysicsBehavior::setPitchRate(Real pitch)
{
	m_pitchRate = pitch;
	setFlag(HAS_PITCHROLLYAW, (m_pitchRate != 0.0f || m_rollRate != 0.0f || m_yawRate != 0.0f));
}

//-------------------------------------------------------------------------------------------------
void PhysicsBehavior::setRollRate(Real roll)
{
	m_rollRate = roll;
	setFlag(HAS_PITCHROLLYAW, (m_pitchRate != 0.0f || m_rollRate != 0.0f || m_yawRate != 0.0f));
}

//-------------------------------------------------------------------------------------------------
void PhysicsBehavior::setYawRate(Real yaw)
{
	m_yawRate = yaw;
	setFlag(HAS_PITCHROLLYAW, (m_pitchRate != 0.0f || m_rollRate != 0.0f || m_yawRate != 0.0f));
}

//-------------------------------------------------------------------------------------------------
void PhysicsBehavior::applyYPRDamping(Real factor)
{
	m_pitchRate *= factor;
	m_rollRate *= factor;
	m_yawRate *= factor;
	setFlag(HAS_PITCHROLLYAW, (m_pitchRate != 0.0f || m_rollRate != 0.0f || m_yawRate != 0.0f));
}

//-------------------------------------------------------------------------------------------------
void PhysicsBehavior::setBounceSound(const AudioEventRTS* bounceSound)
{
	if (bounceSound)
	{
		if (m_bounceSound == nullptr)
			m_bounceSound.Assign_No_Add_Ref(newInstance(DynamicAudioEventRTS));

		*m_bounceSound = *bounceSound;
	}
	else
	{
		m_bounceSound.Clear();
	}
}

//-------------------------------------------------------------------------------------------------
/**
 * Basic rigid body physics using an Euler integrator.
 * @todo Currently, only translations are integrated. Rotations should also be integrated. (MSB)
 */

DECLARE_PERF_TIMER(PhysicsBehavior)
UpdateSleepTime PhysicsBehavior::update()
{
	USE_PERF_TIMER(PhysicsBehavior)

	Object*														obj = getObject();
	const PhysicsBehaviorModuleData*	d = getPhysicsBehaviorModuleData();
	Bool															airborneAtStart = obj->isAboveTerrain();
	Real															activeVelZ = 0;
	Coord3D														bounceForce;
	Bool															gotBounceForce = false;

	DEBUG_ASSERTCRASH(!getFlag(IS_IN_UPDATE), ("impossible"));
	setFlag(IS_IN_UPDATE, true);

	if (!getFlag(UPDATE_EVER_RUN))
	{
		// set the flag so that we don't get bogus "collisions" on the first frame.
		setFlag(WAS_AIRBORNE_LAST_FRAME, airborneAtStart);
	}

	Coord3D prevPos = *obj->getPosition();
	m_prevAccel = m_accel;

	if (!obj->isDisabledByType(DISABLED_HELD))
	{
		Matrix3D mtx = *obj->getTransformMatrix();

		applyGravitationalForces();
		applyFrictionalForces();

		// integrate acceleration into velocity
		m_vel.x += m_accel.x;
		m_vel.y += m_accel.y;
		m_vel.z += m_accel.z;

		// when vel gets tiny, just clamp to zero
		const Real THRESH = 0.001f;
		if (WWMath::Fabsf(m_vel.x) < THRESH) m_vel.x = 0.0f;
		if (WWMath::Fabsf(m_vel.y) < THRESH) m_vel.y = 0.0f;
		if (WWMath::Fabsf(m_vel.z) < THRESH) m_vel.z = 0.0f;

		m_velMag = INVALID_VEL_MAG;

		Real oldPosZ = mtx.Get_Z_Translation();

		// integrate velocity into position
		if (obj->testStatus(OBJECT_STATUS_BRAKING))
		{
			// Don't update position if the locomotor is braking.
			if (!obj->isKindOf(KINDOF_PROJECTILE))
			{
				// Things other than projectiles don't cheat in z.  jba.
				mtx.Adjust_Z_Translation(m_vel.z);
			}
		}
		else
		{
			mtx.Adjust_X_Translation(m_vel.x);
			mtx.Adjust_Y_Translation(m_vel.y);
			mtx.Adjust_Z_Translation(m_vel.z);
		}

		if (_isnan(mtx.Get_X_Translation()) || _isnan(mtx.Get_Y_Translation()) ||
			_isnan(mtx.Get_Z_Translation())) {
			DEBUG_CRASH(("Object position is NAN, deleting."));
			TheGameLogic->destroyObject(obj);
		}

		// Check when to clear the stunned status
		if (getIsStunned())
		{
			if ( (WWMath::Fabs(m_vel.x) < STUN_RELIEF_EPSILON &&
				    WWMath::Fabs(m_vel.y) < STUN_RELIEF_EPSILON &&
				    WWMath::Fabs(m_vel.z) < STUN_RELIEF_EPSILON) ||
           obj->isSignificantlyAboveTerrain() == FALSE )
			{
				setStunned(false);
				getObject()->clearModelConditionState(MODELCONDITION_STUNNED);
			}

    }

		if (getFlag(HAS_PITCHROLLYAW))
		{

			/*
				You may be tempted to do something like this:

					Real rollAngle = -mtx.Get_X_Rotation();
					Real pitchAngle = mtx.Get_Y_Rotation();
					Real yawAngle = mtx.Get_Z_Rotation();
					// do stuff to angles, then rebuild the mtx with 'em

				You must resist this temptation, because your code will be wrong!

				The problem is that you can't use these calls to later reconstruct
				the matrix... because doing such a thing is highly order-dependent,
				and furthermore, you'd have to use Euler angles (Not the Get_?_Rotation
				calls) to be able to reconstruct 'em, and that's too slow to do for
				every object every frame.

				The one exception is that it is OK to use Get_Z_Rotation() to get
				the yaw angle.
			*/

			// only update the position if we are not HELD
			// (otherwise, slowdeath sinking into ground won't work)
			Real yawRateToUse = m_yawRate * d->m_pitchRollYawFactor;
			Real pitchRateToUse = m_pitchRate * d->m_pitchRollYawFactor;
			Real rollRateToUse = m_rollRate * d->m_pitchRollYawFactor;

			// With a center of mass listing, pitchRate needs to dampen towards straight down/straight up
			Real offset = getCenterOfMassOffset();

			// Magnitude sets initial rate, here we care about sign
			if (offset != 0.0f)
			{
				Vector3 xvec = mtx.Get_X_Vector();
				Real xy = WWMath::Sqrtf(sqr(xvec.X) + sqr(xvec.Y));
				Real pitchAngle = WWMath::Atan2(xvec.Z, xy);
				Real remainingAngle = (offset > 0) ? ((PI/2) - pitchAngle) : (-(PI/2) + pitchAngle);
				Real s = Sin(remainingAngle);
				pitchRateToUse *= s;
			}

			// update rotation
			/// @todo Rotation should use torques, and integrate just like forces (MSB)
			// note, we DON'T want to Pre-rotate (either inplace or not),
			// since we want to add our mods to the existing matrix.
			mtx.Rotate_X(rollRateToUse);
			mtx.Rotate_Y(pitchRateToUse);
			mtx.Rotate_Z(yawRateToUse);
		}

		// do not allow object to pass through the ground
		Real groundZ = TheTerrainLogic->getLayerHeight(mtx.Get_X_Translation(), mtx.Get_Y_Translation(), obj->getLayer());
		if( obj->getStatusBits().test( OBJECT_STATUS_DECK_HEIGHT_OFFSET ) )
		{
			groundZ += obj->getCarrierDeckHeight();
		}
		gotBounceForce = handleBounce(oldPosZ, mtx.Get_Z_Translation(), groundZ, &bounceForce);

		// remember our z-vel prior to doing ground-slam adjustment
		activeVelZ = m_vel.z;
		if (mtx.Get_Z_Translation() <= groundZ)
		{
			// Note - when vehicles are going down a slope, they will maintain a small negative
			// z velocity as they go down.  So don't slam it to 0 if they aren't slamming into the
			// ground.
			Real dz = groundZ - mtx.Get_Z_Translation();  // Our excess z velocity.
			m_vel.z += dz;							// Remove the excess z velocity.
			if (m_vel.z > 0.0f)
				m_vel.z = 0.0f;

			m_velMag = INVALID_VEL_MAG;

			mtx.Set_Z_Translation(groundZ);

			// this flag is ALWAYS cleared once we hit the ground.
			setFlag(ALLOW_TO_FALL, false);

			// When a stunned object hits the ground the first time, change it's model state from stunned flailing to just stunned.
			if (getFlag(IS_STUNNED))
			{
				obj->clearModelConditionState(MODELCONDITION_STUNNED_FLAILING);
				obj->setModelConditionState(MODELCONDITION_STUNNED);
			}
		}
		else if (mtx.Get_Z_Translation() > groundZ)
		{
			if (getFlag(IS_IN_FREEFALL))
			{
				obj->setDisabled(DISABLED_FREEFALL);
				obj->setModelConditionState(MODELCONDITION_FREEFALL);
			}
			else if (getFlag(STICK_TO_GROUND) && !getFlag(ALLOW_TO_FALL))
			{
				mtx.Set_Z_Translation(groundZ);
			}
		}

		if (gotBounceForce)
		{
			// Right the object after the bounce since the pitch and roll may have been affected
			Real yawAngle = getObject()->getTransformMatrix()->Get_Z_Rotation();
			setAngles(yawAngle, 0.0f, 0.0f);

			// Set the translation of the after bounce matrix to the one calculated above
			Matrix3D afterBounceMatrix = *getObject()->getTransformMatrix();
			afterBounceMatrix.Set_Translation(mtx.Get_Translation());

			// Set the result of the after bounce matrix as the object's final matrix
			obj->setTransformMatrix(&afterBounceMatrix);
		}
		else
		{
			obj->setTransformMatrix(&mtx);
		}
	}

	// reset the acceleration for accumulation next frame
	m_accel.zero();

	// clear overlap object, which will be set by PhysicsCollide later
	m_previousOverlap = m_currentOverlap;
	m_currentOverlap = INVALID_ID;

	if (gotBounceForce && getFlag(ALLOW_BOUNCE))
	{
		applyForce(&bounceForce);
	}

	Bool airborneAtEnd = obj->isAboveTerrain();

	// it's not good enough to check for airborne being different between
	// the start and end of this func... we have to compare since last frame,
	// since (if we're held by a parachute, for instance) we might have been
	// moved by other bits of code!
	if (getFlag(WAS_AIRBORNE_LAST_FRAME) && !airborneAtEnd && !getFlag(IMMUNE_TO_FALLING_DAMAGE))
	{
		doBounceSound(prevPos);

		// the normal always points straight down, though we could
		// get the alignWithTerrain() normal if it proves interesting
		Coord3D normal;
		normal.x = normal.y = 0.0f;
		normal.z = -1.0f;
		obj->onCollide(nullptr, obj->getPosition(), &normal);

		//
		// don't bother trying to remember how far we've fallen; instead,
		// we back-calc it from our speed & gravity... v = WWMath::Sqrt(2*g*h).
		// (note that m_minFallSpeedForDamage is always POSITIVE.)
		//
		// also note: since projectiles are immune to falling damage, don't
		// even bother doing this check here.
		//
		Real netSpeed = -activeVelZ - d->m_minFallSpeedForDamage;

		if (netSpeed > 0.0f && m_pui == nullptr)
		{
			// only apply force if it's a pretty steep fall, so that things
			// going down hills don't injure themselves (unless the hill is really steep)
			const Real MIN_ANGLE_TAN = 3.0f;	//	roughly 71 degrees
			const Real TINY_DELTA = 0.01f;
			if ((WWMath::Fabs(m_vel.x) <= TINY_DELTA || WWMath::Fabs(activeVelZ / m_vel.x) >= MIN_ANGLE_TAN) &&
				(WWMath::Fabs(m_vel.y) <= TINY_DELTA || WWMath::Fabs(activeVelZ / m_vel.y) >= MIN_ANGLE_TAN))
			{
				Real damageAmt = netSpeed * getMass() * d->m_fallHeightDamageFactor;

				DamageInfo damageInfo;
				damageInfo.in.m_damageType = DAMAGE_FALLING;
				damageInfo.in.m_deathType = DEATH_SPLATTED;
				damageInfo.in.m_sourceID = obj->getID();
				damageInfo.in.m_amount = damageAmt;
        damageInfo.in.m_shockWaveAmount = 0.0f;
				obj->attemptDamage( &damageInfo );
				//DEBUG_LOG(("Dealing %f (%f %f) points of falling damage to %s!",damageAmt,damageInfo.out.m_actualDamageDealt, damageInfo.out.m_actualDamageClipped,obj->getTemplate()->getName().str()));

				// if this killed us, add SPLATTED to get a cool death.
				if (obj->isEffectivelyDead())
				{
					obj->setModelConditionState(MODELCONDITION_SPLATTED);
				}
			}
		}
	}

	if (!airborneAtEnd)
	{
		// just in case.
		setFlag(IS_IN_FREEFALL, false);
		if (obj->isDisabledByType(DISABLED_FREEFALL))
			obj->clearDisabled(DISABLED_FREEFALL);
		obj->clearModelConditionState(MODELCONDITION_FREEFALL);
	}


	// If we are effectively dead, we shouldn't recall kill.
	if (d->m_killWhenRestingOnGround && !airborneAtEnd && isVerySmall3D(m_vel))
	{
		if( !obj->isKindOf( KINDOF_DRONE ) || obj->isEffectivelyDead() || obj->isDisabledByType( DISABLED_UNMANNED ) )
		{
			//Must be one of the following cases in order to splat:
			//1) Not a drone
			//2) Dead drone
			//3) Unmanned drone
			obj->kill();
		}
	}

	setFlag(UPDATE_EVER_RUN, true);
	setFlag(WAS_AIRBORNE_LAST_FRAME, airborneAtEnd);

	setFlag(IS_IN_UPDATE, false);
	return calcSleepTime();
}

//-------------------------------------------------------------------------------------------------
UpdateSleepTime PhysicsBehavior::calcSleepTime() const
{
#ifdef SLEEPY_PHYSICS
	if (isZero3D(m_vel) &&
			isZero3D(m_accel) &&
			!getFlag(HAS_PITCHROLLYAW) &&
			!isMotive() &&
			(getObject()->getLayer() == LAYER_GROUND && !getObject()->isAboveTerrain()) &&
			getCurrentOverlap() == INVALID_ID &&
			getPreviousOverlap() == INVALID_ID &&
			getFlag(UPDATE_EVER_RUN))
	{
		return UPDATE_SLEEP_FOREVER;
	}
	else
#endif
	{
		return UPDATE_SLEEP_NONE;
	}
}

//-------------------------------------------------------------------------------------------------
Real PhysicsBehavior::getVelocityMagnitude() const
{
	if (m_velMag == INVALID_VEL_MAG)
	{
		m_velMag = (Real)WWMath::Sqrtf( sqr(m_vel.x) + sqr(m_vel.y) + sqr(m_vel.z) );
	}
	return m_velMag;
}

//-------------------------------------------------------------------------------------------------
/**
 * Return the current velocity magnitude in the forward direction.
 * If velocity is opposite facing vector, the returned value will be negative.
 */
Real PhysicsBehavior::getForwardSpeed2D() const
{
	const Coord3D *dir = getObject()->getUnitDirectionVector2D();
	Real vx = m_vel.x * dir->x;
	Real vy = m_vel.y * dir->y;
	Real dot = vx + vy;

	if (useLegacyForwardSpeed())
	{
		Real speed = (Real)WWMath::Sqrtf( vx*vx + vy*vy );
		if (dot >= 0.0f)
			return speed;
		return -speed;
	}

	// TheSuperHackers @bugfix xezon 30/07/2026 Now returns the dot product instead of +-sqrtf(vx*vx+vy*vy).
	// The retail formula understates the forward speed by up to 1/sqrt(2) on diagonal headings, which made
	// the Locomotor overshoot its goal speed there. The dot product is the true projection of the velocity
	// onto the facing vector, so this now reports real distance per logic frame and can be used for distance
	// and time calculations. The speeds the Locomotor commands are compensated to match, in LocomotorTemplate.
	return dot;
}

//-------------------------------------------------------------------------------------------------
/**
 * Return the current velocity magnitude in the forward direction.
 * If velocity is opposite facing vector, the returned value will be negative.
 */
Real PhysicsBehavior::getForwardSpeed3D() const
{
	Vector3 dir = getObject()->getTransformMatrix()->Get_X_Vector();
	Real vx = m_vel.x * dir.X;
	Real vy = m_vel.y * dir.Y;
	Real vz = m_vel.z * dir.Z;
	Real dot = vx + vy + vz;

	if (useLegacyForwardSpeed())
	{
		Real speed = (Real)WWMath::Sqrtf( vx*vx + vy*vy + vz*vz );
		if (dot >= 0.0f)
			return speed;
		return -speed;
	}

	// TheSuperHackers @bugfix xezon 30/07/2026 Now returns the dot product instead of
	// +-sqrtf(vx*vx+vy*vy+vz*vz). See getForwardSpeed2D for the rationale.
	return dot;
}

#if USE_RETAIL_PHYSICS_FORWARD_SPEED_DISCREPANCY_IN_CINEMATICS()
//-------------------------------------------------------------------------------------------------
Bool PhysicsBehavior::useLegacyForwardSpeedInCinematics()
{
	// TheSuperHackers @info The retail speeds are kept for the duration of a cinematic, because
	// legacy missions time their cinematics against them.
	return TheScriptEngine->isLetterBoxActive();
}
#endif

//-------------------------------------------------------------------------------------------------
Bool PhysicsBehavior::isCurrentlyOverlapped(Object *obj) const
{
	return obj != nullptr && obj->getID() == m_currentOverlap;
}

//-------------------------------------------------------------------------------------------------
Bool PhysicsBehavior::wasPreviouslyOverlapped(Object *obj) const
{
	return obj != nullptr && obj->getID() == m_previousOverlap;
}

//-------------------------------------------------------------------------------------------------
void PhysicsBehavior::scrubVelocityZ( Real desiredVelocity )
{
	if (WWMath::Fabs(desiredVelocity) < 0.001f)
	{
		m_vel.z = 0;
	}
	else
	{
		if ((desiredVelocity < 0 && m_vel.z < desiredVelocity) || (desiredVelocity > 0 && m_vel.z > desiredVelocity))
		{
			m_vel.z = desiredVelocity;
		}
	}
	m_velMag = INVALID_VEL_MAG;
}

//-------------------------------------------------------------------------------------------------
void PhysicsBehavior::scrubVelocity2D( Real desiredVelocity )
{
	if (desiredVelocity < 0.001f)
	{
		m_vel.x = 0;
		m_vel.y = 0;
	}
	else
	{
		Real curVelocity = WWMath::Sqrtf(m_vel.x*m_vel.x + m_vel.y*m_vel.y);
		if (desiredVelocity > curVelocity)
		{
			return;
		}
		desiredVelocity /= curVelocity;
		m_vel.x *= desiredVelocity;
		m_vel.y *= desiredVelocity;
	}
	m_velMag = INVALID_VEL_MAG;
}

//-------------------------------------------------------------------------------------------------
void PhysicsBehavior::addOverlap(Object *obj)
{
	if (obj && !isCurrentlyOverlapped(obj))
	{
		m_currentOverlap = obj->getID();
	}
}

//-------------------------------------------------------------------------------------------------
void PhysicsBehavior::transferVelocityTo(PhysicsBehavior* that) const
{
	if (that != nullptr)
	{
		that->m_vel.add(m_vel);
		that->m_velMag = INVALID_VEL_MAG;
	}
}

//-------------------------------------------------------------------------------------------------
void PhysicsBehavior::addVelocityTo( const Coord3D *vel)
{
	if (vel != nullptr)
		m_vel.add( *vel );
}

//-------------------------------------------------------------------------------------------------
void PhysicsBehavior::setAngles( Real yaw, Real pitch, Real roll )
{
	const Coord3D* pos = getObject()->getPosition();

	Matrix3D xfrm;
	xfrm.Make_Identity();
	xfrm.Translate( pos->x, pos->y, pos->z );
	// here we DO want to use in-place-etc, cuz we're not adding to any existing rot/etc
	xfrm.In_Place_Pre_Rotate_X( -roll );
	xfrm.In_Place_Pre_Rotate_Y( pitch );
	xfrm.In_Place_Pre_Rotate_Z( yaw );
	getObject()->setTransformMatrix( &xfrm );
}

//-------------------------------------------------------------------------------------------------
Real PhysicsBehavior::getMass() const
{
	Real mass = m_mass;
	ContainModuleInterface* contain = getObject()->getContain();
	if (contain)
		mass += contain->getContainedItemsMass();
	return mass;
}

//-----------------------------------------------------------------------------
inline Real calcDistSqr(const Coord3D& a, const Coord3D& b)
{
	return sqr(a.x - b.x) + sqr(a.y - b.y) + sqr(a.z - b.z);
}

//-------------------------------------------------------------------------------------------------
void PhysicsBehavior::doBounceSound(const Coord3D& prevPos)
{
	if (!m_bounceSound)
		return;

	const Real NORMAL_VEL_Z	= 0.25f;
	const Real NORMAL_MASS	= 50.0f;

	// get the per-unit sound for the collision which was stuffed in on Object creation.
	AudioEventRTS collisionSound = *m_bounceSound.Peek();

//Real vel = fabs(getVelocity()->z);
// can't use velocity, because it's already been updated this frame, and will be zero... (srj)
	Real vel = WWMath::Fabs(prevPos.z - getObject()->getPosition()->z);

	Real mass = WWMath::Fabs(getMass());
	if (vel > NORMAL_VEL_Z) {
		vel = NORMAL_VEL_Z;
	}

	if (mass > NORMAL_MASS) {
		mass = NORMAL_MASS;
	}

	if (vel < 0) {
		vel = 0;
	}

	if (mass < 0) {
		mass = 0;
	}

#ifdef FIX_AUDIO
	Real volAdjust = NormalizeToRange(MuLaw(vel, NORMAL_VEL_Z, 500), -1, 1, 0.25, 1.0);
	volAdjust *= NormalizeToRange(MuLaw(mass, NORMAL_MASS, 500), -1, 1, 0.25, 1.0);
	collisionSound.setVolume(volAdjust);
#endif
	collisionSound.setObjectID(getObject()->getID());
	TheAudio->addAudioEvent(&collisionSound);
}

//-------------------------------------------------------------------------------------------------
/**
 * Resolve the collision between getObject() and other by computing
 * the amount the objects have overlapd, and applying proportional forces to
 * push them apart.
 * Note that this call only applies forces to our object, not to "other".  Since the
 * forces should be equal and opposite, this could be optimized.
 * @todo Make this work properly for non-cylindrical objects (MSB)
 * @todo Physics collision resolution is 2D - should it be 3D? (MSB)
 */
//DECLARE_PERF_TIMER(PhysicsBehavioronCollide)
void PhysicsBehavior::onCollide( Object *other, const Coord3D *loc, const Coord3D *normal )
{
	//USE_PERF_TIMER(PhysicsBehavioronCollide)
	if (m_pui != nullptr)
	{
		// projectiles always get a chance to handle their own collisions, and not go thru here
		if (m_pui->projectileHandleCollision(other))
			return;
	}

	Object *obj = getObject();

	Object* objContainedBy = obj->getContainedBy();

	// Note that other == null means "collide with ground"
	if (other == nullptr)
	{
		// if we are in a container, tell the container we collided with the ground.
		// (handy for parachutes.)
		if (objContainedBy)
		{
			objContainedBy->onCollide(other, loc, normal);
		}
		return;
	}

	// if we are containing this object, ignore collisions with it.
	Object* otherContainedBy = other->getContainedBy();
	if (otherContainedBy == obj || objContainedBy == other)
	{
		return;
	}

	// if we're both on parachutes, we never collide with each other.
	/// @todo srj -- ugh, an awful hack for paradrop problems. fix someday.
	if (obj->testStatus(OBJECT_STATUS_PARACHUTING) && other->testStatus(OBJECT_STATUS_PARACHUTING))
	{
		return;
	}

	// ignore collisions with our "ignore" thingy, if any (and vice versa)
	AIUpdateInterface* ai = obj->getAIUpdateInterface();
	if (ai != nullptr  && ai->getIgnoredObstacleID() == other->getID())
	{
/// @todo srj -- what the hell is this code doing here? ack!
		//Before we return, check for a very special case of an infantry colliding with an unmanned vehicle.
		//If this is the case, it'll become its new pilot!
		if( obj->isKindOf( KINDOF_INFANTRY ) && other->isDisabledByType( DISABLED_UNMANNED ) )
		{
			//This is in fact the case, and we are doing it here because it applies to all infantry in any unmanned vehicle.
			//This could be done via a special/new module, but doing it here doesn't require a new module update to every infantry.
			other->clearDisabled( DISABLED_UNMANNED );

			//We need to be able to test whether an object on a team has been captured, so set here that this object
			//was captured.
			other->setCaptured(true);

			other->defect( obj->getTeam(), 0 );
			//other->setTeam( obj->getTeam() );

			//In order to make things easier for the designers, we are going to transfer the name
			//of the infantry to the vehicle... so the designer can control the vehicle with their scripts.
			TheScriptEngine->transferObjectName( obj->getName(), other );

			TheGameLogic->destroyObject( obj );
		}
		return;
	}

	AIUpdateInterface* aiOther = other->getAIUpdateInterface();
	if (aiOther != nullptr  && aiOther->getIgnoredObstacleID() == obj->getID())
	{
		return;
	}

	if (isIgnoringCollisionsWith(other->getID()))
	{
		return;
	}

	Bool immobile = obj->isKindOf( KINDOF_IMMOBILE );
	Bool otherImmobile = other->isKindOf( KINDOF_IMMOBILE );

	PhysicsBehavior* otherPhysics = other->getPhysics();
	if (otherPhysics)
	{
		if (otherPhysics->isIgnoringCollisionsWith(obj->getID()))
		{
			return;
		}
	}
	else
	{
		// if the object we have collided with has no physics, it is insubstantial
		// (exception: if it's immobile.)
		if (!otherImmobile)
		{
			return;
		}
	}


	if (checkForOverlapCollision(other))
	{
		// we should overlap them, rather than bounce them, so punt here.
		// (note that we do this prior to checking for Physics update,
		// because Overlap doesn't require 'other' to have Physics...)
		return;
	}

	Real mass = getMass();
	if (immobile)
		mass = 999999.0f;

	if (ai)
	{
		// AI objects move under their own initiative, not by getting bounced.  jba.
		// unless they are dead & colliding with something immobile. (srj)
		// or parachuting and colliding with something immobile. (srj)
		if (!((obj->isEffectivelyDead() || obj->testStatus(OBJECT_STATUS_PARACHUTING)) && otherImmobile))
		{
			Bool doForce = ai->processCollision(this, other);
			if (!doForce)
				return;
		}

	}

	Coord3D usCenter, themCenter;
	obj->getGeometryInfo().getCenterPosition(*obj->getPosition(), usCenter);
	other->getGeometryInfo().getCenterPosition(*other->getPosition(), themCenter);

	Coord3D delta;
	delta.x = themCenter.x - usCenter.x;
	delta.y = themCenter.y - usCenter.y;
	delta.z = themCenter.z - usCenter.z;

	Real distSqr, usRadius, themRadius;
	if (obj->isAboveTerrain())
	{
		// do 3d testing.
		usRadius = obj->getGeometryInfo().getBoundingSphereRadius();
		themRadius = other->getGeometryInfo().getBoundingSphereRadius();
		distSqr = sqr(delta.x) + sqr(delta.y) + sqr(delta.z);
	}
	else
	{
		// do 2d testing.
		usRadius = obj->getGeometryInfo().getBoundingCircleRadius();
		themRadius = other->getGeometryInfo().getBoundingCircleRadius();
		distSqr = sqr(delta.x) + sqr(delta.y);
		delta.z = 0;
	}

	if (distSqr > sqr(usRadius + themRadius))
	{
		// We don't overlap at all.  How did we get here?
		return;
	}


	m_lastCollidee = other->getID();

	Real dist = WWMath::Sqrtf(distSqr);
	Real overlap = usRadius + themRadius - dist;

	// if objects are coincident, dist is zero, so force would be infinite -- clearly
	// not what we want. so just cap it here for now. (srj)
	if (dist < 1.0f)
		dist = 1.0f;

	if (getAllowCollideForce())
	{
		Real factor;
		Coord3D force;
		if (otherImmobile && !obj->isDestroyed())
		{
			if (obj->testStatus(OBJECT_STATUS_PARACHUTING))
			{
				// don't let us intersect buildings. cheat. applying a force won't work
				// cuz we are usually braking. jam it.
				Object* objToBounce = obj;
				while (objToBounce->getContainedBy() != nullptr)
					objToBounce = objToBounce->getContainedBy();

				Real bounceOutDist = usRadius * 0.1f;
				Coord3D tmp = *objToBounce->getPosition();
				tmp.x -= bounceOutDist * delta.x / dist;
				tmp.y -= bounceOutDist * delta.y / dist;
				objToBounce->setPosition(&tmp);

				objToBounce->getPhysics()->scrubVelocity2D(0);
				return;
			}

			// if other is immobile, we must apply enough force to at least stop our motion,
			// and preferably negate it, otherwise we will pass thru the object. so cheat,
			// and act like we're bouncing off the ground.

			const Real MIN_STIFF = 0.01f;
			const Real MAX_STIFF = 0.99f;
			Real stiffness = TheGlobalData->m_structureStiffness;
			if (stiffness < MIN_STIFF) stiffness = MIN_STIFF;
			if (stiffness > MAX_STIFF) stiffness = MAX_STIFF;
			// huh huh, he said "stiff"

			Real mag = getVelocityMagnitude();
			const Real MINBOUNCESPEED = 1.0f/(LOGICFRAMES_PER_SECOND*5.0f);
			if (mag < MINBOUNCESPEED)
				mag = MINBOUNCESPEED;
			factor = -mag * getMass() * stiffness;

			// if we are moving down, we may want to blow ourselves into smithereens....
			if (delta.z < 0.0f &&
					obj->getPosition()->z >= TheGlobalData->m_defaultStructureRubbleHeight)
			{
				if (other->isKindOf(KINDOF_STRUCTURE))
				{
					// fall into a building. if a vehicle, blow up. then destroy ourself (not die), regardless.
					if (obj->isKindOf(KINDOF_VEHICLE))
					{
						TheWeaponStore->createAndFireTempWeapon(getPhysicsBehaviorModuleData()->m_vehicleCrashesIntoBuildingWeaponTemplate, obj, obj->getPosition());
					}
					TheGameLogic->destroyObject(obj);
					return;
				}
				else
				{
					// fall into a nonbuilding -- whatever. if we're a vehicle, quietly do a little damage.
					if (obj->isKindOf(KINDOF_VEHICLE))
					{
						TheWeaponStore->createAndFireTempWeapon(getPhysicsBehaviorModuleData()->m_vehicleCrashesIntoNonBuildingWeaponTemplate, obj, obj->getPosition());
					}
				}
			}

			// nuke the velocity. why? very simple: we want to ignore the previous vel in favor of
			// this. in theory, we could be clever and calculate the right force to apply to achieve this,
			// but then if we were still colliding next frame, we'd get a sudden 'acceleration' of bounce
			// that would look freakish. so cheat.
			m_vel.x = 0;
			m_vel.y = 0;
			m_vel.z = 0;
			m_velMag = INVALID_VEL_MAG;
		}
		else
		{
			if (overlap > 5.0f)
				overlap = 5.0f;
			factor = -overlap;
		}

		force.x = factor * delta.x / dist;
		force.y = factor * delta.y / dist;
		force.z = factor * delta.z / dist;	// will be zero for 2d case.
		DEBUG_ASSERTCRASH(!(_isnan(force.x) || _isnan(force.y) || _isnan(force.z)), ("PhysicsBehavior::onCollide force NAN!"));

		applyForce( &force );
	}
}

//-------------------------------------------------------------------------------------------------
static Bool perpsLogicallyEqual( Real perpOne, Real perpTwo )
{
	// Equality with a wiggle fudge.
  const Real PERP_RANGE = 0.15f;
	return WWMath::Fabs( perpOne - perpTwo ) <= PERP_RANGE;
}

//-------------------------------------------------------------------------------------------------
/** Do the collision
 *  Cases:
 *  * The crusheeOther is !OVERLAPPABLE or !CRUSHABLE.								Result: return false
 *  * The crusherMe is !CRUSHER.																			Result: return false
 *  * crusher is CRUSHER, crushee is CRUSHABLE but allied.						Result: return false
 *  * crusher is CRUSHER, crushee is OVERLAPPABLE but not CRUSHABLE.  Result: return false
 *  * crushee has no PhysicsBehavior.																	Result: return false
 *  * crusher is CRUSHER, crushee is CRUSHABLE, but is too hard.			Result: return false
 *  * crusher is CRUSHER, crushee is CRUSHABLE, hardness ok.					Result: crush, return true
 *
 * Return true if we want to skip having physics push us apart // LORENZEN
 */
//-------------------------------------------------------------------------------------------------
Bool PhysicsBehavior::checkForOverlapCollision(Object *other)
{
	//This is the most Supreme Truth... that unless I am moving right now, I may not crush anything!
	if ( isVerySmall3D( *getVelocity() ) )
		return false;


  Object* crusherMe = getObject();
  Object* crusheeOther = other;

	//Determine if we can crush the other object.
	Bool selfCrushingOther = crusherMe->canCrushOrSquish( crusheeOther, TEST_CRUSH_ONLY );
	Bool selfBeingCrushed = crusheeOther->canCrushOrSquish( crusherMe, TEST_CRUSH_ONLY );

	if( selfCrushingOther && selfBeingCrushed )
	{
		//Is it possible to crush and be crushed at the same time?
		DEBUG_CRASH( ("%s (Crusher:%d, Crushable:%d) is attempting to crush %s (Crusher:%d, Crushable:%d) but it is reciprocating -- shouldn't be possible!",
			crusherMe->getTemplate()->getName().str(), crusherMe->getCrusherLevel(), crusherMe->getCrushableLevel(),
			crusheeOther->getTemplate()->getName().str(), crusheeOther->getCrusherLevel(), crusheeOther->getCrushableLevel() ) );
		return false;
	}

	// if we are being crushed, then skip all this and return true;
	// this allows us to NOT react in the normal way and just be passive to the overlap...
	if( selfBeingCrushed )
	{
		return true;
	}

	// grab physics modules if there
	PhysicsBehavior *crusherPhysics = this;
	if( crusherPhysics == nullptr )
	{
		return false;
	}

	if( !selfCrushingOther )
	{
		return false;
	}

	// ok, add this to our list-of-overlapped-things.
	crusherPhysics->addOverlap(crusheeOther);
	if (!crusherPhysics->wasPreviouslyOverlapped(crusheeOther))
	{
		DamageInfo damageInfo;
		damageInfo.in.m_damageType = DAMAGE_CRUSH;
		damageInfo.in.m_deathType = DEATH_CRUSHED;
		damageInfo.in.m_sourceID = crusherMe->getID();
		damageInfo.in.m_amount = 0.0f;	// yes, that's right -- we don't want to do damage, just to trigger the minor DamageFX, if any
		crusheeOther->attemptDamage( &damageInfo );
	}

  const Coord3D *crusheePos = crusheeOther->getPosition();
  const Coord3D *crusherPos = crusherMe->getPosition();

	BodyModuleInterface* crusheeBody = crusheeOther->getBodyModule();
	Bool frontCrushed = crusheeBody->getFrontCrushed();
	Bool backCrushed = crusheeBody->getBackCrushed();
	if( !(frontCrushed && backCrushed) )
	{
		Bool crushIt = FALSE;

		const Coord3D *dir = crusherMe->getUnitDirectionVector2D();
		const Coord3D *crusheeDir = crusheeOther->getUnitDirectionVector2D();
		Real crushPointOffsetDistance = crusheeOther->getGeometryInfo().getMajorRadius() / 2;

		Coord3D crushPointOffset;
		crushPointOffset.x = crusheeDir->x * crushPointOffsetDistance;
		crushPointOffset.y = crusheeDir->y * crushPointOffsetDistance;
		crushPointOffset.z = 0;

		Coord3D comparisonCoord;
		Real dx, dy;

		/// @todo GS To account for different sized crushers, this should be redone as a box or circle test, not a point
		// First decide which crush point has the shortest perp to our direction ray.
//		Real bestPerp = 9999;
		CrushEnum crushTarget = NO_CRUSH;
		if( frontCrushed || backCrushed )
		{
			// Degenerate case; there is only one point to consider.
			if( frontCrushed )
				crushTarget = BACK_END_CRUSH;
			else
				crushTarget = FRONT_END_CRUSH;
		}
		else
		{
			// Else, there are three points.
			Real frontPerpLength, backPerpLength, centerPerpLength;
			Coord3D frontVector, backVector, centerVector;
			{
				comparisonCoord = *crusheePos;
				comparisonCoord.x += crushPointOffset.x;
				comparisonCoord.y += crushPointOffset.y;
				frontVector = comparisonCoord;
				frontVector.x -= crusherPos->x;
				frontVector.y -= crusherPos->y; //vector from me to the front crush point
				frontVector.z = 0;

				Real rayLength = frontVector.x * dir->x + frontVector.y * dir->y;
				Coord3D dirVector;
				dirVector.x = rayLength * dir->x;
				dirVector.y = rayLength * dir->y; //vector from me to point of perp along direction ray
				dirVector.z = 0;

				Coord3D perpVector;
				perpVector.x = dirVector.x - frontVector.x;
				perpVector.y = dirVector.y - frontVector.y; //vector from the front point perp to my direction
				perpVector.z = 0;

				frontPerpLength = perpVector.length();
			}
			{
				comparisonCoord = *crusheePos;
				comparisonCoord.x -= crushPointOffset.x;
				comparisonCoord.y -= crushPointOffset.y;
				backVector = comparisonCoord;
				backVector.x -= crusherPos->x;
				backVector.y -= crusherPos->y; //vector from me to the front crush point
				backVector.z = 0;

				Real rayLength = backVector.x * dir->x + backVector.y * dir->y;
				Coord3D dirVector;
				dirVector.x = rayLength * dir->x;
				dirVector.y = rayLength * dir->y; //vector from me to point of perp along direction ray
				dirVector.z = 0;

				Coord3D perpVector;
				perpVector.x = dirVector.x - backVector.x;
				perpVector.y = dirVector.y - backVector.y; //vector from the front point perp to my direction
				perpVector.z = 0;

				backPerpLength = perpVector.length();
			}
			{
				comparisonCoord = *crusheePos;
				centerVector = comparisonCoord;
				centerVector.x -= crusherPos->x;
				centerVector.y -= crusherPos->y; //vector from me to the front crush point
				centerVector.z = 0;

				Real rayLength = centerVector.x * dir->x + centerVector.y * dir->y;
				Coord3D dirVector;
				dirVector.x = rayLength * dir->x;
				dirVector.y = rayLength * dir->y; //vector from me to point of perp along direction ray
				dirVector.z = 0;

				Coord3D perpVector;
				perpVector.x = dirVector.x - centerVector.x;
				perpVector.y = dirVector.y - centerVector.y; //vector from the front point perp to my direction
				perpVector.z = 0;

				centerPerpLength = perpVector.length();
			}

			// Now find the shortest.  Use the straightline distance to crush point as tie breaker
			if( (frontPerpLength <= centerPerpLength)  && (frontPerpLength <= backPerpLength) )
			{
				if( perpsLogicallyEqual(frontPerpLength, centerPerpLength) ||
					perpsLogicallyEqual(frontPerpLength, backPerpLength)
					)
				{
					Real frontVectorLength = frontVector.length();
					if( perpsLogicallyEqual(frontPerpLength, centerPerpLength) )
					{
						Real centerVectorLength = centerVector.length();
						if( frontVectorLength < centerVectorLength )
							crushTarget = FRONT_END_CRUSH;
						else
							crushTarget = TOTAL_CRUSH;
					}
					else if( perpsLogicallyEqual(frontPerpLength, backPerpLength) )
					{
						Real backVectorLength = backVector.length();
						if( frontVectorLength < backVectorLength )
							crushTarget = FRONT_END_CRUSH;
						else
							crushTarget = BACK_END_CRUSH;
					}
				}
				else
				{
					crushTarget = FRONT_END_CRUSH;
				}
			}
			else if( (backPerpLength <= centerPerpLength)  && (backPerpLength <= frontPerpLength) )
			{
				if( perpsLogicallyEqual(backPerpLength, centerPerpLength) ||
					perpsLogicallyEqual(backPerpLength, frontPerpLength)
					)
				{
					Real backVectorLength = backVector.length();
					if( perpsLogicallyEqual(backPerpLength, centerPerpLength) )
					{
						Real centerVectorLength = centerVector.length();
						if( backVectorLength < centerVectorLength )
							crushTarget = BACK_END_CRUSH;
						else
							crushTarget = TOTAL_CRUSH;
					}
					else if( perpsLogicallyEqual(backPerpLength, frontPerpLength) )
					{
						Real frontVectorLength = frontVector.length();
						if( backVectorLength < frontVectorLength )
							crushTarget = BACK_END_CRUSH;
						else
							crushTarget = FRONT_END_CRUSH;
					}
				}
				else
				{
					crushTarget = BACK_END_CRUSH;
				}
			}
			else // centerperp is shortest
			{
				if( perpsLogicallyEqual(centerPerpLength, backPerpLength) ||
					perpsLogicallyEqual(centerPerpLength, frontPerpLength)
					)
				{
					Real centerVectorLength = centerVector.length();
					if( perpsLogicallyEqual(centerPerpLength, frontPerpLength) )
					{
						Real frontVectorLength = frontVector.length();
						if( centerVectorLength < frontVectorLength )
							crushTarget = TOTAL_CRUSH;
						else
							crushTarget = FRONT_END_CRUSH;
					}
					else if( perpsLogicallyEqual(centerPerpLength, backPerpLength) )
					{
						Real backVectorLength = backVector.length();
						if( centerVectorLength < backVectorLength )
							crushTarget = TOTAL_CRUSH;
						else
							crushTarget = BACK_END_CRUSH;
					}
				}
				else
				{
					crushTarget = TOTAL_CRUSH;
				}
			}
		}

		Real distanceTooFarSquared = 2.25 * crushPointOffsetDistance * crushPointOffsetDistance;
		// And just because there is only one crush point left doesn't
		// mean we automatically get it. (1.5x)^2 means if we are outside the crushed
		//front point, we will not auto get the back point (only 3/8 car spread)

		// Then ask ourselves if we have passed the correct crush point (dot < 0).
		if( crushTarget == TOTAL_CRUSH )
		{
			// Check the middle crush point
			comparisonCoord = *crusheePos; //copy so can move to each crush point

			dx = comparisonCoord.x - crusherPos->x;
			dy = comparisonCoord.y - crusherPos->y;

			Real dot = dir->x * dx + dir->y * dy;
			Real distanceSquared = (dx * dx) + (dy * dy);

			if( (dot < 0)  &&  (distanceSquared < distanceTooFarSquared) )
			{
				// Past target point, but not too far in distance or angle.
				crushIt = TRUE;
			}
		}
		else if( crushTarget == FRONT_END_CRUSH )
		{
			// Check the front point.
			comparisonCoord = *crusheePos;
			comparisonCoord.x += crushPointOffset.x;
			comparisonCoord.y += crushPointOffset.y;

			dx = comparisonCoord.x - crusherPos->x;
			dy = comparisonCoord.y - crusherPos->y;

			Real dot = dir->x * dx + dir->y * dy;
			Real distanceSquared = (dx * dx) + (dy * dy);

			if( (dot < 0)  &&  (distanceSquared < distanceTooFarSquared) )
			{
				// Past target point, but not too far in distance or angle.
				crushIt = TRUE;
			}
		}
		else if( crushTarget == BACK_END_CRUSH )
		{
			// Check back point
			comparisonCoord = *crusheePos;
			comparisonCoord.x -= crushPointOffset.x;
			comparisonCoord.y -= crushPointOffset.y;

			dx = comparisonCoord.x - crusherPos->x;
			dy = comparisonCoord.y - crusherPos->y;

			Real dot = dir->x * dx + dir->y * dy;
			Real distanceSquared = (dx * dx) + (dy * dy);

			if( (dot < 0)  &&  (distanceSquared < distanceTooFarSquared) )
			{
				// Past target point, but not too far in distance or angle.
				crushIt = TRUE;
			}
		}

		if( crushIt )
		{
			// do a boat load of crush damage, and the onDie will handle cases like crushed car object
			DamageInfo damageInfo;
			damageInfo.in.m_damageType = DAMAGE_CRUSH;
			damageInfo.in.m_deathType = DEATH_CRUSHED;
			damageInfo.in.m_sourceID = crusherMe->getID();
			damageInfo.in.m_amount = HUGE_DAMAGE_AMOUNT;			// make sure they die
			crusheeOther->attemptDamage( &damageInfo );
		}

	}

	return true;
}

// ------------------------------------------------------------------------------------------------
/** Test whether unit needs to die because of being on illegal cell, upside down, outside legal bounds **/
// ------------------------------------------------------------------------------------------------
void PhysicsBehavior::testStunnedUnitForDestruction()
{
	// Only do test if unit is stunned
	if (!getFlag(IS_STUNNED))
		return;

	// Grab the object
	Object *obj = getObject();
	const Coord3D *pos = obj->getPosition();

	// If a stunned object is upside down when it hits the ground, kill it
	if(obj->getTransformMatrix()->Get_Z_Vector().Z < 0.0f)
	{
		obj->kill();
		return;
	}

	// Check if unit has exited playable area. If so, kill it
  if (obj->isOffMap())
	{
     obj->kill();
		 return;
  }

	// Check for being in cells that the unit has no locomotor for
	AIUpdateInterface *aiInt = obj->getAI();
	if (!aiInt) return;

	// Check for object being stuck on cliffs. If so kill it
	if (TheTerrainLogic->isCliffCell(pos->x, pos->y) && !aiInt->hasLocomotorForSurface(LOCOMOTORSURFACE_CLIFF))
	{
		obj->kill();
		return;
	}

	// Check for object being stuck on water. If so kill it
	if (TheTerrainLogic->isUnderwater(pos->x, pos->y) && !aiInt->hasLocomotorForSurface(LOCOMOTORSURFACE_WATER))
	{
		obj->kill();
		return;
	}
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void PhysicsBehavior::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void PhysicsBehavior::xfer( Xfer *xfer )
{

	// version
	const XferVersion currentVersion = 2;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// yaw rate
	xfer->xferReal( &m_yawRate );

	// roll rate
	xfer->xferReal( &m_rollRate );

	// pitch rate
	xfer->xferReal( &m_pitchRate );

	// we dont' need to mess with sound stuff
	// m_bounceSound <---- do nothing with this

	// accel
	xfer->xferCoord3D( &m_accel );

	// prev accel
	xfer->xferCoord3D( &m_prevAccel );

	// velocity
	xfer->xferCoord3D( &m_vel );

	// prevPos (now defunct)
	if (version < 2)
	{
		Coord3D tmp;
		tmp.zero();
		xfer->xferCoord3D( &tmp );
	}

	// turning
	xfer->xferUser( &m_turning, sizeof( PhysicsTurningType ) );

	// ignore collisions with
	xfer->xferObjectID( &m_ignoreCollisionsWith );

	// flags
	xfer->xferInt( &m_flags );

	// mass
	xfer->xferReal( &m_mass );

	// current overlap
	xfer->xferObjectID( &m_currentOverlap );

	// previous overlap
	xfer->xferObjectID( &m_previousOverlap );

	// motive force applied
	xfer->xferUnsignedInt( &m_motiveForceExpires );

	// extra bounciness
	xfer->xferReal( &m_extraBounciness );

	// extra friction
	xfer->xferReal( &m_extraFriction );

	// we don't need to save/load this, it is acquired on object creation
	// m_pui  <---- do nothing with this

	// mag of current vel
	xfer->xferReal( &m_velMag );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void PhysicsBehavior::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
