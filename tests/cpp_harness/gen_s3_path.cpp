// Golden vectors for slice S3: Path and PathNode, and the movement-surface half
// of Pathfinder (validity and destination checks, line walks, passability, view
// lines, quick zone tests, tall buildings, aircraft paths), all in the
// unmodified reference library.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_s3_path.cpp -o gen_s3
//   ./gen_s3 docs/golden_vectors/S3_Path.json
//
// Only the terrain is synthetic. The map is built by the real newMap; unit
// occupancy is written with the real PathfindCell setters (the Pathfinder
// functions that do it belong to slice S4). Each row is an input to replay or a
// question with the reference's answer after " = ". Partition searches are
// recorded as the reference makes them and replayed from that tape.
//
// Not covered: bridges (their layers are covered by slice S2), containers,
// slaves and weapons in view-line tests, getBuildingToNotPathAround.
//
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a PartitionManager.cpp.o __ZN16PartitionManager16getClosestObjectEPK7Coord3Df23DistanceCalculationTypePP15PartitionFilterPfPS0_

#include "oracle_wp51_world.h"

#include "GameLogic/AIPathfind.h"
#include "GameLogic/Locomotor.h"
#include "GameLogic/LocomotorSet.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Pathfinder/PathfindCell.h"
#include "GameLogic/Pathfinder/PathfindLayer.h"
#include "GameLogic/Pathfinder/PathNode.h"
#include "GameLogic/Pathfinder/Path.h"
#include "GameLogic/TerrainLogic.h"

#include <cmath>
#include <cstdio>
#include <cstring>
#include <string>
#include <vector>

// `TCheckMovementInfo` is defined in AIPathfind.cpp; this is the same layout.
struct TCheckMovementInfo {
    ICoord2D cell;
    PathfindLayerEnum layer;
    Int radius;
    Bool centerInCell;
    Bool considerTransient;
    LocomotorSurfaceTypeMask acceptableSurfaces;
    Int allyFixedCount;
    Bool enemyFixed;
    Bool allyMoving;
    Bool allyGoal;
};

#define S3_TAG(Tag, Type, Class, Member)                   \
    struct Tag { using type = Type Class::*; };            \
    template struct MemberAccess<Tag, &Class::Member>;     \
    inline Type Class::* access_member(Tag);
#define S3_FN(Tag, Signature, Member)                      \
    struct Tag { using type = Signature; };                \
    template struct MemberAccess<Tag, &Member>;            \
    inline Tag::type access_member(Tag);

S3_TAG(LsSurfaces, LocomotorSurfaceTypeMask, LocomotorSet, m_validLocomotorSurfaces)
S3_TAG(PfIgnore, ObjectID, Pathfinder, m_ignoreObstacleID)
// `Object::setLayer` rewrites the occupancy, so the tests poke the member.
S3_TAG(ObjLayer, PathfindLayerEnum, Object, m_layer)
S3_FN(PfCheckDest, Bool (Pathfinder::*)(const Object*, Int, Int, PathfindLayerEnum, Int, Bool),
      Pathfinder::checkDestination)
S3_FN(PfCheckMove, Bool (Pathfinder::*)(const Object*, TCheckMovementInfo&), Pathfinder::checkForMovement)
S3_FN(PfClear, Int (Pathfinder::*)(Bool, Int, Int, PathfindLayerEnum, Int), Pathfinder::clearCellForDiameter)
typedef Int (*S3Proc)(Pathfinder*, PathfindCell*, PathfindCell*, Int, Int, void*);
S3_FN(PfIterate, Int (Pathfinder::*)(const ICoord2D&, const ICoord2D&, PathfindLayerEnum, S3Proc, void*),
      Pathfinder::iterateCellsAlongLine)
S3_FN(PfSegment, Bool (Pathfinder::*)(const PathNode*, PathNode*, ObjectID, Coord3D*, Coord3D*, Coord3D*),
      Pathfinder::segmentIntersectsTallBuilding)
S3_FN(PfCircle, Bool (Pathfinder::*)(const Coord3D*, const Coord3D*, Real, ObjectID, Coord3D*),
      Pathfinder::circleClipsTallBuilding)

// ---- text -------------------------------------------------------------------

static std::vector<std::string> g_rows;

static std::string f32(float v) {
    uint32_t bits;
    std::memcpy(&bits, &v, 4);
    char buf[16];
    std::snprintf(buf, sizeof(buf), "%08x", bits);
    return buf;
}
static std::string c3(const Coord3D& c) { return f32(c.x) + " " + f32(c.y) + " " + f32(c.z); }
static std::string num(long long v) { return std::to_string(v); }
static void row(const std::string& r) { g_rows.push_back(r); }

static uint32_t g_state = 0x53335041u;
static uint32_t rnd() {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state;
}
static int below(int n) { return n <= 0 ? 0 : static_cast<int>(rnd() % static_cast<uint32_t>(n)); }
static bool chance(int percent) { return below(100) < percent; }
static float between(float lo, float hi) {
    return lo + (hi - lo) * (static_cast<float>(rnd() % 100000u) / 100000.0f);
}
static float coord(float lo, float hi) {
    const float v = between(lo, hi);
    const int pick = below(10);
    if (pick == 0) return std::floor(v / 10.0f) * 10.0f;
    if (pick == 1) return std::floor(v / 5.0f) * 5.0f;
    if (pick == 2) return std::floor(v / 10.0f) * 10.0f - 0.5f;
    return v;
}

// ---- terrain ------------------------------------------------------------------

