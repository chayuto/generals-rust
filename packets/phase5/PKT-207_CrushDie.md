# Translation Packet `PKT-207`: `CrushDie`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-207`
- **Work Package:** `WP-5.2` (Phase 5)
- **Complexity & Assigned Tier:** Class `C1` -> `tier:1a-lite`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/die/crush_die.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (15):**
  - `Common/AudioEventRTS.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/AudioEventRTS.h`
  - `Common/INI.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/INI.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/PlayerList.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/PlayerList.h`
  - `Common/RandomValue.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/RandomValue.h`
  - `Common/ThingFactory.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ThingFactory.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/CrushDie.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CrushDie.h`
  - `GameLogic/Module/DieModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DieModule.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (6):**
  - `"TotalCrushSound" => INI::parseAudioEventRTS`
  - `"BackEndCrushSound" => INI::parseAudioEventRTS`
  - `"FrontEndCrushSound" => INI::parseAudioEventRTS`
  - `"TotalCrushSoundPercent" => INI::parseInt`
  - `"BackEndCrushSoundPercent" => INI::parseInt`
  - `"FrontEndCrushSoundPercent" => INI::parseInt`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (1 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
- **PRNG Calls (1 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValue(0, 99) < getCrushDieModuleData()->m_crushSoundPercent[crushType]) { AudioEventRTS crushSound(getCrushDieModuleData()->m_crushSounds[crushType])`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/CrushDie.h`
```cpp
// FILE: CrushDie.h /////////////////////////////////////////////////////////////////////////////
// Author: Colin Day, November 2001
// Desc:   Default die module
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/AudioEventRTS.h"
#include "Common/INI.h"

#include "GameLogic/Module/DieModule.h"


// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
class Thing;

enum CrushEnum CPP_11(: Int)
{
	TOTAL_CRUSH,
	BACK_END_CRUSH,
	FRONT_END_CRUSH,
	NO_CRUSH,

	CRUSH_COUNT
};

//-------------------------------------------------------------------------------------------------
class CrushDieModuleData : public DieModuleData
{
public:
	AudioEventRTS		m_crushSounds[CRUSH_COUNT];
	Int							m_crushSoundPercent[CRUSH_COUNT];

	CrushDieModuleData()
	{
		for (int i = 0; i < CRUSH_COUNT; i++)
		{
			m_crushSoundPercent[i] = 100;
		}
	}

	static void buildFieldParse(MultiIniFieldParse& p)
	{
    DieModuleData::buildFieldParse(p);

		static const FieldParse dataFieldParse[] =
		{
			{ "TotalCrushSound",					INI::parseAudioEventRTS,			nullptr, offsetof( CrushDieModuleData, m_crushSounds[TOTAL_CRUSH] ) },
			{ "BackEndCrushSound",				INI::parseAudioEventRTS,			nullptr, offsetof( CrushDieModuleData, m_crushSounds[BACK_END_CRUSH] ) },
			{ "FrontEndCrushSound",				INI::parseAudioEventRTS,			nullptr, offsetof( CrushDieModuleData, m_crushSounds[FRONT_END_CRUSH] ) },
			{ "TotalCrushSoundPercent",		INI::parseInt,						nullptr, offsetof( CrushDieModuleData, m_crushSoundPercent[TOTAL_CRUSH] ) },
			{ "BackEndCrushSoundPercent",	INI::parseInt,						nullptr, offsetof( CrushDieModuleData, m_crushSoundPercent[BACK_END_CRUSH] ) },
			{ "FrontEndCrushSoundPercent",INI::parseInt,						nullptr, offsetof( CrushDieModuleData, m_crushSoundPercent[FRONT_END_CRUSH] ) },
			{ nullptr, nullptr, nullptr, 0 }
		};
    p.add(dataFieldParse);
	}
};

//-------------------------------------------------------------------------------------------------
class CrushDie : public DieModule
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( CrushDie, "CrushDie" )

	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( CrushDie, CrushDieModuleData );

