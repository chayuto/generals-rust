// WP-7.2b C++ Golden Vector Generator for the EA (EAC) codecs behind CompressionManager
// (Core/Libraries/Source/Compression/CompressionManager.cpp):
//   COMPRESSION_REFPACK ("EAR\0") -> REF_encode   / REF_decode
//   COMPRESSION_BTREE   ("EAB\0") -> BTREE_encode / BTREE_decode
//   COMPRESSION_HUFF    ("EAH\0") -> HUFF_encode  / HUFF_decode
// Links the unmodified vendor/GeneralsX EAC sources, compiled as separate translation units
// (their codex headers have no include guards).
//
// Byte order: gimex.h's ggetm()/gputm() take a native-endian shortcut when __APPLE__ is
// defined, a PowerPC-era assumption. On little-endian Apple Silicon that writes byte-swapped
// stream headers (fb 10 instead of 10 fb) which REF_decode, parsing bytes by hand, then
// misreads. The canonical format used by Windows/Linux builds and retail .big/.map data is
// big-endian, so the EAC sources are compiled with stubs_wave4_eac/eac_portable_byteorder.h
// force-included, which undefines __APPLE__ after the system headers.
//
// Build & run (from repo root; add -fsanitize=address,undefined to every command to check
// the reference codecs for memory errors):
//   EAC=vendor/GeneralsX/Core/Libraries/Source/Compression/EAC; mkdir -p /tmp/wp72b
//   for f in refencode refdecode btreeencode btreedecode huffencode huffdecode; do
//     c++ -std=c++17 -O1 -ffp-contract=off \
//         -include tests/cpp_harness/stubs_wave4_eac/eac_portable_byteorder.h \
//         -c $EAC/$f.cpp -o /tmp/wp72b/$f.o; done
//   c++ -std=c++17 -O1 -ffp-contract=off -I$EAC tests/cpp_harness/gen_wave4_compression.cpp \
//       /tmp/wp72b/*.o -o /tmp/wp72b/gen_wave4_compression
//   /tmp/wp72b/gen_wave4_compression docs/golden_vectors/WP7_2b_Compression.json
//
// Output: 1,000 small vectors with full hex blobs (seeds 0..999), plus large vectors
// (seeds 1000+) that store FNV-1a 64 digests; their payloads are regenerated from
// (seed, len, pattern, period) by crates/zh_common/tests/golden_wp7_2b_compression.rs.

#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <vector>

#include "refcodex.h"
#include "btreecodex.h"
#include "huffcodex.h"

static uint32_t splitmix32(uint32_t& s) {
    uint32_t z = (s += 0x9e3779b9u);
    z = (z ^ (z >> 16)) * 0x85ebca6bu;
    z = (z ^ (z >> 13)) * 0xc2b2ae35u;
    return z ^ (z >> 16);
}

static uint64_t fnv1a64(const std::vector<uint8_t>& buf) {
    uint64_t h = 0xcbf29ce484222325ull;
    for (uint8_t b : buf) {
        h ^= b;
        h *= 0x100000001b3ull;
    }
    return h;
}

static void fprint_hex(FILE* fp, const std::vector<uint8_t>& buf) {
    for (uint8_t b : buf) {
        std::fprintf(fp, "%02x", b);
    }
}

// Patterns 0..4 are used by small and large vectors; pattern 5 only by large vectors.
// The Rust golden test ports this function exactly, including when `rng` advances.
static std::vector<uint8_t> make_payload(uint32_t seed, uint32_t len, uint32_t pattern,
                                         uint32_t period, uint32_t& rng) {
    std::vector<uint8_t> out(len);
    if (pattern == 0u) {
        // Constant runs of 32 bytes
        uint8_t b = static_cast<uint8_t>(splitmix32(rng) & 0xffu);
        for (uint32_t i = 0; i < len; ++i) {
            if ((i & 31u) == 0u) b = static_cast<uint8_t>(splitmix32(rng) & 0xffu);
            out[i] = b;
        }
    } else if (pattern == 1u) {
        // Repetitive INI-like text (period 68)
        static const char kSample[] = "Object AmericaTankCrusader\n  BuildCost = 900\n  BuildTime = 10.0\nEnd\n";
        for (uint32_t i = 0; i < len; ++i) {
            out[i] = static_cast<uint8_t>(kSample[(i + seed) % (sizeof(kSample) - 1u)]);
        }
    } else if (pattern == 2u) {
        // Structured map-like counters
        for (uint32_t i = 0; i < len; ++i) {
            out[i] = static_cast<uint8_t>((i & 3u) == 0 ? (i >> 2) : ((seed >> (i & 3u)) & 0xffu));
        }
    } else if (pattern == 3u) {
        // Mixed run + noise (high back-reference density)
        for (uint32_t i = 0; i < len; ++i) {
            out[i] = (i % 7u < 4u)
                ? static_cast<uint8_t>((i / 7u) & 0xffu)
                : static_cast<uint8_t>(splitmix32(rng) & 0xffu);
        }
    } else if (pattern == 4u) {
        // Incompressible noise
        for (uint32_t i = 0; i < len; ++i) {
            out[i] = static_cast<uint8_t>(splitmix32(rng) & 0xffu);
        }
    } else {
        // Far repeat: two of every three 1 KiB blocks copy the byte `period` back, so the
        // only matches sit at a long distance (RefPack 4-byte form, 128K window edges).
        for (uint32_t i = 0; i < len; ++i) {
            if (i >= period && ((i >> 10) % 3u) != 0u) {
                out[i] = out[i - period];
            } else {
                out[i] = static_cast<uint8_t>(splitmix32(rng) & 0xffu);
            }
        }
    }
    return out;
}

