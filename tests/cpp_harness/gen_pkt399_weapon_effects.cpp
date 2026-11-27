// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for the effects of a fired weapon (PKT-399): `fireWeaponTemplate`,
// `dealDamageInternal`, the delayed damage queue and the historic bonus of `WeaponStore`,
// run on the unmodified `Weapon.cpp` in a small real world with real bodies, armor, die
// modules, physics, `DumbProjectileBehavior`, `ProjectileStreamUpdate` and
// `SpecialPowerCompletionDie` (players A and B enemies, a neutral player, stepped terrain).
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt399_weapon_effects.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/PKT399_WeaponEffects.json
//
// Built also with `--opt O0`, `--opt O2` and `--sanitize`; the outputs are identical.
// `WEAPON_DUMP=<vector>` prints that vector's rows.
//
// Observed after every operation: the logic seed CRC; every object's flags, position,
// orientation, health, experience, body and physics save bytes, projectile, die module and
// stream state; the delayed damage queue; the historic damage lists; the OCL calls; and the
// special power completions.
//
// Weapons 30 and 31 name a projectile that does not exist. Every eighth vector hammers one
// historic weapon at points that include two exactly one radius apart.
//
// Not covered: bridges (`getBridgeAttackPoints`), a sneaky targeting offset (JetAIUpdate),
// countermeasures (they need an AI), launch bones and turrets, containers around the
// launcher, an unknown laser template, a missing source object for a
// weapon with `ClipSize` set that earns a historic bonus (null dereference), a missing
// source for a `SUICIDE` weapon without radius (null dereference), a projectile with no
// detonation weapon, `DAMAGE_DEPLOY`, `DAMAGE_DISARM`, assistance requests, FX lists,
// sounds, the client laser beam, loading a save, and veterancy level changes during a vector.

#include "oracle_wp51_world.h"

#include "Common/Player.h"
#include "Common/RandomValue.h"
#include "Common/Team.h"
#include "GameClient/Drawable.h"
#include "GameLogic/ExperienceTracker.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/ActiveBody.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/DieModule.h"
#include "GameLogic/Module/DumbProjectileBehavior.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Module/ProjectileStreamUpdate.h"
#include "GameLogic/Module/SpecialPowerCompletionDie.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectCreationList.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/ScriptEngine.h"
#include "GameLogic/Weapon.h"

#include <algorithm>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

// Private state of the reference that the rows record.
template <auto M> struct StealDdi { friend auto steal_ddi() { return M; } };
auto steal_ddi();
template struct StealDdi<&WeaponStore::m_weaponDDI>;
template <auto M> struct StealHistory { friend auto steal_history() { return M; } };
auto steal_history();
template struct StealHistory<&WeaponTemplate::m_historicDamage>;
template <auto M> struct StealFinished { friend auto steal_finished() { return M; } };
auto steal_finished();
template struct StealFinished<&ScriptEngine::m_finishedSpecialPowers>;
template <auto M> struct StealLevel { friend auto steal_level() { return M; } };
auto steal_level();
template struct StealLevel<&ExperienceTracker::m_currentLevel>;

static void die(const char* what) {
    std::fprintf(stderr, "oracle error: %s\n", what);
    std::exit(1);
}

static uint32_t g_state = 0x39900399u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}
static bool chance(uint32_t percent) { return pick(100) < percent; }
template <size_t N> static const char* choose(const char* const (&list)[N]) { return list[pick(N)]; }

static uint32_t bits(Real r) {
    uint32_t u;
    std::memcpy(&u, &r, 4);
    return u;
}
static uint64_t fnv(const std::string& s) {
    uint64_t h = 0xcbf29ce484222325ull;
    for (unsigned char c : s) h = (h ^ c) * 0x100000001b3ull;
    return h;
}

// ---------------------------------------------------------------------------------------------
// Terrain, OCL spy
// ---------------------------------------------------------------------------------------------
class StepTerrain : public oracle_wp51::OracleTerrainLogic {
public:
    static Real height(Real x, Real y) {
        return static_cast<Real>(static_cast<int>(x / 40.0f) % 3) * 4.0f +
               static_cast<Real>(static_cast<int>(y / 60.0f) % 4) * 3.0f;
    }
    Real getGroundHeight(Real x, Real y, Coord3D* normal = nullptr) const override {
        if (normal) { normal->x = 0.0f; normal->y = 0.0f; normal->z = 1.0f; }
        return height(x, y);
    }
};

