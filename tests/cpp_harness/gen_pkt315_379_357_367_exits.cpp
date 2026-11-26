// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for the four production exits: DefaultProductionExitUpdate
// (PKT-315), SupplyCenterProductionExitUpdate (PKT-379),
// QueueProductionExitUpdate (PKT-357) and SpawnPointProductionExitUpdate
// (PKT-367), run on the unmodified reference modules in a small real world with
// the real pathfinder on open ground.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt315_379_357_367_exits.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/PKT315_379_357_367_Exits.json
//
// The AI of an exited unit is a subclass that records the command instead of
// acting on it. The spawn point bones come from a stand-in draw module, because
// no models are loaded.
//
// Not covered: real bridges (the layer is only copied and passed to a stub height
// lookup); blocked or occupied rally cells, so adjustDestination never fails or
// searches on; water-only locomotors; a producer without a drawable; a stealth
// grant to a unit without a StealthUpdate (the reference dereferences null);
// destroyed units still waiting in the destroy list; loading a save.

#include "oracle_wp51_world.h"

#include "Common/DrawModule.h"
#include "Common/Module.h"
#include "Common/ModuleFactory.h"
#include "Common/Team.h"
#include "Common/ThingFactory.h"
#include "GameClient/Drawable.h"
#include "GameLogic/AI.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Module/StealthUpdate.h"
#include "GameLogic/Module/SupplyTruckAIUpdate.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Object.h"

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

// --- member access ----------------------------------------------------------

struct VelTag { using type = Coord3D PhysicsBehavior::*; };
template struct MemberAccess<VelTag, &PhysicsBehavior::m_vel>;
VelTag::type access_member(VelTag);

struct GrantTag { using type = UnsignedInt StealthUpdate::*; };
template struct MemberAccess<GrantTag, &StealthUpdate::m_framesGranted>;
GrantTag::type access_member(GrantTag);

struct DrawArrayTag { using type = Module** (Drawable::*)[NUM_DRAWABLE_MODULE_TYPES]; };
template struct MemberAccess<DrawArrayTag, &Drawable::m_modules>;
DrawArrayTag::type access_member(DrawArrayTag);

struct ExposedFactory : public ModuleFactory { using ModuleFactory::addModuleInternal; };

static void die(const char* what) {
    std::fprintf(stderr, "oracle: %s\n", what);
    std::exit(1);
}

// --- recording AI -----------------------------------------------------------

struct Command {
    int code = 0;  // 1 follow exit production path, 2 move to position
    int source = 0;
    ObjectID ignore = INVALID_ID;
    std::vector<Coord3D> path;
};
static Command g_command;
static int g_commands = 0;

static void record(const AICommandParms* parms) {
    ++g_commands;
    g_command = Command();
    g_command.source = parms->m_cmdSource;
    if (parms->m_cmd == AICMD_FOLLOW_EXITPRODUCTION_PATH) {
        g_command.code = 1;
        g_command.ignore = parms->m_obj ? parms->m_obj->getID() : INVALID_ID;
        g_command.path = parms->m_coords;
    } else if (parms->m_cmd == AICMD_MOVE_TO_POSITION) {
        g_command.code = 2;
        g_command.path.push_back(parms->m_pos);
    } else {
        die("unexpected AI command");
    }
}

class RecAI : public AIUpdateInterface {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(RecAI, "OracleExitRecAI", 64, 64)
public:
    RecAI(Thing* thing, const ModuleData* data) : AIUpdateInterface(thing, data) {}
    void aiDoCommand(const AICommandParms* parms) override { record(parms); }
};
RecAI::~RecAI() {}

class RecTruck : public SupplyTruckAIUpdate {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(RecTruck, "OracleExitRecTruck", 64, 64)
public:
    RecTruck(Thing* thing, const ModuleData* data) : SupplyTruckAIUpdate(thing, data) {}
    void aiDoCommand(const AICommandParms* parms) override { record(parms); }
};
RecTruck::~RecTruck() {}

static Module* make_rec_ai(Thing* t, const ModuleData* d) { return newInstance(RecAI)(t, d); }
static Module* make_rec_truck(Thing* t, const ModuleData* d) { return newInstance(RecTruck)(t, d); }

