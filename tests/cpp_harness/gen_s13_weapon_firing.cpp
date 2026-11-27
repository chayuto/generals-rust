// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for weapon firing (AI port slice S13, PKT-399): the range queries,
// `computeApproachTarget`, the terrain line of sight questions, `estimateWeaponDamage`,
// `Object::fireCurrentWeapon` / `preFireCurrentWeapon` with the real firing tracker,
// scatter, infantry inaccuracy and scatter targets, assistance requests, disarm shots,
// the assault order, `reloadAllAmmo`, bonus retimes and `FireWeaponPower`, run on the
// unmodified reference in a small real world (players A and B enemies, a neutral player).
//
// Harness overrides only: the terrain's ground height and line of sight (answered from a
// hash of the question, which is recorded) and a recording AI (`OracleAI`) that logs the
// attack orders and the assault order and may give a sneaky targeting offset.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_s13_weapon_firing.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/S13_WeaponFiring.json
//
// `S13_DUMP=<vector>` prints that vector's rows.
//
// Not covered: bridge records (a bridge-kind object without one is), aircraft that adjust
// their destination, spawn slaves in the damage estimate, projectiles and countermeasures
// (projectiles: PKT399_WeaponEffects), the update of the firing tracker, loading a save.

#include "oracle_wp51_world.h"

#include "Common/ModuleFactory.h"
#include "Common/Player.h"
#include "Common/RandomValue.h"
#include "Common/Team.h"
#include "GameLogic/FiringTracker.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/AssaultTransportAIUpdate.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/SpecialPowerModule.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Weapon.h"

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

static void die(const char* what) {
    std::fprintf(stderr, "oracle error: %s\n", what);
    std::exit(1);
}

struct ForceReloadTag { using type = UnsignedInt FiringTracker::*; };
template struct MemberAccess<ForceReloadTag, &FiringTracker::m_frameToForceReload>;
inline UnsignedInt FiringTracker::* access_member(ForceReloadTag);
struct StopSoundTag { using type = UnsignedInt FiringTracker::*; };
template struct MemberAccess<StopSoundTag, &FiringTracker::m_frameToStopLoopingSound>;
inline UnsignedInt FiringTracker::* access_member(StopSoundTag);
struct MinesTag { using type = UnsignedInt AcademyStats::*; };
template struct MemberAccess<MinesTag, &AcademyStats::m_minesCleared>;
inline UnsignedInt AcademyStats::* access_member(MinesTag);

static uint32_t g_state = 0x13131313u;
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
// Terrain and recording AI
// ---------------------------------------------------------------------------------------------
static std::ostringstream g_log;

class S13Terrain : public oracle_wp51::OracleTerrainLogic {
public:
    static Real height(Real x, Real y) {
        return static_cast<Real>(static_cast<int>(x / 40.0f) % 3) * 4.0f +
               static_cast<Real>(static_cast<int>(y / 60.0f) % 4) * 3.0f;
    }
    Real getGroundHeight(Real x, Real y, Coord3D* normal = nullptr) const override {
        if (normal) { normal->x = 0.0f; normal->y = 0.0f; normal->z = 1.0f; }
        return height(x, y);
    }
    Bool isClearLineOfSight(const Coord3D& a, const Coord3D& b) const override {
        const uint32_t v[6] = {bits(a.x), bits(a.y), bits(a.z), bits(b.x), bits(b.y), bits(b.z)};
        uint32_t h = 2166136261u;
        for (uint32_t u : v) h = (h ^ u) * 16777619u;
        const Bool clear = ((h >> 7) & 1) != 0;
        g_log << 'L' << v[0] << ',' << v[1] << ',' << v[2] << ',' << v[3] << ',' << v[4] << ',' << v[5]
              << '=' << static_cast<int>(clear) << ';';
        return clear;
    }
};

static const int kMaxUnits = 12;
// Sneaky offsets of the current vector, by object ID.
static bool g_sneaky[kMaxUnits + 64];
static const Real kSneak[3] = {13.0f, -7.5f, 2.0f};

