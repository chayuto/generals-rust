#pragma once
#include <cstring>
#include <vector>

class GameTextInterface {
public:
    UnicodeString fetch(const char* label) const {
        if (!label || !label[0] || std::strcmp(label, "EMPTY_LABEL") == 0) {
            return UnicodeString::TheEmptyString;
        }
        std::vector<WideChar> units;
        const char* prefix = "TXT:";
        for (const char* p = prefix; *p; ++p) {
            units.push_back(static_cast<UnsignedByte>(*p));
        }
        for (const char* p = label; *p; ++p) {
            units.push_back(static_cast<UnsignedByte>(*p));
        }
        UnicodeString out;
        out.setUnits(units.data(), static_cast<Int>(units.size()));
        return out;
    }
};

extern GameTextInterface* TheGameText;
