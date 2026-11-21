# Translation Packet `PKT-451`: `DataChunk`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-451`
- **Work Package:** `WP-7.2` (Phase 7)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:ephemeral-web` (seeded with C++ golden oracle `docs/golden_vectors/PKT-451.json` & Rust skeleton)
- **FFI Seam Pattern:** Core Spine
- **Allowed Rust Output Files:**
  - `crates/zh_common/src/io/data_chunk.rs`
  - `crates/zh_common/tests/golden_pkt451_datachunk.rs` *(remove the `#[ignore]` attributes once implemented)*
  - `docs/PORT_MAP.md` *(regenerate with `python3 scripts/port_map.py --write`)*
- **Mandatory V&V Gates:** `V1, V2-Golden`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test -p zh_common --test golden_pkt451_datachunk && python3 scripts/port_map.py --write && cargo test --workspace
  ```

## Port Contract (read first)

1. **Container header & LIFO Table of Contents (`DataChunkTableOfContents::write`).** Every `DataChunkOutput` file starts with the 4-byte magic `"CkMp"`, followed by `i32` little-endian entry count, then each entry written by walking `m_list` from head to tail. Because `allocateID` prepends new `Mapping` nodes to the head of `m_list`, **TOC entries are written in reverse (LIFO) allocation order**. Each entry is: `u8 name_len`, `name_len` raw ASCII bytes (no NUL), and `u32 id` (LE, allocated starting at `1`). Keep `entries` insertion order and emit in reverse on `write` (`rev()`).
2. **Chunk framing.** `open_data_chunk(name, version)` allocates/looks up `id` in the TOC, then writes: `u32 id` (LE), `u16 version` (LE), and `i32 data_size` (LE placeholder `0` back-patched on `close_data_chunk`). Remember chunks can nest arbitrarily and can have `data_size == 0` (empty chunk).
3. **String & Dict wire encodings.**
   - `write_ascii_string`: `u16 len` (LE) + `len` raw bytes (check length fits `u16`).
   - `write_unicode_string`: `u16 len` (LE) + `len * 2` UTF-16LE bytes (never 4 bytes per char!).
   - `write_name_key`: resolves string name via `NameKeyGenerator::key_to_name`, allocates `toc_id` via TOC, and writes `i32 ((toc_id << 8) | (DictDataType::AsciiString as u8))` LE.
   - `write_dict`: writes `u16 pair_count` (LE), then for each `(key, type, value)` in `0..pair_count`, resolves `key` via `NameKeyGenerator::key_to_name`, allocates `toc_id`, writes `i32 ((toc_id << 8) | (dtype as u8))`, and writes the scalar or string payload matching `DictDataType` (`Bool` = 1-byte `u8`, `Int` = 4-byte `i32` LE, `Real` = 4-byte `f32` LE, `AsciiString`, `UnicodeString`).
4. **LIFO Parser Registration & Unread Tail Skipping (`DataChunkInput`).**
   - `register_parser` prepends to `m_parserList` (LIFO), so if two parsers are registered for the same `(label, parent_label)` pair, the **later-registered** parser matches first.
   - `close_data_chunk` MUST advance/skip any remaining unread `data_left` bytes in the current chunk (and adjust parent chunk `data_left` accordingly via `decrement_data_left` or offset bookkeeping) before popping the stack.
   - Reject malformed/corrupt streams cleanly with `Err(DataChunkError::...)`: zero panics or integer underflows on truncated inputs, negative/overflowing chunk sizes, or unknown `DictDataType` (`CorruptFormat`).
5. **`CachedFileInputStream` Compression Integration.** Reuse `zh_common::io::compression::{is_data_compressed, get_uncompressed_size, decompress_data}`. Mirror C++: if `is_data_compressed(raw)` is true, attempt `decompress_data(raw)`; if `Ok(decompressed)` has `decompressed.len() == get_uncompressed_size(raw) as usize`, replace the internal buffer with `decompressed`; otherwise silently keep the original `raw` bytes (without panicking).
6. **Determinism & Dependencies.** Zero external crates. No `HashMap`/`HashSet`/`BinaryHeap`. Pass Gate V1 (`scripts/lint_determinism.py`). Use `crates/zh_common/tests/golden_pkt451_datachunk.rs` as the 1,000-seed differential oracle (`raw_hex`, `ear_hex`, `parsed_crc_hex`, and truncation/corruption checks).
7. **3-attempt stall rule.** If 3 consecutive fix attempts on failing golden seeds do not shrink the failure set, stop guessing, isolate the minimal failing seed, re-read the exact C++ function in section 3 below, or re-label `tier:3-architect` with exact mismatch details.

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (9):**
  - `Common/DataChunk.h`
  - `Common/Dict.h`
  - `Common/FileSystem.h`
  - `Common/GameMemory.h`
  - `Common/MapReaderWriterInfo.h`
  - `Common/file.h`
  - `Compression.h`
  - `PreRTS.h`
  - `stdlib.h`
- **`Snapshot::crc` / `Snapshot::xfer` Calls:** None in module.

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

### `GeneralsMD/Code/GameEngine/Include/Common/DataChunk.h`
```cpp
// DataChunk.h
// Data chunk classes for saving and loading
// Author: Michael S. Booth, October 2000

