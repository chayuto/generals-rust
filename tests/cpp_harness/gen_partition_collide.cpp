// Partition collision detection: drives the real PartitionManager with real Objects.
//
// Canonical 3-mode build & run:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_partition_collide.cpp --opt O0 -o <dir>/gen_pc_o0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_partition_collide.cpp --opt O2 -o <dir>/gen_pc_o2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_partition_collide.cpp --opt O2 --sanitize -o <dir>/gen_pc_ubsan
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy <dir>/gen_pc_o2 docs/golden_vectors/PARTITION_COLLIDE.json
//
// "pairs": two shapes through PartitionData::friend_collidesWith and
// PartitionManager::geomCollidesWithGeom. A row is
//   [type,height,major,minor,x,y,z,angle (A), the same (B), collides, geomCollides, mask, 6 values]
// with reals as float bits and the values loc.xyz, normal.xyz.
//
// "vectors": scripts over up to eight objects. onCollide is received by a collide
// module appended to each real Object, so every event went through the real
// Object::onCollide, which skips an object that has OBJECT_STATUS_NO_COLLISIONS.
// Operations (every row ends with the dirty list and module list, head first):
//   0 register i; 1 unregister i; 2 move i x y z angle; 3 makeDirty(true) i;
//   4 makeDirty(false) i; 5 NO_COLLISIONS i value; 6 DESTROYED i value;
//   7 update: events [self, other, mask, 6 values]; 8 set reaction of i;
//   9 isColliding i j result.
//
// "mask": bit k set when the C++ wrote value k. Close non-sphere pairs leave
// loc.x, loc.y and the normal uninitialised; such values are recorded as zero.
// The mask comes from calling the real collidesWith twice over two different
// fill patterns.
//
// Not covered: NaN or infinite inputs, ghost objects, geometry changes between
// the dirty-list drain and processing, unregistration from onPartitionCellChange,
// collide modules of real units.

#include "oracle_wp51_world.h"

#include "Common/ThingFactory.h"
#include "GameLogic/Module/CollideModule.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"

#include <cmath>
#include <cstdio>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

struct PcModuleListTag { using type = PartitionData* PartitionManager::*; };
template struct MemberAccess<PcModuleListTag, &PartitionManager::m_moduleList>;
inline PartitionData* PartitionManager::* access_member(PcModuleListTag);
struct PcDirtyTag { using type = PartitionData* PartitionManager::*; };
template struct MemberAccess<PcDirtyTag, &PartitionManager::m_dirtyModules>;
inline PartitionData* PartitionManager::* access_member(PcDirtyTag);
struct PcNextDirtyTag { using type = PartitionData* PartitionData::*; };
template struct MemberAccess<PcNextDirtyTag, &PartitionData::m_nextDirty>;
inline PartitionData* PartitionData::* access_member(PcNextDirtyTag);
struct PcBehaviorsTag { using type = BehaviorModule** Object::*; };
template struct MemberAccess<PcBehaviorsTag, &Object::m_behaviors>;
inline BehaviorModule** Object::* access_member(PcBehaviorsTag);

static const int kSlots = 8;
static const int kVariants = 4;  // bit 0 IMMOBILE, bit 1 NO_COLLIDE
static const int kPairs = 2600;
static const int kVectors = 520;

static void die(const char* what) { std::fprintf(stderr, "harness: %s\n", what); std::exit(1); }
static uint32_t bits(Real v) { uint32_t b; std::memcpy(&b, &v, 4); return b; }

static uint32_t g_state = 0x51C0A7E5u;
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

struct Geom { int type; Real height, major, minor; };
struct Place { Real x, y, z, angle; };
struct Reaction { int kind; int target; Place to; };

static Object* g_pool[kVariants][kSlots];
static Object* g_obj[kSlots];
static Object* g_scratch;  // outside every script
static Reaction g_react[kSlots];

// ------------------------------------------------------------------ probing

struct Probe { bool hit; unsigned mask; uint32_t v[6]; };

