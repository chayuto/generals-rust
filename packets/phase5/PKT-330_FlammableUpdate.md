# Translation Packet `PKT-330`: `FlammableUpdate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-330`
- **Work Package:** `WP-5.4` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/update/flammable_update.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (11):**
  - `Common/AudioEventRTS.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/AudioEventRTS.h`
  - `Common/GameAudio.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameAudio.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/DamageModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DamageModule.h`
  - `GameLogic/Module/FireSpreadUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/FireSpreadUpdate.h`
  - `GameLogic/Module/FlammableUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/FlammableUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `PreRTS.h`
- **Implemented Module Facets (1):**
  - `DamageModuleInterface* getDamage()`
- **INI `FieldParse` Table Entries (7):**
  - `"BurnedDelay" => INI::parseDurationUnsignedInt`
  - `"AflameDuration" => INI::parseDurationUnsignedInt`
  - `"AflameDamageDelay" => INI::parseDurationUnsignedInt`
  - `"AflameDamageAmount" => INI::parseInt`
  - `"BurningSoundName" => INI::parseAsciiString`
  - `"FlameDamageLimit" => INI::parseReal`
  - `"FlameDamageExpiration" => INI::parseDurationUnsignedInt`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (8 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUser( &m_status, sizeof( FlammabilityStatusType ) )`
  3. `xferUnsignedInt( &m_aflameEndFrame )`
  4. `xferUnsignedInt( &m_burnedEndFrame )`
  5. `xferUnsignedInt( &m_damageEndFrame )`
  6. `xferReal( &m_flameDamageLimit )`
  7. `xferUnsignedInt( &m_lastFlameDamageDealt )`
  8. `xferObjectID(&m_flameSource)`
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if RETAIL_COMPATIBLE_CRC || PRESERVE_NO_XP_FROM_FLAME_KILLS`
  - `#if RETAIL_COMPATIBLE_XFER_SAVE`

## 2.5 Existing Rust Symbol Index (Use These Exact Paths)
- `zh_logic::modules::update::base_renerate_update`: `BaseRegenerateUpdateModuleData`, `BaseRegenerateUpdate`
- `zh_logic::modules::update::deletion_update`: `DeletionUpdateModuleData`, `DeletionUpdate`
- `zh_logic::modules::update::dynamic_geometry_info_update`: `DIRECTION_FORWARD`, `DynamicGeometryInfoUpdateModuleData`, `DynamicGeometryInfoUpdate`
- `zh_logic::modules::update::dynamic_shroud_clearing_range_update`: `DynamicShroudClearingRangeUpdateModuleData`, `DscruState`, `DynamicShroudClearingRangeUpdate`
- `zh_logic::modules::update::fire_weapon_update`: `FireWeaponUpdateModuleData`, `FireWeaponUpdate`
- `zh_logic::modules::update::float_update`: `FloatUpdateModuleData`, `FloatUpdate`
- `zh_logic::modules::update::height_die_update`: `HeightDieUpdateModuleData`, `HeightDieUpdate`
- `zh_logic::modules::update::lifetime_update`: `LifetimeUpdateModuleData`, `LifetimeUpdate`
- `zh_logic::modules::update::projectile_stream_update`: `MAX_PROJECTILE_STREAM`, `ProjectileStreamUpdateModuleData`, `ProjectileStreamUpdate`
- `zh_logic::modules::update::prone_update`: `ProneUpdateModuleData`, `ProneUpdate`
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
- `zh_logic::modules::body::active_body`: `ActiveBodyModuleData`, `fn`, `ActiveBody`
- `zh_logic::modules::body::body_module`: `BodyModuleData`, `BodyModuleBase`, `fn`, `BodyModule`
- `zh_logic::modules::body::highlander_body`: `HighlanderBody`
- `zh_logic::modules::body::hive_structure_body`: `HiveStructureBodyModuleData`, `fn`, `HiveStructureBody`
- `zh_logic::modules::body::immortal_body`: `ImmortalBody`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/FlammableUpdate.h`
```cpp
// FILE: FlammableUpdate.h /////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, April 2002
// Desc:   Update that manages Aflame and Burned statuses and their effects
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/AudioEventRTS.h"
#include "GameLogic/Module/DamageModule.h"
#include "GameLogic/Module/UpdateModule.h"


