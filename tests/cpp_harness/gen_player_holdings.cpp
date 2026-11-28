// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for the state `Player` holds that its teams share (sciences,
// radar, handicap, build bans, cost and time changes, bounty, attacked-by, shared
// special power timers, money and its income records) and for `Player::onPowerBrownOutChange` with the callers
// it has here (energy changes, sabotage and its expiry), run on the unmodified
// reference in a small real world. Each vector is one script of operations on two
// players with two teams each.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_player_holdings.cpp -o /tmp/gen
//   (also with --opt O0 and with --opt O2 --sanitize)
//   /tmp/gen docs/golden_vectors/PlayerHoldings.json
//
// Not covered: handing a team to another player, construction, `Player::xfer`.

#include "oracle_wp51_world.h"

#include "Common/AcademyStats.h"
#include "Common/Energy.h"
#include "Common/Handicap.h"
#include "Common/Money.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/Science.h"
#include "Common/SpecialPower.h"
#include "Common/Team.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/Upgrade.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/SpecialPowerModule.h"
#include "GameLogic/Object.h"

#include <cstdio>
#include <sstream>
#include <string>
#include <vector>

#define PEEK(Tag, Type, member)                                \
    struct Tag { using type = Type Player::*; };               \
    template struct MemberAccess<Tag, &Player::member>;        \
    inline Type Player::* access_member(Tag);

typedef Bool AttackedByArray[MAX_PLAYER_COUNT];

PEEK(HRadarTag, Int, m_radarCount)
PEEK(HProofTag, Int, m_disableProofRadarCount)
PEEK(HOffTag, Bool, m_radarDisabled)
PEEK(HTimersTag, std::list<SpecialPowerReadyTimerType>, m_specialPowerReadyTimerList)
PEEK(HSciencesTag, ScienceVec, m_sciences)
PEEK(HChangesTag, std::list<KindOfPercentProductionChange*>, m_kindOfPercentProductionChangeList)
PEEK(HBansUnitsTag, Bool, m_canBuildUnits)
PEEK(HBansBaseTag, Bool, m_canBuildBase)
PEEK(HAttackedTag, AttackedByArray, m_attackedBy)
PEEK(HAttackedFrameTag, UnsignedInt, m_attackedFrame)
PEEK(HBountyTag, Real, m_cashBountyPercent)
PEEK(HCostTag, ProductionChangeMap, m_productionCostChanges)
PEEK(HTimeTag, ProductionChangeMap, m_productionTimeChanges)

#define PEEK_ACADEMY(Tag, member)                                       \
    struct Tag { using type = UnsignedInt AcademyStats::*; };           \
    template struct MemberAccess<Tag, &AcademyStats::member>;           \
    inline UnsignedInt AcademyStats::* access_member(Tag);

PEEK_ACADEMY(HIncomeFrameTag, m_lastIncomeFrame)
PEEK_ACADEMY(HIncomeGapTag, m_maxFramesBetweenIncome)

static uint32_t g_state = 0x5ca1ab1eu;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}

static const int kSciences = 3;
static const char* const kScience[kSciences] = {"SCI_A", "SCI_B", "SCI_C"};
static const char* const kScienceIni =
    "Science SCI_A\n  PrerequisiteSciences = None\n  SciencePurchasePointCost = 1\n"
    "  IsGrantable = Yes\nEnd\n"
    "Science SCI_B\n  PrerequisiteSciences = None\n  SciencePurchasePointCost = 1\n"
    "  IsGrantable = Yes\nEnd\n"
    "Science SCI_C\n  PrerequisiteSciences = None\n  SciencePurchasePointCost = 1\n"
    "  IsGrantable = No\nEnd\n";

