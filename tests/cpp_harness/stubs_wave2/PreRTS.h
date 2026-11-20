#pragma once

#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <ctime>
#include <climits>
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

#ifndef TRUE
#define TRUE 1
#endif
#ifndef FALSE
#define FALSE 0
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
#ifndef FALLTHROUGH
#define FALLTHROUGH [[fallthrough]]
#endif
#ifndef ARRAY_SIZE
#define ARRAY_SIZE(x) (sizeof(x) / sizeof((x)[0]))
#endif
#ifndef PI
#define PI 3.14159265359f
#define TWO_PI 6.28318530718f
#endif

#ifndef RETAIL_COMPATIBLE_CRC
#define RETAIL_COMPATIBLE_CRC 0
#endif
#ifndef RETAIL_COMPATIBLE_XFER_SAVE
#define RETAIL_COMPATIBLE_XFER_SAVE 0
#endif

#include <gmath.h>

template <typename NUM>
inline NUM sqr(NUM x) {
    return x * x;
}

template <typename NUM>
inline NUM clamp(NUM lo, NUM val, NUM hi) {
    if (val < lo) return lo;
    if (val > hi) return hi;
    return val;
}

enum ObjectID : Int {
    INVALID_ID = 0
};

enum VeterancyLevel : Int {
    LEVEL_REGULAR = 0,
    LEVEL_VETERAN = 1,
    LEVEL_ELITE = 2,
    LEVEL_HEROIC = 3,
    LEVEL_COUNT = 4,
    LEVEL_FIRST = LEVEL_REGULAR,
    LEVEL_LAST = LEVEL_HEROIC,
    LEVEL_INVALID = 4
};

struct Coord2D {
    Real x, y;
};
struct Coord3D {
    Real x, y, z;
};
struct Region2D {
    Coord2D lo, hi;
};

struct WWMath {
    static inline float Sqrt(float x) { return gm_sqrtf(x); }
    static inline double Sqrt(double x) { return gm_sqrt(x); }
    static inline float Atan2(float y, float x) { return gm_atan2f(y, x); }
    static inline float Cos(float x) { return gm_cosf(x); }
    static inline float Sin(float x) { return gm_sinf(x); }
};

#include "Common/Debug.h"
#include "Common/Errors.h"
#include "Common/AsciiString.h"
#include "Common/UnicodeString.h"
#include "GameLogic/LogicRandomValue.h"
#include "GameClient/ClientRandomValue.h"
#include "Common/AudioRandomValue.h"
