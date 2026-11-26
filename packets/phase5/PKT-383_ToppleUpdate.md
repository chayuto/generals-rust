# Translation Packet `PKT-383`: `ToppleUpdate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-383`
- **Work Package:** `WP-5.4` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/update/topple_update.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (21):**
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/PlayerList.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/PlayerList.h`
  - `Common/RandomValue.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/RandomValue.h`
  - `Common/ThingFactory.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ThingFactory.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/FXList.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/FXList.h`
  - `GameClient/Module/SwayClientUpdate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/Module/SwayClientUpdate.h`
  - `GameLogic/AI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/AI.h`
  - `GameLogic/AIPathfind.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/AIPathfind.h`
  - `GameLogic/Damage.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/Damage.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/CollideModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CollideModule.h`
  - `GameLogic/Module/PhysicsUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PhysicsUpdate.h`
  - `GameLogic/Module/ToppleUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ToppleUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/ScriptEngine.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ScriptEngine.h`
  - `PreRTS.h`
- **Implemented Module Facets (1):**
  - `CollideModuleInterface* getCollide()`
- **INI `FieldParse` Table Entries (11):**
  - `"ToppleFX" => INI::parseFXList`
  - `"BounceFX" => INI::parseFXList`
  - `"StumpName" => INI::parseAsciiString`
  - `"KillWhenStartToppling" => INI::parseBool`
  - `"KillWhenFinishedToppling" => INI::parseBool`
  - `"KillStumpWhenToppled" => INI::parseBool`
  - `"ToppleLeftOrRightOnly" => INI::parseBool`
  - `"ReorientToppledRubble" => INI::parseBool`
  - `"InitialVelocityPercent" => INI::parsePercentToReal`
  - `"InitialAccelPercent" => INI::parsePercentToReal`
  - `"BounceVelocityPercent" => INI::parsePercentToReal`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (11 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferReal( &m_angularVelocity )`
  3. `xferReal( &m_angularAcceleration )`
  4. `xferCoord3D( &m_toppleDirection )`
  5. `xferUser( &m_toppleState, sizeof( ToppleState ) )`
  6. `xferReal( &m_angularAccumulation )`
  7. `xferReal( &m_angleDeltaX )`
  8. `xferInt( &m_numAngleDeltaX )`
  9. `xferBool( &m_doBounceFX )`
  10. `xferUnsignedInt( &m_options )`
  11. `xferObjectID( &m_stumpID )`

