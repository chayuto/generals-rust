// WP-5.1-B BodyModule hierarchy (PKT-165, 166, 172, 173, 174, 175, 185, 191)
// C++ Golden Vector Generator (Zero-Stub Build against oracle_wp51_world.h).
//
// Canonical 3-mode build & run:
//   python3 scripts/build_oracle.py "$(pwd)/tests/cpp_harness/gen_wp51_body.cpp" \
//     -o /tmp/gen_wp51_body_o0 --opt O0 --extra-flag="-I$(pwd)/tests/cpp_harness"
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy /tmp/gen_wp51_body_o0 docs/golden_vectors/WP51_Body.json

#include "oracle_wp51_world.h"

#include "Common/RandomValue.h"
#include "GameLogic/Armor.h"
#include "GameLogic/Damage.h"
#include "GameLogic/ExperienceTracker.h"
#include "GameLogic/Module/ActiveBody.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/HighlanderBody.h"
#include "GameLogic/Module/HiveStructureBody.h"
#include "GameLogic/Module/ImmortalBody.h"
#include "GameLogic/Module/InactiveBody.h"
#include "GameLogic/Module/SlowDeathBehavior.h"
#include "GameLogic/Module/StructureBody.h"
#include "GameLogic/Module/UndeadBody.h"

static void run_armor_and_dfx_setup(oracle_wp51::World& w) {
    w.load_ini(
        "FXList FX_BodyHit\n"
        "End\n"
        "DamageFX BodyHitFX\n"
        "  ThrottleTime = Default 200\n"
        "  AmountForMajorFX = Default 50\n"
        "  MajorFX = Default FX_BodyHit\n"
        "  MinorFX = Default FX_BodyHit\n"
        "End\n"
        "Armor BodyBaseArmor\n"
        "  Armor = Default 100%\n"
        "  Armor = SMALL_ARMS 50%\n"
        "  Armor = EXPLOSION 80%\n"
        "  Armor = FLAME 125%\n"
        "  Armor = SUBDUAL_MISSILE 100%\n"
        "End\n"
        "Armor BodyVetArmor\n"
        "  Armor = Default 50%\n"
        "  Armor = SMALL_ARMS 25%\n"
        "End\n"
    );
}

