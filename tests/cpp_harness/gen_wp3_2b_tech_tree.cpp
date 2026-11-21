// WP-3.2b (PKT-133 Science + PKT-132 RankInfo + PKT-137 Upgrade + PKT-134 SpecialPower)
// C++ Golden Vector Generator.
// Compiles unmodified vendor/GeneralsX sources against minimal harness stubs.
//
// Canonical build & run command (macOS arm64, Release-aligned macros):
//   clang++ -std=c++17 -O1 -ffp-contract=off -URTS_DEBUG -U_ALLOW_DEBUG_CHEATS_IN_RELEASE -D_GNU_SOURCE \
//     -Wno-macro-redefined -Wno-switch -Wno-invalid-offsetof \
//     -Itests/cpp_harness/stubs_wp3_2b \
//     -Ivendor/GeneralsX/GeneralsMD/Code/GameEngine/Include \
//     -Ivendor/GeneralsX/Core/GameEngine/Include \
//     -Ivendor/GeneralsX/build/macos-vulkan/_deps/gamemath-src/include \
//     tests/cpp_harness/gen_wp3_2b_tech_tree.cpp \
//     vendor/GeneralsX/build/macos-vulkan/_deps/gamemath-build/libgm.a \
//     -o /tmp/gen_wp3_2b && /tmp/gen_wp3_2b docs/golden_vectors/WP3_2b_TechTree.json
//
// Determinism & UB verification builds (must produce byte-identical JSON):
//   clang++ -std=c++17 -O0 -ffp-contract=off -URTS_DEBUG -U_ALLOW_DEBUG_CHEATS_IN_RELEASE ... -o /tmp/gen_wp3_2b_o0
//   clang++ -std=c++17 -O2 -ffp-contract=off -URTS_DEBUG -U_ALLOW_DEBUG_CHEATS_IN_RELEASE ... -o /tmp/gen_wp3_2b_o2
//   clang++ -std=c++17 -O1 -g -fsanitize=address,undefined -fno-omit-frame-pointer -ffp-contract=off \
//     -URTS_DEBUG -U_ALLOW_DEBUG_CHEATS_IN_RELEASE ... -o /tmp/gen_wp3_2b_asan
//
// __APPLE__ audit on vendor files compiled here:
//   - Science.{h,cpp}, RankInfo.{h,cpp}, Upgrade.{h,cpp}, SpecialPower.{h,cpp}, Overridable.h: 0 hits.
//   - INI.cpp: 1 hit (line 1686, strtod fallback on macOS instead of std::from_chars); produces
//     identical finite f32 values for all BuildTime/ViewObjectRange/RadiusCursorRadius inputs here.

#define DEFINE_DEATH_NAMES
#include "PreRTS.h"
#include "Common/crc.h"
#include "Common/INI.h"
#include "Common/Xfer.h"
#include "Common/XferCRC.h"
#include "Common/XferSave.h"
#include "Common/XferLoad.h"
#include "Common/FileSystem.h"
#include "Common/Player.h"
#include "Common/Science.h"
#include "GameLogic/RankInfo.h"
#include "Common/Upgrade.h"
#include "Common/SpecialPower.h"
#include "GameLogic/Damage.h"
#include "GameLogic/Object.h"
#include "GameClient/GameText.h"
#include "GameClient/Image.h"
#include "GameClient/InGameUI.h"

#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/Science.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/GameLogic/System/RankInfo.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/System/Upgrade.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/SpecialPower.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/INI/INIUpgrade.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/INI/INISpecialPower.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/GameLogic/System/Damage.cpp"
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
#include <unistd.h>
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

