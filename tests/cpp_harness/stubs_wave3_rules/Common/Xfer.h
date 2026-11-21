#pragma once
class Xfer {
public:
    virtual ~Xfer() = default;
    virtual Int getXferMode() const { return 1; }
    virtual void xferVersion(XferVersion* v, XferVersion) { xferImplementation(v, 1); }
    virtual void xferBool(Bool* b) { xferImplementation(b, 1); }
    virtual void xferReal(Real* v) { xferImplementation(v, 4); }
    virtual void xferObjectID(ObjectID* v) { xferImplementation(v, 4); }
    virtual void xferCoord3D(Coord3D* v) { xferImplementation(v, 12); }
    virtual void xferAsciiString(AsciiString*) {}
    virtual void xferSnapshot(Snapshot*) {}
    virtual void xferUser(void* data, Int dataSize) { xferImplementation(data, dataSize); }
    virtual void xferImplementation(void* data, Int dataSize) = 0;
};
constexpr Int XFER_LOAD = 2;
