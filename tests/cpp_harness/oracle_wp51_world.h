#pragma once
// Shared zero-stub headless Zero Hour simulation world fixture for WP-5.1..WP-5.x C++
// Golden Vector generators built against pre-built libz_gameengine.a via scripts/build_oracle.py.
//
// Usage:
//   #include "oracle_wp51_world.h"
//   int main(int argc, char** argv) {
//       oracle_wp51::World w;
//       w.load_ini("...");
//       Team* t = w.make_team("TeamA", 0);
//       Object* obj = w.spawn_object("MyUnit", t, {0,0,0}, 0.0f);
//   }

#include "oracle_common.h"

#include <cstdarg>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

#include "Common/BuildAssistant.h"
#include "Common/DamageFX.h"
#include "Common/Dict.h"
#include "Common/GameAudio.h"
#include "Common/GameLOD.h"
#include "Common/GameState.h"
#include "Common/GlobalData.h"
#include "Common/MessageStream.h"
#include "Common/ModuleFactory.h"
#include "Common/MultiplayerSettings.h"
#include "Common/NameKeyGenerator.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/PlayerTemplate.h"
#include "Common/Radar.h"
#include "Common/RandomValue.h"
#include "Common/ResourceGatheringManager.h"
#include "Common/Science.h"
#include "Common/SpecialPower.h"
#include "Common/Team.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/TunnelTracker.h"
#include "Common/Upgrade.h"
#include "GameClient/Anim2D.h"
#include "GameClient/ControlBar.h"
#include "GameClient/DisplayStringManager.h"
#include "GameClient/Drawable.h"
#include "GameClient/Eva.h"
#include "GameClient/FXList.h"
#include "GameClient/GameClient.h"
#include "GameClient/GameText.h"
#include "GameClient/Image.h"
#include "GameClient/InGameUI.h"
#include "GameClient/ParticleSys.h"
#include "GameClient/RayEffect.h"
#include "GameClient/SelectionXlat.h"
#include "GameClient/Snow.h"
#include "GameClient/TerrainRoads.h"
#include "GameClient/Water.h"
#include "GameLogic/AI.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/Armor.h"
#include "GameLogic/CaveSystem.h"
#include "GameLogic/CrateSystem.h"
#include "GameLogic/Damage.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/GhostObject.h"
#include "GameLogic/Locomotor.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectCreationList.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/RankInfo.h"
#include "GameLogic/ScriptEngine.h"
#include "GameLogic/SidesList.h"
#include "GameLogic/TerrainLogic.h"
#include "GameLogic/VictoryConditions.h"
#include "GameLogic/Weapon.h"
#include "GameLogic/WeaponSet.h"

ORACLE_DEFINE_LINK_GLOBALS("wp51_oracle")

struct Wp51GLFrameTag { using type = UnsignedInt GameLogic::*; };
template struct MemberAccess<Wp51GLFrameTag, &GameLogic::m_frame>;
inline UnsignedInt GameLogic::* access_member(Wp51GLFrameTag);

struct Wp51PlayerRgmTag { using type = ResourceGatheringManager* Player::*; };
template struct MemberAccess<Wp51PlayerRgmTag, &Player::m_resourceGatheringManager>;
inline ResourceGatheringManager* Player::* access_member(Wp51PlayerRgmTag);

struct Wp51PlayerListCountTag { using type = Int PlayerList::*; };
template struct MemberAccess<Wp51PlayerListCountTag, &PlayerList::m_playerCount>;
inline Int PlayerList::* access_member(Wp51PlayerListCountTag);

struct Wp51PlayerDefaultTeamTag { using type = Team* Player::*; };
template struct MemberAccess<Wp51PlayerDefaultTeamTag, &Player::m_defaultTeam>;
inline Team* Player::* access_member(Wp51PlayerDefaultTeamTag);

struct Wp51PlayerTunnelTag { using type = TunnelTracker* Player::*; };
template struct MemberAccess<Wp51PlayerTunnelTag, &Player::m_tunnelSystem>;
inline TunnelTracker* Player::* access_member(Wp51PlayerTunnelTag);

namespace oracle_wp51 {

class OracleTerrainLogic : public TerrainLogic {
public:
    Region3D extent;

