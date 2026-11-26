// WP-5.3-C3 behaviors: PKT-262 BridgeTowerBehavior, PKT-263 BunkerBusterBehavior,
// PKT-291 PropagandaTowerBehavior.
//
// Canonical 3-mode build & run:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp53_c3.cpp --opt O0 -o <dir>/gen_wp53_c3_o0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp53_c3.cpp --opt O2 -o <dir>/gen_wp53_c3_o2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp53_c3.cpp --opt O2 --sanitize -o <dir>/gen_wp53_c3_ubsan
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy <dir>/gen_wp53_c3_o2 docs/golden_vectors/WP53_C3.json
//
// Each vector is a short script of operations. After every operation a row records an FNV-1a
// digest of one module's XferSave bytes followed by the observable world state.
//
// Effects are observed with an extra FXNugget on each fixture FXList (the map is revealed so the
// shroud test passes). The bunker buster shockwave is observed through its weapon's `FireFX`.
//
// `PropagandaTowerBehavior` saves its tracked ids in partition scan order, which the port does
// not reproduce, so the ids are sorted before the bytes are recorded. No vector puts a unit that
// a tower scans for inside an enclosing container (see `docs/OPEN_DECISIONS.md`).

#include "oracle_wp51_world.h"

#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/RandomValue.h"
#include "Common/Team.h"
#include "Common/ThingTemplate.h"
#include "Common/Upgrade.h"
#include "GameClient/FXList.h"
#include "GameLogic/Damage.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/BridgeBehavior.h"
#include "GameLogic/Module/BridgeTowerBehavior.h"
#include "GameLogic/Module/BunkerBusterBehavior.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/PropagandaTowerBehavior.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"

#include <algorithm>
#include <cstdio>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

struct CurrentVictimTag { using type = ObjectID AIUpdateInterface::*; };
template struct MemberAccess<CurrentVictimTag, &AIUpdateInterface::m_currentVictimID>;
ObjectID AIUpdateInterface::* access_member(CurrentVictimTag);

// ----------------------------------------------------------------------------- effect log

struct FxEntry { uint32_t name, x, y; };
static std::vector<FxEntry> g_fx;
static std::vector<FxEntry> g_shock;

static uint32_t bits(Real v) { uint32_t b; std::memcpy(&b, &v, 4); return b; }
static Real real_of(uint32_t b) { Real v; std::memcpy(&v, &b, 4); return v; }

enum { FX_DET = 0, FX_CRASH, FX_SHOCK, FX_PULSE, FX_UPPULSE, FX_COUNT };
static const char* kFxName[FX_COUNT] = {
    "FX_P53C3_Det", "FX_P53C3_Crash", "FX_P53C3_Shock", "FX_P53C3_Pulse", "FX_P53C3_UpPulse"};

class OracleNugget : public FXNugget {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(OracleNugget, "OracleNugget", 16, 16)
public:
    OracleNugget(uint32_t id) : m_id(id) {}
    virtual void doFXPos(const Coord3D* primary, const Matrix3D*, const Real,
                         const Coord3D*, const Real) const override {
        if (!primary) return;
        FxEntry e = {m_id, bits(primary->x), bits(primary->y)};
        (m_id == FX_SHOCK ? g_shock : g_fx).push_back(e);
    }
private:
    uint32_t m_id;
};
EMPTY_DTOR(OracleNugget)

static uint32_t log_digest(const std::vector<FxEntry>& log) {
    uint32_t h = 0x811c9dc5u;
    for (const FxEntry& e : log) {
        for (uint32_t w : {e.name, e.x, e.y}) {
            h ^= w;
            h *= 0x01000193u;
        }
    }
    return h;
}

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

