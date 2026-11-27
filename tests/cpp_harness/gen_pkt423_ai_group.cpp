// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for AIGroup and Squad (PKT-423, PKT-409): the unmodified reference
// AIGroup, Squad, AI::createGroup/destroyGroup and the object group links, over real
// objects with real AI modules.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt423_ai_group.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/PKT-423.json
//
// `aiDoCommand` is replaced (in each unit's vtable) by a recorder, so each order a group hands out is seen with
// its parameters. The pathfinder calls of the groups are answered here with synthetic
// results and recorded. The terrain has sloped ground and two different extents.
//
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIPathfind.cpp.o __ZN10Pathfinder30chooseBestLocomotorForPositionE17PathfindLayerEnumP12LocomotorSetPK7Coord3D
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIPathfind.cpp.o __ZN10Pathfinder10removeGoalEP6Object
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIPathfind.cpp.o __ZN10Pathfinder10updateGoalEP6ObjectPK7Coord3D17PathfindLayerEnum
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIPathfind.cpp.o __ZN10Pathfinder9updatePosEP6ObjectPK7Coord3D
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIPathfind.cpp.o __ZN10Pathfinder17adjustDestinationEP6ObjectRK12LocomotorSetP7Coord3DPKS5_
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIPathfind.cpp.o __ZN10Pathfinder14isLinePassableEPK6Objecti17PathfindLayerEnumRK7Coord3DS6_bb
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIPathfind.cpp.o __ZN10Pathfinder14findGroundPathEPK7Coord3DS2_ib
//
// Not covered: containers and their passengers, spawn slaves, special powers,
// upgrades, selling, overcharge,
// cheering, firing and loading a save. Commands go through the unmodified
// `GameLogic::logicMessageDispatcher` with the players' real selections.

#include "PreRTS.h"

#include "oracle_wp51_world.h"

#include "Common/MessageStream.h"

#include "Common/ActionManager.h"
#include "Common/Player.h"
#include "Common/Recorder.h"
#include "Common/RandomValue.h"
#include "Common/StateMachine.h"
#include "GameLogic/AI.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/AIStateMachine.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/LocomotorSet.h"
#include "GameLogic/LogicRandomValue.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/Squad.h"
#include "GameLogic/Weapon.h"

#include <cstdio>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

// --- member access ----------------------------------------------------------

struct LocoVectorTag { using type = std::vector<Locomotor*> LocomotorSet::*; };
template struct MemberAccess<LocoVectorTag, &LocomotorSet::m_locomotors>;
LocoVectorTag::type access_member(LocoVectorTag);

struct DisabledTag { using type = DisabledMaskType Object::*; };
template struct MemberAccess<DisabledTag, &Object::m_disabledMask>;
DisabledTag::type access_member(DisabledTag);

struct MoodTimeTag { using type = UnsignedInt AIUpdateInterface::*; };
template struct MemberAccess<MoodTimeTag, &AIUpdateInterface::m_nextMoodCheckTime>;
MoodTimeTag::type access_member(MoodTimeTag);

struct MoodRandTag { using type = Bool AIUpdateInterface::*; };
template struct MemberAccess<MoodRandTag, &AIUpdateInterface::m_randomlyOffsetMoodCheck>;
MoodRandTag::type access_member(MoodRandTag);

struct MachineTag { using type = AIStateMachine* AIUpdateInterface::*; };
template struct MemberAccess<MachineTag, &AIUpdateInterface::m_stateMachine>;
MachineTag::type access_member(MachineTag);

struct FormationTag { using type = FormationID Object::*; };
template struct MemberAccess<FormationTag, &Object::m_formationID>;
FormationTag::type access_member(FormationTag);

struct OffsetTag { using type = Coord2D Object::*; };
template struct MemberAccess<OffsetTag, &Object::m_formationOffset>;
OffsetTag::type access_member(OffsetTag);

struct GameModeTag { using type = GameMode GameLogic::*; };
template struct MemberAccess<GameModeTag, &GameLogic::m_gameMode>;
GameModeTag::type access_member(GameModeTag);

struct WeaponSetTag { using type = WeaponSet Object::*; };
template struct MemberAccess<WeaponSetTag, &Object::m_weaponSet>;
WeaponSetTag::type access_member(WeaponSetTag);

struct LockTag { using type = WeaponLockType WeaponSet::*; };
template struct MemberAccess<LockTag, &WeaponSet::m_curWeaponLockedStatus>;
LockTag::type access_member(LockTag);

struct NextGroupTag { using type = UnsignedInt AI::*; };
template struct MemberAccess<NextGroupTag, &AI::m_nextGroupID>;
NextGroupTag::type access_member(NextGroupTag);

// --- text -------------------------------------------------------------------

static uint32_t g_state = 0x2545f491u;
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
static std::string num(long long v) { return std::to_string(v); }
static std::string who(const Object* o) { return o ? num(static_cast<int>(o->getID())) : std::string("0"); }

static bool g_recording = false;
static std::string g_tape;
static void tape(const std::string& entry) {
    if (!g_recording) return;
    if (!g_tape.empty()) g_tape += ";";
    g_tape += entry;
}

// --- terrain ----------------------------------------------------------------

