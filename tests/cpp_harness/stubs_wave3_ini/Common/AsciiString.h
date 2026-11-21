#pragma once
#include <cstring>
#include <string>

class AsciiString {
    std::string m_str;
public:
    static const AsciiString TheEmptyString;

    AsciiString() = default;
    AsciiString(const char* s) : m_str(s ? s : "") {}
    const char* str() const { return m_str.c_str(); }
    bool isEmpty() const { return m_str.empty(); }
    Int getLength() const { return static_cast<Int>(m_str.size()); }
    void set(const char* s) { m_str = s ? s : ""; }
    void clear() { m_str.clear(); }
    void concat(char c) { m_str.push_back(c); }
    void concat(const char* s) { if (s) m_str.append(s); }
    bool endsWithNoCase(const char* suffix) const {
        if (!suffix) return false;
        size_t sl = std::strlen(suffix);
        if (m_str.size() < sl) return false;
        return ::strcasecmp(m_str.c_str() + m_str.size() - sl, suffix) == 0;
    }
    void truncateBy(Int n) {
        if (n > 0 && static_cast<size_t>(n) <= m_str.size()) {
            m_str.resize(m_str.size() - static_cast<size_t>(n));
        }
    }
};
