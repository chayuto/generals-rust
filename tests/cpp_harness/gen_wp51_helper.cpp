// WP-5.1-A ObjectHelper hierarchy (PKT-177..181, 184, 186, 189)
// C++ Golden Vector Generator (Zero-Stub Build against oracle_wp51_world.h).

#include "oracle_wp51_world.h"

#include "GameClient/Drawable.h"
#include "GameLogic/AI.h"
#include "GameLogic/Damage.h"
#include "GameLogic/Module/ActiveBody.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/ObjectDefectionHelper.h"
#include "GameLogic/Module/ObjectHelper.h"
#include "GameLogic/Module/ObjectRepulsorHelper.h"
#include "GameLogic/Module/ObjectSMCHelper.h"
#include "GameLogic/Module/ObjectWeaponStatusHelper.h"
#include "GameLogic/Module/StatusDamageHelper.h"
#include "GameLogic/Module/SubdualDamageHelper.h"
#include "GameLogic/Module/TempWeaponBonusHelper.h"

struct ObjSmcTag { using type = ObjectSMCHelper* Object::*; };
template struct MemberAccess<ObjSmcTag, &Object::m_smcHelper>;
ObjectSMCHelper* Object::* access_member(ObjSmcTag);

struct ObjSdTag { using type = StatusDamageHelper* Object::*; };
template struct MemberAccess<ObjSdTag, &Object::m_statusDamageHelper>;
StatusDamageHelper* Object::* access_member(ObjSdTag);

struct ObjSubTag { using type = SubdualDamageHelper* Object::*; };
template struct MemberAccess<ObjSubTag, &Object::m_subdualDamageHelper>;
SubdualDamageHelper* Object::* access_member(ObjSubTag);

struct ObjRepTag { using type = ObjectRepulsorHelper* Object::*; };
template struct MemberAccess<ObjRepTag, &Object::m_repulsorHelper>;
ObjectRepulsorHelper* Object::* access_member(ObjRepTag);

struct ObjDefTag { using type = ObjectDefectionHelper* Object::*; };
template struct MemberAccess<ObjDefTag, &Object::m_defectionHelper>;
ObjectDefectionHelper* Object::* access_member(ObjDefTag);

struct ObjWsTag { using type = ObjectWeaponStatusHelper* Object::*; };
template struct MemberAccess<ObjWsTag, &Object::m_wsHelper>;
ObjectWeaponStatusHelper* Object::* access_member(ObjWsTag);

struct ObjTwbTag { using type = TempWeaponBonusHelper* Object::*; };
template struct MemberAccess<ObjTwbTag, &Object::m_tempWeaponBonusHelper>;
TempWeaponBonusHelper* Object::* access_member(ObjTwbTag);

struct ObjArmorFlagsTag { using type = UnsignedInt Object::*; };

