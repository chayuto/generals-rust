#pragma once

#ifndef NEW
#define NEW new
#endif

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

#ifndef newInstance
#define newInstance(cls) new cls
#endif

inline void deleteInstance(MemoryPoolObject* p) {
    delete p;
}