static Probe probe(const PartitionData* a, const PartitionData* b) {
    CollideLocAndNormal c1, c2;
    std::memset(static_cast<void*>(&c1), 0x5A, sizeof(c1));
    std::memset(static_cast<void*>(&c2), 0xC3, sizeof(c2));
    const bool r1 = a->friend_collidesWith(b, &c1) != 0;
    const bool r2 = a->friend_collidesWith(b, &c2) != 0;
    if (r1 != r2) die("result depends on cinfo");
    const Real* p1[6] = {&c1.loc.x, &c1.loc.y, &c1.loc.z, &c1.normal.x, &c1.normal.y, &c1.normal.z};
    const Real* p2[6] = {&c2.loc.x, &c2.loc.y, &c2.loc.z, &c2.normal.x, &c2.normal.y, &c2.normal.z};
    Probe out = {r1, 0u, {0, 0, 0, 0, 0, 0}};
    for (int k = 0; r1 && k < 6; ++k)
        if (bits(*p1[k]) == bits(*p2[k])) {
            out.mask |= 1u << k;
            out.v[k] = bits(*p1[k]);
        }
    return out;
}

// ------------------------------------------------------------------ recorder

static int slot_of(const Object* o) {
    for (int i = 0; i < kSlots; ++i)
        if (g_obj[i] == o) return i;
    die("unknown object");
    return -1;
}

static std::string g_events;
static int g_first_self = -1, g_first_other = -1;
static unsigned g_first_mask = 0;
static uint32_t g_event_count = 0, g_unwritten_events = 0, g_residue_nonzero = 0;
static uint32_t g_second_calls = 0, g_reactions[16];

static void dirty_if_registered(Object* o) {
    if (o->friend_getPartitionData()) o->friend_getPartitionData()->makeDirty(true);
}

static void move_object(Object* o, const Place& to) {
    o->setOrientation(to.angle);
    const Coord3D p = {to.x, to.y, to.z};
    o->setPosition(&p);
    if (bits(o->getOrientation()) != bits(to.angle)) die("orientation does not round trip");
    if (bits(o->getPosition()->x) != bits(to.x) || bits(o->getPosition()->y) != bits(to.y) ||
        bits(o->getPosition()->z) != bits(to.z))
        die("position does not round trip");
    dirty_if_registered(o);
}

static void set_status(Object* o, ObjectStatusTypes status, bool on) {
    o->setStatus(MAKE_OBJECT_STATUS_MASK(status), on);
}

static void react(Object* self, Object* other, const Reaction& r) {
    Object* third = g_obj[r.target];
    ++g_reactions[r.kind];
    switch (r.kind) {
    case 0: break;
    case 1: set_status(self, OBJECT_STATUS_DESTROYED, true); break;
    case 2: set_status(other, OBJECT_STATUS_DESTROYED, true); break;
    case 3: set_status(self, OBJECT_STATUS_NO_COLLISIONS, true); break;
    case 4: set_status(other, OBJECT_STATUS_NO_COLLISIONS, true); break;
    case 5: ThePartitionManager->unRegisterObject(self); break;
    case 6: ThePartitionManager->unRegisterObject(other); break;
    case 7: move_object(self, r.to); break;
    case 8: move_object(other, r.to); break;
    case 9:
        ThePartitionManager->unRegisterObject(other);
        ThePartitionManager->registerObject(other);
        break;
    case 10: ThePartitionManager->unRegisterObject(third); break;
    case 11: move_object(third, r.to); break;
    case 12: dirty_if_registered(other); break;
    case 13: ThePartitionManager->registerObject(third); break;
    default: die("reaction");
    }
}

class RecCollide : public CollideModule {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(RecCollide, "OracleRecCollide", 64, 64)
    MAKE_STANDARD_MODULE_MACRO(RecCollide)
public:
    RecCollide(Thing* thing, const ModuleData* data) : CollideModule(thing, data) {}

    virtual void onCollide(Object* other, const Coord3D* loc, const Coord3D* normal) override {
        Object* self = getObject();
        const int s = slot_of(self);
        const int o = slot_of(other);
        const Real values[6] = {loc->x, loc->y, loc->z, normal->x, normal->y, normal->z};
        unsigned mask;
        if (g_first_self == o && g_first_other == s) {
            mask = g_first_mask;
            g_first_self = g_first_other = -1;
            ++g_second_calls;
        } else {
            const Probe p = probe(self->friend_getPartitionData(), other->friend_getPartitionData());
            if (!p.hit) die("event without collision");
            mask = p.mask;
            g_first_self = s;
            g_first_other = o;
            g_first_mask = mask;
            for (int k = 0; k < 6; ++k)
                if ((mask >> k & 1u) && p.v[k] != bits(values[k])) die("probe differs from event");
            if (mask != 63u) {
                ++g_unwritten_events;
                for (int k = 0; k < 6; ++k)
                    if (!(mask >> k & 1u) && bits(values[k]) != 0u) { ++g_residue_nonzero; break; }
            }
        }
        std::ostringstream ss;
        ss << (g_events.empty() ? "" : ",") << '[' << s << ',' << o << ',' << mask;
        for (int k = 0; k < 6; ++k) ss << ',' << ((mask >> k & 1u) ? bits(values[k]) : 0u);
        ss << ']';
        g_events += ss.str();
        ++g_event_count;
        react(self, other, g_react[s]);
    }
};
RecCollide::~RecCollide() {}
void RecCollide::crc(Xfer*) {}
void RecCollide::xfer(Xfer*) {}
void RecCollide::loadPostProcess() {}

