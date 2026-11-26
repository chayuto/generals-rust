// The partition grid as the engine drives it: real Objects made by
// ThingFactory::newObject, moved, reshaped, destroyed and queried through the
// engine's own entry points.
//
// Canonical 3-mode build & run:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_partition_wiring.cpp --opt O0 -o <dir>/gen_pw_o0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_partition_wiring.cpp --opt O2 -o <dir>/gen_pw_o2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_partition_wiring.cpp --opt O2 --sanitize -o <dir>/gen_pw_ubsan
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy <dir>/gen_pw_o2 docs/golden_vectors/PARTITION_WIRING.json
//
// Each vector starts from an empty world with 20-unit cells over 0..1000 and is a
// string of operations separated by ';'. Reals are float bits in hex, objects
// are numbered by creation. Every operation is "<op> <inputs>=<result>!<state>".
//   s v x y z a   newObject of template variant v (bit 0 IMMOBILE, bit 1
//                 NO_COLLIDE), setPosition, setOrientation. result: x y z a
//   m i x y z     setPosition. result: x y z a
//   t i a         setOrientation. result: x y z a
//   g i type height major minor   setGeometryInfo
//   u i value     setStatus(UNDER_CONSTRUCTION)
//   n i value     setStatus(NO_COLLISIONS)
//   d i           GameLogic::destroyObject
//   p             GameLogic::processDestroyList
//   f             PartitionManager::update. result: events "self,other,mask,6
//                 values" joined by '|', '/', then "x y z a" of each live object
//   b cell lox loy hix hiy   map boundary change: friend_prepareForMapBoundaryAdjust
//                 on every object, PartitionManager reset and init,
//                 friend_notifyOfNewMapBoundary on every object
//   r i kind target type height major minor x y z a   what the recorder of object
//                 i does from now on
//   q x y z radius    iterateObjectsInRange(pos, FROM_CENTER_2D, ITER_FASTEST)
//   o i radius        the object form, FROM_CENTER_2D
//   w i radius        the object form, FROM_BOUNDINGSPHERE_3D
// The state is "<off map> <dirty list> <module list>": one character per object
// ('-' once deleted, else isOffMap), then the PartitionManager lists, head first.
//
// onCollide is received by a collide module appended to each real Object, so
// every event went through the real Object::onCollide. "mask": bit k set when
// the C++ wrote value k; close non-sphere pairs leave values uninitialised,
// which are recorded as zero.
//
// Not covered: containment, STICK_TO_TERRAIN_SLOPE templates, create modules
// that move the object, mines (UNDER_CONSTRUCTION detonation), NaN or infinite
// inputs, TerrainLogic::setActiveBoundary itself (its two per-object calls and
// the manager reset are made here).

#include "oracle_wp51_world.h"

#include "Common/ThingFactory.h"
#include "GameLogic/Module/CollideModule.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"

#include <cstdio>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

struct PwModuleListTag { using type = PartitionData* PartitionManager::*; };
template struct MemberAccess<PwModuleListTag, &PartitionManager::m_moduleList>;
inline PartitionData* PartitionManager::* access_member(PwModuleListTag);
struct PwDirtyTag { using type = PartitionData* PartitionManager::*; };
template struct MemberAccess<PwDirtyTag, &PartitionManager::m_dirtyModules>;
inline PartitionData* PartitionManager::* access_member(PwDirtyTag);
struct PwNextDirtyTag { using type = PartitionData* PartitionData::*; };
template struct MemberAccess<PwNextDirtyTag, &PartitionData::m_nextDirty>;
inline PartitionData* PartitionData::* access_member(PwNextDirtyTag);
struct PwBehaviorsTag { using type = BehaviorModule** Object::*; };
template struct MemberAccess<PwBehaviorsTag, &Object::m_behaviors>;
inline BehaviorModule** Object::* access_member(PwBehaviorsTag);

static const int kSlots = 10;
static const int kVectors = 320;

static void die(const char* what) { std::fprintf(stderr, "harness: %s\n", what); std::exit(1); }
static std::string hex(uint32_t v) { char b[16]; std::snprintf(b, sizeof b, "%x", v); return b; }
static uint32_t bits(Real v) { uint32_t b; std::memcpy(&b, &v, 4); return b; }
static std::string hx(Real v) { return hex(bits(v)); }

