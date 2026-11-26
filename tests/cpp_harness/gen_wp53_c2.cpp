// WP-5.3-C2 containers: PKT-275 HelixContain.
//
// PKT-264 CaveContain, PKT-284 OverlordContain and PKT-300 TunnelContain of this wave are not
// covered: their Rust ports wait on spine decisions (docs/OPEN_DECISIONS.md).
//
// Canonical 3-mode build & run:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp53_c2.cpp --opt O0 -o <dir>/gen_wp53_c2_o0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp53_c2.cpp --opt O2 -o <dir>/gen_wp53_c2_o2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp53_c2.cpp --opt O2 --sanitize -o <dir>/gen_wp53_c2_ubsan
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy <dir>/gen_wp53_c2_o2 docs/golden_vectors/WP53_C2.json
//
// Each vector spawns one Helix and runs a short script of operations on it. A row is recorded
// after creation and after every operation: an FNV-1a digest of the module's XferSave bytes,
// the observable state as one bit per rider, and a digest of every rider's position, angle,
// health, damage state and team. Riders are numbered payload first (object id order), then in
// the order the script creates them.
//
// No vector destroys a container outright: the reference runs onDelete inside destroyObject,
// the Rust spine at the end of the frame (docs/OPEN_DECISIONS.md).

#include "oracle_wp51_world.h"

#include "Common/Player.h"
#include "Common/RandomValue.h"
#include "Common/Team.h"
#include "Common/ThingTemplate.h"
#include "GameClient/Drawable.h"
#include "GameLogic/Damage.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/HelixContain.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Object.h"

#include <algorithm>
#include <cstdio>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

template <typename T>
static T* find_behavior(Object* obj) {
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m) {
        if (T* typed = dynamic_cast<T*>(*m)) return typed;
    }
    return nullptr;
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

static const uint64_t kFnvOffset = 0xcbf29ce484222325ULL;
static void fnv_byte(uint64_t& h, uint8_t b) {
    h ^= b;
    h *= 0x100000001b3ULL;
}
static void fnv_u32(uint64_t& h, uint32_t v) {
    for (int i = 0; i < 4; ++i) fnv_byte(h, static_cast<uint8_t>(v >> (8 * i)));
}
static std::string hex64(uint64_t h) {
    char buf[17];
    std::snprintf(buf, sizeof(buf), "%016llx", static_cast<unsigned long long>(h));
    return buf;
}
static std::string fnv_hex(const std::string& text) {
    // Digest of the bytes that `text` spells in hex.
    uint64_t h = kFnvOffset;
    for (size_t i = 0; i + 1 < text.size(); i += 2) {
        fnv_byte(h, static_cast<uint8_t>(std::stoul(text.substr(i, 2), nullptr, 16)));
    }
    return hex64(h);
}

// Harness-local input generator, independent of the engine PRNG streams.
static uint32_t g_state = 0x2E91B6C5u;
static uint32_t next_u32() {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state;
}
static uint32_t pick(uint32_t n) { return next_u32() % n; }
static uint32_t bits(Real v) { uint32_t b; std::memcpy(&b, &v, 4); return b; }

static void hurt(Object* o, Real amount) {
    DamageInfo d;
    d.in.m_damageType = DAMAGE_UNRESISTABLE;
    d.in.m_deathType = DEATH_NORMAL;
    d.in.m_amount = amount;
    d.in.m_sourceID = INVALID_ID;
    o->attemptDamage(&d);
}

enum {
    OP_ADD, OP_REMOVE, OP_MOVE, OP_STEP, OP_HURT, OP_HEAL, OP_CAPTURE, OP_KILL_SELF,
    OP_KILL_RIDER, OP_EXIT_DOOR, OP_REMOVE_ALL, OP_NEST, OP_HURT_RIDER,
    OP_DESTROY_RIDER, OP_RESERVE_DOOR
};

struct Op { uint32_t code, a, b, c; };

static const int kFieldCount = 16;
static const char* kFields[kFieldCount] = {
    "count", "alive", "inside", "fire", "nested_fire", "enclosing", "valid", "held",
    "bonus", "masked", "unselectable", "health", "damage_state", "loaded", "destroyed", "door"};

struct Row {
    std::string xfer;
    std::string riders;
    uint32_t obs[kFieldCount];
};

