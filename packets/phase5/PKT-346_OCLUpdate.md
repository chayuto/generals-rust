# Translation Packet `PKT-346`: `OCLUpdate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-346`
- **Work Package:** `WP-5.4` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/update/ocl_update.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (13):**
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/PlayerTemplate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/PlayerTemplate.h`
  - `Common/RandomValue.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/RandomValue.h`
  - `Common/UnicodeString.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/UnicodeString.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/OCLUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/OCLUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/ObjectCreationList.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ObjectCreationList.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `GameLogic/TerrainLogic.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/TerrainLogic.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (6):**
  - `"OCL" => INI::parseObjectCreationList`
  - `"FactionOCL" => parseFactionObjectCreationList`
  - `"MinDelay" => INI::parseDurationUnsignedInt`
  - `"MaxDelay" => INI::parseDurationUnsignedInt`
  - `"CreateAtEdge" => INI::parseBool`
  - `"FactionTriggered" => INI::parseBool`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (5 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUnsignedInt( &m_nextCreationFrame )`
  3. `xferUnsignedInt( &m_timerStartedFrame )`
  4. `xferBool( &m_isFactionNeutral )`
  5. `xferInt( &m_currentPlayerColor )`
- **PRNG Calls (1 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValue( getOCLUpdateModuleData()->m_minDelay, getOCLUpdateModuleData()->m_maxDelay )`
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if RETAIL_COMPATIBLE_CRC`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/OCLUpdate.h`
```cpp
// FILE: OCLUpdate.h /////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, August2002
// Desc:   Update Spits out an OCL on a timer
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/UpdateModule.h"

class ObjectCreationList;

//-------------------------------------------------------------------------------------------------
class OCLUpdateModuleData : public UpdateModuleData
{
public:

	struct FactionOCLInfo
	{
		std::string									m_factionName;
		const ObjectCreationList *	m_ocl;
	};

	typedef std::list<FactionOCLInfo> FactionOCLList;

	const ObjectCreationList *	m_ocl;
	FactionOCLList							m_factionOCL;
	UnsignedInt									m_minDelay;
	UnsignedInt									m_maxDelay;
	Bool												m_isCreateAtEdge;				///< Otherwise, it is created on top of myself
	Bool												m_isFactionTriggered;		///< Faction has to be present before update will happen

	OCLUpdateModuleData();

	static void buildFieldParse(MultiIniFieldParse& p);

private:

};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class OCLUpdate : public UpdateModule
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( OCLUpdate, "OCLUpdate" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( OCLUpdate, OCLUpdateModuleData )

public:

	OCLUpdate( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	virtual UpdateSleepTime update() override;

	Real getCountdownPercent() const; ///< goes from 0% to 100%
	UnsignedInt getRemainingFrames() const; ///< For feedback display
	void resetTimer(); ///< added for sabotage purposes.
	virtual DisabledMaskType getDisabledTypesToProcess() const override { return DISABLEDMASK_ALL; }

protected:

	UnsignedInt			m_nextCreationFrame;
	UnsignedInt			m_timerStartedFrame;
	Bool						m_isFactionNeutral;
	Color						m_currentPlayerColor;

	Bool shouldCreate();
	void setNextCreationFrame();

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/OCLUpdate.cpp`
```cpp
// FILE: OCLUpdate.cpp /////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, August2002
// Desc:   Update Spits out an OCL on a timer
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/RandomValue.h"
#include "Common/Xfer.h"
#include "Common/Player.h"
#include "Common/PlayerTemplate.h"
#include "Common/UnicodeString.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectCreationList.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Module/OCLUpdate.h"
#include "GameLogic/TerrainLogic.h"

//-------------------------------------------------------------------------------------------------
void parseFactionObjectCreationList( INI *ini, void *instance, void *store, const void *userData )
{
	OCLUpdateModuleData::FactionOCLInfo info;
	info.m_ocl = nullptr;

	const char *token = ini->getNextToken( ini->getSepsColon() );
	if ( stricmp(token, "Faction") == 0 )
	{
		token = ini->getNextToken( ini->getSepsColon() );
		info.m_factionName = token;
	}
	else
		throw INI_INVALID_DATA;


	token = ini->getNextToken( ini->getSepsColon() );
	if ( stricmp(token, "OCL") == 0 )
		ini->parseObjectCreationList( ini, instance, &info.m_ocl, nullptr );
	else
		throw INI_INVALID_DATA;

	// Insert the info into the ocl hashmap
	OCLUpdateModuleData::FactionOCLList * theList = (OCLUpdateModuleData::FactionOCLList*)store;
	theList->push_back(info);

}

//-------------------------------------------------------------------------------------------------
OCLUpdateModuleData::OCLUpdateModuleData()
{
	m_minDelay = 0;
	m_maxDelay = 0;
	m_ocl = nullptr;
	m_factionOCL.clear();
	m_isCreateAtEdge = FALSE;
	m_isFactionTriggered = FALSE;
}

//-------------------------------------------------------------------------------------------------
/*static*/ void OCLUpdateModuleData::buildFieldParse(MultiIniFieldParse& p)
{
  UpdateModuleData::buildFieldParse(p);

	static const FieldParse dataFieldParse[] =
	{
		{ "OCL",					INI::parseObjectCreationList,		nullptr, offsetof( OCLUpdateModuleData, m_ocl ) },
		{ "FactionOCL",		parseFactionObjectCreationList,	nullptr, offsetof( OCLUpdateModuleData, m_factionOCL ) },
		{ "MinDelay",			INI::parseDurationUnsignedInt,	nullptr, offsetof( OCLUpdateModuleData, m_minDelay ) },
		{ "MaxDelay",			INI::parseDurationUnsignedInt,	nullptr, offsetof( OCLUpdateModuleData, m_maxDelay ) },
		{ "CreateAtEdge",	INI::parseBool,									nullptr, offsetof( OCLUpdateModuleData, m_isCreateAtEdge ) },
		{ "FactionTriggered",	INI::parseBool,							nullptr, offsetof( OCLUpdateModuleData, m_isFactionTriggered ) },
		{ nullptr, nullptr, nullptr, 0 }
	};
  p.add(dataFieldParse);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
OCLUpdate::OCLUpdate( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{
	m_nextCreationFrame = 0;
	m_timerStartedFrame = 0;
	m_isFactionNeutral = TRUE;
	m_currentPlayerColor = 0;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
OCLUpdate::~OCLUpdate()
{
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
UpdateSleepTime OCLUpdate::update()
{
#if RETAIL_COMPATIBLE_CRC
	if( getObject()->isDisabled() )
	{
		m_nextCreationFrame++;
		return UPDATE_SLEEP_NONE;
	}
#else
	// TheSuperHackers @bugfix dizzyj/Caball009/Mauller 14/07/2025 prevent triggering supply drop when subdued while under construction
	// When the construction is finished, we allow the timer to be initialized and then start shifting the timer while subdued
	if ( m_timerStartedFrame > 0 && getObject()->isDisabled() )
	{
		m_nextCreationFrame++;
		m_timerStartedFrame++;
		return UPDATE_SLEEP_NONE;
	}
#endif

	const OCLUpdateModuleData *data = getOCLUpdateModuleData();

	// Test if the OCL update is faction dependent. If so, check for faction changes
	if (data->m_isFactionTriggered)
	{
		Player *player = getObject()->getControllingPlayer();

		// Test for when a player captures the building
		if (m_isFactionNeutral)
		{
			if( player && player->isPlayableSide() )
			{
				m_currentPlayerColor = player->getPlayerColor();
				m_isFactionNeutral = FALSE;
				setNextCreationFrame();
			}
		}
		// Test for when the building has been made neutral or when it changes faction
		else
		{
			// If this is no longer under player control, then we set the faction to neutral
			if( !player || !player->isPlayableSide() )
			{
				m_isFactionNeutral = TRUE;
			}
			// If another player has taken control, reset the timer
			else if( player && player->getPlayerColor() != m_currentPlayerColor)
			{
				m_currentPlayerColor = player->getPlayerColor();
				setNextCreationFrame();
			}
		}

		// If the building is neutal, skip further update
		if (m_isFactionNeutral)
			return UPDATE_SLEEP_NONE;
	}

/// @todo srj use SLEEPY_UPDATE here
	if( shouldCreate() )
	{
		if( m_nextCreationFrame == 0 )
		{
			// You don't get to actually spread the first try, you start on a timer, then go
			setNextCreationFrame();
			return UPDATE_SLEEP_NONE;
		}

		setNextCreationFrame();

		Coord3D creationCoord;
		if( getOCLUpdateModuleData()->m_isCreateAtEdge )
			creationCoord = TheTerrainLogic->findClosestEdgePoint( getObject()->getPosition() );
		else
			creationCoord = *getObject()->getPosition();

		// If this is faction triggered, search through the faction specific OCLs to find the match
		if (data->m_isFactionTriggered)
		{
			std::string playerFactionName;

			Player *player = getObject()->getControllingPlayer();
			if (!player) return UPDATE_SLEEP_NONE;

			const PlayerTemplate *playerT = player->getPlayerTemplate();
			if (!playerT) return UPDATE_SLEEP_NONE;

			// Get and store the faction side to compare with the faction ocl list
			if (playerT->getSide().str()) playerFactionName = playerT->getSide().str();

			// Loop through the list of faction ocls to find the matching faction that triggers the specific ocls
			for (OCLUpdateModuleData::FactionOCLList::const_iterator it = data->m_factionOCL.begin(); it != data->m_factionOCL.end(); ++it)
			{
				const OCLUpdateModuleData::FactionOCLInfo &info = *it;
				if (playerFactionName == info.m_factionName)
				{
					ObjectCreationList::create( info.m_ocl, getObject(), &creationCoord, getObject()->getPosition(), getObject()->getOrientation() );
					break;
				}
			}
		}
		// Use the non faction OCL information
		else
		{
			ObjectCreationList::create( data->m_ocl, getObject(), &creationCoord, getObject()->getPosition(), getObject()->getOrientation() );
		}
	}
	return UPDATE_SLEEP_NONE;
}

//-------------------------------------------------------------------------------------------------
void OCLUpdate::resetTimer()
{
	setNextCreationFrame();
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
Bool OCLUpdate::shouldCreate()
{
	if( TheGameLogic->getFrame() < m_nextCreationFrame )
		return FALSE;//too soon

	if( getObject()->getStatusBits().test( OBJECT_STATUS_UNDER_CONSTRUCTION ) )
		return FALSE;// not built yet

	return TRUE;
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void OCLUpdate::setNextCreationFrame()
{
	UnsignedInt delay = GameLogicRandomValue( getOCLUpdateModuleData()->m_minDelay,
																						getOCLUpdateModuleData()->m_maxDelay );
	m_timerStartedFrame = TheGameLogic->getFrame();
	m_nextCreationFrame = m_timerStartedFrame + delay;

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
Real OCLUpdate::getCountdownPercent() const
{
	UnsignedInt now = TheGameLogic->getFrame();

	UnsignedInt totalTime = m_nextCreationFrame - m_timerStartedFrame;
	if (totalTime > 0)
	{
		return 1.0f - (( m_nextCreationFrame - now ) / (float)totalTime);
	}
	return 1.0f;
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
UnsignedInt OCLUpdate::getRemainingFrames() const
{
	UnsignedInt now = TheGameLogic->getFrame();

	return ( m_nextCreationFrame - now );
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void OCLUpdate::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void OCLUpdate::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	UpdateModule::xfer( xfer );

	// next creation frame
	xfer->xferUnsignedInt( &m_nextCreationFrame );

	// timer stated frame
	xfer->xferUnsignedInt( &m_timerStartedFrame );

	// faction status
	xfer->xferBool( &m_isFactionNeutral );

	// current owning player color
	xfer->xferInt( &m_currentPlayerColor );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void OCLUpdate::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