class GroupTerrain : public oracle_wp51::OracleTerrainLogic {
public:
    void getExtent(Region3D* out) const override {
        out->lo = {0.0f, 0.0f, 0.0f};
        out->hi = {2000.0f, 2000.0f, 100.0f};
    }
    void getExtentIncludingBorder(Region3D* out) const override { getExtent(out); }
    void getMaximumPathfindExtent(Region3D* out) const override {
        out->lo = {60.0f, 40.0f, 0.0f};
        out->hi = {1850.0f, 1900.0f, 100.0f};
    }
    Real getGroundHeight(Real x, Real y, Coord3D* normal = nullptr) const override {
        if (normal) { normal->x = 0.0f; normal->y = 0.0f; normal->z = 1.0f; }
        return x * 0.01f + y * 0.02f;
    }
};

// --- the recorded engine ----------------------------------------------------

static const char* command_name(AICommandType cmd) {
    switch (cmd) {
        case AICMD_MOVE_TO_POSITION: return "move";
        case AICMD_TIGHTEN_TO_POSITION: return "tighten";
        case AICMD_MOVE_TO_POSITION_AND_EVACUATE: return "moveevac";
        case AICMD_MOVE_TO_POSITION_AND_EVACUATE_AND_EXIT: return "moveevacexit";
        case AICMD_IDLE: return "idle";
        case AICMD_FOLLOW_PATH: return "path";
        case AICMD_FOLLOW_PATH_APPEND: return "append";
        case AICMD_ATTACK_OBJECT: return "attack";
        case AICMD_FORCE_ATTACK_OBJECT: return "forceattack";
        case AICMD_ATTACK_POSITION: return "attackpos";
        case AICMD_ATTACKMOVE_TO_POSITION: return "attackmove";
        case AICMD_HUNT: return "hunt";
        case AICMD_REPAIR: return "repair";
        case AICMD_RESUME_CONSTRUCTION: return "resume";
        case AICMD_GET_HEALED: return "healed";
        case AICMD_GET_REPAIRED: return "repaired";
        case AICMD_ENTER: return "enter";
        case AICMD_DOCK: return "dock";
        case AICMD_EXIT: return "exit";
        case AICMD_EVACUATE: return "evacuate";
        case AICMD_EXECUTE_RAILED_TRANSPORT: return "rail";
        case AICMD_GUARD_POSITION: return "guardpos";
        case AICMD_GUARD_OBJECT: return "guardobj";
        case AICMD_HACK_INTERNET: return "hack";
        case AICMD_COMBATDROP: return "drop";
        default: return "?";
    }
}

struct ObjectAccess : public AIUpdateInterface { Object* owner() { return getObject(); } };

// Takes the place of `aiDoCommand` in each unit's `AICommandInterface` vtable (its only
// slot). A symbol wrap would not reach calls through that table.
static void record_command(AICommandInterface* iface, const AICommandParms* p) {
    AIUpdateInterface* self = static_cast<AIUpdateInterface*>(iface);
    std::string e = "o," + who(static_cast<ObjectAccess*>(self)->owner()) + "," + command_name(p->m_cmd) + "," +
                    num(static_cast<int>(p->m_cmdSource)) + "," + c3(p->m_pos) + "," + who(p->m_obj) + "," +
                    num(p->m_intValue) + "," + num(static_cast<long long>(p->m_coords.size()));
    for (const Coord3D& c : p->m_coords) e += "," + c3(c);
    tape(e);
}

Locomotor* Pathfinder::chooseBestLocomotorForPosition(PathfindLayerEnum, LocomotorSet* set, const Coord3D*) {
    std::vector<Locomotor*>& all = set->*access_member(LocoVectorTag());
    return all.empty() ? nullptr : all[0];
}
void Pathfinder::removeGoal(Object* obj) { tape("R," + who(obj)); }
void Pathfinder::updateGoal(Object* obj, const Coord3D* pos, PathfindLayerEnum layer) {
    tape("G," + who(obj) + "," + c3(*pos) + "," + num(static_cast<int>(layer)));
}
void Pathfinder::updatePos(Object* obj, const Coord3D* pos) {
    // The transform changes of the harness's own set-up are not the group's calls.
    tape("P," + who(obj) + "," + c3(*pos));
}
Bool Pathfinder::adjustDestination(Object* obj, const LocomotorSet&, Coord3D* dest, const Coord3D* group) {
    const std::string before = c3(*dest);
    dest->x += (static_cast<float>(pick(5)) - 2.0f) * 0.5f;
    dest->y += (static_cast<float>(pick(5)) - 2.0f) * 0.25f;
    dest->z += static_cast<float>(pick(3)) * 0.125f;
    tape("A," + who(obj) + "," + before + "," + (group ? c3(*group) : std::string("-")) + "=" + c3(*dest));
    return TRUE;
}
Bool Pathfinder::isLinePassable(const Object* obj, LocomotorSurfaceTypeMask surfaces, PathfindLayerEnum layer,
                                const Coord3D& from, const Coord3D& to, Bool blocked, Bool pinched) {
    const Bool answer = pick(3) != 0;
    tape("L," + who(obj) + "," + num(surfaces) + "," + num(static_cast<int>(layer)) + "," + c3(from) + "," +
         c3(to) + "," + num(blocked) + num(pinched) + "=" + num(answer));
    return answer;
}
Path* Pathfinder::findGroundPath(const Coord3D* from, const Coord3D* to, Int diameter, Bool crusher) {
    std::vector<Coord3D> nodes;
    if (pick(6) != 0) {
        const uint32_t middle = pick(9);
        nodes.push_back(*from);
        for (uint32_t i = 1; i <= middle; ++i) {
            const float t = static_cast<float>(i) / static_cast<float>(middle + 1);
            Coord3D n;
            n.x = from->x + (to->x - from->x) * t + (static_cast<float>(pick(121)) - 60.0f);
            n.y = from->y + (to->y - from->y) * t + (static_cast<float>(pick(121)) - 60.0f);
            n.z = static_cast<float>(pick(10));
            // Some nodes bunch up near an end.
            if (pick(6) == 0) {
                const Coord3D* end = pick(2) ? from : to;
                n.x = end->x + static_cast<float>(pick(41)) - 20.0f;
                n.y = end->y + static_cast<float>(pick(41)) - 20.0f;
            }
            nodes.push_back(n);
        }
        nodes.push_back(*to);
    }
    std::string e = "F," + c3(*from) + "," + c3(*to) + "," + num(diameter) + num(crusher) + "=" +
                    num(static_cast<long long>(nodes.size()));
    for (const Coord3D& n : nodes) e += "," + c3(n);
    tape(e);
    if (nodes.empty()) return nullptr;
    Path* path = newInstance(Path);
    for (const Coord3D& n : nodes) path->appendNode(&n, LAYER_GROUND);
    for (PathNode* n = path->getFirstNode(); n && n->getNext(); n = n->getNext()) n->setNextOptimized(n->getNext());
    size_t count = 0;
    for (PathNode* n = path->getFirstNode(); n; n = n->getNextOptimized()) ++count;
    if (count != nodes.size()) {
        std::fprintf(stderr, "oracle: path lost nodes\n");
        std::abort();
    }
    return path;
}

