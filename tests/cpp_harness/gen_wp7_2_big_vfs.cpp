// WP-7.2 Native .big Archive & VFS (PKT-448 ArchiveFile, PKT-449 ArchiveFileSystem,
// PKT-450 BIGFileSystem, PKT-454 File, PKT-460 RAMFile) C++ Golden Vector Generator.
// Compiles unmodified vendor/GeneralsX sources against minimal harness stubs.
//
// Canonical build & run command (macOS arm64, Release-aligned macros):
//   clang++ -std=c++20 -O1 -ffp-contract=off -URTS_DEBUG -D_UNIX=1 \
//     -DRTS_ZEROHOUR=1 -DRTS_GENERALS=0 -Wno-macro-redefined -Wno-switch \
//     -Itests/cpp_harness/stubs_wp7_2 \
//     -Ivendor/GeneralsX/GeneralsMD/Code/GameEngine/Include \
//     -Ivendor/GeneralsX/Core/GameEngine/Include \
//     -Ivendor/GeneralsX/Core/Libraries/Source/Lib/Include \
//     -Ivendor/GeneralsX/Dependencies/Utility \
//     tests/cpp_harness/gen_wp7_2_big_vfs.cpp \
//     -o /tmp/gen_wp7_2 && /tmp/gen_wp7_2 docs/golden_vectors/WP7_2_BigVfs.json
//
// Determinism & UB verification builds (must produce byte-identical JSON):
//   -O0, -O1, -O2, and -O1 -g -fsanitize=address,undefined (-std=c++20).

#include "PreRTS.h"
#include "Common/file.h"
#include "Common/FileSystem.h"
#include "Common/RAMFile.h"
#include "Common/StreamingArchiveFile.h"
#include "Common/ArchiveFile.h"
#include "Common/ArchiveFileSystem.h"
#include "Common/BIGFileSystem.h"
#include "Common/LocalFileSystem.h"
#include "Common/LocalFile.h"
#include "Common/GameAudio.h"
#include "Common/AudioAffect.h"
#include "Common/GlobalData.h"
#include "Common/Registry.h"

// Unmodified vendor C++ sources for WP-7.2:
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/file.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/RAMFile.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/ArchiveFile.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/ArchiveFileSystem.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/BIGFileSystem.cpp"

#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <map>
#include <string>
#include <vector>

const AsciiString AsciiString::TheEmptyString;

static AudioManager g_audio;
AudioManager* TheAudio = &g_audio;

static GlobalData g_global_data;
GlobalData* TheGlobalData = &g_global_data;

static Int g_stub_registry_sentinel = 0;

Bool GetStringFromGeneralsRegistry(AsciiString, AsciiString, AsciiString&) {
    (void)g_stub_registry_sentinel;
    return FALSE;
}

Bool GetStringFromRegistry(AsciiString, AsciiString, AsciiString&) {
    return FALSE;
}

Bool GetUnsignedIntFromRegistry(AsciiString, AsciiString, UnsignedInt&) {
    return FALSE;
}

// Verbatim FileSystem path key / comparison helpers from Core/GameEngine/Source/Common/System/FileSystem.cpp
FileSystem* TheFileSystem = nullptr;

static UnsignedByte wp72_toPathKeyChar(Char ch) {
    if (isPathSeparator(ch))
        return '\\';
    if (ch >= 'A' && ch <= 'Z')
        return static_cast<UnsignedByte>(ch + ('a' - 'A'));
    return static_cast<UnsignedByte>(ch);
}

static const Char* wp72_getPathKeyPairEnd(const Char* path) {
#ifdef _WIN32
    if (isPathSeparator(path[0]) && isPathSeparator(path[1]))
        return path + 2;
#endif
    return path;
}

Bool FileSystem::makePathKey(Char* key, size_t keySize, const Char* path) {
    Char* const keyBegin = key;
    const Char* const keyEnd = key + keySize - 1;
    const Char* const pairEnd = wp72_getPathKeyPairEnd(path);

    for (;;) {
        const UnsignedByte ch = wp72_toPathKeyChar(*path);
        if (ch == '\0')
            break;
        if (key == keyEnd) {
            *keyBegin = '\0';
            return FALSE;
        }
        *key++ = static_cast<Char>(ch);
        ++path;
        if (ch == '\\') {
            while (path > pairEnd && isPathSeparator(*path))
                ++path;
        }
    }
    *key = '\0';
    return TRUE;
}

Int FileSystem::comparePath(const Char* a, const Char* b) {
    const Char* const pairEndA = wp72_getPathKeyPairEnd(a);
    const Char* const pairEndB = wp72_getPathKeyPairEnd(b);

    for (;;) {
        const UnsignedByte ca = wp72_toPathKeyChar(*a);
        const UnsignedByte cb = wp72_toPathKeyChar(*b);
        if (ca != cb || ca == '\0')
            return static_cast<Int>(ca) - static_cast<Int>(cb);
        ++a;
        ++b;
        if (ca == '\\') {
            while (a > pairEndA && isPathSeparator(*a))
                ++a;
            while (b > pairEndB && isPathSeparator(*b))
                ++b;
        }
    }
}

// Minimal out-of-line definitions for StreamingArchiveFile so ArchiveFile.cpp links cleanly
StreamingArchiveFile::StreamingArchiveFile()
    : m_file(nullptr), m_startingPos(0), m_size(0), m_curPos(0) {}
