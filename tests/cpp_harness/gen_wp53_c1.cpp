// WP-5.3-C1 leaf modules: PKT-261 BridgeScaffoldBehavior, PKT-283 OverchargeBehavior,
// PKT-287 PoisonedBehavior, PKT-292 RailedTransportContain,
// PKT-297 SupplyWarehouseCripplingBehavior, PKT-298 TechBuildingBehavior.
//
// The reference library is built with RETAIL_COMPATIBLE_XFER_SAVE=1, so PoisonedBehavior saves
// version 2, without `m_poisonSource`. The source is recorded as its own column instead.
//
// Canonical 3-mode build & run:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp53_c1.cpp --opt O0 -o <dir>/gen_wp53_c1_o0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp53_c1.cpp --opt O2 -o <dir>/gen_wp53_c1_o2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp53_c1.cpp --opt O2 --sanitize -o <dir>/gen_wp53_c1_ubsan
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy <dir>/gen_wp53_c1_o2 docs/golden_vectors/WP53_C1.json
//
// Each vector is a short script of operations on one object. After every operation a row
// records an FNV-1a digest of the module's XferSave bytes followed by the observable world
// state. `final_xfer` carries the last bytes in full when they are at most 256 bytes, and
// `seed0_diagnostics` carries the first row's bytes of each module in full.

#include "oracle_wp51_world.h"

#include "Common/Energy.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/RandomValue.h"
#include "Common/Team.h"
#include "Common/ThingTemplate.h"
#include "GameClient/Drawable.h"
#include "GameLogic/Damage.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/BridgeScaffoldBehavior.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/DockUpdate.h"
#include "GameLogic/Module/OverchargeBehavior.h"
#include "GameLogic/Module/PoisonedBehavior.h"
#include "GameLogic/Module/PowerPlantUpdate.h"
#include "GameLogic/Module/RailedTransportContain.h"
#include "GameLogic/Module/SupplyWarehouseCripplingBehavior.h"
#include "GameLogic/Module/TechBuildingBehavior.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Object.h"

#include <cstdio>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

struct PpExtendedTag { using type = Bool PowerPlantUpdate::*; };
template struct MemberAccess<PpExtendedTag, &PowerPlantUpdate::m_extended>;
Bool PowerPlantUpdate::* access_member(PpExtendedTag);

struct DockCrippledTag { using type = Bool DockUpdate::*; };
template struct MemberAccess<DockCrippledTag, &DockUpdate::m_dockCrippled>;
Bool DockUpdate::* access_member(DockCrippledTag);

struct PoisonSourceTag { using type = ObjectID PoisonedBehavior::*; };
template struct MemberAccess<PoisonSourceTag, &PoisonedBehavior::m_poisonSource>;
ObjectID PoisonedBehavior::* access_member(PoisonSourceTag);

struct OcLoadTag { using type = void (OverchargeBehavior::*)(); };
template struct MemberAccess<OcLoadTag, &OverchargeBehavior::loadPostProcess>;
void (OverchargeBehavior::* access_member(OcLoadTag))();

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
static uint32_t g_state = 0x5C31A7D9u;
static uint32_t next_u32() {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state;
}
static uint32_t pick(uint32_t n) { return next_u32() % n; }
static uint32_t bits(Real v) { uint32_t b; std::memcpy(&b, &v, 4); return b; }
static Real real_of(uint32_t b) { Real v; std::memcpy(&v, &b, 4); return v; }

struct Op { uint32_t code, a, b, c, d; };

static void hurt(Object* o, DamageType type, Real amount, ObjectID source, DeathType death) {
    DamageInfo d;
    d.in.m_damageType = type;
    d.in.m_deathType = death;
    d.in.m_amount = amount;
    d.in.m_sourceID = source;
    o->attemptDamage(&d);
}

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

