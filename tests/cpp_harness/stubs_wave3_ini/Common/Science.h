#pragma once
class ScienceStore {
public:
    ScienceType friend_lookupScience(const char*) const { return SCIENCE_INVALID; }
};
extern ScienceStore* TheScienceStore;