class S3Terrain : public oracle_wp51::OracleTerrainLogic {
public:
    int w = 1, h = 1;
    std::string cliff, water;
    size_t at(int i, int j) const {
        i = i < 0 ? 0 : (i > w + 1 ? w + 1 : i);
        j = j < 0 ? 0 : (j > h + 1 ? h + 1 : j);
        return static_cast<size_t>(j * (w + 2) + i);
    }
    static int cell(float v) { return static_cast<int>(std::floor(v / 10.0f)); }
    Real getGroundHeight(Real, Real, Coord3D* normal = nullptr) const override {
        if (normal) { normal->x = 0.0f; normal->y = 0.0f; normal->z = 1.0f; }
        return 0.0f;
    }
    Bool isCliffCell(Real x, Real y) const override { return cliff[at(cell(x), cell(y))] == '1'; }
    Bool isUnderwater(Real x, Real y, Real* waterZ = nullptr, Real* terrainZ = nullptr) override {
        if (waterZ) *waterZ = 0.0f;
        if (terrainZ) *terrainZ = 0.0f;
        return water[at(cell(x), cell(y))] == '1';
    }
};

static S3Terrain* g_terrain = nullptr;
static oracle_wp51::World* g_world = nullptr;
static Pathfinder* pf() { return TheAI->pathfinder(); }

static void blobs(std::string& table, int count) {
    S3Terrain& t = *g_terrain;
    for (int n = 0; n < count; ++n) {
        const int x = below(t.w + 2), y = below(t.h + 2), rx = 1 + below(5), ry = 1 + below(5);
        for (int j = y; j < y + ry; ++j)
            for (int i = x; i < x + rx; ++i) table[t.at(i, j)] = '1';
    }
}

// ---- templates and objects ------------------------------------------------------

enum { F_STRUCTURE = 64, F_SEETHROUGH = 128, F_MOBILE = 256, F_WALKWALL = 32, F_DEFWALL = 8 };

static std::string g_ini;
static std::vector<std::string> g_blocks, g_units;
static const char* kSurfaces[] = {"GROUND", "WATER", "CLIFF", "RUBBLE", "AIR", "GROUND WATER",
                                  "GROUND CLIFF", "GROUND RUBBLE", "GROUND CLIFF RUBBLE WATER"};
static const int kMasks[] = {1, 2, 4, 16, 8, 3, 5, 17, 23};

static void object_ini(const std::string& name, const std::string& kindof, const char* geom,
                       float major, float minor, const std::string& extra) {
    char radii[96];
    std::snprintf(radii, sizeof(radii), "  GeometryMajorRadius = %.4f\n  GeometryMinorRadius = %.4f\n",
                  major, minor);
    g_ini += "Object " + name + "\n  KindOf = " + kindof + "\n  Geometry = " + geom + "\n" + radii +
             "  GeometryHeight = 12.0\n  GeometryIsSmall = No\n" + extra +
             "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100.0\n    InitialHealth = 100.0\n  End\nEnd\n";
}

static void make_templates() {
    for (int i = 0; i < 9; ++i) {
        g_ini += "Locomotor S3Loco" + num(i) + "\n  Surfaces = " + kSurfaces[i] +
                 "\n  Appearance = " + (i == 4 ? "WINGS" : "TWO_LEGS") + "\n  Speed = 30\nEnd\n";
    }
    const std::string ai = "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n";
    static const float radii[] = {2.0f, 5.0f, 7.0f, 9.0f, 10.0f, 12.0f, 15.0f, 19.0f, 22.0f, 40.0f};
    for (int i = 0; i < 20; ++i) {
        const std::string name = "S3Unit" + num(i);
        const int crusher = i % 5 == 4 ? 3 : i % 3 == 2 ? 1 : 0;
        const int crushable = i % 4;
        std::string kinds = i % 3 == 0 ? "INFANTRY" : "VEHICLE";
        if (i % 2 == 0) kinds += " ATTACK_NEEDS_LINE_OF_SIGHT";
        std::string extra = "  CrusherLevel = " + num(crusher) + "\n  CrushableLevel = " + num(crushable) + "\n";
        if (i % 7 != 6) extra += ai + "  Locomotor = SET_NORMAL S3Loco0\n";
        object_ini(name, kinds, "CYLINDER", radii[i % 10], radii[i % 10], extra);
        g_units.push_back(name);
    }
    object_ini("S3Plane", "VEHICLE", "CYLINDER", 8, 8,
               "  Behavior = JetAIUpdate ModuleTag_AI\n  End\n  Locomotor = SET_NORMAL S3Loco4\n");
    object_ini("S3Chopper", "VEHICLE", "CYLINDER", 8, 8,
               "  Behavior = ChinookAIUpdate ModuleTag_AI\n  End\n  Locomotor = SET_NORMAL S3Loco4\n");
    static const char* geoms[] = {"BOX", "CYLINDER", "SPHERE"};
    for (int i = 0; i < 18; ++i) {
        std::string kinds = "STRUCTURE IMMOBILE";
        if (i % 6 == 1) kinds += " CAN_SEE_THROUGH_STRUCTURE";
        if (i % 6 == 2) kinds += " AIRCRAFT_PATH_AROUND";
        if (i % 6 == 3) kinds += " ATTACK_NEEDS_LINE_OF_SIGHT";
        if (i % 6 == 4) kinds += " AIRCRAFT_PATH_AROUND";
        const std::string name = "S3Block" + num(i);
        object_ini(name, kinds, geoms[i % 3], between(4.0f, 34.0f), between(4.0f, 34.0f), "");
        g_blocks.push_back(name);
    }
    for (int i = 0; i < 4; ++i) {
        const std::string name = "S3Fence" + num(i);
        char extra[64];
        std::snprintf(extra, sizeof(extra), "  FenceWidth = %.4f\n", between(15.0f, 70.0f));
        object_ini(name, "STRUCTURE IMMOBILE", "BOX", 20, 3, extra);
        g_blocks.push_back(name);
    }
    object_ini("S3Wall", "STRUCTURE IMMOBILE WALK_ON_TOP_OF_WALL", "BOX", 30, 18, "");
}

static int flags_of(Object* o) {
    int f = 0;
    if (o->isKindOf(KINDOF_DEFENSIVE_WALL)) f |= F_DEFWALL;
    if (o->isKindOf(KINDOF_WALK_ON_TOP_OF_WALL)) f |= F_WALKWALL;
    if (o->isKindOf(KINDOF_STRUCTURE)) f |= F_STRUCTURE;
    if (o->isKindOf(KINDOF_CAN_SEE_THROUGH_STRUCTURE)) f |= F_SEETHROUGH;
    if (o->isMobile()) f |= F_MOBILE;
    return f;
}