static uint32_t g_state = 0x7A11C0DEu;
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
// One float step up or down in magnitude.
static Real nudge(Real v) {
    if (v == 0.0f) return v;
    uint32_t b = bits(v);
    const uint32_t k = pick(4);
    if (k == 0) ++b;
    else if (k == 1) --b;
    Real out;
    std::memcpy(&out, &b, 4);
    return out;
}

struct Geom { int type; Real height, major, minor; };
struct Reaction { int kind; int target; Geom geom; Coord3D to; Real angle; };

static Object* g_obj[kSlots];
static int g_count = 0;
static Reaction g_react[kSlots];

static uint32_t g_cov_events = 0, g_cov_reactions[12], g_cov_queries[3], g_cov_nonempty[3],
                g_cov_multi[3], g_cov_offmap = 0, g_cov_unwritten = 0, g_cov_ops[128];

static int slot_of(const Object* o) {
    for (int i = 0; i < g_count; ++i)
        if (g_obj[i] == o) return i;
    die("unknown object");
    return -1;
}

static GeometryInfo geometry(const Geom& g) {
    return GeometryInfo(static_cast<GeometryType>(g.type), false, g.height, g.major, g.minor);
}
static std::string geom_text(const Geom& g) {
    return std::to_string(g.type) + " " + hx(g.height) + " " + hx(g.major) + " " + hx(g.minor);
}
static std::string place_text(const Object* o) {
    const Coord3D* p = o->getPosition();
    return hx(p->x) + " " + hx(p->y) + " " + hx(p->z) + " " + hx(o->getOrientation());
}
static void set_status(Object* o, ObjectStatusTypes status, bool on) {
    o->setStatus(MAKE_OBJECT_STATUS_MASK(status), on);
}

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

static std::string g_events;
static int g_first_self = -1, g_first_other = -1;
static unsigned g_first_mask = 0;

static void react(Object* self, Object* other, const Reaction& r) {
    Object* third = r.target < g_count ? g_obj[r.target] : nullptr;
    ++g_cov_reactions[r.kind];
    switch (r.kind) {
    case 0: break;
    case 1: TheGameLogic->destroyObject(self); break;
    case 2: TheGameLogic->destroyObject(other); break;
    case 3: set_status(self, OBJECT_STATUS_NO_COLLISIONS, true); break;
    case 4: set_status(other, OBJECT_STATUS_NO_COLLISIONS, true); break;
    case 5: self->setPosition(&r.to); break;
    case 6: other->setPosition(&r.to); break;
    case 7: other->setOrientation(r.angle); break;
    case 8: set_status(other, OBJECT_STATUS_UNDER_CONSTRUCTION, true); break;
    case 9: other->setGeometryInfo(geometry(r.geom)); break;
    case 10: if (third) TheGameLogic->destroyObject(third); break;
    case 11: if (third) third->setPosition(&r.to); break;
    default: die("reaction");
    }
}

class RecCollide : public CollideModule {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(RecCollide, "OracleWiringCollide", 64, 64)
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
        } else {
            const Probe p = probe(self->friend_getPartitionData(), other->friend_getPartitionData());
            if (!p.hit) die("event without collision");
            mask = p.mask;
            g_first_self = s;
            g_first_other = o;
            g_first_mask = mask;
            for (int k = 0; k < 6; ++k)
                if ((mask >> k & 1u) && p.v[k] != bits(values[k])) die("probe differs from event");
        }
        if (mask != 63u) ++g_cov_unwritten;
        std::string e = std::to_string(s) + "," + std::to_string(o) + "," + std::to_string(mask);
        for (int k = 0; k < 6; ++k) e += "," + hex((mask >> k & 1u) ? bits(values[k]) : 0u);
        g_events += (g_events.empty() ? "" : "|") + e;
        ++g_cov_events;
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
    BehaviorModule** old = o->*access_member(PwBehaviorsTag());
    int n = 0;
    while (old[n]) {
        if (old[n]->getCollide()) die("template already collides");
        ++n;
    }
    BehaviorModule** list = new BehaviorModule*[n + 2];
    for (int i = 0; i < n; ++i) list[i] = old[i];
    list[n] = static_cast<BehaviorModule*>(RecCollide::friend_newModuleInstance(o, &data));
    list[n + 1] = nullptr;
    o->*access_member(PwBehaviorsTag()) = list;
}

// ------------------------------------------------------------------ world

static oracle_wp51::OracleTerrainLogic* g_terrain;
static Real g_lox, g_loy, g_hix, g_hiy;

