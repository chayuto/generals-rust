// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for DockUpdate (PKT-320), SupplyWarehouseDockUpdate (PKT-381),
// SupplyCenterDockUpdate (PKT-378) and RepairDockUpdate (PKT-363), run on the
// unmodified reference modules in a small real world with the real supply truck
// AIs as dockers.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt320_381_378_363_docks.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/PKT320_381_378_363_Docks.json
//
// The AI that drives a docker is not run: the harness calls the dock methods itself.
// Models are not loaded, so no dock has bones; op 22 writes the members a model
// (or a loaded save) would have filled, and the member it would take them from.
//
// Not covered: objects without a drawable; dock bones read from a model; more than
// ten fixed approach positions (the reference reads past an array) or a count below
// -1 (it crashes); approach indices past the end; a warehouse action by a docker
// without an AI and crippling a warehouse whose active docker is on its way in and
// has no AI (both crash); loadPostProcess and loading a save; the AI side of
// aiIdle beyond its PRNG draw; client calls (supply status art, floating text);
// pathfinding failures (the map is clear); stealth grants to disguising units.

#include "oracle_wp51_world.h"

#include "Common/Money.h"
#include "Common/Player.h"
#include "Common/RandomValue.h"
#include "Common/ScoreKeeper.h"
#include "Common/Team.h"
#include "Common/Upgrade.h"
#include "GameClient/Drawable.h"
#include "GameLogic/Damage.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/RepairDockUpdate.h"
#include "GameLogic/Module/StealthUpdate.h"
#include "GameLogic/Module/SupplyCenterDockUpdate.h"
#include "GameLogic/Module/SupplyTruckAIUpdate.h"
#include "GameLogic/Module/SupplyWarehouseDockUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"

#include <algorithm>
#include <cstdio>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

// Protected members, reached through a derived class's member pointers.
struct Peek : public DockUpdate {
    using DockUpdate::m_activeDocker;
    using DockUpdate::m_approachPositionOwners;
    using DockUpdate::m_approachPositions;
    using DockUpdate::m_dockPosition;
    using DockUpdate::m_dockerInside;
    using DockUpdate::m_enterPosition;
    using DockUpdate::m_exitPosition;
    using DockUpdate::m_numberApproachPositionBones;
    using DockUpdate::m_numberApproachPositions;
    using DockUpdate::m_positionsLoaded;
    using DockUpdate::loadDockPositions;
};

static uint32_t g_state = 0x32038137u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}

static uint32_t bits(Real f) {
    uint32_t u;
    std::memcpy(&u, &f, 4);
    return u;
}

template <typename T>
static T* find_behavior(Object* obj) {
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m) {
        if (T* typed = dynamic_cast<T*>(*m)) return typed;
    }
    return nullptr;
}

// ---------------------------------------------------------------------------
// Fixture. The Rust test builds the same text.
// ---------------------------------------------------------------------------

struct DockType {
    const char* kind_of;
    const char* geometry;  // type, major, minor, height
    const char* approach;  // nullptr leaves the field out
    const char* extra;     // module fields
};

#define CYL "CYLINDER 30 30 20"
#define BOX "BOX 40 24 20"
#define SPH "SPHERE 25 25 25"

static const DockType kWarehouses[] = {
    {"STRUCTURE SUPPLY_SOURCE", CYL, "3", "    StartingBoxes = 3\n"},
    {"STRUCTURE SUPPLY_SOURCE", BOX, "-1", "    StartingBoxes = 20\n    DeleteWhenEmpty = No\n"},
    {"STRUCTURE", SPH, nullptr, ""},
    {"STRUCTURE SUPPLY_SOURCE", CYL, "1", "    StartingBoxes = 2\n    DeleteWhenEmpty = Yes\n    AllowsPassthrough = No\n"},
    {"STRUCTURE IGNORE_DOCKING_BONES", BOX, "2", "    StartingBoxes = 0\n"},
    {"STRUCTURE SUPPLY_SOURCE", CYL, "10", "    StartingBoxes = 40\n"},
    {"STRUCTURE", "CYLINDER 220 220 20", "2", "    StartingBoxes = 5\n"},
};
static const DockType kCenters[] = {
    {"STRUCTURE", BOX, "-1", "    GrantTemporaryStealth = 3000\n"},
    {"STRUCTURE", CYL, "3", ""},
    {"STRUCTURE", SPH, nullptr, "    AllowsPassthrough = No\n"},
    {"STRUCTURE SUPPLY_SOURCE", CYL, "2", "    GrantTemporaryStealth = 100\n"},
};
static const DockType kRepairs[] = {
    {"STRUCTURE", CYL, nullptr, "    TimeForFullHeal = 1000\n"},
    {"STRUCTURE", BOX, "1", ""},
    {"STRUCTURE", CYL, "5", "    TimeForFullHeal = 5000\n"},
    {"STRUCTURE", SPH, "-1", "    TimeForFullHeal = 333\n"},
    {"STRUCTURE SUPPLY_SOURCE", BOX, "3", "    TimeForFullHeal = 0\n    AllowsPassthrough = No\n"},
    {"STRUCTURE IGNORE_DOCKING_BONES", CYL, "-1", "    TimeForFullHeal = 100\n"},
};
static const int kVariants[3] = {7, 4, 6};
static const char* const kDockModule[3] = {"SupplyWarehouseDockUpdate", "SupplyCenterDockUpdate",
                                           "RepairDockUpdate"};
static const DockType* const kDockTypes[3] = {kWarehouses, kCenters, kRepairs};

// The types from D_COUNT on appear in scripted vectors only.
enum { D_TRUCK, D_CHINOOK, D_WORKER, D_STEALTH, D_PLAIN, D_TANK, D_COUNT,
       D_ONE_BOX = D_COUNT, D_NO_BOX };

static std::string geometry(const char* spec) {
    std::istringstream in(spec);
    std::string type, major, minor, height;
    in >> type >> major >> minor >> height;
    return "  Geometry = " + type + "\n  GeometryMajorRadius = " + major +
           "\n  GeometryMinorRadius = " + minor + "\n  GeometryHeight = " + height +
           "\n  GeometryIsSmall = No\n";
}

static std::string body(const char* health) {
    return std::string("  Body = ActiveBody ModuleTag_Body\n    MaxHealth = ") + health +
           "\n    InitialHealth = " + health + "\n  End\n";
}

