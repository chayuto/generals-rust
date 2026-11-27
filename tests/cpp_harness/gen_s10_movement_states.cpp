// SPDX-License-Identifier: GPL-3.0-or-later
// Golden trace vectors for the movement states (AI plan slice S10): the unmodified
// reference AIStates.cpp move-to, attack-move, move-out-of-the-way, tighten and
// follow-path states, entered by real orders on the real pathfind map and stepped
// by the real GameLogic::update.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_s10_movement_states.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/S10_MovementStates.json
//
// States of other slices (attack, crate pickup, waypoints, repulsors and so on),
// including the attack and pickup states of the attack-move sub machine, are
// replaced by a state that records its calls; in the sub machine its updates also
// answer success or failure at random. getNextMoodTarget outside idle answers a
// random enemy. The searches of slice S6 and getMoveAwayFromPath are answered
// with synthetic paths. Calls and answers are in the row's tape, which the Rust
// suite replays.
//
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIUpdate.cpp.o __ZN17AIUpdateInterface10doPathfindEP25PathfindServicesInterface
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIPathfind.cpp.o __ZN10Pathfinder19getMoveAwayFromPathEP6ObjectS1_P4PathS1_S3_
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a PartitionManager.cpp.o __ZN16PartitionManager26getGroundOrStructureHeightEff
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIUpdate.cpp.o __ZN17AIUpdateInterface17getNextMoodTargetEbb
//
// Not covered: crate pickup, the jet's out-of-ammo exit, the Chinook's refusal to
// adjust destinations, bridges and the wall layer, sounds, loading a save.
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

S10_TAG(SubMachineTag, StateMachine*, AIAttackMoveToState, m_attackMoveMachine)

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

void AIUpdateInterface::doPathfind(PathfindServicesInterface* pathfinder) {
    static Services services;
    services.real = pathfinder;
    real_do_pathfind(this, &services);
}

Real PartitionManager::getGroundOrStructureHeight(Real, Real) { return 0.0f; }

static uint32_t g_state_hits[64];
static std::vector<ObjectID> g_ids;

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

// --- the replaced states ----------------------------------------------------

class TapeState : public State {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(TapeState, "TapeState", 512, 512)
public:
    TapeState(StateMachine* machine, StateID id, bool sub) : State(machine, "TapeState"), m_tapeId(id), m_sub(sub) {}
    // As the attack states it stands for answer.
    Bool isAttack() const override {
        return m_tapeId == AI_ATTACK_POSITION || m_tapeId == AI_ATTACK_OBJECT || m_tapeId == AI_FORCE_ATTACK_OBJECT ||
               m_tapeId == AI_ATTACK_AND_FOLLOW_OBJECT;
    }
    StateReturnType onEnter() override {
        tape("E," + who(getMachineOwner()) + "," + num(m_tapeId));
        return STATE_CONTINUE;
    }
    StateReturnType update() override {
        StateReturnType result = STATE_CONTINUE;
        if (m_sub) {
            const uint32_t r = pick(100);
            if (r < 12) result = STATE_SUCCESS;
            else if (r < 20) result = STATE_FAILURE;
        }
        tape("U," + who(getMachineOwner()) + "," + num(m_tapeId) + "=" +
             num(result == STATE_SUCCESS ? 1 : result == STATE_FAILURE ? 2 : 0));
        return result;
    }
    void onExit(StateExitType status) override {
        tape("X," + who(getMachineOwner()) + "," + num(m_tapeId) + "," + num(static_cast<int>(status)));
    }

protected:
    void crc(Xfer*) override {}
    void xfer(Xfer* xfer) override {
        UnsignedByte mark = 0xa5;
        xfer->xferUnsignedByte(&mark);
        UnsignedInt id = m_tapeId;
        xfer->xferUnsignedInt(&id);
    }
    void loadPostProcess() override {}

private:
    StateID m_tapeId;
    bool m_sub;
};
EMPTY_DTOR(TapeState)

