// PKT-442 (GlobalData, WeaponBonus, WeaponBonusSet) (WP-7.1) C++ Golden Vector Generator.
// Compiles unmodified vendor/GeneralsX GlobalData.cpp + INI.cpp + GameType.cpp +
// RandomValue.cpp + Damage.cpp + crc.cpp against Release macro configuration.
//
// Canonical build & run command (macOS arm64, Release-aligned macros):
//   clang++ -std=c++17 -O1 -ffp-contract=off -URTS_DEBUG -U_ALLOW_DEBUG_CHEATS_IN_RELEASE \
//     -DRTS_RELEASE=1 -DRTS_ZEROHOUR=1 -DRTS_GENERALS=0 \
//     -DPRESERVE_RETAIL_SCRIPTED_CAMERA=1 -DALLOW_MONEY_PER_MINUTE_FOR_PLAYER=0 \
//     -DENABLE_CONFIGURABLE_SHROUD=1 -DRETAIL_COMPATIBLE_CRC=0 -DRETAIL_COMPATIBLE_XFER_SAVE=1 \
//     -Wno-macro-redefined -Wno-switch -Wno-invalid-offsetof \
//     -Itests/cpp_harness/stubs_pkt442 \
//     -Ivendor/GeneralsX/GeneralsMD/Code/GameEngine/Include \
//     -Ivendor/GeneralsX/Core/GameEngine/Include \
//     -Ivendor/GeneralsX/build/macos-vulkan/_deps/gamemath-src/include \
//     vendor/GeneralsX/build/macos-vulkan/_deps/gamemath-build/libgm.a \
//     tests/cpp_harness/gen_pkt442_global_data.cpp \
//     -o /tmp/gen_pkt442 && /tmp/gen_pkt442 docs/golden_vectors/PKT-442.json
//
// Determinism & UB verification builds (must produce byte-identical JSON):
//   -O0, -O1, -O2, and -O1 -g -fsanitize=address,undefined.

#define DEFINE_DEATH_NAMES
#define DEFINE_WEAPONBONUSCONDITION_NAMES
#define DEFINE_WEAPONBONUSFIELD_NAMES

#include "PreRTS.h"
#include "Common/crc.h"
#include "Common/GlobalData.h"
#include "Common/INI.h"
#include "Common/Money.h"
#include "Common/OptionPreferences.h"
#include "Common/version.h"
#include "GameLogic/Damage.h"
#include "GameLogic/Weapon.h"

#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/crc.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/RandomValue.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/GameLogic/System/Damage.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/GameType.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/INI/INI.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/GlobalData.cpp"

const AsciiString AsciiString::TheEmptyString;
const UnicodeString UnicodeString::TheEmptyString;

static FileSystem g_fs;
FileSystem* TheFileSystem = &g_fs;
static Version g_version;
Version* TheVersion = &g_version;
OptionPreferencesOverrides g_optionPrefs;
namespace addon {
Bool g_hasFullviewportDat = FALSE;
}

DamageFXStore* TheDamageFXStore = nullptr;
AudioManager* TheAudio = nullptr;
ScienceStore* TheScienceStore = nullptr;
SpecialPowerStore* TheSpecialPowerStore = nullptr;
ThingFactory* TheThingFactory = nullptr;
UpgradeCenter* TheUpgradeCenter = nullptr;
Anim2DCollection* TheAnim2DCollection = nullptr;
GameTextInterface* TheGameText = nullptr;
ImageCollection* TheMappedImageCollection = nullptr;
ParticleSystemManager* TheParticleSystemManager = nullptr;
ArmorStore* TheArmorStore = nullptr;
FXListStore* TheFXListStore = nullptr;
ObjectCreationListStore* TheObjectCreationListStore = nullptr;
WeaponStore* TheWeaponStore = nullptr;

const char* const TheVeterancyNames[] = {
    "REGULAR", "VETERAN", "ELITE", "HEROIC", nullptr
};
static_assert(ARRAY_SIZE(TheVeterancyNames) == LEVEL_COUNT + 1, "Incorrect array size");

void INI::parseGameDataDefinition(INI* ini) {
    GlobalData::parseGameDataDefinition(ini);
}

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

// Exact WeaponBonus / WeaponBonusSet methods from Weapon.cpp:3581-3624
void WeaponBonus::appendBonuses(WeaponBonus& bonus) const {
    for (int f = 0; f < WeaponBonus::FIELD_COUNT; ++f) {
        bonus.m_field[f] += this->m_field[f] - 1.0f;
    }
}

void WeaponBonusSet::parseWeaponBonusSet(INI* ini, void* /*instance*/, void* store, const void* /*userData*/) {
    WeaponBonusSet* self = (WeaponBonusSet*)store;
    self->parseWeaponBonusSet(ini);
}

void WeaponBonusSet::parseWeaponBonusSetPtr(INI* ini, void* /*instance*/, void* store, const void* /*userData*/) {
    WeaponBonusSet** selfPtr = (WeaponBonusSet**)store;
    (*selfPtr)->parseWeaponBonusSet(ini);
}

void WeaponBonusSet::parseWeaponBonusSet(INI* ini) {
    WeaponBonusConditionType wb = (WeaponBonusConditionType)INI::scanIndexList(ini->getNextToken(), TheWeaponBonusNames);
    WeaponBonus::Field wf = (WeaponBonus::Field)INI::scanIndexList(ini->getNextToken(), TheWeaponBonusFieldNames);
    m_bonus[wb].setField(wf, INI::scanPercentToReal(ini->getNextToken()));
}

void WeaponBonusSet::appendBonuses(WeaponBonusConditionFlags flags, WeaponBonus& bonus) const {
    if (flags == 0)
        return;
    for (int i = 0; i < WEAPONBONUSCONDITION_COUNT; ++i) {
        if ((flags & (1u << i)) == 0)
            continue;
        this->m_bonus[i].appendBonuses(bonus);
    }
}

// Exact Money::parseMoneyAmount from Money.cpp:207-214
void Money::parseMoneyAmount(INI* ini, void* instance, void* store, const void* userData) {
    Money* money = (Money*)store;
    UnsignedInt moneyAmount;
    INI::parseUnsignedInt(ini, instance, &moneyAmount, userData);
    money->setStartingCash(moneyAmount);
}

// -----------------------------------------------------------------------------
// SHA-256 helper for vendor source fingerprints
// -----------------------------------------------------------------------------
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

    uint64_t bitlen = static_cast<uint64_t>(data.size()) * 8ULL;
    data.push_back(0x80u);
    while ((data.size() % 64) != 56) data.push_back(0u);
    for (int i = 7; i >= 0; --i) data.push_back(static_cast<uint8_t>((bitlen >> (i * 8)) & 0xffu));

    uint32_t H[8] = {
        0x6a09e667u, 0xbb67ae85u, 0x3c6ef372u, 0xa54ff53au,
        0x510e527fu, 0x9b05688cu, 0x1f83d9abu, 0x5be0cd19u
    };
    auto rotr = [](uint32_t x, uint32_t n) { return (x >> n) | (x << (32 - n)); };
    for (size_t off = 0; off < data.size(); off += 64) {
        uint32_t W[64];
        for (int i = 0; i < 16; ++i) {
            W[i] = (static_cast<uint32_t>(data[off + i * 4]) << 24)
                 | (static_cast<uint32_t>(data[off + i * 4 + 1]) << 16)
                 | (static_cast<uint32_t>(data[off + i * 4 + 2]) << 8)
                 | static_cast<uint32_t>(data[off + i * 4 + 3]);
        }
        for (int i = 16; i < 64; ++i) {
            uint32_t s0 = rotr(W[i - 15], 7) ^ rotr(W[i - 15], 18) ^ (W[i - 15] >> 3);
            uint32_t s1 = rotr(W[i - 2], 17) ^ rotr(W[i - 2], 19) ^ (W[i - 2] >> 10);
            W[i] = W[i - 16] + s0 + W[i - 7] + s1;
        }
        uint32_t a = H[0], b = H[1], c = H[2], d = H[3], e = H[4], f2 = H[5], g = H[6], h = H[7];
        for (int i = 0; i < 64; ++i) {
            uint32_t S1 = rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25);
            uint32_t ch = (e & f2) ^ ((~e) & g);
            uint32_t t1 = h + S1 + ch + K[i] + W[i];
            uint32_t S0 = rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22);
            uint32_t maj = (a & b) ^ (a & c) ^ (b & c);
            uint32_t t2 = S0 + maj;
            h = g; g = f2; f2 = e; e = d + t1; d = c; c = b; b = a; a = t1 + t2;
        }
        H[0] += a; H[1] += b; H[2] += c; H[3] += d;
        H[4] += e; H[5] += f2; H[6] += g; H[7] += h;
    }
    char hex[65];
    for (int i = 0; i < 8; ++i) std::snprintf(hex + i * 8, 9, "%08x", H[i]);
    hex[64] = '\0';
    return std::string(hex);
}

// -----------------------------------------------------------------------------
// Deterministic PRNG & State Hashers (mirrored bit-for-bit in Rust)
// -----------------------------------------------------------------------------
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

static uint32_t crc_str_bytes(const std::string& s) {
    CRC c;
    c.computeCRC(s.data(), static_cast<Int>(s.size()));
    return c.get();
}

struct Hasher {
    CRC c;
    void u8(uint8_t v) { c.computeCRC(&v, 1); }
    void i16(Short v) {
        uint16_t u = static_cast<uint16_t>(v);
        uint8_t buf[2] = { static_cast<uint8_t>(u & 0xffu), static_cast<uint8_t>((u >> 8) & 0xffu) };
        c.computeCRC(buf, 2);
    }
    void u16(UnsignedShort v) {
        uint8_t buf[2] = { static_cast<uint8_t>(v & 0xffu), static_cast<uint8_t>((v >> 8) & 0xffu) };
        c.computeCRC(buf, 2);
    }
    void i32(Int v) { u32(static_cast<uint32_t>(v)); }
    void u32(uint32_t v) {
        uint8_t buf[4] = {
            static_cast<uint8_t>(v & 0xffu),
            static_cast<uint8_t>((v >> 8) & 0xffu),
            static_cast<uint8_t>((v >> 16) & 0xffu),
            static_cast<uint8_t>((v >> 24) & 0xffu)
        };
        c.computeCRC(buf, 4);
    }
    void f32(Real v) { u32(fbits(v)); }
    void coord2d(const Coord2D& v) { f32(v.x); f32(v.y); }
    void coord3d(const Coord3D& v) { f32(v.x); f32(v.y); f32(v.z); }
    void rgb(const RGBColor& v) { f32(v.red); f32(v.green); f32(v.blue); }
    void ascii(const AsciiString& s) {
        uint32_t len = static_cast<uint32_t>(s.getLength());
        u32(len);
        if (len > 0) c.computeCRC(s.str(), static_cast<Int>(len));
    }
    uint32_t finish() const { return c.get(); }
};