static std::string fixture_ini() {
    std::string s =
        "Upgrade Upgrade_AmericaSupplyLines\n  Type = PLAYER\nEnd\n"
        "Locomotor P320_Ground\n  Surfaces = GROUND\n  Speed = 30\n  TurnRate = 180\n"
        "  Acceleration = 30\n  Braking = 30\nEnd\n"
        "Locomotor P320_Air\n  Surfaces = AIR\n  Speed = 60\n  TurnRate = 180\n"
        "  Acceleration = 60\n  Braking = 60\n  MinSpeed = 0\n  Lift = 100\nEnd\n";
    for (int k = 0; k < 3; ++k) {
        for (int v = 0; v < kVariants[k]; ++v) {
            const DockType& d = kDockTypes[k][v];
            s += "Object P320_Dock" + std::to_string(k) + "_" + std::to_string(v) +
                 "\n  KindOf = PRELOAD " + d.kind_of + "\n" + geometry(d.geometry) + body("1000.0") +
                 "  Behavior = " + kDockModule[k] + " ModuleTag_Dock\n" +
                 (d.approach ? std::string("    NumberApproachPositions = ") + d.approach + "\n"
                             : std::string()) +
                 d.extra + "  End\nEnd\n";
        }
    }
    const auto ai = [](const char* module, const char* fields, const char* loco) {
        return std::string("  Behavior = ") + module + " ModuleTag_AI\n" + fields + "  End\n" +
               "  Locomotor = SET_NORMAL " + loco + "\n";
    };
    const auto unit = [&](int i, const char* kind_of, const char* geom, const char* health,
                          const std::string& rest) {
        s += "Object P320_Docker" + std::to_string(i) + "\n  KindOf = PRELOAD " + kind_of + "\n" +
             geometry(geom) + body(health) + rest + "End\n";
    };
    unit(D_TRUCK, "VEHICLE HARVESTER", "CYLINDER 8 8 10", "300.0",
         ai("SupplyTruckAIUpdate", "    MaxBoxes = 4\n", "P320_Ground"));
    unit(D_CHINOOK, "VEHICLE AIRCRAFT HARVESTER", "SPHERE 12 12 12", "250.0",
         ai("ChinookAIUpdate", "    MaxBoxes = 8\n    UpgradedSupplyBoost = 60\n", "P320_Air"));
    unit(D_WORKER, "INFANTRY DOZER HARVESTER", "CYLINDER 4 4 8", "100.0",
         ai("SupplyTruckAIUpdate", "    MaxBoxes = 1\n", "P320_Ground"));
    unit(D_STEALTH, "VEHICLE", "BOX 10 6 5", "180.0",
         ai("SupplyTruckAIUpdate", "    MaxBoxes = 3\n", "P320_Ground") +
             "  Behavior = StealthUpdate ModuleTag_Stealth\n    InnateStealth = No\n  End\n");
    unit(D_PLAIN, "VEHICLE", "BOX 7 5 6", "500.0", "");
    unit(D_TANK, "VEHICLE", "CYLINDER 6 6 8", "400.0",
         ai("AIUpdateInterface", "", "P320_Ground"));
    unit(D_ONE_BOX, "VEHICLE AIRCRAFT", "SPHERE 9 9 9", "100.0",
         ai("ChinookAIUpdate", "    MaxBoxes = 1\n    UpgradedSupplyBoost = 60\n", "P320_Air"));
    unit(D_NO_BOX, "VEHICLE AIRCRAFT", "SPHERE 9 9 9", "100.0",
         ai("ChinookAIUpdate", "    MaxBoxes = 0\n    UpgradedSupplyBoost = 60\n", "P320_Air"));
    return s;
}

// ---------------------------------------------------------------------------

enum {
    OP_CLEAR_APPROACH, OP_RESERVE, OP_ADVANCE, OP_CLEAR_ENTER, OP_CLEAR_ADVANCE,
    OP_ENTER_POS, OP_DOCK_POS, OP_EXIT_POS, OP_APPROACH_REACHED, OP_ENTER_REACHED,
    OP_DOCK_REACHED, OP_EXIT_REACHED, OP_CANCEL, OP_UPDATE, OP_SET_OPEN, OP_SET_CRIPPLED,
    OP_ACTION, OP_QUERIES, OP_MOVE, OP_BOXES, OP_CASH, OP_STEP, OP_BONES, OP_LOAD_POSITIONS,
    OP_STATUS, OP_HURT, OP_GRANT, OP_DESTROY
};

struct Docker {
    Object* obj = nullptr;
    int type = 0;
    int team = 0;  // 0 team_a, 1 team_b (which has the supply lines upgrade)
    int dx = 0, dy = 0, dz = 0;
    // Generator bookkeeping.
    int stage = 0;
    int index = -1;
    bool gone = false;
};

struct Scene {
    int kind = 0, variant = 0, dock_team = 0;
    Real angle = 0.0f;
    int cx = 500, cy = 500;
    UnsignedInt frame = 1;
    uint32_t seed = 0, base_id = 0;
    Object* dock = nullptr;
    DockUpdate* module = nullptr;
    DockUpdateInterface* iface = nullptr;
    std::vector<Docker> dockers;
    UnsignedInt money0[2] = {0, 0};
    Int earned0[2] = {0, 0};
    std::string last_save;
};

static uint32_t model_bits(const Object* o) {
    const Drawable* d = o->getDrawable();
    if (!d) {
        std::fprintf(stderr, "object without a drawable\n");
        std::exit(1);
    }
    const ModelConditionFlags& f = d->getModelConditionFlags();
    return (f.test(MODELCONDITION_DOCKING) ? 1u : 0u) |
           (f.test(MODELCONDITION_DOCKING_BEGINNING) ? 2u : 0u) |
           (f.test(MODELCONDITION_DOCKING_ACTIVE) ? 4u : 0u) |
           (f.test(MODELCONDITION_DOCKING_ENDING) ? 8u : 0u) |
           (f.test(MODELCONDITION_MOVING) ? 16u : 0u);
}

static SupplyTruckAIInterface* truck_of(Object* o) {
    return o->getAI() ? o->getAI()->getSupplyTruckAIInterface() : nullptr;
}

// flags x y z health status(can_stealth|stealthed|dead) boxes force
static void docker_state(std::vector<uint32_t>& out, Object* o) {
    const Coord3D* p = o->getPosition();
    SupplyTruckAIInterface* t = truck_of(o);
    out.push_back(model_bits(o));
    out.push_back(bits(p->x));
    out.push_back(bits(p->y));
    out.push_back(bits(p->z));
    out.push_back(bits(o->getBodyModule()->getHealth()));
    out.push_back((o->testStatus(OBJECT_STATUS_CAN_STEALTH) ? 1u : 0u) |
                  (o->testStatus(OBJECT_STATUS_STEALTHED) ? 2u : 0u) |
                  (o->isEffectivelyDead() ? 4u : 0u));
    out.push_back(t ? static_cast<uint32_t>(t->getNumberBoxes()) : 0u);
    out.push_back(t && t->isForcedIntoWantingState() ? 1u : 0u);
}

static Player* player_of(oracle_wp51::World& w, int team) {
    return (team ? w.team_b : w.team_a)->getControllingPlayer();
}