#pragma once

#include "Common/GameMemory.h"
#include "Common/Dict.h"
#include "Common/MapReaderWriterInfo.h"

typedef unsigned short DataChunkVersionType;

// forward declarations
class DataChunkInput;
class DataChunkOutput;
class DataChunkTableOfContents;

//----------------------------------------------------------------------
// Mapping
//----------------------------------------------------------------------
class Mapping : public MemoryPoolObject
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(Mapping, "Mapping")
public:
	Mapping*			next;
	AsciiString		name;
	UnsignedInt		id;
};
EMPTY_DTOR(Mapping)

//----------------------------------------------------------------------
// OutputChunk
//----------------------------------------------------------------------
class OutputChunk : public MemoryPoolObject
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(OutputChunk, "OutputChunk")
public:
	OutputChunk*	next;
	UnsignedInt		id;															// chunk symbol type from table of contents
	Int						filepos;											// position of file at start of data offset
};
EMPTY_DTOR(OutputChunk)

//----------------------------------------------------------------------
// InputChunk
//----------------------------------------------------------------------
class InputChunk : public MemoryPoolObject
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(InputChunk, "InputChunk")
public:
	InputChunk*						next;
	UnsignedInt						id;															// chunk symbol type from table of contents
	DataChunkVersionType	version;									// version of data
	Int										chunkStart;																// tell position of the start of chunk data (past header).
	Int										dataSize;														// total data size of chunk
	Int										dataLeft;														// data left to read in this chunk
};
EMPTY_DTOR(InputChunk)

//----------------------------------------------------------------------
// DataChunkTableOfContents
//----------------------------------------------------------------------
class DataChunkTableOfContents
{
	Mapping*		m_list;																/// @TODO: This should be a hash table
	Int					m_listLength;
	UnsignedInt	m_nextID;											// simple ID allocator
	Bool				m_headerOpened;

	Mapping *findMapping( const AsciiString& name );			// return mapping data

public:
	DataChunkTableOfContents();
	~DataChunkTableOfContents();

	UnsignedInt getID( const AsciiString& name );				// convert name to integer identifier
	AsciiString getName( UnsignedInt id );	// convert integer identifier to name
	UnsignedInt allocateID( const AsciiString& name );		// create new ID for given name or return existing mapping

	Bool isOpenedForRead() {return m_headerOpened;};

	void write(OutputStream &out);
	void read(ChunkInputStream &in);
};


//----------------------------------------------------------------------
// DataChunkOutput
//----------------------------------------------------------------------
class DataChunkOutput
{
protected:
	OutputStream*							m_pOut;										// The actual output stream.
	FILE *										m_tmp_file;												// tmp output file stream
	DataChunkTableOfContents	m_contents;			// table of contents of data chunk types
	OutputChunk*							m_chunkStack;													// current stack of open data chunks

public:
	DataChunkOutput(  OutputStream *pOut  );
	~DataChunkOutput();

	void openDataChunk( const char *name, DataChunkVersionType ver );
	void closeDataChunk();

	void writeReal(Real r);
	void writeInt(Int i);
	void writeByte(Byte b);
	void writeAsciiString(const AsciiString& string);
	void writeUnicodeString(UnicodeString string);
	void writeArrayOfBytes(char *ptr, Int len);
	void writeDict(const Dict& d);
	void writeNameKey(const NameKeyType key);
};

//----------------------------------------------------------------------
// DataChunkInput
//----------------------------------------------------------------------
struct DataChunkInfo
{
	AsciiString label;
	AsciiString parentLabel;
	DataChunkVersionType version;
	Int dataSize;
};

typedef Bool (*DataChunkParserPtr)( DataChunkInput &file, DataChunkInfo *info, void *userData );

//----------------------------------------------------------------------
// UserParser
//----------------------------------------------------------------------
class UserParser : public MemoryPoolObject
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(UserParser, "UserParser")
public:
	UserParser *next;

	DataChunkParserPtr parser;										// the user parsing function
	AsciiString label;																	// the data chunk label to match
	AsciiString parentLabel;														// the parent chunk's label (the scope)
	void *userData;																	// user data pointer
};
EMPTY_DTOR(UserParser)

//----------------------------------------------------------------------
// DataChunkInput
//----------------------------------------------------------------------
class DataChunkInput
{
	enum {CHUNK_HEADER_BYTES = 4}; // 2 shorts in chunk file header.
protected:
	ChunkInputStream*					m_file;															// input file stream
	DataChunkTableOfContents	m_contents;							// table of contents of data chunk types
	Int												m_fileposOfFirstChunk;										// seek position of first data chunk
	UserParser*								m_parserList;																		// list of all registered parsers for this input stream
	InputChunk*								m_chunkStack;																		// current stack of open data chunks

	void clearChunkStack();										// clear the stack

	void decrementDataLeft( int size );							// update data left in chunk(s)

public:
	void *m_currentObject;														// user parse routines can use this to allow one chunk
																									// to create an object, and a subsequent chunk to
																									// parse values into that object.  However, the second
																									// chunk parser could also create and parse an object
																									// of its own if this pointer is null.
																									// The parser of the base class should set this pointer to null.
	void *m_userData;																	// user data hook

public:
	DataChunkInput( ChunkInputStream *pStream );
	~DataChunkInput();

