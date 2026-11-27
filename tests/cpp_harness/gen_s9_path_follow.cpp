// SPDX-License-Identifier: GPL-3.0-or-later
// Golden trace vectors for path following inside the AI update (AI plan slice S9):
// the unmodified reference AIUpdateInterface requesting, computing and driving paths
// on the real pathfind map, with units placed in each other's way, stepped by the
// real GameLogic::update.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_s9_path_follow.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/S9_PathFollow.json
//
// The movement states are not part of the slice: every state other than idle, dead
// and busy is replaced by a state that only records its calls, and the harness makes
// the calls a movement state would make (request, start and end of a move). The
// searches of slice S6 (findClosestPath, patchPath, findSafePath, asked through the
// services handed to doPathfind, and getMoveAwayFromPath) are answered here with
// synthetic paths; calls and answers are recorded in the row's tape, which the Rust
// suite replays. findPath goes to the real pathfinder.
//
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIUpdate.cpp.o __ZN17AIUpdateInterface10doPathfindEP25PathfindServicesInterface
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIPathfind.cpp.o __ZN10Pathfinder19getMoveAwayFromPathEP6ObjectS1_P4PathS1_S3_
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a PartitionManager.cpp.o __ZN16PartitionManager26getGroundOrStructureHeightEff
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIUpdate.cpp.o __ZN17AIUpdateInterface17getNextMoodTargetEbb
//
// Not covered: computeAttackPath beyond its answer for a unit without a weapon,
// bridges and the wall layer, formations, squishing, setPathFromWaypoint.

#include "PreRTS.h"

#include "oracle_wp51_world.h"

#include "Common/FramePacer.h"
#include "Common/Player.h"
#include "Common/RandomValue.h"
#include "Common/Recorder.h"
#include "Common/StateMachine.h"
#include "Common/MessageStream.h"
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

#define S9_TAG(Tag, Type, Class, Member)                       \
    struct Tag { using type = Type Class::*; };                \
    template struct MemberAccess<Tag, &Class::Member>;         \
    Tag::type access_member(Tag);

using StateMap = std::map<StateID, State*>;
S9_TAG(StateMapTag, StateMap, StateMachine, m_stateMap)
S9_TAG(MachineTag, AIStateMachine*, AIUpdateInterface, m_stateMachine)
S9_TAG(GoalCellTag, ICoord2D, AIUpdateInterface, m_pathfindGoalCell)
S9_TAG(CurCellTag, ICoord2D, AIUpdateInterface, m_pathfindCurCell)
S9_TAG(IsBlockedTag, Bool, AIUpdateInterface, m_isBlocked)
S9_TAG(MaxBlockedTag, Real, AIUpdateInterface, m_curMaxBlockedSpeed)
S9_TAG(BumpTag, Real, AIUpdateInterface, m_bumpSpeedLimit)
S9_TAG(StampTag, UnsignedInt, AIUpdateInterface, m_pathTimestamp)
S9_TAG(RetryTag, Bool, AIUpdateInterface, m_retryPath)
S9_TAG(WaitingTag, Bool, AIUpdateInterface, m_waitingForPath)
S9_TAG(QueueFrameTag, UnsignedInt, AIUpdateInterface, m_queueForPathFrame)
S9_TAG(IgnoreTag, UnsignedInt, AIUpdateInterface, m_ignoreCollisionsUntil)
S9_TAG(VelTag, Coord3D, PhysicsBehavior, m_vel)

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

Object* AIUpdateInterface::getNextMoodTarget(Bool, Bool) { return nullptr; }

// --- the replaced states ----------------------------------------------------

