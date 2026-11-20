#pragma once
#include <cstdlib>

class DynamicMemoryAllocator {
public:
    int getActualAllocationSize(int minBytes) { return minBytes; }
    void* allocateBytes(int bytes, const char*) {
        return std::calloc(1, static_cast<size_t>(bytes));
    }
    void freeBytes(void* p) { std::free(p); }
};

extern DynamicMemoryAllocator* TheDynamicMemoryAllocator;