static void set_extent(Real cs, Real lox, Real loy, Real hix, Real hiy) {
    TheWritableGlobalData->m_partitionCellSize = cs;
    g_lox = lox; g_loy = loy; g_hix = hix; g_hiy = hiy;
    g_terrain->extent.lo = {lox, loy, 0.0f};
    g_terrain->extent.hi = {hix, hiy, 100.0f};
}

static void change_boundary(Real cs, Real lox, Real loy, Real hix, Real hiy) {
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject())
        o->friend_prepareForMapBoundaryAdjust();
    set_extent(cs, lox, loy, hix, hiy);
    ThePartitionManager->reset();
    ThePartitionManager->init();
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject())
        o->friend_notifyOfNewMapBoundary();
}

static std::string state() {
    std::string s = "!";
    for (int i = 0; i < g_count; ++i) {
        if (!g_obj[i]) { s += '-'; continue; }
        s += g_obj[i]->isOffMap() ? '1' : '0';
        if (g_obj[i]->isOffMap()) ++g_cov_offmap;
    }
    s += ' ';
    for (PartitionData* d = ThePartitionManager->*access_member(PwDirtyTag()); d;
         d = d->*access_member(PwNextDirtyTag()))
        s += static_cast<char>('0' + slot_of(d->getObject()));
    s += ' ';
    for (PartitionData* d = ThePartitionManager->*access_member(PwModuleListTag()); d; d = d->getNext())
        s += static_cast<char>('0' + slot_of(d->getObject()));
    return s;
}

// Deletes the destroyed objects and forgets them.
static void process_destroy_list() {
    bool gone[kSlots] = {};
    for (int i = 0; i < g_count; ++i) gone[i] = g_obj[i] && g_obj[i]->isDestroyed();
    TheGameLogic->processDestroyList();
    for (int i = 0; i < g_count; ++i)
        if (gone[i]) g_obj[i] = nullptr;
}

// ------------------------------------------------------------------ inputs

static Real g_cx, g_cy;  // the cluster of the vector

static Coord3D make_place() {
    Coord3D p;
    const uint32_t mode = pick(20);
    if (mode < 9) {  // lattice: exact distances (3-4-5) and exact cell borders
        p.x = g_cx + static_cast<Real>(static_cast<int>(pick(7)) - 3) * (pick(2) ? 3.0f : 5.0f);
        p.y = g_cy + static_cast<Real>(static_cast<int>(pick(7)) - 3) * (pick(2) ? 4.0f : 5.0f);
    } else if (mode < 16) {
        p.x = g_cx + spread(-13.0f, 13.0f);
        p.y = g_cy + spread(-13.0f, 13.0f);
    } else if (mode < 19) {  // on or next to an edge of the current extent
        p.x = pick(2) ? nudge(pick(2) ? g_lox : g_hix) : g_cx + spread(-20.0f, 20.0f);
        p.y = pick(2) ? nudge(pick(2) ? g_loy : g_hiy) : g_cy + spread(-20.0f, 20.0f);
        if (pick(3) == 0) p.x = g_lox - spread(0.0f, 30.0f);
        if (pick(3) == 0) p.y = g_hiy + spread(0.0f, 30.0f);
    } else {  // far away
        p.x = pick(2) ? -spread(100.0f, 3000.0f) : 1000.0f + spread(100.0f, 3000.0f);
        p.y = g_cy + spread(-20.0f, 20.0f);
    }
    p.z = pick(3) ? 0.0f : (pick(2) ? static_cast<Real>(static_cast<int>(pick(9)) - 4) * 3.0f : spread(-6.0f, 10.0f));
    return p;
}

static Real make_angle() {
    const Real a[7] = {0.0f, -0.0f, 1.5707964f, -1.5707964f, 3.1415927f, 0.7853982f, -2.3561945f};
    return pick(2) ? a[pick(7)] : spread(-3.1f, 3.1f);
}

static Real make_size() {
    const Real f[10] = {0.5f, 1.0f, 2.0f, 3.0f, 5.0f, 7.5f, 10.0f, 12.0f, 22.0f, 45.0f};
    if (pick(25) == 0) return 0.0f;
    return pick(4) ? f[pick(pick(8) ? 8 : 10)] : spread(0.1f, 14.0f);
}

