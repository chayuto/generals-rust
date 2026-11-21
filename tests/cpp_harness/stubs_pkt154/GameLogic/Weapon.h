#pragma once
class WeaponTemplate {};
class WeaponStore {
public:
    const WeaponTemplate* findWeaponTemplate(const char*) const { return nullptr; }
};
extern WeaponStore* TheWeaponStore;
