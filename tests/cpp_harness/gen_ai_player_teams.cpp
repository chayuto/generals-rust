// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for AIPlayer / AISkirmishPlayer team building, recruiting,
// WorkOrder / TeamInQueue xfer & crc, isPossibleToBuildTeam, doTeamBuilding,
// buildSpecificAITeam, TeamPrototype::evaluateProductionCondition, Team::tryToRecruit,
// queueDozer, and queueSupplyTruck.
//
// Build 3 ways and check byte-identical output:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_ai_player_teams.cpp --opt O0 -o /tmp/gen_o0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_ai_player_teams.cpp --opt O2 -o /tmp/gen_o2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_ai_player_teams.cpp --opt O2 --sanitize -o /tmp/gen_ubsan

#include "oracle_wp51_world.h"

#include "Common/Money.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/Team.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/WellKnownKeys.h"
#include "GameLogic/AI.h"
#include "GameLogic/AIPlayer.h"
#include "GameLogic/AISkirmishPlayer.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/ProductionUpdate.h"
#include "GameLogic/Module/SupplyTruckAIUpdate.h"
#include "GameLogic/Module/SupplyWarehouseDockUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/ScriptEngine.h"
#include "GameLogic/Scripts.h"
#include "GameLogic/SidesList.h"

#include <cmath>
#include <cstdarg>
#include <cstdio>
#include <cstring>
#include <string>
#include <vector>

#define ACCESS(tag, cls, field, member)              \
    struct tag { using type = field cls::*; };       \
    template struct MemberAccess<tag, &cls::member>; \
    inline tag::type access_member(tag);
#define M(obj, tag) ((obj)->*access_member(tag()))

// ---- Member accessors for private/protected inspection ----
ACCESS(PtAiTag, Player, AIPlayer*, m_ai)
ACCESS(PtBuildListTag, Player, BuildListInfo*, m_pBuildList)
ACCESS(PtCanBuildUnitsTag, Player, Bool, m_canBuildUnits)

ACCESS(AiReadyTeamTag, AIPlayer, Bool, m_readyToBuildTeam)
ACCESS(AiTeamTimerTag, AIPlayer, Int, m_teamTimer)
ACCESS(AiTeamDelayTag, AIPlayer, Int, m_teamDelay)
ACCESS(AiTeamSecondsTag, AIPlayer, Int, m_teamSeconds)
ACCESS(AiDifficultyTag, AIPlayer, GameDifficulty, m_difficulty)
ACCESS(AiRepairOriginTag, AIPlayer, Coord3D, m_repairDozerOrigin)
ACCESS(AiSupplyCheckFrameTag, AIPlayer, UnsignedInt, m_supplySourceAttackCheckFrame)
ACCESS(AiAttackedSupplyTag, AIPlayer, ObjectID, m_attackedSupplyCenter)
ACCESS(AiRepairDozerTag, AIPlayer, ObjectID, m_repairDozer)

struct AiPrependBuildTag { using type = void (AIPlayer::*)(TeamInQueue*); };
template struct MemberAccess<AiPrependBuildTag, &AIPlayer::prependTo_TeamBuildQueue>;
inline AiPrependBuildTag::type access_member(AiPrependBuildTag);

struct AiPrependReadyTag { using type = void (AIPlayer::*)(TeamInQueue*); };
template struct MemberAccess<AiPrependReadyTag, &AIPlayer::prependTo_TeamReadyQueue>;
inline AiPrependReadyTag::type access_member(AiPrependReadyTag);

struct AiIterBuildTag { using type = DLINK_ITERATOR<TeamInQueue> (AIPlayer::*)() const; };
template struct MemberAccess<AiIterBuildTag, &AIPlayer::iterate_TeamBuildQueue>;
inline AiIterBuildTag::type access_member(AiIterBuildTag);

struct AiIterReadyTag { using type = DLINK_ITERATOR<TeamInQueue> (AIPlayer::*)() const; };
template struct MemberAccess<AiIterReadyTag, &AIPlayer::iterate_TeamReadyQueue>;
inline AiIterReadyTag::type access_member(AiIterReadyTag);

struct AiClearQueuesTag { using type = void (AIPlayer::*)(); };
template struct MemberAccess<AiClearQueuesTag, &AIPlayer::clearTeamsInQueue>;
inline AiClearQueuesTag::type access_member(AiClearQueuesTag);

struct AiIsPossibleTag { using type = Bool (AIPlayer::*)(TeamPrototype*, Bool, Bool&); };
template struct MemberAccess<AiIsPossibleTag, &AIPlayer::isPossibleToBuildTeam>;
inline AiIsPossibleTag::type access_member(AiIsPossibleTag);

struct AiDoTeamBuildingTag { using type = void (AIPlayer::*)(); };
template struct MemberAccess<AiDoTeamBuildingTag, &AIPlayer::doTeamBuilding>;
inline AiDoTeamBuildingTag::type access_member(AiDoTeamBuildingTag);

struct AiQueueDozerTag { using type = void (AIPlayer::*)(); };
template struct MemberAccess<AiQueueDozerTag, &AIPlayer::queueDozer>;
inline AiQueueDozerTag::type access_member(AiQueueDozerTag);

struct AiQueueTruckTag { using type = void (AIPlayer::*)(); };
template struct MemberAccess<AiQueueTruckTag, &AIPlayer::queueSupplyTruck>;
inline AiQueueTruckTag::type access_member(AiQueueTruckTag);

ACCESS(WoNextTag, WorkOrder, WorkOrder*, m_next)
ACCESS(WoThingTag, WorkOrder, const ThingTemplate*, m_thing)
ACCESS(WoFacIdTag, WorkOrder, ObjectID, m_factoryID)
ACCESS(WoNumCompTag, WorkOrder, Int, m_numCompleted)
ACCESS(WoNumReqTag, WorkOrder, Int, m_numRequired)
ACCESS(WoReqTag, WorkOrder, Bool, m_required)
ACCESS(WoGatherTag, WorkOrder, Bool, m_isResourceGatherer)

struct WoWaitTag { using type = Bool (WorkOrder::*)(); };
template struct MemberAccess<WoWaitTag, &WorkOrder::isWaitingToBuild>;
inline WoWaitTag::type access_member(WoWaitTag);

struct WoValTag { using type = void (WorkOrder::*)(Player*); };
template struct MemberAccess<WoValTag, &WorkOrder::validateFactory>;
inline WoValTag::type access_member(WoValTag);

ACCESS(TqOrdersTag, TeamInQueue, WorkOrder*, m_workOrders)
ACCESS(TqPriorityTag, TeamInQueue, Bool, m_priorityBuild)
ACCESS(TqTeamTag, TeamInQueue, Team*, m_team)
ACCESS(TqFrameStartedTag, TeamInQueue, Int, m_frameStarted)
ACCESS(TqSentTag, TeamInQueue, Bool, m_sentToStartLocation)
ACCESS(TqStopTag, TeamInQueue, Bool, m_stopQueueing)
ACCESS(TqReinfTag, TeamInQueue, Bool, m_reinforcement)
ACCESS(TqReinfIdTag, TeamInQueue, ObjectID, m_reinforcementID)

