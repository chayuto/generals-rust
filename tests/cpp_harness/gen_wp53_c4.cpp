// WP-5.3-C4 behavior: PKT-281 NeutronBlastBehavior (manifest spelling NeutonBlastBehavior),
// and the `Object::setTeam` side effects it depends on.
//
// Canonical 3-mode build & run:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp53_c4.cpp --opt O0 -o <dir>/gen_wp53_c4_o0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp53_c4.cpp --opt O2 -o <dir>/gen_wp53_c4_o2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp53_c4.cpp --opt O2 --sanitize -o <dir>/gen_wp53_c4_ubsan
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy <dir>/gen_wp53_c4_o2 docs/golden_vectors/WP53_C4.json
//
// Each vector is a short script of operations. After every operation a row records an FNV-1a
// digest of the module's XferSave bytes followed by the observable world state, including every
// target's team and the logic PRNG state (an `aiIdle` draws from it).

#include "oracle_wp51_world.h"

#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/RandomValue.h"
#include "Common/Team.h"
#include "Common/ThingTemplate.h"
#include "GameLogic/Damage.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/NeutronBlastBehavior.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"

#include <algorithm>
#include <cstdio>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

// `Player` has no light way to defeat a side; `killPlayer` tears the whole army down.
struct PlayerDeadTag { using type = Bool Player::*; };
template struct MemberAccess<PlayerDeadTag, &Player::m_isPlayerDead>;
Bool Player::* access_member(PlayerDeadTag);

static uint32_t bits(Real v) { uint32_t b; std::memcpy(&b, &v, 4); return b; }
static Real real_of(uint32_t b) { Real v; std::memcpy(&v, &b, 4); return v; }

// ----------------------------------------------------------------------------- helpers

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

static std::string fnv_hex(const std::string& text) {
    // Digest of the bytes that `text` spells in hex.
    uint64_t h = 0xcbf29ce484222325ULL;
    for (size_t i = 0; i + 1 < text.size(); i += 2) {
        h ^= static_cast<uint64_t>(std::stoul(text.substr(i, 2), nullptr, 16));
        h *= 0x100000001b3ULL;
    }
    char buf[17];
    std::snprintf(buf, sizeof(buf), "%016llx", static_cast<unsigned long long>(h));
    return buf;
}

// Harness-local input generator, independent of the engine PRNG streams.
static uint32_t g_state = 0x7E21B40Du;
static uint32_t next_u32() {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state;
}
static uint32_t pick(uint32_t n) { return next_u32() % n; }
static Real unit_real() { return static_cast<Real>(next_u32() % 100000u) / 100000.0f; }

struct Op { uint32_t code, a, b, c, d; };
struct Row {
    std::string xfer;
    std::vector<uint32_t> obs;
};
struct Vec {
    uint32_t kind = 0, variant = 0, frame = 0, seed = 0;
    std::vector<uint32_t> p;
    std::vector<Op> ops;
    std::vector<Row> rows;
    uint32_t rng = 0;
    uint32_t crc = 0;
};

static const char* kKindName[1] = {"neutron_blast"};
static const std::vector<const char*> kFields[1] = {
    {"t0", "t1", "t2", "t3", "t4", "t5", "rider_dead_mask", "wake", "rng"},
};

static Team* team_of(oracle_wp51::World& w, uint32_t idx) {
    return idx == 0 ? w.team_neutral : (idx == 1 ? w.team_a : w.team_b);
}
static uint32_t team_index(oracle_wp51::World& w, const Object* o) {
    return o->getTeam() == w.team_neutral ? 0u : (o->getTeam() == w.team_a ? 1u : 2u);
}
static uint32_t health_bits(Object* o) { return o ? bits(o->getBodyModule()->getHealth()) : 0u; }

static void hurt(Object* o, DamageType type, Real amount, ObjectID source) {
    DamageInfo d;
    d.in.m_damageType = type;
    d.in.m_deathType = DEATH_NORMAL;
    d.in.m_amount = amount;
    d.in.m_sourceID = source;
    o->attemptDamage(&d);
}

// ----------------------------------------------------------------------------- neutron blast
// p: [bomb team, count, count x (type, team, x, y, z, flags, riders)]
// flags: 1 airborne target, 2 already dead. ops: [0 die] / [1 update]