	// register a parser function for data chunks with labels matching "label", whose parent
	// chunks labels match "parentLabel" (or null for global scope)
	void registerParser( const AsciiString& label, const AsciiString& parentLabel, DataChunkParserPtr parser, void *userData = nullptr );

	Bool parse( void *userData = nullptr );						// parse the chunk stream using registered parsers
																									// assumed to be at the start of chunk when called
																									// can be called recursively

	Bool isValidFileType();											///< Returns TRUE if it is our file format.
	AsciiString openDataChunk(DataChunkVersionType *ver );
	void closeDataChunk();										// close chunk and move to start of next chunk

	Bool atEndOfFile() { return (m_file->eof()) ? true : false; }					// return true if at end of file
	Bool atEndOfChunk();											// return true if all data has been read from this chunk

	void reset();															// reset to just-opened state

	AsciiString getChunkLabel();							// return label of current data chunk
	DataChunkVersionType getChunkVersion();		// return version of current data chunk
	unsigned int getChunkDataSize();					// return size of data stored in this chunk
	unsigned int getChunkDataSizeLeft();			// return size of data left to read in this chunk


	Real readReal();
	Int readInt();
	Byte readByte();

	AsciiString readAsciiString();
	UnicodeString readUnicodeString();
	Dict readDict();
	void readArrayOfBytes(char *ptr, Int len);

	NameKeyType readNameKey();
};
```

### `GeneralsMD/Code/GameEngine/Source/Common/System/DataChunk.cpp`
```cpp
// DataChunk.cpp
// Implementation of Data Chunk save/load system
// Author: Michael S. Booth, October 2000

#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "stdlib.h"
#include "Compression.h"
#include "Common/DataChunk.h"
#include "Common/file.h"
#include "Common/FileSystem.h"

// If verbose, lots of debug logging.
#define not_VERBOSE

CachedFileInputStream::CachedFileInputStream():m_buffer(nullptr),m_size(0)
{
}

CachedFileInputStream::~CachedFileInputStream()
{
	delete[] m_buffer;
	m_buffer=nullptr;
}

Bool CachedFileInputStream::open(AsciiString path)
{
	File *file=TheFileSystem->openFile(path.str(), File::READ | File::BINARY);
	m_size = 0;

	if (file) {
		m_size=file->size();
		if (m_size) {
			m_buffer = file->readEntireAndClose();
			file = nullptr;
		}
		m_pos=0;
	}

	if (CompressionManager::isDataCompressed(m_buffer, m_size) == 0)
	{
		//DEBUG_LOG(("CachedFileInputStream::open() - file %s is uncompressed at %d bytes!", path.str(), m_size));
	}
	else
	{
		Int uncompLen = CompressionManager::getUncompressedSize(m_buffer, m_size);
		//DEBUG_LOG(("CachedFileInputStream::open() - file %s is compressed!  It should go from %d to %d", path.str(),
		//	m_size, uncompLen));
		char *uncompBuffer = NEW char[uncompLen];
		Int actualLen = CompressionManager::decompressData(m_buffer, m_size, uncompBuffer, uncompLen);
		if (actualLen == uncompLen)
		{
			//DEBUG_LOG(("Using uncompressed data"));
			delete[] m_buffer;
			m_buffer = uncompBuffer;
			m_size = uncompLen;
		}
		else
		{
			//DEBUG_LOG(("Decompression failed - using compressed data"));
			// decompression failed.  Maybe we invalidly thought it was compressed?
			delete[] uncompBuffer;
		}
	}
	//if (m_size >= 4)
	//{
	//	DEBUG_LOG(("File starts as '%c%c%c%c'", m_buffer[0], m_buffer[1],
	//		m_buffer[2], m_buffer[3]));
	//}

	if (file)
	{
		file->close();
	}
	return m_size != 0;
}

void CachedFileInputStream::close()
{
	delete[] m_buffer;
	m_buffer=nullptr;

	m_pos=0;
	m_size=0;
}

Int CachedFileInputStream::read(void *pData, Int numBytes)
{
	if (m_buffer) {
		if ((numBytes+m_pos)>m_size) {
			numBytes=m_size-m_pos;
		}
		if (numBytes) {
			memcpy(pData,m_buffer+m_pos,numBytes);
			m_pos+=numBytes;
		}
		return(numBytes);
	}
	return 0;
}

UnsignedInt CachedFileInputStream::tell()
{
	return m_pos;
}

Bool CachedFileInputStream::absoluteSeek(UnsignedInt pos)
{
	if (pos<0) return false;
	if (pos>m_size) {
		pos=m_size;
	}
	m_pos=pos;
	return true;
}

Bool CachedFileInputStream::eof()
{
	return m_size==m_pos;
}

void CachedFileInputStream::rewind()
{
	m_pos=0;
}

// -----------------------------------------------------------

