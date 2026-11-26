// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for StealthUpdate (PKT-374) and StealthDetectorUpdate (PKT-373),
// run on the unmodified reference modules in a small real world: up to eight objects
// (the stealth unit, a rider, a second disguiser, a detector, an enemy with AI, a
// black market, a garrison building and a transport).
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt374_373_stealth.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/PKT374_373_Stealth.json
//
// Modules are stepped one at a time in a fixed order (stealth unit, rider, second
// disguiser, detector) after a partition update, as in gen_pkt358_351_radar_power.cpp.
// Players A and B are enemies in most vectors, otherwise neutral or allies. Some
// vectors open with a fixed script: hide in the garrison and be revealed there, stand
// in a stealthed house, or have the rider attack from inside.
//
// `InitRandom` is called before each object with one of these modules is built, so the
// client draw of the constructor and the logic draw of the detector are reproducible.
//
// Not covered: SpawnBehavior slaves and RiderChangeContain (neither is ported), a
// rider without a StealthUpdate under UseRiderStealth (the reference reads
// uninitialised locals), disguising an object without physics (the reference
// dereferences null), a mine going under construction (`Object::setStatus` detonates
// it, which is unported), objects without a drawable, mines with a transition, frames
// near the scheduler's clamp, and loading a save into a differently built object.

#include "oracle_wp51_world.h"

#include "Common/AcademyStats.h"
#include "Common/FramePacer.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/Team.h"
#include "GameClient/Drawable.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Module/StealthDetectorUpdate.h"
#include "GameLogic/Module/StealthUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Weapon.h"
#include "GameLogic/WeaponSet.h"

#include <cstdio>
#include <sstream>
#include <string>
#include <vector>

struct StVelTag { using type = Coord3D PhysicsBehavior::*; };
template struct MemberAccess<StVelTag, &PhysicsBehavior::m_vel>;
Coord3D PhysicsBehavior::* access_member(StVelTag);
struct StVelMagTag { using type = Real PhysicsBehavior::*; };
template struct MemberAccess<StVelMagTag, &PhysicsBehavior::m_velMag>;
Real PhysicsBehavior::* access_member(StVelMagTag);
struct StFireFrameTag { using type = UnsignedInt Weapon::*; };
template struct MemberAccess<StFireFrameTag, &Weapon::m_lastFireFrame>;
UnsignedInt Weapon::* access_member(StFireFrameTag);
struct StDisguisedCountTag { using type = UnsignedInt AcademyStats::*; };
template struct MemberAccess<StDisguisedCountTag, &AcademyStats::m_vehiclesDisguised>;
UnsignedInt AcademyStats::* access_member(StDisguisedCountTag);
struct StLoadPostTag { using type = void (Snapshot::*)(); };
template struct MemberAccess<StLoadPostTag, &Snapshot::loadPostProcess>;
void (Snapshot::*access_member(StLoadPostTag))();

struct StIndexTag { using type = Int UpdateModule::*; };
template struct MemberAccess<StIndexTag, &UpdateModule::m_indexInLogic>;
Int UpdateModule::* access_member(StIndexTag);

static uint32_t g_state = 0x37400373u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}
static bool chance(uint32_t percent) { return pick(100) < percent; }

static const int kStealth = 48;
static const int kRider = 8;
static const int kDisguiser = 6;
static const int kDetector = 24;

struct StealthVariant {
    bool transport = false;  // carries a rider
    bool disguiser = false;
    bool physics = true;
    bool ai = true;
    unsigned weapons = 0;
};
static StealthVariant g_sv[kStealth];

static const char* const kLevels[9] = {"ATTACKING",      "MOVING",           "USING_ABILITY",
                                       "FIRING_PRIMARY", "FIRING_SECONDARY", "FIRING_TERTIARY",
                                       "NO_BLACK_MARKET", "TAKING_DAMAGE",   "RIDERS_ATTACKING"};

static const char* choose(std::initializer_list<const char*> v) {
    return *(v.begin() + pick(static_cast<uint32_t>(v.size())));
}

static void field(std::string& s, const char* name, const char* value) {
    if (value) s += std::string("    ") + name + " = " + value + "\n";
}

static std::string levels(uint32_t percent) {
    std::string s;
    for (int i = 0; i < 9; ++i)
        if (chance(percent)) s += std::string(s.empty() ? "" : " ") + kLevels[i];
    return s.empty() ? "NONE" : s;
}

