#pragma once

enum BodyDamageType CPP_11(: Int)
{
    BODY_PRISTINE = 0,
    BODY_DAMAGED,
    BODY_REALLYDAMAGED,
    BODY_RUBBLE,
    BODYDAMAGETYPE_COUNT
};

#ifdef DEFINE_BODYDAMAGETYPE_NAMES
static const char* const TheBodyDamageTypeNames[] =
{
    "PRISTINE",
    "DAMAGED",
    "REALLYDAMAGED",
    "RUBBLE",
    nullptr
};
static_assert(ARRAY_SIZE(TheBodyDamageTypeNames) == BODYDAMAGETYPE_COUNT + 1, "Incorrect array size");
#endif