// PKT-399 (Weapon, WeaponTemplate, WeaponBonus, WeaponBonusSet, WeaponStore) &
// PKT-400 (WeaponTemplateSet, WeaponSet) (WP-5.5) C++ Golden Vector Generator (Zero-Stub Build).
// Compiles directly against pre-built vendor/GeneralsX/build/macos-vulkan via scripts/build_oracle.py.
//
// Canonical build & run command:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt399_weapon.cpp -o /tmp/gen_pkt399
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy /tmp/gen_pkt399 docs/golden_vectors/PKT-399.json
//
// Determinism & UB verification builds (must emit byte-identical JSON):
//   --opt O0, --opt O1, --opt O2, and --sanitize (UBSan).

#include "oracle_common.h"

#include <sstream>
#include <unistd.h>

#include "Common/GlobalData.h"
#include "Common/NameKeyGenerator.h"
#include "Common/RandomValue.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Weapon.h"
#include "GameLogic/WeaponSet.h"

ORACLE_DEFINE_LINK_GLOBALS("gen_pkt399_weapon")

struct GLFrameTag { using type = UnsignedInt GameLogic::*; };
template struct MemberAccess<GLFrameTag, &GameLogic::m_frame>;
UnsignedInt GameLogic::* access_member(GLFrameTag);

struct TTNameTag { using type = AsciiString ThingTemplate::*; };
template struct MemberAccess<TTNameTag, &ThingTemplate::m_nameString>;
AsciiString ThingTemplate::* access_member(TTNameTag);

#define DECL_W_TAG(Name, Ty, Field) \
    struct Name { using type = Ty Weapon::*; }; \
    template struct MemberAccess<Name, &Weapon::Field>; \
    Ty Weapon::* access_member(Name);

DECL_W_TAG(WStatusTag, WeaponStatus, m_status)
DECL_W_TAG(WAmmoTag, UnsignedInt, m_ammoInClip)
DECL_W_TAG(WNextShotTag, UnsignedInt, m_whenWeCanFireAgain)
DECL_W_TAG(WPreAtkTag, UnsignedInt, m_whenPreAttackFinished)
DECL_W_TAG(WReloadStartTag, UnsignedInt, m_whenLastReloadStarted)
DECL_W_TAG(WLastFireTag, UnsignedInt, m_lastFireFrame)
DECL_W_TAG(WSuspendFXTag, UnsignedInt, m_suspendFXFrame)
DECL_W_TAG(WStreamIDTag, ObjectID, m_projectileStreamID)
DECL_W_TAG(WMaxShotsTag, Int, m_maxShotCount)
DECL_W_TAG(WCurBarrelTag, Int, m_curBarrel)
DECL_W_TAG(WNumBarrelTag, Int, m_numShotsForCurBarrel)
DECL_W_TAG(WScatterUnusedTag, std::vector<Int>, m_scatterTargetsUnused)
DECL_W_TAG(WPitchLimTag, Bool, m_pitchLimited)
DECL_W_TAG(WLeechTag, Bool, m_leechWeaponRangeActive)

#define DECL_WT_TAG(Name, Ty, Field) \
    struct Name { using type = Ty WeaponTemplate::*; }; \
    template struct MemberAccess<Name, &WeaponTemplate::Field>; \
    Ty WeaponTemplate::* access_member(Name);

DECL_WT_TAG(WTMinDelayTag, Int, m_minDelayBetweenShots)
DECL_WT_TAG(WTMaxDelayTag, Int, m_maxDelayBetweenShots)

#define DECL_WS_TAG(Name, Ty, Field) \
    struct Name { using type = Ty WeaponSet::*; }; \
    template struct MemberAccess<Name, &WeaponSet::Field>; \
    Ty WeaponSet::* access_member(Name);

struct WSCurTmplSetTag { using type = const WeaponTemplateSet* WeaponSet::*; };
template struct MemberAccess<WSCurTmplSetTag, &WeaponSet::m_curWeaponTemplateSet>;
const WeaponTemplateSet* WeaponSet::* access_member(WSCurTmplSetTag);

struct WSWeaponsTag { using type = Weapon* (WeaponSet::*)[WEAPONSLOT_COUNT]; };
template struct MemberAccess<WSWeaponsTag, &WeaponSet::m_weapons>;
Weapon* (WeaponSet::*access_member(WSWeaponsTag))[WEAPONSLOT_COUNT];

DECL_WS_TAG(WSCurWpnTag, WeaponSlotType, m_curWeapon)
DECL_WS_TAG(WSCurLockTag, WeaponLockType, m_curWeaponLockedStatus)
DECL_WS_TAG(WSFilledMaskTag, UnsignedInt, m_filledWeaponSlotMask)
DECL_WS_TAG(WSTotalAntiTag, Int, m_totalAntiMask)
DECL_WS_TAG(WSTotalDmgTag, DamageTypeFlags, m_totalDamageTypeMask)
DECL_WS_TAG(WSHasPitchTag, Bool, m_hasPitchLimit)
DECL_WS_TAG(WSHasDmgTag, Bool, m_hasDamageWeapon)