static UnsignedInt hash_global_data(const GlobalData& g) {
    Hasher h;
    h.ascii(g.m_mapName);
    h.ascii(g.m_moveHintName);
    h.u8(g.m_useTrees);
    h.u8(g.m_useTreeSway);
    h.u8(g.m_useDrawModuleLOD);
    h.u8(g.m_useHeatEffects);
    h.u8(g.m_useFpsLimit);
    h.u8(g.m_dumpAssetUsage);
    h.i32(g.m_framesPerSecondLimit);
    h.i32(g.m_skirmishTickRate);
    h.i32(g.m_chipSetType);
    h.u8(g.m_headless);
    h.u8(g.m_checkForUpdates);
    h.u8(g.m_windowed);
    h.i32(g.m_xResolution);
    h.i32(g.m_yResolution);
    h.i32(g.m_maxShellScreens);
    h.u8(g.m_useCloudMap);
    h.i32(g.m_use3WayTerrainBlends);
    h.u8(g.m_useLightMap);
    h.u8(g.m_bilinearTerrainTex);
    h.u8(g.m_trilinearTerrainTex);
    h.u8(g.m_multiPassTerrain);
    h.u8(g.m_adjustCliffTextures);
    h.u8(g.m_stretchTerrain);
    h.u8(g.m_useHalfHeightMap);
    h.u8(g.m_drawEntireTerrain);
    h.i32(static_cast<Int>(g.m_terrainLOD));
    h.u8(g.m_enableDynamicLOD);
    h.u8(g.m_enableStaticLOD);
    h.i32(g.m_terrainLODTargetTimeMS);
    h.u8(g.m_useAlternateMouse);
    h.u8(g.m_useRightMouseScrollWithAlternateMouse);
    h.u8(g.m_clientRetaliationModeEnabled);
    h.u8(g.m_doubleClickAttackMove);
    h.u8(g.m_rightMouseAlwaysScrolls);
    h.i32(g.m_jpegQuality);
    h.u8(g.m_useWaterPlane);
    h.u8(g.m_useCloudPlane);
    h.u8(g.m_useShadowVolumes);
    h.u8(g.m_useShadowVolumesForSkins);
    h.u8(g.m_shadowsForProjectiles);
    h.u8(g.m_shadowsForProps);
    h.u8(g.m_advancedShadows);
    h.u8(g.m_advancedLighting);
    h.u8(g.m_extendedShadowFrustumCulling);
    h.u8(g.m_naturalSunElevation);
    h.u8(g.m_bridgeShadows);
    h.u8(g.m_waterReflections);
    h.u8(g.m_explosionDynamicLights);
    h.u8(g.m_vehicleHeadlights);
    h.u8(g.m_enablePostProcessing);
    h.u8(g.m_postProcessBloom);
    h.u8(g.m_postProcessHDR);
    h.u8(g.m_postProcessFXAA);
    h.u8(g.m_airUnitGroundIndicator);
    h.u8(g.m_useShadowDecals);
    h.i32(g.m_textureReductionFactor);
    h.u8(g.m_enableBehindBuildingMarkers);
    h.f32(g.m_waterPositionX);
    h.f32(g.m_waterPositionY);
    h.f32(g.m_waterPositionZ);
    h.f32(g.m_waterExtentX);
    h.f32(g.m_waterExtentY);
    h.i32(g.m_waterType);
    h.u8(g.m_showSoftWaterEdge);
    h.u8(g.m_usingWaterTrackEditor);
    h.u8(g.m_isWorldBuilder);
    h.i32(g.m_featherWater);

    for (Int i = 0; i < GlobalData::MAX_WATER_GRID_SETTINGS; ++i) {
        h.ascii(g.m_vertexWaterAvailableMaps[i]);
        h.f32(g.m_vertexWaterHeightClampLow[i]);
        h.f32(g.m_vertexWaterHeightClampHi[i]);
        h.f32(g.m_vertexWaterAngle[i]);
        h.f32(g.m_vertexWaterXPosition[i]);
        h.f32(g.m_vertexWaterYPosition[i]);
        h.f32(g.m_vertexWaterZPosition[i]);
        h.i32(g.m_vertexWaterXGridCells[i]);
        h.i32(g.m_vertexWaterYGridCells[i]);
        h.f32(g.m_vertexWaterGridSize[i]);
        h.f32(g.m_vertexWaterAttenuationA[i]);
        h.f32(g.m_vertexWaterAttenuationB[i]);
        h.f32(g.m_vertexWaterAttenuationC[i]);
        h.f32(g.m_vertexWaterAttenuationRange[i]);
    }

    h.f32(g.m_downwindAngle);
    h.f32(g.m_skyBoxPositionZ);
    h.f32(g.m_drawSkyBox);
    h.f32(g.m_skyBoxScale);
    h.f32(g.m_viewportHeightScale);
    h.f32(g.m_cameraPitch);
    h.f32(g.m_cameraYaw);
    h.f32(g.m_cameraHeight);
    h.f32(g.m_maxCameraHeight);
    h.f32(g.m_minCameraHeight);
    h.f32(g.m_terrainDrawDistanceScale);
    h.f32(g.m_terrainHeightAtEdgeOfMap);
    h.f32(g.m_unitDamagedThresh);
    h.f32(g.m_unitReallyDamagedThresh);
    h.f32(g.m_groundStiffness);
    h.f32(g.m_structureStiffness);
    h.f32(g.m_gravity);
    h.f32(g.m_stealthFriendlyOpacity);
    h.u32(g.m_defaultOcclusionDelay);
    h.u8(g.m_preloadAssets);
    h.u8(g.m_preloadEverything);
    h.u8(g.m_preloadReport);
    h.f32(g.m_partitionCellSize);
    h.coord3d(g.m_ammoPipWorldOffset);
    h.coord3d(g.m_containerPipWorldOffset);
    h.coord2d(g.m_ammoPipScreenOffset);
    h.coord2d(g.m_containerPipScreenOffset);
    h.f32(g.m_ammoPipScaleFactor);
    h.f32(g.m_containerPipScaleFactor);
    h.u32(g.m_historicDamageLimit);
    h.i32(g.m_maxTerrainTracks);
    h.i32(g.m_maxTankTrackEdges);
    h.i32(g.m_maxTankTrackOpaqueEdges);
    h.i32(g.m_maxTankTrackFadeDelay);
    h.ascii(g.m_levelGainAnimationName);
    h.f32(g.m_levelGainAnimationDisplayTimeInSeconds);
    h.f32(g.m_levelGainAnimationZRisePerSecond);
    h.ascii(g.m_getHealedAnimationName);
    h.f32(g.m_getHealedAnimationDisplayTimeInSeconds);
    h.f32(g.m_getHealedAnimationZRisePerSecond);
    h.i32(static_cast<Int>(g.m_timeOfDay));
    h.i32(static_cast<Int>(g.m_weather));
    h.u8(g.m_makeTrackMarks);
    h.u8(g.m_hideGarrisonFlags);
    h.u8(g.m_forceModelsToFollowTimeOfDay);
    h.u8(g.m_forceModelsToFollowWeather);

    for (Int t = 0; t < TIME_OF_DAY_COUNT; ++t) {
        for (Int l = 0; l < MAX_GLOBAL_LIGHTS; ++l) {
            h.rgb(g.m_terrainLighting[t][l].ambient);
            h.rgb(g.m_terrainLighting[t][l].diffuse);
            h.coord3d(g.m_terrainLighting[t][l].lightPos);
            h.rgb(g.m_terrainObjectsLighting[t][l].ambient);
            h.rgb(g.m_terrainObjectsLighting[t][l].diffuse);
            h.coord3d(g.m_terrainObjectsLighting[t][l].lightPos);
        }
    }

    for (Int l = 0; l < MAX_GLOBAL_LIGHTS; ++l) {
        h.rgb(g.m_terrainAmbient[l]);
        h.rgb(g.m_terrainDiffuse[l]);
        h.coord3d(g.m_terrainLightPos[l]);
    }

    // TIME_OF_DAY_INVALID (0) is not initialized by GlobalData::GlobalData() in C++.
    for (Int t = TIME_OF_DAY_FIRST; t < TIME_OF_DAY_COUNT; ++t) {
        h.f32(g.m_infantryLightScale[t]);
    }
    h.f32(g.m_scriptOverrideInfantryLightScale);

    for (Int p = 0; p < PLAYERTYPE_COUNT; ++p) {
        for (Int d = 0; d < DIFFICULTY_COUNT; ++d) {
            h.f32(g.m_soloPlayerHealthBonusForDifficulty[p][d]);
        }
    }

    h.i32(g.m_maxVisibleTranslucentObjects);
    h.i32(g.m_maxVisibleOccluderObjects);
    h.i32(g.m_maxVisibleOccludeeObjects);
    h.i32(g.m_maxVisibleNonOccluderOrOccludeeObjects);
    h.f32(g.m_occludedLuminanceScale);
    h.i32(g.m_numGlobalLights);
    h.i32(g.m_maxRoadSegments);
    h.i32(g.m_maxRoadVertex);
    h.i32(g.m_maxRoadIndex);
    h.i32(g.m_maxRoadTypes);
    h.u8(g.m_audioOn);
    h.u8(g.m_musicOn);
    h.u8(g.m_soundsOn);
    h.u8(g.m_sounds3DOn);
    h.u8(g.m_speechOn);
    h.u8(g.m_videoOn);
    h.u8(g.m_disableCameraMovement);
    h.u8(g.m_useFX);
    h.u8(g.m_showClientPhysics);
    h.u8(g.m_showTerrainNormals);
    h.u32(g.m_noDraw);
    h.i32(static_cast<Int>(g.m_debugAI));
    h.u8(g.m_debugSupplyCenterPlacement);
    h.u8(g.m_debugAIObstacles);
    h.u8(g.m_showObjectHealth);
    h.u8(g.m_scriptDebug);
    h.u8(g.m_particleEdit);
    h.u8(g.m_displayDebug);
    h.u8(g.m_winCursors);
    h.u8(g.m_constantDebugUpdate);
    h.u8(g.m_showTeamDot);
    h.u8(g.m_forceBenchmark);
    h.i32(g.m_fixedSeed);
    h.f32(g.m_particleScale);
    h.ascii(g.m_autoFireParticleSmallPrefix);
    h.ascii(g.m_autoFireParticleSmallSystem);
    h.i32(g.m_autoFireParticleSmallMax);
    h.ascii(g.m_autoFireParticleMediumPrefix);
    h.ascii(g.m_autoFireParticleMediumSystem);
    h.i32(g.m_autoFireParticleMediumMax);
    h.ascii(g.m_autoFireParticleLargePrefix);
    h.ascii(g.m_autoFireParticleLargeSystem);
    h.i32(g.m_autoFireParticleLargeMax);
    h.ascii(g.m_autoSmokeParticleSmallPrefix);
    h.ascii(g.m_autoSmokeParticleSmallSystem);
    h.i32(g.m_autoSmokeParticleSmallMax);
    h.ascii(g.m_autoSmokeParticleMediumPrefix);
    h.ascii(g.m_autoSmokeParticleMediumSystem);
    h.i32(g.m_autoSmokeParticleMediumMax);
    h.ascii(g.m_autoSmokeParticleLargePrefix);
    h.ascii(g.m_autoSmokeParticleLargeSystem);
    h.i32(g.m_autoSmokeParticleLargeMax);
    h.ascii(g.m_autoAflameParticlePrefix);
    h.ascii(g.m_autoAflameParticleSystem);
    h.i32(g.m_autoAflameParticleMax);
    h.i32(g.m_netMinPlayers);
    h.u32(g.m_defaultIP);
    h.u32(g.m_firewallBehavior);
    h.u32(g.m_firewallPortOverride);
    h.i16(g.m_firewallPortAllocationDelta);
    h.i32(g.m_baseValuePerSupplyBox);
    h.f32(g.m_BuildSpeed);
    h.f32(g.m_MinDistFromEdgeOfMapForBuild);
    h.f32(g.m_SupplyBuildBorder);
    h.f32(g.m_allowedHeightVariationForBuilding);
    h.f32(g.m_MinLowEnergyProductionSpeed);
    h.f32(g.m_MaxLowEnergyProductionSpeed);
    h.f32(g.m_LowEnergyPenaltyModifier);
    h.f32(g.m_MultipleFactory);
    h.f32(g.m_RefundPercent);
    h.f32(g.m_commandCenterHealRange);
    h.f32(g.m_commandCenterHealAmount);
    h.i32(g.m_maxLineBuildObjects);
    h.i32(g.m_maxTunnelCapacity);
    h.f32(g.m_horizontalScrollSpeedFactor);
    h.f32(g.m_verticalScrollSpeedFactor);
    h.f32(g.m_scrollAmountCutoff);
    h.f32(g.m_cameraAdjustSpeed);
    h.u8(g.m_enforceMaxCameraHeight);
    h.u8(g.m_buildMapCache);
    h.ascii(g.m_initialFile);
    h.ascii(g.m_pendingFile);
    h.ascii(g.m_loadSaveGame);
    h.ascii(g.m_loadReplayGame);
    h.u32(static_cast<uint32_t>(g.m_simulateReplays.size()));
    h.i32(g.m_simulateReplayJobs);
    h.i32(g.m_maxParticleCount);
    h.i32(g.m_maxFieldParticleCount);

    h.u8(g.m_weaponBonusSet ? 1 : 0);
    if (g.m_weaponBonusSet) {
        for (Int c = 0; c < WEAPONBONUSCONDITION_COUNT; ++c) {
            const WeaponBonus& wb = g.m_weaponBonusSet->getBonusEntry(c);
            for (Int f = 0; f < WeaponBonus::FIELD_COUNT; ++f) {
                h.f32(wb.getField(static_cast<WeaponBonus::Field>(f)));
            }
        }
    }

    for (Int i = LEVEL_FIRST; i <= LEVEL_LAST; ++i) {
        h.f32(g.m_healthBonus[i]);
    }
    h.f32(g.m_defaultStructureRubbleHeight);
    h.ascii(g.m_shellMapName);
    h.u8(g.m_shellMapOn);
    h.u8(g.m_playIntro);
    h.u8(g.m_playSizzle);
    h.u8(g.m_loadScreenRender);
    h.f32(g.m_keyboardScrollFactor);
    h.f32(g.m_mouseScrollFactor);
    h.f32(g.m_keyboardDefaultScrollFactor);
    h.u8(g.m_drawScrollAnchor);
    h.u8(g.m_moveScrollAnchor);
    h.u8(g.m_animateWindows);
    h.u8(g.m_incrementalAGPBuf);
    h.u32(g.m_iniCRC);
    h.u32(g.m_exeCRC);
    h.i32(static_cast<Int>(g.m_movementPenaltyDamageState));
    h.i32(g.m_groupSelectMinSelectSize);
    h.f32(g.m_groupSelectVolumeBase);
    h.f32(g.m_groupSelectVolumeIncrement);
    h.i32(g.m_maxUnitSelectSounds);
    h.f32(g.m_selectionFlashSaturationFactor);
    h.u8(g.m_selectionFlashHouseColor);
    h.f32(g.m_cameraAudibleRadius);
    h.f32(g.m_groupMoveClickToGatherFactor);
    h.u32(g.m_antiAliasLevel);
    h.u32(g.m_textureFilteringMode);
    h.u32(g.m_textureAnisotropyLevel);
    h.u8(g.m_languageFilterPref);
    h.u8(g.m_loadScreenDemo);
    h.u8(g.m_disableRender);
    h.u8(g.m_saveCameraInReplay);
    h.u8(g.m_useCameraInReplay);
    h.u8(g.m_enablePlayerObserver);
    h.i32(g.m_networkLatencyFontSize);
    h.i32(g.m_renderFpsFontSize);
    h.i32(g.m_systemTimeFontSize);
    h.i32(g.m_gameTimeFontSize);
    h.i32(g.m_playerInfoListFontSize);
    h.u8(g.m_showMoneyPerMinute);
    h.u8(g.m_allowMoneyPerMinuteForPlayer);
    h.f32(g.m_gameWindowTransitionSpeedMultiplier);
    h.f32(g.m_shakeSubtleIntensity);
    h.f32(g.m_shakeNormalIntensity);
    h.f32(g.m_shakeStrongIntensity);
    h.f32(g.m_shakeSevereIntensity);
    h.f32(g.m_shakeCineExtremeIntensity);
    h.f32(g.m_shakeCineInsaneIntensity);
    h.f32(g.m_maxShakeIntensity);
    h.f32(g.m_maxShakeRange);
    h.f32(g.m_sellPercentage);
    h.f32(g.m_baseRegenHealthPercentPerSecond);
    h.u32(g.m_baseRegenDelay);
    h.u32(g.m_hotKeyTextColor);
    h.ascii(g.m_specialPowerViewObjectName);
    h.f32(g.m_objectPlacementOpacity);
    h.u8(g.m_objectPlacementShadows);
    h.u32(static_cast<uint32_t>(g.m_standardPublicBones.size()));
    for (const AsciiString& b : g.m_standardPublicBones) {
        h.ascii(b);
    }
    h.f32(g.m_standardMinefieldDensity);
    h.f32(g.m_standardMinefieldDistance);
    h.u8(g.m_showMetrics);
    h.u32(g.m_defaultStartingCash.countMoney());
    h.u8(g.m_debugShowGraphicalFramerate);
    h.i32(g.m_powerBarBase);
    h.f32(g.m_powerBarIntervals);
    h.i32(g.m_powerBarYellowRange);
    h.f32(g.m_displayGamma);
    h.u32(g.m_unlookPersistDuration);
    h.u8(g.m_shouldUpdateTGAToDDS);
    h.u32(g.m_doubleClickTimeMS);
    h.rgb(g.m_shroudColor);
    h.u8(g.m_clearAlpha);
    h.u8(g.m_fogAlpha);
    h.u8(g.m_shroudAlpha);
    h.u32(g.m_networkFPSHistoryLength);
    h.u32(g.m_networkLatencyHistoryLength);
    h.u32(g.m_networkCushionHistoryLength);
    h.u32(g.m_networkRunAheadMetricsTime);
    h.u32(g.m_networkKeepAliveDelay);
    h.u32(g.m_networkRunAheadSlack);
    h.u32(g.m_networkDisconnectTime);
    h.u32(g.m_networkPlayerTimeoutTime);
    h.u32(g.m_networkDisconnectScreenNotifyTime);
    h.f32(g.m_keyboardCameraRotateSpeed);
    h.i32(g.m_playStats);
    h.u8(g.m_TiVOFastMode);
    h.u8(g.m_shroudOn);
    h.u8(g.m_isBreakableMovie);
    h.u8(g.m_breakTheMovie);
    h.ascii(g.m_modDir);
    h.ascii(g.m_modBIG);
    h.ascii(g.m_assetPath);
    h.ascii(g.m_generalsAssetPath);
    h.u8(g.m_looseFilesWin);
    return h.finish();
}

