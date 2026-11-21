// PKT-413 (PathfindCell + PathfindCellInfo + PathfindCellList + PathfindConstants, WP-6.2)
// C++ Golden Vector Generator.
// Compiles unmodified vendor/GeneralsX sources against minimal harness stubs.
//
// Canonical build & run command (macOS arm64, Release-aligned macros):
//   clang++ -std=c++17 -O1 -ffp-contract=off -URTS_DEBUG \
//     -DRETAIL_COMPATIBLE_PATHFINDING=1 -DRETAIL_COMPATIBLE_PATHFINDING_ALLOCATION=1 \
//     -Wno-macro-redefined -Wno-switch \
//     -Itests/cpp_harness/stubs_pkt413 \
//     -Ivendor/GeneralsX/GeneralsMD/Code/GameEngine/Include \
//     -Ivendor/GeneralsX/Core/GameEngine/Include \
//     -Ivendor/GeneralsX/build/macos-vulkan/_deps/gamemath-src/include \
//     tests/cpp_harness/gen_pkt413_pathfind_cell.cpp \
//     vendor/GeneralsX/build/macos-vulkan/_deps/gamemath-build/libgm.a \
//     -o /tmp/gen_pkt413 && /tmp/gen_pkt413 docs/golden_vectors/PKT-413.json
//
// Determinism & UB verification builds (must produce byte-identical JSON):
//   clang++ -std=c++17 -O0 -ffp-contract=off -URTS_DEBUG -DRETAIL_COMPATIBLE_PATHFINDING=1 -DRETAIL_COMPATIBLE_PATHFINDING_ALLOCATION=1 ... -o /tmp/gen_pkt413_o0
//   clang++ -std=c++17 -O2 -ffp-contract=off -URTS_DEBUG -DRETAIL_COMPATIBLE_PATHFINDING=1 -DRETAIL_COMPATIBLE_PATHFINDING_ALLOCATION=1 ... -o /tmp/gen_pkt413_o2
//   clang++ -std=c++17 -O1 -g -fsanitize=address,undefined -fno-omit-frame-pointer -ffp-contract=off -URTS_DEBUG -DRETAIL_COMPATIBLE_PATHFINDING=1 -DRETAIL_COMPATIBLE_PATHFINDING_ALLOCATION=1 ... -o /tmp/gen_pkt413_asan
//
// __APPLE__ audit on vendor files compiled here:
//   - PathfindCell.{h,cpp}, PathfindCellInfo.{h,cpp}, PathfindCellList.{h,cpp}, PathfindConstants.{h,cpp}: 0 hits.

#include "PreRTS.h"
#include "GameLogic/Object.h"
#include "GameLogic/Pathfinder/PathfindConstants.h"
#include "GameLogic/Pathfinder/PathfindCellInfo.h"
#include "GameLogic/Pathfinder/PathfindCellList.h"
#include "GameLogic/Pathfinder/PathfindCell.h"

#include "../../vendor/GeneralsX/Core/GameEngine/Source/GameLogic/AI/Pathfinder/PathfindConstants.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/GameLogic/AI/Pathfinder/PathfindCellInfo.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/GameLogic/AI/Pathfinder/PathfindCellList.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/GameLogic/AI/Pathfinder/PathfindCell.cpp"

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>

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

static constexpr Int GRID_W = 4;
static constexpr Int GRID_H = 4;
static constexpr Int NUM_CELLS = GRID_W * GRID_H;

// Friend-like accessor for inspecting protected PathfindCellInfo pool state without modifying vendor code.
struct InfoInspector : public PathfindCellInfo {
    static UnsignedInt countFree() {
        UnsignedInt cnt = 0;
        const PathfindCellInfo* cur = s_firstFree;
        while (cur && cnt <= CELL_INFOS_TO_ALLOCATE) {
            ++cnt;
            cur = reinterpret_cast<const InfoInspector*>(cur)->m_pathParent;
        }
        return cnt;
    }
    static Int indexOfInfo(const PathfindCellInfo* info, const PathfindCell* base) {
        if (!info) return -1;
        const PathfindCell* c = reinterpret_cast<const InfoInspector*>(info)->m_cell;
        if (!c) return -1;
        return static_cast<Int>(c - base);
    }
};

struct CellInspector : public PathfindCell {
    static Int indexOfTail(const PathfindCellList& list, const PathfindCell* base) {
        struct ListHack { PathfindCell* head; PathfindCell* tail; };
        const ListHack* lh = reinterpret_cast<const ListHack*>(&list);
        if (!lh->tail) return -1;
        return static_cast<Int>(lh->tail - base);
    }
};

static Int cell_ptr_to_idx(const PathfindCell* p, const PathfindCell* base) {
    return p ? static_cast<Int>(p - base) : -1;
}