// --- the world --------------------------------------------------------------

static std::string locomotor(const char* name, const char* surfaces, const char* appearance, int speed,
                             const char* extra) {
    std::ostringstream s;
    s << "Locomotor " << name << "\n  Surfaces = " << surfaces << "\n  Appearance = " << appearance
      << "\n  Speed = " << speed << "\n  SpeedDamaged = " << speed / 2 << "\n  TurnRate = 180\n  Acceleration = 60\n"
      << "  Braking = 60\n  MinTurnSpeed = 0\n" << extra << "End\n";
    return s.str();
}

static std::string locomotors_ini() {
    return locomotor("G_Foot", "GROUND", "TWO_LEGS", 30, "") +
           locomotor("G_FootFront", "GROUND", "TWO_LEGS", 36, "  GroupMovementPriority = MOVES_FRONT\n") +
           locomotor("G_FootBack", "GROUND", "TWO_LEGS", 24, "  GroupMovementPriority = MOVES_BACK\n") +
           locomotor("G_Wheel", "GROUND", "FOUR_WHEELS", 90, "") +
           locomotor("G_Track", "GROUND CLIFF", "TREADS", 60, "") +
           locomotor("G_Air", "AIR", "HOVER", 120, "  PreferredHeight = 50\n  Lift = 120\n");
}

static std::string weapons_ini() {
    const char* body = "\n  PrimaryDamage = 10\n  AttackRange = 120\n  DelayBetweenShots = 500\nEnd\n";
    return std::string("Weapon G_Gun") + body + "Weapon G_Cannon" + body;
}

// A `WeaponSet` block with the given slots.
static std::string arms(const char* slots) {
    std::string s = "  WeaponSet\n    Conditions = None\n";
    for (const char* c = slots; *c; ++c)
        s += std::string("    Weapon = ") + (*c == '1' ? "PRIMARY G_Gun\n" : *c == '2' ? "SECONDARY G_Cannon\n" : "TERTIARY G_Gun\n");
    return s + "  End\n";
}

static std::string object_ini(const char* name, const char* kind_of, int radius, const char* loco,
                              const char* ai_fields, const char* extra) {
    std::ostringstream s;
    s << "Object " << name << "\n  KindOf = " << kind_of << "\n  Geometry = CYLINDER\n"
      << "  GeometryMajorRadius = " << radius << "\n  GeometryHeight = 8\n  GeometryIsSmall = No\n"
      << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n";
    if (ai_fields) s << "  Behavior = AIUpdateInterface ModuleTag_AI\n" << ai_fields << "  End\n";
    s << extra;
    if (loco) s << "  Locomotor = SET_NORMAL " << loco << "\n";
    s << "End\n";
    return s.str();
}

static const char* const kTemplates[] = {"G_Inf",   "G_InfFront", "G_InfBack", "G_Veh",    "G_Tank",
                                         "G_Cliff", "G_Heli",     "G_Jet",     "G_Fixed",  "G_House",
                                         "G_Rock",  "G_Sneak",    "G_Bold",    "G_Granted", "G_Nexus",
                                         "G_Tower"};
static const int kKinds = 16;

static std::string stealth(const char* extra) {
    return std::string("  Behavior = StealthUpdate ModuleTag_Stealth\n    StealthDelay = 2000\n") + extra + "  End\n";
}

