// WP-5.2-A DieModule hierarchy (12 packets: PKT-205, 206, 207, 208, 211, 212,
// 213, 217, 220, 230, 243, 252)
// C++ Golden Vector Generator (Zero-Stub Build against oracle_wp51_world.h).
//
// Canonical build & run:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp52_die.cpp --opt O2 \
//     --out /tmp/gen_wp52_die_o2
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy /tmp/gen_wp52_die_o2 docs/golden_vectors/WP52_Die.json

#include "oracle_wp51_world.h"

#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/RandomValue.h"
#include "Common/SpecialPower.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/Upgrade.h"
#include "GameClient/Drawable.h"
#include "GameClient/FXList.h"
#include "GameLogic/CrateSystem.h"
#include "GameLogic/Damage.h"
#include "GameLogic/ExperienceTracker.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/CreateCrateDie.h"
#include "GameLogic/Module/CreateObjectDie.h"
#include "GameLogic/Module/CrushDie.h"
#include "GameLogic/Module/DamDie.h"
#include "GameLogic/Module/DestroyDie.h"
#include "GameLogic/Module/DieModule.h"
#include "GameLogic/Module/EjectPilotDie.h"
#include "GameLogic/Module/FXListDie.h"
#include "GameLogic/Module/KeepObjectDie.h"
#include "GameLogic/Module/RebuildHoleExposeDie.h"
#include "GameLogic/Module/SpecialPowerCompletionDie.h"
#include "GameLogic/Module/UpgradeDie.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectCreationList.h"

#include <sstream>
#include <string>
#include <unordered_set>
#include <vector>

struct PlayerUpgTag { using type = UpgradeMaskType Player::*; };
template struct MemberAccess<PlayerUpgTag, &Player::m_upgradesCompleted>;
UpgradeMaskType Player::* access_member(PlayerUpgTag);

struct PlayerSciTag { using type = ScienceVec Player::*; };
template struct MemberAccess<PlayerSciTag, &Player::m_sciences>;
ScienceVec Player::* access_member(PlayerSciTag);

static DieModule* find_first_die_module(Object* obj) {
    if (!obj) return nullptr;
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m) {
        if ((*m)->getDie()) {
            return static_cast<DieModule*>(*m);
        }
    }
    return nullptr;
}

static uint32_t obj_count_live() {
    uint32_t cnt = 0;
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
        if (!o->isDestroyed()) ++cnt;
    }
    return cnt;
}

static std::vector<uint8_t> hex_to_bytes(const std::string& hex) {
    std::vector<uint8_t> out;
    out.reserve(hex.size() / 2);
    auto nibble = [](char c) -> uint8_t {
        if (c >= '0' && c <= '9') return static_cast<uint8_t>(c - '0');
        if (c >= 'a' && c <= 'f') return static_cast<uint8_t>(10 + (c - 'a'));
        if (c >= 'A' && c <= 'F') return static_cast<uint8_t>(10 + (c - 'A'));
        return 0;
    };
    for (size_t i = 0; i + 1 < hex.size(); i += 2) {
        out.push_back(static_cast<uint8_t>((nibble(hex[i]) << 4) | nibble(hex[i + 1])));
    }
    return out;
}

