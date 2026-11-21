#pragma once
#include <map>

class Object;

class GameLogic {
public:
    UnsignedInt m_frame = 1;
    std::map<ObjectID, Object*> m_objects;

    UnsignedInt getFrame() const { return m_frame; }
    Object* findObjectByID(ObjectID id) const {
        auto it = m_objects.find(id);
        return (it == m_objects.end()) ? nullptr : it->second;
    }
};

extern GameLogic* TheGameLogic;
