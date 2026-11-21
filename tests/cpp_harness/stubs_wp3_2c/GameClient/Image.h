#pragma once
#include <cstring>

class Image {
public:
    const char* name = "";
    explicit Image(const char* n = "") : name(n) {}
};

class ImageCollection {
    Image m_btnFound{"BTN_FOUND"};
    Image m_hw{"IMG_HW_1"};
    Image m_fw{"IMG_FW_1"};
    Image m_side{"IMG_SIDE_1"};
    Image m_gen{"IMG_GEN_1"};
public:
    const Image* findImageByName(const AsciiString& name) const {
        if (std::strcmp(name.str(), "BTN_FOUND") == 0) return &m_btnFound;
        if (std::strcmp(name.str(), "IMG_HW_1") == 0) return &m_hw;
        if (std::strcmp(name.str(), "IMG_FW_1") == 0) return &m_fw;
        if (std::strcmp(name.str(), "IMG_SIDE_1") == 0) return &m_side;
        if (std::strcmp(name.str(), "IMG_GEN_1") == 0) return &m_gen;
        return nullptr;
    }
};

extern ImageCollection* TheMappedImageCollection;
