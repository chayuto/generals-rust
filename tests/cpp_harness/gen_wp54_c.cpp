// WP-5.4-C update items: PKT-369 SpecialPowerUpdateModule (abstract base),
// PKT-359 RadiusDecalUpdate, PKT-333 HeightDieUpdate.
//
// Canonical 3-mode build & run:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp54_c.cpp --opt O0 -o <dir>/gen_wp54_c_o0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp54_c.cpp --opt O2 -o <dir>/gen_wp54_c_o2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp54_c.cpp --opt O2 --sanitize -o <dir>/gen_wp54_c_ubsan
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy <dir>/gen_wp54_c_o2 docs/golden_vectors/WP54_C.json
//
// Each vector is a script of operations. After every operation a row records the
// module's XferSave bytes followed by observable state. The fixture INI is stored
// in the JSON. Stand-ins: step_single_update replaces the sleepy-update loop;
// ProbeSpecialPowerUpdate is a minimal subclass of the abstract base; the decal
// template name is written directly (it is the template's first member);
// HeightTerrain supplies a sloped ground and the deck height of one landmark bridge.
//
// HeightDie vectors in which the partition cell walk rejects a structure that the
// distance rule accepts are discarded and drawn again; "cell_walk_dropped" in the
// JSON counts them (zero with this fixture, whose structures stay close).
//
// The walk does reject in-range structures elsewhere: with 20-unit cells, an
// owner of radius 1 at altitude 110 and a 200-tall box structure are accepted by
// the distance rule up to about 102 units apart, but the walk finds the structure
// only below 68. The port has no partition cells, so it differs there.
//
// Not covered: loading a save; decal drawing and templates; the pure virtual
// special power interface; disabled owners; wall layers, rubble or multiple
// bridges; particle systems; bodies other than ActiveBody.

#include "oracle_wp51_world.h"

#include "Common/Player.h"
#include "Common/RandomValue.h"
#include "Common/Science.h"
#include "GameClient/RadiusDecal.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/HeightDieUpdate.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/TerrainLogic.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/RadiusDecalUpdate.h"
#include "GameLogic/Module/SpecialPowerUpdateModule.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Object.h"

#include <cstdio>
#include <cstring>
#include <limits>
#include <new>
#include <sstream>
#include <string>
#include <vector>

static const uint32_t kPerKind = 150;
static const uint32_t kKinds = 3;
static const uint32_t kDieVariants = 36;
static const uint32_t kStructures = 6;
static const uint32_t kStructureTemplates = 8;  // the last two are not structures

static uint32_t bits(Real v) { uint32_t b; std::memcpy(&b, &v, 4); return b; }
static Real real_of(uint32_t b) { Real v; std::memcpy(&v, &b, 4); return v; }

static uint32_t g_state = 0x54C0C0DEu;
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

static std::string save_hex(Snapshot* s) {
    oracle::MemoryXfer xs(XFER_SAVE);
    xs.open("save");
    oracle::call_snapshot_xfer(s, &xs);
    xs.close();
    return xs.hex();
}

// A save identical to the previous row of the same vector is written as "=".
static std::string compact(std::string& previous, const std::string& hex) {
    if (hex == previous) return "=";
    previous = hex;
    return hex;
}

static std::string body() {
    return "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100.0\n    InitialHealth = 100.0\n  End\n";
}

static const char* kScience[4] = {"", "SCIENCE_P54C_A", "SCIENCE_P54C_B", "SCIENCE_P54C_C"};

