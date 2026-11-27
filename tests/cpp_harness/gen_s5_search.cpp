// Golden vectors for slice S5: the open and closed lists,
// examineNeighboringCells, internalFindPath, prependCells, clip, checkPathCost,
// cleanOpenAndClosedLists, forceCleanCells, the switch to fixed pathfinding,
// the hierarchical searches, findPath, findGroundPath, slowDoesPathExist and
// the path request queue, all in the unmodified reference library.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_s5_search.cpp -o gen_s5
//   ./gen_s5 docs/golden_vectors/S5_Search.json
//
// Only the terrain is synthetic. The map is built by the real newMap; unit
// occupancy and the wall's layer connections are written with the real
// PathfindCell setters. Each row is an input to replay, or a call with the
// reference's answer after " = ". After every call the whole search state is
// written as digests: every cell info record (the released ones keep the costs
// of the cells a search examined), the free list, every cell and the zone
// blocks' passable flags.
//
// Not covered: pool exhaustion, the aircraft and line-of-sight cases covered by
// earlier slices. doPathfind is answered by the driver, which records the
// queue's calls and runs a scripted search.
//
// oracle-wrap: GeneralsMD/Code/GameEngine/libz_gameengine.a AIUpdate.cpp.o __ZN17AIUpdateInterface10doPathfindEP25PathfindServicesInterface

#include "oracle_wp51_world.h"

#include "GameLogic/AIPathfind.h"
#include "GameLogic/Locomotor.h"
#include "GameLogic/LocomotorSet.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Pathfinder/PathfindCell.h"
#include "GameLogic/Pathfinder/PathfindCellInfo.h"
#include "GameLogic/Pathfinder/PathfindCellList.h"
#include "GameLogic/Pathfinder/PathfindConstants.h"
#include "GameLogic/Pathfinder/PathfindLayer.h"
#include "GameLogic/Pathfinder/PathNode.h"
#include "GameLogic/Pathfinder/Path.h"
#include "GameLogic/TerrainLogic.h"

#include <cmath>
#include <cstdio>
#include <cstring>
#include <map>
#include <string>
#include <vector>

#define S5_TAG(Tag, Type, Class, Member)                   \
    struct Tag { using type = Type Class::*; };            \
    template struct MemberAccess<Tag, &Class::Member>;     \
    inline Type Class::* access_member(Tag);
#define S5_STATIC(Tag, Type, Member)                       \
    struct Tag { using type = Type*; };                    \
    template struct MemberAccess<Tag, &Member>;            \
    inline Type* access_member(Tag);
#define S5_FN(Tag, Signature, Member)                      \
    struct Tag { using type = Signature; };                \
    template struct MemberAccess<Tag, &Member>;            \
    inline Tag::type access_member(Tag);

S5_TAG(LsSurfaces, LocomotorSurfaceTypeMask, LocomotorSet, m_validLocomotorSurfaces)
S5_TAG(LsDownhill, Bool, LocomotorSet, m_downhillOnly)
S5_TAG(ObjLayer, PathfindLayerEnum, Object, m_layer)
S5_TAG(AiThrough, Bool, AIUpdateInterface, m_canPathThroughUnits)
S5_TAG(PlayerKind, PlayerType, Player, m_playerType)
S5_TAG(PfIgnore, ObjectID, Pathfinder, m_ignoreObstacleID)
S5_TAG(PfTunnel, Bool, Pathfinder, m_isTunneling)
S5_TAG(PfCumulative, Int, Pathfinder, m_cumulativeCellsAllocated)
S5_TAG(PfOpen, PathfindCellList, Pathfinder, m_openList)
S5_TAG(PfClosed, PathfindCellList, Pathfinder, m_closedList)
S5_TAG(PfZones, PathfindZoneManager, Pathfinder, m_zoneManager)
S5_TAG(CellInfo, PathfindCellInfo*, PathfindCell, m_info)
S5_STATIC(InfoArray, PathfindCellInfo*, PathfindCellInfo::s_infoArray)
S5_STATIC(InfoFree, PathfindCellInfo*, PathfindCellInfo::s_firstFree)
S5_FN(PfInternal, Path* (Pathfinder::*)(Object*, const LocomotorSet&, const Coord3D*, const Coord3D*),
      Pathfinder::internalFindPath)
S5_FN(PfCost, Int (Pathfinder::*)(Object*, const LocomotorSet&, const Coord3D*, const Coord3D*),
      Pathfinder::checkPathCost)
S5_FN(PfExamine, Int (Pathfinder::*)(PathfindCell*, PathfindCell*, const LocomotorSet&, Bool, Bool, Int,
                                     const ICoord2D&, const Object*, Int),
      Pathfinder::examineNeighboringCells)
S5_FN(PfPrepend, void (Pathfinder::*)(Path*, const Coord3D*, PathfindCell*, Bool), Pathfinder::prependCells)
typedef Path* (Pathfinder::*S5Hier)(Bool, const LocomotorSet&, const Coord3D*, const Coord3D*, Bool);
S5_FN(PfHier, S5Hier, Pathfinder::findHierarchicalPath)
S5_FN(PfClosest, S5Hier, Pathfinder::findClosestHierarchicalPath)

// The layout of a `PathfindCellInfo`, whose members are protected.
struct InfoView {
    InfoView *nextOpen, *prevOpen, *pathParent;
    PathfindCell* cell;
    UnsignedShort totalCost, costSoFar;
    ICoord2D pos;
    ObjectID goalUnit, posUnit, goalAircraft, obstacle;
    UnsignedInt isFree : 1, blockedByAlly : 1, fence : 1, transparent : 1, open : 1, closed : 1;
};
static_assert(sizeof(InfoView) == sizeof(PathfindCellInfo), "PathfindCellInfo layout");
static const int INFO_COUNT = 30000;

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
static std::string hex64(uint64_t v) {
    char buf[24];
    std::snprintf(buf, sizeof(buf), "%016llx", static_cast<unsigned long long>(v));
    return buf;
}
static void row(const std::string& r) { g_rows.push_back(r); }

static uint32_t g_state = 0x53354152u;
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

static const float kHeights[] = {0.0f, 3.0f, 9.5f, 10.0f, 14.0f, 25.0f};

class S5Terrain : public oracle_wp51::OracleTerrainLogic {
public:
    int w = 1, h = 1;
    std::string cliff, water, height;
    size_t at(int i, int j) const {
        i = i < 0 ? 0 : (i > w + 1 ? w + 1 : i);
        j = j < 0 ? 0 : (j > h + 1 ? h + 1 : j);
        return static_cast<size_t>(j * (w + 2) + i);
    }
    static int cell(float v) { return static_cast<int>(std::floor(v / 10.0f)); }
    Real getGroundHeight(Real x, Real y, Coord3D* normal = nullptr) const override {
        if (normal) { normal->x = 0.0f; normal->y = 0.0f; normal->z = 1.0f; }
        return kHeights[height[at(cell(x), cell(y))] - '0'];
    }
    Real getLayerHeight(Real x, Real y, PathfindLayerEnum layer, Coord3D* normal = nullptr,
                        Bool = true) const override {
        return getGroundHeight(x, y, normal) + static_cast<float>(layer - LAYER_GROUND) * 7.5f;
    }
    Bool isCliffCell(Real x, Real y) const override { return cliff[at(cell(x), cell(y))] == '1'; }
    Bool isUnderwater(Real x, Real y, Real* waterZ = nullptr, Real* terrainZ = nullptr) override {
        if (waterZ) *waterZ = 0.0f;
        if (terrainZ) *terrainZ = 0.0f;
        return water[at(cell(x), cell(y))] == '1';
    }
};

static S5Terrain* g_terrain = nullptr;
static oracle_wp51::World* g_world = nullptr;
static Pathfinder* pf() { return TheAI->pathfinder(); }
// A C cast reaches the private base, whose findPath is public.
static PathfindServicesInterface* services() { return (PathfindServicesInterface*)pf(); }
static ObjectID oid(int n) { return static_cast<ObjectID>(n); }
static PathfindZoneManager& zones() { return pf()->*access_member(PfZones()); }

static void blobs(std::string& table, char value, int count, int reach) {
    S5Terrain& t = *g_terrain;
    for (int n = 0; n < count; ++n) {
        const int x = below(t.w + 2), y = below(t.h + 2), rx = 1 + below(reach), ry = 1 + below(reach);
        for (int j = y; j < y + ry; ++j)
            for (int i = x; i < x + rx; ++i) table[t.at(i, j)] = value;
    }
}

// ---- templates and objects ------------------------------------------------------

enum { F_STRUCTURE = 64, F_SEETHROUGH = 128, F_MOBILE = 256, F_WALKWALL = 32, F_DEFWALL = 8 };

