#pragma once
#include "Common/AsciiString.h"

class PlayerTemplate {
public:
    AsciiString m_baseSide;
    const AsciiString& getBaseSide() const { return m_baseSide; }
};