struct TqAllBuiltTag { using type = Bool (TeamInQueue::*)(); };
template struct MemberAccess<TqAllBuiltTag, &TeamInQueue::isAllBuilt>;
inline TqAllBuiltTag::type access_member(TqAllBuiltTag);

struct TqExpiredTag { using type = Bool (TeamInQueue::*)(); };
template struct MemberAccess<TqExpiredTag, &TeamInQueue::isBuildTimeExpired>;
inline TqExpiredTag::type access_member(TqExpiredTag);

struct TqMinBuiltTag { using type = Bool (TeamInQueue::*)(); };
template struct MemberAccess<TqMinBuiltTag, &TeamInQueue::isMinimumBuilt>;
inline TqMinBuiltTag::type access_member(TqMinBuiltTag);

struct TqIncDozerTag { using type = Bool (TeamInQueue::*)(); };
template struct MemberAccess<TqIncDozerTag, &TeamInQueue::includesADozer>;
inline TqIncDozerTag::type access_member(TqIncDozerTag);

struct TqCompleteTag { using type = Bool (TeamInQueue::*)(); };
template struct MemberAccess<TqCompleteTag, &TeamInQueue::areBuildsComplete>;
inline TqCompleteTag::type access_member(TqCompleteTag);

ACCESS(TpInfoTag, TeamPrototype, TeamTemplateInfo, m_teamTemplate)
ACCESS(TpAlwaysFalseTag, TeamPrototype, Bool, m_productionConditionAlwaysFalse)

ACCESS(GlFrameTag, GameLogic, UnsignedInt, m_frame)
ACCESS(GlNextIdTag, GameLogic, ObjectID, m_nextObjID)

