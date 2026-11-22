# Translation Packet `PKT-398`: `ThingTemplate`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-398`
- **Work Package:** `WP-5.5` (Phase 5)
- **Complexity & Assigned Tier:** Class `C4` -> `tier:3-architect`
- **Environment Access:** `env:hybrid-ci`
- **FFI Seam Pattern:** Core Spine
- **Allowed Rust Output File:** `crates/zh_logic/src/object/thing_template.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (42):**
  - `Common/AudioEventRTS.h`
  - `Common/BitFlagsIO.h`
  - `Common/DamageFX.h`
  - `Common/FileSystem.h`
  - `Common/GameAudio.h`
  - `Common/GameCommon.h`
  - `Common/Geometry.h`
  - `Common/GlobalData.h`
  - `Common/INI.h`
  - `Common/KindOf.h`
  - `Common/MessageStream.h`
  - `Common/Module.h`
  - `Common/ModuleFactory.h`
  - `Common/Overridable.h`
  - `Common/Player.h`
  - `Common/PlayerList.h`
  - `Common/ProductionPrerequisite.h`
  - `Common/Radar.h`
  - `Common/RandomValue.h`
  - `Common/STLTypedefs.h`
  - `Common/Science.h`
  - `Common/ThingFactory.h`
  - `Common/ThingSort.h`
  - `Common/ThingTemplate.h`
  - `Common/UnicodeString.h`
  - `Common/UnitTimings.h`
  - `GameClient/Color.h`
  - `GameClient/Drawable.h`
  - `GameClient/FXList.h`
  - `GameClient/GameText.h`
  - `GameClient/Image.h`
  - `GameClient/Shadow.h`
  - `GameLogic/Armor.h`
  - `GameLogic/ArmorSet.h`
  - `GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/SpecialPowerModule.h`
  - `GameLogic/Object.h`
  - `GameLogic/Powers.h`
  - `GameLogic/Weapon.h`
  - `GameLogic/WeaponSet.h`
  - `Lib/BaseType.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (134):**
  - `"DisplayName" => INI::parseAndTranslateLabel`
  - `"RadarPriority" => INI::parseByteSizedIndexList`
  - `"TransportSlotCount" => INI::parseUnsignedByte`
  - `"FenceWidth" => INI::parseReal`
  - `"FenceXOffset" => INI::parseReal`
  - `"IsBridge" => INI::parseBool`
  - `"ArmorSet" => ThingTemplate::parseArmorTemplateSet`
  - `"WeaponSet" => ThingTemplate::parseWeaponTemplateSet`
  - `"VisionRange" => INI::parseReal`
  - `"ShroudClearingRange" => INI::parseReal`
  - `"ShroudRevealToAllRange" => INI::parseReal`
  - `"PlacementViewAngle" => INI::parseAngleReal`
  - `"FactoryExitWidth" => INI::parseReal`
  - `"FactoryExtraBibWidth" => INI::parseReal`
  - `"SkillPointValue" => ThingTemplate::parseIntList`
  - `"ExperienceValue" => ThingTemplate::parseIntList`
  - `"ExperienceRequired" => ThingTemplate::parseIntList`
  - `"IsTrainable" => INI::parseBool`
  - `"EnterGuard" => INI::parseBool`
  - `"HijackGuard" => INI::parseBool`
  - `"Side" => INI::parseAsciiString`
  - `"Prerequisites" => ThingTemplate::parsePrerequisites`
  - `"Buildable" => INI::parseByteSizedIndexList`
  - `"BuildCost" => INI::parseUnsignedShort`
  - `"BuildTime" => INI::parseReal`
  - `"RefundValue" => INI::parseUnsignedShort`
  - `"BuildCompletion" => INI::parseByteSizedIndexList`
  - `"EnergyProduction" => INI::parseInt`
  - `"EnergyBonus" => INI::parseInt`
  - `"IsForbidden" => INI::parseBool`
  - `"IsPrerequisite" => INI::parseBool`
  - `"DisplayColor" => INI::parseColorInt`
  - `"EditorSorting" => INI::parseByteSizedIndexList`
  - `"KindOf" => parseKindOfFromINI`
  - `"CommandSet" => INI::parseAsciiString`
  - `"BuildVariations" => INI::parseAsciiStringVector`
  - `"Behavior" => ThingTemplate::parseModuleName`
  - `"Body" => ThingTemplate::parseModuleName`
  - `"Draw" => ThingTemplate::parseModuleName`
  - `"ClientUpdate" => ThingTemplate::parseModuleName`
  - `"SelectPortrait" => INI::parseAsciiString`
  - `"ButtonImage" => INI::parseAsciiString`
  - `"InventoryImageEnabled" => INI::parseAsciiString`
  - `"InventoryImageDisabled" => INI::parseAsciiString`
  - `"InventoryImageHilite" => INI::parseAsciiString`
  - `"InventoryImagePushed" => INI::parseAsciiString`
  - `"UpgradeCameo1" => INI::parseAsciiString`
  - `"UpgradeCameo2" => INI::parseAsciiString`
  - `"UpgradeCameo3" => INI::parseAsciiString`
  - `"UpgradeCameo4" => INI::parseAsciiString`
  - `"UpgradeCameo5" => INI::parseAsciiString`
  - `"VoiceSelect" => INI::parseDynamicAudioEventRTS`
  - `"VoiceGroupSelect" => INI::parseDynamicAudioEventRTS`
  - `"VoiceMove" => INI::parseDynamicAudioEventRTS`
  - `"VoiceAttack" => INI::parseDynamicAudioEventRTS`
  - `"VoiceEnter" => INI::parseDynamicAudioEventRTS`
  - `"VoiceFear" => INI::parseDynamicAudioEventRTS`
  - `"VoiceSelectElite" => INI::parseDynamicAudioEventRTS`
  - `"VoiceCreated" => INI::parseDynamicAudioEventRTS`
  - `"VoiceTaskUnable" => INI::parseDynamicAudioEventRTS`
  - `"VoiceTaskComplete" => INI::parseDynamicAudioEventRTS`
  - `"VoiceMeetEnemy" => INI::parseDynamicAudioEventRTS`
  - `"VoiceGarrison" => INI::parseDynamicAudioEventRTS`
  - `"VoiceSurrender" => INI::parseDynamicAudioEventRTS`
  - `"VoiceDefect" => INI::parseDynamicAudioEventRTS`
  - `"VoiceAttackSpecial" => INI::parseDynamicAudioEventRTS`
  - `"VoiceAttackAir" => INI::parseDynamicAudioEventRTS`
  - `"VoiceGuard" => INI::parseDynamicAudioEventRTS`
  - `"SoundMoveStart" => INI::parseDynamicAudioEventRTS`
  - `"SoundMoveStartDamaged" => INI::parseDynamicAudioEventRTS`
  - `"SoundMoveLoop" => INI::parseDynamicAudioEventRTS`
  - `"SoundMoveLoopDamaged" => INI::parseDynamicAudioEventRTS`
  - `"SoundDie" => INI::parseDynamicAudioEventRTS`
  - `"SoundDieFire" => INI::parseDynamicAudioEventRTS`
  - `"SoundDieToxin" => INI::parseDynamicAudioEventRTS`
  - `"SoundCrush" => INI::parseDynamicAudioEventRTS`
  - `"SoundAmbient" => INI::parseDynamicAudioEventRTS`
  - `"SoundAmbientDamaged" => INI::parseDynamicAudioEventRTS`
  - `"SoundAmbientReallyDamaged" => INI::parseDynamicAudioEventRTS`
  - `"SoundAmbientRubble" => INI::parseDynamicAudioEventRTS`
  - `"SoundStealthOn" => INI::parseDynamicAudioEventRTS`
  - `"SoundStealthOff" => INI::parseDynamicAudioEventRTS`
  - `"SoundCreated" => INI::parseDynamicAudioEventRTS`
  - `"SoundOnDamaged" => INI::parseDynamicAudioEventRTS`
  - `"SoundOnReallyDamaged" => INI::parseDynamicAudioEventRTS`
  - `"SoundEnter" => INI::parseDynamicAudioEventRTS`
  - `"SoundExit" => INI::parseDynamicAudioEventRTS`
  - `"SoundPromotedVeteran" => INI::parseDynamicAudioEventRTS`
  - `"SoundPromotedElite" => INI::parseDynamicAudioEventRTS`
  - `"SoundPromotedHero" => INI::parseDynamicAudioEventRTS`
  - `"SoundFallingFromPlane" => INI::parseDynamicAudioEventRTS`
  - `"UnitSpecificSounds" => ThingTemplate::parsePerUnitSounds`
  - `"UnitSpecificFX" => ThingTemplate::parsePerUnitFX`
  - `"Scale" => INI::parseReal`
  - `"Geometry" => GeometryInfo::parseGeometryType`
  - `"GeometryMajorRadius" => GeometryInfo::parseGeometryMajorRadius`
  - `"GeometryMinorRadius" => GeometryInfo::parseGeometryMinorRadius`
  - `"GeometryHeight" => GeometryInfo::parseGeometryHeight`
  - `"GeometryIsSmall" => GeometryInfo::parseGeometryIsSmall`
  - `"Shadow" => INI::parseBitString8`
  - `"ShadowSizeX" => INI::parseReal`
  - `"ShadowSizeY" => INI::parseReal`
  - `"ShadowOffsetX" => INI::parseReal`
  - `"ShadowOffsetY" => INI::parseReal`
  - `"ShadowTexture" => INI::parseAsciiString`
  - `"OcclusionDelay" => INI::parseDurationUnsignedInt`
  - `"AddModule" => ThingTemplate::parseAddModule`
  - `"RemoveModule" => ThingTemplate::parseRemoveModule`
  - `"ReplaceModule" => ThingTemplate::parseReplaceModule`
  - `"InheritableModule" => ThingTemplate::parseInheritableModule`
  - `"OverrideableByLikeKind" => ThingTemplate::OverrideableByLikeKind`
  - `"Locomotor" => AIUpdateModuleData::parseLocomotorSet`
  - `"InstanceScaleFuzziness" => INI::parseReal`
  - `"StructureRubbleHeight" => INI::parseUnsignedByte`
  - `"ThreatValue" => INI::parseUnsignedShort`
  - `"MaxSimultaneousOfType" => ThingTemplate::parseMaxSimultaneous`
  - `"MaxSimultaneousLinkKey" => NameKeyGenerator::parseStringAsNameKeyType`
  - `"CrusherLevel" => INI::parseUnsignedByte`
  - `"CrushableLevel" => INI::parseUnsignedByte`
  - `"Draw" => ThingTemplate::parseModuleName`
  - `"Geometry" => GeometryInfo::parseGeometryType`
  - `"GeometryMajorRadius" => GeometryInfo::parseGeometryMajorRadius`
  - `"GeometryMinorRadius" => GeometryInfo::parseGeometryMinorRadius`
  - `"GeometryHeight" => GeometryInfo::parseGeometryHeight`
  - `"GeometryIsSmall" => GeometryInfo::parseGeometryIsSmall`
  - `"FenceWidth" => INI::parseReal`
  - `"FenceXOffset" => INI::parseReal`
  - `"MaxSimultaneousOfType" => ThingTemplate::parseMaxSimultaneous`
  - `"MaxSimultaneousLinkKey" => NameKeyGenerator::parseStringAsNameKeyType`
  - `"Object" => parsePrerequisiteUnit`
  - `"Science" => parsePrerequisiteScience`
  - `"Conditions" => ArmorSetFlags::parseFromINI`
  - `"Armor" => INI::parseArmorTemplate`
  - `"DamageFX" => INI::parseDamageFX`
- **`Snapshot::crc` / `Snapshot::xfer` Calls:** None in module.
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if !(RTS_GENERALS && RETAIL_COMPATIBLE_CRC)`
  - `#if RTS_GENERALS && RETAIL_COMPATIBLE_DATA`

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

