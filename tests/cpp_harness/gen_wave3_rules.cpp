#define DEFINE_DEATH_NAMES
#define DEFINE_SURFACECATEGORY_NAMES

#include "PreRTS.h"
#include "Common/DisabledTypes.h"
#include "Common/KindOf.h"
#include "Common/ModelState.h"
#include "Common/ObjectStatusTypes.h"
#include "Common/SpecialPowerMaskType.h"
#include "Common/SpecialPowerType.h"
#include "GameLogic/ArmorSet.h"
#include "GameLogic/Damage.h"
#include "GameLogic/LocomotorSet.h"
#include "GameLogic/WeaponBonusConditionFlags.h"
#include "GameLogic/WeaponSetFlags.h"
#include "GameLogic/WeaponSetType.h"
#include "GameLogic/WeaponStatus.h"

#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/System/DisabledTypes.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/System/KindOf.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/ObjectStatusTypes.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/GameLogic/System/Damage.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/BitFlags.cpp"

const AsciiString AsciiString::TheEmptyString;
ThingFactory* TheThingFactory = nullptr;

template<>
const char* const SpecialPowerMaskType::s_bitNameList[] =
{
    "SPECIAL_INVALID",
    "SPECIAL_DAISY_CUTTER",
    "SPECIAL_PARADROP_AMERICA",
    "SPECIAL_CARPET_BOMB",
    "SPECIAL_CLUSTER_MINES",
    "SPECIAL_EMP_PULSE",
    "SPECIAL_NAPALM_STRIKE",
    "SPECIAL_CASH_HACK",
    "SPECIAL_NEUTRON_MISSILE",
    "SPECIAL_SPY_SATELLITE",
    "SPECIAL_DEFECTOR",
    "SPECIAL_TERROR_CELL",
    "SPECIAL_AMBUSH",
    "SPECIAL_BLACK_MARKET_NUKE",
    "SPECIAL_ANTHRAX_BOMB",
    "SPECIAL_SCUD_STORM",
#ifdef ALLOW_DEMORALIZE
    "SPECIAL_DEMORALIZE",
#else
    "SPECIAL_DEMORALIZE_OBSOLETE",
#endif
    "SPECIAL_CRATE_DROP",
    "SPECIAL_A10_THUNDERBOLT_STRIKE",
    "SPECIAL_DETONATE_DIRTY_NUKE",
    "SPECIAL_ARTILLERY_BARRAGE",
    "SPECIAL_MISSILE_DEFENDER_LASER_GUIDED_MISSILES",
    "SPECIAL_REMOTE_CHARGES",
    "SPECIAL_TIMED_CHARGES",
    "SPECIAL_HELIX_NAPALM_BOMB",
    "SPECIAL_HACKER_DISABLE_BUILDING",
    "SPECIAL_TANKHUNTER_TNT_ATTACK",
    "SPECIAL_BLACKLOTUS_CAPTURE_BUILDING",
    "SPECIAL_BLACKLOTUS_DISABLE_VEHICLE_HACK",
    "SPECIAL_BLACKLOTUS_STEAL_CASH_HACK",
    "SPECIAL_INFANTRY_CAPTURE_BUILDING",
    "SPECIAL_RADAR_VAN_SCAN",
    "SPECIAL_SPY_DRONE",
    "SPECIAL_DISGUISE_AS_VEHICLE",
    "SPECIAL_BOOBY_TRAP",
    "SPECIAL_REPAIR_VEHICLES",
    "SPECIAL_PARTICLE_UPLINK_CANNON",
    "SPECIAL_CASH_BOUNTY",
    "SPECIAL_CHANGE_BATTLE_PLANS",
    "SPECIAL_CIA_INTELLIGENCE",
    "SPECIAL_CLEANUP_AREA",
    "SPECIAL_LAUNCH_BAIKONUR_ROCKET",
    "SPECIAL_SPECTRE_GUNSHIP",
    "SPECIAL_GPS_SCRAMBLER",
    "SPECIAL_FRENZY",
    "SPECIAL_SNEAK_ATTACK",
    "SPECIAL_CHINA_CARPET_BOMB",
    "EARLY_SPECIAL_CHINA_CARPET_BOMB",
    "SPECIAL_LEAFLET_DROP",
    "EARLY_SPECIAL_LEAFLET_DROP",
    "EARLY_SPECIAL_FRENZY",
    "SPECIAL_COMMUNICATIONS_DOWNLOAD",
    "EARLY_SPECIAL_REPAIR_VEHICLES",
    "SPECIAL_TANK_PARADROP",
    "SUPW_SPECIAL_PARTICLE_UPLINK_CANNON",
    "AIRF_SPECIAL_DAISY_CUTTER",
    "NUKE_SPECIAL_CLUSTER_MINES",
    "NUKE_SPECIAL_NEUTRON_MISSILE",
    "AIRF_SPECIAL_A10_THUNDERBOLT_STRIKE",
    "AIRF_SPECIAL_SPECTRE_GUNSHIP",
    "INFA_SPECIAL_PARADROP_AMERICA",
    "SLTH_SPECIAL_GPS_SCRAMBLER",
    "AIRF_SPECIAL_CARPET_BOMB",
    "SUPR_SPECIAL_CRUISE_MISSILE",
    "LAZR_SPECIAL_PARTICLE_UPLINK_CANNON",
    "SUPW_SPECIAL_NEUTRON_MISSILE",
    "SPECIAL_BATTLESHIP_BOMBARDMENT",
    nullptr
};
static_assert(ARRAY_SIZE(SpecialPowerMaskType::s_bitNameList) == SpecialPowerMaskType::NumBits + 1, "Incorrect array size");