static void replace_state(StateMachine* machine, std::pair<const StateID, State*>& entry, bool sub) {
    const StateID id = entry.first;
    State* old = entry.second;
    State* next = newInstance(TapeState)(machine, id, sub);
    next->friend_setID(id);
    next->friend_onSuccess(AI_IDLE);
    next->friend_onFailure(AI_IDLE);
    entry.second = next;
    deleteInstance(old);
}

static void replace_states(AIUpdateInterface* ai) {
    AIStateMachine* machine = ai->*access_member(MachineTag());
    StateMap& states = machine->*access_member(StateMapTag());
    for (auto& entry : states) {
        const StateID id = entry.first;
        switch (id) {
        case AI_IDLE:
        case AI_DEAD:
        case AI_BUSY:
        case AI_MOVE_TO:
        case AI_MOVE_OUT_OF_THE_WAY:
        case AI_MOVE_AND_TIGHTEN:
        case AI_FOLLOW_PATH:
        case AI_FOLLOW_EXITPRODUCTION_PATH:
            break;
        case AI_ATTACK_MOVE_TO: {
            StateMachine* sub = static_cast<AIAttackMoveToState*>(entry.second)->*access_member(SubMachineTag());
            for (auto& inner : sub->*access_member(StateMapTag()))
                if (inner.first != AI_IDLE) replace_state(sub, inner, true);
            break;
        }
        default:
            replace_state(machine, entry, false);
        }
    }
}

// --- terrain ----------------------------------------------------------------

static const int kCells = 32;

class S10Terrain : public oracle_wp51::OracleTerrainLogic {
public:
    std::string cliff = std::string(kCells * kCells, '0');
    S10Terrain() {
        extent.lo = {0.0f, 0.0f, 0.0f};
        extent.hi = {kCells * 10.0f, kCells * 10.0f, 100.0f};
    }
    Bool isCliffCell(Real x, Real y) const override {
        int i = static_cast<int>(std::floor(x / 10.0f));
        int j = static_cast<int>(std::floor(y / 10.0f));
        i = i < 0 ? 0 : (i >= kCells ? kCells - 1 : i);
        j = j < 0 ? 0 : (j >= kCells ? kCells - 1 : j);
        return cliff[j * kCells + i] == '1';
    }
};

static S10Terrain* g_terrain = nullptr;

// --- fixture ----------------------------------------------------------------

static std::string locomotor(const char* name, const char* surfaces, const char* appearance, int speed,
                             const char* extra) {
    std::ostringstream s;
    s << "Locomotor " << name << "\n  Surfaces = " << surfaces << "\n  Appearance = " << appearance
      << "\n  Speed = " << speed << "\n  TurnRate = 240\n  Acceleration = 90\n  Braking = 90\n"
      << "  MinSpeed = 0\n" << extra << "End\n";
    return s.str();
}

static std::string locomotors_ini() {
    return locomotor("S9_Legs", "GROUND", "TWO_LEGS", 30, "") +
           locomotor("S9_Wheels", "GROUND", "FOUR_WHEELS", 60, "  CanMoveBackwards = Yes\n") +
           locomotor("S9_Treads", "GROUND", "TREADS", 45, "") +
           locomotor("S9_Climb", "GROUND", "CLIMBER", 40, "  CloseEnoughDist3D = Yes\n") +
           locomotor("S9_Hover", "AIR", "HOVER", 90,
                     "  PreferredHeight = 40\n  Lift = 120\n  AllowAirborneMotiveForce = Yes\n"
                     "  ZAxisBehavior = SURFACE_RELATIVE_HEIGHT\n") +
           locomotor("S9_Wings", "AIR", "WINGS", 90,
                     "  PreferredHeight = 40\n  Lift = 120\n  AllowAirborneMotiveForce = Yes\n"
                     "  ZAxisBehavior = SURFACE_RELATIVE_HEIGHT\n  CirclingRadius = 20\n") +
           locomotor("S9_Meander", "GROUND", "FOUR_WHEELS", 50, "  WanderWidthFactor = 0.5\n");
}