//
// FileInputStream - helper class.	Used to read in data using a FILE *
//
/*
FileInputStream::FileInputStream():m_file(nullptr)
{
}

FileInputStream::~FileInputStream()
{
	if (m_file != nullptr) {
		m_file->close();
		m_file = nullptr;
	}
}

Bool FileInputStream::open(AsciiString path)
{
	m_file = TheFileSystem->openFile(path.str(), File::READ | File::BINARY);
	return m_file == nullptr?false:true;
}

void FileInputStream::close()
{
	if (m_file != nullptr) {
		m_file->close();
		m_file = nullptr;
	}
}

Int FileInputStream::read(void *pData, Int numBytes)
{
	int bytesRead = 0;
	if (m_file != nullptr) {
		bytesRead = m_file->read(pData, numBytes);
	}
	return(bytesRead);
}

UnsignedInt FileInputStream::tell()
{
	UnsignedInt pos = 0;
	if (m_file != nullptr) {
		pos = m_file->position();
	}
	return(pos);
}

Bool FileInputStream::absoluteSeek(UnsignedInt pos)
{
	if (m_file != nullptr) {
		return (m_file->seek(pos, File::START) != -1);
	}
	return(false);
}

Bool FileInputStream::eof()
{
	if (m_file != nullptr) {
		return (m_file->size() == m_file->position());
	}
	return(true);
}

void FileInputStream::rewind()
{
	if (m_file != nullptr) {
		m_file->seek(0, File::START);
	}
}
*/

//----------------------------------------------------------------------
// DataChunkOutput
// Data will be stored to a temporary m_tmp_file until the DataChunkOutput
// object is destroyed.  At that time, the actual output m_tmp_file will
// be written, including a table of m_contents.
//----------------------------------------------------------------------

#define TEMP_FILENAME		"_tmpChunk.dat"

DataChunkOutput::DataChunkOutput( OutputStream *pOut ) :
m_pOut(pOut)
{
	AsciiString tmpFileName = TheGlobalData->getPath_UserData();
	tmpFileName.concat(TEMP_FILENAME);
	m_tmp_file = ::fopen( tmpFileName.str(), "wb" );
	m_chunkStack = nullptr;
}

DataChunkOutput::~DataChunkOutput()
{
	// store the table of m_contents
	m_contents.write(*m_pOut);

	// Rewind the temp m_tmp_file
	::fclose(m_tmp_file);

	AsciiString tmpFileName = TheGlobalData->getPath_UserData();
	tmpFileName.concat(TEMP_FILENAME);

 	m_tmp_file = ::fopen( tmpFileName.str(), "rb" );
	::fseek(m_tmp_file, 0, SEEK_SET);

	// append the temp m_tmp_file m_contents
	char buffer[256];
	int len = 256;
	while( len == 256 )
	{
		// copy data from the temp m_tmp_file to the output m_tmp_file
		len = ::fread( buffer, 1, 256, m_tmp_file );
		m_pOut->write( buffer, len );
	}

	::fclose(m_tmp_file);
}

void DataChunkOutput::openDataChunk( const char *name, DataChunkVersionType ver )
{
	// allocate (or get existing) ID from the table of m_contents
	UnsignedInt id = m_contents.allocateID( AsciiString(name) );

	// allocate a new chunk and place it on top of the chunk stack
	OutputChunk *c = newInstance(OutputChunk);
	c->next = m_chunkStack;
	m_chunkStack = c;
	m_chunkStack->id = id;

	// store the chunk ID
	::fwrite( (const char *)&id, sizeof(UnsignedInt), 1, m_tmp_file );

	// store the chunk version number
	::fwrite( (const char *)&ver, sizeof(DataChunkVersionType), 1, m_tmp_file );

	// remember this m_tmp_file position so we can write the real data size later
	c->filepos = ::ftell(m_tmp_file);
#ifdef VERBOSE
	DEBUG_LOG(("Writing chunk %s at %d (%x)", name, ::ftell(m_tmp_file), ::ftell(m_tmp_file)));
#endif
	// store a placeholder for the data size
	Int dummy = 0xffff;
	::fwrite( (const char *)&dummy, sizeof(Int), 1, m_tmp_file  );
}

void DataChunkOutput::closeDataChunk()
{
	if (m_chunkStack == nullptr)
	{
		// TODO: Throw exception
		return;
	}

	// remember where we are
	Int here = ::ftell(m_tmp_file);

	// rewind to store the data size
	::fseek(m_tmp_file, m_chunkStack->filepos , SEEK_SET);

	// compute data size (not including the actual data size itself)
	Int size = here - m_chunkStack->filepos - sizeof(Int);

	// store the data size
	::fwrite( (const char *)&size, sizeof(Int) , 1, m_tmp_file );

	// go back to where we were
	::fseek(m_tmp_file, here , SEEK_SET);

	// pop the chunk off the stack
	OutputChunk *c = m_chunkStack;
#ifdef VERBOSE
	DEBUG_LOG(("Closing chunk %s at %d (%x)", m_contents.getName(c->id).str(), here, here));
#endif
	m_chunkStack = m_chunkStack->next;
	deleteInstance(c);
}

void DataChunkOutput::writeReal( Real r )
{
	::fwrite( (const char *)&r, sizeof(float) , 1, m_tmp_file  );
}

