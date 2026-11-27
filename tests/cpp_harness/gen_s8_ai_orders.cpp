// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for the AI module shell, orders, state machine and idle (AI plan
// slice S8): the unmodified reference AIUpdateInterface, AIStateMachine, AIIdleState,
// AIDeadState, AIBusyState, locomotor set and locomotor in a small real world.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_s8_ai_orders.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/S8_AiOrders.json
//
// Every state of the machine other than idle, dead and busy is replaced by a tape state
// that answers with scripted results. The pathfinder, locomotor terrain, mood target,
// repulsor, action manager and edge point calls are answered here with synthetic
// results. Every such call is recorded: the tape the Rust suite replays.
//
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIPathfind.cpp.o __ZN10Pathfinder30chooseBestLocomotorForPositionE17PathfindLayerEnumP12LocomotorSetPK7Coord3D
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIPathfind.cpp.o __ZN10Pathfinder10updateGoalEP6ObjectPK7Coord3D17PathfindLayerEnum
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIPathfind.cpp.o __ZN10Pathfinder12goalPositionEP6ObjectP7Coord3D
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIPathfind.cpp.o __ZN10Pathfinder12snapPositionEP6ObjectP7Coord3D
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIPathfind.cpp.o __ZN10Pathfinder9updatePosEP6ObjectPK7Coord3D
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIPathfind.cpp.o __ZN10Pathfinder12queueForPathE8ObjectID
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIPathfind.cpp.o __ZN10Pathfinder20validMovementTerrainE17PathfindLayerEnumPK9LocomotorPK7Coord3D
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a PartitionManager.cpp.o __ZN16PartitionManager26getGroundOrStructureHeightEff
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a ActionManager.cpp.o __ZN13ActionManager14canEnterObjectEPK6ObjectS2_17CommandSourceType12CanEnterType
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a ActionManager.cpp.o __ZN13ActionManager14canGetHealedAtEPK6ObjectS2_17CommandSourceType
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a ActionManager.cpp.o __ZN13ActionManager16canGetRepairedAtEPK6ObjectS2_17CommandSourceType
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIUpdate.cpp.o __ZN17AIUpdateInterface17getNextMoodTargetEbb
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AI.cpp.o __ZN2AI19findClosestRepulsorEPK6Objectf
//
// Not covered: path requests and searches, doPathfind, the on-path drive, turrets,
// goal squads, polygon guard areas, containers, privateMoveAwayFromUnit, command
// buttons and loading a save.

#include "PreRTS.h"

#include "oracle_wp51_world.h"

#include "Common/ActionManager.h"
#include "Common/Player.h"
#include "Common/RandomValue.h"
#include "Common/StateMachine.h"
#include "GameLogic/AI.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/AIStateMachine.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/LocomotorSet.h"
#include "GameLogic/LogicRandomValue.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <map>
#include <sstream>
#include <string>
#include <vector>

// --- member access ----------------------------------------------------------

using StateMap = std::map<StateID, State*>;
struct StateMapTag { using type = StateMap StateMachine::*; };
template struct MemberAccess<StateMapTag, &StateMachine::m_stateMap>;
StateMapTag::type access_member(StateMapTag);

struct MachineTag { using type = AIStateMachine* AIUpdateInterface::*; };
template struct MemberAccess<MachineTag, &AIUpdateInterface::m_stateMachine>;
MachineTag::type access_member(MachineTag);

using LocoVector = std::vector<Locomotor*>;
struct LocoVectorTag { using type = LocoVector LocomotorSet::*; };
template struct MemberAccess<LocoVectorTag, &LocomotorSet::m_locomotors>;
LocoVectorTag::type access_member(LocoVectorTag);

struct GoalCellTag { using type = ICoord2D AIUpdateInterface::*; };
template struct MemberAccess<GoalCellTag, &AIUpdateInterface::m_pathfindGoalCell>;
GoalCellTag::type access_member(GoalCellTag);

struct DisabledTag { using type = DisabledMaskType Object::*; };
template struct MemberAccess<DisabledTag, &Object::m_disabledMask>;
DisabledTag::type access_member(DisabledTag);

