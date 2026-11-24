# Translation Packet `PKT-285`: `ParachuteContain`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-285`
- **Work Package:** `WP-5.3` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/contain/parachute_contain.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (15):**
  - `Common/CRCDebug.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/CRCDebug.h`
  - `Common/Player.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Player.h`
  - `Common/RandomValue.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/RandomValue.h`
  - `Common/ThingTemplate.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/ThingTemplate.h`
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameLogic/AIPathfind.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameLogic/AIPathfind.h`
  - `GameLogic/Locomotor.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Locomotor.h`
  - `GameLogic/Module/AIUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/OpenContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/OpenContain.h`
  - `GameLogic/Module/ParachuteContain.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ParachuteContain.h`
  - `GameLogic/Module/PhysicsUpdate.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/PhysicsUpdate.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/PartitionManager.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/PartitionManager.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (7):**
  - `"PitchRateMax" => INI::parseAngularVelocityReal`
  - `"RollRateMax" => INI::parseAngularVelocityReal`
  - `"LowAltitudeDamping" => INI::parseReal`
  - `"ParachuteOpenDist" => INI::parseReal`
  - `"KillWhenLandingInWaterSlop" => INI::parseReal`
  - `"FreeFallDamagePercent" => INI::parsePercentToReal`
  - `"ParachuteOpenSound" => INI::parseAudioEventRTS`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (19 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferReal( &m_pitch )`
  3. `xferReal( &m_roll )`
  4. `xferReal( &m_pitchRate )`
  5. `xferReal( &m_rollRate )`
  6. `xferReal( &m_startZ )`
  7. `xferBool( &m_isLandingOverrideSet )`
  8. `xferCoord3D( &m_landingOverride )`
  9. `xferCoord3D( &m_riderAttachBone )`
  10. `xferCoord3D( &m_riderSwayBone )`
  11. `xferCoord3D( &m_paraAttachBone )`
  12. `xferCoord3D( &m_paraSwayBone )`
  13. `xferCoord3D( &m_riderAttachOffset )`
  14. `xferCoord3D( &m_riderSwayOffset )`
  15. `xferCoord3D( &m_paraAttachOffset )`
  16. `xferCoord3D( &m_paraSwayOffset )`
  17. `xferBool( &m_needToUpdateRiderBones )`
  18. `xferBool( &m_needToUpdateParaBones )`
  19. `xferBool( &m_opened )`
- **PRNG Calls (2 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValueReal(-d->m_pitchRateMax, d->m_pitchRateMax)`
  - `GameLogicRandomValueReal(-d->m_rollRateMax, d->m_rollRateMax)`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/ParachuteContain.h`
```cpp
// FILE: ParachuteContain.h ////////////////////////////////////////////////////////////////////////
// Author: Steven Johnson, March 2002
// Desc:   Contain module for transport units.
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "GameLogic/Module/OpenContain.h"

//-------------------------------------------------------------------------------------------------
class ParachuteContainModuleData : public OpenContainModuleData
{
public:
	Real m_pitchRateMax;
	Real m_rollRateMax;
	Real m_lowAltitudeDamping;
	Real m_paraOpenDist;		///< deploy the parachute when we have traveled this far
	Real m_freeFallDamagePercent;
	Real m_killWhenLandingInWaterSlop;
	AudioEventRTS m_parachuteOpenSound;

	ParachuteContainModuleData();
	static void buildFieldParse(MultiIniFieldParse& p);
};

//-------------------------------------------------------------------------------------------------
class ParachuteContain : public OpenContain
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( ParachuteContain, "ParachuteContain" )
	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( ParachuteContain, ParachuteContainModuleData )

public:

	ParachuteContain( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	virtual void onDrawableBoundToObject() override;

	virtual Bool isValidContainerFor( const Object* obj, bool checkCapacity) const override;
	virtual Bool isEnclosingContainerFor( const Object *obj ) const override { return FALSE; }	///< Does this type of Contain Visibly enclose its contents?
	virtual Bool isSpecialZeroSlotContainer() const override { return true; }

	virtual void onContaining( Object *obj, Bool wasSelected ) override;		///< object now contains 'obj'
	virtual void onRemoving( Object *obj ) override;			///< object no longer contains 'obj'

	virtual UpdateSleepTime update() override;							///< called once per frame

	virtual void containReactToTransformChange() override;

	virtual void onCollide( Object *other, const Coord3D *loc, const Coord3D *normal ) override;
	virtual void onDie( const DamageInfo * damageInfo ) override;

	virtual void setOverrideDestination( const Coord3D *dest ) override; ///< Instead of falling peacefully towards a clear spot, I will now aim here

protected:

	virtual Bool isFullyEnclosingContainer() const { return false; }
	virtual void positionContainedObjectsRelativeToContainer() override;

private:

	void positionRider(Object* obj);
	void updateBonePositions();
	void updateOffsetsFromBones();
	void calcSwayMtx(const Coord3D* offset, Matrix3D* mtx);
	void setSwayMatrices();

	Real m_pitch;
	Real m_roll;
	Real m_pitchRate;
	Real m_rollRate;
	Real m_startZ;
	Bool m_isLandingOverrideSet;
	Coord3D m_landingOverride;
	Coord3D m_riderAttachBone;
	Coord3D m_riderSwayBone;
	Coord3D m_paraAttachBone;
	Coord3D m_paraSwayBone;
	Coord3D m_riderAttachOffset;
	Coord3D m_riderSwayOffset;
	Coord3D m_paraAttachOffset;
	Coord3D m_paraSwayOffset;
	Bool m_needToUpdateRiderBones;
	Bool m_needToUpdateParaBones;
	Bool m_opened;
};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Contain/ParachuteContain.cpp`
```cpp
// FILE: ParachuteContain.cpp //////////////////////////////////////////////////////////////////////
// Author: Steven Johnson, March 2002
// Desc:   Contain module for transport units.
///////////////////////////////////////////////////////////////////////////////////////////////////

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/CRCDebug.h"
#include "Common/Player.h"
#include "Common/RandomValue.h"
#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"

#include "GameLogic/AIPathfind.h"
#include "GameLogic/Locomotor.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/ParachuteContain.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"

#include "GameClient/Drawable.h"


const Real NO_START_Z = 1e10;




// PRIVATE ////////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
ParachuteContainModuleData::ParachuteContainModuleData() :
	m_pitchRateMax(0),
	m_rollRateMax(0),
	m_lowAltitudeDamping(0.2f),
	m_paraOpenDist(0.0f),
	m_freeFallDamagePercent(0.5f),
	m_killWhenLandingInWaterSlop(10.0f)
{
}

//-------------------------------------------------------------------------------------------------
void ParachuteContainModuleData::buildFieldParse(MultiIniFieldParse& p)
{
  OpenContainModuleData::buildFieldParse(p);

	static const FieldParse dataFieldParse[] =
	{
		{ "PitchRateMax",	INI::parseAngularVelocityReal,		nullptr, offsetof( ParachuteContainModuleData, m_pitchRateMax ) },
		{ "RollRateMax",	INI::parseAngularVelocityReal,		nullptr, offsetof( ParachuteContainModuleData, m_rollRateMax ) },
		{ "LowAltitudeDamping",	INI::parseReal,		nullptr, offsetof( ParachuteContainModuleData, m_lowAltitudeDamping ) },
		{ "ParachuteOpenDist",	INI::parseReal,		nullptr, offsetof( ParachuteContainModuleData, m_paraOpenDist ) },
		{ "KillWhenLandingInWaterSlop",	INI::parseReal,		nullptr, offsetof( ParachuteContainModuleData, m_killWhenLandingInWaterSlop ) },
		{ "FreeFallDamagePercent",	INI::parsePercentToReal,		nullptr, offsetof( ParachuteContainModuleData, m_freeFallDamagePercent ) },
		{ "ParachuteOpenSound", INI::parseAudioEventRTS, nullptr, offsetof( ParachuteContainModuleData, m_parachuteOpenSound ) },
		{ nullptr, nullptr, nullptr, 0 }
	};
  p.add(dataFieldParse);
}

// PUBLIC /////////////////////////////////////////////////////////////////////////////////////////
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ParachuteContain::ParachuteContain( Thing *thing, const ModuleData *moduleData ) :
								 OpenContain( thing, moduleData )
{
	m_opened = false;
	m_needToUpdateParaBones = true;
	m_needToUpdateRiderBones = true;
	m_pitch = 0;
	m_roll = 0;
	m_pitchRate = 0;
	m_rollRate = 0;
	m_isLandingOverrideSet = FALSE;
	m_startZ = NO_START_Z;

	m_landingOverride.zero();
	m_paraAttachBone.zero();
	m_paraAttachOffset.zero();
	m_paraSwayBone.zero();
	m_paraSwayOffset.zero();
	m_riderAttachBone.zero();
	m_riderAttachOffset.zero();
	m_riderSwayBone.zero();
	m_riderSwayOffset.zero();

	const ParachuteContainModuleData* d = getParachuteContainModuleData();
	if (d)
	{
		m_pitchRate = GameLogicRandomValueReal(-d->m_pitchRateMax, d->m_pitchRateMax);
		m_rollRate = GameLogicRandomValueReal(-d->m_rollRateMax, d->m_rollRateMax);
	}

	getObject()->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_PARACHUTING ) );
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ParachuteContain::~ParachuteContain()
{
}

