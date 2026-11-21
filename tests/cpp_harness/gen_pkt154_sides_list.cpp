// PKT-154 (SidesList, SidesInfo, TeamsInfo, TeamsInfoRec, BuildListInfo)
// C++ Golden Vector Generator.
// Compiles unmodified vendor/GeneralsX SidesList.cpp + DataChunk.cpp + Dict.cpp + Xfer*.cpp.
//
// Canonical build & run command (macOS arm64, Release-aligned macros):
//   clang++ -std=c++20 -O1 -ffp-contract=off -URTS_DEBUG -U_ALLOW_DEBUG_CHEATS_IN_RELEASE -D_GNU_SOURCE \
//     -Wno-macro-redefined -Wno-switch -Wno-invalid-offsetof \
//     -Itests/cpp_harness/stubs_pkt154 \
//     -Ivendor/GeneralsX/GeneralsMD/Code/GameEngine/Include \
//     -Ivendor/GeneralsX/Core/GameEngine/Include \
//     -Ivendor/GeneralsX/Core/Libraries/Source/Compression \
//     -Ivendor/GeneralsX/build/macos-vulkan/_deps/gamemath-src/include \
//     tests/cpp_harness/gen_pkt154_sides_list.cpp \
//     vendor/GeneralsX/build/macos-vulkan/_deps/gamemath-build/libgm.a \
//     -o /tmp/gen_pkt154 && /tmp/gen_pkt154 docs/golden_vectors/PKT-154.json
//
// Determinism & UB verification builds (byte-identical JSON across -O0/-O1/-O2/ASan):
//   -fsanitize=address,undefined -fno-sanitize=alignment,enum
//   (`-fno-sanitize=alignment,enum` exempts vendor `Dict.h` `DictPairData` 6-byte packed
//   header alignment and bit-packed `DictPairKeyType` enum while keeping ASan/LSan and all
//   other UBSan checks active; zero findings observed across all 1,000 vectors.)

#define DEFINE_DEATH_NAMES
#define INSTANTIATE_WELL_KNOWN_KEYS
#include "PreRTS.h"
#include "Compression.h"
#include "Common/crc.h"
#include "Common/Dict.h"
#include "Common/DataChunk.h"
#include "Common/INI.h"
#include "Common/Xfer.h"
#include "Common/XferCRC.h"
#include "Common/XferSave.h"
#include "Common/XferLoad.h"
#include "Common/FileSystem.h"
#include "Common/Money.h"
#include "Common/Handicap.h"
#include "Common/PlayerTemplate.h"
#include "Common/Science.h"
#include "Common/Upgrade.h"
#include "Common/SpecialPower.h"
#include "Common/WellKnownKeys.h"
#include "GameLogic/Damage.h"
#include "GameLogic/Object.h"
#include "GameLogic/AI.h"
#include "GameLogic/Scripts.h"
#include "GameLogic/SidesList.h"
#include "GameClient/GameText.h"
#include "GameClient/Image.h"
#include "GameClient/InGameUI.h"

#include "../../vendor/GeneralsX/Core/GameEngine/Source/GameLogic/Map/SidesList.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/System/DataChunk.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/Dict.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/NameKeyGenerator.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/Money.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/Science.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/System/Upgrade.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/SpecialPower.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/INI/INIUpgrade.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/INI/INISpecialPower.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/GameLogic/System/Damage.cpp"
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
const char* const TheAcademyClassificationTypeNames[] = {
    "ACT_NONE", "ACT_UPGRADE_RADAR", "ACT_SUPERPOWER", nullptr
};

SpecialPowerModuleInterface* Object::getSpecialPowerModule(const SpecialPowerTemplate*) const {
    return nullptr;
}

static DynamicMemoryAllocator g_dma;
DynamicMemoryAllocator* TheDynamicMemoryAllocator = &g_dma;
static GlobalData g_global_data;
GlobalData* TheGlobalData = &g_global_data;
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

const StaticNameKey TheKey_playerName("playerName");
const StaticNameKey TheKey_playerIsHuman("playerIsHuman");
const StaticNameKey TheKey_playerIsSkirmish("playerIsSkirmish");
const StaticNameKey TheKey_playerDisplayName("playerDisplayName");
const StaticNameKey TheKey_playerFaction("playerFaction");
const StaticNameKey TheKey_playerAllies("playerAllies");
const StaticNameKey TheKey_playerEnemies("playerEnemies");
const StaticNameKey TheKey_teamName("teamName");
const StaticNameKey TheKey_teamOwner("teamOwner");
const StaticNameKey TheKey_teamIsSingleton("teamIsSingleton");

