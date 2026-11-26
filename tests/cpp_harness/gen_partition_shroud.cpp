// Partition shroud: drives the real PartitionManager's cells, reveals, queue,
// map-wide reveals, boundary store/restore, crc and xfer.
//
// Canonical 3-mode build & run:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_partition_shroud.cpp --opt O0 -o <dir>/gen_ps_o0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_partition_shroud.cpp --opt O2 -o <dir>/gen_ps_o2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_partition_shroud.cpp --opt O2 --sanitize -o <dir>/gen_ps_ubsan
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy <dir>/gen_ps_o2 docs/golden_vectors/PARTITION_SHROUD.json
//
// A vector is a script of operations on one grid. Every operation row ends with
// the manager's XferCRC and its XferSave stream ("<length>:<FNV-1a 64>"). The
// first kReadable vectors also list every cell level that is not {1,0}, as
// "cell.player=current/active".
//
// Operations: 0 reveal, 1 undo reveal, 2 cover, 3 undo cover, 4 queue undo
// (all: x,y,radius bits, mask), 5 set frame, 6 update, 7 flush the queue,
// 8 reveal map, 9 reveal permanently, 10 undo permanent reveal, 11 shroud map
// (all: player), 12 status query (player,x,y bits,result), 13 store (flag),
// 14 new boundary (grid), 15 restore (flag).
//
// The local player is replaced by one with index 99 so that no cell reaches the
// display or radar. No objects are filed. Not covered: loading, players above 15,
// NaN or infinite inputs, radii beyond about 130 cells.

#include "oracle_wp51_world.h"

#include "Common/XferCRC.h"
#include "GameLogic/PartitionManager.h"
#include "Common/GameUtility.h"

#include <cstdio>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

struct PsCellsTag { using type = PartitionCell* PartitionManager::*; };
template struct MemberAccess<PsCellsTag, &PartitionManager::m_cells>;
inline PartitionCell* PartitionManager::* access_member(PsCellsTag);
struct PsLevelTag { using type = ShroudLevel (PartitionCell::*)[MAX_PLAYER_COUNT]; };
template struct MemberAccess<PsLevelTag, &PartitionCell::m_shroudLevel>;
inline ShroudLevel (PartitionCell::* access_member(PsLevelTag))[MAX_PLAYER_COUNT];
struct PsQueueTag { using type = std::queue<SightingInfo*> PartitionManager::*; };
template struct MemberAccess<PsQueueTag, &PartitionManager::m_pendingUndoShroudReveals>;
inline std::queue<SightingInfo*> PartitionManager::* access_member(PsQueueTag);
struct PsLocalTag { using type = Player* PlayerList::*; };
template struct MemberAccess<PsLocalTag, &PlayerList::m_local>;
inline Player* PlayerList::* access_member(PsLocalTag);

static const int kVectors = 700;
static const int kReadable = 10;

static uint32_t bits(Real v) { uint32_t b; std::memcpy(&b, &v, 4); return b; }

static uint32_t g_state = 0x51D0C4E7u;
static uint32_t next_u32() {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state;
}
static uint32_t pick(uint32_t n) { return next_u32() % n; }
static Real spread(Real lo, Real hi) {
    return lo + (static_cast<Real>(next_u32() % 100000u) / 100000.0f) * (hi - lo);
}

static void die(const char* what) { std::fprintf(stderr, "harness: %s\n", what); std::exit(1); }

static uint32_t g_unlooks_run = 0, g_unlooks_waiting = 0, g_short_restores = 0, g_positive = 0,
                g_jammed = 0;

struct Grid { Real cs, lox, loy, hix, hiy; };

static Grid make_grid(bool tiny) {
    const Real sizes[8] = {40.0f, 10.0f, 20.0f, 1.0f, 0.5f, 3.7f, 33.333f, 100.0f};
    const Real lows[6] = {0.0f, -200.0f, -37.5f, 13.3f, -1000.5f, 500.0f};
    Grid g;
    g.cs = sizes[pick(8)];
    const Real cs = g.cs < 1.0f ? 1.0f : g.cs;
    g.lox = lows[pick(6)];
    g.loy = lows[pick(6)];
    const bool big = !tiny && pick(15) == 0;
    const int nx = tiny ? 1 + pick(5) : (big ? 30 + pick(25) : 1 + pick(14));
    const int ny = tiny ? 1 + pick(5) : (big ? 30 + pick(25) : 1 + pick(14));
    g.hix = g.lox + nx * cs - (pick(2) ? spread(0.0f, 0.99f) * cs : 0.0f);
    g.hiy = g.loy + ny * cs - (pick(2) ? spread(0.0f, 0.99f) * cs : 0.0f);
    if (pick(20) == 0) g.hix = g.lox;
    return g;
}

