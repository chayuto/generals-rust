// WP-3.3b Combat Leaves (PKT-141 DamageFX, PKT-142 FiringTracker) C++ Golden Vector Generator.
// Compiles directly against vendor/GeneralsX/build/macos-vulkan via scripts/build_oracle.py.
//
// Canonical build & run command:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp3_3b_combat_leaves.cpp \
//     -o /tmp/gen_wp3_3b_combat_leaves --extra-flag "-Itests/cpp_harness"
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy /tmp/gen_wp3_3b_combat_leaves docs/golden_vectors/WP3_3b_CombatLeaves.json
//
// Determinism & UB verification builds (must emit byte-identical JSON):
//   --opt O0, --opt O1, --opt O2, and --sanitize (UBSan).

#include "oracle_common.h"

#include <algorithm>
#include <array>
#include <sstream>
#include <string>
#include <vector>

#include "Common/AudioEventRTS.h"
#include "Common/AudioHandleSpecialValues.h"
#include "Common/DamageFX.h"
#include "Common/GameAudio.h"
#include "Common/GlobalData.h"
#include "Common/ModuleFactory.h"
#include "Common/NameKeyGenerator.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/PlayerTemplate.h"
#include "Common/Radar.h"
#include "Common/RandomValue.h"
#include "Common/Science.h"
#include "Common/SpecialPower.h"
#include "Common/Team.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/Upgrade.h"
#include "GameClient/Anim2D.h"
#include "GameClient/DisplayStringManager.h"
#include "GameClient/Drawable.h"
#include "GameClient/FXList.h"
#include "GameClient/GameClient.h"
#include "GameClient/Image.h"
#include "GameClient/InGameUI.h"
#include "GameLogic/AI.h"
#include "GameLogic/Damage.h"
#include "GameLogic/ExperienceTracker.h"
#include "GameLogic/FiringTracker.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/RankInfo.h"
#include "GameLogic/ScriptActions.h"
#include "GameLogic/ScriptConditions.h"
#include "GameLogic/ScriptEngine.h"
#include "GameLogic/SidesList.h"
#include "GameLogic/Weapon.h"
#include "GameLogic/WeaponSet.h"

ORACLE_DEFINE_LINK_GLOBALS("gen_wp3_3b_combat_leaves")

// --- MemberAccess helpers for private fields/methods ---
struct GLFrameTag { using type = UnsignedInt GameLogic::*; };
template struct MemberAccess<GLFrameTag, &GameLogic::m_frame>;
UnsignedInt GameLogic::* access_member(GLFrameTag);

struct DFXListTag {
    using type = ConstFXListPtr (DamageFX::*)(DamageType, Real, const Object*) const;
};
template struct MemberAccess<DFXListTag, &DamageFX::getDamageFXList>;
ConstFXListPtr (DamageFX::*access_member(DFXListTag))(DamageType, Real, const Object*) const;

struct DFXStoreMapTag {
    using type = std::hash_map<NameKeyType, DamageFX, rts::hash<NameKeyType>, rts::equal_to<NameKeyType>> DamageFXStore::*;
};
template struct MemberAccess<DFXStoreMapTag, &DamageFXStore::m_dfxmap>;
std::hash_map<NameKeyType, DamageFX, rts::hash<NameKeyType>, rts::equal_to<NameKeyType>> DamageFXStore::* access_member(DFXStoreMapTag);

#define DECL_FT_TAG(Name, Ty, Field) \
    struct Name { using type = Ty FiringTracker::*; }; \
    template struct MemberAccess<Name, &FiringTracker::Field>; \
    Ty FiringTracker::* access_member(Name);

DECL_FT_TAG(FTConsecTag, Int, m_consecutiveShots)
DECL_FT_TAG(FTVictimTag, ObjectID, m_victimID)
DECL_FT_TAG(FTCooldownTag, UnsignedInt, m_frameToStartCooldown)
DECL_FT_TAG(FTReloadTag, UnsignedInt, m_frameToForceReload)
DECL_FT_TAG(FTLoopStopTag, UnsignedInt, m_frameToStopLoopingSound)
DECL_FT_TAG(FTAudioTag, AudioHandle, m_audioHandle)

struct FTCalcSleepTag { using type = UpdateSleepTime (FiringTracker::*)(); };
template struct MemberAccess<FTCalcSleepTag, &FiringTracker::calcTimeToSleep>;
UpdateSleepTime (FiringTracker::*access_member(FTCalcSleepTag))();

struct UMGetWakeTag { using type = UpdateSleepTime (UpdateModule::*)() const; };
template struct MemberAccess<UMGetWakeTag, &UpdateModule::getWakeFrame>;
UpdateSleepTime (UpdateModule::*access_member(UMGetWakeTag))() const;

struct FTGetPhaseTag { using type = SleepyUpdatePhase (FiringTracker::*)() const; };
template struct MemberAccess<FTGetPhaseTag, &FiringTracker::getUpdatePhase>;
SleepyUpdatePhase (FiringTracker::*access_member(FTGetPhaseTag))() const;

struct UMNextCallTag { using type = UnsignedInt UpdateModule::*; };
template struct MemberAccess<UMNextCallTag, &UpdateModule::m_nextCallFrameAndPhase>;
UnsignedInt UpdateModule::* access_member(UMNextCallTag);

