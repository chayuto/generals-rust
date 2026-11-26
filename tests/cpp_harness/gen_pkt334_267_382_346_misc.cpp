// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for HelicopterSlowDeathBehavior (PKT-334), DumbProjectileBehavior
// (PKT-267), TensileFormationUpdate (PKT-382) and OCLUpdate (PKT-346), run on the
// unmodified reference modules in a small real world (neutral player, players A and
// B, stepped terrain with arbitrary normals).
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt334_267_382_346_misc.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/PKT334_267_382_346_Misc.json
//
// Observed through: module save bytes and CRC, object matrix, status, body and model
// condition bits, FX and OCL nuggets, the detonation weapon's historic damage list,
// physics and locomotor members and the logic seed CRC. Bones come from a stand-in
// draw module.
//
// The reference library saves DumbProjectileBehavior version 1
// (RETAIL_COMPATIBLE_XFER_SAVE=1), so the flight path step is recorded beside the save,
// and a reload starts the copy at step 0.
//
// Not covered: bridges (so getHighestLayerForDestination always answers the ground and
// there is no bridge-passing detonation), projectile launch bones and turrets, a sneaky-target
// offset (JetAIUpdate), a projectile with no detonation weapon colliding with anything
// but its victim (the reference dereferences null), an empty tree search in
// TensileFormationUpdate (null dereference), objects without a drawable, the pathfind
// footprint walls, audio, particle systems, drawable hiding, an unknown rubble
// template (the reference throws), loading a save of anything but the projectile, and
// the effects of the OCLs and weapons themselves.

#include "oracle_wp51_world.h"

#include "Common/AcademyStats.h"
#include "Common/DrawModule.h"
#include "Common/Player.h"
#include "Common/RandomValue.h"
#include "Common/Team.h"
#include "GameClient/Drawable.h"
#include "GameClient/FXList.h"
#include "GameLogic/ExperienceTracker.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Locomotor.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/DumbProjectileBehavior.h"
#include "GameLogic/Module/HelicopterSlowDeathUpdate.h"
#include "GameLogic/Module/OCLUpdate.h"
#include "GameLogic/Module/ParkingPlaceBehavior.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Module/TensileFormationUpdate.h"
#include "GameLogic/Module/ToppleUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectCreationList.h"
#include "GameLogic/Weapon.h"

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <set>
#include <sstream>
#include <string>
#include <vector>

#define PEEK(tag, cls, mtype, member)                                  \
    struct tag { using type = mtype cls::*; };                         \
    template struct MemberAccess<tag, &cls::member>;                   \
    inline mtype cls::* access_member(tag);

PEEK(MPhysVel, PhysicsBehavior, Coord3D, m_vel)
PEEK(MPhysAccel, PhysicsBehavior, Coord3D, m_accel)
PEEK(MPhysExpires, PhysicsBehavior, UnsignedInt, m_motiveForceExpires)
PEEK(MPhysCollidee, PhysicsBehavior, ObjectID, m_lastCollidee)
PEEK(MPhysIgnore, PhysicsBehavior, ObjectID, m_ignoreCollisionsWith)
PEEK(MPhysPitch, PhysicsBehavior, Real, m_pitchRate)
PEEK(MPhysRoll, PhysicsBehavior, Real, m_rollRate)
PEEK(MPhysYaw, PhysicsBehavior, Real, m_yawRate)
PEEK(MLocoLift, Locomotor, Real, m_maxLift)
PEEK(MLocoBraking, Locomotor, Real, m_maxBraking)
PEEK(MSink, ExperienceTracker, ObjectID, m_experienceSink)
PEEK(MHistoric, WeaponTemplate, HistoricWeaponDamageList, m_historicDamage)
PEEK(MCleared, AcademyStats, UnsignedInt, m_clearedGarrisonedBuildings)
PEEK(MColor, Player, Color, m_color)

struct MPathTag { using type = VecCoord3D DumbProjectileBehavior::*; };
template struct MemberAccess<MPathTag, &DumbProjectileBehavior::m_flightPath>;
inline VecCoord3D DumbProjectileBehavior::* access_member(MPathTag);
PEEK(MDumbFlags, DumbProjectileBehavior, WeaponBonusConditionFlags, m_extraBonusFlags)
PEEK(MDumbDetonated, DumbProjectileBehavior, Bool, m_hasDetonated)
PEEK(MDumbStep, DumbProjectileBehavior, Int, m_currentFlightPathStep)

struct DrawArrayTag { using type = Module** (Drawable::*)[NUM_DRAWABLE_MODULE_TYPES]; };
template struct MemberAccess<DrawArrayTag, &Drawable::m_modules>;
DrawArrayTag::type access_member(DrawArrayTag);

static uint32_t g_state = 0x33426738u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}
static bool chance(uint32_t percent) { return pick(100) < percent; }
static uint32_t bits(Real v) {
    uint32_t u;
    std::memcpy(&u, &v, 4);
    return u;
}
// A value in [lo, hi] on a 1/64 grid.
static Real grid(int lo, int hi) {
    return static_cast<Real>(lo * 64 + static_cast<int>(pick((hi - lo) * 64 + 1))) / 64.0f;
}
static void die(const char* why) {
    std::fprintf(stderr, "%s\n", why);
    std::exit(1);
}
static std::string num(uint32_t v) { return std::to_string(v); }
static std::string rb(Real v) { return std::to_string(bits(v)); }
static std::string xyz(const Coord3D& p) { return rb(p.x) + "," + rb(p.y) + "," + rb(p.z); }
static std::string dash(const std::string& s) { return s.empty() ? std::string("-") : s; }

// ---------------------------------------------------------------------------------------------
// Recorders
// ---------------------------------------------------------------------------------------------
static std::ostringstream g_fx, g_ocl;
static const int kFx = 6;
static const char* const kFxNames[kFx] = {"M_F0", "M_F1", "M_F2", "M_F3", "M_F4", "M_F5"};
static const int kOcl = 6;
static const char* const kOclNames[kOcl] = {"M_O0", "M_O1", "M_O2", "M_O3", "M_O4", "M_O5"};

class RecordingFx : public FXNugget {
public:
    int list = 0;
    void doFXPos(const Coord3D* p, const Matrix3D*, const Real, const Coord3D*, const Real) const override {
        g_fx << list << ',' << xyz(*p) << ';';
    }
protected:
    MemoryPool* getObjectMemoryPool() override { return nullptr; }
};
static RecordingFx g_fx_nugget[kFx];

// The angle an OCL got, checked by the OCLUpdate vectors.
class RecordingOcl : public ObjectCreationNugget {
public:
    int list = 0;
    Object* create(const Object* primaryObj, const Coord3D* primary, const Coord3D* secondary, Real angle,
                   UnsignedInt) const override {
        if (angle != INVALID_ANGLE && angle != primaryObj->getOrientation()) die("unexpected OCL angle");
        g_ocl << list << ',' << primaryObj->getID() << ',' << xyz(*primary) << ','
              << (secondary ? xyz(*secondary) : std::string("-")) << ';';
        return nullptr;
    }
    Object* create(const Object* primary, const Object* secondary, UnsignedInt) const override {
        if (secondary) die("unexpected secondary object");
        g_ocl << list << ',' << primary->getID() << ',' << xyz(*primary->getPosition()) << ",-;";
        return nullptr;
    }
protected:
    MemoryPool* getObjectMemoryPool() override { return nullptr; }
};
static RecordingOcl g_ocl_nugget[kOcl];

static std::string take(std::ostringstream& s) {
    const std::string v = dash(s.str());
    s.str("");
    return v;
}