static void object_row(Object* o) {
    const GeometryInfo& g = o->getGeometryInfo();
    row("obj " + num(o->getID()) + " " + c3(*o->getPosition()) + " " + f32(o->getOrientation()) + " " +
        num(g.getGeomType()) + " " + num(g.getIsSmall()) + " " + f32(g.getMaxHeightAbovePosition()) +
        " " + f32(g.getMajorRadius()) + " " + f32(g.getMinorRadius()) + " " +
        f32(o->getTemplate()->getFenceWidth()) + " " + f32(o->getTemplate()->getFenceXOffset()) +
        " " + num(flags_of(o)) + " " + f32(o->getHeightAboveTerrain()));
}

static Object* spawn(const std::string& name, Team* team, float x, float y, float z, float a) {
    Coord3D p;
    p.x = x; p.y = y; p.z = z;
    Object* o = g_world->spawn_object(name.c_str(), team, p, a);
    if (!o) { std::fprintf(stderr, "no object %s\n", name.c_str()); std::exit(1); }
    return o;
}

// ---- partition tape -------------------------------------------------------------

static bool g_record = false;
extern "C" Object* real_closest(PartitionManager*, const Coord3D*, Real, DistanceCalculationType,
                                PartitionFilter**, Real*, Coord3D*)
    __asm__("_oracle_real__ZN16PartitionManager16getClosestObjectEPK7Coord3Df23DistanceCalculationTypePP15PartitionFilterPfPS0_");
Object* PartitionManager::getClosestObject(const Coord3D* pos, Real maxDist, DistanceCalculationType dc,
                                           PartitionFilter** filters, Real* closestDist, Coord3D* closestDistVec) {
    Object* o = real_closest(this, pos, maxDist, dc, filters, closestDist, closestDistVec);
    if (g_record) row("closest " + c3(*pos) + " " + f32(maxDist) + " " + num(o ? o->getID() : 0));
    return o;
}

// ---- scene ----------------------------------------------------------------------

struct Scene {
    int w = 0, h = 0;
    float maxx = 0, maxy = 0;
    std::vector<Object*> blocks, units, walls, tall, everything;
};

static Object* pick(const std::vector<Object*>& v) { return v[below(static_cast<int>(v.size()))]; }

static Coord3D point(const Scene& s, bool off_map = true) {
    Coord3D p;
    const float out = off_map && chance(8) ? 25.0f : 0.0f;
    p.x = coord(-out, s.maxx + out);
    p.y = coord(-out, s.maxy + out);
    p.z = chance(70) ? 0.0f : between(0.0f, 20.0f);
    return p;
}

// A point near another, so that lines are short enough to succeed sometimes.
static Coord3D near_point(const Scene& s, const Coord3D& from) {
    if (chance(35)) return point(s);
    Coord3D p = from;
    const float reach = chance(50) ? 35.0f : 110.0f;
    p.x += between(-reach, reach);
    p.y += between(-reach, reach);
    if (chance(15)) p.x = from.x;
    if (chance(15)) p.y = from.y;
    return p;
}

static PathfindLayerEnum some_layer(const Scene& s) {
    return !s.walls.empty() && chance(35) ? LAYER_WALL : LAYER_GROUND;
}

// A point on the layer: on the wall, near one of its pieces.
static Coord3D point_on(const Scene& s, PathfindLayerEnum layer) {
    if (layer != LAYER_WALL) return point(s);
    Coord3D p = *pick(s.walls)->getPosition();
    p.x += between(-35.0f, 35.0f);
    p.y += between(-22.0f, 22.0f);
    return p;
}

static ICoord2D cell_near(const Scene& s, PathfindLayerEnum layer) {
    const Coord3D p = point_on(s, layer);
    ICoord2D c;
    c.x = REAL_TO_INT_FLOOR(p.x / 10.0f);
    c.y = REAL_TO_INT_FLOOR(p.y / 10.0f);
    return c;
}

static Object* some_agent(const Scene& s) { return chance(6) ? nullptr : pick(s.units); }
static int id_of(const Object* o) { return o ? static_cast<int>(o->getID()) : 0; }

static void unit_rows(const Scene& s) {
    for (Object* o : s.units) {
        AIUpdateInterface* ai = o->getAIUpdateInterface();
        row("unit " + num(o->getID()) + " " + num(ai != nullptr) + " " +
            num(ai ? ai->getIgnoredObstacleID() : 0) + " " +
            num(ai ? ai->isAircraftThatAdjustsDestination() : 0) + " " + num(o->getCrusherLevel()) + " " +
            num(o->getCrushableLevel()) + " " + f32(o->getGeometryInfo().getBoundingCircleRadius()) + " " +
            num(o->getLayer()) + " " + num(o->isKindOf(KINDOF_INFANTRY) != 0));
    }
    for (Object* a : s.units) {
        std::string r = "rel " + num(a->getID());
        for (Object* b : s.units) {
            r += " " + num(b->getID()) + ":" + num(a->getRelationship(b) == ALLIES) +
                 num(a->canCrushOrSquish(b, TEST_CRUSH_OR_SQUISH));
        }
        row(r);
    }
}

