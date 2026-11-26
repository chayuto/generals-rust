// Partition searches: drives the real PartitionManager with real Objects.
//
// Canonical 3-mode build & run:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_partition_query.cpp --opt O0 -o <dir>/gen_pq_o0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_partition_query.cpp --opt O2 -o <dir>/gen_pq_o2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_partition_query.cpp --opt O2 --sanitize -o <dir>/gen_pq_ubsan
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy <dir>/gen_pq_o2 docs/golden_vectors/PARTITION_QUERY.json
//
// Each world is a grid, up to 24 objects and a script of filing operations and
// searches: [grid, objects, script].
//
//   object  type,small,height,major,minor,circle,sphere,zdelta,x,y,z,angle,cost
//   op      [0,i] register  [1,i] unregister  [2,i,x,y,z,angle] move and dirty
//           [4,i,<first 8 object fields>] geometry  [7] update
//   search  [8,kind,center,x,y,z,rule,radius,order,reject mask,"offered","ids",values]
//           kind 0 closest (values: dist,vx,vy,vz), 1 range (squared distances),
//           2 all objects. center -1 searches from x,y,z. Object i is letter a+i.
//
// Searches run before the first update, after it, with objects moved,
// registered and removed since, and after a further update. Every search records
// the objects offered to the filter, in order, and its result.
//
// Half the worlds put objects on a coarse lattice, which gives equal distances
// and distances equal to the radius. Other radii are taken from a probe search
// so that they sit on or next to an object.
//
// Objects carry OBJECT_STATUS_NO_COLLISIONS, as in gen_partition_grid.cpp.
//
// Not covered: ghost objects, immobile objects, NaN or infinite positions or
// radii, a filter that changes the world or starts another search, positions
// beyond about 100000 cells off the map, build costs whose difference overflows.

#include "oracle_wp51_world.h"

#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectIter.h"
#include "GameLogic/PartitionManager.h"

#include <cmath>
#include <cstdio>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

struct PqModuleListTag { using type = PartitionData* PartitionManager::*; };
template struct MemberAccess<PqModuleListTag, &PartitionManager::m_moduleList>;
inline PartitionData* PartitionManager::* access_member(PqModuleListTag);
struct PqCoiCapTag { using type = Int PartitionData::*; };
template struct MemberAccess<PqCoiCapTag, &PartitionData::m_coiArrayCount>;
inline Int PartitionData::* access_member(PqCoiCapTag);

static const int kObjects = 24;
static const int kWorlds = 130;

static uint32_t bits(Real v) { uint32_t b; std::memcpy(&b, &v, 4); return b; }

static uint32_t g_state = 0x51C7A9E3u;
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
static Geom g_geom[kObjects];

// Coverage counters.
static uint32_t c_queries = 0, c_closest_hit = 0, c_closest_miss = 0, c_range_nonempty = 0, c_ties = 0,
                c_at_radius = 0, c_dropped = 0, c_huge = 0, c_not_nearest = 0, c_sorted_multi = 0;

static void die(const char* what) { std::fprintf(stderr, "harness: %s\n", what); std::exit(1); }

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
        if (n > d->*access_member(PqCoiCapTag())) return true;
            }
    return false;
}

static Real effective_cs(const Grid& g) { return g.cs < 1.0f ? 1.0f : g.cs; }

