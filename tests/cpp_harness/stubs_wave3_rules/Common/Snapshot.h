#pragma once
class Xfer;
class Snapshot {
public:
    virtual ~Snapshot() = default;
    virtual void crc(Xfer*) = 0;
    virtual void xfer(Xfer*) = 0;
    virtual void loadPostProcess() = 0;
};