struct VelTag { using type = Coord3D PhysicsBehavior::*; };
template struct MemberAccess<VelTag, &PhysicsBehavior::m_vel>;
VelTag::type access_member(VelTag);

struct LoadAccess : public AIUpdateInterface { using AIUpdateInterface::loadPostProcess; };

// --- the tape ---------------------------------------------------------------

static bool g_recording = false;
// `updatePos` also follows every transform change of an AI object (`Object::
// reactToTransformChange`, not ported); only `loadPostProcess`'s own call is taped.
static bool g_record_pos = false;
static std::string g_tape;

static uint32_t g_state = 0x9e3779b9u;
static uint32_t pick(uint32_t n) {
    g_state = g_state * 1664525u + 1013904223u;
    return (g_state >> 11) % n;
}

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

static void tape(const std::string& entry) {
    if (!g_recording) return;
    if (!g_tape.empty()) g_tape += ";";
    g_tape += entry;
}

static std::string num(long long v) { return std::to_string(v); }
static std::string who(const Object* o) { return o ? num(static_cast<int>(o->getID())) : std::string("0"); }

static std::vector<ObjectID> g_ids;
static Object* unit(int i) {
    if (i < 0 || i >= static_cast<int>(g_ids.size())) return nullptr;
    return TheGameLogic->findObjectByID(g_ids[i]);
}
static Object* random_unit() { return unit(static_cast<int>(pick(static_cast<uint32_t>(g_ids.size())))); }

// --- answers ----------------------------------------------------------------

Locomotor* Pathfinder::chooseBestLocomotorForPosition(PathfindLayerEnum layer, LocomotorSet* set, const Coord3D* pos) {
    LocoVector& all = set->*access_member(LocoVectorTag());
    const uint32_t count = static_cast<uint32_t>(all.size());
    int index = -1;
    if (count != 0 && pick(8) != 0) index = static_cast<int>(pick(count));
    tape("L," + num(static_cast<int>(layer)) + "," + num(count) + "," + c3(*pos) + "=" + num(index));
    return index < 0 ? nullptr : all[index];
}

void Pathfinder::updateGoal(Object* obj, const Coord3D* pos, PathfindLayerEnum layer) {
    Coord3D taped = *pos;
    // For a moving unit `loadPostProcess` builds the point from the goal cell and
    // never sets `z`: the value is whatever the stack holds.
    if (g_record_pos && obj->getAI()->isMoving()) taped.z = 0.0f;
    tape("G," + who(obj) + "," + c3(taped) + "," + num(static_cast<int>(layer)));
}

Bool Pathfinder::goalPosition(Object* obj, Coord3D* pos) {
    if (pick(4) == 0) {
        tape("P," + who(obj) + "=0");
        return FALSE;
    }
    // Sometimes within a cell of the unit, sometimes far from it.
    const Coord3D& at = *obj->getPosition();
    const float reach = pick(2) ? 3.0f : 40.0f;
    pos->x = at.x + (static_cast<float>(pick(201)) - 100.0f) * 0.01f * reach;
    pos->y = at.y + (static_cast<float>(pick(201)) - 100.0f) * 0.01f * reach;
    pos->z = 0.0f;
    tape("P," + who(obj) + "=1," + c3(*pos));
    return TRUE;
}

void Pathfinder::snapPosition(Object* obj, Coord3D* pos) {
    const std::string in = c3(*pos);
    pos->x += 2.5f;
    pos->y -= 1.25f;
    tape("N," + who(obj) + "," + in + "=" + c3(*pos));
}

void Pathfinder::updatePos(Object* obj, const Coord3D* pos) {
    if (g_record_pos) tape("O," + who(obj) + "," + c3(*pos));
}

Bool Pathfinder::queueForPath(ObjectID id) {
    tape("Q," + num(static_cast<int>(id)));
    return TRUE;
}

Bool Pathfinder::validMovementTerrain(PathfindLayerEnum layer, const Locomotor* locomotor, const Coord3D* pos) {
    const Bool valid = pick(10) != 0;
    tape("V," + num(static_cast<int>(layer)) + "," + num(static_cast<int>(locomotor->getLegalSurfaces())) + "," +
         c3(*pos) + "=" + num(valid ? 1 : 0));
    return valid;
}