static const char* kKindName[3] = {"bridge_tower", "bunker_buster", "propaganda"};
static const std::vector<const char*> kFields[3] = {
    {"hp0", "hp1", "hp2", "hp3", "bridge_hp", "dead_mask", "rng"},
    {"target_hp", "rider_hp0", "rider_hp1", "rider_hp2", "contained_mask", "dead_mask",
     "target_team", "fx_count", "fx_digest", "shock_count", "shock_digest", "rng"},
    {"hp0", "bonus0", "hp1", "bonus1", "hp2", "bonus2", "hp3", "bonus3", "tower_hp",
     "tower_bonus", "next_frame", "count", "fx_count", "fx_digest", "rng"},
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

// ----------------------------------------------------------------------------- bridge tower
// p: [bridge kind (0 BridgeBehavior, 1 plain body, 2 none), present mask, listed mask,
//     4 x health class, 4 x tower type]
// ops: [code, tower, amount bits, source kind, damage type]

static const Real kTowerHealth[4] = {100.0f, 250.0f, 333.0f, 1000.0f};
static const Real kBridgeHealth[4] = {500.0f, 1000.0f, 777.0f, 123.0f};

static void gen_bridge(Vec& v, int i) {
    v.variant = pick(4);
    const uint32_t kind = i == 0 ? 0u : (pick(10) < 7 ? 0u : 1u + pick(2));
    uint32_t present = i == 0 ? 15u : (pick(4) == 0 ? pick(16) : 15u);
    present |= 1u << pick(4);
    const uint32_t listed = i == 0 ? 15u : (pick(4) == 0 ? pick(16) : 15u);
    v.p = {kind, present, listed};
    uint32_t hc[4];
    for (int k = 0; k < 4; ++k) { hc[k] = pick(4); v.p.push_back(hc[k]); }
    for (int k = 0; k < 4; ++k) v.p.push_back(i == 0 ? static_cast<uint32_t>(k) : pick(4));

    // With a `BridgeBehavior` the total damage fraction stays below one, so nothing dies there.
    Real budget = 0.9f;
    const int n = 3 + static_cast<int>(pick(7));
    for (int k = 0; k < n; ++k) {
        uint32_t t = pick(4);
        while (!(present & (1u << t))) t = (t + 1) % 4;
        const uint32_t src = i == 0 && k == 0 ? 1u : pick(6);
        const uint32_t type = pick(2) ? DAMAGE_EXPLOSION : DAMAGE_UNRESISTABLE;
        const uint32_t roll = i == 0 && k == 0 ? 0u : pick(12);
        if (roll < 6) {
            Real frac = unit_real() * 0.25f;
            if (kind == 0) {
                if (frac >= budget) continue;
                budget -= frac;
            } else if (pick(6) == 0) {
                frac = 1.5f;
            }
            v.ops.push_back({0, t, bits(frac * kTowerHealth[hc[t]]), src, type});
        } else if (roll < 9) {
            v.ops.push_back({1, t, bits(unit_real() * 0.3f * kTowerHealth[hc[t]]), src, 0});
        } else if (roll == 9 && kind != 0) {
            v.ops.push_back({2, t, 0, 0, 0});
        } else if (roll == 10) {
            v.ops.push_back({3, t, 0, 0, 0});
        } else {
            v.ops.push_back({4, t, 0, 0, 0});
        }
    }
    if (v.ops.empty()) v.ops.push_back({4, 0, 0, 0, 0});
    if (kind == 0) {
        for (Op& op : v.ops) {
            if (op.code == 0 && real_of(op.b) <= 0.0f) op.b = bits(1.0f);
        }
    }
}

static void run_bridge(oracle_wp51::World& w, Vec& v) {
    const uint32_t kind = v.p[0], present = v.p[1], listed = v.p[2];
    Object* bridge = nullptr;
    if (kind == 0) bridge = w.spawn_object(("P53C3_Bridge" + std::to_string(v.variant)).c_str(), w.team_a);
    if (kind == 1) bridge = w.spawn_object(("P53C3_Plain" + std::to_string(v.variant)).c_str(), w.team_a);
    Object* tower[4] = {};
    BridgeTowerBehavior* mod[4] = {};
    for (int k = 0; k < 4; ++k) {
        if (!(present & (1u << k))) continue;
        tower[k] = w.spawn_object(("P53C3_Tower" + std::to_string(v.p[3 + k])).c_str(), w.team_a);
        mod[k] = find_behavior<BridgeTowerBehavior>(tower[k]);
        BridgeTowerBehaviorInterface* itf = mod[k];
        itf->setTowerType(static_cast<BridgeTowerType>(v.p[7 + k]));
        if (bridge) itf->setBridge(bridge);
        if (kind == 0 && (listed & (1u << k))) {
            find_behavior<BridgeBehavior>(bridge)->setTower(static_cast<BridgeTowerType>(k), tower[k]);
        }
    }
    Object* plain = w.spawn_object("P53C3_Source", w.team_b);

    for (const Op& op : v.ops) {
        const uint32_t t = op.a;
        Object* src = nullptr;
        ObjectID src_id = INVALID_ID;
        if (op.code <= 1) {
            if (op.c == 1) src = plain;
            if (op.c == 2) src = bridge;
            if (op.c == 3) src = tower[(t + 1) % 4];
            if (op.c == 5) src = tower[t];
            if (src) src_id = src->getID();
            if (op.c == 4) src_id = static_cast<ObjectID>(9999);
        }
        if (op.code == 0) hurt(tower[t], static_cast<DamageType>(op.d), real_of(op.b), src_id);
        if (op.code == 1) tower[t]->attemptHealing(real_of(op.b), src);
        if (op.code == 2) tower[t]->kill();
        BridgeTowerBehaviorInterface* itf = mod[t];
        if (op.code == 3) itf->setBridge(nullptr);
        if (op.code == 4 && bridge) itf->setBridge(bridge);

        uint32_t dead = 0;
        for (int k = 0; k < 4; ++k) if (tower[k] && tower[k]->isEffectivelyDead()) dead |= 1u << k;
        if (bridge && bridge->isEffectivelyDead()) dead |= 16u;
        v.rows.push_back({save_hex(mod[t]),
                          {health_bits(tower[0]), health_bits(tower[1]), health_bits(tower[2]),
                           health_bits(tower[3]), health_bits(bridge), dead,
                           GetGameLogicRandomSeedCRC()}});
    }
    for (int k = 0; k < 4; ++k) {
        if (mod[k]) { v.crc = oracle_wp51::World::snapshot_crc(mod[k]); break; }
    }
}

// ----------------------------------------------------------------------------- bunker buster
// p: [target kind (0 none, 1 plain, 2 garrison, 3 transport), rider count, 3 x rider health class,
//     has AI, bomb x, bomb y, target x, target y]
// ops: [0 victim(a: 0 target, 1 nobody, 2 other), 1 update(a frames), 2 killing-self(a), 3 die,
//       4 destroy target, 5 grant upgrade, 6 remove upgrade]

struct BustVariant { int upgrade, occupant, shock, det, crash; const char* freq; };
static const BustVariant kBust[8] = {
    {0, 0, 0, 1, 1, nullptr}, {0, 1, 1, 1, 1, "100"}, {1, 1, 1, 1, 0, "133"}, {1, 0, 1, 0, 1, "333"},
    {0, 2, 0, 1, 1, "100"},   {0, 2, 1, 0, 0, nullptr}, {1, 2, 1, 1, 1, "66"}, {0, 0, 1, 1, 1, "167"},
};

static void gen_bunker(Vec& v, int i) {
    v.variant = i == 0 ? 1u : pick(8);
    const uint32_t target = i == 0 ? 2u : (pick(8) == 0 ? pick(2) : 2u + pick(2));
    v.p = {target, i == 0 ? 3u : pick(4)};
    for (int k = 0; k < 3; ++k) v.p.push_back(i == 0 ? static_cast<uint32_t>(k + 1) : pick(4));
    v.p.push_back(i == 0 ? 1u : (pick(6) ? 1u : 0u));
    for (int k = 0; k < 4; ++k) v.p.push_back(bits(120.0f + unit_real() * 200.0f));
    if (i == 0 || pick(3) == 0) v.ops.push_back({5, 0, 0, 0, 0});
    if (i == 0 || pick(8)) v.ops.push_back({0, 0, 0, 0, 0});
    const int n = static_cast<int>(pick(5));
    for (int k = 0; k < n; ++k) {
        switch (pick(7)) {
            case 0: v.ops.push_back({0, pick(3), 0, 0, 0}); break;
            case 1: v.ops.push_back({2, pick(2), 0, 0, 0}); break;
            case 2: v.ops.push_back({5 + pick(2), 0, 0, 0, 0}); break;
            case 3: if (pick(3) == 0) v.ops.push_back({4, 0, 0, 0, 0}); break;
            default: v.ops.push_back({1, 1 + pick(12), 0, 0, 0}); break;
        }
    }
    if (i == 0 || pick(5)) v.ops.push_back({2, 1, 0, 0, 0});
    v.ops.push_back({1, 1 + pick(12), 0, 0, 0});
    v.ops.push_back({3, 0, 0, 0, 0});
}

static void run_bunker(oracle_wp51::World& w, Vec& v) {
    const uint32_t kind = v.p[0], nriders = v.p[1], has_ai = v.p[5];
    Player* player = w.team_a->getControllingPlayer();
    const UpgradeTemplate* upgrade = TheUpgradeCenter->findUpgrade("Upgrade_P53C3_Bust");
    if (player->hasUpgradeComplete(upgrade)) player->removeUpgrade(upgrade);

    const std::string bomb_name =
        "P53C3_Bomb" + std::to_string(v.variant) + (has_ai ? "A" : "N");
    Object* bomb = w.spawn_object(bomb_name.c_str(), w.team_a);
    Coord3D bp = {real_of(v.p[6]), real_of(v.p[7]), 0.0f};
    bomb->setPosition(&bp);
    Coord3D tp = {real_of(v.p[8]), real_of(v.p[9]), 0.0f};
    Object* target = nullptr;
    const char* target_name[4] = {nullptr, "P53C3_PlainBldg", "P53C3_Garr", "P53C3_Trans"};
    if (kind) {
        target = w.spawn_object(target_name[kind], w.team_b);
        target->setPosition(&tp);
    }
    Object* other = w.spawn_object("P53C3_PlainBldg", w.team_b);
    Object* rider[3] = {};
    ObjectID rider_id[3] = {INVALID_ID, INVALID_ID, INVALID_ID};
    if (kind >= 2) {
        for (uint32_t k = 0; k < nriders; ++k) {
            rider[k] = w.spawn_object(("P53C3_Rider" + std::to_string(v.p[2 + k])).c_str(), w.team_b);
            rider[k]->setPosition(&tp);
            rider_id[k] = rider[k]->getID();
            target->getContain()->addToContain(rider[k]);
        }
    }
    const ObjectID target_id = target ? target->getID() : INVALID_ID;
    BunkerBusterBehavior* m = find_behavior<BunkerBusterBehavior>(bomb);

    g_fx.clear();
    g_shock.clear();
    InitRandom(v.seed);
    UnsignedInt frame = v.frame;
    for (const Op& op : v.ops) {
        // The object shroud status behind `FXList::doFXObj` is computed from the partition cells.
        ThePartitionManager->update();
        Object* t = TheGameLogic->findObjectByID(target_id);
        if (op.code == 0 && has_ai) {
            ObjectID id = INVALID_ID;
            if (op.a == 0) id = target_id;
            if (op.a == 2) id = other->getID();
            bomb->getAI()->*access_member(CurrentVictimTag()) = id;
        }
        if (op.code == 1) for (uint32_t k = 0; k < op.a; ++k) step_single_update(w, m, ++frame);
        if (op.code == 2) bomb->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_MISSILE_KILLING_SELF), op.a != 0);
        if (op.code == 3) bomb->kill();
        if (op.code == 4 && t) {
            for (int k = 0; k < 3; ++k) {
                if (rider[k] && rider[k]->getContainedBy() == t) t->getContain()->removeFromContain(rider[k]);
            }
            TheGameLogic->destroyObject(t);
            w.flush_destroy_list();
        }
        if (op.code == 5 && !player->hasUpgradeComplete(upgrade)) player->addUpgrade(upgrade, UPGRADE_STATUS_COMPLETE);
        if (op.code == 6 && player->hasUpgradeComplete(upgrade)) player->removeUpgrade(upgrade);

        t = TheGameLogic->findObjectByID(target_id);
        uint32_t contained = 0, dead = 0, hp[3] = {0, 0, 0};
        for (int k = 0; k < 3; ++k) {
            Object* r = TheGameLogic->findObjectByID(rider_id[k]);
            if (!r) continue;
            hp[k] = health_bits(r);
            if (t && r->getContainedBy() == t) contained |= 1u << k;
            if (r->isEffectivelyDead()) dead |= 1u << k;
        }
        if (t && t->isEffectivelyDead()) dead |= 8u;
        if (bomb->isEffectivelyDead()) dead |= 16u;
        v.rows.push_back({save_hex(m),
                          {health_bits(t), hp[0], hp[1], hp[2], contained, dead,
                           t ? team_index(w, t) : 3u, static_cast<uint32_t>(g_fx.size()),
                           log_digest(g_fx), static_cast<uint32_t>(g_shock.size()),
                           log_digest(g_shock), GetGameLogicRandomSeedCRC()}});
    }
    v.crc = oracle_wp51::World::snapshot_crc(m);
}