Bool CompressionManager::isDataCompressed(const void*, Int) { return FALSE; }
Int CompressionManager::getUncompressedSize(const void*, Int len) { return len; }
Int CompressionManager::decompressData(void*, Int, void*, Int) { return 0; }

static BuildListInfo* g_lastParsedBuildList = nullptr;
void INI::parseAIDataDefinition(INI* ini) {
    deleteInstance(g_lastParsedBuildList);
    g_lastParsedBuildList = nullptr;
    AISideBuildList ai_list(AsciiString("America"));
    BuildListInfo::parseStructure(ini, &ai_list, nullptr, nullptr);
    if (!ai_list.items.empty()) {
        g_lastParsedBuildList = ai_list.items.front();
    }
}

#define STUB_PARSE(fn) void INI::fn(INI*) {}
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

class MemOutputStream : public OutputStream {
public:
    std::vector<uint8_t> bytes;
    Int write(const void* pData, Int numBytes) override {
        if (numBytes <= 0 || !pData) return 0;
        const uint8_t* p = static_cast<const uint8_t*>(pData);
        bytes.insert(bytes.end(), p, p + numBytes);
        return numBytes;
    }
};

class SliceChunkInputStream : public ChunkInputStream {
    std::vector<uint8_t> m_buf;
    UnsignedInt m_pos = 0;
public:
    explicit SliceChunkInputStream(std::vector<uint8_t> buf) : m_buf(std::move(buf)) {}
    Int read(void* pData, Int numBytes) override {
        if (!pData || numBytes <= 0 || m_pos >= m_buf.size()) return 0;
        size_t avail = m_buf.size() - m_pos;
        size_t take = std::min(avail, static_cast<size_t>(numBytes));
        std::memcpy(pData, m_buf.data() + m_pos, take);
        m_pos += static_cast<UnsignedInt>(take);
        return static_cast<Int>(take);
    }
    UnsignedInt tell() override { return m_pos; }
    Bool absoluteSeek(UnsignedInt pos) override {
        if (pos > m_buf.size()) return FALSE;
        m_pos = pos;
        return TRUE;
    }
    Bool eof() override { return m_pos >= m_buf.size() ? TRUE : FALSE; }
};

static uint32_t f32_bits(Real v) {
    uint32_t u = 0;
    std::memcpy(&u, &v, 4);
    return u;
}

static std::string hex_encode(const std::vector<uint8_t>& bytes) {
    static const char* kHex = "0123456789abcdef";
    std::string out;
    out.reserve(bytes.size() * 2);
    for (uint8_t b : bytes) {
        out.push_back(kHex[b >> 4]);
        out.push_back(kHex[b & 0x0f]);
    }
    return out;
}

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