static Geom make_geom() {
    Geom g;
    g.type = static_cast<int>(pick(3));
    g.height = make_size();
    g.major = make_size();
    g.minor = make_size();
    return g;
}

static Real make_radius() {
    const Real f[10] = {0.0f, 3.0f, 5.0f, 10.0f, 13.0f, 15.0f, 20.0f, 25.0f, 60.0f, 6000.0f};
    if (pick(5) == 0) return spread(0.0f, 60.0f);
    return nudge(f[pick(10)]);
}

// A step on one or all axes, around the 0.01 threshold.
static Real small_step() {
    const Real f[8] = {0.0f, 0.004f, 0.009f, 0.0099f, 0.01f, 0.0101f, 0.011f, 0.02f};
    const Real v = f[pick(8)];
    return pick(2) ? v : -v;
}

static std::string coord_text(const Coord3D& p) { return hx(p.x) + " " + hx(p.y) + " " + hx(p.z); }

// ------------------------------------------------------------------ scripts

static std::string g_script;

static void row(char op, const std::string& inputs, const std::string& result = "") {
    ++g_cov_ops[static_cast<int>(op)];
    g_script += (g_script.empty() ? "" : ";") + std::string(1, op) + (inputs.empty() ? "" : " " + inputs) + "=" +
                result + state();
}

static int live_slot() {
    int live[kSlots], n = 0;
    for (int i = 0; i < g_count; ++i)
        if (g_obj[i]) live[n++] = i;
    return n ? live[pick(n)] : -1;
}

static void spawn(oracle_wp51::World& w) {
    if (g_count >= kSlots) return;
    const uint32_t v = pick(20);
    const int variant = v < 14 ? 0 : (v < 17 ? 1 : (v < 19 ? 2 : 3));
    Coord3D p = make_place();
    if (pick(25) == 0) { p.x = 0.0f; p.y = 0.0f; p.z = pick(2) ? 0.0f : 0.005f; }
    const Real angle = make_angle();
    const std::string name = "PW_Object" + std::to_string(variant);
    Object* o = w.spawn_object(name.c_str(), w.team_a, p, angle);
    if (!o) die("spawn");
    attach_recorder(o);
    const int i = g_count++;
    g_obj[i] = o;
    g_react[i] = Reaction{0, 0, {0, 1.0f, 1.0f, 1.0f}, {0.0f, 0.0f, 0.0f}, 0.0f};
    row('s', std::to_string(variant) + " " + coord_text(p) + " " + hx(angle), place_text(o));
}

static void set_reaction(int i) {
    Reaction r;
    r.kind = pick(100) < 35 ? 0 : 1 + static_cast<int>(pick(11));
    r.target = static_cast<int>(pick(g_count));
    r.geom = make_geom();
    r.to = make_place();
    r.angle = make_angle();
    g_react[i] = r;
    row('r', std::to_string(i) + " " + std::to_string(r.kind) + " " + std::to_string(r.target) + " " +
                 geom_text(r.geom) + " " + coord_text(r.to) + " " + hx(r.angle));
}

static std::string collect(ObjectIterator* iter, int form) {
    MemoryPoolObjectHolder hold(iter);
    std::string out;
    for (Object* o = iter->first(); o; o = iter->next()) out += static_cast<char>('0' + slot_of(o));
    ++g_cov_queries[form];
    if (!out.empty()) ++g_cov_nonempty[form];
    if (out.size() > 1) ++g_cov_multi[form];
    return out;
}

static void query(int form) {
    const int i = live_slot();
    const Real radius = make_radius();
    if (form == 0) {
        Coord3D c = make_place();
        if (i >= 0 && pick(2)) c = *g_obj[i]->getPosition();
        row('q', coord_text(c) + " " + hx(radius),
            collect(ThePartitionManager->iterateObjectsInRange(&c, radius, FROM_CENTER_2D, nullptr, ITER_FASTEST), 0));
    } else if (i >= 0) {
        const DistanceCalculationType dc = form == 1 ? FROM_CENTER_2D : FROM_BOUNDINGSPHERE_3D;
        row(form == 1 ? 'o' : 'w', std::to_string(i) + " " + hx(radius),
            collect(ThePartitionManager->iterateObjectsInRange(g_obj[i], radius, dc, nullptr, ITER_FASTEST), form));
    }
}

static void queries() {
    const uint32_t n = 1 + pick(3);
    for (uint32_t k = 0; k < n; ++k) query(static_cast<int>(pick(3)));
}