// --- bones ------------------------------------------------------------------

static std::vector<Matrix3D> g_bones;
static const char* const kBoneName = "SpawnPoint";

class OracleDraw : public DrawModule, public ObjectDrawInterface {
    MEMORY_POOL_GLUE_WITH_EXPLICIT_CREATE(OracleDraw, "OracleExitDraw", 64, 64)
public:
    OracleDraw(Thing* thing, const ModuleData* data) : DrawModule(thing, data) {}

    NameKeyType getModuleNameKey() const override { return NAMEKEY("OracleExitDraw"); }
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
        if (std::strcmp(prefix, kBoneName) != 0 || start != 1 || positions || !transforms)
            die("unexpected bone query");
        Int n = 0;
        for (; n < static_cast<Int>(g_bones.size()) && n < max; ++n) transforms[n] = g_bones[n];
        return n;
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

class OracleDrawData : public ModuleData {};

// --- terrain ----------------------------------------------------------------

static bool g_slope = false;

// The layer height differs from the ground height so the two lookups are told apart.
class SlopeTerrain : public oracle_wp51::OracleTerrainLogic {
public:
    Real getGroundHeight(Real x, Real y, Coord3D* normal = nullptr) const override {
        if (normal) { normal->x = 0.0f; normal->y = 0.0f; normal->z = 1.0f; }
        return g_slope ? x * 0.01f + y * 0.02f : 0.0f;
    }
    Real getLayerHeight(Real x, Real y, PathfindLayerEnum layer, Coord3D* normal = nullptr,
                        Bool = true) const override {
        return getGroundHeight(x, y, normal) + 2.0f * static_cast<Real>(static_cast<int>(layer) - 1);
    }
};

// --- random -----------------------------------------------------------------

static uint32_t g_state = 0x31537935u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}
static float real_in(float lo, float hi) {
    return lo + (hi - lo) * (static_cast<float>(pick(1000003)) / 1000003.0f);
}
static uint32_t bits(float f) {
    uint32_t u;
    std::memcpy(&u, &f, 4);
    return u;
}
static float real_of(uint32_t u) {
    float f;
    std::memcpy(&f, &u, 4);
    return f;
}

// --- fixture ----------------------------------------------------------------

static const int kUnits = 11;
struct UnitSpec {
    const char* kind_of;
    int radius;
    int ai;         // 0 none, 1 plain, 2 supply truck
    const char* loco;  // nullptr: none
    bool physics;
    int stealth;    // 0 none, 1 not innate, 2 innate
};
static const UnitSpec kUnit[kUnits] = {
    {"PRELOAD", 3, 0, nullptr, false, 0},
    {"PRELOAD", 7, 1, "P_Ground", true, 0},
    {"PRELOAD", 12, 1, "P_Air", true, 0},
    {"PRELOAD", 3, 1, "P_Both", true, 0},
    {"PRELOAD", 15, 1, nullptr, false, 0},
    {"PRELOAD", 30, 2, "P_Ground", true, 1},
    {"PRELOAD PRODUCED_AT_HELIPAD", 6, 1, "P_Both", true, 0},
    {"PRELOAD PRODUCED_AT_HELIPAD", 3, 1, "P_Air", true, 0},
    {"PRELOAD", 12, 1, "P_Ground", false, 2},
    {"PRELOAD", 7, 0, nullptr, true, 0},
    {"PRELOAD PROJECTILE", 10, 1, "P_Ground", false, 0},
};

static const int kKinds = 4;
static const int kVariants = 6;
static const char* const kModule[kKinds] = {
    "DefaultProductionExitUpdate", "SupplyCenterProductionExitUpdate",
    "QueueProductionExitUpdate", "SpawnPointProductionExitUpdate"};
static const char* const kCreate[kVariants] = {
    "X:0 Y:0 Z:0", "X:12.5 Y:-3.25 Z:0", "X:-20 Y:7 Z:15.5",
    "X:3 Y:4 Z:-2", "X:0.1 Y:0.2 Z:0", "X:33 Y:0 Z:0"};