Real PartitionManager::getGroundOrStructureHeight(Real x, Real y) {
    const Real h = static_cast<Real>(pick(4)) * 5.5f;
    tape("H," + fx(x) + "," + fx(y) + "=" + fx(h));
    return h;
}

static Bool answer_action(int query, const Object* obj, const Object* target, CommandSourceType source) {
    const Bool yes = pick(3) != 0;
    tape("A," + num(query) + "," + who(obj) + "," + who(target) + "," + num(static_cast<int>(source)) + "=" +
         num(yes ? 1 : 0));
    return yes;
}
Bool ActionManager::canEnterObject(const Object* obj, const Object* target, CommandSourceType source, CanEnterType) {
    return answer_action(2, obj, target, source);
}
Bool ActionManager::canGetHealedAt(const Object* obj, const Object* target, CommandSourceType source) {
    return answer_action(0, obj, target, source);
}
Bool ActionManager::canGetRepairedAt(const Object* obj, const Object* target, CommandSourceType source) {
    return answer_action(1, obj, target, source);
}

Object* AIUpdateInterface::getNextMoodTarget(Bool byAI, Bool duringIdle) {
    Object* found = pick(10) == 0 ? random_unit() : nullptr;
    if (found == getObject()) found = nullptr;
    tape("M," + who(getObject()) + "," + num(byAI ? 1 : 0) + "," + num(duringIdle ? 1 : 0) + "=" + who(found));
    return found;
}

Object* AI::findClosestRepulsor(const Object* me, Real range) {
    Object* found = pick(5) == 0 ? random_unit() : nullptr;
    tape("R," + who(me) + "," + fx(range) + "=" + who(found));
    return found;
}

static int scripted_return() {
    const uint32_t r = pick(20);
    if (r < 12) return STATE_CONTINUE;
    if (r < 14) return STATE_SUCCESS;
    if (r < 16) return STATE_FAILURE;
    return 1 + static_cast<int>(pick(90));
}

class TapeState : public State {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(TapeState, "TapeState", 512, 512)
public:
    TapeState(StateMachine* machine, StateID id) : State(machine, "TapeState"), m_tapeId(id) {}
    StateReturnType onEnter() override { return reply("E"); }
    StateReturnType update() override { return reply("U"); }
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
    StateReturnType reply(const char* kind) {
        const int value = scripted_return();
        tape(std::string(kind) + "," + who(getMachineOwner()) + "," + num(m_tapeId) + "=" + num(value));
        return static_cast<StateReturnType>(value);
    }
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
        const bool wander = id == AI_WANDER || id == AI_PANIC;
        StateID success = AI_IDLE, failure = AI_IDLE;
        if (wander) failure = AI_MOVE_AWAY_FROM_REPULSORS;
        if (id == AI_MOVE_AWAY_FROM_REPULSORS) success = failure = AI_WANDER_IN_PLACE;
        if (id == AI_WANDER_IN_PLACE) success = failure = AI_MOVE_AWAY_FROM_REPULSORS;
        if (id == AI_MOVE_AND_EVACUATE_AND_EXIT) success = failure = AI_MOVE_AND_DELETE;
        next->friend_onSuccess(success);
        next->friend_onFailure(failure);
        entry.second = next;
        deleteInstance(old);
    }
}

class S8Terrain : public oracle_wp51::OracleTerrainLogic {
public:
    Coord3D findClosestEdgePoint(const Coord3D* pos) const override {
        Coord3D out = *pos;
        out.x = out.x < 500.0f ? 1.0f : 999.0f;
        tape("D," + c3(*pos) + "=" + c3(out));
        return out;
    }
};

// --- fixture ----------------------------------------------------------------

static std::string locomotor(const char* name, const char* surfaces, const char* appearance, int speed,
                             const char* extra) {
    std::ostringstream s;
    s << "Locomotor " << name << "\n  Surfaces = " << surfaces << "\n  Appearance = " << appearance
      << "\n  Speed = " << speed << "\n  TurnRate = 180\n  Acceleration = 60\n  Braking = 60\n"
      << "  MinSpeed = 0\n" << extra << "End\n";
    return s.str();
}