void DataChunkOutput::writeInt( Int i )
{
	::fwrite( (const char *)&i, sizeof(Int) , 1, m_tmp_file );
}

void DataChunkOutput::writeByte( Byte b )
{
	::fwrite( (const char *)&b, sizeof(Byte) , 1, m_tmp_file );
}

void DataChunkOutput::writeArrayOfBytes(char *ptr, Int len)
{
	::fwrite( (const char *)ptr, 1, len , m_tmp_file );
}

void DataChunkOutput::writeAsciiString( const AsciiString& theString )
{
	UnsignedShort len = theString.getLength();
	::fwrite( (const char *)&len, sizeof(UnsignedShort) , 1, m_tmp_file );
	::fwrite( theString.str(), len , 1, m_tmp_file );
}

void DataChunkOutput::writeUnicodeString( UnicodeString theString )
{
	// GeneralsX @bugfix BenderAI 20/02/2026 File format uses UTF-16 (2 bytes/char, Windows wchar_t).
	// On Linux sizeof(WideChar)==4; always write 2 bytes/char for cross-platform compatibility.
	UnsignedShort len = theString.getLength();
	::fwrite( (const char *)&len, sizeof(UnsignedShort) , 1, m_tmp_file );
	if (sizeof(WideChar) == sizeof(UnsignedShort))
	{
		// Windows: WideChar is 2 bytes - write directly
		::fwrite( theString.str(), len * (Int)sizeof(UnsignedShort), 1, m_tmp_file );
	}
	else
	{
		// Linux: WideChar is 4 bytes - narrow each char to UTF-16 before writing
		const WideChar *src = theString.str();
		UnsignedShort *tmp = NEW UnsignedShort[len + 1];
		for (Int i = 0; i < (Int)len; i++)
			tmp[i] = (UnsignedShort)src[i];
		::fwrite( tmp, len * (Int)sizeof(UnsignedShort), 1, m_tmp_file );
		delete[] tmp;
	}
}

void DataChunkOutput::writeNameKey( const NameKeyType key )
{
		AsciiString kname = TheNameKeyGenerator->keyToName(key);
		Int keyAndType = m_contents.allocateID(kname);
		keyAndType <<= 8;
		Dict::DataType t = Dict::DICT_ASCIISTRING;
		keyAndType |= (t & 0xff);
		writeInt(keyAndType);
}

void DataChunkOutput::writeDict( const Dict& d )
{
	UnsignedShort len = d.getPairCount();
	::fwrite( (const char *)&len, sizeof(UnsignedShort) , 1, m_tmp_file );
	for (int i = 0; i < len; i++)
	{
		NameKeyType k = d.getNthKey(i);
		AsciiString kname = TheNameKeyGenerator->keyToName(k);

		Int keyAndType = m_contents.allocateID(kname);
		keyAndType <<= 8;
		Dict::DataType t = d.getNthType(i);
		keyAndType |= (t & 0xff);
		writeInt(keyAndType);

		switch(t)
		{
			case Dict::DICT_BOOL:
				writeByte(d.getNthBool(i)?1:0);
				break;
			case Dict::DICT_INT:
				writeInt(d.getNthInt(i));
				break;
			case Dict::DICT_REAL:
				writeReal(d.getNthReal(i));
				break;
			case Dict::DICT_ASCIISTRING:
				writeAsciiString(d.getNthAsciiString(i));
				break;
			case Dict::DICT_UNICODESTRING:
				writeUnicodeString(d.getNthUnicodeString(i));
				break;
			default:
				DEBUG_CRASH(("impossible"));
				break;
		}
	}
}

//----------------------------------------------------------------------
// DataChunkTableOfContents
//----------------------------------------------------------------------

DataChunkTableOfContents::DataChunkTableOfContents() :
m_list(nullptr),
m_nextID(1),
m_listLength(0),
m_headerOpened(false)
{
}

DataChunkTableOfContents::~DataChunkTableOfContents()
{
	Mapping *m, *next;

	// free all list elements
	for( m=m_list; m; m=next )
	{
		next = m->next;
		deleteInstance(m);
	}
}

// return mapping data
Mapping *DataChunkTableOfContents::findMapping( const AsciiString& name )
{
	Mapping *m;

	for( m=m_list; m; m=m->next )
		if (name == m->name )
			return m;

	return nullptr;
}

// convert name to integer identifier
UnsignedInt DataChunkTableOfContents::getID( const AsciiString& name )
{
	Mapping *m = findMapping( name );

	if (m)
		return m->id;

	DEBUG_CRASH(("name not found in DataChunkTableOfContents::getName for name %s",name.str()));
	return 0;
}

// convert integer identifier to name
AsciiString DataChunkTableOfContents::getName( UnsignedInt id )
{
	Mapping *m;

	for( m=m_list; m; m=m->next )
		if (m->id == id)
			return m->name;

	DEBUG_CRASH(("name not found in DataChunkTableOfContents::getName for id %d",id));
	return AsciiString::TheEmptyString;
}

// create new ID for given name or return existing mapping
UnsignedInt DataChunkTableOfContents::allocateID(const AsciiString& name )
{
	Mapping *m = findMapping( name );

	if (m)
		return m->id;
	else
	{
		// allocate new id mapping
		m = newInstance(Mapping);

		m->id = m_nextID++;
		m->name =  name ;

		// prepend to list
		m->next = m_list;
		m_list = m;

		m_listLength++;

		return m->id;
	}
}

