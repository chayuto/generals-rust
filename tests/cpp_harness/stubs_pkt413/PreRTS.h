#pragma once

#include <climits>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <gmath.h>

using Int = int32_t;
using Int64 = int64_t;
using UnsignedInt = uint32_t;
using UnsignedInt64 = uint64_t;
using Short = int16_t;
using UnsignedShort = uint16_t;
using Byte = int8_t;
using UnsignedByte = uint8_t;
using Char = char;
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
#ifndef USE_DETERMINISTIC_MATH
#define USE_DETERMINISTIC_MATH 1
#endif
#ifndef RETAIL_COMPATIBLE_CRC
#define RETAIL_COMPATIBLE_CRC 0
#endif
#ifndef RETAIL_COMPATIBLE_PATHFINDING
#define RETAIL_COMPATIBLE_PATHFINDING 1
#endif
#ifndef RETAIL_COMPATIBLE_PATHFINDING_ALLOCATION
#define RETAIL_COMPATIBLE_PATHFINDING_ALLOCATION 1
#endif

#ifndef __forceinline
#define __forceinline inline
#endif
#ifndef __inline
#define __inline inline
#endif
#ifndef CPP_11
#define CPP_11(x) x
#endif
#ifndef ARRAY_SIZE
#define ARRAY_SIZE(x) (sizeof(x) / sizeof((x)[0]))
#endif
#ifndef MSGNEW
#define MSGNEW(tag) new
#endif

inline long fast_float2long_round(float f) {
    return ::lroundf(f);
}

#define REAL_TO_INT_FLOOR(x) (fast_float2long_round(gm_floorf(x)))
#define REAL_TO_INT_CEIL(x) (fast_float2long_round(gm_ceilf(x)))

enum ObjectID : Int { INVALID_ID = 0 };
enum KindOfType : Int {
    KINDOF_INVALID = -1,
    KINDOF_CAN_SEE_THROUGH_STRUCTURE = 1
};
enum PathfindLayerEnum CPP_11(: Int) {
    LAYER_INVALID = 0,
    LAYER_GROUND = 1,
    LAYER_WALL = 15,
    LAYER_LAST = 15
};

struct ICoord2D {
    Int x;
    Int y;
    void zero() { x = y = 0; }
};

#include "Common/Debug.h"
