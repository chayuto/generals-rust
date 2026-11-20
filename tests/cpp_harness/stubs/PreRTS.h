// Minimal PreRTS.h harness stub for standalone compilation of vendor/GeneralsX logic units
#pragma once

#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <ctime>
#include <climits>
#include <vector>

using Int = int32_t;
using UnsignedInt = uint32_t;
using Short = int16_t;
using UnsignedShort = uint16_t;
using Byte = int8_t;
using UnsignedByte = uint8_t;
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

#include "Common/GameCommon.h"
#include "Common/GameType.h"
#include "Common/Debug.h"
#include "GameLogic/LogicRandomValue.h"
#include "GameClient/ClientRandomValue.h"
