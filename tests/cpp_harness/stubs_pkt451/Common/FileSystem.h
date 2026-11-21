#pragma once
#include <map>
#include <string>
#include <vector>
#include "Common/file.h"

class FileSystem {
    std::map<std::string, std::vector<uint8_t>> m_files;
public:
    void setFile(const char* name, std::vector<uint8_t> data) {
        m_files[name] = std::move(data);
    }
    void clearFiles() {
        m_files.clear();
    }
    File* openFile(const char* name, Int) {
        if (!name) return nullptr;
        auto it = m_files.find(name);
        if (it == m_files.end()) return nullptr;
        return new MemFile(it->second);
    }
};

extern FileSystem* TheFileSystem;
