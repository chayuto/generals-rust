// SPDX-License-Identifier: GPL-3.0-or-later
// Golden trace vectors for the dozer and worker AI (AI plan slice S16): the
// unmodified reference DozerAIUpdate and WorkerAIUpdate frame updates, state
// machines (build, repair, mine clearing, bored behaviour) and orders, stepped by
// the real GameLogic::update on the real pathfind map.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_s16_dozer_worker_frames.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/S16_DozerWorkerFrames.json
//
// As in gen_s10_movement_states.cpp: states of other slices (attack and so on) are
// replaced by a state that records its calls, the searches of slice S6 and
// getMoveAwayFromPath are answered with synthetic paths, and getNextMoodTarget
// outside idle answers a random enemy. Calls and answers are in the row's tape.
// Both playing sides see the whole map.
//
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIUpdate.cpp.o __ZN17AIUpdateInterface10doPathfindEP25PathfindServicesInterface
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIPathfind.cpp.o __ZN10Pathfinder19getMoveAwayFromPathEP6ObjectS1_P4PathS1_S3_
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a PartitionManager.cpp.o __ZN16PartitionManager26getGroundOrStructureHeightEff
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIUpdate.cpp.o __ZN17AIUpdateInterface17getNextMoodTargetEbb
//
// Not covered: supply gathering (a worker is never sent to a dock), bridges,
// aircraft dozers, the construct order itself, loading a save.

#include "PreRTS.h"

#include "oracle_wp51_world.h"

#include "Common/ActionManager.h"
#include "Common/Energy.h"
#include "Common/FramePacer.h"
#include "Common/ScoreKeeper.h"
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
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/DozerAIUpdate.h"
#include "GameLogic/Module/WorkerAIUpdate.h"
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

#define S16_TAG(Tag, Type, Class, Member)                       \
    struct Tag { using type = Type Class::*; };                \
    template struct MemberAccess<Tag, &Class::Member>;         \
    Tag::type access_member(Tag);

using StateMap = std::map<StateID, State*>;
S16_TAG(StateMapTag, StateMap, StateMachine, m_stateMap)
S16_TAG(MachineTag, AIStateMachine*, AIUpdateInterface, m_stateMachine)
S16_TAG(GoalCellTag, ICoord2D, AIUpdateInterface, m_pathfindGoalCell)
S16_TAG(CurCellTag, ICoord2D, AIUpdateInterface, m_pathfindCurCell)
S16_TAG(IsBlockedTag, Bool, AIUpdateInterface, m_isBlocked)
S16_TAG(MaxBlockedTag, Real, AIUpdateInterface, m_curMaxBlockedSpeed)
S16_TAG(BumpTag, Real, AIUpdateInterface, m_bumpSpeedLimit)
S16_TAG(StampTag, UnsignedInt, AIUpdateInterface, m_pathTimestamp)
S16_TAG(RetryTag, Bool, AIUpdateInterface, m_retryPath)
S16_TAG(WaitingTag, Bool, AIUpdateInterface, m_waitingForPath)
S16_TAG(QueueFrameTag, UnsignedInt, AIUpdateInterface, m_queueForPathFrame)
S16_TAG(IgnoreTag, UnsignedInt, AIUpdateInterface, m_ignoreCollisionsUntil)
S16_TAG(VelTag, Coord3D, PhysicsBehavior, m_vel)

S16_TAG(SubMachineTag, StateMachine*, AIAttackMoveToState, m_attackMoveMachine)

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

static uint32_t g_state = 0x53313646u;
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

// --- the builders' private machines ------------------------------------------

S16_TAG(DozerMachineTag, DozerPrimaryStateMachine*, DozerAIUpdate, m_dozerMachine)
S16_TAG(DozerPreviousTag, DozerTask, DozerAIUpdate, m_previousTask)
S16_TAG(WorkerDozerMachineTag, DozerPrimaryStateMachine*, WorkerAIUpdate, m_dozerMachine)
S16_TAG(WorkerPreviousTag, DozerTask, WorkerAIUpdate, m_previousTask)
S16_TAG(WorkerMachineTag, WorkerStateMachine*, WorkerAIUpdate, m_workerMachine)
S16_TAG(GoalIdTag, ObjectID, StateMachine, m_goalObjectID)