static std::string body() {
    return "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 900\n  End\n";
}

static std::string stealth_object(int i) {
    StealthVariant& v = g_sv[i];
    const bool garrison = i % 8 == 0;  // hides quickly inside the house
    const bool gunner = i % 8 == 4;           // one or two firing slots forbidden
    const bool mover = i % 8 == 5;            // forbidden to move
    v.disguiser = i % 4 == 3;
    v.transport = !v.disguiser && !(garrison || gunner || mover) && i % 5 == 1;
    // Odd transports use their own conditions (even ones take the rider's).
    const bool plain = garrison || gunner || mover || (v.transport && i % 2);
    v.physics = v.disguiser || chance(80);
    v.ai = chance(85);
    v.weapons = chance(70) ? 7 : pick(8);

    std::string kinds = "PRELOAD INFANTRY";
    if (!v.transport && (garrison || chance(50))) kinds += " STEALTH_GARRISON";
    if (i % 16 == 6) kinds += " MINE";

    std::string s = "Object P374_S" + std::to_string(i) + "\n  KindOf = " + kinds + "\n" + body() +
                    "  VisionRange = 100\n  TransportSlotCount = 1\n";
    if (v.weapons) {
        s += "  WeaponSet\n    Conditions = None\n";
        if (v.weapons & 1) s += "    Weapon = PRIMARY P374_Gun\n";
        if (v.weapons & 2) s += "    Weapon = SECONDARY P374_Gun\n";
        if (v.weapons & 4) s += "    Weapon = TERTIARY P374_Gun\n";
        s += "  End\n";
    }
    if (v.physics) s += "  Behavior = PhysicsBehavior ModuleTag_Phys\n  End\n";
    if (v.ai) s += "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n";
    if (v.transport)
        s += "  Behavior = TransportContain ModuleTag_Contain\n    Slots = 3\n"
             "    PassengersAllowedToFire = " +
             std::string(chance(70) || plain ? "Yes" : "No") + "\n    AllowInsideKindOf = INFANTRY\n  End\n";

    s += "  Behavior = StealthUpdate ModuleTag_Stealth\n";
    const char* delay = choose({"0", "33", "100", "500", "2000", "100", "500", nullptr});
    field(s, "StealthDelay", plain ? choose({"0", "33", "100"}) : delay);
    std::string lv = i % 8 == 0   ? "NONE"
                     : i % 8 == 2 ? "FIRING_PRIMARY FIRING_SECONDARY FIRING_TERTIARY"
                                  : levels(i % 3 == 0 ? 55 : 22);
    static const char* const kGuns[] = {"FIRING_PRIMARY", "FIRING_SECONDARY", "FIRING_TERTIARY",
                                        "FIRING_PRIMARY FIRING_TERTIARY"};
    if (gunner) lv = kGuns[(i / 8) % 4];
    if (mover) lv = "MOVING";
    if (v.transport) lv = i % 2 ? (i == 41 ? "NONE" : "RIDERS_ATTACKING") : lv + (lv == "NONE" ? "" : " RIDERS_ATTACKING");
    s += "    StealthForbiddenConditions = " + lv + "\n";
    const char* speed = choose({"0", "3", "30", "150", nullptr});
    field(s, "MoveThresholdSpeed", mover ? ((i / 8) % 2 ? "30" : "3") : speed);
    if (chance(20) && !plain) s += "    RequiredStatus = IS_AIMING_WEAPON\n";
    if (chance(25) && !plain) s += "    ForbiddenStatus = IS_BRAKING IS_AIMING_WEAPON\n";
    field(s, "InnateStealth", plain ? "Yes" : choose({"Yes", "Yes", "Yes", "No", nullptr}));
    if (v.disguiser) {
        s += "    DisguisesAsTeam = Yes\n";
        field(s, "DisguiseTransitionTime", choose({"0", "33", "66", "100", "200", "333"}));
        field(s, "DisguiseRevealTransitionTime", choose({"0", "33", "66", "133", "266"}));
    }
    field(s, "RevealDistanceFromTarget", choose({"0", "0.5", "50", "50.5", nullptr, nullptr}));
    if (chance(35)) s += "    OrderIdleEnemiesToAttackMeUponReveal = Yes\n";
    if (v.transport && i % 2 == 0) s += "    UseRiderStealth = Yes\n";
    field(s, "BlackMarketCheckDelay", choose({"0", "100", "333", "1000", nullptr}));
    if (i % 11 == 5 && !plain) s += "    GrantedBySpecialPower = Yes\n";
    s += "    EnemyDetectionEvaEvent = UnitLost\n    FriendlyOpacityMin = 30%\n";
    s += "  End\nEnd\n";
    return s;
}

