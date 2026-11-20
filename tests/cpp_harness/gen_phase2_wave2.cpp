// Phase 2 Wave 2 C++ Golden Vector Generator:
// - PKT-022: GeometryInfo (1,000 vectors)
// - PKT-038: Dict (1,000 vectors)
// Compiles directly against unmodified vendor/GeneralsX Geometry.cpp, Dict.cpp, RandomValue.cpp, crc.cpp
// and links vendor/GeneralsX's compiled libgm.a.

#include "PreRTS.h"
#include "Common/crc.h"
#include "Common/RandomValue.h"
#include "Common/Xfer.h"
#include "Common/Dict.h"
#include "GameLogic/GameLogic.h"

#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/crc.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/RandomValue.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/System/Geometry.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/Dict.cpp"

const AsciiString AsciiString::TheEmptyString;
const UnicodeString UnicodeString::TheEmptyString;

static DynamicMemoryAllocator g_dma;
DynamicMemoryAllocator* TheDynamicMemoryAllocator = &g_dma;

static GameLogic g_game_logic;
GameLogic* TheGameLogic = &g_game_logic;

static uint32_t splitmix32(uint32_t& state) {
    uint32_t z = (state += 0x9e3779b9u);
    z = (z ^ (z >> 16)) * 0x85ebca6bu;
    z = (z ^ (z >> 13)) * 0xc2b2ae35u;
    return z ^ (z >> 16);
}

static uint32_t fbits(Real v) {
    uint32_t b = 0;
    std::memcpy(&b, &v, 4);
    return b;
}

static uint32_t rot_crc(uint32_t crc, uint32_t word) {
    return ((crc << 1) | (crc >> 31)) + word;
}