// ---------------------------------------------------------------------------------------------
// Terrain
// ---------------------------------------------------------------------------------------------
class StepTerrain : public oracle_wp51::OracleTerrainLogic {
public:
    // The base class answers 0; the game's terrain answers the ground without bridges.
    Real getLayerHeight(Real x, Real y, PathfindLayerEnum, Coord3D* normal = nullptr, Bool = true) const override {
        return getGroundHeight(x, y, normal);
    }
    Real getGroundHeight(Real x, Real y, Coord3D* normal = nullptr) const override {
        if (normal) {
            normal->x = static_cast<Real>(static_cast<int>(x / 25.0f) % 5) * 0.1f;
            normal->y = static_cast<Real>(static_cast<int>(y / 35.0f) % 5) * -0.1f;
            normal->z = 1.0f - static_cast<Real>(static_cast<int>((x + y) / 45.0f) % 4) * 0.05f;
        }
        return x * 0.01f + y * 0.02f + static_cast<Real>(static_cast<int>(x / 40.0f) % 3) * 4.0f +
               static_cast<Real>(static_cast<int>(y / 60.0f) % 4) * 3.0f;
    }
};

// ---------------------------------------------------------------------------------------------
// Bones
// ---------------------------------------------------------------------------------------------
static bool g_have_bone = false;
static Coord3D g_bone;
static const char* const kBoneName = "Blade";

class OracleDraw : public DrawModule, public ObjectDrawInterface {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(OracleDraw, "OracleMiscDraw", 64, 64)
public:
    OracleDraw(Thing* thing, const ModuleData* data) : DrawModule(thing, data) {}

    NameKeyType getModuleNameKey() const override { return NAMEKEY("OracleMiscDraw"); }
    void doDrawModule(const Matrix3D*) override {}
    void setShadowsEnabled(Bool) override {}
    void releaseShadows() override {}
    void allocateShadows() override {}
    void setFullyObscuredByShroud(Bool) override {}
    void reactToTransformChange(const Matrix3D*, const Coord3D*, Real) override {}
    void reactToGeometryChange() override {}
    ObjectDrawInterface* getObjectDrawInterface() override { return this; }
    const ObjectDrawInterface* getObjectDrawInterface() const override { return this; }

    Int getPristineBonePositionsForConditionState(const ModelConditionFlags&, const char* prefix,
                                                  Int start, Coord3D* positions,
                                                  Matrix3D* transforms, Int max) const override {
        if (std::strcmp(prefix, kBoneName) != 0 || start != 0 || !positions || transforms || max != 1)
            die("unexpected bone query");
        if (!g_have_bone) return 0;
        positions[0] = g_bone;
        return 1;
    }

    Bool clientOnly_getRenderObjInfo(Coord3D*, Real*, Matrix3D*) const override { return FALSE; }
    Bool clientOnly_getRenderObjBoundBox(OBBoxClass*) const override { return FALSE; }
    Bool clientOnly_getRenderObjBoneTransform(const AsciiString&, Matrix3D*) const override { return FALSE; }
    Int getCurrentBonePositions(const char*, Int, Coord3D*, Matrix3D*, Int) const override { return 0; }
    Bool getCurrentWorldspaceClientBonePositions(const char*, Matrix3D&) const override { return FALSE; }
    Bool getProjectileLaunchOffset(const ModelConditionFlags&, WeaponSlotType, Int, Matrix3D*,
                                   WhichTurretType, Coord3D*, Coord3D*) const override { return FALSE; }
    void updateProjectileClipStatus(UnsignedInt, UnsignedInt, WeaponSlotType) override {}
    void updateDrawModuleSupplyStatus(Int, Int) override {}
    void notifyDrawModuleDependencyCleared() override {}
    void setHidden(Bool) override {}
    void replaceModelConditionState(const ModelConditionFlags&) override {}
    void replaceIndicatorColor(Color) override {}
    Bool handleWeaponFireFX(WeaponSlotType, Int, const FXList*, Real, const Coord3D*, Real) override { return FALSE; }
    Int getBarrelCount(WeaponSlotType) const override { return 0; }
    void setSelectable(Bool) override {}
    void setAnimationLoopDuration(UnsignedInt) override {}
    void setAnimationCompletionTime(UnsignedInt) override {}
    Bool updateBonesForClientParticleSystems() override { return FALSE; }
    void setAnimationFrame(int) override {}
    void setPauseAnimation(Bool) override {}
    void updateSubObjects() override {}
    void showSubObject(const AsciiString&, Bool) override {}

protected:
    void crc(Xfer*) override {}
    void xfer(Xfer*) override {}
    void loadPostProcess() override {}
};
OracleDraw::~OracleDraw() {}

// ---------------------------------------------------------------------------------------------
// Fixture
// ---------------------------------------------------------------------------------------------
static const int kHeli = 32;
static const int kProj = 40;
static const int kWeapons = 8;
static const int kTensile = 4;
static const int kOclVariants = 32;

template <size_t N>
static const char* choose(const char* const (&list)[N]) { return list[pick(N)]; }

static std::string opt(const char* field, const char* value) {
    return value ? std::string("    ") + field + " = " + value + "\n" : std::string();
}
static std::string asset(const char* field, const char* prefix, int count) {
    if (chance(20)) return std::string();
    return std::string("    ") + field + " = " +
           (chance(10) ? std::string("None") : std::string(prefix) + std::to_string(pick(count))) + "\n";
}
static std::string active_body(const char* health) {
    return std::string("  Body = ActiveBody ModuleTag_Body\n    MaxHealth = ") + health +
           "\n    InitialHealth = " + health + "\n  End\n";
}

// Targets a projectile can hit.
static const int kTargets = 14;
static const char* const kTargetKinds[kTargets] = {
    "STRUCTURE", "SHRUBBERY", "PROJECTILE", "VEHICLE", "SMALL_MISSILE", "BALLISTIC_MISSILE",
    "VEHICLE" /* fence */, "STRUCTURE FS_AIRFIELD", "STRUCTURE" /* garrison */,
    "STRUCTURE" /* immune garrison */, "VEHICLE" /* transport */, "PROJECTILE SMALL_MISSILE",
    "INFANTRY", "STRUCTURE SHRUBBERY"};

