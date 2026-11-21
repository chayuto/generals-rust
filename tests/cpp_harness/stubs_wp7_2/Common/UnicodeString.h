#pragma once

class UnicodeString {
public:
    UnicodeString() = default;
    void truncateTo(Int) {}
    const WideChar* str() const {
        static const WideChar kEmpty[1] = { 0 };
        return kEmpty;
    }
};