static std::string objects_ini() {
    const std::string sneak = stealth("");
    const std::string granted = stealth("    GrantedBySpecialPower = Yes\n");
    const char* acquire = "    AutoAcquireEnemiesWhenIdle = YES\n";
    const char* acquire_stealthed = "    AutoAcquireEnemiesWhenIdle = YES STEALTHED\n";
    return object_ini("G_Inf", "PRELOAD SELECTABLE INFANTRY CAN_ATTACK", 5, "G_Foot", "", arms("12").c_str()) +
           object_ini("G_InfFront", "PRELOAD SELECTABLE INFANTRY", 4, "G_FootFront", "", "") +
           object_ini("G_InfBack", "PRELOAD SELECTABLE INFANTRY CAN_ATTACK", 6, "G_FootBack", "", arms("1").c_str()) +
           object_ini("G_Veh", "PRELOAD SELECTABLE VEHICLE CAN_ATTACK", 12, "G_Wheel", acquire, arms("123").c_str()) +
           object_ini("G_Tank", "PRELOAD SELECTABLE VEHICLE", 18, "G_Track", "", arms("2").c_str()) +
           object_ini("G_Cliff", "PRELOAD SELECTABLE VEHICLE CLIFF_JUMPER", 9, "G_Track", "", "") +
           object_ini("G_Heli", "PRELOAD SELECTABLE VEHICLE AIRCRAFT PRODUCED_AT_HELIPAD", 14, "G_Air", "", "") +
           object_ini("G_Jet", "PRELOAD SELECTABLE VEHICLE AIRCRAFT", 11, "G_Air", "", arms("12").c_str()) +
           object_ini("G_Fixed", "PRELOAD SELECTABLE STRUCTURE IMMOBILE CAN_ATTACK", 20, "G_Foot", "", "") +
           object_ini("G_House", "PRELOAD SELECTABLE STRUCTURE IMMOBILE", 30, nullptr, nullptr, "") +
           object_ini("G_Rock", "PRELOAD SHRUBBERY", 3, nullptr, nullptr, "") +
           object_ini("G_Sneak", "PRELOAD SELECTABLE INFANTRY", 5, "G_Foot", acquire, sneak.c_str()) +
           object_ini("G_Bold", "PRELOAD SELECTABLE VEHICLE", 8, "G_Wheel", acquire_stealthed, sneak.c_str()) +
           object_ini("G_Granted", "PRELOAD SELECTABLE INFANTRY", 5, "G_Foot", acquire, granted.c_str()) +
           object_ini("G_Nexus", "PRELOAD SELECTABLE INFANTRY MOB_NEXUS", 5, "G_Foot", "", "") +
           object_ini("G_Tower", "PRELOAD ALWAYS_SELECTABLE", 10, nullptr, nullptr, "");
}

static std::vector<ObjectID> g_ids;
static std::vector<UnsignedInt> g_groups;

static void hook_orders(AIUpdateInterface* ai) {
    static void* table[3];
    AICommandInterface* iface = ai;
    void*** slot = reinterpret_cast<void***>(iface);
    void** original = *slot;
    if (original == table + 2) return;
    table[0] = original[-2];
    table[1] = original[-1];
    table[2] = reinterpret_cast<void*>(&record_command);
    *slot = table + 2;
}

static Object* unit(int i) {
    if (i < 0 || i >= static_cast<int>(g_ids.size())) return nullptr;
    return TheGameLogic->findObjectByID(g_ids[i]);
}

static AIGroup* group_at(int g) {
    return g >= 0 && g < static_cast<int>(g_groups.size()) ? TheAI->findGroup(g_groups[g]) : nullptr;
}

// C casts: `Snapshot` is not the first base of either class.
static std::string observe_groups() {
    std::string out;
    for (UnsignedInt id : g_groups) {
        AIGroup* g = TheAI->findGroup(id);
        if (!g) continue;
        const VecObjectID& ids = g->getAllIDs();
        std::string list;
        for (ObjectID m : ids) list += (list.empty() ? "" : ".") + num(static_cast<int>(m));
        if (!out.empty()) out += "|";
        uint32_t unused = 0;
        out += num(id) + ":" + num(g->getCount()) + ":" + list + ":" +
               hex32(oracle_wp51::World::snapshot_crc((Snapshot*)g)) + ":" +
               oracle_wp51::World::snapshot_save_hex((Snapshot*)g, &unused);
    }
    return out;
}

static std::string observe_unit(int i) {
    Object* o = unit(i);
    if (!o) return "gone";
    AIGroup* g = o->getGroup();
    std::string s = num(o->getFormationID()) + "," + fx((o->*access_member(OffsetTag())).x) + "," +
                    fx((o->*access_member(OffsetTag())).y) + "," + num(g ? g->getID() : 0);
    const WeaponSet& weapons = o->*access_member(WeaponSetTag());
    s += "," + num(weapons.getCurWeaponSlot()) + num(weapons.*access_member(LockTag())) +
         num(o->testWeaponSetFlag(WEAPONSET_MINE_CLEARING_DETAIL) ? 1 : 0);
    if (AIUpdateInterface* ai = o->getAI())
        s += "," + num(ai->*access_member(MoodTimeTag())) + "," + num(ai->*access_member(MoodRandTag()));
    return s;
}

static Coord3D random_goal() {
    Coord3D c;
    c.x = static_cast<float>(static_cast<int>(pick(2300)) - 150) + static_cast<float>(pick(8)) * 0.125f;
    c.y = static_cast<float>(static_cast<int>(pick(2300)) - 150);
    c.z = static_cast<float>(pick(5));
    return c;
}

static Coord3D g_hub;

static Coord3D random_spot(int spread) {
    Coord3D c;
    c.x = g_hub.x + static_cast<float>(static_cast<int>(pick(2 * spread + 1)) - spread) + static_cast<float>(pick(8)) * 0.125f;
    c.y = g_hub.y + static_cast<float>(static_cast<int>(pick(2 * spread + 1)) - spread);
    c.z = static_cast<float>(pick(4));
    return c;
}