class RecCollideData : public CollideModuleData {};

static void attach_recorder(Object* o) {
    static RecCollideData data;
    BehaviorModule** old = o->*access_member(PcBehaviorsTag());
    int n = 0;
    while (old[n]) {
        if (old[n]->getCollide()) die("template already collides");
        ++n;
    }
    BehaviorModule** list = new BehaviorModule*[n + 2];
    for (int i = 0; i < n; ++i) list[i] = old[i];
    list[n] = static_cast<BehaviorModule*>(RecCollide::friend_newModuleInstance(o, &data));
    list[n + 1] = nullptr;
    o->*access_member(PcBehaviorsTag()) = list;
}

// ------------------------------------------------------------------ shared

static void apply_geom(Object* o, const Geom& g) {
    GeometryInfo info(static_cast<GeometryType>(g.type), false, g.height, g.major, g.minor);
    o->setGeometryInfo(info);
}

static std::string geom_json(const Geom& g) {
    std::ostringstream ss;
    ss << g.type << ',' << bits(g.height) << ',' << bits(g.major) << ',' << bits(g.minor);
    return ss.str();
}
static std::string place_json(const Object* o) {
    const Coord3D* p = o->getPosition();
    std::ostringstream ss;
    ss << bits(p->x) << ',' << bits(p->y) << ',' << bits(p->z) << ',' << bits(o->getOrientation());
    return ss.str();
}

static Real make_angle() {
    const Real a[9] = {0.0f, -0.0f, 1.5707964f, -1.5707964f, 3.1415927f, 0.7853982f, -0.7853982f, 2.3561945f, -3.1415927f};
    return pick(2) ? a[pick(9)] : spread(-3.1f, 3.1f);
}

static Real make_size() {
    const Real f[8] = {0.5f, 1.0f, 2.0f, 3.0f, 4.0f, 5.0f, 7.5f, 10.0f};
    if (pick(25) == 0) return 0.0f;
    return pick(3) ? f[pick(8)] : spread(0.1f, 12.0f);
}

static Geom make_geom(int type) {
    Geom g;
    g.type = type;
    g.major = make_size();
    g.minor = make_size();
    g.height = make_size();
    return g;
}

static Real nudge(Real v) {
    switch (pick(4)) {
    case 0: return std::nextafterf(v, 1e9f);
    case 1: return std::nextafterf(v, -1e9f);
    default: return v;
    }
}

// ------------------------------------------------------------------ pairs

static uint32_t g_hits[9], g_misses[9], g_unwritten_pairs = 0, g_z_disagree = 0;

static Real above(const Geom& g) { return g.type == GEOMETRY_SPHERE ? g.major : g.height; }
static Real below(const Geom& g) { return g.type == GEOMETRY_SPHERE ? g.major : 0.0f; }

