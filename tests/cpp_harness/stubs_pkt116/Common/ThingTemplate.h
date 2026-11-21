#pragma once
#include "Common/KindOf.h"

class Player;

class ThingTemplate {
public:
    uint32_t m_kindMask = 0;
    Int m_buildCost = 1000;

    Bool isKindOf(KindOfType k) const {
        if (k < 0 || k >= 32) return FALSE;
        return (m_kindMask & (1u << static_cast<uint32_t>(k))) ? TRUE : FALSE;
    }
    Int calcCostToBuild(const Player*) const { return m_buildCost; }
};
