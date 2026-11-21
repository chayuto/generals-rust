// PKT-451 (DataChunk) C++ Golden Vector Generator.
// Compiles unmodified vendor/GeneralsX DataChunk.cpp + Dict.cpp + NameKeyGenerator.cpp
// together with EAC refencode.cpp + refdecode.cpp (compiled with portable big-endian prelude).
//
// Canonical build & run command (macos arm64):
//   EAC=vendor/GeneralsX/Core/Libraries/Source/Compression/EAC
//   mkdir -p /tmp/pkt451_obj
//   for f in refencode refdecode; do
//     clang++ -std=c++17 -O1 -ffp-contract=off \
//       -include tests/cpp_harness/stubs_wave4_eac/eac_portable_byteorder.h \
//       -c $EAC/$f.cpp -o /tmp/pkt451_obj/$f.o
//   done
//   clang++ -std=c++17 -O1 -ffp-contract=off -URTS_DEBUG -Wno-switch \
//     -Itests/cpp_harness/stubs_pkt451 \
//     -Ivendor/GeneralsX/GeneralsMD/Code/GameEngine/Include \
//     -Ivendor/GeneralsX/Core/GameEngine/Include \
//     -Ivendor/GeneralsX/Core/Libraries/Source/Compression \
//     -I$EAC \
//     tests/cpp_harness/gen_pkt451_datachunk.cpp \
//     /tmp/pkt451_obj/refencode.o /tmp/pkt451_obj/refdecode.o \
//     -o /tmp/pkt451_obj/gen_pkt451_datachunk && \
//   /tmp/pkt451_obj/gen_pkt451_datachunk docs/golden_vectors/PKT-451.json
//
// Determinism & UB verification: build & run at -O0, -O2, and -fsanitize=address,undefined;
// all JSON outputs must be byte-identical.
//
// __APPLE__ audit on vendor files compiled here:
//   - DataChunk.{h,cpp}, MapReaderWriterInfo.h, Dict.{h,cpp}, NameKeyGenerator.{h,cpp}: 0 hits.
//   - EAC/gimex.h: __APPLE__ PowerPC native-endian shortcut bypassed via eac_portable_byteorder.h
//     and verified at runtime by check_byte_order().

#include "PreRTS.h"
#include "Compression.h"
#include "refcodex.h"
#include "Common/Dict.h"
#include "Common/MapReaderWriterInfo.h"
#include "Common/DataChunk.h"
#include "Common/file.h"
#include "Common/FileSystem.h"

#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/Dict.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/NameKeyGenerator.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/System/DataChunk.cpp"

#include <unistd.h>

const AsciiString AsciiString::TheEmptyString("");
const UnicodeString UnicodeString::TheEmptyString;

static DynamicMemoryAllocator g_dma;
DynamicMemoryAllocator* TheDynamicMemoryAllocator = &g_dma;
static GlobalData g_global_data;
GlobalData* TheGlobalData = &g_global_data;
static FileSystem g_fs;
FileSystem* TheFileSystem = &g_fs;

// CompressionManager subset delegating EAR\0 to portable EAC REF_* codecs.
Bool CompressionManager::isDataCompressed(const void* mem, Int len) {
    if (!mem || len < 8) return FALSE;
    return std::memcmp(mem, "EAR\0", 4) == 0 ? TRUE : FALSE;
}

Int CompressionManager::getUncompressedSize(const void* mem, Int len) {
    if (!mem || len < 8 || !isDataCompressed(mem, len)) return len;
    const uint8_t* b = static_cast<const uint8_t*>(mem);
    uint32_t v = static_cast<uint32_t>(b[4]) |
                 (static_cast<uint32_t>(b[5]) << 8) |
                 (static_cast<uint32_t>(b[6]) << 16) |
                 (static_cast<uint32_t>(b[7]) << 24);
    return static_cast<Int>(v);
}

Int CompressionManager::decompressData(void* src, Int srcLen, void* dest, Int destLen) {
    if (!src || !dest || srcLen < 8 || !isDataCompressed(src, srcLen)) return 0;
    Int expected = getUncompressedSize(src, srcLen);
    if (destLen < expected || expected < 0) return 0;
    int compressed_len = srcLen - 8;
    int actual = REF_decode(dest, static_cast<const uint8_t*>(src) + 8, &compressed_len);
    return actual;
}

