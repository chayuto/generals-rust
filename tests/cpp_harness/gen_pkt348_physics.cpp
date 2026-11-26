// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for PhysicsBehavior (PKT-348), run on the unmodified reference
// module in a small real world.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt348_physics.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/PKT348_Physics.json
//
// Terrain: flat at a per-vector height; with the vector's flags, x > 520 is cliff
// and y > 520 is water. The map is 0..1000.
//
// Each vector builds a few objects and applies operations to them. After every
// operation a row records the objects whose state changed: transform, module save
// bytes, health, status bits, next call frame, last collidee. NaN floats are
// written as 7fc00000 because their payloads are not part of the contract.
//
// Stand-ins: the crash weapons deal 1 damage to no one and are observed through
// their historic damage lists; a projectile update is faked by pointing m_pui at a stub.
//
// Not covered: bounce sounds (client audio), transferObjectName, a carrier deck
// with a height, bridges and other layers' heights, AI processCollision for moving
// units, objects without a drawable, a module's own crc() (it adds nothing),
// MinFallHeightForDamage under a gravity changed after INI load, contain modules
// calling into the physics of their own object, scheduler ordering between modules,
// two objects that can each crush the other, frames above 2^24, the crash weapons'
// own effects, a real projectile update.

#include "oracle_wp51_world.h"

#include "Common/Player.h"
#include "Common/Team.h"
#include "GameClient/Drawable.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/ScriptEngine.h"
#include "GameLogic/Weapon.h"

#include <cmath>
#include <cstdio>
#include <cstring>
#include <set>
#include <sstream>
#include <string>
#include <vector>

struct P348PuiTag { using type = ProjectileUpdateInterface* PhysicsBehavior::*; };
template struct MemberAccess<P348PuiTag, &PhysicsBehavior::m_pui>;
inline ProjectileUpdateInterface* PhysicsBehavior::* access_member(P348PuiTag);

struct P348HistoricTag { using type = HistoricWeaponDamageList WeaponTemplate::*; };
template struct MemberAccess<P348HistoricTag, &WeaponTemplate::m_historicDamage>;
inline HistoricWeaponDamageList WeaponTemplate::* access_member(P348HistoricTag);

// One letter per crash weapon fired: B the building default, N the other. Each
// fire leaves one entry in the template's historic damage list.
static std::string g_fired;
static void collect_fires() {
    static const char* const names[2] = {"VehicleCrashesIntoBuildingWeapon", "P348OtherCrash"};
    for (int k = 0; k < 2; ++k) {
        const WeaponTemplate* t = TheWeaponStore->findWeaponTemplate(names[k]);
        HistoricWeaponDamageList& list = const_cast<WeaponTemplate*>(t)->*access_member(P348HistoricTag());
        for (size_t n = 0; n < list.size(); ++n) g_fired += k ? 'N' : 'B';
        list.clear();
    }
}

struct P348NextIdTag { using type = ObjectID GameLogic::*; };
template struct MemberAccess<P348NextIdTag, &GameLogic::m_nextObjID>;
inline ObjectID GameLogic::* access_member(P348NextIdTag);

static float g_ground = 0.0f;
static bool g_cliff = false;
static bool g_water = false;

class P348Terrain : public oracle_wp51::OracleTerrainLogic {
public:
    Real getGroundHeight(Real, Real, Coord3D* normal = nullptr) const override {
        if (normal) { normal->x = 0.0f; normal->y = 0.0f; normal->z = 1.0f; }
        return g_ground;
    }
    Real getLayerHeight(Real x, Real y, PathfindLayerEnum, Coord3D* normal = nullptr, Bool = true) const override {
        return getGroundHeight(x, y, normal);
    }
    Bool isCliffCell(Real x, Real) const override { return g_cliff && x > 520.0f; }
    Bool isUnderwater(Real, Real y, Real* waterZ = nullptr, Real* terrainZ = nullptr) override {
        if (waterZ) *waterZ = 0.0f;
        if (terrainZ) *terrainZ = g_ground;
        return g_water && y > 520.0f;
    }
};

class P348Projectile : public ProjectileUpdateInterface {
public:
    Bool handled = FALSE;
    int calls = 0;
    void projectileLaunchAtObjectOrPosition(const Object*, const Coord3D*, const Object*, WeaponSlotType, Int,
                                            const WeaponTemplate*, const ParticleSystemTemplate*) override {}
    void projectileFireAtObjectOrPosition(const Object*, const Coord3D*, const WeaponTemplate*,
                                          const ParticleSystemTemplate*) override {}
    Bool projectileIsArmed() const override { return TRUE; }
    ObjectID projectileGetLauncherID() const override { return INVALID_ID; }
    Bool projectileHandleCollision(Object*) override { ++calls; return handled; }
    void setFramesTillCountermeasureDiversionOccurs(UnsignedInt) override {}
    void projectileNowJammed() override {}
};

static uint32_t g_state = 0x34800348u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}
// A value in [-range, range] on a grid of range/256.
static float spread(float range) { return range * (static_cast<float>(pick(513)) - 256.0f) / 256.0f; }
template <size_t N>
static float one_of(const float (&a)[N]) { return a[pick(N)]; }

static uint32_t bits(float f) {
    uint32_t u;
    std::memcpy(&u, &f, 4);
    if (std::isnan(f)) u = 0x7fc00000u;
    return u;
}
static std::string hx(float f) {
    char b[16];
    std::snprintf(b, sizeof b, "%08x", bits(f));
    return b;
}
static std::string hx3(const Coord3D& c) { return hx(c.x) + "," + hx(c.y) + "," + hx(c.z); }