static std::string rider_object(int i) {
    std::string s = "Object P374_R" + std::to_string(i) + "\n  KindOf = PRELOAD INFANTRY\n" + body() +
                    "  VisionRange = 100\n  TransportSlotCount = 1\n"
                    "  Behavior = PhysicsBehavior ModuleTag_Phys\n  End\n"
                    "  Behavior = StealthUpdate ModuleTag_Stealth\n";
    field(s, "StealthDelay", choose({"0", "66", "200", "700"}));
    s += "    StealthForbiddenConditions = " + levels(25) + "\n";
    field(s, "InnateStealth", i % 3 == 2 ? "No" : "Yes");
    if (chance(50)) s += "    OrderIdleEnemiesToAttackMeUponReveal = Yes\n";
    s += "  End\nEnd\n";
    return s;
}

static std::string disguiser_object(int i) {
    std::string s = "Object P374_X" + std::to_string(i) + "\n  KindOf = PRELOAD VEHICLE\n" + body() +
                    "  VisionRange = 100\n"
                    "  Behavior = PhysicsBehavior ModuleTag_Phys\n  End\n"
                    "  Behavior = StealthUpdate ModuleTag_Stealth\n    DisguisesAsTeam = Yes\n";
    field(s, "StealthDelay", choose({"0", "100", "500"}));
    field(s, "DisguiseTransitionTime", choose({"33", "66", "133"}));
    field(s, "DisguiseRevealTransitionTime", choose({"33", "100"}));
    s += "  End\nEnd\n";
    return s;
}

static std::string detector_object(int i) {
    const bool garrisonable = i % 3 != 0;
    std::string s = "Object P373_D" + std::to_string(i) + "\n  KindOf = PRELOAD INFANTRY\n" + body() +
                    "  VisionRange = " + choose({"20", "60", "150"}) + "\n  TransportSlotCount = 1\n"
                    "  Behavior = StealthDetectorUpdate ModuleTag_Detect\n";
    field(s, "DetectionRate", choose({"0", "33", "100", "500", nullptr}));
    field(s, "DetectionRange", choose({"0", "0.5", "30", "100", "150", nullptr}));
    if (i % 6 == 4) s += "    InitiallyDisabled = Yes\n";
    if (i % 7 == 2) s += "    ExtraRequiredKindOf = INFANTRY\n";
    if (i % 7 == 5) s += "    ExtraForbiddenKindOf = VEHICLE\n";
    if (garrisonable && chance(60)) s += "    CanDetectWhileGarrisoned = Yes\n";
    if (chance(50)) s += "    CanDetectWhileContained = Yes\n";
    s += "  End\nEnd\n";
    return s;
}

static std::string fixed_objects() {
    return "Object P374_Enemy\n  KindOf = PRELOAD INFANTRY\n" + body() +
           "  VisionRange = 100\n  Behavior = AIUpdateInterface ModuleTag_AI\n  End\nEnd\n"
           "Object P374_Market\n  KindOf = PRELOAD STRUCTURE FS_BLACK_MARKET\n" + body() +
           "  VisionRange = 10\nEnd\n"
           "Object P374_House\n  KindOf = PRELOAD STRUCTURE\n" + body() +
           "  VisionRange = 10\n  Behavior = GarrisonContain ModuleTag_Contain\n"
           "    ContainMax = 10\n    AllowInsideKindOf = INFANTRY\n  End\nEnd\n"
           "Object P374_Truck\n  KindOf = PRELOAD VEHICLE\n" + body() +
           "  VisionRange = 10\n  Behavior = TransportContain ModuleTag_Contain\n"
           "    Slots = 5\n    AllowInsideKindOf = INFANTRY\n  End\nEnd\n";
}

