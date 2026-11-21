#pragma once
#include <algorithm>
#include <map>
#include <vector>
#include "Common/Money.h"
#include "Common/ThingTemplate.h"

class AcademyStatsStub {
public:
    void recordIncome() {}
};

class Player {
    Money m_money;
    Bool m_buildsInstantly = FALSE;
    std::vector<ScienceType> m_sciences;
    std::map<const ThingTemplate*, Int> m_ownedCounts;
    AcademyStatsStub m_academyStats;
public:
    void setMoneyAmount(UnsignedInt amount) { m_money.setStartingCash(amount); }
    Money* getMoney() { return &m_money; }
    const Money* getMoney() const { return &m_money; }
    void setBuildsInstantly(Bool b) { m_buildsInstantly = b; }
    Bool buildsInstantly() const { return m_buildsInstantly; }
    void clearSciences() { m_sciences.clear(); }
    void grantScience(ScienceType st) {
        if (std::find(m_sciences.begin(), m_sciences.end(), st) == m_sciences.end()) {
            m_sciences.push_back(st);
        }
    }
    Bool hasScience(ScienceType st) const {
        return std::find(m_sciences.begin(), m_sciences.end(), st) != m_sciences.end();
    }
    void clearOwnedObjects() { m_ownedCounts.clear(); }
    void setOwnedCount(const ThingTemplate* tmpl, Int count) {
        if (tmpl) m_ownedCounts[tmpl] = count;
    }
    void countObjectsByThingTemplate(
        Int numTmpls,
        const ThingTemplate* const* tmpls,
        Bool,
        Int* counts
    ) const {
        for (Int i = 0; i < numTmpls; ++i) {
            auto it = m_ownedCounts.find(tmpls[i]);
            counts[i] = (it == m_ownedCounts.end()) ? 0 : it->second;
        }
    }
    AcademyStatsStub* getAcademyStats() { return &m_academyStats; }
};
