// WP-3.2c (PKT-128 MultiplayerSettings + PKT-104 INIMultiplayer + PKT-130 PlayerTemplate +
// PKT-131 ProductionPrerequisite) C++ Golden Vector Generator.
// Compiles unmodified vendor/GeneralsX sources against minimal harness stubs.
//
// Canonical build & run command (macOS arm64, Release-aligned macros):
//   clang++ -std=c++20 -O1 -ffp-contract=off -URTS_DEBUG -U_ALLOW_DEBUG_CHEATS_IN_RELEASE -D_GNU_SOURCE \
//     -Wno-macro-redefined -Wno-switch -Wno-invalid-offsetof \
//     -Itests/cpp_harness/stubs_wp3_2c \
//     -Ivendor/GeneralsX/GeneralsMD/Code/GameEngine/Include \
//     -Ivendor/GeneralsX/Core/GameEngine/Include \
//     -Ivendor/GeneralsX/build/macos-vulkan/_deps/gamemath-src/include \
//     tests/cpp_harness/gen_wp3_2c_rules.cpp \
//     vendor/GeneralsX/build/macos-vulkan/_deps/gamemath-build/libgm.a \
//     -o /tmp/gen_wp3_2c && /tmp/gen_wp3_2c docs/golden_vectors/WP3_2c_Rules.json
//
// Determinism & UB verification builds (must produce byte-identical JSON):
//   -O0, -O1, -O2, and -O1 -g -fsanitize=address,undefined (-std=c++20).

#define DEFINE_DEATH_NAMES
#include "PreRTS.h"
#include "Common/crc.h"
#include "Common/Dict.h"
#include "Common/INI.h"
#include "Common/Xfer.h"
#include "Common/XferCRC.h"
#include "Common/XferSave.h"
#include "Common/XferLoad.h"
#include "Common/FileSystem.h"
#include "Common/Money.h"
#include "Common/Handicap.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/Science.h"
#include "Common/Upgrade.h"
#include "Common/SpecialPower.h"
#include "GameLogic/Damage.h"
#include "GameLogic/Object.h"
#include "Common/MultiplayerSettings.h"
#include "Common/PlayerTemplate.h"
#include "Common/ProductionPrerequisite.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "GameClient/GameText.h"
#include "GameClient/Image.h"
#include "GameClient/InGameUI.h"
#include "GameNetwork/GameInfo.h"

#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/MultiplayerSettings.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/INI/INIMultiplayer.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/PlayerTemplate.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/ProductionPrerequisite.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/Money.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/Handicap.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/Science.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/System/Upgrade.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/SpecialPower.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/INI/INIUpgrade.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/INI/INISpecialPower.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/GameLogic/System/Damage.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/Dict.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/NameKeyGenerator.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/Xfer.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/XferCRC.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/XferSave.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/XferLoad.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/crc.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/RandomValue.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/INI/INI.cpp"

#include <algorithm>
#include <array>
#include <cstdio>
#include <cstring>
#include <string>
#include <vector>

const AsciiString AsciiString::TheEmptyString("");
const UnicodeString UnicodeString::TheEmptyString;

const char* const TheVeterancyNames[] = {
    "REGULAR", "VETERAN", "ELITE", "HEROIC", nullptr
};
static_assert(ARRAY_SIZE(TheVeterancyNames) == LEVEL_COUNT + 1, "Incorrect array size");

const char* const TheAcademyClassificationTypeNames[] = {
    "ACT_NONE", "ACT_UPGRADE_RADAR", "ACT_SUPERPOWER", nullptr
};
static_assert(ARRAY_SIZE(TheAcademyClassificationTypeNames) == ACT_COUNT + 1, "Incorrect array size");

SpecialPowerModuleInterface* Object::getSpecialPowerModule(const SpecialPowerTemplate*) const {
    return nullptr;
}

static DynamicMemoryAllocator g_dma;
DynamicMemoryAllocator* TheDynamicMemoryAllocator = &g_dma;
static GameState g_game_state;
GameState* TheGameState = &g_game_state;
static FileSystem g_fs;
FileSystem* TheFileSystem = &g_fs;
static ThingFactory g_thing_factory;
ThingFactory* TheThingFactory = &g_thing_factory;
static GameTextInterface g_game_text;
GameTextInterface* TheGameText = &g_game_text;
static ImageCollection g_images;
ImageCollection* TheMappedImageCollection = &g_images;
static InGameUI g_in_game_ui;
InGameUI* TheInGameUI = &g_in_game_ui;
static AudioManager g_audio;
AudioManager* TheAudio = &g_audio;
static PlayerList g_player_list;
PlayerList* ThePlayerList = &g_player_list;
static GameLogic g_game_logic;
GameLogic* TheGameLogic = &g_game_logic;

ArmorStore* TheArmorStore = nullptr;
DamageFXStore* TheDamageFXStore = nullptr;
FXListStore* TheFXListStore = nullptr;
Anim2DCollection* TheAnim2DCollection = nullptr;
ParticleSystemManager* TheParticleSystemManager = nullptr;
ObjectCreationListStore* TheObjectCreationListStore = nullptr;
WeaponStore* TheWeaponStore = nullptr;

