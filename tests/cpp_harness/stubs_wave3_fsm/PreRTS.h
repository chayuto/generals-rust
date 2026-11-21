#pragma once

#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <algorithm>
#include <list>
#include <map>
#include <string>
#include <vector>

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
using Color = Int;

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

#ifndef CPP_11
#define CPP_11(x) x
#endif
#ifndef FALLTHROUGH
#define FALLTHROUGH [[fallthrough]]
#endif
#ifndef ARRAY_SIZE
#define ARRAY_SIZE(x) (sizeof(x) / sizeof((x)[0]))
#endif

#define RETAIL_COMPATIBLE_CRC 0
#define RETAIL_COMPATIBLE_XFER_SAVE 0
#define DEEP_CRC_TO_MEMORY 0

#define BitIsSet(x, i) (((x) & (i)) != 0)
#define BitSet(x, i) ((x) |= (i))
#define BitClear(x, i) ((x) &= ~(i))

template <typename T>
inline T min(T a, T b) { return a < b ? a : b; }

template <typename T>
inline T max(T a, T b) { return a > b ? a : b; }

#include "Common/Debug.h"
#include "Common/Errors.h"
#include "Common/AsciiString.h"
#include "Common/UnicodeString.h"
#include "Common/GameType.h"
