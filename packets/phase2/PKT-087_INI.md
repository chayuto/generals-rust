# Translation Packet `PKT-087`: `INI`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-087`
- **Work Package:** `WP-2.4` (Phase 2)
- **Complexity & Assigned Tier:** Class `C1` -> `tier:1a-lite`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 1: C-ABI Leaf
- **Allowed Rust Output File:** `crates/zh_common/src/ini/ini.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_common/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (30):**
  - `Common/AsciiString.h`
  - `Common/DamageFX.h`
  - `Common/FileSystem.h`
  - `Common/GameAudio.h`
  - `Common/GameCommon.h`
  - `Common/INI.h`
  - `Common/INIException.h`
  - `Common/STLTypedefs.h`
  - `Common/Science.h`
  - `Common/SpecialPower.h`
  - `Common/ThingFactory.h`
  - `Common/ThingTemplate.h`
  - `Common/Upgrade.h`
  - `Common/Xfer.h`
  - `Common/XferCRC.h`
  - `Common/file.h`
  - `GameClient/Anim2D.h`
  - `GameClient/Color.h`
  - `GameClient/FXList.h`
  - `GameClient/GameText.h`
  - `GameClient/Image.h`
  - `GameClient/ParticleSys.h`
  - `GameLogic/Armor.h`
  - `GameLogic/ExperienceTracker.h`
  - `GameLogic/FPUControl.h`
  - `GameLogic/ObjectCreationList.h`
  - `GameLogic/ScriptEngine.h`
  - `GameLogic/Weapon.h`
  - `PreRTS.h`
  - `WWMath/wwmath.h`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (1 calls — preserve order identically):**
  1. `xferUser( m_buffer, sizeof( char ) * strlen( m_buffer ) )`

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

### `Core/GameEngine/Include/Common/INI.h`
```cpp
// FILE: INI.h ////////////////////////////////////////////////////////////////////////////////////
// Author: Colin Day, November 2001
// Desc:   INI Reader
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include <stddef.h>	// for offsetof, which we don't use but everyone who includes us does
#include "Common/STLTypedefs.h"
#include "Common/AsciiString.h"
#include "Common/GameCommon.h"

//-------------------------------------------------------------------------------------------------
class INI;
class Xfer;
class File;
enum ScienceType CPP_11(: Int);

//-------------------------------------------------------------------------------------------------
/** These control the behavior of loading the INI data into items */
//-------------------------------------------------------------------------------------------------
enum INILoadType CPP_11(: Int)
{
	INI_LOAD_INVALID,						///< invalid load type
	INI_LOAD_OVERWRITE,					///< create new or load *over* existing data instance
	INI_LOAD_CREATE_OVERRIDES,	///< create new or load into *new* override data instance
	INI_LOAD_MULTIFILE					///< create new or continue loading into existing data instance.
};

//-------------------------------------------------------------------------------------------------
/** INI constant defines */
//-------------------------------------------------------------------------------------------------
enum
{
	INI_MAX_CHARS_PER_LINE = 1028,			///< max characters per line entry in any ini file
};

//-------------------------------------------------------------------------------------------------
/** Status return codes for the INI reader */
//-------------------------------------------------------------------------------------------------
enum
{
	// we map all of these to the same "real" error code, because
	// we generally don't care why it failed; but since the code distinguishes,
	// I didn't want to wipe out that intelligence. if we ever need to distinguish
	// failure modes at runtime, just put in real values for these.
	INI_CANT_SEARCH_DIR		= ERROR_BAD_INI,
	INI_INVALID_DIRECTORY = ERROR_BAD_INI,
	INI_INVALID_PARAMS		= ERROR_BAD_INI,
	INI_INVALID_NAME_LIST = ERROR_BAD_INI,
	INI_INVALID_DATA			= ERROR_BAD_INI,
	INI_MISSING_END_TOKEN = ERROR_BAD_INI,
	INI_UNKNOWN_TOKEN			= ERROR_BAD_INI,
	INI_BUFFER_TOO_SMALL  = ERROR_BAD_INI,
	INI_FILE_NOT_OPEN			= ERROR_BAD_INI,
	INI_FILE_ALREADY_OPEN = ERROR_BAD_INI,
	INI_CANT_OPEN_FILE		= ERROR_BAD_INI,
	INI_UNKNOWN_ERROR			= ERROR_BAD_INI,
	INI_END_OF_FILE				= ERROR_BAD_INI
};

//-------------------------------------------------------------------------------------------------
/** Function typedef for parsing data block fields.
	*
	* buffer - the character buffer of the line from INI that we are reading and parsing
	* instance - instance of what we're loading (for example a ThingTemplate instance)
	* store - where to store the data parsed, this is a field in the *instance* 'instance'
	*/
//-------------------------------------------------------------------------------------------------
typedef void (*INIFieldParseProc)( INI *ini, void *instance, void *store, const void* userData );

//-------------------------------------------------------------------------------------------------
typedef const char* const ConstCharPtr;
typedef ConstCharPtr* ConstCharPtrArray;

//-------------------------------------------------------------------------------------------------
struct LookupListRec
{
	const char* name;
	Int value;
};
typedef const LookupListRec *ConstLookupListRecArray;

//-------------------------------------------------------------------------------------------------
/** Parse tables for all fields of each data block are created using these */
//-------------------------------------------------------------------------------------------------
struct FieldParse
{
	const char*					token;						///< token of the field
	INIFieldParseProc		parse;						///< the parse function
	const void*					userData;					///< field-specific data
	Int									offset;						///< offset to data field

	void set(const char* t, INIFieldParseProc p, const void* u, Int o)
	{
		token = t;
		parse = p;
		userData = u;
		offset = o;
	}
};

//-------------------------------------------------------------------------------------------------
class MultiIniFieldParse
{
private:
	enum { MAX_MULTI_FIELDS = 16 };

	const FieldParse* m_fieldParse[MAX_MULTI_FIELDS];
	UnsignedInt				m_extraOffset[MAX_MULTI_FIELDS];
	Int								m_count;

public:
	MultiIniFieldParse() : m_count(0)
	{
		for(Int i = 0; i < MAX_MULTI_FIELDS; i++)
			m_extraOffset[i] = 0;
	}

	void add(const FieldParse* f, UnsignedInt e = 0);

	Int getCount() const { return m_count; }
	const FieldParse* getNthFieldParse(Int i) const { return m_fieldParse[i]; }
	UnsignedInt getNthExtraOffset(Int i) const { return m_extraOffset[i]; }
};

//-------------------------------------------------------------------------------------------------
/** Function typedef for parsing INI types blocks */
//-------------------------------------------------------------------------------------------------
typedef void (*INIBlockParse)( INI *ini );
typedef void (*BuildMultiIniFieldProc)(MultiIniFieldParse& p);

//-------------------------------------------------------------------------------------------------
/** INI Reader interface */
//-------------------------------------------------------------------------------------------------
class INI
{
	INI(const INI&) FUNCTION_DELETE;
	INI& operator=(const INI&) FUNCTION_DELETE;

public:

	typedef UnsignedInt LoadFlags;
	enum LoadFlags_ CPP_11( : UnsignedInt)
	{
		LoadFlags_SearchSubDirs = 1 << 0,
		LoadFlags_ExpectFileFound = 1 << 1,
	};

public:

	INI();

	// TheSuperHackers @feature xezon 19/08/2025
	// Load a specific INI file by name and/or INI files from a directory (and its subdirectories).
	// For example "Data\INI\Armor" loads "Data\INI\Armor.ini" and all *.ini files in "Data\INI\Armor".
	// Throws if not a single INI file is found or one is not read correctly.
	UnsignedInt loadFileDirectory( AsciiString fileDirName, INILoadType loadType, Xfer *pXfer,
		LoadFlags loadFlags = LoadFlags_SearchSubDirs | LoadFlags_ExpectFileFound );

	// Load INI files from a directory (and its subdirectories).
	// Throws if one INI file is not read correctly.
	UnsignedInt loadDirectory( AsciiString dirName, INILoadType loadType, Xfer *pXfer,
		LoadFlags loadFlags = LoadFlags_SearchSubDirs );

	// Load one specific INI file by name.
	// Throws if the INI file is not found or is not read correctly.
	UnsignedInt load( AsciiString filename, INILoadType loadType, Xfer *pXfer );

	static Bool isDeclarationOfType( AsciiString blockType, AsciiString blockName, char *bufferToCheck );
	static Bool isEndOfBlock( char *bufferToCheck );

	// data type parsing (the highest level of what type of thing we're parsing)
	static void parseObjectDefinition( INI *ini );
	static void parseObjectReskinDefinition( INI *ini );
	static void parseWeaponTemplateDefinition( INI *ini );
	static void parseScienceDefinition( INI *ini );
	static void parseRankDefinition( INI *ini );
	static void parseCrateTemplateDefinition( INI *ini );
	static void parseLocomotorTemplateDefinition( INI *ini );
	static void parseLanguageDefinition( INI *ini );
	static void parsePlayerTemplateDefinition( INI *ini );
	static void parseGameDataDefinition( INI *ini );
	static void parseMapDataDefinition( INI *ini );
	static void parseAnim2DDefinition( INI *ini );
	static void parseAudioEventDefinition( INI *ini );
	static void parseDialogDefinition( INI *ini );
	static void parseMusicTrackDefinition( INI *ini );
	static void parseWebpageURLDefinition( INI *ini );
	static void parseHeaderTemplateDefinition( INI *ini );
	static void parseParticleSystemDefinition( INI *ini );
	static void parseWaterSettingDefinition( INI *ini );
	static void parseWaterTransparencyDefinition( INI *ini );
	static void parseWeatherDefinition( INI *ini );
	static void parseMappedImageDefinition( INI *ini );
	static void parseArmorDefinition( INI *ini );
	static void parseDamageFXDefinition( INI *ini );
	static void parseDrawGroupNumberDefinition( INI *ini );
	static void parseTerrainDefinition( INI *ini );
	static void parseTerrainRoadDefinition( INI *ini );
	static void parseTerrainBridgeDefinition( INI *ini );
	static void parseMetaMapDefinition( INI *ini );
	static void parseFXListDefinition( INI *ini );
	static void parseObjectCreationListDefinition( INI* ini );
	static void parseMultiplayerSettingsDefinition( INI* ini );
	static void parseMultiplayerColorDefinition( INI* ini );
  static void parseMultiplayerStartingMoneyChoiceDefinition( INI* ini );
	static void parseOnlineChatColorDefinition( INI* ini );
	static void parseMapCacheDefinition( INI* ini );
	static void parseVideoDefinition( INI* ini );
	static void parseCommandButtonDefinition( INI *ini );
	static void parseCommandSetDefinition( INI *ini );
	static void parseUpgradeDefinition( INI *ini );
	static void parseMouseDefinition( INI* ini );
	static void parseMouseCursorDefinition( INI* ini );
	static void parseAIDataDefinition( INI *ini );
	static void parseSpecialPowerDefinition( INI *ini );
	static void parseInGameUIDefinition( INI *ini );
	static void parseControlBarSchemeDefinition( INI *ini );
	static void parseControlBarResizerDefinition( INI *ini );
	static void parseShellMenuSchemeDefinition( INI *ini );
	static void parseCampaignDefinition( INI *ini );
	static void parseAudioSettingsDefinition( INI *ini );
	static void parseMiscAudio( INI *ini );
	static void parseStaticGameLODDefinition( INI *ini);
	static void parseDynamicGameLODDefinition( INI *ini);
	static void parseStaticGameLODLevel( INI* ini, void * , void *store, const void*);
	static void parseDynamicGameLODLevel( INI* ini, void * , void *store, const void*);
	static void parseLODPreset( INI* ini);
	static void parseBenchProfile( INI* ini);
	static void parseEvaEvent( INI* ini );
	static void parseCredits( INI* ini );
	static void parseWindowTransitions( INI* ini );
	static void parseChallengeModeDefinition( INI* ini );

	AsciiString getFilename() const { return m_filename; }
	INILoadType getLoadType() const { return m_loadType; }
	UnsignedInt getLineNum() const { return m_lineNum; }
	Bool isEOF() const { return m_endOfFile; }
	static const char *getSeps()        { return " \n\r\t="; }   ///< default delimiters for strtok parsing
	static const char *getSepsPercent() { return " \n\r\t=%%"; } ///< default delimiters & percent delimiter
	static const char *getSepsColon()   { return " \n\r\t=:"; }  ///< default delimiters & colon delimiter
	static const char *getSepsQuote()   { return "\"\n="; }      ///< delimiters to represent a quoted ascii string
	static const char *getEndToken()    { return "End"; }        ///< token to represent an end of data block

