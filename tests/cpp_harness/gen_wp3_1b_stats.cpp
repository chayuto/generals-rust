// WP-3.1b (PKT-117 BattleHonors, PKT-122 ScoreKeeper, PKT-123 SkirmishBattleHonors,
// PKT-124 StatsCollector) C++ Golden Vector Generator (Zero-Stub Build).
// Compiles directly against pre-built vendor/GeneralsX/build/macos-vulkan via scripts/build_oracle.py.
//
// Canonical build & run command (from worktree):
//   python3 scripts/build_oracle.py "$(pwd)/tests/cpp_harness/gen_wp3_1b_stats.cpp" -o /tmp/gen_wp3_1b_stats \
//     --repo-root /Users/chayut/repos/generals-rust --extra-flag "-I$(pwd)/tests/cpp_harness"
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy /tmp/gen_wp3_1b_stats docs/golden_vectors/WP3_1b_Stats.json
//
// Determinism & UB verification builds (must emit byte-identical JSON):
//   --opt O0, --opt O1, --opt O2, and --sanitize (UBSan).

#include "oracle_common.h"

#include <array>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <sys/stat.h>
#include <unistd.h>
#include <vector>

#include "Common/BattleHonors.h"
#include "Common/Dict.h"
#include "Common/FileSystem.h"
#include "Common/GlobalData.h"
#include "Common/KindOf.h"
#include "Common/MessageStream.h"
#include "Common/Money.h"
#include "Common/NameKeyGenerator.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/Radar.h"
#include "Common/ScoreKeeper.h"
#include "Common/SkirmishBattleHonors.h"
#include "Common/StatsCollector.h"
#include "Common/Team.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/UserPreferences.h"
#include "GameClient/Anim2D.h"
#include "GameClient/ControlBar.h"
#include "GameClient/DisplayStringManager.h"
#include "GameClient/Drawable.h"
#include "GameClient/GameClient.h"
#include "GameClient/Image.h"
#include "GameClient/InGameUI.h"
#include "GameClient/RayEffect.h"
#include "GameLogic/AI.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/RankInfo.h"
#include "GameLogic/ScriptEngine.h"

ORACLE_DEFINE_LINK_GLOBALS("gen_wp3_1b_stats")

class OracleInGameUI : public InGameUI {
public:
    void draw() override {}
    View* createView(bool = false) override { return nullptr; }
    void objectChangedTeam(const Object*, Int, Int) override {}
};

class OracleDisplayStringManager : public DisplayStringManager {
public:
    void init() override {}
    void reset() override {}
    void update() override {}
    DisplayString* newDisplayString() override { return nullptr; }
    void freeDisplayString(DisplayString*) override {}
    DisplayString* getGroupNumeralString(Int) override { return nullptr; }
    DisplayString* getFormationLetterString() override { return nullptr; }
};

class OracleGameClient : public GameClient {
public:
    void init() override {}
    void reset() override {}
    void update() override {}
    void createRayEffectByTemplate(const Coord3D*, const Coord3D*, const ThingTemplate*) override {}
    void addScorch(const Coord3D*, Real, Scorches) override {}
    Drawable* friend_createDrawable(const ThingTemplate* thing, DrawableStatusBits statusBits = DRAWABLE_STATUS_DEFAULT) override {
        return newInstance(Drawable)(thing, statusBits);
    }
    void setTeamColor(Int, Int, Int) override {}
    void setTextureLOD(Int) override {}
    void notifyTerrainObjectMoved(Object*) override {}
    Display* createGameDisplay() override { return nullptr; }
    InGameUI* createInGameUI() override { return nullptr; }
    GameWindowManager* createWindowManager() override { return nullptr; }
    FontLibrary* createFontLibrary() override { return nullptr; }
    DisplayStringManager* createDisplayStringManager() override { return nullptr; }
    VideoPlayerInterface* createVideoPlayer() override { return nullptr; }
    TerrainVisual* createTerrainVisual() override { return nullptr; }
    Keyboard* createKeyboard() override { return nullptr; }
    Mouse* createMouse() override { return nullptr; }
    SnowManager* createSnowManager() override { return nullptr; }
    void setFrameRate(Real) override {}
};

// Private field accessors for deterministic C++ oracle setup (zero vendor edits).
struct GlFrameTag { using type = UnsignedInt GameLogic::*; };
template struct MemberAccess<GlFrameTag, &GameLogic::m_frame>;
UnsignedInt GameLogic::* access_member(GlFrameTag);

struct GlObjListTag { using type = Object* GameLogic::*; };
template struct MemberAccess<GlObjListTag, &GameLogic::m_objList>;
Object* GameLogic::* access_member(GlObjListTag);

struct PlPlayersTag { using type = Player* (PlayerList::*)[MAX_PLAYER_COUNT]; };
template struct MemberAccess<PlPlayersTag, &PlayerList::m_players>;
Player* (PlayerList::* access_member(PlPlayersTag))[MAX_PLAYER_COUNT];

struct GdUserDataTag { using type = AsciiString GlobalData::*; };
template struct MemberAccess<GdUserDataTag, &GlobalData::m_userDataDir>;
AsciiString GlobalData::* access_member(GdUserDataTag);

struct PlIndexTag { using type = PlayerIndex Player::*; };
template struct MemberAccess<PlIndexTag, &Player::m_playerIndex>;
PlayerIndex Player::* access_member(PlIndexTag);

struct PlSideTag { using type = AsciiString Player::*; };
template struct MemberAccess<PlSideTag, &Player::m_side>;
AsciiString Player::* access_member(PlSideTag);

struct PlTypeTag { using type = PlayerType Player::*; };
template struct MemberAccess<PlTypeTag, &Player::m_playerType>;
PlayerType Player::* access_member(PlTypeTag);

struct PlMoneyTag { using type = Money Player::*; };
template struct MemberAccess<PlMoneyTag, &Player::m_money>;
Money Player::* access_member(PlMoneyTag);

struct PlTeamTag { using type = Team* Player::*; };
template struct MemberAccess<PlTeamTag, &Player::m_defaultTeam>;
Team* Player::* access_member(PlTeamTag);

struct TtKindOfTag { using type = KindOfMaskType ThingTemplate::*; };
template struct MemberAccess<TtKindOfTag, &ThingTemplate::m_kindof>;
KindOfMaskType ThingTemplate::* access_member(TtKindOfTag);

