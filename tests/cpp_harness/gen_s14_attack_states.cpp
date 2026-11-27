// SPDX-License-Identifier: GPL-3.0-or-later
// Golden trace vectors for the attack states (AI plan slice S14, first half):
// the unmodified reference AIAttackState with its pursue, approach, aim and fire
// states, weapon choice and the able-to-attack questions, entered by real attack
// and attack-move orders on the real pathfind map and stepped by the real
// GameLogic::update until units die.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_s14_attack_states.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/S14_AttackStates.json
//
// In place of computeAttackPath the attack branch of doPathfind gets FALSE, no
// path or a synthetic path; the other searches of slice S6 and
// getMoveAwayFromPath are answered with synthetic paths; getNextMoodTarget
// outside idle answers a random enemy. Calls and answers are in the row's tape,
// which the Rust suite replays. The terrain blocks sight where the middle of a
// line is on a cliff. No unit has a turret. S14_SCENE and S14_FRAME in the
// environment print that frame's AI save bytes.
//
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIUpdate.cpp.o __ZN17AIUpdateInterface10doPathfindEP25PathfindServicesInterface
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIPathfind.cpp.o __ZN10Pathfinder19getMoveAwayFromPathEP6ObjectS1_P4PathS1_S3_
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a PartitionManager.cpp.o __ZN16PartitionManager26getGroundOrStructureHeightEff
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIUpdate.cpp.o __ZN17AIUpdateInterface17getNextMoodTargetEbb
//
// Not covered: turrets, garrisons, bridges, squads, hunt and attack area, team
// common targets, a change of the victim's team, jets, loading a save.
#include "PreRTS.h"

#include "oracle_wp51_world.h"

#include "Common/FramePacer.h"
#include "Common/Player.h"
#include "Common/RandomValue.h"
#include "Common/Recorder.h"
#include "Common/StateMachine.h"
#include "Common/MessageStream.h"
#include "GameClient/Drawable.h"
#include "GameLogic/AI.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Weapon.h"
#include "GameLogic/WeaponSet.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/AIStateMachine.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Locomotor.h"
#include "GameLogic/LocomotorSet.h"
#include "GameLogic/LogicRandomValue.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Pathfinder/PathfindCell.h"
#include "GameLogic/Pathfinder/PathfindCellInfo.h"
#include "GameLogic/Pathfinder/Path.h"
#include "GameLogic/Pathfinder/PathNode.h"

#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <map>
#include <sstream>
#include <string>
#include <vector>

// --- member access ----------------------------------------------------------

#define S10_TAG(Tag, Type, Class, Member)                       \
    struct Tag { using type = Type Class::*; };                \
    template struct MemberAccess<Tag, &Class::Member>;         \
    Tag::type access_member(Tag);

using StateMap = std::map<StateID, State*>;
S10_TAG(StateMapTag, StateMap, StateMachine, m_stateMap)
S10_TAG(MachineTag, AIStateMachine*, AIUpdateInterface, m_stateMachine)
S10_TAG(GoalCellTag, ICoord2D, AIUpdateInterface, m_pathfindGoalCell)
S10_TAG(CurCellTag, ICoord2D, AIUpdateInterface, m_pathfindCurCell)
S10_TAG(IsBlockedTag, Bool, AIUpdateInterface, m_isBlocked)
S10_TAG(MaxBlockedTag, Real, AIUpdateInterface, m_curMaxBlockedSpeed)
S10_TAG(BumpTag, Real, AIUpdateInterface, m_bumpSpeedLimit)
S10_TAG(StampTag, UnsignedInt, AIUpdateInterface, m_pathTimestamp)
S10_TAG(RetryTag, Bool, AIUpdateInterface, m_retryPath)
S10_TAG(WaitingTag, Bool, AIUpdateInterface, m_waitingForPath)
S10_TAG(QueueFrameTag, UnsignedInt, AIUpdateInterface, m_queueForPathFrame)
S10_TAG(IgnoreTag, UnsignedInt, AIUpdateInterface, m_ignoreCollisionsUntil)
S10_TAG(VelTag, Coord3D, PhysicsBehavior, m_vel)

S10_TAG(WeaponSetTag, WeaponSet, Object, m_weaponSet)

// --- text -------------------------------------------------------------------

static std::string hex32(uint32_t v) {
    char b[16];
    std::snprintf(b, sizeof(b), "%08x", v);
    return b;
}
static std::string fx(float f) {
    uint32_t u;
    std::memcpy(&u, &f, 4);
    return hex32(u);
}
static std::string c3(const Coord3D& c) { return fx(c.x) + "," + fx(c.y) + "," + fx(c.z); }
static std::string num(long long v) { return std::to_string(v); }
static std::string who(const Object* o) { return o ? num(static_cast<int>(o->getID())) : std::string("0"); }
static uint64_t fnv(const std::string& s, uint64_t h = 1469598103934665603ull) {
    for (unsigned char c : s) {
        h ^= c;
        h *= 1099511628211ull;
    }
    return h;
}
static std::string hex64(uint64_t v) {
    char b[24];
    std::snprintf(b, sizeof(b), "%016llx", static_cast<unsigned long long>(v));
    return b;
}

static uint32_t g_state = 0x53394631u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return n ? g_state % n : 0;
}
static bool chance(int percent) { return static_cast<int>(pick(100)) < percent; }
static float between(float lo, float hi) { return lo + (hi - lo) * (static_cast<float>(pick(100000)) / 100000.0f); }

static std::string g_tape;
static void tape(const std::string& entry) {
    if (!g_tape.empty()) g_tape += ";";
    g_tape += entry;
}

// --- answers of the unported searches ---------------------------------------

