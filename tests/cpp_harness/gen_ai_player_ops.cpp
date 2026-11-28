// SPDX-License-Identifier: GPL-3.0-or-later
// 3-way C++ differential oracle for remaining AIPlayer (PKT-426) and
// AISkirmishPlayer (PKT-427) methods:
//   getPlayerStructureBounds, getPlayerSuperweaponValue, computeSuperweaponTarget
//   (both base & skirmish), buildAIBaseDefense, buildAIBaseDefenseStructure,
//   calcClosestConstructionZoneLocation, buildSpecificBuildingNearestTeam,
//   isSupplySourceAttacked, isSupplySourceSafe, guardSupplyCenter,
//   buildUpgrade, repairStructure, selectSkillset, updateBridgeRepair,
//   and non-skirmish processBaseBuilding (incl. RebuildHole + replacement dozer).
//
// Build 3 ways and verify byte-identical output:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_ai_player_ops.cpp --opt O0 -o /tmp/gen_ops_o0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_ai_player_ops.cpp --opt O2 -o /tmp/gen_ops_o2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_ai_player_ops.cpp --opt O2 --sanitize -o /tmp/gen_ops_ubsan

#include "oracle_wp51_world.h"

#include "Common/Money.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/SpecialPower.h"
#include "Common/Team.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/Upgrade.h"
#include "Common/WellKnownKeys.h"
#include "GameClient/CampaignManager.h"
#include "GameClient/TerrainVisual.h"
#include "GameLogic/AI.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/AIPlayer.h"
#include "GameLogic/AISkirmishPlayer.h"
#include "GameLogic/Damage.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/LogicRandomValue.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/ActiveBody.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/DozerAIUpdate.h"
#include "GameLogic/Module/ProductionUpdate.h"
#include "GameLogic/Module/RebuildHoleBehavior.h"
#include "GameLogic/Object.h"
#include "GameLogic/ScriptEngine.h"
#include "GameLogic/SidesList.h"
#include "GameLogic/TerrainLogic.h"

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

ACCESS(PtAiTag, Player, AIPlayer*, m_ai)
ACCESS(PtBuildListTag, Player, BuildListInfo*, m_pBuildList)
ACCESS(PtCanBuildBaseTag, Player, Bool, m_canBuildBase)
ACCESS(PtAttackedFrameTag, Player, UnsignedInt, m_attackedFrame)

ACCESS(AiReadyStrTag, AIPlayer, Bool, m_readyToBuildStructure)
ACCESS(AiStrTimerTag, AIPlayer, Int, m_structureTimer)
ACCESS(AiBuildDelayTag, AIPlayer, Int, m_buildDelay)
ACCESS(AiLastBldgTag, AIPlayer, Int, m_frameLastBuildingBuilt)
ACCESS(AiSkillsetTag, AIPlayer, Int, m_skillsetSelector)
ACCESS(AiBaseCenterTag, AIPlayer, Coord3D, m_baseCenter)
ACCESS(AiBaseSetTag, AIPlayer, Bool, m_baseCenterSet)
ACCESS(AiBaseRadiusTag, AIPlayer, Real, m_baseRadius)
struct AiRepairArrTag { using type = ObjectID (AIPlayer::*)[2]; };
template struct MemberAccess<AiRepairArrTag, &AIPlayer::m_structuresToRepair>;
inline AiRepairArrTag::type access_member(AiRepairArrTag);
ACCESS(AiRepairDozerTag, AIPlayer, ObjectID, m_repairDozer)
ACCESS(AiRepairOriginTag, AIPlayer, Coord3D, m_repairDozerOrigin)
ACCESS(AiRepairCntTag, AIPlayer, Int, m_structuresInQueue)
ACCESS(AiRepairQueuedTag, AIPlayer, Bool, m_dozerQueuedForRepair)
ACCESS(AiRepairActiveTag, AIPlayer, Bool, m_dozerIsRepairing)
ACCESS(AiBridgeTimerTag, AIPlayer, Int, m_bridgeTimer)
ACCESS(AiSupplyCheckTag, AIPlayer, UnsignedInt, m_supplySourceAttackCheckFrame)
ACCESS(AiAttackedSupplyTag, AIPlayer, ObjectID, m_attackedSupplyCenter)

ACCESS(SkEnemyTag, AISkirmishPlayer, Player*, m_currentEnemy)
ACCESS(SkCheckEnemyTag, AISkirmishPlayer, UnsignedInt, m_frameToCheckEnemy)
ACCESS(SkFrontCntTag, AISkirmishPlayer, Int, m_curFrontBaseDefense)
ACCESS(SkFlankCntTag, AISkirmishPlayer, Int, m_curFlankBaseDefense)
ACCESS(SkFrontLAngTag, AISkirmishPlayer, Real, m_curFrontLeftDefenseAngle)
ACCESS(SkFrontRAngTag, AISkirmishPlayer, Real, m_curFrontRightDefenseAngle)
ACCESS(SkLFLeftTag, AISkirmishPlayer, Real, m_curLeftFlankLeftDefenseAngle)
ACCESS(SkLFRightTag, AISkirmishPlayer, Real, m_curLeftFlankRightDefenseAngle)
ACCESS(SkRFLeftTag, AISkirmishPlayer, Real, m_curRightFlankLeftDefenseAngle)
ACCESS(SkRFRightTag, AISkirmishPlayer, Real, m_curRightFlankRightDefenseAngle)

struct AiGetStrBoundsTag { using type = void (*)(Region2D*, Int, Bool); };
template struct MemberAccess<AiGetStrBoundsTag, &AIPlayer::getPlayerStructureBounds>;
inline AiGetStrBoundsTag::type access_member(AiGetStrBoundsTag);