// output the table of m_contents to a binary m_tmp_file stream
void DataChunkTableOfContents::write( OutputStream &s )
{
	Mapping *m;
	unsigned char len;

	Byte tag[4]={'C','k', 'M', 'p'};	// Chunky height map. jba.
	s.write(tag,sizeof(tag));

	// output number of elements in the table
	s.write( (void *)&this->m_listLength, sizeof(Int) );

	// output symbol table
	for( m=this->m_list; m; m=m->next )
	{
		len = m->name.getLength();
		s.write( (char *)&len, sizeof(unsigned char) );
		s.write( (char *)m->name.str(),  len);
		s.write( (char *)&m->id, sizeof(UnsignedInt) );
	}
}

// read the table of m_contents from a binary m_tmp_file stream
// TODO: Should this reset the symbol table?
// Append symbols to table
void DataChunkTableOfContents::read( ChunkInputStream &s)
{
	Int count, i;
	UnsignedInt maxID = 0;
	unsigned char len;
	Mapping *m;

	Byte tag[4]={'x','x', 'x', 'x'};	// Chunky height map. jba.
	s.read(tag,sizeof(tag));
	if (tag[0] != 'C' || tag[1] != 'k' || tag[2] != 'M' || tag[3] != 'p') {
		return;	 // Don't throw, may happen with legacy files.
	}

	// get number of symbols in table
	s.read( (char *)&count, sizeof(Int) );

	for( i=0; i<count; i++ )
	{
		// allocate new id mapping
		m = newInstance(Mapping);

		// read string length
		s.read( (char *)&len, sizeof(unsigned char) );

		// allocate and read in string
		if (len>0) {
			char *str = m->name.getBufferForRead(len);
			s.read( str, len );
			str[len] = '\000';
		}

		// read id
		s.read( (char *)&m->id, sizeof(UnsignedInt) );

		// prepend to list
		m->next = this->m_list;
		this->m_list = m;

		this->m_listLength++;

		// track max ID used
		if (m->id > maxID)
			maxID = m->id;
	}
	m_headerOpened = count > 0 && !s.eof();

	// adjust next ID so no ID's are reused
	this->m_nextID = max( this->m_nextID, maxID+1 );
}

//----------------------------------------------------------------------
// DataChunkInput
//----------------------------------------------------------------------
DataChunkInput::DataChunkInput( ChunkInputStream *pStream ) : m_file( pStream ),
																										m_userData(nullptr),
																										m_currentObject(nullptr),
																										m_chunkStack(nullptr),
																										m_parserList(nullptr)
{
	// read table of m_contents
	m_contents.read(*m_file);

	// store location of first data chunk
	m_fileposOfFirstChunk = m_file->tell();
}

DataChunkInput::~DataChunkInput()
{
	clearChunkStack();

	UserParser *p, *next;
	for (p=m_parserList; p; p=next) {
		next = p->next;
		deleteInstance(p);
	}

}

// register a user parsing function for a given DataChunk label
void DataChunkInput::registerParser( const AsciiString& label, const AsciiString& parentLabel,
																		 DataChunkParserPtr parser, void *userData )
{
	UserParser *p = newInstance(UserParser);

	p->label.set( label );
	p->parentLabel.set(parentLabel );
	p->parser = parser;
	p->userData = userData;

	// prepend parser to parser list
	p->next = m_parserList;
	m_parserList = p;
}

// parse the chunk stream using registered parsers
// it is assumed that the file position is at the start of a data chunk
// (it can be inside a parent chunk) when parse is called.
Bool DataChunkInput::parse( void *userData )
{
	AsciiString label;
	AsciiString parentLabel;
	DataChunkVersionType ver;
	UserParser *parser;
	Bool scopeOK;
	DataChunkInfo info;

	// If the header wasn't a chunk table of contents, we can't parse.
	if (!m_contents.isOpenedForRead()) {
		return false;
	}

	// if we are inside a data chunk right now, get its name
	if (m_chunkStack)
		parentLabel = m_contents.getName( m_chunkStack->id );

	while( atEndOfFile() == false )
	{
		if (m_chunkStack) {
			// If we are parsing chunks in a chunk, check current length.
			if (m_chunkStack->dataLeft < CHUNK_HEADER_BYTES) {
				DEBUG_ASSERTCRASH( m_chunkStack->dataLeft==0, ("Unexpected extra data in chunk."));
				break;
			}
		}
		// open the chunk
		label = openDataChunk( &ver );
		if (atEndOfFile()) {
			// FILE * returns eof after you read past end of file, so check.
			break;
		}

		// find a registered parser for this chunk
		for( parser=m_parserList; parser; parser=parser->next )
		{
			// chunk labels must match
			if ( parser->label == label )
			{
				// make sure parent name (scope) also matches
				scopeOK = true;

				if (parentLabel != parser->parentLabel)
					scopeOK = false;

				if (scopeOK)
				{
					// m_tmp_file out the chunk info and call the user parser
					info.label = label;
					info.parentLabel = parentLabel;
					info.version = ver;
					info.dataSize = getChunkDataSize();

					if (parser->parser( *this, &info, userData ) == false)
						return false;
					break;
				}
			}
		}

		// close chunk (and skip to end if need be)
		closeDataChunk();
	}

	return true;
}

