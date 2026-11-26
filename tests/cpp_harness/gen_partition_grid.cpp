// Partition grid filing: drives the real PartitionManager with real Objects.
//
// Canonical 3-mode build & run:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_partition_grid.cpp --opt O0 -o <dir>/gen_pg_o0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_partition_grid.cpp --opt O2 -o <dir>/gen_pg_o2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_partition_grid.cpp --opt O2 --sanitize -o <dir>/gen_pg_ubsan
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy <dir>/gen_pg_o2 docs/golden_vectors/PARTITION_GRID.json
//
// Each vector is a script of operations on up to eight objects over one grid
// configuration. Every operation row ends with the dirty list and the module list
// (head first, object ids are single digits). An update row also records every
// non-empty cell's id list, every registered object's center cell, cell array size
// and cells, and the objects whose center cell changed. A dump longer than 160 characters is
// stored as "#<length>:<FNV-1a 64>", except in the first kReadable vectors.
//
// Objects carry OBJECT_STATUS_NO_COLLISIONS, so processContactList does not
// re-dirty colliding pairs: collision handling is outside this port.
//
// doSmallFill writes cells without a capacity check. A script that would overrun
// the array (undefined behaviour) is discarded and drawn again;
// "small_overflow_dropped" counts them.
//
// Not covered: ghost objects, immobile objects, shroud/threat state, NaN or
// infinite inputs, radii beyond about 130 cells, loading a save.

#include "oracle_wp51_world.h"

#include "Common/ThingFactory.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"

#include <cmath>
#include <cstdio>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

struct PgModuleListTag { using type = PartitionData* PartitionManager::*; };
template struct MemberAccess<PgModuleListTag, &PartitionManager::m_moduleList>;
inline PartitionData* PartitionManager::* access_member(PgModuleListTag);
struct PgDirtyTag { using type = PartitionData* PartitionManager::*; };
template struct MemberAccess<PgDirtyTag, &PartitionManager::m_dirtyModules>;
inline PartitionData* PartitionManager::* access_member(PgDirtyTag);
struct PgNextDirtyTag { using type = PartitionData* PartitionData::*; };
template struct MemberAccess<PgNextDirtyTag, &PartitionData::m_nextDirty>;
inline PartitionData* PartitionData::* access_member(PgNextDirtyTag);
struct PgCoiTag { using type = CellAndObjectIntersection* PartitionData::*; };
template struct MemberAccess<PgCoiTag, &PartitionData::m_coiArray>;
inline CellAndObjectIntersection* PartitionData::* access_member(PgCoiTag);
struct PgCoiCapTag { using type = Int PartitionData::*; };
template struct MemberAccess<PgCoiCapTag, &PartitionData::m_coiArrayCount>;
inline Int PartitionData::* access_member(PgCoiCapTag);
struct PgLastCellTag { using type = const PartitionCell* PartitionData::*; };
template struct MemberAccess<PgLastCellTag, &PartitionData::m_lastCell>;
inline const PartitionCell* PartitionData::* access_member(PgLastCellTag);
struct PgCellsTag { using type = PartitionCell* PartitionManager::*; };
template struct MemberAccess<PgCellsTag, &PartitionManager::m_cells>;
inline PartitionCell* PartitionManager::* access_member(PgCellsTag);
struct PgMaxRadiusTag { using type = Int PartitionManager::*; };
template struct MemberAccess<PgMaxRadiusTag, &PartitionManager::m_maxGcoRadius>;
inline Int PartitionManager::* access_member(PgMaxRadiusTag);
struct PgRadiusVecTag { using type = std::vector<std::vector<ICoord2D>> PartitionManager::*; };
template struct MemberAccess<PgRadiusVecTag, &PartitionManager::m_radiusVec>;
inline std::vector<std::vector<ICoord2D>> PartitionManager::* access_member(PgRadiusVecTag);

