// WP-5.2-B UpgradeModule hierarchy (21 packets: PKT-193, 194, 200, 203, 214, 218,
// 221, 222, 223, 225, 227, 228, 229, 231, 247, 248, 249, 251, 253, 255, 256)
// C++ Golden Vector Generator (Zero-Stub Build against oracle_wp51_world.h).
//
// Canonical build & run:
//   python3 scripts/build_oracle.py "$(pwd)/tests/cpp_harness/gen_wp52_upgrade.cpp" \
//     -o /tmp/gen_wp52_upgrade --extra-flag="-I$(pwd)/tests/cpp_harness"
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy /tmp/gen_wp52_upgrade docs/golden_vectors/WP52_Upgrade.json

#include "oracle_wp51_world.h"

#include "Common/Energy.h"
#include "Common/Player.h"
#include "Common/ResourceGatheringManager.h"
#include "Common/Science.h"
#include "Common/SpecialPower.h"
#include "Common/Upgrade.h"
#include "GameClient/Drawable.h"
#include "GameLogic/ExperienceTracker.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/ActiveShroudUpgrade.h"
#include "GameLogic/Module/ArmorUpgrade.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/CommandSetUpgrade.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/CostModifierUpgrade.h"
#include "GameLogic/Module/ExperienceScalarUpgrade.h"
#include "GameLogic/Module/GrantScienceUpgrade.h"
#include "GameLogic/Module/LocomotorSetUpgrade.h"
#include "GameLogic/Module/MaxHealthUpgrade.h"
#include "GameLogic/Module/ModelConditionUpgrade.h"
#include "GameLogic/Module/ObjectCreationUpgrade.h"
#include "GameLogic/Module/PassengersFireUpgrade.h"
#include "GameLogic/Module/PowerPlantUpgrade.h"
#include "GameLogic/Module/RadarUpgrade.h"
#include "GameLogic/Module/ReplaceObjectUpgrade.h"
#include "GameLogic/Module/SpecialPowerModule.h"
#include "GameLogic/Module/StatusBitsUpgrade.h"
#include "GameLogic/Module/StealthUpgrade.h"
#include "GameLogic/Module/SubObjectsUpgrade.h"
#include "GameLogic/Module/UnpauseSpecialPowerUpgrade.h"
#include "GameLogic/Module/UpgradeModule.h"
#include "GameLogic/Module/WeaponBonusUpgrade.h"
#include "GameLogic/Module/WeaponSetUpgrade.h"

#include <list>

struct PlayerUpgTag { using type = UpgradeMaskType Player::*; };
template struct MemberAccess<PlayerUpgTag, &Player::m_upgradesCompleted>;
UpgradeMaskType Player::* access_member(PlayerUpgTag);

struct PlayerSciTag { using type = ScienceVec Player::*; };
template struct MemberAccess<PlayerSciTag, &Player::m_sciences>;
ScienceVec Player::* access_member(PlayerSciTag);

struct PlayerRadarCountTag { using type = Int Player::*; };
template struct MemberAccess<PlayerRadarCountTag, &Player::m_radarCount>;
Int Player::* access_member(PlayerRadarCountTag);

struct PlayerDisableProofRadarTag { using type = Int Player::*; };
template struct MemberAccess<PlayerDisableProofRadarTag, &Player::m_disableProofRadarCount>;
Int Player::* access_member(PlayerDisableProofRadarTag);

struct PlayerEnergyTag { using type = Energy Player::*; };
template struct MemberAccess<PlayerEnergyTag, &Player::m_energy>;
Energy Player::* access_member(PlayerEnergyTag);

struct EnergyProdTag { using type = Int Energy::*; };
template struct MemberAccess<EnergyProdTag, &Energy::m_energyProduction>;
Int Energy::* access_member(EnergyProdTag);

struct RgmCentersTag { using type = std::list<ObjectID> ResourceGatheringManager::*; };
template struct MemberAccess<RgmCentersTag, &ResourceGatheringManager::m_supplyCenters>;
std::list<ObjectID> ResourceGatheringManager::* access_member(RgmCentersTag);

struct AiCurLocoSetTag { using type = LocomotorSetType AIUpdateInterface::*; };
template struct MemberAccess<AiCurLocoSetTag, &AIUpdateInterface::m_curLocomotorSet>;
LocomotorSetType AIUpdateInterface::* access_member(AiCurLocoSetTag);

struct SpPausedCountTag { using type = Int SpecialPowerModule::*; };
template struct MemberAccess<SpPausedCountTag, &SpecialPowerModule::m_pausedCount>;
Int SpecialPowerModule::* access_member(SpPausedCountTag);

static UpgradeModule* find_upgrade_module(Object* obj, size_t idx = 0) {
    size_t seen = 0;
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m) {
        if ((*m)->getUpgrade()) {
            if (seen == idx) {
                return static_cast<UpgradeModule*>(*m);
            }
            ++seen;
        }
    }
    return nullptr;
}

