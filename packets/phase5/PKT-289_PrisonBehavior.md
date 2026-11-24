# Translation Packet `PKT-289`: `PrisonBehavior`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-289`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/behavior/prison_behavior.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (13):**
  - `Common/GameState.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/GameState.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/RandomValue.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/RandomValue.h`
  - `Common/ThingFactory.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ThingFactory.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/GameClient.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/GameClient.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameClient/Line2D.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/Line2D.h`
  - `GameLogic/Module/OpenContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/OpenContain.h`
  - `GameLogic/Module/PrisonBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PrisonBehavior.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (2):**
  - `"ShowPrisoners" => INI::parseBool`
  - `"YardBonePrefix" => INI::parseAsciiString`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (6 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUnsignedShort( &visualCount )`
  3. `xferObjectID( &visual->m_objectID )`
  4. `xferDrawableID( &visual->m_drawableID )`
  5. `xferObjectID( &visual->m_objectID )`
  6. `xferDrawableID( &visual->m_drawableID )`
- **PRNG Calls (3 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValueReal( yardRegion.lo.x, yardRegion.hi.x )`
  - `GameLogicRandomValueReal( yardRegion.lo.y, yardRegion.hi.y )`
  - `GameLogicRandomValueReal( 0, TWO_PI ) )`

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
- `zh_logic::arena`: `ObjectSlot`, `INVALID`, `fn`, `ModuleSlot`, `ModuleDataId`, `ThingTemplateId`, `FacetMask`, `NONE`, `UPDATE`, `DIE`, `DAMAGE`, `CREATE`, `COLLIDE`, `BODY`, ... (+42)
- `zh_logic::modules::body::active_body`: `ActiveBodyModuleData`, `fn`, `ActiveBody`
- `zh_logic::modules::body::body_module`: `BodyModuleData`, `BodyModuleBase`, `fn`, `BodyModule`
- `zh_logic::modules::body::highlander_body`: `HighlanderBody`
- `zh_logic::modules::body::hive_structure_body`: `HiveStructureBodyModuleData`, `fn`, `HiveStructureBody`
- `zh_logic::modules::body::immortal_body`: `ImmortalBody`
- `zh_logic::modules::body::inactive_body`: `InactiveBody`, `fn`
- `zh_logic::modules::body::structure_body`: `StructureBody`
- `zh_logic::modules::body::undead_body`: `UndeadBodyModuleData`, `fn`, `UndeadBody`
- `zh_logic::modules::collide::collide_module`: `CollideModuleData`, `CollideModule`, `fn`, `CollideBehaviorImpl`
- `zh_logic::modules::collide::convert_to_car_bomb_crate_collide`: `ConvertToCarBombCrateCollideModuleData`, `ConvertToCarBombCrateCollide`
- `zh_logic::modules::collide::convert_to_hijacked_vehicle_crate_collide`: `ConvertToHijackedVehicleCrateCollideModuleData`, `ConvertToHijackedVehicleCrateCollide`
- `zh_logic::modules::collide::crate_collide`: `PRESERVE_MULTI_CRATE_PICKUP`, `SabotageVictimType`, `CrateCollideData`, `CrateCollide`, `CrateCollideImpl`
- `zh_logic::modules::collide::fire_weapon_collide`: `FireWeaponCollideModuleData`, `FireWeaponCollide`
- `zh_logic::modules::collide::heal_crate_collide`: `HealCrateCollideModuleData`, `HealCrateCollide`
- `zh_logic::modules::collide::money_crate_collide`: `UpgradePair`, `MoneyCrateCollideModuleData`, `MoneyCrateCollide`
- `zh_logic::modules::collide::sabotage_command_center_crate_collide`: `SabotageCommandCenterCrateCollideModuleData`, `SabotageCommandCenterCrateCollide`
- `zh_logic::modules::collide::sabotage_fake_building`: `SabotageFakeBuildingCrateCollideModuleData`, `SabotageFakeBuildingCrateCollide`
- `zh_logic::modules::collide::sabotage_internet_center_crate_collide`: `SabotageInternetCenterCrateCollideModuleData`, `SabotageInternetCenterCrateCollide`
- `zh_logic::modules::collide::sabotage_military_factory_crate_collide`: `SabotageMilitaryFactoryCrateCollideModuleData`, `SabotageMilitaryFactoryCrateCollide`
- `zh_logic::modules::collide::sabotage_power_plant_crate_collide`: `SabotagePowerPlantCrateCollideModuleData`, `SabotagePowerPlantCrateCollide`
- `zh_logic::modules::collide::sabotage_superweapon_crate_collide`: `SabotageSuperweaponCrateCollideModuleData`, `SabotageSuperweaponCrateCollide`
- `zh_logic::modules::collide::sabotage_supply_center_crate_collide`: `SabotageSupplyCenterCrateCollideModuleData`, `SabotageSupplyCenterCrateCollide`
- `zh_logic::modules::collide::sabotage_supply_dropzone_crate_collide`: `SabotageSupplyDropzoneCrateCollideModuleData`, `SabotageSupplyDropzoneCrateCollide`
- `zh_logic::modules::collide::salvage_crate_collide`: `SalvageCrateCollideModuleData`, `SalvageCrateCollide`
- `zh_logic::modules::collide::shroud_crate_collide`: `ShroudCrateCollideModuleData`, `ShroudCrateCollide`
- `zh_logic::modules::collide::squish_collide`: `SquishCollideModuleData`, `SquishCollide`
- `zh_logic::modules::collide::unit_crate_collide`: `UnitCrateCollideModuleData`, `UnitCrateCollide`
- `zh_logic::modules::collide::veterancy_crate_collide`: `VeterancyCrateCollideModuleData`, `VeterancyCrateCollide`
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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PrisonBehavior.h`
```cpp
// FILE: PrisonBehavior.h /////////////////////////////////////////////////////////////////////////
// Author: Colin Day, August 2002
// Desc:   Prison Behaviors
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/OpenContain.h"

#ifdef ALLOW_SURRENDER

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
class PrisonVisual;

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
class PrisonBehaviorModuleData : public OpenContainModuleData
{

public:

	PrisonBehaviorModuleData();

	static void buildFieldParse( MultiIniFieldParse &p );

	Bool m_showPrisoners;								///< Will the contained prisoners be shown in the prison yard
	AsciiString m_prisonYardBonePrefix;	///< Bone prefix that makes up the prison yards

};

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
class PrisonBehavior : public OpenContain
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( PrisonBehavior, "PrisonBehavior" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( PrisonBehavior, PrisonBehaviorModuleData )

public:

	PrisonBehavior( Thing *thing, const ModuleData *moduleData );
	// virtual destructor prototype provided by memory pool object

	// module methods
	virtual void onDelete();

	// contain methods
	virtual void onContaining( Object *obj, Bool wasSelected );
	virtual void onRemoving( Object *obj );

protected:

	void pickVisualLocation( Coord3D *pos );		///< pick a location inside the prison yard
	void addVisual( Object *obj );							///< add prisoner visual
	void removeVisual( Object *obj );						///< remove prisoner visual

	PrisonVisual *m_visualList;									///< list of visual representation data (if needed)

};

#endif
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Behavior/PrisonBehavior.cpp`
```cpp
// FILE: PrisonBehavior.cpp ///////////////////////////////////////////////////////////////////////
// Author: Colin Day, August 2002
// Desc:   Prison Behavior
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine
#include "Common/GameState.h"
#include "Common/Player.h"
#include "Common/RandomValue.h"
#include "Common/ThingFactory.h"
#include "Common/Xfer.h"
#include "GameClient/Drawable.h"
#include "GameClient/InGameUI.h"
#include "GameClient/GameClient.h"
#include "GameClient/Line2D.h"
#include "GameLogic/Object.h"
#include "GameLogic/Module/PrisonBehavior.h"

#ifdef ALLOW_SURRENDER

///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
class PrisonVisual : public MemoryPoolObject
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( PrisonVisual, "PrisonVisual" )

public:

	PrisonVisual();
	// virtual destructor prototype provided by memory pool object

	ObjectID m_objectID;				///< object that is contained
	DrawableID m_drawableID;		///< associated visual prisoner drawable
	PrisonVisual *m_next;				///< next

};

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
PrisonVisual::PrisonVisual()
{

	m_objectID = INVALID_ID;
	m_drawableID = INVALID_DRAWABLE_ID;
	m_next = nullptr;

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
PrisonVisual::~PrisonVisual()
{

}

///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
PrisonBehaviorModuleData::PrisonBehaviorModuleData()
{

	m_showPrisoners = FALSE;

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
/*static*/ void PrisonBehaviorModuleData::buildFieldParse( MultiIniFieldParse &p )
{
  OpenContainModuleData::buildFieldParse( p );

	static const FieldParse dataFieldParse[] =
	{

		{ "ShowPrisoners",	INI::parseBool,					nullptr,		offsetof( PrisonBehaviorModuleData, m_showPrisoners ) },
		{ "YardBonePrefix",	INI::parseAsciiString,	nullptr,		offsetof( PrisonBehaviorModuleData, m_prisonYardBonePrefix ) },
		{ nullptr, nullptr, nullptr, 0 }

	};

  p.add( dataFieldParse );

}

///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
PrisonBehavior::PrisonBehavior( Thing *thing, const ModuleData *moduleData )
							: OpenContain( thing, moduleData )
{

	m_visualList = nullptr;

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
PrisonBehavior::~PrisonBehavior()
{

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void PrisonBehavior::onDelete()
{

	// extend functionality
	OpenContain::onDelete();

	// delete our list
	Drawable *draw;
	PrisonVisual *visual;
	while( m_visualList )
	{

		// delete drawable if found
		draw = TheGameClient->findDrawableByID( m_visualList->m_drawableID );
		if( draw )
			TheGameClient->destroyDrawable( draw );

		// delete element and set next to head
		visual = m_visualList->m_next;
		deleteInstance(m_visualList);
		m_visualList = visual;

	}

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void PrisonBehavior::onContaining( Object *obj, Bool wasSelected )
{

	// extend functionality
	OpenContain::onContaining( obj, wasSelected );

	// objects inside a prison are held
	obj->setDisabled( DISABLED_HELD );

	// if we show visuals, make one
	const PrisonBehaviorModuleData *modData = getPrisonBehaviorModuleData();
	if( modData->m_showPrisoners )
		addVisual( obj );

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void PrisonBehavior::onRemoving( Object *obj )
{

	// if we show visuals, remove one
	const PrisonBehaviorModuleData *modData = getPrisonBehaviorModuleData();
	if( modData->m_showPrisoners )
		removeVisual( obj );

	// object is no longer held inside a garrisoned building
	obj->clearDisabled( DISABLED_HELD );

	// extend functionality
	OpenContain::onRemoving( obj );

}

///////////////////////////////////////////////////////////////////////////////////////////////////
// PRIVATE ////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

// ------------------------------------------------------------------------------------------------
/** Pick a random location inside the prison yard */
// ------------------------------------------------------------------------------------------------
void PrisonBehavior::pickVisualLocation( Coord3D *pos )
{
	Object *us = getObject();
	const PrisonBehaviorModuleData *modData = getPrisonBehaviorModuleData();
	Int i;

	// sanity
	if( pos == nullptr )
		return;

	// initialize the picked location to that of the prison center
	Coord3D pickedLocation = *us->getPosition();

	// get the positions of the bones that make up the prison yard area
	const Int MAX_YARD_BONES = 16;
	Coord3D yardPositions[ MAX_YARD_BONES ];
	Int yardBones = us->getMultiLogicalBonePosition( modData->m_prisonYardBonePrefix.str(),
																									 MAX_YARD_BONES,
																									 yardPositions,
																									 nullptr );

	//
	// we must have at least 3 bone locations to make a yard polygon, otherwise we'll
	// default to the object position
	//
	if( yardBones >= 3 )
	{

		// find the bounding region of the yard area
		Region2D yardRegion;
		yardRegion.lo = yardPositions[ 0 ].asCoord2D();
		yardRegion.hi = yardPositions[ 0 ].asCoord2D();
		for( i = 1; i < yardBones; i++ )
		{

			yardRegion.uniteWith(yardPositions[ i ].asCoord2D());

		}

		//
		// now that we have a yard region, the default visual position will be in the middle
		// of the yard region instead of the position of our object
		//
		pickedLocation.x = yardRegion.lo.x + yardRegion.width() / 2.0f;
		pickedLocation.y = yardRegion.lo.y + yardRegion.height() / 2.0f;
		// NOTE: pickedLocation.z is left alone at the object center Z

		// loop till we find a valid location that is inside the yard area
		Int maxTries = 32;
		Coord3D loc;
		for( i = 0; i < maxTries; ++i )
		{

			// pick a location
			loc.x = GameLogicRandomValueReal( yardRegion.lo.x, yardRegion.hi.x );
			loc.y = GameLogicRandomValueReal( yardRegion.lo.y, yardRegion.hi.y );
			loc.z = pickedLocation.z;

			// must be inside the yard polygon
			if( PointInsideArea2D( &loc, yardPositions, yardBones ) == TRUE )
			{

				// use this location, leave Z alone as the center of the prison
				pickedLocation = loc;
				break;  // exit for i

			}

		}

	}

	// return the location picked
	*pos = pickedLocation;

}

// ------------------------------------------------------------------------------------------------
/** Add prisoner visual to the prison yard */
// ------------------------------------------------------------------------------------------------
void PrisonBehavior::addVisual( Object *obj )
{

	// sanity
	if( obj == nullptr )
		return;

	// create a drawable
	Drawable *draw = TheThingFactory->newDrawable( obj->getTemplate() );

	// set the color of the drawable to that of the object
	if (TheGlobalData->m_timeOfDay == TIME_OF_DAY_NIGHT)
		draw->setIndicatorColor( obj->getNightIndicatorColor() );
	else
		draw->setIndicatorColor( obj->getIndicatorColor() );

	// pick a location inside the prison yard
	Coord3D pos;
	pickVisualLocation( &pos );

	// place drawable within the prison yard area
	draw->setPosition( &pos );
	draw->setOrientation( GameLogicRandomValueReal( 0, TWO_PI ) );
	DrawableInfo *drawInfo=draw->getDrawableInfo();
	drawInfo->m_shroudStatusObjectID=getObject()->getID();

	// record this object/drawable pair
	PrisonVisual *visual = newInstance(PrisonVisual);
	visual->m_objectID = obj->getID();
	visual->m_drawableID = draw->getID();
	visual->m_next = m_visualList;
	m_visualList = visual;

}

// ------------------------------------------------------------------------------------------------
/** Remove prisoner visual from the prison yard */
// ------------------------------------------------------------------------------------------------
void PrisonBehavior::removeVisual( Object *obj )
{

	// sanity
	if( obj == nullptr )
		return;

	// initialize a drawable ID to invalid
	DrawableID drawableID = INVALID_DRAWABLE_ID;

	// find visual info in our list, once found, take this opportunity to remove it from that list
	PrisonVisual *visual, *prevVisual = nullptr;
	for( visual = m_visualList; visual; visual = visual->m_next )
	{

		// is this the one we're looking for
		if( visual->m_objectID == obj->getID() )
		{

			// record the information we need here
			drawableID = visual->m_drawableID;

			// remove from list
			if( prevVisual )
				prevVisual->m_next = visual->m_next;
			else
				m_visualList = visual->m_next;

			// delete the element
			deleteInstance(visual);

			break;  // exit for

		}

		// keep a pointer to the previous element
		prevVisual = visual;

	}

	// find the drawable visual and destroy it
	Drawable *draw = TheGameClient->findDrawableByID( drawableID );
	if( draw )
		TheGameClient->destroyDrawable( draw );

}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void PrisonBehavior::crc( Xfer *xfer )
{

	// extend base class
	OpenContain::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void PrisonBehavior::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	OpenContain::xfer( xfer );

	// count and data for the prison visuals
	UnsignedShort visualCount = 0;
	PrisonVisual *visual;
	for( visual = m_visualList; visual; visual = visual->m_next )
		visualCount++;
	xfer->xferUnsignedShort( &visualCount );
	if( xfer->getXferMode() == XFER_SAVE )
	{

		// write all data
		for( visual = m_visualList; visual; visual = visual->m_next )
		{

			// object id
			xfer->xferObjectID( &visual->m_objectID );

			// drawable id
			xfer->xferDrawableID( &visual->m_drawableID );

		}

	}
	else
	{

		// the visual list should be empty
		if( m_visualList != nullptr )
		{

			DEBUG_CRASH(( "PrisonBehavior::xfer - the visual list should be empty but is not" ));
			throw SC_INVALID_DATA;

		}

		// read each item
		for( UnsignedShort i = 0; i < visualCount; ++i )
		{

			// allocate a new visual and tie to list
			visual = newInstance(PrisonVisual);
			visual->m_next = m_visualList;
			m_visualList = visual;

			// read object id
			xfer->xferObjectID( &visual->m_objectID );

			// read drawable id
			xfer->xferDrawableID( &visual->m_drawableID );

		}

	}

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void PrisonBehavior::loadPostProcess()
{

	// extend base class
	OpenContain::loadPostProcess();

}

#endif
```
