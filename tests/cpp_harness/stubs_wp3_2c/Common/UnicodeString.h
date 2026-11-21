#pragma once
#include <vector>
#include "Common/AsciiString.h"

class UnicodeString {
    std::vector<WideChar> m_data; // always NUL-terminated when non-empty
public:
    static const UnicodeString TheEmptyString;

    UnicodeString() = default;
    const WideChar* str() const {
        static const WideChar kEmpty = 0;
        return m_data.empty() ? &kEmpty : m_data.data();
    }
    Int getLength() const {
        return m_data.empty() ? 0 : static_cast<Int>(m_data.size() - 1);
    }
    void clear() { m_data.clear(); }
    void set(const WideChar* s) {
        m_data.clear();
        if (!s || *s == 0) return;
        while (*s != 0) {
            m_data.push_back(*s);
            ++s;
        }
        m_data.push_back(0);
    }
    void setUnits(const WideChar* s, Int len) {
        if (!s || len <= 0) {
            m_data.clear();
            return;
        }
        m_data.assign(s, s + len);
        m_data.push_back(0);
    }
    void concat(const UnicodeString& o) {
        if (o.isEmpty()) return;
        if (!m_data.empty()) m_data.pop_back();
        const Int len = o.getLength();
        const WideChar* p = o.str();
        for (Int i = 0; i < len; ++i) {
            m_data.push_back(p[i]);
        }
        m_data.push_back(0);
    }
    void concat(const wchar_t* s) {
        if (!s || *s == 0) return;
        if (!m_data.empty()) m_data.pop_back();
        while (*s != 0) {
            m_data.push_back(static_cast<WideChar>(*s));
            ++s;
        }
        m_data.push_back(0);
    }
    void translate(const AsciiString& s) {
        m_data.clear();
        if (s.isEmpty()) return;
        for (const char* p = s.str(); *p; ++p) {
            m_data.push_back(static_cast<UnsignedByte>(*p));
        }
        m_data.push_back(0);
    }
    UnicodeString& operator=(const wchar_t* s) {
        m_data.clear();
        if (s && *s != 0) {
            while (*s != 0) {
                m_data.push_back(static_cast<WideChar>(*s));
                ++s;
            }
            m_data.push_back(0);
        }
        return *this;
    }
    bool operator==(const UnicodeString& o) const {
        return m_data == o.m_data;
    }
    bool isEmpty() const { return getLength() == 0; }
};
