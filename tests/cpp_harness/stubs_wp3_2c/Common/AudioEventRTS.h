#pragma once
#include "Common/AsciiString.h"

class AudioEventRTS {
    AsciiString m_eventName;
    Int m_playerIndex = 0;
    Real m_volume = 1.0f;
public:
    AudioEventRTS() = default;
    explicit AudioEventRTS(const AsciiString& name) : m_eventName(name) {}
    void setEventName(const AsciiString& name) { m_eventName = name; }
    const AsciiString& getEventName() const { return m_eventName; }
    void setPlayerIndex(Int idx) { m_playerIndex = idx; }
    void setVolume(Real vol) { m_volume = vol; }
    Real getVolume() const { return m_volume; }
};

class DynamicAudioEventRTS : public AudioEventRTS {};