int main(int argc, char** argv) {
    const char* out_path = (argc > 1) ? argv[1] : "docs/golden_vectors/WP52_Die.json";
    oracle_wp51::World w;

    // Static definitions loaded once before the 1,000-seed loop.
    // Note: in oracle_wp51::World, p_a (player 0) == ThePlayerList->getNeutralPlayer(),
    // and p_b (player 1) == non-neutral active player.
    w.load_ini(
        "Upgrade UpgDieA\n  Type = OBJECT\n  BuildTime = 1.0\n  BuildCost = 100\nEnd\n"
        "Upgrade UpgDieConflict\n  Type = PLAYER\n  BuildTime = 1.0\n  BuildCost = 100\nEnd\n"
        "Upgrade UpgDieRemove\n  Type = OBJECT\n  BuildTime = 1.0\n  BuildCost = 100\nEnd\n"
        "Upgrade UpgDieDrone\n  Type = OBJECT\n  BuildTime = 1.0\n  BuildCost = 100\nEnd\n"
        "Science SCI_WP52_DIE\n  SciencePurchasePointCost = 1\n  IsGrantable = Yes\nEnd\n"
        "FXList FX_DieMain\nEnd\n"
        "FXList FX_DieUpg\nEnd\n"
        "SpecialPower SpWP52_Die\n  Enum = SPECIAL_CASH_HACK\n  ReloadTime = 1000\nEnd\n"
        "Object WP52_DieSpawned\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 80\n    InitialHealth = 80\n  End\n"
        "End\n"
        "Object WP52_CrateItemA\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 10\n    InitialHealth = 10\n  End\n"
        "End\n"
        "Object WP52_CrateItemB\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 10\n    InitialHealth = 10\n  End\n"
        "End\n"
        "Object WP52_CrateItemC\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 10\n    InitialHealth = 10\n  End\n"
        "End\n"
        "Object WP52_WaveGuide\n"
        "  KindOf = WAVEGUIDE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "End\n"
        "Object WP52_NonWave\n"
        "  KindOf = VEHICLE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "End\n"
        "Object WP52_KillerInf\n"
        "  KindOf = INFANTRY\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "End\n"
        "Object WP52_KillerVeh\n"
        "  KindOf = VEHICLE\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "End\n"
        "Object WP52_RebuildHole\n"
        "  KindOf = STRUCTURE\n"
        "  Geometry = BOX\n"
        "  GeometryMajorRadius = 15.0\n"
        "  GeometryMinorRadius = 15.0\n"
        "  GeometryHeight = 10.0\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "End\n"
        "ObjectCreationList OCL_DieTest\n"
        "  CreateObject\n    ObjectNames = WP52_DieSpawned\n    Count = 1\n  End\n"
        "End\n"
        "ObjectCreationList OCL_EjectAir\n"
        "  CreateObject\n    ObjectNames = WP52_CrateItemA\n    Count = 1\n  End\n"
        "End\n"
        "ObjectCreationList OCL_EjectGround\n"
        "  CreateObject\n    ObjectNames = WP52_CrateItemB\n    Count = 1\n  End\n"
        "End\n"
        "CrateData Crate_Basic\n"
        "  CreationChance = 0.50\n"
        "  CrateObject = WP52_CrateItemA 0.30\n"
        "  CrateObject = WP52_CrateItemB 0.50\n"
        "  OwnedByMaker = No\n"
        "End\n"
        "CrateData Crate_VetElite\n"
        "  CreationChance = 0.80\n"
        "  VeterancyLevel = ELITE\n"
        "  CrateObject = WP52_CrateItemB 1.0\n"
        "  OwnedByMaker = Yes\n"
        "End\n"
        "CrateData Crate_KillerInfSci\n"
        "  CreationChance = 0.90\n"
        "  KilledByType = INFANTRY\n"
        "  KillerScience = SCI_WP52_DIE\n"
        "  CrateObject = WP52_CrateItemC 1.0\n"
        "  OwnedByMaker = No\n"
        "End\n"
        "Object D52_Crate_0\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = CreateCrateDie ModuleTag_Die\n"
        "    CrateData = Crate_Basic\n"
        "  End\n"
        "End\n"
        "Object D52_Crate_1\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = CreateCrateDie ModuleTag_Die\n"
        "    CrateData = Crate_VetElite\n"
        "  End\n"
        "End\n"
        "Object D52_Crate_2\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = CreateCrateDie ModuleTag_Die\n"
        "    CrateData = Crate_KillerInfSci\n"
        "  End\n"
        "End\n"
        "Object D52_Crate_3\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = CreateCrateDie ModuleTag_Die\n"
        "    CrateData = Crate_VetElite\n"
        "    CrateData = Crate_Basic\n"
        "  End\n"
        "End\n"
        "Object D52_CreateObj\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = CreateObjectDie ModuleTag_Die\n"
        "    DeathTypes = NONE +NORMAL +EXPLODED\n"
        "    CreationList = OCL_DieTest\n"
        "    TransferPreviousHealth = No\n"
        "    TransferSelection = No\n"
        "  End\n"
        "End\n"
        "Object D52_Dam\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = DamDie ModuleTag_Die\n"
        "    DeathTypes = NONE +EXPLODED\n"
        "  End\n"
        "End\n"
        "Object D52_Eject\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  UnitSpecificSounds\n"
        "    VoiceEject = NoSound\n"
        "    SoundEject = NoSound\n"
        "  End\n"
        "  Behavior = EjectPilotDie ModuleTag_Die\n"
        "    AirCreationList = OCL_EjectAir\n"
        "    GroundCreationList = OCL_EjectGround\n"
        "    InvulnerableTime = 30\n"
        "  End\n"
        "End\n"
        "Object D52_SP\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = SpecialPowerCompletionDie ModuleTag_Die\n"
        "    SpecialPowerTemplate = SpWP52_Die\n"
        "  End\n"
        "End\n"
        "Object D52_UpgDie\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = UpgradeDie ModuleTag_Die\n"
        "    UpgradeToRemove = UpgDieDrone\n"
        "  End\n"
        "End\n"
        "Object D52_FX_00\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = FXListDie ModuleTag_Die\n"
        "    StartsActive = No\n"
        "    DeathFX = FX_DieMain\n"
        "    OrientToObject = No\n"
        "    TriggeredBy = UpgDieA\n"
        "    ConflictsWith = UpgDieConflict\n"
        "    RemovesUpgrades = UpgDieRemove\n"
        "    FXListUpgrade = FX_DieUpg\n"
        "  End\n"
        "End\n"
        "Object D52_FX_01\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = FXListDie ModuleTag_Die\n"
        "    StartsActive = No\n"
        "    DeathFX = FX_DieMain\n"
        "    OrientToObject = Yes\n"
        "    TriggeredBy = UpgDieA\n"
        "    ConflictsWith = UpgDieConflict\n"
        "    RemovesUpgrades = UpgDieRemove\n"
        "    FXListUpgrade = FX_DieUpg\n"
        "  End\n"
        "End\n"
        "Object D52_FX_10\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = FXListDie ModuleTag_Die\n"
        "    StartsActive = Yes\n"
        "    DeathFX = FX_DieMain\n"
        "    OrientToObject = No\n"
        "    TriggeredBy = UpgDieA\n"
        "    ConflictsWith = UpgDieConflict\n"
        "    RemovesUpgrades = UpgDieRemove\n"
        "    FXListUpgrade = FX_DieUpg\n"
        "  End\n"
        "End\n"
        "Object D52_FX_11\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = FXListDie ModuleTag_Die\n"
        "    StartsActive = Yes\n"
        "    DeathFX = FX_DieMain\n"
        "    OrientToObject = Yes\n"
        "    TriggeredBy = UpgDieA\n"
        "    ConflictsWith = UpgDieConflict\n"
        "    RemovesUpgrades = UpgDieRemove\n"
        "    FXListUpgrade = FX_DieUpg\n"
        "  End\n"
        "End\n"
        "Object D52_Crush_0\n"
        "  Geometry = BOX\n"
        "  GeometryMajorRadius = 10.0\n"
        "  GeometryMinorRadius = 5.0\n"
        "  GeometryHeight = 5.0\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = CrushDie ModuleTag_Die\n"
        "    TotalCrushSound = NoSound\n"
        "    BackEndCrushSound = NoSound\n"
        "    FrontEndCrushSound = NoSound\n"
        "  End\n"
        "End\n"
        "Object D52_Crush_1\n"
        "  Geometry = BOX\n"
        "  GeometryMajorRadius = 10.0\n"
        "  GeometryMinorRadius = 5.0\n"
        "  GeometryHeight = 5.0\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        "  Behavior = CrushDie ModuleTag_Die\n"
        "    TotalCrushSound = UnitCrushSound\n"
        "    BackEndCrushSound = UnitCrushSound\n"
        "    FrontEndCrushSound = UnitCrushSound\n"
        "    TotalCrushSoundPercent = 50\n"
        "    BackEndCrushSoundPercent = 75\n"
        "    FrontEndCrushSoundPercent = 25\n"
        "  End\n"
        "End\n"
    );

    // Pre-generate the 81 finite DieMux filter templates (3x3x3x3) ONCE so per-seed
    // INI loading stays minimal (~1 template per seed for continuous RebuildHole values).
    static const char* kDeathFilters[3] = {
        "ALL",
        "NONE +NORMAL +EXPLODED",
        "ALL -CRUSHED -LASERED"
    };
    static const char* kVetFilters[3] = {
        "ALL",
        "NONE +VETERAN +HEROIC",
        "ALL -REGULAR"
    };
    static const char* kExemptFilters[3] = {
        "NONE",
        "UNDER_CONSTRUCTION",
        "STEALTHED"
    };
    static const char* kReqFilters[3] = {
        "NONE",
        "IMMOBILE",
        "STEALTHED"
    };
    {
        std::ostringstream mux_ini;
        for (int df = 0; df < 3; ++df) {
            for (int vf = 0; vf < 3; ++vf) {
                for (int ef = 0; ef < 3; ++ef) {
                    for (int rf = 0; rf < 3; ++rf) {
                        std::string suf = std::to_string(df) + std::to_string(vf) +
                                          std::to_string(ef) + std::to_string(rf);
                        mux_ini << "Object D52_Keep_" << suf << "\n"
                                << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
                                << "  Behavior = KeepObjectDie ModuleTag_Die\n"
                                << "    DeathTypes = " << kDeathFilters[df] << "\n"
                                << "    VeterancyLevels = " << kVetFilters[vf] << "\n"
                                << "    ExemptStatus = " << kExemptFilters[ef] << "\n"
                                << "    RequiredStatus = " << kReqFilters[rf] << "\n"
                                << "  End\nEnd\n"
                                << "Object D52_Dest_" << suf << "\n"
                                << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
                                << "  Behavior = DestroyDie ModuleTag_Die\n"
                                << "    DeathTypes = " << kDeathFilters[df] << "\n"
                                << "    VeterancyLevels = " << kVetFilters[vf] << "\n"
                                << "    ExemptStatus = " << kExemptFilters[ef] << "\n"
                                << "    RequiredStatus = " << kReqFilters[rf] << "\n"
                                << "  End\nEnd\n";
                    }
                }
            }
        }
        w.load_ini(mux_ini.str(), INI_LOAD_OVERWRITE);
    }

    Player* p_a = ThePlayerList->getNthPlayer(0);
    Player* p_b = ThePlayerList->getNthPlayer(1);
    const UpgradeTemplate* ut_a = TheUpgradeCenter->findUpgrade("UpgDieA");
    const UpgradeTemplate* ut_conf = TheUpgradeCenter->findUpgrade("UpgDieConflict");
    const UpgradeTemplate* ut_rem = TheUpgradeCenter->findUpgrade("UpgDieRemove");
    const UpgradeTemplate* ut_drone = TheUpgradeCenter->findUpgrade("UpgDieDrone");
    ScienceType sci_die = TheScienceStore->friend_lookupScience("SCI_WP52_DIE");

    static const DeathType kDeathTypes[5] = {
        DEATH_NORMAL, DEATH_CRUSHED, DEATH_EXPLODED, DEATH_LASERED, DEATH_BURNED
    };
    static const VeterancyLevel kVetLevels[4] = {
        LEVEL_REGULAR, LEVEL_VETERAN, LEVEL_ELITE, LEVEL_HEROIC
    };

    std::string s0_diag;
    std::ostringstream vecs;

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed ^ 0x52a01234u;
        if (rng == 0) rng = 0x13579bdfu;

        UnsignedInt frame0 = 10u + (oracle::splitmix32(rng) % 500u);
        w.set_frame(frame0);

        // Reset player state
        (p_a->*access_member(PlayerUpgTag())) = UpgradeMaskType();
        (p_b->*access_member(PlayerUpgTag())) = UpgradeMaskType();
        (p_a->*access_member(PlayerSciTag())).clear();
        (p_b->*access_member(PlayerSciTag())).clear();

        // Draw seed parameters
        int df = static_cast<int>(oracle::splitmix32(rng) % 3u);
        int vf = static_cast<int>(oracle::splitmix32(rng) % 3u);
        int ef = static_cast<int>(oracle::splitmix32(rng) % 3u);
        int rf = static_cast<int>(oracle::splitmix32(rng) % 3u);
        int obj_vet_idx = static_cast<int>(oracle::splitmix32(rng) % 4u);
        uint32_t obj_status_bits = oracle::splitmix32(rng) % 8u; // bit0=UNDER_CONST, bit1=STEALTHED, bit2=POWERED
        int dmg_death_idx = static_cast<int>(oracle::splitmix32(rng) % 5u);

        bool fx_starts_active = (seed == 0) ? false : ((oracle::splitmix32(rng) & 1u) != 0);
        bool fx_orient = (seed == 0) ? true : ((oracle::splitmix32(rng) & 1u) != 0);
        bool fx_give_upg = (seed == 0) ? true : ((oracle::splitmix32(rng) & 1u) != 0);
        bool fx_conf_obj = (seed == 0) ? false : ((oracle::splitmix32(rng) % 4u) == 0);
        bool fx_conf_ply = (seed == 0) ? false : ((oracle::splitmix32(rng) % 5u) == 0);

        int crate_killer_mode = (seed == 0) ? 3 : static_cast<int>(oracle::splitmix32(rng) % 5u);
        int crate_vet_idx = (seed == 0) ? 2 : static_cast<int>(oracle::splitmix32(rng) % 4u);
        int crate_tmpl_idx = (seed == 0) ? 2 : static_cast<int>(oracle::splitmix32(rng) % 4u);
        if (crate_tmpl_idx == 3 && crate_vet_idx == 2) {
            // Avoid orientation RNG phase shift when first entry is guaranteed to pass vet filter
            crate_tmpl_idx = 1;
        }
        UnsignedInt crate_rng_seed = oracle::splitmix32(rng);

        bool crush_is_crush_dmg = (seed == 0) ? true : ((oracle::splitmix32(rng) % 5u) != 0);
        bool crush_has_dealer = (seed == 0) ? true : ((oracle::splitmix32(rng) % 4u) != 0);
        bool crush_pre_front = (seed == 0) ? false : ((oracle::splitmix32(rng) % 3u) == 0);
        bool crush_pre_back = (seed == 0) ? false : ((oracle::splitmix32(rng) % 3u) == 1);
        bool crush_named_sound = (seed == 0) ? true : ((oracle::splitmix32(rng) & 1u) != 0);
        Real crush_angle = 0.25f * static_cast<Real>(oracle::splitmix32(rng) % 12u);
        Real dealer_dx = -8.0f + 2.0f * static_cast<Real>(oracle::splitmix32(rng) % 9u);
        Real dealer_dy = -8.0f + 2.0f * static_cast<Real>(oracle::splitmix32(rng) % 9u);
        UnsignedInt crush_rng_seed = oracle::splitmix32(rng);

        bool eject_air = (seed == 0) ? true : ((oracle::splitmix32(rng) & 1u) != 0);

        bool rh_neutral = (seed == 0) ? false : ((oracle::splitmix32(rng) % 4u) == 0);
        bool rh_under_const = (seed == 0) ? false : ((oracle::splitmix32(rng) % 4u) == 1);
        Real rh_max_hp = 150.0f + 25.0f * static_cast<Real>(oracle::splitmix32(rng) % 8u);
        Real rh_major_radius = 12.0f + 2.0f * static_cast<Real>(oracle::splitmix32(rng) % 6u);
        bool rh_transfer_atk = (seed == 0) ? true : ((oracle::splitmix32(rng) & 1u) != 0);

        bool sp_set_creator = (seed == 0) ? true : ((oracle::splitmix32(rng) % 4u) != 0);
        UnsignedInt sp_creator_raw = 100u + (oracle::splitmix32(rng) % 50u);
        bool ud_has_producer = (seed == 0) ? true : ((oracle::splitmix32(rng) % 4u) != 0);
        bool ud_has_upgrade = (seed == 0) ? true : ((oracle::splitmix32(rng) % 3u) != 0);

        // Per-seed RebuildHole template (only 1 small block per seed)
        std::string t_rh = "D52_RH_" + std::to_string(seed);
        {
            std::ostringstream ini_ss;
            ini_ss << "Object " << t_rh << "\n"
                   << "  KindOf = STRUCTURE\n"
                   << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 500\n    InitialHealth = 500\n  End\n"
                   << "  Behavior = RebuildHoleExposeDie ModuleTag_Die\n"
                   << "    HoleName = WP52_RebuildHole\n"
                   << "    HoleMaxHealth = " << rh_max_hp << "\n"
                   << "    TransferAttackers = " << (rh_transfer_atk ? "Yes" : "No") << "\n"
                   << "  End\nEnd\n";
            w.load_ini(ini_ss.str(), INI_LOAD_OVERWRITE);
        }

        // Helper to apply status bits
        auto apply_status = [&](Object* o) {
            if (obj_status_bits & 1u) o->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
            if (obj_status_bits & 2u) o->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_STEALTHED));
            if (obj_status_bits & 4u) o->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_IMMOBILE));
        };

        DamageInfo dmg;
        dmg.in.m_damageType = DAMAGE_EXPLOSION;
        dmg.in.m_deathType = kDeathTypes[dmg_death_idx];
        dmg.in.m_amount = 1000.0f;
        dmg.in.m_sourceID = INVALID_ID;

        // 1) DieMuxData + KeepObjectDie + DestroyDie
        std::string suf = std::to_string(df) + std::to_string(vf) +
                          std::to_string(ef) + std::to_string(rf);
        Object* o_keep = w.spawn_object(("D52_Keep_" + suf).c_str(), w.team_b);
        Object* o_dest = w.spawn_object(("D52_Dest_" + suf).c_str(), w.team_b);
        o_keep->getExperienceTracker()->setVeterancyLevel(kVetLevels[obj_vet_idx], FALSE);
        o_dest->getExperienceTracker()->setVeterancyLevel(kVetLevels[obj_vet_idx], FALSE);
        apply_status(o_keep);
        apply_status(o_dest);

        DieModule* dm_keep = find_first_die_module(o_keep);
        DieModule* dm_dest = find_first_die_module(o_dest);
        dm_keep->onDie(&dmg);
        dm_dest->onDie(&dmg);
        bool keep_destroyed = o_keep->isDestroyed();
        bool dest_destroyed = o_dest->isDestroyed();
        uint32_t keep_crc = oracle_wp51::World::snapshot_crc(dm_keep);
        std::string keep_hex = oracle_wp51::World::snapshot_save_hex(dm_keep);
        uint32_t dest_crc = oracle_wp51::World::snapshot_crc(dm_dest);
        std::string dest_hex = oracle_wp51::World::snapshot_save_hex(dm_dest);

        // 2) FXListDie
        std::string t_fx = std::string("D52_FX_") + (fx_starts_active ? "1" : "0") + (fx_orient ? "1" : "0");
        Object* o_fx = w.spawn_object(t_fx.c_str(), w.team_b, {15.0f, 25.0f, 0.0f}, 0.0f);
        o_fx->giveUpgrade(ut_rem);
        bool fx_rem_pre = o_fx->hasUpgrade(ut_rem);
        DieModule* dm_fx = find_first_die_module(o_fx);
        UpgradeModuleInterface* fx_upg = dm_fx->getUpgrade();
        bool fx_active_init = fx_upg->isAlreadyUpgraded();
        if (fx_give_upg) o_fx->giveUpgrade(ut_a);
        bool fx_active_after = fx_upg->isAlreadyUpgraded();
        bool fx_rem_after = o_fx->hasUpgrade(ut_rem);
        if (fx_conf_obj) o_fx->giveUpgrade(ut_conf);
        if (fx_conf_ply) (p_b->*access_member(PlayerUpgTag())).set(ut_conf->getUpgradeMask());
        // Check if onDie fires FX (upgrade active & no obj/player conflict)
        const UpgradeMaskType& fx_conf_mask = ut_conf->getUpgradeMask();
        bool fx_should_fire = fx_active_after &&
                              !o_fx->getObjectCompletedUpgradeMask().testForAny(fx_conf_mask) &&
                              !p_b->getCompletedUpgradeMask().testForAny(fx_conf_mask);
        DamageInfo fx_dmg;
        fx_dmg.in.m_damageType = DAMAGE_EXPLOSION;
        fx_dmg.in.m_deathType = DEATH_NORMAL;
        fx_dmg.in.m_amount = 50.0f;
        dm_fx->onDie(&fx_dmg);
        uint32_t fx_crc = oracle_wp51::World::snapshot_crc(dm_fx);
        std::string fx_hex = oracle_wp51::World::snapshot_save_hex(dm_fx);

        // 3) CreateObjectDie
        Object* o_co = w.spawn_object("D52_CreateObj", w.team_b, {30.0f, 40.0f, 0.0f}, 0.0f);
        DieModule* dm_co = find_first_die_module(o_co);
        uint32_t co_before = obj_count_live();
        dm_co->onDie(&dmg);
        uint32_t co_after = obj_count_live();
        uint32_t co_spawned = co_after - co_before;
        uint32_t co_crc = oracle_wp51::World::snapshot_crc(dm_co);
        std::string co_hex = oracle_wp51::World::snapshot_save_hex(dm_co);

        // 4) EjectPilotDie
        Object* o_ej = w.spawn_object(
            "D52_Eject", w.team_b, {10.0f, 20.0f, eject_air ? 50.0f : 0.0f}, 0.0f
        );
        std::unordered_set<Object*> seen_before_ej;
        for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
            seen_before_ej.insert(o);
        }
        DieModule* dm_ej = find_first_die_module(o_ej);
        DamageInfo ej_dmg;
        ej_dmg.in.m_damageType = DAMAGE_EXPLOSION;
        ej_dmg.in.m_deathType = DEATH_NORMAL;
        ej_dmg.in.m_amount = 50.0f;
        dm_ej->onDie(&ej_dmg);
        std::string ej_spawned_name;
        for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
            if (seen_before_ej.find(o) == seen_before_ej.end() && !o->isDestroyed()) {
                ej_spawned_name = o->getTemplate()->getName().str();
                break;
            }
        }
        uint32_t ej_crc = oracle_wp51::World::snapshot_crc(dm_ej);
        std::string ej_hex = oracle_wp51::World::snapshot_save_hex(dm_ej);

        // 5) CreateCrateDie
        // Victim on team_b; killer depends on crate_killer_mode:
        // 0 = no killer, 1 = ally (team_b), 2 = enemy inf (team_a, no sci),
        // 3 = enemy inf (team_a, with sci), 4 = enemy veh (team_a, with sci)
        std::string crate_tmpl_name = "D52_Crate_" + std::to_string(crate_tmpl_idx);
        Object* o_cr = w.spawn_object(crate_tmpl_name.c_str(), w.team_b, {60.0f, 70.0f, 0.0f}, 0.0f);
        o_cr->setLayer(LAYER_WALL);
        o_cr->getExperienceTracker()->setVeterancyLevel(kVetLevels[crate_vet_idx], FALSE);
        Object* o_killer = nullptr;
        if (crate_killer_mode == 1) {
            o_killer = w.spawn_object("WP52_KillerInf", w.team_b, {62.0f, 70.0f, 0.0f}, 0.0f);
        } else if (crate_killer_mode == 2) {
            o_killer = w.spawn_object("WP52_KillerInf", w.team_a, {62.0f, 70.0f, 0.0f}, 0.0f);
        } else if (crate_killer_mode == 3) {
            (p_a->*access_member(PlayerSciTag())).push_back(sci_die);
            o_killer = w.spawn_object("WP52_KillerInf", w.team_a, {62.0f, 70.0f, 0.0f}, 0.0f);
        } else if (crate_killer_mode == 4) {
            (p_a->*access_member(PlayerSciTag())).push_back(sci_die);
            o_killer = w.spawn_object("WP52_KillerVeh", w.team_a, {62.0f, 70.0f, 0.0f}, 0.0f);
        }
        std::unordered_set<Object*> seen_before_cr;
        for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
            seen_before_cr.insert(o);
        }
        DamageInfo cr_dmg;
        cr_dmg.in.m_damageType = DAMAGE_EXPLOSION;
        cr_dmg.in.m_deathType = DEATH_NORMAL;
        cr_dmg.in.m_amount = 50.0f;
        cr_dmg.in.m_sourceID = o_killer ? o_killer->getID() : INVALID_ID;

        InitRandom(crate_rng_seed);
        DieModule* dm_cr = find_first_die_module(o_cr);
        dm_cr->onDie(&cr_dmg);
        UnsignedInt crate_post_rng_crc = GetGameLogicRandomSeedCRC();

        std::vector<std::string> spawned_crates;
        std::vector<int> spawned_crate_owned;
        for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
            if (seen_before_cr.find(o) == seen_before_cr.end() && !o->isDestroyed()) {
                spawned_crates.push_back(o->getTemplate()->getName().str());
                spawned_crate_owned.push_back((o->getControllingPlayer() == p_b) ? 1 : 0);
            }
        }
        uint32_t cr_crc = oracle_wp51::World::snapshot_crc(dm_cr);
        std::string cr_hex = oracle_wp51::World::snapshot_save_hex(dm_cr);

        // 6) CrushDie
        Object* o_crush = w.spawn_object(
            crush_named_sound ? "D52_Crush_1" : "D52_Crush_0",
            w.team_b,
            {100.0f, 100.0f, 0.0f},
            crush_angle
        );
        o_crush->getBodyModule()->setFrontCrushed(crush_pre_front);
        o_crush->getBodyModule()->setBackCrushed(crush_pre_back);
        Object* o_crusher = nullptr;
        if (crush_has_dealer) {
            o_crusher = w.spawn_object(
                "WP52_KillerVeh",
                w.team_a,
                {100.0f + dealer_dx, 100.0f + dealer_dy, 0.0f},
                0.0f
            );
        }
        DamageInfo crush_dmg;
        crush_dmg.in.m_damageType = crush_is_crush_dmg ? DAMAGE_CRUSH : DAMAGE_EXPLOSION;
        crush_dmg.in.m_deathType = DEATH_CRUSHED;
        crush_dmg.in.m_amount = 50.0f;
        crush_dmg.in.m_sourceID = o_crusher ? o_crusher->getID() : INVALID_ID;

        InitRandom(crush_rng_seed);
        DieModule* dm_crush = find_first_die_module(o_crush);
        dm_crush->onDie(&crush_dmg);
        UnsignedInt crush_post_rng_crc = GetGameLogicRandomSeedCRC();
        bool crush_post_front = o_crush->getBodyModule()->getFrontCrushed();
        bool crush_post_back = o_crush->getBodyModule()->getBackCrushed();
        bool crush_mc_front = o_crush->getDrawable()->getModelConditionFlags().test(MODELCONDITION_FRONTCRUSHED);
        bool crush_mc_back = o_crush->getDrawable()->getModelConditionFlags().test(MODELCONDITION_BACKCRUSHED);
        uint32_t crush_crc = oracle_wp51::World::snapshot_crc(dm_crush);
        std::string crush_hex = oracle_wp51::World::snapshot_save_hex(dm_crush);

        // 7) DamDie
        Object* o_wg1 = w.spawn_object("WP52_WaveGuide", w.team_a);
        Object* o_wg2 = w.spawn_object("WP52_WaveGuide", w.team_b);
        Object* o_nw = w.spawn_object("WP52_NonWave", w.team_b);
        o_wg1->setDisabled(DISABLED_DEFAULT);
        o_wg2->setDisabled(DISABLED_DEFAULT);
        o_nw->setDisabled(DISABLED_DEFAULT);
        Object* o_dam = w.spawn_object("D52_Dam", w.team_b);
        DieModule* dm_dam = find_first_die_module(o_dam);
        dm_dam->onDie(&dmg); // uses dmg.in.m_deathType (DamDie requires DEATH_EXPLOSION)
        bool wg1_dis = o_wg1->isDisabledByType(DISABLED_DEFAULT);
        bool wg2_dis = o_wg2->isDisabledByType(DISABLED_DEFAULT);
        bool nw_dis = o_nw->isDisabledByType(DISABLED_DEFAULT);
        uint32_t dam_crc = oracle_wp51::World::snapshot_crc(dm_dam);
        std::string dam_hex = oracle_wp51::World::snapshot_save_hex(dm_dam);

        // 8) RebuildHoleExposeDie
        Team* rh_team = rh_neutral ? w.team_a : w.team_b;
        Object* o_rh = w.spawn_object(t_rh.c_str(), rh_team, {200.0f, 200.0f, 0.0f}, 0.5f);
        GeometryInfo rh_geom = o_rh->getGeometryInfo();
        rh_geom.setMajorRadius(rh_major_radius);
        o_rh->setGeometryInfo(rh_geom);
        if (rh_under_const) {
            o_rh->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
        }
        std::unordered_set<Object*> seen_before_rh;
        for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
            seen_before_rh.insert(o);
        }
        DieModule* dm_rh = find_first_die_module(o_rh);
        DamageInfo rh_dmg;
        rh_dmg.in.m_damageType = DAMAGE_EXPLOSION;
        rh_dmg.in.m_deathType = DEATH_NORMAL;
        rh_dmg.in.m_amount = 500.0f;
        dm_rh->onDie(&rh_dmg);
        bool rh_spawned = false;
        Real rh_hole_max_hp = 0.0f;
        Real rh_hole_hp = 0.0f;
        Real rh_hole_radius = 0.0f;
        for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
            if (seen_before_rh.find(o) == seen_before_rh.end() && !o->isDestroyed()) {
                rh_spawned = true;
                rh_hole_max_hp = o->getBodyModule()->getMaxHealth();
                rh_hole_hp = o->getBodyModule()->getHealth();
                rh_hole_radius = o->getGeometryInfo().getMajorRadius();
                break;
            }
        }
        uint32_t rh_crc = oracle_wp51::World::snapshot_crc(dm_rh);
        std::string rh_hex = oracle_wp51::World::snapshot_save_hex(dm_rh);

        // 9) SpecialPowerCompletionDie
        Object* o_sp = w.spawn_object("D52_SP", w.team_b);
        SpecialPowerCompletionDie* dm_sp = static_cast<SpecialPowerCompletionDie*>(find_first_die_module(o_sp));
        if (sp_set_creator) {
            dm_sp->setCreator(static_cast<ObjectID>(sp_creator_raw));
            dm_sp->setCreator(static_cast<ObjectID>(9999u)); // ignored once creatorSet == TRUE
        }
        uint32_t sp_crc = oracle_wp51::World::snapshot_crc(dm_sp);
        std::string sp_hex = oracle_wp51::World::snapshot_save_hex(dm_sp);
        // Verify load + re-save roundtrip on a fresh D52_SP instance
        Object* o_sp_rt = w.spawn_object("D52_SP", w.team_b);
        SpecialPowerCompletionDie* dm_sp_rt = static_cast<SpecialPowerCompletionDie*>(find_first_die_module(o_sp_rt));
        oracle_wp51::World::snapshot_load_bytes(dm_sp_rt, hex_to_bytes(sp_hex));
        std::string sp_rt_hex = oracle_wp51::World::snapshot_save_hex(dm_sp_rt);
        DamageInfo sp_dmg;
        sp_dmg.in.m_damageType = DAMAGE_EXPLOSION;
        sp_dmg.in.m_deathType = DEATH_NORMAL;
        dm_sp->onDie(&sp_dmg);
        std::string sp_post_hex = oracle_wp51::World::snapshot_save_hex(dm_sp);

        // 10) UpgradeDie
        Object* o_prod = nullptr;
        if (ud_has_producer) {
            o_prod = w.spawn_object("WP52_NonWave", w.team_b);
            if (ud_has_upgrade) {
                o_prod->giveUpgrade(ut_drone);
            }
        }
        bool ud_prod_pre = o_prod ? o_prod->hasUpgrade(ut_drone) : false;
        Object* o_ud = w.spawn_object("D52_UpgDie", w.team_b);
        if (o_prod) {
            o_ud->setProducer(o_prod);
        }
        DieModule* dm_ud = find_first_die_module(o_ud);
        DamageInfo ud_dmg;
        ud_dmg.in.m_damageType = DAMAGE_EXPLOSION;
        ud_dmg.in.m_deathType = DEATH_NORMAL;
        dm_ud->onDie(&ud_dmg);
        bool ud_prod_post = o_prod ? o_prod->hasUpgrade(ut_drone) : false;
        uint32_t ud_crc = oracle_wp51::World::snapshot_crc(dm_ud);
        std::string ud_hex = oracle_wp51::World::snapshot_save_hex(dm_ud);

        // Clean up all objects before next seed
        std::vector<Object*> to_kill;
        for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
            to_kill.push_back(o);
        }
        for (Object* o : to_kill) {
            TheGameLogic->destroyObject(o);
        }
        w.flush_destroy_list();

        if (seed == 0) {
            std::ostringstream d;
            d << "{"
              << "\"mux\":{\"keep_dest\":" << (keep_destroyed?1:0) << ",\"dest_dest\":" << (dest_destroyed?1:0)
              << ",\"keep_xfer\":\"" << keep_hex << "\",\"dest_xfer\":\"" << dest_hex << "\"},"
              << "\"fx\":{\"active_init\":" << (fx_active_init?1:0) << ",\"active_after\":" << (fx_active_after?1:0)
              << ",\"rem_pre\":" << (fx_rem_pre?1:0) << ",\"rem_after\":" << (fx_rem_after?1:0)
              << ",\"should_fire\":" << (fx_should_fire?1:0) << ",\"xfer\":\"" << fx_hex << "\"},"
              << "\"create_obj\":{\"spawned\":" << co_spawned << ",\"xfer\":\"" << co_hex << "\"},"
              << "\"eject\":{\"spawned\":\"" << ej_spawned_name << "\",\"xfer\":\"" << ej_hex << "\"},"
              << "\"crate\":{\"count\":" << spawned_crates.size() << ",\"post_rng_crc\":" << crate_post_rng_crc << ",\"xfer\":\"" << cr_hex << "\"},"
              << "\"crush\":{\"front\":" << (crush_post_front?1:0) << ",\"back\":" << (crush_post_back?1:0)
              << ",\"mc_front\":" << (crush_mc_front?1:0) << ",\"mc_back\":" << (crush_mc_back?1:0)
              << ",\"post_rng_crc\":" << crush_post_rng_crc << ",\"xfer\":\"" << crush_hex << "\"},"
              << "\"dam\":{\"wg1_dis\":" << (wg1_dis?1:0) << ",\"wg2_dis\":" << (wg2_dis?1:0) << ",\"nw_dis\":" << (nw_dis?1:0) << "},"
              << "\"rebuild_hole\":{\"spawned\":" << (rh_spawned?1:0) << ",\"max_hp_bits\":" << oracle::fbits(rh_hole_max_hp)
              << ",\"hp_bits\":" << oracle::fbits(rh_hole_hp) << ",\"rad_bits\":" << oracle::fbits(rh_hole_radius) << "},"
              << "\"sp\":{\"xfer\":\"" << sp_hex << "\",\"rt_xfer\":\"" << sp_rt_hex << "\",\"post_xfer\":\"" << sp_post_hex << "\"},"
              << "\"upgrade_die\":{\"pre\":" << (ud_prod_pre?1:0) << ",\"post\":" << (ud_prod_post?1:0) << ",\"xfer\":\"" << ud_hex << "\"}"
              << "}";
            s0_diag = d.str();
        }

        if (seed > 0) vecs << ",\n";
        vecs << "    {"
             << "\"seed\":" << seed
             << ",\"in\":{"
             << "\"frame\":" << frame0
             << ",\"df\":" << df << ",\"vf\":" << vf << ",\"ef\":" << ef << ",\"rf\":" << rf
             << ",\"obj_vet\":" << obj_vet_idx
             << ",\"obj_status\":" << obj_status_bits
             << ",\"dmg_death\":" << dmg_death_idx
             << ",\"fx_starts_active\":" << (fx_starts_active?1:0)
             << ",\"fx_orient\":" << (fx_orient?1:0)
             << ",\"fx_give_upg\":" << (fx_give_upg?1:0)
             << ",\"fx_conf_obj\":" << (fx_conf_obj?1:0)
             << ",\"fx_conf_ply\":" << (fx_conf_ply?1:0)
             << ",\"crate_killer_mode\":" << crate_killer_mode
             << ",\"crate_vet\":" << crate_vet_idx
             << ",\"crate_tmpl\":" << crate_tmpl_idx
             << ",\"crate_rng_seed\":" << crate_rng_seed
             << ",\"crush_is_crush\":" << (crush_is_crush_dmg?1:0)
             << ",\"crush_has_dealer\":" << (crush_has_dealer?1:0)
             << ",\"crush_pre_front\":" << (crush_pre_front?1:0)
             << ",\"crush_pre_back\":" << (crush_pre_back?1:0)
             << ",\"crush_named_sound\":" << (crush_named_sound?1:0)
             << ",\"crush_angle_bits\":" << oracle::fbits(crush_angle)
             << ",\"dealer_dx_bits\":" << oracle::fbits(dealer_dx)
             << ",\"dealer_dy_bits\":" << oracle::fbits(dealer_dy)
             << ",\"crush_rng_seed\":" << crush_rng_seed
             << ",\"eject_air\":" << (eject_air?1:0)
             << ",\"rh_neutral\":" << (rh_neutral?1:0)
             << ",\"rh_under_const\":" << (rh_under_const?1:0)
             << ",\"rh_max_hp_bits\":" << oracle::fbits(rh_max_hp)
             << ",\"rh_major_rad_bits\":" << oracle::fbits(rh_major_radius)
             << ",\"rh_transfer_atk\":" << (rh_transfer_atk?1:0)
             << ",\"sp_set_creator\":" << (sp_set_creator?1:0)
             << ",\"sp_creator_id\":" << sp_creator_raw
             << ",\"ud_has_producer\":" << (ud_has_producer?1:0)
             << ",\"ud_has_upgrade\":" << (ud_has_upgrade?1:0)
             << "}"
             << ",\"out\":{"
             << "\"keep_dest\":" << (keep_destroyed?1:0)
             << ",\"dest_dest\":" << (dest_destroyed?1:0)
             << ",\"keep_crc\":" << keep_crc
             << ",\"keep_xfer\":\"" << keep_hex << "\""
             << ",\"dest_crc\":" << dest_crc
             << ",\"dest_xfer\":\"" << dest_hex << "\""
             << ",\"fx\":[" << (fx_active_init?1:0) << "," << (fx_active_after?1:0)
             << "," << (fx_rem_pre?1:0) << "," << (fx_rem_after?1:0) << "," << (fx_should_fire?1:0) << "]"
             << ",\"fx_crc\":" << fx_crc
             << ",\"fx_xfer\":\"" << fx_hex << "\""
             << ",\"co_spawned\":" << co_spawned
             << ",\"co_crc\":" << co_crc
             << ",\"co_xfer\":\"" << co_hex << "\""
             << ",\"ej_spawned\":\"" << ej_spawned_name << "\""
             << ",\"ej_crc\":" << ej_crc
             << ",\"ej_xfer\":\"" << ej_hex << "\""
             << ",\"crates\":[";
        for (size_t i = 0; i < spawned_crates.size(); ++i) {
            if (i > 0) vecs << ",";
            vecs << "\"" << spawned_crates[i] << "\"";
        }
        vecs << "],\"crate_owned\":[";
        for (size_t i = 0; i < spawned_crate_owned.size(); ++i) {
            if (i > 0) vecs << ",";
            vecs << spawned_crate_owned[i];
        }
        vecs << "]"
             << ",\"crate_post_rng_crc\":" << crate_post_rng_crc
             << ",\"cr_crc\":" << cr_crc
             << ",\"cr_xfer\":\"" << cr_hex << "\""
             << ",\"crush\":[" << (crush_post_front?1:0) << "," << (crush_post_back?1:0)
             << "," << (crush_mc_front?1:0) << "," << (crush_mc_back?1:0) << "]"
             << ",\"crush_post_rng_crc\":" << crush_post_rng_crc
             << ",\"crush_crc\":" << crush_crc
             << ",\"crush_xfer\":\"" << crush_hex << "\""
             << ",\"dam\":[" << (wg1_dis?1:0) << "," << (wg2_dis?1:0) << "," << (nw_dis?1:0) << "]"
             << ",\"dam_crc\":" << dam_crc
             << ",\"dam_xfer\":\"" << dam_hex << "\""
             << ",\"rh_spawned\":" << (rh_spawned?1:0)
             << ",\"rh_hole_max_hp_bits\":" << oracle::fbits(rh_hole_max_hp)
             << ",\"rh_hole_hp_bits\":" << oracle::fbits(rh_hole_hp)
             << ",\"rh_hole_rad_bits\":" << oracle::fbits(rh_hole_radius)
             << ",\"rh_crc\":" << rh_crc
             << ",\"rh_xfer\":\"" << rh_hex << "\""
             << ",\"sp_crc\":" << sp_crc
             << ",\"sp_xfer\":\"" << sp_hex << "\""
             << ",\"sp_rt_xfer\":\"" << sp_rt_hex << "\""
             << ",\"sp_post_xfer\":\"" << sp_post_hex << "\""
             << ",\"ud\":[" << (ud_prod_pre?1:0) << "," << (ud_prod_post?1:0) << "]"
             << ",\"ud_crc\":" << ud_crc
             << ",\"ud_xfer\":\"" << ud_hex << "\""
             << "}"
             << "}";
    }

    std::ostringstream doc;
    doc << "{\n"
        << "  \"packet\": \"WP-5.2-A\",\n"
        << "  \"modules\": [\"DieModule\",\"CreateCrateDie\",\"CreateObjectDie\",\"CrushDie\",\"DamDie\",\"DestroyDie\",\"EjectPilotDie\",\"FXListDie\",\"KeepObjectDie\",\"RebuildHoleExposeDie\",\"SpecialPowerCompletionDie\",\"UpgradeDie\"],\n"
        << "  \"seed_count\": 1000,\n"
        << "  \"source_sha256\": \"" << oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Die/DieModule.cpp") << "\",\n"
        << "  \"seed0_diagnostics\": " << s0_diag << ",\n"
        << "  \"vectors\": [\n" << vecs.str() << "\n  ]\n}\n";

    oracle::write_file_atomic(out_path, doc.str());
    return 0;
}