// clear the stack
void DataChunkInput::clearChunkStack()
{
	InputChunk *c, *next;

	for( c=m_chunkStack; c; c=next )
	{
		next = c->next;
		deleteInstance(c);
	}

	m_chunkStack = nullptr;
}

// reset the stream to just-opened state - ready to parse the first chunk
void DataChunkInput::reset()
{
	clearChunkStack();
	m_file->absoluteSeek( m_fileposOfFirstChunk );
}

// Checks if the file has our initial tag word.
Bool DataChunkInput::isValidFileType()
{
	return m_contents.isOpenedForRead();
}

AsciiString DataChunkInput::openDataChunk(DataChunkVersionType *ver )
{
	// allocate a new chunk and place it on top of the chunk stack
	InputChunk *c = newInstance(InputChunk);
	c->id = 0;
	c->version = 0;
	c->dataSize = 0;
	//DEBUG_LOG(("Opening data chunk at offset %d (%x)", m_file->tell(), m_file->tell()));
	// read the chunk ID
	m_file->read( (char *)&c->id, sizeof(UnsignedInt) );
	decrementDataLeft( sizeof(UnsignedInt) );

	// read the chunk version number
	m_file->read( (char *)&c->version, sizeof(DataChunkVersionType) );
	decrementDataLeft( sizeof(DataChunkVersionType) );

	// read the chunk data size
	m_file->read( (char *)&c->dataSize, sizeof(Int) );
	decrementDataLeft( sizeof(Int) );

	// all of the data remains to be read
	c->dataLeft = c->dataSize;
	c->chunkStart = m_file->tell();

	*ver = c->version;

	c->next = m_chunkStack;
	m_chunkStack = c;
	if (this->atEndOfFile()) {
		return AsciiString::TheEmptyString;
	}
	return m_contents.getName( c->id );
}

// close chunk and move to start of next chunk
void DataChunkInput::closeDataChunk()
{
	if (m_chunkStack == nullptr)
	{
		// TODO: Throw exception
		return;
	}

	if (m_chunkStack->dataLeft > 0)
	{
		// skip past the remainder of this chunk
		m_file->absoluteSeek( m_file->tell()+m_chunkStack->dataLeft );
		decrementDataLeft( m_chunkStack->dataLeft );

	}

	// pop the chunk off the stack
	InputChunk *c = m_chunkStack;
	m_chunkStack = m_chunkStack->next;
	deleteInstance(c);
}


// return label of current data chunk
AsciiString DataChunkInput::getChunkLabel()
{
	if (m_chunkStack == nullptr)
	{
		// TODO: Throw exception
		DEBUG_CRASH(("Bad."));
		return AsciiString::TheEmptyString;
	}

	return m_contents.getName( m_chunkStack->id );
}

// return version of current data chunk
DataChunkVersionType DataChunkInput::getChunkVersion()
{
	if (m_chunkStack == nullptr)
	{
		// TODO: Throw exception
		DEBUG_CRASH(("Bad."));
		return 0;
	}

	return m_chunkStack->version;
}

// return size of data stored in this chunk
UnsignedInt DataChunkInput::getChunkDataSize()
{
	if (m_chunkStack == nullptr)
	{
		// TODO: Throw exception
		DEBUG_CRASH(("Bad."));
		return 0;
	}

	return m_chunkStack->dataSize;
}


// return size of data left to read in this chunk
UnsignedInt DataChunkInput::getChunkDataSizeLeft()
{
	if (m_chunkStack == nullptr)
	{
		// TODO: Throw exception
		DEBUG_CRASH(("Bad."));
		return 0;
	}

	return m_chunkStack->dataLeft;
}

Bool DataChunkInput::atEndOfChunk()
{
	if (m_chunkStack)
	{
		if (m_chunkStack->dataLeft <= 0)
			return true;
		return false;
	}

	return true;
}

// update data left in chunk(s)
// since data read from a chunk is also read from all parent chunks,
// traverse the chunk stack and decrement the data left for each
void DataChunkInput::decrementDataLeft( Int size )
{
	InputChunk *c;

	c = m_chunkStack;
	while (c) {
		c->dataLeft -= size;
		c = c->next;
	}
	// The sizes of the parent chunks on the stack are adjusted in closeDataChunk.
}

Real DataChunkInput::readReal()
{
	Real r;
	DEBUG_ASSERTCRASH(m_chunkStack->dataLeft>=sizeof(Real), ("Read past end of chunk."));
	m_file->read( (char *)&r, sizeof(Real) );
	decrementDataLeft( sizeof(Real) );
	return r;
}

Int DataChunkInput::readInt()
{
	Int i;
	DEBUG_ASSERTCRASH(m_chunkStack->dataLeft>=sizeof(Int), ("Read past end of chunk."));
	m_file->read( (char *)&i, sizeof(Int) );
	decrementDataLeft( sizeof(Int) );
	return i;
}

