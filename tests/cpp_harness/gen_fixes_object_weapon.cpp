// Vectors for four faithfulness fixes, from the unmodified reference:
//   parse   : `WeaponTemplate` angle, velocity and duration fields (`WeaponStore` INI parsing)
//   crush   : `Object::canCrushOrSquish(other, TEST_CRUSH_ONLY)`
//   disable : `Object::setDisabled/setDisabledUntil/clearDisabled/checkDisabledStatus`, with the
//             special power pauses, the energy adjustment and the academy special power count
//   bonus   : `Object::setWeaponBonusCondition/clearWeaponBonusCondition` retiming the weapons
//
// Not covered: the squish tests of `canCrushOrSquish`; `DISABLED_UNMANNED` on a non-drone and the
// rider propagation of `setDisabledUntil` (both unported); the global weapon bonus set.
//
// usage: gen_fixes_object_weapon <out.json>

#include "oracle_wp51_world.h"

#include "Common/AcademyStats.h"
#include "Common/Energy.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/SpecialPowerModule.h"

#include <cstdio>
#include <cstring>
#include <string>
#include <vector>

struct FxDisabledTag { using type = UnsignedInt (Object::*)[DISABLED_COUNT]; };
template struct MemberAccess<FxDisabledTag, &Object::m_disabledTillFrame>;
inline UnsignedInt (Object::*access_member(FxDisabledTag))[DISABLED_COUNT];

#define FX_ROB(TAG, CLASS, TYPE, MEMBER)                \
    struct TAG { using type = TYPE CLASS::*; };         \
    template struct MemberAccess<TAG, &CLASS::MEMBER>;  \
    inline TYPE CLASS::*access_member(TAG);

FX_ROB(FxStatus, Weapon, WeaponStatus, m_status)
FX_ROB(FxAmmo, Weapon, UnsignedInt, m_ammoInClip)
FX_ROB(FxFire, Weapon, UnsignedInt, m_whenWeCanFireAgain)
FX_ROB(FxPre, Weapon, UnsignedInt, m_whenPreAttackFinished)
FX_ROB(FxReload, Weapon, UnsignedInt, m_whenLastReloadStarted)
FX_ROB(FxAim, WeaponTemplate, Real, m_aimDelta)
FX_ROB(FxRecoil, WeaponTemplate, Real, m_weaponRecoil)
FX_ROB(FxMinPitch, WeaponTemplate, Real, m_minTargetPitch)
FX_ROB(FxMaxPitch, WeaponTemplate, Real, m_maxTargetPitch)
FX_ROB(FxDamageAngle, WeaponTemplate, Real, m_radiusDamageAngle)
FX_ROB(FxSpeed, WeaponTemplate, Real, m_weaponSpeed)
FX_ROB(FxMinSpeed, WeaponTemplate, Real, m_minWeaponSpeed)
FX_ROB(FxClipReload, WeaponTemplate, Int, m_clipReloadTime)
FX_ROB(FxMinDelay, WeaponTemplate, Int, m_minDelayBetweenShots)
FX_ROB(FxMaxDelay, WeaponTemplate, Int, m_maxDelayBetweenShots)
FX_ROB(FxCoast, WeaponTemplate, UnsignedInt, m_continuousFireCoastFrames)
FX_ROB(FxIdle, WeaponTemplate, UnsignedInt, m_autoReloadWhenIdleFrames)
FX_ROB(FxPreDelay, WeaponTemplate, Int, m_preAttackDelay)
FX_ROB(FxSuspend, WeaponTemplate, UnsignedInt, m_suspendFXDelay)
FX_ROB(FxPowersUsed, AcademyStats, UnsignedInt, m_specialPowersUsed)

static uint32_t g_state = 0x7f1c5eedu;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}

static std::string u(uint32_t v) { return std::to_string(v); }
static std::string bits(Real r) {
    uint32_t b = 0;
    std::memcpy(&b, &r, sizeof b);
    return u(b);
}

// ---------------------------------------------------------------- fixtures

