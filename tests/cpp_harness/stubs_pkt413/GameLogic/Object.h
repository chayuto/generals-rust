#pragma once
#include "GameLogic/Module/BodyModule.h"

class Object {
public:
    ObjectID m_id = INVALID_ID;
    Bool m_hasBody = TRUE;
    BodyModuleInterface m_body;
    KindOfType m_transparent = KINDOF_INVALID;

    ObjectID getID() const { return m_id; }
    BodyModuleInterface* getBodyModule() { return m_hasBody ? &m_body : nullptr; }
    const BodyModuleInterface* getBodyModule() const { return m_hasBody ? &m_body : nullptr; }
    Bool isKindOf(KindOfType k) const { return m_transparent == k ? TRUE : FALSE; }
};
