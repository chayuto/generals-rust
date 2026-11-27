// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for the state `Player` owns (money, energy, upgrades) when a
// player has several teams, run on the unmodified reference in a small real world.
// Each vector is one script; its first row is the setup.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_player_state.cpp -o /tmp/gen
//   (also with --opt O0 and with --opt O2 --sanitize)
//   /tmp/gen docs/golden_vectors/PlayerState.json
//
// Not covered: brown-out effects (no template is KINDOF_POWERED), sabotage expiry
// (golden_s0_frame_driver), income buckets and `Player::xfer` as a whole (neither
// is ported), construction, and moving a player's default team to another player.

#include "oracle_wp51_world.h"

#include "Common/Energy.h"
#include "Common/Money.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/ScoreKeeper.h"
#include "Common/Team.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/Upgrade.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/CreateModule.h"
#include "GameLogic/Module/ProductionUpdate.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Module/UpgradeModule.h"
#include "GameLogic/Object.h"

#include <cstdio>
#include <map>
#include <sstream>
#include <string>
#include <vector>

static uint32_t g_state = 0x71a9c0deu;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}

static const char* const kUpgrades[] = {"PU_Ord0",  "PU_Ord1",  "PU_Ord2", "PU_Ord3",
                                        "PU_Alt",   "PU_Boost", "PU_Grant", "PU_Block",
                                        "PU_Power", "OU_A",     "OU_B",     "OU_Grant"};
static const int kPlayerUpgrades = 9;
static const int kAllUpgrades = 12;

static const char* const kKinds[] = {"G_Plant", "G_Eater", "G_Lab",
                                     "G_Bank",  "G_Grant", "G_GrantObj"};
static const int kKindCount = 6;
static const int kLab = 2;
static const int kBank = 3;

static std::string upgrades_ini() {
    std::string s;
    for (int i = 0; i < kAllUpgrades; ++i) {
        const bool player = i < kPlayerUpgrades;
        s += std::string("Upgrade ") + kUpgrades[i] + "\n  Type = " + (player ? "PLAYER" : "OBJECT") +
             "\n  BuildTime = " + (i % 3 == 0 ? "0.0" : i % 3 == 1 ? "0.1" : "0.2") +
             "\n  BuildCost = " + std::to_string(40 + 30 * i) + "\nEnd\n";
    }
    return s;
}

static std::string ocl_ini() {
    std::string s;
    for (int i = 0; i < 4; ++i) {
        const std::string n = std::to_string(i);
        s += "ObjectCreationList OCL_G" + n + "\n  CreateObject\n    ObjectNames = G_Mark" + n +
             "\n  End\nEnd\n";
    }
    return s;
}

static std::string object_ini(const std::string& name, const std::string& body) {
    std::string s = "Object " + name +
                    "\n  KindOf = STRUCTURE\n"
                    "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100.0\n"
                    "    InitialHealth = 100.0\n  End\n" +
                    body;
    for (int i = 0; i < 4; ++i) {
        const std::string n = std::to_string(i);
        s += "  Behavior = ObjectCreationUpgrade ModuleTag_Ord" + n + "\n    TriggeredBy = PU_Ord" +
             n + "\n    UpgradeObject = OCL_G" + n + "\n  End\n";
    }
    return s + "End\n";
}

static std::string behavior(const std::string& module, const std::string& tag, const std::string& fields) {
    return "  Behavior = " + module + " " + tag + "\n" + fields + "  End\n";
}