template<>
const char* const WeaponSetFlags::s_bitNameList[] =
{
    "VETERAN",
    "ELITE",
    "HERO",
    "PLAYER_UPGRADE",
    "CRATEUPGRADE_ONE",
    "CRATEUPGRADE_TWO",
    "VEHICLE_HIJACK",
    "CARBOMB",
    "MINE_CLEARING_DETAIL",
    "WEAPON_RIDER1",
    "WEAPON_RIDER2",
    "WEAPON_RIDER3",
    "WEAPON_RIDER4",
    "WEAPON_RIDER5",
    "WEAPON_RIDER6",
    "WEAPON_RIDER7",
    "WEAPON_RIDER8",
    nullptr
};
static_assert(ARRAY_SIZE(WeaponSetFlags::s_bitNameList) == WeaponSetFlags::NumBits + 1, "Incorrect array size");

static uint32_t splitmix32(uint32_t& s) {
    uint32_t z = (s += 0x9e3779b9u);
    z = (z ^ (z >> 16)) * 0x85ebca6bu;
    z = (z ^ (z >> 13)) * 0xc2b2ae35u;
    return z ^ (z >> 16);
}

static uint32_t rot_crc(uint32_t crc, uint32_t w) {
    return ((crc << 1) | (crc >> 31)) + w;
}

static uint32_t str_crc(uint32_t crc, const char* s) {
    UnsignedInt n = s ? static_cast<UnsignedInt>(std::strlen(s)) : 0u;
    crc = rot_crc(crc, n);
    for (UnsignedInt i = 0; i < n; ++i) {
        crc = rot_crc(crc, static_cast<UnsignedByte>(s[i]));
    }
    return crc;
}

static uint32_t table_crc(uint32_t crc, const char* const* list, size_t count) {
    crc = rot_crc(crc, static_cast<uint32_t>(count));
    for (size_t i = 0; i < count; ++i) {
        crc = rot_crc(crc, static_cast<uint32_t>(i));
        crc = str_crc(crc, list[i]);
    }
    return crc;
}

