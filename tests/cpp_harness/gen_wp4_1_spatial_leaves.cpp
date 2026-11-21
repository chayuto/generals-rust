// WP-4.1 Spatial Leaves (PKT-150 CaveSystem, PKT-151 CrateSystem, PKT-094 INICrate,
// PKT-152 PolygonTrigger, PKT-153 Powers, PKT-155 Terrain, PKT-157 TerrainTypes,
// PKT-108 INITerrain) C++ Golden Vector Generator.
// Compiles unmodified vendor/GeneralsX sources against minimal harness stubs.
//
// Canonical build & run command (macOS arm64, Release-aligned macros):
//   clang++ -std=c++20 -O1 -ffp-contract=off -URTS_DEBUG -U_ALLOW_DEBUG_CHEATS_IN_RELEASE -D_GNU_SOURCE \
//     -Wno-macro-redefined -Wno-switch -Wno-invalid-offsetof \
//     -Itests/cpp_harness/stubs_wp4_1 \
//     -Ivendor/GeneralsX/GeneralsMD/Code/GameEngine/Include \
//     -Ivendor/GeneralsX/Core/GameEngine/Include \
//     -Ivendor/GeneralsX/Core/Libraries/Source/Compression \
//     -Ivendor/GeneralsX/build/macos-vulkan/_deps/gamemath-src/include \
//     tests/cpp_harness/gen_wp4_1_spatial_leaves.cpp \
//     vendor/GeneralsX/build/macos-vulkan/_deps/gamemath-build/libgm.a \
//     -o /tmp/gen_wp4_1 && /tmp/gen_wp4_1 docs/golden_vectors/WP4_1_SpatialLeaves.json
//
// Determinism & UB verification builds (must produce byte-identical JSON):
//   -O0, -O1, -O2, and -O1 -g -fsanitize=address,undefined (-std=c++20).

#define DEFINE_DEATH_NAMES
#define DEFINE_POWER_NAMES
#define DEFINE_TERRAIN_TYPE_NAMES
#include "PreRTS.h"
#include "Compression.h"
#include "Common/crc.h"
#include "Common/Dict.h"
#include "Common/DataChunk.h"
#include "Common/INI.h"
#include "Common/MapReaderWriterInfo.h"
#include "Common/Overridable.h"
#include "Common/Override.h"
#include "Common/PlayerList.h"
#include "Common/Science.h"
#include "Common/SpecialPower.h"
#include "Common/Terrain.h"
#include "Common/TerrainTypes.h"
#include "Common/TunnelTracker.h"
#include "Common/Upgrade.h"
#include "Common/Xfer.h"
#include "Common/XferCRC.h"
#include "Common/XferSave.h"
#include "Common/XferLoad.h"
#include "Common/FileSystem.h"
#include "Common/NameKeyGenerator.h"
#include "GameClient/InGameUI.h"
#include "GameLogic/CaveSystem.h"
#include "GameLogic/CrateSystem.h"
#include "GameLogic/Damage.h"
#include "GameLogic/PolygonTrigger.h"
#include "GameLogic/Powers.h"
#include "GameLogic/TerrainLogic.h"

#include "../../vendor/GeneralsX/Core/GameEngine/Source/GameLogic/System/CaveSystem.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/GameLogic/System/CrateSystem.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/INI/INICrate.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/GameLogic/Map/PolygonTrigger.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/TerrainTypes.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/INI/INITerrain.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/System/KindOf.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/System/DataChunk.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/Dict.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/NameKeyGenerator.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/Science.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/System/Upgrade.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/SpecialPower.cpp"
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
#include <unistd.h>
#include <vector>

const AsciiString AsciiString::TheEmptyString("");
const UnicodeString UnicodeString::TheEmptyString;

SpecialPowerModuleInterface* Object::getSpecialPowerModule(const SpecialPowerTemplate*) const {
    return nullptr;
}

const char* const TheVeterancyNames[] = {
    "REGULAR", "VETERAN", "ELITE", "HEROIC", nullptr
};
static_assert(ARRAY_SIZE(TheVeterancyNames) == LEVEL_COUNT + 1, "Incorrect array size");

const char* const TheAcademyClassificationTypeNames[] = {
    "ACT_NONE", "ACT_UPGRADE_RADAR", "ACT_SUPERPOWER", nullptr
};
static_assert(ARRAY_SIZE(TheAcademyClassificationTypeNames) == ACT_COUNT + 1, "Incorrect array size");

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
static TerrainLogic g_terrain_logic;
TerrainLogic* TheTerrainLogic = &g_terrain_logic;

ArmorStore* TheArmorStore = nullptr;
DamageFXStore* TheDamageFXStore = nullptr;
FXListStore* TheFXListStore = nullptr;
Anim2DCollection* TheAnim2DCollection = nullptr;
ParticleSystemManager* TheParticleSystemManager = nullptr;
ObjectCreationListStore* TheObjectCreationListStore = nullptr;
WeaponStore* TheWeaponStore = nullptr;

Bool CompressionManager::isDataCompressed(const void*, Int) { return FALSE; }
Int CompressionManager::getUncompressedSize(const void*, Int len) { return len; }
Int CompressionManager::decompressData(void*, Int, void*, Int) { return 0; }

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
STUB_PARSE(parseSpecialPowerDefinition)
STUB_PARSE(parseStaticGameLODDefinition)
STUB_PARSE(parseTerrainRoadDefinition)
STUB_PARSE(parseUpgradeDefinition)
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