StreamingArchiveFile::~StreamingArchiveFile() {}
Bool StreamingArchiveFile::open(const Char*, Int, size_t) { return FALSE; }
void StreamingArchiveFile::close() { RAMFile::close(); }
Int StreamingArchiveFile::read(void*, Int) { return -1; }
Int StreamingArchiveFile::write(const void*, Int) { return -1; }
Int StreamingArchiveFile::seek(Int, seekMode) { return -1; }
Bool StreamingArchiveFile::open(File*) { return FALSE; }
Bool StreamingArchiveFile::openFromArchive(File*, const AsciiString&, Int, Int) { return FALSE; }

// In-memory File & LocalFileSystem mock so the oracle is 100% OS-independent and deterministic
class MemDiskFile : public File {
public:
    MemDiskFile(std::vector<uint8_t> data) : m_data(std::move(data)), m_pos(0) {}
    ~MemDiskFile() override { File::closeWithoutDelete(); }

    Bool open(const Char* filename, Int access = NONE, size_t bufferSize = BUFFERSIZE) override {
        if (!File::open(filename, access, bufferSize)) return FALSE;
        m_pos = 0;
        return TRUE;
    }

    Int read(void* buffer, Int bytes) override {
        if (!m_open || bytes < 0) return -1;
        Int rem = static_cast<Int>(m_data.size()) - m_pos;
        Int to_read = (bytes > rem) ? rem : bytes;
        if (to_read > 0 && buffer != nullptr) {
            std::memcpy(buffer, m_data.data() + m_pos, static_cast<size_t>(to_read));
        }
        m_pos += to_read;
        return to_read;
    }
    Int readChar() override { return EOF; }
    Int readWideChar() override { return WEOF; }
    Int write(const void*, Int) override { return -1; }
    Int writeFormat(const Char*, ...) override { return -1; }
    Int writeFormat(const WideChar*, ...) override { return -1; }
    Int writeChar(const Char*) override { return EOF; }
    Int writeChar(const WideChar*) override { return WEOF; }
    Int seek(Int bytes, seekMode mode = CURRENT) override {
        Int np = 0;
        if (mode == START) np = bytes;
        else if (mode == CURRENT) np = m_pos + bytes;
        else if (mode == END) np = static_cast<Int>(m_data.size()) + bytes;
        else return -1;
        if (np < 0) np = 0;
        Int sz = static_cast<Int>(m_data.size());
        if (np > sz) np = sz;
        m_pos = np;
        return m_pos;
    }
    Bool flush() override { return FALSE; }
    void nextLine(Char*, Int) override {}
    Bool scanInt(Int&) override { return FALSE; }
    Bool scanReal(Real&) override { return FALSE; }
    Bool scanString(AsciiString&) override { return FALSE; }
    Int size() override { return static_cast<Int>(m_data.size()); }
    char* readEntireAndClose() override { return nullptr; }
    File* convertToRAMFile() override { return nullptr; }

private:
    std::vector<uint8_t> m_data;
    Int m_pos;
};

struct MemFileEntry {
    std::vector<uint8_t> bytes;
    FileInfo info;
};

class MockLocalFileSystem : public LocalFileSystem {
public:
    std::map<std::string, MemFileEntry> files;
    AsciiString last_asset_root;

    void clearFiles() {
        files.clear();
        last_asset_root.clear();
    }

    static std::string normKey(const char* p) {
        std::string s = p ? p : "";
        for (char& c : s) {
            if (c == '\\') c = '/';
            else c = static_cast<char>(std::tolower(static_cast<unsigned char>(c)));
        }
        return s;
    }

    static bool globMatch(const char* name, const char* pat) {
        if (!name || !pat) return false;
        while (*pat) {
            if (*pat == '*') {
                ++pat;
                if (*pat == '\0') return true;
                while (*name) {
                    if (globMatch(name, pat)) return true;
                    ++name;
                }
                return globMatch(name, pat);
            } else if (*pat == '?') {
                if (*name == '\0') return false;
                ++name;
                ++pat;
            } else {
                if (std::tolower(static_cast<unsigned char>(*name)) !=
                    std::tolower(static_cast<unsigned char>(*pat))) {
                    return false;
                }
                ++name;
                ++pat;
            }
        }
        return *name == '\0';
    }

    void addFile(const std::string& path, std::vector<uint8_t> bytes, Int ts_hi, Int ts_lo) {
        MemFileEntry e;
        e.info.sizeHigh = 0;
        e.info.sizeLow = static_cast<Int>(bytes.size());
        e.info.timestampHigh = ts_hi;
        e.info.timestampLow = ts_lo;
        e.bytes = std::move(bytes);
        files[normKey(path.c_str())] = std::move(e);
    }

    void init() override {}
    void update() override {}
    void reset() override {}

    File* openFile(const Char* filename, Int access = File::NONE, size_t bufferSize = File::BUFFERSIZE) override {
        auto it = files.find(normKey(filename));
        if (it == files.end()) return nullptr;
        auto* f = new MemDiskFile(it->second.bytes);
        f->deleteOnClose();
        if (!f->open(filename, access, bufferSize)) {
            delete f;
            return nullptr;
        }
        return f;
    }

    Bool doesFileExist(const Char* filename) const override {
        return files.find(normKey(filename)) != files.end() ? TRUE : FALSE;
    }

    void getFileListInDirectory(
        const AsciiString& currentDirectory,
        const AsciiString& /*originalDirectory*/,
        const AsciiString& searchName,
        FilenameList& filenameList,
        Bool searchSubdirectories) const override {
        std::string dir = normKey(currentDirectory.str());
        while (!dir.empty() && dir.back() == '/') dir.pop_back();
        const char* pat = searchName.isEmpty() ? "*" : searchName.str();

        for (const auto& kv : files) {
            const std::string& full = kv.first;
            std::string rel;
            if (dir.empty()) {
                rel = full;
            } else if (full.size() > dir.size() && full.compare(0, dir.size(), dir) == 0 && full[dir.size()] == '/') {
                rel = full.substr(dir.size() + 1);
            } else {
                continue;
            }
            if (!searchSubdirectories && rel.find('/') != std::string::npos) {
                continue;
            }
            size_t slash = rel.rfind('/');
            const char* leaf = (slash == std::string::npos) ? rel.c_str() : (rel.c_str() + slash + 1);
            if (globMatch(leaf, pat)) {
                filenameList.insert(AsciiString(full.c_str()));
            }
        }
    }