//-------------------------------------------------------------------------------------------------
/**
	this is called whenever a drawable is bound to the object.
	drawable is NOT guaranteed to be non-null.
*/
void ParachuteContain::onDrawableBoundToObject()
{
	Drawable* draw = getObject()->getDrawable();
	if (draw)
		draw->setDrawableHidden(!m_opened);
}

//-------------------------------------------------------------------------------------------------
void ParachuteContain::calcSwayMtx(const Coord3D* offset, Matrix3D* mtx)
{
	mtx->Make_Identity();
	mtx->Translate(offset->x, offset->y, offset->z);
	mtx->In_Place_Pre_Rotate_X(m_roll);
	mtx->In_Place_Pre_Rotate_Y(m_pitch);
	mtx->Translate(-offset->x, -offset->y, -offset->z);
}

//-------------------------------------------------------------------------------------------------
void ParachuteContain::updateBonePositions()
{
	if (m_needToUpdateParaBones)
	{
		m_needToUpdateParaBones = false;	// yeah, even if not found.

		Drawable* parachuteDraw = getObject()->getDrawable();
		if (parachuteDraw)
		{
			if (parachuteDraw->getPristineBonePositions( "PARA_COG", 0, &m_paraSwayBone, nullptr, 1) != 1)
			{
				DEBUG_CRASH(("PARA_COG not found"));
				m_paraSwayBone.zero();
			}

			if (parachuteDraw->getPristineBonePositions( "PARA_ATTCH", 0, &m_paraAttachBone, nullptr, 1 ) != 1)
			{
				DEBUG_CRASH(("PARA_ATTCH not found"));
				m_paraAttachBone.zero();
			}
		}
		//DEBUG_LOG(("updating para bone positions %d...",TheGameLogic->getFrame()));
	}

	if (m_needToUpdateRiderBones)
	{

		m_needToUpdateRiderBones = false;	// yeah, even if not found.

		Object* rider = (getContainCount() > 0) ? getContainList().front() : nullptr;
		Drawable* riderDraw = rider ? rider->getDrawable() : nullptr;
		if (riderDraw)
		{
			if (riderDraw->getPristineBonePositions( "PARA_MAN", 0, &m_riderAttachBone, nullptr, 1) != 1)
			{
				//DEBUG_LOG(("*** No parachute-attach bone... using object height!"));
				m_riderAttachBone.zero();
				m_riderAttachBone.z += riderDraw->getDrawableGeometryInfo().getMaxHeightAbovePosition();
			}
		}

		//DEBUG_LOG(("updating rider bone positions %d...",TheGameLogic->getFrame()));
	}
}

