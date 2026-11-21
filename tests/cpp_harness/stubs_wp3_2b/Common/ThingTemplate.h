#pragma once
class AsciiString;
class ThingTemplate {
    AsciiString m_name;
public:
    ThingTemplate() = default;
    explicit ThingTemplate(AsciiString name) : m_name(name) {}
    const AsciiString& getName() const { return m_name; }
};
