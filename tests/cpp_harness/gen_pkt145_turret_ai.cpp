// PKT-145 (TurretAI, TurretAIData, TurretStateMachine) (WP-3.3) C++ Golden Vector Generator (Zero-Stub Build).
// Compiles directly against pre-built vendor/GeneralsX/build/macos-vulkan via scripts/build_oracle.py.
//
// Canonical build & run command:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt145_turret_ai.cpp -o /tmp/gen_pkt145_turret_ai
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy /tmp/gen_pkt145_turret_ai docs/golden_vectors/PKT-145.json
//
// Determinism & UB verification builds (must emit byte-identical JSON):
//   --opt O0, --opt O1, --opt O2, and --sanitize (UBSan).

#include "oracle_common.h"

#include <sstream>
#include <unistd.h>

#include "Common/DamageFX.h"
#include "Common/GameAudio.h"
#include "Common/GameState.h"
#include "Common/GlobalData.h"
#include "Common/ModuleFactory.h"
#include "Common/MultiplayerSettings.h"
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
#include "GameClient/ControlBar.h"
#include "GameClient/DisplayStringManager.h"
#include "GameClient/Drawable.h"
#include "GameClient/FXList.h"
#include "GameClient/GameClient.h"
#include "GameClient/Image.h"
#include "GameClient/InGameUI.h"
#include "GameLogic/AI.h"
#include "GameLogic/Armor.h"
#include "GameLogic/Damage.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/GhostObject.h"
#include "GameLogic/Locomotor.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectCreationList.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/RankInfo.h"
#include "GameLogic/ScriptEngine.h"
#include "GameLogic/SidesList.h"
#include "GameLogic/TerrainLogic.h"
#include "GameLogic/TurretAI.h"
#include "GameLogic/Weapon.h"
#include "GameLogic/WeaponSet.h"

ORACLE_DEFINE_LINK_GLOBALS("gen_pkt145_turret_ai")

struct GLFrameTag { using type = UnsignedInt GameLogic::*; };
template struct MemberAccess<GLFrameTag, &GameLogic::m_frame>;
UnsignedInt GameLogic::* access_member(GLFrameTag);

struct TurretSMTag { using type = TurretStateMachine* TurretAI::*; };
template struct MemberAccess<TurretSMTag, &TurretAI::m_turretStateMachine>;
TurretStateMachine* TurretAI::* access_member(TurretSMTag);

struct TurretAngleTag { using type = Real TurretAI::*; };
template struct MemberAccess<TurretAngleTag, &TurretAI::m_angle>;
Real TurretAI::* access_member(TurretAngleTag);

struct TurretPitchTag { using type = Real TurretAI::*; };
template struct MemberAccess<TurretPitchTag, &TurretAI::m_pitch>;
Real TurretAI::* access_member(TurretPitchTag);

struct TurretSweepUntilTag { using type = UnsignedInt TurretAI::*; };
template struct MemberAccess<TurretSweepUntilTag, &TurretAI::m_enableSweepUntil>;
UnsignedInt TurretAI::* access_member(TurretSweepUntilTag);

struct TurretTargetTag { using type = TurretTargetType TurretAI::*; };
template struct MemberAccess<TurretTargetTag, &TurretAI::m_target>;
TurretTargetType TurretAI::* access_member(TurretTargetTag);

struct TurretContFireTag { using type = UnsignedInt TurretAI::*; };
template struct MemberAccess<TurretContFireTag, &TurretAI::m_continuousFireExpirationFrame>;
UnsignedInt TurretAI::* access_member(TurretContFireTag);

struct TurretSleepUntilTag { using type = UnsignedInt TurretAI::*; };
template struct MemberAccess<TurretSleepUntilTag, &TurretAI::m_sleepUntil>;
UnsignedInt TurretAI::* access_member(TurretSleepUntilTag);

class NullDisplayStringManager : public DisplayStringManager {
public:
    DisplayString* newDisplayString() override { return nullptr; }
    DisplayString* getGroupNumeralString(Int) override { return nullptr; }
    DisplayString* getFormationLetterString() override { return nullptr; }
};