static std::ostringstream g_ocl;
static const int kOcls = 6;
class SpyNugget : public ObjectCreationNugget {
public:
    int list = 0;
    Object* create(const Object*, const Coord3D*, const Coord3D*, Real, UnsignedInt) const override {
        die("positional OCL call");
        return nullptr;
    }
    Object* create(const Object* primary, const Object* secondary, UnsignedInt lifetime) const override {
        if (secondary || lifetime) die("unexpected OCL arguments");
        const Coord3D* p = primary->getPosition();
        g_ocl << 'X' << list << ',' << static_cast<uint32_t>(primary->getID()) << ',' << bits(p->x) << ','
              << bits(p->y) << ',' << bits(p->z) << ';';
        return nullptr;
    }
protected:
    MemoryPool* getObjectMemoryPool() override { return nullptr; }
};
static SpyNugget g_nugget[kOcls];

// ---------------------------------------------------------------------------------------------
// Fixture
// ---------------------------------------------------------------------------------------------
static const char* kArmorIni =
    "Armor E_Plain\n  Armor = Default 100%\nEnd\n"
    "Armor E_Half\n  Armor = Default 50%\n  Armor = FLAME 150%\n  Armor = SMALL_ARMS 37%\nEnd\n"
    "Armor E_Odd\n  Armor = Default 120%\n  Armor = EXPLOSION 0%\n  Armor = SNIPER 200%\nEnd\n";

static const int kWeapons = 72;
// The first weapons are plain, so that bonus weapons and death weapons can be them.
static const int kPlain = 8;
static const int kProjectiles = 4;  // E_Shell0..2 fly, E_Shell3 has no projectile update

static std::string weapon_name(int i) { return "E_W" + std::to_string(i); }

static std::string flags(const char* const* names, int count, uint32_t percent) {
    std::string s;
    for (int i = 0; i < count; ++i)
        if (chance(percent)) s += std::string(" ") + names[i];
    return s;
}

