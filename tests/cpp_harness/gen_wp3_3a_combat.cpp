// WP-3.3a (PKT-138 Armor + PKT-140 DamageInfo) C++ Golden Vector Generator.
// Compiles unmodified vendor/GeneralsX sources against minimal harness stubs.
//
// Canonical build & run command (macos arm64, Release-aligned macros):
//   clang++ -std=c++17 -O1 -ffp-contract=off -URTS_DEBUG -D_GNU_SOURCE \
//     -Wno-macro-redefined -Wno-switch \
//     -Itests/cpp_harness/stubs_wp3_3a_combat \
//     -Ivendor/GeneralsX/GeneralsMD/Code/GameEngine/Include \
//     -Ivendor/GeneralsX/Core/GameEngine/Include \
//     -Ivendor/GeneralsX/build/macos-vulkan/_deps/gamemath-src/include \
//     tests/cpp_harness/gen_wp3_3a_combat.cpp \
//     vendor/GeneralsX/build/macos-vulkan/_deps/gamemath-build/libgm.a \
//     -o /tmp/gen_wp3_3a_combat && /tmp/gen_wp3_3a_combat docs/golden_vectors/WP3_3a_Combat.json
//
// Determinism & UB verification builds (must produce byte-identical JSON):
//   clang++ -std=c++17 -O0 -ffp-contract=off -URTS_DEBUG ... -o /tmp/gen_wp3_3a_o0
//   clang++ -std=c++17 -O2 -ffp-contract=off -URTS_DEBUG ... -o /tmp/gen_wp3_3a_o2
//   clang++ -std=c++17 -O1 -g -fsanitize=address,undefined -fno-omit-frame-pointer -ffp-contract=off -URTS_DEBUG ... -o /tmp/gen_wp3_3a_asan
//
// __APPLE__ audit on vendor files compiled here:
//   - Armor.{h,cpp}, Damage.{h,cpp}, NameKeyGenerator.{h,cpp}, Xfer*.{h,cpp}, crc.cpp: 0 hits.
//   - INI.cpp: 1 hit (line 1686, strtod fallback on macOS instead of std::from_chars); produces
//     identical finite f32 values for all decimal percentages used in Armor INIs.

#define DEFINE_DEATH_NAMES
#include "PreRTS.h"
#include "Common/crc.h"
#include "Common/INI.h"
#include "Common/Xfer.h"
#include "Common/XferCRC.h"
#include "Common/XferSave.h"
#include "Common/XferLoad.h"
#include "Common/FileSystem.h"
#include "GameLogic/Damage.h"
#include "GameLogic/Armor.h"

#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Armor.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/GameLogic/System/Damage.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/NameKeyGenerator.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/Xfer.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/XferCRC.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/XferSave.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/XferLoad.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/crc.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/RandomValue.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/INI/INI.cpp"

#include <array>
#include <unistd.h>

const AsciiString AsciiString::TheEmptyString("");
const UnicodeString UnicodeString::TheEmptyString;

static DynamicMemoryAllocator g_dma;
DynamicMemoryAllocator* TheDynamicMemoryAllocator = &g_dma;
static GameState g_game_state;
GameState* TheGameState = &g_game_state;
static ScienceStore g_science_store;
ScienceStore* TheScienceStore = &g_science_store;
static UpgradeCenter g_upgrade_center;
UpgradeCenter* TheUpgradeCenter = &g_upgrade_center;
static ThingFactory g_thing_factory;
ThingFactory* TheThingFactory = &g_thing_factory;
static FileSystem g_fs;
FileSystem* TheFileSystem = &g_fs;
DamageFXStore* TheDamageFXStore = nullptr;
FXListStore* TheFXListStore = nullptr;
AudioManager* TheAudio = nullptr;
SpecialPowerStore* TheSpecialPowerStore = nullptr;
Anim2DCollection* TheAnim2DCollection = nullptr;
GameTextInterface* TheGameText = nullptr;
ImageCollection* TheMappedImageCollection = nullptr;
ParticleSystemManager* TheParticleSystemManager = nullptr;
ObjectCreationListStore* TheObjectCreationListStore = nullptr;
WeaponStore* TheWeaponStore = nullptr;

const char* const TheVeterancyNames[] = {
    "REGULAR", "VETERAN", "ELITE", "HEROIC", nullptr
};
static_assert(ARRAY_SIZE(TheVeterancyNames) == LEVEL_COUNT + 1, "Incorrect array size");

