#pragma once
class Xfer;
class Snapshot {
public:
    virtual ~Snapshot() = default;
    virtual void crc(Xfer* xfer) = 0;
    virtual void xfer(Xfer* xfer) = 0;
    virtual void loadPostProcess() = 0;
};