static void occupy(const Scene& s) {
    for (int n = 20 + below(60); n > 0; --n) {
        const PathfindLayerEnum layer = some_layer(s);
        ICoord2D c = cell_near(s, layer);
        // Clusters, so that footprints meet several units.
        const int span = 1 + below(3);
        Object* u = pick(s.units);
        const int kind = below(4);  // goal, pos, both, aircraft goal
        for (int i = 0; i < span; ++i) {
            for (int j = 0; j < span; ++j) {
                PathfindCell* cell = pf()->getCell(layer, c.x + i, c.y + j);
                if (!cell) continue;
                ICoord2D at;
                at.x = c.x + i;
                at.y = c.y + j;
                const std::string where =
                    num(layer) + " " + num(at.x) + " " + num(at.y) + " " + num(u->getID());
                if (kind == 0 || kind == 2) {
                    cell->setGoalUnit(u->getID(), at);
                    row("goal " + where);
                }
                if (kind == 1 || kind == 2) {
                    cell->setPosUnit(u->getID(), at);
                    row("pos " + where);
                }
                if (kind == 3) {
                    cell->setGoalAircraft(u->getID(), at);
                    row("air " + where);
                }
            }
        }
    }
}

static void vacate(const Scene& s) {
    ICoord2D at;
    for (int layer = LAYER_GROUND; layer <= LAYER_WALL; layer += LAYER_WALL - LAYER_GROUND) {
        for (int x = 0; x <= s.w + 1; ++x) {
            for (int y = 0; y <= s.h + 1; ++y) {
                PathfindCell* cell = pf()->getCell(static_cast<PathfindLayerEnum>(layer), x, y);
                if (!cell) continue;
                at.x = x;
                at.y = y;
                cell->setGoalUnit(INVALID_ID, at);
                cell->setPosUnit(INVALID_ID, at);
                cell->setGoalAircraft(INVALID_ID, at);
            }
        }
    }
}

// The occupancy of both layers, to prove that a section left it alone.
static uint64_t occupancy(const Scene& s) {
    uint64_t h = 0xcbf29ce484222325ULL;
    for (int layer = LAYER_GROUND; layer <= LAYER_WALL; layer += LAYER_WALL - LAYER_GROUND) {
        for (int x = 0; x <= s.w + 1; ++x) {
            for (int y = 0; y <= s.h + 1; ++y) {
                PathfindCell* cell = pf()->getCell(static_cast<PathfindLayerEnum>(layer), x, y);
                if (!cell) continue;
                const uint64_t v = static_cast<uint64_t>(cell->getFlags()) ^
                                   (static_cast<uint64_t>(cell->getGoalUnit()) << 8) ^
                                   (static_cast<uint64_t>(cell->getPosUnit()) << 24) ^
                                   (static_cast<uint64_t>(cell->isAircraftGoal()) << 40) ^
                                   (static_cast<uint64_t>(cell->getType()) << 44) ^
                                   (static_cast<uint64_t>(cell->getPinched()) << 52);
                h = (h ^ v) * 0x100000001b3ULL;
            }
        }
    }
    return h;
}

static void unchanged(const Scene& s, uint64_t before, const char* section) {
    if (occupancy(s) != before) {
        std::fprintf(stderr, "%s changed the occupancy\n", section);
        std::exit(1);
    }
}

struct Walk {
    std::string out;
    PathfindCell* previous = nullptr;
    int left = 0;
};
static Int walk_proc(Pathfinder*, PathfindCell* from, PathfindCell* to, Int x, Int y, void* data) {
    Walk* w = static_cast<Walk*>(data);
    w->out += " " + num(x) + ":" + num(y) + ":" + (from == nullptr ? "n" : from == w->previous ? "p" : "x");
    w->previous = to;
    return --w->left == 0 ? 7 : 0;
}

static LocomotorSet* surface_set(int mask) {
    static LocomotorSet set;
    set.*access_member(LsSurfaces()) = mask;
    return &set;
}

