#pragma once
// Shared zero-stub helper runtime for C++ golden-vector oracle drivers compiled
// directly against pre-built libz_gameengine.a via scripts/build_oracle.py.

#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <functional>
#include <string>
#include <vector>

#include "Common/CriticalSection.h"
#include "Common/GameMemory.h"
#include "Common/INI.h"
#include "Common/Snapshot.h"
#include "Common/Xfer.h"
#include "Common/XferCRC.h"
#include "Common/crc.h"

#define ORACLE_DEFINE_LINK_GLOBALS(app_name)                                     \
    int __argc = 1;                                                              \
    char* __argv_buf[] = { const_cast<char*>(app_name), nullptr };               \
    char** __argv = __argv_buf;                                                  \
    HWND ApplicationHWnd = nullptr;                                              \
    struct SDL_Window;                                                           \
    SDL_Window* TheSDL3Window = nullptr;                                         \
    const Char* g_csfFile = "data/%s/generals.csf";                              \
    const Char* g_strFile = "data/Generals.str";                                 \
    void OSDisplaySetBusyState(bool, bool) {}                                    \
    void OSDisplayWarningBox(AsciiString, AsciiString, unsigned int, unsigned int) {}

template <typename Tag, typename Tag::type M>
struct MemberAccess {
    friend typename Tag::type access_member(Tag) { return M; }
};

struct OracleSnapshotXferTag { using type = void (Snapshot::*)(Xfer*); };
template struct MemberAccess<OracleSnapshotXferTag, &Snapshot::xfer>;
inline void (Snapshot::*access_member(OracleSnapshotXferTag))(Xfer*);

struct OracleSnapshotCrcTag { using type = void (Snapshot::*)(Xfer*); };
template struct MemberAccess<OracleSnapshotCrcTag, &Snapshot::crc>;
inline void (Snapshot::*access_member(OracleSnapshotCrcTag))(Xfer*);

namespace oracle {

inline void init_memory() {
    static CriticalSection cs1, cs2, cs3, cs4;
    TheAsciiStringCriticalSection = &cs1;
    TheUnicodeStringCriticalSection = &cs2;
    TheDmaCriticalSection = &cs3;
    TheMemoryPoolCriticalSection = &cs4;
    initMemoryManager();
}

inline void call_snapshot_xfer(Snapshot* s, Xfer* x) {
    if (s) {
        (s->*access_member(OracleSnapshotXferTag()))(x);
    }
}

inline void call_snapshot_crc(Snapshot* s, Xfer* x) {
    if (s) {
        (s->*access_member(OracleSnapshotCrcTag()))(x);
    }
}

class MemoryXfer : public Xfer {
public:
    std::vector<uint8_t> bytes;
    size_t cursor = 0;
    std::vector<size_t> block_stack;
    std::function<void(MemoryXfer*, Snapshot*)> snapshot_cb;

    explicit MemoryXfer(
        XferMode mode,
        std::function<void(MemoryXfer*, Snapshot*)> cb = {})
        : snapshot_cb(std::move(cb)) {
        m_xferMode = mode;
    }

    void open(AsciiString identifier) override {
        m_identifier = identifier;
        cursor = 0;
        block_stack.clear();
        if (m_xferMode == XFER_SAVE) {
            bytes.clear();
        }
    }

    void close() override {}

    Int beginBlock() override {
        if (m_xferMode == XFER_SAVE) {
            block_stack.push_back(bytes.size());
            XferBlockSize placeholder = 0;
            xferInt(&placeholder);
            return 0;
        }
        if (m_xferMode == XFER_LOAD) {
            XferBlockSize block_size = 0;
            xferInt(&block_size);
            return block_size;
        }
        return 0;
    }

    void endBlock() override {
        if (m_xferMode == XFER_SAVE && !block_stack.empty()) {
            size_t header_pos = block_stack.back();
            block_stack.pop_back();
            XferBlockSize payload_size =
                static_cast<XferBlockSize>(bytes.size() - header_pos - sizeof(XferBlockSize));
            std::memcpy(bytes.data() + header_pos, &payload_size, sizeof(XferBlockSize));
        }
    }

    void skip(Int data_size) override {
        if (data_size <= 0) return;
        size_t n = static_cast<size_t>(data_size);
        if (m_xferMode == XFER_LOAD) {
            if (cursor + n > bytes.size()) {
                std::fprintf(stderr, "MemoryXfer::skip out of bounds (%zu + %zu > %zu)\n",
                             cursor, n, bytes.size());
                std::exit(2);
            }
            cursor += n;
        } else if (m_xferMode == XFER_SAVE) {
            bytes.resize(bytes.size() + n, 0u);
        }
    }