static const char* const kNames[] = {
    "P348_Base",     // 0
    "P348_Bouncy",   // 1
    "P348_Heavy",    // 2
    "P348_Wall",     // 3  immobile structure without physics
    "P348_Rock",     // 4  immobile non-structure without physics
    "P348_Drone",    // 5
    "P348_Rester",   // 6
    "P348_Crusher",  // 7
    "P348_Crushee",  // 8
    "P348_Carrier",  // 9
    "P348_Walker",   // 10 AI, ground only
    "P348_Climber",  // 11 AI, ground water cliff
    "P348_Flinger",  // 12 dies into a fling
    "P348_Shell",    // 13 projectile kind
    "P348_Bunker",   // 14 immobile with physics
    "P348_Plain",    // 15 no physics, mobile
    "P348_Mortal",   // 16 destroyed on death
};
static const int kTemplates = 17;

static std::string object_ini(const std::string& name, const std::string& kind, const std::string& body_hp,
                              const std::string& geometry, const std::string& extra, const std::string& physics,
                              bool has_physics = true) {
    std::string s = "Object " + name + "\n  KindOf = PRELOAD " + kind + "\n" + geometry +
                    "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = " + body_hp + "\n    InitialHealth = " +
                    body_hp + "\n  End\n" + extra;
    if (has_physics) s += "  Behavior = PhysicsBehavior ModuleTag_Phys\n" + physics + "  End\n";
    s += "End\n";
    return s;
}

static std::string box(const char* major, const char* minor, const char* height) {
    return std::string("  Geometry = BOX\n  GeometryMajorRadius = ") + major + "\n  GeometryMinorRadius = " + minor +
           "\n  GeometryHeight = " + height + "\n";
}
static std::string cyl(const char* radius, const char* height) {
    return std::string("  Geometry = CYLINDER\n  GeometryMajorRadius = ") + radius + "\n  GeometryHeight = " + height +
           "\n";
}

static std::string fixture_ini() {
    std::string s;
    s += "Weapon VehicleCrashesIntoBuildingWeapon\n  PrimaryDamage = 1.0\n  PrimaryDamageRadius = 0.0\n"
         "  AttackRange = 100.0\n  HistoricBonusCount = 1000000\n  HistoricBonusTime = 100000\n  HistoricBonusRadius = 50.0\n  DelayBetweenShots = 1000\n  ClipSize = 0\nEnd\n";
    s += "Weapon P348OtherCrash\n  PrimaryDamage = 1.0\n  PrimaryDamageRadius = 0.0\n"
         "  AttackRange = 100.0\n  HistoricBonusCount = 1000000\n  HistoricBonusTime = 100000\n  HistoricBonusRadius = 50.0\n  DelayBetweenShots = 1000\n  ClipSize = 0\nEnd\n";
    s += "Locomotor P348Walk\n  Surfaces = GROUND\n  Speed = 30\nEnd\n";
    s += "Locomotor P348Climb\n  Surfaces = GROUND WATER CLIFF\n  Speed = 30\nEnd\n";

    s += object_ini("P348_Base", "VEHICLE", "1000000", box("10", "5", "8"), "", "");
    s += object_ini("P348_Bouncy", "INFANTRY", "1000000", cyl("4", "10"), "",
                    "    Mass = 5\n    AllowBouncing = Yes\n    ForwardFriction = 3\n    LateralFriction = 12\n"
                    "    ZFriction = 20\n    AerodynamicFriction = 1.5\n    CenterOfMassOffset = 1.5\n"
                    "    MinFallHeightForDamage = 5\n    FallHeightDamageFactor = 0.5\n    PitchRollYawFactor = 0.7\n"
                    "    ShockResistance = 0.3\n    ShockMaxYaw = 0.4\n    ShockMaxPitch = 0.3\n"
                    "    ShockMaxRoll = 0.2\n");
    s += object_ini("P348_Heavy", "VEHICLE", "1000000", box("14", "8", "12"), "  CrushableLevel = 3\n",
                    "    Mass = 50\n    AllowCollideForce = No\n    AerodynamicFriction = 45\n"
                    "    ForwardFriction = 0.01\n    LateralFriction = 90\n"
                    "    CenterOfMassOffset = -2\n    PitchRollYawFactor = 3.5\n    ShockResistance = 2.0\n"
                    "    VehicleCrashesIntoBuildingWeaponTemplate = P348OtherCrash\n"
                    "    VehicleCrashesIntoNonBuildingWeaponTemplate = VehicleCrashesIntoBuildingWeapon\n");
    s += object_ini("P348_Wall", "STRUCTURE IMMOBILE", "1000000", box("15", "15", "30"), "", "", false);
    s += object_ini("P348_Rock", "IMMOBILE", "1000000", cyl("9", "14"), "", "", false);
    s += object_ini("P348_Drone", "DRONE", "500", cyl("3", "4"), "", "    KillWhenRestingOnGround = Yes\n    CenterOfMassOffset = 0.5\n");
    s += object_ini("P348_Rester", "VEHICLE", "500", box("6", "6", "4"), "",
                    "    KillWhenRestingOnGround = Yes\n    AllowBouncing = Yes\n    Mass = 3\n"
                    "    VehicleCrashesIntoBuildingWeaponTemplate = None\n"
                    "    VehicleCrashesIntoNonBuildingWeaponTemplate = P348OtherCrash\n");
    s += object_ini("P348_Crusher", "VEHICLE", "1000000", box("12", "8", "8"),
                    "  CrusherLevel = 3\n  CrushableLevel = 2\n", "    Mass = 20\n");
    s += object_ini("P348_Crushee", "VEHICLE", "5000000", box("12", "6", "6"),
                    "  CrusherLevel = 0\n  CrushableLevel = 1\n", "    Mass = 4\n");
    s += object_ini("P348_Carrier", "VEHICLE", "1000000", box("16", "10", "10"),
                    "  Behavior = TransportContain ModuleTag_Contain\n    Slots = 10\n  End\n",
                    "    Mass = 30\n    ShockResistance = 0.5\n");
    const std::string ai = "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n";
    s += object_ini("P348_Walker", "INFANTRY", "800", cyl("4", "10"),
                    ai + "  Locomotor = SET_NORMAL P348Walk\n  Locomotor = SET_TAXIING P348Walk\n",
                    "    Mass = 2\n    AllowBouncing = Yes\n");
    s += object_ini("P348_Climber", "INFANTRY", "800", cyl("4", "10"),
                    ai + "  Locomotor = SET_NORMAL P348Climb\n  Locomotor = SET_TAXIING P348Climb\n",
                    "    Mass = 2\n    AllowBouncing = Yes\n    LateralFriction = 6\n");
    s += object_ini("P348_Flinger", "INFANTRY", "12", cyl("4", "10"),
                    "  Behavior = SlowDeathBehavior ModuleTag_Death\n    DeathTypes = ALL\n    FlingForce = 8\n"
                    "    FlingForceVariance = 4\n    FlingPitch = 60\n    FlingPitchVariance = 15\n"
                    "    DestructionDelay = 3000\n  End\n",
                    "    Mass = 3\n    MinFallHeightForDamage = 2\n    FallHeightDamageFactor = 2.0\n");
    s += object_ini("P348_Shell", "PROJECTILE", "1000000", cyl("2", "2"), "",
                    "    Mass = 0.5\n    MinFallHeightForDamage = 1\n");
    s += object_ini("P348_Bunker", "STRUCTURE IMMOBILE", "1000000", box("12", "12", "16"), "", "    Mass = 500\n");
    s += object_ini("P348_Plain", "VEHICLE", "1000000", box("8", "8", "8"), "", "", false);
    s += object_ini("P348_Mortal", "VEHICLE", "30", box("7", "4", "6"),
                    "  Behavior = DestroyDie ModuleTag_Destroy\n  End\n",
                    "    Mass = 4\n    MinFallHeightForDamage = 3\n    AllowBouncing = Yes\n");
    return s;
}

