// SPDX-License-Identifier: GPL-3.0-or-later
// Direct oracle for targeting and mood (docs/AI_PORT_PLAN.md, slice S12): the
// unmodified reference `AI::findClosestEnemy` / `findClosestAlly` /
// `findClosestRepulsor` / `getAdjustedVisionRangeForObject` (AI.cpp) and the mood
// functions of `AIUpdateInterface` (AIUpdate.cpp), over random crowds of real
// objects with the real AIUpdateInterface.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_s12_targeting.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/S12_Targeting.json
//
// The only override that changes behaviour is the terrain's line of sight, a hash
// of the two end points that the Rust test computes the same way (so the end
// points are compared). What the Rust port takes from a tape is recorded per
// (asking unit, other unit): getAbleToAttackSpecificObject, isWithinAttackRange
// of any weapon, isViewBlockedByObstacle, the shroud status, and per asking unit
// getLargestWeaponRange and the team's common target. A logging PartitionFilter
// passed to findClosestEnemy records which objects reach it, in order.
//
// One line is one crowd: the objects, then for some of them the tape and the
// queries with their answers.

#include "oracle_wp51_world.h"

#include "Common/Team.h"
#include "GameLogic/AIStateMachine.h"
#include "GameLogic/LogicRandomValue.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/StealthUpdate.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/ScriptEngine.h"
#include "GameLogic/Weapon.h"

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <fstream>
#include <map>
#include <set>
#include <sstream>
#include <string>
#include <vector>

// A local player of no concern, so shroud changes never reach display or radar.
struct LocalPlayerTag { using type = Player* PlayerList::*; };
template struct MemberAccess<LocalPlayerTag, &PlayerList::m_local>;
LocalPlayerTag::type access_member(LocalPlayerTag);

struct AttitudeTag { using type = AttitudeType AIUpdateInterface::*; };
template struct MemberAccess<AttitudeTag, &AIUpdateInterface::m_attitude>;
AttitudeTag::type access_member(AttitudeTag);

struct NextMoodTag { using type = UnsignedInt AIUpdateInterface::*; };
template struct MemberAccess<NextMoodTag, &AIUpdateInterface::m_nextMoodCheckTime>;
NextMoodTag::type access_member(NextMoodTag);

struct RandomOffsetTag { using type = Bool AIUpdateInterface::*; };
template struct MemberAccess<RandomOffsetTag, &AIUpdateInterface::m_randomlyOffsetMoodCheck>;
RandomOffsetTag::type access_member(RandomOffsetTag);

struct TurretsTag { using type = TurretAI* (AIUpdateInterface::*)[MAX_TURRETS]; };
template struct MemberAccess<TurretsTag, &AIUpdateInterface::m_turretAI>;
TurretsTag::type access_member(TurretsTag);

struct DisguiseTemplateTag { using type = const ThingTemplate* StealthUpdate::*; };
template struct MemberAccess<DisguiseTemplateTag, &StealthUpdate::m_disguiseAsTemplate>;
DisguiseTemplateTag::type access_member(DisguiseTemplateTag);

struct DisguisePlayerTag { using type = Int StealthUpdate::*; };
template struct MemberAccess<DisguisePlayerTag, &StealthUpdate::m_disguiseAsPlayerIndex>;
DisguisePlayerTag::type access_member(DisguisePlayerTag);

struct ModuleDataTag { using type = const AIUpdateModuleData* (AIUpdateInterface::*)() const; };
template struct MemberAccess<ModuleDataTag, &AIUpdateInterface::getAIUpdateModuleData>;
ModuleDataTag::type access_member(ModuleDataTag);

struct MachineTag { using type = AIStateMachine* AIUpdateInterface::*; };
template struct MemberAccess<MachineTag, &AIUpdateInterface::m_stateMachine>;
MachineTag::type access_member(MachineTag);

static void die(const char* what) {
    std::fprintf(stderr, "oracle: %s\n", what);
    std::exit(1);
}

static uint32_t bits(float f) {
    uint32_t u;
    std::memcpy(&u, &f, 4);
    return u;
}

// ---------------------------------------------------------------------------
// Line of sight: a hash of the end points.
// ---------------------------------------------------------------------------

static int g_los_calls = 0;

class LosTerrain : public oracle_wp51::OracleTerrainLogic {
public:
    Bool isClearLineOfSight(const Coord3D& pos, const Coord3D& other) const override {
        ++g_los_calls;
        const float v[6] = {pos.x, pos.y, pos.z, other.x, other.y, other.z};
        uint32_t h = 2166136261u;
        for (float f : v) {
            h ^= bits(f);
            h *= 16777619u;
        }
        return (h >> 7) % 4 != 0;
    }
};

// ---------------------------------------------------------------------------
// Random values of the harness's own (never the logic stream).
// ---------------------------------------------------------------------------

static uint32_t g_state = 0x5312;

static int rnd(int n) {
    g_state = g_state * 1103515245u + 12345u;
    return static_cast<int>((g_state >> 16) % static_cast<uint32_t>(n));
}

static bool chance(int percent) { return rnd(100) < percent; }

