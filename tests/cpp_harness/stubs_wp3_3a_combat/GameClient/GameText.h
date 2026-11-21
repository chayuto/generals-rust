#pragma once
class GameTextInterface {
public:
    UnicodeString fetch(const char*) const { return UnicodeString::TheEmptyString; }
};
extern GameTextInterface* TheGameText;
