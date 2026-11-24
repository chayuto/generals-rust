// WP-5.1-C CreateModule hierarchy (PKT-168, 171, 176, 182, 183, 187, 188, 192)
// C++ Golden Vector Generator (Zero-Stub Build against oracle_wp51_world.h).
//
// Canonical build & run:
//   python3 scripts/build_oracle.py "$(pwd)/tests/cpp_harness/gen_wp51_create.cpp" \
//     -o /tmp/gen_wp51_create --extra-flag="-I$(pwd)/tests/cpp_harness"
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy /tmp/gen_wp51_create docs/golden_vectors/WP51_Create.json

#include "oracle_wp51_world.h"

#include "Common/AcademyStats.h"
#include "Common/ResourceGatheringManager.h"
#include "GameClient/Drawable.h"
#include "GameLogic/ExperienceTracker.h"
#include "GameLogic/Module/CreateModule.h"
#include "GameLogic/Module/GrantUpgradeCreate.h"
#include "GameLogic/Module/LockWeaponCreate.h"
#include "GameLogic/Module/PreorderCreate.h"
#include "GameLogic/Module/SpecialPowerCreate.h"
#include "GameLogic/Module/SupplyCenterCreate.h"
#include "GameLogic/Module/SupplyWarehouseCreate.h"
#include "GameLogic/Module/VeterancyGainCreate.h"

#include <list>

struct PlayerPreorderTag { using type = Bool Player::*; };
template struct MemberAccess<PlayerPreorderTag, &Player::m_isPreorder>;
Bool Player::* access_member(PlayerPreorderTag);

struct PlayerUpgTag { using type = UpgradeMaskType Player::*; };
template struct MemberAccess<PlayerUpgTag, &Player::m_upgradesCompleted>;
UpgradeMaskType Player::* access_member(PlayerUpgTag);

struct PlayerSciTag { using type = ScienceVec Player::*; };
template struct MemberAccess<PlayerSciTag, &Player::m_sciences>;
ScienceVec Player::* access_member(PlayerSciTag);

struct RgmCentersTag { using type = std::list<ObjectID> ResourceGatheringManager::*; };
template struct MemberAccess<RgmCentersTag, &ResourceGatheringManager::m_supplyCenters>;
std::list<ObjectID> ResourceGatheringManager::* access_member(RgmCentersTag);

struct RgmWarehousesTag { using type = std::list<ObjectID> ResourceGatheringManager::*; };
template struct MemberAccess<RgmWarehousesTag, &ResourceGatheringManager::m_supplyWarehouses>;
std::list<ObjectID> ResourceGatheringManager::* access_member(RgmWarehousesTag);

static CreateModule* find_create_module(Object* obj) {
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m) {
        if (CreateModuleInterface* ci = (*m)->getCreate()) {
            return static_cast<CreateModule*>(*m);
        }
    }
    return nullptr;
}

