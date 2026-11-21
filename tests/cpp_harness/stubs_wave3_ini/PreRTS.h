#pragma once

#include <algorithm>
#include <bitset>
#include <cctype>
#include <climits>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <ctime>
#include <list>
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
using Color = UnsignedInt;

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
#define RETAIL_COMPATIBLE_XFER_SAVE 0
#endif

#include <gmath.h>

inline int stricmp(const char* a, const char* b) { return ::strcasecmp(a, b); }
inline int strnicmp(const char* a, const char* b, size_t n) { return ::strncasecmp(a, b, n); }

inline bool endsWithNoCase(const char* s, const char* suffix) {
    if (!s || !suffix) return false;
    size_t sl = std::strlen(s);
    size_t sufl = std::strlen(suffix);
    if (sl < sufl) return false;
    return ::strcasecmp(s + sl - sufl, suffix) == 0;
}

inline const char* getFirstPathSeparator(const char* p) {
    if (!p) return nullptr;
    for (; *p; ++p) {
        if (*p == '\\' || *p == '/') return p;
    }
    return nullptr;
}

template <typename NUM>
inline NUM sqr(NUM x) { return x * x; }

template <typename NUM>
inline NUM clamp(NUM lo, NUM val, NUM hi) {
    if (val < lo) return lo;
    if (val > hi) return hi;
    return val;
}

enum ObjectID : Int { INVALID_ID = 0 };
enum ScienceType : Int { SCIENCE_INVALID = -1 };
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
using VeterancyLevelFlags = UnsignedInt;
constexpr VeterancyLevelFlags VETERANCY_LEVEL_FLAGS_ALL = 0xffffffffu;
constexpr VeterancyLevelFlags VETERANCY_LEVEL_FLAGS_NONE = 0x00000000u;
inline VeterancyLevelFlags setVeterancyLevelFlag(VeterancyLevelFlags f, VeterancyLevel v) {
    return f | (1u << v);
}
inline VeterancyLevelFlags clearVeterancyLevelFlag(VeterancyLevelFlags f, VeterancyLevel v) {
    return f & ~(1u << v);
}

struct Coord2D { Real x, y; void zero() { x = y = 0.0f; } };
struct ICoord2D { Int x, y; void zero() { x = y = 0; } };
struct Coord3D { Real x, y, z; void zero() { x = y = z = 0.0f; } };
struct RGBColor { Real red, green, blue; };
struct RGBAColorInt { UnsignedInt red, green, blue, alpha; };

inline Color GameMakeColor(UnsignedByte red, UnsignedByte green, UnsignedByte blue, UnsignedByte alpha) {
    return (static_cast<UnsignedInt>(alpha) << 24)
        | (static_cast<UnsignedInt>(red) << 16)
        | (static_cast<UnsignedInt>(green) << 8)
        | static_cast<UnsignedInt>(blue);
}

#define newInstance(T) new T()

template <typename T>
class RefCountPtr {
    T* ptr = nullptr;
public:
    RefCountPtr() = default;
    T* Peek() const { return ptr; }
    void Clear() { ptr = nullptr; }
    bool operator==(std::nullptr_t) const { return ptr == nullptr; }
    operator bool() const { return ptr != nullptr; }
    T* operator->() const { return ptr; }
    RefCountPtr& operator=(T* p) { ptr = p; return *this; }
};
template <typename T>
inline T* Create_No_Add_Ref(T* p) { return p; }

#include "Common/Debug.h"
#include "Common/Errors.h"
#include "Common/AsciiString.h"
#include "Common/UnicodeString.h"
#include "Common/RandomValue.h"
#include "GameLogic/LogicRandomValue.h"
#include "GameClient/ClientRandomValue.h"
#include "Common/AudioRandomValue.h"