	void initFromINI( void *what, const FieldParse* parseTable );
	void initFromINIMulti( void *what, const MultiIniFieldParse& parseTableList );
	void initFromINIMultiProc( void *what, BuildMultiIniFieldProc proc );

	static void parseUnsignedByte( INI *ini, void *instance, void *store, const void* userData );
	static void parseShort( INI *ini, void *instance, void *store, const void* userData );
	static void parseUnsignedShort( INI *ini, void *instance, void *store, const void* userData );
	static void parseInt( INI *ini, void *instance, void *store, const void* userData );
	static void parseUnsignedInt( INI *ini, void *instance, void *store, const void* userData );
	static void parseReal( INI *ini, void *instance, void *store, const void* userData );
	static void parsePositiveNonZeroReal( INI *ini, void *instance, void *store, const void* userData );
	static void parseBool( INI *ini, void *instance, void *store, const void* userData );
	static void parseBitInInt32( INI *ini, void *instance, void *store, const void* userData );
	static void parseAsciiString( INI *ini, void *instance, void *store, const void* userData );
	static void parseQuotedAsciiString( INI *ini, void *instance, void *store, const void* userData );
	static void parseAsciiStringVector( INI *ini, void *instance, void *store, const void* userData );
	static void parseAsciiStringVectorAppend( INI *ini, void *instance, void *store, const void* userData );
	static void parseAndTranslateLabel( INI *ini, void *instance, void *store, const void* userData );
	static void parseMappedImage( INI *ini, void *instance, void *store, const void *userData );
	static void parseAnim2DTemplate( INI *ini, void *instance, void *store, const void *userData );
	static void parsePercentToReal( INI *ini, void *instance, void *store, const void* userData );
	static void parseRGBColor( INI *ini, void *instance, void *store, const void* userData );
	static void parseRGBAColorInt( INI *ini, void *instance, void *store, const void* userData );
	static void parseColorInt( INI *ini, void *instance, void *store, const void* userData );
	static void parseCoord3D( INI *ini, void *instance, void *store, const void* userData );
	static void parseCoord2D( INI *ini, void *instance, void *store, const void *userData );
	static void parseICoord2D( INI *ini, void *instance, void *store, const void *userData );
	static void parseDynamicAudioEventRTS( INI *ini, void *instance, void *store, const void* userData );
	static void parseAudioEventRTS( INI *ini, void *instance, void *store, const void* userData );
	static void parseFXList( INI *ini, void *instance, void *store, const void* userData );
	static void parseParticleSystemTemplate( INI *ini, void *instance, void *store, const void *userData );
	static void parseObjectCreationList( INI *ini, void *instance, void *store, const void* userData );
	static void parseSpecialPowerTemplate( INI *ini, void *instance, void *store, const void *userData );
	static void parseUpgradeTemplate( INI *ini, void *instance, void *store, const void *userData );
	static void parseScience( INI *ini, void *instance, void *store, const void *userData );
	static void parseScienceVector( INI *ini, void *instance, void *store, const void *userData );
	static void parseGameClientRandomVariable( INI* ini, void *instance, void *store, const void* userData );
	static void parseBitString8( INI *ini, void *instance, void *store, const void* userData );
	static void parseBitString32( INI *ini, void *instance, void *store, const void* userData );
	static void parseByteSizedIndexList( INI *ini, void *instance, void *store, const void* userData );
	static void parseIndexList( INI *ini, void *instance, void *store, const void* userData );
	static void parseLookupList( INI *ini, void *instance, void *store, const void* userData );
	static void parseThingTemplate( INI *ini, void *instance, void *store, const void* userData );
	static void parseArmorTemplate( INI *ini, void *instance, void *store, const void* userData );
	static void parseDamageFX( INI *ini, void *instance, void *store, const void* userData );
	static void parseWeaponTemplate( INI *ini, void *instance, void *store, const void* userData );
	// parse a duration in msec and convert to duration in frames
	static void parseDurationReal( INI *ini, void *instance, void *store, const void* userData );
	// parse a duration in msec and convert to duration in integral number of frames, (unsignedint) rounding UP
	static void parseDurationUnsignedInt( INI *ini, void *instance, void *store, const void* userData );
	static void parseDurationUnsignedShort( INI *ini, void *instance, void *store, const void *userData );
	// parse acceleration in (dist/sec) and convert to (dist/frame)
	static void parseVelocityReal( INI *ini, void *instance, void *store, const void* userData );
	// parse acceleration in (dist/sec^2) and convert to (dist/frame^2)
	static void parseAccelerationReal( INI *ini, void *instance, void *store, const void* userData );
	// parse angle in degrees and convert to radians
	static void parseAngleReal( INI *ini, void *instance, void *store, const void *userData );
	// note that this parses in degrees/sec, and converts to rads/frame!
	static void parseAngularVelocityReal( INI *ini, void *instance, void *store, const void *userData );
	static void parseDamageTypeFlags(INI* ini, void* instance, void* store, const void* userData);
	static void parseDeathTypeFlags(INI* ini, void* instance, void* store, const void* userData);
	static void parseVeterancyLevelFlags(INI* ini, void* instance, void* store, const void* userData);
	static void parseSoundsList( INI* ini, void *instance, void *store, const void* /*userData*/ );

	/**
		return the next token. if seps is not specified, the standard seps are used.

		this will *never* return null; if there are no more tokens, an exception will be thrown.
	*/
	static const char* getNextToken(const char* seps = getSeps());

	/**
		just like getNextToken(), except that null is returned if no more tokens are present
		(rather than throwing an exception). usually you should call getNextToken(),
		but for some cases this is handier (ie, parsing a variable-length number of tokens).
	*/
	static const char* getNextTokenOrNull(const char* seps = getSeps());

	/**
		This is called when the next thing you expect is something like:

			Tag:value

		pass "Tag" (without the colon) for 'expected', and you will have the 'value'
		token returned.

		If "Tag" is not the next token, an error is thrown.
	*/
	static const char* getNextSubToken(const char* expected);

	/**
		return the next ascii string. this is usually the same the result of getNextToken(),
		except that it allows for quote-delimited strings (eg, "foo bar"), so you can
		get strings with spaces, and/or empty strings.
	*/
	AsciiString getNextAsciiString();
	AsciiString getNextQuotedAsciiString();	//fixed version of above.  We can't fix the regular one for fear of breaking existing code. :-(

	/**
		utility routine that does a sscanf() on the string to get the Science, and throws
		an exception if not of the right form.
	*/
	static ScienceType scanScience(const char* token);

	/**
		utility routine that does a sscanf() on the string to get the int, and throws
		an exception if not of the right form.
	*/
	static Int scanInt(const char* token);

	/**
		utility routine that does a sscanf() on the string to get the unsigned int, and throws
		an exception if not of the right form.
	*/
	static UnsignedInt scanUnsignedInt(const char* token);

	/**
		utility routine that does a sscanf() on the string to get the real, and throws
		an exception if not of the right form.
	*/
	static Real scanReal(const char* token);
	static Real scanPercentToReal(const char* token);

	static Int scanIndexList(const char* token, ConstCharPtrArray nameList);
	static Int scanLookupList(const char* token, ConstLookupListRecArray lookupList);

	static Bool scanBool(const char* token);

protected:

	static Bool isValidINIFilename( const char *filename ); ///< is this a valid .ini filename

	void prepFile( AsciiString filename, INILoadType loadType );
	void unPrepFile();

	void readLine();

	char* m_readBuffer;                       ///< internal read buffer
	unsigned m_readBufferNext;                ///< next char in read buffer
	unsigned m_readBufferUsed;                ///< number of bytes in read buffer

	AsciiString m_filename;										///< filename of file currently loading
	INILoadType m_loadType;										///< load type for current file
	UnsignedInt m_lineNum;										///< current line number that's been read
	char m_buffer[ INI_MAX_CHARS_PER_LINE+1 ];///< buffer to read file contents into
	Bool m_endOfFile;													///< TRUE when we've hit EOF
#ifdef DEBUG_CRASHING
	char m_curBlockStart[ INI_MAX_CHARS_PER_LINE+1 ];	///< first line of cur block
#endif
};
```

### `Core/GameEngine/Source/Common/INI/INI.cpp`
```cpp
// FILE: INI.cpp //////////////////////////////////////////////////////////////////////////////////
// Author: Colin Day, November 2001
// Desc:   INI Reader
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine
#define DEFINE_DEATH_NAMES

#include "WWMath/wwmath.h"
#include "Common/INI.h"
#include "Common/INIException.h"

#include "Common/DamageFX.h"
#include "Common/file.h"
#include "Common/FileSystem.h"
#include "Common/GameAudio.h"
#include "Common/Science.h"
#include "Common/SpecialPower.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/Upgrade.h"
#include "Common/Xfer.h"
#include "Common/XferCRC.h"

#include "GameClient/Anim2D.h"
#include "GameClient/Color.h"
#include "GameClient/FXList.h"
#include "GameClient/GameText.h"
#include "GameClient/Image.h"
#include "GameClient/ParticleSys.h"
#include "GameLogic/Armor.h"
#include "GameLogic/ExperienceTracker.h"
#include "GameLogic/FPUControl.h"
#include "GameLogic/ObjectCreationList.h"
#include "GameLogic/ScriptEngine.h"
#include "GameLogic/Weapon.h"

#if __cplusplus >= 201611L
#define USE_STD_FROM_CHARS_PARSING 1
#else
#define USE_STD_FROM_CHARS_PARSING 0
#endif

#if USE_STD_FROM_CHARS_PARSING
#include <cerrno>
#include <charconv>
#include <cmath>
#include <cstdlib>
#include <limits>
#include <string_view>
#include <type_traits>
#endif

///////////////////////////////////////////////////////////////////////////////////////////////////
// PRIVATE DATA ///////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

static Xfer *s_xfer = nullptr;

//-------------------------------------------------------------------------------------------------
/** This is the table of data types we can have in INI files.  To add a new data type
	* block make a new entry in this table and add an appropriate parsing function */