// ----------------------------------------------------------------------------- propaganda
// p: [tower team, container (0 none, 1 transport, 2 open-topped garrison), second tower,
//     unit count, count x (type, team, x, y)]
// ops: [0 update(a frames), 1 damage unit(a, bits b), 2 move unit(a, x b, y c),
//       3 tower status(a, b set), 4 tower disable(a, b set), 5 kill unit(a), 6 destroy unit(a),
//       7 player upgrade(a set), 8 object upgrade, 9 capture tower(a team), 10 kill tower,
//       11 enter/leave container(a), 12 destroy tower]

struct PropVariant {
    const char* radius; const char* delay; const char* heal; const char* up_heal;
    int object_upgrade, affects_self; const char* kindof;
};
static const PropVariant kProp[8] = {
    {"60.0", "100", "1%", "2%", 0, 0, "STRUCTURE"},
    {"60.0", "500", "30%", "60%", 0, 1, "STRUCTURE"},
    {"120.0", "1000", "5%", "7%", 1, 0, "STRUCTURE"},
    {"120.0", "100", "30%", "10%", 1, 1, "STRUCTURE"},
    {"60.0", "333", "1%", "2%", 0, 1, "INFANTRY"},
    {"80.0", "100", "15%", "45%", 0, 0, "INFANTRY VEHICLE"},
    {"80.0", "200", "15%", "45%", 0, 1, "INFANTRY VEHICLE PORTABLE_STRUCTURE"},
    {"100.0", "0", "3%", "9%", 0, 0, "INFANTRY"},
};
static const char* kUnit[7] = {"P53C3_URifle", "P53C3_UUnarmed", "P53C3_UZero", "P53C3_UBldg",
                               "P53C3_UNoScore", "P53C3_UMp", "P53C3_UCreate"};

