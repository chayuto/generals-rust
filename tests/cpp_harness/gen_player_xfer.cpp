// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for `Player::xfer` and `Player::crc` with the snapshots they call
// (upgrades, AI player, resource gathering, tunnel tracker, squads, relation maps,
// score keeper), over varied player states on the unmodified reference. Each vector
// is the state, the save bytes, the CRC, and the save bytes again after loading
// them into a cleared player (crates/zh_logic/tests/golden_player_xfer.rs).
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_player_xfer.cpp -o /tmp/gen
//   (also with --opt O0 and with --opt O2 --sanitize)
//   /tmp/gen docs/golden_vectors/PlayerXfer.json
//
// Not covered: team prototypes, default team and build lists from maps
// (gen_players_teams.cpp), tunnels with contained objects, AI team queues
// (gen_ai_player_teams.cpp).

#include "oracle_wp51_world.h"

#include "Common/Money.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/ResourceGatheringManager.h"
#include "Common/Science.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/TunnelTracker.h"
#include "Common/Upgrade.h"
#include "GameLogic/AI.h"
#include "GameLogic/AIPlayer.h"
#include "GameLogic/Module/BattlePlanUpdate.h"
#include "GameLogic/Squad.h"

#include <cstdio>
#include <string>
#include <vector>

#define PEEK(Tag, Class, Type, member)                 \
    struct Tag { using type = Type Class::*; };        \
    template struct MemberAccess<Tag, &Class::member>; \
    inline Type Class::* access_member(Tag);
#define M(p, Tag) ((p)->*access_member(Tag()))

typedef Bool AttackedByArray[MAX_PLAYER_COUNT];
typedef Squad* SquadArray[NUM_HOTKEY_SQUADS];
typedef std::map<const ThingTemplate*, Int> CountMap;
typedef std::list<ObjectID> IdList;

PEEK(XAiTag, Player, AIPlayer*, m_ai)
PEEK(XPreorderTag, Player, Bool, m_isPreorder)
PEEK(XDisabledTag, Player, ScienceVec, m_sciencesDisabled)
PEEK(XHiddenTag, Player, ScienceVec, m_sciencesHidden)
PEEK(XSciencesTag, Player, ScienceVec, m_sciences)
PEEK(XRadarTag, Player, Int, m_radarCount)
PEEK(XProofTag, Player, Int, m_disableProofRadarCount)
PEEK(XOffTag, Player, Bool, m_radarDisabled)
PEEK(XDeadTag, Player, Bool, m_isPlayerDead)
PEEK(XUpgradeListTag, Player, Upgrade*, m_upgradeList)
PEEK(XProgressTag, Player, UpgradeMaskType, m_upgradesInProgress)
PEEK(XCompletedTag, Player, UpgradeMaskType, m_upgradesCompleted)
PEEK(XProtosTag, Player, Player::PlayerTeamList, m_playerTeamPrototypes)
PEEK(XBuildTag, Player, BuildListInfo*, m_pBuildList)
PEEK(XTeamTag, Player, Team*, m_defaultTeam)
PEEK(XRankTag, Player, Int, m_rankLevel)
PEEK(XSkillTag, Player, Int, m_skillPoints)
PEEK(XPointsTag, Player, Int, m_sciencePurchasePoints)
PEEK(XUpTag, Player, Int, m_levelUp)
PEEK(XDownTag, Player, Int, m_levelDown)
PEEK(XNameTag, Player, UnicodeString, m_generalName)
PEEK(XPRelTag, Player, PlayerRelationMap*, m_playerRelations)
PEEK(XTRelTag, Player, TeamRelationMap*, m_teamRelations)
PEEK(XBansUnitsTag, Player, Bool, m_canBuildUnits)
PEEK(XBansBaseTag, Player, Bool, m_canBuildBase)
PEEK(XObserverTag, Player, Bool, m_observer)
PEEK(XModTag, Player, Real, m_skillPointsModifier)
PEEK(XListedTag, Player, Bool, m_listInScoreScreen)
PEEK(XAttackedTag, Player, AttackedByArray, m_attackedBy)
PEEK(XBountyTag, Player, Real, m_cashBountyPercent)
PEEK(XChangesTag, Player, std::list<KindOfPercentProductionChange*>,
     m_kindOfPercentProductionChangeList)