static std::string fixture_ini() {
    std::string s;
    // Kept whole: HistoricBonusTime keeps every shot of a vector in the list.
    static const char* const kDamageType[] = {"EXPLOSION", "FLAME", "PARTICLE_BEAM", "ARMOR_PIERCING"};
    static const char* const kSpeed[] = {"150", "600", "999", "2000", "30", "0"};
    static const char* const kMinSpeed[] = {"30", "90", "300", "0"};
    static const char* const kCollides[] = {
        "STRUCTURES", "ALLIES ENEMIES STRUCTURES CONTROLLED_STRUCTURES", "ENEMIES SHRUBBERY WALLS",
        "ALLIES PROJECTILES", "SMALL_MISSILES", "BALLISTIC_MISSILES", "NONE",
        "ALLIES ENEMIES STRUCTURES SHRUBBERY PROJECTILES WALLS SMALL_MISSILES BALLISTIC_MISSILES CONTROLLED_STRUCTURES"};
    static const char* const kDelay[] = {"1000", "Min:100 Max:900", "Min:200 Max:5000", "33"};
    for (int i = 0; i < kWeapons; ++i) {
        s += "Weapon M_W" + std::to_string(i) +
             "\n  PrimaryDamage = 0.0\n  PrimaryDamageRadius = 0.0\n  AttackRange = " +
             std::to_string(100 + pick(500)) + ".0\n  MinimumAttackRange = " + std::to_string(pick(60)) +
             ".0\n  HistoricBonusCount = 1000000\n  HistoricBonusTime = 100000000\n  HistoricBonusRadius = 1.0\n"
             "  ClipSize = 0\n";
        s += std::string("  DelayBetweenShots = ") + choose(kDelay) + "\n";
        s += std::string("  DamageType = ") + kDamageType[i % 4] + "\n";
        s += std::string("  WeaponSpeed = ") + choose(kSpeed) + "\n";
        s += std::string("  MinWeaponSpeed = ") + choose(kMinSpeed) + "\n";
        s += std::string("  ScaleWeaponSpeed = ") + (i >= 4 ? "Yes" : "No") + "\n";
        s += std::string("  ProjectileCollidesWith = ") + (i < 2 ? kCollides[7] : choose(kCollides)) + "\n";
        s += "End\n";
    }

    // Helicopters.
    static const char* const kTurn[] = {"0", "15", "90", "-120", "360", nullptr};
    static const char* const kFwd[] = {"0", "30", "150", "-45", "300", nullptr};
    static const char* const kDamp[] = {"1.0", "0.9", "0.99", "1.1", nullptr};
    static const char* const kSpin[] = {"0", "10", "90", "180", "-30", nullptr};
    static const char* const kSpinDelay[] = {"0", "33", "100", "400", "1000", "-100", nullptr};
    static const char* const kSpinAmt[] = {"0", "1", "10", "90", "300", nullptr};
    static const char* const kPct[] = {"0%", "10%", "50%", "100%", "150%", nullptr};
    static const char* const kBlade[] = {"0", "33", "100", "200", nullptr};
    static const char* const kGroundDelay[] = {"0", "33", "50", "200", "1000", nullptr};
    static const char* const kBraking[] = {"0", "100", "5000", nullptr};
    s += "Locomotor M_Loco\n  Surfaces = AIR\n  Speed = 60\n  SpeedDamaged = 60\n  TurnRate = 180\n"
         "  TurnRateDamaged = 180\n  Acceleration = 60\n  Braking = 60\n  Appearance = HOVER\nEnd\n";
    s += "Object M_Rubble\n  KindOf = PRELOAD VEHICLE\n  Body = InactiveBody ModuleTag_Body\n  End\nEnd\n";
    s += "Object M_Shrub\n  KindOf = PRELOAD SHRUBBERY IMMOBILE\n  Body = InactiveBody ModuleTag_Body\n  End\nEnd\n";
    s += "Object M_Rock\n  KindOf = PRELOAD VEHICLE IMMOBILE\n  Body = InactiveBody ModuleTag_Body\n  End\nEnd\n";
    for (int i = 0; i < kHeli; ++i) {
        s += "Object M_Heli" + std::to_string(i) + "\n  KindOf = PRELOAD VEHICLE\n";
        s += active_body("100.0");
        s += "  Behavior = PhysicsBehavior ModuleTag_Phys\n    Mass = " + std::to_string(1 + pick(50)) +
             ".0\n  End\n";
        const bool ai = i % 4 != 0;
        if (ai) s += "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n  Locomotor = SET_NORMAL M_Loco\n";
        s += "  Behavior = HelicopterSlowDeathBehavior ModuleTag_Death\n";
        s += opt("SpiralOrbitTurnRate", choose(kTurn));
        s += opt("SpiralOrbitForwardSpeed", choose(kFwd));
        s += opt("SpiralOrbitForwardSpeedDamping", choose(kDamp));
        s += opt("MinSelfSpin", choose(kSpin));
        s += opt("MaxSelfSpin", choose(kSpin));
        s += opt("SelfSpinUpdateDelay", choose(kSpinDelay));
        s += opt("SelfSpinUpdateAmount", choose(kSpinAmt));
        s += opt("FallHowFast", choose(kPct));
        s += opt("MinBladeFlyOffDelay", choose(kBlade));
        s += opt("MaxBladeFlyOffDelay", choose(kBlade));
        s += std::string("    BladeBoneName = ") + kBoneName + "\n";
        s += asset("OCLEjectPilot", "M_O", kOcl);
        s += asset("FXBlade", "M_F", kFx);
        s += asset("OCLBlade", "M_O", kOcl);
        s += asset("FXHitGround", "M_F", kFx);
        s += asset("OCLHitGround", "M_O", kOcl);
        s += asset("FXFinalBlowUp", "M_F", kFx);
        s += asset("OCLFinalBlowUp", "M_O", kOcl);
        s += opt("DelayFromGroundToFinalDeath", choose(kGroundDelay));
        s += "    FinalRubbleObject = M_Rubble\n";
        s += opt("MaxBraking", choose(kBraking));
        static const char* const kDestroy[] = {"1000000", "1000000", "1000000", "200", "0"};
        s += std::string("    DestructionDelay = ") + choose(kDestroy) + "\n";
        if (chance(30)) s += "    SinkDelay = 100\n    SinkRate = 3.0\n";
        if (chance(20)) s += "    FlingForce = 8.0\n    FlingForceVariance = 4.0\n    FlingPitch = 60\n";
        if (chance(30))
            s += "    FX = INITIAL M_F" + std::to_string(pick(kFx)) + "\n    OCL = FINAL M_O" +
                 std::to_string(pick(kOcl)) + "\n";
        s += "  End\nEnd\n";
    }

    // Projectiles.
    static const char* const kLife[] = {nullptr, nullptr, "0", "33", "200", "1000"};
    static const char* const kYesNo[] = {nullptr, "Yes", "No", "No"};
    static const char* const kHeight[] = {nullptr, "0", "25.5", "100", "-20"};
    static const char* const kIndent[] = {nullptr, "0%", "20%", "50%", "90%", "130%"};
    static const char* const kAdjust[] = {nullptr, "0", "30", "300", "3000"};
    static const char* const kRequired[] = {nullptr, "INFANTRY", "INFANTRY HERO"};
    static const char* const kForbidden[] = {nullptr, "HERO", "NONE"};
    static const char* const kKill[] = {nullptr, "0", "1", "2", "10"};
    static const char* const kClearer[] = {"1", "2", "3", "10"};
    for (int i = 0; i < kProj; ++i) {
        s += "Object M_Proj" + std::to_string(i) + "\n  KindOf = PRELOAD PROJECTILE\n";
        s += active_body("100.0");
        if (i % 3 != 0) s += "  Behavior = PhysicsBehavior ModuleTag_Phys\n  End\n";
        s += "  Behavior = DumbProjectileBehavior ModuleTag_Flight\n";
        s += opt("MaxLifespan", choose(kLife));
        s += opt("TumbleRandomly", choose(kYesNo));
        s += opt("DetonateCallsKill", choose(kYesNo));
        s += opt("OrientToFlightPath", choose(kYesNo));
        s += opt("FirstHeight", choose(kHeight));
        s += opt("SecondHeight", choose(kHeight));
        s += opt("FirstPercentIndent", choose(kIndent));
        s += opt("SecondPercentIndent", choose(kIndent));
        s += opt("FlightPathAdjustDistPerSecond", choose(kAdjust));
        const bool clearer = i < 10;
        s += opt("GarrisonHitKillCount", clearer ? choose(kClearer) : choose(kKill));
        s += opt("GarrisonHitKillRequiredKindOf", choose(kRequired));
        s += opt("GarrisonHitKillForbiddenKindOf", choose(kForbidden));
        s += clearer ? "    GarrisonHitKillFX = M_F" + std::to_string(i % kFx) + "\n"
                     : asset("GarrisonHitKillFX", "M_F", kFx);
        s += "  End\nEnd\n";
    }
    for (int i = 0; i < 2; ++i) {
        s += "Object M_Launcher" + std::to_string(i) + "\n  KindOf = PRELOAD VEHICLE IMMOBILE\n";
        s += active_body("1000.0");
        if (i == 0) s += "  Behavior = PhysicsBehavior ModuleTag_Phys\n  End\n";
        s += "End\n";
    }
    for (int i = 0; i < kTargets; ++i) {
        s += "Object M_Target" + std::to_string(i) + "\n  KindOf = PRELOAD " + kTargetKinds[i] + " IMMOBILE\n";
        if (i == 6) s += "  FenceWidth = 3.0\n";
        s += "  Geometry = CYLINDER\n  GeometryMajorRadius = 4.0\n  GeometryHeight = " +
             std::to_string(2 + i) + ".0\n";
        s += active_body("1000.0");
        if (i == 7) s += "  Behavior = ParkingPlaceBehavior ModuleTag_Park\n    NumRows = 1\n    NumCols = 2\n    HasRunways = Yes\n  End\n";
        if (i == 8 || i == 9) {
            s += "  Behavior = GarrisonContain ModuleTag_Contain\n    ContainMax = 6\n";
            if (i == 9) s += "    ImmuneToClearBuildingAttacks = Yes\n";
            s += "  End\n";
        }
        if (i == 10) s += "  Behavior = TransportContain ModuleTag_Contain\n    Slots = 6\n    AllowInsideKindOf = INFANTRY\n  End\n";
        s += "End\n";
    }
    static const char* const kSoldierKinds[3] = {"INFANTRY", "INFANTRY HERO", "INFANTRY SALVAGER"};
    for (int i = 0; i < 3; ++i) {
        s += "Object M_Soldier" + std::to_string(i) + "\n  KindOf = PRELOAD " + kSoldierKinds[i] + "\n";
        s += "  TransportSlotCount = 1\n";
        s += active_body("50.0");
        s += "End\n";
    }

    // Formations.
    for (int i = 0; i < kTensile; ++i) {
        s += "Object M_Tensile" + std::to_string(i) + "\n  KindOf = PRELOAD IMMOBILE\n";
        s += "  Geometry = CYLINDER\n  GeometryMajorRadius = " + std::to_string(i < 2 ? 25 : 55) +
             ".0\n  GeometryHeight = 10.0\n";
        s += active_body("100.0");
        s += "  Behavior = TensileFormationUpdate ModuleTag_Tensile\n";
        if (i % 2) s += "    Enabled = Yes\n";
        s += "  End\nEnd\n";
    }
    for (int i = 0; i < 2; ++i) {
        s += std::string("Object M_Tree") + std::to_string(i) + "\n  KindOf = PRELOAD " +
             (i ? "VEHICLE" : "SHRUBBERY") + " IMMOBILE\n  Geometry = CYLINDER\n  GeometryMajorRadius = 3.0\n"
             "  GeometryHeight = 20.0\n";
        s += active_body("100.0");
        s += "  Behavior = ToppleUpdate ModuleTag_Topple\n  End\nEnd\n";
    }

    // Timers.
    static const char* const kMin[] = {nullptr, "0", "33", "100", "500"};
    static const char* const kMax[] = {nullptr, "0", "100", "300", "1000"};
    static const char* const kSides[] = {"America", "China", "GLA", "america"};
    for (int i = 0; i < kOclVariants; ++i) {
        s += "Object M_Ocl" + std::to_string(i) + "\n  KindOf = PRELOAD STRUCTURE IMMOBILE\n";
        s += "  Body = InactiveBody ModuleTag_Body\n  End\n";
        s += "  Behavior = OCLUpdate ModuleTag_Ocl\n";
        s += asset("OCL", "M_O", kOcl);
        s += opt("MinDelay", choose(kMin));
        s += opt("MaxDelay", choose(kMax));
        if (i % 3 == 0) s += "    CreateAtEdge = Yes\n";
        if (i % 2 == 0) s += "    FactionTriggered = Yes\n";
        const int n = static_cast<int>(pick(4));
        for (int k = 0; k < n; ++k)
            s += std::string("    FactionOCL = Faction:") + choose(kSides) + " OCL:M_O" +
                 std::to_string(pick(kOcl)) + "\n";
        s += "  End\nEnd\n";
    }
    return s;
}

