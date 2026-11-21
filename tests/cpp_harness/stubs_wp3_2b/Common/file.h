#pragma once
#include <cstring>
#include <string>

class File {
public:
    enum Access { READ = 1 };
    virtual ~File() = default;
    virtual Int size() const = 0;
    virtual char* readEntireAndClose() = 0;
};

class MemFile : public File {
    std::string m_content;
public:
    explicit MemFile(std::string s) : m_content(std::move(s)) {}
    Int size() const override { return static_cast<Int>(m_content.size()); }
    char* readEntireAndClose() override {
        char* buf = new char[m_content.size() + 1];
        std::memcpy(buf, m_content.data(), m_content.size());
        buf[m_content.size()] = '\0';
        delete this;
        return buf;
    }
};