// One command through the unmodified `GameLogic::logicMessageDispatcher`.
struct Command {
    GameMessage* msg;
    std::string args;
    Command(int player, int type) : msg(newInstance(GameMessage)(static_cast<GameMessage::Type>(type))) {
        msg->friend_setPlayerIndex(player);
    }
    void integer(int v) {
        msg->appendIntegerArgument(v);
        args += " i:" + num(v);
    }
    void boolean(bool v) {
        msg->appendBooleanArgument(v);
        args += " b:" + num(v ? 1 : 0);
    }
    void object(int id) {
        msg->appendObjectIDArgument(static_cast<ObjectID>(id));
        args += " o:" + num(id);
    }
    void location(const Coord3D& c) {
        msg->appendLocationArgument(c);
        args += " l:" + c3(c);
    }
    std::string send(int player, int type) {
        TheGameLogic->logicMessageDispatcher(msg, nullptr);
        deleteInstance(msg);
        return "cmd " + num(player) + " " + num(type) + " " + num(TheAI->*access_member(NextGroupTag())) + args;
    }
};

static int some_id() {
    return pick(10) == 0 ? 9999 : static_cast<int>(g_ids[pick(static_cast<uint32_t>(g_ids.size()))]);
}

static std::string command(int spread) {
    const int player = pick(6) == 0 ? 2 : 1;
    static const int kOrders[] = {1068, 1068, 1068, 1068, 1069, 1070, 1071, 1072, 1073, 1074, 1075, 1094, 1059,
                                  1060, 1061, 1062, 1063, 1064, 1065, 1066, 1067, 1053, 1054, 1055, 1056, 1057,
                                  1076, 1077, 1079, 1037, 1038, 1039, 1087, 1096};
    const uint32_t r = pick(100);
    int type;
    if (r < 22)
        type = pick(2) ? 1001 : 1002;
    else if (r < 26)
        type = 1003;
    else if (r < 33)
        type = 1004;
    else if (r < 38)
        type = 1006 + static_cast<int>(pick(2));
    else if (r < 42)
        type = 1016 + static_cast<int>(pick(2));
    else if (r < 46)
        type = 1026 + static_cast<int>(pick(2));
    else
        type = kOrders[pick(sizeof(kOrders) / sizeof(kOrders[0]))];
    Command c(player, type);
    if (type == 1001 || type == 1002) {
        c.boolean(pick(2) != 0);
        const uint32_t n = 1 + pick(7);
        for (uint32_t i = 0; i < n; ++i) c.object(some_id());
    } else if (type == 1004 || (type >= 1006 && type < 1016)) {
        const uint32_t n = 1 + pick(4);
        for (uint32_t i = 0; i < n; ++i) c.object(some_id());
    } else if (type == 1068 || type == 1069 || type == 1070 || type == 1071 || type == 1087) {
        c.location(pick(4) == 0 ? random_spot(spread) : random_goal());
    } else if (type == 1061 || type == 1056) {
        c.location(random_goal());
    } else if (type == 1072) {
        c.location(random_goal());
        c.integer(static_cast<int>(pick(3)));
    } else if (type == 1073) {
        c.object(some_id());
        c.integer(static_cast<int>(pick(3)));
    } else if (type == 1066) {
        c.object(0);
        c.object(some_id());
    } else if (type == 1059 || type == 1060 || (type >= 1062 && type <= 1065) || type == 1067 || type == 1053 ||
               type == 1057) {
        c.object(some_id());
    } else if (type == 1079) {
        c.integer(static_cast<int>(pick(3)));
    } else if (type == 1037) {
        c.integer(static_cast<int>(pick(3)));
        c.integer(2);
    } else if (type == 1038) {
        c.integer(static_cast<int>(pick(3)));
        c.location(random_goal());
        c.integer(2);
    } else if (type == 1039) {
        c.integer(static_cast<int>(pick(3)));
        c.object(some_id());
        c.integer(2);
    }
    return c.send(player, type);
}

