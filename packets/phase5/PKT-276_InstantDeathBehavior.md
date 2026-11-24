# Translation Packet `PKT-276`: `InstantDeathBehavior`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-276`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/behavior/instant_death_behavior.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (18):**
  - `Common/GameLOD.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/GameLOD.h`
  - `Common/INI.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/INI.h`
  - `Common/RandomValue.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/RandomValue.h`
  - `Common/Thing.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Thing.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/FXList.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/FXList.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/DieModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DieModule.h`
  - `GameLogic/Module/InstantDeathBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/InstantDeathBehavior.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/ObjectCreationList.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ObjectCreationList.h`
  - `GameLogic/Weapon.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (3):**
  - `"FX" => parseFX`
  - `"OCL" => parseOCL`
  - `"Weapon" => parseWeapon`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (1 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
- **PRNG Calls (3 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValue(0, listSize-1)`
  - `GameLogicRandomValue(0, listSize-1)`
  - `GameLogicRandomValue(0, listSize-1)`
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if !RETAIL_COMPATIBLE_CRC`
  - `#if RETAIL_COMPATIBLE_CRC`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/InstantDeathBehavior.h`
```cpp
// FILE: InstantDeathBehavior.h /////////////////////////////////////////////////////////////////////////
// Author: Steven Johnson, Sep 2002
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/DieModule.h"

class FXList;
class ObjectCreationList;
class WeaponTemplate;
class DamageInfo;

//-------------------------------------------------------------------------------------------------
typedef std::vector<const FXList*> FXListVec;
typedef std::vector<const ObjectCreationList*> OCLVec;
typedef std::vector<const WeaponTemplate*> WeaponTemplateVec;

//-------------------------------------------------------------------------------------------------
class InstantDeathBehaviorModuleData : public DieModuleData
{
public:
	FXListVec					m_fx;
	OCLVec						m_ocls;
	WeaponTemplateVec	m_weapons;

	InstantDeathBehaviorModuleData();
	static void buildFieldParse(MultiIniFieldParse& p);

private:

};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class InstantDeathBehavior : public DieModule
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( InstantDeathBehavior, "InstantDeathBehavior" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( InstantDeathBehavior, InstantDeathBehaviorModuleData )

public:

	InstantDeathBehavior( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	// DieModuleInterface
	virtual void onDie( const DamageInfo *damageInfo ) override;

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Behavior/InstantDeathBehavior.cpp`
```cpp
// FILE: InstantDeathBehavior.cpp ///////////////////////////////////////////////////////////////////////
// Author:
// Desc:
///////////////////////////////////////////////////////////////////////////////////////////////////


// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine
#define DEFINE_SLOWDEATHPHASE_NAMES

#include "Common/Thing.h"
#include "Common/ThingTemplate.h"
#include "Common/INI.h"
#include "Common/RandomValue.h"
#include "Common/GameLOD.h"
#include "Common/Xfer.h"
#include "GameClient/Drawable.h"
#include "GameClient/FXList.h"
#include "GameClient/InGameUI.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/InstantDeathBehavior.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectCreationList.h"
#include "GameLogic/Weapon.h"

//-------------------------------------------------------------------------------------------------
InstantDeathBehaviorModuleData::InstantDeathBehaviorModuleData()
{
	// redundant.
	//m_fx.clear();
	//m_ocls.clear();
	//m_weapons.clear();
}

//-------------------------------------------------------------------------------------------------
static void parseFX( INI* ini, void *instance, void * /*store*/, const void* /*userData*/ )
{
	InstantDeathBehaviorModuleData* self = (InstantDeathBehaviorModuleData*)instance;
	for (const char* token = ini->getNextToken(); token; token = ini->getNextTokenOrNull())
	{
		const FXList *fxl = TheFXListStore->findFXList((token));	// could be null! this is OK!
		self->m_fx.push_back(fxl);
	}
}

//-------------------------------------------------------------------------------------------------
static void parseOCL( INI* ini, void *instance, void * /*store*/, const void* /*userData*/ )
{
	InstantDeathBehaviorModuleData* self = (InstantDeathBehaviorModuleData*)instance;
	for (const char* token = ini->getNextToken(); token; token = ini->getNextTokenOrNull())
	{
		const ObjectCreationList *ocl = TheObjectCreationListStore->findObjectCreationList(token);	// could be null! this is OK!
		self->m_ocls.push_back(ocl);
	}
}

//-------------------------------------------------------------------------------------------------
static void parseWeapon( INI* ini, void *instance, void * /*store*/, const void* /*userData*/ )
{
	InstantDeathBehaviorModuleData* self = (InstantDeathBehaviorModuleData*)instance;
	for (const char* token = ini->getNextToken(); token; token = ini->getNextTokenOrNull())
	{
		const WeaponTemplate *wt = TheWeaponStore->findWeaponTemplate(token);	// could be null! this is OK!
		self->m_weapons.push_back(wt);
	}
}

//-------------------------------------------------------------------------------------------------
/*static*/ void InstantDeathBehaviorModuleData::buildFieldParse(MultiIniFieldParse& p)
{
  DieModuleData::buildFieldParse(p);

	static const FieldParse dataFieldParse[] =
	{
		{ "FX",										parseFX,													nullptr, 0 },
		{ "OCL",									parseOCL,													nullptr, 0 },
		{ "Weapon",								parseWeapon,											nullptr, 0 },
		{ nullptr, nullptr, nullptr, 0 }
	};
  p.add(dataFieldParse);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
InstantDeathBehavior::InstantDeathBehavior( Thing *thing, const ModuleData* moduleData ) : DieModule( thing, moduleData )
{
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
InstantDeathBehavior::~InstantDeathBehavior()
{
}

//-------------------------------------------------------------------------------------------------
void InstantDeathBehavior::onDie( const DamageInfo *damageInfo )
{
	if (!isDieApplicable(damageInfo))
		return;

#if RETAIL_COMPATIBLE_CRC
	AIUpdateInterface* ai = getObject()->getAIUpdateInterface();
	if (ai)
	{
		// has another AI already handled us. (hopefully another InstantDeathBehavior)
		if (ai->isAiInDeadState())
			return;
		ai->markAsDead();
	}
#endif

	const InstantDeathBehaviorModuleData* d = getInstantDeathBehaviorModuleData();

	size_t idx, listSize;

	listSize = d->m_fx.size();
	if (listSize > 0)
	{
		idx = (size_t)GameLogicRandomValue(0, listSize-1);
		const FXListVec& v = d->m_fx;
		DEBUG_ASSERTCRASH(idx>=0&&idx<v.size(),("bad idx"));
		const FXList* fxl = v[idx];
		FXList::doFXObj(fxl, getObject(), nullptr);
	}

	listSize = d->m_ocls.size();
	if (listSize > 0)
	{
		idx = (size_t)GameLogicRandomValue(0, listSize-1);
		const OCLVec& v = d->m_ocls;
		DEBUG_ASSERTCRASH(idx>=0&&idx<v.size(),("bad idx"));
		const ObjectCreationList* ocl = v[idx];
		ObjectCreationList::create(ocl, getObject(), nullptr);
	}

#if !RETAIL_COMPATIBLE_CRC
	// TheSuperHackers @bugfix Stubbjax 23/07/2026 Prevent firing weapons when a scaffold is cancelled
	// or destroyed. This logic matches the condition in the FireWeaponWhenDeadBehavior module.
	if (!getObject()->getStatusBits().test(OBJECT_STATUS_UNDER_CONSTRUCTION))
#endif
	{
		listSize = d->m_weapons.size();
		if (listSize > 0)
		{
			idx = (size_t)GameLogicRandomValue(0, listSize-1);
			const WeaponTemplateVec& v = d->m_weapons;
			DEBUG_ASSERTCRASH(idx>=0&&idx<v.size(),("bad idx"));
			const WeaponTemplate* wt = v[idx];
			if (wt)
			{
				TheWeaponStore->createAndFireTempWeapon(wt, getObject(), getObject()->getPosition());
			}
		}
	}

#if !RETAIL_COMPATIBLE_CRC
	// TheSuperHackers @bugfix Stubbjax 21/07/2026 Allow multiple InstantDeathBehaviors to be processed.
  // Multiple FXListDie, CreateObjectDie and FireWeaponWhenDeadBehavior modules are already supported.
	AIUpdateInterface* ai = getObject()->getAIUpdateInterface();
	if (ai)
	{
		// has another AI already handled us. (hopefully another InstantDeathBehavior)
		if (ai->isAiInDeadState())
			return;
		ai->markAsDead();
	}
#endif

	TheGameLogic->destroyObject(getObject());
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void InstantDeathBehavior::crc( Xfer *xfer )
{

	// extend base class
	DieModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void InstantDeathBehavior::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	DieModule::xfer( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void InstantDeathBehavior::loadPostProcess()
{

	// extend base class
	DieModule::loadPostProcess();

}
```