struct ObjStatusTag { using type = ObjectStatusMaskType Object::*; };
template struct MemberAccess<ObjStatusTag, &Object::m_status>;
ObjectStatusMaskType Object::* access_member(ObjStatusTag);

struct ObjTeamTag { using type = Team* Object::*; };
template struct MemberAccess<ObjTeamTag, &Object::m_team>;
Team* Object::* access_member(ObjTeamTag);

struct TeamProtoTag { using type = TeamPrototype* Team::*; };
template struct MemberAccess<TeamProtoTag, &Team::m_proto>;
TeamPrototype* Team::* access_member(TeamProtoTag);

struct TeamProtoCtrlTag { using type = Player* TeamPrototype::*; };
template struct MemberAccess<TeamProtoCtrlTag, &TeamPrototype::m_owningPlayer>;
Player* TeamPrototype::* access_member(TeamProtoCtrlTag);

struct ScBuildTag { using type = UnsignedInt StatsCollector::*; };
template struct MemberAccess<ScBuildTag, &StatsCollector::m_buildCommands>;
UnsignedInt StatsCollector::* access_member(ScBuildTag);

struct ScMoveTag { using type = UnsignedInt StatsCollector::*; };
template struct MemberAccess<ScMoveTag, &StatsCollector::m_moveCommands>;
UnsignedInt StatsCollector::* access_member(ScMoveTag);

struct ScAttackTag { using type = UnsignedInt StatsCollector::*; };
template struct MemberAccess<ScAttackTag, &StatsCollector::m_attackCommands>;
UnsignedInt StatsCollector::* access_member(ScAttackTag);

struct ScScrollCmdTag { using type = UnsignedInt StatsCollector::*; };
template struct MemberAccess<ScScrollCmdTag, &StatsCollector::m_scrollMapCommands>;
UnsignedInt StatsCollector::* access_member(ScScrollCmdTag);

struct ScAiUnitsTag { using type = UnsignedInt StatsCollector::*; };
template struct MemberAccess<ScAiUnitsTag, &StatsCollector::m_AIUnits>;
UnsignedInt StatsCollector::* access_member(ScAiUnitsTag);

struct ScPlayerUnitsTag { using type = UnsignedInt StatsCollector::*; };
template struct MemberAccess<ScPlayerUnitsTag, &StatsCollector::m_playerUnits>;
UnsignedInt StatsCollector::* access_member(ScPlayerUnitsTag);

struct ScScrollBeginTag { using type = UnsignedInt StatsCollector::*; };
template struct MemberAccess<ScScrollBeginTag, &StatsCollector::m_scrollBeginTime>;
UnsignedInt StatsCollector::* access_member(ScScrollBeginTag);

struct ScScrollTimeTag { using type = UnsignedInt StatsCollector::*; };
template struct MemberAccess<ScScrollTimeTag, &StatsCollector::m_scrollTime>;
UnsignedInt StatsCollector::* access_member(ScScrollTimeTag);

struct ScIsScrollingTag { using type = Bool StatsCollector::*; };
template struct MemberAccess<ScIsScrollingTag, &StatsCollector::m_isScrolling>;
Bool StatsCollector::* access_member(ScIsScrollingTag);

struct ScTimeCountTag { using type = Int StatsCollector::*; };
template struct MemberAccess<ScTimeCountTag, &StatsCollector::m_timeCount>;
Int StatsCollector::* access_member(ScTimeCountTag);

struct ScLastUpdateTag { using type = Int StatsCollector::*; };
template struct MemberAccess<ScLastUpdateTag, &StatsCollector::m_lastUpdate>;
Int StatsCollector::* access_member(ScLastUpdateTag);

struct ScStartFrameTag { using type = Int StatsCollector::*; };
template struct MemberAccess<ScStartFrameTag, &StatsCollector::m_startFrame>;
Int StatsCollector::* access_member(ScStartFrameTag);

struct ScFileNameTag { using type = AsciiString StatsCollector::*; };
template struct MemberAccess<ScFileNameTag, &StatsCollector::m_statsFileName>;
AsciiString StatsCollector::* access_member(ScFileNameTag);

static inline uint32_t rot_crc(uint32_t crc, uint32_t word) {
    return ((crc << 1) | (crc >> 31)) + word;
}

static std::string read_text_file(const char* path) {
    FILE* f = std::fopen(path, "rb");
    if (!f) return "";
    std::fseek(f, 0, SEEK_END);
    long sz = std::ftell(f);
    std::fseek(f, 0, SEEK_SET);
    std::string out;
    if (sz > 0) {
        out.resize(static_cast<size_t>(sz));
        size_t n = std::fread(&out[0], 1, static_cast<size_t>(sz), f);
        out.resize(n);
    }
    std::fclose(f);
    return out;
}

// Strip wall-clock Date:/End Time: lines so stats output is 100% deterministic.
static std::string normalize_stats_file(const std::string& raw) {
    std::string out;
    size_t start = 0;
    while (start < raw.size()) {
        size_t nl = raw.find('\n', start);
        size_t end = (nl == std::string::npos) ? raw.size() : nl + 1;
        std::string line = raw.substr(start, end - start);
        if (line.rfind("Date:\t", 0) != 0 && line.rfind("End Time:\t", 0) != 0) {
            out += line;
        }
        start = end;
    }
    return out;
}

// Lightweight fake Object layout? No — let's use real ThingTemplate + Object via ThingFactory!
// Wait: Object constructor touches Thing::m_template, Object::m_status, Object::m_team, etc.
// We test whether real Object construction or lightweight Object works; let's inspect via real ThingFactory + Player + Team.
struct FakeObjectHolder {
    // Only used if real Object::Object needs too many singletons; let's try real Object first!
};

