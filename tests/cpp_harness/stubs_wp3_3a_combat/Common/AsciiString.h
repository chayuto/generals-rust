#pragma once
#include <cctype>
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
    void set(const AsciiString& o) { m_str = o.m_str; }
    void clear() { m_str.clear(); }
    void concat(char c) { m_str.push_back(c); }
    void concat(const char* s) { if (s) m_str.append(s); }
    void concat(const AsciiString& o) { m_str.append(o.m_str); }
    void toLower() {
        for (char& ch : m_str) {
            ch = static_cast<char>(std::tolower(static_cast<unsigned char>(ch)));
        }
    }
    Int compare(const AsciiString& o) const {
        return std::strcmp(m_str.c_str(), o.m_str.c_str());
    }
    Int compareNoCase(const AsciiString& o) const {
        return ::strcasecmp(m_str.c_str(), o.m_str.c_str());
    }
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
    bool operator==(const AsciiString& o) const { return m_str == o.m_str; }
    bool operator!=(const AsciiString& o) const { return m_str != o.m_str; }
    bool operator==(const char* s) const { return m_str == (s ? s : ""); }
    bool operator<(const AsciiString& o) const { return m_str < o.m_str; }
};
