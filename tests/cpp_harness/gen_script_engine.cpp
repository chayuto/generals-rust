// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for ScriptEngine (PKT-433) with the ScriptConditions (PKT-432)
// and ScriptActions (PKT-431) that are ported: the unmodified reference engine
// runs random script lists in a small world.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_script_engine.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/SCRIPT_ENGINE.json
//
// Each scenario line holds the lists; each frame line holds the hook calls made
// between frames and what the world answers before the update; each result line
// holds the engine's save bytes, the script state and the effects on the world.
// Strings are written with '~' for a space and '-' for the empty string.
//
// Not covered: AI players (difficulty comes from the engine), non-zero audio
// lengths, waypoints (map reveals stop at the missing waypoint), the end of the
// game timer running out, loading, the letter box actions (they need a display),
// the kill and delete actions, PLAYER_KILL and the end screens (they need the
// shipped window layouts).

#include "oracle_wp51_world.h"

#include "Common/Energy.h"
#include "Common/Money.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/Team.h"
#include "Common/WellKnownKeys.h"
#include "GameClient/GameWindowManager.h"
#include "GameClient/CampaignManager.h"
#include "GameLogic/AI.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Object.h"
#include "GameLogic/ScriptActions.h"
#include "GameLogic/ScriptConditions.h"
#include "GameLogic/ScriptEngine.h"
#include "GameLogic/Scripts.h"
#include "GameLogic/SidesList.h"

#include <cstdio>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

using oracle_wp51::World;
static const Int kAtEnd = 0x00ffffff;

struct SePlayerNameTag { using type = NameKeyType Player::*; };
template struct MemberAccess<SePlayerNameTag, &Player::m_playerNameKey>;
inline NameKeyType Player::* access_member(SePlayerNameTag);
struct SePlayerSideTag { using type = AsciiString Player::*; };
template struct MemberAccess<SePlayerSideTag, &Player::m_side>;
inline AsciiString Player::* access_member(SePlayerSideTag);
struct SePlayerStartTag { using type = Int Player::*; };
template struct MemberAccess<SePlayerStartTag, &Player::m_mpStartIndex>;
inline Int Player::* access_member(SePlayerStartTag);
struct SePlayerDeadTag { using type = Bool Player::*; };
template struct MemberAccess<SePlayerDeadTag, &Player::m_isPlayerDead>;
inline Bool Player::* access_member(SePlayerDeadTag);
struct SePlayerSciencesTag { using type = ScienceVec Player::*; };
template struct MemberAccess<SePlayerSciencesTag, &Player::m_sciences>;
inline ScienceVec Player::* access_member(SePlayerSciencesTag);
struct SePlayerAttackedTag { using type = Bool (Player::*)[MAX_PLAYER_COUNT]; };
template struct MemberAccess<SePlayerAttackedTag, &Player::m_attackedBy>;
inline Bool (Player::*access_member(SePlayerAttackedTag))[MAX_PLAYER_COUNT];
struct SeCampaignTag { using type = Campaign* CampaignManager::*; };
template struct MemberAccess<SeCampaignTag, &CampaignManager::m_currentCampaign>;
inline Campaign* CampaignManager::* access_member(SeCampaignTag);

static uint32_t g_state = 0x5c4170e1u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}
static bool chance(uint32_t percent) { return pick(100) < percent; }
template <size_t N> static const char* any(const char* const (&pool)[N]) { return pool[pick(N)]; }

static std::string tok(const AsciiString& s) {
    if (s.isEmpty()) return "-";
    std::string out = s.str();
    for (char& c : out)
        if (c == ' ') c = '~';
    return out;
}
static std::string tok(const char* s) { return tok(AsciiString(s)); }

static uint32_t bits(Real r) {
    uint32_t u;
    std::memcpy(&u, &r, 4);
    return u;
}

static const char* const kTemplates[] = {"ScrTank", "ScrBarracks", "ScrDepot", "ScrCrate", "ScrVictim"};
static const int kTemplateCount = 5;
static const char* const kPlayerNames[] = {"PlyrNeutral", "PlyrA", "PlyrB"};
static const char* const kProtos[] = {"TeamA", "TeamB", "Multi", "Lone", "Idle", "Doomed"};
static const int kProtoCount = 6;

static const char* const kSidePool[] = {"PlyrA", "PlyrB", "PlyrNeutral", "<This Player>", "<This Player's Enemy>",
                                        "<Local Player>", "Nobody", "PlyrA", "PlyrB", "ThePlayer"};
static const char* const kTeamPool[] = {"TeamA", "TeamB", "Multi", "Lone", "Idle", "<This Team>", "Nope",
                                        "teamThePlayer", "Multi", "<This Team>"};
static const char* const kUnitPool[] = {"U0", "U1", "U2", "U3", "<This Object>", "Ghost", "U0"};
static const char* const kCounterPool[] = {"C0", "C1", "C2", "T0"};
static const char* const kTimerPool[] = {"T0", "T1", "C0"};
static const char* const kFlagPool[] = {"F0", "F1", "F2", "F00", "USA Team is Building1"};
static const char* const kObjectTypePool[] = {"ScrTank", "ScrBarracks", "ScrDepot", "ScrCrate", "L0", "L1",
                                              "NoSuchThing"};
static const char* const kListPool[] = {"L0", "L1", "ScrDepot"};
static const char* const kStatePool[] = {"idle", "go", ""};
static const char* const kPowerPool[] = {"SP0", "SP1"};
static const char* const kUpgradePool[] = {"UP0", "UP1"};
static const char* const kSciencePool[] = {"SCIENCE_Alpha", "SCIENCE_Beta", "SCIENCE_None"};
static const char* const kMoviePool[] = {"V0", "V1"};
static const char* const kFactionPool[] = {"America", "China", "Civilian"};
static const char* const kRevealPool[] = {"R0", "R1"};

static std::string g_ini() {
    std::string s;
    const char* kinds[] = {"PRELOAD VEHICLE SELECTABLE", "PRELOAD STRUCTURE MP_COUNT_FOR_VICTORY",
                           "PRELOAD STRUCTURE", "PRELOAD CRATE", "PRELOAD VEHICLE"};
    for (int i = 0; i < kTemplateCount; ++i) {
        s += std::string("Object ") + kTemplates[i] + "\n  KindOf = " + kinds[i] +
             "\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 300.0\n    InitialHealth = 300.0\n  End\n";
        if (i == 0) s += "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n";
        s += "End\n";
    }
    s += "Science SCIENCE_Alpha\n  SciencePurchasePointCost = 1\n  IsGrantable = Yes\nEnd\n"
         "Science SCIENCE_Beta\n  SciencePurchasePointCost = 1\n  IsGrantable = No\nEnd\n";
    return s;
}