int main(int argc, char** argv) {
    if (argc != 2) {
        std::fprintf(stderr, "Usage: gen_wave3_rules <out.json>\n");
        return 1;
    }

    FILE* fp = std::fopen(argv[1], "wb");
    if (!fp) { std::perror(argv[1]); return 1; }

    uint32_t names_crc = 0;
    names_crc = table_crc(names_crc, DisabledMaskType::getBitNames(), DISABLED_COUNT);
    names_crc = table_crc(names_crc, KindOfMaskType::getBitNames(), KINDOF_COUNT);
    names_crc = table_crc(names_crc, ModelConditionFlags::getBitNames(), MODELCONDITION_COUNT);
    names_crc = table_crc(names_crc, ObjectStatusMaskType::getBitNames(), OBJECT_STATUS_COUNT);
    names_crc = table_crc(names_crc, SpecialPowerMaskType::getBitNames(), SPECIALPOWER_COUNT);
    names_crc = table_crc(names_crc, ArmorSetFlags::getBitNames(), ARMORSET_COUNT);
    names_crc = table_crc(names_crc, DamageTypeFlags::getBitNames(), DAMAGE_NUM_TYPES);
    names_crc = table_crc(names_crc, TheDeathNames, DEATH_NUM_TYPES);
    names_crc = table_crc(names_crc, TheLocomotorSurfaceTypeNames, 5);
    names_crc = table_crc(names_crc, WeaponSetFlags::getBitNames(), WEAPONSET_COUNT);

    uint64_t subdual_mask = 0;
    uint64_t health_dmg_mask = 0;
    for (Int d = 0; d < DAMAGE_NUM_TYPES; ++d) {
        auto dt = static_cast<DamageType>(d);
        if (IsSubdualDamage(dt)) subdual_mask |= (1ULL << d);
        if (IsHealthDamagingDamage(dt)) health_dmg_mask |= (1ULL << d);
    }

    // Record KINDOFMASK_FS hex and DISABLEDMASK_ALL hex
    uint32_t fs_hex_crc = str_crc(0, KINDOFMASK_FS.toHexString().str());

    std::fprintf(stderr,
        "Counts: DISABLED=%d KINDOF=%d MODELCOND=%d OBJSTATUS=%d SPECIAL=%d ARMORSET=%d DAMAGE=%d DEATH=%d WEAPONSET=%d WEAPONSTATUS=%d\n",
        DISABLED_COUNT, KINDOF_COUNT, MODELCONDITION_COUNT, OBJECT_STATUS_COUNT,
        SPECIALPOWER_COUNT, ARMORSET_COUNT, DAMAGE_NUM_TYPES, DEATH_NUM_TYPES,
        WEAPONSET_COUNT, WEAPON_STATUS_COUNT);

    std::fprintf(fp, "{\n  \"packet_id\": \"WP3_2a_RulesEnums\",\n  \"module\": \"Phase3RulesEnumsBundle\",\n"
                     "  \"oracle\": \"vendor/GeneralsX unmodified Phase 3 enum headers & bitflag tables (-DRTS_ZEROHOUR=1 -DRTS_GENERALS=0)\",\n"
                     "  \"vector_count\": 1000,\n  \"vectors\": [\n");

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed * 0x9e3779b9u + 0x517cc1b7u;

        // Randomly populate each mask type using 3-6 bit indices, plus mix-case lookup names
        KindOfMaskType kf_a, kf_b;
        for (int i = 0; i < 6; ++i) {
            Int idx_a = static_cast<Int>(splitmix32(rng) % KINDOF_COUNT);
            Int idx_b = static_cast<Int>(splitmix32(rng) % KINDOF_COUNT);
            kf_a.set(idx_a, TRUE);
            kf_b.set(idx_b, TRUE);
        }
        KindOfMaskType kf_res = kf_a;
        kf_res.clearAndSet(kf_b, MAKE_KINDOF_MASK(KINDOF_STRUCTURE));
        Bool kf_match = kf_res.testSetAndClear(MAKE_KINDOF_MASK(KINDOF_STRUCTURE), kf_b);

        DisabledMaskType dis;
        dis.set(static_cast<Int>(splitmix32(rng) % DISABLED_COUNT), TRUE);
        dis.set(static_cast<Int>(splitmix32(rng) % DISABLED_COUNT), TRUE);

        ObjectStatusMaskType st;
        st.set(static_cast<Int>(splitmix32(rng) % OBJECT_STATUS_COUNT), TRUE);
        st.set(static_cast<Int>(splitmix32(rng) % OBJECT_STATUS_COUNT), TRUE);

        SpecialPowerMaskType sp;
        sp.set(static_cast<Int>(splitmix32(rng) % SPECIALPOWER_COUNT), TRUE);
        sp.set(static_cast<Int>(splitmix32(rng) % SPECIALPOWER_COUNT), TRUE);

        WeaponSetFlags ws;
        ws.set(static_cast<Int>(splitmix32(rng) % WEAPONSET_COUNT), TRUE);
        ws.set(static_cast<Int>(splitmix32(rng) % WEAPONSET_COUNT), TRUE);

        DamageTypeFlags df = DAMAGE_TYPE_FLAGS_ALL;
        Int clear_dt = static_cast<Int>(splitmix32(rng) % DAMAGE_NUM_TYPES);
        df = clearDamageTypeFlag(df, static_cast<DamageType>(clear_dt));

        // Mixed-case lookup test across KindOf & SpecialPower & ObjectStatus
        Int k_idx = static_cast<Int>(splitmix32(rng) % KINDOF_COUNT);
        Int sp_idx = static_cast<Int>(splitmix32(rng) % SPECIALPOWER_COUNT);
        Int st_idx = static_cast<Int>(splitmix32(rng) % OBJECT_STATUS_COUNT);
        std::string k_mix = KindOfMaskType::getNameFromSingleBit(k_idx);
        for (size_t i = 0; i < k_mix.size(); ++i) {
            if ((i + seed) & 1) k_mix[i] = static_cast<char>(std::tolower(static_cast<unsigned char>(k_mix[i])));
        }
        Int k_looked = KindOfMaskType::getSingleBitFromName(k_mix.c_str());

        uint32_t out_crc = 0;
        out_crc = rot_crc(out_crc, kf_res.toUnsignedInt());
        out_crc = rot_crc(out_crc, kf_match);
        out_crc = rot_crc(out_crc, dis.toUnsignedInt());
        out_crc = rot_crc(out_crc, st.toUnsignedInt());
        out_crc = rot_crc(out_crc, sp.toUnsignedInt());
        out_crc = rot_crc(out_crc, ws.toUnsignedInt());
        out_crc = rot_crc(out_crc, df.toUnsignedInt());
        out_crc = rot_crc(out_crc, static_cast<uint32_t>(k_looked));
        out_crc = str_crc(out_crc, sp.toHexString().str());
        out_crc = str_crc(out_crc, st.toHexString().str());
        out_crc = str_crc(out_crc, ws.toHexString().str());
        out_crc = str_crc(out_crc, dis.toHexString().str());
        out_crc = str_crc(out_crc, SpecialPowerMaskType::getNameFromSingleBit(sp_idx));
        out_crc = str_crc(out_crc, ObjectStatusMaskType::getNameFromSingleBit(st_idx));

        // Reconstruct seed rng cleanly in Rust using seed directly or emit input arrays
        // Emit compact JSON so Rust test doesn't duplicate RNG!
        std::fprintf(fp,
            "    {\"seed\":%u,\"counts\":[%d,%d,%d,%d,%d,%d,%d,%d,%d,%d],"
            "\"names_crc\":%u,\"fs_hex_crc\":%u,\"subdual_mask\":%llu,\"health_dmg_mask\":%llu,"
            "\"kf_res_hex\":\"%s\",\"out_crc\":%u}%s\n",
            seed,
            DISABLED_COUNT, KINDOF_COUNT, MODELCONDITION_COUNT, OBJECT_STATUS_COUNT,
            SPECIALPOWER_COUNT, ARMORSET_COUNT, DAMAGE_NUM_TYPES, DEATH_NUM_TYPES,
            WEAPONSET_COUNT, WEAPON_STATUS_COUNT,
            names_crc, fs_hex_crc,
            static_cast<unsigned long long>(subdual_mask),
            static_cast<unsigned long long>(health_dmg_mask),
            kf_res.toHexString().str(),
            out_crc,
            (seed + 1u < 1000u) ? "," : "");
    }

    std::fprintf(fp, "  ]\n}\n");
    std::fclose(fp);
    std::fprintf(stderr, "Generated 1000 WP3_2a_RulesEnums vectors to %s\n", argv[1]);
    return 0;
}