    OracleTerrainLogic() {
        extent.lo = {0.0f, 0.0f, 0.0f};
        extent.hi = {1000.0f, 1000.0f, 100.0f};
    }

    void getExtent(Region3D* out) const override {
        if (out) *out = extent;
    }

    void getExtentIncludingBorder(Region3D* out) const override {
        if (out) *out = extent;
    }

    void getMaximumPathfindExtent(Region3D* out) const override {
        if (out) *out = extent;
    }

    Real getGroundHeight(Real, Real, Coord3D* normal = nullptr) const override {
        if (normal) { normal->x = 0.0f; normal->y = 0.0f; normal->z = 1.0f; }
        return 0.0f;
    }

    Bool isCliffCell(Real, Real) const override { return FALSE; }

    Bool isUnderwater(Real, Real, Real* waterZ = nullptr, Real* terrainZ = nullptr) override {
        if (waterZ) *waterZ = 0.0f;
        if (terrainZ) *terrainZ = 0.0f;
        return FALSE;
    }
};

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
    // UI text needs `TheGlobalLanguageData`, `TheDisplay` and fonts, none of which exist
    // headless. Without these `Radar::tryUnderAttackEvent` segfaults when a structure is damaged.
    void message(UnicodeString, ...) override {}
    void message(AsciiString, ...) override {}
    void messageColor(const RGBColor*, UnicodeString, ...) override {}
    void messageNoFormat(const UnicodeString&) override {}
    void messageNoFormat(const RGBColor*, const UnicodeString&) override {}
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

class NullGameText : public GameTextInterface {
public:
    AsciiStringVec empty_vec;
    void init() override {}
    void reset() override {}
    void update() override {}
    UnicodeString fetch(const Char*, Bool* exists = nullptr) override {
        if (exists) *exists = FALSE;
        return UnicodeString::TheEmptyString;
    }
    UnicodeString fetch(AsciiString, Bool* exists = nullptr) override {
        if (exists) *exists = FALSE;
        return UnicodeString::TheEmptyString;
    }
    UnicodeString fetchFormat(const Char*, ...) override { return UnicodeString::TheEmptyString; }
    UnicodeString fetchOrSubstitute(const Char*, const WideChar* sub) override {
        return sub ? UnicodeString(sub) : UnicodeString::TheEmptyString;
    }
    UnicodeString fetchOrSubstituteFormat(const Char*, const WideChar*, ...) override {
        return UnicodeString::TheEmptyString;
    }
    UnicodeString fetchOrSubstituteFormatVA(const Char*, const WideChar*, va_list) override {
        return UnicodeString::TheEmptyString;
    }
    AsciiStringVec& getStringsWithLabelPrefix(AsciiString) override { return empty_vec; }
    void initMapStringFile(const AsciiString&) override {}
};

struct World {
    oracle::MemINI mem_ini;
    Team* team_a = nullptr;
    Team* team_b = nullptr;
    Team* team_neutral = nullptr;