//-------------------------------------------------------------------------------------------------
void ParachuteContain::updateOffsetsFromBones()
{
	const Coord3D* objPos = getObject()->getPosition();

	getObject()->convertBonePosToWorldPos(&m_paraSwayBone, nullptr, &m_paraSwayOffset, nullptr);
	m_paraSwayOffset.x -= objPos->x;
	m_paraSwayOffset.y -= objPos->y;
	m_paraSwayOffset.z -= objPos->z;

	getObject()->convertBonePosToWorldPos(&m_paraAttachBone, nullptr, &m_paraAttachOffset, nullptr);
	m_paraAttachOffset.x -= objPos->x;
	m_paraAttachOffset.y -= objPos->y;
	m_paraAttachOffset.z -= objPos->z;

	Object* rider = (getContainCount() > 0) ? getContainList().front() : nullptr;
	if (rider)
	{
		const Coord3D* riderPos = rider->getPosition();

		rider->convertBonePosToWorldPos(&m_riderAttachBone, nullptr, &m_riderAttachOffset, nullptr);
		m_riderAttachOffset.x -= riderPos->x;
		m_riderAttachOffset.y -= riderPos->y;
		m_riderAttachOffset.z -= riderPos->z;

		m_riderAttachOffset.x = m_paraAttachOffset.x - m_riderAttachOffset.x;
		m_riderAttachOffset.y = m_paraAttachOffset.y - m_riderAttachOffset.y;
		m_riderAttachOffset.z = m_paraAttachOffset.z - m_riderAttachOffset.z;

		m_riderSwayOffset.x = m_paraSwayOffset.x - m_riderAttachOffset.x;
		m_riderSwayOffset.y = m_paraSwayOffset.y - m_riderAttachOffset.y;
		m_riderSwayOffset.z = m_paraSwayOffset.z - m_riderAttachOffset.z;
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
/**
	can this container contain this kind of object?
	and, if checkCapacity is TRUE, does this container have enough space left to hold the given unit?
*/
Bool ParachuteContain::isValidContainerFor(const Object* rider, Bool checkCapacity) const
{
	if (!rider)
		return false;

	// extend functionality
	if( OpenContain::isValidContainerFor( rider, checkCapacity ) == false )
		return false;

	Int transportSlotCount = rider->getTransportSlotCount();

	// if 0, this object isn't transportable.
	// (exception: infantry are always transportable by parachutes, regardless
	// of this.... this allows us to paradrop pilots, but not transport them
	// by other means)
	if (transportSlotCount == 0 && !rider->isKindOf(KINDOF_INFANTRY) && !rider->isKindOf(KINDOF_PARACHUTABLE))
		return false;

	// we can only "hold" one item at a time.
	if (getContainCount() > 0)
		return false;

	return true;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void ParachuteContain::containReactToTransformChange()
{
	// a bit of a cheese festival here... hidden is a flag, not a counter, so when we are dumped
	// from a plane, we might get drawn for a frame before our update is called, meaning we
	// should be briefly visible. put this here to ensure we stay hidden appropriately.
	Drawable* draw = getObject()->getDrawable();
	if (draw)
		draw->setDrawableHidden(!m_opened);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
UpdateSleepTime ParachuteContain::update()
{
	OpenContain::update();

	Object* parachute = getObject();
	if( parachute->isDisabledByType( DISABLED_HELD ) )
	{
		return UPDATE_SLEEP_NONE; // my, that was easy
	}

	AIUpdateInterface *parachuteAI = parachute->getAI();

	Drawable* draw = parachute->getDrawable();
	const ParachuteContainModuleData* d = getParachuteContainModuleData();
	Object* rider = (getContainCount() > 0) ? getContainList().front() : nullptr;

	if (m_startZ == NO_START_Z)	{
		m_startZ = parachute->getPosition()->z;
		Real groundHeight = TheTerrainLogic->getGroundHeight(parachute->getPosition()->x, parachute->getPosition()->y);
		if (m_startZ-groundHeight < 2*d->m_paraOpenDist) {
			// Oh dear - we ejected too close to the ground, and there isn't enough
			// room to open the chute.  Well, since it's only a game, we'll fudge
			// a little so that the pilot doesn't slam into the ground & stick.
			m_startZ = groundHeight+2*d->m_paraOpenDist;
		}
	}

	if (!m_opened)
	{
		// see if we need to open.
		if (WWMath::Fabs(m_startZ - parachute->getPosition()->z) >= d->m_paraOpenDist)
		{
			m_opened = true;
			parachute->clearAndSetModelConditionState(MODELCONDITION_FREEFALL, MODELCONDITION_PARACHUTING);
			m_needToUpdateParaBones = true;
			if (rider)
			{
				rider->clearAndSetModelConditionState(MODELCONDITION_FREEFALL, MODELCONDITION_PARACHUTING);
				m_needToUpdateRiderBones = true;

				AudioEventRTS soundToPlay = d->m_parachuteOpenSound;
				soundToPlay.setObjectID( rider->getID() );
				TheAudio->addAudioEvent( &soundToPlay );
			}


			// When a parachute opens, it should look for a good place to land.  This could be explicitly set
			// by a DeliverPayload, otherwise any place clear is good.
			if( parachuteAI )
			{
				Coord3D target = *parachute->getPosition();
				if( m_isLandingOverrideSet )
				{
					target = m_landingOverride;
					if( parachuteAI->getCurLocomotor() )
						parachuteAI->getCurLocomotor()->setUltraAccurate( TRUE );
				}
				else
				{
					FindPositionOptions fpOptions;
					fpOptions.minRadius = 0.0f;
					fpOptions.maxRadius = 100.0f;
					fpOptions.relationshipObject = nullptr;
					fpOptions.flags = FPF_NONE;
					ThePartitionManager->findPositionAround( &target, &fpOptions, &target );
				}
				parachuteAI->aiMoveToPosition( &target, CMD_FROM_AI );
			}
		}
    else if ( rider )
			rider->clearAndSetModelConditionState( MODELCONDITION_PARACHUTING, MODELCONDITION_FREEFALL );

  }
	draw->setDrawableHidden(!m_opened);

	if (!m_opened || getContainCount() == 0)
	{
		// unopened, or empty, chutes, don't collide with anything, to simplify
		// ejections, paradrops, landings, etc...
		parachute->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_NO_COLLISIONS ) );
		if (rider)
			rider->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_NO_COLLISIONS ) );
	}
	else
	{
		// opened/nonempty chutes DO collide...
		parachute->clearStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_NO_COLLISIONS ) );
		if (rider)
			rider->clearStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_NO_COLLISIONS ) );
	}

	AIUpdateInterface* ai = parachute->getAIUpdateInterface();
	if (ai && !parachute->isEffectivelyDead())
	{
		ai->chooseLocomotorSet(m_opened ? LOCOMOTORSET_NORMAL : LOCOMOTORSET_FREEFALL);

		Locomotor* locomotor = ai->getCurLocomotor();
		if (locomotor)
		{
			// damp the swaying a bunch when we get close, so that things land vertically (or nearly so)
			Real altitudeDamping = 0;
			if (getContainCount() > 0)
			{
				Object* rider = getContainList().front();
				const Real ALTITUDE_DAMP_START = 20.0f;
				if (rider->getHeightAboveTerrain() <= ALTITUDE_DAMP_START)
					altitudeDamping = d->m_lowAltitudeDamping;
			}

			if (m_opened)
			{
				const Real PITCH_STIFFNESS = locomotor->getPitchStiffness();
				const Real ROLL_STIFFNESS =  locomotor->getRollStiffness();
				const Real PITCH_DAMPING = locomotor->getPitchDamping() + altitudeDamping;
				const Real ROLL_DAMPING = locomotor->getRollDamping() + altitudeDamping;
				m_pitchRate += ((-PITCH_STIFFNESS * m_pitch) + (-PITCH_DAMPING * m_pitchRate));		// spring/damper
				m_rollRate += ((-ROLL_STIFFNESS * m_roll) + (-ROLL_DAMPING * m_rollRate));		// spring/damper

				m_pitch += m_pitchRate;
				m_roll += m_rollRate;

				if( m_isLandingOverrideSet )
				{
					// Need to wait until after opening to do this, or else we are sending the message to the Freefall locomotor.
					locomotor->setCloseEnoughDist( 10.0 );
					locomotor->setCloseEnoughDist3D( FALSE );
				}
			}

			if (draw)
			{
				updateBonePositions();
				updateOffsetsFromBones();

				Matrix3D tmp;
				calcSwayMtx(&m_paraSwayOffset, &tmp);
				draw->setInstanceMatrix(&tmp);
			}

			positionContainedObjectsRelativeToContainer();
		}
	}

	// allow us to land on bridges!
	const Coord3D* paraPos = getObject()->getPosition();
	PathfindLayerEnum newLayer = TheTerrainLogic->getHighestLayerForDestination(paraPos);
	getObject()->setLayer(newLayer);
	if (rider)
		rider->setLayer(newLayer);

	// If we have lost our passenger for whatever reason, die early.  Otherwise we just sit around forever.
	if( getContainCount() == 0 )
		getObject()->kill();

	// the collide system doesn't always collide us with the ground if we fall into water.
	// so force the issue.
	Real waterZ;
	if (!getObject()->isEffectivelyDead() &&
			getObject()->getLayer() == LAYER_GROUND &&
			TheTerrainLogic->isUnderwater(paraPos->x, paraPos->y, &waterZ) &&
			(paraPos->z - waterZ) < d->m_killWhenLandingInWaterSlop)
	{
		getObject()->kill();
	}


	return UPDATE_SLEEP_NONE;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void ParachuteContain::onContaining( Object *rider, Bool wasSelected )
{
	OpenContain::onContaining( rider, wasSelected );

	// objects inside a transport are held
	rider->setDisabled( DISABLED_HELD );
	rider->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_PARACHUTING ) );

	rider->clearAndSetModelConditionState(MODELCONDITION_PARACHUTING, MODELCONDITION_FREEFALL);
	m_needToUpdateRiderBones = true;

	// position him correctly.
	positionRider(rider);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void ParachuteContain::onRemoving( Object *rider )
{
	OpenContain::onRemoving(rider);

	const ParachuteContainModuleData* d = getParachuteContainModuleData();

	// object is no longer held inside a transport
	rider->clearDisabled( DISABLED_HELD );
	rider->clearStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_PARACHUTING ) );

	// mark parachute as "no-collisions"... it is just ephemeral at this point,
	// and having the chute collide with the soldier (and both bounce apart) is
	// just dumb-lookin'...
	getObject()->setStatus( MAKE_OBJECT_STATUS_MASK( OBJECT_STATUS_NO_COLLISIONS ) );

	// position him correctly.
	positionRider(rider);

	rider->clearModelConditionFlags(MAKE_MODELCONDITION_MASK2(MODELCONDITION_FREEFALL, MODELCONDITION_PARACHUTING));
	m_needToUpdateRiderBones = true;

	// temporarily mark the guy as being allowed to fall
	// (overriding his locomotor's stick-to-ground attribute).
	// this will be reset (by PhysicsBehavior) when he touches the ground.
	PhysicsBehavior* physics = rider->getPhysics();
	if (physics)
	{
		physics->setAllowToFall(true);

		Coord3D force;
		force.zero();
		physics->applyForce(&force);	// force its physics to wake up... should be done when DISABLED_HELD is cleared, but it not, and scared to do it now.
	}


	AIUpdateInterface* riderAI = rider->getAIUpdateInterface();
	if (riderAI)
	{
		Player* controller = rider->getControllingPlayer();
		if (controller && controller->isSkirmishAIPlayer())
		{
			riderAI->aiHunt(CMD_FROM_AI);	// hunt, as per Dustin's request.
		}
		else
		{
			bool hasRallyPoint = false;
			// Get the transport of the rider
			Object *transport = TheGameLogic->findObjectByID(rider->getProducerID());
			if (transport)
			{
				// Get the building that produced the transport
				Object *transportProducer = TheGameLogic->findObjectByID(transport->getProducerID());
				if (transportProducer)
				{
					// See if we need to set a rally point for the object being parachuted
					ExitInterface* exitInterface = transportProducer->getObjectExitInterface();
					if (exitInterface && exitInterface->useSpawnRallyPoint())
					{
						exitInterface->exitObjectViaDoor(rider, DOOR_1);
						hasRallyPoint = true;
					}
				}
			}

			if (!hasRallyPoint)
				riderAI->aiIdle(CMD_FROM_AI); // become idle.
		}
	}

	// if we land in the water, we die. alas.
	const Coord3D* riderPos = rider->getPosition();
	Real waterZ, terrainZ;
	if (TheTerrainLogic->isUnderwater(riderPos->x, riderPos->y, &waterZ, &terrainZ) &&
			riderPos->z <= waterZ + d->m_killWhenLandingInWaterSlop &&
			rider->getLayer() == LAYER_GROUND)
	{
		// don't call kill(); do it manually, so we can specify DEATH_FLOODED
		DamageInfo damageInfo;
		damageInfo.in.m_damageType = DAMAGE_WATER;	// use this instead of UNRESISTABLE so we don't get a dusty damage effect
		damageInfo.in.m_deathType = DEATH_FLOODED;
		damageInfo.in.m_sourceID = INVALID_ID;
		damageInfo.in.m_amount = HUGE_DAMAGE_AMOUNT;
		rider->attemptDamage( &damageInfo );
	}

	// Kill if we landed on impassable ground
	Int cellX = REAL_TO_INT( rider->getPosition()->x / PATHFIND_CELL_SIZE );
	Int cellY = REAL_TO_INT( rider->getPosition()->y / PATHFIND_CELL_SIZE );

	PathfindCell* cell = TheAI->pathfinder()->getCell( rider->getLayer(), cellX, cellY );
	PathfindCell::CellType cellType = cell ? cell->getType() : PathfindCell::CELL_IMPASSABLE;

	// If we land outside the map from a faulty parachute, we die too.
	// Otherwise we exist outside the PartitionManger like a cheater.
	if( rider->isOffMap() ||
    (cellType == PathfindCell::CELL_CLIFF) ||
    (cellType == PathfindCell::CELL_WATER) ||
    (cellType == PathfindCell::CELL_IMPASSABLE) )
	{
		// The Paradrop command was legal, the parachute destination was legal, but the parachute
		// can still fail to adjust back on the map.  SO this is the place to cap the cheater.
		rider->kill();
	}

  // Note: for future enhancement of this feature, we should test the object against the cell type he is on,
  // using obj->getAI()->hasLocomotorForSurface( __ ). We cshould not assume here that the parachutist can not
  // find happiness on cliffs or water or whatever.


}