static std::string run_pair(int index) {
    const int combo = index % 9;
    const int category = (index / 9) % 8;
    Geom ga = make_geom(combo / 3);
    Geom gb = make_geom(combo % 3);
    Place a, b;
    const bool origin = pick(3) == 0;
    a.x = origin ? 0.0f : static_cast<Real>(static_cast<int>(pick(401)) - 200);
    a.y = origin ? 0.0f : static_cast<Real>(static_cast<int>(pick(401)) - 200);
    a.z = origin ? 0.0f : static_cast<Real>(static_cast<int>(pick(41)) - 10);
    if (pick(6) == 0) a.x = spread(-1000.0f, 1000.0f);
    if (pick(6) == 0) a.y = spread(-1000.0f, 1000.0f);
    if (pick(12) == 0) a.x = -0.0f;
    a.angle = make_angle();
    b = a;
    b.angle = make_angle();
    const Real reach = ga.major + gb.major + 1.0f;
    const bool along_x = pick(2) != 0;
    const Real sign = pick(2) ? 1.0f : -1.0f;
    auto offset = [&](Real d) {
        if (along_x) b.x = a.x + sign * d;
        else b.y = a.y + sign * d;
    };
    auto small_offset = [&]() {
        if (pick(3)) {
            b.x = a.x + spread(-0.7f, 0.7f) * reach;
            b.y = a.y + spread(-0.7f, 0.7f) * reach;
        }
    };
    switch (category) {
    case 0:  // anywhere near
    case 1:
        b.x = a.x + spread(-1.3f, 1.3f) * reach;
        b.y = a.y + spread(-1.3f, 1.3f) * reach;
        b.z = a.z + spread(-1.0f, 1.0f) * (ga.height + gb.height + 1.0f);
        break;
    case 2:  // coincident centres
        if (pick(2)) b.z = a.z + spread(-4.0f, 4.0f);
        break;
    case 3:  // touching along one axis
        if (pick(3)) { a.angle = 0.0f; b.angle = 0.0f; }
        offset(nudge(ga.major + gb.major));
        break;
    case 4: {  // preflight boundary: centre distance equals the smallest radius
        Real m = ga.major;
        for (Real r : {ga.minor, gb.major, gb.minor})
            if (m > r) m = r;
        if (pick(2)) offset(nudge(m));
        else offset(m * spread(0.0f, 1.2f));
        break;
    }
    case 5: {  // height boundaries of both tests
        small_offset();
        switch (pick(4)) {
        case 0: b.z = nudge(a.z + above(ga) + below(gb)); break;
        case 1: b.z = nudge(a.z - below(ga) - above(gb)); break;
        case 2: b.z = nudge(a.z + above(ga)); break;
        default: b.z = nudge(a.z - above(gb)); break;
        }
        break;
    }
    case 6: {  // one centre below the other's base or above its top
        small_offset();
        const Real f[5] = {0.0f, 0.5f, 0.999f, 1.0f, 1.001f};
        switch (pick(4)) {
        case 0: b.z = nudge(a.z + ga.major * f[pick(5)]); break;
        case 1: b.z = nudge(a.z - above(gb) - ga.major * f[pick(5)]); break;
        case 2: b.z = nudge(a.z - gb.major * f[pick(5)]); break;
        default: b.z = nudge(a.z + above(ga) + gb.major * f[pick(5)]); break;
        }
        break;
    }
    default: {  // corners of rotated rectangles
        const Real d = (ga.major + gb.major + ga.minor + gb.minor) * 0.5f * spread(0.6f, 1.4f);
        const Real t = spread(0.0f, 6.2831f);
        b.x = a.x + d * Cos(t);
        b.y = a.y + d * Sin(t);
        break;
    }
    }

    Object* oa = g_pool[0][0];
    Object* ob = g_pool[0][1];
    apply_geom(oa, ga);
    apply_geom(ob, gb);
    oa->setOrientation(a.angle);
    ob->setOrientation(b.angle);
    const Coord3D pa = {a.x, a.y, a.z}, pb = {b.x, b.y, b.z};
    oa->setPosition(&pa);
    ob->setPosition(&pb);

    const Probe p = probe(oa->friend_getPartitionData(), ob->friend_getPartitionData());
    const bool geom_hit = ThePartitionManager->geomCollidesWithGeom(
        oa->getPosition(), oa->getGeometryInfo(), oa->getOrientation(),
        ob->getPosition(), ob->getGeometryInfo(), ob->getOrientation()) != 0;
    if ((ThePartitionManager->isColliding(oa, ob) != 0) != p.hit) die("isColliding differs");
    ++(p.hit ? g_hits : g_misses)[combo];
    if (p.hit && p.mask != 63u) ++g_unwritten_pairs;
    if (p.hit != geom_hit) ++g_z_disagree;

    std::ostringstream ss;
    ss << '[' << geom_json(ga) << ',' << place_json(oa) << ',' << geom_json(gb) << ',' << place_json(ob) << ','
       << (p.hit ? 1 : 0) << ',' << (geom_hit ? 1 : 0) << ',' << p.mask;
    for (int k = 0; k < 6; ++k) ss << ',' << p.v[k];
    ss << ']';
    return ss.str();
}

// ------------------------------------------------------------------ scripts