static std::string object_ini(const char* name, const char* kind_of, const char* locomotor, float radius,
                              bool mobile, const char* extra) {
    std::ostringstream s;
    s << "Object " << name << "\n  KindOf = " << kind_of << "\n  Geometry = CYLINDER\n"
      << "  GeometryMajorRadius = " << radius << "\n  GeometryHeight = 8\n  GeometryIsSmall = No\n"
      << "  VisionRange = 100\n"
      << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n";
    if (mobile) {
        s << "  Behavior = PhysicsBehavior ModuleTag_Phys\n    Mass = 5\n  End\n"
          << "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
          << "  Locomotor = SET_NORMAL " << locomotor << "\n";
    }
    s << extra << "End\n";
    return s.str();
}

static const char* const kTemplates[] = {"S9_Inf",     "S9_Veh",  "S9_Tank",  "S9_Dozer",   "S9_Big",
                                         "S9_Climber", "S9_Air",  "S9_Block", "S9_Jet",     "S9_Tower",
                                         "S9_Meander", "S9_Crusher", "S10_Missile", "S10_Digger"};
enum { K_INF, K_VEH, K_TANK, K_DOZER, K_BIG, K_CLIMBER, K_AIR, K_BLOCK, K_JET, K_TOWER, K_MEANDER, K_CRUSHER,
       K_MISSILE, K_DIGGER };

