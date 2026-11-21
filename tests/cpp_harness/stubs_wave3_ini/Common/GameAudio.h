#pragma once
class AudioEventRTS {
public:
    void setEventName(const AsciiString&) {}
};
class DynamicAudioEventRTS {
public:
    void setEventName(const AsciiString&) {}
};
class AudioManager {
public:
    void getInfoForAudioEvent(const void*) {}
};
extern AudioManager* TheAudio;
