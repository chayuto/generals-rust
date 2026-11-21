#pragma once
class DamageFX {};
class DamageFXStore {
public:
    const DamageFX* findDamageFX(const char*) const { return nullptr; }
};
extern DamageFXStore* TheDamageFXStore;
