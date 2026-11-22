#pragma once

typedef enum _TerrainLOD CPP_11(: Int)
{
    TERRAIN_LOD_INVALID = 0,
    TERRAIN_LOD_MIN,
    TERRAIN_LOD_NO_CLOUDS,
    TERRAIN_LOD_NO_WATER,
    TERRAIN_LOD_MAX,
    TERRAIN_LOD_AUTOMATIC,
    TERRAIN_LOD_DISABLE,
    TERRAIN_LOD_NUM_TYPES
} TerrainLOD;

#ifdef DEFINE_TERRAIN_LOD_NAMES
static const char* const TerrainLODNames[] =
{
    "NONE",
    "MIN",
    "NO_CLOUDS",
    "NO_WATER",
    "MAX",
    "AUTOMATIC",
    "DISABLE",
    nullptr
};
static_assert(ARRAY_SIZE(TerrainLODNames) == TERRAIN_LOD_NUM_TYPES + 1, "Incorrect array size");
#endif