// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for `Player::attemptToPurchaseScience`, run on the unmodified
// reference: the gate, the points, the science list, the academy count, and the
// special power modules of the player's objects.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_science_purchase.cpp -o /tmp/gen
//   (also with --opt O0 and with --opt O2 --sanitize)
//   /tmp/gen docs/golden_vectors/SciencePurchase.json
//
// Not covered: the control bar (client) and the script engine's list.

#include "oracle_wp51_world.h"

#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/Science.h"
#include "Common/SpecialPower.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/SpecialPowerModule.h"
#include "GameLogic/Object.h"

#include <cstdio>
#include <sstream>
#include <string>
#include <vector>

struct SciPointsTag { using type = Int Player::*; };
template struct MemberAccess<SciPointsTag, &Player::m_sciencePurchasePoints>;
Int Player::* access_member(SciPointsTag);
struct SciListTag { using type = ScienceVec Player::*; };
template struct MemberAccess<SciListTag, &Player::m_sciences>;
ScienceVec Player::* access_member(SciListTag);
struct AcademySpentTag { using type = UnsignedInt AcademyStats::*; };
template struct MemberAccess<AcademySpentTag, &AcademyStats::m_generalsPointsSpent>;
UnsignedInt AcademyStats::* access_member(AcademySpentTag);

static uint32_t g_state = 0x5c1e7ce5u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}

static const int kSciences = 6;
static const char* const kScience[kSciences] = {"SCI_A", "SCI_B", "SCI_C",
                                                "SCI_Free", "SCI_Locked", "SCI_Dear"};
static const char* const kScienceIni =
    "Science SCI_A\n  PrerequisiteSciences = None\n  SciencePurchasePointCost = 1\n"
    "  IsGrantable = Yes\nEnd\n"
    "Science SCI_B\n  PrerequisiteSciences = SCI_A\n  SciencePurchasePointCost = 1\n"
    "  IsGrantable = Yes\nEnd\n"
    "Science SCI_C\n  PrerequisiteSciences = SCI_A SCI_B\n  SciencePurchasePointCost = 3\n"
    "  IsGrantable = Yes\nEnd\n"
    "Science SCI_Free\n  PrerequisiteSciences = None\n  SciencePurchasePointCost = 0\n"
    "  IsGrantable = Yes\nEnd\n"
    "Science SCI_Locked\n  PrerequisiteSciences = None\n  SciencePurchasePointCost = 2\n"
    "  IsGrantable = No\nEnd\n"
    "Science SCI_Dear\n  PrerequisiteSciences = None\n  SciencePurchasePointCost = 5\n"
    "  IsGrantable = Yes\nEnd\n";

static const char* const kPowerIni =
    "SpecialPower SP_A\n  Enum = SPECIAL_CASH_HACK\n  ReloadTime = 3000\n"
    "  RequiredScience = SCI_A\nEnd\n"
    "SpecialPower SP_B\n  Enum = SPECIAL_DEFECTOR\n  ReloadTime = 6000\n"
    "  RequiredScience = SCI_B\n  SharedSyncedTimer = Yes\nEnd\n"
    "SpecialPower SP_Locked\n  Enum = SPECIAL_CARPET_BOMB\n  ReloadTime = 9000\n"
    "  RequiredScience = SCI_Locked\nEnd\n"
    "SpecialPower SP_Open\n  Enum = SPECIAL_DAISY_CUTTER\n  ReloadTime = 1500\nEnd\n";

static std::string module(int tag, const char* power, bool paused) {
    return "  Behavior = SpecialAbility ModuleTag_" + std::to_string(tag) +
           "\n    SpecialPowerTemplate = " + power + "\n" +
           (paused ? "    StartsPaused = Yes\n" : "") + "  End\n";
}

static const char* const kKinds[2] = {"G_Centre", "G_Other"};

static std::string objects_ini() {
    const std::string head =
        "\n  KindOf = STRUCTURE\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100.0\n"
        "    InitialHealth = 100.0\n  End\n";
    return std::string("Object G_Centre") + head + module(0, "SP_A", false) +
           module(1, "SP_B", false) + module(2, "SP_Open", false) + "End\n" +
           "Object G_Other" + head + module(0, "SP_B", true) + module(1, "SP_Locked", false) +
           module(2, "SP_A", false) + "End\n";
}

static std::string escaped(const std::string& text) {
    std::string out;
    for (char c : text) {
        if (c == '\n') out += "\\n";
        else out += c;
    }
    return out;
}