static const int kObjects = 8;
static const int kGrids = 90;
static const int kVectors = 1200;
static const int kReadable = 12;
static const size_t kInline = 160;

static uint32_t bits(Real v) { uint32_t b; std::memcpy(&b, &v, 4); return b; }

static uint32_t g_state = 0x9A17C311u;
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

struct Grid { Real cs, lox, loy, hix, hiy; };
struct Geom { int type; bool small; Real height, major, minor; };

static Object* g_obj[kObjects];
static uint32_t g_small_dropped = 0, g_small_wide = 0, g_full = 0, g_off_map = 0;

static void die(const char* what) { std::fprintf(stderr, "harness: %s\n", what); std::exit(1); }

static std::string fnv(const std::string& s) {
    uint64_t h = 0xcbf29ce484222325ull;
    for (unsigned char c : s) { h ^= c; h *= 0x100000001b3ull; }
    char out[48];
    std::snprintf(out, sizeof(out), "#%zu:%016llx", s.size(), static_cast<unsigned long long>(h));
    return out;
}
static std::string pack(const std::string& s, bool readable) {
    return "\"" + ((readable || s.size() <= kInline) ? s : fnv(s)) + "\"";
}

static char digit_of(const PartitionData* data) {
    for (int i = 0; i < kObjects; ++i)
        if (g_obj[i]->friend_getPartitionData() == data) return static_cast<char>('1' + i);
    die("unknown partition data");
    return '?';
}

static std::string dirty_list() {
    std::string s;
    for (PartitionData* d = ThePartitionManager->*access_member(PgDirtyTag()); d;
         d = d->*access_member(PgNextDirtyTag()))
        s += digit_of(d);
    return s;
}
static std::string module_list() {
    std::string s;
    for (PartitionData* d = ThePartitionManager->*access_member(PgModuleListTag()); d; d = d->getNext())
        s += digit_of(d);
    return s;
}

static int cell_index_of(const PartitionCell* cell) {
    return static_cast<int>(cell - ThePartitionManager->*access_member(PgCellsTag()));
}

static std::string cell_dump() {
    std::string s;
    const int total = ThePartitionManager->getCellCountX() * ThePartitionManager->getCellCountY();
    PartitionCell* cells = ThePartitionManager->*access_member(PgCellsTag());
    for (int i = 0; i < total; ++i) {
        if (!cells[i].getFirstCoiInCell()) {
            if (cells[i].getCoiCount() != 0) die("count of empty cell");
            continue;
        }
        s += std::to_string(i) + ":";
        int n = 0;
        for (CellAndObjectIntersection* c = cells[i].getFirstCoiInCell(); c; c = c->getNextCoi(), ++n)
            s += digit_of(c->getModule());
        if (n != cells[i].getCoiCount()) die("cell count");
        s += ",";
    }
    return s;
}

static std::string object_dump() {
    std::string s;
    for (int i = 0; i < kObjects; ++i) {
        PartitionData* d = g_obj[i]->friend_getPartitionData();
        if (!d) continue;
        const PartitionCell* last = d->*access_member(PgLastCellTag());
        s += static_cast<char>('1' + i);
        s += "@" + (last ? std::to_string(cell_index_of(last)) : std::string("-"));
        s += "/" + std::to_string(d->*access_member(PgCoiCapTag())) + ":";
        CellAndObjectIntersection* coi = d->*access_member(PgCoiTag());
        const int used = d->friend_getCoiInUseCount();
        if (used > d->*access_member(PgCoiCapTag())) die("coi overrun");
        if (used && used == d->*access_member(PgCoiCapTag())) ++g_full;
        for (int k = 0; k < used; ++k) {
            if (k) s += ".";
            s += std::to_string(cell_index_of(coi[k].getCell()));
        }
        s += ",";
    }
    return s;
}

