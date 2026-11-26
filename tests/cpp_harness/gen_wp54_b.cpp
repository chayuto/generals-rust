// WP-5.4-B update modules: PKT-322 DynamicGeometryInfoUpdate, PKT-355 ProjectileStreamUpdate.
//
// Canonical 3-mode build & run:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp54_b.cpp --opt O0 -o <dir>/gen_wp54_b_o0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp54_b.cpp --opt O2 -o <dir>/gen_wp54_b_o2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp54_b.cpp --opt O2 --sanitize -o <dir>/gen_wp54_b_ubsan
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy <dir>/gen_wp54_b_o2 docs/golden_vectors/WP54_B.json
//
// Each vector is a script of operations. After every operation a row records the module's
// XferSave bytes followed by observable world state. The fixture INI is stored in the JSON.

#include "oracle_wp51_world.h"

#include "Common/RandomValue.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/DynamicGeometryInfoUpdate.h"
#include "GameLogic/Module/ProjectileStreamUpdate.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Object.h"
#include "WWMath/vector3.h"

#include <cstdio>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

static const uint32_t kDynVariants = 24;
static const uint32_t kPerKind = 150;

static uint32_t bits(Real v) { uint32_t b; std::memcpy(&b, &v, 4); return b; }
static Real real_of(uint32_t b) { Real v; std::memcpy(&v, &b, 4); return v; }

static uint32_t g_state = 0x54B0C0DEu;
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