struct Vec {
    uint32_t variant = 0, frame = 0, seed = 0, rng = 0, crc = 0;
    std::vector<Op> ops;
    std::vector<Row> rows;
};

static const int kVariants = 8;
static const char* kRiderNames[4] = {"P53C2_Inf", "P53C2_Veh", "P53C2_Gun", "P53C2_Bunker"};
static const size_t kMaxRiders = 24;

static void gen(Vec& v) {
    const uint32_t n = 6 + pick(18);
    for (uint32_t i = 0; i < n; ++i) {
        Op op = {0, 0, 0, 0};
        const uint32_t r = pick(100);
        if (r < 28) op.code = OP_ADD;
        else if (r < 36) op.code = OP_REMOVE;
        else if (r < 46) op.code = OP_MOVE;
        else if (r < 54) op.code = OP_STEP;
        else if (r < 62) op.code = OP_HURT;
        else if (r < 66) op.code = OP_HEAL;
        else if (r < 70) op.code = OP_CAPTURE;
        else if (r < 72) op.code = OP_KILL_SELF;
        else if (r < 77) op.code = OP_KILL_RIDER;
        else if (r < 83) op.code = OP_EXIT_DOOR;
        else if (r < 87) op.code = OP_REMOVE_ALL;
        else if (r < 93) op.code = OP_NEST;
        else if (r < 95) op.code = OP_HURT_RIDER;
        else if (r < 98) op.code = OP_DESTROY_RIDER;
        else op.code = OP_RESERVE_DOOR;
        switch (op.code) {
            case OP_ADD: {
                const uint32_t k = pick(10);
                op.a = k < 4 ? 0u : (k < 5 ? 1u : (k < 8 ? 2u : 3u));
                break;
            }
            case OP_MOVE:
                op.a = pick(2000);
                op.b = pick(2000);
                op.c = pick(628);
                break;
            case OP_STEP: op.a = 1 + pick(40); break;
            case OP_HURT: op.a = 20 + pick(400); break;
            case OP_HEAL: op.a = 20 + pick(600); break;
            case OP_CAPTURE: op.a = pick(3); break;
            case OP_HURT_RIDER: op.a = pick(24); op.b = 5 + pick(80); break;
            case OP_NEST: op.a = pick(24); break;
            default: op.a = pick(24); break;
        }
        v.ops.push_back(op);
        if (op.code == OP_KILL_SELF) break;
    }
}

static uint32_t team_index(oracle_wp51::World& w, const Object* o) {
    if (o->getTeam() == w.team_a) return 0;
    if (o->getTeam() == w.team_b) return 1;
    if (o->getTeam() == w.team_neutral) return 2;
    return 3;
}

static Row observe(oracle_wp51::World& w, Object* o, HelixContain* m,
                   const std::vector<Object*>& riders, uint32_t door) {
    Row row;
    row.xfer = save_hex(m);
    uint32_t alive = 0, inside = 0, fire = 0, nested = 0, encl = 0, valid = 0, held = 0,
             bonus = 0, masked = 0, unsel = 0;
    uint64_t digest = kFnvOffset;
    for (size_t k = 0; k < riders.size(); ++k) {
        Object* r = riders[k];
        const uint32_t bit = 1u << k;
        const bool live = !r->isDestroyed() && !r->isEffectivelyDead();
        if (live) alive |= bit;
        if (r->getContainedBy() == o) inside |= bit;
        if (live) {
            if (m->isPassengerAllowedToFire(r->getID())) fire |= bit;
            if (m->isEnclosingContainerFor(r)) encl |= bit;
            if (!r->getContainedBy() && m->isValidContainerFor(r, TRUE)) valid |= bit;
            if (r->getContain()) {
                // Can anyone inside this rider fire? Resolved through the outer Helix.
                const ContainedItemsList* inner = r->getContain()->getContainedItemsList();
                if (inner) {
                    for (Object* x : *inner) {
                        if (r->getContain()->isPassengerAllowedToFire(x->getID())) nested |= bit;
                    }
                }
            }
        }
        if (r->isDisabledByType(DISABLED_HELD)) held |= bit;
        if (r->testWeaponBonusCondition(WEAPONBONUSCONDITION_GARRISONED)) bonus |= bit;
        if (r->testStatus(OBJECT_STATUS_MASKED)) masked |= bit;
        if (r->testStatus(OBJECT_STATUS_UNSELECTABLE)) unsel |= bit;

        const Coord3D* p = r->getPosition();
        fnv_u32(digest, bits(p->x));
        fnv_u32(digest, bits(p->y));
        fnv_u32(digest, bits(p->z));
        fnv_u32(digest, bits(r->getOrientation()));
        fnv_u32(digest, bits(r->getBodyModule()->getHealth()));
        fnv_u32(digest, static_cast<uint32_t>(r->getBodyModule()->getDamageState()));
        fnv_u32(digest, team_index(w, r));
    }
    row.riders = hex64(digest);
    Drawable* d = o->getDrawable();
    const uint32_t loaded = d && d->getModelConditionFlags().test(MODELCONDITION_LOADED) ? 1u : 0u;
    const uint32_t values[kFieldCount] = {
        m->getContainCount(), alive, inside, fire, nested, encl, valid, held, bonus, masked, unsel,
        bits(o->getBodyModule()->getHealth()),
        static_cast<uint32_t>(o->getBodyModule()->getDamageState()), loaded,
        o->isDestroyed() ? 1u : 0u, door};
    std::memcpy(row.obs, values, sizeof(values));
    return row;
}

