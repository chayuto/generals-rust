// PKT-418 (PathfindZoneManager) + PKT-420 (ZoneBlock) (WP-6.2) C++ Golden Vector Generator.
// Compiles unmodified vendor/GeneralsX sources against minimal harness stubs.
//
// Canonical build & run command (macOS arm64, Release-aligned macros):
//   clang++ -std=c++17 -O1 -ffp-contract=off -URTS_DEBUG \
//     -DRTS_ZEROHOUR=1 -DRTS_GENERALS=0 -DRETAIL_COMPATIBLE_CRC=0 \
//     -DRETAIL_COMPATIBLE_PATHFINDING=1 -DRETAIL_COMPATIBLE_PATHFINDING_ALLOCATION=1 \
//     -Wno-macro-redefined -Wno-switch \
//     -Itests/cpp_harness/stubs_pkt418 \
//     -Ivendor/GeneralsX/GeneralsMD/Code/GameEngine/Include \
//     -Ivendor/GeneralsX/Core/GameEngine/Include \
//     -Ivendor/GeneralsX/build/macos-vulkan/_deps/gamemath-src/include \
//     tests/cpp_harness/gen_pkt418_zone_manager.cpp \
//     vendor/GeneralsX/build/macos-vulkan/_deps/gamemath-build/libgm.a \
//     -o /tmp/gen_pkt418 && /tmp/gen_pkt418 docs/golden_vectors/PKT-418.json
//
// Determinism & UB verification builds (must produce byte-identical JSON):
//   -O0, -O1, -O2, and -O1 -g -fsanitize=address,undefined.

#include "PreRTS.h"
#include "GameLogic/Object.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/LocomotorSet.h"
#include "GameLogic/Pathfinder/PathfindConstants.h"
#include "GameLogic/Pathfinder/PathfindCellInfo.h"
#include "GameLogic/Pathfinder/PathfindCellList.h"
#include "GameLogic/Pathfinder/PathfindCell.h"
#include "GameLogic/Pathfinder/PathfindLayer.h"
#include "GameLogic/Pathfinder/ZoneBlock.h"
#include "GameLogic/Pathfinder/PathfindZoneManager.h"

#include "../../vendor/GeneralsX/Core/GameEngine/Source/GameLogic/AI/Pathfinder/PathfindConstants.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/GameLogic/AI/Pathfinder/PathfindCellInfo.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/GameLogic/AI/Pathfinder/PathfindCellList.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/GameLogic/AI/Pathfinder/PathfindCell.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/GameLogic/AI/Pathfinder/ZoneBlock.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/GameLogic/AI/Pathfinder/PathfindZoneManager.cpp"

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>

static GameLogic g_game_logic;
GameLogic* TheGameLogic = &g_game_logic;

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
            std::perror("fread");
            std::exit(1);
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

struct Rng {
    uint32_t state;
    explicit Rng(uint32_t seed) : state(seed == 0 ? 0x6c8e9cf5u : seed) {}
    uint32_t next_u32() {
        uint32_t x = state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        state = x;
        return x;
    }
    int32_t srange(int32_t lo, int32_t hi_incl) {
        return lo + static_cast<int32_t>(next_u32() % static_cast<uint32_t>(hi_incl - lo + 1));
    }
};

struct CellMap2D {
    Int w = 0;
    Int h = 0;
    std::vector<PathfindCell> storage;
    std::vector<PathfindCell*> cols;

    CellMap2D(Int width, Int height)
        : w(width), h(height), storage(static_cast<size_t>(width * height)), cols(static_cast<size_t>(width)) {
        for (Int x = 0; x < w; ++x) {
            cols[x] = &storage[static_cast<size_t>(x * h)];
        }
    }

    ~CellMap2D() {
        for (auto& c : storage) {
            c.reset();
        }
    }

    PathfindCell** ptr() { return cols.data(); }
};