static const int kLevels[5] = {0, 1, 2, 3, 255};

static std::string fixture_ini() {
    std::string s;
    for (int c = 0; c < 5; ++c) {
        for (int v = 0; v < 5; ++v) {
            s += "Object FX_Crush" + std::to_string(c) + std::to_string(v) +
                 "\n  CrusherLevel = " + std::to_string(kLevels[c]) +
                 "\n  CrushableLevel = " + std::to_string(kLevels[v]) +
                 "\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\nEnd\n";
        }
    }
    s += "SpecialPower FX_PowerSuper\n  Enum = SPECIAL_DAISY_CUTTER\n  ReloadTime = 3000\n"
         "  AcademyClassify = ACT_SUPERPOWER\nEnd\n"
         "SpecialPower FX_PowerOther\n  Enum = SPECIAL_CARPET_BOMB\n  ReloadTime = 1300\n"
         "  AcademyClassify = ACT_UPGRADE_RADAR\nEnd\n";
    s += "Object FX_Disable\n  KindOf = STRUCTURE DRONE\n  EnergyProduction = 10\n"
         "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
         "  Behavior = SpecialAbility ModuleTag_A\n    SpecialPowerTemplate = FX_PowerSuper\n  End\n"
         "  Behavior = SpecialAbility ModuleTag_B\n    SpecialPowerTemplate = FX_PowerOther\n  End\n"
         "End\n";
    s += "Weapon FX_WeaponA\n  ClipSize = 4\n  ClipReloadTime = 2000\n  DelayBetweenShots = 500\n"
         "  WeaponBonus = VETERAN RATE_OF_FIRE 150%\n  WeaponBonus = ELITE RATE_OF_FIRE 200%\n"
         "  WeaponBonus = HERO RATE_OF_FIRE 300%\n  WeaponBonus = PLAYER_UPGRADE DAMAGE 200%\nEnd\n"
         "Weapon FX_WeaponB\n  ClipSize = 0\n  ClipReloadTime = 700\n  DelayBetweenShots = Min:300 Max:900\n"
         "  WeaponBonus = ENTHUSIASTIC RATE_OF_FIRE 125%\n  WeaponBonus = VETERAN RATE_OF_FIRE 110%\nEnd\n"
         "Weapon FX_WeaponC\n  ClipSize = 2\n  ClipReloadTime = 1000\n  DelayBetweenShots = Min:100 Max:400\n"
         "  WeaponBonus = GARRISONED RATE_OF_FIRE 133%\n  WeaponBonus = HERO RATE_OF_FIRE 70%\nEnd\n";
    const std::string rider_body =
        "  KindOf = INFANTRY\n  TransportSlotCount = 1\n"
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n";
    s += "Object FX_Gunner0\n" + rider_body +
         "  WeaponSet\n    Conditions = None\n    Weapon = PRIMARY FX_WeaponA\n"
         "    Weapon = SECONDARY FX_WeaponB\n    Weapon = TERTIARY FX_WeaponC\n  End\nEnd\n";
    s += "Object FX_Gunner1\n" + rider_body +
         "  WeaponSet\n    Conditions = None\n    Weapon = PRIMARY FX_WeaponA\n"
         "    Weapon = SECONDARY FX_WeaponB\n    Weapon = TERTIARY FX_WeaponC\n"
         "    ShareWeaponReloadTime = Yes\n  End\nEnd\n";
    s += "Object FX_Gunner2\n" + rider_body +
         "  WeaponSet\n    Conditions = None\n    Weapon = PRIMARY FX_WeaponC\n"
         "    Weapon = TERTIARY FX_WeaponB\n  End\nEnd\n";
    for (int pass = 0; pass < 2; ++pass) {
        s += "Object FX_Fort" + std::to_string(pass) +
             "\n  KindOf = STRUCTURE\n  GeometryMajorRadius = 20\n  GeometryHeight = 20\n"
             "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 1000\n  End\n"
             "  Behavior = GarrisonContain ModuleTag_Contain\n    ContainMax = 5\n"
             "    WeaponBonusPassedToPassengers = " + (pass ? "Yes" : "No") + "\n  End\nEnd\n";
    }
    return s;
}

