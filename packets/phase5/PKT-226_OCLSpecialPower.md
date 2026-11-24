# Translation Packet `PKT-226`: `OCLSpecialPower`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-226`
- **Work Package:** `WP-5.2` (Phase 5)
- **Complexity & Assigned Tier:** Class `C1` -> `tier:1a-lite`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/special_power/ocl_special_power.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (12):**
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/RandomValue.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/RandomValue.h`
  - `Common/Science.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Science.h`
  - `Common/ThingFactory.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ThingFactory.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameLogic/Module/OCLSpecialPower.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/OCLSpecialPower.h`
  - `GameLogic/Module/SpecialPowerModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpecialPowerModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/ObjectCreationList.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ObjectCreationList.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `GameLogic/TerrainLogic.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/TerrainLogic.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (5):**
  - `"UpgradeOCL" => parseOCLUpgradePair`
  - `"OCL" => INI::parseObjectCreationList`
  - `"CreateLocation" => INI::parseIndexList`
  - `"ReferenceObject" => INI::parseAsciiString`
  - `"OCLAdjustPositionToPassable" => INI::parseBool`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (1 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`

## 2.5 Existing Rust Symbol Index (Use These Exact Paths)
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
- `zh_logic::arena`: `ObjectSlot`, `INVALID`, `fn`, `ModuleSlot`, `ModuleDataId`, `ThingTemplateId`, `FacetMask`, `NONE`, `UPDATE`, `DIE`, `DAMAGE`, `CREATE`, `COLLIDE`, `BODY`, ... (+34)
- `zh_logic::modules::body::active_body`: `ActiveBodyModuleData`, `fn`, `ActiveBody`
- `zh_logic::modules::body::body_module`: `BodyModuleData`, `BodyModuleBase`, `fn`, `BodyModule`
- `zh_logic::modules::body::highlander_body`: `HighlanderBody`
- `zh_logic::modules::body::hive_structure_body`: `HiveStructureBodyModuleData`, `fn`, `HiveStructureBody`
- `zh_logic::modules::body::immortal_body`: `ImmortalBody`
- `zh_logic::modules::body::inactive_body`: `InactiveBody`, `fn`
- `zh_logic::modules::body::structure_body`: `StructureBody`
- `zh_logic::modules::body::undead_body`: `UndeadBodyModuleData`, `fn`, `UndeadBody`
- `zh_logic::modules::create::create_module`: `CreateModuleData`, `CreateModuleBase`, `fn`, `CreateModule`
- `zh_logic::modules::create::grant_upgrade_create`: `GrantUpgradeCreateModuleData`, `GrantUpgradeCreate`
- `zh_logic::modules::create::lock_weapon_create`: `LockWeaponCreateModuleData`, `LockWeaponCreate`
- `zh_logic::modules::create::preorder_create`: `PreorderCreate`
- `zh_logic::modules::create::special_power_create`: `SpecialPowerCreate`
- `zh_logic::modules::create::supply_center_create`: `SupplyCenterCreate`
- `zh_logic::modules::create::supply_warehouse_create`: `SupplyWarehouseCreate`
- `zh_logic::modules::create::veterancy_gain_create`: `VeterancyGainCreateModuleData`, `VeterancyGainCreate`
- `zh_logic::modules::damage::bone_fx_damage`: `BoneFXDamage`, `fn`
- `zh_logic::modules::damage::damage_module`: `DamageModuleData`, `DamageModuleBase`, `fn`, `DamageModule`
- `zh_logic::modules::damage::transition_damage_fx`: `DAMAGE_MODULE_MAX_FX`, `FxDamageLocType`, `FxLocInfo`, `fn`, `FxDamageFxListInfo`, `FxDamageOclInfo`, `FxDamageParticleSystemInfo`, `TransitionDamageFxModuleData`, `TransitionDamageFX`
- `zh_logic::modules::destroy::destroy_module`: `DestroyModuleData`, `DestroyModuleBase`, `fn`, `DestroyModule`
- `zh_logic::modules::helper::object_defection_helper`: `DEFECTION_DETECTION_TIME_MAX`, `ObjectDefectionHelper`
- `zh_logic::modules::helper::object_helper`: `ObjectHelperBase`, `ObjectHelper`
- `zh_logic::modules::helper::object_repulsor_helper`: `ObjectRepulsorHelper`
- `zh_logic::modules::helper::object_smc_helper`: `ObjectSMCHelper`
- `zh_logic::modules::helper::object_weapon_status_helper`: `ObjectWeaponStatusHelper`
- `zh_logic::modules::helper::status_damage_helper`: `StatusDamageHelper`
- `zh_logic::modules::helper::subdual_damage_helper`: `SubdualDamageHelper`
- `zh_logic::modules::helper::temp_weapon_bonus_helper`: `WEAPON_BONUS_CONDITION_INVALID`, `TempWeaponBonusHelper`
- `zh_logic::modules::module`: `INVALID_BODY_TIMESTAMP`, `MaxHealthChangeType`, `fn`, `MAX_HEALTH_CHANGE_NAMES`, `ModuleData`, `StubModuleData`, `ModuleSnapshotHeader`, `BehaviorModule`, `UpdateFacet`, `BodyFacet`, `DieFacet`, `DamageFacet`, `CreateFacet`, `CollideFacet`, ... (+9)
- `zh_logic::modules::module_factory`: `ModuleParseContext`, `ModuleDataParseFn`, `ModuleCreateFn`, `BuiltinModuleEntry`, `BUILTIN_ZH_MODULES`, `W3D_DRAW_STUB_NAMES`, `ModuleDataStore`, `ModuleDescriptor`, `ModuleRegistry`
- `zh_logic::object::experience_tracker`: `ExperienceParentAccess`, `ExperienceTrackerState`, `fn`
- `zh_logic::object::object_script_status_bits`: `ObjectScriptStatusBit`, `fn`, `OBJECT_STATUS_SCRIPT_DISABLED`, `OBJECT_STATUS_SCRIPT_UNPOWERED`, `OBJECT_STATUS_SCRIPT_UNSELLABLE`, `OBJECT_STATUS_SCRIPT_UNSTEALTHED`, `OBJECT_STATUS_SCRIPT_TARGETABLE`
- `zh_logic::object::object_types`: `ObjectTypes`
- `zh_logic::object::overridable`: `OverridableEntry`, `fn`
- `zh_logic::object::override`: `OverrideStore`, `Override`, `fn`
- `zh_logic::object::thing`: `INITIAL_THING_POS_X`, `INITIAL_THING_POS_Y`, `ThingTerrainAccess`, `ThingTransformReactor`, `NoopTransformReactor`, `Thing`, `fn`
- `zh_logic::object::thing_factory`: `ThingFactory`, `fn`
- `zh_logic::object::thing_template`: `LEVEL_COUNT`, `MAX_UPGRADE_CAMEO_UPGRADES`, `USE_EXP_VALUE_FOR_SKILL_VALUE`, `RADAR_PRIORITY_NAMES`, `MODULEINTERFACE_UPDATE`, `MODULEINTERFACE_DIE`, `MODULEINTERFACE_DAMAGE`, `MODULEINTERFACE_CREATE`, `MODULEINTERFACE_COLLIDE`, `MODULEINTERFACE_BODY`, `MODULEINTERFACE_CONTAIN`, `MODULEINTERFACE_UPGRADE`, `MODULEINTERFACE_SPECIAL_POWER`, `MODULEINTERFACE_DESTROY`, ... (+27)

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/OCLSpecialPower.h`
```cpp
// FILE: OCLSpecialPower.h ////////////////////////////////////////////////////////////////////////
// Author: Colin Day, April 2002
// Desc:   Special powers that are driven by object creation lists
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/SpecialPowerModule.h"
#include "Common/Science.h"

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
class ObjectCreationList;

