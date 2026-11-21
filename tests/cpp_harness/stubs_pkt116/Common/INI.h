#pragma once

class INI {
public:
    const char* getNextToken() { return ""; }
    static void parseUnsignedInt(INI*, void*, void* store, const void*) {
        *static_cast<UnsignedInt*>(store) = 0;
    }
};
