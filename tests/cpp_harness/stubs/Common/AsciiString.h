#pragma once
#include <cctype>
#include <cstring>
#include <string>

#ifndef _WIN32
#include <strings.h>
#ifndef stricmp
#define stricmp strcasecmp
#endif
#ifndef _stricmp
#define _stricmp strcasecmp
#endif
#endif

class AsciiString {
private:
    std::string m_data;

public:
    static const AsciiString TheEmptyString;

    AsciiString() = default;
    AsciiString(const char* s) : m_data(s ? s : "") {}
    AsciiString(const AsciiString&) = default;
    AsciiString& operator=(const AsciiString&) = default;
    AsciiString& operator=(const char* s) {
        m_data = s ? s : "";
        return *this;
    }

    const char* str() const { return m_data.c_str(); }
    void concat(const char* s) { if (s) m_data += s; }
    int compare(const AsciiString& other) const { return std::strcmp(str(), other.str()); }
    int compareNoCase(const AsciiString& other) const { return _stricmp(str(), other.str()); }
    void toLower() {
        for (char& c : m_data) {
            c = static_cast<char>(std::tolower(static_cast<unsigned char>(c)));
        }
    }
};