    void xferSnapshot(Snapshot* snapshot) override {
        if (!snapshot) return;
        if (snapshot_cb) {
            snapshot_cb(this, snapshot);
        } else {
            call_snapshot_xfer(snapshot, this);
        }
    }

    void xferAsciiString(AsciiString* s) override {
        if (m_xferMode == XFER_SAVE) {
            Int len_i = s ? s->getLength() : 0;
            if (len_i > 255) {
                std::fprintf(stderr, "MemoryXfer AsciiString too long (%d)\n", len_i);
                std::exit(2);
            }
            UnsignedByte len = static_cast<UnsignedByte>(len_i);
            xferUnsignedByte(&len);
            if (len > 0) {
                xferUser(const_cast<char*>(s->str()), sizeof(Byte) * len);
            }
        } else if (m_xferMode == XFER_LOAD) {
            UnsignedByte len = 0;
            xferUnsignedByte(&len);
            char buf[256];
            if (len > 0) {
                xferUser(buf, sizeof(Byte) * len);
            }
            buf[len] = '\0';
            if (s) s->set(buf);
        } else {
            Xfer::xferAsciiString(s);
        }
    }

    void xferUnicodeString(UnicodeString* s) override {
        if (m_xferMode == XFER_SAVE) {
            Int len_i = s ? s->getLength() : 0;
            if (len_i > 255) {
                std::fprintf(stderr, "MemoryXfer UnicodeString too long (%d)\n", len_i);
                std::exit(2);
            }
            UnsignedByte len = static_cast<UnsignedByte>(len_i);
            xferUnsignedByte(&len);
            if (len > 0) {
                xferUser(const_cast<WideChar*>(s->str()), sizeof(WideChar) * len);
            }
        } else if (m_xferMode == XFER_LOAD) {
            UnsignedByte len = 0;
            xferUnsignedByte(&len);
            WideChar buf[256];
            if (len > 0) {
                xferUser(buf, sizeof(WideChar) * len);
            }
            buf[len] = L'\0';
            if (s) s->set(buf);
        } else {
            Xfer::xferUnicodeString(s);
        }
    }

    std::string hex() const {
        static const char* kHex = "0123456789abcdef";
        std::string out;
        out.reserve(bytes.size() * 2);
        for (uint8_t b : bytes) {
            out.push_back(kHex[(b >> 4) & 0xf]);
            out.push_back(kHex[b & 0xf]);
        }
        return out;
    }

