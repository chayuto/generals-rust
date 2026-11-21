#pragma once
#include "Common/AcademyStats.h"

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
    AcademyClassificationType m_class = ACT_NONE;
    AcademyClassificationType getAcademyClassificationType() const { return m_class; }
    UpgradeTemplate* friend_getNext() const { return nullptr; }
    const UpgradeMaskType& getUpgradeMask() const {
        static UpgradeMaskType empty;
        return empty;
    }
    AsciiString getUpgradeName() const { return AsciiString::TheEmptyString; }
};

class UpgradeCenterStub {
public:
    UpgradeTemplate* firstUpgradeTemplate() const { return nullptr; }
    const UpgradeTemplate* findUpgrade(const AsciiString&) const { return nullptr; }
};

extern UpgradeCenterStub* TheUpgradeCenter;