static void run(oracle_wp51::World& w, Vec& v) {
    const std::string name = "P53C2_Helix" + std::to_string(v.variant);
    Object* o = w.spawn_object(name.c_str(), w.team_a);
    HelixContain* m = find_behavior<HelixContain>(o);
    if (!m) { std::fprintf(stderr, "no HelixContain on %s\n", name.c_str()); std::exit(1); }

    std::vector<Object*> riders;
    for (Object* x = TheGameLogic->getFirstObject(); x; x = x->getNextObject()) {
        if (x != o && !x->isDestroyed()) riders.push_back(x);
    }
    std::sort(riders.begin(), riders.end(),
              [](const Object* l, const Object* r) { return l->getID() < r->getID(); });

    UnsignedInt frame = v.frame;
    v.rows.push_back(observe(w, o, m, riders, 99));
    for (const Op& op : v.ops) {
        uint32_t door = 99;
        Object* r = riders.empty() ? nullptr : riders[op.a % riders.size()];
        const bool live = r && !r->isDestroyed() && !r->isEffectivelyDead();
        switch (op.code) {
            case OP_ADD: {
                if (riders.size() >= kMaxRiders) break;
                Object* x = w.spawn_object(kRiderNames[op.a], o->getTeam());
                riders.push_back(x);
                if (m->isValidContainerFor(x, TRUE)) m->addToContain(x);
                break;
            }
            case OP_REMOVE:
                if (live && r->getContainedBy() == o) m->removeFromContain(r);
                break;
            case OP_MOVE: {
                Coord3D p = {static_cast<Real>(op.a), static_cast<Real>(op.b), 0.0f};
                o->setPosition(&p);
                o->setOrientation(static_cast<Real>(op.c) / 100.0f);
                break;
            }
            case OP_STEP:
                for (uint32_t k = 0; k < op.a; ++k) step_single_update(w, m, ++frame);
                break;
            case OP_HURT: w.set_frame(frame); hurt(o, static_cast<Real>(op.a)); break;
            case OP_HEAL: w.set_frame(frame); o->attemptHealing(static_cast<Real>(op.a), nullptr); break;
            case OP_CAPTURE:
                w.set_frame(frame);
                o->setTeam(op.a == 0 ? w.team_a : (op.a == 1 ? w.team_b : w.team_neutral));
                break;
            case OP_KILL_SELF: w.set_frame(frame); o->kill(); break;
            case OP_KILL_RIDER: if (live) r->kill(); break;
            case OP_EXIT_DOOR:
                if (live && r->getContainedBy() == o) m->exitObjectViaDoor(r, DOOR_1);
                break;
            case OP_REMOVE_ALL: m->removeAllContained(); break;
            case OP_NEST: {
                if (!live || !r->getContain() || riders.size() >= kMaxRiders) break;
                Object* x = w.spawn_object("P53C2_Inf", r->getTeam());
                riders.push_back(x);
                if (r->getContain()->isValidContainerFor(x, TRUE)) r->getContain()->addToContain(x);
                break;
            }
            case OP_HURT_RIDER: if (live) hurt(r, static_cast<Real>(op.b)); break;
            case OP_DESTROY_RIDER:
                // Containers are excluded: the Rust spine runs onDelete at the end of the frame.
                if (r && !r->isDestroyed() && !r->getContain()) TheGameLogic->destroyObject(r);
                break;
            case OP_RESERVE_DOOR:
                if (live) door = static_cast<uint32_t>(m->reserveDoorForExit(r->getTemplate(), r));
                break;
        }
        v.rows.push_back(observe(w, o, m, riders, door));
    }
    v.crc = oracle_wp51::World::snapshot_crc(m);
}