//-------------------------------------------------------------------------------------------------
extern void parseReallyLowMHz( INI* ini);		// yeah, so sue me (srj)
struct BlockParse
{
	const char *token;
	INIBlockParse parse;
};
static const BlockParse theTypeTable[] =
{
	{ "AIData",                         INI::parseAIDataDefinition },
	{ "Animation",                      INI::parseAnim2DDefinition },
	{ "Armor",                          INI::parseArmorDefinition },
	{ "AudioEvent",                     INI::parseAudioEventDefinition },
	{ "AudioSettings",                  INI::parseAudioSettingsDefinition },
	{ "BenchProfile",                   INI::parseBenchProfile },
	{ "Bridge",                         INI::parseTerrainBridgeDefinition },
	{ "Campaign",                       INI::parseCampaignDefinition },
	{ "ChallengeGenerals",              INI::parseChallengeModeDefinition },
	{ "CommandButton",                  INI::parseCommandButtonDefinition },
	{ "CommandMap",                     INI::parseMetaMapDefinition },
	{ "CommandSet",                     INI::parseCommandSetDefinition },
	{ "ControlBarResizer",              INI::parseControlBarResizerDefinition },
	{ "ControlBarScheme",               INI::parseControlBarSchemeDefinition },
	{ "CrateData",                      INI::parseCrateTemplateDefinition },
	{ "Credits",                        INI::parseCredits },
	{ "DamageFX",                       INI::parseDamageFXDefinition },
	{ "DialogEvent",                    INI::parseDialogDefinition },
	{ "DrawGroupInfo",                  INI::parseDrawGroupNumberDefinition },
	{ "DynamicGameLOD",                 INI::parseDynamicGameLODDefinition },
	{ "EvaEvent",                       INI::parseEvaEvent },
	{ "FXList",                         INI::parseFXListDefinition },
	{ "GameData",                       INI::parseGameDataDefinition },
	{ "HeaderTemplate",                 INI::parseHeaderTemplateDefinition },
	{ "InGameUI",                       INI::parseInGameUIDefinition },
	{ "LODPreset",                      INI::parseLODPreset },
	{ "Language",                       INI::parseLanguageDefinition },
	{ "Locomotor",                      INI::parseLocomotorTemplateDefinition },
	{ "MapCache",                       INI::parseMapCacheDefinition },
	{ "MapData",                        INI::parseMapDataDefinition },
	{ "MappedImage",                    INI::parseMappedImageDefinition },
	{ "MiscAudio",                      INI::parseMiscAudio },
	{ "Mouse",                          INI::parseMouseDefinition },
	{ "MouseCursor",                    INI::parseMouseCursorDefinition },
	{ "MultiplayerColor",               INI::parseMultiplayerColorDefinition },
	{ "MultiplayerSettings",            INI::parseMultiplayerSettingsDefinition },
	{ "MultiplayerStartingMoneyChoice", INI::parseMultiplayerStartingMoneyChoiceDefinition },
	{ "MusicTrack",                     INI::parseMusicTrackDefinition },
	{ "Object",                         INI::parseObjectDefinition },
	{ "ObjectCreationList",             INI::parseObjectCreationListDefinition },
	{ "ObjectReskin",                   INI::parseObjectReskinDefinition },
	{ "OnlineChatColors",               INI::parseOnlineChatColorDefinition },
	{ "ParticleSystem",                 INI::parseParticleSystemDefinition },
	{ "PlayerTemplate",                 INI::parsePlayerTemplateDefinition },
	{ "Rank",                           INI::parseRankDefinition },
	{ "ReallyLowMHz",                   parseReallyLowMHz },
	{ "Road",                           INI::parseTerrainRoadDefinition },
	{ "Science",                        INI::parseScienceDefinition },
	{ "ScriptAction",                   ScriptEngine::parseScriptAction },
	{ "ScriptCondition",                ScriptEngine::parseScriptCondition },
	{ "ShellMenuScheme",                INI::parseShellMenuSchemeDefinition },
	{ "SpecialPower",                   INI::parseSpecialPowerDefinition },
	{ "StaticGameLOD",                  INI::parseStaticGameLODDefinition },
	{ "Terrain",                        INI::parseTerrainDefinition },
	{ "Upgrade",                        INI::parseUpgradeDefinition },
	{ "Video",                          INI::parseVideoDefinition },
	{ "WaterSet",                       INI::parseWaterSettingDefinition },
	{ "WaterTransparency",              INI::parseWaterTransparencyDefinition },
	{ "Weapon",                         INI::parseWeaponTemplateDefinition },
	{ "Weather",                        INI::parseWeatherDefinition },
	{ "WebpageURL",                     INI::parseWebpageURLDefinition },
	{ "WindowTransition",               INI::parseWindowTransitions },
};


