# Translation Packet `PKT-382`: `TensileFormationUpdate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-382`
- **Work Package:** `WP-5.4` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/update/tensile_formation_update.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (14):**
  - `Common/AudioEventRTS.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/AudioEventRTS.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/TerrainVisual.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/TerrainVisual.h`
  - `GameLogic/AI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/AI.h`
  - `GameLogic/AIPathfind.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/AIPathfind.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/TensileFormationUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/TensileFormationUpdate.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `GameLogic/TerrainLogic.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/TerrainLogic.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (2):**
  - `"Enabled" => INI::parseBool`
  - `"CrackSound" => INI::parseAudioEventRTS`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (2 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferBool( &m_enabled )`
- **PRNG Calls (1 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValueReal(-PI,PI))`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/TensileFormationUpdate.h`
```cpp
// FILE: TensileFormationUpdate.h ////////////////////////////////////////////////////////////////////////////
// Author: Mark Lorenzen, November 2002
// Desc:   Springy formation movement like that of say, an avalanche
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/UpdateModule.h"

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
struct FieldParse;

//-------------------------------------------------------------------------------------------------
class TensileFormationUpdateModuleData: public UpdateModuleData
{

public:

	TensileFormationUpdateModuleData();

	static void buildFieldParse(MultiIniFieldParse& p);

	Bool m_enabled;							///< enabled
	AudioEventRTS				m_crackSound;

};

//-------------------------------------------------------------------------------------------------
class TensileFormationUpdate : public UpdateModule
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( TensileFormationUpdate, "TensileFormationUpdate" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( TensileFormationUpdate, TensileFormationUpdateModuleData )

public:

	TensileFormationUpdate( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	void setEnabled( Bool enabled ) { m_enabled = enabled; }  ///< enable/disable formation
	virtual UpdateSleepTime update() override;	///< Deciding whether or not to make new guys

protected:

	void propagateDislodgement( Bool enabled );
	void initLinks();

	struct TensileLink
	{
		ObjectID id;
		Coord3D tensor;
	};

	TensileLink m_links[4];//standard textile algorithm
	Coord3D m_inertia;
	Bool m_enabled;			///< enabled
	Bool m_linksInited;
	UnsignedInt m_motionlessCounter;
	UnsignedInt m_life;
	Real m_lowestSlideElevation;

	AudioEventRTS				m_crackSound;

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/TensileFormationUpdate.cpp`
```cpp
// FILE: TensileFormationUpdate.cpp //////////////////////////////////////////////////////////////////////////
// Author: Mark Lorenzen, November 2002
// Desc:   Springy formation movement like that of say, an avalanche
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/Xfer.h"
#include "Common/AudioEventRTS.h"

#include "GameLogic/Object.h"
#include "GameLogic/TerrainLogic.h"
#include "GameLogic/Module/TensileFormationUpdate.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/AI.h"
#include "GameLogic/AIPathfind.h"

#include "GameClient/TerrainVisual.h"
#include "GameClient/Drawable.h"





#define MAP_XY_FACTOR			(10.0f)	 //How wide and tall each height map square is in world space.
#define MAP_HEIGHT_SCALE	(MAP_XY_FACTOR/16.0f)		//divide all map heights by 8.





//---------------------------------------------------------------------------------
TensileFormationUpdate* getTFU( Object *obj )
{
	static NameKeyType theTFUKey = TheNameKeyGenerator->nameToKey( "TensileFormationUpdate" );
	TensileFormationUpdate* tfu = (TensileFormationUpdate *)obj->findUpdateModule( theTFUKey );
	return tfu;
}

class PartitionFilterTensileFormationMember : public PartitionFilter
{
private:
	Object* m_obj;

public:
	PartitionFilterTensileFormationMember( Object* obj ) : m_obj( obj ) { }
#if defined(RTS_DEBUG)
	virtual const char* debugGetName() override { return "PartitionFilterTensileFormationMember"; }
#endif
	virtual Bool allow( Object *objOther ) override
	{
		return ( getTFU( objOther ) != nullptr );
	}
};


// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
TensileFormationUpdateModuleData::TensileFormationUpdateModuleData()
{

	m_enabled = FALSE;

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
/*static*/ void TensileFormationUpdateModuleData::buildFieldParse(MultiIniFieldParse& p)
{

	UpdateModuleData::buildFieldParse( p );

	static const FieldParse dataFieldParse[] =
	{
		{ "Enabled",	    INI::parseBool,	nullptr, offsetof( TensileFormationUpdateModuleData, m_enabled ) },
		{ "CrackSound",	INI::parseAudioEventRTS,	nullptr,	offsetof( TensileFormationUpdateModuleData, m_crackSound) },
		{ nullptr, nullptr, nullptr, 0 }
	};
	p.add(dataFieldParse);

}

///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
TensileFormationUpdate::TensileFormationUpdate( Thing *thing, const ModuleData *moduleData )
						:UpdateModule( thing, moduleData )
{


	const TensileFormationUpdateModuleData *modData = getTensileFormationUpdateModuleData();

	// save our initial enabled status based on INI settings
	m_enabled = ((TensileFormationUpdateModuleData *)moduleData)->m_enabled;
	m_inertia.set(0,0,0);
	m_linksInited = FALSE;
	m_motionlessCounter = 0;
	m_life = 0;
	m_lowestSlideElevation = 255.0f;

	for ( int t = 0; t < 4; ++t)
	{
		m_links[ t ].id = INVALID_ID;
		m_links[ t ].tensor.set(0,0,0);
	}

	m_crackSound = modData->m_crackSound;
	m_crackSound.setObjectID( getObject()->getID() ) ;

	TheAI->pathfinder()->createAWallFromMyFootprint( getObject() ); // Temporarily treat this object as an obstacle.


	//Coord3D pos = *getObject()->getPosition();
	//if ( pos.z > 80)
	//{
	//	pos.z = 80 + ( ( pos.z - 80 ) * 2);
	//	getObject()->setPosition( &pos );
	//}



}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
TensileFormationUpdate::~TensileFormationUpdate()
{

}



void TensileFormationUpdate::initLinks()
{

	m_linksInited = TRUE;

	Object *obj = getObject();
	if ( !obj )
		return;

	PartitionFilterTensileFormationMember tfmFilter( getObject() );
	PartitionFilter *filters[] = { &tfmFilter, nullptr };
	SimpleObjectIterator *iter = nullptr;
	iter = ThePartitionManager->iterateObjectsInRange(getObject(), 1000.0f, FROM_BOUNDINGSPHERE_3D, filters);
	MemoryPoolObjectHolder hold(iter);

	Real closestDistance = 99999.9f;
	Real thisDistance    = 99999.9f;
	const Coord3D *myPos = obj->getPosition();

	for (Object* other = iter->first(); other; other = iter->next())
	{
		const Coord3D *theirPos = other->getPosition();
		Coord3D delta = { theirPos->x - myPos->x, theirPos->y - myPos->y, theirPos->z - myPos->z };

		thisDistance = delta.length();
		if ( closestDistance > thisDistance )
		{
			closestDistance = thisDistance;

			//push a new link
			for ( int t = 3; t > 0; --t )
				m_links[ t ] = m_links[ t-1 ];

			m_links[ 0 ].id = other->getID();
			m_links[ 0 ].tensor = delta;
		}
	}

	//ICoord2D gridPos;
	//gridPos.x = REAL_TO_INT_FLOOR(getObject()->getPosition()->x/MAP_XY_FACTOR);
	//gridPos.y = REAL_TO_INT_FLOOR(getObject()->getPosition()->y/MAP_XY_FACTOR);
	//TheTerrainVisual->setRawMapHeight(&gridPos, 500);

	getObject()->setOrientation(GameLogicRandomValueReal(-PI,PI));

}






// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
UpdateSleepTime TensileFormationUpdate::update()
{

	if ( ! m_linksInited )
		initLinks();



	if( m_enabled == FALSE )
	{
	//We are all going to sit here idle and do not much of anything until one of us gets hurt
		BodyModuleInterface *body = getObject()->getBodyModule();
		BodyDamageType bdt = body->getDamageState();

		if ( bdt >= BODY_DAMAGED )
		{
			//THen the hurt one (maybe me) will tell every other member of the formation to enable
			//So we will start moving
			setEnabled( TRUE );

			TheAI->pathfinder()->removeWallFromMyFootprint( getObject() );  // Undo createAWallFromMyFootprint.


			//ALert players of disaster happening
			if ( ! m_crackSound.isCurrentlyPlaying())
				m_crackSound.setPlayingHandle(TheAudio->addAudioEvent( &m_crackSound ));

		}
		else
			return UPDATE_SLEEP(30);
	}


	Drawable *draw = getObject()->getDrawable();
	if ( ! draw )
		return UPDATE_SLEEP_NONE;

	if (++m_life > 300)
	{
		draw->clearModelConditionFlags(MAKE_MODELCONDITION_MASK(MODELCONDITION_MOVING));
		draw->clearModelConditionFlags(MAKE_MODELCONDITION_MASK(MODELCONDITION_FREEFALL));
		draw->clearModelConditionFlags(MAKE_MODELCONDITION_MASK(MODELCONDITION_POST_COLLAPSE));

		BodyModuleInterface *body = getObject()->getBodyModule();
		body->setDamageState( BODY_RUBBLE );


		TheAI->pathfinder()->createAWallFromMyFootprint( getObject() ); // Temporarily treat this object as an obstacle.

		return UPDATE_SLEEP_FOREVER;
	}


	if ( m_life%30 == 29 )
		propagateDislodgement( TRUE );


	//APPLY PHYSICS===========================
	const Coord3D *pos = getObject()->getPosition();

	Coord3D normal = { 0.0f, 0.0f, 1.0f };
	TheTerrainLogic->getGroundHeight(pos->x, pos->y, &normal); // which way does the ground slope?

	Coord3D slope = { normal.x, normal.y, 0.0f};
	Real steepness = 1.0f - normal.z;
	slope.scale( 0.3f + steepness);

	m_inertia.add( slope );

	Real friction = 0.95f;
	m_inertia.scale( friction );



	Coord3D newPos;
	newPos.x = pos->x + m_inertia.x;//flow down the slope
	newPos.y = pos->y + m_inertia.y;
	newPos.z = TheTerrainLogic->getGroundHeight(newPos.x, newPos.y);//rest on surface here



	Object *tree = ThePartitionManager->getClosestObject( &newPos, getObject()->getGeometryInfo().getMajorRadius(), FROM_CENTER_2D );
	if (tree->isKindOf( KINDOF_SHRUBBERY ))
		tree->topple( &m_inertia, m_inertia.length(), 1 );//No Bounce



	//APPLY TENSORS===========================
	Coord3D tensorSum = { 0, 0, 0 };
	for ( int t = 0; t < 4; ++t )
	{
		Object *other = TheGameLogic->findObjectByID( m_links[ t ].id );

		if ( other )
		{
			Coord3D desiredPos = *other->getPosition();

			Coord3D tensor = m_links[ t ].tensor;

			desiredPos.sub( tensor );

			//Coord3D desiredPos = { theirPos->x - m_links[ t ].tensor.x, theirPos->y - m_links[ t ].tensor.y, theirPos->z - m_links[ t ].tensor.z };

			newPos.x = newPos.x*0.93f + desiredPos.x*0.07f;
			newPos.y = newPos.y*0.93f + desiredPos.y*0.07f;
			newPos.z = MIN( m_lowestSlideElevation, TheTerrainLogic->getGroundHeight(newPos.x, newPos.y) );//rest on surface here

			tensor.normalize();
			tensorSum.add( tensor );

		}

	}

	tensorSum.normalize();

	Coord3D inertiaNormal = m_inertia;
	inertiaNormal.normalize();









	draw->setModelConditionFlags(MAKE_MODELCONDITION_MASK(MODELCONDITION_POST_COLLAPSE));

	if ( m_life < 200 )
		draw->setModelConditionFlags(MAKE_MODELCONDITION_MASK(MODELCONDITION_MOVING));
	else
		draw->clearModelConditionFlags(MAKE_MODELCONDITION_MASK(MODELCONDITION_MOVING));

	if ( WWMath::Fabs( pos->z - newPos.z ) > 0.2f && m_life < 100)
		draw->setModelConditionFlags(MAKE_MODELCONDITION_MASK(MODELCONDITION_FREEFALL));
	else
		draw->clearModelConditionFlags(MAKE_MODELCONDITION_MASK(MODELCONDITION_FREEFALL));

//	{
//		RGBColor cyan = {0,1,1};
//		draw->flashAsSelected(&cyan);
//	}



	//else
		//m_motionlessCounter = 0;


	//if ( newPos.z > 80)
	//	newPos.z = 80 + ( ( newPos.z - 80 ) * 2);


	m_lowestSlideElevation = newPos.z;
	getObject()->setPosition( &newPos );



	return UPDATE_SLEEP_NONE;
}





//-------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------
void TensileFormationUpdate::propagateDislodgement ( Bool enabled )
{
	PartitionFilterTensileFormationMember tfmFilter( getObject() );
	PartitionFilter *filters[] = { &tfmFilter, nullptr };
	SimpleObjectIterator *iter = nullptr;
	iter = ThePartitionManager->iterateObjectsInRange(getObject(), 100.0f, FROM_BOUNDINGSPHERE_3D, filters);
	MemoryPoolObjectHolder hold(iter);
	for (Object* other = iter->first(); other; other = iter->next())
	{

		//Object *other = TheGameLogic->findObjectByID( m_links[ t ].id );
		if ( other )
		{
			BodyModuleInterface *body = other->getBodyModule();
			if ( body )
			{
				body->setDamageState( BODY_DAMAGED );
			}
		}


		//TensileFormationUpdate* tfu = getTFU(other);
		//if ( tfu != nullptr )
		//{
		//	tfu->setEnabled( enabled );
		//}
	}

	for ( int t = 0; t < 4; ++t )
	{
		Object *other = TheGameLogic->findObjectByID( m_links[ t ].id );
		if ( other )
		{
			BodyModuleInterface *body = other->getBodyModule();
			if ( body )
			{
				body->setDamageState( BODY_DAMAGED );
			}
		}
	}



}




// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void TensileFormationUpdate::crc( Xfer *xfer )
{

	// extend base class
	UpdateModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void TensileFormationUpdate::xfer( Xfer *xfer )
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
void TensileFormationUpdate::loadPostProcess()
{

	// extend base class
	UpdateModule::loadPostProcess();

}
```
