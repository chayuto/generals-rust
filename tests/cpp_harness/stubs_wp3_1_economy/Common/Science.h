#pragma once
#include <vector>

using ScienceVec = std::vector<ScienceType>;

class ScienceStoreStub {
public:
    AsciiString getInternalNameForScience(ScienceType) const { return AsciiString("SCI_NONE"); }
    ScienceType getScienceFromInternalName(const AsciiString&) const { return SCIENCE_INVALID; }
};

extern ScienceStoreStub* TheScienceStore;
