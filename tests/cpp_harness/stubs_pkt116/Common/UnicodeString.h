#pragma once
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <cwchar>

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
    void set(const WideChar* s) {
        uint32_t n = 0;
        if (s) {
            while (s[n] != 0) ++n;
        }
        setUnits(s, static_cast<Int>(n));
    }
    void format(const wchar_t* fmt) {
        if (!fmt) { clear(); return; }
        std::vector<WideChar> tmp;
        for (const wchar_t* p = fmt; *p; ++p) {
            tmp.push_back(static_cast<WideChar>(*p));
        }
        setUnits(tmp.data(), static_cast<Int>(tmp.size()));
    }
    void concat(const UnicodeString& other) {
        Int olen = other.getLength();
        if (olen <= 0) return;
        Int clen = getLength();
        std::vector<WideChar> tmp(static_cast<size_t>(clen + olen));
        if (clen > 0) std::memcpy(tmp.data(), str(), static_cast<size_t>(clen) * sizeof(WideChar));
        std::memcpy(tmp.data() + clen, other.str(), static_cast<size_t>(olen) * sizeof(WideChar));
        setUnits(tmp.data(), clen + olen);
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
