#pragma once

template <size_t NUMBITS, typename TAG>
class BitFlags;

struct KindOfMaskType {
    static AsciiString getNameFromSingleBit(KindOfType) { return AsciiString::TheEmptyString; }
    static Int getSingleBitFromName(const char*) { return -1; }
};
