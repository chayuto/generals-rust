#pragma once
#include "Common/AsciiString.h"

class AudioEventRTS {
    AsciiString m_eventName;
public:
    AudioEventRTS() = default;
    explicit AudioEventRTS(const AsciiString& name) : m_eventName(name) {}
    void setEventName(const AsciiString& name) { m_eventName = name; }
    const AsciiString& getEventName() const { return m_eventName; }
};

class DynamicAudioEventRTS : public AudioEventRTS {};
