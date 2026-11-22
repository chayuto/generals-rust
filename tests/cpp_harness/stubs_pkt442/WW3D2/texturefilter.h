#pragma once

class TextureFilterClass {
public:
    enum TextureFilterMode {
        TEXTURE_FILTER_NONE = 0,
        TEXTURE_FILTER_POINT,
        TEXTURE_FILTER_BILINEAR,
        TEXTURE_FILTER_TRILINEAR,
        TEXTURE_FILTER_ANISOTROPIC,
        TEXTURE_FILTER_COUNT
    };

    enum AnisotropicFilterMode {
        TEXTURE_FILTER_ANISOTROPIC_2X = 2,
        TEXTURE_FILTER_ANISOTROPIC_4X = 4,
        TEXTURE_FILTER_ANISOTROPIC_8X = 8,
        TEXTURE_FILTER_ANISOTROPIC_16X = 16
    };
};