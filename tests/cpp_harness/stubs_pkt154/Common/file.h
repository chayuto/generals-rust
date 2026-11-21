#pragma once
#include <cstring>
#include <string>
#include <vector>

class File {
public:
    enum Access { READ = 1, BINARY = 2 };
    virtual ~File() = default;
    virtual Int size() const = 0;
    virtual char* readEntireAndClose() = 0;
    virtual void close() { delete this; }
};

class MemFile : public File {
    std::vector<uint8_t> m_content;
public:
    explicit MemFile(std::string s) : m_content(s.begin(), s.end()) {}
    explicit MemFile(std::vector<uint8_t> b) : m_content(std::move(b)) {}
    Int size() const override { return static_cast<Int>(m_content.size()); }
    char* readEntireAndClose() override {
        char* buf = new char[m_content.size() + 1];
        if (!m_content.empty()) {
            std::memcpy(buf, m_content.data(), m_content.size());
        }
        buf[m_content.size()] = '\0';
        delete this;
        return buf;
    }
};