class TapeState : public State {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(TapeState, "TapeState", 512, 512)
public:
    TapeState(StateMachine* machine, StateID id) : State(machine, "TapeState"), m_tapeId(id) {}
    StateReturnType onEnter() override {
        tape("E," + who(getMachineOwner()) + "," + num(m_tapeId));
        return STATE_CONTINUE;
    }
    StateReturnType update() override {
        tape("U," + who(getMachineOwner()) + "," + num(m_tapeId));
        return STATE_CONTINUE;
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
};
EMPTY_DTOR(TapeState)

static void replace_states(AIUpdateInterface* ai) {
    AIStateMachine* machine = ai->*access_member(MachineTag());
    StateMap& states = machine->*access_member(StateMapTag());
    for (auto& entry : states) {
        const StateID id = entry.first;
        if (id == AI_IDLE || id == AI_DEAD || id == AI_BUSY) continue;
        State* old = entry.second;
        State* next = newInstance(TapeState)(machine, id);
        next->friend_setID(id);
        next->friend_onSuccess(AI_IDLE);
        next->friend_onFailure(AI_IDLE);
        entry.second = next;
        deleteInstance(old);
    }
}

// --- terrain ----------------------------------------------------------------

static const int kCells = 32;

class S9Terrain : public oracle_wp51::OracleTerrainLogic {
public:
    std::string cliff = std::string(kCells * kCells, '0');
    S9Terrain() {
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

static S9Terrain* g_terrain = nullptr;

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
                                         "S9_Meander", "S9_Crusher"};
enum { K_INF, K_VEH, K_TANK, K_DOZER, K_BIG, K_CLIMBER, K_AIR, K_BLOCK, K_JET, K_TOWER, K_MEANDER, K_CRUSHER };

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
           object_ini("S9_Crusher", "PRELOAD SELECTABLE VEHICLE", "S9_Treads", 8, true, "  CrusherLevel = 2\n");
}

// --- observation ------------------------------------------------------------

static std::vector<ObjectID> g_ids;
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

static void command(Object* o, AICommandType type, Object* target, const Coord3D* pos) {
    AICommandParms parms(type, CMD_FROM_AI);
    parms.m_obj = target;
    if (pos) {
        parms.m_pos = *pos;
        parms.m_coords.push_back(*pos);
    }
    o->getAI()->aiDoCommand(&parms);
}

// What a movement state does when it is entered.
static void begin_move(AIUpdateInterface* ai) {
    ai->friend_startingMove();
    ai->setLocomotorGoalPositionOnPath();
}

static std::vector<std::string> g_ops;
static std::vector<bool> g_driving;
static std::vector<Coord3D> g_dest;

static void op_go(int u, const Coord3D& to, int final_goal) {
    Object* o = unit(u);
    if (!o || !o->getAI()) return;
    AIUpdateInterface* ai = o->getAI();
    command(o, AICMD_BUSY, nullptr, nullptr);
    Coord3D goal = to;
    (ai->*access_member(MachineTag()))->setGoalPosition(&goal);
    ai->requestPath(&goal, final_goal);
    begin_move(ai);
    g_driving[u] = true;
    g_dest[u] = to;
    g_ops.push_back("go " + num(u) + " " + c3(to) + " " + num(final_goal));
}

// What a movement state does for a unit that is stuck or lost its path.
static void op_repath(int u, int final_goal) {
    AIUpdateInterface* ai = unit(u)->getAI();
    g_ops.push_back("repath " + num(u) + " " + c3(g_dest[u]) + " " + num(final_goal));
    Coord3D goal = g_dest[u];
    ai->requestPath(&goal, final_goal);
}

static void op_away(int u, int v) {
    g_ops.push_back("away " + num(u) + " " + num(v));
    command(unit(u), AICMD_MOVE_AWAY_FROM_UNIT, unit(v), nullptr);
}

// A collision, then the same push aside twice.
static void op_push(int u, int v, int w) {
    Object* o = unit(u);
    if (!o->getPhysics() || v == u) return;
    const Bool result = o->getAI()->processCollision(o->getPhysics(), unit(v));
    g_ops.push_back("coll " + num(u) + " " + num(v) + " " + num(result ? 1 : 0));
    op_away(u, w);
    op_away(u, w);
}

// Two idle units on top of each other, and their collision.
static void op_stack(int u, int v) {
    Object* o = unit(u);
    Object* other = unit(v);
    if (u == v || !o->getPhysics()) return;
    g_ops.push_back("idle " + num(u));
    command(o, AICMD_IDLE, nullptr, nullptr);
    g_ops.push_back("idle " + num(v));
    command(other, AICMD_IDLE, nullptr, nullptr);
    g_driving[u] = g_driving[v] = false;
    Coord3D p = *other->getPosition();
    p.x += between(-3.0f, 3.0f);
    p.y += between(-3.0f, 3.0f);
    const float angle = between(-3.14f, 3.14f);
    g_ops.push_back("pos " + num(u) + " " + c3(p) + " " + fx(angle));
    o->setPosition(&p);
    o->setOrientation(angle);
    const Bool result = o->getAI()->processCollision(o->getPhysics(), other);
    g_ops.push_back("coll " + num(u) + " " + num(v) + " " + num(result ? 1 : 0));
}

static void random_op(int units) {
    const int u = static_cast<int>(pick(units));
    Object* o = unit(u);
    if (!o || !o->getAI()) return;
    AIUpdateInterface* ai = o->getAI();
    const int v = static_cast<int>(pick(units));
    Object* other = unit(v);
    const Coord3D to = at(between(15.0f, 305.0f), between(15.0f, 305.0f));
    const uint32_t r = pick(100);
    const uint32_t extra = pick(100);
    if (extra < 6) {
        if (other && other->getAI()) op_stack(u, v);
        return;
    }
    if (extra < 12) {
        const int w = static_cast<int>(pick(units));
        if (other && other->getAI() && unit(w) && unit(w)->getAI()) op_push(u, v, w);
        return;
    }
    if (extra < 18) {
        // An attack path without a weapon falls back to an ordinary one.
        if (!other || ai->isBlockedAndStuck()) return;
        g_ops.push_back("attack " + num(u) + " " + num(v));
        command(o, AICMD_BUSY, nullptr, nullptr);
        ai->requestAttackPath(g_ids[v], other->getPosition());
        begin_move(ai);
        g_driving[u] = true;
        return;
    }
    if (extra < 24 && o->isKindOf(KINDOF_AIRCRAFT)) {
        g_ops.push_back("approach " + num(u) + " " + c3(to));
        command(o, AICMD_BUSY, nullptr, nullptr);
        Coord3D goal = to;
        ai->requestApproachPath(&goal);
        begin_move(ai);
        g_driving[u] = true;
        return;
    }
    if (r < 30) {
        // Towards another unit, past it, or anywhere.
        Coord3D dest = to;
        if (other && chance(50)) {
            dest = *other->getPosition();
            if (chance(50)) {
                dest.x += (dest.x - o->getPosition()->x) * 0.5f;
                dest.y += (dest.y - o->getPosition()->y) * 0.5f;
                dest.x = dest.x < 15.0f ? 15.0f : (dest.x > 305.0f ? 305.0f : dest.x);
                dest.y = dest.y < 15.0f ? 15.0f : (dest.y > 305.0f ? 305.0f : dest.y);
            }
            dest.z = 0.0f;
        }
        if (chance(5)) dest.x = 400.0f;
        op_go(u, dest, chance(70) ? 1 : 0);
    } else if (r < 36) {
        if (!other || !other->getAI()) return;
        g_ops.push_back("away " + num(u) + " " + num(v));
        command(o, AICMD_MOVE_AWAY_FROM_UNIT, other, nullptr);
    } else if (r < 41) {
        g_ops.push_back("approach " + num(u) + " " + c3(to));
        command(o, AICMD_BUSY, nullptr, nullptr);
        Coord3D goal = to;
        ai->requestApproachPath(&goal);
        begin_move(ai);
        g_driving[u] = true;
    } else if (r < 45) {
        g_ops.push_back("safe " + num(u) + " " + num(v));
        command(o, AICMD_BUSY, nullptr, nullptr);
        ai->requestSafePath(g_ids[v]);
        begin_move(ai);
        g_driving[u] = true;
    } else if (r < 50) {
        g_ops.push_back("exit " + num(u) + " " + c3(to));
        Coord3D goal = to;
        command(o, AICMD_FOLLOW_EXITPRODUCTION_PATH, nullptr, &goal);
        ai->setCanPathThroughUnits(TRUE);
        ai->requestPath(&goal, TRUE);
        begin_move(ai);
        g_driving[u] = true;
    } else if (r < 56) {
        g_ops.push_back("ign " + num(u) + " " + num(v));
        ai->ignoreObstacleID(g_ids[v]);
    } else if (r < 64) {
        if (!other || !other->getAI() || v == u) return;
        // Next to the other unit, so that they touch.
        Coord3D p = *other->getPosition();
        p.x += between(-6.0f, 6.0f);
        p.y += between(-6.0f, 6.0f);
        const float angle = between(-3.14f, 3.14f);
        g_ops.push_back("pos " + num(u) + " " + c3(p) + " " + fx(angle));
        o->setPosition(&p);
        o->setOrientation(angle);
    } else if (r < 76) {
        if (!other || !other->getAI() || v == u || !o->getPhysics()) return;
        const Bool result = ai->processCollision(o->getPhysics(), other);
        g_ops.push_back("coll " + num(u) + " " + num(v) + " " + num(result ? 1 : 0));
    } else if (r < 78) {
        if (!ai->getCurLocomotor()) return;
        g_ops.push_back("ultra " + num(u));
        ai->getCurLocomotor()->setUltraAccurate(TRUE);
    } else if (r < 81) {
        g_ops.push_back("idle " + num(u));
        command(o, AICMD_IDLE, nullptr, nullptr);
        g_driving[u] = false;
    } else if (r < 82) {
        g_ops.push_back("quick " + num(u) + " " + c3(to));
        ai->computeQuickPath(&to);
    } else if (r < 83) {
        // Again, next to where the last quick path ends.
        if (!ai->getPath()) return;
        Coord3D near = *ai->getPath()->getLastNode()->getPosition();
        near.x += between(-0.9f, 0.9f);
        g_ops.push_back("quick " + num(u) + " " + c3(near));
        ai->computeQuickPath(&near);
    } else if (r < 84) {
        g_ops.push_back("dist " + num(u) + " " + fx(ai->getLocomotorDistanceToGoal()));
    } else if (r < 86) {
        const float d = chance(50) ? 0.0f : between(0.0f, 20.0f);
        g_ops.push_back("extra " + num(u) + " " + fx(d));
        ai->setPathExtraDistance(d);
    } else if (r < 88) {
        const int on = chance(30) ? 1 : 0;
        g_ops.push_back("cpt " + num(u) + " " + num(on));
        ai->setCanPathThroughUnits(on);
    } else if (r < 90) {
        if (g_driving[u]) op_repath(u, chance(80) ? 1 : 0);
    } else if (r < 91) {
        g_ops.push_back("panic " + num(u));
        (ai->*access_member(MachineTag()))->setState(AI_PANIC);
        begin_move(ai);
    } else if (r < 93) {
        const int frames = static_cast<int>(pick(40));
        g_ops.push_back("nocoll " + num(u) + " " + num(frames));
        ai->setIgnoreCollisionTime(frames);
    } else {
        const float speed = chance(30) ? 999999.0f : between(5.0f, 60.0f);
        g_ops.push_back("speed " + num(u) + " " + fx(speed));
        ai->setDesiredSpeed(speed);
    }
}

// A movement state ends the move once the unit is there.
static void end_arrived(int units) {
    for (int u = 0; u < units; ++u) {
        Object* o = unit(u);
        if (!o || !o->getAI() || !g_driving[u]) continue;
        AIUpdateInterface* ai = o->getAI();
        if (ai->isWaitingForPath()) continue;
        if ((ai->isBlockedAndStuck() && chance(60)) || (!ai->getPath() && chance(10))) {
            op_repath(u, chance(80) ? 1 : 0);
            continue;
        }
        if (!ai->getPath() || !ai->getCurLocomotor()) continue;
        if (!chance(40)) continue;
        // The goal type is `POSITION_ON_PATH` while the unit drives.
        const Real distance = ai->getLocomotorDistanceToGoal();
        g_ops.push_back("dist " + num(u) + " " + fx(distance));
        if (distance < 6.0f) {
            g_ops.push_back("end " + num(u));
            ai->friend_endingMove();
            g_driving[u] = false;
        }
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

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_s9_path_follow <out.json>\n");
        return 2;
    }
    oracle_wp51::World w;
    TheFramePacer = new FramePacer;
    TheCommandList = NEW CommandList;
    TheCommandList->init();
    TheRecorder = new RecorderClass;
    TheRecorder->init();
    g_terrain = new S9Terrain;
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

    const int kScenarios = 63;
    const int kFrames = 60;
    std::fprintf(out, "{\"vector_count\":%d,\"frames\":%d,\"cells\":%d,\"locomotors\":\"%s\",\"objects\":\"%s\",\"vectors\":[\n",
                 kScenarios, kFrames, kCells, escape(locomotors_ini()).c_str(), escape(objects_ini()).c_str());

    bool first_row = true;
    for (int scenario = 0; scenario < kScenarios; ++scenario) {
        const int layout = scenario % 7;
        const UnsignedInt frame = scenario % 9 == 0 ? 1 : 100 + pick(3000);
        const UnsignedInt seed = 1 + pick(1000000000);
        std::string& cliff = g_terrain->cliff;
        cliff.assign(kCells * kCells, '0');
        std::vector<Spawn> spawns;
        std::vector<std::pair<int, Coord3D>> orders;
        if (layout == 0) {
            // Two units, or two pairs, head on.
            const float y = between(80.0f, 240.0f);
            const int kind = mover_kind() % K_AIR;
            const int pairs = 1 + static_cast<int>(pick(2));
            for (int p = 0; p < pairs; ++p) {
                const float row = y + p * 14.0f;
                const int a = static_cast<int>(spawns.size());
                spawns.push_back({kind, 0, 65.0f, row, 0.0f});
                spawns.push_back({chance(60) ? kind : mover_kind() % K_AIR, chance(70) ? 0 : 1, 205.0f,
                                  row + between(-4.0f, 4.0f), 3.14159f});
                orders.push_back({a, at(225.0f, row)});
                orders.push_back({a + 1, at(45.0f, row)});
            }
        } else if (layout == 1) {
            // Through a gap in a wall of cliffs.
            const int gap = 8 + static_cast<int>(pick(14));
            const int width = 2 + static_cast<int>(pick(3));
            blob(cliff, 15, 0, 2, gap);
            blob(cliff, 15, gap + width, 2, kCells);
            const float gy = (gap + width * 0.5f) * 10.0f;
            for (int i = 0; i < 4; ++i) {
                const bool left = i < 3 || chance(50);
                const float x = left ? 60.0f + i * 18.0f : 240.0f;
                const float y = gy + between(-30.0f, 30.0f);
                spawns.push_back({mover_kind() % K_BIG, 0, x, y, left ? 0.0f : 3.14159f});
                orders.push_back({i, at(left ? 265.0f : 55.0f, gy + between(-20.0f, 20.0f))});
            }
        } else if (layout == 2) {
            // Into and through a crowd of idle units.
            for (int i = 0; i < 4; ++i)
                spawns.push_back({chance(60) ? K_INF : mover_kind() % K_AIR, chance(80) ? 0 : 1,
                                  155.0f + between(-18.0f, 18.0f), 165.0f + between(-18.0f, 18.0f),
                                  between(-3.14f, 3.14f)});
            const int movers = 1 + static_cast<int>(pick(2));
            for (int i = 0; i < movers; ++i) {
                spawns.push_back({chance(60) ? K_VEH : mover_kind() % K_AIR, 0, 55.0f, 165.0f + i * 15.0f, 0.0f});
                orders.push_back({4 + i, at(chance(50) ? 265.0f : 155.0f, 165.0f)});
            }
        } else if (layout == 3) {
            // Crossing columns around blocks.
            for (int n = 0; n < 4; ++n)
                blob(cliff, 4 + static_cast<int>(pick(24)), 4 + static_cast<int>(pick(24)), 1 + pick(4), 1 + pick(4));
            for (int i = 0; i < 6; ++i) {
                const bool across = i % 2 == 0;
                const float lane = 140.0f + (i / 2) * 20.0f;
                spawns.push_back({mover_kind(), 0, across ? 45.0f : lane, across ? lane : 45.0f,
                                  across ? 0.0f : 1.5708f});
                orders.push_back({i, at(across ? 275.0f : lane, across ? lane : 275.0f)});
            }
        } else if (layout == 5) {
            // A jam of vehicles from both ends of a corridor.
            blob(cliff, 8, 13, 16, 2);
            blob(cliff, 8, 18, 16, 2);
            for (int i = 0; i < 6; ++i) {
                const bool left = i % 2 == 0;
                const int kinds[] = {K_VEH, K_VEH, K_TANK, K_MEANDER, K_CRUSHER, K_INF, K_DOZER};
                const float x = (left ? 40.0f : 270.0f) + (left ? 1.0f : -1.0f) * (i / 2) * 20.0f;
                const float y = 165.0f + between(-4.0f, 4.0f);
                spawns.push_back({kinds[pick(7)], chance(75) ? 0 : 1, x, y, left ? 0.0f : 3.14159f});
                orders.push_back({i, at(left ? 280.0f : 40.0f, 165.0f + between(-8.0f, 8.0f))});
            }
        } else if (layout == 6) {
            // Aircraft among towers.
            for (int i = 0; i < 4; ++i)
                spawns.push_back({i < 3 ? K_JET : K_AIR, 0, 40.0f + i * 25.0f, 40.0f + between(0.0f, 60.0f), 0.7f});
            for (int i = 0; i < 4; ++i) orders.push_back({i, at(between(200.0f, 290.0f), between(150.0f, 290.0f))});
            for (int n = 2 + static_cast<int>(pick(3)); n > 0; --n)
                spawns.push_back({K_TOWER, 0, between(100.0f, 230.0f), between(80.0f, 220.0f), 0.0f});
        } else {
            // Anything, with buildings in the way.
            for (int n = 0; n < 3; ++n)
                blob(cliff, static_cast<int>(pick(28)), static_cast<int>(pick(28)), 1 + pick(5), 1 + pick(3));
            const int count = 3 + static_cast<int>(pick(4));
            for (int i = 0; i < count; ++i)
                spawns.push_back({mover_kind(), chance(75) ? 0 : 1, between(40.0f, 280.0f), between(40.0f, 280.0f),
                                  between(-3.14f, 3.14f)});
            for (int i = 0; i < count; ++i)
                if (chance(70)) orders.push_back({i, at(between(20.0f, 300.0f), between(20.0f, 300.0f))});
            for (int n = static_cast<int>(pick(3)); n > 0; --n)
                spawns.push_back({K_BLOCK, 0, between(60.0f, 260.0f), between(60.0f, 260.0f), 0.0f});
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
            if ((s.kind == K_AIR || s.kind == K_JET) && s.angle > 0.0f) p.z = 40.0f;
            o->setOrientation(s.angle);
            o->setPosition(&p);
            if (s.kind == K_BLOCK || s.kind == K_TOWER) pf->addObjectToPathfindMap(o);
            g_ids.push_back(o->getID());
            spawn_text << (i ? ";" : "") << s.kind << "," << s.team << "," << c3(p) << "," << fx(s.angle);
        }
        const int units = static_cast<int>(g_ids.size());
        g_driving.assign(units, false);
        g_dest.assign(units, at(0.0f, 0.0f));

        std::fprintf(out, "%s{\"s\":%d,\"frame\":%u,\"seed\":%u,\"spawns\":\"%s\",\"cliff\":\"%s\",\"tape\":\"%s\"}",
                     first_row ? "" : ",\n", scenario, static_cast<unsigned>(frame), static_cast<unsigned>(seed),
                     spawn_text.str().c_str(), cliff.c_str(), g_tape.c_str());
        first_row = false;

        std::vector<std::string> last(units);
        for (int f = 0; f < kFrames; ++f) {
            g_ops.clear();
            g_tape.clear();
            if (f == 1) {
                for (const auto& order : orders) op_go(order.first, order.second, chance(85) ? 1 : 0);
            } else if (f > 1) {
                end_arrived(units);
                const int count = chance(layout == 4 || layout == 5 ? 45 : 22) ? 1 + static_cast<int>(pick(2)) : 0;
                for (int n = 0; n < count; ++n) random_op(units);
            }
            TheGameLogic->update();
            std::string ops;
            for (size_t k = 0; k < g_ops.size(); ++k) ops += (k ? ";" : "") + g_ops[k];
            std::ostringstream row;
            row << "{\"s\":" << scenario << ",\"f\":" << f << ",\"ops\":\"" << ops << "\",\"tape\":\"" << g_tape
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