// One operation: its text, or an empty string when it does not apply.
static std::string operate(int spread) {
    const int u = static_cast<int>(pick(static_cast<uint32_t>(g_ids.size())));
    const int other = static_cast<int>(pick(static_cast<uint32_t>(g_ids.size())));
    int g = static_cast<int>(pick(static_cast<uint32_t>(g_groups.size())));
    for (int tries = 0; tries < 6 && !group_at(g); ++tries) g = static_cast<int>(pick(static_cast<uint32_t>(g_groups.size())));
    AIGroup* group = group_at(g);
    Object* o = unit(u);
    Object* target = unit(other);
    const int source = static_cast<int>(pick(4) == 0 ? pick(3) : CMD_FROM_PLAYER);
    const CommandSourceType src = static_cast<CommandSourceType>(source);
    std::ostringstream op;
    uint32_t r = pick(100);
    if (pick(3) == 0) {
        g_recording = true;
        const std::string text = command(spread);
        g_recording = false;
        return text;
    }
    if (r < 4) {
        if (g_groups.size() >= 12) return "";
        op << "new";
        g_recording = true;
        g_groups.push_back(TheAI->createGroup()->getID());
    } else if (r < 14) {
        if (!group || !o) return "";
        op << "add " << g << " " << u;
        group->add(o);
    } else if (r < 18) {
        if (!group || !o) return "";
        op << "remove " << g << " " << u << " " << static_cast<int>(group->remove(o));
    } else if (r < 19) {
        if (!group) return "";
        op << "drop " << g;
        TheAI->destroyGroup(group);
    } else if (r < 40) {
        if (!group) return "";
        const Coord3D p = pick(4) == 0 ? random_spot(spread) : random_goal();
        const int waypoint = pick(8) == 0 ? 1 : 0;
        op << "move " << g << " " << c3(p) << " " << waypoint << " " << source;
        g_recording = true;
        group->groupMoveToPosition(&p, waypoint, src);
    } else if (r < 43) {
        if (!group) return "";
        const Coord3D p = random_spot(spread);
        const int waypoint = pick(4) == 0 ? 1 : 0;
        op << "tighten " << g << " " << c3(p) << " " << waypoint << " " << source;
        g_recording = true;
        group->groupTightenToPosition(&p, waypoint, src);
    } else if (r < 46) {
        if (!group) return "";
        op << "scatter " << g << " " << source;
        g_recording = true;
        group->groupScatter(src);
    } else if (r < 50) {
        if (!group) return "";
        op << "idle " << g << " " << source;
        g_recording = true;
        group->groupIdle(src);
    } else if (r < 53) {
        if (!group || !target) return "";
        const int forced = static_cast<int>(pick(2));
        const int shots = static_cast<int>(pick(3));
        op << "attack " << g << " " << forced << " " << other << " " << shots << " " << source;
        g_recording = true;
        if (forced)
            group->groupForceAttackObject(target, shots, src);
        else
            group->groupAttackObject(target, shots, src);
    } else if (r < 55) {
        if (!group) return "";
        const int has = static_cast<int>(pick(2));
        const Coord3D p = random_goal();
        const int shots = static_cast<int>(pick(3));
        op << "apos " << g << " " << has << " " << c3(p) << " " << shots << " " << source;
        g_recording = true;
        group->groupAttackPosition(has ? &p : nullptr, shots, src);
    } else if (r < 57) {
        if (!group) return "";
        const Coord3D p = random_goal();
        op << "amove " << g << " " << c3(p) << " " << source;
        g_recording = true;
        group->groupAttackMoveToPosition(&p, 7, src);
    } else if (r < 62) {
        if (!group || !target) return "";
        const int kind = static_cast<int>(pick(13));
        const Coord3D p = random_goal();
        const int mode = static_cast<int>(pick(3));
        op << "order " << g << " " << kind << " " << other << " " << c3(p) << " " << mode << " " << source;
        g_recording = true;
        switch (kind) {
            case 0: group->groupRepair(target, src); break;
            case 1: group->groupResumeConstruction(target, src); break;
            case 2: group->groupGetHealed(target, src); break;
            case 3: group->groupGetRepaired(target, src); break;
            case 4: group->groupEnter(target, src); break;
            case 5: group->groupDock(target, src); break;
            case 6: group->groupExit(target, src); break;
            case 7: group->groupHunt(src); break;
            case 8: group->groupExecuteRailedTransport(src); break;
            case 9: group->groupGuardPosition(&p, static_cast<GuardMode>(mode), src); break;
            case 10: group->groupGuardObject(target, static_cast<GuardMode>(mode), src); break;
            case 11: group->groupHackInternet(src); break;
            default: group->groupCombatDrop(mode ? target : nullptr, p, src); break;
        }
    } else if (r < 65) {
        if (!group) return "";
        const int kind = static_cast<int>(pick(3));
        const Coord3D p = random_goal();
        op << "evac " << g << " " << kind << " " << c3(p) << " " << source;
        g_recording = true;
        if (kind == 0)
            group->groupEvacuate(src);
        else if (kind == 1)
            group->groupMoveToAndEvacuate(&p, src);
        else
            group->groupMoveToAndEvacuateAndExit(&p, src);
    } else if (r < 70) {
        if (!group) return "";
        op << "form " << g;
        group->groupCreateFormation(CMD_FROM_PLAYER);
    } else if (r < 72) {
        if (!group) return "";
        const int player = 1 + static_cast<int>(pick(2));
        const Player* p = ThePlayerList->getNthPlayer(player);
        const int any = group->containsAnyObjectsNotOwnedByPlayer(p) ? 1 : 0;
        const int destroyed = group->removeAnyObjectsNotOwnedByPlayer(p) ? 1 : 0;
        op << "owner " << g << " " << player << " " << any << destroyed;
    } else if (r < 78) {
        if (!group) return "";
        Coord3D center;
        Coord3D other_center;
        Coord2D min;
        Coord2D max;
        const int has = group->getCenter(&center) ? 1 : 0;
        const int formation = group->getMinMaxAndCenter(&min, &max, &other_center) ? 1 : 0;
        const float speed = group->getSpeed();
        const int idle = group->isIdle() ? 1 : 0;
        const int busy = group->isBusy() ? 1 : 0;
        const int dead = group->isGroupAiDead() ? 1 : 0;
        op << "query " << g << " " << has << "," << c3(center) << "," << formation << "," << fx(min.x) << ","
           << fx(min.y) << "," << fx(max.x) << "," << fx(max.y) << "," << c3(other_center) << "," << fx(speed) << ","
           << idle << busy << dead;
    } else if (r < 82) {
        if (!group) return "";
        // A squad of ids, some stale and some repeated, fills the group and takes it back.
        Squad* squad = newInstance(Squad);
        const uint32_t n = pick(6);
        std::string ids;
        for (uint32_t i = 0; i < n; ++i) {
            const int pickid = pick(8) == 0 ? 9999 : static_cast<int>(g_ids[pick(static_cast<uint32_t>(g_ids.size()))]);
            squad->addObjectID(static_cast<ObjectID>(pickid));
            ids += (ids.empty() ? "" : ".") + num(pickid);
        }
        uint32_t unused = 0;
        squad->aiGroupFromSquad(group);
        const std::string kept = oracle_wp51::World::snapshot_save_hex((Snapshot*)squad, &unused);
        AIGroup* back = group_at(g);
        squad->squadFromAIGroup(back, pick(2) != 0);
        const std::string taken = oracle_wp51::World::snapshot_save_hex((Snapshot*)squad, &unused);
        op << "squad " << g << " " << (ids.empty() ? "-" : ids) << " " << kept << " " << taken;
        deleteInstance(squad);
    } else if (r < 86) {
        if (!o) return "";
        const Coord3D p = pick(3) == 0 ? random_goal() : random_spot(spread);
        op << "pos " << u << " " << c3(p);
        o->setPosition(&p);
    } else if (r < 90) {
        if (!o) return "";
        static const ObjectStatusTypes kStatus[] = {OBJECT_STATUS_CAN_STEALTH, OBJECT_STATUS_STEALTHED,
                                                    OBJECT_STATUS_DETECTED, OBJECT_STATUS_AIRBORNE_TARGET,
                                                    OBJECT_STATUS_NO_ATTACK, OBJECT_STATUS_UNSELECTABLE};
        const int which = static_cast<int>(pick(6));
        const int on = which == 0 ? (pick(4) != 0) : (pick(3) == 0);
        op << "status " << u << " " << static_cast<int>(kStatus[which]) << " " << on;
        o->setStatus(MAKE_OBJECT_STATUS_MASK(kStatus[which]), on);
    } else if (r < 93) {
        if (!o) return "";
        const int on = pick(3) == 0 ? 1 : 0;
        op << "held " << u << " " << on;
        (o->*access_member(DisabledTag())).set(DISABLED_HELD, on);
    } else if (r < 95) {
        if (!o) return "";
        const int state = static_cast<int>(pick(3));
        op << "damage " << u << " " << state;
        o->getBodyModule()->setDamageState(static_cast<BodyDamageType>(state));
    } else if (r < 96) {
        if (!o || !o->getAI()) return "";
        op << "busy " << u;
        (o->getAI()->*access_member(MachineTag()))->setState(AI_BUSY);
    } else if (r < 97) {
        if (!o) return "";
        op << "dead " << u;
        o->setEffectivelyDead(TRUE);
    } else if (r < 98) {
        if (!o) return "";
        op << "destroy " << u;
        TheGameLogic->destroyObject(o);
        TheGameLogic->processDestroyList();
    } else {
        const UnsignedInt n = 1 + pick(200);
        op << "frames " << n;
        TheGameLogic->*access_member(Wp51GLFrameTag()) = TheGameLogic->getFrame() + n;
    }
    g_recording = false;
    return op.str();
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_pkt423_ai_group <out.json>\n");
        return 2;
    }
    oracle_wp51::World w;
    TheTerrainLogic = NEW GroupTerrain;
    TheTerrainLogic->init();
    TheWritableGlobalData->m_gravity = -1.0f;
    TheActionManager = NEW ActionManager;
    w.load_ini(weapons_ini());
    w.load_ini(locomotors_ini());
    w.load_ini(objects_ini());
    w.init_with_neutral_player();
    // `setGameMode` also tells the mouse, which a headless world does not have.
    TheGameLogic->*access_member(GameModeTag()) = GAME_SKIRMISH;
    // The dispatcher only asks the recorder whether it plays back: all zero says no.
    alignas(16) static unsigned char recorder[sizeof(RecorderClass)];
    TheRecorder = reinterpret_cast<RecorderClass*>(recorder);

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

    const int kScenarios = 240;
    const int kSteps = 48;
    std::fprintf(out, "{\"vector_count\":%d,\"weapons\":\"%s\",\"locomotors\":\"%s\",\"objects\":\"%s\",\"vectors\":[\n",
                 kScenarios, escape(weapons_ini()).c_str(), escape(locomotors_ini()).c_str(),
                 escape(objects_ini()).c_str());

    for (int scenario = 0; scenario < kScenarios; ++scenario) {
        const UnsignedInt frame = 1 + pick(3000);
        const UnsignedInt seed = 1 + pick(1000000000);
        TheGameLogic->*access_member(Wp51GLFrameTag()) = frame;
        InitRandom(seed);
        const int human_a = static_cast<int>(pick(2));
        const int human_b = static_cast<int>(pick(2));
        ThePlayerList->getNthPlayer(1)->setPlayerType(human_a ? PLAYER_HUMAN : PLAYER_COMPUTER, FALSE);
        ThePlayerList->getNthPlayer(2)->setPlayerType(human_b ? PLAYER_HUMAN : PLAYER_COMPUTER, FALSE);

        TAiData* data = const_cast<TAiData*>(TheAI->getAiData());
        data->m_minInfantryForGroup = 1 + static_cast<int>(pick(5));
        data->m_minVehiclesForGroup = 1 + static_cast<int>(pick(5));
        data->m_minDistanceForGroup = static_cast<float>(pick(3) * 150);
        data->m_distanceRequiresGroup = static_cast<float>(pick(4) * 200);
        TheWritableGlobalData->m_groupMoveClickToGatherFactor = static_cast<float>(pick(3)) * 0.75f;
        TheWritableGlobalData->m_movementPenaltyDamageState = pick(2) ? BODY_REALLYDAMAGED : BODY_RUBBLE;

        // Empty the players' selections and the hotkey squads the commands use.
        for (int player = 1; player <= 2; ++player) {
            Command(player, 1003).send(player, 1003);
            Command(player, 1006).send(player, 1006);
            Command(player, 1007).send(player, 1007);
        }
        const UnsignedInt formation_base = static_cast<UnsignedInt>(TheAI->getNextFormationID());

        const ObjectID first_id = static_cast<ObjectID>(1000);
        TheGameLogic->setObjectIDCounter(first_id);
        g_ids.clear();
        g_groups.clear();

        // A hub the units stand around: tight crowds march, wide ones do not.
        g_hub.x = static_cast<float>(200 + pick(1600));
        g_hub.y = static_cast<float>(200 + pick(1600));
        g_hub.z = 0.0f;
        const int spread = pick(3) == 0 ? 600 : 90;
        // Most scenarios are of one kind of unit so that groups reach march size.
        const int theme = static_cast<int>(pick(8));
        // Themes 6 and 7 are crowds: enough infantry for five columns, and enough
        // helicopters for a second ring.
        const int units = theme >= 6 ? 17 + static_cast<int>(pick(8)) : 3 + static_cast<int>(pick(14));
        std::ostringstream spawns;
        for (int i = 0; i < units; ++i) {
            int kind = static_cast<int>(pick(kKinds));
            if (theme == 0 && pick(8) != 0) kind = static_cast<int>(pick(3));
            if (theme == 1 && pick(8) != 0) kind = 3 + static_cast<int>(pick(3));
            if (theme == 2 && pick(4) != 0) kind = static_cast<int>(pick(6));
            if (theme == 3 && pick(3) != 0) kind = 11 + static_cast<int>(pick(3));
            if (theme == 4 && pick(3) != 0) kind = 6 + static_cast<int>(pick(4));
            if (theme == 6 && pick(12) != 0) kind = static_cast<int>(pick(3));
            if (theme == 7 && pick(4) != 0) kind = 6;
            const int team = static_cast<int>(pick(6) == 0 ? 1 : 0);
            const Coord3D p = random_spot(spread);
            const ThingTemplate* tt = TheThingFactory->findTemplate(kTemplates[kind]);
            if (!tt) return 3;
            Object* o = TheThingFactory->newObject(tt, team ? w.team_b : w.team_a);
            o->setPosition(&p);
            if (o->getAI()) hook_orders(o->getAI());
            g_ids.push_back(o->getID());
            spawns << (i ? ";" : "") << kind << "," << team << "," << c3(p);
        }

        std::string steps;
        std::vector<std::string> last(g_ids.size());
        auto record = [&](const std::string& op) {
            std::ostringstream s;
            s << op << "~" << g_tape << "~" << hex32(GetGameLogicRandomSeedCRC()) << "~" << observe_groups();
            for (size_t i = 0; i < g_ids.size(); ++i) {
                const std::string now = observe_unit(static_cast<int>(i));
                s << "~" << (now == last[i] ? std::string("=") : now);
                last[i] = now;
            }
            steps += (steps.empty() ? "" : "#") + s.str();
        };

        // Two or three groups to start with, most units in the first.
        UnsignedInt first_group = 0;
        const int groups = 2 + static_cast<int>(pick(2));
        for (int g = 0; g < groups; ++g) {
            g_groups.push_back(TheAI->createGroup()->getID());
            if (g == 0) first_group = g_groups[0];
        }
        std::ostringstream init;
        init << "init";
        for (int i = 0; i < units; ++i) {
            const int g = pick(5) == 0 ? static_cast<int>(pick(groups)) : 0;
            if (AIGroup* group = group_at(g)) group->add(unit(i));
            init << " " << g;
        }
        g_tape.clear();
        record(init.str());
        for (int step = 0; step < kSteps;) {
            g_tape.clear();
            const std::string op = operate(spread);
            g_recording = false;
            if (op.empty()) {
                // Keep a group to work on.
                bool any = false;
                for (size_t g = 0; g < g_groups.size(); ++g) any = any || group_at(static_cast<int>(g));
                if (!any) {
                    g_groups.push_back(TheAI->createGroup()->getID());
                    record("new");
                    ++step;
                }
                continue;
            }
            ++step;
            record(op);
        }
        std::fprintf(out,
                     "%s{\"i\":%d,\"frame\":%u,\"seed\":%u,\"humans\":\"%d%d\",\"data\":\"%d,%d,%s,%s,%s,%d\","
                     "\"gid\":%u,\"fid\":%u,\"groups\":%d,\"spawns\":\"%s\",\"steps\":\"%s\"}",
                     scenario ? ",\n" : "", scenario, static_cast<unsigned>(frame), static_cast<unsigned>(seed),
                     human_a, human_b, data->m_minInfantryForGroup, data->m_minVehiclesForGroup,
                     fx(data->m_minDistanceForGroup).c_str(), fx(data->m_distanceRequiresGroup).c_str(),
                     fx(TheGlobalData->m_groupMoveClickToGatherFactor).c_str(),
                     static_cast<int>(TheGlobalData->m_movementPenaltyDamageState), static_cast<unsigned>(first_group),
                     static_cast<unsigned>(formation_base), groups, spawns.str().c_str(), steps.c_str());
        std::fflush(out);

        for (UnsignedInt id : g_groups)
            if (AIGroup* g = TheAI->findGroup(id)) TheAI->destroyGroup(g);
        for (ObjectID id : g_ids)
            if (Object* o = TheGameLogic->findObjectByID(id)) TheGameLogic->destroyObject(o);
        w.flush_destroy_list();
    }
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    return std::rename(tmp.c_str(), argv[1]) == 0 ? 0 : 1;
}