static std::string body(const char* health) {
    return std::string("  Body = ActiveBody ModuleTag_Body\n    MaxHealth = ") + health +
           "\n    InitialHealth = " + health + "\n  End\n";
}

static std::string fixture_ini() {
    std::ostringstream ss;
    ss << "Object P53C2_Inf\n  KindOf = PRELOAD INFANTRY\n  TransportSlotCount = 1\n"
       << body("60.0") << "End\n"
       << "Object P53C2_Veh\n  KindOf = PRELOAD VEHICLE\n  TransportSlotCount = 3\n"
       << body("200.0") << "End\n"
       << "Object P53C2_Gun\n  KindOf = PRELOAD PORTABLE_STRUCTURE\n  TransportSlotCount = 1\n"
       << body("100.0") << "End\n"
       << "Object P53C2_Bunker\n  KindOf = PRELOAD PORTABLE_STRUCTURE\n  TransportSlotCount = 1\n"
       << body("300.0")
       << "  Behavior = TransportContain ModuleTag_Contain\n    Slots = 3\n"
       << "    AllowInsideKindOf = INFANTRY\n    PassengersAllowedToFire = Yes\n  End\nEnd\n";

    const char* fields[kVariants] = {
        "    Slots = 5\n    AllowInsideKindOf = INFANTRY\n    PassengersAllowedToFire = Yes\n",
        "    Slots = 2\n    AllowInsideKindOf = INFANTRY\n    PassengersAllowedToFire = No\n"
        "    PayloadTemplateName = P53C2_Gun\n",
        "    Slots = 5\n    AllowInsideKindOf = INFANTRY\n    PassengersAllowedToFire = Yes\n"
        "    PayloadTemplateName = P53C2_Bunker\n    ShouldDrawPips = No\n",
        "    Slots = 4\n    AllowInsideKindOf = INFANTRY PORTABLE_STRUCTURE\n"
        "    PassengersAllowedToFire = Yes\n"
        "    PayloadTemplateName = P53C2_Inf P53C2_Gun P53C2_Veh P53C2_Missing\n"
        "    HealthRegen%PerSec = 10\n",
        "    Slots = 6\n    AllowInsideKindOf = INFANTRY PORTABLE_STRUCTURE\n"
        "    PassengersAllowedToFire = No\n    PayloadTemplateName = P53C2_Gun\n"
        "    PayloadTemplateName = P53C2_Bunker P53C2_Inf\n",
        "    Slots = 3\n    AllowInsideKindOf = INFANTRY VEHICLE\n"
        "    PassengersAllowedToFire = Yes\n    InitialPayload = P53C2_Inf 2\n"
        "    DestroyRidersWhoAreNotFreeToExit = Yes\n    ExitDelay = 500\n",
        "    Slots = 1\n    AllowInsideKindOf = INFANTRY\n    PassengersAllowedToFire = Yes\n"
        "    PayloadTemplateName = P53C2_Inf P53C2_Inf P53C2_Bunker\n",
        "    Slots = 0\n    PayloadTemplateName = P53C2_Veh P53C2_Bunker\n"
        "    HealthRegen%PerSec = 25\n",
    };
    for (int i = 0; i < kVariants; ++i) {
        ss << "Object P53C2_Helix" << i << "\n  KindOf = PRELOAD VEHICLE\n" << body("1000.0")
           << "  Behavior = HelixContain ModuleTag_Contain\n" << fields[i] << "  End\nEnd\n";
    }
    return ss.str();
}

// ----------------------------------------------------------------------------- main