static std::string weapon_ini() {
    static const char* const kDamage[] = {"0.5", "7", "25", "40", "80", "200", "1000"};
    static const char* const kRadius[] = {"0", "0", "5", "15", "30", "60", "200"};
    static const char* const kType[] = {"EXPLOSION", "EXPLOSION", "ARMOR_PIERCING", "SMALL_ARMS",
                                        "FLAME", "SNIPER", "UNRESISTABLE", "PARTICLE_BEAM"};
    static const char* const kDeath[] = {"NORMAL", "BURNED", "EXPLODED", "TOPPLED"};
    static const char* const kSlow[] = {"3", "5", "13", "40", "90", "400"};
    static const char* const kRange[] = {"20", "60", "100", "200", "1000"};
    static const char* const kMin[] = {"5", "20", "40", "70"};
    static const char* const kAffects[] = {"SELF", "ALLIES", "ENEMIES", "NEUTRALS", "SUICIDE",
                                           "NOT_SIMILAR", "NOT_AIRBORNE"};
    static const char* const kCollides[] = {"ALLIES", "ENEMIES", "STRUCTURES", "SHRUBBERY",
                                            "PROJECTILES", "WALLS", "SMALL_MISSILES",
                                            "BALLISTIC_MISSILES", "CONTROLLED_STRUCTURES"};
    static const char* const kScatter[] = {"1", "7.5", "30", "100"};
    static const char* const kAngle[] = {"20", "60", "90", "135", "179"};
    static const char* const kShock[] = {"5", "40", "200"};
    static const char* const kBonus[] = {"GARRISONED DAMAGE 125%", "GARRISONED RADIUS 150%",
                                         "HORDE RANGE 50%", "HORDE DAMAGE 300%",
                                         "CONTINUOUS_FIRE_MEAN RATE_OF_FIRE 200%",
                                         "CONTINUOUS_FIRE_MEAN RADIUS 25%"};
    std::string s;
    for (int i = 0; i < kWeapons; ++i) {
        const bool plain = i < kPlain;
        const bool historic = i >= 56;
        const bool projectile = !plain && i >= 32 && i < 52;
        const bool laser = i >= 52 && i < 56;
        s += "Weapon " + weapon_name(i) + "\n";
        s += std::string("  PrimaryDamage = ") + choose(kDamage) + "\n";
        const char* radius = choose(kRadius);
        s += std::string("  PrimaryDamageRadius = ") + radius + "\n";
        if (chance(50)) {
            s += std::string("  SecondaryDamage = ") + choose(kDamage) + "\n";
            s += std::string("  SecondaryDamageRadius = ") + (chance(20) ? "0" : choose(kRadius)) + "\n";
        }
        // The first historic weapons are the ones `hammer` vectors fire.
        const bool hammered = i >= 56 && i < 60;
        s += std::string("  AttackRange = ") + (hammered ? "1000" : choose(kRange)) + "\n";
        if (chance(20) && !hammered) s += std::string("  MinimumAttackRange = ") + choose(kMin) + "\n";
        if (chance(10)) s += "  LeechRangeWeapon = Yes\n";
        s += std::string("  DamageType = ") + choose(kType) + "\n";
        s += std::string("  DeathType = ") + choose(kDeath) + "\n";
        if (i >= 8 && i < 20) s += std::string("  WeaponSpeed = ") + choose(kSlow) + "\n";
        else if (projectile) s += std::string("  WeaponSpeed = ") + choose(kSlow) + "\n";
        else if (chance(10)) s += "  WeaponSpeed = 100000\n";
        if (projectile && chance(25)) s += "  ScaleWeaponSpeed = Yes\n  MinWeaponSpeed = 3\n";
        if (chance(50)) s += "  RadiusDamageAffects =" + flags(kAffects, 7, 40) + "\n";
        if (chance(10)) s += "  DamageDealtAtSelfPosition = Yes\n";
        if (!plain && chance(20)) s += std::string("  RadiusDamageAngle = ") + choose(kAngle) + "\n";
        if (!plain && chance(25)) {
            s += std::string("  ShockWaveAmount = ") + choose(kShock) + "\n";
            if (chance(90)) s += std::string("  ShockWaveRadius = ") + (chance(10) ? "0" : "50") + "\n";
            if (chance(50)) s += "  ShockWaveTaperOff = 0.25\n";
        }
        // Shots that fly to the scattered point show the size of both scatters.
        const bool both = (projectile || laser) && i % 3 == 0;
        if (!plain && (both || chance(25))) s += std::string("  ScatterRadius = ") + choose(kScatter) + "\n";
        if (!plain && (both || chance(20))) s += std::string("  ScatterRadiusVsInfantry = ") + choose(kScatter) + "\n";
        if (i >= 20 && i < 26) {
            s += "  ScatterTarget = X:10 Y:0\n  ScatterTarget = X:-4.5 Y:7\n  ScatterTarget = X:0 Y:-12\n";
            s += "  ScatterTargetScalar = 2.5\n";
        }
        for (int b = 0; b < 2; ++b)
            if (chance(20)) s += std::string("  WeaponBonus = ") + choose(kBonus) + "\n";
        if (!plain && chance(15)) s += "  FireOCL = E_O" + std::to_string(pick(2)) + "\n";
        if (!plain && chance(10)) s += "  VeterancyFireOCL = ELITE E_O" + std::to_string(2 + pick(2)) + "\n";
        if (!plain && chance(15)) s += "  ProjectileDetonationOCL = E_O" + std::to_string(4 + pick(2)) + "\n";
        if (!plain && chance(5)) s += "  VeterancyProjectileDetonationOCL = VETERAN E_O0\n";
        if (projectile) {
            s += "  ProjectileObject = E_Shell" + std::to_string(i % kProjectiles) + "\n";
            s += "  ProjectileCollidesWith =" + flags(kCollides, 9, 35) + "\n";
            if (i % 5 == 0) s += "  ProjectileStreamName = E_Stream\n";
        }
        // A name the thing factory does not know leaves an instant weapon.
        if (i == 30 || i == 31) s += "  ProjectileObject = E_Nothing\n";
        if (laser) {
            s += "  LaserName = E_Laser\n";
            if (i >= 54) s += "  ScatterRadius = 100\n";
        }
        if (historic) {
            s += std::string("  HistoricBonusCount = ") + std::to_string(hammered ? 3 + (i % 2) : 1 + pick(4)) + "\n";
            s += std::string("  HistoricBonusRadius = ") + (hammered || chance(50) ? "40" : "400") + "\n";
            s += std::string("  HistoricBonusTime = ") + (chance(60) ? "100000" : "100") + "\n";
            // A chain downwards, a plain weapon, or itself (which never counts).
            const int bonus = i == 71 ? i : (i >= 60 && chance(40) ? 56 + static_cast<int>(pick(i - 56))
                                                                  : static_cast<int>(pick(kPlain)));
            s += "  HistoricBonusWeapon = " + weapon_name(bonus) + "\n";
        }
        s += "End\n";
    }
    return s;
}