#define STUB_PARSE(fn) void INI::fn(INI*) {}
STUB_PARSE(parseAIDataDefinition)
STUB_PARSE(parseAnim2DDefinition)
STUB_PARSE(parseArmorDefinition)
STUB_PARSE(parseAudioEventDefinition)
STUB_PARSE(parseAudioSettingsDefinition)
STUB_PARSE(parseBenchProfile)
STUB_PARSE(parseTerrainBridgeDefinition)
STUB_PARSE(parseCampaignDefinition)
STUB_PARSE(parseChallengeModeDefinition)
STUB_PARSE(parseCommandButtonDefinition)
STUB_PARSE(parseMetaMapDefinition)
STUB_PARSE(parseCommandSetDefinition)
STUB_PARSE(parseControlBarResizerDefinition)
STUB_PARSE(parseControlBarSchemeDefinition)
STUB_PARSE(parseCrateTemplateDefinition)
STUB_PARSE(parseCredits)
STUB_PARSE(parseDamageFXDefinition)
STUB_PARSE(parseDialogDefinition)
STUB_PARSE(parseDrawGroupNumberDefinition)
STUB_PARSE(parseDynamicGameLODDefinition)
STUB_PARSE(parseEvaEvent)
STUB_PARSE(parseFXListDefinition)
STUB_PARSE(parseGameDataDefinition)
STUB_PARSE(parseHeaderTemplateDefinition)
STUB_PARSE(parseInGameUIDefinition)
STUB_PARSE(parseLODPreset)
STUB_PARSE(parseLanguageDefinition)
STUB_PARSE(parseLocomotorTemplateDefinition)
STUB_PARSE(parseMapCacheDefinition)
STUB_PARSE(parseMapDataDefinition)
STUB_PARSE(parseMappedImageDefinition)
STUB_PARSE(parseMiscAudio)
STUB_PARSE(parseMouseDefinition)
STUB_PARSE(parseMouseCursorDefinition)
STUB_PARSE(parseMusicTrackDefinition)
STUB_PARSE(parseObjectDefinition)
STUB_PARSE(parseObjectCreationListDefinition)
STUB_PARSE(parseObjectReskinDefinition)
STUB_PARSE(parseOnlineChatColorDefinition)
STUB_PARSE(parseParticleSystemDefinition)
STUB_PARSE(parseStaticGameLODDefinition)
STUB_PARSE(parseTerrainDefinition)
STUB_PARSE(parseTerrainRoadDefinition)
STUB_PARSE(parseRankDefinition)
STUB_PARSE(parseVideoDefinition)
STUB_PARSE(parseWaterSettingDefinition)
STUB_PARSE(parseWaterTransparencyDefinition)
STUB_PARSE(parseWeaponTemplateDefinition)
STUB_PARSE(parseWeatherDefinition)
STUB_PARSE(parseWebpageURLDefinition)
STUB_PARSE(parseWindowTransitions)
STUB_PARSE(parseShellMenuSchemeDefinition)
void parseReallyLowMHz(INI*) {}