static std::string fixture_ini() {
    std::ostringstream ss;
    ss.precision(7);
    ss << "PlayerTemplate FactionP54C\n  Side = America\n  PlayableSide = Yes\nEnd\n";
    for (int i = 1; i < 4; ++i) ss << "Science " << kScience[i] << "\n  IsGrantable = Yes\nEnd\n";
    ss << "Object P54C_Plain\n  KindOf = PRELOAD\n" << body() << "End\n";
    ss << "Object P54C_Decal\n  KindOf = PRELOAD\n" << body()
       << "  Behavior = RadiusDecalUpdate ModuleTag_Update\n  End\nEnd\n";
    const char* types[3] = {"SPHERE", "CYLINDER", "BOX"};
    const float radii[4] = {1.0f, 3.0f, 8.0f, 25.0f};
    const float heights[3] = {2.0f, 10.0f, 40.0f};
    const char* targets[6] = {nullptr, "0.0", "3.5", "8.0", "20.0", "-4.0"};
    const char* particles[4] = {nullptr, "5.0", "40.0", "-100.0"};
    const char* delays[5] = {nullptr, "0", "100", "500", "33"};
    for (uint32_t i = 0; i < kDieVariants; ++i) {
        ss << "Object P54C_Die_" << i << "\n  KindOf = PRELOAD\n  Geometry = " << types[i % 3]
           << "\n  GeometryMajorRadius = " << radii[(i / 3) % 4] << "\n  GeometryMinorRadius = 2.0"
           << "\n  GeometryHeight = " << heights[pick(3)] << "\n  GeometryIsSmall = " << (i % 2 ? "Yes" : "No")
           << "\n" << body() << "  Behavior = HeightDieUpdate ModuleTag_Update\n";
        if (i) {  // variant 0 keeps every default
            if (const char* t = targets[pick(6)]) ss << "    TargetHeight = " << t << "\n";
            ss << "    TargetHeightIncludesStructures = " << (pick(5) < 3 ? "Yes" : "No") << "\n";
            ss << "    OnlyWhenMovingDown = " << (pick(3) == 0 ? "Yes" : "No") << "\n";
            if (const char* t = particles[pick(4)]) ss << "    DestroyAttachedParticlesAtHeight = " << t << "\n";
            ss << "    SnapToGroundOnDeath = " << (pick(3) == 0 ? "Yes" : "No") << "\n";
            if (const char* t = delays[pick(5)]) ss << "    InitialDelay = " << t << "\n";
        }
        ss << "  End\n";
        if (i % 4 == 3) ss << "  Behavior = DestroyDie ModuleTag_Die\n  End\n";
        ss << "End\n";
    }
    const float sradius[kStructureTemplates] = {6.0f, 10.0f, 4.0f, 12.0f, 8.0f, 5.0f, 6.0f, 9.0f};
    const float sheight[kStructureTemplates] = {5.0f, 18.0f, 30.0f, 12.0f, 45.0f, 60.0f, 80.0f, 55.0f};
    for (uint32_t i = 0; i < kStructureTemplates; ++i) {
        ss << "Object P54C_Structure_" << i << "\n  KindOf = PRELOAD" << (i < 6 ? " STRUCTURE" : "")
           << "\n  Geometry = " << types[(i + 1) % 3] << "\n  GeometryMajorRadius = " << sradius[i]
           << "\n  GeometryMinorRadius = " << sradius[i] * 0.5f << "\n  GeometryHeight = " << sheight[i]
           << "\n" << body() << "End\n";
    }
    ss << "Object P54C_Bridge\n  KindOf = PRELOAD\n  Geometry = BOX\n  GeometryMajorRadius = 60.0\n"
       << "  GeometryMinorRadius = 60.0\n  GeometryHeight = 1.0\n" << body() << "End\n";
    return ss.str();
}

static void step_single_update(oracle_wp51::World& w, UpdateModule* u, UnsignedInt frame) {
    const Object* obj = u->friend_getObject();
    w.set_frame(frame);
    if (!obj || obj->isDestroyed()) return;
    if (u->friend_getNextCallFrame() > frame) return;
    UpdateSleepTime sleep_len = UPDATE_SLEEP_NONE;
    DisabledMaskType dis = obj->getDisabledFlags();
    if (u->getDisabledTypesToProcess().testForAll(dis)) {
        sleep_len = u->update();
        if (sleep_len < 1) sleep_len = UPDATE_SLEEP_NONE;
    }
    UnsignedInt next = frame + static_cast<UnsignedInt>(sleep_len);
    if (next > UPDATE_SLEEP_FOREVER) next = UPDATE_SLEEP_FOREVER;
    u->friend_setNextCallFrame(next);
}

static void destroy_and_flush_all(oracle_wp51::World& w) {
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
        if (!o->isDestroyed()) TheGameLogic->destroyObject(o);
    }
    w.flush_destroy_list();
}

