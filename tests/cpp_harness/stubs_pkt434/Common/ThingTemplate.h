#pragma once
enum BuildableStatus : Int {
    BSTATUS_YES = 0,
    BSTATUS_IGNORE_PREREQUISITES = 1,
    BSTATUS_NO = 2,
    BSTATUS_ONLY_BY_AI = 3,
    BSTATUS_NUM_TYPES = 4
};
#ifdef DEFINE_BUILDABLE_STATUS_NAMES
static const char* const BuildableStatusNames[] = {
    "Yes",
    "Ignore_Prerequisites",
    "No",
    "Only_By_AI",
    nullptr
};
#endif
class ThingTemplate;