static uint32_t compute_getters_mask(const GlobalData& g) {
    uint32_t m = 0;
    if (g.isUseShadowVolumesForSkins()) m |= (1u << 0);
    if (g.isShadowsForProjectiles()) m |= (1u << 1);
    if (g.isShadowsForProps()) m |= (1u << 2);
    if (g.isExtendedShadowFrustumCulling()) m |= (1u << 3);
    if (g.isNaturalSunElevation()) m |= (1u << 4);
    if (g.isBridgeShadows()) m |= (1u << 5);
    if (g.isWaterReflections()) m |= (1u << 6);
    if (g.isExplosionDynamicLights()) m |= (1u << 7);
    if (g.isVehicleHeadlights()) m |= (1u << 8);
    if (g.isEnablePostProcessing()) m |= (1u << 9);
    if (g.isPostProcessBloom()) m |= (1u << 10);
    if (g.isPostProcessHDR()) m |= (1u << 11);
    if (g.isPostProcessFXAA()) m |= (1u << 12);
    if (g.isAirUnitGroundIndicator()) m |= (1u << 13);
    return m;
}

// -----------------------------------------------------------------------------
// Field Specification Table (all 360 Release entries of s_GlobalDataFieldParseTable)
// -----------------------------------------------------------------------------
enum FieldKind {
    K_BOOL = 0,
    K_INT,
    K_U8,
    K_REAL,
    K_ANGLE_REAL,
    K_ACCEL_REAL,
    K_PERCENT,
    K_DURATION_UINT,
    K_ASCII,
    K_ASCII_VEC_APPEND,
    K_RGB,
    K_COLOR_INT,
    K_COORD2D,
    K_COORD3D,
    K_TERRAIN_LOD,
    K_TIME_OF_DAY,
    K_WEATHER,
    K_BODY_DAMAGE,
    K_MONEY,
    K_WEAPON_BONUS,
    K_SHORT_INT
};