static std::string locomotors_ini() {
    return locomotor("S8_Legs", "GROUND", "TWO_LEGS", 30, "") +
           locomotor("S8_Fast", "GROUND", "TWO_LEGS", 90, "  AirborneTargetingHeight = 3\n") +
           locomotor("S8_Wheels", "GROUND", "FOUR_WHEELS", 60, "") +
           locomotor("S8_Hover", "AIR", "HOVER", 90,
                     "  PreferredHeight = 50\n  Lift = 120\n  AllowAirborneMotiveForce = Yes\n"
                     "  LocomotorWorksWhenDead = Yes\n  ZAxisBehavior = SURFACE_RELATIVE_HEIGHT\n");
}

static std::string object_ini(const char* name, const char* kind_of, const char* ai_fields, const char* sets,
                              bool physics, const char* extra) {
    std::ostringstream s;
    s << "Object " << name << "\n  KindOf = " << kind_of << "\n  Geometry = CYLINDER\n"
      << "  GeometryMajorRadius = 5\n  GeometryHeight = 8\n  GeometryIsSmall = No\n  VisionRange = 100\n"
      << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n";
    if (physics)
        s << "  Behavior = PhysicsBehavior ModuleTag_Phys\n    Mass = 5\n  End\n";
    if (ai_fields)
        s << "  Behavior = AIUpdateInterface ModuleTag_AI\n" << ai_fields << "  End\n";
    s << extra << sets << "End\n";
    return s.str();
}

static const char* const kTemplates[] = {"S8_Inf",  "S8_Veh",  "S8_Air", "S8_Fixed", "S8_Shy",
                                         "S8_Proj", "S8_Repulsed", "S8_Quiet", "S8_Crate"};
static const int kAiKinds = 8;
static const int kCrate = 8;

static std::string objects_ini() {
    return object_ini("S8_Inf", "PRELOAD SELECTABLE INFANTRY", "",
                      "  Locomotor = SET_NORMAL S8_Legs\n  Locomotor = SET_PANIC S8_Fast\n", true, "") +
           object_ini("S8_Veh", "PRELOAD SELECTABLE VEHICLE",
                      "    AutoAcquireEnemiesWhenIdle = YES\n    MoodAttackCheckRate = 500\n",
                      "  Locomotor = SET_NORMAL S8_Wheels S8_Fast\n"
                      "  Locomotor = SET_NORMAL_UPGRADED S8_Fast S8_Wheels\n  Locomotor = SET_WANDER S8_Legs\n",
                      true, "") +
           object_ini("S8_Air", "PRELOAD SELECTABLE AIRCRAFT", "", "  Locomotor = SET_NORMAL S8_Hover\n", true,
                      "") +
           object_ini("S8_Fixed", "PRELOAD STRUCTURE IMMOBILE", "", "  Locomotor = SET_NORMAL S8_Legs\n", false,
                      "") +
           object_ini("S8_Shy", "PRELOAD VEHICLE", "    ForbidPlayerCommands = Yes\n",
                      "  Locomotor = SET_NORMAL S8_Wheels\n", true, "") +
           object_ini("S8_Proj", "PRELOAD PROJECTILE", "", "  Locomotor = SET_NORMAL S8_Legs\n", true, "") +
           object_ini("S8_Repulsed", "PRELOAD INFANTRY CAN_BE_REPULSED", "",
                      "  Locomotor = SET_NORMAL S8_Legs\n", true, "") +
           object_ini("S8_Quiet", "PRELOAD INFANTRY IGNORED_IN_GUI", "", "  Locomotor = SET_NORMAL S8_Legs\n",
                      false, "") +
           object_ini("S8_Crate", "PRELOAD CRATE", nullptr, "", true,
                      "  Behavior = MoneyCrateCollide ModuleTag_Crate\n    MoneyProvided = 100\n    RequiredKindOf = INFANTRY\n  End\n");
}

// --- observation ------------------------------------------------------------

static std::vector<std::string> g_last;

