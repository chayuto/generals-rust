#pragma once
#include "Common/GameMemory.h"
#include "Common/Snapshot.h"
#include "Common/Xfer.h"

class TunnelTracker : public MemoryPoolObject, public Snapshot {
public:
    Int m_containCount = 0;
    UnsignedInt m_tag = 0;

    UnsignedInt getContainCount() const {
        return static_cast<UnsignedInt>(m_containCount > 0 ? m_containCount : 0);
    }

    void crc(Xfer*) override {}

    void xfer(Xfer* xfer) override {
        XferVersion currentVersion = 1;
        XferVersion version = currentVersion;
        xfer->xferVersion(&version, currentVersion);
        xfer->xferInt(&m_containCount);
        xfer->xferUnsignedInt(&m_tag);
    }

    void loadPostProcess() override {}
};
