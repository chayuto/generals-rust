// Golden vectors for PlayerList / Team / TeamPrototype / TeamFactory at game start
// (crates/zh_logic/tests/golden_players_teams.rs), over the unmodified reference
// library. The output is a script of rows: rows without a leading '=' are inputs
// the Rust test applies, '=' rows are observations it must reproduce.
//
// Build:
//   CPATH=vendor/GeneralsX/GeneralsMD/Code/GameEngineDevice/Include:vendor/GeneralsX/Core/GameEngineDevice/Include \
//     python3 scripts/build_oracle.py tests/cpp_harness/gen_players_teams.cpp -o /tmp/gen
// Run:
//   /tmp/gen docs/golden_vectors/PlayersTeams.json
//
// Retail mode, local only. <list> has one "<fnv64 of the file> <path>" per
// extracted map; <ini-dir> holds science.ini, rank.ini, playertemplate.ini and
// default_science.ini, default_playertemplate.ini from INIZH.big. Only digests
// are written.
//   /tmp/gen --retail <list> docs/golden_vectors/PlayersTeamsRetail.json <ini-dir>
#include "oracle_wp51_world.h"

#include "Common/DataChunk.h"
#include "Common/FileSystem.h"
#include "Common/Money.h"
#include "Common/PlayerList.h"
#include "Common/Team.h"
#include "Common/WellKnownKeys.h"
#include "GameLogic/AI.h"
#include "Common/ResourceGatheringManager.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/AIPlayer.h"
#include "GameLogic/Module/BattlePlanUpdate.h"
#include "GameLogic/ScriptEngine.h"
#include "GameLogic/Scripts.h"
#include "GameLogic/SidesList.h"
#include "GameLogic/Squad.h"
#include "GameNetwork/GameInfo.h"

#include "Common/LocalFileSystem.h"
#include "StdDevice/Common/StdLocalFileSystem.h"
#include "Common/TerrainTypes.h"
#include "GameClient/MapUtil.h"
#include "W3DDevice/GameLogic/W3DTerrainLogic.h"

#include <cstdio>
#include <cstring>
#include <fstream>
#include <string>
#include <vector>

struct PtAiTag { using type = AIPlayer* Player::*; };
template struct MemberAccess<PtAiTag, &Player::m_ai>;
inline AIPlayer* Player::* access_member(PtAiTag);

struct PtBonusTag { using type = BattlePlanBonuses* Player::*; };
template struct MemberAccess<PtBonusTag, &Player::m_battlePlanBonuses>;
inline BattlePlanBonuses* Player::* access_member(PtBonusTag);

struct PtWaypointTag { using type = Waypoint* TerrainLogic::*; };
template struct MemberAccess<PtWaypointTag, &TerrainLogic::m_waypointListHead>;
inline Waypoint* TerrainLogic::* access_member(PtWaypointTag);

struct PtPlayerRelTag { using type = PlayerRelationMap* Player::*; };
template struct MemberAccess<PtPlayerRelTag, &Player::m_playerRelations>;
inline PlayerRelationMap* Player::* access_member(PtPlayerRelTag);

struct PtTeamRelTag { using type = TeamRelationMap* Player::*; };
template struct MemberAccess<PtTeamRelTag, &Player::m_teamRelations>;
inline TeamRelationMap* Player::* access_member(PtTeamRelTag);

struct PtHuntTag { using type = Bool Player::*; };
template struct MemberAccess<PtHuntTag, &Player::m_unitsShouldHunt>;
inline Bool Player::* access_member(PtHuntTag);

struct PtTimerTag { using type = std::list<SpecialPowerReadyTimerType> Player::*; };
template struct MemberAccess<PtTimerTag, &Player::m_specialPowerReadyTimerList>;
inline std::list<SpecialPowerReadyTimerType> Player::* access_member(PtTimerTag);

struct PtUpDoneTag { using type = UpgradeMaskType Player::*; };
template struct MemberAccess<PtUpDoneTag, &Player::m_upgradesCompleted>;
inline UpgradeMaskType Player::* access_member(PtUpDoneTag);

struct PtUpBusyTag { using type = UpgradeMaskType Player::*; };
template struct MemberAccess<PtUpBusyTag, &Player::m_upgradesInProgress>;
inline UpgradeMaskType Player::* access_member(PtUpBusyTag);

struct PtPostTag { using type = void (Snapshot::*)(); };
template struct MemberAccess<PtPostTag, &Snapshot::loadPostProcess>;
inline void (Snapshot::*access_member(PtPostTag))();

// Rule files translate labels, which must not come back empty. The general's name
// stays empty, as in the synthetic vectors.
class PtGameText : public oracle_wp51::NullGameText {
public:
    UnicodeString fetch(const Char* label, Bool* exists = nullptr) override {
        if (exists) *exists = TRUE;
        if (std::strcmp(label, "SCIENCE:GeneralName") == 0) return UnicodeString::TheEmptyString;
        return UnicodeString(L"x");
    }
    UnicodeString fetch(AsciiString label, Bool* exists = nullptr) override { return fetch(label.str(), exists); }
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

static uint64_t fnv(const std::string& s) {
    uint64_t h = 0xcbf29ce484222325ull;
    for (unsigned char c : s) {
        h ^= c;
        h *= 0x100000001b3ull;
    }
    return h;
}

static uint64_t g_state = 0x9e3779b97f4a7c15ull;
static uint32_t pick(uint32_t n) {
    return n ? static_cast<uint32_t>(oracle::splitmix64(g_state) % n) : 0;
}
static bool chance(uint32_t percent) {
    return pick(100) < percent;
}

// ---- dictionaries -------------------------------------------------------------

// "key|type|value" joined by ';'.
struct Spec {
    Dict dict;
    std::string text;