    World() {
        oracle::init_memory();

        TheWritableGlobalData = NEW GlobalData;
        TheWritableGlobalData->init();
        TheWritableGlobalData->m_headless = TRUE;
        TheWritableGlobalData->m_partitionCellSize = 20.0f;

        TheGameLODManager = NEW GameLODManager;

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

        TheTerrainRoads = NEW TerrainRoadCollection;
        TheTerrainRoads->init();

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

        TheCrateSystem = NEW CrateSystem;
        TheCrateSystem->init();

        TheCaveSystem = NEW CaveSystem;
        TheCaveSystem->init();

        TheBuildAssistant = NEW BuildAssistant;
        TheBuildAssistant->init();

        TheAI = NEW AI;
        TheAI->init();

        TheTerrainLogic = NEW OracleTerrainLogic;
        TheTerrainLogic->init();

        TheGhostObjectManager = NEW GhostObjectManagerDummy;

        ThePartitionManager = NEW PartitionManager;
        ThePartitionManager->init();

        TheScriptEngine = NEW ScriptEngine;
        TheScriptEngine->init();

        TheRadar = NEW RadarDummy;
        TheRadar->init();

        TheParticleSystemManager = NEW ParticleSystemManagerDummy;
        TheParticleSystemManager->init();

        TheMappedImageCollection = NEW ImageCollection;
        TheMappedImageCollection->init();

        TheAnim2DCollection = NEW Anim2DCollection;

        TheGameState = NEW GameState;

        TheDisplayStringManager = new NullDisplayStringManager;
        TheGameClient = new NullGameClient;
        TheInGameUI = new NullInGameUI;
        TheGameText = new NullGameText;
        TheGameText->init();
        TheMessageStream = NEW MessageStream;
        TheMessageStream->init();
        TheSelectionTranslator = NEW SelectionTranslator;
        TheVictoryConditions = createVictoryConditions();
        TheVictoryConditions->init();
        TheControlBar = NEW ControlBar;
        TheRayEffects = NEW RayEffectSystem;
        TheRayEffects->init();
        TheEva = NEW Eva;
        TheEva->reset();
        TheWaterTransparency = newInstance(WaterTransparencySetting);
        TheWeatherSetting = newInstance(WeatherSetting);

        TheGameLogic = NEW GameLogic;
        TheGameLogic->reset();
        // `GameLogic::reset()` invokes `ThePartitionManager->reset()` which deletes `m_cells`;
        // re-initialize `PartitionManager` against `OracleTerrainLogic` for WP-5.2 spatial queries.
        ThePartitionManager->init();
        // `GameLogic::reset()` also resets `TheAI`; allocate pathfind cells against `OracleTerrainLogic`.
        TheAI->pathfinder()->newMap();

        Player* p_a = ThePlayerList->getNthPlayer(0);
        Player* p_b = ThePlayerList->getNthPlayer(1);
        ThePlayerList->*access_member(Wp51PlayerListCountTag()) = 2;
        if (p_a) {
            if (!p_a->getResourceGatheringManager()) {
                p_a->*access_member(Wp51PlayerRgmTag()) = newInstance(ResourceGatheringManager);
            }
            if (!p_a->getTunnelSystem()) {
                p_a->*access_member(Wp51PlayerTunnelTag()) = newInstance(TunnelTracker);
            }
        }
        if (p_b) {
            p_b->init(nullptr);
            if (!p_b->getResourceGatheringManager()) {
                p_b->*access_member(Wp51PlayerRgmTag()) = newInstance(ResourceGatheringManager);
            }
            if (!p_b->getTunnelSystem()) {
                p_b->*access_member(Wp51PlayerTunnelTag()) = newInstance(TunnelTracker);
            }
        }
        if (p_a && p_b) {
            p_a->setPlayerRelationship(p_b, ENEMIES);
            p_b->setPlayerRelationship(p_a, ENEMIES);
        }
        Dict dict_a, dict_b;
        TheTeamFactory->initTeam("TeamA", AsciiString::TheEmptyString, TRUE, &dict_a);
        TheTeamFactory->initTeam("TeamB", AsciiString::TheEmptyString, TRUE, &dict_b);
        team_a = TheTeamFactory->findTeam("TeamA");
        team_b = TheTeamFactory->findTeam("TeamB");
        if (team_a && p_a) {
            team_a->setControllingPlayer(p_a);
            p_a->*access_member(Wp51PlayerDefaultTeamTag()) = team_a;
        }
        if (team_b && p_b) {
            team_b->setControllingPlayer(p_b);
            p_b->*access_member(Wp51PlayerDefaultTeamTag()) = team_b;
        }
        set_frame(1);
    }