// ---------------------------------------------------------------------------
// Random script lists. Subroutines call only later subroutines, so no cycle.

// Commands to an AI draw logic random numbers of their own. Odd scenarios give
// none, so the engine is the only user of the generator and its seed is compared.
static bool g_pure_random = false;

static bool commands_ai(ScriptAction::ScriptActionType t) {
    return t == ScriptAction::TEAM_EXECUTE_SEQUENTIAL_SCRIPT ||
           t == ScriptAction::TEAM_EXECUTE_SEQUENTIAL_SCRIPT_LOOPING ||
           t == ScriptAction::TEAM_GUARD_FOR_FRAMECOUNT || t == ScriptAction::TEAM_IDLE_FOR_FRAMECOUNT ||
           t == ScriptAction::UNIT_GUARD_FOR_FRAMECOUNT || t == ScriptAction::UNIT_IDLE_FOR_FRAMECOUNT;
}
static bool draws_random(ScriptAction::ScriptActionType t) {
    return t == ScriptAction::SET_RANDOM_TIMER || t == ScriptAction::SET_RANDOM_MSEC_TIMER;
}

static std::string g_scriptName(int i) { return "S" + std::to_string(i); }
static const int kScripts = 14;
static const int kFirstSub = 9;  // S9.. are subroutines, S12 and S13 sequential bodies

struct Gen {
    int self = 0;  // script number being filled
};

static void fill(Parameter* p, const Gen& g) {
    switch (p->getParameterType()) {
        case Parameter::SIDE: p->friend_setString(any(kSidePool)); break;
        case Parameter::TEAM: p->friend_setString(any(kTeamPool)); break;
        case Parameter::UNIT: p->friend_setString(any(kUnitPool)); break;
        case Parameter::COUNTER: p->friend_setString(any(kCounterPool)); break;
        case Parameter::FLAG: p->friend_setString(any(kFlagPool)); break;
        case Parameter::COMPARISON: p->friend_setInt(pick(7)); break;
        case Parameter::BOOLEAN: p->friend_setInt(pick(2)); break;
        case Parameter::INT: p->friend_setInt(static_cast<int>(pick(9)) - 2); break;
        case Parameter::REAL:
        case Parameter::PERCENT:
        case Parameter::ANGLE: p->friend_setReal(static_cast<Real>(pick(400)) / 97.0f - 0.5f); break;
        case Parameter::OBJECT_TYPE: p->friend_setString(any(kObjectTypePool)); break;
        case Parameter::OBJECT_TYPE_LIST: p->friend_setString(any(kListPool)); break;
        case Parameter::TEAM_STATE: p->friend_setString(chance(30) ? "" : any(kStatePool)); break;
        case Parameter::SPECIAL_POWER: p->friend_setString(any(kPowerPool)); break;
        case Parameter::UPGRADE: p->friend_setString(any(kUpgradePool)); break;
        case Parameter::SCIENCE: p->friend_setString(any(kSciencePool)); break;
        case Parameter::MOVIE:
        case Parameter::DIALOG:
        case Parameter::SOUND: p->friend_setString(any(kMoviePool)); break;
        case Parameter::FACTION_NAME: p->friend_setString(any(kFactionPool)); break;
        case Parameter::REVEALNAME: p->friend_setString(any(kRevealPool)); break;
        case Parameter::WAYPOINT: p->friend_setString("NoWaypoint"); break;
        case Parameter::RELATION: p->friend_setInt(pick(3)); break;
        case Parameter::SCRIPT:
        case Parameter::SCRIPT_SUBROUTINE: {
            if (chance(15)) p->friend_setString(chance(50) ? "G1" : "Missing");
            else p->friend_setString(g_scriptName(pick(kScripts)).c_str());
            break;
        }
        case Parameter::COORD3D: {
            Coord3D c;
            c.x = static_cast<Real>(pick(21)) - 10.0f;
            c.y = static_cast<Real>(pick(21)) - 10.0f;
            c.z = static_cast<Real>(pick(5));
            p->friend_setCoord3D(&c);
            break;
        }
        default: p->friend_setString("X"); break;
    }
}

#define C(x) Condition::x
static const Condition::ConditionType kConditions[] = {
    C(CONDITION_FALSE), C(CONDITION_TRUE), C(CONDITION_TRUE), C(COUNTER), C(COUNTER), C(FLAG), C(FLAG),
    C(TIMER_EXPIRED), C(TIMER_EXPIRED), C(PLAYER_ALL_DESTROYED), C(PLAYER_ALL_BUILDFACILITIES_DESTROYED),
    C(TEAM_DESTROYED), C(TEAM_HAS_UNITS), C(TEAM_STATE_IS), C(TEAM_STATE_IS_NOT), C(TEAM_CREATED),
    C(TEAM_OWNED_BY_PLAYER), C(NAMED_DESTROYED), C(NAMED_NOT_DESTROYED), C(NAMED_DYING),
    C(NAMED_TOTALLY_DEAD), C(NAMED_CREATED), C(NAMED_OWNED_BY_PLAYER), C(UNIT_HEALTH), C(PLAYER_HAS_CREDITS),
    C(PLAYER_HAS_POWER), C(PLAYER_HAS_NO_POWER), C(PLAYER_POWER_COMPARE_PERCENT),
    C(PLAYER_EXCESS_POWER_COMPARE_VALUE), C(PLAYER_HAS_N_OR_FEWER_BUILDINGS),
    C(PLAYER_HAS_N_OR_FEWER_FACTION_BUILDINGS), C(PLAYER_HAS_OBJECT_COMPARISON), C(BUILT_BY_PLAYER),
    C(PLAYER_LOST_OBJECT_TYPE), C(PLAYER_ACQUIRED_SCIENCE), C(PLAYER_TRIGGERED_SPECIAL_POWER),
    C(PLAYER_TRIGGERED_SPECIAL_POWER_FROM_NAMED), C(PLAYER_MIDWAY_SPECIAL_POWER),
    C(PLAYER_MIDWAY_SPECIAL_POWER_FROM_NAMED), C(PLAYER_COMPLETED_SPECIAL_POWER),
    C(PLAYER_COMPLETED_SPECIAL_POWER_FROM_NAMED), C(PLAYER_BUILT_UPGRADE), C(PLAYER_BUILT_UPGRADE_FROM_NAMED),
    C(HAS_FINISHED_VIDEO), C(HAS_FINISHED_SPEECH), C(HAS_FINISHED_AUDIO), C(SKIRMISH_PLAYER_FACTION),
    C(SKIRMISH_PLAYER_HAS_BEEN_ATTACKED_BY_PLAYER), C(START_POSITION_IS), C(MISSION_ATTEMPTS)};
