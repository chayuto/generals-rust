#pragma once
#include <vector>
#include "Common/GameMemory.h"
#include "Common/AsciiString.h"

class BuildListInfo;

class AISideBuildList : public MemoryPoolObject {
    MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(AISideBuildList, "AISideBuildList")
public:
    AsciiString m_side;
    std::vector<BuildListInfo*> items;
    explicit AISideBuildList(AsciiString side = AsciiString::TheEmptyString) : m_side(side) {}
    void addInfo(BuildListInfo* info) { items.push_back(info); }
};
inline AISideBuildList::~AISideBuildList() = default;
