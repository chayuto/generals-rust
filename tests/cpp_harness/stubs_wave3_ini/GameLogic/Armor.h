#pragma once
class ArmorTemplate {};
class ArmorStore {
public:
    const ArmorTemplate* findArmorTemplate(const char*) const { return nullptr; }
};
extern ArmorStore* TheArmorStore;