static std::vector<Int> compute_predicate_matrix() {
    std::vector<Int> out;
    out.reserve(7 * 2 * 7 * 2);
    s_useFixedPathfinding = true; // allows fence flag directly on PathfindCell cleanly
    const ICoord2D origin{0, 0};
    for (int tt = 0; tt <= 6; ++tt) {
        for (int tf = 0; tf <= 1; ++tf) {
            for (int st = 0; st <= 6; ++st) {
                for (int sf = 0; sf <= 1; ++sf) {
                    PathfindCell tc, sc;
                    if (tt == PathfindCell::CELL_OBSTACLE) {
                        Object o; o.m_id = static_cast<ObjectID>(10);
                        tc.setTypeAsObstacle(&o, tf != 0, origin);
                    } else {
                        tc.setType(static_cast<PathfindCell::CellType>(tt));
                    }
                    if (st == PathfindCell::CELL_OBSTACLE) {
                        Object o; o.m_id = static_cast<ObjectID>(20);
                        sc.setTypeAsObstacle(&o, sf != 0, origin);
                    } else {
                        sc.setType(static_cast<PathfindCell::CellType>(st));
                    }
                    Int mask = 0;
                    if (PathfindCell::typesMatch(tc, sc)) mask |= 1 << 0;
                    if (PathfindCell::waterGround(tc, sc)) mask |= 1 << 1;
                    if (PathfindCell::groundRubble(tc, sc)) mask |= 1 << 2;
                    if (PathfindCell::terrain(tc, sc)) mask |= 1 << 3;
                    if (PathfindCell::crusherGround(tc, sc)) mask |= 1 << 4;
                    if (PathfindCell::groundCliff(tc, sc)) mask |= 1 << 5;
                    out.push_back(mask);
                }
            }
        }
    }
    return out;
}

static void run_startup_self_checks() {
    PathfindCellInfo::allocateCellInfos();
    if (InfoInspector::countFree() != CELL_INFOS_TO_ALLOCATE) {
        std::fprintf(stderr, "Self-check failed: initial free count != 30000\n");
        std::exit(1);
    }

    // Check waterGround bitwise & CELL_WATER quirk on RUBBLE(3) & BRIDGE_IMPASSABLE(5).
    PathfindCell c_clear, c_rubble, c_bridge;
    c_clear.setType(PathfindCell::CELL_CLEAR);
    c_rubble.setType(PathfindCell::CELL_RUBBLE);
    c_bridge.setType(PathfindCell::CELL_BRIDGE_IMPASSABLE);
    if (!PathfindCell::waterGround(c_clear, c_rubble) ||
        !PathfindCell::waterGround(c_clear, c_bridge)) {
        std::fprintf(stderr, "Self-check failed: expected C++ bitwise waterGround quirk on RUBBLE/BRIDGE_IMPASSABLE\n");
        std::exit(1);
    }

    // Check costToHierGoal returns 100000 when m_info is null.
    if (c_clear.costToHierGoal(&c_rubble) != 100000u) {
        std::fprintf(stderr, "Self-check failed: costToHierGoal without info != 100000\n");
        std::exit(1);
    }

    PathfindCellInfo::releaseCellInfos();
}

struct SimShadow {
    bool in_open[NUM_CELLS]{};
    bool in_closed[NUM_CELLS]{};
    bool open_corrupt = false;
    bool closed_corrupt = false;
};

static std::string snapshot_cells(const PathfindCell* cells, const PathfindCell* base) {
    std::string out;
    for (Int i = 0; i < NUM_CELLS; ++i) {
        const PathfindCell& c = cells[i];
        if (i > 0) out.push_back(';');
        Int has_info = c.hasInfo() ? 1 : 0;
        Int ally = (s_useFixedPathfinding || has_info) ? (c.isBlockedByAlly() ? 1 : 0) : -1;
        Int open = has_info ? (c.getOpen() ? 1 : 0) : 0;
        Int closed = has_info ? (c.getClosed() ? 1 : 0) : 0;
        UnsignedInt tcost = has_info ? c.getTotalCost() : 0u;
        UnsignedInt csf = has_info ? c.getCostSoFar() : 0u;
        Int parent = c.getParentCell() ? static_cast<Int>(c.getParentCell() - base) : -1;
        Int gu = static_cast<Int>(c.getGoalUnit());
        Int pu = static_cast<Int>(c.getPosUnit());
        Int ga = static_cast<Int>(c.getGoalAircraft());
        char buf[160];
        std::snprintf(buf, sizeof(buf),
                      "%d,%d,%u,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%u,%u,%d,%d,%d,%d",
                      static_cast<Int>(c.getType()),
                      static_cast<Int>(c.getFlags()),
                      static_cast<UnsignedInt>(c.getZone()),
                      c.isAircraftGoal() ? 1 : 0,
                      c.getPinched() ? 1 : 0,
                      static_cast<Int>(c.getLayer()),
                      static_cast<Int>(c.getConnectLayer()),
                      has_info,
                      static_cast<Int>(c.getObstacleID()),
                      c.isObstacleFence() ? 1 : 0,
                      c.isObstacleTransparent() ? 1 : 0,
                      ally,
                      open,
                      closed,
                      tcost,
                      csf,
                      parent,
                      gu,
                      pu,
                      ga);
        out.append(buf);
    }
    return out;
}