SpecialPowerModuleInterface* Object::getSpecialPowerModule(const SpecialPowerTemplate* tmpl) const {
    if (!tmpl) return nullptr;
    UnsignedInt id = tmpl->getID();
    for (UnsignedInt supported : m_supportedSpecialPowerIds) {
        if (supported == id) {
            static SpecialPowerModuleInterface dummyModule;
            return &dummyModule;
        }
    }
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
STUB_PARSE(parseMultiplayerColorDefinition)
STUB_PARSE(parseMultiplayerSettingsDefinition)
STUB_PARSE(parseMultiplayerStartingMoneyChoiceDefinition)
STUB_PARSE(parseMusicTrackDefinition)
STUB_PARSE(parseObjectDefinition)
STUB_PARSE(parseObjectCreationListDefinition)
STUB_PARSE(parseObjectReskinDefinition)
STUB_PARSE(parseOnlineChatColorDefinition)
STUB_PARSE(parseParticleSystemDefinition)
STUB_PARSE(parsePlayerTemplateDefinition)
STUB_PARSE(parseTerrainRoadDefinition)
STUB_PARSE(parseShellMenuSchemeDefinition)
STUB_PARSE(parseStaticGameLODDefinition)
STUB_PARSE(parseTerrainDefinition)
STUB_PARSE(parseVideoDefinition)
STUB_PARSE(parseWaterSettingDefinition)
STUB_PARSE(parseWaterTransparencyDefinition)
STUB_PARSE(parseWeaponTemplateDefinition)
STUB_PARSE(parseWeatherDefinition)
STUB_PARSE(parseWebpageURLDefinition)
STUB_PARSE(parseWindowTransitions)
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

static uint32_t splitmix32(uint32_t& s) {
    uint32_t z = (s += 0x9e3779b9u);
    z = (z ^ (z >> 16)) * 0x85ebca6bu;
    z = (z ^ (z >> 13)) * 0xc2b2ae35u;
    return z ^ (z >> 16);
}

static uint32_t fbits(Real v) {
    uint32_t b = 0;
    std::memcpy(&b, &v, 4);
    return b;
}

static std::string to_hex(const uint8_t* p, size_t n) {
    static const char* kHex = "0123456789abcdef";
    std::string out;
    out.reserve(n * 2);
    for (size_t i = 0; i < n; ++i) {
        out.push_back(kHex[(p[i] >> 4) & 0xF]);
        out.push_back(kHex[p[i] & 0xF]);
    }
    return out;
}

static std::string str_hex(const std::string& s) {
    return to_hex(reinterpret_cast<const uint8_t*>(s.data()), s.size());
}

static std::vector<uint8_t> read_file_bytes(const char* path) {
    FILE* fp = std::fopen(path, "rb");
    if (!fp) { std::perror(path); std::exit(1); }
    std::fseek(fp, 0, SEEK_END);
    long sz = std::ftell(fp);
    std::fseek(fp, 0, SEEK_SET);
    std::vector<uint8_t> out(static_cast<size_t>(sz));
    if (sz > 0 && std::fread(out.data(), 1, static_cast<size_t>(sz), fp) != static_cast<size_t>(sz)) {
        std::perror("fread");
        std::exit(1);
    }
    std::fclose(fp);
    return out;
}

static std::string unicode_to_ascii(const UnicodeString& u) {
    std::string s;
    const WideChar* p = u.str();
    Int len = u.getLength();
    s.reserve(static_cast<size_t>(len));
    for (Int i = 0; i < len; ++i) {
        s.push_back(static_cast<char>(p[i] & 0x7Fu));
    }
    return s;
}

static void load_ini_string(const std::string& text, INILoadType mode) {
    g_fs.nextFileContent = text;
    g_fs.nextFileExists = true;
    INI ini;
    ini.load(AsciiString("Data\\INI\\Test.ini"), mode, nullptr);
}

template <typename Fn>
static bool expect_ini_invalid_data(Fn&& fn) {
    try {
        fn();
    } catch (const INIException&) {
        return true;
    } catch (decltype(INI_INVALID_DATA) code) {
        return code == INI_INVALID_DATA;
    }
    return false;
}

static void reset_all_stores() {
    delete TheSpecialPowerStore;
    TheSpecialPowerStore = nullptr;
    delete TheUpgradeCenter;
    TheUpgradeCenter = nullptr;
    delete TheRankInfoStore;
    TheRankInfoStore = nullptr;
    delete TheScienceStore;
    TheScienceStore = nullptr;
    delete TheNameKeyGenerator;
    TheNameKeyGenerator = nullptr;

    TheNameKeyGenerator = new NameKeyGenerator();
    TheNameKeyGenerator->init();
    TheScienceStore = new ScienceStore();
    TheScienceStore->init();
    TheRankInfoStore = new RankInfoStore();
    TheRankInfoStore->init();
    TheUpgradeCenter = new UpgradeCenter();
    TheSpecialPowerStore = new SpecialPowerStore();
    TheSpecialPowerStore->init();
    g_in_game_ui.resetStats();
}

static void run_startup_self_checks(const char* xfer_tmp) {
    static_assert(SPECIALPOWER_COUNT == 67, "Zero Hour SPECIALPOWER_COUNT must be 67");
    static_assert(UPGRADE_MAX_COUNT == 512, "Zero Hour UPGRADE_MAX_COUNT must be 512");
    static_assert(ACT_COUNT == 3, "ACT_COUNT must be 3");

    reset_all_stores();

    // 1. Duplicate Science in Overwrite throws INI_INVALID_DATA.
    load_ini_string(
        "Science SCI_ONE\n"
        "  PrerequisiteSciences = None\n"
        "  SciencePurchasePointCost = 1\n"
        "  IsGrantable = Yes\n"
        "  DisplayName = SCI:One\n"
        "  Description = SCI:OneDesc\n"
        "End\n",
        INI_LOAD_OVERWRITE);
    if (!expect_ini_invalid_data([]() {
            load_ini_string(
                "Science SCI_ONE\n"
                "  SciencePurchasePointCost = 2\n"
                "End\n",
                INI_LOAD_OVERWRITE);
        })) {
        std::fprintf(stderr, "Self-check failed: duplicate Science did not throw\n");
        std::exit(1);
    }

    // 2. Unknown PrerequisiteScience throws INI_INVALID_DATA.
    if (!expect_ini_invalid_data([]() {
            load_ini_string(
                "Science SCI_TWO\n"
                "  PrerequisiteSciences = SCI_DOES_NOT_EXIST\n"
                "End\n",
                INI_LOAD_OVERWRITE);
        })) {
        std::fprintf(stderr, "Self-check failed: unknown prereq science did not throw\n");
        std::exit(1);
    }

    // 3. Empty translated label throws INI_INVALID_DATA.
    if (!expect_ini_invalid_data([]() {
            load_ini_string(
                "Science SCI_BAD_TXT\n"
                "  DisplayName = EMPTY_LABEL\n"
                "End\n",
                INI_LOAD_OVERWRITE);
        })) {
        std::fprintf(stderr, "Self-check failed: EMPTY_LABEL did not throw\n");
        std::exit(1);
    }

    // 4. RankInfo monotonic / out-of-range checks.
    if (!expect_ini_invalid_data([]() {
            load_ini_string(
                "Rank 2\n"
                "  RankName = RANK:Two\n"
                "  SkillPointsNeeded = 100\n"
                "  SciencesGranted = None\n"
                "  SciencePurchasePointsGranted = 1\n"
                "End\n",
                INI_LOAD_OVERWRITE);
        })) {
        std::fprintf(stderr, "Self-check failed: non-monotonic Rank 2 did not throw\n");
        std::exit(1);
    }
    load_ini_string(
        "Rank 1\n"
        "  RankName = RANK:One\n"
        "  SkillPointsNeeded = 0\n"
        "  SciencesGranted = SCI_ONE\n"
        "  SciencePurchasePointsGranted = 1\n"
        "End\n",
        INI_LOAD_OVERWRITE);
    if (!expect_ini_invalid_data([]() {
            load_ini_string("Rank 0\n  SkillPointsNeeded = 5\nEnd\n", INI_LOAD_CREATE_OVERRIDES);
        }) ||
        !expect_ini_invalid_data([]() {
            load_ini_string("Rank 2\n  SkillPointsNeeded = 5\nEnd\n", INI_LOAD_CREATE_OVERRIDES);
        })) {
        std::fprintf(stderr, "Self-check failed: out-of-range Rank override did not throw\n");
        std::exit(1);
    }
    if (TheRankInfoStore->getRankInfo(0) != nullptr || TheRankInfoStore->getRankInfo(2) != nullptr) {
        std::fprintf(stderr, "Self-check failed: out-of-range getRankInfo != nullptr\n");
        std::exit(1);
    }

    // 5. Duplicate SpecialPower & unknown RequiredScience checks.
    load_ini_string(
        "SpecialPower SP_BASE\n"
        "  Enum = SPECIAL_DAISY_CUTTER\n"
        "  ReloadTime = 1000\n"
        "  AcademyClassify = ACT_SUPERPOWER\n"
        "End\n",
        INI_LOAD_OVERWRITE);
    if (!expect_ini_invalid_data([]() {
            load_ini_string("SpecialPower SP_BASE\n  ReloadTime = 2000\nEnd\n", INI_LOAD_OVERWRITE);
        })) {
        std::fprintf(stderr, "Self-check failed: duplicate SpecialPower did not throw\n");
        std::exit(1);
    }
    if (!expect_ini_invalid_data([]() {
            load_ini_string("SpecialPower SP_BAD\n  RequiredScience = SCI_UNKNOWN\nEnd\n", INI_LOAD_OVERWRITE);
        })) {
        std::fprintf(stderr, "Self-check failed: invalid RequiredScience did not throw\n");
        std::exit(1);
    }

    // 6. Upgrade XferSave / XferLoad / XferCRC sanity.
    Upgrade upg(nullptr);
    upg.setStatus(UPGRADE_STATUS_COMPLETE);
    XferSave xs;
    xs.open(AsciiString(xfer_tmp));
    xs.xferSnapshot(&upg);
    xs.close();
    Upgrade loaded(nullptr);
    XferLoad xl;
    xl.open(AsciiString(xfer_tmp));
    xl.xferSnapshot(&loaded);
    xl.close();
    if (loaded.getStatus() != UPGRADE_STATUS_COMPLETE) {
        std::fprintf(stderr, "Self-check failed: Upgrade Xfer round-trip mismatch\n");
        std::exit(1);
    }
    XferCRC xcrc;
    xcrc.open(AsciiString("upg_crc"));
    xcrc.xferSnapshot(&upg);
    xcrc.close();
    if (xcrc.getCRC() != 0u) {
        std::fprintf(stderr, "Self-check failed: Upgrade::crc expected 0, got 0x%08x\n", xcrc.getCRC());
        std::exit(1);
    }
}

static void join_csv(const std::vector<std::string>& items, std::string& out) {
    out.clear();
    for (size_t i = 0; i < items.size(); ++i) {
        if (i > 0) out.push_back(',');
        out.append(items[i]);
    }
}

static std::string snapshot_sciences(
    const Player& player,
    const std::vector<std::string>& probe_names,
    std::string& names_csv,
    std::vector<Int>& costs,
    std::vector<UnsignedInt>& grantables,
    std::vector<UnsignedInt>& valids,
    std::vector<UnsignedInt>& has_prereqs,
    std::vector<UnsignedInt>& has_roots,
    std::string& purchasable_csv,
    std::string& potential_csv,
    std::string& leaf_text
) {
    std::vector<AsciiString> names = TheScienceStore->friend_getScienceNames();
    std::vector<std::string> raw_names;
    for (const auto& n : names) raw_names.emplace_back(n.str());
    join_csv(raw_names, names_csv);

    costs.clear();
    grantables.clear();
    valids.clear();
    has_prereqs.clear();
    has_roots.clear();
    for (const std::string& pn : probe_names) {
        ScienceType st = (pn == "INVALID")
            ? SCIENCE_INVALID
            : TheScienceStore->getScienceFromInternalName(AsciiString(pn.c_str()));
        costs.push_back(TheScienceStore->getSciencePurchaseCost(st));
        grantables.push_back(TheScienceStore->isScienceGrantable(st) ? 1u : 0u);
        valids.push_back(TheScienceStore->isValidScience(st) ? 1u : 0u);
        has_prereqs.push_back(TheScienceStore->playerHasPrereqsForScience(&player, st) ? 1u : 0u);
        has_roots.push_back(TheScienceStore->playerHasRootPrereqsForScience(&player, st) ? 1u : 0u);
    }

    ScienceVec purchasable, potentially;
    TheScienceStore->getPurchasableSciences(&player, purchasable, potentially);
    std::vector<std::string> p_names, pot_names;
    for (ScienceType st : purchasable) {
        p_names.emplace_back(TheScienceStore->getInternalNameForScience(st).str());
    }
    for (ScienceType st : potentially) {
        pot_names.emplace_back(TheScienceStore->getInternalNameForScience(st).str());
    }
    join_csv(p_names, purchasable_csv);
    join_csv(pot_names, potential_csv);

    UnicodeString dn, desc;
    ScienceType leaf_st = TheScienceStore->getScienceFromInternalName(AsciiString("SCI_LEAF"));
    if (TheScienceStore->getNameAndDescription(leaf_st, dn, desc)) {
        leaf_text = unicode_to_ascii(dn) + "|" + unicode_to_ascii(desc);
    } else {
        leaf_text = "NONE";
    }
    return names_csv;
}

static void snapshot_ranks(Int& count, std::string& dump) {
    count = TheRankInfoStore->getRankLevelCount();
    dump.clear();
    for (Int lvl = 1; lvl <= count; ++lvl) {
        const RankInfo* ri = TheRankInfoStore->getRankInfo(lvl);
        if (!ri) continue;
        if (lvl > 1) dump.push_back(';');
        char hdr[256];
        std::snprintf(hdr, sizeof(hdr), "%d:%s:%d:%d:",
                      lvl,
                      unicode_to_ascii(ri->m_rankName).c_str(),
                      ri->m_skillPointsNeeded,
                      ri->m_sciencePurchasePointsGranted);
        dump.append(hdr);
        for (size_t j = 0; j < ri->m_sciencesGranted.size(); ++j) {
            if (j > 0) dump.push_back(',');
            dump.append(TheScienceStore->getInternalNameForScience(ri->m_sciencesGranted[j]).str());
        }
    }
}

static void snapshot_special_powers(
    const Object& obj,
    Int& count,
    std::string& dump
) {
    count = TheSpecialPowerStore->getNumSpecialPowers();
    dump.clear();
    for (Int i = 0; i < count; ++i) {
        const SpecialPowerTemplate* sp = TheSpecialPowerStore->getSpecialPowerTemplateByIndex(static_cast<UnsignedInt>(i));
        if (!sp) continue;
        if (i > 0) dump.push_back(';');
        ScienceType req = sp->getRequiredScience();
        std::string req_name = (req == SCIENCE_INVALID)
            ? "INVALID"
            : std::string(TheScienceStore->getInternalNameForScience(req).str());
        Bool can_use = TheSpecialPowerStore->canUseSpecialPower(const_cast<Object*>(&obj), sp);
        char buf[512];
        std::snprintf(buf, sizeof(buf),
                      "%s|%u|%d|%u|%s|%s|%s|%d|%u|%u|0x%08x|0x%08x|%u|%u|%u|%u",
                      sp->getName().str(),
                      sp->getID(),
                      static_cast<Int>(sp->getSpecialPowerType()),
                      sp->getReloadTime(),
                      req_name.c_str(),
                      sp->getInitiateSound()->getEventName().str(),
                      sp->getInitiateAtTargetSound()->getEventName().str(),
                      static_cast<Int>(sp->getAcademyClassificationType()),
                      sp->getDetectionTime(),
                      sp->getViewObjectDuration(),
                      fbits(sp->getViewObjectRange()),
                      fbits(sp->getRadiusCursorRadius()),
                      static_cast<uint32_t>(sp->hasPublicTimer()),
                      static_cast<uint32_t>(sp->isSharedNSync()),
                      static_cast<uint32_t>(sp->isShortcutPower()),
                      static_cast<uint32_t>(can_use));
        dump.append(buf);
    }
}

static void fprintf_i32_arr(FILE* fp, const char* key, const std::vector<Int>& v) {
    std::fprintf(fp, "\"%s\": [", key);
    for (size_t i = 0; i < v.size(); ++i) {
        std::fprintf(fp, "%d%s", v[i], (i + 1 < v.size()) ? ", " : "");
    }
    std::fprintf(fp, "]");
}

static void fprintf_u32_arr(FILE* fp, const char* key, const std::vector<UnsignedInt>& v) {
    std::fprintf(fp, "\"%s\": [", key);
    for (size_t i = 0; i < v.size(); ++i) {
        std::fprintf(fp, "%u%s", v[i], (i + 1 < v.size()) ? ", " : "");
    }
    std::fprintf(fp, "]");
}

int main(int argc, char** argv) {
    if (argc != 2) {
        std::fprintf(stderr, "Usage: gen_wp3_2b <out.json>\n");
        return 1;
    }
    const char* out_path = argv[1];
    std::string tmp_out = std::string(out_path) + ".tmp";

    char xfer_tmp[128];
    std::snprintf(xfer_tmp, sizeof(xfer_tmp), "/tmp/wp3_2b_upg_%ld.bin", static_cast<long>(getpid()));

    run_startup_self_checks(xfer_tmp);

    const char* source_paths[] = {
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Science.h",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/Science.cpp",
        "vendor/GeneralsX/Core/GameEngine/Include/GameLogic/RankInfo.h",
        "vendor/GeneralsX/Core/GameEngine/Source/GameLogic/System/RankInfo.cpp",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Upgrade.h",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/System/Upgrade.cpp",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/SpecialPower.h",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/SpecialPower.cpp"
    };

    FILE* fp = std::fopen(tmp_out.c_str(), "wb");
    if (!fp) { std::perror(tmp_out.c_str()); return 1; }

    std::fprintf(fp,
        "{\n"
        "  \"packet_id\": \"WP-3.2b\",\n"
        "  \"module\": \"Science+RankInfo+Upgrade+SpecialPower\",\n"
        "  \"oracle\": \"vendor/GeneralsX unmodified Science.cpp + RankInfo.cpp + Upgrade.cpp + SpecialPower.cpp + INI.cpp + Xfer*.cpp\",\n"
        "  \"vector_count\": 1000,\n"
        "  \"source_sha256\": {\n");
    for (size_t i = 0; i < ARRAY_SIZE(source_paths); ++i) {
        std::string digest = sha256_file(source_paths[i]);
        std::fprintf(fp, "    \"%s\": \"%s\"%s\n",
                     source_paths[i], digest.c_str(),
                     (i + 1 < ARRAY_SIZE(source_paths)) ? "," : "");
    }
    std::fprintf(fp, "  },\n  \"vectors\": [\n");

    const std::vector<std::string> sci_probes = {
        "INVALID", "SCI_ROOT_A", "SCI_ROOT_B", "SCI_MID", "SCI_LEAF", "SCI_OV_NEW"
    };
    const char* act_names[] = { "ACT_NONE", "ACT_UPGRADE_RADAR", "ACT_SUPERPOWER" };
    const char* snd_names[] = { "SoundAlpha", "SoundBeta", "SoundGamma", "NoSound" };
    const char* sp_enum_names[] = {
        "SPECIAL_INVALID", "SPECIAL_DAISY_CUTTER", "SPECIAL_PARADROP_AMERICA",
        "SPECIAL_CARPET_BOMB", "SPECIAL_AMBUSH", "SPECIAL_NEUTRON_MISSILE",
        "SPECIAL_SCUD_STORM", "SPECIAL_BATTLESHIP_BOMBARDMENT"
    };

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed * 0x9e3779b9u + 0x7f4a7c15u;
        reset_all_stores();

        Bool do_ov = (seed == 0 || seed == 2 || seed == 3 || (seed % 3u == 0)) ? TRUE : FALSE;
        Bool do_reset = (seed == 2 || seed == 3 || (seed % 5u == 0)) ? TRUE : FALSE;

        // 1. Construct Science INIs.
        Int cost_a = (seed == 0) ? 1 : static_cast<Int>(splitmix32(rng) % 5u);
        Int cost_b = (seed == 0) ? 2 : static_cast<Int>(splitmix32(rng) % 5u);
        Int cost_mid = (seed == 0) ? 1 : static_cast<Int>(splitmix32(rng) % 4u);
        Int cost_leaf = (seed == 0) ? 3 : static_cast<Int>(1u + (splitmix32(rng) % 4u));
        const char* grant_b = ((seed % 2u) == 0) ? "Yes" : "No";
        std::string mid_prereq = (seed % 4u == 1) ? "None" : "SCI_ROOT_A";
        std::string leaf_prereq = (seed % 3u == 2) ? "SCI_MID SCI_ROOT_B" : "SCI_MID";

        char sci_buf[2048];
        std::snprintf(sci_buf, sizeof(sci_buf),
            "Science SCI_ROOT_A\n"
            "  PrerequisiteSciences = None\n"
            "  SciencePurchasePointCost = %d\n"
            "  IsGrantable = Yes\n"
            "  DisplayName = SCI:RootA\n"
            "  Description = DESC:RootA\n"
            "End\n"
            "Science SCI_ROOT_B\n"
            "  PrerequisiteSciences = None\n"
            "  SciencePurchasePointCost = %d\n"
            "  IsGrantable = %s\n"
            "  DisplayName = SCI:RootB\n"
            "  Description = DESC:RootB\n"
            "End\n"
            "Science SCI_MID\n"
            "  PrerequisiteSciences = %s\n"
            "  SciencePurchasePointCost = %d\n"
            "  IsGrantable = Yes\n"
            "  DisplayName = SCI:Mid\n"
            "  Description = DESC:Mid\n"
            "End\n"
            "Science SCI_LEAF\n"
            "  PrerequisiteSciences = %s\n"
            "  SciencePurchasePointCost = %d\n"
            "  IsGrantable = Yes\n"
            "  DisplayName = SCI:Leaf\n"
            "  Description = DESC:Leaf\n"
            "End\n",
            cost_a, cost_b, grant_b, mid_prereq.c_str(), cost_mid, leaf_prereq.c_str(), cost_leaf);
        std::string sci_base_ini = sci_buf;

        std::string sci_ov_ini;
        if (do_ov) {
            char ov_buf[1024];
            std::snprintf(ov_buf, sizeof(ov_buf),
                "Science SCI_MID\n"
                "  PrerequisiteSciences = SCI_ROOT_B\n"
                "  SciencePurchasePointCost = %d\n"
                "End\n"
                "Science SCI_LEAF\n"
                "  DisplayName = SCI:LeafOv\n"
                "  Description = DESC:LeafOv\n"
                "  SciencePurchasePointCost = %d\n"
                "End\n"
                "Science SCI_OV_NEW\n"
                "  PrerequisiteSciences = SCI_ROOT_A\n"
                "  SciencePurchasePointCost = 2\n"
                "  IsGrantable = Yes\n"
                "  DisplayName = SCI:OvNew\n"
                "  Description = DESC:OvNew\n"
                "End\n",
                (seed % 2u) + 1, cost_leaf + 2);
            sci_ov_ini = ov_buf;
        }

        load_ini_string(sci_base_ini, INI_LOAD_OVERWRITE);
        if (do_ov) load_ini_string(sci_ov_ini, INI_LOAD_CREATE_OVERRIDES);

        // Player state (owned sciences + money).
        Player player;
        UnsignedInt money = (seed == 0) ? 1000u :
                            (seed == 1) ? 999u :
                            (splitmix32(rng) % 5000u);
        player.setMoneyAmount(money);
        std::vector<std::string> owned_vec;
        const char* cand_owned[] = { "SCI_ROOT_A", "SCI_ROOT_B", "SCI_MID", "SCI_LEAF" };
        uint32_t mask_bits = (seed == 0) ? 0x1u : (seed == 1 ? 0x0u : (seed == 2 ? 0x3u : (splitmix32(rng) & 0xFu)));
        for (int b = 0; b < 4; ++b) {
            if (mask_bits & (1u << b)) {
                owned_vec.emplace_back(cand_owned[b]);
                ScienceType st = TheScienceStore->getScienceFromInternalName(AsciiString(cand_owned[b]));
                player.grantScience(st);
            }
        }
        std::string owned_csv;
        join_csv(owned_vec, owned_csv);

        std::string sci_names_ov, sci_purch_ov, sci_pot_ov, sci_text_ov;
        std::vector<Int> sci_costs_ov;
        std::vector<UnsignedInt> sci_grant_ov, sci_valid_ov, sci_has_pre_ov, sci_has_roots_ov;
        snapshot_sciences(player, sci_probes, sci_names_ov, sci_costs_ov, sci_grant_ov, sci_valid_ov,
                          sci_has_pre_ov, sci_has_roots_ov, sci_purch_ov, sci_pot_ov, sci_text_ov);

        // 2. Construct RankInfo INIs (must happen BEFORE ScienceStore::reset() clears override-only sciences).
        Int num_ranks = (seed == 0) ? 3 : static_cast<Int>(1u + (splitmix32(rng) % 4u));
        std::string rank_base_ini;
        for (Int r = 1; r <= num_ranks; ++r) {
            Int pts = (r - 1) * 500 + static_cast<Int>(splitmix32(rng) % 100u);
            UnsignedInt spp = 1u + (splitmix32(rng) % 3u);
            const char* sci_gr = (r == 1) ? "SCI_ROOT_A" : (r == 2 ? "SCI_ROOT_A SCI_ROOT_B" : "None");
            char rb[512];
            std::snprintf(rb, sizeof(rb),
                "Rank %d\n"
                "  RankName = RANK:Level%d\n"
                "  SkillPointsNeeded = %d\n"
                "  SciencesGranted = %s\n"
                "  SciencePurchasePointsGranted = %u\n"
                "End\n",
                r, r, pts, sci_gr, spp);
            rank_base_ini.append(rb);
        }
        std::string rank_ov_ini;
        if (do_ov && num_ranks >= 1) {
            char ro[512];
            std::snprintf(ro, sizeof(ro),
                "Rank 1\n"
                "  RankName = RANK:OvOne\n"
                "  SkillPointsNeeded = 42\n"
                "  SciencesGranted = None\n"
                "End\n");
            rank_ov_ini = ro;
        }
        load_ini_string(rank_base_ini, INI_LOAD_OVERWRITE);
        if (do_ov) load_ini_string(rank_ov_ini, INI_LOAD_CREATE_OVERRIDES);

        Int rank_cnt_ov = 0;
        std::string rank_dump_ov;
        snapshot_ranks(rank_cnt_ov, rank_dump_ov);

        // 3. Construct Upgrade INI + exercise UpgradeCenter + Upgrade Xfer.
        TheUpgradeCenter->init(); // creates VETERAN, ELITE, HEROIC at mask bits 0,1,2
        Real def_bt = (seed == 0) ? 10.5f : static_cast<Real>(splitmix32(rng) % 200u) * 0.25f;
        Int def_cost = (seed == 0) ? 1000 : static_cast<Int>(splitmix32(rng) % 3000u);
        const char* upg_type_str = ((seed % 2u) == 0) ? "PLAYER" : "OBJECT";
        const char* act_def = act_names[splitmix32(rng) % 3u];
        const char* def_rsnd = snd_names[(seed == 0) ? 0u : (splitmix32(rng) % 4u)];
        const char* alpha_rsnd = (seed == 0) ? "NoSound" : snd_names[splitmix32(rng) % 4u];
        const char* beta_usnd = snd_names[splitmix32(rng) % 4u];
        Real bt_alpha = (seed == 0) ? 7.35f : static_cast<Real>(splitmix32(rng) % 300u) * 0.1f;
        Int cost_alpha = (seed == 1) ? 1000 : static_cast<Int>(splitmix32(rng) % 2500u);
        Int cost_beta = (seed == 1) ? 999 : static_cast<Int>(splitmix32(rng) % 2500u);
        const char* btn_img = ((seed % 2u) == 0) ? "BTN_FOUND" : "BTN_MISSING";

        char upg_buf[2048];
        std::snprintf(upg_buf, sizeof(upg_buf),
            "Upgrade DefaultUpgrade\n"
            "  DisplayName = UPG:Default\n"
            "  Type = PLAYER\n"
            "  BuildTime = %.3f\n"
            "  BuildCost = %d\n"
            "  ButtonImage = %s\n"
            "  ResearchSound = %s\n"
            "  UnitSpecificSound = SoundUnitDef\n"
            "  AcademyClassify = %s\n"
            "End\n"
            "Upgrade Upgrade_Alpha\n"
            "  DisplayName = UPG:Alpha\n"
            "  Type = %s\n"
            "  BuildTime = %.3f\n"
            "  BuildCost = %d\n"
            "  ResearchSound = %s\n"
            "End\n"
            "Upgrade Upgrade_Beta\n"
            "  DisplayName = UPG:Beta\n"
            "  BuildCost = %d\n"
            "  UnitSpecificSound = %s\n"
            "  AcademyClassify = %s\n"
            "End\n"
            "Upgrade Upgrade_Alpha\n"
            "  BuildCost = %d\n"
            "End\n",
            static_cast<double>(def_bt), def_cost, btn_img, def_rsnd, act_def,
            upg_type_str, static_cast<double>(bt_alpha), cost_alpha, alpha_rsnd,
            cost_beta, beta_usnd, act_names[splitmix32(rng) % 3u],
            cost_alpha + 25);
        std::string upg_base_ini = upg_buf;
        load_ini_string(upg_base_ini, INI_LOAD_OVERWRITE);

        TheUpgradeCenter->reset();
        TheUpgradeCenter->reset(); // verify second reset is idempotent on buttonImagesCached

        std::vector<AsciiString> lifo_names = TheUpgradeCenter->getUpgradeNames();
        std::vector<std::string> lifo_vec;
        std::string upg_summary;
        Bool disp_reason = ((seed % 2u) == 1) ? TRUE : FALSE;
        for (size_t i = 0; i < lifo_names.size(); ++i) {
            const UpgradeTemplate* ut = TheUpgradeCenter->findUpgrade(lifo_names[i]);
            lifo_vec.emplace_back(lifo_names[i].str());
            if (!ut) continue;
            if (i > 0) upg_summary.push_back(';');
            Int mask_bit = -1;
            for (Int b = 0; b < UPGRADE_MAX_COUNT; ++b) {
                if (ut->getUpgradeMask().test(b)) { mask_bit = b; break; }
            }
            Bool can_aff = TheUpgradeCenter->canAffordUpgrade(&player, ut, disp_reason);
            char urow[512];
            std::snprintf(urow, sizeof(urow),
                          "%s|%d|%d|%s|%d|%d|%d|%s|%s|%d|%u",
                          ut->getUpgradeName().str(),
                          static_cast<Int>(ut->getUpgradeNameKey()),
                          static_cast<Int>(ut->getUpgradeType()),
                          ut->getDisplayNameLabel().str(),
                          ut->calcTimeToBuild(&player),
                          ut->calcCostToBuild(&player),
                          mask_bit,
                          ut->getResearchCompleteSound()->getEventName().str(),
                          ut->getUnitSpecificSound()->getEventName().str(),
                          static_cast<Int>(ut->getAcademyClassificationType()),
                          static_cast<uint32_t>(can_aff));
            upg_summary.append(urow);
        }
        std::string upg_names_lifo;
        join_csv(lifo_vec, upg_names_lifo);
        Int upg_ui_msgs = g_in_game_ui.messageCount;

        // Exercise Upgrade XferSave + XferLoad + XferCRC.
        UpgradeStatusType upg_status = static_cast<UpgradeStatusType>(seed % 3u);
        Upgrade upg_inst(TheUpgradeCenter->findVeterancyUpgrade(LEVEL_ELITE));
        upg_inst.setStatus(upg_status);
        {
            XferSave xs;
            xs.open(AsciiString(xfer_tmp));
            xs.xferSnapshot(&upg_inst);
            xs.close();
        }
        std::vector<uint8_t> upg_xfer_bytes = read_file_bytes(xfer_tmp);
        XferCRC upg_xcrc;
        upg_xcrc.open(AsciiString("upg_crc"));
        upg_xcrc.xferSnapshot(&upg_inst);
        upg_xcrc.close();
        uint32_t upg_crc = upg_xcrc.getCRC();

        // 4. Construct SpecialPower INI (DefaultSpecialPower explicitly sets AcademyClassify!).
        UnsignedInt sp_rel_0 = (seed == 0) ? 1000u : (splitmix32(rng) % 60000u);
        UnsignedInt sp_rel_1 = (seed == 0) ? 33u : (1u + (splitmix32(rng) % 120000u));
        UnsignedInt sp_det_0 = (seed == 0) ? 500u : (splitmix32(rng) % 20000u);
        UnsignedInt sp_vdur_1 = (seed == 0) ? 1234u : (splitmix32(rng) % 15000u);
        Real sp_vrng = static_cast<Real>(splitmix32(rng) % 500u) * 0.5f;
        Real sp_rad = static_cast<Real>(splitmix32(rng) % 300u) * 0.25f;
        const char* enum0 = sp_enum_names[splitmix32(rng) % ARRAY_SIZE(sp_enum_names)];
        const char* enum1 = sp_enum_names[splitmix32(rng) % ARRAY_SIZE(sp_enum_names)];
        const char* act0 = act_names[splitmix32(rng) % 3u];
        const char* act1 = act_names[splitmix32(rng) % 3u];
        const char* pub0 = ((seed % 2u) == 0) ? "Yes" : "No";
        const char* shd1 = ((seed % 3u) == 0) ? "Yes" : "No";
        const char* sc1 = ((seed % 2u) == 1) ? "Yes" : "No";
        const char* req0 = (seed % 2u == 0) ? "SCI_ROOT_A" : "SCI_LEAF";

        char sp_buf[2048];
        std::snprintf(sp_buf, sizeof(sp_buf),
            "SpecialPower DefaultSpecialPower\n"
            "  Enum = SPECIAL_INVALID\n"
            "  ReloadTime = 10000\n"
            "  InitiateSound = NoSound\n"
            "  InitiateAtLocationSound = SoundLocDef\n"
            "  PublicTimer = No\n"
            "  DetectionTime = 5000\n"
            "  SharedSyncedTimer = No\n"
            "  ViewObjectDuration = 100\n"
            "  ViewObjectRange = 50.0\n"
            "  RadiusCursorRadius = 10.0\n"
            "  ShortcutPower = No\n"
            "  AcademyClassify = ACT_NONE\n"
            "End\n"
            "SpecialPower SP_Alpha\n"
            "  Enum = %s\n"
            "  ReloadTime = %u\n"
            "  RequiredScience = %s\n"
            "  InitiateSound = SoundAlpha\n"
            "  InitiateAtLocationSound = NoSound\n"
            "  PublicTimer = %s\n"
            "  DetectionTime = %u\n"
            "  AcademyClassify = %s\n"
            "End\n"
            "SpecialPower SP_Beta\n"
            "  Enum = %s\n"
            "  ReloadTime = %u\n"
            "  SharedSyncedTimer = %s\n"
            "  ViewObjectDuration = %u\n"
            "  ViewObjectRange = %.2f\n"
            "  RadiusCursorRadius = %.2f\n"
            "  ShortcutPower = %s\n"
            "  AcademyClassify = %s\n"
            "End\n",
            enum0, sp_rel_0, req0, pub0, sp_det_0, act0,
            enum1, sp_rel_1, shd1, sp_vdur_1, static_cast<double>(sp_vrng),
            static_cast<double>(sp_rad), sc1, act1);
        std::string sp_base_ini = sp_buf;

        std::string sp_ov_ini;
        if (do_ov) {
            char spo[1024];
            std::snprintf(spo, sizeof(spo),
                "SpecialPower SP_Alpha\n"
                "  ReloadTime = %u\n"
                "  AcademyClassify = ACT_UPGRADE_RADAR\n"
                "  ShortcutPower = Yes\n"
                "End\n"
                "SpecialPower SP_OverrideOnly\n"
                "  Enum = SPECIAL_AMBUSH\n"
                "  ReloadTime = 3000\n"
                "  RequiredScience = SCI_OV_NEW\n"
                "  AcademyClassify = ACT_SUPERPOWER\n"
                "End\n",
                sp_rel_0 + 2500u);
            sp_ov_ini = spo;
        }
        load_ini_string(sp_base_ini, INI_LOAD_OVERWRITE);
        if (do_ov) load_ini_string(sp_ov_ini, INI_LOAD_CREATE_OVERRIDES);

        // Build Object supporting a deterministic subset of SpecialPower IDs (1..=4).
        Object obj;
        obj.m_owner = &player;
        std::vector<std::string> sp_sup_vec;
        uint32_t sup_mask = (seed == 0) ? 0x2u : (splitmix32(rng) & 0xFu);
        for (UnsignedInt id = 1; id <= 4; ++id) {
            if (sup_mask & (1u << (id - 1))) {
                obj.m_supportedSpecialPowerIds.push_back(id);
                sp_sup_vec.push_back(std::to_string(id));
            }
        }
        std::string sp_sup_csv;
        join_csv(sp_sup_vec, sp_sup_csv);

        Int sp_cnt_ov = 0;
        std::string sp_summary_ov;
        snapshot_special_powers(obj, sp_cnt_ov, sp_summary_ov);

        // Optional reset() on Science, RankInfo, SpecialPower!
        if (do_reset) {
            TheScienceStore->reset();
            TheRankInfoStore->reset();
            TheSpecialPowerStore->reset();
        }

        std::string sci_names_rst, sci_purch_rst, sci_pot_rst, sci_text_rst;
        std::vector<Int> sci_costs_rst;
        std::vector<UnsignedInt> sci_grant_rst, sci_valid_rst, sci_has_pre_rst, sci_has_roots_rst;
        snapshot_sciences(player, sci_probes, sci_names_rst, sci_costs_rst, sci_grant_rst, sci_valid_rst,
                          sci_has_pre_rst, sci_has_roots_rst, sci_purch_rst, sci_pot_rst, sci_text_rst);

        Int rank_cnt_rst = 0;
        std::string rank_dump_rst;
        snapshot_ranks(rank_cnt_rst, rank_dump_rst);

        Int sp_cnt_rst = 0;
        std::string sp_summary_rst;
        snapshot_special_powers(obj, sp_cnt_rst, sp_summary_rst);

        std::fprintf(fp,
            "    {\"seed\": %u, \"player_money\": %u, \"owned_sciences_csv\": \"%s\", "
            "\"sp_supported_csv\": \"%s\", \"do_ov\": %u, \"do_reset\": %u, "
            "\"sci_base_hex\": \"%s\", \"sci_ov_hex\": \"%s\", "
            "\"sci_names_ov\": \"%s\", \"sci_names_rst\": \"%s\", ",
            seed, money, owned_csv.c_str(), sp_sup_csv.c_str(),
            static_cast<uint32_t>(do_ov), static_cast<uint32_t>(do_reset),
            str_hex(sci_base_ini).c_str(), str_hex(sci_ov_ini).c_str(),
            sci_names_ov.c_str(), sci_names_rst.c_str());
        fprintf_i32_arr(fp, "sci_costs_ov", sci_costs_ov); std::fprintf(fp, ", ");
        fprintf_i32_arr(fp, "sci_costs_rst", sci_costs_rst); std::fprintf(fp, ", ");
        fprintf_u32_arr(fp, "sci_grant_ov", sci_grant_ov); std::fprintf(fp, ", ");
        fprintf_u32_arr(fp, "sci_grant_rst", sci_grant_rst); std::fprintf(fp, ", ");
        fprintf_u32_arr(fp, "sci_valid_ov", sci_valid_ov); std::fprintf(fp, ", ");
        fprintf_u32_arr(fp, "sci_valid_rst", sci_valid_rst); std::fprintf(fp, ", ");
        fprintf_u32_arr(fp, "sci_has_prereqs_ov", sci_has_pre_ov); std::fprintf(fp, ", ");
        fprintf_u32_arr(fp, "sci_has_roots_ov", sci_has_roots_ov); std::fprintf(fp, ", ");
        std::fprintf(fp,
            "\"sci_purchasable_ov\": \"%s\", \"sci_potential_ov\": \"%s\", "
            "\"sci_purchasable_rst\": \"%s\", \"sci_potential_rst\": \"%s\", "
            "\"sci_text_leaf_ov\": \"%s\", \"sci_text_leaf_rst\": \"%s\", "
            "\"rank_base_hex\": \"%s\", \"rank_ov_hex\": \"%s\", "
            "\"rank_cnt_ov\": %d, \"rank_cnt_rst\": %d, "
            "\"rank_dump_ov\": \"%s\", \"rank_dump_rst\": \"%s\", "
            "\"upg_base_hex\": \"%s\", \"upg_status\": %d, \"upg_disp_reason\": %u, "
            "\"upg_names_lifo\": \"%s\", \"upg_summary\": \"%s\", \"upg_ui_msgs\": %d, "
            "\"upg_xfer_hex\": \"%s\", \"upg_crc_hex\": \"0x%08x\", "
            "\"sp_base_hex\": \"%s\", \"sp_ov_hex\": \"%s\", "
            "\"sp_cnt_ov\": %d, \"sp_cnt_rst\": %d, "
            "\"sp_summary_ov\": \"%s\", \"sp_summary_rst\": \"%s\"}%s\n",
            sci_purch_ov.c_str(), sci_pot_ov.c_str(),
            sci_purch_rst.c_str(), sci_pot_rst.c_str(),
            sci_text_ov.c_str(), sci_text_rst.c_str(),
            str_hex(rank_base_ini).c_str(), str_hex(rank_ov_ini).c_str(),
            rank_cnt_ov, rank_cnt_rst,
            rank_dump_ov.c_str(), rank_dump_rst.c_str(),
            str_hex(upg_base_ini).c_str(), static_cast<Int>(upg_status), static_cast<uint32_t>(disp_reason),
            upg_names_lifo.c_str(), upg_summary.c_str(), upg_ui_msgs,
            to_hex(upg_xfer_bytes.data(), upg_xfer_bytes.size()).c_str(), upg_crc,
            str_hex(sp_base_ini).c_str(), str_hex(sp_ov_ini).c_str(),
            sp_cnt_ov, sp_cnt_rst,
            sp_summary_ov.c_str(), sp_summary_rst.c_str(),
            (seed + 1u < 1000u) ? "," : "");
    }

    std::fprintf(fp, "  ]\n}\n");
    if (std::fflush(fp) != 0 || std::fclose(fp) != 0) {
        std::perror("flush/close");
        std::remove(xfer_tmp);
        std::remove(tmp_out.c_str());
        return 1;
    }
    std::remove(xfer_tmp);
    if (std::rename(tmp_out.c_str(), out_path) != 0) {
        std::perror("rename");
        return 1;
    }

    delete TheSpecialPowerStore; TheSpecialPowerStore = nullptr;
    delete TheUpgradeCenter; TheUpgradeCenter = nullptr;
    delete TheRankInfoStore; TheRankInfoStore = nullptr;
    delete TheScienceStore; TheScienceStore = nullptr;
    delete TheNameKeyGenerator; TheNameKeyGenerator = nullptr;
    return 0;
}