    UnsignedInt bytes_crc() const {
        CRC c;
        c.computeCRC(bytes.data(), static_cast<Int>(bytes.size()));
        return c.get();
    }

protected:
    void xferImplementation(void* data, Int data_size) override {
        if (data_size <= 0) return;
        size_t n = static_cast<size_t>(data_size);
        if (m_xferMode == XFER_SAVE) {
            const uint8_t* p = static_cast<const uint8_t*>(data);
            bytes.insert(bytes.end(), p, p + n);
        } else if (m_xferMode == XFER_LOAD) {
            if (cursor + n > bytes.size()) {
                std::fprintf(stderr, "MemoryXfer::xferImplementation EOF (%zu + %zu > %zu)\n",
                             cursor, n, bytes.size());
                std::exit(2);
            }
            std::memcpy(data, bytes.data() + cursor, n);
            cursor += n;
        }
    }
};

class MemINI : public INI {
public:
    void parse_blocks(
        const std::string& text,
        INILoadType load_type,
        const std::function<void(INI*, const char*)>& on_block) {
        m_readBuffer = new char[text.size() + 1];
        std::memcpy(m_readBuffer, text.data(), text.size() + 1);
        m_readBufferUsed = static_cast<unsigned>(text.size());
        m_readBufferNext = 0;
        m_lineNum = 0;
        m_endOfFile = FALSE;
        m_filename = "oracle_mem.ini";
        m_loadType = load_type;
        try {
            while (!m_endOfFile) {
                readLine();
                const char* tok = std::strtok(m_buffer, getSeps());
                if (tok) {
                    on_block(this, tok);
                }
            }
        } catch (...) {
            unPrepFile();
            throw;
        }
        unPrepFile();
    }
};

inline uint64_t splitmix64(uint64_t& state) {
    uint64_t z = (state += 0x9e3779b97f4a7c15ULL);
    z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9ULL;
    z = (z ^ (z >> 27)) * 0x94d049bb133111ebULL;
    return z ^ (z >> 31);
}

inline uint32_t splitmix32(uint32_t& state) {
    uint32_t z = (state += 0x9e3779b9u);
    z = (z ^ (z >> 16)) * 0x85ebca6bu;
    z = (z ^ (z >> 13)) * 0xc2b2ae35u;
    return z ^ (z >> 16);
}

inline uint32_t fbits(Real v) {
    uint32_t b = 0;
    std::memcpy(&b, &v, sizeof(b));
    return b;
}

inline Real from_bits(uint32_t b) {
    Real v = 0.0f;
    std::memcpy(&v, &b, sizeof(v));
    return v;
}

inline std::string json_escape(const std::string& s) {
    std::string out;
    out.reserve(s.size() + 8);
    for (unsigned char c : s) {
        switch (c) {
            case '"': out += "\\\""; break;
            case '\\': out += "\\\\"; break;
            case '\n': out += "\\n"; break;
            case '\r': out += "\\r"; break;
            case '\t': out += "\\t"; break;
            default:
                if (c < 0x20) {
                    char buf[7];
                    std::snprintf(buf, sizeof(buf), "\\u%04x", c);
                    out += buf;
                } else {
                    out.push_back(static_cast<char>(c));
                }
                break;
        }
    }
    return out;
}

inline std::string sha256_file(const char* path) {
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
    std::vector<uint8_t> d;
    std::fseek(f, 0, SEEK_END);
    long sz = std::ftell(f);
    std::fseek(f, 0, SEEK_SET);
    if (sz > 0) {
        d.resize(static_cast<size_t>(sz));
        if (std::fread(d.data(), 1, static_cast<size_t>(sz), f) != static_cast<size_t>(sz)) std::exit(1);
    }
    std::fclose(f);
    uint64_t bl = static_cast<uint64_t>(d.size()) * 8ULL;
    d.push_back(0x80u);
    while ((d.size() % 64) != 56) d.push_back(0u);
    for (int i = 7; i >= 0; --i) d.push_back(static_cast<uint8_t>((bl >> (i * 8)) & 0xffu));
    uint32_t H[8] = {
        0x6a09e667u, 0xbb67ae85u, 0x3c6ef372u, 0xa54ff53au,
        0x510e527fu, 0x9b05688cu, 0x1f83d9abu, 0x5be0cd19u
    };
    auto r = [](uint32_t x, uint32_t n) { return (x >> n) | (x << (32 - n)); };
    for (size_t off = 0; off < d.size(); off += 64) {
        uint32_t W[64];
        for (int i = 0; i < 16; ++i) {
            W[i] = (uint32_t(d[off + i * 4]) << 24) | (uint32_t(d[off + i * 4 + 1]) << 16) |
                   (uint32_t(d[off + i * 4 + 2]) << 8) | uint32_t(d[off + i * 4 + 3]);
        }
        for (int i = 16; i < 64; ++i) {
            W[i] = W[i - 16] + (r(W[i - 15], 7) ^ r(W[i - 15], 18) ^ (W[i - 15] >> 3)) +
                   W[i - 7] + (r(W[i - 2], 17) ^ r(W[i - 2], 19) ^ (W[i - 2] >> 10));
        }
        uint32_t a = H[0], b = H[1], c = H[2], d0 = H[3], e = H[4], f2 = H[5], g = H[6], h = H[7];
        for (int i = 0; i < 64; ++i) {
            uint32_t t1 = h + (r(e, 6) ^ r(e, 11) ^ r(e, 25)) + ((e & f2) ^ ((~e) & g)) + K[i] + W[i];
            uint32_t t2 = (r(a, 2) ^ r(a, 13) ^ r(a, 22)) + ((a & b) ^ (a & c) ^ (b & c));
            h = g; g = f2; f2 = e; e = d0 + t1; d0 = c; c = b; b = a; a = t1 + t2;
        }
        H[0] += a; H[1] += b; H[2] += c; H[3] += d0;
        H[4] += e; H[5] += f2; H[6] += g; H[7] += h;
    }
    char hex[65];
    for (int i = 0; i < 8; ++i) std::snprintf(hex + i * 8, 9, "%08x", H[i]);
    hex[64] = '\0';
    return std::string(hex);
}

inline void write_file_atomic(const char* final_path, const std::string& content) {
    std::string tmp = std::string(final_path) + ".tmp";
    FILE* fp = std::fopen(tmp.c_str(), "wb");
    if (!fp) { std::perror(tmp.c_str()); std::exit(1); }
    if (!content.empty() && std::fwrite(content.data(), 1, content.size(), fp) != content.size()) {
        std::perror("fwrite");
        std::fclose(fp);
        std::remove(tmp.c_str());
        std::exit(1);
    }
    if (std::fclose(fp) != 0) {
        std::perror("fclose");
        std::remove(tmp.c_str());
        std::exit(1);
    }
    if (std::rename(tmp.c_str(), final_path) != 0) {
        std::perror("rename");
        std::remove(tmp.c_str());
        std::exit(1);
    }
}

} // namespace oracle