static void queries(Scene& s) {
    for (int n = 0; n < 60; ++n) {
        const PathfindLayerEnum layer = some_layer(s);
        const Coord3D p = point_on(s, layer);
        const int k = below(9);
        const Locomotor* loco = TheLocomotorStore->newLocomotor(TheLocomotorStore->findLocomotorTemplate(
            NAMEKEY(("S3Loco" + num(k)).c_str())));
        row("terr " + num(layer) + " " + num(kMasks[k]) + " " + c3(p) + " = " +
            num(pf()->validMovementTerrain(layer, loco, &p)));
        deleteInstance(const_cast<Locomotor*>(loco));
        const bool crusher = chance(40);
        const ObjectID ignore = chance(30) && !s.blocks.empty() ? pick(s.blocks)->getID() : INVALID_ID;
        pf()->*access_member(PfIgnore()) = ignore;
        row("valid " + num(crusher) + " " + num(ignore) + " " + num(layer) + " " + num(kMasks[k]) + " " + c3(p) +
            " = " + num(pf()->validMovementPosition(crusher, layer, *surface_set(kMasks[k]), &p)));
        pf()->*access_member(PfIgnore()) = INVALID_ID;
    }
    {
        // chooseBestLocomotorForPosition over a set in a random order.
        for (int n = 0; n < 40; ++n) {
            LocomotorSet set;
            std::string have;
            std::vector<const Locomotor*> order;
            for (int i = 1 + below(3); i > 0; --i) {
                const int k = below(9);
                set.addLocomotor(TheLocomotorStore->findLocomotorTemplate(NAMEKEY(("S3Loco" + num(k)).c_str())));
                have += " " + num(kMasks[k]);
            }
            const PathfindLayerEnum layer = some_layer(s);
            const Coord3D p = point_on(s, layer);
            Locomotor* best = pf()->chooseBestLocomotorForPosition(layer, &set, &p);
            row("best " + num(layer) + " " + c3(p) + have + " = " +
                num(best ? best->getLegalSurfaces() : -1));
        }
    }
    for (int n = 0; n < 160; ++n) {
        Object* a = some_agent(s);
        const PathfindLayerEnum layer = some_layer(s);
        const ICoord2D c = cell_near(s, layer);
        const int radius = below(3);
        const bool center = chance(50);
        row("dest " + num(id_of(a)) + " " + num(layer) + " " + num(c.x) + " " + num(c.y) + " " +
            num(radius) + " " + num(center) + " = " +
            num((pf()->*access_member(PfCheckDest()))(a, c.x, c.y, layer, radius, center)));
    }
    for (int n = 0; n < 200; ++n) {
        Object* a = some_agent(s);
        TCheckMovementInfo info;
        std::memset(&info, 0, sizeof(info));
        info.layer = some_layer(s);
        info.cell = cell_near(s, info.layer);
        info.radius = below(3);
        info.centerInCell = chance(50);
        info.considerTransient = chance(50);
        const Bool ok = (pf()->*access_member(PfCheckMove()))(a, info);
        row("move " + num(id_of(a)) + " " + num(info.layer) + " " + num(info.cell.x) + " " + num(info.cell.y) +
            " " + num(info.radius) + " " + num(info.centerInCell) + " " + num(info.considerTransient) + " = " +
            num(ok) + " " + num(info.allyFixedCount) + " " + num(info.enemyFixed) + " " + num(info.allyMoving) +
            " " + num(info.allyGoal));
    }
    for (int n = 0; n < 120; ++n) {
        const bool crusher = chance(50);
        const PathfindLayerEnum layer = some_layer(s);
        const ICoord2D c = cell_near(s, layer);
        const int diameter = below(7);
        row("clear " + num(crusher) + " " + num(c.x) + " " + num(c.y) + " " + num(layer) + " " + num(diameter) +
            " = " + num((pf()->*access_member(PfClear()))(crusher, c.x, c.y, layer, diameter)));
    }
    for (int n = 0; n < 40; ++n) {
        const PathfindLayerEnum layer = some_layer(s);
        ICoord2D a = cell_near(s, layer), b = cell_near(s, layer);
        if (chance(25)) b.x = a.x + (b.y - a.y);  // diagonals
        if (chance(10)) b = a;
        Walk w;
        w.left = chance(20) ? 1 + below(8) : 0;
        const int left = w.left;
        const Int ret = (pf()->*access_member(PfIterate()))(a, b, layer, walk_proc, &w);
        row("cells " + num(a.x) + " " + num(a.y) + " " + num(b.x) + " " + num(b.y) + " " + num(layer) + " " +
            num(left) + " = " + num(ret) + w.out);
    }
    for (int n = 0; n < 220; ++n) {
        Object* a = some_agent(s);
        const int mask = kMasks[below(9)];
        const PathfindLayerEnum layer = some_layer(s);
        const Coord3D from = point_on(s, layer);
        const Coord3D to = near_point(s, from);
        const bool blocked = chance(50), pinched = chance(50);
        row("line " + num(id_of(a)) + " " + num(mask) + " " + num(layer) + " " + c3(from) + " " + c3(to) + " " +
            num(blocked) + " " + num(pinched) + " = " +
            num(pf()->isLinePassable(a, mask, layer, from, to, blocked, pinched)));
    }
    for (int n = 0; n < 100; ++n) {
        const bool crusher = chance(50);
        const PathfindLayerEnum layer = some_layer(s);
        const Coord3D from = point_on(s, layer);
        const Coord3D to = near_point(s, from);
        const int diameter = below(6);
        row("ground " + num(crusher) + " " + c3(from) + " " + num(layer) + " " + c3(to) + " " + num(diameter) +
            " = " + num(pf()->isGroundPathPassable(crusher, from, layer, to, diameter)));
    }
    for (int n = 0; n < 100; ++n) {
        const int mask = kMasks[below(9)];
        const Coord3D from = point(s, false);
        const Coord3D to = near_point(s, from);
        const std::string head = num(mask) + " " + c3(from) + " " + c3(to) + " " +
                                 num(TheTerrainLogic->getLayerForDestination(&from)) + " " +
                                 num(TheTerrainLogic->getLayerForDestination(&to));
        row("quick " + head + " = " +
            num(pf()->clientSafeQuickDoesPathExist(*surface_set(mask), &from, &to)) + " " +
            num(pf()->clientSafeQuickDoesPathExistForUI(*surface_set(mask), &from, &to)));
    }
}

static void views(Scene& s) {
    std::vector<Object*> parties = s.units;
    parties.insert(parties.end(), s.blocks.begin(), s.blocks.end());
    TAiData* data = const_cast<TAiData*>(TheAI->getAiData());
    for (int n = 0; n < 160; ++n) {
        Object* a = pick(parties);
        if (chance(60)) {
            for (int tries = 0; tries < 20 && !a->isKindOf(KINDOF_ATTACK_NEEDS_LINE_OF_SIGHT); ++tries) a = pick(parties);
        }
        Object* v = chance(10) ? nullptr : pick(parties);
        const bool uses = !chance(8);
        data->m_attackUsesLineOfSight = uses;
        const PathfindLayerEnum al = chance(12) ? LAYER_WALL : LAYER_GROUND;
        const PathfindLayerEnum vl = chance(12) ? LAYER_WALL : LAYER_GROUND;
        const PathfindLayerEnum was_a = a->getLayer();
        a->*access_member(ObjLayer()) = al;
        const PathfindLayerEnum was_v = v ? v->getLayer() : LAYER_GROUND;
        if (v) v->*access_member(ObjLayer()) = vl;
        Coord3D from = chance(70) ? *a->getPosition() : point(s);
        Coord3D to = v && chance(70) ? *v->getPosition() : near_point(s, from);
        const std::string head =
            num(uses) + " " + num(a->getID()) + " " + num(a->isKindOf(KINDOF_ATTACK_NEEDS_LINE_OF_SIGHT) != 0) + " " +
            num(a->getLayer()) + " " + num(v ? v->getID() : 0) + " " + num(v ? v->getLayer() : 0);
        row("view " + head + " " + c3(from) + " " + c3(to) + " = " +
            num(pf()->isAttackViewBlockedByObstacle(a, from, v, to)));
        if (v) {
            row("sight " + head + " " + c3(*a->getPosition()) + " " + c3(*v->getPosition()) + " " +
                num(v->isSignificantlyAboveTerrain()) + " = " + num(pf()->isViewBlockedByObstacle(a, v)));
        }
        a->*access_member(ObjLayer()) = was_a;
        if (v) v->*access_member(ObjLayer()) = was_v;
    }
    data->m_attackUsesLineOfSight = TRUE;
}