static std::string sha256_file(const char* path) {
    static const uint32_t K[64] = {
        0x428a2f98u, 0x71374491u, 0xb5c0fbcfu, 0xe9b5dba5u, 0x3956c25bu, 0x59f111f1u, 0x923f82a4u, 0xab1c5ed5u,
        0xd807aa98u, 0x12835b01u, 0x243185beu, 0x550c7dc3u, 0x72be5d74u, 0x80deb1feu, 0x9bdc06a7u, 0xc19bf174u,
        0xe49b69c1u, 0xefbe4786u, 0x0fc19dc6u, 0x240ca1ccu, 0x2de92c6fu, 0x4a7484aau, 0x5cb0a9dcu, 0x76f988dau,
        0x983e5152u, 0xa831c66du, 0xb00327c8u, 0xbf597fc7u, 0xc6e00bf3u, 0xd5a79147u, 0x06ca6351u, 0x14292967u,
        0x27b70a85u, 0x2e1b2138u, 0x4d2c6dfcu, 0x53380d13u, 0x650a7354u, 0x766a0abbu, 0x81c2c92eu, 0x92722c85u,
        0xa2bfe8a1u, 0xa81a664bu, 0xc24b8b70u, 0xc76c51a3u, 0xd192e819u, 0xd6990624u, 0xf40e3585u, 0x106aa070u,
        0x19a4c116u, 0x1e376c08u, 0x2748774cu, 0x34b0bcb5u, 0x391c0cb3u, 0x4ed8aa4au, 0x5b9cca4fu, 0x682e6ff3u,
        0x748f82eeu, 0x78a5636fu, 0x84c87814u, 0x8cc70208u, 0x90befffau, 0xa4506cebu, 0xbef9a3f7u, 0xc67178f2u
    };
    FILE* f = std::fopen(path, "rb");
    if (!f) { std::perror(path); std::exit(1); }
    std::vector<uint8_t> data;
    std::fseek(f, 0, SEEK_END);
    long sz = std::ftell(f);
    std::fseek(f, 0, SEEK_SET);
    if (sz > 0) {
        data.resize(static_cast<size_t>(sz));
        if (std::fread(data.data(), 1, static_cast<size_t>(sz), f) != static_cast<size_t>(sz)) {
            std::perror("fread");
            std::exit(1);
        }
    }
    std::fclose(f);
    uint64_t bit_len = static_cast<uint64_t>(data.size()) * 8ULL;
    data.push_back(0x80u);
    while ((data.size() % 64u) != 56u) data.push_back(0x00u);
    for (int shift = 56; shift >= 0; shift -= 8) {
        data.push_back(static_cast<uint8_t>((bit_len >> shift) & 0xFFu));
    }
    uint32_t H[8] = {
        0x6a09e667u, 0xbb67ae85u, 0x3c6ef372u, 0xa54ff53au,
        0x510e527fu, 0x9b05688cu, 0x1f83d9abu, 0x5be0cd19u
    };
    auto rotr = [](uint32_t x, uint32_t n) -> uint32_t { return (x >> n) | (x << (32u - n)); };
    for (size_t off = 0; off < data.size(); off += 64u) {
        uint32_t w[64]{};
        for (int i = 0; i < 16; ++i) {
            w[i] = (static_cast<uint32_t>(data[off + i * 4]) << 24) |
                   (static_cast<uint32_t>(data[off + i * 4 + 1]) << 16) |
                   (static_cast<uint32_t>(data[off + i * 4 + 2]) << 8) |
                   static_cast<uint32_t>(data[off + i * 4 + 3]);
        }
        for (int i = 16; i < 64; ++i) {
            uint32_t s0 = rotr(w[i - 15], 7) ^ rotr(w[i - 15], 18) ^ (w[i - 15] >> 3);
            uint32_t s1 = rotr(w[i - 2], 17) ^ rotr(w[i - 2], 19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16] + s0 + w[i - 7] + s1;
        }
        uint32_t a = H[0], b = H[1], c = H[2], d = H[3];
        uint32_t e = H[4], f0 = H[5], g = H[6], h = H[7];
        for (int i = 0; i < 64; ++i) {
            uint32_t S1 = rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25);
            uint32_t ch = (e & f0) ^ ((~e) & g);
            uint32_t temp1 = h + S1 + ch + K[i] + w[i];
            uint32_t S0 = rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22);
            uint32_t maj = (a & b) ^ (a & c) ^ (b & c);
            uint32_t temp2 = S0 + maj;
            h = g; g = f0; f0 = e; e = d + temp1;
            d = c; c = b; b = a; a = temp1 + temp2;
        }
        H[0] += a; H[1] += b; H[2] += c; H[3] += d;
        H[4] += e; H[5] += f0; H[6] += g; H[7] += h;
    }
    char hex[65];
    for (int i = 0; i < 8; ++i) std::snprintf(hex + i * 8, 9, "%08x", H[i]);
    hex[64] = '\0';
    return std::string(hex);
}

static std::string json_escape(const std::string& s) {
    std::string out;
    for (unsigned char c : s) {
        if (c == '\\') out += "\\\\";
        else if (c == '"') out += "\\\"";
        else if (c == '\n') out += "\\n";
        else if (c == '\r') out += "\\r";
        else if (c == '\t') out += "\\t";
        else out.push_back(static_cast<char>(c));
    }
    return out;
}

static std::string hex_u16_slice(const UnicodeString& u) {
    static const char* kHex = "0123456789abcdef";
    std::string out;
    const Int len = u.getLength();
    const WideChar* p = u.str();
    for (Int i = 0; i < len; ++i) {
        uint16_t v = p[i];
        uint8_t lo = static_cast<uint8_t>(v & 0xffu);
        uint8_t hi = static_cast<uint8_t>((v >> 8) & 0xffu);
        out.push_back(kHex[lo >> 4]);
        out.push_back(kHex[lo & 0xfu]);
        out.push_back(kHex[hi >> 4]);
        out.push_back(kHex[hi & 0xfu]);
    }
    return out;
}

static uint32_t f32_bits(Real v) {
    uint32_t u = 0;
    std::memcpy(&u, &v, 4);
    return u;
}

struct Rng {
    uint32_t state;
    explicit Rng(uint32_t seed) : state(seed == 0 ? 0x6d2b79f5u : seed) {}
    uint32_t next_u32() {
        uint32_t x = state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        state = x;
        return x;
    }
    uint32_t range(uint32_t lo, uint32_t hi_excl) {
        return lo + (next_u32() % (hi_excl - lo));
    }
};

static void load_ini_string(const std::string& content, INILoadType loadType = INI_LOAD_OVERWRITE) {
    g_fs.nextFileContent = content;
    g_fs.nextFileExists = true;
    INI ini;
    try {
        ini.load(AsciiString("wp3_2c.ini"), loadType, nullptr);
    } catch (...) {
        std::fprintf(stderr, "INIException caught!\nContent:\n%s\n", content.c_str());
        throw;
    }
}

static void init_name_keys_once() {
    delete TheNameKeyGenerator;
    TheNameKeyGenerator = new NameKeyGenerator();
    TheNameKeyGenerator->init();
}

static void reset_all_stores() {
    delete TheMultiplayerSettings;
    TheMultiplayerSettings = nullptr;
    delete ThePlayerTemplateStore;
    ThePlayerTemplateStore = nullptr;
    delete TheScienceStore;
    TheScienceStore = nullptr;

    TheScienceStore = new ScienceStore();
    TheScienceStore->init();
    ThePlayerTemplateStore = new PlayerTemplateStore();
    ThePlayerTemplateStore->init();
    // Leave TheMultiplayerSettings nullptr so first MultiplayerSettings INI block allocates it cleanly.
}

