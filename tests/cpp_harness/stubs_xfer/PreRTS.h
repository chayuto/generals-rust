#pragma once

#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>
#include <list>

using Int = int32_t;
using Int64 = int64_t;
using UnsignedInt = uint32_t;
using UnsignedInt64 = uint64_t;
using Short = int16_t;
using UnsignedShort = uint16_t;
using Byte = int8_t;
using UnsignedByte = uint8_t;
using Char = char;
using WideChar = uint16_t; // Canonical 16-bit UTF-16 across all platforms
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

#ifndef CPP_11
#define CPP_11(x) x
#endif
#ifndef FALLTHROUGH
#define FALLTHROUGH [[fallthrough]]
#endif

#define RETAIL_COMPATIBLE_CRC 0
#define RETAIL_COMPATIBLE_XFER_SAVE 0
#define DEEP_CRC_TO_MEMORY 0

#define BitIsSet(x, i) (((x) & (i)) != 0)
#define BitSet(x, i) ((x) |= (i))
#define BitClear(x, i) ((x) &= ~(i))

enum ObjectID : Int {
    INVALID_ID = 0
};
enum DrawableID : Int {
    INVALID_DRAWABLE_ID = 0
};
enum KindOfType : Int {
    KINDOF_INVALID = -1
};
enum ScienceType : Int {
    SCIENCE_INVALID = -1
};

struct Coord2D {
    Real x, y;
};
struct ICoord2D {
    Int x, y;
};
struct Region2D {
    Coord2D lo, hi;
};
struct IRegion2D {
    ICoord2D lo, hi;
};
struct Coord3D {
    Real x, y, z;
};
struct ICoord3D {
    Int x, y, z;
};
struct Region3D {
    Coord3D lo, hi;
};
struct IRegion3D {
    ICoord3D lo, hi;
};
struct RealRange {
    Real lo, hi;
};
struct RGBColor {
    Real red, green, blue;
};
struct RGBAColorReal {
    Real red, green, blue, alpha;
};
struct RGBAColorInt {
    UnsignedInt red, green, blue, alpha;
};
struct Vector4 {
    Real X, Y, Z, W;
};
class Matrix3D {
public:
    Vector4 Row[3];
    Vector4& operator[](int i) { return Row[i]; }
    const Vector4& operator[](int i) const { return Row[i]; }
};

#include "Common/Debug.h"
#include "Common/AsciiString.h"
#include "Common/UnicodeString.h"