static std::string path_dump(Path& path) {
    std::vector<PathNode*> nodes;
    for (PathNode* n = path.getFirstNode(); n; n = n->getNext()) nodes.push_back(n);
    auto index = [&](const PathNode* n) {
        for (size_t i = 0; i < nodes.size(); ++i)
            if (nodes[i] == n) return static_cast<int>(i);
        return -1;
    };
    std::string out = num(static_cast<long long>(nodes.size())) + " " + num(index(path.getLastNode()));
    for (PathNode* n : nodes) {
        Coord2D dir;
        Real dist;
        const PathNode* opti = n->getNextOptimized(&dir, &dist);
        out += " | " + c3(*n->getPosition()) + " " + num(n->getLayer()) + " " + num(n->getCanOptimize() != 0) + " " +
               num(opti ? index(opti) : -1) + " " + f32(dist) + " " + f32(dir.x) + " " + f32(dir.y) + " " +
               c3(*n->computeDirectionVector()) + " " + num(index(n->getPrevious()));
    }
    return out;
}

static void tall_buildings(Scene& s) {
    for (Object* o : s.tall) {
        row("tall " + num(o->getID()) + " " + c3(*o->getPosition()) + " " +
            f32(o->getGeometryInfo().getBoundingCircleRadius()));
    }
    if (s.tall.empty()) return;
    auto around = [&](Coord3D& p) {
        p = *pick(s.tall)->getPosition();
        const float reach = chance(40) ? 20.0f : 120.0f;
        p.x += between(-reach, reach);
        p.y += between(-reach, reach);
        if (chance(6)) p = *pick(s.tall)->getPosition();
    };
    for (int n = 0; n < 60; ++n) {
        Coord3D from, to;
        around(from);
        around(to);
        const ObjectID ignore = chance(15) ? pick(s.tall)->getID() : INVALID_ID;
        PathNode* a = newInstance(PathNode);
        PathNode* b = newInstance(PathNode);
        a->setPosition(&from);
        b->setPosition(&to);
        Coord3D p1, p2, p3;
        p1.zero();
        p2.zero();
        p3.zero();
        const Bool hit = (pf()->*access_member(PfSegment()))(a, b, ignore, &p1, &p2, &p3);
        std::string r = "segment " + c3(from) + " " + c3(to) + " " + num(ignore) + " = " + num(hit) + " " +
                        c3(*b->getPosition());
        if (hit) r += " " + c3(p1) + " " + c3(p2) + " " + c3(p3);
        row(r);
        deleteInstance(a);
        deleteInstance(b);
    }
    for (int n = 0; n < 40; ++n) {
        Coord3D from, to, adjusted;
        around(from);
        around(to);
        adjusted = to;
        const Real radius = chance(50) ? 100.0f : between(0.0f, 60.0f);
        g_record = true;
        const Bool hit = (pf()->*access_member(PfCircle()))(&from, &to, radius, INVALID_ID, &adjusted);
        g_record = false;
        row("circle " + c3(from) + " " + c3(to) + " " + f32(radius) + " = " + num(hit) + " " + c3(adjusted));
    }
}

static void aircraft(Scene& s, const std::vector<Object*>& flyers) {
    for (int n = 0; n < 30; ++n) {
        Object* o = pick(flyers);
        Coord3D at = point(s, false);
        if (!s.tall.empty() && chance(60)) {
            at = *pick(s.tall)->getPosition();
            at.x -= between(20.0f, 150.0f);
            at.y += between(-60.0f, 60.0f);
        }
        at.z = between(0.0f, 50.0f);
        o->setPosition(&at);
        Coord3D to = near_point(s, at);
        if (!s.tall.empty() && chance(60)) {
            to = *pick(s.tall)->getPosition();
            to.x += between(-30.0f, 150.0f);
            to.y += between(-60.0f, 60.0f);
        }
        const AIUpdateInterface* ai = o->getAI();
        const bool winged = ai && ai->getCurLocomotor() && ai->getCurLocomotor()->getAppearance() == LOCO_WINGS;
        g_record = true;
        Path* path = pf()->getAircraftPath(o, &to);
        g_record = false;
        row("aircraft " + c3(at) + " " + c3(to) + " " + num(winged) + " = " + path_dump(*path));
        deleteInstance(path);
    }
}

static PathNode* nth(Path& path, int i) {
    PathNode* n = path.getFirstNode();
    while (n && i-- > 0) n = n->getNext();
    return n;
}
static int count_nodes(Path& path) {
    int n = 0;
    for (PathNode* p = path.getFirstNode(); p; p = p->getNext()) ++n;
    return n;
}

// A walk over cells, as the searches build paths, with some jumps.
static void walk_path(const Scene& s, Path& path, bool marks) {
    const PathfindLayerEnum start = some_layer(s);
    Coord3D p = point_on(s, start);
    p.x = std::floor(p.x / 10.0f) * 10.0f + (chance(50) ? 5.0f : 0.5f);
    p.y = std::floor(p.y / 10.0f) * 10.0f + (chance(50) ? 5.0f : 0.5f);
    int dx = below(3) - 1, dy = below(3) - 1;
    PathfindLayerEnum layer = start;
    const int n = 1 + below(chance(15) ? 30 : 14);
    for (int i = 0; i < n; ++i) {
        row("append " + c3(p) + " " + num(layer));
        path.appendNode(&p, layer);
        if (marks) {
            const bool can = !chance(8);
            path.getLastNode()->setCanOptimize(can);
            row("canopt " + num(count_nodes(path) - 1) + " " + num(can));
        }
        if (chance(30)) {
            dx = below(3) - 1;
            dy = below(3) - 1;
        }
        if (!s.walls.empty() && chance(8)) layer = layer == LAYER_GROUND ? LAYER_WALL : LAYER_GROUND;
        p.x += 10.0f * dx;
        p.y += 10.0f * dy;
        if (chance(8)) p.x += between(-25.0f, 25.0f);
        if (chance(8)) p.y += between(-25.0f, 25.0f);
        if (chance(5)) p.z = between(0.0f, 20.0f);
    }
}

