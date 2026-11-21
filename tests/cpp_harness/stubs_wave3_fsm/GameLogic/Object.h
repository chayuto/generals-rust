#pragma once
#include "Common/ThingTemplate.h"

class Object {
    ObjectID m_id = INVALID_ID;
    Coord3D m_pos{0.0f, 0.0f, 0.0f};
    ThingTemplate m_tmpl;
public:
    Object() = default;
    Object(ObjectID id, Coord3D pos) : m_id(id), m_pos(pos) {}
    ObjectID getID() const { return m_id; }
    const Coord3D* getPosition() const { return &m_pos; }
    const ThingTemplate* getTemplate() const { return &m_tmpl; }
};