struct FieldSpec {
    const char* name;
    FieldKind kind;
};

static const FieldSpec kReleaseFields[] = {
    { "Windowed", K_BOOL },
    { "CheckForUpdates", K_BOOL },
    { "XResolution", K_INT },
    { "YResolution", K_INT },
    { "MapName", K_ASCII },
    { "MoveHintName", K_ASCII },
    { "UseTrees", K_BOOL },
    { "UseFPSLimit", K_BOOL },
    { "DumpAssetUsage", K_BOOL },
    { "FramesPerSecondLimit", K_INT },
    { "SkirmishTickRate", K_INT },
    { "TickRate", K_INT },
    { "ChipsetType", K_INT },
    { "MaxShellScreens", K_INT },
    { "UseCloudMap", K_BOOL },
    { "UseLightMap", K_BOOL },
    { "BilinearTerrainTex", K_BOOL },
    { "TrilinearTerrainTex", K_BOOL },
    { "MultiPassTerrain", K_BOOL },
    { "AdjustCliffTextures", K_BOOL },
    { "Use3WayTerrainBlends", K_INT },
    { "StretchTerrain", K_BOOL },
    { "UseHalfHeightMap", K_BOOL },
    { "DrawEntireTerrain", K_BOOL },
    { "TerrainLOD", K_TERRAIN_LOD },
    { "TerrainLODTargetTimeMS", K_INT },
    { "RightMouseAlwaysScrolls", K_BOOL },
    { "UseWaterPlane", K_BOOL },
    { "UseCloudPlane", K_BOOL },
    { "DownwindAngle", K_REAL },
    { "UseShadowVolumes", K_BOOL },
    { "UseShadowVolumesForSkins", K_BOOL },
    { "ShadowsForProjectiles", K_BOOL },
    { "ShadowsForProps", K_BOOL },
    { "AdvancedShadows", K_BOOL },
    { "AdvancedLighting", K_BOOL },
    { "ExtendedShadowFrustumCulling", K_BOOL },
    { "NaturalSunElevation", K_BOOL },
    { "BridgeShadows", K_BOOL },
    { "WaterReflections", K_BOOL },
    { "ExplosionDynamicLights", K_BOOL },
    { "VehicleHeadlights", K_BOOL },
    { "EnablePostProcessing", K_BOOL },
    { "PostProcessBloom", K_BOOL },
    { "PostProcessHDR", K_BOOL },
    { "PostProcessFXAA", K_BOOL },
    { "AirUnitGroundIndicator", K_BOOL },
    { "UseShadowDecals", K_BOOL },
    { "TextureReductionFactor", K_INT },
    { "UseBehindBuildingMarker", K_BOOL },
    { "WaterPositionX", K_REAL },
    { "WaterPositionY", K_REAL },
    { "WaterPositionZ", K_REAL },
    { "WaterExtentX", K_REAL },
    { "WaterExtentY", K_REAL },
    { "WaterType", K_INT },
    { "FeatherWater", K_INT },
    { "ShowSoftWaterEdge", K_BOOL },
    { "VertexWaterAvailableMaps1", K_ASCII },
    { "VertexWaterHeightClampLow1", K_REAL },
    { "VertexWaterHeightClampHi1", K_REAL },
    { "VertexWaterAngle1", K_ANGLE_REAL },
    { "VertexWaterXPosition1", K_REAL },
    { "VertexWaterYPosition1", K_REAL },
    { "VertexWaterZPosition1", K_REAL },
    { "VertexWaterXGridCells1", K_INT },
    { "VertexWaterYGridCells1", K_INT },
    { "VertexWaterGridSize1", K_REAL },
    { "VertexWaterAttenuationA1", K_REAL },
    { "VertexWaterAttenuationB1", K_REAL },
    { "VertexWaterAttenuationC1", K_REAL },
    { "VertexWaterAttenuationRange1", K_REAL },
    { "VertexWaterAvailableMaps2", K_ASCII },
    { "VertexWaterHeightClampLow2", K_REAL },
    { "VertexWaterHeightClampHi2", K_REAL },
    { "VertexWaterAngle2", K_ANGLE_REAL },
    { "VertexWaterXPosition2", K_REAL },
    { "VertexWaterYPosition2", K_REAL },
    { "VertexWaterZPosition2", K_REAL },
    { "VertexWaterXGridCells2", K_INT },
    { "VertexWaterYGridCells2", K_INT },
    { "VertexWaterGridSize2", K_REAL },
    { "VertexWaterAttenuationA2", K_REAL },
    { "VertexWaterAttenuationB2", K_REAL },
    { "VertexWaterAttenuationC2", K_REAL },
    { "VertexWaterAttenuationRange2", K_REAL },
    { "VertexWaterAvailableMaps3", K_ASCII },
    { "VertexWaterHeightClampLow3", K_REAL },
    { "VertexWaterHeightClampHi3", K_REAL },
    { "VertexWaterAngle3", K_ANGLE_REAL },
    { "VertexWaterXPosition3", K_REAL },
    { "VertexWaterYPosition3", K_REAL },
    { "VertexWaterZPosition3", K_REAL },
    { "VertexWaterXGridCells3", K_INT },
    { "VertexWaterYGridCells3", K_INT },
    { "VertexWaterGridSize3", K_REAL },
    { "VertexWaterAttenuationA3", K_REAL },
    { "VertexWaterAttenuationB3", K_REAL },
    { "VertexWaterAttenuationC3", K_REAL },
    { "VertexWaterAttenuationRange3", K_REAL },
    { "VertexWaterAvailableMaps4", K_ASCII },
    { "VertexWaterHeightClampLow4", K_REAL },
    { "VertexWaterHeightClampHi4", K_REAL },
    { "VertexWaterAngle4", K_ANGLE_REAL },
    { "VertexWaterXPosition4", K_REAL },
    { "VertexWaterYPosition4", K_REAL },
    { "VertexWaterZPosition4", K_REAL },
    { "VertexWaterXGridCells4", K_INT },
    { "VertexWaterYGridCells4", K_INT },
    { "VertexWaterGridSize4", K_REAL },
    { "VertexWaterAttenuationA4", K_REAL },
    { "VertexWaterAttenuationB4", K_REAL },
    { "VertexWaterAttenuationC4", K_REAL },
    { "VertexWaterAttenuationRange4", K_REAL },
    { "SkyBoxPositionZ", K_REAL },
    { "SkyBoxScale", K_REAL },
    { "DrawSkyBox", K_BOOL },
    { "ViewportHeightScale", K_REAL },
    { "CameraPitch", K_REAL },
    { "CameraYaw", K_REAL },
    { "CameraHeight", K_REAL },
    { "MaxCameraHeight", K_REAL },
    { "MinCameraHeight", K_REAL },
    { "TerrainDrawDistanceScale", K_REAL },
    { "TerrainHeightAtEdgeOfMap", K_REAL },
    { "UnitDamagedThreshold", K_REAL },
    { "UnitReallyDamagedThreshold", K_REAL },
    { "GroundStiffness", K_REAL },
    { "StructureStiffness", K_REAL },
    { "Gravity", K_ACCEL_REAL },
    { "StealthFriendlyOpacity", K_PERCENT },
    { "DefaultOcclusionDelay", K_DURATION_UINT },
    { "PartitionCellSize", K_REAL },
    { "AmmoPipScaleFactor", K_REAL },
    { "ContainerPipScaleFactor", K_REAL },
    { "AmmoPipWorldOffset", K_COORD3D },
    { "ContainerPipWorldOffset", K_COORD3D },
    { "AmmoPipScreenOffset", K_COORD2D },
    { "ContainerPipScreenOffset", K_COORD2D },
    { "HistoricDamageLimit", K_DURATION_UINT },
    { "MaxTerrainTracks", K_INT },
    { "TimeOfDay", K_TIME_OF_DAY },
    { "Weather", K_WEATHER },
    { "MakeTrackMarks", K_BOOL },
    { "HideGarrisonFlags", K_BOOL },
    { "ForceModelsToFollowTimeOfDay", K_BOOL },
    { "ForceModelsToFollowWeather", K_BOOL },
    { "LevelGainAnimationName", K_ASCII },
    { "LevelGainAnimationTime", K_REAL },
    { "LevelGainAnimationZRise", K_REAL },
    { "GetHealedAnimationName", K_ASCII },
    { "GetHealedAnimationTime", K_REAL },
    { "GetHealedAnimationZRise", K_REAL },
    { "TerrainLightingMorningAmbient", K_RGB },
    { "TerrainLightingMorningDiffuse", K_RGB },
    { "TerrainLightingMorningLightPos", K_COORD3D },
    { "TerrainLightingAfternoonAmbient", K_RGB },
    { "TerrainLightingAfternoonDiffuse", K_RGB },
    { "TerrainLightingAfternoonLightPos", K_COORD3D },
    { "TerrainLightingEveningAmbient", K_RGB },
    { "TerrainLightingEveningDiffuse", K_RGB },
    { "TerrainLightingEveningLightPos", K_COORD3D },
    { "TerrainLightingNightAmbient", K_RGB },
    { "TerrainLightingNightDiffuse", K_RGB },
    { "TerrainLightingNightLightPos", K_COORD3D },
    { "TerrainObjectsLightingMorningAmbient", K_RGB },
    { "TerrainObjectsLightingMorningDiffuse", K_RGB },
    { "TerrainObjectsLightingMorningLightPos", K_COORD3D },
    { "TerrainObjectsLightingAfternoonAmbient", K_RGB },
    { "TerrainObjectsLightingAfternoonDiffuse", K_RGB },
    { "TerrainObjectsLightingAfternoonLightPos", K_COORD3D },
    { "TerrainObjectsLightingEveningAmbient", K_RGB },
    { "TerrainObjectsLightingEveningDiffuse", K_RGB },
    { "TerrainObjectsLightingEveningLightPos", K_COORD3D },
    { "TerrainObjectsLightingNightAmbient", K_RGB },
    { "TerrainObjectsLightingNightDiffuse", K_RGB },
    { "TerrainObjectsLightingNightLightPos", K_COORD3D },
    { "TerrainLightingMorningAmbient2", K_RGB },
    { "TerrainLightingMorningDiffuse2", K_RGB },
    { "TerrainLightingMorningLightPos2", K_COORD3D },
    { "TerrainLightingAfternoonAmbient2", K_RGB },
    { "TerrainLightingAfternoonDiffuse2", K_RGB },
    { "TerrainLightingAfternoonLightPos2", K_COORD3D },
    { "TerrainLightingEveningAmbient2", K_RGB },
    { "TerrainLightingEveningDiffuse2", K_RGB },
    { "TerrainLightingEveningLightPos2", K_COORD3D },
    { "TerrainLightingNightAmbient2", K_RGB },
    { "TerrainLightingNightDiffuse2", K_RGB },
    { "TerrainLightingNightLightPos2", K_COORD3D },
    { "TerrainObjectsLightingMorningAmbient2", K_RGB },
    { "TerrainObjectsLightingMorningDiffuse2", K_RGB },
    { "TerrainObjectsLightingMorningLightPos2", K_COORD3D },
    { "TerrainObjectsLightingAfternoonAmbient2", K_RGB },
    { "TerrainObjectsLightingAfternoonDiffuse2", K_RGB },
    { "TerrainObjectsLightingAfternoonLightPos2", K_COORD3D },
    { "TerrainObjectsLightingEveningAmbient2", K_RGB },
    { "TerrainObjectsLightingEveningDiffuse2", K_RGB },
    { "TerrainObjectsLightingEveningLightPos2", K_COORD3D },
    { "TerrainObjectsLightingNightAmbient2", K_RGB },
    { "TerrainObjectsLightingNightDiffuse2", K_RGB },
    { "TerrainObjectsLightingNightLightPos2", K_COORD3D },
    { "TerrainLightingMorningAmbient3", K_RGB },
    { "TerrainLightingMorningDiffuse3", K_RGB },
    { "TerrainLightingMorningLightPos3", K_COORD3D },
    { "TerrainLightingAfternoonAmbient3", K_RGB },
    { "TerrainLightingAfternoonDiffuse3", K_RGB },
    { "TerrainLightingAfternoonLightPos3", K_COORD3D },
    { "TerrainLightingEveningAmbient3", K_RGB },
    { "TerrainLightingEveningDiffuse3", K_RGB },
    { "TerrainLightingEveningLightPos3", K_COORD3D },
    { "TerrainLightingNightAmbient3", K_RGB },
    { "TerrainLightingNightDiffuse3", K_RGB },
    { "TerrainLightingNightLightPos3", K_COORD3D },
    { "TerrainObjectsLightingMorningAmbient3", K_RGB },
    { "TerrainObjectsLightingMorningDiffuse3", K_RGB },
    { "TerrainObjectsLightingMorningLightPos3", K_COORD3D },
    { "TerrainObjectsLightingAfternoonAmbient3", K_RGB },
    { "TerrainObjectsLightingAfternoonDiffuse3", K_RGB },
    { "TerrainObjectsLightingAfternoonLightPos3", K_COORD3D },
    { "TerrainObjectsLightingEveningAmbient3", K_RGB },
    { "TerrainObjectsLightingEveningDiffuse3", K_RGB },
    { "TerrainObjectsLightingEveningLightPos3", K_COORD3D },
    { "TerrainObjectsLightingNightAmbient3", K_RGB },
    { "TerrainObjectsLightingNightDiffuse3", K_RGB },
    { "TerrainObjectsLightingNightLightPos3", K_COORD3D },
    { "NumberGlobalLights", K_INT },
    { "InfantryLightMorningScale", K_REAL },
    { "InfantryLightAfternoonScale", K_REAL },
    { "InfantryLightEveningScale", K_REAL },
    { "InfantryLightNightScale", K_REAL },
    { "MaxTranslucentObjects", K_INT },
    { "OccludedColorLuminanceScale", K_REAL },
    { "MaxRoadSegments", K_INT },
    { "MaxRoadVertex", K_INT },
    { "MaxRoadIndex", K_INT },
    { "MaxRoadTypes", K_INT },
    { "ValuePerSupplyBox", K_INT },
    { "AudioOn", K_BOOL },
    { "MusicOn", K_BOOL },
    { "SoundsOn", K_BOOL },
    { "Sounds3DOn", K_BOOL },
    { "SpeechOn", K_BOOL },
    { "VideoOn", K_BOOL },
    { "DisableCameraMovements", K_BOOL },
    { "DebugAI", K_BOOL },
    { "DebugAIObstacles", K_BOOL },
    { "ShowClientPhysics", K_BOOL },
    { "ShowTerrainNormals", K_BOOL },
    { "ShowObjectHealth", K_BOOL },
    { "ParticleScale", K_REAL },
    { "AutoFireParticleSmallPrefix", K_ASCII },
    { "AutoFireParticleSmallSystem", K_ASCII },
    { "AutoFireParticleSmallMax", K_INT },
    { "AutoFireParticleMediumPrefix", K_ASCII },
    { "AutoFireParticleMediumSystem", K_ASCII },
    { "AutoFireParticleMediumMax", K_INT },
    { "AutoFireParticleLargePrefix", K_ASCII },
    { "AutoFireParticleLargeSystem", K_ASCII },
    { "AutoFireParticleLargeMax", K_INT },
    { "AutoSmokeParticleSmallPrefix", K_ASCII },
    { "AutoSmokeParticleSmallSystem", K_ASCII },
    { "AutoSmokeParticleSmallMax", K_INT },
    { "AutoSmokeParticleMediumPrefix", K_ASCII },
    { "AutoSmokeParticleMediumSystem", K_ASCII },
    { "AutoSmokeParticleMediumMax", K_INT },
    { "AutoSmokeParticleLargePrefix", K_ASCII },
    { "AutoSmokeParticleLargeSystem", K_ASCII },
    { "AutoSmokeParticleLargeMax", K_INT },
    { "AutoAflameParticlePrefix", K_ASCII },
    { "AutoAflameParticleSystem", K_ASCII },
    { "AutoAflameParticleMax", K_INT },
    { "BuildSpeed", K_REAL },
    { "MinDistFromEdgeOfMapForBuild", K_REAL },
    { "SupplyBuildBorder", K_REAL },
    { "AllowedHeightVariationForBuilding", K_REAL },
    { "MinLowEnergyProductionSpeed", K_REAL },
    { "MaxLowEnergyProductionSpeed", K_REAL },
    { "LowEnergyPenaltyModifier", K_REAL },
    { "MultipleFactory", K_REAL },
    { "RefundPercent", K_PERCENT },
    { "CommandCenterHealRange", K_REAL },
    { "CommandCenterHealAmount", K_REAL },
    { "StandardMinefieldDensity", K_REAL },
    { "StandardMinefieldDistance", K_REAL },
    { "MaxLineBuildObjects", K_INT },
    { "MaxTunnelCapacity", K_INT },
    { "MaxParticleCount", K_INT },
    { "MaxFieldParticleCount", K_INT },
    { "HorizontalScrollSpeedFactor", K_REAL },
    { "VerticalScrollSpeedFactor", K_REAL },
    { "ScrollAmountCutoff", K_REAL },
    { "CameraAdjustSpeed", K_REAL },
    { "EnforceMaxCameraHeight", K_BOOL },
    { "KeyboardScrollSpeedFactor", K_REAL },
    { "MouseScrollSpeedFactor", K_REAL },
    { "KeyboardDefaultScrollSpeedFactor", K_REAL },
    { "KeyboardCameraRotateSpeed", K_REAL },
    { "MovementPenaltyDamageState", K_BODY_DAMAGE },
    { "HealthBonus_Veteran", K_PERCENT },
    { "HealthBonus_Elite", K_PERCENT },
    { "HealthBonus_Heroic", K_PERCENT },
    { "HumanSoloPlayerHealthBonus_Easy", K_PERCENT },
    { "HumanSoloPlayerHealthBonus_Normal", K_PERCENT },
    { "HumanSoloPlayerHealthBonus_Hard", K_PERCENT },
    { "AISoloPlayerHealthBonus_Easy", K_PERCENT },
    { "AISoloPlayerHealthBonus_Normal", K_PERCENT },
    { "AISoloPlayerHealthBonus_Hard", K_PERCENT },
    { "WeaponBonus", K_WEAPON_BONUS },
    { "DefaultStructureRubbleHeight", K_REAL },
    { "FixedSeed", K_INT },
    { "ShellMapName", K_ASCII },
    { "ShellMapOn", K_BOOL },
    { "PlayIntro", K_BOOL },
    { "FirewallBehavior", K_INT },
    { "FirewallPortOverride", K_INT },
    { "FirewallPortAllocationDelta", K_SHORT_INT },
    { "GroupSelectMinSelectSize", K_INT },
    { "GroupSelectVolumeBase", K_REAL },
    { "GroupSelectVolumeIncrement", K_REAL },
    { "MaxUnitSelectSounds", K_INT },
    { "SelectionFlashSaturationFactor", K_REAL },
    { "SelectionFlashHouseColor", K_BOOL },
    { "CameraAudibleRadius", K_REAL },
    { "GroupMoveClickToGatherAreaFactor", K_REAL },
    { "ShakeSubtleIntensity", K_REAL },
    { "ShakeNormalIntensity", K_REAL },
    { "ShakeStrongIntensity", K_REAL },
    { "ShakeSevereIntensity", K_REAL },
    { "ShakeCineExtremeIntensity", K_REAL },
    { "ShakeCineInsaneIntensity", K_REAL },
    { "MaxShakeIntensity", K_REAL },
    { "MaxShakeRange", K_REAL },
    { "SellPercentage", K_PERCENT },
    { "BaseRegenHealthPercentPerSecond", K_PERCENT },
    { "BaseRegenDelay", K_DURATION_UINT },
    { "SpecialPowerViewObject", K_ASCII },
    { "ObjectPlacementOpacity", K_REAL },
    { "ObjectPlacementShadows", K_BOOL },
    { "StandardPublicBone", K_ASCII_VEC_APPEND },
    { "ShowMetrics", K_BOOL },
    { "DefaultStartingCash", K_MONEY },
    { "ShroudColor", K_RGB },
    { "ClearAlpha", K_U8 },
    { "FogAlpha", K_U8 },
    { "ShroudAlpha", K_U8 },
    { "HotKeyTextColor", K_COLOR_INT },
    { "PowerBarBase", K_INT },
    { "PowerBarIntervals", K_REAL },
    { "PowerBarYellowRange", K_INT },
    { "UnlookPersistDuration", K_DURATION_UINT },
    { "NetworkFPSHistoryLength", K_INT },
    { "NetworkLatencyHistoryLength", K_INT },
    { "NetworkRunAheadMetricsTime", K_INT },
    { "NetworkCushionHistoryLength", K_INT },
    { "NetworkRunAheadSlack", K_INT },
    { "NetworkKeepAliveDelay", K_INT },
    { "NetworkDisconnectTime", K_INT },
    { "NetworkPlayerTimeoutTime", K_INT },
    { "NetworkDisconnectScreenNotifyTime", K_INT },
    { "PlayStats", K_INT },
};
static_assert(ARRAY_SIZE(kReleaseFields) == 360, "Expected 360 Release GlobalData fields");

