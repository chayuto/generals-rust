#pragma once
#include "Common/AcademyStats.h"
#include "Common/Energy.h"
#include "Common/Money.h"
#include "Common/PlayerTemplate.h"
#include "GameLogic/Object.h"
#include <vector>

using ObjectIterateFunc = void (*)(Object*, void*);

class Player {
public:
    const PlayerTemplate* m_template = nullptr;
    Money m_money;
    Energy m_energy;
    AcademyStats m_academyStats;
    std::vector<Object*> m_objects;

    const PlayerTemplate* getPlayerTemplate() const { return m_template; }
    const Money* getMoney() const { return &m_money; }
    const Energy* getEnergy() const { return &m_energy; }
    AcademyStats* getAcademyStats() { return &m_academyStats; }
    const AcademyStats* getAcademyStats() const { return &m_academyStats; }

    void iterateObjects(ObjectIterateFunc fn, void* userData) const {
        for (Object* obj : m_objects) {
            fn(obj, userData);
        }
    }
};