template <typename T>
static T* find_behavior(Object* obj) {
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m) {
        if (T* typed = dynamic_cast<T*>(*m)) return typed;
    }
    return nullptr;
}

struct Ent {
    int tmpl = 0;
    int team = 0;
    Object* obj = nullptr;
    PhysicsBehavior* phys = nullptr;
    P348Projectile pui;
    uint32_t ret = 0;
    std::string last;
};

static std::vector<Ent> g_ents;

static int index_of(ObjectID id) {
    if (id == INVALID_ID) return -1;
    for (size_t i = 0; i < g_ents.size(); ++i)
        if (g_ents[i].obj->getID() == id) return static_cast<int>(i);
    return -2;
}

// The save bytes, with NaN members made canonical.
static std::string phys_save(PhysicsBehavior* p) {
    std::string hex = oracle_wp51::World::snapshot_save_hex(p);
    // yaw, roll, pitch, accel, prevAccel, vel, [turning, ignore, flags], mass,
    // [overlaps, motive], bounciness, friction, velMag: the last 88 bytes.
    const size_t start = hex.size() - 88 * 2;
    static const int floats[] = {0, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 44, 60, 76, 80, 84};
    for (int off : floats) {
        const size_t at = start + off * 2;
        uint32_t u = 0;
        for (int b = 3; b >= 0; --b) u = (u << 8) | static_cast<uint32_t>(std::stoul(hex.substr(at + b * 2, 2), nullptr, 16));
        float f;
        std::memcpy(&f, &u, 4);
        if (std::isnan(f)) hex.replace(at, 8, "0000c07f");
    }
    return hex;
}

static std::string ent_row(Ent& e) {
    std::ostringstream o;
    Object* ob = e.obj;
    const Matrix3D* m = ob->getTransformMatrix();
    for (int r = 0; r < 3; ++r)
        for (int c = 0; c < 4; ++c) o << hx((*m)[r][c]) << (r == 2 && c == 3 ? "" : ",");
    const Drawable* d = ob->getDrawable();
    if (!d) {
        std::fprintf(stderr, "object without a drawable\n");
        std::exit(1);
    }
    const ModelConditionFlags& f = d->getModelConditionFlags();
    o << ' ' << hx(ob->getBodyModule()->getHealth()) << ' ' << (ob->isDestroyed() ? 1 : 0)
      << (ob->isEffectivelyDead() ? 1 : 0) << (ob->isDisabledByType(DISABLED_FREEFALL) ? 1 : 0)
      << (ob->isDisabledByType(DISABLED_UNMANNED) ? 1 : 0) << (f.test(MODELCONDITION_FREEFALL) ? 1 : 0)
      << (f.test(MODELCONDITION_SPLATTED) ? 1 : 0) << (f.test(MODELCONDITION_STUNNED) ? 1 : 0)
      << (f.test(MODELCONDITION_STUNNED_FLAILING) ? 1 : 0) << (ob->isOffMap() ? 1 : 0)
      << (ob->isCaptured() ? 1 : 0) << (ob->getTeam()->getControllingPlayer()->getPlayerIndex());
    if (e.phys) {
        UpdateModule* u = e.phys;
        o << ' ' << phys_save(e.phys) << ' ' << u->friend_getNextCallFrame() << ' ' << e.ret << ' '
          << index_of(e.phys->getLastCollidee()) << ' ' << e.pui.calls;
    }
    return o.str();
}

// "<changed objects> | <rng crc> <crash sounds so far>"
static std::string row(const std::string& query) {
    std::string out;
    for (size_t i = 0; i < g_ents.size(); ++i) {
        const std::string r = ent_row(g_ents[i]);
        if (r != g_ents[i].last) {
            out += std::to_string(i) + "=" + r + ";";
            g_ents[i].last = r;
        }
    }
    collect_fires();
    out += "|" + std::to_string(GetGameLogicRandomSeedCRC()) + " " + g_fired;
    if (!query.empty()) out += " " + query;
    return out;
}

static void step_all(oracle_wp51::World& w, UnsignedInt frame) {
    w.set_frame(frame);
    for (Ent& e : g_ents) {
        if (!e.phys || e.obj->isDestroyed()) continue;
        UpdateModule* u = e.phys;
        if (u->friend_getNextCallFrame() > frame) continue;
        UpdateSleepTime sleep_len = u->update();
        e.ret = static_cast<uint32_t>(sleep_len);
        if (sleep_len < 1) sleep_len = UPDATE_SLEEP_NONE;
        UnsignedInt next = frame + static_cast<UnsignedInt>(sleep_len);
        if (next > UPDATE_SLEEP_FOREVER) next = UPDATE_SLEEP_FOREVER;
        u->friend_setNextCallFrame(next);
    }
}