#undef C

#define A(x) ScriptAction::x
static const ScriptAction::ScriptActionType kActions[] = {
    A(SET_COUNTER), A(SET_TREE_SWAY), A(INCREMENT_COUNTER), A(DECREMENT_COUNTER), A(SET_FLAG), A(SET_FLAG),
    A(STOP_TIMER), A(RESTART_TIMER), A(SET_TIMER), A(SET_MILLISECOND_TIMER), A(SET_RANDOM_TIMER),
    A(SET_RANDOM_MSEC_TIMER), A(ADD_TO_MSEC_TIMER), A(SUB_FROM_MSEC_TIMER), A(ENABLE_SCRIPT),
    A(DISABLE_SCRIPT), A(CALL_SUBROUTINE), A(CALL_SUBROUTINE), A(CAMERA_FADE_ADD), A(CAMERA_FADE_SUBTRACT),
    A(CAMERA_FADE_SATURATE), A(CAMERA_FADE_MULTIPLY), A(SET_ATTACK_PRIORITY_THING),
    A(SET_ATTACK_PRIORITY_KIND_OF), A(SET_DEFAULT_ATTACK_PRIORITY), A(NO_OP), A(OBJECTLIST_ADDOBJECTTYPE),
    A(OBJECTLIST_ADDOBJECTTYPE), A(OBJECTLIST_REMOVEOBJECTTYPE), A(TEAM_EXECUTE_SEQUENTIAL_SCRIPT),
    A(TEAM_EXECUTE_SEQUENTIAL_SCRIPT_LOOPING), A(UNIT_EXECUTE_SEQUENTIAL_SCRIPT),
    A(UNIT_EXECUTE_SEQUENTIAL_SCRIPT_LOOPING), A(TEAM_STOP_SEQUENTIAL_SCRIPT), A(UNIT_STOP_SEQUENTIAL_SCRIPT),
    A(TEAM_GUARD_FOR_FRAMECOUNT), A(TEAM_IDLE_FOR_FRAMECOUNT), A(UNIT_GUARD_FOR_FRAMECOUNT),
    A(UNIT_IDLE_FOR_FRAMECOUNT), A(PLAYER_SET_MONEY), A(PLAYER_GIVE_MONEY), A(PLAYER_GRANT_SCIENCE),
    A(PLAYER_ENABLE_UNIT_CONSTRUCTION), A(PLAYER_DISABLE_UNIT_CONSTRUCTION), A(PLAYER_ENABLE_BASE_CONSTRUCTION),
    A(PLAYER_DISABLE_BASE_CONSTRUCTION), A(PLAYER_RELATES_PLAYER), A(TEAM_SET_STATE),
    A(NAMED_SET_TOPPLE_DIRECTION), A(MAP_REVEAL_PERMANENTLY_AT_WAYPOINT),
    A(MAP_UNDO_REVEAL_PERMANENTLY_AT_WAYPOINT), A(OBJECT_ALLOW_BONUSES), A(CHOOSE_VICTIM_ALWAYS_USES_NORMAL),
    A(FREEZE_TIME), A(UNFREEZE_TIME)};
// Allowed inside a sequential body, besides the idle timers.
static const ScriptAction::ScriptActionType kSeqActions[] = {
    A(SET_COUNTER), A(INCREMENT_COUNTER), A(SET_FLAG), A(NO_OP), A(UNIT_IDLE_FOR_FRAMECOUNT),
    A(TEAM_IDLE_FOR_FRAMECOUNT), A(TEAM_GUARD_FOR_FRAMECOUNT), A(UNIT_GUARD_FOR_FRAMECOUNT)};
#undef A

static ScriptAction* make_action(ScriptAction::ScriptActionType type, const Gen& g) {
    ScriptAction* a = newInstance(ScriptAction)(type);
    for (int i = 0; i < a->getNumParameters(); ++i) fill(a->getParameter(i), g);
    switch (type) {
        case ScriptAction::CALL_SUBROUTINE: {
            // Only later subroutines or the subroutine group.
            int lo = g.self >= kFirstSub ? g.self + 1 : kFirstSub;
            if (lo >= 12) a->getParameter(0)->friend_setString("Missing");
            else if (g.self < 7 && chance(20)) a->getParameter(0)->friend_setString("G1");
            else a->getParameter(0)->friend_setString(g_scriptName(lo + pick(12 - lo)).c_str());
            break;
        }
        case ScriptAction::SET_TIMER:
        case ScriptAction::SET_RANDOM_TIMER:
        case ScriptAction::SET_MILLISECOND_TIMER:
        case ScriptAction::SET_RANDOM_MSEC_TIMER:
        case ScriptAction::STOP_TIMER:
        case ScriptAction::RESTART_TIMER:
            a->getParameter(0)->friend_setString(any(kTimerPool));
            break;
        case ScriptAction::ADD_TO_MSEC_TIMER:
        case ScriptAction::SUB_FROM_MSEC_TIMER:
            a->getParameter(1)->friend_setString(any(kTimerPool));
            break;
        case ScriptAction::SET_ATTACK_PRIORITY_THING: {
            // One template per set: the reference orders a set by address.
            const int set = pick(2);
            a->getParameter(0)->friend_setString(set ? "AP1" : "AP0");
            if (chance(20)) a->getParameter(1)->friend_setString("NoSuchThing");
            else if (!set && chance(30)) a->getParameter(1)->friend_setString("LP");
            else a->getParameter(1)->friend_setString(set ? "ScrBarracks" : "ScrTank");
            break;
        }
        case ScriptAction::SET_ATTACK_PRIORITY_KIND_OF:
            a->getParameter(0)->friend_setString("AP2");
            a->getParameter(1)->friend_setInt(KINDOF_CRATE);
            break;
        case ScriptAction::SET_DEFAULT_ATTACK_PRIORITY:
            a->getParameter(0)->friend_setString(chance(30) ? "AP3" : (chance(50) ? "AP0" : "AP2"));
            break;
        case ScriptAction::CAMERA_FADE_ADD:
        case ScriptAction::CAMERA_FADE_SUBTRACT:
        case ScriptAction::CAMERA_FADE_SATURATE:
        case ScriptAction::CAMERA_FADE_MULTIPLY:
            for (int i = 2; i < 5; ++i) a->getParameter(i)->friend_setInt(pick(4));
            break;
        case ScriptAction::TEAM_EXECUTE_SEQUENTIAL_SCRIPT:
        case ScriptAction::TEAM_EXECUTE_SEQUENTIAL_SCRIPT_LOOPING:
        case ScriptAction::UNIT_EXECUTE_SEQUENTIAL_SCRIPT:
        case ScriptAction::UNIT_EXECUTE_SEQUENTIAL_SCRIPT_LOOPING:
            a->getParameter(1)->friend_setString(chance(10) ? "Missing" : (chance(50) ? "S12" : "S13"));
            if (a->getNumParameters() > 2) a->getParameter(2)->friend_setInt(static_cast<int>(pick(4)));
            break;
        case ScriptAction::PLAYER_SET_MONEY:
        case ScriptAction::PLAYER_GIVE_MONEY:
            a->getParameter(1)->friend_setInt(static_cast<int>(pick(2001)) - 500);
            break;
        default:
            break;
    }
    return a;
}