class NullGameClient : public GameClient {
public:
    void init() override {}
    void reset() override {}
    void update() override {}
    void createRayEffectByTemplate(const Coord3D*, const Coord3D*, const ThingTemplate*) override {}
    void addScorch(const Coord3D*, Real, Scorches) override {}
    Drawable* friend_createDrawable(const ThingTemplate* thing, DrawableStatusBits statusBits) override {
        return thing ? newInstance(Drawable)(thing, statusBits) : nullptr;
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

class NullInGameUI : public InGameUI {
public:
    void init() override {}
    void reset() override {}
    void update() override {}
    void draw() override {}
    View* createView(bool = false) override { return nullptr; }
};

class NullAudioManager : public AudioManager {
public:
    void init() override {}
    void postProcessLoad() override {}
    void reset() override {}
    void update() override {}
#if defined(RTS_DEBUG)
    void audioDebugDisplay(DebugDisplayInterface*, void*, FILE*) override {}
#endif
    void stopAudio(AudioAffect) override {}
    void pauseAudio(AudioAffect) override {}
    void resumeAudio(AudioAffect) override {}
    void pauseAmbient(Bool) override {}
    AudioHandle addAudioEvent(const AudioEventRTS*) override { return 0; }
    void removeAudioEvent(AudioHandle) override {}
    void killAudioEventImmediately(AudioHandle) override {}
    Bool isCurrentlyPlaying(AudioHandle) override { return FALSE; }
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

static const char* kWeaponSlots[WEAPONSLOT_COUNT] = { "PRIMARY", "SECONDARY", "TERTIARY" };

static std::string build_turret_ini(
    uint32_t& rng,
    uint32_t seed,
    uint32_t& out_slots_mask,
    bool& out_allows_pitch,
    bool& out_fires_while_turning,
    bool& out_init_disabled
) {
    auto rfloat = [&](int lo, int hi, Real scale) -> Real {
        int32_t span = hi - lo + 1;
        return static_cast<Real>(lo + static_cast<int32_t>(oracle::splitmix32(rng) % span)) * scale;
    };

    Real turn_rate_deg = (seed % 11 == 0) ? rfloat(300, 1800, 0.5f) : rfloat(20, 360, 0.5f);
    Real pitch_rate_deg = rfloat(15, 240, 0.5f);
    Real nat_angle_deg = (seed == 0) ? 0.0f : static_cast<Real>(static_cast<int32_t>(oracle::splitmix32(rng) % 361) - 180);
    Real nat_pitch_deg = (seed % 3 == 0) ? 0.0f : static_cast<Real>(static_cast<int32_t>(oracle::splitmix32(rng) % 61) - 30);
    Real fire_pitch_deg = (seed % 5 == 0) ? rfloat(10, 60, 0.5f) : 0.0f;
    Real min_pitch_deg = (seed % 4 == 0) ? -rfloat(5, 50, 0.5f) : 0.0f;
    Real ground_pitch_deg = (seed % 3 == 1) ? rfloat(5, 45, 0.5f) : 0.0f;

    Real sweep0 = (seed % 2 == 0) ? rfloat(2, 40, 0.5f) : 0.0f;
    Real sweep1 = (seed % 3 == 0) ? rfloat(2, 35, 0.5f) : 0.0f;
    Real sweep2 = (seed % 7 == 0) ? rfloat(2, 30, 0.5f) : 0.0f;
    Real sspd0 = rfloat(2, 30, 0.1f);
    Real sspd1 = rfloat(2, 30, 0.1f);
    Real sspd2 = rfloat(2, 30, 0.1f);

    static const uint32_t slot_masks[] = { 1u, 2u, 3u, 5u, 7u };
    out_slots_mask = (seed == 0) ? 1u : slot_masks[oracle::splitmix32(rng) % 5u];
    out_allows_pitch = (seed == 0) ? true : ((oracle::splitmix32(rng) & 1u) != 0);
    out_fires_while_turning = (seed % 4 == 2);
    out_init_disabled = (seed % 9 == 4);

    // Exercise 0/0 early return, lo == hi, lo < hi, and lo > hi PRNG guard!
    Real min_scan_deg = 0.0f;
    Real max_scan_deg = 0.0f;
    int scan_mode = static_cast<int>(seed % 6);
    if (scan_mode == 0) {
        min_scan_deg = rfloat(5, 25, 1.0f);
        max_scan_deg = min_scan_deg + rfloat(5, 45, 1.0f);
    } else if (scan_mode == 1) {
        min_scan_deg = rfloat(10, 40, 1.0f);
        max_scan_deg = min_scan_deg;
    } else if (scan_mode == 2) {
        min_scan_deg = 0.0f;
        max_scan_deg = 0.0f;
    } else if (scan_mode == 3) {
        min_scan_deg = rfloat(30, 60, 1.0f);
        max_scan_deg = rfloat(5, 25, 1.0f); // lo > hi (maxA - minA < 0 -> no-advance guard!)
    } else {
        min_scan_deg = rfloat(1, 20, 1.0f);
        max_scan_deg = rfloat(25, 90, 1.0f);
    }

    uint32_t min_iv_ms = 0;
    uint32_t max_iv_ms = 0;
    int iv_mode = static_cast<int>(seed % 5);
    if (iv_mode == 0) {
        min_iv_ms = 100u + (oracle::splitmix32(rng) % 5u) * 34u;
        max_iv_ms = min_iv_ms + 67u + (oracle::splitmix32(rng) % 6u) * 34u;
    } else if (iv_mode == 1) {
        min_iv_ms = 167u;
        max_iv_ms = 167u;
    } else if (iv_mode == 2) {
        min_iv_ms = 334u;
        max_iv_ms = 100u; // lo > hi no-advance guard!
    } else {
        min_iv_ms = 67u;
        max_iv_ms = 267u;
    }

    uint32_t recenter_ms = 100u + (oracle::splitmix32(rng) % 8u) * 50u;

    std::ostringstream ss;
    ss << "  TurretTurnRate = " << turn_rate_deg << "\n";
    ss << "  TurretPitchRate = " << pitch_rate_deg << "\n";
    ss << "  NaturalTurretAngle = " << nat_angle_deg << "\n";
    ss << "  NaturalTurretPitch = " << nat_pitch_deg << "\n";
    ss << "  FirePitch = " << fire_pitch_deg << "\n";
    ss << "  MinPhysicalPitch = " << min_pitch_deg << "\n";
    ss << "  GroundUnitPitch = " << ground_pitch_deg << "\n";
    ss << "  TurretFireAngleSweep = PRIMARY " << sweep0 << "\n";
    ss << "  TurretFireAngleSweep = SECONDARY " << sweep1 << "\n";
    ss << "  TurretFireAngleSweep = TERTIARY " << sweep2 << "\n";
    ss << "  TurretSweepSpeedModifier = PRIMARY " << sspd0 << "\n";
    ss << "  TurretSweepSpeedModifier = SECONDARY " << sspd1 << "\n";
    ss << "  TurretSweepSpeedModifier = TERTIARY " << sspd2 << "\n";
    ss << "  ControlledWeaponSlots =";
    for (int i = 0; i < WEAPONSLOT_COUNT; ++i) {
        if ((out_slots_mask >> i) & 1u) ss << " " << kWeaponSlots[i];
    }
    ss << "\n";
    ss << "  AllowsPitch = " << (out_allows_pitch ? "Yes" : "No") << "\n";
    ss << "  MinIdleScanAngle = " << min_scan_deg << "\n";
    ss << "  MaxIdleScanAngle = " << max_scan_deg << "\n";
    ss << "  MinIdleScanInterval = " << min_iv_ms << "\n";
    ss << "  MaxIdleScanInterval = " << max_iv_ms << "\n";
    ss << "  RecenterTime = " << recenter_ms << "\n";
    ss << "  InitiallyDisabled = " << (out_init_disabled ? "Yes" : "No") << "\n";
    ss << "  FiresWhileTurning = " << (out_fires_while_turning ? "Yes" : "No") << "\n";
    return ss.str();
}

static void parse_turret_ini_fields(INI* ini, TurretAIData* td) {
    MultiIniFieldParse p;
    TurretAIData::buildFieldParse(p);
    ini->initFromINIMulti(td, p);
}

int main(int argc, char** argv) {
    const char* out_path = (argc > 1) ? argv[1] : "docs/golden_vectors/PKT-145.json";
    oracle::init_memory();

    TheWritableGlobalData = NEW GlobalData;
    TheWritableGlobalData->init();
    TheWritableGlobalData->m_headless = TRUE;
    TheWritableGlobalData->m_partitionCellSize = 20.0f;

    TheNameKeyGenerator = NEW NameKeyGenerator;
    TheNameKeyGenerator->init();

    TheAudio = new NullAudioManager;
    TheAudio->init();

    TheModuleFactory = NEW ModuleFactory;
    TheModuleFactory->init();

    TheScienceStore = NEW ScienceStore;
    TheScienceStore->init();

    TheRankInfoStore = NEW RankInfoStore;
    TheRankInfoStore->init();

    TheMultiplayerSettings = NEW MultiplayerSettings;
    TheMultiplayerSettings->init();

    TheSidesList = NEW SidesList;
    TheSidesList->init();

    TheTeamFactory = NEW TeamFactory;
    TheTeamFactory->init();

    ThePlayerTemplateStore = NEW PlayerTemplateStore;
    ThePlayerTemplateStore->init();

    ThePlayerList = NEW PlayerList;
    ThePlayerList->init();

    TheWeaponStore = NEW WeaponStore;
    TheWeaponStore->init();

    TheLocomotorStore = NEW LocomotorStore;
    TheLocomotorStore->init();

    TheArmorStore = NEW ArmorStore;
    TheArmorStore->init();

    TheDamageFXStore = NEW DamageFXStore;
    TheDamageFXStore->init();

    TheFXListStore = NEW FXListStore;
    TheFXListStore->init();

    TheSpecialPowerStore = NEW SpecialPowerStore;
    TheSpecialPowerStore->init();

    TheObjectCreationListStore = NEW ObjectCreationListStore;
    TheObjectCreationListStore->init();

    TheUpgradeCenter = NEW UpgradeCenter;
    TheUpgradeCenter->init();

    TheThingFactory = NEW ThingFactory;
    TheThingFactory->init();

    TheAI = NEW AI;
    TheAI->init();

    TheTerrainLogic = NEW TerrainLogic;
    TheTerrainLogic->init();

    TheGhostObjectManager = NEW GhostObjectManagerDummy;

    ThePartitionManager = NEW PartitionManager;
    ThePartitionManager->init();

    TheScriptEngine = NEW ScriptEngine;
    TheScriptEngine->init();

    TheRadar = NEW RadarDummy;
    TheRadar->init();

    TheMappedImageCollection = NEW ImageCollection;
    TheMappedImageCollection->init();

    TheAnim2DCollection = NEW Anim2DCollection;

    TheGameState = NEW GameState;

    TheDisplayStringManager = new NullDisplayStringManager;
    TheGameClient = new NullGameClient;
    TheInGameUI = new NullInGameUI;
    TheControlBar = NEW ControlBar;

    TheGameLogic = NEW GameLogic;

    // Create two distinct teams with enemy relationship so attack checks work cleanly.
    Player* p_a = ThePlayerList->getNthPlayer(0);
    Player* p_b = ThePlayerList->getNthPlayer(1);
    if (p_b) {
        p_b->init(nullptr);
    }
    if (p_a && p_b) {
        p_a->setPlayerRelationship(p_b, ENEMIES);
        p_b->setPlayerRelationship(p_a, ENEMIES);
    }
    Dict team_dict_a, team_dict_b;
    TheTeamFactory->initTeam("TeamA", AsciiString::TheEmptyString, TRUE, &team_dict_a);
    TheTeamFactory->initTeam("TeamB", AsciiString::TheEmptyString, TRUE, &team_dict_b);
    Team* team_a = TheTeamFactory->findTeam("TeamA");
    Team* team_b = TheTeamFactory->findTeam("TeamB");
    if (team_a && p_a) team_a->setControllingPlayer(p_a);
    if (team_b && p_b) team_b->setControllingPlayer(p_b);

    const char* h_turret = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/TurretAI.h";
    const char* cpp_turret = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/AI/TurretAI.cpp";

    oracle::MemINI mem_ini;
    std::string s0_diag;
    std::ostringstream vecs;

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed ^ 0x1457a1c3u;
        if (rng == 0) rng = 0x13572468u;

        uint32_t slots_mask = 1u;
        bool allows_pitch = false;
        bool fires_while_turning = false;
        bool init_disabled = false;
        std::string turret_fields = build_turret_ini(rng, seed, slots_mask, allows_pitch, fires_while_turning, init_disabled);

        // Parse standalone TurretAIData to inspect all 18 parsed fields directly.
        TurretAIData* parsed_td = newInstance(TurretAIData);
        {
            std::string td_block = "TestTurretData\n" + turret_fields + "End\n";
            mem_ini.parse_blocks(td_block, INI_LOAD_OVERWRITE, [&](INI* ini, const char* tok) {
                if (std::strcmp(tok, "TestTurretData") == 0) {
                    parse_turret_ini_fields(ini, parsed_td);
                }
            });
        }

        // Define weapons for this seed.
        Real atk_range = static_cast<Real>(120 + (oracle::splitmix32(rng) % 280));
        Real min_range = (seed % 6 == 0) ? 20.0f : 0.0f;
        int clip_sz = 1 + static_cast<int>(oracle::splitmix32(rng) % 4);
        int delay_ms = 67 + static_cast<int>((oracle::splitmix32(rng) % 5) * 34);
        int reload_ms = 200 + static_cast<int>((oracle::splitmix32(rng) % 6) * 50);
        int pre_atk_ms = (seed % 4 == 0) ? 100 : 0;
        std::string wname = "TurWpn_" + std::to_string(seed);
        std::string w_ini =
            "Weapon " + wname + "\n"
            "  PrimaryDamage = 15\n"
            "  AttackRange = " + std::to_string(static_cast<int>(atk_range)) + "\n"
            "  MinimumAttackRange = " + std::to_string(static_cast<int>(min_range)) + "\n"
            "  DamageType = SMALL_ARMS\n"
            "  DeathType = NORMAL\n"
            "  WeaponSpeed = 999999\n"
            "  ClipSize = " + std::to_string(clip_sz) + "\n"
            "  ClipReloadTime = " + std::to_string(reload_ms) + "\n"
            "  DelayBetweenShots = " + std::to_string(delay_ms) + "\n"
            "  PreAttackDelay = " + std::to_string(pre_atk_ms) + "\n"
            "  AutoReloadsClip = YES\n"
            "  AntiGround = YES\n"
            "  AntiAirborneVehicle = YES\n"
            "  AntiAirborneInfantry = YES\n"
            "End\n";
        mem_ini.parse_blocks(w_ini, INI_LOAD_OVERWRITE, [](INI* ini, const char* tok) {
            if (std::strcmp(tok, "Weapon") == 0) WeaponStore::parseWeaponTemplateDefinition(ini);
        });

        // Define Attacker Object template with AIUpdateInterface + Turret (or AltTurret).
        bool use_alt_turret = (seed % 7 == 3);
        bool turrets_linked = (seed % 8 == 5);
        int cur_slot_idx = static_cast<int>(seed % WEAPONSLOT_COUNT);
        uint32_t mood_check_ms = 100u + (oracle::splitmix32(rng) % 5u) * 34u;
        std::string oname = "TurUnit_" + std::to_string(seed);
        std::string vname = "TurTarget_" + std::to_string(seed);
        bool victim_immobile = (seed % 3 == 0);

        std::ostringstream obj_ini;
        obj_ini << "Object " << oname << "\n"
                << "  KindOf = VEHICLE CAN_ATTACK SELECTABLE\n"
                << "  Geometry = BOX\n"
                << "  GeometryMajorRadius = 15\n"
                << "  GeometryMinorRadius = 10\n"
                << "  GeometryHeight = 20\n"
                << "  WeaponSet\n"
                << "    Conditions = None\n"
                << "    Weapon = PRIMARY " << wname << "\n"
                << "    Weapon = SECONDARY " << wname << "\n"
                << "    Weapon = TERTIARY " << wname << "\n"
                << "  End\n"
                << "  Body = ActiveBody ModuleTag_Body\n"
                << "    MaxHealth = 500\n"
                << "    InitialHealth = 500\n"
                << "  End\n"
                << "  Behavior = AIUpdateInterface ModuleTag_AI\n"
                << "    MoodAttackCheckRate = " << mood_check_ms << "\n"
                << "    TurretsLinked = " << (turrets_linked ? "Yes" : "No") << "\n"
                << "    " << (use_alt_turret ? "AltTurret" : "Turret") << "\n"
                << turret_fields
                << "    End\n"
                << "  End\n"
                << "End\n"
                << "Object " << vname << "\n"
                << "  KindOf = " << (victim_immobile ? "STRUCTURE IMMOBILE" : "VEHICLE") << " CAN_ATTACK\n"
                << "  Geometry = CYLINDER\n"
                << "  GeometryMajorRadius = 12\n"
                << "  GeometryHeight = 14\n"
                << "  Body = ActiveBody ModuleTag_Body\n"
                << "    MaxHealth = 5000\n"
                << "    InitialHealth = 5000\n"
                << "  End\n"
                << "End\n";

        mem_ini.parse_blocks(obj_ini.str(), INI_LOAD_OVERWRITE, [](INI* ini, const char* tok) {
            if (std::strcmp(tok, "Object") == 0) {
                AsciiString name = ini->getNextToken();
                ThingFactory::parseObjectDefinition(ini, name, AsciiString::TheEmptyString);
            }
        });

        const ThingTemplate* attacker_tt = TheThingFactory->findTemplate(oname.c_str());
        const ThingTemplate* victim_tt = TheThingFactory->findTemplate(vname.c_str());

        uint32_t rand_seed = 0x5000u + seed * 7919u;
        UnsignedInt frame0 = (seed == 0) ? 100u : (10u + (oracle::splitmix32(rng) % 2000u));
        TheGameLogic->*access_member(GLFrameTag()) = frame0;

        Coord3D owner_pos{
            static_cast<Real>(static_cast<int32_t>(oracle::splitmix32(rng) % 200) - 100),
            static_cast<Real>(static_cast<int32_t>(oracle::splitmix32(rng) % 200) - 100),
            static_cast<Real>(oracle::splitmix32(rng) % 40)
        };
        Real owner_angle = static_cast<Real>(static_cast<int32_t>(oracle::splitmix32(rng) % 360) - 180) * (PI / 180.0f);

        Coord3D target_pos{
            owner_pos.x + static_cast<Real>(static_cast<int32_t>(oracle::splitmix32(rng) % 300) - 150),
            owner_pos.y + static_cast<Real>(static_cast<int32_t>(oracle::splitmix32(rng) % 300) - 150),
            owner_pos.z + static_cast<Real>(static_cast<int32_t>(oracle::splitmix32(rng) % 80) - 20)
        };
        // Avoid zero distance on target_pos unless deliberately testing zero vec on seed % 17 == 0.
        if (seed % 17 == 0) {
            target_pos = owner_pos;
        } else if (std::abs(target_pos.x - owner_pos.x) < 10.0f && std::abs(target_pos.y - owner_pos.y) < 10.0f) {
            target_pos.x = owner_pos.x + 80.0f;
        }

        Coord3D victim_pos{
            owner_pos.x + static_cast<Real>(static_cast<int32_t>(oracle::splitmix32(rng) % 240) - 120),
            owner_pos.y + static_cast<Real>(static_cast<int32_t>(oracle::splitmix32(rng) % 240) - 120),
            owner_pos.z + static_cast<Real>(static_cast<int32_t>(oracle::splitmix32(rng) % 60) - 10)
        };
        if (std::abs(victim_pos.x - owner_pos.x) < 15.0f && std::abs(victim_pos.y - owner_pos.y) < 15.0f) {
            victim_pos.y = owner_pos.y + 75.0f;
        }

        Object* attacker = TheThingFactory->newObject(attacker_tt, team_a);
        Object* victim = TheThingFactory->newObject(victim_tt, team_b);
        attacker->setPosition(&owner_pos);
        attacker->setOrientation(owner_angle);
        victim->setPosition(&victim_pos);

        // Reset GameLogic PRNG right before standalone TurretAI construction so `tai`
        // begins from a clean, unpolluted `InitRandom(rand_seed)` state.
        InitRandom(rand_seed);

        // Construct a standalone TurretAI directly on `attacker` so we have direct pointer & can inspect everything!
        WhichTurretType tur = use_alt_turret ? TURRET_ALT : TURRET_MAIN;
        TurretAI* tai = newInstance(TurretAI)(attacker, parsed_td, tur);

        UnsignedInt rcrc_init = GetGameLogicRandomSeedCRC();
        TurretStateMachine* sm = tai->*access_member(TurretSMTag());
        Int init_state = static_cast<Int>(sm->getCurrentStateID());
        uint32_t init_angle = oracle::fbits(tai->getTurretAngle());
        uint32_t init_pitch = oracle::fbits(tai->getTurretPitch());
        UnsignedInt init_sleep_until = tai->*access_member(TurretSleepUntilTag());

        // Query slot helpers
        uint32_t slot_ok_mask = 0;
        for (int s = 0; s < WEAPONSLOT_COUNT; ++s) {
            if (tai->isWeaponSlotOnTurret(static_cast<WeaponSlotType>(s))) slot_ok_mask |= (1u << s);
        }
        int init_natural = tai->isTurretInNaturalPosition() ? 1 : 0;

        // Run 20 deterministic simulation ticks driving all turret states & edges.
        const int NUM_TICKS = 20;
        Int tick_state[NUM_TICKS];
        uint32_t tick_angle[NUM_TICKS];
        uint32_t tick_pitch[NUM_TICKS];
        uint32_t tick_sleep[NUM_TICKS];
        uint32_t tick_rcrc[NUM_TICKS];

        std::vector<uint8_t> xfer_bytes;
        uint32_t xfer_crc_val = 0;
        uint32_t xfer_save_crc = 0;
        Int post_load_state = -1;
        uint32_t post_load_angle = 0;
        uint32_t post_load_pitch = 0;

        bool force_atk = (seed % 2 == 1);
        int mode = static_cast<int>(seed % 4);

        for (int t = 0; t < NUM_TICKS; ++t) {
            TheGameLogic->*access_member(GLFrameTag()) = frame0 + 1u + static_cast<UnsignedInt>(t);

            // Inject deterministic events at specific ticks depending on mode.
            if (init_disabled && t == 2) {
                tai->setTurretEnabled(TRUE);
            }
            if (mode == 0) {
                // Idle -> IdleScan -> TargetPosition -> Aim -> Fire -> Clear -> Hold -> Recenter
                if (t == 6) tai->setTurretTargetPosition(&target_pos);
                if (t == 13) tai->setTurretTargetPosition(nullptr);
            } else if (mode == 1) {
                // TargetObject -> Aim -> Fire -> Team change or kill at t=14
                if (t == 2) tai->setTurretTargetObject(victim, force_atk ? TRUE : FALSE);
                if (t == 14) {
                    if (seed % 8 == 1) {
                        victim->setTeam(team_a);
                    } else {
                        tai->setTurretTargetObject(nullptr, FALSE);
                    }
                }
            } else if (mode == 2) {
                // Under construction gating + recenterTurret + disable/enable
                if (t == 0) attacker->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
                if (t == 3) attacker->clearStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
                if (t == 5) tai->setTurretTargetPosition(&target_pos);
                if (t == 9) tai->recenterTurret();
                if (t == 12) tai->setTurretEnabled(FALSE);
                if (t == 15) tai->setTurretEnabled(TRUE);
            } else {
                // Pure Idle -> IdleScan -> Hold -> Recenter -> Aim
                if (t == 13) tai->setTurretTargetObject(victim, FALSE);
            }

            // Force notifyFired on tick 8 if in FIRE/AIM to test sweep window.
            if (t == 8 && (seed % 3 == 0)) {
                tai->notifyFired();
            }

            UpdateSleepTime slp = tai->updateTurretAI();
            tick_state[t] = static_cast<Int>(sm->getCurrentStateID());
            tick_angle[t] = oracle::fbits(tai->getTurretAngle());
            tick_pitch[t] = oracle::fbits(tai->getTurretPitch());
            tick_sleep[t] = static_cast<uint32_t>(slp);
            tick_rcrc[t] = GetGameLogicRandomSeedCRC();

            if (t == 10) {
                XferCRC xcrc;
                xcrc.open("turret_crc");
                oracle::call_snapshot_crc(tai, &xcrc);
                xcrc.close();
                xfer_crc_val = xcrc.getCRC();

                oracle::MemoryXfer xsave(XFER_SAVE);
                xsave.open("ts");
                oracle::call_snapshot_xfer(tai, &xsave);
                xsave.close();
                xfer_bytes = xsave.bytes;
                xfer_save_crc = xsave.bytes_crc();
            }
        }

        // Verify C++ save/load round trip AFTER the 20-tick simulation loop so PRNG is unpolluted.
        {
            TurretAI* tai2 = newInstance(TurretAI)(attacker, parsed_td, tur);
            oracle::MemoryXfer xload(XFER_LOAD);
            xload.bytes = xfer_bytes;
            xload.open("tl");
            oracle::call_snapshot_xfer(tai2, &xload);
            xload.close();

            oracle::MemoryXfer xresave(XFER_SAVE);
            xresave.open("tr");
            oracle::call_snapshot_xfer(tai2, &xresave);
            xresave.close();
            if (xresave.bytes != xfer_bytes) {
                std::fprintf(stderr, "TurretAI Xfer round-trip mismatch on seed %u\n", seed);
                return 3;
            }
            post_load_state = static_cast<Int>((tai2->*access_member(TurretSMTag()))->getCurrentStateID());
            post_load_angle = oracle::fbits(tai2->getTurretAngle());
            post_load_pitch = oracle::fbits(tai2->getTurretPitch());
            deleteInstance(tai2);
        }

        // Encode hex of xfer_bytes
        static const char* kHex = "0123456789abcdef";
        std::string xhex;
        xhex.reserve(xfer_bytes.size() * 2);
        for (uint8_t b : xfer_bytes) {
            xhex.push_back(kHex[(b >> 4) & 0xf]);
            xhex.push_back(kHex[b & 0xf]);
        }

        auto write_arr_u32 = [&](std::ostringstream& s, const uint32_t* a, int n) {
            s << "[";
            for (int i = 0; i < n; ++i) {
                if (i) s << ",";
                s << a[i];
            }
            s << "]";
        };
        auto write_arr_i32 = [&](std::ostringstream& s, const Int* a, int n) {
            s << "[";
            for (int i = 0; i < n; ++i) {
                if (i) s << ",";
                s << a[i];
            }
            s << "]";
        };

        uint32_t td_bits[15] = {
            oracle::fbits(parsed_td->m_turnRate),
            oracle::fbits(parsed_td->m_pitchRate),
            oracle::fbits(parsed_td->m_naturalTurretAngle),
            oracle::fbits(parsed_td->m_naturalTurretPitch),
            oracle::fbits(parsed_td->m_firePitch),
            oracle::fbits(parsed_td->m_minPitch),
            oracle::fbits(parsed_td->m_groundUnitPitch),
            oracle::fbits(parsed_td->m_turretFireAngleSweep[0]),
            oracle::fbits(parsed_td->m_turretFireAngleSweep[1]),
            oracle::fbits(parsed_td->m_turretFireAngleSweep[2]),
            oracle::fbits(parsed_td->m_turretSweepSpeedModifier[0]),
            oracle::fbits(parsed_td->m_turretSweepSpeedModifier[1]),
            oracle::fbits(parsed_td->m_turretSweepSpeedModifier[2]),
            oracle::fbits(parsed_td->m_minIdleScanAngle),
            oracle::fbits(parsed_td->m_maxIdleScanAngle),
        };
        uint32_t td_ints[7] = {
            parsed_td->m_turretWeaponSlots,
            parsed_td->m_minIdleScanInterval,
            parsed_td->m_maxIdleScanInterval,
            parsed_td->m_recenterTime,
            parsed_td->m_initiallyDisabled ? 1u : 0u,
            parsed_td->m_firesWhileTurning ? 1u : 0u,
            parsed_td->m_isAllowsPitch ? 1u : 0u,
        };
        uint32_t o_pos[4] = { oracle::fbits(owner_pos.x), oracle::fbits(owner_pos.y), oracle::fbits(owner_pos.z), oracle::fbits(owner_angle) };
        uint32_t t_pos[3] = { oracle::fbits(target_pos.x), oracle::fbits(target_pos.y), oracle::fbits(target_pos.z) };
        uint32_t v_pos[3] = { oracle::fbits(victim_pos.x), oracle::fbits(victim_pos.y), oracle::fbits(victim_pos.z) };

        if (seed == 0) {
            std::ostringstream d;
            d << "  \"seed_0_diagnostics\": {\n"
              << "    \"turn_rate_bits\": " << td_bits[0] << ",\n"
              << "    \"pitch_rate_bits\": " << td_bits[1] << ",\n"
              << "    \"natural_angle_bits\": " << td_bits[2] << ",\n"
              << "    \"natural_pitch_bits\": " << td_bits[3] << ",\n"
              << "    \"weapon_slots_mask\": " << td_ints[0] << ",\n"
              << "    \"recenter_time_frames\": " << td_ints[3] << ",\n"
              << "    \"init_state_id\": " << init_state << ",\n"
              << "    \"init_sleep_until\": " << init_sleep_until << ",\n"
              << "    \"init_rand_crc\": " << rcrc_init << ",\n"
              << "    \"xfer_len\": " << xfer_bytes.size() << ",\n"
              << "    \"xfer_save_crc\": " << xfer_save_crc << ",\n"
              << "    \"xfer_hex\": \"" << xhex << "\"\n"
              << "  },\n";
            s0_diag = d.str();
        }

        vecs << "    {\"seed\":" << seed
             << ",\"ini\":\"" << oracle::json_escape(turret_fields) << "\""
             << ",\"td_bits\":";
        write_arr_u32(vecs, td_bits, 15);
        vecs << ",\"td_ints\":";
        write_arr_u32(vecs, td_ints, 7);
        vecs << ",\"tur\":" << static_cast<Int>(tur)
             << ",\"mode\":" << mode
             << ",\"linked\":" << (turrets_linked ? 1 : 0)
             << ",\"force\":" << (force_atk ? 1 : 0)
             << ",\"v_immob\":" << (victim_immobile ? 1 : 0)
             << ",\"cur_slot\":" << cur_slot_idx
             << ",\"atk_rng\":" << oracle::fbits(atk_range)
             << ",\"min_rng\":" << oracle::fbits(min_range)
             << ",\"wpn_p\":[" << clip_sz << "," << delay_ms << "," << reload_ms << "," << pre_atk_ms << "]"
             << ",\"mood_ms\":" << mood_check_ms
             << ",\"rand_seed\":" << rand_seed
             << ",\"frame0\":" << frame0
             << ",\"o_pos\":";
        write_arr_u32(vecs, o_pos, 4);
        vecs << ",\"t_pos\":";
        write_arr_u32(vecs, t_pos, 3);
        vecs << ",\"v_pos\":";
        write_arr_u32(vecs, v_pos, 3);
        vecs << ",\"rcrc_init\":" << rcrc_init
             << ",\"init_state\":" << init_state
             << ",\"init_angle\":" << init_angle
             << ",\"init_pitch\":" << init_pitch
             << ",\"init_sleep_until\":" << init_sleep_until
             << ",\"slot_ok_mask\":" << slot_ok_mask
             << ",\"init_natural\":" << init_natural
             << ",\"tick_state\":";
        write_arr_i32(vecs, tick_state, NUM_TICKS);
        vecs << ",\"tick_angle\":";
        write_arr_u32(vecs, tick_angle, NUM_TICKS);
        vecs << ",\"tick_pitch\":";
        write_arr_u32(vecs, tick_pitch, NUM_TICKS);
        vecs << ",\"tick_sleep\":";
        write_arr_u32(vecs, tick_sleep, NUM_TICKS);
        vecs << ",\"tick_rcrc\":";
        write_arr_u32(vecs, tick_rcrc, NUM_TICKS);
        vecs << ",\"xcrc\":" << xfer_crc_val
             << ",\"xlen\":" << xfer_bytes.size()
             << ",\"xsave_crc\":" << xfer_save_crc
             << ",\"xhex\":\"" << xhex << "\""
             << ",\"post_state\":" << post_load_state
             << ",\"post_angle\":" << post_load_angle
             << ",\"post_pitch\":" << post_load_pitch
             << "}";
        if (seed + 1 < 1000) vecs << ",";
        vecs << "\n";

        deleteInstance(tai);
        deleteInstance(parsed_td);
    }

    std::ostringstream out;
    out << "{\n"
        << "  \"packet_id\": \"PKT-145\",\n"
        << "  \"module\": \"TurretAI\",\n"
        << "  \"vector_count\": 1000,\n"
        << "  \"retail_compatible_xfer_save\": " << RETAIL_COMPATIBLE_XFER_SAVE << ",\n"
        << "  \"source_sha256\": {\n"
        << "    \"" << h_turret << "\": \"" << oracle::sha256_file(h_turret) << "\",\n"
        << "    \"" << cpp_turret << "\": \"" << oracle::sha256_file(cpp_turret) << "\"\n"
        << "  },\n"
        << s0_diag
        << "  \"vectors\": [\n"
        << vecs.str()
        << "  ]\n"
        << "}\n";

    oracle::write_file_atomic(out_path, out.str());
    std::printf("Wrote 1000 PKT-145 golden vectors to %s\n", out_path);
    return 0;
}
