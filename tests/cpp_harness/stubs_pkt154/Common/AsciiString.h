#pragma once
#include <cctype>
#include <cstdint>
#include <cstdlib>
#include <cstring>

class AsciiString {
private:
    struct Data {
        uint32_t ref_count;
        uint32_t len;
        char buf[1];
    };
    Data* m_data;

    static Data* alloc_data(const char* s, uint32_t n) {
        if (n == 0) return nullptr;
        Data* d = static_cast<Data*>(std::malloc(sizeof(Data) + n));
        d->ref_count = 1;
        d->len = n;
        if (s) {
            std::memcpy(d->buf, s, n);
        } else {
            std::memset(d->buf, 0, n);
        }
        d->buf[n] = '\0';
        return d;
    }

    static Data* alloc_cstr(const char* s) {
        if (!s || s[0] == '\0') return nullptr;
        return alloc_data(s, static_cast<uint32_t>(std::strlen(s)));
    }

public:
    static const AsciiString TheEmptyString;

    AsciiString() : m_data(nullptr) {}
    AsciiString(const char* s) : m_data(alloc_cstr(s)) {}
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
        Data* next = alloc_cstr(s);
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

    void set(const char* s) { *this = s; }
    void set(const AsciiString& o) { *this = o; }

    char* getBufferForRead(Int len) {
        clear();
        if (len <= 0) {
            static char kDummy = '\0';
            return &kDummy;
        }
        m_data = alloc_data(nullptr, static_cast<uint32_t>(len));
        return m_data->buf;
    }

    void concat(char c) {
        char tmp[2] = { c, '\0' };
        concat(tmp);
    }

    void concat(const char* s) {
        if (!s || s[0] == '\0') return;
        uint32_t a = static_cast<uint32_t>(getLength());
        uint32_t b = static_cast<uint32_t>(std::strlen(s));
        Data* d = alloc_data(nullptr, a + b);
        if (a > 0) std::memcpy(d->buf, str(), a);
        std::memcpy(d->buf + a, s, b);
        d->buf[a + b] = '\0';
        clear();
        m_data = d;
    }

    void concat(const AsciiString& o) { concat(o.str()); }

    void toLower() {
        if (!m_data) return;
        Data* d = alloc_data(m_data->buf, m_data->len);
        for (uint32_t i = 0; i < d->len; ++i) {
            d->buf[i] = static_cast<char>(std::tolower(static_cast<unsigned char>(d->buf[i])));
        }
        clear();
        m_data = d;
    }

    Bool startsWith(const char* p) const {
        if (!p) return FALSE;
        size_t plen = std::strlen(p);
        size_t mlen = static_cast<size_t>(getLength());
        return (mlen >= plen && std::strncmp(str(), p, plen) == 0) ? TRUE : FALSE;
    }

    Bool startsWith(const AsciiString& o) const { return startsWith(o.str()); }

    bool endsWithNoCase(const char* suffix) const {
        if (!suffix) return false;
        size_t sl = std::strlen(suffix);
        size_t ml = static_cast<size_t>(getLength());
        if (ml < sl) return false;
        return ::strcasecmp(str() + ml - sl, suffix) == 0;
    }

    void truncateBy(Int n) {
        if (n <= 0 || !m_data) return;
        uint32_t cur = m_data->len;
        if (static_cast<uint32_t>(n) >= cur) {
            clear();
            return;
        }
        Data* d = alloc_data(m_data->buf, cur - static_cast<uint32_t>(n));
        clear();
        m_data = d;
    }

    Bool nextToken(AsciiString* tok, const char* separators = nullptr) {
        if (isEmpty()) {
            tok->clear();
            return FALSE;
        }
        const char* seps = separators ? separators : " \n\r\t";
        const char* s = str();
        size_t mlen = static_cast<size_t>(getLength());
        size_t start = 0;
        while (start < mlen && std::strchr(seps, s[start]) != nullptr) ++start;
        size_t end = start;
        while (end < mlen && std::strchr(seps, s[end]) == nullptr) ++end;
        if (end > start) {
            Data* tokData = alloc_data(s + start, static_cast<uint32_t>(end - start));
            Data* restData = alloc_cstr(s + end);
            tok->clear();
            tok->m_data = tokData;
            clear();
            m_data = restData;
            return TRUE;
        }
        clear();
        tok->clear();
        return FALSE;
    }

    const char* str() const { return m_data ? m_data->buf : ""; }
    Int getLength() const { return m_data ? static_cast<Int>(m_data->len) : 0; }
    bool isEmpty() const { return getLength() == 0; }
    bool isNotEmpty() const { return !isEmpty(); }

    bool operator==(const AsciiString& o) const {
        return getLength() == o.getLength() &&
               std::memcmp(str(), o.str(), static_cast<size_t>(getLength())) == 0;
    }
    bool operator!=(const AsciiString& o) const { return !(*this == o); }
    bool operator==(const char* s) const { return std::strcmp(str(), s ? s : "") == 0; }
    bool operator!=(const char* s) const { return !(*this == s); }
    bool operator<(const AsciiString& o) const { return compare(o) < 0; }
    Int compare(const AsciiString& o) const { return std::strcmp(str(), o.str()); }
    Int compareNoCase(const AsciiString& o) const { return ::strcasecmp(str(), o.str()); }
};

static_assert(sizeof(AsciiString) == sizeof(void*), "Dict assumes AsciiString fits in void*");
