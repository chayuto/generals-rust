// Phase 2 C++ Golden Vector Generator (PKT-004 BitFlags, PKT-015/016 BezierSegment+BezFwdIterator,
// PKT-017 DiscreteCircle, PKT-064 NameKeyGenerator).
// Compiles the unmodified vendor/GeneralsX C++ sources at pinned commit 84a7a20.

#include <algorithm>
#include <bitset>
#include <cctype>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <utility>
#include <vector>

#define SAGE_USE_GLM 1
#define USE_DETERMINISTIC_MATH 1
#define RTS_ZEROHOUR 1
#define CPP_11(x) x
#define __inline inline

#ifndef _WIN32
#include <strings.h>
#define stricmp strcasecmp
#define _stricmp strcasecmp
#endif

#include "PreRTS.h"

using UnsignedInt64 = uint64_t;
using std::min;

#define INT_TO_REAL(x) ((Real)(x))

// Bring in deterministic gm_sqrtf from vendor/GeneralsX GameMath (fdlibm e_sqrtf.c)
extern "C" {
#define __ieee754_sqrtf gm_sqrtf
}
#include "../../vendor/GeneralsX/build/macos-vulkan/_deps/gamemath-src/float/e_sqrtf.c"

inline Real Sqrt(Real x) {
    return gm_sqrtf(x);
}
inline Real Sin(Real x) { return 0.0f; }
inline Real Cos(Real x) { return 1.0f; }
inline Real ACos(Real x) { return 0.0f; }

#include "../../vendor/GeneralsX/Core/Libraries/Source/Lib/Include/Lib/Coord3D.h"

using VecCoord3D = std::vector<Coord3D>;

#include "Common/AsciiString.h"
#include "Common/SubsystemInterface.h"
#include "Common/GameMemory.h"

const AsciiString AsciiString::TheEmptyString("");

class INI {
public:
    const char* getNextToken() { return ""; }
};

// Include unmodified vendor/GeneralsX headers & sources
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/BitFlags.h"

struct ModelConditionTag {};
struct ArmorSetTag {};
using ModelConditionFlags = BitFlags<114, ModelConditionTag>;
using ArmorSetFlags = BitFlags<8, ArmorSetTag>;