static void generate_pkt022_geometry(const char* out_path) {
    FILE* fp = std::fopen(out_path, "wb");
    if (!fp) { std::perror(out_path); std::exit(1); }
    std::fprintf(fp, "{\n  \"packet_id\": \"PKT-022\",\n  \"module\": \"Geometry\",\n"
                     "  \"oracle\": \"vendor/GeneralsX unmodified Geometry.cpp + RandomValue.cpp + libgm.a\",\n"
                     "  \"vector_count\": 1000,\n  \"vectors\": [\n");

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed * 0x6c8e9cf5u + 0x13579BDFu;
        InitRandom(seed + 1u);

        GeometryType gtype = static_cast<GeometryType>(seed % 3u);
        Bool is_small = (splitmix32(rng) & 1u) ? TRUE : FALSE;
        Real height = static_cast<Real>((splitmix32(rng) % 200u) + 2u) * 0.5f;
        Real major_r = static_cast<Real>((splitmix32(rng) % 300u) + 2u) * 0.25f;
        Real minor_r = static_cast<Real>((splitmix32(rng) % 300u) + 2u) * 0.25f;
        Real angle = static_cast<Real>(static_cast<int32_t>(splitmix32(rng) % 6283u) - 3141) * 0.001f;

        GeometryInfo geom(gtype, is_small, height, major_r, minor_r);
        if ((seed & 7u) == 3u) {
            geom.expandFootprint(2.5f);
        } else if ((seed & 7u) == 5u) {
            geom.setMaxHeightAbovePosition(height + 4.0f);
        }

        auto coord_val = [&](Real scale, Int off) -> Real {
            return static_cast<Real>(static_cast<int32_t>(splitmix32(rng) % 401u) - off) * scale;
        };

        Coord3D center_in = {coord_val(1.0f, 200), coord_val(1.0f, 200), coord_val(0.5f, 0)};
        Coord3D center_out = {0.0f, 0.0f, 0.0f};
        geom.getCenterPosition(center_in, center_out);

        Region2D bounds2d{};
        geom.get2DBounds(center_in, angle, bounds2d);

        Coord3D test_pt = {
            center_in.x + coord_val(0.5f, 200),
            center_in.y + coord_val(0.5f, 200),
            center_in.z
        };
        Bool in_fp = geom.isPointInFootprint(center_in, test_pt);
        Coord3D clipped_pt = test_pt;
        geom.clipPointToFootprint(center_in, clipped_pt);

        Coord3D seg_from = {center_in.x + coord_val(1.0f, 200), center_in.y + coord_val(1.0f, 200), center_in.z + coord_val(0.5f, 100)};
        Coord3D seg_to = {center_in.x + coord_val(1.0f, 200), center_in.y + coord_val(1.0f, 200), center_in.z + coord_val(0.5f, 100)};
        Bool hit_seg = geom.isIntersectedByLineSegment(center_in, seg_from, seg_to);

        GeometryInfo that_geom(
            static_cast<GeometryType>((seed + 1u) % 3u),
            FALSE,
            height * 0.75f,
            major_r * 0.5f,
            minor_r * 0.5f
        );
        Coord3D that_pos = {center_in.x + 25.0f, center_in.y - 15.0f, center_in.z + 10.0f};
        Real min_pitch = 0.0f, max_pitch = 0.0f;
        geom.calcPitches(center_in, that_geom, that_pos, min_pitch, max_pitch);

        Coord3D rand_fp = {0.0f, 0.0f, 0.0f};
        geom.makeRandomOffsetWithinFootprint(rand_fp);
        Coord3D rand_perim = {0.0f, 0.0f, 0.0f};
        geom.makeRandomOffsetOnPerimeter(rand_perim);
        UnsignedInt rng_crc = GetGameLogicRandomSeedCRC();

        HarnessXferCRC xcrc;
        reinterpret_cast<Snapshot*>(&geom)->xfer(&xcrc);

        std::fprintf(
            fp,
            "    {\"seed\": %u, \"gtype\": %d, \"is_small\": %u, "
            "\"height_bits\": %u, \"major_bits\": %u, \"minor_bits\": %u, \"angle_bits\": %u, "
            "\"center_in_bits\": [%u,%u,%u], \"test_pt_bits\": [%u,%u,%u], "
            "\"seg_from_bits\": [%u,%u,%u], \"seg_to_bits\": [%u,%u,%u], "
            "\"bcircle_bits\": %u, \"bsphere_bits\": %u, \"area_bits\": %u, "
            "\"max_above_bits\": %u, \"max_below_bits\": %u, \"zdelta_bits\": %u, \"center_z_bits\": %u, "
            "\"bounds_bits\": [%u,%u,%u,%u], \"in_fp\": %u, \"clip_bits\": [%u,%u], \"hit_seg\": %u, "
            "\"min_pitch_bits\": %u, \"max_pitch_bits\": %u, "
            "\"rand_fp_bits\": [%u,%u], \"rand_perim_bits\": [%u,%u], "
            "\"rng_crc\": %u, \"xfer_crc\": %u}%s\n",
            seed, static_cast<int>(gtype), static_cast<unsigned>(is_small),
            fbits(height), fbits(major_r), fbits(minor_r), fbits(angle),
            fbits(center_in.x), fbits(center_in.y), fbits(center_in.z),
            fbits(test_pt.x), fbits(test_pt.y), fbits(test_pt.z),
            fbits(seg_from.x), fbits(seg_from.y), fbits(seg_from.z),
            fbits(seg_to.x), fbits(seg_to.y), fbits(seg_to.z),
            fbits(geom.getBoundingCircleRadius()), fbits(geom.getBoundingSphereRadius()),
            fbits(geom.getFootprintArea()),
            fbits(geom.getMaxHeightAbovePosition()), fbits(geom.getMaxHeightBelowPosition()),
            fbits(geom.getZDeltaToCenterPosition()), fbits(center_out.z),
            fbits(bounds2d.lo.x), fbits(bounds2d.lo.y), fbits(bounds2d.hi.x), fbits(bounds2d.hi.y),
            static_cast<unsigned>(in_fp), fbits(clipped_pt.x), fbits(clipped_pt.y),
            static_cast<unsigned>(hit_seg),
            fbits(min_pitch), fbits(max_pitch),
            fbits(rand_fp.x), fbits(rand_fp.y), fbits(rand_perim.x), fbits(rand_perim.y),
            rng_crc, xcrc.getCRC(),
            (seed + 1 < 1000) ? "," : ""
        );
    }
    std::fprintf(fp, "  ]\n}\n");
    std::fclose(fp);
}

