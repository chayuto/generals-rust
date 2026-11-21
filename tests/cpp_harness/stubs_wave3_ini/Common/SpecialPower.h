#pragma once
class SpecialPowerTemplate {};
class SpecialPowerStore {
public:
    const SpecialPowerTemplate* findSpecialPowerTemplate(const AsciiString&) const { return nullptr; }
};
extern SpecialPowerStore* TheSpecialPowerStore;