enum { NT_INF = 0, NT_VEH, NT_DRONE, NT_BIKE, NT_AIR, NT_GARR, NT_TRANS, NT_BLDG, NT_AIVEH, NT_COUNT };
static const char* kNeutronTarget[NT_COUNT] = {
    "P53C4_Rider1", "P53C4_Veh", "P53C4_Drone", "P53C4_Bike", "P53C4_Air",
    "P53C4_Garr", "P53C4_Trans", "P53C4_PlainBldg", "P53C4_AiVeh"};

static void gen_neutron(Vec& v, int i) {
    v.variant = i == 0 ? 11u : pick(12);
    const uint32_t count = i == 0 ? 6u : 1u + pick(6);
    v.p = {i == 0 ? 1u : pick(3), count};
    static const uint32_t first[6] = {NT_INF, NT_VEH, NT_GARR, NT_TRANS, NT_BIKE, NT_AIVEH};
    for (uint32_t k = 0; k < count; ++k) {
        const uint32_t type = i == 0 ? first[k] : pick(NT_COUNT);
        const uint32_t team = i == 0 ? 2u : pick(3);
        const Real reach = pick(3) == 0 ? 90.0f : 30.0f;
        const Real x = 200.0f + (unit_real() - 0.5f) * reach;
        const Real y = 200.0f + (unit_real() - 0.5f) * reach;
        const Real z = pick(4) == 0 ? unit_real() * 100.0f : 0.0f;
        uint32_t flags = 0;
        const bool container = type == NT_GARR || type == NT_TRANS;
        if (i != 0 && !container && pick(4) == 0) flags |= 1u;
        if (i != 0 && !container && pick(8) == 0) flags |= 2u;
        const uint32_t riders = container ? (i == 0 ? 2u : pick(3)) : 0u;
        v.p.insert(v.p.end(), {type, team, bits(x), bits(y), bits(z), flags, riders});
    }
    // Relationship override per ordered player pair: 0 keep, 1 ally. Appended last.
    uint32_t allies = 0;
    if (i != 0) {
        for (uint32_t b = 0; b < 6; ++b) {
            if (pick(3) == 0) allies |= 1u << b;
        }
    }
    v.p.push_back(allies);
    if (pick(2)) v.ops.push_back({1, 1, 0, 0, 0});
    // Team changes before and after the blast.
    const uint32_t before = i == 0 ? 0u : pick(3);
    for (uint32_t k = 0; k < before; ++k) {
        if (pick(4) == 0) v.ops.push_back({3, 1 + pick(2), 0, 0, 0});
        v.ops.push_back({2, pick(count), pick(3), 0, 0});
    }
    v.ops.push_back({0, 0, 0, 0, 0});
    const uint32_t after = i == 0 ? 2u : pick(4);
    for (uint32_t k = 0; k < after; ++k) {
        if (pick(4) == 0) v.ops.push_back({3, 1 + pick(2), 0, 0, 0});
        v.ops.push_back({2, pick(count), pick(3), 0, 0});
    }
}

static const int kPairs[6][2] = {{0, 1}, {1, 0}, {0, 2}, {2, 0}, {1, 2}, {2, 1}};
static void set_dead(Player* p, bool dead) { p->*access_member(PlayerDeadTag()) = dead ? TRUE : FALSE; }