static std::string g_ini;
static std::vector<std::string> g_blocks, g_units;

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
    g_ini += "Locomotor S5Loco\n  Surfaces = GROUND\n  Appearance = TWO_LEGS\n  Speed = 30\nEnd\n";
    const std::string ai = "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n  Locomotor = SET_NORMAL S5Loco\n";
    static const float radii[] = {2.0f, 5.0f, 7.0f, 9.0f, 10.0f, 12.0f, 15.0f, 19.0f, 22.0f, 40.0f};
    for (int i = 0; i < 20; ++i) {
        const std::string name = "S5Unit" + num(i);
        const int crusher = i % 5 == 4 ? 3 : i % 3 == 2 ? 1 : 0;
        std::string kinds = i % 3 == 0 ? "INFANTRY" : "VEHICLE";
        if (i % 4 == 1) kinds += " DOZER";
        std::string extra =
            "  CrusherLevel = " + num(crusher) + "\n  CrushableLevel = " + num(i % 4) + "\n" + ai;
        object_ini(name, kinds, "CYLINDER", radii[i % 10], radii[i % 10], extra);
        g_units.push_back(name);
    }
    static const char* geoms[] = {"BOX", "CYLINDER", "SPHERE"};
    for (int i = 0; i < 12; ++i) {
        const std::string name = "S5Block" + num(i);
        object_ini(name, i % 6 == 1 ? "STRUCTURE IMMOBILE CAN_SEE_THROUGH_STRUCTURE" : "STRUCTURE IMMOBILE",
                   geoms[i % 3], between(4.0f, 30.0f), between(4.0f, 30.0f), "");
        g_blocks.push_back(name);
    }
    for (int i = 0; i < 4; ++i) {
        const std::string name = "S5Fence" + num(i);
        char extra[64];
        std::snprintf(extra, sizeof(extra), "  FenceWidth = %.4f\n", between(15.0f, 70.0f));
        object_ini(name, "STRUCTURE IMMOBILE", "BOX", 20, 3, extra);
        g_blocks.push_back(name);
    }
    object_ini("S5Wall", "STRUCTURE IMMOBILE WALK_ON_TOP_OF_WALL", "BOX", 30, 18, "");
    object_ini("S5Deck", "IMMOBILE", "BOX", 30, 12, "");
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

// ---- scene ----------------------------------------------------------------------

struct Scene {
    std::vector<Coord3D> bridge_points;
    std::vector<std::string> bridges;
    int w = 0, h = 0;
    float maxx = 0, maxy = 0;
    std::vector<Object*> blocks, units, walls, everything;
    std::vector<PathfindCell*> cells;  // ground, then wall cells
    std::vector<int> keys;
    std::map<const PathfindCell*, int> key_of;
};

static Object* pick(const std::vector<Object*>& v) { return v[below(static_cast<int>(v.size()))]; }

static void index_cells(Scene& s) {
    for (int layer : {LAYER_GROUND, LAYER_WALL}) {
        for (int x = 0; x <= s.maxx / 10.0f - 1; ++x) {
            for (int y = 0; y <= s.maxy / 10.0f - 1; ++y) {
                PathfindCell* c = pf()->getCell(static_cast<PathfindLayerEnum>(layer), x, y);
                if (!c || (layer != LAYER_GROUND && c == pf()->getCell(LAYER_GROUND, x, y))) continue;
                const int key = layer * 1000000 + x * 1000 + y;
                s.cells.push_back(c);
                s.keys.push_back(key);
                s.key_of[c] = key;
            }
        }
    }
}

static int key(const Scene& s, const PathfindCell* c) {
    if (!c) return -1;
    auto it = s.key_of.find(c);
    return it == s.key_of.end() ? -2 : it->second;
}

// ---- state digests ----------------------------------------------------------------

struct Fnv {
    uint64_t h = 0xcbf29ce484222325ULL;
    void add(long long v) { h = (h ^ static_cast<uint64_t>(v)) * 0x100000001b3ULL; }
};

static InfoView* infos() { return reinterpret_cast<InfoView*>(*access_member(InfoArray())); }
static long long info_index(const InfoView* p) { return p ? p - infos() : -1; }

// One record: whose it is, its links, costs and flags.
static std::vector<long long> record(const Scene& s, int i) {
    const InfoView& v = infos()[i];
    const bool used = !v.isFree;
    const long long parent = used && v.pathParent ? key(s, v.pathParent->cell) : -1;
    return {i,
            key(s, v.cell),
            parent,
            info_index(v.nextOpen),
            info_index(v.prevOpen),
            v.totalCost,
            v.costSoFar,
            v.pos.x,
            v.pos.y,
            static_cast<long long>(v.goalUnit),
            static_cast<long long>(v.posUnit),
            static_cast<long long>(v.goalAircraft),
            static_cast<long long>(v.obstacle),
            v.isFree + 2 * v.blockedByAlly + 4 * v.fence + 8 * v.transparent + 16 * v.open + 32 * v.closed};
}

// Records no call has taken still sit in order at the end of the free list;
// their fields are whatever the allocation held, so they are left out.
static int used_records() {
    std::vector<long long> list;
    for (InfoView* p = reinterpret_cast<InfoView*>(*access_member(InfoFree()));
         p && list.size() <= static_cast<size_t>(INFO_COUNT); p = p->pathParent) {
        list.push_back(info_index(p));
    }
    if (list.empty() || list.back() != INFO_COUNT - 1) return INFO_COUNT;
    size_t k = list.size() - 1;
    while (k > 0 && list[k - 1] == list[k] - 1) --k;
    return static_cast<int>(list[k]);
}

static std::string pool_digest(const Scene& s) {
    Fnv f;
    for (int i = 0, used = used_records(); i < used; ++i) {
        for (long long v : record(s, i)) f.add(v);
    }
    return hex64(f.h);
}

static std::string free_digest() {
    Fnv f;
    int n = 0;
    for (InfoView* p = reinterpret_cast<InfoView*>(*access_member(InfoFree())); p && n <= INFO_COUNT;
         p = p->pathParent, ++n) {
        f.add(info_index(p));
    }
    return hex64(f.h);
}

static std::string cells_digest(const Scene& s) {
    Fnv f;
    for (size_t n = 0; n < s.cells.size(); ++n) {
        PathfindCell* c = s.cells[n];
        InfoView* v = reinterpret_cast<InfoView*>(c->*access_member(CellInfo()));
        f.add(info_index(v));
        f.add(c->getType());
        f.add(c->getFlags());
        f.add(c->getConnectLayer());
        f.add(c->getPinched());
        f.add(c->isAircraftGoal());
        f.add(c->getObstacleID());
        f.add(v || s_useFixedPathfinding ? c->isBlockedByAlly() : 0);
    }
    return hex64(f.h);
}

static std::string pass_bits(const Scene& s) {
    std::string out;
    for (int x = 0; x < s.maxx / 10.0f; x += 10)
        for (int y = 0; y < s.maxy / 10.0f; y += 10) out += zones().isPassable(x, y) ? '1' : '0';
    return out;
}

static std::string state(const Scene& s) {
    return num(s_useFixedPathfinding) + " " + num(s_forceCleanCells) + " " + num(pf()->*access_member(PfTunnel())) +
           " " + num(pf()->*access_member(PfCumulative())) + " " + pool_digest(s) + " " + free_digest() + " " +
           cells_digest(s) + " " + pass_bits(s);
}

// Every record in full, to find what a digest disagrees about.
static void full_state(const Scene& s) {
    std::string out = "records =";
    for (int i = 0, used = used_records(); i < used; ++i) {
        const std::vector<long long> r = record(s, i);
        out += " ";
        for (size_t k = 0; k < r.size(); ++k) out += (k ? "," : "") + num(r[k]);
    }
    row(out);
}

static std::string path_dump(Path* path) {
    if (!path) return "null";
    std::vector<PathNode*> nodes;
    for (PathNode* n = path->getFirstNode(); n; n = n->getNext()) nodes.push_back(n);
    auto index = [&](const PathNode* n) {
        for (size_t i = 0; i < nodes.size(); ++i)
            if (nodes[i] == n) return static_cast<int>(i);
        return -1;
    };
    std::string out = num(static_cast<long long>(nodes.size())) + " " + num(path->getBlockedByAlly());
    for (PathNode* n : nodes) {
        out += " / " + c3(*n->getPosition()) + " " + num(n->getLayer()) + " " + num(n->getCanOptimize() != 0) + " " +
               num(index(n->getNextOptimized()));
    }
    return out;
}

// ---- setup ----------------------------------------------------------------------

static Coord3D point(const Scene& s, bool off_map = true) {
    Coord3D p;
    const float out = off_map && chance(6) ? 40.0f : 0.0f;
    p.x = coord(-out, s.maxx + out);
    p.y = coord(-out, s.maxy + out);
    p.z = chance(70) ? 0.0f : between(0.0f, 20.0f);
    if (!s.walls.empty() && chance(12)) {
        p = *pick(s.walls)->getPosition();
        p.x += between(-30.0f, 30.0f);
        p.y += between(-18.0f, 18.0f);
        p.z = 12.0f;
    }
    return p;
}

