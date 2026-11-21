#pragma once

#include <algorithm>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
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

enum ObjectID : Int { INVALID_ID = 0 };
enum DrawableID : Int { INVALID_DRAWABLE_ID = 0 };

struct Coord3D { Real x, y, z; };
struct ICoord2D { Int x, y; };
struct IRegion2D { ICoord2D lo, hi; };

#include "Common/Debug.h"
#include "Common/Errors.h"
#include "Common/GameMemory.h"
#include "Common/SubsystemInterface.h"
#include "Common/AsciiString.h"