static const char* kFields[6][9] = {
    {"px", "py", "pz", "destroyed", "motion"},
    {"health", "prod_a", "prod_b", "extended", "destroyed", "active"},
    {"health", "dead", "source"},
    {"count", "dock_open", "door", "alive_mask", "contained_mask", "destroyed"},
    {"health", "crippled", "damage_state"},
    {"captured", "team", "health"},
};
static const int kFieldCount[6] = {5, 6, 3, 6, 3, 3};

static const char* kKindName[6] = {
    "scaffold", "overcharge", "poisoned", "railed", "supply", "tech"};

// ----------------------------------------------------------------------------- inputs

static void gen_scaffold(Vec& v, int i) {
    v.variant = 0;
    auto coord = [&]() { return bits(static_cast<Real>(static_cast<int>(pick(161)) - 80) * 0.25f); };
    for (int k = 0; k < 12; ++k) v.p.push_back(coord());
    // create, rise, build, start (usually the creation point)
    if (pick(4) != 0) { v.p[9] = v.p[0]; v.p[10] = v.p[1]; v.p[11] = v.p[2]; }
    auto speed = [&]() {
        switch (pick(8)) {
            case 0: return bits(0.0f);
            case 1: return bits(0.0005f);
            case 2: return bits(-1.5f);
            case 3: return bits(100.0f);
            default: return bits(static_cast<Real>(1 + pick(32)) * 0.3125f);
        }
    };
    // Zero-length leg: the build point equals the rise point, so the speed ratio divides by
    // zero and clamps to top speed. A fixed script with positive speeds keeps 0/0 and 0*inf out:
    // those give NaN positions, on which the reference destroys the object outside this module.
    if (i != 0 && pick(12) == 0) {
        v.p[5] = bits(30.0f);  // above every creation point, so the sink leg has length
        v.p[6] = v.p[3]; v.p[7] = v.p[4]; v.p[8] = v.p[5];
        v.p[9] = v.p[0]; v.p[10] = v.p[1]; v.p[11] = v.p[2];
        v.p.push_back(bits(static_cast<Real>(1 + pick(32)) * 0.3125f));
        v.p.push_back(bits(static_cast<Real>(1 + pick(32)) * 0.3125f));
        v.ops.push_back({0, 2, 0, 0, 0});
        v.ops.push_back({2, 1, 0, 0, 0});
        v.ops.push_back({2, 400, 0, 0, 0});
        v.ops.push_back({0, 4, 0, 0, 0});
        v.ops.push_back({2, 200, 0, 0, 0});
        return;
    }
    v.p.push_back(speed());
    v.p.push_back(speed());
    v.ops.push_back({0, i == 0 ? 1u : pick(5), 0, 0, 0});
    const int n = 4 + static_cast<int>(pick(6));
    for (int k = 0; k < n; ++k) {
        switch (pick(8)) {
            case 0: v.ops.push_back({0, pick(5), 0, 0, 0}); break;
            case 1: v.ops.push_back({1, 0, 0, 0, 0}); break;
            case 2: v.ops.push_back({3, speed(), 0, 0, 0}); break;
            case 3: v.ops.push_back({4, speed(), 0, 0, 0}); break;
            default: v.ops.push_back({2, 1 + pick(60), 0, 0, 0}); break;
        }
    }
}

static void gen_overcharge(Vec& v, int i) {
    v.variant = pick(4);
    const int n = 5 + static_cast<int>(pick(8));
    if (i == 0 || pick(3) != 0) v.ops.push_back({0, 0, 0, 0, 0});
    for (int k = 0; k < n; ++k) {
        switch (pick(12)) {
            case 0: v.ops.push_back({0, 0, 0, 0, 0}); break;
            case 1: v.ops.push_back({1, pick(2), 0, 0, 0}); break;
            case 2: v.ops.push_back({3, 0, 0, 0, 0}); break;
            case 3: v.ops.push_back({4, 0, 0, 0, 0}); break;
            case 4: v.ops.push_back({5, 0, 0, 0, 0}); break;
            case 5: v.ops.push_back({6, bits(static_cast<Real>(pick(600))), 0, 0, 0}); break;
            case 6: v.ops.push_back({7, bits(static_cast<Real>(pick(600))), 0, 0, 0}); break;
            case 7: v.ops.push_back({8, 0, 0, 0, 0}); break;
            default: v.ops.push_back({2, 1 + pick(90), 0, 0, 0}); break;
        }
    }
    if (pick(2) == 0) v.ops.push_back({9, 0, 0, 0, 0});
}