// ---------------------------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------------------------
static oracle_wp51::World* W;
// Reseeded after the setup: spawning an AI object draws from the logic stream.
static uint32_t g_logic_seed;
static std::vector<Object*> g_spawned;

static Object* spawn(const std::string& name, Team* team, const Coord3D& pos, Real angle) {
    Object* o = W->spawn_object(name.c_str(), team, pos, angle);
    if (!o) die(("cannot spawn " + name).c_str());
    g_spawned.push_back(o);
    return o;
}
static Team* team_of(uint32_t i) { return i == 0 ? W->team_neutral : i == 1 ? W->team_a : W->team_b; }

template <typename T>
static T* find_behavior(Object* obj) {
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m)
        if (T* typed = dynamic_cast<T*>(*m)) return typed;
    return nullptr;
}

static std::string head(UpdateModule* m, uint32_t ret) {
    return oracle_wp51::World::snapshot_save_hex(m) + " " + num(oracle_wp51::World::snapshot_crc(m)) + " " +
           num(ret) + " " + num(m->friend_getNextCallFrame());
}
static std::string matrix_of(Object* o) {
    std::string s;
    const Matrix3D* m = o->getTransformMatrix();
    for (int r = 0; r < 3; ++r)
        for (int c = 0; c < 4; ++c) s += (s.empty() ? "" : ",") + rb((*m)[r][c]);
    return s;
}
static std::string seed() { return num(GetGameLogicRandomSeedCRC()); }
static std::string cond(Object* o, ModelConditionFlagType f) {
    return o->getDrawable()->getModelConditionFlags().test(f) ? "1" : "0";
}

struct Vec {
    std::string setup;
    std::vector<std::string> ops, rows;
    void field(const std::string& v) { setup += (setup.empty() ? "" : " ") + v; }
};

static std::string pos_text(const Coord3D& p) { return rb(p.x) + " " + rb(p.y) + " " + rb(p.z); }