// True if the next update would make doSmallFill write past the cell array.
static bool small_fill_overflows() {
    for (int i = 0; i < kObjects; ++i) {
        PartitionData* d = g_obj[i]->friend_getPartitionData();
        const GeometryInfo& g = g_obj[i]->getGeometryInfo();
        if (!d || !g.getIsSmall()) continue;
        Real r = g.getMajorRadius();
        const Real half = ThePartitionManager->getCellSize() * 0.5f;
        if (r > half) r = half;
        const Coord3D* p = g_obj[i]->getPosition();
        Int x1, y1, x2, y2;
        ThePartitionManager->worldToCell(p->x - r, p->y - r, &x1, &y1);
        ThePartitionManager->worldToCell(p->x + r, p->y + r, &x2, &y2);
        int n = 0;
        for (Int x = x1; x <= x2; ++x)
            for (Int y = y1; y <= y2; ++y)
                if (ThePartitionManager->getCellAt(x, y)) ++n;
        if (n > d->*access_member(PgCoiCapTag())) return true;
        if (x2 - x1 > 1 || y2 - y1 > 1) ++g_small_wide;
    }
    return false;
}

static Real effective_cs(const Grid& g) { return g.cs < 1.0f ? 1.0f : g.cs; }

static Grid make_grid(int i) {
    if (i == 0) return {40.0f, -80.0f, -40.0f, 80.0f, 80.0f};
    const Real sizes[14] = {40.0f, 40.0f, 10.0f, 20.0f, 1.0f, 0.5f, 3.7f, 7.3f, 12.5f, 33.333f, 100.0f, 0.0f, -5.0f, 0.999f};
    const Real lows[8] = {0.0f, -200.0f, -37.5f, 13.3f, -1000.5f, 500.0f, -0.25f, 100000.5f};
    Grid g;
    g.cs = pick(8) == 0 ? spread(0.6f, 90.0f) : sizes[pick(14)];
    const Real cs = effective_cs(g);
    g.lox = lows[pick(8)];
    g.loy = lows[pick(8)];
    const bool big = pick(10) == 0;
    int nx = big ? 30 + static_cast<int>(pick(35)) : 1 + static_cast<int>(pick(20));
    int ny = big ? 30 + static_cast<int>(pick(35)) : 1 + static_cast<int>(pick(20));
    Real w = nx * cs, h = ny * cs;
    if (pick(2)) w -= spread(0.0f, 0.99f) * cs;
    if (pick(2)) h -= spread(0.0f, 0.99f) * cs;
    if (pick(12) == 0) w = pick(2) ? 0.3f : -50.0f;
    if (pick(12) == 0) h = pick(2) ? 0.0f : 0.999f;
    g.hix = g.lox + w;
    g.hiy = g.loy + h;
    return g;
}

static void set_grid(oracle_wp51::OracleTerrainLogic* terrain, const Grid& g) {
    TheWritableGlobalData->m_partitionCellSize = g.cs;
    terrain->extent.lo = {g.lox, g.loy, 0.0f};
    terrain->extent.hi = {g.hix, g.hiy, 100.0f};
    ThePartitionManager->reset();
    ThePartitionManager->init();
}

static Real make_radius(Real cs) {
    const Real f[14] = {0.0f, 0.1f, 0.25f, 0.5f, 0.4999f, 0.500001f, 1.0f, 1.5f, 2.0f, 2.5f, 3.0f, 5.0f, 8.0f, 1.2f};
    switch (pick(10)) {
    case 0: return spread(0.0f, 3.0f * cs);
    case 1: return spread(0.0f, 12.0f * cs);
    case 2: return pick(4) ? spread(0.0f, 50.0f) : (pick(2) ? 3.0f : 17.0f);
    case 3: return pick(5) ? spread(0.0f, 0.5f * cs) : spread(15.0f * cs, pick(6) ? 40.0f * cs : 130.0f * cs);
    default: return f[pick(14)] * cs;
    }
}