int main(int argc, char** argv) {
    if (argc != 2) {
        std::fprintf(stderr, "Usage: %s <output.json>\n", argv[0]);
        return 1;
    }

    PathfindCellInfo::allocateCellInfos();

    const char* source_paths[4] = {
        "vendor/GeneralsX/Core/GameEngine/Include/GameLogic/Pathfinder/PathfindZoneManager.h",
        "vendor/GeneralsX/Core/GameEngine/Source/GameLogic/AI/Pathfinder/PathfindZoneManager.cpp",
        "vendor/GeneralsX/Core/GameEngine/Include/GameLogic/Pathfinder/ZoneBlock.h",
        "vendor/GeneralsX/Core/GameEngine/Source/GameLogic/AI/Pathfinder/ZoneBlock.cpp",
    };

    FILE* out = std::fopen(argv[1], "wb");
    if (!out) { std::perror(argv[1]); return 1; }

    std::fprintf(out, "{\n");
    std::fprintf(out, "  \"packet_id\": \"PKT-418\",\n");
    std::fprintf(out, "  \"sub_packets\": [\"PKT-418\", \"PKT-420\"],\n");
    std::fprintf(out, "  \"work_package\": \"WP-6.2\",\n");
    std::fprintf(out, "  \"reference_commit\": \"84a7a20769aa5749f388b1f49bd542c79b3c33e8\",\n");
    std::fprintf(out, "  \"vector_count\": 1000,\n");
    std::fprintf(out, "  \"constants\": {\n");
    std::fprintf(out, "    \"INITIAL_ZONES\": %d,\n", PathfindZoneManager::INITIAL_ZONES);
    std::fprintf(out, "    \"ZONE_BLOCK_SIZE\": %d,\n", PathfindZoneManager::ZONE_BLOCK_SIZE);
    std::fprintf(out, "    \"UNINITIALIZED_ZONE\": %d,\n", PathfindZoneManager::UNINITIALIZED_ZONE);
    std::fprintf(out, "    \"LAYER_LAST\": %d,\n", LAYER_LAST);
    std::fprintf(out, "    \"ZONE_UPDATE_FREQUENCY\": %d\n", ZONE_UPDATE_FREQUENCY);
    std::fprintf(out, "  },\n");
    std::fprintf(out, "  \"source_sha256\": {\n");
    for (size_t i = 0; i < 4; ++i) {
        std::string sha = sha256_file(source_paths[i]);
        std::fprintf(out, "    \"%s\": \"%s\"%s\n", source_paths[i], sha.c_str(), i + 1 < 4 ? "," : "");
    }
    std::fprintf(out, "  },\n");
    std::fprintf(out, "  \"vectors\": [\n");

    static const Int kWidths[6]  = {1, 10, 11, 15, 24, 22};
    static const Int kHeights[6] = {1, 10, 11, 23, 18, 12};
    static const Int kMasks[11] = {
        LOCOMOTORSURFACE_AIR,
        LOCOMOTORSURFACE_GROUND | LOCOMOTORSURFACE_WATER | LOCOMOTORSURFACE_CLIFF,
        LOCOMOTORSURFACE_GROUND | LOCOMOTORSURFACE_CLIFF,
        LOCOMOTORSURFACE_GROUND | LOCOMOTORSURFACE_WATER,
        LOCOMOTORSURFACE_GROUND | LOCOMOTORSURFACE_RUBBLE,
        LOCOMOTORSURFACE_GROUND,
        LOCOMOTORSURFACE_WATER,
        LOCOMOTORSURFACE_CLIFF,
        LOCOMOTORSURFACE_RUBBLE,
        LOCOMOTORSURFACE_CLIFF | LOCOMOTORSURFACE_WATER,
        0
    };

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        Rng rng(seed ^ 0x418a420bU);

        // Standalone ZoneBlock helper oracle on small synthetic zone equivalency table.
        std::vector<zoneStorageType> zb_eq(12);
        std::vector<zoneStorageType> zb_hier(12);
        for (Int i = 0; i < 12; ++i) {
            zb_eq[i] = static_cast<zoneStorageType>(i);
            zb_hier[i] = static_cast<zoneStorageType>(i);
        }
        Int z_a = rng.srange(1, 11), z_b = rng.srange(1, 11);
        Int z_c = rng.srange(1, 11), z_d = rng.srange(1, 11);
        ZoneBlock::resolveZones(z_a, z_b, zb_hier.data(), 12);
        ZoneBlock::resolveZones(z_c, z_d, zb_eq.data(), 12);
        std::vector<zoneStorageType> zb_before_flatten = zb_eq;
        ZoneBlock::flattenZones(zb_eq.data(), zb_hier.data(), 12);

        // Map geometry
        Int shape_idx = static_cast<Int>(seed % 6);
        Int w = kWidths[shape_idx];
        Int h = kHeights[shape_idx];
        CellMap2D grid(w, h);
        IRegion2D global_bounds{};
        global_bounds.lo = ICoord2D{0, 0};
        global_bounds.hi = ICoord2D{w - 1, h - 1};

        // Populate cell terrain types and fences.
        std::vector<Int> cell_types(static_cast<size_t>(w * h));
        std::vector<Int> cell_fence(static_cast<size_t>(w * h), 0);
        std::vector<Int> cell_connect(static_cast<size_t>(w * h), LAYER_INVALID);
        Object obs_obj;
        obs_obj.m_id = static_cast<ObjectID>(77);

        for (Int x = 0; x < w; ++x) {
            for (Int y = 0; y < h; ++y) {
                Int idx = x * h + y;
                uint32_t roll = rng.next_u32() % 20;
                PathfindCell::CellType ct = PathfindCell::CELL_CLEAR;
                if (roll < 8) ct = PathfindCell::CELL_CLEAR;
                else if (roll < 11) ct = PathfindCell::CELL_WATER;
                else if (roll < 13) ct = PathfindCell::CELL_CLIFF;
                else if (roll < 15) ct = PathfindCell::CELL_RUBBLE;
                else if (roll < 17) ct = PathfindCell::CELL_OBSTACLE;
                else if (roll < 18) ct = PathfindCell::CELL_BRIDGE_IMPASSABLE;
                else ct = PathfindCell::CELL_IMPASSABLE;

                PathfindCell* c = &grid.cols[x][y];
                if (ct == PathfindCell::CELL_OBSTACLE) {
                    Bool is_fence = (rng.next_u32() % 2) != 0 ? TRUE : FALSE;
                    c->setTypeAsObstacle(&obs_obj, is_fence, ICoord2D{x, y});
                    cell_fence[idx] = is_fence ? 1 : 0;
                } else {
                    c->setType(ct);
                }
                cell_types[idx] = static_cast<Int>(c->getType());
            }
        }

        // Configure bridge layers (2..15) and connect layers on selected CLEAR cells.
        PathfindLayer layers[LAYER_LAST + 1];
        std::vector<Int> layer_unused(LAYER_LAST + 1, 1);
        std::vector<Int> layer_destroyed(LAYER_LAST + 1, 0);
        std::vector<Int> layer_start_x(LAYER_LAST + 1, 0);
        std::vector<Int> layer_start_y(LAYER_LAST + 1, 0);
        std::vector<Int> layer_end_x(LAYER_LAST + 1, 0);
        std::vector<Int> layer_end_y(LAYER_LAST + 1, 0);

        Int n_active_bridges = static_cast<Int>(seed % 4); // 0..3 bridges
        for (Int bi = 0; bi < n_active_bridges; ++bi) {
            Int lidx = 2 + static_cast<Int>(rng.next_u32() % 13);
            layers[lidx].m_unused = FALSE;
            layers[lidx].m_destroyed = (rng.next_u32() % 4 == 0) ? TRUE : FALSE;
            // Include occasional out-of-bounds bridge endpoints to verify off-grid guard.
            Int sx = rng.srange(-2, w + 1);
            Int sy = rng.srange(-2, h + 1);
            Int ex = rng.srange(0, w - 1);
            Int ey = rng.srange(0, h - 1);
            layers[lidx].m_startCell = ICoord2D{sx, sy};
            layers[lidx].m_endCell = ICoord2D{ex, ey};
            layer_unused[lidx] = 0;
            layer_destroyed[lidx] = layers[lidx].m_destroyed ? 1 : 0;
            layer_start_x[lidx] = sx;
            layer_start_y[lidx] = sy;
            layer_end_x[lidx] = ex;
            layer_end_y[lidx] = ey;

            Int gx = rng.srange(0, w - 1);
            Int gy = rng.srange(0, h - 1);
            grid.cols[gx][gy].setConnectLayer(static_cast<PathfindLayerEnum>(lidx));
            cell_connect[gx * h + gy] = lidx;
        }

        // Test markZonesDirty / needToCalculateZones before and after calculateZones.
        PathfindZoneManager zm;
        UnsignedInt frame0 = (seed < 4) ? seed : ((seed == 5) ? 0xFFFFFF00u : (rng.next_u32() % 10000u));
        g_game_logic.m_frame = frame0;
        Bool need_init = zm.needToCalculateZones();
        zm.markZonesDirty();
        g_game_logic.m_frame = (seed % 3 == 0) ? frame0 + 1 : frame0 + 350;
        Bool need_after_dirty = zm.needToCalculateZones();

        zm.allocateBlocks(global_bounds);
        ICoord2D extent{};
        zm.getExtent(extent);
        zm.calculateZones(grid.ptr(), layers, global_bounds);
        Bool need_after_calc = zm.needToCalculateZones();
        g_game_logic.m_frame = frame0 + 5;
        zm.markZonesDirty();
        Bool need_after_dirty2 = zm.needToCalculateZones();

        std::vector<Int> cell_zones_after_calc(static_cast<size_t>(w * h));
        zoneStorageType max_obs_zone = 1;
        for (Int x = 0; x < w; ++x) {
            for (Int y = 0; y < h; ++y) {
                zoneStorageType z = grid.cols[x][y].getZone();
                cell_zones_after_calc[static_cast<size_t>(x * h + y)] = static_cast<Int>(z);
                if (z > max_obs_zone) max_obs_zone = z;
            }
        }
        std::vector<Int> layer_zones(LAYER_LAST + 1);
        std::vector<Int> layer_applied(LAYER_LAST + 1);
        for (Int li = 0; li <= LAYER_LAST; ++li) {
            layer_zones[li] = layers[li].m_zone;
            layer_applied[li] = layers[li].m_appliedZone;
        }

        std::vector<Int> blocks_bridge_after_calc(static_cast<size_t>(extent.x * extent.y));
        for (Int bx = 0; bx < extent.x; ++bx) {
            for (Int by = 0; by < extent.y; ++by) {
                blocks_bridge_after_calc[static_cast<size_t>(bx * extent.y + by)] =
                    zm.interactsWithBridge(bx * 10, by * 10) ? 1 : 0;
            }
        }

        // Sample getEffectiveZone, getEffectiveTerrainZone, and getBlockZone.
        std::vector<Int> eff_queries;
        std::vector<Int> eff_results;
        for (Int qi = 0; qi < 12; ++qi) {
            Int mask = kMasks[(seed + qi) % 11];
            Bool crusher = ((rng.next_u32() % 2) != 0) ? TRUE : FALSE;
            Int z = (qi == 0) ? 0
                  : (qi == 1) ? static_cast<Int>(max_obs_zone)
                  : (qi == 2) ? static_cast<Int>(max_obs_zone + 64)
                  : rng.srange(1, static_cast<Int>(max_obs_zone));
            zoneStorageType out_z = zm.getEffectiveZone(mask, crusher, static_cast<zoneStorageType>(z));
            eff_queries.push_back(mask);
            eff_queries.push_back(crusher ? 1 : 0);
            eff_queries.push_back(z);
            eff_results.push_back(static_cast<Int>(out_z));
        }

        std::vector<Int> terr_q;
        std::vector<Int> terr_r;
        for (Int qi = 0; qi < 4; ++qi) {
            Int z = rng.srange(0, static_cast<Int>(max_obs_zone));
            terr_q.push_back(z);
            terr_r.push_back(static_cast<Int>(zm.getEffectiveTerrainZone(static_cast<zoneStorageType>(z))));
        }

        std::vector<Int> bz_q;
        std::vector<Int> bz_r;
        for (Int qi = 0; qi < 6; ++qi) {
            Int mask = kMasks[(seed * 3 + qi) % 11];
            Bool crusher = ((rng.next_u32() % 2) != 0) ? TRUE : FALSE;
            Int cx = rng.srange(0, w - 1);
            Int cy = rng.srange(0, h - 1);
            zoneStorageType bz = zm.getBlockZone(mask, crusher, cx, cy, grid.ptr());
            bz_q.push_back(mask);
            bz_q.push_back(crusher ? 1 : 0);
            bz_q.push_back(cx);
            bz_q.push_back(cy);
            bz_r.push_back(static_cast<Int>(bz));
        }

        // Passability & setBridge probes (including out-of-bounds and negative cells).
        zm.clearPassableFlags();
        Int pass_after_clear = zm.isPassable(0, 0) ? 1 : 0;
        zm.setAllPassable();
        Int pass_after_all = zm.isPassable(0, 0) ? 1 : 0;
        Int set_pass_x = rng.srange(0, w - 1);
        Int set_pass_y = rng.srange(0, h - 1);
        zm.setPassable(set_pass_x, set_pass_y, FALSE);
        Int p_in_ok = zm.isPassable(set_pass_x, set_pass_y) ? 1 : 0;
        Int p_neg_clip = zm.clipIsPassable(-1, 0) ? 1 : 0;
        Int p_oob_clip = zm.clipIsPassable(w + 20, h + 20) ? 1 : 0;
        Int p_neg_is = zm.isPassable(-1, 0) ? 1 : 0;
        zm.setBridge(set_pass_x, set_pass_y, TRUE);
        Int br_set = zm.interactsWithBridge(set_pass_x, set_pass_y) ? 1 : 0;

        // Exercise updateZonesForModify by clearing a sub-region's zones to UNINITIALIZED_ZONE (0).
        Int m_lx = rng.srange(0, w - 1);
        Int m_hx = rng.srange(m_lx, w - 1);
        Int m_ly = rng.srange(0, h - 1);
        Int m_hy = rng.srange(m_ly, h - 1);
        for (Int x = m_lx; x <= m_hx; ++x) {
            for (Int y = m_ly; y <= m_hy; ++y) {
                grid.cols[x][y].setZone(PathfindZoneManager::UNINITIALIZED_ZONE);
            }
        }
        IRegion2D mod_bounds{ICoord2D{m_lx, m_ly}, ICoord2D{m_hx, m_hy}};
        zm.updateZonesForModify(grid.ptr(), layers, mod_bounds, global_bounds);
        std::vector<Int> cell_zones_after_modify(static_cast<size_t>(w * h));
        for (Int x = 0; x < w; ++x) {
            for (Int y = 0; y < h; ++y) {
                cell_zones_after_modify[static_cast<size_t>(x * h + y)] =
                    static_cast<Int>(grid.cols[x][y].getZone());
            }
        }

        // Emit JSON object line for seed.
        auto print_vec = [&](const char* k, const std::vector<Int>& v) {
            std::fprintf(out, ", \"%s\": [", k);
            for (size_t i = 0; i < v.size(); ++i) std::fprintf(out, "%s%d", i ? "," : "", v[i]);
            std::fprintf(out, "]");
        };
        auto print_zvec = [&](const char* k, const std::vector<zoneStorageType>& v) {
            std::fprintf(out, ", \"%s\": [", k);
            for (size_t i = 0; i < v.size(); ++i) std::fprintf(out, "%s%u", i ? "," : "", v[i]);
            std::fprintf(out, "]");
        };

        std::fprintf(out, "    {\"seed\": %u", seed);
        std::fprintf(out, ", \"zb_pairs\": [%d,%d,%d,%d]", z_a, z_b, z_c, z_d);
        print_zvec("zb_hier", zb_hier);
        print_zvec("zb_pre_flat", zb_before_flatten);
        print_zvec("zb_post_flat", zb_eq);

        std::fprintf(out, ", \"w\": %d, \"h\": %d", w, h);
        print_vec("cell_types", cell_types);
        print_vec("cell_fence", cell_fence);
        print_vec("cell_connect", cell_connect);
        print_vec("layer_unused", layer_unused);
        print_vec("layer_destroyed", layer_destroyed);
        print_vec("layer_sx", layer_start_x);
        print_vec("layer_sy", layer_start_y);
        print_vec("layer_ex", layer_end_x);
        print_vec("layer_ey", layer_end_y);

        std::fprintf(out, ", \"frames\": [%u,%u]", frame0, g_game_logic.m_frame);
        std::fprintf(out, ", \"need_calc\": [%d,%d,%d,%d]",
            need_init ? 1 : 0, need_after_dirty ? 1 : 0,
            need_after_calc ? 1 : 0, need_after_dirty2 ? 1 : 0);
        std::fprintf(out, ", \"extent\": [%d,%d]", extent.x, extent.y);
        print_vec("cell_zones_calc", cell_zones_after_calc);
        print_vec("layer_zones", layer_zones);
        print_vec("layer_applied", layer_applied);
        print_vec("blocks_bridge_calc", blocks_bridge_after_calc);
        print_vec("eff_q", eff_queries);
        print_vec("eff_r", eff_results);
        print_vec("terr_q", terr_q);
        print_vec("terr_r", terr_r);
        print_vec("bz_q", bz_q);
        print_vec("bz_r", bz_r);
        std::fprintf(out, ", \"pass_ops\": [%d,%d,%d,%d,%d,%d,%d,%d,%d]",
            set_pass_x, set_pass_y, pass_after_clear, pass_after_all,
            p_in_ok, p_neg_clip, p_oob_clip, p_neg_is, br_set);
        std::fprintf(out, ", \"mod_bounds\": [%d,%d,%d,%d]", m_lx, m_ly, m_hx, m_hy);
        print_vec("cell_zones_mod", cell_zones_after_modify);

        std::fprintf(out, "}%s\n", seed + 1 < 1000 ? "," : "");
    }

    std::fprintf(out, "  ]\n}\n");
    std::fclose(out);
    return 0;
}
