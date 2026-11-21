#pragma once
#include "Common/AudioEventRTS.h"

class AudioManager {
public:
    void getInfoForAudioEvent(const void*) {}
};

extern AudioManager* TheAudio;