static std::vector<uint8_t> read_file_bytes(const char* path) {
    FILE* f = std::fopen(path, "rb");
    if (!f) { std::perror(path); std::exit(1); }
    std::fseek(f, 0, SEEK_END);
    long sz = std::ftell(f);
    std::fseek(f, 0, SEEK_SET);
    std::vector<uint8_t> out(sz > 0 ? static_cast<size_t>(sz) : 0);
    if (sz > 0 && std::fread(out.data(), 1, static_cast<size_t>(sz), f) != static_cast<size_t>(sz)) {
        std::perror("fread");
        std::exit(1);
    }
    std::fclose(f);
    return out;
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

static Dict make_side_dict(const char* name, const char* faction, const char* allies, const char* enemies, Bool human) {
    Dict d;
    d.setAsciiString(TheKey_playerName, AsciiString(name));
    d.setAsciiString(TheKey_playerFaction, AsciiString(faction));
    d.setAsciiString(TheKey_playerAllies, AsciiString(allies));
    d.setAsciiString(TheKey_playerEnemies, AsciiString(enemies));
    d.setBool(TheKey_playerIsHuman, human);
    return d;
}

static Dict make_team_dict(const char* name, const char* owner, Bool singleton) {
    Dict d;
    d.setAsciiString(TheKey_teamName, AsciiString(name));
    d.setAsciiString(TheKey_teamOwner, AsciiString(owner));
    d.setBool(TheKey_teamIsSingleton, singleton);
    return d;
}

static void run_self_checks(const char* tmp_xfer) {
    SidesList sl;
    TheSidesList = &sl;
    sl.clear();

    // 1. removeSide <= 1 guard and validateSides neutral addition.
    sl.addPlayerByTemplate(AsciiString("FactionAmerica"));
    if (sl.getNumSides() != 1) std::exit(1);
    sl.removeSide(0);
    if (sl.getNumSides() != 1) std::exit(1);

    if (!sl.validateSides()) std::exit(1);
    if (sl.getNumSides() != 2) std::exit(1);

    // 2. BuildListInfo UNLIMITED_REBUILDS & decrement/increment boundaries.
    BuildListInfo bli;
    bli.setNumRebuilds(BuildListInfo::UNLIMITED_REBUILDS);
    bli.decrementNumRebuilds();
    bli.incrementNumRebuilds();
    if (bli.getNumRebuilds() != static_cast<Int>(BuildListInfo::UNLIMITED_REBUILDS) || !bli.isBuildable()) {
        std::exit(1);
    }
    bli.setNumRebuilds(1);
    bli.decrementNumRebuilds();
    if (bli.getNumRebuilds() != 0 || bli.isBuildable()) {
        std::exit(1);
    }

    // 3. BuildListInfo::parseStructure via public INI load hook.
    g_fs.nextFileContent =
        "AIData AmericaCommandCenter\n"
        "  Name = CommandCenter0\n"
        "  Location = X:120.5 Y:240.25\n"
        "  Rebuilds = 3\n"
        "  Angle = 90.0\n"
        "  InitiallyBuilt = Yes\n"
        "  RallyPointOffset = X:-10.0 Y:5.5\n"
        "  AutomaticallyBuild = No\n"
        "End\n";
    g_fs.nextFileExists = true;
    {
        INI ini;
        ini.load(AsciiString("build.ini"), INI_LOAD_OVERWRITE, nullptr);
    }
    if (!g_lastParsedBuildList ||
        g_lastParsedBuildList->getBuildingName() != "CommandCenter0" ||
        g_lastParsedBuildList->getTemplateName() != "AmericaCommandCenter" ||
        g_lastParsedBuildList->getNumRebuilds() != 3 ||
        !g_lastParsedBuildList->isInitiallyBuilt() ||
        g_lastParsedBuildList->isAutomaticBuild()) {
        std::fprintf(stderr, "Self-check failed: BuildListInfo::parseStructure\n");
        std::exit(1);
    }
    deleteInstance(g_lastParsedBuildList);
    g_lastParsedBuildList = nullptr;

    // 4. SidesList::xfer mismatch exception check.
    XferSave xs;
    xs.open(AsciiString(tmp_xfer));
    xs.xferSnapshot(&sl);
    xs.close();
    std::vector<uint8_t> saved = read_file_bytes(tmp_xfer);
    if (saved.empty()) std::exit(1);

    SidesList mismatch;
    XferLoad xl;
    xl.open(AsciiString(tmp_xfer));
    bool threw = false;
    try {
        xl.xferSnapshot(&mismatch);
    } catch (SaveCode sc) {
        threw = (sc == SC_INVALID_DATA);
    }
    xl.close();
    if (!threw) {
        std::fprintf(stderr, "Self-check failed: SidesList::xfer sideCount mismatch did not throw SC_INVALID_DATA\n");
        std::exit(1);
    }
    std::remove(tmp_xfer);
    TheSidesList = nullptr;
}

int main(int argc, char** argv) {
    if (argc != 2) {
        std::fprintf(stderr, "Usage: %s <output.json>\n", argv[0]);
        return 1;
    }

    TheNameKeyGenerator = new NameKeyGenerator();
    TheNameKeyGenerator->init();
    TheScienceStore = new ScienceStore();
    TheScienceStore->init();
    TheUpgradeCenter = new UpgradeCenter();
    TheSpecialPowerStore = new SpecialPowerStore();
    TheSpecialPowerStore->init();

    // Fixed WellKnownKeys intern order (documented in PKT-154 Port Contract).
    (void)TheKey_playerName.key();
    (void)TheKey_playerIsHuman.key();
    (void)TheKey_playerIsSkirmish.key();
    (void)TheKey_playerDisplayName.key();
    (void)TheKey_playerFaction.key();
    (void)TheKey_playerAllies.key();
    (void)TheKey_playerEnemies.key();
    (void)TheKey_teamName.key();
    (void)TheKey_teamOwner.key();
    (void)TheKey_teamIsSingleton.key();

    const char* tmp_xfer = "/tmp/pkt154_xfer_tmp.bin";
    run_self_checks(tmp_xfer);

    const std::string sha_h = sha256_file("vendor/GeneralsX/Core/GameEngine/Include/GameLogic/SidesList.h");
    const std::string sha_c = sha256_file("vendor/GeneralsX/Core/GameEngine/Source/GameLogic/Map/SidesList.cpp");

    FILE* out = std::fopen(argv[1], "wb");
    if (!out) { std::perror(argv[1]); return 1; }

    std::fprintf(out, "{\n");
    std::fprintf(out, "  \"packet_id\": \"PKT-154\",\n");
    std::fprintf(out, "  \"work_package\": \"WP-4.1\",\n");
    std::fprintf(out, "  \"reference_commit\": \"84a7a20769aa5749f388b1f49bd542c79b3c33e8\",\n");
    std::fprintf(out, "  \"vector_count\": 1000,\n");
    std::fprintf(out, "  \"constants\": {\n");
    std::fprintf(out, "    \"MAX_PLAYER_COUNT\": %d,\n", MAX_PLAYER_COUNT);
    std::fprintf(out, "    \"MAX_TEAM_DEPTH\": %d,\n", SidesList::MAX_TEAM_DEPTH);
    std::fprintf(out, "    \"MAX_SKIRMISH_SIDES\": %d,\n", SidesList::MAX_SKIRMISH_SIDES);
    std::fprintf(out, "    \"UNLIMITED_REBUILDS\": %u,\n", static_cast<UnsignedInt>(BuildListInfo::UNLIMITED_REBUILDS));
    std::fprintf(out, "    \"MAX_RESOURCE_GATHERERS\": %d,\n", BuildListInfo::MAX_RESOURCE_GATHERERS);
    std::fprintf(out, "    \"K_SIDES_DATA_VERSION_1\": 1,\n");
    std::fprintf(out, "    \"K_SIDES_DATA_VERSION_2\": 2,\n");
    std::fprintf(out, "    \"K_SIDES_DATA_VERSION_3\": 3,\n");
    std::fprintf(out, "    \"K_PLAYERS_NAMES_FOR_SCRIPTS_VERSION_1\": 1,\n");
    std::fprintf(out, "    \"K_PLAYERS_NAMES_FOR_SCRIPTS_VERSION_2\": 2\n");
    std::fprintf(out, "  },\n");
    std::fprintf(out, "  \"source_sha256\": {\n");
    std::fprintf(out, "    \"SidesList.h\": \"%s\",\n", sha_h.c_str());
    std::fprintf(out, "    \"SidesList.cpp\": \"%s\"\n", sha_c.c_str());
    std::fprintf(out, "  },\n");
    std::fprintf(out, "  \"vectors\": [\n");

    static const char* kTemplates[5] = { "", "FactionAmerica", "FactionCivilian", "ChinaTank", "FactionGLA" };
    static const char* kSideNames[5] = { "", "PlyrAmerica", "PlyrCivilian", "PlyrChina", "SkirmishCustom" };

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        Rng rng((seed + 1u) * 0x9e3779b9u);
        SidesList sides;
        TheSidesList = &sides;
        sides.clear();
        ScriptList::resetReadLists();
        g_fs.binaryFiles.clear();

        // Collect explicit initial inputs to serialize cleanly in JSON for deterministic replay.
        std::vector<std::string> init_tmpls;
        Bool add_neutral = (rng.range(0, 3) != 0);
        Int num_players = static_cast<Int>(rng.range(1, 5));
        if (add_neutral) {
            init_tmpls.emplace_back("");
            sides.addPlayerByTemplate(AsciiString::TheEmptyString);
        }
        for (Int i = 0; i < num_players; ++i) {
            const char* tmpl = kTemplates[rng.range(1, 5)];
            init_tmpls.emplace_back(tmpl);
            sides.addPlayerByTemplate(AsciiString(tmpl));
        }

        struct BuildInput {
            Int side_idx;
            Int pos;
            std::string bname;
            std::string tname;
            uint32_t x_bits, y_bits, z_bits, angle_bits;
            Bool init_built;
            UnsignedInt num_rebuilds;
            std::string script;
            Int health;
            Bool whiner, unsellable, repairable;
            Int obj_id;
            UnsignedInt obj_ts;
            Bool under_con, supply;
            Int des_gath, cur_gath;
            Bool prio;
            std::array<Int, 10> gath;
        };
        std::vector<BuildInput> init_blds;
        std::vector<Int> init_reorders;
        struct ScriptInput { Int side_idx; Bool hs, hg; Int tag; };
        std::vector<ScriptInput> init_scripts;

        for (Int i = 0; i < sides.getNumSides(); ++i) {
            Dict* sd = sides.getSideInfo(i)->getDict();
            AsciiString allies = "NoSuchPlayer ";
            allies.concat(sd->getAsciiString(TheKey_playerName));
            if (i + 1 < sides.getNumSides()) {
                allies.concat(" ");
                allies.concat(sides.getSideInfo(i + 1)->getDict()->getAsciiString(TheKey_playerName));
            }
            sd->setAsciiString(TheKey_playerAllies, allies);

            if (rng.range(0, 2) == 0) {
                Int nbl = static_cast<Int>(rng.range(1, 4));
                for (Int b = 0; b < nbl; ++b) {
                    BuildInput bi{};
                    bi.side_idx = i;
                    char bname[32], sname[32];
                    std::snprintf(bname, sizeof(bname), "Bldg_%u_%d_%d", seed % 10, i, b);
                    std::snprintf(sname, sizeof(sname), "Scr_%d", b);
                    bi.bname = bname;
                    bi.tname = (b % 2 == 0) ? "AmericaCommandCenter" : "GLABarracks";
                    Real lx = static_cast<Real>(rng.range(10, 500)) * 0.5f;
                    Real ly = static_cast<Real>(rng.range(10, 500)) * 0.5f;
                    Real lz = static_cast<Real>(rng.range(1, 50));
                    Real angle = static_cast<Real>(rng.range(0, 360)) * 0.017453292f;
                    bi.x_bits = f32_bits(lx);
                    bi.y_bits = f32_bits(ly);
                    bi.z_bits = f32_bits(lz);
                    bi.angle_bits = f32_bits(angle);
                    bi.init_built = rng.range(0, 2) != 0 ? TRUE : FALSE;
                    bi.num_rebuilds = (b == 0 && (seed % 7 == 0)) ? BuildListInfo::UNLIMITED_REBUILDS : rng.range(0, 5);
                    bi.script = sname;
                    bi.health = static_cast<Int>(rng.range(10, 101));
                    bi.whiner = rng.range(0, 2) != 0 ? TRUE : FALSE;
                    bi.unsellable = rng.range(0, 2) != 0 ? TRUE : FALSE;
                    bi.repairable = rng.range(0, 2) != 0 ? TRUE : FALSE;
                    bi.obj_id = static_cast<Int>(rng.range(0, 100));
                    bi.obj_ts = rng.range(0, 5000);
                    bi.under_con = rng.range(0, 2) != 0 ? TRUE : FALSE;
                    bi.supply = rng.range(0, 2) != 0 ? TRUE : FALSE;
                    bi.des_gath = static_cast<Int>(rng.range(0, 5));
                    bi.cur_gath = static_cast<Int>(rng.range(0, 5));
                    bi.prio = rng.range(0, 2) != 0 ? TRUE : FALSE;
                    for (Int g = 0; g < BuildListInfo::MAX_RESOURCE_GATHERERS; ++g) {
                        bi.gath[g] = (g < 3) ? static_cast<Int>(rng.range(1, 200)) : 0;
                    }
                    bi.pos = static_cast<Int>(rng.range(0, 4));
                    init_blds.push_back(bi);

                    auto* bli = newInstance(BuildListInfo);
                    bli->setBuildingName(AsciiString(bi.bname.c_str()));
                    bli->setTemplateName(AsciiString(bi.tname.c_str()));
                    bli->setLocation(Coord3D{lx, ly, lz});
                    bli->setAngle(angle);
                    bli->setInitiallyBuilt(bi.init_built);
                    bli->setNumRebuilds(bi.num_rebuilds);
                    bli->setScript(AsciiString(bi.script.c_str()));
                    bli->setHealth(bi.health);
                    bli->setWhiner(bi.whiner);
                    bli->setUnsellable(bi.unsellable);
                    bli->setRepairable(bi.repairable);
                    bli->setObjectID(static_cast<ObjectID>(bi.obj_id));
                    bli->setObjectTimestamp(bi.obj_ts);
                    bli->setUnderConstruction(bi.under_con);
                    bli->setSupplyBuilding(bi.supply);
                    bli->setDesiredGatherers(bi.des_gath);
                    bli->setCurrentGatherers(bi.cur_gath);
                    if (bi.prio) bli->markPriorityBuild();
                    for (Int g = 0; g < BuildListInfo::MAX_RESOURCE_GATHERERS; ++g) {
                        bli->setGathererID(g, static_cast<ObjectID>(bi.gath[g]));
                    }
                    sides.getSideInfo(i)->addToBuildList(bli, bi.pos);
                }
                if (rng.range(0, 3) == 0 && sides.getSideInfo(i)->getBuildList() &&
                    sides.getSideInfo(i)->getBuildList()->getNext()) {
                    sides.getSideInfo(i)->reorderInBuildList(sides.getSideInfo(i)->getBuildList(), 2);
                    init_reorders.push_back(i);
                }
            }

            Bool hs = (rng.range(0, 2) != 0) ? TRUE : FALSE;
            Bool hg = (rng.range(0, 2) != 0) ? TRUE : FALSE;
            if (rng.range(0, 4) != 0) {
                Int tag = static_cast<Int>(seed * 10 + i);
                sides.getSideInfo(i)->setScriptList(new ScriptList(hs, hg, tag));
                init_scripts.push_back({i, hs, hg, tag});
            }
        }

        struct BadTeamInput { std::string name, owner; Bool singleton; };
        std::vector<BadTeamInput> init_bad_teams;
        if (seed % 4 == 0 && sides.getNumSides() > 1) {
            std::string pname_last = sides.getSideInfo(sides.getNumSides() - 1)->getDict()->getAsciiString(TheKey_playerName).str();
            init_bad_teams.push_back({pname_last, "NoOwner", TRUE});
            Dict bad_team1 = make_team_dict(pname_last.c_str(), "NoOwner", TRUE);
            sides.addTeam(&bad_team1);
            init_bad_teams.push_back({"OrphanTeam", "OrphanTeam", FALSE});
            Dict bad_team2 = make_team_dict("OrphanTeam", "OrphanTeam", FALSE);
            sides.addTeam(&bad_team2);
        }

        Bool val_mod = sides.validateSides();

        // Also exercise BuildListInfo::parseStructure via public INI::load on AIData block.
        char bli_ini_buf[512];
        Int ini_rebuilds = static_cast<Int>(rng.range(0, 6));
        Int ini_deg = static_cast<Int>(rng.range(0, 360));
        Real ini_x = static_cast<Real>(rng.range(1, 200)) * 0.5f;
        Real ini_y = static_cast<Real>(rng.range(1, 200)) * 0.5f;
        std::snprintf(
            bli_ini_buf, sizeof(bli_ini_buf),
            "AIData ChinaPowerPlant_%u\n"
            "  Name = Power_%u\n"
            "  Location = X:%.1f Y:%.1f\n"
            "  Rebuilds = %d\n"
            "  Angle = %d\n"
            "  InitiallyBuilt = %s\n"
            "  RallyPointOffset = X:2.5 Y:-3.5\n"
            "  AutomaticallyBuild = %s\n"
            "End\n",
            seed, seed, ini_x, ini_y, ini_rebuilds, ini_deg,
            (seed % 2 == 0) ? "Yes" : "No",
            (seed % 3 == 0) ? "Yes" : "No"
        );
        g_fs.nextFileContent = bli_ini_buf;
        g_fs.nextFileExists = true;
        {
            INI ini;
            ini.load(AsciiString("bli.ini"), INI_LOAD_OVERWRITE, nullptr);
        }
        std::vector<uint8_t> bli_ini_xfer_bytes;
        if (g_lastParsedBuildList) {
            XferSave bix;
            bix.open(AsciiString(tmp_xfer));
            bix.xferSnapshot(g_lastParsedBuildList);
            bix.close();
            bli_ini_xfer_bytes = read_file_bytes(tmp_xfer);
            std::remove(tmp_xfer);
            deleteInstance(g_lastParsedBuildList);
            g_lastParsedBuildList = nullptr;
        }

        // Serialize SidesList DataChunk ("CkMp" container)
        MemOutputStream sides_chunk_out;
        {
            DataChunkOutput cw(&sides_chunk_out);
            SidesList::WriteSidesDataChunk(cw);
        }

        // Reparse written DataChunk into a second SidesList
        SidesList parsed_sides;
        TheSidesList = &parsed_sides;
        {
            SliceChunkInputStream cin(sides_chunk_out.bytes);
            DataChunkInput reader(&cin);
            if (!reader.isValidFileType()) { std::fprintf(stderr, "Invalid DataChunk header\n"); std::exit(1); }
            reader.registerParser("SidesList", AsciiString::TheEmptyString, SidesList::ParseSidesDataChunk);
            if (!reader.parse(&parsed_sides)) { std::fprintf(stderr, "ParseSidesDataChunk failed\n"); std::exit(1); }
        }
        TheSidesList = &sides;

        // XferSave & XferCRC on SidesList
        XferSave xsave;
        xsave.open(AsciiString(tmp_xfer));
        xsave.xferSnapshot(&sides);
        xsave.close();
        std::vector<uint8_t> sides_xfer_bytes = read_file_bytes(tmp_xfer);
        std::remove(tmp_xfer);

        XferCRC xcrc;
        xcrc.open(AsciiString("crc"));
        xcrc.xferSnapshot(&sides);
        xcrc.close();
        UnsignedInt sides_crc = xcrc.getCRC();

        // Snapshot BuildListInfo[0] (v2 xfer) if present on side 0
        std::vector<uint8_t> bli_xfer_bytes;
        BuildListInfo* b0 = sides.getSideInfo(0)->getBuildList();
        if (b0) {
            XferSave bx;
            bx.open(AsciiString(tmp_xfer));
            bx.xferSnapshot(b0);
            bx.close();
            bli_xfer_bytes = read_file_bytes(tmp_xfer);
            std::remove(tmp_xfer);
        }

        // Build synthetic SkirmishScripts.scb
        std::vector<uint8_t> skirmish_scb_bytes;
        Bool include_scb = (rng.range(0, 2) == 0);
        if (include_scb) {
            MemOutputStream scb_out;
            {
                DataChunkOutput sw(&scb_out);
                sw.openDataChunk("ScriptsPlayers", K_PLAYERS_NAMES_FOR_SCRIPTS_VERSION_2);
                sw.writeInt(1);
                sw.writeInt(2);
                const char* p0_name = kSideNames[rng.range(1, 5)];
                sw.writeAsciiString(AsciiString(p0_name));
                sw.writeDict(make_side_dict(p0_name, "FactionAmerica", "", "", FALSE));
                sw.writeAsciiString(AsciiString("SkirmishModSide"));
                sw.writeDict(make_side_dict("", "FactionGLA", "", "", FALSE));
                sw.closeDataChunk();

                sw.openDataChunk("ScriptTeams", 1);
                sw.writeDict(make_team_dict("teamSkirmishExtra", p0_name, TRUE));
                sw.writeDict(make_team_dict("teamOrphanSkipped", "MissingSkirmishOwner", TRUE));
                sw.closeDataChunk();

                ScriptList* s0 = new ScriptList(TRUE, FALSE, static_cast<Int>(seed + 100));
                ScriptList* s1 = new ScriptList(FALSE, TRUE, static_cast<Int>(seed + 200));
                ScriptList* sarr[2] = { s0, s1 };
                ScriptList::WriteScriptsDataChunk(sw, sarr, 2);
                delete s0;
                delete s1;
            }
            skirmish_scb_bytes = scb_out.bytes;
            g_fs.binaryFiles["Data\\Scripts\\SkirmishScripts.scb"] = skirmish_scb_bytes;
        }

        // Copy sides to test prepareForMP_or_Skirmish
        SidesList mp_sides;
        mp_sides.clear();
        for (Int i = 0; i < sides.getNumSides(); ++i) {
            mp_sides.addSide(sides.getSideInfo(i)->getDict());
            if (ScriptList* sl = sides.getSideInfo(i)->getScriptList()) {
                if (seed % 2 == 1) {
                    mp_sides.getSideInfo(i)->setScriptList(sl->duplicate());
                }
            }
        }
        for (Int i = 0; i < sides.getNumTeams(); ++i) {
            mp_sides.addTeam(sides.getTeamInfo(i)->getDict());
        }
        TheSidesList = &mp_sides;
        mp_sides.prepareForMP_or_Skirmish();
        TheSidesList = &sides;

        Int prep_sides = mp_sides.getNumSides();
        Int prep_skirmish_sides = mp_sides.getNumSkirmishSides();
        Int prep_teams = mp_sides.getNumTeams();
        Int prep_skirmish_teams = mp_sides.getNumSkirmishTeams();
        Int prep_skirmish_script_sum = 0;
        for (Int i = 0; i < prep_skirmish_sides; ++i) {
            if (ScriptList* psl = mp_sides.getSkirmishSideInfo(i)->getScriptList()) {
                prep_skirmish_script_sum += (psl->m_tag + (psl->getScript() ? 1 : 0) + (psl->getScriptGroup() ? 2 : 0));
            }
        }

        std::fprintf(out, "    {\"seed\":%u", seed);
        std::fprintf(out, ",\"init_tmpls\":[");
        for (size_t i = 0; i < init_tmpls.size(); ++i) {
            if (i) std::fputc(',', out);
            std::fprintf(out, "\"%s\"", json_escape(init_tmpls[i]).c_str());
        }
        std::fprintf(out, "],\"init_blds\":[");
        for (size_t i = 0; i < init_blds.size(); ++i) {
            const auto& bi = init_blds[i];
            if (i) std::fputc(',', out);
            std::fprintf(
                out,
                "[%d,%d,\"%s\",\"%s\",%u,%u,%u,%u,%u,%u,\"%s\",%d,%u,%u,%u,%d,%u,%u,%u,%d,%d,%u,[%d,%d,%d,%d,%d,%d,%d,%d,%d,%d]]",
                bi.side_idx, bi.pos, bi.bname.c_str(), bi.tname.c_str(),
                bi.x_bits, bi.y_bits, bi.z_bits, bi.angle_bits,
                bi.init_built, bi.num_rebuilds, bi.script.c_str(), bi.health,
                bi.whiner, bi.unsellable, bi.repairable, bi.obj_id, bi.obj_ts,
                bi.under_con, bi.supply, bi.des_gath, bi.cur_gath, bi.prio,
                bi.gath[0], bi.gath[1], bi.gath[2], bi.gath[3], bi.gath[4],
                bi.gath[5], bi.gath[6], bi.gath[7], bi.gath[8], bi.gath[9]
            );
        }
        std::fprintf(out, "],\"init_reorders\":[");
        for (size_t i = 0; i < init_reorders.size(); ++i) {
            if (i) std::fputc(',', out);
            std::fprintf(out, "%d", init_reorders[i]);
        }
        std::fprintf(out, "],\"init_scripts\":[");
        for (size_t i = 0; i < init_scripts.size(); ++i) {
            if (i) std::fputc(',', out);
            std::fprintf(out, "[%d,%u,%u,%d]", init_scripts[i].side_idx, init_scripts[i].hs, init_scripts[i].hg, init_scripts[i].tag);
        }
        std::fprintf(out, "],\"init_bad_teams\":[");
        for (size_t i = 0; i < init_bad_teams.size(); ++i) {
            if (i) std::fputc(',', out);
            std::fprintf(out, "[\"%s\",\"%s\",%u]", init_bad_teams[i].name.c_str(), init_bad_teams[i].owner.c_str(), init_bad_teams[i].singleton);
        }
        std::fprintf(out, "],\"bli_ini\":\"%s\",\"bli_ini_xfer_hex\":\"%s\"",
                     json_escape(bli_ini_buf).c_str(), hex_encode(bli_ini_xfer_bytes).c_str());
        std::fprintf(out, ",\"val_mod\":%u,\"num_sides\":%d,\"num_teams\":%d",
                     val_mod, sides.getNumSides(), sides.getNumTeams());
        std::fprintf(out, ",\"parsed_sides\":%d,\"parsed_teams\":%d",
                     parsed_sides.getNumSides(), parsed_sides.getNumTeams());
        std::fprintf(out, ",\"sides_crc\":%u", sides_crc);
        std::fprintf(out, ",\"sides_chunk_hex\":\"%s\"", hex_encode(sides_chunk_out.bytes).c_str());
        std::fprintf(out, ",\"sides_xfer_hex\":\"%s\"", hex_encode(sides_xfer_bytes).c_str());
        std::fprintf(out, ",\"bli_xfer_hex\":\"%s\"", hex_encode(bli_xfer_bytes).c_str());
        std::fprintf(out, ",\"skirmish_scb_hex\":\"%s\"", hex_encode(skirmish_scb_bytes).c_str());
        std::fprintf(out, ",\"force_empty_scripts\":%u", (seed % 2 == 0) ? 1u : 0u);
        std::fprintf(out, ",\"prep_res\":[%d,%d,%d,%d,%d]",
                     prep_sides, prep_skirmish_sides, prep_teams, prep_skirmish_teams, prep_skirmish_script_sum);

        std::fprintf(out, ",\"final_sides\":[");
        for (Int i = 0; i < sides.getNumSides(); ++i) {
            if (i) std::fputc(',', out);
            Dict* d = sides.getSideInfo(i)->getDict();
            std::fprintf(out, "[\"%s\",\"%s\",\"%s\"]",
                         json_escape(d->getAsciiString(TheKey_playerName).str()).c_str(),
                         json_escape(d->getAsciiString(TheKey_playerAllies).str()).c_str(),
                         json_escape(d->getAsciiString(TheKey_playerEnemies).str()).c_str());
        }
        std::fprintf(out, "]");
        std::fprintf(out, ",\"final_teams\":[");
        for (Int i = 0; i < sides.getNumTeams(); ++i) {
            if (i) std::fputc(',', out);
            TeamsInfo* ti = sides.getTeamInfo(i);
            std::fprintf(out, "[\"%s\",\"%s\",%u]",
                         json_escape(ti->getDict()->getAsciiString(TheKey_teamName).str()).c_str(),
                         json_escape(ti->getDict()->getAsciiString(TheKey_teamOwner).str()).c_str(),
                         sides.isPlayerDefaultTeam(ti));
        }
        std::fprintf(out, "]}%s\n", (seed + 1u < 1000u) ? "," : "");
    }

    std::fprintf(out, "  ]\n}\n");
    std::fclose(out);
    TheSidesList = nullptr;
    return 0;
}