template <class T, int N>
static T pick(const T (&items)[N]) {
    return items[rnd(N)];
}

// ---------------------------------------------------------------------------
// Templates
// ---------------------------------------------------------------------------

struct Kind {
    const char* name;
    const char* kind_of;
    const char* shape;
    int radius;
    int height;
    int vision;
    const char* weapons;  // WeaponSet lines
    const char* ai;       // module data lines, or null for no AI
    bool turret;
    bool air;
    const char* contain;  // behavior block
    const char* stealth;  // behavior block
};

#define W_GUN "    Weapon = PRIMARY S12_Gun\n"
#define W_SHORT "    Weapon = PRIMARY S12_Short\n"
#define W_AA "    Weapon = PRIMARY S12_AA\n    Weapon = SECONDARY S12_Gun\n"

#define GARRISON "  Behavior = GarrisonContain ModuleTag_Contain\n    ContainMax = 10\n    AllowInsideKindOf = INFANTRY\n  End\n"
#define TRANSPORT_NOFIRE "  Behavior = TransportContain ModuleTag_Contain\n    Slots = 5\n    PassengersAllowedToFire = No\n    AllowInsideKindOf = INFANTRY\n  End\n"
#define TRANSPORT_FIRE "  Behavior = TransportContain ModuleTag_Contain\n    Slots = 5\n    PassengersAllowedToFire = Yes\n    AllowInsideKindOf = INFANTRY\n  End\n"
#define STEALTH "  Behavior = StealthUpdate ModuleTag_Stealth\n  End\n"
#define STEALTH_GRANT "  Behavior = StealthUpdate ModuleTag_Stealth\n    GrantedBySpecialPower = Yes\n  End\n"
#define DISGUISE "  Behavior = StealthUpdate ModuleTag_Stealth\n    DisguisesAsTeam = Yes\n  End\n"

static const Kind kKinds[] = {
    {"S12_Tank", "SELECTABLE CAN_ATTACK VEHICLE", "CYLINDER", 7, 8, 120, W_GUN,
     "    AutoAcquireEnemiesWhenIdle = Yes\n    MoodAttackCheckRate = 500\n", true, false, "", ""},
    {"S12_Rifle", "SELECTABLE INFANTRY CAN_ATTACK ATTACK_NEEDS_LINE_OF_SIGHT STEALTH_GARRISON", "CYLINDER", 3, 10, 90, W_SHORT,
     "    AutoAcquireEnemiesWhenIdle = Yes Stealthed\n    MoodAttackCheckRate = 1000\n", false, false, "", STEALTH},
    {"S12_Stinger", "INFANTRY IGNORED_IN_GUI ATTACK_NEEDS_LINE_OF_SIGHT", "CYLINDER", 3, 6, 110, W_SHORT,
     "    AutoAcquireEnemiesWhenIdle = Yes NotWhileAttacking Attack_Buildings\n    MoodAttackCheckRate = 66\n", false, false, "", ""},
    {"S12_Flyer", "VEHICLE AIRCRAFT CAN_ATTACK", "SPHERE", 9, 9, 150, W_AA,
     "    AutoAcquireEnemiesWhenIdle = Yes\n    MoodAttackCheckRate = 2000\n", false, true, "", ""},
    {"S12_Civilian", "INFANTRY", "CYLINDER", 2, 8, 60, "",
     "", false, false, "", ""},
    {"S12_Crowd", "INFANTRY", "CYLINDER", 2, 8, 30, "", nullptr, false, false, "", ""},
    {"S12_House", "STRUCTURE IMMOBILE", "BOX", 12, 20, 40, "", nullptr, false, false, "", ""},
    {"S12_Defense", "STRUCTURE IMMOBILE FS_BASE_DEFENSE CAN_ATTACK", "CYLINDER", 8, 14, 160, W_GUN,
     "    AutoAcquireEnemiesWhenIdle = Yes Attack_Buildings\n    MoodAttackCheckRate = 100\n", true, false, "", ""},
    {"S12_Factory", "STRUCTURE IMMOBILE FS_WARFACTORY", "BOX", 20, 18, 80, "", nullptr, false, false, "", ""},
    {"S12_Bunker", "STRUCTURE IMMOBILE", "CYLINDER", 10, 12, 100, "", nullptr, false, false, GARRISON, ""},
    {"S12_Truck", "VEHICLE", "CYLINDER", 9, 10, 70, "",
     "    AutoAcquireEnemiesWhenIdle = Yes\n    MoodAttackCheckRate = 300\n", false, false, TRANSPORT_NOFIRE, ""},
    {"S12_Humvee", "VEHICLE CAN_ATTACK", "CYLINDER", 8, 9, 130, W_GUN,
     "    AutoAcquireEnemiesWhenIdle = Yes Stealthed\n    MoodAttackCheckRate = 233\n", true, false, TRANSPORT_FIRE, STEALTH_GRANT},
    {"S12_Spy", "VEHICLE DISGUISER", "CYLINDER", 6, 7, 100, W_SHORT,
     "    AutoAcquireEnemiesWhenIdle = No\n", false, false, "", DISGUISE},
    {"S12_Inert", "INERT", "SPHERE", 4, 4, 50, W_GUN,
     "    AutoAcquireEnemiesWhenIdle = Yes\n", false, false, "", ""},
    {"S12_Sneak", "INFANTRY CAN_ATTACK STEALTH_GARRISON", "CYLINDER", 2, 9, 100, W_SHORT,
     "    AutoAcquireEnemiesWhenIdle = Yes\n    MoodAttackCheckRate = 400\n", false, false, "", STEALTH},
    {"S12_BunkerDefense", "STRUCTURE IMMOBILE CAN_ATTACK", "CYLINDER", 10, 12, 100, W_GUN, nullptr, false, false, GARRISON, ""},
};
static const int kKindCount = static_cast<int>(sizeof(kKinds) / sizeof(kKinds[0]));