typedef int (*EncodeFn)(void*, const void*, int, int*);
typedef int (*DecodeFn)(void*, const void*, int*);

// The Huffman decoder refills its bit buffer 16 bits at a time and so reads a few bytes past
// the end of a valid stream (the engine always decodes from a larger file buffer). Each
// stream is therefore decoded from a copy followed by kDecodePad bytes of 0x00 and then
// 0xff padding; both must reproduce the source, proving the padding never reaches the output.
static const size_t kDecodePad = 64u;

static void verify_decode(const char magic[4], const std::vector<uint8_t>& src,
                          const std::vector<uint8_t>& stream, uint8_t pad_byte, DecodeFn decode_fn) {
    int src_len = static_cast<int>(src.size());
    std::vector<uint8_t> padded(stream);
    padded.resize(stream.size() + kDecodePad, pad_byte);
    std::vector<uint8_t> dec(static_cast<size_t>(src_len) + 64u, 0);
    int consumed = 0;
    int decoded_len = decode_fn(dec.data(), padded.data(), &consumed);
    if (decoded_len != src_len ||
        (src_len > 0 && std::memcmp(dec.data(), src.data(), src.size()) != 0)) {
        std::fprintf(stderr, "FATAL: %.3s round trip failed len=%d decoded=%d pad=0x%02x\n",
                     magic, src_len, decoded_len, pad_byte);
        std::exit(2);
    }
}

// Mirrors CompressionManager::compressData for an EAC type: "EAx\0" magic, little-endian Int
// uncompressed length, then the codec stream. An encoder result of 0 is the manager's error
// path (returns 0), recorded as an empty blob.
static std::vector<uint8_t> compress_like_manager(const char magic[4],
                                                  const std::vector<uint8_t>& src,
                                                  EncodeFn encode_fn, DecodeFn decode_fn) {
    static const uint8_t kEmpty[1] = {0};
    const uint8_t* src_ptr = src.empty() ? kEmpty : src.data();
    int src_len = static_cast<int>(src.size());

    std::vector<uint8_t> enc(static_cast<size_t>(src_len) * 2u + 2048u + 8u, 0);
    std::memcpy(enc.data(), magic, 4);
    int ret = encode_fn(enc.data() + 8, src_ptr, src_len, nullptr);
    if (ret <= 0) {
        if (src_len == 0) {
            return {};
        }
        std::fprintf(stderr, "FATAL: %.3s encode returned %d for len=%d\n", magic, ret, src_len);
        std::exit(2);
    }
    uint32_t ulen = static_cast<uint32_t>(src_len);
    enc[4] = static_cast<uint8_t>(ulen & 0xffu);
    enc[5] = static_cast<uint8_t>((ulen >> 8) & 0xffu);
    enc[6] = static_cast<uint8_t>((ulen >> 16) & 0xffu);
    enc[7] = static_cast<uint8_t>((ulen >> 24) & 0xffu);
    enc.resize(8u + static_cast<size_t>(ret));

    std::vector<uint8_t> stream(enc.begin() + 8, enc.end());
    // HUFF_encode turns an empty input into a stream that HUFF_decode never terminates on: it
    // reads unboundedly past the buffer. The encoder output is still recorded, but that one
    // decode has no defined reference behaviour and is not checked.
    bool huff_empty = (src_len == 0 && std::memcmp(magic, "EAH", 3) == 0);
    if (!huff_empty) {
        verify_decode(magic, src, stream, 0x00u, decode_fn);
        verify_decode(magic, src, stream, 0xffu, decode_fn);
    }
    return enc;
}

struct LargeCase {
    uint32_t len;
    uint32_t pattern;
    uint32_t period;
};

static const LargeCase kLargeCases[] = {
    {16400u, 5u, 16383u},    // offsets straddling the 16384 three/four-byte boundary
    {16400u, 5u, 16385u},
    {40000u, 5u, 30000u},
    {70000u, 5u, 65537u},
    {140000u, 5u, 131000u},  // just inside the 131071 back-reference window
    {140000u, 5u, 131071u},
    {140000u, 5u, 131072u},  // just outside the window
    {200000u, 5u, 140000u},
    {140000u, 1u, 0u},       // dense short matches; link[] wraps at 131072
    {140000u, 3u, 0u},
    {300000u, 0u, 0u},
    {300000u, 4u, 0u},
};

