#pragma once

class AcademyStatsStub {
public:
    uint32_t m_incomeCalls = 0;
    void recordIncome() { ++m_incomeCalls; }
};

class Player {
public:
    Int m_playerIndex = 0;
    AcademyStatsStub m_academyStats;
    uint32_t m_brownoutCallbacks = 0;
    uint32_t m_brownoutTraceCrc = 0;
    Bool m_lastBrownout = FALSE;

    Int getPlayerIndex() const { return m_playerIndex; }
    AcademyStatsStub* getAcademyStats() { return &m_academyStats; }
    void onPowerBrownOutChange(Bool isBrownout) {
        ++m_brownoutCallbacks;
        m_lastBrownout = isBrownout;
        m_brownoutTraceCrc = ((m_brownoutTraceCrc << 1) | (m_brownoutTraceCrc >> 31)) + static_cast<uint32_t>(isBrownout);
    }
};