static std::string objects_ini() {
    return object_ini("S9_Inf", "PRELOAD SELECTABLE INFANTRY", "S9_Legs", 4, true, "  CrushableLevel = 0\n") +
           object_ini("S9_Veh", "PRELOAD SELECTABLE VEHICLE", "S9_Wheels", 5, true, "") +
           object_ini("S9_Tank", "PRELOAD SELECTABLE VEHICLE", "S9_Treads", 9, true, "") +
           object_ini("S9_Dozer", "PRELOAD SELECTABLE VEHICLE DOZER", "S9_Wheels", 8, true, "") +
           object_ini("S9_Big", "PRELOAD SELECTABLE VEHICLE", "S9_Treads", 16, true, "") +
           object_ini("S9_Climber", "PRELOAD SELECTABLE INFANTRY", "S9_Climb", 4, true, "") +
           object_ini("S9_Air", "PRELOAD SELECTABLE AIRCRAFT", "S9_Hover", 6, true, "") +
           object_ini("S9_Block", "PRELOAD STRUCTURE IMMOBILE", "", 12, false, "") +
           object_ini("S9_Jet", "PRELOAD SELECTABLE AIRCRAFT", "S9_Wings", 6, true, "") +
           object_ini("S9_Tower", "PRELOAD STRUCTURE IMMOBILE AIRCRAFT_PATH_AROUND", "", 18, false, "") +
           object_ini("S9_Meander", "PRELOAD SELECTABLE VEHICLE", "S9_Meander", 6, true, "") +
           object_ini("S9_Crusher", "PRELOAD SELECTABLE VEHICLE", "S9_Treads", 8, true, "  CrusherLevel = 2\n") +
           object_ini("S10_Missile", "PRELOAD PROJECTILE", "S9_Hover", 2, true, "") +
           object_ini("S10_Digger", "PRELOAD SELECTABLE VEHICLE DOZER HARVESTER", "S9_Wheels", 6, true, "");
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

// Fields are separated by '|' so a mismatch names the one that differs.
static std::string observe(int i) {
    Object* o = unit(i);
    if (!o) return "gone";
    AIUpdateInterface* ai = o->getAI();
    if (!ai) return c3(*o->getPosition());
    uint32_t unused = 0;
    std::string save = oracle_wp51::World::snapshot_save_hex(ai, &unused);
    // `Path::xfer` writes an uninitialized word before its last byte; zero it.
    if (Path* path = ai->getPath()) {
        const std::string bytes = oracle_wp51::World::snapshot_save_hex(path, &unused);
        const size_t at = save.find(bytes.substr(0, bytes.size() - 10));
        if (at == std::string::npos) std::abort();
        for (size_t k = at + bytes.size() - 10; k < at + bytes.size() - 2; ++k) save[k] = '0';
    }
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

static void command(Object* o, AICommandType type, CommandSourceType source, Object* target,
                    const std::vector<Coord3D>& points) {
    AICommandParms parms(type, source);
    parms.m_obj = target;
    if (!points.empty()) {
        parms.m_pos = points[0];
        parms.m_coords = points;
    }
    parms.m_intValue = NO_MAX_SHOTS_LIMIT;
    o->getAI()->aiDoCommand(&parms);
}

static std::vector<std::string> g_ops;

static std::string points_text(const std::vector<Coord3D>& points) {
    std::string s;
    for (size_t i = 0; i < points.size(); ++i) s += (i ? "/" : "") + c3(points[i]);
    return s.empty() ? std::string("-") : s;
}

struct Order {
    const char* name;
    AICommandType type;
};

// "<name> <unit> <player> <target or -1> <points or ->"
static void op_order(int u, const char* name, AICommandType type, bool player, int target,
                     const std::vector<Coord3D>& points) {
    Object* o = unit(u);
    if (!o || !o->getAI()) return;
    if (target >= 0 && !unit(target)) return;
    g_ops.push_back(std::string(name) + " " + num(u) + " " + num(player ? 1 : 0) + " " + num(target) + " " +
                    points_text(points));
    command(o, type, player ? CMD_FROM_PLAYER : CMD_FROM_AI, target >= 0 ? unit(target) : nullptr, points);
}

// A scene may fence in an area no ground path reaches.
static bool g_pen = false;

static void fence_pen(std::string& cliff);

// A point near the unit most of the time, so that moves arrive.
static Coord3D some_point(const Object* o) {
    if (g_pen && chance(35)) return at(between(215.0f, 235.0f), between(215.0f, 235.0f));
    if (chance(35)) return at(between(15.0f, 305.0f), between(15.0f, 305.0f));
    const float reach = chance(30) ? 25.0f : 90.0f;
    return at(inside(o->getPosition()->x + between(-reach, reach)),
              inside(o->getPosition()->y + between(-reach, reach)));
}

static std::vector<Coord3D> some_path(const Object* o) {
    std::vector<Coord3D> points;
    Coord3D p = *o->getPosition();
    for (int n = 1 + static_cast<int>(pick(4)); n > 0; --n) {
        if (chance(12)) {
            // Too close to the point before: skipped by the state.
            points.push_back(at(inside(p.x + between(-6.0f, 6.0f)), inside(p.y + between(-6.0f, 6.0f))));
            continue;
        }
        p = at(inside(p.x + between(-70.0f, 70.0f)), inside(p.y + between(-70.0f, 70.0f)));
        if (g_pen && chance(25)) p = at(between(215.0f, 235.0f), between(215.0f, 235.0f));
        points.push_back(p);
    }
    return points;
}

// The weights of the orders a scene gives.
enum Favour { F_ANY, F_ATTACK_MOVE, F_PATH, F_OBJECT };

static void give_order(int u, int units, Favour favour) {
    Object* o = unit(u);
    if (!o || !o->getAI()) return;
    const int v = static_cast<int>(pick(units));
    const bool player = chance(55);
    const Coord3D to = some_point(o);
    uint32_t r = pick(100);
    if (favour == F_ATTACK_MOVE && chance(60)) r = 30;
    if (favour == F_PATH && chance(55)) r = 50;
    if (favour == F_OBJECT && chance(60)) r = 22;
    if (r < 20) {
        op_order(u, "move", AICMD_MOVE_TO_POSITION, player, -1, {to});
    } else if (r < 28) {
        if (v != u && unit(v)) op_order(u, "moveobj", AICMD_MOVE_TO_OBJECT, player, v, {});
    } else if (r < 44) {
        op_order(u, "amove", AICMD_ATTACKMOVE_TO_POSITION, player, -1, {to});
    } else if (r < 58) {
        op_order(u, "follow", AICMD_FOLLOW_PATH, player, -1, some_path(o));
    } else if (r < 64) {
        op_order(u, "append", AICMD_FOLLOW_PATH_APPEND, player, -1, {to});
    } else if (r < 72) {
        const int ignore = chance(30) && v != u && unit(v) ? v : -1;
        op_order(u, "exit", AICMD_FOLLOW_EXITPRODUCTION_PATH, player, ignore, some_path(o));
    } else if (r < 82) {
        op_order(u, "tighten", AICMD_TIGHTEN_TO_POSITION, player, -1, {to});
    } else if (r < 90) {
        if (v != u && unit(v) && unit(v)->getAI()) op_order(u, "away", AICMD_MOVE_AWAY_FROM_UNIT, player, v, {});
    } else if (r < 93) {
        op_order(u, "idle", AICMD_IDLE, player, -1, {});
    } else {
        std::vector<Coord3D> path = some_path(o);
        g_ops.push_back("quickexit " + num(u) + " " + points_text(path));
        o->getAI()->doQuickExit(&path);
    }
}

static void set_status_bit(Object* o, ObjectStatusTypes bit, bool on) {
    o->setStatus(MAKE_OBJECT_STATUS_MASK(bit), on);
}

static void random_op(int units, Favour favour) {
    const int u = static_cast<int>(pick(units));
    Object* o = unit(u);
    if (!o || !o->getAI()) return;
    AIUpdateInterface* ai = o->getAI();
    const int v = static_cast<int>(pick(units));
    Object* other = unit(v);
    const uint32_t r = pick(100);
    if (r < 55) {
        give_order(u, units, favour);
    } else if (r < 62) {
        if (!other || !other->getAI() || v == u) return;
        Coord3D p = *other->getPosition();
        p.x += between(-6.0f, 6.0f);
        p.y += between(-6.0f, 6.0f);
        const float angle = between(-3.14f, 3.14f);
        g_ops.push_back("pos " + num(u) + " " + c3(p) + " " + fx(angle));
        o->setPosition(&p);
        o->setOrientation(angle);
    } else if (r < 70) {
        if (!other || !other->getAI() || v == u || !o->getPhysics()) return;
        const Bool result = ai->processCollision(o->getPhysics(), other);
        g_ops.push_back("coll " + num(u) + " " + num(v) + " " + num(result ? 1 : 0));
    } else if (r < 73) {
        if (!ai->getCurLocomotor()) return;
        g_ops.push_back("ultra " + num(u));
        ai->getCurLocomotor()->setUltraAccurate(TRUE);
    } else if (r < 77) {
        if (v == u || !other) return;
        g_ops.push_back("ign " + num(u) + " " + num(v));
        ai->ignoreObstacleID(g_ids[v]);
        // Order towards it, so that the state refuses to adjust.
        if (chance(70)) op_order(u, "moveobj", AICMD_MOVE_TO_OBJECT, chance(50), v, {});
    } else if (r < 83) {
        // The statuses the states look at, on for a while.
        static const ObjectStatusTypes kBits[] = {OBJECT_STATUS_IMMOBILE, OBJECT_STATUS_PARACHUTING,
                                                  OBJECT_STATUS_RIDER8};
        const int bit = static_cast<int>(pick(3));
        const int on = chance(60) ? 1 : 0;
        g_ops.push_back("status " + num(u) + " " + num(kBits[bit]) + " " + num(on));
        set_status_bit(o, kBits[bit], on != 0);
        if (on && chance(60)) give_order(u, units, favour);
    } else if (r < 88) {
        const int attitude = chance(60) ? ATTITUDE_AGGRESSIVE : (chance(50) ? ATTITUDE_ALERT : ATTITUDE_NORMAL);
        g_ops.push_back("att " + num(u) + " " + num(attitude));
        ai->setAttitude(static_cast<AttitudeType>(attitude));
    } else if (r < 92) {
        if (!other || !other->getAI() || v == u) return;
        g_ops.push_back("form " + num(u) + " " + num(v));
        AIGroupPtr group = TheAI->createGroup();
        group->add(o);
        group->add(other);
        o->setFormationID(static_cast<FormationID>(7));
        give_order(u, units, favour);
    } else if (r < 95) {
        const int frames = static_cast<int>(pick(40));
        g_ops.push_back("nocoll " + num(u) + " " + num(frames));
        ai->setIgnoreCollisionTime(frames);
    } else if (r < 97) {
        const float speed = chance(30) ? 999999.0f : between(5.0f, 60.0f);
        g_ops.push_back("speed " + num(u) + " " + fx(speed));
        ai->setDesiredSpeed(speed);
    } else {
        op_order(u, "idle", AICMD_IDLE, chance(50), -1, {});
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

static int mover_kind() {
    const uint32_t r = pick(20);
    if (r < 6) return K_INF;
    if (r < 11) return K_VEH;
    if (r < 14) return K_TANK;
    if (r < 16) return K_DOZER;
    if (r < 17) return K_BIG;
    if (r < 19) return K_CLIMBER;
    return K_AIR;
}

static void fence_pen(std::string& cliff) {
    blob(cliff, 18, 18, 9, 2);
    blob(cliff, 18, 25, 9, 2);
    blob(cliff, 18, 18, 2, 9);
    blob(cliff, 25, 18, 2, 9);
    g_pen = true;
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_s10_movement_states <out.json>\n");
        return 2;
    }
    oracle_wp51::World w;
    TheFramePacer = new FramePacer;
    TheCommandList = NEW CommandList;
    TheCommandList->init();
    TheRecorder = new RecorderClass;
    TheRecorder->init();
    g_terrain = new S10Terrain;
    TheTerrainLogic = g_terrain;
    TheTerrainLogic->init();
    TheWritableGlobalData->m_gravity = -1.0f;
    TheWritableGlobalData->m_partitionCellSize = 20.0f;
    ThePartitionManager = NEW PartitionManager;
    ThePartitionManager->init();
    const_cast<TAiData*>(TheAI->getAiData())->m_forceIdleFramesCount = 45;
    const_cast<TAiData*>(TheAI->getAiData())->m_repulsedDistance = 50.0f;
    w.load_ini(locomotors_ini());
    w.load_ini(objects_ini());
    w.init_with_neutral_player();
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

    const int kScenarios = 60;
    const int kFrames = 110;
    // The last scenes run long with few orders, for the retries seconds apart.
    const int kLongFrom = 56;
    const int kLongFrames = 330;
    std::fprintf(out, "{\"vector_count\":%d,\"frames\":%d,\"cells\":%d,\"locomotors\":\"%s\",\"objects\":\"%s\",\"vectors\":[\n",
                 kScenarios, kFrames, kCells, escape(locomotors_ini()).c_str(), escape(objects_ini()).c_str());

    bool first_row = true;
    for (int scenario = 0; scenario < kScenarios; ++scenario) {
        const bool quiet = scenario >= kLongFrom;
        const int layout = quiet ? 4 : scenario % 8;
        const UnsignedInt frame = scenario % 9 == 0 ? 1 : 100 + pick(3000);
        const UnsignedInt seed = 1 + pick(1000000000);
        std::string& cliff = g_terrain->cliff;
        cliff.assign(kCells * kCells, '0');
        std::vector<Spawn> spawns;
        Favour favour = F_ANY;
        g_pen = false;
        if (layout == 0) {
            // Few units in the open.
            for (int i = 0; i < 3; ++i)
                spawns.push_back({i == 2 ? K_DIGGER : mover_kind() % K_AIR, i % 2, between(60.0f, 260.0f),
                                  between(60.0f, 260.0f), between(-3.14f, 3.14f)});
        } else if (layout == 1) {
            // A wall with a gap.
            const int gap = 8 + static_cast<int>(pick(14));
            const int width = 2 + static_cast<int>(pick(3));
            blob(cliff, 15, 0, 2, gap);
            blob(cliff, 15, gap + width, 2, kCells);
            for (int i = 0; i < 4; ++i)
                spawns.push_back({mover_kind() % K_BIG, i == 3 ? 1 : 0, 60.0f + i * 18.0f,
                                  between(60.0f, 260.0f), 0.0f});
        } else if (layout == 2) {
            // Into and through a crowd.
            for (int i = 0; i < 4; ++i)
                spawns.push_back({chance(60) ? K_INF : mover_kind() % K_AIR, chance(80) ? 0 : 1,
                                  155.0f + between(-18.0f, 18.0f), 165.0f + between(-18.0f, 18.0f),
                                  between(-3.14f, 3.14f)});
            for (int i = 0; i < 2; ++i)
                spawns.push_back({chance(60) ? K_VEH : mover_kind() % K_AIR, 0, 55.0f, 165.0f + i * 15.0f, 0.0f});
        } else if (layout == 3) {
            // Paths around blocks and buildings.
            favour = F_PATH;
            if (scenario % 16 == 3) fence_pen(cliff);
            for (int n = 0; n < 4; ++n)
                blob(cliff, 4 + static_cast<int>(pick(24)), 4 + static_cast<int>(pick(24)), 1 + pick(4), 1 + pick(4));
            for (int i = 0; i < 4; ++i)
                spawns.push_back({mover_kind(), chance(75) ? 0 : 1, between(40.0f, 280.0f), between(40.0f, 280.0f),
                                  between(-3.14f, 3.14f)});
            for (int n = 0; n < 2; ++n)
                spawns.push_back({K_BLOCK, 0, between(60.0f, 260.0f), between(60.0f, 260.0f), 0.0f});
        } else if (layout == 4) {
            // Attack moves, with enemies about.
            favour = F_ATTACK_MOVE;
            if (scenario % 16 == 4 || quiet) fence_pen(cliff);
            for (int i = 0; i < 5; ++i)
                spawns.push_back({mover_kind() % K_AIR, i % 2, between(60.0f, 260.0f), between(60.0f, 260.0f),
                                  between(-3.14f, 3.14f)});
        } else if (layout == 5) {
            // A jam of vehicles from both ends of a corridor.
            blob(cliff, 8, 13, 16, 2);
            blob(cliff, 8, 18, 16, 2);
            for (int i = 0; i < 6; ++i) {
                const bool left = i % 2 == 0;
                const int kinds[] = {K_VEH, K_VEH, K_TANK, K_MEANDER, K_CRUSHER, K_INF, K_DOZER};
                const float x = (left ? 40.0f : 270.0f) + (left ? 1.0f : -1.0f) * (i / 2) * 20.0f;
                spawns.push_back({kinds[pick(7)], chance(75) ? 0 : 1, x, 165.0f + between(-4.0f, 4.0f),
                                  left ? 0.0f : 3.14159f});
            }
        } else if (layout == 6) {
            // Climbers among cliffs.
            for (int n = 0; n < 6; ++n)
                blob(cliff, 3 + static_cast<int>(pick(24)), 3 + static_cast<int>(pick(24)), 3 + pick(5), 3 + pick(5));
            for (int i = 0; i < 4; ++i)
                spawns.push_back({i < 3 ? K_CLIMBER : K_INF, 0, between(40.0f, 280.0f), between(40.0f, 280.0f),
                                  between(-3.14f, 3.14f)});
        } else {
            // Aircraft, missiles and what they chase.
            favour = F_OBJECT;
            for (int i = 0; i < 2; ++i)
                spawns.push_back({K_MISSILE, 0, 40.0f + i * 25.0f, 40.0f + between(0.0f, 60.0f), 0.7f});
            spawns.push_back({K_JET, 0, between(40.0f, 120.0f), between(150.0f, 280.0f), 0.7f});
            spawns.push_back({K_AIR, 1, between(150.0f, 280.0f), between(40.0f, 280.0f), -0.7f});
            for (int i = 0; i < 2; ++i)
                spawns.push_back({K_VEH, 1, between(120.0f, 280.0f), between(120.0f, 280.0f), 0.0f});
            for (int n = 1 + static_cast<int>(pick(3)); n > 0; --n)
                spawns.push_back({K_TOWER, 0, between(100.0f, 230.0f), between(80.0f, 220.0f), 0.0f});
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
        g_ops.clear();
        g_tape.clear();
        std::ostringstream spawn_text;
        for (size_t i = 0; i < spawns.size(); ++i) {
            const Spawn& s = spawns[i];
            const ThingTemplate* tt = TheThingFactory->findTemplate(kTemplates[s.kind]);
            if (!tt) return 3;
            Object* o = TheThingFactory->newObject(tt, s.team ? w.team_b : w.team_a);
            if (o->getAI()) replace_states(o->getAI());
            Coord3D p = at(s.x, s.y);
            if ((s.kind == K_AIR || s.kind == K_JET || s.kind == K_MISSILE) && s.angle > 0.0f) p.z = 40.0f;
            o->setOrientation(s.angle);
            o->setPosition(&p);
            if (s.kind == K_BLOCK || s.kind == K_TOWER) pf->addObjectToPathfindMap(o);
            g_ids.push_back(o->getID());
            spawn_text << (i ? ";" : "") << s.kind << "," << s.team << "," << c3(p) << "," << fx(s.angle);
        }
        const int units = static_cast<int>(g_ids.size());

        std::fprintf(out, "%s{\"s\":%d,\"frame\":%u,\"seed\":%u,\"spawns\":\"%s\",\"cliff\":\"%s\",\"tape\":\"%s\"}",
                     first_row ? "" : ",\n", scenario, static_cast<unsigned>(frame), static_cast<unsigned>(seed),
                     spawn_text.str().c_str(), cliff.c_str(), g_tape.c_str());
        first_row = false;

        std::vector<std::string> last(units);
        const int frames = quiet ? kLongFrames : kFrames;
        for (int f = 0; f < frames; ++f) {
            g_ops.clear();
            g_tape.clear();
            if (f == 1 && quiet) {
                // Into the fenced area, which no path reaches.
                for (int u = 0; u < units; ++u) {
                    const bool follow = u == 4;
                    op_order(u, follow ? "follow" : "amove",
                             follow ? AICMD_FOLLOW_PATH : AICMD_ATTACKMOVE_TO_POSITION, true, -1,
                             {at(between(215.0f, 235.0f), between(215.0f, 235.0f))});
                }
            } else if (f == 1) {
                for (int u = 0; u < units; ++u)
                    if (chance(85)) give_order(u, units, favour);
            } else if (f > 1 && !quiet) {
                const int rate = layout == 4 || layout == 5 || layout == 2 ? 20 : 12;
                const int count = chance(rate) ? 1 + static_cast<int>(pick(2)) : 0;
                for (int n = 0; n < count; ++n) random_op(units, favour);
            }
            TheGameLogic->update();
            std::string ops_text;
            for (size_t k = 0; k < g_ops.size(); ++k) ops_text += (k ? ";" : "") + g_ops[k];
            std::ostringstream row;
            row << "{\"s\":" << scenario << ",\"f\":" << f << ",\"ops\":\"" << ops_text << "\",\"tape\":\"" << g_tape
                << "\",\"seed\":\"" << hex32(GetGameLogicRandomSeedCRC()) << "\",\"map\":\"" << cells_digest()
                << "\",\"u\":\"";
            for (int i = 0; i < units; ++i) {
                const std::string now = observe(i);
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
        if (TheGameLogic->getFirstObject()) return 4;
    }
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    return std::rename(tmp.c_str(), argv[1]) == 0 ? 0 : 1;
}
