// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for the movement half of Locomotor.cpp (PKT-143, AI plan slice S7),
// run on the unmodified reference locomotor, physics module and object in a small real
// world.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_s7_locomotor_movement.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/S7_LocomotorMovement.json
//
// Two library functions that are not ported yet are answered here, with synthetic
// results, and every call the reference locomotor makes to them is recorded: the tape
// the Rust suite replays. The synthetic terrain overrides TerrainLogic virtuals only.
//
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIPathfind.cpp.o __ZN10Pathfinder20validMovementTerrainE17PathfindLayerEnumPK9LocomotorPK7Coord3D
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a PartitionManager.cpp.o __ZN16PartitionManager26getGroundOrStructureHeightEff
//
// Not covered:
//  - a tracked mover with a turn pivot offset whose goal is within 0.1 of the pivot:
//    the reference then reads an uninitialized `relAngle` (goals are moved away);
//  - an object that ends at a NaN position (the vector stops before comparing it);
//  - a walker or climber with a positive minimum speed: `moveTowardsPositionLegs` calls
//    `locoUpdate_moveTowardsAngle`, which for such a locomotor moves towards a position
//    again, until the stack overflows;
//  - a wander length factor of zero (an infinite offset increment);
//  - a producer with a landing deck, so the deck height status subtracts 0;
//  - bridges: the highest layer for a destination is always the ground;
//  - objects without physics, loading a save, and LocomotorSet.

#include "PreRTS.h"

#include "Common/AsciiString.h"
#include "Common/GameCommon.h"
#include "Common/GameMemory.h"
#include "Common/Snapshot.h"
#include <map>
#include <vector>
#define private public
#define protected public
#include "GameLogic/Locomotor.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#undef protected
#undef private

#include "oracle_wp51_world.h"

#include "GameClient/Drawable.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/PartitionManager.h"

#include <cmath>
#include <cstdio>
#include <cstring>
#include <sstream>
#include <string>

static uint32_t g_state = 0x14300007u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}
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
static std::string hx3(const Coord3D& c) { return hx(c.x) + " " + hx(c.y) + " " + hx(c.z); }

// ---------------------------------------------------------------------------
// Synthetic terrain
// ---------------------------------------------------------------------------

static float g_base = 0.0f, g_slope = 0.0f, g_step = 0.0f, g_water_z = 0.0f;
static int g_water = 0;
static uint32_t g_rate = 0, g_salt = 0;

class S7Terrain : public oracle_wp51::OracleTerrainLogic {
public:
    Real getGroundHeight(Real x, Real y, Coord3D* normal = nullptr) const override {
        if (normal) { normal->x = 0.0f; normal->y = 0.0f; normal->z = 1.0f; }
        Real h = g_base;
        if (x > 500.0f) h += (x - 500.0f) * g_slope;
        if (y > 506.0f) h += g_step;
        return h;
    }
    Real getLayerHeight(Real x, Real y, PathfindLayerEnum layer, Coord3D* normal = nullptr, Bool = true) const override {
        const Real h = getGroundHeight(x, y, normal);
        return layer == LAYER_GROUND ? h : h + 7.0f;
    }
    Bool isUnderwater(Real x, Real y, Real* waterZ = nullptr, Real* terrainZ = nullptr) override {
        if (waterZ) *waterZ = g_water_z;
        if (terrainZ) *terrainZ = getGroundHeight(x, y, nullptr);
        return g_water && x < 504.0f;
    }
};

// ---------------------------------------------------------------------------
// The tape
// ---------------------------------------------------------------------------

static bool g_recording = false;
static std::string g_tape;

static uint32_t cell_hash(Int x, Int y) {
    uint32_t h = static_cast<uint32_t>(x) * 73856093u ^ static_cast<uint32_t>(y) * 19349663u ^ g_salt;
    h ^= h >> 13;
    h *= 0x5bd1e995u;
    return h ^ (h >> 15);
}

extern "C" Bool real_valid_movement_terrain(Pathfinder*, PathfindLayerEnum, const Locomotor*, const Coord3D*)
    __asm__("_oracle_real__ZN10Pathfinder20validMovementTerrainE17PathfindLayerEnumPK9LocomotorPK7Coord3D");
extern "C" Real real_ground_or_structure_height(PartitionManager*, Real, Real)
    __asm__("_oracle_real__ZN16PartitionManager26getGroundOrStructureHeightEff");

Bool Pathfinder::validMovementTerrain(PathfindLayerEnum layer, const Locomotor* locomotor, const Coord3D* pos) {
    if (!g_recording) return real_valid_movement_terrain(this, layer, locomotor, pos);
    const Int cx = REAL_TO_INT_FLOOR(pos->x / 10.0f), cy = REAL_TO_INT_FLOOR(pos->y / 10.0f);
    const Bool valid = cell_hash(cx, cy) % 100u >= g_rate;
    std::ostringstream o;
    o << (g_tape.empty() ? "" : ";") << "V," << static_cast<int>(layer) << ","
      << static_cast<int>(locomotor->getLegalSurfaces()) << "," << hx(pos->x) << "," << hx(pos->y) << ","
      << hx(pos->z) << "=" << (valid ? 1 : 0);
    g_tape += o.str();
    return valid;
}

