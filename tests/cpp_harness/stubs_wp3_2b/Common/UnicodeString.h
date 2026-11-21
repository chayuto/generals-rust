#pragma once
#include <vector>

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
    bool isEmpty() const { return getLength() == 0; }
};
