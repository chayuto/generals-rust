#pragma once
#include <cstddef>
#include <cstdlib>

class MemoryPoolObject {
public:
    virtual ~MemoryPoolObject() = default;
    static void* operator new(std::size_t sz) {
        return std::calloc(1, sz > 0 ? sz : 1);
    }
    static void operator delete(void* p) noexcept {
        std::free(p);
    }
};

#define MEMORY_POOL_GLUE_ABC(cls) \
public: \
    virtual ~cls();

#define MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(cls, poolName) \
public: \
    ~cls();

#define EMPTY_DTOR(cls) inline cls::~cls() {}

#ifndef NEW
#define NEW new
#endif
#ifndef MSGNEW
#define MSGNEW(tag) new
#endif
#ifndef newInstance
#define newInstance(cls) new cls
#endif

inline void deleteInstance(MemoryPoolObject* p) {
    delete p;
}
