#pragma once
#include <cstdio>

#define DEBUG_LOG(x) do {} while (0)
#define DEBUG_ASSERTCRASH(cond, msg) do { if (!(cond)) { std::fprintf(stderr, "ASSERT: "); std::printf msg; std::fprintf(stderr, "\n"); } } while (0)
#define DEBUG_ASSERTLOG(cond, msg) do { (void)(cond); } while (0)
#define DEBUG_CRASH(msg) do { std::fprintf(stderr, "CRASH: "); std::printf msg; std::fprintf(stderr, "\n"); } while (0)
