# Translation Packet `PKT-397`: `ThingFactory`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-397`
- **Work Package:** `WP-5.5` (Phase 5)
- **Complexity & Assigned Tier:** Class `C4` -> `tier:3-architect`
- **Environment Access:** `env:hybrid-ci`
- **FFI Seam Pattern:** Core Spine
- **Allowed Rust Output File:** `crates/zh_logic/src/object/thing_factory.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (22):**
  - `Common/AsciiString.h`
  - `Common/FileSystem.h`
  - `Common/GameAudio.h`
  - `Common/GameMemory.h`
  - `Common/INI.h`
  - `Common/MapObject.h`
  - `Common/ModuleFactory.h`
  - `Common/Player.h`
  - `Common/PlayerList.h`
  - `Common/ProductionPrerequisite.h`
  - `Common/RandomValue.h`
  - `Common/STLTypedefs.h`
  - `Common/SubsystemInterface.h`
  - `Common/ThingFactory.h`
  - `Common/ThingTemplate.h`
  - `GameClient/Drawable.h`
  - `GameClient/GameClient.h`
  - `GameLogic/GameLogic.h`
  - `GameLogic/Module/CreateModule.h`
  - `GameLogic/Object.h`
  - `GameLogic/PartitionManager.h`
  - `PreRTS.h`
- **`Snapshot::crc` / `Snapshot::xfer` Calls:** None in module.
- **PRNG Calls (1 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValue(0, asv.size()-1)`

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

### `GeneralsMD/Code/GameEngine/Include/Common/ThingFactory.h`
```cpp
// FILE: ThingFactory.h ///////////////////////////////////////////////////////////////////////////
// Created: Colin Day, April 2001
// Desc:		This is how we go and make our things, we make our things, we make our things!
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////

#include "Common/STLTypedefs.h"
#include "Common/SubsystemInterface.h"
#include "Common/GameMemory.h"
#include "Common/AsciiString.h"
#include "GameClient/Drawable.h"
#include "GameLogic/Object.h"

class ThingTemplate;
class Object;
class Drawable;
class INI;

typedef std::hash_map<AsciiString, ThingTemplate*, rts::hash<AsciiString>, rts::equal_to<AsciiString>/**/> ThingTemplateHashMap;
typedef ThingTemplateHashMap::iterator ThingTemplateHashMapIt;
//-------------------------------------------------------------------------------------------------
/** Implementation of the thing manager interface singleton */
//-------------------------------------------------------------------------------------------------
class ThingFactory : public SubsystemInterface
{
public:

	ThingFactory();
	virtual ~ThingFactory() override;

	// From the subsystem interface =================================================================
	virtual void init() override;
	virtual void postProcessLoad() override;
	virtual void reset() override;
	virtual void update() override;
	//===============================================================================================

	/// create a new template with name 'name' and add to template list
	ThingTemplate *newTemplate( const AsciiString& name );

	// Get the first template in the database. Does not resolve the final override.
	const ThingTemplate *firstTemplate() { return m_firstTemplate; }

	/**
		Get a template by database name. Returns null if not found.
		Does not resolve the final override.
	*/
	const ThingTemplate *findTemplate( const AsciiString& name, Bool check = TRUE ) { return findTemplateInternal( name, check ); }

	/**
		Get a template by ID. Returns null if not found.
		Does not resolve the final override.
	*/
	const ThingTemplate *findByTemplateID( UnsignedShort id );

	/** request a new object using the given template.
		this will throw an exception on failure; it will never return null.
	*/
	Object *newObject( const ThingTemplate *tmplate, Team *team, ObjectStatusMaskType statusMask = OBJECT_STATUS_MASK_NONE );

	/** request a new drawable using the given template.
		this will throw an exception on failure; it will never return null.
	*/
	Drawable *newDrawable(const ThingTemplate *tmplate, DrawableStatusBits statusBits = DRAWABLE_STATUS_DEFAULT );

	static void parseObjectDefinition( INI* ini, const AsciiString& name, const AsciiString& reskinFrom );

private:

	/// free all template database data
	void freeDatabase();

	void addTemplate( ThingTemplate *thing );		///< add the template to the DB

	/**
		create a new template with name 'name', do *NOT* add to template, but instead
		add as m_override of template 'template'

		Note, this is private, and should always remain so. It should never, ever, ever
		be called outside of ini parsing code, because if you do, you will tempted to make
		changes to the override at runtime, and end up with changes that can't be saved in savegames,
		since ThingTemplates aren't saved (and should not be). (srj)
	*/
	ThingTemplate* newOverride( ThingTemplate *thingTemplate );


	/**
		This now does a search through the hashmap, and returns the associated thing template
		This should speed up load times dramatically, as we've been wasting lots of time in this
		O(N^2) insertion.

		NOTE: this is protected since it returns a NON-CONST template, and
		folks outside of the template system itself shouldn't get access...
	*/
	ThingTemplate *findTemplateInternal( const AsciiString& name, Bool check = TRUE );

	ThingTemplate					*m_firstTemplate;			///< head of linked list
	UnsignedShort					m_nextTemplateID;			///< next available ID for templates

	ThingTemplateHashMap	m_templateHashMap;		///< all thing templates, for fast lookup.

};

// EXTERN /////////////////////////////////////////////////////////////////////////////////////////
extern ThingFactory *TheThingFactory;  ///< the template singleton
```