static void run_neutron(oracle_wp51::World& w, Vec& v) {
    const std::string name = "P53C4_Neutron" + std::to_string(v.variant);
    Object* bomb = w.spawn_object(name.c_str(), team_of(w, v.p[0]));
    Coord3D center = {200.0f, 200.0f, 0.0f};
    bomb->setPosition(&center);
    std::vector<Object*> target;
    std::vector<Object*> riders;
    for (uint32_t k = 0; k < v.p[1]; ++k) {
        const uint32_t* t = &v.p[2 + k * 7];
        Object* o = w.spawn_object(kNeutronTarget[t[0]], team_of(w, t[1]));
        Coord3D pos = {real_of(t[2]), real_of(t[3]), real_of(t[4])};
        o->setPosition(&pos);
        for (uint32_t r = 0; r < t[6]; ++r) {
            Object* rider = w.spawn_object("P53C4_Rider1", team_of(w, t[1]));
            rider->setPosition(&pos);
            o->getContain()->addToContain(rider);
            riders.push_back(rider);
        }
        if (t[5] & 1u) o->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_AIRBORNE_TARGET));
        if (t[5] & 2u) o->kill();
        target.push_back(o);
    }
    NeutronBlastBehavior* m = find_behavior<NeutronBlastBehavior>(bomb);
    ThePartitionManager->update();

    Player* players[3];
    Relationship saved[6];
    for (uint32_t t = 0; t < 3; ++t) players[t] = team_of(w, t)->getControllingPlayer();
    for (int b = 0; b < 6; ++b) {
        saved[b] = players[kPairs[b][0]]->getRelationship(team_of(w, kPairs[b][1]));
        if (v.p.back() & (1u << b)) {
            players[kPairs[b][0]]->setPlayerRelationship(players[kPairs[b][1]], ALLIES);
        }
    }

    InitRandom(v.seed);
    UnsignedInt frame = v.frame;
    for (const Op& op : v.ops) {
        if (op.code == 1) step_single_update(w, m, ++frame);
        if (op.code == 0) bomb->kill();
        if (op.code == 2) target[op.a]->setTeam(team_of(w, op.b));
        if (op.code == 3) set_dead(players[op.a], true);
        std::vector<uint32_t> obs(6, 0u);
        for (size_t k = 0; k < target.size(); ++k) {
            const Object* o = target[k];
            obs[k] = 1u | (o->isEffectivelyDead() ? 2u : 0u) |
                     (o->isDisabledByType(DISABLED_UNMANNED) ? 4u : 0u) | (team_index(w, o) << 4);
        }
        uint32_t rider_dead = 0;
        for (size_t k = 0; k < riders.size(); ++k) {
            if (riders[k]->isEffectivelyDead()) rider_dead |= 1u << k;
        }
        obs.push_back(rider_dead);
        obs.push_back(m->friend_getNextCallFrame() >= UPDATE_SLEEP_FOREVER ? 1u : 0u);
        obs.push_back(GetGameLogicRandomSeedCRC());
        v.rows.push_back({save_hex(m), obs});
    }
    v.crc = oracle_wp51::World::snapshot_crc(m);
    for (int b = 0; b < 6; ++b) {
        players[kPairs[b][0]]->setPlayerRelationship(players[kPairs[b][1]], saved[b]);
    }
    set_dead(players[1], false);
    set_dead(players[2], false);
}

static std::string body(const std::string& health) {
    return "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = " + health +
           "\n    InitialHealth = " + health + "\n  End\n";
}

static const char* kAi = "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n";

static std::string weapon_set(const char* weapon) {
    return std::string("  WeaponSet\n    Conditions = None\n    Weapon = PRIMARY ") + weapon +
           "\n  End\n";
}

static std::string fixture_ini() {
    std::ostringstream ss;
    ss << "PlayerTemplate FactionP53C4\n  Side = America\n  PlayableSide = Yes\nEnd\n";
    const std::string contain_ini = "  Behavior = GarrisonContain ModuleTag_Contain\n    ContainMax = 10\n  End\nEnd\n";
    ss << "Object P53C4_Rider1\n  KindOf = PRELOAD INFANTRY\n  TransportSlotCount = 1\n" << body("100.0") << "End\n";
    ss << "Object P53C4_PlainBldg\n  KindOf = PRELOAD STRUCTURE\n" << body("1000.0") << "End\n";
    ss << "Object P53C4_Garr\n  KindOf = PRELOAD STRUCTURE\n" << body("1000.0") << contain_ini;
    ss << "Object P53C4_Trans\n  KindOf = PRELOAD VEHICLE\n" << body("500.0")
       << "  Behavior = TransportContain ModuleTag_Contain\n    Slots = 8\n"
       << "    AllowInsideKindOf = INFANTRY\n  End\nEnd\n";

    // Neutron blast.
    const char* radius[3] = {"10.0", "25.0", "60.0"};
    for (int i = 0; i < 12; ++i) {
        ss << "Object P53C4_Neutron" << i << "\n  KindOf = PRELOAD PROJECTILE\n" << body("10.0")
           << "  Behavior = NeutronBlastBehavior ModuleTag_Neutron\n"
           << "    BlastRadius = " << radius[i % 3] << "\n"
           << "    AffectAirborne = " << ((i / 3) % 2 ? "Yes" : "No") << "\n"
           << "    AffectAllies = " << (i / 6 ? "Yes" : "No") << "\n  End\nEnd\n";
    }
    ss << "Object P53C4_Veh\n  KindOf = PRELOAD VEHICLE\n" << body("300.0") << "End\n";
    ss << "Object P53C4_Drone\n  KindOf = PRELOAD VEHICLE DRONE\n" << body("80.0") << "End\n";
    ss << "Object P53C4_Bike\n  KindOf = PRELOAD VEHICLE CLIFF_JUMPER\n" << body("120.0") << "End\n";
    ss << "Object P53C4_Air\n  KindOf = PRELOAD VEHICLE AIRCRAFT\n" << body("150.0") << "End\n";
    ss << "Object P53C4_AiVeh\n  KindOf = PRELOAD VEHICLE\n" << body("300.0") << kAi << "End\n";

    return ss.str();
}