class MemOutputStream : public OutputStream {
public:
    std::vector<uint8_t> bytes;
    Int write(const void* pData, Int numBytes) override {
        if (numBytes <= 0 || !pData) return 0;
        const uint8_t* p = static_cast<const uint8_t*>(pData);
        bytes.insert(bytes.end(), p, p + numBytes);
        return numBytes;
    }
};

class SliceChunkInputStream : public ChunkInputStream {
    std::vector<uint8_t> m_buf;
    UnsignedInt m_pos = 0;
public:
    explicit SliceChunkInputStream(std::vector<uint8_t> buf) : m_buf(std::move(buf)) {}
    Int read(void* pData, Int numBytes) override {
        if (!pData || numBytes <= 0 || m_pos >= m_buf.size()) return 0;
        size_t avail = m_buf.size() - m_pos;
        size_t take = std::min(avail, static_cast<size_t>(numBytes));
        std::memcpy(pData, m_buf.data() + m_pos, take);
        m_pos += static_cast<UnsignedInt>(take);
        return static_cast<Int>(take);
    }
    UnsignedInt tell() override { return m_pos; }
    Bool absoluteSeek(UnsignedInt pos) override {
        if (pos > m_buf.size()) return FALSE;
        m_pos = pos;
        return TRUE;
    }
    Bool eof() override { return m_pos >= m_buf.size() ? TRUE : FALSE; }
};

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
            std::perror("fread"); std::exit(1);
        }
    }
    std::fclose(f);
    uint64_t bit_len = static_cast<uint64_t>(data.size()) * 8ULL;
    data.push_back(0x80u);
    while ((data.size() % 64u) != 56u) data.push_back(0x00u);
    for (int shift = 56; shift >= 0; shift -= 8) {
        data.push_back(static_cast<uint8_t>((bit_len >> shift) & 0xFFu));
    }
    uint32_t H[8] = {
        0x6a09e667u, 0xbb67ae85u, 0x3c6ef372u, 0xa54ff53au,
        0x510e527fu, 0x9b05688cu, 0x1f83d9abu, 0x5be0cd19u
    };
    auto rotr = [](uint32_t x, uint32_t n) -> uint32_t { return (x >> n) | (x << (32u - n)); };
    for (size_t off = 0; off < data.size(); off += 64u) {
        uint32_t w[64]{};
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
        uint32_t a = H[0], b = H[1], c = H[2], d = H[3];
        uint32_t e = H[4], f0 = H[5], g = H[6], h = H[7];
        for (int i = 0; i < 64; ++i) {
            uint32_t S1 = rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25);
            uint32_t ch = (e & f0) ^ ((~e) & g);
            uint32_t temp1 = h + S1 + ch + K[i] + w[i];
            uint32_t S0 = rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22);
            uint32_t maj = (a & b) ^ (a & c) ^ (b & c);
            uint32_t temp2 = S0 + maj;
            h = g; g = f0; f0 = e; e = d + temp1;
            d = c; c = b; b = a; a = temp1 + temp2;
        }
        H[0] += a; H[1] += b; H[2] += c; H[3] += d;
        H[4] += e; H[5] += f0; H[6] += g; H[7] += h;
    }
    char hex[65];
    for (int i = 0; i < 8; ++i) std::snprintf(hex + i * 8, 9, "%08x", H[i]);
    hex[64] = '\0';
    return std::string(hex);
}

static uint32_t splitmix32(uint32_t& s) {
    uint32_t z = (s += 0x9e3779b9u);
    z = (z ^ (z >> 16)) * 0x85ebca6bu;
    z = (z ^ (z >> 13)) * 0xc2b2ae35u;
    return z ^ (z >> 16);
}

static uint32_t fbits(Real v) {
    uint32_t b = 0;
    std::memcpy(&b, &v, 4);
    return b;
}

static Real bitsf(uint32_t b) {
    Real v = 0.0f;
    std::memcpy(&v, &b, 4);
    return v;
}