static void unit_rows(const Scene& s) {
    for (Object* o : s.units) {
        AIUpdateInterface* ai = o->getAIUpdateInterface();
        row("unit " + num(o->getID()) + " " + num(ai->getIgnoredObstacleID()) + " " +
            num(ai->isAircraftThatAdjustsDestination()) + " " + num(o->getCrusherLevel()) + " " +
            num(o->getCrushableLevel()) + " " + f32(o->getGeometryInfo().getBoundingCircleRadius()) + " " +
            num(o->getLayer()) + " " + num(o->isKindOf(KINDOF_INFANTRY) != 0) + " " +
            num(o->getControllingPlayer()->getPlayerType() != PLAYER_COMPUTER) + " " +
            num(o->isKindOf(KINDOF_DOZER) != 0) + " " + num(o->isKindOf(KINDOF_VEHICLE) != 0));
    }
    for (Object* a : s.units) {
        std::string r = "rel " + num(a->getID());
        for (Object* b : s.units) {
            r += " " + num(b->getID()) + ":" + num(a->getRelationship(b) == ALLIES) +
                 num(a->canCrushOrSquish(b, TEST_CRUSH_OR_SQUISH));
        }
        row(r);
        if (!a->isKindOf(KINDOF_DOZER)) continue;
        std::string f = "friendly " + num(a->getID());
        for (Object* b : s.everything) {
            if (a->getRelationship(b) != ENEMIES) f += " " + num(b->getID());
        }
        row(f);
    }
}

static ICoord2D cell_of(const Coord3D& p) {
    ICoord2D c;
    c.x = REAL_TO_INT_FLOOR(p.x / 10.0f);
    c.y = REAL_TO_INT_FLOOR(p.y / 10.0f);
    return c;
}

