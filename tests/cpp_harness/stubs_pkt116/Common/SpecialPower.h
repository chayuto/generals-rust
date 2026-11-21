#pragma once
#include "Common/AcademyStats.h"

class SpecialPowerTemplate {
public:
    AcademyClassificationType m_class = ACT_NONE;
    AcademyClassificationType getAcademyClassificationType() const { return m_class; }
};