static const char* kDamageNames[] = {
    "EXPLOSION", "CRUSH", "ARMOR_PIERCING", "SMALL_ARMS", "GATTLING", "RADIATION",
    "FLAME", "LASER", "SNIPER", "POISON", "HEALING", "UNRESISTABLE", "WATER",
    "DEPLOY", "SURRENDER", "HACK", "KILL_PILOT", "PENALTY", "FALLING", "MELEE",
    "DISARM", "HAZARD_CLEANUP", "PARTICLE_BEAM", "TOPPLING", "INFANTRY_MISSILE",
    "AURORA_BOMB", "LAND_MINE", "JET_MISSILES", "STEALTHJET_MISSILES",
    "MOLOTOV_COCKTAIL", "COMANCHE_VULCAN", "SUBDUAL_MISSILE", "SUBDUAL_VEHICLE",
    "SUBDUAL_BUILDING", "SUBDUAL_UNRESISTABLE", "MICROWAVE", "KILL_GARRISONED",
    "STATUS"
};
static const char* kDeathNames[] = {
    "NORMAL", "NONE", "CRUSHED", "BURNED", "EXPLODED", "POISONED", "TOPPLED",
    "FLOODED", "SUICIDED", "LASERED", "DETONATED", "SPLATTED", "POISONED_BETA",
    "EXTRA_2", "EXTRA_3", "EXTRA_4", "EXTRA_5", "EXTRA_6", "EXTRA_7", "EXTRA_8",
    "POISONED_GAMMA"
};
static const char* kReloadNames[] = { "YES", "NO", "RETURN_TO_BASE" };
static const char* kPrefireNames[] = { "PER_SHOT", "PER_ATTACK", "PER_CLIP" };
static const char* kBonusCondNames[] = {
    "GARRISONED", "HORDE", "CONTINUOUS_FIRE_MEAN", "CONTINUOUS_FIRE_FAST",
    "NATIONALISM", "PLAYER_UPGRADE", "DRONE_SPOTTING", "DEMORALIZED_OBSOLETE",
    "ENTHUSIASTIC", "VETERAN", "ELITE", "HERO", "BATTLEPLAN_BOMBARDMENT",
    "BATTLEPLAN_HOLDTHELINE", "BATTLEPLAN_SEARCHANDDESTROY", "SUBLIMINAL",
    "SOLO_HUMAN_EASY", "SOLO_HUMAN_NORMAL", "SOLO_HUMAN_HARD",
    "SOLO_AI_EASY", "SOLO_AI_NORMAL", "SOLO_AI_HARD", "TARGET_FAERIE_FIRE",
    "FANATICISM", "FRENZY_ONE", "FRENZY_TWO", "FRENZY_THREE"
};
static const char* kBonusFieldNames[] = {
    "DAMAGE", "RADIUS", "RANGE", "RATE_OF_FIRE", "PRE_ATTACK"
};
static const char* kAffectsNames[] = {
    "SELF", "ALLIES", "ENEMIES", "NEUTRALS", "SUICIDE", "NOT_SIMILAR", "NOT_AIRBORNE"
};
static const char* kCollideNames[] = {
    "ALLIES", "ENEMIES", "STRUCTURES", "SHRUBBERY", "PROJECTILES", "WALLS",
    "SMALL_MISSILES", "BALLISTIC_MISSILES", "CONTROLLED_STRUCTURES"
};
static const char* kWeaponSetFlagNames[] = {
    "VETERAN", "ELITE", "HERO", "PLAYER_UPGRADE", "CRATEUPGRADE_ONE",
    "CRATEUPGRADE_TWO", "VEHICLE_HIJACK", "CARBOMB", "MINE_CLEARING_DETAIL",
    "WEAPON_RIDER1", "WEAPON_RIDER2", "WEAPON_RIDER3", "WEAPON_RIDER4",
    "WEAPON_RIDER5", "WEAPON_RIDER6", "WEAPON_RIDER7", "WEAPON_RIDER8"
};
static const char* kCmdSourceNames[] = {
    "FROM_PLAYER", "FROM_AI", "FROM_SCRIPT", "DEFAULT_SWITCH_WEAPON"
};
static const char* kKindOfSampleNames[] = {
    "INFANTRY", "VEHICLE", "STRUCTURE", "AIRCRAFT", "PROJECTILE", "SMALL_MISSILE"
};

