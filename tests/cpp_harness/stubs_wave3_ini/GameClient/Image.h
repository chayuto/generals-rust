#pragma once
class Image {};
class ImageCollection {
public:
    const Image* findImageByName(const char*) const { return nullptr; }
};
extern ImageCollection* TheMappedImageCollection;
