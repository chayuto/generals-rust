#pragma once
#include <bitset>
#include <functional>
#include <list>
#include <map>
#include <numeric>
#include <string>
#include <unordered_map>
#include <vector>

class AsciiString;
using AsciiStringList = std::list<AsciiString>;
using FilenameList = std::list<AsciiString>;
using ScienceVec = std::vector<ScienceType>;
using ProductionChangeMap = std::map<NameKeyType, Real>;
using ProductionVeterancyMap = std::map<NameKeyType, VeterancyLevel>;

namespace std {
template <typename K, typename V, typename H = std::hash<K>, typename Eq = std::equal_to<K>>
using hash_map = std::unordered_map<K, V, H, Eq>;
}

namespace rts {
template <typename T>
struct hash {
    size_t operator()(const T& x) const noexcept {
        return std::hash<Int>()(static_cast<Int>(x));
    }
};
template <typename T>
struct equal_to {
    bool operator()(const T& a, const T& b) const noexcept {
        return a == b;
    }
};
}