struct AiGetSwValTag { using type = Int (*)(Coord3D*, Int, Real, Bool); };
template struct MemberAccess<AiGetSwValTag, &AIPlayer::getPlayerSuperweaponValue>;
inline AiGetSwValTag::type access_member(AiGetSwValTag);

struct AiProcessBaseTag { using type = void (AIPlayer::*)(); };
template struct MemberAccess<AiProcessBaseTag, &AIPlayer::processBaseBuilding>;
inline AiProcessBaseTag::type access_member(AiProcessBaseTag);

struct AiUpdateBridgeTag { using type = void (AIPlayer::*)(); };
template struct MemberAccess<AiUpdateBridgeTag, &AIPlayer::updateBridgeRepair>;
inline AiUpdateBridgeTag::type access_member(AiUpdateBridgeTag);

ACCESS(TlWaypointHeadTag, TerrainLogic, Waypoint*, m_waypointListHead)
ACCESS(SeCallingTeamTag, ScriptEngine, Team*, m_callingTeam)
ACCESS(SeCurrentPlayerTag, ScriptEngine, Player*, m_currentPlayer)
ACCESS(SeSkirmishHumanTag, ScriptEngine, Player*, m_skirmishHumanPlayer)
ACCESS(AiNextGroupTag, AI, UnsignedInt, m_nextGroupID)
ACCESS(AbLastDamageTsTag, ActiveBody, UnsignedInt, m_lastDamageTimestamp)
ACCESS(AbLastDamageInfoTag, ActiveBody, DamageInfo, m_lastDamageInfo)
ACCESS(GlFrameTag, GameLogic, UnsignedInt, m_frame)