static const char* const kWeaponIni =
    "Weapon P374_Gun\n  PrimaryDamage = 1.0\n  PrimaryDamageRadius = 0.0\n"
    "  AttackRange = 100.0\n  DamageType = EXPLOSION\n  DelayBetweenShots = 100\nEnd\n";

// Newlines as '|', so the Rust test can read the text back from one JSON string.
static std::string one_line(const std::string& s) {
    std::string out = s;
    for (char& c : out)
        if (c == '\n') c = '|';
    return out;
}

template <typename T>
static T* find_behavior(Object* obj) {
    if (!obj) return nullptr;
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m) {
        if (T* typed = dynamic_cast<T*>(*m)) return typed;
    }
    return nullptr;
}

static void step(oracle_wp51::World& w, UpdateModule* u, UnsignedInt frame) {
    w.set_frame(frame);
    if (u->friend_getNextCallFrame() > frame) return;
    UpdateSleepTime sleep_len = u->update();
    if (sleep_len < 1) sleep_len = UPDATE_SLEEP_NONE;
    UnsignedInt next = frame + static_cast<UnsignedInt>(sleep_len);
    if (next > UPDATE_SLEEP_FOREVER) next = UPDATE_SLEEP_FOREVER;
    u->friend_setNextCallFrame(next);
}

enum { S, R, X, D, E, M, G, T, kObjects };

static const Real kOffsets[][3] = {{0, 0, 0},   {10, 0, 0},   {0, 29, 0},  {31, 0, 0},  {0, 49, 0},
                                   {50, 0, 0},  {0, 51, 0},   {99, 0, 0},  {60, 80, 0}, {0, 101, 0},
                                   {80, 0, 60}, {150, 0, 0},  {400, 0, 0}, {0, 20, 40}, {30, 40, 0}};
static const int kPositions = 15;

static const ObjectStatusTypes kStatuses[] = {
    OBJECT_STATUS_IS_FIRING_WEAPON, OBJECT_STATUS_IS_USING_ABILITY, OBJECT_STATUS_IS_ATTACKING,
    OBJECT_STATUS_CAN_STEALTH,      OBJECT_STATUS_STEALTHED,        OBJECT_STATUS_DETECTED,
    OBJECT_STATUS_IS_AIMING_WEAPON, OBJECT_STATUS_BRAKING,       OBJECT_STATUS_UNDER_CONSTRUCTION,
    OBJECT_STATUS_SOLD};
static const int kStatusCount = 10;

static const Real kVelocities[][3] = {{0, 0, 0},    {0.1f, 0, 0}, {0, 1.0f, 0}, {1.01f, 0, 0},
                                      {0, 0, 4.9f}, {3, 4, 0},    {6, 6, 6},    {0.07f, 0.07f, 0}};
static const int kVelocityCount = 8;

static UnsignedInt disguised(int player) {
    return ThePlayerList->getNthPlayer(player)->getAcademyStats()->*access_member(StDisguisedCountTag());
}

struct Scene {
    oracle_wp51::World& w;
    Object* o[kObjects] = {};
    StealthUpdate* st[3] = {};
    StealthDetectorUpdate* det = nullptr;
    std::vector<std::string> last;
    UnsignedInt base[2] = {0, 0};

    UpdateModule* module(int i) const {
        return i < 3 ? static_cast<UpdateModule*>(st[i]) : static_cast<UpdateModule*>(det);
    }

