#pragma once

class GameLogic {
public:
    UnsignedInt m_frame = 1;
    UnsignedInt getFrame() const { return m_frame; }
};

extern GameLogic* TheGameLogic;