PEEK(XTimersTag, Player, std::list<SpecialPowerReadyTimerType>, m_specialPowerReadyTimerList)
PEEK(XSquadsTag, Player, SquadArray, m_squads)
PEEK(XSelectionTag, Player, Squad*, m_currentSelection)
PEEK(XBonusTag, Player, BattlePlanBonuses*, m_battlePlanBonuses)
PEEK(XBombardTag, Player, Int, m_bombardBattlePlans)
PEEK(XHoldTag, Player, Int, m_holdTheLineBattlePlans)
PEEK(XSearchTag, Player, Int, m_searchAndDestroyBattlePlans)
PEEK(XHuntTag, Player, Bool, m_unitsShouldHunt)

PEEK(XTeamTimerTag, AIPlayer, Int, m_teamTimer)
PEEK(XStructTimerTag, AIPlayer, Int, m_structureTimer)
PEEK(XTeamSecondsTag, AIPlayer, Int, m_teamSeconds)
PEEK(XBuildDelayTag, AIPlayer, Int, m_buildDelay)
PEEK(XTeamDelayTag, AIPlayer, Int, m_teamDelay)
PEEK(XFrameBuiltTag, AIPlayer, Int, m_frameLastBuildingBuilt)
PEEK(XDifficultyTag, AIPlayer, GameDifficulty, m_difficulty)
PEEK(XSkillsetTag, AIPlayer, Int, m_skillsetSelector)
PEEK(XRadiusTag, AIPlayer, Real, m_baseRadius)
PEEK(XDozerTag, AIPlayer, ObjectID, m_repairDozer)
PEEK(XBridgeTag, AIPlayer, Int, m_bridgeTimer)
PEEK(XWarehouseTag, AIPlayer, ObjectID, m_curWarehouseID)

PEEK(XWarehousesTag, ResourceGatheringManager, IdList, m_supplyWarehouses)
PEEK(XCentersTag, ResourceGatheringManager, IdList, m_supplyCenters)
PEEK(XTunnelsTag, TunnelTracker, IdList, m_tunnelIDs)

PEEK(XEarnedTag, ScoreKeeper, Int, m_totalMoneyEarned)
PEEK(XScoreTag, ScoreKeeper, Int, m_currentScore)
PEEK(XBuiltTag, ScoreKeeper, CountMap, m_objectsBuilt)
PEEK(XLostTag, ScoreKeeper, CountMap, m_objectsLost)
PEEK(XCapturedTag, ScoreKeeper, CountMap, m_objectsCaptured)

static uint32_t g_state = 0x0ddba11u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}
static Int number() {
    static const Int kEdge[6] = {0, 1, -1, 0x7fffffff, 255, 65536};
    return pick(3) ? static_cast<Int>(pick(100000)) - 500 : kEdge[pick(6)];
}
static Real real() { return static_cast<Real>(static_cast<Int>(pick(4001)) - 2000) / 7.0f; }

static const int kSciences = 4;
static const char* const kScience[kSciences] = {"SCI_A", "SCI_B", "SCI_C", "SCI_D"};
static const char* const kScienceIni =
    "Science SCI_A\n  PrerequisiteSciences = None\nEnd\n"
    "Science SCI_B\n  PrerequisiteSciences = None\nEnd\n"
    "Science SCI_C\n  PrerequisiteSciences = None\nEnd\n"
    "Science SCI_D\n  PrerequisiteSciences = None\nEnd\n";
static const int kUpgrades = 5;
static const char* const kUpgrade[kUpgrades] = {"XU_A", "XU_B", "XU_C", "XU_D", "XU_E"};
static const char* const kUpgradeIni =
    "Upgrade XU_A\n  Type = PLAYER\nEnd\nUpgrade XU_B\n  Type = PLAYER\nEnd\n"
    "Upgrade XU_C\n  Type = OBJECT\nEnd\nUpgrade XU_D\n  Type = PLAYER\nEnd\n"
    "Upgrade XU_E\n  Type = PLAYER\nEnd\n";
static const int kTemplates = 4;
static const char* const kTemplate[kTemplates] = {"X_Hut", "X_Car", "X_Gun", "X_Yard"};
static const char* const kObjectIni =
    "Object X_Hut\n  KindOf = STRUCTURE\nEnd\nObject X_Car\n  KindOf = VEHICLE\nEnd\n"
    "Object X_Gun\n  KindOf = VEHICLE\nEnd\nObject X_Yard\n  KindOf = STRUCTURE\nEnd\n";

static KindOfMaskType kind_mask(int i) {
    KindOfMaskType m;
    if (i == 0 || i == 2) m.set(KINDOF_STRUCTURE);
    if (i == 1 || i == 2) m.set(KINDOF_VEHICLE);
    return m;
}