static char digit_of(const PartitionData* data) {
    for (int i = 0; i < kSlots; ++i)
        if (g_obj[i]->friend_getPartitionData() == data) return static_cast<char>('0' + i);
    die("unknown partition data");
    return '?';
}
static std::string dirty_list() {
    std::string s;
    for (PartitionData* d = ThePartitionManager->*access_member(PcDirtyTag()); d;
         d = d->*access_member(PcNextDirtyTag()))
        s += digit_of(d);
    return s;
}
static std::string module_list() {
    std::string s;
    for (PartitionData* d = ThePartitionManager->*access_member(PcModuleListTag()); d; d = d->getNext())
        s += digit_of(d);
    return s;
}

struct Grid { Real cs, lox, loy, hix, hiy; };
static const Grid kGrids[5] = {
    {10.0f, 0.0f, 0.0f, 100.0f, 100.0f},
    {40.0f, 0.0f, 0.0f, 200.0f, 200.0f},
    {4.0f, -40.0f, -40.0f, 40.0f, 40.0f},
    {7.3f, 13.3f, -37.5f, 90.0f, 50.0f},
    {25.0f, -100.0f, -100.0f, 100.0f, 100.0f},
};

static void set_grid(oracle_wp51::OracleTerrainLogic* terrain, const Grid& g) {
    TheWritableGlobalData->m_partitionCellSize = g.cs;
    terrain->extent.lo = {g.lox, g.loy, 0.0f};
    terrain->extent.hi = {g.hix, g.hiy, 100.0f};
    ThePartitionManager->reset();
    ThePartitionManager->init();
}

// A cluster of places, so that most scripts collide.
static Place g_centre;

static Place make_place() {
    Place p;
    if (pick(4) == 0) {
        p.x = g_centre.x + static_cast<Real>(static_cast<int>(pick(5)) - 2) * 4.0f;
        p.y = g_centre.y + static_cast<Real>(static_cast<int>(pick(5)) - 2) * 4.0f;
    } else {
        p.x = g_centre.x + spread(-12.0f, 12.0f);
        p.y = g_centre.y + spread(-12.0f, 12.0f);
    }
    p.z = pick(2) ? 0.0f : spread(-4.0f, 8.0f);
    if (pick(30) == 0) p.x = g_centre.x + 5000.0f;  // off the map
    p.angle = make_angle();
    // Read the angle back through the engine, so that recorded places round trip.
    g_scratch->setOrientation(p.angle);
    p.angle = g_scratch->getOrientation();
    return p;
}

static std::string place_values(const Place& p) {
    std::ostringstream ss;
    ss << bits(p.x) << ',' << bits(p.y) << ',' << bits(p.z) << ',' << bits(p.angle);
    return ss.str();
}

static Reaction make_reaction(int n) {
    Reaction r;
    r.kind = pick(100) < 55 ? 0 : 1 + static_cast<int>(pick(13));
    r.target = static_cast<int>(pick(n));
    r.to = make_place();
    return r;
}
static std::string reaction_values(const Reaction& r) {
    return std::to_string(r.kind) + "," + std::to_string(r.target) + "," + place_values(r.to);
}

static uint32_t g_updates = 0, g_empty_updates = 0, g_query_hits = 0, g_query_misses = 0;

