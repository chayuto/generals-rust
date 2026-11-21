#pragma once
#include <vector>

class Player;
class SpecialPowerTemplate;
class SpecialPowerModuleInterface {};

class Object {
public:
    Player* m_owner = nullptr;
    std::vector<UnsignedInt> m_supportedSpecialPowerIds;

    Player* getControllingPlayer() const { return m_owner; }
    SpecialPowerModuleInterface* getSpecialPowerModule(const SpecialPowerTemplate* tmpl) const;
};