static std::string build_weapon_ini(const char* name, uint32_t& rng, uint32_t seed) {
    auto rfloat = [&](int lo, int hi, Real scale) -> Real {
        int32_t span = hi - lo + 1;
        return static_cast<Real>(lo + static_cast<int32_t>(oracle::splitmix32(rng) % span)) * scale;
    };
    Real prim_dmg = (seed % 11 == 0) ? 0.0f : rfloat(5, 400, 0.5f);
    Real prim_rad = rfloat(0, 120, 0.5f);
    Real sec_dmg = (seed % 4 == 0) ? 0.0f : rfloat(2, 200, 0.5f);
    Real sec_rad = rfloat(0, 160, 0.5f);
    Real atk_range = (seed % 13 == 0) ? rfloat(4, 24, 0.5f) : rfloat(30, 800, 0.5f);
    Real min_range = (seed % 3 == 0) ? 0.0f : rfloat(4, 120, 0.5f);
    Real speed = rfloat(50, 1200, 1.0f);
    Real min_spd = rfloat(20, 600, 1.0f);
    Real aim_deg = static_cast<Real>(oracle::splitmix32(rng) % 45);
    Real recoil_deg = static_cast<Real>(oracle::splitmix32(rng) % 30);
    bool pitch_lim = (seed % 3 != 1);
    Real min_pitch = pitch_lim ? -static_cast<Real>(10 + (oracle::splitmix32(rng) % 70)) : -180.0f;
    Real max_pitch = pitch_lim ? static_cast<Real>(10 + (oracle::splitmix32(rng) % 70)) : 180.0f;
    Real rad_angle = static_cast<Real>(15 + (oracle::splitmix32(rng) % 166));
    int dmg_idx = static_cast<int>(oracle::splitmix32(rng) % 38);
    int death_idx = static_cast<int>(oracle::splitmix32(rng) % 21);
    int reload_idx = static_cast<int>(oracle::splitmix32(rng) % 3);
    int prefire_idx = static_cast<int>(oracle::splitmix32(rng) % 3);
    int clip_sz = static_cast<int>(oracle::splitmix32(rng) % 8);
    int clip_reload_ms = static_cast<int>((oracle::splitmix32(rng) % 40) * 100);
    int delay_min_ms = static_cast<int>(50 + (oracle::splitmix32(rng) % 1500));
    int delay_max_ms = delay_min_ms + static_cast<int>(oracle::splitmix32(rng) % 1200);
    bool range_delay = (seed % 2 == 1);
    int shots_barrel = 1 + static_cast<int>(oracle::splitmix32(rng) % 4);
    int pre_atk_ms = static_cast<int>((oracle::splitmix32(rng) % 25) * 50);
    int idle_reload_ms = static_cast<int>((oracle::splitmix32(rng) % 30) * 100);
    int coast_ms = static_cast<int>((oracle::splitmix32(rng) % 25) * 100);
    int susp_fx_ms = static_cast<int>((oracle::splitmix32(rng) % 20) * 50);
    uint32_t anti_m = 1u + (oracle::splitmix32(rng) % 255u);
    uint32_t aff_m = 1u + (oracle::splitmix32(rng) % 127u);
    uint32_t col_m = 1u + (oracle::splitmix32(rng) % 511u);
    int n_scatter = static_cast<int>(oracle::splitmix32(rng) % 4);
    int n_bonus = 1 + static_cast<int>(oracle::splitmix32(rng) % 3);

    std::ostringstream ss;
    ss << "Weapon " << name << "\n";
    ss << "  PrimaryDamage = " << prim_dmg << "\n";
    ss << "  PrimaryDamageRadius = " << prim_rad << "\n";
    ss << "  SecondaryDamage = " << sec_dmg << "\n";
    ss << "  SecondaryDamageRadius = " << sec_rad << "\n";
    ss << "  ShockWaveAmount = " << rfloat(0, 80, 0.5f) << "\n";
    ss << "  ShockWaveRadius = " << rfloat(0, 100, 0.5f) << "\n";
    ss << "  ShockWaveTaperOff = " << rfloat(0, 20, 0.05f) << "\n";
    ss << "  AttackRange = " << atk_range << "\n";
    ss << "  MinimumAttackRange = " << min_range << "\n";
    ss << "  RequestAssistRange = " << rfloat(0, 200, 1.0f) << "\n";
    ss << "  AcceptableAimDelta = " << aim_deg << "\n";
    ss << "  ScatterRadius = " << rfloat(0, 60, 0.5f) << "\n";
    ss << "  ScatterTargetScalar = " << rfloat(0, 80, 0.5f) << "\n";
    ss << "  ScatterRadiusVsInfantry = " << rfloat(0, 40, 0.5f) << "\n";
    ss << "  DamageType = " << kDamageNames[dmg_idx] << "\n";
    ss << "  DeathType = " << kDeathNames[death_idx] << "\n";
    ss << "  WeaponSpeed = " << speed << "\n";
    ss << "  MinWeaponSpeed = " << min_spd << "\n";
    ss << "  ScaleWeaponSpeed = " << ((seed & 1) ? "Yes" : "No") << "\n";
    ss << "  WeaponRecoil = " << recoil_deg << "\n";
    if (pitch_lim) {
        ss << "  MinTargetPitch = " << min_pitch << "\n";
        ss << "  MaxTargetPitch = " << max_pitch << "\n";
    }
    ss << "  RadiusDamageAngle = " << rad_angle << "\n";
    ss << "  ClipSize = " << clip_sz << "\n";
    ss << "  ClipReloadTime = " << clip_reload_ms << "\n";
    if (range_delay) {
        ss << "  DelayBetweenShots = Min:" << delay_min_ms << " Max:" << delay_max_ms << "\n";
    } else {
        ss << "  DelayBetweenShots = " << delay_min_ms << "\n";
    }
    ss << "  ContinuousFireOne = " << (1 + (oracle::splitmix32(rng) % 10)) << "\n";
    ss << "  ContinuousFireTwo = " << (2 + (oracle::splitmix32(rng) % 20)) << "\n";
    ss << "  ContinuousFireCoast = " << coast_ms << "\n";
    ss << "  AutoReloadWhenIdle = " << idle_reload_ms << "\n";
    ss << "  ShotsPerBarrel = " << shots_barrel << "\n";
    ss << "  DamageDealtAtSelfPosition = " << ((seed % 5 == 0) ? "Yes" : "No") << "\n";
    ss << "  AutoReloadsClip = " << kReloadNames[reload_idx] << "\n";
    ss << "  PreAttackDelay = " << pre_atk_ms << "\n";
    ss << "  PreAttackType = " << kPrefireNames[prefire_idx] << "\n";
    ss << "  ContinueAttackRange = " << rfloat(0, 150, 0.5f) << "\n";
    ss << "  SuspendFXDelay = " << susp_fx_ms << "\n";
    ss << "  LeechRangeWeapon = " << ((seed % 4 == 0) ? "Yes" : "No") << "\n";
    ss << "  CapableOfFollowingWaypoints = " << ((seed % 3 == 0) ? "Yes" : "No") << "\n";
    ss << "  ShowsAmmoPips = " << ((seed % 2 == 0) ? "Yes" : "No") << "\n";
    ss << "  AllowAttackGarrisonedBldgs = " << ((seed % 5 == 1) ? "Yes" : "No") << "\n";
    ss << "  PlayFXWhenStealthed = " << ((seed % 6 == 0) ? "Yes" : "No") << "\n";
    ss << "  MissileCallsOnDie = " << ((seed % 7 == 0) ? "Yes" : "No") << "\n";
    ss << "  AntiAirborneVehicle = " << ((anti_m & 0x01) ? "Yes" : "No") << "\n";
    ss << "  AntiGround = " << ((anti_m & 0x02) ? "Yes" : "No") << "\n";
    ss << "  AntiProjectile = " << ((anti_m & 0x04) ? "Yes" : "No") << "\n";
    ss << "  AntiSmallMissile = " << ((anti_m & 0x08) ? "Yes" : "No") << "\n";
    ss << "  AntiMine = " << ((anti_m & 0x10) ? "Yes" : "No") << "\n";
    ss << "  AntiAirborneInfantry = " << ((anti_m & 0x20) ? "Yes" : "No") << "\n";
    ss << "  AntiBallisticMissile = " << ((anti_m & 0x40) ? "Yes" : "No") << "\n";
    ss << "  AntiParachute = " << ((anti_m & 0x80) ? "Yes" : "No") << "\n";
    ss << "  RadiusDamageAffects =";
    for (int i = 0; i < 7; ++i) if ((aff_m >> i) & 1u) ss << " " << kAffectsNames[i];
    ss << "\n  ProjectileCollidesWith =";
    for (int i = 0; i < 9; ++i) if ((col_m >> i) & 1u) ss << " " << kCollideNames[i];
    ss << "\n";
    for (int s = 0; s < n_scatter; ++s) {
        int sx = static_cast<int>(oracle::splitmix32(rng) % 41) - 20;
        int sy = static_cast<int>(oracle::splitmix32(rng) % 41) - 20;
        ss << "  ScatterTarget = X:" << sx << " Y:" << sy << "\n";
    }
    for (int b = 0; b < n_bonus; ++b) {
        int bc = static_cast<int>(oracle::splitmix32(rng) % 27);
        int bf = static_cast<int>(oracle::splitmix32(rng) % 5);
        int bpct = 50 + static_cast<int>((oracle::splitmix32(rng) % 16) * 10);
        ss << "  WeaponBonus = " << kBonusCondNames[bc] << " " << kBonusFieldNames[bf] << " " << bpct << "%\n";
    }
    ss << "End\n";
    return ss.str();
}