static std::string body(const char* health) {
    return std::string("  Body = ActiveBody ModuleTag_Body\n    MaxHealth = ") + health +
           "\n    InitialHealth = " + health + "\n  End\n";
}
static std::string armor(const char* name) {
    return std::string("  ArmorSet\n    Conditions = None\n    Armor = ") + name + "\n    DamageFX = None\n  End\n";
}
static const char* kPhysics = "  Behavior = PhysicsBehavior ModuleTag_Phys\n    ShockResistance = 0.25\n"
                              "    ShockMaxYaw = 0.5\n    ShockMaxPitch = 0.75\n    ShockMaxRoll = 1.0\n  End\n";
static const char* kDestroy = "  Behavior = DestroyDie ModuleTag_Die\n  End\n";
static const char* kSpecialDie =
    "  Behavior = SpecialPowerCompletionDie ModuleTag_Power\n    SpecialPowerTemplate = E_Power\n  End\n";

static const int kUnits = 8;
static std::string unit_name(int i) { return "E_U" + std::to_string(i); }

static std::string object_ini() {
    std::string s;
    auto object = [&](const std::string& name, const std::string& kinds, const std::string& rest) {
        s += "Object " + name + "\n  KindOf = PRELOAD " + kinds + "\n" + rest + "End\n";
    };
    auto geom = [](const char* type, const char* major, const char* minor, const char* height) {
        return std::string("  Geometry = ") + type + "\n  GeometryMajorRadius = " + major +
               "\n  GeometryMinorRadius = " + minor + "\n  GeometryHeight = " + height + "\n";
    };
    object("E_U0", "VEHICLE", geom("CYLINDER", "6", "6", "8") + body("220.0") + armor("E_Half") + kPhysics);
    object("E_U1", "INFANTRY", geom("CYLINDER", "2", "2", "5") + body("60.0") + armor("E_Plain") + kPhysics + kDestroy);
    object("E_U2", "STRUCTURE IMMOBILE",
           geom("BOX", "12", "10", "20") + body("500.0") + armor("E_Odd") +
               "  Behavior = FireWeaponWhenDeadBehavior ModuleTag_Boom\n    StartsActive = Yes\n"
               "    DeathWeapon = E_W2\n  End\n" +
               kDestroy);
    object("E_U3", "AIRCRAFT VEHICLE", geom("SPHERE", "5", "5", "5") + body("120.0") + kPhysics);
    object("E_U4", "SHRUBBERY IMMOBILE", geom("CYLINDER", "3", "3", "6") + body("40.0") + kDestroy);
    object("E_U5", "STRUCTURE IMMOBILE",
           "  FenceWidth = 3.0\n" + geom("BOX", "8", "2", "6") + body("90.0") + armor("E_Plain"));
    object("E_U6", "VEHICLE", geom("CYLINDER", "4", "4", "6") + body("100.0") + kPhysics + kSpecialDie + kDestroy);
    object("E_U7", "SMALL_MISSILE BALLISTIC_MISSILE PROJECTILE",
           geom("SPHERE", "2", "2", "2") + body("30.0") + kPhysics + kDestroy);
    auto flight = [](const std::string& extra) {
        return "  Behavior = DumbProjectileBehavior ModuleTag_Flight\n    FirstHeight = 10\n    SecondHeight = 25.5\n"
               "    FirstPercentIndent = 20%\n    SecondPercentIndent = 90%\n" +
               extra + "  End\n";
    };
    object("E_Shell0", "PROJECTILE", geom("SPHERE", "2", "2", "2") + body("50.0") + kPhysics + flight("") + kDestroy);
    object("E_Shell1", "PROJECTILE SMALL_MISSILE",
           geom("SPHERE", "2", "2", "2") + body("50.0") + kPhysics + flight("    DetonateCallsKill = Yes\n") +
               kSpecialDie + kDestroy);
    object("E_Shell2", "PROJECTILE", geom("SPHERE", "1", "1", "1") + body("50.0") +
                                         flight("    MaxLifespan = 200\n"));
    object("E_Shell3", "PROJECTILE", geom("SPHERE", "1", "1", "1") + body("50.0") + kSpecialDie);
    object("E_Stream", "INERT", "  Behavior = ProjectileStreamUpdate ModuleTag_Stream\n  End\n");
    object("E_Laser", "INERT", "");
    return s;
}