static UpdateModule* find_update(Object* obj, const char* name) {
    UpdateModule* m = obj->findUpdateModule(TheNameKeyGenerator->nameToKey(name));
    if (!m) { std::fprintf(stderr, "missing module %s\n", name); std::exit(1); }
    return m;
}

struct Op { uint32_t code, a, b, c, d; };
struct Vec {
    uint32_t kind, variant, frame, seed, crc, rng;
    std::vector<Op> ops;
    std::string out;
};

class ProbeSpecialPowerUpdate : public SpecialPowerUpdateModule {
    MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(ProbeSpecialPowerUpdate, "ProbeSpecialPowerUpdate")
public:
    ScienceType extra = SCIENCE_INVALID;
    ProbeSpecialPowerUpdate(Thing* thing, const ModuleData* data) : SpecialPowerUpdateModule(thing, data) {}
    ScienceType getExtraRequiredScience() const override { return extra; }
    Bool initiateIntentToDoSpecialPower(const SpecialPowerTemplate*, const Object*, const Coord3D*, const Waypoint*, UnsignedInt) override { return FALSE; }
    Bool isSpecialAbility() const override { return FALSE; }
    Bool isSpecialPower() const override { return FALSE; }
    Bool isActive() const override { return FALSE; }
    CommandOption getCommandOption() const override { return static_cast<CommandOption>(0); }
    Bool doesSpecialPowerHaveOverridableDestinationActive() const override { return FALSE; }
    Bool doesSpecialPowerHaveOverridableDestination() const override { return FALSE; }
    void setSpecialPowerOverridableDestination(const Coord3D*) override {}
    Bool isPowerCurrentlyInUse(const CommandButton* = nullptr) const override { return FALSE; }
    UpdateSleepTime update() override { return UPDATE_SLEEP_FOREVER; }
    NameKeyType getModuleNameKey() const override { return NAMEKEY_INVALID; }
};
ProbeSpecialPowerUpdate::~ProbeSpecialPowerUpdate() {}

static ScienceType science_of(uint32_t i) {
    return i ? TheScienceStore->getScienceFromInternalName(kScience[i]) : SCIENCE_INVALID;
}

static void run_special_power(oracle_wp51::World& w, Vec& v) {
    Team* teams[2] = {w.team_a, w.team_neutral};
    Object* owners[2];
    ProbeSpecialPowerUpdate* probes[2];
    static const UpdateModuleData data;
    for (int i = 0; i < 2; ++i) {
        teams[i]->getControllingPlayer()->resetSciences();
        owners[i] = w.spawn_object("P54C_Plain", teams[i]);
        // Never destroyed: the probe is not owned by the object.
        probes[i] = ::new (std::malloc(sizeof(ProbeSpecialPowerUpdate))) ProbeSpecialPowerUpdate(owners[i], &data);
    }
    std::string last_save[2];
    auto record = [&](uint32_t who, int result) {
        std::ostringstream ss;
        ss << compact(last_save[who], save_hex(probes[who])) << ',' << result << ';';
        v.out += ss.str();
    };
    record(0, 2);
    record(1, 2);
    for (const Op& op : v.ops) {
        const uint32_t who = op.a;
        int result = 2;
        switch (op.code) {
        case 0:
            probes[who]->extra = science_of(op.b);
            result = probes[who]->doesSpecialPowerUpdatePassScienceTest() ? 1 : 0;
            break;
        case 1: teams[who]->getControllingPlayer()->grantScience(science_of(op.b)); break;
        case 2: teams[who]->getControllingPlayer()->resetSciences(); break;
        case 3: probes[who]->friend_setNextCallFrame(op.b); break;
        }
        record(who, result);
    }
    v.crc = oracle_wp51::World::snapshot_crc(probes[0]);
}

