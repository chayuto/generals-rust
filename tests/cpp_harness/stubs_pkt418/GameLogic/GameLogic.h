#pragma once

class GameLogic {
public:
    UnsignedInt m_frame = 0;
    UnsignedInt getFrame() const { return m_frame; }
};

extern GameLogic* TheGameLogic;