static uint32_t rand_pos_bits() { return bits(200.0f + (unit_real() - 0.5f) * (pick(3) ? 70.0f : 240.0f)); }

static void gen_propaganda(Vec& v, int i) {
    v.variant = i == 0 ? 1u : pick(8);
    uint32_t container = 0;
    if (i != 0 && v.variant >= 4 && pick(3) == 0) container = 1 + pick(2);
    uint32_t second = i == 0 ? 0u : (pick(5) == 0 ? 1u : 0u);
    // The port's range query still returns objects shut inside a transport, which the reference
    // partition does not hold, so no second tower scans for an enclosed one.
    if (container == 1) second = 0;
    const uint32_t count = i == 0 ? 4u : 1u + pick(4);
    v.p = {i == 0 ? 1u : 1u + pick(2), container, second, count};
    static const uint32_t first[4] = {0, 1, 5, 3};
    for (uint32_t k = 0; k < count; ++k) {
        const uint32_t type = i == 0 ? first[k] : (pick(2) ? 0u : pick(7));
        const uint32_t team = i == 0 ? v.p[0] : (pick(4) ? v.p[0] : pick(3));
        const uint32_t x = i == 0 ? bits(210.0f) : rand_pos_bits();
        const uint32_t y = i == 0 ? bits(190.0f + 5.0f * k) : rand_pos_bits();
        v.p.insert(v.p.end(), {type, team, x, y});
    }
    for (uint32_t k = 0; k < count; ++k) v.ops.push_back({1, k, bits(1.0f + unit_real() * 60.0f), 0, 0});
    if (i != 0 && pick(3) == 0) v.ops.push_back({7, 1, 0, 0, 0});
    v.ops.push_back({0, 1, 0, 0, 0});
    const int n = 4 + static_cast<int>(pick(9));
    for (int k = 0; k < n; ++k) {
        const uint32_t u = pick(count);
        switch (pick(20)) {
            case 0: v.ops.push_back({1, u, bits(1.0f + unit_real() * 60.0f), 0, 0}); break;
            case 1: v.ops.push_back({2, u, rand_pos_bits(), rand_pos_bits(), 0}); break;
            case 2: v.ops.push_back({3, pick(4), pick(2), 0, 0}); break;
            case 3: v.ops.push_back({4, pick(2), pick(2), 0, 0}); break;
            case 4: if (pick(2)) v.ops.push_back({5, u, 0, 0, 0}); break;
            case 5: if (pick(2)) v.ops.push_back({6, u, 0, 0, 0}); break;
            case 6: v.ops.push_back({7, pick(2), 0, 0, 0}); break;
            case 7: v.ops.push_back({8, 0, 0, 0, 0}); break;
            case 8: if (!container) v.ops.push_back({9, pick(3), 0, 0, 0}); break;
            case 9: if (pick(4) == 0) v.ops.push_back({10, 0, 0, 0, 0}); break;
            case 10: if (container) v.ops.push_back({11, pick(2), 0, 0, 0}); break;
            case 11: if (pick(6) == 0 && !container) v.ops.push_back({12, 0, 0, 0, 0}); break;
            default: v.ops.push_back({0, 1 + pick(20), 0, 0, 0}); break;
        }
        if (!v.ops.empty() && v.ops.back().code != 0 && pick(3) == 0) v.ops.push_back({0, 1 + pick(8), 0, 0, 0});
    }
}

