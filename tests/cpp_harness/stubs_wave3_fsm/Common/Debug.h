#pragma once
#define DEBUG_LOG(x) do {} while (0)
#define DEBUG_LOG_RAW(x) do {} while (0)
#define DEBUG_ASSERTCRASH(cond, msg) do { (void)(cond); } while (0)
#define DEBUG_ASSERTLOG(cond, msg) do { (void)(cond); } while (0)
#define DEBUG_CRASH(msg) do {} while (0)