    // One string per observed thing; an unchanged one is written as "=".
    std::vector<std::string> observe() const {
        std::vector<std::string> out;
        for (int i = 0; i < 4; ++i) {
            UpdateModule* m = module(i);
            std::ostringstream s;
            if (m) {
                s << oracle_wp51::World::snapshot_save_hex(m) << ' ' << oracle_wp51::World::snapshot_crc(m)
                  << ' ' << m->friend_getNextCallFrame();
            }
            out.push_back(s.str());
        }
        for (int i = 0; i < kObjects; ++i) {
            std::ostringstream s;
            if (o[i]) {
                uint64_t bits = 0;
                const ObjectStatusMaskType mask = o[i]->getStatusBits();
                for (int b = 0; b < OBJECT_STATUS_COUNT; ++b)
                    if (mask.test(b)) bits |= uint64_t(1) << b;
                const PhysicsBehavior* p = o[i]->getPhysics();
                uint32_t v[3] = {0, 0, 0};
                if (p) std::memcpy(v, p->getVelocity(), sizeof(v));
                const Drawable* d = o[i]->getDrawable();
                if (!d) {
                    std::fprintf(stderr, "object without a drawable\n");
                    std::exit(1);
                }
                s << bits << ' ' << (d->getModelConditionFlags().test(MODELCONDITION_DISGUISED) ? 1 : 0)
                  << (d->getModelConditionFlags().test(MODELCONDITION_GARRISONED) ? 1 : 0)
                  << ' ' << o[i]->getControllingPlayer()->getPlayerIndex() << ' ' << v[0] << ' '
                  << v[1] << ' ' << v[2];
            }
            out.push_back(s.str());
        }
        std::ostringstream g;
        // Players outlive the vectors, so the counts are relative to the vector's start.
        g << GetGameLogicRandomSeedCRC() << ' ' << disguised(1) - base[0] << ' '
          << disguised(2) - base[1];
        out.push_back(g.str());
        return out;
    }

    // "E" for each enemy AI woken by the op: the enemy unit, the stealth unit's own AI.
    std::string woken() const {
        std::string s;
        for (int i : {S, E}) {
            AIUpdateInterface* ai = o[i]->getAI();
            const bool woke = ai && ai->getNextMoodCheckTime() != 0xfffffffeu;
            s += woke ? '1' : '0';
        }
        return s;
    }

    void arm_wake() const;

    std::string row() {
        const std::vector<std::string> now = observe();
        std::string s;
        for (size_t i = 0; i < now.size(); ++i) {
            if (i) s += ';';
            s += (i < last.size() && last[i] == now[i]) ? std::string("=") : now[i];
        }
        last = now;
        return s;
    }
};

struct StMoodTag { using type = UnsignedInt AIUpdateInterface::*; };
template struct MemberAccess<StMoodTag, &AIUpdateInterface::m_nextMoodCheckTime>;
UnsignedInt AIUpdateInterface::* access_member(StMoodTag);

void Scene::arm_wake() const {
    for (int i : {S, E}) {
        if (AIUpdateInterface* ai = o[i]->getAI()) ai->*access_member(StMoodTag()) = 0xfffffffeu;
    }
}

