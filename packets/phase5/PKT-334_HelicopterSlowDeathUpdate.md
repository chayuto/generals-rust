# Translation Packet `PKT-334`: `HelicopterSlowDeathUpdate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-334`
- **Work Package:** `WP-5.4` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/update/helicopter_slow_death_update.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (22):**
  - `Common/AudioEventRTS.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/AudioEventRTS.h`
  - `Common/GameAudio.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameAudio.h`
  - `Common/GlobalData.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/GlobalData.h`
  - `Common/RandomValue.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/RandomValue.h`
  - `Common/ThingFactory.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ThingFactory.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/FXList.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/FXList.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameClient/ParticleSys.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/ParticleSys.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Locomotor.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Locomotor.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/EjectPilotDie.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/EjectPilotDie.h`
  - `GameLogic/Module/HelicopterSlowDeathUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/HelicopterSlowDeathUpdate.h`
  - `GameLogic/Module/PhysicsUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PhysicsUpdate.h`
  - `GameLogic/Module/SlowDeathBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SlowDeathBehavior.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/ObjectCreationList.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ObjectCreationList.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (26):**
  - `"SpiralOrbitTurnRate" => INI::parseAngularVelocityReal`
  - `"SpiralOrbitForwardSpeed" => INI::parseVelocityReal`
  - `"SpiralOrbitForwardSpeedDamping" => INI::parseReal`
  - `"MinSelfSpin" => INI::parseAngularVelocityReal`
  - `"MaxSelfSpin" => INI::parseAngularVelocityReal`
  - `"SelfSpinUpdateDelay" => INI::parseDurationReal`
  - `"SelfSpinUpdateAmount" => INI::parseAngleReal`
  - `"FallHowFast" => INI::parsePercentToReal`
  - `"MinBladeFlyOffDelay" => INI::parseDurationReal`
  - `"MaxBladeFlyOffDelay" => INI::parseDurationReal`
  - `"AttachParticle" => INI::parseParticleSystemTemplate`
  - `"AttachParticleBone" => INI::parseAsciiString`
  - `"AttachParticleLoc" => INI::parseCoord3D`
  - `"BladeObjectName" => INI::parseAsciiString`
  - `"BladeBoneName" => INI::parseAsciiString`
  - `"OCLEjectPilot" => INI::parseObjectCreationList`
  - `"FXBlade" => INI::parseFXList`
  - `"OCLBlade" => INI::parseObjectCreationList`
  - `"FXHitGround" => INI::parseFXList`
  - `"OCLHitGround" => INI::parseObjectCreationList`
  - `"FXFinalBlowUp" => INI::parseFXList`
  - `"OCLFinalBlowUp" => INI::parseObjectCreationList`
  - `"DelayFromGroundToFinalDeath" => INI::parseDurationReal`
  - `"FinalRubbleObject" => INI::parseAsciiString`
  - `"SoundDeathLoop" => INI::parseAudioEventRTS`
  - `"MaxBraking" => INI::parseAccelerationReal`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (9 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferInt( &m_orbitDirection )`
  3. `xferReal( &m_forwardAngle )`
  4. `xferReal( &m_forwardSpeed )`
  5. `xferReal( &m_selfSpin )`
  6. `xferBool( &m_selfSpinTowardsMax )`
  7. `xferUnsignedInt( &m_lastSelfSpinUpdateFrame )`
  8. `xferUnsignedInt( &m_bladeFlyOffFrame )`
  9. `xferUnsignedInt( &m_hitGroundFrame )`