// ---------------------------------------------------------------------------------------------
// World state
// ---------------------------------------------------------------------------------------------
template <typename T> static T* find_behavior(const Object* obj) {
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m)
        if (T* typed = dynamic_cast<T*>(*m)) return typed;
    return nullptr;
}

static std::vector<Object*> all_objects() {
    std::vector<Object*> out;
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) out.push_back(o);
    std::sort(out.begin(), out.end(), [](Object* a, Object* b) { return a->getID() < b->getID(); });
    return out;
}

static std::string module_state(Module* m) {
    Snapshot* s = m;
    const std::string save = oracle_wp51::World::snapshot_save_hex(s);
    std::ostringstream out;
    out << fnv(save) << '.' << oracle_wp51::World::snapshot_crc(s);
    return out.str();
}

static std::string state(uint32_t ret) {
    std::ostringstream out;
    out << 'R' << GetGameLogicRandomSeedCRC() << ',' << ret << ';';
    for (Object* o : all_objects()) {
        const Coord3D* p = o->getPosition();
        BodyModuleInterface* b = o->getBodyModule();
        out << 'O' << static_cast<uint32_t>(o->getID()) << ',' << o->getTemplate()->getName().str() << ','
            << (o->isDestroyed() ? 1 : 0) << (o->isEffectivelyDead() ? 1 : 0) << ',' << bits(p->x) << ','
            << bits(p->y) << ',' << bits(p->z) << ',' << bits(o->getOrientation()) << ','
            << (b ? bits(b->getHealth()) : 0u) << ','
            << o->getExperienceTracker()->getCurrentExperience();
        if (ActiveBody* ab = find_behavior<ActiveBody>(o)) out << ",b" << module_state(ab);
        if (PhysicsBehavior* ph = o->getPhysics()) out << ",p" << module_state(ph);
        if (DumbProjectileBehavior* db = find_behavior<DumbProjectileBehavior>(o))
            out << ",f" << static_cast<uint32_t>(db->projectileGetLauncherID());
        if (SpecialPowerCompletionDie* sp = find_behavior<SpecialPowerCompletionDie>(o))
            out << ",s" << module_state(sp);
        if (ProjectileStreamUpdate* ps = find_behavior<ProjectileStreamUpdate>(o))
            out << ",t" << module_state(ps);
        const Drawable* d = o->getDrawable();
        out << ',' << (d && d->getModelConditionFlags().test(MODELCONDITION_STUNNED_FLAILING) ? 1 : 0) << ';';
    }
    for (const auto& ddi : TheWeaponStore->*steal_ddi()) {
        out << 'D' << ddi.m_delayedWeapon->getName().str() << ',' << bits(ddi.m_delayDamagePos.x) << ','
            << bits(ddi.m_delayDamagePos.y) << ',' << bits(ddi.m_delayDamagePos.z) << ',' << ddi.m_delayDamageFrame
            << ',' << static_cast<uint32_t>(ddi.m_delaySourceID) << ','
            << static_cast<uint32_t>(ddi.m_delayIntendedVictimID) << ','
            << bits(ddi.m_bonus.getField(WeaponBonus::DAMAGE)) << ','
            << bits(ddi.m_bonus.getField(WeaponBonus::RADIUS)) << ';';
    }
    for (int i = 56; i < kWeapons; ++i) {
        const WeaponTemplate* wt = TheWeaponStore->findWeaponTemplate(weapon_name(i).c_str());
        for (const auto& h : wt->*steal_history())
            out << 'H' << i << ',' << h.frame << ',' << bits(h.location.x) << ',' << bits(h.location.y) << ';';
    }
    out << g_ocl.str();
    g_ocl.str("");
    auto& finished = TheScriptEngine->*steal_finished();
    // The port keeps no player with a completion, so they are compared in creator order.
    std::vector<uint32_t> creators;
    for (int player = 0; player < MAX_PLAYER_COUNT; ++player) {
        for (const auto& pair : finished[player]) creators.push_back(static_cast<uint32_t>(pair.second));
        finished[player].clear();
    }
    std::sort(creators.begin(), creators.end());
    for (uint32_t creator : creators) out << 'S' << creator << ';';
    return out.str();
}