    void init_with_neutral_player(const char* tmpl_a = nullptr, const char* tmpl_b = nullptr) {
        Player* p_n = ThePlayerList->getNthPlayer(0);
        Player* p_a = ThePlayerList->getNthPlayer(1);
        Player* p_b = ThePlayerList->getNthPlayer(2);
        ThePlayerList->*access_member(Wp51PlayerListCountTag()) = 3;
        if (p_n) {
            p_n->init(nullptr);
            if (!p_n->getResourceGatheringManager()) {
                p_n->*access_member(Wp51PlayerRgmTag()) = newInstance(ResourceGatheringManager);
            }
            if (!p_n->getTunnelSystem()) {
                p_n->*access_member(Wp51PlayerTunnelTag()) = newInstance(TunnelTracker);
            }
            // `Player::init` leaves no default team, but retail always has one for the neutral
            // player. Without it `GarrisonContain::onObjectCreated` (InitialRoster) spawns
            // into `getControllingPlayer()->getDefaultTeam() == nullptr` and segfaults.
            team_neutral = p_n->getDefaultTeam();
            if (!team_neutral) {
                Dict dict_n;
                TheTeamFactory->initTeam("TeamNeutral", AsciiString::TheEmptyString, TRUE, &dict_n);
                team_neutral = TheTeamFactory->findTeam("TeamNeutral");
                if (team_neutral) {
                    team_neutral->setControllingPlayer(p_n);
                    p_n->*access_member(Wp51PlayerDefaultTeamTag()) = team_neutral;
                }
            }
            if (!p_n->getDefaultTeam()) {
                std::fprintf(stderr, "oracle: neutral player has no default team\n");
                std::abort();
            }
        }
        if (p_a) {
            const PlayerTemplate* pta =
                tmpl_a ? ThePlayerTemplateStore->findPlayerTemplate(NAMEKEY(tmpl_a)) : nullptr;
            p_a->init(pta);
            if (!p_a->getResourceGatheringManager()) {
                p_a->*access_member(Wp51PlayerRgmTag()) = newInstance(ResourceGatheringManager);
            }
            if (!p_a->getTunnelSystem()) {
                p_a->*access_member(Wp51PlayerTunnelTag()) = newInstance(TunnelTracker);
            }
            if (team_a) {
                team_a->setControllingPlayer(p_a);
                p_a->*access_member(Wp51PlayerDefaultTeamTag()) = team_a;
            }
        }
        if (p_b) {
            const PlayerTemplate* ptb =
                tmpl_b ? ThePlayerTemplateStore->findPlayerTemplate(NAMEKEY(tmpl_b)) : nullptr;
            p_b->init(ptb);
            if (!p_b->getResourceGatheringManager()) {
                p_b->*access_member(Wp51PlayerRgmTag()) = newInstance(ResourceGatheringManager);
            }
            if (!p_b->getTunnelSystem()) {
                p_b->*access_member(Wp51PlayerTunnelTag()) = newInstance(TunnelTracker);
            }
            if (team_b) {
                team_b->setControllingPlayer(p_b);
                p_b->*access_member(Wp51PlayerDefaultTeamTag()) = team_b;
            }
        }
        if (p_a && p_b) {
            p_a->setPlayerRelationship(p_b, ENEMIES);
            p_b->setPlayerRelationship(p_a, ENEMIES);
        }
        if (p_n && p_a) {
            p_a->setPlayerRelationship(p_n, NEUTRAL);
            p_n->setPlayerRelationship(p_a, NEUTRAL);
        }
        if (p_n && p_b) {
            p_b->setPlayerRelationship(p_n, NEUTRAL);
            p_n->setPlayerRelationship(p_b, NEUTRAL);
        }
    }

    void init_players(const char* tmpl_a, const char* tmpl_b) {
        Player* p_a = ThePlayerList->getNthPlayer(0);
        Player* p_b = ThePlayerList->getNthPlayer(1);
        if (p_a) {
            const PlayerTemplate* pta =
                tmpl_a ? ThePlayerTemplateStore->findPlayerTemplate(NAMEKEY(tmpl_a)) : nullptr;
            p_a->init(pta);
            if (!p_a->getResourceGatheringManager()) {
                p_a->*access_member(Wp51PlayerRgmTag()) = newInstance(ResourceGatheringManager);
            }
            if (!p_a->getTunnelSystem()) {
                p_a->*access_member(Wp51PlayerTunnelTag()) = newInstance(TunnelTracker);
            }
            if (team_a) {
                team_a->setControllingPlayer(p_a);
                p_a->*access_member(Wp51PlayerDefaultTeamTag()) = team_a;
            }
        }
        if (p_b) {
            const PlayerTemplate* ptb =
                tmpl_b ? ThePlayerTemplateStore->findPlayerTemplate(NAMEKEY(tmpl_b)) : nullptr;
            p_b->init(ptb);
            if (!p_b->getResourceGatheringManager()) {
                p_b->*access_member(Wp51PlayerRgmTag()) = newInstance(ResourceGatheringManager);
            }
            if (!p_b->getTunnelSystem()) {
                p_b->*access_member(Wp51PlayerTunnelTag()) = newInstance(TunnelTracker);
            }
            if (team_b) {
                team_b->setControllingPlayer(p_b);
                p_b->*access_member(Wp51PlayerDefaultTeamTag()) = team_b;
            }
        }
        if (p_a && p_b) {
            p_a->setPlayerRelationship(p_b, ENEMIES);
            p_b->setPlayerRelationship(p_a, ENEMIES);
        }
    }

