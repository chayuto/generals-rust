#pragma once

class GameLogic {
public:
    Bool isInSkirmishGame() const { return FALSE; }
};

extern GameLogic* TheGameLogic;
