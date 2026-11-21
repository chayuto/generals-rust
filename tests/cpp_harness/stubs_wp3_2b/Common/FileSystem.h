#pragma once
#include "Common/file.h"

class FileSystem {
public:
    std::string nextFileContent;
    bool nextFileExists = false;

    bool doesFileExist(const char*) const { return nextFileExists; }
    void getFileListInDirectory(const AsciiString&, const char*, FilenameList&, Bool) {}
    File* openFile(const char*, Int) {
        if (!nextFileExists) return nullptr;
        return new MemFile(nextFileContent);
    }
};

extern FileSystem* TheFileSystem;
