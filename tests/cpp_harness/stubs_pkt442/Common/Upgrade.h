#pragma once
class UpgradeTemplate {};
class UpgradeCenter {
public:
    const UpgradeTemplate* findUpgrade(const char*) const { return nullptr; }
};
extern UpgradeCenter* TheUpgradeCenter;