enum OCLCreateLocType CPP_11(: Int)
{
	CREATE_AT_EDGE_NEAR_SOURCE,
	CREATE_AT_EDGE_NEAR_TARGET,
	CREATE_AT_LOCATION,
	USE_OWNER_OBJECT,
	CREATE_ABOVE_LOCATION,
	CREATE_AT_EDGE_FARTHEST_FROM_TARGET,

	OCL_CREATE_LOC_COUNT
};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class OCLSpecialPowerModuleData : public SpecialPowerModuleData
{

public:

	struct Upgrades
	{
		ScienceType									m_science;
		const ObjectCreationList*		m_ocl;

		Upgrades() : m_science(SCIENCE_INVALID), m_ocl(nullptr)
		{
		}
	};

	std::vector<Upgrades>			m_upgradeOCL;
	const ObjectCreationList*	m_defaultOCL;
	OCLCreateLocType					m_createLoc;
	Bool											m_isOCLAdjustPositionToPassable; ///< Adjust target to nearest Passable cell

	//We need to know what the final product is going to be for script placement calculations
	//for construction sites like the sneak attack.
	AsciiString				m_referenceThingName;

	OCLSpecialPowerModuleData();
	static void buildFieldParse(MultiIniFieldParse& p);

};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class OCLSpecialPower : public SpecialPowerModule
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( OCLSpecialPower, "OCLSpecialPower" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( OCLSpecialPower, OCLSpecialPowerModuleData )

public:

	OCLSpecialPower( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool object

	virtual void doSpecialPower( UnsignedInt commandOptions ) override;
	virtual void doSpecialPowerAtObject( Object *obj, UnsignedInt commandOptions ) override;
	virtual void doSpecialPowerAtLocation( const Coord3D *loc, Real angle, UnsignedInt commandOptions ) override;

	//If the special power launches a construction site, we need to know the final product for placement purposes.
	virtual const ThingTemplate* getReferenceThingTemplate() const override;

protected:

	const ObjectCreationList* findOCL() const;
};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/SpecialPower/OCLSpecialPower.cpp`
```cpp
// FILE: OCLSpecialPower.cpp //////////////////////////////////////////////////////////////////////
// Author: Colin Day, April 2002
// Desc:   Special powers that are driven by object creation lists
///////////////////////////////////////////////////////////////////////////////////////////////////

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/Player.h"
#include "Common/RandomValue.h"
#include "Common/ThingFactory.h"
#include "Common/Xfer.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectCreationList.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/TerrainLogic.h"
#include "GameLogic/Module/OCLSpecialPower.h"

///////////////////////////////////////////////////////////////////////////////////////////////////
// MODULE DATA ////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////


enum
{
	CREATE_ABOVE_LOCATION_HEIGHT = 300,
	MAX_ADJUST_RADIUS = 500
};

static const char* const TheOCLCreateLocTypeNames[] =
{
	"CREATE_AT_EDGE_NEAR_SOURCE",
	"CREATE_AT_EDGE_NEAR_TARGET",
	"CREATE_AT_LOCATION",
	"USE_OWNER_OBJECT",
	"CREATE_ABOVE_LOCATION",
	"CREATE_AT_EDGE_FARTHEST_FROM_TARGET",
	nullptr
};
static_assert(ARRAY_SIZE(TheOCLCreateLocTypeNames) == OCL_CREATE_LOC_COUNT + 1, "Incorrect array size");

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
OCLSpecialPowerModuleData::OCLSpecialPowerModuleData()
{
	m_defaultOCL = nullptr;
	m_upgradeOCL.clear();
	m_createLoc = CREATE_AT_EDGE_NEAR_SOURCE;
	m_isOCLAdjustPositionToPassable = FALSE;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
static void parseOCLUpgradePair( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	OCLSpecialPowerModuleData::Upgrades up;

	INI::parseScience(ini, nullptr, &up.m_science, nullptr);
	INI::parseObjectCreationList(ini, nullptr, &up.m_ocl, nullptr);

	std::vector<OCLSpecialPowerModuleData::Upgrades>* s = (std::vector<OCLSpecialPowerModuleData::Upgrades>*)store;
	s->push_back(up);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
/* static */ void OCLSpecialPowerModuleData::buildFieldParse(MultiIniFieldParse& p)
{
	SpecialPowerModuleData::buildFieldParse( p );

	static const FieldParse dataFieldParse[] =
	{
		{ "UpgradeOCL", parseOCLUpgradePair, nullptr, offsetof( OCLSpecialPowerModuleData, m_upgradeOCL ) },
		{ "OCL", INI::parseObjectCreationList, nullptr, offsetof( OCLSpecialPowerModuleData, m_defaultOCL ) },
		{ "CreateLocation", INI::parseIndexList, TheOCLCreateLocTypeNames, offsetof( OCLSpecialPowerModuleData, m_createLoc ) },
		{ "ReferenceObject", INI::parseAsciiString, nullptr, offsetof( OCLSpecialPowerModuleData, m_referenceThingName ) },
		{ "OCLAdjustPositionToPassable", INI::parseBool, nullptr, offsetof( OCLSpecialPowerModuleData, m_isOCLAdjustPositionToPassable ) },
		{ nullptr, nullptr, nullptr, 0 }
	};
	p.add(dataFieldParse);

}

///////////////////////////////////////////////////////////////////////////////////////////////////
// MODULE /////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
OCLSpecialPower::OCLSpecialPower( Thing *thing, const ModuleData *moduleData )
							 : SpecialPowerModule( thing, moduleData )
{

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
const ObjectCreationList* OCLSpecialPower::findOCL() const
{
	const OCLSpecialPowerModuleData* d = getOCLSpecialPowerModuleData();
	const Player* controller = getObject()->getControllingPlayer();
	if (controller != nullptr)
	{
		for (std::vector<OCLSpecialPowerModuleData::Upgrades>::const_iterator it = d->m_upgradeOCL.begin();
					it != d->m_upgradeOCL.end();
					++it)
		{
			if (controller->hasScience(it->m_science))
				return it->m_ocl;
		}
	}
	return d->m_defaultOCL;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
OCLSpecialPower::~OCLSpecialPower()
{

}

//-------------------------------------------------------------------------------------------------
/** Execute the power */
//-------------------------------------------------------------------------------------------------
void OCLSpecialPower::doSpecialPowerAtLocation( const Coord3D *loc, Real angle, UnsignedInt commandOptions )
{
	if (getObject()->isDisabled())
		return;

	// sanity
	if( loc == nullptr )
		return;

	// get the module data
	const OCLSpecialPowerModuleData *modData = getOCLSpecialPowerModuleData();

	Coord3D targetCoord = *loc;

	if( modData->m_isOCLAdjustPositionToPassable )
	{
		FindPositionOptions fpOptions;
		fpOptions.flags = FPF_CLEAR_CELLS_ONLY;
		fpOptions.maxRadius = MAX_ADJUST_RADIUS;
		if ( ! ThePartitionManager->findPositionAround(&targetCoord, &fpOptions, &targetCoord) )
    {
    	// if findPosition() fails, then don't monkey with target Coord!
    	targetCoord = *loc;
    }


	}

	// call the base class action cause we are *EXTENDING* functionality
	SpecialPowerModule::doSpecialPowerAtLocation( &targetCoord, angle, commandOptions );

	// GeneralsX @bugfix Copilot 06/09/2026 Do not create an OCL after the base special-power intent was rejected.
	if (m_availableOnFrame == 0xFFFFFFFF)
		return;

	const ObjectCreationList* ocl = findOCL();

	// at what point will the "deliverer" come in
	Coord3D creationCoord;
	switch (modData->m_createLoc)
	{
		case CREATE_AT_EDGE_NEAR_SOURCE:
			creationCoord = TheTerrainLogic->findClosestEdgePoint( getObject()->getPosition() );
			ObjectCreationList::create( ocl, getObject(), &creationCoord, &targetCoord, angle );
			break;
		case CREATE_AT_EDGE_NEAR_TARGET:
			creationCoord = TheTerrainLogic->findClosestEdgePoint(&targetCoord);
			ObjectCreationList::create( ocl, getObject(), &creationCoord, &targetCoord, angle );
			break;
		case CREATE_AT_EDGE_FARTHEST_FROM_TARGET:
			creationCoord = TheTerrainLogic->findFarthestEdgePoint(&targetCoord);
			creationCoord.z += CREATE_ABOVE_LOCATION_HEIGHT;
			ObjectCreationList::create( ocl, getObject(), &creationCoord, &targetCoord, angle );
			break;
		case CREATE_AT_LOCATION:
			// this is the case where the special power stuff originates at the location of the mouse click
			creationCoord = targetCoord;
			ObjectCreationList::create( ocl, getObject(), &creationCoord, &targetCoord, angle );
			break;
		case USE_OWNER_OBJECT:
			creationCoord.set( targetCoord );
			ObjectCreationList::create( ocl, getObject(), &creationCoord, &targetCoord, angle, false );
			break;
		case CREATE_ABOVE_LOCATION:
			// this is the case where the special power stuff originates above the location of the mouse click
			creationCoord = targetCoord;
			creationCoord.z += CREATE_ABOVE_LOCATION_HEIGHT;
			ObjectCreationList::create( ocl, getObject(), &creationCoord, &targetCoord, angle );
			break;
	}
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void OCLSpecialPower::doSpecialPowerAtObject( Object *obj, UnsignedInt commandOptions )
{
	if (getObject()->isDisabled())
		return;

	// convert to a location
	if( !obj )
		return;
	doSpecialPowerAtLocation( obj->getPosition(), INVALID_ANGLE, commandOptions );
}

// ------------------------------------------------------------------------------------------------
void OCLSpecialPower::doSpecialPower( UnsignedInt commandOptions )
{
	if (getObject()->isDisabled())
		return;

	Coord3D creationCoord;
	creationCoord.set( *getObject()->getPosition() );

	// call the base class action cause we are *EXTENDING* functionality
	SpecialPowerModule::doSpecialPowerAtLocation( &creationCoord, INVALID_ANGLE, commandOptions );

	// GeneralsX @bugfix Copilot 06/09/2026 Do not create a targetless OCL after its intent was rejected.
	if (m_availableOnFrame == 0xFFFFFFFF)
		return;

	const ObjectCreationList* ocl = findOCL();
	ObjectCreationList::create( ocl, getObject(), &creationCoord, &creationCoord, false );
}

// ------------------------------------------------------------------------------------------------
const ThingTemplate* OCLSpecialPower::getReferenceThingTemplate() const
{
	return TheThingFactory->findTemplate( getOCLSpecialPowerModuleData()->m_referenceThingName );
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void OCLSpecialPower::crc( Xfer *xfer )
{

	// extend base class
	SpecialPowerModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void OCLSpecialPower::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	SpecialPowerModule::xfer( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void OCLSpecialPower::loadPostProcess()
{

	// extend base class
	SpecialPowerModule::loadPostProcess();

}
```