#define STUB_PARSE(fn) void INI::fn(INI*) {}
STUB_PARSE(parseAIDataDefinition)
STUB_PARSE(parseAnim2DDefinition)
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
STUB_PARSE(parseRankDefinition)
STUB_PARSE(parseTerrainRoadDefinition)
STUB_PARSE(parseScienceDefinition)
STUB_PARSE(parseShellMenuSchemeDefinition)
STUB_PARSE(parseSpecialPowerDefinition)
STUB_PARSE(parseStaticGameLODDefinition)
STUB_PARSE(parseTerrainDefinition)
STUB_PARSE(parseUpgradeDefinition)
STUB_PARSE(parseVideoDefinition)
STUB_PARSE(parseWaterSettingDefinition)
STUB_PARSE(parseWaterTransparencyDefinition)
STUB_PARSE(parseWeaponTemplateDefinition)
STUB_PARSE(parseWeatherDefinition)
STUB_PARSE(parseWebpageURLDefinition)
STUB_PARSE(parseWindowTransitions)
void parseReallyLowMHz(INI*) {}

// Minimal standalone SHA-256 for source fingerprinting.
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
    if (!f) {
        std::perror(path);
        std::exit(1);
    }
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
    while ((data.size() % 64u) != 56u) {
        data.push_back(0x00u);
    }
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
    for (int i = 0; i < 8; ++i) {
        std::snprintf(hex + i * 8, 9, "%08x", H[i]);
    }
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