static std::string objects_ini() {
    std::string s;
    for (int i = 0; i < 4; ++i) {
        s += "Object G_Mark" + std::to_string(i) +
             "\n  KindOf = INERT\n  Body = InactiveBody ModuleTag_Body\n  End\nEnd\n";
    }
    s += object_ini("G_Plant",
                    "  EnergyProduction = 10\n  EnergyBonus = 7\n" +
                    behavior("PowerPlantUpdate", "ModuleTag_Rods", "    RodsExtendTime = 100\n") +
                    behavior("PowerPlantUpgrade", "ModuleTag_Power", "    TriggeredBy = PU_Power\n"));
    s += object_ini("G_Eater",
                    "  EnergyProduction = -6\n" +
                    behavior("MaxHealthUpgrade", "ModuleTag_Any",
                             "    TriggeredBy = PU_Alt OU_A\n    AddMaxHealth = 25.0\n") +
                    behavior("MaxHealthUpgrade", "ModuleTag_Blocked",
                             "    TriggeredBy = PU_Boost\n    ConflictsWith = PU_Block\n"
                             "    AddMaxHealth = 7.0\n"));
    s += object_ini("G_Lab",
                    "  EnergyProduction = -3\n  CommandSet = G_LabSet\n" +
                    behavior("ProductionUpdate", "ModuleTag_Production", "    MaxQueueEntries = 3\n") +
                    behavior("CommandSetUpgrade", "ModuleTag_Set",
                             "    TriggeredBy = PU_Power\n    CommandSet = G_LabSet2\n"
                             "    CommandSetAlt = G_LabSet3\n    TriggerAlt = PU_Alt\n"));
    s += object_ini("G_Bank",
                    "  EnergyProduction = -1\n" +
                    behavior("AutoDepositUpdate", "ModuleTag_Deposit",
                             "    DepositTiming = 100\n    DepositAmount = 20\n"
                             "    InitialCaptureBonus = 50\n"
                             "    UpgradedBoost = UpgradeType:PU_Boost Boost:5\n") +
                    behavior("WeaponBonusUpgrade", "ModuleTag_Bonus",
                             "    TriggeredBy = PU_Alt OU_B\n    RequiresAllTriggers = Yes\n"));
    s += object_ini("G_Grant",
                    "  EnergyProduction = 4\n  EnergyBonus = 3\n" +
                    behavior("GrantUpgradeCreate", "ModuleTag_Grant",
                             "    UpgradeToGrant = PU_Grant\n    ExemptStatus = UNDER_CONSTRUCTION\n") +
                    behavior("PowerPlantUpgrade", "ModuleTag_Power", "    TriggeredBy = PU_Grant\n"));
    s += object_ini("G_GrantObj",
                    behavior("GrantUpgradeCreate", "ModuleTag_Grant",
                             "    UpgradeToGrant = OU_Grant\n    ExemptStatus = UNDER_CONSTRUCTION\n") +
                    behavior("MaxHealthUpgrade", "ModuleTag_Health",
                             "    TriggeredBy = OU_Grant\n    AddMaxHealth = 11.0\n"));
    return s;
}

// The command sets: which upgrades each of the lab's three sets can research.
static const int kSetCount = 3;
static const char* const kSets[kSetCount] = {"G_LabSet", "G_LabSet2", "G_LabSet3"};
static const int kSetUpgrades[kSetCount][7] = {
    {0, 1, 4, 5, 8, 9, -1}, {2, 3, 4, 6, 10, -1, -1}, {0, 2, 5, 7, 9, 10, -1}};

static std::string commands_ini() {
    std::string s;
    for (int i = 0; i < kAllUpgrades; ++i) {
        s += std::string("CommandButton G_B_") + kUpgrades[i] + "\n  Command = " +
             (i < kPlayerUpgrades ? "PLAYER_UPGRADE" : "OBJECT_UPGRADE") + "\n  Upgrade = " +
             kUpgrades[i] + "\nEnd\n";
    }
    for (int set = 0; set < kSetCount; ++set) {
        s += std::string("CommandSet ") + kSets[set] + "\n";
        for (int k = 0; kSetUpgrades[set][k] >= 0; ++k) {
            s += "  " + std::to_string(k + 1) + " = G_B_" + kUpgrades[kSetUpgrades[set][k]] + "\n";
        }
        s += "End\n";
    }
    return s;
}