static void update() {
    g_events.clear();
    g_first_self = g_first_other = -1;
    ThePartitionManager->update();
    std::string places;
    for (int i = 0; i < g_count; ++i)
        if (g_obj[i]) places += (places.empty() ? "" : " ") + place_text(g_obj[i]);
    row('f', "", g_events + "/" + places);
}

static std::string run_vector(oracle_wp51::World& w) {
    g_script.clear();
    g_count = 0;
    for (int i = 0; i < kSlots; ++i) g_obj[i] = nullptr;
    set_extent(20.0f, 0.0f, 0.0f, 1000.0f, 1000.0f);
    ThePartitionManager->reset();
    ThePartitionManager->init();
    switch (pick(5)) {
    case 0: g_cx = 0.0f; g_cy = pick(2) ? 0.0f : 500.0f; break;
    case 1: g_cx = 1000.0f; g_cy = pick(2) ? 1000.0f : 40.0f; break;
    case 2: g_cx = -40.0f; g_cy = 100.0f; break;
    default: g_cx = static_cast<Real>(2 + pick(45)) * 20.0f; g_cy = static_cast<Real>(2 + pick(45)) * 20.0f; break;
    }

    const int initial = 2 + static_cast<int>(pick(6));
    for (int i = 0; i < initial; ++i) {
        spawn(w);
        if (pick(2)) {
            const Geom g = make_geom();
            g_obj[i]->setGeometryInfo(geometry(g));
            row('g', std::to_string(i) + " " + geom_text(g));
        }
    }
    for (int i = 0; i < g_count; ++i)
        if (pick(2)) set_reaction(i);
    queries();  // nothing is filed yet
    update();
    queries();

    const int ops = 14 + static_cast<int>(pick(16));
    for (int k = 0; k < ops; ++k) {
        const int i = live_slot();
        if (i < 0) { spawn(w); continue; }
        Object* o = g_obj[i];
        const std::string id = std::to_string(i);
        switch (pick(24)) {
        case 0: case 1: spawn(w); queries(); break;
        case 2: case 3: case 4: {
            const Coord3D p = make_place();
            o->setPosition(&p);
            row('m', id + " " + coord_text(p), place_text(o));
            queries();
            break;
        }
        case 5: case 6: {
            Coord3D p = *o->getPosition();
            switch (pick(4)) {
            case 0: p.x += small_step(); break;
            case 1: p.y += small_step(); break;
            case 2: p.z += small_step(); break;
            default: p.x += small_step(); p.y += small_step(); p.z += small_step(); break;
            }
            o->setPosition(&p);
            row('m', id + " " + coord_text(p), place_text(o));
            break;
        }
        case 7: {
            const Real a = pick(2) ? make_angle() : o->getOrientation() + small_step();
            o->setOrientation(a);
            row('t', id + " " + hx(a), place_text(o));
            break;
        }
        case 8: case 9: {
            const Geom g = make_geom();
            o->setGeometryInfo(geometry(g));
            row('g', id + " " + geom_text(g));
            queries();
            break;
        }
        case 10: case 11: {
            const bool on = pick(2) != 0;
            const bool construction = pick(3) != 0;
            set_status(o, construction ? OBJECT_STATUS_UNDER_CONSTRUCTION : OBJECT_STATUS_NO_COLLISIONS,
                       construction ? on : pick(3) == 0);
            const bool value = o->getStatusBits().test(
                construction ? OBJECT_STATUS_UNDER_CONSTRUCTION : OBJECT_STATUS_NO_COLLISIONS);
            row(construction ? 'u' : 'n', id + (value ? " 1" : " 0"));
            break;
        }
        case 12: case 13:
            TheGameLogic->destroyObject(o);
            row('d', id);
            queries();
            break;
        case 14:
            process_destroy_list();
            row('p', "");
            queries();
            break;
        case 15: set_reaction(i); break;
        case 16: {
            const Real cells[4] = {20.0f, 10.0f, 40.0f, 33.0f};
            const Real cs = cells[pick(4)];
            Real lox = 0.0f, loy = 0.0f, hix = 1000.0f, hiy = 1000.0f;
            if (pick(3)) {
                lox = g_cx - static_cast<Real>(pick(4)) * 5.0f;
                loy = g_cy - static_cast<Real>(pick(8)) * 10.0f;
                hix = g_cx + static_cast<Real>(1 + pick(6)) * 10.0f;
                hiy = g_cy + static_cast<Real>(1 + pick(6)) * 10.0f;
            }
            change_boundary(cs, lox, loy, hix, hiy);
            row('b', hx(cs) + " " + hx(lox) + " " + hx(loy) + " " + hx(hix) + " " + hx(hiy));
            queries();
            break;
        }
        case 17: case 18: queries(); break;
        default:
            update();
            queries();
            if (pick(2)) {
                process_destroy_list();
                row('p', "");
            }
            break;
        }
    }
    update();
    queries();
    update();

    for (int i = 0; i < g_count; ++i)
        if (g_obj[i]) {
            g_react[i].kind = 0;
            TheGameLogic->destroyObject(g_obj[i]);
        }
    process_destroy_list();
    if (TheGameLogic->getFirstObject()) die("objects left");
    if (ThePartitionManager->*access_member(PwModuleListTag())) die("modules left");
    return g_script;
}