// The layout of `DozerActionState`, which is local to DozerAIUpdate.cpp.
class MirrorActionState : public State {
public:
    DozerTask m_task;
    StateMachine* m_actionMachine;
};

// --- terrain ----------------------------------------------------------------

static const int kCells = 32;

class S16Terrain : public oracle_wp51::OracleTerrainLogic {
public:
    S16Terrain() {
        extent.lo = {0.0f, 0.0f, 0.0f};
        extent.hi = {kCells * 10.0f, kCells * 10.0f, 100.0f};
    }
};

// --- fixture ----------------------------------------------------------------

static std::string locomotor(const char* name, const char* appearance, int speed) {
    std::ostringstream s;
    s << "Locomotor " << name << "\n  Surfaces = GROUND\n  Appearance = " << appearance << "\n  Speed = " << speed
      << "\n  TurnRate = 360\n  Acceleration = 120\n  Braking = 120\n  MinSpeed = 0\nEnd\n";
    return s.str();
}
static std::string locomotors_ini() {
    return locomotor("S16_Wheels", "FOUR_WHEELS", 90) + locomotor("S16_Legs", "TWO_LEGS", 60);
}

static std::string weapons_ini() {
    return "Weapon S16_Sweeper\n  PrimaryDamage = 5\n  AttackRange = 25\n  AntiMine = Yes\n  AntiGround = No\n"
           "  DelayBetweenShots = 500\nEnd\n";
}

static std::string unit_ini(const char* name, const char* kind_of, const char* module, const char* fields,
                            const char* loco, float radius) {
    std::ostringstream s;
    s << "Object " << name << "\n  KindOf = " << kind_of << "\n  Geometry = CYLINDER\n"
      << "  GeometryMajorRadius = " << radius << "\n  GeometryHeight = 8\n  GeometryIsSmall = No\n"
      << "  VisionRange = 100\n"
      << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
      << "  Behavior = PhysicsBehavior ModuleTag_Phys\n    Mass = 5\n  End\n"
      << "  Behavior = " << module << " ModuleTag_AI\n" << fields << "  End\n"
      << "  WeaponSet\n    Conditions = MINE_CLEARING_DETAIL\n    Weapon = PRIMARY S16_Sweeper\n  End\n"
      << "  Locomotor = SET_NORMAL " << loco << "\nEnd\n";
    return s.str();
}

static std::string building_ini(const char* name, const char* geometry, int major, int minor, const char* seconds,
                                int health, int energy) {
    std::ostringstream s;
    s << "Object " << name << "\n  KindOf = PRELOAD STRUCTURE IMMOBILE SELECTABLE\n  Geometry = " << geometry
      << "\n  GeometryMajorRadius = " << major << "\n  GeometryMinorRadius = " << minor
      << "\n  GeometryHeight = 10\n  GeometryIsSmall = No\n  VisionRange = 60\n"
      << "  BuildTime = " << seconds << "\n  BuildCost = 100\n";
    if (energy) s << "  EnergyProduction = " << energy << "\n";
    s << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = " << health << "\n    InitialHealth = " << health
      << "\n  End\nEnd\n";
    return s.str();
}

static const char* const kTemplates[] = {"S16_Dozer", "S16_Worker",  "S16_Unit",  "S16_House",
                                         "S16_Quick", "S16_Reactor", "S16_Mine"};
enum { K_DOZER, K_WORKER, K_UNIT, K_HOUSE, K_QUICK, K_REACTOR, K_MINE };
static bool is_builder(int kind) { return kind == K_DOZER || kind == K_WORKER; }
static bool is_building(int kind) { return kind >= K_HOUSE && kind <= K_REACTOR; }