### `Core/GameEngine/Include/Common/ThingTemplate.h`
```cpp
// FILE: ThingTemplate.h //////////////////////////////////////////////////////////////////////////
// Author: Colin Day, April 2001
// Desc:	 Thing templates are a 'roadmap' to creating things
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Lib/BaseType.h"

#include "Common/AudioEventRTS.h"
#include "Common/FileSystem.h"
#include "Common/GameCommon.h"
#include "Common/Geometry.h"
#include "Common/KindOf.h"
#include "Common/ModuleFactory.h"
#include "Common/Overridable.h"
#include "Common/ProductionPrerequisite.h"
#include "Common/Science.h"
#include "Common/UnicodeString.h"

#include "GameLogic/ArmorSet.h"
#include "GameLogic/WeaponSet.h"
#include "Common/STLTypedefs.h"
#include "GameClient/Color.h"

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
class AIUpdateModuleData;
class Image;
class Object;
class Drawable;
class ProductionPrerequisite;
struct FieldParse;
class Player;
class INI;
enum RadarPriorityType CPP_11(: Int);
enum ScienceType CPP_11(: Int);
enum EditorSortingType CPP_11(: Int);
enum ShadowType CPP_11(: Int);
class WeaponTemplateSet;
class ArmorTemplateSet;
class FXList;

// TYPEDEFS FOR FILE //////////////////////////////////////////////////////////////////////////////
typedef std::map<AsciiString, AudioEventRTS> PerUnitSoundMap;
typedef std::map<AsciiString, const FXList*> PerUnitFXMap;

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
//Code renderer handles these states now.
//enum InventoryImageType
//{
//	INV_IMAGE_ENABLED = 0,
//	INV_IMAGE_DISABLED,
//	INV_IMAGE_HILITE,
//	INV_IMAGE_PUSHED,
//
//	INV_IMAGE_NUM_IMAGES
//
//};
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
enum
{
	MAX_UPGRADE_CAMEO_UPGRADES = 5
};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
enum ThingTemplateAudioType CPP_11(: Int)
{
	TTAUDIO_voiceSelect,							///< Response when unit is selected
	TTAUDIO_voiceGroupSelect,					///< Response when a group of this unit is selected
	TTAUDIO_voiceSelectElite,					///< Response when unit is selected and elite
	TTAUDIO_voiceMove,								///< Response when unit moves
	TTAUDIO_voiceAttack,							///< Response when unit is told to attack
	TTAUDIO_voiceEnter,								///< Response when unit is told to enter a building
	TTAUDIO_voiceFear,								///< Response when unit is under attack
	TTAUDIO_voiceCreated,							///< Response when unit is created
	TTAUDIO_voiceNearEnemy,						///< Unit is near an enemy
	TTAUDIO_voiceTaskUnable,					///< Unit is told to do something impossible
	TTAUDIO_voiceTaskComplete,				///< Unit completes a move, or other task indicated
	TTAUDIO_voiceMeetEnemy,						///< Unit meets an enemy unit
	TTAUDIO_soundMoveStart,						///< Sound when unit starts moving
	TTAUDIO_soundMoveStartDamaged,		///< Sound when unit starts moving and is damaged
	TTAUDIO_soundMoveLoop,						///< Sound when unit is moving
	TTAUDIO_soundMoveLoopDamaged,			///< Sound when unit is moving and is damaged
#if RTS_GENERALS && RETAIL_COMPATIBLE_DATA
	TTAUDIO_soundDie,									///< Sound when unit is dieing
	TTAUDIO_soundDieFire,							///< Sound when unit dies by fire. NOTE: Replaces soundDie if present and unit dies by fire.
	TTAUDIO_soundDieToxin,						///< Sound when unit dies by Toxin. NOTE: Replaces soundDie if present and unit dies by fire.
	TTAUDIO_soundCrush,								///< Sound when unit is crushed
#endif
	TTAUDIO_soundAmbient,							///< Ambient sound for unit during normal status. Also the default sound
	TTAUDIO_soundAmbientDamaged,			///< Ambient sound for unit if damaged. Corresponds to body info damage
	TTAUDIO_soundAmbientReallyDamaged,///< Ambient sound for unit if badly damaged.
	TTAUDIO_soundAmbientRubble,				///< Ambient sound for unit if it is currently rubble. (Dam, for instance)
	TTAUDIO_soundStealthOn,           ///< Sound when unit stealths
	TTAUDIO_soundStealthOff,          ///< Sound when unit destealths
	TTAUDIO_soundCreated,							///< Sound when unit is created
	TTAUDIO_soundOnDamaged,           ///< Sound when unit enters damaged state
	TTAUDIO_soundOnReallyDamaged,     ///< Sound when unit enters really damaged state
	TTAUDIO_soundEnter,								///< Sound when another unit enters me.
	TTAUDIO_soundExit,								///< Sound when another unit exits me.
	TTAUDIO_soundPromotedVeteran,			///< Sound when unit gets promoted to Veteran level
	TTAUDIO_soundPromotedElite,				///< Sound when unit gets promoted to Elite level
	TTAUDIO_soundPromotedHero,				///< Sound when unit gets promoted to Hero level
	TTAUDIO_voiceGarrison,						///< Unit is ordered to enter a garrisonable building
	TTAUDIO_soundFalling,							///< This sound is actually called on a unit when it is exiting another.
																		///< However, there is a soundExit which refers to the container, and this is only used for bombs falling from planes.
#ifdef ALLOW_SURRENDER
	TTAUDIO_voiceSurrender,						///< Unit surrenders
#endif
	TTAUDIO_voiceDefect,							///< Unit is forced to defect
	TTAUDIO_voiceAttackSpecial,				///< Unit is ordered to use a special attack
	TTAUDIO_voiceAttackAir,						///< Unit is ordered to attack an airborne unit
	TTAUDIO_voiceGuard,								///< Unit is ordered to guard an area

	TTAUDIO_COUNT
};

class AudioArray
{
public:
	RefCountPtr<DynamicAudioEventRTS> m_audio[TTAUDIO_COUNT];

	AudioArray() {}

	~AudioArray() {}

	AudioArray(const AudioArray& that)
	{
		for (Int i = 0; i < TTAUDIO_COUNT; ++i)
		{
			if (that.m_audio[i])
				m_audio[i].Assign_No_Add_Ref(newInstance(DynamicAudioEventRTS)(*that.m_audio[i]));
			else
				m_audio[i] = nullptr;
		}
	}

	AudioArray& operator=(const AudioArray& that)
	{
		if (this != &that)
		{
			for (Int i = 0; i < TTAUDIO_COUNT; ++i)
			{
				if (that.m_audio[i])
				{
					if (m_audio[i])
						*m_audio[i] = *that.m_audio[i];
					else
						m_audio[i].Assign_No_Add_Ref(newInstance(DynamicAudioEventRTS)(*that.m_audio[i]));
				}
				else
				{
					m_audio[i] = nullptr;
				}
			}
		}
		return *this;
	}
};

//-------------------------------------------------------------------------------------------------
/** Object class type enumeration */
//-------------------------------------------------------------------------------------------------
enum BuildCompletionType CPP_11(: Int)
{
	BC_INVALID = 0,
	BC_APPEARS_AT_RALLY_POINT,	///< unit appears at rally point of its #1 prereq
	BC_PLACED_BY_PLAYER,				///< unit must be manually placed by player

	BC_NUM_TYPES
};
#ifdef DEFINE_BUILD_COMPLETION_NAMES
static const char *const BuildCompletionNames[] =
{
	"INVALID",
	"APPEARS_AT_RALLY_POINT",
	"PLACED_BY_PLAYER",

	nullptr
};
static_assert(ARRAY_SIZE(BuildCompletionNames) == BC_NUM_TYPES + 1, "Incorrect array size");
#endif  // end DEFINE_BUILD_COMPLETION_NAMES

enum BuildableStatus CPP_11(: Int)
{
	// saved into savegames... do not change or remove values!
	BSTATUS_YES = 0,
	BSTATUS_IGNORE_PREREQUISITES,
	BSTATUS_NO,
	BSTATUS_ONLY_BY_AI,

	BSTATUS_NUM_TYPES
};

#ifdef DEFINE_BUILDABLE_STATUS_NAMES
static const char *const BuildableStatusNames[] =
{
	"Yes",
	"Ignore_Prerequisites",
	"No",
	"Only_By_AI",
	nullptr
};
static_assert(ARRAY_SIZE(BuildableStatusNames) == BSTATUS_NUM_TYPES + 1, "Incorrect array size");
#endif	// end DEFINE_BUILDABLE_STATUS_NAMES

//-------------------------------------------------------------------------------------------------
enum ModuleParseMode CPP_11(: Int)
{
	MODULEPARSE_NORMAL,
	MODULEPARSE_ADD_REMOVE_REPLACE,
	MODULEPARSE_INHERITABLE,
  MODULEPARSE_OVERRIDEABLE_BY_LIKE_KIND,

};

//-------------------------------------------------------------------------------------------------
class ModuleInfo
{
private:
	struct Nugget
	{
		AsciiString first;
		AsciiString m_moduleTag;
		const ModuleData* second;
		Int interfaceMask;
		Bool copiedFromDefault;
		Bool inheritable;
    Bool overrideableByLikeKind;

		Nugget(const AsciiString& n, const AsciiString& moduleTag, const ModuleData* d, Int i, Bool inh, Bool oblk)
		: first(n),
			m_moduleTag(moduleTag),
			second(d),
			interfaceMask(i),
			copiedFromDefault(false),
			inheritable(inh),
      overrideableByLikeKind(oblk)
		{
		}

	};
	std::vector<Nugget> m_info;

public:

	ModuleInfo() { }

	void addModuleInfo( ThingTemplate *thingTemplate, const AsciiString& name, const AsciiString& moduleTag, const ModuleData* data, Int interfaceMask, Bool inheritable, Bool overrideableByLikeKind = FALSE );
	const ModuleInfo::Nugget *getNuggetWithTag( const AsciiString& tag ) const;

	Int getCount() const
	{
		return m_info.size();
	}

#if defined(RTS_DEBUG)
	Bool containsPartialName(const char* n) const
	{
		for (size_t i = 0; i < m_info.size(); i++)
			if (strstr(m_info[i].first.str(), n) != nullptr)
				return true;
		return false;
	}
#endif

	AsciiString getNthName(size_t i) const
	{
		if (i >= 0 && i < m_info.size())
		{
			return m_info[i].first;
		}
		return AsciiString::TheEmptyString;
	}

	AsciiString getNthTag(size_t i) const
	{
		if (i >= 0 && i < m_info.size())
		{
			return m_info[i].m_moduleTag;
		}
		return AsciiString::TheEmptyString;
	}

	const ModuleData* getNthData(size_t i) const
	{
		if (i >= 0 && i < m_info.size())
		{
			return m_info[i].second;
		}
		return nullptr;
	}

	// for use only by ThingTemplate::friend_getAIModuleInfo
	ModuleData* friend_getNthData(Int i);

	void clear()
	{
		m_info.clear();
	}

	void setCopiedFromDefault(Bool v)
	{
		for (size_t i = 0; i < m_info.size(); i++)
			m_info[i].copiedFromDefault = v;
	}

	Bool clearModuleDataWithTag(const AsciiString& tagToClear, AsciiString& clearedModuleNameOut);
	Bool clearCopiedFromDefaultEntries(Int interfaceMask, const AsciiString &name, const ThingTemplate *fullTemplate );
	Bool clearAiModuleInfo();
};

//-------------------------------------------------------------------------------------------------
/** Definition of a thing template to read from our game data framework */
//-------------------------------------------------------------------------------------------------
class ThingTemplate : public Overridable
{

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(ThingTemplate, "ThingTemplatePool" )

private:

#if defined(_MSC_VER) && _MSC_VER < 1300
	ThingTemplate(const ThingTemplate& that) : m_geometryInfo(that.m_geometryInfo)
	{
		DEBUG_CRASH(("This should never be called"));
	}
#else
	ThingTemplate(const ThingTemplate& that) = delete;
#endif

public:


	ThingTemplate();

	// copy the guts of that into this, but preserve this' name, id, and list-links.
	void copyFrom(const ThingTemplate* that);

	/// called by ThingFactory after all templates have been loaded.
	void resolveNames();

#ifdef LOAD_TEST_ASSETS
	void initForLTA(const AsciiString& name);
	inline AsciiString getLTAName() const { return m_LTAName; }
#endif

	/**
		return a unique identifier suitable for identifying this ThingTemplate on machines playing
		across the net. this should be considered a Magic Cookie and used only for net traffic or
		similar sorts of things. To convert an id back to a ThingTemplate, use ThingFactory::findByID().
		Note that 0 is always an invalid id.  NOTE that we are not referencing m_override here
		because even though we actually have multiple templates here representing overrides,
		we still only conceptually have one template and want to always use one single
		pointer for comparisons of templates.  However, even if we did reference m_override
		the IDs would be the same for each one since every override first *COPIES* data
		from the current/parent template data.
	*/
	UnsignedShort getTemplateID() const { return m_templateID; }

	// note that m_override is not used here, see getTemplateID(), for it is the same reasons
	const AsciiString& getName() const { return m_nameString; }  ///< return the name of this template

	/// get the display color (used for the editor)
	Color getDisplayColor() const { return m_displayColor; }

	/// get the editor sorting
	EditorSortingType getEditorSorting() const { return (EditorSortingType)m_editorSorting; }

	/// return true iff the template has the specified kindOf flag set.
	Bool isKindOf(KindOfType t) const
	{
		return TEST_KINDOFMASK(m_kindof, t);
	}

	/// convenience for doing multiple kindof testing at once.
	Bool isKindOfMulti(const KindOfMaskType& mustBeSet, const KindOfMaskType& mustBeClear) const
	{
		return TEST_KINDOFMASK_MULTI(m_kindof, mustBeSet, mustBeClear);
	}

	Bool isAnyKindOf( const KindOfMaskType& anyKindOf ) const
	{
		return TEST_KINDOFMASK_ANY(m_kindof, anyKindOf);
	}

	/// set the display name
	const UnicodeString& getDisplayName() const { return m_displayName; }  ///< return display name

	RadarPriorityType getDefaultRadarPriority() const { return (RadarPriorityType)m_radarPriority; }  ///< return radar priority from INI

	// note, you should not call this directly; rather, call Object::getTransportSlotCount().
	Int getRawTransportSlotCount() const { return m_transportSlotCount; }

	Real getFenceWidth() const { return m_fenceWidth; }  // return fence width

	Real getFenceXOffset() const { return m_fenceXOffset; }  // return fence offset

	Bool isBridge() const { return m_isBridge; }  // return fence offset
	Bool isBridgeLike() const { return isBridge() || isKindOf(KINDOF_WALK_ON_TOP_OF_WALL); }

	// Only Object can ask this.  Everyone else should ask the Object.  In fact, you really should ask the Object everything.
	Real friend_calcVisionRange() const { return m_visionRange; }  ///< get vision range
	Real friend_calcShroudClearingRange() const { return m_shroudClearingRange; }  ///< get vision range for Shroud ONLY (Design requested split)

	//This one is okay to check directly... because it doesn't get effected by bonuses.
	Real getShroudRevealToAllRange() const { return m_shroudRevealToAllRange; }

	// This function is only for use by the AIUpdateModuleData::parseLocomotorSet function.
	AIUpdateModuleData *friend_getAIModuleInfo();

	ShadowType getShadowType() const { return (ShadowType)m_shadowType; }
	Real getShadowSizeX() const { return m_shadowSizeX; }
	Real getShadowSizeY() const { return m_shadowSizeY; }
	Real getShadowOffsetX() const { return m_shadowOffsetX; }
	Real getShadowOffsetY() const { return m_shadowOffsetY; }

	const AsciiString& getShadowTextureName() const { return m_shadowTextureName; }
	UnsignedInt getOcclusionDelay() const { return m_occlusionDelay;}

	const ModuleInfo& getBehaviorModuleInfo() const { return m_behaviorModuleInfo; }
	const ModuleInfo& getDrawModuleInfo() const { return m_drawModuleInfo; }
	const ModuleInfo& getClientUpdateModuleInfo() const { return m_clientUpdateModuleInfo; }

	const Image *getSelectedPortraitImage() const { return m_selectedPortraitImage; }
	const Image *getButtonImage() const { return m_buttonImage; }

	//Code renderer handles these states now.
	//const AsciiString& getInventoryImageName( InventoryImageType type ) const { return m_inventoryImage[ type ]; }

	Int getSkillPointValue(Int level) const;

	Int getExperienceValue(Int level) const { return m_experienceValues[level]; }
	Int getExperienceRequired(Int level) const {return m_experienceRequired[level]; }
	Bool isTrainable() const{return m_isTrainable; }
	Bool isEnterGuard() const{return m_enterGuard; }
	Bool isHijackGuard() const{return m_hijackGuard; }

	const AudioEventRTS *getVoiceSelect() const								{ return getAudio(TTAUDIO_voiceSelect); }
	const AudioEventRTS *getVoiceGroupSelect() const					{ return getAudio(TTAUDIO_voiceGroupSelect); }
	const AudioEventRTS *getVoiceMove() const									{ return getAudio(TTAUDIO_voiceMove); }
	const AudioEventRTS *getVoiceAttack() const								{ return getAudio(TTAUDIO_voiceAttack); }
	const AudioEventRTS *getVoiceEnter() const								{ return getAudio(TTAUDIO_voiceEnter); }
	const AudioEventRTS *getVoiceFear() const									{ return getAudio(TTAUDIO_voiceFear); }
	const AudioEventRTS *getVoiceSelectElite() const					{ return getAudio(TTAUDIO_voiceSelectElite); }
	const AudioEventRTS *getVoiceCreated() const							{ return getAudio(TTAUDIO_voiceCreated); }
	const AudioEventRTS *getVoiceNearEnemy() const						{ return getAudio(TTAUDIO_voiceNearEnemy); }
	const AudioEventRTS *getVoiceTaskUnable() const						{ return getAudio(TTAUDIO_voiceTaskUnable); }
	const AudioEventRTS *getVoiceTaskComplete() const					{ return getAudio(TTAUDIO_voiceTaskComplete); }
	const AudioEventRTS *getVoiceMeetEnemy() const						{ return getAudio(TTAUDIO_voiceMeetEnemy); }
	const AudioEventRTS *getVoiceGarrison() const							{ return getAudio(TTAUDIO_voiceGarrison); }
#ifdef ALLOW_SURRENDER
	const AudioEventRTS *getVoiceSurrender() const						{ return getAudio(TTAUDIO_voiceSurrender); }
#endif
	const AudioEventRTS *getVoiceDefect() const								{ return getAudio(TTAUDIO_voiceDefect); }
	const AudioEventRTS *getVoiceAttackSpecial() const				{ return getAudio(TTAUDIO_voiceAttackSpecial); }
	const AudioEventRTS *getVoiceAttackAir() const						{ return getAudio(TTAUDIO_voiceAttackAir); }
	const AudioEventRTS *getVoiceGuard() const								{ return getAudio(TTAUDIO_voiceGuard); }
	const AudioEventRTS *getSoundMoveStart() const						{ return getAudio(TTAUDIO_soundMoveStart); }
	const AudioEventRTS *getSoundMoveStartDamaged() const			{ return getAudio(TTAUDIO_soundMoveStartDamaged); }
	const AudioEventRTS *getSoundMoveLoop() const							{ return getAudio(TTAUDIO_soundMoveLoop); }
	const AudioEventRTS *getSoundMoveLoopDamaged() const			{ return getAudio(TTAUDIO_soundMoveLoopDamaged); }
#if RTS_GENERALS && RETAIL_COMPATIBLE_DATA
	const AudioEventRTS *getSoundDie() const									{ return getAudio(TTAUDIO_soundDie); }
	const AudioEventRTS *getSoundDieFire() const							{ return getAudio(TTAUDIO_soundDieFire); }
	const AudioEventRTS *getSoundDieToxin() const							{ return getAudio(TTAUDIO_soundDieToxin); }
	const AudioEventRTS *getSoundCrush() const								{ return getAudio(TTAUDIO_soundCrush); }
#endif
	const AudioEventRTS *getSoundAmbient() const							{ return getAudio(TTAUDIO_soundAmbient); }
	const AudioEventRTS *getSoundAmbientDamaged() const				{ return getAudio(TTAUDIO_soundAmbientDamaged); }
	const AudioEventRTS *getSoundAmbientReallyDamaged() const	{ return getAudio(TTAUDIO_soundAmbientReallyDamaged); }
	const AudioEventRTS *getSoundAmbientRubble() const				{ return getAudio(TTAUDIO_soundAmbientRubble); }
	const AudioEventRTS *getSoundStealthOn() const						{ return getAudio(TTAUDIO_soundStealthOn); }
	const AudioEventRTS *getSoundStealthOff() const						{ return getAudio(TTAUDIO_soundStealthOff); }
	const AudioEventRTS *getSoundCreated() const							{ return getAudio(TTAUDIO_soundCreated); }
	const AudioEventRTS *getSoundOnDamaged() const						{ return getAudio(TTAUDIO_soundOnDamaged); }
	const AudioEventRTS *getSoundOnReallyDamaged() const			{ return getAudio(TTAUDIO_soundOnReallyDamaged); }
	const AudioEventRTS *getSoundEnter() const								{ return getAudio(TTAUDIO_soundEnter); }
	const AudioEventRTS *getSoundExit() const									{ return getAudio(TTAUDIO_soundExit); }
	const AudioEventRTS *getSoundPromotedVeteran() const			{ return getAudio(TTAUDIO_soundPromotedVeteran); }
	const AudioEventRTS *getSoundPromotedElite() const				{ return getAudio(TTAUDIO_soundPromotedElite); }
	const AudioEventRTS *getSoundPromotedHero() const					{ return getAudio(TTAUDIO_soundPromotedHero); }
	const AudioEventRTS *getSoundFalling() const							{ return getAudio(TTAUDIO_soundFalling); }

  Bool hasSoundAmbient() const                              { return hasAudio(TTAUDIO_soundAmbient); }

  const AudioEventRTS *getPerUnitSound(const AsciiString& soundName) const;
	const FXList* getPerUnitFX(const AsciiString& fxName) const;

	UnsignedInt getThreatValue() const								{ return m_threatValue; }

  //-------------------------------------------------------------------------------------------------
  /** If this is not NAMEKEY_INVALID, it indicates that all the templates which return the same name key
    * should be counted as the same "type" when looking at getMaxSimultaneousOfType(). For instance,
    * a Scud Storm and a Scud Storm rebuild hole will return the same value, so that the player
    * can't build another Scud Storm while waiting for the rebuild hole to start rebuilding */
  //-------------------------------------------------------------------------------------------------
  NameKeyType getMaxSimultaneousLinkKey() const { return m_maxSimultaneousLinkKey; }
  UnsignedInt getMaxSimultaneousOfType() const;

	void validate();

// The version that does not take an Object argument is labeled friend for use by WorldBuilder.  All game requests
// for CommandSet must use Object::getCommandSetString, as we have two different sources for dynamic answers.
	const AsciiString& friend_getCommandSetString() const { return m_commandSetString; }

	const std::vector<AsciiString>& getBuildVariations() const { return m_buildVariations; }

	Real getAssetScale() const { return m_assetScale; }						///< return uniform scaling
	Real getInstanceScaleFuzziness() const { return m_instanceScaleFuzziness; }						///< return uniform scaling
	Real getStructureRubbleHeight() const { return (Real)m_structureRubbleHeight; }						///< return uniform scaling

	/*
		NOTE: if you have a Thing, don't call this function; call Thing::getGeometryInfo instead, since
		geometry can now vary on a per-object basis. Only call this when you have no Thing around,
		and want to get info for the "prototype" (eg, for building new Things)...
	*/
	const GeometryInfo& getTemplateGeometryInfo() const { return m_geometryInfo; }

	//
	// these are intended ONLY for the private use of ThingFactory and do not use
	// the m_override pointer, it deals only with templates at the "top" level
	//
	void friend_setTemplateName( const AsciiString& name ) { m_nameString = name; }
	ThingTemplate *friend_getNextTemplate() const { return m_nextThingTemplate; }
	void friend_setNextTemplate(ThingTemplate *tmplate) { m_nextThingTemplate = tmplate; }
	void friend_setTemplateID(UnsignedShort id) { m_templateID = id; }

	Int getEnergyProduction() const { return m_energyProduction; }
	Int getEnergyBonus() const { return m_energyBonus; }

	// these are NOT publicly available; you should call calcCostToBuild() or calcTimeToBuild()
	// instead, because they will take player handicaps into account.
	// Int getBuildCost() const { return m_buildCost; }

	Int getRefundValue() const { return m_refundValue; }

	BuildCompletionType getBuildCompletion() const { return (BuildCompletionType)m_buildCompletion; }

	BuildableStatus getBuildable() const;

	Int getPrereqCount() const { return m_prereqInfo.size(); }
	const ProductionPrerequisite *getNthPrereq(Int i) const { return &m_prereqInfo[i]; }

	/**
		return the BuildFacilityTemplate, if any.

		if this template needs no build facility, null is returned.

		if the template needs a build facility but the given player doesn't have any in existence,
		null will be returned.

		if you pass null for player, we'll return the 'natural' build facility.
	*/
	const ThingTemplate *getBuildFacilityTemplate( const Player *player ) const;

	Bool isBuildableItem() const;

	/// calculate how long (in logic frames) it will take the given player to build this unit
	Int calcTimeToBuild( const Player* player) const;

	/// calculate how much money it will take the given player to build this unit
	Int calcCostToBuild( const Player* player) const;

	/// Used only by Skirmish AI. Everyone else should call calcCostToBuild.
	Int friend_getBuildCost() const { return m_buildCost; }

	const AsciiString& getDefaultOwningSide() const { return m_defaultOwningSide; }

	/// get us the table to parse the fields for thing templates
	const FieldParse* getFieldParse() const { return s_objectFieldParseTable; }
	const FieldParse* getReskinFieldParse() const { return s_objectReskinFieldParseTable; }

	Bool isBuildFacility() const { return m_isBuildFacility; }
	Real getPlacementViewAngle() const { return m_placementViewAngle; }

	Real getFactoryExitWidth() const { return m_factoryExitWidth; }
	Real getFactoryExtraBibWidth() const { return m_factoryExtraBibWidth; }

	void setCopiedFromDefault();

	void setReskinnedFrom(const ThingTemplate* tt) { DEBUG_ASSERTCRASH(m_reskinnedFrom == nullptr, ("should be null")); m_reskinnedFrom = tt; }

	Bool isPrerequisite() const { return m_isPrerequisite; }

	const WeaponTemplateSet* findWeaponTemplateSet(const WeaponSetFlags& t) const;
	const ArmorTemplateSet* findArmorTemplateSet(const ArmorSetFlags& t) const;

	// returns true iff we have at least one weaponset that contains a weapon.
	// returns false if we have no weaponsets, or they are all empty.
	Bool canPossiblyHaveAnyWeapon() const;

	Bool isEquivalentTo(const ThingTemplate* tt) const;

	UnsignedByte getCrushableLevel() const { return m_crushableLevel; }
	UnsignedByte getCrusherLevel() const { return m_crusherLevel; }

	AsciiString getUpgradeCameoName( Int n)const{ return m_upgradeCameoUpgradeNames[n];	}

	const WeaponTemplateSetVector& getWeaponTemplateSets() const {return m_weaponTemplateSets;}

protected:

	//
	// these are NOT publicly available; you should call calcCostToBuild() or calcTimeToBuild()
	// instead, because they will take player handicaps into account.
	//
	Int getBuildCost() const { return m_buildCost; }
	Real getBuildTime() const { return m_buildTime; }
	const PerUnitSoundMap* getAllPerUnitSounds() const { return &m_perUnitSounds; }
	void validateAudio();
	const AudioEventRTS* getAudio(ThingTemplateAudioType t) const { return m_audioarray.m_audio[t] ? m_audioarray.m_audio[t].Peek() : &s_audioEventNoSound; }
  Bool hasAudio(ThingTemplateAudioType t) const { return m_audioarray.m_audio[t] != nullptr; }

	// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
	/** Table for parsing the object fields */
	// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

	static void parseArmorTemplateSet( INI* ini, void *instance, void *store, const void* /*userData*/ );
	static void parseWeaponTemplateSet( INI* ini, void *instance, void *store, const void* /*userData*/ );
	static void parsePrerequisites( INI* ini, void *instance, void * /*store*/, const void* /*userData*/ );
	static void parseModuleName(INI* ini, void *instance, void* /*store*/, const void* userData);
	static void parseIntList(INI* ini, void *instance, void* store, const void* userData);

	static void parsePerUnitSounds(INI* ini, void *instance, void* store, const void* userData);
	static void parsePerUnitFX(INI* ini, void *instance, void* store, const void* userData);

	static void parseAddModule(INI *ini, void *instance, void *store, const void *userData);
	static void parseRemoveModule(INI *ini, void *instance, void *store, const void *userData);
	static void parseReplaceModule(INI *ini, void *instance, void *store, const void *userData);
	static void parseInheritableModule(INI *ini, void *instance, void *store, const void *userData);
  static void OverrideableByLikeKind(INI *ini, void *instance, void *store, const void *userData);

  static void parseMaxSimultaneous(INI *ini, void *instance, void *store, const void *userData);

	Bool removeModuleInfo(const AsciiString& moduleToRemove, AsciiString& clearedModuleNameOut);

private:
	static const FieldParse s_objectFieldParseTable[];		///< the parse table
	static const FieldParse s_objectReskinFieldParseTable[];		///< the parse table
	static AudioEventRTS s_audioEventNoSound;

private:

	// ---- Strings
	UnicodeString			m_displayName;			///< UI display for onscreen display
	AsciiString				m_nameString;					///< name of this thing template
	AsciiString				m_defaultOwningSide;	///< default owning side (owning player is inferred)
	AsciiString				m_commandSetString;
	AsciiString				m_selectedPortraitImageName;
	AsciiString				m_buttonImageName;
	AsciiString				m_upgradeCameoUpgradeNames[MAX_UPGRADE_CAMEO_UPGRADES];	///< Use these to find the upgrade images to display on the control bar
	AsciiString				m_shadowTextureName;					///< name of texture to use for shadow decal
	AsciiString				m_moduleBeingReplacedName;		///< used only during map.ini loading... name (not tag) of Module being replaced, or empty if not inside ReplaceModule block
	AsciiString				m_moduleBeingReplacedTag;			///< used only during map.ini loading... tag (not name) of Module being replaced, or empty if not inside ReplaceModule block
#ifdef LOAD_TEST_ASSETS
	AsciiString				m_LTAName;
#endif

	// ---- Misc Larger-than-int things
	GeometryInfo			m_geometryInfo;			///< geometry information
	KindOfMaskType		m_kindof;					///< kindof bits
	AudioArray				m_audioarray;
	ModuleInfo				m_behaviorModuleInfo;
	ModuleInfo				m_drawModuleInfo;
	ModuleInfo				m_clientUpdateModuleInfo;

	// ---- Misc Arrays-of-things
	Int											m_skillPointValues[LEVEL_COUNT];
	Int											m_experienceValues[LEVEL_COUNT];		///< How much I am worth at each experience level
	Int											m_experienceRequired[LEVEL_COUNT];	///< How many experience points I need for each level

	//Code renderer handles these states now.
	//AsciiString							m_inventoryImage[ INV_IMAGE_NUM_IMAGES ];  ///< portrait inventory pictures

	// TheSuperHackers @bugfix Caball009/xezon 06/07/2025 No longer copy SparseMatchFinder to prevent copied instances linking to unrelated data.
	// This avoids mismatching in certain maps, for example those that spawn units with veterancy.
	typedef SparseMatchFinder<WeaponTemplateSet, WeaponSetFlags, SparseMatchFinderFlags_NoCopy> WeaponTemplateSetFinder;
	typedef SparseMatchFinder<ArmorTemplateSet, ArmorSetFlags, SparseMatchFinderFlags_NoCopy> ArmorTemplateSetFinder;

	// ---- STL-sized things
	std::vector<ProductionPrerequisite>	m_prereqInfo;				///< the unit Prereqs for this tech
	std::vector<AsciiString>						m_buildVariations;	/**< if we build a unit of this type via script or ui, randomly choose one
																														of these templates instead. (doesn't apply to MapObject-created items) */
	WeaponTemplateSetVector							m_weaponTemplateSets;					///< our weaponsets
	WeaponTemplateSetFinder							m_weaponTemplateSetFinder;		///< helper to allow us to find the best sets, quickly
	ArmorTemplateSetVector							m_armorTemplateSets;	///< our armorsets
	ArmorTemplateSetFinder							m_armorTemplateSetFinder;		///< helper to allow us to find the best sets, quickly
	PerUnitSoundMap											m_perUnitSounds;					///< An additional set of sounds that only apply for this template.
	PerUnitFXMap												m_perUnitFX;									///< An additional set of fx that only apply for this template.

	// ---- Pointer-sized things
	ThingTemplate*				m_nextThingTemplate;
	const ThingTemplate*	m_reskinnedFrom;									///< non nullptr if we were generated via a reskin
	const Image *					m_selectedPortraitImage;		/// portrait image when selected (to display in GUI)
	const Image	*					m_buttonImage;

	// ---- Real-sized things
	Real					m_fenceWidth;								///< Fence width for fence type objects.
	Real					m_fenceXOffset;							///< Fence X offset for fence type objects.
	Real					m_visionRange;								///< object "sees" this far around itself
	Real					m_shroudClearingRange;				///< Since So many things got added to "Seeing" functionality, we need to split this part out.
	Real					m_shroudRevealToAllRange;			///< When > zero, the shroud gets revealed to all players.
	Real					m_placementViewAngle;				///< when placing buildings this will be the angle of the building when "floating" at the mouse
	Real					m_factoryExitWidth;					///< when placing buildings this will be the width of the reserved exit area on the right side.
	Real					m_factoryExtraBibWidth;					///< when placing buildings this will be the width of the reserved exit area on the right side.
	Real					m_buildTime;									///< Seconds to build
	Real					m_assetScale;
	Real					m_instanceScaleFuzziness; ///< scale randomization tolerance to init for each Drawable instance,
	Real					m_shadowSizeX;				///< world-space extent of decal shadow texture
	Real					m_shadowSizeY;				///< world-space extent of decal shadow texture
	Real					m_shadowOffsetX;			///< world-space offset of decal shadow texture
	Real					m_shadowOffsetY;			///< world-space offset of decal shadow texture

	// ---- Int-sized things
	Int						m_energyProduction;						///< how much Energy this takes (negative values produce Energy, rather than consuming it)
	Int						m_energyBonus;								///< how much extra Energy this produces due to the upgrade
	Color					m_displayColor;								///< for the editor display color
	UnsignedInt		m_occlusionDelay;							///< delay after object creation before building occlusion is allowed.
  NameKeyType   m_maxSimultaneousLinkKey;     ///< If this is not NAMEKEY_INVALID, it indicates that all the templates which have the same name key should be counted as the same "type" when looking at getMaxSimultaneousOfType().

	// ---- Short-sized things
	UnsignedShort		m_templateID;									///< id for net (etc.) transmission purposes
	UnsignedShort		m_buildCost;									///< money to build (0 == not buildable)
	UnsignedShort		m_refundValue;								///< custom resale value, if sold. (0 == use default)
	UnsignedShort		m_threatValue;								///< Threat map info
	UnsignedShort		m_maxSimultaneousOfType;			///< max simultaneous of this unit we can have (per player) at one time. (0 == unlimited)

	// ---- Bool-sized things
  Bool          m_maxSimultaneousDeterminedBySuperweaponRestriction; ///< If true, override value in m_maxSimultaneousOfType with value from GameInfo::getSuperweaponRestriction()
	Bool					m_isPrerequisite;							///< Is this thing considered in a prerequisite for any other thing?
	Bool					m_isBridge;										///< True if this model is a bridge.
 	Bool					m_isBuildFacility;						///< is this the build facility for something? (calculated based on other template's prereqs)
	Bool					m_isTrainable;								///< Whether or not I can even gain experience
	Bool          m_enterGuard;									///< Whether or not I can enter objects when guarding
	Bool          m_hijackGuard;								///< Whether or not I can hijack objects when guarding
	Bool					m_isForbidden;								///< useful when overriding in <mapfile>.ini
	Bool					m_armorCopiedFromDefault;
	Bool					m_weaponsCopiedFromDefault;

	// ---- Byte-sized things
	Byte					m_radarPriority;						///< does object appear on radar, and if so at what priority
	Byte					m_transportSlotCount;				///< how many "slots" we take in a transport (0 == not transportable)
	Byte					m_buildable;								///< is this thing buildable at all?
	Byte					m_buildCompletion;					///< how the units come into the world when build is complete
	Byte					m_editorSorting;						///< editor sorting type, see EditorSortingType enum
	Byte					m_structureRubbleHeight;
	Byte					m_shadowType;								///< settings which determine the type of shadow rendered
	Byte					m_moduleParsingMode;
	UnsignedByte	m_crusherLevel;							///< crusher > crushable level to actually crush
	UnsignedByte	m_crushableLevel;						///< Specifies the level of crushability (must be hit by a crusher greater than this to crush me).


};

//-----------------------------------------------------------------------------
//           Inlining
//-----------------------------------------------------------------------------

//-----------------------------------------------------------------------------
//           Externals
//-----------------------------------------------------------------------------
```