// ---------------------------------------------------------------------------------------------
// Helicopter
// ---------------------------------------------------------------------------------------------
static void run_heli(Vec& v, UnsignedInt frame) {
    const uint32_t variant = pick(kHeli);
    const uint32_t team = pick(3);
    Coord3D pos;
    pos.x = grid(100, 900);
    pos.y = grid(100, 900);
    const Real ground = TheTerrainLogic->getGroundHeight(pos.x, pos.y);
    pos.z = ground + (chance(30) ? grid(-2, 3) : grid(3, 60));
    const Real angle = grid(-3, 3);
    Object* o = spawn("M_Heli" + std::to_string(variant), team_of(team), pos, angle);
    Coord3D tp = {10.0f, 10.0f, 0.0f};
    Object* shrub = spawn("M_Shrub", W->team_neutral, tp, 0.0f);
    tp.x = 30.0f;
    Object* rock = spawn("M_Rock", W->team_neutral, tp, 0.0f);
    HelicopterSlowDeathBehavior* hb = find_behavior<HelicopterSlowDeathBehavior>(o);
    PhysicsBehavior* phys = o->getPhysics();
    if (!hb || !phys) die("helicopter modules");
    AIUpdateInterface* ai = o->getAIUpdateInterface();

    Drawable* d = o->getDrawable();
    const uint32_t bones = pick(4);  // 0 none installed, 1 installed but empty, 2-3 a bone
    static ModuleData draw_data;
    Module* fake[2] = {nullptr, nullptr};
    Module** saved = nullptr;
    g_have_bone = bones >= 2;
    g_bone.x = grid(-20, 20);
    g_bone.y = grid(-20, 20);
    g_bone.z = grid(-5, 20);
    if (bones != 0) {
        fake[0] = newInstance(OracleDraw)(d, &draw_data);
        saved = (d->*access_member(DrawArrayTag()))[0];
        (d->*access_member(DrawArrayTag()))[0] = fake;
    }

    // `FXList` asks the object's shroud state, which needs it filed in cells.
    ThePartitionManager->update();

    v.field(num(variant));
    v.field(num(team));
    v.field(pos_text(pos));
    v.field(rb(angle));
    v.field(num(g_have_bone ? 1 : 0));
    v.field(pos_text(g_bone));

    std::set<Object*> known(g_spawned.begin(), g_spawned.end());
    std::vector<Object*> rubble;
    auto row = [&](uint32_t ret) {
        std::string r = head(hb, ret) + " " + matrix_of(o) + " ";
        r += xyz(phys->*access_member(MPhysAccel())) + " " + num(phys->*access_member(MPhysExpires())) + " ";
        Locomotor* loco = ai ? ai->getCurLocomotor() : nullptr;
        struct Open : HelicopterSlowDeathBehavior { using HelicopterSlowDeathBehavior::isSlowDeathActivated; };
        if (loco && (hb->*(&Open::isSlowDeathActivated))())
            r += rb(loco->*access_member(MLocoLift())) + "," + rb(loco->*access_member(MLocoBraking())) + " ";
        else
            r += "- ";
        r += std::string(o->isDisabledByType(DISABLED_HELD) ? "1" : "0") + cond(o, MODELCONDITION_SPECIAL_DAMAGED) +
             (o->isDestroyed() ? "1" : "0") + " " + num(o->getLayer()) + " ";
        std::string made;
        for (Object* u = TheGameLogic->getFirstObject(); u; u = u->getNextObject()) {
            if (known.count(u)) continue;
            known.insert(u);
            if (u->getTemplate()->getName() != "M_Rubble") die("unexpected new object");
            made += num(u->getControllingPlayer()->getPlayerIndex()) + "," + matrix_of(u) + ";";
            g_spawned.push_back(u);
        }
        r += dash(made) + " " + take(g_fx) + " " + take(g_ocl) + " " + seed();
        v.rows.push_back(r);
    };

    InitRandom(g_logic_seed);
    const int ops = 6 + static_cast<int>(pick(18));
    bool started = false;
    const int start_at = static_cast<int>(pick(3));
    for (int k = 0; k < ops && !o->isDestroyed(); ++k) {
        uint32_t code = pick(10);
        if (!started && k >= start_at) code = pick(8) == 0 ? 1 : 0;
        else if (code < 2) code = pick(12) == 0 ? 0 : 2;
        uint32_t ret = 0;
        std::string op;
        if (code == 0 || code == 1) {  // 0 begin directly, 1 die
            const uint32_t d1 = pick(3);
            frame += d1;
            W->set_frame(frame);
            op = num(code) + " " + num(d1);
            if (code == 0) {
                DamageInfo info;
                static_cast<SlowDeathBehaviorInterface*>(hb)->beginSlowDeath(&info);
            } else if (!o->isEffectivelyDead()) {
                o->kill();
            }
            started = true;
        } else if (code == 3) {  // collide with something
            const uint32_t what = pick(3);
            op = "3 " + num(what);
            phys->*access_member(MPhysCollidee()) =
                what == 0 ? INVALID_ID : what == 1 ? shrub->getID() : rock->getID();
        } else if (code == 4) {  // move vertically
            Coord3D p = *o->getPosition();
            const Real g = TheTerrainLogic->getGroundHeight(p.x, p.y);
            p.z = g + (chance(40) ? grid(0, 2) : grid(1, 80));
            if (chance(15)) p.z = g + 1.0f;
            op = "4 " + rb(p.z);
            o->setPosition(&p);
        } else if (code == 5) {  // veterancy
            const uint32_t level = pick(4);
            op = "5 " + num(level);
            o->getExperienceTracker()->setVeterancyLevel(static_cast<VeterancyLevel>(level), FALSE);
        } else {  // update
            const uint32_t d1 = chance(60) ? 1 : chance(70) ? pick(12) : pick(80);
            frame += d1;
            W->set_frame(frame);
            op = "2 " + num(d1);
            ret = static_cast<uint32_t>(hb->update());
        }
        v.ops.push_back(op);
        row(ret);
    }
    if (saved) {
        (d->*access_member(DrawArrayTag()))[0] = saved;
        deleteInstance(static_cast<OracleDraw*>(fake[0]));
    }
}

// ---------------------------------------------------------------------------------------------
// Projectile
// ---------------------------------------------------------------------------------------------
static uint32_t fnv_path(const VecCoord3D& path) {
    uint32_t h = 2166136261u;
    for (const Coord3D& p : path) {
        const uint32_t w[3] = {bits(p.x), bits(p.y), bits(p.z)};
        for (uint32_t x : w) h = (h ^ x) * 16777619u;
    }
    return h;
}

static Coord3D map_point() {
    Coord3D p;
    if (chance(6)) {  // off the map
        p.x = chance(50) ? grid(-900, -300) : grid(1300, 1900);
        p.y = grid(-200, 1200);
    } else {
        p.x = grid(20, 980);
        p.y = grid(20, 980);
    }
    p.z = grid(0, 80);
    return p;
}

static Coord3D near_point(const Coord3D& c, int span) {
    Coord3D p;
    p.x = c.x + grid(-span, span);
    p.y = c.y + grid(-span, span);
    p.z = grid(0, 80);
    return p;
}