int main(int argc, char** argv) {
    const char* out_path = (argc > 1) ? argv[1] : "docs/golden_vectors/WP51_Body.json";
    oracle_wp51::World w;
    run_armor_and_dfx_setup(w);

    static const DamageType kDmgTypes[5] = {
        DAMAGE_SMALL_ARMS, DAMAGE_EXPLOSION, DAMAGE_FLAME, DAMAGE_SNIPER, DAMAGE_UNRESISTABLE
    };
    static const char* kDmgNames[5] = {
        "SMALL_ARMS", "EXPLOSION", "FLAME", "SNIPER", "UNRESISTABLE"
    };
    static const MaxHealthChangeType kChangeTypes[4] = {
        SAME_CURRENTHEALTH, PRESERVE_RATIO, ADD_CURRENT_HEALTH_TOO, FULLY_HEAL
    };
    static const BodyDamageType kBodyStates[4] = {
        BODY_PRISTINE, BODY_DAMAGED, BODY_REALLYDAMAGED, BODY_RUBBLE
    };

    std::string s0_diag;
    std::ostringstream vecs;

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed ^ 0x51b0beefu;
        if (rng == 0) rng = 0x13572468u;

        UnsignedInt frame0 = 20u + (oracle::splitmix32(rng) % 400u);
        UnsignedInt rng_seed = 0x51b00001u + seed;
        InitRandom(rng_seed);
        w.set_frame(frame0);

        Real max_hp = static_cast<Real>(100 + static_cast<int>(oracle::splitmix32(rng) % 300u));
        Real init_hp = static_cast<Real>(50 + static_cast<int>(oracle::splitmix32(rng) % (static_cast<int>(max_hp) - 49)));
        Real sub_cap = static_cast<Real>(60 + static_cast<int>(oracle::splitmix32(rng) % 140u));
        Int sub_rate_ms = 100 * static_cast<Int>(1u + (oracle::splitmix32(rng) % 5u));
        Real sub_heal = static_cast<Real>(4 + static_cast<int>(oracle::splitmix32(rng) % 12u));
        static const Real kScalars[4] = { 0.5f, 0.75f, 1.0f, 1.25f };
        Real scalar1 = kScalars[oracle::splitmix32(rng) % 4u];
        Real dmg1 = static_cast<Real>(20 + static_cast<int>(oracle::splitmix32(rng) % 150u));
        int dt_idx = static_cast<int>(oracle::splitmix32(rng) % 5u);
        bool kill1 = (seed % 11u == 0);
        Real heal1 = static_cast<Real>(10 + static_cast<int>(oracle::splitmix32(rng) % 60u));
        int ct_idx = static_cast<int>(oracle::splitmix32(rng) % 4u);
        Real new_max_hp = static_cast<Real>(80 + static_cast<int>(oracle::splitmix32(rng) % 280u));
        Int initial_pct = 25 * static_cast<Int>(1u + (oracle::splitmix32(rng) % 4u));
        int ds_idx = static_cast<int>(oracle::splitmix32(rng) % 4u);
        Real hero_bonus = ((oracle::splitmix32(rng) & 1u) != 0) ? 1.2f : 1.0f;
        Real rubble_h = static_cast<Real>(2 + static_cast<int>(oracle::splitmix32(rng) % 8u));
        Real sec_max_hp = static_cast<Real>(50 + static_cast<int>(oracle::splitmix32(rng) % 150u));
        bool front_crushed = (seed & 1u) != 0;
        bool back_crushed = (seed & 2u) != 0;
        bool start_under_const = (seed % 5u == 0);

        TheWritableGlobalData->m_healthBonus[LEVEL_HEROIC] = hero_bonus;

        std::string t_atk = "B51_Atk_" + std::to_string(seed);
        std::string t_act = "B51_Act_" + std::to_string(seed);
        std::string t_high = "B51_High_" + std::to_string(seed);
        std::string t_imm = "B51_Imm_" + std::to_string(seed);
        std::string t_inac = "B51_Inac_" + std::to_string(seed);
        std::string t_str = "B51_Str_" + std::to_string(seed);
        std::string t_hive = "B51_Hive_" + std::to_string(seed);
        std::string t_und = "B51_Und_" + std::to_string(seed);

        std::ostringstream ini_ss;
        ini_ss << "Object " << t_atk << "\n"
               << "  KindOf = VEHICLE SELECTABLE\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 500\n    InitialHealth = 500\n  End\n"
               << "End\n";

        ini_ss << "Object " << t_act << "\n"
               << "  KindOf = SELECTABLE STRUCTURE\n"
               << "  StructureRubbleHeight = " << static_cast<Int>(rubble_h) << "\n"
               << "  ArmorSet\n    Conditions = None\n    Armor = BodyBaseArmor\n    DamageFX = BodyHitFX\n  End\n"
               << "  ArmorSet\n    Conditions = VETERAN\n    Armor = BodyVetArmor\n    DamageFX = BodyHitFX\n  End\n"
               << "  Body = ActiveBody ModuleTag_Body\n"
               << "    MaxHealth = " << max_hp << "\n"
               << "    InitialHealth = " << init_hp << "\n"
               << "    SubdualDamageCap = " << sub_cap << "\n"
               << "    SubdualDamageHealRate = " << sub_rate_ms << "\n"
               << "    SubdualDamageHealAmount = " << sub_heal << "\n"
               << "  End\n"
               << "End\n";

        ini_ss << "Object " << t_high << "\n"
               << "  KindOf = SELECTABLE\n"
               << "  ArmorSet\n    Conditions = None\n    Armor = BodyBaseArmor\n  End\n"
               << "  Body = HighlanderBody ModuleTag_Body\n    MaxHealth = " << max_hp << "\n    InitialHealth = " << max_hp << "\n  End\n"
               << "End\n";

        ini_ss << "Object " << t_imm << "\n"
               << "  KindOf = SELECTABLE\n"
               << "  Body = ImmortalBody ModuleTag_Body\n    MaxHealth = " << max_hp << "\n    InitialHealth = " << max_hp << "\n  End\n"
               << "End\n";

        ini_ss << "Object " << t_inac << "\n"
               << "  KindOf = SELECTABLE\n"
               << "  Body = InactiveBody ModuleTag_Body\n  End\n"
               << "End\n";

        ini_ss << "Object " << t_str << "\n"
               << "  KindOf = STRUCTURE SELECTABLE\n"
               << "  StructureRubbleHeight = " << static_cast<Int>(rubble_h) << "\n"
               << "  Body = StructureBody ModuleTag_Body\n    MaxHealth = 150\n    InitialHealth = 150\n  End\n"
               << "End\n";

        ini_ss << "Object " << t_hive << "\n"
               << "  KindOf = STRUCTURE SELECTABLE\n"
               << "  Body = HiveStructureBody ModuleTag_Body\n"
               << "    MaxHealth = 180\n    InitialHealth = 180\n"
               << "    PropagateDamageTypesToSlavesWhenExisting = NONE +SMALL_ARMS\n"
               << "    SwallowDamageTypesIfSlavesNotExisting = ALL -UNRESISTABLE\n"
               << "  End\n"
               << "End\n";

        ini_ss << "Object " << t_und << "\n"
               << "  KindOf = VEHICLE SELECTABLE\n"
               << "  ArmorSet\n    Conditions = None\n    Armor = BodyBaseArmor\n  End\n"
               << "  ArmorSet\n    Conditions = SECOND_LIFE\n    Armor = BodyVetArmor\n  End\n"
               << "  Body = UndeadBody ModuleTag_Body\n"
               << "    MaxHealth = 120\n    InitialHealth = 120\n"
               << "    SecondLifeMaxHealth = " << sec_max_hp << "\n"
               << "  End\n"
               << "  Behavior = SlowDeathBehavior ModuleTag_SD\n"
               << "    ProbabilityModifier = 10\n"
               << "    DestructionDelay = 1000\n"
               << "  End\n"
               << "End\n";

        w.load_ini(ini_ss.str(), INI_LOAD_OVERWRITE);

        Object* atk = w.spawn_object(t_atk.c_str(), w.team_b);
        Object* act = w.spawn_object(t_act.c_str(), w.team_a);
        if (start_under_const) {
            act->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
        }

        BodyModuleInterface* b_act = act->getBodyModule();
        Snapshot* s_act = static_cast<ActiveBody*>(b_act);
        Real act_hp0 = b_act->getHealth();
        Int act_ds0 = static_cast<Int>(b_act->getDamageState());
        UnsignedInt sub_rate_f = b_act->getSubdualDamageHealRate();
        std::string act_x0 = oracle_wp51::World::snapshot_save_hex(s_act);

        DamageInfoInput est_in;
        est_in.m_damageType = DAMAGE_SNIPER;
        est_in.m_amount = 50.0f;
        Real act_est_snip = b_act->estimateDamage(est_in);
        est_in.m_damageType = DAMAGE_SUBDUAL_MISSILE;
        est_in.m_amount = 30.0f;
        Real act_est_sub = b_act->estimateDamage(est_in);
        est_in.m_damageType = DAMAGE_KILL_GARRISONED;
        est_in.m_amount = 2.0f;
        Real act_est_kg = b_act->estimateDamage(est_in);

        b_act->applyDamageScalar(scalar1);

        DamageInfo hit1;
        hit1.in.m_sourceID = atk->getID();
        hit1.in.m_damageType = kDmgTypes[dt_idx];
        hit1.in.m_amount = dmg1;
        hit1.in.m_kill = kill1 ? TRUE : FALSE;
        b_act->attemptDamage(&hit1);

        Real act_dealt1 = hit1.out.m_actualDamageDealt;
        Real act_clip1 = hit1.out.m_actualDamageClipped;
        Real act_hp1 = b_act->getHealth();
        Real act_prev1 = b_act->getPreviousHealth();
        Int act_ds1 = static_cast<Int>(b_act->getDamageState());
        bool act_dead1 = (act->isEffectivelyDead() != FALSE);
        UnsignedInt act_dmg_ts1 = b_act->getLastDamageTimestamp() ? *b_act->getLastDamageTimestamp() : 0xffffffffu;
        ObjectID act_clr_atk1 = b_act->getClearableLastAttacker();
        Real act_gz1 = act->getGeometryInfo().getMaxHeightAbovePosition();
        bool act_nocol1 = (act->getStatusBits().test(OBJECT_STATUS_NO_COLLISIONS) != FALSE);
        UnsignedInt act_rng_crc1 = GetGameLogicRandomSeedCRC();

        b_act->clearLastAttacker();
        ObjectID act_clr_atk2 = b_act->getClearableLastAttacker();
        b_act->setFrontCrushed(front_crushed ? TRUE : FALSE);
        b_act->setBackCrushed(back_crushed ? TRUE : FALSE);
        b_act->setArmorSetFlag(ARMORSET_CRATE_UPGRADE_ONE);
        std::string act_x1 = oracle_wp51::World::snapshot_save_hex(s_act);

        // Second fresh ActiveBody for healing / subdual / max_health / initial_health / setDamageState / veterancy
        Object* act2 = w.spawn_object(t_act.c_str(), w.team_a);
        BodyModuleInterface* b2 = act2->getBodyModule();
        Snapshot* s2 = static_cast<ActiveBody*>(b2);
        DamageInfo pre_dmg;
        pre_dmg.in.m_sourceID = atk->getID();
        pre_dmg.in.m_damageType = DAMAGE_SMALL_ARMS;
        pre_dmg.in.m_amount = 80.0f;
        b2->attemptDamage(&pre_dmg);

        DamageInfo heal_info;
        heal_info.in.m_damageType = DAMAGE_HEALING;
        heal_info.in.m_amount = heal1;
        b2->attemptHealing(&heal_info);
        Real heal_dealt = heal_info.out.m_actualDamageDealt;
        Real heal_clip = heal_info.out.m_actualDamageClipped;
        Real heal_hp = b2->getHealth();
        UnsignedInt heal_ts = b2->getLastHealingTimestamp() ? *b2->getLastHealingTimestamp() : 0xffffffffu;

        DamageInfo sub_hit;
        sub_hit.in.m_damageType = DAMAGE_SUBDUAL_MISSILE;
        sub_hit.in.m_amount = sub_cap;
        b2->attemptDamage(&sub_hit);
        bool sub_is_sub1 = (static_cast<ActiveBody*>(b2)->isSubdued() != FALSE);
        bool sub_dis1 = (act2->isDisabledByType(DISABLED_SUBDUED) != FALSE);

        DamageInfo sub_rec;
        sub_rec.in.m_damageType = DAMAGE_SUBDUAL_UNRESISTABLE;
        sub_rec.in.m_amount = -(sub_heal * 20.0f);
        b2->attemptDamage(&sub_rec);
        bool sub_is_sub2 = (static_cast<ActiveBody*>(b2)->isSubdued() != FALSE);
        Real sub_after = b2->getCurrentSubdualDamageAmount();

        b2->setMaxHealth(new_max_hp, kChangeTypes[ct_idx]);
        Real maxhp_h = b2->getHealth();
        Real maxhp_m = b2->getMaxHealth();

        b2->setInitialHealth(initial_pct);
        Real init_h = b2->getHealth();

        b2->setDamageState(kBodyStates[ds_idx]);
        Real setds_h = b2->getHealth();
        Int setds_s = static_cast<Int>(b2->getDamageState());

        // Restore nonzero health before veterancy test
        b2->internalChangeHealth(new_max_hp * 0.5f);
        b2->onVeterancyLevelChanged(LEVEL_REGULAR, LEVEL_HEROIC, FALSE);
        Real vet_h = b2->getHealth();
        Real vet_m = b2->getMaxHealth();
        bool vet_flag_hero = (b2->testArmorSetFlag(ARMORSET_HERO) != FALSE);

        b2->setIndestructible(TRUE);
        DamageInfo ind_hit;
        ind_hit.in.m_damageType = DAMAGE_UNRESISTABLE;
        ind_hit.in.m_amount = 9999.0f;
        Real ind_before = b2->getHealth();
        b2->attemptDamage(&ind_hit);
        bool ind_unch = (b2->getHealth() == ind_before);
        std::string act2_x1 = oracle_wp51::World::snapshot_save_hex(s2);

        // HighlanderBody
        Object* o_high = w.spawn_object(t_high.c_str(), w.team_a);
        BodyModuleInterface* b_high = o_high->getBodyModule();
        DamageInfo hd1;
        hd1.in.m_damageType = DAMAGE_EXPLOSION;
        hd1.in.m_amount = max_hp * 3.0f;
        b_high->attemptDamage(&hd1);
        Real high_hp1 = b_high->getHealth();
        bool high_dead1 = (o_high->isEffectivelyDead() != FALSE);
        std::string high_x1 = oracle_wp51::World::snapshot_save_hex(static_cast<HighlanderBody*>(b_high));
        DamageInfo hd2;
        hd2.in.m_damageType = DAMAGE_UNRESISTABLE;
        hd2.in.m_amount = 50.0f;
        b_high->attemptDamage(&hd2);
        Real high_hp2 = b_high->getHealth();
        bool high_dead2 = (o_high->isEffectivelyDead() != FALSE);

        // ImmortalBody
        Object* o_imm = w.spawn_object(t_imm.c_str(), w.team_a);
        BodyModuleInterface* b_imm = o_imm->getBodyModule();
        b_imm->internalChangeHealth(-9999.0f);
        DamageInfo im_hit;
        im_hit.in.m_damageType = DAMAGE_UNRESISTABLE;
        im_hit.in.m_amount = 9999.0f;
        b_imm->attemptDamage(&im_hit);
        Real imm_hp = b_imm->getHealth();
        bool imm_dead = (o_imm->isEffectivelyDead() != FALSE);
        std::string imm_x1 = oracle_wp51::World::snapshot_save_hex(static_cast<ImmortalBody*>(b_imm));

        // InactiveBody
        Object* o_inac = w.spawn_object(t_inac.c_str(), w.team_a);
        BodyModuleInterface* b_inac = o_inac->getBodyModule();
        bool inac_dead0 = (o_inac->isEffectivelyDead() != FALSE);
        Real inac_hp0 = b_inac->getHealth();
        DamageInfo in_d1;
        in_d1.in.m_damageType = DAMAGE_EXPLOSION;
        in_d1.in.m_amount = 100.0f;
        b_inac->attemptDamage(&in_d1);
        bool inac_noeff1 = (in_d1.out.m_noEffect != FALSE);
        DamageInfo in_d2;
        in_d2.in.m_damageType = DAMAGE_UNRESISTABLE;
        in_d2.in.m_amount = 1.0f;
        b_inac->attemptDamage(&in_d2);
        bool inac_noeff2 = (in_d2.out.m_noEffect != FALSE);
        std::string inac_x1 = oracle_wp51::World::snapshot_save_hex(static_cast<InactiveBody*>(b_inac));

        // StructureBody
        Object* o_str = w.spawn_object(t_str.c_str(), w.team_a);
        StructureBody* b_str = static_cast<StructureBody*>(o_str->getBodyModule());
        b_str->setConstructorObject(atk);
        DamageInfo sd_hit;
        sd_hit.in.m_damageType = DAMAGE_UNRESISTABLE;
        sd_hit.in.m_amount = 500.0f;
        b_str->attemptDamage(&sd_hit);
        Int str_ds = static_cast<Int>(b_str->getDamageState());
        Real str_gz = o_str->getGeometryInfo().getMaxHeightAbovePosition();
        bool str_nocol = (o_str->getStatusBits().test(OBJECT_STATUS_NO_COLLISIONS) != FALSE);
        std::string str_x1 = oracle_wp51::World::snapshot_save_hex(b_str);

        // HiveStructureBody
        Object* o_hive = w.spawn_object(t_hive.c_str(), w.team_a);
        HiveStructureBody* b_hive = static_cast<HiveStructureBody*>(o_hive->getBodyModule());
        DamageInfo hv_hit;
        hv_hit.in.m_sourceID = atk->getID();
        hv_hit.in.m_damageType = DAMAGE_SMALL_ARMS;
        hv_hit.in.m_amount = 20.0f;
        o_hive->getBodyModule()->attemptDamage(&hv_hit);
        Real hive_hp = b_hive->getHealth();
        std::string hive_x1 = oracle_wp51::World::snapshot_save_hex(b_hive);

        // UndeadBody (first lethal hit triggers second life, second kills)
        Object* o_und = w.spawn_object(t_und.c_str(), w.team_a);
        UndeadBody* b_und = static_cast<UndeadBody*>(o_und->getBodyModule());
        UnsignedInt und_rng_pre = GetGameLogicRandomSeedCRC();
        DamageInfo ud1;
        ud1.in.m_damageType = DAMAGE_SMALL_ARMS;
        ud1.in.m_amount = 300.0f;
        b_und->attemptDamage(&ud1);
        Real und_hp1 = b_und->getHealth();
        Real und_max1 = b_und->getMaxHealth();
        bool und_arm_sec = (b_und->testArmorSetFlag(ARMORSET_SECOND_LIFE) != FALSE);
        std::string und_x1 = oracle_wp51::World::snapshot_save_hex(b_und);
        DamageInfo ud2;
        ud2.in.m_damageType = DAMAGE_SMALL_ARMS;
        ud2.in.m_amount = sec_max_hp * 4.0f + 10.0f;
        b_und->attemptDamage(&ud2);
        Real und_hp2 = b_und->getHealth();
        bool und_dead2 = (o_und->isEffectivelyDead() != FALSE);

        TheGameLogic->destroyObject(atk);
        TheGameLogic->destroyObject(act);
        TheGameLogic->destroyObject(act2);
        TheGameLogic->destroyObject(o_high);
        TheGameLogic->destroyObject(o_imm);
        TheGameLogic->destroyObject(o_inac);
        TheGameLogic->destroyObject(o_str);
        TheGameLogic->destroyObject(o_hive);
        TheGameLogic->destroyObject(o_und);
        TheGameLogic->processDestroyList();

        if (seed == 0) {
            std::ostringstream d;
            d << "{"
              << "\"act_hp0\":" << oracle::fbits(act_hp0)
              << ",\"act_hp1\":" << oracle::fbits(act_hp1)
              << ",\"act_ds1\":" << act_ds1
              << ",\"act_rng_crc1\":" << act_rng_crc1
              << ",\"sub_rate_f\":" << sub_rate_f
              << ",\"high_hp1\":" << oracle::fbits(high_hp1)
              << ",\"imm_hp\":" << oracle::fbits(imm_hp)
              << ",\"str_ds\":" << str_ds
              << ",\"hive_hp\":" << oracle::fbits(hive_hp)
              << ",\"und_hp1\":" << oracle::fbits(und_hp1)
              << ",\"und_arm_sec\":" << (und_arm_sec ? 1 : 0)
              << "}";
            s0_diag = d.str();
        }

        if (seed > 0) vecs << ",\n";
        vecs << "    {"
             << "\"seed\":" << seed
             << ",\"in\":{\"frame\":" << frame0
             << ",\"rng_seed\":" << rng_seed
             << ",\"max_hp\":" << oracle::fbits(max_hp)
             << ",\"init_hp\":" << oracle::fbits(init_hp)
             << ",\"sub_cap\":" << oracle::fbits(sub_cap)
             << ",\"sub_rate_ms\":" << sub_rate_ms
             << ",\"sub_heal\":" << oracle::fbits(sub_heal)
             << ",\"scalar1\":" << oracle::fbits(scalar1)
             << ",\"dmg1\":" << oracle::fbits(dmg1)
             << ",\"dt_idx\":" << dt_idx
             << ",\"dt_name\":\"" << kDmgNames[dt_idx] << "\""
             << ",\"kill1\":" << (kill1 ? 1 : 0)
             << ",\"heal1\":" << oracle::fbits(heal1)
             << ",\"ct_idx\":" << ct_idx
             << ",\"new_max_hp\":" << oracle::fbits(new_max_hp)
             << ",\"initial_pct\":" << initial_pct
             << ",\"ds_idx\":" << ds_idx
             << ",\"hero_bonus\":" << oracle::fbits(hero_bonus)
             << ",\"rubble_h\":" << oracle::fbits(rubble_h)
             << ",\"sec_max_hp\":" << oracle::fbits(sec_max_hp)
             << ",\"front_crushed\":" << (front_crushed ? 1 : 0)
             << ",\"back_crushed\":" << (back_crushed ? 1 : 0)
             << ",\"start_uc\":" << (start_under_const ? 1 : 0)
             << "}"
             << ",\"out\":{"
             << "\"act0\":[" << oracle::fbits(act_hp0) << "," << act_ds0 << "," << sub_rate_f << ","
             << oracle::fbits(act_est_snip) << "," << oracle::fbits(act_est_sub) << "," << oracle::fbits(act_est_kg) << "]"
             << ",\"act_x0\":\"" << act_x0 << "\""
             << ",\"act1\":[" << oracle::fbits(act_dealt1) << "," << oracle::fbits(act_clip1) << ","
             << oracle::fbits(act_hp1) << "," << oracle::fbits(act_prev1) << "," << act_ds1 << ","
             << (act_dead1 ? 1 : 0) << "," << act_dmg_ts1 << "," << act_clr_atk1 << ","
             << act_clr_atk2 << "," << oracle::fbits(act_gz1) << "," << (act_nocol1 ? 1 : 0) << ","
             << act_rng_crc1 << "]"
             << ",\"act_x1\":\"" << act_x1 << "\""
             << ",\"act2\":[" << oracle::fbits(heal_dealt) << "," << oracle::fbits(heal_clip) << ","
             << oracle::fbits(heal_hp) << "," << heal_ts << "," << (sub_is_sub1 ? 1 : 0) << ","
             << (sub_dis1 ? 1 : 0) << "," << (sub_is_sub2 ? 1 : 0) << "," << oracle::fbits(sub_after) << ","
             << oracle::fbits(maxhp_h) << "," << oracle::fbits(maxhp_m) << "," << oracle::fbits(init_h) << ","
             << oracle::fbits(setds_h) << "," << setds_s << "," << oracle::fbits(vet_h) << ","
             << oracle::fbits(vet_m) << "," << (vet_flag_hero ? 1 : 0) << "," << (ind_unch ? 1 : 0) << "]"
             << ",\"act2_x1\":\"" << act2_x1 << "\""
             << ",\"high\":[" << oracle::fbits(high_hp1) << "," << (high_dead1 ? 1 : 0) << ","
             << oracle::fbits(high_hp2) << "," << (high_dead2 ? 1 : 0) << "]"
             << ",\"high_x1\":\"" << high_x1 << "\""
             << ",\"imm\":[" << oracle::fbits(imm_hp) << "," << (imm_dead ? 1 : 0) << "]"
             << ",\"imm_x1\":\"" << imm_x1 << "\""
             << ",\"inac\":[" << (inac_dead0 ? 1 : 0) << "," << oracle::fbits(inac_hp0) << ","
             << (inac_noeff1 ? 1 : 0) << "," << (inac_noeff2 ? 1 : 0) << "]"
             << ",\"inac_x1\":\"" << inac_x1 << "\""
             << ",\"str\":[" << str_ds << "," << oracle::fbits(str_gz) << "," << (str_nocol ? 1 : 0) << "]"
             << ",\"str_x1\":\"" << str_x1 << "\""
             << ",\"hive\":[" << oracle::fbits(hive_hp) << "]"
             << ",\"hive_x1\":\"" << hive_x1 << "\""
             << ",\"und\":[" << und_rng_pre << "," << oracle::fbits(und_hp1) << ","
             << oracle::fbits(und_max1) << "," << (und_arm_sec ? 1 : 0) << ","
             << oracle::fbits(und_hp2) << "," << (und_dead2 ? 1 : 0) << "]"
             << ",\"und_x1\":\"" << und_x1 << "\""
             << "}"
             << "}";
    }

    std::ostringstream doc;
    doc << "{\n"
        << "  \"packet\": \"WP-5.1-B\",\n"
        << "  \"modules\": [\"ActiveBody\",\"BodyModule\",\"HighlanderBody\",\"HiveStructureBody\",\"ImmortalBody\",\"InactiveBody\",\"StructureBody\",\"UndeadBody\"],\n"
        << "  \"seed_count\": 1000,\n"
        << "  \"source_hashes\": {\n"
        << "    \"ActiveBody.cpp\": \"" << oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Body/ActiveBody.cpp") << "\",\n"
        << "    \"BodyModule.cpp\": \"" << oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Body/BodyModule.cpp") << "\",\n"
        << "    \"UndeadBody.cpp\": \"" << oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Body/UndeadBody.cpp") << "\"\n"
        << "  },\n"
        << "  \"seed0_diagnostics\": " << s0_diag << ",\n"
        << "  \"vectors\": [\n" << vecs.str() << "\n  ]\n}\n";

    oracle::write_file_atomic(out_path, doc.str());
    return 0;
}