static uint32_t crc_u32(uint32_t c, uint32_t v) {
    return ((c << 1) | (c >> 31)) + v;
}

static uint32_t crc_str(uint32_t c, const char* s) {
    size_t n = s ? std::strlen(s) : 0u;
    c = crc_u32(c, static_cast<uint32_t>(n));
    for (size_t i = 0; i < n; ++i) {
        c = crc_u32(c, static_cast<uint8_t>(s[i]));
    }
    return c;
}

static std::string to_hex(const uint8_t* p, size_t n) {
    static const char* kHex = "0123456789abcdef";
    std::string out;
    out.reserve(n * 2);
    for (size_t i = 0; i < n; ++i) {
        out.push_back(kHex[(p[i] >> 4) & 0xF]);
        out.push_back(kHex[p[i] & 0xF]);
    }
    return out;
}

static std::vector<uint8_t> compress_ear(const std::vector<uint8_t>& raw) {
    std::vector<uint8_t> enc(raw.size() * 2u + 2048u + 8u, 0);
    std::memcpy(enc.data(), "EAR\0", 4);
    uint32_t ulen = static_cast<uint32_t>(raw.size());
    enc[4] = static_cast<uint8_t>(ulen & 0xffu);
    enc[5] = static_cast<uint8_t>((ulen >> 8) & 0xffu);
    enc[6] = static_cast<uint8_t>((ulen >> 16) & 0xffu);
    enc[7] = static_cast<uint8_t>((ulen >> 24) & 0xffu);
    int clen = REF_encode(enc.data() + 8, raw.data(), static_cast<int>(raw.size()), nullptr);
    if (clen <= 0) {
        std::fprintf(stderr, "REF_encode failed clen=%d\n", clen);
        std::exit(2);
    }
    enc.resize(8u + static_cast<size_t>(clen));
    return enc;
}

static void check_byte_order() {
    const char probe[] = "GeneralsX RefPack portable big-endian check";
    uint8_t out[128]{};
    int clen = REF_encode(out, probe, static_cast<int>(sizeof(probe) - 1u), nullptr);
    if (clen < 2 || out[0] != 0x10 || out[1] != 0xfb) {
        std::fprintf(stderr,
            "FATAL: REF_encode produced header %02x %02x instead of 10 fb (compile EAC codecs with eac_portable_byteorder.h)\n",
            out[0], out[1]);
        std::exit(2);
    }
}

struct ParseState {
    uint32_t crc = 0;
    Int tag = 0;
    Real val_real = 0.0f;
    Int val_i32 = 0;
    Byte val_byte = 0;
    AsciiString val_ascii;
    UnicodeString val_uni;
    std::vector<uint8_t> val_bytes;
    Dict dict;
    NameKeyType nk = NAMEKEY_INVALID;
};

static Bool parse_dup_loser(DataChunkInput&, DataChunkInfo*, void* userData) {
    ParseState* st = static_cast<ParseState*>(userData);
    st->tag = 1;
    return TRUE;
}

