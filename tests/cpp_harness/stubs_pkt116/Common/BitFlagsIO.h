#pragma once

struct KindOfMaskType {
    static AsciiString getNameFromSingleBit(KindOfType) { return AsciiString::TheEmptyString; }
    static Int getSingleBitFromName(const char*) { return -1; }
};