static void run_decal(oracle_wp51::World& w, Vec& v) {
    Object* obj = w.spawn_object("P54C_Decal", w.team_a);
    UpdateModule* m = find_update(obj, "RadiusDecalUpdate");
    RadiusDecalUpdate* decal = static_cast<RadiusDecalUpdate*>(m);
    std::string last_save;
    auto record = [&]() {
        std::ostringstream ss;
        ss << compact(last_save, save_hex(m)) << ',' << m->friend_getNextCallFrame() << ';';
        v.out += ss.str();
    };
    record();
    UnsignedInt frame = v.frame;
    const Coord3D at = {1.0f, 2.0f, 3.0f};
    for (const Op& op : v.ops) {
        switch (op.code) {
        case 0: step_single_update(w, m, ++frame); break;
        case 1: {
            RadiusDecalTemplate tmpl;
            // m_name is the first member of RadiusDecalTemplate.
            if (op.a) *reinterpret_cast<AsciiString*>(&tmpl) = "SCCDecal";
            if (tmpl.valid() != (op.a != 0)) { std::fprintf(stderr, "template name not set\n"); std::exit(1); }
            decal->createRadiusDecal(tmpl, real_of(op.b), at);
            break;
        }
        case 2: decal->killRadiusDecal(); break;
        case 3: decal->killWhenNoLongerAttacking(op.a != 0); break;
        case 4:
            obj->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_IS_ATTACKING), op.a != 0);
            break;
        }
        record();
    }
    v.crc = oracle_wp51::World::snapshot_crc(m);
}

class HeightTerrain : public oracle_wp51::OracleTerrainLogic {
public:
    Real getGroundHeight(Real x, Real y, Coord3D* normal = nullptr) const override {
        if (normal) { normal->x = 0.0f; normal->y = 0.0f; normal->z = 1.0f; }
        return (x - 500.0f) * 0.125f + (y - 500.0f) * 0.0625f;
    }
    Real getLayerHeight(Real x, Real, PathfindLayerEnum, Coord3D* = nullptr, Bool = true) const override {
        return (x - 500.0f) * 0.5f + 3.0f;
    }
};

// True when the cell walk and the bare distance rule accept the same structures.
static bool cell_walk_agrees(Object* me, const std::vector<Object*>& others) {
    const Real range = me->getGeometryInfo().getBoundingCircleRadius();
    ObjectIterator* iter = ThePartitionManager->iterateObjectsInRange(me, range, FROM_BOUNDINGSPHERE_3D, nullptr);
    MemoryPoolObjectHolder hold(iter);
    std::vector<Object*> walked;
    for (Object* o = iter->first(); o; o = iter->next()) walked.push_back(o);
    for (Object* o : others) {
        if (!o->isKindOf(KINDOF_STRUCTURE)) continue;
        const bool near = ThePartitionManager->getDistanceSquared(me, o, FROM_BOUNDINGSPHERE_3D) < range * range;
        bool found = false;
        for (Object* f : walked) found = found || f == o;
        if (near != found) return false;
    }
    return true;
}

// Returns false when the vector has to be discarded.
static bool run_die(oracle_wp51::World& w, Vec& v) {
    Object* obj = w.spawn_object(("P54C_Die_" + std::to_string(v.variant)).c_str(), w.team_a);
    Object* container = w.spawn_object("P54C_Plain", w.team_a);
    std::vector<Object*> others;
    UpdateModule* m = find_update(obj, "HeightDieUpdate");
    std::string last_save;
    auto record = [&]() {
        const Coord3D* p = obj->getPosition();
        std::ostringstream ss;
        ss << compact(last_save, save_hex(m)) << ',' << (obj->isDestroyed() ? 1 : 0) << ','
           << (obj->isEffectivelyDead() ? 1 : 0) << ',' << bits(obj->getBodyModule()->getHealth()) << ','
           << bits(p->x) << ',' << bits(p->y) << ',' << bits(p->z) << ',' << m->friend_getNextCallFrame() << ';';
        v.out += ss.str();
    };
    record();
    UnsignedInt frame = v.frame;
    for (const Op& op : v.ops) {
        switch (op.code) {
        case 0: {
            ThePartitionManager->update();
            if (!obj->isDestroyed() && !cell_walk_agrees(obj, others)) return false;
            frame += op.a;
            step_single_update(w, m, frame);
            break;
        }
        case 1: {
            Coord3D at = {real_of(op.a), real_of(op.b), real_of(op.c)};
            obj->setPosition(&at);
            break;
        }
        case 2: obj->friend_setContainedBy(op.a ? container : nullptr); break;
        case 3: TheGameLogic->destroyObject(others[op.a]); break;
        case 4: {
            Object* o = w.spawn_object(("P54C_Structure_" + std::to_string(op.a)).c_str(), w.team_a);
            Coord3D at = {real_of(op.b), real_of(op.c), real_of(op.d)};
            o->setPosition(&at);
            others.push_back(o);
            break;
        }
        }
        record();
    }
    v.crc = oracle_wp51::World::snapshot_crc(m);
    obj->friend_setContainedBy(nullptr);
    return true;
}