namespace {

class NullTerrainVisual : public TerrainVisual {
public:
    void getTerrainColorAt(Real, Real, RGBColor*) override {}
    TerrainType* getTerrainTile(Real, Real) override { return nullptr; }
    void enableWaterGrid(Bool) override {}
    void setWaterGridHeightClamps(const WaterHandle*, Real, Real) override {}
    void setWaterAttenuationFactors(const WaterHandle*, Real, Real, Real, Real) override {}
    void setWaterTransform(const WaterHandle*, Real, Real, Real, Real) override {}
    void setWaterTransform(const Matrix3D*) override {}
    void getWaterTransform(const WaterHandle*, Matrix3D*) override {}
    void setWaterGridResolution(const WaterHandle*, Real, Real, Real) override {}
    void getWaterGridResolution(const WaterHandle*, Real*, Real*, Real*) override {}
    void changeWaterHeight(Real, Real, Real) override {}
    void addWaterVelocity(Real, Real, Real, Real) override {}
    Bool getWaterGridHeight(Real, Real, Real*) override { return FALSE; }
    void setTerrainTracksDetail() override {}
    void setShoreLineDetail() override {}
    void addFactionBib(Object*, Bool, Real = 0) override {}
    void removeFactionBib(Object*) override {}
    void addFactionBibDrawable(Drawable*, Bool, Real = 0) override {}
    void removeFactionBibDrawable(Drawable*) override {}
    void removeAllBibs() override {}
    void removeBibHighlighting() override {}
    void removeTreesAndPropsForConstruction(const Coord3D*, const GeometryInfo&, Real) override {}
    void addProp(const ThingTemplate*, const Coord3D*, Real) override {}
    void setRawMapHeight(const ICoord2D*, Int) override {}
    Int getRawMapHeight(const ICoord2D*) override { return 0; }
    void replaceSkyboxTextures(const AsciiString*[NumSkyboxTextures], const AsciiString*[NumSkyboxTextures]) override {}
};

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

static std::string fx(float f) {
    uint32_t u = 0;
    std::memcpy(&u, &f, 4);
    char b[16];
    std::snprintf(b, sizeof(b), "%08x", u);
    return b;
}

static uint64_t g_state = 0x4264279876543210ull;
static uint32_t pick(uint32_t n) {
    return n ? static_cast<uint32_t>(oracle::splitmix64(g_state) % n) : 0;
}

static std::string commands_ini() {
    return
        "CommandButton Btn_UpgAlpha\n"
        "  Command = PLAYER_UPGRADE\n  Upgrade = UpgPlayerAlpha\n"
        "End\n"
        "CommandButton Btn_UnitA\n"
        "  Command = UNIT_BUILD\n  Object = UnitA\n"
        "End\n"
        "CommandSet CmdWithUpgrade\n"
        "  1 = Btn_UnitA\n  2 = Btn_UpgAlpha\n"
        "End\n"
        "CommandSet CmdNoUpgrade\n"
        "  1 = Btn_UnitA\n"
        "End\n";
}

static std::string ops_ini() {
    return
        "Locomotor AptGround\n"
        "  Surfaces = GROUND\n"
        "  Appearance = FOUR_WHEELS\n"
        "  Speed = 30\n  TurnRate = 180\n  Acceleration = 60\n  Braking = 60\n  MinSpeed = 0\n"
        "End\n"
        "Upgrade UpgPlayerAlpha\n"
        "  Type = PLAYER\n  BuildCost = 500\n  BuildTime = 5.0\n"
        "End\n"
        "Upgrade UpgObjBeta\n"
        "  Type = OBJECT\n  BuildCost = 200\n  BuildTime = 2.0\n"
        "End\n"
        "SpecialPower SpNeutron\n"
        "  Enum = SPECIAL_NEUTRON_MISSILE\n"
        "  ReloadTime = 60000\n"
        "End\n"
        "SpecialPower SpSneak\n"
        "  Enum = SPECIAL_SNEAK_ATTACK\n"
        "  ReloadTime = 60000\n"
        "End\n"
        "SpecialPower SpClusterMines\n"
        "  Enum = SPECIAL_CLUSTER_MINES\n"
        "  ReloadTime = 60000\n"
        "End\n"
        "SpecialPower SpNukeMines\n"
        "  Enum = NUKE_SPECIAL_CLUSTER_MINES\n"
        "  ReloadTime = 60000\n"
        "End\n"
        "PlayerTemplate FactionCivilian\n  Side = Civilian\n  PreferredColor = R:100 G:100 B:100\nEnd\n"
        "PlayerTemplate FactionAmerica\n  Side = America\n  PreferredColor = R:10 G:20 B:255\nEnd\n"
        "Object UnitA\n"
        "  Side = America\n  KindOf = PRELOAD SELECTABLE VEHICLE\n"
        "  BuildCost = 800\n  BuildTime = 1.0\n"
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 6\n  GeometryHeight = 8\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "  Locomotor = SET_NORMAL AptGround\n"
        "End\n"
        "Object InfA\n"
        "  Side = America\n  KindOf = PRELOAD SELECTABLE INFANTRY\n"
        "  BuildCost = 300\n  BuildTime = 0.5\n"
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 4\n  GeometryHeight = 6\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "  Locomotor = SET_NORMAL AptGround\n"
        "End\n"
        "Object AirA\n"
        "  Side = America\n  KindOf = PRELOAD SELECTABLE VEHICLE AIRCRAFT\n"
        "  BuildCost = 1500\n  BuildTime = 1.0\n"
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 8\n  GeometryHeight = 8\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "  Locomotor = SET_NORMAL AptGround\n"
        "End\n"
        "Object DozerA\n"
        "  Side = America\n  KindOf = PRELOAD SELECTABLE VEHICLE DOZER\n"
        "  BuildCost = 1000\n  BuildTime = 1.0\n"
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 7\n  GeometryHeight = 8\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n"
        "  Behavior = DozerAIUpdate ModuleTag_AI\n"
        "    RepairHealthPercentPerSecond = 50%\n    BoredTime = 5000\n    BoredRange = 150\n"
        "  End\n"
        "  Locomotor = SET_NORMAL AptGround\n"
        "End\n"
        "Object HarvA\n"
        "  Side = America\n  KindOf = PRELOAD SELECTABLE VEHICLE HARVESTER\n"
        "  BuildCost = 600\n  BuildTime = 1.0\n"
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 6\n  GeometryHeight = 8\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "  Locomotor = SET_NORMAL AptGround\n"
        "End\n"
        "Object CmdCenter\n"
        "  Side = America\n  KindOf = PRELOAD STRUCTURE COMMANDCENTER\n"
        "  BuildCost = 2000\n  BuildTime = 1.0\n"
        "  Geometry = BOX\n  GeometryMajorRadius = 25\n  GeometryMinorRadius = 25\n  GeometryHeight = 20\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 2000\n    InitialHealth = 2000\n  End\n"
        "End\n"
        "Object Superwep\n"
        "  Side = America\n  KindOf = PRELOAD STRUCTURE FS_SUPERWEAPON\n"
        "  BuildCost = 5000\n  BuildTime = 1.0\n"
        "  Geometry = BOX\n  GeometryMajorRadius = 25\n  GeometryMinorRadius = 25\n  GeometryHeight = 20\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 2000\n    InitialHealth = 2000\n  End\n"
        "End\n"
        "Object BaseDef\n"
        "  Side = America\n  KindOf = PRELOAD STRUCTURE FS_BASE_DEFENSE\n"
        "  BuildCost = 900\n  BuildTime = 1.0\n"
        "  Geometry = BOX\n  GeometryMajorRadius = 12\n  GeometryMinorRadius = 12\n  GeometryHeight = 15\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 600\n    InitialHealth = 600\n  End\n"
        "End\n"
        "Object TechDef\n"
        "  Side = America\n  KindOf = PRELOAD STRUCTURE TECH_BASE_DEFENSE\n"
        "  BuildCost = 700\n  BuildTime = 1.0\n"
        "  Geometry = BOX\n  GeometryMajorRadius = 12\n  GeometryMinorRadius = 12\n  GeometryHeight = 15\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 600\n    InitialHealth = 600\n  End\n"
        "End\n"
        "Object ConservBldg\n"
        "  Side = America\n  KindOf = PRELOAD STRUCTURE CONSERVATIVE_BUILDING\n"
        "  BuildCost = 400\n  BuildTime = 1.0\n"
        "  Geometry = BOX\n  GeometryMajorRadius = 15\n  GeometryMinorRadius = 15\n  GeometryHeight = 15\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 600\n    InitialHealth = 600\n  End\n"
        "End\n"
        "Object CashGen\n"
        "  Side = America\n  KindOf = PRELOAD STRUCTURE CASH_GENERATOR FS_SUPPLY_CENTER\n"
        "  BuildCost = 1500\n  BuildTime = 1.0\n"
        "  Geometry = BOX\n  GeometryMajorRadius = 20\n  GeometryMinorRadius = 20\n  GeometryHeight = 15\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 1000\n  End\n"
        "End\n"
        "Object Warehouse\n"
        "  Side = America\n  KindOf = PRELOAD STRUCTURE SUPPLY_SOURCE\n"
        "  Geometry = BOX\n  GeometryMajorRadius = 20\n  GeometryMinorRadius = 20\n  GeometryHeight = 15\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 1000\n  End\n"
        "  Behavior = SupplyWarehouseDockUpdate ModuleTag_Dock\n    NumberApproachPositions = 2\n    StartingBoxes = 20\n  End\n"
        "End\n"
        "Object FactUp\n"
        "  Side = America\n  KindOf = PRELOAD STRUCTURE FS_FACTORY FS_WARFACTORY\n"
        "  CommandSet = CmdWithUpgrade\n"
        "  BuildCost = 2000\n  BuildTime = 1.0\n"
        "  Geometry = BOX\n  GeometryMajorRadius = 20\n  GeometryMinorRadius = 20\n  GeometryHeight = 20\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 1000\n  End\n"
        "  Behavior = ProductionUpdate ModuleTag_Prod\n  End\n"
        "End\n"
        "Object FactPlain\n"
        "  Side = America\n  KindOf = PRELOAD STRUCTURE FS_FACTORY FS_WARFACTORY\n"
        "  CommandSet = CmdNoUpgrade\n"
        "  BuildCost = 1500\n  BuildTime = 1.0\n"
        "  Geometry = BOX\n  GeometryMajorRadius = 20\n  GeometryMinorRadius = 20\n  GeometryHeight = 20\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 1000\n  End\n"
        "  Behavior = ProductionUpdate ModuleTag_Prod\n  End\n"
        "End\n"
        "Object RebuildHole\n"
        "  Side = America\n  KindOf = PRELOAD STRUCTURE REBUILD_HOLE\n"
        "  Geometry = BOX\n  GeometryMajorRadius = 15\n  GeometryMinorRadius = 15\n  GeometryHeight = 5\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n"
        "  Behavior = RebuildHoleBehavior ModuleTag_Hole\n    WorkerRespawnDelay = 5000\n  End\n"
        "End\n";
}

static void reset_world(oracle_wp51::World& w, bool skirmish) {
    TheSidesList->emptySides();
    TheSidesList->emptyTeams();
    TheTeamFactory->reset();
    M(TheTerrainLogic, TlWaypointHeadTag) = nullptr;

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
    {
        Dict d;
        d.setAsciiString(NAMEKEY("playerName"), AsciiString("AiPlayer1"));
        d.setAsciiString(NAMEKEY("playerFaction"), AsciiString("FactionAmerica"));
        d.setBool(NAMEKEY("playerIsHuman"), FALSE);
        d.setInt(NAMEKEY("skirmishDifficulty"), DIFFICULTY_NORMAL);
        TheSidesList->addSide(&d);
        Dict td;
        td.setAsciiString(NAMEKEY("teamName"), AsciiString("teamAiPlayer1"));
        td.setAsciiString(NAMEKEY("teamOwner"), AsciiString("AiPlayer1"));
        td.setBool(NAMEKEY("teamIsSingleton"), TRUE);
        TheSidesList->addTeam(&td);
    }
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
    {
        Dict td;
        td.setAsciiString(NAMEKEY("teamName"), AsciiString("GuardTeam"));
        td.setAsciiString(NAMEKEY("teamOwner"), AsciiString("AiPlayer1"));
        td.setBool(NAMEKEY("teamIsSingleton"), TRUE);
        TheSidesList->addTeam(&td);
    }

    ThePlayerList->newGame();
    TheScriptEngine->newMap();
    if (skirmish) {
        ThePlayerList->getNthPlayer(1)->setPlayerType(PLAYER_COMPUTER, TRUE);
    }
    for (Object* o = TheGameLogic->getFirstObject(); o;) {
        Object* nxt = o->getNextObject();
        TheGameLogic->destroyObject(o);
        o = nxt;
    }
    w.flush_destroy_list();
    M(TheGameLogic, GlFrameTag) = 1;
}

static void add_waypoint(WaypointID id, const char* name, Coord3D loc, const char* l1) {
    Waypoint* wp = newInstance(Waypoint)(
        id, AsciiString(name), &loc, AsciiString(l1), AsciiString(""), AsciiString(""), FALSE);
    wp->setNext(M(TheTerrainLogic, TlWaypointHeadTag));
    M(TheTerrainLogic, TlWaypointHeadTag) = wp;
}

static void clear_all_objects(oracle_wp51::World& w) {
    for (Object* o = TheGameLogic->getFirstObject(); o;) {
        Object* nxt = o->getNextObject();
        TheGameLogic->destroyObject(o);
        o = nxt;
    }
    w.flush_destroy_list();
}

static void run_superweapon_and_bounds(oracle_wp51::World& w) {
    reset_world(w, true);
    Player* p1 = ThePlayerList->getNthPlayer(1);
    Player* p2 = ThePlayerList->getNthPlayer(2);
    Team* t2 = p2->getDefaultTeam();
    AIPlayer* ai1 = M(p1, PtAiTag);
    AISkirmishPlayer* sk1 = static_cast<AISkirmishPlayer*>(ai1);
    M(ai1, AiBaseCenterTag) = {200.0f, 200.0f, 0.0f};
    M(ai1, AiBaseSetTag) = TRUE;
    M(ai1, AiBaseRadiusTag) = 120.0f;
    M(sk1, SkEnemyTag) = p2;
    M(sk1, SkCheckEnemyTag) = 999999;

    const char* const pool[] = {
        "CmdCenter", "Superwep", "BaseDef", "TechDef", "ConservBldg",
        "UnitA", "InfA", "AirA", "DozerA", "HarvA", "FactPlain"
    };
    const SpecialPowerTemplate* sp_neutron = TheSpecialPowerStore->findSpecialPowerTemplate(AsciiString("SpNeutron"));
    const SpecialPowerTemplate* sp_sneak = TheSpecialPowerStore->findSpecialPowerTemplate(AsciiString("SpSneak"));
    const SpecialPowerTemplate* sp_cluster = TheSpecialPowerStore->findSpecialPowerTemplate(AsciiString("SpClusterMines"));
    const SpecialPowerTemplate* sp_nuke_mines = TheSpecialPowerStore->findSpecialPowerTemplate(AsciiString("SpNukeMines"));

    for (int s = 0; s < 16; ++s) {
        clear_all_objects(w);
        InitRandom(1000u + static_cast<UnsignedInt>(s) * 17u);
        std::string spawned;
        int n = (s == 0) ? 0 : static_cast<int>(2 + pick(10));
        for (int k = 0; k < n; ++k) {
            const char* name = pool[pick(11)];
            float x = 150.0f + static_cast<float>(pick(700));
            float y = 150.0f + static_cast<float>(pick(700));
            float z = (std::strcmp(name, "AirA") == 0 && (k % 2 == 0)) ? 50.0f : 0.0f;
            Coord3D pos = {x, y, z};
            Object* o = w.spawn_object(name, t2, pos);
            if (!spawned.empty()) spawned += ";";
            spawned += fmt("%s,%s,%s,%s", name, fx(x).c_str(), fx(y).c_str(), fx(z).c_str());
            (void)o;
        }

        Region2D b0, b1;
        (*access_member(AiGetStrBoundsTag()))(&b0, 2, FALSE);
        (*access_member(AiGetStrBoundsTag()))(&b1, 2, TRUE);
        row(fmt("BOUNDS %d %s %s %s %s %s %s %s %s %s",
                s, spawned.empty() ? "-" : spawned.c_str(),
                fx(b0.lo.x).c_str(), fx(b0.lo.y).c_str(), fx(b0.hi.x).c_str(), fx(b0.hi.y).c_str(),
                fx(b1.lo.x).c_str(), fx(b1.lo.y).c_str(), fx(b1.hi.x).c_str(), fx(b1.hi.y).c_str()));

        for (int q = 0; q < 2; ++q) {
            Coord3D c = {
                100.0f + static_cast<float>(pick(800)),
                100.0f + static_cast<float>(pick(800)),
                0.0f
            };
            Real rad = (q == 0) ? 15.0f : (50.0f + static_cast<Real>(pick(200)));
            Int v0 = (*access_member(AiGetSwValTag()))(&c, 2, rad, FALSE);
            Int v1 = (*access_member(AiGetSwValTag()))(&c, 2, rad, TRUE);
            row(fmt("SWVAL %d %s %s %s %d %d", s, fx(c.x).c_str(), fx(c.y).c_str(), fx(rad).c_str(), v0, v1));
        }

        for (int p = 0; p < 2; ++p) {
            const SpecialPowerTemplate* sp = (p == 0) ? sp_neutron : sp_sneak;
            Real wr = (s % 3 == 0) ? 0.5f : (40.0f + static_cast<Real>(pick(120)));
            Coord3D ret = {0.0f, 0.0f, 0.0f};
            Bool ok = ai1->AIPlayer::computeSuperweaponTarget(sp, &ret, 2, wr);
            UnsignedInt crc = GetGameLogicRandomSeedCRC();
            row(fmt("SWBASE %d %d %s %d %s %s %s %u",
                    s, p, fx(wr).c_str(), ok ? 1 : 0,
                    fx(ret.x).c_str(), fx(ret.y).c_str(), fx(ret.z).c_str(), crc));
        }

        M(TheTerrainLogic, TlWaypointHeadTag) = nullptr;
        if (s % 2 == 1) {
            add_waypoint(static_cast<WaypointID>(1), "W1", {400.0f, 200.0f, 0.0f}, "Center1");
            add_waypoint(static_cast<WaypointID>(2), "W2", {200.0f, 450.0f, 0.0f}, "Flank1");
            add_waypoint(static_cast<WaypointID>(3), "W3", {50.0f, 50.0f, 0.0f}, "Backdoor1");
        }
        for (int k = 0; k < 2; ++k) {
            const SpecialPowerTemplate* sp = (k == 0) ? sp_cluster : sp_nuke_mines;
            Coord3D ret = {0.0f, 0.0f, 0.0f};
            Bool ok = sk1->computeSuperweaponTarget(sp, &ret, 2, 75.0f);
            UnsignedInt crc = GetGameLogicRandomSeedCRC();
            row(fmt("SWSKIRMISH %d %d %d %d %s %s %s %u",
                    s, s % 2, k, ok ? 1 : 0,
                    fx(ret.x).c_str(), fx(ret.y).c_str(), fx(ret.z).c_str(), crc));
        }
    }
}

static void run_base_defense(oracle_wp51::World& w) {
    reset_world(w, true);
    Player* p1 = ThePlayerList->getNthPlayer(1);
    Player* p2 = ThePlayerList->getNthPlayer(2);
    AIPlayer* ai1 = M(p1, PtAiTag);
    AISkirmishPlayer* sk1 = static_cast<AISkirmishPlayer*>(ai1);
    M(ai1, AiBaseCenterTag) = {500.0f, 500.0f, 0.0f};
    M(ai1, AiBaseSetTag) = TRUE;
    M(ai1, AiBaseRadiusTag) = 100.0f;
    M(sk1, SkEnemyTag) = p2;
    M(sk1, SkCheckEnemyTag) = 999999;

    add_waypoint(static_cast<WaypointID>(1), "C", {700.0f, 500.0f, 0.0f}, "Center1");
    add_waypoint(static_cast<WaypointID>(2), "F", {500.0f, 700.0f, 0.0f}, "Flank1");
    add_waypoint(static_cast<WaypointID>(3), "B", {300.0f, 500.0f, 0.0f}, "Backdoor1");

    // Place an obstacle near the straight Center1 defense point (500 + 100 + 100? extra=0 => 600,500)
    // so the angle sweep steps left/right!
    w.spawn_object("BaseDef", p1->getDefaultTeam(), {600.0f, 500.0f, 0.0f});

    for (int step = 0; step < 10; ++step) {
        Bool flank = (step >= 4) ? TRUE : FALSE;
        sk1->buildAIBaseDefenseStructure(AsciiString("BaseDef"), flank);
        BuildListInfo* head = M(p1, PtBuildListTag);
        std::string head_str = head
            ? fmt("%s,%s,%s,%s", head->getTemplateName().str(),
                  fx(head->getLocation()->x).c_str(),
                  fx(head->getLocation()->y).c_str(),
                  fx(head->getAngle()).c_str())
            : "-";
        int count = 0;
        for (BuildListInfo* bi = head; bi; bi = bi->getNext()) ++count;
        row(fmt("BASEDEF %d %d %d %d %s %s %s %s %s %s %d %s",
                step, flank ? 1 : 0,
                M(sk1, SkFrontCntTag), M(sk1, SkFlankCntTag),
                fx(M(sk1, SkFrontLAngTag)).c_str(), fx(M(sk1, SkFrontRAngTag)).c_str(),
                fx(M(sk1, SkLFLeftTag)).c_str(), fx(M(sk1, SkLFRightTag)).c_str(),
                fx(M(sk1, SkRFLeftTag)).c_str(), fx(M(sk1, SkRFRightTag)).c_str(),
                count, head_str.c_str()));
    }
}

static void run_cczone_and_nearteam(oracle_wp51::World& w) {
    for (int skirmish = 0; skirmish < 2; ++skirmish) {
        reset_world(w, skirmish != 0);
        Player* p1 = ThePlayerList->getNthPlayer(1);
        AIPlayer* ai1 = M(p1, PtAiTag);
        M(ai1, AiBaseCenterTag) = {200.0f, 200.0f, 0.0f};
        M(ai1, AiBaseSetTag) = TRUE;
        const ThingTemplate* bldg = TheThingFactory->findTemplate("BaseDef");

        // Case 0: unblocked initial spot -> C++ `valid = false` quirk!
        {
            Coord3D loc = {400.0f, 400.0f, 0.0f};
            Bool ok = ai1->calcClosestConstructionZoneLocation(bldg, &loc);
            row(fmt("CCZONE %d 0 %d %s %s %s", skirmish, ok ? 1 : 0,
                    fx(loc.x).c_str(), fx(loc.y).c_str(), fx(loc.z).c_str()));
        }
        // Case 1: blocked at (400, 400) -> enters wiggle loop!
        w.spawn_object("FactPlain", p1->getDefaultTeam(), {400.0f, 400.0f, 0.0f});
        {
            Coord3D loc = {400.0f, 400.0f, 0.0f};
            Bool ok = ai1->calcClosestConstructionZoneLocation(bldg, &loc);
            row(fmt("CCZONE %d 1 %d %s %s %s", skirmish, ok ? 1 : 0,
                    fx(loc.x).c_str(), fx(loc.y).c_str(), fx(loc.z).c_str()));
        }
        // Case 2: Team unit at (400, 400) -> blocked by FactPlain -> buildSpecificBuildingNearestTeam
        Team* gt = TheTeamFactory->findTeam(AsciiString("GuardTeam"));
        Object* u = w.spawn_object("UnitA", gt, {400.0f, 400.0f, 0.0f});
        (void)u;
        ai1->buildSpecificBuildingNearestTeam(AsciiString("BaseDef"), gt);
        BuildListInfo* head = M(p1, PtBuildListTag);
        row(fmt("NEARTEAM %d %s %s %s %s", skirmish,
                head ? head->getTemplateName().str() : "-",
                head ? fx(head->getLocation()->x).c_str() : "-",
                head ? fx(head->getLocation()->y).c_str() : "-",
                head ? fx(head->getAngle()).c_str() : "-"));
    }
}

static void run_supply_ops(oracle_wp51::World& w) {
    reset_world(w, true);
    Player* p1 = ThePlayerList->getNthPlayer(1);
    Player* p2 = ThePlayerList->getNthPlayer(2);
    AIPlayer* ai1 = M(p1, PtAiTag);
    AISkirmishPlayer* sk1 = static_cast<AISkirmishPlayer*>(ai1);
    M(ai1, AiBaseCenterTag) = {200.0f, 200.0f, 0.0f};
    M(ai1, AiBaseSetTag) = TRUE;
    M(sk1, SkEnemyTag) = p2;
    M(sk1, SkCheckEnemyTag) = 999999;

    // 1) isSupplySourceSafe: no warehouse => true
    row(fmt("SUP_SAFE 0 %d", ai1->isSupplySourceSafe(100) ? 1 : 0));
    Object* wh = w.spawn_object("Warehouse", ThePlayerList->getNthPlayer(0)->getDefaultTeam(), {300.0f, 300.0f, 0.0f});
    (void)wh;
    row(fmt("SUP_SAFE 1 %d", ai1->isSupplySourceSafe(100) ? 1 : 0));
    Object* enemy = w.spawn_object("UnitA", p2->getDefaultTeam(), {310.0f, 300.0f, 0.0f});
    row(fmt("SUP_SAFE 2 %d", ai1->isSupplySourceSafe(100) ? 1 : 0));
    TheGameLogic->destroyObject(enemy);
    w.flush_destroy_list();

    // 2) isSupplySourceAttacked
    Object* harv = w.spawn_object("HarvA", p1->getDefaultTeam(), {250.0f, 250.0f, 0.0f});
    M(TheGameLogic, GlFrameTag) = 0;
    Bool a0 = ai1->isSupplySourceAttacked();
    row(fmt("SUP_ATK 0 %d %u %u", a0 ? 1 : 0, M(ai1, AiSupplyCheckTag),
            static_cast<unsigned>(M(ai1, AiAttackedSupplyTag))));

    // Frame 5 (< checkFrame 10) => false
    M(TheGameLogic, GlFrameTag) = 5;
    Bool a1 = ai1->isSupplySourceAttacked();
    row(fmt("SUP_ATK 1 %d %u %u", a1 ? 1 : 0, M(ai1, AiSupplyCheckTag),
            static_cast<unsigned>(M(ai1, AiAttackedSupplyTag))));

    // Frame 15, damaged with no_effect=TRUE => false
    M(TheGameLogic, GlFrameTag) = 15;
    M(p1, PtAttackedFrameTag) = 14;
    ActiveBody* body = static_cast<ActiveBody*>(harv->getBodyModule());
    M(body, AbLastDamageTsTag) = 14;
    DamageInfo di;
    di.out.m_noEffect = TRUE;
    M(body, AbLastDamageInfoTag) = di;
    Bool a2 = ai1->isSupplySourceAttacked();
    row(fmt("SUP_ATK 2 %d %u %u", a2 ? 1 : 0, M(ai1, AiSupplyCheckTag),
            static_cast<unsigned>(M(ai1, AiAttackedSupplyTag))));

    // Frame 30, damaged with no_effect=FALSE => true!
    M(TheGameLogic, GlFrameTag) = 30;
    M(p1, PtAttackedFrameTag) = 28;
    M(body, AbLastDamageTsTag) = 28;
    di.out.m_noEffect = FALSE;
    M(body, AbLastDamageInfoTag) = di;
    Bool a3 = ai1->isSupplySourceAttacked();
    row(fmt("SUP_ATK 3 %d %u %u %u", a3 ? 1 : 0, M(ai1, AiSupplyCheckTag),
            static_cast<unsigned>(M(ai1, AiAttackedSupplyTag)),
            static_cast<unsigned>(harv->getID())));

    // 3) guardSupplyCenter
    w.spawn_object("CmdCenter", p2->getDefaultTeam(), {700.0f, 300.0f, 0.0f});
    Team* gt = TheTeamFactory->findTeam(AsciiString("GuardTeam"));
    Object* g1 = w.spawn_object("UnitA", gt, {100.0f, 100.0f, 0.0f});
    M(TheScriptEngine, SeCallingTeamTag) = gt;
    M(TheScriptEngine, SeCurrentPlayerTag) = p1;
    M(TheScriptEngine, SeSkirmishHumanTag) = p2;
    ai1->guardSupplyCenter(gt, 100);
    M(TheScriptEngine, SeCallingTeamTag) = nullptr;
    M(TheScriptEngine, SeCurrentPlayerTag) = nullptr;
    AIUpdateInterface* gai = g1->getAIUpdateInterface();
    row(fmt("GUARD %u %u %d", M(ai1, AiSupplyCheckTag), M(TheAI, AiNextGroupTag),
            gai ? static_cast<int>(gai->getLastCommandSource()) : -1));
}

static void run_upgrade_skillset_repair(oracle_wp51::World& w) {
    reset_world(w, false);
    Player* p1 = ThePlayerList->getNthPlayer(1);
    AIPlayer* ai1 = M(p1, PtAiTag);
    M(p1, PtCanBuildBaseTag) = TRUE;

    ai1->selectSkillset(3);
    row(fmt("SKILLSET %d", M(ai1, AiSkillsetTag)));

    // buildUpgrade tests
    p1->getMoney()->deposit(1000, FALSE);
    Object* f_plain = w.spawn_object("FactPlain", p1->getDefaultTeam(), {200.0f, 200.0f, 0.0f});
    Object* f_up = w.spawn_object("FactUp", p1->getDefaultTeam(), {300.0f, 200.0f, 0.0f});
    BuildListInfo* bl0 = newInstance(BuildListInfo);
    bl0->setTemplateName(AsciiString("FactPlain"));
    bl0->setObjectID(f_plain->getID());
    BuildListInfo* bl1 = newInstance(BuildListInfo);
    bl1->setTemplateName(AsciiString("FactUp"));
    bl1->setObjectID(f_up->getID());
    bl0->setNextBuildList(bl1);
    M(p1, PtBuildListTag) = bl0;

    // Missing upgrade, object upgrade, then valid upgrade
    ai1->buildUpgrade(AsciiString("NoSuchUpg"));
    ai1->buildUpgrade(AsciiString("UpgObjBeta"));
    UnsignedInt money_before = p1->getMoney()->countMoney();
    ai1->buildUpgrade(AsciiString("UpgPlayerAlpha"));
    UnsignedInt money_after = p1->getMoney()->countMoney();
    ProductionUpdateInterface* pu = f_up->getProductionUpdateInterface();
    row(fmt("UPGRADE %u %u %u", money_before, money_after,
            pu ? pu->getProductionCount() : 99u));

    // repairStructure + updateBridgeRepair tests
    Object* target_pristine = w.spawn_object("CmdCenter", p1->getDefaultTeam(), {400.0f, 400.0f, 0.0f});
    Object* target_dmg = w.spawn_object("BaseDef", p1->getDefaultTeam(), {450.0f, 450.0f, 0.0f});
    DamageInfo dmg;
    dmg.in.m_damageType = DAMAGE_EXPLOSION;
    dmg.in.m_deathType = DEATH_NORMAL;
    dmg.in.m_amount = 450.0f;
    target_dmg->attemptDamage(&dmg);
    ai1->repairStructure(static_cast<ObjectID>(99999));
    ai1->repairStructure(target_pristine->getID());
    ai1->repairStructure(target_dmg->getID());
    ai1->repairStructure(target_dmg->getID()); // duplicate ignored
    row(fmt("REPAIRQ %d %u %u", M(ai1, AiRepairCntTag),
            static_cast<unsigned>(M(ai1, AiRepairArrTag)[0]),
            static_cast<unsigned>(target_dmg->getID())));

    // Step updateBridgeRepair: first call has timer=0 => becomes 30, finds no dozer => queues dozer!
    (ai1->*access_member(AiUpdateBridgeTag()))();
    row(fmt("BREP 0 %d %u %d %d", M(ai1, AiBridgeTimerTag),
            static_cast<unsigned>(M(ai1, AiRepairDozerTag)),
            M(ai1, AiRepairQueuedTag) ? 1 : 0, M(ai1, AiRepairActiveTag) ? 1 : 0));

    // Reset timer to 1, spawn a dozer, clear queued flag, step => acquires dozer & orders repair!
    M(ai1, AiRepairQueuedTag) = FALSE;
    M(ai1, AiBridgeTimerTag) = 1;
    Object* doz = w.spawn_object("DozerA", p1->getDefaultTeam(), {220.0f, 220.0f, 0.0f});
    (ai1->*access_member(AiUpdateBridgeTag()))();
    row(fmt("BREP 1 %d %u %u %d %s %s", M(ai1, AiBridgeTimerTag),
            static_cast<unsigned>(M(ai1, AiRepairDozerTag)),
            static_cast<unsigned>(doz->getID()),
            M(ai1, AiRepairActiveTag) ? 1 : 0,
            fx(M(ai1, AiRepairOriginTag).x).c_str(),
            fx(M(ai1, AiRepairOriginTag).y).c_str()));

    // Heal target to pristine, finish dozer task, set base_center_set, step => goes home & dequeues!
    target_dmg->healCompletely();
    M(ai1, AiBaseCenterTag) = {180.0f, 180.0f, 0.0f};
    M(ai1, AiBaseSetTag) = TRUE;
    M(ai1, AiBridgeTimerTag) = 1;
    (ai1->*access_member(AiUpdateBridgeTag()))();
    row(fmt("BREP 2 %d %d %u", M(ai1, AiRepairCntTag), M(ai1, AiRepairActiveTag) ? 1 : 0,
            static_cast<unsigned>(M(ai1, AiRepairDozerTag))));
}

static void run_nonskirmish_base(oracle_wp51::World& w) {
    reset_world(w, false);
    Player* p1 = ThePlayerList->getNthPlayer(1);
    AIPlayer* ai1 = M(p1, PtAiTag);
    p1->getMoney()->deposit(5000, FALSE);

    // Spawn an existing building and a destroyed building with RebuildHole, plus an unbuilt entry
    Object* hole = w.spawn_object("RebuildHole", p1->getDefaultTeam(), {300.0f, 300.0f, 0.0f});
    ObjectID old_spawner_id = static_cast<ObjectID>(777);
    if (RebuildHoleBehaviorInterface* rhbi = RebuildHoleBehavior::getRebuildHoleBehaviorInterfaceFromObject(hole)) {
        rhbi->startRebuildProcess(TheThingFactory->findTemplate("BaseDef"), old_spawner_id);
    }

    BuildListInfo* bl_hole = newInstance(BuildListInfo);
    bl_hole->setTemplateName(AsciiString("BaseDef"));
    bl_hole->setObjectID(old_spawner_id);
    bl_hole->setNumRebuilds(0); // not buildable directly, only tests hole adoption

    BuildListInfo* bl_new = newInstance(BuildListInfo);
    bl_new->setTemplateName(AsciiString("FactPlain"));
    bl_new->setLocation({500.0f, 500.0f, 0.0f});
    bl_new->setNumRebuilds(2);
    bl_hole->setNextBuildList(bl_new);
    M(p1, PtBuildListTag) = bl_hole;

    w.spawn_object("DozerA", p1->getDefaultTeam(), {480.0f, 500.0f, 0.0f});
    M(ai1, AiReadyStrTag) = TRUE;
    M(TheGameLogic, GlFrameTag) = 50;
    (ai1->*access_member(AiProcessBaseTag()))();

    row(fmt("NSBASE %u %u %u %u %d %d %d",
            static_cast<unsigned>(bl_hole->getObjectID()),
            static_cast<unsigned>(hole->getID()),
            static_cast<unsigned>(bl_new->getObjectID()),
            bl_new->getNumRebuilds(),
            M(ai1, AiReadyStrTag) ? 1 : 0,
            M(ai1, AiStrTimerTag),
            M(ai1, AiLastBldgTag)));
}

}  // namespace