struct ObjFiringTrackerTag { using type = FiringTracker* Object::*; };
template struct MemberAccess<ObjFiringTrackerTag, &Object::m_firingTracker>;
FiringTracker* Object::* access_member(ObjFiringTrackerTag);

struct ObjVetTag { using type = ExperienceTracker* Object::*; };
template struct MemberAccess<ObjVetTag, &Object::m_experienceTracker>;
ExperienceTracker* Object::* access_member(ObjVetTag);

// Minimal deterministic mock AudioManager for FiringTracker tests.
class MockAudioManager final : public AudioManager {
public:
    AudioHandle next_handle = 100;
    Bool playing_flag = TRUE;
    uint32_t add_count = 0;
    uint32_t remove_count = 0;
    uint32_t rapid_fire_count = 0;
    AudioHandle last_removed = AHSV_NoSound;

    void reset_counters() {
        next_handle = 100;
        playing_flag = TRUE;
        add_count = 0;
        remove_count = 0;
        rapid_fire_count = 0;
        last_removed = AHSV_NoSound;
    }

    AudioHandle addAudioEvent(const AudioEventRTS* eventToAdd) override {
        ++add_count;
        if (eventToAdd && eventToAdd->getEventName() == "OracleRapidFireVoice") {
            ++rapid_fire_count;
        }
        AudioHandle h = next_handle++;
        return h;
    }
    void removeAudioEvent(AudioHandle audioEvent) override {
        if (audioEvent != AHSV_NoSound && audioEvent != AHSV_StopTheMusic) {
            ++remove_count;
            last_removed = audioEvent;
        }
    }
    Bool isCurrentlyPlaying(AudioHandle) override {
        return playing_flag;
    }

#if defined(RTS_DEBUG)
    void audioDebugDisplay(DebugDisplayInterface*, void*, FILE*) override {}
#endif
    void stopAudio(AudioAffect) override {}
    void pauseAudio(AudioAffect) override {}
    void resumeAudio(AudioAffect) override {}
    void pauseAmbient(Bool) override {}
    void killAudioEventImmediately(AudioHandle) override {}
    AsciiString nextMusicTrack() override { return AsciiString::TheEmptyString; }
    AsciiString prevMusicTrack() override { return AsciiString::TheEmptyString; }
    Bool isMusicPlaying() const override { return FALSE; }
    Bool hasMusicTrackCompleted(const AsciiString&, Int) const override { return FALSE; }
    void openDevice() override {}
    void closeDevice() override {}
    void* getDevice() override { return nullptr; }
    void notifyOfAudioCompletion(UnsignedInt, UnsignedInt) override {}
    UnsignedInt getProviderCount() const override { return 0; }
    AsciiString getProviderName(UnsignedInt) const override { return AsciiString::TheEmptyString; }
    UnsignedInt getProviderIndex(AsciiString) const override { return 0; }
    void selectProvider(UnsignedInt) override {}
    void unselectProvider() override {}
    UnsignedInt getSelectedProvider() const override { return 0; }
    void setSpeakerType(UnsignedInt) override {}
    UnsignedInt getSpeakerType() override { return 0; }
    UnsignedInt getNum2DSamples() const override { return 0; }
    UnsignedInt getNum3DSamples() const override { return 0; }
    UnsignedInt getNumStreams() const override { return 0; }
    UnsignedInt getNumAvailable2DSamples() const override { return 0; }
    UnsignedInt getNumAvailable3DSamples() const override { return 0; }
    Bool doesViolateLimit(AudioEventRTS*) const override { return FALSE; }
    Bool isPlayingLowerPriority(AudioEventRTS*) const override { return FALSE; }
    Bool isPlayingAlready(AudioEventRTS*) const override { return FALSE; }
    Bool isObjectPlayingVoice(UnsignedInt) const override { return FALSE; }
    void adjustVolumeOfPlayingAudio(AsciiString, Real) override {}
    void removePlayingAudio(AsciiString) override {}
    void removeAllDisabledAudio() override {}
    void* getHandleForBink() override { return nullptr; }
    void releaseHandleForBink() override {}
    void friend_forcePlayAudioEventRTS(const AudioEventRTS*) override {}
    void setPreferredProvider(AsciiString) override {}
    void setPreferredSpeaker(AsciiString) override {}
    Real getFileLengthMS(AsciiString) const override { return 0.0f; }
    void closeAnySamplesUsingFile(const void*) override {}
    void setDeviceListenerPosition() override {}
};

class MockInGameUI final : public InGameUI {
public:
    void draw() override {}
    View* createView(bool = false) override { return nullptr; }
};

class MockDisplayStringManager final : public DisplayStringManager {
public:
    DisplayString* newDisplayString() override { return nullptr; }
    DisplayString* getGroupNumeralString(Int) override { return nullptr; }
    DisplayString* getFormationLetterString() override { return nullptr; }
};

