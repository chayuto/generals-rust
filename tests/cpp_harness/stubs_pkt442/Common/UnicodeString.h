#pragma once
#include <string>
#include <vector>

class UnicodeString {
    std::vector<WideChar> m_u16;
public:
    static const UnicodeString TheEmptyString;
    bool isEmpty() const { return m_u16.empty(); }
    const WideChar* str() const {
        static const WideChar kEmpty = 0;
        return m_u16.empty() ? &kEmpty : m_u16.data();
    }
    void set(const WideChar* p) {
        m_u16.clear();
        if (p) {
            while (*p) m_u16.push_back(*p++);
            m_u16.push_back(0);
        }
    }
};