static void generate_pkt038_dict(const char* out_path) {
    FILE* fp = std::fopen(out_path, "wb");
    if (!fp) { std::perror(out_path); std::exit(1); }
    std::fprintf(fp, "{\n  \"packet_id\": \"PKT-038\",\n  \"module\": \"Dict\",\n"
                     "  \"oracle\": \"vendor/GeneralsX unmodified Dict.cpp\",\n"
                     "  \"vector_count\": 1000,\n  \"vectors\": [\n");

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed * 0x94d049bbu + 0x2468ACE0u;
        Dict d;
        Dict d_other;

        if (seed == 0) {
            // Seed 0 exercises clear(), empty Dict queries, and copyPairFrom removal
            d.setInt(static_cast<NameKeyType>(5), 42);
            d.clear();
        } else {
            for (int step = 0; step < 16; ++step) {
                uint32_t op = splitmix32(rng);
                NameKeyType key = static_cast<NameKeyType>((op % 12u) + 1u);
                uint32_t kind = (op >> 8) % 7u;
                if (kind == 0) {
                    d.setBool(key, ((op >> 16) & 1u) ? TRUE : FALSE);
                } else if (kind == 1) {
                    d.setInt(key, static_cast<Int>(op >> 4) - 50000);
                } else if (kind == 2) {
                    Real rv = static_cast<Real>(static_cast<int32_t>((op >> 4) % 10001u) - 5000) * 0.125f;
                    d.setReal(key, rv);
                } else if (kind == 3) {
                    char buf[24];
                    std::snprintf(buf, sizeof(buf), "V_%u_%d", static_cast<unsigned>(key), step);
                    d.setAsciiString(key, AsciiString(buf));
                } else if (kind == 4) {
                    WideChar wbuf[5] = {
                        static_cast<WideChar>('U'),
                        static_cast<WideChar>('0' + (key % 10)),
                        static_cast<WideChar>(0x0410u + step),
                        0
                    };
                    UnicodeString u;
                    u.setUnits(wbuf, 3);
                    d.setUnicodeString(key, u);
                } else if (kind == 5) {
                    d.remove(key);
                } else {
                    if ((op & 0x200u) != 0) {
                        d_other.setInt(key, static_cast<Int>(seed * 100u + step));
                    } else {
                        d_other.remove(key);
                    }
                    d.copyPairFrom(d_other, key);
                }
            }
        }

        // Copy-on-write clone check: mutating clone must NOT mutate d
        Dict d_clone = d;
        d_clone.setInt(static_cast<NameKeyType>(13), static_cast<Int>(seed));

        Int pair_count = d.getPairCount();
        uint32_t state_crc = 0;
        for (Int i = 0; i < pair_count; ++i) {
            NameKeyType k = d.getNthKey(i);
            Dict::DataType dt = d.getNthType(i);
            state_crc = rot_crc(state_crc, static_cast<uint32_t>(k));
            state_crc = rot_crc(state_crc, static_cast<uint32_t>(dt));
            if (dt == Dict::DICT_BOOL) {
                state_crc = rot_crc(state_crc, static_cast<uint32_t>(d.getNthBool(i)));
            } else if (dt == Dict::DICT_INT) {
                state_crc = rot_crc(state_crc, static_cast<uint32_t>(d.getNthInt(i)));
            } else if (dt == Dict::DICT_REAL) {
                state_crc = rot_crc(state_crc, fbits(d.getNthReal(i)));
            } else if (dt == Dict::DICT_ASCIISTRING) {
                AsciiString s = d.getNthAsciiString(i);
                state_crc = rot_crc(state_crc, static_cast<uint32_t>(s.getLength()));
                for (int c = 0; c < s.getLength(); ++c) {
                    state_crc = rot_crc(state_crc, static_cast<uint8_t>(s.str()[c]));
                }
            } else if (dt == Dict::DICT_UNICODESTRING) {
                UnicodeString u = d.getNthUnicodeString(i);
                state_crc = rot_crc(state_crc, static_cast<uint32_t>(u.getLength()));
                for (int c = 0; c < u.getLength(); ++c) {
                    state_crc = rot_crc(state_crc, static_cast<uint32_t>(u.str()[c]));
                }
            }
        }

        NameKeyType probe_key = static_cast<NameKeyType>((seed % 12u) + 1u);
        Dict::DataType probe_type = d.getType(probe_key);
        Bool exists = FALSE;
        Int probe_int = d.getInt(probe_key, &exists);

        std::fprintf(
            fp,
            "    {\"seed\": %u, \"pair_count\": %d, \"clone_count\": %d, \"state_crc\": %u, "
            "\"probe_key\": %d, \"probe_type\": %d, \"probe_int\": %d, \"probe_exists\": %u}%s\n",
            seed, pair_count, d_clone.getPairCount(), state_crc,
            static_cast<int>(probe_key), static_cast<int>(probe_type), probe_int, static_cast<unsigned>(exists),
            (seed + 1 < 1000) ? "," : ""
        );
    }
    std::fprintf(fp, "  ]\n}\n");
    std::fclose(fp);
}

int main() {
    generate_pkt022_geometry("docs/golden_vectors/PKT-022.json");
    generate_pkt038_dict("docs/golden_vectors/PKT-038.json");
    std::printf("Generated 2,000 C++ Golden Test Vectors across PKT-022 and PKT-038.\n");
    return 0;
}