class RecAI : public AIUpdateInterface, public AssaultTransportAIInterface {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(RecAI, "OracleS13RecAI", 64, 64)
public:
    RecAI(Thing* thing, const ModuleData* data) : AIUpdateInterface(thing, data) {}
    Bool isIdle() const override { return TRUE; }
    Bool getSneakyTargetingOffset(Coord3D* offset) const override {
        const UnsignedInt id = static_cast<UnsignedInt>(getObject()->getID());
        if (id >= kMaxUnits + 64 || !g_sneaky[id]) return FALSE;
        if (offset) { offset->x = kSneak[0]; offset->y = kSneak[1]; offset->z = kSneak[2]; }
        return TRUE;
    }
    AssaultTransportAIInterface* getAssaultTransportAIInterface() override { return this; }
    const AssaultTransportAIInterface* getAssaultTransportAIInterface() const override { return this; }
    void beginAssault(const Object* target) const override {
        g_log << 'B' << static_cast<UnsignedInt>(getObject()->getID()) << ','
              << (target ? static_cast<UnsignedInt>(target->getID()) : 0u) << ';';
    }
    void aiDoCommand(const AICommandParms* p) override {
        const UnsignedInt id = static_cast<UnsignedInt>(getObject()->getID());
        switch (p->m_cmd) {
        case AICMD_ATTACK_OBJECT:
            g_log << 'C' << id << " attack " << static_cast<int>(p->m_cmdSource) << ' '
                  << static_cast<UnsignedInt>(p->m_obj->getID()) << ' ' << p->m_intValue << ';';
            break;
        case AICMD_ATTACK_POSITION:
            g_log << 'C' << id << " attackpos " << static_cast<int>(p->m_cmdSource) << ' ' << bits(p->m_pos.x)
                  << ' ' << bits(p->m_pos.y) << ' ' << bits(p->m_pos.z) << ' ' << p->m_intValue << ';';
            break;
        default:
            break;  // orders other modules give on damage or creation are not compared
        }
    }
};
RecAI::~RecAI() {}
struct ExposedFactory : public ModuleFactory { using ModuleFactory::addModuleInternal; };
static Module* make_rec_ai(Thing* t, const ModuleData* d) { return newInstance(RecAI)(t, d); }

// ---------------------------------------------------------------------------------------------
// Fixture
// ---------------------------------------------------------------------------------------------
static const char* kArmorIni =
    "Armor E_Plain\n  Armor = Default 100%\nEnd\n"
    "Armor E_Odd\n  Armor = Default 120%\n  Armor = EXPLOSION 0%\n  Armor = SNIPER 200%\n"
    "  Armor = FLAME 37%\nEnd\n";

static const int kWeapons = 72;
static const int kTypes = 12;
static std::string weapon_name(int i) { return "E_W" + std::to_string(i); }