//-------------------------------------------------------------------------------------------------
void ParachuteContain::positionRider(Object* rider)
{
	updateBonePositions();
	updateOffsetsFromBones();

	Coord3D pos = *getObject()->getPosition();
	///DUMPCOORD3D(&pos);
	pos.x += m_riderAttachOffset.x;
	pos.y += m_riderAttachOffset.y;
	pos.z += m_riderAttachOffset.z;
	//DUMPCOORD3D(&pos);
	rider->setPosition(&pos);

	Real alt = rider->getHeightAboveTerrain();
	if (alt < 0.0f)
	{
		// don't let him go below ground.
		pos.z -= alt;
		rider->setPosition(&pos);
	}

	rider->setOrientation(getObject()->getOrientation());

	Drawable* draw = rider->getDrawable();
	if (draw)
	{
		if( rider->isDisabledByType( DISABLED_HELD ) )
		{
			Matrix3D tmp;
			calcSwayMtx(&m_riderSwayOffset, &tmp);
			draw->setInstanceMatrix(&tmp);
		}
		else
		{
			draw->setInstanceMatrix(nullptr);
		}
	}
}

//-------------------------------------------------------------------------------------------------
void ParachuteContain::positionContainedObjectsRelativeToContainer()
{
	for(ContainedItemsList::const_iterator it = getContainList().begin(); it != getContainList().end(); ++it)
	{
		positionRider(*it);
	}
}