static void gen_die(Vec& v, uint32_t index) {
    v.variant = index % kDieVariants;
    // Half of the vectors sit over the landmark bridge (440..560 on both axes).
    const bool on_bridge = pick(2) != 0;
    const float cx = (on_bridge ? 500.0f : 700.0f) + spread(-20.0f, 20.0f);
    const float cy = (on_bridge ? 500.0f : 300.0f) + spread(-20.0f, 20.0f);
    const float ground = (cx - 500.0f) * 0.125f + (cy - 500.0f) * 0.0625f;
    float z = ground + spread(-5.0f, 90.0f);
    v.ops.push_back({1, bits(cx), bits(cy), bits(z), 0});
    const uint32_t count = pick(kStructures);
    for (uint32_t i = 0; i < count; ++i) {
        const float reach = pick(3) ? 14.0f : 40.0f;
        const float x = cx + spread(-reach, reach);
        const float y = cy + spread(-reach, reach);
        const float sz = ground + (pick(4) ? 0.0f : spread(-10.0f, 50.0f));
        v.ops.push_back({4, pick(kStructureTemplates), bits(x), bits(y), bits(sz)});
    }
    const uint32_t n = 6 + pick(16);
    for (uint32_t i = 0; i < n; ++i) {
        const uint32_t r = pick(100);
        if (r < 50) {
            v.ops.push_back({0, pick(8) ? 1u : 1u + pick(20), 0, 0, 0});
        } else if (r < 90) {
            z += pick(4) ? -spread(0.0f, 22.0f) : spread(0.0f, 10.0f);
            if (pick(10) == 0) z = ground + spread(-5.0f, 90.0f);
            const float x = pick(3) ? cx : cx + spread(-8.0f, 8.0f);
            const float y = pick(3) ? cy : cy + spread(-8.0f, 8.0f);
            v.ops.push_back({1, bits(x), bits(y), bits(pick(15) ? z : ground), 0});
            v.ops.push_back({0, 1, 0, 0, 0});
        } else if (r < 95) {
            v.ops.push_back({2, pick(3) ? 1u : 0u, 0, 0, 0});
        } else if (count) {
            v.ops.push_back({3, pick(count), 0, 0, 0});
        }
    }
    v.ops.push_back({0, 1, 0, 0, 0});
}

static void gen_special_power(Vec& v) {
    const uint32_t n = 6 + pick(14);
    for (uint32_t i = 0; i < n; ++i) {
        Op op = {0, pick(2), 0, 0, 0};
        const uint32_t r = pick(100);
        if (r < 55) { op.code = 0; op.b = pick(4); }
        else if (r < 75) { op.code = 1; op.b = 1 + pick(3); }
        else if (r < 83) { op.code = 2; }
        else { op.code = 3; op.b = pick(3) ? next_u32() % 0x3fffffffu : 0x3fffffffu; }
        v.ops.push_back(op);
    }
}

static void gen_decal(Vec& v) {
    const float inf = std::numeric_limits<float>::infinity();
    const float radii[10] = {0.0f, -0.0f, -1.0f, 1.0f, 25.5f, 1.0e-40f, -1.0e-40f, inf, -inf,
                             std::numeric_limits<float>::quiet_NaN()};
    const uint32_t n = 8 + pick(16);
    for (uint32_t i = 0; i < n; ++i) {
        Op op = {0, 0, 0, 0, 0};
        const uint32_t r = pick(100);
        if (r < 40) { op.code = 0; }
        else if (r < 62) { op.code = 1; op.a = pick(5) ? 1 : 0; op.b = bits(pick(2) ? radii[pick(10)] : spread(-5.0f, 90.0f)); }
        else if (r < 70) { op.code = 2; }
        else if (r < 82) { op.code = 3; op.a = pick(2); }
        else { op.code = 4; op.a = pick(2); }
        v.ops.push_back(op);
    }
}