static std::string run_vector(oracle_wp51::OracleTerrainLogic* terrain, int grid_index) {
    const Grid& grid = kGrids[grid_index];
    set_grid(terrain, grid);
    g_centre.x = grid.lox + spread(0.2f, 0.8f) * (grid.hix - grid.lox);
    g_centre.y = grid.loy + spread(0.2f, 0.8f) * (grid.hiy - grid.loy);
    const int n = 2 + static_cast<int>(pick(pick(2) ? 5 : 7));

    // Choose the template variants before the pool is touched.
    int variants[kSlots];
    for (int i = 0; i < kSlots; ++i) {
        const uint32_t v = pick(20);
        variants[i] = v < 15 ? 0 : (v < 17 ? 1 : (v < 19 ? 2 : 3));
        g_obj[i] = g_pool[variants[i]][i];
        set_status(g_obj[i], OBJECT_STATUS_NO_COLLISIONS, false);
        set_status(g_obj[i], OBJECT_STATUS_DESTROYED, false);
    }

    std::ostringstream ss;
    ss << "{\"g\":" << grid_index << ",\"o\":[";
    for (int i = 0; i < n; ++i) {
        Geom g = make_geom(static_cast<int>(pick(3)));
        if (g.major > 8.0f) g.major = 8.0f;
        if (g.minor > 8.0f) g.minor = 8.0f;
        apply_geom(g_obj[i], g);
        const Place p = make_place();
        move_object(g_obj[i], p);
        ss << (i ? "," : "") << '[' << variants[i] << ',' << geom_json(g) << ',' << place_json(g_obj[i]) << ']';
    }
    ss << "],\"r\":[";
    for (int i = 0; i < n; ++i) {
        g_react[i] = make_reaction(n);
        ss << (i ? "," : "") << '[' << reaction_values(g_react[i]) << ']';
    }
    ss << "],\"s\":[";
    bool first = true;
    auto row = [&](const std::string& op) {
        ss << (first ? "" : ",") << '[' << op << ",\"" << dirty_list() << "\",\"" << module_list() << "\"]";
        first = false;
    };
    auto update = [&]() {
        g_events.clear();
        g_first_self = g_first_other = -1;
        ThePartitionManager->update();
        ++g_updates;
        if (g_events.empty()) ++g_empty_updates;
        row("7,[" + g_events + "]");
    };
    auto op_register = [&](int i) {
        ThePartitionManager->registerObject(g_obj[i]);
        row("0," + std::to_string(i));
    };

    for (int i = 0; i < n; ++i)
        if (pick(10)) op_register(i);
    update();
    const int ops = 6 + static_cast<int>(pick(12));
    for (int k = 0; k < ops; ++k) {
        const int i = static_cast<int>(pick(n));
        Object* o = g_obj[i];
        const std::string id = std::to_string(i);
        switch (pick(22)) {
        case 0: case 1: op_register(i); break;
        case 2:
            ThePartitionManager->unRegisterObject(o);
            row("1," + id);
            break;
        case 3: case 4: case 5: {
            const Place p = make_place();
            move_object(o, p);
            row("2," + id + "," + place_values(p));
            break;
        }
        case 6:
            dirty_if_registered(o);
            row("3," + id);
            break;
        case 7:
            if (o->friend_getPartitionData()) o->friend_getPartitionData()->makeDirty(false);
            row("4," + id);
            break;
        case 8: case 9: {
            const bool on = pick(2) != 0;
            set_status(o, OBJECT_STATUS_NO_COLLISIONS, on);
            row("5," + id + "," + (on ? "1" : "0"));
            break;
        }
        case 10: case 11: {
            const bool on = pick(4) == 0;
            set_status(o, OBJECT_STATUS_DESTROYED, on);
            row("6," + id + "," + (on ? "1" : "0"));
            break;
        }
        case 12:
            g_react[i] = make_reaction(n);
            row("8," + id + "," + reaction_values(g_react[i]));
            break;
        case 13: case 14: {
            const int j = static_cast<int>(pick(n));
            const bool hit = ThePartitionManager->isColliding(o, g_obj[j]) != 0;
            ++(hit ? g_query_hits : g_query_misses);
            row("9," + id + "," + std::to_string(j) + "," + (hit ? "1" : "0"));
            break;
        }
        default: update(); break;
        }
    }
    update();
    update();
    for (int i = 0; i < kSlots; ++i) ThePartitionManager->unRegisterObject(g_obj[i]);
    if (ThePartitionManager->*access_member(PcModuleListTag())) die("modules left");
    ss << "]}";
    return ss.str();
}

