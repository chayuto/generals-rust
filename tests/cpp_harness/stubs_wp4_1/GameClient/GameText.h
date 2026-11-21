#pragma once
#include <cstring>
#include <vector>

class GameTextInterface {
public:
    UnicodeString fetch(const char* label, Bool* exists = nullptr) const {
        if (!label || !label[0] || std::strcmp(label, "EMPTY_LABEL") == 0) {
            if (exists) *exists = FALSE;
            return UnicodeString::TheEmptyString;
        }
        if (exists) *exists = TRUE;
        const char* mapped = nullptr;
        if (std::strcmp(label, "CONTROLBAR:OrRequirement") == 0) {
            mapped = "OR_REQ";
        } else if (std::strcmp(label, "CONTROLBAR:GeneralsPromotion") == 0) {
            mapped = "GEN_PROMO";
        }
        std::vector<WideChar> units;
        if (mapped) {
            for (const char* p = mapped; *p; ++p) {
                units.push_back(static_cast<UnsignedByte>(*p));
            }
        } else {
            const char* prefix = "TXT:";
            for (const char* p = prefix; *p; ++p) {
                units.push_back(static_cast<UnsignedByte>(*p));
            }
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