static SpecialPowerModule* find_sp_module(Object* obj) {
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m) {
        if ((*m)->getSpecialPower()) {
            return static_cast<SpecialPowerModule*>(*m);
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

int main(int argc, char** argv) {
    const char* out_path = (argc > 1) ? argv[1] : "docs/golden_vectors/WP52_Upgrade.json";
    oracle_wp51::World w;

    w.load_ini(
        "Upgrade UpgA\n  Type = OBJECT\n  BuildTime = 1.0\n  BuildCost = 100\nEnd\n"
        "Upgrade UpgB\n  Type = OBJECT\n  BuildTime = 1.0\n  BuildCost = 100\nEnd\n"
        "Upgrade UpgConflict\n  Type = OBJECT\n  BuildTime = 1.0\n  BuildCost = 100\nEnd\n"
        "Upgrade UpgRemovable\n  Type = OBJECT\n  BuildTime = 1.0\n  BuildCost = 100\nEnd\n"
        "Upgrade UpgPlayerAlt\n  Type = PLAYER\n  BuildTime = 1.0\n  BuildCost = 100\nEnd\n"
        "Upgrade UpgObjAlt\n  Type = OBJECT\n  BuildTime = 1.0\n  BuildCost = 100\nEnd\n"
        "Upgrade Upgrade_AmericaChemicalSuits\n  Type = PLAYER\n  BuildTime = 1.0\n  BuildCost = 100\nEnd\n"
        "Science SCI_WP52_A\n  SciencePurchasePointCost = 1\n  IsGrantable = Yes\nEnd\n"
        "Science SCI_WP52_B\n  SciencePurchasePointCost = 1\n  IsGrantable = Yes\nEnd\n"
        "FXList FX_WP52_Upg\nEnd\n"
        "SpecialPower SpWP52_A\n  Enum = SPECIAL_CASH_HACK\n  ReloadTime = 1000\nEnd\n"
        "CommandSet CmdBase\nEnd\n"
        "CommandSet CmdMain\nEnd\n"
        "CommandSet CmdAlt\nEnd\n"
        "Locomotor LocoNormal\n  Speed = 30\n  SpeedDamaged = 20\n  TurnRate = 90\n  TurnRateDamaged = 90\n  Acceleration = 10\n  Braking = 10\n  Appearance = TREADS\nEnd\n"
        "Locomotor LocoUpgraded\n  Speed = 60\n  SpeedDamaged = 40\n  TurnRate = 120\n  TurnRateDamaged = 120\n  Acceleration = 20\n  Braking = 20\n  Appearance = TREADS\nEnd\n"
        "Object WP52_Spawned\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 50\n    InitialHealth = 50\n  End\n"
        "End\n"
        "Object WP52_ReplTarget\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 250\n    InitialHealth = 250\n  End\n"
        "  Behavior = SupplyCenterCreate ModuleTag_SC\n  End\n"
        "End\n"
        "ObjectCreationList OCL_WP52_Test\n"
        "  CreateObject\n    ObjectNames = WP52_Spawned\n    Count = 1\n  End\n"
        "End\n"
    );

    Player* p_a = ThePlayerList->getNthPlayer(0);
    Player* p_b = ThePlayerList->getNthPlayer(1);

    const UpgradeTemplate* ut_a = TheUpgradeCenter->findUpgrade("UpgA");
    const UpgradeTemplate* ut_b = TheUpgradeCenter->findUpgrade("UpgB");
    const UpgradeTemplate* ut_conf = TheUpgradeCenter->findUpgrade("UpgConflict");
    const UpgradeTemplate* ut_rem = TheUpgradeCenter->findUpgrade("UpgRemovable");
    const UpgradeTemplate* ut_ply_alt = TheUpgradeCenter->findUpgrade("UpgPlayerAlt");
    const UpgradeTemplate* ut_obj_alt = TheUpgradeCenter->findUpgrade("UpgObjAlt");
    const UpgradeTemplate* ut_chem = TheUpgradeCenter->findUpgrade("Upgrade_AmericaChemicalSuits");

    ScienceType sci_a = TheScienceStore->friend_lookupScience("SCI_WP52_A");
    ScienceType sci_b = TheScienceStore->friend_lookupScience("SCI_WP52_B");

    static const char* kChangeTypes[4] = {
        "SAME_CURRENTHEALTH", "PRESERVE_RATIO", "ADD_CURRENT_HEALTH_TOO", "FULLY_HEAL"
    };
    static const char* kMcFlags[4] = {
        "USER_1", "USER_2", "WEAPONSET_PLAYER_UPGRADE", "ARMORSET_CRATEUPGRADE_ONE"
    };
    static const char* kKindOfs[3] = { "INFANTRY", "VEHICLE", "STRUCTURE" };
    static const KindOfType kKindOfEnums[3] = { KINDOF_INFANTRY, KINDOF_VEHICLE, KINDOF_STRUCTURE };
    static const char* kPctStrs[4] = { "-15%", "20%", "-25.5%", "10%" };

    std::string s0_diag;
    std::ostringstream vecs;

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed ^ 0x52b09876u;
        if (rng == 0) rng = 0x2468ace1u;

        UnsignedInt frame0 = 20u + (oracle::splitmix32(rng) % 500u);
        w.set_frame(frame0);

        bool req_all = (seed == 0) ? true : ((oracle::splitmix32(rng) & 1u) != 0);
        bool give_conflict_first = (seed == 0) ? false : ((oracle::splitmix32(rng) % 4u) == 0);
        bool chem_armor = (seed == 0) ? true : ((oracle::splitmix32(rng) & 1u) != 0);
        int cmd_alt_mode = (seed == 0) ? 1 : static_cast<int>(oracle::splitmix32(rng) % 4u);
        // cmd_alt_mode: 0=no TriggerAlt, 1=TriggerAlt=UpgPlayerAlt (granted), 2=TriggerAlt=UpgObjAlt (granted), 3=TriggerAlt=UpgObjAlt (NOT granted)
        int cost_ko_idx = static_cast<int>(oracle::splitmix32(rng) % 3u);
        int cost_pct_idx = static_cast<int>(oracle::splitmix32(rng) % 4u);
        bool cost_capture = (seed == 0) ? true : ((oracle::splitmix32(rng) & 1u) != 0);
        Real xp_add = 0.25f * static_cast<Real>(1u + (oracle::splitmix32(rng) % 8u));
        bool grant_sci_b = (oracle::splitmix32(rng) & 1u) != 0;
        int hp_change_idx = static_cast<int>(oracle::splitmix32(rng) % 4u);
        Real hp_add = 50.0f * static_cast<Real>(1u + (oracle::splitmix32(rng) % 6u));
        Real hp_dmg = 20.0f + 10.0f * static_cast<Real>(oracle::splitmix32(rng) % 5u);
        int mc_idx = static_cast<int>(oracle::splitmix32(rng) % 4u);
        Real shroud_range = 100.0f + 25.0f * static_cast<Real>(oracle::splitmix32(rng) % 10u);
        bool radar_disable_proof = (seed == 0) ? true : ((oracle::splitmix32(rng) & 1u) != 0);
        bool radar_disabled_on_del = (seed == 0) ? false : ((oracle::splitmix32(rng) % 3u) == 0);
        bool sp_starts_ready = (seed == 0) ? true : ((oracle::splitmix32(rng) & 1u) != 0);
        bool subobj_conflict = (seed == 0) ? false : ((oracle::splitmix32(rng) % 4u) == 0);

        // Reset player state deterministically
        (p_a->*access_member(PlayerUpgTag())) = UpgradeMaskType();
        (p_b->*access_member(PlayerUpgTag())) = UpgradeMaskType();
        (p_a->*access_member(PlayerSciTag())).clear();
        (p_b->*access_member(PlayerSciTag())).clear();
        p_a->*access_member(PlayerRadarCountTag()) = 0;
        p_a->*access_member(PlayerDisableProofRadarTag()) = 0;
        p_b->*access_member(PlayerRadarCountTag()) = 0;
        p_b->*access_member(PlayerDisableProofRadarTag()) = 0;
        (p_a->*access_member(PlayerEnergyTag())).*access_member(EnergyProdTag()) = 0;
        (p_b->*access_member(PlayerEnergyTag())).*access_member(EnergyProdTag()) = 0;
        if (ResourceGatheringManager* rgm = p_a->getResourceGatheringManager()) {
            (rgm->*access_member(RgmCentersTag())).clear();
        }

        std::string pfx = "U52_" + std::to_string(seed) + "_";
        std::string t_mux = pfx + "Mux";
        std::string t_shroud = pfx + "Shroud";
        std::string t_armor = pfx + "Armor";
        std::string t_cmd = pfx + "Cmd";
        std::string t_cost = pfx + "Cost";
        std::string t_sci = pfx + "Sci";
        std::string t_loco = pfx + "Loco";
        std::string t_hp = pfx + "HP";
        std::string t_mc = pfx + "MC";
        std::string t_ocl = pfx + "OCL";
        std::string t_pass = pfx + "Pass";
        std::string t_pp = pfx + "PP";
        std::string t_rad = pfx + "Rad";
        std::string t_repl = pfx + "Repl";
        std::string t_stat = pfx + "Stat";
        std::string t_stl = pfx + "Stl";
        std::string t_sub = pfx + "Sub";
        std::string t_sp = pfx + "SP";
        std::string t_wb = pfx + "WB";
        std::string t_ws = pfx + "WS";

        std::ostringstream ini_ss;
        // 1) Mux + ExperienceScalarUpgrade + Removable StatusBitsUpgrade
        ini_ss << "Object " << t_mux << "\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = ExperienceScalarUpgrade ModuleTag_Mux\n"
               << "    TriggeredBy = UpgA UpgB\n"
               << "    ConflictsWith = UpgConflict\n"
               << "    RemovesUpgrades = UpgRemovable\n"
               << "    FXListUpgrade = FX_WP52_Upg\n"
               << "    RequiresAllTriggers = " << (req_all ? "Yes" : "No") << "\n"
               << "    AddXPScalar = " << xp_add << "\n"
               << "  End\n"
               << "  Behavior = StatusBitsUpgrade ModuleTag_Rem\n"
               << "    TriggeredBy = UpgRemovable\n"
               << "    StatusToSet = IMMOBILE\n"
               << "  End\nEnd\n";

        // 2) ActiveShroudUpgrade
        ini_ss << "Object " << t_shroud << "\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = ActiveShroudUpgrade ModuleTag_Upg\n"
               << "    TriggeredBy = UpgA\n"
               << "    NewShroudRange = " << shroud_range << "\n"
               << "  End\nEnd\n";

        // 3) ArmorUpgrade
        ini_ss << "Object " << t_armor << "\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = ArmorUpgrade ModuleTag_Upg\n"
               << "    TriggeredBy = " << (chem_armor ? "Upgrade_AmericaChemicalSuits" : "UpgA") << "\n"
               << "  End\nEnd\n";

        // 4) CommandSetUpgrade
        ini_ss << "Object " << t_cmd << "\n"
               << "  CommandSet = CmdBase\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = CommandSetUpgrade ModuleTag_Upg\n"
               << "    TriggeredBy = UpgA\n"
               << "    CommandSet = CmdMain\n"
               << "    CommandSetAlt = CmdAlt\n";
        if (cmd_alt_mode == 1) ini_ss << "    TriggerAlt = UpgPlayerAlt\n";
        else if (cmd_alt_mode >= 2) ini_ss << "    TriggerAlt = UpgObjAlt\n";
        ini_ss << "  End\nEnd\n";

        // 5) CostModifierUpgrade
        ini_ss << "Object " << t_cost << "\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = CostModifierUpgrade ModuleTag_Upg\n"
               << "    TriggeredBy = UpgA\n"
               << "    EffectKindOf = " << kKindOfs[cost_ko_idx] << "\n"
               << "    Percentage = " << kPctStrs[cost_pct_idx] << "\n"
               << "  End\nEnd\n";

        // 6) GrantScienceUpgrade
        ini_ss << "Object " << t_sci << "\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = GrantScienceUpgrade ModuleTag_Upg\n"
               << "    TriggeredBy = UpgA\n"
               << "    GrantScience = " << (grant_sci_b ? "SCI_WP52_B" : "SCI_WP52_A") << "\n"
               << "  End\nEnd\n";

        // 7) LocomotorSetUpgrade
        ini_ss << "Object " << t_loco << "\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
               << "  Locomotor = SET_NORMAL LocoNormal\n"
               << "  Locomotor = SET_NORMAL_UPGRADED LocoUpgraded\n"
               << "  Behavior = LocomotorSetUpgrade ModuleTag_Upg\n"
               << "    TriggeredBy = UpgA\n"
               << "  End\nEnd\n";

        // 8) MaxHealthUpgrade
        ini_ss << "Object " << t_hp << "\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = MaxHealthUpgrade ModuleTag_Upg\n"
               << "    TriggeredBy = UpgA\n"
               << "    AddMaxHealth = " << hp_add << "\n"
               << "    ChangeType = " << kChangeTypes[hp_change_idx] << "\n"
               << "  End\nEnd\n";

        // 9) ModelConditionUpgrade
        ini_ss << "Object " << t_mc << "\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = ModelConditionUpgrade ModuleTag_Upg\n"
               << "    TriggeredBy = UpgA\n"
               << "    ConditionFlag = " << kMcFlags[mc_idx] << "\n"
               << "  End\nEnd\n";

        // 10) ObjectCreationUpgrade
        ini_ss << "Object " << t_ocl << "\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = ObjectCreationUpgrade ModuleTag_Upg\n"
               << "    TriggeredBy = UpgA\n"
               << "    UpgradeObject = OCL_WP52_Test\n"
               << "  End\nEnd\n";

        // 11) PassengersFireUpgrade
        ini_ss << "Object " << t_pass << "\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = OpenContain ModuleTag_Contain\n"
               << "    ContainMax = 4\n"
               << "  End\n"
               << "  Behavior = PassengersFireUpgrade ModuleTag_Upg\n"
               << "    TriggeredBy = UpgA\n"
               << "  End\nEnd\n";

        // 12) PowerPlantUpgrade
        ini_ss << "Object " << t_pp << "\n"
               << "  EnergyBonus = 7\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = PowerPlantUpgrade ModuleTag_Upg\n"
               << "    TriggeredBy = UpgA\n"
               << "  End\nEnd\n";

        // 13) RadarUpgrade
        ini_ss << "Object " << t_rad << "\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = RadarUpgrade ModuleTag_Upg\n"
               << "    TriggeredBy = UpgA\n"
               << "    DisableProof = " << (radar_disable_proof ? "Yes" : "No") << "\n"
               << "  End\nEnd\n";

        // 14) ReplaceObjectUpgrade
        ini_ss << "Object " << t_repl << "\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = ReplaceObjectUpgrade ModuleTag_Upg\n"
               << "    TriggeredBy = UpgA\n"
               << "    ReplaceObject = WP52_ReplTarget\n"
               << "  End\nEnd\n";

        // 15) StatusBitsUpgrade
        ini_ss << "Object " << t_stat << "\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = StatusBitsUpgrade ModuleTag_Upg\n"
               << "    TriggeredBy = UpgA\n"
               << "    StatusToSet = CAN_STEALTH IMMOBILE\n"
               << "    StatusToClear = UNDER_CONSTRUCTION\n"
               << "  End\nEnd\n";

        // 16) StealthUpgrade
        ini_ss << "Object " << t_stl << "\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = StealthUpgrade ModuleTag_Upg\n"
               << "    TriggeredBy = UpgA\n"
               << "  End\nEnd\n";

        // 17) SubObjectsUpgrade (multi-line append!)
        ini_ss << "Object " << t_sub << "\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = SubObjectsUpgrade ModuleTag_Upg\n"
               << "    TriggeredBy = UpgA\n"
               << "    ConflictsWith = UpgConflict\n"
               << "    ShowSubObjects = TurretA Barrel1\n"
               << "    ShowSubObjects = Barrel2\n"
               << "    HideSubObjects = OldMeshA\n"
               << "    HideSubObjects = OldMeshB\n"
               << "  End\nEnd\n";

        // 18) UnpauseSpecialPowerUpgrade
        ini_ss << "Object " << t_sp << "\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = CashHackSpecialPower ModuleTag_SP\n"
               << "    SpecialPowerTemplate = SpWP52_A\n"
               << "    StartsPaused = Yes\n"
               << "    StartsReady = " << (sp_starts_ready ? "Yes" : "No") << "\n"
               << "  End\n"
               << "  Behavior = UnpauseSpecialPowerUpgrade ModuleTag_Upg\n"
               << "    TriggeredBy = UpgA\n"
               << "    SpecialPowerTemplate = SpWP52_A\n"
               << "  End\nEnd\n";

        // 19) WeaponBonusUpgrade & 20) WeaponSetUpgrade
        ini_ss << "Object " << t_wb << "\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = WeaponBonusUpgrade ModuleTag_Upg\n"
               << "    TriggeredBy = UpgA\n"
               << "  End\nEnd\n"
               << "Object " << t_ws << "\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = WeaponSetUpgrade ModuleTag_Upg\n"
               << "    TriggeredBy = UpgA\n"
               << "  End\nEnd\n";

        w.load_ini(ini_ss.str(), INI_LOAD_OVERWRITE);

        // --- Exercise 1: Mux + ExperienceScalarUpgrade ---
        Object* o_mux = w.spawn_object(t_mux.c_str(), w.team_a);
        UpgradeModule* um_mux = find_upgrade_module(o_mux, 0);
        UpgradeModule* um_rem = find_upgrade_module(o_mux, 1);
        o_mux->giveUpgrade(ut_rem);
        bool rem_pre = um_rem->isAlreadyUpgraded();
        if (give_conflict_first) o_mux->giveUpgrade(ut_conf);
        bool would_a = um_mux->wouldUpgrade(ut_a->getUpgradeMask());
        UpgradeMaskType ab_mask = ut_a->getUpgradeMask();
        ab_mask.set(ut_b->getUpgradeMask());
        bool would_ab = um_mux->wouldUpgrade(ab_mask);
        bool cond_ab = um_mux->testUpgradeConditions(ab_mask);
        bool is_sub_false = um_mux->isSubObjectsUpgrade();

        o_mux->giveUpgrade(ut_a);
        bool upg_after_a = um_mux->isAlreadyUpgraded();
        o_mux->giveUpgrade(ut_b);
        bool upg_after_ab = um_mux->isAlreadyUpgraded();
        bool rem_after = um_rem->isAlreadyUpgraded();
        Real xp_post1 = o_mux->getExperienceTracker()->getExperienceScalar();
        um_mux->forceRefreshUpgrade();
        Real xp_post2 = o_mux->getExperienceTracker()->getExperienceScalar();
        bool trig_a_true = um_mux->isTriggeredBy("UpgA");
        bool trig_x_false = um_mux->isTriggeredBy("UpgUnknown");

        uint32_t mux_xfer_crc = 0;
        std::string mux_hex = oracle_wp51::World::snapshot_save_hex(um_mux, &mux_xfer_crc);
        uint32_t mux_crc = oracle_wp51::World::snapshot_crc(um_mux);

        o_mux->removeUpgrade(ut_a);
        bool upg_after_reset = um_mux->isAlreadyUpgraded();

        // --- Exercise 2: ActiveShroudUpgrade ---
        Object* o_shroud = w.spawn_object(t_shroud.c_str(), w.team_a);
        Real shroud_pre = o_shroud->getShroudRange();
        o_shroud->giveUpgrade(ut_a);
        Real shroud_post = o_shroud->getShroudRange();
        std::string shroud_hex = oracle_wp51::World::snapshot_save_hex(find_upgrade_module(o_shroud));

        // --- Exercise 3: ArmorUpgrade ---
        Object* o_armor = w.spawn_object(t_armor.c_str(), w.team_a);
        o_armor->giveUpgrade(chem_armor ? ut_chem : ut_a);
        bool armor_flag = o_armor->getBodyModule()->testArmorSetFlag(ARMORSET_PLAYER_UPGRADE);
        std::string armor_hex = oracle_wp51::World::snapshot_save_hex(find_upgrade_module(o_armor));

        // --- Exercise 4: CommandSetUpgrade ---
        Object* o_cmd = w.spawn_object(t_cmd.c_str(), w.team_a);
        if (cmd_alt_mode == 1) {
            p_a->addUpgrade(ut_ply_alt, UPGRADE_STATUS_COMPLETE);
        } else if (cmd_alt_mode == 2) {
            o_cmd->giveUpgrade(ut_obj_alt);
        }
        o_cmd->giveUpgrade(ut_a);
        std::string cmd_str = o_cmd->getCommandSetString().str();

        // --- Exercise 5: CostModifierUpgrade ---
        Object* o_cost = w.spawn_object(t_cost.c_str(), w.team_a);
        KindOfMaskType ko_test;
        ko_test.set(kKindOfEnums[cost_ko_idx]);
        Real cost_pre = p_a->getProductionCostChangeBasedOnKindOf(ko_test);
        o_cost->giveUpgrade(ut_a);
        Real cost_post = p_a->getProductionCostChangeBasedOnKindOf(ko_test);
        Real cost_cap_a = cost_post;
        Real cost_cap_b = p_b->getProductionCostChangeBasedOnKindOf(ko_test);
        if (cost_capture) {
            o_cost->setTeam(w.team_b);
            cost_cap_a = p_a->getProductionCostChangeBasedOnKindOf(ko_test);
            cost_cap_b = p_b->getProductionCostChangeBasedOnKindOf(ko_test);
        }
        UpgradeModule* um_cost = find_upgrade_module(o_cost);
        bool cost_upg_pre_del = um_cost->isAlreadyUpgraded();
        TheGameLogic->destroyObject(o_cost);
        TheGameLogic->processDestroyList();
        Real cost_after_del_a = p_a->getProductionCostChangeBasedOnKindOf(ko_test);
        Real cost_after_del_b = p_b->getProductionCostChangeBasedOnKindOf(ko_test);

        // --- Exercise 6: GrantScienceUpgrade ---
        Object* o_sci = w.spawn_object(t_sci.c_str(), w.team_a);
        o_sci->giveUpgrade(ut_a);
        bool has_sci_a = p_a->hasScience(sci_a);
        bool has_sci_b = p_a->hasScience(sci_b);

        // --- Exercise 7: LocomotorSetUpgrade ---
        Object* o_loco = w.spawn_object(t_loco.c_str(), w.team_a);
        Int loco_pre = static_cast<Int>(o_loco->getAIUpdateInterface()->*access_member(AiCurLocoSetTag()));
        o_loco->giveUpgrade(ut_a);
        Int loco_post = static_cast<Int>(o_loco->getAIUpdateInterface()->*access_member(AiCurLocoSetTag()));

        // --- Exercise 8: MaxHealthUpgrade ---
        Object* o_hp = w.spawn_object(t_hp.c_str(), w.team_a);
        o_hp->getBodyModule()->internalChangeHealth(-hp_dmg);
        o_hp->giveUpgrade(ut_a);
        Real hp_post = o_hp->getBodyModule()->getHealth();
        Real max_hp_post = o_hp->getBodyModule()->getMaxHealth();

        // --- Exercise 9: ModelConditionUpgrade ---
        Object* o_mc = w.spawn_object(t_mc.c_str(), w.team_a);
        o_mc->giveUpgrade(ut_a);
        bool mc_set = o_mc->getDrawable()->getModelConditionFlags().test(
            ModelConditionFlags::getSingleBitFromName(kMcFlags[mc_idx])
        );

        // --- Exercise 10: ObjectCreationUpgrade ---
        Object* o_ocl = w.spawn_object(t_ocl.c_str(), w.team_a);
        uint32_t cnt_before_ocl = obj_count_live();
        o_ocl->giveUpgrade(ut_a);
        uint32_t cnt_after_ocl = obj_count_live();

        // --- Exercise 11: PassengersFireUpgrade ---
        Object* o_pass = w.spawn_object(t_pass.c_str(), w.team_a);
        bool pass_pre = o_pass->getContain()->isPassengerAllowedToFire();
        o_pass->giveUpgrade(ut_a);
        bool pass_post = o_pass->getContain()->isPassengerAllowedToFire();

        // --- Exercise 12: PowerPlantUpgrade ---
        Object* o_pp = w.spawn_object(t_pp.c_str(), w.team_a);
        Int pp_pre_a = p_a->getEnergy()->getProduction();
        o_pp->giveUpgrade(ut_a);
        Int pp_post_a = p_a->getEnergy()->getProduction();
        o_pp->setTeam(w.team_b);
        Int pp_cap_a = p_a->getEnergy()->getProduction();
        Int pp_cap_b = p_b->getEnergy()->getProduction();
        TheGameLogic->destroyObject(o_pp);
        TheGameLogic->processDestroyList();
        Int pp_del_b = p_b->getEnergy()->getProduction();

        // --- Exercise 13: RadarUpgrade ---
        Object* o_rad = w.spawn_object(t_rad.c_str(), w.team_a);
        o_rad->giveUpgrade(ut_a);
        Int rad_a = p_a->*access_member(PlayerRadarCountTag());
        Int dprad_a = p_a->*access_member(PlayerDisableProofRadarTag());
        o_rad->setTeam(w.team_b);
        Int rad_cap_a = p_a->*access_member(PlayerRadarCountTag());
        Int rad_cap_b = p_b->*access_member(PlayerRadarCountTag());
        if (radar_disabled_on_del) {
            o_rad->setDisabled(DISABLED_EMP);
        }
        TheGameLogic->destroyObject(o_rad);
        TheGameLogic->processDestroyList();
        Int rad_del_b = p_b->*access_member(PlayerRadarCountTag());

        // --- Exercise 14: ReplaceObjectUpgrade ---
        Coord3D repl_pos = {120.0f, 240.0f, 0.0f};
        Object* o_repl = w.spawn_object(t_repl.c_str(), w.team_a, repl_pos, 1.25f);
        size_t sc_before_repl = (p_a->getResourceGatheringManager()->*access_member(RgmCentersTag())).size();
        o_repl->giveUpgrade(ut_a);
        bool repl_old_destroyed = o_repl->isDestroyed();
        size_t sc_after_repl = (p_a->getResourceGatheringManager()->*access_member(RgmCentersTag())).size();

        // --- Exercise 15: StatusBitsUpgrade ---
        Object* o_stat = w.spawn_object(t_stat.c_str(), w.team_a);
        o_stat->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
        o_stat->giveUpgrade(ut_a);
        bool stat_stealth = o_stat->getStatusBits().test(OBJECT_STATUS_CAN_STEALTH);
        bool stat_immob = o_stat->getStatusBits().test(OBJECT_STATUS_IMMOBILE);
        bool stat_uc = o_stat->getStatusBits().test(OBJECT_STATUS_UNDER_CONSTRUCTION);

        // --- Exercise 16: StealthUpgrade ---
        Object* o_stl = w.spawn_object(t_stl.c_str(), w.team_a);
        o_stl->giveUpgrade(ut_a);
        bool stl_flag = o_stl->getStatusBits().test(OBJECT_STATUS_CAN_STEALTH);

        // --- Exercise 17: SubObjectsUpgrade ---
        Object* o_sub = w.spawn_object(t_sub.c_str(), w.team_a);
        UpgradeModule* um_sub = find_upgrade_module(o_sub);
        bool is_sub_true = um_sub->isSubObjectsUpgrade();
        if (subobj_conflict) o_sub->giveUpgrade(ut_conf);
        o_sub->giveUpgrade(ut_a);
        bool sub_upg_done = um_sub->isAlreadyUpgraded();
        std::string sub_hex = oracle_wp51::World::snapshot_save_hex(um_sub);

        // --- Exercise 18: UnpauseSpecialPowerUpgrade ---
        Object* o_sp = w.spawn_object(t_sp.c_str(), w.team_a);
        SpecialPowerModule* sp_mod = find_sp_module(o_sp);
        bool sp_paused_pre = (sp_mod->*access_member(SpPausedCountTag())) > 0;
        o_sp->giveUpgrade(ut_a);
        bool sp_paused_post = (sp_mod->*access_member(SpPausedCountTag())) > 0;
        UnsignedInt sp_ready_post = static_cast<const SpecialPowerModule*>(sp_mod)->getReadyFrame();

        // --- Exercise 19 & 20: WeaponBonusUpgrade & WeaponSetUpgrade ---
        Object* o_wb = w.spawn_object(t_wb.c_str(), w.team_a);
        o_wb->giveUpgrade(ut_a);
        bool wb_flag = o_wb->testWeaponBonusCondition(WEAPONBONUSCONDITION_PLAYER_UPGRADE);

        Object* o_ws = w.spawn_object(t_ws.c_str(), w.team_a);
        o_ws->giveUpgrade(ut_a);
        bool ws_flag = o_ws->testWeaponSetFlag(WEAPONSET_PLAYER_UPGRADE);

        // Clean up all remaining live objects before next seed
        std::vector<Object*> to_kill;
        for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
            to_kill.push_back(o);
        }
        for (Object* o : to_kill) {
            TheGameLogic->destroyObject(o);
        }
        TheGameLogic->processDestroyList();

        if (seed == 0) {
            std::ostringstream d;
            d << "{"
              << "\"mux\":{\"would_a\":" << (would_a?1:0) << ",\"would_ab\":" << (would_ab?1:0)
              << ",\"cond_ab\":" << (cond_ab?1:0) << ",\"upg_a\":" << (upg_after_a?1:0)
              << ",\"upg_ab\":" << (upg_after_ab?1:0) << ",\"rem_pre\":" << (rem_pre?1:0)
              << ",\"rem_after\":" << (rem_after?1:0) << ",\"xp_post1_bits\":" << oracle::fbits(xp_post1)
              << ",\"xp_post2_bits\":" << oracle::fbits(xp_post2) << ",\"upg_reset\":" << (upg_after_reset?1:0)
              << ",\"trig_a\":" << (trig_a_true?1:0) << ",\"trig_x\":" << (trig_x_false?1:0)
              << ",\"xfer\":\"" << mux_hex << "\",\"crc\":" << mux_crc << "},"
              << "\"shroud\":{\"pre_bits\":" << oracle::fbits(shroud_pre) << ",\"post_bits\":" << oracle::fbits(shroud_post) << "},"
              << "\"armor\":{\"flag\":" << (armor_flag?1:0) << "},"
              << "\"cmd\":{\"str\":\"" << cmd_str << "\"},"
              << "\"cost\":{\"pre_bits\":" << oracle::fbits(cost_pre) << ",\"post_bits\":" << oracle::fbits(cost_post)
              << ",\"cap_a_bits\":" << oracle::fbits(cost_cap_a) << ",\"cap_b_bits\":" << oracle::fbits(cost_cap_b)
              << ",\"del_b_bits\":" << oracle::fbits(cost_after_del_b) << "},"
              << "\"sci\":{\"a\":" << (has_sci_a?1:0) << ",\"b\":" << (has_sci_b?1:0) << "},"
              << "\"loco\":{\"pre\":" << loco_pre << ",\"post\":" << loco_post << "},"
              << "\"hp\":{\"hp_bits\":" << oracle::fbits(hp_post) << ",\"max_bits\":" << oracle::fbits(max_hp_post) << "},"
              << "\"mc\":{\"set\":" << (mc_set?1:0) << "},"
              << "\"ocl\":{\"before\":" << cnt_before_ocl << ",\"after\":" << cnt_after_ocl << "},"
              << "\"pass\":{\"pre\":" << (pass_pre?1:0) << ",\"post\":" << (pass_post?1:0) << "},"
              << "\"pp\":{\"pre_a\":" << pp_pre_a << ",\"post_a\":" << pp_post_a
              << ",\"cap_a\":" << pp_cap_a << ",\"cap_b\":" << pp_cap_b << ",\"del_b\":" << pp_del_b << "},"
              << "\"rad\":{\"a\":" << rad_a << ",\"dp_a\":" << dprad_a
              << ",\"cap_a\":" << rad_cap_a << ",\"cap_b\":" << rad_cap_b << ",\"del_b\":" << rad_del_b << "},"
              << "\"repl\":{\"old_dest\":" << (repl_old_destroyed?1:0) << ",\"sc_pre\":" << sc_before_repl << ",\"sc_post\":" << sc_after_repl << "},"
              << "\"stat\":{\"stl\":" << (stat_stealth?1:0) << ",\"imm\":" << (stat_immob?1:0) << ",\"uc\":" << (stat_uc?1:0) << "},"
              << "\"stl\":{\"flag\":" << (stl_flag?1:0) << "},"
              << "\"sub\":{\"is_sub\":" << (is_sub_true?1:0) << ",\"done\":" << (sub_upg_done?1:0) << ",\"xfer\":\"" << sub_hex << "\"},"
              << "\"sp\":{\"paused_pre\":" << (sp_paused_pre?1:0) << ",\"paused_post\":" << (sp_paused_post?1:0) << ",\"ready\":" << sp_ready_post << "},"
              << "\"wb\":{\"flag\":" << (wb_flag?1:0) << "},"
              << "\"ws\":{\"flag\":" << (ws_flag?1:0) << "}"
              << "}";
            s0_diag = d.str();
        }

        if (seed > 0) vecs << ",\n";
        vecs << "    {"
             << "\"seed\":" << seed
             << ",\"in\":{\"frame\":" << frame0
             << ",\"req_all\":" << (req_all ? 1 : 0)
             << ",\"conflict_first\":" << (give_conflict_first ? 1 : 0)
             << ",\"chem_armor\":" << (chem_armor ? 1 : 0)
             << ",\"cmd_alt_mode\":" << cmd_alt_mode
             << ",\"cost_ko_idx\":" << cost_ko_idx
             << ",\"cost_pct_idx\":" << cost_pct_idx
             << ",\"cost_capture\":" << (cost_capture ? 1 : 0)
             << ",\"xp_add_bits\":" << oracle::fbits(xp_add)
             << ",\"grant_sci_b\":" << (grant_sci_b ? 1 : 0)
             << ",\"hp_change_idx\":" << hp_change_idx
             << ",\"hp_add_bits\":" << oracle::fbits(hp_add)
             << ",\"hp_dmg_bits\":" << oracle::fbits(hp_dmg)
             << ",\"mc_idx\":" << mc_idx
             << ",\"shroud_bits\":" << oracle::fbits(shroud_range)
             << ",\"radar_dp\":" << (radar_disable_proof ? 1 : 0)
             << ",\"radar_dis_del\":" << (radar_disabled_on_del ? 1 : 0)
             << ",\"sp_ready\":" << (sp_starts_ready ? 1 : 0)
             << ",\"sub_conflict\":" << (subobj_conflict ? 1 : 0)
             << "}"
             << ",\"out\":{"
             << "\"mux\":[" << (would_a?1:0) << "," << (would_ab?1:0) << "," << (cond_ab?1:0)
             << "," << (is_sub_false?1:0) << "," << (upg_after_a?1:0) << "," << (upg_after_ab?1:0)
             << "," << (rem_pre?1:0) << "," << (rem_after?1:0) << "," << (upg_after_reset?1:0)
             << "," << (trig_a_true?1:0) << "," << (trig_x_false?1:0) << "]"
             << ",\"xp_bits\":[" << oracle::fbits(xp_post1) << "," << oracle::fbits(xp_post2) << "]"
             << ",\"mux_crc\":" << mux_crc
             << ",\"mux_xfer_crc\":" << mux_xfer_crc
             << ",\"mux_xfer\":\"" << mux_hex << "\""
             << ",\"shroud_out\":[" << oracle::fbits(shroud_pre) << "," << oracle::fbits(shroud_post) << "]"
             << ",\"shroud_xfer\":\"" << shroud_hex << "\""
             << ",\"armor_flag\":" << (armor_flag ? 1 : 0)
             << ",\"armor_xfer\":\"" << armor_hex << "\""
             << ",\"cmd_str\":\"" << cmd_str << "\""
             << ",\"cost_bits\":[" << oracle::fbits(cost_pre) << "," << oracle::fbits(cost_post)
             << "," << oracle::fbits(cost_cap_a) << "," << oracle::fbits(cost_cap_b)
             << "," << oracle::fbits(cost_after_del_a) << "," << oracle::fbits(cost_after_del_b) << "]"
             << ",\"cost_pre_del\":" << (cost_upg_pre_del ? 1 : 0)
             << ",\"sci\":[" << (has_sci_a?1:0) << "," << (has_sci_b?1:0) << "]"
             << ",\"loco\":[" << loco_pre << "," << loco_post << "]"
             << ",\"hp_bits\":[" << oracle::fbits(hp_post) << "," << oracle::fbits(max_hp_post) << "]"
             << ",\"mc_set\":" << (mc_set ? 1 : 0)
             << ",\"ocl\":[" << cnt_before_ocl << "," << cnt_after_ocl << "]"
             << ",\"pass\":[" << (pass_pre?1:0) << "," << (pass_post?1:0) << "]"
             << ",\"pp\":[" << pp_pre_a << "," << pp_post_a << "," << pp_cap_a << "," << pp_cap_b << "," << pp_del_b << "]"
             << ",\"rad\":[" << rad_a << "," << dprad_a << "," << rad_cap_a << "," << rad_cap_b << "," << rad_del_b << "]"
             << ",\"repl\":[" << (repl_old_destroyed?1:0) << "," << sc_before_repl << "," << sc_after_repl << "]"
             << ",\"stat\":[" << (stat_stealth?1:0) << "," << (stat_immob?1:0) << "," << (stat_uc?1:0) << "]"
             << ",\"stl\":" << (stl_flag ? 1 : 0)
             << ",\"sub\":[" << (is_sub_true?1:0) << "," << (sub_upg_done?1:0) << "]"
             << ",\"sub_xfer\":\"" << sub_hex << "\""
             << ",\"sp\":[" << (sp_paused_pre?1:0) << "," << (sp_paused_post?1:0) << "," << sp_ready_post << "]"
             << ",\"wb\":" << (wb_flag ? 1 : 0)
             << ",\"ws\":" << (ws_flag ? 1 : 0)
             << "}"
             << "}";
    }

    std::ostringstream doc;
    doc << "{\n"
        << "  \"packet\": \"WP-5.2-B\",\n"
        << "  \"modules\": [\"UpgradeModule\",\"ActiveShroudUpgrade\",\"ArmorUpgrade\",\"CommandSetUpgrade\",\"CostModifierUpgrade\",\"ExperienceScalarUpgrade\",\"GrantScienceUpgrade\",\"LocomotorSetUpgrade\",\"MaxHealthUpgrade\",\"ModelConditionUpgrade\",\"ObjectCreationUpgrade\",\"PassengersFireUpgrade\",\"PowerPlantUpgrade\",\"RadarUpgrade\",\"ReplaceObjectUpgrade\",\"StatusBitsUpgrade\",\"StealthUpgrade\",\"SubObjectsUpgrade\",\"UnpauseSpecialPowerUpgrade\",\"WeaponBonusUpgrade\",\"WeaponSetUpgrade\"],\n"
        << "  \"seed_count\": 1000,\n"
        << "  \"source_sha256\": \"" << oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Upgrade/UpgradeModule.cpp") << "\",\n"
        << "  \"seed0_diagnostics\": " << s0_diag << ",\n"
        << "  \"vectors\": [\n" << vecs.str() << "\n  ]\n}\n";

    oracle::write_file_atomic(out_path, doc.str());
    return 0;
}
