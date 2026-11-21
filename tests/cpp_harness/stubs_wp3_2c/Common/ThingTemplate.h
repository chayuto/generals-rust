#pragma once
#include "Common/AsciiString.h"
#include "Common/UnicodeString.h"

class ThingTemplate {
    AsciiString m_name;
    UnicodeString m_displayName;
    Bool m_isStructure = FALSE;
public:
    ThingTemplate() = default;
    ThingTemplate(AsciiString name, const char* displayAscii = "", Bool isStructure = FALSE)
        : m_name(name), m_isStructure(isStructure) {
        m_displayName.translate(AsciiString(displayAscii));
    }
    const AsciiString& getName() const { return m_name; }
    const UnicodeString& getDisplayName() const { return m_displayName; }
    Bool isKindOf(KindOfType k) const {
        return k == KINDOF_STRUCTURE ? m_isStructure : FALSE;
    }
};