static void gen_poisoned(Vec& v, int i) {
    v.variant = pick(5);
    const DeathType deaths[4] = {DEATH_NORMAL, DEATH_POISONED, DEATH_BURNED, DEATH_POISONED_BETA};
    auto dose = [&](bool poison) {
        return Op{0, poison ? 1u : 0u,
                  bits(static_cast<Real>(pick(12) == 0 ? 400 : pick(30)) * 0.75f),
                  4000 + pick(50), static_cast<uint32_t>(deaths[pick(4)])};
    };
    if (i == 0 || pick(6) != 0) v.ops.push_back(dose(true));
    const int n = 4 + static_cast<int>(pick(8));
    for (int k = 0; k < n; ++k) {
        switch (pick(12)) {
            case 0: v.ops.push_back(dose(true)); break;
            case 1: v.ops.push_back(dose(pick(2) == 0)); break;
            case 2: v.ops.push_back({1, bits(static_cast<Real>(pick(20))), 0, 0, 0}); break;
            case 3: v.ops.push_back({3, 1 + pick(200), 0, 0, 0}); break;
            case 4: v.ops.push_back({4, 0, 0, 0, 0}); break;
            case 5: v.ops.push_back({5, 0, 0, 0, 0}); break;
            default: v.ops.push_back({2, 1 + pick(40), 0, 0, 0}); break;
        }
    }
}

static void gen_railed(Vec& v, int i) {
    v.variant = i == 0 ? 0u : pick(8);  // bit0 big, bit1 destroy riders, bit2 no dock
    const int n = 5 + static_cast<int>(pick(8));
    v.ops.push_back({0, 0, 0, 0, 0});
    for (int k = 0; k < n; ++k) {
        switch (pick(11)) {
            case 0: case 1: case 2: v.ops.push_back({0, 0, 0, 0, 0}); break;
            case 3: case 4: v.ops.push_back({1, pick(4), 0, 0, 0}); break;
            case 5: case 6: v.ops.push_back({2, pick(3) == 0 ? 1u : 0u, 0, 0, 0}); break;
            case 7: v.ops.push_back({3, pick(4), 0, 0, 0}); break;
            case 8: v.ops.push_back({4, pick(4), 0, 0, 0}); break;
            default: v.ops.push_back({5, 1 + pick(20), 0, 0, 0}); break;
        }
    }
    if (pick(3) != 0) {
        v.ops.push_back({0, 0, 0, 0, 0});
        v.ops.push_back({2, pick(2), 0, 0, 0});
        v.ops.push_back({3, 0, 0, 0, 0});
        v.ops.push_back({6, 0, 0, 0, 0});
    }
}

static void gen_supply(Vec& v, int i) {
    v.variant = pick(4);
    const int n = 5 + static_cast<int>(pick(8));
    v.ops.push_back({0, bits(i == 0 ? 950.0f : static_cast<Real>(pick(990))), 0, 0, 0});
    for (int k = 0; k < n; ++k) {
        switch (pick(9)) {
            case 0: case 1: v.ops.push_back({0, bits(static_cast<Real>(pick(5) == 0 ? 0 : pick(400))), 0, 0, 0}); break;
            case 2: v.ops.push_back({2, bits(static_cast<Real>(pick(600))), 0, 0, 0}); break;
            case 3: v.ops.push_back({3, 1 + pick(300), 0, 0, 0}); break;
            default: v.ops.push_back({1, 1 + pick(200), 0, 0, 0}); break;
        }
    }
}