int main(int argc, char** argv) {
    if (argc != 2) {
        std::fprintf(stderr, "usage: gen_ai_player_ops <out.json>\n");
        return 2;
    }

    oracle_wp51::World w;
    TheCampaignManager = NEW CampaignManager;
    TheTerrainVisual = new NullTerrainVisual;
    TheTerrainVisual->init();
    TheWritableGlobalData->m_gravity = -1.0f;
    w.load_ini(ops_ini());
    w.mem_ini.parse_blocks(commands_ini(), INI_LOAD_OVERWRITE, [](INI* ini, const char* tok) {
        if (std::strcmp(tok, "CommandButton") == 0) {
            INI::parseCommandButtonDefinition(ini);
        } else if (std::strcmp(tok, "CommandSet") == 0) {
            INI::parseCommandSetDefinition(ini);
        }
    });
    TheThingFactory->postProcessLoad();
    w.init_with_neutral_player();

    run_superweapon_and_bounds(w);
    run_base_defense(w);
    run_cczone_and_nearteam(w);
    run_supply_ops(w);
    run_upgrade_skillset_repair(w);
    run_nonskirmish_base(w);

    std::string out = "{\n \"rows\": [";
    for (size_t i = 0; i < g_rows.size(); ++i) {
        out += std::string(i ? "," : "") + "\n  \"" + g_rows[i] + "\"";
    }
    out += "\n ]\n}\n";
    oracle::write_file_atomic(argv[1], out);
    std::printf("wrote %zu rows to %s\n", g_rows.size(), argv[1]);
    return 0;
}