// Formats signed integer quarter-units (val * 0.25) purely with integer ops so C++ and Rust match byte-for-byte.
static std::string fmt_quarter(int32_t q) {
    char buf[32];
    if (q < 0) {
        uint32_t aq = static_cast<uint32_t>(-q);
        std::snprintf(buf, sizeof(buf), "-%u.%02u", aq / 4u, (aq % 4u) * 25u);
    } else {
        uint32_t aq = static_cast<uint32_t>(q);
        std::snprintf(buf, sizeof(buf), "%u.%02u", aq / 4u, (aq % 4u) * 25u);
    }
    return std::string(buf);
}

static std::string format_field_line(const FieldSpec& spec, uint32_t& rng) {
    char buf[256];
    switch (spec.kind) {
    case K_BOOL: {
        bool v = (splitmix32(rng) & 1u) != 0;
        std::snprintf(buf, sizeof(buf), "  %s = %s\n", spec.name, v ? "Yes" : "No");
        return buf;
    }
    case K_INT: {
        int32_t v = static_cast<int32_t>(splitmix32(rng) % 2001u) - 500;
        std::snprintf(buf, sizeof(buf), "  %s = %d\n", spec.name, v);
        return buf;
    }
    case K_SHORT_INT: {
        int32_t v = static_cast<int32_t>(splitmix32(rng) % 40001u) - 20000;
        std::snprintf(buf, sizeof(buf), "  %s = %d\n", spec.name, v);
        return buf;
    }
    case K_U8: {
        uint32_t v = splitmix32(rng) % 256u;
        std::snprintf(buf, sizeof(buf), "  %s = %u\n", spec.name, v);
        return buf;
    }
    case K_REAL:
    case K_ANGLE_REAL:
    case K_ACCEL_REAL: {
        int32_t q = static_cast<int32_t>(splitmix32(rng) % 2001u) - 1000;
        std::string s = fmt_quarter(q);
        std::snprintf(buf, sizeof(buf), "  %s = %s\n", spec.name, s.c_str());
        return buf;
    }
    case K_PERCENT: {
        uint32_t p = splitmix32(rng) % 401u;
        bool with_sym = (splitmix32(rng) & 1u) != 0;
        std::string s = fmt_quarter(static_cast<int32_t>(p));
        std::snprintf(buf, sizeof(buf), "  %s = %s%s\n", spec.name, s.c_str(), with_sym ? "%" : "");
        return buf;
    }
    case K_DURATION_UINT: {
        uint32_t ms = splitmix32(rng) % 30000u;
        std::snprintf(buf, sizeof(buf), "  %s = %u\n", spec.name, ms);
        return buf;
    }
    case K_ASCII: {
        uint32_t tag = splitmix32(rng) % 10000u;
        std::snprintf(buf, sizeof(buf), "  %s = Asset_%u_Path/Item\n", spec.name, tag);
        return buf;
    }
    case K_ASCII_VEC_APPEND: {
        uint32_t a = splitmix32(rng) % 1000u;
        uint32_t b = splitmix32(rng) % 1000u;
        std::snprintf(buf, sizeof(buf), "  %s = BONE_%u BONE_%u\n", spec.name, a, b);
        return buf;
    }
    case K_RGB: {
        uint32_t r = splitmix32(rng) % 256u;
        uint32_t g = splitmix32(rng) % 256u;
        uint32_t bl = splitmix32(rng) % 256u;
        std::snprintf(buf, sizeof(buf), "  %s = R:%u G:%u B:%u\n", spec.name, r, g, bl);
        return buf;
    }
    case K_COLOR_INT: {
        uint32_t r = splitmix32(rng) % 256u;
        uint32_t g = splitmix32(rng) % 256u;
        uint32_t bl = splitmix32(rng) % 256u;
        bool with_a = (splitmix32(rng) & 1u) != 0;
        if (with_a) {
            uint32_t a = splitmix32(rng) % 256u;
            std::snprintf(buf, sizeof(buf), "  %s = R:%u G:%u B:%u A:%u\n", spec.name, r, g, bl, a);
        } else {
            std::snprintf(buf, sizeof(buf), "  %s = R:%u G:%u B:%u\n", spec.name, r, g, bl);
        }
        return buf;
    }
    case K_COORD2D: {
        std::string x = fmt_quarter(static_cast<int32_t>(splitmix32(rng) % 801u) - 400);
        std::string y = fmt_quarter(static_cast<int32_t>(splitmix32(rng) % 801u) - 400);
        std::snprintf(buf, sizeof(buf), "  %s = X:%s Y:%s\n", spec.name, x.c_str(), y.c_str());
        return buf;
    }
    case K_COORD3D: {
        std::string x = fmt_quarter(static_cast<int32_t>(splitmix32(rng) % 801u) - 400);
        std::string y = fmt_quarter(static_cast<int32_t>(splitmix32(rng) % 801u) - 400);
        std::string z = fmt_quarter(static_cast<int32_t>(splitmix32(rng) % 801u) - 400);
        std::snprintf(buf, sizeof(buf), "  %s = X:%s Y:%s Z:%s\n", spec.name, x.c_str(), y.c_str(), z.c_str());
        return buf;
    }
    case K_TERRAIN_LOD: {
        uint32_t idx = splitmix32(rng) % TERRAIN_LOD_NUM_TYPES;
        std::snprintf(buf, sizeof(buf), "  %s = %s\n", spec.name, TerrainLODNames[idx]);
        return buf;
    }
    case K_TIME_OF_DAY: {
        uint32_t idx = splitmix32(rng) % TIME_OF_DAY_COUNT;
        std::snprintf(buf, sizeof(buf), "  %s = %s\n", spec.name, TimeOfDayNames[idx]);
        return buf;
    }
    case K_WEATHER: {
        uint32_t idx = splitmix32(rng) % WEATHER_COUNT;
        std::snprintf(buf, sizeof(buf), "  %s = %s\n", spec.name, WeatherNames[idx]);
        return buf;
    }
    case K_BODY_DAMAGE: {
        uint32_t idx = splitmix32(rng) % BODYDAMAGETYPE_COUNT;
        std::snprintf(buf, sizeof(buf), "  %s = %s\n", spec.name, TheBodyDamageTypeNames[idx]);
        return buf;
    }
    case K_MONEY: {
        uint32_t cash = splitmix32(rng) % 100000u;
        std::snprintf(buf, sizeof(buf), "  %s = %u\n", spec.name, cash);
        return buf;
    }
    case K_WEAPON_BONUS: {
        uint32_t c = splitmix32(rng) % WEAPONBONUSCONDITION_COUNT;
        uint32_t f = splitmix32(rng) % WeaponBonus::FIELD_COUNT;
        uint32_t pct = (splitmix32(rng) % 300u) + 10u;
        std::snprintf(buf, sizeof(buf), "  WeaponBonus = %s %s %u%%\n",
                      TheWeaponBonusNames[c], TheWeaponBonusFieldNames[f], pct);
        return buf;
    }
    }
    return "";
}