// Floating cash text needs fonts and a display, which a headless run has none of.
class QuietUI : public oracle_wp51::NullInGameUI {
public:
    void addFloatingText(const UnicodeString&, const Coord3D*, Color) override {}
};

// "set:upgrade,upgrade;..." for the Rust side to build the same command sets.
static std::string sets_text() {
    std::string s;
    for (int set = 0; set < kSetCount; ++set) {
        s += std::string(kSets[set]) + ":";
        for (int k = 0; kSetUpgrades[set][k] >= 0; ++k)
            s += (k ? "," : "") + std::string(kUpgrades[kSetUpgrades[set][k]]);
        s += ";";
    }
    return s;
}

static std::string escaped(const std::string& text) {
    std::string out;
    for (char c : text) {
        if (c == '\n') out += "\\n";
        else out += c;
    }
    return out;
}

static const UpgradeTemplate* upgrade(int i) {
    const UpgradeTemplate* u = TheUpgradeCenter->findUpgrade(kUpgrades[i]);
    if (!u) {
        std::fprintf(stderr, "missing upgrade %s\n", kUpgrades[i]);
        std::exit(1);
    }
    return u;
}

struct Trial {
    // Team labels: 0 neutral, 1 and 2 the default teams of players 1 and 2,
    // 3 a second prototype of player 1, 4 and 6.. instances of a third, 5 player 2's.
    std::vector<Team*> teams;
    std::vector<int> instances;  // labels of the multi-instance prototype
    std::vector<Object*> objects;  // nullptr once destroyed
    std::vector<int> kinds;
    std::map<ObjectID, int> index_of;
    ObjectID seen = INVALID_ID;
    std::string multi_name;

    int label_of(const Team* t) const {
        for (size_t i = 0; i < teams.size(); ++i)
            if (teams[i] == t) return static_cast<int>(i);
        std::fprintf(stderr, "unknown team\n");
        std::exit(1);
    }

    Player* player(int team) const { return teams[team]->getControllingPlayer(); }

    // "k:i," for every marker created since the last call, oldest first: the
    // `ObjectCreationUpgrade` number and the index of the object that ran it.
    std::string markers() {
        std::map<ObjectID, const Object*> fresh;
        ObjectID top = seen;
        for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
            if (o->getID() <= seen) continue;
            if (o->getID() > top) top = o->getID();
            if (o->getTemplate()->getName().startsWith("G_Mark")) fresh[o->getID()] = o;
        }
        seen = top;
        std::string out;
        for (const auto& entry : fresh) {
            const auto it = index_of.find(entry.second->getProducerID());
            if (it == index_of.end()) {
                std::fprintf(stderr, "marker without a known producer\n");
                std::exit(1);
            }
            out += std::string(1, entry.second->getTemplate()->getName().str()[6]) + ":" +
                   std::to_string(it->second) + ",";
        }
        return out;
    }
};

static std::string mask_bits(const UpgradeMaskType& mask, int from, int to) {
    std::string out;
    for (int i = from; i < to; ++i) out += mask.testForAny(upgrade(i)->getUpgradeMask()) ? '1' : '0';
    return out;
}

// "money production consumption sufficient energy-save done pending earned spent"
static std::string player_row(int p) {
    Player* player = ThePlayerList->getNthPlayer(p);
    Energy* e = player->getEnergy();
    std::ostringstream out;
    out << player->getMoney()->countMoney() << ' ' << e->getProduction() << ' ' << e->getConsumption()
        << ' ' << (e->hasSufficientPower() ? 1 : 0) << ' ' << oracle_wp51::World::snapshot_save_hex(e)
        << ' ';
    for (int i = 0; i < kPlayerUpgrades; ++i)
        out << (player->hasUpgradeComplete(upgrade(i)) ? '1' : '0');
    out << ' ';
    for (int i = 0; i < kPlayerUpgrades; ++i)
        out << (player->hasUpgradeInProduction(upgrade(i)) ? '1' : '0');
    out << ' ' << player->getScoreKeeper()->getTotalMoneyEarned() << ' '
        << player->getScoreKeeper()->getTotalMoneySpent();
    return out.str();
}