- **PRNG Calls (1 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValueReal( modData->m_minBladeFlyOffDelay, modData->m_maxBladeFlyOffDelay )`

## 2.5 Existing Rust Symbol Index (Use These Exact Paths)
- `zh_logic::modules::update::assisted_targeting_update`: `AssistedTargetingUpdateModuleData`, `AssistedTargetingUpdate`
- `zh_logic::modules::update::auto_deposit_update`: `AutoDepositUpdateModuleData`, `AutoDepositUpdate`
- `zh_logic::modules::update::auto_find_healing_update`: `AutoFindHealingUpdateModuleData`, `AutoFindHealingUpdate`
- `zh_logic::modules::update::base_renerate_update`: `BaseRegenerateUpdateModuleData`, `BaseRegenerateUpdate`
- `zh_logic::modules::update::default_production_exit_update`: `Producer`, `DefaultProductionExitUpdateModuleData`, `DefaultProductionExitUpdate`
- `zh_logic::modules::update::deletion_update`: `DeletionUpdateModuleData`, `DeletionUpdate`
- `zh_logic::modules::update::dock_update`: `DockUpdateModuleData`, `FACET_MASK`, `DockUpdateState`
- `zh_logic::modules::update::dynamic_geometry_info_update`: `DIRECTION_FORWARD`, `DynamicGeometryInfoUpdateModuleData`, `DynamicGeometryInfoUpdate`
- `zh_logic::modules::update::dynamic_shroud_clearing_range_update`: `DynamicShroudClearingRangeUpdateModuleData`, `DscruState`, `DynamicShroudClearingRangeUpdate`
- `zh_logic::modules::update::fire_spread_update`: `FireSpreadUpdateModuleData`, `FireSpreadUpdate`
- `zh_logic::modules::update::fire_weapon_update`: `FireWeaponUpdateModuleData`, `FireWeaponUpdate`
- `zh_logic::modules::update::flammable_update`: `FlammableUpdateModuleData`, `FlammableUpdate`
- `zh_logic::modules::update::float_update`: `FloatUpdateModuleData`, `FloatUpdate`
- `zh_logic::modules::update::height_die_update`: `HeightDieUpdateModuleData`, `HeightDieUpdate`
- `zh_logic::modules::update::helicopter_slow_death_update`: `HelicopterSlowDeathBehaviorModuleData`, `HelicopterSlowDeathBehavior`
- `zh_logic::modules::update::horde_update`: `HordeActionType`, `TerrainDecalType`, `HordeUpdateModuleData`, `HordeUpdate`
- `zh_logic::modules::update::lifetime_update`: `LifetimeUpdateModuleData`, `LifetimeUpdate`
- `zh_logic::modules::update::ocl_update`: `FactionOclInfo`, `OclUpdateModuleData`, `OclUpdate`
- `zh_logic::modules::update::physics_update`: `STICK_TO_GROUND`, `ALLOW_BOUNCE`, `APPLY_FRICTION2D_WHEN_AIRBORNE`, `UPDATE_EVER_RUN`, `WAS_AIRBORNE_LAST_FRAME`, `ALLOW_COLLIDE_FORCE`, `ALLOW_TO_FALL`, `HAS_PITCHROLLYAW`, `IMMUNE_TO_FALLING_DAMAGE`, `IS_IN_FREEFALL`, `IS_IN_UPDATE`, `IS_STUNNED`, `TURN_NEGATIVE`, `TURN_NONE`, ... (+4)
- `zh_logic::modules::update::power_plant_update`: `PowerPlantUpdateModuleData`, `PowerPlantUpdate`
- `zh_logic::modules::update::projectile_stream_update`: `MAX_PROJECTILE_STREAM`, `ProjectileStreamUpdateModuleData`, `ProjectileStreamUpdate`
- `zh_logic::modules::update::prone_update`: `ProneUpdateModuleData`, `ProneUpdate`
- `zh_logic::modules::update::queue_production_exit_update`: `QueueProductionExitUpdateModuleData`, `QueueProductionExitUpdate`
- `zh_logic::modules::update::radar_update`: `RadarUpdateModuleData`, `RadarUpdate`
- `zh_logic::modules::update::radius_decal_update`: `RadiusDecalUpdateModuleData`, `RadiusDecalUpdate`
- `zh_logic::modules::update::repair_dock_update`: `RepairDockUpdateModuleData`, `RepairDockUpdate`
- `zh_logic::modules::update::smart_bomb_target_homing_update`: `SmartBombTargetHomingUpdateModuleData`, `SmartBombTargetHomingUpdate`
- `zh_logic::modules::update::spawn_point_production_exit_update`: `MAX_SPAWN_POINTS`, `SpawnPointProductionExitUpdateModuleData`, `SpawnPointProductionExitUpdate`
- `zh_logic::modules::update::spectre_gunship_deployment_update`: `COMMAND_FIRED_BY_SCRIPT`, `GunshipCreateLocType`, `SpectreGunshipDeploymentUpdateModuleData`, `SpectreGunshipDeploymentUpdate`
- `zh_logic::modules::update::spy_vision_update`: `SpyVisionUpdateModuleData`, `SpyVisionUpdate`, `fn`
- `zh_logic::modules::update::stealth_detector_update`: `StealthDetectorUpdateModuleData`, `StealthDetectorUpdate`
- `zh_logic::modules::update::stealth_update`: `STEALTH_NOT_WHILE_ATTACKING`, `STEALTH_NOT_WHILE_MOVING`, `STEALTH_NOT_WHILE_USING_ABILITY`, `STEALTH_NOT_WHILE_FIRING_PRIMARY`, `STEALTH_NOT_WHILE_FIRING_SECONDARY`, `STEALTH_NOT_WHILE_FIRING_TERTIARY`, `STEALTH_ONLY_WITH_BLACK_MARKET`, `STEALTH_NOT_WHILE_TAKING_DAMAGE`, `STEALTH_NOT_WHILE_RIDERS_ATTACKING`, `STEALTH_NOT_WHILE_FIRING_WEAPON`, `STEALTH_LEVEL_NAMES`, `StealthUpdateModuleData`, `StealthUpdate`
- `zh_logic::modules::update::structure_topple_update`: `STPHASE_INITIAL`, `STPHASE_DELAY`, `STPHASE_FINAL`, `ST_PHASE_COUNT`, `TOPPLESTATE_STANDING`, `TOPPLESTATE_WAITINGFORTOPPLESTART`, `TOPPLESTATE_TOPPLING`, `TOPPLESTATE_WAITINGFORDONE`, `TOPPLESTATE_DONE`, `StructureToppleUpdateModuleData`, `StructureToppleUpdate`
- `zh_logic::modules::update::supply_center_dock_update`: `SupplyCenterDockUpdateModuleData`, `SupplyCenterDockUpdate`
- `zh_logic::modules::update::supply_center_production_exit_update`: `SupplyCenterProductionExitUpdateModuleData`, `SupplyCenterProductionExitUpdate`
- `zh_logic::modules::update::supply_warehouse_dock_update`: `SupplyWarehouseDockUpdateModuleData`, `SupplyWarehouseDockUpdate`
- `zh_logic::modules::update::tensile_formation_update`: `TensileFormationUpdateModuleData`, `TensileLink`, `TensileFormationUpdate`
- `zh_logic::modules::update::topple_update`: `TOPPLE_OPTIONS_NONE`, `TOPPLE_OPTIONS_NO_BOUNCE`, `TOPPLE_OPTIONS_NO_FX`, `TOPPLE_UPRIGHT`, `TOPPLE_FALLING`, `TOPPLE_DOWN`, `ToppleUpdateModuleData`, `ToppleUpdate`
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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/HelicopterSlowDeathUpdate.h`
```cpp
// FILE: HelicopterSlowDeathUpdate.h //////////////////////////////////////////////////////////////
// Author: Colin Day, March 2002
// Desc:   Helicopter slow deaths
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "Common/AudioEventRTS.h"
#include "GameLogic/Module/SlowDeathBehavior.h"

// FORWARD DECLARATIONS ///////////////////////////////////////////////////////////////////////////
class ParticleSystemTemplate;

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class HelicopterSlowDeathBehaviorModuleData : public SlowDeathBehaviorModuleData
{

public:

	HelicopterSlowDeathBehaviorModuleData();

	static void buildFieldParse(MultiIniFieldParse &p );

	Real m_spiralOrbitTurnRate;						///< (rads per frame) rate at which we do big circles down toward the ground
	Real m_spiralOrbitForwardSpeed;				///< (dist per frame) speed at which we move "forward" in the downward spiral
	Real m_spiralOrbitForwardSpeedDamping;///< every frame our forward speed in the orbit is adjusted by this amount
	Real m_minSelfSpin;										///< (rads per frame) min turning rate at which we spin around our center of gravity
	Real m_maxSelfSpin;										///< (rads per frame) max turning rate at which we spin around our center of gravity
	Real m_selfSpinUpdateDelay;						///< (frames) every this many frames we will update the self spin angle, but we'll keep it in between the min and max self spin
	Real m_selfSpinUpdateAmount;					///< (radian) when we update the self spin every SelfSpinUpdateDelay frames, we change it this much, but keep it between min and max self spin
	Real m_fallHowFast;										///< a fraction of gravity we use to modify the helicopert locmotor lift
	Real m_minBladeFlyOffDelay;						///< (frames) min frame that the blade will fly off at
	Real m_maxBladeFlyOffDelay;						///< (frames) max frame that the blade will fly off at
	const ParticleSystemTemplate *m_attachParticleSystem;		///< particle system to attach
	AsciiString m_attachParticleBone;			///< bone to attach particle system to
	Coord3D m_attachParticleLoc;					///< loc attach particle system to if bone not present
	AsciiString m_bladeObjectName;				///< object name of the blade piece
	AsciiString m_bladeBone;							///< bone name of main propeller blade
	const ObjectCreationList *m_oclEjectPilot;	///< OCL for ejecting pilot
	const FXList *m_fxBlade;							///< blade fly off event fx list
	const ObjectCreationList *m_oclBlade;	///< OCL at blade fly off event
	const FXList *m_fxHitGround;					///< fx for hitting the ground
	const ObjectCreationList *m_oclHitGround;  ///< OCL at hit ground event
	const FXList *m_fxFinalBlowUp;				///< the final blow up
	const ObjectCreationList *m_oclFinalBlowUp;///< OCL at final blow up event
	Real m_delayFromGroundToFinalDeath;		///< (frames) delay from when we hit the ground to final BOOM!
	AsciiString m_finalRubbleObject;			///< final rubble object to create after it's ALL over
	Real m_maxBraking;										///< max braking we may use during death spiral

	// @todo propagate this up to SlowDeathBehaviorModuleData. I don't wanna do it today, cause its 4/3. jkmcd
	AudioEventRTS m_deathSound;						///< Sound played during death sequence.

};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class HelicopterSlowDeathBehavior : public SlowDeathBehavior
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( HelicopterSlowDeathBehavior, "HelicopterSlowDeathBehavior" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( HelicopterSlowDeathBehavior, HelicopterSlowDeathBehaviorModuleData )

public:

	HelicopterSlowDeathBehavior( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	virtual void beginSlowDeath( const DamageInfo *damageInfo ) override;	///< begin the slow death cycle
	virtual UpdateSleepTime update() override;

protected:

	Int m_orbitDirection;			 ///< -1 or +1 ... use ORBIT_DIRECTION_LEFT/RIGHT
	Real m_forwardAngle;			 /**< angle of the direction "forward" we want to move in our downspin
																	which is independent of the actual direction angle of the object */
	Real m_forwardSpeed;			 ///< the speed we're travelling forward in our spiral orbit
	Real m_selfSpin;					 ///< rads per frame that we change our facing direction
	Bool m_selfSpinTowardsMax; ///< TRUE when our self spin rate is increasing towards the MaxSelfSpin
	UnsignedInt m_lastSelfSpinUpdateFrame;	///< frame we last updated the self spin on
	UnsignedInt m_bladeFlyOffFrame;		///< frame we throw the blade off at
	UnsignedInt m_hitGroundFrame;			///< frame we hit the ground on

	// @todo propagate this up to SlowDeathBehavior. I don't wanna do it today, cause its 4/3. jkmcd
	AudioEventRTS m_deathSound;						///< Sound played during death sequence.

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/HelicopterSlowDeathUpdate.cpp`
```cpp
// FILE: HelicopterSlowDeathBehavior.cpp ////////////////////////////////////////////////////////////
// Author: Colin Day, March 2002
// Desc:   Helicopter slow deaths
///////////////////////////////////////////////////////////////////////////////////////////////////

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/GameAudio.h"
#include "Common/GlobalData.h"
#include "Common/RandomValue.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"
#include "GameClient/Drawable.h"
#include "GameClient/FXList.h"
#include "GameClient/InGameUI.h"
#include "GameClient/ParticleSys.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Locomotor.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectCreationList.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/EjectPilotDie.h"
#include "GameLogic/Module/HelicopterSlowDeathUpdate.h"
#include "GameLogic/Module/PhysicsUpdate.h"

///////////////////////////////////////////////////////////////////////////////////////////////////
// Helicopter slow death update module data ///////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////




//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
HelicopterSlowDeathBehaviorModuleData::HelicopterSlowDeathBehaviorModuleData()
{
	m_attachParticleBone.clear();
	m_bladeBone.clear();
	m_bladeObjectName.clear();
	m_finalRubbleObject.clear();

	m_spiralOrbitTurnRate = 0.0f;
	m_spiralOrbitForwardSpeed = 0.0f;
	m_spiralOrbitForwardSpeedDamping = 1.0f;  // no damping
	m_minSelfSpin = 0.0f;
	m_maxSelfSpin = 0.0f;
	m_selfSpinUpdateDelay = 0.0f;
	m_selfSpinUpdateAmount = 0.0f;
	m_fallHowFast = 0.0f;
	m_minBladeFlyOffDelay = 0.0;
	m_maxBladeFlyOffDelay = 0.0;

	m_attachParticleSystem = nullptr;
	m_attachParticleLoc.x = 0.0f;
	m_attachParticleLoc.y = 0.0f;
	m_attachParticleLoc.z = 0.0f;
	m_oclEjectPilot = nullptr;
	m_fxBlade = nullptr;
	m_oclBlade = nullptr;
	m_fxHitGround = nullptr;
	m_oclHitGround = nullptr;
	m_fxFinalBlowUp = nullptr;
	m_oclFinalBlowUp = nullptr;
	m_delayFromGroundToFinalDeath = 0;
	m_maxBraking = 99999.0f;

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
/*static*/ void HelicopterSlowDeathBehaviorModuleData::buildFieldParse( MultiIniFieldParse &p )
{
  SlowDeathBehaviorModuleData::buildFieldParse( p );

	static const FieldParse dataFieldParse[] =
	{
		{ "SpiralOrbitTurnRate",	INI::parseAngularVelocityReal, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_spiralOrbitTurnRate ) },
		{ "SpiralOrbitForwardSpeed", INI::parseVelocityReal, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_spiralOrbitForwardSpeed ) },
		{ "SpiralOrbitForwardSpeedDamping", INI::parseReal, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_spiralOrbitForwardSpeedDamping ) },
		{ "MinSelfSpin", INI::parseAngularVelocityReal, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_minSelfSpin ) },
		{ "MaxSelfSpin", INI::parseAngularVelocityReal, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_maxSelfSpin ) },
		{ "SelfSpinUpdateDelay", INI::parseDurationReal, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_selfSpinUpdateDelay ) },
		{ "SelfSpinUpdateAmount", INI::parseAngleReal, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_selfSpinUpdateAmount ) },
		{ "FallHowFast", INI::parsePercentToReal, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_fallHowFast ) },
		{ "MinBladeFlyOffDelay", INI::parseDurationReal, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_minBladeFlyOffDelay ) },
		{ "MaxBladeFlyOffDelay", INI::parseDurationReal, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_maxBladeFlyOffDelay ) },
		{ "AttachParticle", INI::parseParticleSystemTemplate, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_attachParticleSystem ) },
		{ "AttachParticleBone", INI::parseAsciiString, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_attachParticleBone ) },
		{ "AttachParticleLoc", INI::parseCoord3D, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_attachParticleLoc ) },
		{ "BladeObjectName", INI::parseAsciiString, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_bladeObjectName ) },
		{ "BladeBoneName", INI::parseAsciiString, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_bladeBone ) },
		{ "OCLEjectPilot", INI::parseObjectCreationList, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_oclEjectPilot ) },
		{ "FXBlade", INI::parseFXList, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_fxBlade ) },
		{ "OCLBlade", INI::parseObjectCreationList, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_oclBlade ) },
		{ "FXHitGround", INI::parseFXList, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_fxHitGround ) },
		{ "OCLHitGround", INI::parseObjectCreationList, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_oclHitGround ) },
		{ "FXFinalBlowUp", INI::parseFXList, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_fxFinalBlowUp ) },
		{ "OCLFinalBlowUp", INI::parseObjectCreationList, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_oclFinalBlowUp ) },
		{ "DelayFromGroundToFinalDeath", INI::parseDurationReal, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_delayFromGroundToFinalDeath ) },
		{ "FinalRubbleObject", INI::parseAsciiString, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_finalRubbleObject ) },
		{ "SoundDeathLoop", INI::parseAudioEventRTS, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_deathSound) },
		{ "MaxBraking", INI::parseAccelerationReal, nullptr, offsetof( HelicopterSlowDeathBehaviorModuleData, m_maxBraking) },

		{ nullptr, nullptr, nullptr, 0 }

	};

  p.add(dataFieldParse);

}

///////////////////////////////////////////////////////////////////////////////////////////////////
// Helicopter slow death update ///////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

//
// we use these to guide the direction of the circular orbit, as well as the direction the
// object self spins
//
#define ORBIT_DIRECTION_LEFT  ( 1.0f)
#define ORBIT_DIRECTION_RIGHT (-1.0f)

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
HelicopterSlowDeathBehavior::HelicopterSlowDeathBehavior( Thing *thing, const ModuleData *moduleData )
													: SlowDeathBehavior( thing, moduleData )
{

	m_orbitDirection = ORBIT_DIRECTION_LEFT;
	m_forwardAngle = 0.0f;
	m_forwardSpeed = 0.0f;
	m_selfSpin = 0.0f;
	m_selfSpinTowardsMax = FALSE;
	m_lastSelfSpinUpdateFrame = 0;
	m_bladeFlyOffFrame = 0;
	m_hitGroundFrame = 0;

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
HelicopterSlowDeathBehavior::~HelicopterSlowDeathBehavior()
{

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void HelicopterSlowDeathBehavior::beginSlowDeath( const DamageInfo *damageInfo )
{

	// extending functionality
	SlowDeathBehavior::beginSlowDeath( damageInfo );

	// stop the current movement sound
	if (getObject()->getDrawable())
	{
		getObject()->getDrawable()->stopAmbientSound();
	}

	// get the module data
	const HelicopterSlowDeathBehaviorModuleData *modData = getHelicopterSlowDeathBehaviorModuleData();

	m_deathSound = modData->m_deathSound;

	if (m_deathSound.getEventName().isEmpty() == false)
	{
		m_deathSound.setObjectID(getObject()->getID());
		m_deathSound.setPlayingHandle(TheAudio->addAudioEvent(&m_deathSound));
	}


	// pick a frame we will fly the blade off at
	m_bladeFlyOffFrame = GameLogicRandomValueReal( modData->m_minBladeFlyOffDelay,
																								 modData->m_maxBladeFlyOffDelay );

	// for now, make it always fall to the left
	m_orbitDirection = ORBIT_DIRECTION_LEFT;

	//
	// record the current forward angle of the object to be the start angle of our downward
	// spin circle angle
	//
	m_forwardAngle = getObject()->getOrientation();

	// set or forward speed in the spiral orbit speed to that specified
	m_forwardSpeed = modData->m_spiralOrbitForwardSpeed;

	// start our self spinning at the min self spin rate
	m_selfSpin = modData->m_minSelfSpin;

	// we will start off changing the self spin towards the MaxSelfSpin
	m_selfSpinTowardsMax = TRUE;

	// the fall of the helicopter
	if( !getObject()->getAIUpdateInterface() )
	{
		return;
	}
	Locomotor *locomotor = getObject()->getAIUpdateInterface()->getCurLocomotor();
	locomotor->setMaxLift( -TheGlobalData->m_gravity * (1.0f - modData->m_fallHowFast) );
	locomotor->setMaxBraking( modData->m_maxBraking );

	// attach particle system to bone if present
	if( modData->m_attachParticleSystem )
	{
		ParticleSystem *pSys = TheParticleSystemManager->createParticleSystem( modData->m_attachParticleSystem );
		if( pSys )
		{

			// where do the offset attachment to
			if( modData->m_attachParticleBone.isEmpty() == FALSE )
			{
				Drawable *draw = getObject()->getDrawable();

				if( draw )
				{
					Coord3D pos;

					if( draw->getPristineBonePositions( modData->m_attachParticleBone.str(), 0, &pos, nullptr, 1 ) )
						pSys->setPosition( &pos );

				}

			}
			else
			{

				// use location coord specified ... it will be zero if not given which is center of obj anyway
				pSys->setPosition( &modData->m_attachParticleLoc );

			}

			// attach the particle system to the object
			pSys->attachToObject( getObject() );

		}

	}



}




//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
UpdateSleepTime HelicopterSlowDeathBehavior::update()
{
/// @todo srj use SLEEPY_UPDATE here
	// call the base class cause we're extending functionality
	SlowDeathBehavior::update();

	// get out of here if we're not activated yet
	if( isSlowDeathActivated() == FALSE )
		return UPDATE_SLEEP_NONE;

	// get the module data
	const HelicopterSlowDeathBehaviorModuleData *modData = getHelicopterSlowDeathBehaviorModuleData();

	// get the helicopter for easy access in here
	Object *copter = getObject();

	// update self spin and orbit if we haven't hit the ground yet
	if( m_hitGroundFrame == 0 )
	{

		//
		// change the angle of the helicopter based on the spiral rate for the turning on the
		// large downward spin circle
		//

		//copter->setOrientation( copter->getOrientation() + m_selfSpin * m_orbitDirection );

		Matrix3D xfrm = *copter->getTransformMatrix();
		xfrm.In_Place_Pre_Rotate_Z(m_selfSpin * m_orbitDirection);
		copter->setTransformMatrix( &xfrm );

		//
		// over time we change the rate at which we self spin around our center of gravity ... we
		// will ping pong back and forth between the MinSelfSpin and MaxSelfSpin defined in INI
		//
		if( modData->m_selfSpinUpdateDelay &&
				TheGameLogic->getFrame() - m_lastSelfSpinUpdateFrame > modData->m_selfSpinUpdateDelay )
		{

			// update the self spin
			if( m_selfSpinTowardsMax == TRUE )
			{

				// we're going towards the max self spin, increase it
				m_selfSpin += modData->m_selfSpinUpdateAmount / static_cast<float>(LOGICFRAMES_PER_SECOND);
				if( m_selfSpin > modData->m_maxSelfSpin )
				{

					m_selfSpin = modData->m_maxSelfSpin;  // cap at max
					m_selfSpinTowardsMax = FALSE;					// now start changing spin towards min again

				}

			}
			else
			{

				// we're going towards the min self spin, decrease it
				m_selfSpin -= modData->m_selfSpinUpdateAmount / static_cast<float>(LOGICFRAMES_PER_SECOND);
				if( m_selfSpin < modData->m_minSelfSpin )
				{

					m_selfSpin = modData->m_minSelfSpin;  // cap at min
					m_selfSpinTowardsMax = TRUE;					// now start chaning spin towards max again

				}

			}

			// we have made a change to the self spinning
			m_lastSelfSpinUpdateFrame = TheGameLogic->getFrame();

		}

		// get the physics update module
		PhysicsBehavior *physics = copter->getPhysics();
		DEBUG_ASSERTCRASH( physics, ("HelicopterSlowDeathBehavior: object '%s' does not have a physics module",
																 copter->getTemplate()->getName().str()) );

		//
		// apply a force to the helicopter pushing it in a forward motion	according to the
		// forward angle & speed we are keeping track of to make our downward circle (that forward angle
		// is *NOT* the angle the object is facing
		//
		Coord3D force;
		force.x = DOUBLE_TO_REAL( Cos( m_forwardAngle ) ) * m_forwardSpeed;
		force.y = DOUBLE_TO_REAL( Sin( m_forwardAngle ) ) * m_forwardSpeed;
		force.z = 0.0f;
		physics->applyMotiveForce( &force );

		// update our forward angle for travelling along the large spiral downward circle
		m_forwardAngle += (modData->m_spiralOrbitTurnRate * m_orbitDirection);

		// adjust our forward spiral orbit by the damping factor specified
		m_forwardSpeed *= modData->m_spiralOrbitForwardSpeedDamping;

		// is it time to have the blade fly off
		if( m_bladeFlyOffFrame > 0 )
		{
			// decrement count
			m_bladeFlyOffFrame--;
			if( m_bladeFlyOffFrame <= 0 )
			{
				Coord3D bladePos = *copter->getPosition();

				// get the blade position from the bone in the model
				Drawable *draw = copter->getDrawable();
				if( draw )
				{

					draw->getPristineBonePositions( modData->m_bladeBone.str(), 0, &bladePos, nullptr, 1 );
					draw->convertBonePosToWorldPos( &bladePos, nullptr, &bladePos, nullptr );

				}

				// create the blades flying through the air
	//			const ObjectCreationList *ocl = TheObjectCreationListStore->findObjectCreationList( "OCL_ComancheBlades" );
	//			ObjectCreationList::create( ocl, &bladePos );

				// run the fx at the blade position
				FXList::doFXPos( modData->m_fxBlade, &bladePos );
				ObjectCreationList::create( modData->m_oclBlade, copter, &bladePos, nullptr, INVALID_ANGLE );

				//
				// if we have (potentially) a pilot ejection, do it here.
				// note that we call EjectPilotDie::ejectPilot() rather than ObjectCreationList::create(),
				// because the former makes the right sounds, and also constrains to veteran-or-better status.
				//
				if( modData->m_oclEjectPilot && copter->getVeterancyLevel() > LEVEL_REGULAR )
					EjectPilotDie::ejectPilot( modData->m_oclEjectPilot, copter, nullptr );

			}

		}

	}



	Bool hitATree = FALSE;
	// Here we want to make sure we crash if we collide with a tree on the way down
	PhysicsBehavior *phys = copter->getPhysics();
	if ( m_hitGroundFrame == 0 && phys )
	{
		ObjectID treeID = phys->getLastCollidee();
		Object *tree = TheGameLogic->findObjectByID( treeID );
		if ( tree )
		{
			if (tree->isKindOf( KINDOF_SHRUBBERY ) )
			hitATree = TRUE;
		}
	}



	// when we hit the ground
	const Coord3D *pos = copter->getPosition();
	if (m_hitGroundFrame == 0)
	{
		// srj sez: if we haven't yet hit the ground, adjust our layer properly so we crash on bridges
		Coord3D tmpPt = *pos;
		tmpPt.z = 99999.0f;
		PathfindLayerEnum newLayer = TheTerrainLogic->getHighestLayerForDestination(&tmpPt);
		copter->setLayer(newLayer);

		Real ground = TheTerrainLogic->getLayerHeight( tmpPt.x, tmpPt.y, newLayer );

		if (pos->z <= ground + 1.0f || hitATree )
		{

			// mark the frame we hit the ground on
			m_hitGroundFrame = TheGameLogic->getFrame();

			// make hit ground effect
			FXList::doFXObj( modData->m_fxHitGround, copter );
			ObjectCreationList::create( modData->m_oclHitGround, copter, nullptr );

			// hold the copter in place now
			copter->setDisabled( DISABLED_HELD );

			// special damage state ... has the blades no on anymore
			Drawable *draw = copter->getDrawable();
			if( draw )
				draw->setModelConditionState( MODELCONDITION_SPECIAL_DAMAGED );

			// Stop the sound from playing.
			TheAudio->removeAudioEvent(m_deathSound.getPlayingHandle());

		}
	}

	// if we're on the ground, see if it's time for our final boom
	if( m_hitGroundFrame &&
			TheGameLogic->getFrame() - m_hitGroundFrame > modData->m_delayFromGroundToFinalDeath )
	{

		// make effect
		FXList::doFXObj( modData->m_fxFinalBlowUp, copter );
		ObjectCreationList::create( modData->m_oclFinalBlowUp, copter, nullptr );

		// we are now all done, destroy us and make a rubble shell copter
		const ThingTemplate* ttn = TheThingFactory->findTemplate(modData->m_finalRubbleObject);
		Object *rubble = TheThingFactory->newObject( ttn, copter->getTeam() );
		if( rubble )
		{

			rubble->setTransformMatrix( copter->getTransformMatrix() );

		}

		// destroy the copter finally
		TheGameLogic->destroyObject( copter );

	}

	return UPDATE_SLEEP_NONE;

}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void HelicopterSlowDeathBehavior::crc( Xfer *xfer )
{

	// extend base class
	SlowDeathBehavior::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void HelicopterSlowDeathBehavior::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	SlowDeathBehavior::xfer( xfer );

	// orbit direction
	xfer->xferInt( &m_orbitDirection );

	// forward angle
	xfer->xferReal( &m_forwardAngle );

	// forward speed
	xfer->xferReal( &m_forwardSpeed );

	// self spin
	xfer->xferReal( &m_selfSpin );

	// self sping towards max
	xfer->xferBool( &m_selfSpinTowardsMax );

	// last self spin update frame
	xfer->xferUnsignedInt( &m_lastSelfSpinUpdateFrame );

	// blade fly off frame
	xfer->xferUnsignedInt( &m_bladeFlyOffFrame );

	// hit ground frame
	xfer->xferUnsignedInt( &m_hitGroundFrame );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void HelicopterSlowDeathBehavior::loadPostProcess()
{

	// extend base class
	SlowDeathBehavior::loadPostProcess();

}
```
