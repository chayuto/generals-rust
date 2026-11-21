#pragma once
#include "Common/KindOf.h"
#include "Common/AsciiString.h"
#include "GameLogic/Module/ContainModule.h"

class Object {
public:
    uint32_t m_kindMask = 0;
    AsciiString m_commandSetString;
    ContainModuleInterface m_contain;
    Bool m_hasContain = FALSE;

    Bool isKindOf(KindOfType k) const {
        if (k < 0 || k >= 32) return FALSE;
        return (m_kindMask & (1u << static_cast<uint32_t>(k))) ? TRUE : FALSE;
    }
    const AsciiString& getCommandSetString() const { return m_commandSetString; }
    const ContainModuleInterface* getContain() const { return m_hasContain ? &m_contain : nullptr; }
};
