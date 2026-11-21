#pragma once

class Snapshot;

class GameState {
public:
    AsciiString realMapPathToPortableMapPath(const AsciiString& s) const { return s; }
    AsciiString portableMapPathToRealMapPath(const AsciiString& s) const { return s; }
    void addPostProcessSnapshot(Snapshot*) {}
};

extern GameState* TheGameState;