static Geom make_geom(Real cs) {
    Geom g;
    g.type = static_cast<int>(pick(3));
    g.small = pick(10) < 3;
    g.height = spread(0.0f, 30.0f);
    g.major = make_radius(cs);
    g.minor = make_radius(cs);
    // `make_radius` may return an absolute value; keep the span in cells bounded.
    if (g.major > 130.0f * cs) g.major = 130.0f * cs;
    if (g.minor > 130.0f * cs) g.minor = 130.0f * cs;
    if (g.type == GEOMETRY_BOX && (g.major > 40.0f * cs || g.minor > 40.0f * cs) && pick(3)) {
        g.major *= 0.25f;
        g.minor *= 0.25f;
    }
    return g;
}

static Real make_coord(Real lo, Real hi, Real cs, int cells, Real radius) {
    const Real edge = lo + static_cast<Real>(static_cast<int>(pick(cells + 3)) - 1) * cs;
    switch (pick(10)) {
    case 0: return edge;
    case 1: return std::nextafterf(edge, pick(2) ? 1e9f : -1e9f);
    case 2: return edge + 0.5f * cs;
    case 3: return pick(2) ? edge + radius : edge - radius;
    case 4: return pick(2) ? lo - spread(0.0f, 6.0f * cs) : hi + spread(0.0f, 6.0f * cs);
    case 5: return pick(8) ? edge + (pick(2) ? 0.5f : -0.5f) * cs : (pick(2) ? lo - 100000.0f : hi + 100000.0f);
    default: return spread(lo, lo + cells * cs);
    }
}

static Coord3D make_pos(const Geom& g) {
    const Real cs = ThePartitionManager->getCellSize();
    const int nx = ThePartitionManager->getCellCountX();
    const int ny = ThePartitionManager->getCellCountY();
    const Region3D& e = static_cast<oracle_wp51::OracleTerrainLogic*>(TheTerrainLogic)->extent;
    Coord3D p;
    p.x = make_coord(e.lo.x, e.hi.x, cs, nx, g.major);
    p.y = make_coord(e.lo.y, e.hi.y, cs, ny, pick(2) ? g.major : g.minor);
    p.z = spread(-5.0f, 50.0f);
    return p;
}

static Real make_angle() {
    const Real a[8] = {0.0f, 1.5707964f, -1.5707964f, 3.1415927f, 0.7853982f, -0.7853982f, 2.3561945f, -3.1415927f};
    return pick(2) ? a[pick(8)] : spread(-10.0f, 10.0f);
}

static void apply_geom(Object* o, const Geom& g) {
    GeometryInfo info(static_cast<GeometryType>(g.type), g.small, g.height, g.major, g.minor);
    o->setGeometryInfo(info);
}

static std::string geom_json(const Object* o, const Geom& g) {
    std::ostringstream ss;
    ss << g.type << ',' << (g.small ? 1 : 0) << ',' << bits(g.height) << ',' << bits(g.major) << ','
       << bits(g.minor) << ',' << bits(o->getGeometryInfo().getMajorRadius()) << ','
       << bits(o->getGeometryInfo().getMinorRadius());
    return ss.str();
}

static std::string place_json(const Object* o) {
    const Coord3D* p = o->getPosition();
    std::ostringstream ss;
    ss << bits(p->x) << ',' << bits(p->y) << ',' << bits(p->z) << ',' << bits(o->getOrientation());
    return ss.str();
}

static void clear_objects() {
    for (int i = 0; i < kObjects; ++i) ThePartitionManager->unRegisterObject(g_obj[i]);
}

static void dirty_if_registered(Object* o) {
    if (o->friend_getPartitionData()) o->friend_getPartitionData()->makeDirty(true);
}

