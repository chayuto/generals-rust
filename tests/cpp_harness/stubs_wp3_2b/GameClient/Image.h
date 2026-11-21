#pragma once
#include <cstring>

class Image {
public:
    const char* name = "";
    explicit Image(const char* n = "") : name(n) {}
};

class ImageCollection {
    Image m_btnFound{"BTN_FOUND"};
public:
    const Image* findImageByName(const AsciiString& name) const {
        if (std::strcmp(name.str(), "BTN_FOUND") == 0) {
            return &m_btnFound;
        }
        return nullptr;
    }
};

extern ImageCollection* TheMappedImageCollection;
