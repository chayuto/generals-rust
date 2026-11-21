#pragma once
#include <map>
#include <string>
#include "Common/ThingTemplate.h"

class ThingFactory {
    std::map<std::string, ThingTemplate> m_templates;
public:
    const ThingTemplate* registerTemplate(
        const char* name,
        const char* displayAscii = nullptr,
        Bool isStructure = FALSE
    ) {
        if (!name || !name[0]) return nullptr;
        auto it = m_templates.find(name);
        if (it == m_templates.end()) {
            const char* disp = displayAscii ? displayAscii : name;
            it = m_templates.emplace(
                std::string(name),
                ThingTemplate(AsciiString(name), disp, isStructure)
            ).first;
        }
        return &it->second;
    }
    const ThingTemplate* findTemplate(const char* name) const {
        if (!name || !name[0]) return nullptr;
        auto it = m_templates.find(name);
        return it == m_templates.end() ? nullptr : &it->second;
    }
    const ThingTemplate* findTemplate(const AsciiString& name) const {
        return findTemplate(name.str());
    }
};

extern ThingFactory* TheThingFactory;