static void set_grid(oracle_wp51::OracleTerrainLogic* terrain, const Grid& g) {
    TheWritableGlobalData->m_partitionCellSize = g.cs;
    terrain->extent.lo = {g.lox, g.loy, 0.0f};
    terrain->extent.hi = {g.hix, g.hiy, 100.0f};
    ThePartitionManager->reset();
    ThePartitionManager->init();
}

static std::string grid_json(const Grid& g) {
    std::ostringstream ss;
    ss << bits(g.cs) << ',' << bits(g.lox) << ',' << bits(g.loy) << ',' << bits(g.hix) << ','
       << bits(g.hiy);
    return ss.str();
}

static Real make_coord(Real lo, Real hi, Real cs) {
    switch (pick(10)) {
    case 0: return lo + static_cast<Real>(static_cast<int>(pick(8)) - 1) * cs;
    case 1: return pick(2) ? lo - spread(0.0f, 8.0f * cs) : hi + spread(0.0f, 8.0f * cs);
    case 2: return pick(2) ? lo : hi;
    default: return spread(lo, hi > lo ? hi : lo + cs);
    }
}

static Real make_radius(Real cs) {
    const Real f[10] = {0.0f, 0.5f, 1.0f, 1.0001f, 2.0f, 3.0f, 4.0f, 5.0f, 7.0f, 11.0f};
    switch (pick(10)) {
    case 0: return -spread(0.0f, 3.0f * cs);
    case 1: return spread(0.0f, 6.0f * cs);
    case 2: return pick(4) ? spread(0.0f, 20.0f * cs) : spread(20.0f * cs, 130.0f * cs);
    default: return f[pick(10)] * cs;
    }
}

static unsigned make_mask() {
    switch (pick(6)) {
    case 0: return 0xFFFFu;
    case 1: return next_u32() & 0xFFFFu;
    case 2: return 0u;
    default: {
        unsigned m = 1u << pick(5);
        if (pick(2)) m |= 1u << pick(16);
        return m;
    }
    }
}

static int make_player() { return pick(3) ? static_cast<int>(pick(4)) : static_cast<int>(pick(16)); }

static std::string state_json(bool readable) {
    XferCRC crc;
    crc.open("crc");
    ThePartitionManager->crc(&crc);
    crc.close();

    oracle::MemoryXfer save(XFER_SAVE);
    save.open("save");
    ThePartitionManager->xfer(&save);
    save.close();
    uint64_t h = 0xcbf29ce484222325ull;
    for (uint8_t c : save.bytes) { h ^= c; h *= 0x100000001b3ull; }

    char out[80];
    std::snprintf(out, sizeof(out), "%u,\"%zu:%016llx\"", crc.getCRC(), save.bytes.size(),
                  static_cast<unsigned long long>(h));
    std::string s = out;

    PartitionCell* cells = ThePartitionManager->*access_member(PsCellsTag());
    const int total = ThePartitionManager->getCellCountX() * ThePartitionManager->getCellCountY();
    std::ostringstream levels;
    for (int i = 0; i < total; ++i) {
        for (int p = 0; p < MAX_PLAYER_COUNT; ++p) {
            const ShroudLevel& l = (cells[i].*access_member(PsLevelTag()))[p];
            if (l.m_currentShroud > 1) ++g_positive;
            if (l.m_activeShroudLevel > 0) ++g_jammed;
            if (readable && (l.m_currentShroud != 1 || l.m_activeShroudLevel != 0))
                levels << i << '.' << p << '=' << l.m_currentShroud << '/' << l.m_activeShroudLevel
                       << ' ';
        }
    }
    if (readable) s += ",\"" + levels.str() + "\"";
    return s;
}

static size_t queue_size() { return (ThePartitionManager->*access_member(PsQueueTag())).size(); }

