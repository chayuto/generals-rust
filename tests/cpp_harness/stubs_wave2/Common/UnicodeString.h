#pragma once
#include <cstdint>
#include <cstdlib>
#include <cstring>

// Pointer-sized refcounted UnicodeString stub matching Dict.cpp's sizeof(UnicodeString) <= sizeof(void*)
// invariant and zero-initialized empty-string layout.
class UnicodeString {
private:
    struct Data {
        uint32_t ref_count;
        uint32_t len;
        WideChar buf[1];
    };
    Data* m_data;

    static Data* alloc_data(const WideChar* s, uint32_t n) {
        if (!s || n == 0) return nullptr;
        Data* d = static_cast<Data*>(std::malloc(sizeof(Data) + n * sizeof(WideChar)));
        d->ref_count = 1;
        d->len = n;
        std::memcpy(d->buf, s, n * sizeof(WideChar));
        d->buf[n] = 0;
        return d;
    }

public:
    static const UnicodeString TheEmptyString;

    UnicodeString() : m_data(nullptr) {}
    void setUnits(const WideChar* s, Int len) {
        Data* next = (len > 0) ? alloc_data(s, static_cast<uint32_t>(len)) : nullptr;
        clear();
        m_data = next;
    }
    UnicodeString(const UnicodeString& o) : m_data(o.m_data) {
        if (m_data) ++m_data->ref_count;
    }
    ~UnicodeString() { clear(); }

    UnicodeString& operator=(const UnicodeString& o) {
        if (this != &o) {
            Data* next = o.m_data;
            if (next) ++next->ref_count;
            clear();
            m_data = next;
        }
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

    const WideChar* str() const {
        static const WideChar kEmpty = 0;
        return m_data ? m_data->buf : &kEmpty;
    }
    Int getLength() const { return m_data ? static_cast<Int>(m_data->len) : 0; }
};

static_assert(sizeof(UnicodeString) == sizeof(void*), "Dict assumes UnicodeString fits in void*");