int main(int argc, char** argv) {
    const char* out_path = (argc > 1) ? argv[1] : "docs/golden_vectors/WP51_Helper.json";
    oracle_wp51::World w;

    w.load_ini(
        "Weapon HelperWpn\n"
        "  PrimaryDamage = 15\n"
        "  AttackRange = 100\n"
        "  DamageType = SMALL_ARMS\n"
        "  DeathType = NORMAL\n"
        "  WeaponSpeed = 999999\n"
        "  ClipSize = 2\n"
        "  ClipReloadTime = 1000\n"
        "  DelayBetweenShots = 100\n"
        "End\n"
    );

    static const char* kStatusNames[4] = { "AFLAME", "BURNED", "FAERIE_FIRE", "IMMOBILE" };
    static const ObjectStatusTypes kStatusEnums[4] = {
        OBJECT_STATUS_AFLAME, OBJECT_STATUS_BURNED, OBJECT_STATUS_FAERIE_FIRE, OBJECT_STATUS_IMMOBILE
    };
    static const WeaponBonusConditionType kBonusEnums[4] = {
        WEAPONBONUSCONDITION_FRENZY_ONE,
        WEAPONBONUSCONDITION_TARGET_FAERIE_FIRE,
        WEAPONBONUSCONDITION_VETERAN,
        WEAPONBONUSCONDITION_FANATICISM
    };

    std::string s0_diag;
    std::ostringstream vecs;

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed ^ 0x51a0cafeu;
        if (rng == 0) rng = 0x12345678u;

        UnsignedInt frame0 = 20u + (oracle::splitmix32(rng) % 400u);
        w.set_frame(frame0);

        bool enable_repulsors = (seed == 0) ? true : ((oracle::splitmix32(rng) & 1u) != 0);
        bool inactive_body = (seed == 0) ? false : ((oracle::splitmix32(rng) % 5u) == 0);
        bool can_repulse = (seed == 0) ? true : ((oracle::splitmix32(rng) & 1u) != 0);
        bool shrub = (seed == 0) ? false : ((oracle::splitmix32(rng) % 4u) == 0);
        bool has_wpn = (seed == 0) ? true : ((oracle::splitmix32(rng) % 3u) != 0);

        UnsignedInt smc_dur = (seed % 9u == 0) ? 0u : (1u + (oracle::splitmix32(rng) % 40u));
        int status_idx = static_cast<int>(oracle::splitmix32(rng) % 4u);
        Real status_dur = 1.0f + static_cast<Real>(oracle::splitmix32(rng) % 30u) * 0.5f;
        int bonus_idx = static_cast<int>(oracle::splitmix32(rng) % 4u);
        UnsignedInt bonus_dur = 1u + (oracle::splitmix32(rng) % 50u);
        UnsignedInt def_dur = 2u + (oracle::splitmix32(rng) % 60u);
        bool def_fx = (seed == 0) ? true : ((oracle::splitmix32(rng) & 1u) != 0);
        bool def_blow_cover = (seed % 7u == 0);
        UnsignedInt def_steps = 1u + (oracle::splitmix32(rng) % 8u);

        Real sub_cap = 50.0f + static_cast<Real>(oracle::splitmix32(rng) % 100u);
        Int sub_rate_ms = 100 * static_cast<Int>(1u + (oracle::splitmix32(rng) % 5u));
        Real sub_heal_amt = 2.0f + static_cast<Real>(oracle::splitmix32(rng) % 10u);
        Real sub_dmg_amt = 5.0f + static_cast<Real>(oracle::splitmix32(rng) % 40u);
        UnsignedInt sub_steps = 1u + (oracle::splitmix32(rng) % 5u);

        const_cast<TAiData*>(TheAI->getAiData())->m_enableRepulsors = enable_repulsors ? TRUE : FALSE;

        std::string obj_name = "H51_Unit_" + std::to_string(seed);
        std::ostringstream ini_ss;
        ini_ss << "Object " << obj_name << "\n"
               << "  KindOf = SELECTABLE";
        if (can_repulse) ini_ss << " CAN_BE_REPULSED";
        if (shrub) ini_ss << " SHRUBBERY";
        if (has_wpn) ini_ss << " CAN_ATTACK";
        ini_ss << "\n";
        if (has_wpn) {
            ini_ss << "  WeaponSet\n    Conditions = None\n    Weapon = PRIMARY HelperWpn\n  End\n";
        }
        if (inactive_body) {
            ini_ss << "  Body = InactiveBody ModuleTag_Body\n  End\n";
        } else {
            ini_ss << "  Body = ActiveBody ModuleTag_Body\n"
                   << "    MaxHealth = 200\n    InitialHealth = 200\n"
                   << "    SubdualDamageCap = " << sub_cap << "\n"
                   << "    SubdualDamageHealRate = " << sub_rate_ms << "\n"
                   << "    SubdualDamageHealAmount = " << sub_heal_amt << "\n"
                   << "  End\n";
        }
        ini_ss << "End\n";
        w.load_ini(ini_ss.str(), INI_LOAD_OVERWRITE);

        Object* obj = w.spawn_object(obj_name.c_str(), w.team_a, {0.0f, 0.0f, 0.0f}, 0.0f);
        ObjectSMCHelper* h_smc = obj->*access_member(ObjSmcTag());
        StatusDamageHelper* h_sd = obj->*access_member(ObjSdTag());
        SubdualDamageHelper* h_sub = obj->*access_member(ObjSubTag());
        ObjectRepulsorHelper* h_rep = obj->*access_member(ObjRepTag());
        ObjectDefectionHelper* h_def = obj->*access_member(ObjDefTag());
        ObjectWeaponStatusHelper* h_ws = obj->*access_member(ObjWsTag());
        TempWeaponBonusHelper* h_twb = obj->*access_member(ObjTwbTag());

        UnsignedInt sub_rate_frames = obj->getBodyModule()->getSubdualDamageHealRate();
        Real sub_heal_out = obj->getBodyModule()->getSubdualDamageHealAmount();

        uint32_t pri_init[7] = {
            h_smc ? h_smc->friend_getPriority() : 0u,
            h_sd ? h_sd->friend_getPriority() : 0u,
            h_sub ? h_sub->friend_getPriority() : 0u,
            h_rep ? h_rep->friend_getPriority() : 0u,
            h_def ? h_def->friend_getPriority() : 0u,
            h_ws ? h_ws->friend_getPriority() : 0u,
            h_twb ? h_twb->friend_getPriority() : 0u
        };

        std::string smc_x0 = oracle_wp51::World::snapshot_save_hex(h_smc);
        std::string sd_x0 = h_sd ? oracle_wp51::World::snapshot_save_hex(h_sd) : "";
        std::string sub_x0 = h_sub ? oracle_wp51::World::snapshot_save_hex(h_sub) : "";
        std::string rep_x0 = h_rep ? oracle_wp51::World::snapshot_save_hex(h_rep) : "";
        std::string def_x0 = h_def ? oracle_wp51::World::snapshot_save_hex(h_def) : "";
        std::string ws_x0 = h_ws ? oracle_wp51::World::snapshot_save_hex(h_ws) : "";
        std::string twb_x0 = h_twb ? oracle_wp51::World::snapshot_save_hex(h_twb) : "";

        // 1) ObjectSMCHelper
        obj->setSpecialModelConditionState(MODELCONDITION_SPECIAL_CHEERING, smc_dur);
        uint32_t smc_pri1 = h_smc->friend_getPriority();
        std::string smc_x1 = oracle_wp51::World::snapshot_save_hex(h_smc);
        UnsignedInt smc_ret = static_cast<UnsignedInt>(h_smc->update());
        bool smc_flag_after = obj->getDrawable()
            ? (obj->getDrawable()->getModelConditionFlags().test(MODELCONDITION_SPECIAL_CHEERING) != FALSE)
            : false;

        // 2) StatusDamageHelper
        uint32_t sd_pri1 = 0, sd_ret = 0;
        std::string sd_x1, sd_x2;
        bool sd_set_flag = false, sd_clr_flag = false;
        if (h_sd) {
            obj->doStatusDamage(kStatusEnums[status_idx], status_dur);
            sd_set_flag = (obj->getStatusBits().test(kStatusEnums[status_idx]) != FALSE);
            sd_pri1 = h_sd->friend_getPriority();
            sd_x1 = oracle_wp51::World::snapshot_save_hex(h_sd);
            w.set_frame(frame0 + 1000u);
            sd_ret = static_cast<UnsignedInt>(h_sd->update());
            sd_clr_flag = (obj->getStatusBits().test(kStatusEnums[status_idx]) != FALSE);
            sd_x2 = oracle_wp51::World::snapshot_save_hex(h_sd);
            w.set_frame(frame0);
        }

        // 3) ObjectRepulsorHelper
        uint32_t rep_pri1 = 0, rep_ret = 0;
        bool rep_set_flag = false, rep_clr_flag = false;
        if (h_rep) {
            obj->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_REPULSOR));
            rep_set_flag = (obj->getStatusBits().test(OBJECT_STATUS_REPULSOR) != FALSE);
            rep_pri1 = h_rep->friend_getPriority();
            rep_ret = static_cast<UnsignedInt>(h_rep->update());
            rep_clr_flag = (obj->getStatusBits().test(OBJECT_STATUS_REPULSOR) != FALSE);
        }

        // 4) ObjectDefectionHelper
        uint32_t def_pri1 = 0, def_ret = 0;
        bool def_pre = false, def_post = false;
        std::string def_x1, def_x2;
        if (h_def) {
            obj->defect(w.team_b, def_dur);
            def_pre = (obj->getIsUndetectedDefector() != FALSE);
            def_pri1 = h_def->friend_getPriority();
            def_x1 = oracle_wp51::World::snapshot_save_hex(h_def);
            for (UnsignedInt k = 0; k < def_steps; ++k) {
                w.set_frame(frame0 + 1u + k);
                if (def_blow_cover && k == 1u) {
                    obj->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_IS_FIRING_WEAPON));
                }
                def_ret = static_cast<UnsignedInt>(h_def->update());
            }
            def_post = (obj->getIsUndetectedDefector() != FALSE);
            def_x2 = oracle_wp51::World::snapshot_save_hex(h_def);
            w.set_frame(frame0);
        }

        // 5) TempWeaponBonusHelper
        uint32_t twb_pri1 = 0, twb_ret = 0;
        UnsignedInt twb_mask1 = 0, twb_mask2 = 0;
        std::string twb_x1, twb_x2;
        if (h_twb) {
            obj->doTempWeaponBonus(kBonusEnums[bonus_idx], bonus_dur);
            twb_pri1 = h_twb->friend_getPriority();
            twb_x1 = oracle_wp51::World::snapshot_save_hex(h_twb);
            twb_mask1 = obj->getWeaponBonusCondition();
            w.set_frame(frame0 + bonus_dur + 1u);
            twb_ret = static_cast<UnsignedInt>(h_twb->update());
            twb_mask2 = obj->getWeaponBonusCondition();
            twb_x2 = oracle_wp51::World::snapshot_save_hex(h_twb);
            w.set_frame(frame0);
        }

        // 6) SubdualDamageHelper
        uint32_t sub_pri1 = 0, sub_ret = 0;
        Real sub_after = 0.0f;
        std::string sub_x1, sub_x2;
        if (h_sub) {
            DamageInfo d;
            d.in.m_damageType = DAMAGE_SUBDUAL_UNRESISTABLE;
            d.in.m_amount = sub_dmg_amt;
            obj->attemptDamage(&d);
            sub_pri1 = h_sub->friend_getPriority();
            sub_x1 = oracle_wp51::World::snapshot_save_hex(h_sub);
            for (UnsignedInt k = 0; k < sub_steps; ++k) {
                w.set_frame(frame0 + 1u + k);
                sub_ret = static_cast<UnsignedInt>(h_sub->update());
            }
            sub_after = obj->getBodyModule()->getCurrentSubdualDamageAmount();
            sub_x2 = oracle_wp51::World::snapshot_save_hex(h_sub);
            w.set_frame(frame0);
        }

        // 7) ObjectWeaponStatusHelper
        uint32_t ws_ret = 0;
        Int ws_phase = -1;
        if (h_ws) {
            ws_phase = static_cast<Int>(h_ws->friend_getNextCallPhase());
            ws_ret = static_cast<UnsignedInt>(h_ws->update());
        }

        TheGameLogic->destroyObject(obj);
        TheGameLogic->processDestroyList();

        if (seed == 0) {
            std::ostringstream d;
            d << "{"
              << "\"present\":[" << (h_smc?1:0) << "," << (h_sd?1:0) << "," << (h_sub?1:0)
              << "," << (h_rep?1:0) << "," << (h_def?1:0) << "," << (h_ws?1:0) << "," << (h_twb?1:0) << "]"
              << ",\"pri_init\":[" << pri_init[0] << "," << pri_init[1] << "," << pri_init[2] << ","
              << pri_init[3] << "," << pri_init[4] << "," << pri_init[5] << "," << pri_init[6] << "]"
              << ",\"smc_x0\":\"" << smc_x0 << "\",\"sd_x0\":\"" << sd_x0 << "\",\"def_x2\":\"" << def_x2 << "\""
              << ",\"sub_rate_frames\":" << sub_rate_frames
              << "}";
            s0_diag = d.str();
        }

        if (seed > 0) vecs << ",\n";
        vecs << "    {"
             << "\"seed\":" << seed
             << ",\"in\":{\"frame\":" << frame0
             << ",\"repulsors\":" << (enable_repulsors ? 1 : 0)
             << ",\"inactive_body\":" << (inactive_body ? 1 : 0)
             << ",\"can_repulse\":" << (can_repulse ? 1 : 0)
             << ",\"shrub\":" << (shrub ? 1 : 0)
             << ",\"has_wpn\":" << (has_wpn ? 1 : 0)
             << ",\"smc_dur\":" << smc_dur
             << ",\"status\":\"" << kStatusNames[status_idx] << "\""
             << ",\"status_enum\":" << static_cast<Int>(kStatusEnums[status_idx])
             << ",\"status_dur\":" << oracle::fbits(status_dur)
             << ",\"bonus_enum\":" << static_cast<Int>(kBonusEnums[bonus_idx])
             << ",\"bonus_dur\":" << bonus_dur
             << ",\"def_dur\":" << def_dur
             << ",\"def_fx\":" << (def_fx ? 1 : 0)
             << ",\"def_blow\":" << (def_blow_cover ? 1 : 0)
             << ",\"def_steps\":" << def_steps
             << ",\"sub_cap\":" << oracle::fbits(sub_cap)
             << ",\"sub_rate_ms\":" << sub_rate_ms
             << ",\"sub_rate_f\":" << sub_rate_frames
             << ",\"sub_heal\":" << oracle::fbits(sub_heal_out)
             << ",\"sub_dmg\":" << oracle::fbits(sub_dmg_amt)
             << ",\"sub_steps\":" << sub_steps
             << "}"
             << ",\"out\":{\"present\":[" << (h_smc?1:0) << "," << (h_sd?1:0) << "," << (h_sub?1:0)
             << "," << (h_rep?1:0) << "," << (h_def?1:0) << "," << (h_ws?1:0) << "," << (h_twb?1:0) << "]"
             << ",\"pri_init\":[" << pri_init[0] << "," << pri_init[1] << "," << pri_init[2] << ","
             << pri_init[3] << "," << pri_init[4] << "," << pri_init[5] << "," << pri_init[6] << "]"
             << ",\"x0\":[\"" << smc_x0 << "\",\"" << sd_x0 << "\",\"" << sub_x0 << "\",\""
             << rep_x0 << "\",\"" << def_x0 << "\",\"" << ws_x0 << "\",\"" << twb_x0 << "\"]"
             << ",\"smc\":[" << smc_pri1 << "," << smc_ret << "," << (smc_flag_after?1:0) << "]"
             << ",\"smc_x1\":\"" << smc_x1 << "\""
             << ",\"sd\":[" << sd_pri1 << "," << sd_ret << "," << (sd_set_flag?1:0) << "," << (sd_clr_flag?1:0) << "]"
             << ",\"sd_x1\":\"" << sd_x1 << "\",\"sd_x2\":\"" << sd_x2 << "\""
             << ",\"rep\":[" << rep_pri1 << "," << rep_ret << "," << (rep_set_flag?1:0) << "," << (rep_clr_flag?1:0) << "]"
             << ",\"def\":[" << def_pri1 << "," << def_ret << "," << (def_pre?1:0) << "," << (def_post?1:0) << "]"
             << ",\"def_x1\":\"" << def_x1 << "\",\"def_x2\":\"" << def_x2 << "\""
             << ",\"twb\":[" << twb_pri1 << "," << twb_ret << "," << twb_mask1 << "," << twb_mask2 << "]"
             << ",\"twb_x1\":\"" << twb_x1 << "\",\"twb_x2\":\"" << twb_x2 << "\""
             << ",\"sub\":[" << sub_pri1 << "," << sub_ret << "," << oracle::fbits(sub_after) << "]"
             << ",\"sub_x1\":\"" << sub_x1 << "\",\"sub_x2\":\"" << sub_x2 << "\""
             << ",\"ws\":[" << ws_phase << "," << ws_ret << "]"
             << "}"
             << "}";
    }

    std::ostringstream doc;
    doc << "{\n"
        << "  \"packet\": \"WP-5.1-A\",\n"
        << "  \"modules\": [\"ObjectHelper\",\"ObjectDefectionHelper\",\"ObjectRepulsorHelper\",\"ObjectSMCHelper\",\"ObjectWeaponStatusHelper\",\"StatusDamageHelper\",\"SubdualDamageHelper\",\"TempWeaponBonusHelper\"],\n"
        << "  \"seed_count\": 1000,\n"
        << "  \"source_hashes\": {\n"
        << "    \"ObjectHelper.cpp\": \"" << oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Helper/ObjectHelper.cpp") << "\",\n"
        << "    \"StatusDamageHelper.cpp\": \"" << oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Helper/StatusDamageHelper.cpp") << "\",\n"
        << "    \"ObjectDefectionHelper.cpp\": \"" << oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Helper/ObjectDefectionHelper.cpp") << "\"\n"
        << "  },\n"
        << "  \"seed0_diagnostics\": " << s0_diag << ",\n"
        << "  \"vectors\": [\n" << vecs.str() << "\n  ]\n}\n";

    oracle::write_file_atomic(out_path, doc.str());
    return 0;
}