int main(int argc, char** argv) {
    if (argc < 2) { std::fprintf(stderr, "usage: gen_partition_collide <out.json>\n"); return 2; }
    const std::string out_path = argv[1];
    oracle_wp51::World w;
    std::string ini = "PlayerTemplate FactionPC\n  Side = America\n  PlayableSide = Yes\nEnd\n";
    const char* kinds[kVariants] = {"PRELOAD", "PRELOAD IMMOBILE", "PRELOAD NO_COLLIDE", "PRELOAD IMMOBILE NO_COLLIDE"};
    for (int v = 0; v < kVariants; ++v)
        ini += "Object PC_Object" + std::to_string(v) + "\n  KindOf = " + kinds[v] +
               "\n  Geometry = SPHERE\n  GeometryMajorRadius = 1.0\n"
               "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100.0\n    InitialHealth = 100.0\n  End\nEnd\n";
    w.load_ini(ini);
    w.init_with_neutral_player("FactionPC", nullptr);
    auto* terrain = static_cast<oracle_wp51::OracleTerrainLogic*>(TheTerrainLogic);
    set_grid(terrain, kGrids[1]);
    for (int v = 0; v < kVariants; ++v)
        for (int i = 0; i < kSlots; ++i) {
            const std::string name = "PC_Object" + std::to_string(v);
            Object* o = w.spawn_object(name.c_str(), w.team_a, {100.0f, 100.0f, 0.0f});
            if (!o) die("spawn");
            if ((o->isKindOf(KINDOF_IMMOBILE) != 0) != ((v & 1) != 0)) die("immobile kind");
            if ((o->isKindOf(KINDOF_NO_COLLIDE) != 0) != ((v & 2) != 0)) die("no-collide kind");
            attach_recorder(o);
            g_pool[v][i] = o;
            ThePartitionManager->unRegisterObject(o);
        }
    g_scratch = w.spawn_object("PC_Object0", w.team_a, {100.0f, 100.0f, 0.0f});
    if (!g_scratch) die("spawn scratch");
    ThePartitionManager->unRegisterObject(g_scratch);
    if (ThePartitionManager->*access_member(PcModuleListTag())) die("other modules registered");

    ThePartitionManager->registerObject(g_pool[0][0]);
    ThePartitionManager->registerObject(g_pool[0][1]);
    std::vector<std::string> pairs;
    for (int i = 0; i < kPairs; ++i) pairs.push_back(run_pair(i));
    ThePartitionManager->unRegisterObject(g_pool[0][0]);
    ThePartitionManager->unRegisterObject(g_pool[0][1]);

    std::vector<std::string> grid_rows;
    for (const Grid& g : kGrids) {
        std::ostringstream ss;
        ss << '[' << bits(g.cs) << ',' << bits(g.lox) << ',' << bits(g.loy) << ',' << bits(g.hix) << ','
           << bits(g.hiy) << ']';
        grid_rows.push_back(ss.str());
    }
    std::vector<std::string> vectors;
    for (int i = 0; i < kVectors; ++i) vectors.push_back(run_vector(terrain, static_cast<int>(pick(5))));

    const std::string tmp = out_path + ".tmp";
    FILE* f = std::fopen(tmp.c_str(), "wb");
    if (!f) { std::perror("open"); return 1; }
    const std::string sha = oracle::sha256_file(
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/PartitionManager.cpp");
    std::fprintf(f, "{\n\"packet\": \"PARTITION_COLLIDE\",\n\"pair_count\": %d,\n\"vector_count\": %d,\n",
                 kPairs, kVectors);
    std::fprintf(f, "\"event_count\": %u,\n\"sha256_PartitionManager\": \"%s\",\n", g_event_count, sha.c_str());
    auto block = [&](const char* name, const std::vector<std::string>& rows, bool last) {
        std::fprintf(f, "\"%s\": [\n", name);
        for (size_t i = 0; i < rows.size(); ++i)
            std::fprintf(f, "%s%s\n", rows[i].c_str(), i + 1 < rows.size() ? "," : "");
        std::fprintf(f, "]%s\n", last ? "" : ",");
    };
    block("grids", grid_rows, false);
    block("pairs", pairs, false);
    block("vectors", vectors, true);
    std::fprintf(f, "}\n");
    std::fclose(f);
    if (std::rename(tmp.c_str(), out_path.c_str())) { std::perror("rename"); return 1; }

    std::fprintf(stderr, "coverage: pair hit/miss per combo:");
    for (int c = 0; c < 9; ++c) std::fprintf(stderr, " %u/%u", g_hits[c], g_misses[c]);
    std::fprintf(stderr, "\ncoverage: unwritten_pairs=%u collide_vs_geom_differ=%u\n", g_unwritten_pairs, g_z_disagree);
    std::fprintf(stderr, "coverage: updates=%u without_events=%u events=%u second_calls=%u unwritten_events=%u "
                         "(nonzero residue in %u) queries=%u/%u\n",
                 g_updates, g_empty_updates, g_event_count, g_second_calls, g_unwritten_events,
                 g_residue_nonzero, g_query_hits, g_query_misses);
    std::fprintf(stderr, "coverage: reactions:");
    for (int k = 0; k < 14; ++k) std::fprintf(stderr, " %d=%u", k, g_reactions[k]);
    std::fprintf(stderr, "\n");
    return 0;
}