static Condition* make_condition(const Gen& g) {
    const Condition::ConditionType type = kConditions[pick(sizeof(kConditions) / sizeof(kConditions[0]))];
    Condition* c = newInstance(Condition)(type);
    for (int i = 0; i < c->getNumParameters(); ++i) fill(c->getParameter(i), g);
    switch (type) {
        case Condition::TIMER_EXPIRED: c->getParameter(0)->friend_setString(any(kTimerPool)); break;
        case Condition::PLAYER_HAS_CREDITS: c->getParameter(0)->friend_setInt(static_cast<int>(pick(1500)) - 100); break;
        case Condition::UNIT_HEALTH: c->getParameter(2)->friend_setInt(pick(5) * 25); break;
        case Condition::PLAYER_POWER_COMPARE_PERCENT: c->getParameter(2)->friend_setInt(pick(5) * 50); break;
        case Condition::PLAYER_EXCESS_POWER_COMPARE_VALUE: c->getParameter(2)->friend_setInt((static_cast<int>(pick(7)) - 3) * 5); break;
        case Condition::FLAG: c->getParameter(1)->friend_setInt(pick(2)); break;
        default: break;
    }
    return c;
}

static Script* make_script(int number, const std::string& name) {
    Gen g;
    g.self = number;
    const bool sequential = number >= 12;
    Script* s = newInstance(Script)();
    s->setName(name.c_str());
    s->setActive(chance(85));
    s->setOneShot(chance(35));
    s->setSubroutine(number >= kFirstSub && number < 12 ? true : (sequential ? chance(80) : false));
    s->setEasy(chance(80));
    s->setNormal(chance(80));
    s->setHard(chance(80));
    s->setDelayEvalSeconds(chance(15) ? 1 : 0);
    const int ors = chance(8) ? 0 : 1 + pick(3);
    OrCondition* last_or = nullptr;
    for (int o = 0; o < ors; ++o) {
        OrCondition* oc = newInstance(OrCondition)();
        const int ands = chance(5) ? 0 : 1 + (chance(30) ? pick(3) : 0);
        Condition* last = nullptr;
        for (int a = 0; a < ands; ++a) {
            Condition* c = make_condition(g);
            if (last) last->setNextCondition(c);
            else oc->setFirstAndCondition(c);
            last = c;
        }
        if (last_or) last_or->setNextOrCondition(oc);
        else s->setOrCondition(oc);
        last_or = oc;
    }
    for (int list = 0; list < 2; ++list) {
        int n = list ? (chance(30) ? 1 + pick(2) : 0) : pick(5);
        if (sequential) n = list ? 0 : 1 + pick(4);
        ScriptAction* last = nullptr;
        for (int i = 0; i < n; ++i) {
            ScriptAction::ScriptActionType type;
            do {
                type = sequential ? kSeqActions[pick(sizeof(kSeqActions) / sizeof(kSeqActions[0]))]
                                  : kActions[pick(sizeof(kActions) / sizeof(kActions[0]))];
            } while (g_pure_random ? commands_ai(type) : draws_random(type));
            ScriptAction* a = make_action(type, g);
            if (sequential && a->getNumParameters() > 1 &&
                (type == ScriptAction::UNIT_IDLE_FOR_FRAMECOUNT || type == ScriptAction::TEAM_IDLE_FOR_FRAMECOUNT ||
                 type == ScriptAction::TEAM_GUARD_FOR_FRAMECOUNT || type == ScriptAction::UNIT_GUARD_FOR_FRAMECOUNT))
                a->getParameter(1)->friend_setInt(pick(4));
            if (last) last->setNextAction(a);
            else if (list) s->setFalseAction(a);
            else s->setAction(a);
            last = a;
        }
    }
    return s;
}

// ---------------------------------------------------------------------------
// Serialisation of the lists and of their mutable state.

static void put_parms(std::ostringstream& o, int type, int n, Parameter* (*get)(void*, int), void* owner) {
    o << ' ' << type << ' ' << n;
    for (int i = 0; i < n; ++i) {
        Parameter* p = get(owner, i);
        Coord3D c = {0, 0, 0};
        if (p->getParameterType() == Parameter::COORD3D) p->getCoord3D(&c);
        o << ' ' << static_cast<int>(p->getParameterType()) << ' ' << p->getInt() << ' ' << bits(p->getReal()) << ' '
          << tok(p->getString()) << ' ' << bits(c.x) << ' ' << bits(c.y) << ' ' << bits(c.z);
    }
}
static Parameter* cond_parm(void* c, int i) { return static_cast<Condition*>(c)->getParameter(i); }
static Parameter* act_parm(void* a, int i) { return static_cast<ScriptAction*>(a)->getParameter(i); }

static void put_actions(std::ostringstream& o, ScriptAction* head) {
    int n = 0;
    for (ScriptAction* a = head; a; a = a->getNext()) ++n;
    o << ' ' << n;
    for (ScriptAction* a = head; a; a = a->getNext())
        put_parms(o, a->getActionType(), a->getNumParameters(), act_parm, a);
}