int main(int argc, char** argv) {
    if (argc < 2) { std::fprintf(stderr, "usage: gen_wp54_c <out.json>\n"); return 2; }
    const std::string out_path = argv[1];
    const std::string ini = fixture_ini();
    oracle_wp51::World w;
    w.load_ini(ini);
    w.init_with_neutral_player("FactionP54C", nullptr);
    TheTerrainLogic = NEW HeightTerrain;
    TheTerrainLogic->init();
    {
        Coord3D deck = {500.0f, 500.0f, 10.0f};
        Object* bridge = w.spawn_object("P54C_Bridge", w.team_a, deck);
        TheTerrainLogic->addLandmarkBridgeToLogic(bridge);
        const Coord3D above = {500.0f, 500.0f, 12.0f};
        if (TheTerrainLogic->getHighestLayerForDestination(&above) == LAYER_GROUND) {
            std::fprintf(stderr, "no bridge layer\n");
            return 1;
        }
        destroy_and_flush_all(w);
    }
    // A decal for the local player would need a shadow manager.
    if (w.team_a->getControllingPlayer() == ThePlayerList->getLocalPlayer()) {
        std::fprintf(stderr, "team_a is the local player\n");
        return 1;
    }

    const std::string tmp = out_path + ".tmp";
    FILE* f = std::fopen(tmp.c_str(), "wb");
    if (!f) { std::perror("open"); return 1; }
    const char* root = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/";
    std::fprintf(f, "{\n\"packet\": \"WP54_C\",\n\"vector_count\": %u,\n", kPerKind * kKinds);
    for (const char* s : {"SpecialPowerUpdateModule", "RadiusDecalUpdate", "HeightDieUpdate"}) {
        const std::string h = oracle::sha256_file((std::string(root) + s + ".cpp").c_str());
        std::fprintf(f, "\"sha256_%s\": \"%s\",\n", s, h.c_str());
    }
    static const char* hexd = "0123456789abcdef";
    std::string ini_hex;
    for (unsigned char c : ini) { ini_hex += hexd[c >> 4]; ini_hex += hexd[c & 15]; }
    std::fprintf(f, "\"ini_hex\": \"%s\",\n\"vectors\": [\n", ini_hex.c_str());

    uint32_t dropped = 0;
    for (uint32_t i = 0; i < kPerKind * kKinds; ++i) {
        Vec v = {};
        for (;;) {
            v = Vec{};
            v.kind = i % kKinds;
            v.frame = 100 + pick(5000);
            v.seed = next_u32() & 0x7fffffff;
            if (v.kind == 0) gen_special_power(v);
            else if (v.kind == 1) gen_decal(v);
            else gen_die(v, i / kKinds);
            TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
            w.set_frame(v.frame);
            InitRandom(v.seed);
            bool keep = true;
            if (v.kind == 0) run_special_power(w, v);
            else if (v.kind == 1) run_decal(w, v);
            else keep = run_die(w, v);
            v.rng = GetGameLogicRandomSeedCRC();
            destroy_and_flush_all(w);
            ThePartitionManager->update();
            if (keep) break;
            ++dropped;
        }

        std::ostringstream ops;
        for (const Op& o : v.ops) ops << o.code << ' ' << o.a << ' ' << o.b << ' ' << o.c << ' ' << o.d << ';';
        std::fprintf(f, "{\"k\":%u,\"v\":%u,\"frame\":%u,\"seed\":%u,\"crc\":%u,\"rng\":%u,\"ops\":\"%s\",\"out\":\"%s\"}%s\n",
                     v.kind, v.variant, v.frame, v.seed, v.crc, v.rng, ops.str().c_str(), v.out.c_str(),
                     i + 1 < kPerKind * kKinds ? "," : "");
    }
    std::fprintf(f, "],\n\"cell_walk_dropped\": %u\n}\n", dropped);
    if (std::fclose(f) != 0 || std::rename(tmp.c_str(), out_path.c_str()) != 0) {
        std::perror("write");
        return 1;
    }
    std::fprintf(stderr, "wrote %u vectors\n", kPerKind * kKinds);
    return 0;
}
