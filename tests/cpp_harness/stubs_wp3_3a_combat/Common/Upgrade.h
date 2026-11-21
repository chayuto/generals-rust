#pragma once

class UpgradeMaskType {
public:
    uint64_t lo = 0;
    uint64_t hi = 0;
    void clear() { lo = hi = 0; }
    void set(const UpgradeMaskType& o) { lo |= o.lo; hi |= o.hi; }
    bool testForAll(const UpgradeMaskType& o) const {
        return (lo & o.lo) == o.lo && (hi & o.hi) == o.hi;
    }
};

class UpgradeTemplate {
public:
    UpgradeTemplate* friend_getNext() const { return nullptr; }
    const UpgradeMaskType& getUpgradeMask() const {
        static UpgradeMaskType empty;
        return empty;
    }
    AsciiString getUpgradeName() const { return AsciiString::TheEmptyString; }
};

class UpgradeCenter {
public:
    UpgradeTemplate* firstUpgradeTemplate() const { return nullptr; }
    const UpgradeTemplate* findUpgrade(const char*) const { return nullptr; }
    const UpgradeTemplate* findUpgrade(const AsciiString&) const { return nullptr; }
};

extern UpgradeCenter* TheUpgradeCenter;
