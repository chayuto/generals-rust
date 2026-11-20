#pragma once
#include "PreRTS.h"
#include "GameLogic/Object.h"
#include <map>

class GameLogic {
public:
    UnsignedInt m_frame = 0;
    std::map<ObjectID, Object*> m_objects;

    UnsignedInt getFrame() const { return m_frame; }
    Object* findObjectByID(ObjectID id) {
        auto it = m_objects.find(id);
        return (it != m_objects.end()) ? it->second : nullptr;
    }
};

extern GameLogic* TheGameLogic;