static void put_script(std::ostringstream& o, Script* s) {
    o << ' ' << tok(s->getName()) << ' ' << int(s->isActive()) << ' ' << int(s->isOneShot()) << ' '
      << int(s->isSubroutine()) << ' ' << int(s->isEasy()) << ' ' << int(s->isNormal()) << ' ' << int(s->isHard())
      << ' ' << s->getDelayEvalSeconds();
    int ors = 0;
    for (OrCondition* oc = s->getOrCondition(); oc; oc = oc->getNextOrCondition()) ++ors;
    o << ' ' << ors;
    for (OrCondition* oc = s->getOrCondition(); oc; oc = oc->getNextOrCondition()) {
        int ands = 0;
        for (Condition* c = oc->getFirstAndCondition(); c; c = c->getNext()) ++ands;
        o << ' ' << ands;
        for (Condition* c = oc->getFirstAndCondition(); c; c = c->getNext())
            put_parms(o, c->getConditionType(), c->getNumParameters(), cond_parm, c);
    }
    put_actions(o, s->getAction());
    put_actions(o, s->getFalseAction());
}

static void put_state(std::ostringstream& o, Script* s) {
    o << ' ' << int(s->isActive()) << ' ' << s->getFrameToEvaluate() << ' ' << tok(s->getConditionTeamName());
    for (OrCondition* oc = s->getOrCondition(); oc; oc = oc->getNextOrCondition())
        for (Condition* c = oc->getFirstAndCondition(); c; c = c->getNext()) {
            o << ' ' << c->getCustomData() << ' ' << c->getCustomFrame();
            for (int i = 0; i < c->getNumParameters(); ++i) o << ' ' << c->getParameter(i)->getInt();
        }
    ScriptAction* heads[2] = {s->getAction(), s->getFalseAction()};
    for (ScriptAction* head : heads)
        for (ScriptAction* a = head; a; a = a->getNext())
            for (int i = 0; i < a->getNumParameters(); ++i) o << ' ' << a->getParameter(i)->getInt();
}

template <typename F> static void each_script(ScriptList* list, F f) {
    for (Script* s = list->getScript(); s; s = s->getNext()) f(s);
    for (ScriptGroup* g = list->getScriptGroup(); g; g = g->getNext())
        for (Script* s = g->getScript(); s; s = s->getNext()) f(s);
}

// ---------------------------------------------------------------------------
// The world.

struct Scene {
    Player* players[3];
    std::vector<Team*> teams;
    std::vector<Object*> objects;
};

static std::vector<Team*> instances(const char* proto) {
    std::vector<Team*> out;
    TeamPrototype* p = TheTeamFactory->findTeamPrototype(proto);
    if (!p) return out;
    for (DLINK_ITERATOR<Team> it = p->iterate_TeamInstanceList(); !it.done(); it.advance()) out.push_back(it.cur());
    return out;
}

static bool group_query(Team* t, bool dead) {
    AIGroupPtr g = TheAI->createGroup();
#if RETAIL_COMPATIBLE_AIGROUP
    t->getTeamAsAIGroup(g);
    return dead ? g->isGroupAiDead() : g->isIdle();
#else
    t->getTeamAsAIGroup(g.Peek());
    return dead ? g->isGroupAiDead() : g->isIdle();
#endif
}

static std::string snapshot(Scene& sc) {
    std::ostringstream o;
    o << TheGameLogic->getFrame() << ' ' << TheScriptEngine->getFrameObjectCountChanged();
    for (Player* p : sc.players) {
        Energy* e = p->getEnergy();
        KindOfMaskType mask;
        mask.set(KINDOF_MP_COUNT_FOR_VICTORY);
        mask.set(KINDOF_STRUCTURE);
        int attacked = 0;
        for (int i = 0; i < 3; ++i)
            if (p->getAttackedBy(i)) attacked |= 1 << i;
        o << ' ' << p->getMoney()->countMoney() << ' ' << e->getProduction() << ' ' << e->getConsumption() << ' '
          << int(e->hasSufficientPower()) << ' ' << bits(e->getEnergySupplyRatio()) << ' ' << int(p->hasAnyObjects())
          << ' ' << int(p->hasAnyBuildFacility()) << ' ' << p->countBuildings() << ' '
          << p->countObjects(mask, KINDOFMASK_NONE) << ' ' << int(p->isPlayerActive()) << ' ' << attacked;
        for (Player* q : sc.players) o << ' ' << static_cast<int>(p->getRelationship(q->getDefaultTeam()));
        for (int t = 0; t < kTemplateCount; ++t) {
            const ThingTemplate* tt = TheThingFactory->findTemplate(kTemplates[t]);
            Int all = 0, living = 0;
            p->countObjectsByThingTemplate(1, &tt, false, &all);
            p->countObjectsByThingTemplate(1, &tt, true, &living);
            o << ' ' << all << ' ' << living;
        }
    }
    o << ' ' << kProtoCount;
    for (const char* proto : kProtos) {
        TeamPrototype* p = TheTeamFactory->findTeamPrototype(proto);
        std::vector<Team*> list = instances(proto);
        o << ' ' << proto << ' ' << int(p->getIsSingleton()) << ' ' << p->getTemplateInfo()->m_maxInstances << ' '
          << list.size();
        for (Team* t : list) {
            const Player* owner = t->getControllingPlayer();
            o << ' ' << t->getID() << ' ' << int(t->isActive()) << ' ' << int(t->isCreated()) << ' '
              << tok(t->getState()) << ' ' << (owner ? owner->getPlayerIndex() : -1) << ' ' << int(t->hasAnyObjects())
              << ' ' << int(t->hasAnyUnits()) << ' ' << int(group_query(t, false)) << ' ' << int(group_query(t, true));
        }
    }
    o << ' ' << sc.objects.size();
    for (Object* obj : sc.objects) {
        const Player* owner = obj->getControllingPlayer();
        BodyModuleInterface* body = obj->getBodyModule();
        AIUpdateInterface* ai = obj->getAIUpdateInterface();
        o << ' ' << obj->getID() << ' ' << tok(obj->getName()) << ' ' << int(obj->isEffectivelyDead()) << ' '
          << (owner ? owner->getPlayerIndex() : -1) << ' ' << bits(body->getHealth()) << ' '
          << bits(body->getInitialHealth()) << ' ' << (ai ? int(ai->isIdle()) : -1) << ' '
          << (obj->getTeam() ? int(obj->getTeam()->getID()) : 0) << ' ' << int(obj->isKindOf(KINDOF_STRUCTURE));
    }
    return o.str();
}

