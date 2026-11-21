#pragma once
class GameLogic {
public:
    UnsignedInt getFrame() const { return 1; }
    Bool isInGameLogicUpdate() const { return FALSE; }
};
extern GameLogic* TheGameLogic;