///////////////////////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTIONS //////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
Bool INI::isValidINIFilename( const char *filename )
{
	if( filename == nullptr )
		return FALSE;

	return endsWithNoCase(filename, ".ini");
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS ///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
INI::INI()
{

	m_readBuffer = nullptr;
	m_readBufferNext = 0;
	m_readBufferUsed = 0;
	m_filename					= "None";
	m_loadType					= INI_LOAD_INVALID;
	m_lineNum						= 0;
	m_buffer[0]					= 0;
	m_endOfFile					= FALSE;
#ifdef DEBUG_CRASHING
	m_curBlockStart[0]	= 0;
#endif

}

//-------------------------------------------------------------------------------------------------
UnsignedInt INI::loadFileDirectory( AsciiString fileDirName, INILoadType loadType, Xfer *pXfer, LoadFlags loadFlags )
{
	// GeneralsX @feature BenderAI 20/02/2026 Debug hang investigation
	fprintf(stderr, "[INI] loadFileDirectory('%s') START\n", fileDirName.str());
	fflush(stderr);
	
	UnsignedInt filesRead = 0;

	AsciiString iniDir = fileDirName;
	AsciiString iniFile = fileDirName;

	char ext[] = ".ini";

	if (iniDir.endsWithNoCase(ext))
	{
		iniDir.truncateBy(ARRAY_SIZE(ext)-1);
	}

	if (!iniFile.endsWithNoCase(ext))
	{
		iniFile.concat(ext);
	}

	fprintf(stderr, "[INI] loadFileDirectory - checking iniFile: '%s'\n", iniFile.str());
	fflush(stderr);
	
	if (TheFileSystem->doesFileExist(iniFile.str()))
	{
		fprintf(stderr, "[INI] loadFileDirectory - loading iniFile: '%s' START\n", iniFile.str());
		fflush(stderr);
		filesRead += load(iniFile, loadType, pXfer);
		fprintf(stderr, "[INI] loadFileDirectory - loading iniFile: '%s' END\n", iniFile.str());
		fflush(stderr);
	}

	// Load any additional ini files from a "filename" directory and its subdirectories.
	fprintf(stderr, "[INI] loadFileDirectory - calling loadDirectory('%s') START\n", iniDir.str());
	fflush(stderr);
	filesRead += loadDirectory(iniDir, loadType, pXfer, loadFlags & ~LoadFlags_ExpectFileFound);
	fprintf(stderr, "[INI] loadFileDirectory - calling loadDirectory('%s') END\n", iniDir.str());
	fflush(stderr);

	// Expect to open and load at least one file.
	const Bool expectFileFound = (loadFlags & LoadFlags_ExpectFileFound) != 0;
	if (expectFileFound && filesRead == 0)
	{
		fprintf(stderr, "[INI] ERROR: No files read from directory '%s'\n", fileDirName.str());
		fflush(stderr);
		throw INI_CANT_OPEN_FILE;
	}

	fprintf(stderr, "[INI] loadFileDirectory('%s') END - filesRead=%d\n", fileDirName.str(), filesRead);
	fflush(stderr);
	return filesRead;
}

//-------------------------------------------------------------------------------------------------
/** Load all INI files in the specified directory (and subdirectories if indicated).
	* If we are to load subdirectories, we will load them *after* we load all the
	* files in the current directory */
//-------------------------------------------------------------------------------------------------
UnsignedInt INI::loadDirectory( AsciiString dirName, INILoadType loadType, Xfer *pXfer, LoadFlags loadFlags )
{
	// GeneralsX @feature BenderAI 20/02/2026 Debug hang investigation
	fprintf(stderr, "[INI] loadDirectory('%s') START\n", dirName.str());
	fflush(stderr);
	
	UnsignedInt filesRead = 0;

	// sanity
	if( dirName.isEmpty() )
	{
		fprintf(stderr, "[INI] ERROR: Empty directory name in loadDirectory\n");
		fflush(stderr);
		throw INI_INVALID_DIRECTORY;
	}

	const Bool subdirs = (loadFlags & LoadFlags_SearchSubDirs) != 0;
	FilenameList filenameList;
	dirName.concat('\\');
	TheFileSystem->getFileListInDirectory(dirName, "*.ini", filenameList, subdirs);
	// Load the INI files in the dir now, in a sorted order.  This keeps things the same between machines
	// in a network game.
	FilenameList::const_iterator it = filenameList.begin();
	while (it != filenameList.end())
	{
		AsciiString tempname;
		tempname = (*it).str() + dirName.getLength();

		if (getFirstPathSeparator(tempname.str()) == nullptr) {
			// this file doesn't reside in a subdirectory, load it first.
			filesRead += load( *it, loadType, pXfer );
		}
		++it;
	}

	it = filenameList.begin();
	while (it != filenameList.end())
	{
		AsciiString tempname;
		tempname = (*it).str() + dirName.getLength();

		if (getFirstPathSeparator(tempname.str()) != nullptr) {
			filesRead += load( *it, loadType, pXfer );
		}
		++it;
	}

	// Expect to open and load at least one file.
	const Bool expectFileFound = (loadFlags & LoadFlags_ExpectFileFound) != 0;
	if (expectFileFound && filesRead == 0)
	{
		throw INI_CANT_OPEN_FILE;
	}

	return filesRead;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void INI::prepFile( AsciiString filename, INILoadType loadType )
{
	// if we have a file open already -- we can't do another one
	if( m_readBuffer != nullptr )
	{

		DEBUG_CRASH(( "INI::load, cannot open file '%s', file already open", filename.str() ));
		throw INI_FILE_ALREADY_OPEN;

	}

	// open the file
	File* file = TheFileSystem->openFile(filename.str(), File::READ);
	if( file == nullptr )
	{

		DEBUG_CRASH(( "INI::load, cannot open file '%s'", filename.str() ));
		throw INI_CANT_OPEN_FILE;

	}

	m_readBufferNext = 0;
	m_readBufferUsed = file->size();
	m_readBuffer = file->readEntireAndClose();

	// save our filename
	m_filename = filename;

	// save our load type
	m_loadType = loadType;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void INI::unPrepFile()
{
	// delete the buffer
	delete[] m_readBuffer;
	m_readBuffer = nullptr;
	m_readBufferNext = 0;
	m_readBufferUsed = 0;

	m_filename = "None";
	m_loadType = INI_LOAD_INVALID;
	m_lineNum = 0;
	m_endOfFile = FALSE;
	s_xfer = nullptr;
}

//-------------------------------------------------------------------------------------------------
static INIBlockParse findBlockParse(const char* token)
{
	for (size_t i = 0; i < ARRAY_SIZE(theTypeTable); ++i)
	{
		if (strcmp(theTypeTable[i].token, token) == 0)
		{
			return theTypeTable[i].parse;
		}
	}

	return nullptr;
}

//-------------------------------------------------------------------------------------------------
static INIFieldParseProc findFieldParse(const FieldParse* parseTable, const char* token, int& offset, const void*& userData)
{
	const FieldParse* parse = parseTable;
	for (; parse->token; ++parse)
	{
		if (strcmp( parse->token, token ) == 0)
		{
			offset = parse->offset;
			userData = parse->userData;
			return parse->parse;
		}
	}

	if (!parse->token && parse->parse)
	{
		offset = parse->offset;
		userData = token;
		return parse->parse;
	}
	else
	{
		return nullptr;
	}
}

//-------------------------------------------------------------------------------------------------
/** Load and parse an INI file */
//-------------------------------------------------------------------------------------------------
UnsignedInt INI::load( AsciiString filename, INILoadType loadType, Xfer *pXfer )
{
	// GeneralsX @feature BenderAI 20/02/2026 Debug hang investigation
	fprintf(stderr, "[INI] load('%s') START\n", filename.str());
	fflush(stderr);
	
	setFPMode(); // so we have consistent Real values for GameLogic -MDC

	s_xfer = pXfer;
	fprintf(stderr, "[INI] load - calling prepFile('%s') START\n", filename.str());
	fflush(stderr);
	prepFile(filename, loadType);
	fprintf(stderr, "[INI] load - prepFile completed\n");
	fflush(stderr);

	try
	{

		// read all lines in the file
		DEBUG_ASSERTCRASH( m_endOfFile == FALSE, ("INI::load, EOF at the beginning!") );
		UnsignedInt lineCount = 0;
		while( m_endOfFile == FALSE )
		{
			lineCount++;
			if ((lineCount % 100) == 0) {
				fprintf(stderr, "[INI] load - processed %d lines\n", lineCount);
				fflush(stderr);
			}
			
			// read this line
			readLine();

			AsciiString currentLine = m_buffer;

			// the first word is the type of data we're processing
			const char *token = strtok( m_buffer, getSeps() );
			if( token )
			{
				INIBlockParse parse = findBlockParse(token);
				if (parse)
				{
					#ifdef DEBUG_CRASHING
					static_assert(ARRAY_SIZE(m_curBlockStart) >= ARRAY_SIZE(m_buffer), "Incorrect array size");
					strcpy(m_curBlockStart, m_buffer);
					#endif
					try {
						(*parse)( this );

					// GeneralsX @bugfix Copilot 20/09/2026 Preserve the innermost INI field diagnostic.
					} catch (const INIException&) {
						throw;
					} catch (...) {
						DEBUG_CRASH(("Error parsing block '%s' in INI file '%s'", token, m_filename.str()) );
						char buff[1024];
						snprintf(buff, ARRAY_SIZE(buff), "Error parsing INI file '%s' (Line: '%s')\n",
							m_filename.str(), currentLine.str());

						throw INIException(buff);
					}
					#ifdef DEBUG_CRASHING
						strcpy(m_curBlockStart, "NO_BLOCK");
					#endif
				}
				else
				{
					DEBUG_CRASH( ("[LINE: %d - FILE: '%s'] Unknown block '%s'",
														 getLineNum(), getFilename().str(), token ) );
					throw INI_UNKNOWN_TOKEN;
				}

			}

		}
		fprintf(stderr, "[INI] load - processed total %d lines\n", lineCount);
		fflush(stderr);
	}
	catch (...)
	{
		fprintf(stderr, "[INI] ERROR in load('%s') - exception caught\n", filename.str());
		fflush(stderr);
		unPrepFile();

		// propagate the exception.
		throw;
	}

	unPrepFile();

	fprintf(stderr, "[INI] load('%s') END\n", filename.str());
	fflush(stderr);
	return 1;
}

//-------------------------------------------------------------------------------------------------
/** Read a line from the already open file.  Any comments will be removed and
	* therefore ignored from any given line
	* 
	* TheSuperHackers @performance xezon 18/01/2026 The file contents are now read directly from a
	* full File Ram buffer into the INI Line Buffer without a third buffer in between.
	*/
//-------------------------------------------------------------------------------------------------
void INI::readLine()
{
	// sanity
	DEBUG_ASSERTCRASH( m_readBuffer, ("readLine(), read buffer is null") );

	if (m_endOfFile)
	{
		*m_buffer = 0;
	}
	else
	{
		// read up till the newline or semicolon character, or until out of space
		char *p = m_buffer;
		while (p != m_buffer+INI_MAX_CHARS_PER_LINE)
		{
			// test end of read buffer
			if (m_readBufferNext==m_readBufferUsed)
			{
				m_endOfFile = true;
				*p = 0;
				break;
			}

			// get next character
			*p = m_readBuffer[m_readBufferNext++];

			// check for new line
			if (*p == '\n')
			{
				*p = 0;
				break;
			}

			DEBUG_ASSERTCRASH(*p != '\t', ("tab characters are not allowed in INI files (%s). please check your editor settings. Line Number %d", m_filename.str(), getLineNum()));

			// if this is a semicolon, that represents the start of a comment
			if (*p == ';')
			{
				*p = 0;
			}

			// make whitespace characters actual spaces
			else if (*p > 0 && *p < 32)
			{
				*p = ' ';
			}

			p++;
		}

		*p = 0;

		// increase our line count
		m_lineNum++;

		// check for at the max
		if ( p == m_buffer+INI_MAX_CHARS_PER_LINE )
		{
			DEBUG_CRASH( ("Buffer too small (%d) and was truncated, increase INI_MAX_CHARS_PER_LINE", INI_MAX_CHARS_PER_LINE) );
		}
	}

	if (s_xfer)
	{
		s_xfer->xferUser( m_buffer, sizeof( char ) * strlen( m_buffer ) );
		//DEBUG_LOG(("Xfer val is now 0x%8.8X in %s, line %s", ((XferCRC *)s_xfer)->getCRC(), m_filename.str(), m_buffer));
	}
}

//-------------------------------------------------------------------------------------------------
/** Parse UnsignedByte from buffer and assign at location 'store' */
//-------------------------------------------------------------------------------------------------
void INI::parseUnsignedByte( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	const char *token = ini->getNextToken();
	Int value = scanInt(token);
	if (value < 0 || value > 255)
	{
		DEBUG_CRASH(("Bad value INI::parseUnsignedByte"));
		throw ERROR_BUG;
	}
	*(Byte *)store = (Byte)value;
}

//-------------------------------------------------------------------------------------------------
/** Parse signed short from buffer and assign at location 'store' */
//-------------------------------------------------------------------------------------------------
void INI::parseShort( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	const char *token = ini->getNextToken();
	Int value = scanInt(token);
	if (value < -32768 || value > 32767)
	{
		DEBUG_CRASH(("Bad value INI::parseShort"));
		throw ERROR_BUG;
	}
	*(Short *)store = (Short)value;
}

//-------------------------------------------------------------------------------------------------
/** Parse unsigned short from buffer and assign at location 'store' */
//-------------------------------------------------------------------------------------------------
void INI::parseUnsignedShort( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	const char *token = ini->getNextToken();
	Int value = scanInt(token);
	if (value < 0 || value > 65535)
	{
		DEBUG_CRASH(("Bad value INI::parseUnsignedShort"));
		throw ERROR_BUG;
	}
	*(UnsignedShort *)store = (UnsignedShort)value;
}

//-------------------------------------------------------------------------------------------------
/** Parse integer from buffer and assign at location 'store' */
//-------------------------------------------------------------------------------------------------
void INI::parseInt( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	const char *token = ini->getNextToken();
	*(Int *)store = scanInt(token);

}

//-------------------------------------------------------------------------------------------------
/** Parse unsigned integer from buffer and assign at location 'store' */
//-------------------------------------------------------------------------------------------------
void INI::parseUnsignedInt( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	const char *token = ini->getNextToken();
	*(UnsignedInt *)store = scanUnsignedInt(token);

}

//-------------------------------------------------------------------------------------------------
/** Parse real from buffer and assign at location 'store' */
//-------------------------------------------------------------------------------------------------
void INI::parseReal( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	const char *token = ini->getNextToken();
	*(Real *)store = scanReal(token);

}

//-------------------------------------------------------------------------------------------------
/** Parse real from buffer and assign at location 'store' */
//-------------------------------------------------------------------------------------------------
void INI::parsePositiveNonZeroReal( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	const char *token = ini->getNextToken();
	*(Real *)store = scanReal(token);
	if (*(Real *)store <= 0.0f)
	{
		DEBUG_CRASH(("invalid Real value %f -- expected > 0",*(Real*)store));
		throw INI_INVALID_DATA;
	}

}

//-------------------------------------------------------------------------------------------------
/** Parse a degree value (0 to 360) and store the radian value of that degree
	* in a Real */
//-------------------------------------------------------------------------------------------------
void INI::parseAngleReal( INI *ini, void * /*instance*/,
																			void *store, const void *userData )
{
	const char *token = ini->getNextToken();

	const Real RADS_PER_DEGREE = PI / 180.0f;
	*(Real *)store = scanReal( token ) * RADS_PER_DEGREE;

}

//-------------------------------------------------------------------------------------------------
/** Parse an angular velocity in degrees-per-sec and store the rads-per-frame value of that degree
	* in a Real */
//-------------------------------------------------------------------------------------------------
void INI::parseAngularVelocityReal( INI *ini, void * /*instance*/,
																			void *store, const void *userData )
{
	const char *token = ini->getNextToken();

	// scan the int and convert to radian and store as a real
	*(Real *)store = ConvertAngularVelocityInDegreesPerSecToRadsPerFrame(scanReal( token ));

}

//-------------------------------------------------------------------------------------------------
/** Parse Bool from buffer and assign at location 'store'.  The buffer token must
	* be in the form of a string "Yes" or "No" (case is ignored) */
//-------------------------------------------------------------------------------------------------
void INI::parseBool( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	*(Bool*)store = INI::scanBool(ini->getNextToken());
}

//-------------------------------------------------------------------------------------------------
/** Parse Bool from buffer; if true, or in MASK, otherwise and out MASK. The buffer token must
	* be in the form of a string "Yes" or "No" (case is ignored) */
//-------------------------------------------------------------------------------------------------
void INI::parseBitInInt32( INI *ini, void *instance, void *store, const void* userData )
{
	UnsignedInt* s = (UnsignedInt*)store;
	UnsignedInt mask = (UnsignedInt)(uintptr_t)userData;

	if (INI::scanBool(ini->getNextToken()))
		*s |= mask;
	else
		*s &= ~mask;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
/*static*/ Bool INI::scanBool(const char* token)
{
	// translate string yes/no into TRUE/FALSE
	if( stricmp( token, "yes" ) == 0 )
		return TRUE;
	else if( stricmp( token, "no" ) == 0 )
		return FALSE;
	else
	{
		DEBUG_CRASH(("invalid boolean token %s -- expected Yes or No",token));
		throw INI_INVALID_DATA;
		return false;	// keep compiler happy
	}

}

//-------------------------------------------------------------------------------------------------
/** Parse an *ASCII* string from buffer and assign at location 'store' */
//-------------------------------------------------------------------------------------------------
void INI::parseAsciiString( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	AsciiString* asciiString = (AsciiString *)store;
	*asciiString = ini->getNextAsciiString();
}

//-------------------------------------------------------------------------------------------------
/** Parse an *ASCII* string from buffer and assign at location 'store'. Has better support for quoted strings.
We don't really need this function, but parseString() is broken and we want to leave it broken to
maintain existing code.
 */
//-------------------------------------------------------------------------------------------------
void INI::parseQuotedAsciiString( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	AsciiString* asciiString = (AsciiString *)store;
	*asciiString = ini->getNextQuotedAsciiString();
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void INI::parseAsciiStringVector( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	std::vector<AsciiString>* asv = (std::vector<AsciiString>*)store;
	asv->clear();
	for (const char *token = ini->getNextTokenOrNull(); token; token = ini->getNextTokenOrNull())
	{
		asv->push_back(token);
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void INI::parseAsciiStringVectorAppend( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	std::vector<AsciiString>* asv = (std::vector<AsciiString>*)store;
	// nope, don't clear. duh.
	// asv->clear();
	for (const char *token = ini->getNextTokenOrNull(); token; token = ini->getNextTokenOrNull())
	{
		asv->push_back(token);
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
/* static */void INI::parseScienceVector( INI *ini, void * /*instance*/, void *store, const void *userData )
{
	ScienceVec* asv = (ScienceVec*)store;
	asv->clear();
	for (const char *token = ini->getNextTokenOrNull(); token; token = ini->getNextTokenOrNull())
	{
		if (stricmp(token, "None") == 0)
		{
			asv->clear();
			return;
		}
		asv->push_back(INI::scanScience( token ));
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
AsciiString INI::getNextQuotedAsciiString()
{
	AsciiString result;
	char buff[INI_MAX_CHARS_PER_LINE];
	buff[0] = '\0';

	const char *token = getNextTokenOrNull();	// if null, just leave an empty string
	if (token != nullptr)
	{
		if (token[0] != '\"')
		{
			// if token is simply "
			result.set( token );	// Start following the "
		}
		else
		{	int strLen=0;
			Bool done=FALSE;
			if ((strLen=strlen(token)) > 1)
			{
				strlcpy(buff, &token[1], ARRAY_SIZE(buff));	//skip the starting quote
				//Check for end of quoted string.  Checking here fixes cases where quoted string on same line with other data.
				if (buff[strLen-2]=='"')	//skip ending quote if present
				{	buff[strLen-2]='\0';
					done=TRUE;
				}
			}

			if (!done)
			{
				token = getNextToken(getSepsQuote());

				if (strlen(token) > 1 && token[1] != '\t')
				{
					strlcat(buff, " ", ARRAY_SIZE(buff));
					strlcat(buff, token, ARRAY_SIZE(buff));
				}
				else
				{	Int buflen=strlen(buff);
					if (buff[buflen-1]=='\"')
						buff[buflen-1]='\0';
				}
			}
			result.set(buff);
		}
	}
	return result;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
AsciiString INI::getNextAsciiString()
{
	AsciiString result;

	const char *token = getNextTokenOrNull();	// if null, just leave an empty string
	if (token != nullptr)
	{
		if (token[0] != '\"')
		{
			// if token is simply "
			result.set( token );	// Start following the "
		}
		else
		{
			static char buff[INI_MAX_CHARS_PER_LINE];
			buff[0] = 0;
			if (strlen(token) > 1)
			{
				strlcpy(buff, &token[1], ARRAY_SIZE(buff));
			}

			token = getNextTokenOrNull(getSepsQuote());
			if (token) {
				if (strlen(token) > 1 && token[1] != '\t')
				{
					strlcat(buff, " ", ARRAY_SIZE(buff));
				}
				strlcat(buff, token, ARRAY_SIZE(buff));
				result.set(buff);
			} else {
				Int len = strlen(buff);
				if (len && buff[len-1] == '"') {
					// strip off trailing quote jba. [2/12/2003]
					buff[len-1] = 0;
				}
				result.set(buff);
			}
		}
	}
	return result;
}

//-------------------------------------------------------------------------------------------------
/** Parse a string label, get the *translated* actual text from the label and store
	* into a *UNICODE* string. */
//-------------------------------------------------------------------------------------------------
void INI::parseAndTranslateLabel( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	const char *token = ini->getNextToken();

	// translate
	UnicodeString translated = TheGameText->fetch( token );
	if( translated.isEmpty() )
		throw INI_INVALID_DATA;

	// save the translated text
	UnicodeString *theString = (UnicodeString *)store;
	theString->set( translated.str() );

}

//-------------------------------------------------------------------------------------------------
/** Parse a string label assumed as an image as part of the image collection.  Translate
	* to an image pointer for storage */
//-------------------------------------------------------------------------------------------------
void INI::parseMappedImage( INI *ini, void * /*instance*/, void *store, const void *userData )
{
	const char *token = ini->getNextToken();

	if( TheMappedImageCollection )
	{
		typedef const Image* ConstImagePtr;
		*(ConstImagePtr*)store = TheMappedImageCollection->findImageByName( token );
	}

	//KM: If we are in the worldbuilder, we want to parse commandbuttons for informational purposes,
	//but we don't care about the images -- because we never access them. In RTS/GUIEdit, they always
	//exist -- and in those cases, it will never call this code anyways because it'll throw long before.
	//else
	//	throw INI_UNKNOWN_ERROR;

}

// ------------------------------------------------------------------------------------------------
/** Parse a string label assumed as a Anim2D template name.  Translate that name to an
	* actual template pointer for storage */
// ------------------------------------------------------------------------------------------------
/*static*/ void INI::parseAnim2DTemplate( INI *ini, void *instance, void *store, const void *userData )
{
	const char *token = ini->getNextToken();

	if( TheAnim2DCollection )
	{
		Anim2DTemplate **anim2DTemplate = (Anim2DTemplate **)store;
		*anim2DTemplate = TheAnim2DCollection->findTemplate( AsciiString( token ) );
	}
	else
	{

		DEBUG_CRASH(( "INI::parseAnim2DTemplate - TheAnim2DCollection is null" ));
		throw INI_UNKNOWN_ERROR;

	}

}

//-------------------------------------------------------------------------------------------------
/** Parse a percent in int or real form such as "23%" or "95.4%" and assign
	* to location 'store' as a number from 0.0 to 1.0 */
//-------------------------------------------------------------------------------------------------
void INI::parsePercentToReal( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	const char *token = ini->getNextToken(ini->getSepsPercent());
	Real *theReal = (Real *)store;
	*theReal = scanPercentToReal(token);

}

//-------------------------------------------------------------------------------------------------
/** 'store' points to an 32 bit unsigned integer.  We will zero that integer, parse each token
	* in the buffer, if the token is in the userData table of strings, we will set the
	* according bit flag for it */
//-------------------------------------------------------------------------------------------------
void INI::parseBitString8( INI* ini, void * /*instance*/, void *store, const void* userData )
{
	UnsignedInt tmp = *(Byte*)store;
	INI::parseBitString32(ini, nullptr, &tmp, userData);
	if (tmp & 0xffffff00)
	{
		DEBUG_CRASH(("Bad bitstring list INI::parseBitString8"));
		throw ERROR_BUG;
	}
	*(Byte*)store = (Byte)tmp;
}

//-------------------------------------------------------------------------------------------------
/** 'store' points to an 32 bit unsigned integer.  We will zero that integer, parse each token
	* in the buffer, if the token is in the userData table of strings, we will set the
	* according bit flag for it */
//-------------------------------------------------------------------------------------------------
void INI::parseBitString32( INI* ini, void * /*instance*/, void *store, const void* userData )
{
	ConstCharPtrArray flagList = (ConstCharPtrArray)userData;
	UnsignedInt *bits = (UnsignedInt *)store;

	if( flagList == nullptr || flagList[ 0 ] == nullptr)
	{
		DEBUG_ASSERTCRASH( flagList, ("INTERNAL ERROR! parseBitString32: No flag list provided!") );
		throw INI_INVALID_NAME_LIST;
	}

	Bool foundNormal = false;
	Bool foundAddOrSub = false;

	// loop through all tokens
	for (const char *token = ini->getNextTokenOrNull(); token; token = ini->getNextTokenOrNull())
	{
		if (stricmp(token, "NONE") == 0)
		{
			if (foundNormal || foundAddOrSub)
			{
				DEBUG_CRASH(("you may not mix normal and +- ops in bitstring lists"));
				throw INI_INVALID_NAME_LIST;
			}
			*bits = 0;
			break;
		}

		if (token[0] == '+')
		{
			if (foundNormal)
			{
				DEBUG_CRASH(("you may not mix normal and +- ops in bitstring lists"));
				throw INI_INVALID_NAME_LIST;
			}
			Int bitIndex = INI::scanIndexList(token+1, flagList);	// this throws if the token is not found
			*bits |= (1 << bitIndex);
			foundAddOrSub = true;
		}
		else if (token[0] == '-')
		{
			if (foundNormal)
			{
				DEBUG_CRASH(("you may not mix normal and +- ops in bitstring lists"));
				throw INI_INVALID_NAME_LIST;
			}
			Int bitIndex = INI::scanIndexList(token+1, flagList);	// this throws if the token is not found
			*bits &= ~(1 << bitIndex);
			foundAddOrSub = true;
		}
		else
		{
			if (foundAddOrSub)
			{
				DEBUG_CRASH(("you may not mix normal and +- ops in bitstring lists"));
				throw INI_INVALID_NAME_LIST;
			}

			if (!foundNormal)
				*bits = 0;

			Int bitIndex = INI::scanIndexList(token, flagList);	// this throws if the token is not found
			*bits |= (1 << bitIndex);
			foundNormal = true;
		}
	}
}

//-------------------------------------------------------------------------------------------------
/** Parse a color in the form of
	*
	* RGB_COLOR = R:100 G:114 B:245
	* and store in "RGBColor" structure pointed to by 'store' */
//-------------------------------------------------------------------------------------------------
void INI::parseRGBColor( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	const char* names[3] = { "R", "G", "B" };
	Int colors[3];
	for( Int i = 0; i < 3; i++ )
	{
		colors[i] = scanInt(ini->getNextSubToken(names[i]));
		if( colors[ i ] < 0 )
			throw INI_INVALID_DATA;
		if( colors[ i ] > 255 )
			throw INI_INVALID_DATA;
	}

	// assign the color components to the "RGBColor" pointer at 'store'
	RGBColor *theColor = (RGBColor *)store;
	theColor->red		= (Real)colors[ 0 ] / 255.0f;
	theColor->green = (Real)colors[ 1 ] / 255.0f;
	theColor->blue	= (Real)colors[ 2 ] / 255.0f;

}

//-------------------------------------------------------------------------------------------------
/** Parse a color in the form of
	*
	* RGB_COLOR = R:100 G:114 B:245 [A:233]
	* and store in "RGBAColorInt" structure pointed to by 'store' */
//-------------------------------------------------------------------------------------------------
void INI::parseRGBAColorInt( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	const char* names[4] = { "R", "G", "B", "A" };
	Int colors[4];
	for( Int i = 0; i < 4; i++ )
	{
		const char* token = ini->getNextTokenOrNull(ini->getSepsColon());
		if (token == nullptr)
		{
			if (i < 3)
			{
				throw INI_INVALID_DATA;
			}
			else
			{
				// it's ok for A to be omitted.
				colors[i] = 255;
			}
		}
		else
		{
			// if present, the token must match.
			if (stricmp(token, names[i]) != 0)
			{
				throw INI_INVALID_DATA;
			}
			colors[i] = scanInt(ini->getNextToken(ini->getSepsColon()));
		}
		if( colors[ i ] < 0 )
			throw INI_INVALID_DATA;
		if( colors[ i ] > 255 )
			throw INI_INVALID_DATA;
	}

	//
	// assign the color components to the "RGBColorInt" pointer at 'store', keep
	// the numbers as between 0 and 255
	//
	RGBAColorInt *theColor = (RGBAColorInt *)store;
	theColor->red		= colors[ 0 ];
	theColor->green = colors[ 1 ];
	theColor->blue	= colors[ 2 ];
	theColor->alpha = colors[ 3 ];

}

//-------------------------------------------------------------------------------------------------
/** Parse a color in the form of
	*
	* RGB_COLOR = R:100 G:114 B:245 [A:233]
	* and store in "Color" structure pointed to by 'store' */
//-------------------------------------------------------------------------------------------------
void INI::parseColorInt( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	const char* names[4] = { "R", "G", "B", "A" };
	Int colors[4];
	for( Int i = 0; i < 4; i++ )
	{
		const char* token = ini->getNextTokenOrNull(ini->getSepsColon());
		if (token == nullptr)
		{
			if (i < 3)
			{
				throw INI_INVALID_DATA;
			}
			else
			{
				// it's ok for A to be omitted.
				colors[i] = 255;
			}
		}
		else
		{
			// if present, the token must match.
			if (stricmp(token, names[i]) != 0)
			{
				throw INI_INVALID_DATA;
			}
			colors[i] = scanInt(ini->getNextToken(ini->getSepsColon()));
		}
		if( colors[ i ] < 0 )
			throw INI_INVALID_DATA;
		if( colors[ i ] > 255 )
			throw INI_INVALID_DATA;
	}

	//
	// assign the color components to the "Color" pointer at 'store', keep
	// the numbers as between 0 and 255
	//
	Color *theColor = (Color *)store;
	*theColor = GameMakeColor(colors[0], colors[1], colors[2], colors[3]);

}

//-------------------------------------------------------------------------------------------------
/** Parse a 3D coordinate of reals in the form of:
	* FIELD_NAME = X:400 Y:-214.3 Z:8.6 */
//-------------------------------------------------------------------------------------------------
void INI::parseCoord3D( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	Coord3D *theCoord = (Coord3D *)store;

	theCoord->x = scanReal(ini->getNextSubToken("X"));
	theCoord->y = scanReal(ini->getNextSubToken("Y"));
	theCoord->z = scanReal(ini->getNextSubToken("Z"));

}

//-------------------------------------------------------------------------------------------------
/** Parse a 2D coordinate of reals in the form of:
	* FIELD_NAME = X:400 Y:-214.3 */
//-------------------------------------------------------------------------------------------------
void INI::parseCoord2D( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	Coord2D *theCoord = (Coord2D *)store;

	theCoord->x = scanReal(ini->getNextSubToken("X"));
	theCoord->y = scanReal(ini->getNextSubToken("Y"));

}

//-------------------------------------------------------------------------------------------------
/** Parse a 2D coordinate of Ints in the form of:
	* FIELD_NAME = X:400 Y:-214 */
//-------------------------------------------------------------------------------------------------
void INI::parseICoord2D( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	ICoord2D *theCoord = (ICoord2D *)store;

	theCoord->x = scanInt(ini->getNextSubToken("X"));
	theCoord->y = scanInt(ini->getNextSubToken("Y"));

}

//-------------------------------------------------------------------------------------------------
/** Parse an audio event and assign to the 'AudioEventRTS*' at store */
//-------------------------------------------------------------------------------------------------
void INI::parseDynamicAudioEventRTS( INI *ini, void * /*instance*/, void *store, const void* userData )
{
	const char *token = ini->getNextToken();
	RefCountPtr<DynamicAudioEventRTS>* theSound = (RefCountPtr<DynamicAudioEventRTS>*)store;

	// translate the string into a sound
	if (stricmp(token, "NoSound") == 0)
	{
		theSound->Clear();
	}
	else
	{
		if (*theSound == nullptr)
			*theSound = Create_No_Add_Ref(newInstance(DynamicAudioEventRTS));
		(*theSound)->setEventName(AsciiString(token));
	}

	if (*theSound)
		TheAudio->getInfoForAudioEvent(theSound->Peek());
}

//-------------------------------------------------------------------------------------------------
/** Parse an audio event and assign to the 'AudioEventRTS*' at store */
//-------------------------------------------------------------------------------------------------
void INI::parseAudioEventRTS( INI *ini, void * /*instance*/, void *store, const void* userData )
{
	const char *token = ini->getNextToken();

	AudioEventRTS *theSound = (AudioEventRTS*)store;

	// translate the string into a sound
	if (stricmp(token, "NoSound") != 0) {
		theSound->setEventName(AsciiString(token));
	}

	TheAudio->getInfoForAudioEvent(theSound);
}

//-------------------------------------------------------------------------------------------------
/** Parse an ThingTemplate and assign to the 'ThingTemplate *' at store */
//-------------------------------------------------------------------------------------------------
void INI::parseThingTemplate( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	const char *token = ini->getNextToken();

	if (!TheThingFactory)
	{
		DEBUG_CRASH(("TheThingFactory not inited yet"));
		throw ERROR_BUG;
	}

	typedef const ThingTemplate *ConstThingTemplatePtr;
	ConstThingTemplatePtr* theThingTemplate = (ConstThingTemplatePtr*)store;

	if (stricmp(token, "None") == 0)
	{
		*theThingTemplate = nullptr;
	}
	else
	{
		const ThingTemplate *tt = TheThingFactory->findTemplate(token);	// could be null!
		DEBUG_ASSERTCRASH(tt, ("ThingTemplate %s not found!",token));
		// assign it, even if null!
		*theThingTemplate = tt;
	}

}

//-------------------------------------------------------------------------------------------------
/** Parse an ArmorTemplate and assign to the 'ArmorTemplate *' at store */
//-------------------------------------------------------------------------------------------------
void INI::parseArmorTemplate( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	const char *token = ini->getNextToken();

	typedef const ArmorTemplate *ConstArmorTemplatePtr;
	ConstArmorTemplatePtr* theArmorTemplate = (ConstArmorTemplatePtr*)store;

	if (stricmp(token, "None") == 0)
	{
		*theArmorTemplate = nullptr;
	}
	else
	{
		const ArmorTemplate *tt = TheArmorStore->findArmorTemplate(token);	// could be null!
		DEBUG_ASSERTCRASH(tt, ("ArmorTemplate %s not found!",token));
		// assign it, even if null!
		*theArmorTemplate = tt;
	}

}

//-------------------------------------------------------------------------------------------------
/** Parse an WeaponTemplate and assign to the 'WeaponTemplate *' at store */
//-------------------------------------------------------------------------------------------------
void INI::parseWeaponTemplate( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	const char *token = ini->getNextToken();

	typedef const WeaponTemplate *ConstWeaponTemplatePtr;
	ConstWeaponTemplatePtr* theWeaponTemplate = (ConstWeaponTemplatePtr*)store;

	const WeaponTemplate *tt = TheWeaponStore->findWeaponTemplate(token);	// could be null!
	DEBUG_ASSERTCRASH(tt || stricmp(token, "None") == 0, ("WeaponTemplate %s not found!",token));
	// assign it, even if null!
	*theWeaponTemplate = tt;

}

//-------------------------------------------------------------------------------------------------
/** Parse an FXList and assign to the 'FXList *' at store */
//-------------------------------------------------------------------------------------------------
void INI::parseFXList( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	const char *token = ini->getNextToken();

	typedef const FXList *ConstFXListPtr;
	ConstFXListPtr* theFXList = (ConstFXListPtr*)store;

	const FXList *fxl = TheFXListStore->findFXList(token);	// could be null!
	DEBUG_ASSERTCRASH(fxl != nullptr || stricmp(token, "None") == 0, ("FXList %s not found!",token));
	// assign it, even if null!
	*theFXList = fxl;

}

//-------------------------------------------------------------------------------------------------
/** Parse a particle system and assign to 'ParticleSystemTemplate *' at store */
//-------------------------------------------------------------------------------------------------
void INI::parseParticleSystemTemplate( INI *ini, void * /*instance*/, void *store, const void *userData )
{
	const char *token = ini->getNextToken();

	const ParticleSystemTemplate *pSystemT = TheParticleSystemManager->findTemplate( AsciiString( token ) );
	DEBUG_ASSERTCRASH( pSystemT || stricmp( token, "None" ) == 0, ("ParticleSystem %s not found!",token) );

	typedef const ParticleSystemTemplate* ConstParticleSystemTemplatePtr;
	ConstParticleSystemTemplatePtr* theParticleSystemTemplate = (ConstParticleSystemTemplatePtr*)store;

	*theParticleSystemTemplate = pSystemT;

}

//-------------------------------------------------------------------------------------------------
/** Parse an DamageFX and assign to the 'DamageFX *' at store */
//-------------------------------------------------------------------------------------------------
void INI::parseDamageFX( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	const char *token = ini->getNextToken();

	typedef const DamageFX *ConstDamageFXPtr;
	ConstDamageFXPtr* theDamageFX = (ConstDamageFXPtr*)store;

	if (stricmp(token, "None") == 0)
	{
		*theDamageFX = nullptr;
	}
	else
	{
		const DamageFX *fxl = TheDamageFXStore->findDamageFX(token);	// could be null!
		DEBUG_ASSERTCRASH(fxl, ("DamageFX %s not found!",token));
		// assign it, even if null!
		*theDamageFX = fxl;
	}

}

//-------------------------------------------------------------------------------------------------
/** Parse an ObjectCreationList and assign to the 'ObjectCreationList *' at store */
//-------------------------------------------------------------------------------------------------
void INI::parseObjectCreationList( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	const char *token = ini->getNextToken();

	typedef const ObjectCreationList *ConstObjectCreationListPtr;
	ConstObjectCreationListPtr* theObjectCreationList = (ConstObjectCreationListPtr*)store;

	const ObjectCreationList *ocl = TheObjectCreationListStore->findObjectCreationList(token);	// could be null!
	DEBUG_ASSERTCRASH(ocl || stricmp(token, "None") == 0, ("ObjectCreationList %s not found!",token));
	// assign it, even if null!
	*theObjectCreationList = ocl;

}

//-------------------------------------------------------------------------------------------------
/** Parse a upgrade template string and store as template pointer */
//-------------------------------------------------------------------------------------------------
void INI::parseUpgradeTemplate( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	const char *token = ini->getNextToken();

	if (!TheUpgradeCenter)
	{
		DEBUG_CRASH(("TheUpgradeCenter not inited yet"));
		throw ERROR_BUG;
	}

	const UpgradeTemplate *uu = TheUpgradeCenter->findUpgrade( token );
	DEBUG_ASSERTCRASH( uu || stricmp( token, "None" ) == 0, ("Upgrade %s not found!",token) );

	typedef const UpgradeTemplate* ConstUpgradeTemplatePtr;
	ConstUpgradeTemplatePtr* theUpgradeTemplate = (ConstUpgradeTemplatePtr *)store;
	*theUpgradeTemplate = uu;
}

//-------------------------------------------------------------------------------------------------
/** Parse a special power template string and store as template pointer */
//-------------------------------------------------------------------------------------------------
void INI::parseSpecialPowerTemplate( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	const char *token = ini->getNextToken();

	if (!TheSpecialPowerStore)
	{
		DEBUG_CRASH(("TheSpecialPowerStore not inited yet"));
		throw ERROR_BUG;
	}

	const SpecialPowerTemplate *sPowerT = TheSpecialPowerStore->findSpecialPowerTemplate( AsciiString( token ) );
	if( !sPowerT && stricmp( token, "None" ) != 0 )
	{
		DEBUG_CRASH( ("[LINE: %d in '%s'] Specialpower %s not found!", ini->getLineNum(), ini->getFilename().str(), token) );
	}

	typedef const SpecialPowerTemplate* ConstSpecialPowerTemplatePtr;
	ConstSpecialPowerTemplatePtr* theSpecialPowerTemplate = (ConstSpecialPowerTemplatePtr *)store;
	*theSpecialPowerTemplate = sPowerT;
}

//-------------------------------------------------------------------------------------------------
/** Parse a science string and store as science type */
//-------------------------------------------------------------------------------------------------
/* static */void INI::parseScience( INI *ini, void * /*instance*/, void *store, const void *userData )
{
	const char *token = ini->getNextToken();

	if (!TheScienceStore)
	{
		DEBUG_CRASH(("TheScienceStore not inited yet"));
		throw ERROR_BUG;
	}

	*((ScienceType *)store) = INI::scanScience(token);

}

//-------------------------------------------------------------------------------------------------
/** Parse a single string token, check for that token in the index list
	* of names provided and store the index into that list.
	*
	* NOTE: Is is assumed that we are going to store the index into
	*				a 4 byte integer.  This works well for INT and ENUM definitions */
//-------------------------------------------------------------------------------------------------
void INI::parseIndexList( INI* ini, void * /*instance*/, void *store, const void* userData )
{
	ConstCharPtrArray nameList = (ConstCharPtrArray)userData;
	*(Int *)store = scanIndexList(ini->getNextToken(), nameList);
}

//-------------------------------------------------------------------------------------------------
/** Parse a single string token, check for that token in the index list
	* of names provided and store the index into that list.
	*
	* NOTE: Is is assumed that we are going to store the index into
	*				a 4 byte integer.  This works well for INT and ENUM definitions */
//-------------------------------------------------------------------------------------------------
void INI::parseByteSizedIndexList( INI* ini, void * /*instance*/, void *store, const void* userData )
{
	ConstCharPtrArray nameList = (ConstCharPtrArray)userData;
	Int value = scanIndexList(ini->getNextToken(), nameList);
	if (value < 0 || value > 255)
	{
		DEBUG_CRASH(("Bad index list INI::parseByteSizedIndexList"));
		throw ERROR_BUG;
	}
	*(Byte *)store = (Byte)value;
}

//-------------------------------------------------------------------------------------------------
/** Parse a single string token, check for that token in the index list
	* of names provided and store the associated value into that list.
	*
	* NOTE: Is is assumed that we are going to store the index into
	*				a 4 byte integer.  This works well for INT and ENUM definitions */
//-------------------------------------------------------------------------------------------------
void INI::parseLookupList( INI* ini, void * /*instance*/, void *store, const void* userData )
{
	ConstLookupListRecArray lookupList = (ConstLookupListRecArray)userData;
	*(Int *)store = scanLookupList(ini->getNextToken(), lookupList);
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// PRIVATE FUNCTIONS //////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////


//-------------------------------------------------------------------------------------------------
void MultiIniFieldParse::add(const FieldParse* f, UnsignedInt e)
{
	if (m_count < MAX_MULTI_FIELDS)
	{
		m_fieldParse[m_count] = f;
		m_extraOffset[m_count] = e;
		++m_count;
	}
	else
	{
		DEBUG_CRASH(("too many multi-fields in INI::initFromINIMultiProc"));
		throw ERROR_BUG;
	}
}

//-------------------------------------------------------------------------------------------------
void INI::initFromINI( void *what, const FieldParse* parseTable )
{
	MultiIniFieldParse p;
	p.add(parseTable);
	initFromINIMulti(what, p);
}

//-------------------------------------------------------------------------------------------------
void INI::initFromINIMultiProc( void *what, BuildMultiIniFieldProc proc )
{
	MultiIniFieldParse p;
	(*proc)(p);
	initFromINIMulti(what, p);
}

//-------------------------------------------------------------------------------------------------
void INI::initFromINIMulti( void *what, const MultiIniFieldParse& parseTableList )
{
	Bool done = FALSE;

	if( what == nullptr )
	{
		DEBUG_CRASH( ("INI::initFromINI - Invalid parameters supplied!") );
		throw INI_INVALID_PARAMS;
	}

	// read each of the data fields
	while( !done )
	{

		// read next line
		readLine();

		// check for end token
		const char* field = strtok( m_buffer, INI::getSeps() );
		if( field )
		{

			if( stricmp( field, getEndToken() ) == 0 )
			{
				done = TRUE;
			}
			else
			{
				Bool found = false;
				for (int ptIdx = 0; ptIdx < parseTableList.getCount(); ++ptIdx)
				{
					int offset = 0;
					const void* userData = nullptr;
					INIFieldParseProc parse = findFieldParse(parseTableList.getNthFieldParse(ptIdx), field, offset, userData);
					if (parse)
					{
						// parse this block and check for parse errors
						try {

						(*parse)( this, what, (char *)what + offset + parseTableList.getNthExtraOffset(ptIdx), userData );

						// GeneralsX @bugfix Copilot 20/09/2026 Do not replace nested field errors with an enclosing module.
						} catch (const INIException&) {
							throw;
						} catch (...) {
							DEBUG_CRASH( ("[LINE: %d - FILE: '%s'] Error reading field '%s' of block '%s'",
																 INI::getLineNum(), INI::getFilename().str(), field, m_curBlockStart) );


							char buff[1024];
							snprintf(buff, ARRAY_SIZE(buff), "[LINE: %d - FILE: '%s'] Error reading field '%s'\n",
								INI::getLineNum(), INI::getFilename().str(), field);
							throw INIException(buff);
						}

						found = true;
						break;

					}
				}

				if (!found)
				{
					DEBUG_CRASH( ("[LINE: %d - FILE: '%s'] Unknown field '%s' in block '%s'",
														 INI::getLineNum(), INI::getFilename().str(), field, m_curBlockStart) );
				}

			}

		}

		// sanity check for reaching end of file with no closing end token
		if( done == FALSE && INI::isEOF() == TRUE )
		{

			done = TRUE;
			DEBUG_CRASH( ("Error parsing block '%s', in INI file '%s'.  Missing '%s' token",
												 m_curBlockStart, getFilename().str(), getEndToken()) );
			throw INI_MISSING_END_TOKEN;

		}

	}

}

//-------------------------------------------------------------------------------------------------
/*static*/ const char* INI::getNextToken(const char* seps)
{
	const char *token = ::strtok(nullptr, seps);
	if (!token)
		throw INI_INVALID_DATA;
	return token;
}

//-------------------------------------------------------------------------------------------------
/*static*/ const char* INI::getNextTokenOrNull(const char* seps)
{
	const char *token = ::strtok(nullptr, seps);
	return token;
}

//-------------------------------------------------------------------------------------------------
/*static*/ ScienceType INI::scanScience(const char* token)
{
	return TheScienceStore->friend_lookupScience( token );
}

#if USE_STD_FROM_CHARS_PARSING

template <typename Type>
Type scanType(const char* tokenString)
{
	std::string_view token(tokenString);
	DEBUG_ASSERTCRASH(!token.empty(), ("token is not expected to be empty"));

	// Unlike sscanf, std::from_chars cannot parse "+".
	// Skip it for from_chars while preserving the original token for the fallback.
	const size_t tokenOffset = (token[0] == '+');
	if (tokenOffset && token.size() > 1 && token[1] == '-')
		throw INI_INVALID_DATA;

	if constexpr (std::is_floating_point_v<Type>)
	{
#if defined(__APPLE__)
		const std::string tokenStr(token.data() + tokenOffset, token.size() - tokenOffset);
		char *end = nullptr;
		errno = 0;
		const double result = std::strtod(tokenStr.c_str(), &end);

		if (end == tokenStr.c_str())
		{
			throw INI_INVALID_DATA;
		}

		if (!std::isfinite(result) && errno != ERANGE)
		{
			throw INI_INVALID_DATA;
		}

		const double maxValue = static_cast<double>(std::numeric_limits<Type>::max());
		if (result > maxValue)
		{
			fprintf(stderr, "[INI] Numeric token '%.*s' out of range, saturating to limit\n",
				static_cast<int>(token.size()), token.data());
			fflush(stderr);
			return std::numeric_limits<Type>::max();
		}
		if (result < -maxValue)
		{
			fprintf(stderr, "[INI] Numeric token '%.*s' out of range, saturating to limit\n",
				static_cast<int>(token.size()), token.data());
			fflush(stderr);
			return -std::numeric_limits<Type>::max();
		}

		return static_cast<Type>(result);
#else
		Type result{};
		const auto [ptr, ec] = std::from_chars(token.data() + tokenOffset, token.data() + token.size(), result);

		if (ec != std::errc{})
		{
			if (ec == std::errc::result_out_of_range)
			{
				char* end;
				const Type value = std::strtof(tokenString, &end);
				if (end == tokenString)
					throw INI_INVALID_DATA;
				return value;
			}

			fprintf(stderr, "[INI] Cannot parse numeric token '%.*s': %s\n",
				static_cast<int>(token.size()), token.data(),
				"invalid number");
			fflush(stderr);
			throw INI_INVALID_DATA;
		}

		if (!std::isfinite(result))
		{
			throw INI_INVALID_DATA;
		}

		return result;
#endif
	}
	else
	{
		// TheSuperHackers @info std::from_chars cannot parse "-1" as uint32 so the result needs to be int64 for integers.
		std::conditional_t<std::is_integral_v<Type>, Int64, Type> result{};
		const auto [ptr, ec] = std::from_chars(token.data() + tokenOffset, token.data() + token.size(), result);

		if (ec != std::errc{})
		{
			if (ec == std::errc::result_out_of_range)
			{
				// GeneralsX @bugfix fbraz 25/09/2026 Saturate overflowing integers to field limits to match retail behavior for mods (#297).
				fprintf(stderr, "[INI] Numeric token '%.*s' out of range, saturating to limit\n",
					static_cast<int>(token.size()), token.data());
				fflush(stderr);

				if (!token.empty() && token[tokenOffset] == '-')
				{
					return std::numeric_limits<Type>::min();
				}
				return std::numeric_limits<Type>::max();
			}

			// GeneralsX @bugfix Copilot 20/09/2026 Keep numeric conversion failures visible in release builds.
			fprintf(stderr, "[INI] Cannot parse numeric token '%.*s': %s\n",
				static_cast<int>(token.size()), token.data(),
				"invalid number");
			fflush(stderr);
			throw INI_INVALID_DATA;
		}

		if constexpr (std::is_unsigned_v<Type>)
		{
			// For unsigned integers, negative values like -1 are sentinels (~0U) and should wrap via static_cast.
			// Positive values exceeding Type's range saturate to max.
			if (result > static_cast<Int64>(std::numeric_limits<Type>::max()))
			{
				// GeneralsX @bugfix fbraz 25/09/2026 Saturate overflowing integers to field limits to match retail behavior for mods (#297).
				fprintf(stderr, "[INI] Numeric token '%.*s' exceeds max value, saturating to limit\n",
					static_cast<int>(token.size()), token.data());
				fflush(stderr);
				return std::numeric_limits<Type>::max();
			}
		}
		else if constexpr (std::is_signed_v<Type>)
		{
			if (result > static_cast<Int64>(std::numeric_limits<Type>::max()))
			{
				// GeneralsX @bugfix fbraz 25/09/2026 Saturate overflowing integers to field limits to match retail behavior for mods (#297).
				fprintf(stderr, "[INI] Numeric token '%.*s' exceeds max value, saturating to limit\n",
					static_cast<int>(token.size()), token.data());
				fflush(stderr);
				return std::numeric_limits<Type>::max();
			}
			if (result < static_cast<Int64>(std::numeric_limits<Type>::min()))
			{
				// GeneralsX @bugfix fbraz 25/09/2026 Saturate overflowing integers to field limits to match retail behavior for mods (#297).
				fprintf(stderr, "[INI] Numeric token '%.*s' exceeds min value, saturating to limit\n",
					static_cast<int>(token.size()), token.data());
				fflush(stderr);
				return std::numeric_limits<Type>::min();
			}
		}

		return static_cast<Type>(result);
	}
}

#endif

//-------------------------------------------------------------------------------------------------
/*static*/ Int INI::scanInt(const char* token)
{
#if USE_STD_FROM_CHARS_PARSING == 1
	return scanType<Int>(token);
#else
	Int value;
	if (sscanf( token, "%d", &value ) != 1)
		throw INI_INVALID_DATA;

#if USE_STD_FROM_CHARS_PARSING == -1
	if (value != scanType<Int>(token))
		throw INI_INVALID_DATA;
#endif

	return value;
#endif
}

//-------------------------------------------------------------------------------------------------
/*static*/ UnsignedInt INI::scanUnsignedInt(const char* token)
{
#if USE_STD_FROM_CHARS_PARSING == 1
	return scanType<UnsignedInt>(token);
#else
	UnsignedInt value;
	if (sscanf( token, "%u", &value ) != 1)	// unsigned int is %u, not %d
		throw INI_INVALID_DATA;

#if USE_STD_FROM_CHARS_PARSING == -1
	if (value != scanType<UnsignedInt>(token))
		throw INI_INVALID_DATA;
#endif

	return value;
#endif
}

//-------------------------------------------------------------------------------------------------
/*static*/ Real INI::scanReal(const char* token)
{
#if USE_STD_FROM_CHARS_PARSING == 1
	return scanType<Real>(token);
#else
	Real value;
	if (sscanf( token, "%f", &value ) != 1)
		throw INI_INVALID_DATA;

#if USE_STD_FROM_CHARS_PARSING == -1
	if (value != scanType<Real>(token))
		throw INI_INVALID_DATA;
#endif

	return value;
#endif
}

//-------------------------------------------------------------------------------------------------
/*static*/ Real INI::scanPercentToReal(const char* token)
{
	return scanReal(token) / 100.0f;
}

//-------------------------------------------------------------------------------------------------
/*static*/ Int INI::scanIndexList(const char* token, ConstCharPtrArray nameList)
{
	if( nameList == nullptr || nameList[ 0 ] == nullptr )
	{

		DEBUG_CRASH( ("INTERNAL ERROR! scanIndexList, invalid name list") );
		throw INI_INVALID_NAME_LIST;

	}

	// search for matching name
	Int count = 0;
	for(ConstCharPtrArray name = nameList; *name; name++, count++ )
	{
		if( stricmp( *name, token ) == 0 )
		{
			return count;
		}
	}

	DEBUG_CRASH(("token %s is not a valid member of the index list",token));
	throw INI_INVALID_DATA;
	return 0;	// never executed, but keeps compiler happy

}
//-------------------------------------------------------------------------------------------------
/*static*/ Int INI::scanLookupList(const char* token, ConstLookupListRecArray lookupList)
{
	if( lookupList == nullptr || lookupList[ 0 ].name == nullptr )
	{
		DEBUG_CRASH( ("INTERNAL ERROR! scanLookupList, invalid name list") );
		throw INI_INVALID_NAME_LIST;
	}

	// search for matching name
	for( const LookupListRec* lookup = &lookupList[0]; lookup->name; lookup++ )
	{
		if( stricmp( lookup->name, token ) == 0 )
		{
			return lookup->value;
		}
	}

	DEBUG_CRASH(("token %s is not a valid member of the lookup list",token));
	throw INI_INVALID_DATA;
	return 0;	// never executed, but keeps compiler happy

}

//-------------------------------------------------------------------------------------------------
const char* INI::getNextSubToken(const char* expected)
{
	const char* token = getNextToken(getSepsColon());
	if (stricmp(token, expected) != 0)
		throw INI_INVALID_DATA;
	return getNextToken(getSepsColon());
}

//-------------------------------------------------------------------------------------------------
/**
 * Parse a "random variable".
 * The format is "FIELD = low high [distribution]".
 */
void INI::parseGameClientRandomVariable( INI* ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	GameClientRandomVariable *var = static_cast<GameClientRandomVariable *>(store);

	const char* token;

	token = ini->getNextToken();
	Real low = INI::scanReal(token);

	token = ini->getNextToken();
	Real high = INI::scanReal(token);

	// if omitted, assume uniform
	GameClientRandomVariable::DistributionType type = GameClientRandomVariable::UNIFORM;
	token = ini->getNextTokenOrNull();
	if (token)
		type = (GameClientRandomVariable::DistributionType)INI::scanIndexList(token, GameClientRandomVariable::DistributionTypeNames);

	// set the range of the random variable
	var->setRange( low, high, type );
}

//-------------------------------------------------------------------------------------------------
// parse a duration in msec and convert to duration in frames
void INI::parseDurationReal( INI *ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	Real val = scanReal(ini->getNextToken());
	*(Real *)store = ConvertDurationFromMsecsToFrames(val);
}

//-------------------------------------------------------------------------------------------------
// parse a duration in msec and convert to duration in integral number of frames, (unsignedint) rounding UP
void INI::parseDurationUnsignedInt( INI *ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	UnsignedInt val = scanUnsignedInt(ini->getNextToken());
	*(UnsignedInt *)store = (UnsignedInt)WWMath::Ceilf(ConvertDurationFromMsecsToFrames((Real)val));
}

// ------------------------------------------------------------------------------------------------
// parse a duration in msec and convert to duration in integral number of frames, (unsignedshort) rounding UP
void INI::parseDurationUnsignedShort( INI *ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	UnsignedInt val = scanUnsignedInt(ini->getNextToken());
	*(UnsignedShort *)store = (UnsignedShort)WWMath::Ceilf(ConvertDurationFromMsecsToFrames((Real)val));
}

//-------------------------------------------------------------------------------------------------
// parse acceleration in (dist/sec) and convert to (dist/frame)
void INI::parseVelocityReal( INI *ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	const char *token = ini->getNextToken();
	Real val = scanReal(token);
	*(Real *)store = ConvertVelocityInSecsToFrames(val);
}

//-------------------------------------------------------------------------------------------------
// parse acceleration in (dist/sec^2) and convert to (dist/frame^2)
void INI::parseAccelerationReal( INI *ini, void * /*instance*/, void *store, const void* /*userData*/ )
{
	const char *token = ini->getNextToken();
	Real val = scanReal(token);
	*(Real *)store = ConvertAccelerationInSecsToFrames(val);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void INI::parseVeterancyLevelFlags(INI* ini, void* /*instance*/, void* store, const void* /*userData*/)
{
	VeterancyLevelFlags flags = VETERANCY_LEVEL_FLAGS_ALL;
	for (const char* token = ini->getNextToken(); token; token = ini->getNextTokenOrNull())
	{
		if (stricmp(token, "ALL") == 0)
		{
			flags = VETERANCY_LEVEL_FLAGS_ALL;
			continue;
		}
		else if (stricmp(token, "NONE") == 0)
		{
			flags = VETERANCY_LEVEL_FLAGS_NONE;
			continue;
		}
		else if (token[0] == '+')
		{
			VeterancyLevel dt = (VeterancyLevel)INI::scanIndexList(token+1, TheVeterancyNames);
			flags = setVeterancyLevelFlag(flags, dt);
			continue;
		}
		else if (token[0] == '-')
		{
			VeterancyLevel dt = (VeterancyLevel)INI::scanIndexList(token+1, TheVeterancyNames);
			flags = clearVeterancyLevelFlag(flags, dt);
			continue;
		}
		else
		{
			throw INI_UNKNOWN_TOKEN;
		}
	}
	*(VeterancyLevelFlags*)store = flags;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void INI::parseSoundsList( INI* ini, void *instance, void *store, const void* /*userData*/ )
{
	std::vector<AsciiString> *vec = (std::vector<AsciiString>*) store;
	vec->clear();

	constexpr const char* Seps = " \t,=";
	for (const char* token = ini->getNextTokenOrNull(Seps); token; token = ini->getNextTokenOrNull(Seps))
	{
		vec->push_back(token);
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void INI::parseDamageTypeFlags(INI* ini, void* /*instance*/, void* store, const void* /*userData*/)
{
	DamageTypeFlags flags = DAMAGE_TYPE_FLAGS_ALL;

	for (const char* token = ini->getNextToken(); token; token = ini->getNextTokenOrNull())
	{
		if (stricmp(token, "ALL") == 0)
		{
			flags = DAMAGE_TYPE_FLAGS_ALL;
			continue;
		}
		if (stricmp(token, "NONE") == 0)
		{
			flags = DAMAGE_TYPE_FLAGS_NONE;
			continue;
		}
		if (token[0] == '+')
		{
			DamageType dt = (DamageType)scanIndexList(token+1, DamageTypeFlags::getBitNames());
			flags = setDamageTypeFlag(flags, dt);
			continue;
		}
		if (token[0] == '-')
		{
			DamageType dt = (DamageType)scanIndexList(token+1, DamageTypeFlags::getBitNames());
			flags = clearDamageTypeFlag(flags, dt);
			continue;
		}
		throw INI_UNKNOWN_TOKEN;
	}
	*(DamageTypeFlags*)store = flags;
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void INI::parseDeathTypeFlags(INI* ini, void* /*instance*/, void* store, const void* /*userData*/)
{
	DeathTypeFlags flags = DEATH_TYPE_FLAGS_ALL;
	for (const char* token = ini->getNextToken(); token; token = ini->getNextTokenOrNull())
	{
		if (stricmp(token, "ALL") == 0)
		{
			flags = DEATH_TYPE_FLAGS_ALL;
			continue;
		}
		if (stricmp(token, "NONE") == 0)
		{
			flags = DEATH_TYPE_FLAGS_NONE;
			continue;
		}
		if (token[0] == '+')
		{
			DeathType dt = (DeathType)INI::scanIndexList(token+1, TheDeathNames);
			flags = setDeathTypeFlag(flags, dt);
			continue;
		}
		if (token[0] == '-')
		{
			DeathType dt = (DeathType)INI::scanIndexList(token+1, TheDeathNames);
			flags = clearDeathTypeFlag(flags, dt);
			continue;
		}
		throw INI_UNKNOWN_TOKEN;
	}
	*(DeathTypeFlags*)store = flags;
}

//-------------------------------------------------------------------------------------------------
// parse the line and return whether the given line is a Block declaration of the form
// [whitespace] blockType [whitespace] blockName [EOL]
// both blockType and blockName are case insensitive
Bool INI::isDeclarationOfType( AsciiString blockType, AsciiString blockName, char *bufferToCheck )
{
	if (!bufferToCheck || blockType.isEmpty() || blockName.isEmpty())
		return false;

	const char* tempBuff = bufferToCheck;

	while (isspace(*tempBuff))
		++tempBuff;

	const int blockTypeLength = blockType.getLength();
	if (strnicmp(tempBuff, blockType.str(), blockTypeLength) != 0)
		return false;

	tempBuff += blockTypeLength;

	if (!isspace(*tempBuff++))
		return false;

	while (isspace(*tempBuff))
		++tempBuff;

	const int blockNameLength = blockName.getLength();
	if (strnicmp(tempBuff, blockName.str(), blockNameLength) != 0)
		return false;

	tempBuff += blockNameLength;

	while (isspace(*tempBuff))
		++tempBuff;

	if (*tempBuff != '\0')
		return false;

	return true;
}

//-------------------------------------------------------------------------------------------------
// parse the line and return whether the given line is a Block declaration of the form
// [whitespace] end [EOL]
Bool INI::isEndOfBlock( char *bufferToCheck )
{
	Bool retVal = true;
	if (!bufferToCheck) {
		return false;
	}

	// DO NOT RETURN EARLY FROM THIS FUNCTION (beyond this point)
	// we have to restore the bufferToCheck to its previous state before returning, so
	// it is important to get through all the checks.

	static const char* endString = "End";
	int endStringLength = strlen(endString);
	char restoreChar;
	char *tempBuff = bufferToCheck;


	while (isspace(*tempBuff)) {
		++tempBuff;
	}

	if (strlen(tempBuff) > endStringLength) {
		restoreChar = tempBuff[endStringLength];
		tempBuff[endStringLength] = 0;

		if (stricmp(endString, tempBuff) != 0) {
			retVal = false;
		}

		tempBuff[endStringLength] = restoreChar;
		tempBuff = tempBuff + endStringLength;
	} else {
		retVal = false;
	}

	while (*tempBuff && retVal) {
		retVal = isspace(*tempBuff);
		++tempBuff;
	}

	return retVal;
}
```


---

## 4. Architect Notes & Exact C++ Determinism Quirks

1. **Exact Imports & Existing Helpers Available in `zh_common`:**
   - `use crate::math::{gm_ceilf, Coord2D, Coord3D, ICoord2D, RGBAColorInt, RGBColor, PI};`
   - `use crate::random_value::DistributionType;`
   - `use crate::types::{Bool, Byte, Int, Real, Short, UnsignedByte, UnsignedInt, UnsignedShort, FALSE, TRUE};`
   - `use crate::xfer::bit_flags::BitFlags;`
   - `use crate::xfer::Xfer;`
   - Skeleton target already exists at [`crates/zh_common/src/ini/mod.rs`](../../crates/zh_common/src/ini/mod.rs) with `#[ignore]`d Golden Vector test at [`crates/zh_common/tests/golden_pkt087.rs`](../../crates/zh_common/tests/golden_pkt087.rs).

2. **Exact Conversion Constants & Rounding (`GameCommon.h:64-103`, `INI.cpp:1969-2007`):**
   - `LOGICFRAMES_PER_SECOND = 30`, `MSEC_PER_SECOND = 1000`.
   - `LOGICFRAMES_PER_MSEC_REAL = 30.0_f32 / 1000.0_f32`.
   - `SECONDS_PER_LOGICFRAME_REAL = 1.0_f32 / 30.0_f32`.
   - `RADS_PER_DEGREE = crate::math::PI / 180.0_f32` (uses `3.14159265359_f32`, NOT `std::f32::consts::PI`).
   - `parse_duration_unsigned_int` and `parse_duration_unsigned_short` MUST use `crate::math::gm_ceilf(convert_duration_from_msecs_to_frames(val as Real))` (`DET003` forbids `.ceil()`).
   - `game_make_color(r, g, b, a) = ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)`.

3. **Line Reader, Comments, Control Characters & `XferCrc` Hashing (`INI.cpp:510-577`):**
   - Stops at `\n` (consuming `\n`, not storing it) or after reading `INI_MAX_CHARS_PER_LINE` (`1028`) bytes or at EOF (`end_of_file = true`).
   - Truncates the line at the first `;` (`; comment` sets terminator at `*p = 0`), **but continues consuming bytes from the read buffer until `\n` or EOF**, and overwrites any remaining control chars (`1..=31`, i.e., `\r` or `0x0b`) before `;` with `' '` (`0x20`). Note that high-bit bytes (`>= 128`, signed `char < 0`) are **not** replaced with `' '` because C++ checks `*p > 0 && *p < 32` (`i8`!).
   - After finishing the line, if `xfer` is `Some`, calls `xfer.xfer_implementation(line_bytes)` on the NUL-terminated buffer slice (`strlen(buffer)` bytes, even if empty `""`). Note `Xfer::xfer_user` / `xfer_implementation` hashes `0` bytes cleanly (no-op when len == 0).

4. **`strtok` Tokenizer & `get_next_ascii_string` / `get_next_quoted_ascii_string` (`INI.cpp:804-894, 1649-1662`):**
   - Replicate C `strtok(buf, seps)` cursor semantics on `line_bytes` cleanly (skip leading separator bytes, take bytes until next separator or `0`, advance cursor; note separator set can vary per call: `SEPS = " \n\r\t="`, `SEPS_PERCENT = " \n\r\t=%"`, `SEPS_COLON = " \n\r\t=:"`, `SEPS_QUOTE = "\"\n="`).
   - Note that in C++, `getNextAsciiString()` and `getNextQuotedAsciiString()` inspect `token[0] == '"'` and consume up to `SEPS_QUOTE`, matching `INI.cpp:804-894` (for `"Two Words"`, first token is `"\"Two"`, remainder is `"Words"`, `len("Words") > 1` so `" "` is prepended -> `"Two Words"`).

5. **`is_end_of_block` Length > 3 Quirk (`INI.cpp:2173-2214`):**
   - After skipping leading ASCII whitespace, `isEndOfBlock` checks `strlen(tempBuff) > 3` (`len > 3`, NOT `>= 3`!), case-insensitive `"End"` prefix, and remaining characters all whitespace! Consequently bare `"End"` (length 3, no trailing `\r`/` ` /`\n`) returns `false`, whereas `"End\r"` or `"  End  "` returns `true`. Replicate this exact check!

6. **`scan_int` / `scan_unsigned_int` / `scan_real` (`INI.cpp:1672-1873`, Zero Hour `USE_STD_FROM_CHARS_PARSING=1`):**
   - Rejects empty strings and `"+-..."` (`token[0] == '+' && token.get(1) == Some('-')`). Skips a single leading `'+'`.
   - Parses leading numeric **prefix** only (`"123abc"` -> `Ok(123)`, `"+45px"` -> `Ok(45)`, `"12.5%"` -> `Ok(12.5)`) instead of Rust's strict whole-string `.parse()`.
   - Integer overflow saturates to `i32::MAX` / `i32::MIN` / `u32::MAX` (after parsing `i64`; note `"-1"` in `scan_unsigned_int` parses `-1_i64` and wraps to `u32::MAX` = `4294967295` per `INI.cpp:1780-1790`).
   - `scan_real` parses the float prefix, rejects non-finite `NaN` / `Inf`, and returns `Real`. If using `.parse::<f64>()` inside `scan_real` on an already-validated finite decimal slice, annotate that single line with `// determinism-ok: pre-validated decimal prefix for C++ from_chars/strtod parity` so Gate V1 (`DET011`) passes cleanly.

7. **BitFlags & Zero Hour 38-Bit `DamageTypeFlags` (`INI.cpp:977-1060, 2011-2127`):**
   - `parse_bit_string32` preserves existing bits when tokens start with `'+'`/`'-'`, clears existing bits on first normal token, and **errors (`IniError::InvalidNameList`) if normal tokens and `'+'`/`'-'` tokens are mixed** or `"NONE"` follows earlier tokens.
   - `parse_veterancy_level_flags`, `parse_damage_type_flags`, and `parse_death_type_flags` start from `*_ALL` (not `NONE`), overwrite with `*_ALL`/`*_NONE` on `"ALL"`/`"NONE"`, apply `+NAME`/`-NAME`, and throw `IniError::UnknownToken` on tokens not starting with `+`/`-` (unless `"ALL"`/`"NONE"`).
   - Zero Hour `DAMAGE_TYPE_NAMES` has **38** entries (`BitFlags<38>`, omitting `#if RTS_GENERALS` `FLESHY_SNIPER`). `DEATH_TYPE_NAMES` has **21** entries (`DEATH_NORMAL`..`DEATH_POISONED_GAMMA`).