### `Core/GameEngine/Source/Common/Thing/ThingTemplate.cpp`
```cpp
// FILE: ThingTemplate.cpp ////////////////////////////////////////////////////////////////////////
// Created: Colin Day, April 2001
// Desc:    Thing templates
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#define DEFINE_POWER_NAMES								// for PowerNames[]
#define DEFINE_SHADOW_NAMES								// for TheShadowNames[]
#define DEFINE_GEOMETRY_NAMES							// for GeometryNames[]
#define DEFINE_BUILD_COMPLETION_NAMES			// for BuildCompletionNames[]
#define DEFINE_EDITOR_SORTING_NAMES				// for EditorSortingNames[]
#define DEFINE_RADAR_PRIORITY_NAMES				// for RadarPriorityNames[]
#define DEFINE_BUILDABLE_STATUS_NAMES			// for BuildableStatusNames[]

#include "Common/DamageFX.h"
#include "Common/GameAudio.h"
#include "Common/GameCommon.h"
#include "Common/GlobalData.h"
#include "Common/INI.h"
#include "Common/MessageStream.h"
#include "Common/Module.h"
#include "Common/ModuleFactory.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/ProductionPrerequisite.h"
#include "Common/Radar.h"
#include "Common/RandomValue.h"
#include "Common/Science.h"
#include "Common/ThingTemplate.h"
#include "Common/ThingFactory.h"
#include "Common/ThingSort.h"
#include "Common/BitFlagsIO.h"

#include "GameClient/Drawable.h"
#include "GameClient/FXList.h"
#include "GameClient/GameText.h"
#include "GameClient/Image.h"
#include "GameClient/Shadow.h"

#include "GameLogic/Armor.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/SpecialPowerModule.h"
#include "GameLogic/Object.h"
#include "GameLogic/Powers.h"
#include "GameLogic/Weapon.h"

#include "Common/UnitTimings.h" //Contains the DO_UNIT_TIMINGS define jba.


//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
const Int USE_EXP_VALUE_FOR_SKILL_VALUE = -999;


///////////////////////////////////////////////////////////////////////////////////////////////////
// PRIVATE DATA ///////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

AudioEventRTS ThingTemplate::s_audioEventNoSound;

static void parseKindOfFromINI(INI* ini, void* instance, void *store, const void* userData)
{
	KindOfMaskType::parseFromINI(ini, instance, store, userData);

#if RTS_GENERALS
	KindOfMaskType* kindOf = reinterpret_cast<KindOfMaskType*>(store);

	if (kindOf->test(KINDOF_AIRFIELD))
	{
		// KINDOF_AIRFIELD became KINDOF_FS_AIRFIELD in Zero Hour.
		kindOf->set(KINDOF_FS_AIRFIELD);
	}

	if (kindOf->test(KINDOF_DRONE))
	{
		// KINDOF_DRONE was implicitly KINDOF_NO_SELECT in Generals.
		kindOf->set(KINDOF_NO_SELECT);
	}
#endif
}

/*
	NOTE NOTE NOTE -- s_objectFieldParseTable and s_objectReskinFieldParseTable must be updated in tandem!

	"s_objectReskinFieldParseTable" is intended to be used for parsing the "ObjectReskin" keyword, and
	should only allow you to specify things that affect the appearance of a template.

	The idea is that some units are functionally the same, but different visually; rather than replicate
	all the settings, you can specify it as "this one is like that one, but looks different".

	Thus, currently, the only things in the reskin table are:

		-- DrawModules
		-- DrawModuleData
		-- Geometry

	So: if you add/remove/modify any settings that deal with visual appearance, you *may* want to
	add 'em to the reskin table... but do so VERY CAUTIOUSLY and with careful deliberation (and
	after checking around for other opinions).

*/

// NOTE NOTE NOTE -- s_objectFieldParseTable and s_objectReskinFieldParseTable must be updated in tandem -- see comment above
const FieldParse ThingTemplate::s_objectFieldParseTable[] =
{
	{ "DisplayName",					INI::parseAndTranslateLabel,					nullptr,								offsetof( ThingTemplate, m_displayName ) },
	{ "RadarPriority",				INI::parseByteSizedIndexList,					RadarPriorityNames, offsetof( ThingTemplate, m_radarPriority ) },
	{ "TransportSlotCount",		INI::parseUnsignedByte,								nullptr,		offsetof( ThingTemplate, m_transportSlotCount ) },
	{ "FenceWidth",						INI::parseReal,												nullptr,		offsetof( ThingTemplate, m_fenceWidth ) },
	{ "FenceXOffset",					INI::parseReal,												nullptr,		offsetof( ThingTemplate, m_fenceXOffset ) },
	{ "IsBridge",							INI::parseBool,												nullptr,		offsetof( ThingTemplate, m_isBridge ) },
	{ "ArmorSet",							ThingTemplate::parseArmorTemplateSet, nullptr, 0},
	{ "WeaponSet",						ThingTemplate::parseWeaponTemplateSet,nullptr, 0},
	{ "VisionRange",					INI::parseReal,												nullptr,		offsetof( ThingTemplate, m_visionRange ) },
	{ "ShroudClearingRange",	INI::parseReal,												nullptr,		offsetof( ThingTemplate, m_shroudClearingRange ) },
	{ "ShroudRevealToAllRange",	INI::parseReal,											nullptr,		offsetof( ThingTemplate, m_shroudRevealToAllRange ) }, // Added in Zero Hour

	{ "PlacementViewAngle",		INI::parseAngleReal,									nullptr,		offsetof( ThingTemplate, m_placementViewAngle ) },

	{ "FactoryExitWidth",			INI::parseReal,												nullptr,		offsetof( ThingTemplate, m_factoryExitWidth ) },
	{ "FactoryExtraBibWidth",	INI::parseReal,												nullptr,		offsetof( ThingTemplate, m_factoryExtraBibWidth ) },

	{ "SkillPointValue",			ThingTemplate::parseIntList,					(void*)LEVEL_COUNT,		offsetof( ThingTemplate, m_skillPointValues ) },
	{ "ExperienceValue",			ThingTemplate::parseIntList,					(void*)LEVEL_COUNT,		offsetof( ThingTemplate, m_experienceValues ) },
	{ "ExperienceRequired",		ThingTemplate::parseIntList,					(void*)LEVEL_COUNT,		offsetof( ThingTemplate, m_experienceRequired ) },
	{ "IsTrainable",					INI::parseBool,												nullptr,									offsetof( ThingTemplate, m_isTrainable ) },
	{ "EnterGuard",						INI::parseBool,												nullptr,									offsetof( ThingTemplate, m_enterGuard ) }, // Added in Zero Hour
	{ "HijackGuard",					INI::parseBool,												nullptr,									offsetof( ThingTemplate, m_hijackGuard ) }, // Added in Zero Hour

	{ "Side",									INI::parseAsciiString,								nullptr,	offsetof( ThingTemplate, m_defaultOwningSide ) },

// NOTE NOTE NOTE -- s_objectFieldParseTable and s_objectReskinFieldParseTable must be updated in tandem -- see comment above
	{ "Prerequisites",				ThingTemplate::parsePrerequisites,	nullptr, 0 },
	{ "Buildable",						INI::parseByteSizedIndexList,				BuildableStatusNames, offsetof( ThingTemplate, m_buildable) },
	{ "BuildCost",						INI::parseUnsignedShort,						nullptr,		offsetof( ThingTemplate, m_buildCost ) },
	{ "BuildTime",						INI::parseReal,											nullptr,		offsetof( ThingTemplate, m_buildTime ) },
	{ "RefundValue",					INI::parseUnsignedShort,						nullptr,   offsetof( ThingTemplate, m_refundValue ) },
	{ "BuildCompletion",			INI::parseByteSizedIndexList,				BuildCompletionNames,		offsetof( ThingTemplate, m_buildCompletion ) },
	{ "EnergyProduction",			INI::parseInt,											nullptr,   offsetof( ThingTemplate, m_energyProduction ) },
	{ "EnergyBonus",					INI::parseInt,											nullptr,   offsetof( ThingTemplate, m_energyBonus ) },
	{ "IsForbidden",					INI::parseBool,											nullptr,		offsetof( ThingTemplate, m_isForbidden ) },
	{ "IsPrerequisite",				INI::parseBool,											nullptr,		offsetof( ThingTemplate, m_isPrerequisite ) },
	{ "DisplayColor",					INI::parseColorInt,									nullptr,		offsetof( ThingTemplate, m_displayColor ) },
	{ "EditorSorting",				INI::parseByteSizedIndexList,				EditorSortingNames, offsetof( ThingTemplate, m_editorSorting ) },
	{ "KindOf",								parseKindOfFromINI,									nullptr,		offsetof( ThingTemplate, m_kindof ) },
	{ "CommandSet",						INI::parseAsciiString,							nullptr,		offsetof( ThingTemplate, m_commandSetString ) },
	{ "BuildVariations",			INI::parseAsciiStringVector,				nullptr,		offsetof( ThingTemplate, m_buildVariations ) },

// NOTE NOTE NOTE -- s_objectFieldParseTable and s_objectReskinFieldParseTable must be updated in tandem -- see comment above
	{ "Behavior",							ThingTemplate::parseModuleName,		(const void*)MODULETYPE_BEHAVIOR, offsetof(ThingTemplate, m_behaviorModuleInfo) },
	{ "Body",									ThingTemplate::parseModuleName,		(const void*)999, offsetof(ThingTemplate, m_behaviorModuleInfo) },
	{ "Draw",									ThingTemplate::parseModuleName,		(const void*)MODULETYPE_DRAW, offsetof(ThingTemplate, m_drawModuleInfo) },
	{ "ClientUpdate",					ThingTemplate::parseModuleName,		(const void*)MODULETYPE_CLIENT_UPDATE, offsetof(ThingTemplate, m_clientUpdateModuleInfo) },
// NOTE NOTE NOTE -- s_objectFieldParseTable and s_objectReskinFieldParseTable must be updated in tandem -- see comment above

	{ "SelectPortrait",					INI::parseAsciiString,	nullptr,		offsetof( ThingTemplate, m_selectedPortraitImageName ) },
	{ "ButtonImage",						INI::parseAsciiString,	nullptr,		offsetof( ThingTemplate, m_buttonImageName ) },

	//Code renderer handles these states now.
	//{ "InventoryImageEnabled",	INI::parseAsciiString,	nullptr,		offsetof( ThingTemplate, m_inventoryImage[ INV_IMAGE_ENABLED ] ) },
	//{ "InventoryImageDisabled",	INI::parseAsciiString,	nullptr,		offsetof( ThingTemplate, m_inventoryImage[ INV_IMAGE_DISABLED ] ) },
	//{ "InventoryImageHilite",		INI::parseAsciiString,	nullptr,		offsetof( ThingTemplate, m_inventoryImage[ INV_IMAGE_HILITE ] ) },
	//{ "InventoryImagePushed",		INI::parseAsciiString,	nullptr,		offsetof( ThingTemplate, m_inventoryImage[ INV_IMAGE_PUSHED ] ) },

	{ "UpgradeCameo1",		INI::parseAsciiString,	nullptr,		offsetof( ThingTemplate, m_upgradeCameoUpgradeNames[ 0 ] ) },
	{ "UpgradeCameo2",		INI::parseAsciiString,	nullptr,		offsetof( ThingTemplate, m_upgradeCameoUpgradeNames[ 1 ] ) },
	{ "UpgradeCameo3",		INI::parseAsciiString,	nullptr,		offsetof( ThingTemplate, m_upgradeCameoUpgradeNames[ 2 ] ) },
	{ "UpgradeCameo4",		INI::parseAsciiString,	nullptr,		offsetof( ThingTemplate, m_upgradeCameoUpgradeNames[ 3 ] ) },
	{ "UpgradeCameo5",		INI::parseAsciiString,	nullptr,		offsetof( ThingTemplate, m_upgradeCameoUpgradeNames[ 4 ] ) },

// NOTE NOTE NOTE -- s_objectFieldParseTable and s_objectReskinFieldParseTable must be updated in tandem -- see comment above

	{ "VoiceSelect",					INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_voiceSelect]) },
	{ "VoiceGroupSelect",			INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_voiceGroupSelect]) },
	{ "VoiceMove",						INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_voiceMove]) },
	{ "VoiceAttack",					INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_voiceAttack]) },
	{ "VoiceEnter",						INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_voiceEnter ]) },
	{ "VoiceFear",						INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_voiceFear ]) },
	{ "VoiceSelectElite",			INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_voiceSelectElite ]) },
	{ "VoiceCreated",					INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_voiceCreated]) },
	{ "VoiceTaskUnable",			INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_voiceTaskUnable ]) },
	{ "VoiceTaskComplete",		INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_voiceTaskComplete ]) },
	{ "VoiceMeetEnemy",				INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_voiceMeetEnemy]) },
	{ "VoiceGarrison",				INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_voiceGarrison]) },
#ifdef ALLOW_SURRENDER
	{ "VoiceSurrender",				INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_voiceSurrender]) },
#endif
	{ "VoiceDefect",					INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_voiceDefect]) },
	{ "VoiceAttackSpecial",		INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_voiceAttackSpecial ]) },
	{ "VoiceAttackAir",				INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_voiceAttackAir ]) },
	{ "VoiceGuard",						INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_voiceGuard ]) },
	{ "SoundMoveStart",				INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_soundMoveStart]) },
	{ "SoundMoveStartDamaged",INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_soundMoveStartDamaged]) },
	{ "SoundMoveLoop",				INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_soundMoveLoop]) },
	{ "SoundMoveLoopDamaged",	INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_soundMoveLoopDamaged]) },
#if RTS_GENERALS && RETAIL_COMPATIBLE_DATA
	{ "SoundDie",							INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_soundDie]) },
	{ "SoundDieFire",					INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_soundDieFire ]) },
	{ "SoundDieToxin",				INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_soundDieToxin ]) },
	{ "SoundCrush",						INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_soundCrush ]) },
#endif
	{ "SoundAmbient",					INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_soundAmbient ]) },
	{ "SoundAmbientDamaged",	INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_soundAmbientDamaged ]) },
	{ "SoundAmbientReallyDamaged",INI::parseDynamicAudioEventRTS,	nullptr,offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_soundAmbientReallyDamaged ]) },
	{ "SoundAmbientRubble",		INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_soundAmbientRubble]) },
	{ "SoundStealthOn",       INI::parseDynamicAudioEventRTS,  nullptr,  offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_soundStealthOn ]) },
	{ "SoundStealthOff",      INI::parseDynamicAudioEventRTS,  nullptr,  offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_soundStealthOff ]) },
	{ "SoundCreated",					INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_soundCreated ]) },
	{ "SoundOnDamaged",				INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_soundOnDamaged ]) },
	{ "SoundOnReallyDamaged",	INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_soundOnReallyDamaged ]) },
	{ "SoundEnter",						INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_soundEnter ]) },
	{ "SoundExit",						INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_soundExit ]) },
	{ "SoundPromotedVeteran",	INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_soundPromotedVeteran ]) },
	{ "SoundPromotedElite",		INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_soundPromotedElite ]) },
	{ "SoundPromotedHero",		INI::parseDynamicAudioEventRTS,	nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_soundPromotedHero ]) },
	{ "SoundFallingFromPlane",INI::parseDynamicAudioEventRTS, nullptr,		offsetof( ThingTemplate, m_audioarray.m_audio[TTAUDIO_soundFalling ]) },

	{ "UnitSpecificSounds",		ThingTemplate::parsePerUnitSounds, nullptr, offsetof(ThingTemplate, m_perUnitSounds) },
	{ "UnitSpecificFX",				ThingTemplate::parsePerUnitFX, nullptr, offsetof(ThingTemplate, m_perUnitFX) },
	{ "Scale",								INI::parseReal,						nullptr,		offsetof( ThingTemplate, m_assetScale ) },
	{ "Geometry",							GeometryInfo::parseGeometryType,				nullptr,  offsetof( ThingTemplate, m_geometryInfo ) },
	{ "GeometryMajorRadius",	GeometryInfo::parseGeometryMajorRadius,	nullptr,		offsetof( ThingTemplate, m_geometryInfo ) },
	{ "GeometryMinorRadius",	GeometryInfo::parseGeometryMinorRadius,	nullptr,		offsetof( ThingTemplate, m_geometryInfo ) },
	{ "GeometryHeight",				GeometryInfo::parseGeometryHeight,			nullptr,		offsetof( ThingTemplate, m_geometryInfo ) },
	{ "GeometryIsSmall",			GeometryInfo::parseGeometryIsSmall,			nullptr,		offsetof( ThingTemplate, m_geometryInfo ) },
	{ "Shadow",								INI::parseBitString8,		TheShadowNames,		offsetof( ThingTemplate, m_shadowType ) },
	{ "ShadowSizeX",					INI::parseReal,						nullptr,	offsetof( ThingTemplate, m_shadowSizeX ) },
	{ "ShadowSizeY",					INI::parseReal,						nullptr,	offsetof( ThingTemplate, m_shadowSizeY ) },
	{ "ShadowOffsetX",				INI::parseReal,						nullptr,	offsetof( ThingTemplate, m_shadowOffsetX ) },
	{ "ShadowOffsetY",				INI::parseReal,						nullptr,	offsetof( ThingTemplate, m_shadowOffsetY ) },
	{ "ShadowTexture",				INI::parseAsciiString,		nullptr,	offsetof( ThingTemplate, m_shadowTextureName ) },
	{ "OcclusionDelay",					INI::parseDurationUnsignedInt,		nullptr, offsetof( ThingTemplate, m_occlusionDelay ) },
	{ "AddModule",						ThingTemplate::parseAddModule,			nullptr, 0 },
	{ "RemoveModule",					ThingTemplate::parseRemoveModule,		nullptr, 0 },
	{ "ReplaceModule",				ThingTemplate::parseReplaceModule,	nullptr, 0 },
	{ "InheritableModule",		ThingTemplate::parseInheritableModule,	nullptr, 0 },

  { "OverrideableByLikeKind",		ThingTemplate::OverrideableByLikeKind,	nullptr, 0 }, // Added in Zero Hour

	{ "Locomotor",						AIUpdateModuleData::parseLocomotorSet, nullptr, 0 },
	{ "InstanceScaleFuzziness",	INI::parseReal,					nullptr, offsetof(ThingTemplate, m_instanceScaleFuzziness ) },
	{ "StructureRubbleHeight",	INI::parseUnsignedByte,					nullptr, offsetof(ThingTemplate, m_structureRubbleHeight ) },
	{ "ThreatValue",						INI::parseUnsignedShort,		nullptr, offsetof(ThingTemplate, m_threatValue ) },
  { "MaxSimultaneousOfType",	ThingTemplate::parseMaxSimultaneous,		nullptr, offsetof(ThingTemplate, m_maxSimultaneousOfType ) },
  { "MaxSimultaneousLinkKey",	NameKeyGenerator::parseStringAsNameKeyType,		nullptr, offsetof(ThingTemplate, m_maxSimultaneousLinkKey ) }, // Added in Zero Hour
	{ "CrusherLevel",					INI::parseUnsignedByte,			nullptr, offsetof( ThingTemplate, m_crusherLevel ) },
	{ "CrushableLevel",				INI::parseUnsignedByte,			nullptr, offsetof( ThingTemplate, m_crushableLevel ) },

	{ nullptr, nullptr, nullptr, 0 }

};
// NOTE NOTE NOTE -- s_objectFieldParseTable and s_objectReskinFieldParseTable must be updated in tandem -- see comment above

// NOTE NOTE NOTE -- s_objectFieldParseTable and s_objectReskinFieldParseTable must be updated in tandem -- see comment above
const FieldParse ThingTemplate::s_objectReskinFieldParseTable[] =
{
	{ "Draw",									ThingTemplate::parseModuleName,		(const void*)MODULETYPE_DRAW, offsetof(ThingTemplate, m_drawModuleInfo) },

	{ "Geometry",							GeometryInfo::parseGeometryType,				nullptr,  offsetof( ThingTemplate, m_geometryInfo ) },
	{ "GeometryMajorRadius",	GeometryInfo::parseGeometryMajorRadius,	nullptr,		offsetof( ThingTemplate, m_geometryInfo ) },
	{ "GeometryMinorRadius",	GeometryInfo::parseGeometryMinorRadius,	nullptr,		offsetof( ThingTemplate, m_geometryInfo ) },
	{ "GeometryHeight",				GeometryInfo::parseGeometryHeight,			nullptr,		offsetof( ThingTemplate, m_geometryInfo ) },
	{ "GeometryIsSmall",			GeometryInfo::parseGeometryIsSmall,			nullptr,		offsetof( ThingTemplate, m_geometryInfo ) },
	{ "FenceWidth",						INI::parseReal,													nullptr,		offsetof( ThingTemplate, m_fenceWidth ) },
	{ "FenceXOffset",					INI::parseReal,													nullptr,		offsetof( ThingTemplate, m_fenceXOffset ) },

  // Needed to avoid some cheats with the scud storm rebuild hole
  { "MaxSimultaneousOfType",	ThingTemplate::parseMaxSimultaneous,		nullptr, offsetof(ThingTemplate, m_maxSimultaneousOfType ) }, // Added in Zero Hour
  { "MaxSimultaneousLinkKey",	NameKeyGenerator::parseStringAsNameKeyType,		nullptr, offsetof(ThingTemplate, m_maxSimultaneousLinkKey ) }, // Added in Zero Hour

	{ nullptr, nullptr, nullptr, 0 }

};
// NOTE NOTE NOTE -- s_objectFieldParseTable and s_objectReskinFieldParseTable must be updated in tandem -- see comment above

// ------------------------------------------------------------------------------------------------
/** See if the tag string is present in any of the module info entries here */
// ------------------------------------------------------------------------------------------------
const ModuleInfo::Nugget *ModuleInfo::getNuggetWithTag( const AsciiString& tag ) const
{

	std::vector< Nugget >::const_iterator it;
	for( it = m_info.begin(); it != m_info.end(); ++it )
		if( (*it).m_moduleTag == tag )
			return &(*it);

	// no match
	return nullptr;

}

// ------------------------------------------------------------------------------------------------
/** Add this module info to the thing template */
// ------------------------------------------------------------------------------------------------
void ModuleInfo::addModuleInfo(ThingTemplate *thingTemplate,
															 const AsciiString& name,
															 const AsciiString& moduleTag,
															 const ModuleData* data,
															 Int interfaceMask,
															 Bool inheritable,
                               Bool overrideableByLikeKind)
{

	//
	// there must be a module tag present, and it must be unique across all module infos
	// for this thing template
	//
#if defined(RTS_DEBUG)
	// get module info
	const Nugget *nugget;

	nugget = thingTemplate->getBehaviorModuleInfo().getNuggetWithTag( moduleTag );
	if( nugget != nullptr )
	{

		// compare this nugget tag against the tag for the new data we're going to submit
		DEBUG_ASSERTCRASH( nugget->m_moduleTag != moduleTag,
											 ("addModuleInfo - ERROR defining module '%s' on thing template '%s'.  The module '%s' has the tag '%s' which must be unique among all modules for this object, but the tag '%s' is also already on module '%s' within this object.\n\nPlease make unique tag names within an object definition",
												name.str(),
												thingTemplate->getName().str(),
												name.str(),
												moduleTag.str(),
												moduleTag.str(),
												nugget->first.str()) );

		// srj sez: prevent people from ignoring this.
		throw INI_INVALID_DATA;
	}

	nugget = thingTemplate->getDrawModuleInfo().getNuggetWithTag( moduleTag );
	if( nugget != nullptr )
	{

		// compare this nugget tag against the tag for the new data we're going to submit
		DEBUG_ASSERTCRASH( nugget->m_moduleTag != moduleTag,
											 ("addModuleInfo - ERROR defining module '%s' on thing template '%s'.  The module '%s' has the tag '%s' which must be unique among all modules for this object, but the tag '%s' is also already on module '%s' within this object.\n\nPlease make unique tag names within an object definition",
												name.str(),
												thingTemplate->getName().str(),
												name.str(),
												moduleTag.str(),
												moduleTag.str(),
												nugget->first.str()) );

		// srj sez: prevent people from ignoring this.
		throw INI_INVALID_DATA;
	}

	nugget = thingTemplate->getClientUpdateModuleInfo().getNuggetWithTag( moduleTag );
	if( nugget != nullptr )
	{

		// compare this nugget tag against the tag for the new data we're going to submit
		DEBUG_ASSERTCRASH( nugget->m_moduleTag != moduleTag,
											 ("addModuleInfo - ERROR defining module '%s' on thing template '%s'.  The module '%s' has the tag '%s' which must be unique among all modules for this object, but the tag '%s' is also already on module '%s' within this object.\n\nPlease make unique tag names within an object definition",
												name.str(),
												thingTemplate->getName().str(),
												name.str(),
												moduleTag.str(),
												moduleTag.str(),
												nugget->first.str()) );
		// srj sez: prevent people from ignoring this.
		throw INI_INVALID_DATA;
	}

#endif

	m_info.push_back(Nugget(name, moduleTag, data, interfaceMask, inheritable, overrideableByLikeKind));

}

//-------------------------------------------------------------------------------------------------
Bool ModuleInfo::clearModuleDataWithTag(const AsciiString& tagToClear, AsciiString& clearedModuleNameOut)
{
	Bool cleared = false;

	// do NOT clear... we only want to modify this if we return true.
	// if we return false, we should leave this unmodified.
	//clearedModuleNameOut.clear();

	for (std::vector<Nugget>::iterator it = m_info.begin(); it != m_info.end(); /* empty */ )
	{
		if (it->m_moduleTag == tagToClear)
		{
			DEBUG_ASSERTCRASH(!cleared, ("Hmm, multiple clears in ModuleInfo::clearModuleDataWithTag, should this be possible?"));
			clearedModuleNameOut = it->first;
			it = m_info.erase(it);
			cleared = true;
		}
		else
		{
			++it;
		}
	}
	return cleared;
}





//-------------------------------------------------------------------------------------------------
Bool ModuleInfo::clearCopiedFromDefaultEntries(Int interfaceMask, const AsciiString &newName, const ThingTemplate *fullTemplate )
{
  static KindOfMaskType ImmuneToGPSScramblerMask;
  KindOfMaskType &m = ImmuneToGPSScramblerMask;
  m.set(KINDOF_AIRCRAFT);// NO PLANES or helicopters
  m.set(KINDOF_SHRUBBERY);// NO trees or bushes
  m.set(KINDOF_OPTIMIZED_TREE);
  m.set(KINDOF_STRUCTURE);// NO buildings
  m.set(KINDOF_DRAWABLE_ONLY);
  m.set(KINDOF_MOB_NEXUS);
  m.set(KINDOF_IGNORED_IN_GUI);
  m.set(KINDOF_CLEARED_BY_BUILD);
  m.set(KINDOF_DEFENSIVE_WALL);
  m.set(KINDOF_BALLISTIC_MISSILE);
  m.set(KINDOF_SUPPLY_SOURCE);
  m.set(KINDOF_BOAT);
  m.set(KINDOF_INERT);
  m.set(KINDOF_BRIDGE);
  m.set(KINDOF_LANDMARK_BRIDGE);
  m.set(KINDOF_BRIDGE_TOWER);
  Bool disallowed =  fullTemplate->isAnyKindOf( ImmuneToGPSScramblerMask );

  static KindOfMaskType CandidateForGPSScramblerMask;
  CandidateForGPSScramblerMask.set(KINDOF_SCORE);
  CandidateForGPSScramblerMask.set(KINDOF_VEHICLE);
  CandidateForGPSScramblerMask.set(KINDOF_INFANTRY);
  CandidateForGPSScramblerMask.set(KINDOF_PORTABLE_STRUCTURE);
  Bool candidate =  fullTemplate->isAnyKindOf( CandidateForGPSScramblerMask );

  Bool ret = false;

	std::vector<Nugget>::iterator it = m_info.begin();
	while( it != m_info.end() )
	{
		if( (it->interfaceMask & interfaceMask) != 0 && it->copiedFromDefault )
		{
      if ( it->inheritable )
			{
#if !(RTS_GENERALS && RETAIL_COMPATIBLE_CRC)
				if( it->m_moduleTag.compare("ModuleTag_DefaultAutoHealBehavior") == 0  && !fullTemplate->isTrainable() )
				{
					// Don't inherit this module if it is entirely useless to us.
          it = m_info.erase( it );
			    ret = true;
				}
				else
#endif
				{
					++it;//skip to the next nugget, 'cause we inherit this one
				}
			}
      else if ( it->overrideableByLikeKind)
      {

        AsciiString oldName = it->first;
        if ( oldName == newName ||  //we will dump this instance, since the INI author requested a specific one of the same class
             disallowed ||  // or, we just do not Add these special overrideables to these kinds of templates, so just dump it
             candidate == FALSE )
        {
          it = m_info.erase( it );
			    ret = true;
        }
        else
			    ++it;//no match, preserve the default instance of this Module for now
      }
      else // just dump this instance of this Module, since one of the same interface mask has been added by caller
      {
        it = m_info.erase( it );
			  ret = true;
      }
    }
    else
			++it;
	}
	return ret;
}








//-------------------------------------------------------------------------------------------------
Bool ModuleInfo::clearAiModuleInfo()
{
	Bool ret = false;

	std::vector<Nugget>::iterator it = m_info.begin();
	while( it != m_info.end() )
	{
		if (it->second->isAiModuleData() )
		{
			it = m_info.erase( it );
			ret = true;
		}
		else
		{
			++it;
		}
	}
	return ret;
}

//-------------------------------------------------------------------------------------------------
void ThingTemplate::parseModuleName(INI* ini, void *instance, void* store, const void* userData)
{
	ThingTemplate* self = (ThingTemplate*)instance;
	ModuleInfo* mi = (ModuleInfo*)store;
	ModuleType type = (ModuleType)(uintptr_t)userData;
	const char* token = ini->getNextToken();
	AsciiString tokenStr = token;

	// get the tag string (it is now required)
	AsciiString moduleTagStr;
	try
	{
		moduleTagStr = ini->getNextToken();
	}
	catch( ... )
	{

		DEBUG_CRASH(( "[LINE: %d - FILE: '%s'] Module tag not found for module '%s' on thing template '%s'.  Module tags are required and must be unique for all modules within an object definition",
									ini->getLineNum(), ini->getFilename().str(),
									tokenStr.str(), self->getName().str() ));
		throw;

	}

	Int interfaceMask;

	// ugh -- special case for "Body".
	if (type == 999)
	{
		type = MODULETYPE_BEHAVIOR;
	// what interface(s) does this module support?
		interfaceMask = TheModuleFactory->findModuleInterfaceMask(tokenStr, type);
		if ((interfaceMask & (MODULEINTERFACE_BODY)) == 0)
		{
			DEBUG_CRASH(("Only Body allowed here"));
			throw INI_INVALID_DATA;
		}
	}
	else
	{
		interfaceMask = TheModuleFactory->findModuleInterfaceMask(tokenStr, type);
		if ((interfaceMask & (MODULEINTERFACE_BODY)) != 0)
		{
			DEBUG_CRASH(("No Body allowed here"));
			throw INI_INVALID_DATA;
		}
	}

	// if we're overriding, we can totally skip over this block
	if (ini->getLoadType() == INI_LOAD_CREATE_OVERRIDES)
	{
		if (self->m_moduleParsingMode == MODULEPARSE_ADD_REMOVE_REPLACE)
		{
			// do nothing, just fall thru
		}
		else
		{
			DEBUG_CRASH(("[LINE: %d - FILE: '%s'] You must use AddModule to add modules in override INI files.",
				ini->getLineNum(), ini->getFilename().str(), self->getName().str()));
			throw INI_INVALID_DATA;
		}
	}
	else
	{

//    if (self->getName().compare("GLAVehicleQuadCannon"))
//      DEBUG_CRASH( ("WE ARE CLEARING DEFAULT MODULES FROM A QUAD CANNON.") );

		self->m_behaviorModuleInfo.clearCopiedFromDefaultEntries(interfaceMask, tokenStr, self );
		self->m_drawModuleInfo.clearCopiedFromDefaultEntries(interfaceMask, tokenStr, self );
		self->m_clientUpdateModuleInfo.clearCopiedFromDefaultEntries(interfaceMask, tokenStr, self );
	}

	if (self->m_moduleParsingMode == MODULEPARSE_ADD_REMOVE_REPLACE &&
			self->m_moduleBeingReplacedName.isNotEmpty() &&
			self->m_moduleBeingReplacedName != tokenStr)
	{
		DEBUG_CRASH(("[LINE: %d - FILE: '%s'] ReplaceModule must replace modules with another module of the same type, but you are attempting to replace a %s with a %s for Object %s.",
			ini->getLineNum(), ini->getFilename().str(), self->m_moduleBeingReplacedName.str(), tokenStr.str(), self->getName().str()));
		throw INI_INVALID_DATA;
	}

	if (self->m_moduleParsingMode == MODULEPARSE_ADD_REMOVE_REPLACE &&
			self->m_moduleBeingReplacedTag.isNotEmpty() &&
			self->m_moduleBeingReplacedTag == moduleTagStr)
	{
		DEBUG_CRASH(("[LINE: %d - FILE: '%s'] ReplaceModule must specify a new, unique tag for the replaced module, but you are not doing so for %s (%s) for Object %s.",
			ini->getLineNum(), ini->getFilename().str(), moduleTagStr.str(), self->m_moduleBeingReplacedName.str(), self->getName().str()));
		throw INI_INVALID_DATA;
	}

	ModuleData* data = TheModuleFactory->newModuleDataFromINI(ini, tokenStr, type, moduleTagStr);

	if (data->isAiModuleData())
	{
		Bool replaced = mi->clearAiModuleInfo();
		if (replaced)
		{
			//Kris: Commented this out for SPAM reasons. Do we really need this?
			//DEBUG_LOG(("replaced an AI for %s!",self->getName().str()));
		}
	}

	Bool inheritable = (self->m_moduleParsingMode == MODULEPARSE_INHERITABLE);
  Bool overrideableByLikeKind = (self->m_moduleParsingMode == MODULEPARSE_OVERRIDEABLE_BY_LIKE_KIND);
	mi->addModuleInfo(self, tokenStr, moduleTagStr, data, interfaceMask, inheritable, overrideableByLikeKind);
}

//-------------------------------------------------------------------------------------------------
void ThingTemplate::parseIntList(INI* ini, void *instance, void* store, const void* userData)
{
	Int numberEntries = (Int)(intptr_t)userData;
	Int *intList = (Int*)store;

	for( Int intIndex = 0; intIndex < numberEntries; intIndex ++ )
	{
		const char *token = ini->getNextToken();
		intList[intIndex] = ini->scanInt(token);
	}
}

//-------------------------------------------------------------------------------------------------
static void parsePrerequisiteUnit( INI* ini, void *instance, void * /*store*/, const void* /*userData*/ )
{
	std::vector<ProductionPrerequisite>* v = (std::vector<ProductionPrerequisite>*)instance;

	ProductionPrerequisite prereq;
	Bool orUnitWithPrevious = FALSE;
	for (const char *token = ini->getNextToken(); token; token = ini->getNextTokenOrNull())
	{
		prereq.addUnitPrereq( AsciiString( token ), orUnitWithPrevious );
		orUnitWithPrevious = TRUE;
	}

	v->push_back(prereq);
}

//-------------------------------------------------------------------------------------------------
static void parsePrerequisiteScience( INI* ini, void *instance, void * /*store*/, const void* /*userData*/ )
{
	std::vector<ProductionPrerequisite>* v = (std::vector<ProductionPrerequisite>*)instance;

	ProductionPrerequisite prereq;
	prereq.addSciencePrereq(INI::scanScience(ini->getNextToken()));

	v->push_back(prereq);
}

//-------------------------------------------------------------------------------------------------
void ThingTemplate::parsePrerequisites( INI* ini, void *instance, void *store, const void* userData )
{
	ThingTemplate* self = (ThingTemplate*)instance;

	static const FieldParse myFieldParse[] =
	{
		{ "Object", parsePrerequisiteUnit, nullptr, 0 },
		{ "Science", parsePrerequisiteScience,	nullptr, 0 },
		{ nullptr, nullptr, nullptr, 0 }
	};

	if (ini->getLoadType() == INI_LOAD_CREATE_OVERRIDES)
	{
		self->m_prereqInfo.clear();
	}

	ini->initFromINI(&self->m_prereqInfo, myFieldParse);
}

//-------------------------------------------------------------------------------------------Static
static void parseArbitraryFXIntoMap( INI* ini, void *instance, void* /* store */, const void* userData )
{
	PerUnitFXMap* mapFX = (PerUnitFXMap*)instance;
	const char* name = (const char*)userData;
	const char* token = ini->getNextToken();
	const FXList* fxl = TheFXListStore->findFXList(token);	// could be null!
	DEBUG_ASSERTCRASH(fxl != nullptr || stricmp(token, "None") == 0, ("FXList %s not found!",token));
	mapFX->insert(std::make_pair(AsciiString(name), fxl));
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void ThingTemplate::parsePerUnitFX( INI* ini, void *instance, void *store, const void *userData )
{
	PerUnitFXMap* fxmap = (PerUnitFXMap*)store;

	fxmap->clear();

	static const FieldParse myFieldParse[] =
	{
		{ nullptr, parseArbitraryFXIntoMap, nullptr, 0 },
		{ nullptr, nullptr, nullptr, 0 }
	};

	ini->initFromINI(fxmap, myFieldParse);
}

//-------------------------------------------------------------------------------------------Static
static void parseArbitrarySoundsIntoMap( INI* ini, void *instance, void* /* store */, const void* userData )
{
	PerUnitSoundMap *mapSounds = (PerUnitSoundMap*) instance;
	const char* name = (const char*)userData;
	const char* token = ini->getNextToken();

	AudioEventRTS a;
	if (token)
		a.setEventName(token);
	mapSounds->insert(std::make_pair(AsciiString(name), a));
}

//-------------------------------------------------------------------------------------------------
/** Parse Additional per unit sounds such as TankTurretMove and TankTurretMoveLoop. */
//-------------------------------------------------------------------------------------------------
void ThingTemplate::parsePerUnitSounds( INI* ini, void *instance, void *store, const void *userData )
{
	PerUnitSoundMap *mapSounds = (PerUnitSoundMap*)store;
	mapSounds->clear();

	static const FieldParse myFieldParse[] =
	{
		{ nullptr, parseArbitrarySoundsIntoMap, nullptr, 0 },
		{ nullptr, nullptr, nullptr, 0 }
	};

	ini->initFromINI(mapSounds, myFieldParse);
}

//-------------------------------------------------------------------------------------------------
/** Parse modules to add to the existing set of modules. */
//-------------------------------------------------------------------------------------------------
void ThingTemplate::parseAddModule(INI *ini, void *instance, void *store, const void *userData)
{
	// don't care about the result.
	ThingTemplate* self = (ThingTemplate*)instance;

	ModuleParseMode oldMode = (ModuleParseMode)self->m_moduleParsingMode;
	if (oldMode != MODULEPARSE_NORMAL)
		throw INI_INVALID_DATA;

	self->m_moduleParsingMode = MODULEPARSE_ADD_REMOVE_REPLACE;

	ini->initFromINI(self, self->getFieldParse());

	self->m_moduleParsingMode = oldMode;
}

//-------------------------------------------------------------------------------------------------
/** Parse modules to remove from the existing set of modules. */
//-------------------------------------------------------------------------------------------------
void ThingTemplate::parseRemoveModule(INI *ini, void *instance, void *store, const void *userData)
{
	ThingTemplate* self = (ThingTemplate*)instance;

	ModuleParseMode oldMode = (ModuleParseMode)self->m_moduleParsingMode;
	if (oldMode != MODULEPARSE_NORMAL)
		throw INI_INVALID_DATA;

	self->m_moduleParsingMode = MODULEPARSE_ADD_REMOVE_REPLACE;

	const char *modToRemove = ini->getNextToken();
	AsciiString removedModuleName;
	Bool removed = self->removeModuleInfo(modToRemove, removedModuleName);
	if (!removed)
	{
		DEBUG_ASSERTCRASH(removed, ("RemoveModule %s was not found for %s. The game will crash now!",modToRemove, self->getName().str()));
		throw INI_INVALID_DATA;
	}

	self->m_moduleParsingMode = oldMode;
}

//-------------------------------------------------------------------------------------------------
/** Replace the existing tagged modules with the new modules. */
//-------------------------------------------------------------------------------------------------
void ThingTemplate::parseReplaceModule(INI *ini, void *instance, void *store, const void *userData)
{
	ThingTemplate* self = (ThingTemplate*)instance;

	ModuleParseMode oldMode = (ModuleParseMode)self->m_moduleParsingMode;
	if (oldMode != MODULEPARSE_NORMAL)
		throw INI_INVALID_DATA;

	self->m_moduleParsingMode = MODULEPARSE_ADD_REMOVE_REPLACE;

	const char *modToRemove = ini->getNextToken();
	AsciiString removedModuleName;
	Bool removed = self->removeModuleInfo(modToRemove, removedModuleName);
	if (!removed)
	{
		DEBUG_CRASH(("[LINE: %d - FILE: '%s'] ReplaceModule %s was not found for %s; cannot continue.",
															ini->getLineNum(), ini->getFilename().str(), modToRemove, self->getName().str()));
		throw INI_INVALID_DATA;
	}

	self->m_moduleBeingReplacedName = removedModuleName;
	self->m_moduleBeingReplacedTag = modToRemove;
	ini->initFromINI(self, self->getFieldParse());
	self->m_moduleBeingReplacedName.clear();
	self->m_moduleBeingReplacedTag.clear();

	self->m_moduleParsingMode = oldMode;
}

//-------------------------------------------------------------------------------------------------
/** mark the module(s) as being "Inheritable". */
//-------------------------------------------------------------------------------------------------
void ThingTemplate::parseInheritableModule(INI *ini, void *instance, void *store, const void *userData)
{
	ThingTemplate* self = (ThingTemplate*)instance;

	ModuleParseMode oldMode = (ModuleParseMode)self->m_moduleParsingMode;
	if (oldMode != MODULEPARSE_NORMAL)
		throw INI_INVALID_DATA;

	self->m_moduleParsingMode = MODULEPARSE_INHERITABLE;

	ini->initFromINI(self, self->getFieldParse());

	self->m_moduleParsingMode = oldMode;
}


//-------------------------------------------------------------------------------------------------
/** mark the module(s) as being "VverrideableByLikeKind". default module will be replaced by any of the exact same class */
//-------------------------------------------------------------------------------------------------
void ThingTemplate::OverrideableByLikeKind(INI *ini, void *instance, void *store, const void *userData)
{
	ThingTemplate* self = (ThingTemplate*)instance;

	ModuleParseMode oldMode = (ModuleParseMode)self->m_moduleParsingMode;
	if (oldMode != MODULEPARSE_NORMAL)
		throw INI_INVALID_DATA;

	self->m_moduleParsingMode = MODULEPARSE_OVERRIDEABLE_BY_LIKE_KIND;

	ini->initFromINI(self, self->getFieldParse());

	self->m_moduleParsingMode = oldMode;
}






//-------------------------------------------------------------------------------------------------
/** Remove the module whose tag matches moduleToRemove. */
//-------------------------------------------------------------------------------------------------
Bool ThingTemplate::removeModuleInfo(const AsciiString& moduleToRemove, AsciiString& clearedModuleNameOut)
{
	Bool removed = false;

	// do NOT clear... we only want to modify this if we return true.
	// if we return false, we should leave this unmodified.
	//clearedModuleNameOut.clear();

	if (m_behaviorModuleInfo.clearModuleDataWithTag(moduleToRemove, clearedModuleNameOut))
	{
		DEBUG_ASSERTCRASH(!removed, ("Hmm, multiple removed in ThingTemplate::removeModuleInfo, should this be possible?"));
		removed = true;
	}
	if (m_drawModuleInfo.clearModuleDataWithTag(moduleToRemove, clearedModuleNameOut))
	{
		DEBUG_ASSERTCRASH(!removed, ("Hmm, multiple removed in ThingTemplate::removeModuleInfo, should this be possible?"));
		removed = true;
	}
	if (m_clientUpdateModuleInfo.clearModuleDataWithTag(moduleToRemove, clearedModuleNameOut))
	{
		DEBUG_ASSERTCRASH(!removed, ("Hmm, multiple removed in ThingTemplate::removeModuleInfo, should this be possible?"));
		removed = true;
	}

	return removed;
}

//-------------------------------------------------------------------------------------------------
/// @todo srj -- move this to another file
void ArmorTemplateSet::parseArmorTemplateSet( INI* ini )
{
	static const FieldParse myFieldParse[] =
	{
		{ "Conditions", ArmorSetFlags::parseFromINI, nullptr, offsetof( ArmorTemplateSet, m_types ) },
		{ "Armor", INI::parseArmorTemplate,	nullptr, offsetof( ArmorTemplateSet, m_template ) },
		{ "DamageFX",	INI::parseDamageFX,	nullptr, offsetof( ArmorTemplateSet, m_fx ) },
		{ nullptr, nullptr, nullptr, 0 }
	};

	ini->initFromINI(this, myFieldParse);
}

//-------------------------------------------------------------------------------------------------
void ThingTemplate::parseArmorTemplateSet( INI* ini, void *instance, void * /*store*/, const void* /*userData*/ )
{
	ThingTemplate* self = (ThingTemplate*)instance;
	if (self->m_armorCopiedFromDefault == TRUE)
	{
		self->m_armorCopiedFromDefault = FALSE;
		self->m_armorTemplateSets.clear();
	}

	ArmorTemplateSet ws;
	ws.parseArmorTemplateSet(ini);
#if defined(RTS_DEBUG)
	if (ini->getLoadType() != INI_LOAD_CREATE_OVERRIDES)
	{
		for (ArmorTemplateSetVector::const_iterator it = self->m_armorTemplateSets.begin(); it != self->m_armorTemplateSets.end(); ++it)
		{
			if (it->getNthConditionsYes(0) == ws.getNthConditionsYes(0))
			{
				DEBUG_CRASH(("dup armorset condition in %s",self->getName().str()));
			}
		}
	}
#endif
	self->m_armorTemplateSets.push_back(ws);
	self->m_armorTemplateSetFinder.clear();
}

//-------------------------------------------------------------------------------------------------
void ThingTemplate::parseWeaponTemplateSet( INI* ini, void *instance, void * /*store*/, const void* /*userData*/ )
{
	ThingTemplate* self = (ThingTemplate*)instance;
	if (self->m_weaponsCopiedFromDefault == TRUE)
	{
		self->m_weaponsCopiedFromDefault = FALSE;
		self->m_weaponTemplateSets.clear();
	}

	WeaponTemplateSet ws;
	ws.parseWeaponTemplateSet(ini, self);
#if defined(RTS_DEBUG)
	if (ini->getLoadType() != INI_LOAD_CREATE_OVERRIDES)
	{
		for (WeaponTemplateSetVector::const_iterator it = self->m_weaponTemplateSets.begin(); it != self->m_weaponTemplateSets.end(); ++it)
		{
			if (it->getNthConditionsYes(0) == ws.getNthConditionsYes(0))
			{
				DEBUG_CRASH(("dup weaponset condition in %s",self->getName().str()));
			}
		}
	}
#endif
	self->m_weaponTemplateSets.push_back(ws);
	self->m_weaponTemplateSetFinder.clear();
}

//-------------------------------------------------------------------------------------------------
// Parse the "maxSimultaneousOfType" keyword
void ThingTemplate::parseMaxSimultaneous(INI *ini, void *instance, void *store, const void *userData)
{
  // Most of the time, this is an UnsignedShort, but sometimes this is the keyword
  // "DeterminedBySuperweaponRestriction"
  const char DETERMINED_BY_SUPERWEAPON_KEYWORD[] = "DeterminedBySuperweaponRestriction";

  ThingTemplate *myTemplate = (ThingTemplate *)instance;
  DEBUG_ASSERTCRASH ( &myTemplate->m_maxSimultaneousOfType == store, ("Bad store passed to parseMaxSimultaneous" ) );

  const char * token = ini->getNextToken();
  if ( stricmp( token, DETERMINED_BY_SUPERWEAPON_KEYWORD ) == 0 )
  {
    myTemplate->m_maxSimultaneousDeterminedBySuperweaponRestriction = true;
    *(UnsignedShort *)store = 0;
  }
  else
  {
    // Copied from parseUnsignedShort
    Int value = INI::scanInt(token);
    if (value < 0 || value > 65535)
    {
      DEBUG_CRASH(("Bad value parseMaxSimultaneous"));
      throw ERROR_BUG;
    }
    *(UnsignedShort *)store = (UnsignedShort)value;
    myTemplate->m_maxSimultaneousDeterminedBySuperweaponRestriction = false;
  }
}


//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ThingTemplate::ThingTemplate() :
	m_geometryInfo(GEOMETRY_SPHERE, FALSE, 1, 1, 1)
{
	m_moduleParsingMode = MODULEPARSE_NORMAL;
	m_reskinnedFrom = nullptr;
	m_radarPriority = RADAR_PRIORITY_INVALID;

	m_nextThingTemplate = nullptr;
	m_transportSlotCount = 0;
	m_fenceWidth = 0;
	m_fenceXOffset = 0;
	m_visionRange = 0.0f;
	m_shroudClearingRange = -1.0f;
	m_shroudRevealToAllRange = -1.0f;

	m_buildCost = 0;
	m_buildTime = 1;
	m_refundValue = 0;
	m_energyProduction = 0;
	m_energyBonus = 0;
	m_buildCompletion = BC_APPEARS_AT_RALLY_POINT;

	for( Int levelIndex = 0; levelIndex < LEVEL_COUNT; levelIndex++ )
	{
		m_experienceValues[levelIndex] = 0;
		m_experienceRequired[levelIndex] = 0;
		// -1 means "same value as experienceValues for that level"
		m_skillPointValues[levelIndex] = USE_EXP_VALUE_FOR_SKILL_VALUE;
	}
	m_isTrainable = FALSE;
	m_enterGuard = FALSE;
	m_hijackGuard = FALSE;

	m_templateID = 0;
	m_kindof = KINDOFMASK_NONE;
	//m_defaultOwningSide = "";	// unnecessary
	m_isBuildFacility = FALSE;
	m_isPrerequisite = FALSE;
	m_placementViewAngle = 0.0f;
	m_factoryExitWidth = 0.0f;
	m_factoryExtraBibWidth = 0.0f;

	m_selectedPortraitImage = nullptr;
	m_buttonImage = nullptr;

	m_shadowType = SHADOW_NONE;
	m_shadowSizeX = 0.0f;
	m_shadowSizeY = 0.0f;
	m_shadowOffsetX = 0.0f;
	m_shadowOffsetY = 0.0f;
	m_occlusionDelay = TheGlobalData->m_defaultOcclusionDelay;

	m_structureRubbleHeight = 0;
	m_instanceScaleFuzziness = 0;
	m_threatValue = 0;
	m_maxSimultaneousOfType = 0;	// unlimited
  m_maxSimultaneousLinkKey = NAMEKEY_INVALID; // Not linked
  m_maxSimultaneousDeterminedBySuperweaponRestriction = false;
	m_crusherLevel = 0;			//Unspecified, this object is unable to crush anything!
	m_crushableLevel = 255; //Unspecified, this object is unable to be crushed by anything!

}

//-------------------------------------------------------------------------------------------------
AIUpdateModuleData *ThingTemplate::friend_getAIModuleInfo()
{
	Int numModInfos = m_behaviorModuleInfo.getCount();
	for (int j = 0; j < numModInfos; ++j)
	{
		if (m_behaviorModuleInfo.getNthData(j) && m_behaviorModuleInfo.getNthData(j)->isAiModuleData())
		{
			return (AIUpdateModuleData *)m_behaviorModuleInfo.friend_getNthData(j);
		}
	}

	return nullptr;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void ThingTemplate::validateAudio()
{
#if defined(RTS_DEBUG)

	#define AUDIO_TEST(y) \
		if (!get##y()->getEventName().isEmpty() && get##y()->getEventName().compareNoCase("NoSound") != 0) { \
			DEBUG_ASSERTLOG(TheAudio->isValidAudioEvent(get##y()), ("Invalid Sound '%s' in Object '%s'. (%s?)", #y, getName().str(), get##y()->getEventName().str())); \
		}

	AUDIO_TEST(VoiceSelect)
	AUDIO_TEST(VoiceGroupSelect)
	AUDIO_TEST(VoiceMove)
	AUDIO_TEST(VoiceAttack)
	AUDIO_TEST(VoiceEnter)
	AUDIO_TEST(VoiceFear)
	AUDIO_TEST(VoiceSelectElite)
	AUDIO_TEST(VoiceCreated)
	AUDIO_TEST(VoiceNearEnemy)
	AUDIO_TEST(VoiceTaskUnable)
	AUDIO_TEST(VoiceTaskComplete)
	AUDIO_TEST(VoiceMeetEnemy)
	AUDIO_TEST(VoiceGarrison)
#ifdef ALLOW_SURRENDER
	AUDIO_TEST(VoiceSurrender)
#endif
	AUDIO_TEST(VoiceDefect)
	AUDIO_TEST(VoiceAttackSpecial)
	AUDIO_TEST(VoiceAttackAir)
	AUDIO_TEST(VoiceGuard)

	AUDIO_TEST(SoundMoveStart)
	AUDIO_TEST(SoundMoveStartDamaged)
	AUDIO_TEST(SoundMoveLoop)
	AUDIO_TEST(SoundMoveLoopDamaged)
#if RTS_GENERALS && RETAIL_COMPATIBLE_DATA
	AUDIO_TEST(SoundDie)
	AUDIO_TEST(SoundDieFire)
	AUDIO_TEST(SoundDieToxin)
	AUDIO_TEST(SoundCrush)
#endif
	AUDIO_TEST(SoundAmbient)
	AUDIO_TEST(SoundAmbientDamaged)
	AUDIO_TEST(SoundAmbientReallyDamaged)
	AUDIO_TEST(SoundAmbientRubble)
	AUDIO_TEST(SoundStealthOn)
	AUDIO_TEST(SoundStealthOff)
	AUDIO_TEST(SoundCreated)

	AUDIO_TEST(SoundOnDamaged)
	AUDIO_TEST(SoundOnReallyDamaged)
	AUDIO_TEST(SoundEnter)
	AUDIO_TEST(SoundExit)
	AUDIO_TEST(SoundPromotedVeteran)
	AUDIO_TEST(SoundPromotedElite)
	AUDIO_TEST(SoundPromotedHero)

	#undef AUDIO_TEST

	const PerUnitSoundMap *perUnitSounds = getAllPerUnitSounds();
	if (!perUnitSounds)
	{
		return;
	}

	for (PerUnitSoundMap::const_iterator it = perUnitSounds->begin(); it != perUnitSounds->end(); ++it)
	{
		if (!it->second.getEventName().isEmpty() && it->second.getEventName().compareNoCase("NoSound") != 0)
		{
			DEBUG_ASSERTCRASH(TheAudio->isValidAudioEvent(&it->second),
												("Invalid UnitSpecificSound '%s' in Object '%s'. (%s?)",
												it->first.str(),
												getName().str(),
												it->second.getEventName().str()));
		}
	}
#endif
}

//-------------------------------------------------------------------------------------------------
void ThingTemplate::validate()
{
	if (m_shadowTextureName.isEmpty())
	{
		// no texture given, pick a default
		switch (getTemplateGeometryInfo().getGeomType())
		{
			case GEOMETRY_SPHERE:
			case GEOMETRY_CYLINDER:
				m_shadowTextureName = "shadow";
				break;
			case GEOMETRY_BOX:
				m_shadowTextureName = "shadows";
				break;
		}
	}

	validateAudio();

#if defined(RTS_DEBUG)

	if (getName() == "DefaultThingTemplate")
		return;

	// neutron missile is an old special case....
	if (getName() == "NeutronMissile")
		return;

	// another cheesy special case....
	if (getName() == "FlamethrowerProjectileStream")
		return;

	// drawable-only templates are exempt from further checks.
	if (isKindOf(KINDOF_DRAWABLE_ONLY))
		return;

	// build-variation templates are exempt from further checks.
	if (!m_buildVariations.empty())
		return;

	Bool isImmobile = isKindOf(KINDOF_IMMOBILE);

	if (isKindOf(KINDOF_SHRUBBERY) && !isImmobile)
	{
		DEBUG_CRASH(("SHRUBBERY %s must be marked IMMOBILE!",getName().str()));
	}

	if (isKindOf(KINDOF_STRUCTURE) && !isImmobile)
	{
		DEBUG_CRASH(("Structure %s is not marked immobile, but probably should be -- please fix it. (If we ever add mobile structures, this debug sniffer will need to be revised.)",getName().str()));
	}

	if (isKindOf(KINDOF_STICK_TO_TERRAIN_SLOPE) && !isImmobile)
	{
		DEBUG_CRASH(("item %s is marked STICK_TO_TERRAIN_SLOPE but not IMMOBILE -- please fix it.",getName().str()));
	}

	if (isKindOf(KINDOF_STRUCTURE))
	{
		if (m_armorTemplateSets.empty() || (m_armorTemplateSets.size() == 1 && m_armorTemplateSets[0].getArmorTemplate() == nullptr))
		{
			DEBUG_CRASH(("Structure %s has no armor, but probably should (StructureArmor) -- please fix it.)",getName().str()));
		}
		for (ArmorTemplateSetVector::const_iterator it = m_armorTemplateSets.begin(); it != m_armorTemplateSets.end(); ++it)
		{
			if (it->getDamageFX() == nullptr)
			{
				DEBUG_CRASH(("Structure %s has no ArmorDamageFX, and really should.",getName().str()));
			}
		}
	}

#endif
}

//-------------------------------------------------------------------------------------------------
// copy the guts of that into this, but preserve this' name, id, and list-links.
void ThingTemplate::copyFrom(const ThingTemplate* that)
{
	if (!that)
		return;

	ThingTemplate* next = this->m_nextThingTemplate;
	UnsignedShort id = this->m_templateID;
	AsciiString name = this->m_nameString;

	*this = *that;

	this->m_nextThingTemplate = next;
	this->m_templateID = id;
	this->m_nameString = name;
}

//-------------------------------------------------------------------------------------------------
void ThingTemplate::setCopiedFromDefault()
{
	m_armorCopiedFromDefault = true;
	m_weaponsCopiedFromDefault = true;
	m_behaviorModuleInfo.setCopiedFromDefault(true);
	m_drawModuleInfo.setCopiedFromDefault(true);
	m_clientUpdateModuleInfo.setCopiedFromDefault(true);
}

//-------------------------------------------------------------------------------------------------
ThingTemplate::~ThingTemplate()
{
	// note, we don't need to take any special action for Armor/WeaponSets...
	// though it is just a list of 'raw' pointers, we don't have ownership of 'em,
	// and so we MUST NOT delete them
}

//=============================================================================
void ThingTemplate::resolveNames()
{
	Int i, j;

	//Kris: July 31, 2003
	//NOTE: Make sure that all code in this function supports caching properly. For example,
	//      templates can be partially overridden by map.ini files. When this happens, strings
	//      that have been parsed are looked up, cached, then cleared. The problem is if a string
	//      gets cached, but not overridden, it will be clear the next time we call this function.
	//      so we will want to make sure we don't NULL out cached data if the string is empty. A
	//      concrete example is overriding an object with prerequisites. We just override the portrait.
	//      So the 1st time we call this function, we get the standard template data. During this first
	//      call, the strings are looked up, cached, and cleared. Then we override the portrait in the
	//      map.ini. The next time we call this function, we look up all the strings again. The prereq
	//      names didn't used to check for empty strings so they would NULL out all the previous prereqs
	//      the object had. So be sure to make sure all string lookups don't blindly lookup things -- check
	//      if the string isNotEmpty first!

	for (i = 0; i < m_prereqInfo.size(); i++)
	{
		m_prereqInfo[i].resolveNames();
	}

	const Int MAX_BF = 32;
	const ThingTemplate* tmpls[MAX_BF];
	for (i = 0; i < m_prereqInfo.size(); i++)
	{
		Int count = m_prereqInfo[i].getAllPossibleBuildFacilityTemplates(tmpls, MAX_BF);
		for (j = 0; j < count; j++)
		{
			// casting const away is a little evil, but justified in this case:
			// PropductionPrerequisite should only be allowed 'const' access,
			// but ThingTemplate can muck with stuff with gleeful abandon. (srj)
			if( tmpls[ j ] )
				const_cast<ThingTemplate*>(tmpls[j])->m_isBuildFacility = true;
			// DEBUG_LOG(("BF: %s is a buildfacility for %s",tmpls[j]->m_nameString.str(),this->m_nameString.str()));
		}
	}

	if (isKindOf(KINDOF_COMMANDCENTER)) {
		// Command centers are considered factories. jba.
		m_isBuildFacility = true;
	}

	// keep a pointer to portrait and button image if present for speed later
	if( TheMappedImageCollection )
	{
		if( m_selectedPortraitImageName.isNotEmpty() )
		{
			m_selectedPortraitImage = TheMappedImageCollection->findImageByName( m_selectedPortraitImageName );
			DEBUG_ASSERTCRASH( m_selectedPortraitImage, ("%s is looking for Portrait %s but can't find it. Skipping...", getName().str(), m_buttonImageName.str() ) );
			m_selectedPortraitImageName.clear();	// we're done with this, so nuke it
		}
		if( m_buttonImageName.isNotEmpty() )
		{
			m_buttonImage = TheMappedImageCollection->findImageByName( m_buttonImageName );
			DEBUG_ASSERTCRASH( m_buttonImage, ("%s is looking for ButtonImage %s but can't find it. Skipping...", getName().str(), m_buttonImageName.str() ) );
			m_buttonImageName.clear();	// we're done with this, so nuke it
		}
	}

}

//=============================================================================
#ifdef LOAD_TEST_ASSETS
void ThingTemplate::initForLTA(const AsciiString& name)
{
	m_nameString = name;

	char buffer[1024];
	strlcpy(buffer, name.str(), sizeof(buffer));
	int i=0;
	for (; buffer[i]; i++) {
		if (buffer[i] == '/') {
			i++;
			break;
		}
	}
	m_LTAName = AsciiString(buffer+i);

	m_behaviorModuleInfo.clear();
	m_drawModuleInfo.clear();
	m_clientUpdateModuleInfo.clear();

	AsciiString moduleTag;

	moduleTag.format( "LTA_%sDestroyDie", m_LTAName.str() );
	m_behaviorModuleInfo.addModuleInfo(this, "DestroyDie", moduleTag, TheModuleFactory->newModuleDataFromINI(nullptr, "DestroyDie", MODULETYPE_BEHAVIOR, moduleTag), (MODULEINTERFACE_DIE), false);

	moduleTag.format( "LTA_%sInactiveBody", m_LTAName.str() );
	m_behaviorModuleInfo.addModuleInfo(this, "InactiveBody", moduleTag, TheModuleFactory->newModuleDataFromINI(nullptr, "InactiveBody", MODULETYPE_BEHAVIOR, moduleTag), (MODULEINTERFACE_BODY), false);

	moduleTag.format( "LTA_%sW3DDefaultDraw", m_LTAName.str() );
	m_drawModuleInfo.addModuleInfo(this, "W3DDefaultDraw", moduleTag, TheModuleFactory->newModuleDataFromINI(nullptr, "W3DDefaultDraw", MODULETYPE_DRAW, moduleTag), (MODULEINTERFACE_DRAW), false);

	m_armorCopiedFromDefault = false;
	m_weaponsCopiedFromDefault = false;

	m_kindof = KINDOFMASK_NONE;
	m_assetScale = 1.0f;
	m_instanceScaleFuzziness = 0.0f;	///< tolerance to randomly vary scale per instance
	m_structureRubbleHeight = 0.0f;		// zero means "use global default"
	m_displayName.translate( name );
	m_shadowType = SHADOW_VOLUME;

	m_geometryInfo.set(GEOMETRY_SPHERE, false, 10.0, 10.0, 10.0);

}
#endif


//=============================================================================
const ArmorTemplateSet* ThingTemplate::findArmorTemplateSet(const ArmorSetFlags& t) const
{
  return m_armorTemplateSetFinder.findBestInfo(m_armorTemplateSets, t);
}

//=============================================================================
const WeaponTemplateSet* ThingTemplate::findWeaponTemplateSet(const WeaponSetFlags& t) const
{
  return m_weaponTemplateSetFinder.findBestInfo(m_weaponTemplateSets, t);
}

//-----------------------------------------------------------------------------
// returns true iff we have at least one weaponset that contains a weapon.
// returns false if we have no weaponsets, or they are all empty.
Bool ThingTemplate::canPossiblyHaveAnyWeapon() const
{
	for (WeaponTemplateSetVector::const_iterator it = m_weaponTemplateSets.begin();
					it != m_weaponTemplateSets.end();
					++it)
	{
		if (it->hasAnyWeapons())
			return true;
	}
	return false;
}

//-----------------------------------------------------------------------------
Int ThingTemplate::getSkillPointValue(Int level) const
{
	Int value = m_skillPointValues[level];

	if (value == USE_EXP_VALUE_FOR_SKILL_VALUE)
		value = getExperienceValue(level);

	return value;
}

//-----------------------------------------------------------------------------
const ThingTemplate *ThingTemplate::getBuildFacilityTemplate( const Player *player ) const
{
	if (getPrereqCount() > 0)
	{
		return m_prereqInfo[0].getExistingBuildFacilityTemplate(player);	// might return null
	}
	else
	{
		return nullptr;
	}
}

//-------------------------------------------------------------------------------------------------
BuildableStatus ThingTemplate::getBuildable() const
{
	BuildableStatus bs;
	if (TheGameLogic && TheGameLogic->findBuildableStatusOverride(this, bs))
		return bs;

	return (BuildableStatus)m_buildable;
}

//-------------------------------------------------------------------------------------------------
const FXList *ThingTemplate::getPerUnitFX(const AsciiString& fxName) const
{
	if (fxName.isEmpty())
	{
		return nullptr;
	}

	PerUnitFXMap::const_iterator it = m_perUnitFX.find(fxName);
	if (it == m_perUnitFX.end())
	{
		DEBUG_CRASH(("Unknown FX name (%s) asked for in ThingTemplate (%s)", fxName.str(), m_nameString.str()));
		return nullptr;
	}

	return (it->second);
}

//-------------------------------------------------------------------------------------------------
const AudioEventRTS *ThingTemplate::getPerUnitSound(const AsciiString& soundName) const
{
	if (soundName.isEmpty())
	{
		return &s_audioEventNoSound;
	}

	PerUnitSoundMap::const_iterator it = m_perUnitSounds.find(soundName);
	if (it == m_perUnitSounds.end())
	{
#ifndef DO_UNIT_TIMINGS
    DEBUG_LOG(("Unknown Audio name (%s) asked for in ThingTemplate (%s).", soundName.str(), m_nameString.str()));
#endif
    return &s_audioEventNoSound;
	}

	return &(it->second);
}

//-------------------------------------------------------------------------------------------------
UnsignedInt ThingTemplate::getMaxSimultaneousOfType() const
{
  if ( m_maxSimultaneousDeterminedBySuperweaponRestriction && TheGameLogic )
  {
    return TheGameLogic->getSuperweaponRestriction();
  }

  return m_maxSimultaneousOfType;
}




//-------------------------------------------------------------------------------------------------
Bool ThingTemplate::isEquivalentTo(const ThingTemplate* tt) const
{
	// sanity
	if (!tt)
		return false;

	// sanity
	if (this == tt)
		return true;

	if (this->getFinalOverride() == tt->getFinalOverride())
		return true;

	// This reskinned from that?
	if (this->m_reskinnedFrom == tt)
		return true;

	// That reskinned from this?
	if (this == tt->m_reskinnedFrom)
		return true;

	// This reskinned from that reskinned from?
	// Kris: added case (chassis 2 compared to chassis 3 -- nullptr possible if not reskinned)
	if( this->m_reskinnedFrom && this->m_reskinnedFrom == tt->m_reskinnedFrom )
		return true;

	// Is this thing a build variation of that thing or vice versa
	Int i;

	Int numVariations = m_buildVariations.size();
	for (i = 0; i < numVariations; ++i)
		if (m_buildVariations[i].compareNoCase(tt->getName()) == 0)
			return true;

	numVariations = tt->m_buildVariations.size();
	for (i = 0; i < numVariations; ++i)
		if (tt->m_buildVariations[i].compareNoCase(getName()) == 0)
			return true;

	// Guess we're not equivalent.
	return false;
}

//-------------------------------------------------------------------------------------------------
Bool ThingTemplate::isBuildableItem() const
{
	return (getBuildCost() != 0);
}

//-------------------------------------------------------------------------------------------------
/** NOTE that we're not paying attention to m_override here, instead the portions
	* that retrieve template data values use the get() wrappers, which *DO* pay
	* attention to the override values */
//-------------------------------------------------------------------------------------------------
Int ThingTemplate::calcCostToBuild( const Player* player) const
{
	if (!player)
		return 0;

	// changePercent format is "-.2 equals 20% cheaper"
	Real factionModifier = 1 + player->getProductionCostChangePercent( getName() );
	factionModifier *= player->getProductionCostChangeBasedOnKindOf( m_kindof );
	return getBuildCost() * factionModifier * player->getHandicap()->getHandicap(Handicap::BUILDCOST, this);
}

//-------------------------------------------------------------------------------------------------
/** NOTE that we're not paying attention to m_override here, instead the portions
	* that retrieve template data values use the get() wrappers, which *DO* pay
	* attention to the override values */
//-------------------------------------------------------------------------------------------------
Int ThingTemplate::calcTimeToBuild( const Player* player) const
{
	Int buildTime = getBuildTime() * static_cast<float>(LOGICFRAMES_PER_SECOND);
	buildTime *= player->getHandicap()->getHandicap(Handicap::BUILDTIME, this);

	Real factionModifier = 1 + player->getProductionTimeChangePercent( getName() );
	buildTime *= factionModifier;

#if defined(RTS_DEBUG) || defined(_ALLOW_DEBUG_CHEATS_IN_RELEASE)
	if( player->buildsInstantly() )
	{
		buildTime = 1;
	}
#endif

	// Adjust build time based on energy supply.

	Real EnergyPercent = player->getEnergy()->getEnergySupplyRatio();	//I'm at 80% Energy
	if (EnergyPercent > 1.0f)
		EnergyPercent = 1.0f;	// getEnergySupplyRatio() returns a true ratio, but we don't care about excess.
	Real EnergyShort = 1.0f - EnergyPercent;					//so I am 20% short
	EnergyShort *= TheGlobalData->m_LowEnergyPenaltyModifier;	//which is a 40% penalty, or a 10% penalty
	Real penaltyRate = 1.0f - EnergyShort;
	penaltyRate = max(penaltyRate, TheGlobalData->m_MinLowEnergyProductionSpeed);	//bind so 0% does not dead stop you

	if( EnergyPercent < 1.0f )	//and make 99% look like 80% (eg) since most of the time you are down only a little
		penaltyRate = min(penaltyRate, TheGlobalData->m_MaxLowEnergyProductionSpeed);

	if (penaltyRate <= 0.0f)
		penaltyRate = 0.01f;	// Design won't make the minimum 0, they promise

	buildTime /= penaltyRate;//and voila.  Makes sense, designers have control, all is happy.

	//	Multiple build facilities can have an added production bonus.

	if (getBuildCompletion() == BC_APPEARS_AT_RALLY_POINT)
	{
		const ThingTemplate *tmpl = getBuildFacilityTemplate(player);	// could be null if none exist
		Int count = 0;
		if (tmpl)
		{
			player->countObjectsByThingTemplate(1, &tmpl, false, &count);
			Real factoryMult = TheGlobalData->m_MultipleFactory;
			if (factoryMult > 0.0f)
			{
				for(int i=0; i < count - 1; i++)
					buildTime *= factoryMult;
			}
		}
	}

	return(buildTime);
}

//---------------------------------------------------------------------------------------ModuleInfo
//-------------------------------------------------------------------------------------------------
ModuleData* ModuleInfo::friend_getNthData(Int i)
{
	if (i >= 0 && i < m_info.size())
	{
		// This is kinda naughty, but its necessary.
		return const_cast<ModuleData*>(m_info[i].second);
	}
	return nullptr;
}
```
