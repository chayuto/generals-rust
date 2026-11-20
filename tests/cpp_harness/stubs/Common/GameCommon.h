#pragma once
#include <cstdint>

enum VeterancyLevel : int32_t {
    LEVEL_REGULAR = 0,
    LEVEL_VETERAN = 1,
    LEVEL_ELITE = 2,
    LEVEL_HEROIC = 3,
    LEVEL_COUNT = 4,
    LEVEL_FIRST = LEVEL_REGULAR,
    LEVEL_LAST = LEVEL_HEROIC,
    LEVEL_INVALID = 4
};

enum Relationship : int32_t {
    ENEMIES = 0,
    NEUTRAL = 1,
    ALLIES = 2
};