// ---------------------------------------------------------------------------------------------
// Vectors
// ---------------------------------------------------------------------------------------------
struct Unit {
    int type, team, x, y, lift, angle, veterancy, bonus, producer;
};
struct Vec {
    uint32_t frame, seed;
    std::vector<Unit> units;
    std::vector<std::vector<int>> ops;
    std::vector<std::string> rows;
};

static const Real kAngles[8] = {0.0f, 0.5f, 1.5707964f, 2.5f, 3.1415927f, -1.0f, -2.0f, 4.5f};

static void generate(Vec& v, int index) {
    v.frame = 1000 + pick(100000);
    v.seed = pick(0x7fffffff);
    const int units = 3 + static_cast<int>(pick(5));
    const int spread = chance(30) ? 15 : (chance(60) ? 60 : 160);
    for (int i = 0; i < units; ++i) {
        Unit u;
        u.type = static_cast<int>(pick(kUnits));
        if (i > 0 && chance(20)) u.type = v.units[0].type;
        u.team = static_cast<int>(pick(5)) % 3;
        u.x = 100 + static_cast<int>(pick(spread));
        u.y = 100 + static_cast<int>(pick(spread));
        u.lift = u.type == 3 && chance(80) ? 20 + static_cast<int>(pick(40)) : 0;
        u.angle = static_cast<int>(pick(8));
        u.veterancy = static_cast<int>(pick(4));
        u.bonus = chance(30) ? static_cast<int>(pick(8)) : 0;
        u.producer = chance(15) ? static_cast<int>(pick(units)) : -1;
        v.units.push_back(u);
    }
    const int ops = 4 + static_cast<int>(pick(8));
    if (index % 8 == 0) {
        // Hammer one historic weapon at two points exactly its radius apart and one far off.
        static const int kSpots[4][2] = {{100, 100}, {124, 132}, {110, 104}, {300, 300}};
        const int weapon = 56 + static_cast<int>(pick(4));
        const int shots = 8 + static_cast<int>(pick(10));
        for (int k = 0; k < shots; ++k) {
            const int* spot = kSpots[pick(4)];
            if (chance(15)) v.ops.push_back({4, 1 + static_cast<int>(pick(chance(80) ? 3 : 40))});
            else v.ops.push_back({1, weapon, static_cast<int>(pick(1000)), spot[0], spot[1], 5});
        }
        return;
    }
    // A vector may hammer one weapon, so that delayed and historic damage pile up.
    const int focus = index % 4 == 0 ? 56 + static_cast<int>(pick(kWeapons - 56)) : -1;
    for (int k = 0; k < ops; ++k) {
        uint32_t roll = pick(100);
        const int weapon = focus >= 0 && chance(70) ? focus : static_cast<int>(pick(kWeapons));
        const int src = static_cast<int>(pick(1000));
        const int tgt = static_cast<int>(pick(1000));
        const int x = 80 + static_cast<int>(pick(spread + 40));
        const int y = 80 + static_cast<int>(pick(spread + 40));
        const int z = static_cast<int>(pick(30));
        if (roll < 34) v.ops.push_back({0, weapon, src, tgt});
        else if (roll < 48) v.ops.push_back({1, weapon, src, x, y, z});
        else if (roll < 58) v.ops.push_back({2, weapon, src, x, y, z, chance(50) ? 0 : static_cast<int>(pick(8))});
        else if (roll < 66) v.ops.push_back({3, weapon, src, x, y, z});
        else if (roll < 91) v.ops.push_back({4, 1 + static_cast<int>(pick(chance(70) ? 6 : 40))});
        else if (roll < 95) v.ops.push_back({5, src, x, y, static_cast<int>(pick(8))});
        else v.ops.push_back({6, src});
    }
}