static Bool parse_dup_winner(DataChunkInput& input, DataChunkInfo* info, void* userData) {
    ParseState* st = static_cast<ParseState*>(userData);
    st->tag = 2; // Registered second -> wins in LIFO m_parserList!
    st->crc = crc_u32(st->crc, 2u);
    st->crc = crc_str(st->crc, info->label.str());
    st->crc = crc_str(st->crc, info->parentLabel.str());
    st->crc = crc_u32(st->crc, info->version);
    st->crc = crc_u32(st->crc, static_cast<uint32_t>(info->dataSize));
    st->nk = input.readNameKey();
    st->crc = crc_str(st->crc, TheNameKeyGenerator->keyToName(st->nk).str());
    st->dict = input.readDict();
    Int pc = st->dict.getPairCount();
    st->crc = crc_u32(st->crc, static_cast<uint32_t>(pc));
    for (Int i = 0; i < pc; ++i) {
        NameKeyType dk = st->dict.getNthKey(i);
        st->crc = crc_str(st->crc, TheNameKeyGenerator->keyToName(dk).str());
        Dict::DataType dt = st->dict.getNthType(i);
        st->crc = crc_u32(st->crc, static_cast<uint32_t>(dt));
        switch (dt) {
            case Dict::DICT_BOOL: st->crc = crc_u32(st->crc, st->dict.getNthBool(i)); break;
            case Dict::DICT_INT: st->crc = crc_u32(st->crc, static_cast<uint32_t>(st->dict.getNthInt(i))); break;
            case Dict::DICT_REAL: st->crc = crc_u32(st->crc, fbits(st->dict.getNthReal(i))); break;
            case Dict::DICT_ASCIISTRING: st->crc = crc_str(st->crc, st->dict.getNthAsciiString(i).str()); break;
            case Dict::DICT_UNICODESTRING: {
                UnicodeString us = st->dict.getNthUnicodeString(i);
                st->crc = crc_u32(st->crc, static_cast<uint32_t>(us.getLength()));
                for (Int j = 0; j < us.getLength(); ++j) {
                    st->crc = crc_u32(st->crc, static_cast<uint32_t>(us.str()[j]));
                }
                break;
            }
            default: break;
        }
    }
    return TRUE;
}

static Bool parse_child(DataChunkInput& input, DataChunkInfo* info, void* userData) {
    ParseState* st = static_cast<ParseState*>(userData);
    st->crc = crc_str(st->crc, info->label.str());
    st->crc = crc_str(st->crc, info->parentLabel.str());
    st->crc = crc_u32(st->crc, info->version);
    st->crc = crc_u32(st->crc, static_cast<uint32_t>(info->dataSize));
    st->val_real = input.readReal();
    st->val_i32 = input.readInt();
    st->val_byte = input.readByte();
    st->val_ascii = input.readAsciiString();
    st->val_uni = input.readUnicodeString();
    st->crc = crc_u32(st->crc, fbits(st->val_real));
    st->crc = crc_u32(st->crc, static_cast<uint32_t>(st->val_i32));
    st->crc = crc_u32(st->crc, static_cast<uint8_t>(st->val_byte));
    st->crc = crc_str(st->crc, st->val_ascii.str());
    st->crc = crc_u32(st->crc, static_cast<uint32_t>(st->val_uni.getLength()));
    for (Int i = 0; i < st->val_uni.getLength(); ++i) {
        st->crc = crc_u32(st->crc, static_cast<uint32_t>(st->val_uni.str()[i]));
    }
    if (!st->val_bytes.empty()) {
        std::vector<uint8_t> tmp(st->val_bytes.size(), 0);
        input.readArrayOfBytes(reinterpret_cast<char*>(tmp.data()), static_cast<Int>(tmp.size()));
        for (uint8_t b : tmp) st->crc = crc_u32(st->crc, b);
    }
    // Intentionally leave tail_len unread so closeDataChunk() must skip it!
    return TRUE;
}

static Bool parse_root(DataChunkInput& input, DataChunkInfo* info, void* userData) {
    ParseState* st = static_cast<ParseState*>(userData);
    st->crc = crc_str(st->crc, info->label.str());
    st->crc = crc_u32(st->crc, info->version);
    st->crc = crc_u32(st->crc, static_cast<uint32_t>(info->dataSize));
    if (!input.parse(userData)) return FALSE;
    return TRUE;
}