static std::string observe(int i) {
    Object* o = unit(i);
    if (!o) return "gone";
    AIUpdateInterface* ai = o->getAI();
    if (!ai) {
        return "p" + c3(*o->getPosition());
    }
    uint32_t unused = 0;
    const std::string save = oracle_wp51::World::snapshot_save_hex(ai, &unused);
    const uint32_t crc = oracle_wp51::World::snapshot_crc(ai);
    std::ostringstream s;
    PhysicsBehavior* phys = o->getPhysics();
    // The mood matrix and its adjustment of every action; what the unit is.
    std::string mood = hex32(ai->getMoodMatrixValue());
    for (int action = MM_Action_Idle; action <= MM_Action_AttackMove; ++action)
        mood += "." + hex32(ai->getMoodMatrixActionAdjustment(static_cast<MoodMatrixAction>(action)));
    s << hex32(crc) << "," << mood << "," << c3(*o->getPosition()) << "," << (o->testStatus(OBJECT_STATUS_AIRBORNE_TARGET) ? 1 : 0)
      << "," << (o->getAI()->isIdle() ? 1 : 0) << (ai->isMoving() ? 1 : 0) << (o->isEffectivelyDead() ? 1 : 0)
      << "," << (phys ? c3(phys->*access_member(VelTag())) : "-")
      << "," << save;
    return s.str();
}

// --- operations -------------------------------------------------------------

struct Command { AICommandType type; };
static const AICommandType kCommands[] = {
    AICMD_MOVE_TO_POSITION, AICMD_MOVE_TO_POSITION_EVEN_IF_SLEEPING, AICMD_MOVE_TO_OBJECT,
    AICMD_TIGHTEN_TO_POSITION, AICMD_MOVE_TO_POSITION_AND_EVACUATE,
    AICMD_MOVE_TO_POSITION_AND_EVACUATE_AND_EXIT, AICMD_IDLE, AICMD_FOLLOW_WAYPOINT_PATH,
    AICMD_FOLLOW_WAYPOINT_PATH_AS_TEAM, AICMD_FOLLOW_WAYPOINT_PATH_EXACT,
    AICMD_FOLLOW_WAYPOINT_PATH_AS_TEAM_EXACT, AICMD_FOLLOW_PATH, AICMD_FOLLOW_PATH_APPEND,
    AICMD_FOLLOW_EXITPRODUCTION_PATH, AICMD_ATTACK_OBJECT, AICMD_FORCE_ATTACK_OBJECT, AICMD_GUARD_RETALIATE,
    AICMD_ATTACK_POSITION, AICMD_ATTACKMOVE_TO_POSITION, AICMD_ATTACKFOLLOW_WAYPOINT_PATH,
    AICMD_ATTACKFOLLOW_WAYPOINT_PATH_AS_TEAM, AICMD_HUNT, AICMD_ATTACK_AREA, AICMD_REPAIR,
    AICMD_RESUME_CONSTRUCTION, AICMD_GET_HEALED, AICMD_GET_REPAIRED, AICMD_ENTER, AICMD_DOCK, AICMD_EXIT,
    AICMD_EXIT_INSTANTLY, AICMD_EVACUATE, AICMD_EVACUATE_INSTANTLY, AICMD_EXECUTE_RAILED_TRANSPORT,
    AICMD_GUARD_POSITION, AICMD_GUARD_OBJECT, AICMD_GUARD_TUNNEL_NETWORK, AICMD_HACK_INTERNET,
    AICMD_FACE_OBJECT, AICMD_FACE_POSITION, AICMD_RAPPEL_INTO, AICMD_COMBATDROP, AICMD_WANDER,
    AICMD_WANDER_IN_PLACE, AICMD_PANIC, AICMD_BUSY,
};
static const int kCommandCount = static_cast<int>(sizeof(kCommands) / sizeof(kCommands[0]));
static const int kIdleCommand = 6;

static float coordinate() {
    // Mostly inside the map, sometimes in the border or outside.
    const uint32_t r = pick(10);
    if (r == 0) return -50.0f + static_cast<float>(pick(100));
    if (r == 1) return 950.0f + static_cast<float>(pick(100));
    return 20.0f + static_cast<float>(pick(96000)) * 0.01f;
}
static Coord3D random_point() {
    Coord3D p;
    p.x = coordinate();
    p.y = coordinate();
    p.z = 0.0f;
    return p;
}