static void run(oracle_wp51::World& w, Vec& v) {
    TheWeaponStore->reset();
    auto& finished = TheScriptEngine->*steal_finished();
    for (int player = 0; player < MAX_PLAYER_COUNT; ++player) finished[player].clear();
    g_ocl.str("");

    Team* teams[3] = {w.team_a, w.team_b, w.team_neutral};
    std::vector<Object*> spawned;
    for (const Unit& u : v.units) {
        Coord3D p;
        p.x = static_cast<Real>(u.x);
        p.y = static_cast<Real>(u.y);
        p.z = StepTerrain::height(p.x, p.y) + static_cast<Real>(u.lift);
        Object* o = w.spawn_object(unit_name(u.type).c_str(), teams[u.team], p, kAngles[u.angle]);
        if (!o) die("spawn");
        o->getExperienceTracker()->*steal_level() = static_cast<VeterancyLevel>(u.veterancy);
        for (int b = 0; b < 3; ++b)
            if (u.bonus & (1 << b)) {
                static const WeaponBonusConditionType kConditions[3] = {
                    WEAPONBONUSCONDITION_GARRISONED, WEAPONBONUSCONDITION_HORDE,
                    WEAPONBONUSCONDITION_CONTINUOUS_FIRE_MEAN};
                o->setWeaponBonusCondition(kConditions[b]);
            }
        if (u.lift > 0) o->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_AIRBORNE_TARGET));
        spawned.push_back(o);
    }
    for (size_t i = 0; i < v.units.size(); ++i)
        if (v.units[i].producer >= 0) spawned[i]->setProducer(spawned[v.units[i].producer]);
    ThePartitionManager->update();
    v.rows.push_back(state(0));

    UnsignedInt frame = v.frame;
    for (const std::vector<int>& op : v.ops) {
        w.set_frame(frame);
        std::vector<Object*> live = all_objects();
        // Only things with a body can be aimed at or used as a source.
        live.erase(std::remove_if(live.begin(), live.end(), [](Object* o) { return !o->getBodyModule(); }), live.end());
        uint32_t ret = 0;
        const int code = op[0];
        if (code == 4) {
            for (int t = 0; t < op[1]; ++t) {
                w.set_frame(++frame);
                for (Object* o : all_objects()) {
                    DumbProjectileBehavior* db = find_behavior<DumbProjectileBehavior>(o);
                    if (db && !o->isDestroyed()) static_cast<UpdateModuleInterface*>(db)->update();
                }
                // The order of `GameLogic::update`.
                ThePartitionManager->update();
                w.flush_destroy_list();
                TheWeaponStore->update();
            }
        } else if (!live.empty()) {
            Object* src = live[static_cast<size_t>(op[code >= 5 ? 1 : 2]) % live.size()];
            if (code <= 3) {
                const WeaponTemplate* wt = TheWeaponStore->findWeaponTemplate(weapon_name(op[1]).c_str());
                if (!wt) die("weapon");
                if (code == 0) {
                    // Never at itself: C++ asserts against it.
                    Object* tgt = live[static_cast<size_t>(op[3]) % live.size()];
                    if (tgt != src) TheWeaponStore->createAndFireTempWeapon(wt, src, tgt);
                } else {
                    Coord3D p;
                    p.x = static_cast<Real>(op[3]);
                    p.y = static_cast<Real>(op[4]);
                    p.z = static_cast<Real>(op[5]);
                    if (code == 1) {
                        TheWeaponStore->createAndFireTempWeapon(wt, src, &p);
                    } else if (code == 2) {
                        TheWeaponStore->handleProjectileDetonation(wt, src, &p,
                                                                   static_cast<WeaponBonusConditionFlags>(op[6]));
                    } else {
                        Weapon* weapon = TheWeaponStore->allocateNewWeapon(wt, PRIMARY_WEAPON);
                        weapon->loadAmmoNow(src);
                        Object* shell = weapon->forceFireWeapon(src, &p);
                        ret = shell ? static_cast<uint32_t>(shell->getID()) : 0u;
                        deleteInstance(weapon);
                    }
                }
            } else if (code == 5) {
                Coord3D p;
                p.x = static_cast<Real>(op[2]);
                p.y = static_cast<Real>(op[3]);
                p.z = StepTerrain::height(p.x, p.y);
                src->setPosition(&p);
                src->setOrientation(kAngles[op[4]]);
                ThePartitionManager->update();
            } else {
                src->kill();
            }
        }
        v.rows.push_back(state(ret));
        w.flush_destroy_list();
    }

    TheWeaponStore->reset();
    for (Object* o : all_objects()) TheGameLogic->destroyObject(o);
    w.flush_destroy_list();
    ThePartitionManager->update();
}

static std::string json_string(const std::string& s) {
    std::string out = "\"";
    for (char c : s) {
        if (c == '\n') out += "\\n";
        else if (c == '"' || c == '\\') { out += '\\'; out += c; }
        else out += c;
    }
    return out + "\"";
}

