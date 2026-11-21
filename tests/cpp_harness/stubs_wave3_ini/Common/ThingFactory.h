#pragma once
class ThingTemplate;
class ThingFactory {
public:
    const ThingTemplate* findTemplate(const char*) const { return nullptr; }
    const ThingTemplate* findTemplate(const AsciiString&) const { return nullptr; }
};
extern ThingFactory* TheThingFactory;