//-------------------------------------------------------------------------------------------------
void ParachuteContain::setOverrideDestination( const Coord3D *dest )
{
	// Instead of trying to float straight down, I am going to nail this spot.
	m_landingOverride = *dest;
	m_isLandingOverrideSet = TRUE;
}

//-------------------------------------------------------------------------------------------------
void ParachuteContain::onDie( const DamageInfo * damageInfo )
{
	// if we are airborne when killed, the guy falls screaming to his death...
	if (getObject()->isSignificantlyAboveTerrain())
	{
		Object* rider = (getContainCount() > 0) ? getContainList().front() : nullptr;
		if (rider)
		{
			removeAllContained();
			const ParachuteContainModuleData* d = getParachuteContainModuleData();
			if (d->m_freeFallDamagePercent > 0.0f)
			{
				// do some damage just for losing your parachute.
				// not very realistic, but practical to help ensure that
				// you really do die from going "splat" on the ground.
				DamageInfo extraDamageInfo;
				extraDamageInfo.in.m_damageType = DAMAGE_FALLING;
				extraDamageInfo.in.m_deathType = DEATH_SPLATTED;
				extraDamageInfo.in.m_sourceID = damageInfo->in.m_sourceID;
				extraDamageInfo.in.m_amount = rider->getBodyModule()->getMaxHealth() * d->m_freeFallDamagePercent;
				rider->attemptDamage(&extraDamageInfo);
			}
			PhysicsBehavior* physics = rider->getPhysics();
			if (physics)
			{
				physics->setAllowToFall(true);
				physics->setIsInFreeFall(true);	// bwah ha ha

				Coord3D force;
				force.zero();
				physics->applyForce(&force);	// force its physics to wake up... should be done when DISABLED_HELD is cleared, but it not, and scared to do it now.
			}
		}
	}


	OpenContain::onDie(damageInfo);
}