// ---------------------------------------------------------------- weapon parsing

static std::string number(bool fraction_ok) {
    static const char* const k[] = {"0", "1", "33", "34", "99", "100", "180", "360", "999", "1000", "1001",
                                    "2000", "123456", "-1", "-90"};
    if (fraction_ok && pick(4) == 0) return std::to_string(pick(2000)) + "." + std::to_string(pick(1000));
    if (pick(2) == 0) return std::to_string(pick(100000));
    return k[pick(sizeof k / sizeof *k)];
}

static std::string parse_block(oracle_wp51::World& w, int i) {
    static const char* const kReal[] = {"AcceptableAimDelta", "WeaponRecoil", "MinTargetPitch", "MaxTargetPitch",
                                        "RadiusDamageAngle", "WeaponSpeed", "MinWeaponSpeed"};
    static const char* const kDuration[] = {"ClipReloadTime", "ContinuousFireCoast", "AutoReloadWhenIdle",
                                            "PreAttackDelay", "SuspendFXDelay"};
    std::vector<std::string> lines;
    for (const char* f : kReal)
        if (pick(3) != 0) lines.push_back(std::string(f) + " = " + number(true));
    for (const char* f : kDuration)
        if (pick(3) != 0) lines.push_back(std::string(f) + " = " + number(true));
    const uint32_t shape = pick(5);
    const std::string a = number(false), b = number(false);
    if (shape == 1) lines.push_back("DelayBetweenShots = " + a);
    if (shape == 2) lines.push_back("DelayBetweenShots = Min:" + a + " Max:" + b);
    if (shape == 3) lines.push_back("DelayBetweenShots = min : " + a + " MAX : " + b);
    // A second label other than Max leaves the maximum equal to the minimum.
    if (shape == 4) lines.push_back("DelayBetweenShots = Min:" + a + " Top:" + b);

    const std::string name = "FX_Parse" + std::to_string(i);
    std::string ini = "Weapon " + name + "\n", joined;
    for (size_t n = 0; n < lines.size(); ++n) {
        ini += "  " + lines[n] + "\n";
        joined += (n ? "|" : "") + lines[n];
    }
    ini += "End\n";
    w.load_ini(ini);
    const WeaponTemplate* t = TheWeaponStore->findWeaponTemplate(name.c_str());
    if (!t) {
        std::fprintf(stderr, "missing weapon %s\n", name.c_str());
        std::exit(1);
    }
    WeaponTemplate& m = *const_cast<WeaponTemplate*>(t);
    std::string out;
    for (Real r : {m.*access_member(FxAim()), m.*access_member(FxRecoil()), m.*access_member(FxMinPitch()),
                   m.*access_member(FxMaxPitch()), m.*access_member(FxDamageAngle()),
                   m.*access_member(FxSpeed()), m.*access_member(FxMinSpeed())})
        out += bits(r) + " ";
    for (uint32_t v : {uint32_t(m.*access_member(FxClipReload())), uint32_t(m.*access_member(FxCoast())),
                       uint32_t(m.*access_member(FxIdle())), uint32_t(m.*access_member(FxPreDelay())),
                       uint32_t(m.*access_member(FxSuspend())), uint32_t(m.*access_member(FxMinDelay())),
                       uint32_t(m.*access_member(FxMaxDelay()))})
        out += u(v) + " ";
    out.pop_back();
    return "{\"ini\":\"" + joined + "\",\"out\":\"" + out + "\"}";
}

// ---------------------------------------------------------------- crush