namespace {

static std::vector<std::string> g_rows;

static std::string fmt(const char* f, ...) {
    static char buf[65536];
    va_list args;
    va_start(args, f);
    std::vsnprintf(buf, sizeof(buf), f, args);
    va_end(args);
    return buf;
}

static void row(const std::string& s) {
    if (s.find('"') != std::string::npos || s.find('\\') != std::string::npos) {
        std::fprintf(stderr, "unquotable row %s\n", s.c_str());
        std::exit(2);
    }
    g_rows.push_back(s);
}

static uint64_t g_state = 0x426427123456789bull;
static uint32_t pick(uint32_t n) {
    return n ? static_cast<uint32_t>(oracle::splitmix64(g_state) % n) : 0;
}
static bool chance(uint32_t percent) { return pick(100) < percent; }

static std::string commands_ini() {
    return
        "CommandButton Btn_UnitA\n  Command = UNIT_BUILD\n  Object = UnitA\nEnd\n"
        "CommandButton Btn_UnitB\n  Command = UNIT_BUILD\n  Object = UnitB\nEnd\n"
        "CommandButton Btn_UnitC\n  Command = UNIT_BUILD\n  Object = UnitC\nEnd\n"
        "CommandButton Btn_DozerA\n  Command = UNIT_BUILD\n  Object = DozerA\nEnd\n"
        "CommandButton Btn_DozerB\n  Command = UNIT_BUILD\n  Object = DozerB\nEnd\n"
        "CommandButton Btn_WorkerGatherer\n  Command = UNIT_BUILD\n  Object = WorkerGatherer\nEnd\n"
        "CommandSet CmdFactA\n"
        "  1 = Btn_UnitA\n"
        "  2 = Btn_UnitC\n"
        "  3 = Btn_DozerA\n"
        "  4 = Btn_DozerB\n"
        "  5 = Btn_WorkerGatherer\n"
        "End\n"
        "CommandSet CmdFactB\n"
        "  1 = Btn_UnitB\n"
        "End\n";
}

static std::string fixture_ini() {
    return
        "Locomotor AptGround\n"
        "  Surfaces = GROUND\n"
        "  Appearance = FOUR_WHEELS\n"
        "  Speed = 30\n"
        "  TurnRate = 180\n"
        "  Acceleration = 60\n"
        "  Braking = 60\n"
        "  MinSpeed = 0\n"
        "End\n"
        "PlayerTemplate FactionCivilian\n  Side = Civilian\n  PreferredColor = R:100 G:100 B:100\nEnd\n"
        "PlayerTemplate FactionAmerica\n  Side = America\n  PreferredColor = R:10 G:20 B:255\nEnd\n"
        "Object UnitA\n"
        "  Side = America\n  KindOf = PRELOAD SELECTABLE VEHICLE\n"
        "  BuildCost = 1000\n  BuildTime = 1.0\n"
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 6\n  GeometryHeight = 8\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "  Locomotor = SET_NORMAL AptGround\n"
        "End\n"
        "Object UnitBVar\n"
        "  Side = America\n  KindOf = PRELOAD SELECTABLE INFANTRY\n"
        "  BuildCost = 333\n  BuildTime = 0.5\n"
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 4\n  GeometryHeight = 6\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "  Locomotor = SET_NORMAL AptGround\n"
        "End\n"
        "Object UnitB\n"
        "  Side = America\n  KindOf = PRELOAD SELECTABLE INFANTRY\n"
        "  BuildCost = 333\n  BuildTime = 0.5\n"
        "  BuildVariations = UnitBVar\n"
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 4\n  GeometryHeight = 6\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "  Locomotor = SET_NORMAL AptGround\n"
        "End\n"
        "Object UnitC\n"
        "  Side = America\n  KindOf = PRELOAD SELECTABLE VEHICLE\n"
        "  BuildCost = 777\n  BuildTime = 1.0\n"
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 6\n  GeometryHeight = 8\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 150\n    InitialHealth = 150\n  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "  Locomotor = SET_NORMAL AptGround\n"
        "End\n"
        "Object UnitNoFact\n"
        "  Side = America\n  KindOf = PRELOAD SELECTABLE VEHICLE\n"
        "  BuildCost = 500\n  BuildTime = 1.0\n"
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 6\n  GeometryHeight = 8\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 150\n    InitialHealth = 150\n  End\n"
        "End\n"
        "Object DozerA\n"
        "  Side = America\n  KindOf = PRELOAD SELECTABLE VEHICLE DOZER\n"
        "  BuildCost = 1000\n  BuildTime = 1.0\n"
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 7\n  GeometryHeight = 8\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n"
        "  Behavior = DozerAIUpdate ModuleTag_AI\n    RepairHealthPercentPerSecond = 2%\n    BoredTime = 5000\n    BoredRange = 150\n  End\n"
        "  Locomotor = SET_NORMAL AptGround\n"
        "End\n"
        "Object DozerB\n"
        "  Side = America\n  KindOf = PRELOAD SELECTABLE VEHICLE DOZER\n"
        "  BuildCost = 1200\n  BuildTime = 1.0\n"
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 7\n  GeometryHeight = 8\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n"
        "  Behavior = DozerAIUpdate ModuleTag_AI\n    RepairHealthPercentPerSecond = 2%\n    BoredTime = 5000\n    BoredRange = 150\n  End\n"
        "  Locomotor = SET_NORMAL AptGround\n"
        "End\n"
        "Object WorkerGatherer\n"
        "  Side = America\n  KindOf = PRELOAD SELECTABLE VEHICLE HARVESTER\n"
        "  BuildCost = 600\n  BuildTime = 1.0\n"
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 6\n  GeometryHeight = 8\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 150\n    InitialHealth = 150\n  End\n"
        "  Behavior = SupplyTruckAIUpdate ModuleTag_AI\n    MaxBoxes = 4\n  End\n"
        "  Locomotor = SET_NORMAL AptGround\n"
        "End\n"
        "Object FactA\n"
        "  Side = America\n  KindOf = PRELOAD STRUCTURE FS_FACTORY FS_WARFACTORY\n"
        "  CommandSet = CmdFactA\n"
        "  Geometry = BOX\n  GeometryMajorRadius = 20\n  GeometryMinorRadius = 20\n  GeometryHeight = 20\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 1000\n  End\n"
        "  Behavior = ProductionUpdate ModuleTag_Prod\n  End\n"
        "  Behavior = DefaultProductionExitUpdate ModuleTag_Exit\n"
        "    UnitCreatePoint = X:0.0 Y:0.0 Z:0.0\n"
        "    NaturalRallyPoint = X:40.0 Y:0.0 Z:0.0\n"
        "  End\n"
        "End\n"
        "Object FactB\n"
        "  Side = America\n  KindOf = PRELOAD STRUCTURE FS_FACTORY FS_BARRACKS\n"
        "  CommandSet = CmdFactB\n"
        "  Geometry = BOX\n  GeometryMajorRadius = 20\n  GeometryMinorRadius = 20\n  GeometryHeight = 20\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 1000\n  End\n"
        "  Behavior = ProductionUpdate ModuleTag_Prod\n  End\n"
        "  Behavior = DefaultProductionExitUpdate ModuleTag_Exit\n"
        "    UnitCreatePoint = X:0.0 Y:0.0 Z:0.0\n"
        "    NaturalRallyPoint = X:40.0 Y:0.0 Z:0.0\n"
        "  End\n"
        "End\n"
        "Object SupplyCtr\n"
        "  Side = America\n  KindOf = PRELOAD STRUCTURE FS_SUPPLY_CENTER\n"
        "  Geometry = BOX\n  GeometryMajorRadius = 25\n  GeometryMinorRadius = 20\n  GeometryHeight = 15\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 1000\n  End\n"
        "  Behavior = SupplyCenterDockUpdate ModuleTag_Dock\n    NumberApproachPositions = 2\n  End\n"
        "End\n"
        "Object Warehouse\n"
        "  Side = America\n  KindOf = PRELOAD STRUCTURE SUPPLY_SOURCE\n"
        "  Geometry = BOX\n  GeometryMajorRadius = 20\n  GeometryMinorRadius = 20\n  GeometryHeight = 15\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 1000\n  End\n"
        "  Behavior = SupplyWarehouseDockUpdate ModuleTag_Dock\n    NumberApproachPositions = 2\n    StartingBoxes = 20\n  End\n"
        "End\n";
}

static Script* make_cond_script(const char* name, Condition::ConditionType cond_type,
                                bool easy, bool normal, bool hard, int delay_sec) {
    Script* s = newInstance(Script);
    s->setName(AsciiString(name));
    s->setActive(TRUE);
    s->setEasy(easy ? TRUE : FALSE);
    s->setNormal(normal ? TRUE : FALSE);
    s->setHard(hard ? TRUE : FALSE);
    s->setDelayEvalSeconds(delay_sec);
    Condition* c = newInstance(Condition)(cond_type);
    OrCondition* oc = newInstance(OrCondition);
    oc->setFirstAndCondition(c);
    s->setOrCondition(oc);
    return s;
}

static void add_team_proto(const char* name, const char* owner, bool singleton,
                           int prio, int max_inst, const std::string& prod_cond,
                           const std::vector<TCreateUnitsInfo>& units,
                           const char* transport = "", bool starts_full = false) {
    Dict d;
    d.setAsciiString(NAMEKEY("teamName"), AsciiString(name));
    d.setAsciiString(NAMEKEY("teamOwner"), AsciiString(owner));
    d.setBool(NAMEKEY("teamIsSingleton"), singleton);
    d.setInt(NAMEKEY("teamProductionPriority"), prio);
    d.setInt(NAMEKEY("teamMaxInstances"), max_inst);
    d.setBool(NAMEKEY("teamStartsFull"), starts_full);
    if (!prod_cond.empty()) {
        d.setAsciiString(NAMEKEY("teamProductionCondition"), AsciiString(prod_cond.c_str()));
    }
    if (transport && transport[0]) {
        d.setAsciiString(NAMEKEY("teamTransport"), AsciiString(transport));
    }
    d.setInt(NAMEKEY("teamUnitMinCount1"), units.size() > 0 ? units[0].minUnits : 0);
    d.setInt(NAMEKEY("teamUnitMaxCount1"), units.size() > 0 ? units[0].maxUnits : 0);
    if (units.size() > 0) d.setAsciiString(NAMEKEY("teamUnitType1"), units[0].unitThingName);
    if (units.size() > 1) {
        d.setInt(NAMEKEY("teamUnitMinCount2"), units[1].minUnits);
        d.setInt(NAMEKEY("teamUnitMaxCount2"), units[1].maxUnits);
        d.setAsciiString(NAMEKEY("teamUnitType2"), units[1].unitThingName);
    }
    if (units.size() > 2) {
        d.setInt(NAMEKEY("teamUnitMinCount3"), units[2].minUnits);
        d.setInt(NAMEKEY("teamUnitMaxCount3"), units[2].maxUnits);
        d.setAsciiString(NAMEKEY("teamUnitType3"), units[2].unitThingName);
    }
    TheSidesList->addTeam(&d);
}

static void setup_world(oracle_wp51::World& w, bool skirmish) {
    TheSidesList->emptySides();
    TheSidesList->emptyTeams();
    TheTeamFactory->reset();

    // Neutral
    {
        Dict d;
        d.setAsciiString(NAMEKEY("playerName"), AsciiString(""));
        d.setAsciiString(NAMEKEY("playerFaction"), AsciiString("FactionCivilian"));
        d.setBool(NAMEKEY("playerIsHuman"), FALSE);
        TheSidesList->addSide(&d);
        Dict td;
        td.setAsciiString(NAMEKEY("teamName"), AsciiString("team"));
        td.setAsciiString(NAMEKEY("teamOwner"), AsciiString(""));
        td.setBool(NAMEKEY("teamIsSingleton"), TRUE);
        TheSidesList->addTeam(&td);
    }
    // AI player (idx 1)
    {
        Dict d;
        d.setAsciiString(NAMEKEY("playerName"), AsciiString("AiPlayer1"));
        d.setAsciiString(NAMEKEY("playerFaction"), AsciiString("FactionAmerica"));
        d.setBool(NAMEKEY("playerIsHuman"), FALSE);
        d.setInt(NAMEKEY("skirmishDifficulty"), DIFFICULTY_NORMAL);
        TheSidesList->addSide(&d);

        ScriptList* sl = newInstance(ScriptList);
        sl->addScript(make_cond_script("ScrTrueAll", Condition::CONDITION_TRUE, true, true, true, 0), 0);
        sl->addScript(make_cond_script("ScrFalseAll", Condition::CONDITION_FALSE, true, true, true, 0), 1);
        sl->addScript(make_cond_script("ScrEasyOnly", Condition::CONDITION_TRUE, true, false, false, 0), 2);
        sl->addScript(make_cond_script("ScrDelay3", Condition::CONDITION_TRUE, true, true, true, 3), 3);
        TheSidesList->getSideInfo(1)->setScriptList(sl);

        Dict td;
        td.setAsciiString(NAMEKEY("teamName"), AsciiString("teamAiPlayer1"));
        td.setAsciiString(NAMEKEY("teamOwner"), AsciiString("AiPlayer1"));
        td.setBool(NAMEKEY("teamIsSingleton"), TRUE);
        TheSidesList->addTeam(&td);
    }
    // Enemy player (idx 2)
    {
        Dict d;
        d.setAsciiString(NAMEKEY("playerName"), AsciiString("EnemyPlayer2"));
        d.setAsciiString(NAMEKEY("playerFaction"), AsciiString("FactionAmerica"));
        d.setBool(NAMEKEY("playerIsHuman"), TRUE);
        TheSidesList->addSide(&d);
        Dict td;
        td.setAsciiString(NAMEKEY("teamName"), AsciiString("teamEnemyPlayer2"));
        td.setAsciiString(NAMEKEY("teamOwner"), AsciiString("EnemyPlayer2"));
        td.setBool(NAMEKEY("teamIsSingleton"), TRUE);
        TheSidesList->addTeam(&td);
    }

    // Team prototypes for AiPlayer1
    add_team_proto("TeamAlpha", "AiPlayer1", false, 10, 3, "ScrTrueAll",
                   {{1, 3, AsciiString("UnitA")}});
    add_team_proto("TeamLow", "AiPlayer1", true, 5, 1, "", {});
    add_team_proto("TeamHigh", "AiPlayer1", true, 20, 1, "", {});
    add_team_proto("TeamTestPoss", "AiPlayer1", false, 10, 1, "", {{1, 1, AsciiString("UnitA")}});
    add_team_proto("BT_Min0Max3", "AiPlayer1", false, 10, 5, "", {{0, 3, AsciiString("UnitA")}});
    add_team_proto("BT_Min2Max2", "AiPlayer1", false, 10, 5, "", {{2, 2, AsciiString("UnitA")}});
    add_team_proto("BT_Min1Max4", "AiPlayer1", false, 10, 5, "", {{1, 4, AsciiString("UnitA")}});
    add_team_proto("BT_Multi", "AiPlayer1", false, 10, 5, "",
                   {{1, 3, AsciiString("UnitA")}, {0, 2, AsciiString("UnitB")}, {2, 2, AsciiString("UnitC")}});
    add_team_proto("BT_Transport", "AiPlayer1", false, 10, 5, "",
                   {{1, 2, AsciiString("UnitA")}}, "UnitC", true);
    add_team_proto("EC_Empty", "AiPlayer1", false, 10, 3, "", {});
    add_team_proto("EC_Missing", "AiPlayer1", false, 10, 3, "NoSuchScript", {});
    add_team_proto("EC_True", "AiPlayer1", false, 10, 3, "ScrTrueAll", {});
    add_team_proto("EC_False", "AiPlayer1", false, 10, 3, "ScrFalseAll", {});
    add_team_proto("EC_Easy", "AiPlayer1", false, 10, 3, "ScrEasyOnly", {});
    add_team_proto("EC_Delay3", "AiPlayer1", false, 10, 3, "ScrDelay3", {});
    add_team_proto("RecruitTarget", "AiPlayer1", false, 10, 3, "", {});
    add_team_proto("RecSrcOk", "AiPlayer1", false, 5, 3, "", {});
    add_team_proto("RecSrcHighPrio", "AiPlayer1", false, 20, 3, "", {});
    add_team_proto("RecSrcNoFlag", "AiPlayer1", false, 5, 3, "", {});

    ThePlayerList->newGame();
    TheScriptEngine->newMap();
    if (skirmish) {
        ThePlayerList->getNthPlayer(1)->setPlayerType(PLAYER_COMPUTER, TRUE);
    }
    TheGameLogic->*access_member(GlFrameTag()) = 1;

    // Ensure AIPlayer has base objects cleared if any leftover.
    for (Object* o = TheGameLogic->getFirstObject(); o; ) {
        Object* nxt = o->getNextObject();
        TheGameLogic->destroyObject(o);
        o = nxt;
    }
    w.flush_destroy_list();
}

static const char* const kThings[] = {
    nullptr, "UnitA", "UnitB", "UnitBVar", "UnitC", "DozerA", "DozerB", "WorkerGatherer"
};

static WorkOrder* make_wo(int thing_idx, ObjectID fac_id, Int num_comp, Int num_req, bool req, bool gath) {
    WorkOrder* w = newInstance(WorkOrder);
    M(w, WoThingTag) = kThings[thing_idx] ? TheThingFactory->findTemplate(kThings[thing_idx]) : nullptr;
    M(w, WoFacIdTag) = fac_id;
    M(w, WoNumCompTag) = num_comp;
    M(w, WoNumReqTag) = num_req;
    M(w, WoReqTag) = req ? TRUE : FALSE;
    M(w, WoGatherTag) = gath ? TRUE : FALSE;
    return w;
}

static void run_wo_and_tiq_xfer(oracle_wp51::World& w) {
    setup_world(w, false);
    Player* p1 = ThePlayerList->getNthPlayer(1);
    Player* p2 = ThePlayerList->getNthPlayer(2);
    Object* my_fac = w.spawn_object("FactA", p1->getDefaultTeam(), {100.0f, 100.0f, 0.0f});
    Object* enemy_fac = w.spawn_object("FactA", p2->getDefaultTeam(), {300.0f, 300.0f, 0.0f});
    const ObjectID my_id = my_fac->getID();
    const ObjectID enemy_id = enemy_fac->getID();

    // 200 WorkOrder vectors
    for (int i = 0; i < 200; ++i) {
        int ti = pick(8);
        int fac_kind = pick(5);
        ObjectID fac_id = INVALID_ID;
        if (fac_kind == 1) fac_id = my_id;
        else if (fac_kind == 2) fac_id = enemy_id;
        else if (fac_kind == 3) fac_id = static_cast<ObjectID>(9999);
        else if (fac_kind == 4) fac_id = static_cast<ObjectID>(1);
        Int comp = static_cast<Int>(pick(6));
        Int req_cnt = static_cast<Int>(pick(6));
        bool is_req = chance(50);
        bool gath = chance(40);

        WorkOrder* wo = make_wo(ti, fac_id, comp, req_cnt, is_req, gath);
        Bool wait_before = (wo->*access_member(WoWaitTag()))();
        uint32_t crc = oracle_wp51::World::snapshot_crc(wo);
        std::string hex = oracle_wp51::World::snapshot_save_hex(wo);
        (wo->*access_member(WoValTag()))(p1);
        ObjectID fac_after = M(wo, WoFacIdTag);
        row(fmt("WOXFER %d %s %u %d %d %d %d %d %u %08x %s",
                i, kThings[ti] ? kThings[ti] : "-",
                static_cast<unsigned>(fac_id), comp, req_cnt,
                is_req ? 1 : 0, gath ? 1 : 0,
                wait_before ? 1 : 0, static_cast<unsigned>(fac_after), crc, hex.c_str()));
        deleteInstance(wo);
    }

    // Create teams for TeamInQueue
    Team* t_alpha1 = TheTeamFactory->createTeam(AsciiString("TeamAlpha"));
    Team* t_low = TheTeamFactory->findTeam(AsciiString("TeamLow"));

    // 200 TeamInQueue vectors
    for (int i = 0; i < 200; ++i) {
        TeamInQueue* tq = newInstance(TeamInQueue);
        int n_wo = pick(5);
        std::string specs;
        // Prepend into m_workOrders (linked list via m_next)
        for (int k = 0; k < n_wo; ++k) {
            int ti = 1 + pick(7);
            ObjectID fid = chance(35) ? my_id : INVALID_ID;
            Int comp = static_cast<Int>(pick(4));
            Int req_cnt = static_cast<Int>(pick(5));
            bool is_req = chance(50);
            bool gath = chance(35);
            WorkOrder* wo = make_wo(ti, fid, comp, req_cnt, is_req, gath);
            M(wo, WoNextTag) = M(tq, TqOrdersTag);
            M(tq, TqOrdersTag) = wo;
            if (!specs.empty()) specs += ";";
            specs += fmt("%s,%u,%d,%d,%d,%d", kThings[ti], static_cast<unsigned>(fid),
                         comp, req_cnt, is_req ? 1 : 0, gath ? 1 : 0);
        }
        bool prio = chance(50);
        int team_sel = pick(3);
        Team* team_ptr = team_sel == 0 ? nullptr : (team_sel == 1 ? t_alpha1 : t_low);
        Int fs_pool[] = {0, 1, 10, 500, -5};
        Int frame_started = fs_pool[pick(5)];
        bool sent = chance(50);
        bool stop = chance(50);
        bool reinf = chance(50);
        ObjectID reinf_id = chance(50) ? static_cast<ObjectID>(77) : INVALID_ID;

        M(tq, TqPriorityTag) = prio ? TRUE : FALSE;
        M(tq, TqTeamTag) = team_ptr;
        M(tq, TqFrameStartedTag) = frame_started;
        M(tq, TqSentTag) = sent ? TRUE : FALSE;
        M(tq, TqStopTag) = stop ? TRUE : FALSE;
        M(tq, TqReinfTag) = reinf ? TRUE : FALSE;
        M(tq, TqReinfIdTag) = reinf_id;

        Bool all_b = (tq->*access_member(TqAllBuiltTag()))();
        Bool min_b = (tq->*access_member(TqMinBuiltTag()))();
        Bool doz_b = (tq->*access_member(TqIncDozerTag()))();
        Bool done_b = (tq->*access_member(TqCompleteTag()))();

        // Check isBuildTimeExpired on team_alpha1 prototype with various initialIdleFrames + frame
        Int idle_pool[] = {-1, 0, 1, 30};
        std::string exp_bits;
        if (team_ptr) {
            TeamPrototype* proto = const_cast<TeamPrototype*>(team_ptr->getPrototype());
            Int orig_idle = M(proto, TpInfoTag).m_initialIdleFrames;
            for (Int idle : idle_pool) {
                UnsignedInt frames[] = {
                    0u,
                    static_cast<UnsignedInt>(frame_started + idle - 1),
                    static_cast<UnsignedInt>(frame_started + idle),
                    static_cast<UnsignedInt>(frame_started + idle + 1)
                };
                M(proto, TpInfoTag).m_initialIdleFrames = idle;
                for (UnsignedInt f : frames) {
                    TheGameLogic->*access_member(GlFrameTag()) = f;
                    exp_bits += (tq->*access_member(TqExpiredTag()))() ? "1" : "0";
                }
            }
            M(proto, TpInfoTag).m_initialIdleFrames = orig_idle;
        } else {
            exp_bits = "-";
        }

        uint32_t crc = oracle_wp51::World::snapshot_crc(tq);
        std::string hex = oracle_wp51::World::snapshot_save_hex(tq);
        if (specs.empty()) specs = "-";
        row(fmt("TIQXFER %d %s %d %u %d %d %d %d %u %d%d%d%d %s %08x %s",
                i, specs.c_str(), prio ? 1 : 0,
                team_ptr ? static_cast<unsigned>(team_ptr->getID()) : 0u,
                frame_started, sent ? 1 : 0, stop ? 1 : 0, reinf ? 1 : 0,
                static_cast<unsigned>(reinf_id),
                all_b ? 1 : 0, min_b ? 1 : 0, doz_b ? 1 : 0, done_b ? 1 : 0,
                exp_bits.c_str(), crc, hex.c_str()));
        deleteInstance(tq);
    }

    // 60 AIPlayer / AISkirmishPlayer xfer vectors
    for (int i = 0; i < 60; ++i) {
        const bool sk = (i % 2 == 1);
        setup_world(w, sk);
        Player* p = ThePlayerList->getNthPlayer(1);
        AIPlayer* ai = M(p, PtAiTag);
        Team* t1 = TheTeamFactory->createTeam(AsciiString("TeamAlpha"));
        Team* t2 = TheTeamFactory->findTeam(AsciiString("TeamLow"));

        int nb = pick(3);
        int nr = pick(3);
        for (int b = 0; b < nb; ++b) {
            TeamInQueue* tq = newInstance(TeamInQueue);
            WorkOrder* wo = make_wo(1 + b, INVALID_ID, b, b + 2, true, false);
            M(tq, TqOrdersTag) = wo;
            M(tq, TqPriorityTag) = (b % 2 == 0) ? TRUE : FALSE;
            M(tq, TqTeamTag) = b == 0 ? t1 : t2;
            M(tq, TqFrameStartedTag) = 10 + b;
            (ai->*access_member(AiPrependBuildTag()))(tq);
        }
        for (int r = 0; r < nr; ++r) {
            TeamInQueue* tq = newInstance(TeamInQueue);
            WorkOrder* wo = make_wo(2 + r, INVALID_ID, 2, 2, true, false);
            M(tq, TqOrdersTag) = wo;
            M(tq, TqTeamTag) = t2;
            M(tq, TqFrameStartedTag) = 20 + r;
            M(tq, TqReinfTag) = TRUE;
            M(tq, TqReinfIdTag) = static_cast<ObjectID>(50 + r);
            (ai->*access_member(AiPrependReadyTag()))(tq);
        }
        M(ai, AiRepairOriginTag) = {12.5f + i, -4.0f, 0.0f};
        M(ai, AiSupplyCheckFrameTag) = static_cast<UnsignedInt>(100 + i);
        M(ai, AiAttackedSupplyTag) = static_cast<ObjectID>(i % 3 == 0 ? 77 : 0);

        uint32_t crc = oracle_wp51::World::snapshot_crc(ai);
        std::string hex = oracle_wp51::World::snapshot_save_hex(ai);
        row(fmt("AIXFER %d %d %d %d %08x %s", i, sk ? 1 : 0, nb, nr, crc, hex.c_str()));
    }
}

static void run_is_possible_1000(oracle_wp51::World& w) {
    setup_world(w, true);
    Player* p1 = ThePlayerList->getNthPlayer(1);
    AIPlayer* ai = M(p1, PtAiTag);
    M(p1, PtCanBuildUnitsTag) = TRUE;

    Object* fa = w.spawn_object("FactA", p1->getDefaultTeam(), {100.0f, 100.0f, 0.0f});
    Object* fb = w.spawn_object("FactB", p1->getDefaultTeam(), {200.0f, 100.0f, 0.0f});

    BuildListInfo* bl_a = newInstance(BuildListInfo);
    bl_a->setTemplateName(AsciiString("FactA"));
    bl_a->setObjectID(fa->getID());
    BuildListInfo* bl_b = newInstance(BuildListInfo);
    bl_b->setTemplateName(AsciiString("FactB"));
    bl_b->setObjectID(fb->getID());
    bl_a->setNextBuildList(bl_b);
    M(p1, PtBuildListTag) = bl_a;

    // Prototype we mutate per case
    TeamPrototype* proto = TheTeamFactory->findTeamPrototype(AsciiString("TeamTestPoss"));
    TAiData* data = const_cast<TAiData*>(TheAI->getAiData());
    const Real orig_res = data->m_teamResourcesToBuild;

    const Real res_pool[] = {0.0f, 0.25f, 0.333f, 0.5f, 0.75f, 1.0f, 1.25f, 1.5f, 2.0f};
    const char* unit_pool[] = {"UnitA", "UnitB", "UnitC", "UnitNoFact"};
    const Int cost_pool[] = {1000, 333, 777, 500};

    ProductionUpdateInterface* pu_a = fa->getProductionUpdateInterface();
    ProductionUpdateInterface* pu_b = fb->getProductionUpdateInterface();
    const ThingTemplate* t_a = TheThingFactory->findTemplate("UnitA");
    const ThingTemplate* t_b = TheThingFactory->findTemplate("UnitB");

    for (int i = 0; i < 1000; ++i) {
        Real res = res_pool[pick(9)];
        data->m_teamResourcesToBuild = res;

        bool fa_busy = chance(40);
        bool fb_busy = chance(40);
        bool fa_present = chance(85);
        bool require_idle = chance(50);

        pu_a->cancelAndRefundAllProduction();
        pu_b->cancelAndRefundAllProduction();
        if (fa_busy) pu_a->queueCreateUnit(t_a, pu_a->requestUniqueUnitID());
        if (fb_busy) pu_b->queueCreateUnit(t_b, pu_b->requestUniqueUnitID());
        bl_a->setObjectID(fa_present ? fa->getID() : INVALID_ID);

        int n_units = 1 + pick(3);
        TeamTemplateInfo& info = M(proto, TpInfoTag);
        info.m_numUnitsInfo = n_units;
        std::string uspec;
        Int approx_cost = 0;
        for (int u = 0; u < n_units; ++u) {
            int ui = pick(4);
            // Bias toward valid units so money boundaries trigger often
            if (i < 850 && ui == 3) ui = pick(3);
            Int mn = pick(5);
            Int mx = mn + pick(5);
            info.m_unitsInfo[u].minUnits = mn;
            info.m_unitsInfo[u].maxUnits = mx;
            info.m_unitsInfo[u].unitThingName = AsciiString(unit_pool[ui]);
            if (!uspec.empty()) uspec += ";";
            uspec += fmt("%s,%d,%d", unit_pool[ui], mn, mx);
            Real avg = static_cast<Real>(mx + mn) / 2.0f;
            approx_cost = static_cast<Int>(static_cast<Real>(approx_cost) + cost_pool[ui] * avg);
        }
        approx_cost = static_cast<Int>(static_cast<Real>(approx_cost) * res);

        UnsignedInt money = 0;
        int mkind = pick(5);
        if (mkind == 0 && approx_cost > 0) money = static_cast<UnsignedInt>(approx_cost - 1);
        else if (mkind == 1 && approx_cost >= 0) money = static_cast<UnsignedInt>(approx_cost);
        else if (mkind == 2 && approx_cost >= 0) money = static_cast<UnsignedInt>(approx_cost + 1);
        else if (mkind == 3) money = 0;
        else money = static_cast<UnsignedInt>(pick(20000));

        UnsignedInt cur = p1->getMoney()->countMoney();
        if (cur > 0) p1->getMoney()->withdraw(cur, FALSE);
        if (money > 0) p1->getMoney()->deposit(money, FALSE, FALSE);

        Bool need_money = FALSE;
        Bool ok = (ai->*access_member(AiIsPossibleTag()))(proto, require_idle ? TRUE : FALSE, need_money);
        row(fmt("ISPOSS %d %08x %d %d %d %d %u %s %d %d",
                i, oracle::fbits(res), fa_present ? 1 : 0, fa_busy ? 1 : 0, fb_busy ? 1 : 0,
                require_idle ? 1 : 0, money, uspec.c_str(), ok ? 1 : 0, need_money ? 1 : 0));
    }

    pu_a->cancelAndRefundAllProduction();
    pu_b->cancelAndRefundAllProduction();
    data->m_teamResourcesToBuild = orig_res;
}

static void run_do_team_building(oracle_wp51::World& w) {
    struct Case { bool sk; bool can_units; Int timer; Bool ready; Int delay; };
    const Case cases[] = {
        {false, false, 2, FALSE, 0},
        {false, true, 2, FALSE, 0},
        {true, true, 2, FALSE, 0},
        {false, true, 200, FALSE, 0},
        {true, true, 200, FALSE, 0},
        {true, true, 1, FALSE, 4},
    };
    for (int ci = 0; ci < 6; ++ci) {
        setup_world(w, cases[ci].sk);
        Player* p = ThePlayerList->getNthPlayer(1);
        AIPlayer* ai = M(p, PtAiTag);
        M(p, PtCanBuildUnitsTag) = cases[ci].can_units ? TRUE : FALSE;
        M(ai, AiTeamTimerTag) = cases[ci].timer;
        M(ai, AiReadyTeamTag) = cases[ci].ready;
        M(ai, AiTeamDelayTag) = cases[ci].delay;
        std::string seq;
        for (int step = 0; step < 8; ++step) {
            (ai->*access_member(AiDoTeamBuildingTag()))();
            if (!seq.empty()) seq += ";";
            seq += fmt("%d,%d,%d", M(ai, AiReadyTeamTag) ? 1 : 0,
                       M(ai, AiTeamTimerTag), M(ai, AiTeamDelayTag));
        }
        row(fmt("DOTB %d %d %d %d %d %d %s", ci, cases[ci].sk ? 1 : 0, cases[ci].can_units ? 1 : 0,
                cases[ci].timer, cases[ci].ready ? 1 : 0, cases[ci].delay, seq.c_str()));
    }
}

static void run_build_specific_team(oracle_wp51::World& w) {
    setup_world(w, true);
    Player* p1 = ThePlayerList->getNthPlayer(1);
    AIPlayer* ai = M(p1, PtAiTag);
    M(p1, PtCanBuildUnitsTag) = TRUE;
    Object* fa = w.spawn_object("FactA", p1->getDefaultTeam(), {100.0f, 100.0f, 0.0f});
    Object* fb = w.spawn_object("FactB", p1->getDefaultTeam(), {200.0f, 100.0f, 0.0f});
    BuildListInfo* bl_a = newInstance(BuildListInfo);
    bl_a->setTemplateName(AsciiString("FactA"));
    bl_a->setObjectID(fa->getID());
    BuildListInfo* bl_b = newInstance(BuildListInfo);
    bl_b->setTemplateName(AsciiString("FactB"));
    bl_b->setObjectID(fb->getID());
    bl_a->setNextBuildList(bl_b);
    M(p1, PtBuildListTag) = bl_a;

    const char* names[] = {"BT_Min0Max3", "BT_Min2Max2", "BT_Min1Max4", "BT_Multi", "BT_Transport"};
    for (int i = 0; i < 5; ++i) {
        for (int prio = 0; prio <= 1; ++prio) {
            (ai->*access_member(AiClearQueuesTag()))();
            fa->getProductionUpdateInterface()->cancelAndRefundAllProduction();
            fb->getProductionUpdateInterface()->cancelAndRefundAllProduction();

            TeamPrototype* proto = TheTeamFactory->findTeamPrototype(AsciiString(names[i]));
            ai->buildSpecificAITeam(proto, prio != 0);

            std::string qstr;
            for (DLINK_ITERATOR<TeamInQueue> it = (ai->*access_member(AiIterBuildTag()))(); !it.done(); it.advance()) {
                TeamInQueue* tq = it.cur();
                if (!qstr.empty()) qstr += "|";
                std::string wostr;
                for (WorkOrder* wo = M(tq, TqOrdersTag); wo; wo = M(wo, WoNextTag)) {
                    if (!wostr.empty()) wostr += ";";
                    const ThingTemplate* tt = M(wo, WoThingTag);
                    wostr += fmt("%s,%d,%d,%u", tt ? tt->getName().str() : "-",
                                 M(wo, WoNumReqTag), M(wo, WoReqTag) ? 1 : 0,
                                 M(wo, WoFacIdTag) != INVALID_ID ? 1u : 0u);
                }
                qstr += fmt("%d:%d:%d:%s", M(tq, TqPriorityTag) ? 1 : 0,
                            M(tq, TqReinfTag) ? 1 : 0,
                            M(tq, TqTeamTag) ? 1 : 0, wostr.c_str());
            }
            if (qstr.empty()) qstr = "-";
            row(fmt("BUILDTEAM %s %d %s", names[i], prio, qstr.c_str()));
        }
    }
    (ai->*access_member(AiClearQueuesTag()))();
}

static void run_eval_cond(oracle_wp51::World& w) {
    setup_world(w, true);
    Player* p1 = ThePlayerList->getNthPlayer(1);
    AIPlayer* ai = M(p1, PtAiTag);

    const char* protos[] = {"EC_Empty", "EC_Missing", "EC_True", "EC_False", "EC_Easy"};
    const GameDifficulty diffs[] = {DIFFICULTY_EASY, DIFFICULTY_NORMAL, DIFFICULTY_HARD};
    for (int d = 0; d < 3; ++d) {
        ai->setAIDifficulty(diffs[d]);
        for (const char* pname : protos) {
            TeamPrototype* pr = TheTeamFactory->findTeamPrototype(AsciiString(pname));
            // Reset cache flag to evaluate cleanly under new difficulty only for EC_Easy test
            M(pr, TpAlwaysFalseTag) = FALSE;
            Bool r1 = pr->evaluateProductionCondition();
            Bool af = M(pr, TpAlwaysFalseTag);
            Bool r2 = pr->evaluateProductionCondition();
            row(fmt("EVALCOND %d %s %d %d %d", d, pname, r1 ? 1 : 0, af ? 1 : 0, r2 ? 1 : 0));
        }
    }

    ai->setAIDifficulty(DIFFICULTY_NORMAL);
    TeamPrototype* pdel = TheTeamFactory->findTeamPrototype(AsciiString("EC_Delay3"));
    UnsignedInt frames[] = {10, 11, 50, 99, 100, 101, 190, 191};
    for (UnsignedInt f : frames) {
        TheGameLogic->*access_member(GlFrameTag()) = f;
        Bool r = pdel->evaluateProductionCondition();
        row(fmt("EVALDELAY %u %d", f, r ? 1 : 0));
    }
}

static void run_try_to_recruit(oracle_wp51::World& w) {
    setup_world(w, true);
    Player* p1 = ThePlayerList->getNthPlayer(1);
    Player* p2 = ThePlayerList->getNthPlayer(2);

    TeamPrototype* p_ok = TheTeamFactory->findTeamPrototype(AsciiString("RecSrcOk"));
    M(p_ok, TpInfoTag).m_isAIRecruitable = TRUE;
    TeamPrototype* p_high = TheTeamFactory->findTeamPrototype(AsciiString("RecSrcHighPrio"));
    M(p_high, TpInfoTag).m_isAIRecruitable = TRUE;

    Team* t_target = TheTeamFactory->createTeam(AsciiString("RecruitTarget"));
    Team* t_ok = TheTeamFactory->createTeam(AsciiString("RecSrcOk"));
    t_ok->setActive();
    Team* t_ok_inactive = TheTeamFactory->createInactiveTeam(AsciiString("RecSrcOk"));
    Team* t_high = TheTeamFactory->createTeam(AsciiString("RecSrcHighPrio"));
    t_high->setActive();
    Team* t_noflag = TheTeamFactory->createTeam(AsciiString("RecSrcNoFlag"));
    t_noflag->setActive();
    Team* t_override_on = TheTeamFactory->createTeam(AsciiString("RecSrcNoFlag"));
    t_override_on->setActive();
    t_override_on->setRecruitable(true);

    const ThingTemplate* tb = TheThingFactory->findTemplate("UnitB");
    const ThingTemplate* ta = TheThingFactory->findTemplate("UnitA");
    Coord3D home = {0.0f, 0.0f, 0.0f};

    // Case 1: Only default-team unit VERY FAR away (distance 5000 > maxDist 100) -> quirk MUST accept!
    Object* u_def_far = w.spawn_object("UnitBVar", p1->getDefaultTeam(), {5000.0f, 0.0f, 0.0f});
    Object* u_enemy_close = w.spawn_object("UnitB", p2->getDefaultTeam(), {5.0f, 0.0f, 0.0f});
    Object* u_high_close = w.spawn_object("UnitB", t_high, {10.0f, 0.0f, 0.0f});
    Object* u_inact_close = w.spawn_object("UnitB", t_ok_inactive, {12.0f, 0.0f, 0.0f});
    Object* u_noflag_close = w.spawn_object("UnitB", t_noflag, {14.0f, 0.0f, 0.0f});
    Object* u_held = w.spawn_object("UnitB", t_ok, {15.0f, 0.0f, 0.0f});
    u_held->setDisabled(DISABLED_HELD);
    Object* picked1 = t_target->tryToRecruit(tb, &home, 100.0f);
    row(fmt("RECRUIT def_far %u %u", static_cast<unsigned>(u_def_far->getID()),
            picked1 ? static_cast<unsigned>(picked1->getID()) : 0u));

    // Case 2: Non-default recruitable team unit outside maxDist=100 must NOT be picked, so def_far wins
    Object* u_ok_far = w.spawn_object("UnitB", t_ok, {250.0f, 0.0f, 0.0f});
    Object* picked2 = t_target->tryToRecruit(tb, &home, 100.0f);
    row(fmt("RECRUIT nondef_out_of_range %u %u", static_cast<unsigned>(u_def_far->getID()),
            picked2 ? static_cast<unsigned>(picked2->getID()) : 0u));

    // Case 3: With maxDist=300, closer non-default recruitable unit beats def_far
    Object* picked3 = t_target->tryToRecruit(tb, &home, 300.0f);
    row(fmt("RECRUIT nondef_in_range %u %u", static_cast<unsigned>(u_ok_far->getID()),
            picked3 ? static_cast<unsigned>(picked3->getID()) : 0u));

    // Case 4: Team override recruitable=true closer than u_ok_far beats u_ok_far
    Object* u_ov_on = w.spawn_object("UnitB", t_override_on, {80.0f, 0.0f, 0.0f});
    Object* picked4 = t_target->tryToRecruit(tb, &home, 300.0f);
    row(fmt("RECRUIT override_on %u %u", static_cast<unsigned>(u_ov_on->getID()),
            picked4 ? static_cast<unsigned>(picked4->getID()) : 0u));

    // Case 5: Team override recruitable=false stops t_ok
    t_ok->setRecruitable(false);
    t_override_on->setRecruitable(false);
    Object* picked5 = t_target->tryToRecruit(ta, &home, 300.0f);
    row(fmt("RECRUIT none_ta %u", picked5 ? static_cast<unsigned>(picked5->getID()) : 0u));

    (void)u_enemy_close;
    (void)u_high_close;
    (void)u_inact_close;
    (void)u_noflag_close;
}

static void run_queue_dozer_and_truck(oracle_wp51::World& w) {
    // queueDozer test
    {
        setup_world(w, true);
        Player* p1 = ThePlayerList->getNthPlayer(1);
        AIPlayer* ai = M(p1, PtAiTag);
        M(p1, PtCanBuildUnitsTag) = FALSE;
        p1->getMoney()->deposit(5000, FALSE, FALSE);
        Object* fa = w.spawn_object("FactA", p1->getDefaultTeam(), {100.0f, 100.0f, 0.0f});
        BuildListInfo* bl_a = newInstance(BuildListInfo);
        bl_a->setTemplateName(AsciiString("FactA"));
        bl_a->setObjectID(fa->getID());
        M(p1, PtBuildListTag) = bl_a;

        (ai->*access_member(AiQueueDozerTag()))();
        Bool can_after1 = M(p1, PtCanBuildUnitsTag);
        // Second call blocked by dozerInQueue
        (ai->*access_member(AiQueueDozerTag()))();
        int q_count = 0;
        std::string first_thing = "-";
        for (DLINK_ITERATOR<TeamInQueue> it = (ai->*access_member(AiIterBuildTag()))(); !it.done(); it.advance()) {
            ++q_count;
            WorkOrder* wo = M(it.cur(), TqOrdersTag);
            if (wo && M(wo, WoThingTag)) first_thing = M(wo, WoThingTag)->getName().str();
        }
        row(fmt("QDOZ %d %d %s", can_after1 ? 1 : 0, q_count, first_thing.c_str()));
        (ai->*access_member(AiClearQueuesTag()))();
    }

    // queueSupplyTruck test
    {
        setup_world(w, true);
        Player* p1 = ThePlayerList->getNthPlayer(1);
        AIPlayer* ai = M(p1, PtAiTag);
        M(p1, PtCanBuildUnitsTag) = TRUE;
        p1->getMoney()->deposit(5000, FALSE, FALSE);

        Object* fa = w.spawn_object("FactA", p1->getDefaultTeam(), {100.0f, 100.0f, 0.0f});
        Object* sc = w.spawn_object("SupplyCtr", p1->getDefaultTeam(), {200.0f, 100.0f, 0.0f});
        w.spawn_object("Warehouse", ThePlayerList->getNeutralPlayer()->getDefaultTeam(), {230.0f, 100.0f, 0.0f});

        BuildListInfo* bl_a = newInstance(BuildListInfo);
        bl_a->setTemplateName(AsciiString("FactA"));
        bl_a->setObjectID(fa->getID());
        BuildListInfo* bl_sc = newInstance(BuildListInfo);
        bl_sc->setTemplateName(AsciiString("SupplyCtr"));
        bl_sc->setObjectID(sc->getID());
        bl_sc->setSupplyBuilding(TRUE);
        bl_sc->setCurrentGatherers(-1);
        bl_sc->setDesiredGatherers(1);
        bl_a->setNextBuildList(bl_sc);
        M(p1, PtBuildListTag) = bl_a;

        (ai->*access_member(AiQueueTruckTag()))();
        int q1 = 0;
        std::string truck_thing = "-";
        int is_gath = 0;
        for (DLINK_ITERATOR<TeamInQueue> it = (ai->*access_member(AiIterBuildTag()))(); !it.done(); it.advance()) {
            ++q1;
            WorkOrder* wo = M(it.cur(), TqOrdersTag);
            if (wo) {
                if (M(wo, WoThingTag)) truck_thing = M(wo, WoThingTag)->getName().str();
                is_gath = M(wo, WoGatherTag) ? 1 : 0;
            }
        }
        // Second call sees truckInQueue
        (ai->*access_member(AiQueueTruckTag()))();
        int q2 = 0;
        for (DLINK_ITERATOR<TeamInQueue> it = (ai->*access_member(AiIterBuildTag()))(); !it.done(); it.advance()) {
            ++q2;
        }
        row(fmt("QTRUCK %d %d %s %d", q1, q2, truck_thing.c_str(), is_gath));
        (ai->*access_member(AiClearQueuesTag()))();
    }
}

}  // namespace

