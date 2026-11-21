#pragma once

using LocomotorSurfaceTypeMask = Int;

enum LocomotorSurfaceType : Int {
    LOCOMOTORSURFACE_GROUND = 1 << 0,
    LOCOMOTORSURFACE_WATER  = 1 << 1,
    LOCOMOTORSURFACE_CLIFF  = 1 << 2,
    LOCOMOTORSURFACE_AIR    = 1 << 3,
    LOCOMOTORSURFACE_RUBBLE = 1 << 4
};