static void dump_row(Path& path) { row("dump = " + path_dump(path)); }

static void save_rows(Path& path) {
    std::string hex = oracle_wp51::World::snapshot_save_hex(&path);
    // The word before the blocked flag is uninitialized stack memory.
    for (size_t i = hex.size() - 10; i < hex.size() - 2; ++i) hex[i] = '0';
    row("save = " + hex + " " + num(oracle_wp51::World::snapshot_crc(&path)));
    std::vector<uint8_t> bytes;
    for (size_t i = 0; i + 1 < hex.size(); i += 2)
        bytes.push_back(static_cast<uint8_t>(std::stoul(hex.substr(i, 2), nullptr, 16)));
    Path* loaded = newInstance(Path);
    oracle_wp51::World::snapshot_load_bytes(loaded, bytes);
    row("load " + hex + " = " + path_dump(*loaded));
    std::string again = oracle_wp51::World::snapshot_save_hex(loaded);
    for (size_t i = again.size() - 10; i < again.size() - 2; ++i) again[i] = '0';
    row("resave = " + again);
    deleteInstance(loaded);
}

static void point_queries(const Scene& s, Path& path, int count) {
    Object* a = pick(s.units);
    Coord3D p = point(s, false);
    if (path.getFirstNode()) p = *path.getFirstNode()->getPosition();
    int mask = kMasks[below(9)];
    for (int n = 0; n < count; ++n) {
        const int len = count_nodes(path);
        if (len > 0 && chance(45)) {
            p = *nth(path, below(len))->getPosition();
            const float reach = chance(30) ? 3.0f : chance(60) ? 18.0f : 60.0f;
            p.x += between(-reach, reach);
            p.y += between(-reach, reach);
        } else if (chance(55)) {
            // Hardly moved: the cached answer.
            p.x += chance(50) ? 0.05f : 0.0f;
            p.y += chance(10) ? 0.11f : 0.0f;
        } else if (chance(30)) {
            a = pick(s.units);
            mask = kMasks[below(9)];
        }
        if (chance(15) && len > 0) {
            p = *nth(path, below(len))->getPosition();  // on a node
        }
        const PathfindLayerEnum was = a->getLayer();
        const PathfindLayerEnum al = chance(10) ? LAYER_WALL : LAYER_GROUND;
        a->*access_member(ObjLayer()) = al;
        ClosestPointOnPathInfo out;
        const int repeat = chance(4) ? 24 : 1;  // run the cache countdown out
        for (int r = 0; r < repeat; ++r) {
            path.computePointOnPath(a, *surface_set(mask), p, out);
            Coord3D peek;
            path.peekCachedPointOnPath(peek);
            row("cpop " + num(a->getID()) + " " + num(al) + " " + num(mask) + " " + c3(p) + " = " +
                f32(out.distAlongPath) + " " + c3(out.posOnPath) + " " + num(out.layer) + " " + c3(peek));
        }
        a->*access_member(ObjLayer()) = was;
        if (chance(50)) {
            Coord3D goal;
            const Real dist = path.computeFlightDistToGoal(&p, goal);
            row("flight " + c3(p) + " = " + f32(dist) + " " + c3(goal));
        }
    }
}

static void paths(Scene& s) {
    for (int n = 0; n < 12; ++n) {
        Path* path = newInstance(Path);
        row("path");
        if (n == 0) {
            ClosestPointOnPathInfo out;
            const Coord3D p = point(s);
            path->computePointOnPath(pick(s.units), *surface_set(1), p, out);
            Coord3D goal;
            const Real dist = path->computeFlightDistToGoal(&p, goal);
            row("empty " + c3(p) + " = " + f32(out.distAlongPath) + " " + c3(out.posOnPath) + " " +
                num(out.layer) + " " + f32(dist) + " " + c3(goal));
            save_rows(*path);
        }
        walk_path(s, *path, true);
        if (chance(30)) {
            Coord3D p = point(s);
            const PathfindLayerEnum layer = some_layer(s);
            path->prependNode(&p, layer);
            row("prepend " + c3(p) + " " + num(layer));
        }
        dump_row(*path);
        if (chance(30)) point_queries(s, *path, 3);  // before it is optimized
        if (n % 3 == 2) {
            const bool crusher = chance(50);
            const int diameter = below(5);
            path->optimizeGroundPath(crusher, diameter);
            row("gopt " + num(crusher) + " " + num(diameter));
        } else {
            Object* a = some_agent(s);
            const int mask = kMasks[below(9)];
            const bool blocked = chance(40);
            path->optimize(a, mask, blocked);
            row("opt " + num(id_of(a)) + " " + num(mask) + " " + num(blocked));
        }
        dump_row(*path);
        if (chance(30)) {
            const bool blocked = chance(70);
            path->setBlockedByAlly(blocked);
            row("blocked " + num(blocked) + " = " + num(path->getBlockedByAlly()));
        }
        point_queries(s, *path, 14);
        if (chance(60)) {
            Coord3D p = *path->getLastNode()->getPosition();
            p.x += between(-30.0f, 30.0f);
            p.y += between(-30.0f, 30.0f);
            const PathfindLayerEnum layer = TheTerrainLogic->getLayerForDestination(&p);
            path->updateLastNode(&p);
            row("update " + c3(p) + " " + num(layer));
            dump_row(*path);
        }
        if (chance(60)) {
            Coord3D p = *path->getLastNode()->getPosition();
            if (chance(50)) p.x += 20.0f;  // else the same place, which is refused
            p.z += 1.0f;
            path->appendNode(&p, LAYER_GROUND);
            row("append " + c3(p) + " " + num(LAYER_GROUND));
            dump_row(*path);
        }
        if (chance(30) && count_nodes(*path) > 1) {
            const int i = below(count_nodes(*path));
            // Only forward links: a cycle would make the queries loop forever.
            const int j = chance(25) || i + 1 >= count_nodes(*path) ? -1 : i + 1 + below(count_nodes(*path) - i - 1);
            nth(*path, i)->setNextOptimized(j < 0 ? nullptr : nth(*path, j));
            row("setopt " + num(i) + " " + num(j));
            dump_row(*path);
        }
        point_queries(s, *path, 4);
        save_rows(*path);
        deleteInstance(path);
    }
}

