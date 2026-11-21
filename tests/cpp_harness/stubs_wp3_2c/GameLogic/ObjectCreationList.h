#pragma once
class ObjectCreationList {};
class ObjectCreationListStore {
public:
    const ObjectCreationList* findObjectCreationList(const char*) const { return nullptr; }
};
extern ObjectCreationListStore* TheObjectCreationListStore;
