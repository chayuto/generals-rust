#pragma once
#include "Common/ThingTemplate.h"

class Object {
public:
    ThingTemplate m_tmpl;
    Bool m_isDisabled = FALSE;

    Object() = default;
    Object(Bool is_struct, Int energy_prod, Int energy_bonus, Bool is_disabled)
        : m_tmpl(is_struct, energy_prod, energy_bonus), m_isDisabled(is_disabled) {}

    const ThingTemplate* getTemplate() const { return &m_tmpl; }
    Bool isDisabled() const { return m_isDisabled; }
};