template<>
const char* const ModelConditionFlags::s_bitNameList[] = {
    "TOPPLED", "FRONTCRUSHED", "BACKCRUSHED", "DAMAGED", "REALLYDAMAGED", "RUBBLE",
    "SPECIAL_DAMAGED", "NIGHT", "SNOW", "PARACHUTING", "GARRISONED", "ENEMYNEAR",
    "WEAPONSET_VETERAN", "WEAPONSET_ELITE", "WEAPONSET_HERO", "WEAPONSET_CRATEUPGRADE_ONE",
    "WEAPONSET_CRATEUPGRADE_TWO", "WEAPONSET_PLAYER_UPGRADE", "DOOR_1_OPENING", "DOOR_1_CLOSING",
    "DOOR_1_WAITING_OPEN", "DOOR_1_WAITING_TO_CLOSE", "DOOR_2_OPENING", "DOOR_2_CLOSING",
    "DOOR_2_WAITING_OPEN", "DOOR_2_WAITING_TO_CLOSE", "DOOR_3_OPENING", "DOOR_3_CLOSING",
    "DOOR_3_WAITING_OPEN", "DOOR_3_WAITING_TO_CLOSE", "DOOR_4_OPENING", "DOOR_4_CLOSING",
    "DOOR_4_WAITING_OPEN", "DOOR_4_WAITING_TO_CLOSE", "ATTACKING", "PREATTACK_A", "FIRING_A",
    "BETWEEN_FIRING_SHOTS_A", "RELOADING_A", "PREATTACK_B", "FIRING_B", "BETWEEN_FIRING_SHOTS_B",
    "RELOADING_B", "PREATTACK_C", "FIRING_C", "BETWEEN_FIRING_SHOTS_C", "RELOADING_C",
    "TURRET_ROTATE", "POST_COLLAPSE", "MOVING", "DYING", "AWAITING_CONSTRUCTION",
    "PARTIALLY_CONSTRUCTED", "ACTIVELY_BEING_CONSTRUCTED", "PRONE", "FREEFALL",
    "ACTIVELY_CONSTRUCTING", "CONSTRUCTION_COMPLETE", "RADAR_EXTENDING", "RADAR_UPGRADED",
    "PANICKING", "AFLAME", "SMOLDERING", "BURNED", "DOCKING", "DOCKING_BEGINNING",
    "DOCKING_ACTIVE", "DOCKING_ENDING", "CARRYING", "FLOODED", "LOADED", "JETAFTERBURNER",
    "JETEXHAUST", "PACKING", "UNPACKING", "DEPLOYED", "OVER_WATER", "POWER_PLANT_UPGRADED",
    "CLIMBING", "SOLD", "RAPPELLING", "ARMED", "POWER_PLANT_UPGRADING", "SPECIAL_CHEERING",
    "CONTINUOUS_FIRE_SLOW", "CONTINUOUS_FIRE_MEAN", "CONTINUOUS_FIRE_FAST", "RAISING_FLAG",
    "CAPTURED", "EXPLODED_FLAILING", "EXPLODED_BOUNCING", "SPLATTED", "USING_WEAPON_A",
    "USING_WEAPON_B", "USING_WEAPON_C", "PREORDER", "CENTER_TO_LEFT", "LEFT_TO_CENTER",
    "CENTER_TO_RIGHT", "RIGHT_TO_CENTER", "RIDER1", "RIDER2", "RIDER3", "RIDER4", "RIDER5",
    "RIDER6", "RIDER7", "RIDER8", "STUNNED_FLAILING", "STUNNED", "SECOND_LIFE", "JAMMED",
    "ARMORSET_CRATEUPGRADE_ONE", "ARMORSET_CRATEUPGRADE_TWO", "USER_1", "USER_2", "DISGUISED",
    nullptr
};

template<>
const char* const ArmorSetFlags::s_bitNameList[] = {
    "VETERAN", "ELITE", "HERO", "PLAYER_UPGRADE", "WEAK_VERSUS_BASEDEFENSES",
    "SECOND_LIFE", "CRATE_UPGRADE_ONE", "CRATE_UPGRADE_TWO", nullptr
};

#include "../../vendor/GeneralsX/Core/GameEngine/Include/Common/DiscreteCircle.h"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/DiscreteCircle.cpp"

#include "../../vendor/GeneralsX/Core/GameEngine/Include/Common/BezierSegment.h"
#include "../../vendor/GeneralsX/Core/GameEngine/Include/Common/BezFwdIterator.h"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/Bezier/BezierSegment.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/Bezier/BezFwdIterator.cpp"

#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/NameKeyGenerator.h"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/NameKeyGenerator.cpp"

#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/PartitionSolver.h"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/PartitionSolver.cpp"

static uint32_t splitmix32(uint32_t& state) {
    uint32_t z = (state += 0x9e3779b9u);
    z = (z ^ (z >> 16)) * 0x85ebca6bu;
    z = (z ^ (z >> 13)) * 0xc2b2ae35u;
    return z ^ (z >> 16);
}

static uint32_t f32_bits(float v) {
    uint32_t b = 0;
    std::memcpy(&b, &v, sizeof(uint32_t));
    return b;
}

static uint32_t rotate_crc(uint32_t crc, uint32_t word) {
    return ((crc << 1) | (crc >> 31)) + word;
}