### `GeneralsMD/Code/GameEngine/Source/Common/Thing/ThingFactory.cpp`
```cpp
// FILE: ThingFactory.cpp /////////////////////////////////////////////////////////////////////////
// Created:   Colin Day, April 2001
// Desc:		This is how we go and make our things, we make our things, we make our things!
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/FileSystem.h"
#include "Common/GameAudio.h"
#include "Common/MapObject.h"
#include "Common/ModuleFactory.h"
#include "Common/RandomValue.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Module/CreateModule.h"
#include "Common/ProductionPrerequisite.h"
#include "GameClient/GameClient.h"
#include "GameClient/Drawable.h"
#include "Common/INI.h"


enum { TEMPLATE_HASH_SIZE = 12288 };

// PUBLIC DATA ////////////////////////////////////////////////////////////////////////////////////
ThingFactory *TheThingFactory = nullptr;  ///< Thing manager singleton declaration

// STATIC FUNCTIONS ///////////////////////////////////////////////////////////////////////////////

///////////////////////////////////////////////////////////////////////////////////////////////////
// PRIVATE METHODS
///////////////////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
/** Free all data loaded into this template database */
//-------------------------------------------------------------------------------------------------
void ThingFactory::freeDatabase()
{
	while (m_firstTemplate)
	{
		ThingTemplate* tmpl = m_firstTemplate;
		m_firstTemplate = m_firstTemplate->friend_getNextTemplate();
		deleteInstance(tmpl);
	}

	m_templateHashMap.clear();

}

//-------------------------------------------------------------------------------------------------
/** add the thing template passed in, into the database */
//-------------------------------------------------------------------------------------------------
void ThingFactory::addTemplate( ThingTemplate *tmplate )
{
	ThingTemplateHashMapIt tIt = m_templateHashMap.find(tmplate->getName());

	if (tIt != m_templateHashMap.end()) {
		DEBUG_CRASH(("Duplicate Thing Template name found: %s", tmplate->getName().str()));
	}

	// Link it to the list
	tmplate->friend_setNextTemplate(m_firstTemplate);
	m_firstTemplate = tmplate;

	// Add it to the hash table.
	m_templateHashMap[tmplate->getName()] = tmplate;
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// PUBLIC METHODS
///////////////////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ThingFactory::ThingFactory()
{
	m_firstTemplate = nullptr;
	m_nextTemplateID = 1;	// not zero!

#ifdef USING_STLPORT
	m_templateHashMap.resize( TEMPLATE_HASH_SIZE );
#else
	m_templateHashMap.reserve( TEMPLATE_HASH_SIZE );
#endif
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
ThingFactory::~ThingFactory()
{

	// free all the template data
	freeDatabase();

}

//-------------------------------------------------------------------------------------------------
/** Create a new template with name 'name' and add to our template list */
//-------------------------------------------------------------------------------------------------
ThingTemplate *ThingFactory::newTemplate( const AsciiString& name )
{
	ThingTemplate *newTemplate;

	// allocate template
	newTemplate = newInstance(ThingTemplate);

	// if the default template is present, get it and copy over any data to the new template
	const ThingTemplate *defaultT = findTemplate( "DefaultThingTemplate", FALSE );
	if( defaultT )
	{

		// copy over static data
		*newTemplate = *defaultT;
		newTemplate->setCopiedFromDefault();

	}

	// give template a unique identifier
	newTemplate->friend_setTemplateID( m_nextTemplateID++ );
	DEBUG_ASSERTCRASH( m_nextTemplateID != 0, ("m_nextTemplateID wrapped to zero") );

	// assign name
	newTemplate->friend_setTemplateName( name );

	// add to list
	addTemplate( newTemplate );

	// return the newly created template
	return newTemplate;

}

//-------------------------------------------------------------------------------------------------
/** Create newTemplate, copy data from final override of 'thingTemplate' to the newly created one,
	* and add newTemplate as the m_override of that final override.  NOTE that newTemplate
	* is *NOT* added to master template list, it is a hidden place to store
	* override values for 'thingTemplate' */
//-------------------------------------------------------------------------------------------------
ThingTemplate* ThingFactory::newOverride( ThingTemplate *thingTemplate )
{

	// sanity
	DEBUG_ASSERTCRASH( thingTemplate, ("newOverride(): null 'parent' thing template") );

	// sanity just for debugging, the weapon must be in the master list to do overrides
	DEBUG_ASSERTCRASH( findTemplate( thingTemplate->getName() ) != nullptr,
										 ("newOverride(): Thing template '%s' not in master list",
										 thingTemplate->getName().str()) );

	// find final override of the 'parent' template
	ThingTemplate *child = (ThingTemplate*) thingTemplate->friend_getFinalOverride();

	// allocate new template
	ThingTemplate *newTemplate = newInstance(ThingTemplate);

	// copy data from final override to 'newTemplate' as a set of initial default values
	*newTemplate = *child;
	newTemplate->setCopiedFromDefault();

	newTemplate->markAsOverride();
	child->setNextOverride(newTemplate);

	// return the newly created override for us to set values with etc
	return newTemplate;

}

//-------------------------------------------------------------------------------------------------
/** Init */
//-------------------------------------------------------------------------------------------------
void ThingFactory::init()
{

}

//-------------------------------------------------------------------------------------------------
/** Reset */
//-------------------------------------------------------------------------------------------------
void ThingFactory::reset()
{
	ThingTemplate *t;
	// go through all templates and delete any overrides
	for( t = m_firstTemplate; t; /* empty */ )
	{
		Bool possibleAdjustment = FALSE;
		// t itself can be deleted if it is something created for this map only. Therefore,
		// we need to store what the next item is so that we don't orphan a bunch of templates.
		ThingTemplate *nextT = t->friend_getNextTemplate();
		DEBUG_ASSERTCRASH(!nextT || t->getTemplateID() == nextT->getTemplateID() + 1, ("Next template ID is unexpected"));

		if (t == m_firstTemplate) {
			possibleAdjustment = TRUE;
		}

		// if stillValid is nullptr after we delete the overrides, then this template was created for
		// this map only. If it also happens to be m_firstTemplate, then we need to update m_firstTemplate
		// as well. Finally, if it was only created for this map, we need to remove the name from the
		// hash map, to prevent any crashes.

		AsciiString templateName = t->getName();

		Overridable *stillValid = t->deleteOverrides();
		if (stillValid == nullptr && possibleAdjustment) {
			m_firstTemplate = nextT;
		}

		if (stillValid == nullptr) {
			// Also needs to be removed from the Hash map.
			m_templateHashMap.erase(templateName);
		}

		t = nextT;
	}

	// TheSuperHackers @bugfix Caball009 25/12/2025 Avoid mismatches by making m_nextTemplateID unique for a single match instead of unique since game launch.
	DEBUG_ASSERTCRASH(m_firstTemplate && m_firstTemplate->getTemplateID() == m_templateHashMap.size(), ("Template ID is unexpected after deleting overrides"));
	m_nextTemplateID = static_cast<UnsignedShort>(m_firstTemplate->getTemplateID() + 1);
}

//-------------------------------------------------------------------------------------------------
/** Update */
//-------------------------------------------------------------------------------------------------
void ThingFactory::update()
{

}

//-------------------------------------------------------------------------------------------------
/** Return the template with the matching database name */
//-------------------------------------------------------------------------------------------------
const ThingTemplate *ThingFactory::findByTemplateID( UnsignedShort id )
{
	for (ThingTemplate *tmpl = m_firstTemplate; tmpl; tmpl = tmpl->friend_getNextTemplate())
	{
		if (tmpl->getTemplateID() == id)
			return tmpl;
	}
	DEBUG_CRASH(("template %d not found",(Int)id));
	return nullptr;
}

//-------------------------------------------------------------------------------------------------
/** Return the template with the matching database name */
//-------------------------------------------------------------------------------------------------
ThingTemplate *ThingFactory::findTemplateInternal( const AsciiString& name, Bool check )
{
	ThingTemplateHashMapIt tIt = m_templateHashMap.find(name);

	if (tIt != m_templateHashMap.end()) {
		return tIt->second;
	}

#ifdef LOAD_TEST_ASSETS
	if (!strncmp(name.str(), TEST_STRING, strlen(TEST_STRING)))
	{
		ThingTemplate *tmplate = newTemplate( "Un-namedTemplate" );

		// load the values
		tmplate->initForLTA( name );

		// Kinda lame, but necessary.
		m_templateHashMap.erase("Un-namedTemplate");
		m_templateHashMap[name] = tmplate;

		// add tmplate template to the database
		return findTemplateInternal( name );

	}

#endif

	if( check && name.isNotEmpty() )
	{
		DEBUG_CRASH( ("Failed to find thing template %s (case sensitive) This issue has a chance of crashing after you ignore it!", name.str() ) );
	}
	return nullptr;

}

//=============================================================================
Object *ThingFactory::newObject( const ThingTemplate *tmplate, Team *team, ObjectStatusMaskType statusBits )
{
	if (tmplate == nullptr)
		throw ERROR_BAD_ARG;

	const std::vector<AsciiString>& asv = tmplate->getBuildVariations();
	if (!asv.empty())
	{
		Int which = GameLogicRandomValue(0, asv.size()-1);
		const ThingTemplate* tmp = findTemplate( asv[which] );
		if (tmp != nullptr)
			tmplate = tmp;
	}

	DEBUG_ASSERTCRASH(!tmplate->isKindOf(KINDOF_DRAWABLE_ONLY), ("You may not create Objects with the template %s, only Drawables",tmplate->getName().str()));

	// have the game logic create an object of the correct type.
	// (this will throw an exception on failure.)
	//Added ability to pass in optional statusBits. This is needed to be set prior to
	//the onCreate() calls... in the case of constructing.
	Object *obj = TheGameLogic->friend_createObject( tmplate, statusBits, team );

	// run the create function for the thing
	for (BehaviorModule** m = obj->getBehaviorModules(); *m; ++m)
	{
		CreateModuleInterface* create = (*m)->getCreate();
		if (!create)
			continue;

		create->onCreate();
	}

	//
	// all objects are part of the partition manager system, add it to that
	// system now
	//
	ThePartitionManager->registerObject( obj );

	obj->initObject();

	return obj;

}

//=============================================================================
Drawable *ThingFactory::newDrawable(const ThingTemplate *tmplate, DrawableStatusBits statusBits)
{
	if (tmplate == nullptr)
		throw ERROR_BAD_ARG;

	Drawable *draw = TheGameClient->friend_createDrawable( tmplate, statusBits );

	/** @todo we should keep track of all the drawables we've allocated here
	but we'll wait until we have an drawable storage to do that cause it will
	all be tied together */

	return draw;

}

#if defined(RTS_DEBUG) || defined(DEBUG_CRASHING)
AsciiString TheThingTemplateBeingParsedName;
#endif

//-------------------------------------------------------------------------------------------------
/** Parse Object entry */
//-------------------------------------------------------------------------------------------------
/*static*/ void ThingFactory::parseObjectDefinition( INI* ini, const AsciiString& name, const AsciiString& reskinFrom )
{
#if defined(RTS_DEBUG) || defined(DEBUG_CRASHING)
	TheThingTemplateBeingParsedName = name;
#endif

	// find existing item if present
	ThingTemplate *thingTemplate = TheThingFactory->findTemplateInternal( name, FALSE );
	if( !thingTemplate )
	{
		// no item is present, create a new one
		thingTemplate = TheThingFactory->newTemplate( name );
		if ( ini->getLoadType() == INI_LOAD_CREATE_OVERRIDES )
		{
			// This ThingTemplate is actually an override, so we will mark it as such so that it properly
			// gets deleted on ::reset().
			thingTemplate->markAsOverride();
		}
	}
	else if( ini->getLoadType() != INI_LOAD_CREATE_OVERRIDES )
	{
		//Holy crap, this sucks to debug!!!
		//If you have two different objects, the previous code would simply
		//allow you to define multiple objects with the same name, and just
		//nuke the old one with the new one. So, I (KM) have added this
		//assert to notify in case of two same-name objects.
		DEBUG_CRASH(( "[LINE: %d in '%s'] Duplicate factionunit %s found!", ini->getLineNum(), ini->getFilename().str(), name.str() ));
	}
	else
	{
		thingTemplate = TheThingFactory->newOverride( thingTemplate );
	}

	if (reskinFrom.isNotEmpty())
	{
		const ThingTemplate* reskinTmpl = TheThingFactory->findTemplate(reskinFrom);
		if (reskinTmpl)
		{
			thingTemplate->copyFrom(reskinTmpl);
			thingTemplate->setCopiedFromDefault();
			thingTemplate->setReskinnedFrom(reskinTmpl);
			ini->initFromINI( thingTemplate, thingTemplate->getReskinFieldParse() );
		}
		else
		{
			DEBUG_CRASH(("ObjectReskin must come after the original Object (%s, %s).",reskinFrom.str(),name.str()));
			throw INI_INVALID_DATA;
		}
	}
	else
	{
		ini->initFromINI( thingTemplate, thingTemplate->getFieldParse() );
	}

	thingTemplate->validate();

	if( ini->getLoadType() == INI_LOAD_CREATE_OVERRIDES )
	{
		thingTemplate->resolveNames();
	}

#if defined(RTS_DEBUG) || defined(DEBUG_CRASHING)
	TheThingTemplateBeingParsedName.clear();
#endif
}

//#define CHECK_THING_NAMES
#ifdef CHECK_THING_NAMES

#include "Common/STLTypedefs.h"

const char *outFilenameINI				= "thing.txt";
const char *outFilenameStringFile	= "thingString.txt";

void resetReportFile()
{
	FILE *fp = fopen(outFilenameINI, "w");
	if (fp)
	{
		fprintf(fp, "-- ThingTemplate INI Report --\n\n");
		fclose(fp);
	}

	fp = fopen(outFilenameStringFile, "w");
	if (fp)
	{
		fprintf(fp, "-- ThingTemplate String File Report --\n\n");
		fclose(fp);
	}
}

AsciiStringList missingStrings;
void reportMissingNameInStringFile( AsciiString templateName )
{
	// see if we've seen it before
	AsciiStringListConstIterator cit = std::find(missingStrings.begin(), missingStrings.end(), templateName);
	if (cit != missingStrings.end())
		return;

	missingStrings.push_back(templateName);
}

void dumpMissingStringNames()
{
	missingStrings.sort();
	FILE *fp = fopen(outFilenameStringFile, "w");
	if (fp)
	{
		fprintf(fp, "-- ThingTemplate String File Report --\n\n");
		for (AsciiStringListConstIterator cit = missingStrings.begin(); cit!=missingStrings.end(); cit++)
		{
			fprintf(fp, "OBJECT:%s\n\"%s\"\nEND\n\n", cit->str(), cit->str());
		}
		fclose(fp);
	}
}

AsciiStringList missingNames;
void reportMissingNameInTemplate( AsciiString templateName )
{
	// see if we've seen it before
	AsciiStringListConstIterator cit = std::find(missingNames.begin(), missingNames.end(), templateName);
	if (cit != missingNames.end())
		return;

	missingNames.push_back(templateName);

	FILE *fp = fopen(outFilenameINI, "a+");
	if (fp)
	{
		fprintf(fp, "  DisplayName      = OBJECT:%s\n", templateName.str());
		fclose(fp);
	}

	//reportMissingNameInStringFile( templateName );
}

#endif

//-------------------------------------------------------------------------------------------------
/** Post process phase after loading the database files */
//-------------------------------------------------------------------------------------------------
void ThingFactory::postProcessLoad()
{
#ifdef CHECK_THING_NAMES
	//resetReportFile();
#endif

	// go through all thing templates
	for( ThingTemplate *thingTemplate = m_firstTemplate;
			 thingTemplate;
			 thingTemplate = thingTemplate->friend_getNextTemplate() )
	{

		// resolve the prerequisite names
		thingTemplate->resolveNames();

#ifdef CHECK_THING_NAMES
		if (thingTemplate->getDisplayName().isEmpty())
		{
			reportMissingNameInTemplate( thingTemplate->getName() );
		}
		else if (wcsstr(thingTemplate->getDisplayName().str(), L"MISSING:"))
		{
			AsciiString asciiName;
			asciiName.translate(thingTemplate->getDisplayName());
			asciiName.removeLastChar();
			asciiName = asciiName.str() + 17;
			reportMissingNameInStringFile( asciiName );
		}
#endif

	}

#ifdef CHECK_THING_NAMES
	dumpMissingStringNames();
	exit(0);
#endif
}
```