int main(int argc, char** argv) {
    if (argc < 2) { std::fprintf(stderr, "usage: gen_wp53_c2 <out.json>\n"); return 2; }
    const std::string out_path = argv[1];
    oracle_wp51::World w;
    w.load_ini(fixture_ini());
    w.init_with_neutral_player(nullptr, nullptr);
    TheWritableGlobalData->m_unitDamagedThresh = 0.7f;
    TheWritableGlobalData->m_unitReallyDamagedThresh = 0.35f;

    const int kTotal = 1000;
    std::vector<Vec> vecs;
    for (int i = 0; i < kTotal; ++i) {
        Vec v;
        v.variant = static_cast<uint32_t>(i % kVariants);
        v.frame = 100 + pick(5000);
        v.seed = next_u32() & 0x7fffffff;
        gen(v);
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(v.frame);
        InitRandom(v.seed);
        run(w, v);
        v.rng = GetGameLogicRandomSeedCRC();
        destroy_and_flush_all(w);
        vecs.push_back(std::move(v));
    }

    const std::string root = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/";
    const std::string tmp = out_path + ".tmp";
    FILE* f = std::fopen(tmp.c_str(), "wb");
    if (!f) { std::perror("open"); return 1; }
    std::fprintf(f, "{\n\"packet\": \"WP53_C2\",\n\"vector_count\": %d,\n\"source_sha256\": {", kTotal);
    std::fprintf(f, "\"helix\":\"%s\",\"transport\":\"%s\"",
                 oracle::sha256_file((root + "Contain/HelixContain.cpp").c_str()).c_str(),
                 oracle::sha256_file((root + "Contain/TransportContain.cpp").c_str()).c_str());
    std::fprintf(f, "},\n\"seed0_diagnostics\": {");
    for (int k = 0; k < kVariants; ++k) {
        const Vec& v = vecs[k];
        const Row& last = v.rows.back();
        std::fprintf(f, "%s\"helix%d_rows\":%zu,\"helix%d_xfer\":\"%s\",\"helix%d_riders\":\"%s\"",
                     k ? "," : "", k, v.rows.size(), k, v.rows.front().xfer.c_str(), k,
                     last.riders.c_str());
        for (int c = 0; c < kFieldCount; ++c) {
            std::fprintf(f, ",\"helix%d_created_%s\":%u", k, kFields[c], v.rows.front().obs[c]);
            std::fprintf(f, ",\"helix%d_last_%s\":%u", k, kFields[c], last.obs[c]);
        }
    }
    std::fprintf(f, "},\n\"vectors\": [\n");
    for (int i = 0; i < kTotal; ++i) {
        const Vec& v = vecs[i];
        std::fprintf(f, "{\"in\":{\"variant\":%u,\"frame\":%u,\"seed\":%u,\"ops\":[",
                     v.variant, v.frame, v.seed);
        for (size_t k = 0; k < v.ops.size(); ++k) {
            const Op& o = v.ops[k];
            std::fprintf(f, "%s[%u,%u,%u,%u]", k ? "," : "", o.code, o.a, o.b, o.c);
        }
        std::fprintf(f, "]},\"out\":{\"rng\":%u,\"crc\":%u,\"rows\":[", v.rng, v.crc);
        for (size_t k = 0; k < v.rows.size(); ++k) {
            const Row& r = v.rows[k];
            std::fprintf(f, "%s[\"%s\",\"%s\"", k ? "," : "", fnv_hex(r.xfer).c_str(),
                         r.riders.c_str());
            for (uint32_t x : r.obs) std::fprintf(f, ",%u", x);
            std::fprintf(f, "]");
        }
        // The module saves about 1.6 kB (fire-point matrices); only the first vector of each
        // variant spells its last bytes out, the rest are covered by the row digests.
        std::fprintf(f, "],\"final_xfer\":\"%s\"}}%s\n",
                     i < kVariants ? v.rows.back().xfer.c_str() : "-", i + 1 < kTotal ? "," : "");
    }
    std::fprintf(f, "]\n}\n");
    if (std::fclose(f) != 0 || std::rename(tmp.c_str(), out_path.c_str()) != 0) {
        std::perror("write");
        return 1;
    }
    std::fprintf(stderr, "wrote %d vectors\n", kTotal);
    return 0;
}
