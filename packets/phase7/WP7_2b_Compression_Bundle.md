# Bundled Translation Packet: `WP-7.2b` — EA Compression Codecs (RefPack, BTree, Huffman)

- **Parent work package:** `WP-7.2` (#32), `.big`/`.map` readers. Retail `.map` files are RefPack (`"EAR\0"`) blobs, and `DataChunk` (`PKT-451`) decompresses them through `CompressionManager`.
- **Reference:** `fbraz3/GeneralsX` @ `84a7a20`, `Core/Libraries/Source/Compression/` (`CompressionManager.cpp` + `EAC/`). This code lives outside `GameLogic`/`Common`, so it has no `PKT-*` id in `docs/MODULE_MANIFEST.json`.
- **Target Rust file:** `crates/zh_common/src/io/compression.rs`. The public API is fixed: keep every signature. You may add private helpers or private submodules under `crates/zh_common/src/io/`.
- **C++ golden vector oracle:** `docs/golden_vectors/WP7_2b_Compression.json` (1,000 small vectors with full hex blobs + 12 large vectors as FNV-1a 64 digests).
- **Gate V2-Golden test suite:** `crates/zh_common/tests/golden_wp7_2b_compression.rs`. Remove its four `#[ignore]` attributes. Do not edit the vectors or the assertions.
- **Oracle generator (local only, for reference):** `tests/cpp_harness/gen_wave4_compression.cpp` with `tests/cpp_harness/stubs_wave4_eac/eac_portable_byteorder.h`. The build line is in the harness header. Its output is byte-identical at `-O0`, at `-O2`, and under ASan+UBSan.

## 1. Port Contract (read first)

1. **Byte order: always big-endian codec headers.** `gimex.h`'s `ggetm()`/`gputm()` have an `__APPLE__` branch that does native-endian loads and stores. It was written for big-endian PowerPC Macs. On little-endian Apple Silicon it writes byte-swapped headers (`fb 10` instead of `10 fb`), which `REF_decode` then misreads; an unpatched macOS build of the generator crashes in `REF_decode`. The oracle was generated on the portable path, which is the format Windows/Linux builds and retail data use. Port only the `#else` branches: 2/3/4-byte big-endian. Never port the `__APPLE__` branch.
2. **Blob layout** (`CompressionManager::compressData`): `magic[4]` (`"EAR\0"`, `"EAB\0"`, `"EAH\0"`), then the uncompressed length as a **little-endian `i32`** (the C++ stores a native `Int`, little-endian on retail x86), then the codec stream. `compress_data` returns the whole blob. `refpack::encode`/`btree::encode`/`huff::encode` return the stream only, starting at the `10fb` / `46fb` / `30fb` type word.
3. **Encoders: default mode only.** `CompressionManager` always calls `REF_encode`/`BTREE_encode`/`HUFF_encode` with `opts == nullptr`. For RefPack that is `maxback = 131072`, `quick = 0`; for Huffman, `opt = 0` (no delta pre-pass); for BTree, `opt = 0`. Do not port the other option paths. Outputs must match byte-for-byte, including streams larger than the input (incompressible data).
4. **Decoders: total, bounded, no panics.** The C++ decoders have no bounds checks. The Rust ones must:
   - never panic and always terminate, on any input;
   - return `CodecError::Truncated` when the stream ends early. Exception: the Huffman decoder refills its bit buffer 16 bits at a time and legitimately reads up to **3 bytes** past the end of a valid stream (measured under ASan over every oracle stream). Treat up to **4 bytes** past the end of `stream` as `0x00`, and any read beyond that as `Truncated`. The oracle decodes every stream with `0x00` and `0xff` padding and gets identical results, so look-ahead bytes never reach the output. RefPack and BTree never read past a valid stream;
   - return `CodecError::BadReference` for a back-reference before the start of the output;
   - never produce more bytes than the stream header's uncompressed length, and return `CodecError::LengthMismatch` if the decoded length differs from it. `decompress_data` also checks the stream length against the manager header length;
   - return `CodecError::BadHeader` for a stream type word the codec does not know.

   `test_wp7_2b_decoders_never_panic_on_truncated_or_corrupt_input` checks this with truncations and single-byte corruptions.
5. **Scope of `CompressionManager`:**
   - `get_compression_type`: inputs shorter than 8 bytes are `None`; otherwise match the 13 magics in source order (`NOX`, `ZL1`..`ZL9`, `EAB`, `EAH`, `EAR`), with anything else `None`.
   - `get_uncompressed_size`: `len` when shorter than 8 bytes or when the type is `None`; otherwise the header `i32`, for every compressed type including NOX and zlib.
   - `compress_data`: `Some(blob)` for `RefPack`/`BTree`/`Huff` only, and `None` for every other type, as in C++ where `compressData` returns 0 for them.
   - `decompress_data`: decodes the three EAC types. `NOXLZH` and zlib return `CodecError::Unsupported`; they need the LZH and zlib libraries, which Zero Hour does not use for retail data (`getPreferredCompression()` is `COMPRESSION_REFPACK`). Inputs shorter than 8 bytes, or with an unknown magic, are `BadHeader`.
   - A zero-length payload decodes to `Ok(vec![])`; the C++ returns an ambiguous 0 there.
6. **Huffman stream variants.** `HUFF_decode` masks off the `0x100` flag (an extra size field to skip), then dispatches on the type: `30fb` (with `31fb`, `b0fb`, `b1fb`) is plain; `32fb` is delta; `34fb` is accelerated. The engine only ever writes `30fb`. Port the plain decoder, and return `CodecError::Unsupported` for the delta and accelerated variants, since the oracle can't verify them.
7. **Empty input.** All three encoders produce a non-empty stream for empty input; the oracle records these. `HUFF_decode` never terminates on the stream `HUFF_encode` makes for empty input: it reads unboundedly past the buffer. That one decode has no reference result, and the Rust decoder only has to return (an `Err` is fine).
8. **Integer semantics.** The codecs mix `int`, `unsigned int` and pointer differences. Mirror them exactly:
   - `unsigned int` arithmetic that can wrap uses `wrapping_*`;
   - `qmin`/`qmax` over mixed types follow C++ promotion rules;
   - pointer differences such as `cptr-from-131071` are signed (`isize`/`i64`), never `usize` underflow;
   - `(unsigned char)` casts truncate.

   The dev profile has `overflow-checks = true`, so a missed wrap panics in the tests.
9. **Data structures.** Keep the C++ encoder state (RefPack `hashtbl[65536]` initialised to `-1`, `link[131072]`) and its update order. Match selection depends on chain order, so a "better" match finder changes the output. Use `Vec`/arrays only; no `HashMap` (Gate V1).
10. **Performance.** Tests run at `opt-level = 1`, and the large vectors include 300,000-byte inputs. Keep the C++ algorithms' complexity; don't add per-byte allocations.

## 2. Oracle Coverage

- **Small vectors (seeds 0..999):**
  - lengths: 0..512, with 0, 1, 2, 3, 4, 5, 7, 8, 9, 15, 16, 17, 31, 32, 64, 128, 256, 384 and 512 each appearing twice;
  - five payload patterns: runs, repetitive text, counters, mixed, noise;
  - checked for every codec: encode, `compress_data`, header helpers, `is`/`size`, decode, `decompress_data`.
- **Large vectors (seeds 1000..1011, 16,400–300,000 bytes):** compared by blob length + FNV-1a 64, then round-tripped. A probe of the reference RefPack streams confirmed they reach:
  - the 4-byte "very int" form with offsets ≥ 16384, up to 131,070 (the window edge);
  - matches of the 1,028-byte maximum;
  - no matches at periods 131,072 and 140,000 (outside the window);
  - dense hash chains, where `link[]` wraps at 131,072.
- **Not covered by the oracle** (port as written, keep the header parsing faithful):
  - inputs over 16 MiB (`90fb`/`b0fb` 4-byte size headers);
  - the size-skip header variants `11fb`, `47fb` and `31fb`, which the encoders never write.

## 3. Mandatory Rust Conventions

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

## 4. C++ Source Code (`GeneralsX @ 84a7a20`)

Not included: `refabout.cpp`, `btreeabout.cpp` and `huffabout.cpp` (`*_about()` codec metadata, unused by `CompressionManager`), and `LZHCompress/` (`NOXLZH`, out of scope).

### `Compression.h` (`Core/Libraries/Source/Compression/Compression.h`)

```cpp
// FILE: Compression.h ///////////////////////////////////////////////////////
// Author: Matthew D. Campbell
//////////////////////////////////////////////////////////////////////////////

#pragma once

enum CompressionType
{
	COMPRESSION_NONE,
	COMPRESSION_REFPACK,
	COMPRESSION_NOXLZH,
	COMPRESSION_ZLIB1,
	COMPRESSION_ZLIB2,
	COMPRESSION_ZLIB3,
	COMPRESSION_ZLIB4,
	COMPRESSION_ZLIB5,
	COMPRESSION_ZLIB6,
	COMPRESSION_ZLIB7,
	COMPRESSION_ZLIB8,
	COMPRESSION_ZLIB9,
	COMPRESSION_BTREE,
	COMPRESSION_HUFF,

	COMPRESSION_COUNT,
	COMPRESSION_MIN = COMPRESSION_NONE,
	COMPRESSION_MAX = COMPRESSION_COUNT - 1,
};

class CompressionManager
{
public:

	static Bool isDataCompressed( const void *mem, Int len );
	static CompressionType getCompressionType( const void *mem, Int len );

	static Int getMaxCompressedSize( Int uncompressedLen, CompressionType compType );
	static Int getUncompressedSize( const void *mem, Int len );

	static Int compressData( CompressionType compType, void *src, Int srcLen, void *dest, Int destLen ); // 0 on error
	static Int decompressData( void *src, Int srcLen, void *dest, Int destLen ); // 0 on error

	static const char *getCompressionNameByType( CompressionType compType );

	// For perf timers, so we can have separate ones for compression/decompression
	static const char *getDecompressionNameByType( CompressionType compType );

	static CompressionType getPreferredCompression();
};
```

### `CompressionManager.cpp` (`Core/Libraries/Source/Compression/CompressionManager.cpp`)

Port the EAC (`REFPACK`, `BTREE`, `HUFF`) paths and the header helpers. The `NOXLZH` and zlib paths are out of scope (see Port Contract §5).

```cpp
// FILE: Compression.cpp /////////////////////////////////////////////////////
// Author: Matthew D. Campbell
// LZH wrapper taken from Nox, originally from Jeff Brown
//////////////////////////////////////////////////////////////////////////////

#include <math.h>
#include <string.h>

#include "Compression.h"
#include "LZHCompress/NoxCompress.h"

#define __MACTYPES__
#include <zlib.h>

#include "EAC/codex.h"
#include "EAC/btreecodex.h"
#include "EAC/huffcodex.h"
#include "EAC/refcodex.h"


// TheSuperHackers @todo Recover debug logging in this file?
#define DEBUG_LOG(x)

const char *CompressionManager::getCompressionNameByType( CompressionType compType )
{
	static const char *s_compressionNames[] = {
		"No compression",
		"RefPack",
		"LZHL",
		"ZLib 1 (fast)",
		"ZLib 2",
		"ZLib 3",
		"ZLib 4",
		"ZLib 5 (default)",
		"ZLib 6",
		"ZLib 7",
		"ZLib 8",
		"ZLib 9 (slow)",
		"BTree",
		"Huff",
	};
	static_assert(ARRAY_SIZE(s_compressionNames) == COMPRESSION_COUNT, "Incorrect array size");
	return s_compressionNames[compType];
}

// For perf timers, so we can have separate ones for compression/decompression
const char *CompressionManager::getDecompressionNameByType( CompressionType compType )
{
	static const char *s_decompressionNames[] = {
		"d_None",
		"d_RefPack",
		"d_NoxLZW",
		"d_ZLib1",
		"d_ZLib2",
		"d_ZLib3",
		"d_ZLib4",
		"d_ZLib5",
		"d_ZLib6",
		"d_ZLib7",
		"d_ZLib8",
		"d_ZLib9",
		"d_BTree",
		"d_Huff",
	};
	static_assert(ARRAY_SIZE(s_decompressionNames) == COMPRESSION_COUNT, "Incorrect array size");
	return s_decompressionNames[compType];
}

// ---------------------------------------------------------------------------------------

Bool CompressionManager::isDataCompressed( const void *mem, Int len )
{
	CompressionType t = getCompressionType(mem, len);
	return t != COMPRESSION_NONE;
}

CompressionType CompressionManager::getPreferredCompression()
{
	return COMPRESSION_REFPACK;
}


CompressionType CompressionManager::getCompressionType( const void *mem, Int len )
{
	if (len < 8)
		return COMPRESSION_NONE;

	if ( memcmp( mem, "NOX\0", 4 ) == 0 )
		return COMPRESSION_NOXLZH;
	if ( memcmp( mem, "ZL1\0", 4 ) == 0 )
		return COMPRESSION_ZLIB1;
	if ( memcmp( mem, "ZL2\0", 4 ) == 0 )
		return COMPRESSION_ZLIB2;
	if ( memcmp( mem, "ZL3\0", 4 ) == 0 )
		return COMPRESSION_ZLIB3;
	if ( memcmp( mem, "ZL4\0", 4 ) == 0 )
		return COMPRESSION_ZLIB4;
	if ( memcmp( mem, "ZL5\0", 4 ) == 0 )
		return COMPRESSION_ZLIB5;
	if ( memcmp( mem, "ZL6\0", 4 ) == 0 )
		return COMPRESSION_ZLIB6;
	if ( memcmp( mem, "ZL7\0", 4 ) == 0 )
		return COMPRESSION_ZLIB7;
	if ( memcmp( mem, "ZL8\0", 4 ) == 0 )
		return COMPRESSION_ZLIB8;
	if ( memcmp( mem, "ZL9\0", 4 ) == 0 )
		return COMPRESSION_ZLIB9;
	if ( memcmp( mem, "EAB\0", 4 ) == 0 )
		return COMPRESSION_BTREE;
	if ( memcmp( mem, "EAH\0", 4 ) == 0 )
		return COMPRESSION_HUFF;
	if ( memcmp( mem, "EAR\0", 4 ) == 0 )
		return COMPRESSION_REFPACK;

	return COMPRESSION_NONE;
}

Int CompressionManager::getMaxCompressedSize( Int uncompressedLen, CompressionType compType )
{
	switch (compType)
	{
		case COMPRESSION_NOXLZH:
			return CalcNewSize(uncompressedLen) + 8;

		case COMPRESSION_BTREE:   // guessing here
		case COMPRESSION_HUFF:    // guessing here
		case COMPRESSION_REFPACK: // guessing here
			return uncompressedLen + 8;
		case COMPRESSION_ZLIB1:
		case COMPRESSION_ZLIB2:
		case COMPRESSION_ZLIB3:
		case COMPRESSION_ZLIB4:
		case COMPRESSION_ZLIB5:
		case COMPRESSION_ZLIB6:
		case COMPRESSION_ZLIB7:
		case COMPRESSION_ZLIB8:
		case COMPRESSION_ZLIB9:
			return (Int)(ceil(uncompressedLen * 1.1 + 12 + 8));
	}

	return 0;
}

Int CompressionManager::getUncompressedSize( const void *mem, Int len )
{
	if (len < 8)
		return len;

	CompressionType compType = getCompressionType( mem, len );
	switch (compType)
	{
		case COMPRESSION_NOXLZH:
		case COMPRESSION_ZLIB1:
		case COMPRESSION_ZLIB2:
		case COMPRESSION_ZLIB3:
		case COMPRESSION_ZLIB4:
		case COMPRESSION_ZLIB5:
		case COMPRESSION_ZLIB6:
		case COMPRESSION_ZLIB7:
		case COMPRESSION_ZLIB8:
		case COMPRESSION_ZLIB9:
		case COMPRESSION_BTREE:
		case COMPRESSION_HUFF:
		case COMPRESSION_REFPACK:
			return *(Int *)(((UnsignedByte *)mem)+4);
	}

	return len;
}

Int CompressionManager::compressData( CompressionType compType, void *srcVoid, Int srcLen, void *destVoid, Int destLen )
{
	if (destLen < 8)
		return 0;

	destLen -= 8;

	UnsignedByte *src = (UnsignedByte *)srcVoid;
	UnsignedByte *dest = (UnsignedByte *)destVoid;

	if (compType == COMPRESSION_BTREE)
	{
		memcpy(dest, "EAB\0", 4);
		*(Int *)(dest+4) = 0;
		Int ret = BTREE_encode(dest+8, src, srcLen);
		if (ret)
		{
			*(Int *)(dest+4) = srcLen;
			return ret + 8;
		}
		else
			return 0;
	}
	if (compType == COMPRESSION_HUFF)
	{
		memcpy(dest, "EAH\0", 4);
		*(Int *)(dest+4) = 0;
		Int ret = HUFF_encode(dest+8, src, srcLen);
		if (ret)
		{
			*(Int *)(dest+4) = srcLen;
			return ret + 8;
		}
		else
			return 0;
	}
	if (compType == COMPRESSION_REFPACK)
	{
		memcpy(dest, "EAR\0", 4);
		*(Int *)(dest+4) = 0;
		Int ret = REF_encode(dest+8, src, srcLen);
		if (ret)
		{
			*(Int *)(dest+4) = srcLen;
			return ret + 8;
		}
		else
			return 0;
	}

	if (compType == COMPRESSION_NOXLZH)
	{
		memcpy(dest, "NOX\0", 4);
		*(Int *)(dest+4) = 0;
		Bool ret = CompressMemory(src, srcLen, dest+8, destLen);
		if (ret)
		{
			*(Int *)(dest+4) = srcLen;
			return destLen + 8;
		}
		else
			return 0;
	}

	if (compType >= COMPRESSION_ZLIB1 && compType <= COMPRESSION_ZLIB9)
	{
		Int level = compType - COMPRESSION_ZLIB1 + 1; // 1-9
		memcpy(dest, "ZL0\0", 4);
		dest[2] = '0' + level;
		*(Int *)(dest+4) = 0;

		unsigned long outLen = destLen;
		Int err = compress2( (Bytef*)dest+8, &outLen, (const Bytef*)src, srcLen, level );

		if (err == Z_OK || err == Z_STREAM_END)
		{
			*(Int *)(dest+4) = srcLen;
			return outLen + 8;
		}
		else
		{
			DEBUG_LOG(("ZLib compression error (level is %d, src len is %d) %d", level, srcLen, err));
			return 0;
		}
	}

	return 0;
}

Int CompressionManager::decompressData( void *srcVoid, Int srcLen, void *destVoid, Int destLen )
{
	if (srcLen < 8)
		return 0;

	UnsignedByte *src = (UnsignedByte *)srcVoid;
	UnsignedByte *dest = (UnsignedByte *)destVoid;

	CompressionType compType = getCompressionType(src, srcLen);

	if (compType == COMPRESSION_BTREE)
	{
		Int slen = srcLen - 8;
		Int ret = BTREE_decode(dest, src+8, &slen);
		if (ret)
			return ret;
		else
			return 0;
	}
	if (compType == COMPRESSION_HUFF)
	{
		Int slen = srcLen - 8;
		Int ret = HUFF_decode(dest, src+8, &slen);
		if (ret)
			return ret;
		else
			return 0;
	}
	if (compType == COMPRESSION_REFPACK)
	{
		Int slen = srcLen - 8;
		Int ret = REF_decode(dest, src+8, &slen);
		if (ret)
			return ret;
		else
			return 0;
	}

	if (compType == COMPRESSION_NOXLZH)
	{
		Bool ret = DecompressMemory(src+8, srcLen-8, dest, destLen);
		if (ret)
			return destLen;
		else
			return 0;
	}

	if (compType >= COMPRESSION_ZLIB1 && compType <= COMPRESSION_ZLIB9)
	{
		unsigned long outLen = destLen;
		Int err = uncompress((Bytef*)dest, &outLen, (const Bytef*)src+8, srcLen-8);
		if (err == Z_OK || err == Z_STREAM_END)
		{
			return outLen;
		}
		else
		{
			DEBUG_LOG(("ZLib decompression error (src is level %d, %d bytes long) %d",
				compType - COMPRESSION_ZLIB1 + 1 /* 1-9 */, srcLen, err));
			return 0;
		}
	}

	return 0;
}

///////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////
/////  Performance Testing  ///////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////

#ifdef TEST_COMPRESSION

#include "GameClient/MapUtil.h"
#include "Common/FileSystem.h"
#include "Common/file.h"

#include "Common/PerfTimer.h"
enum { NUM_TIMES = 10 };

struct CompData
{
public:
	Int origSize;
	Int compressedSize[COMPRESSION_COUNT];
};

void DoCompressTest()
{

	Int i;

	PerfGather *s_compressGathers[COMPRESSION_COUNT];
	PerfGather *s_decompressGathers[COMPRESSION_COUNT];
	for (i = 0; i < COMPRESSION_COUNT; ++i)
	{
		s_compressGathers[i] = new PerfGather(CompressionManager::getCompressionNameByType((CompressionType)i));
		s_decompressGathers[i] = new PerfGather(CompressionManager::getDecompressionNameByType((CompressionType)i));
	}

	std::map<AsciiString, CompData> s_sizes;

	std::map<AsciiString, MapMetaData>::const_iterator it = TheMapCache->begin();
	while (it != TheMapCache->end())
	{
		//if (it->second.m_isOfficial)
		//{
			//++it;
			//continue;
		//}
		//static Int count = 0;
		//if (count++ > 2)
			//break;
		File *f = TheFileSystem->openFile(it->first.str());
		if (f)
		{
			DEBUG_LOG(("***************************\nTesting '%s'\n", it->first.str()));
			Int origSize = f->size();
			UnsignedByte *buf = (UnsignedByte *)f->readEntireAndClose();
			UnsignedByte *uncompressedBuf = NEW UnsignedByte[origSize];

			CompData d = s_sizes[it->first];
			d.origSize = origSize;
			d.compressedSize[COMPRESSION_NONE] = origSize;

			for (i=COMPRESSION_MIN; i<=COMPRESSION_MAX; ++i)
			{
				DEBUG_LOG(("================================================="));
				DEBUG_LOG(("Compression Test %d", i));

				Int maxCompressedSize = CompressionManager::getMaxCompressedSize( origSize, (CompressionType)i );
				DEBUG_LOG(("Orig size is %d, max compressed size is %d bytes", origSize, maxCompressedSize));

				UnsignedByte *compressedBuf = NEW UnsignedByte[maxCompressedSize];
				memset(compressedBuf, 0, maxCompressedSize);
				memset(uncompressedBuf, 0, origSize);

				Int compressedLen, decompressedLen;

				for (Int j=0; j < NUM_TIMES; ++j)
				{
					s_compressGathers[i]->startTimer();
					compressedLen = CompressionManager::compressData((CompressionType)i, buf, origSize, compressedBuf, maxCompressedSize);
					s_compressGathers[i]->stopTimer();
					s_decompressGathers[i]->startTimer();
					decompressedLen = CompressionManager::decompressData(compressedBuf, compressedLen, uncompressedBuf, origSize);
					s_decompressGathers[i]->stopTimer();
				}
				d.compressedSize[i] = compressedLen;
				DEBUG_LOG(("Compressed len is %d (%g%% of original size)", compressedLen, (double)compressedLen/(double)origSize*100.0));
				DEBUG_ASSERTCRASH(compressedLen, ("Failed to compress"));
				DEBUG_LOG(("Decompressed len is %d (%g%% of original size)", decompressedLen, (double)decompressedLen/(double)origSize*100.0));

				DEBUG_ASSERTCRASH(decompressedLen == origSize, ("orig size does not match compressed+uncompressed output"));
				if (decompressedLen == origSize)
				{
					Int ret = memcmp(buf, uncompressedBuf, origSize);
					if (ret != 0)
					{
						DEBUG_CRASH(("orig buffer does not match compressed+uncompressed output - ret was %d", ret));
					}
				}

				delete compressedBuf;
				compressedBuf = nullptr;
			}

			DEBUG_LOG(("d = %d -> %d", d.origSize, d.compressedSize[i]));
			s_sizes[it->first] = d;
			DEBUG_LOG(("s_sizes[%s] = %d -> %d", it->first.str(), s_sizes[it->first].origSize, s_sizes[it->first].compressedSize[i]));

			delete[] buf;
			buf = nullptr;

			delete[] uncompressedBuf;
			uncompressedBuf = nullptr;
		}

		++it;
	}

	for (i=COMPRESSION_MIN; i<=COMPRESSION_MAX; ++i)
	{
		Real maxCompression = 1000.0f;
		Real minCompression = 0.0f;
		Int totalUncompressedBytes = 0;
		Int totalCompressedBytes = 0;
		for (std::map<AsciiString, CompData>::iterator cd = s_sizes.begin(); cd != s_sizes.end(); ++cd)
		{
			CompData d = cd->second;

			Real ratio = d.compressedSize[i]/(Real)d.origSize;
			maxCompression = min(maxCompression, ratio);
			minCompression = max(minCompression, ratio);

			totalUncompressedBytes += d.origSize;
			totalCompressedBytes += d.compressedSize[i];
		}
		DEBUG_LOG(("***************************************************"));
		DEBUG_LOG(("Compression method %s:", CompressionManager::getCompressionNameByType((CompressionType)i)));
		DEBUG_LOG(("%d bytes compressed to %d (%g%%)", totalUncompressedBytes, totalCompressedBytes,
			totalCompressedBytes/(Real)totalUncompressedBytes*100.0f));
		DEBUG_LOG(("Min ratio: %g%%, Max ratio: %g%%",
			minCompression*100.0f, maxCompression*100.0f));
		DEBUG_LOG((""));
	}

	PerfGather::dumpAll(10000);
	//PerfGather::displayGraph(TheGameLogic->getFrame());
	PerfGather::resetAll();
	CopyFile( "AAAPerfStats.csv", "AAACompressPerfStats.csv", FALSE );

	for (i = 0; i < COMPRESSION_COUNT; ++i)
	{
		delete s_compressGathers[i];
		s_compressGathers[i] = nullptr;

		delete s_decompressGathers[i];
		s_decompressGathers[i] = nullptr;
	}

}

#endif // TEST_COMPRESSION
```

### `codex.h` (`Core/Libraries/Source/Compression/EAC/codex.h`)

```cpp
/* Copyright (C) Electronic Arts Canada Inc. 1995-2002.  All rights reserved. */

/*------------------------------------------------------------------*/
/*                                                                  */
/*         Compression Decompression Exchange(CODEX) v2.00          */
/*                                                                  */
/*                     by FrANK G. Barchard, EAC                    */
/*                                                                  */
/*                    Header Module - Oct 15, 2001                  */
/*                                                                  */
/*------------------------------------------------------------------*/
/*                                                                  */
/* Version Date   SE History                                        */
/* ------- ------ -- -------                                        */
/*    1.00 990824 FB codex API separated from huff tool             */
/*    1.01 010427 FB fb6 32 bit size header                         */
/*    1.02 011011 FB c++ defaults                                   */
/*    2.00 011015 FB bool, dest/source, new about struct,no QPUBLIC */
/*                                                                  */
/*------------------------------------------------------------------*/

#pragma once

#ifdef __cplusplus
extern "C" {
#endif

#define CODEX_VERSION 200

/****************************************************************/
/*  Data Types                                                  */
/****************************************************************/

/* Info structure describing bitmaps */

typedef struct
{
    int signature;              /* signature of codex ie 'tga ' (optional) */
    int size;                   /* size of CODEXABOUT structure */
    int version;                /* version number of CODEXABOUT structure (200) */

    unsigned int decode    :1;  /* supports decoding */
    unsigned int encode    :1;  /* supports encoding */
    unsigned int size32    :1;  /* support 32 bit size field */
    unsigned int pad       :29;

    char versionstr[8];          /* version number of codex module ie 1.00 */
    char shorttypestr[8];        /* 3 or 4 character type string ie ref */
    char longtypestr[16];        /* full name of data format ie Refpack */
} CODEXABOUT;

#define QMAKEID(a,b,c,d) (((a)<<24)|((b)<<16)|((c)<<8)|(d))

#if !defined(GCALL)
#if defined(_MSC_VER)
#define GCALL __stdcall
#else
#define GCALL
#endif
#endif


typedef struct QFUNCTIONS
{
    CODEXABOUT * (GCALL * CODEX_about)();
    bool         (GCALL * CODEX_is)(const void *compressed);
    int          (GCALL * CODEX_size)(const void *compressed);
    int          (GCALL * CODEX_decode)(void *dest, const void *source, int *sourcesizeptr);
    int          (GCALL * CODEX_encode)(void *dest, const void *source, int sourcesize, int *opts);
} QFUNCTIONS;

extern struct QFUNCTIONS qfunctions[];

/****************************************************************/
/*  Codex Module Example Prototypes                             */
/****************************************************************/

#include "gimex.h" /* for memory IO */

/* Information Functions */

CODEXABOUT *GCALL CODEX_about();
bool        GCALL CODEX_is(const void *source);
int         GCALL CODEX_size(const void *source);

/* Decode/Encode Functions */

#ifdef __cplusplus
int GCALL CODEX_decode(void *dest, const void *source, int *sourcesizeptr=0);
int GCALL CODEX_encode(void *dest, const void *source, int sourcesize, int *opts=0);
#else
int GCALL CODEX_decode(void *dest, const void *source, int *sourcesizeptr);
int GCALL CODEX_encode(void *dest, const void *source, int sourcesize, int *opts);
#endif

#ifdef __cplusplus
}
#endif
```

### `gimex.h (byte-order helpers only)` (`Core/Libraries/Source/Compression/EAC/gimex.h`)

Excerpt: lines 364-491, the `ggetm`/`ggeti`/`gputm`/`gputi` helpers. The rest of the file is the unrelated GIMEX image API. Port only the portable `#else` branches (big-endian `ggetm`/`gputm`), never the `__APPLE__` branches.

```cpp
/****************************************************************************/
/* Inline C++ (cross platform)                                              */
/****************************************************************************/

#ifdef _MSC_VER
#pragma warning(disable: 4514) // unreferenced inline function has been removed
#endif

/* get motorola memory */
static __inline unsigned int ggetm(const void *src, int bytes)
{
    if (bytes==1)
        return (unsigned int) *(const unsigned char *) src;
#if defined(__APPLE__) || (defined(__MWERKS__) && defined(__PPCGEKKO__))
    else if (bytes==2)
        return (unsigned int) *(const unsigned short *) src;
    else if (bytes==4)
        return (unsigned int) *(const unsigned int *) src;
#else
    else if (bytes==2)
        return (((unsigned int) *(const unsigned char *) src)<<8) | (((unsigned int) *((const unsigned char *) src+1)));
    else if (bytes==4)
        return (((unsigned int) *(const unsigned char *) src)<<24) | (((unsigned int) *((const unsigned char *) src+1))<<16) | (((unsigned int) *((const unsigned char *) src+2))<<8) | (((unsigned int) *((const unsigned char *) src+3)));
#endif
    else if (bytes==3)
        return (((unsigned int) *(const unsigned char *) src)<<16) | (((unsigned int) *((const unsigned char *) src+1))<<8) | (((unsigned int) *((const unsigned char *) src+2)));
    return 0;
}

/* get intel memory */

static __inline unsigned int ggeti(const void *src, int bytes)
{
    if (bytes==1)
        return (unsigned int) *(const unsigned char *) src;
#if defined(_MSC_VER) || defined(__i386__)
    else if (bytes==2)
        return (unsigned int) *(const unsigned short *) src;
    else if (bytes==4)
        return (unsigned int) *(const unsigned int *) src;
#else
    else if (bytes==2)
        return (((unsigned int) *((const unsigned char *) src+1))<<8) | (((unsigned int) *(const unsigned char *) src+0));
    else if (bytes==4)
        return (((unsigned int) *((const unsigned char *) src+3))<<24) | (((unsigned int) *((const unsigned char *) src+2))<<16) | (((unsigned int) *((const unsigned char *) src+1))<<8) | (((unsigned int) *(const unsigned char *) src+0));
#endif
    else if (bytes==3)
        return (((unsigned int) *((const unsigned char *) src+2))<<16) | (((unsigned int) *((const unsigned char *) src+1))<<8) | (((unsigned int) *(const unsigned char *) src+0));
    return 0;
}

/* put motorolla memory */

static __inline void gputm(void *dst, unsigned int data, int bytes)
{
    if (bytes==1)
    {
        ((unsigned char *) dst)[0] = (unsigned char) data;
    }
#if defined(__APPLE__) || (defined(__MWERKS__) && defined(__PPCGEKKO__))
    else if (bytes==2)
    {
        ((unsigned short *) dst)[0] = (unsigned short) data;
    }
    else if (bytes==4)
    {
        ((unsigned int *) dst)[0] = (unsigned int) data;
    }
#else
    else if (bytes==2)
    {
        ((unsigned char *) dst)[0] = (unsigned char) (data>>8);
        ((unsigned char *) dst)[1] = (unsigned char) data;
    }
    else if (bytes==4)
    {
        ((unsigned char *) dst)[0] = (unsigned char) (data>>24);
        ((unsigned char *) dst)[1] = (unsigned char) (data>>16);
        ((unsigned char *) dst)[2] = (unsigned char) (data>>8);
        ((unsigned char *) dst)[3] = (unsigned char) data;
    }
#endif
    else if (bytes==3)
    {
        ((unsigned char *) dst)[0] = (unsigned char) (data>>16);
        ((unsigned char *) dst)[1] = (unsigned char) (data>>8);
        ((unsigned char *) dst)[2] = (unsigned char) data;
    }
}

/* put intel memory */

static __inline void gputi(void *dst, unsigned int data, int bytes)
{
    if (bytes==1)
    {
        ((unsigned char *) dst)[0] = (unsigned char) data;
    }
#if defined(_MSC_VER) || defined(__i386__)
    else if (bytes==2)
    {
        ((unsigned short *) dst)[0] = (unsigned short) data;
    }
    else if (bytes==4)
    {
        ((unsigned int *) dst)[0] = (unsigned int) data;
    }
#else
    else if (bytes==2)
    {
        ((unsigned char *) dst)[0] = (unsigned char) data;
        ((unsigned char *) dst)[1] = (unsigned char) (data>>8);
    }
    else if (bytes==4)
    {
        ((unsigned char *) dst)[0] = (unsigned char) data;
        ((unsigned char *) dst)[1] = (unsigned char) (data>>8);
        ((unsigned char *) dst)[2] = (unsigned char) (data>>16);
        ((unsigned char *) dst)[3] = (unsigned char) (data>>24);
    }
#endif
    else if (bytes==3)
    {
        ((unsigned char *) dst)[0] = (unsigned char) data;
        ((unsigned char *) dst)[1] = (unsigned char) (data>>8);
        ((unsigned char *) dst)[2] = (unsigned char) (data>>16);
    }
}
```

### `refcodex.h` (`Core/Libraries/Source/Compression/EAC/refcodex.h`)

```cpp
/* Copyright (C) Electronic Arts Canada Inc. 1995-2002.  All rights reserved. */


#ifdef __cplusplus
extern "C" {
#endif

#include "codex.h"

/****************************************************************/
/*  REF Codex                                                   */
/****************************************************************/

/* Information Functions */

CODEXABOUT *GCALL REF_about();
bool        GCALL REF_is(const void *compresseddata);

/* Decode Functions */

int        GCALL REF_size(const void *compresseddata);
#ifdef __cplusplus
int        GCALL REF_decode(void *dest, const void *compresseddata, int *compressedsize=0);
#else
int        GCALL REF_decode(void *dest, const void *compresseddata, int *compressedsize);
#endif

/* Encode Functions */

#ifdef __cplusplus
int        GCALL REF_encode(void *compresseddata, const void *source, int sourcesize, int *opts=0);
#else
int        GCALL REF_encode(void *compresseddata, const void *source, int sourcesize, int *opts);
#endif

/****************************************************************/
/*  Internal                                                    */
/****************************************************************/

#ifndef qmin
#define qmin(a,b) ((a)<(b)?(a):(b))
#endif

#ifndef qmax
#define qmax(a,b) ((a)>(b)?(a):(b))
#endif

#ifdef __cplusplus
}
#endif
```

### `refencode.cpp` (`Core/Libraries/Source/Compression/EAC/refencode.cpp`)

```cpp
// Copyright (C) Electronic Arts Canada Inc. 1995-2002.  All rights reserved.

#ifndef __REFWRITE
#define __REFWRITE 1

#include <string.h>
#include "codex.h"
#include "refcodex.h"

/****************************************************************/
/*  Internal Functions                                          */
/****************************************************************/

static unsigned int matchlen(unsigned char *s,unsigned char *d, unsigned int maxmatch)
{
    unsigned int current;

    for (current=0; current<maxmatch && *s++==*d++; ++current)
        ;

    return(current);
}

#define HASH(cptr) (int)((((unsigned int)(unsigned char)cptr[0]<<8) | ((unsigned int)(unsigned char)cptr[2])) ^ ((unsigned int)(unsigned char)cptr[1]<<4))

static int refcompress(unsigned char *from, int len, unsigned char *dest, int maxback, int quick)
{
    unsigned int tlen;
    unsigned int tcost;
//    unsigned int ccost;    // context cost
    unsigned int run;
    unsigned int toffset;
    unsigned int boffset;
    unsigned int blen;
    unsigned int bcost;
    unsigned int mlen;
    unsigned char *tptr;
    unsigned char *cptr;
    unsigned char *to;
    unsigned char *rptr;
    int countliterals=0;
    int countshort=0;
    int countint=0;
    int countvint=0;
    int hash;
    int hoffset;
    int minhoffset;
    int i;
    int *link;
    int *hashtbl;

    to = dest;
    run = 0;
    cptr = rptr = from;

    if ((unsigned int)maxback > (unsigned int)131071)
        maxback = 131071;

	hashtbl = (int *) galloc(65536L*sizeof(int));
	if (!hashtbl)
        return(0);
	link = (int *) galloc(131072L*sizeof(int));
	if (!link)
	{
	    gfree(hashtbl);
	    return(0);
	}

    memset(hashtbl,-1,65536L*sizeof(int));

    len -= 4;
    while (len>=0)
    {
        boffset = 0;
        blen = 2;
        bcost = 2;
//        ccost = 0;
        mlen = qmin(len,1028);
        tptr=cptr-1;
        hash = HASH(cptr);
        hoffset = hashtbl[hash];
        minhoffset = qmax(cptr-from-131071,0);


        if (hoffset>=minhoffset)
        {
            do
            {
                tptr = from+hoffset;
                if (cptr[blen]==tptr[blen])
                {
                    tlen = matchlen(cptr,tptr,mlen);
                    if (tlen > blen)
                    {
                        toffset = (cptr-1)-tptr;
                        if (toffset<1024 && tlen<=10)       /* two byte int form */
                            tcost = 2;
                        else if (toffset<16384 && tlen<=67) /* three byte int form */
                            tcost = 3;
                        else                                /* four byte very int form */
                            tcost = 4;

                        if (tlen-tcost+4 > blen-bcost+4)
                        {
                            blen = tlen;
                            bcost = tcost;
                            boffset = toffset;
                            if (blen>=1028) break;
                        }
                    }
                }
            } while ((hoffset = link[hoffset&131071]) >= minhoffset);
        }

//        ccost = 0;
//        if ((run<4) && ((run+blen)>=4))
//            ccost = 1;  // extra packet cost to switch out of literal into reference

//        if (bcost>blen || (blen<=2 && bcost==blen && !ccost) || (len<4))
        if (bcost>=blen || len<4)
        {
            hoffset = (cptr-from);
            link[hoffset&131071] = hashtbl[hash];
            hashtbl[hash] = hoffset;

            ++run;
            ++cptr;
            --len;
        }
        else
        {
            while (run>3)                   /* literal block of data */
            {
                tlen = qmin(112,run&~3);
                run -= tlen;
                *to++ = (unsigned char) (0xe0+(tlen>>2)-1);
                memcpy(to,rptr,tlen);
                rptr += tlen;
                to += tlen;
                ++countliterals;
            }
            if (bcost==2)                   /* two byte int form */
            {
                *to++ = (unsigned char) (((boffset>>8)<<5) + ((blen-3)<<2) + run);
                *to++ = (unsigned char) boffset;
                ++countshort;
            }
            else if (bcost==3)              /* three byte int form */
            {
                *to++ = (unsigned char) (0x80 + (blen-4));
                *to++ = (unsigned char) ((run<<6) + (boffset>>8));
                *to++ = (unsigned char) boffset;
                ++countint;
            }
            else                            /* four byte very int form */
            {
                *to++ = (unsigned char) (0xc0 + ((boffset>>16)<<4) + (((blen-5)>>8)<<2) + run);
                *to++ = (unsigned char) (boffset>>8);
                *to++ = (unsigned char) (boffset);
                *to++ = (unsigned char) (blen-5);
                ++countvint;
            }
            if (run)
            {
                memcpy(to, rptr, run);
                to += run;
                run = 0;
            }

            if (quick)
            {
                hoffset = (cptr-from);
                link[hoffset&131071] = hashtbl[hash];
                hashtbl[hash] = hoffset;
                cptr += blen;
            }
            else
            {
                for (i=0; i < (int)blen; ++i)
                {
                    hash = HASH(cptr);
                    hoffset = (cptr-from);
                    link[hoffset&131071] = hashtbl[hash];
                    hashtbl[hash] = hoffset;
                    ++cptr;
                }
            }

            rptr = cptr;
            len -= blen;
        }
    }
    len += 4;
    run += len;
    while (run>3)                       /* no match at end, use literal */
    {
        tlen = qmin(112,run&~3);
        run -= tlen;
        *to++ = (unsigned char) (0xe0+(tlen>>2)-1);
        memcpy(to,rptr,tlen);
        rptr += tlen;
        to += tlen;
    }

    *to++ = (unsigned char) (0xfc+run); /* end of stream command + 0..3 literal */
    if (run)
    {
        memcpy(to,rptr,run);
        to += run;
    }

	gfree(link);
	gfree(hashtbl);
    return(to-dest);
}


/****************************************************************/
/*  Encode Function                                             */
/****************************************************************/

int GCALL REF_encode(void *compresseddata, const void *source, int sourcesize, int *opts)
{
    int    maxback=131072;
    int     quick=0;
    int    plen;
    int    hlen;


    /* simple fb6 header */

    if (sourcesize>0xffffff)  // 32 bit header required
    {
        gputm(compresseddata,   (unsigned int) 0x90fb, 2);
        gputm((char *)compresseddata+2, (unsigned int) sourcesize, 4);
        hlen = 6L;
    }
    else
    {
        gputm(compresseddata,   (unsigned int) 0x10fb, 2);
        gputm((char *)compresseddata+2, (unsigned int) sourcesize, 3);
        hlen = 5L;
    }
    plen = hlen+refcompress((unsigned char *)source, sourcesize, (unsigned char *)compresseddata+hlen, maxback, quick);
    return(plen);
}

#endif
```

### `refdecode.cpp` (`Core/Libraries/Source/Compression/EAC/refdecode.cpp`)

```cpp
// Copyright (C) Electronic Arts Canada Inc. 1995-2002.  All rights reserved.

#ifndef __REFREAD
#define __REFREAD 1

#include <string.h>
#include "codex.h"
#include "refcodex.h"

/****************************************************************/
/*  Information Functions                                       */
/****************************************************************/

/* check for reasonable header: */
/* 10fb header */

bool GCALL REF_is(const void *compresseddata)
{
    bool ok=false;
    int packtype=ggetm(compresseddata,2);

    if (packtype==0x10fb ||
     packtype==0x11fb ||
     packtype==0x90fb ||
     packtype==0x91fb)
        ok = true;

    return(ok);
}


/****************************************************************/
/*  Decode Functions                                            */
/****************************************************************/

int GCALL REF_size(const void *compresseddata)
{
    int len=0;
    int packtype=ggetm(compresseddata,2);
    int ssize=(packtype&0x8000)?4:3;

    if (packtype&0x100)     /* 11fb */
    {
        len = ggetm((char *)compresseddata+2+ssize,ssize);
    }
    else                    /* 10fb */
    {
        len = ggetm((char *)compresseddata+2,ssize);
    }

    return(len);
}


int GCALL REF_decode(void *dest, const void *compresseddata, int *compressedsize)
{
    unsigned char *s;
    unsigned char *ref;
    unsigned char *d;
    unsigned char first;
    unsigned char second;
    unsigned char third;
    unsigned char forth;
    unsigned int  run;
    unsigned int  type;
    int          ulen;

    s = (unsigned char *) compresseddata;
    d = (unsigned char *) dest;
    ulen = 0L;

    if (s)
    {
        type = *s++;
        type = (type<<8) + *s++;

        if (type&0x8000) /* 4 byte size field */
        {
            if (type&0x100)                       /* skip ulen */
                s += 4;

            ulen = *s++;
            ulen = (ulen<<8) + *s++;
            ulen = (ulen<<8) + *s++;
            ulen = (ulen<<8) + *s++;
        }
        else
        {
            if (type&0x100)                       /* skip ulen */
                s += 3;

            ulen = *s++;
            ulen = (ulen<<8) + *s++;
            ulen = (ulen<<8) + *s++;
        }

        for (;;)
        {
            first = *s++;
            if (!(first&0x80))          /* short form */
            {
                second = *s++;
                run = first&3;
                while (run--)
                    *d++ = *s++;
                ref = d-1 - (((first&0x60)<<3) + second);
                run = ((first&0x1c)>>2)+3-1;
                do
                {
                    *d++ = *ref++;
                } while (run--);
                continue;
            }
            if (!(first&0x40))          /* int form */
            {
                second = *s++;
                third = *s++;
                run = second>>6;
                while (run--)
                    *d++ = *s++;

                ref = d-1 - (((second&0x3f)<<8) + third);

                run = (first&0x3f)+4-1;
                do
                {
                    *d++ = *ref++;
                } while (run--);
                continue;
            }
            if (!(first&0x20))          /* very int form */
            {
                second = *s++;
                third = *s++;
                forth = *s++;
                run = first&3;
                while (run--)
                    *d++ = *s++;

                ref = d-1 - (((first&0x10)>>4<<16) +  (second<<8) + third);

                run = ((first&0x0c)>>2<<8) + forth + 5-1;
                do
                {
                    *d++ = *ref++;
                } while (run--);
                continue;
            }
            run = ((first&0x1f)<<2)+4;  /* literal */
            if (run<=112)
            {
                while (run--)
                    *d++ = *s++;
                continue;
            }
            run = first&3;              /* eof (+0..3 literal) */
            while (run--)
                *d++ = *s++;
            break;
        }
    }
    if (compressedsize)
        *compressedsize = (int)((char *)s-(char *)compresseddata);
    return(ulen);
}

#endif
```

### `btreecodex.h` (`Core/Libraries/Source/Compression/EAC/btreecodex.h`)

```cpp
/* Copyright (C) Electronic Arts Canada Inc. 1995-2002.  All rights reserved. */


#ifdef __cplusplus
extern "C" {
#endif

#include "codex.h"

/****************************************************************/
/*  BTR Codex                                                   */
/****************************************************************/

/* Information Functions */

CODEXABOUT *GCALL BTREE_about();
bool        GCALL BTREE_is(const void *compresseddata);

/* Decode Functions */

int        GCALL BTREE_size(const void *compresseddata);
#ifdef __cplusplus
int        GCALL BTREE_decode(void *dest, const void *compresseddata, int *compressedsize=0);
#else
int        GCALL BTREE_decode(void *dest, const void *compresseddata, int *compressedsize);
#endif

/* Encode Functions */

#ifdef __cplusplus
int        GCALL BTREE_encode(void *compresseddata, const void *source, int sourcesize, int *opts=0);
#else
int        GCALL BTREE_encode(void *compresseddata, const void *source, int sourcesize, int *opts);
#endif

#ifdef __cplusplus
}
#endif
```

### `btreeencode.cpp` (`Core/Libraries/Source/Compression/EAC/btreeencode.cpp`)

```cpp
// Copyright (C) Electronic Arts Canada Inc. 1995-2002.  All rights reserved.

#ifndef __BTRWRITE
#define __BTRWRITE 1

#include <string.h>
#include "codex.h"
#include "btreecodex.h"

/****************************************************************/
/*  Internal Functions                                          */
/****************************************************************/

#define BTREEWORD	 short
#define BTREECODES	 256
#define BTREEBIGNUM	 32000
#define	BTREESLOPAGE 16384

struct BTREEMemStruct
{
	char	*ptr;
	int	len;
};

struct BTreeEncodeContext
{
    unsigned int	packbits;
    unsigned int	workpattern;
    unsigned char  *bufptr;
    unsigned int	ulen;
    unsigned int	masks[17];
    unsigned char	clueq[BTREECODES];
    unsigned char	right[BTREECODES];
    unsigned char	join[BTREECODES];
    unsigned int	plen;
    unsigned char  *bufbase;
    unsigned char  *bufend;
    unsigned char  *buffer;
    unsigned char  *buf1;
    unsigned char  *buf2;
};

static void BTREE_writebits(struct BTreeEncodeContext *EC,
                            struct BTREEMemStruct *dest,
                            unsigned int bitpattern,
                            unsigned int len)
{
	if (len > 16)
	{
		BTREE_writebits(EC,dest,(unsigned int) (bitpattern>>16), len-16);
		BTREE_writebits(EC,dest,(unsigned int) bitpattern, 16);
	}
	else
	{
		EC->packbits += len;
		EC->workpattern += (bitpattern & EC->masks[len]) << (24-EC->packbits);
		while (EC->packbits > 7)
		{
			*(dest->ptr+dest->len) = (unsigned char) (EC->workpattern >> 16);
			++dest->len;

			EC->workpattern = EC->workpattern << 8;
			EC->packbits -= 8;
			++EC->plen;
		}
	}
}


static void BTREE_adjcount(unsigned char *s, unsigned char *bend, BTREEWORD *count)
{

	unsigned BTREEWORD		i;

    #define COUNTADJ(j)	i = (BTREEWORD)(((i<<8) | *(s+j)));	++*(count+(int)i);

	i = (unsigned BTREEWORD) *s++;

	bend -= 16;

	if (s<bend)
	{
		do
		{
			COUNTADJ(0);
			COUNTADJ(1);
			COUNTADJ(2);
			COUNTADJ(3);

			COUNTADJ(4);
			COUNTADJ(5);
			COUNTADJ(6);
			COUNTADJ(7);

			COUNTADJ(8);
			COUNTADJ(9);
			COUNTADJ(10);
			COUNTADJ(11);

			COUNTADJ(12);
			COUNTADJ(13);
			COUNTADJ(14);
			COUNTADJ(15);
            s += 16;

		} while (s<bend);
	}

	bend += 16;

	if (s<bend)
	{
		do
		{
			COUNTADJ(0);
            ++s;

		} while (s<bend);
	}
}

/* Clear 128 ints (256 words). Done as ints for speed. */

static void BTREE_clearcount(unsigned char *tryq,BTREEWORD *countbuf)
{
	int		zero=0;
	int					i;
	int					j;
	int				*intptr;

	intptr = (int *) countbuf;
	j = 256;
	do
	{
		if (*tryq)
		{
			*tryq++ = 1;

			i = 8;
			do
			{
				*(intptr+0) = zero;
				*(intptr+1) = zero;
				*(intptr+2) = zero;
				*(intptr+3) = zero;

				*(intptr+4) = zero;
				*(intptr+5) = zero;
				*(intptr+6) = zero;
				*(intptr+7) = zero;

				*(intptr+8) = zero;
				*(intptr+9) = zero;
				*(intptr+10) = zero;
				*(intptr+11) = zero;

				*(intptr+12) = zero;
				*(intptr+13) = zero;
				*(intptr+14) = zero;
				*(intptr+15) = zero;
                intptr += 16;

			} while (--i);
		}
		else
		{
			++tryq;
			intptr += 128;
		}
	} while (--j);
}

static void BTREE_joinnodes(struct BTreeEncodeContext *EC,
                            unsigned char *cluep,
                            unsigned char *rightp,
                            unsigned char *joinp,
                            unsigned int  clue)
{
	unsigned char			*s;
	unsigned char			*d;
	unsigned char			*bend;
	unsigned int			i;

    /* pack file */

    s = EC->bufbase;
	if (s==EC->buf1)
	{
		d = EC->buf2;
	}
	else
    {
		d = EC->buf1;
	}
	EC->bufbase = d;
	bend = EC->bufend;

	*bend = (unsigned char) clue;
	while (s<=bend)
	{
		while (!cluep[(unsigned int) (*d++ = *s++)])
			;

		i = (unsigned int) *(s-1);

		if (cluep[i]==1)
		{	if (*s==rightp[i])
			{	*(d-1) = joinp[i];
				++s;
			}
		}
		else
		{	if (cluep[i]==3)
			{	*(d-1) = (unsigned char) clue;
				*d++ = *(s-1);
			}
			else
				*d++ = *s++;
		}
	}
	EC->bufend = d-2;
}


/* find 48 most common nodes */

static unsigned int	BTREE_findbest(BTREEWORD     *countptr,
                                   unsigned char *tryq,
                                   unsigned int  *bestn,
                                   unsigned int  *bestval,
                                   int			  ratio)
{
	unsigned int				i;
	unsigned int				i1;
	unsigned int				i2;
	unsigned int				bestsize;

	unsigned int				i3;
	unsigned int				l;

	bestsize = 1;
	i = 3;
	l = 0;
	for (i2=0;i2<256;++i2)
	{
		if (tryq[i2])
		{	for (i1=0;i1<256;++i1)
			{
				if (*countptr++>(BTREEWORD)i)
				{
					if (tryq[i1])
					{	i = *(countptr-1);
						i3=bestsize;
						while (bestval[i3-1]<i)
						{
							bestn[i3] = bestn[i3-1];
							bestval[i3] = bestval[i3-1];
							--i3;
						}
						bestn[i3] = l+i1;
						bestval[i3] = i;
						if (bestsize<48)
							++bestsize;
						while (bestval[bestsize-1]<(bestval[1]/ratio))
							--bestsize;
						if (bestsize<48)
							i = bestval[1]/ratio;
						else
							i = bestval[bestsize-1];
					}
				}
			}
		}
		else
			countptr += 256;
		l += 256;
	}
	return(bestsize);
}

static void BTREE_treepack(struct BTreeEncodeContext *EC,
                           struct BTREEMemStruct *dest,
                           unsigned int     passes,
                           unsigned int     multimax,
                           unsigned int     quick,
                          int				 zerosuppress)
{

	unsigned char	*bend;
	unsigned char	*ptr1;
	unsigned char	*treebuf;
	BTREEWORD		*count;
	unsigned int	i;
	unsigned int	i1;

	int				joinnode, leftnode, rightnode;
	int				ratio;
	unsigned int	hlen;
	unsigned int	domore;
	unsigned int	cost, save;
	unsigned int	tcost, tsave;
    unsigned int	clue;

	unsigned int	freeptr;
	unsigned int	count2[BTREECODES];
	unsigned char	tryq[BTREECODES];
	unsigned char	freeq[BTREECODES];

	unsigned int	bestsize;
	unsigned char	bestjoin[BTREECODES];
	unsigned int	bestn[BTREECODES];
	unsigned int	bestval[BTREECODES];

	unsigned int	bt_size;
	unsigned int	bt_node[BTREECODES];
	unsigned int	bt_left[BTREECODES];
	unsigned int	bt_right[BTREECODES];
    unsigned int	sortptr[BTREECODES];

	int			treebufsize;
	int			buf1size;
	int			buf2size;

    // 3/2 allows for worst case, where 2nd most popular
	treebufsize = 65536L*sizeof(BTREEWORD);  /* 131K */
	buf1size = EC->ulen*3/2+(int)BTREESLOPAGE;
	buf2size = EC->ulen*3/2+(int)BTREESLOPAGE;

	treebuf =	(unsigned char *) galloc(treebufsize);
	if (!treebuf)
        return; /* failure Insufficient memory for work buffer */

	EC->buf1 =	(unsigned char *) galloc(buf1size);
	if (!EC->buf1)
	{
		gfree(treebuf);
		return; /* failure Insufficient memory for work buffer */
	}

	EC->buf2 =	(unsigned char *) galloc(buf2size);
	if (!EC->buf2)
	{
		gfree(treebuf);
		gfree(EC->buf1);
		return; /* failure Insufficient memory for work buffer */
	}

    memcpy(EC->buf1, EC->buffer, EC->ulen); /* copy to scratch buffer */

	EC->buffer = EC->buf1;
	EC->bufptr = EC->buf1+EC->ulen;

	EC->bufbase = EC->buffer;
	EC->bufend = EC->bufptr;

	count = (BTREEWORD *) treebuf;

	if (quick)	ratio = quick;
	else		ratio = 2;

/*** count file ***/

	for (i=0; i<BTREECODES; ++i)
		count2[i] = 0;

	i1 = 0;
	bend = EC->bufptr;
	ptr1 = EC->buffer;
	while (ptr1<bend)
	{	i = (unsigned int) *ptr1++;
		i1 = i1 + i;
		++count2[i];
	}

/* init clue array */

	for (i=0;i<BTREECODES;++i)
		EC->clueq[i] = 0;

/* get a clue byte (forced) */

	for(i=0; i<BTREECODES; ++i)
	{	freeq[i] = 1;
		if (count2[i]>3)
			tryq[i] = 1;
		else
			tryq[i] = 0;
	}

/* don't use 0 for clue or node (so that [clue][0] is reserved for EOF) */

	count2[0] = BTREEBIGNUM;		/* don't use for clue or node */

/* asciiz btree suppress packing of 0..31 */

	if (zerosuppress)
	{
		for (i=0; i<32; ++i)
		{
			count2[i] = BTREEBIGNUM;		/* don't use for nodes */
			tryq[i] = 0;			/* don't try to pack */
			freeq[i] = 0;			/* don't try to pack */
		}
	}


/* sort codes with least used codes first */
/* primary key is count, secondary key is higher code */

/* registers vars usage
    i  - code
    i1 - done flag/swap temp
*/

	for (i=0;i<BTREECODES;++i)
		sortptr[i] = i;

	i1=1;
	while (i1)
	{
		i1 = 0;
		for (i=1; i<BTREECODES; ++i)
		{
			if (count2[sortptr[i]]<count2[sortptr[i-1]] || ((count2[sortptr[i]]==count2[sortptr[i-1]]) && (sortptr[i]>sortptr[i-1])))
			{
				i1 = sortptr[i];
				sortptr[i] = sortptr[i-1];
				sortptr[i-1] = i1;
				i1 = 1;
			}
		}
	}

	freeptr = 0;
	i = sortptr[freeptr++];
	clue = (unsigned char) i;
	freeq[i] = 0;
	tryq[i] = 0;
	EC->clueq[i] = 3;

	if (count2[i])
		BTREE_joinnodes(EC,EC->clueq, EC->right, EC->join, clue);

	EC->clueq[i] = 2;

/* init best array */

	bestval[0] = (unsigned int) -1;


/* count file (pass 2) */

	bt_size = 0;
	domore = passes;
	while (domore)
	{

/* clear count array */

		BTREE_clearcount(tryq,count);

/* do an adjacency count */

		ptr1 = EC->bufbase;
		bend = EC->bufend;

		BTREE_adjcount(ptr1, bend, count);

/* find most common nodes */

		bestsize = BTREE_findbest(count,tryq,bestn,bestval,ratio);

/* decide which nodes should be joined to what */

		domore = 0;
		if (bestsize>1)
		{
			tcost = tsave = 0;
			i = i1 = domore = 1;
			while (domore)
			{
				leftnode = (bestn[i]>>8)&255;
				rightnode = bestn[i]&255;

				if ((tryq[leftnode]==1) && (tryq[rightnode]==1))
				{
					domore = 0;
					while ((freeptr<BTREECODES) && (!freeq[sortptr[freeptr]]))
						++freeptr;

					if (freeptr<BTREECODES)
					{
						joinnode = sortptr[freeptr];

						cost = 3+count2[joinnode];
						save = bestval[i];

						if (cost<save)
						{
							tcost += cost;
							tsave += save;

							bestjoin[i1] = (unsigned char) joinnode;
							bestn[i1] = bestn[i];
							++i1;

							freeq[joinnode] = 0;
							tryq[joinnode] = 2;
							EC->clueq[joinnode] = 3;

							freeq[leftnode] = 0;
							tryq[leftnode] = 2;
							EC->clueq[leftnode] = 1;
							EC->right[leftnode] = (unsigned char) rightnode;
							EC->join[leftnode] =  (unsigned char) joinnode;

							freeq[rightnode] = 0;
							tryq[rightnode] = 2;

							bt_node[bt_size] = joinnode;
							bt_left[bt_size] = leftnode;
							bt_right[bt_size] = rightnode;
							++bt_size;

							if (i1<=multimax)
								domore = 1;
						}
					}
				}

				++i;
				if (i>=bestsize)
					domore = 0;
			}

			bestsize = i1;

			if (bestsize>1)
			{
				/* multijoin nodes */

				BTREE_joinnodes(EC,EC->clueq, EC->right, EC->join, clue);

				/* restore clue table */

				for(i=1;i<bestsize;++i)
				{	leftnode = (bestn[i]>>8)&255;
					joinnode = bestjoin[i];
					EC->clueq[leftnode] = EC->clueq[joinnode] = 0;
				}

				domore = --passes;
			}

		}

	}
	EC->bufptr = EC->bufend;

/* write header */

	BTREE_writebits(EC,dest,(unsigned int) clue, 8);	/* clue byte */
	BTREE_writebits(EC,dest,(unsigned int) bt_size, 8);	/* tree size */

	for (i=0; i<bt_size;++i)
	{	BTREE_writebits(EC,dest,(unsigned int) bt_node[i], 8);
		BTREE_writebits(EC,dest,(unsigned int) bt_left[i], 8);
		BTREE_writebits(EC,dest,(unsigned int) bt_right[i], 8);
	}

	hlen = EC->plen;

/*** write packed file ***/

	ptr1 = EC->bufbase;
	bend = EC->bufend;

	while (ptr1<bend)
		BTREE_writebits(EC,dest,(unsigned int) *ptr1++, 8);

	BTREE_writebits(EC,dest,(unsigned int) clue, 8);
	BTREE_writebits(EC,dest,(unsigned int) 0, 8);

	BTREE_writebits(EC,dest,0L,7);	/* flush bits */

	gfree(EC->buf2);
	gfree(EC->buf1);
	gfree(treebuf);
}

static int BTREE_compressfile(struct BTreeEncodeContext *EC,
                               struct BTREEMemStruct *infile,
                               struct BTREEMemStruct *outfile,
                               int			 ulen,
                               int				 zerosuppress)
{
	unsigned int	i;

	unsigned int	passes;
	unsigned int	multimax;
    int			flen;

/* set defaults */

	EC->packbits = 0;

	EC->workpattern = 0L;
	passes = 256;
	multimax = 32;
	EC->masks[0] = 0;
	for (i=1;i<17;++i)
		EC->masks[i] = (EC->masks[i-1] << 1) + 1;

/* read in a source file */

	EC->buffer = (unsigned char *) (infile->ptr);
	flen = infile->len;

	EC->ulen = flen;
	EC->bufptr = EC->buffer + flen;

/* pack a file */

	outfile->ptr = outfile->ptr;
	outfile->len = 0L;

	EC->packbits = 0;
	EC->workpattern = 0L;
	EC->plen = 0L;

/* write standard header stuff (type/signature/ulen/adjust) */

	/* simple fb6 header */

	if (ulen==infile->len)
	{
		BTREE_writebits(EC,outfile,(unsigned int) 0x46fb, 16);
		BTREE_writebits(EC,outfile,(unsigned int) infile->len, 24);
	}

	/* composite fb6 header */

	else
	{
		BTREE_writebits(EC,outfile,(unsigned int) 0x47fb, 16);
		BTREE_writebits(EC,outfile,(unsigned int) ulen, 24);
		BTREE_writebits(EC,outfile,(unsigned int) infile->len, 24);
	}

	BTREE_treepack(EC,outfile,passes, multimax, 0, zerosuppress);

    return(outfile->len);
}



/****************************************************************/
/*  Encode Function                                             */
/****************************************************************/

int GCALL BTREE_encode(void *compresseddata, const void *source, int sourcesize, int *opts)
{
    int   plen;
    struct BTREEMemStruct infile;
    struct BTREEMemStruct outfile;
    struct BTreeEncodeContext EC;
    int opt=0;
    if (opts)
        opt = opts[0];

    infile.ptr = (char *)source;
    infile.len = sourcesize;
    outfile.ptr = (char *)compresseddata;
    outfile.len = sourcesize;

    plen = BTREE_compressfile(&EC,&infile, &outfile, sourcesize, opt);

    return(plen);
}

#endif
```

### `btreedecode.cpp` (`Core/Libraries/Source/Compression/EAC/btreedecode.cpp`)

```cpp
// Copyright (C) Electronic Arts Canada Inc. 1995-2002.  All rights reserved.

#ifndef __BTRREAD
#define __BTRREAD 1

#include <string.h>
#include "codex.h"
#include "btreecodex.h"

/****************************************************************/
/*  Internal Functions                                          */
/****************************************************************/

struct BTreeDecodeContext
{
    signed char    cluetbl[256];
    unsigned char  left[256];
    unsigned char  right[256];
    unsigned char  *d;
};

static void BTREE_chase(struct BTreeDecodeContext *DC, unsigned char node)
{
    if (DC->cluetbl[node])
    {
        BTREE_chase(DC,DC->left[node]);
        BTREE_chase(DC,DC->right[node]);
        return;
    }
    *DC->d++ = node;
}

static int BTREE_decompress(unsigned char *packbuf,unsigned char *unpackbuf)
{
    int  node;
    int  i;
    int  nodes;
    int  clue;
    int ulen;
    unsigned char *s;
    signed char c;
    unsigned int type;
    struct BTreeDecodeContext DC;

    s = packbuf;
    DC.d = unpackbuf;
    ulen = 0L;

    if (s)
    {
        type = ggetm(s,2);
        s += 2;

        /* (skip nothing for 0x46fb) */
        if (type==0x47fb)                       /* skip ulen */
            s += 3;

        ulen = ggetm(s,3);
        s += 3;

        for (i=0;i<256;++i)                     /* 0 means a code is a leaf */
            DC.cluetbl[i] = 0;

        clue = *s++;
        DC.cluetbl[clue] = 1;                   /* mark clue as special */

        nodes = *s++;
        for (i=0;i<nodes;++i)
        {   node = *s++;
            DC.left[node]  = *s++;
            DC.right[node] = *s++;
            DC.cluetbl[node] = (signed char)-1;
        }

        for (;;)
        {
            node = (int) *s++;
            c=DC.cluetbl[node];
            if (!c)
            {
                *DC.d++ = (unsigned char) node;
                continue;
            }
            if (c<0)
            {
                BTREE_chase(&DC,DC.left[node]);
                BTREE_chase(&DC,DC.right[node]);
                continue;
            }
            node = (int) *s++;
            if (node)
            {
                *DC.d++ = (char) node;
                continue;
            }
            break;
        }
    }
    return(ulen);
}

/****************************************************************/
/*  Information Functions                                       */
/****************************************************************/

/* check for reasonable header: */
/* 46fb header */

bool GCALL BTREE_is(const void *compresseddata)
{
    bool ok=false;

    if (ggetm(compresseddata,2)==0x46fb ||
     ggetm(compresseddata,2)==0x47fb)
        ok = true;

    return(ok);
}


/****************************************************************/
/*  Decode Functions                                            */
/****************************************************************/

int GCALL BTREE_size(const void *compresseddata)
{
    int len=0;

    if (ggetm(compresseddata,2)==0x46fb)
    {
        len = ggetm((char *)compresseddata+2,3);
    }
    else
    {
        len = ggetm((char *)compresseddata+2+3,3);
    }

    return(len);
}

int GCALL BTREE_decode(void *dest, const void *compresseddata, int *compressedsize)
{
    return(BTREE_decompress((unsigned char *)compresseddata,(unsigned char *)dest));
}

#endif
```

### `huffcodex.h` (`Core/Libraries/Source/Compression/EAC/huffcodex.h`)

```cpp
/* Copyright (C) Electronic Arts Canada Inc. 1995-2002.  All rights reserved. */


#ifdef __cplusplus
extern "C" {
#endif

#include "codex.h"

/****************************************************************/
/*  HUF Codex                                                   */
/****************************************************************/

/* Information Functions */

CODEXABOUT *GCALL HUFF_about();
bool        GCALL HUFF_is(const void *compresseddata);

/* Decode Functions */

int        GCALL HUFF_size(const void *compresseddata);
#ifdef __cplusplus
int        GCALL HUFF_decode(void *dest, const void *compresseddata, int *compressedsize=0);
#else
int        GCALL HUFF_decode(void *dest, const void *compresseddata, int *compressedsize);
#endif

/* Encode Functions */

#ifdef __cplusplus
int        GCALL HUFF_encode(void *compresseddata, const void *source, int sourcesize, int *opts=0);
#else
int        GCALL HUFF_encode(void *compresseddata, const void *source, int sourcesize, int *opts);
#endif

/****************************************************************/
/*  Internal                                                    */
/****************************************************************/

#ifndef qmin
#define qmin(a,b) ((a)<(b)?(a):(b))
#endif

#ifndef qmax
#define qmax(a,b) ((a)>(b)?(a):(b))
#endif

#ifdef __cplusplus
}
#endif
```

### `huffencode.cpp` (`Core/Libraries/Source/Compression/EAC/huffencode.cpp`)

```cpp
// Copyright (C) Electronic Arts Canada Inc. 1995-2002.  All rights reserved.

#ifndef __HUFWRITE
#define __HUFWRITE 1

#include <stddef.h>
#include <string.h>
#include "codex.h"
#include "huffcodex.h"

/****************************************************************/
/*  Internal Functions                                          */
/****************************************************************/

struct HUFFMemStruct
{
	char	*ptr;
	int	len;
};

#define HUFFBIGNUM					32000
#define HUFFTREESIZE				520
#define HUFFCODES					256
#define HUFFMAXBITS					16
#define HUFFREPTBL					252

struct HuffEncodeContext
{
    char            qleapcode[HUFFCODES];
    unsigned int	count[768];
    unsigned int	bitnum[HUFFMAXBITS+1];
    unsigned int	repbits[HUFFREPTBL];
    unsigned int	repbase[HUFFREPTBL];
    unsigned int	tree_left[HUFFTREESIZE];
    unsigned int	tree_right[HUFFTREESIZE];
    unsigned int	bitsarray[HUFFCODES];
    unsigned int	patternarray[HUFFCODES];
    unsigned int	masks[17];
    unsigned int	packbits;
    unsigned int	workpattern;
    unsigned char	*buffer;
    unsigned char	*bufptr;
    int			flen;
    unsigned int	csum;
    unsigned int	mostbits;
    unsigned int	codes;
    unsigned int	chainused;
    unsigned int	clue;
    unsigned int	dclue;
    unsigned int	clues;
    unsigned int	dclues;
    int				mindelta;
    int				maxdelta;
    unsigned int	plen;
    unsigned int	ulen;
    unsigned int	sortptr[HUFFCODES];
};

static void HUFF_deltabytes(const void *source,void *dest,int len)
{
    const unsigned char *s = (const unsigned char *) source;
    unsigned char *d = (unsigned char *) dest;
	unsigned char c;
	unsigned char c1;
	const unsigned char *send;

	c = '\0';
	send = s+len;
	while (s<send)
	{
		c1 = *s++;
		*d++ = (unsigned char)(c1-c);
		c = c1;
	}
}


static void HUFF_writebits(struct HuffEncodeContext *EC,
                    struct HUFFMemStruct *dest,
                    unsigned int	  bitpattern,
                    unsigned int	  len)
{
	if (len > 16)
	{
		HUFF_writebits(EC,dest,(unsigned int) (bitpattern>>16), len-16);
		HUFF_writebits(EC,dest,(unsigned int) bitpattern, 16);
	}
	else
	{
		EC->packbits += len;
		EC->workpattern += (bitpattern & EC->masks[len]) << (24-EC->packbits);
		while (EC->packbits > 7)
		{
			*(dest->ptr+dest->len) = (unsigned char) (EC->workpattern >> 16);
			++dest->len;

			EC->workpattern = EC->workpattern << 8;
			EC->packbits -= 8;
			++EC->plen;
		}
	}
}

static void HUFF_treechase(struct HuffEncodeContext *EC,
                    unsigned int node,
                    unsigned int bits)
{
	if (node < HUFFCODES)
		EC->bitsarray[node] = bits;
	else
	{	HUFF_treechase(EC,EC->tree_left[node], bits+1);
		HUFF_treechase(EC,EC->tree_right[node], bits+1);
	}
}

static void HUFF_maketree(struct HuffEncodeContext *EC)
{
	unsigned int	i, i1;
	unsigned int	ptr1;
	unsigned int	val1, val2;

	unsigned int			ptr2, nodes;

	unsigned int			list_count[HUFFCODES+2];
	unsigned int			list_ptr[HUFFCODES+2];

/* initialize tree */

/* registers vars usage
    i  - code
    i1 - code index (number of unjoined codes)
    i2 -
    i3 -
*/

	nodes = HUFFCODES;
	i1 = 0;
	list_count[i1++] = 0L;
	for (i=0; i<HUFFCODES; ++i)
	{	EC->bitsarray[i] = 99;
		if (EC->count[i])
		{	list_count[i1] = (unsigned int) EC->count[i];
			list_ptr[i1++] = i;
		}
	}
	EC->codes = i1-1;

/* make tree */

/* registers vars usage
    i  - code index/temp
    i1 - number of unjoined codes
    i2 -
    i3 -
*/

	if (i1>2)
	{
		while (i1>2)
		{
			/* get 2 smallest node counts */

			i = i1;
			val1 = list_count[--i];			/* initialize 2 values */
			ptr1 = i;
			val2 = list_count[--i];
			ptr2 = i;
			if (val1 < val2)
			{	val2 = val1;
				ptr2 = ptr1;
				val1 = list_count[i];
				ptr1 = i;
			}

			while (i)
			{	--i;
				while (list_count[i] > val1)
					--i;

				if (i)
				{	val1 = list_count[i];
					ptr1 = i;
					if (val1 <= val2)
					{	val1 = val2;
						ptr1 = ptr2;
						val2 = list_count[i];
						ptr2 = i;
					}
				}
			}

			EC->tree_left[nodes] = list_ptr[ptr1];
			EC->tree_right[nodes] = list_ptr[ptr2];
			list_count[ptr1] = val1 + val2;
			list_ptr[ptr1] = nodes;
			list_count[ptr2] = list_count[--i1];
			list_ptr[ptr2] = list_ptr[i1];
			++nodes;
		}

		/* traverse tree */

		HUFF_treechase(EC,nodes-1, 0);
	}
	else
	{
		/* traverse tree */

		HUFF_treechase(EC,list_ptr[EC->codes], 1);
	}
}

static int HUFF_minrep(struct HuffEncodeContext *EC,
                unsigned int remaining,
                unsigned int r)
{
	int	min, min1, use, newremaining;

	if (r)
	{	min = HUFF_minrep(EC,remaining, r-1);
		if (EC->count[EC->clue+r])
		{	use = remaining/r;
			newremaining = remaining-(use*r);
			min1 = HUFF_minrep(EC,newremaining, r-1)+EC->bitsarray[EC->clue+r]*use;
			if (min1<min)
				min = min1;
		}
	}
	else
	{	min = 0;
		if (remaining)
		{	min = 20;
			if (remaining<HUFFREPTBL)
				min = EC->bitsarray[EC->clue]+3+EC->repbits[remaining]*2;
		}
	}
	return(min);
}

static void HUFF_writenum(struct HuffEncodeContext *EC,
                   struct HUFFMemStruct	*dest,
                   unsigned int		num)
{
	unsigned int	dphuf;
	unsigned int	dbase;

	if (num<HUFFREPTBL)
	{	dphuf = EC->repbits[(unsigned int) num];
		dbase = (unsigned int) EC->repbase[(unsigned int) num];
	}
	else
	{	if (num<508L)
		{	dphuf = 6;
			dbase = 252L;
		}
		else if (num<1020L)
		{	dphuf = 7;
			dbase = 508L;
		}
		else if (num<2044L)
		{	dphuf = 8;
			dbase = 1020L;
		}
		else if (num<4092L)
		{	dphuf = 9;
			dbase = 2044L;
		}
		else if (num<8188L)
		{	dphuf = 10;
			dbase = 4092L;
		}
		else if (num<16380L)
		{	dphuf = 11;
			dbase = 8188L;
		}
		else if (num<32764L)
		{	dphuf = 12;
			dbase = 16380L;
		}
		else if (num<65532L)
		{	dphuf = 13;
			dbase = 32764L;
		}
		else if (num<131068L)
		{	dphuf = 14;
			dbase = 65532L;
		}
		else if (num<262140L)
		{	dphuf = 15;
			dbase = 131068L;
		}
		else if (num<524288L)
		{	dphuf = 16;
			dbase = 262140L;
		}
		else if (num<1048576L)
		{	dphuf = 17;
			dbase = 524288L;
		}
		else
		{	dphuf = 18;
			dbase = 1048576L;
		}
	}
	HUFF_writebits(EC,dest,(unsigned int) 0x00000001, dphuf+1);
	HUFF_writebits(EC,dest,(unsigned int) (num - dbase), dphuf+2);
}

/* write explicite byte ([clue] 0gn [0] [byte]) */

static void HUFF_writeexp(struct HuffEncodeContext *EC,
                  struct HUFFMemStruct *dest,
                  unsigned int code)
{
	HUFF_writebits(EC,dest,EC->patternarray[EC->clue], EC->bitsarray[EC->clue]);
	HUFF_writenum(EC,dest,0L);
	HUFF_writebits(EC,dest,(unsigned int) code, 9);
}

static void HUFF_writecode(struct HuffEncodeContext *EC,
                    struct HUFFMemStruct *dest,
                    unsigned int code)
{
	if (code==EC->clue)
		HUFF_writeexp(EC,dest,code);
	else
		HUFF_writebits(EC,dest,EC->patternarray[code], EC->bitsarray[code]);
}

static void HUFF_init(struct HuffEncodeContext *EC)
{
	unsigned int	i;

/* precalculate repeat field lengths */

/* registers vars usage
    i  - num
    i1 -
    i2 -
    i3 -
*/

	i = 0;
	while (i<4)
	{	EC->repbits[i] = 0;
		EC->repbase[i++] = 0L;
	}
	while (i<12)
	{	EC->repbits[i] = 1;
		EC->repbase[i++] = 4L;
	}
	while (i<28)
	{	EC->repbits[i] = 2;
		EC->repbase[i++] = 12L;
	}
	while (i<60)
	{	EC->repbits[i] = 3;
		EC->repbase[i++] = 28L;
	}
	while (i<124)
	{	EC->repbits[i] = 4;
		EC->repbase[i++] = 60L;
	}
	while (i<252)
	{	EC->repbits[i] = 5;
		EC->repbase[i++] = 124L;
	}
}


static void HUFF_analysis(struct HuffEncodeContext *EC,
                   unsigned int opt,
                   unsigned int chainsaw)
{
	unsigned char			*bptr1;
	unsigned char			*bptr2;
	unsigned int			i;
	unsigned int			i1;
	unsigned int			i2=0;
	unsigned int			i3;

	unsigned int			thres=0;
	int						di;
	unsigned int			pattern;
	unsigned int			rep1;
	unsigned int			repn;
	unsigned int			ncode;
	unsigned int			irep;
	unsigned int			remaining;

	unsigned int			count2[HUFFCODES];
	unsigned int			dcount[HUFFCODES];

/* count file (pass 1) */

/* registers vars usage
    i  - current byte
    i1 - previous byte
    i2 - rep
    i3 - checksum
*/

	for (i=0; i<768; ++i)
		EC->count[i] = 0;

	bptr1 = EC->buffer;
	i1 = 256;
	i3 = 0;
	while (bptr1<EC->bufptr)
	{	i = (unsigned int) *bptr1++;
		i3 = i3 + i;
		if (i == i1)
		{
			i2 = 0;
			bptr2 = bptr1+30000;
			if (bptr2>(EC->bufptr))
				bptr2 = EC->bufptr;

			while ((i == i1) && (bptr1 < bptr2))
			{	++i2;
				i = (unsigned int) *bptr1++;
				i3 = i3 + i;
			}

			if (i2 < 255)
				++EC->count[512+i2];
			else
				++EC->count[512];
		}
		++EC->count[i];
		++EC->count[((i+256-i1)&255)+256];
		i1 = i;
	}
	EC->csum = i3;
	if (!EC->count[512])
		++EC->count[512];

/* find clue bytes */

/* registers vars usage
    i  - code
    i1 - cluebyte
    i2 - clues
    i3 - best forced clue
*/

	EC->clues = 0;
	EC->dclues = 0;
	i3 = 0;
	for (i=0;i<HUFFCODES; ++i)
	{	i1 = i;
		i2 = 0;
		if (EC->count[i]<EC->count[i3])
			i3 = i;
		while (!EC->count[i] && (i<256))
		{	++i2;
			++i;
		}
		if (i2 >= EC->dclues)
		{	EC->dclue = i1;
			EC->dclues = i2;
			if (EC->dclues >= EC->clues)
			{	EC->dclue = EC->clue;
				EC->dclues = EC->clues;
				EC->clue = i1;
				EC->clues = i2;
			}
		}
	}

/* force a clue byte */

	if (opt & 32)
	{	if (!EC->clues)
		{	EC->clues = 1;
			EC->clue = i3;
		}
	}

/* disable & split clue bytes */

	if ((~opt) & 2)
	{	if (EC->clues>1)
			EC->clues = 1;
		if ((~opt) & 1)
			EC->clues = 0;
	}
	if ((~opt) & 4)
		EC->dclues = 0;
	else
	{	if (EC->dclues > 10)
		{	i1 = EC->clue;
			i2 = EC->clues;
			EC->clue = EC->dclue;
			EC->clues = EC->dclues;
			EC->dclue = i1;
			EC->dclues = i2;
		}

		if ((EC->clues*4) < EC->dclues)
		{	EC->clues = EC->dclues/4;
			EC->dclues = EC->dclues-EC->clues;
			EC->clue = EC->dclue+EC->dclues;
		}
	}


/* copy delta clue bytes */

	if (EC->dclues)
	{
		EC->mindelta = -((int)EC->dclues/2);
		EC->maxdelta = EC->dclues+EC->mindelta;
		thres = (int) (EC->ulen/25);

		for (i=1;i<=(unsigned int)EC->maxdelta;++i)
			if (EC->count[256+i] > thres)
				EC->count[EC->dclue+(i-1)*2] = EC->count[256+i];
		for (i=1;i<=(unsigned int)(-EC->mindelta);++i)
			if (EC->count[512-i] > thres)
				EC->count[EC->dclue+(i-1)*2+1] = EC->count[512-i];

/* adjust delta clues */

		for (i=0, i2=0;i<EC->dclues;++i)
			if (EC->count[EC->dclue+i])
				i2 = i;
		di = EC->dclues-i2-1;
		EC->dclues -= di;
		if (EC->clue == (EC->dclue+EC->dclues+di))
		{	EC->clue -= di;
			EC->clues += di;
		}
		EC->mindelta = -((int)EC->dclues/2);
		EC->maxdelta = EC->dclues+EC->mindelta;
	}

/* copy rep clue bytes */

	if (EC->clues)
	{
		for (i=0;i<EC->clues;++i)
			EC->count[EC->clue+i] = EC->count[512+i];
	}

/* make a first approximation tree */

	HUFF_maketree(EC);



/* remove implied rep clues */

/* registers vars usage
    i  - clues
    i1 - minrep
    i2 -
    i3 -
*/

	if (EC->clues>1)
	{
		for (i=1; i<EC->clues; ++i)
		{	i1 = i-1;
			if (i1>8)
				i1 = 8;
			if (EC->count[EC->clue+i])
			{	i1 = HUFF_minrep(EC,i,i1);
				if ((i1 <= EC->bitsarray[EC->clue+i]) ||
					 (EC->count[EC->clue+i]*(i1-EC->bitsarray[EC->clue+i])<(i/2)))
				{	EC->count[EC->clue+i] = 0;
				}
			}
		}
	}

/* count file (pass 2) */

/* registers vars usage
    i  - current byte
    i1 - previous byte
    i2 - rep
    i3 - rep2
*/

	for (i=0; i<HUFFCODES; ++i)
	{	count2[i] = EC->count[i];
		dcount[i] = 0;
		EC->count[i] = 0;
		EC->count[256+i] = 0;
		EC->count[512+i] = 0;
	}

	i = 1;
	i1 = 256;
	bptr1 = EC->buffer;
	while (bptr1<EC->bufptr)
	{	i = (unsigned int) *bptr1++;
		if (i == i1)
		{
			i2 = 0;
			bptr2 = bptr1+30000;
			if (bptr2>(EC->bufptr))
				bptr2 = EC->bufptr;

			while ((i == i1) && (bptr1 < bptr2))
			{	i = (unsigned int) *bptr1++;
				++i2;
			}

			repn = HUFFBIGNUM;
			irep = HUFFBIGNUM;
			ncode = i2*EC->bitsarray[i1];
			if (EC->clues)
			{
				if (count2[EC->clue])
				{	repn = 20;
					if (i2 < HUFFREPTBL)
						repn = EC->bitsarray[EC->clue]+3+EC->repbits[i2]*2;
				}
				if (EC->clues>1)
				{
					remaining = i2;
					irep = 0;
					i3=EC->clues-1;
					while (i3)
					{	if (count2[EC->clue+i3])
						{	rep1 = remaining/i3;
							irep = irep+rep1*EC->bitsarray[EC->clue+i3];
							remaining = remaining-rep1*i3;
						}
						--i3;
					}
					if (remaining)
						irep=HUFFBIGNUM;
				}
			}

			if ((ncode<=repn) && (ncode<=irep))
				EC->count[i1] += i2;
			else
			{	if (repn < irep)
					++EC->count[EC->clue];
				else
				{
					remaining = i2;
					irep = 0;
					i3=EC->clues-1;
					while (i3)
					{	if (count2[EC->clue+i3])
						{	rep1 = remaining/i3;
							irep = irep+rep1*EC->bitsarray[EC->clue+i3];
							remaining = remaining-rep1*i3;
							EC->count[EC->clue+i3] += rep1;
						}
						--i3;
					}
				}
			}
		}
		if (EC->dclues)
		{	i3 = 0;
			di = i-i1;
			if (di <= EC->maxdelta)
			{	if (di >= EC->mindelta)
				{	di = (i-i1-1)*2+EC->dclue;
					if (i<i1)
						di = (i1-i-1)*2+EC->dclue+1;
					if (count2[di]>thres)
					{	if (count2[i]<4)
							++i3;
						if (EC->bitsarray[di] < EC->bitsarray[i])
							++i3;
						if (EC->bitsarray[di] == EC->bitsarray[i])
							if (EC->count[di] > EC->count[i])
								++i3;
					}
				}
			}
			if (i3)
				++EC->count[di];
			else
				++EC->count[i];
		}
		else
			++EC->count[i];
		i1 = i;
	}

/* force a clue byte */

	if (opt & 32)
		++EC->count[EC->clue];

/* make a second approximation tree */

	HUFF_maketree(EC);

/* chainsaw IV branch clipping algorithm */

/* - maintains perfect tree

   - find intest code
   - find intest branch that's shorter than maximum bits
   - graft one branch to the shorter branch
   - shorten the other code by 1
*/

/* registers vars usage
    i  - code
    i1 - codes lengths
    i2 - int code1 ptr
    i3 - int code2 ptr
*/

	EC->chainused = 0;
	i1 = 99;
	while (i1>chainsaw)
	{	i1 = 0;
		for (i=0; i<HUFFCODES; ++i)					/* find intest code */
		{	if (EC->count[i])
			{	if (EC->bitsarray[i]>=i1)
				{	i3 = i2;
					i2 = i;
					i1 = EC->bitsarray[i];
				}
			}
		}

		if (i1>chainsaw)
		{
			i1 = 0;
			while (i1<HUFFCODES)
			{	if (EC->count[i1])
				{
					if (EC->bitsarray[i1]<chainsaw)
						break;
				}
				++i1;
			}
			for (i=i1; i<HUFFCODES; ++i)				/* find code to graft to */
			{	if (EC->count[i])
				{
					if ((EC->bitsarray[i]<chainsaw) && (EC->bitsarray[i]>EC->bitsarray[i1]))
						i1 = i;
				}
			}

			i = EC->bitsarray[i1]+1;				/* graft to short code */
			EC->bitsarray[i1] = i;
			EC->bitsarray[i2] = i;
			EC->bitsarray[i3] = EC->bitsarray[i3]-1;/* shorten other code by 1 */

			EC->chainused = chainsaw;
			i1 = 99;
		}
	}

/* if huffman inhibited make all codes 8 bits */

/* registers vars usage
    i  - code
    i1 - 8
    i2 -
    i3 -
*/

	if ((~opt) & 8)
	{
		i1 = 8;
		for (i=0; i<HUFFCODES; ++i)
			EC->bitsarray[i] = i1;
	}

/* count bitnums */

/* registers vars usage
    i  - code/bits
    i1 -
    i2 -
    i3 -
*/

	for (i=0; i<=HUFFMAXBITS; ++i)
		EC->bitnum[i] = 0;
	for (i=0; i<HUFFCODES; ++i)
	{	if (EC->bitsarray[i]<=HUFFMAXBITS)
			++EC->bitnum[EC->bitsarray[i]];
	}

/* sort codes */

/* registers vars usage
    i  - next sorted ptr
    i1 - code
    i2 - bits
    i3 - EC->mostbits
*/

	i=0;
	i3=0;
	for (i2=1; i2<=HUFFMAXBITS; ++i2)
	{	if (EC->bitnum[i2])
		{	for (i1=0; i1<HUFFCODES; ++i1)
				if (EC->bitsarray[i1] == i2)
					EC->sortptr[i++] = i1;
			i3 = i2;
		}
	}
	EC->mostbits = i3;

/* assign bit patterns */

/* registers vars usage
    i  - code index
    i1 - code
    i2 - bits
    i3 -
*/

	pattern = 0L;
	i2 = 0;
	for (i=0; i<EC->codes; ++i)
	{	i1 = EC->sortptr[i];
		while (i2 < EC->bitsarray[i1])
		{	++i2;
			pattern = pattern << 1;
		}
		EC->patternarray[i1] = pattern;
		++pattern;
	}
}


static void HUFF_pack(struct HuffEncodeContext *EC,
               struct HUFFMemStruct *dest,
               unsigned int	opt)
{
	unsigned char			*bptr1;
	unsigned char			*bptr2;
	unsigned int			i;
	unsigned int			i1;
	unsigned int			i2;
	unsigned int			i3;
	int						uptype;
	unsigned int			hlen, ibits, rladjust;
	int						di, firstcode, firstbits;
	unsigned int			rep1, repn, ncode, irep, remaining,curpc;

/* write header */

	curpc = 0L;

	uptype = 38;
	rladjust = 1;
	if (uptype==38)
	{
		if (uptype==34)
		{
			HUFF_writenum(EC,dest,(unsigned int) EC->ulen);

			ibits = 0;
			if ((opt & 16) && (!EC->chainused))
				ibits = 1;

			i = 0;									/* write options field */
			if (EC->clues)
				i = 1;
			if (ibits)
				i += 2;
			if (EC->dclues)
				i += 4;
			HUFF_writenum(EC,dest,(unsigned int) i);

			if (EC->clues)
			{	HUFF_writenum(EC,dest,(unsigned int) EC->clue);
				HUFF_writenum(EC,dest,(unsigned int) EC->clues);
			}
			if (EC->dclues)
			{	HUFF_writenum(EC,dest,(unsigned int) EC->dclue);
				HUFF_writenum(EC,dest,(unsigned int) EC->dclues);
			}

			if (!ibits)
				HUFF_writenum(EC,dest,(unsigned int) EC->mostbits);
		}
		else
		{
			HUFF_writebits(EC,dest,(unsigned int) EC->clue, 8);	/* clue */
			rladjust = 0;
		}

		for (i=1; i <= EC->mostbits; ++i)
			HUFF_writenum(EC,dest,(unsigned int) EC->bitnum[i]);

		for (i=0; i<HUFFCODES; ++i)
			EC->qleapcode[i] = 0;

		i = 0;
		i2 = 255;
		firstbits = 0;
		firstcode = -1;
		while (i<EC->codes)
		{
			i1 = EC->sortptr[i];
#if 0
			if (EC->bitsarray[i1]!=firstbits)
			{
				i2 = firstcode;
				firstcode = i1;
				firstbits = EC->bitsarray[i2];
			}
#endif

/* calculate leapfrog delta */

			di = -1;
			do
			{
				i2 = (i2+1)&255;
				if (!EC->qleapcode[i2])
					++di;
			} while (i1!=i2);
			EC->qleapcode[i2] = 1;
			HUFF_writenum(EC,dest,(unsigned int) di);
			++i;
		}
	}
	hlen = EC->plen+1;

	if (!EC->clues)
		EC->clue = HUFFBIGNUM;

/* write packed file */

/* registers vars usage
    i  - current byte
    i1 - previous byte
    i2 - rep
    i3 - rep2
*/

	i = 1;
	i1 = 256;
	bptr1 = EC->buffer;
	while (bptr1<EC->bufptr)
	{	i = (unsigned int) *bptr1++;

		if (i == i1)
		{
			i2 = 0;
			bptr2 = bptr1+30000;
			if (bptr2>(EC->bufptr))
				bptr2 = EC->bufptr;

			while ((i == i1) && (bptr1 < bptr2))
			{	i = (unsigned int) *bptr1++;
				++i2;
			}

			repn = HUFFBIGNUM;
			irep = HUFFBIGNUM;
			ncode = i2*EC->bitsarray[i1];
			if (EC->clues)
			{
				if (EC->count[EC->clue])
				{	repn = 20;
					if (i2 < HUFFREPTBL)
						repn = EC->bitsarray[EC->clue]+3+EC->repbits[i2]*2;
				}
				if (EC->clues>1)
				{
					remaining = i2;
					irep = 0;
					i3=EC->clues-1;
					while (i3)
					{	if (EC->count[EC->clue+i3])
						{	rep1 = remaining/i3;
							irep = irep+rep1*EC->bitsarray[EC->clue+i3];
							remaining = remaining-rep1*i3;
						}
						--i3;
					}
					if (remaining)
						irep = HUFFBIGNUM;
				}
			}

			if ((ncode<=repn) && (ncode<=irep))
			{
				while (i2--)
					HUFF_writecode(EC,dest,i1);
			}
			else
			{	if (repn < irep)
				{
					HUFF_writebits(EC,dest,EC->patternarray[EC->clue], EC->bitsarray[EC->clue]);
					HUFF_writenum(EC,dest,(unsigned int) (i2-rladjust));
				}
				else
				{
					remaining = i2;
					irep = 0;
					i3=EC->clues-1;
					while (i3)
					{	if (EC->count[EC->clue+i3])
						{	rep1 = remaining/i3;
							irep = irep+rep1*EC->bitsarray[EC->clue+i3];
							remaining = remaining-rep1*i3;
							while (rep1--)
								HUFF_writecode(EC,dest,EC->clue+i3);
						}
						--i3;
					}
				}
			}
		}
		i3 = 0;
		if (EC->dclues)
		{
			di = i-i1;
			if ((di <= EC->maxdelta) && (di >= EC->mindelta))
			{	di = (i-i1-1)*2+EC->dclue;
				if (i<i1)
					di = (i1-i-1)*2+EC->dclue+1;
				if (EC->bitsarray[di] < EC->bitsarray[i])
				{	HUFF_writebits(EC,dest,EC->patternarray[di], EC->bitsarray[di]);
					++i3;
				}
			}
		}
		i1 = i;
		if (!i3)
			HUFF_writecode(EC,dest,i);

		if ((bptr1 - EC->buffer) >= (ptrdiff_t)(EC->plen+curpc))
			curpc = (unsigned int)((bptr1 - EC->buffer) - EC->plen);
	}

	/* write EOF ([clue] 0gn [10]) */

	HUFF_writebits(EC,dest,EC->patternarray[EC->clue], EC->bitsarray[EC->clue]);
	HUFF_writenum(EC,dest,0L);
	HUFF_writebits(EC,dest,(unsigned int) 2, 2);

	/* flush bits */

	HUFF_writebits(EC,dest,(unsigned int) 0,7);

	curpc += 2;
}

static int HUFF_packfile(struct HuffEncodeContext *EC,
                   struct HUFFMemStruct	*infile,
                   struct HUFFMemStruct	*outfile,
                   int	ulen,
                   int	deltaed)
{
	unsigned int i;
	unsigned int uptype=0;
	unsigned int chainsaw;
	unsigned int opt;

/* set defaults */

	EC->packbits = 0;
	EC->workpattern = 0L;
	chainsaw = 15;
	EC->masks[0] = 0;
	for (i=1;i<17;++i)
		EC->masks[i] = (EC->masks[i-1] << 1) + 1;

/* initialize huffman vars */

	HUFF_init(EC);

/* read in a source file */

	EC->buffer = (unsigned char *) (infile->ptr);
	EC->flen = infile->len;

	EC->ulen = EC->flen;
	EC->bufptr = EC->buffer + EC->flen;

/* pack a file */

	outfile->ptr = outfile->ptr;
	outfile->len = 0L;

	EC->packbits = 0;
	EC->workpattern = 0L;
	EC->plen = 0L;

	opt = 57 | 49;

	HUFF_analysis(EC,opt, chainsaw);

/* write standard header stuff (type/signature/ulen/adjust) */

    if (ulen>0xffffff)  // 32 bit header required
    {
    	/* simple fb6 header */

    	if (ulen==infile->len)
    	{
    		if (deltaed==0) 		uptype = 0xb0fb;
    		else if (deltaed==1)	uptype = 0xb2fb;
    		else if (deltaed==2)	uptype = 0xb4fb;
    		HUFF_writebits(EC,outfile,(unsigned int) uptype, 16);
    		HUFF_writebits(EC,outfile,(unsigned int) infile->len, 32);
    	}

    	/* composite fb4 header */

    	else
    	{
    		if (deltaed==0) 		uptype = 0xb1fb;
    		else if (deltaed==1)	uptype = 0xb3fb;
    		else if (deltaed==2)	uptype = 0xb5fb;
    		HUFF_writebits(EC,outfile,(unsigned int) uptype, 16);
    		HUFF_writebits(EC,outfile,(unsigned int) ulen, 32);
    		HUFF_writebits(EC,outfile,(unsigned int) infile->len, 32);
    	}
    }
    else
    {
    	/* simple fb6 header */


    	if (ulen==infile->len)
    	{
    		if (deltaed==0) 		uptype = 0x30fb;
    		else if (deltaed==1)	uptype = 0x32fb;
    		else if (deltaed==2)	uptype = 0x34fb;
    		HUFF_writebits(EC,outfile,(unsigned int) uptype, 16);
    		HUFF_writebits(EC,outfile,(unsigned int) infile->len, 24);
    	}

    	/* composite fb4 header */

    	else
    	{
    		if (deltaed==0) 		uptype = 0x31fb;
    		else if (deltaed==1)	uptype = 0x33fb;
    		else if (deltaed==2)	uptype = 0x35fb;
    		HUFF_writebits(EC,outfile,(unsigned int) uptype, 16);
    		HUFF_writebits(EC,outfile,(unsigned int) ulen, 24);
    		HUFF_writebits(EC,outfile,(unsigned int) infile->len, 24);
    	}
    }

	HUFF_pack(EC,outfile, opt);

    return(outfile->len);
}


/****************************************************************/
/*  Encode Function                                             */
/****************************************************************/

int GCALL HUFF_encode(void *compresseddata, const void *source, int sourcesize, int *opts)
{
    int   plen=0;
    struct HUFFMemStruct infile;
    struct HUFFMemStruct outfile;
    struct HuffEncodeContext *EC=nullptr;
    void *deltabuf=nullptr;
    int opt=0;
    if (opts)
        opt = opts[0];

    EC = (struct HuffEncodeContext *)galloc(sizeof(struct HuffEncodeContext));
    if (EC)
    {
        switch (opt)
        {
            default:
            case 0:
                infile.ptr = (char *)source;
                break;

            case 1:
                deltabuf = galloc(sourcesize);
    			HUFF_deltabytes(source,deltabuf,sourcesize);
                infile.ptr = (char *) deltabuf;
                break;

            case 2:
                deltabuf = galloc(sourcesize);
    			HUFF_deltabytes(source,deltabuf,sourcesize);
    			HUFF_deltabytes(deltabuf,deltabuf,sourcesize);
                infile.ptr = (char *) deltabuf;
                break;
        }

        infile.len = sourcesize;
        outfile.ptr = (char *)compresseddata;
        outfile.len = sourcesize;

        plen = HUFF_packfile(EC,&infile, &outfile, sourcesize, opt);

        if (deltabuf) gfree(deltabuf);
        gfree(EC);
    }
    return(plen);
}

#endif
```

### `huffdecode.cpp` (`Core/Libraries/Source/Compression/EAC/huffdecode.cpp`)

```cpp
// Copyright (C) Electronic Arts Canada Inc. 1995-2002.  All rights reserved.

#ifndef __HUFREAD
#define __HUFREAD 1

#include <string.h>
#include "codex.h"
#include "huffcodex.h"

#if defined(_MSC_VER)
#pragma warning(push,1)
#endif

/****************************************************************/
/*  Internal Functions                                          */
/****************************************************************/

struct HuffDecodeContext
{
    unsigned char  *s;
    int             bitsleft;
    unsigned int   bits;
};

/****************************************************************/
/*  Huffman Unpacker                                            */
/****************************************************************/

static int ZERO=0;

#define GET16BITS() \
    bitsunshifted =  qs[0] | (bitsunshifted << 8);\
    bitsunshifted =  qs[1] | (bitsunshifted << 8);\
    qs += 2;


#define SQgetbits(v,n)\
    if (n) \
    { \
        v = bits >> (32-(n));\
        bits <<= (n);\
        bitsleft -= (n);\
    } \
\
    if (bitsleft<0)\
    {\
        GET16BITS() \
\
        bits = bitsunshifted<<(-bitsleft);\
        bitsleft += 16;\
    }


#define SQgetnum(v) \
    if ((int)bits<0)\
    {\
        SQgetbits(v,3);\
        v -= 4;\
    }\
    else\
    {\
        int             n;\
        unsigned int   v1;\
\
        if (bits>>16)\
        {\
            n=2;\
            do\
            {\
                bits <<= 1; \
                ++n;\
            }\
            while ((int)bits>=0);\
            bits <<= 1;\
            bitsleft -= (n-1);\
            SQgetbits(v,ZERO);\
        }\
        else\
        {\
            n=2;\
            do\
            {\
                ++n;\
                SQgetbits(v,1);\
            }\
            while (!v);\
        }\
        if (n>16)\
        {\
            SQgetbits(v,n-16);\
            SQgetbits(v1,16);\
            v = (v1|(v<<16))+(1<<n)-4;\
        }\
        else\
        {\
            SQgetbits(v,n);\
            v = (v+(1<<n)-4);\
        }\
    }

/* note:  requires 'amt' be a multiple of 16 */
#define SQmemset(ptr, val, amt) \
{\
    int i = amt/16;\
    int intval = (val<<24)|(val<<16)|(val<<8)|val;\
    int* lptr = (int*)ptr;\
\
    while (i)\
    {\
        lptr[0] = intval;\
        lptr[1] = intval;\
        lptr[2] = intval;\
        lptr[3] = intval;\
        lptr += 4;\
        --i;\
    }\
}\

static int HUFF_decompress(unsigned char *packbuf, unsigned char *unpackbuf)
{
    unsigned int    type;
    unsigned char   clue;
    int            ulen;
    unsigned int    cmp;
    int             bitnum=0;
    int             cluelen=0;
    unsigned char   *qs;
    unsigned char   *qd;
    unsigned int   bits;
    unsigned int   bitsunshifted=0;
    int             numbits;
    int             bitsleft;
    unsigned int   v;

    qs = packbuf;
    qd = unpackbuf;
    ulen = 0L;

    if (qs)
    {
        {
            int             mostbits;
            int             i;
            int             bitnumtbl[16];
            unsigned int    deltatbl[16];
            unsigned int    cmptbl[16];
            unsigned char   codetbl[256];
            unsigned char   quickcodetbl[256];
            unsigned char   quicklentbl[256];

            bitsleft = -16;                                 /* init bit stream */
            bits = 0;
            SQgetbits(v,ZERO);

            SQgetbits(type,16);

            if (type&0x8000) /* 4 byte size field */
            {
                /* (skip nothing for 0x30fb) */
                if (type&0x100)                                 /* skip ulen */
                {
                    SQgetbits(v,16);
                    SQgetbits(v,16);
                }
                type &= ~0x100;

                SQgetbits(v,16);                                 /* unpack len */
                SQgetbits(ulen,16);
                ulen |= (v<<16);
            }
            else
            {
                /* (skip nothing for 0x30fb) */
                if (type&0x100)                                 /* skip ulen */
                {
                    SQgetbits(v,8);
                    SQgetbits(v,16);
                }
                type &= ~0x100;

                SQgetbits(v,8);                                 /* unpack len */
                SQgetbits(ulen,16);
                ulen |= (v<<16);
            }

            {
                {
                    int numchars;

                    {
                        unsigned int basecmp;

                        {
                            unsigned int t;
                            SQgetbits(t,8);                          /* clue byte */
                            clue = (unsigned char)t;
                        }

                        numchars = 0;
                        numbits = 1;
                        basecmp = (unsigned int) 0;

                        /* decode bitnums */

                        do
                        {
                            basecmp <<= 1;
                            deltatbl[numbits] = basecmp-numchars;

                            SQgetnum(bitnum);               /* # of codes of n bits */
                            bitnumtbl[numbits] = bitnum;

                            numchars += bitnum;
                            basecmp += bitnum;

                            cmp = 0;
                            if (bitnum)                             /* left justify cmp */
                                cmp = (basecmp << (16-numbits) & 0xffff);

                            cmptbl[numbits++] = cmp;

                        }
                        while (!bitnum || cmp);                     /* n+1 bits in cmp? */
                    }
                    cmptbl[numbits-1] = 0xffffffff;               /* force match on most bits */

                    mostbits = numbits-1;

                    /* decode leapfrog code table */

                    {
                        signed char     leap[256];
                        unsigned char   nextchar;

                        SQmemset(leap,0,256);
                        nextchar = (unsigned char) -1;

                        for (i=0;i<numchars;++i)
                        {
                            int leapdelta=0;

                            SQgetnum(leapdelta);
                            ++leapdelta;

                            do
                            {
                                ++nextchar;
                                if (!leap[nextchar])
                                    --leapdelta;
                            } while (leapdelta);

                            leap[nextchar] = 1;
                            codetbl[i] = nextchar;
                        }
                    }
                }

/****************************************************************/
/*  Make fast 8 tables                                          */
/****************************************************************/

                SQmemset(quicklentbl,64,256);

                {
                    int bits;
                    int bitnum;
                    int numbitentries;
                    int nextcode;
                    int nextlen;
                    int i;
                    unsigned char *codeptr;
                    unsigned char *quickcodeptr;
                    unsigned char *quicklenptr;

                    codeptr = codetbl;
                    quickcodeptr = quickcodetbl;
                    quicklenptr = quicklentbl;

                    for (bits=1; bits<=mostbits; ++bits)
                    {
                        bitnum = bitnumtbl[bits];
                        if (bits>=9)
                            break;
                        numbitentries = 1<<(8-bits);

                        while (bitnum--)
                        {
                            nextcode = *codeptr++;
                            nextlen = bits;
                            if (nextcode==clue)
                            {
                                cluelen = bits;
                                nextlen = 96;                   /* will force out of main loop */
                            }
                            for (i=0; i<numbitentries; ++i)
                            {
                                *quickcodeptr++ = (unsigned char) nextcode;
                                *quicklenptr++ = (unsigned char) nextlen;
                            }
                        }
                    }
                }
            }

/****************************************************************/
/*  Main decoder                                                */
/****************************************************************/

            for (;;)
            {
                unsigned char   *quickcodeptr = quickcodetbl;
                unsigned char   *quicklenptr  = quicklentbl;

                goto nextloop;

/* quick 8 fetch */

                do
                {

                    *qd++ = quickcodeptr[bits>>24];
                    GET16BITS();
                    bits = bitsunshifted<<(16-bitsleft);

/* quick 8 decode */

nextloop:
                    numbits = quicklenptr[bits>>24];
                    bitsleft -= numbits;

                    if (bitsleft>=0)
                    {
                        do
                        {
                            *qd++ = quickcodeptr[bits>>24];
                            bits <<= numbits;

                            numbits = quicklenptr[bits>>24];
                            bitsleft -= numbits;
                            if (bitsleft<0) break;
                            *qd++ = quickcodeptr[bits>>24];
                            bits <<= numbits;

                            numbits = quicklenptr[bits>>24];
                            bitsleft -= numbits;
                            if (bitsleft<0) break;
                            *qd++ = quickcodeptr[bits>>24];
                            bits <<= numbits;

                            numbits = quicklenptr[bits>>24];
                            bitsleft -= numbits;
                            if (bitsleft<0) break;
                            *qd++ = quickcodeptr[bits>>24];
                            bits <<= numbits;

                            numbits = quicklenptr[bits>>24];
                            bitsleft -= numbits;

                        } while (bitsleft>=0);
                    }
                    bitsleft += 16;

                } while (bitsleft>=0);  /* would fetching 16 bits do it? */

                bitsleft = bitsleft-16+numbits;   /* back to normal */

/****************************************************************/
/*  16 bit decoder                                              */
/****************************************************************/

                {
                    unsigned char   code;


                    if (numbits!=96)
                    {
                        cmp = (unsigned int) (bits>>16);  /* 16 bit left justified compare */

                        numbits = 8;
                        do
                        {
                            ++numbits;
                        }
                        while (cmp>=cmptbl[numbits]);
                    }
                    else
                        numbits = cluelen;


                    cmp = bits >> (32-(numbits));
                    bits <<= (numbits);
                    bitsleft -= (numbits);

                    code = codetbl[cmp-deltatbl[numbits]];  /* the code */

                    if (code!=clue && bitsleft>=0)
                    {
                        *qd++ = code;
                        goto nextloop;
                    }

                    if (bitsleft<0)
                    {
                        GET16BITS();
                        bits = bitsunshifted<<-bitsleft;
                        bitsleft += 16;
                    }

                    if (code!=clue)
                    {
                        *qd++ = code;
                        goto nextloop;
                    }

                    /* handle clue */

                    {
                        int    runlen=0;
                        unsigned char *d=qd;
                        unsigned char *dest;

                        SQgetnum(runlen);
                        if (runlen)                             /* runlength sequence */
                        {
                            dest = d+runlen;
                            code = *(d-1);
                            do
                            {
                                *d++ = code;
                            } while (d<dest);

                            qd = d;
                            goto nextloop;
                        }
                    }

                    SQgetbits(v,1);                         /* End Of File */
                    if (v)
                        break;

                    {
                        unsigned int t;
                        SQgetbits(t,8);                    /* explicite byte */
                        code = (unsigned char)t;
                    }
                    *qd++ = code;
                    goto nextloop;
                }

            }


/****************************************************************/
/*  Undelta                                                     */
/****************************************************************/

            {
                int i;
                int nextchar;

                if (type==0x32fb || type==0xb2fb)                           /* deltaed? */
                {
                    i = 0;
                    qd = unpackbuf;
                    while (qd<unpackbuf+ulen)
                    {
                        i += (int) *qd;
                        *qd++ = (unsigned char) i;
                    }
                }
                else if (type==0x34fb || type==0xb4fb)                      /* accelerated? */
                {

                    i = 0;
                    nextchar = 0;
                    qd = unpackbuf;
                    while (qd<unpackbuf+ulen)
                    {
                        i += (int) *qd;
                        nextchar += i;
                        *qd++ = (unsigned char) nextchar;
                    }
                }
            }
        }
    }
    return(ulen);
}

#if defined(_MSC_VER)
#pragma warning(pop)
#endif

/****************************************************************/
/*  Information Functions                                       */
/****************************************************************/

/* check for reasonable header: */
/* 30fb..35fb header */

bool GCALL HUFF_is(const void *compresseddata)
{
    bool ok=false;
    int packtype=ggetm(compresseddata,2);

    if (packtype==0x30fb ||
     packtype==0x31fb ||
     packtype==0x32fb ||
     packtype==0x33fb ||
     packtype==0x34fb ||
     packtype==0x35fb ||
     packtype==0xb0fb ||
     packtype==0xb1fb ||
     packtype==0xb2fb ||
     packtype==0xb3fb ||
     packtype==0xb4fb ||
     packtype==0xb5fb)
        ok = true;

    return(ok);
}


/****************************************************************/
/*  Decode Functions                                            */
/****************************************************************/

int GCALL HUFF_size(const void *compresseddata)
{
    int len=0;
    int packtype=ggetm(compresseddata,2);
    int ssize=(packtype&0x8000)?4:3;

    if (packtype&0x100)     /* 31fb 33fb 35fb */
    {
        len = ggetm((char *)compresseddata+2+ssize,ssize);
    }
    else                    /* 30fb 32fb 34fb */
    {
        len = ggetm((char *)compresseddata+2,ssize);
    }

    return(len);
}

int GCALL HUFF_decode(void *dest, const void *compresseddata, int *compressedsize)
{
    return(HUFF_decompress((unsigned char *)compresseddata, (unsigned char *)dest));
}

#endif
```