static void run_error_self_checks() {
    // 1. Not CkMp -> isValidFileType == FALSE
    std::vector<uint8_t> bogus = {'N', 'O', 'P', 'E', 0, 0, 0, 0};
    SliceChunkInputStream bogus_stream(bogus);
    DataChunkInput dci_bogus(&bogus_stream);
    if (dci_bogus.isValidFileType() != FALSE) {
        std::fprintf(stderr, "Self-check failed: bogus header accepted\n");
        std::exit(1);
    }
    // 2. Corrupt EAR header length mismatches -> CachedFileInputStream keeps raw buffer
    std::vector<uint8_t> raw_probe = {'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'};
    std::vector<uint8_t> bad_ear = compress_ear(raw_probe);
    // Corrupt header claiming LARGER uncompressed length (8 ^ 0x10 = 24) triggers actualLen != uncompLen
    // fallback in CachedFileInputStream without tripping REF_decode's missing output bounds check;
    // smaller-buffer/truncation bounds checks are tested directly in Rust.
    bad_ear[4] ^= 0x10;
    g_fs.setFile("bad.map", bad_ear);
    CachedFileInputStream cfis;
    if (!cfis.open(AsciiString("bad.map"))) {
        std::fprintf(stderr, "Self-check failed: CachedFileInputStream::open failed\n");
        std::exit(1);
    }
    uint8_t first4[4]{};
    Int got = cfis.read(first4, 4);
    cfis.close();
    if (got != 4 || std::memcmp(first4, "EAR\0", 4) != 0) {
        std::fprintf(stderr, "Self-check failed: bad EAR did not fall back to raw buffer\n");
        std::exit(1);
    }
    // 3. Corrupt Dict type throws ERROR_CORRUPT_FILE_FORMAT
    MemOutputStream mos;
    {
        DataChunkOutput out(&mos);
        out.openDataChunk("CorruptDict", 1);
        out.writeInt(0); // placeholder replaced below
        out.closeDataChunk();
    }
    // Craft minimal chunk with Dict count=1 and invalid dtype=99
    std::vector<uint8_t> bad_dict_chunk = {
        'C', 'k', 'M', 'p',
        1, 0, 0, 0, // toc count = 1
        1, 'X', 1, 0, 0, 0, // "X" -> id 1
        1, 0, 0, 0, // chunk id = 1
        1, 0,       // ver = 1
        6, 0, 0, 0, // size = 6
        1, 0,       // dict pairCount = 1
        99, 1, 0, 0 // (toc_id 1 << 8) | 99
    };
    SliceChunkInputStream s_bad(bad_dict_chunk);
    DataChunkInput d_bad(&s_bad);
    UnsignedShort v = 0;
    d_bad.openDataChunk(&v);
    bool caught = false;
    try {
        (void)d_bad.readDict();
    } catch (Int code) {
        if (code == ERROR_CORRUPT_FILE_FORMAT) caught = true;
    }
    if (!caught) {
        std::fprintf(stderr, "Self-check failed: invalid Dict type did not throw ERROR_CORRUPT_FILE_FORMAT\n");
        std::exit(1);
    }
    g_fs.clearFiles();
}

