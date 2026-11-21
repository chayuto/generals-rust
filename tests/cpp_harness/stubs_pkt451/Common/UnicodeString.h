#pragma once
#include <cstdint>
#include <cstdlib>
#include <cstring>

class UnicodeString {
private:
    struct Data {
        uint32_t ref_count;
        uint32_t len;
        WideChar buf[1];
    };
    Data* m_data;

    static Data* alloc_data(const WideChar* s, uint32_t n) {
        if (n == 0) return nullptr;
        Data* d = static_cast<Data*>(std::malloc(sizeof(Data) + n * sizeof(WideChar)));
        d->ref_count = 1;
        d->len = n;
        if (s) {
            std::memcpy(d->buf, s, n * sizeof(WideChar));
        } else {
            std::memset(d->buf, 0, n * sizeof(WideChar));
        }
        d->buf[n] = 0;
        return d;
    }

public:
    static const UnicodeString TheEmptyString;

    UnicodeString() : m_data(nullptr) {}
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

    WideChar* getBufferForRead(Int len) {
        clear();
        if (len <= 0) {
            static WideChar kDummy = 0;
            return &kDummy;
        }
        m_data = alloc_data(nullptr, static_cast<uint32_t>(len));
        return m_data->buf;
    }

    const WideChar* str() const {
        static const WideChar kEmpty = 0;
        return m_data ? m_data->buf : &kEmpty;
    }
    Int getLength() const { return m_data ? static_cast<Int>(m_data->len) : 0; }
    bool isEmpty() const { return getLength() == 0; }
    bool operator==(const UnicodeString& o) const {
        return getLength() == o.getLength() &&
               std::memcmp(str(), o.str(), static_cast<size_t>(getLength()) * sizeof(WideChar)) == 0;
    }
};

static_assert(sizeof(UnicodeString) == sizeof(void*), "Dict assumes UnicodeString fits in void*");
