#pragma once
#include <cstring>
#include <vector>

class Xfer {
public:
    virtual ~Xfer() = default;
    virtual void xferImplementation(void* data, Int dataSize) = 0;

    void xferVersion(XferVersion* versionData, XferVersion) {
        xferImplementation(versionData, sizeof(XferVersion));
    }
    void xferBool(Bool* boolData) {
        xferImplementation(boolData, sizeof(Bool));
    }
    void xferReal(Real* realData) {
        xferImplementation(realData, sizeof(Real));
    }
    void xferUser(void* data, Int dataSize) {
        xferImplementation(data, dataSize);
    }
};

class HarnessXferCRC : public Xfer {
public:
    UnsignedInt m_crc = 0;

    void addCRC(UnsignedInt val) {
        m_crc = (m_crc << 1) + val + ((m_crc >> 31) & 0x01u);
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
            m_crc = (m_crc << 1) + val + ((m_crc >> 31) & 0x01u);
            [[fallthrough]];
        default:
            break;
        }
    }

    UnsignedInt getCRC() const { return m_crc; }
};