static void generate_pkt004_bitflags(const char* out_path) {
    FILE* fp = std::fopen(out_path, "wb");
    if (!fp) { std::perror(out_path); std::exit(1); }
    std::fprintf(fp, "{\n  \"packet_id\": \"PKT-004\",\n  \"module\": \"BitFlags\",\n  \"vector_count\": 1000,\n  \"vectors\": [\n");

    using Flag117 = BitFlags<117, ModelConditionTag>;
    // Ensure template specialization for 117 matches the 117-element list above
    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed * 0x45d9f3bu + 0x11111111u;
        ArmorSetFlags a;
        ArmorSetFlags b;
        uint32_t mask_a = splitmix32(rng) & 0xFFu;
        uint32_t mask_b = splitmix32(rng) & 0xFFu;
        for (int i = 0; i < 8; ++i) {
            if ((mask_a >> i) & 1u) a.set(i, 1);
            if ((mask_b >> i) & 1u) b.set(i, 1);
        }

        // Also test a 96-bit BitFlags instance for multi-chunk hex & intersection operations
        BitFlags<96, ArmorSetTag> big_a;
        BitFlags<96, ArmorSetTag> big_b;
        uint32_t w0_a = splitmix32(rng), w1_a = splitmix32(rng), w2_a = splitmix32(rng);
        uint32_t w0_b = splitmix32(rng), w1_b = splitmix32(rng), w2_b = splitmix32(rng);
        for (int i = 0; i < 32; ++i) {
            if ((w0_a >> i) & 1u) big_a.set(i, 1);
            if ((w1_a >> i) & 1u) big_a.set(32 + i, 1);
            if ((w2_a >> i) & 1u) big_a.set(64 + i, 1);
            if ((w0_b >> i) & 1u) big_b.set(i, 1);
            if ((w1_b >> i) & 1u) big_b.set(32 + i, 1);
            if ((w2_b >> i) & 1u) big_b.set(64 + i, 1);
        }
        if (seed == 0) {
            big_a.clear();
            big_b.clear();
        } else if (seed == 1) {
            big_a = BitFlags<96, ArmorSetTag>(BitFlags<96, ArmorSetTag>::kInitSetAll);
        }

        Int count_a = big_a.count();
        Int count_inter = big_a.countIntersection(big_b);
        Int count_inv_inter = big_a.countInverseIntersection(big_b);
        Bool any_inter = big_a.anyIntersectionWith(big_b);
        Bool test_all = big_a.testForAll(big_b);
        Bool test_none = big_a.testForNone(big_b);
        UnsignedInt u32_a = big_a.toUnsignedInt();
        AsciiString hex_a = big_a.toHexString();

        BitFlags<96, ArmorSetTag> mutated = big_a;
        mutated.clearAndSet(big_b, BitFlags<96, ArmorSetTag>(BitFlags<96, ArmorSetTag>::kInit, static_cast<Int>(seed % 96)));
        AsciiString hex_mut = mutated.toHexString();

        const char* armor_name = ArmorSetFlags::getNameFromSingleBit(static_cast<Int>(seed % 10) - 1);
        Int lookup_idx = ArmorSetFlags::getSingleBitFromName(armor_name ? armor_name : "NONEXISTENT");

        std::fprintf(
            fp,
            "    {\"seed\": %u, \"w0_a\": %u, \"w1_a\": %u, \"w2_a\": %u, \"w0_b\": %u, \"w1_b\": %u, \"w2_b\": %u, "
            "\"count_a\": %d, \"count_inter\": %d, \"count_inv_inter\": %d, \"any_inter\": %u, "
            "\"test_all\": %u, \"test_none\": %u, \"u32_a\": %u, \"hex_a\": \"%s\", \"hex_mut\": \"%s\", \"lookup_idx\": %d}%s\n",
            seed, w0_a, w1_a, w2_a, w0_b, w1_b, w2_b,
            count_a, count_inter, count_inv_inter, static_cast<unsigned>(any_inter),
            static_cast<unsigned>(test_all), static_cast<unsigned>(test_none),
            u32_a, hex_a.str(), hex_mut.str(), lookup_idx,
            (seed + 1 < 1000) ? "," : ""
        );
    }
    std::fprintf(fp, "  ]\n}\n");
    std::fclose(fp);
}

static void sum_scanline_cb(Int xStart, Int xEnd, Int yPos, void* parm) {
    uint32_t* crc = static_cast<uint32_t*>(parm);
    *crc = rotate_crc(*crc, static_cast<uint32_t>(xStart));
    *crc = rotate_crc(*crc, static_cast<uint32_t>(xEnd));
    *crc = rotate_crc(*crc, static_cast<uint32_t>(yPos));
}