// "<save|=> <crc> <next call> <observations>"
static std::string row(oracle_wp51::World& w, Scene& s, const std::vector<uint32_t>& ret, int d) {
    std::string save = oracle_wp51::World::snapshot_save_hex(s.module);
    std::ostringstream out;
    out << (save == s.last_save ? std::string("=") : save);
    s.last_save = save;
    std::vector<uint32_t> v;
    v.push_back(oracle_wp51::World::snapshot_crc(s.module));
    v.push_back(s.module->friend_getNextCallFrame());
    v.push_back(model_bits(s.dock));
    v.push_back(GetGameLogicRandomSeedCRC());
    for (int t = 0; t < 2; ++t) {
        v.push_back(player_of(w, t)->getMoney()->countMoney() - s.money0[t]);
        v.push_back(static_cast<uint32_t>(player_of(w, t)->getScoreKeeper()->getTotalMoneyEarned() -
                                          s.earned0[t]));
    }
    SupplyWarehouseDockUpdate* warehouse = dynamic_cast<SupplyWarehouseDockUpdate*>(s.module);
    v.push_back(warehouse ? static_cast<uint32_t>(warehouse->getBoxesStored()) : 0u);
    v.push_back(s.dock->isDestroyed() ? 1u : 0u);
    // A hash over every docker, then the one the operation named.
    std::vector<uint32_t> all;
    for (const Docker& k : s.dockers)
        if (!k.gone) docker_state(all, k.obj);
    uint32_t h = 2166136261u;
    for (uint32_t x : all) h = (h ^ x) * 16777619u;
    v.push_back(h);
    if (d >= 0 && !s.dockers[d].gone) docker_state(v, s.dockers[d].obj);
    for (uint32_t x : v) out << ' ' << x;
    out << " |";
    for (uint32_t x : ret) out << ' ' << x;
    return out.str();
}

static void position_result(std::vector<uint32_t>& ret, const Coord3D& p) {
    ret.push_back(bits(p.x));
    ret.push_back(bits(p.y));
    ret.push_back(bits(p.z));
}

// Positions op 22 writes. Pattern 0 leaves the enter position zero.
static void set_bones(DockUpdate* m, uint32_t pattern) {
    static const Real kT[4][3] = {{0, 0, 0}, {40, 0, 0}, {-12.5f, 33.0f, 4.0f}, {0.25f, -60.0f, 0}};
    const uint32_t a = pattern & 3, b = (pattern >> 2) & 3, c = (pattern >> 4) & 3;
    const auto at = [](uint32_t i) {
        Coord3D p;
        p.x = kT[i][0];
        p.y = kT[i][1];
        p.z = kT[i][2];
        return p;
    };
    m->*&Peek::m_enterPosition = at(a);
    m->*&Peek::m_dockPosition = at(b);
    m->*&Peek::m_exitPosition = at(c);
    std::vector<Coord3D>& approach = m->*&Peek::m_approachPositions;
    for (size_t i = 0; i < approach.size(); ++i) approach[i] = at((c + i) & 3);
    const uint32_t bone = (pattern >> 6) & 3;  // 0 keep, 1 none loaded, 2 two bones, 3 zero
    if (bone) m->*&Peek::m_numberApproachPositionBones = bone == 1 ? -1 : (bone == 2 ? 2 : 0);
    const uint32_t count = (pattern >> 8) & 3;  // 0 keep, 1 dynamic, 2 one more than held
    if (count == 1) m->*&Peek::m_numberApproachPositions = -1;
    if (count == 2) m->*&Peek::m_numberApproachPositions = static_cast<Int>(approach.size()) + 1;
}

static Coord3D docker_position(const Scene& s, int dx, int dy) {
    Coord3D p;
    p.x = static_cast<Real>(s.cx + dx);
    p.y = static_cast<Real>(s.cy + dy);
    p.z = 0.0f;
    return p;
}