static const char* const kNatural[kVariants] = {
    "X:0 Y:0 Z:0", "X:40 Y:0 Z:0", "X:-13.5 Y:22.25 Z:6",
    "X:0.3 Y:0.4 Z:0", "X:1 Y:1 Z:1", "X:17.7 Y:-31.9 Z:0"};

static std::string producer_fields(int kind, int v) {
    std::string s;
    if (kind != 3) {
        s += std::string("    UnitCreatePoint = ") + kCreate[v] + "\n";
        s += std::string("    NaturalRallyPoint = ") + kNatural[(v + kind) % kVariants] + "\n";
    }
    if (kind == 0 && v % 2) s += "    UseSpawnRallyPoint = Yes\n";
    if (kind == 1) {
        static const char* const grant[kVariants] = {nullptr, "0", "1", "1000", "5000", "333"};
        if (grant[v]) s += std::string("    GrantTemporaryStealth = ") + grant[v] + "\n";
    }
    if (kind == 2) {
        static const char* const delay[kVariants] = {nullptr, "0", "1", "100", "300", "1000"};
        static const int burst[kVariants] = {0, 2, 0, 1, 0, 3};
        if (delay[v]) s += std::string("    ExitDelay = ") + delay[v] + "\n";
        if (burst[v]) s += "    InitialBurst = " + std::to_string(burst[v]) + "\n";
        if (v >= 3) s += "    AllowAirborneCreation = Yes\n";
    }
    if (kind == 3) s += std::string("    SpawnPointBoneName = ") + kBoneName + "\n";
    return s;
}

static std::string fixture_ini() {
    std::ostringstream s;
    for (int i = 0; i < kUnits; ++i) {
        const UnitSpec& u = kUnit[i];
        s << "Object P_Unit" << i << "\n  KindOf = " << u.kind_of
          << "\n  Geometry = CYLINDER\n  GeometryMajorRadius = " << u.radius
          << "\n  GeometryHeight = 4\n  GeometryIsSmall = No\n"
          << "  Body = InactiveBody ModuleTag_Body\n  End\n";
        if (u.physics)
            s << "  Behavior = PhysicsBehavior ModuleTag_Physics\n    Mass = " << (i + 2)
              << ".5\n    CenterOfMassOffset = " << (i % 2 ? "-" : "") << (i + 1) << ".25\n  End\n";
        if (u.ai) {
            s << "  Behavior = " << (u.ai == 2 ? "OracleExitTruck" : "OracleExitAI")
              << " ModuleTag_AI\n  End\n";
            if (u.loco) s << "  Locomotor = SET_NORMAL " << u.loco << "\n";
        }
        if (u.stealth)
            s << "  Behavior = StealthUpdate ModuleTag_Stealth\n    InnateStealth = "
              << (u.stealth == 2 ? "Yes" : "No") << "\n  End\n";
        s << "End\n";
    }
    for (int k = 0; k < kKinds; ++k) {
        for (int v = 0; v < kVariants; ++v) {
            for (int p = 0; p < 2; ++p) {
                s << "Object P_Exit" << k << "_" << v << "_" << p << "\n  KindOf = PRELOAD\n"
                  << "  Body = InactiveBody ModuleTag_Body\n  End\n";
                if (p)
                    s << "  Behavior = PhysicsBehavior ModuleTag_Physics\n    Mass = 40\n  End\n";
                s << "  Behavior = " << kModule[k] << " ModuleTag_Exit\n"
                  << producer_fields(k, v) << "  End\nEnd\n";
            }
        }
    }
    return s.str();
}

static const char* const kLocomotors =
    "Locomotor P_Ground\n  Surfaces = GROUND\n  Speed = 30\n  TurnRate = 180\n"
    "  Acceleration = 30\n  Braking = 30\nEnd\n"
    "Locomotor P_Air\n  Surfaces = AIR\n  Speed = 60\n  TurnRate = 180\n  Acceleration = 60\n"
    "  Braking = 60\n  MinSpeed = 0\n  Lift = 100\nEnd\n"
    "Locomotor P_Both\n  Surfaces = GROUND AIR\n  Speed = 60\n  TurnRate = 180\n  Acceleration = 60\n"
    "  Braking = 60\n  MinSpeed = 0\n  Lift = 100\nEnd\n";