// A bent two segment path, or none. The text is what the answer was.
static Path* synthetic_path(const Coord3D& from, const Coord3D& to, std::string& answer) {
    if (pick(6) == 0) {
        answer = "-";
        return nullptr;
    }
    Coord3D mid;
    mid.x = (from.x + to.x) * 0.5f + between(-15.0f, 15.0f);
    mid.y = (from.y + to.y) * 0.5f + between(-15.0f, 15.0f);
    mid.z = from.z;
    Path* path = newInstance(Path);
    path->prependNode(&to, LAYER_GROUND);
    path->prependNode(&mid, LAYER_GROUND);
    path->prependNode(&from, LAYER_GROUND);
    path->getFirstNode()->setNextOptimized(path->getFirstNode()->getNext());
    path->getFirstNode()->getNext()->setNextOptimized(path->getLastNode());
    answer = c3(from) + "," + c3(mid) + "," + c3(to);
    return path;
}

static std::string path_ends(Path* path) {
    if (!path) return "-";
    int count = 0;
    for (PathNode* n = path->getFirstNode(); n; n = n->getNext()) ++count;
    return num(count) + "," + c3(*path->getFirstNode()->getPosition()) + "," + c3(*path->getLastNode()->getPosition());
}

static Path* closest_answer(Object* obj, const Coord3D* from, Coord3D* to, Bool blocked, Real pathCostFactor) {
    Coord3D end;
    end.x = to->x + between(-12.0f, 12.0f);
    end.y = to->y + between(-12.0f, 12.0f);
    end.z = to->z;
    if (chance(40)) {
        // Stops well short, so that the states try again.
        const float part = between(0.1f, 0.7f);
        end.x = from->x + (to->x - from->x) * part;
        end.y = from->y + (to->y - from->y) * part;
    }
    std::string answer;
    Path* path = synthetic_path(*from, end, answer);
    std::string asked = "C," + who(obj) + "," + c3(*from) + "," + c3(*to) + "," + num(blocked) + "," + fx(pathCostFactor);
    if (path) *to = end;
    tape(asked + "=" + answer);
    return path;
}

static Path* patch_answer(const Object* obj, Path* originalPath, Bool blocked) {
    std::string answer = "-";
    Path* path = nullptr;
    if (originalPath) {
        path = synthetic_path(*obj->getPosition(), *originalPath->getLastNode()->getPosition(), answer);
    }
    tape("P," + who(obj) + "," + path_ends(originalPath) + "," + num(blocked) + "=" + answer);
    return path;
}

static Path* safe_answer(const Object* obj, const Coord3D* from, const Coord3D* pos1, const Coord3D* pos2,
                         Real radius) {
    Coord3D to = *from;
    to.x += between(-40.0f, 40.0f);
    to.y += between(-40.0f, 40.0f);
    std::string answer;
    Path* path = synthetic_path(*from, to, answer);
    tape("S," + who(obj) + "," + c3(*from) + "," + c3(*pos1) + "," + c3(*pos2) + "," + fx(radius) + "=" + answer);
    return path;
}

Path* Pathfinder::getMoveAwayFromPath(Object* obj, Object* otherObj, Path* pathToAvoid, Object* otherObj2,
                                      Path* pathToAvoid2) {
    Coord3D to = *obj->getPosition();
    to.x += between(-30.0f, 30.0f);
    to.y += between(-30.0f, 30.0f);
    std::string answer;
    Path* path = synthetic_path(*obj->getPosition(), to, answer);
    tape("M," + who(obj) + "," + who(otherObj) + "," + path_ends(pathToAvoid) + "," + who(otherObj2) + "," +
         path_ends(pathToAvoid2) + "=" + answer);
    return path;
}

class Services : public PathfindServicesInterface {
public:
    PathfindServicesInterface* real;
    Path* findPath(Object* obj, const LocomotorSet& set, const Coord3D* from, const Coord3D* to) override {
        return real->findPath(obj, set, from, to);
    }
    Path* findClosestPath(Object* obj, const LocomotorSet&, const Coord3D* from, Coord3D* to, Bool blocked,
                          Real factor, Bool) override {
        return closest_answer(obj, from, to, blocked, factor);
    }
    Path* findAttackPath(const Object*, const LocomotorSet&, const Coord3D*, const Object*, const Coord3D*,
                         const Weapon*) override {
        std::abort();
    }
    Path* patchPath(const Object* obj, const LocomotorSet&, Path* originalPath, Bool blocked) override {
        return patch_answer(obj, originalPath, blocked);
    }
    Path* findSafePath(const Object* obj, const LocomotorSet&, const Coord3D* from, const Coord3D* pos1,
                       const Coord3D* pos2, Real radius) override {
        return safe_answer(obj, from, pos1, pos2, radius);
    }
};

void real_do_pathfind(AIUpdateInterface*, PathfindServicesInterface*)
    __asm__("_oracle_real__ZN17AIUpdateInterface10doPathfindEP25PathfindServicesInterface");

// computeAttackPath: FALSE, TRUE without a path, or TRUE with a path short of
// the victim.
static Bool attack_answer(const Object* obj, const Object* victim, const Coord3D* victimPos, Path*& path) {
    const std::string asked = "A," + who(obj) + "," + who(victim) + "," + c3(*victimPos);
    if (chance(55)) {
        tape(asked + "=F");
        return FALSE;
    }
    std::string answer = "-";
    Path* next = nullptr;
    if (chance(80)) {
        const Coord3D from = *obj->getPosition();
        const float part = between(0.4f, 0.95f);
        Coord3D to;
        to.x = from.x + (victimPos->x - from.x) * part;
        to.y = from.y + (victimPos->y - from.y) * part;
        to.z = from.z;
        next = synthetic_path(from, to, answer);
    }
    if (path) deleteInstance(path);
    path = next;
    tape(asked + "=" + answer);
    return TRUE;
}