static std::string objects_ini() {
    return unit_ini("S16_Dozer", "PRELOAD SELECTABLE VEHICLE DOZER", "DozerAIUpdate",
                    "    RepairHealthPercentPerSecond = 10%\n    BoredTime = 600\n    BoredRange = 60\n",
                    "S16_Wheels", 7) +
           unit_ini("S16_Worker", "PRELOAD SELECTABLE INFANTRY DOZER HARVESTER", "WorkerAIUpdate",
                    "    MaxBoxes = 4\n    RepairHealthPercentPerSecond = 7%\n    BoredTime = 900\n"
                    "    BoredRange = 90\n    SupplyCenterActionDelay = 300\n"
                    "    SupplyWarehouseActionDelay = 400\n",
                    "S16_Legs", 4) +
           unit_ini("S16_Unit", "PRELOAD SELECTABLE VEHICLE", "AIUpdateInterface", "", "S16_Wheels", 6) +
           building_ini("S16_House", "BOX", 22, 14, "2.0", 200, 0) +
           building_ini("S16_Quick", "CYLINDER", 12, 12, "0.4", 150, 0) +
           building_ini("S16_Reactor", "CYLINDER", 18, 18, "1.1", 300, 5) +
           "Object S16_Mine\n  KindOf = PRELOAD MINE IMMOBILE\n  Geometry = CYLINDER\n  GeometryMajorRadius = 3\n"
           "  GeometryHeight = 2\n  VisionRange = 0\n"
           "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 10\n    InitialHealth = 10\n  End\nEnd\n";
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

static std::string model_text(Object* o) {
    std::string flags;
    if (Drawable* d = o->getDrawable()) {
        const ModelConditionFlags& bits = d->getModelConditionFlags();
        for (int b = 0; b < MODELCONDITION_COUNT; ++b)
            if (bits.test(b)) flags += num(b) + ".";
    }
    return flags;
}

static std::string status_text(Object* o) {
    std::string s;
    const ObjectStatusMaskType status = o->getStatusBits();
    for (int b = 0; b < OBJECT_STATUS_COUNT; ++b)
        if (status.test(b)) s += num(b) + ".";
    return s;
}

static std::string dozer_text(AIUpdateInterface* ai) {
    DozerAIInterface* d = ai->getDozerAIInterface();
    if (!d) return "-";
    const bool worker = ai->getWorkerAIInterface() != nullptr;
    WorkerAIUpdate* w = worker ? static_cast<WorkerAIUpdate*>(ai) : nullptr;
    DozerAIUpdate* z = worker ? nullptr : static_cast<DozerAIUpdate*>(ai);
    StateMachine* machine = reinterpret_cast<StateMachine*>(worker ? w->*access_member(WorkerDozerMachineTag())
                                                                   : z->*access_member(DozerMachineTag()));
    const DozerTask previous = worker ? w->*access_member(WorkerPreviousTag()) : z->*access_member(DozerPreviousTag());
    std::string s = num(machine->getCurrentStateID()) + "," + num(d->getCurrentTask()) + "," + num(previous) + "," +
                    num(d->getBuildSubTask());
    StateMap& states = machine->*access_member(StateMapTag());
    for (int t = 0; t < DOZER_NUM_TASKS; ++t) {
        const DozerTask task = static_cast<DozerTask>(t);
        s += "/" + num(static_cast<int>(d->getTaskTarget(task)));
        for (int p = 0; p < DOZER_NUM_DOCK_POINTS; ++p) {
            const Coord3D* point = d->getDockPoint(task, static_cast<DozerDockPoint>(p));
            s += ";" + (point ? c3(*point) : std::string("-"));
        }
        StateMachine* action = reinterpret_cast<MirrorActionState*>(states.find(t + 1)->second)->m_actionMachine;
        s += ";" + num(action->getCurrentStateID()) + ";" +
             num(static_cast<int>(action->*access_member(GoalIdTag()))) + ";" + c3(*action->getGoalPosition());
    }
    if (worker) s += "/" + num(reinterpret_cast<StateMachine*>(w->*access_member(WorkerMachineTag()))->getCurrentStateID());
    return s;
}

// Fields are separated by '|' so a mismatch names the one that differs.
static std::string observe(int i) {
    Object* o = unit(i);
    if (!o) return "gone";
    AIUpdateInterface* ai = o->getAI();
    BodyModuleInterface* body = o->getBodyModule();
    if (!ai) {
        return fx(body->getHealth()) + "|" + fx(o->getConstructionPercent()) + "|" + status_text(o) + "|" +
               num(static_cast<int>(o->getBuilderID())) + "," + num(static_cast<int>(o->getProducerID())) + "," +
               num(static_cast<int>(o->getSoleHealingBenefactor())) + "|" + model_text(o);
    }
    uint32_t unused = 0;
    std::string save = oracle_wp51::World::snapshot_save_hex(ai, &unused);
    // `Path::xfer` writes an uninitialized word before its last byte; zero it.
    if (Path* path = ai->getPath()) {
        const std::string bytes = oracle_wp51::World::snapshot_save_hex(path, &unused);
        const size_t found = save.find(bytes.substr(0, bytes.size() - 10));
        if (found == std::string::npos) std::abort();
        for (size_t k = found + bytes.size() - 10; k < found + bytes.size() - 2; ++k) save[k] = '0';
    }
    const uint32_t crc = oracle_wp51::World::snapshot_crc(ai);
    PhysicsBehavior* phys = o->getPhysics();
    const ICoord2D goal = ai->*access_member(GoalCellTag());
    const ICoord2D cur = ai->*access_member(CurCellTag());
    std::string s = c3(*o->getPosition());
    s += "|" + fx(o->getOrientation());
    s += "|" + (phys ? c3(phys->*access_member(VelTag())) : std::string("-"));
    s += "|" + num(goal.x) + "," + num(goal.y) + "," + num(cur.x) + "," + num(cur.y);
    s += "|" + path_text(ai->getPath());
    s += "|" + num(ai->getCurrentStateID()) + "," + model_text(o);
    s += "|" + num(o->testWeaponSetFlag(WEAPONSET_MINE_CLEARING_DETAIL) ? 1 : 0) + "," + fx(body->getHealth()) + "," +
         status_text(o);
    s += "|" + num(static_cast<long long>(ai->friend_getNextCallFrame()));
    s += "|" + dozer_text(ai);
    s += "|" + hex32(crc);
    s += "|" + hex64(fnv(save));
    return s;
}

// Which unit stands in and wants which ground cell, and the cell types.
static std::string cells_digest() {
    std::string s;
    for (int y = 0; y < kCells; ++y)
        for (int x = 0; x < kCells; ++x) {
            PathfindCell* cell = TheAI->pathfinder()->getCell(LAYER_GROUND, x, y);
            if (!cell) continue;
            const int goal = static_cast<int>(cell->getGoalUnit());
            const int present = static_cast<int>(cell->getPosUnit());
            const int type = static_cast<int>(cell->getType());
            if (goal || present || type)
                s += num(x) + "," + num(y) + "," + num(goal) + "," + num(present) + "," + num(type) + ";";
        }
    return hex64(fnv(s));
}

// Only the fields that changed since `last`; an unchanged one is "=".
static std::string changed(const std::string& now, std::vector<std::string>& last) {
    std::vector<std::string> fields;
    for (size_t from = 0;;) {
        const size_t bar = now.find('|', from);
        fields.push_back(now.substr(from, bar == std::string::npos ? bar : bar - from));
        if (bar == std::string::npos) break;
        from = bar + 1;
    }
    if (fields.size() != last.size()) {
        last = fields;
        return now;
    }
    std::string s;
    bool any = false;
    for (size_t k = 0; k < fields.size(); ++k) {
        const bool same = fields[k] == last[k];
        any = any || !same;
        s += (k ? "|" : "") + (same ? std::string("=") : fields[k]);
    }
    last = fields;
    return any ? s : std::string("=");
}

static std::string players_text() {
    std::string s;
    for (int p = 1; p <= 2; ++p) {
        Player* player = ThePlayerList->getNthPlayer(p);
        s += num(player->getScoreKeeper()->getTotalBuildingsBuilt()) + "," +
             num(player->getEnergy()->getProduction()) + "," + num(player->getEnergy()->getConsumption()) + ";";
    }
    return s;
}

// --- operations -------------------------------------------------------------

static Coord3D at(float x, float y) {
    Coord3D p;
    p.x = x;
    p.y = y;
    p.z = 0.0f;
    return p;
}

static std::vector<std::string> g_ops;
struct Spawn {
    int kind, team;
    float x, y, angle;
    int mode;  // 0 built, 1 damaged, 2 under construction
};
static std::vector<Spawn> g_spawns;

static void order(int u, const char* name, AICommandType type, bool player, int target, const Coord3D* to) {
    Object* o = unit(u);
    if (!o || !o->getAI()) return;
    Object* t = unit(target);
    if (target >= 0 && !t) return;
    AICommandParms parms(type, player ? CMD_FROM_PLAYER : CMD_FROM_AI);
    parms.m_obj = t;
    std::string points = "-";
    if (to) {
        parms.m_pos = *to;
        parms.m_coords = {*to};
        points = c3(*to);
    }
    parms.m_intValue = NO_MAX_SHOTS_LIMIT;
    o->getAI()->aiDoCommand(&parms);
    g_ops.push_back(std::string(name) + " " + num(u) + " " + num(player ? 1 : 0) + " " + num(target) + " " + points);
}

static void damage(int t, float amount) {
    Object* o = unit(t);
    if (!o) return;
    // Never to rubble, whose effect on the pathfind map is not the AI's.
    const float spare = o->getBodyModule()->getHealth() - 1.0f;
    if (spare <= 0.0f) return;
    if (amount > spare) amount = spare;
    o->getBodyModule()->internalChangeHealth(-amount);
    g_ops.push_back("damage " + num(t) + " " + fx(amount));
}

static void disable(int u, bool emp, int frames) {
    Object* o = unit(u);
    if (!o) return;
    o->setDisabledUntil(emp ? DISABLED_EMP : DISABLED_UNDERPOWERED, TheGameLogic->getFrame() + frames);
    g_ops.push_back((emp ? "emp " : "under ") + num(u) + " " + num(frames));
}

static void unmanned(int u) {
    Object* o = unit(u);
    if (!o) return;
    o->setDisabled(DISABLED_UNMANNED);
    g_ops.push_back("unmanned " + num(u));
}

static void destroy(int u) {
    Object* o = unit(u);
    if (!o) return;
    TheGameLogic->destroyObject(o);
    g_ops.push_back("destroy " + num(u));
}

static int some(bool (*test)(int)) {
    const size_t n = g_spawns.size();
    const size_t start = pick(static_cast<uint32_t>(n));
    for (size_t k = 0; k < n; ++k) {
        const size_t i = (start + k) % n;
        if (unit(static_cast<int>(i)) && test(g_spawns[i].kind)) return static_cast<int>(i);
    }
    return -1;
}

static int some_building(bool unbuilt, int team) {
    const size_t n = g_spawns.size();
    const size_t start = pick(static_cast<uint32_t>(n));
    for (size_t k = 0; k < n; ++k) {
        const size_t i = (start + k) % n;
        Object* o = unit(static_cast<int>(i));
        if (!o || !is_building(g_spawns[i].kind) || g_spawns[i].team != team) continue;
        if (o->testStatus(OBJECT_STATUS_UNDER_CONSTRUCTION) == unbuilt) return static_cast<int>(i);
    }
    return -1;
}

// The order a builder is most likely to be given.
static void give_work(int u, bool player) {
    const int team = g_spawns[u].team;
    const int unbuilt = some_building(true, team);
    const int built = some_building(false, team);
    if (unbuilt >= 0 && (built < 0 || chance(75)))
        order(u, "resume", AICMD_RESUME_CONSTRUCTION, player, unbuilt, nullptr);
    else if (built >= 0)
        order(u, "repair", AICMD_REPAIR, player, built, nullptr);
}

static void random_op() {
    const int u = some(is_builder);
    if (u < 0) return;
    const bool player = chance(65);
    const int roll = static_cast<int>(pick(100));
    const float x = between(25.0f, 295.0f);
    const float y = between(25.0f, 295.0f);
    const Coord3D to = at(x, y);
    if (roll < 22) {
        give_work(u, player);
    } else if (roll < 28) {
        // Any building, of either side and in any state.
        const int t = some(is_building);
        if (t >= 0) {
            const bool resume = chance(50);
            order(u, resume ? "resume" : "repair", resume ? AICMD_RESUME_CONSTRUCTION : AICMD_REPAIR, player, t,
                  nullptr);
        }
    } else if (roll < 40) {
        order(u, "move", AICMD_MOVE_TO_POSITION, player, -1, &to);
    } else if (roll < 46) {
        order(u, "idle", AICMD_IDLE, player, -1, nullptr);
    } else if (roll < 54) {
        const int other = chance(50) ? some(is_builder) : some([](int k) { return k == K_UNIT; });
        if (other >= 0 && other != u) order(u, "away", AICMD_MOVE_AWAY_FROM_UNIT, false, other, nullptr);
    } else if (roll < 68) {
        const int t = some(is_building);
        if (t >= 0) damage(t, between(5.0f, 60.0f));
    } else if (roll < 78) {
        const bool emp = chance(60);
        disable(u, emp, 5 + static_cast<int>(pick(40)));
    } else if (roll < 80) {
        unmanned(u);
    } else if (roll < 86) {
        const int t = some(is_building);
        if (t >= 0) destroy(t);
    } else if (roll < 88) {
        destroy(u);
    } else {
        // Two orders in one frame.
        give_work(u, player);
        give_work(u, player);
    }
}

// --- scenes -----------------------------------------------------------------

static void spawn_buildings(int count, int team, int mode_a, int mode_b) {
    static const float kSlot[] = {60.0f, 160.0f, 260.0f};
    int taken[9] = {0};
    for (const Spawn& s : g_spawns)
        if (is_building(s.kind))
            for (int c = 0; c < 9; ++c)
                if (std::fabs(s.x - kSlot[c % 3]) < 30.0f && std::fabs(s.y - kSlot[c / 3]) < 30.0f) taken[c] = 1;
    for (int n = 0; n < count; ++n) {
        int c = static_cast<int>(pick(9));
        for (int k = 0; k < 9 && taken[c]; ++k) c = (c + 1) % 9;
        if (taken[c]) return;
        taken[c] = 1;
        Spawn s;
        s.kind = K_HOUSE + static_cast<int>(pick(3));
        s.team = team;
        s.x = kSlot[c % 3] + between(-8.0f, 8.0f);
        s.y = kSlot[c / 3] + between(-8.0f, 8.0f);
        const bool turned = chance(50);
        s.angle = turned ? between(-3.14f, 3.14f) : 0.0f;
        s.mode = chance(50) ? mode_a : mode_b;
        g_spawns.push_back(s);
    }
}

static void spawn_unit(int kind, int team) {
    float x = 0, y = 0;
    for (int tries = 0; tries < 60; ++tries) {
        x = between(25.0f, 295.0f);
        y = between(25.0f, 295.0f);
        bool clear = true;
        for (const Spawn& s : g_spawns)
            if (std::fabs(s.x - x) < 36.0f && std::fabs(s.y - y) < 36.0f) clear = false;
        if (clear) break;
    }
    const float angle = between(-3.14f, 3.14f);
    g_spawns.push_back({kind, team, x, y, angle, 0});
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_s16_dozer_worker_frames <out.json>\n");
        return 2;
    }
    oracle_wp51::World w;
    TheFramePacer = new FramePacer;
    TheCommandList = NEW CommandList;
    TheCommandList->init();
    TheRecorder = new RecorderClass;
    TheRecorder->init();
    TheActionManager = NEW ActionManager;
    S16Terrain* terrain = new S16Terrain;
    TheTerrainLogic = terrain;
    TheTerrainLogic->init();
    TheWritableGlobalData->m_gravity = -1.0f;
    TheWritableGlobalData->m_partitionCellSize = 20.0f;
    ThePartitionManager = NEW PartitionManager;
    ThePartitionManager->init();
    const_cast<TAiData*>(TheAI->getAiData())->m_forceIdleFramesCount = 45;
    const_cast<TAiData*>(TheAI->getAiData())->m_repulsedDistance = 50.0f;
    const_cast<TAiData*>(TheAI->getAiData())->m_aiDozerBoredRadiusModifier = 2.0f;
    w.load_ini(locomotors_ini());
    w.load_ini(weapons_ini());
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

    const int kScenarios = 48;
    const int kFrames = 220;
    static const char* const kSources[] = {
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/AIUpdate/DozerAIUpdate.cpp",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Update/AIUpdate/WorkerAIUpdate.cpp",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/ActionManager.cpp"};
    std::string hashes;
    for (const char* source : kSources) {
        const std::string hash = oracle::sha256_file(source);
        if (hash.size() != 64) return 5;
        hashes += std::string(hashes.empty() ? "" : ",") + "\"" + source + "\":\"" + hash + "\"";
    }
    std::fprintf(out,
                 "{\"source_sha256\":{%s},\"vector_count\":%d,\"frames\":%d,\"cells\":%d,\"locomotors\":\"%s\",\"weapons\":\"%s\","
                 "\"objects\":\"%s\",\"vectors\":[\n",
                 hashes.c_str(), kScenarios, kFrames, kCells, escape(locomotors_ini()).c_str(), escape(weapons_ini()).c_str(),
                 escape(objects_ini()).c_str());

    bool first_row = true;
    for (int scenario = 0; scenario < kScenarios; ++scenario) {
        const int layout = scenario % 4;
        const UnsignedInt drawn = 100 + pick(3000);
        const UnsignedInt frame = scenario % 11 == 0 ? 1 : drawn;
        const UnsignedInt seed = 1 + pick(1000000000);
        g_spawns.clear();
        if (layout == 0) {
            // Construction.
            const int team = scenario % 16 == 8 ? 1 : 0;
            spawn_buildings(3, team, 2, 2);
            spawn_buildings(1, team, 1, 0);
            spawn_unit(scenario % 3 == 0 ? K_WORKER : K_DOZER, team);
            if (chance(60)) spawn_unit(chance(50) ? K_WORKER : K_DOZER, team);
        } else if (layout == 1) {
            // Bored builders and damaged buildings.
            spawn_buildings(3, 0, 1, 1);
            spawn_buildings(3, 1, 1, 0);
            spawn_unit(K_DOZER, 0);
            spawn_unit(K_DOZER, 1);
            spawn_unit(K_WORKER, scenario % 8 < 4 ? 0 : 1);
        } else if (layout == 2) {
            // Mines of the other side.
            spawn_unit(K_DOZER, 0);
            spawn_unit(K_WORKER, 0);
            if (chance(50)) spawn_unit(K_DOZER, 0);
            const size_t builders = g_spawns.size();
            for (size_t b = 0; b < builders; ++b)
                for (int n = static_cast<int>(pick(3)); n > 0; --n) {
                    const float away = between(15.0f, 120.0f);
                    const float x = g_spawns[b].x < 160.0f ? g_spawns[b].x + away : g_spawns[b].x - away;
                    const float y = g_spawns[b].y + between(-15.0f, 15.0f);
                    g_spawns.push_back({K_MINE, 1, x, y, 0.0f, 0});
                }
        } else {
            // Everything at once.
            spawn_buildings(3, 0, 2, 1);
            spawn_buildings(2, 0, 2, 0);
            spawn_buildings(1, 1, 1, 2);
            spawn_unit(K_DOZER, 0);
            spawn_unit(K_WORKER, 0);
            spawn_unit(K_DOZER, chance(60) ? 0 : 1);
            spawn_unit(K_UNIT, 0);
        }

        TheGameLogic->*access_member(Wp51GLFrameTag()) = frame;
        InitRandom(seed);
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1000));
        Pathfinder* pf = TheAI->pathfinder();
        pf->reset();
        // The records outlive the map; start each scene from the same free list.
        PathfindCellInfo::allocateCellInfos();
        pf->newMap();
        // No building of a side is fogged for it, which the port cannot answer.
        ThePartitionManager->revealMapForPlayerPermanently(1);
        ThePartitionManager->revealMapForPlayerPermanently(2);

        g_ids.clear();
        g_ops.clear();
        g_tape.clear();
        std::ostringstream spawn_text;
        for (size_t i = 0; i < g_spawns.size(); ++i) {
            const Spawn& s = g_spawns[i];
            const ThingTemplate* tt = TheThingFactory->findTemplate(kTemplates[s.kind]);
            if (!tt) return 3;
            const bool unbuilt = s.mode == 2;
            ObjectStatusMaskType status;
            if (unbuilt) status.set(OBJECT_STATUS_UNDER_CONSTRUCTION);
            Object* o = TheThingFactory->newObject(tt, s.team ? w.team_b : w.team_a, status);
            if (o->getAI()) replace_states(o->getAI());
            const Coord3D p = at(s.x, s.y);
            o->setOrientation(s.angle);
            o->setPosition(&p);
            float amount = 0.0f;
            if (!is_builder(s.kind) && s.kind != K_UNIT) pf->addObjectToPathfindMap(o);
            if (unbuilt) {
                // As `DozerAIUpdate::construct` leaves a building it ordered.
                o->setConstructionPercent(0.0f);
                BodyModuleInterface* body = o->getBodyModule();
                body->internalChangeHealth(-body->getHealth() + 1.0f);
                o->clearAndSetModelConditionFlags(
                    MAKE_MODELCONDITION_MASK2(MODELCONDITION_PARTIALLY_CONSTRUCTED,
                                              MODELCONDITION_ACTIVELY_BEING_CONSTRUCTED),
                    MAKE_MODELCONDITION_MASK(MODELCONDITION_AWAITING_CONSTRUCTION));
            } else if (s.mode == 1) {
                amount = between(10.0f, 120.0f);
                o->getBodyModule()->internalChangeHealth(-amount);
            }
            g_ids.push_back(o->getID());
            spawn_text << (i ? ";" : "") << s.kind << "," << s.team << "," << c3(p) << "," << fx(s.angle) << ","
                       << s.mode << "," << fx(amount);
        }
        const int units = static_cast<int>(g_ids.size());

        std::fprintf(out, "%s{\"s\":%d,\"frame\":%u,\"seed\":%u,\"spawns\":\"%s\",\"tape\":\"%s\"}",
                     first_row ? "" : ",\n", scenario, static_cast<unsigned>(frame), static_cast<unsigned>(seed),
                     spawn_text.str().c_str(), g_tape.c_str());
        first_row = false;

        std::vector<std::vector<std::string>> last(units);
        for (int f = 0; f < kFrames; ++f) {
            g_ops.clear();
            g_tape.clear();
            if (f == 1 && (layout == 0 || layout == 3)) {
                for (int u = 0; u < units; ++u)
                    if (is_builder(g_spawns[u].kind) && chance(90)) give_work(u, chance(70));
            } else if (f > 1) {
                const int rate = layout == 3 ? 9 : (layout == 0 ? 3 : (layout == 1 ? 2 : 1));
                if (chance(rate)) random_op();
                // Idle builders get work again.
                if (layout == 0 && f % 40 == 0) {
                    const int u = some(is_builder);
                    if (u >= 0 && unit(u)->getAI()->isIdle()) give_work(u, true);
                }
            }
            TheGameLogic->update();
            std::string ops_text;
            for (size_t k = 0; k < g_ops.size(); ++k) ops_text += (k ? ";" : "") + g_ops[k];
            std::ostringstream row;
            row << "{\"s\":" << scenario << ",\"f\":" << f << ",\"ops\":\"" << ops_text << "\",\"tape\":\"" << g_tape
                << "\",\"seed\":\"" << hex32(GetGameLogicRandomSeedCRC()) << "\",\"map\":\"" << cells_digest()
                << "\",\"p\":\"" << players_text() << "\",\"u\":\"";
            for (int i = 0; i < units; ++i) {
                row << (i ? "~" : "") << changed(observe(i), last[i]);
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