// --- observation ------------------------------------------------------------

static void put(std::ostringstream& out, const Coord3D& c) {
    out << ' ' << bits(c.x) << ' ' << bits(c.y) << ' ' << bits(c.z);
}

static UpdateModule* exit_module(Object* o) {
    for (BehaviorModule** m = o->getBehaviorModules(); *m; ++m) {
        if ((*m)->getUpdateExitInterface()) return static_cast<UpdateModule*>(*m);
    }
    die("no exit module");
    return nullptr;
}

static std::string row(Object* producer, UpdateModule* m, int door) {
    ExitInterface* e = producer->getObjectExitInterface();
    std::ostringstream out;
    out << oracle_wp51::World::snapshot_save_hex(m) << ' ' << oracle_wp51::World::snapshot_crc(m)
        << ' ' << m->friend_getNextCallFrame() << ' ' << door;
    Coord3D c;
    c.x = c.y = c.z = -1.0f;
    out << ' ' << static_cast<int>(e->getExitPosition(c));
    put(out, c);
    for (int offset = 1; offset >= 0; --offset) {
        c.x = c.y = c.z = -1.0f;
        out << ' ' << static_cast<int>(e->getNaturalRallyPoint(c, offset != 0));
        put(out, c);
    }
    const Coord3D* r = e->getRallyPoint();
    out << ' ' << (r ? 1 : 0);
    if (r) put(out, *r);
    out << ' ' << static_cast<int>(e->useSpawnRallyPoint()) << static_cast<int>(e->isExitBusy());
    return out.str();
}

static std::string unit_row(Object* u) {
    std::ostringstream out;
    out << " U " << static_cast<int>(u->getID());
    put(out, *u->getPosition());
    out << ' ' << bits(u->getOrientation()) << ' ' << static_cast<int>(u->getLayer()) << ' '
        << static_cast<int>(u->isDisabledByType(DISABLED_HELD) != FALSE) << ' '
        << static_cast<int>(u->testStatus(OBJECT_STATUS_STEALTHED) != FALSE)
        << static_cast<int>(u->testStatus(OBJECT_STATUS_CAN_STEALTH) != FALSE);
    StealthUpdate* s = u->getStealth();
    out << ' ' << (s ? s->*access_member(GrantTag()) : 0u);
    AIUpdateInterface* ai = u->getAIUpdateInterface();
    SupplyTruckAIInterface* truck = ai ? ai->getSupplyTruckAIInterface() : nullptr;
    out << ' ' << (truck ? static_cast<int>(truck->isForcedIntoWantingState()) : 0);
    PhysicsBehavior* p = u->getPhysics();
    out << ' ' << (p ? oracle_wp51::World::snapshot_save_hex(p) : std::string("-"));
    if (g_commands > 1) die("more than one AI command in an exit");
    out << ' ' << g_command.code << ' ' << g_command.source << ' '
        << static_cast<int>(g_command.ignore) << ' ' << g_command.path.size();
    for (const Coord3D& c : g_command.path) put(out, c);
    return out.str();
}

// --- vectors ----------------------------------------------------------------

static const float kAngles[] = {0.0f, 1.5707964f, 3.1415927f, -1.5707964f, 0.5f, -2.75f, 6.0f, 0.001f};

struct Op { uint32_t code, a, b, c, d, e; };

static Coord3D at(float x, float y, float z) {
    Coord3D c;
    c.x = x;
    c.y = y;
    c.z = z;
    return c;
}

// Producer positions: mostly inland, sometimes at the map edge so goal cells clamp.
static Coord3D random_site() {
    float x = real_in(100.0f, 900.0f), y = real_in(100.0f, 900.0f);
    if (pick(8) == 0) x = real_in(0.0f, 25.0f);
    if (pick(8) == 0) y = real_in(980.0f, 999.0f);
    if (pick(3) == 0) { x = std::floor(x / 10.0f) * 10.0f; y = std::floor(y / 10.0f) * 10.0f; }
    float z = 0.0f;
    const uint32_t zk = pick(4);
    if (zk == 1) z = real_in(-8.0f, 40.0f);
    if (zk == 2 && g_slope) z = x * 0.01f + y * 0.02f;
    return at(x, y, z);
}