static void run_proj(Vec& v, UnsignedInt frame) {
    // A projectile aimed into a garrisoned building.
    const bool assault = chance(25);
    const uint32_t variant = assault ? pick(10) : pick(kProj);
    const uint32_t proj_team = pick(3);
    Coord3D base;
    base.x = grid(200, 800);
    base.y = grid(200, 800);
    base.z = grid(10, 60);
    if (chance(5)) base.x = -700.0f;
    const Real angle = grid(-3, 3);

    const uint32_t launcher_kind = pick(2);
    const uint32_t launcher_team = pick(3);
    Coord3D lp = chance(94) ? near_point(base, 30) : map_point();
    const Real langle = grid(-3, 3);
    Object* launcher = spawn("M_Launcher" + std::to_string(launcher_kind), team_of(launcher_team), lp, langle);
    // Where the launcher sits: 0 outside, 1 inside the garrison target, 2 inside the transport.
    const uint32_t inside = chance(12) ? 1 + pick(2) : 0;
    Coord3D vel = {grid(-4, 4), grid(-4, 4), grid(-2, 2)};
    if (launcher->getPhysics()) launcher->getPhysics()->*access_member(MPhysVel()) = vel;
    const uint32_t flag_bits = chance(60) ? 0 : pick(16);
    for (int b = 0; b < 4; ++b)
        if (flag_bits & (1u << b)) launcher->setWeaponBonusCondition(static_cast<WeaponBonusConditionType>(b));

    const uint32_t target_kind = assault ? (chance(80) ? 8 : 9 + pick(2))
                                 : chance(30) ? 8 + pick(3)
                                 : chance(15) ? 7
                                              : pick(kTargets);
    const uint32_t target_team = pick(3);
    Coord3D tp = near_point(base, chance(80) ? 150 : 12);
    Object* target = spawn("M_Target" + std::to_string(target_kind), team_of(target_team), tp, 0.0f);
    const bool burned = chance(25);
    if (burned) target->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_BURNED));

    const uint32_t victim_team = pick(3);
    Coord3D vp = near_point(base, 120);
    Object* victim = spawn("M_Target3", team_of(victim_team), vp, 0.0f);

    const bool reserved = target_kind == 7 && chance(70);
    if (reserved) {
        ParkingPlaceBehavior* pp = find_behavior<ParkingPlaceBehavior>(target);
        if (!pp) die("parking");
        pp->reserveSpace(victim->getID(), 0.0f, nullptr);
        if (!pp->hasReservedSpace(victim->getID())) die("not reserved");
    }

    std::vector<Object*> soldiers;
    std::string soldier_text;
    Object* holder = target;
    if (target_kind >= 8 && target_kind <= 10) {
        const uint32_t n = assault ? 1 + pick(5) : pick(5);
        soldier_text = num(n);
        for (uint32_t i = 0; i < n; ++i) {
            const uint32_t kind = pick(3);
            const uint32_t team = target_kind == 10 ? target_team : target_team == 0 ? 1 : target_team;
            Object* s = spawn("M_Soldier" + std::to_string(kind), team_of(team), tp, 0.0f);
            holder->getContain()->addToContain(s);
            if (s->getContainedBy() != holder) die("not contained");
            soldiers.push_back(s);
            soldier_text += "," + num(kind) + "," + num(team);
        }
    } else {
        soldier_text = "0";
    }
    if (inside != 0 && target_kind == (inside == 1 ? 8u : 10u) && launcher_kind == 1) {
        holder->getContain()->addToContain(launcher);
    }
    const bool contained = launcher->getContainedBy() != nullptr;

    ThePartitionManager->update();

    Object* o = spawn("M_Proj" + std::to_string(variant), team_of(proj_team), base, angle);
    DumbProjectileBehavior* db = find_behavior<DumbProjectileBehavior>(o);
    if (!db) die("projectile module");

    v.field(num(variant));
    v.field(num(proj_team));
    v.field(pos_text(base));
    v.field(rb(angle));
    v.field(num(launcher_kind));
    v.field(num(launcher_team));
    v.field(pos_text(lp));
    v.field(rb(langle));
    v.field(pos_text(vel));
    v.field(num(flag_bits));
    v.field(num(target_kind));
    v.field(num(target_team));
    v.field(pos_text(tp));
    v.field(num(burned));
    v.field(num(victim_team));
    v.field(pos_text(vp));
    v.field(num(reserved));
    v.field(soldier_text);
    v.field(num(contained ? target_kind : 0));

    AcademyStats* stats[3];
    for (int i = 0; i < 3; ++i) stats[i] = team_of(i)->getControllingPlayer()->getAcademyStats();
    UnsignedInt cleared_before[3];
    for (int i = 0; i < 3; ++i) cleared_before[i] = stats[i]->*access_member(MCleared());
    for (int i = 0; i < kWeapons; ++i) {
        const WeaponTemplate* t = TheWeaponStore->findWeaponTemplate(("M_W" + std::to_string(i)).c_str());
        (const_cast<WeaponTemplate*>(t)->*access_member(MHistoric())).clear();
    }

    Object* current = o;
    DumbProjectileBehavior* m = db;
    auto row = [&](uint32_t ret) {
        std::string r = head(m, ret) + " " + matrix_of(current) + " " + num(current->getLayer()) + " ";
        r += num(m->*access_member(MDumbStep())) + " ";
        r += std::string(current->isDestroyed() ? "1" : "0") + (current->isEffectivelyDead() ? "1" : "0") +
             (current->getStatusBits().test(OBJECT_STATUS_NO_COLLISIONS) ? "1" : "0") +
             (m->*access_member(MDumbDetonated()) ? "1" : "0") + " ";
        r += rb(current->getBodyModule()->getHealth()) + " ";
        const VecCoord3D& path = m->*access_member(MPathTag());
        r += num(path.size()) + "," + num(fnv_path(path)) + " ";
        PhysicsBehavior* p = current->getPhysics();
        if (p) {
            r += xyz(p->*access_member(MPhysVel())) + "," + num(p->*access_member(MPhysIgnore())) + "," +
                 rb(p->*access_member(MPhysPitch())) + "," + rb(p->*access_member(MPhysYaw())) + "," +
                 rb(p->*access_member(MPhysRoll())) + " ";
        } else {
            r += "- ";
        }
        r += num(current->getExperienceTracker()->*access_member(MSink())) + " " +
             num(m->*access_member(MDumbFlags())) + " ";
        std::string shots;
        for (int i = 0; i < kWeapons; ++i) {
            const WeaponTemplate* t = TheWeaponStore->findWeaponTemplate(("M_W" + std::to_string(i)).c_str());
            HistoricWeaponDamageList& list = const_cast<WeaponTemplate*>(t)->*access_member(MHistoric());
            for (const HistoricWeaponDamageInfo& h : list) shots += num(i) + "," + xyz(h.location) + ";";
            list.clear();
        }
        r += dash(shots) + " ";
        std::string dead;
        for (Object* s : soldiers) dead += s->isEffectivelyDead() ? "1" : "0";
        r += dash(dead) + " ";
        std::string cleared;
        for (int i = 0; i < 3; ++i) cleared += num((stats[i]->*access_member(MCleared())) - cleared_before[i]);
        r += cleared + " " + take(g_fx) + " " + seed();
        v.rows.push_back(r);
    };

    InitRandom(g_logic_seed);
    const uint32_t weapon = assault ? pick(2) : chance(10) ? kWeapons : pick(kWeapons);
    auto weapon_of = [&](uint32_t i) -> const WeaponTemplate* {
        if (i >= static_cast<uint32_t>(kWeapons)) return nullptr;
        return TheWeaponStore->findWeaponTemplate(("M_W" + std::to_string(i)).c_str());
    };

    const int ops = 4 + static_cast<int>(pick(14));
    bool launched = false;
    uint32_t current_weapon = kWeapons;
    int extra = 0;
    for (int k = 0; k < ops; ++k) {
        uint32_t code = pick(12);
        if (!launched && k >= static_cast<int>(pick(2))) code = chance(80) || assault ? 0 : 1;
        else if (code < 2 && !chance(10)) code = 2;
        if (assault && launched && k < 4 && chance(50)) code = 4;
        if (current->isDestroyed() && ++extra > 2) break;
        uint32_t ret = 0;
        std::string op;
        W->set_frame(frame);
        if (code <= 1) {  // 0 launch, 1 fire in place
            const uint32_t with_victim = pick(3);  // 0 position, 1 victim, 2 the target as victim
            Object* vic = with_victim == 0 ? nullptr : with_victim == 1 ? victim : target;
            Coord3D aim = chance(80) ? near_point(base, chance(85) ? 150 : 8) : map_point();
            if (chance(4)) aim = *current->getPosition();
            const uint32_t with_launcher = chance(90) ? 1 : 0;
            if (chance(8)) {  // straight up or down
                aim = code == 0 && with_launcher ? *launcher->getPosition() : *current->getPosition();
                aim.z += grid(-60, 60);
            }
            op = num(code) + " " + num(with_victim) + " " + pos_text(aim) + " " + num(weapon) + " " +
                 num(with_launcher);
            if (code == 0)
                m->projectileLaunchAtObjectOrPosition(vic, &aim, with_launcher ? launcher : nullptr, PRIMARY_WEAPON,
                                                      0, weapon_of(weapon), nullptr);
            else
                m->projectileFireAtObjectOrPosition(vic, &aim, weapon_of(weapon), nullptr);
            launched = true;
            // Only the launch keeps the detonation weapon.
            if (code == 0) current_weapon = weapon;
        } else if (code == 3) {  // move the victim
            Coord3D p = chance(30) ? *victim->getPosition() : near_point(*victim->getPosition(), chance(60) ? 20 : 200);
            if (chance(10)) p = map_point();
            if (chance(10)) {  // a hair
                p = *victim->getPosition();
                p.x += 0.25f;
            }
            op = "3 " + pos_text(p);
            victim->setPosition(&p);
        } else if (code == 4) {  // collide
            // 0 nothing, 1 the target, 2 the victim, 3 the launcher, 4 a soldier
            uint32_t what = chance(55) ? 1 : pick(5);
            if (what == 4 && soldiers.empty()) what = 1;
            Object* other = what == 0 ? nullptr : what == 1 ? target : what == 2 ? victim : what == 3 ? launcher : soldiers[0];
            // Without a detonation weapon the reference dereferences null.
            if (current_weapon >= static_cast<uint32_t>(kWeapons)) what = 0, other = nullptr;
            op = "4 " + num(what);
            if (!current->isDestroyed()) ret = m->projectileHandleCollision(other) ? 1 : 0;
        } else if (code == 6) {  // as if it had flown onto a bridge
            const uint32_t layer = 1 + pick(3);
            op = "6 " + num(layer);
            current->setLayer(static_cast<PathfindLayerEnum>(layer));
        } else if (code == 5) {  // save, load into a new projectile and carry on with it
            oracle::MemoryXfer xs(XFER_SAVE);
            xs.open("save");
            oracle::call_snapshot_xfer(m, &xs);
            xs.close();
            op = "5";
            Object* copy = spawn("M_Proj" + std::to_string(variant), team_of(proj_team), base, angle);
            DumbProjectileBehavior* cb = find_behavior<DumbProjectileBehavior>(copy);
            // Loading drops a live module's scheduler index (README quirk); keep it.
            const Int index = cb->friend_getIndexInLogic();
            oracle_wp51::World::snapshot_load_bytes(cb, xs.bytes);
            cb->friend_setIndexInLogic(index);
            struct Load : DumbProjectileBehavior { using DumbProjectileBehavior::loadPostProcess; };
            (cb->*(&Load::loadPostProcess))();
            copy->setTransformMatrix(current->getTransformMatrix());
            copy->setPosition(current->getPosition());
            current = copy;
            m = cb;
        } else {  // update
            uint32_t d1 = chance(80) ? 1 : chance(70) ? pick(10) : pick(400);
            frame += d1;
            W->set_frame(frame);
            op = "2 " + num(d1);
            ret = static_cast<uint32_t>(m->update());
        }
        v.ops.push_back(op);
        row(ret);
    }
    if (reserved) {
        ParkingPlaceBehavior* pp = find_behavior<ParkingPlaceBehavior>(target);
        pp->releaseSpace(victim->getID());
    }
}