// Sorts the saved tracker ids that follow the empty-list encoding of length `fixed`.
static std::string sorted_ids(const std::string& hex, size_t fixed) {
    std::vector<std::string> ids;
    for (size_t i = fixed; i + 8 <= hex.size(); i += 8) ids.push_back(hex.substr(i, 8));
    std::sort(ids.begin(), ids.end(), [](const std::string& a, const std::string& b) {
        return std::stoul(a, nullptr, 16) < std::stoul(b, nullptr, 16);
    });
    std::string out = hex.substr(0, fixed);
    for (const std::string& s : ids) out += s;
    return out;
}

static void run_propaganda(oracle_wp51::World& w, Vec& v) {
    const uint32_t tteam = v.p[0], container = v.p[1], second = v.p[2], count = v.p[3];
    Player* player = team_of(w, tteam)->getControllingPlayer();
    const UpgradeTemplate* sub = TheUpgradeCenter->findUpgrade("Upgrade_P53C3_Sub");
    const UpgradeTemplate* obj_up = TheUpgradeCenter->findUpgrade("Upgrade_P53C3_Obj");
    for (Int p = 0; p < 3; ++p) {
        Player* pl = ThePlayerList->getNthPlayer(p);
        if (pl->hasUpgradeComplete(sub)) pl->removeUpgrade(sub);
    }
    g_fx.clear();
    g_shock.clear();

    const std::string name = "P53C3_Prop" + std::to_string(v.variant);
    Coord3D center = {200.0f, 200.0f, 0.0f};
    Object* tower = w.spawn_object(name.c_str(), team_of(w, tteam));
    tower->setPosition(&center);
    PropagandaTowerBehavior* m = find_behavior<PropagandaTowerBehavior>(tower);
    const ObjectID tower_id = tower->getID();
    const size_t fixed = save_hex(m).size();
    PropagandaTowerBehavior* m2 = nullptr;
    if (second) {
        Object* t2 = w.spawn_object(name.c_str(), team_of(w, tteam));
        t2->setPosition(&center);
        m2 = find_behavior<PropagandaTowerBehavior>(t2);
    }
    Object* box = nullptr;
    if (container) {
        box = w.spawn_object(container == 1 ? "P53C3_Trans" : "P53C3_OpenGarr", team_of(w, tteam));
        box->setPosition(&center);
    }
    std::vector<ObjectID> unit;
    for (uint32_t k = 0; k < count; ++k) {
        const uint32_t* u = &v.p[4 + k * 4];
        Object* o = w.spawn_object(kUnit[u[0]], team_of(w, u[1]));
        Coord3D pos = {real_of(u[2]), real_of(u[3]), 0.0f};
        o->setPosition(&pos);
        unit.push_back(o->getID());
    }

    InitRandom(v.seed);
    UnsignedInt frame = v.frame;
    for (const Op& op : v.ops) {
        Object* t = TheGameLogic->findObjectByID(tower_id);
        Object* u = op.code == 1 || op.code == 2 || op.code == 5 || op.code == 6
                        ? TheGameLogic->findObjectByID(unit[op.a]) : nullptr;
        ThePartitionManager->update();
        if (op.code == 0) {
            for (uint32_t k = 0; k < op.a; ++k) {
                ++frame;
                w.set_frame(frame);
                if (TheGameLogic->findObjectByID(tower_id)) step_single_update(w, m, frame);
                if (m2) step_single_update(w, m2, frame);
            }
        }
        if (op.code == 1 && u) hurt(u, DAMAGE_UNRESISTABLE, real_of(op.b), INVALID_ID);
        if (op.code == 2 && u) {
            Coord3D pos = {real_of(op.b), real_of(op.c), 0.0f};
            u->setPosition(&pos);
        }
        if (op.code == 3 && t) {
            const ObjectStatusTypes s[4] = {OBJECT_STATUS_UNDER_CONSTRUCTION, OBJECT_STATUS_SOLD,
                                            OBJECT_STATUS_STEALTHED, OBJECT_STATUS_DETECTED};
            t->setStatus(MAKE_OBJECT_STATUS_MASK(s[op.a]), op.b != 0);
        }
        if (op.code == 4 && t) {
            const DisabledType d = op.a == 0 ? DISABLED_HELD : DISABLED_EMP;
            if (op.b) t->setDisabled(d); else t->clearDisabled(d);
        }
        if (op.code == 5 && u && !u->isEffectivelyDead()) u->kill();
        if (op.code == 6 && u) {
            TheGameLogic->destroyObject(u);
            w.flush_destroy_list();
        }
        if (op.code == 7) {
            if (op.a && !player->hasUpgradeComplete(sub)) player->addUpgrade(sub, UPGRADE_STATUS_COMPLETE);
            if (!op.a && player->hasUpgradeComplete(sub)) player->removeUpgrade(sub);
        }
        if (op.code == 8 && t) t->giveUpgrade(obj_up);
        if (op.code == 9 && t) t->setTeam(team_of(w, op.a));
        if (op.code == 10 && t && !t->isEffectivelyDead()) t->kill();
        if (op.code == 11 && t && box && !t->isEffectivelyDead()) {
            if (op.a && !t->getContainedBy()) box->getContain()->addToContain(t);
            if (!op.a && t->getContainedBy() == box) box->getContain()->removeFromContain(t);
        }
        if (op.code == 12 && t) {
            TheGameLogic->destroyObject(t);
            w.flush_destroy_list();
        }

        t = TheGameLogic->findObjectByID(tower_id);
        std::vector<uint32_t> obs;
        for (uint32_t k = 0; k < 4; ++k) {
            Object* o = k < count ? TheGameLogic->findObjectByID(unit[k]) : nullptr;
            obs.push_back(health_bits(o));
            obs.push_back(!o ? 4u
                             : (o->testWeaponBonusCondition(WEAPONBONUSCONDITION_ENTHUSIASTIC) ? 1u : 0u) |
                                   (o->testWeaponBonusCondition(WEAPONBONUSCONDITION_SUBLIMINAL) ? 2u : 0u));
        }
        obs.push_back(health_bits(t));
        obs.push_back(!t ? 4u
                         : (t->testWeaponBonusCondition(WEAPONBONUSCONDITION_ENTHUSIASTIC) ? 1u : 0u) |
                               (t->testWeaponBonusCondition(WEAPONBONUSCONDITION_SUBLIMINAL) ? 2u : 0u));
        std::string x = "-";
        uint32_t tracked = 0, next = 0;
        if (t) {
            next = m->friend_getNextCallFrame();
            const std::string raw = save_hex(m);
            tracked = static_cast<uint32_t>((raw.size() - fixed) / 8);
            x = sorted_ids(raw, fixed);
        }
        obs.push_back(next);
        obs.push_back(tracked);
        obs.push_back(static_cast<uint32_t>(g_fx.size()));
        obs.push_back(log_digest(g_fx));
        obs.push_back(GetGameLogicRandomSeedCRC());
        v.rows.push_back({x, obs});
    }
    v.crc = TheGameLogic->findObjectByID(tower_id) ? oracle_wp51::World::snapshot_crc(m) : 0u;
}

