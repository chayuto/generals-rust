#pragma once
#include "PreRTS.h"

class ThingTemplate {
public:
    Bool m_isTrainable = TRUE;
    Int m_expRequired[LEVEL_COUNT] = {0, 50, 100, 200};
    Int m_expValue[LEVEL_COUNT] = {10, 20, 30, 50};

    Bool isTrainable() const { return m_isTrainable; }
    Int getExperienceRequired(Int level) const {
        if (level >= 0 && level < LEVEL_COUNT) return m_expRequired[level];
        return 0;
    }
    Int getExperienceValue(Int level) const {
        if (level >= 0 && level < LEVEL_COUNT) return m_expValue[level];
        return 0;
    }
};
