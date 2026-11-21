#pragma once
#include "GameLogic/Scripts.h"

struct TScriptListReadInfo {
    Int numLists;
    ScriptList* readLists[MAX_PLAYER_COUNT];
};

class ScriptEngineStub {
public:
    ConditionTemplate m_condTemplates[Condition::NUM_ITEMS];
    ActionTemplate m_actTemplates[ScriptAction::NUM_ITEMS];
    AsciiString m_lastUISignal;

    void signalUIInteract(const char* name) { m_lastUISignal = name; }
    const ConditionTemplate* getConditionTemplate(Int idx) const {
        if (idx < 0 || idx >= Condition::NUM_ITEMS) return nullptr;
        return &m_condTemplates[idx];
    }
    const ActionTemplate* getActionTemplate(Int idx) const {
        if (idx < 0 || idx >= ScriptAction::NUM_ITEMS) return nullptr;
        return &m_actTemplates[idx];
    }
};

extern ScriptEngineStub* TheScriptEngine;
