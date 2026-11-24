# Translation Packet `PKT-244`: `SpecialPowerModule`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-244`
- **Work Package:** `WP-5.2` (Phase 5)
- **Complexity & Assigned Tier:** Class `C1` -> `tier:1a-lite`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/special_power/special_power_module.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (25):**
  - `Common/AudioEventRTS.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/AudioEventRTS.h`
  - `Common/GameAudio.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameAudio.h`
  - `Common/GameUtility.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/GameUtility.h`
  - `Common/GlobalData.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/GlobalData.h`
  - `Common/INI.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/INI.h`
  - `Common/Module.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Module.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/PlayerList.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/PlayerList.h`
  - `Common/Science.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Science.h`
  - `Common/SpecialPower.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/SpecialPower.h`
  - `Common/ThingFactory.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ThingFactory.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/ControlBar.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/ControlBar.h`
  - `GameClient/Eva.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Eva.h`
  - `GameClient/InGameUI.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/InGameUI.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/BehaviorModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/DeletionUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DeletionUpdate.h`
  - `GameLogic/Module/SpecialPowerModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpecialPowerModule.h`
  - `GameLogic/Module/SpecialPowerUpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpecialPowerUpdateModule.h`
  - `GameLogic/Module/UpdateModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/ScriptEngine.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ScriptEngine.h`
  - `PreRTS.h`
- **Implemented Module Facets (1):**
  - `SpecialPowerModuleInterface* getSpecialPower()`