static const uint32_t kSmallCount = 1000u;
static const uint32_t kLargeCount = sizeof(kLargeCases) / sizeof(kLargeCases[0]);

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "Usage: %s <out.json>\n", argv[0]);
        return 1;
    }

    FILE* fp = std::fopen(argv[1], "wb");
    if (!fp) { std::perror(argv[1]); return 1; }

    std::fprintf(fp, "{\n"
                     "  \"work_package\": \"WP-7.2b\",\n"
                     "  \"module\": \"EACompression\",\n"
                     "  \"reference_commit\": \"84a7a20769aa5749f388b1f49bd542c79b3c33e8\",\n"
                     "  \"oracle\": \"vendor/GeneralsX Core/Libraries/Source/Compression/EAC/{ref,btree,huff}{encode,decode}.cpp\",\n"
                     "  \"byte_order\": \"portable big-endian ggetm/gputm (gimex.h __APPLE__ branch disabled)\",\n"
                     "  \"blob_layout\": \"magic[4] + uncompressed_len (LE i32) + codec stream; empty = compressData error\",\n"
                     "  \"vector_count\": %u,\n"
                     "  \"large_vector_count\": %u,\n"
                     "  \"vectors\": [\n",
                 kSmallCount, kLargeCount);

    static const uint32_t kEdgeLens[] = {
        0, 1, 2, 3, 4, 5, 7, 8, 9, 15, 16, 17, 31, 32, 64, 128, 256, 384, 512
    };
    const uint32_t kEdgeCount = sizeof(kEdgeLens) / sizeof(kEdgeLens[0]);

    for (uint32_t seed = 0; seed < kSmallCount; ++seed) {
        uint32_t rng = seed * 0x9e3779b9u + 0x517cc1b7u;
        uint32_t len = (seed < 2u * kEdgeCount) ? kEdgeLens[seed % kEdgeCount]
                                                : (splitmix32(rng) % 257u);
        std::vector<uint8_t> src = make_payload(seed, len, seed % 5u, 0u, rng);

        std::vector<uint8_t> ref_blob = compress_like_manager("EAR\0", src, REF_encode, REF_decode);
        std::vector<uint8_t> btree_blob = compress_like_manager("EAB\0", src, BTREE_encode, BTREE_decode);
        std::vector<uint8_t> huff_blob = compress_like_manager("EAH\0", src, HUFF_encode, HUFF_decode);

        std::fprintf(fp, "    {\"seed\":%u,\"len\":%u,\"pattern\":%u,\"src_hex\":\"",
                     seed, len, seed % 5u);
        fprint_hex(fp, src);
        std::fprintf(fp, "\",\"ref_hex\":\"");
        fprint_hex(fp, ref_blob);
        std::fprintf(fp, "\",\"btree_hex\":\"");
        fprint_hex(fp, btree_blob);
        std::fprintf(fp, "\",\"huff_hex\":\"");
        fprint_hex(fp, huff_blob);
        std::fprintf(fp, "\"},\n");
    }

    for (uint32_t k = 0; k < kLargeCount; ++k) {
        const LargeCase& c = kLargeCases[k];
        uint32_t seed = kSmallCount + k;
        uint32_t rng = seed * 0x9e3779b9u + 0x517cc1b7u;
        std::vector<uint8_t> src = make_payload(seed, c.len, c.pattern, c.period, rng);

        std::vector<uint8_t> ref_blob = compress_like_manager("EAR\0", src, REF_encode, REF_decode);
        std::vector<uint8_t> btree_blob = compress_like_manager("EAB\0", src, BTREE_encode, BTREE_decode);
        std::vector<uint8_t> huff_blob = compress_like_manager("EAH\0", src, HUFF_encode, HUFF_decode);

        std::fprintf(fp,
            "    {\"seed\":%u,\"len\":%u,\"pattern\":%u,\"period\":%u,\"src_fnv\":\"%016llx\","
            "\"ref_len\":%zu,\"ref_fnv\":\"%016llx\",\"btree_len\":%zu,\"btree_fnv\":\"%016llx\","
            "\"huff_len\":%zu,\"huff_fnv\":\"%016llx\"}%s\n",
            seed, c.len, c.pattern, c.period,
            static_cast<unsigned long long>(fnv1a64(src)),
            ref_blob.size(), static_cast<unsigned long long>(fnv1a64(ref_blob)),
            btree_blob.size(), static_cast<unsigned long long>(fnv1a64(btree_blob)),
            huff_blob.size(), static_cast<unsigned long long>(fnv1a64(huff_blob)),
            (k + 1u < kLargeCount) ? "," : "");
    }

    std::fprintf(fp, "  ]\n}\n");
    std::fclose(fp);
    std::fprintf(stderr, "Generated %u small + %u large WP-7.2b compression vectors to %s\n",
                 kSmallCount, kLargeCount, argv[1]);
    return 0;
}