// `key a b c;` fields of one vector's state, replayed by the Rust test in order.
struct State {
    std::string text;
    void add(const std::string& key, std::initializer_list<long long> values) {
        text += key;
        for (long long v : values) text += " " + std::to_string(v);
        text += ";";
    }
};
static long long bits(Real v) { return static_cast<long long>(oracle::fbits(v)); }

static const ThingTemplate* thing(int i) {
    const ThingTemplate* t = TheThingFactory->findTemplate(kTemplate[i]);
    if (!t) std::exit(1);
    return t;
}
static const UpgradeTemplate* upgrade(int i) {
    const UpgradeTemplate* t = TheUpgradeCenter->findUpgrade(kUpgrade[i]);
    if (!t) std::exit(1);
    return t;
}

// What both sides empty before a state is set, and before a load.
static void clear_player(Player* p, bool skirmish) {
    M(p, XUpgradeListTag) = nullptr;
    M(p, XProgressTag).clear();
    M(p, XCompletedTag).clear();
    M(p, XSciencesTag).clear();
    M(p, XDisabledTag).clear();
    M(p, XHiddenTag).clear();
    M(p, XProtosTag).clear();
    M(p, XTeamTag) = nullptr;
    M(p, XBuildTag) = nullptr;
    M(p, XChangesTag).clear();
    M(p, XTimersTag).clear();
    M(p, XPRelTag) = newInstance(PlayerRelationMap);
    M(p, XTRelTag) = newInstance(TeamRelationMap);
    for (int i = 0; i < NUM_HOTKEY_SQUADS; ++i) M(p, XSquadsTag)[i]->clearSquad();
    M(p, XSelectionTag)->clearSquad();
    M(p, XBonusTag) = nullptr;
    p->getScoreKeeper()->reset(p->getPlayerIndex());
    p->setPlayerType(PLAYER_COMPUTER, skirmish);
    M(p, Wp51PlayerRgmTag) = newInstance(ResourceGatheringManager);
    M(p, Wp51PlayerTunnelTag) = newInstance(TunnelTracker);
    p->getMoney()->init();
}