static std::string crush_case(oracle_wp51::World& w) {
    const uint32_t ca = pick(5), va = pick(5), cb = pick(5), vb = pick(5);
    const uint32_t team_b = pick(4) == 0 ? 1 : 2;
    const uint32_t rel_ab = pick(3), rel_ba = pick(3);
    // Bits 0 and 1: unmanned. Bits 2 and 3: undetected defector.
    uint32_t flags = 0;
    for (int k = 0; k < 4; ++k)
        if (pick(k < 2 ? 6 : 8) == 0) flags |= 1u << k;

    Player* pa = ThePlayerList->getNthPlayer(1);
    Player* pb = ThePlayerList->getNthPlayer(2);
    pa->setPlayerRelationship(pb, static_cast<Relationship>(rel_ab));
    pb->setPlayerRelationship(pa, static_cast<Relationship>(rel_ba));

    const std::string na = "FX_Crush" + std::to_string(ca) + std::to_string(va);
    const std::string nb = "FX_Crush" + std::to_string(cb) + std::to_string(vb);
    Object* a = w.spawn_object(na.c_str(), w.team_a);
    Object* b = w.spawn_object(nb.c_str(), team_b == 1 ? w.team_a : w.team_b);
    if (flags & 1) a->setDisabled(DISABLED_UNMANNED);
    if (flags & 2) b->setDisabled(DISABLED_UNMANNED);
    if (flags & 4) a->friend_setUndetectedDefector(TRUE);
    if (flags & 8) b->friend_setUndetectedDefector(TRUE);
    const int ab = a->canCrushOrSquish(b, TEST_CRUSH_ONLY) ? 1 : 0;
    const int ba = b->canCrushOrSquish(a, TEST_CRUSH_ONLY) ? 1 : 0;
    TheGameLogic->destroyObject(a);
    TheGameLogic->destroyObject(b);
    w.flush_destroy_list();
    return "{\"in\":\"" + u(ca) + " " + u(va) + " " + u(cb) + " " + u(vb) + " " + u(team_b) + " " + u(rel_ab) +
           " " + u(rel_ba) + " " + u(flags) + "\",\"out\":\"" + std::to_string(ab) + std::to_string(ba) + "\"}";
}

// ---------------------------------------------------------------- disable

static const DisabledType kTypes[] = {DISABLED_DEFAULT, DISABLED_HACKED,       DISABLED_EMP,
                                      DISABLED_HELD,    DISABLED_PARALYZED,    DISABLED_UNDERPOWERED,
                                      DISABLED_SUBDUED, DISABLED_SCRIPT_DISABLED};

static std::string disable_row(Object* o, Int base_power, UnsignedInt base_used, int ret) {
    Player* p = o->getControllingPlayer();
    std::string r;
    UnsignedInt mask = 0;
    for (int i = 0; i < DISABLED_COUNT; ++i)
        if (o->getDisabledFlags().test(i)) mask |= 1u << i;
    r += u(mask) + " ";
    UnsignedInt(&till)[DISABLED_COUNT] = o->*access_member(FxDisabledTag());
    for (DisabledType t : kTypes) r += u(till[t]) + ",";
    r += " " + std::to_string(p->getEnergy()->getProduction() - base_power);
    r += " " + u(p->getAcademyStats()->*access_member(FxPowersUsed()) - base_used);
    r += " " + std::to_string(ret);
    for (BehaviorModule** m = o->getBehaviorModules(); *m; ++m) {
        if (dynamic_cast<SpecialPowerModule*>(*m)) r += " " + oracle_wp51::World::snapshot_save_hex(*m);
    }
    return r;
}