// "team modules object-upgrades max-health weapon-bonus command-set disabled queued affected"
static std::string object_row(const Trial& s, int index, int asked) {
    Object* o = s.objects[index];
    std::ostringstream out;
    out << s.label_of(o->getTeam()) << ' ';
    unsigned queued = 0;
    for (BehaviorModule** m = o->getBehaviorModules(); *m; ++m) {
        if (UpgradeModuleInterface* u = (*m)->getUpgrade()) out << (u->isAlreadyUpgraded() ? '1' : '0');
        if (ProductionUpdateInterface* p = (*m)->getProductionUpdateInterface())
            queued = p->getProductionCount();
    }
    out << ' ' << mask_bits(o->getObjectCompletedUpgradeMask(), kPlayerUpgrades, kAllUpgrades) << ' '
        << static_cast<int>(o->getBodyModule()->getMaxHealth()) << ' ' << o->getWeaponBonusCondition()
        << ' ' << o->getCommandSetString().str() << ' ' << (o->isDisabled() ? 1 : 0) << ' ' << queued
        << ' ' << (o->affectedByUpgrade(upgrade(asked)) ? 1 : 0);
    return out.str();
}

static std::string row(Trial& s, int asked, uint32_t ret) {
    std::ostringstream out;
    out << ret << '|' << s.markers() << '|';
    for (size_t t = 0; t < s.teams.size(); ++t)
        out << s.player(static_cast<int>(t))->getPlayerIndex();
    for (int p = 0; p < 3; ++p) out << '|' << player_row(p);
    out << '|';
    for (size_t i = 0; i < s.objects.size(); ++i) {
        if (s.objects[i]) out << i << ' ' << object_row(s, static_cast<int>(i), asked) << ';';
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
        player->getMoney()->withdraw(player->getMoney()->countMoney(), FALSE);
        for (int i = 0; i < kPlayerUpgrades; ++i) player->removeUpgrade(upgrade(i));
        player->getEnergy()->init(player);
        player->getScoreKeeper()->reset(p);
    }
}