// The attack branch is the reference's `doPathfind`, repeated because its call to
// `computeAttackPath` stays inside the object file; the real function does the
// rest.
void AIUpdateInterface::doPathfind(PathfindServicesInterface* pathfinder) {
    static Services services;
    services.real = pathfinder;
    if (m_waitingForPath && !m_isSafePath && !m_isApproachPath && m_isAttackPath) {
        Object* victim = nullptr;
        if (m_requestedVictimID != INVALID_ID) victim = TheGameLogic->findObjectByID(m_requestedVictimID);
        if (attack_answer(getObject(), victim, &m_requestedDestination, m_path)) {
            m_waitingForPath = FALSE;
            if (getPath()) {
                TheAI->pathfinder()->updateGoal(getObject(), getPath()->getLastNode()->getPosition(),
                                                getPath()->getLastNode()->getLayer());
            }
            m_isAttackPath = TRUE;
            return;
        }
        m_isAttackPath = FALSE;
        if (victim) {
            m_requestedDestination = *victim->getPosition();
            TheAI->pathfinder()->adjustToPossibleDestination(getObject(), getLocomotorSet(),
                                                             &m_requestedDestination);
            ignoreObstacle(victim);
        }
    }
    real_do_pathfind(this, &services);
}

Real PartitionManager::getGroundOrStructureHeight(Real, Real) { return 0.0f; }

static std::vector<ObjectID> g_ids;
static int g_dump_unit = -1;
static std::string g_dump;

// A random enemy of `me`, or none.
static Object* some_enemy(const Object* me) {
    if (g_ids.empty()) return nullptr;
    const size_t start = pick(static_cast<uint32_t>(g_ids.size()));
    for (size_t k = 0; k < g_ids.size(); ++k) {
        Object* o = TheGameLogic->findObjectByID(g_ids[(start + k) % g_ids.size()]);
        if (o && o != me && me->getRelationship(o) == ENEMIES && !o->isEffectivelyDead()) return o;
    }
    return nullptr;
}

Object* AIUpdateInterface::getNextMoodTarget(Bool calledByAI, Bool calledDuringIdle) {
    if (calledDuringIdle) return nullptr;
    Object* target = chance(8) ? some_enemy(getObject()) : nullptr;
    tape("T," + who(getObject()) + "," + num(calledByAI ? 1 : 0) + "=" + who(target));
    return target;
}

// --- terrain ----------------------------------------------------------------

static const int kCells = 32;

class S14Terrain : public oracle_wp51::OracleTerrainLogic {
public:
    std::string cliff = std::string(kCells * kCells, '0');
    S14Terrain() {
        extent.lo = {0.0f, 0.0f, 0.0f};
        extent.hi = {kCells * 10.0f, kCells * 10.0f, 100.0f};
    }
    Bool isClearLineOfSight(const Coord3D& a, const Coord3D& b) const override {
        return !isCliffCell((a.x + b.x) * 0.5f, (a.y + b.y) * 0.5f);
    }
    Bool isCliffCell(Real x, Real y) const override {
        int i = static_cast<int>(std::floor(x / 10.0f));
        int j = static_cast<int>(std::floor(y / 10.0f));
        i = i < 0 ? 0 : (i >= kCells ? kCells - 1 : i);
        j = j < 0 ? 0 : (j >= kCells ? kCells - 1 : j);
        return cliff[j * kCells + i] == '1';
    }
};

static S14Terrain* g_terrain = nullptr;

// --- fixture ----------------------------------------------------------------

static std::string locomotor(const char* name, const char* appearance, int speed, int min_speed, const char* extra) {
    std::ostringstream s;
    s << "Locomotor " << name << "\n  Surfaces = " << (std::strstr(name, "Hover") ? "AIR" : "GROUND")
      << "\n  Appearance = " << appearance << "\n  Speed = " << speed
      << "\n  TurnRate = 200\n  Acceleration = 90\n  Braking = 90\n  MinSpeed = " << min_speed << "\n"
      << extra << "End\n";
    return s.str();
}

static std::string locomotors_ini() {
    return locomotor("S14_Legs", "TWO_LEGS", 30, 0, "") + locomotor("S14_Wheels", "FOUR_WHEELS", 55, 0, "") +
           locomotor("S14_Treads", "TREADS", 40, 0, "") + locomotor("S14_Creep", "FOUR_WHEELS", 45, 8, "") +
           locomotor("S14_Hover", "HOVER", 70, 0,
                     "  PreferredHeight = 30\n  Lift = 120\n  AllowAirborneMotiveForce = Yes\n"
                     "  ZAxisBehavior = SURFACE_RELATIVE_HEIGHT\n");
}

static std::string weapon(const char* name, int damage, int range, int delay, const char* extra) {
    std::ostringstream s;
    s << "Weapon " << name << "\n  PrimaryDamage = " << damage << "\n  AttackRange = " << range
      << "\n  DelayBetweenShots = " << delay << "\n  DamageType = "
      << (std::strstr(extra, "DamageType") ? "" : "SMALL_ARMS\n  DeathType = ") << "NORMAL\n"
      << extra << "End\n";
    return s.str();
}

static std::string weapons_ini() {
    std::string s;
    auto add = [&](const char* name, int damage, int range, int delay, const char* type, const char* extra) {
        std::ostringstream w;
        w << "Weapon " << name << "\n  PrimaryDamage = " << damage << "\n  AttackRange = " << range
          << "\n  DelayBetweenShots = " << delay << "\n  DamageType = " << type << "\n  DeathType = NORMAL\n"
          << "  WeaponSpeed = " << (std::strstr(extra, "WeaponSpeed") ? "" : "999999") << "\n";
        std::string text = w.str();
        if (std::strstr(extra, "WeaponSpeed")) text.erase(text.rfind("  WeaponSpeed = "));
        s += text + extra + "End\n";
    };
    add("W_Rifle", 7, 60, 300, "SMALL_ARMS",
        "  ClipSize = 4\n  ClipReloadTime = 900\n  AutoReloadsClip = Yes\n");
    add("W_Cannon", 24, 90, 800, "ARMOR_PIERCING",
        "  AcceptableAimDelta = 20\n  PreAttackDelay = 300\n  PreAttackType = PER_SHOT\n");
    add("W_Knife", 20, 6, 400, "SURRENDER", "");
    add("W_Splash", 18, 75, 500, "EXPLOSION",
        "  PrimaryDamageRadius = 12\n  SecondaryDamage = 6\n  SecondaryDamageRadius = 24\n"
        "  ContinueAttackRange = 90\n  RadiusDamageAffects = ENEMIES NEUTRALS\n");
    add("W_Long", 30, 130, 900, "ARMOR_PIERCING",
        "  MinimumAttackRange = 45\n  ClipSize = 2\n  ClipReloadTime = 1500\n  AutoReloadsClip = Yes\n"
        "  PreAttackDelay = 500\n  PreAttackType = PER_CLIP\n");
    add("W_Leech", 9, 45, 250, "SMALL_ARMS",
        "  LeechRangeWeapon = Yes\n  PreAttackDelay = 400\n  PreAttackType = PER_ATTACK\n");
    add("W_Spent", 12, 70, 200, "SMALL_ARMS",
        "  ClipSize = 3\n  AutoReloadsClip = No\n");
    add("W_Slow", 22, 100, 600, "SMALL_ARMS", "  WeaponSpeed = 240\n");
    add("W_Flak", 15, 110, 300, "SMALL_ARMS", "  AntiAirborneVehicle = Yes\n  AntiGround = No\n");
    add("W_Weak", 2, 50, 150, "SMALL_ARMS", "  AcceptableAimDelta = 90\n");
    return s;
}