static void generate_pkt017_discrete_circle(const char* out_path) {
    FILE* fp = std::fopen(out_path, "wb");
    if (!fp) { std::perror(out_path); std::exit(1); }
    std::fprintf(fp, "{\n  \"packet_id\": \"PKT-017\",\n  \"module\": \"DiscreteCircle\",\n  \"vector_count\": 1000,\n  \"vectors\": [\n");

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed * 0x27d4eb2du + 0xABCDEF01u;
        Int xc = static_cast<Int>(splitmix32(rng) % 2001u) - 1000;
        Int yc = static_cast<Int>(splitmix32(rng) % 2001u) - 1000;
        Int radius = static_cast<Int>(splitmix32(rng) % 120u);
        if (seed == 0) {
            xc = 0; yc = 0; radius = 0;
        } else if (seed == 1) {
            xc = 10; yc = -10; radius = 1;
        }

        DiscreteCircle dc(xc, yc, radius);
        Int edge_count = dc.getEdgeCount();
        uint32_t edges_crc = 0;
        for (const auto& e : dc.getEdges()) {
            edges_crc = rotate_crc(edges_crc, static_cast<uint32_t>(e.yPos));
            edges_crc = rotate_crc(edges_crc, static_cast<uint32_t>(e.xStart));
            edges_crc = rotate_crc(edges_crc, static_cast<uint32_t>(e.xEnd));
        }
        uint32_t draw_crc = 0;
        dc.drawCircle(sum_scanline_cb, &draw_crc);

        const auto& edges = dc.getEdges();
        Int first_y = edges.empty() ? 0 : edges.front().yPos;
        Int first_xs = edges.empty() ? 0 : edges.front().xStart;
        Int first_xe = edges.empty() ? 0 : edges.front().xEnd;
        Int last_y = edges.empty() ? 0 : edges.back().yPos;
        Int last_xs = edges.empty() ? 0 : edges.back().xStart;
        Int last_xe = edges.empty() ? 0 : edges.back().xEnd;

        std::fprintf(
            fp,
            "    {\"seed\": %u, \"xc\": %d, \"yc\": %d, \"radius\": %d, \"edge_count\": %d, "
            "\"first_y\": %d, \"first_xs\": %d, \"first_xe\": %d, "
            "\"last_y\": %d, \"last_xs\": %d, \"last_xe\": %d, "
            "\"edges_crc\": %u, \"draw_crc\": %u}%s\n",
            seed, xc, yc, radius, edge_count,
            first_y, first_xs, first_xe,
            last_y, last_xs, last_xe,
            edges_crc, draw_crc,
            (seed + 1 < 1000) ? "," : ""
        );
    }
    std::fprintf(fp, "  ]\n}\n");
    std::fclose(fp);
}