static std::string effects(Scene& sc) {
    std::ostringstream o;
    for (Player* p : sc.players) {
        o << p->getMoney()->countMoney() << ' ' << int(p->getCanBuildUnits()) << int(p->getCanBuildBase());
        for (Player* q : sc.players) o << ' ' << static_cast<int>(p->getRelationship(q->getDefaultTeam()));
        o << ' ';
    }
    // The team update deletes emptied instances, so only some are listed.
    for (const char* proto : kProtos)
        if (TheTeamFactory->findTeamPrototype(proto)->getIsSingleton())
            for (Team* t : instances(proto)) o << tok(t->getState()) << ' ';
    for (Object* obj : sc.objects) {
        AIUpdateInterface* ai = obj->getAIUpdateInterface();
        o << (ai ? int(ai->isIdle()) : -1) << ' ';
    }
    return o.str();
}

static void set_money(Player* p, UnsignedInt amount) {
    p->getMoney()->withdraw(p->getMoney()->countMoney(), FALSE);
    p->getMoney()->deposit(amount, FALSE, FALSE);
}

// Hook calls between frames, mirrored by the test.
static std::string perturb(Scene& sc) {
    std::ostringstream o;
    int n = 0;
    std::ostringstream ops;
    const int count = chance(40) ? 0 : 1 + pick(4);
    for (int k = 0; k < count; ++k) {
        const int op = pick(20);
        const int player = pick(3);
        Object* obj = sc.objects[pick(4)];
        if (op == 0) {
            set_money(sc.players[player], pick(1500));
        } else if (op == 1) {
            sc.players[player]->getEnergy()->adjustPower(static_cast<int>(pick(31)) - 15, chance(60));
        } else if (op == 2) {
            obj->setEffectivelyDead(!obj->isEffectivelyDead());
        } else if (op == 3) {
            BodyModuleInterface* body = obj->getBodyModule();
            body->internalChangeHealth(static_cast<Real>(pick(301)) - body->getHealth() + 0.25f);
        } else if (op == 4) {
            // The object count changed, as when anything is built or dies.
            TheScriptEngine->notifyOfObjectCreationOrDestruction();
        } else if (op == 5) {
            const char* v = any(kMoviePool);
            TheScriptEngine->notifyOfCompletedVideo(v);
            ops << " video " << v;
            ++n;
        } else if (op >= 6 && op <= 9) {
            const char* name = op == 9 ? any(kUpgradePool) : any(kPowerPool);
            const ObjectID source = chance(70) ? sc.objects[pick(4)]->getID() : INVALID_ID;
            if (op == 6) TheScriptEngine->notifyOfTriggeredSpecialPower(player, name, source);
            if (op == 7) TheScriptEngine->notifyOfMidwaySpecialPower(player, name, source);
            if (op == 8) TheScriptEngine->notifyOfCompletedSpecialPower(player, name, source);
            if (op == 9) TheScriptEngine->notifyOfCompletedUpgrade(player, name, source);
            ops << " special " << op - 6 << ' ' << player << ' ' << name << ' ' << int(source);
            ++n;
        } else if (op == 10) {
            const ScienceType s = TheScienceStore->getScienceFromInternalName(chance(50) ? "SCIENCE_Alpha" : "SCIENCE_Beta");
            TheScriptEngine->notifyOfAcquiredScience(player, s);
            ops << " science " << player << ' ' << int(s);
            ++n;
        } else if (op == 11) {
            const char* hook = chance(70) ? any(kFlagPool) : "Unrelated";
            TheScriptEngine->signalUIInteract(hook);
            ops << " ui " << tok(hook);
            ++n;
        } else if (op == 12) {
            // Cache edits: add (perhaps renaming), remove, or hand a name over.
            const int what = pick(3);
            Object* other = sc.objects[pick(sc.objects.size())];
            if (what == 0) {
                other->setName(any(kUnitPool));
                if (other->getName() == AsciiString("<This Object>")) other->setName("U1");
                TheScriptEngine->addObjectToCache(other);
                ops << " add " << int(other->getID()) << ' ' << tok(other->getName());
            } else if (what == 1) {
                TheScriptEngine->removeObjectFromCache(other);
                ops << " remove " << int(other->getID());
            } else {
                const AsciiString name = chance(80) ? "U2" : "Fresh";
                ops << " world " << snapshot(sc);
                ++n;
                TheScriptEngine->transferObjectName(name, other);
                ops << " transfer " << int(other->getID()) << ' ' << tok(name);
            }
            ++n;
        } else if (op == 13) {
            const std::string name = chance(10) ? "<none>" : (chance(10) ? "G1" : g_scriptName(pick(kScripts)));
            std::vector<Team*> list = instances(any(kProtos));
            Team* team = chance(30) || list.empty() ? nullptr : list[pick(list.size())];
            const std::string at_call = snapshot(sc);
            TheScriptEngine->runScript(name.c_str(), team);
            ops << " world " << at_call;
            ++n;
            ops << " run " << name << ' ' << (team ? team->getID() : 0);
            ++n;
        } else if (op == 14) {
            const std::string name = chance(10) ? "G1" : g_scriptName(kFirstSub + pick(3));
            const std::string at_call = snapshot(sc);
            TheScriptEngine->runObjectScript(name.c_str(), obj);
            ops << " world " << at_call;
            ++n;
            ops << " runobj " << name << ' ' << int(obj->getID());
            ++n;
        } else if (op == 15) {
            TheScriptEngine->clearTeamFlags();
            ops << " clearteam";
            ++n;
        } else if (op == 16 && !g_pure_random) {
            Object* walker = sc.objects[pick(sc.objects.size())];
            AIUpdateInterface* ai = walker->getAIUpdateInterface();
            if (ai) {
                if (chance(60)) {
                    ai->aiIdle(CMD_FROM_SCRIPT);
                } else {
                    Coord3D pos = *walker->getPosition();
                    ai->aiGuardPosition(&pos, GUARDMODE_NORMAL, CMD_FROM_SCRIPT);
                }
            }
        } else if (op == 17) {
            std::vector<Team*> list = instances(chance(50) ? "TeamB" : "Multi");
            if (!list.empty()) {
                Team* team = list[pick(list.size())];
                if (chance(50)) {
                    team->setState(any(kStatePool));
                } else {
                    TheScriptEngine->notifyOfTeamDestruction(team);
                    ops << " teamgone " << team->getID();
                    ++n;
                }
            }
        } else if (op == 18) {
            sc.players[player]->*access_member(SePlayerDeadTag()) = chance(40);
            (sc.players[player]->*access_member(SePlayerAttackedTag()))[pick(3)] = chance(50);
        } else {
            // A topple adjustment, which only reads.
            Coord3D d = {3.0f, 4.0f, 0.5f};
            TheScriptEngine->adjustToppleDirection(obj, &d);
            ops << " topple " << tok(obj->getName()) << ' ' << bits(d.x) << ' ' << bits(d.y) << ' ' << bits(d.z);
            ++n;
        }
    }
    o << n << ops.str();
    return o.str();
}

