#pragma once

enum
{
    BaseFps = 30,
    LOGICFRAMES_PER_SECOND = 30,
    MSEC_PER_SECOND = 1000
};
const Real LOGICFRAMES_PER_MSEC_REAL = (((Real)LOGICFRAMES_PER_SECOND) / ((Real)MSEC_PER_SECOND));
const Real MSEC_PER_LOGICFRAME_REAL = (((Real)MSEC_PER_SECOND) / ((Real)LOGICFRAMES_PER_SECOND));
const Real LOGICFRAMES_PER_SECONDS_REAL = (Real)LOGICFRAMES_PER_SECOND;
const Real SECONDS_PER_LOGICFRAME_REAL = 1.0f / LOGICFRAMES_PER_SECONDS_REAL;

enum GameDifficulty CPP_11(: Int)
{
    DIFFICULTY_EASY = 0,
    DIFFICULTY_NORMAL = 1,
    DIFFICULTY_HARD = 2,
    DIFFICULTY_COUNT
};

enum PlayerType CPP_11(: Int)
{
    PLAYER_HUMAN = 0,
    PLAYER_COMPUTER = 1,
    PLAYERTYPE_COUNT
};

inline Real ConvertDurationFromMsecsToFrames(Real msec) {
    return (msec * LOGICFRAMES_PER_MSEC_REAL);
}
inline Real ConvertVelocityInSecsToFrames(Real distPerMsec) {
    return (distPerMsec * SECONDS_PER_LOGICFRAME_REAL);
}
inline Real ConvertAccelerationInSecsToFrames(Real distPerSec2) {
    const Real SEC_PER_LOGICFRAME_SQR = (SECONDS_PER_LOGICFRAME_REAL * SECONDS_PER_LOGICFRAME_REAL);
    return (distPerSec2 * SEC_PER_LOGICFRAME_SQR);
}
inline Real ConvertAngularVelocityInDegreesPerSecToRadsPerFrame(Real degPerSec) {
    const Real RADS_PER_DEGREE = PI / 180.0f;
    return (degPerSec * (SECONDS_PER_LOGICFRAME_REAL * RADS_PER_DEGREE));
}

extern const char* const TheVeterancyNames[];