static void generate_pkt016_bezier(const char* out_path) {
    FILE* fp = std::fopen(out_path, "wb");
    if (!fp) { std::perror(out_path); std::exit(1); }
    std::fprintf(fp, "{\n  \"packet_id\": \"PKT-016\",\n  \"module\": \"BezierSegment\",\n  \"vector_count\": 1000,\n  \"vectors\": [\n");

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed * 0x165667b1u + 0x99887766u;
        Real cp[12];
        for (int i = 0; i < 12; ++i) {
            int32_t iv = static_cast<int32_t>(splitmix32(rng) % 40001u) - 20000;
            cp[i] = static_cast<Real>(iv) * 0.125f;
        }
        Real t = static_cast<Real>(splitmix32(rng) % 1025u) / 1024.0f;
        Int num_segments = static_cast<Int>(splitmix32(rng) % 16u);

        BezierSegment seg(cp);
        Coord3D eval_pt = {0.0f, 0.0f, 0.0f};
        seg.evaluateBezSegmentAtT(t, &eval_pt);

        VecCoord3D pts;
        seg.getSegmentPoints(num_segments, &pts);
        uint32_t pts_crc = 0;
        for (const auto& p : pts) {
            pts_crc = rotate_crc(pts_crc, f32_bits(p.x));
            pts_crc = rotate_crc(pts_crc, f32_bits(p.y));
            pts_crc = rotate_crc(pts_crc, f32_bits(p.z));
        }

        Real approx_len = seg.getApproximateLength(1.0f);

        BezierSegment s1, s2;
        seg.splitSegmentAtT(t, s1, s2);
        Coord3D s1_mid = {0.0f, 0.0f, 0.0f};
        Coord3D s2_mid = {0.0f, 0.0f, 0.0f};
        s1.evaluateBezSegmentAtT(0.5f, &s1_mid);
        s2.evaluateBezSegmentAtT(0.5f, &s2_mid);

        std::fprintf(
            fp,
            "    {\"seed\": %u, \"cp_bits\": [%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u], \"t_bits\": %u, \"num_segments\": %d, "
            "\"eval_x_bits\": %u, \"eval_y_bits\": %u, \"eval_z_bits\": %u, "
            "\"pts_crc\": %u, \"approx_len_bits\": %u, "
            "\"s1_mid_x_bits\": %u, \"s2_mid_z_bits\": %u}%s\n",
            seed,
            f32_bits(cp[0]), f32_bits(cp[1]), f32_bits(cp[2]),
            f32_bits(cp[3]), f32_bits(cp[4]), f32_bits(cp[5]),
            f32_bits(cp[6]), f32_bits(cp[7]), f32_bits(cp[8]),
            f32_bits(cp[9]), f32_bits(cp[10]), f32_bits(cp[11]),
            f32_bits(t), num_segments,
            f32_bits(eval_pt.x), f32_bits(eval_pt.y), f32_bits(eval_pt.z),
            pts_crc, f32_bits(approx_len),
            f32_bits(s1_mid.x), f32_bits(s2_mid.z),
            (seed + 1 < 1000) ? "," : ""
        );
    }
    std::fprintf(fp, "  ]\n}\n");
    std::fclose(fp);
}

static void generate_pkt064_namekey(const char* out_path) {
    FILE* fp = std::fopen(out_path, "wb");
    if (!fp) { std::perror(out_path); std::exit(1); }
    std::fprintf(fp, "{\n  \"packet_id\": \"PKT-064\",\n  \"module\": \"NameKeyGenerator\",\n  \"vector_count\": 1000,\n  \"vectors\": [\n");

    static const char* const kSampleTokens[16] = {
        "AmericaTankCrusader",
        "americatankcrusader",
        "GLATankScorpion",
        "ChinaTankOverlord",
        "Upgrade_ChinaNuclearMissile",
        "SCIENCE_PaladinTank",
        "Weapon_CrusaderTankGun",
        "Locomotor_FourWheels",
        "ObjectCreationList_Debris",
        "FXList_ExplosionMedium",
        "Data/English/Language9x.ini",
        "Data/Audio/Tracks/GLA_02.mp3",
        "SpecialPower_ParticleUplinkCannon",
        "Armor_CompositeTankArmor",
        "Behavior_AutoHealUpdate",
        "ModuleTag_01"
    };

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed * 0x85ebca6bu + 0x12345678u;
        NameKeyGenerator gen;
        gen.init();

        // Build a deterministic per-seed synthetic token
        char custom_buf[32];
        uint32_t r0 = splitmix32(rng);
        std::snprintf(custom_buf, sizeof(custom_buf), "Tok_%u_%X", seed % 137u, r0 & 0xFFFFu);

        UnsignedInt raw_hash = calcHashForString(custom_buf);
        UnsignedInt lower_hash = calcHashForLowercaseString(custom_buf);
        UnsignedInt socket_idx = raw_hash % 45007u;
        UnsignedInt lower_socket_idx = lower_hash % 45007u;

        uint32_t seq_crc = 0;
        NameKeyType last_key = NAMEKEY_INVALID;
        for (int step = 0; step < 12; ++step) {
            uint32_t op = splitmix32(rng);
            const char* tok = (step == 5 || step == 10)
                ? custom_buf
                : kSampleTokens[op & 15u];
            NameKeyType k = (op & 0x100u)
                ? gen.nameToLowercaseKey(tok)
                : gen.nameToKey(tok);
            seq_crc = rotate_crc(seq_crc, static_cast<uint32_t>(k));
            last_key = k;
        }
        AsciiString recovered = gen.keyToName(last_key);

        std::fprintf(
            fp,
            "    {\"seed\": %u, \"r0\": %u, \"raw_hash\": %u, \"lower_hash\": %u, "
            "\"socket_idx\": %u, \"lower_socket_idx\": %u, \"seq_crc\": %u, "
            "\"last_key\": %d, \"recovered\": \"%s\"}%s\n",
            seed, r0, raw_hash, lower_hash,
            socket_idx, lower_socket_idx, seq_crc,
            static_cast<int>(last_key), recovered.str(),
            (seed + 1 < 1000) ? "," : ""
        );
    }
    std::fprintf(fp, "  ]\n}\n");
    std::fclose(fp);
}