static uint32_t player_crc(int p) {
    return oracle_wp51::World::snapshot_crc(ThePlayerList->getNthPlayer(p));
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_player_state <out.json>\n");
        return 2;
    }
    oracle_wp51::World w;
    TheInGameUI = new QuietUI;
    w.load_ini(upgrades_ini());
    w.load_ini(ocl_ini());
    w.load_ini(objects_ini());
    w.mem_ini.parse_blocks(commands_ini(), INI_LOAD_OVERWRITE, [](INI* ini, const char* tok) {
        if (std::strcmp(tok, "CommandButton") == 0) INI::parseCommandButtonDefinition(ini);
        else if (std::strcmp(tok, "CommandSet") == 0) INI::parseCommandSetDefinition(ini);
    });
    TheThingFactory->postProcessLoad();
    w.init_with_neutral_player();
    // `Player::init` sets the money's player only with a template.
    for (int p = 0; p < 3; ++p) ThePlayerList->getNthPlayer(p)->getMoney()->setPlayerIndex(p);
    for (int p = 0; p < 3; ++p) {
        if (ThePlayerList->getNthPlayer(p)->getPlayerIndex() != p) {
            std::fprintf(stderr, "player index\n");
            return 1;
        }
    }
    // Money, energy and upgrades are not part of the game CRC.
    const uint32_t crc_before[3] = {player_crc(0), player_crc(1), player_crc(2)};

    const std::string tmp = std::string(argv[1]) + ".tmp";
    FILE* out = std::fopen(tmp.c_str(), "w");
    if (!out) return 1;
    const int kTotal = 1000;
    std::fprintf(out, "{\"vector_count\":%d,\n\"source_sha256\":{", kTotal);
    const char* const kSources[] = {"Player", "Team", "Energy", "Money"};
    for (int i = 0; i < 4; ++i) {
        const std::string path = std::string("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/") +
                                 kSources[i] + ".cpp";
        std::fprintf(out, "%s\"%s.cpp\":\"%s\"", i ? "," : "", kSources[i],
                     oracle::sha256_file(path.c_str()).c_str());
    }
    std::fprintf(out, "},\n\"upgrades\":\"%s\",\n\"ocl\":\"%s\",\n\"objects\":\"%s\",\n",
                 escaped(upgrades_ini()).c_str(), escaped(ocl_ini()).c_str(), escaped(objects_ini()).c_str());
    std::string names, kinds;
    for (int i = 0; i < kAllUpgrades; ++i) names += (i ? "," : "") + std::string(kUpgrades[i]);
    for (int i = 0; i < kKindCount; ++i) kinds += (i ? "," : "") + std::string(kKinds[i]);
    std::fprintf(out, "\"names\":\"%s\",\n\"kinds\":\"%s\",\n\"sets\":\"%s\",\n", names.c_str(),
                 kinds.c_str(), sets_text().c_str());
    std::fprintf(out, "\"vectors\":[");

    UnsignedInt base = 10;
    for (int v = 0; v < kTotal; ++v) {
        reset_players();
        base += 1000;
        UnsignedInt frame = base;
        w.set_frame(frame);

        Trial s;
        for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) s.seen = o->getID();
        Dict dict;
        const std::string tag = "V" + std::to_string(v);
        s.multi_name = tag + "_M";
        TheTeamFactory->initTeam((tag + "_A2").c_str(), AsciiString::TheEmptyString, TRUE, &dict);
        TheTeamFactory->initTeam(s.multi_name.c_str(), AsciiString::TheEmptyString, FALSE, &dict);
        TheTeamFactory->initTeam((tag + "_B2").c_str(), AsciiString::TheEmptyString, TRUE, &dict);
        Team* a2 = TheTeamFactory->findTeam((tag + "_A2").c_str());
        Team* m0 = TheTeamFactory->createTeam(s.multi_name.c_str());
        Team* b2 = TheTeamFactory->findTeam((tag + "_B2").c_str());
        s.teams = {w.team_neutral, w.team_a, w.team_b, a2, m0, b2};
        s.instances = {4};

        // The order of each player's team list.
        std::vector<int> order_a = {1, 3, 4};
        for (int i = 2; i > 0; --i) std::swap(order_a[i], order_a[pick(i + 1)]);
        std::vector<int> order_b = {2, 5};
        if (pick(2)) std::swap(order_b[0], order_b[1]);
        for (int t : order_a) s.teams[t]->setControllingPlayer(ThePlayerList->getNthPlayer(1));
        for (int t : order_b) s.teams[t]->setControllingPlayer(ThePlayerList->getNthPlayer(2));

        std::ostringstream setup;
        std::vector<uint32_t> money(3);
        for (int p = 0; p < 3; ++p) {
            // Some players start with nothing, so no purse exists before the first use.
            money[p] = pick(4) == 0 ? 0 : pick(5) == 0 ? pick(120) : 200 + pick(2000);
            ThePlayerList->getNthPlayer(p)->getMoney()->deposit(money[p], FALSE, FALSE);
        }
        std::vector<std::string> rows, ops;
        rows.push_back(row(s, 0, 0));

        const int count = 14 + static_cast<int>(pick(16));
        for (int k = 0; k < count; ++k) {
            // Codes: 0 spawn, 1 destroy, 2 move, 3 grant, 4 start research mark, 5 remove,
            // 6 give object upgrade, 7 remove object upgrade, 8 deposit, 9 withdraw,
            // 10 disable, 11 enable, 12 sabotage, 13 frames, 14 hand a team over,
            // 15 new team instance, 16 queue, 17 cancel, 18 build complete.
            static const int kWeights[] = {0, 0, 0, 0, 1, 2, 2, 2, 3, 3, 3, 4, 5, 6, 6, 7, 8, 9,
                                           10, 11, 12, 13, 13, 13, 14, 15, 16, 16, 16, 17, 18};
            uint32_t code = kWeights[pick(sizeof(kWeights) / sizeof(kWeights[0]))];
            std::vector<int> alive, labs;
            for (size_t i = 0; i < s.objects.size(); ++i) {
                if (!s.objects[i]) continue;
                alive.push_back(static_cast<int>(i));
                if (s.kinds[i] == kLab) labs.push_back(static_cast<int>(i));
            }
            // Start by populating the teams.
            if (k < 5) code = 0;
            const bool needs_object = code == 1 || code == 2 || code == 6 || code == 7 ||
                                      code == 10 || code == 11 || code == 18;
            if ((needs_object && alive.empty()) || ((code == 16 || code == 17) && labs.empty()) ||
                (code == 0 && alive.size() >= 12))
                code = 3;
            const int team = static_cast<int>(pick(static_cast<uint32_t>(s.teams.size())));
            int a = 0, b = 0;
            uint32_t ret = 0;
            w.set_frame(frame);
            if (code == 0) {
                a = team == 0 && pick(2) ? 1 + static_cast<int>(pick(5)) : team;
                b = k == 0 && pick(2) ? kLab : static_cast<int>(pick(kKindCount));
                const int index = static_cast<int>(s.objects.size());
                // Registered before creation: its create modules already run upgrades.
                const ThingTemplate* tt = TheThingFactory->findTemplate(kKinds[b]);
                if (!tt) return 1;
                const ObjectID next = TheGameLogic->getObjectIDCounter();
                s.index_of[next] = index;
                Object* o = TheThingFactory->newObject(tt, s.teams[a]);
                if (!o || o->getID() != next) {
                    std::fprintf(stderr, "object id\n");
                    return 1;
                }
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
                b = static_cast<int>(pick(kPlayerUpgrades));
                if (pick(3) == 0) b = static_cast<int>(pick(4));
                s.player(a)->addUpgrade(upgrade(b), UPGRADE_STATUS_COMPLETE);
            } else if (code == 4) {
                a = team;
                b = static_cast<int>(pick(kPlayerUpgrades));
                s.player(a)->addUpgrade(upgrade(b), UPGRADE_STATUS_IN_PRODUCTION);
            } else if (code == 5) {
                a = team;
                b = static_cast<int>(pick(kPlayerUpgrades));
                s.player(a)->removeUpgrade(upgrade(b));
            } else if (code == 6 || code == 7) {
                a = alive[pick(alive.size())];
                b = kPlayerUpgrades + static_cast<int>(pick(kAllUpgrades - kPlayerUpgrades));
                if (code == 6) s.objects[a]->giveUpgrade(upgrade(b));
                else s.objects[a]->removeUpgrade(upgrade(b));
            } else if (code == 8) {
                a = team;
                b = static_cast<int>(pick(400));
                s.player(a)->getMoney()->deposit(static_cast<UnsignedInt>(b), FALSE);
            } else if (code == 9) {
                a = team;
                b = static_cast<int>(pick(1500));
                ret = s.player(a)->getMoney()->withdraw(static_cast<UnsignedInt>(b), FALSE);
            } else if (code == 10 || code == 11) {
                a = alive[pick(alive.size())];
                if (code == 10) s.objects[a]->setDisabled(DISABLED_HACKED);
                else ret = s.objects[a]->clearDisabled(DISABLED_HACKED) ? 1 : 0;
            } else if (code == 12) {
                a = team;
                b = pick(4) == 0 ? 0 : static_cast<int>(pick(40));
                s.player(a)->getEnergy()->setPowerSabotagedTillFrame(frame + b);
            } else if (code == 13) {
                a = 1 + static_cast<int>(pick(8));
                for (int t = 0; t < a; ++t) {
                    w.set_frame(++frame);
                    for (size_t i = 0; i < s.objects.size(); ++i) {
                        if (!s.objects[i] || (s.kinds[i] != kLab && s.kinds[i] != kBank)) continue;
                        for (BehaviorModule** m = s.objects[i]->getBehaviorModules(); *m; ++m) {
                            if ((*m)->getModuleNameKey() == NAMEKEY(s.kinds[i] == kLab
                                                                       ? "ProductionUpdate"
                                                                       : "AutoDepositUpdate")) {
                                static_cast<UpdateModule*>(*m)->update();
                            }
                        }
                    }
                }
            } else if (code == 14) {
                // Never a default team: `Player::m_defaultTeam` would keep pointing at it.
                a = 3 + static_cast<int>(pick(static_cast<uint32_t>(s.teams.size()) - 3));
                b = pick(4) == 0 ? 0 : 1 + static_cast<int>(pick(2));
                s.teams[a]->setControllingPlayer(ThePlayerList->getNthPlayer(b));
            } else if (code == 15) {
                a = static_cast<int>(s.teams.size());
                b = s.instances[pick(s.instances.size())];
                s.teams.push_back(TheTeamFactory->createTeam(s.multi_name.c_str()));
                s.instances.push_back(a);
            } else if (code == 16 || code == 17) {
                a = labs[pick(labs.size())];
                const char* set_name = s.objects[a]->getCommandSetString().str();
                int set = 0;
                for (int i = 0; i < kSetCount; ++i)
                    if (std::strcmp(set_name, kSets[i]) == 0) set = i;
                int n = 0;
                while (kSetUpgrades[set][n] >= 0) ++n;
                b = pick(5) == 0 ? static_cast<int>(pick(kAllUpgrades))
                                 : kSetUpgrades[set][pick(n)];
                ProductionUpdateInterface* pu = s.objects[a]->getProductionUpdateInterface();
                if (code == 16) {
                    ret = pu->queueUpgrade(upgrade(b)) ? 1 : 0;
                } else {
                    pu->cancelUpgrade(upgrade(b));
                }
            } else {
                a = alive[pick(alive.size())];
                for (BehaviorModule** m = s.objects[a]->getBehaviorModules(); *m; ++m) {
                    if (CreateModuleInterface* c = (*m)->getCreate()) {
                        if (c->shouldDoOnBuildComplete()) c->onBuildComplete();
                    }
                }
            }
            const int asked = static_cast<int>(pick(kAllUpgrades));
            ops.push_back("[" + std::to_string(code) + "," + std::to_string(a) + "," +
                          std::to_string(b) + "," + std::to_string(asked) + "]");
            rows.push_back(row(s, asked, ret));
        }
        for (int p = 0; p < 3; ++p) {
            if (player_crc(p) != crc_before[p]) {
                std::fprintf(stderr, "vector %d changed the CRC of player %d\n", v, p);
                return 1;
            }
        }

        std::fprintf(out, "%s\n{\"v\":%d,\"frame\":%u,\"order_a\":\"%d%d%d\",\"order_b\":\"%d%d\","
                          "\"m0\":%u,\"m1\":%u,\"m2\":%u,\"ops\":\"",
                     v ? "," : "", v, base, order_a[0], order_a[1], order_a[2], order_b[0], order_b[1],
                     money[0], money[1], money[2]);
        for (const auto& op : ops) std::fprintf(out, "%s", op.c_str());
        std::fprintf(out, "\",\"rows\":\"");
        for (size_t i = 0; i < rows.size(); ++i)
            std::fprintf(out, "%s%s", i ? "~" : "", rows[i].c_str());
        std::fprintf(out, "\"}");
    }
    std::fprintf(out, "\n]}\n");
    if (std::fclose(out) != 0) return 1;
    if (std::rename(tmp.c_str(), argv[1]) != 0) return 1;
    return 0;
}
