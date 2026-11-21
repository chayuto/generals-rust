#pragma once
#include "Common/UnicodeString.h"

class GameTextInterface {
public:
    UnicodeString fetch(const char* label) const {
        std::vector<WideChar> units;
        if (label) {
            for (const char* p = label; *p; ++p) {
                units.push_back(static_cast<UnsignedByte>(*p));
            }
        }
        UnicodeString out;
        out.setUnits(units.data(), static_cast<Int>(units.size()));
        return out;
    }
};

extern GameTextInterface* TheGameText;