    Bool getFileInfo(const AsciiString& filename, FileInfo* fileInfo) const override {
        auto it = files.find(normKey(filename.str()));
        if (it == files.end()) return FALSE;
        if (fileInfo) *fileInfo = it->second.info;
        return TRUE;
    }

    Bool createDirectory(AsciiString) override { return TRUE; }
    AsciiString normalizePath(const AsciiString& filePath) const override { return filePath; }
    void setAssetRootPath(const AsciiString& path) override { last_asset_root = path; }
};

static MockLocalFileSystem g_local_fs;
LocalFileSystem* TheLocalFileSystem = &g_local_fs;

// Minimal out-of-line FileSystem forwarding for RAMFile::open(filename)
FileSystem::FileSystem() = default;
FileSystem::~FileSystem() = default;
void FileSystem::init() {}
void FileSystem::reset() {}
void FileSystem::update() {}
File* FileSystem::openFile(const Char* filename, Int access, size_t bufferSize, FileInstance /*instance*/) {
    return TheLocalFileSystem->openFile(filename, access, bufferSize);
}

static FileSystem g_fs;

static std::string sha256_file(const char* path) {
    static const uint32_t K[64] = {
        0x428a2f98u, 0x71374491u, 0xb5c0fbcfu, 0xe9b5dba5u, 0x3956c25bu, 0x59f111f1u, 0x923f82a4u, 0xab1c5ed5u,
        0xd807aa98u, 0x12835b01u, 0x243185beu, 0x550c7dc3u, 0x72be5d74u, 0x80deb1feu, 0x9bdc06a7u, 0xc19bf174u,
        0xe49b69c1u, 0xefbe4786u, 0x0fc19dc6u, 0x240ca1ccu, 0x2de92c6fu, 0x4a7484aau, 0x5cb0a9dcu, 0x76f988dau,
        0x983e5152u, 0xa831c66du, 0xb00327c8u, 0xbf597fc7u, 0xc6e00bf3u, 0xd5a79147u, 0x06ca6351u, 0x14292967u,
        0x27b70a85u, 0x2e1b2138u, 0x4d2c6dfcu, 0x53380d13u, 0x650a7354u, 0x766a0abbu, 0x81c2c92eu, 0x92722c85u,
        0xa2bfe8a1u, 0xa81a664bu, 0xc24b8b70u, 0xc76c51a3u, 0xd192e819u, 0xd6990624u, 0xf40e3585u, 0x106aa070u,
        0x19a4c116u, 0x1e376c08u, 0x2748774cu, 0x34b0bcb5u, 0x391c0cb3u, 0x4ed8aa4au, 0x5b9cca4fu, 0x682e6ff3u,
        0x748f82eeu, 0x78a5636fu, 0x84c87814u, 0x8cc70208u, 0x90befffau, 0xa4506cebu, 0xbef9a3f7u, 0xc67178f2u
    };
    FILE* f = std::fopen(path, "rb");
    if (!f) { std::perror(path); std::exit(1); }
    std::vector<uint8_t> data;
    std::fseek(f, 0, SEEK_END);
    long sz = std::ftell(f);
    std::fseek(f, 0, SEEK_SET);
    if (sz > 0) {
        data.resize(static_cast<size_t>(sz));
        if (std::fread(data.data(), 1, static_cast<size_t>(sz), f) != static_cast<size_t>(sz)) {
            std::fclose(f);
            std::exit(1);
        }
    }
    std::fclose(f);

    uint64_t bit_len = static_cast<uint64_t>(data.size()) * 8ULL;
    data.push_back(0x80);
    while ((data.size() % 64) != 56) data.push_back(0);
    for (int i = 7; i >= 0; --i) data.push_back(static_cast<uint8_t>((bit_len >> (i * 8)) & 0xff));

    uint32_t h[8] = {
        0x6a09e667u, 0xbb67ae85u, 0x3c6ef372u, 0xa54ff53au,
        0x510e527fu, 0x9b05688cu, 0x1f83d9abu, 0x5be0cd19u
    };
    auto rotr = [](uint32_t x, uint32_t n) { return (x >> n) | (x << (32 - n)); };
    for (size_t off = 0; off < data.size(); off += 64) {
        uint32_t w[64];
        for (int i = 0; i < 16; ++i) {
            w[i] = (static_cast<uint32_t>(data[off + i * 4]) << 24) |
                   (static_cast<uint32_t>(data[off + i * 4 + 1]) << 16) |
                   (static_cast<uint32_t>(data[off + i * 4 + 2]) << 8) |
                   static_cast<uint32_t>(data[off + i * 4 + 3]);
        }
        for (int i = 16; i < 64; ++i) {
            uint32_t s0 = rotr(w[i - 15], 7) ^ rotr(w[i - 15], 18) ^ (w[i - 15] >> 3);
            uint32_t s1 = rotr(w[i - 2], 17) ^ rotr(w[i - 2], 19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16] + s0 + w[i - 7] + s1;
        }
        uint32_t a = h[0], b = h[1], c = h[2], d = h[3], e = h[4], f0 = h[5], g = h[6], hh = h[7];
        for (int i = 0; i < 64; ++i) {
            uint32_t S1 = rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25);
            uint32_t ch = (e & f0) ^ ((~e) & g);
            uint32_t temp1 = hh + S1 + ch + K[i] + w[i];
            uint32_t S0 = rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22);
            uint32_t maj = (a & b) ^ (a & c) ^ (b & c);
            uint32_t temp2 = S0 + maj;
            hh = g; g = f0; f0 = e; e = d + temp1;
            d = c; c = b; b = a; a = temp1 + temp2;
        }
        h[0] += a; h[1] += b; h[2] += c; h[3] += d;
        h[4] += e; h[5] += f0; h[6] += g; h[7] += hh;
    }
    char out[65];
    for (int i = 0; i < 8; ++i) std::snprintf(out + i * 8, 9, "%08x", h[i]);
    out[64] = '\0';
    return std::string(out);
}

