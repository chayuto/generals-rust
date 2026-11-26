// WP-5.4-A update leaf modules: PKT-308 BaseRegenerateUpdate (manifest spelling
// BaseRenerateUpdate), PKT-316 DeletionUpdate, PKT-331 FloatUpdate, PKT-339 LifetimeUpdate,
// PKT-356 ProneUpdate, PKT-366 SmartBombTargetHomingUpdate.
//
// Canonical 3-mode build & run:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp54_a.cpp --opt O0 -o <dir>/gen_wp54_a_o0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp54_a.cpp --opt O2 -o <dir>/gen_wp54_a_o2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp54_a.cpp --opt O2 --sanitize -o <dir>/gen_wp54_a_ubsan
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy <dir>/gen_wp54_a_o2 docs/golden_vectors/WP54_A.json
//
// Each vector is a short script of operations. After every operation a row records the module's
// XferSave bytes followed by the observable world state, including the logic PRNG state.
//
// The fixture terrain is replaced by one with settable water. Water is always present while a
// FloatUpdate runs: without it `TerrainLogic::isUnderwater` leaves `waterZ` uninitialised.

#include "oracle_wp51_world.h"

#include "Common/RandomValue.h"
#include "GameClient/Drawable.h"
#include "GameLogic/Damage.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/DeletionUpdate.h"
#include "GameLogic/Module/FloatUpdate.h"
#include "GameLogic/Module/LifetimeUpdate.h"
#include "GameLogic/Module/ProneUpdate.h"
#include "GameLogic/Module/SmartBombTargetHomingUpdate.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Object.h"

#include <cstdio>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

static uint32_t bits(Real v) { uint32_t b; std::memcpy(&b, &v, 4); return b; }
static Real real_of(uint32_t b) { Real v; std::memcpy(&v, &b, 4); return v; }

// ----------------------------------------------------------------------------- terrain

static Real g_water_z = 0.0f;

class WaterTerrain : public oracle_wp51::OracleTerrainLogic {
public:
    Bool isUnderwater(Real x, Real, Real* waterZ = nullptr, Real* terrainZ = nullptr) override {
        // The height depends on x so a stale position shows up.
        if (waterZ) *waterZ = g_water_z + x * 0.125f;
        if (terrainZ) *terrainZ = 0.0f;
        return TRUE;
    }
};

// ----------------------------------------------------------------------------- helpers