int main(int argc, char** argv) {
    const char* out_path = (argc > 1) ? argv[1] : "docs/golden_vectors/PKT-399.json";
    oracle::init_memory();
    TheWritableGlobalData = NEW GlobalData;
    TheNameKeyGenerator = NEW NameKeyGenerator;
    TheNameKeyGenerator->init();
    TheGameLogic = NEW GameLogic;
    TheWeaponStore = NEW WeaponStore;

    const char* h_wpn = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h";
    const char* cpp_wpn = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Weapon.cpp";
    const char* h_wset = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/WeaponSet.h";
    const char* cpp_wset = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/WeaponSet.cpp";

    std::ostringstream out;
    out << "{\n"
        << "  \"packet_id\": \"PKT-399\",\n"
        << "  \"packets\": [\"PKT-399\", \"PKT-400\"],\n"
        << "  \"modules\": [\"Weapon\", \"WeaponSet\"],\n"
        << "  \"vector_count\": 1000,\n"
        << "  \"source_sha256\": {\n"
        << "    \"" << h_wpn << "\": \"" << oracle::sha256_file(h_wpn) << "\",\n"
        << "    \"" << cpp_wpn << "\": \"" << oracle::sha256_file(cpp_wpn) << "\",\n"
        << "    \"" << h_wset << "\": \"" << oracle::sha256_file(h_wset) << "\",\n"
        << "    \"" << cpp_wset << "\": \"" << oracle::sha256_file(cpp_wset) << "\"\n"
        << "  },\n  \"vectors\": [\n";

    oracle::MemINI mem_ini;
    std::vector<std::string> known_weapons;

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed ^ 0x399c0de1u;
        if (rng == 0) rng = 0x77773333u;

        std::string wname = "Wpn_" + std::to_string(seed);
        std::string w_ini = build_weapon_ini(wname.c_str(), rng, seed);
        mem_ini.parse_blocks(w_ini, INI_LOAD_OVERWRITE, [](INI* ini, const char* tok) {
            if (std::strcmp(tok, "Weapon") == 0) WeaponStore::parseWeaponTemplateDefinition(ini);
        });
        known_weapons.push_back(wname);

        bool do_override = (seed % 5 == 0);
        std::string ovr_ini;
        if (do_override) {
            ovr_ini = "Weapon " + wname + "\n  PrimaryDamage = 275\n  AttackRange = 320\nEnd\n";
            mem_ini.parse_blocks(ovr_ini, INI_LOAD_CREATE_OVERRIDES, [](INI* ini, const char* tok) {
                if (std::strcmp(tok, "Weapon") == 0) WeaponStore::parseWeaponTemplateDefinition(ini);
            });
        }

        const WeaponTemplate* tmpl = TheWeaponStore->findWeaponTemplate(wname.c_str());
        uint32_t rand_seed = 0x2000u + seed * 6151u;
        InitRandom(rand_seed);

        WeaponBonus bonus;
        uint32_t bflags = (oracle::splitmix32(rng) & 0x07ffffffu);
        if (tmpl->getExtraBonus()) {
            tmpl->getExtraBonus()->appendBonuses(bflags, bonus);
        }
        uint32_t bf_bits[WeaponBonus::FIELD_COUNT];
        for (int f = 0; f < WeaponBonus::FIELD_COUNT; ++f) {
            bf_bits[f] = oracle::fbits(bonus.getField(static_cast<WeaponBonus::Field>(f)));
        }

        uint32_t atk_rng = oracle::fbits(tmpl->getAttackRange(bonus));
        uint32_t unmod_rng = oracle::fbits(tmpl->getUnmodifiedAttackRange());
        uint32_t min_rng = oracle::fbits(tmpl->getMinimumAttackRange());
        Int shot_delay = tmpl->getDelayBetweenShots(bonus);
        UnsignedInt rand_crc_after_delay = GetGameLogicRandomSeedCRC();
        Int clip_reload = tmpl->getClipReloadTime(bonus);
        uint32_t prim_dmg = oracle::fbits(tmpl->getPrimaryDamage(bonus));
        uint32_t prim_rad = oracle::fbits(tmpl->getPrimaryDamageRadius(bonus));
        uint32_t sec_dmg = oracle::fbits(tmpl->getSecondaryDamage(bonus));
        uint32_t sec_rad = oracle::fbits(tmpl->getSecondaryDamageRadius(bonus));
        Int pre_atk_delay = tmpl->getPreAttackDelay(bonus);
        int is_contact = tmpl->isContactWeapon() ? 1 : 0;

        UnsignedInt frame0 = (seed == 0) ? 100u : (oracle::splitmix32(rng) % 4000u);
        TheGameLogic->*access_member(GLFrameTag()) = frame0;
        WeaponSlotType wslot = static_cast<WeaponSlotType>(seed % WEAPONSLOT_COUNT);
        Weapon* wpn = TheWeaponStore->allocateNewWeapon(tmpl, wslot);
        int is_dmg_wpn = wpn->isDamageWeapon() ? 1 : 0;
        int is_pitch_lim = wpn->isPitchLimited() ? 1 : 0;

        Real pct1 = static_cast<Real>(1 + (oracle::splitmix32(rng) % 10)) * 0.1f;
        Real pct2 = static_cast<Real>(oracle::splitmix32(rng) % 11) * 0.1f;
        Bool allow_red = (seed & 1) ? TRUE : FALSE;
        wpn->setClipPercentFull(pct1, TRUE);
        UnsignedInt f1 = frame0 + 5u + (oracle::splitmix32(rng) % 40u);
        TheGameLogic->*access_member(GLFrameTag()) = f1;
        wpn->setClipPercentFull(pct2, allow_red);

        WeaponStatus st_override_val = static_cast<WeaponStatus>(oracle::splitmix32(rng) % WEAPON_STATUS_COUNT);
        UnsignedInt next_shot_f = f1 + (oracle::splitmix32(rng) % 80u);
        UnsignedInt pre_atk_f = (seed % 4 == 0) ? (f1 + 10u + (oracle::splitmix32(rng) % 30u)) : 0u;
        UnsignedInt last_fire_f = frame0;
        ObjectID stream_id = static_cast<ObjectID>((seed % 3 == 0) ? (10 + (seed % 50)) : 0);
        Int max_shots = (seed % 5 == 0) ? static_cast<Int>(1 + (seed % 12)) : NO_MAX_SHOTS_LIMIT;
        Bool leech_act = (seed % 3 == 1) ? TRUE : FALSE;

        wpn->*access_member(WStatusTag()) = st_override_val;
        wpn->setPossibleNextShotFrame(next_shot_f);
        wpn->setPreAttackFinishedFrame(pre_atk_f);
        wpn->setLastReloadStartedFrame(f1);
        wpn->*access_member(WLastFireTag()) = last_fire_f;
        wpn->*access_member(WStreamIDTag()) = stream_id;
        wpn->setMaxShotCount(max_shots);
        wpn->setLeechRangeActive(leech_act);

        UnsignedInt query_f = f1 + (oracle::splitmix32(rng) % 60u);
        TheGameLogic->*access_member(GLFrameTag()) = query_f;
        int queried_status = static_cast<int>(wpn->getStatus());
        uint32_t pct_ready = oracle::fbits(wpn->getPercentReadyToFire());
        UnsignedInt rem_ammo = wpn->getRemainingAmmo();

        oracle::MemoryXfer w_save(XFER_SAVE); w_save.open("ws");
        oracle::call_snapshot_xfer(wpn, &w_save); w_save.close();
        XferCRC w_xcrc; w_xcrc.open("wc");
        oracle::call_snapshot_crc(wpn, &w_xcrc); w_xcrc.close();

        Weapon* w_loaded = TheWeaponStore->allocateNewWeapon(tmpl, PRIMARY_WEAPON);
        oracle::MemoryXfer w_load(XFER_LOAD); w_load.bytes = w_save.bytes; w_load.open("wl");
        oracle::call_snapshot_xfer(w_loaded, &w_load); w_load.close();
        oracle::MemoryXfer w_resave(XFER_SAVE); w_resave.open("wr");
        oracle::call_snapshot_xfer(w_loaded, &w_resave); w_resave.close();
        if (w_save.bytes != w_resave.bytes) {
            std::fprintf(stderr, "Weapon round-trip mismatch seed %u\n", seed);
            return 3;
        }
        deleteInstance(w_loaded);

        // PKT-400 WeaponTemplateSet & WeaponSet
        int ws_flag0 = static_cast<int>(oracle::splitmix32(rng) % WEAPONSET_COUNT);
        int ws_flag1 = static_cast<int>(oracle::splitmix32(rng) % WEAPONSET_COUNT);
        std::string w1_name = known_weapons[(seed + known_weapons.size() - 1) % known_weapons.size()];
        std::string w2_name = known_weapons[(seed * 3) % known_weapons.size()];
        bool has_sec = (seed % 3 != 2);
        bool has_ter = (seed % 4 == 0);
        int cmd_idx = static_cast<int>(oracle::splitmix32(rng) % 4);
        int ko_idx = static_cast<int>(oracle::splitmix32(rng) % 6);
        bool share_reload = (seed % 2 == 0);
        bool share_lock = (seed % 3 == 0);

        std::ostringstream wts_ss;
        wts_ss << "  Conditions = " << kWeaponSetFlagNames[ws_flag0] << " " << kWeaponSetFlagNames[ws_flag1] << "\n";
        wts_ss << "  Weapon = PRIMARY " << wname << "\n";
        if (has_sec) wts_ss << "  Weapon = SECONDARY " << w1_name << "\n";
        if (has_ter) wts_ss << "  Weapon = TERTIARY " << w2_name << "\n";
        wts_ss << "  AutoChooseSources = PRIMARY " << kCmdSourceNames[cmd_idx] << "\n";
        if (has_sec) wts_ss << "  PreferredAgainst = SECONDARY " << kKindOfSampleNames[ko_idx] << "\n";
        wts_ss << "  ShareWeaponReloadTime = " << (share_reload ? "Yes" : "No") << "\n";
        wts_ss << "  WeaponLockSharedAcrossSets = " << (share_lock ? "Yes" : "No") << "\n";
        wts_ss << "End\n";
        std::string wts_ini = wts_ss.str();

        alignas(ThingTemplate) unsigned char tt_buf[sizeof(ThingTemplate)]{};
        ThingTemplate* dummy_tt = reinterpret_cast<ThingTemplate*>(tt_buf);
        new (&(dummy_tt->*access_member(TTNameTag()))) AsciiString("DummyThing");

        WeaponTemplateSet wts;
        std::string wrapped_wts = "WeaponSet\n" + wts_ini;
        mem_ini.parse_blocks(wrapped_wts, INI_LOAD_OVERWRITE, [&](INI* ini, const char* tok) {
            if (std::strcmp(tok, "WeaponSet") == 0) wts.parseWeaponTemplateSet(ini, dummy_tt);
        });

        WeaponSet wset;
        wset.*access_member(WSCurTmplSetTag()) = &wts;
        auto& slots = wset.*access_member(WSWeaponsTag());
        UnsignedInt filled_mask = 0;
        Int total_anti = 0;
        DamageTypeFlags total_dmg;
        total_dmg.clear();
        Bool has_pitch = FALSE;
        Bool has_dmg = FALSE;
        for (int i = WEAPONSLOT_COUNT - 1; i >= PRIMARY_WEAPON; --i) {
            const WeaponTemplate* slot_wt = wts.getNth(static_cast<WeaponSlotType>(i));
            if (slot_wt) {
                Weapon* sw = TheWeaponStore->allocateNewWeapon(slot_wt, static_cast<WeaponSlotType>(i));
                if (i == PRIMARY_WEAPON) {
                    // Copy full state from wpn into slot 0 for realistic Xfer coverage
                    sw->*access_member(WStatusTag()) = wpn->*access_member(WStatusTag());
                    sw->*access_member(WAmmoTag()) = wpn->*access_member(WAmmoTag());
                    sw->setPossibleNextShotFrame(wpn->getPossibleNextShotFrame());
                    sw->setPreAttackFinishedFrame(wpn->getPreAttackFinishedFrame());
                    sw->setLastReloadStartedFrame(wpn->getLastReloadStartedFrame());
                    sw->*access_member(WLastFireTag()) = wpn->*access_member(WLastFireTag());
                    sw->*access_member(WSuspendFXTag()) = wpn->getSuspendFXFrame();
                    sw->*access_member(WStreamIDTag()) = wpn->*access_member(WStreamIDTag());
                    sw->setMaxShotCount(wpn->getMaxShotCount());
                    sw->*access_member(WCurBarrelTag()) = wpn->*access_member(WCurBarrelTag());
                    sw->*access_member(WNumBarrelTag()) = wpn->*access_member(WNumBarrelTag());
                    sw->*access_member(WScatterUnusedTag()) = wpn->*access_member(WScatterUnusedTag());
                    sw->setLeechRangeActive(wpn->hasLeechRange());
                }
                slots[i] = sw;
                filled_mask |= (1u << i);
                total_anti |= sw->getAntiMask();
                total_dmg.set(sw->getDamageType(), 1);
                if (sw->isPitchLimited()) has_pitch = TRUE;
                if (sw->isDamageWeapon()) has_dmg = TRUE;
            }
        }
        wset.*access_member(WSFilledMaskTag()) = filled_mask;
        wset.*access_member(WSTotalAntiTag()) = total_anti;
        wset.*access_member(WSTotalDmgTag()) = total_dmg;
        wset.*access_member(WSHasPitchTag()) = has_pitch;
        wset.*access_member(WSHasDmgTag()) = has_dmg;

        int lock_op = static_cast<int>(seed % 4);
        WeaponSlotType lock_slot = has_sec ? SECONDARY_WEAPON : PRIMARY_WEAPON;
        if (lock_op == 1) {
            wset.setWeaponLock(lock_slot, LOCKED_TEMPORARILY);
        } else if (lock_op == 2) {
            wset.setWeaponLock(lock_slot, LOCKED_PERMANENTLY);
        } else if (lock_op == 3) {
            wset.setWeaponLock(lock_slot, LOCKED_TEMPORARILY);
            wset.releaseWeaponLock(LOCKED_TEMPORARILY);
        }

        UnsignedInt most_ready = wset.getMostPercentReadyToFireAnyWeapon();
        int out_of_ammo = wset.isOutOfAmmo() ? 1 : 0;
        WeaponSetConditionType ws_cond = static_cast<WeaponSetConditionType>(seed % WSF_COUNT);
        ModelConditionFlags mc_flags = WeaponSet::getModelConditionForWeaponSlot(wslot, ws_cond);
        oracle::MemoryXfer mc_save(XFER_SAVE); mc_save.open("mc");
        mc_flags.xfer(&mc_save); mc_save.close();

        oracle::MemoryXfer ws_save(XFER_SAVE); ws_save.open("wss");
        oracle::call_snapshot_xfer(&wset, &ws_save); ws_save.close();

        (dummy_tt->*access_member(TTNameTag())).~AsciiString();

        out << "    {\"seed\":" << seed
            << ",\"ini\":\"" << oracle::json_escape(w_ini) << "\""
            << ",\"ovr\":\"" << oracle::json_escape(ovr_ini) << "\""
            << ",\"wts_ini\":\"" << oracle::json_escape(wrapped_wts) << "\""
            << ",\"rand_seed\":" << rand_seed
            << ",\"bflags\":" << bflags
            << ",\"frame0\":" << frame0
            << ",\"f1\":" << f1
            << ",\"query_f\":" << query_f
            << ",\"wslot\":" << static_cast<int>(wslot)
            << ",\"pcts\":[" << oracle::fbits(pct1) << "," << oracle::fbits(pct2) << "," << (allow_red ? 1 : 0) << "]"
            << ",\"wstate\":[" << static_cast<int>(st_override_val) << "," << next_shot_f << "," << pre_atk_f
            << "," << last_fire_f << "," << static_cast<int>(stream_id) << "," << max_shots << "," << (leech_act ? 1 : 0) << "]"
            << ",\"lock_op\":[" << lock_op << "," << static_cast<int>(lock_slot) << "," << static_cast<int>(ws_cond) << "]"
            << ",\"bf\":[" << bf_bits[0] << "," << bf_bits[1] << "," << bf_bits[2] << "," << bf_bits[3] << "," << bf_bits[4] << "]"
            << ",\"t_stats\":[" << atk_rng << "," << unmod_rng << "," << min_rng << "," << prim_dmg << "," << prim_rad
            << "," << sec_dmg << "," << sec_rad << "]"
            << ",\"t_ints\":[" << shot_delay << "," << clip_reload << "," << pre_atk_delay << "," << is_contact
            << "," << is_dmg_wpn << "," << is_pitch_lim << "]"
            << ",\"rand_crc\":" << rand_crc_after_delay
            << ",\"w_q\":[" << queried_status << "," << pct_ready << "," << rem_ammo << "]"
            << ",\"wxlen\":" << w_save.bytes.size()
            << ",\"wxb_crc\":" << w_save.bytes_crc()
            << ",\"wxcrc\":" << w_xcrc.getCRC()
            << ",\"ws_q\":[" << static_cast<int>(wset.getCurWeaponSlot()) << "," << (wset.isCurWeaponLocked() ? 1 : 0)
            << "," << static_cast<int>(filled_mask) << "," << total_anti << "," << (has_pitch ? 1 : 0)
            << "," << (has_dmg ? 1 : 0) << "," << most_ready << "," << out_of_ammo
            << "," << static_cast<int>(wts.getNthCommandSourceMask(PRIMARY_WEAPON)) << "," << (wts.isSharedReloadTime() ? 1 : 0)
            << "," << (wts.isWeaponLockSharedAcrossSets() ? 1 : 0) << "]"
            << ",\"mc_crc\":" << mc_save.bytes_crc()
            << ",\"wsxlen\":" << ws_save.bytes.size()
            << ",\"wsxcrc\":" << ws_save.bytes_crc();

        if (seed == 0) {
            out << ",\"s0_tmpl\":{"
                << "\"min_delay\":" << tmpl->*access_member(WTMinDelayTag())
                << ",\"max_delay\":" << tmpl->*access_member(WTMaxDelayTag())
                << ",\"anti\":" << tmpl->getAntiMask()
                << ",\"affects\":" << tmpl->getAffectsMask()
                << ",\"collide\":" << tmpl->getProjectileCollideMask()
                << ",\"clip_size\":" << tmpl->getClipSize()
                << ",\"shots_per_barrel\":" << tmpl->getShotsPerBarrel()
                << ",\"scatter_cnt\":" << tmpl->getScatterTargetsVector().size()
                << "}"
                << ",\"s0_wpn\":{"
                << "\"ammo\":" << wpn->*access_member(WAmmoTag())
                << ",\"suspend_fx\":" << wpn->getSuspendFXFrame()
                << ",\"num_shots_barrel\":" << wpn->*access_member(WNumBarrelTag())
                << ",\"w_xhex\":\"" << w_save.hex() << "\""
                << "}"
                << ",\"s0_wset\":{"
                << "\"ws_xhex\":\"" << ws_save.hex() << "\""
                << ",\"mc_xhex\":\"" << mc_save.hex() << "\""
                << "}";
        }
        out << "}" << ((seed + 1u < 1000u) ? "," : "") << "\n";

        deleteInstance(wpn);
    }

    out << "  ]\n}\n";
    oracle::write_file_atomic(out_path, out.str());
    shutdownMemoryManager();
    _exit(0);
}
