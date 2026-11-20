#pragma once
#include "PreRTS.h"
#include <vector>
#include <cstring>

class Xfer {
public:
    virtual ~Xfer() = default;
    virtual void xferImplementation(void* data, Int dataSize) = 0;

    void xferVersion(XferVersion* versionData, XferVersion currentVersion) {
        (void)currentVersion;
        xferImplementation(versionData, sizeof(XferVersion));
    }
    void xferInt(Int* intData) {
        xferImplementation(intData, sizeof(Int));
    }
    void xferUnsignedInt(UnsignedInt* uintData) {
        xferImplementation(uintData, sizeof(UnsignedInt));
    }
    void xferReal(Real* realData) {
        xferImplementation(realData, sizeof(Real));
    }
    void xferBool(Bool* boolData) {
        xferImplementation(boolData, sizeof(Bool));
    }
    void xferObjectID(ObjectID* objectIDData) {
        xferImplementation(objectIDData, sizeof(ObjectID));
    }
    void xferUser(void* data, Int dataSize) {
        xferImplementation(data, dataSize);
    }
};

// Exact XferCRC implementation from Core/GameEngine/Source/Common/System/XferCRC.cpp
class HarnessXferCRC : public Xfer {
public:
    UnsignedInt m_crc = 0;

    void addCRC(UnsignedInt val) {
        m_crc = (m_crc << 1) + val + ((m_crc >> 31) & 0x01);
    }

    void xferImplementation(void* data, Int dataSize) override {
        const UnsignedInt* uintPtr = static_cast<const UnsignedInt*>(data);
        dataSize *= (data != nullptr);
        int dataBytes = (dataSize / 4);
        for (Int i = 0; i < dataBytes; ++i) {
            UnsignedInt word = 0;
            std::memcpy(&word, uintPtr, sizeof(UnsignedInt));
            addCRC(word);
            ++uintPtr;
        }
        UnsignedInt val = 0;
        const unsigned char* c = reinterpret_cast<const unsigned char*>(uintPtr);
        switch (dataSize & 3) {
        case 3:
            val += (static_cast<UnsignedInt>(c[2]) << 16);
            [[fallthrough]];
        case 2:
            val += (static_cast<UnsignedInt>(c[1]) << 8);
            [[fallthrough]];
        case 1:
            val += static_cast<UnsignedInt>(c[0]);
            m_crc = (m_crc << 1) + val + ((m_crc >> 31) & 0x01);
            [[fallthrough]];
        default:
            break;
        }
    }

    UnsignedInt getCRC() const { return m_crc; }
};

class HarnessXferSave : public Xfer {
public:
    std::vector<uint8_t> bytes;
    void xferImplementation(void* data, Int dataSize) override {
        if (data && dataSize > 0) {
            const uint8_t* p = static_cast<const uint8_t*>(data);
            bytes.insert(bytes.end(), p, p + dataSize);
        }
    }
};

class HarnessXferLoad : public Xfer {
public:
    const uint8_t* buf = nullptr;
    Int len = 0;
    Int pos = 0;
    HarnessXferLoad(const uint8_t* b, Int l) : buf(b), len(l), pos(0) {}
    void xferImplementation(void* data, Int dataSize) override {
        if (data && dataSize > 0 && pos + dataSize <= len) {
            std::memcpy(data, buf + pos, static_cast<size_t>(dataSize));
            pos += dataSize;
        }
    }
};
