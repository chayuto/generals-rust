#pragma once

class TerrainLogic {
public:
    Real groundHeightScaleX = 0.01f;
    Real groundHeightScaleY = 0.02f;
    Real groundHeightBase = 12.5f;

    Real getGroundHeight(Real x, Real y) const {
        return groundHeightBase + x * groundHeightScaleX - y * groundHeightScaleY;
    }
};

extern TerrainLogic* TheTerrainLogic;
