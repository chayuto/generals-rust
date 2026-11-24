# Translation Packet `PKT-253`: `UpgradeModule`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-253`
- **Work Package:** `WP-5.2` (Phase 5)
- **Complexity & Assigned Tier:** Class `C1` -> `tier:1a-lite`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/upgrade/upgrade_module.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (9):**
  - `Common/Module.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Module.h`
  - `Common/STLTypedefs.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/STLTypedefs.h`
  - `Common/Upgrade.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Upgrade.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/FXList.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/FXList.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/UpgradeModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpgradeModule.h`
  - `PreRTS.h`
- **Implemented Module Facets (1):**
  - `UpgradeModuleInterface* getUpgrade()`
- **INI `FieldParse` Table Entries (5):**
  - `"TriggeredBy" => INI::parseAsciiStringVector`
  - `"ConflictsWith" => INI::parseAsciiStringVector`
  - `"RemovesUpgrades" => INI::parseAsciiStringVector`
  - `"FXListUpgrade" => INI::parseFXList`
  - `"RequiresAllTriggers" => INI::parseBool`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (3 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferVersion( &version, currentVersion )`
  3. `xferBool( &m_upgradeExecuted )`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpgradeModule.h`
```cpp
// FILE: UpgradeModule.h /////////////////////////////////////////////////////////////////////////////////
// Author: Graham Smallwood, March 2002
// Desc:	 A Module that responds to PlayerUpgrade Bitsetting by executing code once and only once
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

#include "Common/Module.h"
#include "Common/STLTypedefs.h"
#include "Common/Upgrade.h"

#include "GameClient/Drawable.h"
#include "GameClient/FXList.h"

#include "GameLogic/Module/BehaviorModule.h"

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
class Player;

//-------------------------------------------------------------------------------------------------
/** OBJECT DIE MODULE base class */
//-------------------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------------------
class UpgradeModuleInterface
{
public:

 	virtual Bool isAlreadyUpgraded() const = 0;
	virtual Bool attemptUpgrade( const UpgradeMaskType& keyMask ) = 0;
	virtual Bool wouldUpgrade( const UpgradeMaskType& keyMask ) const = 0;
	virtual Bool resetUpgrade( const UpgradeMaskType& keyMask ) = 0;
	virtual Bool isSubObjectsUpgrade() = 0;
	virtual void forceRefreshUpgrade() = 0;
	virtual Bool testUpgradeConditions( const UpgradeMaskType& keyMask ) const = 0;

};

//-------------------------------------------------------------------------------------------------
class UpgradeMuxData	// does NOT inherit from ModuleData.
{
public:
	mutable std::vector<AsciiString>	m_triggerUpgradeNames;
	mutable std::vector<AsciiString>	m_activationUpgradeNames;
	mutable std::vector<AsciiString>	m_conflictingUpgradeNames;
	mutable std::vector<AsciiString>	m_removalUpgradeNames;

	mutable const FXList*							m_fxListUpgrade;
	mutable UpgradeMaskType						m_activationMask;				///< Activation only supports a single name currently
	mutable UpgradeMaskType						m_conflictingMask;			///< Conflicts support multiple listings, and they are an OR
	mutable Bool											m_requiresAllTriggers;

	UpgradeMuxData()
	{
		m_triggerUpgradeNames.clear();
		m_activationUpgradeNames.clear();
		m_conflictingUpgradeNames.clear();
		m_removalUpgradeNames.clear();

		m_fxListUpgrade = nullptr;
		m_activationMask.clear();
		m_conflictingMask.clear();
		m_requiresAllTriggers = false;
	}

	static const FieldParse* getFieldParse()
	{
		static const FieldParse dataFieldParse[] =
		{
			{ "TriggeredBy",		INI::parseAsciiStringVector, nullptr, offsetof( UpgradeMuxData, m_activationUpgradeNames ) },
			{ "ConflictsWith",	INI::parseAsciiStringVector, nullptr, offsetof( UpgradeMuxData, m_conflictingUpgradeNames ) },
			{ "RemovesUpgrades",INI::parseAsciiStringVector, nullptr, offsetof( UpgradeMuxData, m_removalUpgradeNames ) },
			{ "FXListUpgrade",	INI::parseFXList, nullptr, offsetof( UpgradeMuxData, m_fxListUpgrade ) },
			{ "RequiresAllTriggers", INI::parseBool, nullptr, offsetof( UpgradeMuxData, m_requiresAllTriggers ) },
			{ nullptr, nullptr, nullptr, 0 }
		};
		return dataFieldParse;
	}
	Bool requiresAllActivationUpgrades() const;
	void getUpgradeActivationMasks(UpgradeMaskType& activation, UpgradeMaskType& conflicting) const;	///< The first time someone looks at my mask, I'll figure it out.
	void performUpgradeFX(Object* obj) const;
	void muxDataProcessUpgradeRemoval(Object* obj) const;
	Bool isTriggeredBy(const std::string &upgrade) const;
};

//-------------------------------------------------------------------------------------------------
// implements some handy mix-in guts, but doesn't descend from ObjectModule... useful for Behaviors
// that want to implement Upgrades
class UpgradeMux : public UpgradeModuleInterface
{
public:

	UpgradeMux();

	virtual Bool isAlreadyUpgraded() const override ;
	// ***DANGER! DANGER! Don't use this, unless you are forcing an already made upgrade to refresh!!
	virtual void forceRefreshUpgrade() override;
	virtual Bool attemptUpgrade( const UpgradeMaskType& keyMask ) override;
	virtual Bool wouldUpgrade( const UpgradeMaskType& keyMask ) const override;
	virtual Bool resetUpgrade( const UpgradeMaskType& keyMask ) override;
	virtual Bool testUpgradeConditions( const UpgradeMaskType& keyMask ) const override;

protected:

	void setUpgradeExecuted(Bool e) { m_upgradeExecuted = e; }
	virtual void upgradeImplementation() = 0; ///< Here's the actual work of Upgrading
	virtual void getUpgradeActivationMasks(UpgradeMaskType& activation, UpgradeMaskType& conflicting) const = 0; ///< Here's the actual work of Upgrading
	virtual void performUpgradeFX() = 0;	///< perform the associated fx list
	virtual Bool requiresAllActivationUpgrades() const = 0;
	virtual void processUpgradeRemoval() = 0;

	void giveSelfUpgrade();

	//
	// this is not a snapshot class itself, but it is a base class used in conjunction with
	// multiple inheritance for modules, so those modules need to be able to poke inside
	// this class and xfer the data here during a snapshot process
	//
	virtual void upgradeMuxCRC( Xfer *xfer );
	virtual void upgradeMuxXfer( Xfer *xfer);
	virtual void upgradeMuxLoadPostProcess();

private:
	Bool m_upgradeExecuted;				///< Upgrade only executes once

};

//-------------------------------------------------------------------------------------------------
struct UpgradeModuleData : public BehaviorModuleData
{
public:
	UpgradeMuxData				m_upgradeMuxData;

	static void buildFieldParse(MultiIniFieldParse& p)
	{
    ModuleData::buildFieldParse(p);
		p.add(UpgradeMuxData::getFieldParse(), offsetof( UpgradeModuleData, m_upgradeMuxData ));
	}
};

//-------------------------------------------------------------------------------------------------
class UpgradeModule : public BehaviorModule, public UpgradeMux
{

	MEMORY_POOL_GLUE_ABC( UpgradeModule )
	MAKE_STANDARD_MODULE_MACRO_ABC( UpgradeModule )
	MAKE_STANDARD_MODULE_DATA_MACRO_ABC(UpgradeModule, UpgradeModuleData)

public:

	UpgradeModule( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype defined by MemoryPoolObject

	// module methods
	static Int getInterfaceMask() { return MODULEINTERFACE_UPGRADE; }

	// BehaviorModule
	virtual UpgradeModuleInterface* getUpgrade() override { return this; }

	bool isTriggeredBy(const std::string & upgrade) const { return getUpgradeModuleData()->m_upgradeMuxData.isTriggeredBy(upgrade); }

protected:

	virtual void processUpgradeRemoval() override
	{
		// I can't take it any more.  Let the record show that I think the UpgradeMux multiple inheritance is CRAP.
		getUpgradeModuleData()->m_upgradeMuxData.muxDataProcessUpgradeRemoval(getObject());
	}

	virtual Bool requiresAllActivationUpgrades() const override
	{
		return getUpgradeModuleData()->m_upgradeMuxData.m_requiresAllTriggers;
	}

	virtual void getUpgradeActivationMasks(UpgradeMaskType& activation, UpgradeMaskType& conflicting) const override
	{
		getUpgradeModuleData()->m_upgradeMuxData.getUpgradeActivationMasks(activation, conflicting);
	}

	virtual void performUpgradeFX() override
	{
		getUpgradeModuleData()->m_upgradeMuxData.performUpgradeFX(getObject());
	}

};
inline UpgradeModule::UpgradeModule( Thing *thing, const ModuleData* moduleData ) : BehaviorModule( thing, moduleData ) { }
inline UpgradeModule::~UpgradeModule() { }
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Upgrade/UpgradeModule.cpp`
```cpp
// FILE: UpgradeModule.cpp ////////////////////////////////////////////////////////////////////////
// Author: Johnson, Day, Smallwood September 2002
// Desc:   Upgrade module basic implementations
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"
#include "Common/Xfer.h"
#include "GameLogic/Module/UpgradeModule.h"


// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void UpgradeModule::crc( Xfer *xfer )
{

	// extend base class
	BehaviorModule::crc( xfer );

	// extned base class
	UpgradeMux::upgradeMuxCRC( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer Method */
// ------------------------------------------------------------------------------------------------
void UpgradeModule::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// call base class
	BehaviorModule::xfer( xfer );

	// extend base class
	UpgradeMux::upgradeMuxXfer( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void UpgradeModule::loadPostProcess()
{

	// call base class
	BehaviorModule::loadPostProcess();

	// extend base class
	UpgradeMux::upgradeMuxLoadPostProcess();

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
UpgradeMux::UpgradeMux() : m_upgradeExecuted(false)
{

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
Bool UpgradeMux::isAlreadyUpgraded() const
{
	return m_upgradeExecuted;
}

// ------------------------------------------------------------------------------------------------
// ***DANGER! DANGER! Don't use this, unless you are forcing an already made upgrade to refresh!!
// ------------------------------------------------------------------------------------------------
void UpgradeMux::forceRefreshUpgrade()
{
	if( m_upgradeExecuted )
	{
		//Only do this if we've already made the upgrade!
		upgradeImplementation();
	}
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
Bool UpgradeMux::attemptUpgrade( const UpgradeMaskType& keyMask )
{
	if (wouldUpgrade(keyMask))
	{
		// If I have an activation condition, and I haven't activated, and this key matches my condition.
		giveSelfUpgrade();
		return true;
	}
	return false;
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
Bool UpgradeMux::wouldUpgrade( const UpgradeMaskType& keyMask ) const
{
	UpgradeMaskType activation, conflicting;
	getUpgradeActivationMasks(activation, conflicting);

	//Make sure we have activation conditions and we haven't performed the upgrade already.
	if( activation.any() && keyMask.any() && !m_upgradeExecuted )
	{
		//Okay, make sure we don't have any conflicting upgrades
		if( !keyMask.testForAny( conflicting) )
		{
			//Finally check to see if our upgrade conditions match.
			if( requiresAllActivationUpgrades() )
			{
				//Make sure ALL triggers requirements are upgraded
				if( keyMask.testForAll( activation ) )
				{
					return TRUE;
				}
			}
			else
			{
				//Check if ANY trigger requirements are met.
				if( keyMask.testForAny( activation ) )
				{
					return TRUE;
				}
			}
		}
	}
	//We can't upgrade!
	return FALSE;
}

//-------------------------------------------------------------------------------------------------
void UpgradeMux::giveSelfUpgrade()
{
	// If I have an activation condition, and I haven't activated, and this key matches my condition.
	performUpgradeFX();
	processUpgradeRemoval();// Need to execute removals first, to prevent both being on for a moment.
	upgradeImplementation();
	setUpgradeExecuted(true);
}

//-------------------------------------------------------------------------------------------------
Bool UpgradeMux::testUpgradeConditions( const UpgradeMaskType& keyMask ) const
{
	UpgradeMaskType activation, conflicting;
	getUpgradeActivationMasks(activation, conflicting);

	//Okay, make sure we don't have any conflicting upgrades
	if( keyMask.testForNone( conflicting ) )
	{
		//Make sure we have activation conditions
		if( activation.any() )
		{
			//Finally check to see if our upgrade conditions match.
			if( requiresAllActivationUpgrades() )
			{
				//Make sure ALL triggers requirements are upgraded
				if( keyMask.testForAll( activation ) )
				{
					return TRUE;
				}
			}
			else
			{
				//Check if ANY trigger requirements are met.
				if( keyMask.testForAny( activation ) )
				{
					return TRUE;
				}
			}
		}
		else
		{
			//This *upgrade* is relying only on not having conflicts.
			return true;
		}
	}
	//We can't upgrade!
	return false;
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
Bool UpgradeMux::resetUpgrade( const UpgradeMaskType& keyMask )
{
	UpgradeMaskType activation, conflicting;
	getUpgradeActivationMasks(activation, conflicting);
	if( keyMask.testForAny( activation ) && m_upgradeExecuted )
	{
		m_upgradeExecuted = false;
		return true;
	}
	return false;
}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void UpgradeMux::upgradeMuxCRC( Xfer *xfer )
{

	// just call the regular xfer, it's simple
	upgradeMuxXfer( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer
	* Version Info
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void UpgradeMux::upgradeMuxXfer( Xfer *xfer )
{

	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// upgrade executed
	xfer->xferBool( &m_upgradeExecuted );

}

// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void UpgradeMux::upgradeMuxLoadPostProcess()
{

}
```
