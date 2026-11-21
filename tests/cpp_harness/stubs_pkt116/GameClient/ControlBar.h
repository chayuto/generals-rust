#pragma once
#include "Common/ThingTemplate.h"

constexpr Int MAX_COMMANDS_PER_SET = 18;

class CommandButton {
public:
    const ThingTemplate* m_thing = nullptr;
    const ThingTemplate* getThingTemplate() const { return m_thing; }
};

class CommandSet {
public:
    const CommandButton* m_buttons[MAX_COMMANDS_PER_SET]{};
    const CommandButton* getCommandButton(Int i) const {
        if (i < 0 || i >= MAX_COMMANDS_PER_SET) return nullptr;
        return m_buttons[i];
    }
};

class ControlBar {
public:
    CommandSet m_commandSet;
    const CommandSet* findCommandSet(const AsciiString&) const { return &m_commandSet; }
};

extern ControlBar* TheControlBar;