static void apply(oracle_wp51::World& w, Scene& s, int code, int d, uint32_t a,
                  std::vector<uint32_t>& ret) {
    Object* o = d >= 0 ? s.dockers[d].obj : nullptr;
    DockUpdateInterface* dock = s.iface;
    const Int index = static_cast<Int>(a) - 1;
    Coord3D pos;
    pos.zero();
    switch (code) {
        case OP_CLEAR_APPROACH:
            ret.push_back(dock->isClearToApproach(o) ? 1 : 0);
            break;
        case OP_RESERVE: {
            Int got = -7;
            const Bool ok = dock->reserveApproachPosition(o, &pos, &got);
            ret.push_back(ok ? 1 : 0);
            if (ok) {
                position_result(ret, pos);
                ret.push_back(static_cast<uint32_t>(got));
                s.dockers[d].index = got;
                if (s.dockers[d].stage == 0) s.dockers[d].stage = 1;
            }
            break;
        }
        case OP_ADVANCE: {
            Int got = index;
            const Bool ok = dock->advanceApproachPosition(o, &pos, &got);
            ret.push_back(ok ? 1 : 0);
            if (ok) {
                position_result(ret, pos);
                ret.push_back(static_cast<uint32_t>(got));
                s.dockers[d].index = got;
                s.dockers[d].stage = 1;
            }
            break;
        }
        case OP_CLEAR_ENTER:
            ret.push_back(dock->isClearToEnter(o) ? 1 : 0);
            break;
        case OP_CLEAR_ADVANCE:
            ret.push_back(dock->isClearToAdvance(o, index) ? 1 : 0);
            break;
        case OP_ENTER_POS:
            dock->getEnterPosition(o, &pos);
            position_result(ret, pos);
            break;
        case OP_DOCK_POS:
            dock->getDockPosition(o, &pos);
            position_result(ret, pos);
            break;
        case OP_EXIT_POS:
            dock->getExitPosition(o, &pos);
            position_result(ret, pos);
            break;
        case OP_APPROACH_REACHED:
            dock->onApproachReached(o);
            if (s.dockers[d].stage == 1) s.dockers[d].stage = 2;
            break;
        case OP_ENTER_REACHED:
            dock->onEnterReached(o);
            s.dockers[d].stage = 4;
            break;
        case OP_DOCK_REACHED:
            dock->onDockReached(o);
            s.dockers[d].stage = 5;
            break;
        case OP_EXIT_REACHED:
            dock->onExitReached(o);
            s.dockers[d].stage = 0;
            break;
        case OP_CANCEL:
            dock->cancelDock(o);
            s.dockers[d].stage = 0;
            break;
        case OP_UPDATE:
            ret.push_back(static_cast<uint32_t>(s.module->update()));
            break;
        case OP_SET_OPEN:
            dock->setDockOpen(a ? TRUE : FALSE);
            break;
        case OP_SET_CRIPPLED:
            dock->setDockCrippled(a ? TRUE : FALSE);
            break;
        case OP_ACTION: {
            Object* drone = a ? s.dockers[a - 1].obj : nullptr;
            ret.push_back(dock->action(o, drone) ? 1 : 0);
            if (drone) docker_state(ret, drone);
            break;
        }
        case OP_QUERIES: {
            ret.push_back(dock->isDockOpen() ? 1 : 0);
            ret.push_back(dock->isAllowPassthroughType() ? 1 : 0);
            ret.push_back(dock->isRallyPointAfterDockType() ? 1 : 0);
            break;
        }
        case OP_MOVE: {
            s.dockers[d].dx = static_cast<int>(a & 0xffff) - 200;
            s.dockers[d].dy = static_cast<int>(a >> 16) - 200;
            const Coord3D p = docker_position(s, s.dockers[d].dx, s.dockers[d].dy);
            o->setPosition(&p);
            break;
        }
        case OP_BOXES: {
            SupplyTruckAIInterface* t = truck_of(o);
            ret.push_back(a ? (t->loseOneBox() ? 1 : 0) : (t->gainOneBox(5) ? 1 : 0));
            break;
        }
        case OP_CASH:
            static_cast<SupplyWarehouseDockUpdate*>(s.module)->setCashValue(static_cast<Int>(a) - 100);
            break;
        case OP_STEP:
            for (uint32_t k = 0; k < a; ++k) {
                w.set_frame(++s.frame);
                if (s.module->friend_getNextCallFrame() > s.frame) continue;
                UpdateSleepTime sleep_len = s.module->update();
                if (sleep_len < 1) sleep_len = UPDATE_SLEEP_NONE;
                UnsignedInt next = s.frame + static_cast<UnsignedInt>(sleep_len);
                if (next > UPDATE_SLEEP_FOREVER) next = UPDATE_SLEEP_FOREVER;
                s.module->friend_setNextCallFrame(next);
            }
            break;
        case OP_BONES:
            set_bones(s.module, a);
            break;
        case OP_LOAD_POSITIONS:
            (s.module->*&Peek::loadDockPositions)();
            break;
        case OP_STATUS: {
            // bit0 on the docker, bit1 STEALTHED else CAN_STEALTH, bit2 set;
            // 8 sets the docker's MOVING condition instead.
            if (a == 8) {
                o->setModelConditionState(MODELCONDITION_MOVING);
                break;
            }
            Object* target = (a & 1) ? o : s.dock;
            target->setStatus(MAKE_OBJECT_STATUS_MASK((a & 2) ? OBJECT_STATUS_STEALTHED
                                                              : OBJECT_STATUS_CAN_STEALTH),
                              (a & 4) ? TRUE : FALSE);
            break;
        }
        case OP_HURT: {
            DamageInfo info;
            info.in.m_damageType = DAMAGE_UNRESISTABLE;
            info.in.m_deathType = DEATH_NORMAL;
            info.in.m_sourceID = INVALID_ID;
            info.in.m_amount = static_cast<Real>(a);
            o->attemptDamage(&info);
            break;
        }
        case OP_GRANT:
            o->getStealth()->receiveGrant(TRUE, a);
            break;
        case OP_DESTROY:
            TheGameLogic->destroyObject(o);
            w.flush_destroy_list();
            s.dockers[d].gone = true;
            break;
    }
}

// ---------------------------------------------------------------------------
// Scripted vectors: one named situation each, before the random ones.
// ---------------------------------------------------------------------------

struct Op {
    int code, d;
    uint32_t a;
};
struct DockerSpec {
    int type, team, dx, dy;
    int dz = 0;
};
struct Scripted {
    int kind, variant, team;
    std::vector<DockerSpec> dockers;
    std::vector<Op> ops;
    int cx = 0;  // 0 keeps the usual place
};

static uint32_t at_offset(int dx, int dy) {
    return static_cast<uint32_t>(dx + 200) | (static_cast<uint32_t>(dy + 200) << 16);
}