static const char* const kWeapons =
    "Weapon S12_Gun\n  PrimaryDamage = 10\n  PrimaryDamageRadius = 0\n  AttackRange = 120\n"
    "  DamageType = ARMOR_PIERCING\n  DeathType = NORMAL\n  DelayBetweenShots = 1000\n"
    "  ClipSize = 0\n  AntiGround = Yes\nEnd\n"
    "Weapon S12_Short\n  PrimaryDamage = 5\n  PrimaryDamageRadius = 0\n  AttackRange = 45\n"
    "  MinimumAttackRange = 10\n  DamageType = SMALL_ARMS\n  DeathType = NORMAL\n"
    "  DelayBetweenShots = 500\n  ClipSize = 0\n  AntiGround = Yes\nEnd\n"
    "Weapon S12_AA\n  PrimaryDamage = 8\n  PrimaryDamageRadius = 0\n  AttackRange = 90\n"
    "  DamageType = EXPLOSION\n  DeathType = NORMAL\n  DelayBetweenShots = 700\n  ClipSize = 0\n"
    "  AntiGround = No\n  AntiAirborneVehicle = Yes\nEnd\n";

static const char* const kLocomotors =
    "Locomotor S12_Walk\n  Surfaces = GROUND\n  Speed = 30\n  TurnRate = 180\n"
    "  Acceleration = 30\n  Braking = 30\n  MinSpeed = 0\n  Appearance = TWO_LEGS\nEnd\n"
    "Locomotor S12_Fly\n  Surfaces = AIR\n  Speed = 90\n  TurnRate = 180\n"
    "  Acceleration = 30\n  Braking = 30\n  MinSpeed = 0\n  Appearance = HOVER\nEnd\n";

// The part of a template the Rust side reads.
static std::string template_head(const Kind& k) {
    std::ostringstream s;
    s << "Object " << k.name << "\n  KindOf = " << k.kind_of << "\n  Geometry = " << k.shape
      << "\n  GeometryMajorRadius = " << k.radius << "\n  GeometryMinorRadius = " << k.radius
      << "\n  GeometryHeight = " << k.height << "\n  GeometryIsSmall = No\n  VisionRange = " << k.vision
      << "\n";
    return s.str();
}

static std::string fixture_ini(const Kind& k) {
    std::ostringstream s;
    s << template_head(k);
    if (k.weapons[0]) s << "  WeaponSet\n    Conditions = None\n" << k.weapons << "  End\n";
    s << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 500\n    InitialHealth = 500\n  End\n";
    if (k.ai) {
        s << "  Behavior = AIUpdateInterface ModuleTag_AI\n" << k.ai;
        if (k.turret)
            s << "    Turret\n      ControlledWeaponSlots = PRIMARY\n      TurretTurnRate = 60\n    End\n";
        s << "  End\n  Locomotor = SET_NORMAL " << (k.air ? "S12_Fly" : "S12_Walk") << "\n";
    }
    s << k.contain << k.stealth << "End\n";
    return s.str();
}

// ---------------------------------------------------------------------------
// One crowd
// ---------------------------------------------------------------------------

static std::ofstream g_out;
static std::map<std::string, int> g_stats;

struct Crowd {
    std::vector<Object*> objects;
    std::vector<int> kinds;
    std::map<ObjectID, int> index;

    int of(const Object* o) const {
        if (!o) return -1;
        auto it = index.find(o->getID());
        if (it == index.end()) die("unknown object");
        return it->second;
    }
};

class LogFilter : public PartitionFilter {
public:
    const Crowd* crowd = nullptr;
    std::set<int> reject;
    std::vector<int> log;

    Bool allow(Object* other) override {
        const int i = crowd->of(other);
        log.push_back(i);
        return reject.count(i) == 0;
    }
};

static std::string list(const std::vector<int>& v) {
    std::ostringstream s;
    s << "[";
    for (size_t i = 0; i < v.size(); ++i) s << (i ? "," : "") << v[i];
    s << "]";
    return s.str();
}

static Player* player(int i) { return ThePlayerList->getNthPlayer(i); }

static const float kRanges[] = {0.0f, 12.0f, 40.0f, 75.0f, 130.0f, 220.0f, 400.0f, 1000000.0f};
static const int kHumanChance = 45;

