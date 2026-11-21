#pragma once

class GlobalDataStub {
public:
    Bool m_useAlternateMouse = FALSE;
};
extern GlobalDataStub* TheGlobalData;

class GameLogic {
public:
    UnsignedInt m_frame = 0;
    UnsignedInt getFrame() const { return m_frame; }
};

extern GameLogic* TheGameLogic;