## 2.5 Existing Rust Symbol Index (Use These Exact Paths)
- `zh_logic::modules::update::auto_deposit_update`: `AutoDepositUpdateModuleData`, `AutoDepositUpdate`
- `zh_logic::modules::update::base_renerate_update`: `BaseRegenerateUpdateModuleData`, `BaseRegenerateUpdate`
- `zh_logic::modules::update::deletion_update`: `DeletionUpdateModuleData`, `DeletionUpdate`
- `zh_logic::modules::update::dock_update`: `DockUpdateModuleData`, `FACET_MASK`, `DockUpdateState`
- `zh_logic::modules::update::dynamic_geometry_info_update`: `DIRECTION_FORWARD`, `DynamicGeometryInfoUpdateModuleData`, `DynamicGeometryInfoUpdate`
- `zh_logic::modules::update::dynamic_shroud_clearing_range_update`: `DynamicShroudClearingRangeUpdateModuleData`, `DscruState`, `DynamicShroudClearingRangeUpdate`
- `zh_logic::modules::update::fire_spread_update`: `FireSpreadUpdateModuleData`, `FireSpreadUpdate`
- `zh_logic::modules::update::fire_weapon_update`: `FireWeaponUpdateModuleData`, `FireWeaponUpdate`
- `zh_logic::modules::update::flammable_update`: `FlammableUpdateModuleData`, `FlammableUpdate`
- `zh_logic::modules::update::float_update`: `FloatUpdateModuleData`, `FloatUpdate`
- `zh_logic::modules::update::height_die_update`: `HeightDieUpdateModuleData`, `HeightDieUpdate`
- `zh_logic::modules::update::lifetime_update`: `LifetimeUpdateModuleData`, `LifetimeUpdate`
- `zh_logic::modules::update::physics_update`: `STICK_TO_GROUND`, `ALLOW_BOUNCE`, `APPLY_FRICTION2D_WHEN_AIRBORNE`, `UPDATE_EVER_RUN`, `WAS_AIRBORNE_LAST_FRAME`, `ALLOW_COLLIDE_FORCE`, `ALLOW_TO_FALL`, `HAS_PITCHROLLYAW`, `IMMUNE_TO_FALLING_DAMAGE`, `IS_IN_FREEFALL`, `IS_IN_UPDATE`, `IS_STUNNED`, `TURN_NEGATIVE`, `TURN_NONE`, ... (+4)
- `zh_logic::modules::update::power_plant_update`: `PowerPlantUpdateModuleData`, `PowerPlantUpdate`
- `zh_logic::modules::update::projectile_stream_update`: `MAX_PROJECTILE_STREAM`, `ProjectileStreamUpdateModuleData`, `ProjectileStreamUpdate`
- `zh_logic::modules::update::prone_update`: `ProneUpdateModuleData`, `ProneUpdate`
- `zh_logic::modules::update::radar_update`: `RadarUpdateModuleData`, `RadarUpdate`
- `zh_logic::modules::update::radius_decal_update`: `RadiusDecalUpdateModuleData`, `RadiusDecalUpdate`
- `zh_logic::modules::update::repair_dock_update`: `RepairDockUpdateModuleData`, `RepairDockUpdate`
- `zh_logic::modules::update::smart_bomb_target_homing_update`: `SmartBombTargetHomingUpdateModuleData`, `SmartBombTargetHomingUpdate`
- `zh_logic::modules::update::spy_vision_update`: `SpyVisionUpdateModuleData`, `SpyVisionUpdate`, `fn`
- `zh_logic::modules::update::structure_topple_update`: `STPHASE_INITIAL`, `STPHASE_DELAY`, `STPHASE_FINAL`, `ST_PHASE_COUNT`, `TOPPLESTATE_STANDING`, `TOPPLESTATE_WAITINGFORTOPPLESTART`, `TOPPLESTATE_TOPPLING`, `TOPPLESTATE_WAITINGFORDONE`, `TOPPLESTATE_DONE`, `StructureToppleUpdateModuleData`, `StructureToppleUpdate`
- `zh_logic::modules::update::supply_center_dock_update`: `SupplyCenterDockUpdateModuleData`, `SupplyCenterDockUpdate`
- `zh_logic::modules::update::supply_warehouse_dock_update`: `SupplyWarehouseDockUpdateModuleData`, `SupplyWarehouseDockUpdate`
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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ToppleUpdate.h`
```cpp
// FILE: ToppleUpdate.h /////////////////////////////////////////////////////////////////////////////
// Author:
// Desc:
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/CollideModule.h"
#include "GameLogic/Module/UpdateModule.h"

//-------------------------------------------------------------------------------------------------
class FXList;

// ------------------------------------------------------------------------------------------------
/** Topple options */
// ------------------------------------------------------------------------------------------------
enum
{
	TOPPLE_OPTIONS_NONE			 = 0x00000000,
	TOPPLE_OPTIONS_NO_BOUNCE = 0x00000001,  ///< do not bounce when hit the ground
	TOPPLE_OPTIONS_NO_FX		 = 0x00000002,	///< do not play any FX when hit the ground
};

//-------------------------------------------------------------------------------------------------
/** Topple update */
//-------------------------------------------------------------------------------------------------
class ToppleUpdateModuleData : public UpdateModuleData
{
public:
	const FXList* m_toppleFX;
	const FXList* m_bounceFX;
	AsciiString m_stumpName;
	Real m_initialVelocityPercent;
	Real m_initialAccelPercent;
	Real m_bounceVelocityPercent;
	Bool m_killWhenToppled;
	Bool m_killWhenStartToppled;
	Bool m_killStumpWhenToppled;
	Bool m_toppleLeftOrRightOnly;	// constrained to topple to my left or right only
	Bool m_reorientToppledRubble;

	ToppleUpdateModuleData();
	static void buildFieldParse(MultiIniFieldParse& p);

};