//-------------------------------------------------------------------------------------------------
enum FlammabilityStatusType CPP_11(: Int)
{
	// These show the state I last noticed my object was in.
	FS_NORMAL = 0,
	FS_AFLAME,
	FS_BURNED,
};

//-------------------------------------------------------------------------------------------------
class FlammableUpdateModuleData : public UpdateModuleData
{
public:
	UnsignedInt		m_burnedDelay;	///< How long before I am ::Burned.  0 means never
	UnsignedInt		m_aflameDuration; ///< How long I stay ::Aflame.  Independent of Burned.
	// When aflame wears out is when I check to be normal or burned,  So my model can
	// change to burned while I am still aflame.
	UnsignedInt		m_aflameDamageDelay;	///< While ::Aflame, I take damage this often.  If 0, never.
	Int						m_aflameDamageAmount;	///< And this is how much I take.
	AsciiString		m_burningSoundName;			///< Sound to loop-play while burning (Not an AudioEventRTS here, since that belongs to the module)
	Real					m_flameDamageLimitData;
	UnsignedInt		m_flameDamageExpirationDelay;

	FlammableUpdateModuleData();

	static void buildFieldParse(MultiIniFieldParse& p);

private:

};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class FlammableUpdate : public UpdateModule, public DamageModuleInterface
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( FlammableUpdate, "FlammableUpdate" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( FlammableUpdate, FlammableUpdateModuleData )

public:

	FlammableUpdate( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	static Int getInterfaceMask() { return UpdateModule::getInterfaceMask() | (MODULEINTERFACE_DAMAGE); }
	virtual DamageModuleInterface* getDamage() override { return this; }

	void tryToIgnite(); ///< FlammabeDamage uses this.  It is up to me to decide if I am burnable
	Bool wouldIgnite(); ///< Since we need to cheat sometimes and light something directly, ask if this would light

	//UpdateModuleInterface
	virtual UpdateSleepTime update() override;

	//DamageModuleInterface
	virtual void onDamage( DamageInfo *damageInfo ) override;
	virtual void onHealing( DamageInfo *damageInfo ) override { }
	virtual void onBodyDamageStateChange( const DamageInfo *damageInfo,
																				BodyDamageType oldState,
																				BodyDamageType newState ) override { }

protected:

	UpdateSleepTime calcSleepTime();
	void doAflameDamage();
	void startBurningSound();
	void stopBurningSound();

	FlammabilityStatusType	m_status;
	UnsignedInt							m_aflameEndFrame;
	UnsignedInt							m_burnedEndFrame;
	UnsignedInt							m_damageEndFrame;
	AudioHandle							m_audioHandle;
	Real										m_flameDamageLimit;
	ObjectID								m_flameSource;
	UnsignedInt							m_lastFlameDamageDealt;
};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/FlammableUpdate.cpp`
```cpp
// FILE: FlammableUpdate.cpp /////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, April 2002
// Desc:   Update that manages Aflame and Burned statuses and their effects
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/AudioEventRTS.h"
#include "Common/GameAudio.h"
#include "Common/Xfer.h"

#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/FlammableUpdate.h"
#include "GameLogic/Module/FireSpreadUpdate.h"

//-------------------------------------------------------------------------------------------------
FlammableUpdateModuleData::FlammableUpdateModuleData()
{
	m_burnedDelay = 0;
	m_aflameDuration = 0;
	m_aflameDamageDelay = 0;
	m_aflameDamageAmount = 0;
	m_burningSoundName.clear();
	m_flameDamageLimitData = 20.0f;
	m_flameDamageExpirationDelay = LOGICFRAMES_PER_SECOND * 2;
}

//-------------------------------------------------------------------------------------------------
/*static*/ void FlammableUpdateModuleData::buildFieldParse(MultiIniFieldParse& p)
{
  UpdateModuleData::buildFieldParse(p);

	static const FieldParse dataFieldParse[] =
	{
		{ "BurnedDelay",						INI::parseDurationUnsignedInt,	nullptr, offsetof( FlammableUpdateModuleData, m_burnedDelay ) },
		{ "AflameDuration",					INI::parseDurationUnsignedInt,	nullptr, offsetof( FlammableUpdateModuleData, m_aflameDuration ) },
		{ "AflameDamageDelay",			INI::parseDurationUnsignedInt,	nullptr, offsetof( FlammableUpdateModuleData, m_aflameDamageDelay ) },
		{ "AflameDamageAmount",			INI::parseInt,									nullptr, offsetof( FlammableUpdateModuleData, m_aflameDamageAmount ) },
		{ "BurningSoundName",				INI::parseAsciiString,					nullptr,	offsetof( FlammableUpdateModuleData, m_burningSoundName) },
		{ "FlameDamageLimit",				INI::parseReal,									nullptr,	offsetof( FlammableUpdateModuleData, m_flameDamageLimitData ) },
		{ "FlameDamageExpiration",	INI::parseDurationUnsignedInt,	nullptr,	offsetof( FlammableUpdateModuleData, m_flameDamageExpirationDelay ) },
		{ nullptr, nullptr, nullptr, 0 }
	};
  p.add(dataFieldParse);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
FlammableUpdate::FlammableUpdate( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{
	m_status = FS_NORMAL;
	m_aflameEndFrame = 0;
	m_burnedEndFrame = 0;
	m_damageEndFrame = 0;
	m_audioHandle = 0;
	m_flameDamageLimit = getFlammableUpdateModuleData()->m_flameDamageLimitData;
	m_flameSource = INVALID_ID;
	m_lastFlameDamageDealt = 0;

	setWakeFrame(getObject(), UPDATE_SLEEP_FOREVER);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
FlammableUpdate::~FlammableUpdate()
{
	stopBurningSound();
}

//-------------------------------------------------------------------------------------------------
/** Damage has been dealt, this is an opportunity to reach to that damage */
//-------------------------------------------------------------------------------------------------
void FlammableUpdate::onDamage( DamageInfo *damageInfo )
{
	if( damageInfo->in.m_damageType == DAMAGE_FLAME || damageInfo->in.m_damageType == DAMAGE_PARTICLE_BEAM )
	{
		UnsignedInt now = TheGameLogic->getFrame();
		if( now - getFlammableUpdateModuleData()->m_flameDamageExpirationDelay > m_lastFlameDamageDealt )
		{
			// If it has been a long time since our last flame damage, reset the threshold
			m_flameDamageLimit = getFlammableUpdateModuleData()->m_flameDamageLimitData;
		}
		m_lastFlameDamageDealt = now;
#if RETAIL_COMPATIBLE_CRC || PRESERVE_NO_XP_FROM_FLAME_KILLS
		m_flameSource = getObject()->getID();
#else
		// TheSuperHackers @bugfix Stubbjax 03/09/2025 Allow flame damage to award xp to the flame source.
		m_flameSource = damageInfo->in.m_sourceID;
#endif

		Object *me = getObject();
		if( !me->getStatusBits().test( OBJECT_STATUS_AFLAME ) && !me->getStatusBits().test( OBJECT_STATUS_BURNED ) )
		{
			// If I'm not on fire, and I haven't burned up, see if I should try to catch fire.
			m_flameDamageLimit -= damageInfo->out.m_actualDamageDealt;
			if( m_flameDamageLimit <= 0 )
			{
				tryToIgnite();
			}
		}
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
UpdateSleepTime FlammableUpdate::update()
{
	Object *me = getObject();
	DEBUG_ASSERTCRASH(m_status == FS_AFLAME, ("hmm, should be aflame"));

	UnsignedInt now = TheGameLogic->getFrame();
	const FlammableUpdateModuleData *data = getFlammableUpdateModuleData();

	if( m_damageEndFrame != 0 && now >= m_damageEndFrame )
	{
		m_damageEndFrame = now + data->m_aflameDamageDelay;
		doAflameDamage();
	}

	if( m_burnedEndFrame != 0 && now >= m_burnedEndFrame )
	{
		// So this status is set, but I am still aflame on an independent timer.
		me->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_BURNED ) );
		me->setModelConditionState( MODELCONDITION_SMOLDERING );
	}

	if( m_aflameEndFrame != 0 && now >= m_aflameEndFrame )
	{
		// This is the important one.  I am no longer on fire.
		if( me->getStatusBits().test( OBJECT_STATUS_BURNED ) )
		{
			// If I am burned, then I will never catch fire again.
			m_status = FS_BURNED;
		}
		else
		{
			// otherwise I am free to burn again
			m_status = FS_NORMAL;
		}
		stopBurningSound();
		me->clearStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_AFLAME ) );
		me->getBodyModule()->setAflame( FALSE );
		me->clearModelConditionState( MODELCONDITION_AFLAME );
	}

	return calcSleepTime();
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
UpdateSleepTime FlammableUpdate::calcSleepTime()
{
	UnsignedInt now = TheGameLogic->getFrame();
	if (m_status == FS_AFLAME && m_aflameEndFrame != 0 && m_aflameEndFrame > now)
	{
		UnsignedInt soonest = m_aflameEndFrame;
		if (m_burnedEndFrame != 0 && m_burnedEndFrame < soonest && m_burnedEndFrame > now) soonest = m_burnedEndFrame;
		if (m_damageEndFrame != 0 && m_damageEndFrame < soonest && m_damageEndFrame > now) soonest = m_damageEndFrame;
		DEBUG_ASSERTCRASH(soonest - now > 0, ("hmm"));
		// UPDATE_SLEEP requires a count-of-frames, not an absolute-frame, so subtract 'now'
		return UPDATE_SLEEP(soonest - now);
	}
	else
	{
		return UPDATE_SLEEP_FOREVER;
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void FlammableUpdate::tryToIgnite()
{
	if( m_status == FS_NORMAL )
	{
		Object *me = getObject();
		me->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_AFLAME ) );
		me->getBodyModule()->setAflame( TRUE );
		me->setModelConditionState( MODELCONDITION_AFLAME );
		startBurningSound();

		// bleah. this sucks. (srj)
		static const NameKeyType key_FireSpreadUpdate = NAMEKEY("FireSpreadUpdate");
		FireSpreadUpdate* fu = (FireSpreadUpdate*)getObject()->findUpdateModule(key_FireSpreadUpdate);
		if (fu != nullptr)
		{
			fu->startFireSpreading();
		}

		m_status = FS_AFLAME;

		const FlammableUpdateModuleData *data = getFlammableUpdateModuleData();
		UnsignedInt now = TheGameLogic->getFrame();
		m_aflameEndFrame = now + data->m_aflameDuration;
		m_burnedEndFrame = data->m_burnedDelay ? now + data->m_burnedDelay : 0;
		m_damageEndFrame = data->m_aflameDamageDelay ? now + data->m_aflameDamageDelay : 0;

		setWakeFrame(getObject(), calcSleepTime());
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void FlammableUpdate::doAflameDamage()
{
	const FlammableUpdateModuleData *data = getFlammableUpdateModuleData();

	DamageInfo info;
	info.in.m_amount = data->m_aflameDamageAmount;
	info.in.m_sourceID = m_flameSource;
	info.in.m_damageType = DAMAGE_FLAME;
	info.in.m_deathType = DEATH_BURNED;

	getObject()->attemptDamage( &info );
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void FlammableUpdate::startBurningSound()
{
	const FlammableUpdateModuleData *data = getFlammableUpdateModuleData();

	AudioEventRTS audio(data->m_burningSoundName, getObject()->getID());
	m_audioHandle = TheAudio->addAudioEvent( &audio );
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void FlammableUpdate::stopBurningSound()
{
	if (m_audioHandle)
	{
		TheAudio->removeAudioEvent( m_audioHandle );
		m_audioHandle = 0;
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Bool FlammableUpdate::wouldIgnite()
{
	if( m_status == FS_NORMAL )
		return TRUE;

	return FALSE;
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void FlammableUpdate::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version
	* 2. TheSuperHackers @tweak Serialize flame source
	*/
// ------------------------------------------------------------------------------------------------
void FlammableUpdate::xfer( Xfer *xfer )
{

	// version
#if RETAIL_COMPATIBLE_XFER_SAVE
	XferVersion currentVersion = 1;
#else
	XferVersion currentVersion = 2;
#endif
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// flammability status
	xfer->xferUser( &m_status, sizeof( FlammabilityStatusType ) );

	// aflame end frame
	xfer->xferUnsignedInt( &m_aflameEndFrame );

	// burned end frame
	xfer->xferUnsignedInt( &m_burnedEndFrame );

	// damage end frame
	xfer->xferUnsignedInt( &m_damageEndFrame );

	// flame damage limit
	xfer->xferReal( &m_flameDamageLimit );

	// last flame damage dealt
	xfer->xferUnsignedInt( &m_lastFlameDamageDealt );

	if (version >= 2)
	{
		xfer->xferObjectID(&m_flameSource);
	}
}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void FlammableUpdate::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