// ----------------------------------------------------------------------------- fixture

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
    for (int i = 0; i < FX_COUNT; ++i) ss << "FXList " << kFxName[i] << "\nEnd\n";
    ss << "PlayerTemplate FactionP53C3\n  Side = America\n  PlayableSide = Yes\nEnd\n";
    ss << "Upgrade Upgrade_P53C3_Bust\n  Type = PLAYER\nEnd\n"
       << "Upgrade Upgrade_P53C3_Sub\n  Type = PLAYER\nEnd\n"
       << "Upgrade Upgrade_P53C3_Obj\n  Type = OBJECT\nEnd\n";
    ss << "Weapon Wpn_P53C3_Occ0\n  PrimaryDamage = 1.0\n  DamageType = EXPLOSION\n  DeathType = EXPLODED\nEnd\n"
       << "Weapon Wpn_P53C3_Occ1\n  PrimaryDamage = 1.0\n  DamageType = FLAME\n  DeathType = BURNED\nEnd\n"
       << "Weapon Wpn_P53C3_Shock\n  PrimaryDamage = 0.0\n  AttackRange = 100.0\n"
       << "  DelayBetweenShots = 100\n  FireFX = FX_P53C3_Shock\nEnd\n"
       << "Weapon Wpn_P53C3_Rifle\n  PrimaryDamage = 10.0\n  AttackRange = 100.0\n  DamageType = SMALL_ARMS\nEnd\n"
       << "Weapon Wpn_P53C3_Zero\n  PrimaryDamage = 0.0\n  AttackRange = 100.0\n  DamageType = HACK\nEnd\n";

    // Bridge towers.
    for (int i = 0; i < 4; ++i) {
        const std::string h = std::to_string(static_cast<int>(kBridgeHealth[i])) + ".0";
        ss << "Object P53C3_Bridge" << i << "\n  KindOf = PRELOAD IMMOBILE BRIDGE\n" << body(h)
           << "  Behavior = BridgeBehavior ModuleTag_Bridge\n  End\nEnd\n";
        ss << "Object P53C3_Plain" << i << "\n  KindOf = PRELOAD IMMOBILE BRIDGE\n" << body(h)
           << "End\n";
        ss << "Object P53C3_Tower" << i << "\n  KindOf = PRELOAD IMMOBILE BRIDGE_TOWER\n"
           << body(std::to_string(static_cast<int>(kTowerHealth[i])) + ".0")
           << "  Behavior = BridgeTowerBehavior ModuleTag_Tower\n  End\nEnd\n";
    }
    ss << "Object P53C3_Source\n  KindOf = PRELOAD VEHICLE\n" << body("100.0") << "End\n";

    // Bunker buster.
    const char* occupant[3] = {nullptr, "Wpn_P53C3_Occ0", "Wpn_P53C3_Occ1"};
    for (int i = 0; i < 8; ++i) {
        const BustVariant& b = kBust[i];
        for (int ai = 0; ai < 2; ++ai) {
            ss << "Object P53C3_Bomb" << i << (ai ? "A" : "N") << "\n  KindOf = PRELOAD PROJECTILE\n"
               << body("10.0") << (ai ? kAi : "")
               << "  Behavior = BunkerBusterBehavior ModuleTag_Buster\n";
            if (b.upgrade) ss << "    UpgradeRequired = Upgrade_P53C3_Bust\n";
            if (b.occupant) ss << "    OccupantDamageWeaponTemplate = " << occupant[b.occupant] << "\n";
            if (b.shock) ss << "    ShockwaveWeaponTemplate = Wpn_P53C3_Shock\n";
            if (b.det) ss << "    DetonationFX = FX_P53C3_Det\n";
            if (b.crash) ss << "    CrashThroughBunkerFX = FX_P53C3_Crash\n";
            if (b.freq) ss << "    CrashThroughBunkerFXFrequency = " << b.freq << "\n";
            ss << "  End\nEnd\n";
        }
    }
    const char* rider_health[4] = {"50.0", "100.0", "101.0", "250.0"};
    for (int i = 0; i < 4; ++i) {
        ss << "Object P53C3_Rider" << i << "\n  KindOf = PRELOAD INFANTRY\n  TransportSlotCount = 1\n"
           << body(rider_health[i]) << "End\n";
    }
    ss << "Object P53C3_PlainBldg\n  KindOf = PRELOAD STRUCTURE\n" << body("1000.0") << "End\n";
    ss << "Object P53C3_Garr\n  KindOf = PRELOAD STRUCTURE\n" << body("1000.0")
       << "  Behavior = GarrisonContain ModuleTag_Contain\n    ContainMax = 10\n  End\nEnd\n";
    ss << "Object P53C3_OpenGarr\n  KindOf = PRELOAD STRUCTURE\n" << body("1000.0")
       << "  Behavior = GarrisonContain ModuleTag_Contain\n    ContainMax = 10\n"
       << "    IsEnclosingContainer = No\n  End\nEnd\n";
    ss << "Object P53C3_Trans\n  KindOf = PRELOAD VEHICLE\n" << body("500.0")
       << "  Behavior = TransportContain ModuleTag_Contain\n    Slots = 8\n"
       << "    AllowInsideKindOf = INFANTRY\n  End\nEnd\n";

    // Propaganda tower.
    for (int i = 0; i < 8; ++i) {
        const PropVariant& p = kProp[i];
        ss << "Object P53C3_Prop" << i << "\n  KindOf = PRELOAD SCORE " << p.kindof
           << "\n  TransportSlotCount = 1\n" << body("1000.0")
           << weapon_set("Wpn_P53C3_Rifle")
           << "  Behavior = PropagandaTowerBehavior ModuleTag_Prop\n"
           << "    Radius = " << p.radius << "\n"
           << "    DelayBetweenUpdates = " << p.delay << "\n"
           << "    HealPercentEachSecond = " << p.heal << "\n"
           << "    UpgradedHealPercentEachSecond = " << p.up_heal << "\n"
           << "    UpgradeRequired = " << (p.object_upgrade ? "Upgrade_P53C3_Obj" : "Upgrade_P53C3_Sub") << "\n"
           << "    PulseFX = FX_P53C3_Pulse\n"
           << "    UpgradedPulseFX = FX_P53C3_UpPulse\n"
           << "    AffectsSelf = " << (p.affects_self ? "Yes" : "No") << "\n  End\nEnd\n";
    }
    const char* unit_kind[7] = {"INFANTRY SCORE", "INFANTRY SCORE", "INFANTRY SCORE", "STRUCTURE SCORE",
                                "INFANTRY", "VEHICLE MP_COUNT_FOR_VICTORY", "INFANTRY SCORE_CREATE"};
    for (int i = 0; i < 7; ++i) {
        ss << "Object " << kUnit[i] << "\n  KindOf = PRELOAD " << unit_kind[i] << "\n" << body("200.0");
        if (i != 1) ss << weapon_set(i == 2 ? "Wpn_P53C3_Zero" : "Wpn_P53C3_Rifle");
        ss << "End\n";
    }
    return ss.str();
}