static void gen_tech(Vec& v, int i) {
    v.variant = pick(4);
    v.p.push_back(i == 0 ? 1u : pick(3));  // starting team: 0 neutral, 1 playable, 2 not playable
    const int n = 4 + static_cast<int>(pick(7));
    v.ops.push_back({0, 1 + pick(3), 0, 0, 0});
    for (int k = 0; k < n; ++k) {
        switch (pick(9)) {
            case 0: case 1: case 2: v.ops.push_back({1, pick(3), 0, 0, 0}); break;
            case 3: v.ops.push_back({2, 0, 0, 0, 0}); break;
            default: v.ops.push_back({0, 1 + pick(80), 0, 0, 0}); break;
        }
    }
}

// ----------------------------------------------------------------------------- execution

static Team* team_of(oracle_wp51::World& w, uint32_t idx) {
    return idx == 0 ? w.team_neutral : (idx == 1 ? w.team_a : w.team_b);
}
static uint32_t team_index(oracle_wp51::World& w, Object* o) {
    return o->getTeam() == w.team_neutral ? 0u : (o->getTeam() == w.team_a ? 1u : 2u);
}
static uint32_t health_bits(Object* o) { return bits(o->getBodyModule()->getHealth()); }

static void run_scaffold(oracle_wp51::World& w, Vec& v) {
    Object* o = w.spawn_object("P53C_Scaffold", w.team_a);
    Coord3D c[4];
    for (int k = 0; k < 4; ++k) {
        c[k].x = real_of(v.p[k * 3]);
        c[k].y = real_of(v.p[k * 3 + 1]);
        c[k].z = real_of(v.p[k * 3 + 2]);
    }
    o->setPosition(&c[3]);
    BridgeScaffoldBehavior* m = find_behavior<BridgeScaffoldBehavior>(o);
    BridgeScaffoldBehaviorInterface* itf = m;
    itf->setPositions(&c[0], &c[1], &c[2]);
    itf->setLateralSpeed(real_of(v.p[12]));
    itf->setVerticalSpeed(real_of(v.p[13]));
    UnsignedInt frame = v.frame;
    for (const Op& op : v.ops) {
        if (op.code == 0) itf->setMotion(static_cast<ScaffoldTargetMotion>(op.a));
        if (op.code == 1) itf->reverseMotion();
        if (op.code == 2) for (uint32_t k = 0; k < op.a; ++k) step_single_update(w, m, ++frame);
        if (op.code == 3) itf->setLateralSpeed(real_of(op.a));
        if (op.code == 4) itf->setVerticalSpeed(real_of(op.a));
        const Coord3D* p = o->getPosition();
        v.rows.push_back({save_hex(m),
                          {bits(p->x), bits(p->y), bits(p->z), o->isDestroyed() ? 1u : 0u,
                           static_cast<uint32_t>(itf->getCurrentMotion())}});
    }
    v.crc = oracle_wp51::World::snapshot_crc(m);
}