// ---------------------------------------------------------------------------------------------
// Formation
// ---------------------------------------------------------------------------------------------
static void run_tensile(Vec& v, UnsignedInt frame) {
    Coord3D c;
    c.x = grid(350, 650);
    c.y = grid(350, 650);
    c.z = 0.0f;
    const uint32_t members = 1 + pick(7);
    const uint32_t variant = pick(kTensile);
    const uint32_t trees = pick(4);
    std::string setup = num(members);
    std::vector<Object*> all;
    std::vector<TensileFormationUpdate*> tfus;
    for (uint32_t i = 0; i < members; ++i) {
        const uint32_t kind = i == 0 ? variant : pick(kTensile);
        Coord3D p = c;
        if (i != 0) {
            const int span = chance(60) ? 40 : chance(70) ? 120 : 300;
            p.x += grid(-span, span);
            p.y += grid(-span, span);
            if (chance(15)) p.x = c.x, p.y = c.y;
        }
        p.z = TheTerrainLogic->getGroundHeight(p.x, p.y) + (chance(20) ? grid(0, 20) : 0.0f);
        Object* o = spawn("M_Tensile" + std::to_string(kind), W->team_neutral, p, 0.0f);
        all.push_back(o);
        tfus.push_back(find_behavior<TensileFormationUpdate>(o));
        setup += " " + num(kind) + " " + pos_text(p);
    }
    setup += " " + num(trees);
    std::vector<Object*> tree_objs;
    for (uint32_t i = 0; i < trees; ++i) {
        const uint32_t kind = chance(75) ? 0 : 1;
        Coord3D p = c;
        const int reach = chance(50) ? 3 : 12;
        p.x += grid(-reach, reach);
        p.y += grid(-reach, reach);
        p.z = TheTerrainLogic->getGroundHeight(p.x, p.y);
        tree_objs.push_back(spawn("M_Tree" + std::to_string(kind), W->team_neutral, p, 0.0f));
        setup += " " + num(kind) + " " + pos_text(p);
    }
    ThePartitionManager->update();
    v.field(setup);

    // Every member's update runs; the row reports the one that was updated.
    auto row = [&](uint32_t who, uint32_t ret) {
        TensileFormationUpdate* t = tfus[who];
        Object* u = all[who];
        std::string r = head(t, ret) + " " + matrix_of(u) + " ";
        r += cond(u, MODELCONDITION_MOVING) + cond(u, MODELCONDITION_FREEFALL) +
             cond(u, MODELCONDITION_POST_COLLAPSE) + " ";
        std::string health;
        for (Object* x : all)
            health += rb(x->getBodyModule()->getHealth()) + "," + num(x->getBodyModule()->getDamageState()) + ";";
        r += health + " ";
        std::string tree;
        for (Object* x : tree_objs) {
            // The module CRC does not cover the topple state; hash the saved bytes.
            ToppleUpdate* tu = find_behavior<ToppleUpdate>(x);
            uint32_t h = 2166136261u;
            for (char ch : oracle_wp51::World::snapshot_save_hex(tu)) h = (h ^ static_cast<uint8_t>(ch)) * 16777619u;
            tree += num(h) + ";";
        }
        r += dash(tree) + " " + seed();
        v.rows.push_back(r);
    };

    InitRandom(g_logic_seed);
    const int ops = 4 + static_cast<int>(pick(9));
    for (int k = 0; k < ops; ++k) {
        uint32_t code = pick(10);
        uint32_t who = chance(75) ? 0 : pick(members);
        if (k == 0 && chance(50)) code = 0;
        uint32_t ret = 0;
        std::string op;
        W->set_frame(frame);
        if (code <= 1) {  // damage a member
            static const Real kAmounts[] = {10.0f, 49.0f, 51.0f, 75.0f, 95.0f, 200.0f};
            const uint32_t a = pick(6);
            op = "0 " + num(who) + " " + num(a);
            DamageInfo info;
            info.in.m_damageType = DAMAGE_UNRESISTABLE;
            info.in.m_deathType = DEATH_NORMAL;
            info.in.m_amount = kAmounts[a];
            all[who]->attemptDamage(&info);
        } else if (code == 2) {  // move a member
            Coord3D p = *all[who]->getPosition();
            p.x += grid(-40, 40);
            p.y += grid(-40, 40);
            op = "2 " + num(who) + " " + pos_text(p);
            all[who]->setPosition(&p);
            ThePartitionManager->update();
        } else {  // run one member's update many times
            const uint32_t times = chance(55) ? 1 : chance(60) ? pick(60) : 60 + pick(320);
            op = "1 " + num(who) + " " + num(times);
            for (uint32_t t = 0; t < times; ++t) {
                // The reference dereferences the closest object; keep one within reach.
                ret = static_cast<uint32_t>(tfus[who]->update());
                ThePartitionManager->update();
                if (ret == static_cast<uint32_t>(UPDATE_SLEEP_FOREVER)) break;
            }
        }
        v.ops.push_back(op);
        row(who, ret);
    }
    (void)frame;
}