Real PartitionManager::getGroundOrStructureHeight(Real x, Real y) {
    if (!g_recording) return real_ground_or_structure_height(this, x, y);
    const Real h = TheTerrainLogic->getGroundHeight(x, y, nullptr) +
                   static_cast<Real>(cell_hash(REAL_TO_INT_FLOOR(x), REAL_TO_INT_FLOOR(y)) % 4u) * 5.5f;
    g_tape += std::string(g_tape.empty() ? "" : ";") + "G," + hx(x) + "," + hx(y) + "=" + hx(h);
    return h;
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

static const int kUnits = 5;
static std::string unit_ini() {
    static const char* const kind[kUnits] = {"VEHICLE", "VEHICLE DOZER", "PROJECTILE", "INFANTRY", "VEHICLE"};
    static const char* const geom[kUnits] = {
        "  Geometry = BOX\n  GeometryMajorRadius = 10\n  GeometryMinorRadius = 5\n  GeometryHeight = 8\n",
        "  Geometry = BOX\n  GeometryMajorRadius = 12\n  GeometryMinorRadius = 8\n  GeometryHeight = 8\n",
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 2\n  GeometryHeight = 2\n",
        "  Geometry = CYLINDER\n  GeometryMajorRadius = 4\n  GeometryHeight = 10\n",
        "  Geometry = BOX\n  GeometryMajorRadius = 3\n  GeometryMinorRadius = 2\n  GeometryHeight = 6\n",
    };
    static const char* const phys[kUnits] = {
        "    Mass = 5\n", "    Mass = 20\n", "    Mass = 0.5\n", "    Mass = 1\n",
        "    Mass = 50\n    ForwardFriction = 3\n    LateralFriction = 12\n    ZFriction = 20\n    AerodynamicFriction = 1.5\n",
    };
    std::string s;
    for (int i = 0; i < kUnits; ++i) {
        s += "Object S7_U" + std::to_string(i) + "\n  KindOf = PRELOAD " + kind[i] + "\n" + geom[i] +
             "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100000000\n    InitialHealth = 100000000\n  End\n"
             "  Behavior = PhysicsBehavior ModuleTag_Phys\n" + phys[i] + "  End\nEnd\n";
    }
    return s;
}

static const char* const kAppearance[] = {"TWO_LEGS", "FOUR_WHEELS", "TREADS", "HOVER", "THRUST", "WINGS",
                                          "CLIMBER",  "OTHER",       "MOTORCYCLE"};
static const char* const kZ[] = {"NO_Z_MOTIVE_FORCE",
                                 "SEA_LEVEL",
                                 "SURFACE_RELATIVE_HEIGHT",
                                 "ABSOLUTE_HEIGHT",
                                 "FIXED_SURFACE_RELATIVE_HEIGHT",
                                 "FIXED_ABSOLUTE_HEIGHT",
                                 "FIXED_RELATIVE_TO_GROUND_AND_BUILDINGS",
                                 "RELATIVE_TO_HIGHEST_LAYER"};
static const int kAppearances = 9;
static const int kPerAppearance = 40;

static std::string field(const char* name, const char* const* values, int n) {
    const char* v = values[pick(n)];
    return v ? std::string("  ") + name + " = " + v + "\n" : std::string();
}
#define FIELD(name, ...)                                                  \
    do {                                                                  \
        static const char* const v_[] = {__VA_ARGS__};                    \
        s += field(name, v_, static_cast<int>(sizeof(v_) / sizeof(*v_))); \
    } while (0)

static std::string template_ini(int index) {
    const int appearance = index % kAppearances;
    const bool thrust = appearance == 4;
    const bool air = appearance == 3 || appearance == 4 || appearance == 5;
    std::string s = "Locomotor S7_L" + std::to_string(index) + "\n";
    s += std::string("  Appearance = ") + kAppearance[appearance] + "\n";
    if (air && pick(4) != 0) {
        FIELD("Surfaces", "AIR", "AIR GROUND", "AIR WATER");
    } else {
        FIELD("Surfaces", "GROUND", "GROUND", "GROUND WATER", "CLIFF", "AIR", "GROUND CLIFF RUBBLE");
    }
    FIELD("Speed", "30", "60", "90", "200", "600", "0", "17");
    FIELD("SpeedDamaged", nullptr, nullptr, "20", "45", "0");
    FIELD("TurnRate", "180", "90", "30", "500", "2000", "0", "75");
    FIELD("TurnRateDamaged", nullptr, nullptr, "45", "10");
    FIELD("Acceleration", "100", "30", "1000", "20000", "7", "0", "350");
    FIELD("AccelerationDamaged", nullptr, nullptr, "50", "11");
    FIELD("Braking", nullptr, "100", "40", "1000", "20000", "13", "300");
    if (thrust && pick(6) == 0) s += "  Braking = 0\n";
    // Walkers and climbers recurse without end when MinSpeed is positive (see the top).
    if (appearance != 0 && appearance != 6) FIELD("MinSpeed", nullptr, nullptr, "5", "30", "0", "13");
    FIELD("MinTurnSpeed", nullptr, "0", "10", "60", "25");
    if (!thrust) {
        FIELD("ZAxisBehavior", nullptr, kZ[1], kZ[2], kZ[3], kZ[4], kZ[5], kZ[6], kZ[7], kZ[2], kZ[7], kZ[3]);
        FIELD("Lift", nullptr, "60", "1200", "3000", "300", "900", "9000");
        FIELD("LiftDamaged", nullptr, nullptr, "200", "1500");
    }
    FIELD("PreferredHeight", nullptr, "0", "20", "100", "-5", "10");
    FIELD("PreferredHeightDamping", nullptr, "1.0", "0.5", "0.1", "0");
    FIELD("CirclingRadius", nullptr, "0", "40", "-40", "12");
    FIELD("SpeedLimitZ", nullptr, "1", "30", "300");
    FIELD("MaxThrustAngle", nullptr, "0", "5", "45", "180", "12");
    FIELD("TurnPivotOffset", nullptr, nullptr, "-1.0", "-0.33", "0.5", "1.0", "0");
    FIELD("Extra2DFriction", nullptr, "0.1", "3");
    FIELD("Apply2DFrictionWhenAirborne", nullptr, "Yes", "No");
    FIELD("StickToGround", nullptr, nullptr, "Yes");
    FIELD("AllowAirborneMotiveForce", nullptr, "Yes", air ? "Yes" : "No");
    FIELD("DownhillOnly", nullptr, nullptr, nullptr, nullptr, "Yes");
    FIELD("CanMoveBackwards", nullptr, "Yes");
    FIELD("CloseEnoughDist3D", nullptr, "Yes");
    FIELD("SlideIntoPlaceTime", nullptr, "100", "500", "1700", "33", "7", "1234.5");
    FIELD("AccelerationPitchLimit", nullptr, "10");
    FIELD("DecelerationPitchLimit", nullptr, nullptr, "5");
    FIELD("WanderWidthFactor", nullptr, nullptr, "0", "0.5", "2");
    FIELD("WanderLengthFactor", nullptr, "1", "0.5", "2", "3");
    s += "End\n";
    return s;
}

// Locomotors that can finish a trip, three of each appearance, for the drives.
static const int kDriveTemplates = kAppearances * 3;
static std::string drive_ini(int index) {
    const int appearance = index % kAppearances, kind = index / kAppearances;
    const bool air = appearance == 4 || appearance == 5;
    std::string s = "Locomotor S7_D" + std::to_string(index) + "\n";
    s += std::string("  Appearance = ") + kAppearance[appearance] + "\n";
    s += air ? "  Surfaces = AIR\n" : "  Surfaces = GROUND\n";
    s += kind == 0 ? "  Speed = 90\n  TurnRate = 180\n  Acceleration = 300\n  Braking = 300\n"
         : kind == 1 ? "  Speed = 150\n  TurnRate = 500\n  Acceleration = 900\n  Braking = 150\n"
                     : "  Speed = 60\n  TurnRate = 90\n  Acceleration = 100\n  Braking = 900\n";
    if (appearance != 0 && appearance != 6) {
        s += "  MinTurnSpeed = 10\n";
        if (appearance == 5 || appearance == 4) s += "  MinSpeed = 15\n";
    }
    if (appearance == 3 || appearance == 5)
        s += "  ZAxisBehavior = SURFACE_RELATIVE_HEIGHT\n  Lift = 2000\n  PreferredHeight = 20\n  AllowAirborneMotiveForce = Yes\n";
    if (appearance == 4) s += "  MaxThrustAngle = 45\n  AllowAirborneMotiveForce = Yes\n";
    if (appearance == 2 && kind == 1) s += "  TurnPivotOffset = -0.33\n";
    if (appearance == 1 && kind != 0) s += "  CanMoveBackwards = Yes\n";
    if (appearance == 7 && kind == 2) s += "  SlideIntoPlaceTime = 500\n";
    s += "End\n";
    return s;
}

// ---------------------------------------------------------------------------
// Rows
// ---------------------------------------------------------------------------

// The save bytes, with NaN members made canonical: x86 gives generated NaNs another sign.
static std::string phys_save(PhysicsBehavior* p) {
    std::string hex = oracle_wp51::World::snapshot_save_hex(p);
    const size_t start = hex.size() - 88 * 2;
    static const int floats[] = {0, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 44, 60, 76, 80, 84};
    for (int off : floats) {
        const size_t at = start + off * 2;
        uint32_t u = 0;
        for (int b = 3; b >= 0; --b)
            u = (u << 8) | static_cast<uint32_t>(std::stoul(hex.substr(at + b * 2, 2), nullptr, 16));
        float f;
        std::memcpy(&f, &u, 4);
        if (std::isnan(f)) hex.replace(at, 8, "0000c07f");
    }
    return hex;
}

// "<ret>|<matrix>,<angle>|<loco save>|<loco crc>|<physics save>|<physics crc>|<bits>|<rng crc>|<tape>"
static std::string row(long long ret, Object* o, Locomotor* loco) {
    PhysicsBehavior* p = o->getPhysics();
    std::ostringstream out;
    out << ret << '|';
    const Matrix3D* m = o->getTransformMatrix();
    for (int r = 0; r < 3; ++r)
        for (int c = 0; c < 4; ++c) out << hx((*m)[r][c]) << ',';
    out << hx(o->getOrientation()) << '|';
    out << oracle_wp51::World::snapshot_save_hex(loco) << '|' << oracle_wp51::World::snapshot_crc(loco) << '|';
    out << phys_save(p) << '|' << oracle_wp51::World::snapshot_crc(p) << '|';
    const Drawable* d = o->getDrawable();
    if (!d) {
        std::fprintf(stderr, "object without a drawable\n");
        std::exit(1);
    }
    out << (o->getStatusBits().test(OBJECT_STATUS_BRAKING) ? 1 : 0)
        << (d->getModelConditionFlags().test(MODELCONDITION_OVER_WATER) ? 1 : 0) << '|';
    out << GetGameLogicRandomSeedCRC() << '|' << g_tape;
    return out.str();
}

// What parsing and validate() leave in the template.
static std::string parsed_fields(const LocomotorTemplate* t) {
    std::ostringstream o;
    const float f[] = {t->m_maxSpeed, t->m_maxSpeedDamaged, t->m_minSpeed, t->m_maxTurnRate, t->m_maxTurnRateDamaged,
                       t->m_acceleration, t->m_accelerationDamaged, t->m_lift, t->m_liftDamaged, t->m_braking,
                       t->m_minTurnSpeed, t->m_preferredHeight, t->m_preferredHeightDamping, t->m_circlingRadius,
                       t->m_speedLimitZ, t->m_extra2DFriction, t->m_maxThrustAngle, t->m_accelPitchLimit,
                       t->m_decelPitchLimit, t->m_turnPivotOffset, t->m_closeEnoughDist,
                       t->m_ultraAccurateSlideIntoPlaceFactor, t->m_wanderWidthFactor, t->m_wanderLengthFactor};
    for (float v : f) o << hx(v) << ' ';
    o << static_cast<int>(t->m_surfaces) << ' ' << static_cast<int>(t->m_behaviorZ) << ' '
      << static_cast<int>(t->m_appearance) << ' ' << (t->m_apply2DFrictionWhenAirborne ? 1 : 0)
      << (t->m_stickToGround ? 1 : 0) << (t->m_allowMotiveForceWhileAirborne ? 1 : 0) << (t->m_downhillOnly ? 1 : 0)
      << (t->m_canMoveBackward ? 1 : 0) << (t->m_isCloseEnoughDist3D ? 1 : 0);
    return o.str();
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_s7_locomotor_movement <out.json>\n");
        return 2;
    }
    oracle_wp51::World w;
    TheTerrainLogic = NEW S7Terrain;
    TheTerrainLogic->init();
    TheWritableGlobalData->m_gravity = -1.0f;
    if (!TheLocomotorStore) TheLocomotorStore = NEW LocomotorStore;
    w.load_ini(unit_ini());
    w.init_with_neutral_player();

    const std::string tmp = std::string(argv[1]) + ".tmp";
    FILE* out = std::fopen(tmp.c_str(), "w");
    if (!out) return 1;

    const int kRandomTemplates = kAppearances * kPerAppearance;
    const int kTemplates = kRandomTemplates + kDriveTemplates;
    std::vector<const LocomotorTemplate*> templates;
    auto escape = [](const std::string& text) {
        std::string escaped;
        for (char c : text) {
            if (c == '\n') escaped += "\\n";
            else escaped += c;
        }
        return escaped;
    };
    std::fprintf(out, "{\"units\":\"%s\",\n\"templates\":[", escape(unit_ini()).c_str());
    for (int i = 0; i < kTemplates; ++i) {
        const std::string text = i < kRandomTemplates ? template_ini(i) : drive_ini(i - kRandomTemplates);
        w.mem_ini.parse_blocks(text, INI_LOAD_OVERWRITE, [](INI* ini, const char* tok) {
            if (std::strcmp(tok, "Locomotor") == 0) LocomotorStore::parseLocomotorTemplateDefinition(ini);
        });
        const std::string name = i < kRandomTemplates ? "S7_L" + std::to_string(i)
                                                      : "S7_D" + std::to_string(i - kRandomTemplates);
        const LocomotorTemplate* t = TheLocomotorStore->findLocomotorTemplate(NAMEKEY(name.c_str()));
        if (!t) {
            std::fprintf(stderr, "template %s did not parse\n", name.c_str());
            return 1;
        }
        templates.push_back(t);
        std::fprintf(out, "%s\n\"%s\"", i ? "," : "", escape(text).c_str());
    }

    std::fprintf(out, "],\n\"parsed\":[");
    for (int i = 0; i < kTemplates; ++i)
        std::fprintf(out, "%s\n\"p %s\"", i ? "," : "", parsed_fields(templates[i]).c_str());

    const int kTotal = 1200;
    std::fprintf(out, "],\n\"vector_count\":%d,\"vectors\":[", kTotal);

    static const float gravities[] = {-1.0f, -1.0f, -1.0f, -64.0f / 900.0f, -2.5f, -0.3f, 0.0f};
    static const float bases[] = {0.0f, 0.0f, 5.5f, -3.0f};
    static const float slopes[] = {0.0f, 0.0f, 0.5f, 2.0f, -0.25f};
    static const float steps[] = {0.0f, 0.0f, 20.0f, -15.0f, 0.05f, 3.0f};
    static const float waters[] = {0.0f, 8.0f, -2.0f, 40.0f};
    static const float heights[] = {0.0f, 0.0f, 0.0f, 0.5f, 2.0f, 8.9f, 9.0f, 9.5f, 30.0f, 110.0f, -4.0f};
    static const float speeds[] = {0.0f, 0.1f, 0.5f, 1.0f, 3.0f, 7.0f, 20.0f, 99999.0f, -1.0f};
    static const float dists[] = {0.0f, 0.5f, 5.0f, 10.0f, 10.5f, 25.0f, 39.0f, 40.0f, 41.0f, 100.0f, 99999.0f};
    static const float reach[] = {0.05f, 0.5f, 3.0f, 12.0f, 18.0f, 30.0f, 80.0f, 300.0f};
    static const float rises[] = {0.0f, 0.0f, 0.5f, -0.5f, 1.0f, -1.0f, 2.0f, 9.0f, 10.5f, -12.0f, 60.0f};
    // Bearings off the heading, around the 9, 15, 45 and 90 degree limits of the movers.
    static const float bearings[] = {0.15f, -0.17f, 0.25f, -0.28f, 0.77f, -0.8f, 0.82f, -0.84f, 1.55f, -1.6f, 3.0f};
    static const float veloc[] = {0.0f, 0.0f, 0.2f, 1.0f, 3.0f, 10.0f};
    static const float overrides[] = {0.0f, 0.02f, 0.3f, 2.0f, 12.0f, 99999.0f};
    static const float pref[] = {0.0f, 10.0f, 50.0f, -3.0f};

    long long nan_stops = 0, tape_calls = 0, rows_total = 0, arrivals = 0;
    for (int i = 0; i < kTotal; ++i) {
        // The first vectors are drives: one goal, followed until the unit stops.
        const bool drive = i < 2 * kDriveTemplates;
        const int t = drive ? kRandomTemplates + i % kDriveTemplates : static_cast<int>(pick(kRandomTemplates));
        const LocomotorTemplate* tmpl = templates[t];
        const int unit = drive ? 0 : pick(3) == 0 ? static_cast<int>(pick(kUnits)) : (pick(2) ? 0 : 3);
        const float gravity = drive ? -1.0f : one_of(gravities);
        const int penalty = pick(5) == 0 ? BODY_DAMAGED : BODY_REALLYDAMAGED;
        g_base = one_of(bases);
        g_slope = drive ? 0.0f : one_of(slopes);
        g_step = drive ? 0.0f : one_of(steps);
        g_water = pick(3) == 0;
        g_water_z = one_of(waters);
        g_rate = drive ? (pick(8) == 0 ? 10 : 0) : (pick(3) == 0 ? 0 : (pick(2) ? 15 : 45));
        g_salt = pick(0x7fffffffu);
        const uint32_t seed = pick(0x7fffffffu);
        // Some above 2^24, where `frame + 75.0f` rounds.
        UnsignedInt frame = pick(6) == 0 ? 0x01000000u + pick(0x3e000000u) : 1 + pick(500);
        TheWritableGlobalData->m_gravity = -1.0f;
        TheWritableGlobalData->m_movementPenaltyDamageState = static_cast<BodyDamageType>(penalty);
        w.set_frame(frame);

        Coord3D pos{500.0f + spread(8.0f), 500.0f + spread(8.0f), 0.0f};
        pos.z = TheTerrainLogic->getGroundHeight(pos.x, pos.y, nullptr) + one_of(heights);
        const float angle = spread(3.14159265f);
        Object* o = w.spawn_object(("S7_U" + std::to_string(unit)).c_str(), w.team_a, pos, angle);
        PhysicsBehavior* phys = o->getPhysics();
        if (!phys) {
            std::fprintf(stderr, "object without physics\n");
            return 1;
        }
        TheWritableGlobalData->m_gravity = gravity;
        InitRandom(seed);
        g_tape.clear();
        Locomotor* loco = newInstance(Locomotor)(tmpl);

        std::ostringstream setup;
        setup << t << ' ' << unit << ' ' << hx(gravity) << ' ' << penalty << ' ' << hx(g_base) << ' ' << hx(g_slope)
              << ' ' << hx(g_step) << ' ' << g_water << ' ' << hx(g_water_z) << ' ' << g_rate << ' ' << g_salt << ' '
              << seed << ' ' << frame << ' ' << hx3(pos) << ' ' << hx(angle);

        std::vector<std::string> ops, rows;
        bool stopped = false;
        auto run = [&](const std::string& op) {
            if (stopped) return;
            std::istringstream in(op);
            std::string code;
            in >> code;
            auto f = [&]() {
                std::string tok;
                in >> tok;
                const uint32_t u = static_cast<uint32_t>(std::stoul(tok, nullptr, 16));
                float v;
                std::memcpy(&v, &u, 4);
                return v;
            };
            auto n = [&]() {
                long long v;
                in >> v;
                return v;
            };
            auto c = [&]() {
                Coord3D r;
                r.x = f();
                r.y = f();
                r.z = f();
                return r;
            };
            long long ret = 0;
            g_tape.clear();
            g_recording = true;
            if (code == "T") {
                const Coord3D goal = c();
                const float dist = f(), speed = f();
                Bool blocked = static_cast<Bool>(n());
                loco->locoUpdate_moveTowardsPosition(o, goal, dist, speed, &blocked);
                ret = blocked;
            } else if (code == "A") {
                loco->locoUpdate_moveTowardsAngle(o, f());
            } else if (code == "M") {
                ret = loco->locoUpdate_maintainCurrentPosition(o);
            } else if (code == "R") {
                const Coord3D goal = c();
                Real rel = 0.0f;
                const PhysicsTurningType turn = loco->rotateTowardsPosition(o, goal, &rel);
                ret = static_cast<long long>(turn) * 10000000000LL + bits(rel);
            } else if (code == "Z") {
                ret = loco->handleBehaviorZ(o, phys, c());
            } else if (code == "P") {
                g_recording = false;
                ret = static_cast<long long>(static_cast<UpdateModule*>(phys)->update());
            } else if (code == "O") {
                loco->setPhysicsOptions(o);
            } else if (code == "S") {
                loco->startMove();
            } else if (code == "F") {
                const long long which = n();
                const Bool on = static_cast<Bool>(n());
                if (which == 0) loco->setUltraAccurate(on);
                else if (which == 1) loco->setUsePreciseZPos(on);
                else if (which == 2) loco->setNoSlowDownAsApproachingDest(on);
                else if (which == 3) loco->setAllowInvalidPosition(on);
                else loco->setCloseEnoughDist3D(on);
            } else if (code == "V") {
                const long long which = n();
                const float v = f();
                if (which == 0) loco->setMaxSpeed(v);
                else if (which == 1) loco->setMaxAcceleration(v);
                else if (which == 2) loco->setMaxBraking(v);
                else if (which == 3) loco->setMaxTurnRate(v);
                else if (which == 4) loco->setMaxLift(v);
                else if (which == 5) loco->setPreferredHeight(v);
                else loco->setCloseEnoughDist(v);
            } else if (code == "W") {
                phys->m_vel = c();
                phys->m_velMag = -1.0f;
            } else if (code == "Q") {
                const Coord3D p = c();
                const float a = f();
                o->setPosition(&p);
                o->setOrientation(a);
            } else if (code == "D") {
                o->getBodyModule()->setDamageState(static_cast<BodyDamageType>(n()));
            } else if (code == "K") {
                const long long which = n();
                const Bool on = static_cast<Bool>(n());
                if (which == 0) phys->setStunned(on);
                else if (which == 1) {
                    if (on) o->setDisabled(DISABLED_HELD);
                    else o->clearDisabled(DISABLED_HELD);
                } else if (which == 2) o->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_BRAKING), on);
                else if (which == 3) o->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_DECK_HEIGHT_OFFSET), on);
                else o->setLayer(on ? static_cast<PathfindLayerEnum>(LAYER_GROUND + 1) : LAYER_GROUND);
            } else if (code == "N") {
                frame += static_cast<UnsignedInt>(n());
                w.set_frame(frame);
            } else {
                std::fprintf(stderr, "unknown op %s\n", code.c_str());
                std::exit(1);
            }
            g_recording = false;
            const Coord3D at = *o->getPosition();
            if (std::isnan(at.x) || std::isnan(at.y) || std::isnan(at.z) || o->isDestroyed()) {
                stopped = true;
                ++nan_stops;
                return;
            }
            for (char ch : g_tape) tape_calls += ch == '=';
            ops.push_back(op);
            rows.push_back(row(ret, o, loco));
        };

        const std::string spawn_row = row(0, o, loco);

        // A goal that a tracked mover cannot read uninitialized memory from.
        auto safe_goal = [&](Coord3D goal) {
            if (tmpl->m_appearance == LOCO_TREADS && tmpl->m_turnPivotOffset != 0.0f) {
                const Real off = tmpl->m_turnPivotOffset * o->getGeometryInfo().getBoundingCircleRadius();
                const Coord3D* at = o->getPosition();
                const Coord3D* dir = o->getUnitDirectionVector2D();
                const Real px = at->x + dir->x * off, py = at->y + dir->y * off;
                if (std::fabs(goal.x - px) < 0.2f && std::fabs(goal.y - py) < 0.2f) goal.x += 1.0f;
            }
            return goal;
        };
        auto random_goal = [&]() {
            const Coord3D at = *o->getPosition();
            Coord3D g{at.x + spread(one_of(reach)), at.y + spread(one_of(reach)), at.z + one_of(rises)};
            const uint32_t kind = pick(16);
            if (kind == 0) g = {at.x, at.y, at.z};
            else if (kind <= 2) {
                // Dead ahead or dead astern.
                const Coord3D* dir = o->getUnitDirectionVector2D();
                const Real len = (kind == 1 ? 1.0f : -1.0f) * one_of(reach);
                g.x = at.x + dir->x * len;
                g.y = at.y + dir->y * len;
            } else if (kind == 3 && tmpl->m_turnPivotOffset != 0.0f) {
                const Real off = tmpl->m_turnPivotOffset * o->getGeometryInfo().getBoundingCircleRadius();
                const Coord3D* dir = o->getUnitDirectionVector2D();
                g.x = at.x + dir->x * off + spread(0.12f);
                g.y = at.y + dir->y * off + spread(0.12f);
            }
            else if (kind <= 5) {
                const Real a = o->getOrientation() + one_of(bearings);
                const Real len = one_of(reach);
                g.x = at.x + Cos(a) * len;
                g.y = at.y + Sin(a) * len;
            }
            return safe_goal(g);
        };
        auto path_dist = [&](const Coord3D& g) {
            const Coord3D at = *o->getPosition();
            const Real dx = g.x - at.x, dy = g.y - at.y;
            const Real flat = std::sqrt(dx * dx + dy * dy);
            const uint32_t kind = pick(8);
            if (kind >= 6) {
                // Around the distances at which the movers start and stop braking.
                static const float shares[] = {0.8f, 1.0f, 1.5f, 1.9f, 2.1f, 2.5f, 3.5f};
                const Real v = o->getPhysics()->getForwardSpeed2D();
                const Real braking = loco->getBraking();
                return one_of(shares) * (v / 1.5f) * (v / braking);
            }
            if (kind == 0) return one_of(dists);
            if (kind == 1) return flat * 0.4f;
            if (kind == 2) return flat * 3.0f;
            return flat;
        };
        auto move_op = [&](const Coord3D& g, float dist, float speed, int blocked) {
            run("T " + hx3(g) + " " + hx(dist) + " " + hx(speed) + " " + std::to_string(blocked));
        };

        if (drive) {
            const Coord3D goal{pos.x + spread(60.0f), pos.y + spread(60.0f), pos.z};
            run("O");
            run("S");
            bool arrived = false;
            for (int k = 0; k < 40 && !stopped; ++k) {
                const Coord3D at = *o->getPosition();
                const Real dx = goal.x - at.x, dy = goal.y - at.y;
                const Real flat = std::sqrt(dx * dx + dy * dy);
                arrived = arrived || flat < 6.0f;
                if (arrived) run("M");
                else move_op(safe_goal(goal), flat, 99999.0f, 0);
                run("P");
                run("N 1");
                // Two frames of holding after arrival are enough.
                if (arrived && k > 2 && ops.size() >= 6 && ops[ops.size() - 6] == "M") break;
            }
            arrivals += arrived;
        } else {
            if (pick(3) == 0) run("O");
            if (pick(3) == 0) {
                Coord3D v{spread(one_of(veloc)), spread(one_of(veloc)), pick(3) ? 0.0f : spread(one_of(veloc))};
                if (pick(2)) {
                    const Coord3D* dir = o->getUnitDirectionVector2D();
                    const Real s = spread(one_of(veloc));
                    v = {dir->x * s, dir->y * s, v.z};
                }
                run("W " + hx3(v));
            }
            if (pick(6) == 0) run("D " + std::to_string(1 + pick(3)));
            if (pick(4) == 0) run("F " + std::to_string(pick(5)) + " 1");
            if (pick(4) == 0) run("V " + std::to_string(pick(7)) + " " + hx(pick(4) == 0 ? one_of(pref) : one_of(overrides)));
            if (pick(8) == 0) run("K " + std::to_string(pick(5)) + " 1");
            const int app = tmpl->m_appearance;
            if (app == LOCO_TREADS || app == LOCO_WHEELS_FOUR || app == LOCO_MOTORCYCLE) {
                // Start braking, then ask again around twice the braking distance, with the
                // goal too close to raise the path distance.
                static const float vs[] = {3.0f, 7.0f, 10.0f, 20.0f};
                static const float shares[] = {0.6f, 0.9f, 1.2f, 1.6f, 1.9f, 2.1f, 2.6f};
                const Real v = one_of(vs);
                const Coord3D at = *o->getPosition();
                const Coord3D dir = *o->getUnitDirectionVector2D();
                const Coord3D near_goal = safe_goal({at.x + dir.x * 2.0f, at.y + dir.y * 2.0f, at.z});
                run("W " + hx3({dir.x * v, dir.y * v, 0.0f}));
                move_op(near_goal, 1.0f, 99999.0f, 0);
                const Real speed = o->getPhysics()->getForwardSpeed2D(), braking = loco->getBraking();
                const Real slow = app == LOCO_TREADS ? (speed / 1.5f) * (speed / braking)
                                                     : (speed / 1.5f) * (speed / braking + 1.0f) + speed;
                move_op(near_goal, one_of(shares) * slow, 99999.0f, 0);
            }
            const int count = 3 + static_cast<int>(pick(8));
            Coord3D held = random_goal();
            for (int k = 0; k < count && !stopped; ++k) {
                if (pick(3) == 0) held = random_goal();
                held = safe_goal(held);
                const uint32_t code = pick(100);
                if (code < 38) {
                    const float speed = pick(2) ? 99999.0f : one_of(speeds);
                    move_op(held, path_dist(held), speed, pick(7) == 0);
                    if (pick(2)) run("P");
                    if (pick(2)) run("N " + std::to_string(1 + pick(pick(3) ? 3 : 90)));
                } else if (code < 46) {
                    run("A " + hx(spread(3.14159265f)));
                } else if (code < 58) {
                    run("M");
                    if (pick(2)) run("P");
                } else if (code < 62) {
                    run("R " + hx3(held));
                } else if (code < 66) {
                    run("Z " + hx3(held));
                } else if (code < 70) {
                    run("P");
                } else if (code < 74) {
                    run("F " + std::to_string(pick(5)) + " " + std::to_string(pick(3) != 0));
                } else if (code < 78) {
                    run("V " + std::to_string(pick(7)) + " " + hx(pick(4) == 0 ? one_of(pref) : one_of(overrides)));
                } else if (code < 84) {
                    const Coord3D* dir = o->getUnitDirectionVector2D();
                    const Real s = spread(one_of(veloc));
                    Coord3D v{dir->x * s, dir->y * s, pick(3) ? 0.0f : spread(one_of(veloc))};
                    if (pick(3) == 0) v = {spread(one_of(veloc)), spread(one_of(veloc)), v.z};
                    run("W " + hx3(v));
                } else if (code < 90) {
                    Coord3D p{500.0f + spread(20.0f), 500.0f + spread(20.0f), 0.0f};
                    p.z = TheTerrainLogic->getGroundHeight(p.x, p.y, nullptr) + one_of(heights);
                    run("Q " + hx3(p) + " " + hx(spread(3.14159265f)));
                } else if (code < 93) {
                    run("K " + std::to_string(pick(5)) + " " + std::to_string(pick(2)));
                } else if (code < 95) {
                    run("D " + std::to_string(pick(4)));
                } else if (code < 97) {
                    run("S");
                } else {
                    run("N " + std::to_string(1 + pick(100)));
                }
            }
        }

        std::fprintf(out, "%s\n{\"v\":%d,\"setup\":\"%s\",\"spawn\":\"%s\",\"ops\":[", i ? "," : "", i,
                     setup.str().c_str(), spawn_row.c_str());
        for (size_t k = 0; k < ops.size(); ++k) std::fprintf(out, "%s\"%s\"", k ? "," : "", ops[k].c_str());
        std::fprintf(out, "],\"rows\":[");
        // A row equal to the one before it is written as "=".
        for (size_t k = 0; k < rows.size(); ++k)
            std::fprintf(out, "%s\"%s\"", k ? "," : "",
                         rows[k] == (k ? rows[k - 1] : spawn_row) ? "=" : rows[k].c_str());
        std::fprintf(out, "]}");
        rows_total += static_cast<long long>(rows.size());

        deleteInstance(loco);
        TheGameLogic->destroyObject(o);
        w.flush_destroy_list();
    }
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    if (std::rename(tmp.c_str(), argv[1]) != 0) return 1;
    std::printf("rows %lld, tape calls %lld, vectors cut at a NaN position %lld, drives arrived %lld of %d\n",
                rows_total, tape_calls, nan_stops, arrivals, 2 * kDriveTemplates);
    return 0;
}