int main(int argc, char** argv) {
    if (argc != 2) {
        std::fprintf(stderr, "usage: gen_ai_player_teams <out.json>\n");
        return 2;
    }

    oracle_wp51::World w;
    TheTerrainLogic = NEW oracle_wp51::OracleTerrainLogic;
    TheTerrainLogic->init();
    TheWritableGlobalData->m_gravity = -1.0f;
    w.load_ini(fixture_ini());
    w.mem_ini.parse_blocks(commands_ini(), INI_LOAD_OVERWRITE, [](INI* ini, const char* tok) {
        if (std::strcmp(tok, "CommandButton") == 0) {
            INI::parseCommandButtonDefinition(ini);
        } else if (std::strcmp(tok, "CommandSet") == 0) {
            INI::parseCommandSetDefinition(ini);
        }
    });
    TheThingFactory->postProcessLoad();
    w.init_with_neutral_player();

    run_wo_and_tiq_xfer(w);
    run_is_possible_1000(w);
    run_do_team_building(w);
    run_build_specific_team(w);
    run_eval_cond(w);
    run_try_to_recruit(w);
    run_queue_dozer_and_truck(w);

    std::string out = "{\n \"rows\": [";
    for (size_t i = 0; i < g_rows.size(); ++i) {
        out += std::string(i ? "," : "") + "\n  \"" + g_rows[i] + "\"";
    }
    out += "\n ]\n}\n";
    oracle::write_file_atomic(argv[1], out);
    std::printf("wrote %zu rows to %s\n", g_rows.size(), argv[1]);
    return 0;
}