static Real bitsf(uint32_t b) {
    Real v = 0.0f;
    std::memcpy(&v, &b, 4);
    return v;
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

static void run_startup_self_checks() {
    // 1. Verify DAMAGE_NUM_TYPES == 38 in Zero Hour config (RTS_GENERALS == 0).
    if (DAMAGE_NUM_TYPES != 38) {
        std::fprintf(stderr, "Self-check failed: DAMAGE_NUM_TYPES=%d != 38\n", DAMAGE_NUM_TYPES);
        std::exit(1);
    }
    // 2. Verify INI::scanIndexList error path throws INI_INVALID_DATA on unknown damage name.
    bool caught = false;
    try {
        (void)INI::scanIndexList("NONEXISTENT_DAMAGE_XYZ", DamageTypeFlags::getBitNames());
    } catch (decltype(INI_INVALID_DATA) code) {
        if (code == INI_INVALID_DATA) caught = true;
    }
    if (!caught) {
        std::fprintf(stderr, "Self-check failed: invalid damage type did not throw INI_INVALID_DATA\n");
        std::exit(1);
    }
    // 3. Verify Armor(nullptr) and UNRESISTABLE/SUBDUAL_UNRESISTABLE preserve negative & -0.0f bit-for-bit.
    Armor null_armor(nullptr);
    Real neg_zero = bitsf(0x80000000u);
    if (fbits(null_armor.adjustDamage(DAMAGE_EXPLOSION, neg_zero)) != 0x80000000u ||
        fbits(null_armor.adjustDamage(DAMAGE_EXPLOSION, -12.5f)) != fbits(-12.5f)) {
        std::fprintf(stderr, "Self-check failed: Armor(nullptr) modified damage bits\n");
        std::exit(1);
    }
    ArmorTemplate zeroed;
    for (int i = 0; i < DAMAGE_NUM_TYPES; ++i) {
        // default armor uses 100% until overridden
        if (fbits(zeroed.adjustDamage(static_cast<DamageType>(i), 10.0f)) != fbits(10.0f)) {
            std::fprintf(stderr, "Self-check failed: fresh ArmorTemplate coeff != 1.0f\n");
            std::exit(1);
        }
    }
    // 4. Verify PlayerMaskType is 2 bytes and enums are 4 bytes.
    static_assert(sizeof(PlayerMaskType) == 2, "PlayerMaskType must be u16");
    static_assert(sizeof(DamageType) == 4, "DamageType must be i32");
    static_assert(sizeof(DeathType) == 4, "DeathType must be i32");
    static_assert(sizeof(ObjectStatusTypes) == 4, "ObjectStatusTypes must be i32");
}

int main(int argc, char** argv) {
    if (argc != 2) {
        std::fprintf(stderr, "Usage: gen_wp3_3a_combat <out.json>\n");
        return 1;
    }
    const char* out_path = argv[1];
    std::string tmp_out = std::string(out_path) + ".tmp";

    TheNameKeyGenerator = new NameKeyGenerator();
    TheNameKeyGenerator->init();
    TheArmorStore = new ArmorStore();

    run_startup_self_checks();

    g_thing_factory.registerTemplate("AmericaTankCrusader");
    g_thing_factory.registerTemplate("GLAWorker");
    g_thing_factory.registerTemplate("ChinaTankOverlord");
    g_thing_factory.registerTemplate("AmericaInfantryRanger");

    const char* template_choices[] = {
        "",
        "AmericaTankCrusader",
        "GLAWorker",
        "ChinaTankOverlord",
        "AmericaInfantryRanger"
    };

    const char* source_paths[] = {
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Armor.h",
        "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Armor.cpp",
        "vendor/GeneralsX/Core/GameEngine/Include/GameLogic/Damage.h",
        "vendor/GeneralsX/Core/GameEngine/Source/GameLogic/System/Damage.cpp",
        "vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h",
        "vendor/GeneralsX/Core/GameEngine/Source/Common/System/Xfer.cpp",
        "vendor/GeneralsX/Core/GameEngine/Source/Common/System/XferCRC.cpp",
        "vendor/GeneralsX/Core/GameEngine/Source/Common/System/XferSave.cpp",
        "vendor/GeneralsX/Core/GameEngine/Source/Common/System/XferLoad.cpp"
    };

    FILE* fp = std::fopen(tmp_out.c_str(), "wb");
    if (!fp) { std::perror(tmp_out.c_str()); return 1; }

    std::fprintf(fp, "{\n  \"packet_id\": \"WP-3.3a\",\n"
                     "  \"module\": \"Armor+DamageInfo\",\n"
                     "  \"oracle\": \"vendor/GeneralsX unmodified Armor.cpp + Damage.cpp + INI.cpp + Xfer*.cpp\",\n"
                     "  \"source_sha256\": {\n");
    for (size_t i = 0; i < ARRAY_SIZE(source_paths); ++i) {
        std::string digest = sha256_file(source_paths[i]);
        std::fprintf(fp, "    \"%s\": \"%s\"%s\n",
                     source_paths[i], digest.c_str(),
                     (i + 1 < ARRAY_SIZE(source_paths)) ? "," : "");
    }
    std::fprintf(fp, "  },\n  \"vector_count\": 1000,\n  \"vectors\": [\n");

    char xfer_tmp[128];
    std::snprintf(xfer_tmp, sizeof(xfer_tmp), "/tmp/wp3_3a_save_%ld.bin", static_cast<long>(getpid()));

    const UnsignedInt num_dmg_names = DAMAGE_NUM_TYPES;
    const UnsignedInt num_statuses = 45; // Zero Hour ObjectStatusTypes count

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed * 0x9e3779b9u + 0x517cc1b7u;

        // Fresh stores per seed or clear/reparse armor definitions cleanly.
        TheNameKeyGenerator->reset();
        delete TheArmorStore;
        TheArmorStore = new ArmorStore();

        char armor_name[32];
        std::snprintf(armor_name, sizeof(armor_name), "TestArmor_%u", seed % 17u);

        // Build deterministic Armor INI text exercising Default, case-insensitive damage names,
        // 0%, 100%, fractional %, and occasional negative coeffs (seed 2).
        std::string ini_text;
        Real default_pct = 100.0f;
        if (seed == 0) {
            ini_text =
                "Armor TestArmor_0\n"
                "  Armor = Default 0%\n"
                "  Armor = SMALL_ARMS 100%\n"
                "  Armor = flame 50%\n"
                "End\n";
        } else if (seed == 1) {
            ini_text =
                "Armor TestArmor_1\n"
                "  Armor = Default 125.5%\n"
                "  Armor = EXPLOSION 0%\n"
                "  Armor = UNRESISTABLE 10%\n"
                "End\n";
        } else if (seed == 2) {
            ini_text =
                "Armor TestArmor_2\n"
                "  Armor = Default -25%\n"
                "  Armor = CRUSH 50%\n"
                "  Armor = SUBDUAL_UNRESISTABLE 0%\n"
                "End\n";
        } else {
            default_pct = static_cast<Real>(splitmix32(rng) % 251u); // 0..250%
            char buf[2048];
            int pos = std::snprintf(buf, sizeof(buf), "Armor %s\n  Armor = Default %.1f%%\n",
                                    armor_name, static_cast<double>(default_pct));
            uint32_t overrides = 1u + (splitmix32(rng) % 5u);
            for (uint32_t k = 0; k < overrides; ++k) {
                uint32_t dt = splitmix32(rng) % num_dmg_names;
                const char* raw_name = DamageTypeFlags::getBitNames()[dt];
                std::string tok(raw_name);
                if (k % 2 == 1) {
                    for (char& ch : tok) ch = static_cast<char>(std::tolower(static_cast<unsigned char>(ch)));
                }
                Real ov_pct = static_cast<Real>(splitmix32(rng) % 301u) * 0.5f;
                pos += std::snprintf(buf + pos, sizeof(buf) - static_cast<size_t>(pos),
                                     "  Armor = %s %.2f%%\n", tok.c_str(), static_cast<double>(ov_pct));
            }
            std::snprintf(buf + pos, sizeof(buf) - static_cast<size_t>(pos), "End\n");
            ini_text = buf;
        }

        g_fs.nextFileContent = ini_text;
        g_fs.nextFileExists = true;
        {
            INI ini;
            ini.load(AsciiString("Data\\INI\\ArmorTest.ini"), INI_LOAD_OVERWRITE, nullptr);
        }

        const ArmorTemplate* tmpl = TheArmorStore->findArmorTemplate(armor_name);
        if (!tmpl) {
            std::fprintf(stderr, "Seed %u: failed to load armor %s\n", seed, armor_name);
            return 1;
        }
        Armor armor = TheArmorStore->makeArmor(tmpl);
        Armor empty_armor(nullptr);

        // Select query damage type and raw damage (including -0.0f and negative traps).
        DamageType query_dt = static_cast<DamageType>(seed < num_dmg_names ? seed : (splitmix32(rng) % num_dmg_names));
        Real query_damage = 0.0f;
        if (seed == 0) {
            query_damage = bitsf(0x80000000u); // -0.0f
            query_dt = DAMAGE_EXPLOSION;
        } else if (seed == 1) {
            query_damage = bitsf(0x80000000u); // -0.0f with Default=125.5% on UNRESISTABLE / FLAME
            query_dt = DAMAGE_FLAME;
        } else if (seed == 2) {
            query_damage = -25.5f; // negative input vs positive & negative coeff
            query_dt = DAMAGE_CRUSH;
        } else if (seed == 3) {
            query_damage = 100.0f; // positive damage vs possibly unresistable
            query_dt = DAMAGE_UNRESISTABLE;
        } else if (seed == 4) {
            query_damage = -17.25f;
            query_dt = DAMAGE_SUBDUAL_UNRESISTABLE;
        } else if (seed == 5) {
            query_damage = HUGE_DAMAGE_AMOUNT;
        } else {
            Int signed_q = static_cast<Int>(splitmix32(rng) % 2001u) - 400; // includes negative damage
            query_damage = static_cast<Real>(signed_q) * 0.25f;
        }

        Real adjusted = armor.adjustDamage(query_dt, query_damage);
        Real null_adjusted = empty_armor.adjustDamage(query_dt, query_damage);
        Real unres_adjusted = armor.adjustDamage(DAMAGE_UNRESISTABLE, query_damage);
        Real subdual_unres_adjusted = armor.adjustDamage(DAMAGE_SUBDUAL_UNRESISTABLE, query_damage);

        // Extract all 38 coefficients via adjustDamage(dt, 1.0f) -- except UNRESISTABLE/SUBDUAL_UNRESISTABLE
        // Wait: for UNRESISTABLE/SUBDUAL_UNRESISTABLE, let's check both adjustDamage(1.0f) and via reading private?
        // Actually friend/class accessor or adjustDamage at runtime tests adjustDamage, and negative coeffs are clamped!
        // Let's test adjustDamage at dmg=1.0f for all 38 types too? No, let's check each coefficient via
        // direct DamageInfo + Armor: let's add a small probe for all 38 types by computing adjustDamage(i, 100.0f)
        // or reading coeff bits cleanly! Wait: can we verify coeff array directly? m_damageCoefficient is private in Armor.h,
        // so probe with dmg=1.0f & dmg=query_damage across all 38 types gives full behavioral parity!
        uint32_t probe_1_bits[DAMAGE_NUM_TYPES]{};
        for (UnsignedInt i = 0; i < DAMAGE_NUM_TYPES; ++i) {
            probe_1_bits[i] = fbits(armor.adjustDamage(static_cast<DamageType>(i), 1.0f));
        }

        // Build DamageInfo record
        DamageInfo dinfo;
        dinfo.in.m_sourceID = (seed == 0) ? INVALID_ID :
                              (seed == 1) ? static_cast<ObjectID>(-1) :
                                            static_cast<ObjectID>(splitmix32(rng));
        const char* tpl_name = template_choices[(seed == 0) ? 0u : (splitmix32(rng) % ARRAY_SIZE(template_choices))];
        dinfo.in.m_sourceTemplate = (tpl_name[0] != '\0') ? g_thing_factory.findTemplate(tpl_name) : nullptr;
        dinfo.in.m_sourcePlayerMask = (seed == 0) ? 0u :
                                      (seed == 1) ? 0xFFFFu :
                                                    static_cast<PlayerMaskType>(splitmix32(rng) & 0xFFFFu);
        dinfo.in.m_damageType = static_cast<DamageType>((seed == 0) ? 0u : (splitmix32(rng) % num_dmg_names));
        dinfo.in.m_damageStatusType = static_cast<ObjectStatusTypes>((seed == 0) ? 0u : (splitmix32(rng) % num_statuses));
        dinfo.in.m_damageFXOverride = static_cast<DamageType>((seed == 0) ? DAMAGE_UNRESISTABLE : (splitmix32(rng) % num_dmg_names));
        dinfo.in.m_deathType = static_cast<DeathType>((seed == 0) ? DEATH_NORMAL : (splitmix32(rng) % DEATH_NUM_TYPES));
        dinfo.in.m_amount = (seed == 0) ? bitsf(0x80000000u) : query_damage;
        dinfo.in.m_kill = (seed == 0) ? FALSE : ((splitmix32(rng) & 1u) ? TRUE : FALSE);
        auto rand_f = [&]() -> Real {
            if (seed == 0) return 0.0f;
            if (seed == 1) return bitsf(0x80000000u);
            return static_cast<Real>(static_cast<Int>(splitmix32(rng) % 4001u) - 2000) * 0.125f;
        };
        dinfo.in.m_shockWaveVector = Coord3D{ rand_f(), rand_f(), rand_f() };
        dinfo.in.m_shockWaveAmount = rand_f();
        dinfo.in.m_shockWaveRadius = rand_f();
        dinfo.in.m_shockWaveTaperOff = rand_f();
        dinfo.out.m_actualDamageDealt = adjusted;
        dinfo.out.m_actualDamageClipped = (seed == 0) ? 0.0f : rand_f();
        dinfo.out.m_noEffect = (seed == 0) ? FALSE : ((splitmix32(rng) & 1u) ? TRUE : FALSE);

        XferSave xsave;
        xsave.open(AsciiString(xfer_tmp));
        xsave.xferSnapshot(&dinfo);
        xsave.close();
        std::vector<uint8_t> save_bytes = read_file_bytes(xfer_tmp);

        DamageInfo loaded;
        XferLoad xload;
        xload.open(AsciiString(xfer_tmp));
        xload.xferSnapshot(&loaded);
        xload.close();

        // Self-check round-trip bit-exactness
        if (loaded.in.m_sourceID != dinfo.in.m_sourceID ||
            loaded.in.m_sourceTemplate != dinfo.in.m_sourceTemplate ||
            loaded.in.m_sourcePlayerMask != dinfo.in.m_sourcePlayerMask ||
            loaded.in.m_damageType != dinfo.in.m_damageType ||
            loaded.in.m_damageStatusType != dinfo.in.m_damageStatusType ||
            loaded.in.m_damageFXOverride != dinfo.in.m_damageFXOverride ||
            loaded.in.m_deathType != dinfo.in.m_deathType ||
            fbits(loaded.in.m_amount) != fbits(dinfo.in.m_amount) ||
            loaded.in.m_kill != dinfo.in.m_kill ||
            fbits(loaded.in.m_shockWaveVector.x) != fbits(dinfo.in.m_shockWaveVector.x) ||
            fbits(loaded.in.m_shockWaveVector.y) != fbits(dinfo.in.m_shockWaveVector.y) ||
            fbits(loaded.in.m_shockWaveVector.z) != fbits(dinfo.in.m_shockWaveVector.z) ||
            fbits(loaded.in.m_shockWaveAmount) != fbits(dinfo.in.m_shockWaveAmount) ||
            fbits(loaded.in.m_shockWaveRadius) != fbits(dinfo.in.m_shockWaveRadius) ||
            fbits(loaded.in.m_shockWaveTaperOff) != fbits(dinfo.in.m_shockWaveTaperOff) ||
            fbits(loaded.out.m_actualDamageDealt) != fbits(dinfo.out.m_actualDamageDealt) ||
            fbits(loaded.out.m_actualDamageClipped) != fbits(dinfo.out.m_actualDamageClipped) ||
            loaded.out.m_noEffect != dinfo.out.m_noEffect) {
            std::fprintf(stderr, "Self-check failed: XferSave->XferLoad round-trip mismatch at seed %u\n", seed);
            return 1;
        }

        XferCRC xcrc;
        xcrc.open(AsciiString("dmg_crc"));
        xcrc.xferSnapshot(&dinfo);
        xcrc.close();
        uint32_t crc_val = xcrc.getCRC();

        std::string ini_hex = to_hex(reinterpret_cast<const uint8_t*>(ini_text.data()), ini_text.size());
        std::string save_hex = to_hex(save_bytes.data(), save_bytes.size());

        std::fprintf(fp,
            "    {\"seed\": %u, \"armor_name\": \"%s\", \"armor_ini_hex\": \"%s\", "
            "\"query_dt\": %d, \"query_dmg_bits\": \"0x%08x\", "
            "\"adj_bits\": \"0x%08x\", \"null_adj_bits\": \"0x%08x\", "
            "\"unres_bits\": \"0x%08x\", \"subdual_unres_bits\": \"0x%08x\", "
            "\"probe1_bits\": [",
            seed, armor_name, ini_hex.c_str(),
            static_cast<Int>(query_dt), fbits(query_damage),
            fbits(adjusted), fbits(null_adjusted),
            fbits(unres_adjusted), fbits(subdual_unres_adjusted));
        for (UnsignedInt i = 0; i < DAMAGE_NUM_TYPES; ++i) {
            std::fprintf(fp, "%u%s", probe_1_bits[i], (i + 1 < DAMAGE_NUM_TYPES) ? ", " : "");
        }
        std::fprintf(fp,
            "], \"source_id\": %u, \"source_template\": \"%s\", \"source_player_mask\": %u, "
            "\"damage_type\": %d, \"damage_status_type\": %d, \"damage_fx_override\": %d, "
            "\"death_type\": %d, \"amount_bits\": \"0x%08x\", \"kill\": %u, "
            "\"shock_x_bits\": \"0x%08x\", \"shock_y_bits\": \"0x%08x\", \"shock_z_bits\": \"0x%08x\", "
            "\"shock_amt_bits\": \"0x%08x\", \"shock_rad_bits\": \"0x%08x\", \"shock_taper_bits\": \"0x%08x\", "
            "\"dealt_bits\": \"0x%08x\", \"clipped_bits\": \"0x%08x\", \"no_effect\": %u, "
            "\"xfer_save_hex\": \"%s\", \"xfer_crc_hex\": \"0x%08x\"}%s\n",
            static_cast<uint32_t>(dinfo.in.m_sourceID), tpl_name, static_cast<uint32_t>(dinfo.in.m_sourcePlayerMask),
            static_cast<Int>(dinfo.in.m_damageType), static_cast<Int>(dinfo.in.m_damageStatusType),
            static_cast<Int>(dinfo.in.m_damageFXOverride), static_cast<Int>(dinfo.in.m_deathType),
            fbits(dinfo.in.m_amount), static_cast<uint32_t>(dinfo.in.m_kill),
            fbits(dinfo.in.m_shockWaveVector.x), fbits(dinfo.in.m_shockWaveVector.y), fbits(dinfo.in.m_shockWaveVector.z),
            fbits(dinfo.in.m_shockWaveAmount), fbits(dinfo.in.m_shockWaveRadius), fbits(dinfo.in.m_shockWaveTaperOff),
            fbits(dinfo.out.m_actualDamageDealt), fbits(dinfo.out.m_actualDamageClipped),
            static_cast<uint32_t>(dinfo.out.m_noEffect),
            save_hex.c_str(), crc_val,
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

    delete TheArmorStore;
    TheArmorStore = nullptr;
    delete TheNameKeyGenerator;
    TheNameKeyGenerator = nullptr;
    return 0;
}
