#pragma once
class ThingTemplate {
public:
    AsciiString getName() const { return AsciiString::TheEmptyString; }
};
class ThingFactory {
public:
    const ThingTemplate* findTemplate(const AsciiString&) const { return nullptr; }
};
extern ThingFactory* TheThingFactory;