int main(int argc, char** argv) {
    if (argc < 2) die("usage: gen_pkt399_weapon_effects <out.json>");
    const std::string out_path = argv[1];
    const int dump = std::getenv("WEAPON_DUMP") ? std::atoi(std::getenv("WEAPON_DUMP")) : -1;

    oracle_wp51::World w;
    TheTerrainLogic = NEW StepTerrain;
    static_cast<StepTerrain*>(TheTerrainLogic)->extent.lo = {0.0f, 0.0f, 0.0f};
    static_cast<StepTerrain*>(TheTerrainLogic)->extent.hi = {1000.0f, 1000.0f, 100.0f};
    ThePartitionManager->init();
    w.load_ini(kArmorIni);
    std::string ocls;
    for (int i = 0; i < kOcls; ++i) ocls += "ObjectCreationList E_O" + std::to_string(i) + "\nEnd\n";
    w.load_ini(ocls);
    for (int i = 0; i < kOcls; ++i) {
        const ObjectCreationList* list =
            TheObjectCreationListStore->findObjectCreationList(("E_O" + std::to_string(i)).c_str());
        if (!list) die("missing OCL");
        g_nugget[i].list = i;
        const_cast<ObjectCreationList*>(list)->addObjectCreationNugget(&g_nugget[i]);
    }
    w.load_ini("SpecialPower E_Power\n  Enum = SPECIAL_DAISY_CUTTER\nEnd\n");
    w.init_with_neutral_player(nullptr, nullptr);
    const std::string weapons = weapon_ini();
    const std::string objects = object_ini();
    w.load_ini(weapons);
    w.load_ini(objects);
    TheWeaponStore->postProcessLoad();
    TheWritableGlobalData->m_unitDamagedThresh = 0.7f;
    TheWritableGlobalData->m_unitReallyDamagedThresh = 0.35f;
    TheWritableGlobalData->m_gravity = -1.0f;

    const int kTotal = 1500;
    // Rows of the first vectors are kept whole to locate a difference; the rest are hashed.
    const int kWhole = 40;
    const std::string tmp = out_path + ".tmp";
    FILE* f = std::fopen(tmp.c_str(), "w");
    if (!f) die("open");
    std::fprintf(f, "{\"vector_count\":%d,\n\"weapon_ini\":%s,\n\"object_ini\":%s,\n\"armor_ini\":%s,\n\"vectors\":[\n",
                 kTotal, json_string(weapons).c_str(), json_string(objects).c_str(), json_string(kArmorIni).c_str());
    for (int i = 0; i < kTotal; ++i) {
        Vec v;
        generate(v, i);
        if (std::getenv("WEAPON_TRACE")) std::fprintf(stderr, "vector %d\n", i);
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(v.frame);
        InitRandom(v.seed);
        run(w, v);
        if (i == dump)
            for (size_t r = 0; r < v.rows.size(); ++r) std::fprintf(stderr, "row %zu: %s\n", r, v.rows[r].c_str());

        std::ostringstream line;
        line << "{\"vector\":" << i << ",\"frame\":" << v.frame << ",\"seed\":" << v.seed << ",\"units\":[";
        for (size_t u = 0; u < v.units.size(); ++u) {
            const Unit& n = v.units[u];
            line << (u ? "," : "") << '[' << n.type << ',' << n.team << ',' << n.x << ',' << n.y << ',' << n.lift
                 << ',' << n.angle << ',' << n.veterancy << ',' << n.bonus << ',' << n.producer << ']';
        }
        line << "],\"ops\":[";
        for (size_t k = 0; k < v.ops.size(); ++k) {
            line << (k ? "," : "") << '[';
            for (size_t a = 0; a < v.ops[k].size(); ++a) line << (a ? "," : "") << v.ops[k][a];
            line << ']';
        }
        line << "],\"rows\":[";
        for (size_t r = 0; r < v.rows.size(); ++r) {
            line << (r ? "," : "");
            if (i < kWhole) line << json_string(v.rows[r]);
            else line << "\"#" << fnv(v.rows[r]) << "\"";
        }
        line << "]}";
        std::fprintf(f, "%s%s\n", line.str().c_str(), i + 1 < kTotal ? "," : "");
    }
    std::fprintf(f, "]}\n");
    std::fclose(f);
    if (std::rename(tmp.c_str(), out_path.c_str()) != 0) die("rename");
    return 0;
}