static std::vector<Scripted> scripts() {
    std::vector<Scripted> all;
    // A full queue: a docker too many, a cancel, an advance, entry and boxes.
    all.push_back({0, 0, 0,
                   {{D_WORKER, 0, 60, 0}, {D_TRUCK, 0, 0, 70}, {D_CHINOOK, 1, -50, 5},
                    {D_PLAIN, 0, 5, -80}},
                   {{OP_RESERVE, 0, 0}, {OP_RESERVE, 1, 0}, {OP_RESERVE, 2, 0}, {OP_RESERVE, 3, 0},
                    {OP_CLEAR_APPROACH, 3, 0}, {OP_CLEAR_APPROACH, 1, 0}, {OP_RESERVE, 0, 0},
                    {OP_APPROACH_REACHED, 1, 0}, {OP_CLEAR_ADVANCE, 1, 2}, {OP_CANCEL, 0, 0},
                    {OP_CLEAR_ADVANCE, 1, 2}, {OP_CLEAR_ADVANCE, 0, 2}, {OP_ADVANCE, 1, 2},
                    {OP_CLEAR_ADVANCE, 1, 1}, {OP_ADVANCE, 1, 1}, {OP_ADVANCE, 1, 0},
                    {OP_APPROACH_REACHED, 1, 0}, {OP_UPDATE, 1, 0}, {OP_CLEAR_ENTER, 1, 0},
                    {OP_CLEAR_ENTER, 2, 0}, {OP_ENTER_POS, 1, 0}, {OP_ENTER_REACHED, 1, 0},
                    {OP_DOCK_POS, 1, 0}, {OP_DOCK_REACHED, 1, 0}, {OP_ACTION, 1, 0},
                    {OP_MOVE, 1, at_offset(35, 0)}, {OP_ACTION, 1, 0}, {OP_ACTION, 1, 0},
                    {OP_ACTION, 1, 0}, {OP_ACTION, 1, 0}, {OP_ACTION, 1, 0},
                    {OP_EXIT_POS, 1, 0}, {OP_EXIT_REACHED, 1, 0}, {OP_STEP, -1, 3}}});
    // The update stops a worker that is still animating as moving.
    all.push_back({0, 0, 0,
                   {{D_WORKER, 0, 40, 0}, {D_TRUCK, 0, -40, 0}},
                   {{OP_RESERVE, 0, 0}, {OP_APPROACH_REACHED, 0, 0}, {OP_UPDATE, 0, 0},
                    {OP_ENTER_REACHED, 0, 0}, {OP_STATUS, 0, 8}, {OP_UPDATE, 0, 0},
                    {OP_DOCK_REACHED, 0, 0}, {OP_STATUS, 0, 8}, {OP_UPDATE, 0, 0},
                    {OP_CANCEL, 0, 0}, {OP_CANCEL, 1, 0}}});
    // Crippled with the docker inside: a ground docker dies, an airborne one lives.
    for (int type : {D_TRUCK, D_CHINOOK, D_PLAIN}) {
        all.push_back({type == D_PLAIN ? 2 : 0, 1, 1,
                       {{type, 0, 30, 10}, {D_TRUCK, 0, -60, 0}},
                       {{OP_RESERVE, 0, 0}, {OP_APPROACH_REACHED, 0, 0}, {OP_UPDATE, 0, 0},
                        {OP_ENTER_REACHED, 0, 0}, {OP_SET_CRIPPLED, -1, 0},
                        {OP_SET_CRIPPLED, -1, 1}, {OP_RESERVE, 1, 0}, {OP_APPROACH_REACHED, 1, 0},
                        {OP_UPDATE, 0, 0}, {OP_SET_CRIPPLED, -1, 0}, {OP_UPDATE, 0, 0}}});
    }
    // Crippled with the docker on its way in: a supply truck is turned away.
    for (int type : {D_TRUCK, D_CHINOOK, D_STEALTH}) {
        all.push_back({0, 0, 0,
                       {{type, 0, 90, 0}},
                       {{OP_RESERVE, 0, 0}, {OP_APPROACH_REACHED, 0, 0}, {OP_UPDATE, 0, 0},
                        {OP_SET_CRIPPLED, -1, 1}, {OP_UPDATE, 0, 0}, {OP_CANCEL, 0, 0},
                        {OP_UPDATE, 0, 0}}});
    }
    // The active docker is destroyed.
    all.push_back({0, 0, 0,
                   {{D_TRUCK, 0, 50, 0}, {D_WORKER, 0, -50, 0}},
                   {{OP_RESERVE, 0, 0}, {OP_APPROACH_REACHED, 0, 0}, {OP_UPDATE, 0, 0},
                    {OP_DESTROY, 0, 0}, {OP_SET_CRIPPLED, -1, 1}, {OP_UPDATE, -1, 0},
                    {OP_SET_CRIPPLED, -1, 0}, {OP_RESERVE, 1, 0}, {OP_APPROACH_REACHED, 1, 0},
                    {OP_UPDATE, 1, 0}}});
    // A warehouse that is deleted when its last box goes.
    all.push_back({0, 3, 0,
                   {{D_TRUCK, 0, 32, 0}},
                   {{OP_ACTION, 0, 0}, {OP_ACTION, 0, 0}}});
    // An empty warehouse, and a docker that is full.
    all.push_back({0, 0, 0,
                   {{D_WORKER, 0, 32, 0}, {D_TANK, 0, 32, 5}},
                   {{OP_ACTION, 0, 0}, {OP_ACTION, 0, 0}, {OP_ACTION, 1, 0}, {OP_CASH, -1, 100},
                    {OP_ACTION, 0, 0}, {OP_MOVE, 0, at_offset(150, 0)}, {OP_ACTION, 0, 0}}});
    // The distance limit of a warehouse action, step by step.
    {
        Scripted s{0, 2, 0, {{D_TRUCK, 0, 40, 0}}, {{OP_CASH, -1, 3100}}};
        for (int x = 44; x <= 66; ++x) {
            s.ops.push_back({OP_MOVE, 0, at_offset(x, 0)});
            s.ops.push_back({OP_ACTION, 0, 0});
            s.ops.push_back({OP_BOXES, 0, 1});
        }
        all.push_back(s);
    }
    all.push_back({0, 2, 0,
                   {{D_TRUCK, 0, 90, 0}},
                   {{OP_CASH, -1, 100}, {OP_CASH, -1, 101}, {OP_CASH, -1, 199}, {OP_CASH, -1, 200},
                    {OP_CASH, -1, 201}, {OP_CASH, -1, 99}, {OP_CASH, -1, 0}, {OP_CASH, -1, 1300}}});
    // Stealth lent by a stealthed supply center: first grant, refresh, and the
    // cases that get none.
    all.push_back({1, 0, 0,
                   {{D_STEALTH, 0, 40, 0}, {D_TRUCK, 1, -40, 0}},
                   {{OP_BOXES, 0, 0}, {OP_ACTION, 0, 0}, {OP_STATUS, 0, 6}, {OP_ACTION, 0, 0},
                    {OP_BOXES, 0, 0}, {OP_ACTION, 0, 0}, {OP_BOXES, 0, 0}, {OP_BOXES, 0, 0},
                    {OP_ACTION, 0, 0}, {OP_GRANT, 0, 0}, {OP_STATUS, 0, 3}, {OP_BOXES, 0, 0},
                    {OP_ACTION, 0, 0}, {OP_STATUS, 0, 1}, {OP_BOXES, 0, 0}, {OP_ACTION, 0, 0},
                    {OP_GRANT, 0, 9}, {OP_STATUS, 0, 3}, {OP_BOXES, 0, 0}, {OP_ACTION, 0, 0},
                    {OP_BOXES, 1, 0}, {OP_ACTION, 1, 0}, {OP_STATUS, 0, 2}, {OP_STATUS, 0, 3},
                    {OP_STATUS, 0, 1}, {OP_BOXES, 0, 0}, {OP_ACTION, 0, 0}}});
    // The upgraded boost, scaled by the load.
    all.push_back({1, 1, 1,
                   {{D_CHINOOK, 1, 40, 0}, {D_CHINOOK, 0, -40, 0}, {D_PLAIN, 0, 0, 40},
                    {D_TANK, 0, 0, -40}},
                   {{OP_ACTION, 0, 0}, {OP_BOXES, 0, 0}, {OP_BOXES, 0, 0}, {OP_BOXES, 0, 0},
                    {OP_ACTION, 0, 0}, {OP_BOXES, 1, 0}, {OP_ACTION, 1, 0}, {OP_ACTION, 2, 0},
                    {OP_ACTION, 3, 0}}});
    all.push_back({1, 1, 1,
                   {{D_ONE_BOX, 1, 40, 0}, {D_NO_BOX, 1, -40, 0}},
                   {{OP_BOXES, 0, 0}, {OP_ACTION, 0, 0}, {OP_BOXES, 1, 0}, {OP_ACTION, 1, 0}}});
    // A grant of one frame is still temporary, so it is renewed.
    all.push_back({1, 0, 0,
                   {{D_STEALTH, 0, 40, 0}},
                   {{OP_STATUS, -1, 6}, {OP_GRANT, 0, 1}, {OP_STATUS, 0, 3}, {OP_BOXES, 0, 0},
                    {OP_ACTION, 0, 0}}});
    // Repair: the rate of a docker that leaves early passes to the next; a drone.
    all.push_back({2, 2, 0,
                   {{D_PLAIN, 0, 40, 0}, {D_TRUCK, 0, -40, 0}, {D_TANK, 0, 0, 40}},
                   {{OP_HURT, 0, 150}, {OP_HURT, 2, 60}, {OP_ACTION, 0, 0}, {OP_ACTION, 0, 3},
                    {OP_HURT, 1, 10}, {OP_ACTION, 1, 0}, {OP_ACTION, 1, 0}, {OP_ACTION, 1, 0},
                    {OP_HURT, 1, 120}, {OP_ACTION, 1, 0}, {OP_ACTION, 1, 1}, {OP_ACTION, 0, 0},
                    {OP_HURT, 2, 1000}, {OP_ACTION, 2, 0}}});
    // A zero heal time: not a number when the docker is whole, infinity when hurt.
    all.push_back({2, 4, 0,
                   {{D_PLAIN, 0, 40, 0}},
                   {{OP_ACTION, 0, 0}, {OP_HURT, 0, 100}, {OP_ACTION, 0, 0}, {OP_ACTION, 0, 0}}});
    // Positions with and without bones, for ground and airborne dockers.
    for (uint32_t bones : {0u, 1u, 4u | 16u, 2u | (3u << 2) | (1u << 4), 1u | (2u << 6),
                           3u | (2u << 2) | (3u << 6)}) {
        all.push_back({0, 0, 0,
                       {{D_TRUCK, 0, 55, 20}, {D_CHINOOK, 0, -30, -45}},
                       {{OP_LOAD_POSITIONS, -1, 0}, {OP_BONES, -1, bones}, {OP_ENTER_POS, 0, 0},
                        {OP_ENTER_POS, 1, 0}, {OP_DOCK_POS, 0, 0}, {OP_DOCK_POS, 1, 0},
                        {OP_EXIT_POS, 0, 0}, {OP_EXIT_POS, 1, 0}, {OP_RESERVE, 0, 0},
                        {OP_RESERVE, 1, 0}, {OP_LOAD_POSITIONS, -1, 0}, {OP_RESERVE, 0, 0}}});
    }
    // Each position query loads the positions first.
    for (int q : {OP_ENTER_POS, OP_DOCK_POS, OP_EXIT_POS, OP_ADVANCE, OP_RESERVE}) {
        all.push_back({0, 4, 0, {{D_TRUCK, 0, 55, 20}}, {{q, 0, 1}, {OP_RESERVE, 0, 0}}});
    }
    // A dynamic list that has to grow, and a fixed one turned dynamic.
    {
        Scripted s{2, 3, 0, {}, {}};
        for (int i = 0; i < 12; ++i) {
            s.dockers.push_back({i % 2 ? D_PLAIN : D_TRUCK, 0, 40 + 4 * i, 3 * i - 20});
            s.ops.push_back({OP_CLEAR_APPROACH, i, 0});
            s.ops.push_back({OP_RESERVE, i, 0});
        }
        s.ops.push_back({OP_APPROACH_REACHED, 11, 0});
        s.ops.push_back({OP_CLEAR_ADVANCE, 11, 12});
        s.ops.push_back({OP_CANCEL, 3, 0});
        s.ops.push_back({OP_ADVANCE, 4, 5});
        s.ops.push_back({OP_RESERVE, 3, 0});
        s.ops.push_back({OP_UPDATE, 11, 0});
        all.push_back(s);
    }
    // Off the map the search returns the point it was given: the bare approach
    // arithmetic, in three dimensions. A dock too wide to clear does the same.
    for (int variant : {0, 6}) {
        Scripted s{0, variant, 0,
                   {{D_TRUCK, 0, 40, 25, 30}, {D_CHINOOK, 0, -33, 41, 55}, {D_PLAIN, 0, 0, 0, 0}},
                   {{OP_RESERVE, 0, 0}, {OP_RESERVE, 1, 0}, {OP_RESERVE, 2, 0},
                    {OP_RESERVE, 0, 0}, {OP_ENTER_POS, 1, 0}, {OP_BONES, -1, 2u | (2u << 4)},
                    {OP_RESERVE, 0, 0}, {OP_RESERVE, 1, 0}}};
        if (variant == 0) s.cx = -200;
        all.push_back(s);
    }
    all.push_back({0, 3, 0,
                   {{D_TRUCK, 0, 50, 0}, {D_TRUCK, 0, -50, 0}},
                   {{OP_RESERVE, 0, 0}, {OP_RESERVE, 1, 0}, {OP_BONES, -1, 1u << 8},
                    {OP_RESERVE, 1, 0}, {OP_CLEAR_APPROACH, 1, 0}}});
    return all;
}

