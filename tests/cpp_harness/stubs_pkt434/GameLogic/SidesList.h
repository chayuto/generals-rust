#pragma once
#include "GameLogic/Scripts.h"

class SidesInfoStub {
public:
    ScriptList* m_scripts = nullptr;
    ScriptList* getScriptList() const { return m_scripts; }
    void setScriptList(ScriptList* s) { m_scripts = s; }
};

class SidesListStub {
public:
    Int m_numSides = 0;
    SidesInfoStub m_sides[MAX_PLAYER_COUNT];
    Int getNumSides() const { return m_numSides; }
    SidesInfoStub* getSideInfo(Int i) { return &m_sides[i]; }
};

extern SidesListStub* TheSidesList;