int main(int argc, char** argv) {
    const char* out_path = (argc > 1) ? argv[1] : "docs/golden_vectors/WP51_Create.json";
    oracle_wp51::World w;

    // Define shared upgrades, sciences, and weapons for WP-5.1-C seeds.
    w.load_ini(
        "Upgrade UpgPlayerTest\n"
        "  Type = PLAYER\n"
        "  BuildTime = 1.0\n"
        "  BuildCost = 100\n"
        "End\n"
        "Upgrade UpgObjectTest\n"
        "  Type = OBJECT\n"
        "  BuildTime = 1.0\n"
        "  BuildCost = 50\n"
        "End\n"
        "Science SCI_WP51_A\n"
        "  SciencePurchasePointCost = 1\n"
        "  IsGrantable = Yes\n"
        "End\n"
        "Science SCI_WP51_B\n"
        "  SciencePurchasePointCost = 1\n"
        "  IsGrantable = Yes\n"
        "End\n"
        "Weapon CreateWpnPrimary\n"
        "  PrimaryDamage = 10\n"
        "  AttackRange = 100\n"
        "  DamageType = SMALL_ARMS\n"
        "  DeathType = NORMAL\n"
        "  WeaponSpeed = 999999\n"
        "  ClipSize = 1\n"
        "  DelayBetweenShots = 100\n"
        "End\n"
        "Weapon CreateWpnSecondary\n"
        "  PrimaryDamage = 20\n"
        "  AttackRange = 150\n"
        "  DamageType = EXPLOSION\n"
        "  DeathType = NORMAL\n"
        "  WeaponSpeed = 999999\n"
        "  ClipSize = 1\n"
        "  DelayBetweenShots = 100\n"
        "End\n"
        "Weapon CreateWpnTertiary\n"
        "  PrimaryDamage = 30\n"
        "  AttackRange = 200\n"
        "  DamageType = ARMOR_PIERCING\n"
        "  DeathType = NORMAL\n"
        "  WeaponSpeed = 999999\n"
        "  ClipSize = 1\n"
        "  DelayBetweenShots = 100\n"
        "End\n"
    );

    Player* p_a = ThePlayerList->getNthPlayer(0);
    const UpgradeTemplate* upg_player = TheUpgradeCenter->findUpgrade("UpgPlayerTest");
    const UpgradeTemplate* upg_object = TheUpgradeCenter->findUpgrade("UpgObjectTest");
    ScienceType sci_a = TheScienceStore->friend_lookupScience("SCI_WP51_A");
    ScienceType sci_b = TheScienceStore->friend_lookupScience("SCI_WP51_B");

    static const char* kVetNames[4] = { "REGULAR", "VETERAN", "ELITE", "HEROIC" };
    static const char* kSlotNames[3] = { "PRIMARY", "SECONDARY", "TERTIARY" };

    std::string s0_diag;
    std::ostringstream vecs;

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed ^ 0x51c01234u;
        if (rng == 0) rng = 0x13572468u;

        UnsignedInt frame0 = 10u + (oracle::splitmix32(rng) % 500u);
        w.set_frame(frame0);

        bool grant_player_upg = ((oracle::splitmix32(rng) & 1u) != 0);
        bool exempt_under_const = (seed == 0) ? true : ((oracle::splitmix32(rng) % 3u) != 0);
        bool start_under_const = (seed == 0) ? false : ((oracle::splitmix32(rng) & 1u) != 0);
        bool call_build_complete = (seed == 0) ? true : ((oracle::splitmix32(rng) % 5u) != 0);
        bool second_build_complete = (seed == 0) ? true : ((oracle::splitmix32(rng) & 1u) != 0);
        int lock_slot = static_cast<int>(oracle::splitmix32(rng) % 3u);
        bool is_preorder = (seed == 0) ? true : ((oracle::splitmix32(rng) & 1u) != 0);
        int vet_lvl = (seed == 0) ? 2 : static_cast<int>(1u + (oracle::splitmix32(rng) % 3u));
        bool require_sci = (seed == 0) ? true : ((oracle::splitmix32(rng) & 1u) != 0);
        bool grant_sci = (seed == 0) ? true : ((oracle::splitmix32(rng) & 1u) != 0);
        bool sci_b_toggle = (oracle::splitmix32(rng) & 1u) != 0;

        // Reset mutable player preorder/upgrades/science/rgm state deterministically per seed.
        p_a->*access_member(PlayerPreorderTag()) = is_preorder ? TRUE : FALSE;
        (p_a->*access_member(PlayerUpgTag())) = UpgradeMaskType();
        (p_a->*access_member(PlayerSciTag())).clear();
        if (ResourceGatheringManager* rgm = p_a->getResourceGatheringManager()) {
            (rgm->*access_member(RgmCentersTag())).clear();
            (rgm->*access_member(RgmWarehousesTag())).clear();
        }
        if (grant_sci) p_a->grantScience(sci_a);
        if (sci_b_toggle) p_a->grantScience(sci_b);

        std::string upg_name = grant_player_upg ? "UpgPlayerTest" : "UpgObjectTest";
        std::string sci_name = require_sci ? (sci_b_toggle ? "SCI_WP51_B" : "SCI_WP51_A") : "SCIENCE_NONE";
        // For deterministic Veterinarian Science test when grant_sci==false, pick a non-granted science if needed.
        if (require_sci && !grant_sci) sci_name = "SCI_WP51_B";
        if (require_sci && sci_b_toggle) sci_name = "SCI_WP51_A";

        std::string pfx = "C51_" + std::to_string(seed) + "_";
        std::string t_grant = pfx + "Grant";
        std::string t_lock = pfx + "Lock";
        std::string t_pre = pfx + "Pre";
        std::string t_sp = pfx + "SP";
        std::string t_sc = pfx + "SC";
        std::string t_sw = pfx + "SW";
        std::string t_vet = pfx + "Vet";

        std::ostringstream ini_ss;
        ini_ss << "Object " << t_grant << "\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = GrantUpgradeCreate ModuleTag_Create\n"
               << "    UpgradeToGrant = " << upg_name << "\n"
               << (exempt_under_const ? "    ExemptStatus = UNDER_CONSTRUCTION\n" : "")
               << "  End\nEnd\n"
               << "Object " << t_lock << "\n"
               << "  WeaponSet\n    Conditions = None\n"
               << "    Weapon = PRIMARY CreateWpnPrimary\n"
               << "    Weapon = SECONDARY CreateWpnSecondary\n"
               << "    Weapon = TERTIARY CreateWpnTertiary\n  End\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = LockWeaponCreate ModuleTag_Create\n"
               << "    SlotToLock = " << kSlotNames[lock_slot] << "\n  End\nEnd\n"
               << "Object " << t_pre << "\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = PreorderCreate ModuleTag_Create\n  End\nEnd\n"
               << "Object " << t_sp << "\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = SpecialPowerCreate ModuleTag_Create\n  End\nEnd\n"
               << "Object " << t_sc << "\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = SupplyCenterCreate ModuleTag_Create\n  End\nEnd\n"
               << "Object " << t_sw << "\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = SupplyWarehouseCreate ModuleTag_Create\n  End\nEnd\n"
               << "Object " << t_vet << "\n"
               << "  IsTrainable = Yes\n"
               << "  ExperienceRequired = 0 100 200 400\n"
               << "  ExperienceValue = 10 20 30 40\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
               << "  Behavior = VeterancyGainCreate ModuleTag_Create\n"
               << "    StartingLevel = " << kVetNames[vet_lvl] << "\n";
        if (require_sci && (sci_name == "SCI_WP51_A" || sci_name == "SCI_WP51_B")) {
            ini_ss << "    ScienceRequired = " << sci_name << "\n";
        }
        ini_ss << "  End\nEnd\n";

        w.load_ini(ini_ss.str(), INI_LOAD_OVERWRITE);

        // 1) GrantUpgradeCreate
        Object* o_grant = nullptr;
        {
            const ThingTemplate* tt = TheThingFactory->findTemplate(t_grant.c_str());
            ObjectStatusMaskType st;
            if (start_under_const) st.set(OBJECT_STATUS_UNDER_CONSTRUCTION);
            o_grant = TheThingFactory->newObject(tt, w.team_a, st);
        }
        CreateModule* cm_grant = find_create_module(o_grant);
        bool g_obj_pre = o_grant->hasUpgrade(upg_object);
        bool g_ply_pre = p_a->hasUpgradeComplete(upg_player);
        bool g_need_pre = cm_grant->shouldDoOnBuildComplete();
        if (call_build_complete) cm_grant->onBuildComplete();
        bool g_obj_post1 = o_grant->hasUpgrade(upg_object);
        bool g_ply_post1 = p_a->hasUpgradeComplete(upg_player);
        bool g_need_post1 = cm_grant->shouldDoOnBuildComplete();
        if (second_build_complete) cm_grant->onBuildComplete();
        uint32_t grant_xfer_crc = 0;
        std::string grant_hex = oracle_wp51::World::snapshot_save_hex(cm_grant, &grant_xfer_crc);
        uint32_t grant_crc = oracle_wp51::World::snapshot_crc(cm_grant);

        // 2) LockWeaponCreate
        Object* o_lock = w.spawn_object(t_lock.c_str(), w.team_a);
        CreateModule* cm_lock = find_create_module(o_lock);
        if (call_build_complete) cm_lock->onBuildComplete();
        Int locked_slot = o_lock->getCurrentWeapon() ? static_cast<Int>(o_lock->getCurrentWeapon()->getWeaponSlot()) : -1;
        bool lock_need_post = cm_lock->shouldDoOnBuildComplete();
        uint32_t lock_xfer_crc = 0;
        std::string lock_hex = oracle_wp51::World::snapshot_save_hex(cm_lock, &lock_xfer_crc);

        // 3) PreorderCreate
        Object* o_pre = w.spawn_object(t_pre.c_str(), w.team_a);
        CreateModule* cm_pre = find_create_module(o_pre);
        if (call_build_complete) cm_pre->onBuildComplete();
        bool pre_flag = o_pre->getDrawable() ? o_pre->getDrawable()->getModelConditionFlags().test(MODELCONDITION_PREORDER) : false;
        bool pre_need_post = cm_pre->shouldDoOnBuildComplete();
        uint32_t pre_xfer_crc = 0;
        std::string pre_hex = oracle_wp51::World::snapshot_save_hex(cm_pre, &pre_xfer_crc);

        // 4) SpecialPowerCreate
        Object* o_sp = w.spawn_object(t_sp.c_str(), w.team_a);
        CreateModule* cm_sp = find_create_module(o_sp);
        if (call_build_complete) cm_sp->onBuildComplete();
        bool sp_need_post = cm_sp->shouldDoOnBuildComplete();
        uint32_t sp_xfer_crc = 0;
        std::string sp_hex = oracle_wp51::World::snapshot_save_hex(cm_sp, &sp_xfer_crc);

        // 5) SupplyCenterCreate & 6) SupplyWarehouseCreate
        Object* o_sc = w.spawn_object(t_sc.c_str(), w.team_a);
        CreateModule* cm_sc = find_create_module(o_sc);
        size_t sc_cnt_pre = (p_a->getResourceGatheringManager()->*access_member(RgmCentersTag())).size();
        if (call_build_complete) cm_sc->onBuildComplete();
        size_t sc_cnt_post = (p_a->getResourceGatheringManager()->*access_member(RgmCentersTag())).size();
        Object* o_sw = w.spawn_object(t_sw.c_str(), w.team_a);
        CreateModule* cm_sw = find_create_module(o_sw);
        size_t sw_cnt_pre = (p_a->getResourceGatheringManager()->*access_member(RgmWarehousesTag())).size();
        uint32_t sc_xfer_crc = 0;
        std::string sc_hex = oracle_wp51::World::snapshot_save_hex(cm_sc, &sc_xfer_crc);
        uint32_t sw_xfer_crc = 0;
        std::string sw_hex = oracle_wp51::World::snapshot_save_hex(cm_sw, &sw_xfer_crc);

        // 7) VeterancyGainCreate
        Object* o_vet = w.spawn_object(t_vet.c_str(), w.team_a);
        CreateModule* cm_vet = find_create_module(o_vet);
        Int vet_out = static_cast<Int>(o_vet->getExperienceTracker()->getVeterancyLevel());
        Real hp_out = o_vet->getBodyModule()->getHealth();
        Real max_hp_out = o_vet->getBodyModule()->getMaxHealth();
        bool vet_need_pre = cm_vet->shouldDoOnBuildComplete();
        if (call_build_complete) cm_vet->onBuildComplete();
        bool vet_need_post = cm_vet->shouldDoOnBuildComplete();
        uint32_t vet_xfer_crc = 0;
        std::string vet_hex = oracle_wp51::World::snapshot_save_hex(cm_vet, &vet_xfer_crc);
        NameKeyType vet_sci_key = static_cast<NameKeyType>((sci_name == "SCI_WP51_A") ? sci_a : ((sci_name == "SCI_WP51_B") ? sci_b : NAMEKEY_INVALID));

        // Clean objects cleanly to keep GameLogic arena compact across 1000 seeds.
        TheGameLogic->destroyObject(o_grant);
        TheGameLogic->destroyObject(o_lock);
        TheGameLogic->destroyObject(o_pre);
        TheGameLogic->destroyObject(o_sp);
        TheGameLogic->destroyObject(o_sc);
        TheGameLogic->destroyObject(o_sw);
        TheGameLogic->destroyObject(o_vet);
        TheGameLogic->processDestroyList();

        if (seed == 0) {
            std::ostringstream d;
            d << "{"
              << "\"grant\":{\"obj_pre\":" << (g_obj_pre?1:0) << ",\"obj_post\":" << (g_obj_post1?1:0)
              << ",\"ply_pre\":" << (g_ply_pre?1:0) << ",\"ply_post\":" << (g_ply_post1?1:0)
              << ",\"need_pre\":" << (g_need_pre?1:0) << ",\"need_post\":" << (g_need_post1?1:0)
              << ",\"xfer\":\"" << grant_hex << "\"},"
              << "\"lock\":{\"slot\":" << locked_slot << ",\"need_post\":" << (lock_need_post?1:0) << ",\"xfer\":\"" << lock_hex << "\"},"
              << "\"preorder\":{\"flag\":" << (pre_flag?1:0) << ",\"need_post\":" << (pre_need_post?1:0) << ",\"xfer\":\"" << pre_hex << "\"},"
              << "\"special_power\":{\"need_post\":" << (sp_need_post?1:0) << ",\"xfer\":\"" << sp_hex << "\"},"
              << "\"supply\":{\"sc_pre\":" << sc_cnt_pre << ",\"sc_post\":" << sc_cnt_post << ",\"sw_pre\":" << sw_cnt_pre << "},"
              << "\"veterancy\":{\"lvl\":" << vet_out << ",\"hp_bits\":" << oracle::fbits(hp_out) << ",\"max_hp_bits\":" << oracle::fbits(max_hp_out)
              << ",\"sci_key\":" << static_cast<int32_t>(vet_sci_key) << ",\"xfer\":\"" << vet_hex << "\"}"
              << "}";
            s0_diag = d.str();
        }

        if (seed > 0) vecs << ",\n";
        vecs << "    {"
             << "\"seed\":" << seed
             << ",\"in\":{\"frame\":" << frame0
             << ",\"upg\":\"" << upg_name << "\""
             << ",\"exempt_uc\":" << (exempt_under_const ? 1 : 0)
             << ",\"start_uc\":" << (start_under_const ? 1 : 0)
             << ",\"build\":" << (call_build_complete ? 1 : 0)
             << ",\"build2\":" << (second_build_complete ? 1 : 0)
             << ",\"lock_slot\":" << lock_slot
             << ",\"preorder\":" << (is_preorder ? 1 : 0)
             << ",\"vet_lvl\":" << vet_lvl
             << ",\"sci_name\":\"" << ((sci_name == "SCI_WP51_A" || sci_name == "SCI_WP51_B") ? sci_name : "") << "\""
             << ",\"sci_a_known\":" << (p_a->hasScience(sci_a) ? 1 : 0)
             << ",\"sci_b_known\":" << (p_a->hasScience(sci_b) ? 1 : 0)
             << "}"
             << ",\"out\":{\"g_obj\":[" << (g_obj_pre?1:0) << "," << (g_obj_post1?1:0) << "]"
             << ",\"g_ply\":[" << (g_ply_pre?1:0) << "," << (g_ply_post1?1:0) << "]"
             << ",\"g_need\":[" << (g_need_pre?1:0) << "," << (g_need_post1?1:0) << "]"
             << ",\"grant_crc\":" << grant_crc
             << ",\"grant_xfer\":\"" << grant_hex << "\""
             << ",\"lock_slot_out\":" << locked_slot
             << ",\"lock_need\":" << (lock_need_post ? 1 : 0)
             << ",\"lock_xfer\":\"" << lock_hex << "\""
             << ",\"pre_flag\":" << (pre_flag ? 1 : 0)
             << ",\"pre_need\":" << (pre_need_post ? 1 : 0)
             << ",\"pre_xfer\":\"" << pre_hex << "\""
             << ",\"sp_need\":" << (sp_need_post ? 1 : 0)
             << ",\"sp_xfer\":\"" << sp_hex << "\""
             << ",\"sc\":[" << sc_cnt_pre << "," << sc_cnt_post << "]"
             << ",\"sw\":" << sw_cnt_pre
             << ",\"sc_xfer\":\"" << sc_hex << "\""
             << ",\"sw_xfer\":\"" << sw_hex << "\""
             << ",\"vet_lvl\":" << vet_out
             << ",\"vet_hp\":" << oracle::fbits(hp_out)
             << ",\"vet_max_hp\":" << oracle::fbits(max_hp_out)
             << ",\"vet_need\":[" << (vet_need_pre?1:0) << "," << (vet_need_post?1:0) << "]"
             << ",\"vet_sci_key\":" << static_cast<int32_t>(vet_sci_key)
             << ",\"vet_xfer\":\"" << vet_hex << "\""
             << ",\"xfer_crcs\":[" << grant_xfer_crc << "," << lock_xfer_crc << "," << pre_xfer_crc
             << "," << sp_xfer_crc << "," << sc_xfer_crc << "," << sw_xfer_crc << "," << vet_xfer_crc << "]"
             << "}"
             << "}";
    }

    std::ostringstream doc;
    doc << "{\n"
        << "  \"packet\": \"WP-5.1-C\",\n"
        << "  \"modules\": [\"CreateModule\",\"GrantUpgradeCreate\",\"LockWeaponCreate\",\"PreorderCreate\",\"SpecialPowerCreate\",\"SupplyCenterCreate\",\"SupplyWarehouseCreate\",\"VeterancyGainCreate\"],\n"
        << "  \"seed_count\": 1000,\n"
        << "  \"source_hashes\": {\n"
        << "    \"CreateModule.cpp\": \"" << oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Create/CreateModule.cpp") << "\",\n"
        << "    \"GrantUpgradeCreate.cpp\": \"" << oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Create/GrantUpgradeCreate.cpp") << "\",\n"
        << "    \"VeterancyGainCreate.cpp\": \"" << oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Create/VeterancyGainCreate.cpp") << "\"\n"
        << "  },\n"
        << "  \"seed0_diagnostics\": " << s0_diag << ",\n"
        << "  \"vectors\": [\n" << vecs.str() << "\n  ]\n}\n";

    oracle::write_file_atomic(out_path, doc.str());
    return 0;
}