// False if the script must be discarded.
static bool run_vector(oracle_wp51::OracleTerrainLogic* terrain, const Grid& grid, int grid_index,
                       bool readable, std::string& out) {
    set_grid(terrain, grid);
    const Real cs = ThePartitionManager->getCellSize();
    const int n = 1 + static_cast<int>(pick(kObjects)) ;
    Geom geoms[kObjects];
    std::ostringstream ss;
    ss << "{\"g\":" << grid_index << ",\"o\":[";
    for (int i = 0; i < n; ++i) {
        Object* o = g_obj[i];
        geoms[i] = make_geom(cs);
        apply_geom(o, geoms[i]);
        const Coord3D p = make_pos(geoms[i]);
        o->setPosition(&p);
        o->setOrientation(make_angle());
        ss << (i ? "," : "") << "[" << geom_json(o, geoms[i]) << ',' << place_json(o) << "]";
    }
    ss << "],\"s\":[";
    bool first = true;
    bool ok = true;
    auto row = [&](const std::string& op) {
        ss << (first ? "" : ",") << "[" << op << ",\"" << dirty_list() << "\",\"" << module_list() << "\"]";
        first = false;
    };
    auto update = [&]() {
        if (small_fill_overflows()) { ok = false; return; }
        const std::string order = dirty_list();
        const PartitionCell* before[kObjects] = {};
        for (int i = 0; i < kObjects; ++i)
            if (PartitionData* d = g_obj[i]->friend_getPartitionData()) before[i] = d->*access_member(PgLastCellTag());
        ThePartitionManager->update();
        std::string changed;
        for (char c : order) {
            const int i = c - '1';
            if (g_obj[i]->friend_getPartitionData()->*access_member(PgLastCellTag()) != before[i]) changed += c;
        }
        for (int i = 0; i < n; ++i) {
            PartitionData* d = g_obj[i]->friend_getPartitionData();
            if (d && d->friend_getCoiInUseCount() == 0) ++g_off_map;
        }
        row("7," + pack(cell_dump(), readable) + "," + pack(object_dump(), readable) + ",\"" + changed + "\"");
    };
    auto op_register = [&](int i) {
        ThePartitionManager->registerObject(g_obj[i]);
        row("0," + std::to_string(i + 1));
    };

    for (int i = 0; i < n; ++i)
        if (pick(8)) op_register(i);
    update();
    const int ops = 5 + static_cast<int>(pick(10));
    for (int k = 0; k < ops && ok; ++k) {
        const int i = static_cast<int>(pick(n));
        Object* o = g_obj[i];
        const std::string id = std::to_string(i + 1);
        switch (pick(20)) {
        case 0: op_register(i); break;
        case 1:
            ThePartitionManager->unRegisterObject(o);
            row("1," + id);
            break;
        case 2: case 3: case 4: case 5: case 6: {
            Coord3D p = make_pos(geoms[i]);
            // Sometimes share another object's place, to stack cell lists.
            if (pick(5) == 0) p = *g_obj[pick(n)]->getPosition();
            o->setPosition(&p);
            dirty_if_registered(o);
            row("2," + id + "," + place_json(o));
            break;
        }
        case 7: case 8:
            o->setOrientation(make_angle());
            dirty_if_registered(o);
            row("3," + id + "," + place_json(o));
            break;
        case 9: case 10:
            geoms[i] = make_geom(cs);
            apply_geom(o, geoms[i]);
            row("4," + id + "," + geom_json(o, geoms[i]));
            break;
        case 11: case 12:
            dirty_if_registered(o);
            row("5," + id);
            break;
        case 13:
            if (o->friend_getPartitionData()) o->friend_getPartitionData()->makeDirty(false);
            row("6," + id);
            break;
        default: update(); break;
        }
    }
    if (ok) update();
    clear_objects();
    if (ThePartitionManager->*access_member(PgModuleListTag())) die("modules left");
    ss << "]}";
    out = ss.str();
    return ok;
}

