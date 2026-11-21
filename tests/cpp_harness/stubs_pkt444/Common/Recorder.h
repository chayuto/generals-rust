#pragma once

enum RecorderModeType : Int {
    RECORDERMODETYPE_NONE = 0,
    RECORDERMODETYPE_RECORD = 1,
    RECORDERMODETYPE_PLAYBACK = 2
};

class Recorder {
public:
    Bool isMultiplayer() const { return FALSE; }
    RecorderModeType getMode() const { return RECORDERMODETYPE_NONE; }
};

extern Recorder* TheRecorder;