    void flush_destroy_list() {
        TheGameLogic->processDestroyList();
    }

    void set_frame(UnsignedInt f) {
        TheGameLogic->*access_member(Wp51GLFrameTag()) = f;
    }

    UnsignedInt frame() const {
        return TheGameLogic->getFrame();
    }

    void load_ini(const std::string& text, INILoadType mode = INI_LOAD_OVERWRITE) {
        mem_ini.parse_blocks(text, mode, [](INI* ini, const char* tok) {
            if (std::strcmp(tok, "Armor") == 0) {
                ArmorStore::parseArmorDefinition(ini);
            } else if (std::strcmp(tok, "Upgrade") == 0) {
                INI::parseUpgradeDefinition(ini);
            } else if (std::strcmp(tok, "Science") == 0) {
                INI::parseScienceDefinition(ini);
            } else if (std::strcmp(tok, "DamageFX") == 0) {
                INI::parseDamageFXDefinition(ini);
            } else if (std::strcmp(tok, "FXList") == 0) {
                FXListStore::parseFXListDefinition(ini);
            } else if (std::strcmp(tok, "ObjectCreationList") == 0) {
                ObjectCreationListStore::parseObjectCreationListDefinition(ini);
            } else if (std::strcmp(tok, "SpecialPower") == 0) {
                SpecialPowerStore::parseSpecialPowerDefinition(ini);
            } else if (std::strcmp(tok, "CrateData") == 0) {
                INI::parseCrateTemplateDefinition(ini);
            } else if (std::strcmp(tok, "CommandSet") == 0) {
                INI::parseCommandSetDefinition(ini);
            } else if (std::strcmp(tok, "PlayerTemplate") == 0) {
                INI::parsePlayerTemplateDefinition(ini);
            } else if (std::strcmp(tok, "Weapon") == 0) {
                WeaponStore::parseWeaponTemplateDefinition(ini);
            } else if (std::strcmp(tok, "Locomotor") == 0) {
                LocomotorStore::parseLocomotorTemplateDefinition(ini);
            } else if (std::strcmp(tok, "Bridge") == 0) {
                INI::parseTerrainBridgeDefinition(ini);
            } else if (std::strcmp(tok, "Object") == 0) {
                AsciiString name = ini->getNextToken();
                ThingFactory::parseObjectDefinition(ini, name, AsciiString::TheEmptyString);
            }
        });
    }

    Object* spawn_object(
        const char* tmpl_name,
        Team* team,
        const Coord3D& pos = {0.0f, 0.0f, 0.0f},
        Real angle = 0.0f
    ) {
        const ThingTemplate* tt = TheThingFactory->findTemplate(tmpl_name);
        if (!tt) return nullptr;
        Object* obj = TheThingFactory->newObject(tt, team ? team : team_a);
        if (!obj) return nullptr;
        obj->setPosition(&pos);
        obj->setOrientation(angle);
        return obj;
    }

    static std::string snapshot_save_hex(Snapshot* s, uint32_t* out_crc = nullptr) {
        oracle::MemoryXfer xs(XFER_SAVE);
        xs.open("save");
        oracle::call_snapshot_xfer(s, &xs);
        xs.close();
        if (out_crc) *out_crc = xs.bytes_crc();
        return xs.hex();
    }

    static uint32_t snapshot_crc(Snapshot* s) {
        XferCRC xc;
        xc.open("crc");
        oracle::call_snapshot_crc(s, &xc);
        xc.close();
        return xc.getCRC();
    }

    static void snapshot_load_bytes(Snapshot* s, const std::vector<uint8_t>& bytes) {
        oracle::MemoryXfer xl(XFER_LOAD);
        xl.open("load");
        xl.bytes = bytes;
        oracle::call_snapshot_xfer(s, &xl);
        xl.close();
    }
};

} // namespace oracle_wp51