static std::string weapon_ini() {
    static const char* const kDamage[] = {"0", "0.5", "7", "25", "80", "200"};
    static const char* const kRadius[] = {"0", "0", "5", "15", "30"};
    // Weapon `i` has type `i % 12`, so that every type can be put in a shooter's hands.
    static const char* const kType[kTypes] = {"EXPLOSION", "DISARM", "SNIPER", "DEPLOY", "FLAME", "SMALL_ARMS",
                                              "SURRENDER", "ARMOR_PIERCING", "SNIPER", "DISARM", "EXPLOSION",
                                              "HACK"};
    static const char* const kDeath[] = {"NORMAL", "NORMAL", "BURNED", "EXPLODED"};
    static const char* const kRange[] = {"5", "12", "13", "40", "70.5", "100", "200", "1000"};
    static const char* const kMin[] = {"2", "8", "12.5", "12.5", "13", "15", "40", "90"};
    static const char* const kSlow[] = {"3", "13", "90"};
    static const char* const kScatter[] = {"1", "7.5", "30"};
    static const char* const kPrefire[] = {"PER_SHOT", "PER_ATTACK", "PER_CLIP"};
    static const char* const kBonus[] = {"GARRISONED RANGE 133%", "HORDE RANGE 50%", "HORDE RATE_OF_FIRE 150%",
                                         "CONTINUOUS_FIRE_MEAN RATE_OF_FIRE 200%",
                                         "CONTINUOUS_FIRE_MEAN PRE_ATTACK 50%", "GARRISONED RADIUS 150%",
                                         "HORDE DAMAGE 300%", "CONTINUOUS_FIRE_FAST RATE_OF_FIRE 300%",
                                         "GARRISONED DAMAGE 125%", "GARRISONED PRE_ATTACK 200%"};
    std::string s;
    for (int i = 0; i < kWeapons; ++i) {
        s += "Weapon " + weapon_name(i) + "\n";
        s += std::string("  PrimaryDamage = ") + choose(kDamage) + "\n";
        s += std::string("  PrimaryDamageRadius = ") + choose(kRadius) + "\n";
        s += std::string("  AttackRange = ") + choose(kRange) + "\n";
        // The first shooter's primary has a minimum of exactly one cell, where the retreat starts.
        if (i % kTypes == 0) s += "  MinimumAttackRange = 12.5\n";
        else if (chance(50)) s += std::string("  MinimumAttackRange = ") + choose(kMin) + "\n";
        s += std::string("  DamageType = ") + kType[i % kTypes] + "\n";
        s += std::string("  DeathType = ") + choose(kDeath) + "\n";
        if (chance(15)) s += std::string("  WeaponSpeed = ") + choose(kSlow) + "\n";
        if (chance(30)) s += std::string("  ScatterRadius = ") + choose(kScatter) + "\n";
        if (chance(35)) s += std::string("  ScatterRadiusVsInfantry = ") + choose(kScatter) + "\n";
        if (chance(20)) {
            s += "  ScatterTarget = X:10 Y:0\n  ScatterTarget = X:-4.5 Y:7\n  ScatterTarget = X:0 Y:-12\n";
            s += "  ScatterTargetScalar = 2.5\n";
        }
        if (chance(10)) s += "  LeechRangeWeapon = Yes\n";
        if (chance(20)) s += "  AllowAttackGarrisonedBldgs = Yes\n";
        if (chance(25)) {
            s += chance(50) ? "  MinTargetPitch = -20\n" : "  MinTargetPitch = 5\n";
            s += chance(50) ? "  MaxTargetPitch = 25\n" : "  MaxTargetPitch = 80\n";
        }
        if (chance(60)) s += "  ClipSize = " + std::to_string(1 + pick(4)) + "\n";
        if (chance(50)) s += "  ClipReloadTime = " + std::to_string(30 * (1 + pick(10))) + "\n";
        if (chance(30)) s += "  AutoReloadsClip = No\n";
        if (chance(50)) {
            static const char* const kCollides[] = {"ALLIES", "ENEMIES", "STRUCTURES", "SHRUBBERY",
                                                    "PROJECTILES", "CONTROLLED_STRUCTURES"};
            s += "  ProjectileCollidesWith =";
            for (const char* name : kCollides)
                if (chance(40)) s += std::string(" ") + name;
            s += "\n";
        }
        if (chance(40)) {
            const int lo = 33 * (1 + pick(6));
            s += chance(50) ? "  DelayBetweenShots = " + std::to_string(lo) + "\n"
                            : "  DelayBetweenShots = Min:" + std::to_string(lo) + " Max:" +
                                  std::to_string(lo + 100) + "\n";
        }
        if (chance(60)) {
            s += std::string("  PreAttackDelay = ") + std::to_string(33 * (1 + pick(30))) + "\n";
            s += std::string("  PreAttackType = ") + choose(kPrefire) + "\n";
        }
        if (chance(30)) s += "  ShotsPerBarrel = " + std::to_string(1 + pick(3)) + "\n";
        if (chance(35)) s += "  RequestAssistRange = " + std::to_string(20 + 40 * pick(3)) + "\n";
        if (chance(50)) s += "  WeaponBonus = GARRISONED DAMAGE 125%\n";
        if (chance(20)) s += "  AutoReloadWhenIdle = " + std::to_string(100 * (1 + pick(5))) + "\n";
        if (chance(30)) {
            s += "  ContinuousFireOne = " + std::to_string(1 + pick(3)) + "\n";
            s += "  ContinuousFireTwo = " + std::to_string(3 + pick(3)) + "\n";
        }
        if (chance(30)) s += "  ContinuousFireCoast = " + std::to_string(100 * (1 + pick(8))) + "\n";
        if (chance(15)) s += "  FireSoundLoopTime = " + std::to_string(100 * (1 + pick(5))) + "\n";
        for (int b = 0; b < 3; ++b)
            if (chance(15)) s += std::string("  WeaponBonus = ") + choose(kBonus) + "\n";
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
static std::string geom(const char* type, const char* major, const char* minor, const char* height) {
    return std::string("  Geometry = ") + type + "\n  GeometryMajorRadius = " + major +
           "\n  GeometryMinorRadius = " + minor + "\n  GeometryHeight = " + height + "\n";
}
static const char* kAI = "  Behavior = OracleAI ModuleTag_AI\n  End\n";

static const int kShooters = 8;
static const int kTargets = 12;
static std::string unit_name(int i) {
    return i < kShooters ? "E_S" + std::to_string(i) : "E_T" + std::to_string(i - kShooters);
}

static std::string object_ini() {
    std::string s;
    // Each weapon slot of the shooters takes the next damage type.
    int next_type = 0;
    auto gun = [&]() { return weapon_name(next_type++ % kTypes + kTypes * static_cast<int>(pick(kWeapons / kTypes))); };
    auto object = [&](const std::string& name, const std::string& kinds, const std::string& rest) {
        s += "Object " + name + "\n  KindOf = PRELOAD " + kinds +
             "\n  VisionRange = 0\n  ShroudClearingRange = 0\n" + rest + "End\n";
    };
    std::vector<std::string> primaries;
    for (int i = 0; i < kShooters; ++i) primaries.push_back(gun());
    for (int i = 0; i < kShooters; ++i) {
        std::string set = "  WeaponSet\n    Conditions = None\n    Weapon = PRIMARY " + primaries[i] + "\n";
        if (i % 2 == 0) set += "    Weapon = SECONDARY " + gun() + "\n";
        if (i % 4 == 0) set += "    Weapon = TERTIARY " + gun() + "\n";
        if (i == 6) set += "    ShareWeaponReloadTime = Yes\n    Weapon = SECONDARY " + gun() + "\n";
        set += "  End\n";
        std::string kinds = i == 4 ? "AIRCRAFT VEHICLE" : (i == 5 ? "INFANTRY" : "VEHICLE");
        std::string shape = i == 1 ? geom("BOX", "7", "4", "6")
                                   : (i == 4 ? geom("SPHERE", "5", "5", "5")
                                             : geom("CYLINDER", i == 5 ? "2" : "6", "6", "8"));
        std::string extra = set + shape + body("1000000.0") + armor("E_Plain") + kAI;
        if (i == 0 || i == 1)
            extra += "  Behavior = AssistedTargetingUpdate ModuleTag_Assist\n    AssistingClipSize = 3\n"
                     "    AssistingWeaponSlot = PRIMARY\n  End\n";
        if (i == 2 || i == 6)
            extra += "  Behavior = FireWeaponPower ModuleTag_Power\n    SpecialPowerTemplate = E_Power\n"
                     "    MaxShotsToFire = 3\n  End\n";
        object(unit_name(i), kinds, extra);
    }
    auto target = [&](int t, const std::string& kinds, const std::string& rest) {
        object("E_T" + std::to_string(t), kinds, rest);
    };
    const std::string die = "  Behavior = DestroyDie ModuleTag_Die\n  End\n";
    target(0, "INFANTRY", geom("CYLINDER", "2", "2", "5") + body("100000.0") + armor("E_Plain"));
    target(1, "STRUCTURE IMMOBILE",
           geom("BOX", "12", "10", "20") + body("100000.0") + armor("E_Odd") +
               "  Behavior = GarrisonContain ModuleTag_G\n    ContainMax = 5\n  End\n");
    target(2, "STRUCTURE IMMOBILE",
           geom("BOX", "10", "10", "15") + body("100000.0") + armor("E_Plain") +
               "  Behavior = GarrisonContain ModuleTag_G\n    ContainMax = 5\n"
               "    InitialRoster = E_Rider 2\n  End\n");
    target(3, "STRUCTURE IMMOBILE",
           geom("CYLINDER", "9", "9", "12") + body("100000.0") + armor("E_Plain") +
               "  Behavior = GarrisonContain ModuleTag_G\n    ContainMax = 5\n"
               "    ImmuneToClearBuildingAttacks = Yes\n    InitialRoster = E_Rider 1\n  End\n");
    target(4, "SHRUBBERY IMMOBILE", geom("CYLINDER", "3", "3", "6") + body("100000.0"));
    target(5, "MINE IMMOBILE",
           geom("CYLINDER", "3", "3", "1") + body("100.0") + die +
               "  Behavior = MinefieldBehavior ModuleTag_Mine\n    DetonationWeapon = E_W0\n"
               "    NumVirtualMines = 4\n  End\n");
    target(6, "MINE IMMOBILE", geom("CYLINDER", "3", "3", "1") + body("100.0") + die);
    target(7, "BOOBY_TRAP IMMOBILE", geom("CYLINDER", "2", "2", "2") + body("100.0") + die);
    target(8, "DEMOTRAP STRUCTURE IMMOBILE", geom("BOX", "4", "4", "3") + body("100.0") + die);
    target(9, "BRIDGE STRUCTURE IMMOBILE", geom("BOX", "30", "8", "4") + body("100000.0"));
    target(10, "STRUCTURE IMMOBILE", geom("BOX", "8", "6", "30") + body("100000.0") + armor("E_Odd"));
    target(11, "AIRCRAFT VEHICLE", geom("SPHERE", "4", "4", "4") + body("100000.0") + armor("E_Plain"));
    object("E_Rider", "INFANTRY", geom("CYLINDER", "2", "2", "4") + body("100000.0"));
    return s;
}

// ---------------------------------------------------------------------------------------------
// Vectors
// ---------------------------------------------------------------------------------------------
struct Unit {
    int type, team, x, y, lift, angle, bonus, flags;
};
struct Vec {
    uint32_t frame, seed;
    std::vector<Unit> units;
    std::vector<std::vector<int>> ops;
    std::vector<std::string> rows;
};

static const Real kAngles[8] = {0.0f, 0.5f, 1.5707964f, 2.5f, 3.1415927f, -1.0f, -2.0f, 4.5f};
static const Real kOffsets[4] = {0.0f, 0.0f, 0.7f, -2.1f};

static void generate(Vec& v, int index) {
    v.frame = 1000 + pick(100000);
    v.seed = pick(0x7fffffff);
    const int spread = chance(35) ? 16 : (chance(50) ? 60 : 160);
    // Near the edges too, where the approach target is clipped to the extent.
    const int base = chance(8) ? 2 : (chance(8) ? 960 - spread : 300);
    const int shooters = 2 + static_cast<int>(pick(3));
    const int targets = 2 + static_cast<int>(pick(5));
    const int lead = index % 5 == 0 ? static_cast<int>(pick(2)) : static_cast<int>(pick(kShooters));
    for (int i = 0; i < shooters + targets; ++i) {
        Unit u;
        if (i < shooters) {
            // Copies of the first shooter answer its assistance requests.
            u.type = i == 0 || chance(40) ? lead : static_cast<int>(pick(kShooters));
            u.team = chance(85) ? 0 : static_cast<int>(pick(3));
        } else {
            u.type = kShooters + static_cast<int>(pick(kTargets));
            u.team = chance(75) ? 1 : static_cast<int>(pick(3));
        }
        u.x = base + static_cast<int>(pick(spread));
        u.y = base + static_cast<int>(pick(spread));
        u.lift = (u.type == 4 || u.type == kShooters + 11) && chance(75) ? 15 + static_cast<int>(pick(60))
                 : (chance(10) ? static_cast<int>(pick(40)) : 0);
        u.angle = static_cast<int>(pick(8));
        u.bonus = chance(30) ? static_cast<int>(pick(16)) : 0;
        u.flags = 0;
        if (u.lift >= 15 && chance(80)) u.flags |= 1;  // airborne target
        // Under construction clears shroud, which a neutral unit cannot do without a client.
        if (chance(8) && u.team != 2) u.flags |= 2;
        if (u.type < kShooters && chance(12)) u.flags |= 4;  // sneaky offset
        if (chance(15)) u.flags |= 8;                        // undetected defector
        v.units.push_back(u);
    }
    const int ops = 6 + static_cast<int>(pick(14));
    const int n = shooters + targets;
    for (int k = 0; k < ops; ++k) {
        const uint32_t roll = pick(140);
        const int s = static_cast<int>(pick(shooters));
        const int t = chance(80) ? shooters + static_cast<int>(pick(targets)) : static_cast<int>(pick(n));
        const int x = base + static_cast<int>(pick(spread + 30));
        const int y = base + static_cast<int>(pick(spread + 30));
        const int z = static_cast<int>(pick(40));
        const int slot = static_cast<int>(pick(3));
        if (roll < 30) v.ops.push_back({0, s, slot, t, x, y, z, static_cast<int>(pick(4))});
        else if (roll < 54) v.ops.push_back({1, s, t});
        else if (roll < 62) v.ops.push_back({2, s, x, y, z});
        else if (roll < 68) v.ops.push_back({3, s, t});
        else if (roll < 80) v.ops.push_back({4, 1 + static_cast<int>(pick(chance(70) ? 8 : 90))});
        else if (roll < 83) v.ops.push_back({5, s, static_cast<int>(pick(2))});
        else if (roll < 87) v.ops.push_back({6, s, static_cast<int>(pick(3)), t, x, y});
        else if (roll < 93) v.ops.push_back({7, s, static_cast<int>(pick(4)), static_cast<int>(pick(2))});
        else if (roll < 96) v.ops.push_back({8, s, slot, 1 + static_cast<int>(pick(2))});
        else if (roll < 100) v.ops.push_back({9, static_cast<int>(pick(n)), x, y});
        // Whole and half unit offsets along an axis, so that range limits are met exactly.
        else if (roll < 110)
            v.ops.push_back({11, s, slot, t, static_cast<int>(pick(chance(70) ? 100 : 420)),
                             static_cast<int>(pick(2)), 0, static_cast<int>(pick(4))});
        else if (roll < 121)
            // The assistance ranges are among the offsets.
            v.ops.push_back({12, static_cast<int>(pick(n)), s,
                             chance(30) ? 40 + 80 * static_cast<int>(pick(3))
                                        : static_cast<int>(pick(chance(70) ? 100 : 420))});
        else if (roll < 124) v.ops.push_back({10, s});
        // The target is put at a range limit, or half a unit either side, and asked about.
        else if (roll < 134)
            v.ops.push_back({13, s, slot, t, static_cast<int>(pick(4)), static_cast<int>(pick(3)),
                             0, static_cast<int>(pick(4))});
        // The pre-attack starts exactly one frame before the weapon can fire, or two.
        else v.ops.push_back({14, s, t, 1 + static_cast<int>(pick(2))});
    }
}



static std::vector<Object*> all_objects() {
    std::vector<Object*> out;
    for (UnsignedInt id = 1; id < 200; ++id)
        if (Object* o = TheGameLogic->findObjectByID(static_cast<ObjectID>(id))) out.push_back(o);
    return out;
}

static std::string state(const std::string& answer) {
    std::ostringstream out;
    out << 'R' << GetGameLogicRandomSeedCRC() << ';' << answer << g_log.str();
    g_log.str("");
    for (Object* o : all_objects()) {
        const Coord3D* p = o->getPosition();
        out << 'O' << static_cast<UnsignedInt>(o->getID()) << ',' << bits(o->getBodyModule()->getHealth()) << ','
            << bits(p->x) << ',' << bits(p->y) << ',' << bits(p->z) << ',' << static_cast<int>(o->getIsUndetectedDefector());
        WeaponSlotType cur = PRIMARY_WEAPON;
        if (o->getCurrentWeapon(&cur)) out << ",c" << static_cast<int>(cur) << static_cast<int>(o->isCurWeaponLocked());
        for (int w = 0; w < WEAPONSLOT_COUNT; ++w) {
            Weapon* weapon = o->getWeaponInWeaponSlot(static_cast<WeaponSlotType>(w));
            if (weapon) out << ",w" << oracle_wp51::World::snapshot_save_hex(weapon);
        }
        for (const char* name : {"FiringTracker", "FireWeaponPower"})
            for (BehaviorModule** m = o->getBehaviorModules(); *m; ++m)
                if ((*m)->getModuleNameKey() == NAMEKEY(name)) {
                    out << ',' << name[4] << oracle_wp51::World::snapshot_save_hex(*m) << '.'
                        << oracle_wp51::World::snapshot_crc(*m);
                    // The reference's save omits these two members of the tracker.
                    if (name[4] == 'n') {
                        const FiringTracker* tracker = static_cast<const FiringTracker*>(static_cast<UpdateModule*>(*m));
                        out << '.' << tracker->*access_member(ForceReloadTag()) << '.'
                            << tracker->*access_member(StopSoundTag());
                    }
                }
        out << ';';
    }
    UnsignedInt mines = 0;
    for (int p = 0; p < 3; ++p)
        mines = mines * 100 + ThePlayerList->getNthPlayer(p)->getAcademyStats()->*access_member(MinesTag());
    out << 'M' << mines << ';';
    return out.str();
}

static std::string queries(Object* s, int slot, Object* t, const Coord3D& pos, Real angle) {
    Weapon* w = s->getWeaponInWeaponSlot(static_cast<WeaponSlotType>(slot));
    if (!w) w = s->getWeaponInWeaponSlot(PRIMARY_WEAPON);
    std::ostringstream out;
    out << 'Q' << bits(w->getAttackRange(s)) << ',' << bits(w->getPrimaryDamageRadius(s)) << ','
        << bits(w->getPercentReadyToFire()) << ',' << bits(w->getAttackDistance(s, t, &pos)) << ','
        << bits(w->getAttackDistance(s, nullptr, &pos));
    out << ",a" << static_cast<int>(w->isWithinAttackRange(s, &pos))
        << static_cast<int>(w->isTooClose(s, &pos))
        << static_cast<int>(w->isGoalPosWithinAttackRange(s, &pos, nullptr, &pos))
        << static_cast<int>(w->isSourceObjectWithGoalPositionWithinAttackRange(s, &pos, nullptr, t ? t->getPosition() : &pos));
    Coord3D approach;
    approach.zero();
    Bool done = w->computeApproachTarget(s, nullptr, &pos, angle, approach);
    out << ",p" << static_cast<int>(done) << '.' << bits(approach.x) << '.' << bits(approach.y) << '.' << bits(approach.z);
    out << ",d" << w->getPreAttackDelay(s, nullptr) << ",e" << bits(w->estimateWeaponDamage(s, &pos));
    out << ",l" << static_cast<int>(w->isClearFiringLineOfSightTerrain(s, pos))
        << static_cast<int>(w->isClearGoalFiringLineOfSightTerrain(s, pos, *s->getPosition()));
    if (t && t != s) {
        out << ",A" << static_cast<int>(w->isWithinAttackRange(s, t))
            << static_cast<int>(w->isTooClose(s, t))
            << static_cast<int>(w->isGoalPosWithinAttackRange(s, &pos, t, t->getPosition()))
            << static_cast<int>(w->isSourceObjectWithGoalPositionWithinAttackRange(s, &pos, t, t->getPosition()))
            << static_cast<int>(w->isWithinTargetPitch(s, t));
        approach.zero();
        done = w->computeApproachTarget(s, t, nullptr, angle, approach);
        out << ",P" << static_cast<int>(done) << '.' << bits(approach.x) << '.' << bits(approach.y) << '.' << bits(approach.z);
        out << ",D" << w->getPreAttackDelay(s, t) << ",E" << bits(w->estimateWeaponDamage(s, t));
        out << ",L" << static_cast<int>(w->isClearFiringLineOfSightTerrain(s, t))
            << static_cast<int>(w->isClearGoalFiringLineOfSightTerrain(s, pos, t));
        out << ",K" << static_cast<int>(w->getTemplate()->shouldProjectileCollideWith(nullptr, s, t, INVALID_ID));
    }
    out << ';';
    return out.str();
}

static SpecialPowerModuleInterface* find_power(Object* o) {
    for (BehaviorModule** m = o->getBehaviorModules(); *m; ++m)
        if ((*m)->getModuleNameKey() == NAMEKEY("FireWeaponPower")) return (*m)->getSpecialPower();
    return nullptr;
}

static void run(oracle_wp51::World& w, Vec& v) {
    TheWeaponStore->reset();
    g_log.str("");
    std::memset(g_sneaky, 0, sizeof(g_sneaky));
    for (int p = 0; p < 3; ++p)
        ThePlayerList->getNthPlayer(p)->getAcademyStats()->*access_member(MinesTag()) = 0;

    Team* teams[3] = {w.team_a, w.team_b, w.team_neutral};
    std::vector<ObjectID> ids;
    for (const Unit& u : v.units) {
        Coord3D p;
        p.x = static_cast<Real>(u.x);
        p.y = static_cast<Real>(u.y);
        p.z = S13Terrain::height(p.x, p.y) + static_cast<Real>(u.lift);
        Object* o = w.spawn_object(unit_name(u.type).c_str(), teams[u.team], p, kAngles[u.angle]);
        if (!o) die("spawn");
        // The roster was created before `spawn_object` placed the container.
        if (ContainModuleInterface* contain = o->getContain())
            if (const ContainedItemsList* riders = contain->getContainedItemsList())
                for (Object* rider : *riders) rider->setPosition(o->getPosition());
        static const WeaponBonusConditionType kConditions[4] = {
            WEAPONBONUSCONDITION_GARRISONED, WEAPONBONUSCONDITION_HORDE,
            WEAPONBONUSCONDITION_CONTINUOUS_FIRE_MEAN, WEAPONBONUSCONDITION_CONTINUOUS_FIRE_FAST};
        for (int b = 0; b < 4; ++b)
            if (u.bonus & (1 << b)) o->setWeaponBonusCondition(kConditions[b]);
        if (u.flags & 1) o->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_AIRBORNE_TARGET));
        if (u.flags & 2) o->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
        if (u.flags & 4) g_sneaky[static_cast<UnsignedInt>(o->getID())] = true;
        if (u.flags & 8) o->friend_setUndetectedDefector(TRUE);
        ids.push_back(o->getID());
    }
    ThePartitionManager->update();
    v.rows.push_back(state(""));

    UnsignedInt frame = v.frame;
    for (const std::vector<int>& op : v.ops) {
        std::string answer;
        auto obj = [&](int i) { return TheGameLogic->findObjectByID(ids[i]); };
        if (op[0] == 4) {
            frame += static_cast<UnsignedInt>(op[1]);
            w.set_frame(frame);
            TheWeaponStore->update();
        } else if (op[0] == 9) {
            if (Object* o = obj(op[1])) {
                Coord3D p;
                p.x = static_cast<Real>(op[2]);
                p.y = static_cast<Real>(op[3]);
                p.z = S13Terrain::height(p.x, p.y);
                o->setPosition(&p);
            }
        } else if (op[0] == 12) {
            Object* o = obj(op[1]);
            Object* anchor = obj(op[2]);
            if (o && anchor && o != anchor) {
                Coord3D p = *anchor->getPosition();
                p.x += static_cast<Real>(op[3]) * 0.5f;
                if (p.x < 990.0f) o->setPosition(&p);
            }
        } else if (Object* s = obj(op[1])) {
            switch (op[0]) {
            case 0: {
                Coord3D p;
                p.x = static_cast<Real>(op[4]) + 0.25f;
                p.y = static_cast<Real>(op[5]);
                p.z = static_cast<Real>(op[6]);
                if (s->getWeaponInWeaponSlot(PRIMARY_WEAPON))
                    answer = queries(s, op[2], obj(op[3]), p, kOffsets[op[7]]);
                break;
            }
            case 1: {
                Object* t = obj(op[2]);
                if (t && t != s) s->fireCurrentWeapon(t);
                break;
            }
            case 2: {
                Coord3D p;
                p.x = static_cast<Real>(op[2]);
                p.y = static_cast<Real>(op[3]);
                p.z = static_cast<Real>(op[4]);
                s->fireCurrentWeapon(&p);
                break;
            }
            case 3: {
                Object* t = obj(op[2]);
                if (t && t != s) s->preFireCurrentWeapon(t);
                break;
            }
            case 5: s->reloadAllAmmo(op[2] != 0); break;
            case 6: {
                SpecialPowerModuleInterface* power = find_power(s);
                if (!power) break;
                Coord3D p;
                p.x = static_cast<Real>(op[4]);
                p.y = static_cast<Real>(op[5]);
                p.z = 0.0f;
                if (op[2] == 0) power->doSpecialPower(0);
                else if (op[2] == 1) power->doSpecialPowerAtLocation(&p, 0.0f, 0);
                else if (obj(op[3])) power->doSpecialPowerAtObject(obj(op[3]), 0);
                break;
            }
            case 7: {
                static const WeaponBonusConditionType kConditions[4] = {
                    WEAPONBONUSCONDITION_GARRISONED, WEAPONBONUSCONDITION_HORDE,
                    WEAPONBONUSCONDITION_CONTINUOUS_FIRE_MEAN, WEAPONBONUSCONDITION_CONTINUOUS_FIRE_FAST};
                if (op[3]) s->setWeaponBonusCondition(kConditions[op[2]]);
                else s->clearWeaponBonusCondition(kConditions[op[2]]);
                break;
            }
            case 8:
                s->setWeaponLock(static_cast<WeaponSlotType>(op[2]),
                                 op[3] == 1 ? LOCKED_TEMPORARILY : LOCKED_PERMANENTLY);
                break;
            case 10: s->friend_setUndetectedDefector(TRUE); break;
            case 13: {
                Object* t = obj(op[3]);
                Weapon* weapon = s->getWeaponInWeaponSlot(static_cast<WeaponSlotType>(op[2]));
                if (!weapon) weapon = s->getWeaponInWeaponSlot(PRIMARY_WEAPON);
                if (!weapon || !t || t == s) break;
                Real limit = op[4] % 2 ? weapon->getTemplate()->getMinimumAttackRange()
                                       : weapon->getAttackRange(s);
                // The goal position question moves both limits by a quarter cell.
                if (op[4] == 2) limit -= 2.5f;
                if (op[4] == 3) limit += 2.5f;
                limit += static_cast<Real>(op[5] - 1) * 0.5f;
                const Real mine = s->getGeometryInfo().getBoundingCircleRadius();
                Coord3D p = *s->getPosition();
                p.x += limit + mine + t->getGeometryInfo().getBoundingCircleRadius();
                if (p.x >= 990.0f || p.x < 0.0f) break;
                t->setPosition(&p);
                ThePartitionManager->update();
                Coord3D at = *s->getPosition();
                at.x += limit + mine;
                answer = queries(s, op[2], t, at, kOffsets[op[7]]);
                break;
            }
            case 14: {
                Weapon* weapon = s->getCurrentWeapon();
                Object* t = obj(op[2]);
                if (!weapon || !t || t == s) break;
                const UnsignedInt next = weapon->getPossibleNextShotFrame();
                if (next < 0x7fffffffu && next >= frame + static_cast<UnsignedInt>(op[3])) {
                    frame = next - static_cast<UnsignedInt>(op[3]);
                    w.set_frame(frame);
                }
                s->preFireCurrentWeapon(t);
                break;
            }
            case 11: {
                Coord3D p = *s->getPosition();
                const Real d = static_cast<Real>(op[4]) * 0.5f;
                if (op[5]) p.y += d;
                else p.x += d;
                if (s->getWeaponInWeaponSlot(PRIMARY_WEAPON))
                    answer = queries(s, op[2], obj(op[3]), p, kOffsets[op[7]]);
                break;
            }
            default: die("op");
            }
        }
        w.flush_destroy_list();
        ThePartitionManager->update();
        v.rows.push_back(state(answer));
    }

    TheWeaponStore->reset();
    for (Object* o : all_objects()) TheGameLogic->destroyObject(o);
    w.flush_destroy_list();
    ThePartitionManager->update();
    g_log.str("");
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
    if (argc < 2) die("usage: gen_s13_weapon_firing <out.json>");
    const std::string out_path = argv[1];
    const int dump = std::getenv("S13_DUMP") ? std::atoi(std::getenv("S13_DUMP")) : -1;

    oracle_wp51::World w;
    TheTerrainLogic = NEW S13Terrain;
    static_cast<S13Terrain*>(TheTerrainLogic)->extent.lo = {0.0f, 0.0f, 0.0f};
    static_cast<S13Terrain*>(TheTerrainLogic)->extent.hi = {1000.0f, 1000.0f, 100.0f};
    ThePartitionManager->init();
    (TheModuleFactory->*(&ExposedFactory::addModuleInternal))(
        make_rec_ai, AIUpdateInterface::friend_newModuleData, MODULETYPE_BEHAVIOR, "OracleAI",
        AIUpdateInterface::getInterfaceMask());
    w.load_ini(kArmorIni);
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

    const int kTotal = 3000;
    // Rows of the first vectors are kept whole to locate a difference; the rest are hashed.
    const int kWhole = 25;
    const std::string tmp = out_path + ".tmp";
    FILE* f = std::fopen(tmp.c_str(), "w");
    if (!f) die("open");
    std::fprintf(f, "{\"vector_count\":%d,\n\"weapon_ini\":%s,\n\"object_ini\":%s,\n\"armor_ini\":%s,\n\"vectors\":[\n",
                 kTotal, json_string(weapons).c_str(), json_string(objects).c_str(), json_string(kArmorIni).c_str());
    for (int i = 0; i < kTotal; ++i) {
        Vec v;
        generate(v, i);
        if (std::getenv("S13_TRACE")) std::fprintf(stderr, "vector %d\n", i);
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
                 << ',' << n.angle << ',' << n.bonus << ',' << n.flags << ']';
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