// ---------------------------------------------------------------------------------------------
// Timer
// ---------------------------------------------------------------------------------------------
static void run_ocl(Vec& v, UnsignedInt frame) {
    const uint32_t variant = pick(kOclVariants);
    uint32_t team = pick(3);
    Coord3D pos;
    pos.x = grid(0, 1000);
    pos.y = grid(0, 1000);
    if (chance(15)) pos.y = pos.x;  // ties between edges
    if (chance(10)) pos.x = 500.0f, pos.y = 500.0f;
    pos.z = grid(0, 40);
    const Real angle = grid(-3, 3);
    uint32_t colors[3] = {pick(3), pick(3), pick(3)};
    for (int i = 0; i < 3; ++i) team_of(i)->getControllingPlayer()->*access_member(MColor()) = colors[i];
    W->set_frame(frame);
    Object* o = spawn("M_Ocl" + std::to_string(variant), team_of(team), pos, angle);
    OCLUpdate* m = find_behavior<OCLUpdate>(o);
    if (!m) die("OCL module");
    v.field(num(variant));
    v.field(num(team));
    v.field(pos_text(pos));
    v.field(rb(angle));
    v.field(num(colors[0]) + " " + num(colors[1]) + " " + num(colors[2]));

    auto row = [&](uint32_t ret) {
        std::string r = head(m, ret) + " " + rb(m->getCountdownPercent()) + " " + num(m->getRemainingFrames()) + " ";
        r += take(g_ocl) + " " + seed();
        v.rows.push_back(r);
    };
    InitRandom(g_logic_seed);
    const int ops = 8 + static_cast<int>(pick(20));
    for (int k = 0; k < ops; ++k) {
        const uint32_t code = pick(14);
        uint32_t ret = 0;
        std::string op;
        W->set_frame(frame);
        if (code == 0) {
            op = "0";
            m->resetTimer();
        } else if (code == 1) {  // disable or enable
            const uint32_t on = pick(2);
            op = "1 " + num(on);
            if (on) o->setDisabled(DISABLED_HELD);
            else o->clearDisabled(DISABLED_HELD);
        } else if (code == 2) {  // capture
            team = pick(3);
            op = "2 " + num(team);
            o->setTeam(team_of(team));
        } else if (code == 3) {  // recolor
            const uint32_t who = pick(3);
            const uint32_t color = pick(3);
            op = "3 " + num(who) + " " + num(color);
            team_of(who)->getControllingPlayer()->*access_member(MColor()) = color;
        } else if (code == 4) {  // under construction
            const uint32_t on = chance(40);
            op = "4 " + num(on);
            if (on) o->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
            else o->clearStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
        } else {
            const uint32_t d1 = chance(45) ? 1 : chance(70) ? pick(12) : pick(40);
            frame += d1;
            W->set_frame(frame);
            op = "5 " + num(d1);
            ret = static_cast<uint32_t>(m->update());
        }
        v.ops.push_back(op);
        row(ret);
    }
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_pkt334_267_382_346_misc <out.json>\n");
        return 2;
    }
    oracle_wp51::World w;
    W = &w;
    TheTerrainLogic = NEW StepTerrain;
    TheTerrainLogic->init();
    ThePartitionManager->reset();
    ThePartitionManager->init();
    TheAI->pathfinder()->newMap();

    std::string lists;
    for (int i = 0; i < kFx; ++i) lists += std::string("FXList ") + kFxNames[i] + "\nEnd\n";
    for (int i = 0; i < kOcl; ++i) lists += std::string("ObjectCreationList ") + kOclNames[i] + "\nEnd\n";
    w.load_ini(lists);
    for (int i = 0; i < kFx; ++i) {
        g_fx_nugget[i].list = i;
        const_cast<FXList*>(TheFXListStore->findFXList(kFxNames[i]))->addFXNugget(&g_fx_nugget[i]);
    }
    for (int i = 0; i < kOcl; ++i) {
        g_ocl_nugget[i].list = i;
        const_cast<ObjectCreationList*>(TheObjectCreationListStore->findObjectCreationList(kOclNames[i]))
            ->addObjectCreationNugget(&g_ocl_nugget[i]);
    }
    const std::string templates =
        "PlayerTemplate M_America\n  Side = America\n  PlayableSide = Yes\nEnd\n"
        "PlayerTemplate M_China\n  Side = China\n  PlayableSide = Yes\nEnd\n";
    w.load_ini(templates);
    const std::string ini = fixture_ini();
    w.load_ini(ini);
    w.init_with_neutral_player("M_America", "M_China");

    // `FXList` drops effects at positions the local player cannot see.
    // Player 1 must not be local while revealing: there is no display to update.
    ThePartitionManager->revealMapForPlayerPermanently(1);
    ThePlayerList->setLocalPlayer(ThePlayerList->getNthPlayer(1));
    ThePartitionManager->update();

    FILE* out = std::fopen(argv[1], "w");
    if (!out) return 1;
    const int kPerKind = 400;
    const int kTotal = 4 * kPerKind;
    std::string escaped;
    for (char ch : ini) escaped += ch == '\n' ? std::string("\\n") : std::string(1, ch);
    std::fprintf(out, "{\"vector_count\":%d,\n\"ini\":\"%s\",\n\"vectors\":[", kTotal, escaped.c_str());
    for (int i = 0; i < kTotal; ++i) {
        const int kind = i % 4;
        const UnsignedInt frame = chance(10) ? 0x01000000u + pick(0x10000000u) : 1 + pick(5000);
        const uint32_t logic_seed = pick(1000000);
        TheGameLogic->setObjectIDCounter(static_cast<ObjectID>(1));
        w.set_frame(frame);
        g_logic_seed = logic_seed;
        g_spawned.clear();
        g_fx.str("");
        g_ocl.str("");
        Vec v;
        if (kind == 0) run_heli(v, frame);
        else if (kind == 1) run_proj(v, frame);
        else if (kind == 2) run_tensile(v, frame);
        else run_ocl(v, frame);

        std::fprintf(out, "%s\n{\"kind\":%d,\"frame\":%u,\"seed\":%u,\"setup\":\"%s\",\"ops\":[", i ? "," : "",
                     kind, frame, logic_seed, v.setup.c_str());
        for (size_t k = 0; k < v.ops.size(); ++k) std::fprintf(out, "%s\"%s\"", k ? "," : "", v.ops[k].c_str());
        std::fprintf(out, "],\"rows\":[");
        for (size_t k = 0; k < v.rows.size(); ++k) std::fprintf(out, "%s\"%s\"", k ? "," : "", v.rows[k].c_str());
        std::fprintf(out, "]}");

        for (auto it = g_spawned.rbegin(); it != g_spawned.rend(); ++it) TheGameLogic->destroyObject(*it);
        w.flush_destroy_list();
        ThePartitionManager->update();
        if (TheGameLogic->getFirstObject()) die("objects left behind");
    }
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    return 0;
}