static uint32_t crc32_bytes(const uint8_t* data, size_t len) {
    uint32_t crc = 0xFFFFFFFFu;
    for (size_t i = 0; i < len; ++i) {
        crc ^= data[i];
        for (int b = 0; b < 8; ++b) {
            uint32_t mask = static_cast<uint32_t>(-static_cast<int32_t>(crc & 1u));
            crc = (crc >> 1) ^ (0xEDB88320u & mask);
        }
    }
    return ~crc;
}

struct Rng32 {
    uint32_t s;
    explicit Rng32(uint32_t seed) : s(seed ? seed : 0x6d2b79f5u) {}
    uint32_t next() {
        uint32_t x = s;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        s = x;
        return x;
    }
    uint32_t range(uint32_t lo, uint32_t hi) {
        if (hi <= lo) return lo;
        return lo + (next() % (hi - lo + 1));
    }
    int32_t irange(int32_t lo, int32_t hi) {
        return lo + static_cast<int32_t>(range(0, static_cast<uint32_t>(hi - lo)));
    }
};

static std::string json_escape(const std::string& s) {
    std::string out;
    out.reserve(s.size() + 4);
    for (unsigned char c : s) {
        if (c == '"') out += "\\\"";
        else if (c == '\\') out += "\\\\";
        else if (c == '\n') out += "\\n";
        else if (c == '\r') out += "\\r";
        else if (c == '\t') out += "\\t";
        else if (c < 0x20) {
            char buf[8];
            std::snprintf(buf, sizeof(buf), "\\u%04x", c);
            out += buf;
        } else {
            out += static_cast<char>(c);
        }
    }
    return out;
}

static std::string hex_encode(const std::vector<uint8_t>& v) {
    static const char* kHex = "0123456789abcdef";
    std::string out;
    out.reserve(v.size() * 2);
    for (uint8_t b : v) {
        out.push_back(kHex[(b >> 4) & 0xF]);
        out.push_back(kHex[b & 0xF]);
    }
    return out;
}

struct BigEntrySpec {
    std::string path;
    std::vector<uint8_t> data;
};

static std::vector<uint8_t> build_big_archive(const std::vector<BigEntrySpec>& entries) {
    // Compute directory table size
    size_t dir_size = 0;
    for (const auto& e : entries) {
        dir_size += 8 + e.path.size() + 1;
    }
    uint32_t first_offset = static_cast<uint32_t>(16 + dir_size);
    uint32_t cur_offset = first_offset;
    for (const auto& e : entries) {
        cur_offset += static_cast<uint32_t>(e.data.size());
    }
    uint32_t total_size = cur_offset;

    std::vector<uint8_t> buf(total_size, 0);
    buf[0] = 'B'; buf[1] = 'I'; buf[2] = 'G'; buf[3] = 'F';
    // LE archiveFileSize
    buf[4] = static_cast<uint8_t>(total_size & 0xff);
    buf[5] = static_cast<uint8_t>((total_size >> 8) & 0xff);
    buf[6] = static_cast<uint8_t>((total_size >> 16) & 0xff);
    buf[7] = static_cast<uint8_t>((total_size >> 24) & 0xff);
    // BE numFiles
    uint32_t nf = static_cast<uint32_t>(entries.size());
    buf[8]  = static_cast<uint8_t>((nf >> 24) & 0xff);
    buf[9]  = static_cast<uint8_t>((nf >> 16) & 0xff);
    buf[10] = static_cast<uint8_t>((nf >> 8) & 0xff);
    buf[11] = static_cast<uint8_t>(nf & 0xff);
    // BE firstEntryOffset
    buf[12] = static_cast<uint8_t>((first_offset >> 24) & 0xff);
    buf[13] = static_cast<uint8_t>((first_offset >> 16) & 0xff);
    buf[14] = static_cast<uint8_t>((first_offset >> 8) & 0xff);
    buf[15] = static_cast<uint8_t>(first_offset & 0xff);

    size_t dir_pos = 16;
    uint32_t data_pos = first_offset;
    for (const auto& e : entries) {
        uint32_t fsz = static_cast<uint32_t>(e.data.size());
        buf[dir_pos + 0] = static_cast<uint8_t>((data_pos >> 24) & 0xff);
        buf[dir_pos + 1] = static_cast<uint8_t>((data_pos >> 16) & 0xff);
        buf[dir_pos + 2] = static_cast<uint8_t>((data_pos >> 8) & 0xff);
        buf[dir_pos + 3] = static_cast<uint8_t>(data_pos & 0xff);
        buf[dir_pos + 4] = static_cast<uint8_t>((fsz >> 24) & 0xff);
        buf[dir_pos + 5] = static_cast<uint8_t>((fsz >> 16) & 0xff);
        buf[dir_pos + 6] = static_cast<uint8_t>((fsz >> 8) & 0xff);
        buf[dir_pos + 7] = static_cast<uint8_t>(fsz & 0xff);
        std::memcpy(buf.data() + dir_pos + 8, e.path.data(), e.path.size());
        buf[dir_pos + 8 + e.path.size()] = 0;
        dir_pos += 8 + e.path.size() + 1;

        if (!e.data.empty()) {
            std::memcpy(buf.data() + data_pos, e.data.data(), e.data.size());
            data_pos += fsz;
        }
    }
    return buf;
}

