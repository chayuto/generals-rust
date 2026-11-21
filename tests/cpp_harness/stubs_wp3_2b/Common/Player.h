#pragma once
#include <algorithm>
#include <vector>

class Money {
    UnsignedInt m_money = 0;
public:
    void setAmount(UnsignedInt amount) { m_money = amount; }
    UnsignedInt countMoney() const { return m_money; }
};

class Player {
    Money m_money;
    Bool m_buildsInstantly = FALSE;
    std::vector<ScienceType> m_sciences;
public:
    void setMoneyAmount(UnsignedInt amount) { m_money.setAmount(amount); }
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
};