    void field(const std::string& key, char type, const std::string& value) {
        if (!text.empty()) text += ';';
        text += key + "|" + type + "|" + value;
        const NameKeyType k = NAMEKEY(AsciiString(key.c_str()));
        switch (type) {
            case 's': dict.setAsciiString(k, AsciiString(value.c_str())); break;
            case 'b': dict.setBool(k, value == "1"); break;
            case 'i': dict.setInt(k, static_cast<Int>(std::strtol(value.c_str(), nullptr, 10))); break;
            case 'r': dict.setReal(k, oracle::from_bits(static_cast<uint32_t>(std::strtoul(value.c_str(), nullptr, 16)))); break;
            case 'u': {
                UnicodeString u;
                u.translate(AsciiString(value.c_str()));
                dict.setUnicodeString(k, u);
                break;
            }
        }
    }
    void s(const std::string& key, const std::string& v) { field(key, 's', v); }
    void b(const std::string& key, bool v) { field(key, 'b', v ? "1" : "0"); }
    void i(const std::string& key, Int v) { field(key, 'i', fmt("%d", v)); }
    void r(const std::string& key, Real v) { field(key, 'r', fmt("%08x", oracle::fbits(v))); }
    void u(const std::string& key, const std::string& v) { field(key, 'u', v); }
};

// ---- observations -------------------------------------------------------------

static std::string snapshot_hex(Snapshot* s) {
    return oracle_wp51::World::snapshot_save_hex(s);
}

static void blob_rows(int player_count) {
    for (int i = 0; i < player_count; ++i) {
        Player* p = ThePlayerList->getNthPlayer(i);
        if (p->*access_member(PtAiTag())) {
            row(fmt("BLOB %d ai ", i) + snapshot_hex(static_cast<Snapshot*>(p->*access_member(PtAiTag()))));
        }
        if (p->getResourceGatheringManager()) {
            row(fmt("BLOB %d rgm ", i) + snapshot_hex(static_cast<Snapshot*>(p->getResourceGatheringManager())));
        }
        if (p->getTunnelSystem()) {
            row(fmt("BLOB %d tunnel ", i) + snapshot_hex(static_cast<Snapshot*>(p->getTunnelSystem())));
        }
    }
}

static std::string digest_or_full(const std::string& hex, bool full) {
    return full ? hex : fmt("%zu:%016llx", hex.size() / 2, static_cast<unsigned long long>(fnv(hex)));
}

static std::vector<Int> proto_ids() {
    std::vector<Int> ids;
    for (int i = 0; i < MAX_PLAYER_COUNT; ++i) {
        for (TeamPrototype* t : *ThePlayerList->getNthPlayer(i)->getPlayerTeams()) {
            ids.push_back(static_cast<Int>(t->getID()));
        }
    }
    return ids;
}

static std::vector<Team*> live_teams() {
    std::vector<Team*> out;
    for (int i = 0; i < MAX_PLAYER_COUNT; ++i) {
        for (TeamPrototype* t : *ThePlayerList->getNthPlayer(i)->getPlayerTeams()) {
            for (DLINK_ITERATOR<Team> it = t->iterate_TeamInstanceList(); !it.done(); it.advance()) {
                out.push_back(it.cur());
            }
        }
    }
    return out;
}

static void check_players(bool full) {
    const Int count = ThePlayerList->getPlayerCount();
    row(fmt("= list local=%d count=%d slots=", ThePlayerList->getLocalPlayer()->getPlayerIndex(), count) + [&] {
        std::string s;
        for (int i = 0; i < MAX_PLAYER_COUNT; ++i) s += fmt("%d,", ThePlayerList->getSlotIndex(i));
        return s;
    }());
    for (int i = 0; i <= count && i < MAX_PLAYER_COUNT; ++i) {
        Player* p = ThePlayerList->getNthPlayer(i);
        std::string protos;
        for (TeamPrototype* t : *p->getPlayerTeams()) protos += fmt("%u,", t->getID());
        const Player* found = ThePlayerList->findPlayerWithNameKey(p->getPlayerNameKey());
        const AsciiString unit("PtUnit");
        const std::string changes = fmt("retal=%d cost=%08x time=%08x vet=%d ", p->isLogicalRetaliationModeEnabled() ? 1 : 0,
                                        oracle::fbits(p->getProductionCostChangePercent(unit)),
                                        oracle::fbits(p->getProductionTimeChangePercent(unit)),
                                        static_cast<int>(p->getProductionVeterancyLevel(unit)));
        row(fmt("= player %d side=%s base=%s type=%d color=%08x night=%08x money=%u def=%d pre=%d obs=%d dead=%d mp=%d keyfound=%d mask=%u ",
                i, p->getSide().str(), p->getBaseSide().str(), p->getPlayerType(),
                static_cast<unsigned>(p->getPlayerColor()), static_cast<unsigned>(p->getPlayerNightColor()),
                p->getMoney()->countMoney(), p->getDefaultTeam() ? static_cast<int>(p->getDefaultTeam()->getID()) : 0,
                p->didPlayerPreorder() ? 1 : 0, p->isPlayerObserver() ? 1 : 0, p->isPlayerDead() ? 1 : 0,
                p->getMpStartIndex(), found ? found->getPlayerIndex() : -1,
                static_cast<unsigned>(p->getPlayerMask())) + changes + "protos=" + protos);
    }
    for (Int id : proto_ids()) {
        const TeamPrototype* t = nullptr;
        for (int i = 0; i < MAX_PLAYER_COUNT && !t; ++i) {
            for (TeamPrototype* c : *ThePlayerList->getNthPlayer(i)->getPlayerTeams()) {
                if (static_cast<Int>(c->getID()) == id) t = c;
            }
        }
        const TeamTemplateInfo* n = t->getTemplateInfo();
        std::string units;
        for (int u = 0; u < n->m_numUnitsInfo; ++u) {
            units += fmt("%d/%d/%s,", n->m_unitsInfo[u].minUnits, n->m_unitsInfo[u].maxUnits, n->m_unitsInfo[u].unitThingName.str());
        }
        std::string generic;
        for (int g = 0; g < MAX_GENERIC_SCRIPTS; ++g) {
            if (!n->m_teamGenericScripts[g].isEmpty()) generic += fmt("%d:%s,", g, n->m_teamGenericScripts[g].str());
        }
        std::string text = fmt("%s owner=%d single=%d units=", t->getName().str(),
                               t->getControllingPlayer()->getPlayerIndex(), t->getIsSingleton() ? 1 : 0) + units;
        text += fmt(" home=%d:%08x,%08x,%08x", n->m_hasHomeLocation ? 1 : 0, oracle::fbits(n->m_homeLocation.x),
                    oracle::fbits(n->m_homeLocation.y), oracle::fbits(n->m_homeLocation.z));
        text += fmt(" s=%s|%s|%s|%s|%s|%s idle=%d thresh=%08x", n->m_scriptOnCreate.str(), n->m_scriptOnIdle.str(),
                    n->m_scriptOnEnemySighted.str(), n->m_scriptOnAllClear.str(), n->m_scriptOnUnitDestroyed.str(),
                    n->m_scriptOnDestroyed.str(), n->m_initialIdleFrames, oracle::fbits(n->m_destroyedThreshold));
        text += fmt(" f=%d%d%d%d%d%d%d%d%d%d max=%d pri=%d,%d,%d att=%d", n->m_isAIRecruitable, n->m_isBaseDefense,
                    n->m_isPerimeterDefense, n->m_automaticallyReinforce, n->m_transportsReturn, n->m_avoidThreats,
                    n->m_attackCommonTarget, n->m_teamStartsFull, n->m_transportsExit, n->m_executeActions,
                    n->m_maxInstances, n->m_productionPriority, n->m_productionPrioritySuccessIncrease,
                    n->m_productionPriorityFailureDecrease, static_cast<int>(n->m_initialTeamAttitude));
        text += fmt(" tr=%s at=%s vet=%d cond=%s g=", n->m_transportUnitType.str(), n->m_startReinforceWaypoint.str(),
                    static_cast<int>(n->m_veterancy), n->m_productionCondition.str()) + generic;
        row(fmt("= tpl %d ", id) + digest_or_full(text, full));
    }
}

static void check_teams() {
    std::string instances;
    for (int i = 0; i < MAX_PLAYER_COUNT; ++i) {
        for (TeamPrototype* t : *ThePlayerList->getNthPlayer(i)->getPlayerTeams()) {
            instances += fmt("%u:", t->getID());
            for (DLINK_ITERATOR<Team> it = t->iterate_TeamInstanceList(); !it.done(); it.advance()) {
                instances += fmt("%u,", it.cur()->getID());
            }
            instances += ';';
        }
    }
    row("= inst " + instances);
    std::string masks;
    for (int source = -1; source <= ThePlayerList->getPlayerCount(); ++source) {
        for (unsigned allowed = 0; allowed < 16; ++allowed) {
            masks += fmt("%x.", static_cast<unsigned>(ThePlayerList->getPlayersWithRelationship(source, allowed)));
        }
    }
    row("= masks " + digest_or_full(masks, false));
    const std::vector<Team*> teams = live_teams();
    std::string tt, pt;
    for (Team* a : teams) {
        for (Team* b : teams) tt += fmt("%d", static_cast<int>(a->getRelationship(b)));
        tt += fmt("%d", static_cast<int>(a->getRelationship(nullptr)));
        for (int i = 0; i < ThePlayerList->getPlayerCount(); ++i) {
            pt += fmt("%d", static_cast<int>(ThePlayerList->getNthPlayer(i)->getRelationship(a)));
        }
    }
    row(fmt("= trel %zu ", teams.size()) + digest_or_full(tt, false) + " " + digest_or_full(pt, false));
}

static void check_saves(bool full) {
    const Int count = ThePlayerList->getPlayerCount();
    row("GENERAL " + [&] {
        UnicodeString u = TheGameText->fetch("SCIENCE:GeneralName");
        std::string s;
        for (int i = 0; i < u.getLength(); ++i) s += fmt("%04x", static_cast<unsigned>(u.getCharAt(i)));
        return s.empty() ? std::string("-") : s;
    }());
    blob_rows(count);
    for (int i = 0; i < count; ++i) {
        row(fmt("= xp %d ", i) + digest_or_full(snapshot_hex(ThePlayerList->getNthPlayer(i)), full));
    }
    row("= xl " + digest_or_full(snapshot_hex(ThePlayerList), false));
    row("= xt " + digest_or_full(snapshot_hex(TheTeamFactory), full));
    row(fmt("= crc %08x", oracle_wp51::World::snapshot_crc(ThePlayerList)));
}

static void check_sides() {
    std::string s = "= sides teams=";
    for (int i = 0; i < TheSidesList->getNumTeams(); ++i) {
        Dict* d = TheSidesList->getTeamInfo(i)->getDict();
        s += fmt("%s:%s:%s:%s,", d->getAsciiString(TheKey_teamName).str(), d->getAsciiString(TheKey_teamOwner).str(),
                 d->getAsciiString(TheKey_teamOnCreateScript).str(),
                 d->getAsciiString(NAMEKEY("teamGenericScriptHook3")).str());
    }
    s += " scripts=";
    for (int i = 0; i < TheSidesList->getNumSides(); ++i) {
        s += TheSidesList->getSideInfo(i)->getScriptList() ? '1' : '0';
    }
    s += " skscripts=";
    for (int i = 0; i < TheSidesList->getNumSkirmishSides(); ++i) {
        s += TheSidesList->getSkirmishSideInfo(i)->getScriptList() ? '1' : '0';
    }
    s += " build=";
    for (int i = 0; i < TheSidesList->getNumSides(); ++i) {
        s += TheSidesList->getSideInfo(i)->getBuildList() ? '1' : '0';
    }
    row(s);
}

// ---- one case -----------------------------------------------------------------

static const char* const FACTIONS[] = {"FactionCivilian", "FactionAmerica", "FactionChina", "FactionGLA",
                                       "FactionObserver", "FactionNope"};

static std::string player_list_text(const std::vector<std::string>& names, int self) {
    std::string s;
    for (size_t k = 0; k < names.size(); ++k) {
        if (chance(35)) s += names[k] + (chance(20) ? "  " : " ");
    }
    if (chance(10)) s += "PlyrNobody";
    (void)self;
    return s;
}

static std::vector<std::string> g_waypoints;

static void build_waypoints(int c) {
    Waypoint* head = nullptr;
    g_waypoints.clear();
    const int n = pick(5);
    for (int i = 0; i < n; ++i) {
        const std::string name = fmt("Way%u", pick(3));
        Coord3D at;
        at.x = static_cast<Real>(pick(100000)) / 7.0f;
        at.y = static_cast<Real>(pick(100000)) / 11.0f;
        at.z = static_cast<Real>(pick(1000)) / 3.0f;
        Waypoint* w = newInstance(Waypoint)(static_cast<WaypointID>(i + 1 + c), AsciiString(name.c_str()), &at,
                                            AsciiString::TheEmptyString, AsciiString::TheEmptyString,
                                            AsciiString::TheEmptyString, FALSE);
        w->setNext(head);
        head = w;
    }
    TheTerrainLogic->*access_member(PtWaypointTag()) = head;
    for (Waypoint* w = head; w; w = w->getNext()) {
        row(fmt("WP %u %s %08x %08x %08x", static_cast<unsigned>(w->getID()), w->getName().str(),
                oracle::fbits(w->getLocation()->x), oracle::fbits(w->getLocation()->y),
                oracle::fbits(w->getLocation()->z)));
    }
}

static Spec team_spec(int c, const std::string& name, const std::string& owner, bool singleton) {
    Spec t;
    t.s("teamName", name);
    t.s("teamOwner", owner);
    if (singleton || chance(50)) t.b("teamIsSingleton", singleton);
    for (int u = 1; u <= 7; ++u) {
        if (chance(10)) t.i(fmt("teamUnitMinCount%d", u), pick(4));
        if (chance(15)) t.i(fmt("teamUnitMaxCount%d", u), static_cast<Int>(pick(5)) - 1);
        if (chance(12)) t.s(fmt("teamUnitType%d", u), chance(80) ? fmt("Unit%d", u) : "");
    }
    if (chance(30)) t.s("teamHome", fmt("Way%u", pick(4)));
    // Script names do not resolve: only the state around them is observable.
    if (chance(15)) t.s("teamOnCreateScript", "ScrCreate");
    if (chance(15)) t.s("teamOnIdleScript", "ScrIdle");
    if (chance(15)) t.s("teamEnemySightedScript", "ScrSighted");
    if (chance(15)) t.s("teamAllClearScript", "ScrClear");
    if (chance(15)) t.s("teamOnUnitDestroyedScript", "ScrUnitDead");
    if (chance(30)) t.s("teamOnDestroyedScript", "ScrDead");
    if (chance(30)) t.r("teamDestroyedThreshold", static_cast<Real>(pick(1200)) / 1000.0f - 0.1f);
    if (chance(20)) t.i("teamInitialIdleFrames", pick(300));
    const char* flags[] = {"teamIsAIRecruitable", "teamIsBaseDefense", "teamIsPerimeterDefense", "teamAutoReinforce",
                           "teamTransportsReturn", "teamAvoidThreats", "teamAttackCommonTarget", "teamStartsFull",
                           "teamTransportsExit"};
    for (const char* f : flags) {
        if (chance(15)) t.b(f, chance(60));
    }
    if (chance(15)) t.b("teamExecutesActionsOnCreate", chance(60));
    if (chance(15)) t.s("teamProductionCondition", "ScrCondition");
    if (chance(20)) t.i("teamAggressiveness", static_cast<Int>(pick(5)) - 2);
    if (chance(20)) t.i("teamMaxInstances", pick(9));
    if (chance(30)) t.i("teamProductionPriority", static_cast<Int>(pick(200)) - 50);
    if (chance(15)) t.i("teamProductionPrioritySuccessIncrease", pick(9));
    if (chance(15)) t.i("teamProductionPriorityFailureDecrease", pick(9));
    if (chance(15)) t.s("teamTransport", "Chopper");
    if (chance(15)) t.s("teamReinforcementOrigin", "Way1");
    if (chance(15)) t.i("teamVeterancy", pick(4));
    for (int g = 0; g < 16; ++g) {
        if (chance(3)) t.s(fmt("teamGenericScriptHook%d", g), "ScrGeneric");
    }
    return t;
}

static std::vector<Object*> g_objects;

static void add_build_list(int side) {
    const int n = chance(25) ? 1 + pick(3) : 0;
    for (int b = 0; b < n; ++b) {
        BuildListInfo* info = newInstance(BuildListInfo);
        const std::string name = fmt("Building%u", pick(5));
        Coord3D at;
        at.x = static_cast<Real>(pick(100000)) / 13.0f;
        at.y = static_cast<Real>(pick(100000)) / 17.0f;
        at.z = 0.0f;
        const Real angle = static_cast<Real>(pick(6283)) / 1000.0f;
        const Int rebuilds = pick(4);
        info->setTemplateName(AsciiString(name.c_str()));
        info->setLocation(at);
        info->setAngle(angle);
        info->setNumRebuilds(rebuilds);
        TheSidesList->getSideInfo(side)->addToBuildList(info, 1000);
        row(fmt("B %d %s %08x %08x %08x %d", side, name.c_str(), oracle::fbits(at.x), oracle::fbits(at.y),
                oracle::fbits(angle), rebuilds));
    }
}

static void emit_dict(const char* tag, const Spec& s) {
    row(std::string(tag) + " " + s.text);
}

static void run_case(int c) {
    const bool full = c < 2;
    const std::string suffix = fmt("c%d", c);
    row(fmt("G %d", c));

    // Settings.
    static SkirmishGameInfo* info = nullptr;
    delete info;
    info = nullptr;
    TheGameInfo = nullptr;
    unsigned slots = 0;
    UnsignedInt cash = 0;
    if (chance(20)) {
        info = new SkirmishGameInfo;
        info->init();
        slots = 1 + pick(255);
        cash = pick(3) * 5000;
        for (int i = 0; i < MAX_SLOTS; ++i) {
            GameSlot slot;
            slot.setState((slots >> i) & 1 ? SLOT_EASY_AI : SLOT_OPEN);
            info->setSlot(i, slot);
        }
        Money m;
        if (cash) m.deposit(cash, FALSE, FALSE);
        info->setStartingCash(m);
        TheGameInfo = info;
        for (int i = 0; i < MAX_SLOTS; ++i) {
            if (info->getConstSlot(i)->isOccupied() != (((slots >> i) & 1) != 0)) {
                std::fprintf(stderr, "slot state\n");
                std::exit(2);
            }
        }
    } else {
        cash = TheGlobalData->m_defaultStartingCash.countMoney();
    }
    const UnsignedInt frame = pick(5000);
    TheGameLogic->*access_member(Wp51GLFrameTag()) = frame;
    row(fmt("INFO %u %u %u %d %d", slots, cash, frame, TheNetwork ? 1 : 0,
            static_cast<int>(TheScriptEngine->getGlobalDifficulty())));
    row(fmt("AIDATA %d", TheAI->getAiData()->m_forceSkirmishAI ? 1 : 0));
    build_waypoints(c);

    // Sides.
    TheSidesList->emptySides();
    TheSidesList->emptyTeams();
    const bool skirmish = chance(35);
    std::vector<std::string> names;
    std::vector<int> factions;
    const int named = chance(5) ? 14 : 1 + static_cast<int>(pick(6));
    for (int i = 0; i < named; ++i) {
        const int f = chance(4) ? 5 : static_cast<int>(pick(5));
        factions.push_back(f);
        names.push_back(fmt("P%d_%s", i, suffix.c_str()));
    }
    const bool no_human = chance(8);
    if (slots) {
        std::vector<int> free_slots = {0, 1, 2, 3, 4, 5, 6, 7};
        for (int k = 0; k < named && k < 8; ++k) {
            const uint32_t at = pick(static_cast<uint32_t>(free_slots.size()));
            if (chance(70)) names[k] = fmt("player%d", free_slots[at]);
            free_slots.erase(free_slots.begin() + at);
        }
    }
    const int neutral_at = chance(12) ? static_cast<int>(pick(named + 1)) : 0;
    for (int i = 0; i <= named; ++i) {
        Spec s;
        const int k = i < neutral_at ? i : i - 1;
        if (i == neutral_at) {
            s.s("playerName", "");
            s.u("playerDisplayName", "Neutral");
            s.b("playerIsHuman", false);
            s.s("playerFaction", "");
            s.s("playerAllies", "");
            s.s("playerEnemies", chance(10) ? names[0] : std::string());
        } else {
            s.s("playerName", names[k]);
            s.u("playerDisplayName", fmt("Disp%d", k));
            s.s("playerFaction", FACTIONS[factions[k]]);
            if (!no_human && chance(40)) s.b("playerIsHuman", true);
            else if (chance(50)) s.b("playerIsHuman", false);
            s.s("playerAllies", player_list_text(names, k));
            s.s("playerEnemies", player_list_text(names, k));
            // A computer civilian skirmish side would find no script list.
            if (skirmish && factions[k] != 0 && chance(75)) s.b("playerIsSkirmish", true);
            if (chance(15)) s.b("playerIsPreorder", true);
            if (chance(40)) s.i("multiplayerStartIndex", pick(8));
            if (chance(20)) s.i("skirmishDifficulty", pick(3));
            if (chance(25)) s.i("playerColor", static_cast<Int>(pick(0xffffff)));
            if (chance(15)) s.i("playerNightColor", static_cast<Int>(pick(0xffffff)));
            if (chance(20)) s.i("playerStartMoney", chance(10) ? 0 : static_cast<Int>(pick(20000)));
            if (chance(4)) s.b("multiplayerIsLocal", chance(80));
            const char* handicaps[] = {"HANDICAP_BUILDCOST_GENERIC", "HANDICAP_BUILDCOST_BUILDINGS",
                                       "HANDICAP_BUILDTIME_GENERIC", "HANDICAP_BUILDTIME_BUILDINGS"};
            for (const char* h : handicaps) {
                if (chance(4)) s.r(h, static_cast<Real>(pick(300)) / 100.0f);
            }
        }
        TheSidesList->addSide(&s.dict);
        emit_dict("S", s);
        add_build_list(i);
    }

    // Teams: the default team of every side, then others.
    int serial = 0;
    for (int i = 0; i <= named; ++i) {
        const std::string owner = i == 0 ? std::string() : names[i - 1];
        if (i != 0 && chance(3)) continue;
        Spec t = team_spec(c, "team" + owner, owner, true);
        TheSidesList->addTeam(&t.dict);
        emit_dict("T", t);
    }
    std::vector<std::string> team_names;
    const int extra = chance(10) ? 0 : static_cast<int>(pick(10));
    for (int i = 0; i < extra; ++i) {
        const std::string owner = chance(8) ? "" : chance(5) ? "PlyrNobody" : names[pick(named)];
        const std::string name = (i > 1 && chance(6)) ? team_names[pick(static_cast<uint32_t>(team_names.size()))]
                                                      : fmt("Team%d_%s", serial++, suffix.c_str());
        team_names.push_back(name);
        Spec t = team_spec(c, name, owner, chance(35));
        TheSidesList->addTeam(&t.dict);
        emit_dict("T", t);
    }

    // Skirmish sides and teams.
    if (skirmish) {
        std::vector<std::string> skirmish_names;
        for (int f = 0; f < 4; ++f) {
            if (!chance(f == 0 ? 85 : 75)) continue;
            Spec s;
            const std::string name = fmt("Sk%d_%s", f, suffix.c_str());
            skirmish_names.push_back(name);
            s.s("playerName", name);
            s.s("playerFaction", FACTIONS[f]);
            TheSidesList->addSkirmishSide(&s.dict);
            emit_dict("SK", s);
            const int index = TheSidesList->getNumSkirmishSides() - 1;
            TheSidesList->getSkirmishSideInfo(index)->setScriptList(newInstance(ScriptList));
            row(fmt("SS %d", index));
        }
        const int count = skirmish_names.empty() ? 0 : static_cast<int>(pick(8));
        for (int i = 0; i < count; ++i) {
            const std::string owner = skirmish_names[pick(static_cast<uint32_t>(skirmish_names.size()))];
            const std::string name = chance(30) ? "team" + owner : fmt("SkTeam%d_%s", i, suffix.c_str());
            Spec t = team_spec(c, name, owner, chance(40));
            TheSidesList->addSkirmishTeam(&t.dict);
            emit_dict("ST", t);
        }
    }

    // Objects of the previous case stay teamless once their teams go.
    ThePlayerList->newGame();
    g_objects.clear();
    row("newgame");
    check_sides();
    check_players(full);
    check_teams();
    check_saves(full);

    // ---- operations -------------------------------------------------------
    const Int count = ThePlayerList->getPlayerCount();
    auto any_player = [&]() { return ThePlayerList->getNthPlayer(pick(count)); };
    auto any_team = [&]() -> Team* {
        const std::vector<Team*> teams = live_teams();
        return teams.empty() ? nullptr : teams[pick(static_cast<uint32_t>(teams.size()))];
    };
    auto any_name = [&]() -> std::string {
        if (!team_names.empty() && chance(75)) return team_names[pick(static_cast<uint32_t>(team_names.size()))];
        return chance(80) ? "team" + names[pick(named)] : std::string("TeamNone_") + suffix;
    };
    auto id_of = [](Team* t) { return t ? static_cast<int>(t->getID()) : 0; };

    const int rounds = 1 + pick(3);
    for (int round = 0; round < rounds; ++round) {
        const int ops = 6 + pick(18);
        for (int op = 0; op < ops; ++op) {
            const int kind = pick(30);
            const Relationship r = static_cast<Relationship>(pick(3));
            Team* team = any_team();
            Team* other = any_team();
            if (!team || !other) break;
            Player* p = any_player();
            Player* q = any_player();
            const int tid = id_of(team);
            switch (kind) {
                case 0:
                    p->setPlayerRelationship(q, r);
                    row(fmt("op prel %d %d %d", p->getPlayerIndex(), q->getPlayerIndex(), r));
                    break;
                case 1: {
                    const bool all = chance(15);
                    const Bool done = p->removePlayerRelationship(all ? nullptr : q);
                    row(fmt("op prem %d %d", p->getPlayerIndex(), all ? -1 : q->getPlayerIndex()));
                    row(fmt("= done %d", done));
                    break;
                }
                case 2:
                    p->setTeamRelationship(team, r);
                    row(fmt("op trel %d %d %d", p->getPlayerIndex(), tid, r));
                    break;
                case 3: {
                    const bool all = chance(15);
                    const Bool done = p->removeTeamRelationship(all ? nullptr : team);
                    row(fmt("op trem %d %d", p->getPlayerIndex(), all ? 0 : tid));
                    row(fmt("= done %d", done));
                    break;
                }
                case 4:
                    team->setOverrideTeamRelationship(other->getID(), r);
                    row(fmt("op ovt %d %d %d", tid, id_of(other), r));
                    break;
                case 5: {
                    const int target = chance(15) ? 0 : id_of(other);
                    const Bool done = team->removeOverrideTeamRelationship(static_cast<TeamID>(target));
                    row(fmt("op ovtr %d %d", tid, target));
                    row(fmt("= done %d", done));
                    break;
                }
                case 6:
                    team->setOverridePlayerRelationship(q->getPlayerIndex(), r);
                    row(fmt("op ovp %d %d %d", tid, q->getPlayerIndex(), r));
                    break;
                case 7: {
                    const int target = chance(15) ? -1 : q->getPlayerIndex();
                    const Bool done = team->removeOverridePlayerRelationship(target);
                    row(fmt("op ovpr %d %d", tid, target));
                    row(fmt("= done %d", done));
                    break;
                }
                case 8: {
                    const std::string name = any_name();
                    const bool active = chance(60);
                    Team* made = nullptr;
                    if (TheTeamFactory->findTeamPrototype(AsciiString(name.c_str()))) {
                        made = active ? TheTeamFactory->createTeam(AsciiString(name.c_str()))
                                      : TheTeamFactory->createInactiveTeam(AsciiString(name.c_str()));
                    }
                    row(fmt("op make %s %d", name.c_str(), active));
                    row(fmt("= team %d", id_of(made)));
                    break;
                }
                case 9: {
                    const std::string name = any_name();
                    row(fmt("op find %s", name.c_str()));
                    row(fmt("= team %d", id_of(TheTeamFactory->findTeam(AsciiString(name.c_str())))));
                    break;
                }
                case 10: {
                    const std::vector<Int> ids = proto_ids();
                    const Int id = ids[pick(static_cast<uint32_t>(ids.size()))];
                    TeamPrototype* proto = TheTeamFactory->findTeamPrototypeByID(static_cast<TeamPrototypeID>(id));
                    row(fmt("op onproto %d", id));
                    row(fmt("= team %d", id_of(proto ? TheTeamFactory->createTeamOnPrototype(proto) : nullptr)));
                    break;
                }
                case 11:
                    team->setControllingPlayer(q);
                    row(fmt("op move %d %d", tid, q->getPlayerIndex()));
                    break;
                case 12:
                case 13: {
                    Coord3D at;
                    at.x = 40.0f + static_cast<Real>(g_objects.size() % 30) * 30.0f;
                    at.y = 40.0f + static_cast<Real>(g_objects.size() / 30 % 30) * 30.0f;
                    at.z = 0.0f;
                    Object* o = TheThingFactory->newObject(TheThingFactory->findTemplate("PtUnit"), team);
                    o->setPosition(&at);
                    g_objects.push_back(o);
                    row(fmt("op spawn %zu %d %d", g_objects.size() - 1, static_cast<int>(o->getID()), tid));
                    break;
                }
                case 14:
                    if (!g_objects.empty()) {
                        const uint32_t k = pick(static_cast<uint32_t>(g_objects.size()));
                        g_objects[k]->setTeam(team);
                        row(fmt("op setteam %u %d", k, tid));
                    }
                    break;
                case 15:
                    if (!g_objects.empty()) {
                        const uint32_t k = pick(static_cast<uint32_t>(g_objects.size()));
                        g_objects[k]->setEffectivelyDead(TRUE);
                        row(fmt("op dead %d", static_cast<int>(g_objects[k]->getID())));
                    }
                    break;
                case 16: {
                    const std::string state = fmt("State%u", pick(5));
                    const int what = pick(4);
                    const int value = chance(50) ? 1 : 0;
                    if (what == 0) team->setState(AsciiString(state.c_str()));
                    if (what == 1) team->setRecruitable(value != 0);
                    if (what == 2) team->setActive();
                    if (what == 3) team->setEnteredExited();
                    row(fmt("op set %d %d %s %d", tid, what, state.c_str(), value));
                    break;
                }
                case 17: {
                    const std::vector<Team*> teams = live_teams();
                    // Neither a singleton nor a default team may be deleted.
                    bool is_default = false;
                    for (int i = 0; i < MAX_PLAYER_COUNT; ++i) {
                        is_default = is_default || ThePlayerList->getNthPlayer(i)->getDefaultTeam() == team;
                    }
                    if (!is_default && !team->getPrototype()->getIsSingleton()) {
                        row(fmt("op delete %d", tid));
                        TheTeamFactory->teamAboutToBeDeleted(team);
                        deleteInstance(team);
                    }
                    break;
                }
                case 18: {
                    const UnsignedInt till = chance(50) ? pick(6000) : 0;
                    p->getEnergy()->setPowerSabotagedTillFrame(till);
                    row(fmt("op sabotage %d %u", p->getPlayerIndex(), till));
                    break;
                }
                case 19: {
                    const UnsignedInt amount = pick(5000);
                    p->getMoney()->deposit(amount, FALSE, TRUE);
                    row(fmt("op income %d %u", p->getPlayerIndex(), amount));
                    break;
                }
                case 20: {
                    const Int delta = static_cast<Int>(pick(9)) - 4;
                    p->addSciencePurchasePoints(delta);
                    row(fmt("op spp %d %d", p->getPlayerIndex(), delta));
                    break;
                }
                case 21: {
                    BattlePlanBonuses*& bonus = p->*access_member(PtBonusTag());
                    deleteInstance(bonus);
                    bonus = newInstance(BattlePlanBonuses);
                    bonus->m_armorScalar = static_cast<Real>(pick(1000)) / 333.0f;
                    bonus->m_sightRangeScalar = static_cast<Real>(pick(1000)) / 77.0f;
                    bonus->m_bombardment = pick(3);
                    bonus->m_holdTheLine = pick(3);
                    bonus->m_searchAndDestroy = pick(3);
                    bonus->m_validKindOf.clear();
                    bonus->m_invalidKindOf.clear();
                    const int valid = pick(KINDOF_COUNT);
                    const int invalid = pick(KINDOF_COUNT);
                    bonus->m_validKindOf.set(valid);
                    bonus->m_invalidKindOf.set(invalid);
                    row(fmt("op bonus %d %08x %08x %d %d %d %d %d", p->getPlayerIndex(),
                            oracle::fbits(bonus->m_armorScalar), oracle::fbits(bonus->m_sightRangeScalar),
                            bonus->m_bombardment, bonus->m_holdTheLine, bonus->m_searchAndDestroy, valid, invalid));
                    break;
                }
                case 22: {
                    unsigned mask = pick(1u << MAX_PLAYER_COUNT);
                    PlayerMaskType adjusted = static_cast<PlayerMaskType>(mask);
                    Player* each = adjusted ? ThePlayerList->getEachPlayerFromMask(adjusted) : nullptr;
                    const unsigned one = 1u << pick(MAX_PLAYER_COUNT);
                    row(fmt("op mask %u %u", mask, one));
                    row(fmt("= mask %d %u %d", each ? each->getPlayerIndex() : -1, static_cast<unsigned>(adjusted),
                            ThePlayerList->getPlayerFromMask(static_cast<PlayerMaskType>(one))->getPlayerIndex()));
                    break;
                }
                case 23: {
                    const std::string name = any_name();
                    row(fmt("op validate %s", name.c_str()));
                    row(fmt("= team %d", id_of(ThePlayerList->validateTeam(AsciiString(name.c_str())))));
                    break;
                }
                case 24: {
                    const int what = pick(6);
                    const int value = pick(MAX_PLAYER_COUNT);
                    if (what == 0) p->setCashBounty(static_cast<Real>(value) / 7.0f);
                    if (what == 1) p->setSkillPointsModifier(static_cast<Real>(value) / 3.0f);
                    if (what == 2) p->setListInScoreScreen(value & 1);
                    if (what == 3) p->setAttackedBy(value);
                    if (what == 4) p->*access_member(PtHuntTag()) = (value & 1) != 0;
                    if (what == 5) p->setLogicalRetaliationModeEnabled(value & 1);
                    row(fmt("op poke %d %d %d", p->getPlayerIndex(), what, value));
                    break;
                }
                case 25: {
                    const int kind_bit = pick(KINDOF_COUNT);
                    const Real percent = static_cast<Real>(static_cast<int>(pick(200)) - 100) / 128.0f;
                    KindOfMaskType mask;
                    mask.set(kind_bit);
                    p->addKindOfProductionCostChange(mask, percent);
                    row(fmt("op kindof %d %d %08x", p->getPlayerIndex(), kind_bit, oracle::fbits(percent)));
                    break;
                }
                case 26: {
                    SpecialPowerReadyTimerType timer;
                    timer.m_templateID = pick(100);
                    timer.m_readyFrame = pick(100000);
                    (p->*access_member(PtTimerTag())).push_back(timer);
                    row(fmt("op timer %d %u %u", p->getPlayerIndex(), timer.m_templateID, timer.m_readyFrame));
                    break;
                }
                case 27: {
                    const UpgradeTemplate* up = TheUpgradeCenter->findUpgrade(AsciiString(chance(50) ? "PtUpA" : "PtUpB"));
                    const bool done = chance(50);
                    (p->*(done ? access_member(PtUpDoneTag()) : access_member(PtUpBusyTag()))).set(up->getUpgradeMask());
                    row(fmt("op upgrade %d %d %s", p->getPlayerIndex(), done, up->getUpgradeName().str()));
                    break;
                }
                case 28: {
                    const int squad = pick(10);
                    const int id = 1 + pick(5000);
                    p->getHotkeySquad(squad)->addObjectID(static_cast<ObjectID>(id));
                    row(fmt("op squad %d %d %d", p->getPlayerIndex(), squad, id));
                    break;
                }
                case 29: {
                    const char* name = chance(50) ? "SCIENCE_PtA" : "SCIENCE_PtB";
                    const int avail = pick(3);
                    p->setScienceAvailability(TheScienceStore->getScienceFromInternalName(AsciiString(name)),
                                              static_cast<ScienceAvailabilityType>(avail));
                    row(fmt("op avail %d %s %d", p->getPlayerIndex(), name, avail));
                    break;
                }
            }
        }
        // The update passes.
        if (chance(80)) {
            ThePlayerList->updateTeamStates();
            row("op update_teams");
        }
        if (chance(60)) {
            const UnsignedInt at = pick(7000);
            TheGameLogic->*access_member(Wp51GLFrameTag()) = at;
            // The AI players run in another module.
            std::vector<AIPlayer*> ai(MAX_PLAYER_COUNT, nullptr);
            for (int i = 0; i < MAX_PLAYER_COUNT; ++i) {
                std::swap(ai[i], ThePlayerList->getNthPlayer(i)->*access_member(PtAiTag()));
            }
            ThePlayerList->update();
            for (int i = 0; i < MAX_PLAYER_COUNT; ++i) {
                std::swap(ai[i], ThePlayerList->getNthPlayer(i)->*access_member(PtAiTag()));
            }
            row(fmt("op update %u", at));
        }
        check_teams();
        check_saves(false);
    }
    // The id counters rebuilt as after a load; later cases continue from them.
    if (chance(10)) {
        (static_cast<Snapshot*>(TheTeamFactory)->*access_member(PtPostTag()))();
        row("op postprocess");
    }
}

static const char* const RULES =
    "Science SCIENCE_PtA\n  SciencePurchasePointCost = 1\n  IsGrantable = Yes\nEnd\n"
    "Science SCIENCE_PtB\n  SciencePurchasePointCost = 1\n  IsGrantable = Yes\nEnd\n"
    "Science SCIENCE_PtRank\n  SciencePurchasePointCost = 0\n  IsGrantable = Yes\nEnd\n"
    "Rank 1\n  SkillPointsNeeded = 0\n  SciencesGranted = SCIENCE_PtRank\n  SciencePurchasePointsGranted = 1\nEnd\n"
    "Rank 2\n  SkillPointsNeeded = 800\n  SciencePurchasePointsGranted = 2\nEnd\n"
    "PlayerTemplate FactionCivilian\n  Side = Civilian\n  PreferredColor = R:100 G:101 B:102\nEnd\n"
    "PlayerTemplate FactionAmerica\n  Side = America\n  BaseSide = USA\n  PreferredColor = R:1 G:2 B:255\n"
    "  IntrinsicSciences = SCIENCE_PtA SCIENCE_PtRank\n  IntrinsicSciencePurchasePoints = 3\n"
    "  ProductionCostChange = PtUnit -10%\n  ProductionTimeChange = PtUnit 25%\n"
    "  ProductionVeterancyLevel = PtUnit VETERAN\nEnd\n"
    "PlayerTemplate FactionChina\n  Side = China\n  BaseSide = China\n  StartMoney = 7000\n"
    "  PreferredColor = R:255 G:2 B:3\n  IntrinsicSciences = SCIENCE_PtB\nEnd\n"
    "PlayerTemplate FactionGLA\n  Side = GLA\n  BaseSide = GLA\n  PreferredColor = R:9 G:200 B:8\nEnd\n"
    "PlayerTemplate FactionObserver\n  Side = Observer\n  IsObserver = Yes\n  PreferredColor = R:7 G:7 B:7\nEnd\n";

static void load_rules(oracle_wp51::World& w, const std::string& text) {
    w.mem_ini.parse_blocks(text, INI_LOAD_OVERWRITE, [](INI* ini, const char* tok) {
        if (std::strcmp(tok, "Science") == 0) {
            INI::parseScienceDefinition(ini);
        } else if (std::strcmp(tok, "Rank") == 0) {
            INI::parseRankDefinition(ini);
        } else if (std::strcmp(tok, "PlayerTemplate") == 0) {
            INI::parsePlayerTemplateDefinition(ini);
        }
    });
}

// ---- retail maps ---------------------------------------------------------------

// A player's save without the parts other systems own.
static std::string bare_player_hex(Player* p) {
    AIPlayer* ai = p->*access_member(PtAiTag());
    ResourceGatheringManager* rgm = p->*access_member(Wp51PlayerRgmTag());
    TunnelTracker* tunnel = p->*access_member(Wp51PlayerTunnelTag());
    p->*access_member(PtAiTag()) = nullptr;
    p->*access_member(Wp51PlayerRgmTag()) = nullptr;
    p->*access_member(Wp51PlayerTunnelTag()) = nullptr;
    const std::string hex = snapshot_hex(p);
    p->*access_member(PtAiTag()) = ai;
    p->*access_member(Wp51PlayerRgmTag()) = rgm;
    p->*access_member(Wp51PlayerTunnelTag()) = tunnel;
    return hex;
}

static std::string retail_digest(const std::string& path) {
    // The device loader reads the map objects the waypoints come from.
    TheTerrainLogic->reset();
    if (!TheTerrainLogic->loadMap(AsciiString(path.c_str()), TRUE)) {
        std::fprintf(stderr, "loadMap failed: %s\n", path.c_str());
        std::exit(1);
    }
    TheSidesList->emptySides();
    TheSidesList->emptyTeams();
    {
        CachedFileInputStream stream;
        if (!stream.open(AsciiString(path.c_str()))) std::exit(1);
        DataChunkInput file(&stream);
        file.registerParser("SidesList", AsciiString::TheEmptyString, SidesList::ParseSidesDataChunk);
        if (!file.parse(nullptr)) std::exit(1);
    }
    ThePlayerList->newGame();
    g_rows.clear();
    check_sides();
    {
        // Script lists are the sides list's own concern.
        std::string& s = g_rows.back();
        const size_t from = s.find(" scripts=");
        const size_t to = s.find(" build=");
        s.erase(from, to - from);
    }
    check_players(false);
    check_teams();
    std::string saves;
    for (int i = 0; i < ThePlayerList->getPlayerCount(); ++i) {
        saves += bare_player_hex(ThePlayerList->getNthPlayer(i)) + ";";
    }
    row("= xp " + saves);
    row("= xt " + snapshot_hex(TheTeamFactory));
    row(fmt("= crc %08x", oracle_wp51::World::snapshot_crc(ThePlayerList)));

    const char* const sections[] = {"sides", "list", "player", "tpl", "inst", "masks", "trel", "xp", "xt", "crc"};
    std::string out = fmt("players=%d teams=%zu protos=%zu", ThePlayerList->getPlayerCount(), live_teams().size(),
                          proto_ids().size());
    for (const char* section : sections) {
        const std::string prefix = std::string("= ") + section + " ";
        std::string joined;
        for (const std::string& r : g_rows) {
            if (r.compare(0, prefix.size(), prefix) == 0) joined += r + "\n";
        }
        out += fmt(" %s=%016llx", section, static_cast<unsigned long long>(fnv(joined)));
    }
    g_rows.clear();
    return out;
}

static void load_file(oracle_wp51::World& w, const std::string& path) {
    std::ifstream in(path, std::ios::binary);
    if (!in) return;
    const std::string text((std::istreambuf_iterator<char>(in)), std::istreambuf_iterator<char>());
    load_rules(w, text);
}

static void write_rows(const char* path, const char* name) {
    std::string out = std::string("{\n \"") + name + "\": [";
    for (size_t i = 0; i < g_rows.size(); ++i) {
        out += std::string(i ? "," : "") + "\n  \"" + g_rows[i] + "\"";
    }
    out += "\n ]\n}\n";
    oracle::write_file_atomic(path, out);
}

int main(int argc, char** argv) {
    const bool retail = argc == 5 && std::strcmp(argv[1], "--retail") == 0;
    if (!retail && argc != 2) {
        std::fprintf(stderr, "usage: gen out.json | gen --retail list out.json ini-dir\n");
        return 1;
    }
    static oracle_wp51::World w;
    w.load_ini("Object PtUnit\n  KindOf = SELECTABLE\n  VisionRange = 0\n  ShroudClearingRange = 0\n  Geometry = CYLINDER\n  GeometryMajorRadius = 1\n"
               "  GeometryHeight = 5\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\nEnd\n");
    if (retail) {
        TheFileSystem = NEW FileSystem;
        TheLocalFileSystem = NEW StdLocalFileSystem;
        TheGameText = new PtGameText;
        TheTerrainTypes = NEW TerrainTypeCollection;
        static void* map_cache[64];
        TheMapCache = reinterpret_cast<MapCache*>(map_cache);
        TheTerrainLogic = NEW W3DTerrainLogic;
        const std::string dir = argv[4];
        for (const char* name : {"default_science", "science", "rank", "default_playertemplate", "playertemplate"}) {
            load_file(w, dir + "/" + name + ".ini");
        }
    } else {
        load_rules(w, RULES);
        w.load_ini("Upgrade PtUpA\nEnd\nUpgrade PtUpB\nEnd\n");
        for (UpgradeTemplate* u = TheUpgradeCenter->firstUpgradeTemplate(); u; u = u->friend_getNext()) {
            const UpgradeMaskType mask = u->getUpgradeMask();
            for (int bit = 0; bit < static_cast<int>(mask.size()); ++bit) {
                if (mask.test(bit)) row(fmt("UPG %s %d", u->getUpgradeName().str(), bit));
            }
        }
    }
    // The world gave players relations: start from the maps the constructors make.
    for (int i = 0; i < MAX_PLAYER_COUNT; ++i) {
        Player* p = ThePlayerList->getNthPlayer(i);
        (p->*access_member(PtPlayerRelTag()))->m_map = PlayerRelationMapType();
        (p->*access_member(PtTeamRelTag()))->m_map = TeamRelationMapType();
    }
    ThePlayerList->init();
    // The world may have made teams: start the id counters from zero.
    TheTeamFactory->reset();
    if (retail) {
        std::ifstream list(argv[2]);
        std::string key, path;
        std::vector<std::string> out;
        const UnicodeString general = TheGameText->fetch("SCIENCE:GeneralName");
        if (!general.isEmpty()) std::exit(1);
        while (list >> key >> path) {
            std::fprintf(stderr, "map %zu\n", out.size());
            const std::string digest = retail_digest(path);
            out.push_back(key + " " + digest);
        }
        for (const std::string& r : out) row(r);
        write_rows(argv[3], "maps");
        return out.empty() ? 1 : 0;
    }
    {
        std::string line;
        for (const char* c = RULES; *c; ++c) {
            if (*c == '\n') {
                row("INI " + line);
                line.clear();
            } else {
                line += *c;
            }
        }
    }
    const int cases = 220;
    for (int c = 0; c < cases; ++c) {
        g_state = 0x5eed0000ull + static_cast<uint64_t>(c) * 0x9e3779b97f4a7c15ull;
        run_case(c);
    }
    // The relation maps' iteration order, through growth, removal and clear.
    for (int m = 0; m < 40; ++m) {
        g_state = 0x7e1a0000ull + static_cast<uint64_t>(m) * 0x9e3779b97f4a7c15ull;
        TeamRelationMap* map = newInstance(TeamRelationMap);
        row("RM new");
        const uint32_t span = m % 2 ? 40u : 100000u;
        const int ops = 20 + pick(120);
        for (int i = 0; i < ops; ++i) {
            const uint32_t kind = pick(100);
            if (kind < 2) {
                map->m_map.clear();
                row("RM clear");
            } else if (kind < 25 && !map->m_map.empty()) {
                const TeamID key = map->m_map.begin()->first;
                const TeamID other = chance(50) ? key : 1 + pick(span);
                map->m_map.erase(other);
                row(fmt("RM remove %u", other));
            } else {
                const TeamID key = 1 + pick(span);
                const int r = pick(3);
                map->m_map[key] = static_cast<Relationship>(r);
                row(fmt("RM set %u %d", key, r));
            }
            if (chance(15) || i + 1 == ops) {
                const std::string hex = snapshot_hex(static_cast<Snapshot*>(map));
                row("= rm " + digest_or_full(hex, m < 2 && i + 1 == ops));
            }
        }
        deleteInstance(map);
    }
    std::fprintf(stderr, "%d cases, %zu rows\n", cases, g_rows.size());
    write_rows(argv[1], "rows");
    return 0;
}