//-------------------------------------------------------------------------------------------------
class ToppleUpdate : public UpdateModule,
										 public CollideModuleInterface
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( ToppleUpdate, "ToppleUpdate" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( ToppleUpdate, ToppleUpdateModuleData )

public:

	ToppleUpdate( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	static Int getInterfaceMask() { return UpdateModule::getInterfaceMask() | (MODULEINTERFACE_COLLIDE); }

	// BehaviorModule
	virtual CollideModuleInterface* getCollide() override { return this; }

	void applyTopplingForce( const Coord3D* toppleDirection, Real toppleSpeed, UnsignedInt options);
	Bool isAbleToBeToppled() const;

	// UpdateModuleInterface
	virtual UpdateSleepTime update() override;

	// CollideModuleInterface
	virtual void onCollide( Object *other, const Coord3D *loc, const Coord3D *normal ) override;
	/// this is used for things like pilots, to determine if they can "enter" something
	virtual Bool wouldLikeToCollideWith(const Object* other) const override { return false; }
	virtual Bool isHijackedVehicleCrateCollide() const override { return false; }
	virtual Bool isCarBombCrateCollide() const override { return false; }
	virtual Bool isRailroad() const override { return false; }
	virtual Bool isSalvageCrateCollide() const override { return false; }
	virtual Bool isSabotageBuildingCrateCollide() const override { return FALSE; }

protected:

	enum ToppleState
	{
		TOPPLE_UPRIGHT = 0,
		TOPPLE_FALLING,
		TOPPLE_DOWN
	};

	Real					m_angularVelocity;				///< Velocity in degrees per frame (or is it radians per frame?)
	Real					m_angularAcceleration;		///< Acceleration angularVelocity is increasing
	Coord3D				m_toppleDirection;				///< Z-less direction we are toppling
	ToppleState		m_toppleState;						///< Stage this module is in.
	Real					m_angularAccumulation;		///< How much have I rotated so I know when to bounce.
	Real					m_angleDeltaX;						///< how much to modify x angle each frame
	Int						m_numAngleDeltaX;					///< how many frames to tweak x angle
	Bool					m_doBounceFX;							///< do the bounce FX if we do bounce
	UnsignedInt		m_options;								///< topple options
	ObjectID			m_stumpID;								///< stump generated, if any

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/ToppleUpdate.cpp`
```cpp
// FILE: ToppleUpdate.cpp ///////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, December 2001
// Desc:   Update for something that sits around until hit, then topples and stops
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/ThingTemplate.h"
#include "Common/ThingFactory.h"
#include "Common/PlayerList.h"
#include "Common/Player.h"
#include "Common/RandomValue.h"
#include "Common/Xfer.h"
#include "GameClient/FXList.h"
#include "GameLogic/AI.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/Module/ToppleUpdate.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameClient/Drawable.h"
#include "GameClient/Module/SwayClientUpdate.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/ScriptEngine.h"
#include "GameLogic/Damage.h"

//-------------------------------------------------------------------------------------------------
// this is our "bounce" limit -- slightly less that 90 degrees, to account for slop.
static const Real ANGULAR_LIMIT = PI/2 - PI/64;

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ToppleUpdateModuleData::ToppleUpdateModuleData()
{
	const Real START_VELOCITY_PERCENT = 0.2f;
	const Real START_ACCEL_PERCENT = 0.01f;
	const Real VELOCITY_BOUNCE_PERCENT = 0.3f;			// multiply the velocity by this when you bounce
	m_toppleFX = nullptr;
	m_bounceFX = nullptr;
	m_stumpName.clear();
	m_killWhenToppled = true;
	m_killWhenStartToppled = false;
	m_killStumpWhenToppled = false;
	m_toppleLeftOrRightOnly = false;
	m_reorientToppledRubble = false;
	m_initialVelocityPercent = START_VELOCITY_PERCENT;
	m_initialAccelPercent = START_ACCEL_PERCENT;
	m_bounceVelocityPercent = VELOCITY_BOUNCE_PERCENT;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void ToppleUpdateModuleData::buildFieldParse(MultiIniFieldParse& p)
{
  UpdateModuleData::buildFieldParse(p);

	static const FieldParse dataFieldParse[] =
	{
		{ "ToppleFX",	INI::parseFXList, nullptr, offsetof( ToppleUpdateModuleData, m_toppleFX ) },
		{ "BounceFX",	INI::parseFXList, nullptr, offsetof( ToppleUpdateModuleData, m_bounceFX ) },
		{ "StumpName",	INI::parseAsciiString, nullptr, offsetof( ToppleUpdateModuleData, m_stumpName ) },
		{ "KillWhenStartToppling",	INI::parseBool, nullptr, offsetof( ToppleUpdateModuleData, m_killWhenStartToppled ) },
		{ "KillWhenFinishedToppling",	INI::parseBool, nullptr, offsetof( ToppleUpdateModuleData, m_killWhenToppled ) },
		{ "KillStumpWhenToppled",	INI::parseBool, nullptr, offsetof( ToppleUpdateModuleData, m_killStumpWhenToppled ) },
		{ "ToppleLeftOrRightOnly",	INI::parseBool, nullptr, offsetof( ToppleUpdateModuleData, m_toppleLeftOrRightOnly ) },
		{ "ReorientToppledRubble",	INI::parseBool, nullptr, offsetof( ToppleUpdateModuleData, m_reorientToppledRubble ) },
		{ "InitialVelocityPercent",	INI::parsePercentToReal, nullptr, offsetof( ToppleUpdateModuleData, m_initialVelocityPercent ) },
		{ "InitialAccelPercent",	INI::parsePercentToReal, nullptr, offsetof( ToppleUpdateModuleData, m_initialAccelPercent ) },
		{ "BounceVelocityPercent",	INI::parsePercentToReal, nullptr, offsetof( ToppleUpdateModuleData, m_bounceVelocityPercent ) },
		{ nullptr, nullptr, nullptr, 0 }
	};
  p.add(dataFieldParse);
}


//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ToppleUpdate::ToppleUpdate( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{
	m_angleDeltaX = 0.0f;
	m_doBounceFX = FALSE;
	m_numAngleDeltaX = 0;
	m_angularVelocity = 0;
	m_angularAccumulation = 0;
	m_angularAcceleration = 0;
	m_toppleDirection.x = 0;
	m_toppleDirection.y = 0;
	m_toppleDirection.z = 0;
	m_toppleState = TOPPLE_UPRIGHT;
	m_options = TOPPLE_OPTIONS_NONE;
	m_stumpID = INVALID_ID;
	setWakeFrame(getObject(), UPDATE_SLEEP_FOREVER);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ToppleUpdate::~ToppleUpdate()
{
}

//-------------------------------------------------------------------------------------------------
static Real angleClosestTo(Real a1, Real a2, Real desired)
{
	a1 = normalizeAngle(a1);
	a2 = normalizeAngle(a2);
	return (WWMath::Fabs(stdAngleDiff(desired, a1)) < WWMath::Fabs(stdAngleDiff(desired, a2))) ? a1 : a2;
}

//-------------------------------------------------------------------------------------------------
///< Start the toppling process by giving a force vector
//-------------------------------------------------------------------------------------------------
void ToppleUpdate::applyTopplingForce( const Coord3D* toppleDirection, Real toppleSpeed,
																			 UnsignedInt options )
{
	if (getObject()->isEffectivelyDead())
		return;

	//DEBUG_LOG(("awaking ToppleUpdate %08lx",this));
	setWakeFrame(getObject(), UPDATE_SLEEP_NONE);

	const ToppleUpdateModuleData* d = getToppleUpdateModuleData();

	if (d->m_killWhenStartToppled)
	{
		setWakeFrame(getObject(), UPDATE_SLEEP_FOREVER);
		getObject()->kill();
		return;
	}

	m_toppleDirection = *toppleDirection;
	m_toppleDirection.normalize();
	TheScriptEngine->adjustToppleDirection(getObject(), &m_toppleDirection);

	m_angularVelocity = toppleSpeed * d->m_initialVelocityPercent;
	m_angularAcceleration = toppleSpeed * d->m_initialAccelPercent;
	m_toppleState = TOPPLE_FALLING;
	m_options = options;

	// tell the drawable to stop swaying
	Drawable * draw = getObject()->getDrawable();
	static NameKeyType nameKeySwayUpdate = NAMEKEY("SwayClientUpdate");

	ClientUpdateModule ** clientModules = draw->getClientUpdateModules();
	if (clientModules)
	{
		while (*clientModules)
		{
			if ((*clientModules)->getModuleNameKey() == nameKeySwayUpdate)
				(*(SwayClientUpdate **)clientModules)->stopSway();

			++clientModules;
		}
	}

	// rotate around the z-axis so that our x-axis is perpendicular to the topple direction.
	// this is really a trick to ensure that relatively planar things (eg, streetlights)
	// fall parallel to the ground, so that they don't up sticking thru the ground.
	// yeah, it assumes the models are constructed appropriately, but is a cheap way
	// of minimizing the problem. (srj)
	Real curAngleX = normalizeAngle(getObject()->getOrientation());
	Real toppleAngle = normalizeAngle(WWMath::Atan2(m_toppleDirection.y, m_toppleDirection.x));
	if (d->m_toppleLeftOrRightOnly)
	{
		// it's a fence or such, and can only topple left or right, so pick the closest
		toppleAngle = angleClosestTo(curAngleX + PI/2, curAngleX - PI/2, toppleAngle);
		m_toppleDirection.x = Cos(toppleAngle);
		m_toppleDirection.y = Sin(toppleAngle);

		// go ahead and remove it from the pathfinder now, rather than waiting for the topple to
		// finish.... since we might be in a slightly different position when toppled, which can
		// confuse the pathfinder and not de-obstacle everything correctly
		TheAI->pathfinder()->removeObjectFromPathfindMap(getObject());

	}
	// desired angle is toppleAngle +/- pi/2, whichever is closer to curangle
	Real desiredAngleX = angleClosestTo(toppleAngle + PI/2, toppleAngle - PI/2, curAngleX);
	m_numAngleDeltaX = REAL_TO_INT_FLOOR(WWMath::Div_Safe(ANGULAR_LIMIT, m_angularVelocity * 2.0f));
	if (m_numAngleDeltaX < 1)
		m_numAngleDeltaX = 1;
	m_angleDeltaX = (desiredAngleX - curAngleX) / m_numAngleDeltaX;

	getObject()->getDrawable()->setModelConditionState(MODELCONDITION_TOPPLED);
	FXList::doFXObj(d->m_toppleFX, getObject());

	// if this is a tree, create a stump
	if (!d->m_stumpName.isEmpty())
	{
		const ThingTemplate* ttn = TheThingFactory->findTemplate(d->m_stumpName);
		Object *stump = TheThingFactory->newObject( ttn, nullptr );
		if (stump)
		{
			stump->setPosition( getObject()->getPosition() );
			stump->setOrientation( getObject()->getOrientation() );
			m_stumpID = stump->getID();

			// if we are "burned", then we will burn our stump
			const Drawable* draw = getObject()->getDrawable();
			if( draw )
			{
				if( draw->getModelConditionFlags().test( MODELCONDITION_BURNED ) == TRUE )
				{
					Drawable* stumpDraw = stump->getDrawable();

					if( stumpDraw )
						stumpDraw->setModelConditionState( MODELCONDITION_BURNED );

				}
			}
		}
	}
}

//-------------------------------------------------------------------------------------------------
///< Ask if this module is able to be toppled
//-------------------------------------------------------------------------------------------------
Bool ToppleUpdate::isAbleToBeToppled() const
{
	return m_toppleState == TOPPLE_UPRIGHT;
}

//-------------------------------------------------------------------------------------------------
static void deathByToppling(Object* obj)
{
	// use a special "topppling" damage type here so that
	// toppled stuff can have different damage/die modules
	// for toppled-death vs other-death
	DamageInfo damageInfo;
	damageInfo.in.m_damageType = DAMAGE_UNRESISTABLE;
	damageInfo.in.m_deathType = DEATH_TOPPLED;
	damageInfo.in.m_sourceID = INVALID_ID;
	damageInfo.in.m_amount = HUGE_DAMAGE_AMOUNT;
	obj->attemptDamage(&damageInfo);
}

//-------------------------------------------------------------------------------------------------
///< Keep track of rotational fall distance, bounce and/or stop when needed.
//-------------------------------------------------------------------------------------------------
UpdateSleepTime ToppleUpdate::update()
{
	//DEBUG_LOG(("updating ToppleUpdate %08lx",this));
	DEBUG_ASSERTCRASH(m_toppleState != TOPPLE_UPRIGHT, ("hmm, we should be sleeping here"));
	if ( (m_toppleState == TOPPLE_UPRIGHT)  ||  (m_toppleState == TOPPLE_DOWN) )
		return UPDATE_SLEEP_FOREVER;

	const ToppleUpdateModuleData* d = getToppleUpdateModuleData();
	const Real VELOCITY_BOUNCE_LIMIT = 0.01f;				// if the velocity after a bounce will be this or lower, just stop at zero
	const Real VELOCITY_BOUNCE_SOUND_LIMIT = 0.03f;	// and if this low, then skip the bounce sound

	Object* obj = getObject();
	if (m_numAngleDeltaX)
	{
		Matrix3D xfrm = *obj->getTransformMatrix();
		xfrm.In_Place_Pre_Rotate_Z(m_angleDeltaX);
		obj->setTransformMatrix(&xfrm);
		--m_numAngleDeltaX;
	}

	Real curVelToUse = m_angularVelocity;
	if (m_angularAccumulation + curVelToUse > ANGULAR_LIMIT)
		curVelToUse = ANGULAR_LIMIT - m_angularAccumulation;

	Matrix3D xfrm = *obj->getTransformMatrix();
	xfrm.In_Place_Pre_Rotate_X(-curVelToUse * m_toppleDirection.y);
	xfrm.In_Place_Pre_Rotate_Y(curVelToUse * m_toppleDirection.x);
	obj->setTransformMatrix(&xfrm);

	m_angularAccumulation += curVelToUse;
	if ((m_angularAccumulation >= ANGULAR_LIMIT) && (m_angularVelocity > 0))
	{
		// Hit so either bounce or stop if too little remaining velocity.
		m_angularVelocity *= -d->m_bounceVelocityPercent;

		if( BitIsSet( m_options, TOPPLE_OPTIONS_NO_BOUNCE ) == TRUE ||
				WWMath::Fabs(m_angularVelocity) < VELOCITY_BOUNCE_LIMIT )
		{
			// too slow, just stop
			m_angularVelocity = 0;
			m_toppleState = TOPPLE_DOWN;

			if (d->m_killWhenToppled)
			{
				deathByToppling(obj);
				if (d->m_reorientToppledRubble)
				{
					// we have a separate rubble state that needs to be upright, and centered
					// on the new "center" pos...
					Vector3 pos;
					pos.X = 0;
					pos.Y = 0;
					pos.Z = obj->getGeometryInfo().getMaxHeightAbovePosition();
					Matrix3D::Transform_Vector(*obj->getTransformMatrix(), pos, &pos);

					Coord3D tmp;
					tmp.x = pos.X;
					tmp.y = pos.Y;
					tmp.z = pos.Z;
					obj->setPosition(&tmp);

					// this relies on the fact that setOrientation always forces us straight up in the Z axis!
					obj->setOrientation(obj->getOrientation());

				}
			}

			if (d->m_killStumpWhenToppled)
			{
				Object* stump = TheGameLogic->findObjectByID(m_stumpID);
				if (stump)
				{
					deathByToppling(stump);
				}
			}
		}
		else if( WWMath::Fabs(m_angularVelocity) >= VELOCITY_BOUNCE_SOUND_LIMIT )
		{
			// fast enough bounce to warrant the bounce fx
			if( BitIsSet( m_options, TOPPLE_OPTIONS_NO_FX ) == FALSE )
				FXList::doFXObj(d->m_bounceFX, obj);
		}
	}
	else
	{
		m_angularVelocity += m_angularAcceleration;
	}

	Drawable *draw=obj->getDrawable();
	if (draw)
		draw->setShadowsEnabled(false);

	return UPDATE_SLEEP_NONE;
}

//-------------------------------------------------------------------------------------------------
/** Do the collision */
//-------------------------------------------------------------------------------------------------
void ToppleUpdate::onCollide( Object *other, const Coord3D *loc, const Coord3D *normal )
{
	// Note that other == null means "collide with ground"
	//
	if (other == nullptr)
		return;

	//@todo JohnA -- Should you get around to adding trees to avoidance pathfinding, then you'll
	//want to change this code:
	//if( other->getCrusherLevel() > getObject()->getCrushableLevel() ) //<----proper tree method
	if( other->getCrusherLevel() > 1 )
	{

		// Give a vector with direction to thing and my speed.
		Coord3D toppleVector = *getObject()->getPosition();
		toppleVector.x -= other->getPosition()->x;
		toppleVector.y -= other->getPosition()->y;
		toppleVector.z = 0;

		Coord3D vel;
		PhysicsBehavior* phys = other->getPhysics();
		if (phys)
			vel = *phys->getVelocity();
		else
			vel.zero();
		getObject()->topple( &toppleVector, vel.length(), TOPPLE_OPTIONS_NONE );

	}

}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void ToppleUpdate::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void ToppleUpdate::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// angular velocity
	xfer->xferReal( &m_angularVelocity );

	// angular acceleration
	xfer->xferReal( &m_angularAcceleration );

	// topple direction
	xfer->xferCoord3D( &m_toppleDirection );

	// topple state
	xfer->xferUser( &m_toppleState, sizeof( ToppleState	) );

	// angular accumulation
	xfer->xferReal( &m_angularAccumulation );

	// angle delta X
	xfer->xferReal( &m_angleDeltaX );

	// num angle delta X
	xfer->xferInt( &m_numAngleDeltaX );

	// do bounce FX
	xfer->xferBool( &m_doBounceFX );

	// options
	xfer->xferUnsignedInt( &m_options );

	// stump id
	xfer->xferObjectID( &m_stumpID );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void ToppleUpdate::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