static std::string grid_json(const Grid& g, bool readable) {
    std::ostringstream ss;
    const int max_radius = ThePartitionManager->*access_member(PgMaxRadiusTag());
    const auto& rings = ThePartitionManager->*access_member(PgRadiusVecTag());
    ss << "[" << bits(g.cs) << ',' << bits(g.lox) << ',' << bits(g.loy) << ',' << bits(g.hix) << ','
       << bits(g.hiy) << ',' << bits(ThePartitionManager->getCellSize()) << ','
       << ThePartitionManager->getCellCountX() << ',' << ThePartitionManager->getCellCountY() << ','
       << max_radius << ",[";
    for (int r = 0; r <= max_radius; ++r) {
        std::string s;
        for (const ICoord2D& c : rings[r]) s += std::to_string(c.x) + "." + std::to_string(c.y) + " ";
        ss << (r ? "," : "") << "\"" << (readable ? s : fnv(s)) << "\"";
    }
    ss << "]]";
    return ss.str();
}

int main(int argc, char** argv) {
    if (argc < 2) { std::fprintf(stderr, "usage: gen_partition_grid <out.json>\n"); return 2; }
    const std::string out_path = argv[1];
    oracle_wp51::World w;
    w.load_ini(
        "PlayerTemplate FactionPG\n  Side = America\n  PlayableSide = Yes\nEnd\n"
        "Object PG_Object\n  KindOf = PRELOAD\n  Geometry = SPHERE\n  GeometryMajorRadius = 1.0\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100.0\n    InitialHealth = 100.0\n  End\nEnd\n");
    w.init_with_neutral_player("FactionPG", nullptr);
    auto* terrain = static_cast<oracle_wp51::OracleTerrainLogic*>(TheTerrainLogic);
    for (int i = 0; i < kObjects; ++i) {
        g_obj[i] = w.spawn_object("PG_Object", w.team_a, {100.0f, 100.0f, 0.0f});
        if (!g_obj[i]) die("spawn");
        g_obj[i]->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_NO_COLLISIONS));
    }
    clear_objects();
    if (ThePartitionManager->*access_member(PgModuleListTag())) die("other modules registered");

    std::vector<Grid> grids;
    std::vector<std::string> grid_rows;
    for (int i = 0; i < kGrids; ++i) {
        grids.push_back(make_grid(i));
        set_grid(terrain, grids.back());
        grid_rows.push_back(grid_json(grids.back(), i == 0));
    }
    std::vector<std::string> vectors;
    for (int i = 0; i < kVectors; ++i) {
        std::string row;
        const int g = i < kReadable ? i : static_cast<int>(pick(kGrids));
        while (!run_vector(terrain, grids[g], g, i < kReadable, row)) ++g_small_dropped;
        vectors.push_back(row);
    }

    const std::string tmp = out_path + ".tmp";
    FILE* f = std::fopen(tmp.c_str(), "wb");
    if (!f) { std::perror("open"); return 1; }
    const std::string sha = oracle::sha256_file(
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/PartitionManager.cpp");
    std::fprintf(f, "{\n\"packet\": \"PARTITION_GRID\",\n\"vector_count\": %d,\n", kVectors);
    std::fprintf(f, "\"sha256_PartitionManager\": \"%s\",\n", sha.c_str());
    std::fprintf(f, "\"small_overflow_dropped\": %u,\n\"grids\": [\n", g_small_dropped);
    for (size_t i = 0; i < grid_rows.size(); ++i)
        std::fprintf(f, "%s%s\n", grid_rows[i].c_str(), i + 1 < grid_rows.size() ? "," : "");
    std::fprintf(f, "],\n\"vectors\": [\n");
    for (size_t i = 0; i < vectors.size(); ++i)
        std::fprintf(f, "%s%s\n", vectors[i].c_str(), i + 1 < vectors.size() ? "," : "");
    std::fprintf(f, "]\n}\n");
    std::fclose(f);
    if (std::rename(tmp.c_str(), out_path.c_str())) { std::perror("rename"); return 1; }
    std::fprintf(stderr, "coverage: dropped=%u small_wide=%u full=%u off_map=%u\n",
                 g_small_dropped, g_small_wide, g_full, g_off_map);
    return 0;
}