static void run_overcharge(oracle_wp51::World& w, Vec& v) {
    const std::string name = "P53C_Over" + std::to_string(v.variant);
    const Int base_a = w.team_a->getControllingPlayer()->getEnergy()->getProduction();
    const Int base_b = w.team_b->getControllingPlayer()->getEnergy()->getProduction();
    Object* o = w.spawn_object(name.c_str(), w.team_a);
    OverchargeBehavior* m = find_behavior<OverchargeBehavior>(o);
    OverchargeBehaviorInterface* itf = m;
    PowerPlantUpdate* pp = find_behavior<PowerPlantUpdate>(o);
    UnsignedInt frame = v.frame;
    std::string last = save_hex(m);
    v.crc = oracle_wp51::World::snapshot_crc(m);
    bool gone = false;
    uint32_t health = 0, extended = 0, active = 0;
    for (const Op& op : v.ops) {
        if (!gone) {
            if (op.code == 0) itf->toggle();
            if (op.code == 1) itf->enable(op.a ? TRUE : FALSE);
            if (op.code == 2) for (uint32_t k = 0; k < op.a; ++k) step_single_update(w, m, ++frame);
            if (op.code == 3) o->setDisabled(DISABLED_EMP);
            if (op.code == 4) o->clearDisabled(DISABLED_EMP);
            if (op.code == 5) o->setTeam(o->getTeam() == w.team_a ? w.team_b : w.team_a);
            if (op.code == 6) hurt(o, DAMAGE_EXPLOSION, real_of(op.a), INVALID_ID, DEATH_NORMAL);
            if (op.code == 7) o->attemptHealing(real_of(op.a), nullptr);
            if (op.code == 8) (m->*access_member(OcLoadTag()))();
            last = save_hex(m);
            health = health_bits(o);
            extended = (pp->*access_member(PpExtendedTag())) ? 1u : 0u;
            active = itf->isOverchargeActive() ? 1u : 0u;
            if (op.code == 9) {
                // Object deletion runs `onDelete`, which returns the bonus.
                TheGameLogic->destroyObject(o);
                w.flush_destroy_list();
                gone = true;
            }
        }
        v.rows.push_back(
            {gone ? std::string("-") : last,
             {gone ? 0u : health,
              static_cast<uint32_t>(w.team_a->getControllingPlayer()->getEnergy()->getProduction() - base_a),
              static_cast<uint32_t>(w.team_b->getControllingPlayer()->getEnergy()->getProduction() - base_b),
              gone ? 0u : extended, gone ? 1u : 0u, gone ? 0u : active}});
    }
}

static void run_poisoned(oracle_wp51::World& w, Vec& v) {
    const std::string name = "P53C_Poison" + std::to_string(v.variant);
    Object* o = w.spawn_object(name.c_str(), w.team_a);
    PoisonedBehavior* m = find_behavior<PoisonedBehavior>(o);
    UnsignedInt frame = v.frame;
    for (const Op& op : v.ops) {
        if (op.code == 0) {
            hurt(o, op.a ? DAMAGE_POISON : DAMAGE_EXPLOSION, real_of(op.b),
                 static_cast<ObjectID>(op.c), static_cast<DeathType>(op.d));
        }
        if (op.code == 1) o->attemptHealing(real_of(op.a), nullptr);
        if (op.code == 2) for (uint32_t k = 0; k < op.a; ++k) step_single_update(w, m, ++frame);
        if (op.code == 3) { frame += op.a; step_single_update(w, m, frame); }
        if (op.code == 4) o->setDisabled(DISABLED_EMP);
        if (op.code == 5) o->clearDisabled(DISABLED_EMP);
        v.rows.push_back({save_hex(m),
                          {health_bits(o), o->isEffectivelyDead() ? 1u : 0u,
                           static_cast<uint32_t>(m->*access_member(PoisonSourceTag()))}});
    }
    v.crc = oracle_wp51::World::snapshot_crc(m);
}

