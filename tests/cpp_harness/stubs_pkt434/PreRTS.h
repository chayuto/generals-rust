#pragma once

#include <algorithm>
#include <bitset>
#include <cctype>
#include <climits>
#include <cstdarg>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <ctime>
#include <list>
#include <map>
#include <string>
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
#ifndef FUNCTION_DELETE
#define FUNCTION_DELETE = delete
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
#define RETAIL_COMPATIBLE_XFER_SAVE 1
#endif
#ifndef DEEP_CRC_TO_MEMORY
#define DEEP_CRC_TO_MEMORY 0
#endif

#define BitIsSet(x, i) (((x) & (i)) != 0)
#define BitSet(x, i) ((x) |= (i))
#define BitClear(x, i) ((x) &= ~(i))

inline int stricmp(const char* a, const char* b) { return ::strcasecmp(a, b); }
inline int strnicmp(const char* a, const char* b, size_t n) { return ::strncasecmp(a, b, n); }
#ifndef _stricmp
#define _stricmp stricmp
#endif

template <typename EnumType>
constexpr inline Bool equalsEnumBitFlagsCount(size_t expectedSize, EnumType bitFlagsEnd) {
    return (size_t(1) << (expectedSize - 1)) == static_cast<size_t>(bitFlagsEnd) - 1;
}

enum {
    MAX_PLAYER_COUNT = 16
};

enum ObjectID : Int { INVALID_ID = 0 };
enum DrawableID : Int { INVALID_DRAWABLE_ID = 0 };
enum ScienceType : Int { SCIENCE_INVALID = -1 };
using PlayerMaskType = UnsignedShort;

enum Relationship : Int {
    ENEMIES = 0,
    NEUTRAL = 1,
    ALLIES = 2
};

enum VeterancyLevel : Int {
    LEVEL_REGULAR = 0,
    LEVEL_VETERAN = 1,
    LEVEL_ELITE = 2,
    LEVEL_HEROIC = 3,
    LEVEL_COUNT = 4
};

struct Coord2D { Real x, y; void zero() { x = y = 0.0f; } };
struct ICoord2D { Int x, y; void zero() { x = y = 0; } };
struct Region2D { Coord2D lo, hi; };
struct IRegion2D { ICoord2D lo, hi; };
struct Coord3D { Real x, y, z; void zero() { x = y = z = 0.0f; } };
struct ICoord3D { Int x, y, z; void zero() { x = y = z = 0; } };
struct Region3D { Coord3D lo, hi; };
struct IRegion3D { ICoord3D lo, hi; };
struct RealRange { Real lo, hi; };
struct RGBColor { Real red, green, blue; };
struct RGBAColorReal { Real red, green, blue, alpha; };
struct RGBAColorInt { UnsignedInt red, green, blue, alpha; };
struct Vector4 { Real X, Y, Z, W; };
class Matrix3D {
public:
    Vector4 Row[3];
    Vector4& operator[](int i) { return Row[i]; }
    const Vector4& operator[](int i) const { return Row[i]; }
};

#include "Common/INI.h"
#include "Common/Debug.h"
#include "Common/Errors.h"
#include "Common/GameMemory.h"
#include "Common/SubsystemInterface.h"
#include "Common/AsciiString.h"
#include "Common/UnicodeString.h"
#include "Common/NameKeyGenerator.h"

class GlobalData {
    AsciiString m_userDataPath;
public:
    void setPath_UserData(const char* p) { m_userDataPath = AsciiString(p); }
    const AsciiString& getPath_UserData() const { return m_userDataPath; }
};
extern GlobalData* TheGlobalData;
