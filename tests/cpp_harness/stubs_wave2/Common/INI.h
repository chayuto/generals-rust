#pragma once

class INI {
public:
    const char* getNextToken() { return ""; }
    static Int scanIndexList(const char*, const char* const*) { return 0; }
    static Bool scanBool(const char*) { return FALSE; }
    static Real scanReal(const char*) { return 0.0f; }
};