static void unit_flags(Object* u, uint32_t flags) {
    PhysicsBehavior* p = u->getPhysics();
    if ((flags & 4) && p) p->setAllowToFall(true);
    StealthUpdate* s = u->getStealth();
    if ((flags & 8) && s) s->receiveGrant(TRUE, 90);
    if (flags & 16) u->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_CAN_STEALTH));
    if (flags & 1) u->setDisabled(DISABLED_HELD);
    if (flags & 2) u->setDisabled(DISABLED_UNMANNED);
}

static uint32_t random_flags() {
    uint32_t f = 0;
    if (pick(6) == 0) f |= 1;
    if (pick(4) == 0) f |= 2;
    if (pick(3) == 0) f |= 4;
    if (pick(3) == 0) f |= 8;
    if (pick(4) == 0) f |= 16;
    return f;
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "usage: gen_pkt315_379_357_367_exits <out.json>\n");
        return 2;
    }
    oracle_wp51::World w;
    TheTerrainLogic = NEW SlopeTerrain;
    TheTerrainLogic->init();
    (TheModuleFactory->*(&ExposedFactory::addModuleInternal))(
        make_rec_ai, AIUpdateInterface::friend_newModuleData, MODULETYPE_BEHAVIOR,
        "OracleExitAI", AIUpdateInterface::getInterfaceMask());
    (TheModuleFactory->*(&ExposedFactory::addModuleInternal))(
        make_rec_truck, SupplyTruckAIUpdate::friend_newModuleData, MODULETYPE_BEHAVIOR,
        "OracleExitTruck", SupplyTruckAIUpdate::getInterfaceMask());
    w.load_ini(kLocomotors);
    w.load_ini(fixture_ini());
    w.init_with_neutral_player();

    static OracleDrawData draw_data;
    FILE* out = std::fopen(argv[1], "w");
    if (!out) return 1;
    const int kTotal = 1200;
    std::fprintf(out, "{\"vector_count\":%d,\"vectors\":[", kTotal);
    for (int i = 0; i < kTotal; ++i) {
        const int kind = i % kKinds;
        const int variant = static_cast<int>(pick(kVariants));
        const int phys = static_cast<int>(pick(3) != 0);
        g_slope = pick(3) == 0;
        const UnsignedInt first_frame = 1 + pick(5000);
        UnsignedInt frame = first_frame;
        w.set_frame(frame);

        const std::string name = "P_Exit" + std::to_string(kind) + "_" + std::to_string(variant) +
                                 "_" + std::to_string(phys);
        const Coord3D site = random_site();
        const float angle = pick(2) ? kAngles[pick(8)] : real_in(-3.14f, 3.14f);
        Object* producer = w.spawn_object(name.c_str(), pick(2) ? w.team_a : w.team_b, site, angle);
        if (!producer) die("missing producer template");
        const int layer = pick(4) == 0 ? 2 + static_cast<int>(pick(3)) : 1;
        producer->setLayer(static_cast<PathfindLayerEnum>(layer));
        UpdateModule* m = exit_module(producer);

        // Spawn point bones: mostly a few, sometimes none or more than fit.
        g_bones.clear();
        std::ostringstream bone_text;
        Module* fake[2] = {nullptr, nullptr};
        Module** saved = nullptr;
        Drawable* d = producer->getDrawable();
        if (kind == 3) {
            if (!d) die("producer without a drawable");
            uint32_t n = pick(4) == 0 ? pick(14) : 1 + pick(3);
            for (uint32_t b = 0; b < n; ++b) {
                Matrix3D mtx(true);
                mtx.Translate(real_in(-30.0f, 30.0f), real_in(-30.0f, 30.0f), real_in(-5.0f, 12.0f));
                mtx.Rotate_Z(real_in(-3.14f, 3.14f));
                if (pick(3) == 0) mtx.Rotate_X(real_in(-1.0f, 1.0f));
                if (pick(3) == 0) mtx.Rotate_Y(real_in(-1.0f, 1.0f));
                g_bones.push_back(mtx);
                for (int r = 0; r < 3; ++r)
                    for (int c = 0; c < 4; ++c)
                        bone_text << (bone_text.tellp() > 0 ? " " : "") << bits(mtx[r][c]);
            }
            // A quarter keep the headless drawable, which reports no bones.
            if (pick(8) != 0) {
                fake[0] = newInstance(OracleDraw)(d, &draw_data);
                saved = (d->*access_member(DrawArrayTag()))[0];
                (d->*access_member(DrawArrayTag()))[0] = fake;
            } else {
                bone_text.str("");
                g_bones.clear();
            }
        }

        std::fprintf(out,
                     "%s\n{\"kind\":%d,\"variant\":%d,\"phys\":%d,\"slope\":%d,\"frame\":%u,"
                     "\"team\":%d,\"id\":%u,\"x\":%u,\"y\":%u,\"z\":%u,\"angle\":%u,\"layer\":%d,"
                     "\"bones\":\"%s\",\"spawn\":\"%s\"",
                     i ? "," : "", kind, variant, phys, g_slope ? 1 : 0, first_frame,
                     producer->getTeam() == w.team_a ? 0 : 1,
                     static_cast<unsigned>(producer->getID()), bits(site.x), bits(site.y),
                     bits(site.z), bits(angle), layer, bone_text.str().c_str(),
                     row(producer, m, 0).c_str());

        std::vector<Object*> spawned;
        std::vector<Op> ops;
        std::vector<std::string> rows;
        const int n_ops = 3 + static_cast<int>(pick(kind == 3 ? 22 : 12));
        bool stealthed = false;
        for (int k = 0; k < n_ops; ++k) {
            Op op = {0, 0, 0, 0, 0, 0};
            const uint32_t roll = pick(20);
            if (kind == 1 && k == 0 && pick(3) != 0) op.code = 6;
            else if (roll < 8) op.code = 0;
            else if (roll < 10) op.code = 1;
            else if (roll < 12) op.code = kind == 2 ? 2 : 4;
            else if (roll < 14) op.code = 3;
            else if (roll < 15) op.code = 4;
            else if (roll < 17) op.code = kind == 2 ? 5 : (kind == 3 ? 7 : 0);
            else if (roll < 18) op.code = 6;
            else op.code = kind == 2 ? 2 : (kind == 3 ? 7 : 0);

            if (op.code == 0 || op.code == 5) {
                op.a = pick(kUnits);
                if (kind == 1 && pick(3) == 0) op.a = pick(2) ? 5 : 8;
                // The reference dereferences a missing StealthUpdate.
                if (kind == 1 && stealthed && variant >= 2 && !kUnit[op.a].stealth)
                    op.a = pick(2) ? 5 : 8;
                op.b = random_flags();
                if (op.code == 5) {
                    op.c = pick(3) != 0;  // with a bud host
                    op.d = 1 + pick(4);   // its layer
                    op.e = bits(random_site().x);  // host x; the rest follows from it
                }
            } else if (op.code == 1) {
                // Rally points stay inland.
                float x = real_in(40.0f, 960.0f), y = real_in(40.0f, 960.0f);
                if (pick(4) == 0) { x = std::floor(x / 10.0f) * 10.0f; y = std::floor(y / 10.0f) * 10.0f; }
                op.a = bits(x);
                op.b = bits(y);
                op.c = bits(real_in(-3.0f, 20.0f));
            } else if (op.code == 2) {
                op.a = 1 + pick(pick(2) ? 4 : 40);
                op.b = pick(4) == 0;
            } else if (op.code == 4) {
                const Coord3D c = random_site();
                op.a = bits(c.x);
                op.b = bits(c.y);
                op.c = bits(c.z);
                op.d = bits(pick(2) ? kAngles[pick(8)] : real_in(-3.14f, 3.14f));
            } else if (op.code == 6) {
                op.a = pick(4) != 0;
                if (pick(3) != 0) {
                    op.b = bits(real_in(-9.0f, 9.0f));
                    op.c = bits(real_in(-9.0f, 9.0f));
                    op.d = bits(real_in(-9.0f, 9.0f));
                }
            } else if (op.code == 7) {
                if (spawned.empty()) op.code = 3;
                else op.a = pick(static_cast<uint32_t>(spawned.size()));
            }

            w.set_frame(frame);
            ExitInterface* e = producer->getObjectExitInterface();
            int door = 99;
            std::string extra;
            if (op.code == 0 || op.code == 5) {
                const std::string unit = "P_Unit" + std::to_string(op.a);
                Object* u = w.spawn_object(unit.c_str(), w.team_a, at(500.0f, 500.0f, 0.0f), 1.0f);
                if (!u) die("missing unit template");
                unit_flags(u, op.b);
                g_commands = 0;
                g_command = Command();
                Object* host = nullptr;
                if (op.code == 0) {
                    e->exitObjectViaDoor(u, DOOR_1);
                } else {
                    if (op.c) {
                        const float hx = real_of(op.e);
                        host = w.spawn_object("P_Unit0", w.team_a, at(hx, 1000.0f - hx, hx * 0.1f),
                                              hx * 0.003f);
                        host->setLayer(static_cast<PathfindLayerEnum>(op.d));
                    }
                    e->exitObjectByBudding(u, host);
                }
                extra = unit_row(u);
                if (host) TheGameLogic->destroyObject(host);
                if (kind == 3) spawned.push_back(u);
                else TheGameLogic->destroyObject(u);
                w.flush_destroy_list();
            } else if (op.code == 1) {
                const Coord3D r = at(real_of(op.a), real_of(op.b), real_of(op.c));
                e->setRallyPoint(&r);
            } else if (op.code == 2) {
                if (op.b) {
                    door = 100 + static_cast<int>(m->update());
                } else {
                    for (uint32_t t = 0; t < op.a; ++t) {
                        w.set_frame(++frame);
                        if (m->friend_getNextCallFrame() > frame) continue;
                        UpdateSleepTime sleep_len = m->update();
                        if (sleep_len < 1) sleep_len = UPDATE_SLEEP_NONE;
                        UnsignedInt next = frame + static_cast<UnsignedInt>(sleep_len);
                        if (next > UPDATE_SLEEP_FOREVER) next = UPDATE_SLEEP_FOREVER;
                        m->friend_setNextCallFrame(next);
                    }
                }
            } else if (op.code == 3) {
                door = e->reserveDoorForExit(producer->getTemplate(), nullptr);
                e->unreserveDoorForExit(static_cast<ExitDoorType>(door));
            } else if (op.code == 4) {
                const Coord3D c = at(real_of(op.a), real_of(op.b), real_of(op.c));
                producer->setPosition(&c);
                producer->setOrientation(real_of(op.d));
            } else if (op.code == 6) {
                stealthed = op.a != 0;
                if (stealthed) producer->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_STEALTHED));
                else producer->clearStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_STEALTHED));
                if (PhysicsBehavior* p = producer->getPhysics())
                    p->*access_member(VelTag()) = at(real_of(op.b), real_of(op.c), real_of(op.d));
            } else {
                Object* victim = spawned[op.a];
                spawned.erase(spawned.begin() + op.a);
                TheGameLogic->destroyObject(victim);
                w.flush_destroy_list();
            }
            ops.push_back(op);
            rows.push_back(row(producer, m, door) + extra);
        }

        std::fprintf(out, ",\"ops\":\"");
        for (size_t k = 0; k < ops.size(); ++k)
            std::fprintf(out, "%s%u %u %u %u %u %u", k ? ";" : "", ops[k].code, ops[k].a, ops[k].b,
                         ops[k].c, ops[k].d, ops[k].e);
        std::fprintf(out, "\",\"rows\":\"");
        for (size_t r = 0; r < rows.size(); ++r)
            std::fprintf(out, "%s%s", r ? "|" : "", rows[r].c_str());
        std::fprintf(out, "\"}");

        if (saved) {
            (d->*access_member(DrawArrayTag()))[0] = saved;
            deleteInstance(static_cast<OracleDraw*>(fake[0]));
        }
        for (Object* u : spawned) TheGameLogic->destroyObject(u);
        TheGameLogic->destroyObject(producer);
        w.flush_destroy_list();
    }
    std::fprintf(out, "\n]}\n");
    std::fclose(out);
    return 0;
}
