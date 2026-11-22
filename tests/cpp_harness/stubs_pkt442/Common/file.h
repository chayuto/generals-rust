#pragma once
#include <algorithm>
#include <cstring>
#include <string>
#include <vector>

class File {
public:
    enum Access {
        READ = 1,
        BINARY = 0x0040,
    };
    virtual ~File() = default;
    virtual Int size() const = 0;
    virtual Int read(void* pData, Int numBytes) = 0;
    virtual char* readEntireAndClose() = 0;
    virtual void close() = 0;
};

class MemFile : public File {
    std::string m_content;
    size_t m_pos = 0;
public:
    explicit MemFile(std::string s) : m_content(std::move(s)) {}
    Int size() const override { return static_cast<Int>(m_content.size()); }
    Int read(void* pData, Int numBytes) override {
        if (!pData || numBytes <= 0 || m_pos >= m_content.size()) return 0;
        size_t avail = m_content.size() - m_pos;
        size_t take = std::min(avail, static_cast<size_t>(numBytes));
        std::memcpy(pData, m_content.data() + m_pos, take);
        m_pos += take;
        return static_cast<Int>(take);
    }
    char* readEntireAndClose() override {
        char* buf = new char[m_content.size() + 1];
        std::memcpy(buf, m_content.data(), m_content.size());
        buf[m_content.size()] = '\0';
        delete this;
        return buf;
    }
    void close() override { delete this; }
};

class PatternFile : public File {
    uint64_t m_total_bytes;
    uint64_t m_pos = 0;
    uint32_t m_seed;
public:
    PatternFile(uint64_t total_bytes, uint32_t seed)
        : m_total_bytes(total_bytes), m_seed(seed) {}
    Int size() const override {
        return m_total_bytes > 0x7fffffffULL ? 0x7fffffff : static_cast<Int>(m_total_bytes);
    }
    Int read(void* pData, Int numBytes) override {
        if (!pData || numBytes <= 0 || m_pos >= m_total_bytes) return 0;
        uint64_t avail = m_total_bytes - m_pos;
        size_t take = static_cast<size_t>(std::min<uint64_t>(avail, static_cast<uint64_t>(numBytes)));
        unsigned char* out = static_cast<unsigned char*>(pData);
        for (size_t i = 0; i < take; ++i) {
            uint64_t idx = m_pos + i;
            out[i] = static_cast<unsigned char>((idx * 131ULL + m_seed + (idx >> 16)) & 0xffu);
        }
        m_pos += take;
        return static_cast<Int>(take);
    }
    char* readEntireAndClose() override {
        delete this;
        return nullptr;
    }
    void close() override { delete this; }
};