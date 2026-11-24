# Translation Packet `PKT-262`: `BridgeTowerBehavior`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-262`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/behavior/bridge_tower_behavior.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (13):**
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/BridgeBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BridgeBehavior.h`
  - `GameLogic/Module/BridgeTowerBehavior.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BridgeTowerBehavior.h`
  - `GameLogic/Module/DamageModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DamageModule.h`
  - `GameLogic/Module/DieModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DieModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/TerrainLogic.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/TerrainLogic.h`
  - `PreRTS.h`
- **Implemented Module Facets (3):**
  - `BridgeTowerBehaviorInterface* getBridgeTowerBehaviorInterface()`
  - `DamageModuleInterface* getDamage()`
  - `DieModuleInterface* getDie()`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (3 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferObjectID( &m_bridgeID )`
  3. `xferUser( &m_type, sizeof( BridgeTowerType ) )`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BridgeTowerBehavior.h`
```cpp
// FILE: BridgeTowerBehavior.h ////////////////////////////////////////////////////////////////////
// Author: Colin Day, July 2002
// Desc:   Behavior module for the towers attached to bridges that can be targeted
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/DamageModule.h"
#include "GameLogic/Module/DieModule.h"

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
class BridgeTowerBehaviorInterface
{

public:

	virtual void setBridge( Object *bridge ) = 0;
	virtual ObjectID getBridgeID() = 0;
	virtual void setTowerType( BridgeTowerType type ) = 0;

};

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
class BridgeTowerBehavior : public BehaviorModule,
														public BridgeTowerBehaviorInterface,
														public DieModuleInterface,
														public DamageModuleInterface
{

	MAKE_STANDARD_MODULE_MACRO( BridgeTowerBehavior );
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( BridgeTowerBehavior, "BridgeTowerBehavior" )

public:

	BridgeTowerBehavior( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	static Int getInterfaceMask() { return (MODULEINTERFACE_DAMAGE) | (MODULEINTERFACE_DIE); }
	virtual BridgeTowerBehaviorInterface* getBridgeTowerBehaviorInterface() override { return this; }

	virtual void setBridge( Object *bridge ) override;
	virtual ObjectID getBridgeID() override;
	virtual void setTowerType( BridgeTowerType type ) override;

	static BridgeTowerBehaviorInterface *getBridgeTowerBehaviorInterfaceFromObject( Object *obj );

	// Damage methods
	virtual DamageModuleInterface* getDamage() override { return this; }
	virtual void onDamage( DamageInfo *damageInfo ) override;
	virtual void onHealing( DamageInfo *damageInfo ) override;
	virtual void onBodyDamageStateChange( const DamageInfo* damageInfo,
																				BodyDamageType oldState,
																				BodyDamageType newState ) override;

	// Die methods
	virtual DieModuleInterface* getDie() override { return this; }
	virtual void onDie( const DamageInfo *damageInfo ) override;

protected:

	ObjectID m_bridgeID;					///< the bridge we're a part of
	BridgeTowerType m_type;				///< type of tower (positioning) we are

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Behavior/BridgeTowerBehavior.cpp`
```cpp
// FILE: BridgeTowerBehavior.cpp //////////////////////////////////////////////////////////////////
// Author: Colin Day, July 2002
// Desc:   Behavior module for the towers attached to bridges that can be targeted
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"
#include "GameClient/InGameUI.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/BridgeBehavior.h"
#include "GameLogic/Module/BridgeTowerBehavior.h"
#include "GameLogic/TerrainLogic.h"

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
BridgeTowerBehavior::BridgeTowerBehavior( Thing *thing, const ModuleData *moduleData )
									 : BehaviorModule( thing, moduleData )
{

	m_bridgeID = INVALID_ID;

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
BridgeTowerBehavior::~BridgeTowerBehavior()
{

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void BridgeTowerBehavior::setBridge( Object *bridge )
{

	if( bridge == nullptr )
		m_bridgeID = INVALID_ID;
	else
		m_bridgeID = bridge->getID();

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
ObjectID BridgeTowerBehavior::getBridgeID()
{

	return m_bridgeID;

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void BridgeTowerBehavior::setTowerType( BridgeTowerType type )
{

	m_type = type;

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void BridgeTowerBehavior::onDamage( DamageInfo *damageInfo )
{
	Object *bridge = TheGameLogic->findObjectByID( getBridgeID() );

	// sanity
	if( bridge == nullptr )
		return;

	//
	// get our body info so we now how much damage percent is being done to us ... we need this
	// so that we can propagate the same damage percentage amount the towers and the bridge
	//
	BodyModuleInterface *body = getObject()->getBodyModule();
	Real damagePercentage = damageInfo->in.m_amount / body->getMaxHealth();

	// get the bridge behavior module for our bridge
	BehaviorModule **bmi;
	BridgeBehaviorInterface *bridgeInterface = nullptr;
	for( bmi = bridge->getBehaviorModules(); *bmi; ++bmi )
	{

		bridgeInterface = (*bmi)->getBridgeBehaviorInterface();
		if( bridgeInterface )
			break;

	}
	DEBUG_ASSERTCRASH( bridgeInterface != nullptr, ("BridgeTowerBehavior::onDamage - no 'BridgeBehaviorInterface' found") );
	if( bridgeInterface )
	{

		//
		// damage each of the other towers if the source of this damage isn't from the bridge
		// or other towers
		//
		Object *source = TheGameLogic->findObjectByID( damageInfo->in.m_sourceID );
		if( source == nullptr ||
			  (source->isKindOf( KINDOF_BRIDGE ) == FALSE &&
				 source->isKindOf( KINDOF_BRIDGE_TOWER ) == FALSE) )
		{

			for( Int i = 0; i < BRIDGE_MAX_TOWERS; ++i )
			{
				Object *tower;

				tower = TheGameLogic->findObjectByID( bridgeInterface->getTowerID( (BridgeTowerType)i ) );
				if( tower && tower != getObject() )
				{
					BodyModuleInterface *towerBody = tower->getBodyModule();
					DamageInfo towerDamage;

					towerDamage.in.m_amount = damagePercentage * towerBody->getMaxHealth();
					towerDamage.in.m_sourceID = getObject()->getID();  // we're now the source
					towerDamage.in.m_damageType = damageInfo->in.m_damageType;
					towerDamage.in.m_deathType = damageInfo->in.m_deathType;
					tower->attemptDamage( &towerDamage );

				}

			}

			//
			// damage bridge object, but make sure it's done through the bridge interface
			// so that it doesn't automatically propagate that damage to the towers
			//
			BodyModuleInterface *bridgeBody = bridge->getBodyModule();
			DamageInfo bridgeDamage;

			bridgeDamage.in.m_amount = damagePercentage * bridgeBody->getMaxHealth();
			bridgeDamage.in.m_sourceID = getObject()->getID();  // we're now the source
			bridgeDamage.in.m_damageType = damageInfo->in.m_damageType;
			bridgeDamage.in.m_deathType = damageInfo->in.m_deathType;
			bridge->attemptDamage( &bridgeDamage );

		}

	}

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void BridgeTowerBehavior::onHealing( DamageInfo *damageInfo )
{
	Object *bridge = TheGameLogic->findObjectByID( getBridgeID() );

	// sanity
	if( bridge == nullptr )
		return;

	//
	// get our body info so we now how much healing percent is being done to us ... we need this
	// so that we can propagate the same healing percentage amount the towers and the bridge
	//
	BodyModuleInterface *body = getObject()->getBodyModule();
	Real healingPercentage = damageInfo->in.m_amount / body->getMaxHealth();

	// get the bridge behavior module for our bridge
	BehaviorModule **bmi;
	BridgeBehaviorInterface *bridgeInterface = nullptr;
	for( bmi = bridge->getBehaviorModules(); *bmi; ++bmi )
	{

		bridgeInterface = (*bmi)->getBridgeBehaviorInterface();
		if( bridgeInterface )
			break;

	}
	DEBUG_ASSERTCRASH( bridgeInterface != nullptr, ("BridgeTowerBehavior::onHealing - no 'BridgeBehaviorInterface' found") );
	if( bridgeInterface )
	{

		//
		// heal each of the other towers if the source of this healing isn't from the bridge
		// or other towers
		//
		Object *source = TheGameLogic->findObjectByID( damageInfo->in.m_sourceID );
		if( source == nullptr ||
			  (source->isKindOf( KINDOF_BRIDGE ) == FALSE &&
				 source->isKindOf( KINDOF_BRIDGE_TOWER ) == FALSE) )
		{

			for( Int i = 0; i < BRIDGE_MAX_TOWERS; ++i )
			{
				Object *tower;

				tower = TheGameLogic->findObjectByID( bridgeInterface->getTowerID( (BridgeTowerType)i ) );
				if( tower && tower != getObject() )
				{
					BodyModuleInterface *towerBody = tower->getBodyModule();
					tower->attemptHealing(healingPercentage * towerBody->getMaxHealth(), getObject());

				}

			}

			//
			// heal bridge object, but make sure it's done through the bridge interface
			// so that it doesn't automatically propagate that healing to the towers.
			//
			BodyModuleInterface *bridgeBody = bridge->getBodyModule();
			bridge->attemptHealing(healingPercentage * bridgeBody->getMaxHealth(), getObject());

		}

	}

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void BridgeTowerBehavior::onBodyDamageStateChange( const DamageInfo* damageInfo,
																									 BodyDamageType oldState,
																									 BodyDamageType newState )
{

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void BridgeTowerBehavior::onDie( const DamageInfo *damageInfo )
{

	// kill the bridge object, this will kill all the towers
	Object *bridge = TheGameLogic->findObjectByID( getBridgeID() );
	if( bridge )
		bridge->kill();

}

///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

// ------------------------------------------------------------------------------------------------
/** Given an object, return a bridge tower interface if that object has one */
// ------------------------------------------------------------------------------------------------
BridgeTowerBehaviorInterface *BridgeTowerBehavior::getBridgeTowerBehaviorInterfaceFromObject( Object *obj )
{

	// sanity
	if( obj == nullptr || obj->isKindOf( KINDOF_BRIDGE_TOWER ) == FALSE )
		return nullptr;

	BehaviorModule **bmi;
	BridgeTowerBehaviorInterface *bridgeTowerInterface = nullptr;
	for( bmi = obj->getBehaviorModules(); *bmi; ++bmi )
	{

		bridgeTowerInterface = (*bmi)->getBridgeTowerBehaviorInterface();
		if( bridgeTowerInterface )
			return bridgeTowerInterface;

	}

	// interface not found
	return nullptr;

}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void BridgeTowerBehavior::crc( Xfer *xfer )
{

	// extend base class
	BehaviorModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void BridgeTowerBehavior::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	BehaviorModule::xfer( xfer );

	// xfer bridge object ID
	xfer->xferObjectID( &m_bridgeID );

	// xfer tower type
	xfer->xferUser( &m_type, sizeof( BridgeTowerType ) );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void BridgeTowerBehavior::loadPostProcess()
{

	// extend base class
	BehaviorModule::loadPostProcess();

}
```