static const char* kKinds[4] = {"PRELOAD", "PRELOAD IMMOBILE", "PRELOAD NO_COLLIDE", "PRELOAD IMMOBILE NO_COLLIDE"};

int main(int argc, char** argv) {
    if (argc < 2) { std::fprintf(stderr, "usage: gen_partition_wiring <out.json>\n"); return 2; }
    const std::string out_path = argv[1];
    oracle_wp51::World w;
    std::string objects;
    for (int v = 0; v < 4; ++v)
        objects += "Object PW_Object" + std::to_string(v) + "\n  KindOf = " + kKinds[v] +
                   "\n  Geometry = SPHERE\n  GeometryMajorRadius = 3.0\n"
                   "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100.0\n    InitialHealth = 100.0\n  End\nEnd\n";
    w.load_ini("PlayerTemplate FactionPC\n  Side = America\n  PlayableSide = Yes\nEnd\n" + objects);
    w.init_with_neutral_player("FactionPC", nullptr);
    g_terrain = static_cast<oracle_wp51::OracleTerrainLogic*>(TheTerrainLogic);
    if (TheGameLogic->getFirstObject()) die("world not empty");

    std::vector<std::string> vectors;
    for (int i = 0; i < kVectors; ++i) vectors.push_back(run_vector(w));

    const std::string tmp = out_path + ".tmp";
    FILE* f = std::fopen(tmp.c_str(), "wb");
    if (!f) { std::perror("open"); return 1; }
    const std::string sha = oracle::sha256_file(
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/PartitionManager.cpp");
    std::string ini_hex;
    for (unsigned char c : objects) {
        char b[3];
        std::snprintf(b, sizeof b, "%02x", c);
        ini_hex += b;
    }
    std::fprintf(f, "{\n\"packet\": \"PARTITION_WIRING\",\n\"vector_count\": %d,\n\"event_count\": %u,\n", kVectors,
                 g_cov_events);
    std::fprintf(f, "\"sha256_PartitionManager\": \"%s\",\n\"ini_hex\": \"%s\",\n\"vectors\": [\n", sha.c_str(),
                 ini_hex.c_str());
    for (size_t i = 0; i < vectors.size(); ++i)
        std::fprintf(f, "{\"seed\":%zu,\"v\":\"%s\"}%s\n", i, vectors[i].c_str(), i + 1 < vectors.size() ? "," : "");
    std::fprintf(f, "]\n}\n");
    std::fclose(f);
    if (std::rename(tmp.c_str(), out_path.c_str())) { std::perror("rename"); return 1; }

    std::fprintf(stderr, "coverage: events=%u unwritten=%u offmap_states=%u\ncoverage: reactions:", g_cov_events,
                 g_cov_unwritten, g_cov_offmap);
    for (int k = 0; k < 12; ++k) std::fprintf(stderr, " %d=%u", k, g_cov_reactions[k]);
    std::fprintf(stderr, "\ncoverage: queries/nonempty/multi:");
    for (int k = 0; k < 3; ++k)
        std::fprintf(stderr, " %u/%u/%u", g_cov_queries[k], g_cov_nonempty[k], g_cov_multi[k]);
    std::fprintf(stderr, "\ncoverage: ops:");
    for (const char* c = "smtgundpfbrqow"; *c; ++c) std::fprintf(stderr, " %c=%u", *c, g_cov_ops[static_cast<int>(*c)]);
    std::fprintf(stderr, "\n");
    return 0;
}