static const int kPowers = 4;
static const char* const kPower[kPowers] = {"SP_Plain", "SP_Sci", "SP_Shared", "SP_SharedSci"};
static const char* const kPowerIni =
    "SpecialPower SP_Plain\n  Enum = SPECIAL_CASH_HACK\n  ReloadTime = 300\nEnd\n"
    "SpecialPower SP_Sci\n  Enum = SPECIAL_DEFECTOR\n  ReloadTime = 900\n"
    "  RequiredScience = SCI_A\nEnd\n"
    "SpecialPower SP_Shared\n  Enum = SPECIAL_CARPET_BOMB\n  ReloadTime = 1500\n"
    "  SharedSyncedTimer = Yes\nEnd\n"
    "SpecialPower SP_SharedSci\n  Enum = SPECIAL_DAISY_CUTTER\n  ReloadTime = 3000\n"
    "  RequiredScience = SCI_B\n  SharedSyncedTimer = Yes\nEnd\n";

static const char* const kUpgradeIni =
    "Upgrade PU_Radar\n  Type = PLAYER\nEnd\nUpgrade PU_Power\n  Type = PLAYER\nEnd\n";
static const char* const kUpgrade[2] = {"PU_Radar", "PU_Power"};

static const int kKinds = 7;
static const char* const kKind[kKinds] = {"G_Plant", "G_Eater",   "G_Gun",  "G_Radar",
                                          "G_Proof", "G_Centre",  "G_Solar"};

static std::string power_module(int tag, const char* power, bool paused) {
    return "  Behavior = SpecialAbility ModuleTag_Power" + std::to_string(tag) +
           "\n    SpecialPowerTemplate = " + power + "\n" +
           (paused ? "    StartsPaused = Yes\n" : "") + "  End\n";
}

static std::string object_ini(const char* name, const char* kinds, int cost, const char* time,
                              int energy, const std::string& body) {
    return std::string("Object ") + name + "\n  KindOf = " + kinds +
           "\n  BuildCost = " + std::to_string(cost) + "\n  BuildTime = " + time +
           "\n  EnergyProduction = " + std::to_string(energy) +
           "\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100.0\n"
           "    InitialHealth = 100.0\n  End\n" +
           body + "End\n";
}

static std::string objects_ini() {
    std::string s;
    s += object_ini("G_Plant", "STRUCTURE", 801, "10.0", 10,
                    "  EnergyBonus = 7\n"
                    "  Behavior = PowerPlantUpdate ModuleTag_Rods\n    RodsExtendTime = 100\n  End\n"
                    "  Behavior = PowerPlantUpgrade ModuleTag_Bonus\n    TriggeredBy = PU_Power\n  End\n");
    s += object_ini("G_Eater", "STRUCTURE POWERED", 1003, "7.0", -6,
                    power_module(0, "SP_Plain", false) + power_module(1, "SP_Shared", false));
    s += object_ini("G_Gun", "VEHICLE POWERED", 455, "5.0", 0,
                    power_module(0, "SP_SharedSci", false) + power_module(1, "SP_Sci", true));
    s += object_ini("G_Radar", "STRUCTURE", 997, "8.0", -4,
                    "  Behavior = RadarUpgrade ModuleTag_Radar\n    TriggeredBy = PU_Radar\n  End\n");
    s += object_ini("G_Proof", "STRUCTURE POWERED", 650, "6.5", -2,
                    "  Behavior = RadarUpgrade ModuleTag_Radar\n    TriggeredBy = PU_Radar\n"
                    "    DisableProof = Yes\n  End\n");
    s += object_ini("G_Centre", "STRUCTURE", 2001, "30.0", 0,
                    power_module(0, "SP_Shared", true) + power_module(1, "SP_Sci", false) +
                        power_module(2, "SP_SharedSci", false));
    s += object_ini("G_Solar", "STRUCTURE POWERED", 333, "3.0", 5, "");
    return s;
}

// The kind sets a cost change names.
static const int kMaskCount = 4;
static const char* const kMaskNames[kMaskCount] = {"STRUCTURE", "VEHICLE", "POWERED",
                                                   "STRUCTURE POWERED"};
static KindOfMaskType kind_mask(int i) {
    KindOfMaskType m;
    if (i == 0 || i == 3) m.set(KINDOF_STRUCTURE);
    if (i == 1) m.set(KINDOF_VEHICLE);
    if (i == 2 || i == 3) m.set(KINDOF_POWERED);
    return m;
}