// Runs one random operation and returns its text.
static std::string operate(int units) {
    const int u = static_cast<int>(pick(units));
    Object* o = unit(u);
    AIUpdateInterface* ai = o ? o->getAI() : nullptr;
    if (!ai) return "";
    const int other = static_cast<int>(pick(static_cast<uint32_t>(g_ids.size())));
    const uint32_t r = pick(100);
    std::ostringstream op;
    if (r < 28) {
        const int cmd = pick(5) == 0 ? kIdleCommand : static_cast<int>(pick(kCommandCount));
        const int source = static_cast<int>(pick(3));  // player, script, AI
        const bool needs_target = kCommands[cmd] == AICMD_GET_HEALED || kCommands[cmd] == AICMD_GET_REPAIRED ||
                                  kCommands[cmd] == AICMD_ENTER;
        const int target = (!needs_target && pick(8) == 0) ? -1 : other;
        const Coord3D pos = random_point();
        const int value = static_cast<int>(pick(3));
        std::vector<Coord3D> path;
        const uint32_t n = pick(4);
        for (uint32_t i = 0; i < n; ++i) path.push_back(random_point());
        op << "c " << u << " " << cmd << " " << source << " " << target << " " << c3(pos) << " " << value << " " << n;
        for (const Coord3D& p : path) op << " " << c3(p);
        AICommandParms parms(kCommands[cmd], static_cast<CommandSourceType>(source));
        parms.m_pos = pos;
        parms.m_obj = unit(target);
        parms.m_intValue = value;
        parms.m_coords = path;
        g_recording = true;
        ai->aiDoCommand(&parms);
    } else if (r < 62) {
        op << "t " << u;
        g_recording = true;
        const UpdateSleepTime sleep = ai->update();
        op << " " << static_cast<unsigned>(sleep);
        TheGameLogic->friend_awakenUpdateModule(o, ai, TheGameLogic->getFrame() + sleep);
    } else if (r < 70) {
        const uint32_t n = 1 + (pick(4) == 0 ? pick(200) : pick(4));
        op << "f " << n;
        TheGameLogic->*access_member(Wp51GLFrameTag()) = TheGameLogic->getFrame() + n;
    } else if (r < 71) {
        op << "dead " << u;
        g_recording = true;
        ai->markAsDead();
    } else if (r < 73) {
        const int set = pick(2) ? LOCOMOTORSET_NORMAL : static_cast<int>(pick(LOCOMOTORSET_COUNT));
        op << "loco " << u << " " << set;
        g_recording = true;
        ai->chooseLocomotorSet(static_cast<LocomotorSetType>(set));
    } else if (r < 74) {
        const int on = static_cast<int>(pick(2));
        op << "upg " << u << " " << on;
        g_recording = true;
        ai->setLocomotorUpgrade(on);
    } else if (r < 76) {
        const int attitude = static_cast<int>(pick(5)) - 2;
        op << "att " << u << " " << attitude;
        ai->setAttitude(static_cast<AttitudeType>(attitude));
    } else if (r < 77) {
        op << "ign " << u << " " << other;
        ai->ignoreObstacleID(g_ids[other]);
    } else if (r < 79) {
        const Coord3D p = random_point();
        const int count = pick(10) == 0 ? 20 : 1;
        op << "wq " << u << " " << c3(p) << " " << count;
        for (int i = 0; i < count; ++i) ai->queueWaypoint(&p);
        if (pick(2)) {
            ai->executeWaypointQueue();
            op << " 1";
        } else {
            op << " 0";
        }
    } else if (r < 82) {
        static const int kDisabled[] = {DISABLED_HELD, DISABLED_PARALYZED, DISABLED_UNMANNED, DISABLED_EMP,
                                        DISABLED_SUBDUED, DISABLED_HACKED, DISABLED_UNDERPOWERED};
        const int type = kDisabled[pick(7)];
        const int on = pick(3) == 0 ? 1 : 0;
        op << "dis " << u << " " << type << " " << on;
        (o->*access_member(DisabledTag())).set(type, on);
    } else if (r < 85) {
        const int end = static_cast<int>(pick(2));
        op << "move " << u << " " << end;
        if (end)
            ai->friend_endingMove();
        else
            ai->friend_startingMove();
    } else if (r < 87) {
        const int frames = static_cast<int>(pick(3) == 0 ? 0 : 1 + pick(60));
        op << "qp " << u << " " << frames;
        g_recording = true;
        ai->setQueueForPathTime(frames);
    } else if (r < 90) {
        const uint32_t kind = pick(3);
        if (kind == 0) {
            const Coord3D p = random_point();
            op << "gp " << u << " " << c3(p);
            ai->setLocomotorGoalPositionExplicit(p);
        } else if (kind == 1) {
            const float angle = (static_cast<float>(pick(628)) - 314.0f) * 0.01f;
            op << "ga " << u << " " << fx(angle);
            ai->setLocomotorGoalOrientation(angle);
        } else {
            op << "gn " << u;
            ai->setLocomotorGoalNone();
        }
    } else if (r < 91) {
        const float speed = static_cast<float>(pick(5));
        op << "speed " << u << " " << fx(speed);
        ai->setDesiredSpeed(speed);
    } else if (r < 92) {
        op << "join " << u;
        g_recording = true;
        ai->joinTeam();
    } else if (r < 93) {
        op << "wake " << u;
        ai->wakeUpAndAttemptToTarget();
    } else if (r < 94) {
        const int target = pick(3) == 0 ? -1 : other;
        op << "goalobj " << u << " " << target;
        ai->friend_setGoalObject(unit(target));
    } else if (r < 95) {
        std::vector<Coord3D> path;
        const uint32_t n = 1 + pick(3);
        for (uint32_t i = 0; i < n; ++i) path.push_back(random_point());
        op << "quick " << u << " " << n;
        for (const Coord3D& p : path) op << " " << c3(p);
        g_recording = true;
        ai->doQuickExit(&path);
    } else if (r < 96) {
        op << "destroy " << u;
        ai->destroyPath();
    } else if (r < 97) {
        // The crate is the last object.
        op << "notify " << u << " " << static_cast<int>(g_ids.size()) - 1;
        ai->notifyCrate(g_ids.back());
    } else if (r < 99) {
        Coord3D p = random_point();
        p.z = pick(3) == 0 ? static_cast<float>(pick(60)) : 0.0f;
        const float v = static_cast<float>(pick(7)) - 3.0f;
        op << "pos " << u << " " << c3(p) << " " << fx(v);
        o->setPosition(&p);
        if (PhysicsBehavior* phys = o->getPhysics()) {
            (phys->*access_member(VelTag())).x = v;
            (phys->*access_member(VelTag())).y = v * 0.5f;
        }
    } else {
        ICoord2D cell;
        cell.x = static_cast<Int>(pick(4)) - 1;
        cell.y = static_cast<Int>(pick(4)) - 1;
        op << "load " << u << " " << cell.x << " " << cell.y;
        ai->*access_member(GoalCellTag()) = cell;
        g_recording = true;
        g_record_pos = true;
        (ai->*(&LoadAccess::loadPostProcess))();
        g_record_pos = false;
    }
    g_recording = false;
    return op.str();
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_s8_ai_orders <out.json>\n");
        return 2;
    }
    oracle_wp51::World w;
    TheTerrainLogic = NEW S8Terrain;
    TheTerrainLogic->init();
    TheWritableGlobalData->m_gravity = -1.0f;
    TheActionManager = NEW ActionManager;
    const_cast<TAiData*>(TheAI->getAiData())->m_forceIdleFramesCount = 45;
    w.load_ini(locomotors_ini());
    w.load_ini(objects_ini());
    w.init_with_neutral_player();

    const std::string tmp = std::string(argv[1]) + ".tmp";
    FILE* out = std::fopen(tmp.c_str(), "w");
    if (!out) return 1;
    auto escape = [](const std::string& text) {
        std::string escaped;
        for (char c : text) {
            if (c == '\n') escaped += "\\n";
            else escaped += c;
        }
        return escaped;
    };

    const int kScenarios = 220;
    const int kSteps = 36;
    std::fprintf(out, "{\"vector_count\":%d,\"locomotors\":\"%s\",\"objects\":\"%s\",\"vectors\":[\n", kScenarios,
                 escape(locomotors_ini()).c_str(), escape(objects_ini()).c_str());

    for (int scenario = 0; scenario < kScenarios; ++scenario) {
        const UnsignedInt frame = pick(3) == 0 ? 1 : 2 + pick(3000);
        const UnsignedInt seed = 1 + pick(1000000000);
        TheGameLogic->*access_member(Wp51GLFrameTag()) = frame;
        InitRandom(seed);
        const int human_a = static_cast<int>(pick(2));
        const int human_b = static_cast<int>(pick(2));
        ThePlayerList->getNthPlayer(1)->setPlayerType(human_a ? PLAYER_HUMAN : PLAYER_COMPUTER, FALSE);
        ThePlayerList->getNthPlayer(2)->setPlayerType(human_b ? PLAYER_HUMAN : PLAYER_COMPUTER, FALSE);

        const ObjectID first_id = static_cast<ObjectID>(1000);
        TheGameLogic->setObjectIDCounter(first_id);
        g_ids.clear();
        const int units = 2 + static_cast<int>(pick(2));
        std::ostringstream spawns;
        std::string created;
        for (int i = 0; i <= units; ++i) {
            const int kind = i == units ? kCrate : static_cast<int>(scenario % kAiKinds == 0 || i == 0
                                                       ? (scenario / 2 + i) % kAiKinds
                                                       : pick(kAiKinds));
            const int team = static_cast<int>(pick(4) == 0 ? 1 : 0);
            Coord3D p = random_point();
            if (kind == 2) p.z = pick(2) ? 40.0f : 0.0f;
            const float angle = static_cast<float>(pick(628)) * 0.01f;
            const ThingTemplate* tt = TheThingFactory->findTemplate(kTemplates[kind]);
            if (!tt) return 3;
            g_tape.clear();
            g_recording = true;
            Object* o = TheThingFactory->newObject(tt, team ? w.team_b : w.team_a);
            g_recording = false;
            o->setPosition(&p);
            o->setOrientation(angle);
            g_ids.push_back(o->getID());
            if (o->getAI()) replace_states(o->getAI());
            spawns << (i ? ";" : "") << kind << "," << team << "," << c3(p) << "," << fx(angle);
            created += (i ? "|" : "") + g_tape;
        }

        std::fprintf(out,
                     "%s{\"i\":%d,\"frame\":%u,\"seed\":%u,\"humans\":\"%d%d\",\"spawns\":\"%s\",\"created\":\"%s\"",
                     scenario ? ",\n" : "", scenario, static_cast<unsigned>(frame), static_cast<unsigned>(seed),
                     human_a, human_b, spawns.str().c_str(), created.c_str());
        g_last.assign(g_ids.size(), "");
        std::string steps;
        std::ostringstream first;
        first << "init~~" << hex32(GetGameLogicRandomSeedCRC());
        for (size_t i = 0; i < g_ids.size(); ++i) {
            g_last[i] = observe(static_cast<int>(i));
            first << "~" << g_last[i];
        }
        steps = first.str();
        for (int step = 0; step < kSteps;) {
            g_tape.clear();
            const std::string op = operate(units);
            if (op.empty()) continue;
            ++step;
            std::ostringstream record;
            record << op << "~" << g_tape << "~" << hex32(GetGameLogicRandomSeedCRC());
            for (size_t i = 0; i < g_ids.size(); ++i) {
                const std::string now = observe(static_cast<int>(i));
                record << "~" << (now == g_last[i] ? std::string("=") : now);
                g_last[i] = now;
            }
            steps += "#" + record.str();
        }
        std::fprintf(out, ",\"steps\":\"%s\"}", steps.c_str());
        std::fflush(out);

        for (ObjectID id : g_ids)
            if (Object* o = TheGameLogic->findObjectByID(id)) TheGameLogic->destroyObject(o);
        w.flush_destroy_list();
    }
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    return std::rename(tmp.c_str(), argv[1]) == 0 ? 0 : 1;
}