static void run_scene(int index) {
    Scene s;
    std::fprintf(stderr, "scene %d at %zu rows\n", index, g_rows.size());
    row("scenario " + num(index));
    s.w = 14 + below(28);
    s.h = 14 + below(22);
    s.maxx = s.w * 10.0f;
    s.maxy = s.h * 10.0f;
    S3Terrain& t = *g_terrain;
    t.w = s.w;
    t.h = s.h;
    t.cliff.assign(static_cast<size_t>((s.w + 2) * (s.h + 2)), '0');
    t.water = t.cliff;
    if (index % 4 != 0) {
        blobs(t.cliff, below(4));
        blobs(t.water, below(4));
    }
    t.extent.lo.x = 0.0f;
    t.extent.lo.y = 0.0f;
    t.extent.lo.z = 0.0f;
    t.extent.hi.x = s.maxx;
    t.extent.hi.y = s.maxy;
    t.extent.hi.z = 100.0f;
    const float wall_height = 12.0f;
    const_cast<TAiData*>(TheAI->getAiData())->m_wallHeight = wall_height;
    pf()->reset();
    row("map " + f32(s.maxx) + " " + f32(s.maxy) + " " + f32(wall_height));
    row("terrain " + num(s.w) + " " + num(s.h) + " " + t.cliff + " " + t.water);
    g_world->set_frame(100);
    // An AI picks its locomotor from the map as it is constructed.
    pf()->newMap();

    for (int n = 3 + below(index % 4 == 0 ? 4 : 12); n > 0; --n) {
        const std::string name = g_blocks[below(static_cast<int>(g_blocks.size()))];
        Object* o = spawn(name, g_world->team_b, coord(0.0f, s.maxx), coord(0.0f, s.maxy), 0.0f,
                          chance(50) ? 0.0f : between(-3.2f, 3.2f));
        s.blocks.push_back(o);
        if (o->isKindOf(KINDOF_AIRCRAFT_PATH_AROUND)) s.tall.push_back(o);
    }
    if (index % 3 != 0) {
        float x = coord(30.0f, s.maxx * 0.5f);
        const float y = coord(30.0f, s.maxy - 30.0f);
        for (int n = 1 + below(5); n > 0; --n) {
            Object* o = spawn("S3Wall", g_world->team_a, x, y, 0.0f, 0.0f);
            s.walls.push_back(o);
            x += 48.0f;
        }
    }
    std::vector<Object*> flyers;
    for (int n = 4 + below(6); n > 0; --n) {
        const std::string name = g_units[below(static_cast<int>(g_units.size()))];
        Object* o = spawn(name, chance(55) ? g_world->team_a : g_world->team_b, coord(0.0f, s.maxx),
                          coord(0.0f, s.maxy), chance(15) ? 30.0f : 0.0f, 0.0f);
        s.units.push_back(o);
    }
    for (const char* name : {"S3Plane", "S3Chopper"}) {
        Object* o = spawn(name, g_world->team_a, coord(0.0f, s.maxx), coord(0.0f, s.maxy), 0.0f, 0.0f);
        s.units.push_back(o);
        flyers.push_back(o);
    }
    flyers.push_back(s.units[0]);
    for (Object* o : s.units) {
        if (o->getAIUpdateInterface() && !s.blocks.empty() && chance(35)) {
            o->getAIUpdateInterface()->ignoreObstacle(pick(s.blocks));
        }
    }
    s.everything = s.blocks;
    s.everything.insert(s.everything.end(), s.walls.begin(), s.walls.end());
    s.everything.insert(s.everything.end(), s.units.begin(), s.units.end());
    // `reset` forgets the wall pieces, so it comes before they are added.
    pf()->reset();
    for (Object* o : s.walls) {
        pf()->addWallPiece(o);
        row("wallpiece " + num(o->getID()));
    }
    std::string order = "order";
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
        order += " " + num(o->getID());
        object_row(o);
    }
    row(order);
    ThePartitionManager->update();
    pf()->newMap();
    row("newmap 100");
    unit_rows(s);
    occupy(s);

    const uint64_t occupied = occupancy(s);
    queries(s);
    unchanged(s, occupied, "queries");
    views(s);
    unchanged(s, occupied, "views");
    tall_buildings(s);
    unchanged(s, occupied, "tall buildings");
    paths(s);
    unchanged(s, occupied, "paths");
    // Last: moving the flyers runs `updatePos`, which rewrites the occupancy.
    aircraft(s, flyers);

    vacate(s);
    for (Object* o : s.everything) TheGameLogic->destroyObject(o);
    TheGameLogic->processDestroyList();
    ThePartitionManager->update();
    TheTerrainLogic->reset();
    pf()->reset();
    row("end");
}

int main(int argc, char** argv) {
    if (argc < 2) return 2;
    oracle_wp51::World world;
    g_world = &world;
    make_templates();
    world.load_ini(g_ini);
    g_terrain = new S3Terrain;
    TheTerrainLogic = g_terrain;
    for (int index = 0; index < 9; ++index) run_scene(index);

    std::FILE* out = std::fopen(argv[1], "wb");
    if (!out) return 1;
    std::fputs("{\n \"rows\": [", out);
    for (size_t r = 0; r < g_rows.size(); ++r) {
        std::fprintf(out, "%s\n  \"%s\"", r ? "," : "", g_rows[r].c_str());
    }
    std::fputs("\n ]\n}\n", out);
    std::fclose(out);
    std::fprintf(stderr, "%zu rows\n", g_rows.size());
    return 0;
}