static UpdateModule* find_update(Object* obj, const char* name) {
    return obj->findUpdateModule(TheNameKeyGenerator->nameToKey(name));
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

static std::string save_hex(Snapshot* s) {
    oracle::MemoryXfer xs(XFER_SAVE);
    xs.open("save");
    oracle::call_snapshot_xfer(s, &xs);
    xs.close();
    return xs.hex();
}

// Harness-local input generator, independent of the engine PRNG streams.
static uint32_t g_state = 0x54A0C0DEu;
static uint32_t next_u32() {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state;
}
static uint32_t pick(uint32_t n) { return next_u32() % n; }
static Real unit_real() { return static_cast<Real>(next_u32() % 100000u) / 100000.0f; }
static Real spread(Real lo, Real hi) { return lo + unit_real() * (hi - lo); }
template <size_t N>
static Real one_of(const Real (&a)[N]) { return a[pick(N)]; }

// ----------------------------------------------------------------------------- vectors

enum { K_DELETION = 0, K_LIFETIME, K_REGEN, K_FLOAT, K_PRONE, K_SMART, K_COUNT };
static const char* kKindName[K_COUNT] = {"deletion", "lifetime", "base_regen",
                                         "float", "prone", "smart_bomb"};
static const char* kModule[K_COUNT] = {"DeletionUpdate", "LifetimeUpdate", "BaseRegenerateUpdate",
                                       "FloatUpdate", "ProneUpdate", "SmartBombTargetHomingUpdate"};
static const char* kFields[] = {"state", "x", "y", "z", "health", "flags", "wake", "rng"};

enum {
    OP_STEP = 0, OP_KILL, OP_DAMAGE, OP_HEAL, OP_STATUS, OP_DISABLED, OP_RANGE, OP_ENABLE,
    OP_MOVE, OP_WATER, OP_PRONE, OP_TARGET, OP_GLOBALS
};

struct Op { uint32_t code, a, b, c, d; };
struct Row { std::string xfer; std::vector<uint32_t> obs; };
struct Vec {
    uint32_t kind = 0, variant = 0, frame = 0, seed = 0;
    std::vector<uint32_t> p;  // [x, y, z, a, b]
    std::vector<Op> ops;
    std::vector<Row> rows;
    uint32_t rng = 0;
    uint32_t crc = 0;
};

// (MinLifetime, MaxLifetime) in milliseconds; a negative value omits the line.
static const int kRanges[8][2] = {{-1, -1}, {0, 0},      {100, 100},     {500, 3000},
                                  {3000, 500}, {33, 34}, {10000, 60000}, {-1, 200}};
static const char* kHealth[4] = {"100.0", "333.3", "1000.0", "7.0"};
static const char* kRatio[8] = {nullptr, "0.0", "1.0", "0.5", "2.5", "-1.0", "0.1", "0.3333"};
static const char* kScalar[8] = {nullptr, "-0.5", "0.0", "0.25", "0.5", "0.99", "1.0", "1.5"};
static const uint32_t kVariants[K_COUNT] = {8, 16, 4, 2, 8, 8};

static const Real kGravity = -1.0f;

static void gen(Vec& v, bool diag) {
    const uint32_t k = v.kind;
    v.variant = pick(kVariants[k]);
    Real x = spread(50.0f, 900.0f), y = spread(50.0f, 900.0f), z = 0.0f;
    uint32_t a = 0, b = 0;
    if (k == K_LIFETIME) {
        a = pick(3) == 0 ? pick(40) + 1u : 0u;  // hulk override + 1; 0 means none
        if (pick(8) == 0) a = 1u;               // override of zero frames
    }
    if (k == K_REGEN) {
        static const Real pct[6] = {0.0f, 0.01f, 0.02f, 0.1f, 1.0f, -0.05f};
        static const uint32_t delay[5] = {0, 1, 2, 45, 300};
        a = bits(one_of(pct));
        b = delay[pick(5)];
    }
    if (k == K_SMART) {
        static const Real heights[7] = {0.0f, 8.999f, 9.0f, 9.001f, 50.0f, 300.0f, -4.0f};
        z = pick(3) == 0 ? spread(0.0f, 200.0f) : one_of(heights);
    }
    if (diag) {
        v.variant = k == K_DELETION || k == K_LIFETIME ? 3u : (k == K_FLOAT ? 1u : (k == K_REGEN ? 1u : 4u));
        if (k == K_LIFETIME) a = 0;
        if (k == K_REGEN) { a = bits(0.02f); b = 2; }
        if (k == K_SMART) z = 300.0f;
    }
    v.p = {bits(x), bits(y), bits(z), a, b};

    auto step = [&](uint32_t delta) { v.ops.push_back({OP_STEP, delta, 0, 0, 0}); };
    auto some_step = [&]() {
        static const uint32_t d[6] = {1, 1, 2, 3, 30, 2000};
        step(d[pick(6)]);
    };
    auto disable = [&](uint32_t on) { v.ops.push_back({OP_DISABLED, on, 0, 0, 0}); };

    const uint32_t count = diag ? 8u : 3u + pick(10);
    if (k == K_SMART) {
        if (diag) v.ops.push_back({OP_TARGET, 0, bits(400.0f), bits(500.0f), bits(0.0f)});
    }
    for (uint32_t i = 0; i < count; ++i) {
        switch (k) {
        case K_DELETION:
        case K_LIFETIME: {
            const uint32_t c = diag ? (i % 2 ? 0u : (i == 2 ? 6u : 0u)) : pick(12);
            if (c == 6 || c == 7) {
                uint32_t lo = pick(5) == 0 ? 0u : pick(60);
                uint32_t hi = pick(5) == 0 ? 0u : pick(60);
                v.ops.push_back({OP_RANGE, lo, hi, 0, 0});
            } else if (c == 8) {
                disable(pick(2));
            } else if (c == 9 && k == K_LIFETIME) {
                v.ops.push_back({OP_KILL, 0, 0, 0, 0});
            } else {
                some_step();
            }
            break;
        }
        case K_REGEN: {
            const uint32_t c = diag ? (i == 0 ? 2u : 0u) : pick(12);
            if (c == 2 || c == 3) {
                static const Real amt[6] = {0.0f, 0.5f, 3.0f, 40.0f, 99999.0f, 1.0f};
                const uint32_t type = diag ? 0u : (pick(5) == 0 ? 1u : (pick(4) == 0 ? 2u : 0u));
                const Real amount = diag ? 40.0f : (pick(3) == 0 ? spread(0.0f, 80.0f) : one_of(amt));
                v.ops.push_back({OP_DAMAGE, type, bits(amount), 0, 0});
            } else if (c == 4) {
                v.ops.push_back({OP_HEAL, 0, bits(spread(0.0f, 50.0f)), 0, 0});
            } else if (c == 5) {
                v.ops.push_back({OP_STATUS, pick(2), pick(2), 0, 0});
            } else if (c == 6 && pick(3) == 0) {
                v.ops.push_back({OP_KILL, 0, 0, 0, 0});
            } else if (c == 7) {
                disable(pick(2));
            } else if (c == 8 && pick(2)) {
                static const Real pct[4] = {0.0f, 0.03f, 0.25f, -0.05f};
                v.ops.push_back({OP_GLOBALS, bits(one_of(pct)), pick(10), 0, 0});
            } else {
                static const uint32_t d[6] = {1, 2, 3, 3, 45, 400};
                step(diag ? 3u : d[pick(6)]);
            }
            break;
        }
        case K_FLOAT: {
            const uint32_t c = diag ? (i % 3) : pick(10);
            if (c == 1) {
                v.ops.push_back({OP_WATER, 0, bits(diag ? 12.5f : spread(-100.0f, 300.0f)), 0, 0});
            } else if (c == 2) {
                v.ops.push_back({OP_MOVE, 0, bits(spread(0.0f, 900.0f)), bits(spread(0.0f, 900.0f)),
                                 bits(spread(-50.0f, 100.0f))});
            } else if (c == 3) {
                v.ops.push_back({OP_ENABLE, pick(2), 0, 0, 0});
            } else if (c == 4) {
                disable(pick(2));
            } else if (c == 5 && pick(3) == 0) {
                v.ops.push_back({OP_KILL, 0, 0, 0, 0});
            } else {
                step(1);
            }
            break;
        }
        case K_PRONE: {
            const uint32_t c = diag ? (i == 0 ? 0u : 5u) : pick(12);
            if (c < 3) {
                static const Real dmg[10] = {0.0f, 0.9f, 1.0f, 2.5f, 3.0f, 7.0f, -3.7f, -0.5f,
                                             1000.0f, 16777217.0f};
                const Real amount = diag ? 3.0f : (pick(4) == 0 ? spread(-5.0f, 12.0f) : one_of(dmg));
                v.ops.push_back({OP_PRONE, 0, bits(amount), 0, 0});
            } else if (c == 3) {
                v.ops.push_back({OP_STATUS, 2, pick(2), 0, 0});
            } else if (c == 4) {
                disable(pick(2));
            } else {
                step(1);
            }
            break;
        }
        default: {
            const uint32_t c = diag ? 5u : pick(12);
            if (c < 2) {
                static const Real special[5] = {0.0f, -0.0f, 1.0e-30f, 1.0e-20f, -250.0f};
                Real t[3];
                const uint32_t mode = pick(4);
                for (int j = 0; j < 3; ++j) {
                    t[j] = mode == 0 ? one_of(special) : (mode == 1 ? 0.0f : spread(-100.0f, 900.0f));
                }
                v.ops.push_back({OP_TARGET, 0, bits(t[0]), bits(t[1]), bits(t[2])});
            } else if (c == 2) {
                static const Real heights[5] = {0.0f, 9.0f, 9.001f, 40.0f, 250.0f};
                v.ops.push_back({OP_MOVE, 0, bits(spread(0.0f, 900.0f)), bits(spread(0.0f, 900.0f)),
                                 bits(one_of(heights))});
            } else if (c == 3) {
                disable(pick(2));
            } else if (c == 4 && pick(3) == 0) {
                v.ops.push_back({OP_KILL, 0, 0, 0, 0});
            } else {
                step(1);
            }
            break;
        }
        }
    }
}

static std::string template_name(uint32_t kind, uint32_t variant) {
    return "P54A_" + std::to_string(kind) + "_" + std::to_string(variant);
}

static void set_regen_globals(uint32_t pct, uint32_t delay) {
    TheWritableGlobalData->m_baseRegenHealthPercentPerSecond = real_of(pct);
    TheWritableGlobalData->m_baseRegenDelay = delay;
}

static void run(oracle_wp51::World& w, Vec& v) {
    const uint32_t k = v.kind;
    g_water_z = 5.0f;
    TheGameLogic->setHulkMaxLifetimeOverride(k == K_LIFETIME ? static_cast<Int>(v.p[3]) - 1 : -1);
    if (k == K_REGEN) set_regen_globals(v.p[3], v.p[4]);
    else set_regen_globals(0, 0);

    Object* obj = w.spawn_object(template_name(k, v.variant).c_str(), w.team_a);
    Coord3D start = {real_of(v.p[0]), real_of(v.p[1]), real_of(v.p[2])};
    obj->setPosition(&start);
    UpdateModule* m = find_update(obj, kModule[k]);
    if (!m) { std::fprintf(stderr, "missing module %s\n", kModule[k]); std::exit(1); }

    auto record = [&]() {
        std::vector<uint32_t> obs;
        obs.push_back((obj->isDestroyed() ? 1u : 0u) | (obj->isEffectivelyDead() ? 2u : 0u));
        const Coord3D* pos = obj->getPosition();
        obs.push_back(bits(pos->x));
        obs.push_back(bits(pos->y));
        obs.push_back(bits(pos->z));
        obs.push_back(bits(obj->getBodyModule()->getHealth()));
        const Bool prone = obj->getDrawable() &&
            obj->getDrawable()->getModelConditionFlags().test(MODELCONDITION_PRONE);
        obs.push_back((obj->testStatus(OBJECT_STATUS_NO_ATTACK) ? 1u : 0u) | (prone ? 2u : 0u));
        obs.push_back(m->friend_getNextCallFrame());
        obs.push_back(GetGameLogicRandomSeedCRC());
        v.rows.push_back({save_hex(m), obs});
    };

    // The first row is the state straight after construction.
    record();
    UnsignedInt frame = v.frame;
    for (const Op& op : v.ops) {
        w.set_frame(frame);
        switch (op.code) {
        case OP_STEP:
            frame += op.a;
            step_single_update(w, m, frame);
            break;
        case OP_KILL:
            obj->kill();
            break;
        case OP_DAMAGE: {
            DamageInfo d;
            d.in.m_damageType = op.a == 0 ? DAMAGE_EXPLOSION
                                          : (op.a == 1 ? DAMAGE_HEALING : DAMAGE_UNRESISTABLE);
            d.in.m_deathType = DEATH_NORMAL;
            d.in.m_amount = real_of(op.b);
            d.in.m_sourceID = INVALID_ID;
            obj->attemptDamage(&d);
            break;
        }
        case OP_HEAL:
            obj->attemptHealing(real_of(op.b), obj);
            break;
        case OP_STATUS: {
            const ObjectStatusTypes s = op.a == 0 ? OBJECT_STATUS_UNDER_CONSTRUCTION
                                      : (op.a == 1 ? OBJECT_STATUS_SOLD : OBJECT_STATUS_NO_ATTACK);
            if (op.b) obj->setStatus(MAKE_OBJECT_STATUS_MASK(s));
            else obj->clearStatus(MAKE_OBJECT_STATUS_MASK(s));
            break;
        }
        case OP_DISABLED:
            if (op.a) obj->setDisabled(DISABLED_HELD);
            else obj->clearDisabled(DISABLED_HELD);
            break;
        case OP_RANGE:
            if (k == K_DELETION) static_cast<DeletionUpdate*>(m)->setLifetimeRange(op.a, op.b);
            else static_cast<LifetimeUpdate*>(m)->setLifetimeRange(op.a, op.b);
            break;
        case OP_ENABLE:
            static_cast<FloatUpdate*>(m)->setEnabled(op.a ? TRUE : FALSE);
            break;
        case OP_MOVE: {
            Coord3D p = {real_of(op.b), real_of(op.c), real_of(op.d)};
            obj->setPosition(&p);
            break;
        }
        case OP_WATER:
            g_water_z = real_of(op.b);
            break;
        case OP_PRONE: {
            DamageInfo d;
            d.out.m_actualDamageDealt = real_of(op.b);
            static_cast<ProneUpdate*>(m)->goProne(&d);
            break;
        }
        case OP_TARGET: {
            Coord3D t = {real_of(op.b), real_of(op.c), real_of(op.d)};
            static_cast<SmartBombTargetHomingUpdate*>(m)->SetTargetPosition(t);
            break;
        }
        case OP_GLOBALS:
            set_regen_globals(op.a, op.b);
            break;
        }
        record();
    }
    v.crc = oracle_wp51::World::snapshot_crc(m);
}

static std::string body(const std::string& health) {
    return "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = " + health +
           "\n    InitialHealth = " + health + "\n  End\n";
}

static std::string fixture_ini() {
    std::ostringstream ss;
    ss << "PlayerTemplate FactionP54A\n  Side = America\n  PlayableSide = Yes\nEnd\n";
    for (uint32_t k = 0; k < K_COUNT; ++k) {
        for (uint32_t i = 0; i < kVariants[k]; ++i) {
            const bool hulk = k == K_LIFETIME && i >= 8;
            ss << "Object " << template_name(k, i) << "\n  KindOf = PRELOAD "
               << (hulk ? "VEHICLE HULK" : "VEHICLE") << "\n"
               << body(k == K_REGEN ? kHealth[i] : "100.0")
               << "  Behavior = " << kModule[k] << " ModuleTag_Update\n";
            if (k == K_DELETION || k == K_LIFETIME) {
                const int* r = kRanges[i % 8];
                if (r[0] >= 0) ss << "    MinLifetime = " << r[0] << "\n";
                if (r[1] >= 0) ss << "    MaxLifetime = " << r[1] << "\n";
            }
            if (k == K_FLOAT) ss << "    Enabled = " << (i ? "Yes" : "No") << "\n";
            if (k == K_PRONE && kRatio[i]) ss << "    DamageToFramesRatio = " << kRatio[i] << "\n";
            if (k == K_SMART && kScalar[i]) ss << "    CourseCorrectionScalar = " << kScalar[i] << "\n";
            ss << "  End\nEnd\n";
        }
    }
    return ss.str();
}

// ----------------------------------------------------------------------------- main

int main(int argc, char** argv) {
    if (argc < 2) { std::fprintf(stderr, "usage: gen_wp54_a <out.json>\n"); return 2; }
    const std::string out_path = argv[1];
    oracle_wp51::World w;
    w.load_ini(fixture_ini());
    w.init_with_neutral_player("FactionP54A", nullptr);
    TheTerrainLogic = NEW WaterTerrain;
    TheTerrainLogic->init();
    TheWritableGlobalData->m_gravity = kGravity;
    TheWritableGlobalData->m_unitDamagedThresh = 0.7f;
    TheWritableGlobalData->m_unitReallyDamagedThresh = 0.35f;

    const int kTotal = 1000;
    std::vector<Vec> vecs;
    for (int i = 0; i < kTotal; ++i) {
        Vec v;
        v.kind = static_cast<uint32_t>(i % K_COUNT);
        v.frame = 100 + pick(5000);
        v.seed = next_u32() & 0x7fffffff;
        gen(v, i < K_COUNT);
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(v.frame);
        InitRandom(v.seed);
        run(w, v);
        v.rng = GetGameLogicRandomSeedCRC();
        destroy_and_flush_all(w);
        vecs.push_back(std::move(v));
    }

    const char* root = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/";
    const char* sources[K_COUNT] = {"DeletionUpdate.cpp", "LifetimeUpdate.cpp", "BaseRenerateUpdate.cpp",
                                    "FloatUpdate.cpp", "ProneUpdate.cpp",
                                    "SmartBombTargetHomingUpdate.cpp"};

    const std::string tmp = out_path + ".tmp";
    FILE* f = std::fopen(tmp.c_str(), "wb");
    if (!f) { std::perror("open"); return 1; }
    std::fprintf(f, "{\n\"packet\": \"WP54_A\",\n\"vector_count\": %d,\n\"source_sha256\": {", kTotal);
    for (int k = 0; k < K_COUNT; ++k) {
        const std::string h = oracle::sha256_file((std::string(root) + sources[k]).c_str());
        std::fprintf(f, "%s\"%s\":\"%s\"", k ? "," : "", kKindName[k], h.c_str());
    }
    std::fprintf(f, "},\n\"seed0_diagnostics\": {");
    for (int k = 0; k < K_COUNT; ++k) {
        const Vec& v = vecs[k];
        const Row& last = v.rows.back();
        std::fprintf(f, "%s\"%s_rows\":%zu,\"%s_xfer\":\"%s\"", k ? "," : "", kKindName[k],
                     v.rows.size(), kKindName[k], v.rows.front().xfer.c_str());
        for (size_t c = 0; c < 8; ++c) {
            std::fprintf(f, ",\"%s_%s\":%u", kKindName[k], kFields[c], last.obs[c]);
        }
    }
    std::fprintf(f, "},\n\"vectors\": [\n");
    for (int i = 0; i < kTotal; ++i) {
        const Vec& v = vecs[i];
        std::fprintf(f, "{\"in\":{\"kind\":%u,\"variant\":%u,\"frame\":%u,\"seed\":%u,\"p\":[",
                     v.kind, v.variant, v.frame, v.seed);
        for (size_t k = 0; k < v.p.size(); ++k) std::fprintf(f, "%s%u", k ? "," : "", v.p[k]);
        std::fprintf(f, "],\"ops\":[");
        for (size_t k = 0; k < v.ops.size(); ++k) {
            const Op& o = v.ops[k];
            std::fprintf(f, "%s[%u,%u,%u,%u,%u]", k ? "," : "", o.code, o.a, o.b, o.c, o.d);
        }
        std::fprintf(f, "]},\"out\":{\"rng\":%u,\"crc\":%u,\"rows\":[", v.rng, v.crc);
        for (size_t k = 0; k < v.rows.size(); ++k) {
            std::fprintf(f, "%s[\"%s\"", k ? "," : "", v.rows[k].xfer.c_str());
            for (uint32_t o : v.rows[k].obs) std::fprintf(f, ",%u", o);
            std::fprintf(f, "]");
        }
        std::fprintf(f, "]}}%s\n", i + 1 < kTotal ? "," : "");
    }
    std::fprintf(f, "]\n}\n");
    if (std::fclose(f) != 0 || std::rename(tmp.c_str(), out_path.c_str()) != 0) {
        std::perror("write");
        return 1;
    }
    std::fprintf(stderr, "wrote %d vectors\n", kTotal);
    return 0;
}