static void run_railed(oracle_wp51::World& w, Vec& v) {
    const std::string name = "P53C_Rail" + std::to_string(v.variant);
    Object* o = w.spawn_object(name.c_str(), w.team_a);
    RailedTransportContain* m = find_behavior<RailedTransportContain>(o);
    DockUpdateInterface* dock = o->getDockUpdateInterface();
    std::vector<Object*> riders;
    UnsignedInt frame = v.frame;
    for (const Op& op : v.ops) {
        uint32_t door = 99;
        if (op.code == 0 && riders.size() < 24) {
            Object* r = w.spawn_object("P53C_Rider", w.team_a);
            riders.push_back(r);
            if (m->isValidContainerFor(r, TRUE)) m->addToContain(r);
        }
        if ((op.code == 1 || op.code == 3 || op.code == 4) && !riders.empty()) {
            Object* r = riders[op.a % riders.size()];
            if (op.code == 1 && r->getContainedBy() == o) m->removeFromContain(r);
            if (op.code == 3) door = static_cast<uint32_t>(m->reserveDoorForExit(r->getTemplate(), r));
            if (op.code == 4 && r->getContainedBy() == o) m->exitObjectViaDoor(r, DOOR_1);
        }
        if (op.code == 2 && dock) dock->setDockOpen(op.a ? TRUE : FALSE);
        if (op.code == 5) for (uint32_t k = 0; k < op.a; ++k) step_single_update(w, m, ++frame);
        if (op.code == 6) hurt(o, DAMAGE_UNRESISTABLE, 100000.0f, INVALID_ID, DEATH_NORMAL);
        uint32_t alive = 0, inside = 0;
        for (size_t k = 0; k < riders.size(); ++k) {
            if (!riders[k]->isDestroyed() && !riders[k]->isEffectivelyDead()) alive |= 1u << k;
            if (riders[k]->getContainedBy() == o) inside |= 1u << k;
        }
        v.rows.push_back({save_hex(m),
                          {m->getContainCount(), (!dock || dock->isDockOpen()) ? 1u : 0u, door,
                           alive, inside, o->isDestroyed() ? 1u : 0u}});
    }
    v.crc = oracle_wp51::World::snapshot_crc(m);
}

static void run_supply(oracle_wp51::World& w, Vec& v) {
    const std::string name = "P53C_Supply" + std::to_string(v.variant);
    Object* o = w.spawn_object(name.c_str(), w.team_a);
    SupplyWarehouseCripplingBehavior* m = find_behavior<SupplyWarehouseCripplingBehavior>(o);
    DockUpdate* dock = find_behavior<DockUpdate>(o);
    UnsignedInt frame = v.frame;
    for (const Op& op : v.ops) {
        if (op.code == 0) hurt(o, DAMAGE_EXPLOSION, real_of(op.a), INVALID_ID, DEATH_NORMAL);
        if (op.code == 1) for (uint32_t k = 0; k < op.a; ++k) step_single_update(w, m, ++frame);
        if (op.code == 2) o->attemptHealing(real_of(op.a), nullptr);
        if (op.code == 3) { frame += op.a; step_single_update(w, m, frame); }
        v.rows.push_back({save_hex(m),
                          {health_bits(o), (dock->*access_member(DockCrippledTag())) ? 1u : 0u,
                           static_cast<uint32_t>(o->getBodyModule()->getDamageState())}});
    }
    v.crc = oracle_wp51::World::snapshot_crc(m);
}

static void run_tech(oracle_wp51::World& w, Vec& v) {
    const std::string name = "P53C_Tech" + std::to_string(v.variant);
    Object* o = w.spawn_object(name.c_str(), team_of(w, v.p[0]));
    TechBuildingBehavior* m = find_behavior<TechBuildingBehavior>(o);
    UnsignedInt frame = v.frame;
    for (const Op& op : v.ops) {
        if (op.code == 0) for (uint32_t k = 0; k < op.a; ++k) step_single_update(w, m, ++frame);
        if (op.code == 1) { w.set_frame(frame); o->setTeam(team_of(w, op.a)); }
        if (op.code == 2) { w.set_frame(frame); hurt(o, DAMAGE_UNRESISTABLE, 100000.0f, INVALID_ID, DEATH_NORMAL); }
        Drawable* d = o->getDrawable();
        if (!d) { std::fprintf(stderr, "tech building has no drawable\n"); std::exit(1); }
        v.rows.push_back({save_hex(m),
                          {d->getModelConditionFlags().test(MODELCONDITION_CAPTURED) ? 1u : 0u,
                           team_index(w, o), health_bits(o)}});
    }
    v.crc = oracle_wp51::World::snapshot_crc(m);
}

// ----------------------------------------------------------------------------- templates

static std::string body(const char* health) {
    return std::string("  Body = ActiveBody ModuleTag_Body\n    MaxHealth = ") + health +
           "\n    InitialHealth = " + health + "\n  End\n";
}

