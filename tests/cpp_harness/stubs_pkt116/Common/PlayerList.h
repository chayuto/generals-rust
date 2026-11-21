#pragma once
#include "Common/Player.h"

class PlayerList {
public:
    Player m_localPlayer;
    Player m_neutralPlayer;

    Player* getLocalPlayer() { return &m_localPlayer; }
    Player* getNeutralPlayer() { return &m_neutralPlayer; }
};

extern PlayerList* ThePlayerList;
