# Translation Packet `PKT-373`: `StealthDetectorUpdate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-373`
- **Work Package:** `WP-5.4` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/update/stealth_detector_update.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (21):**
  - `Common/BitFlagsIO.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/BitFlagsIO.h`
  - `Common/GameUtility.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameUtility.h`
  - `Common/MiscAudio.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/MiscAudio.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/PlayerList.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/PlayerList.h`
  - `Common/Radar.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Radar.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/Eva.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Eva.h`
  - `GameClient/GameText.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/GameText.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameClient/ParticleSys.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/ParticleSys.h`
  - `GameLogic/Damage.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/Damage.h`
  - `GameLogic/Module/ContainModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ContainModule.h`
  - `GameLogic/Module/StealthDetectorUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/StealthDetectorUpdate.h`
  - `GameLogic/Module/StealthUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/StealthUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (14):**
  - `"DetectionRate" => INI::parseDurationUnsignedInt`
  - `"DetectionRange" => INI::parseReal`
  - `"InitiallyDisabled" => INI::parseBool`
  - `"PingSound" => INI::parseAudioEventRTS`
  - `"LoudPingSound" => INI::parseAudioEventRTS`
  - `"IRBeaconParticleSysName" => INI::parseParticleSystemTemplate`
  - `"IRParticleSysName" => INI::parseParticleSystemTemplate`
  - `"IRBrightParticleSysName" => INI::parseParticleSystemTemplate`
  - `"IRGridParticleSysName" => INI::parseParticleSystemTemplate`
  - `"IRParticleSysBone" => INI::parseAsciiString`
  - `"ExtraRequiredKindOf" => KindOfMaskType::parseFromINI`
  - `"ExtraForbiddenKindOf" => KindOfMaskType::parseFromINI`
  - `"CanDetectWhileGarrisoned" => INI::parseBool`
  - `"CanDetectWhileContained" => INI::parseBool`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (2 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferBool( &m_enabled )`
