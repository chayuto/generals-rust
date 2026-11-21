#pragma once
#include "Common/AudioSettings.h"
#include "Common/MiscAudio.h"

class AudioManager {
public:
    AudioSettings m_settings;
    MiscAudio m_misc;
    uint32_t m_traceCrc = 0;
    uint32_t m_eventCount = 0;

    const AudioSettings* getAudioSettings() const { return &m_settings; }
    const MiscAudio* getMiscAudio() const { return &m_misc; }
    void addAudioEvent(const AudioEventRTS* ev) {
        if (!ev) return;
        ++m_eventCount;
        auto rot = [](uint32_t c, uint32_t w) { return ((c << 1) | (c >> 31)) + w; };
        m_traceCrc = rot(m_traceCrc, static_cast<uint32_t>(ev->m_kind));
        m_traceCrc = rot(m_traceCrc, static_cast<uint32_t>(ev->m_playerIndex));
    }
};

extern AudioManager* TheAudio;
