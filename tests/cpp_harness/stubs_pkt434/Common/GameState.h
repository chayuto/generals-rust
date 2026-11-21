#pragma once

class Snapshot;

enum SaveCode : Int {
    SC_OK = 0,
    SC_INVALID_DATA = 1
};

class GameState {
public:
    AsciiString realMapPathToPortableMapPath(const AsciiString& s) const { return s; }
    AsciiString portableMapPathToRealMapPath(const AsciiString& s) const { return s; }
    void addPostProcessSnapshot(Snapshot*) {}
};

extern GameState* TheGameState;
