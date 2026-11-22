#pragma once

class Version {
public:
    UnsignedInt versionNumber = 0;
    UnsignedInt getVersionNumber() const { return versionNumber; }
};

extern Version* TheVersion;