// ----------------------------------------------------------------------------- main

int main(int argc, char** argv) {
    if (argc < 2) { std::fprintf(stderr, "usage: gen_wp53_c4 <out.json>\n"); return 2; }
    const std::string out_path = argv[1];
    oracle_wp51::World w;
    w.load_ini(fixture_ini());
    w.init_with_neutral_player("FactionP53C4", nullptr);
    TheWritableGlobalData->m_unitDamagedThresh = 0.7f;
    TheWritableGlobalData->m_unitReallyDamagedThresh = 0.35f;
    const Int local = w.team_a->getControllingPlayer()->getPlayerIndex();

    const int kTotal = 1000;
    std::vector<Vec> vecs;
    
    for (int i = 0; i < kTotal; ++i) {
        Vec v;
        v.kind = 0;
        const int n = i;
        v.frame = 100 + pick(5000);
        v.seed = next_u32() & 0x7fffffff;
        gen_neutron(v, n);
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(v.frame);
        InitRandom(v.seed);
        run_neutron(w, v);
        v.rng = GetGameLogicRandomSeedCRC();
        destroy_and_flush_all(w);
        ThePartitionManager->update();
        vecs.push_back(std::move(v));
    }

    const char* root = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/";
    const char* sources[1] = {"Behavior/NeutonBlastBehavior.cpp"};

    const std::string tmp = out_path + ".tmp";
    FILE* f = std::fopen(tmp.c_str(), "wb");
    if (!f) { std::perror("open"); return 1; }
    std::fprintf(f, "{\n\"packet\": \"WP53_C4\",\n\"vector_count\": %d,\n\"local_player\": %d,\n\"source_sha256\": {",
                 kTotal, local);
    for (int k = 0; k < 1; ++k) {
        const std::string h = oracle::sha256_file((std::string(root) + sources[k]).c_str());
        std::fprintf(f, "%s\"%s\":\"%s\"", k ? "," : "", kKindName[k], h.c_str());
    }
    std::fprintf(f, "},\n\"seed0_diagnostics\": {");
    for (int k = 0; k < 1; ++k) {
        const Vec& v = vecs[k];
        const Row& last = v.rows.back();
        std::fprintf(f, "%s\"%s_rows\":%zu,\"%s_xfer\":\"%s\"", k ? "," : "", kKindName[k],
                     v.rows.size(), kKindName[k], v.rows.front().xfer.c_str());
        for (size_t c = 0; c < kFields[k].size(); ++c) {
            std::fprintf(f, ",\"%s_%s\":%u", kKindName[k], kFields[k][c], last.obs[c]);
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
            const std::string& x = v.rows[k].xfer;
            std::fprintf(f, "%s[\"%s\"", k ? "," : "", x == "-" ? "-" : fnv_hex(x).c_str());
            for (uint32_t o : v.rows[k].obs) std::fprintf(f, ",%u", o);
            std::fprintf(f, "]");
        }
        const std::string& last = v.rows.back().xfer;
        const bool keep = last != "-" && last.size() <= 512;
        std::fprintf(f, "],\"final_xfer\":\"%s\"}}%s\n", keep ? last.c_str() : "-",
                     i + 1 < kTotal ? "," : "");
    }
    std::fprintf(f, "]\n}\n");
    if (std::fclose(f) != 0 || std::rename(tmp.c_str(), out_path.c_str()) != 0) {
        std::perror("write");
        return 1;
    }
    std::fprintf(stderr, "wrote %d vectors\n", kTotal);
    return 0;
}