static Grid make_grid(int i) {
    (void)i;
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

static Real make_angle() {
    const Real a[8] = {0.0f, 1.5707964f, -1.5707964f, 3.1415927f, 0.7853982f, -0.7853982f, 2.3561945f, -3.1415927f};
    return pick(2) ? a[pick(8)] : spread(-10.0f, 10.0f);
}

static void apply_geom(Object* o, const Geom& g) {
    GeometryInfo info(static_cast<GeometryType>(g.type), g.small, g.height, g.major, g.minor);
    o->setGeometryInfo(info);
}

struct World_ {
    bool lattice = false;
    Real step = 1.0f;
    int span = 1;
};

static Real lattice_coord(const World_& w, Real lo) {
    return lo + static_cast<Real>(static_cast<int>(pick(w.span + 1))) * w.step;
}

static Geom make_query_geom(const World_& w, Real cs) {
    Geom g = make_geom(cs);
    if (w.lattice) {
        const Real r[6] = {0.0f, 0.0f, 0.5f, 1.0f, 1.0f, 2.0f};
        g.major = r[pick(6)] * w.step;
        g.minor = r[pick(6)] * w.step;
        g.height = r[pick(6)] * w.step * 2.0f;
    }
    // Keep the filing cost bounded.
    const Real top = (pick(12) == 0 ? 20.0f : 4.0f) * cs;
    if (g.major > top) g.major = top;
    if (g.minor > top) g.minor = top;
    return g;
}

static Coord3D make_point(const World_& w) {
    const Real cs = ThePartitionManager->getCellSize();
    const int nx = ThePartitionManager->getCellCountX();
    const int ny = ThePartitionManager->getCellCountY();
    const Region3D& e = static_cast<oracle_wp51::OracleTerrainLogic*>(TheTerrainLogic)->extent;
    Coord3D p;
    if (w.lattice) {
        p.x = lattice_coord(w, e.lo.x);
        p.y = lattice_coord(w, e.lo.y);
        p.z = pick(3) ? 0.0f : static_cast<Real>(pick(5)) * w.step;
        if (pick(20) == 0) p.x = e.lo.x - w.step * static_cast<Real>(1 + pick(6));
    } else {
        p.x = make_coord(e.lo.x, e.hi.x, cs, nx, cs);
        p.y = make_coord(e.lo.y, e.hi.y, cs, ny, cs);
        p.z = spread(-5.0f, 50.0f);
    }
    return p;
}

static int index_of(const Object* o) {
    for (int i = 0; i < kObjects; ++i)
        if (g_obj[i] == o) return i;
    die("unknown object");
    return -1;
}
static char letter(const Object* o) { return static_cast<char>('a' + index_of(o)); }

class Recorder : public PartitionFilter {
public:
    uint32_t reject = 0;
    std::string calls;
    virtual Bool allow(Object* other) override {
        calls += letter(other);
        return (reject >> index_of(other)) & 1u ? FALSE : TRUE;
    }
};

struct Query {
    int kind;    // 0 closest, 1 range, 2 all
    int center;  // object index, or -1 for a position
    Coord3D pos{0.0f, 0.0f, 0.0f};
    int dc = 0;
    Real dist = 0.0f;
    int order = 0;
    uint32_t reject = 0;
};

static std::vector<std::pair<Object*, Real>> drain(SimpleObjectIterator* iter) {
    std::vector<std::pair<Object*, Real>> out;
    Real num = 0.0f;
    for (Object* o = iter->firstWithNumeric(&num); o; o = iter->nextWithNumeric(&num)) out.push_back({o, num});
    if (static_cast<Int>(out.size()) != iter->getCount()) die("count");
    deleteInstance(iter);
    return out;
}

static std::vector<std::pair<Object*, Real>> range(const Query& q, Real dist, PartitionFilter** filters, int order) {
    const DistanceCalculationType dc = static_cast<DistanceCalculationType>(q.dc);
    const IterOrderType o = static_cast<IterOrderType>(order);
    if (q.center >= 0)
        return drain(ThePartitionManager->iterateObjectsInRange(
            static_cast<const Object*>(g_obj[q.center]), dist, dc, filters, o));
    return drain(ThePartitionManager->iterateObjectsInRange(&q.pos, dist, dc, filters, o));
}

static std::string run_query(const Query& q) {
    Recorder rec;
    rec.reject = q.reject;
    PartitionFilter* filters[2] = {&rec, nullptr};
    std::ostringstream ss;
    ss << "[8," << q.kind << ',' << q.center << ',' << bits(q.pos.x) << ',' << bits(q.pos.y) << ',' << bits(q.pos.z)
       << ',' << q.dc << ',' << bits(q.dist) << ',' << q.order << ',' << q.reject << ',';
    ++c_queries;
    if (q.kind != 2 && q.dist >= HUGE_DIST) ++c_huge;
    if (q.kind == 0) {
        Real dist = -123.0f;
        Coord3D vec{-1.0f, -2.0f, -3.0f};
        const DistanceCalculationType dc = static_cast<DistanceCalculationType>(q.dc);
        Object* found = q.center >= 0
            ? ThePartitionManager->getClosestObject(static_cast<const Object*>(g_obj[q.center]), q.dist, dc, filters, &dist, &vec)
            : ThePartitionManager->getClosestObject(&q.pos, q.dist, dc, filters, &dist, &vec);
        const std::string calls = rec.calls;
        if (found) {
            ++c_closest_hit;
            // Count results which the ring walk made other than the nearest.
            Recorder probe_rec;
            probe_rec.reject = q.reject;
            PartitionFilter* probe[2] = {&probe_rec, nullptr};
            const auto all = range(q, q.dist, probe, ITER_SORTED_NEAR_TO_FAR);
            if (!all.empty() && std::sqrt(all.front().second) < dist) ++c_not_nearest;
        } else {
            ++c_closest_miss;
        }
        ss << "\"" << calls << "\",\"" << (found ? std::string(1, letter(found)) : std::string()) << "\",["
           << bits(dist) << ',' << bits(vec.x) << ',' << bits(vec.y) << ',' << bits(vec.z) << "]]";
        return ss.str();
    }
    std::vector<std::pair<Object*, Real>> out;
    if (q.kind == 1) {
        out = range(q, q.dist, filters, q.order);
        if (!out.empty()) ++c_range_nonempty;
        if (out.size() > 2 && q.order) ++c_sorted_multi;
        for (size_t i = 0; i < out.size(); ++i)
            for (size_t j = i + 1; j < out.size(); ++j)
                if (out[i].second == out[j].second) { ++c_ties; i = out.size() - 1; break; }
    } else {
        out = drain(ThePartitionManager->iterateAllObjects(filters));
    }
    std::string ids;
    for (const auto& e : out) ids += letter(e.first);
    ss << "\"" << rec.calls << "\",\"" << ids << "\",[";
    for (size_t i = 0; i < out.size(); ++i) ss << (i ? "," : "") << bits(out[i].second);
    ss << "]]";
    return ss.str();
}

static Query make_query(const World_& w, int n) {
    Query q;
    const Real cs = ThePartitionManager->getCellSize();
    const uint32_t k = pick(20);
    q.kind = k < 7 ? 0 : (k < 19 ? 1 : 2);
    q.center = -1;
    q.dc = static_cast<int>(pick(4));
    q.order = static_cast<int>(pick(5));
    const uint32_t f = pick(6);
    if (f == 0) q.reject = next_u32() & 0xFFFFFFu;
    else if (f == 1) q.reject = next_u32() & next_u32() & 0xFFFFFFu;
    else if (f == 2 && pick(4) == 0) q.reject = 0xFFFFFFu;
    if (q.kind == 2) { q.dc = 0; q.order = 0; return q; }
    if (pick(2)) {
        q.center = static_cast<int>(pick(n));
    } else {
        const Region3D& e = static_cast<oracle_wp51::OracleTerrainLogic*>(TheTerrainLogic)->extent;
        switch (pick(8)) {
        case 0: case 1: q.pos = *g_obj[pick(n)]->getPosition(); break;
        case 2:
            q.pos = make_point(w);
            q.pos.x = pick(2) ? e.lo.x - spread(0.0f, 8.0f * cs) : e.hi.x + spread(0.0f, 8.0f * cs);
            break;
        case 3:
            q.pos = make_point(w);
            if (pick(3) == 0) q.pos.y = pick(2) ? e.lo.y - 90000.0f * cs : e.hi.y + 90000.0f * cs;
            break;
        default: q.pos = make_point(w); break;
        }
    }
    const Real diag = (ThePartitionManager->getCellCountX() + ThePartitionManager->getCellCountY()) * cs;
    switch (pick(16)) {
    case 0: q.dist = 0.0f; break;
    case 1: q.dist = pick(2) ? -spread(0.0f, 3.0f * cs) : -0.25f * cs; break;
    case 2: q.dist = pick(2) ? HUGE_DIST : std::nextafterf(HUGE_DIST, 0.0f); break;
    case 3: q.dist = pick(2) ? 1e9f : 1e20f; break;
    case 4: q.dist = spread(0.0f, diag); break;
    case 5: q.dist = spread(0.0f, 4.0f * cs); break;
    case 6: case 7: case 8:
        q.dist = w.lattice ? w.step * static_cast<Real>(pick(9)) : cs * static_cast<Real>(pick(6));
        break;
    case 9: case 10: case 11: case 12: {
        // On, or next to, the distance of some object.
        const auto all = range(q, 1e9f, nullptr, ITER_FASTEST);
        if (all.empty()) { q.dist = 2.0f * cs; break; }
        q.dist = WWMath::Sqrtf(all[pick(static_cast<uint32_t>(all.size()))].second);
        if (pick(3) == 0) q.dist = std::nextafterf(q.dist, pick(2) ? 1e9f : -1e9f);
        break;
    }
    default: q.dist = spread(0.0f, diag * 0.5f) + cs; break;
    }
    if (q.dist >= 0.0f && q.dist < HUGE_DIST) {
        for (const auto& e : range(q, q.dist * 1.001f + 0.001f, nullptr, ITER_FASTEST))
            if (e.second == q.dist * q.dist) { ++c_at_radius; break; }
    }
    return q;
}

static void clear_objects() {
    for (int i = 0; i < kObjects; ++i) ThePartitionManager->unRegisterObject(g_obj[i]);
}

static std::string geom_fields(int i) {
    const Geom& g = g_geom[i];
    const GeometryInfo& info = g_obj[i]->getGeometryInfo();
    std::ostringstream ss;
    ss << g.type << ',' << (g.small ? 1 : 0) << ',' << bits(g.height) << ',' << bits(g.major) << ',' << bits(g.minor)
       << ',' << bits(info.getBoundingCircleRadius()) << ',' << bits(info.getBoundingSphereRadius()) << ','
       << bits(info.getZDeltaToCenterPosition());
    return ss.str();
}

static std::string place_fields(int i) {
    const Coord3D* p = g_obj[i]->getPosition();
    std::ostringstream ss;
    ss << bits(p->x) << ',' << bits(p->y) << ',' << bits(p->z) << ',' << bits(g_obj[i]->getOrientation());
    return ss.str();
}

static void move_object(const World_& w, int i, int n) {
    Coord3D p = make_point(w);
    if (pick(6) == 0) p = *g_obj[pick(n)]->getPosition();
    g_obj[i]->setPosition(&p);
    g_obj[i]->setOrientation(make_angle());
    if (g_obj[i]->friend_getPartitionData()) g_obj[i]->friend_getPartitionData()->makeDirty(true);
}

// False if the world must be discarded.
static bool run_world(oracle_wp51::OracleTerrainLogic* terrain, std::string& out) {
    World_ w;
    w.lattice = pick(2) == 0;
    Grid grid = make_grid(0);
    if (w.lattice) {
        const Real sizes[3] = {10.0f, 20.0f, 40.0f};
        const Real lows[3] = {0.0f, -200.0f, 500.0f};
        const Real cs = sizes[pick(3)];
        grid.cs = cs;
        grid.lox = lows[pick(3)];
        grid.loy = lows[pick(3)];
        grid.hix = grid.lox + cs * static_cast<Real>(2 + pick(12));
        grid.hiy = grid.loy + cs * static_cast<Real>(2 + pick(12));
        const Real steps[4] = {0.25f, 0.5f, 1.0f, 2.0f};
        w.step = steps[pick(4)] * cs;
        w.span = pick(2) ? 3 : 7;
    }
    set_grid(terrain, grid);
    const Real cs = ThePartitionManager->getCellSize();
    const int n = pick(10) == 0 ? 1 + static_cast<int>(pick(3)) : 4 + static_cast<int>(pick(kObjects - 3));

    std::ostringstream ss;
    ss << "[[" << bits(grid.cs) << ',' << bits(grid.lox) << ',' << bits(grid.loy) << ',' << bits(grid.hix)
       << ',' << bits(grid.hiy) << "],[";
    for (int i = 0; i < n; ++i) {
        g_geom[i] = make_query_geom(w, cs);
        apply_geom(g_obj[i], g_geom[i]);
        move_object(w, i, i + 1);
        ss << (i ? "," : "") << "[" << geom_fields(i) << ',' << place_fields(i) << ','
           << g_obj[i]->getTemplate()->friend_getBuildCost() << "]";
    }
    ss << "],[";
    bool first = true;
    bool ok = true;
    auto row = [&](const std::string& s) {
        ss << (first ? "\n" : ",\n") << s;
        first = false;
    };
    auto op_register = [&](int i) {
        ThePartitionManager->registerObject(g_obj[i]);
        row("[0," + std::to_string(i) + "]");
    };
    auto op_unregister = [&](int i) {
        ThePartitionManager->unRegisterObject(g_obj[i]);
        row("[1," + std::to_string(i) + "]");
    };
    auto update = [&]() {
        if (small_fill_overflows()) { ok = false; return; }
        ThePartitionManager->update();
        row("[7]");
    };
    auto queries = [&](int count) {
        for (int k = 0; k < count && ok; ++k) row(run_query(make_query(w, n)));
    };

    for (int i = 0; i < n; ++i)
        if (pick(6)) op_register(i);
    if (pick(5) == 0) queries(3);  // nothing is filed yet
    update();
    queries(7);
    for (int round = 0; round < 2 && ok; ++round) {
        const int changes = 2 + static_cast<int>(pick(n));
        for (int k = 0; k < changes; ++k) {
            const int i = static_cast<int>(pick(n));
            switch (pick(8)) {
            case 0: op_register(i); break;
            case 1: if (pick(2)) op_unregister(i); else op_register(i); break;
            case 2: {
                // `setGeometryInfo` re-registers; here that is spelled out.
                const bool registered = g_obj[i]->friend_getPartitionData() != nullptr;
                if (registered) op_unregister(i);
                g_geom[i] = make_query_geom(w, cs);
                apply_geom(g_obj[i], g_geom[i]);
                row("[4," + std::to_string(i) + "," + geom_fields(i) + "]");
                if (registered) op_register(i);
                break;
            }
            default:
                move_object(w, i, n);
                row("[2," + std::to_string(i) + "," + place_fields(i) + "]");
                break;
            }
        }
        queries(5);  // stale cells, unfiled registrations
        update();
        queries(5);
    }
    clear_objects();
    if (ThePartitionManager->*access_member(PqModuleListTag())) die("modules left");
    ss << "]]";
    out = ss.str();
    return ok;
}

int main(int argc, char** argv) {
    if (argc < 2) { std::fprintf(stderr, "usage: gen_partition_query <out.json>\n"); return 2; }
    const std::string out_path = argv[1];
    oracle_wp51::World w;
    std::string ini =
        "PlayerTemplate FactionPQ\n  Side = America\n  PlayableSide = Yes\nEnd\n";
    const char* costs[5] = {"700", "100", "0", "100", "2500"};
    for (int t = 0; t < 5; ++t)
        ini += std::string("Object PQ_Object") + std::to_string(t) +
               "\n  KindOf = PRELOAD\n  BuildCost = " + costs[t] +
               "\n  Geometry = SPHERE\n  GeometryMajorRadius = 1.0\n"
               "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100.0\n    InitialHealth = 100.0\n  End\nEnd\n";
    w.load_ini(ini.c_str());
    w.init_with_neutral_player("FactionPQ", nullptr);
    auto* terrain = static_cast<oracle_wp51::OracleTerrainLogic*>(TheTerrainLogic);
    for (int i = 0; i < kObjects; ++i) {
        const std::string name = "PQ_Object" + std::to_string((i * 7) % 5);
        g_obj[i] = w.spawn_object(name.c_str(), w.team_a, {100.0f, 100.0f, 0.0f});
        if (!g_obj[i]) die("spawn");
        g_obj[i]->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_NO_COLLISIONS));
    }
    clear_objects();
    if (ThePartitionManager->*access_member(PqModuleListTag())) die("other modules registered");

    std::vector<std::string> worlds;
    for (int i = 0; i < kWorlds; ++i) {
        std::string row;
        for (;;) {
            const uint32_t before[10] = {c_queries, c_closest_hit, c_closest_miss, c_range_nonempty, c_ties,
                                         c_at_radius, c_huge, c_not_nearest, c_sorted_multi, 0};
            if (run_world(terrain, row)) break;
            ++c_dropped;
            clear_objects();
            c_queries = before[0]; c_closest_hit = before[1]; c_closest_miss = before[2];
            c_range_nonempty = before[3]; c_ties = before[4]; c_at_radius = before[5];
            c_huge = before[6]; c_not_nearest = before[7]; c_sorted_multi = before[8];
        }
        worlds.push_back(row);
    }

    const std::string tmp = out_path + ".tmp";
    FILE* f = std::fopen(tmp.c_str(), "wb");
    if (!f) { std::perror("open"); return 1; }
    const std::string sha = oracle::sha256_file(
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/PartitionManager.cpp");
    std::fprintf(f, "{\n\"packet\": \"PARTITION_QUERY\",\n\"vector_count\": %u,\n", c_queries);
    std::fprintf(f, "\"sha256_PartitionManager\": \"%s\",\n", sha.c_str());
    std::fprintf(f, "\"small_overflow_dropped\": %u,\n\"worlds\": [\n", c_dropped);
    for (size_t i = 0; i < worlds.size(); ++i)
        std::fprintf(f, "%s%s\n", worlds[i].c_str(), i + 1 < worlds.size() ? "," : "");
    std::fprintf(f, "]\n}\n");
    std::fclose(f);
    if (std::rename(tmp.c_str(), out_path.c_str())) { std::perror("rename"); return 1; }
    std::fprintf(stderr,
        "coverage: queries=%u closest_hit=%u closest_miss=%u not_nearest=%u range_nonempty=%u sorted_multi=%u "
        "ties=%u at_radius=%u huge=%u dropped=%u\n",
        c_queries, c_closest_hit, c_closest_miss, c_not_nearest, c_range_nonempty, c_sorted_multi, c_ties,
        c_at_radius, c_huge, c_dropped);
    return 0;
}