class MockGameClient final : public GameClient {
public:
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

static const char* kDamageNames[DAMAGE_NUM_TYPES] = {
    "EXPLOSION", "CRUSH", "ARMOR_PIERCING", "SMALL_ARMS", "GATTLING", "RADIATION",
    "FLAME", "LASER", "SNIPER", "POISON", "HEALING", "UNRESISTABLE", "WATER",
    "DEPLOY", "SURRENDER", "HACK", "KILL_PILOT", "PENALTY", "FALLING", "MELEE",
    "DISARM", "HAZARD_CLEANUP", "PARTICLE_BEAM", "TOPPLING", "INFANTRY_MISSILE",
    "AURORA_BOMB", "LAND_MINE", "JET_MISSILES", "STEALTHJET_MISSILES",
    "MOLOTOV_COCKTAIL", "COMANCHE_VULCAN", "SUBDUAL_MISSILE", "SUBDUAL_VEHICLE",
    "SUBDUAL_BUILDING", "SUBDUAL_UNRESISTABLE", "MICROWAVE", "KILL_GARRISONED",
    "STATUS"
};

static const char* kVetNames[LEVEL_COUNT] = { "REGULAR", "VETERAN", "ELITE", "HEROIC" };

static const char* kFXNames[13] = {
    "None", "FX_Alpha", "FX_Beta", "FX_Gamma", "FX_Delta", "FX_Epsilon", "FX_Zeta",
    "FX_Eta", "FX_Theta", "FX_Iota", "FX_Kappa", "FX_Lambda", "FX_Mu"
};

static const FXList* kFXPtrs[13] = { nullptr };

static int fx_ptr_to_id(ConstFXListPtr ptr) {
    if (!ptr) return 0;
    for (int i = 1; i < 13; ++i) {
        if (kFXPtrs[i] == ptr) return i;
    }
    return -1;
}

static uint32_t fnv_step(uint32_t h, uint32_t v) {
    h ^= v;
    h *= 16777619u;
    return h;
}

static std::string build_damage_fx_ini(const char* name, uint32_t& rng, uint32_t seed) {
    std::ostringstream ss;
    ss << "DamageFX " << name << "\n";

    if (seed % 3 != 2) {
        Real def_amt = static_cast<Real>((oracle::splitmix32(rng) % 200)) * 0.5f;
        int def_maj = 1 + static_cast<int>(oracle::splitmix32(rng) % 12);
        int def_min = static_cast<int>(oracle::splitmix32(rng) % 13);
        uint32_t def_thr = (oracle::splitmix32(rng) % 20) * 50u;
        ss << "  AmountForMajorFX = Default " << def_amt << "\n";
        ss << "  MajorFX = Default " << kFXNames[def_maj] << "\n";
        ss << "  MinorFX = Default " << kFXNames[def_min] << "\n";
        ss << "  ThrottleTime = Default " << def_thr << "\n";
    }

    int extra_entries = 2 + static_cast<int>(oracle::splitmix32(rng) % 6);
    for (int e = 0; e < extra_entries; ++e) {
        bool is_vet = (oracle::splitmix32(rng) & 1u) != 0;
        int vet_idx = static_cast<int>(oracle::splitmix32(rng) % LEVEL_COUNT);
        int dt_idx = static_cast<int>(oracle::splitmix32(rng) % DAMAGE_NUM_TYPES);
        int op = static_cast<int>(oracle::splitmix32(rng) % 4);
        if (op == 0) {
            Real amt = static_cast<Real>((oracle::splitmix32(rng) % 300)) * 0.5f;
            if (is_vet) {
                ss << "  VeterancyAmountForMajorFX = " << kVetNames[vet_idx] << " " << kDamageNames[dt_idx] << " " << amt << "\n";
            } else {
                ss << "  AmountForMajorFX = " << kDamageNames[dt_idx] << " " << amt << "\n";
            }
        } else if (op == 1) {
            int fx_idx = static_cast<int>(oracle::splitmix32(rng) % 13);
            if (is_vet) {
                ss << "  VeterancyMajorFX = " << kVetNames[vet_idx] << " " << kDamageNames[dt_idx] << " " << kFXNames[fx_idx] << "\n";
            } else {
                ss << "  MajorFX = " << kDamageNames[dt_idx] << " " << kFXNames[fx_idx] << "\n";
            }
        } else if (op == 2) {
            int fx_idx = static_cast<int>(oracle::splitmix32(rng) % 13);
            if (is_vet) {
                ss << "  VeterancyMinorFX = " << kVetNames[vet_idx] << " " << kDamageNames[dt_idx] << " " << kFXNames[fx_idx] << "\n";
            } else {
                ss << "  MinorFX = " << kDamageNames[dt_idx] << " " << kFXNames[fx_idx] << "\n";
            }
        } else {
            uint32_t thr = (oracle::splitmix32(rng) % 40) * 33u;
            if (is_vet) {
                ss << "  VeterancyThrottleTime = " << kVetNames[vet_idx] << " " << kDamageNames[dt_idx] << " " << thr << "\n";
            } else {
                ss << "  ThrottleTime = " << kDamageNames[dt_idx] << " " << thr << "\n";
            }
        }
    }
    ss << "End\n";
    return ss.str();
}

int main(int argc, char** argv) {
    const char* out_path = (argc > 1) ? argv[1] : "docs/golden_vectors/WP3_3b_CombatLeaves.json";
    oracle::init_memory();

    TheWritableGlobalData = NEW GlobalData;
    TheWritableGlobalData->m_headless = TRUE;
    TheNameKeyGenerator = NEW NameKeyGenerator;
    TheNameKeyGenerator->init();
    TheScienceStore = NEW ScienceStore;
    TheScienceStore->init();
    TheRankInfoStore = NEW RankInfoStore;
    TheRankInfoStore->init();
    ThePlayerTemplateStore = NEW PlayerTemplateStore;
    ThePlayerTemplateStore->init();
    TheSpecialPowerStore = NEW SpecialPowerStore;
    TheSpecialPowerStore->init();
    TheUpgradeCenter = NEW UpgradeCenter;
    TheUpgradeCenter->init();
    TheSidesList = NEW SidesList;
    TheSidesList->init();
    TheSidesList->validateSides();
    TheModuleFactory = NEW ModuleFactory;
    TheModuleFactory->init();
    TheFXListStore = NEW FXListStore;
    TheDamageFXStore = NEW DamageFXStore;
    TheWeaponStore = NEW WeaponStore;
    TheThingFactory = NEW ThingFactory;
    TheAI = NEW AI;
    TheAI->init();
    ThePartitionManager = NEW PartitionManager;
    ThePartitionManager->init();
    TheScriptActions = NEW ScriptActions;
    TheScriptConditions = NEW ScriptConditions;
    TheScriptEngine = NEW ScriptEngine;
    TheScriptEngine->init();
    TheGameLogic = NEW GameLogic;
    TheTeamFactory = NEW TeamFactory;
    TheTeamFactory->init();
    ThePlayerList = NEW PlayerList;
    ThePlayerList->init();
    ThePlayerList->newGame();
    TheRadar = NEW RadarDummy;
    TheRadar->init();
    TheMappedImageCollection = NEW ImageCollection;
    TheAnim2DCollection = NEW Anim2DCollection;

    MockAudioManager mock_audio;
    TheAudio = &mock_audio;
    MockInGameUI mock_ui;
    TheInGameUI = &mock_ui;
    MockDisplayStringManager mock_dsm;
    TheDisplayStringManager = &mock_dsm;
    MockGameClient mock_client;
    TheGameClient = &mock_client;

    oracle::MemINI mem_ini;
    {
        std::ostringstream fx_ini;
        for (int i = 1; i < 13; ++i) {
            fx_ini << "FXList " << kFXNames[i] << "\nEnd\n";
        }
        mem_ini.parse_blocks(fx_ini.str(), INI_LOAD_OVERWRITE, [](INI* ini, const char* tok) {
            if (std::strcmp(tok, "FXList") == 0) FXListStore::parseFXListDefinition(ini);
        });
        for (int i = 1; i < 13; ++i) {
            kFXPtrs[i] = TheFXListStore->findFXList(kFXNames[i]);
        }
    }

    // Create ThingTemplate with a Primary weapon & VoiceRapidFire so Object creates FiringTracker!
    {
        const char* wpn_ini =
            "Weapon OracleBaseGun\n"
            "  PrimaryDamage = 10\n"
            "  AttackRange = 100\n"
            "  DelayBetweenShots = 100\n"
            "End\n";
        mem_ini.parse_blocks(wpn_ini, INI_LOAD_OVERWRITE, [](INI* ini, const char* tok) {
            if (std::strcmp(tok, "Weapon") == 0) WeaponStore::parseWeaponTemplateDefinition(ini);
        });
        const char* thing_ini =
            "Object OracleShooterTemplate\n"
            "  WeaponSet\n"
            "    Conditions = None\n"
            "    Weapon = PRIMARY OracleBaseGun\n"
            "  End\n"
            "  Body = InactiveBody ModuleTag_Body\n"
            "  End\n"
            "  UnitSpecificSounds\n"
            "    VoiceRapidFire = OracleRapidFireVoice\n"
            "  End\n"
            "End\n"
            "Object OracleVictimTemplate\n"
            "  Body = InactiveBody ModuleTag_Body\n"
            "  End\n"
            "End\n";
        mem_ini.parse_blocks(thing_ini, INI_LOAD_OVERWRITE, [](INI* ini, const char* tok) {
            if (std::strcmp(tok, "Object") == 0) INI::parseObjectDefinition(ini);
        });
    }

    const ThingTemplate* shooter_tmpl = TheThingFactory->findTemplate("OracleShooterTemplate");
    const ThingTemplate* victim_tmpl = TheThingFactory->findTemplate("OracleVictimTemplate");
    if (!shooter_tmpl || !victim_tmpl) {
        std::fprintf(stderr, "Failed to find oracle ThingTemplates\n");
        return 2;
    }

    const char* h_dfx = "vendor/GeneralsX/Core/GameEngine/Include/Common/DamageFX.h";
    const char* cpp_dfx = "vendor/GeneralsX/Core/GameEngine/Source/Common/DamageFX.cpp";
    const char* cpp_inidfx = "vendor/GeneralsX/Core/GameEngine/Source/Common/INI/INIDamageFX.cpp";
    const char* h_ft = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/FiringTracker.h";
    const char* cpp_ft = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/FiringTracker.cpp";

    std::ostringstream out;
    out << "{\n"
        << "  \"packet_id\": \"WP3_3b_CombatLeaves\",\n"
        << "  \"packets\": [\"PKT-141\", \"PKT-142\", \"PKT-135\", \"PKT-136\", \"PKT-146\", \"PKT-147\", \"PKT-148\", \"PKT-149\"],\n"
        << "  \"modules\": [\"DamageFX\", \"DamageFXStore\", \"FiringTracker\"],\n"
        << "  \"vector_count\": 1000,\n"
        << "  \"retail_compatible_xfer_save\": " << (RETAIL_COMPATIBLE_XFER_SAVE ? 1 : 0) << ",\n"
        << "  \"source_sha256\": {\n"
        << "    \"" << h_dfx << "\": \"" << oracle::sha256_file(h_dfx) << "\",\n"
        << "    \"" << cpp_dfx << "\": \"" << oracle::sha256_file(cpp_dfx) << "\",\n"
        << "    \"" << cpp_inidfx << "\": \"" << oracle::sha256_file(cpp_inidfx) << "\",\n"
        << "    \"" << h_ft << "\": \"" << oracle::sha256_file(h_ft) << "\",\n"
        << "    \"" << cpp_ft << "\": \"" << oracle::sha256_file(cpp_ft) << "\"\n"
        << "  },\n";

    std::ostringstream vec_out;
    std::string seed0_diag;

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed ^ 0x33b14142u;
        if (rng == 0) rng = 0x12345678u;

        // ============================================================
        // Part 1: PKT-141 DamageFX / DamageFXStore
        // ============================================================
        std::string dfx_name = "DFX_" + std::to_string(seed);
        std::string dfx_ini = build_damage_fx_ini(dfx_name.c_str(), rng, seed);
        if (seed % 10 == 0) {
            // Redefinition must clear previous entries
            dfx_ini += build_damage_fx_ini(dfx_name.c_str(), rng, seed + 1);
        }
        TheDamageFXStore->*access_member(DFXStoreMapTag()) = {};
        mem_ini.parse_blocks(dfx_ini, INI_LOAD_OVERWRITE, [](INI* ini, const char* tok) {
            if (std::strcmp(tok, "DamageFX") == 0) DamageFXStore::parseDamageFXDefinition(ini);
        });

        bool bad_threw = false;
        std::string bad_ini;
        if (seed == 0 || (seed % 47 == 0)) {
            bad_ini = "DamageFX BadDFX_" + std::to_string(seed) + "\n  AmountForMajorFX = NONEXISTENT_DMG_XYZ 10\nEnd\n";
            try {
                mem_ini.parse_blocks(bad_ini, INI_LOAD_OVERWRITE, [](INI* ini, const char* tok) {
                    if (std::strcmp(tok, "DamageFX") == 0) DamageFXStore::parseDamageFXDefinition(ini);
                });
            } catch (...) {
                bad_threw = true;
            }
        }

        const DamageFX* dfx = TheDamageFXStore->findDamageFX(dfx_name.c_str());
        if (!dfx) {
            std::fprintf(stderr, "Missing DamageFX %s\n", dfx_name.c_str());
            return 3;
        }

        // Create temporary source objects for each VeterancyLevel (0..3)
        Object* src_objs[LEVEL_COUNT];
        for (int v = 0; v < LEVEL_COUNT; ++v) {
            src_objs[v] = TheThingFactory->newObject(victim_tmpl, nullptr);
            if (src_objs[v] && src_objs[v]->getExperienceTracker()) {
                src_objs[v]->getExperienceTracker()->setVeterancyLevel(static_cast<VeterancyLevel>(v), FALSE);
            }
        }

        // Probe queries + full table digest
        uint32_t table_hash = 2166136261u;
        for (int dt = 0; dt < DAMAGE_NUM_TYPES; ++dt) {
            for (int v = 0; v < LEVEL_COUNT; ++v) {
                UnsignedInt thr = dfx->getDamageFXThrottleTime(static_cast<DamageType>(dt), src_objs[v]);
                // Probe at 0.0f, -1.0f, 50.0f, 1000.0f
                ConstFXListPtr f0 = (dfx->*access_member(DFXListTag()))(static_cast<DamageType>(dt), 0.0f, src_objs[v]);
                ConstFXListPtr f_neg = (dfx->*access_member(DFXListTag()))(static_cast<DamageType>(dt), -1.0f, src_objs[v]);
                ConstFXListPtr f_mid = (dfx->*access_member(DFXListTag()))(static_cast<DamageType>(dt), 50.0f, src_objs[v]);
                ConstFXListPtr f_hi = (dfx->*access_member(DFXListTag()))(static_cast<DamageType>(dt), 1000.0f, src_objs[v]);
                table_hash = fnv_step(table_hash, thr);
                table_hash = fnv_step(table_hash, static_cast<uint32_t>(fx_ptr_to_id(f0)));
                table_hash = fnv_step(table_hash, static_cast<uint32_t>(fx_ptr_to_id(f_neg)));
                table_hash = fnv_step(table_hash, static_cast<uint32_t>(fx_ptr_to_id(f_mid)));
                table_hash = fnv_step(table_hash, static_cast<uint32_t>(fx_ptr_to_id(f_hi)));
            }
        }

        // 4 specific probes per seed
        int p_dt[4];
        int p_vet[4]; // -1 => nullptr source
        Real p_amt[4];
        int p_fx[4];
        UnsignedInt p_thr[4];
        for (int p = 0; p < 4; ++p) {
            p_dt[p] = static_cast<int>(oracle::splitmix32(rng) % DAMAGE_NUM_TYPES);
            p_vet[p] = (p == 0 && (seed % 2 == 0)) ? -1 : static_cast<int>(oracle::splitmix32(rng) % LEVEL_COUNT);
            if (p == 1 && (seed % 3 == 0)) {
                p_amt[p] = 0.0f;
            } else if (p == 2 && (seed % 5 == 0)) {
                p_amt[p] = -5.0f;
            } else {
                p_amt[p] = static_cast<Real>(oracle::splitmix32(rng) % 250) * 0.5f;
            }
            const Object* s_ptr = (p_vet[p] < 0) ? nullptr : src_objs[p_vet[p]];
            ConstFXListPtr fx = (dfx->*access_member(DFXListTag()))(static_cast<DamageType>(p_dt[p]), p_amt[p], s_ptr);
            p_fx[p] = fx_ptr_to_id(fx);
            p_thr[p] = dfx->getDamageFXThrottleTime(static_cast<DamageType>(p_dt[p]), s_ptr);
        }

        // ============================================================
        // Part 2: PKT-142 FiringTracker
        // ============================================================
        // Configure a randomized WeaponTemplate + Weapon instance
        Int shots_one = 1 + static_cast<Int>(oracle::splitmix32(rng) % 4);
        Int shots_two = shots_one + 1 + static_cast<Int>(oracle::splitmix32(rng) % 5);
        UnsignedInt coast_ms = (seed % 7 == 0) ? 0u : (100u + (oracle::splitmix32(rng) % 12u) * 100u);
        UnsignedInt reload_ms = (seed % 6 == 0) ? 0u : (150u + (oracle::splitmix32(rng) % 10u) * 100u);
        UnsignedInt loop_ms = (seed % 4 == 0) ? 0u : (100u + (oracle::splitmix32(rng) % 8u) * 100u);

        std::string ft_wpn_name = "FTWpn_" + std::to_string(seed);
        {
            std::ostringstream wi;
            wi << "Weapon " << ft_wpn_name << "\n"
               << "  PrimaryDamage = 10\n"
               << "  AttackRange = 120\n"
               << "  ClipSize = 10\n"
               << "  ClipReloadTime = 1000\n"
               << "  DelayBetweenShots = 100\n"
               << "  ContinuousFireOne = " << shots_one << "\n"
               << "  ContinuousFireTwo = " << shots_two << "\n"
               << "  ContinuousFireCoast = " << coast_ms << "\n"
               << "  AutoReloadWhenIdle = " << reload_ms << "\n"
               << "  FireSound = OracleFireSound\n"
               << "  FireSoundLoopTime = " << loop_ms << "\n"
               << "End\n";
            mem_ini.parse_blocks(wi.str(), INI_LOAD_OVERWRITE, [](INI* ini, const char* tok) {
                if (std::strcmp(tok, "Weapon") == 0) WeaponStore::parseWeaponTemplateDefinition(ini);
            });
        }
        const WeaponTemplate* wtmpl = TheWeaponStore->findWeaponTemplate(ft_wpn_name.c_str());
        Weapon* wpn = TheWeaponStore->allocateNewWeapon(wtmpl, PRIMARY_WEAPON);

        UnsignedInt frame0 = 100u + (oracle::splitmix32(rng) % 500u);
        TheGameLogic->*access_member(GLFrameTag()) = frame0;

        Object* shooter = TheThingFactory->newObject(shooter_tmpl, nullptr);
        Object* victim_a = TheThingFactory->newObject(victim_tmpl, nullptr);
        Object* victim_b = TheThingFactory->newObject(victim_tmpl, nullptr);
        if ((seed & 1u) != 0) victim_a->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_FAERIE_FIRE));
        if ((seed & 2u) != 0) victim_b->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_FAERIE_FIRE));

        FiringTracker* ft = shooter->*access_member(ObjFiringTrackerTag());
        if (!ft) {
            std::fprintf(stderr, "Shooter missing FiringTracker!\n");
            return 4;
        }

        mock_audio.reset_counters();
        uint32_t init_phase = static_cast<uint32_t>((ft->*access_member(FTGetPhaseTag()))());
        uint32_t init_wake = static_cast<uint32_t>((ft->*access_member(UMGetWakeTag()))());
        uint32_t init_disabled_all = (ft->getDisabledTypesToProcess() == DISABLEDMASK_ALL) ? 1u : 0u;

        // Run a deterministic sequence of 12 events (shots + frame advances + update calls)
        struct StepEvent {
            uint8_t kind; // 0 = shot at A, 1 = shot at B, 2 = ground shot (INVALID_ID), 3 = update()
            UnsignedInt dt_frames;
            UnsignedInt next_shot_offset;
            Bool audio_playing;
        };
        std::vector<StepEvent> steps;
        uint32_t trace_hash = 2166136261u;
        UnsignedInt cur_frame = frame0;
        std::ostringstream s0_steps;

        for (int st = 0; st < 12; ++st) {
            StepEvent ev{};
            ev.kind = static_cast<uint8_t>(oracle::splitmix32(rng) % 4);
            ev.dt_frames = 1u + (oracle::splitmix32(rng) % 35u);
            ev.next_shot_offset = oracle::splitmix32(rng) % 10u;
            ev.audio_playing = ((oracle::splitmix32(rng) & 3u) != 0) ? TRUE : FALSE;
            if (seed == 0 && st < 8) {
                // Force coverage of speedUp (mean -> fast), target switch, ground shot, and coolDown in seed 0
                ev.kind = (st < 5) ? 0 : (st == 5 ? 1 : (st == 6 ? 2 : 3));
                ev.dt_frames = (st == 7) ? 40u : 3u;
                ev.audio_playing = (st != 2) ? TRUE : FALSE;
            }
            steps.push_back(ev);

            cur_frame += ev.dt_frames;
            TheGameLogic->*access_member(GLFrameTag()) = cur_frame;
            mock_audio.playing_flag = ev.audio_playing;

            UnsignedInt upd_sleep = 0;
            if (ev.kind < 3) {
                ObjectID vid = (ev.kind == 0) ? victim_a->getID()
                              : (ev.kind == 1) ? victim_b->getID()
                                               : INVALID_ID;
                wpn->setPossibleNextShotFrame(cur_frame + ev.next_shot_offset);
                ft->shotFired(wpn, vid);
            } else {
                upd_sleep = static_cast<UnsignedInt>(ft->update());
            }

            Int consec = ft->getNumConsecutiveShotsAtVictim(victim_a);
            Int consec_b = ft->getNumConsecutiveShotsAtVictim(victim_b);
            Int consec_null = ft->getNumConsecutiveShotsAtVictim(nullptr);
            ObjectID last_v = ft->getLastShotVictim();
            UnsignedInt wake_f = static_cast<UnsignedInt>((ft->*access_member(UMGetWakeTag()))());
            UnsignedInt cd_f = ft->*access_member(FTCooldownTag());
            UnsignedInt rl_f = ft->*access_member(FTReloadTag());
            UnsignedInt lp_f = ft->*access_member(FTLoopStopTag());
            uint32_t wb_mask = shooter->getWeaponBonusCondition();
            const Drawable* sdraw = shooter->getDrawable();
            uint32_t mc_mean = (sdraw && sdraw->getModelConditionFlags().test(MODELCONDITION_CONTINUOUS_FIRE_MEAN)) ? 1u : 0u;
            uint32_t mc_fast = (sdraw && sdraw->getModelConditionFlags().test(MODELCONDITION_CONTINUOUS_FIRE_FAST)) ? 1u : 0u;
            uint32_t mc_slow = (sdraw && sdraw->getModelConditionFlags().test(MODELCONDITION_CONTINUOUS_FIRE_SLOW)) ? 1u : 0u;

            trace_hash = fnv_step(trace_hash, upd_sleep);
            trace_hash = fnv_step(trace_hash, static_cast<uint32_t>(consec));
            trace_hash = fnv_step(trace_hash, static_cast<uint32_t>(consec_b));
            trace_hash = fnv_step(trace_hash, static_cast<uint32_t>(consec_null));
            trace_hash = fnv_step(trace_hash, static_cast<uint32_t>(last_v));
            trace_hash = fnv_step(trace_hash, wake_f);
            trace_hash = fnv_step(trace_hash, cd_f);
            trace_hash = fnv_step(trace_hash, rl_f);
            trace_hash = fnv_step(trace_hash, lp_f);
            trace_hash = fnv_step(trace_hash, wb_mask);
            trace_hash = fnv_step(trace_hash, (mc_mean << 2) | (mc_fast << 1) | mc_slow);
            trace_hash = fnv_step(trace_hash, mock_audio.add_count);
            trace_hash = fnv_step(trace_hash, mock_audio.remove_count);
            trace_hash = fnv_step(trace_hash, mock_audio.rapid_fire_count);

            if (seed == 0) {
                if (st > 0) s0_steps << ",";
                s0_steps << "{\"step\":" << st
                         << ",\"kind\":" << static_cast<int>(ev.kind)
                         << ",\"frame\":" << cur_frame
                         << ",\"upd_sleep\":" << upd_sleep
                         << ",\"consec\":" << ft->*access_member(FTConsecTag())
                         << ",\"last_v\":" << static_cast<int32_t>(last_v)
                         << ",\"wake\":" << wake_f
                         << ",\"cd_f\":" << cd_f
                         << ",\"rl_f\":" << rl_f
                         << ",\"lp_f\":" << lp_f
                         << ",\"wb_mask\":" << wb_mask
                         << ",\"mc_bits\":" << ((mc_mean << 2) | (mc_fast << 1) | mc_slow)
                         << ",\"audio_add\":" << mock_audio.add_count
                         << ",\"audio_rem\":" << mock_audio.remove_count
                         << ",\"rapid\":" << mock_audio.rapid_fire_count
                         << "}";
            }
        }

        // Snapshot XferSave, XferLoad round-trip, and XferCRC
        oracle::MemoryXfer ft_save(XFER_SAVE);
        ft_save.open("fts");
        oracle::call_snapshot_xfer(ft, &ft_save);
        ft_save.close();

        XferCRC ft_xcrc;
        ft_xcrc.open("ftc");
        oracle::call_snapshot_crc(ft, &ft_xcrc);
        ft_xcrc.close();

        // Test XferLoad round-trip on a fresh shooter's FiringTracker
        Object* shooter_copy = TheThingFactory->newObject(shooter_tmpl, nullptr);
        FiringTracker* ft_copy = shooter_copy->*access_member(ObjFiringTrackerTag());
        oracle::MemoryXfer ft_load(XFER_LOAD);
        ft_load.bytes = ft_save.bytes;
        ft_load.open("ftl");
        oracle::call_snapshot_xfer(ft_copy, &ft_load);
        ft_load.close();
        oracle::MemoryXfer ft_resave(XFER_SAVE);
        ft_resave.open("ftr");
        oracle::call_snapshot_xfer(ft_copy, &ft_resave);
        ft_resave.close();
        if (ft_save.bytes != ft_resave.bytes) {
            std::fprintf(stderr, "FiringTracker Xfer round-trip mismatch at seed %u\n", seed);
            return 5;
        }

        vec_out << "    {\"seed\":" << seed
                << ",\"dfx_name\":\"" << dfx_name << "\""
                << ",\"dfx_ini\":\"" << oracle::json_escape(dfx_ini) << "\""
                << ",\"bad_ini\":\"" << oracle::json_escape(bad_ini) << "\""
                << ",\"bad_threw\":" << (bad_threw ? "true" : "false")
                << ",\"dfx_table_hash\":" << table_hash
                << ",\"p_dt\":[" << p_dt[0] << "," << p_dt[1] << "," << p_dt[2] << "," << p_dt[3] << "]"
                << ",\"p_vet\":[" << p_vet[0] << "," << p_vet[1] << "," << p_vet[2] << "," << p_vet[3] << "]"
                << ",\"p_amt_bits\":[" << oracle::fbits(p_amt[0]) << "," << oracle::fbits(p_amt[1])
                                      << "," << oracle::fbits(p_amt[2]) << "," << oracle::fbits(p_amt[3]) << "]"
                << ",\"p_fx\":[" << p_fx[0] << "," << p_fx[1] << "," << p_fx[2] << "," << p_fx[3] << "]"
                << ",\"p_thr\":[" << p_thr[0] << "," << p_thr[1] << "," << p_thr[2] << "," << p_thr[3] << "]"
                << ",\"ft_frame0\":" << frame0
                << ",\"ft_shots_one\":" << shots_one
                << ",\"ft_shots_two\":" << shots_two
                << ",\"ft_coast_ms\":" << coast_ms
                << ",\"ft_reload_ms\":" << reload_ms
                << ",\"ft_loop_ms\":" << loop_ms
                << ",\"ft_shooter_id\":" << static_cast<int32_t>(shooter->getID())
                << ",\"ft_victim_a_id\":" << static_cast<int32_t>(victim_a->getID())
                << ",\"ft_victim_b_id\":" << static_cast<int32_t>(victim_b->getID())
                << ",\"ft_victim_a_ff\":" << ((seed & 1u) != 0 ? 1 : 0)
                << ",\"ft_victim_b_ff\":" << ((seed & 2u) != 0 ? 1 : 0)
                << ",\"ft_init_phase\":" << init_phase
                << ",\"ft_init_wake\":" << init_wake
                << ",\"ft_init_dis_all\":" << init_disabled_all
                << ",\"ft_steps\":[";
        for (size_t k = 0; k < steps.size(); ++k) {
            if (k > 0) vec_out << ",";
            vec_out << "[" << static_cast<int>(steps[k].kind) << "," << steps[k].dt_frames << ","
                    << steps[k].next_shot_offset << "," << (steps[k].audio_playing ? 1 : 0) << "]";
        }
        vec_out << "]"
                << ",\"ft_trace_hash\":" << trace_hash
                << ",\"ft_final_consec\":" << ft->*access_member(FTConsecTag())
                << ",\"ft_final_victim\":" << static_cast<int32_t>(ft->getLastShotVictim())
                << ",\"ft_final_cd\":" << ft->*access_member(FTCooldownTag())
                << ",\"ft_final_rl\":" << ft->*access_member(FTReloadTag())
                << ",\"ft_final_lp\":" << ft->*access_member(FTLoopStopTag())
                << ",\"ft_next_call_raw\":" << ft->*access_member(UMNextCallTag())
                << ",\"ft_xfer_hex\":\"" << ft_save.hex() << "\""
                << ",\"ft_xfer_crc\":" << ft_xcrc.getCRC()
                << "}" << ((seed + 1u < 1000u) ? "," : "") << "\n";

        if (seed == 0) {
            std::ostringstream d0;
            d0 << "  \"seed_0_diagnostics\": {\n"
               << "    \"dfx_table_hash\": " << table_hash << ",\n"
               << "    \"ft_init_phase\": " << init_phase << ",\n"
               << "    \"ft_init_wake\": " << init_wake << ",\n"
               << "    \"ft_steps\": [" << s0_steps.str() << "],\n"
               << "    \"ft_xfer_hex\": \"" << ft_save.hex() << "\",\n"
               << "    \"ft_xfer_crc\": " << ft_xcrc.getCRC() << "\n"
               << "  },\n";
            seed0_diag = d0.str();
        }

        // Destroy temporary weapon; skip GameLogic::update() since GameLogic::init() is intentionally skipped
        deleteInstance(wpn);
    }

    out << seed0_diag
        << "  \"vectors\": [\n"
        << vec_out.str()
        << "  ]\n}\n";

    oracle::write_file_atomic(out_path, out.str());
    shutdownMemoryManager();
    _exit(0);
}