static std::vector<uint8_t> gen_payload(Rng32& rng, size_t len, uint8_t tag) {
    std::vector<uint8_t> out(len);
    for (size_t i = 0; i < len; ++i) {
        out[i] = static_cast<uint8_t>((rng.next() ^ (tag * 37u + i)) & 0xffu);
    }
    return out;
}

static uint32_t f32_bits(Real v) {
    uint32_t bits = 0;
    std::memcpy(&bits, &v, 4);
    return bits;
}

int main(int argc, char** argv) {
    unsetenv("CNC_GENERALS_ZH_PATH");
    unsetenv("CNC_GENERALS_PATH");
    unsetenv("GENERALSX_ASSET_PATH");
    unsetenv("GENERALSX_GENERALS_ASSET_PATH");
    unsetenv("CNC_ZH_INSTALLPATH");
    unsetenv("CNC_GENERALS_INSTALLPATH");
    unsetenv("GENERALSX_LOOSE_WINS");

    TheFileSystem = &g_fs;

    const char* out_path = (argc > 1) ? argv[1] : "docs/golden_vectors/WP7_2_BigVfs.json";
    FILE* out = std::fopen(out_path, "wb");
    if (!out) {
        std::perror(out_path);
        return 1;
    }

    const char* vendor_files[] = {
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/file.h",
        "vendor/GeneralsX/Core/GameEngine/Source/Common/System/file.cpp",
        "vendor/GeneralsX/Core/GameEngine/Include/Common/RAMFile.h",
        "vendor/GeneralsX/Core/GameEngine/Source/Common/System/RAMFile.cpp",
        "vendor/GeneralsX/Core/GameEngine/Include/Common/ArchiveFile.h",
        "vendor/GeneralsX/Core/GameEngine/Source/Common/System/ArchiveFile.cpp",
        "vendor/GeneralsX/Core/GameEngine/Include/Common/ArchiveFileSystem.h",
        "vendor/GeneralsX/Core/GameEngine/Source/Common/System/ArchiveFileSystem.cpp",
        "vendor/GeneralsX/Core/GameEngine/Include/Common/BIGFileSystem.h",
        "vendor/GeneralsX/Core/GameEngine/Source/Common/System/BIGFileSystem.cpp",
    };

    std::fprintf(out, "{\n");
    std::fprintf(out, "  \"packet_group\": \"WP-7.2\",\n");
    std::fprintf(out, "  \"packets\": [\"PKT-448\", \"PKT-449\", \"PKT-450\", \"PKT-454\", \"PKT-460\"],\n");
    std::fprintf(out, "  \"source_sha256\": {\n");
    for (size_t i = 0; i < ARRAY_SIZE(vendor_files); ++i) {
        std::string h = sha256_file(vendor_files[i]);
        std::fprintf(out, "    \"%s\": \"%s\"%s\n",
            vendor_files[i], h.c_str(), (i + 1 < ARRAY_SIZE(vendor_files)) ? "," : "");
    }
    std::fprintf(out, "  },\n");
    std::fprintf(out, "  \"vector_count\": 1000,\n");
    std::fprintf(out, "  \"vectors\": [\n");

    static const char* kPathSamples[] = {
        "Data\\INI\\GameData.ini",
        "data/ini//gamedata.ini",
        "DATA\\\\INI/Object\\Tank.ini",
        "maps\\Alpine Assault\\alpine assault.map",
        "Maps/Alpine Assault/alpine assault.map",
        "Art/Textures/tex_01.tga",
        "Data/Scripts/SkirmishScripts.scb",
        "Data\\Scripts\\MULTIPLAYERSCRIPTS.SCB",
        "Scripts.ini",
        "data/ini/unit.txt",
        "\\\\server\\share\\file.big",
        "///triple///slash",
        "plainfile",
        "",
    };
    static const char* kTextTokens[] = {
        "header: -42 0 12.5 foo\n",
        "val=.5 second=-.25 third=7. tail=0.125\r\n",
        "  +19   -0.0  999  0007 end\n",
        "weird - . -. abc 1024 -2048 3.75 done",
        "line_one_exact_16\nline_two_longer_than_buf\nlast_no_nl",
        "  \t  4294967295  -9999999  -1.5  word_A word_B  \n",
    };
    static const char* kQueryPaths[] = {
        "Data\\INI\\GameData.ini",
        "data/ini/unit.ini",
        "Data\\Scripts\\SkirmishScripts.scb",
        "Data/INI/Empty.ini",
        "Data\\INI\\*",
        "Art\\Textures\\tex1.dds",
        "Missing\\Nope.ini",
    };
    static const char* kSearchDirs[] = {
        "",
        "Data",
        "data\\ini",
        "Data/Missing",
    };
    static const char* kSearchPatterns[] = {
        "*",
        "*.ini",
        "unit?.ini",
        "*script*",
    };

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        Rng32 rng(0x9E3779B9u ^ (seed * 0x85EBCA6Bu));

        // 1. File::open & path utilities
        static const Int kRawFlags[] = {
            File::NONE,
            File::READ,
            File::WRITE,
            File::READWRITE,
            File::APPEND,
            File::WRITE | File::APPEND,
            File::TEXT | File::READ,
            File::BINARY | File::WRITE,
            File::STREAMING | File::READ,
            File::STREAMING | File::WRITE,
            File::TEXT | File::BINARY,
            File::LINEBUF | File::WRITE,
            File::FULLBUF | File::READ,
        };
        Int raw_flag = kRawFlags[rng.range(0, ARRAY_SIZE(kRawFlags) - 1)];
        MemDiskFile tf(std::vector<uint8_t>{1, 2, 3, 4});
        Bool open1 = tf.open("test/path.bin", raw_flag);
        Int norm_access = open1 ? tf.getAccess() : -1;
        Bool open2 = tf.open("test/other.bin", File::READ);
        tf.close();

        const char* pa = kPathSamples[rng.range(0, ARRAY_SIZE(kPathSamples) - 1)];
        const char* pb = kPathSamples[rng.range(0, ARRAY_SIZE(kPathSamples) - 1)];
        Int cmp_ab = FileSystem::comparePath(pa, pb);
        Int cmp_norm = (cmp_ab < 0) ? -1 : ((cmp_ab > 0) ? 1 : 0);
        char keybuf[24];
        size_t key_cap = static_cast<size_t>(rng.range(4, 24));
        Bool key_ok = FileSystem::makePathKey(keybuf, key_cap, pa);
        const char* fname_ptr = getFileName(pa);

        // 2. RAMFile operations
        g_local_fs.clearFiles();
        std::string txt_in = std::string(kTextTokens[rng.range(0, ARRAY_SIZE(kTextTokens) - 1)])
            + " " + kTextTokens[rng.range(0, ARRAY_SIZE(kTextTokens) - 1)];
        if (seed % 7 == 0) txt_in.clear();
        std::vector<uint8_t> txt_bytes(txt_in.begin(), txt_in.end());
        g_local_fs.addFile("mem/text.txt", txt_bytes, 1, static_cast<Int>(seed));

        RAMFile rf;
        Bool rf_ok = rf.open("mem/text.txt", File::READ | File::BINARY, 0);
        Int rf_sz = rf.size();
        std::vector<int32_t> scan_ints;
        for (int k = 0; k < 2; ++k) {
            Int v = -777;
            Bool ok = rf.scanInt(v);
            scan_ints.push_back(ok ? 1 : 0);
            scan_ints.push_back(v);
        }
        rf.seek(0, File::START);
        std::vector<uint32_t> scan_real_bits;
        for (int k = 0; k < 2; ++k) {
            Real rv = 0.0f;
            Bool ok = rf.scanReal(rv);
            scan_real_bits.push_back(ok ? 1u : 0u);
            scan_real_bits.push_back(f32_bits(rv));
        }
        rf.seek(0, File::START);
        std::vector<std::string> scan_strs;
        for (int k = 0; k < 2; ++k) {
            AsciiString s;
            if (rf.scanString(s)) scan_strs.push_back(s.str());
            else scan_strs.push_back("<EOF>");
        }
        Int seek_off = rng.irange(-10, static_cast<int32_t>(txt_bytes.size()) + 10);
        Int seek_mode_i = static_cast<Int>(rng.range(0, 3));
        Int seek_ret = rf.seek(seek_off, static_cast<File::seekMode>(seek_mode_i));
        Int end_pos = rf.position();
        Bool is_eof = rf.eof();

        Int next_buf_sz = static_cast<Int>(rng.range(4, 24));
        std::vector<char> line_buf(static_cast<size_t>(next_buf_sz + 1), 0);
        rf.seek(rng.irange(0, static_cast<int32_t>(txt_bytes.size())), File::START);
        rf.nextLine(line_buf.data(), next_buf_sz);
        std::string next_line_str(line_buf.data());
        Int after_nl_pos = rf.position();

        // Also test readEntireAndClose on a fresh RAMFile
        RAMFile rf2;
        Bool rf2_ok = rf2.open("mem/text.txt", File::READ | File::BINARY, 0);
        uint32_t entire_crc = 0;
        if (rf2_ok) {
            Int sz2 = rf2.size();
            char* raw = rf2.readEntireAndClose();
            entire_crc = crc32_bytes(reinterpret_cast<const uint8_t*>(raw), static_cast<size_t>(sz2));
            delete[] raw;
        }
        rf.close();

        // 3. BIG binary parser (valid + occasional corrupt BIG)
        g_local_fs.clearFiles();
        std::vector<BigEntrySpec> corrupt_entries;
        corrupt_entries.push_back({"Data\\INI\\Test.ini", gen_payload(rng, 25, 1)});
        corrupt_entries.push_back({"Data\\*\\Dummy", gen_payload(rng, 5, 2)});
        corrupt_entries.push_back({"Data\\INI\\Zero.ini", {}});
        std::vector<uint8_t> raw_big = build_big_archive(corrupt_entries);
        uint32_t corrupt_mode = seed % 5; // 0: valid, 1: bad magic, 2: short header, 3: short entry, 4: unterminated path
        std::vector<uint8_t> test_big = raw_big;
        if (corrupt_mode == 1) test_big[0] = 'X';
        else if (corrupt_mode == 2) test_big.resize(10);
        else if (corrupt_mode == 3) test_big.resize(20);
        else if (corrupt_mode == 4) {
            // Fill string area without null terminator for first entry
            for (size_t k = 24; k < test_big.size(); ++k) test_big[k] = 'a';
            test_big.resize(24 + 270, 'a');
        }
        g_local_fs.addFile("archives/probe.big", test_big, 7, static_cast<Int>(seed + 10));

        // 4. Multi-archive overlay, Patch*, generalsx*, !mod, @mod, Data/INI/INIZH.big skip, ZH_Generals skip
        bool allow_patch = (seed % 2 == 0);
        bool overwrite = ((seed / 2) % 2 == 0);
        TheGlobalData->m_looseFilesWin = ((seed / 4) % 2 == 0) ? TRUE : FALSE;
        TheGlobalData->m_modBIG = (seed % 3 == 0) ? AsciiString("root/custom_mod.big") : AsciiString("");
        TheGlobalData->m_modDir = (seed % 4 == 0) ? AsciiString("root/mymod") : AsciiString("");

        std::vector<BigEntrySpec> e_base = {
            {"Data\\INI\\GameData.ini", gen_payload(rng, rng.range(8, 48), 11)},
            {"DATA/ini/unit.ini", gen_payload(rng, rng.range(8, 48), 12)},
            {"Data\\INI\\Empty.ini", {}},
            {"Data\\*", {'x'}},
            {"Data\\Bad?Name.ini", {'y'}},
            {"Art\\Textures\\tex1.dds", gen_payload(rng, 16, 13)},
        };
        std::vector<BigEntrySpec> e_patch = {
            {"Data/INI/GameData.ini", gen_payload(rng, rng.range(10, 40), 21)},
            {"Data\\Scripts\\SkirmishScripts.scb", gen_payload(rng, 20, 22)},
        };
        std::vector<BigEntrySpec> e_gx = {
            {"data\\ini\\unit.ini", gen_payload(rng, rng.range(12, 36), 31)},
        };
        std::vector<BigEntrySpec> e_mod = {
            {"Data\\INI\\GameData.ini", gen_payload(rng, rng.range(14, 30), 41)},
            {"Data\\Scripts\\SkirmishScripts.scb", gen_payload(rng, 18, 42)},
        };

        std::vector<uint8_t> big_base = build_big_archive(e_base);
        std::vector<uint8_t> big_patch = build_big_archive(e_patch);
        std::vector<uint8_t> big_gx = build_big_archive(e_gx);
        std::vector<uint8_t> big_mod = build_big_archive(e_mod);

        g_local_fs.addFile("root/INIZH.big", big_base, 100, 1);
        g_local_fs.addFile("root/Data/INI/INIZH.big", big_gx, 100, 2); // must be skipped in ZH
        g_local_fs.addFile("root/ZH_Generals/INI.big", big_gx, 100, 3); // skipped if allow_patch
        g_local_fs.addFile("root/PatchZH.big", big_patch, 200, 4);
        g_local_fs.addFile("root/generalsx.big", big_gx, 300, 5);
        g_local_fs.addFile("root/!ModAlpha.big", big_mod, 400, 6);
        g_local_fs.addFile("root/mymod/CustomPack.big", big_mod, 500, 7);
        g_local_fs.addFile("root/custom_mod.big", big_patch, 600, 8);

        BIGFileSystem bfs;
        ArchiveFile* probe_arc = bfs.openArchiveFile("archives/probe.big");
        Int probe_ok = (probe_arc != nullptr) ? 1 : 0;
        uint32_t probe_crc = 0;
        Int probe_info_size = -1;
        if (probe_arc != nullptr) {
            FileInfo fi{};
            if (probe_arc->getFileInfo(AsciiString("Data/INI/Test.ini"), &fi)) {
                probe_info_size = fi.sizeLow;
            }
            File* sub = probe_arc->openFile("data\\ini\\test.ini", File::READ | File::BINARY);
            if (sub != nullptr) {
                std::vector<uint8_t> tmp(static_cast<size_t>(sub->size()));
                Int got = tmp.empty() ? 0 : sub->read(tmp.data(), static_cast<Int>(tmp.size()));
                if (got > 0) probe_crc = crc32_bytes(tmp.data(), static_cast<size_t>(got));
                sub->close();
            }
            delete probe_arc;
        }

        Bool load_ok = bfs.loadArchiveFilesFromDirectory(
            AsciiString("root"), AsciiString("*.big"), overwrite ? TRUE : FALSE, allow_patch ? TRUE : FALSE);
        // Test mod loading cleanly on even seeds
        if (seed % 2 == 0) {
            bfs.loadMods();
        }
        // Test duplicate load prevention
        Bool dup_load = bfs.loadArchiveFilesFromDirectory(
            AsciiString("root"), AsciiString("INIZH.big"), FALSE, FALSE);

        const char* qpath = kQueryPaths[rng.range(0, ARRAY_SIZE(kQueryPaths) - 1)];
        Int qinst = static_cast<Int>(rng.range(0, 2));
        Int qfilt = static_cast<Int>(rng.range(0, 2));

        Bool exists_all = bfs.doesFileExist(qpath, static_cast<FileInstance>(qinst), ArchiveFileSystem::ARCHIVE_FILTER_ALL);
        Bool exists_filt = bfs.doesFileExist(qpath, static_cast<FileInstance>(qinst), static_cast<ArchiveFileSystem::ArchiveFilter>(qfilt));
        Int cnt_all = static_cast<Int>(bfs.getFileCount(qpath, ArchiveFileSystem::ARCHIVE_FILTER_ALL));
        Int cnt_mod = static_cast<Int>(bfs.getFileCount(qpath, ArchiveFileSystem::ARCHIVE_FILTER_MOD_ONLY));
        Int cnt_non = static_cast<Int>(bfs.getFileCount(qpath, ArchiveFileSystem::ARCHIVE_FILTER_NON_MOD_ONLY));
        Bool stock_loose = ArchiveFileSystem::isStockLooseFile(qpath);
        Bool has_mod_ovr = bfs.hasModArchiveOverride(qpath);

        FileInfo qfi{};
        Bool qfi_ok = bfs.getFileInfo(AsciiString(qpath), &qfi, static_cast<FileInstance>(qinst), static_cast<ArchiveFileSystem::ArchiveFilter>(qfilt));
        uint32_t qdata_crc = 0;
        Int qdata_len = -1;
        File* qf = bfs.openFile(qpath, File::READ | File::BINARY, static_cast<FileInstance>(qinst), static_cast<ArchiveFileSystem::ArchiveFilter>(qfilt));
        if (qf != nullptr) {
            Int sz = qf->size();
            std::vector<uint8_t> tmp(static_cast<size_t>(sz));
            Int r = (sz > 0) ? qf->read(tmp.data(), sz) : 0;
            qdata_len = r;
            if (r > 0) qdata_crc = crc32_bytes(tmp.data(), static_cast<size_t>(r));
            qf->close();
        }

        const char* sdir = kSearchDirs[rng.range(0, ARRAY_SIZE(kSearchDirs) - 1)];
        const char* spat = kSearchPatterns[rng.range(0, ARRAY_SIZE(kSearchPatterns) - 1)];
        FilenameList found_list;
        bfs.getFileListInDirectory(AsciiString(sdir), AsciiString(sdir), AsciiString(spat), found_list, TRUE);

        std::fprintf(out, "    {\"seed\":%u,", seed);
        std::fprintf(out, "\"raw_flag\":%d,\"open1\":%d,\"norm_access\":%d,\"open2\":%d,",
            raw_flag, static_cast<int>(open1), norm_access, static_cast<int>(open2));
        std::fprintf(out, "\"pa\":\"%s\",\"pb\":\"%s\",\"cmp\":%d,\"key_cap\":%zu,\"key_ok\":%d,\"key\":\"%s\",\"fname\":\"%s\",",
            json_escape(pa).c_str(), json_escape(pb).c_str(), cmp_norm, key_cap, static_cast<int>(key_ok),
            json_escape(keybuf).c_str(), json_escape(fname_ptr).c_str());
        std::fprintf(out, "\"txt_hex\":\"%s\",\"rf_ok\":%d,\"rf_sz\":%d,\"scan_ints\":[%d,%d,%d,%d],",
            hex_encode(txt_bytes).c_str(), static_cast<int>(rf_ok), rf_sz,
            scan_ints[0], scan_ints[1], scan_ints[2], scan_ints[3]);
        std::fprintf(out, "\"scan_real_bits\":[%u,%u,%u,%u],\"scan_strs\":[\"%s\",\"%s\"],",
            scan_real_bits[0], scan_real_bits[1], scan_real_bits[2], scan_real_bits[3],
            json_escape(scan_strs[0]).c_str(), json_escape(scan_strs[1]).c_str());
        std::fprintf(out, "\"seek_off\":%d,\"seek_mode\":%d,\"seek_ret\":%d,\"end_pos\":%d,\"eof\":%d,",
            seek_off, seek_mode_i, seek_ret, end_pos, static_cast<int>(is_eof));
        std::fprintf(out, "\"nl_cap\":%d,\"nl_str\":\"%s\",\"after_nl\":%d,\"entire_crc\":%u,",
            next_buf_sz, json_escape(next_line_str).c_str(), after_nl_pos, entire_crc);
        std::fprintf(out, "\"probe_mode\":%u,\"probe_hex\":\"%s\",\"probe_ok\":%d,\"probe_sz\":%d,\"probe_crc\":%u,",
            corrupt_mode, hex_encode(test_big).c_str(), probe_ok, probe_info_size, probe_crc);
        std::fprintf(out, "\"allow_patch\":%d,\"overwrite\":%d,\"loose_wins\":%d,\"load_ok\":%d,\"dup_load\":%d,",
            allow_patch ? 1 : 0, overwrite ? 1 : 0, static_cast<int>(TheGlobalData->m_looseFilesWin),
            static_cast<int>(load_ok), static_cast<int>(dup_load));
        std::fprintf(out, "\"base_hex\":\"%s\",\"patch_hex\":\"%s\",\"gx_hex\":\"%s\",\"mod_hex\":\"%s\",",
            hex_encode(big_base).c_str(), hex_encode(big_patch).c_str(),
            hex_encode(big_gx).c_str(), hex_encode(big_mod).c_str());
        std::fprintf(out, "\"qpath\":\"%s\",\"qinst\":%d,\"qfilt\":%d,",
            json_escape(qpath).c_str(), qinst, qfilt);
        std::fprintf(out, "\"ex_all\":%d,\"ex_filt\":%d,\"cnt\":[%d,%d,%d],\"stock\":%d,\"mod_ovr\":%d,",
            static_cast<int>(exists_all), static_cast<int>(exists_filt), cnt_all, cnt_mod, cnt_non,
            static_cast<int>(stock_loose), static_cast<int>(has_mod_ovr));
        std::fprintf(out, "\"qfi_ok\":%d,\"qfi_sz\":%d,\"qfi_ts\":[%d,%d],\"qlen\":%d,\"qcrc\":%u,",
            static_cast<int>(qfi_ok), qfi.sizeLow, qfi.timestampHigh, qfi.timestampLow, qdata_len, qdata_crc);
        std::fprintf(out, "\"sdir\":\"%s\",\"spat\":\"%s\",\"flist\":[",
            json_escape(sdir).c_str(), json_escape(spat).c_str());
        size_t idx = 0;
        for (const auto& item : found_list) {
            std::fprintf(out, "\"%s\"%s", json_escape(item.str()).c_str(), (idx + 1 < found_list.size()) ? "," : "");
            ++idx;
        }
        std::fprintf(out, "]}%s\n", (seed + 1 < 1000) ? "," : "");
    }

    std::fprintf(out, "  ]\n}\n");
    std::fclose(out);
    return 0;
}