static const Real kPercents[5] = {-0.1f, 0.25f, -0.33f, 0.0f, -0.0f};
static const Real kHandicaps[4] = {1.0f, 0.75f, 1.5f, 0.4f};
static const char* const kHandicapKeys[4] = {"HANDICAP_BUILDCOST_GENERIC",
                                             "HANDICAP_BUILDCOST_BUILDINGS",
                                             "HANDICAP_BUILDTIME_GENERIC",
                                             "HANDICAP_BUILDTIME_BUILDINGS"};

static std::string escaped(const std::string& text) {
    std::string out;
    for (char c : text) {
        if (c == '\n') out += "\\n";
        else out += c;
    }
    return out;
}

static std::string bits(Real v) {
    uint32_t u;
    std::memcpy(&u, &v, 4);
    char buf[16];
    std::snprintf(buf, sizeof(buf), "%08x", u);
    return buf;
}

struct Trial {
    // Team labels: 0 and 1 the default teams of players 1 and 2, 2 a second team
    // of player 1, 3 one of player 2.
    std::vector<Team*> teams;
    std::vector<Object*> objects;  // nullptr once destroyed
    std::vector<int> kinds;

    Player* player(int team) const { return teams[team]->getControllingPlayer(); }

    int label_of(const Team* t) const {
        for (size_t i = 0; i < teams.size(); ++i)
            if (teams[i] == t) return static_cast<int>(i);
        std::fprintf(stderr, "unknown team\n");
        std::exit(1);
    }
};

static SpecialPowerModuleInterface* power_of(Object* o, int nth) {
    for (BehaviorModule** m = o->getBehaviorModules(); *m; ++m) {
        if (SpecialPowerModuleInterface* sp = (*m)->getSpecialPower()) {
            if (nth-- == 0) return sp;
        }
    }
    std::fprintf(stderr, "no special power module\n");
    std::exit(1);
}

static int power_count(Object* o) {
    int n = 0;
    for (BehaviorModule** m = o->getBehaviorModules(); *m; ++m)
        if ((*m)->getSpecialPower()) ++n;
    return n;
}

static std::string player_row(int p) {
    Player* player = ThePlayerList->getNthPlayer(p);
    Energy* e = player->getEnergy();
    std::ostringstream out;
    out << player->*access_member(HRadarTag()) << ',' << player->*access_member(HProofTag()) << ','
        << (player->*access_member(HOffTag()) ? 1 : 0) << ',' << (player->hasRadar() ? 1 : 0) << ' '
        << e->getProduction() << ',' << e->getConsumption() << ','
        << (e->hasSufficientPower() ? 1 : 0) << ',' << e->getPowerSabotagedTillFrame() << ' ';
    out << 't';
    for (const SpecialPowerReadyTimerType& t : player->*access_member(HTimersTag()))
        out << t.m_templateID << ':' << t.m_readyFrame << ';';
    out << " s";
    for (ScienceType sci : player->*access_member(HSciencesTag())) {
        for (int i = 0; i < kSciences; ++i)
            if (TheScienceStore->getScienceFromInternalName(kScience[i]) == sci) out << i;
    }
    out << " c";
    for (KindOfPercentProductionChange* change : player->*access_member(HChangesTag())) {
        for (int i = 0; i < kMaskCount; ++i)
            if (change->m_kindOf == kind_mask(i)) out << i;
        out << ':' << bits(change->m_percent) << ':' << change->m_ref << ';';
    }
    out << " b" << bits(player->*access_member(HBountyTag())) << ' '
        << (player->*access_member(HBansUnitsTag()) ? 1 : 0)
        << (player->*access_member(HBansBaseTag()) ? 1 : 0) << ' ';
    for (int i = 0; i < 3; ++i) out << (player->getAttackedBy(i) ? '1' : '0');
    out << ',' << player->*access_member(HAttackedFrameTag());
    AcademyStats* academy = player->getAcademyStats();
    out << " m" << player->getMoney()->countMoney() << ','
        << player->getMoney()->getCashPerMinute() << ','
        << academy->*access_member(HIncomeFrameTag()) << ','
        << academy->*access_member(HIncomeGapTag());
    return out.str();
}