static const char* const kArmorIni = "Armor E_Plain\n  Armor = DEFAULT 100%\nEnd\n";

struct Kind {
    const char* name;
    const char* kind_of;
    const char* locomotor;
    float radius;
    const char* weapons;
    int health;
};

#define GUN(slot, name) "    Weapon = " slot " " name "\n"
static const Kind kKinds[] = {
    {"A_Rifle", "SELECTABLE INFANTRY CAN_ATTACK", "S14_Legs", 4, GUN("PRIMARY", "W_Rifle"), 120},
    {"A_Tank", "SELECTABLE VEHICLE CAN_ATTACK ATTACK_NEEDS_LINE_OF_SIGHT", "S14_Treads", 8,
     GUN("PRIMARY", "W_Cannon") GUN("SECONDARY", "W_Rifle") "    PreferredAgainst = SECONDARY INFANTRY\n", 200},
    {"A_Knife", "SELECTABLE INFANTRY CAN_ATTACK", "S14_Legs", 4, GUN("PRIMARY", "W_Knife"), 120},
    {"A_Splash", "SELECTABLE VEHICLE CAN_ATTACK", "S14_Wheels", 6, GUN("PRIMARY", "W_Splash"), 140},
    {"A_Long", "SELECTABLE VEHICLE CAN_ATTACK", "S14_Treads", 7, GUN("PRIMARY", "W_Long"), 140},
    {"A_Leech", "SELECTABLE INFANTRY CAN_ATTACK", "S14_Legs", 4, GUN("PRIMARY", "W_Leech"), 120},
    {"A_Spent", "SELECTABLE VEHICLE CAN_ATTACK", "S14_Wheels", 5, GUN("PRIMARY", "W_Spent"), 120},
    {"A_Creep", "SELECTABLE VEHICLE CAN_ATTACK ATTACK_NEEDS_LINE_OF_SIGHT", "S14_Creep", 6,
     GUN("PRIMARY", "W_Slow") GUN("SECONDARY", "W_Weak") "    AutoChooseSources = SECONDARY FROM_AI\n", 150},
    {"A_Flak", "SELECTABLE VEHICLE CAN_ATTACK", "S14_Wheels", 6,
     GUN("PRIMARY", "W_Flak") GUN("SECONDARY", "W_Cannon"), 120},
    {"A_Hover", "SELECTABLE AIRCRAFT VEHICLE CAN_ATTACK", "S14_Hover", 6, GUN("PRIMARY", "W_Rifle"), 100},
    {"A_Dummy", "SELECTABLE VEHICLE", "S14_Treads", 7, "", 90},
    {"A_Bunker", "STRUCTURE IMMOBILE", "", 14, "", 150},
    {"A_Turretless", "STRUCTURE IMMOBILE CAN_ATTACK", "-", 10, GUN("PRIMARY", "W_Cannon"), 160},
    {"A_Portable", "PORTABLE_STRUCTURE CAN_ATTACK VEHICLE", "S14_Treads", 6, GUN("PRIMARY", "W_Weak"), 120},
};
static const int kKindCount = static_cast<int>(sizeof(kKinds) / sizeof(kKinds[0]));
enum { K_RIFLE, K_TANK, K_KNIFE, K_SPLASH, K_LONG, K_LEECH, K_SPENT, K_CREEP, K_FLAK, K_HOVER, K_DUMMY, K_BUNKER,
       K_TURRETLESS, K_PORTABLE };

static std::string objects_ini() {
    std::string s;
    for (int i = 0; i < kKindCount; ++i) {
        const Kind& k = kKinds[i];
        std::ostringstream o;
        o << "Object " << k.name << "\n  KindOf = PRELOAD " << k.kind_of << "\n  Geometry = CYLINDER\n"
          << "  GeometryMajorRadius = " << k.radius << "\n  GeometryHeight = 8\n  GeometryIsSmall = No\n"
          << "  VisionRange = 120\n  ShroudClearingRange = 0\n"
          << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = " << k.health << "\n    InitialHealth = "
          << k.health << "\n  End\n"
          << "  Behavior = DestroyDie ModuleTag_Die\n  End\n";
        if (k.locomotor[0] == '-') {
            o << "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n";
        } else if (k.locomotor[0]) {
            o << "  Behavior = PhysicsBehavior ModuleTag_Phys\n    Mass = 5\n  End\n"
              << "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
              << "  Locomotor = SET_NORMAL " << k.locomotor << "\n";
        }
        if (k.weapons[0]) o << "  WeaponSet\n    Conditions = None\n" << k.weapons << "  End\n";
        o << "End\n";
        s += o.str();
    }
    return s;
}

// --- observation ------------------------------------------------------------

static Object* unit(int i) {
    if (i < 0 || i >= static_cast<int>(g_ids.size())) return nullptr;
    return TheGameLogic->findObjectByID(g_ids[i]);
}