- **INI `FieldParse` Table Entries (6):**
  - `"SpecialPowerTemplate" => INI::parseSpecialPowerTemplate`
  - `"UpdateModuleStartsAttack" => INI::parseBool`
  - `"StartsPaused" => INI::parseBool`
  - `"StartsReady" => INI::parseBool`
  - `"InitiateSound" => INI::parseAudioEventRTS`
  - `"ScriptedSpecialPowerOnly" => INI::parseBool`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (5 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUnsignedInt( &m_availableOnFrame )`
  3. `xferInt( &m_pausedCount )`
  4. `xferUnsignedInt( &m_pausedOnFrame )`
  5. `xferReal( &m_pausedPercent )`
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/SpecialPowerModule.h`
```cpp
// FILE: SpecialPowerModule.h /////////////////////////////////////////////////////////////////////
// Author: Colin Day, April 2002
// Desc:	 Special power module interface
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "Common/AudioEventRTS.h"
#include "Common/Module.h"
#include "Common/Science.h"
#include "GameLogic/Module/BehaviorModule.h"

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
class Object;
class SpecialPowerTemplate;
struct FieldParse;


//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class SpecialPowerModuleInterface
{
public:

	virtual Bool isModuleForPower( const SpecialPowerTemplate *specialPowerTemplate ) const = 0;
	virtual Bool isReady() const = 0;
//  This is the althernate way to one-at-a-time BlackLotus' specials; we'll keep it commented her until Dustin decides, or until 12/10/02
//	virtual Bool isBusy() const = 0;
	virtual Real getPercentReady() const = 0;
	virtual UnsignedInt getReadyFrame() const = 0;
	virtual AsciiString getPowerName() const = 0;
	virtual const SpecialPowerTemplate* getSpecialPowerTemplate() const = 0;
	virtual ScienceType getRequiredScience() const = 0;
	virtual void onSpecialPowerCreation() = 0;
	virtual void setReadyFrame( UnsignedInt frame ) = 0;
	virtual void pauseCountdown( Bool pause ) = 0;
	virtual void doSpecialPower( UnsignedInt commandOptions ) = 0;
	virtual void doSpecialPowerAtObject( Object *obj, UnsignedInt commandOptions ) = 0;
	virtual void doSpecialPowerAtLocation( const Coord3D *loc, Real angle, UnsignedInt commandOptions ) = 0;
	virtual void doSpecialPowerUsingWaypoints( const Waypoint *way, UnsignedInt commandOptions ) = 0;
	virtual void markSpecialPowerTriggered( const Coord3D *location ) = 0;
	virtual void startPowerRecharge() = 0;
	virtual const AudioEventRTS& getInitiateSound() const = 0;
	virtual Bool startsReady() const = 0;
	virtual Bool isScriptOnly() const = 0;

	//If the special power launches a construction site, we need to know the final product for placement purposes.
	virtual const ThingTemplate* getReferenceThingTemplate() const = 0;
};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class SpecialPowerModuleData : public BehaviorModuleData
{
public:

	SpecialPowerModuleData();

	static void buildFieldParse(MultiIniFieldParse& p);

	const SpecialPowerTemplate *m_specialPowerTemplate;		///< pointer to the special power template
	AudioEventRTS			m_initiateSound;
	Bool							m_updateModuleStartsAttack;	///< update module determines when the special power actually starts! If true, update module is required.
	Bool							m_startsPaused; ///< Paused on creation, someone else will have to unpause (like upgrade module, or script)
	Bool							m_startsReady; ///< If true, the special power will be ready immediately, otherwise it will have a reload time on creation.
	Bool							m_scriptedSpecialPowerOnly;
};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class SpecialPowerModule : public BehaviorModule,
													 public SpecialPowerModuleInterface
{

	MEMORY_POOL_GLUE_ABC( SpecialPowerModule )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( SpecialPowerModule, SpecialPowerModuleData )

public:

	SpecialPowerModule( Thing* thing, const ModuleData* moduleData );

	static Int getInterfaceMask() { return MODULEINTERFACE_SPECIAL_POWER; }

	// BehaviorModule
	virtual SpecialPowerModuleInterface* getSpecialPower() override { return this; }

	virtual Bool isModuleForPower( const SpecialPowerTemplate *specialPowerTemplate ) const override;	///< is this module for the specified special power
	virtual Bool isReady() const override; 						///< is this special power available now
//  This is the althernate way to one-at-a-time BlackLotus' specials; we'll keep it commented her until Dustin decides, or until 12/10/02
//	Bool isBusy() const { return FALSE; }

	virtual Real getPercentReady() const override;		///< get the percent ready (1.0 = ready now, 0.5 = half charged up etc.)

	virtual UnsignedInt getReadyFrame() const override;		///< get the frame at which we are ready
	virtual AsciiString getPowerName() const override;
	void syncReadyFrameToStatusQuo();

	virtual const SpecialPowerTemplate* getSpecialPowerTemplate() const override;
	virtual ScienceType getRequiredScience() const override;

	virtual void onSpecialPowerCreation() override;	// called by a create module to start our countdown
	//
	// The following methods are for use by the scripting engine ONLY
	//

	virtual void setReadyFrame( UnsignedInt frame ) override;
	UnsignedInt getReadyFrame() { return m_availableOnFrame; }// USED BY PLAYER TO KEEP RECHARGE TIMERS IN SYNC
	virtual void pauseCountdown( Bool pause ) override;

	//
	// the following methods should be *EXTENDED* for any special power module implementations
	// and carry out the special power executions
	//
	virtual void doSpecialPower( UnsignedInt commandOptions ) override;
	virtual void doSpecialPowerAtObject( Object *obj, UnsignedInt commandOptions ) override;
	virtual void doSpecialPowerAtLocation( const Coord3D *loc, Real angle, UnsignedInt commandOptions ) override;
	virtual void doSpecialPowerUsingWaypoints( const Waypoint *way, UnsignedInt commandOptions ) override;

	/**
	 Now, there are special powers that require some preliminary processing before the actual
	 special power triggers. When the ini setting "UpdateModuleStartsAttack" is true, then
	 the update module will call the doSpecialPower a second time. This function then resets
	 the power recharge, and tells the scriptengine that the attack has started.

	 A good example of something that uses this is the Black Lotus - capture building hack attack.
	 When the user initiates the attack, the doSpecialPower is called, which triggers the update
	 module. The update module then orders the unit to move within range, and it isn't until the
	 hacker start the physical attack, that the timer is reset and the attack technically begins.
	*/
	virtual void markSpecialPowerTriggered( const Coord3D *location ) override;

	/** start the recharge process for this special power. public because some powers call it repeatedly.
	*/
	virtual void startPowerRecharge() override;
	virtual const AudioEventRTS& getInitiateSound() const override;

	virtual Bool startsReady() const override;
	virtual Bool isScriptOnly() const override;

	//If the special power launches a construction site, we need to know the final product for placement purposes.
	virtual const ThingTemplate* getReferenceThingTemplate() const override { return nullptr; }

protected:

	Bool initiateIntentToDoSpecialPower( const Object *targetObj, const Coord3D *targetPos, const Waypoint *way, UnsignedInt commandOptions );
	void triggerSpecialPower( const Coord3D *location );
	void createViewObject( const Coord3D *location );
	void resolveSpecialPower();
	void aboutToDoSpecialPower( const Coord3D *location );

	UnsignedInt m_availableOnFrame;			///< on this frame, this special power is available
	Int m_pausedCount;									///< Reference count of sources pausing me
	UnsignedInt m_pausedOnFrame;
	Real m_pausedPercent;

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/SpecialPower/SpecialPowerModule.cpp`
```cpp
// FILE: SpecialPowerModule.cpp ///////////////////////////////////////////////////////////////////
// Author: Colin Day, April 2002
// Desc:	 Special power module interface
///////////////////////////////////////////////////////////////////////////////////////////////////

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/GameAudio.h"
#include "Common/GameUtility.h"
#include "Common/GlobalData.h"
#include "Common/INI.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/Science.h"
#include "Common/SpecialPower.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"

#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/Module/DeletionUpdate.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Module/SpecialPowerModule.h"
#include "GameLogic/Module/SpecialPowerUpdateModule.h"
#include "GameLogic/ScriptEngine.h"

#include "GameClient/Eva.h"
#include "GameClient/InGameUI.h"
#include "GameClient/ControlBar.h"



//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
SpecialPowerModuleData::SpecialPowerModuleData()
{

	m_specialPowerTemplate = nullptr;
	m_updateModuleStartsAttack = false;
	m_startsPaused = FALSE;
	m_startsReady = FALSE;
	m_scriptedSpecialPowerOnly = FALSE;

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
/* static */ void SpecialPowerModuleData::buildFieldParse(MultiIniFieldParse& p)
{
	BehaviorModuleData::buildFieldParse( p );

	static const FieldParse dataFieldParse[] =
	{
		{ "SpecialPowerTemplate",			INI::parseSpecialPowerTemplate, nullptr, offsetof( SpecialPowerModuleData, m_specialPowerTemplate ) },
		{ "UpdateModuleStartsAttack", INI::parseBool,									nullptr, offsetof( SpecialPowerModuleData, m_updateModuleStartsAttack ) },
		{ "StartsPaused",							INI::parseBool,									nullptr, offsetof( SpecialPowerModuleData, m_startsPaused ) },
		{ "StartsReady",							INI::parseBool,									nullptr, offsetof( SpecialPowerModuleData, m_startsReady ) },
		{ "InitiateSound",						INI::parseAudioEventRTS,				nullptr, offsetof( SpecialPowerModuleData, m_initiateSound ) },
		{ "ScriptedSpecialPowerOnly", INI::parseBool,									nullptr, offsetof( SpecialPowerModuleData, m_scriptedSpecialPowerOnly ) },
		{ nullptr, nullptr, nullptr, 0 }
	};
	p.add(dataFieldParse);

}

///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
SpecialPowerModule::SpecialPowerModule( Thing *thing, const ModuleData *moduleData )
									: BehaviorModule( thing, moduleData )
{

	// TheSuperHackers @bugfix fbraz 13/07/2026: Always initialize to 0.
	// Disabling RETAIL_COMPATIBLE_CRC previously forced this to 0xFFFFFFFF, which caused unpaused
	// targetless special powers (like Spy Satellite and Battle Plans) to never become ready.
	m_availableOnFrame = 0;
	m_pausedCount = 0;
	m_pausedOnFrame = 0;
	m_pausedPercent = 0.0f;

	// we won't be able to use the power for X number of frames now

	// if we're pre-built, start counting down
	if( !getObject()->getStatusBits().test( OBJECT_STATUS_UNDER_CONSTRUCTION ) )
	{
		//A sharedNSync special only startPowerRecharges when first scienced or when executed,
		//Since a new module with same SPTemplates may construct at any time.
		if ( getSpecialPowerTemplate()->isSharedNSync() == FALSE )
		{
			startPowerRecharge();

			if (startsReady())
				m_availableOnFrame = TheGameLogic->getFrame();
		}
	}
	// WE USED TO DO THE POLL-EVERYBODY-AND-VOTE-ON-WHO-TO-SYNC-TO THING HERE,
	// BUT NO MORE, NOW IT IS HANDLED IN PLAYER

	// Some Special powers need to be activated by an Upgrade, so prevent the timer from going until then
	const SpecialPowerModuleData *md = (const SpecialPowerModuleData *)moduleData;
	if( md->m_startsPaused )
		pauseCountdown( TRUE );

	resolveSpecialPower();

	// Now, if we find that we have just come into being,
	// but there is already a science granted for our shared superweapon,
	// lets make sure TheIngameUI knows about our public timer
	// add this weapon to the UI if it has a public timer for all to see
	if( m_pausedCount == 0 &&
			getSpecialPowerTemplate()->isSharedNSync() == TRUE &&
			getSpecialPowerTemplate()->hasPublicTimer() == TRUE &&
			getObject()->getControllingPlayer() &&
			getObject()->isKindOf( KINDOF_STRUCTURE ) )
	{
		TheInGameUI->addSuperweapon( getObject()->getControllingPlayer()->getPlayerIndex(),
																 getPowerName(),
																 getObject()->getID(),
																 getSpecialPowerModuleData()->m_specialPowerTemplate );
	}


}

//-------------------------------------------------------------------------------------------------
const AudioEventRTS& SpecialPowerModule::getInitiateSound() const
{
	return getSpecialPowerModuleData()->m_initiateSound;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
SpecialPowerModule::~SpecialPowerModule()
{

 	if( getSpecialPowerModuleData()->m_specialPowerTemplate->hasPublicTimer() == TRUE &&
			getObject()->getControllingPlayer() )
 		TheInGameUI->removeSuperweapon( getObject()->getControllingPlayer()->getPlayerIndex(),
																		getPowerName(),
																		getObject()->getID(),
																		getSpecialPowerModuleData()->m_specialPowerTemplate );

}

//-------------------------------------------------------------------------------------------------
void SpecialPowerModule::setReadyFrame( UnsignedInt frame )
{
	m_availableOnFrame = frame;

	//If a script should change the ready frame, we need to update the paused frame. This value isn't
	//used directly to determine if paused or not... it uses m_pausedCount.
	m_pausedOnFrame = TheGameLogic->getFrame();
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void SpecialPowerModule::resolveSpecialPower()
{
	/*

	// if we're pre-built, and not from a command center, and a building register us with the UI
	if( getSpecialPowerModuleData()->m_specialPowerTemplate->hasPublicTimer() == TRUE &&
			TheGameLogic->getFrame() == 0 && getObject()->getControllingPlayer() &&
		  getObject()->isKindOf( KINDOF_COMMANDCENTER ) == FALSE &&
			getObject()->isKindOf( KINDOF_STRUCTURE ) )
	{
		//KM: The KINDOF_STRUCTURE check was made to prevent scripted bombers from registering their
		//    special powers as public timers.
		TheInGameUI->addSuperweapon( getObject()->getControllingPlayer()->getPlayerIndex(),
																 getPowerName(),
																 getObject()->getID(),
																 getSpecialPowerModuleData()->m_specialPowerTemplate );
	}
	*/
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void SpecialPowerModule::onSpecialPowerCreation()
{
	// THIS gets called by addScience(), that is, when the General has purchased a new special power,
	// and this module is thus activated.

	// start a power recharge going
	startPowerRecharge();

	// Dustin wants these special powers to start ready to fire,
	// so here (and only here) we will expressly set them to ready-now.
	if ( getSpecialPowerTemplate()->isSharedNSync())
	{
		Player *player = getObject()->getControllingPlayer();
		if ( player )
		{
			player->expressSpecialPowerReadyFrame( getSpecialPowerTemplate(), TheGameLogic->getFrame() );
			m_availableOnFrame = player->getOrStartSpecialPowerReadyFrame( getSpecialPowerTemplate() );
		}
	}

	// Some Special powers need to be activated by an Upgrade, so prevent the timer from going until then
	const SpecialPowerModuleData *md = getSpecialPowerModuleData();
	if( md->m_startsPaused )
		pauseCountdown( TRUE );

	// add this weapon to the UI if it has a public timer for all to see
	if( getSpecialPowerModuleData()->m_specialPowerTemplate->hasPublicTimer() == TRUE &&
			getObject()->getControllingPlayer() &&
			getObject()->isKindOf( KINDOF_STRUCTURE ) )
	{
		TheInGameUI->addSuperweapon( getObject()->getControllingPlayer()->getPlayerIndex(),
																 getPowerName(),
																 getObject()->getID(),
																 getSpecialPowerModuleData()->m_specialPowerTemplate );
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ScienceType SpecialPowerModule::getRequiredScience() const
{

	return getSpecialPowerModuleData()->m_specialPowerTemplate->getRequiredScience();
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
const SpecialPowerTemplate * SpecialPowerModule::getSpecialPowerTemplate() const
{

	return getSpecialPowerModuleData()->m_specialPowerTemplate;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
AsciiString SpecialPowerModule::getPowerName() const
{

	return getSpecialPowerModuleData()->m_specialPowerTemplate->getName();
}

//-------------------------------------------------------------------------------------------------
/** Is this module designed for the power identifier template passed in? */
//-------------------------------------------------------------------------------------------------
Bool SpecialPowerModule::isModuleForPower( const SpecialPowerTemplate *specialPowerTemplate ) const
{

	// get the module data
	const SpecialPowerModuleData *modData = getSpecialPowerModuleData();

	//
	// if the special power template defined in the module data matches the template we want
	// to check then we are for it!
	//
	if( modData->m_specialPowerTemplate == specialPowerTemplate )
	{
		//We match templates.
		return TRUE;
	}
	//We don't match templates.
	return FALSE;

}

//-------------------------------------------------------------------------------------------------
/** Is this special power ready to use */
//-------------------------------------------------------------------------------------------------
Bool SpecialPowerModule::isReady() const
{
#if defined(RTS_DEBUG) || defined(_ALLOW_DEBUG_CHEATS_IN_RELEASE)
	// this is a cheat ... remove this for release!
	if( TheGlobalData->m_specialPowerUsesDelay == FALSE )
		return TRUE;
#endif

	const Object* obj = getObject();
	const SpecialPowerModuleData *modData = getSpecialPowerModuleData();

	if ( obj && modData )
	{
		Player *player = getObject()->getControllingPlayer();
		if ( player )
		{
			if ( modData->m_specialPowerTemplate->isSharedNSync())
				return (TheGameLogic->getFrame() >= player->getOrStartSpecialPowerReadyFrame( modData->m_specialPowerTemplate ) );
		}
	}

	return (m_pausedCount == 0) && (TheGameLogic->getFrame() >= m_availableOnFrame);

}

//-------------------------------------------------------------------------------------------------
/** Get the percentage ready a special power is to use
	* 1.0f = ready now
	* 0.5f = 50% ready
	* 0.2f = 20% ready
	* etc ... */
//-------------------------------------------------------------------------------------------------
Real SpecialPowerModule::getPercentReady() const
{
	if( m_pausedCount > 0 && m_pausedPercent == 1.0f )
	{
			//Don't consider it ready if paused.
		return 0.99999f;
	}

#if defined(RTS_DEBUG) || defined(_ALLOW_DEBUG_CHEATS_IN_RELEASE)
	if( TheGlobalData->m_specialPowerUsesDelay == FALSE )
		return 1.0f;
#endif

	// easy case ... is ready
	if( isReady() )
		return 1.0f;

	if( m_pausedCount > 0 )
	{
		return m_pausedPercent;
	}

	// get the module data
	const SpecialPowerModuleData *modData = getSpecialPowerModuleData();

	// sanity
	if( modData->m_specialPowerTemplate == nullptr )
		return 0.0f;

	UnsignedInt readyFrame = m_availableOnFrame;

	//unless
	const Object* obj = getObject();
	if ( obj )
	{
		Player *player = getObject()->getControllingPlayer();
		if ( player )
		{
			if ( modData->m_specialPowerTemplate->isSharedNSync())
			{
				readyFrame = player->getOrStartSpecialPowerReadyFrame( getSpecialPowerTemplate() );
			}
		}
	}

	// calculate the percent
	Real percent = 1.0f - ((readyFrame - TheGameLogic->getFrame()) /
												 (Real)modData->m_specialPowerTemplate->getReloadTime());

	return percent;
}

Bool SpecialPowerModule::startsReady() const
{
#if RETAIL_COMPATIBLE_CRC
	return false;
#endif

	const SpecialPowerModuleData* modData = getSpecialPowerModuleData();
	return modData->m_startsReady;
}

//-------------------------------------------------------------------------------------------------
// A special power module that is only supposed to be fired via scripts. An example of this
// are the various cargo plane units we have. Scripters can launch specials from them after
// specifying a waypoint path for them to follow them.
//-------------------------------------------------------------------------------------------------
Bool SpecialPowerModule::isScriptOnly() const
{
	const SpecialPowerModuleData *modData = getSpecialPowerModuleData();
	return modData->m_scriptedSpecialPowerOnly;
}


//-------------------------------------------------------------------------------------------------
/** A special power has been used ... start the recharge process by computing the frame
	* we will become fully available on in the future again */
//-------------------------------------------------------------------------------------------------
void SpecialPowerModule::startPowerRecharge()
{
#if defined(RTS_DEBUG) || defined(_ALLOW_DEBUG_CHEATS_IN_RELEASE)
	// this is a cheat ... remove this for release!
	if( TheGlobalData->m_specialPowerUsesDelay == FALSE )
		return;
#endif

	const SpecialPowerModuleData *modData = getSpecialPowerModuleData();

	// sanity
	if( modData->m_specialPowerTemplate == nullptr )
	{
		DEBUG_CRASH(("special power not found"));
		return;
	}

	Object* obj = getObject();
	if (!obj)
		return;

	Player* player = getObject()->getControllingPlayer();
	if (!player)
		return;

	//Here, we make sure that general specials work as one between command centers
	// only factory type faction buildings should do this, and only with generals powers (in general)
	// but there are no restrictions on the use of SharedNSync on specialPowerTemplates at large
	if ( modData->m_specialPowerTemplate->isSharedNSync() )
	{
		player->resetOrStartSpecialPowerReadyFrame( modData->m_specialPowerTemplate );
	}
	else
	{
		// set the frame we will be 100% available on now
		m_availableOnFrame = TheGameLogic->getFrame() + getSpecialPowerTemplate()->getReloadTime();
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Bool SpecialPowerModule::initiateIntentToDoSpecialPower( const Object *targetObj, const Coord3D *targetPos, const Waypoint *way, UnsignedInt commandOptions )
{
	Bool valid = false;
	// tell our update modules that we intend to do this special power.
	for( BehaviorModule** u = getObject()->getBehaviorModules(); *u; ++u )
	{
		SpecialPowerUpdateInterface* spu = (*u)->getSpecialPowerUpdateInterface();
		if( spu )
		{
			//Validate that we are calling the correct module!
			if( isModuleForPower( getSpecialPowerModuleData()->m_specialPowerTemplate ) )
			{
				if( spu->doesSpecialPowerUpdatePassScienceTest() )
				{
					if( spu->initiateIntentToDoSpecialPower( getSpecialPowerModuleData()->m_specialPowerTemplate, targetObj, targetPos, way, commandOptions ) )
					{
						//Kris: Aug 2003
						//We have executed the special power, so don't try to execute any more. This logic
						//was changed for multi-level spectres. Before, multiple modules would get launched
						//causing 2 or 3 spectres to be created.
						valid = true;
						break;
					}
				}
			}
		}
	}

	// GeneralsX @bugfix Copilot 06/09/2026 Stop a rejected missile-launcher intent before recording or triggering it.
	// Upstream reference: Mauller, PR #1218
	// https://github.com/TheSuperHackers/GeneralsGameCode/pull/1218
	if (m_availableOnFrame == 0xFFFFFFFF)
	{
		DEBUG_ASSERTCRASH(!valid, ("Using MissileLauncherBuildingUpdate escape path when valid is set to true"));
		return false;
	}

	getObject()->getControllingPlayer()->getAcademyStats()->recordSpecialPowerUsed( getSpecialPowerModuleData()->m_specialPowerTemplate );

	//If we depend on our update module to trigger the special power, make sure we have the
	//appropriate update module!
	if( !valid && getSpecialPowerModuleData()->m_updateModuleStartsAttack )
	{
		DEBUG_CRASH( ("Object does not contain a special power module to execute.  Did you forget to add it to the object INI?"));
		//DEBUG_CRASH(( "Object does not contain special power module (%s) to execute.  Did you forget to add it to the object INI?",
		//							command->m_specialPower->getName().str() ));
	}

	return valid;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void SpecialPowerModule::triggerSpecialPower( const Coord3D *location )
{
	aboutToDoSpecialPower( location );	// do BEFORE recharge

	createViewObject(location);

	// we won't be able to use the power for X number of frames now
	startPowerRecharge();
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void SpecialPowerModule::createViewObject( const Coord3D *location )
{
	const SpecialPowerModuleData *modData = getSpecialPowerModuleData();
	const SpecialPowerTemplate *powerTemplate = modData->m_specialPowerTemplate;

	if( modData == nullptr  ||  powerTemplate == nullptr )
		return;

	Real visionRange = powerTemplate->getViewObjectRange();
	UnsignedInt visionDuration = powerTemplate->getViewObjectDuration();

	if( visionRange == 0 || visionDuration == 0 )
		return; // We don't want a view object at all.

	AsciiString objectName = TheGlobalData->m_specialPowerViewObjectName;
	if( objectName.isEmpty() )
		return;

	const ThingTemplate *viewObjectTemplate = TheThingFactory->findTemplate( objectName );
	if( viewObjectTemplate == nullptr )
		return;

	Object *viewObject = TheThingFactory->newObject( viewObjectTemplate, getObject()->getControllingPlayer()->getDefaultTeam() );

	if( viewObject == nullptr )
		return;

	viewObject->setPosition( location );
	viewObject->setShroudClearingRange( visionRange );

	static NameKeyType key_DeletionUpdate = NAMEKEY("DeletionUpdate");
	DeletionUpdate* dup = (DeletionUpdate*)viewObject->findUpdateModule(key_DeletionUpdate);
	if( dup )
	{
		dup->setLifetimeRange( visionDuration, visionDuration );
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void SpecialPowerModule::markSpecialPowerTriggered( const Coord3D *location )
{
	triggerSpecialPower( location );
}


//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void SpecialPowerModule::aboutToDoSpecialPower( const Coord3D *location )
{
	// Tell the scripting engine!
	TheScriptEngine->notifyOfTriggeredSpecialPower(
		getObject()->getControllingPlayer()->getPlayerIndex(),
		getSpecialPowerModuleData()->m_specialPowerTemplate->getName(),
		getObject()->getID());

	// Let EVA do her thing
	SpecialPowerType type = getSpecialPowerModuleData()->m_specialPowerTemplate->getSpecialPowerType();

	Player *localPlayer = rts::getObservedOrLocalPlayer();
	Relationship relationship = localPlayer->getRelationship(getObject()->getTeam());

  // Only play the EVA sounds if this is not the local player, and the local player doesn't consider the
	// person an enemy.
	// Kris: Actually, all players need to hear these warnings.
  // Ian: But now there are different Eva messages depending on who launched
	//if (localPlayer != getObject()->getControllingPlayer() && localPlayer->getRelationship(getObject()->getTeam()) != ENEMIES)
  {
		if( type == SPECIAL_PARTICLE_UPLINK_CANNON || type == SUPW_SPECIAL_PARTICLE_UPLINK_CANNON || type == LAZR_SPECIAL_PARTICLE_UPLINK_CANNON )
    {
      if ( localPlayer == getObject()->getControllingPlayer() )
      {
        TheEva->setShouldPlay(EVA_SuperweaponLaunched_Own_ParticleCannon);
      }
      else if (relationship != ENEMIES)
      {
        // Note: counting relationship NEUTRAL as ally. Not sure if this makes a difference???
        TheEva->setShouldPlay(EVA_SuperweaponLaunched_Ally_ParticleCannon);
      }
      else
      {
        TheEva->setShouldPlay(EVA_SuperweaponLaunched_Enemy_ParticleCannon);
      }
    }
    else if( type == SPECIAL_NEUTRON_MISSILE || type == NUKE_SPECIAL_NEUTRON_MISSILE || type == SUPW_SPECIAL_NEUTRON_MISSILE )
    {
      if ( localPlayer == getObject()->getControllingPlayer() )
      {
        TheEva->setShouldPlay(EVA_SuperweaponLaunched_Own_Nuke);
      }
      else if (relationship != ENEMIES)
      {
        // Note: counting relationship NEUTRAL as ally. Not sure if this makes a difference???
        TheEva->setShouldPlay(EVA_SuperweaponLaunched_Ally_Nuke);
      }
      else
      {
        TheEva->setShouldPlay(EVA_SuperweaponLaunched_Enemy_Nuke);
      }
    }
		else if (type == SPECIAL_SCUD_STORM)
    {
      if ( localPlayer == getObject()->getControllingPlayer() )
      {
        TheEva->setShouldPlay(EVA_SuperweaponLaunched_Own_ScudStorm);
      }
      else if (relationship != ENEMIES)
      {
        // Note: counting relationship NEUTRAL as ally. Not sure if this makes a difference???
        TheEva->setShouldPlay(EVA_SuperweaponLaunched_Ally_ScudStorm);
      }
      else
      {
        TheEva->setShouldPlay(EVA_SuperweaponLaunched_Enemy_ScudStorm);
      }
    }
		else if (type == SPECIAL_GPS_SCRAMBLER || type == SLTH_SPECIAL_GPS_SCRAMBLER )
    {
			// This is Ghetto.  Voices should be ini lines in the special power entry.  You shouldn't have to
			// add to an enum to get a new voice
      if ( localPlayer == getObject()->getControllingPlayer() )
      {
        TheEva->setShouldPlay(EVA_SuperweaponLaunched_Own_GPS_Scrambler);
      }
      else if (relationship != ENEMIES)
      {
        // Note: counting relationship NEUTRAL as ally. Not sure if this makes a difference???
        TheEva->setShouldPlay(EVA_SuperweaponLaunched_Ally_GPS_Scrambler);
      }
      else
      {
        TheEva->setShouldPlay(EVA_SuperweaponLaunched_Enemy_GPS_Scrambler);
      }
    }
		else if (type == SPECIAL_SNEAK_ATTACK)
    {
      if ( localPlayer == getObject()->getControllingPlayer() )
      {
        TheEva->setShouldPlay(EVA_SuperweaponLaunched_Own_Sneak_Attack);
      }
      else if (relationship != ENEMIES)
      {
        // Note: counting relationship NEUTRAL as ally. Not sure if this makes a difference???
        TheEva->setShouldPlay(EVA_SuperweaponLaunched_Ally_Sneak_Attack);
      }
      else
      {
        TheEva->setShouldPlay(EVA_SuperweaponLaunched_Enemy_Sneak_Attack);
      }
    }
	}

	// get module data
	const SpecialPowerModuleData *modData = getSpecialPowerModuleData();

	// play our initiate sound if we have one
	AudioEventRTS audioEvent = *modData->m_specialPowerTemplate->getInitiateSound();
	audioEvent.setObjectID(getObject()->getID());
	TheAudio->addAudioEvent( &audioEvent );

	// play sound at target location if specified
	if( location )
	{

		AudioEventRTS soundAtLocation = *modData->m_specialPowerTemplate->getInitiateAtTargetSound();
		soundAtLocation.setPosition( location );
		soundAtLocation.setPlayerIndex(getObject()->getControllingPlayer()->getPlayerIndex());
		TheAudio->addAudioEvent( &soundAtLocation );

	}

}

//-------------------------------------------------------------------------------------------------
//By default, special powers are not triggered by it's update module -- in which case
//it triggers it and resets its timer immediately. When the update module triggers it,
//then all we do is initiate the special power, and trust that the update module will
//do the rest.
//-------------------------------------------------------------------------------------------------
void SpecialPowerModule::doSpecialPower( UnsignedInt commandOptions )
{
	if (m_pausedCount > 0 || getObject()->isDisabled()) {
		return;
	}

	//This tells the update module that we want to do our special power. The update modules
	//will then start processing each frame.
	initiateIntentToDoSpecialPower( nullptr, nullptr, nullptr, commandOptions );

	//Only trigger the special power immediately if the updatemodule doesn't start the attack.
	//An example of a case that wouldn't trigger immediately is for a unit that needs to
	//close to range before firing the special attack. A case that would trigger immediately
	//is the napalm strike. If we don't call this now, it's up to the update module to do so.
	if( !getSpecialPowerModuleData()->m_updateModuleStartsAttack )
	{
		triggerSpecialPower( nullptr );// Location-less trigger
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void SpecialPowerModule::doSpecialPowerAtObject( Object *obj, UnsignedInt commandOptions )
{
	if (m_pausedCount > 0 || getObject()->isDisabled()) {
		return;
	}

	//This tells the update module that we want to do our special power. The update modules
	//will then start processing each frame.
	initiateIntentToDoSpecialPower( obj, nullptr, nullptr, commandOptions );

	//Only trigger the special power immediately if the updatemodule doesn't start the attack.
	//An example of a case that wouldn't trigger immediately is for a unit that needs to
	//close to range before firing the special attack. A case that would trigger immediately
	//is the napalm strike. If we don't call this now, it's up to the update module to do so.
	if( !getSpecialPowerModuleData()->m_updateModuleStartsAttack )
	{
		triggerSpecialPower( obj->getPosition() );
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void SpecialPowerModule::doSpecialPowerAtLocation( const Coord3D *loc, Real angle, UnsignedInt commandOptions )
{
	if (m_pausedCount > 0 || getObject()->isDisabled()) {
		return;
	}

	//This tells the update module that we want to do our special power. The update modules
	//will then start processing each frame.
	initiateIntentToDoSpecialPower( nullptr, loc, nullptr, commandOptions );

	// GeneralsX @bugfix Copilot 06/09/2026 Do not trigger a location power after its update module rejected the intent.
	if (m_availableOnFrame == 0xFFFFFFFF)
		return;

	//Only trigger the special power immediately if the updatemodule doesn't start the attack.
	//An example of a case that wouldn't trigger immediately is for a unit that needs to
	//close to range before firing the special attack. A case that would trigger immediately
	//is the napalm strike. If we don't call this now, it's up to the update module to do so.
	if( !getSpecialPowerModuleData()->m_updateModuleStartsAttack )
	{
		triggerSpecialPower( loc );
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void SpecialPowerModule::doSpecialPowerUsingWaypoints( const Waypoint *way, UnsignedInt commandOptions )
{
	if (m_pausedCount > 0 || getObject()->isDisabled()) {
		return;
	}

	//This tells the update module that we want to do our special power. The update modules
	//will then start processing each frame.
	initiateIntentToDoSpecialPower( nullptr, nullptr, way, commandOptions );

	//Only trigger the special power immediately if the updatemodule doesn't start the attack.
	//An example of a case that wouldn't trigger immediately is for a unit that needs to
	//close to range before firing the special attack. A case that would trigger immediately
	//is the napalm strike. If we don't call this now, it's up to the update module to do so.
	if( !getSpecialPowerModuleData()->m_updateModuleStartsAttack )
	{
		triggerSpecialPower( nullptr );// This type doesn't create view objects
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void SpecialPowerModule::pauseCountdown( Bool pause )
{
	if (pause)// If pausing
	{
		if( m_pausedCount == 0 )
		{
			// Only record this with the first pausing, otherwise upon final unpausing you would get credited the time
			// between pauses as time served.
			m_pausedOnFrame = TheGameLogic->getFrame();
			m_pausedPercent = getPercentReady();
		}

		++m_pausedCount;
	}
	else if( m_pausedCount > 0 )//Else if unpausing, but only if I am in fact paused, so multiple unpauses don't break.
	{
		--m_pausedCount;

		// And only update the ready time if we are fully unpaused now.
		if( m_pausedCount == 0 )
		{
			m_availableOnFrame += (TheGameLogic->getFrame() - m_pausedOnFrame);
		}
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
UnsignedInt SpecialPowerModule::getReadyFrame() const
{
	if ( getSpecialPowerTemplate()->isSharedNSync() )
	{
		const Object* obj = getObject();
		if ( obj )
		{
			Player *player = getObject()->getControllingPlayer();
			if ( player )
				return player->getOrStartSpecialPowerReadyFrame( getSpecialPowerTemplate() );
		}
	}

	if (m_pausedCount > 0 || getObject()->isDisabled())
	{
		Int pausedFrames = TheGameLogic->getFrame() - m_pausedOnFrame;
		return m_availableOnFrame + pausedFrames;
	}
	else
	{
		return m_availableOnFrame;
	}
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void SpecialPowerModule::crc( Xfer *xfer )
{

	// extend base class
	BehaviorModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void SpecialPowerModule::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	BehaviorModule::xfer( xfer );

	// available on frame
	xfer->xferUnsignedInt( &m_availableOnFrame );

	// paused by script
	xfer->xferInt( &m_pausedCount );

	// paused on frame
	xfer->xferUnsignedInt( &m_pausedOnFrame );

	// paused percent
	xfer->xferReal( &m_pausedPercent );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void SpecialPowerModule::loadPostProcess()
{

	// extend base class
	BehaviorModule::loadPostProcess();



	// Now, if we find that we have just come into being,
	// but there is already a science granted for our shared superweapon,
	// lets make sure TheIngameUI knows about our public timer
	// add this weapon to the UI if it has a public timer for all to see
	if( m_pausedCount == 0 &&
			getSpecialPowerTemplate()->isSharedNSync() == TRUE &&
			getSpecialPowerTemplate()->hasPublicTimer() == TRUE &&
			getObject()->getControllingPlayer() &&
			getObject()->isKindOf( KINDOF_STRUCTURE ) )
	{
		TheInGameUI->addSuperweapon( getObject()->getControllingPlayer()->getPlayerIndex(),
																 getPowerName(),
																 getObject()->getID(),
																 getSpecialPowerModuleData()->m_specialPowerTemplate );
	}






}
```
