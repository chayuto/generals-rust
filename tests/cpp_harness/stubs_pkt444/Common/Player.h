#pragma once

class Player {
    Int m_index = 0;
public:
    explicit Player(Int idx = 0) : m_index(idx) {}
    void setPlayerIndex(Int idx) { m_index = idx; }
    Int getPlayerIndex() const { return m_index; }
};
