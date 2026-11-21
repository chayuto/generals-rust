#pragma once
#include "Common/AsciiString.h"

class InGameUI {
public:
    Int messageCount = 0;
    AsciiString lastMessage;

    void message(const char* msg) {
        ++messageCount;
        lastMessage.set(msg ? msg : "");
    }
    void resetStats() {
        messageCount = 0;
        lastMessage.clear();
    }
};

extern InGameUI* TheInGameUI;
