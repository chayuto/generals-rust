#pragma once
#include <string>

class AsciiString {
private:
    std::string m_data;

public:
    static const AsciiString TheEmptyString;

    AsciiString() = default;
    AsciiString(const char* s) : m_data(s ? s : "") {}
    const char* str() const { return m_data.c_str(); }
    Int getLength() const { return static_cast<Int>(m_data.size()); }
    void clear() { m_data.clear(); }
    void set(const char* s) { m_data = s ? s : ""; }
    bool operator==(const AsciiString& o) const { return m_data == o.m_data; }
};
