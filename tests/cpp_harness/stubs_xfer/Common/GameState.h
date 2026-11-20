#pragma once

class Snapshot;

class GameStateStub {
public:
    AsciiString realMapPathToPortableMapPath(const AsciiString& s) const { return s; }
    AsciiString portableMapPathToRealMapPath(const AsciiString& s) const { return s; }
    void addPostProcessSnapshot(Snapshot*) {}
};

extern GameStateStub* TheGameState;