static bool contains(Object* container, Object* rider) {
    return rider->getContainedBy() == container;
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_pkt374_373_stealth <out.json>\n");
        return 2;
    }
    oracle_wp51::World w;
    TheFramePacer = new FramePacer;

    std::string objects;
    for (int i = 0; i < kStealth; ++i) objects += stealth_object(i);
    for (int i = 0; i < kRider; ++i) objects += rider_object(i);
    for (int i = 0; i < kDisguiser; ++i) objects += disguiser_object(i);
    for (int i = 0; i < kDetector; ++i) objects += detector_object(i);
    objects += fixed_objects();
    w.load_ini(kWeaponIni);
    w.load_ini(objects);
    w.init_with_neutral_player();
    // A local player that owns nothing: client feedback stays off, and shroud changes
    // never reach the display, which does not exist headless.
    ThePlayerList->setLocalPlayer(ThePlayerList->getNthPlayer(5));

    const std::string tmp = std::string(argv[1]) + ".tmp";
    FILE* out = std::fopen(tmp.c_str(), "w");
    if (!out) return 1;
    const int kTotal = 1600;
    std::fprintf(out, "{\"vector_count\":%d,\n\"weapon_ini\":\"%s\",\n\"object_ini\":\"%s\",\n\"vectors\":[",
                 kTotal, one_line(kWeaponIni).c_str(), one_line(objects).c_str());

    for (int n = 0; n < kTotal; ++n) {
        const int sv = n % kStealth;
        const StealthVariant& v = g_sv[sv];
        const int rv = pick(kRider);
        const int xv = pick(kDisguiser);
        const int dv = pick(kDetector);
        const int dteam = chance(75) ? 2 : pick(3);  // 0 neutral, 1 A, 2 B
        const uint32_t seed = 1 + pick(0x7ffffffe);
        UnsignedInt frame = chance(8) ? pick(3) : 1 + pick(5000);
        w.set_frame(frame);

        Team* teams[3] = {w.team_neutral, w.team_a, w.team_b};
        // How players A and B see each other: mostly enemies.
        const Relationship rel = chance(75) ? ENEMIES : (chance(50) ? NEUTRAL : ALLIES);
        ThePlayerList->getNthPlayer(1)->setPlayerRelationship(ThePlayerList->getNthPlayer(2), rel);
        ThePlayerList->getNthPlayer(2)->setPlayerRelationship(ThePlayerList->getNthPlayer(1), rel);
        Scene sc{w};
        sc.base[0] = disguised(1);
        sc.base[1] = disguised(2);
        int pos[kObjects];
        for (int i = 0; i < kObjects; ++i) pos[i] = chance(35) ? 0 : static_cast<int>(pick(kPositions));
        pos[S] = 0;

        auto place = [&](int i, int p) {
            Coord3D c = {200.0f + kOffsets[p][0], 200.0f + kOffsets[p][1], kOffsets[p][2]};
            sc.o[i]->setPosition(&c);
        };
        auto make = [&](int i, const std::string& name, Team* team) {
            Coord3D c = {200.0f + kOffsets[pos[i]][0], 200.0f + kOffsets[pos[i]][1], kOffsets[pos[i]][2]};
            sc.o[i] = w.spawn_object(name.c_str(), team, c);
            if (!sc.o[i]) {
                std::fprintf(stderr, "cannot spawn %s\n", name.c_str());
                std::exit(1);
            }
        };
        InitRandom(seed);
        make(S, "P374_S" + std::to_string(sv), w.team_a);
        if (v.transport) {
            InitRandom(seed + 1);
            make(R, "P374_R" + std::to_string(rv), w.team_a);
        }
        InitRandom(seed + 2);
        make(X, "P374_X" + std::to_string(xv), w.team_b);
        InitRandom(seed + 3);
        make(D, "P373_D" + std::to_string(dv), teams[dteam]);
        make(E, "P374_Enemy", w.team_b);
        make(M, "P374_Market", w.team_a);
        make(G, "P374_House", w.team_neutral);
        make(T, "P374_Truck", teams[dteam]);
        for (int i = 0; i < 3; ++i) sc.st[i] = find_behavior<StealthUpdate>(sc.o[i]);
        sc.det = find_behavior<StealthDetectorUpdate>(sc.o[D]);
        if (!sc.st[S] || !sc.st[X] || !sc.det) {
            std::fprintf(stderr, "missing module in vector %d\n", n);
            return 1;
        }
        if (v.transport) sc.o[S]->getContain()->addToContain(sc.o[R]);
        ThePartitionManager->update();
        InitRandom(seed + 4);

        std::fprintf(out,
                     "%s\n{\"v\":%d,\"seed\":%u,\"frame\":%u,\"sv\":%d,\"rv\":%d,\"xv\":%d,\"dv\":%d,"
                     "\"dteam\":%d,\"rel\":%d,\"rider\":%d,\"weapons\":%u,\"pos\":[",
                     n ? "," : "", n, seed, frame, sv, rv, xv, dv, dteam, static_cast<int>(rel),
                     v.transport ? 1 : 0,
                     v.weapons);
        for (int i = 0; i < kObjects; ++i) std::fprintf(out, "%s%d", i ? "," : "", pos[i]);
        std::fprintf(out, "],\"spawn\":\"%s\",\"ops\":[", sc.row().c_str());

        std::vector<std::string> rows;
        const int ops = 8 + static_cast<int>(pick(22));
        // Some vectors open by hiding the unit in the house, then revealing it there.
        const bool directed = sv % 8 == 0 && chance(65);
        const bool hidden_house = !directed && !v.transport && chance(8);
        for (int k = 0; k < ops; ++k) {
            // Weighted towards stepping, so the timers run out.
            static const int kCodes[] = {0, 0, 0, 0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  1,  1,
                                         1, 1, 1, 2,  2,  3,  3,  4,  4,  5,  6,  6,  7,  7,  7,
                                         8, 8, 8, 8,  9,  9,  10, 11, 11, 11, 12, 13, 13, 13, 14,
                                         15, 15};
            int code = kCodes[pick(sizeof(kCodes) / sizeof(kCodes[0]))];
            static const int kOpening[] = {13, 0, 6, 0, 0};
            static const int kHouse[] = {13, 1, 0, 0};
            static const int kRiders[] = {0, 1, 0, 0};
            const bool riding = v.transport && k < 4 && n % 2;
            const bool forced = (directed && k < 5) || (hidden_house && k < 4) || riding;
            if (forced) code = riding ? kRiders[k] : directed ? kOpening[k] : kHouse[k];
            uint32_t a = 0, b = 0;
            sc.arm_wake();
            w.set_frame(frame);
            const int present[] = {S, R, X, D, E, M};
            auto any_object = [&]() {
                for (;;) {
                    const int i = chance(50) ? S : present[pick(6)];
                    if (sc.o[i]) return static_cast<uint32_t>(i);
                }
            };
            auto stealth_object_index = [&]() {
                if (v.transport && chance(30)) return static_cast<uint32_t>(R);
                return static_cast<uint32_t>(chance(80) ? S : X);
            };
            switch (code) {
            case 0: {  // run `a` frames
                a = chance(60) ? 1 + pick(3) : 1 + pick(chance(80) ? 20 : 90);
                for (uint32_t t = 0; t < a; ++t) {
                    ++frame;
                    w.set_frame(frame);
                    ThePartitionManager->update();
                    for (int i = 0; i < 4; ++i)
                        if (sc.module(i)) step(w, sc.module(i), frame);
                }
                break;
            }
            case 1: {  // toggle a status
                b = chance(12) ? static_cast<uint32_t>(chance(50) ? G : T) : any_object();
                a = pick(kStatusCount);
                // The building states switch a detector off for good: keep them rare.
                if (a >= 8 && chance(60)) a = pick(8);
                if (forced) {
                    b = riding ? R : G;
                    a = riding ? 2 : 4;  // IS_ATTACKING, or STEALTHED
                }
                // `setStatus` also detonates a mine that goes under construction (unported).
                if (kStatuses[a] == OBJECT_STATUS_UNDER_CONSTRUCTION && sc.o[S]->isKindOf(KINDOF_MINE))
                    a = 0;
                const ObjectStatusTypes bit = kStatuses[a];
                sc.o[b]->setStatus(MAKE_OBJECT_STATUS_MASK(bit), !sc.o[b]->testStatus(bit));
                break;
            }
            case 2: {  // damage or heal
                b = any_object();
                a = chance(75) ? 0 : 1;
                DamageInfo info;
                info.in.m_damageType = a ? DAMAGE_HEALING : DAMAGE_UNRESISTABLE;
                info.in.m_deathType = DEATH_NORMAL;
                info.in.m_amount = 1.0f;
                info.in.m_sourceID = INVALID_ID;
                if (a) sc.o[b]->attemptHealing(1.0f, sc.o[E]);
                else sc.o[b]->attemptDamage(&info);
                break;
            }
            case 3: {  // velocity
                b = any_object();
                a = pick(kVelocityCount);
                if (PhysicsBehavior* p = sc.o[b]->getPhysics()) {
                    Coord3D& vel = p->*access_member(StVelTag());
                    vel.x = kVelocities[a][0];
                    vel.y = kVelocities[a][1];
                    vel.z = kVelocities[a][2];
                    p->*access_member(StVelMagTag()) = -1.0f;
                }
                break;
            }
            case 4: {  // weapon slot fired `ago` frames back
                const uint32_t slot = pick(3);
                const uint32_t ago = pick(3);
                const uint32_t firing = chance(60) ? 1 : 0;
                a = slot | (ago << 4) | (firing << 8);
                b = S;
                if (firing) sc.o[S]->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_IS_FIRING_WEAPON));
                Weapon* gun = sc.o[S]->getWeaponInWeaponSlot(static_cast<WeaponSlotType>(slot));
                if (gun) gun->*access_member(StFireFrameTag()) = frame >= ago ? frame - ago : 0;
                break;
            }
            case 5: {
                b = any_object();
                const Bool on = sc.o[b]->testScriptStatusBit(OBJECT_STATUS_SCRIPT_UNSTEALTHED);
                sc.o[b]->setScriptStatus(OBJECT_STATUS_SCRIPT_UNSTEALTHED, !on);
                break;
            }
            case 6: {  // markAsDetected
                b = forced ? static_cast<uint32_t>(S) : stealth_object_index();
                a = chance(50) ? 0 : 1 + pick(40);
                sc.st[b]->markAsDetected(a);
                break;
            }
            case 7: {  // receiveGrant
                b = stealth_object_index();
                const uint32_t active = chance(75) ? 1 : 0;
                const uint32_t frames = chance(35) ? 0 : 1 + pick(8);
                a = active | (frames << 1);
                sc.st[b]->receiveGrant(active != 0, frames);
                break;
            }
            case 8: {  // disguiseAsObject: 0 none, else an object index
                b = chance(80) ? S : X;
                const uint32_t targets[] = {0, 0, 0, E, X, X, S, G, M};
                a = targets[pick(9)];
                // Without physics the disguise change dereferences null.
                if (!sc.o[b]->getPhysics()) b = X;
                sc.st[b]->disguiseAsObject(a ? sc.o[a] : nullptr);
                break;
            }
            case 9: {  // current victim of the stealth unit
                const uint32_t targets[] = {0, E, E, X, M};
                a = targets[pick(5)];
                if (AIUpdateInterface* ai = sc.o[S]->getAI()) ai->setCurrentVictim(a ? sc.o[a] : nullptr);
                break;
            }
            case 10: {  // last command source
                a = chance(40) ? CMD_FROM_PLAYER : (chance(50) ? CMD_FROM_AI : CMD_FROM_SCRIPT);
                if (AIUpdateInterface* ai = sc.o[S]->getAI())
                    ai->friend_setLastCommandSource(static_cast<CommandSourceType>(a));
                break;
            }
            case 11: {  // move
                const int movers[] = {S, D, E, X, R};
                b = movers[pick(4)];
                a = pick(kPositions);
                if (!sc.o[b]->getContainedBy()) place(b, a);
                break;
            }
            case 12: {  // setSDEnabled
                a = chance(60) ? 1 : 0;
                sc.det->setSDEnabled(a != 0);
                break;
            }
            case 13: {  // containment: a = container (0 leaves), b = unit; bit 8 exposes
                const bool detector = !forced && chance(35);
                b = detector ? D : S;
                Object* unit = sc.o[b];
                Object* holder = unit->getContainedBy();
                if (holder) {
                    const uint32_t expose = chance(60) ? 1 : 0;
                    a = expose << 8;
                    holder->getContain()->removeFromContain(unit, expose != 0);
                } else if (b == S && !v.transport) {
                    // The house takes one side at a time.
                    if (!contains(sc.o[G], sc.o[D])) {
                        a = G;
                        sc.o[G]->getContain()->addToContain(unit);
                    }
                } else if (b == D) {
                    a = (chance(50) && !contains(sc.o[G], sc.o[S])) ? G : T;
                    sc.o[a]->getContain()->addToContain(unit);
                }
                break;
            }
            case 14: {  // toggle effectively dead
                b = chance(60) ? M : any_object();
                if (chance(70) && b != M) b = M;
                sc.o[b]->setEffectivelyDead(!sc.o[b]->isEffectivelyDead());
                break;
            }
            case 15: {  // save, load back, loadPostProcess
                b = chance(25) ? D : stealth_object_index();
                Snapshot* snap = sc.module(b);
                oracle::MemoryXfer xs(XFER_SAVE);
                xs.open("save");
                oracle::call_snapshot_xfer(snap, &xs);
                xs.close();
                // Loading drops the scheduler index; the real load rebuilds the heap.
                const Int index = sc.module(b)->*access_member(StIndexTag());
                oracle_wp51::World::snapshot_load_bytes(snap, xs.bytes);
                sc.module(b)->*access_member(StIndexTag()) = index;
                (snap->*access_member(StLoadPostTag()))();
                break;
            }
            }
            std::fprintf(out, "%s[%d,%u,%u]", k ? "," : "", code, a, b);
            rows.push_back(sc.woken() + ";" + sc.row());
        }
        std::fprintf(out, "],\"rows\":[");
        for (size_t r = 0; r < rows.size(); ++r)
            std::fprintf(out, "%s\"%s\"", r ? "," : "", rows[r].c_str());
        std::fprintf(out, "]}");

        for (int i = kObjects - 1; i >= 0; --i)
            if (sc.o[i]) TheGameLogic->destroyObject(sc.o[i]);
        w.flush_destroy_list();
        ThePartitionManager->update();
    }
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    return std::rename(tmp.c_str(), argv[1]) == 0 ? 0 : 1;
}