static std::string path_text(Path* path) {
    if (!path) return "-";
    std::vector<PathNode*> nodes;
    for (PathNode* n = path->getFirstNode(); n; n = n->getNext()) nodes.push_back(n);
    std::string s = num(static_cast<long long>(nodes.size()));
    for (PathNode* n : nodes) {
        int next = -1;
        for (size_t k = 0; k < nodes.size(); ++k)
            if (nodes[k] == n->getNextOptimized()) next = static_cast<int>(k);
        s += "/" + c3(*n->getPosition()) + "," + num(static_cast<int>(n->getLayer())) + "," + num(next);
    }
    return s;
}

// Health, status bits and each weapon's save bytes.
static std::string combat_text(Object* o) {
    std::string s = fx(o->getBodyModule()->getHealth()) + "|";
    const ObjectStatusMaskType status = o->getStatusBits();
    for (int b = 0; b < OBJECT_STATUS_COUNT; ++b)
        if (status.test(b)) s += num(b) + ".";
    s += "|";
    WeaponSet& set = o->*access_member(WeaponSetTag());
    s += num(static_cast<int>(set.getCurWeaponSlot()));
    for (int slot = 0; slot < WEAPONSLOT_COUNT; ++slot) {
        Weapon* weapon = set.getWeaponInWeaponSlot(static_cast<WeaponSlotType>(slot));
        s += ",";
        if (weapon) s += oracle_wp51::World::snapshot_save_hex(weapon);
    }
    return s;
}

// Fields are separated by '|' so a mismatch names the one that differs.
static std::string observe(int i) {
    Object* o = unit(i);
    if (!o) return "gone";
    AIUpdateInterface* ai = o->getAI();
    if (!ai) return c3(*o->getPosition()) + "|" + combat_text(o);
    uint32_t unused = 0;
    std::string save = oracle_wp51::World::snapshot_save_hex(ai, &unused);
    // `Path::xfer` writes an uninitialized word before its last byte; zero it.
    if (Path* path = ai->getPath()) {
        const std::string bytes = oracle_wp51::World::snapshot_save_hex(path, &unused);
        const size_t at = save.find(bytes.substr(0, bytes.size() - 10));
        if (at == std::string::npos) std::abort();
        for (size_t k = at + bytes.size() - 10; k < at + bytes.size() - 2; ++k) save[k] = '0';
    }
    if (g_dump_unit == i) g_dump = save;
    const uint32_t crc = oracle_wp51::World::snapshot_crc(ai);
    PhysicsBehavior* phys = o->getPhysics();
    const ICoord2D goal = ai->*access_member(GoalCellTag());
    const ICoord2D cur = ai->*access_member(CurCellTag());
    std::string s = c3(*o->getPosition());
    s += "|" + fx(o->getOrientation());
    s += "|" + (phys ? c3(phys->*access_member(VelTag())) : std::string("-"));
    s += "|" + num(ai->getNumFramesBlocked()) + num(ai->*access_member(IsBlockedTag()) ? 1 : 0) +
         num(ai->isBlockedAndStuck() ? 1 : 0) + num(ai->*access_member(RetryTag()) ? 1 : 0) +
         num(ai->*access_member(WaitingTag()) ? 1 : 0) + num(ai->isMoving() ? 1 : 0);
    s += "|" + fx(ai->*access_member(MaxBlockedTag())) + "," + fx(ai->*access_member(BumpTag()));
    s += "|" + num(ai->*access_member(StampTag())) + "," + num(ai->*access_member(QueueFrameTag())) + "," +
         num(ai->*access_member(IgnoreTag()));
    s += "|" + num(goal.x) + "," + num(goal.y) + "," + num(cur.x) + "," + num(cur.y) + "," +
         num(static_cast<int>(o->getLayer()));
    s += "|" + path_text(ai->getPath());
    std::string flags;
    if (Drawable* d = o->getDrawable()) {
        const ModelConditionFlags& bits = d->getModelConditionFlags();
        for (int b = 0; b < MODELCONDITION_COUNT; ++b)
            if (bits.test(b)) flags += num(b) + ".";
    }
    s += "|" + num(ai->getCurrentStateID()) + "," + flags;
    s += "|" + num(ai->isAttacking() ? 1 : 0) + "," + who(ai->getCurrentVictim()) + "," + who(ai->getGoalObject()) +
         "," + (ai->getCurrentVictimPos() ? c3(*ai->getCurrentVictimPos()) : std::string("-"));
    s += "|" + combat_text(o);
    s += "|" + hex32(crc);
    s += "|" + hex64(fnv(save));
    return s;
}

// Which unit stands in and wants which ground cell.
static std::string cells_digest() {
    std::string s;
    for (int y = 0; y < kCells; ++y)
        for (int x = 0; x < kCells; ++x) {
            PathfindCell* cell = TheAI->pathfinder()->getCell(LAYER_GROUND, x, y);
            if (!cell) continue;
            const int goal = static_cast<int>(cell->getGoalUnit());
            const int present = static_cast<int>(cell->getPosUnit());
            if (goal || present) s += num(x) + "," + num(y) + "," + num(goal) + "," + num(present) + ";";
        }
    return hex64(fnv(s));
}

// --- operations -------------------------------------------------------------

static Coord3D at(float x, float y) {
    Coord3D p;
    p.x = x;
    p.y = y;
    p.z = 0.0f;
    return p;
}

static float inside(float v) { return v < 15.0f ? 15.0f : (v > 305.0f ? 305.0f : v); }

static std::vector<std::string> g_ops;

// "<name> <unit> <player> <target or -1> <point or -> <shots>"
static void op_order(int u, const char* name, AICommandType type, bool player, int target, const Coord3D* point,
                     int shots) {
    Object* o = unit(u);
    if (!o || !o->getAI()) return;
    if (target >= 0 && !unit(target)) return;
    g_ops.push_back(std::string(name) + " " + num(u) + " " + num(player ? 1 : 0) + " " + num(target) + " " +
                    (point ? c3(*point) : std::string("-")) + " " + num(shots));
    AICommandParms parms(type, player ? CMD_FROM_PLAYER : CMD_FROM_AI);
    parms.m_obj = target >= 0 ? unit(target) : nullptr;
    if (point) {
        parms.m_pos = *point;
        parms.m_coords.push_back(*point);
    }
    parms.m_intValue = shots;
    o->getAI()->aiDoCommand(&parms);
}