Byte DataChunkInput::readByte()
{
	Byte b;
	DEBUG_ASSERTCRASH(m_chunkStack->dataLeft>=sizeof(Byte), ("Read past end of chunk."));
	m_file->read( (char *)&b, sizeof(Byte) );
	decrementDataLeft( sizeof(Byte) );
	return b;
}

void DataChunkInput::readArrayOfBytes(char *ptr, Int len)
{
	DEBUG_ASSERTCRASH(m_chunkStack->dataLeft>=len, ("Read past end of chunk."));
	m_file->read( ptr, len );
	decrementDataLeft( len );
}

NameKeyType DataChunkInput::readNameKey()
{
		Int keyAndType = readInt();
#ifdef DEBUG_CRASHING
		Dict::DataType t = (Dict::DataType)(keyAndType & 0xff);
		DEBUG_ASSERTCRASH(t==Dict::DICT_ASCIISTRING,("Invalid key data."));
#endif
		keyAndType >>= 8;

		AsciiString kname = m_contents.getName(keyAndType);
		NameKeyType k = TheNameKeyGenerator->nameToKey(kname);
		return k;
}

Dict DataChunkInput::readDict()
{
	UnsignedShort len;
	DEBUG_ASSERTCRASH(m_chunkStack->dataLeft>=sizeof(UnsignedShort), ("Read past end of chunk."));
	m_file->read( &len, sizeof(UnsignedShort) );
	decrementDataLeft( sizeof(UnsignedShort) );
	DEBUG_ASSERTCRASH(m_chunkStack->dataLeft>=len, ("Read past end of chunk."));

	Dict d(len);

	for (int i = 0; i < len; i++)
	{
		Int keyAndType = readInt();
		Dict::DataType t = (Dict::DataType)(keyAndType & 0xff);
		keyAndType >>= 8;

		AsciiString kname = m_contents.getName(keyAndType);
		NameKeyType k = TheNameKeyGenerator->nameToKey(kname);

		switch(t)
		{
			case Dict::DICT_BOOL:
				d.setBool(k, readByte() ? true : false);
				break;
			case Dict::DICT_INT:
				d.setInt(k, readInt());
				break;
			case Dict::DICT_REAL:
				d.setReal(k, readReal());
				break;
			case Dict::DICT_ASCIISTRING:
				d.setAsciiString(k, readAsciiString());
				break;
			case Dict::DICT_UNICODESTRING:
				d.setUnicodeString(k, readUnicodeString());
				break;
			default:
				throw ERROR_CORRUPT_FILE_FORMAT;
				break;
		}
	}

	return d;
}

AsciiString DataChunkInput::readAsciiString()
{
	UnsignedShort len;
	DEBUG_ASSERTCRASH(m_chunkStack->dataLeft>=sizeof(UnsignedShort), ("Read past end of chunk."));
	m_file->read( &len, sizeof(UnsignedShort) );
	decrementDataLeft( sizeof(UnsignedShort) );
	DEBUG_ASSERTCRASH(m_chunkStack->dataLeft>=len, ("Read past end of chunk."));
	AsciiString theString;
	if (len>0) {
		char *str = theString.getBufferForRead(len);
		m_file->read( str, len );
		decrementDataLeft( len );
		// add null delimiter to string.  Note that getBufferForRead allocates space for terminating null.
		str[len] = '\000';
	}

	return theString;
}

UnicodeString DataChunkInput::readUnicodeString()
{
	// GeneralsX @bugfix BenderAI 20/02/2026 File format uses UTF-16 (2 bytes/char, Windows wchar_t).
	// On Linux sizeof(WideChar)==4; reading len*sizeof(WideChar) bytes over-consumed file data,
	// corrupting chunk dataLeft and preventing PlayerScriptsList from being parsed (no scripts/camera).
	UnsignedShort len;
	DEBUG_ASSERTCRASH(m_chunkStack->dataLeft>=sizeof(UnsignedShort), ("Read past end of chunk."));
	m_file->read( &len, sizeof(UnsignedShort) );
	decrementDataLeft( sizeof(UnsignedShort) );
	const Int bytesInFile = (Int)len * (Int)sizeof(UnsignedShort);  // always 2 bytes/char in file
	DEBUG_ASSERTCRASH(m_chunkStack->dataLeft>=bytesInFile, ("Read past end of chunk."));
	UnicodeString theString;
	if (len>0) {
		WideChar *str = theString.getBufferForRead(len);
		if (sizeof(WideChar) == sizeof(UnsignedShort))
		{
			// Windows: WideChar is 2 bytes - read directly
			m_file->read( (char*)str, bytesInFile );
		}
		else
		{
			// Linux: WideChar is 4 bytes - read UTF-16 then widen each char
			UnsignedShort *tmp = NEW UnsignedShort[len + 1];
			m_file->read( (char*)tmp, bytesInFile );
			for (Int i = 0; i < (Int)len; i++)
				str[i] = (WideChar)tmp[i];
			delete[] tmp;
		}
		decrementDataLeft( bytesInFile );
		// add null delimiter to string.  Note that getBufferForRead allocates space for terminating null.
		str[len] = '\000';
	}

	return theString;
}
```
