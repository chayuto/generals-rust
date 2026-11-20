#pragma once

class MemoryPoolObject {
public:
    virtual ~MemoryPoolObject() = default;
};

#define MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(cls, poolName) \
public: \
    virtual ~cls();