int main(int argc, char** argv) {
    const char* out_arg = (argc > 1) ? argv[1] : "docs/golden_vectors/WP3_1b_Stats.json";
    char cwd_buf[4096];
    if (!getcwd(cwd_buf, sizeof(cwd_buf))) return 1;
    std::string orig_cwd(cwd_buf);
    std::string out_path = (out_arg[0] == '/') ? std::string(out_arg) : (orig_cwd + "/" + out_arg);

    oracle::init_memory();

    TheWritableGlobalData = NEW GlobalData;
    TheNameKeyGenerator = NEW NameKeyGenerator;
    TheNameKeyGenerator->init();
    TheFileSystem = NEW FileSystem;
    TheRankInfoStore = NEW RankInfoStore;
    TheRankInfoStore->init();
    TheThingFactory = NEW ThingFactory;
    TheTeamFactory = NEW TeamFactory;
    TheTeamFactory->init();
    TheDisplayStringManager = NEW OracleDisplayStringManager;
    TheMappedImageCollection = NEW ImageCollection;
    TheAnim2DCollection = NEW Anim2DCollection;
    TheRayEffects = NEW RayEffectSystem;
    TheRayEffects->init();
    TheControlBar = NEW ControlBar;
    TheInGameUI = NEW OracleInGameUI;
    TheGameClient = NEW OracleGameClient;
    TheRadar = NEW RadarDummy;
    TheRadar->init();
    TheScriptEngine = NEW ScriptEngine;
    TheAI = NEW AI;
    TheGameLogic = NEW GameLogic;
    TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
    ThePlayerList = NEW PlayerList;

    // Setup temp workspace in /tmp for SkirmishBattleHonors & StatsCollector I/O
    std::string tmp_root = "/tmp/wp3_1b_oracle_run";
    mkdir(tmp_root.c_str(), 0755);
    std::string userdata_dir = tmp_root + "/userdata";
    mkdir(userdata_dir.c_str(), 0755);
    TheWritableGlobalData->*access_member(GdUserDataTag()) = userdata_dir.c_str();
    if (chdir(tmp_root.c_str()) != 0) {
        std::perror("chdir");
        return 1;
    }

    // Create 8 deterministic ThingTemplates with strictly ascending pointer addresses?
    // Verify pointer ordering explicitly; sort into template catalog by pointer order if needed
    // so Rust BTreeMap<TemplateId, ...> matches C++ std::map<const ThingTemplate*, Int> xfer order!
    struct TmplSpec {
        const char* name;
        KindOfType k1;
        KindOfType k2;
        KindOfType k3;
    };
    static const TmplSpec kSpecs[8] = {
        {"Tmpl_Bld_Score",       KINDOF_STRUCTURE, KINDOF_SCORE,         KINDOF_INVALID},
        {"Tmpl_Bld_Create",      KINDOF_STRUCTURE, KINDOF_SCORE_CREATE,  KINDOF_INVALID},
        {"Tmpl_Bld_Destroy",     KINDOF_STRUCTURE, KINDOF_SCORE_DESTROY, KINDOF_INVALID},
        {"Tmpl_Bld_Tech",        KINDOF_STRUCTURE, KINDOF_TECH_BUILDING, KINDOF_INVALID},
        {"Tmpl_Inf_Score",       KINDOF_INFANTRY,  KINDOF_SCORE,         KINDOF_INVALID},
        {"Tmpl_Veh_ScoreCreate", KINDOF_VEHICLE,   KINDOF_SCORE_CREATE,  KINDOF_INVALID},
        {"Tmpl_Veh_ScoreDest",   KINDOF_VEHICLE,   KINDOF_SCORE_DESTROY, KINDOF_INVALID},
        {"Tmpl_Air_Score",       KINDOF_AIRCRAFT,  KINDOF_SCORE,         KINDOF_INVALID},
    };
    ThingTemplate* raw_tmpls[8]{};
    for (int i = 0; i < 8; ++i) {
        raw_tmpls[i] = TheThingFactory->newTemplate(AsciiString(kSpecs[i].name));
        KindOfMaskType mask;
        if (kSpecs[i].k1 != KINDOF_INVALID) mask.set(kSpecs[i].k1);
        if (kSpecs[i].k2 != KINDOF_INVALID) mask.set(kSpecs[i].k2);
        if (kSpecs[i].k3 != KINDOF_INVALID) mask.set(kSpecs[i].k3);
        raw_tmpls[i]->*access_member(TtKindOfTag()) = mask;
    }
    // Check whether pointer order equals 0..7; if not, report in seed_0_diagnostics!
    std::vector<int> ptr_order(8);
    for (int i = 0; i < 8; ++i) ptr_order[i] = i;
    std::sort(ptr_order.begin(), ptr_order.end(), [&](int a, int b) {
        return raw_tmpls[a] < raw_tmpls[b];
    });
    // Re-order mapping so index `id` in Rust matches pointer rank in C++:
    int ptr_rank[8]{};
    for (int r = 0; r < 8; ++r) ptr_rank[ptr_order[r]] = r;

    // Setup 16 Players + Default Teams
    // Setup 16 Players + Default Teams
    Player* players[MAX_PLAYER_COUNT]{};
    Team* teams[MAX_PLAYER_COUNT]{};
    Dict empty_team_dict;
    for (Int p = 0; p < MAX_PLAYER_COUNT; ++p) {
        players[p] = ThePlayerList->getNthPlayer(p);
        players[p]->*access_member(PlIndexTag()) = p;
        players[p]->*access_member(PlTypeTag()) = (p == 1) ? PLAYER_HUMAN : PLAYER_COMPUTER;
        if (p == 0) {
            players[p]->*access_member(PlSideTag()) = AsciiString("");
        } else if (p == MAX_PLAYER_COUNT - 1) {
            players[p]->*access_member(PlSideTag()) = AsciiString("Civilian");
        } else {
            players[p]->*access_member(PlSideTag()) = AsciiString((p % 2 == 0) ? "China" : "America");
        }
        AsciiString tname;
        tname.format("teamPlayer%d", p);
        TheTeamFactory->initTeam(tname, AsciiString::TheEmptyString, FALSE, &empty_team_dict);
        teams[p] = TheTeamFactory->createTeam(tname);
        teams[p]->setControllingPlayer(players[p]);
        players[p]->*access_member(PlTeamTag()) = teams[p];
    }
    ThePlayerList->setLocalPlayer(players[1]);

    // Compute source sha256 from repo root (supporting worktree runs where vendor/ lives in main repo)
    std::string vendor_root = orig_cwd;
    {
        struct stat st{};
        std::string probe = vendor_root + "/vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/BattleHonors.h";
        if (stat(probe.c_str(), &st) != 0) {
            vendor_root = "/Users/chayut/repos/generals-rust";
        }
    }
    auto repo_sha = [&](const char* rel) {
        std::string full = vendor_root + "/" + rel;
        return oracle::sha256_file(full.c_str());
    };
    const char* s_bh_h = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/BattleHonors.h";
    const char* s_sk_h = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ScoreKeeper.h";
    const char* s_sk_c = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/ScoreKeeper.cpp";
    const char* s_sbh_h = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/SkirmishBattleHonors.h";
    const char* s_sbh_c = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/SkirmishBattleHonors.cpp";
    const char* s_sc_h = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/StatsCollector.h";
    const char* s_sc_c = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/StatsCollector.cpp";

    std::string sha_bh_h = repo_sha(s_bh_h);
    std::string sha_sk_h = repo_sha(s_sk_h);
    std::string sha_sk_c = repo_sha(s_sk_c);
    std::string sha_sbh_h = repo_sha(s_sbh_h);
    std::string sha_sbh_c = repo_sha(s_sbh_c);
    std::string sha_sc_h = repo_sha(s_sc_h);
    std::string sha_sc_c = repo_sha(s_sc_c);

    std::string json;
    json.reserve(4 * 1024 * 1024);
    json += "{\n";
    json += "  \"work_package\": \"WP-3.1b\",\n";
    json += "  \"modules\": [\"BattleHonors\", \"ScoreKeeper\", \"SkirmishBattleHonors\", \"StatsCollector\"],\n";
    json += "  \"vector_count\": 1000,\n";
    json += "  \"source_sha256\": {\n";
    json += std::string("    \"") + s_bh_h + "\": \"" + sha_bh_h + "\",\n";
    json += std::string("    \"") + s_sk_h + "\": \"" + sha_sk_h + "\",\n";
    json += std::string("    \"") + s_sk_c + "\": \"" + sha_sk_c + "\",\n";
    json += std::string("    \"") + s_sbh_h + "\": \"" + sha_sbh_h + "\",\n";
    json += std::string("    \"") + s_sbh_c + "\": \"" + sha_sbh_c + "\",\n";
    json += std::string("    \"") + s_sc_h + "\": \"" + sha_sc_h + "\",\n";
    json += std::string("    \"") + s_sc_c + "\": \"" + sha_sc_c + "\"\n";
    json += "  },\n";

    // Build reusable Object instances? Let's test if `NEW Object(...)` works cleanly here;
    // Note: if Object::Object requires ThePartitionManager/TheModuleFactory, let's check at link/run time!
    // Wait: let's also write seed_0_diagnostics after seed 0 runs.
    // Pre-create reusable ScoreKeeper objects (8 templates) and StatsCollector world objects (6)
    // so we never churn MemoryPool/GameLogic object lists across 1,000 seeds.
    Object* sk_objs[8]{};
    for (int i = 0; i < 8; ++i) {
        sk_objs[i] = newInstance(Object)(raw_tmpls[i], ObjectStatusMaskType(), teams[0]);
    }
    int sc_obj_tmpls[6] = {4, 5, 6, 4, 4, 0}; // inf, veh, veh, inf(neutral), inf(civilian), bld(AI)
    Object* sc_world_objs[6]{};
    for (int oi = 0; oi < 6; ++oi) {
        sc_world_objs[oi] = newInstance(Object)(raw_tmpls[sc_obj_tmpls[oi]], ObjectStatusMaskType(), teams[0]);
    }
    for (int oi = 0; oi < 6; ++oi) {
        sc_world_objs[oi]->friend_setPrevObject(oi > 0 ? sc_world_objs[oi - 1] : nullptr);
        sc_world_objs[oi]->friend_setNextObject(oi + 1 < 6 ? sc_world_objs[oi + 1] : nullptr);
    }
    TheGameLogic->*access_member(GlObjListTag()) = sc_world_objs[0];

    std::string vectors_json;
    vectors_json.reserve(3 * 1024 * 1024);
    std::string seed0_diag;

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed * 0x9e3779b9u ^ 0x31b0cafeu;
        if (rng == 0) rng = 0x13572468u;

        // ---------------- 1. BattleHonors (PKT-117) ----------------
        uint32_t bh_raw = oracle::splitmix32(rng);
        uint32_t bh_all =
            BATTLE_HONOR_LADDER_CHAMP | BATTLE_HONOR_STREAK |
            BATTLE_HONOR_LOYALTY_USA | BATTLE_HONOR_LOYALTY_CHINA |
            BATTLE_HONOR_BATTLE_TANK | BATTLE_HONOR_AIR_WING |
            BATTLE_HONOR_LOYALTY_GLA | BATTLE_HONOR_ENDURANCE |
            BATTLE_HONOR_CAMPAIGN_USA | BATTLE_HONOR_CAMPAIGN_CHINA |
            BATTLE_HONOR_CAMPAIGN_GLA | BATTLE_HONOR_BLITZ5 |
            BATTLE_HONOR_BLITZ10 | BATTLE_HONOR_FAIR_PLAY |
            BATTLE_HONOR_APOCALYPSE | BATTLE_HONOR_OFFICERSCLUB |
            BATTLE_HONOR_DOMINATION | BATTLE_HONOR_CHALLENGE_MODE |
            BATTLE_HONOR_ULTIMATE | BATTLE_HONOR_GLOBAL_GENERAL |
            BATTLE_HONOR_DOMINATION_ONLINE | BATTLE_HONOR_STREAK_ONLINE |
            BATTLE_HONOR_CHALLENGE | BATTLE_HONOR_NOT_GAINED;
        uint32_t bh_masked = bh_raw & bh_all;
        uint32_t bh_chal_all =
            BH_CHALLENGE_MASK_1 | BH_CHALLENGE_MASK_2 | BH_CHALLENGE_MASK_3 |
            BH_CHALLENGE_MASK_4 | BH_CHALLENGE_MASK_5 | BH_CHALLENGE_MASK_6 |
            BH_CHALLENGE_MASK_7;
        uint32_t bh_chal = (oracle::splitmix32(rng) & 0xffu) & bh_chal_all;

        // ---------------- 2. SkirmishBattleHonors (PKT-123) ----------------
        std::string ini_path = userdata_dir + "/SkirmishStats.ini";
        std::remove(ini_path.c_str());
        if (seed == 0) {
            // Seed pre-existing INI lines on seed 0 to exercise UserPreferences::load() parsing
            std::string pre =
                "  Wins = 7 \n"
                "SCUD = YES\n"
                "PPC = TrUe\n"
                "Nuke = 0\n"
                "LastHouse = AmericaAirForceGeneral\n"
                "EmptySkip = \n";
            oracle::write_file_atomic(ini_path.c_str(), pre);
        }

        SkirmishBattleHonors sbh;
        Int sbh_pre_wins = sbh.getWins();
        Bool sbh_pre_scud = sbh.builtSCUD();
        Bool sbh_pre_ppc = sbh.builtParticleCannon();
        Bool sbh_pre_nuke = sbh.builtNuke();

        Int s_wins = static_cast<Int>(oracle::splitmix32(rng) % 500u);
        Int s_losses = static_cast<Int>(oracle::splitmix32(rng) % 500u);
        Int s_streak = static_cast<Int>(oracle::splitmix32(rng) % 100u);
        Int s_best = static_cast<Int>(oracle::splitmix32(rng) % 200u);
        Int s_chal_medals = static_cast<Int>(oracle::splitmix32(rng) % 128u);
        Int s_loyal = static_cast<Int>(oracle::splitmix32(rng) % 60u);
        uint32_t s_super = oracle::splitmix32(rng) & 7u;
        Int s_honors1 = static_cast<Int>(oracle::splitmix32(rng) & 0x0fffffffu);
        Int s_honors2 = static_cast<Int>(oracle::splitmix32(rng) & 0x0fffffffu);
        Int s_end_diff = static_cast<Int>(oracle::splitmix32(rng) % 3u);
        Int s_end_ais = static_cast<Int>(1u + (oracle::splitmix32(rng) % 7u));
        static const char* kMaps[4] = {
            "Tournament Desert", "Twilight Flame", "Fallen Empire", "Lone Eagle"
        };
        static const char* kGens[5] = {
            "AmericaAirForceGeneral", "ChinaTankGeneral", "GLAToxinGeneral", "America", ""
        };
        int s_map_idx = static_cast<int>(oracle::splitmix32(rng) % 4u);
        int s_gen_idx = static_cast<int>(oracle::splitmix32(rng) % 5u);
        uint32_t s_usa_m = oracle::splitmix32(rng) & 7u;
        uint32_t s_chi_m = oracle::splitmix32(rng) & 7u;
        uint32_t s_gla_m = oracle::splitmix32(rng) & 7u;
        Int s_chal_idx = static_cast<Int>(oracle::splitmix32(rng) % 9u);
        Int s_chal_diff = static_cast<Int>(oracle::splitmix32(rng) % 3u);

        sbh.setWins(s_wins);
        sbh.setLosses(s_losses);
        sbh.setWinStreak(s_streak);
        sbh.setBestWinStreak(s_best);
        sbh.setChallengeMedals(s_chal_medals);
        sbh.setNumGamesLoyal(s_loyal);
        if (s_super & 1u) sbh.setBuiltSCUD();
        if (s_super & 2u) sbh.setBuiltParticleCannon();
        if (s_super & 4u) sbh.setBuiltNuke();
        sbh.setHonors(s_honors1);
        sbh.setHonors(s_honors2);
        sbh.setEnduranceMedal(AsciiString(kMaps[s_map_idx]), s_end_diff, s_end_ais);
        sbh.setLastGeneral(AsciiString(kGens[s_gen_idx]));
        for (int d = 0; d < 3; ++d) {
            if (s_usa_m & (1u << d)) sbh.setUSACampaignComplete(static_cast<GameDifficulty>(d));
            if (s_chi_m & (1u << d)) sbh.setCHINACampaignComplete(static_cast<GameDifficulty>(d));
            if (s_gla_m & (1u << d)) sbh.setGLACampaignComplete(static_cast<GameDifficulty>(d));
        }
        sbh.setChallengeCampaignComplete(s_chal_idx, static_cast<GameDifficulty>(s_chal_diff));

        // Write and reload via UserPreferences::write()/loadFromIniFile() to verify INI roundtrip!
        sbh.write();
        std::string sbh_written = read_text_file(ini_path.c_str());
        SkirmishBattleHonors sbh2;

        Int sbh_getters[11] = {
            sbh2.getWins(), sbh2.getLosses(), sbh2.getWinStreak(), sbh2.getBestWinStreak(),
            sbh2.getChallengeMedals(), sbh2.builtSCUD(), sbh2.builtParticleCannon(),
            sbh2.builtNuke(), sbh2.getHonors(),
            sbh2.getEnduranceMedal(AsciiString(kMaps[s_map_idx]), s_end_diff),
            sbh2.getNumGamesLoyal()
        };
        uint32_t sbh_camp_bits = 0;
        for (int d = 0; d < 3; ++d) {
            if (sbh2.getUSACampaignComplete(static_cast<GameDifficulty>(d))) sbh_camp_bits |= (1u << d);
            if (sbh2.getCHINACampaignComplete(static_cast<GameDifficulty>(d))) sbh_camp_bits |= (1u << (d + 3));
            if (sbh2.getGLACampaignComplete(static_cast<GameDifficulty>(d))) sbh_camp_bits |= (1u << (d + 6));
        }
        if (sbh2.getChallengeCampaignComplete(s_chal_idx, static_cast<GameDifficulty>(s_chal_diff))) {
            sbh_camp_bits |= (1u << 9);
        }
        std::string sbh_last_gen = sbh2.getLastGeneral().str();

        // ---------------- 3. ScoreKeeper (PKT-122) ----------------
        Int sk_player = static_cast<Int>(oracle::splitmix32(rng) % MAX_PLAYER_COUNT);
        ScoreKeeper sk;
        sk.reset(sk_player);

        // Execute 16 ops: [kind, arg0, arg1, arg2]
        // kind 0: enableScoring(arg0)
        // kind 1: addMoneyEarned(arg0)
        // kind 2: addMoneySpent(arg0)
        // kind 3: addObjectBuilt(tmpl=arg0)
        // kind 4: removeObjectBuilt(tmpl=arg0)
        // kind 5: addObjectCaptured(tmpl=arg0)
        // kind 6: addObjectDestroyed(tmpl=arg0, owner=arg1, under_constr=arg2)
        // kind 7: addObjectLost(tmpl=arg0, under_constr=arg1)
        std::vector<int32_t> sk_ops;
        uint32_t sk_tcrc = 0;
        for (int step = 0; step < 16; ++step) {
            int32_t kind = static_cast<int32_t>(oracle::splitmix32(rng) % 8u);
            if (seed == 0 && step == 0) kind = 0;
            int32_t a0 = 0, a1 = 0, a2 = 0;
            if (kind == 0) {
                a0 = (seed == 0 && step == 0) ? 1 : static_cast<int32_t>((oracle::splitmix32(rng) % 5u) != 0);
                TheGameLogic->enableScoring(a0 ? TRUE : FALSE);
            } else if (kind == 1 || kind == 2) {
                a0 = static_cast<int32_t>(oracle::splitmix32(rng) % 5000u) - 200;
                if (kind == 1) sk.addMoneyEarned(a0);
                else sk.addMoneySpent(a0);
            } else {
                int raw_idx = static_cast<int>(oracle::splitmix32(rng) % 8u);
                a0 = ptr_rank[raw_idx]; // canonical rank ID matching C++ pointer order
                ThingTemplate* tmpl = raw_tmpls[raw_idx];
                ObjectStatusMaskType st;
                if (kind == 6) {
                    a1 = static_cast<int32_t>(oracle::splitmix32(rng) % MAX_PLAYER_COUNT);
                    a2 = static_cast<int32_t>(oracle::splitmix32(rng) & 1u);
                    if (a2) st.set(OBJECT_STATUS_UNDER_CONSTRUCTION);
                } else if (kind == 7) {
                    a1 = static_cast<int32_t>(oracle::splitmix32(rng) & 1u);
                    if (a1) st.set(OBJECT_STATUS_UNDER_CONSTRUCTION);
                } else {
                    a1 = sk_player;
                }
                Int owner = (kind == 6) ? a1 : sk_player;
                Object* obj = sk_objs[raw_idx];
                obj->*access_member(ObjTeamTag()) = teams[owner];
                obj->*access_member(ObjStatusTag()) = st;
                if (kind == 3) sk.addObjectBuilt(obj);
                else if (kind == 4) sk.removeObjectBuilt(obj);
                else if (kind == 5) sk.addObjectCaptured(obj);
                else if (kind == 6) sk.addObjectDestroyed(obj);
                else if (kind == 7) sk.addObjectLost(obj);
            }
            sk_ops.push_back(kind);
            sk_ops.push_back(a0);
            sk_ops.push_back(a1);
            sk_ops.push_back(a2);
            sk_tcrc = rot_crc(sk_tcrc, static_cast<uint32_t>(sk.calculateScore()));
            sk_tcrc = rot_crc(sk_tcrc, static_cast<uint32_t>(sk.getTotalUnitsBuilt()));
            sk_tcrc = rot_crc(sk_tcrc, static_cast<uint32_t>(sk.getTotalBuildingsBuilt()));
            sk_tcrc = rot_crc(sk_tcrc, static_cast<uint32_t>(sk.getTotalUnitsDestroyed()));
            sk_tcrc = rot_crc(sk_tcrc, static_cast<uint32_t>(sk.getTotalBuildingsDestroyed()));
            sk_tcrc = rot_crc(sk_tcrc, static_cast<uint32_t>(sk.getTotalUnitsLost()));
            sk_tcrc = rot_crc(sk_tcrc, static_cast<uint32_t>(sk.getTotalBuildingsLost()));
        }
        TheGameLogic->enableScoring(TRUE);

        KindOfMaskType mask_veh_valid, mask_veh_invalid;
        mask_veh_valid.set(KINDOF_VEHICLE);
        mask_veh_invalid.set(KINDOF_SCORE_DESTROY);
        Int sk_units_veh = sk.getTotalUnitsBuilt(mask_veh_valid, mask_veh_invalid);
        Int sk_obj_bld = sk.getTotalObjectsBuilt(raw_tmpls[0]);
        Int sk_obj_inf = sk.getTotalObjectsBuilt(raw_tmpls[4]);

        oracle::MemoryXfer sk_save(XFER_SAVE);
        sk_save.open("sk_save");
        sk_save.xferSnapshot(&sk);
        sk_save.close();

        ScoreKeeper sk_loaded;
        oracle::MemoryXfer sk_load(XFER_LOAD);
        sk_load.bytes = sk_save.bytes;
        sk_load.open("sk_load");
        sk_load.xferSnapshot(&sk_loaded);
        sk_load.close();

        oracle::MemoryXfer sk_resave(XFER_SAVE);
        sk_resave.open("sk_resave");
        sk_resave.xferSnapshot(&sk_loaded);
        sk_resave.close();

        // ---------------- 4. StatsCollector (PKT-124) ----------------
        Int sc_play_stats = static_cast<Int>(1u + (oracle::splitmix32(rng) % 5u));
        Int sc_local_p = 1 + static_cast<Int>(oracle::splitmix32(rng) % 4u);
        ThePlayerList->setLocalPlayer(players[sc_local_p]);
        players[sc_local_p]->*access_member(PlMoneyTag()) = Money();
        players[sc_local_p]->getMoney()->deposit(
            static_cast<UnsignedInt>(oracle::splitmix32(rng) % 25000u), FALSE, FALSE);
        TheWritableGlobalData->m_playStats = sc_play_stats;
        TheWritableGlobalData->m_mapName = AsciiString("Maps/Tournament Desert/Tournament Desert.map");
        UnsignedInt sc_start_f = 10u + (oracle::splitmix32(rng) % 100u);
        TheGameLogic->*access_member(GlFrameTag()) = sc_start_f;

        // Configure deterministic object list owners in TheGameLogic->m_objList for collectUnitCountStats
        // 6 objects across local, AI, neutral, Civilian, structure, infantry, vehicle
        Int obj_owners[6] = {sc_local_p, sc_local_p, 2, 0, MAX_PLAYER_COUNT - 1, 3};
        for (int oi = 0; oi < 6; ++oi) {
            if (obj_owners[oi] == sc_local_p && oi == 2) obj_owners[oi] = (sc_local_p == 2) ? 3 : 2;
            sc_world_objs[oi]->*access_member(ObjTeamTag()) = teams[obj_owners[oi]];
        }
        TheGameLogic->*access_member(GlObjListTag()) = sc_world_objs[0];

        StatsCollector sc;
        sc.reset();

        std::vector<int32_t> sc_ops;
        uint32_t sc_tcrc = 0;
        for (int step = 0; step < 12; ++step) {
            int32_t kind = static_cast<int32_t>(oracle::splitmix32(rng) % 8u);
            TheGameLogic->*access_member(GlFrameTag()) += (oracle::splitmix32(rng) % 45u);
            int32_t a0 = 0, a1 = 0;
            if (kind == 0) {
                static const GameMessage::Type kMsgTypes[5] = {
                    GameMessage::MSG_QUEUE_UNIT_CREATE,
                    GameMessage::MSG_DOZER_CONSTRUCT,
                    GameMessage::MSG_DOZER_CONSTRUCT_LINE,
                    GameMessage::MSG_DO_MOVETO,
                    GameMessage::MSG_DO_ATTACK_OBJECT,
                };
                int mt_idx = static_cast<int>(oracle::splitmix32(rng) % 5u);
                a0 = static_cast<int32_t>(kMsgTypes[mt_idx]);
                a1 = ((oracle::splitmix32(rng) % 3u) == 0) ? ((sc_local_p + 1) % MAX_PLAYER_COUNT) : sc_local_p;
                GameMessage* msg = newInstance(GameMessage)(kMsgTypes[mt_idx]);
                msg->friend_setPlayerIndex(a1);
                sc.collectMsgStats(msg);
                deleteInstance(msg);
            } else if (kind == 1) {
                sc.incrementScrollMoveCount();
            } else if (kind == 2) {
                sc.incrementAttackCount();
            } else if (kind == 3) {
                sc.incrementBuildCount();
            } else if (kind == 4) {
                sc.incrementMoveCount();
            } else if (kind == 5) {
                sc.startScrollTime();
            } else if (kind == 6) {
                sc.endScrollTime();
            } else {
                sc.update();
            }
            sc_ops.push_back(kind);
            sc_ops.push_back(a0);
            sc_ops.push_back(a1);
            sc_ops.push_back(static_cast<int32_t>(TheGameLogic->getFrame()));

            sc_tcrc = rot_crc(sc_tcrc, sc.*access_member(ScBuildTag()));
            sc_tcrc = rot_crc(sc_tcrc, sc.*access_member(ScMoveTag()));
            sc_tcrc = rot_crc(sc_tcrc, sc.*access_member(ScAttackTag()));
            sc_tcrc = rot_crc(sc_tcrc, sc.*access_member(ScScrollCmdTag()));
            sc_tcrc = rot_crc(sc_tcrc, sc.*access_member(ScAiUnitsTag()));
            sc_tcrc = rot_crc(sc_tcrc, sc.*access_member(ScPlayerUnitsTag()));
            sc_tcrc = rot_crc(sc_tcrc, sc.*access_member(ScScrollTimeTag()));
            sc_tcrc = rot_crc(sc_tcrc, static_cast<uint32_t>(sc.*access_member(ScTimeCountTag())));
            sc_tcrc = rot_crc(sc_tcrc, static_cast<uint32_t>(sc.*access_member(ScLastUpdateTag())));
        }
        TheGameLogic->*access_member(GlFrameTag()) += 37u;
        UnsignedInt sc_end_frame = TheGameLogic->getFrame();
        sc.writeFileEnd();

        std::string sc_fname = (sc.*access_member(ScFileNameTag())).str();
        std::string sc_norm = normalize_stats_file(read_text_file(sc_fname.c_str()));
        std::remove(sc_fname.c_str());

        if (seed == 0) {
            char diag[4096];
            std::snprintf(diag, sizeof(diag),
                "  \"seed_0_diagnostics\": {\n"
                "    \"bh_constants\": {\"ladder_champ\":%u,\"streak\":%u,\"loyalty_usa\":%u,\"loyalty_china\":%u,"
                "\"battle_tank\":%u,\"air_wing\":%u,\"loyalty_gla\":%u,\"endurance\":%u,\"campaign_usa\":%u,"
                "\"campaign_china\":%u,\"campaign_gla\":%u,\"blitz5\":%u,\"blitz10\":%u,\"fair_play\":%u,"
                "\"apocalypse\":%u,\"officersclub\":%u,\"domination\":%u,\"challenge_mode\":%u,\"ultimate\":%u,"
                "\"global_general\":%u,\"domination_online\":%u,\"streak_online\":%u,\"challenge\":%u,\"not_gained\":%u,"
                "\"chal_mask_1\":%u,\"chal_mask_7\":%u,\"max_cols\":%u,\"img_w\":%u,\"img_h\":%u},\n"
                "    \"sbh_seed0_pre\": {\"wins\":%d,\"scud\":%u,\"ppc\":%u,\"nuke\":%u},\n"
                "    \"sk_seed0\": {\"player\":%d,\"score\":%d,\"save_len\":%zu,\"resave_match\":%u,\"ptr_order\":[%d,%d,%d,%d,%d,%d,%d,%d]},\n"
                "    \"sc_seed0\": {\"start_frame\":%u,\"end_frame\":%u,\"time_count\":%d,\"last_update\":%d,\"norm_len\":%zu}\n"
                "  },\n",
                BATTLE_HONOR_LADDER_CHAMP, BATTLE_HONOR_STREAK, BATTLE_HONOR_LOYALTY_USA, BATTLE_HONOR_LOYALTY_CHINA,
                BATTLE_HONOR_BATTLE_TANK, BATTLE_HONOR_AIR_WING, BATTLE_HONOR_LOYALTY_GLA, BATTLE_HONOR_ENDURANCE,
                BATTLE_HONOR_CAMPAIGN_USA, BATTLE_HONOR_CAMPAIGN_CHINA, BATTLE_HONOR_CAMPAIGN_GLA, BATTLE_HONOR_BLITZ5,
                BATTLE_HONOR_BLITZ10, BATTLE_HONOR_FAIR_PLAY, BATTLE_HONOR_APOCALYPSE, BATTLE_HONOR_OFFICERSCLUB,
                BATTLE_HONOR_DOMINATION, BATTLE_HONOR_CHALLENGE_MODE, BATTLE_HONOR_ULTIMATE, BATTLE_HONOR_GLOBAL_GENERAL,
                BATTLE_HONOR_DOMINATION_ONLINE, BATTLE_HONOR_STREAK_ONLINE, BATTLE_HONOR_CHALLENGE, BATTLE_HONOR_NOT_GAINED,
                BH_CHALLENGE_MASK_1, BH_CHALLENGE_MASK_7,
                MAX_BATTLE_HONOR_COLUMNS, MAX_BATTLE_HONOR_IMAGE_WIDTH, MAX_BATTLE_HONOR_IMAGE_HEIGHT,
                sbh_pre_wins, sbh_pre_scud, sbh_pre_ppc, sbh_pre_nuke,
                sk_player, sk.calculateScore(), sk_save.bytes.size(),
                static_cast<unsigned>(sk_save.bytes == sk_resave.bytes),
                ptr_order[0], ptr_order[1], ptr_order[2], ptr_order[3],
                ptr_order[4], ptr_order[5], ptr_order[6], ptr_order[7],
                sc.*access_member(ScStartFrameTag()), sc_end_frame,
                sc.*access_member(ScTimeCountTag()), sc.*access_member(ScLastUpdateTag()),
                sc_norm.size());
            seed0_diag = diag;
        }

        // Emit vector JSON line (use large buffer / dynamic append to avoid truncation)
        char buf[8192];
        int n1 = std::snprintf(buf, sizeof(buf),
            "    {\"seed\":%u,\"bh_raw\":%u,\"bh_masked\":%u,\"bh_chal\":%u,"
            "\"s_in\":[%d,%d,%d,%d,%d,%d,%u,%d,%d,%d,%d,%d,%d,%u,%u,%u,%d,%d],"
            "\"s_out\":[%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d],\"s_camp\":%u,\"s_last\":\"%s\",\"s_ini\":\"%s\","
            "\"sk_player\":%d,\"sk_ops\":[",
            seed, bh_raw, bh_masked, bh_chal,
            s_wins, s_losses, s_streak, s_best, s_chal_medals, s_loyal, s_super,
            s_honors1, s_honors2, s_end_diff, s_end_ais, s_map_idx, s_gen_idx,
            s_usa_m, s_chi_m, s_gla_m, s_chal_idx, s_chal_diff,
            sbh_getters[0], sbh_getters[1], sbh_getters[2], sbh_getters[3],
            sbh_getters[4], sbh_getters[5], sbh_getters[6], sbh_getters[7],
            sbh_getters[8], sbh_getters[9], sbh_getters[10],
            sbh_camp_bits, oracle::json_escape(sbh_last_gen).c_str(),
            oracle::json_escape(sbh_written).c_str(),
            sk_player);
        if (n1 < 0 || static_cast<size_t>(n1) >= sizeof(buf)) std::exit(2);
        vectors_json += buf;

        for (size_t i = 0; i < sk_ops.size(); ++i) {
            std::snprintf(buf, sizeof(buf), "%d%s", sk_ops[i], (i + 1 < sk_ops.size()) ? "," : "");
            vectors_json += buf;
        }
        int n2 = std::snprintf(buf, sizeof(buf),
            "],\"sk_tcrc\":%u,\"sk_score\":%d,\"sk_stats\":[%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d],"
            "\"sk_save_hex\":\"%s\",\"sc_play_stats\":%d,\"sc_local_p\":%d,\"sc_money\":%u,\"sc_start_f\":%u,\"sc_end_f\":%u,\"sc_ops\":[",
            sk_tcrc, sk.calculateScore(),
            sk.getTotalMoneyEarned(), sk.getTotalMoneySpent(),
            sk.getTotalUnitsBuilt(), sk.getTotalUnitsLost(),
            sk.getTotalBuildingsBuilt(), sk.getTotalBuildingsLost(),
            sk.getTotalUnitsDestroyed(), sk.getTotalBuildingsDestroyed(),
            sk.getTotalTechBuildingsCaptured(), sk.getTotalFactionBuildingsCaptured(),
            sk_units_veh, sk_obj_bld, sk_obj_inf,
            sk_save.hex().c_str(),
            sc_play_stats, sc_local_p, players[sc_local_p]->getMoney()->countMoney(),
            sc_start_f, sc_end_frame);
        if (n2 < 0 || static_cast<size_t>(n2) >= sizeof(buf)) std::exit(2);
        vectors_json += buf;

        for (size_t i = 0; i < sc_ops.size(); ++i) {
            std::snprintf(buf, sizeof(buf), "%d%s", sc_ops[i], (i + 1 < sc_ops.size()) ? "," : "");
            vectors_json += buf;
        }
        int n3 = std::snprintf(buf, sizeof(buf),
            "],\"sc_tcrc\":%u,\"sc_out\":[%u,%u,%u,%u,%u,%u,%u,%u,%u,%d,%d,%d],\"sc_norm\":\"%s\"}%s\n",
            sc_tcrc,
            sc.*access_member(ScBuildTag()), sc.*access_member(ScMoveTag()),
            sc.*access_member(ScAttackTag()), sc.*access_member(ScScrollCmdTag()),
            sc.*access_member(ScAiUnitsTag()), sc.*access_member(ScPlayerUnitsTag()),
            sc.*access_member(ScScrollBeginTag()), sc.*access_member(ScScrollTimeTag()),
            static_cast<unsigned>(sc.*access_member(ScIsScrollingTag())),
            sc.*access_member(ScTimeCountTag()), sc.*access_member(ScLastUpdateTag()),
            sc.*access_member(ScStartFrameTag()),
            oracle::json_escape(sc_norm).c_str(),
            (seed + 1u < 1000u) ? "," : "");
        if (n3 < 0 || static_cast<size_t>(n3) >= sizeof(buf)) std::exit(2);
        vectors_json += buf;
    }

    json += seed0_diag;
    json += "  \"vectors\": [\n";
    json += vectors_json;
    json += "  ]\n}\n";

    oracle::write_file_atomic(out_path.c_str(), json);
    std::fprintf(stderr, "Generated 1000 WP-3.1b vectors to %s\n", out_path.c_str());
    return 0;
}