static std::string hex_encode_bytes(const void* ptr, size_t len) {
    static const char* kHex = "0123456789abcdef";
    const auto* p = static_cast<const uint8_t*>(ptr);
    std::string out;
    out.reserve(len * 2);
    for (size_t i = 0; i < len; ++i) {
        out.push_back(kHex[p[i] >> 4]);
        out.push_back(kHex[p[i] & 0x0f]);
    }
    return out;
}

static std::string hex_encode(const std::vector<uint8_t>& bytes) {
    return hex_encode_bytes(bytes.data(), bytes.size());
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
    std::vector<uint8_t> data = read_file_bytes(path);
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

struct Rng {
    uint32_t state;
    explicit Rng(uint32_t seed) : state(seed == 0 ? 0x9e3779b9u : seed) {}
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
    int32_t srange(int32_t lo, int32_t hi_incl) {
        return lo + static_cast<int32_t>(next_u32() % static_cast<uint32_t>(hi_incl - lo + 1));
    }
};

static std::vector<uint8_t> save_snapshot_bytes(Snapshot* obj, const char* tmp_path) {
    XferSave xs;
    xs.open(AsciiString(tmp_path));
    xs.xferSnapshot(obj);
    xs.close();
    std::vector<uint8_t> bytes = read_file_bytes(tmp_path);
    std::remove(tmp_path);
    return bytes;
}

static UnsignedInt crc_snapshot(Snapshot* obj) {
    XferCRC xcrc;
    xcrc.open(AsciiString("crc"));
    xcrc.xferSnapshot(obj);
    xcrc.close();
    return xcrc.getCRC();
}

static std::string serialize_crate_queries(
    const CrateSystem& cs,
    const std::vector<std::string>& names
) {
    std::string hex;
    auto push_u8 = [&](uint8_t v) {
        char b[3]; std::snprintf(b, sizeof(b), "%02x", v); hex += b;
    };
    auto push_i32 = [&](int32_t v) {
        uint32_t u = static_cast<uint32_t>(v);
        for (int s = 0; s < 4; ++s) push_u8(static_cast<uint8_t>((u >> (s * 8)) & 0xffu));
    };
    auto push_u32 = [&](uint32_t u) {
        for (int s = 0; s < 4; ++s) push_u8(static_cast<uint8_t>((u >> (s * 8)) & 0xffu));
    };
    auto push_str = [&](const AsciiString& s) {
        int32_t len = s.getLength();
        push_i32(len);
        const char* c = s.str();
        for (int32_t i = 0; i < len; ++i) push_u8(static_cast<uint8_t>(c[i]));
    };

    for (const auto& q : names) {
        const CrateTemplate* ct = cs.findCrateTemplate(AsciiString(q.c_str()));
        if (!ct) {
            push_u8(0);
            continue;
        }
        push_u8(1);
        push_str(ct->m_name);
        push_u32(f32_bits(ct->m_creationChance));
        push_i32(static_cast<int32_t>(ct->m_veterancyLevel));
        push_str(ct->m_killedByTypeKindof.toHexString());
        AsciiString sci_name = TheScienceStore->getInternalNameForScience(ct->m_killerScience);
        push_str(sci_name);
        push_u8(ct->m_isOwnedByMaker ? 1 : 0);
        push_i32(static_cast<int32_t>(ct->m_possibleCrates.size()));
        for (const auto& e : ct->m_possibleCrates) {
            push_str(e.crateName);
            push_u32(f32_bits(e.crateChance));
        }
    }
    return hex;
}

static std::string serialize_terrain_list(TerrainTypeCollection& tc) {
    std::string hex;
    auto push_u8 = [&](uint8_t v) {
        char b[3]; std::snprintf(b, sizeof(b), "%02x", v); hex += b;
    };
    auto push_i32 = [&](int32_t v) {
        uint32_t u = static_cast<uint32_t>(v);
        for (int s = 0; s < 4; ++s) push_u8(static_cast<uint8_t>((u >> (s * 8)) & 0xffu));
    };
    auto push_str = [&](const AsciiString& s) {
        int32_t len = s.getLength();
        push_i32(len);
        const char* c = s.str();
        for (int32_t i = 0; i < len; ++i) push_u8(static_cast<uint8_t>(c[i]));
    };

    std::vector<TerrainType*> items;
    for (TerrainType* t = tc.firstTerrain(); t; t = tc.nextTerrain(t)) {
        items.push_back(t);
    }
    push_i32(static_cast<int32_t>(items.size()));
    for (TerrainType* t : items) {
        push_str(t->getName());
        push_str(t->getTexture());
        push_u8(t->isBlendEdge() ? 1 : 0);
        push_i32(static_cast<int32_t>(t->getClass()));
        push_u8(t->getRestrictConstruction() ? 1 : 0);
    }
    return hex;
}

static std::string serialize_triggers_summary() {
    std::string hex;
    auto push_u8 = [&](uint8_t v) {
        char b[3]; std::snprintf(b, sizeof(b), "%02x", v); hex += b;
    };
    auto push_i32 = [&](int32_t v) {
        uint32_t u = static_cast<uint32_t>(v);
        for (int s = 0; s < 4; ++s) push_u8(static_cast<uint8_t>((u >> (s * 8)) & 0xffu));
    };
    auto push_str = [&](const AsciiString& s) {
        int32_t len = s.getLength();
        push_i32(len);
        const char* c = s.str();
        for (int32_t i = 0; i < len; ++i) push_u8(static_cast<uint8_t>(c[i]));
    };

    std::vector<PolygonTrigger*> trigs;
    for (PolygonTrigger* p = PolygonTrigger::getFirstPolygonTrigger(); p; p = p->getNext()) {
        trigs.push_back(p);
    }
    push_i32(static_cast<int32_t>(trigs.size()));
    for (PolygonTrigger* p : trigs) {
        push_str(p->getTriggerName());
        push_str(p->getLayerName());
        push_i32(p->getID());
        push_u8(p->isWaterArea() ? 1 : 0);
        push_u8(p->isRiver() ? 1 : 0);
        push_i32(p->getRiverStart());
        push_i32(p->getNumPoints());
        for (Int i = 0; i < p->getNumPoints(); ++i) {
            const ICoord3D* pt = p->getPoint(i);
            push_i32(pt->x);
            push_i32(pt->y);
            push_i32(pt->z);
        }
    }
    return hex;
}

static void run_self_checks(const char* tmp_xfer) {
    // Verify CaveSystem non-empty load throws SC_INVALID_DATA and Sparse reset works.
    CaveSystem cave;
    TheCaveSystem = &cave;
    cave.init();
    cave.registerNewCave(0);
    cave.registerNewCave(2);
    if (cave.getTunnelTrackerForCaveIndex(1) != nullptr) std::exit(1);
    if (!cave.canSwitchIndexToIndex(0, 1)) std::exit(1);
    cave.getTunnelTrackerForCaveIndex(2)->m_containCount = 2;
    if (cave.canSwitchIndexToIndex(0, 2)) std::exit(1);
    if (!cave.canSwitchIndexToIndex(-1, 0)) std::exit(1);
    cave.reset();

    cave.registerNewCave(0);
    cave.getTunnelTrackerForCaveIndex(0)->m_containCount = 1;
    cave.getTunnelTrackerForCaveIndex(0)->m_tag = 0x1234abcd;
    XferSave xs;
    xs.open(AsciiString(tmp_xfer));
    xs.xferSnapshot(&cave);
    xs.close();
    XferLoad xl;
    xl.open(AsciiString(tmp_xfer));
    bool threw = false;
    try {
        xl.xferSnapshot(&cave);
    } catch (SaveCode sc) {
        threw = (sc == SC_INVALID_DATA);
    }
    xl.close();
    std::remove(tmp_xfer);
    if (!threw) {
        std::fprintf(stderr, "Self-check failed: CaveSystem non-empty XferLoad did not throw\n");
        std::exit(1);
    }
    cave.reset();
    TheCaveSystem = nullptr;

    // Verify PolygonTrigger deleteTriggers & DataChunk empty parse.
    PolygonTrigger::deleteTriggers();
    auto* pt = newInstance(PolygonTrigger)(2);
    pt->addPoint(ICoord3D{0, 0, 0});
    pt->addPoint(ICoord3D{100, 0, 0});
    pt->addPoint(ICoord3D{100, 100, 0});
    PolygonTrigger::addPolygonTrigger(pt);
    if (!PolygonTrigger::getPolygonTriggerByID(pt->getID())) std::exit(1);
    MemOutputStream mos;
    {
        DataChunkOutput dco(&mos);
        PolygonTrigger::WritePolygonTriggersDataChunk(dco);
    }
    PolygonTrigger::deleteTriggers();
    SliceChunkInputStream cis(mos.bytes);
    DataChunkInput dci(&cis);
    dci.registerParser(AsciiString("PolygonTriggers"), AsciiString::TheEmptyString,
        PolygonTrigger::ParsePolygonTriggersDataChunk, nullptr);
    if (!dci.parse(nullptr) || PolygonTrigger::getFirstPolygonTrigger() == nullptr) {
        std::fprintf(stderr, "Self-check failed: PolygonTrigger DataChunk roundtrip\n");
        std::exit(1);
    }
    PolygonTrigger::deleteTriggers();
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

    // Register test sciences cleanly via Science::parseScienceDefinition.
    g_fs.nextFileContent =
        "Science SCIENCE_CRATE_ALPHA\n  SciencePurchasePointCost = 1\n  IsGrantable = Yes\nEnd\n"
        "Science SCIENCE_CRATE_BETA\n  SciencePurchasePointCost = 2\n  IsGrantable = Yes\nEnd\n"
        "Science SCIENCE_CRATE_GAMMA\n  SciencePurchasePointCost = 3\n  IsGrantable = Yes\nEnd\n";
    g_fs.nextFileExists = true;
    {
        INI ini;
        ini.load(AsciiString("sciences.ini"), INI_LOAD_OVERWRITE, nullptr);
    }
    g_fs.nextFileExists = false;

    char tmp_xfer[128];
    std::snprintf(tmp_xfer, sizeof(tmp_xfer), "/tmp/wp4_1_xfer_%d.bin", static_cast<int>(::getpid()));
    run_self_checks(tmp_xfer);

    const char* source_paths[12] = {
        "vendor/GeneralsX/Core/GameEngine/Include/GameLogic/CaveSystem.h",
        "vendor/GeneralsX/Core/GameEngine/Source/GameLogic/System/CaveSystem.cpp",
        "vendor/GeneralsX/Core/GameEngine/Include/GameLogic/CrateSystem.h",
        "vendor/GeneralsX/Core/GameEngine/Source/GameLogic/System/CrateSystem.cpp",
        "vendor/GeneralsX/Core/GameEngine/Source/Common/INI/INICrate.cpp",
        "vendor/GeneralsX/Core/GameEngine/Include/GameLogic/PolygonTrigger.h",
        "vendor/GeneralsX/Core/GameEngine/Source/GameLogic/Map/PolygonTrigger.cpp",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Powers.h",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Terrain.h",
        "vendor/GeneralsX/Core/GameEngine/Include/Common/TerrainTypes.h",
        "vendor/GeneralsX/Core/GameEngine/Source/Common/TerrainTypes.cpp",
        "vendor/GeneralsX/Core/GameEngine/Source/Common/INI/INITerrain.cpp",
    };

    FILE* out = std::fopen(argv[1], "wb");
    if (!out) { std::perror(argv[1]); return 1; }

    std::fprintf(out, "{\n");
    std::fprintf(out, "  \"work_package\": \"WP-4.1\",\n");
    std::fprintf(out, "  \"packets\": [\"PKT-150\", \"PKT-151\", \"PKT-094\", \"PKT-152\", \"PKT-153\", \"PKT-155\", \"PKT-157\", \"PKT-108\"],\n");
    std::fprintf(out, "  \"reference_commit\": \"84a7a20769aa5749f388b1f49bd542c79b3c33e8\",\n");
    std::fprintf(out, "  \"vector_count\": 1000,\n");
    std::fprintf(out, "  \"constants\": {\n");
    std::fprintf(out, "    \"POWERS_NUM_POWERS\": %d,\n", POWERS_NUM_POWERS);
    std::fprintf(out, "    \"MAX_TERRAIN_NAME_LEN\": %d,\n", MAX_TERRAIN_NAME_LEN);
    std::fprintf(out, "    \"TERRAIN_NUM_CLASSES\": %d,\n", TERRAIN_NUM_CLASSES);
    std::fprintf(out, "    \"K_TRIGGERS_VERSION_1\": %d,\n", K_TRIGGERS_VERSION_1);
    std::fprintf(out, "    \"K_TRIGGERS_VERSION_2\": %d,\n", K_TRIGGERS_VERSION_2);
    std::fprintf(out, "    \"K_TRIGGERS_VERSION_3\": %d,\n", K_TRIGGERS_VERSION_3);
    std::fprintf(out, "    \"K_TRIGGERS_VERSION_4\": %d,\n", K_TRIGGERS_VERSION_4);
    std::fprintf(out, "    \"POWER_NAMES\": [");
    for (Int i = 0; PowerNames[i]; ++i) {
        std::fprintf(out, "%s\"%s\"", i == 0 ? "" : ", ", PowerNames[i]);
    }
    std::fprintf(out, "],\n");
    std::fprintf(out, "    \"TERRAIN_TYPE_NAMES\": [");
    for (Int i = 0; terrainTypeNames[i]; ++i) {
        std::fprintf(out, "%s\"%s\"", i == 0 ? "" : ", ", terrainTypeNames[i]);
    }
    std::fprintf(out, "]\n  },\n");

    std::fprintf(out, "  \"source_sha256\": {\n");
    for (size_t i = 0; i < 12; ++i) {
        std::string sha = sha256_file(source_paths[i]);
        std::fprintf(out, "    \"%s\": \"%s\"%s\n", source_paths[i], sha.c_str(), i + 1 < 12 ? "," : "");
    }
    std::fprintf(out, "  },\n");
    std::fprintf(out, "  \"vectors\": [\n");

    static const char* kSciNames[4] = {
        nullptr, "SCIENCE_CRATE_ALPHA", "SCIENCE_CRATE_BETA", "SCIENCE_CRATE_GAMMA"
    };
    static const char* kTexNames[4] = {
        "TGrass01.tga", "TDirt02.tga", "TSnow03.tga", "TSand04.tga"
    };

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        Rng rng(seed ^ 0x51f15a41u);

        // --- 1. CaveSystem ---
        CaveSystem cave;
        TheCaveSystem = &cave;
        cave.init();
        Int n_ops = static_cast<Int>(1 + (rng.next_u32() % 5));
        std::vector<Int> cave_ops_flat;
        for (Int i = 0; i < n_ops; ++i) {
            Int idx = static_cast<Int>(rng.next_u32() % 8);
            Int cnt = static_cast<Int>(rng.next_u32() % 4);
            UnsignedInt tag = rng.next_u32();
            cave.registerNewCave(idx);
            TunnelTracker* tr = cave.getTunnelTrackerForCaveIndex(idx);
            tr->m_containCount = cnt;
            tr->m_tag = tag;
            cave_ops_flat.push_back(idx);
            cave_ops_flat.push_back(cnt);
            cave_ops_flat.push_back(static_cast<Int>(tag));
        }
        Int sw_old_a = rng.srange(-1, 9);
        Int sw_new_a = rng.srange(-1, 9);
        Int sw_old_b = rng.srange(0, 7);
        Int sw_new_b = rng.srange(0, 7);
        Bool sw_ok_a = cave.canSwitchIndexToIndex(sw_old_a, sw_new_a);
        Bool sw_ok_b = cave.canSwitchIndexToIndex(sw_old_b, sw_new_b);
        std::vector<Int> cave_queries = {
            rng.srange(-2, 10),
            rng.srange(0, 8),
            rng.srange(0, 8),
            rng.srange(-1, 9)
        };
        std::vector<Int> cave_query_counts;
        for (Int q : cave_queries) {
            // Avoid C++ DEBUG_ASSERTCRASH print if any; call getTunnelTrackerForCaveIndex cleanly in Release.
            TunnelTracker* tr = cave.getTunnelTrackerForCaveIndex(q);
            cave_query_counts.push_back(tr ? static_cast<Int>(tr->m_containCount) : -1);
        }
        cave.reset();

        Int dense_n = static_cast<Int>(1 + (rng.next_u32() % 6));
        std::vector<Int> cave_dense_counts(dense_n);
        std::vector<UnsignedInt> cave_dense_tags(dense_n);
        for (Int i = 0; i < dense_n; ++i) {
            cave.registerNewCave(i);
            Int c = static_cast<Int>(rng.next_u32() % 12);
            UnsignedInt t = rng.next_u32();
            cave_dense_counts[i] = c;
            cave_dense_tags[i] = t;
            TunnelTracker* tr = cave.getTunnelTrackerForCaveIndex(i);
            tr->m_containCount = c;
            tr->m_tag = t;
        }
        std::vector<uint8_t> cave_xfer_bytes = save_snapshot_bytes(&cave, tmp_xfer);
        UnsignedInt cave_xfer_crc = crc_snapshot(&cave);
        cave.reset();
        TheCaveSystem = nullptr;

        // --- 2. CrateSystem + INICrate ---
        CrateSystem crate_sys;
        TheCrateSystem = &crate_sys;
        crate_sys.init();

        Bool has_default_crate = (rng.next_u32() % 3) != 0;
        std::string crate_base_ini;
        if (has_default_crate) {
            Int d_chance = rng.srange(5, 95);
            crate_base_ini += "CrateData DefaultCrate\n";
            crate_base_ini += "  CreationChance = 0." + std::to_string(d_chance) + "\n";
            crate_base_ini += "  VeterancyLevel = REGULAR\n";
            crate_base_ini += "  KilledByType = VEHICLE INFANTRY\n";
            crate_base_ini += "  KillerScience = SCIENCE_CRATE_ALPHA\n";
            crate_base_ini += "  OwnedByMaker = No\n";
            crate_base_ini += "End\n";
        }
        for (Int ci = 0; ci < 2; ++ci) {
            std::string cname = ci == 0 ? "CrateA" : "CrateB";
            Int pct = rng.srange(1, 99);
            const char* vet = TheVeterancyNames[rng.next_u32() % 4];
            const char* k1 = BitFlags<KINDOF_COUNT, KindOfMaskTypeTag>::getNameFromSingleBit(
                static_cast<Int>(rng.next_u32() % KINDOF_COUNT));
            const char* k2 = BitFlags<KINDOF_COUNT, KindOfMaskTypeTag>::getNameFromSingleBit(
                static_cast<Int>(rng.next_u32() % KINDOF_COUNT));
            std::string kind_line;
            if ((rng.next_u32() % 4) == 0) {
                kind_line = "NONE";
            } else if ((rng.next_u32() % 3) == 0 && has_default_crate) {
                kind_line = std::string("+") + k1 + " -INFANTRY";
            } else {
                kind_line = std::string(k1) + " " + k2;
            }
            const char* sci = kSciNames[rng.next_u32() % 4];
            Bool owned = (rng.next_u32() % 2) != 0;
            crate_base_ini += "CrateData " + cname + "\n";
            crate_base_ini += "  CreationChance = 0." + std::string(pct < 10 ? "0" : "") + std::to_string(pct) + "\n";
            crate_base_ini += std::string("  VeterancyLevel = ") + vet + "\n";
            crate_base_ini += "  KilledByType = " + kind_line + "\n";
            if (sci != nullptr) {
                crate_base_ini += std::string("  KillerScience = ") + sci + "\n";
            }
            crate_base_ini += std::string("  OwnedByMaker = ") + (owned ? "Yes\n" : "No\n");
            Int n_obj = static_cast<Int>(1 + (rng.next_u32() % 3));
            for (Int oi = 0; oi < n_obj; ++oi) {
                Int w = rng.srange(10, 250);
                crate_base_ini += "  CrateObject = SalvageItem" + std::to_string(oi) + " 0." + std::to_string(w) + "\n";
            }
            crate_base_ini += "End\n";
        }
        g_fs.nextFileContent = crate_base_ini;
        g_fs.nextFileExists = true;
        {
            INI ini;
            ini.load(AsciiString("crate_base.ini"), INI_LOAD_OVERWRITE, nullptr);
        }

        std::string crate_ov_ini;
        Int ov_pct = rng.srange(10, 90);
        crate_ov_ini += "CrateData CrateA\n";
        crate_ov_ini += "  CreationChance = 0." + std::to_string(ov_pct) + "\n";
        crate_ov_ini += "  OwnedByMaker = Yes\n";
        crate_ov_ini += "End\n";
        crate_ov_ini += "CrateData CrateOvOnly\n";
        crate_ov_ini += "  CreationChance = 0.75\n";
        crate_ov_ini += "  VeterancyLevel = HEROIC\n";
        crate_ov_ini += "  CrateObject = BonusCash 1.5\n";
        crate_ov_ini += "End\n";
        g_fs.nextFileContent = crate_ov_ini;
        g_fs.nextFileExists = true;
        {
            INI ini;
            ini.load(AsciiString("crate_override.ini"), INI_LOAD_CREATE_OVERRIDES, nullptr);
        }
        g_fs.nextFileExists = false;

        std::vector<std::string> crate_queries = {"DefaultCrate", "CrateA", "CrateB", "CrateOvOnly", "MissingCrate"};
        std::string crate_pre_hex = serialize_crate_queries(crate_sys, crate_queries);
        crate_sys.reset();
        std::string crate_post_hex = serialize_crate_queries(crate_sys, crate_queries);
        TheCrateSystem = nullptr;

        // --- 3. PolygonTrigger ---
        PolygonTrigger::deleteTriggers();
        Int poly_count = static_cast<Int>(3 + (rng.next_u32() % 5));
        std::vector<Int> poly_init_pts;
        auto* trig = newInstance(PolygonTrigger)(2);
        trig->setTriggerName(AsciiString(("Trig" + std::to_string(seed)).c_str()));
        trig->setLayerName(AsciiString((seed % 2 == 0) ? "LayerMain" : ""));
        Bool poly_is_water = (rng.next_u32() % 2) != 0;
        Bool poly_is_river = poly_is_water && ((rng.next_u32() % 2) != 0);
        Int poly_river_start = static_cast<Int>(rng.next_u32() % 4);
        trig->setWaterArea(poly_is_water);
        trig->setRiver(poly_is_river);
        trig->setRiverStart(poly_river_start);
        trig->setDoExportWithScripts((rng.next_u32() % 2) != 0);
        for (Int pi = 0; pi < poly_count; ++pi) {
            Int px = rng.srange(-400, 600);
            Int py = rng.srange(-400, 600);
            Int pz = rng.srange(0, 120);
            trig->addPoint(ICoord3D{px, py, pz});
            poly_init_pts.push_back(px);
            poly_init_pts.push_back(py);
            poly_init_pts.push_back(pz);
        }
        Int ins_idx = rng.srange(0, poly_count);
        ICoord3D ins_pt{rng.srange(-300, 300), rng.srange(-300, 300), rng.srange(0, 50)};
        trig->insertPoint(ins_pt, ins_idx);

        Int set_idx = rng.srange(0, trig->getNumPoints() - 1);
        ICoord3D set_pt{rng.srange(-300, 300), rng.srange(-300, 300), rng.srange(0, 50)};
        trig->setPoint(set_pt, set_idx);

        Int del_idx = rng.srange(0, trig->getNumPoints() - 1);
        trig->deletePoint(del_idx);

        Int clamp_q = rng.srange(-2, trig->getNumPoints() + 2);
        const ICoord3D clamped_pt = *trig->getPoint(clamp_q);

        Real rad = trig->getRadius();
        Coord3D center{};
        trig->getCenterPoint(&center);

        std::vector<Int> poly_probe_pts;
        std::vector<Int> poly_probe_inside;
        for (Int qi = 0; qi < 4; ++qi) {
            ICoord3D qp{rng.srange(-450, 650), rng.srange(-450, 650), 0};
            poly_probe_pts.push_back(qp.x);
            poly_probe_pts.push_back(qp.y);
            poly_probe_pts.push_back(qp.z);
            poly_probe_inside.push_back(trig->pointInTrigger(qp) ? 1 : 0);
        }
        // Include exact vertex & edge midpoint probe to exercise boundary cases.
        if (trig->getNumPoints() > 0) {
            ICoord3D vp = *trig->getPoint(0);
            poly_probe_pts.push_back(vp.x);
            poly_probe_pts.push_back(vp.y);
            poly_probe_pts.push_back(vp.z);
            poly_probe_inside.push_back(trig->pointInTrigger(vp) ? 1 : 0);
        }

        std::vector<uint8_t> poly_xfer_bytes = save_snapshot_bytes(trig, tmp_xfer);
        UnsignedInt poly_xfer_crc = crc_snapshot(trig);

        // Build trigger list with head-insert semantics, write v4 chunk, plus synthesize v1/v2/v3 chunk.
        PolygonTrigger::addPolygonTrigger(trig);
        auto* trig2 = newInstance(PolygonTrigger)(3);
        trig2->setTriggerName(AsciiString("SecondPoly"));
        trig2->setLayerName(AsciiString("Secondary"));
        trig2->addPoint(ICoord3D{10, 10, 5});
        trig2->addPoint(ICoord3D{50, 10, 5});
        trig2->addPoint(ICoord3D{30, 50, 5});
        PolygonTrigger::addPolygonTrigger(trig2);

        MemOutputStream poly_v4_out;
        {
            DataChunkOutput dco(&poly_v4_out);
            PolygonTrigger::WritePolygonTriggersDataChunk(dco);
        }
        PolygonTrigger::deleteTriggers();

        {
            SliceChunkInputStream cis(poly_v4_out.bytes);
            DataChunkInput dci(&cis);
            dci.registerParser(AsciiString("PolygonTriggers"), AsciiString::TheEmptyString,
                PolygonTrigger::ParsePolygonTriggersDataChunk, nullptr);
            dci.parse(nullptr);
        }
        std::string poly_v4_roundtrip_hex = serialize_triggers_summary();
        PolygonTrigger::deleteTriggers();

        // Synthesize v1..v3 DataChunk including a 1-point trigger (dropped by Retail CRC == 0).
        Int legacy_ver = static_cast<Int>(1 + (seed % 3));
        TheGlobalData->m_waterExtentX = rng.srange(100, 1000);
        TheGlobalData->m_waterExtentY = rng.srange(100, 1000);
        MemOutputStream poly_legacy_out;
        {
            DataChunkOutput dco(&poly_legacy_out);
            dco.openDataChunk("PolygonTriggers", static_cast<DataChunkVersionType>(legacy_ver));
            dco.writeInt(2); // 2 triggers: 1-pt dropped, 3-pt kept.
            // Trigger 1 (1 point -> dropped when numPoints < 2):
            dco.writeAsciiString(AsciiString("DroppedSinglePt"));
            dco.writeInt(10);
            if (legacy_ver >= K_TRIGGERS_VERSION_2) dco.writeByte(0);
            if (legacy_ver >= K_TRIGGERS_VERSION_3) { dco.writeByte(0); dco.writeInt(0); }
            dco.writeInt(1);
            dco.writeInt(5); dco.writeInt(6); dco.writeInt(7);
            // Trigger 2 (3 points):
            dco.writeAsciiString(AsciiString("KeptTri"));
            dco.writeInt(25);
            if (legacy_ver >= K_TRIGGERS_VERSION_2) dco.writeByte((rng.next_u32() % 2) ? 1 : 0);
            if (legacy_ver >= K_TRIGGERS_VERSION_3) { dco.writeByte(1); dco.writeInt(2); }
            dco.writeInt(3);
            dco.writeInt(0); dco.writeInt(0); dco.writeInt(0);
            dco.writeInt(40); dco.writeInt(0); dco.writeInt(0);
            dco.writeInt(20); dco.writeInt(40); dco.writeInt(0);
            dco.closeDataChunk();
        }
        PolygonTrigger::deleteTriggers();
        {
            SliceChunkInputStream cis(poly_legacy_out.bytes);
            DataChunkInput dci(&cis);
            dci.registerParser(AsciiString("PolygonTriggers"), AsciiString::TheEmptyString,
                PolygonTrigger::ParsePolygonTriggersDataChunk, nullptr);
            dci.parse(nullptr);
        }
        std::string poly_legacy_roundtrip_hex = serialize_triggers_summary();
        // Check next generated trigger ID (`s_currentID = maxTriggerId + 1`).
        auto* id_probe = newInstance(PolygonTrigger)(2);
        Int next_trig_id = id_probe->getID();
        deleteInstance(id_probe);
        PolygonTrigger::deleteTriggers();

        // --- 4. TerrainTypes + INITerrain ---
        TerrainTypeCollection tc;
        TheTerrainTypes = &tc;
        std::string terrain_ini;
        Bool has_def_terrain = (rng.next_u32() % 2) == 0;
        if (has_def_terrain) {
            const char* cls_name = terrainTypeNames[rng.next_u32() % TERRAIN_NUM_CLASSES];
            terrain_ini += "Terrain DefaultTerrain\n";
            terrain_ini += std::string("  Texture = ") + kTexNames[rng.next_u32() % 4] + "\n";
            terrain_ini += "  BlendEdges = Yes\n";
            terrain_ini += std::string("  Class = ") + cls_name + "\n";
            terrain_ini += "  RestrictConstruction = Yes\n";
            terrain_ini += "End\n";
        }
        Int n_ter = static_cast<Int>(2 + (rng.next_u32() % 3));
        for (Int ti = 0; ti < n_ter; ++ti) {
            std::string tname = "Terrain_" + std::to_string(ti);
            const char* cls_name = terrainTypeNames[rng.next_u32() % TERRAIN_NUM_CLASSES];
            terrain_ini += "Terrain " + tname + "\n";
            terrain_ini += std::string("  Texture = ") + kTexNames[rng.next_u32() % 4] + "\n";
            if ((rng.next_u32() % 2) == 0) {
                terrain_ini += "  BlendEdges = No\n";
            }
            terrain_ini += std::string("  Class = ") + cls_name + "\n";
            if ((rng.next_u32() % 2) == 0) {
                terrain_ini += "  RestrictConstruction = No\n";
            }
            terrain_ini += "End\n";
        }
        // Re-open existing Terrain_0 to verify findTerrain() in-place update without duplicating list node.
        terrain_ini += "Terrain Terrain_0\n";
        terrain_ini += std::string("  Texture = ") + kTexNames[rng.next_u32() % 4] + "\n";
        terrain_ini += "End\n";

        g_fs.nextFileContent = terrain_ini;
        g_fs.nextFileExists = true;
        {
            INI ini;
            ini.load(AsciiString("terrain.ini"), INI_LOAD_OVERWRITE, nullptr);
        }
        g_fs.nextFileExists = false;
        std::string terrain_hex = serialize_terrain_list(tc);
        TheTerrainTypes = nullptr;

        // --- Emit single JSON object line for seed ---
        std::fprintf(out, "    {\"seed\": %u", seed);

        auto print_i32_vec = [&](const char* k, const std::vector<Int>& v) {
            std::fprintf(out, ", \"%s\": [", k);
            for (size_t i = 0; i < v.size(); ++i) std::fprintf(out, "%s%d", i ? "," : "", v[i]);
            std::fprintf(out, "]");
        };
        auto print_u32_vec = [&](const char* k, const std::vector<UnsignedInt>& v) {
            std::fprintf(out, ", \"%s\": [", k);
            for (size_t i = 0; i < v.size(); ++i) std::fprintf(out, "%s%u", i ? "," : "", v[i]);
            std::fprintf(out, "]");
        };

        // CaveSystem fields
        print_i32_vec("cave_ops", cave_ops_flat);
        std::fprintf(out, ", \"cave_switches\": [%d,%d,%d,%d]", sw_old_a, sw_new_a, sw_old_b, sw_new_b);
        std::fprintf(out, ", \"cave_switch_ok\": [%d,%d]", sw_ok_a ? 1 : 0, sw_ok_b ? 1 : 0);
        print_i32_vec("cave_queries", cave_queries);
        print_i32_vec("cave_query_counts", cave_query_counts);
        print_i32_vec("cave_dense_counts", cave_dense_counts);
        print_u32_vec("cave_dense_tags", cave_dense_tags);
        std::fprintf(out, ", \"cave_xfer_crc\": \"0x%08x\", \"cave_xfer_hex\": \"%s\"",
            cave_xfer_crc, hex_encode(cave_xfer_bytes).c_str());

        // CrateSystem fields
        std::fprintf(out, ", \"crate_base_ini_hex\": \"%s\"",
            hex_encode_bytes(crate_base_ini.data(), crate_base_ini.size()).c_str());
        std::fprintf(out, ", \"crate_ov_ini_hex\": \"%s\"",
            hex_encode_bytes(crate_ov_ini.data(), crate_ov_ini.size()).c_str());
        std::fprintf(out, ", \"crate_pre_hex\": \"%s\", \"crate_post_hex\": \"%s\"",
            crate_pre_hex.c_str(), crate_post_hex.c_str());

        // PolygonTrigger fields
        print_i32_vec("poly_init_pts", poly_init_pts);
        std::fprintf(out, ", \"poly_ops\": [%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d]",
            ins_idx, ins_pt.x, ins_pt.y, ins_pt.z,
            set_idx, set_pt.x, set_pt.y, set_pt.z,
            del_idx, clamp_q, poly_river_start);
        std::fprintf(out, ", \"poly_flags\": [%d,%d,%d]",
            poly_is_water ? 1 : 0, poly_is_river ? 1 : 0, TheGlobalData->m_waterExtentX);
        std::fprintf(out, ", \"poly_water_ext_y\": %d", TheGlobalData->m_waterExtentY);
        std::fprintf(out, ", \"poly_clamped_pt\": [%d,%d,%d]", clamped_pt.x, clamped_pt.y, clamped_pt.z);
        std::fprintf(out, ", \"poly_radius_bits\": %u, \"poly_center_bits\": [%u,%u,%u]",
            f32_bits(rad), f32_bits(center.x), f32_bits(center.y), f32_bits(center.z));
        print_i32_vec("poly_probe_pts", poly_probe_pts);
        print_i32_vec("poly_probe_inside", poly_probe_inside);
        std::fprintf(out, ", \"poly_xfer_crc\": \"0x%08x\", \"poly_xfer_hex\": \"%s\"",
            poly_xfer_crc, hex_encode(poly_xfer_bytes).c_str());
        std::fprintf(out, ", \"poly_v4_chunk_hex\": \"%s\", \"poly_v4_rt_hex\": \"%s\"",
            hex_encode(poly_v4_out.bytes).c_str(), poly_v4_roundtrip_hex.c_str());
        std::fprintf(out, ", \"poly_legacy_ver\": %d, \"poly_legacy_chunk_hex\": \"%s\", \"poly_legacy_rt_hex\": \"%s\", \"poly_next_id\": %d",
            legacy_ver, hex_encode(poly_legacy_out.bytes).c_str(), poly_legacy_roundtrip_hex.c_str(), next_trig_id);

        // TerrainTypes fields
        std::fprintf(out, ", \"terrain_ini_hex\": \"%s\", \"terrain_hex\": \"%s\"",
            hex_encode_bytes(terrain_ini.data(), terrain_ini.size()).c_str(), terrain_hex.c_str());

        std::fprintf(out, "}%s\n", seed + 1 < 1000 ? "," : "");
    }

    std::fprintf(out, "  ]\n}\n");
    std::fclose(out);
    delete TheScienceStore;
    delete TheNameKeyGenerator;
    return 0;
}
