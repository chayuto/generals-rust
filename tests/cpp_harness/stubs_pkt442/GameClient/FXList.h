#pragma once
class FXList {};
class FXListStore {
public:
    const FXList* findFXList(const char*) const { return nullptr; }
};
extern FXListStore* TheFXListStore;