static std::string fixture_ini() {
    std::ostringstream ss;
    ss.precision(7);
    ss << "PlayerTemplate FactionP54B\n  Side = America\n  PlayableSide = Yes\nEnd\n";
    const char* types[3] = {"SPHERE", "CYLINDER", "BOX"};
    const char* keys[6] = {"InitialHeight", "InitialMajorRadius", "InitialMinorRadius",
                           "FinalHeight", "FinalMajorRadius", "FinalMinorRadius"};
    for (uint32_t i = 0; i < kDynVariants; ++i) {
        ss << "Object P54B_Dyn_" << i << "\n  KindOf = PRELOAD\n  Geometry = " << types[i % 3]
           << "\n  GeometryIsSmall = " << ((i / 3) % 2 ? "Yes" : "No")
           << "\n  GeometryMajorRadius = 4.0\n  GeometryMinorRadius = 3.0\n  GeometryHeight = 9.0\n"
           << body() << "  Behavior = DynamicGeometryInfoUpdate ModuleTag_Update\n";
        if (i) {  // variant 0 keeps every default
            ss << "    InitialDelay = " << pick(260) << "\n";
            ss << "    TransitionTime = " << (i % 7 == 3 ? 0u : pick(300)) << "\n";
            for (int k = 0; k < 6; ++k) {
                if (pick(8)) ss << "    " << keys[k] << " = " << spread(0.0f, 90.0f) << "\n";
            }
            ss << "    ReverseAtTransitionTime = " << (i % 2 ? "Yes" : "No") << "\n";
        }
        ss << "  End\nEnd\n";
    }
    ss << "Object P54B_Stream\n  KindOf = PRELOAD\n" << body()
       << "  Behavior = ProjectileStreamUpdate ModuleTag_Update\n  End\nEnd\n";
    const char* src[2] = {"VEHICLE", "INFANTRY"};
    const char* geo[2] = {"CYLINDER", "BOX"};
    for (int i = 0; i < 2; ++i) {
        for (int g = 0; g < 2; ++g) {
            ss << "Object P54B_Source_" << (i * 2 + g) << "\n  KindOf = PRELOAD " << src[i]
               << "\n  Geometry = " << geo[g]
               << "\n  GeometryMajorRadius = 10.0\n  GeometryMinorRadius = 6.0\n  GeometryHeight = 7.5\n"
               << body() << "End\n";
        }
    }
    ss << "Object P54B_Thing\n  KindOf = PRELOAD\n" << body() << "End\n";
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

static void run_dynamic(oracle_wp51::World& w, Vec& v) {
    Object* obj = w.spawn_object(("P54B_Dyn_" + std::to_string(v.variant)).c_str(), w.team_a);
    UpdateModule* m = find_update(obj, "DynamicGeometryInfoUpdate");
    std::string last_save;
    auto record = [&]() {
        const GeometryInfo& g = obj->getGeometryInfo();
        std::ostringstream ss;
        ss << compact(last_save, save_hex(m)) << ',' << (obj->isDestroyed() ? 1 : 0) << ',' << m->friend_getNextCallFrame()
           << ',' << static_cast<int>(g.getGeomType()) << ',' << (g.getIsSmall() ? 1 : 0)
           << ',' << bits(g.getMaxHeightAbovePosition()) << ',' << bits(g.getMajorRadius())
           << ',' << bits(g.getMinorRadius()) << ',' << bits(g.getBoundingCircleRadius())
           << ',' << bits(g.getBoundingSphereRadius()) << ';';
        v.out += ss.str();
    };
    record();
    UnsignedInt frame = v.frame;
    for (size_t i = 0; i < v.ops.size(); ++i) {
        step_single_update(w, m, ++frame);
        record();
    }
    v.crc = oracle_wp51::World::snapshot_crc(m);
}

static void run_stream(oracle_wp51::World& w, Vec& v) {
    Object* obj = w.spawn_object("P54B_Stream", w.team_a);
    Object* source = w.spawn_object(("P54B_Source_" + std::to_string(v.variant)).c_str(), w.team_a);
    Coord3D sp = {100.0f, 200.0f, 10.0f};
    source->setPosition(&sp);
    Object* victims[2] = {w.spawn_object("P54B_Thing", w.team_a), w.spawn_object("P54B_Thing", w.team_a)};
    const ObjectID source_id = source->getID();
    const ObjectID victim_ids[2] = {victims[0]->getID(), victims[1]->getID()};
    UpdateModule* m = find_update(obj, "ProjectileStreamUpdate");
    ProjectileStreamUpdate* stream = static_cast<ProjectileStreamUpdate*>(m);
    const Coord3D targets[3] = {{0.0f, 0.0f, 0.0f}, {5.0f, 6.0f, 7.0f}, {5.0f, 6.0f, 7.5f}};
    std::vector<ObjectID> projectiles;
    std::string points;
    std::string last_save;
    auto record = [&]() {
        std::ostringstream ss;
        const Coord3D* pos = obj->getPosition();
        ss << compact(last_save, save_hex(m)) << ',' << (obj->isDestroyed() ? 1 : 0) << ',' << m->friend_getNextCallFrame()
           << ',' << bits(pos->x) << ',' << bits(pos->y) << ',' << bits(pos->z) << points << ';';
        v.out += ss.str();
        points.clear();
    };
    record();
    UnsignedInt frame = v.frame;
    for (const Op& op : v.ops) {
        switch (op.code) {
        case 0: {
            Object* p = w.spawn_object("P54B_Thing", w.team_a);
            Coord3D at = {real_of(op.a), real_of(op.b), real_of(op.c)};
            p->setPosition(&at);
            projectiles.push_back(p->getID());
            const uint32_t d = op.d;
            const ObjectID victim = d < 2 ? victim_ids[d] : INVALID_ID;
            const Coord3D* where = (d >= 2 && d < 5) ? &targets[d - 2] : nullptr;
            stream->addProjectile(source_id, p->getID(), victim, where);
            break;
        }
        case 1:
        case 4:
        case 7: {
            ObjectID id = source_id;
            if (op.code == 1) {
                if (projectiles.empty()) break;
                id = projectiles[op.a % projectiles.size()];
            } else if (op.code == 7) {
                id = victim_ids[op.a % 2];
            }
            Object* o = TheGameLogic->findObjectByID(id);
            if (o && !o->isDestroyed()) TheGameLogic->destroyObject(o);
            break;
        }
        case 2:
            // The stream object itself may be on the list; it is not touched after that.
            if (obj->isDestroyed()) break;
            w.flush_destroy_list();
            break;
        case 3:
            step_single_update(w, m, ++frame);
            break;
        case 5: {
            Vector3 pts[MAX_PROJECTILE_STREAM];
            Int count = 0;
            stream->getAllPoints(pts, &count);
            std::ostringstream ss;
            ss << ',' << count;
            for (Int i = 0; i < count; ++i) {
                ss << ',' << bits(pts[i].X) << ',' << bits(pts[i].Y) << ',' << bits(pts[i].Z);
            }
            points = ss.str();
            break;
        }
        case 6: {
            Coord3D at = {real_of(op.a), real_of(op.b), real_of(op.c)};
            stream->setPosition(&at);
            break;
        }
        }
        record();
    }
    v.crc = oracle_wp51::World::snapshot_crc(m);
}

static void gen_stream(Vec& v, uint32_t index) {
    v.variant = index % 4;
    // Every tenth vector adds enough projectiles to wrap the ring.
    const bool wrap = index % 10 == 9;
    const uint32_t n = wrap ? 30 : 5 + pick(8);
    const uint32_t sticky = pick(5);
    for (uint32_t i = 0; i < n; ++i) {
        Op op = {0, 0, 0, 0, 0};
        const uint32_t r = pick(100);
        if (wrap ? r < 80 : r < 35) {
            op.code = 0;
            op.a = bits(100.0f + spread(-22.0f, 22.0f));
            op.b = bits(200.0f + spread(-22.0f, 22.0f));
            op.c = bits(spread(0.0f, 30.0f));
            op.d = pick(4) ? sticky : pick(6);
        } else if (r < 83) {
            const uint32_t codes[8] = {1, 1, 2, 3, 3, 5, 5, 6};
            op.code = codes[pick(8)];
            op.a = op.code == 6 ? bits(spread(-50.0f, 50.0f)) : pick(40);
            op.b = bits(spread(-50.0f, 50.0f));
            op.c = bits(spread(0.0f, 20.0f));
        } else if (r < 87) {
            op.code = 4;
        } else if (r < 90) {
            op.code = 7;
            op.a = pick(2);
        } else {
            op.code = 3;
        }
        v.ops.push_back(op);
    }
    // A third of the vectors drain the stream so the module reaches its death test.
    if (index % 3 == 0) {
        v.ops.push_back({4, 0, 0, 0, 0});
        for (uint32_t i = 0; i < n; ++i) v.ops.push_back({1, i, 0, 0, 0});
        v.ops.push_back({3, 0, 0, 0, 0});
        v.ops.push_back({2, 0, 0, 0, 0});
        v.ops.push_back({3, 0, 0, 0, 0});
        v.ops.push_back({3, 0, 0, 0, 0});
    }
}

int main(int argc, char** argv) {
    if (argc < 2) { std::fprintf(stderr, "usage: gen_wp54_b <out.json>\n"); return 2; }
    const std::string out_path = argv[1];
    const std::string ini = fixture_ini();
    oracle_wp51::World w;
    w.load_ini(ini);
    w.init_with_neutral_player("FactionP54B", nullptr);

    const std::string tmp = out_path + ".tmp";
    FILE* f = std::fopen(tmp.c_str(), "wb");
    if (!f) { std::perror("open"); return 1; }
    const char* root = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/";
    std::fprintf(f, "{\n\"packet\": \"WP54_B\",\n\"vector_count\": %u,\n", kPerKind * 2);
    for (const char* s : {"DynamicGeometryInfoUpdate", "ProjectileStreamUpdate"}) {
        const std::string h = oracle::sha256_file((std::string(root) + s + ".cpp").c_str());
        std::fprintf(f, "\"sha256_%s\": \"%s\",\n", s, h.c_str());
    }
    static const char* hexd = "0123456789abcdef";
    std::string ini_hex;
    for (unsigned char c : ini) { ini_hex += hexd[c >> 4]; ini_hex += hexd[c & 15]; }
    std::fprintf(f, "\"ini_hex\": \"%s\",\n\"vectors\": [\n", ini_hex.c_str());

    for (uint32_t i = 0; i < kPerKind * 2; ++i) {
        Vec v = {};
        v.kind = i % 2;
        const uint32_t index = i / 2;
        v.frame = 100 + pick(5000);
        v.seed = next_u32() & 0x7fffffff;
        if (v.kind == 0) {
            v.variant = index % kDynVariants;
            const uint32_t n = 4 + pick(18);
            v.ops.assign(n, Op{3, 0, 0, 0, 0});
        } else {
            gen_stream(v, index);
        }
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(v.frame);
        InitRandom(v.seed);
        if (v.kind == 0) run_dynamic(w, v);
        else run_stream(w, v);
        v.rng = GetGameLogicRandomSeedCRC();
        destroy_and_flush_all(w);

        std::ostringstream ops;
        for (const Op& o : v.ops) ops << o.code << ' ' << o.a << ' ' << o.b << ' ' << o.c << ' ' << o.d << ';';
        std::fprintf(f, "{\"k\":%u,\"v\":%u,\"frame\":%u,\"seed\":%u,\"crc\":%u,\"rng\":%u,\"ops\":\"%s\",\"out\":\"%s\"}%s\n",
                     v.kind, v.variant, v.frame, v.seed, v.crc, v.rng, ops.str().c_str(), v.out.c_str(),
                     i + 1 < kPerKind * 2 ? "," : "");
    }
    std::fprintf(f, "]\n}\n");
    if (std::fclose(f) != 0 || std::rename(tmp.c_str(), out_path.c_str()) != 0) {
        std::perror("write");
        return 1;
    }
    std::fprintf(stderr, "wrote %u vectors\n", kPerKind * 2);
    return 0;
}
