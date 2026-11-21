#pragma once

class AudioEventRTS {
public:
    Int m_kind = 0; // 1 = withdraw, 2 = deposit
    Int m_playerIndex = 0;
    Real m_volume = 1.0f;

    AudioEventRTS() = default;
    explicit AudioEventRTS(Int kind, Real vol = 1.0f) : m_kind(kind), m_volume(vol) {}

    Real getVolume() const { return m_volume; }
    void setVolume(Real v) { m_volume = v; }
    void setPlayerIndex(Int idx) { m_playerIndex = idx; }
};

struct MiscAudio {
    AudioEventRTS m_moneyWithdrawSound{1, 1.0f};
    AudioEventRTS m_moneyDepositSound{2, 1.0f};
};