struct SeedInputs {
    std::string base_ini;
    std::string override_ini;
    std::string multi_ini;
    Int tod_arg = TIME_OF_DAY_AFTERNOON;
    UnsignedInt bonus_flags = 0;

    OptionPreferencesOverrides prefs;
    Bool has_fullviewport_dat = FALSE;

    bool has_exe = false;
    bool use_pattern_exe = false;
    uint64_t pattern_exe_bytes = 0;
    uint32_t pattern_exe_seed = 0;
    std::vector<UnsignedByte> exe_bytes;
    bool has_version = false;
    UnsignedInt version_num = 0;
    bool has_skirmish_scb = false;
    std::vector<UnsignedByte> skirmish_scb;
    bool has_mp_scb = false;
    std::vector<UnsignedByte> mp_scb;
};

static Real real_from_bits(uint32_t b) {
    Real r = 0.0f;
    std::memcpy(&r, &b, 4);
    return r;
}

static SeedInputs build_seed_inputs(uint32_t seed) {
    uint32_t rng = seed ^ 0x474C4F42u;
    SeedInputs inp;

    inp.base_ini = "GameData\n";
    if (seed == 0) {
        for (size_t i = 0; i < ARRAY_SIZE(kReleaseFields); ++i) {
            inp.base_ini += format_field_line(kReleaseFields[i], rng);
        }
        for (size_t c = 0; c < 6; ++c) {
            uint32_t tmp = rng;
            (void)tmp;
            inp.base_ini += format_field_line(FieldSpec{"WeaponBonus", K_WEAPON_BONUS}, rng);
        }
        inp.base_ini += format_field_line(FieldSpec{"StandardPublicBone", K_ASCII_VEC_APPEND}, rng);
    } else {
        uint32_t count = 24u + (splitmix32(rng) % 16u);
        for (uint32_t k = 0; k < count; ++k) {
            size_t idx = ((seed * 19u) + (splitmix32(rng) % 360u)) % ARRAY_SIZE(kReleaseFields);
            inp.base_ini += format_field_line(kReleaseFields[idx], rng);
        }
        uint32_t wb_n = 1u + (splitmix32(rng) % 4u);
        for (uint32_t w = 0; w < wb_n; ++w) {
            inp.base_ini += format_field_line(FieldSpec{"WeaponBonus", K_WEAPON_BONUS}, rng);
        }
        // Always exercise quirks periodically
        if ((seed & 3u) == 0u) {
            inp.base_ini += format_field_line(FieldSpec{"DrawSkyBox", K_BOOL}, rng);
            inp.base_ini += format_field_line(FieldSpec{"DebugAI", K_BOOL}, rng);
            inp.base_ini += format_field_line(FieldSpec{"FirewallPortAllocationDelta", K_SHORT_INT}, rng);
        }
        inp.base_ini += format_field_line(FieldSpec{((seed & 1u) == 0u) ? "SkirmishTickRate" : "TickRate", K_INT}, rng);
    }
    inp.base_ini += "End\n";

    static const Int kTodArgs[] = { -1, 0, 1, 2, 3, 4, 5, 99 };
    inp.tod_arg = kTodArgs[(seed + (splitmix32(rng) % 3u)) % ARRAY_SIZE(kTodArgs)];

    if (seed == 0) {
        inp.bonus_flags = 0;
    } else if (seed == 1) {
        inp.bonus_flags = 0x07ffffffu;
    } else if (seed == 2) {
        inp.bonus_flags = 0xf8000000u; // high bits ignored
    } else {
        inp.bonus_flags = splitmix32(rng);
    }

    // Override INI (never includes WeaponBonus to avoid EA shallow-copy alias mutation)
    inp.override_ini = "GameData\n";
    uint32_t ov_count = 8u + (splitmix32(rng) % 8u);
    for (uint32_t k = 0; k < ov_count; ++k) {
        size_t idx = splitmix32(rng) % ARRAY_SIZE(kReleaseFields);
        if (kReleaseFields[idx].kind == K_WEAPON_BONUS) {
            idx = (idx + 1u) % ARRAY_SIZE(kReleaseFields);
        }
        inp.override_ini += format_field_line(kReleaseFields[idx], rng);
    }
    inp.override_ini += "End\n";

    // Multifile INI
    inp.multi_ini = "GameData\n";
    uint32_t mu_count = 6u + (splitmix32(rng) % 6u);
    for (uint32_t k = 0; k < mu_count; ++k) {
        size_t idx = splitmix32(rng) % ARRAY_SIZE(kReleaseFields);
        inp.multi_ini += format_field_line(kReleaseFields[idx], rng);
    }
    inp.multi_ini += "End\n";

    // OptionPreferences overrides
    inp.prefs.passthrough = false;
    inp.prefs.useAlternateMouse = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.useRightMouseScrollWithAlternateMouse = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.clientRetaliationModeEnabled = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.doubleClickAttackMove = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.jpegQuality = static_cast<Int>(50u + (splitmix32(rng) % 46u));
    inp.prefs.keyboardScrollFactor = static_cast<Real>(1u + (splitmix32(rng) % 200u)) * 0.01f;
    inp.prefs.drawScrollAnchor = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.moveScrollAnchor = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.defaultIP = splitmix32(rng);
    inp.prefs.firewallBehavior = static_cast<Int>(splitmix32(rng) % 10u);
    inp.prefs.firewallPortAllocationDelta = static_cast<Short>(static_cast<Int>(splitmix32(rng) % 20001u) - 10000);
    inp.prefs.firewallPortOverride = static_cast<UnsignedShort>(splitmix32(rng) % 65536u);
    inp.prefs.saveCameraInReplay = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.useCameraInReplay = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.enablePlayerObserver = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.networkLatencyFontSize = static_cast<Int>(splitmix32(rng) % 24u);
    inp.prefs.renderFpsFontSize = static_cast<Int>(splitmix32(rng) % 24u);
    inp.prefs.systemTimeFontSize = static_cast<Int>(splitmix32(rng) % 24u);
    inp.prefs.gameTimeFontSize = static_cast<Int>(splitmix32(rng) % 24u);
    inp.prefs.playerInfoListFontSize = static_cast<Int>(splitmix32(rng) % 24u);
    inp.prefs.showMoneyPerMinute = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.gameWindowTransitionSpeedMultiplier = static_cast<Real>(1u + (splitmix32(rng) % 50u)) * 0.25f;

    // MaxCameraHeight including non-finite & boundary values
    static const uint32_t kMaxCamBits[] = {
        0x7fc00000u, // NaN
        0x7f800000u, // +Inf
        0xff800000u, // -Inf
        fbits(-50.0f),
        fbits(50.0f),
        fbits(100.0f),
        fbits(450.0f),
        fbits(1000.0f),
        fbits(1250.5f),
    };
    inp.prefs.hasMaxCameraHeight = (splitmix32(rng) % 5u) != 0u;
    inp.prefs.maxCameraHeight = real_from_bits(kMaxCamBits[splitmix32(rng) % ARRAY_SIZE(kMaxCamBits)]);

    // TerrainDrawDistanceScale including non-finite & boundary values
    static const uint32_t kDrawDistBits[] = {
        0x7fc00000u, // NaN
        0x7f800000u, // +Inf
        0xff800000u, // -Inf
        fbits(-1.0f),
        fbits(0.5f),
        fbits(1.0f),
        fbits(1.5f),
        fbits(2.0f),
        fbits(3.25f),
    };
    inp.prefs.hasTerrainDrawDistanceScale = (splitmix32(rng) % 5u) != 0u;
    inp.prefs.terrainDrawDistanceScale = real_from_bits(kDrawDistBits[splitmix32(rng) % ARRAY_SIZE(kDrawDistBits)]);

    inp.prefs.skirmishTickRate = static_cast<Int>(5u + (splitmix32(rng) % 116u));
    inp.prefs.advancedShadows = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.advancedLighting = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.useShadowVolumesForSkins = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.extendedShadowFrustumCulling = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.naturalSunElevation = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.shadowsForProjectiles = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.shadowsForProps = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.bridgeShadows = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.waterReflections = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.explosionDynamicLights = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.vehicleHeadlights = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.enablePostProcessing = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.postProcessBloom = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.postProcessHDR = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.postProcessFXAA = (splitmix32(rng) & 1u) ? TRUE : FALSE;
    inp.prefs.airUnitGroundIndicator = (splitmix32(rng) & 1u) ? TRUE : FALSE;

    static const UnsignedInt kMsaaModes[] = { 0u, 2u, 4u, 8u };
    static const UnsignedInt kAnisoModes[] = { 2u, 4u, 8u, 16u };
    inp.prefs.antiAliasing = kMsaaModes[splitmix32(rng) % ARRAY_SIZE(kMsaaModes)];
    inp.prefs.textureFilterMode = splitmix32(rng) % 5u;
    inp.prefs.textureAnisotropyLevel = kAnisoModes[splitmix32(rng) % ARRAY_SIZE(kAnisoModes)];

    static const Real kGammaVals[] = { -5.0f, 0.0f, 0.75f, 1.0f, 25.0f, 49.5f, 50.0f, 50.9f, 51.0f, 75.0f, 100.0f, 120.0f };
    inp.prefs.gammaValue = kGammaVals[(seed + (splitmix32(rng) % 4u)) % ARRAY_SIZE(kGammaVals)];
    inp.prefs.xResolution = static_cast<Int>(800u + (splitmix32(rng) % 4u) * 320u);
    inp.prefs.yResolution = static_cast<Int>(600u + (splitmix32(rng) % 4u) * 240u);
    inp.has_fullviewport_dat = (splitmix32(rng) & 1u) ? TRUE : FALSE;

    // generateExeCRC inputs
    if (seed == 999) {
        // Test the readCount > 1000 blockSize=65536 early break
        inp.has_exe = true;
        inp.use_pattern_exe = true;
        inp.pattern_exe_bytes = 65536ULL * 1002ULL + 17ULL;
        inp.pattern_exe_seed = 0x55AA1234u;
    } else {
        inp.has_exe = (splitmix32(rng) % 8u) != 0u;
        inp.use_pattern_exe = false;
        if (inp.has_exe) {
            static const size_t kExeLens[] = { 0, 1, 17, 255, 4096, 65535, 65536, 65537, 131072 };
            size_t len = kExeLens[splitmix32(rng) % ARRAY_SIZE(kExeLens)];
            inp.exe_bytes.resize(len);
            for (size_t i = 0; i < len; ++i) {
                inp.exe_bytes[i] = static_cast<UnsignedByte>(splitmix32(rng) & 0xffu);
            }
        }
    }
    inp.has_version = (splitmix32(rng) % 4u) != 0u;
    inp.version_num = splitmix32(rng);

    inp.has_skirmish_scb = (splitmix32(rng) % 3u) != 0u;
    if (inp.has_skirmish_scb) {
        size_t slen = static_cast<size_t>(splitmix32(rng) % 256u) + ((seed % 7u == 0u) ? 70000u : 0u);
        inp.skirmish_scb.resize(slen);
        for (size_t i = 0; i < slen; ++i) {
            inp.skirmish_scb[i] = static_cast<UnsignedByte>((i * 37u + seed) & 0xffu);
        }
    }
    inp.has_mp_scb = (splitmix32(rng) % 3u) != 0u;
    if (inp.has_mp_scb) {
        size_t mlen = static_cast<size_t>(splitmix32(rng) % 256u) + ((seed % 11u == 0u) ? 66000u : 0u);
        inp.mp_scb.resize(mlen);
        for (size_t i = 0; i < mlen; ++i) {
            inp.mp_scb[i] = static_cast<UnsignedByte>((i * 53u + seed * 3u) & 0xffu);
        }
    }

    return inp;
}

