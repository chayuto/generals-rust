#pragma once

#include <algorithm>
#include <bitset>
#include <cctype>
#include <climits>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
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

#ifndef CPP_11
#define CPP_11(x) x
#endif
#ifndef FALLTHROUGH
#define FALLTHROUGH [[fallthrough]]
#endif
#ifndef ARRAY_SIZE
#define ARRAY_SIZE(x) (sizeof(x) / sizeof((x)[0]))
#endif

#ifndef RETAIL_COMPATIBLE_CRC
#define RETAIL_COMPATIBLE_CRC 0
#endif
#ifndef RETAIL_COMPATIBLE_XFER_SAVE
#define RETAIL_COMPATIBLE_XFER_SAVE 0
#endif

inline int stricmp(const char* a, const char* b) { return ::strcasecmp(a, b); }

template <typename T>
inline T min(T a, T b) { return a < b ? a : b; }

template <typename EnumType>
constexpr inline Bool equalsEnumBitFlagsCount(size_t expectedSize, EnumType bitFlagsEnd) {
    return (size_t(1) << (expectedSize - 1)) == static_cast<size_t>(bitFlagsEnd) - 1;
}

#define DEBUG_LOG(x) do {} while (0)
#define DEBUG_ASSERTCRASH(cond, msg) do { (void)(cond); } while (0)
#define DEBUG_CRASH(msg) do {} while (0)

enum ObjectID : Int { INVALID_ID = 0 };
enum ScienceType : Int { SCIENCE_INVALID = -1 };
using PlayerMaskType = UnsignedShort;

struct Coord3D {
    Real x = 0.0f, y = 0.0f, z = 0.0f;
    void zero() { x = y = z = 0.0f; }
};

class AsciiString {
    std::string m_str;
public:
    static const AsciiString TheEmptyString;
    AsciiString() = default;
    AsciiString(const char* s) : m_str(s ? s : "") {}
    const char* str() const { return m_str.c_str(); }
    bool isEmpty() const { return m_str.empty(); }
    void concat(const char* s) { if (s) m_str.append(s); }
};