public:

	CrushDie( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	virtual void onDie( const DamageInfo *damageInfo ) override;

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Die/CrushDie.cpp`
```cpp
// FILE: CrushDie.cpp ///////////////////////////////////////////////////////////////////////////
// Author: Colin Day, November 2001
// Desc:   Crush Die module
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/RandomValue.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"
#include "GameClient/Drawable.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/CrushDie.h"

//-------------------------------------------------------------------------------------------------
// Figure out which crush point was hit so the correct crushed object can be swapped in

// Figure out which crush point was hit so the correct crushed object can be swapped in
static CrushEnum crushLocationCheck( Object* crusherObject, Object* victimObject )
{
	if( (crusherObject == nullptr)  ||  (victimObject == nullptr) )
		return NO_CRUSH;

	Bool frontCrushed = victimObject->getBodyModule()->getFrontCrushed();
	Bool backCrushed = victimObject->getBodyModule()->getBackCrushed();

//	const Coord3D *dir = crusherObject->getUnitDirectionVector2D();
	const Coord3D *otherDir = victimObject->getUnitDirectionVector2D();
	const Coord3D *pos = crusherObject->getPosition();
	const Coord3D *otherPos = victimObject->getPosition();

	Real crushPointOffsetDistance = victimObject->getGeometryInfo().getMajorRadius() * 0.5;

	Coord3D crushPointOffset;
	crushPointOffset.x = otherDir->x * crushPointOffsetDistance;
	crushPointOffset.y = otherDir->y * crushPointOffsetDistance;
	crushPointOffset.z = 0;

	Coord3D comparisonCoord;
	Real dx, dy;

	CrushEnum retval = NO_CRUSH;
	Real bestDist = 99999;

	// PhysicsCollide has already done the logic of which point to smoosh and waited until we crossed that point
	// so at this point we just need to know which crush point is closest.

	if( !frontCrushed && !backCrushed )
	{
		// Check the middle crush point
		comparisonCoord = *otherPos;//copy so can move to each crush point

		dx = comparisonCoord.x - pos->x;
		dy = comparisonCoord.y - pos->y;
		Real dist = (Real)( dx*dx + dy*dy );

		//otherwise we want to make sure we get the closest valid crush point
		retval = TOTAL_CRUSH;
		bestDist = dist;
	}

	if( !frontCrushed )
	{
		// Check the front point.
		comparisonCoord = *otherPos;
		comparisonCoord.x += crushPointOffset.x;
		comparisonCoord.y += crushPointOffset.y;

		dx = comparisonCoord.x - pos->x;
		dy = comparisonCoord.y - pos->y;
		Real dist = (Real)( dx*dx + dy*dy );

		if( dist < bestDist )//closer
		{
			if( backCrushed )
			{
				retval = TOTAL_CRUSH;
				bestDist = dist;
			}
			else
			{
				retval = FRONT_END_CRUSH;
				bestDist = dist;
			}
		}
	}

	if( !backCrushed )
	{
		// Check back point
		comparisonCoord = *otherPos;
		comparisonCoord.x -= crushPointOffset.x;
		comparisonCoord.y -= crushPointOffset.y;

		dx = comparisonCoord.x - pos->x;
		dy = comparisonCoord.y - pos->y;
		Real dist = (Real)( dx*dx + dy*dy );

		if( dist < bestDist )//closer
		{
			if( frontCrushed )
			{
				retval = TOTAL_CRUSH;
				bestDist = dist;
			}
			else
			{
				retval = BACK_END_CRUSH;
				bestDist = dist;
			}
		}
	}

	return retval;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
CrushDie::CrushDie( Thing *thing, const ModuleData* moduleData ) : DieModule( thing, moduleData )
{
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
CrushDie::~CrushDie()
{

}

//-------------------------------------------------------------------------------------------------
/** The die callback. */
//-------------------------------------------------------------------------------------------------
void CrushDie::onDie( const DamageInfo * damageInfo )
{
	if (!isDieApplicable(damageInfo))
		return;

	DEBUG_ASSERTCRASH(damageInfo->in.m_damageType == DAMAGE_CRUSH, ("this should only be used for crush damage"));
	if (damageInfo->in.m_damageType != DAMAGE_CRUSH)
		return;

	Object *damageDealer = TheGameLogic->findObjectByID( damageInfo->in.m_sourceID );
	DEBUG_ASSERTCRASH(damageDealer,("You must have a damageDealer source for this effect"));

	CrushEnum crushType = damageDealer ? crushLocationCheck(damageDealer, getObject()) : TOTAL_CRUSH;

	if (crushType != NO_CRUSH)
	{
		if (getCrushDieModuleData()->m_crushSounds[crushType].getEventName().isEmpty() == false)
		{
			// be sure that 0==never, 100==always
			// MDC: moving to GameLogicRandomValue.  This does not need to be synced, but having it so makes searches *so* much nicer.
			if (GameLogicRandomValue(0, 99) < getCrushDieModuleData()->m_crushSoundPercent[crushType])
			{
				AudioEventRTS crushSound(getCrushDieModuleData()->m_crushSounds[crushType]);
				crushSound.setObjectID(getObject()->getID());
				TheAudio->addAudioEvent(&crushSound);
			}
		}
		{
			Object *me = getObject();

			if (me)
			{
				me->getBodyModule()->setFrontCrushed(crushType == TOTAL_CRUSH || crushType == FRONT_END_CRUSH);
				me->getBodyModule()->setBackCrushed(crushType == TOTAL_CRUSH || crushType == BACK_END_CRUSH);

				ModelConditionFlags newCrushed;
				newCrushed.set(MODELCONDITION_FRONTCRUSHED, (crushType == TOTAL_CRUSH || crushType == FRONT_END_CRUSH));
				newCrushed.set(MODELCONDITION_BACKCRUSHED, crushType == TOTAL_CRUSH || crushType == BACK_END_CRUSH);

				me->getDrawable()->clearAndSetModelConditionFlags(
					MAKE_MODELCONDITION_MASK2(MODELCONDITION_BACKCRUSHED, MODELCONDITION_FRONTCRUSHED),
					newCrushed);
			}
		}
	}
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void CrushDie::crc( Xfer *xfer )
{

	// extend base class
	DieModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void CrushDie::xfer( Xfer *xfer )
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
void CrushDie::loadPostProcess()
{

	// extend base class
	DieModule::loadPostProcess();

}
```
