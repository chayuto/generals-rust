#pragma once
#include <map>
#include <string>
#include <vector>
#include "Common/file.h"

class FileSystem {
public:
    std::string nextFileContent;
    bool nextFileExists = false;
    std::map<std::string, std::vector<uint8_t>> binaryFiles;

    bool doesFileExist(const char* name) const {
        if (name && binaryFiles.find(name) != binaryFiles.end()) return true;
        return nextFileExists;
    }
    void getFileListInDirectory(const AsciiString&, const char*, FilenameList&, Bool) {}
    File* openFile(const char* name, Int) {
        if (name) {
            auto it = binaryFiles.find(name);
            if (it != binaryFiles.end()) {
                return new MemFile(it->second);
            }
        }
        if (!nextFileExists) return nullptr;
        return new MemFile(nextFileContent);
    }
};

extern FileSystem* TheFileSystem;