int main(int argc, char** argv) {
    if (argc != 2) {
        std::fprintf(stderr, "Usage: gen_pkt451_datachunk <out.json>\n");
        return 1;
    }
    check_byte_order();

    char user_prefix[128];
    std::snprintf(user_prefix, sizeof(user_prefix), "/tmp/pkt451_%ld_", static_cast<long>(getpid()));
    g_global_data.setPath_UserData(user_prefix);
    std::string tmp_chunk_path = std::string(user_prefix) + "_tmpChunk.dat";

    TheNameKeyGenerator = new NameKeyGenerator();
    TheNameKeyGenerator->init();

    run_error_self_checks();

    const char* source_paths[] = {
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/DataChunk.h",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/System/DataChunk.cpp",
        "vendor/GeneralsX/Core/GameEngine/Include/Common/MapReaderWriterInfo.h",
        "vendor/GeneralsX/Core/GameEngine/Include/Common/Dict.h",
        "vendor/GeneralsX/Core/GameEngine/Source/Common/Dict.cpp",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/NameKeyGenerator.h",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/NameKeyGenerator.cpp",
        "vendor/GeneralsX/Core/Libraries/Source/Compression/EAC/refencode.cpp",
        "vendor/GeneralsX/Core/Libraries/Source/Compression/EAC/refdecode.cpp"
    };

    const char* out_path = argv[1];
    std::string tmp_out = std::string(out_path) + ".tmp";
    FILE* fp = std::fopen(tmp_out.c_str(), "wb");
    if (!fp) { std::perror(tmp_out.c_str()); return 1; }

    std::fprintf(fp, "{\n  \"packet_id\": \"PKT-451\",\n"
                     "  \"module\": \"DataChunk\",\n"
                     "  \"oracle\": \"vendor/GeneralsX unmodified DataChunk.cpp + Dict.cpp + NameKeyGenerator.cpp + EAC REF_*\",\n"
                     "  \"source_sha256\": {\n");
    for (size_t i = 0; i < ARRAY_SIZE(source_paths); ++i) {
        std::string digest = sha256_file(source_paths[i]);
        std::fprintf(fp, "    \"%s\": \"%s\"%s\n",
                     source_paths[i], digest.c_str(),
                     (i + 1 < ARRAY_SIZE(source_paths)) ? "," : "");
    }
    std::fprintf(fp, "  },\n  \"vector_count\": 1000,\n  \"vectors\": [\n");

    const char* root_labels[] = { "HeightMapData", "BlendTileData", "ObjectsList", "PlayerScriptsList" };
    const char* child_labels[] = { "Object", "TriggerArea", "PolygonTrigger", "ScriptGroup" };
    const char* dup_labels[] = { "WorldInfo", "BuildListInfo", "HeightMapData", "Ways" };

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed * 0x9e3779b9u + 0x7f4a7c15u;
        TheNameKeyGenerator->reset();

        const char* root_label = root_labels[seed == 0 ? 0 : (splitmix32(rng) % ARRAY_SIZE(root_labels))];
        UnsignedShort root_ver = static_cast<UnsignedShort>(1u + (seed == 0 ? 0u : (splitmix32(rng) % 7u)));
        const char* child_label = child_labels[seed == 0 ? 0 : (splitmix32(rng) % ARRAY_SIZE(child_labels))];
        UnsignedShort child_ver = static_cast<UnsignedShort>(1u + (seed == 0 ? 0u : (splitmix32(rng) % 5u)));
        const char* dup_label = dup_labels[seed == 0 ? 2 : (seed == 1 ? 0 : (splitmix32(rng) % ARRAY_SIZE(dup_labels)))];
        UnsignedShort dup_ver = static_cast<UnsignedShort>(1u + (splitmix32(rng) % 4u));

        Real val_real = (seed == 0) ? bitsf(0x80000000u) :
                        static_cast<Real>(static_cast<Int>(splitmix32(rng) % 4001u) - 2000) * 0.25f;
        Int val_i32 = (seed == 0) ? INT32_MIN :
                      (seed == 1) ? INT32_MAX :
                                    static_cast<Int>(splitmix32(rng));
        Byte val_byte = (seed == 0) ? -128 :
                        (seed == 1) ? 127 :
                                      static_cast<Byte>((splitmix32(rng) & 0xFFu) - 128);

        std::string val_ascii;
        uint32_t alen = (seed == 0) ? 0u : (1u + (splitmix32(rng) % 18u));
        for (uint32_t i = 0; i < alen; ++i) {
            val_ascii.push_back(static_cast<char>('a' + (splitmix32(rng) % 26u)));
        }

        std::vector<WideChar> val_uni_units;
        uint32_t ulen = (seed == 0) ? 2u : (splitmix32(rng) % 10u);
        if (seed == 0) {
            val_uni_units = { 0x4e2d, 0x0041 }; // non-ASCII + ASCII BMP UTF-16
        } else {
            for (uint32_t i = 0; i < ulen; ++i) {
                val_uni_units.push_back(static_cast<WideChar>(0x0041u + (splitmix32(rng) % 0x4e00u)));
            }
        }
        UnicodeString val_uni;
        val_uni.setUnits(val_uni_units.data(), static_cast<Int>(val_uni_units.size()));

        uint32_t blen = (seed == 0) ? 0u : (splitmix32(rng) % 16u);
        std::vector<uint8_t> val_bytes(blen);
        for (uint32_t i = 0; i < blen; ++i) val_bytes[i] = static_cast<uint8_t>(splitmix32(rng) & 0xFFu);

        uint32_t tlen = (seed == 0) ? 5u : (splitmix32(rng) % 9u);
        std::vector<uint8_t> tail_bytes(tlen);
        for (uint32_t i = 0; i < tlen; ++i) tail_bytes[i] = static_cast<uint8_t>(splitmix32(rng) & 0xFFu);

        const char* dict_bool_k = "mapIsNight";
        Bool dict_bool_v = (splitmix32(rng) & 1u) ? TRUE : FALSE;
        const char* dict_int_k = "borderWidth";
        Int dict_int_v = static_cast<Int>(splitmix32(rng));
        const char* dict_real_k = "waterHeight";
        Real dict_real_v = static_cast<Real>(static_cast<Int>(splitmix32(rng) % 2001u) - 1000) * 0.5f;
        const char* dict_ascii_k = "mapName";
        std::string dict_ascii_v = "Map_" + std::to_string(seed);
        const char* dict_uni_k = "displayTitle";
        std::vector<WideChar> dict_uni_units = {
            static_cast<WideChar>(0x004d),
            static_cast<WideChar>(0x0416 + (seed % 32u)),
            static_cast<WideChar>(0x0030 + (seed % 10u))
        };
        UnicodeString dict_uni_v;
        dict_uni_v.setUnits(dict_uni_units.data(), static_cast<Int>(dict_uni_units.size()));
        const char* dict_namekey_name = "NameKeyTargetLabel";

        Dict d;
        d.setBool(TheNameKeyGenerator->nameToKey(dict_bool_k), dict_bool_v);
        d.setInt(TheNameKeyGenerator->nameToKey(dict_int_k), dict_int_v);
        d.setReal(TheNameKeyGenerator->nameToKey(dict_real_k), dict_real_v);
        d.setAsciiString(TheNameKeyGenerator->nameToKey(dict_ascii_k), AsciiString(dict_ascii_v.c_str()));
        d.setUnicodeString(TheNameKeyGenerator->nameToKey(dict_uni_k), dict_uni_v);
        NameKeyType nk_val = TheNameKeyGenerator->nameToKey(dict_namekey_name);

        MemOutputStream mos;
        {
            DataChunkOutput out(&mos);
            out.openDataChunk(root_label, root_ver);
            out.openDataChunk(child_label, child_ver);
            out.writeReal(val_real);
            out.writeInt(val_i32);
            out.writeByte(val_byte);
            out.writeAsciiString(AsciiString(val_ascii.c_str()));
            out.writeUnicodeString(val_uni);
            if (!val_bytes.empty()) {
                out.writeArrayOfBytes(reinterpret_cast<char*>(val_bytes.data()), static_cast<Int>(val_bytes.size()));
            }
            if (!tail_bytes.empty()) {
                out.writeArrayOfBytes(reinterpret_cast<char*>(tail_bytes.data()), static_cast<Int>(tail_bytes.size()));
            }
            out.closeDataChunk();

            out.openDataChunk(dup_label, dup_ver);
            out.writeNameKey(nk_val);
            out.writeDict(d);
            out.closeDataChunk();

            // Empty nested sibling chunk on even seeds (proves empty chunk & TOC label reuse).
            if ((seed & 1u) == 0u) {
                out.openDataChunk(child_label, child_ver);
                out.closeDataChunk();
            }
            out.closeDataChunk();
        }

        std::vector<uint8_t> raw_bytes = mos.bytes;
        std::vector<uint8_t> ear_bytes = compress_ear(raw_bytes);

        auto verify_stream = [&](const char* vname, const std::vector<uint8_t>& file_bytes) -> uint32_t {
            g_fs.clearFiles();
            g_fs.setFile(vname, file_bytes);
            CachedFileInputStream cfis;
            if (!cfis.open(AsciiString(vname))) {
                std::fprintf(stderr, "Seed %u: CachedFileInputStream open failed\n", seed);
                std::exit(2);
            }
            DataChunkInput dci(&cfis);
            if (!dci.isValidFileType()) {
                std::fprintf(stderr, "Seed %u: invalid CkMp file type\n", seed);
                std::exit(2);
            }
            ParseState st;
            st.val_bytes = val_bytes;
            dci.registerParser(AsciiString(root_label), AsciiString::TheEmptyString, parse_root, &st);
            dci.registerParser(AsciiString(child_label), AsciiString(root_label), parse_child, &st);
            // Register loser first, winner second -> winner must win under LIFO!
            dci.registerParser(AsciiString(dup_label), AsciiString(root_label), parse_dup_loser, &st);
            dci.registerParser(AsciiString(dup_label), AsciiString(root_label), parse_dup_winner, &st);
            if (!dci.parse(&st) || st.tag != 2) {
                std::fprintf(stderr, "Seed %u: parse failed or LIFO winner tag=%d != 2\n", seed, st.tag);
                std::exit(2);
            }
            cfis.close();
            return st.crc;
        };

        uint32_t crc_raw = verify_stream("test.map", raw_bytes);
        uint32_t crc_ear = verify_stream("test_comp.map", ear_bytes);
        if (crc_raw != crc_ear) {
            std::fprintf(stderr, "Seed %u: raw CRC 0x%08x != ear CRC 0x%08x\n", seed, crc_raw, crc_ear);
            return 2;
        }

        std::fprintf(fp,
            "    {\"seed\": %u, \"root_label\": \"%s\", \"root_ver\": %u, "
            "\"child_label\": \"%s\", \"child_ver\": %u, "
            "\"dup_label\": \"%s\", \"dup_ver\": %u, "
            "\"has_empty_sibling\": %u, "
            "\"val_real_bits\": \"0x%08x\", \"val_i32\": %d, \"val_byte\": %d, "
            "\"val_ascii\": \"%s\", \"val_u16_units\": [",
            seed, root_label, root_ver, child_label, child_ver, dup_label, dup_ver,
            ((seed & 1u) == 0u) ? 1u : 0u,
            fbits(val_real), val_i32, static_cast<Int>(val_byte), val_ascii.c_str());
        for (size_t i = 0; i < val_uni_units.size(); ++i) {
            std::fprintf(fp, "%u%s", static_cast<uint32_t>(val_uni_units[i]),
                         (i + 1 < val_uni_units.size()) ? ", " : "");
        }
        std::fprintf(fp,
            "], \"val_bytes_hex\": \"%s\", \"tail_hex\": \"%s\", "
            "\"dict_bool_k\": \"%s\", \"dict_bool_v\": %u, "
            "\"dict_int_k\": \"%s\", \"dict_int_v\": %d, "
            "\"dict_real_k\": \"%s\", \"dict_real_bits\": \"0x%08x\", "
            "\"dict_ascii_k\": \"%s\", \"dict_ascii_v\": \"%s\", "
            "\"dict_uni_k\": \"%s\", \"dict_uni_units\": [",
            to_hex(val_bytes.data(), val_bytes.size()).c_str(),
            to_hex(tail_bytes.data(), tail_bytes.size()).c_str(),
            dict_bool_k, static_cast<uint32_t>(dict_bool_v),
            dict_int_k, dict_int_v,
            dict_real_k, fbits(dict_real_v),
            dict_ascii_k, dict_ascii_v.c_str(),
            dict_uni_k);
        for (size_t i = 0; i < dict_uni_units.size(); ++i) {
            std::fprintf(fp, "%u%s", static_cast<uint32_t>(dict_uni_units[i]),
                         (i + 1 < dict_uni_units.size()) ? ", " : "");
        }
        std::fprintf(fp,
            "], \"dict_namekey_name\": \"%s\", \"winner_tag\": 2, "
            "\"parsed_crc_hex\": \"0x%08x\", \"raw_hex\": \"%s\", \"ear_hex\": \"%s\"}%s\n",
            dict_namekey_name, crc_raw,
            to_hex(raw_bytes.data(), raw_bytes.size()).c_str(),
            to_hex(ear_bytes.data(), ear_bytes.size()).c_str(),
            (seed + 1u < 1000u) ? "," : "");
    }

    std::fprintf(fp, "  ]\n}\n");
    if (std::fflush(fp) != 0 || std::fclose(fp) != 0) {
        std::perror("flush/close");
        std::remove(tmp_chunk_path.c_str());
        std::remove(tmp_out.c_str());
        return 1;
    }
    std::remove(tmp_chunk_path.c_str());
    if (std::rename(tmp_out.c_str(), out_path) != 0) {
        std::perror("rename");
        return 1;
    }

    delete TheNameKeyGenerator;
    TheNameKeyGenerator = nullptr;
    return 0;
}
