#pragma once

class INI;

class Money {
private:
    UnsignedInt m_money = 0;
public:
    void init() { m_money = 0; }
    UnsignedInt countMoney() const { return m_money; }
    void setStartingCash(UnsignedInt amount) { m_money = amount; }
    static void parseMoneyAmount(INI* ini, void* instance, void* store, const void* userData);
};