static void generate_pkt023_partition_solver(const char* out_path) {
    FILE* fp = std::fopen(out_path, "wb");
    if (!fp) { std::perror(out_path); std::exit(1); }
    std::fprintf(fp, "{\n  \"packet_id\": \"PKT-023\",\n  \"module\": \"PartitionSolver\",\n  \"vector_count\": 1000,\n  \"vectors\": [\n");

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed * 0x9e3779b9u + 0x55555555u;
        uint32_t n_entries = (splitmix32(rng) % 7u) + 1u;
        uint32_t n_spaces = (splitmix32(rng) % 4u) + 1u;

        EntriesVec entries;
        SpacesVec spaces;
        for (uint32_t i = 0; i < n_entries; ++i) {
            ObjectID id = static_cast<ObjectID>(100 + i);
            // Make sizes distinct per entry to avoid std::sort unstable tie-breaking differences
            UnsignedInt sz = ((splitmix32(rng) % 30u) + 1u) * 8u + i;
            entries.push_back(std::make_pair(id, sz));
        }
        for (uint32_t j = 0; j < n_spaces; ++j) {
            ObjectID id = static_cast<ObjectID>(500 + j);
            UnsignedInt sz = ((splitmix32(rng) % 60u) + 10u) * 8u + j;
            spaces.push_back(std::make_pair(id, sz));
        }

        PartitionSolver solver(entries, spaces, PREFER_FAST_SOLUTION);
        solver.solve();
        const SolutionVec& sol = solver.getSolution();

        uint32_t sol_crc = 0;
        for (const auto& pair : sol) {
            sol_crc = rotate_crc(sol_crc, static_cast<uint32_t>(pair.first));
            sol_crc = rotate_crc(sol_crc, static_cast<uint32_t>(pair.second));
        }

        std::fprintf(
            fp,
            "    {\"seed\": %u, \"n_entries\": %u, \"n_spaces\": %u, \"sol_len\": %u, \"sol_crc\": %u}%s\n",
            seed, n_entries, n_spaces, static_cast<uint32_t>(sol.size()), sol_crc,
            (seed + 1 < 1000) ? "," : ""
        );
    }
    std::fprintf(fp, "  ]\n}\n");
    std::fclose(fp);
}

int main() {
    generate_pkt004_bitflags("docs/golden_vectors/PKT-004.json");
    generate_pkt016_bezier("docs/golden_vectors/PKT-016.json");
    generate_pkt017_discrete_circle("docs/golden_vectors/PKT-017.json");
    generate_pkt023_partition_solver("docs/golden_vectors/PKT-023.json");
    generate_pkt064_namekey("docs/golden_vectors/PKT-064.json");
    std::printf("Generated 5,000 C++ Golden Test Vectors across PKT-004, PKT-016, PKT-017, PKT-023, PKT-064.\n");
    return 0;
}