static void put_u32(std::string& hex, size_t at, uint32_t v) {
    static const char* const d = "0123456789abcdef";
    for (int b = 0; b < 4; ++b) {
        const unsigned byte = (v >> (8 * b)) & 0xff;
        hex[at + b * 2] = d[byte >> 4];
        hex[at + b * 2 + 1] = d[byte & 15];
    }
}

static std::vector<uint8_t> unhex(const std::string& h) {
    std::vector<uint8_t> out;
    for (size_t i = 0; i + 1 < h.size(); i += 2)
        out.push_back(static_cast<uint8_t>(std::stoul(h.substr(i, 2), nullptr, 16)));
    return out;
}

// A save stream with the same header and made-up members; sometimes version 1.
static std::string made_up_save(PhysicsBehavior* p) {
    std::string hex = oracle_wp51::World::snapshot_save_hex(p);
    const size_t start = hex.size() - 88 * 2;
    static const float vals[] = {0.0f, -1.0f, 1.0f, 0.25f, -3.5f, 0.0005f, 12.0f, -0.02f};
    for (int k = 0; k < 22; ++k) {
        const size_t at = start + k * 8;
        if (pick(3) != 0) continue;
        uint32_t v;
        if (k == 12) v = pick(3) - 1;                           // turning
        else if (k == 13 || k == 16 || k == 17) v = pick(5);    // ids
        else if (k == 14) v = pick(0x1000) & ~0x0400u;          // flags, not IS_IN_UPDATE
        else if (k == 15) v = bits(1.0f + static_cast<float>(pick(8)));  // mass
        else if (k == 18) v = pick(2) ? pick(60) : 0xffffffffu; // motive expiry
        else v = bits(one_of(vals));
        put_u32(hex, at, v);
    }
    if (pick(4) == 0) {
        hex[0] = '0';
        hex[1] = '1';
        // version 1 carries a discarded Coord3D after the velocity
        hex.insert(start + 48 * 2, "0000803f0000004000004040");
    }
    return hex;
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_pkt348_physics <out.json>\n");
        return 2;
    }
    oracle_wp51::World w;
    TheTerrainLogic = NEW P348Terrain;
    TheTerrainLogic->init();
    if (!TheScriptEngine) {
        TheScriptEngine = NEW ScriptEngine;
        TheScriptEngine->init();
    }
    TheWritableGlobalData->m_gravity = -1.0f;
    w.load_ini(fixture_ini());
    w.init_with_neutral_player();

    const std::string tmp = std::string(argv[1]) + ".tmp";
    FILE* out = std::fopen(tmp.c_str(), "w");
    if (!out) return 1;
    const int kTotal = 1200;
    std::fprintf(out, "{\"vector_count\":%d,\"vectors\":[", kTotal);

    static const float gravities[] = {-1.0f, -1.0f, -64.0f / 900.0f, -2.5f, -0.3f};
    static const float stiffs[] = {0.5f, 0.0f, 2.0f, 0.2f, 0.9f};
    static const float grounds[] = {0.0f, 0.0f, 5.5f, -3.0f};
    static const float rubbles[] = {1.0f, 1.0f, 20.0f};
    static const float heights[] = {0.0f, 0.0f, 0.5f, 2.0f, 8.0f, 9.5f, 14.0f, 40.0f, -1.0f};
    static const int subjects[] = {0, 1, 2, 5, 6, 7, 9, 10, 11, 12, 13, 16, 1, 7, 0, 10};
    static const int others[] = {0, 1, 3, 4, 8, 8, 8, 9, 14, 15, 7, 3, 10, 12, 2, 3};

    for (int i = 0; i < kTotal; ++i) {
        g_ground = one_of(grounds);
        g_cliff = pick(3) == 0;
        g_water = pick(3) == 0;
        // Stunned and falling at exactly the relief speed.
        const bool relief = i % 32 == 10;
        const float gravity = relief ? -0.3f : one_of(gravities);
        const float stiff = one_of(stiffs);
        const float sstiff = one_of(stiffs);
        const float rubble = one_of(rubbles);
        const uint32_t seed = pick(0x7fffffffu);
        // Below 2^24: above it the temporary crash weapon rounds `frame + reload` and may not fire.
        UnsignedInt frame = pick(5) == 0 ? 0x00100000u + pick(0x00e00000u) : 1 + pick(500);
        w.set_frame(frame);
        TheWritableGlobalData->m_gravity = -1.0f;
        TheWritableGlobalData->m_groundStiffness = stiff;
        TheWritableGlobalData->m_structureStiffness = sstiff;
        TheWritableGlobalData->m_defaultStructureRubbleHeight = rubble;
        TheGameLogic->*access_member(P348NextIdTag()) = static_cast<ObjectID>(1);
        g_fired.clear();

        // The "duel" vectors put two or three objects within reach of each other.
        const int count = 2 + static_cast<int>(pick(3));
        const bool crush_duel = i % 8 == 3;
        const bool wall_drop = i % 8 == 5;
        const bool capture = i % 16 == 1;
        const bool cargo = i % 16 == 7;
        const bool stunned = i % 8 == 2;
        // Near an edge, or across the cliff and water lines.
        const float cx = pick(10) == 0 ? (pick(2) ? 2.0f : 998.0f) : (pick(stunned ? 4 : 2) == 0 ? 300.0f : 522.0f);
        const float cy = pick(stunned ? 4 : 2) == 0 ? 300.0f : 522.0f;
        g_ents.clear();
        g_ents.resize(count);
        bool at_rubble = false;
        std::ostringstream setup;
        setup << hx(g_ground) << ' ' << g_cliff << ' ' << g_water << ' ' << hx(gravity) << ' ' << hx(stiff) << ' '
              << hx(sstiff) << ' ' << hx(rubble) << ' ' << seed << ' ' << frame;
        for (int k = 0; k < count; ++k) {
            Ent& e = g_ents[k];
            e.tmpl = k == 0 ? subjects[pick(16)] : others[pick(16)];
            if (crush_duel && k < 2) e.tmpl = k == 0 ? 7 : 8;
            if (wall_drop && k == 1) e.tmpl = pick(4) == 0 ? 4 : (pick(4) == 0 ? 14 : 3);
            if (capture && k < 2) e.tmpl = k == 0 ? 10 + static_cast<int>(pick(2)) : (pick(3) == 0 ? 8 : 0);
            if (cargo) e.tmpl = k == 0 ? 9 : (pick(2) ? 1 : (pick(2) ? 10 : 0));
            if (cargo && k == 2 && pick(2)) e.tmpl = 3;
            if (stunned && k == 0) {
                static const int walkers[] = {10, 10, 11, 1, 6, 16};
                e.tmpl = walkers[pick(6)];
            }
            e.team = k == 0 ? 0 : static_cast<int>(pick(4) != 0);
            Coord3D pos{cx + (k ? spread(18.0f) : 0.0f), cy + (k ? spread(18.0f) : 0.0f), g_ground};
            if (k == 0 || pick(3) == 0) pos.z += one_of(heights);
            if (crush_duel && k < 2) pos.z = g_ground;
            if (wall_drop && k == 0) pos.z = g_ground + 8.0f + static_cast<float>(pick(30));
            if (stunned && k == 0) pos.z = g_ground + (pick(3) == 0 ? 0.0f : static_cast<float>(pick(10)));
            if (relief && k == 0) pos.z = g_ground + 40.0f;
            // Exactly at the rubble height.
            if (wall_drop && k == 0 && pick(4) == 0) {
                pos.z = rubble;
                at_rubble = true;
            }
            if (crush_duel && k == 1 && pick(8) == 0) e.tmpl = 2;
            float angle = pick(4) == 0 ? 0.0f : spread(3.0f);
            if (crush_duel && k == 1 && pick(2)) angle = 0.0f;
            e.obj = w.spawn_object(kNames[e.tmpl], e.team ? w.team_b : w.team_a, pos, angle);
            if (!e.obj) {
                std::fprintf(stderr, "no object %s\n", kNames[e.tmpl]);
                return 1;
            }
            e.phys = find_behavior<PhysicsBehavior>(e.obj);
            setup << ' ' << e.tmpl << ' ' << e.team << ' ' << hx3(pos) << ' ' << hx(angle);
        }
        TheWritableGlobalData->m_gravity = gravity;
        InitRandom(seed);

        std::vector<std::string> ops;
        std::vector<std::string> rows;
        rows.push_back(row(""));

        auto vec = [&](float range) {
            Coord3D v{spread(range), spread(range), spread(range)};
            if (pick(6) == 0) { v.x = 0.0f; v.y = 0.0f; }
            if (pick(12) == 0) v.z = 0.0f;
            if (pick(40) == 0) v.x = -0.0f;
            if (pick(30) == 0) { v.x = 0.0004f; v.y = -0.0009f; v.z = 0.0005f; }
            return v;
        };

        // Scripted openings for the rarer paths.
        auto done = [&](const std::string& op) {
            ops.push_back(op);
            rows.push_back(row(""));
        };
        auto steps = [&](uint32_t n) {
            for (uint32_t f = 0; f < n; ++f) step_all(w, ++frame);
            done("S " + std::to_string(n));
        };
        auto set_bit = [&](int a, uint32_t which, int on) {
            PhysicsBehavior* p = g_ents[a].phys;
            switch (which) {
                case 0: p->setAllowToFall(on); break;
                case 2: p->setAllowBouncing(on); break;
                case 6: p->setStunned(on); break;
                default: break;
            }
            done("B " + std::to_string(a) + " " + std::to_string(which) + " " + std::to_string(on));
        };
        auto add_v = [&](int a, const Coord3D& v) {
            if (!g_ents[a].obj->isDestroyed()) g_ents[a].phys->addVelocityTo(&v);
            done("V " + std::to_string(a) + " " + hx3(v));
        };
        auto collide = [&](int a, int b) {
            Object* me = g_ents[a].obj;
            Object* them = g_ents[b].obj;
            if (!me->isDestroyed() && !them->isDestroyed()) {
                Coord3D loc = *me->getPosition();
                g_ents[a].phys->onCollide(them, &loc, nullptr);
            }
            done("L " + std::to_string(a) + " " + std::to_string(b));
        };
        auto parachute = [&](int a, int b) {
            const auto status = MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_PARACHUTING);
            if (!g_ents[a].obj->isDestroyed()) g_ents[a].obj->setStatus(status, TRUE);
            if (b >= 0 && !g_ents[b].obj->isDestroyed()) g_ents[b].obj->setStatus(status, TRUE);
            done("U " + std::to_string(a) + " 1 1 " + std::to_string(b));
        };
        if (crush_duel && pick(3) == 0) {
            const int front = pick(2);
            const int back = pick(2);
            g_ents[1].obj->getBodyModule()->setFrontCrushed(front);
            g_ents[1].obj->getBodyModule()->setBackCrushed(back);
            done("Z 1 " + std::to_string(front) + " " + std::to_string(back));
        }
        if (i % 8 == 4) {
            // Scrubbing to exactly the current speed still drops the cached magnitude.
            add_v(0, Coord3D{3.0f, 0.0f, 0.0f});
            g_ents[0].phys->scrubVelocity2D(3.0f);
            done("C 0 0 " + hx(3.0f));
        }
        if (relief) {
            set_bit(0, 6, 1);
            add_v(0, pick(2) ? Coord3D{0.5f, 0.0f, 0.0f} : Coord3D{0.0f, -0.5f, 0.0f});
            steps(1);
        }
        if (at_rubble) collide(0, 1);
        if (capture) {
            // Infantry walking to an unmanned vehicle.
            g_ents[0].obj->getAI()->ignoreObstacle(g_ents[1].obj);
            done("IG 0 1");
            if (pick(5) != 0) {
                g_ents[1].obj->setDisabled(DISABLED_UNMANNED);
                done("H 1 0 1");
            }
            collide(pick(4) == 0 ? 1 : 0, pick(4) == 0 ? 0 : 1);
            collide(0, 1);
        }
        if (cargo) {
            for (int r = 1; r < count; ++r) {
                if (!g_ents[r].phys) continue;
                g_ents[0].obj->getContain()->addToContain(g_ents[r].obj);
                done("N 0 " + std::to_string(r));
            }
            if (pick(3) == 0 && count > 2) {
                parachute(1, -1);
                collide(1, 2);
            }
        }
        if (stunned && !relief) {
            set_bit(0, 6, 1);
            if (pick(3) != 0) set_bit(0, 2, 1);
            if (pick(4) == 0) set_bit(0, 0, 1);
            if (pick(4) == 0) {
                const float yaw = spread(3.0f);
                g_ents[0].phys->setAngles(yaw, 0.0f, 3.1f);
                done("O 0 " + hx(yaw) + " " + hx(0.0f) + " " + hx(3.1f));
            }
            add_v(0, Coord3D{pick(2) ? spread(2.0f) : 0.0f, 0.0f, -static_cast<float>(pick(12))});
            steps(1);
            steps(1 + pick(6));
        }
        if (wall_drop && pick(3) == 0) {
            parachute(0, pick(3) == 0 ? 1 : -1);
            add_v(0, Coord3D{spread(3.0f), spread(3.0f), -1.0f});
            collide(0, 1);
        }

        const int nops = 5 + static_cast<int>(pick(12));
        for (int k = 0; k < nops; ++k) {
            const int a = pick(5) < 3 ? 0 : static_cast<int>(pick(count));
            const int b = static_cast<int>(pick(count));
            Ent& e = g_ents[a];
            Ent& t = g_ents[b];
            PhysicsBehavior* p = e.phys;
            std::ostringstream op;
            std::string query;
            uint32_t code = pick(100);
            bool crush_op = false;
            if (crush_duel && k < 7) {
                // Drive the crusher at the crushee.
                crush_op = true;
            }
            if (crush_op) {
                Ent& me = g_ents[0];
                Ent& them = g_ents[1];
                const Coord3D* tp = them.obj->getPosition();
                const Coord3D* td = them.obj->getUnitDirectionVector2D();
                if (k % 2 == 0) {
                    // Place the crusher relative to one of the three crush points.
                    const float half = them.obj->getGeometryInfo().getMajorRadius() / 2;
                    const float along = static_cast<float>(static_cast<int>(pick(3)) - 1) * half;
                    const float heading = pick(4) == 0 ? spread(3.0f) : static_cast<float>(pick(4)) * 1.5707964f;
                    const float off = pick(3) == 0 ? 0.0f : spread(pick(2) ? 12.0f : 0.5f);
                    const float past = pick(3) == 0 ? spread(1.0f) : spread(14.0f);
                    Coord3D pos{tp->x + td->x * along + WWMath::Cosf(heading) * past - WWMath::Sinf(heading) * off,
                                tp->y + td->y * along + WWMath::Sinf(heading) * past + WWMath::Cosf(heading) * off,
                                g_ground};
                    float turned = heading;
                    if (td->y == 0.0f && pick(2)) {
                        // Halfway between two crush points, so their distances tie.
                        static const float sides[] = {0.0f, 0.0f, 0.125f, -0.125f, 0.05f, -0.05f, 0.015625f, 1.0f, -4.0f};
                        pos.x = tp->x + static_cast<float>(static_cast<int>(pick(3)) - 1) * half * 0.5f;
                        pos.y = tp->y + one_of(sides);
                        static const float headings[] = {0.0f, 0.0f, 1.5707964f, 0.01f, -0.01f, 0.03f, -0.03f, 0.045f, -0.045f};
                        turned = one_of(headings);
                    }
                    if (!me.obj->isDestroyed()) {
                        me.obj->setPosition(&pos);
                        me.obj->setOrientation(turned);
                        Coord3D v{pick(8) == 0 ? 0.005f : 3.0f * WWMath::Cosf(turned), pick(8) == 0 ? 0.0f : 3.0f * WWMath::Sinf(turned), 0.0f};
                        // Exactly on the "very small" threshold.
                        if (pick(8) == 0) v = Coord3D{0.01f, 0.0f, 0.0f};
                        me.phys->addVelocityTo(&v);
                    }
                        // Record the velocity actually added.
                    Coord3D added = me.obj->isDestroyed() ? Coord3D{0, 0, 0} : *me.phys->getVelocity();
                    op << "CR " << hx3(pos) << ' ' << hx(turned) << ' ' << hx3(added);
                } else {
                    if (!me.obj->isDestroyed() && !them.obj->isDestroyed()) {
                        me.phys->onCollide(them.obj, me.obj->getPosition(), nullptr);
                    }
                    op << "L 0 1";
                }
                // CR stores the resulting velocity, so replay sets it exactly.
                ops.push_back(op.str());
                rows.push_back(row(""));
                continue;
            }
            const bool dead = e.obj->isDestroyed();
            if (code < 30 || !p) {
                const uint32_t n = 1 + (pick(3) == 0 ? pick(40) : pick(4));
                op << "S " << n;
                for (uint32_t f = 0; f < n; ++f) step_all(w, ++frame);
            } else if (code < 36) {
                Coord3D v = vec(pick(2) ? 12.0f : 200.0f);
                if (pick(15) == 0) (pick(3) == 0 ? v.x : (pick(2) ? v.y : v.z)) = std::nanf("");
                op << "F " << a << ' ' << hx3(v);
                w.set_frame(frame);
                if (!dead) p->applyForce(&v);
            } else if (code < 40) {
                const Coord3D v = vec(40.0f);
                op << "M " << a << ' ' << hx3(v);
                if (!dead) p->applyMotiveForce(&v);
            } else if (code < 43) {
                const Coord3D v = vec(60.0f);
                op << "K " << a << ' ' << hx3(v);
                if (!dead) p->applyShock(&v);
            } else if (code < 46) {
                op << "R " << a;
                if (!dead) p->applyRandomRotation();
            } else if (code < 56) {
                Coord3D v = vec(6.0f);
                if (pick(3) == 0) v.z = -static_cast<float>(pick(40));
                if (pick(80) == 0) v.y = std::nanf("");
                op << "V " << a << ' ' << hx3(v);
                if (!dead) p->addVelocityTo(&v);
            } else if (code < 59) {
                const uint32_t which = pick(3);
                const float r = pick(4) == 0 ? 0.0f : spread(0.6f);
                op << "Y " << a << ' ' << which << ' ' << hx(r);
                if (!dead) {
                    if (which == 0) p->setYawRate(r);
                    else if (which == 1) p->setPitchRate(r);
                    else p->setRollRate(r);
                }
            } else if (code < 66) {
                const uint32_t which = pick(8);
                const int on = pick(4) != 0;
                op << "B " << a << ' ' << which << ' ' << on;
                if (!dead) {
                    switch (which) {
                        case 0: p->setAllowToFall(on); break;
                        case 1: p->setStickToGround(on); break;
                        case 2: p->setAllowBouncing(on); break;
                        case 3: p->setAllowCollideForce(on); break;
                        case 4: p->setAllowAirborneFriction(on); break;
                        case 5: p->setImmuneToFallingDamage(on); break;
                        case 6: p->setStunned(on); break;
                        default: p->setIsInFreeFall(on); break;
                    }
                }
            } else if (code < 68) {
                static const float extras[] = {0.0f, -0.1f, 0.5f, 2.0f, -3.0f, 0.99f};
                const float v = one_of(extras);
                const uint32_t which = pick(2);
                op << "E " << a << ' ' << which << ' ' << hx(v);
                if (!dead) {
                    if (which) p->setExtraFriction(v);
                    else p->setExtraBounciness(v);
                }
            } else if (code < 70) {
                static const float scrubs[] = {0.0f, 0.0005f, 0.001f, 0.5f, 3.0f, -3.0f, -0.5f, 100.0f, -0.0005f};
                const float v = one_of(scrubs);
                const uint32_t which = pick(2);
                op << "C " << a << ' ' << which << ' ' << hx(v);
                if (!dead) {
                    if (which) p->scrubVelocityZ(v);
                    else p->scrubVelocity2D(v);
                }
            } else if (code < 71) {
                op << "D " << a;
                if (!dead) p->resetDynamicPhysics();
            } else if (code < 74) {
                const Coord3D* at = t.obj->getPosition();
                Coord3D pos{at->x + spread(14.0f), at->y + spread(14.0f), g_ground + one_of(heights)};
                if (pick(15) == 0) pos.x = pick(2) ? 0.0f : 1000.0f;
                op << "P " << a << ' ' << hx3(pos);
                if (!dead) e.obj->setPosition(&pos);
            } else if (code < 76) {
                const float yaw = spread(3.0f);
                const float pitch = pick(2) ? 0.0f : spread(1.5f);
                const float roll = pick(3) == 0 ? 3.0f + spread(0.4f) : (pick(2) ? 0.0f : spread(1.5f));
                op << "O " << a << ' ' << hx(yaw) << ' ' << hx(pitch) << ' ' << hx(roll);
                if (!dead) p->setAngles(yaw, pitch, roll);
            } else if (code < 84) {
                const int other = pick(10) == 0 ? -1 : (b == a ? (a + 1) % count : b);
                op << "L " << a << ' ' << other;
                Object* o = other < 0 ? nullptr : g_ents[other].obj;
                if (!dead && !(o && o->isDestroyed())) {
                    Coord3D loc = *e.obj->getPosition();
                    p->onCollide(o, &loc, nullptr);
                }
            } else if (code < 85) {
                op << "T " << a << ' ' << b;
                if (!dead) p->transferVelocityTo(t.phys);
            } else if (code < 86) {
                const int other = pick(3) == 0 ? -1 : b;
                op << "I " << a << ' ' << other;
                if (!dead) p->setIgnoreCollisionsWith(other < 0 ? nullptr : g_ents[other].obj);
            } else if (code < 87) {
                const float m = 0.5f + static_cast<float>(pick(40));
                op << "W " << a << ' ' << hx(m);
                if (!dead) p->setMass(m);
            } else if (code < 88) {
                if (pick(2)) {
                    op << "A " << a;
                    if (!dead) p->clearAcceleration();
                } else {
                    const int turning = static_cast<int>(pick(3)) - 1;
                    op << "J " << a << ' ' << turning;
                    if (!dead) p->setTurning(static_cast<PhysicsTurningType>(turning));
                }
            } else if (code < 91) {
                static const ObjectStatusTypes kinds[] = {OBJECT_STATUS_BRAKING, OBJECT_STATUS_PARACHUTING,
                                                          OBJECT_STATUS_DECK_HEIGHT_OFFSET};
                const uint32_t which = pick(3);
                const int on = pick(4) != 0;
                const int both = which == 1 && pick(2);
                op << "U " << a << ' ' << which << ' ' << on << ' ' << (both ? b : -1);
                if (!dead) e.obj->setStatus(MAKE_OBJECT_STATUS_MASK(kinds[which]), on);
                if (both && !t.obj->isDestroyed()) t.obj->setStatus(MAKE_OBJECT_STATUS_MASK(kinds[which]), on);
            } else if (code < 92) {
                const int held = pick(2);
                const int on = pick(3) != 0;
                const int target = pick(2) ? a : b;
                op << "H " << target << ' ' << held << ' ' << on;
                Object* o = g_ents[target].obj;
                const DisabledType type = held ? DISABLED_HELD : DISABLED_UNMANNED;
                if (!o->isDestroyed()) {
                    if (on) o->setDisabled(type);
                    else o->clearDisabled(type);
                }
            } else if (code < 93) {
                const int front = pick(2);
                const int back = pick(2);
                op << "Z " << b << ' ' << front << ' ' << back;
                if (!t.obj->isDestroyed()) {
                    t.obj->getBodyModule()->setFrontCrushed(front);
                    t.obj->getBodyModule()->setBackCrushed(back);
                }
            } else if (code < 94) {
                // Put an object inside a carrier.
                int carrier = -1, rider = -1;
                for (int c = 0; c < count; ++c)
                    if (g_ents[c].tmpl == 9 && !g_ents[c].obj->isDestroyed()) carrier = c;
                for (int c = 0; c < count; ++c) {
                    const int tm = g_ents[c].tmpl;
                    if (c != carrier && (tm == 1 || tm == 10 || tm == 11 || tm == 0) && !g_ents[c].obj->isDestroyed() &&
                        !g_ents[c].obj->getContainedBy())
                        rider = c;
                }
                if (carrier >= 0 && rider >= 0) {
                    op << "N " << carrier << ' ' << rider;
                    g_ents[carrier].obj->getContain()->addToContain(g_ents[rider].obj);
                } else {
                    op << "G " << a;
                }
                if (op.str()[0] == 'G') query = "q";
            } else if (code < 96) {
                // An AI object: ignore an obstacle, or change locomotor set.
                int walker = -1;
                for (int c = 0; c < count; ++c)
                    if ((g_ents[c].tmpl == 10 || g_ents[c].tmpl == 11) && !g_ents[c].obj->isDestroyed()) walker = c;
                if (walker < 0) {
                    op << "S 1";
                    step_all(w, ++frame);
                } else if (pick(3) == 0) {
                    const int taxi = pick(2);
                    op << "X " << walker << ' ' << taxi;
                    g_ents[walker].obj->getAI()->chooseLocomotorSet(taxi ? LOCOMOTORSET_TAXIING : LOCOMOTORSET_NORMAL);
                    InitRandom(seed + k);
                    op << ' ' << (seed + k);
                } else {
                    const int other = walker == 0 ? 1 : 0;
                    op << "IG " << walker << ' ' << other;
                    g_ents[walker].obj->getAI()->ignoreObstacle(g_ents[other].obj);
                }
            } else if (code < 97) {
                const int handled = pick(2);
                const int on = pick(4) != 0;
                op << "PU " << a << ' ' << handled << ' ' << on;
                if (!dead) {
                    e.pui.handled = handled;
                    p->*access_member(P348PuiTag()) = on ? &e.pui : nullptr;
                }
            } else if (code < 98) {
                if (dead) {
                    op << "S 1";
                    step_all(w, ++frame);
                } else {
                    const std::string stream = made_up_save(p);
                    op << "LD " << a << ' ' << stream;
                    // A load forgets the place in the sleepy heap, which a real load rebuilds.
                    UpdateModule* u = p;
                    const Int at = u->friend_getIndexInLogic();
                    oracle_wp51::World::snapshot_load_bytes(p, unhex(stream));
                    u->friend_setIndexInLogic(at);
                }
            } else if (code < 99) {
                op << "AO " << a << ' ' << b;
                if (!dead && !t.obj->isDestroyed()) p->addOverlap(t.obj);
            } else {
                const int layer = pick(2);
                op << "LY " << a << ' ' << layer;
                if (!dead) e.obj->setLayer(layer ? static_cast<PathfindLayerEnum>(LAYER_GROUND + 1) : LAYER_GROUND);
            }
            // Queries cache the velocity magnitude, which is saved.
            if (query == "q" || (pick(7) == 0 && p && !e.obj->isDestroyed())) {
                if (p && !e.obj->isDestroyed()) {
                    std::ostringstream q;
                    q << "Q" << a;
                    op << " ?" << a;
                    q << ' ' << hx(p->getVelocityMagnitude()) << ' ' << hx(p->getForwardSpeed2D()) << ' '
                      << hx(p->getForwardSpeed3D()) << ' ' << hx(p->getMass()) << ' ' << (p->isMotive() ? 1 : 0) << ' '
                      << hx(p->getCenterOfMassOffset()) << ' ' << (p->getIsStunned() ? 1 : 0)
                      << (p->getAllowToFall() ? 1 : 0);
                    query = q.str();
                } else {
                    query.clear();
                }
            }
            ops.push_back(op.str());
            rows.push_back(row(query));
        }
        // Let everything settle, so sleeping and landing are reached.
        for (int k = 0; k < 3; ++k) {
            const uint32_t n = k == 2 ? 60 : 3;
            ops.push_back("S " + std::to_string(n));
            for (uint32_t f = 0; f < n; ++f) step_all(w, ++frame);
            rows.push_back(row(""));
        }

        std::fprintf(out, "%s\n{\"v\":%d,\"setup\":\"%s\",\"ops\":[", i ? "," : "", i, setup.str().c_str());
        for (size_t k = 0; k < ops.size(); ++k) std::fprintf(out, "%s\"%s\"", k ? "," : "", ops[k].c_str());
        std::fprintf(out, "],\"rows\":[");
        for (size_t k = 0; k < rows.size(); ++k) std::fprintf(out, "%s\"%s\"", k ? "," : "", rows[k].c_str());
        std::fprintf(out, "]}");

        w.set_frame(frame);
        for (Ent& e : g_ents)
            if (e.phys) e.phys->*access_member(P348PuiTag()) = nullptr;
        for (Ent& e : g_ents)
            if (!e.obj->isDestroyed()) TheGameLogic->destroyObject(e.obj);
        w.flush_destroy_list();
        g_ents.clear();
    }
    std::fprintf(out, "\n]}\n");
    if (std::fclose(out) != 0) return 1;
    return std::rename(tmp.c_str(), argv[1]) == 0 ? 0 : 1;
}