static int run_retail_ini_check(const char* path) {
    FILE* f = std::fopen(path, "rb");
    if (!f) { std::perror(path); return 1; }
    std::fseek(f, 0, SEEK_END);
    long sz = std::ftell(f);
    std::fseek(f, 0, SEEK_SET);
    std::string content(static_cast<size_t>(sz), '\0');
    if (sz > 0 && std::fread(&content[0], 1, static_cast<size_t>(sz), f) != static_cast<size_t>(sz)) {
        std::fclose(f);
        return 1;
    }
    std::fclose(f);

    TheWritableGlobalData = NEW GlobalData;
    g_optionPrefs = OptionPreferencesOverrides{};
    g_optionPrefs.passthrough = true;
    addon::g_hasFullviewportDat = FALSE;
    TheFileSystem->binaryMode = false;
    TheFileSystem->nextFileExists = true;
    TheFileSystem->nextFileContent = content;

    INI ini;
    HarnessXferCRC xfer;
    ini.load(AsciiString("Data\\INI\\GameData.ini"), INI_LOAD_OVERWRITE, &xfer);

    std::printf(
        "RETAIL_OK size=%ld ini_crc=0x%08x cash=%u build_speed=%.4f gravity_bits=0x%08x tick_rate=%d bones=%zu\n",
        sz, xfer.getCRC(),
        TheGlobalData->m_defaultStartingCash.countMoney(),
        TheGlobalData->m_BuildSpeed,
        fbits(TheGlobalData->m_gravity),
        TheGlobalData->m_skirmishTickRate,
        TheGlobalData->m_standardPublicBones.size()
    );

    TheWritableGlobalData->reset();
    delete TheWritableGlobalData;
    HarnessAllocTracker::cleanup();
    return 0;
}

