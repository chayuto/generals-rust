#pragma once
#include <string>
#include <vector>
#include "Common/file.h"

class FileSystem {
public:
    std::string nextFileContent;
    bool nextFileExists = false;

    // Executable / .scb streams used by GlobalData::generateExeCRC()
    bool binaryMode = false;
    bool hasExe = false;
    bool usePatternExe = false;
    uint64_t patternExeBytes = 0;
    uint32_t patternExeSeed = 0;
    std::vector<UnsignedByte> exeBytes;

    bool hasSkirmishScb = false;
    std::vector<UnsignedByte> skirmishScbBytes;

    bool hasMpScb = false;
    std::vector<UnsignedByte> mpScbBytes;

    bool doesFileExist(const char*) const { return nextFileExists; }
    void getFileListInDirectory(const AsciiString&, const char*, FilenameList&, Bool) {}
    File* openFile(const char* filename, Int access) {
        if ((access & File::BINARY) != 0 || binaryMode) {
            if (filename && std::strcmp(filename, "Data\\Scripts\\SkirmishScripts.scb") == 0) {
                if (!hasSkirmishScb) return nullptr;
                return new MemFile(std::string(reinterpret_cast<const char*>(skirmishScbBytes.data()), skirmishScbBytes.size()));
            }
            if (filename && std::strcmp(filename, "Data\\Scripts\\MultiplayerScripts.scb") == 0) {
                if (!hasMpScb) return nullptr;
                return new MemFile(std::string(reinterpret_cast<const char*>(mpScbBytes.data()), mpScbBytes.size()));
            }
            if (usePatternExe) {
                return new PatternFile(patternExeBytes, patternExeSeed);
            }
            if (!hasExe) return nullptr;
            return new MemFile(std::string(reinterpret_cast<const char*>(exeBytes.data()), exeBytes.size()));
        }
        if (!nextFileExists) return nullptr;
        return new MemFile(nextFileContent);
    }
};

extern FileSystem* TheFileSystem;