static void record_list(PathfindCell* head, const PathfindCell* base, std::vector<Int>& ids) {
    ids.clear();
    PathfindCell* cur = head;
    Int guard = 0;
    while (cur && guard < NUM_CELLS * 2) {
        ids.push_back(static_cast<Int>(cur - base));
        cur = cur->hasInfo() ? cur->getNextOpen() : nullptr;
        ++guard;
    }
}

int main(int argc, char** argv) {
    if (argc != 2) {
        std::fprintf(stderr, "Usage: gen_pkt413 <out.json>\n");
        return 1;
    }
    const char* out_path = argv[1];
    std::string tmp_out = std::string(out_path) + ".tmp";

    run_startup_self_checks();

    const char* source_paths[] = {
        "vendor/GeneralsX/Core/GameEngine/Include/GameLogic/Pathfinder/PathfindCell.h",
        "vendor/GeneralsX/Core/GameEngine/Source/GameLogic/AI/Pathfinder/PathfindCell.cpp",
        "vendor/GeneralsX/Core/GameEngine/Include/GameLogic/Pathfinder/PathfindCellInfo.h",
        "vendor/GeneralsX/Core/GameEngine/Source/GameLogic/AI/Pathfinder/PathfindCellInfo.cpp",
        "vendor/GeneralsX/Core/GameEngine/Include/GameLogic/Pathfinder/PathfindCellList.h",
        "vendor/GeneralsX/Core/GameEngine/Source/GameLogic/AI/Pathfinder/PathfindCellList.cpp",
        "vendor/GeneralsX/Core/GameEngine/Include/GameLogic/Pathfinder/PathfindConstants.h",
        "vendor/GeneralsX/Core/GameEngine/Source/GameLogic/AI/Pathfinder/PathfindConstants.cpp"
    };

    std::vector<Int> pred_matrix = compute_predicate_matrix();

    FILE* fp = std::fopen(tmp_out.c_str(), "wb");
    if (!fp) { std::perror(tmp_out.c_str()); return 1; }

    std::fprintf(fp,
        "{\n"
        "  \"packet_id\": \"PKT-413\",\n"
        "  \"bundle\": [\"PKT-413 PathfindCell\", \"PKT-414 PathfindCellInfo\", \"PKT-415 PathfindCellList\", \"PKT-416 PathfindConstants\"],\n"
        "  \"vector_count\": 1000,\n"
        "  \"source_sha256\": {\n");
    for (size_t i = 0; i < ARRAY_SIZE(source_paths); ++i) {
        std::string digest = sha256_file(source_paths[i]);
        std::fprintf(fp, "    \"%s\": \"%s\"%s\n",
                     source_paths[i], digest.c_str(),
                     (i + 1 < ARRAY_SIZE(source_paths)) ? "," : "");
    }
    std::fprintf(fp, "  },\n");
    std::fprintf(fp,
        "  \"constants\": {"
        "\"MAX_WALL_PIECES\": %d, "
        "\"PATHFIND_QUEUE_LEN\": %d, "
        "\"MOVE_ALLIES_MAX_DEPTH\": %d, "
        "\"LAYER_Z_CLOSE_ENOUGH_F_BITS\": \"0x%08x\", "
        "\"PATHFIND_CELL_SIZE\": %u, "
        "\"PATHFIND_CELL_SIZE_F_BITS\": \"0x%08x\", "
        "\"ZONE_UPDATE_FREQUENCY\": %u, "
        "\"MAX_CELL_COUNT\": %u, "
        "\"MAX_ADJUSTMENT_CELL_COUNT\": %u, "
        "\"MAX_SAFE_PATH_CELL_COUNT\": %u, "
        "\"PATHFIND_CELLS_PER_FRAME\": %u, "
        "\"COST_ORTHOGONAL\": %d, "
        "\"COST_DIAGONAL\": %d, "
        "\"COST_TO_DISTANCE_FACTOR_BITS\": \"0x%08x\", "
        "\"COST_TO_DISTANCE_FACTOR_SQR_BITS\": \"0x%08x\", "
        "\"CELL_INFOS_TO_ALLOCATE\": %u},\n",
        MAX_WALL_PIECES, PATHFIND_QUEUE_LEN, MOVE_ALLIES_MAX_DEPTH,
        fbits(LAYER_Z_CLOSE_ENOUGH_F), PATHFIND_CELL_SIZE, fbits(PATHFIND_CELL_SIZE_F),
        ZONE_UPDATE_FREQUENCY, MAX_CELL_COUNT, MAX_ADJUSTMENT_CELL_COUNT,
        MAX_SAFE_PATH_CELL_COUNT, PATHFIND_CELLS_PER_FRAME,
        COST_ORTHOGONAL, COST_DIAGONAL,
        fbits(COST_TO_DISTANCE_FACTOR), fbits(COST_TO_DISTANCE_FACTOR_SQR),
        CELL_INFOS_TO_ALLOCATE);

    std::fprintf(fp, "  \"predicate_matrix\": [");
    for (size_t i = 0; i < pred_matrix.size(); ++i) {
        std::fprintf(fp, "%d%s", pred_matrix[i], (i + 1 < pred_matrix.size()) ? ", " : "");
    }
    std::fprintf(fp, "],\n  \"vectors\": [\n");

    ICoord2D coords[NUM_CELLS];
    for (Int i = 0; i < NUM_CELLS; ++i) {
        coords[i] = ICoord2D{ i % GRID_W, i / GRID_W };
    }

    PathfindCellInfo::allocateCellInfos();

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed * 0x9e3779b9u + 0x13572468u;
        bool initial_fixed = (seed % 2u == 1u);
        s_useFixedPathfinding = initial_fixed;
        s_forceCleanCells = false;

        PathfindCell cells[NUM_CELLS];
        PathfindCellList open_list;
        PathfindCellList closed_list;
        SimShadow shadow;
        std::vector<std::string> ops;
        std::vector<std::string> probes;

        auto emit = [&](const std::string& op, const std::string& pr) {
            ops.push_back(op);
            if (!pr.empty()) probes.push_back(pr);
        };

        auto do_obs = [&](Int c, Int id, Int rub, Int fence, Int trans) {
            Object o;
            o.m_id = static_cast<ObjectID>(id);
            o.m_body.m_damageState = rub ? BODY_RUBBLE : BODY_PRISTINE;
            o.m_transparent = trans ? KINDOF_CAN_SEE_THROUGH_STRUCTURE : KINDOF_INVALID;
            Bool res = cells[c].setTypeAsObstacle(&o, fence != 0, coords[c]);
            char ob[64], pr[64];
            std::snprintf(ob, sizeof(ob), "OBS:%d,%d,%d,%d,%d", c, id, rub, fence, trans);
            std::snprintf(pr, sizeof(pr), "obs=%u:%u:%u:%u:%u",
                          res ? 1u : 0u,
                          static_cast<UnsignedInt>(cells[c].getObstacleID()),
                          cells[c].isObstaclePresent(static_cast<ObjectID>(id)) ? 1u : 0u,
                          cells[c].isObstacleFence() ? 1u : 0u,
                          cells[c].isObstacleTransparent() ? 1u : 0u);
            emit(ob, pr);
        };

        auto do_rmobs = [&](Int c, Int id) {
            Object o;
            o.m_id = static_cast<ObjectID>(id);
            Bool res = cells[c].removeObstacle(&o);
            char ob[32], pr[32];
            std::snprintf(ob, sizeof(ob), "RMOBS:%d,%d", c, id);
            std::snprintf(pr, sizeof(pr), "rmobs=%u", res ? 1u : 0u);
            emit(ob, pr);
        };

        if (seed == 0 || seed == 1) {
            // Seed 0 (retail) & Seed 1 (fixed): obstacle, rubble, zone 14-bit wrap, layer, pinch, ally.
            do_obs(0, 101, 0, 1, 1);
            do_obs(0, 102, 0, 0, 0); // already obstacle -> fails
            do_rmobs(0, 999);        // wrong id -> fails
            do_rmobs(0, 101);        // matching id -> clears
            do_obs(1, 103, 1, 1, 1); // rubble -> CELL_RUBBLE
            do_rmobs(1, 103);        // removes rubble -> CELL_CLEAR (and obs_id mismatch check!)
            cells[2].setType(PathfindCell::CELL_WATER);
            emit("TYP:2,1", "");
            do_obs(2, 104, 0, 0, 0); // water blocks obstacle
            do_obs(3, 105, 0, 1, 0);
            if (s_useFixedPathfinding || cells[3].hasInfo()) {
                cells[3].setBlockedByAlly(TRUE);
                char pr[32];
                std::snprintf(pr, sizeof(pr), "ally=%u", cells[3].isBlockedByAlly() ? 1u : 0u);
                emit("ALLY:3,1", pr);
            }
            cells[4].setZone(0x4005u); // tests 14-bit masking -> 5
            cells[4].setPinched(TRUE);
            cells[4].setLayer(LAYER_WALL);
            cells[4].setConnectLayer(LAYER_GROUND);
            emit("ZONE:4,16389", "");
            emit("PINCH:4,1", "");
            emit("LAY:4,15", "");
            emit("CLAY:4,1", "");
        } else if (seed == 2) {
            // Cost calculations & parent chain turns (0, 45, 90, 135 deg), pinched, missing info.
            cells[0].allocateInfo(coords[0]);
            cells[1].allocateInfo(coords[1]);
            cells[5].allocateInfo(coords[5]);
            cells[6].allocateInfo(coords[6]);
            cells[9].allocateInfo(coords[9]);
            cells[10].allocateInfo(coords[10]);
            cells[15].allocateInfo(coords[15]);
            emit("ALLOC:0", ""); emit("ALLOC:1", ""); emit("ALLOC:5", "");
            emit("ALLOC:6", ""); emit("ALLOC:9", ""); emit("ALLOC:10", ""); emit("ALLOC:15", "");

            cells[1].setParentCell(&cells[0]);
            cells[5].setParentCell(&cells[1]);   // 90 deg turn
            cells[10].setParentCell(&cells[5]);  // 45 deg turn
            cells[9].setParentCell(&cells[10]);  // 135 deg turn!
            cells[6].setParentCellHierarchical(&cells[0]); // >1 cell distance ok for hier
            cells[6].clearParentCell();
            cells[10].setPinched(TRUE);

            emit("SPAR:1,0", ""); emit("SPAR:5,1", ""); emit("SPAR:10,5", "");
            emit("SPAR:9,10", ""); emit("SHPAR:6,0", ""); emit("CPAR:6", ""); emit("PINCH:10,1", "");

            char pr[256];
            UnsignedInt ctg = cells[0].costToGoal(&cells[15]);
            UnsignedInt cthg = cells[0].costToHierGoal(&cells[15]);
            UnsignedInt cthg_noinfo = cells[7].costToHierGoal(&cells[15]);
            UnsignedInt csf0 = cells[0].costSoFar(nullptr);
            UnsignedInt csf1 = cells[1].costSoFar(&cells[0]);
            UnsignedInt csf5 = cells[5].costSoFar(&cells[1]);
            UnsignedInt csf10 = cells[10].costSoFar(&cells[5]);
            UnsignedInt csf9 = cells[9].costSoFar(&cells[10]);
            UnsignedInt tcd_miss = cells[0].getTotalCostDifference(cells[7]);
            cells[1].setCostSoFar(10);
            cells[1].setTotalCost(45);
            cells[5].setTotalCost(120);
            UnsignedInt tcd_ok = cells[1].getTotalCostDifference(cells[5]);

            emit("CSF:1,10", "");
            emit("TCOST:1,45", "");
            emit("TCOST:5,120", "");
            std::snprintf(pr, sizeof(pr),
                          "ctg=%u,cthg=%u,cthgn=%u,csf=%u:%u:%u:%u:%u,tcd=%u:%u",
                          ctg, cthg, cthg_noinfo, csf0, csf1, csf5, csf10, csf9, tcd_miss, tcd_ok);
            emit("CTG:0,15", pr);
        } else if (seed == 3 || seed == 4) {
            // Seed 3 (retail) & Seed 4 (fixed): open list insertion order + FIFO ties + reverse sort + closed list.
            for (Int c = 0; c < 6; ++c) {
                cells[c].allocateInfo(coords[c]);
                char ob[16];
                std::snprintf(ob, sizeof(ob), "ALLOC:%d", c);
                emit(ob, "");
            }
            const UnsignedInt costs[6] = { 100, 50, 100, 200, 100, 180 };
            for (Int c = 0; c < 6; ++c) {
                cells[c].setTotalCost(costs[c]);
                char tb[24];
                std::snprintf(tb, sizeof(tb), "TCOST:%d,%u", c, costs[c]);
                emit(tb, "");
                cells[c].putOnSortedOpenList(open_list);
                shadow.in_open[c] = true;
                char ob[16];
                std::snprintf(ob, sizeof(ob), "OPEN:%d", c);
                emit(ob, "");
            }
            cells[1].removeFromOpenList(open_list);
            shadow.in_open[1] = false;
            cells[1].putOnClosedList(closed_list);
            shadow.in_closed[1] = true;
            cells[2].removeFromOpenList(open_list);
            shadow.in_open[2] = false;
            cells[2].putOnClosedList(closed_list);
            shadow.in_closed[2] = true;
            cells[1].removeFromClosedList(closed_list);
            shadow.in_closed[1] = false;

            emit("RMOPEN:1", ""); emit("CLOSED:1", "");
            emit("RMOPEN:2", ""); emit("CLOSED:2", "");
            emit("RMCL:1", "");

            // Also test startPathfind open-flag difference on cell 7 targeting cell 15.
            cells[7].allocateInfo(coords[7]);
            cells[15].allocateInfo(coords[15]);
            cells[7].startPathfind(&cells[15]);
            emit("ALLOC:7", ""); emit("ALLOC:15", "");
            char spr[32];
            std::snprintf(spr, sizeof(spr), "sp=%u:%u", cells[7].getOpen() ? 1u : 0u, cells[7].getTotalCost());
            emit("START:7,15", spr);
        } else if (seed == 5) {
            // Goal/Pos unit + Aircraft goal CellFlags state machine transitions.
            cells[0].setGoalUnit(static_cast<ObjectID>(11), coords[0]);
            cells[0].setPosUnit(static_cast<ObjectID>(11), coords[0]); // UNIT_PRESENT_FIXED
            cells[0].setPosUnit(static_cast<ObjectID>(22), coords[0]); // early return when already occupied!
            cells[0].setGoalUnit(static_cast<ObjectID>(33), coords[0]); // UNIT_GOAL_OTHER_MOVING
            cells[1].setPosUnit(static_cast<ObjectID>(44), coords[1]); // UNIT_PRESENT_MOVING
            cells[1].setGoalUnit(static_cast<ObjectID>(55), coords[1]); // UNIT_GOAL_OTHER_MOVING
            cells[1].setPosUnit(INVALID_ID, coords[1]);               // UNIT_GOAL
            cells[1].setGoalUnit(INVALID_ID, coords[1]);              // NO_UNITS -> releaseInfo!
            cells[2].setGoalAircraft(static_cast<ObjectID>(77), coords[2]);
            cells[2].setGoalAircraft(INVALID_ID, coords[2]);
            emit("GUNIT:0,11", ""); emit("PUNIT:0,11", ""); emit("PUNIT:0,22", ""); emit("GUNIT:0,33", "");
            emit("PUNIT:1,44", ""); emit("GUNIT:1,55", ""); emit("PUNIT:1,0", ""); emit("GUNIT:1,0", "");
            emit("GAIR:2,77", ""); emit("GAIR:2,0", "");
        } else if (seed == 6) {
            // Mode switch + forceCleanPathFindCellInfos + isObstructionInvalid + clearObstruction.
            do_obs(0, 200, 0, 1, 0);
            s_useFixedPathfinding = true;
            cells[0].clearObstruction(); // clears orphaned obstacle state
            emit("FIX:1", "");
            emit("CLROBS:0", "");
            cells[1].allocateInfo(coords[1]);
            cells[1].setTotalCost(50);
            cells[1].putOnSortedOpenList(open_list);
            shadow.in_open[1] = true;
            PathfindCell::releaseOpenList(open_list);
            shadow.in_open[1] = false;
            open_list.reset();
            PathfindCellInfo::forceCleanPathFindCellInfos();
            emit("ALLOC:1", ""); emit("TCOST:1,50", ""); emit("OPEN:1", "");
            emit("RELOPEN", ""); emit("FCLEAN", "");
        } else {
            // Fuzz valid deterministic state & open/closed list operations across 4x4 grid.
            Int steps = 12 + static_cast<Int>(splitmix32(rng) % 10u);
            for (Int s = 0; s < steps; ++s) {
                uint32_t kind = splitmix32(rng) % 14u;
                Int c = static_cast<Int>(splitmix32(rng) % NUM_CELLS);
                if (kind == 0) {
                    if (!shadow.in_open[c] && !shadow.in_closed[c] && cells[c].getFlags() == PathfindCell::NO_UNITS) {
                        Int id = static_cast<Int>(100u + (splitmix32(rng) % 30u));
                        Int rub = (splitmix32(rng) % 4u == 0) ? 1 : 0;
                        Int fence = splitmix32(rng) & 1;
                        Int trans = splitmix32(rng) & 1;
                        do_obs(c, id, rub, fence, trans);
                    } else {
                        Int t = static_cast<Int>(splitmix32(rng) % 7u);
                        if (t != PathfindCell::CELL_OBSTACLE && cells[c].getObstacleID() == INVALID_ID) {
                            cells[c].setType(static_cast<PathfindCell::CellType>(t));
                            char tb[24];
                            std::snprintf(tb, sizeof(tb), "TYP:%d,%d", c, t);
                            emit(tb, "");
                        }
                    }
                } else if (kind == 1) {
                    Int id = (splitmix32(rng) % 2u == 0 && cells[c].getObstacleID() != INVALID_ID)
                        ? static_cast<Int>(cells[c].getObstacleID())
                        : static_cast<Int>(100u + (splitmix32(rng) % 5u));
                    if (!shadow.in_open[c] && !shadow.in_closed[c]) {
                        do_rmobs(c, id);
                    }
                } else if (kind == 2) {
                    if (!cells[c].hasInfo()) {
                        Bool ok = cells[c].allocateInfo(coords[c]);
                        char ab[24], pr[16];
                        std::snprintf(ab, sizeof(ab), "ALLOC:%d", c);
                        std::snprintf(pr, sizeof(pr), "alloc=%u", ok ? 1u : 0u);
                        emit(ab, pr);
                    } else if (!shadow.in_open[c] && !shadow.in_closed[c]) {
                        cells[c].releaseInfo();
                        char rb[24];
                        std::snprintf(rb, sizeof(rb), "REL:%d", c);
                        emit(rb, "");
                    }
                } else if (kind == 3) {
                    if (cells[c].hasInfo() && !shadow.in_open[c] && !shadow.in_closed[c]) {
                        UnsignedInt cost = splitmix32(rng) % 400u;
                        cells[c].setTotalCost(cost);
                        char tb[24];
                        std::snprintf(tb, sizeof(tb), "TCOST:%d,%u", c, cost);
                        emit(tb, "");
                        cells[c].putOnSortedOpenList(open_list);
                        shadow.in_open[c] = true;
                        char ob[16];
                        std::snprintf(ob, sizeof(ob), "OPEN:%d", c);
                        emit(ob, "");
                    }
                } else if (kind == 4) {
                    if (shadow.in_open[c] && !shadow.in_closed[c]) {
                        cells[c].removeFromOpenList(open_list);
                        shadow.in_open[c] = false;
                        char rb[16];
                        std::snprintf(rb, sizeof(rb), "RMOPEN:%d", c);
                        emit(rb, "");
                    }
                } else if (kind == 5) {
                    if (cells[c].hasInfo() && !shadow.in_open[c] && !shadow.in_closed[c]) {
                        cells[c].putOnClosedList(closed_list);
                        shadow.in_closed[c] = true;
                        char cb[16];
                        std::snprintf(cb, sizeof(cb), "CLOSED:%d", c);
                        emit(cb, "");
                    }
                } else if (kind == 6) {
                    if (shadow.in_closed[c] && !shadow.in_open[c]) {
                        cells[c].removeFromClosedList(closed_list);
                        shadow.in_closed[c] = false;
                        char rb[16];
                        std::snprintf(rb, sizeof(rb), "RMCL:%d", c);
                        emit(rb, "");
                    }
                } else if (kind == 7) {
                    UnsignedInt z = splitmix32(rng) % 20000u;
                    cells[c].setZone(static_cast<UnsignedShort>(z));
                    char zb[24];
                    std::snprintf(zb, sizeof(zb), "ZONE:%d,%u", c, z);
                    emit(zb, "");
                } else if (kind == 8) {
                    Int pinch = splitmix32(rng) & 1;
                    cells[c].setPinched(pinch != 0);
                    char pb[24];
                    std::snprintf(pb, sizeof(pb), "PINCH:%d,%d", c, pinch);
                    emit(pb, "");
                } else if (kind == 9) {
                    Int l = (splitmix32(rng) % 3u == 0) ? LAYER_GROUND : (splitmix32(rng) % 3u == 1 ? LAYER_WALL : LAYER_INVALID);
                    cells[c].setLayer(static_cast<PathfindLayerEnum>(l));
                    char lb[24];
                    std::snprintf(lb, sizeof(lb), "LAY:%d,%d", c, l);
                    emit(lb, "");
                    if (s_useFixedPathfinding || cells[c].hasInfo()) {
                        Int ally = splitmix32(rng) & 1;
                        cells[c].setBlockedByAlly(ally != 0);
                        char ab[24], pr[24];
                        std::snprintf(ab, sizeof(ab), "ALLY:%d,%d", c, ally);
                        std::snprintf(pr, sizeof(pr), "ally=%u", cells[c].isBlockedByAlly() ? 1u : 0u);
                        emit(ab, pr);
                    }
                } else if (kind == 10) {
                    if (!shadow.in_open[c] && !shadow.in_closed[c] && cells[c].getType() != PathfindCell::CELL_OBSTACLE) {
                        Int unit = static_cast<Int>(10u + (splitmix32(rng) % 3u));
                        if (splitmix32(rng) & 1) {
                            cells[c].setGoalUnit(static_cast<ObjectID>(unit), coords[c]);
                            char gb[24];
                            std::snprintf(gb, sizeof(gb), "GUNIT:%d,%d", c, unit);
                            emit(gb, "");
                        } else {
                            cells[c].setPosUnit(static_cast<ObjectID>(unit), coords[c]);
                            char pb[24];
                            std::snprintf(pb, sizeof(pb), "PUNIT:%d,%d", c, unit);
                            emit(pb, "");
                        }
                    }
                } else if (kind == 11) {
                    Int g = static_cast<Int>(splitmix32(rng) % NUM_CELLS);
                    if (cells[g].hasInfo()) {
                        UnsignedInt hcost = cells[c].costToHierGoal(&cells[g]);
                        char ob[24], pr[24];
                        std::snprintf(ob, sizeof(ob), "CTHG:%d,%d", c, g);
                        std::snprintf(pr, sizeof(pr), "cthg=%u", hcost);
                        emit(ob, pr);
                    }
                } else if (kind == 12) {
                    Int b = static_cast<Int>(splitmix32(rng) % NUM_CELLS);
                    UnsignedInt tcd = cells[c].getTotalCostDifference(cells[b]);
                    char ob[24], pr[32];
                    std::snprintf(ob, sizeof(ob), "TCD:%d,%d", c, b);
                    std::snprintf(pr, sizeof(pr), "tcd=%u", tcd);
                    emit(ob, pr);
                } else {
                    Int g = static_cast<Int>(splitmix32(rng) % NUM_CELLS);
                    if (cells[c].hasInfo() && cells[g].hasInfo()) {
                        UnsignedInt ctg = cells[c].costToGoal(&cells[g]);
                        char ob[24], pr[24];
                        std::snprintf(ob, sizeof(ob), "CTG:%d,%d", c, g);
                        std::snprintf(pr, sizeof(pr), "ctg=%u", ctg);
                        emit(ob, pr);
                    }
                }
            }
        }

        // Snapshot state before cleanup.
        std::vector<Int> open_ids, closed_ids;
        record_list(open_list.getHead(), cells, open_ids);
        record_list(closed_list.getHead(), cells, closed_ids);
        Int open_head = cell_ptr_to_idx(open_list.getHead(), cells);
        Int open_tail = CellInspector::indexOfTail(open_list, cells);
        Int closed_head = cell_ptr_to_idx(closed_list.getHead(), cells);
        Int closed_tail = CellInspector::indexOfTail(closed_list, cells);
        UnsignedInt end_free = InfoInspector::countFree();
        std::string cells_str = snapshot_cells(cells, cells);

        std::string ops_joined, probes_joined;
        for (size_t i = 0; i < ops.size(); ++i) {
            if (i > 0) ops_joined.push_back(';');
            ops_joined.append(ops[i]);
        }
        for (size_t i = 0; i < probes.size(); ++i) {
            if (i > 0) probes_joined.push_back(';');
            probes_joined.append(probes[i]);
        }

        std::fprintf(fp,
            "    {\"seed\": %u, \"fixed\": %u, \"ops\": \"%s\", \"probes\": \"%s\", "
            "\"open_head\": %d, \"open_tail\": %d, \"closed_head\": %d, \"closed_tail\": %d, "
            "\"end_free\": %u, \"open_ids\": [",
            seed, initial_fixed ? 1u : 0u, ops_joined.c_str(), probes_joined.c_str(),
            open_head, open_tail, closed_head, closed_tail, end_free);
        for (size_t i = 0; i < open_ids.size(); ++i) {
            std::fprintf(fp, "%d%s", open_ids[i], (i + 1 < open_ids.size()) ? ", " : "");
        }
        std::fprintf(fp, "], \"closed_ids\": [");
        for (size_t i = 0; i < closed_ids.size(); ++i) {
            std::fprintf(fp, "%d%s", closed_ids[i], (i + 1 < closed_ids.size()) ? ", " : "");
        }
        std::fprintf(fp, "], \"cells\": \"%s\"}%s\n",
                     cells_str.c_str(), (seed + 1u < 1000u) ? "," : "");

        // Deterministic cleanup per seed.
        PathfindCell::releaseOpenList(open_list);
        PathfindCell::releaseClosedList(closed_list);
        open_list.reset();
        closed_list.reset();
        for (Int i = 0; i < NUM_CELLS; ++i) {
            cells[i].reset();
        }
        if (InfoInspector::countFree() != CELL_INFOS_TO_ALLOCATE) {
            std::fprintf(stderr, "Seed %u: info pool leak (%u != %u)\n",
                         seed, InfoInspector::countFree(), CELL_INFOS_TO_ALLOCATE);
            return 1;
        }
    }

    PathfindCellInfo::releaseCellInfos();

    std::fprintf(fp, "  ]\n}\n");
    if (std::fflush(fp) != 0 || std::fclose(fp) != 0) {
        std::perror("flush/close");
        std::remove(tmp_out.c_str());
        return 1;
    }
    if (std::rename(tmp_out.c_str(), out_path) != 0) {
        std::perror("rename");
        return 1;
    }
    return 0;
}
