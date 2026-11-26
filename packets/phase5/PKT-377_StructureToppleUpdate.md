# Translation Packet `PKT-377`: `StructureToppleUpdate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-377`
- **Work Package:** `WP-5.4` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/update/structure_topple_update.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (22):**
  - `Common/INI.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/INI.h`
  - `Common/RandomValue.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/RandomValue.h`
  - `Common/Thing.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Thing.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/FXList.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/FXList.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameClient/ParticleSys.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/ParticleSys.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/BoneFXUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BoneFXUpdate.h`
  - `GameLogic/Module/DieModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DieModule.h`
  - `GameLogic/Module/StructureToppleUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/StructureToppleUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/ObjectCreationList.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ObjectCreationList.h`
  - `GameLogic/ScriptEngine.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ScriptEngine.h`
  - `GameLogic/Weapon.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h`
  - `Lib/BaseType.h`
  - `PreRTS.h`
- **Implemented Module Facets (1):**
  - `DieModuleInterface* getDie()`
- **INI `FieldParse` Table Entries (15):**
  - `"MinToppleDelay" => INI::parseDurationUnsignedInt`
  - `"MaxToppleDelay" => INI::parseDurationUnsignedInt`
  - `"MinToppleBurstDelay" => INI::parseDurationUnsignedInt`
  - `"MaxToppleBurstDelay" => INI::parseDurationUnsignedInt`
  - `"StructuralIntegrity" => INI::parseReal`
  - `"StructuralDecay" => INI::parseReal`
  - `"DamageFXTypes" => INI::parseDamageTypeFlags`
  - `"TopplingFX" => INI::parseFXList`
  - `"ToppleDelayFX" => INI::parseFXList`
  - `"ToppleStartFX" => INI::parseFXList`
  - `"ToppleDoneFX" => INI::parseFXList`
  - `"CrushingFX" => INI::parseFXList`
  - `"CrushingWeaponName" => INI::parseAsciiString`
  - `"OCL" => parseOCL`
  - `"AngleFX" => parseAngleFX`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (10 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUnsignedInt( &m_toppleFrame )`
  3. `xferCoord2D( &m_toppleDirection )`
  4. `xferUser( &m_toppleState, sizeof( StructureToppleStateType ) )`
  5. `xferReal( &m_toppleVelocity )`
  6. `xferReal( &m_accumulatedAngle )`
  7. `xferReal( &m_structuralIntegrity )`
  8. `xferReal( &m_lastCrushedLocation )`
  9. `xferInt( &m_nextBurstFrame )`
  10. `xferCoord3D( &m_delayBurstLocation )`
- **PRNG Calls (7 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValue(d->m_minToppleDelay, d->m_maxToppleDelay)`
  - `GameLogicRandomValueReal(0.0, 2*PI)`
  - `GameLogicRandomValueReal(-PI/8, PI/8)`
  - `GameClientRandomValue(d->m_minToppleBurstDelay, d->m_maxToppleBurstDelay)`
  - `GameClientRandomValue(d->m_minToppleBurstDelay, d->m_maxToppleBurstDelay)`
  - `GameClientRandomValue(d->m_minToppleBurstDelay, d->m_maxToppleBurstDelay)`
  - `GameLogicRandomValue(0, range-1)`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/StructureToppleUpdate.h`
```cpp
// FILE: StructureToppleUpdate.h /////////////////////////////////////////////////////////////////////////
// Author: Bryan Cleveland, March 2002
// Desc:   Update that will topple over a building and crush and destroy stuff that it hits.
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Module/DieModule.h"
#include "Lib/BaseType.h"
#include "Common/RandomValue.h"
#include "GameClient/ParticleSys.h"

class FXList;
class ObjectCreationList;

typedef std::vector<const ObjectCreationList*> OCLVec;

//-------------------------------------------------------------------------------------------------
struct FXBoneInfo
{
	AsciiString boneName;
	const ParticleSystemTemplate* particleSystemTemplate;
};

typedef std::vector<FXBoneInfo> FXBoneInfoVector;

//-------------------------------------------------------------------------------------------------
struct AngleFXInfo
{
	Real angle;
	FXList *fxList;
};

typedef std::vector<AngleFXInfo> AngleFXInfoVector;

//-------------------------------------------------------------------------------------------------
enum StructureTopplePhaseType CPP_11(: Int)
{
	STPHASE_INITIAL = 0,
	STPHASE_DELAY,
	STPHASE_FINAL,

	ST_PHASE_COUNT
};

static const char *const TheStructureTopplePhaseNames[] =
{
	"INITIAL",
	"DELAY",
	"FINAL",

	nullptr
};
static_assert(ARRAY_SIZE(TheStructureTopplePhaseNames) == ST_PHASE_COUNT + 1, "Incorrect array size");

//-------------------------------------------------------------------------------------------------
class StructureToppleUpdateModuleData : public UpdateModuleData
{
public:
	DieMuxData m_dieMuxData;
	Int m_minToppleDelay;
	Int m_maxToppleDelay;
	Real m_structuralIntegrity;
	Real m_structuralDecay;
	DamageTypeFlags m_damageFXTypes;		///< flags used to play or not play the effects
	FXList *m_toppleStartFXList;
	FXList *m_toppleDelayFXList;
	FXList *m_toppleFXList;
	FXList *m_toppleDoneFXList;
	FXList *m_crushingFXList;
	AsciiString	m_crushingWeaponName;
	Int m_minToppleBurstDelay;
	Int m_maxToppleBurstDelay;
	OCLVec m_ocls[ST_PHASE_COUNT];
	UnsignedInt m_oclCount[ST_PHASE_COUNT];
	FXBoneInfoVector fxbones;			///< Bone names and attached particle systems.
	AngleFXInfoVector angleFX;


	StructureToppleUpdateModuleData()
	{
		m_minToppleDelay = 0;
		m_maxToppleDelay = 0;
		m_minToppleBurstDelay = 0;
		m_maxToppleBurstDelay = 0;
		m_structuralIntegrity = 0.1f;
		m_structuralDecay = 0.0f;
		m_damageFXTypes = DAMAGE_TYPE_FLAGS_NONE;
		m_damageFXTypes.flip();
		m_toppleStartFXList = nullptr;
		m_toppleDelayFXList = nullptr;
		m_toppleDoneFXList = nullptr;
		m_toppleFXList = nullptr;
		m_crushingFXList = nullptr;

		for (int i = 0; i < ST_PHASE_COUNT; ++i)
		{
			// init to one, so that if these are omitted, we choose exactly one of each.
			m_oclCount[i] = 1;
		}
		fxbones.clear();
		angleFX.clear();
	}

	static void buildFieldParse(MultiIniFieldParse& p);

};


//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class StructureToppleUpdate : public UpdateModule,
															public DieModuleInterface
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( StructureToppleUpdate, "StructureToppleUpdate" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( StructureToppleUpdate, StructureToppleUpdateModuleData )

public:

	StructureToppleUpdate( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	static Int getInterfaceMask() { return UpdateModule::getInterfaceMask() | (MODULEINTERFACE_DIE); }

	// BehaviorModule
	virtual DieModuleInterface* getDie() override { return this; }

	// UpdateModuleInterface
	virtual UpdateSleepTime update() override;

	// DieModuleInterface
	virtual void onDie( const DamageInfo *damageInfo ) override;

protected:

	void beginStructureTopple( const DamageInfo *damageInfo );

	void applyCrushingDamage(Real theta);
	void doDamageLine(Object *building, const WeaponTemplate* wt, Real jcos, Real jsin, Real facingWidth, Real toppleAngle);
	void doToppleStartFX(Object *building, const DamageInfo *damageInfo);
	void doToppleDelayBurstFX();

	void doToppleDoneStuff();
	void doAngleFX(Real curAngle, Real newAngle);

	void doPhaseStuff(StructureTopplePhaseType stphase, const Coord3D *target);

	enum StructureToppleStateType {
		TOPPLESTATE_STANDING,
		TOPPLESTATE_WAITINGFORTOPPLESTART,
		TOPPLESTATE_TOPPLING,
		TOPPLESTATE_WAITINGFORDONE,
		TOPPLESTATE_DONE
	};

	UnsignedInt m_toppleFrame;
	Coord2D m_toppleDirection;
	StructureToppleStateType m_toppleState;
	Real m_toppleVelocity;
	Real m_accumulatedAngle;
	Real m_structuralIntegrity;
	Real m_lastCrushedLocation;
	Int m_nextBurstFrame;
	Coord3D m_delayBurstLocation;
	Real m_buildingHeight;

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/StructureToppleUpdate.cpp`
```cpp
// FILE: StructureToppleUpdate.cpp ///////////////////////////////////////////////////////////////////////
// Author:
// Desc:
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/Thing.h"
#include "Common/ThingTemplate.h"
#include "Common/INI.h"
#include "Common/RandomValue.h"
#include "Common/Xfer.h"
#include "GameClient/FXList.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/BoneFXUpdate.h"
#include "GameLogic/Module/StructureToppleUpdate.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectCreationList.h"
#include "GameLogic/ScriptEngine.h"
#include "GameLogic/Weapon.h"
#include "GameClient/Drawable.h"
#include "GameClient/InGameUI.h"



const Int MAX_IDX = 32;

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
StructureToppleUpdate::StructureToppleUpdate( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{
	m_delayBurstLocation.zero();
	m_structuralIntegrity = 0.0f;
	m_toppleDirection.x = m_toppleDirection.y = 0;
	m_toppleFrame = 0;
	m_toppleState = TOPPLESTATE_STANDING;
	m_toppleVelocity = 0.0f;
	m_accumulatedAngle = 0.001f; // Need to give it a little nudge in the right direction
	m_lastCrushedLocation = 0.0f;
	m_nextBurstFrame = -1;
	setWakeFrame(getObject(), UPDATE_SLEEP_FOREVER);

	//Get the extent height here, rather than after it dies -- when it switches to dead state
	//the rubble state has a tiny height.
	Object *building = getObject();
	m_buildingHeight = building->getGeometryInfo().getMaxHeightAbovePosition();
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
StructureToppleUpdate::~StructureToppleUpdate()
{
}

//-------------------------------------------------------------------------------------------------
static void parseOCL( INI* ini, void *instance, void * /*store*/, const void* /*userData*/ )
{
	StructureToppleUpdateModuleData* self = (StructureToppleUpdateModuleData*)instance;
	StructureTopplePhaseType stphase = (StructureTopplePhaseType)INI::scanIndexList(ini->getNextToken(), TheStructureTopplePhaseNames);
	for (const char* token = ini->getNextToken(); token; token = ini->getNextTokenOrNull())
	{
		const ObjectCreationList *ocl = TheObjectCreationListStore->findObjectCreationList(token);	// could be null! this is OK!
		self->m_ocls[stphase].push_back(ocl);
	}
}

//-------------------------------------------------------------------------------------------------
static void parseAngleFX(INI* ini, void *instance, void * /* store */, const void * /*userData*/)
{
	StructureToppleUpdateModuleData* self = (StructureToppleUpdateModuleData*)instance;
	AngleFXInfo info;
	INI::parseReal(ini, instance, &(info.angle), nullptr);
	info.angle = info.angle * PI / 180.0f; // convert from degrees to radians.
	INI::parseFXList(ini, instance, &(info.fxList), nullptr);
	self->angleFX.push_back(info);
}

//-------------------------------------------------------------------------------------------------
/*static*/ void StructureToppleUpdateModuleData::buildFieldParse(MultiIniFieldParse& p)
{
  UpdateModuleData::buildFieldParse(p);

	static const FieldParse dataFieldParse[] =
	{
		{ "MinToppleDelay",						INI::parseDurationUnsignedInt,		nullptr, offsetof( StructureToppleUpdateModuleData, m_minToppleDelay ) },
		{ "MaxToppleDelay",						INI::parseDurationUnsignedInt,		nullptr, offsetof( StructureToppleUpdateModuleData, m_maxToppleDelay ) },
		{ "MinToppleBurstDelay",			INI::parseDurationUnsignedInt,		nullptr, offsetof( StructureToppleUpdateModuleData, m_minToppleBurstDelay ) },
		{ "MaxToppleBurstDelay",			INI::parseDurationUnsignedInt,		nullptr, offsetof( StructureToppleUpdateModuleData, m_maxToppleBurstDelay ) },
		{ "StructuralIntegrity",			INI::parseReal,										nullptr, offsetof( StructureToppleUpdateModuleData, m_structuralIntegrity ) },
		{ "StructuralDecay",					INI::parseReal,										nullptr, offsetof( StructureToppleUpdateModuleData, m_structuralDecay ) },
		{ "DamageFXTypes",						INI::parseDamageTypeFlags,				nullptr, offsetof( StructureToppleUpdateModuleData, m_damageFXTypes ) },
		{ "TopplingFX",								INI::parseFXList,									nullptr, offsetof( StructureToppleUpdateModuleData, m_toppleFXList ) },
		{ "ToppleDelayFX",						INI::parseFXList,									nullptr, offsetof( StructureToppleUpdateModuleData, m_toppleDelayFXList ) },
		{ "ToppleStartFX",						INI::parseFXList,									nullptr, offsetof( StructureToppleUpdateModuleData, m_toppleStartFXList ) },
		{ "ToppleDoneFX",							INI::parseFXList,									nullptr, offsetof( StructureToppleUpdateModuleData, m_toppleDoneFXList ) },
		{ "CrushingFX",								INI::parseFXList,									nullptr, offsetof( StructureToppleUpdateModuleData, m_crushingFXList ) },
		{ "CrushingWeaponName",				INI::parseAsciiString,						nullptr, offsetof( StructureToppleUpdateModuleData, m_crushingWeaponName ) },
		{ "OCL",											parseOCL,													nullptr, 0 },
		{ "AngleFX",									parseAngleFX,											nullptr, 0 },
		{ nullptr, nullptr, nullptr, 0 }
	};
  p.add(dataFieldParse);
	p.add(DieMuxData::getFieldParse(), offsetof( StructureToppleUpdateModuleData, m_dieMuxData ));
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void StructureToppleUpdate::beginStructureTopple(const DamageInfo *damageInfo)
{
	const StructureToppleUpdateModuleData *d = getStructureToppleUpdateModuleData();

	if (d)
	{
		UnsignedInt now = TheGameLogic->getFrame();
		m_toppleFrame = now + GameLogicRandomValue(d->m_minToppleDelay, d->m_maxToppleDelay);

		Object *attacker = TheGameLogic->findObjectByID(damageInfo->in.m_sourceID);
		Object *building = getObject();

		Real toppleAngle = 0.0;
		if (attacker == nullptr) {
			toppleAngle = GameLogicRandomValueReal(0.0, 2*PI);
		} else {
			const Coord3D *attackerPos = attacker->getPosition();
			const Coord3D *buildingPos = building->getPosition();

			// Calculate the topple direction to be the opposite of the direction fired from.
			m_toppleDirection.x = buildingPos->x - attackerPos->x;
			m_toppleDirection.y = buildingPos->y - attackerPos->y;

			// Give it a little randomness...
			toppleAngle = m_toppleDirection.toAngle();
			toppleAngle += GameLogicRandomValueReal(-PI/8, PI/8);
		}
		m_toppleDirection.x = Cos(toppleAngle);
		m_toppleDirection.y = Sin(toppleAngle);
		TheScriptEngine->adjustToppleDirection(getObject(), &m_toppleDirection);

		Real averageRadius = (building->getGeometryInfo().getMajorRadius() + building->getGeometryInfo().getMinorRadius()) / 2;
		Real explosionRadius = averageRadius * 0.90;

		m_delayBurstLocation.x = building->getPosition()->x + explosionRadius * Cos(toppleAngle);
		m_delayBurstLocation.y = building->getPosition()->y + explosionRadius * Sin(toppleAngle);
		m_delayBurstLocation.z = TheTerrainLogic->getGroundHeight(m_delayBurstLocation.x, m_delayBurstLocation.y);

		doToppleStartFX(building, damageInfo);
		m_nextBurstFrame = now + GameClientRandomValue(d->m_minToppleBurstDelay, d->m_maxToppleBurstDelay);

		m_toppleState = TOPPLESTATE_WAITINGFORTOPPLESTART;

		setWakeFrame(getObject(), UPDATE_SLEEP_NONE);
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void StructureToppleUpdate::onDie( const DamageInfo *damageInfo )
{
	const StructureToppleUpdateModuleData* d = getStructureToppleUpdateModuleData();
	if (!d->m_dieMuxData.isDieApplicable(getObject(), damageInfo))
		return;

	AIUpdateInterface *ai = getObject()->getAIUpdateInterface();
	if (ai)
		ai->markAsDead();

	// Deselect the object for all players.
	TheGameLogic->deselectObject(getObject(), PLAYERMASK_ALL, TRUE);

	beginStructureTopple(damageInfo);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
UpdateSleepTime StructureToppleUpdate::update()
{
	static const Real TOPPLE_ACCELERATION_FACTOR = 0.02f;

	const StructureToppleUpdateModuleData *d = getStructureToppleUpdateModuleData();

	if (m_toppleState == TOPPLESTATE_STANDING)
	{
		DEBUG_CRASH(("hmm, what?"));
		return UPDATE_SLEEP_FOREVER;
	}

	// get last damage info
	const DamageInfo *lastDamageInfo = getObject()->getBodyModule()->getLastDamageInfo();

	// We are in the dramatic pause between when the building has lost all its hit points and
	// when it starts toppling over.
	if (m_toppleState == TOPPLESTATE_WAITINGFORTOPPLESTART) {
		UnsignedInt now = TheGameLogic->getFrame();
		if (now >= m_nextBurstFrame) {
			doToppleDelayBurstFX();
			// This uses a game client random value because the delay bursts are purely visual and aural effects.
			m_nextBurstFrame = now + GameClientRandomValue(d->m_minToppleBurstDelay, d->m_maxToppleBurstDelay);
		}
		if (now >= m_toppleFrame) {
			m_toppleState = TOPPLESTATE_TOPPLING;
			m_structuralIntegrity = d->m_structuralIntegrity;
		}
	}

	// The building is in the process of falling over.
	if (m_toppleState == TOPPLESTATE_TOPPLING) {
		UnsignedInt now = TheGameLogic->getFrame();
		Real toppleAcceleration = TOPPLE_ACCELERATION_FACTOR * (Sin(m_accumulatedAngle) * (1.0 - m_structuralIntegrity));
//		DEBUG_LOG(("toppleAcceleration = %f", toppleAcceleration));
		m_toppleVelocity += toppleAcceleration;
//		DEBUG_LOG(("m_toppleVelocity = %f", m_toppleVelocity));

		// doesn't make sense to have a structural integrity less than zero.
		if (m_structuralIntegrity > 0.0f) {
			m_structuralIntegrity *= d->m_structuralDecay;
			if (m_structuralIntegrity < 0.0f) {
				m_structuralIntegrity = 0.0f;
			}
		}
//		DEBUG_LOG(("m_structuralIntegrity = %f", m_structuralIntegrity));

		doAngleFX(m_accumulatedAngle, m_accumulatedAngle + m_toppleVelocity);

		m_accumulatedAngle += m_toppleVelocity;

		applyCrushingDamage(PI/2 - m_accumulatedAngle);

		if (m_accumulatedAngle >= PI/2) {
			m_toppleVelocity -= m_accumulatedAngle - PI/2;
			m_accumulatedAngle = PI/2;
			m_toppleState = TOPPLESTATE_WAITINGFORDONE;

			applyCrushingDamage(0.0f);
			doPhaseStuff(STPHASE_FINAL, getObject()->getPosition());

			if( lastDamageInfo == nullptr || getDamageTypeFlag( d->m_damageFXTypes, lastDamageInfo->in.m_damageType ) )
				FXList::doFXObj(d->m_toppleDoneFXList, getObject());

			m_toppleFrame = TheGameLogic->getFrame();
		}

		if (now >= m_nextBurstFrame) {
			doToppleDelayBurstFX();
			// This uses a game client random value because the delay bursts are purely visual and aural effects.
			m_nextBurstFrame = now + GameClientRandomValue(d->m_minToppleBurstDelay, d->m_maxToppleBurstDelay);
		}

		Object *building = getObject();
		Matrix3D xfrm = *building->getTransformMatrix();
		xfrm.In_Place_Pre_Rotate_X(-m_toppleVelocity * m_toppleDirection.y);
		xfrm.In_Place_Pre_Rotate_Y(m_toppleVelocity * m_toppleDirection.x);
		building->setTransformMatrix(&xfrm);
	}

	// The building is now flat on the ground and done with all the crushing and all that.
	if (m_toppleState == TOPPLESTATE_WAITINGFORDONE)
	{
		if (m_toppleFrame <= TheGameLogic->getFrame())
		{
			Object *building = getObject();
			Drawable *drawable = building->getDrawable();
			drawable->clearModelConditionState(MODELCONDITION_RUBBLE);
			drawable->setModelConditionState(MODELCONDITION_POST_COLLAPSE);


			// Need to update body particle systems, now
			BodyModuleInterface *body = building->getBodyModule();
			body->updateBodyParticleSystems();

			doToppleDoneStuff();

			m_toppleState = TOPPLESTATE_DONE;

			return UPDATE_SLEEP_FOREVER;
		}
	}

	return UPDATE_SLEEP_NONE;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void StructureToppleUpdate::doToppleDoneStuff()
{
	static NameKeyType key_BoneFXUpdate = NAMEKEY("BoneFXUpdate");
	BoneFXUpdate *bfxu = (BoneFXUpdate *)getObject()->findUpdateModule(key_BoneFXUpdate);
	if (bfxu != nullptr) {
		bfxu->stopAllBoneFX();
	}

	Object *building = getObject();

	Real origAngle = building->getOrientation();
	building->setOrientation(origAngle);

	Real toppleAngle = m_toppleDirection.toAngle();

	Matrix3D xfrm = *building->getTransformMatrix();
	xfrm.In_Place_Pre_Rotate_Z(toppleAngle-origAngle);
	building->setTransformMatrix(&xfrm);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void StructureToppleUpdate::doAngleFX(Real curAngle, Real newAngle)
{
	const StructureToppleUpdateModuleData *d = getStructureToppleUpdateModuleData();
	const DamageInfo *lastDamageInfo = getObject()->getBodyModule()->getLastDamageInfo();

	for (std::vector<AngleFXInfo>::const_iterator it = d->angleFX.begin(); it != d->angleFX.end(); ++it)
	{
		if ((it->angle > curAngle) && (it->angle <= newAngle))
		{
			if( lastDamageInfo == nullptr || getDamageTypeFlag( d->m_damageFXTypes, lastDamageInfo->in.m_damageType ) )
				FXList::doFXObj(it->fxList, getObject());
		}
	}

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
// theta is the angle of the building with respect to the ground.
void StructureToppleUpdate::applyCrushingDamage(Real theta)
{
//	static const Real THETA_CEILING = PI/8; // This weapon won't do any damage until theta is less than this value.
	static const Real THETA_CEILING = PI/6; // This weapon won't do any damage until theta is less than this value.
	static const Real WEAPON_SPACING_PERPENDICULAR = 25;	// The spacing between weapon firing locations,
																												// distance is perpendicular to the direction the
																												// building is falling in.

	const StructureToppleUpdateModuleData *d = getStructureToppleUpdateModuleData();
	if (theta > THETA_CEILING) {
		return;
	}

	Object *building = getObject();
	Real orientationAngle = building->getOrientation();
	Real toppleAngle = m_toppleDirection.toAngle();

	// Figure out the width of the projection of the boundary of the building along the topple direction.
	// Do this because the amount of ground that is affected will be different if the building falls
	// in different orientations.
	Real angle = orientationAngle - toppleAngle;
	Real minorComponent = building->getGeometryInfo().getMinorRadius() * Cos(angle);
	Real majorComponent = building->getGeometryInfo().getMajorRadius() * Sin(angle);

	Coord3D temp3D;
	temp3D.x = majorComponent;
	temp3D.y = minorComponent;
	temp3D.z = 0.0f;

	Real facingWidth = temp3D.length() / 2;

	// Get the crushing weapon.
	const WeaponTemplate* wt = TheWeaponStore->findWeaponTemplate(d->m_crushingWeaponName);
	if (wt == nullptr) {
		return;
	}

	// The furthest away from the base of the building to explode on.
	Real maxDistance = m_buildingHeight * (1.0 - Sin(theta));

	/*
	 * Fire explosions at regular intervals across the area that the building is currently
	 * crushing.  The explosions occur across the face and along the length of the building.
	 */
	Real jcos;
	Real jsin;
//	Coord3D target;
	Real j = m_lastCrushedLocation;
	for (; j < maxDistance; j += WEAPON_SPACING_PERPENDICULAR) {
		jcos = j * Cos(toppleAngle);
		jsin = j * Sin(toppleAngle);
		doDamageLine(building, wt, jcos, jsin, facingWidth, toppleAngle);
	}


	jcos = maxDistance * Cos(toppleAngle);
	jsin = maxDistance * Sin(toppleAngle);
	doDamageLine(building, wt, jcos, jsin, facingWidth, toppleAngle);

	m_lastCrushedLocation = j;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void StructureToppleUpdate::doDamageLine(Object *building, const WeaponTemplate* wt, Real jcos, Real jsin, Real facingWidth, Real toppleAngle)
{
	const DamageInfo *lastDamageInfo = getObject()->getBodyModule()->getLastDamageInfo();
	static const Real WEAPON_SPACING_PARALLEL = 25;				// The spacing between weapon firing locations,
																												// distance is parallel to the direction the building
																												// is falling in.

	const StructureToppleUpdateModuleData *d = getStructureToppleUpdateModuleData();

	Coord3D target;

	for (Real i = -facingWidth; i < facingWidth; i += WEAPON_SPACING_PARALLEL)
	{
		target.x = building->getPosition()->x + jcos + (i * Sin(toppleAngle));
		target.y = building->getPosition()->y + jsin + (i * Cos(toppleAngle));
		target.z = TheTerrainLogic->getGroundHeight(target.x, target.y);

	  TheWeaponStore->createAndFireTempWeapon(wt, building, &target);

		// do the crushing particle effects
		if( lastDamageInfo == nullptr || getDamageTypeFlag( d->m_damageFXTypes, lastDamageInfo->in.m_damageType ) )
			FXList::doFXPos(d->m_crushingFXList, &target);
	}

	// Make sure there are weapons fired and FX done on the edge of the building.
	target.x = building->getPosition()->x + jcos + (facingWidth * Sin(toppleAngle));
	target.y = building->getPosition()->y + jsin + (facingWidth * Cos(toppleAngle));
	target.z = TheTerrainLogic->getGroundHeight(target.x, target.y);

  TheWeaponStore->createAndFireTempWeapon(wt, building, &target);

	// do the crushing particle effects
	if( lastDamageInfo == nullptr || getDamageTypeFlag( d->m_damageFXTypes, lastDamageInfo->in.m_damageType ) )
		FXList::doFXPos(d->m_crushingFXList, &target);

	// Do the flying debris for this line.
	target.x = building->getPosition()->x + jcos;
	target.y = building->getPosition()->y + jsin;
	target.z = TheTerrainLogic->getGroundHeight(target.x, target.y);

	doPhaseStuff(STPHASE_FINAL, &target);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void StructureToppleUpdate::doToppleStartFX(Object *building, const DamageInfo *damageInfo)
{
	const StructureToppleUpdateModuleData *d = getStructureToppleUpdateModuleData();
	const DamageInfo *lastDamageInfo = getObject()->getBodyModule()->getLastDamageInfo();

	if( lastDamageInfo == nullptr || getDamageTypeFlag( d->m_damageFXTypes, lastDamageInfo->in.m_damageType ) )
		FXList::doFXPos(d->m_toppleStartFXList, building->getPosition());

	doPhaseStuff(STPHASE_INITIAL, building->getPosition());
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void StructureToppleUpdate::doToppleDelayBurstFX()
{
	const StructureToppleUpdateModuleData *d = getStructureToppleUpdateModuleData();
	const DamageInfo *lastDamageInfo = getObject()->getBodyModule()->getLastDamageInfo();

	DEBUG_LOG(("Doing topple delay burst on frame %d", TheGameLogic->getFrame()));
	if( lastDamageInfo == nullptr || getDamageTypeFlag( d->m_damageFXTypes, lastDamageInfo->in.m_damageType ) )
		FXList::doFXPos(d->m_toppleDelayFXList, &m_delayBurstLocation);

	Object *building = getObject();
	Drawable *drawable = building->getDrawable();

	if( lastDamageInfo == nullptr || getDamageTypeFlag( d->m_damageFXTypes, lastDamageInfo->in.m_damageType ) )
	{

		for (std::vector<FXBoneInfo>::const_iterator it = d->fxbones.begin(); it != d->fxbones.end(); ++it)
		{
			ParticleSystem *sys = TheParticleSystemManager->createParticleSystem(it->particleSystemTemplate);
			if (sys != nullptr)
			{
				Coord3D pos;
				if (drawable->getPristineBonePositions(it->boneName.str(), 0, &pos, nullptr, 1) == 1)
				{
					// got the bone position...
					sys->setPosition(&pos);

					// Attach it to the object...
					sys->attachToDrawable(drawable);
				}
			}
		}
	}

	doPhaseStuff(STPHASE_DELAY, &m_delayBurstLocation);

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
inline Bool inList(Int value, Int count, const Int idxList[])
{
	for (Int j = 0; j < count; ++j)
	{
		if (idxList[j] == value)
			return true;
	}
	return false;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
static void buildNonDupRandomIndexList(Int range, Int count, Int idxList[])
{
	for (Int i = 0; i < count; ++i)
	{
		Int idx;
		do
		{
			idx = GameLogicRandomValue(0, range-1);
		}
		while (inList(idx, i, idxList));
		idxList[i] = idx;
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void StructureToppleUpdate::doPhaseStuff(StructureTopplePhaseType stphase, const Coord3D *target)
{
	const StructureToppleUpdateModuleData* d = getStructureToppleUpdateModuleData();
	Int i, idx, count, listSize;
	Int idxList[MAX_IDX];

	listSize = d->m_ocls[stphase].size();
	if (listSize > 0)
	{
		count = d->m_oclCount[stphase];
		buildNonDupRandomIndexList(listSize, count, idxList);
		for (i = 0; i < count; ++i)
		{
			idx = idxList[i];
			const OCLVec& v = d->m_ocls[stphase];
			DEBUG_ASSERTCRASH(idx>=0&&idx<v.size(),("bad idx"));
			const ObjectCreationList* ocl = v[idx];
			ObjectCreationList::create(ocl, getObject(), target, nullptr, INVALID_ANGLE );
		}
	}
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void StructureToppleUpdate::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void StructureToppleUpdate::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// topple frame
	xfer->xferUnsignedInt( &m_toppleFrame );

	// topple direction
	xfer->xferCoord2D( &m_toppleDirection );

	// topple state
	xfer->xferUser( &m_toppleState, sizeof( StructureToppleStateType ) );

	// topple velocity
	xfer->xferReal( &m_toppleVelocity );

	// accumulated angle
	xfer->xferReal( &m_accumulatedAngle );

	// structural integrity
	xfer->xferReal( &m_structuralIntegrity );

	// last crushed location
	xfer->xferReal( &m_lastCrushedLocation );

	// next burst frame
	xfer->xferInt( &m_nextBurstFrame );

	// delay burst location
	xfer->xferCoord3D( &m_delayBurstLocation );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void StructureToppleUpdate::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