static std::vector<int> g_teams;

static int some_other(int u, bool enemy) {
    const int n = static_cast<int>(g_ids.size());
    const int start = static_cast<int>(pick(n));
    for (int k = 0; k < n; ++k) {
        const int v = (start + k) % n;
        if (v != u && unit(v) && (g_teams[v] != g_teams[u]) == enemy) return v;
    }
    return -1;
}

static Coord3D some_point(const Object* o) {
    const float reach = chance(40) ? 40.0f : 140.0f;
    return at(inside(o->getPosition()->x + between(-reach, reach)),
              inside(o->getPosition()->y + between(-reach, reach)));
}

static void give_order(int u) {
    Object* o = unit(u);
    if (!o || !o->getAI()) return;
    const bool player = chance(55);
    const int shots = chance(85) ? NO_MAX_SHOTS_LIMIT : 1 + static_cast<int>(pick(3));
    const uint32_t r = pick(100);
    const int enemy = some_other(u, true);
    const Coord3D to = some_point(o);
    if (r < 48) {
        if (enemy >= 0) op_order(u, "attack", AICMD_ATTACK_OBJECT, player, enemy, nullptr, shots);
    } else if (r < 58) {
        const int v = chance(60) ? some_other(u, false) : enemy;
        if (v >= 0) op_order(u, "force", AICMD_FORCE_ATTACK_OBJECT, player, v, nullptr, shots);
    } else if (r < 70) {
        op_order(u, "attackpos", AICMD_ATTACK_POSITION, player, -1, &to, shots);
    } else if (r < 82) {
        op_order(u, "amove", AICMD_ATTACKMOVE_TO_POSITION, player, -1, &to, NO_MAX_SHOTS_LIMIT);
    } else if (r < 92) {
        op_order(u, "move", AICMD_MOVE_TO_POSITION, player, -1, &to, 0);
    } else {
        op_order(u, "idle", AICMD_IDLE, player, -1, nullptr, 0);
    }
}

static void set_status_bit(Object* o, ObjectStatusTypes bit, bool on) {
    o->setStatus(MAKE_OBJECT_STATUS_MASK(bit), on);
}

static void random_op(int units) {
    const int u = static_cast<int>(pick(units));
    Object* o = unit(u);
    if (!o) return;
    AIUpdateInterface* ai = o->getAI();
    const uint32_t r = pick(100);
    if (r < 56) {
        give_order(u);
    } else if (r < 66) {
        // A jump, often into or out of range.
        const int v = some_other(u, true);
        if (v < 0) return;
        Coord3D p = *unit(v)->getPosition();
        p.x = inside(p.x + between(-90.0f, 90.0f));
        p.y = inside(p.y + between(-90.0f, 90.0f));
        const float angle = between(-3.14f, 3.14f);
        g_ops.push_back("pos " + num(u) + " " + c3(p) + " " + fx(angle));
        o->setPosition(&p);
        o->setOrientation(angle);
    } else if (r < 76) {
        static const ObjectStatusTypes kBits[] = {OBJECT_STATUS_STEALTHED, OBJECT_STATUS_DETECTED,
                                                  OBJECT_STATUS_MASKED, OBJECT_STATUS_NO_ATTACK_FROM_AI,
                                                  OBJECT_STATUS_AIRBORNE_TARGET, OBJECT_STATUS_UNDER_CONSTRUCTION,
                                                  OBJECT_STATUS_NO_ATTACK};
        const int bit = static_cast<int>(pick(7));
        const int on = chance(55) ? 1 : 0;
        g_ops.push_back("status " + num(u) + " " + num(kBits[bit]) + " " + num(on));
        set_status_bit(o, kBits[bit], on != 0);
    } else if (r < 80) {
        if (!ai) return;
        const int attitude = chance(35) ? ATTITUDE_SLEEP : (chance(50) ? ATTITUDE_AGGRESSIVE : ATTITUDE_NORMAL);
        g_ops.push_back("att " + num(u) + " " + num(attitude));
        ai->setAttitude(static_cast<AttitudeType>(attitude));
    } else if (r < 86) {
        const int slot = static_cast<int>(pick(3));
        const int lock = static_cast<int>(pick(3));
        g_ops.push_back("lock " + num(u) + " " + num(slot) + " " + num(lock));
        if (lock == 0)
            o->releaseWeaponLock(LOCKED_PERMANENTLY);
        else
            o->setWeaponLock(static_cast<WeaponSlotType>(slot), static_cast<WeaponLockType>(lock));
    } else if (r < 90) {
        const int on = chance(60) ? 1 : 0;
        g_ops.push_back("held " + num(u) + " " + num(on));
        if (on)
            o->setDisabled(DISABLED_HELD);
        else
            o->clearDisabled(DISABLED_HELD);
    } else if (r < 94) {
        g_ops.push_back("kill " + num(u));
        o->kill();
    } else {
        if (ai) op_order(u, "idle", AICMD_IDLE, chance(50), -1, nullptr, 0);
    }
}

// --- scenes -----------------------------------------------------------------

struct Spawn {
    int kind;
    int team;
    float x, y, angle;
};

static void blob(std::string& table, int x, int y, int w, int h) {
    for (int j = y; j < y + h && j < kCells; ++j)
        for (int i = x; i < x + w && i < kCells; ++i)
            if (i >= 0 && j >= 0) table[j * kCells + i] = '1';
}