static std::string row(const Trial& s) {
    std::ostringstream out;
    for (int p = 0; p < 3; ++p) out << player_row(p) << '|';
    // "cost,time,allowed;" of every template for every team.
    for (size_t t = 0; t < s.teams.size(); ++t) {
        const Player* player = s.player(static_cast<int>(t));
        for (int k = 0; k < kKinds; ++k) {
            const ThingTemplate* tt = TheThingFactory->findTemplate(kKind[k]);
            out << tt->calcCostToBuild(player) << ',' << tt->calcTimeToBuild(player) << ','
                << (player->allowedToBuild(tt) ? 1 : 0) << ';';
        }
        out << '|';
    }
    for (size_t i = 0; i < s.objects.size(); ++i) {
        Object* o = s.objects[i];
        if (!o) continue;
        unsigned mask = 0;
        for (int d = 0; d < DISABLED_COUNT; ++d)
            if (o->isDisabledByType(static_cast<DisabledType>(d))) mask |= 1u << d;
        out << i << ' ' << s.label_of(o->getTeam()) << ' ' << mask << ';';
    }
    return out.str();
}

static void reset_players() {
    std::vector<Object*> all;
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) all.push_back(o);
    for (Object* o : all) TheGameLogic->destroyObject(o);
    TheGameLogic->processDestroyList();
    if (TheGameLogic->getFirstObject()) {
        std::fprintf(stderr, "objects left over\n");
        std::exit(1);
    }
    for (int p = 0; p < 3; ++p) {
        Player* player = ThePlayerList->getNthPlayer(p);
        for (int i = 0; i < 2; ++i)
            player->removeUpgrade(TheUpgradeCenter->findUpgrade(kUpgrade[i]));
        player->getEnergy()->init(player);
        player->*access_member(HRadarTag()) = 0;
        player->*access_member(HProofTag()) = 0;
        player->*access_member(HOffTag()) = FALSE;
        (player->*access_member(HTimersTag())).clear();
        (player->*access_member(HSciencesTag())).clear();
        (player->*access_member(HChangesTag())).clear();
        player->setCanBuildUnits(TRUE);
        player->setCanBuildBase(TRUE);
        player->setCashBounty(0.0f);
        for (int i = 0; i < MAX_PLAYER_COUNT; ++i) (player->*access_member(HAttackedTag()))[i] = FALSE;
        player->*access_member(HAttackedFrameTag()) = 0;
        player->getHandicap()->init();
        player->getMoney()->init();
        player->getAcademyStats()->*access_member(HIncomeFrameTag()) = 0;
        player->getAcademyStats()->*access_member(HIncomeGapTag()) = 0;
        (player->*access_member(HCostTag())).clear();
        (player->*access_member(HTimeTag())).clear();
    }
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_player_holdings <out.json>\n");
        return 2;
    }
    oracle_wp51::World w;
    w.load_ini(kScienceIni);
    w.load_ini(kPowerIni);
    w.load_ini(kUpgradeIni);
    w.load_ini(objects_ini());
    TheThingFactory->postProcessLoad();
    TheWritableGlobalData->m_LowEnergyPenaltyModifier = 0.4f;
    TheWritableGlobalData->m_MinLowEnergyProductionSpeed = 0.2f;
    TheWritableGlobalData->m_MaxLowEnergyProductionSpeed = 0.8f;
    TheWritableGlobalData->m_MultipleFactory = 0.85f;
    w.init_with_neutral_player();
    for (int p = 0; p < 3; ++p) {
        if (ThePlayerList->getNthPlayer(p)->getPlayerIndex() != p) {
            std::fprintf(stderr, "player index\n");
            return 1;
        }
    }

    const std::string tmp = std::string(argv[1]) + ".tmp";
    FILE* out = std::fopen(tmp.c_str(), "w");
    if (!out) return 1;
    const int kTotal = 160;
    std::fprintf(out, "{\"vector_count\":%d,\n\"source_sha256\":{", kTotal);
    const char* const kSources[] = {"RTS/Player", "RTS/Energy", "RTS/Handicap"};
    for (int i = 0; i < 3; ++i) {
        const std::string path =
            std::string("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/") + kSources[i] +
            ".cpp";
        std::fprintf(out, "%s\"%s.cpp\":\"%s\"", i ? "," : "", kSources[i],
                     oracle::sha256_file(path.c_str()).c_str());
    }
    std::string power_ids;
    for (int i = 0; i < kPowers; ++i) {
        power_ids += (i ? "," : "") +
                     std::to_string(TheSpecialPowerStore->findSpecialPowerTemplate(kPower[i])->getID());
    }
    std::fprintf(out,
                 "},\n\"sciences\":\"%s\",\n\"powers\":\"%s\",\n\"upgrades\":\"%s\",\n"
                 "\"objects\":\"%s\",\n\"power_ids\":\"%s\",\n\"vectors\":[",
                 escaped(kScienceIni).c_str(), escaped(kPowerIni).c_str(),
                 escaped(kUpgradeIni).c_str(), escaped(objects_ini()).c_str(), power_ids.c_str());

    UnsignedInt base = 10;
    for (int v = 0; v < kTotal; ++v) {
        reset_players();
        base += 5000;
        UnsignedInt frame = base;
        w.set_frame(frame);

        Trial s;
        Dict dict;
        const std::string tag = "V" + std::to_string(v);
        TheTeamFactory->initTeam((tag + "_A2").c_str(), AsciiString::TheEmptyString, TRUE, &dict);
        TheTeamFactory->initTeam((tag + "_B2").c_str(), AsciiString::TheEmptyString, TRUE, &dict);
        Team* a2 = TheTeamFactory->findTeam((tag + "_A2").c_str());
        Team* b2 = TheTeamFactory->findTeam((tag + "_B2").c_str());
        a2->setControllingPlayer(ThePlayerList->getNthPlayer(1));
        b2->setControllingPlayer(ThePlayerList->getNthPlayer(2));
        s.teams = {w.team_a, w.team_b, a2, b2};

        std::vector<std::string> rows, ops;
        rows.push_back(row(s));
        const int count = 20 + static_cast<int>(pick(30));
        for (int k = 0; k < count; ++k) {
            // Codes: 0 spawn, 1 destroy, 2 change team, 3 grant upgrade, 4 add radar,
            // 5 remove radar, 6 sabotage, 7 frames, 8 hack or unhack, 9 add cost
            // change, 10 remove it, 11 bans, 12 handicap, 13 template cost or time
            // change, 14 grant science, 15 bounty, 16 attacked, 17 is ready,
            // 18 ready frame, 19 percent ready, 20 recharge, 21 power creation,
            // 22 pause, 23 express a player timer, 24 reset a player timer,
            // 25 read a player timer, 26 deposit (tracked or not), 27 withdraw.
            static const int kWeights[] = {0,  0,  0,  0,  0,  1,  2,  2,  3,  3,  4,  5,
                                           6,  6,  7,  7,  7,  8,  8,  9,  9,  10, 11, 12,
                                           13, 14, 14, 15, 16, 17, 17, 17, 18, 18, 19, 19,
                                           20, 20, 21, 22, 22, 23, 24, 24, 25, 26,
                                           26, 26, 27, 27};
            int code = kWeights[pick(sizeof(kWeights) / sizeof(kWeights[0]))];
            if (k < 6) code = 0;
            std::vector<int> alive, powered;
            for (size_t i = 0; i < s.objects.size(); ++i) {
                if (!s.objects[i]) continue;
                alive.push_back(static_cast<int>(i));
                if (power_count(s.objects[i]) > 0) powered.push_back(static_cast<int>(i));
            }
            const bool needs_object = code == 1 || code == 2 || code == 8 || code == 16;
            const bool needs_power = code >= 17 && code <= 22;
            if ((needs_object && alive.empty()) || (needs_power && powered.empty()) ||
                (code == 0 && alive.size() >= 14))
                code = 7;
            const int team = static_cast<int>(pick(4));
            int a = 0, b = 0, c = 0;
            uint32_t ret = 0;
            w.set_frame(frame);
            if (code == 0) {
                a = team;
                b = static_cast<int>(pick(kKinds));
                const ThingTemplate* tt = TheThingFactory->findTemplate(kKind[b]);
                if (!tt) return 1;
                Object* o = TheThingFactory->newObject(tt, s.teams[a]);
                if (!o) return 1;
                s.objects.push_back(o);
                s.kinds.push_back(b);
            } else if (code == 1) {
                a = alive[pick(alive.size())];
                TheGameLogic->destroyObject(s.objects[a]);
                TheGameLogic->processDestroyList();
                s.objects[a] = nullptr;
            } else if (code == 2) {
                a = alive[pick(alive.size())];
                b = team;
                s.objects[a]->setTeam(s.teams[b]);
            } else if (code == 3) {
                a = team;
                b = static_cast<int>(pick(2));
                s.player(a)->addUpgrade(TheUpgradeCenter->findUpgrade(kUpgrade[b]),
                                        UPGRADE_STATUS_COMPLETE);
            } else if (code == 4 || code == 5) {
                a = team;
                b = static_cast<int>(pick(2));
                if (code == 4) s.player(a)->addRadar(b != 0);
                else s.player(a)->removeRadar(b != 0);
            } else if (code == 6) {
                a = team;
                b = static_cast<int>(pick(60));
                // `SabotagePowerPlantCrateCollide::executeCrateBehavior`.
                s.player(a)->getEnergy()->setPowerSabotagedTillFrame(frame + b);
                s.player(a)->onPowerBrownOutChange(TRUE);
            } else if (code == 7) {
                a = 1 + static_cast<int>(pick(40));
                frame += a;
                w.set_frame(frame);
                for (int p = 0; p < 3; ++p) ThePlayerList->getNthPlayer(p)->update();
            } else if (code == 8) {
                a = alive[pick(alive.size())];
                b = static_cast<int>(pick(2));
                if (b) s.objects[a]->setDisabled(DISABLED_HACKED);
                else ret = s.objects[a]->clearDisabled(DISABLED_HACKED) ? 1 : 0;
            } else if (code == 9) {
                a = team;
                b = static_cast<int>(pick(kMaskCount));
                c = static_cast<int>(pick(5));
                s.player(a)->addKindOfProductionCostChange(kind_mask(b), kPercents[c]);
            } else if (code == 10) {
                a = team;
                // An existing change, so the reference does not assert.
                const std::list<KindOfPercentProductionChange*>& list =
                    s.player(a)->*access_member(HChangesTag());
                if (list.empty()) {
                    b = -1;
                } else {
                    auto it = list.begin();
                    std::advance(it, pick(list.size()));
                    for (int i = 0; i < kMaskCount; ++i)
                        if ((*it)->m_kindOf == kind_mask(i)) b = i;
                    // -0.0 and 0.0 compare equal.
                    for (int i = 0; i < 5; ++i)
                        if (bits(kPercents[i]) == bits((*it)->m_percent)) c = i;
                    if (c >= 3) c = 3 + static_cast<int>(pick(2));
                    s.player(a)->removeKindOfProductionCostChange(kind_mask(b), kPercents[c]);
                }
            } else if (code == 11) {
                a = team;
                b = static_cast<int>(pick(4));
                s.player(a)->setCanBuildUnits((b & 1) != 0);
                s.player(a)->setCanBuildBase((b & 2) != 0);
            } else if (code == 12) {
                a = team;
                b = static_cast<int>(pick(4));
                c = static_cast<int>(pick(4));
                Dict h;
                h.setReal(NAMEKEY(kHandicapKeys[b]), kHandicaps[c]);
                s.player(a)->getHandicap()->readFromDict(&h);
            } else if (code == 13) {
                a = team;
                b = static_cast<int>(pick(kKinds * 2));
                c = static_cast<int>(pick(3));
                ProductionChangeMap& map = b < kKinds ? s.player(a)->*access_member(HCostTag())
                                                      : s.player(a)->*access_member(HTimeTag());
                map[NAMEKEY(kKind[b % kKinds])] = kPercents[c];
            } else if (code == 14) {
                a = team;
                b = static_cast<int>(pick(kSciences));
                ret = s.player(a)->grantScience(
                          TheScienceStore->getScienceFromInternalName(kScience[b]))
                          ? 1
                          : 0;
            } else if (code == 15) {
                a = team;
                b = static_cast<int>(pick(3));
                s.player(a)->setCashBounty(kPercents[b]);
            } else if (code == 16) {
                a = alive[pick(alive.size())];
                b = alive[pick(alive.size())];
                s.objects[a]->getControllingPlayer()->setAttackedBy(
                    s.objects[b]->getControllingPlayer()->getPlayerIndex());
            } else if (code >= 17 && code <= 22) {
                a = powered[pick(powered.size())];
                b = static_cast<int>(pick(power_count(s.objects[a])));
                SpecialPowerModuleInterface* sp = power_of(s.objects[a], b);
                if (code == 17) {
                    ret = sp->isReady() ? 1 : 0;
                } else if (code == 18) {
                    ret = sp->getReadyFrame();
                } else if (code == 19) {
                    const Real percent = sp->getPercentReady();
                    std::memcpy(&ret, &percent, 4);
                } else if (code == 20) {
                    sp->startPowerRecharge();
                } else if (code == 21) {
                    sp->onSpecialPowerCreation();
                } else {
                    c = static_cast<int>(pick(2));
                    sp->pauseCountdown(c != 0);
                }
            } else if (code == 26) {
                a = team;
                b = pick(8) == 0 ? 0 : static_cast<int>(pick(5000));
                c = static_cast<int>(pick(2));
                s.player(a)->getMoney()->deposit(b, TRUE, c != 0);
            } else if (code == 27) {
                a = team;
                b = pick(8) == 0 ? 0 : static_cast<int>(pick(5000));
                ret = s.player(a)->getMoney()->withdraw(b);
            } else {
                a = team;
                b = static_cast<int>(pick(kPowers));
                c = static_cast<int>(pick(200));
                const SpecialPowerTemplate* power =
                    TheSpecialPowerStore->findSpecialPowerTemplate(kPower[b]);
                if (code == 23) s.player(a)->expressSpecialPowerReadyFrame(power, frame + c);
                else if (code == 24) s.player(a)->resetOrStartSpecialPowerReadyFrame(power);
                else ret = s.player(a)->getOrStartSpecialPowerReadyFrame(power);
            }
            std::ostringstream op;
            op << code << ' ' << a << ' ' << b << ' ' << c << ' ' << frame;
            ops.push_back(op.str());
            rows.push_back(std::to_string(ret) + "!" + row(s));
        }

        std::fprintf(out, "%s\n{\"setup\":\"%u\",\"ops\":[", v ? "," : "", base);
        for (size_t i = 0; i < ops.size(); ++i)
            std::fprintf(out, "%s\"%s\"", i ? "," : "", ops[i].c_str());
        std::fprintf(out, "],\"rows\":[");
        for (size_t i = 0; i < rows.size(); ++i)
            std::fprintf(out, "%s\n\"%s\"", i ? "," : "", rows[i].c_str());
        std::fprintf(out, "]}");
    }
    std::fprintf(out, "]}\n");
    if (std::fclose(out) != 0 || std::rename(tmp.c_str(), argv[1]) != 0) return 1;
    std::fflush(nullptr);
    std::_Exit(0);
}