static std::string state(Player* const players[2], const std::vector<Object*>& objects) {
    std::ostringstream s;
    for (int p = 0; p < 2; ++p) {
        s << " " << players[p]->*access_member(SciPointsTag()) << "/"
          << players[p]->getAcademyStats()->*access_member(AcademySpentTag()) << "/";
        const ScienceVec& list = players[p]->*access_member(SciListTag());
        for (ScienceType sci : list) {
            for (int i = 0; i < kSciences; ++i) {
                if (TheScienceStore->getScienceFromInternalName(kScience[i]) == sci) s << i;
            }
        }
    }
    for (Object* o : objects) {
        s << " ";
        for (BehaviorModule** m = o->getBehaviorModules(); *m; ++m) {
            SpecialPowerModuleInterface* sp = (*m)->getSpecialPower();
            if (sp) s << sp->getReadyFrame() << ",";
        }
    }
    return s.str();
}

int main(int argc, char** argv) {
    if (argc < 2) return 2;
    oracle_wp51::World w;
    w.load_ini(kScienceIni);
    w.load_ini(kPowerIni);
    w.load_ini(objects_ini());
    TheThingFactory->postProcessLoad();
    w.init_with_neutral_player();

    Player* players[2] = {ThePlayerList->getNthPlayer(1), ThePlayerList->getNthPlayer(2)};
    Team* teams[2] = {w.team_a, w.team_b};
    std::vector<Object*> objects;
    const int kObjects = 3;
    const int kind_of[kObjects] = {0, 1, 0};
    const int team_of[kObjects] = {0, 0, 1};
    for (int i = 0; i < kObjects; ++i) {
        const ThingTemplate* t = TheThingFactory->findTemplate(kKinds[kind_of[i]]);
        if (!t) return 1;
        objects.push_back(TheThingFactory->newObject(t, teams[team_of[i]]));
        if (objects.back()->getControllingPlayer() != players[team_of[i]]) {
            std::fprintf(stderr, "team owner\n");
            return 1;
        }
    }

    std::vector<std::string> rows;
    rows.push_back("sciences " + escaped(kScienceIni));
    rows.push_back("powers " + escaped(kPowerIni));
    rows.push_back("objects " + escaped(objects_ini()));
    rows.push_back("start" + state(players, objects));
    UnsignedInt frame = 100;
    for (int k = 0; k < 400; ++k) {
        const int p = pick(4) == 0 ? 1 : 0;
        const int code = k % 40 == 0 ? 0 : static_cast<int>(pick(8));
        int a = 0;
        int ret = 0;
        w.set_frame(frame);
        if (code == 0) {
            // Start again: no sciences, nothing disabled or hidden.
            for (int q = 0; q < 2; ++q) {
                (players[q]->*access_member(SciListTag())).clear();
                for (int i = 0; i < kSciences; ++i) {
                    players[q]->setScienceAvailability(
                        TheScienceStore->getScienceFromInternalName(kScience[i]),
                        SCIENCE_AVAILABLE);
                }
            }
        } else if (code == 1) {
            a = static_cast<int>(pick(9));
            players[p]->*access_member(SciPointsTag()) = a;
        } else if (code == 2) {
            a = static_cast<int>(pick(kSciences * 3));
            players[p]->setScienceAvailability(
                TheScienceStore->getScienceFromInternalName(kScience[a / 3]),
                static_cast<ScienceAvailabilityType>(a % 3));
        } else if (code == 3) {
            a = 1 + static_cast<int>(pick(500));
            frame += a;
        } else {
            // -1 is `SCIENCE_INVALID`.
            a = static_cast<int>(pick(kSciences + 1)) - 1;
            if (code >= 6) a = static_cast<int>(pick(3));
            const ScienceType sci =
                a < 0 ? SCIENCE_INVALID : TheScienceStore->getScienceFromInternalName(kScience[a]);
            ret = players[p]->attemptToPurchaseScience(sci) ? 1 : 0;
        }
        std::ostringstream row;
        row << "op " << code << " " << p << " " << a << " " << frame << " " << ret
            << state(players, objects);
        rows.push_back(row.str());
    }

    FILE* out = std::fopen(argv[1], "w");
    if (!out) return 1;
    std::fputs("[\n", out);
    for (size_t i = 0; i < rows.size(); ++i) {
        std::fprintf(out, " \"%s\"%s\n", rows[i].c_str(), i + 1 < rows.size() ? "," : "");
    }
    std::fputs("]\n", out);
    std::fclose(out);
    std::printf("%zu rows\n", rows.size());
    return 0;
}