static void run_crowd(oracle_wp51::World& w, int case_no) {
    Team* teams[3] = {w.team_neutral, w.team_a, w.team_b};
    const UnsignedInt frame = 100 + rnd(5000);
    w.set_frame(frame);

    // Players: type and mutual relationships.
    int human[3];
    std::vector<int> relations;
    for (int p = 0; p < 3; ++p) {
        human[p] = chance(kHumanChance) ? 1 : 0;
        player(p)->setPlayerType(human[p] ? PLAYER_HUMAN : PLAYER_COMPUTER, FALSE);
    }
    for (int a = 0; a < 3; ++a) {
        for (int b = 0; b < 3; ++b) {
            Relationship r = ALLIES;
            if (a != b) {
                const int roll = rnd(100);
                r = roll < 65 ? ENEMIES : roll < 80 ? NEUTRAL : ALLIES;
            }
            player(a)->setPlayerRelationship(player(b), r);
            relations.push_back(static_cast<int>(r));
        }
    }

    // AI data.
    TAiData* data = const_cast<TAiData*>(TheAI->getAiData());
    static const float kModifiers[] = {0.0f, 0.5f, 1.0f, 1.25f, 2.0f};
    static const float kDistanceModifiers[] = {0.0f, 7.0f, 25.0f, 100.0f, 1000.0f};
    data->m_guardInnerModifierAI = pick(kModifiers);
    data->m_guardOuterModifierAI = pick(kModifiers) + 0.1f;
    data->m_guardInnerModifierHuman = pick(kModifiers);
    data->m_guardOuterModifierHuman = pick(kModifiers) + 0.05f;
    data->m_alertRangeModifier = 1.0f + 0.1f * rnd(8);
    data->m_aggressiveRangeModifier = 1.0f + 0.3f * rnd(6);
    data->m_attackPriorityDistanceModifier = pick(kDistanceModifiers);
    data->m_forceIdleFramesCount = rnd(40);
    data->m_enableRepulsors = chance(90);
    data->m_attackUsesLineOfSight = chance(80);
    data->m_attackIgnoreInsignificantBuildings = chance(40);

    // Shroud: each player sees all, nothing, or patches.
    for (int p = 0; p < 3; ++p) {
        if (chance(45))
            ThePartitionManager->revealMapForPlayer(p);
        else
            ThePartitionManager->shroudMapForPlayer(p);
    }

    // Objects.
    Crowd c;
    const float base_x = 300.0f + 10.0f * rnd(30);
    const float base_y = 300.0f + 10.0f * rnd(30);
    const int spread = chance(30) ? 4 : chance(50) ? 10 : 24;
    const int count = 3 + rnd(11);
    for (int i = 0; i < count; ++i) {
        const int k = rnd(kKindCount);
        const int team = chance(12) ? 0 : 1 + rnd(2);
        Coord3D pos;
        pos.x = base_x + 5.0f * (rnd(2 * spread + 1) - spread);
        pos.y = base_y + 5.0f * (rnd(2 * spread + 1) - spread);
        pos.z = chance(15) ? 5.0f * rnd(8) : 0.0f;
        if (chance(7)) {
            pos.x = -5.0f * (1 + rnd(4));  // off the map
            if (chance(50)) pos.y = base_y;
        }
        Object* o = w.spawn_object(kKinds[k].name, teams[team], pos, 0.0f);
        if (!o) die("spawn");
        c.index[o->getID()] = i;
        c.objects.push_back(o);
        c.kinds.push_back(k);
    }
    // Crowds near the edge of the map keep off-map objects within reach.
    if (chance(8)) {
        for (Object* o : c.objects) {
            Coord3D p = *o->getPosition();
            if (p.x > 0.0f) p.x -= base_x - 40.0f;
            o->setPosition(&p);
        }
    }

    // Riders. Sneaks stealth first so containers count them.
    std::vector<int> riders;
    for (int i = 0; i < count; ++i) {
        const Kind& k = kKinds[c.kinds[i]];
        const bool infantry = std::strstr(k.kind_of, "INFANTRY") != nullptr;
        if (infantry && k.stealth[0] && chance(60))
            c.objects[i]->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_STEALTHED), TRUE);
    }
    for (int i = 0; i < count; ++i) {
        const Kind& k = kKinds[c.kinds[i]];
        if (!k.contain[0] || !chance(70)) continue;
        for (int j = 0; j < count && static_cast<int>(riders.size()) < 6; ++j) {
            Object* r = c.objects[j];
            if (r->getContainedBy() || !std::strstr(kKinds[c.kinds[j]].kind_of, "INFANTRY")) continue;
            if (!chance(50)) continue;
            ContainModuleInterface* contain = c.objects[i]->getContain();
            if (!contain) die("no contain");
            contain->addToContain(r);
            riders.push_back(j);
        }
    }

    // Statuses, stealth, deaths, damage.
    for (int i = 0; i < count; ++i) {
        Object* o = c.objects[i];
        const bool sneaky = kKinds[c.kinds[i]].stealth[0] != 0;
        if (chance(sneaky ? 45 : 8)) o->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_STEALTHED), TRUE);
        if (chance(15)) o->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_DETECTED), TRUE);
        if (chance(7)) o->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_REPULSOR), TRUE);
        if (chance(4)) o->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_IS_USING_ABILITY), TRUE);
        if (chance(3)) o->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_NO_ATTACK), TRUE);
        if (StealthUpdate* st = o->getStealth()) {
            if (std::strstr(kKinds[c.kinds[i]].kind_of, "DISGUISER")) {
                if (chance(75)) {
                    o->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_STEALTHED), TRUE);
                    o->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_DISGUISED), TRUE);
                }
                if (chance(85)) {
                    st->*access_member(DisguiseTemplateTag()) = TheThingFactory->findTemplate("S12_Tank");
                    st->*access_member(DisguisePlayerTag()) = chance(10) ? -1 : rnd(3);
                }
            }
        }
    }
    for (int i = 0; i < count; ++i) {
        Object* o = c.objects[i];
        if (chance(30)) {
            DamageInfo info;
            info.in.m_damageType = chance(25) ? DAMAGE_HEALING : chance(50) ? DAMAGE_EXPLOSION : DAMAGE_SMALL_ARMS;
            info.in.m_deathType = DEATH_NORMAL;
            info.in.m_amount = 1.0f;
            info.in.m_sourceID = chance(85) ? c.objects[rnd(count)]->getID() : INVALID_ID;
            if (info.in.m_damageType == DAMAGE_HEALING)
                o->getBodyModule()->attemptHealing(&info);
            else
                o->getBodyModule()->attemptDamage(&info);
        }
        if (chance(8)) o->setEffectivelyDead(TRUE);
    }

    // Obstacles the pathfinder's view check can see.
    std::vector<Object*> walls;
    for (int i = 0; i < count; ++i) {
        if (std::strstr(kKinds[c.kinds[i]].kind_of, "STRUCTURE") && chance(60)) {
            TheAI->pathfinder()->addObjectToPathfindMap(c.objects[i]);
            walls.push_back(c.objects[i]);
        }
    }

    // Shroud patches around some objects.
    for (int i = 0; i < count; ++i) {
        if (chance(20)) {
            const Coord3D* p = c.objects[i]->getPosition();
            const int mask = 1 << rnd(3);
            ThePartitionManager->doShroudReveal(p->x, p->y, 30.0f + 10.0f * rnd(6), static_cast<PlayerMaskType>(mask));
        }
    }
    ThePartitionManager->update();

    // Team common targets.
    int common[3];
    for (int t = 0; t < 3; ++t) {
        TeamTemplateInfo* info = const_cast<TeamTemplateInfo*>(teams[t]->getPrototype()->getTemplateInfo());
        common[t] = chance(25) ? 1 : 0;
        info->m_attackCommonTarget = common[t];
        teams[t]->setTeamTargetObject(chance(70) ? c.objects[rnd(count)] : nullptr);
    }

    // Attack priority sets: one custom, plus the default.
    AttackPriorityInfo priorities;
    priorities.friend_setName("S12");
    const int default_priority = chance(20) ? 0 : 1 + rnd(6);
    priorities.friend_setDefaultPriority(default_priority);
    std::vector<std::string> prio_text;
    for (int k = 0; k < kKindCount; ++k) {
        if (!chance(40)) continue;
        const int p = chance(15) ? 0 : 1 + rnd(30);
        priorities.setPriority(TheThingFactory->findTemplate(kKinds[k].name), p);
        prio_text.push_back("[" + std::to_string(k) + "," + std::to_string(p) + "]");
    }

    // Attackers (their state machine is in an attack state).
    for (int i = 0; i < count; ++i) {
        AIUpdateInterface* ai = c.objects[i]->getAI();
        if (!ai) continue;
        static const int kAttitudes[] = {-2, -1, -1, 0, 0, 1, 2, 3};
        ai->setAttitude(static_cast<AttitudeType>(pick(kAttitudes)));
        if (!c.objects[i]->isEffectivelyDead() && chance(15)) {
            if (chance(60)) {
                Coord3D at = *c.objects[i]->getPosition();
                at.x += 30.0f;
                ai->aiAttackPosition(&at, NO_MAX_SHOTS_LIMIT, CMD_FROM_SCRIPT);
            } else {
                ai->aiBusy(CMD_FROM_SCRIPT);
            }
        }
    }

    std::ostringstream line;
    line << "{\"n\":" << case_no << ",\"frame\":" << frame << ",\"human\":[" << human[0] << "," << human[1] << ","
         << human[2] << "],\"rel\":" << list(relations);
    line << ",\"ai\":[" << bits(data->m_guardInnerModifierAI) << "," << bits(data->m_guardOuterModifierAI) << ","
         << bits(data->m_guardInnerModifierHuman) << "," << bits(data->m_guardOuterModifierHuman) << ","
         << bits(data->m_alertRangeModifier) << "," << bits(data->m_aggressiveRangeModifier) << ","
         << bits(data->m_attackPriorityDistanceModifier) << "," << data->m_forceIdleFramesCount << ","
         << (data->m_enableRepulsors ? 1 : 0) << "," << (data->m_attackUsesLineOfSight ? 1 : 0) << ","
         << (data->m_attackIgnoreInsignificantBuildings ? 1 : 0) << "]";
    line << ",\"prio\":[" << default_priority;
    for (const std::string& s : prio_text) line << "," << s;
    line << "]";

    // Objects, one array each:
    // kind team x y z status dead offmap container inpartition able vision
    line << ",\"objects\":[";
    for (int i = 0; i < count; ++i) {
        Object* o = c.objects[i];
        static const ObjectStatusTypes kStatus[] = {
            OBJECT_STATUS_STEALTHED, OBJECT_STATUS_DETECTED, OBJECT_STATUS_DISGUISED,
            OBJECT_STATUS_REPULSOR, OBJECT_STATUS_IS_USING_ABILITY, OBJECT_STATUS_NO_ATTACK};
        int status = 0;
        for (int s = 0; s < 6; ++s)
            if (o->testStatus(kStatus[s])) status |= 1 << s;
        int team = -1;
        for (int t = 0; t < 3; ++t)
            if (o->getTeam() == teams[t]) team = t;
        if (team < 0) die("team");
        const Coord3D* p = o->getPosition();
        line << (i ? "," : "") << "[" << c.kinds[i] << "," << team << "," << bits(p->x) << "," << bits(p->y) << ","
             << bits(p->z) << "," << status << "," << (o->isEffectivelyDead() ? 1 : 0) << ","
             << (o->isOffMap() ? 1 : 0) << "," << c.of(o->getContainedBy()) << ","
             << (o->friend_getPartitionData() ? 1 : 0) << "," << (o->isAbleToAttack() ? 1 : 0) << ","
             << bits(o->getVisionRange()) << "]";
        g_stats[o->isOffMap() ? "offmap" : "onmap"]++;
    }
    line << "]";

    // Containers: object, count, stealth units, garrisonable, passengers fire,
    // apparent player for observers 0..2, then the items.
    line << ",\"contain\":[";
    bool first = true;
    for (int i = 0; i < count; ++i) {
        ContainModuleInterface* ci = c.objects[i]->getContain();
        if (!ci) continue;
        line << (first ? "" : ",") << "[" << i << "," << ci->getContainCount() << "," << ci->getStealthUnitsContained()
             << "," << (ci->isGarrisonable() ? 1 : 0) << "," << (ci->isPassengerAllowedToFire() ? 1 : 0);
        first = false;
        for (int p = 0; p < 3; ++p) {
            const Player* apparent = ci->getApparentControllingPlayer(player(p));
            line << "," << (apparent ? apparent->getPlayerIndex() : -1);
        }
        const ContainedItemsList* items = ci->getContainedItemsList();
        if (items)
            for (Object* item : *items) line << "," << c.of(item);
        line << "]";
        if (ci->getStealthUnitsContained() > 0) g_stats["stealth_contain"]++;
    }
    line << "]";

    // Stealth modules: object, disguised, disguise player, granted.
    line << ",\"stealth\":[";
    first = true;
    for (int i = 0; i < count; ++i) {
        StealthUpdate* st = c.objects[i]->getStealth();
        if (!st) continue;
        line << (first ? "" : ",") << "[" << i << "," << (st->isDisguised() ? 1 : 0) << "," << st->getDisguisedPlayerIndex()
             << "," << (st->isGrantedBySpecialPower() ? 1 : 0) << "]";
        first = false;
    }
    line << "]";

    // Bodies: last damage type, source object.
    line << ",\"damage\":[";
    for (int i = 0; i < count; ++i) {
        const DamageInfo* info = c.objects[i]->getBodyModule()->getLastDamageInfo();
        Object* source = TheGameLogic->findObjectByID(info->in.m_sourceID);
        line << (i ? "," : "") << "[" << static_cast<int>(info->in.m_damageType) << "," << c.of(source) << "]";
    }
    line << "]";

    // Who asks: up to three, units with an AI preferred.
    std::vector<int> askers;
    for (int tries = 0; tries < 40 && askers.size() < 3; ++tries) {
        const int i = rnd(count);
        if (!c.objects[i]->getAI() && !chance(15)) continue;
        bool seen = false;
        for (int a : askers) seen = seen || a == i;
        if (!seen) askers.push_back(i);
    }

    line << ",\"askers\":[";
    bool first_asker = true;
    for (int me_i : askers) {
        Object* me = c.objects[me_i];
        AIUpdateInterface* ai = me->getAI();

        // The tape.
        std::vector<int> attack, within, blocked, fog;
        for (int j = 0; j < count; ++j) {
            Object* other = c.objects[j];
            attack.push_back(static_cast<int>(
                me->getAbleToAttackSpecificObject(ATTACK_NEW_TARGET, other, CMD_FROM_AI)));
            bool in_range = false;
            for (int s = 0; s < WEAPONSLOT_COUNT; ++s) {
                const Weapon* weapon = me->getWeaponInWeaponSlot(static_cast<WeaponSlotType>(s));
                if (weapon && weapon->isWithinAttackRange(me, other)) in_range = true;
            }
            within.push_back(in_range ? 1 : 0);
            blocked.push_back(TheAI->pathfinder()->isViewBlockedByObstacle(me, other) ? 1 : 0);
            fog.push_back(other->getShroudedStatus(me->getControllingPlayer()->getPlayerIndex()) == OBJECTSHROUD_CLEAR ? 1 : 0);
            g_stats["attack" + std::to_string(attack.back())]++;
            g_stats["blocked" + std::to_string(blocked.back())]++;
            g_stats["fog" + std::to_string(fog.back())]++;
            g_stats["within" + std::to_string(within.back())]++;
        }
        const float largest = me->getLargestWeaponRange();
        int team_target = -1;
        if (me->getTeam()->getPrototype()->getTemplateInfo()->m_attackCommonTarget)
            team_target = c.of(me->getTeam()->getTeamTargetObject());

        line << (first_asker ? "" : ",") << "{\"me\":" << me_i << ",\"attack\":" << list(attack)
             << ",\"within\":" << list(within) << ",\"blocked\":" << list(blocked) << ",\"fog\":" << list(fog)
             << ",\"largest\":" << bits(largest) << ",\"team\":" << team_target;
        first_asker = false;

        // The unit as the mood code sees it:
        // statemachine attitude surfaces turret attacking idle autoacquire rate prio
        const int prio_mode = ai ? rnd(3) : 0;
        if (ai) {
            ai->setAttackInfo(prio_mode == 2 ? &priorities
                              : prio_mode == 1 ? TheScriptEngine->getDefaultAttackInfo() : nullptr);
            const AIUpdateModuleData* md = (ai->*access_member(ModuleDataTag()))();
            TurretAI* turret = (ai->*access_member(TurretsTag()))[0];
            line << ",\"unit\":[" << (ai->*access_member(MachineTag()) ? 1 : 0) << "," << static_cast<int>(ai->getMoodMatrixValue())
                 << "," << static_cast<int>(ai->getLocomotorSet().getValidSurfaces()) << "," << (turret ? 1 : 0) << ","
                 << (ai->isAttacking() ? 1 : 0) << "," << (ai->isIdle() ? 1 : 0) << ","
                 << md->m_autoAcquireEnemiesWhenIdle << "," << md->m_moodAttackCheckRate << "," << prio_mode << ","
                 << ai->*access_member(NextMoodTag()) << "," << (ai->*access_member(RandomOffsetTag()) ? 1 : 0) << "]";
            if (ai->isAttacking()) g_stats["attacking"]++;
            if (!ai->isIdle()) g_stats["busy"]++;
        } else {
            line << ",\"unit\":[]";
        }
        // The mood matrix does not show the attitude of a human's unit.
        line << ",\"attitude\":" << (ai ? static_cast<int>(ai->*access_member(AttitudeTag())) : 0);

        line << ",\"queries\":[";
        bool first_query = true;
        auto sep = [&]() -> std::ostringstream& {
            line << (first_query ? "" : ",");
            first_query = false;
            return line;
        };

        // findClosestEnemy: [0, range, qualifiers, prio, reject, result, los, log...]
        for (int q = 0; q < 5; ++q) {
            const float range = chance(60) ? kRanges[4 + rnd(4)] : kRanges[rnd(8)];
            const UnsignedInt qualifiers = rnd(64) & (chance(50) ? 63 : rnd(64));
            const int mode = rnd(3);
            LogFilter filter;
            filter.crowd = &c;
            const bool with_filter = chance(80);
            std::vector<int> reject;
            for (int j = 0; j < count; ++j)
                if (chance(10)) {
                    filter.reject.insert(j);
                    reject.push_back(j);
                }
            const AttackPriorityInfo* info = mode == 2 ? &priorities
                                             : mode == 1 ? TheScriptEngine->getDefaultAttackInfo() : nullptr;
            g_los_calls = 0;
            Object* found = TheAI->findClosestEnemy(me, range, qualifiers, info, with_filter ? &filter : nullptr);
            sep() << "[0," << bits(range) << "," << qualifiers << "," << mode << "," << (with_filter ? 1 : 0) << ","
                  << list(reject) << "," << c.of(found) << "," << g_los_calls << "," << list(filter.log) << "]";
            g_stats[std::string(mode == 2 ? "enemy_prio_" : "enemy_") + (found ? "found" : "none")]++;
        }
        // findClosestAlly: [1, range, qualifiers, result, los]
        {
            const float range = kRanges[rnd(8)];
            const UnsignedInt qualifiers = chance(40) ? AI::CAN_SEE : 0;
            g_los_calls = 0;
            Object* found = TheAI->findClosestAlly(me, range, qualifiers);
            sep() << "[1," << bits(range) << "," << qualifiers << "," << c.of(found) << "," << g_los_calls << "]";
            g_stats[std::string("ally_") + (found ? "found" : "none")]++;
        }
        // findClosestRepulsor: [2, range, result]
        for (int q = 0; q < 2; ++q) {
            const float range = kRanges[2 + rnd(6)];
            Object* found = TheAI->findClosestRepulsor(me, range);
            sep() << "[2," << bits(range) << "," << c.of(found) << "]";
            g_stats[std::string("repulsor_") + (found ? "found" : "none")]++;
        }
        // getAdjustedVisionRangeForObject: [3, factors, bits]
        for (int factors = 0; factors < 8; ++factors) {
            const float range = TheAI->getAdjustedVisionRangeForObject(me, factors);
            sep() << "[3," << factors << "," << bits(range) << "]";
        }
        if (ai) {
            // getMoodMatrixValue and adjustments: [4, value, idle, move, attack, attackmove, stealthed]
            sep() << "[4," << ai->getMoodMatrixValue() << "," << ai->getMoodMatrixActionAdjustment(MM_Action_Idle) << ","
                  << ai->getMoodMatrixActionAdjustment(MM_Action_Move) << ","
                  << ai->getMoodMatrixActionAdjustment(MM_Action_Attack) << ","
                  << ai->getMoodMatrixActionAdjustment(MM_Action_AttackMove) << ","
                  << (ai->canAutoAcquireWhileStealthed() ? 1 : 0) << "]";

            UnsignedInt& next = ai->*access_member(NextMoodTag());
            Bool& offset = ai->*access_member(RandomOffsetTag());
            for (int q = 0; q < 8; ++q) {
                // A timer operation: [5, op, arg, next, offset]
                const int op = rnd(4);
                UnsignedInt arg = 0;
                if (op == 0) {
                    ai->wakeUpAndAttemptToTarget();
                } else if (op == 1) {
                    ai->resetNextMoodCheckTime();
                } else if (op == 2) {
                    arg = chance(70) ? frame - rnd(5) : frame + 1 + rnd(5);
                    ai->setNextMoodCheckTime(arg);
                } else {
                    // No operation: set the members directly.
                    arg = frame - rnd(3) + (chance(20) ? 3 : 0);
                    next = arg;
                    offset = chance(50);
                    sep() << "[7," << next << "," << (offset ? 1 : 0) << "]";
                }
                if (op != 3) sep() << "[5," << op << "," << arg << "," << next << "," << (offset ? 1 : 0) << "]";

                // getNextMoodTarget: [6, byai, idle, seed, result, next, offset, rngcrc]
                const Bool by_ai = chance(60);
                const Bool idle = chance(50);
                const UnsignedInt seed = 1 + rnd(100000);
                InitRandom(seed);
                Object* found = ai->getNextMoodTarget(by_ai, idle);
                sep() << "[6," << (by_ai ? 1 : 0) << "," << (idle ? 1 : 0) << "," << seed << "," << c.of(found) << ","
                      << next << "," << (offset ? 1 : 0) << "," << GetGameLogicRandomSeedCRC() << "]";
                g_stats[std::string(by_ai ? "mood_ai_" : "mood_direct_") + (found ? "found" : "none")]++;
            }
        }
        line << "]}";
    }
    line << "]}";
    g_out << (case_no ? ",\n" : "") << line.str();

    // Tear down.
    for (Object* o : c.objects) {
        if (AIUpdateInterface* ai = o->getAI()) ai->setAttackInfo(nullptr);
    }
    for (Object* wall : walls) TheAI->pathfinder()->removeObjectFromPathfindMap(wall);
    for (int t = 0; t < 3; ++t) teams[t]->setTeamTargetObject(nullptr);
    for (Object* o : c.objects)
        if (o->getContainedBy()) o->getContainedBy()->getContain()->removeFromContain(o);
    for (Object* o : c.objects) TheGameLogic->destroyObject(o);
    w.flush_destroy_list();
    ThePartitionManager->update();
}

