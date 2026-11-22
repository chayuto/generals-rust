#pragma once

using PlayerMaskType = UnsignedShort;

enum TimeOfDay CPP_11(: Int)
{
    TIME_OF_DAY_INVALID = 0,
    TIME_OF_DAY_MORNING,
    TIME_OF_DAY_AFTERNOON,
    TIME_OF_DAY_EVENING,
    TIME_OF_DAY_NIGHT,
    TIME_OF_DAY_COUNT,
    TIME_OF_DAY_FIRST = TIME_OF_DAY_MORNING
};

extern const char* const TimeOfDayNames[];

enum Weather CPP_11(: Int)
{
    WEATHER_NORMAL = 0,
    WEATHER_SNOWY = 1,
    WEATHER_COUNT
};

extern const char* const WeatherNames[];
