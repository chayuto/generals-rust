#pragma once
#include "Common/KindOf.h"

class ThingTemplate {
public:
    Bool m_isStructure = FALSE;
    Int m_energyProduction = 0;
    Int m_energyBonus = 0;

    ThingTemplate() = default;
    ThingTemplate(Bool is_struct, Int energy_prod, Int energy_bonus)
        : m_isStructure(is_struct), m_energyProduction(energy_prod), m_energyBonus(energy_bonus) {}

    Bool isKindOf(KindOfType k) const {
        return (k == KINDOF_STRUCTURE) ? m_isStructure : FALSE;
    }
    Int getEnergyProduction() const { return m_energyProduction; }
    Int getEnergyBonus() const { return m_energyBonus; }
    AsciiString getName() const { return AsciiString::TheEmptyString; }
};