static std::string run_vector(oracle_wp51::OracleTerrainLogic* terrain, oracle_wp51::World& w,
                              bool readable) {
    Grid grid = make_grid(readable);
    set_grid(terrain, grid);
    const int players = pick(3) ? 3 + pick(4) : 1 + pick(16);
    ThePlayerList->*access_member(Wp51PlayerListCountTag()) = players;
    const UnsignedInt persists[6] = {0u, 1u, 5u, 30u, 30u, 0xFFFFFFF0u};
    const UnsignedInt persist = persists[pick(6)];
    TheWritableGlobalData->m_unlookPersistDuration = persist;
    UnsignedInt frame = pick(8) == 0 ? 0xFFFFFFC0u + pick(60) : pick(100);
    w.set_frame(frame);

    std::ostringstream ss;
    ss << "{\"g\":[" << grid_json(grid) << "],\"players\":" << players << ",\"persist\":" << persist
       << ",\"frame\":" << frame << ",\"s\":[";
    bool first = true;
    auto row = [&](const std::string& op) {
        ss << (first ? "" : ",") << "[" << op << "," << state_json(readable) << "]";
        first = false;
    };

    // Reveals made so far, so that undos mostly match one.
    struct Circle { Real x, y, r; unsigned mask; };
    std::vector<Circle> made;
    const Real cs0 = ThePartitionManager->getCellSize();
    auto circle = [&](bool reuse) {
        if (reuse && !made.empty() && pick(5)) {
            const size_t at = pick(static_cast<uint32_t>(made.size()));
            const Circle c = made[at];
            made.erase(made.begin() + at);
            return c;
        }
        return Circle{make_coord(grid.lox, grid.hix, cs0), make_coord(grid.loy, grid.hiy, cs0),
                      make_radius(cs0), make_mask()};
    };
    auto op_circle = [&](int op, const Circle& c) {
        const PlayerMaskType mask = static_cast<PlayerMaskType>(c.mask);
        switch (op) {
        case 0: ThePartitionManager->doShroudReveal(c.x, c.y, c.r, mask); break;
        case 1: ThePartitionManager->undoShroudReveal(c.x, c.y, c.r, mask); break;
        case 2: ThePartitionManager->doShroudCover(c.x, c.y, c.r, mask); break;
        case 3: ThePartitionManager->undoShroudCover(c.x, c.y, c.r, mask); break;
        default: ThePartitionManager->queueUndoShroudReveal(c.x, c.y, c.r, mask); break;
        }
        std::ostringstream o;
        o << op << ',' << bits(c.x) << ',' << bits(c.y) << ',' << bits(c.r) << ',' << c.mask;
        row(o.str());
    };
    auto op_player = [&](int op, int p) {
        switch (op) {
        case 8: ThePartitionManager->revealMapForPlayer(p); break;
        case 9: ThePartitionManager->revealMapForPlayerPermanently(p); break;
        case 10: ThePartitionManager->undoRevealMapForPlayerPermanently(p); break;
        default: ThePartitionManager->shroudMapForPlayer(p); break;
        }
        row(std::to_string(op) + "," + std::to_string(p));
    };
    auto set_frame = [&](UnsignedInt f) {
        frame = f;
        w.set_frame(frame);
        row("5," + std::to_string(frame));
    };
    auto update = [&]() {
        const size_t before = queue_size();
        ThePartitionManager->update();
        g_unlooks_run += static_cast<uint32_t>(before - queue_size());
        g_unlooks_waiting += static_cast<uint32_t>(queue_size());
        row("6");
    };
    auto flush = [&]() {
        ThePartitionManager->processEntirePendingUndoShroudRevealQueue();
        row("7");
    };

    // setActiveBoundary: flush, store fog, objects leave, store reveals, rebuild,
    // restore reveals, objects return, restore fog.
    auto boundary = [&]() {
        ShroudStatusStoreRestore store;
        flush();
        ThePartitionManager->storeFoggedCells(store, TRUE);
        row("13,1");
        for (int n = pick(3); n > 0; --n) op_circle(pick(2) ? 1 : 4, circle(true));
        ThePartitionManager->storeFoggedCells(store, FALSE);
        row("13,0");
        const Grid old = grid;
        grid = make_grid(readable);
        // Keep the cell size: restoring maps cells by index.
        const Real cs = old.cs;
        const Real w_new = (grid.hix - grid.lox) / (grid.cs < 1.0f ? 1.0f : grid.cs);
        const Real h_new = (grid.hiy - grid.loy) / (grid.cs < 1.0f ? 1.0f : grid.cs);
        grid.cs = cs;
        grid.hix = grid.lox + w_new * (cs < 1.0f ? 1.0f : cs);
        grid.hiy = grid.loy + h_new * (cs < 1.0f ? 1.0f : cs);
        const int old_rows = ThePartitionManager->getCellCountY();
        set_grid(terrain, grid);
        if (ThePartitionManager->getCellCountY() < old_rows) ++g_short_restores;
        made.clear();
        row("14," + grid_json(grid));
        ThePartitionManager->restoreFoggedCells(store, FALSE);
        row("15,0");
        for (int n = pick(3); n > 0; --n) {
            const Circle c = circle(false);
            made.push_back(c);
            op_circle(0, c);
        }
        ThePartitionManager->restoreFoggedCells(store, TRUE);
        row("15,1");
    };

    const int ops = 8 + pick(18);
    for (int k = 0; k < ops; ++k) {
        switch (pick(26)) {
        case 0: case 1: case 2: case 3: case 4: {
            const Circle c = circle(false);
            made.push_back(c);
            op_circle(0, c);
            break;
        }
        case 5: case 6: op_circle(1, circle(true)); break;
        case 7: case 8: op_circle(2, circle(false)); break;
        case 9: op_circle(3, circle(false)); break;
        case 10: case 11: case 12: op_circle(4, circle(true)); break;
        case 13: case 14: {
            const UnsignedInt step[6] = {1u, 2u, 5u, 29u, 31u, 100u};
            set_frame(frame + step[pick(6)]);
            update();
            break;
        }
        case 15: update(); break;
        case 16: flush(); break;
        case 17: op_player(8, make_player()); break;
        case 18: if (pick(2)) op_player(9, make_player()); break;
        case 19: if (pick(3) == 0) op_player(10, make_player()); break;
        case 20: op_player(11, make_player()); break;
        case 21: case 22: case 23: {
            Coord3D at;
            at.x = make_coord(grid.lox, grid.hix, cs0);
            at.y = make_coord(grid.loy, grid.hiy, cs0);
            at.z = 0.0f;
            const int p = pick(8) == 0 ? -1 : make_player();
            const int result = static_cast<int>(ThePartitionManager->getShroudStatusForPlayer(p, &at));
            std::ostringstream o;
            o << "12," << p << ',' << bits(at.x) << ',' << bits(at.y) << ',' << result;
            row(o.str());
            break;
        }
        case 24: if (pick(2)) boundary(); break;
        default: set_frame(pick(2) ? frame + 1 : frame); break;
        }
    }
    update();
    ss << "]}";
    return ss.str();
}