// ----------------------------------------------------------------------------- main

int main(int argc, char** argv) {
    if (argc < 2) { std::fprintf(stderr, "usage: gen_wp53_c3 <out.json>\n"); return 2; }
    const std::string out_path = argv[1];
    oracle_wp51::World w;
    w.load_ini(fixture_ini());
    w.init_with_neutral_player("FactionP53C3", nullptr);
    TheWritableGlobalData->m_unitDamagedThresh = 0.7f;
    TheWritableGlobalData->m_unitReallyDamagedThresh = 0.35f;
    for (int i = 0; i < FX_COUNT; ++i) {
        FXList* fx = const_cast<FXList*>(TheFXListStore->findFXList(kFxName[i]));
        if (!fx) { std::fprintf(stderr, "missing %s\n", kFxName[i]); return 1; }
        fx->addFXNugget(newInstance(OracleNugget)(static_cast<uint32_t>(i)));
    }
    // Reveal before making the player local: a local player's reveal needs a display.
    const Int local = w.team_a->getControllingPlayer()->getPlayerIndex();
    ThePartitionManager->revealMapForPlayerPermanently(local);
    ThePlayerList->setLocalPlayer(w.team_a->getControllingPlayer());

    const int kTotal = 1000;
    std::vector<Vec> vecs;
    int per_kind[3] = {0, 0, 0};
    for (int i = 0; i < kTotal; ++i) {
        Vec v;
        v.kind = static_cast<uint32_t>(i % 3);
        const int n = per_kind[v.kind]++;
        v.frame = 100 + pick(5000);
        v.seed = next_u32() & 0x7fffffff;
        switch (v.kind) {
            case 0: gen_bridge(v, n); break;
            case 1: gen_bunker(v, n); break;
            default: gen_propaganda(v, n); break;
        }
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(v.frame);
        InitRandom(v.seed);
        switch (v.kind) {
            case 0: run_bridge(w, v); break;
            case 1: run_bunker(w, v); break;
            default: run_propaganda(w, v); break;
        }
        v.rng = GetGameLogicRandomSeedCRC();
        destroy_and_flush_all(w);
        ThePartitionManager->update();
        vecs.push_back(std::move(v));
    }

    const char* root = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/";
    const char* sources[3] = {
        "Behavior/BridgeTowerBehavior.cpp", "Behavior/BunkerBusterBehavior.cpp",
        "Behavior/PropagandaTowerBehavior.cpp"};

    const std::string tmp = out_path + ".tmp";
    FILE* f = std::fopen(tmp.c_str(), "wb");
    if (!f) { std::perror("open"); return 1; }
    std::fprintf(f, "{\n\"packet\": \"WP53_C3\",\n\"vector_count\": %d,\n\"local_player\": %d,\n\"source_sha256\": {",
                 kTotal, local);
    for (int k = 0; k < 3; ++k) {
        const std::string h = oracle::sha256_file((std::string(root) + sources[k]).c_str());
        std::fprintf(f, "%s\"%s\":\"%s\"", k ? "," : "", kKindName[k], h.c_str());
    }
    std::fprintf(f, "},\n\"seed0_diagnostics\": {");
    for (int k = 0; k < 3; ++k) {
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
