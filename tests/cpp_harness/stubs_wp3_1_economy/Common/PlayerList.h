#pragma once
#include "Common/Player.h"

class PlayerList {
public:
    Player m_players[MAX_PLAYER_COUNT];

    Player* getNthPlayer(Int idx) {
        if (idx < 0 || idx >= MAX_PLAYER_COUNT) return nullptr;
        return &m_players[idx];
    }
};

extern PlayerList* ThePlayerList;