int main(int argc, char** argv) {
    if (argc < 2) { std::fprintf(stderr, "usage: gen_partition_shroud <out.json>\n"); return 2; }
    const std::string out_path = argv[1];
    oracle_wp51::World w;
    w.load_ini("PlayerTemplate FactionPS\n  Side = America\n  PlayableSide = Yes\nEnd\n");
    w.init_with_neutral_player("FactionPS", nullptr);
    auto* terrain = static_cast<oracle_wp51::OracleTerrainLogic*>(TheTerrainLogic);
    Player* nobody = NEW Player(99);
    ThePlayerList->*access_member(PsLocalTag()) = nobody;
    if (rts::getObservedOrLocalPlayer()->getPlayerIndex() != 99) die("local player");
    for (int p = 0; p < MAX_PLAYER_COUNT; ++p)
        if (ThePlayerList->getNthPlayer(p)->getPlayerMask() != (1 << p)) die("player mask");

    std::vector<std::string> vectors;
    for (int i = 0; i < kVectors; ++i)
        vectors.push_back(run_vector(terrain, w, i < kReadable));

    const std::string tmp = out_path + ".tmp";
    FILE* f = std::fopen(tmp.c_str(), "wb");
    if (!f) { std::perror("open"); return 1; }
    const std::string sha = oracle::sha256_file(
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/PartitionManager.cpp");
    std::fprintf(f, "{\n\"packet\": \"PARTITION_SHROUD\",\n\"vector_count\": %d,\n", kVectors);
    std::fprintf(f, "\"sha256_PartitionManager\": \"%s\",\n\"vectors\": [\n", sha.c_str());
    for (size_t i = 0; i < vectors.size(); ++i)
        std::fprintf(f, "%s%s\n", vectors[i].c_str(), i + 1 < vectors.size() ? "," : "");
    std::fprintf(f, "]\n}\n");
    std::fclose(f);
    if (std::rename(tmp.c_str(), out_path.c_str())) { std::perror("rename"); return 1; }
    std::fprintf(stderr, "coverage: unlooks_run=%u waiting=%u short_restores=%u positive=%u jammed=%u\n",
                 g_unlooks_run, g_unlooks_waiting, g_short_restores, g_positive, g_jammed);
    return 0;
}