static std::string disable_case(oracle_wp51::World& w) {
    UnsignedInt frame = 100 + pick(5000);
    w.set_frame(frame);
    Player* p = ThePlayerList->getNthPlayer(1);
    const Int base_power = p->getEnergy()->getProduction();
    const UnsignedInt base_used = p->getAcademyStats()->*access_member(FxPowersUsed());
    Object* o = w.spawn_object("FX_Disable", w.team_a);
    const SpecialPowerTemplate* tmpl[2] = {TheSpecialPowerStore->findSpecialPowerTemplate("FX_PowerSuper"),
                                           TheSpecialPowerStore->findSpecialPowerTemplate("FX_PowerOther")};
    std::string rows = "\"" + disable_row(o, base_power, base_used, 0) + "\"";
    std::string ops;
    const int n = 4 + static_cast<int>(pick(20));
    // A few types per sequence, so that sets and clears often meet.
    const uint32_t palette[3] = {pick(8), pick(8), pick(8)};
    for (int k = 0; k < n; ++k) {
        static const uint32_t kCodes[] = {0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 4, 4, 5};
        const uint32_t code = kCodes[pick(sizeof kCodes / sizeof *kCodes)];
        uint32_t type = pick(4) == 0 ? pick(8) : palette[pick(3)];
        uint32_t arg = 0;
        int ret = 0;
        if (code == 0) {
            o->setDisabled(kTypes[type]);
        } else if (code == 1) {
            const uint32_t shape = pick(6);
            arg = shape == 0 ? frame - 1 - pick(5) : shape == 1 ? frame : shape == 2 ? 0 : frame + 1 + pick(120);
            o->setDisabledUntil(kTypes[type], arg);
        } else if (code == 2) {
            ret = o->clearDisabled(kTypes[type]) ? 1 : 0;
        } else if (code == 3) {
            // The `GameLogic::update` walk.
            arg = 1 + pick(100);
            frame += arg;
            w.set_frame(frame);
            if (o->isDisabled()) o->checkDisabledStatus();
        } else if (code == 4) {
            // Time passes without the walk, leaving expired types set.
            arg = 1 + pick(120);
            frame += arg;
            w.set_frame(frame);
        } else {
            type = pick(2);
            o->getSpecialPowerModule(tmpl[type])->doSpecialPower(0);
        }
        ops += (k ? "," : "") + std::string("\"") + u(code) + " " + u(type) + " " + u(arg) + "\"";
        rows += ",\"" + disable_row(o, base_power, base_used, ret) + "\"";
    }
    TheGameLogic->destroyObject(o);
    w.flush_destroy_list();
    return "\"ops\":[" + ops + "],\"rows\":[" + rows + "]}";
}

// ---------------------------------------------------------------- weapon bonus

static const WeaponBonusConditionType kConds[] = {
    WEAPONBONUSCONDITION_GARRISONED,   WEAPONBONUSCONDITION_PLAYER_UPGRADE, WEAPONBONUSCONDITION_ENTHUSIASTIC,
    WEAPONBONUSCONDITION_VETERAN,      WEAPONBONUSCONDITION_ELITE,          WEAPONBONUSCONDITION_HERO};

static std::string bonus_row(Object* gunner, Object* fort) {
    std::string r = u(gunner->getWeaponBonusCondition()) + " " + u(fort->getWeaponBonusCondition()) + " " +
                    u(GetGameLogicRandomSeedCRC()) + " " + (gunner->getContainedBy() ? "1" : "0");
    for (int s = 0; s < WEAPONSLOT_COUNT; ++s) {
        Weapon* wp = gunner->getWeaponInWeaponSlot(static_cast<WeaponSlotType>(s));
        if (!wp) {
            r += " -";
            continue;
        }
        r += " " + u(static_cast<uint32_t>(wp->*access_member(FxStatus()))) + "," + u(wp->*access_member(FxAmmo())) +
             "," + u(wp->*access_member(FxFire())) + "," + u(wp->*access_member(FxPre())) + "," +
             u(wp->*access_member(FxReload()));
    }
    return r;
}