static void occupy(const Scene& s, int clusters) {
    for (int n = clusters; n > 0; --n) {
        const PathfindLayerEnum layer = !s.walls.empty() && chance(25) ? LAYER_WALL : LAYER_GROUND;
        Coord3D p = point(s, false);
        if (layer == LAYER_WALL) p = *pick(s.walls)->getPosition();
        ICoord2D c = cell_of(p);
        const int span = 1 + below(3);
        Object* u = pick(s.units);
        const int kind = below(7);  // 0 goal, 1 and 4 to 6 pos, 2 both, 3 aircraft goal
        for (int i = 0; i < span; ++i) {
            for (int j = 0; j < span; ++j) {
                PathfindCell* cell = pf()->getCell(layer, c.x + i, c.y + j);
                if (!cell) continue;
                ICoord2D at;
                at.x = c.x + i;
                at.y = c.y + j;
                const std::string where = num(layer) + " " + num(at.x) + " " + num(at.y) + " " + num(u->getID());
                if (kind == 0 || kind == 2) {
                    cell->setGoalUnit(u->getID(), at);
                    row("goal " + where);
                }
                if (kind == 1 || kind == 2 || kind >= 4) {
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

static void connect(const Scene& s) {
    if (s.walls.empty()) return;
    for (int n = 0; n < 10; ++n) {
        const ICoord2D c = cell_of(*pick(s.walls)->getPosition());
        const int x = c.x - 3 + below(7), y = c.y - 2 + below(5);
        PathfindCell* top = pf()->getCell(LAYER_WALL, x, y);
        PathfindCell* ground = pf()->getCell(LAYER_GROUND, x, y);
        if (!top || !ground || top == ground) continue;
        ground->setConnectLayer(LAYER_WALL);
        top->setConnectLayer(LAYER_GROUND);
        row("connect " + num(x) + " " + num(y));
    }
}

static void begin_frame(Scene& s, UnsignedInt frame) {
    const Region3D full = g_terrain->extent;
    Region3D& e = g_terrain->extent;
    if (chance(60)) {
        e.lo.x = 10.0f * below(4);
        e.lo.y = 10.0f * below(4);
        e.hi.x = full.hi.x - 10.0f * below(4);
        e.hi.y = full.hi.y - 10.0f * below(4);
    }
    g_world->set_frame(frame);
    // The first call may only recalculate the zones; the second sets the
    // logical extent.
    for (int n = 0; n < 2; ++n) {
        row("begin " + c3(e.lo) + " " + c3(e.hi) + " " + num(frame));
        pf()->processPathfindQueue();
    }
    e = full;
}

// ---- calls ----------------------------------------------------------------------

static void passable(const Scene& s) {
    const int mode = below(10);
    if (mode < 5) {
        zones().setAllPassable();
        row("allpass");
    } else if (mode < 7) {
        zones().clearPassableFlags();
        std::string r = "somepass";
        for (int n = 2 + below(8); n > 0; --n) {
            const int x = below(s.w), y = below(s.h);
            zones().setPassable(x, y, true);
            r += " " + num(x) + ":" + num(y);
        }
        row(r);
    }
    // Otherwise whatever the last search left.
}

struct Mover {
    Object* unit = nullptr;
    LocomotorSet set;
    int surfaces = 1;
    bool downhill = false, through = false;
    PathfindLayerEnum layer = LAYER_GROUND, was = LAYER_GROUND;
    ObjectID ignore = INVALID_ID;
};

static const int kMasks[] = {1, 1, 1, 1, 2, 3, 4, 5, 16, 17, 23, 8};

static void choose(const Scene& s, Mover& m, const Coord3D& from) {
    m.unit = pick(s.units);
    m.surfaces = kMasks[below(12)];
    m.downhill = chance(7);
    m.through = chance(20);
    m.was = m.unit->getLayer();
    m.layer = !s.walls.empty() && from.z > 6.0f && chance(70) ? LAYER_WALL : LAYER_GROUND;
    m.ignore = chance(15) && !s.blocks.empty() ? pick(s.blocks)->getID() : INVALID_ID;
    m.set.*access_member(LsSurfaces()) = m.surfaces;
    m.set.*access_member(LsDownhill()) = m.downhill;
    m.unit->*access_member(ObjLayer()) = m.layer;
    m.unit->getAIUpdateInterface()->*access_member(AiThrough()) = m.through;
    pf()->*access_member(PfIgnore()) = m.ignore;
}

static std::string mover_text(const Mover& m) {
    return num(m.unit->getID()) + " " + num(m.surfaces) + " " + num(m.downhill) + " " + num(m.through) + " " +
           num(m.layer) + " " + num(m.ignore);
}

static void done(Mover& m) {
    m.unit->*access_member(ObjLayer()) = m.was;
    pf()->*access_member(PfIgnore()) = INVALID_ID;
}

static void endpoints(const Scene& s, Coord3D& from, Coord3D& to) {
    from = point(s);
    to = point(s);
    if (chance(35)) {
        const float reach = chance(50) ? 25.0f : 80.0f;
        to = from;
        to.x += between(-reach, reach);
        to.y += between(-reach, reach);
    }
    if (chance(5)) to = from;
    if (chance(2)) { to.x = 0.0f; to.y = 0.0f; }
}

static bool g_full = false, g_first_scene = false;
static int g_found = 0, g_missed = 0, g_costed = 0, g_uncosted = 0;

static const Coord3D* g_start = nullptr;

static void find_row(Scene& s) {
    Coord3D from, to;
    endpoints(s, from, to);
    if (g_start) from = *g_start;
    passable(s);
    Mover m;
    choose(s, m, from);
    Path* path = (pf()->*access_member(PfInternal()))(m.unit, m.set, &from, &to);
    (path ? g_found : g_missed)++;
    row("find " + mover_text(m) + " " + c3(from) + " " + c3(to) + " = " + path_dump(path) + " | " + state(s));
    if (path) deleteInstance(path);
    done(m);
    if (g_full) full_state(s);
}

static void cost_row(Scene& s) {
    Coord3D from, to;
    endpoints(s, from, to);
    Mover m;
    choose(s, m, from);
    const bool tunneling = chance(25);
    pf()->*access_member(PfTunnel()) = tunneling;
    const Int cost = (pf()->*access_member(PfCost()))(m.unit, m.set, &from, &to);
    (cost != 0x7fff0000 ? g_costed : g_uncosted)++;
    row("cost " + mover_text(m) + " " + c3(from) + " " + c3(to) + " " + num(tunneling) + " = " + num(cost) + " | " +
        state(s));
    pf()->*access_member(PfTunnel()) = false;
    done(m);
    if (g_full) full_state(s);
}

static std::string list_dump(const Scene& s) {
    std::string out;
    PathfindCell* c = (pf()->*access_member(PfOpen())).getHead();
    for (int n = 0; c && n < 4000; ++n) {
        out += " " + num(key(s, c)) + ":" + num(c->getTotalCost()) + ":" + num(c->getCostSoFar());
        c = c->getNextOpen();
    }
    return out;
}

// examineNeighboringCells on its own, also as the attack search calls it.
static void examine_row(Scene& s) {
    const Coord3D a = point(s, false), b = point(s, false);
    Mover m;
    choose(s, m, a);
    passable(s);
    const ICoord2D at = cell_of(a), goal_at = cell_of(b);
    PathfindCell* parent = pf()->getCell(m.layer, at.x, at.y);
    PathfindCell* goal = pf()->getCell(LAYER_GROUND, goal_at.x, goal_at.y);
    if (!parent || !goal || parent == goal) {
        done(m);
        return;
    }
    goal->allocateInfo(goal_at);
    parent->allocateInfo(at);
    const bool tunnel = chance(25), human = chance(50), center = chance(50), none = chance(10);
    const int radius = below(3);
    const int attack = chance(50) ? 0 : below(400);
    const ICoord2D start = cell_of(chance(60) ? a : point(s, false));
    parent->startPathfind(goal);
    if (s_useFixedPathfinding) {
        (pf()->*access_member(PfOpen())).reset();
        parent->putOnSortedOpenList(pf()->*access_member(PfOpen()));
    } else {
        (pf()->*access_member(PfOpen())).reset(parent);
    }
    (pf()->*access_member(PfClosed())).reset();
    parent->removeFromOpenList(pf()->*access_member(PfOpen()));
    parent->putOnClosedList(pf()->*access_member(PfClosed()));
    pf()->*access_member(PfTunnel()) = tunnel;
    const Int count = (pf()->*access_member(PfExamine()))(parent, none ? nullptr : goal, m.set, human, center,
                                                          radius, start, m.unit, attack);
    const std::string opened = list_dump(s);
    const bool tunnel_after = pf()->*access_member(PfTunnel());
    pf()->*access_member(PfTunnel()) = false;
    pf()->cleanOpenAndClosedLists();
    parent->releaseInfo();
    goal->releaseInfo();
    row("examine " + mover_text(m) + " " + num(at.x) + " " + num(at.y) + " " + num(goal_at.x) + " " +
        num(goal_at.y) + " " + num(tunnel) + " " + num(human) + " " + num(center) + " " + num(none) + " " +
        num(radius) + " " + num(attack) + " " + num(start.x) + " " + num(start.y) + " = " + num(count) + " " +
        num(tunnel_after) + opened + " | " + state(s));
    done(m);
}

static void clip_rows(const Scene& s) {
    for (int n = 0; n < 40; ++n) {
        Coord3D a, b;
        auto one = [&](Coord3D& p) {
            p.x = coord(-s.maxx, 2.0f * s.maxx);
            p.y = coord(-s.maxy, 2.0f * s.maxy);
            p.z = 1.0f;
            if (chance(30)) p.x = coord(0.0f, s.maxx);
            if (chance(30)) p.y = coord(0.0f, s.maxy);
        };
        one(a);
        one(b);
        if (chance(10)) b.y = a.y;
        if (chance(10)) b.x = a.x;
        const std::string in = c3(a) + " " + c3(b);
        pf()->clip(&a, &b);
        row("clip " + in + " = " + c3(a) + " " + c3(b));
    }
}

static void adjust_row(Scene& s) {
    Coord3D dest = point(s, false);
    const Coord3D group = point(s, false);
    Mover m;
    choose(s, m, dest);
    const std::string in = mover_text(m) + " " + c3(m.unit->getPosition() ? *m.unit->getPosition() : dest) + " " +
                           c3(dest) + " " + c3(group);
    const Bool ok = pf()->adjustDestination(m.unit, m.set, &dest, &group);
    row("adjust " + in + " = " + num(ok) + " " + c3(dest) + " | " + state(s));
    done(m);
}

// A parent chain that ends in a cell without a record: prependCells takes the
// crash exit, switches to the fixed code and cleans every cell.
static void chain_row(Scene& s) {
    // Under the fixed code the same chain is undefined behaviour.
    if (s_useFixedPathfinding) return;
    std::vector<PathfindCell*> cells;
    std::vector<ICoord2D> at;
    int x = 1 + below(s.w - 8), y = 1 + below(s.h - 8);
    const int length = 2 + below(5);
    for (int i = 0; i <= length; ++i) {
        PathfindCell* c = pf()->getCell(LAYER_GROUND, x, y);
        if (!c || c->hasInfo()) return;  // an occupied cell would keep its record
        if (c->getType() == PathfindCell::CELL_OBSTACLE) return;
        ICoord2D p;
        p.x = x;
        p.y = y;
        cells.push_back(c);
        at.push_back(p);
        x += 1;
        y += below(2);
    }
    std::string r = "chain";
    for (size_t i = 0; i < cells.size(); ++i) {
        cells[i]->allocateInfo(at[i]);
        r += " " + num(at[i].x) + ":" + num(at[i].y);
    }
    for (size_t i = 0; i + 1 < cells.size(); ++i) cells[i]->setParentCell(cells[i + 1]);
    cells.back()->releaseInfo();
    const Coord3D from = point(s, false);
    const bool center = chance(50);
    Path* path = newInstance(Path);
    (pf()->*access_member(PfPrepend()))(path, &from, cells.front(), center);
    row(r + " " + c3(from) + " " + num(center) + " = " + path_dump(path) + " | " + state(s));
    deleteInstance(path);
    if (g_first_scene) full_state(s);
}

// A list whose head has no record: releasing it switches to the fixed code.
static void lists_row(Scene& s, bool open) {
    PathfindCell* bare = nullptr;
    PathfindCell* held = nullptr;
    ICoord2D bare_at, held_at;
    for (int n = 0; n < 200 && (!bare || !held); ++n) {
        ICoord2D p;
        p.x = below(s.w);
        p.y = below(s.h);
        PathfindCell* c = pf()->getCell(LAYER_GROUND, p.x, p.y);
        if (!c || c->hasInfo()) continue;
        if (!bare) {
            bare = c;
            bare_at = p;
        } else if (c != bare) {
            held = c;
            held_at = p;
        }
    }
    if (!bare || !held) return;
    held->allocateInfo(held_at);
    PathfindCellList& other = open ? pf()->*access_member(PfClosed()) : pf()->*access_member(PfOpen());
    PathfindCellList& broken = open ? pf()->*access_member(PfOpen()) : pf()->*access_member(PfClosed());
    other.reset();
    if (open) held->putOnClosedList(other);
    else held->putOnSortedOpenList(other);
    broken.reset(bare);
    pf()->cleanOpenAndClosedLists();
    held->releaseInfo();
    row("lists " + num(open) + " " + num(bare_at.x) + " " + num(bare_at.y) + " " + num(held_at.x) + " " +
        num(held_at.y) + " = " + state(s));
}

// More equal costs than the retail insertion walks over.
static void cap_row(Scene& s) {
    PathfindCellList list;
    std::vector<PathfindCell*> cells;
    std::vector<ICoord2D> at;
    for (int x = 0; x < s.w && cells.size() < 5100; ++x) {
        for (int y = 0; y < s.h && cells.size() < 5100; ++y) {
            PathfindCell* c = pf()->getCell(LAYER_GROUND, x, y);
            if (!c || c->hasInfo()) continue;
            ICoord2D p;
            p.x = x;
            p.y = y;
            cells.push_back(c);
            at.push_back(p);
        }
    }
    for (size_t i = 0; i < cells.size(); ++i) {
        cells[i]->allocateInfo(at[i]);
        cells[i]->setTotalCost(i < 3 ? 1 : 7);
        cells[i]->putOnSortedOpenList(list);
    }
    Fnv f;
    int count = 0;
    for (PathfindCell* c = list.getHead(); c; c = c->getNextOpen(), ++count) f.add(key(s, c));
    PathfindCell::releaseOpenList(list);
    row("cap = " + num(static_cast<long long>(cells.size())) + " " + num(count) + " " + hex64(f.h) + " | " +
        state(s));
}

// Cells without records or units, for the cases built by hand.
static bool bare_cells(const Scene& s, int count, std::vector<PathfindCell*>& cells, std::vector<ICoord2D>& at) {
    for (int n = 0; n < 300 && static_cast<int>(cells.size()) < count; ++n) {
        ICoord2D p;
        p.x = 2 + below(s.w - 4);
        p.y = 2 + below(s.h - 4);
        PathfindCell* c = pf()->getCell(LAYER_GROUND, p.x, p.y);
        if (!c || c->hasInfo() || c->getType() == PathfindCell::CELL_OBSTACLE) continue;
        bool taken = false;
        for (PathfindCell* other : cells) taken = taken || other == c;
        if (taken) continue;
        cells.push_back(c);
        at.push_back(p);
    }
    return static_cast<int>(cells.size()) == count;
}

static std::string at_text(const std::vector<ICoord2D>& at) {
    std::string out;
    for (const ICoord2D& p : at) out += " " + num(p.x) + " " + num(p.y);
    return out;
}

// costSoFar when the parent's parent link has outlived its record: the cell it
// names has none, or the record now belongs to another cell.
static void grand_row(Scene& s) {
    if (s_useFixedPathfinding) return;
    std::vector<PathfindCell*> cells;
    std::vector<ICoord2D> at;
    if (!bare_cells(s, 4, cells, at)) return;
    PathfindCell *grand = cells[0], *parent = cells[1], *cell = cells[2], *taker = cells[3];
    grand->allocateInfo(at[0]);
    parent->allocateInfo(at[1]);
    cell->allocateInfo(at[2]);
    parent->setParentCellHierarchical(grand);
    const int cost = 200 + below(100);
    parent->setCostSoFar(cost);
    const UnsignedInt held = cell->costSoFar(parent);
    grand->releaseInfo();
    const UnsignedInt released = cell->costSoFar(parent);
    // The free list is last in, first out: the new record is the released one.
    taker->allocateInfo(at[3]);
    const UnsignedInt reused = cell->costSoFar(parent);
    const std::string digests = state(s);
    parent->clearParentCell();
    taker->releaseInfo();
    cell->releaseInfo();
    parent->releaseInfo();
    row("grand" + at_text(at) + " " + num(cost) + " = " + num(held) + " " + num(released) + " " + num(reused) + " " + digests);
}

// The retail insertion sort cuts a link to a cell that has lost its record.
static void cut_row(Scene& s) {
    if (s_useFixedPathfinding) return;
    std::vector<PathfindCell*> cells;
    std::vector<ICoord2D> at;
    if (!bare_cells(s, 4, cells, at)) return;
    PathfindCellList list;
    PathfindCell *first = cells[0], *second = cells[1], *gone = cells[2], *added = cells[3];
    for (int i = 0; i < 4; ++i) cells[i]->allocateInfo(at[i]);
    first->setTotalCost(10);
    second->setTotalCost(20);
    added->setTotalCost(30);
    first->putOnSortedOpenList(list);
    second->putOnSortedOpenList(list);
    InfoView* stale = reinterpret_cast<InfoView*>(gone->*access_member(CellInfo()));
    gone->releaseInfo();
    reinterpret_cast<InfoView*>(second->*access_member(CellInfo()))->nextOpen = stale;
    added->putOnSortedOpenList(list);
    std::string out;
    int n = 0;
    for (PathfindCell* c = list.getHead(); c && n < 8; c = c->getNextOpen(), ++n) out += " " + num(key(s, c));
    const std::string digests = state(s);
    PathfindCell::releaseOpenList(list);
    gone->releaseInfo();
    row("cut" + at_text(at) + " =" + out + " | " + digests);
}

// ---- hierarchical, ground and findPath ---------------------------------------------

static bool g_decks_up = false;
// A bridge from one bank to the other, along x or along y.
static void add_bridge(Scene& s, bool along_x, float lo, float hi, float across) {
    BridgeInfo info;
    const float deck_z = chance(75) || g_decks_up ? 5.0f : 0.0f;
    const float half = chance(50) ? 15.0f : between(8.0f, 26.0f);
    const float skew = chance(20) ? between(-30.0f, 30.0f) : 0.0f;
    info.from.x = along_x ? lo : across;
    info.from.y = along_x ? across : lo;
    info.from.z = deck_z;
    info.to.x = along_x ? hi : across + skew;
    info.to.y = along_x ? across + skew : hi;
    info.to.z = deck_z;
    float dx = info.to.x - info.from.x, dy = info.to.y - info.from.y;
    const float d = std::sqrt(dx * dx + dy * dy);
    const float px = -dy / d * half, py = dx / d * half;
    info.bridgeWidth = 2.0f * half;
    info.fromLeft.x = info.from.x + px; info.fromLeft.y = info.from.y + py; info.fromLeft.z = deck_z;
    info.fromRight.x = info.from.x - px; info.fromRight.y = info.from.y - py; info.fromRight.z = deck_z;
    info.toLeft.x = info.to.x + px; info.toLeft.y = info.to.y + py; info.toLeft.z = deck_z;
    info.toRight.x = info.to.x - px; info.toRight.y = info.to.y - py; info.toRight.z = deck_z;
    Object* deck = spawn("S5Deck", g_world->team_a, info.from.x, info.from.y, deck_z, 0.0f);
    s.everything.push_back(deck);
    s.bridge_points.push_back(info.from);
    s.bridge_points.push_back(info.to);
    info.bridgeObjectID = deck->getID();
    for (int k = 0; k < BRIDGE_MAX_TOWERS; ++k) info.towerObjectID[k] = INVALID_ID;
    info.bridgeIndex = 0;
    info.curDamageState = BODY_PRISTINE;
    info.damageStateChanged = false;
    Dict props;
    TheTerrainLogic->addBridgeToLogic(&info, &props, AsciiString("S5NoRoad"));
    s.bridges.push_back("bridge " + num(deck->getID()) + " " + c3(info.from) + " " + c3(info.to) + " " +
                        c3(info.fromLeft) + " " + c3(info.fromRight) + " " + c3(info.toLeft) + " " +
                        c3(info.toRight) + " = " + num(TheTerrainLogic->getFirstBridge()->getLayer()));
}

// Where a search starts or ends: anywhere, or by a bridge.
static Coord3D far_point(const Scene& s) {
    Coord3D p = point(s, chance(5));
    // Mostly on open ground, so that searches have something to find.
    for (int n = 0; n < 20 && chance(85); ++n) {
        const PathfindCell* c = pf()->getCell(LAYER_GROUND, &p);
        if (c && c->getType() == PathfindCell::CELL_CLEAR && !c->getPinched()) break;
        p = point(s, false);
    }
    if (!s.bridge_points.empty() && chance(30)) {
        p = s.bridge_points[below(static_cast<int>(s.bridge_points.size()))];
        p.x += between(-12.0f, 12.0f);
        p.y += between(-12.0f, 12.0f);
    } else if (chance(10)) {
        p.z = 5.0f;
    }
    return p;
}

static void far_endpoints(const Scene& s, Coord3D& from, Coord3D& to) {
    from = far_point(s);
    to = far_point(s);
    if (chance(35)) {  // across the whole map
        from.x = coord(0.0f, s.maxx * 0.2f);
        to.x = coord(s.maxx * 0.8f, s.maxx);
    }
    if (chance(3)) to = from;
    if (chance(2)) { to.x = 0.0f; to.y = 0.0f; }
}

static int g_hier = 0, g_hier_null = 0, g_paths = 0, g_paths_null = 0, g_ground = 0, g_ground_null = 0;

static void hier_row(Scene& s) {
    Coord3D from, to;
    far_endpoints(s, from, to);
    Mover m;
    choose(s, m, from);
    const bool human = chance(50), crusher = chance(20), closest = chance(35);
    Path* path = (pf()->*(closest ? access_member(PfClosest()) : access_member(PfHier())))(human, m.set, &from, &to,
                                                                                          crusher);
    (path ? g_hier : g_hier_null)++;
    row("hier " + mover_text(m) + " " + num(human) + " " + num(crusher) + " " + num(closest) + " " + c3(from) + " " +
        c3(to) + " = " + path_dump(path) + " | " + state(s));
    if (path) deleteInstance(path);
    done(m);
    if (g_full) full_state(s);
}

static void path_row(Scene& s) {
    Coord3D from, to;
    far_endpoints(s, from, to);
    Mover m;
    choose(s, m, from);
    Path* path = services()->findPath(m.unit, m.set, &from, &to);
    (path ? g_paths : g_paths_null)++;
    row("path " + mover_text(m) + " " + c3(from) + " " + c3(to) + " = " + path_dump(path) + " | " + state(s));
    if (path) deleteInstance(path);
    done(m);
}

static void exists_row(Scene& s) {
    Coord3D from, to;
    far_endpoints(s, from, to);
    Mover m;
    choose(s, m, from);
    m.surfaces = m.unit->getAIUpdateInterface()->getLocomotorSet().getValidSurfaces();
    m.downhill = m.unit->getAIUpdateInterface()->getLocomotorSet().isDownhillOnly();
    const ObjectID ignore = !s.blocks.empty() && chance(60) ? pick(s.blocks)->getID() : INVALID_ID;
    if (ignore != INVALID_ID && chance(70)) to = *TheGameLogic->findObjectByID(ignore)->getPosition();
    const std::string in = mover_text(m);
    const Bool found = pf()->slowDoesPathExist(m.unit, &from, &to, ignore);
    row("exists " + in + " " + c3(from) + " " + c3(to) + " " + num(ignore) + " = " + num(found) + " " +
        num(pf()->*access_member(PfIgnore())) + " | " + state(s));
    done(m);
}

static void ground_row(Scene& s) {
    Coord3D from, to;
    far_endpoints(s, from, to);
    if (!s.blocks.empty() && chance(40)) {  // a goal too narrow for the group
        to = *pick(s.blocks)->getPosition();
        to.x += between(-45.0f, 45.0f);
        to.y += between(-45.0f, 45.0f);
        to.z = 0.0f;
    }
    const int diameter = chance(15) ? below(12) : 1 + below(5);
    const bool crusher = chance(30);
    Path* path = pf()->findGroundPath(&from, &to, diameter, crusher);
    (path ? g_ground : g_ground_null)++;
    row("ground " + num(diameter) + " " + num(crusher) + " " + c3(from) + " " + c3(to) + " = " + path_dump(path) +
        " | " + state(s));
    if (path) deleteInstance(path);
}

// ---- the request queue ----------------------------------------------------------

struct Plan {
    Coord3D from, to;
    ObjectID again = INVALID_ID;  // queued once while it is served
};
static std::map<ObjectID, Plan> g_plans;
static Scene* g_scene = nullptr;

void AIUpdateInterface::doPathfind(PathfindServicesInterface*) {
    const ObjectID id = getObject()->getID();
    Plan& plan = g_plans[id];
    const Int before = pf()->*access_member(PfCumulative());
    Path* path = services()->findPath(getObject(), getLocomotorSet(), &plan.from, &plan.to);
    std::string queued = "-";
    if (plan.again != INVALID_ID) {
        queued = num(pf()->queueForPath(plan.again));
        plan.again = INVALID_ID;
    }
    row("serve " + num(id) + " = " + num(before) + " " + path_dump(path) + " " + queued + " " +
        num(pf()->*access_member(PfCumulative())) + " " + num(oracle_wp51::World::snapshot_crc(pf())));
    if (path) deleteInstance(path);
}

static void queue_row(ObjectID id) {
    const Bool ok = pf()->queueForPath(id);
    row("queue " + num(id) + " = " + num(ok) + " " + num(oracle_wp51::World::snapshot_crc(pf())));
}

static void frame_row(Scene& s, UnsignedInt frame) {
    g_world->set_frame(frame);
    const Region3D& e = g_terrain->extent;
    row("frame " + c3(e.lo) + " " + c3(e.hi) + " " + num(frame));
    pf()->processPathfindQueue();
    row("frameend = " + state(s) + " " + num(oracle_wp51::World::snapshot_crc(pf())));
}

static void queue_section(Scene& s, UnsignedInt frame) {
    g_plans.clear();
    // An AI keeps its own locomotor set and flags; clear what the searches poked.
    for (Object* o : s.units) o->getAIUpdateInterface()->*access_member(AiThrough()) = false;
    std::vector<ObjectID> ids;
    for (Object* o : s.units) {
        Plan& plan = g_plans[o->getID()];
        far_endpoints(s, plan.from, plan.to);
        if (chance(80)) {  // long searches, to spend a frame's cells
            plan.from.x = coord(0.0f, s.maxx * 0.2f);
            plan.to.x = coord(s.maxx * 0.8f, s.maxx);
        }
        if (chance(30)) plan.again = chance(40) ? o->getID() : pick(s.units)->getID();
        row("plan " + num(o->getID()) + " " + c3(plan.from) + " " + c3(plan.to) + " " + num(plan.again) + " " +
            num(o->getAIUpdateInterface()->getLocomotorSet().getValidSurfaces()));
        ids.push_back(o->getID());
    }
    for (ObjectID id : ids) queue_row(id);
    for (int n = 30 + below(30); n > 0; --n) {
        const int kind = below(10);
        if (kind == 0) queue_row(oid(900000 + below(5)));  // no such object
        else if (kind == 1 && !s.blocks.empty()) queue_row(pick(s.blocks)->getID());  // no AI
        else queue_row(ids[below(static_cast<int>(ids.size()))]);
    }
    for (int n = 0; n < 8; ++n) {
        frame_row(s, frame + n);
        for (int k = below(5); k > 0; --k) queue_row(ids[below(static_cast<int>(ids.size()))]);
    }
}

// More requests than the ring holds.
static void overflow_section(Scene& s, UnsignedInt frame) {
    for (int n = 0; n < 530; ++n) {
        const Bool ok = pf()->queueForPath(oid(700000 + n));
        if (n % 40 == 0 || !ok || n > 505) {
            row("queue " + num(700000 + n) + " = " + num(ok) + " " + num(oracle_wp51::World::snapshot_crc(pf())));
        } else {
            row("fill " + num(700000 + n));
        }
    }
    queue_row(oid(700000));
    frame_row(s, frame);
    for (int n = 0; n < 600; ++n) {
        const Bool ok = pf()->queueForPath(oid(800000 + n));
        if (n > 505) row("queue " + num(800000 + n) + " = " + num(ok) + " " + num(oracle_wp51::World::snapshot_crc(pf())));
        else row("fill " + num(800000 + n));
    }
    frame_row(s, frame + 1);
}

// Goals along the edges of the zone blocks, from where the hierarchical pass
// steps: it only expands cells that already have a record.
static void occupy_edges(const Scene& s, int percent) {
    for (int x = 0; x < s.w; ++x) {
        for (int y = 0; y < s.h; ++y) {
            const int mx = x % 10, my = y % 10;
            const bool edge = ((mx == 0 || mx == 9) && my > 1 && my < 9) || ((my == 0 || my == 9) && mx > 1 && mx < 9);
            if (!edge || !chance(percent)) continue;
            PathfindCell* cell = pf()->getCell(LAYER_GROUND, x, y);
            Object* u = pick(s.units);
            ICoord2D at;
            at.x = x;
            at.y = y;
            cell->setGoalUnit(u->getID(), at);
            row("goal " + num(LAYER_GROUND) + " " + num(x) + " " + num(y) + " " + num(u->getID()));
        }
    }
}

// ---- driver ---------------------------------------------------------------------

static void run_scene(int index) {
    Scene s;
    std::fprintf(stderr, "scene %d at %zu rows\n", index, g_rows.size());
    row("scenario " + num(index));
    const bool large = index >= 10;
    s.w = large ? 78 + below(8) : 16 + below(34);
    s.h = large ? 74 + below(8) : 16 + below(26);
    s.maxx = s.w * 10.0f;
    s.maxy = s.h * 10.0f;
    S5Terrain& t = *g_terrain;
    t.w = s.w;
    t.h = s.h;
    t.cliff.assign(static_cast<size_t>((s.w + 2) * (s.h + 2)), '0');
    t.water = t.cliff;
    t.height = t.cliff;
    const int rough = index % 4;
    if (rough != 0) {
        blobs(t.cliff, '1', below(4) + (large ? 8 : 0), 6);
        blobs(t.water, '1', below(4) + (large ? 6 : 0), 6);
    }
    if (rough >= 2) {
        for (int v = 1; v <= 5; ++v) blobs(t.height, static_cast<char>('0' + v), 2 + below(6), 8);
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
    row("terrain " + num(s.w) + " " + num(s.h) + " " + t.cliff + " " + t.water + " " + t.height);
    g_world->set_frame(100);
    // An AI picks its locomotor from the map as it is constructed.
    pf()->newMap();

    const int blocks = index % 4 == 0 ? below(4) : 3 + below(large ? 40 : 12);
    for (int n = blocks; n > 0; --n) {
        const std::string name = g_blocks[below(static_cast<int>(g_blocks.size()))];
        s.blocks.push_back(spawn(name, chance(50) ? g_world->team_a : g_world->team_b, coord(0.0f, s.maxx),
                                 coord(0.0f, s.maxy), 0.0f, chance(50) ? 0.0f : between(-3.2f, 3.2f)));
    }
    if (index % 3 != 0) {
        float x = coord(30.0f, s.maxx * 0.4f);
        const float y = coord(30.0f, s.maxy - 30.0f);
        for (int n = 2 + below(4); n > 0; --n) {
            s.walls.push_back(spawn("S5Wall", g_world->team_a, x, y, 0.0f, 0.0f));
            x += 48.0f;
        }
    }
    for (int n = 6 + below(6); n > 0; --n) {
        const std::string name = g_units[below(static_cast<int>(g_units.size()))];
        s.units.push_back(spawn(name, chance(55) ? g_world->team_a : g_world->team_b, coord(0.0f, s.maxx),
                                coord(0.0f, s.maxy), 0.0f, 0.0f));
    }
    for (Object* o : s.units) {
        if (!s.blocks.empty() && chance(25)) o->getAIUpdateInterface()->ignoreObstacle(pick(s.blocks));
    }
    s.everything = s.blocks;
    s.everything.insert(s.everything.end(), s.walls.begin(), s.walls.end());
    s.everything.insert(s.everything.end(), s.units.begin(), s.units.end());
    // `reset` forgets the wall pieces, so it comes before they are added.
    pf()->reset();
    // The records outlive the map (the pathfinder allocates them once); start
    // each scene from the same free list.
    PathfindCellInfo::allocateCellInfos();
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
    index_cells(s);
    unit_rows(s);
    occupy(s, index % 5 == 0 ? 4 : 20 + below(large ? 120 : 50));
    connect(s);
    g_full = index == 0;
    g_first_scene = index == 0;
    if (g_full) full_state(s);
    row("setup = " + state(s));
    for (int n = 0; n < 20 && !s.walls.empty(); ++n) {  // from the top of the wall
        Coord3D top = *pick(s.walls)->getPosition();
        top.x += between(-20.0f, 20.0f);
        top.y += between(-10.0f, 10.0f);
        top.z = 12.0f;
        g_start = &top;
        find_row(s);
        g_start = nullptr;
    }
    begin_frame(s, 1000);
    row("setup = " + state(s));

    clip_rows(s);
    const int searches = large ? 12 : 26;
    for (int n = 0; n < searches; ++n) {
        if (n == 8) g_full = false;
        find_row(s);
        cost_row(s);
        if (n % 2 == 0) examine_row(s);
    }
    for (int n = 0; n < 4; ++n) adjust_row(s);
    for (int n = 0; n < 3; ++n) grand_row(s);
    cut_row(s);
    if (large) cap_row(s);
    // The switch to the fixed code, by each way in, then the same calls again.
    if (index % 2 == 0) {
        if (index % 6 == 0) {
            for (int n = 0; n < 30 && !s_useFixedPathfinding; ++n) chain_row(s);
        } else {
            lists_row(s, index % 6 == 2);
        }
        for (int n = 0; n < searches; ++n) {
            find_row(s);
            cost_row(s);
            if (n % 2 == 0) examine_row(s);
        }
        for (int n = 0; n < 4; ++n) adjust_row(s);
        if (large) cap_row(s);
    } else {
        for (int n = 0; n < 30 && !s_useFixedPathfinding; ++n) chain_row(s);
    }

    for (Object* o : s.everything) TheGameLogic->destroyObject(o);
    TheGameLogic->processDestroyList();
    ThePartitionManager->update();
    TheTerrainLogic->reset();
    pf()->reset();
    row("end");
}

// A larger map cut by water, with bridges, for the zone blocks.
static void run_far_scene(int index) {
    Scene s;
    std::fprintf(stderr, "scene %d at %zu rows\n", index, g_rows.size());
    row("scenario " + num(index));
    const bool big = index % 5 == 0;
    // One large map with little in it, where a frame's searches pass the budget.
    const bool open_field = index == 20;
    s.w = big ? 82 + below(10) : 36 + below(40);
    s.h = big ? 74 + below(10) : 24 + below(34);
    s.maxx = s.w * 10.0f;
    s.maxy = s.h * 10.0f;
    S5Terrain& t = *g_terrain;
    t.w = s.w;
    t.h = s.h;
    t.cliff.assign(static_cast<size_t>((s.w + 2) * (s.h + 2)), '0');
    t.water = t.cliff;
    t.height = t.cliff;
    // Strips of water or cliff from one side to the other, sometimes with a gap.
    struct Strip { bool across_x; int at, width; };
    std::vector<Strip> strips;
    if (index % 4 != 3 && !open_field) {
        const int count = 1 + below(2);
        for (int k = 0; k < count; ++k) {
            Strip strip;
            strip.across_x = k == 0 || chance(50);
            const int span = strip.across_x ? s.w : s.h;
            strip.width = 2 + below(4);
            strip.at = span / 4 + below(span / 2);
            strips.push_back(strip);
            std::string& table = chance(75) ? t.water : t.cliff;
            for (int i = 0; i < (strip.across_x ? s.h : s.w) + 2; ++i) {
                for (int d = 0; d < strip.width; ++d) {
                    const int x = strip.across_x ? strip.at + d : i;
                    const int y = strip.across_x ? i : strip.at + d;
                    table[t.at(x, y)] = '1';
                }
            }
            if (chance(20)) {  // a ford
                const int y = below((strip.across_x ? s.h : s.w) - 4);
                for (int i = y; i < y + 3; ++i)
                    for (int d = 0; d < strip.width; ++d)
                        table[t.at(strip.across_x ? strip.at + d : i, strip.across_x ? i : strip.at + d)] = '0';
            }
        }
    }
    blobs(t.cliff, '1', open_field ? 3 : below(5), 6);
    blobs(t.water, '1', open_field ? 0 : below(4), 6);
    if (open_field) {  // two barriers open at opposite ends, so searches spread
        for (int y = 0; y < s.h - 8; ++y) {
            t.cliff[t.at(s.w / 3, y)] = '1';
            t.cliff[t.at(2 * s.w / 3, s.h - 1 - y)] = '1';
        }
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
    row("terrain " + num(s.w) + " " + num(s.h) + " " + t.cliff + " " + t.water + " " + t.height);
    g_world->set_frame(100);
    pf()->newMap();

    for (int n = open_field ? 2 : below(big ? 30 : 14); n > 0; --n) {
        const std::string name = g_blocks[below(static_cast<int>(g_blocks.size()))];
        s.blocks.push_back(spawn(name, chance(50) ? g_world->team_a : g_world->team_b, coord(0.0f, s.maxx),
                                 coord(0.0f, s.maxy), 0.0f, chance(50) ? 0.0f : between(-3.2f, 3.2f)));
    }
    if (index % 6 == 1) {
        float x = coord(30.0f, s.maxx * 0.4f);
        const float y = coord(30.0f, s.maxy - 30.0f);
        for (int n = 2 + below(4); n > 0; --n) {
            s.walls.push_back(spawn("S5Wall", g_world->team_a, x, y, 0.0f, 0.0f));
            x += 48.0f;
        }
    }
    for (int n = big ? 20 + below(8) : 6 + below(6); n > 0; --n) {
        const std::string name = g_units[below(static_cast<int>(g_units.size()))];
        s.units.push_back(spawn(name, chance(55) ? g_world->team_a : g_world->team_b, coord(0.0f, s.maxx),
                                coord(0.0f, s.maxy), 0.0f, 0.0f));
    }
    for (Object* o : s.units) {
        if (!s.blocks.empty() && chance(25)) o->getAIUpdateInterface()->ignoreObstacle(pick(s.blocks));
    }
    pf()->reset();
    PathfindCellInfo::allocateCellInfos();
    for (Object* o : s.walls) {
        pf()->addWallPiece(o);
        row("wallpiece " + num(o->getID()));
    }
    for (const Strip& strip : strips) {
        const int length = strip.across_x ? s.h : s.w;
        for (int n = chance(10) ? 0 : 1 + below(3); n > 0; --n) {
            const float across = 10.0f * (2 + below(length - 4)) + (chance(50) ? 5.0f : 0.0f);
            const float lo = 10.0f * strip.at - 10.0f * (1 + below(2));
            const float hi = 10.0f * (strip.at + strip.width) + 10.0f * (1 + below(2));
            add_bridge(s, strip.across_x, lo, hi, across);
        }
    }
    s.everything.insert(s.everything.end(), s.blocks.begin(), s.blocks.end());
    s.everything.insert(s.everything.end(), s.walls.begin(), s.walls.end());
    s.everything.insert(s.everything.end(), s.units.begin(), s.units.end());
    for (const std::string& r : s.bridges) row(r);
    std::string order = "order";
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
        order += " " + num(o->getID());
        object_row(o);
    }
    row(order);
    ThePartitionManager->update();
    pf()->newMap();
    index_cells(s);
    unit_rows(s);
    occupy(s, index % 3 == 0 ? 6 : open_field ? 3 : 15 + below(big ? 20 : 40));
    if (index % 3 != 2) occupy_edges(s, index % 3 == 0 ? 90 : 55);
    g_full = index == 13;
    g_first_scene = false;
    if (g_full) full_state(s);
    row("setup = " + state(s));
    begin_frame(s, 1000);
    row("setup = " + state(s));

    const int searches = big ? 9 : 16;
    for (int n = 0; n < searches; ++n) {
        if (n == 6) g_full = false;
        hier_row(s);
        path_row(s);
        ground_row(s);
        if (n % 3 == 0) exists_row(s);
        if (n % 4 == 1) find_row(s);
    }
    // Occupied cells keep their records: the hierarchical pass steps from them.
    if (index % 3 == 1) {
        for (int n = 0; n < 30; ++n) chain_row(s);
        for (int n = 0; n < searches / 2; ++n) {
            hier_row(s);
            path_row(s);
            ground_row(s);
            if (n % 3 == 0) exists_row(s);
        }
    }
    queue_section(s, 2000);
    if (index == 13) overflow_section(s, 3000);

    for (Object* o : s.everything) TheGameLogic->destroyObject(o);
    TheGameLogic->processDestroyList();
    ThePartitionManager->update();
    TheTerrainLogic->reset();
    pf()->reset();
    row("end");
}

// ---- a pool that runs out -----------------------------------------------------

static int free_records() {
    int n = 0;
    for (InfoView* p = reinterpret_cast<InfoView*>(*access_member(InfoFree())); p && n <= INFO_COUNT;
         p = p->pathParent) {
        ++n;
    }
    return n;
}

static std::vector<PathfindCell*> g_hoard;

// Takes records for cells from the far side until `left` are free, or gives
// them all back. The columns around the first bridges stay without records.
static void hoard_row(const Scene& s, int left) {
    if (left < 0) {
        for (PathfindCell* c : g_hoard) c->releaseInfo();
        g_hoard.clear();
        row("unhoard = " + state(s));
        return;
    }
    for (int x = s.w - 1; x >= 0 && free_records() > left; --x) {
        if (x >= 36 && x < 50) continue;
        for (int y = s.h - 1; y >= 0 && free_records() > left; --y) {
            PathfindCell* c = pf()->getCell(LAYER_GROUND, x, y);
            if (!c || c->hasInfo()) continue;
            ICoord2D at;
            at.x = x;
            at.y = y;
            c->allocateInfo(at);
            g_hoard.push_back(c);
        }
    }
    row("hoard " + num(left) + " = " + num(free_records()) + " " + state(s));
}

// A map with more cells than there are records.
static void run_exhaust_scene(int index) {
    Scene s;
    std::fprintf(stderr, "scene %d at %zu rows\n", index, g_rows.size());
    row("scenario " + num(index));
    s.w = 196;
    s.h = 170;
    s.maxx = s.w * 10.0f;
    s.maxy = s.h * 10.0f;
    S5Terrain& t = *g_terrain;
    t.w = s.w;
    t.h = s.h;
    t.cliff.assign(static_cast<size_t>((s.w + 2) * (s.h + 2)), '0');
    t.water = t.cliff;
    t.height = t.cliff;
    // Two water strips: only bridges cross them.
    for (int y = 0; y < s.h + 2; ++y) {
        for (int d = 0; d < 3; ++d) {
            t.water[t.at(40 + d, y)] = '1';
            t.water[t.at(90 + d, y)] = '1';
        }
    }
    t.extent.lo.x = 0.0f;
    t.extent.lo.y = 0.0f;
    t.extent.lo.z = 0.0f;
    t.extent.hi.x = s.maxx;
    t.extent.hi.y = s.maxy;
    t.extent.hi.z = 100.0f;
    const_cast<TAiData*>(TheAI->getAiData())->m_wallHeight = 12.0f;
    pf()->reset();
    row("map " + f32(s.maxx) + " " + f32(s.maxy) + " " + f32(12.0f));
    row("terrain " + num(s.w) + " " + num(s.h) + " " + t.cliff + " " + t.water + " " + t.height);
    g_world->set_frame(100);
    pf()->newMap();
    for (int n = 0; n < 8; ++n) {
        const std::string name = g_units[below(static_cast<int>(g_units.size()))];
        s.units.push_back(spawn(name, chance(55) ? g_world->team_a : g_world->team_b, coord(0.0f, 350.0f),
                                coord(0.0f, s.maxy), 0.0f, 0.0f));
    }
    pf()->reset();
    PathfindCellInfo::allocateCellInfos();
    g_decks_up = true;
    for (int n = 0; n < 2; ++n) {
        const float across = 10.0f * (24 + 60 * n) + 5.0f;  // off the block edges
        add_bridge(s, true, 380.0f, 440.0f, across);
        add_bridge(s, true, 880.0f, 940.0f, across + 200.0f);
    }
    s.everything.insert(s.everything.end(), s.units.begin(), s.units.end());
    for (const std::string& r : s.bridges) row(r);
    std::string order = "order";
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
        order += " " + num(o->getID());
        object_row(o);
    }
    row(order);
    ThePartitionManager->update();
    pf()->newMap();
    index_cells(s);
    unit_rows(s);
    occupy(s, 10);
    occupy_edges(s, 60);
    g_full = false;
    begin_frame(s, 1000);
    row("setup = " + state(s));

    // Hierarchical searches starting on a bridge, with room for none, one or
    // both of its ends, in the retail layout and then fixed.
    auto on_bridge_row = [&](int left, bool closest, bool from_ground = false) {
        if (left >= 0) hoard_row(s, left);
        Coord3D from = s.bridge_points[0];
        from.x += from_ground ? -25.0f : 25.0f;
        if (from_ground) from.z = 0.0f;
        Coord3D to;
        to.x = 1500.0f + 100.0f * below(3);
        to.y = coord(100.0f, s.maxy - 100.0f);
        to.z = 0.0f;
        // A goal on open ground, or the zones differ and nothing is searched.
        for (int n = 0; n < 50; ++n) {
            const PathfindCell* c = pf()->getCell(LAYER_GROUND, &to);
            if (c && c->getType() == PathfindCell::CELL_CLEAR && !c->getPinched()) break;
            to.y = between(100.0f, s.maxy - 100.0f);
        }
        Mover m;
        choose(s, m, from);
        m.surfaces = 1;
        m.set.*access_member(LsSurfaces()) = 1;
        const bool human = false, crusher = false;
        Path* path = (pf()->*(closest ? access_member(PfClosest()) : access_member(PfHier())))(human, m.set, &from, &to,
                                                                                              crusher);
        row("hier " + mover_text(m) + " " + num(human) + " " + num(crusher) + " " + num(closest) + " " + c3(from) +
            " " + c3(to) + " = " + path_dump(path) + " | " + state(s));
        if (path) deleteInstance(path);
        done(m);
        if (left >= 0) hoard_row(s, -1);
    };
    for (int n = 0; n < 4; ++n) on_bridge_row(-1, n % 2);
    static const int kLeft[] = {0, 1, 2, 3, 4, 5, 6, 8, 12, 20, 40, 100, 300, 1000};
    auto at_each = [&]() {
        for (int left : kLeft) {
            hoard_row(s, left);
            find_row(s);
            cost_row(s);
            if (left <= 12) examine_row(s);
            hier_row(s);
            path_row(s);
            ground_row(s);
            hoard_row(s, -1);
        }
    };
    // Room runs out during the bridge crossing: this switches to the fixed code.
    for (int n = 0; n < 4; ++n) on_bridge_row(-1, n % 2, true);
    // Room runs out on the bridge's own start, then in a crossing from the
    // ground beside it; each switches to the fixed code, which is switched
    // back so that both are met in the retail layout.
    auto retail = [&]() {
        s_useFixedPathfinding = false;
        row("retail = " + state(s));
    };
    auto met = [&](const char* what) {
        if (!s_useFixedPathfinding) {
            std::fprintf(stderr, "%s did not run out of records\n", what);
            std::exit(1);
        }
        retail();
    };
    on_bridge_row(40, false);
    // With the goal's record already taken, the start takes one and the near
    // end none, then the near end takes one and the far end none.
    on_bridge_row(1, false);
    met("the near end of the start bridge");
    on_bridge_row(2, false);
    if (s_useFixedPathfinding) retail();
    for (int left : {3, 0}) on_bridge_row(left, false);
    retail();
    for (int left : {6, 3, 2, 1}) on_bridge_row(left, false, true);
    at_each();
    // Starts without a record, so taking one can fail.
    for (int left : {0, 1, 2, 3, 5}) {
        for (int n = 0; n < 3; ++n) {
            Coord3D start;
            start.x = between(365.0f, 395.0f);
            start.y = between(20.0f, s.maxy - 20.0f);
            start.z = 0.0f;
            hoard_row(s, left);
            g_start = &start;
            find_row(s);
            g_start = nullptr;
            hoard_row(s, -1);
        }
    }
    for (int left : {40, 6, 3, 2, 1}) on_bridge_row(left, false);
    for (int left : {6, 3, 2, 1}) on_bridge_row(left, false, true);
    for (int n = 0; n < 4; ++n) on_bridge_row(-1, n % 2);

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
    g_terrain = new S5Terrain;
    TheTerrainLogic = g_terrain;
    Player* computer = world.team_b->getControllingPlayer();
    if (!computer) {
        std::fprintf(stderr, "team B has no player\n");
        return 1;
    }
    computer->*access_member(PlayerKind()) = PLAYER_COMPUTER;
    Player* human = world.team_a->getControllingPlayer();
    if (!human || human == computer) {
        std::fprintf(stderr, "team A has no player of its own\n");
        return 1;
    }
    human->*access_member(PlayerKind()) = PLAYER_HUMAN;
    for (int index = 0; index < 12; ++index) run_scene(index);
    for (int index = 12; index < 22; ++index) run_far_scene(index);
    run_exhaust_scene(22);
    std::fprintf(stderr, "hierarchical %d/%d, paths %d/%d, ground %d/%d (found/not)\n", g_hier, g_hier_null,
                 g_paths, g_paths_null, g_ground, g_ground_null);
    std::fprintf(stderr, "paths %d found %d not, costs %d found %d not\n", g_found, g_missed, g_costed,
                 g_uncosted);

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
