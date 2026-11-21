#pragma once
#include <vector>

class UnicodeString {
    std::vector<WideChar> m_data;
public:
    static const UnicodeString TheEmptyString;

    UnicodeString() = default;
    const WideChar* str() const {
        static const WideChar kEmpty = 0;
        return m_data.empty() ? &kEmpty : m_data.data();
    }
    Int getLength() const { return static_cast<Int>(m_data.size()); }
    void clear() { m_data.clear(); }
    void set(const WideChar* s) {
        m_data.clear();
        if (!s) return;
        while (*s != 0) {
            m_data.push_back(*s);
            ++s;
        }
    }
    void setUnits(const WideChar* s, Int len) {
        if (!s || len <= 0) {
            m_data.clear();
            return;
        }
        m_data.assign(s, s + len);
    }
    bool isEmpty() const { return m_data.empty(); }
};
