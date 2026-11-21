#pragma once
#include "Common/AudioEventRTS.h"
#include "Common/AudioSettings.h"
#include "Common/MiscAudio.h"

class AudioManager {
    AudioSettings m_settings;
    MiscAudio m_misc;
public:
    void getInfoForAudioEvent(const void*) {}
    const AudioSettings* getAudioSettings() const { return &m_settings; }
    const MiscAudio* getMiscAudio() const { return &m_misc; }
    void addAudioEvent(const AudioEventRTS*) {}
};

extern AudioManager* TheAudio;