// The reference's templates, one line each.
static void put_templates(FILE* out) {
    for (int kind = 0; kind < 2; ++kind) {
        const int count = kind ? ScriptAction::NUM_ITEMS : Condition::NUM_ITEMS;
        for (int i = 0; i < count; ++i) {
            const Template* t = kind ? static_cast<const Template*>(TheScriptEngine->getActionTemplate(i))
                                     : static_cast<const Template*>(TheScriptEngine->getConditionTemplate(i));
            std::ostringstream o;
            o << kind << ' ' << i << ' ' << tok(t->m_internalName) << ' ' << t->getNumParameters();
            for (int p = 0; p < t->getNumParameters(); ++p) o << ' ' << static_cast<int>(t->getParameterType(p));
            std::fprintf(out, "{\"seed\":%d,\"t\":\"template\",\"d\":\"%s\"},\n", i, o.str().c_str());
        }
    }
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_script_engine <out.json>\n");
        return 2;
    }
    static World w;
    w.load_ini(g_ini());
    w.init_with_neutral_player();
    TheWindowManager = new GameWindowManagerDummy;
    TheCampaignManager = new CampaignManager;
    TheScriptActions = new ScriptActions;
    TheScriptConditions = new ScriptConditions;

    Scene sc;
    for (int i = 0; i < 3; ++i) {
        Player* p = ThePlayerList->getNthPlayer(i);
        sc.players[i] = p;
        p->*access_member(SePlayerNameTag()) = NAMEKEY(kPlayerNames[i]);
        p->*access_member(SePlayerSideTag()) = kFactionPool[i == 0 ? 2 : i - 1];
        p->*access_member(SePlayerStartTag()) = i == 0 ? -1 : i - 1;
    }
    sc.players[1]->setPlayerType(PLAYER_HUMAN, false);
    sc.players[2]->setPlayerType(PLAYER_COMPUTER, false);
    ThePlayerList->setLocalPlayer(sc.players[1]);

    Dict single;
    TheTeamFactory->initTeam("Idle", AsciiString::TheEmptyString, TRUE, &single);
    Dict doomed;
    TheTeamFactory->initTeam("Doomed", AsciiString::TheEmptyString, TRUE, &doomed);
    Dict multi;
    multi.setInt(TheKey_teamMaxInstances, 3);
    TheTeamFactory->initTeam("Multi", AsciiString::TheEmptyString, FALSE, &multi);
    Dict lone;
    lone.setInt(TheKey_teamMaxInstances, 1);
    TheTeamFactory->initTeam("Lone", AsciiString::TheEmptyString, FALSE, &lone);
    for (int i = 0; i < 2; ++i) TheTeamFactory->createTeam("Multi")->setControllingPlayer(sc.players[2]);
    TheTeamFactory->createTeam("Lone")->setControllingPlayer(sc.players[1]);
    w.team_a->setActive();
    w.team_b->setActive();
    TheTeamFactory->findTeam("Doomed")->setActive();

    for (int i = 0; i < 3; ++i) {
        Dict side;
        side.setAsciiString(TheKey_playerName, kPlayerNames[i]);
        TheSidesList->addSide(&side);
    }

    FILE* out = std::fopen(argv[1], "w");
    if (!out) return 1;
    std::fprintf(out, "{\"vectors\":[\n");
    put_templates(out);
    // The engine as constructed and reset, before any map.
    TheScriptEngine->reset();
    std::fprintf(out, "{\"seed\":0,\"t\":\"fresh\",\"save\":\"%s\"},\n",
                 World::snapshot_save_hex(TheScriptEngine).c_str());

    const int kSeeds = 120;
    const int kFrames = 12;
    for (int seed = 0; seed < kSeeds; ++seed) {
        g_state = 0x9e3779b9u * (seed + 1);
        g_pure_random = seed % 2 == 1;
        for (int i = 0; i < 8; ++i) pick(2);
        w.set_frame(1 + pick(100));

        // World reset.
        for (int i = 0; i < 3; ++i) {
            Player* p = sc.players[i];
            p->*access_member(SePlayerDeadTag()) = FALSE;
            (p->*access_member(SePlayerSciencesTag())).clear();
            for (int j = 0; j < MAX_PLAYER_COUNT; ++j) (p->*access_member(SePlayerAttackedTag()))[j] = FALSE;
            p->setCanBuildUnits(TRUE);
            p->setCanBuildBase(TRUE);
            for (int j = 0; j < 3; ++j)
                if (i != j) p->setPlayerRelationship(sc.players[j], static_cast<Relationship>(pick(3)));
            set_money(p, pick(1200));
        }
        for (const char* proto : kProtos)
            for (Team* t : instances(proto)) t->setState("");

        // Objects: named units U0..U3, an unnamed tank, buildings and a crate.
        // Emptied instances are deleted by the team updates.
        while (instances("Multi").size() < 2) TheTeamFactory->createTeam("Multi")->setControllingPlayer(sc.players[2]);
        if (instances("Lone").empty()) TheTeamFactory->createTeam("Lone")->setControllingPlayer(sc.players[1]);
        Team* multi_teams[2] = {instances("Multi")[0], instances("Multi")[1]};
        struct Spawn { const char* tmpl; Team* team; const char* name; };
        const Spawn spawns[] = {
            {"ScrTank", w.team_a, "U0"},        {"ScrTank", w.team_b, "U1"},
            {"ScrTank", multi_teams[0], "U2"},  {"ScrDepot", w.team_a, "U3"},
            {"ScrTank", multi_teams[1], ""},    {"ScrBarracks", w.team_b, ""},
            {"ScrCrate", multi_teams[0], ""},   {"ScrVictim", TheTeamFactory->findTeam("Doomed"), "Victim"},
            {"ScrTank", instances("Lone")[0], ""},
        };
        for (const Spawn& s : spawns) {
            if (pick(100) < 12 && s.name[0] != 'U') continue;
            Object* obj = w.spawn_object(s.tmpl, s.team);
            if (*s.name) obj->setName(s.name);
            sc.objects.push_back(obj);
        }

        // Some scenarios are challenge campaign missions.
        const bool challenge = seed % 5 == 0;
        static Campaign* campaign = newInstance(Campaign);
        campaign->m_isChallengeCampaign = TRUE;
        TheCampaignManager->*access_member(SeCampaignTag()) = challenge ? campaign : nullptr;

        TheScriptEngine->reset();
        const GameDifficulty difficulty = static_cast<GameDifficulty>(pick(3));
        TheScriptEngine->setGlobalDifficulty(difficulty);

        // Script lists.
        std::vector<ScriptList*> lists(3, nullptr);
        for (int side = 0; side < 3; ++side)
            if (chance(80) || side == 1) lists[side] = newInstance(ScriptList);
        std::vector<ScriptGroup*> groups;
        std::vector<int> group_sides;
        ScriptList* first_list = lists[1];
        ScriptGroup* g0 = newInstance(ScriptGroup)();
        g0->setName("G0");
        g0->setActive(chance(80));
        g0->setSubroutine(chance(10));
        ScriptGroup* g1 = newInstance(ScriptGroup)();
        g1->setName("G1");
        g1->setActive(chance(80));
        g1->setSubroutine(chance(85));
        for (int i = 0; i < kScripts; ++i) {
            ScriptList* target = lists[pick(3)];
            if (!target) target = first_list;
            Script* s = make_script(i, g_scriptName(i));
            if (i == 7 || i == 8) g0->addScript(s, kAtEnd);
            else if (i == 10 || (i == 6 && chance(50))) g1->addScript(s, kAtEnd);
            else target->addScript(s, kAtEnd);
        }
        (chance(50) || !lists[0] ? first_list : lists[0])->addGroup(g0, kAtEnd);
        (chance(50) || !lists[2] ? first_list : lists[2])->addGroup(g1, kAtEnd);
        for (int side = 0; side < 3; ++side) TheSidesList->getSideInfo(side)->setScriptList(lists[side]);

        std::ostringstream setup;
        const uint32_t rng_seed = 1000u + seed;
        InitRandom(rng_seed);
        setup << rng_seed << ' ' << static_cast<int>(difficulty) << ' ' << TheGameLogic->getFrame() << ' '
              << int(KINDOF_CRATE);
        setup << ' ' << int(TheScienceStore->getScienceFromInternalName("SCIENCE_Alpha")) << ' '
              << int(TheScienceStore->getScienceFromInternalName("SCIENCE_Beta"));
        setup << ' ' << int(challenge) << ' ' << w.team_a->getID() << " 3";
        for (int side = 0; side < 3; ++side) {
            ScriptList* list = lists[side];
            if (!list) {
                setup << " 0";
                continue;
            }
            int n = 0;
            for (Script* s = list->getScript(); s; s = s->getNext()) ++n;
            setup << " 1 " << n;
            for (Script* s = list->getScript(); s; s = s->getNext()) put_script(setup, s);
            int ng = 0;
            for (ScriptGroup* g = list->getScriptGroup(); g; g = g->getNext()) ++ng;
            setup << ' ' << ng;
            for (ScriptGroup* g = list->getScriptGroup(); g; g = g->getNext()) {
                int ns = 0;
                for (Script* s = g->getScript(); s; s = s->getNext()) ++ns;
                setup << ' ' << tok(g->getName()) << ' ' << int(g->isActive()) << ' ' << int(g->isSubroutine()) << ' '
                      << ns;
                for (Script* s = g->getScript(); s; s = s->getNext()) put_script(setup, s);
            }
        }
        // The new map reads team prototypes.
        const std::string before_map = snapshot(sc);
        TheScriptEngine->newMap();
        std::fprintf(out, "{\"seed\":%d,\"t\":\"setup\",\"rng\":%u,\"d\":\"%s\",\"w\":\"%s\"},\n", seed,
                     GetGameLogicRandomSeedCRC(), setup.str().c_str(), before_map.c_str());

        for (int frame = 0; frame < kFrames; ++frame) {
            // The first update builds the name cache, so hooks come after it.
                const std::string ops = frame ? perturb(sc) : std::string("0");
            const std::string world = snapshot(sc);
            TheScriptEngine->update();

            std::ostringstream state;
            for (int side = 0; side < 3; ++side) {
                if (!lists[side]) continue;
                each_script(lists[side], [&](Script* s) { put_state(state, s); });
                for (ScriptGroup* g = lists[side]->getScriptGroup(); g; g = g->getNext())
                    state << ' ' << int(g->isActive());
            }
            std::ostringstream prio;
            const char* const sets[] = {"AP0", "AP1", "AP2", "AP3", "Unset"};
            for (const char* set : sets) {
                const AttackPriorityInfo* info = TheScriptEngine->getAttackInfo(set);
                for (int t = 0; t < kTemplateCount; ++t)
                    prio << info->getPriority(TheThingFactory->findTemplate(kTemplates[t])) << ' ';
            }
            std::fprintf(out,
                         "{\"seed\":%d,\"t\":\"frame\",\"frame\":%u,\"ops\":\"%s\",\"w\":\"%s\","
                         "\"save\":\"%s\",\"rng\":%u,\"state\":\"%s\",\"fx\":\"%s\",\"prio\":\"%s\"}%s\n",
                         seed, TheGameLogic->getFrame(), ops.c_str(), world.c_str(),
                         World::snapshot_save_hex(TheScriptEngine).c_str(), GetGameLogicRandomSeedCRC(),
                         state.str().c_str(), effects(sc).c_str(), prio.str().c_str(),
                         seed + 1 == kSeeds && frame + 1 == kFrames ? "" : ",");
            w.set_frame(TheGameLogic->getFrame() + 1 + (chance(10) ? 40 : 0));
        }

        TheScriptEngine->reset();
        for (int side = 0; side < 3; ++side) TheSidesList->getSideInfo(side)->setScriptList(nullptr);
        for (Object* obj : sc.objects) TheGameLogic->destroyObject(obj);
        w.flush_destroy_list();
        sc.objects.clear();
    }

    std::fprintf(out, "]}\n");
    std::fclose(out);
    return 0;
}