static std::string fixture_ini() {
    std::ostringstream ss;
    ss << "FXList FX_P53C_Pulse\nEnd\n"
       << "PlayerTemplate FactionP53C\n  Side = America\n  PlayableSide = Yes\nEnd\n";
    ss << "Object P53C_Scaffold\n  KindOf = PRELOAD IMMOBILE\n" << body("100.0")
       << "  Behavior = BridgeScaffoldBehavior ModuleTag_Scaffold\n  End\nEnd\n";

    const char* drain[4] = {"2%", "10%", "100%", "25%"};
    const char* floor_pct[4] = {"50%", "25%", "0%", "90%"};
    const int bonus[4] = {50, 7, 3, 100};
    for (int i = 0; i < 4; ++i) {
        ss << "Object P53C_Over" << i << "\n  KindOf = PRELOAD STRUCTURE\n"
           << "  EnergyProduction = 0\n  EnergyBonus = " << bonus[i] << "\n" << body("1000.0")
           << "  Behavior = PowerPlantUpdate ModuleTag_Rods\n    RodsExtendTime = 600\n  End\n"
           << "  Behavior = OverchargeBehavior ModuleTag_Over\n"
           << "    HealthPercentToDrainPerSecond = " << drain[i] << "\n"
           << "    NotAllowedWhenHealthBelowPercent = " << floor_pct[i] << "\n  End\nEnd\n";
    }

    const int interval[5] = {333, 1000, 100, 0, 500};
    const int duration[5] = {3000, 500, 2000, 1000, 0};
    for (int i = 0; i < 5; ++i) {
        ss << "Object P53C_Poison" << i << "\n  KindOf = PRELOAD INFANTRY\n" << body("200.0")
           << "  Behavior = PoisonedBehavior ModuleTag_Poison\n"
           << "    PoisonDamageInterval = " << interval[i] << "\n"
           << "    PoisonDuration = " << duration[i] << "\n  End\nEnd\n";
    }

    ss << "Object P53C_Rider\n  KindOf = PRELOAD INFANTRY\n  TransportSlotCount = 1\n"
       << body("50.0") << "End\n";
    for (int i = 0; i < 8; ++i) {
        ss << "Object P53C_Rail" << i << "\n  KindOf = PRELOAD VEHICLE\n" << body("300.0")
           << "  Behavior = RailedTransportContain ModuleTag_Contain\n"
           << "    Slots = " << ((i & 1) ? 6 : 2) << "\n"
           << "    AllowInsideKindOf = INFANTRY\n"
           << "    ExitDelay = 100\n"
           << "    DestroyRidersWhoAreNotFreeToExit = " << ((i & 2) ? "Yes" : "No") << "\n  End\n";
        if (!(i & 4)) ss << "  Behavior = RepairDockUpdate ModuleTag_Dock\n  End\n";
        ss << "End\n";
    }

    const int suppression[4] = {1000, 3000, 33, 500};
    const int delay[4] = {500, 100, 33, 1000};
    const char* amount[4] = {"50.0", "7.5", "2000.0", "0.0"};
    for (int i = 0; i < 4; ++i) {
        ss << "Object P53C_Supply" << i << "\n  KindOf = PRELOAD STRUCTURE\n" << body("1000.0")
           << "  Behavior = RepairDockUpdate ModuleTag_Dock\n  End\n"
           << "  Behavior = SupplyWarehouseCripplingBehavior ModuleTag_Cripple\n"
           << "    SelfHealSupression = " << suppression[i] << "\n"
           << "    SelfHealDelay = " << delay[i] << "\n"
           << "    SelfHealAmount = " << amount[i] << "\n  End\nEnd\n";
    }

    const char* tech_fields[4] = {
        "",
        "    PulseFX = FX_P53C_Pulse\n    PulseFXRate = 1000\n",
        "    PulseFX = FX_P53C_Pulse\n",
        "    PulseFXRate = 500\n",
    };
    for (int i = 0; i < 4; ++i) {
        ss << "Object P53C_Tech" << i << "\n  KindOf = PRELOAD STRUCTURE\n" << body("500.0")
           << "  Behavior = TechBuildingBehavior ModuleTag_Tech\n" << tech_fields[i]
           << "  End\nEnd\n";
    }
    return ss.str();
}

