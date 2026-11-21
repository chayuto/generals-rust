#pragma once
#include "Common/Player.h"

class PlayerList {
    Player m_local;
public:
    void setLocalPlayerIndex(Int idx) { m_local.setPlayerIndex(idx); }
    Player* getLocalPlayer() { return &m_local; }
};

extern PlayerList* ThePlayerList;
