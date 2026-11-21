#pragma once
#include <algorithm>
#include <cstring>
#include <vector>

class File {
public:
    enum Access {
        READ = 1,
        BINARY = 2
    };
    enum seekMode {
        START = 0,
        CURRENT = 1,
        END = 2
    };

    virtual ~File() = default;
    virtual Int size() const = 0;
    virtual Int read(void* buf, Int bytes) = 0;
    virtual Int seek(Int offset, seekMode mode) = 0;
    virtual char* readEntireAndClose() = 0;
    virtual void close() = 0;
};

class MemFile : public File {
    std::vector<uint8_t> m_bytes;
    Int m_pos = 0;
public:
    explicit MemFile(std::vector<uint8_t> bytes) : m_bytes(std::move(bytes)) {}

    Int size() const override { return static_cast<Int>(m_bytes.size()); }

    Int read(void* buf, Int bytes) override {
        if (!buf || bytes <= 0 || m_pos < 0 || static_cast<size_t>(m_pos) >= m_bytes.size()) {
            return 0;
        }
        size_t avail = m_bytes.size() - static_cast<size_t>(m_pos);
        size_t take = std::min(avail, static_cast<size_t>(bytes));
        std::memcpy(buf, m_bytes.data() + m_pos, take);
        m_pos += static_cast<Int>(take);
        return static_cast<Int>(take);
    }

    Int seek(Int offset, seekMode mode) override {
        Int target = 0;
        if (mode == START) target = offset;
        else if (mode == CURRENT) target = m_pos + offset;
        else target = size() + offset;
        if (target < 0 || target > size()) return -1;
        m_pos = target;
        return m_pos;
    }

    // Matches DataChunk.cpp line 46: `delete[] m_buffer;`
    char* readEntireAndClose() override {
        size_t sz = m_bytes.size();
        char* out = new char[sz > 0 ? sz : 1];
        if (sz > 0) {
            std::memcpy(out, m_bytes.data(), sz);
        }
        delete this;
        return out;
    }

    void close() override {
        delete this;
    }
};
