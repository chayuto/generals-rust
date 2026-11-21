#pragma once

#include <algorithm>
#include <bit>
#include <bitset>
#include <cctype>
#include <climits>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <ctime>
#include <list>
#include <map>
#include <set>
#include <string>
#include <unordered_map>
#include <vector>

using std::max;
using std::min;

using Int = int32_t;
using Int64 = int64_t;
using UnsignedInt = uint32_t;
using UnsignedInt64 = uint64_t;
using Short = int16_t;
using UnsignedShort = uint16_t;
using Byte = int8_t;
using UnsignedByte = uint8_t;
using Char = char;
using WideChar = uint16_t;
using Bool = uint8_t;
using Real = float;
using XferVersion = uint8_t;

#ifndef TRUE
#define TRUE 1
#endif
#ifndef FALSE
#define FALSE 0
#endif

#ifndef RTS_ZEROHOUR
#define RTS_ZEROHOUR 1
#endif
#ifndef RTS_GENERALS
#define RTS_GENERALS 0
#endif

#ifndef _UNIX
#define _UNIX 1
#endif

#ifndef _MAX_PATH
#define _MAX_PATH 260
#endif

#ifndef CPP_11
#define CPP_11(x) x
#endif
#ifndef FALLTHROUGH
#define FALLTHROUGH [[fallthrough]]
#endif
#ifndef MAYBE_UNUSED
#define MAYBE_UNUSED [[maybe_unused]]
#endif
#ifndef ARRAY_SIZE
#define ARRAY_SIZE(x) (sizeof(x) / sizeof((x)[0]))
#endif

#define BitIsSet(x, i) (((x) & (i)) != 0)
#define BitSet(x, i) ((x) |= (i))
#define BitClear(x, i) ((x) &= ~(i))

#ifndef ENABLE_FILESYSTEM_EXISTENCE_CACHE
#define ENABLE_FILESYSTEM_EXISTENCE_CACHE 0
#endif
#ifndef ENABLE_FILESYSTEM_LOGGING
#define ENABLE_FILESYSTEM_LOGGING 0
#endif

inline int stricmp(const char* a, const char* b) { return ::strcasecmp(a, b); }
inline int strnicmp(const char* a, const char* b, size_t n) { return ::strncasecmp(a, b, n); }
#ifndef _stricmp
#define _stricmp stricmp
#endif

#include "Lib/PathUtil.h"
#include "Common/Debug.h"
#include "Common/GameMemory.h"
#include "Common/SubsystemInterface.h"
#include "Common/AsciiString.h"
#include "Common/UnicodeString.h"