// Branches reached, printed at the end of the run.
static int g_hits[10];
static const char* const kHits[10] = {"jostle",        "box_given",     "dock_deleted",
                                      "money_paid",    "stealth_grant", "heal",
                                      "drone_heal",    "crippled_kill", "crippled_turned_away",
                                      "moving_cleared"};

// The next operation: mostly the step a docking unit would take next.
static void choose(Scene& s, int& code, int& d, uint32_t& a) {
    std::vector<int> alive;
    for (size_t i = 0; i < s.dockers.size(); ++i)
        if (!s.dockers[i].gone) alive.push_back(static_cast<int>(i));
    d = alive.empty() ? -1 : alive[pick(static_cast<uint32_t>(alive.size()))];
    a = 0;
    const DockUpdate* m = s.module;
    const std::vector<ObjectID>& owners = const_cast<DockUpdate*>(m)->*&Peek::m_approachPositionOwners;
    const int size = static_cast<int>(owners.size());
    const auto idx_arg = [&]() -> uint32_t {
        const int cur = d >= 0 ? s.dockers[d].index : -1;
        int i = (cur >= 0 && pick(4) != 0) ? cur : static_cast<int>(pick(size + 1)) - 1;
        if (i >= size) i = size - 1;
        return static_cast<uint32_t>(i + 1);
    };
    // A stealthed supply center is what lends stealth.
    if (s.kind == 1 && pick(10) == 0 && !s.dock->testStatus(OBJECT_STATUS_STEALTHED)) {
        code = OP_STATUS;
        a = 6;
        return;
    }
    if (d >= 0 && pick(100) < 55) {
        const Docker& k = s.dockers[d];
        switch (k.stage) {
            case 0: code = OP_RESERVE; return;
            case 1: code = OP_APPROACH_REACHED; return;
            case 2:
                if (pick(4) == 0) {
                    code = OP_CLEAR_ADVANCE;
                    a = idx_arg();
                    return;
                }
                if (pick(2)) {
                    code = OP_ADVANCE;
                    a = idx_arg();
                    return;
                }
                code = pick(3) ? OP_UPDATE : OP_CLEAR_ENTER;
                if (const_cast<DockUpdate*>(m)->*&Peek::m_activeDocker == k.obj->getID()) {
                    code = OP_ENTER_REACHED;
                }
                return;
            case 4:
                code = pick(4) ? OP_DOCK_REACHED : OP_UPDATE;
                if (k.type == D_WORKER && pick(3) == 0) {
                    // Moving while docking; the dock's update stops that.
                    code = (model_bits(k.obj) & 16) ? OP_UPDATE : OP_STATUS;
                    a = code == OP_STATUS ? 8 : 0;
                    return;
                }
                // What the action of this dock needs: boxes to deliver, or damage.
                if (s.kind == 1 && truck_of(k.obj) && pick(2)) {
                    code = OP_BOXES;
                } else if (s.kind == 2 && pick(2)) {
                    code = OP_HURT;
                    a = 1 + pick(90);
                }
                return;
            default:
                code = pick(3) ? OP_ACTION : OP_EXIT_REACHED;
                if (code == OP_ACTION && s.kind == 2 && pick(3) == 0) {
                    a = static_cast<uint32_t>(alive[pick(static_cast<uint32_t>(alive.size()))]) + 1;
                }
                return;
        }
    }
    const int weights[] = {
        /*CLEAR_APPROACH*/ 3, 5, 5, 3, 4, 3, 3, 3, 4, 3, 3, 3, 4, 5, 2, 4,
        /*ACTION*/ 10, 2, 6, 6, 2, 3, 2, 2, 5, 5, 2, 1};
    int total = 0;
    for (int x : weights) total += x;
    int r = static_cast<int>(pick(total));
    for (code = 0; r >= weights[code]; ++code) r -= weights[code];
    if (d < 0 && !(code == OP_UPDATE || code == OP_SET_OPEN || code == OP_SET_CRIPPLED ||
                   code == OP_QUERIES || code == OP_STEP || code == OP_BONES ||
                   code == OP_LOAD_POSITIONS || (code == OP_CASH && s.kind == 0))) {
        code = OP_UPDATE;
    }
    switch (code) {
        case OP_ADVANCE:
        case OP_CLEAR_ADVANCE: a = idx_arg(); break;
        case OP_SET_OPEN: a = pick(2); break;
        case OP_SET_CRIPPLED: a = pick(4) != 0; break;
        case OP_ACTION:
            if (s.kind == 2 && pick(3) == 0) {
                a = static_cast<uint32_t>(alive[pick(static_cast<uint32_t>(alive.size()))]) + 1;
            }
            break;
        case OP_MOVE: {
            const int reach = pick(2) ? 40 : 190;
            const int dx = static_cast<int>(pick(2 * reach + 1)) - reach;
            const int dy = static_cast<int>(pick(2 * reach + 1)) - reach;
            const bool onto = pick(10) == 0;  // exactly on the dock
            a = static_cast<uint32_t>((onto ? 200 : dx + 200)) |
                (static_cast<uint32_t>(onto ? 200 : dy + 200) << 16);
            break;
        }
        case OP_BOXES:
            if (!truck_of(s.dockers[d].obj)) code = OP_CLEAR_ENTER;
            else a = pick(4) == 0;
            break;
        case OP_CASH:
            if (s.kind != 0) {
                code = OP_QUERIES;
            } else {
                a = pick(3) == 0 ? pick(5) * 100 : pick(1300);
            }
            break;
        case OP_STEP: a = 1 + pick(6); break;
        case OP_BONES: a = pick(64) | (pick(3) == 0 ? pick(4) << 6 : 0) | (pick(5) == 0 ? (1 + pick(2)) << 8 : 0); break;
        case OP_STATUS:
            if (pick(4) == 0) a = 8;
            else a = pick(8) | (pick(2) ? 4u : 0u);
            break;
        case OP_HURT: a = pick(4) == 0 ? 1000 : 1 + pick(150); break;
        case OP_GRANT:
            if (!s.dockers[d].obj->getStealth()) code = OP_CLEAR_ENTER;
            else a = pick(3) == 0 ? 0 : 1 + pick(500);
            break;
        case OP_DESTROY:
            if (pick(2)) code = OP_UPDATE;
            break;
        default: break;
    }
}