static int attacker_kind() {
    const uint32_t r = pick(20);
    if (r < 4) return K_RIFLE;
    if (r < 7) return K_TANK;
    if (r < 8) return K_KNIFE;
    if (r < 10) return K_SPLASH;
    if (r < 12) return K_LONG;
    if (r < 13) return K_LEECH;
    if (r < 14) return K_SPENT;
    if (r < 16) return K_CREEP;
    if (r < 17) return K_FLAK;
    if (r < 18) return K_HOVER;
    return K_DUMMY;
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_s14_attack_states <out.json>\n");
        return 2;
    }
    const int dump_scene = std::getenv("S14_SCENE") ? std::atoi(std::getenv("S14_SCENE")) : -1;
    const int dump_frame = std::getenv("S14_FRAME") ? std::atoi(std::getenv("S14_FRAME")) : -1;
    oracle_wp51::World w;
    TheFramePacer = new FramePacer;
    TheCommandList = NEW CommandList;
    TheCommandList->init();
    TheRecorder = new RecorderClass;
    TheRecorder->init();
    g_terrain = new S14Terrain;
    TheTerrainLogic = g_terrain;
    TheTerrainLogic->init();
    TheWritableGlobalData->m_gravity = -1.0f;
    TheWritableGlobalData->m_partitionCellSize = 20.0f;
    TheWritableGlobalData->m_unitDamagedThresh = 0.7f;
    TheWritableGlobalData->m_unitReallyDamagedThresh = 0.35f;
    ThePartitionManager = NEW PartitionManager;
    ThePartitionManager->init();
    const_cast<TAiData*>(TheAI->getAiData())->m_forceIdleFramesCount = 45;
    const_cast<TAiData*>(TheAI->getAiData())->m_repulsedDistance = 50.0f;
    w.load_ini(kArmorIni);
    w.load_ini(locomotors_ini());
    w.init_with_neutral_player();
    w.load_ini(weapons_ini());
    w.load_ini(objects_ini());
    TheWeaponStore->postProcessLoad();
    ThePlayerList->getNthPlayer(1)->setPlayerType(PLAYER_HUMAN, FALSE);
    ThePlayerList->getNthPlayer(2)->setPlayerType(PLAYER_COMPUTER, FALSE);

    const std::string tmp = std::string(argv[1]) + ".tmp";
    FILE* out = std::fopen(tmp.c_str(), "w");
    if (!out) return 1;
    auto escape = [](const std::string& text) {
        std::string escaped;
        for (char c : text) escaped += c == '\n' ? std::string("\\n") : std::string(1, c);
        return escaped;
    };
    std::string names;
    for (int i = 0; i < kKindCount; ++i) names += (i ? "," : "") + std::string(kKinds[i].name);

    const int kScenarios = 80;
    const int kFrames = 150;
    std::fprintf(out,
                 "{\"vector_count\":%d,\"frames\":%d,\"cells\":%d,\"names\":\"%s\",\"locomotors\":\"%s\","
                 "\"weapons\":\"%s\",\"armor\":\"%s\",\"objects\":\"%s\",\"vectors\":[\n",
                 kScenarios, kFrames, kCells, names.c_str(), escape(locomotors_ini()).c_str(),
                 escape(weapons_ini()).c_str(), escape(kArmorIni).c_str(), escape(objects_ini()).c_str());

    bool first_row = true;
    for (int scenario = 0; scenario < kScenarios; ++scenario) {
        const int layout = scenario % 8;
        const UnsignedInt frame = 100 + pick(3000);
        const UnsignedInt seed = 1 + pick(1000000000);
        std::string& cliff = g_terrain->cliff;
        cliff.assign(kCells * kCells, '0');
        std::vector<Spawn> spawns;
        if (layout == 0) {
            // One duel in the open, each pair of sides in turn.
            spawns.push_back({(scenario / 8) % 10, 0, between(40.0f, 90.0f), between(60.0f, 260.0f),
                              between(-3.14f, 3.14f)});
            spawns.push_back({chance(60) ? K_DUMMY : attacker_kind(), 1, between(180.0f, 280.0f),
                              between(60.0f, 260.0f), between(-3.14f, 3.14f)});
        } else if (layout == 1) {
            // Squads facing each other.
            for (int i = 0; i < 6; ++i)
                spawns.push_back({attacker_kind(), i % 2, (i % 2 ? 210.0f : 90.0f) + between(-20.0f, 20.0f),
                                  80.0f + i * 30.0f, i % 2 ? 3.14f : 0.0f});
        } else if (layout == 2) {
            // Buildings in the way of line of sight.
            for (int i = 0; i < 2; ++i)
                spawns.push_back({chance(70) ? (chance(50) ? K_TANK : K_CREEP) : attacker_kind(), 0,
                                  between(30.0f, 90.0f), 90.0f + i * 100.0f + between(-30.0f, 30.0f), 0.0f});
            for (int i = 0; i < 2; ++i)
                spawns.push_back({chance(60) ? K_DUMMY : attacker_kind(), 1, between(230.0f, 290.0f),
                                  90.0f + i * 100.0f + between(-30.0f, 30.0f), 3.14f});
            for (int n = 0; n < 3; ++n)
                spawns.push_back({K_BUNKER, 2, between(130.0f, 190.0f), 60.0f + n * 95.0f, 0.0f});
        } else if (layout == 3) {
            // Immobile guns and portable ones against movers.
            spawns.push_back({K_TURRETLESS, 0, between(90.0f, 130.0f), between(100.0f, 200.0f), 0.0f});
            spawns.push_back({K_PORTABLE, 0, between(90.0f, 130.0f), between(200.0f, 280.0f), 0.0f});
            for (int i = 0; i < 3; ++i)
                spawns.push_back({attacker_kind(), 1, between(150.0f, 280.0f), between(40.0f, 280.0f),
                                  between(-3.14f, 3.14f)});
        } else if (layout == 4) {
            // A crowd, for splash damage and the next victim.
            spawns.push_back({K_SPLASH, 0, 60.0f, 160.0f, 0.0f});
            spawns.push_back({chance(50) ? K_SPLASH : attacker_kind(), 0, 60.0f, 200.0f, 0.0f});
            for (int i = 0; i < 4; ++i)
                spawns.push_back({chance(70) ? K_DUMMY : attacker_kind(), 1, 190.0f + between(-25.0f, 25.0f),
                                  160.0f + between(-35.0f, 35.0f), 3.14f});
        } else if (layout == 5) {
            // Walls of cliff between the sides.
            const int gap = 8 + static_cast<int>(pick(12));
            blob(cliff, 15, 0, 2, gap);
            blob(cliff, 15, gap + 4, 2, kCells);
            for (int i = 0; i < 4; ++i)
                spawns.push_back({attacker_kind(), i % 2, i % 2 ? between(210.0f, 280.0f) : between(30.0f, 100.0f),
                                  between(60.0f, 260.0f), between(-3.14f, 3.14f)});
        } else if (layout == 6) {
            // Aircraft and what shoots them.
            for (int i = 0; i < 2; ++i)
                spawns.push_back({K_HOVER, i, between(60.0f, 260.0f), between(60.0f, 260.0f), 0.0f});
            spawns.push_back({K_FLAK, 0, between(40.0f, 120.0f), between(150.0f, 280.0f), 0.7f});
            spawns.push_back({K_FLAK, 1, between(180.0f, 280.0f), between(40.0f, 200.0f), -0.7f});
            spawns.push_back({attacker_kind(), 1, between(150.0f, 280.0f), between(40.0f, 280.0f), -0.7f});
        } else {
            // Close quarters.
            for (int i = 0; i < 4; ++i)
                spawns.push_back({i == 0 ? K_KNIFE : (i == 1 ? K_LONG : attacker_kind()), i % 2,
                                  150.0f + between(-35.0f, 35.0f), 150.0f + between(-35.0f, 35.0f),
                                  between(-3.14f, 3.14f)});
        }

        TheGameLogic->*access_member(Wp51GLFrameTag()) = frame;
        InitRandom(seed);
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1000));
        Pathfinder* pf = TheAI->pathfinder();
        pf->reset();
        // The records outlive the map; start each scene from the same free list.
        PathfindCellInfo::allocateCellInfos();
        pf->newMap();

        g_ids.clear();
        g_teams.clear();
        g_ops.clear();
        g_tape.clear();
        std::ostringstream spawn_text;
        for (size_t i = 0; i < spawns.size(); ++i) {
            const Spawn& s = spawns[i];
            const ThingTemplate* tt = TheThingFactory->findTemplate(kKinds[s.kind].name);
            if (!tt) return 3;
            // Team 2 is the neutral side.
            Team* team = s.team == 2 ? ThePlayerList->getNthPlayer(0)->getDefaultTeam()
                                     : (s.team ? w.team_b : w.team_a);
            Object* o = TheThingFactory->newObject(tt, team);
            Coord3D p = at(s.x, s.y);
            if (s.kind == K_HOVER) p.z = 30.0f;
            o->setOrientation(s.angle);
            o->setPosition(&p);
            if (s.kind == K_BUNKER || s.kind == K_TURRETLESS) pf->addObjectToPathfindMap(o);
            if (s.kind == K_HOVER) set_status_bit(o, OBJECT_STATUS_AIRBORNE_TARGET, true);
            g_ids.push_back(o->getID());
            g_teams.push_back(s.team);
            spawn_text << (i ? ";" : "") << s.kind << "," << s.team << "," << c3(p) << "," << fx(s.angle);
        }
        const int units = static_cast<int>(g_ids.size());

        std::fprintf(out, "%s{\"s\":%d,\"frame\":%u,\"seed\":%u,\"spawns\":\"%s\",\"cliff\":\"%s\",\"tape\":\"%s\"}",
                     first_row ? "" : ",\n", scenario, static_cast<unsigned>(frame), static_cast<unsigned>(seed),
                     spawn_text.str().c_str(), cliff.c_str(), g_tape.c_str());
        first_row = false;

        std::vector<std::string> last(units);
        for (int f = 0; f < kFrames; ++f) {
            g_ops.clear();
            g_tape.clear();
            if (f == 1) {
                for (int u = 0; u < units; ++u) {
                    const int enemy = some_other(u, true);
                    if (layout == 0 && u == 0)
                        op_order(u, "attack", AICMD_ATTACK_OBJECT, true, 1, nullptr, NO_MAX_SHOTS_LIMIT);
                    else if (layout != 0 && chance(55) && enemy >= 0)
                        op_order(u, "attack", AICMD_ATTACK_OBJECT, chance(60), enemy, nullptr, NO_MAX_SHOTS_LIMIT);
                    else if (layout != 0)
                        give_order(u);
                }
            } else if (f > 1) {
                const int rate = layout == 0 ? 3 : 14;
                const int count = chance(rate) ? 1 + static_cast<int>(pick(2)) : 0;
                for (int n = 0; n < count; ++n) random_op(units);
            }
            g_dump_unit = -1;
            TheGameLogic->update();
            std::string ops_text;
            for (size_t k = 0; k < g_ops.size(); ++k) ops_text += (k ? ";" : "") + g_ops[k];
            std::ostringstream row;
            row << "{\"s\":" << scenario << ",\"f\":" << f << ",\"ops\":\"" << ops_text << "\",\"tape\":\"" << g_tape
                << "\",\"seed\":\"" << hex32(GetGameLogicRandomSeedCRC()) << "\",\"map\":\"" << cells_digest()
                << "\",\"u\":\"";
            for (int i = 0; i < units; ++i) {
                if (scenario == dump_scene && f == dump_frame) g_dump_unit = i;
                const std::string now = observe(i);
                if (g_dump_unit == i) std::fprintf(stderr, "S14 unit %d save %s\n", i, g_dump.c_str());
                row << (i ? "~" : "") << (now == last[i] ? std::string("=") : now);
                last[i] = now;
            }
            row << "\"}";
            std::fprintf(out, ",\n%s", row.str().c_str());
        }
        std::fflush(out);

        for (ObjectID id : g_ids)
            if (Object* o = TheGameLogic->findObjectByID(id)) TheGameLogic->destroyObject(o);
        w.flush_destroy_list();
        pf->processPathfindQueue();
        pf->processPathfindQueue();
        ThePartitionManager->update();
        TheWeaponStore->reset();
        if (TheGameLogic->getFirstObject()) return 4;
    }
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    return std::rename(tmp.c_str(), argv[1]) == 0 ? 0 : 1;
}