int main(int argc, char** argv) {
    // Prevent BuildUserDataPathFromRegistry() from touching host filesystem or reading $HOME
    unsetenv("HOME");

    static_assert(sizeof(Real) == 4, "Real must be 32-bit float");
    static_assert(sizeof(Bool) == 1, "Bool must be 1 byte");
    if (PRESERVE_RETAIL_SCRIPTED_CAMERA != 1) {
        std::fprintf(stderr, "PRESERVE_RETAIL_SCRIPTED_CAMERA must be 1\n");
        return 2;
    }

    if (argc == 3 && std::strcmp(argv[1], "--check-retail-ini") == 0) {
        return run_retail_ini_check(argv[2]);
    }

    if (argc != 2) {
        std::fprintf(stderr, "Usage: gen_pkt442 <out.json>\n");
        return 1;
    }

    const std::string out_path = argv[1];
    const std::string tmp_path = out_path + ".tmp";
    FILE* fp = std::fopen(tmp_path.c_str(), "wb");
    if (!fp) { std::perror(tmp_path.c_str()); return 1; }

    std::fprintf(
        fp,
        "{\n"
        "  \"packet_id\": \"PKT-442\",\n"
        "  \"module\": \"GlobalData\",\n"
        "  \"oracle\": \"vendor/GeneralsX unmodified GlobalData.cpp + INI.cpp + GameType.cpp + crc.cpp + libgm.a\",\n"
        "  \"source_sha256\": {\n"
        "    \"GlobalData.h\": \"%s\",\n"
        "    \"GlobalData.cpp\": \"%s\",\n"
        "    \"INI.cpp\": \"%s\",\n"
        "    \"GameType.cpp\": \"%s\",\n"
        "    \"Weapon.h\": \"%s\",\n"
        "    \"Weapon.cpp\": \"%s\",\n"
        "    \"Money.cpp\": \"%s\",\n"
        "    \"crc.cpp\": \"%s\"\n"
        "  },\n"
        "  \"vector_count\": 1000,\n"
        "  \"vectors\": [\n",
        sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/GlobalData.h").c_str(),
        sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/GlobalData.cpp").c_str(),
        sha256_file("vendor/GeneralsX/Core/GameEngine/Source/Common/INI/INI.cpp").c_str(),
        sha256_file("vendor/GeneralsX/Core/GameEngine/Source/Common/System/GameType.cpp").c_str(),
        sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h").c_str(),
        sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Weapon.cpp").c_str(),
        sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/Money.cpp").c_str(),
        sha256_file("vendor/GeneralsX/Core/GameEngine/Source/Common/crc.cpp").c_str()
    );

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        SeedInputs inp = build_seed_inputs(seed);

        const uint32_t base_ini_crc = crc_str_bytes(inp.base_ini);
        const uint32_t override_ini_crc = crc_str_bytes(inp.override_ini);
        const uint32_t multi_ini_crc = crc_str_bytes(inp.multi_ini);

        TheWritableGlobalData = NEW GlobalData;
        const UnsignedInt defaults_crc = hash_global_data(*TheGlobalData);

        // 1. Parse base INI in passthrough OptionPreferences mode
        g_optionPrefs = OptionPreferencesOverrides{};
        g_optionPrefs.passthrough = true;
        addon::g_hasFullviewportDat = FALSE;
        TheFileSystem->binaryMode = false;
        TheFileSystem->nextFileExists = true;
        TheFileSystem->nextFileContent = inp.base_ini;
        {
            INI ini;
            HarnessXferCRC xfer;
            ini.load(AsciiString("Data\\INI\\GameData.ini"), INI_LOAD_OVERWRITE, &xfer);
        }
        const UnsignedInt after_base_crc = hash_global_data(*TheGlobalData);

        // 2. setTimeOfDay check
        const Bool tod_ret = TheWritableGlobalData->setTimeOfDay(static_cast<TimeOfDay>(inp.tod_arg));
        const UnsignedInt after_tod_crc = hash_global_data(*TheGlobalData);
        if (tod_ret == FALSE && after_tod_crc != after_base_crc) {
            std::fprintf(stderr, "Seed %u invariant failed: invalid tod mutated state\n", seed);
            return 3;
        }

        // 3. WeaponBonusSet::appendBonuses
        WeaponBonus bonus;
        TheGlobalData->m_weaponBonusSet->appendBonuses(inp.bonus_flags, bonus);
        uint32_t bonus_bits[5];
        for (int f = 0; f < WeaponBonus::FIELD_COUNT; ++f) {
            bonus_bits[f] = fbits(bonus.getField(static_cast<WeaponBonus::Field>(f)));
        }

        // 4. INI_LOAD_CREATE_OVERRIDES -> reset() -> INI_LOAD_MULTIFILE
        TheFileSystem->nextFileContent = inp.override_ini;
        {
            INI ini;
            HarnessXferCRC xfer;
            ini.load(AsciiString("Map\\GameData.ini"), INI_LOAD_CREATE_OVERRIDES, &xfer);
        }
        const UnsignedInt after_override_crc = hash_global_data(*TheGlobalData);

        TheWritableGlobalData->reset();
        const UnsignedInt after_reset_crc = hash_global_data(*TheGlobalData);
        if (after_reset_crc != after_tod_crc) {
            std::fprintf(stderr, "Seed %u invariant failed: after_reset_crc != after_tod_crc\n", seed);
            return 4;
        }

        TheFileSystem->nextFileContent = inp.multi_ini;
        {
            INI ini;
            HarnessXferCRC xfer;
            ini.load(AsciiString("Map\\GameDataMulti.ini"), INI_LOAD_MULTIFILE, &xfer);
        }
        const UnsignedInt after_multifile_crc = hash_global_data(*TheGlobalData);

        // 5. OptionPreferences overrides + parseCustomDefinition + getters
        g_optionPrefs = inp.prefs;
        g_optionPrefs.passthrough = false;
        addon::g_hasFullviewportDat = inp.has_fullviewport_dat;
        TheFileSystem->nextFileContent = "GameData\nEnd\n";
        {
            INI ini;
            HarnessXferCRC xfer;
            ini.load(AsciiString("Data\\INI\\EmptyGameData.ini"), INI_LOAD_OVERWRITE, &xfer);
        }
        TheWritableGlobalData->parseCustomDefinition();
        const UnsignedInt after_prefs_crc = hash_global_data(*TheGlobalData);
        const uint32_t getters_mask = compute_getters_mask(*TheGlobalData);

        // 6. generateExeCRC via GlobalData::init()
        TheFileSystem->binaryMode = true;
        TheFileSystem->hasExe = inp.has_exe;
        TheFileSystem->usePatternExe = inp.use_pattern_exe;
        TheFileSystem->patternExeBytes = inp.pattern_exe_bytes;
        TheFileSystem->patternExeSeed = inp.pattern_exe_seed;
        TheFileSystem->exeBytes = inp.exe_bytes;
        TheFileSystem->hasSkirmishScb = inp.has_skirmish_scb;
        TheFileSystem->skirmishScbBytes = inp.skirmish_scb;
        TheFileSystem->hasMpScb = inp.has_mp_scb;
        TheFileSystem->mpScbBytes = inp.mp_scb;
        g_version.versionNumber = inp.version_num;
        TheVersion = inp.has_version ? &g_version : nullptr;

        TheWritableGlobalData->init();
        const UnsignedInt exe_crc = TheGlobalData->m_exeCRC;

        delete TheWritableGlobalData;
        HarnessAllocTracker::cleanup();

        std::fprintf(
            fp,
            "    {\"seed\":%u,\"base_ini_crc\":%u,\"override_ini_crc\":%u,\"multi_ini_crc\":%u,"
            "\"defaults_crc\":%u,\"after_base_crc\":%u,\"tod_ret\":%u,\"after_tod_crc\":%u,"
            "\"bonus_bits\":[%u,%u,%u,%u,%u],\"after_override_crc\":%u,\"after_multifile_crc\":%u,"
            "\"after_reset_crc\":%u,\"after_prefs_crc\":%u,\"getters_mask\":%u,\"exe_crc\":%u}%s\n",
            seed, base_ini_crc, override_ini_crc, multi_ini_crc,
            defaults_crc, after_base_crc, static_cast<unsigned>(tod_ret), after_tod_crc,
            bonus_bits[0], bonus_bits[1], bonus_bits[2], bonus_bits[3], bonus_bits[4],
            after_override_crc, after_multifile_crc, after_reset_crc, after_prefs_crc,
            getters_mask, exe_crc, (seed + 1u < 1000u) ? "," : ""
        );
    }

    std::fprintf(fp, "  ]\n}\n");
    if (std::ferror(fp) || std::fclose(fp) != 0) {
        std::perror("write/close");
        std::remove(tmp_path.c_str());
        return 1;
    }
    if (std::rename(tmp_path.c_str(), out_path.c_str()) != 0) {
        std::perror("rename");
        std::remove(tmp_path.c_str());
        return 1;
    }
    return 0;
}
