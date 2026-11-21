#pragma once
#include <string>

class AsciiString {
    std::string m_s;
public:
    static const AsciiString TheEmptyString;
    AsciiString() = default;
    AsciiString(const char* s) : m_s(s ? s : "") {}
    const char* str() const { return m_s.c_str(); }
};
