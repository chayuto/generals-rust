#pragma once

class ScienceStore {
public:
    ScienceType friend_lookupScience(const char*) const { return SCIENCE_INVALID; }
    AsciiString getInternalNameForScience(ScienceType) const { return AsciiString("SCI_NONE"); }
    ScienceType getScienceFromInternalName(const AsciiString&) const { return SCIENCE_INVALID; }
};

extern ScienceStore* TheScienceStore;