static void run_self_checks() {
    reset_all_stores();
    // 1. Default MultiplayerSettings starting money choices include 100k/150k/200k ascending.
    TheMultiplayerSettings = new MultiplayerSettings();
    const auto& defaults = TheMultiplayerSettings->getStartingMoneyList();
    if (defaults.size() != 3 ||
        defaults[0].countMoney() != 100000 ||
        defaults[1].countMoney() != 150000 ||
        defaults[2].countMoney() != 200000) {
        std::fprintf(stderr, "Self-check failed: MultiplayerSettings default money list\n");
        std::exit(1);
    }
    if (TheMultiplayerSettings->getColor(PLAYERTEMPLATE_RANDOM) == nullptr ||
        TheMultiplayerSettings->getColor(PLAYERTEMPLATE_OBSERVER) == nullptr ||
        TheMultiplayerSettings->getColor(0) != nullptr ||
        TheMultiplayerSettings->getColor(-3) != nullptr) {
        std::fprintf(stderr, "Self-check failed: MultiplayerSettings getColor boundaries\n");
        std::exit(1);
    }

    // 2. PlayerTemplate legacy remap + overwrite-in-place behavior.
    load_ini_string(
        "PlayerTemplate FactionAmerica\n"
        "  Side = America\n"
        "  BaseSide = America\n"
        "  PlayableSide = Yes\n"
        "End\n"
        "PlayerTemplate FactionAmericaTankCommand\n"
        "  Side = TankGen\n"
        "End\n"
    );
    if (ThePlayerTemplateStore->getPlayerTemplateCount() != 1) {
        std::fprintf(stderr, "Self-check failed: legacy remap should overwrite in place\n");
        std::exit(1);
    }
    const PlayerTemplate* remapped = ThePlayerTemplateStore->getNthPlayerTemplate(0);
    if (!remapped || remapped->getName() != "FactionAmericaTankCommand" ||
        remapped->getSide() != "TankGen" || remapped->getBaseSide() != "America") {
        std::fprintf(stderr, "Self-check failed: remapped PlayerTemplate fields mismatch\n");
        std::exit(1);
    }
    // Note PlayerTemplateStore::reset() is a documented no-op.
    ThePlayerTemplateStore->reset();
    if (ThePlayerTemplateStore->getPlayerTemplateCount() != 1) {
        std::fprintf(stderr, "Self-check failed: PlayerTemplateStore::reset must keep templates\n");
        std::exit(1);
    }

    // 3. ProductionPrerequisite unresolved names + MAX_PREREQ=32 cap.
    const ThingTemplate* t_a = g_thing_factory.registerTemplate("UnitA", "DispA", TRUE);
    ProductionPrerequisite pp;
    pp.addUnitPrereq(AsciiString("MissingTemplate"), FALSE);
    pp.addUnitPrereq(AsciiString("UnitA"), TRUE);
    pp.resolveNames();
    Player p;
    p.setOwnedCount(t_a, 1);
    if (!pp.isSatisfied(&p)) {
        std::fprintf(stderr, "Self-check failed: MissingTemplate OR UnitA should be satisfied\n");
        std::exit(1);
    }
    if (pp.isSatisfied(nullptr) || !pp.getRequiresList(nullptr).isEmpty()) {
        std::fprintf(stderr, "Self-check failed: nullptr player contract\n");
        std::exit(1);
    }

    ProductionPrerequisite pp_cap;
    for (int i = 0; i < 36; ++i) {
        char buf[32];
        std::snprintf(buf, sizeof(buf), "CapUnit%d", i);
        const ThingTemplate* t_i = g_thing_factory.registerTemplate(buf, buf, FALSE);
        pp_cap.addUnitPrereq(AsciiString(buf), FALSE);
        if (i < 32) {
            p.setOwnedCount(t_i, 1);
        }
    }
    pp_cap.resolveNames();
    if (!pp_cap.isSatisfied(&p)) {
        std::fprintf(stderr, "Self-check failed: MAX_PREREQ=32 cap not respected\n");
        std::exit(1);
    }
}

