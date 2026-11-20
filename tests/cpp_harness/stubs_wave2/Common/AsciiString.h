#pragma once
#include <cstdint>
#include <cstdlib>
#include <cstring>

// Pointer-sized refcounted AsciiString stub matching Dict.cpp's sizeof(AsciiString) <= sizeof(void*)
// invariant and zero-initialized empty-string layout.
class AsciiString {
private:
    struct Data {
        uint32_t ref_count;
        uint32_t len;
        char buf[1];
    };
    Data* m_data;

    static Data* alloc_data(const char* s) {
        if (!s || s[0] == '\0') return nullptr;
        uint32_t n = static_cast<uint32_t>(std::strlen(s));
        Data* d = static_cast<Data*>(std::malloc(sizeof(Data) + n));
        d->ref_count = 1;
        d->len = n;
        std::memcpy(d->buf, s, n + 1);
        return d;
    }

public:
    static const AsciiString TheEmptyString;

    AsciiString() : m_data(nullptr) {}
    AsciiString(const char* s) : m_data(alloc_data(s)) {}
    AsciiString(const AsciiString& o) : m_data(o.m_data) {
        if (m_data) ++m_data->ref_count;
    }
    ~AsciiString() { clear(); }

    AsciiString& operator=(const AsciiString& o) {
        if (this != &o) {
            Data* next = o.m_data;
            if (next) ++next->ref_count;
            clear();
            m_data = next;
        }
        return *this;
    }

    AsciiString& operator=(const char* s) {
        Data* next = alloc_data(s);
        clear();
        m_data = next;
        return *this;
    }

    void clear() {
        if (m_data) {
            if (--m_data->ref_count == 0) {
                std::free(m_data);
            }
            m_data = nullptr;
        }
    }

    const char* str() const { return m_data ? m_data->buf : ""; }
    Int getLength() const { return m_data ? static_cast<Int>(m_data->len) : 0; }
};

static_assert(sizeof(AsciiString) == sizeof(void*), "Dict assumes AsciiString fits in void*");