static std::string bonus_case(oracle_wp51::World& w) {
    const uint32_t kind = pick(3), pass = pick(2), seed = pick(100000);
    UnsignedInt frame = 100 + pick(5000);
    const UnsignedInt start = frame;
    w.set_frame(frame);
    Object* gunner = w.spawn_object(("FX_Gunner" + std::to_string(kind)).c_str(), w.team_a);
    Object* fort = w.spawn_object(("FX_Fort" + std::to_string(pass)).c_str(), w.team_a);
    InitRandom(seed);
    std::string rows = "\"" + bonus_row(gunner, fort) + "\"";
    std::string ops;
    const int n = 4 + static_cast<int>(pick(20));
    for (int k = 0; k < n; ++k) {
        static const uint32_t kCodes[] = {0, 0, 0, 1, 1, 2, 3, 3, 3, 3, 4, 5, 6};
        uint32_t code = kCodes[pick(sizeof kCodes / sizeof *kCodes)];
        uint32_t a = 0, b = 0, c = 0, d = 0, e = 0;
        if (code <= 2) {
            a = pick(6);
            if (code == 2) b = pick(2);  // 1 clears on the fort
            if (code == 0) gunner->setWeaponBonusCondition(kConds[a]);
            else if (code == 1) gunner->clearWeaponBonusCondition(kConds[a]);
            else if (b) fort->clearWeaponBonusCondition(kConds[a]);
            else fort->setWeaponBonusCondition(kConds[a]);
        } else if (code == 3) {
            // Put a weapon in some state.
            a = pick(3);
            Weapon* wp = gunner->getWeaponInWeaponSlot(static_cast<WeaponSlotType>(a));
            b = pick(5);                                         // status
            c = pick(5);                                         // ammo
            d = pick(4) == 0 ? frame - pick(50) : frame + pick(200);  // when it can fire again
            e = pick(5) == 0 ? frame + 1 + pick(50) : 0;         // pre attack
            if (wp) {
                wp->*access_member(FxStatus()) = static_cast<WeaponStatus>(b);
                wp->*access_member(FxAmmo()) = c;
                wp->setPossibleNextShotFrame(d);
                wp->setPreAttackFinishedFrame(e);
            }
        } else if (code == 4) {
            a = 1 + pick(60);
            frame += a;
            w.set_frame(frame);
        } else if (code == 5) {
            if (!gunner->getContainedBy()) fort->getContain()->addToContain(gunner);
        } else {
            if (gunner->getContainedBy()) fort->getContain()->removeFromContain(gunner, FALSE);
        }
        ops += (k ? "," : "") + std::string("\"") + u(code) + " " + u(a) + " " + u(b) + " " + u(c) + " " + u(d) +
               " " + u(e) + "\"";
        rows += ",\"" + bonus_row(gunner, fort) + "\"";
    }
    if (gunner->getContainedBy()) fort->getContain()->removeFromContain(gunner, FALSE);
    TheGameLogic->destroyObject(gunner);
    TheGameLogic->destroyObject(fort);
    w.flush_destroy_list();
    return "{\"setup\":\"" + u(kind) + " " + u(pass) + " " + u(seed) + " " + u(start) + "\",\"ops\":[" + ops +
           "],\"rows\":[" + rows + "]}";
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_fixes_object_weapon <out.json>\n");
        return 2;
    }
    oracle_wp51::World w;
    w.load_ini(fixture_ini());
    w.init_with_neutral_player();

    FILE* out = std::fopen(argv[1], "w");
    if (!out) return 1;
    std::fprintf(out, "{\"parse\":[");
    for (int i = 0; i < 1000; ++i) std::fprintf(out, "%s\n%s", i ? "," : "", parse_block(w, i).c_str());
    std::fprintf(out, "\n],\"crush\":[");
    for (int i = 0; i < 1500; ++i) std::fprintf(out, "%s\n%s", i ? "," : "", crush_case(w).c_str());
    Player* pa = ThePlayerList->getNthPlayer(1);
    Player* pb = ThePlayerList->getNthPlayer(2);
    pa->setPlayerRelationship(pb, ENEMIES);
    pb->setPlayerRelationship(pa, ENEMIES);
    std::fprintf(out, "\n],\"disable\":[");
    for (int i = 0; i < 1000; ++i) {
        // The sequence draws its own start frame; peek at it for the line header.
        const uint32_t saved = g_state;
        const UnsignedInt start = 100 + pick(5000);
        g_state = saved;
        std::fprintf(out, "%s\n{\"start\":%u,%s", i ? "," : "", start, disable_case(w).c_str());
    }
    std::fprintf(out, "\n],\"bonus\":[");
    for (int i = 0; i < 1000; ++i) std::fprintf(out, "%s\n%s", i ? "," : "", bonus_case(w).c_str());
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    return 0;
}