// Calls that crash the reference.
static bool crashes(Scene& s, int code, int d, uint32_t a) {
    if (s.kind != 0) return false;
    if (code == OP_ACTION) return !s.dockers[d].obj->getAI();
    if (code == OP_SET_CRIPPLED && a) {
        const ObjectID active = s.module->*&Peek::m_activeDocker;
        if (active == INVALID_ID || s.module->*&Peek::m_dockerInside) return false;
        const Object* victim = TheGameLogic->findObjectByID(active);
        return victim && !victim->getAI();
    }
    return false;
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_pkt320_381_378_363_docks <out.json>\n");
        return 2;
    }
    oracle_wp51::World w;
    w.load_ini(fixture_ini());
    w.init_with_neutral_player();
    // The floating cash text needs fonts and a display, which a headless run has not.
    TheGameLogic->setDrawIconUI(FALSE);
    {
        const UpgradeTemplate* lines = TheUpgradeCenter->findUpgrade("Upgrade_AmericaSupplyLines");
        if (!lines) return 1;
        player_of(w, 1)->addUpgrade(lines, UPGRADE_STATUS_COMPLETE);
    }

    const std::string tmp = std::string(argv[1]) + ".tmp";
    FILE* out = std::fopen(tmp.c_str(), "w");
    if (!out) return 1;
    const std::vector<Scripted> scripted = scripts();
    const int kScripted = static_cast<int>(scripted.size());
    const int kTotal = 1200;
    std::fprintf(out, "{\"vector_count\":%d,\"vectors\":[", kTotal);
    uint32_t next_id = 0;
    for (int i = 0; i < kTotal; ++i) {
        const Scripted* script = i < kScripted ? &scripted[i] : nullptr;
        Scene s;
        s.kind = script ? script->kind : i % 3;
        s.variant = script ? script->variant : static_cast<int>(pick(kVariants[s.kind]));
        s.dock_team = script ? script->team : static_cast<int>(pick(2));
        static const Real kAngles[4] = {0.0f, 1.5707964f, 0.7f, -2.3f};
        const uint32_t angle = pick(4);
        s.angle = kAngles[angle];
        s.cx = 480 + static_cast<int>(pick(41));
        s.cy = 480 + static_cast<int>(pick(41));
        if (script && script->cx) s.cx = script->cx;
        s.frame = 1 + pick(5000);
        s.seed = 1 + pick(0x7ffffffeu);
        w.set_frame(s.frame);

        Coord3D at = docker_position(s, 0, 0);
        const std::string name =
            "P320_Dock" + std::to_string(s.kind) + "_" + std::to_string(s.variant);
        s.dock = w.spawn_object(name.c_str(), s.dock_team ? w.team_b : w.team_a, at, s.angle);
        s.module = find_behavior<DockUpdate>(s.dock);
        s.iface = s.dock->getDockUpdateInterface();
        if (!s.module || !s.iface) {
            std::fprintf(stderr, "missing module on %s\n", name.c_str());
            return 1;
        }
        s.base_id = s.dock->getID();
        if (next_id && s.base_id < next_id) return 1;

        const int count = script ? static_cast<int>(script->dockers.size())
                                 : 2 + static_cast<int>(pick(4));
        for (int d = 0; d < count; ++d) {
            Docker k;
            // Mostly the units that suit the dock.
            if (s.kind == 2) k.type = static_cast<int>(pick(D_COUNT));
            else k.type = pick(8) == 0 ? (pick(2) ? D_PLAIN : D_TANK) : static_cast<int>(pick(4));
            if (s.kind == 1 && pick(4) == 0) k.type = D_STEALTH;
            k.team = static_cast<int>(pick(2));
            const int reach = pick(2) ? 45 : 150;
            k.dx = static_cast<int>(pick(2 * reach + 1)) - reach;
            k.dy = static_cast<int>(pick(2 * reach + 1)) - reach;
            if (k.type == D_CHINOOK || pick(8) == 0) k.dz = static_cast<int>(pick(60));
            if (script) {
                const DockerSpec& spec = script->dockers[d];
                k.type = spec.type;
                k.team = spec.team;
                k.dx = spec.dx;
                k.dy = spec.dy;
                k.dz = spec.dz;
            }
            const std::string unit = "P320_Docker" + std::to_string(k.type);
            Coord3D place = docker_position(s, k.dx, k.dy);
            place.z = static_cast<Real>(k.dz);
            k.obj = w.spawn_object(unit.c_str(), k.team ? w.team_b : w.team_a, place, 0.0f);
            if (!k.obj) return 1;
            s.dockers.push_back(k);
        }
        next_id = s.dockers.back().obj->getID() + 1;
        ThePartitionManager->update();
        for (int t = 0; t < 2; ++t) {
            s.money0[t] = player_of(w, t)->getMoney()->countMoney();
            s.earned0[t] = player_of(w, t)->getScoreKeeper()->getTotalMoneyEarned();
        }
        InitRandom(s.seed);

        std::fprintf(out, "%s\n{\"kind\":%d,\"variant\":%d,\"team\":%d,\"angle\":%u,\"cx\":%d,\"cy\":%d,"
                          "\"frame\":%u,\"seed\":%u,\"id\":%u,\"dockers\":\"",
                     i ? "," : "", s.kind, s.variant, s.dock_team, angle, s.cx, s.cy, s.frame, s.seed,
                     s.base_id);
        for (size_t d = 0; d < s.dockers.size(); ++d) {
            const Docker& k = s.dockers[d];
            std::fprintf(out, "%s%d %d %d %d %d", d ? ";" : "", k.type, k.team, k.dx, k.dy, k.dz);
        }
        std::fprintf(out, "\",\"spawn\":\"%s\",\"ops\":\"", row(w, s, {}, -1).c_str());

        std::vector<std::string> rows;
        const int ops = script ? static_cast<int>(script->ops.size())
                               : 8 + static_cast<int>(pick(22));
        for (int k = 0; k < ops; ++k) {
            int code = 0, d = -1;
            uint32_t a = 0;
            if (script) {
                code = script->ops[k].code;
                d = script->ops[k].d;
                a = script->ops[k].a;
                if (crashes(s, code, d, a)) {
                    std::fprintf(stderr, "script %d op %d would crash\n", i, k);
                    return 1;
                }
            } else {
                choose(s, code, d, a);
                if (crashes(s, code, d, a)) {
                    code = OP_CLEAR_ENTER;
                    a = 0;
                }
            }
            std::vector<uint32_t> ret;
            Object* o = d >= 0 ? s.dockers[d].obj : nullptr;
            const Coord3D before = o ? *o->getPosition() : at;
            const UnsignedInt cash = player_of(w, s.dock_team)->getMoney()->countMoney();
            const bool stealthed = o && o->testStatus(OBJECT_STATUS_STEALTHED);
            const ObjectID active = s.module->*&Peek::m_activeDocker;
            Object* victim = active != INVALID_ID ? TheGameLogic->findObjectByID(active) : nullptr;
            const bool dead = victim && victim->isEffectivelyDead();
            const bool forced = victim && truck_of(victim) && truck_of(victim)->isForcedIntoWantingState();
            std::vector<uint32_t> moving;
            for (const Docker& k : s.dockers) moving.push_back(k.gone ? 0 : model_bits(k.obj) & 16);
            apply(w, s, code, d, a, ret);
            if (code == OP_ACTION) {
                if (s.kind == 0 && o->getPosition()->x != before.x) ++g_hits[0];
                if (s.kind == 0 && ret[0]) ++g_hits[1];
                if (s.dock->isDestroyed()) ++g_hits[2];
                if (player_of(w, s.dock_team)->getMoney()->countMoney() != cash) ++g_hits[3];
                if (s.kind == 1 && !stealthed && o->testStatus(OBJECT_STATUS_STEALTHED)) ++g_hits[4];
                if (s.kind == 2 && ret[0]) ++g_hits[5];
                if (s.kind == 2 && ret[0] && a) ++g_hits[6];
            }
            if (code == OP_SET_CRIPPLED && victim) {
                if (!dead && victim->isEffectivelyDead()) ++g_hits[7];
                if (!forced && truck_of(victim) && truck_of(victim)->isForcedIntoWantingState()) ++g_hits[8];
            }
            if (code == OP_UPDATE || code == OP_STEP) {
                for (size_t k = 0; k < s.dockers.size(); ++k)
                    if (!s.dockers[k].gone && moving[k] && !(model_bits(s.dockers[k].obj) & 16)) ++g_hits[9];
            }
            ThePartitionManager->update();
            std::fprintf(out, "%s%d %d %u", k ? ";" : "", code, d, a);
            rows.push_back(row(w, s, ret, d));
            if (s.dock->isDestroyed()) break;
        }
        std::fprintf(out, "\",\"rows\":[");
        for (size_t r = 0; r < rows.size(); ++r)
            std::fprintf(out, "%s\"%s\"", r ? "," : "", rows[r].c_str());
        std::fprintf(out, "]}");

        for (const Docker& k : s.dockers)
            if (!k.gone) TheGameLogic->destroyObject(k.obj);
        if (!s.dock->isDestroyed()) TheGameLogic->destroyObject(s.dock);
        w.flush_destroy_list();
        ThePartitionManager->update();
    }
    for (int i = 0; i < 10; ++i) std::fprintf(stderr, "hit %s %d\n", kHits[i], g_hits[i]);
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    return std::rename(tmp.c_str(), argv[1]) == 0 ? 0 : 1;
}