// ----------------------------------------------------------------------------- main

int main(int argc, char** argv) {
    if (argc < 2) { std::fprintf(stderr, "usage: gen_wp53_c1 <out.json>\n"); return 2; }
    const std::string out_path = argv[1];
    oracle_wp51::World w;
    w.load_ini(fixture_ini());
    w.init_with_neutral_player("FactionP53C", nullptr);
    TheWritableGlobalData->m_unitDamagedThresh = 0.7f;
    TheWritableGlobalData->m_unitReallyDamagedThresh = 0.35f;
    if (!w.team_a->getControllingPlayer()->isPlayableSide() ||
        w.team_b->getControllingPlayer()->isPlayableSide() ||
        w.team_neutral->getControllingPlayer()->isPlayableSide()) {
        std::fprintf(stderr, "unexpected playable sides\n");
        return 1;
    }

    const int kTotal = 1000;
    std::vector<Vec> vecs;
    int per_kind[6] = {0, 0, 0, 0, 0, 0};
    for (int i = 0; i < kTotal; ++i) {
        Vec v;
        v.kind = static_cast<uint32_t>(i % 6);
        const int n = per_kind[v.kind]++;
        v.frame = 100 + pick(5000);
        v.seed = next_u32() & 0x7fffffff;
        switch (v.kind) {
            case 0: gen_scaffold(v, n); break;
            case 1: gen_overcharge(v, n); break;
            case 2: gen_poisoned(v, n); break;
            case 3: gen_railed(v, n); break;
            case 4: gen_supply(v, n); break;
            default: gen_tech(v, n); break;
        }
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(v.frame);
        InitRandom(v.seed);
        switch (v.kind) {
            case 0: run_scaffold(w, v); break;
            case 1: run_overcharge(w, v); break;
            case 2: run_poisoned(w, v); break;
            case 3: run_railed(w, v); break;
            case 4: run_supply(w, v); break;
            default: run_tech(w, v); break;
        }
        v.rng = GetGameLogicRandomSeedCRC();
        destroy_and_flush_all(w);
        vecs.push_back(std::move(v));
    }

    const char* root = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/";
    const char* sources[6] = {
        "Behavior/BridgeScaffoldBehavior.cpp", "Behavior/OverchargeBehavior.cpp",
        "Behavior/PoisonedBehavior.cpp", "Contain/RailedTransportContain.cpp",
        "Behavior/SupplyWarehouseCripplingBehavior.cpp", "Behavior/TechBuildingBehavior.cpp"};

    const std::string tmp = out_path + ".tmp";
    FILE* f = std::fopen(tmp.c_str(), "wb");
    if (!f) { std::perror("open"); return 1; }
    std::fprintf(f, "{\n\"packet\": \"WP53_C1\",\n\"vector_count\": %d,\n\"source_sha256\": {", kTotal);
    for (int k = 0; k < 6; ++k) {
        const std::string h = oracle::sha256_file((std::string(root) + sources[k]).c_str());
        std::fprintf(f, "%s\"%s\":\"%s\"", k ? "," : "", kKindName[k], h.c_str());
    }
    std::fprintf(f, "},\n\"seed0_diagnostics\": {");
    for (int k = 0; k < 6; ++k) {
        const Vec& v = vecs[k];
        const Row& last = v.rows.back();
        std::fprintf(f, "%s\"%s_rows\":%zu,\"%s_xfer\":\"%s\"", k ? "," : "", kKindName[k],
                     v.rows.size(), kKindName[k], v.rows.front().xfer.c_str());
        for (int c = 0; c < kFieldCount[k]; ++c) {
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
            for (uint32_t x : v.rows[k].obs) std::fprintf(f, ",%u", x);
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