static void randomize(Player* p, State& s) {
    Money* money = p->getMoney();
    const int deposits = pick(3);
    for (int i = 0; i < deposits; ++i) {
        const UnsignedInt amount = pick(5000);
        money->deposit(amount, FALSE, TRUE);
        s.add("deposit", {amount});
        const UnsignedInt frame = pick(100000);
        TheGameLogic->*access_member(Wp51GLFrameTag()) = frame;
        money->updateIncomeBucket();
        s.add("bucket", {frame});
    }
    auto flag = [&](const char* key, Bool& member) {
        member = pick(2) != 0;
        s.add(key, {member ? 1 : 0});
    };
    auto integer = [&](const char* key, Int& member) {
        member = number();
        s.add(key, {member});
    };
    flag("preorder", M(p, XPreorderTag));
    flag("dead", M(p, XDeadTag));
    flag("radar_off", M(p, XOffTag));
    flag("units", M(p, XBansUnitsTag));
    flag("base", M(p, XBansBaseTag));
    flag("observer", M(p, XObserverTag));
    flag("listed", M(p, XListedTag));
    flag("hunt", M(p, XHuntTag));
    integer("radar", M(p, XRadarTag));
    integer("proof", M(p, XProofTag));
    integer("rank", M(p, XRankTag));
    integer("skill", M(p, XSkillTag));
    integer("points", M(p, XPointsTag));
    integer("up", M(p, XUpTag));
    integer("down", M(p, XDownTag));
    integer("bombard", M(p, XBombardTag));
    integer("hold", M(p, XHoldTag));
    integer("search", M(p, XSearchTag));
    M(p, XModTag) = real();
    s.add("modifier", {bits(M(p, XModTag))});
    p->setCashBounty(real());
    s.add("bounty", {bits(M(p, XBountyTag))});

    ScienceVec* const vecs[3] = {&M(p, XSciencesTag), &M(p, XDisabledTag), &M(p, XHiddenTag)};
    for (int v = 0; v < 3; ++v) {
        const int count = pick(4);
        for (int i = 0; i < count; ++i) {
            const int sci = pick(kSciences);
            vecs[v]->push_back(TheScienceStore->getScienceFromInternalName(kScience[sci]));
            s.add("science", {v, sci});
        }
    }
    const int ops = pick(9);
    for (int i = 0; i < ops; ++i) {
        const int u = pick(kUpgrades);
        if (pick(5) == 0) {
            p->removeUpgrade(upgrade(u));
            s.add("remove_upgrade", {u});
        } else {
            const int status = pick(3);
            p->addUpgrade(upgrade(u), static_cast<UpgradeStatusType>(status));
            s.add("add_upgrade", {u, status});
        }
    }
    Squad** squads = M(p, XSquadsTag);
    const int members = pick(6);
    for (int i = 0; i < members; ++i) {
        const int squad = pick(NUM_HOTKEY_SQUADS + 1);
        const int id = 1 + pick(5000);
        Squad* target = squad == NUM_HOTKEY_SQUADS ? M(p, XSelectionTag) : squads[squad];
        target->addObjectID(static_cast<ObjectID>(id));
        s.add("squad", {squad, id});
    }
    const int general = pick(4);
    UnicodeString name;
    for (int i = 0; i < general; ++i) {
        const WideChar c = static_cast<WideChar>(pick(2) ? 'a' + pick(26) : 0x400 + pick(64));
        name.concat(c);
        s.add("general", {c});
    }
    M(p, XNameTag) = name;
    for (int map = 0; map < 2; ++map) {
        const int count = pick(map ? 14 : 8);
        for (int i = 0; i < count; ++i) {
            const int key = pick(map ? 4000 : 16);
            const int r = pick(3);
            if (map) M(p, XTRelTag)->m_map[key] = static_cast<Relationship>(r);
            else M(p, XPRelTag)->m_map[key] = static_cast<Relationship>(r);
            s.add("relation", {map, key, r});
        }
    }
    for (int i = 0; i < MAX_PLAYER_COUNT; ++i) {
        if (pick(5) == 0) {
            M(p, XAttackedTag)[i] = TRUE;
            s.add("attacked", {i});
        } else {
            M(p, XAttackedTag)[i] = FALSE;
        }
    }
    const int changes = pick(4);
    for (int i = 0; i < changes; ++i) {
        const int mask = pick(3);
        const Real percent = pick(3) ? real() : 0.25f;
        p->addKindOfProductionCostChange(kind_mask(mask), percent);
        s.add("change", {mask, bits(percent)});
    }
    const int timers = pick(4);
    for (int i = 0; i < timers; ++i) {
        SpecialPowerReadyTimerType t;
        t.m_templateID = pick(1000);
        t.m_readyFrame = pick(100000);
        M(p, XTimersTag).push_back(t);
        s.add("timer", {t.m_templateID, t.m_readyFrame});
    }
    if (pick(2)) {
        BattlePlanBonuses* b = newInstance(BattlePlanBonuses);
        b->m_armorScalar = real();
        b->m_sightRangeScalar = real();
        b->m_bombardment = number();
        b->m_holdTheLine = number();
        b->m_searchAndDestroy = number();
        const int valid = pick(3), invalid = pick(3);
        b->m_validKindOf = kind_mask(valid);
        b->m_invalidKindOf = kind_mask(invalid);
        M(p, XBonusTag) = b;
        s.add("bonus", {bits(b->m_armorScalar), bits(b->m_sightRangeScalar), b->m_bombardment,
                        b->m_holdTheLine, b->m_searchAndDestroy, valid, invalid});
    }
    ScoreKeeper* score = p->getScoreKeeper();
    M(score, XEarnedTag) = number();
    s.add("earned", {M(score, XEarnedTag)});
    M(score, XScoreTag) = number();
    s.add("score", {M(score, XScoreTag)});
    CountMap* const maps[3] = {&M(score, XBuiltTag), &M(score, XLostTag),
                               &M(score, XCapturedTag)};
    for (int v = 0; v < 3; ++v) {
        const int count = pick(4);
        for (int i = 0; i < count; ++i) {
            const int t = pick(kTemplates);
            const Int n = number();
            (*maps[v])[thing(t)] = n;
            s.add("count", {v, t, n});
        }
    }
    ResourceGatheringManager* rgm = p->getResourceGatheringManager();
    IdList* const lists[3] = {&M(rgm, XWarehousesTag), &M(rgm, XCentersTag),
                              &M(p->getTunnelSystem(), XTunnelsTag)};
    for (int v = 0; v < 3; ++v) {
        const int count = pick(4);
        for (int i = 0; i < count; ++i) {
            const int id = 1 + pick(5000);
            lists[v]->push_back(static_cast<ObjectID>(id));
            s.add("id", {v, id});
        }
    }

    AIPlayer* ai = M(p, XAiTag);
    auto ai_int = [&](const char* key, Int& member) {
        member = number();
        s.add(key, {member});
    };
    ai_int("team_timer", M(ai, XTeamTimerTag));
    ai_int("structure_timer", M(ai, XStructTimerTag));
    ai_int("team_seconds", M(ai, XTeamSecondsTag));
    ai_int("build_delay", M(ai, XBuildDelayTag));
    ai_int("team_delay", M(ai, XTeamDelayTag));
    ai_int("frame_built", M(ai, XFrameBuiltTag));
    ai_int("skillset", M(ai, XSkillsetTag));
    ai_int("bridge_timer", M(ai, XBridgeTag));
    const int difficulty = pick(3);
    M(ai, XDifficultyTag) = static_cast<GameDifficulty>(difficulty);
    s.add("difficulty", {difficulty});
    M(ai, XRadiusTag) = real();
    s.add("radius", {bits(M(ai, XRadiusTag))});
    M(ai, XDozerTag) = static_cast<ObjectID>(pick(5000));
    s.add("dozer", {M(ai, XDozerTag)});
    M(ai, XWarehouseTag) = static_cast<ObjectID>(pick(5000));
    s.add("warehouse", {M(ai, XWarehouseTag)});
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_player_xfer <out.json>\n");
        return 2;
    }
    oracle_wp51::World w;
    w.load_ini(kScienceIni);
    w.load_ini(kUpgradeIni);
    w.load_ini(kObjectIni);
    TheThingFactory->postProcessLoad();
    w.init_with_neutral_player();

    const std::string tmp = std::string(argv[1]) + ".tmp";
    FILE* out = std::fopen(tmp.c_str(), "w");
    if (!out) return 1;
    const int kTotal = 120;
    std::fprintf(out, "{\"vector_count\":%d,\n\"source_sha256\":{", kTotal);
    const char* const kSources[] = {"Player", "ResourceGatheringManager", "TunnelTracker"};
    for (int i = 0; i < 3; ++i) {
        const std::string path = std::string(
            "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/") + kSources[i] + ".cpp";
        std::fprintf(out, "%s\"%s.cpp\":\"%s\"", i ? "," : "", kSources[i],
                     oracle::sha256_file(path.c_str()).c_str());
    }
    // Upgrades in `UpgradeCenter::firstUpgradeTemplate` order with their mask bit.
    std::fprintf(out, "},\n\"upgrades\":[");
    const char* comma = "";
    for (UpgradeTemplate* t = TheUpgradeCenter->firstUpgradeTemplate(); t;
         t = t->friend_getNext()) {
        const UpgradeMaskType mask = t->getUpgradeMask();
        for (unsigned bit = 0; bit < mask.size(); ++bit) {
            if (mask.test(bit)) {
                std::fprintf(out, "%s\"%s %u\"", comma, t->getUpgradeName().str(), bit);
                comma = ",";
            }
        }
    }
    std::fprintf(out, "],\n\"sciences\":\"");
    for (const char* c = kScienceIni; *c; ++c) {
        if (*c == '\n') std::fputs("\\n", out);
        else std::fputc(*c, out);
    }
    std::fprintf(out, "\",\n\"templates\":[");
    for (int i = 0; i < kTemplates; ++i) {
        std::fprintf(out, "%s\"%s %u\"", i ? "," : "", kTemplate[i],
                     static_cast<unsigned>(thing(i)->getTemplateID()));
    }
    std::fprintf(out, "],\n\"vectors\":[");

    for (int v = 0; v < kTotal; ++v) {
        const int index = 1 + (v % 2);
        const bool skirmish = (v % 4) >= 2;
        Player* p = ThePlayerList->getNthPlayer(index);
        clear_player(p, skirmish);
        State s;
        randomize(p, s);

        const std::string save = oracle_wp51::World::snapshot_save_hex(p);
        const uint32_t crc = oracle_wp51::World::snapshot_crc(p);

        oracle::MemoryXfer xs(XFER_SAVE);
        xs.open("save");
        oracle::call_snapshot_xfer(p, &xs);
        xs.close();
        const std::vector<uint8_t> bytes = xs.bytes;
        clear_player(p, skirmish);
        oracle_wp51::World::snapshot_load_bytes(p, bytes);
        const std::string resave = oracle_wp51::World::snapshot_save_hex(p);

        std::fprintf(out,
                     "%s\n{\"player\":%d,\"skirmish\":%d,\"state\":\"%s\",\"save\":\"%s\","
                     "\"crc\":%u,\"resave\":\"%s\"}",
                     v ? "," : "", index, skirmish ? 1 : 0, s.text.c_str(), save.c_str(), crc,
                     resave.c_str());
    }
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    return std::rename(tmp.c_str(), argv[1]);
}
