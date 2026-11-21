#pragma once
#include <cstdlib>

class MemoryPoolObject {
public:
    virtual ~MemoryPoolObject() = default;
};

class DynamicMemoryAllocator {
public:
    int getActualAllocationSize(int minBytes) { return minBytes; }
    void* allocateBytes(int bytes, const char*) {
        return std::calloc(1, static_cast<size_t>(bytes > 0 ? bytes : 1));
    }
    void freeBytes(void* p) { std::free(p); }
};

extern DynamicMemoryAllocator* TheDynamicMemoryAllocator;

#define MEMORY_POOL_GLUE_ABC(cls) \
public: \
    virtual ~cls();

#define MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(cls, poolName) \
public: \
    ~cls();

#define EMPTY_DTOR(cls) inline cls::~cls() {}

#ifndef newInstance
#define newInstance(cls) new cls
#endif

inline void deleteInstance(MemoryPoolObject* p) {
    delete p;
}
