#pragma once
#include <algorithm>
#include <type_traits>
#include <vector>

class MemoryPoolObject {
public:
    virtual ~MemoryPoolObject() = default;
};

#define MEMORY_POOL_GLUE_ABC(cls) \
public: \
    virtual ~cls();

#define MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(cls, poolName) \
public: \
    ~cls();

#define EMPTY_DTOR(cls) inline cls::~cls() {}

struct HarnessAllocTracker {
    static inline std::vector<MemoryPoolObject*> tracked;

    template <typename T>
    static T* track(T* p) {
        if constexpr (std::is_base_of_v<MemoryPoolObject, T>) {
            tracked.push_back(p);
        }
        return p;
    }

    static bool isTracked(MemoryPoolObject* p) {
        return std::find(tracked.begin(), tracked.end(), p) != tracked.end();
    }

    static void cleanup() {
        for (MemoryPoolObject* p : tracked) {
            delete p;
        }
        tracked.clear();
    }
};

#define newInstance(cls) HarnessAllocTracker::track(new cls())

inline void deleteInstance(MemoryPoolObject* p) {
    if (p && !HarnessAllocTracker::isTracked(p)) {
        delete p;
    }
}