//-------------------------------------------------------------------------------------------------
void ParachuteContain::onCollide( Object *other, const Coord3D *loc, const Coord3D *normal )
{
	// Note that other == null means "collide with ground"
	if( other == nullptr )
	{
		// if we're in a container (eg, a transport plane), just ignore this...
		if( getObject()->getContainedBy() != nullptr )
			return;

		removeAllContained();

		// TheGameLogic->destroyObject(obj);
		// kill it, so that the chute's SlowDeath will trigger!
		getObject()->kill();
	}
}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void ParachuteContain::crc( Xfer *xfer )
{

	// extend base class
	OpenContain::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void ParachuteContain::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	OpenContain::xfer( xfer );

	// pitch
	xfer->xferReal( &m_pitch );

	// roll
	xfer->xferReal( &m_roll );

	// pitch rage
	xfer->xferReal( &m_pitchRate );

	// roll rate
	xfer->xferReal( &m_rollRate );

	// start Z
	xfer->xferReal( &m_startZ );

	// is landing override set
	xfer->xferBool( &m_isLandingOverrideSet );

	// landing override
	xfer->xferCoord3D( &m_landingOverride );

	// rider attach bone
	xfer->xferCoord3D( &m_riderAttachBone );

	// rider sway bone
	xfer->xferCoord3D( &m_riderSwayBone );

	// para attach bone
	xfer->xferCoord3D( &m_paraAttachBone );

	// para sway bone
	xfer->xferCoord3D( &m_paraSwayBone );

	// rider attach offset
	xfer->xferCoord3D( &m_riderAttachOffset );

	// rider sway offset
	xfer->xferCoord3D( &m_riderSwayOffset );

	// para attach offset
	xfer->xferCoord3D( &m_paraAttachOffset );

	// para sway offset
	xfer->xferCoord3D( &m_paraSwayOffset );

	// need to update rider bones
	xfer->xferBool( &m_needToUpdateRiderBones );

	// need to update para bones
	xfer->xferBool( &m_needToUpdateParaBones );

	// opened
	xfer->xferBool( &m_opened );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void ParachuteContain::loadPostProcess()
{

	// extend base class
	OpenContain::loadPostProcess();

}
```