- **PRNG Calls (1 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValue(1, data->m_updateRate)) : UPDATE_SLEEP_FOREVER)`

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
- `zh_logic::modules::update::power_plant_update`: `PowerPlantUpdateModuleData`, `PowerPlantUpdate`
- `zh_logic::modules::update::projectile_stream_update`: `MAX_PROJECTILE_STREAM`, `ProjectileStreamUpdateModuleData`, `ProjectileStreamUpdate`
- `zh_logic::modules::update::prone_update`: `ProneUpdateModuleData`, `ProneUpdate`
- `zh_logic::modules::update::radar_update`: `RadarUpdateModuleData`, `RadarUpdate`
- `zh_logic::modules::update::radius_decal_update`: `RadiusDecalUpdateModuleData`, `RadiusDecalUpdate`
- `zh_logic::modules::update::smart_bomb_target_homing_update`: `SmartBombTargetHomingUpdateModuleData`, `SmartBombTargetHomingUpdate`
- `zh_logic::modules::update::spy_vision_update`: `SpyVisionUpdateModuleData`, `SpyVisionUpdate`, `fn`
- `zh_logic::modules::update::stealth_detector_update`: `StealthDetectorUpdateModuleData`, `StealthDetectorUpdate`
- `zh_logic::modules::update::stealth_update`: `STEALTH_NOT_WHILE_ATTACKING`, `STEALTH_NOT_WHILE_MOVING`, `STEALTH_NOT_WHILE_USING_ABILITY`, `STEALTH_NOT_WHILE_FIRING_PRIMARY`, `STEALTH_NOT_WHILE_FIRING_SECONDARY`, `STEALTH_NOT_WHILE_FIRING_TERTIARY`, `STEALTH_ONLY_WITH_BLACK_MARKET`, `STEALTH_NOT_WHILE_TAKING_DAMAGE`, `STEALTH_NOT_WHILE_RIDERS_ATTACKING`, `STEALTH_NOT_WHILE_FIRING_WEAPON`, `STEALTH_LEVEL_NAMES`, `StealthUpdateModuleData`, `StealthUpdate`
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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/StealthDetectorUpdate.h`
```cpp
// FILE: StealthDetectorUpdate.h //////////////////////////////////////////////////////////////////////////
// Author: Steven Johnson, May 2002
// Desc:	 An update that checks for a status bit to stealth the owning object
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/UpdateModule.h"

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
class Thing;

//-------------------------------------------------------------------------------------------------
class StealthDetectorUpdateModuleData : public UpdateModuleData
{
public:
	UnsignedInt											m_updateRate;
	Real														m_detectionRange;
	Bool														m_initiallyDisabled;
	AudioEventRTS										m_pingSound;
	AudioEventRTS										m_loudPingSound;
	const ParticleSystemTemplate*   m_IRBeaconParticleSysTmpl;
	const ParticleSystemTemplate*   m_IRParticleSysTmpl;
	const ParticleSystemTemplate*   m_IRBrightParticleSysTmpl;
	const ParticleSystemTemplate*   m_IRGridParticleSysTmpl;
	AsciiString											m_IRParticleSysBone;
	KindOfMaskType									m_extraDetectKindof;			///< units must match any kindof bits set here, in order to be detected
	KindOfMaskType									m_extraDetectKindofNot;		///< units must NOT match any kindof bits set here, in order to be detected
	Bool														m_canDetectWhileGarrisoned;
	Bool														m_canDetectWhileTransported;

	StealthDetectorUpdateModuleData()
	{
		m_updateRate = 1;
		m_detectionRange = 0.0f;
		m_initiallyDisabled = false;
		m_IRBeaconParticleSysTmpl = nullptr;
		m_IRParticleSysTmpl = nullptr;
		m_IRBrightParticleSysTmpl = nullptr;
		m_IRGridParticleSysTmpl = nullptr;
		m_extraDetectKindof.clear();
		m_extraDetectKindofNot.clear();
		m_canDetectWhileGarrisoned = false;
		m_canDetectWhileTransported = false;
	}

	static void buildFieldParse(MultiIniFieldParse& p);

};

//-------------------------------------------------------------------------------------------------
class StealthDetectorUpdate : public UpdateModule
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( StealthDetectorUpdate, "StealthDetectorUpdate" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( StealthDetectorUpdate, StealthDetectorUpdateModuleData );

public:

	StealthDetectorUpdate( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	Bool isSDEnabled() const { return m_enabled; }
	void setSDEnabled( Bool enabled );
	virtual UpdateSleepTime update() override;
	virtual DisabledMaskType getDisabledTypesToProcess() const override { return MAKE_DISABLED_MASK( DISABLED_HELD ); }

private:
	Bool m_enabled;

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/StealthDetectorUpdate.cpp`
```cpp
// FILE: StealthDetectorUpdate.cpp ////////////////////////////////////////////////////////////////////////
// Author: Kris Morness, May 2002
// Desc:	 An update that checks for a status bit to stealth the owning object
///////////////////////////////////////////////////////////////////////////////////////////////////

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#define DEFINE_STEALTHLEVEL_NAMES

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/GameUtility.h"
#include "Common/MiscAudio.h"
#include "Common/Radar.h"
#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"
#include "GameClient/Drawable.h"
#include "GameClient/Eva.h"
#include "GameClient/GameText.h"
#include "GameClient/InGameUI.h"
#include "GameClient/ParticleSys.h"
#include "GameLogic/Damage.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/StealthUpdate.h"
#include "GameLogic/Module/StealthDetectorUpdate.h"
#include "Common/BitFlagsIO.h"
#include "Common/PlayerList.h"
#include "Common/Player.h"


//-------------------------------------------------------------------------------------------------
void StealthDetectorUpdateModuleData::buildFieldParse(MultiIniFieldParse& p)
{
  UpdateModuleData::buildFieldParse(p);

	static const FieldParse dataFieldParse[] =
	{
		{ "DetectionRate",							INI::parseDurationUnsignedInt,			nullptr, offsetof( StealthDetectorUpdateModuleData, m_updateRate ) },
		{ "DetectionRange",							INI::parseReal,											nullptr, offsetof( StealthDetectorUpdateModuleData, m_detectionRange ) },
		{ "InitiallyDisabled",					INI::parseBool,											nullptr, offsetof( StealthDetectorUpdateModuleData, m_initiallyDisabled ) },
		{ "PingSound",									INI::parseAudioEventRTS,						nullptr, offsetof( StealthDetectorUpdateModuleData, m_pingSound ) },
		{ "LoudPingSound",							INI::parseAudioEventRTS,						nullptr, offsetof( StealthDetectorUpdateModuleData, m_loudPingSound ) },
		{ "IRBeaconParticleSysName",		INI::parseParticleSystemTemplate,		nullptr, offsetof( StealthDetectorUpdateModuleData, m_IRBeaconParticleSysTmpl ) },
		{ "IRParticleSysName",					INI::parseParticleSystemTemplate,		nullptr, offsetof( StealthDetectorUpdateModuleData, m_IRParticleSysTmpl ) },
		{ "IRBrightParticleSysName",		INI::parseParticleSystemTemplate,		nullptr, offsetof( StealthDetectorUpdateModuleData, m_IRBrightParticleSysTmpl ) },
		{ "IRGridParticleSysName",			INI::parseParticleSystemTemplate,		nullptr, offsetof( StealthDetectorUpdateModuleData, m_IRGridParticleSysTmpl ) },
		{ "IRParticleSysBone",					INI::parseAsciiString,							nullptr, offsetof( StealthDetectorUpdateModuleData, m_IRParticleSysBone ) },
		{ "ExtraRequiredKindOf",				KindOfMaskType::parseFromINI,				nullptr, offsetof( StealthDetectorUpdateModuleData, m_extraDetectKindof ) },
		{ "ExtraForbiddenKindOf",				KindOfMaskType::parseFromINI,				nullptr, offsetof( StealthDetectorUpdateModuleData, m_extraDetectKindofNot ) },
		{ "CanDetectWhileGarrisoned",		INI::parseBool,											nullptr, offsetof( StealthDetectorUpdateModuleData, m_canDetectWhileGarrisoned ) },
		{ "CanDetectWhileContained",		INI::parseBool,											nullptr, offsetof( StealthDetectorUpdateModuleData, m_canDetectWhileTransported ) },

		{ nullptr, nullptr, nullptr, 0 }
	};
  p.add(dataFieldParse);
}
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
StealthDetectorUpdate::StealthDetectorUpdate( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{
	const StealthDetectorUpdateModuleData *data = getStealthDetectorUpdateModuleData();
	m_enabled = !data->m_initiallyDisabled;
	// start these guys with random phasings so that we don't
	// have all of 'em check on the same frame.
	setWakeFrame(getObject(), m_enabled ? UPDATE_SLEEP(GameLogicRandomValue(1, data->m_updateRate)) : UPDATE_SLEEP_FOREVER);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
StealthDetectorUpdate::~StealthDetectorUpdate()
{
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void StealthDetectorUpdate::setSDEnabled( Bool enabled )
{
	m_enabled = enabled;
	setWakeFrame(getObject(), m_enabled ? UPDATE_SLEEP_NONE : UPDATE_SLEEP_FOREVER);
}





//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class PartitionFilterStealthedOrStealthGarrisoned : public PartitionFilter
{
public:
	PartitionFilterStealthedOrStealthGarrisoned() { }

	virtual Bool allow(Object *objOther) override;

#if defined(RTS_DEBUG)
	virtual const char* debugGetName() override { return "PartitionFilterStealthedOrStealthGarrisoned"; }
#endif
};

Bool PartitionFilterStealthedOrStealthGarrisoned::allow( Object *objOther)
{
	if( ! objOther )
		return FALSE;

	if( objOther->getStatusBits().test( OBJECT_STATUS_STEALTHED ) )
		return TRUE;

	ContainModuleInterface *contain = objOther->getContain();
	if( contain && contain->isGarrisonable() && contain->getStealthUnitsContained() )
		return TRUE;

	return FALSE;
}

//-------------------------------------------------------------------------------------------------
/** The update callback. */
//-------------------------------------------------------------------------------------------------
UpdateSleepTime StealthDetectorUpdate::update()
{
	const StealthDetectorUpdateModuleData *data = getStealthDetectorUpdateModuleData();
	Object* self = getObject();

	if (self->isEffectivelyDead())
		return UPDATE_SLEEP_FOREVER;

	// We have to wait until we are fully constructed, but we will detect the moment we finish
	if( self->testStatus(OBJECT_STATUS_UNDER_CONSTRUCTION) )
		return UPDATE_SLEEP_NONE;

	// We turn off forever the moment we are sold.
	if( self->testStatus(OBJECT_STATUS_SOLD) )
		return UPDATE_SLEEP_FOREVER;

	//Are we contained by anything?
	Object *containedBy = self->getContainedBy();
	if( containedBy )
	{
		ContainModuleInterface *contain = containedBy->getContain();
		if( contain )
		{
			//Are we eligible to detect stealth while in a container?
			if( contain->isGarrisonable() )
			{
				//We are in a garrisonable structure.
				if( !data->m_canDetectWhileGarrisoned )
				{
					//But we can't detect stuff while inside.
					return UPDATE_SLEEP(data->m_updateRate);
				}
			}
			else if( !data->m_canDetectWhileTransported )
			{
				//We are in a normal container and can't detect!
				return UPDATE_SLEEP(data->m_updateRate);
			}
		}
	}

	// only consider items that are currently stealthed.
	PartitionFilterStealthedOrStealthGarrisoned		filterStealthOrStealthGarrisoned;
	//PartitionFilterAcceptByObjectStatus		filterStatus(OBJECT_STATUS_STEALTHED, 0);
	PartitionFilterRelationship						filterTeam(self, PartitionFilterRelationship::ALLOW_ENEMIES | PartitionFilterRelationship::ALLOW_NEUTRAL );
	PartitionFilterAcceptByKindOf					filterKindof(data->m_extraDetectKindof, data->m_extraDetectKindofNot);
	PartitionFilterSameMapStatus					filterMapStatus(getObject());
	PartitionFilter*											filters[] = {
		&filterStealthOrStealthGarrisoned,
		&filterTeam,
		&filterKindof,
		&filterMapStatus,
		nullptr
	};

	Real visionRange = self->getVisionRange();
	if( data->m_detectionRange > 0.0f )
	{
		visionRange = data->m_detectionRange;
	}
	Bool foundSomeone = FALSE;

	SimpleObjectIterator *iter = ThePartitionManager->iterateObjectsInRange(
								self, visionRange, FROM_CENTER_2D, filters);
	MemoryPoolObjectHolder hold(iter);
	for (Object *them = iter->first(); them; them = iter->next())
	{
		if ( them->isEffectivelyDead() )
			continue;

		StealthUpdate* stealth = them->getStealth();
		if ( stealth )
		{

			// we have found someone
			foundSomeone = TRUE;

			//
			// if this object was not previously detected it is now being revealed and we
			// want to do some UI feedback
			//
			if( them->testStatus( OBJECT_STATUS_DETECTED ) == FALSE )
			{

				// for the player revealing the stealth unit do some UI feedback
				if( rts::getObservedOrLocalPlayer() == self->getControllingPlayer() &&
						self->getRelationship( them ) != ALLIES )
				{
					Bool doFeedback = TRUE;

					//Kris: Aug 18, 2003 -- This whole system is no longer viable with the inclusion of so many
					//stealth units. It's possible to get spammed hundreds of times in a row in MD_GLA02 because
					//we're blindly creating radar events for each detection (unless they are mines). The best thing
					//to do is tryEvent for everything...
					doFeedback = TheRadar->tryEvent( RADAR_EVENT_STEALTH_DISCOVERED, them->getPosition() );
					//OLD CODE:
					/*
					// do a radar event, for mines we only make events if there weren't other
					// mine events within close proximity and time to other mines
					//
					if( them->isKindOf( KINDOF_MINE ) || them->isKindOf( KINDOF_BOOBY_TRAP ) || them->isKindOf( KINDOF_DEMOTRAP ) )
						doFeedback = TheRadar->tryEvent( RADAR_EVENT_STEALTH_DISCOVERED, them->getPosition() );
					else
 						TheRadar->createEvent( them->getPosition(), RADAR_EVENT_STEALTH_DISCOVERED );
					*/

					// do audio and UI message if we need to do feedback
					if( doFeedback )
					{
 						// audio msg
 						static AudioEventRTS discoveredSound = TheAudio->getMiscAudio()->m_stealthDiscoveredSound;
 						discoveredSound.setPlayerIndex( self->getControllingPlayer()->getPlayerIndex() );
 						TheAudio->addAudioEvent( &discoveredSound );
 						// ui msg
 						TheInGameUI->message( TheGameText->fetch( "MESSAGE:StealthDiscovered" ) );

            // If revealing this unit is suppose to cause an Eva event, do it
            EvaMessage message = stealth->getEnemyDetectionEvaEvent();
            if ( message != EVA_Invalid && TheEva != nullptr )
            {
              TheEva->setShouldPlay( message );
            }
					}

				}

				// for the unit being revealed, do some UI feedback
				if( rts::getObservedOrLocalPlayer() == them->getControllingPlayer() &&
						self->getRelationship( them ) != ALLIES )
				{
 					Bool doFeedback = TRUE;

					//
					// do a radar event, for mines we only make events if there weren't other
					// mine events within close proximity and time to other mines
					//
					if( them->isKindOf( KINDOF_MINE ) || them->isKindOf( KINDOF_BOOBY_TRAP ) || them->isKindOf( KINDOF_DEMOTRAP ) )
						doFeedback = TheRadar->tryEvent( RADAR_EVENT_STEALTH_NEUTRALIZED, them->getPosition() );
					else
 						TheRadar->createEvent( them->getPosition(), RADAR_EVENT_STEALTH_NEUTRALIZED );

					// do audio and UI message if we need to do feedback
					if( doFeedback )
					{

 						// audio msg
 						static AudioEventRTS neutralizedSound = TheAudio->getMiscAudio()->m_stealthNeutralizedSound;
 						neutralizedSound.setPlayerIndex( them->getControllingPlayer()->getPlayerIndex() );
 						TheAudio->addAudioEvent( &neutralizedSound );
 						// ui msg
 						TheInGameUI->message( TheGameText->fetch( "MESSAGE:StealthNeutralized" ) );

            // If revealing this unit is suppose to cause an Eva event, do it
            EvaMessage message = stealth->getOwnDetectionEvaEvent();
            if ( message != EVA_Invalid && TheEva != nullptr )
            {
              TheEva->setShouldPlay( message );
            }
					}

				}

			}

			// updateRate PLUS 1 is necessary to ensure it stays detected 'till we are called again...
			stealth->markAsDetected(data->m_updateRate + 1);

			/** @todo srj -- evil hack here... this whole heat-vision thing is fucked.
				don't want it on mines but no good way to do that. hack for now. */
			Drawable *theirDraw = them->getDrawable();
			if ( theirDraw && !them->isKindOf(KINDOF_MINE))
			{
				theirDraw->setSecondMaterialPassOpacity( 1.0f );
			}

			if (data->m_IRGridParticleSysTmpl)
			{
				const ParticleSystemTemplate *gridTemplate = data->m_IRGridParticleSysTmpl;
				if (gridTemplate)
				{
					ParticleSystem *sys = TheParticleSystemManager->createParticleSystem( gridTemplate );//GRID
					if (sys)
					{
						Coord3D gridPosition = *them->getPosition();
						gridPosition.z = self->getPosition()->z + 17;
						gridPosition.x -= ((Int)gridPosition.x)%12;
						gridPosition.y -= ((Int)gridPosition.y)%12;

						sys->setPosition( &gridPosition );
					}
				}
			}

		}
		else // perhaps they are garrisoning something stealthy, eh?
		{
			ContainModuleInterface *contain = them->getContain();
			if( contain && contain->isGarrisonable() && contain->getStealthUnitsContained() )
			{
				Object* rider = nullptr;
				for(ContainedItemsList::const_iterator it = contain->getContainedItemsList()->begin(); it != contain->getContainedItemsList()->end(); ++it)
				{
					rider = *it;

          StealthUpdate *stealth = rider->getStealth();
					if ( stealth )
					{
						// we have found someone
						foundSomeone = TRUE;
						if( self->getControllingPlayer() != rider->getControllingPlayer() && self->getRelationship( rider ) != ALLIES )
						{
							stealth->markAsDetected(data->m_updateRate + 2);
						}

					}
				}
			}
		}
	}


  const Player *localPlayer = rts::getObservedOrLocalPlayer();

	//Make sure the detector is visible to the local player before we add effects or sounds.
	if ( self->getShroudedStatus( localPlayer->getPlayerIndex() ) <= OBJECTSHROUD_PARTIAL_CLEAR )
	{
    if ( self->testStatus( OBJECT_STATUS_STEALTHED ) == FALSE || self->getControllingPlayer() == localPlayer )
    {
		  Drawable *myDraw = self->getDrawable();
		  Coord3D bonePosition = {-1.66f,5.5f,15};
		  if (myDraw)
			  myDraw->getPristineBonePositions( data->m_IRParticleSysBone.str(), 0, &bonePosition, nullptr, 1);

		  const ParticleSystemTemplate *pingTemplate;
		  if ( foundSomeone )
			  pingTemplate = data->m_IRBrightParticleSysTmpl;
		  else
			  pingTemplate = data->m_IRParticleSysTmpl;

		  if (pingTemplate)
		  {
			  ParticleSystem *sys = TheParticleSystemManager->createParticleSystem( pingTemplate );
			  if (sys)
			  {
				  if (myDraw)
					  sys->attachToDrawable( myDraw );
				  else
					  sys->attachToObject( self );

				  sys->setPosition( &bonePosition );
			  }
		  }

		  const ParticleSystemTemplate *beaconTemplate = data->m_IRBeaconParticleSysTmpl;
		  if (beaconTemplate)
		  {
			  ParticleSystem *sys = TheParticleSystemManager->createParticleSystem( beaconTemplate );//BEACON
			  if (sys)
			  {
				  if (myDraw)
					  sys->attachToDrawable( myDraw );
				  else
					  sys->attachToObject( self );

				  sys->setPosition( &bonePosition );

			  }
		  }

		  AudioEventRTS IRPingSound;
		  if (foundSomeone)
			  IRPingSound = data->m_loudPingSound;
		  else
			  IRPingSound = data->m_pingSound;

		  IRPingSound.setObjectID( self->getID() );
		  TheAudio->addAudioEvent(&IRPingSound);

    }

	}


	return UPDATE_SLEEP(data->m_updateRate);

}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void StealthDetectorUpdate::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void StealthDetectorUpdate::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// enabled
	xfer->xferBool( &m_enabled );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void StealthDetectorUpdate::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