int main(int argc, char** argv) {
    if (argc < 2) die("usage: gen <out.json>");
    oracle_wp51::World w;
    w.init_with_neutral_player();

    ThePlayerList->*access_member(LocalPlayerTag()) = NEW Player(99);

    TheTerrainLogic = NEW LosTerrain;
    TheTerrainLogic->init();

    w.load_ini(kWeapons);
    w.load_ini(kLocomotors);
    std::string rust_ini;
    for (int k = 0; k < kKindCount; ++k) {
        w.load_ini(fixture_ini(kKinds[k]));
        rust_ini += template_head(kKinds[k]) + "End\n";
    }
    for (int k = 0; k < kKindCount; ++k) {
        const ThingTemplate* t = TheThingFactory->findTemplate(kKinds[k].name);
        if (!t) die("template");
    }

    g_out.open(argv[1], std::ios::binary);
    if (!g_out) die("open");
    // First line: the templates.
    g_out << "[\n{\"ini\":\"";
    for (unsigned char ch : rust_ini) {
        char hex[3];
        std::snprintf(hex, sizeof hex, "%02x", ch);
        g_out << hex;
    }
    g_out << "\",\"names\":\"";
    for (int k = 0; k < kKindCount; ++k) g_out << (k ? " " : "") << kKinds[k].name;
    g_out << "\"},\n";

    for (int n = 0; n < 1000; ++n) run_crowd(w, n);
    g_out << "\n]\n";
    g_out.close();

    for (const auto& it : g_stats) std::fprintf(stderr, "stat %s %d\n", it.first.c_str(), it.second);
    return 0;
}