int main(int argc, char** argv) {
    if (argc != 2) {
        std::fprintf(stderr, "Usage: %s <output.json>\n", argv[0]);
        return 1;
    }

    init_name_keys_once();

    // Register fixed pool of 12 ThingTemplates once (deterministic map ordering).
    std::array<const ThingTemplate*, 12> tmpls{};
    std::array<std::string, 12> tmpl_names{};
    for (int i = 0; i < 12; ++i) {
        char nbuf[32], dbuf[32];
        std::snprintf(nbuf, sizeof(nbuf), "TMPL_%02d", i);
        std::snprintf(dbuf, sizeof(dbuf), "DISP_%02d", i);
        tmpl_names[i] = nbuf;
        tmpls[i] = g_thing_factory.registerTemplate(nbuf, dbuf, (i % 2 == 0) ? TRUE : FALSE);
    }

    static const char* kLegacyRemaps[12] = {
        "FactionAmericaChooseAGeneral", "FactionAmericaTankCommand",
        "FactionAmericaSpecialForces", "FactionAmericaAirForce",
        "FactionChinaChooseAGeneral", "FactionChinaRedArmy",
        "FactionChinaSpecialWeapons", "FactionChinaSecretPolice",
        "FactionGLAChooseAGeneral", "FactionGLATerrorCell",
        "FactionGLABiowarCommand", "FactionGLAWarlordCommand"
    };
    static const char* kBaseFactions[3] = { "FactionAmerica", "FactionChina", "FactionGLA" };
    static const char* kVetNames[4] = { "REGULAR", "VETERAN", "ELITE", "HEROIC" };
    static const char* kSidePool[5] = { "America", "China", "GLA", "Civilian", "Observer" };
    static const char* kImagePool[6] = {
        "BTN_FOUND", "IMG_HW_1", "IMG_FW_1", "IMG_SIDE_1", "IMG_GEN_1", "IMG_MISS"
    };

    const std::string sha_mph = sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/MultiplayerSettings.h");
    const std::string sha_mpc = sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/MultiplayerSettings.cpp");
    const std::string sha_imp = sha256_file("vendor/GeneralsX/Core/GameEngine/Source/Common/INI/INIMultiplayer.cpp");
    const std::string sha_pth = sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/PlayerTemplate.h");
    const std::string sha_ptc = sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/PlayerTemplate.cpp");
    const std::string sha_pph = sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ProductionPrerequisite.h");
    const std::string sha_ppc = sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/ProductionPrerequisite.cpp");

    FILE* out = std::fopen(argv[1], "wb");
    if (!out) { std::perror(argv[1]); return 1; }

    std::fprintf(out, "{\n");
    std::fprintf(out, "  \"work_package\": \"WP-3.2c\",\n");
    std::fprintf(out, "  \"reference_commit\": \"84a7a20769aa5749f388b1f49bd542c79b3c33e8\",\n");
    std::fprintf(out, "  \"vector_count\": 1000,\n");
    std::fprintf(out, "  \"constants\": {\n");
    std::fprintf(out, "    \"PLAYERTEMPLATE_RANDOM\": %d,\n", PLAYERTEMPLATE_RANDOM);
    std::fprintf(out, "    \"PLAYERTEMPLATE_OBSERVER\": %d,\n", PLAYERTEMPLATE_OBSERVER);
    std::fprintf(out, "    \"PLAYERTEMPLATE_MIN\": %d,\n", PLAYERTEMPLATE_MIN);
    std::fprintf(out, "    \"GAME_COLOR_UNDEFINED\": %u,\n", GAME_COLOR_UNDEFINED);
    std::fprintf(out, "    \"MAX_MP_STARTING_UNITS\": %d,\n", MAX_MP_STARTING_UNITS);
    std::fprintf(out, "    \"MAX_PREREQ\": 32\n");
    std::fprintf(out, "  },\n");
    std::fprintf(out, "  \"source_sha256\": {\n");
    std::fprintf(out, "    \"MultiplayerSettings.h\": \"%s\",\n", sha_mph.c_str());
    std::fprintf(out, "    \"MultiplayerSettings.cpp\": \"%s\",\n", sha_mpc.c_str());
    std::fprintf(out, "    \"INIMultiplayer.cpp\": \"%s\",\n", sha_imp.c_str());
    std::fprintf(out, "    \"PlayerTemplate.h\": \"%s\",\n", sha_pth.c_str());
    std::fprintf(out, "    \"PlayerTemplate.cpp\": \"%s\",\n", sha_ptc.c_str());
    std::fprintf(out, "    \"ProductionPrerequisite.h\": \"%s\",\n", sha_pph.c_str());
    std::fprintf(out, "    \"ProductionPrerequisite.cpp\": \"%s\"\n", sha_ppc.c_str());
    std::fprintf(out, "  },\n");
    std::fprintf(out, "  \"vectors\": [\n");

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        reset_all_stores();
        Rng rng((seed + 1u) * 0x9e3779b9u);

        // Seed Sciences needed for PlayerTemplate & ProductionPrerequisite.
        load_ini_string(
            "Science SCI_A\n  PrerequisiteSciences = None\n  SciencePurchasePointCost = 1\nEnd\n"
            "Science SCI_B\n  PrerequisiteSciences = SCI_A\n  SciencePurchasePointCost = 2\nEnd\n",
            INI_LOAD_OVERWRITE
        );

        // --- Part 1: MultiplayerSettings + INIMultiplayer ---
        std::string mp_ini;
        char buf[2048];
        Int timer = static_cast<Int>(rng.range(0, 30));
        Int beacons = static_cast<Int>(rng.range(1, 10));
        Bool shroud = static_cast<Bool>(rng.range(0, 2));
        Bool rand_pt = static_cast<Bool>(rng.range(0, 2));
        Bool rand_pos = static_cast<Bool>(rng.range(0, 2));
        Bool rand_col = static_cast<Bool>(rng.range(0, 2));

        std::snprintf(
            buf, sizeof(buf),
            "MultiplayerSettings\n"
            "  StartCountdownTimer = %d\n"
            "  MaxBeaconsPerPlayer = %d\n"
            "  UseShroud = %s\n"
            "  ShowRandomPlayerTemplate = %s\n"
            "  ShowRandomStartPos = %s\n"
            "  ShowRandomColor = %s\n"
            "End\n",
            timer, beacons,
            shroud ? "Yes" : "No",
            rand_pt ? "Yes" : "No",
            rand_pos ? "Yes" : "No",
            rand_col ? "Yes" : "No"
        );
        mp_ini += buf;

        // Sometimes ensure TheMultiplayerSettings was pre-existing (no error on re-parse in Overwrite)
        Int color_blocks = static_cast<Int>(rng.range(1, 5));
        for (Int c = 0; c < color_blocks; ++c) {
            // Test both matching tooltip name (so overwrite fires if repeated) and distinct tooltip name.
            const char* block_name = (c == color_blocks - 1 && (seed % 3 == 0)) ? "Tip0" : (c == 0 ? "Red" : (c == 1 ? "Blue" : "Green"));
            const char* tip_name = (c == 0 && (seed % 3 == 0)) ? "Tip0" : (c == 0 ? "GUI:Red" : (c == 1 ? "Blue" : "GUI:Green"));
            Int r1 = static_cast<Int>(rng.range(0, 256)), g1 = static_cast<Int>(rng.range(0, 256)), b1 = static_cast<Int>(rng.range(0, 256));
            Int r2 = static_cast<Int>(rng.range(0, 256)), g2 = static_cast<Int>(rng.range(0, 256)), b2 = static_cast<Int>(rng.range(0, 256));
            std::snprintf(
                buf, sizeof(buf),
                "MultiplayerColor %s\n"
                "  TooltipName = %s\n"
                "  RGBColor = R:%d G:%d B:%d\n"
                "  RGBNightColor = R:%d G:%d B:%d\n"
                "End\n",
                block_name, tip_name, r1, g1, b1, r2, g2, b2
            );
            mp_ini += buf;
        }

        Int money_choices = static_cast<Int>(rng.range(1, 4));
        Int default_choice_idx = static_cast<Int>(rng.range(0, static_cast<uint32_t>(money_choices)));
        for (Int m = 0; m < money_choices; ++m) {
            UnsignedInt val = rng.range(1, 60) * 1000u;
            Bool is_def = (m == default_choice_idx) ? TRUE : FALSE;
            std::snprintf(
                buf, sizeof(buf),
                "MultiplayerStartingMoneyChoice\n"
                "  Value = %u\n"
                "  Default = %s\n"
                "End\n",
                val, is_def ? "Yes" : "No"
            );
            mp_ini += buf;
        }
        load_ini_string(mp_ini, INI_LOAD_OVERWRITE);

        // --- Part 2: PlayerTemplate + PlayerTemplateStore ---
        std::string pt_ini;
        Int pt_blocks = static_cast<Int>(rng.range(2, 5));
        for (Int p = 0; p < pt_blocks; ++p) {
            std::string bname;
            if (p == 0) bname = kBaseFactions[seed % 3];
            else if (p == 1 && (seed % 2 == 0)) bname = kLegacyRemaps[rng.range(0, 12)];
            else {
                std::snprintf(buf, sizeof(buf), "FactionGen_%u_%d", seed % 17, p);
                bname = buf;
            }
            const char* side = kSidePool[rng.range(0, 5)];
            const char* bside = kSidePool[rng.range(0, 3)];
            Bool playable = (rng.range(0, 2) != 0);
            Bool is_obs = (!playable && rng.range(0, 3) == 0);
            Bool old_fac = (rng.range(0, 2) != 0);
            UnsignedInt start_money = rng.range(0, 50) * 500u;
            Int cr = static_cast<Int>(rng.range(0, 256)), cg = static_cast<Int>(rng.range(0, 256)), cb = static_cast<Int>(rng.range(0, 256));
            Int su_a = static_cast<Int>(rng.range(0, 10));
            Int su_b = static_cast<Int>(rng.range(0, 10));
            Int cost_pct = static_cast<Int>(rng.range(50, 150)) - 100; // negative & positive
            UnsignedInt time_pct = rng.range(50, 160);
            const char* vet = kVetNames[rng.range(0, 4)];
            const char* sci_str = (rng.range(0, 3) == 0) ? "None" : ((rng.range(0, 2) == 0) ? "SCI_A" : "SCI_A SCI_B");
            Int spp = static_cast<Int>(rng.range(0, 6));
            Int sp_btns = static_cast<Int>(rng.range(0, 12));
            const char* hw_img = kImagePool[rng.range(0, 6)];
            const char* en_img = kImagePool[rng.range(0, 6)];

            std::snprintf(
                buf, sizeof(buf),
                "PlayerTemplate %s\n"
                "  Side = %s\n"
                "  BaseSide = %s\n"
                "  PlayableSide = %s\n"
                "  DisplayName = NAME:%s\n"
                "  StartMoney = %u\n"
                "  PreferredColor = R:%d G:%d B:%d\n"
                "  StartingBuilding = Bldg_%s\n"
                "  StartingUnit%d = UnitStartA_%d\n"
                "  StartingUnit%d = UnitStartB_%d\n"
                "  ProductionCostChange = %s %d%%\n"
                "  ProductionTimeChange = %s %u%%\n"
                "  ProductionVeterancyLevel = %s %s\n"
                "  IntrinsicSciences = %s\n"
                "  IsObserver = %s\n"
                "  OldFaction = %s\n"
                "  IntrinsicSciencePurchasePoints = %d\n"
                "  SpecialPowerShortcutButtonCount = %d\n"
                "  HeadWaterMark = %s\n"
                "  EnabledImage = %s\n"
                "  BeaconName = Beacon_%u\n"
                "End\n",
                bname.c_str(), side, bside,
                playable ? "Yes" : "No",
                bname.c_str(), start_money, cr, cg, cb, side,
                su_a, su_a, su_b, su_b,
                tmpl_names[rng.range(0, 4)].c_str(), cost_pct,
                tmpl_names[rng.range(0, 4)].c_str(), time_pct,
                tmpl_names[rng.range(0, 4)].c_str(), vet,
                sci_str,
                is_obs ? "Yes" : "No",
                old_fac ? "Yes" : "No",
                spp, sp_btns, hw_img, en_img, seed % 5
            );
            pt_ini += buf;
        }
        load_ini_string(pt_ini, INI_LOAD_OVERWRITE);
        ThePlayerTemplateStore->reset();

        AsciiStringList sides_out;
        sides_out.push_back(AsciiString("SeedSide"));
        ThePlayerTemplateStore->getAllSideStrings(&sides_out);

        // --- Part 3: ProductionPrerequisite ---
        ProductionPrerequisite pp;
        Int sci_prereq_mode = static_cast<Int>(rng.range(0, 4));
        if (sci_prereq_mode & 1) pp.addSciencePrereq(INI::scanScience("SCI_A"));
        if (sci_prereq_mode & 2) pp.addSciencePrereq(INI::scanScience("SCI_B"));

        Int ucount = static_cast<Int>(rng.range(0, 8));
        // Store prereq construction ops so Rust replays identical calls.
        struct OpRec { Int idx; Bool or_prev; Int mode; }; // mode 0: addUnitPrereq(single), 1: vec len2
        std::vector<OpRec> pp_ops;
        for (Int u = 0; u < ucount; ++u) {
            if (u + 1 < ucount && rng.range(0, 3) == 0) {
                Int idx0 = static_cast<Int>(rng.range(0, 12));
                Int idx1 = static_cast<Int>(rng.range(0, 12));
                std::vector<AsciiString> vec{
                    AsciiString(tmpl_names[idx0].c_str()),
                    AsciiString(tmpl_names[idx1].c_str())
                };
                pp.addUnitPrereq(vec);
                pp_ops.push_back({idx0, FALSE, 1});
                pp_ops.push_back({idx1, TRUE, 2});
                ++u;
            } else {
                Int idx0 = static_cast<Int>(rng.range(0, 12));
                Bool or_prev = (u > 0 && rng.range(0, 2) == 0) ? TRUE : FALSE;
                pp.addUnitPrereq(AsciiString(tmpl_names[idx0].c_str()), or_prev);
                pp_ops.push_back({idx0, or_prev, 0});
            }
        }
        pp.resolveNames();

        Player pp_player;
        uint32_t player_sci_mask = rng.range(0, 4);
        if (player_sci_mask & 1u) pp_player.grantScience(INI::scanScience("SCI_A"));
        if (player_sci_mask & 2u) pp_player.grantScience(INI::scanScience("SCI_B"));

        uint32_t owned_mask = rng.next_u32() & 0x0FFFu;
        for (int t = 0; t < 12; ++t) {
            if (owned_mask & (1u << t)) {
                pp_player.setOwnedCount(tmpls[t], static_cast<Int>(rng.range(1, 4)));
            }
        }
        Bool pp_sat = pp.isSatisfied(&pp_player);
        const ThingTemplate* fac = pp.getExistingBuildFacilityTemplate(&pp_player);
        const ThingTemplate* all_facs[8]{};
        Int fac_cap = (seed % 3 == 0) ? 1 : 8;
        Int fac_cnt = pp.getAllPossibleBuildFacilityTemplates(all_facs, fac_cap);
        UnicodeString req_list = pp.getRequiresList(&pp_player);

        // Build JSON line
        std::fprintf(out, "    {\"seed\":%u", seed);
        std::fprintf(out, ",\"mp_ini\":\"%s\"", json_escape(mp_ini).c_str());
        std::fprintf(out, ",\"mp_timer\":%d,\"mp_beacons\":%d",
                     TheMultiplayerSettings->getStartCountdownTimerSeconds(),
                     TheMultiplayerSettings->getMaxBeaconsPerPlayer());
        std::fprintf(out, ",\"mp_flags\":[%u,%u,%u,%u]",
                     TheMultiplayerSettings->isShroudInMultiplayer(),
                     TheMultiplayerSettings->showRandomPlayerTemplate(),
                     TheMultiplayerSettings->showRandomStartPos(),
                     TheMultiplayerSettings->showRandomColor());
        std::fprintf(out, ",\"mp_num_colors\":%d", TheMultiplayerSettings->getNumColors());
        std::fprintf(out, ",\"mp_money_list\":[");
        const auto& mlist = TheMultiplayerSettings->getStartingMoneyList();
        for (size_t i = 0; i < mlist.size(); ++i) {
            if (i) std::fputc(',', out);
            std::fprintf(out, "%u", mlist[i].countMoney());
        }
        std::fprintf(out, "],\"mp_default_money\":%u",
                     TheMultiplayerSettings->getDefaultStartingMoney().countMoney());

        std::fprintf(out, ",\"mp_colors\":[");
        for (Int i = PLAYERTEMPLATE_OBSERVER; i <= TheMultiplayerSettings->getNumColors(); ++i) {
            if (i > PLAYERTEMPLATE_OBSERVER) std::fputc(',', out);
            const MultiplayerColorDefinition* cd = TheMultiplayerSettings->getColor(i);
            if (!cd) {
                std::fprintf(out, "[-1,0]");
            } else {
                std::fprintf(out, "[%d,%d]", cd->getColor(), cd->getNightColor());
            }
        }
        std::fprintf(out, "]");

        std::fprintf(out, ",\"pt_ini\":\"%s\"", json_escape(pt_ini).c_str());
        std::fprintf(out, ",\"pt_count\":%d", ThePlayerTemplateStore->getPlayerTemplateCount());
        std::fprintf(out, ",\"pt_sides\":[");
        bool first_side = true;
        for (const auto& s : sides_out) {
            if (!first_side) std::fputc(',', out);
            first_side = false;
            std::fprintf(out, "\"%s\"", json_escape(s.str()).c_str());
        }
        std::fprintf(out, "]");

        // Query legacy remap & case-insensitive name lookup
        const char* remap_q = kLegacyRemaps[seed % 12];
        const PlayerTemplate* remap_res = ThePlayerTemplateStore->findPlayerTemplate(
            TheNameKeyGenerator->nameToKey(remap_q)
        );
        Int num_by_name = ThePlayerTemplateStore->getTemplateNumByName(
            AsciiString(kBaseFactions[seed % 3])
        );
        Int num_miss = ThePlayerTemplateStore->getTemplateNumByName(AsciiString("DOES_NOT_EXIST"));
        std::fprintf(out, ",\"pt_remap_hit\":\"%s\"",
                     remap_res ? json_escape(remap_res->getName().str()).c_str() : "");
        std::fprintf(out, ",\"pt_lookup\":[%d,%d]", num_by_name, num_miss);

        std::fprintf(out, ",\"pt_entries\":[");
        for (Int i = 0; i < ThePlayerTemplateStore->getPlayerTemplateCount(); ++i) {
            const PlayerTemplate* pt = ThePlayerTemplateStore->getNthPlayerTemplate(i);
            if (i) std::fputc(',', out);
            std::fprintf(out, "[\"%s\",\"%s\",\"%s\",%u,%d,%d,%d,%d,%d,%u,%u]",
                         json_escape(pt->getName().str()).c_str(),
                         json_escape(pt->getSide().str()).c_str(),
                         json_escape(pt->getBaseSide().str()).c_str(),
                         pt->getMoney()->countMoney(),
                         pt->getPreferredColor()->getAsInt(),
                         pt->isPlayableSide(),
                         pt->isObserver(),
                         pt->isOldFaction(),
                         pt->getIntrinsicSciencePurchasePoints(),
                         pt->getHeadWaterMarkImage() ? 1u : 0u,
                         pt->getEnabledImage() ? 1u : 0u);
        }
        std::fprintf(out, "]");

        // Extra production map check on entry 0
        const PlayerTemplate* pt0 = ThePlayerTemplateStore->getNthPlayerTemplate(0);
        std::fprintf(out, ",\"pt0_maps\":[%zu,%zu,%zu,%zu]",
                     pt0->getProductionCostChanges().size(),
                     pt0->getProductionTimeChanges().size(),
                     pt0->getProductionVeterancyLevels().size(),
                     pt0->getIntrinsicSciences().size());
        uint32_t pt0_cost_sum = 0;
        for (const auto& kv : pt0->getProductionCostChanges()) {
            pt0_cost_sum ^= static_cast<uint32_t>(kv.first) ^ f32_bits(kv.second);
        }
        uint32_t pt0_vet_sum = 0;
        for (const auto& kv : pt0->getProductionVeterancyLevels()) {
            pt0_vet_sum ^= static_cast<uint32_t>(kv.first) ^ static_cast<uint32_t>(kv.second);
        }
        std::fprintf(out, ",\"pt0_hashes\":[%u,%u]", pt0_cost_sum, pt0_vet_sum);

        std::fprintf(out, ",\"pp_sci_req\":%d,\"pp_sci_own\":%u,\"pp_owned_mask\":%u,\"pp_fac_cap\":%d",
                     sci_prereq_mode, player_sci_mask, owned_mask, fac_cap);
        std::fprintf(out, ",\"pp_ops\":[");
        for (size_t i = 0; i < pp_ops.size(); ++i) {
            if (i) std::fputc(',', out);
            std::fprintf(out, "[%d,%d,%d]", pp_ops[i].idx, pp_ops[i].or_prev, pp_ops[i].mode);
        }
        std::fprintf(out, "],\"pp_sat\":%u,\"pp_fac\":\"%s\"",
                     pp_sat, fac ? fac->getName().str() : "");
        std::fprintf(out, ",\"pp_all_fac\":[");
        for (Int i = 0; i < fac_cnt; ++i) {
            if (i) std::fputc(',', out);
            std::fprintf(out, "\"%s\"", all_facs[i] ? all_facs[i]->getName().str() : "");
        }
        std::fprintf(out, "],\"pp_req_u16_hex\":\"%s\"}%s\n",
                     hex_u16_slice(req_list).c_str(),
                     (seed + 1u < 1000u) ? "," : "");
    }

    std::fprintf(out, "  ]\n}\n");
    std::fclose(out);
    run_self_checks();
    return 0;
}
