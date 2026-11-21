#pragma once

#include <algorithm>
#include <cctype>
#include <climits>
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
#ifndef __forceinline
#define __forceinline inline
#endif
#ifndef ARRAY_SIZE
#define ARRAY_SIZE(x) (sizeof(x) / sizeof((x)[0]))
#endif

using std::max;
using std::min;

#define BitIsSet(x, i) (((x) & (i)) != 0)
#define BitSet(x, i) ((x) |= (i))
#define BitClear(x, i) ((x) &= ~(i))

inline int stricmp(const char* a, const char* b) { return ::strcasecmp(a, b); }
#ifndef _stricmp
#define _stricmp stricmp
#endif

#include "Common/Debug.h"
#include "Common/Errors.h"
#include "Common/GameMemory.h"
#include "Common/SubsystemInterface.h"
#include "Common/AsciiString.h"
#include "Common/UnicodeString.h"

class INI {
public:
    const char* getNextToken() { return ""; }
};
#include "Common/NameKeyGenerator.h"

class GlobalData {
    AsciiString m_userDataPath;
public:
    void setPath_UserData(const char* p) { m_userDataPath = AsciiString(p); }
    const AsciiString& getPath_UserData() const { return m_userDataPath; }
};

extern GlobalData* TheGlobalData;
