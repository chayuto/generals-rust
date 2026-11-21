#pragma once
class Anim2DTemplate {};
class Anim2DCollection {
public:
    Anim2DTemplate* findTemplate(const AsciiString&) const { return nullptr; }
};
extern Anim2DCollection* TheAnim2DCollection;
