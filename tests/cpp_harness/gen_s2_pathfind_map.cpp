// Golden vectors for slice S2, the pathfind map: classification, object
// footprints, fences, wall pieces, bridge layers, zones, Pathfinder::crc and the
// bridge queries of TerrainLogic, all in the unmodified reference library.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_s2_pathfind_map.cpp -o gen_s2
//   ./gen_s2 docs/golden_vectors/S2_PathfindMap.json
//
// Only the terrain is synthetic (cliff, water and height tables behind a
// TerrainLogic subclass). Each row is an input to replay or the state expected
// after it. A state row carries one digest per field over every cell, so a
// mismatch names the field; the first scenario also carries the cell types.

#include "oracle_wp51_world.h"

#include "GameLogic/AIPathfind.h"
#include "GameLogic/Damage.h"
#include "GameLogic/LocomotorSet.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Object.h"
#include "GameLogic/Pathfinder/PathfindCell.h"
#include "GameLogic/Pathfinder/PathfindLayer.h"
#include "GameLogic/Pathfinder/PathfindZoneManager.h"
#include "GameLogic/TerrainLogic.h"

#include <cmath>
#include <cstdio>
#include <cstring>
#include <string>
#include <vector>

#define S2_TAG(Tag, Type, Class, Member)                   \
    struct Tag { using type = Type Class::*; };            \
    template struct MemberAccess<Tag, &Class::Member>;     \
    inline Type Class::* access_member(Tag);

typedef PathfindLayer S2LayerArray[LAYER_LAST + 1];
typedef ObjectID S2WallArray[MAX_WALL_PIECES];

S2_TAG(PfMap, PathfindCell**, Pathfinder, m_map)
S2_TAG(PfExtent, IRegion2D, Pathfinder, m_extent)
S2_TAG(PfLogical, IRegion2D, Pathfinder, m_logicalExtent)
S2_TAG(PfReady, Bool, Pathfinder, m_isMapReady)
S2_TAG(PfZones, PathfindZoneManager, Pathfinder, m_zoneManager)
S2_TAG(PfLayers, S2LayerArray, Pathfinder, m_layers)
S2_TAG(PfWalls, S2WallArray, Pathfinder, m_wallPieces)
S2_TAG(PfNumWalls, Int, Pathfinder, m_numWallPieces)
S2_TAG(ZmMax, UnsignedShort, PathfindZoneManager, m_maxZone)
S2_TAG(ZmNext, UnsignedInt, PathfindZoneManager, m_nextFrameToCalculateZones)
S2_TAG(PlCells, PathfindCell**, PathfindLayer, m_layerCells)
S2_TAG(PlWidth, Int, PathfindLayer, m_width)
S2_TAG(PlHeight, Int, PathfindLayer, m_height)
S2_TAG(PlXOrigin, Int, PathfindLayer, m_xOrigin)
S2_TAG(PlYOrigin, Int, PathfindLayer, m_yOrigin)
S2_TAG(LsSurfaces, LocomotorSurfaceTypeMask, LocomotorSet, m_validLocomotorSurfaces)

extern Bool LineInRegion(const Coord2D* p1, const Coord2D* p2, const Region2D* clipRegion);

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

struct Fnv {
    uint64_t h = 0xcbf29ce484222325ULL;
    void add(int32_t v) {
        for (int i = 0; i < 4; ++i) {
            h ^= static_cast<uint8_t>(static_cast<uint32_t>(v) >> (8 * i));
            h *= 0x100000001b3ULL;
        }
    }
    std::string hex() const {
        char buf[24];
        std::snprintf(buf, sizeof(buf), "%016llx", static_cast<unsigned long long>(h));
        return buf;
    }
};

// ---- randomness ---------------------------------------------------------------

static uint32_t g_state = 0x52000001u;
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
// A coordinate that is often on a cell or half-cell boundary.
static float coord(float lo, float hi) {
    const float v = between(lo, hi);
    const int pick = below(10);
    if (pick == 0) return std::floor(v / 10.0f) * 10.0f;
    if (pick == 1) return std::floor(v / 5.0f) * 5.0f;
    if (pick == 2) return std::floor(v / 10.0f) * 10.0f - 0.5f;
    return v;
}
static float angle() {
    static const float fixed[] = {0.0f, 1.5707964f, 3.1415927f, -1.5707964f, 0.7853982f};
    return chance(45) ? fixed[below(5)] : between(-3.2f, 3.2f);
}

// ---- terrain ------------------------------------------------------------------

class S2Terrain : public oracle_wp51::OracleTerrainLogic {
public:
    int w = 1, h = 1;  // tables of (w + 2) x (h + 2)
    std::string cliff, water;
    std::vector<float> ground;

    int clampx(int i) const { return i < 0 ? 0 : (i > w + 1 ? w + 1 : i); }
    int clampy(int j) const { return j < 0 ? 0 : (j > h + 1 ? h + 1 : j); }
    size_t at(int i, int j) const { return static_cast<size_t>(clampy(j) * (w + 2) + clampx(i)); }
    static int cell(float v) { return static_cast<int>(std::floor(v / 10.0f)); }

    Real getGroundHeight(Real x, Real y, Coord3D* normal = nullptr) const override {
        if (normal) { normal->x = 0.0f; normal->y = 0.0f; normal->z = 1.0f; }
        return ground[at(cell(x), cell(y))];
    }
    Bool isCliffCell(Real x, Real y) const override { return cliff[at(cell(x), cell(y))] == '1'; }
    Bool isUnderwater(Real x, Real y, Real* waterZ = nullptr, Real* terrainZ = nullptr) override {
        if (waterZ) *waterZ = 0.0f;
        if (terrainZ) *terrainZ = getGroundHeight(x, y);
        return water[at(cell(x), cell(y))] == '1';
    }
};

static S2Terrain* g_terrain = nullptr;

static void blobs(std::string& table, const S2Terrain& t, int count) {
    for (int n = 0; n < count; ++n) {
        const int x = below(t.w + 2), y = below(t.h + 2), rx = 1 + below(5), ry = 1 + below(5);
        for (int j = y; j < y + ry; ++j)
            for (int i = x; i < x + rx; ++i)
                table[t.at(i, j)] = '1';
    }
}

static void make_terrain(int w, int h, bool rough) {
    S2Terrain& t = *g_terrain;
    t.w = w;
    t.h = h;
    const size_t n = static_cast<size_t>((w + 2) * (h + 2));
    t.cliff.assign(n, '0');
    t.water.assign(n, '0');
    t.ground.assign(n, 0.0f);
    if (rough) {
        blobs(t.cliff, t, below(5));
        blobs(t.water, t, below(5));
    }
    static const float levels[] = {0.0f, 0.0f, 0.0f, 4.0f, 9.999f, 10.0f, 15.0f, 25.0f, 40.0f};
    for (int b = below(rough ? 6 : 3); b > 0; --b) {
        const float level = levels[below(9)];
        const int x = below(w + 2), y = below(h + 2), rx = 2 + below(9), ry = 2 + below(9);
        for (int j = y; j < y + ry; ++j)
            for (int i = x; i < x + rx; ++i)
                t.ground[t.at(i, j)] = level;
    }
}

static void terrain_rows() {
    const S2Terrain& t = *g_terrain;
    row("terrain " + num(t.w) + " " + num(t.h) + " " + t.cliff + " " + t.water);
    // Heights as run lengths.
    std::string runs = "heights";
    size_t i = 0;
    while (i < t.ground.size()) {
        size_t j = i;
        while (j < t.ground.size() && t.ground[j] == t.ground[i]) ++j;
        runs += " " + num(static_cast<long long>(j - i)) + ":" + f32(t.ground[i]);
        i = j;
    }
    row(runs);
}

// ---- templates and objects ------------------------------------------------------

struct Kind {
    std::string name;
    int flags;  // the fixed bits of the footprint row
};

enum { F_MINE = 1, F_PROJ = 2, F_TOWER = 4, F_DEFWALL = 8, F_CRATER = 16, F_WALKWALL = 32,
       F_STRUCTURE = 64, F_SEETHROUGH = 128, F_MOBILE = 256, F_RUBBLE = 512 };

static std::vector<Kind> g_structures, g_fences, g_pieces, g_others;
static std::string g_ini;

static std::string decimal(float v) {
    char buf[32];
    std::snprintf(buf, sizeof(buf), "%.4f", v);
    return buf;
}

static void add_template(std::vector<Kind>& into, const std::string& name, const std::string& kindof,
                         const char* geom, float major, float minor, bool is_small, float fence,
                         float offset) {
    g_ini += "Object " + name + "\n  KindOf = " + kindof + "\n  Geometry = " + geom +
             "\n  GeometryMajorRadius = " + decimal(major) +
             "\n  GeometryMinorRadius = " + decimal(minor) +
             "\n  GeometryHeight = 12.0\n  GeometryIsSmall = " + (is_small ? "Yes" : "No") +
             "\n  FenceWidth = " + decimal(fence) + "\n  FenceXOffset = " + decimal(offset) +
             "\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100.0\n"
             "    InitialHealth = 100.0\n  End\nEnd\n";
    into.push_back({name, 0});
}

static float radius_choice() {
    static const float fixed[] = {5.0f, 10.0f, 15.0f, 20.0f, 25.0f, 2.0f};
    return chance(35) ? fixed[below(6)] : between(3.0f, 48.0f);
}

static void make_templates() {
    static const char* geoms[] = {"BOX", "CYLINDER", "SPHERE"};
    static const char* extra[] = {"", "", "", " CAN_SEE_THROUGH_STRUCTURE", " BLAST_CRATER",
                                  " MINE", " PROJECTILE", " BRIDGE_TOWER", ""};
    for (int i = 0; i < 36; ++i) {
        std::string kinds = "STRUCTURE";
        if (!chance(10)) kinds += " IMMOBILE";
        if (chance(45)) kinds += extra[below(9)];
        add_template(g_structures, "S2Structure" + num(i), kinds, geoms[i % 3], radius_choice(),
                     radius_choice(), chance(6), 0.0f, 0.0f);
    }
    add_template(g_structures, "S2NotStructure", "IMMOBILE", "BOX", 20, 20, false, 0, 0);
    for (int i = 0; i < 10; ++i) {
        const std::string kinds = i == 9 ? "STRUCTURE IMMOBILE DEFENSIVE_WALL"
                                         : (i == 8 ? "STRUCTURE IMMOBILE CAN_SEE_THROUGH_STRUCTURE"
                                                   : "STRUCTURE IMMOBILE");
        add_template(g_fences, "S2Fence" + num(i), kinds, "BOX", 20.0f, 3.0f, false,
                     i < 3 ? 10.0f * (i + 1) : between(4.0f, 90.0f),
                     chance(40) ? 0.0f : between(-10.0f, 30.0f));
    }
    for (int i = 0; i < 6; ++i) {
        std::string kinds = "STRUCTURE IMMOBILE WALK_ON_TOP_OF_WALL";
        float fence = 0.0f;
        if (i >= 4) { kinds += " DEFENSIVE_WALL"; fence = 20.0f; }
        add_template(g_pieces, "S2Wall" + num(i), kinds, i == 3 ? "CYLINDER" : "BOX",
                     between(14.0f, 40.0f), between(9.0f, 25.0f), false, fence, 0.0f);
    }
    add_template(g_pieces, "S2WallSphere", "STRUCTURE IMMOBILE WALK_ON_TOP_OF_WALL", "SPHERE",
                 24.0f, 5.0f, false, 0, 0);
    add_template(g_others, "S2Unit", "INFANTRY", "CYLINDER", 4.0f, 4.0f, true, 0, 0);
    add_template(g_others, "S2Probe", "INFANTRY", "BOX", 9.0f, 7.0f, false, 0, 0);
    add_template(g_others, "S2Deck", "IMMOBILE", "BOX", 30.0f, 12.0f, false, 0, 0);
    add_template(g_others, "S2Landmark", "IMMOBILE", "BOX", 55.0f, 18.0f, false, 0, 0);
}

static oracle_wp51::World* g_world = nullptr;

static int flags_of(Object* o) {
    int f = 0;
    if (o->isKindOf(KINDOF_MINE)) f |= F_MINE;
    if (o->isKindOf(KINDOF_PROJECTILE)) f |= F_PROJ;
    if (o->isKindOf(KINDOF_BRIDGE_TOWER)) f |= F_TOWER;
    if (o->isKindOf(KINDOF_DEFENSIVE_WALL)) f |= F_DEFWALL;
    if (o->isKindOf(KINDOF_BLAST_CRATER)) f |= F_CRATER;
    if (o->isKindOf(KINDOF_WALK_ON_TOP_OF_WALL)) f |= F_WALKWALL;
    if (o->isKindOf(KINDOF_STRUCTURE)) f |= F_STRUCTURE;
    if (o->isKindOf(KINDOF_CAN_SEE_THROUGH_STRUCTURE)) f |= F_SEETHROUGH;
    if (o->isMobile()) f |= F_MOBILE;
    if (o->getBodyModule() && o->getBodyModule()->getDamageState() == BODY_RUBBLE) f |= F_RUBBLE;
    return f;
}

static void object_row(Object* o) {
    const GeometryInfo& g = o->getGeometryInfo();
    row("obj " + num(o->getID()) + " " + c3(*o->getPosition()) + " " + f32(o->getOrientation()) + " " +
        num(g.getGeomType()) + " " + num(g.getIsSmall()) + " " + f32(g.getMaxHeightAbovePosition()) +
        " " + f32(g.getMajorRadius()) + " " + f32(g.getMinorRadius()) + " " +
        f32(o->getTemplate()->getFenceWidth()) + " " + f32(o->getTemplate()->getFenceXOffset()) +
        " " + num(flags_of(o)) + " " + f32(o->getHeightAboveTerrain()) + " " + num(o->getLayer()));
}

static void order_row() {
    std::string r = "order";
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
        r += " " + num(o->getID());
    }
    row(r);
}

static Object* spawn(const std::string& name, float x, float y, float z, float a) {
    Coord3D p;
    p.x = x;
    p.y = y;
    p.z = z;
    Object* o = g_world->spawn_object(name.c_str(), g_world->team_a, p, a);
    if (!o) { std::fprintf(stderr, "no object %s\n", name.c_str()); std::exit(1); }
    return o;
}

// ---- state ------------------------------------------------------------------------

static Pathfinder* pf() { return TheAI->pathfinder(); }

static void state_row(bool verbose) {
    Pathfinder* p = pf();
    const IRegion2D extent = p->*access_member(PfExtent());
    const IRegion2D logical = p->*access_member(PfLogical());
    PathfindCell** map = p->*access_member(PfMap());
    PathfindZoneManager& zm = p->*access_member(PfZones());
    S2LayerArray& layers = p->*access_member(PfLayers());

    Fnv meta, type, pinched, zone, connect, layer, obstacle, obsflags, info;
    meta.add(extent.lo.x); meta.add(extent.lo.y); meta.add(extent.hi.x); meta.add(extent.hi.y);
    meta.add(logical.lo.x); meta.add(logical.lo.y); meta.add(logical.hi.x); meta.add(logical.hi.y);
    meta.add(p->*access_member(PfReady()));
    meta.add(zm.*access_member(ZmMax()));
    meta.add(static_cast<int32_t>(zm.*access_member(ZmNext())));
    meta.add(p->*access_member(PfNumWalls()));
    S2WallArray& walls = p->*access_member(PfWalls());
    for (int i = 0; i < p->*access_member(PfNumWalls()); ++i) meta.add(walls[i]);

    std::string picture;
    for (int i = 0; i <= extent.hi.x; ++i) {
        for (int j = 0; j <= extent.hi.y; ++j) {
            PathfindCell& c = map[i][j];
            type.add(c.getType());
            pinched.add(c.getPinched());
            zone.add(c.getZone());
            connect.add(c.getConnectLayer());
            layer.add(c.getLayer());
            obstacle.add(c.getObstacleID());
            obsflags.add((c.isObstacleFence() ? 1 : 0) | (c.isObstacleTransparent() ? 2 : 0));
            info.add(c.hasInfo());
            if (verbose) picture += static_cast<char>('0' + c.getType() + (c.getPinched() ? 10 : 0));
        }
    }

    Fnv lmeta, ltype, lpinched, lzone, lconnect, llayer, lget;
    for (int k = 0; k <= LAYER_LAST; ++k) {
        PathfindLayer& l = layers[k];
        ICoord2D start, end;
        l.getStartCellIndex(&start);
        l.getEndCellIndex(&end);
        lmeta.add(l.isUnused());
        lmeta.add(l.isDestroyed());
        lmeta.add(l.getZone());
        lmeta.add(start.x); lmeta.add(start.y); lmeta.add(end.x); lmeta.add(end.y);
        PathfindCell** cells = l.*access_member(PlCells());
        const int w = cells ? l.*access_member(PlWidth()) : 0;
        const int h = cells ? l.*access_member(PlHeight()) : 0;
        lmeta.add(w);
        lmeta.add(h);
        if (w > 0) {
            lmeta.add(l.*access_member(PlXOrigin()));
            lmeta.add(l.*access_member(PlYOrigin()));
        }
        for (int i = 0; i < w; ++i) {
            for (int j = 0; j < h; ++j) {
                PathfindCell& c = cells[i][j];
                ltype.add(c.getType());
                lpinched.add(c.getPinched());
                lzone.add(c.getZone());
                lconnect.add(c.getConnectLayer());
                llayer.add(c.getLayer());
            }
        }
        // `getCell`, over a border around the layer.
        if (w > 0) {
            const int x0 = l.*access_member(PlXOrigin()), y0 = l.*access_member(PlYOrigin());
            for (int i = x0 - 1; i <= x0 + w; ++i)
                for (int j = y0 - 1; j <= y0 + h; ++j)
                    lget.add(l.getCell(i, j) != nullptr);
        }
    }

    static const int masks[] = {LOCOMOTORSURFACE_GROUND,
                                LOCOMOTORSURFACE_GROUND | LOCOMOTORSURFACE_CLIFF,
                                LOCOMOTORSURFACE_GROUND | LOCOMOTORSURFACE_WATER,
                                LOCOMOTORSURFACE_GROUND | LOCOMOTORSURFACE_RUBBLE,
                                LOCOMOTORSURFACE_WATER, LOCOMOTORSURFACE_AIR,
                                LOCOMOTORSURFACE_CLIFF,
                                LOCOMOTORSURFACE_GROUND | LOCOMOTORSURFACE_CLIFF |
                                    LOCOMOTORSURFACE_WATER};
    Fnv equiv, block, bridges;
    for (int m = 0; m < 8; ++m) {
        for (int crusher = 0; crusher < 2; ++crusher) {
            for (int i = 0; i <= extent.hi.x; ++i) {
                for (int j = 0; j <= extent.hi.y; ++j) {
                    equiv.add(zm.getEffectiveZone(masks[m], crusher, map[i][j].getZone()));
                    block.add(zm.getBlockZone(masks[m], crusher, i, j, map));
                }
            }
        }
    }
    for (int i = 0; i <= extent.hi.x; ++i) {
        for (int j = 0; j <= extent.hi.y; ++j) {
            equiv.add(zm.getEffectiveTerrainZone(map[i][j].getZone()));
            bridges.add(zm.interactsWithBridge(i, j));
        }
    }

    char crc[16];
    std::snprintf(crc, sizeof(crc), "%08x", oracle_wp51::World::snapshot_crc(p));
    row("state meta=" + meta.hex() + " type=" + type.hex() + " pinched=" + pinched.hex() +
        " connect=" + connect.hex() + " layer=" + layer.hex() + " obstacle=" + obstacle.hex() +
        " obsflags=" + obsflags.hex() + " info=" + info.hex() + " lmeta=" + lmeta.hex() +
        " ltype=" + ltype.hex() + " lpinched=" + lpinched.hex() + " lzone=" + lzone.hex() +
        " lconnect=" + lconnect.hex() + " llayer=" + llayer.hex() + " lget=" + lget.hex() +
        " zone=" + zone.hex() + " equiv=" + equiv.hex() +
        " block=" + block.hex() + " bridges=" + bridges.hex() + " crc=" + crc);
    if (verbose) row("picture " + picture);
}

// ---- queries ------------------------------------------------------------------------

static std::vector<Bridge*> bridge_list() {
    std::vector<Bridge*> out;
    for (Bridge* b = TheTerrainLogic->getFirstBridge(); b; b = b->getNext()) out.push_back(b);
    return out;
}

static std::string geometry_answers(Bridge* b, const Coord3D& p, const Region2D& r) {
    Coord3D normal;
    const float height = b->getBridgeHeight(&p, &normal);
    return num(b->isPointOnBridge(&p)) + num(b->isCellOnEnd(&r)) + num(b->isCellOnSide(&r)) +
           num(b->isCellEntryPoint(&r)) + " " + f32(height) + " " + c3(normal);
}

static Region2D random_region(const Coord3D& p) {
    Region2D r;
    if (chance(60)) {
        r.lo.x = std::floor(p.x / 10.0f) * 10.0f;
        r.lo.y = std::floor(p.y / 10.0f) * 10.0f;
        r.hi.x = r.lo.x + 10.0f;
        r.hi.y = r.lo.y + 10.0f;
    } else {
        r.lo.x = p.x - between(0.0f, 20.0f);
        r.lo.y = p.y - between(0.0f, 20.0f);
        r.hi.x = r.lo.x + between(0.0f, 30.0f);
        r.hi.y = r.lo.y + between(0.0f, 30.0f);
    }
    return r;
}

static Coord3D near_a_bridge(float maxx, float maxy) {
    Coord3D p;
    const std::vector<Bridge*> all = bridge_list();
    if (!all.empty() && chance(80)) {
        const BridgeInfo* i = all[below(static_cast<int>(all.size()))]->peekBridgeInfo();
        const float u = between(-0.2f, 1.2f), v = between(-0.3f, 1.3f);
        const float lx = i->fromLeft.x + (i->toLeft.x - i->fromLeft.x) * u;
        const float ly = i->fromLeft.y + (i->toLeft.y - i->fromLeft.y) * u;
        const float rx = i->fromRight.x + (i->toRight.x - i->fromRight.x) * u;
        const float ry = i->fromRight.y + (i->toRight.y - i->fromRight.y) * u;
        p.x = lx + (rx - lx) * v;
        p.y = ly + (ry - ly) * v;
        if (chance(10)) { p.x = i->fromLeft.x; p.y = i->fromLeft.y; }
        const float deck = i->from.z + (i->to.z - i->from.z) * u;
        static const float off[] = {0.0f, 1.0f, -1.0f, 9.5f, -9.5f, 10.5f, 30.0f, -30.0f};
        p.z = chance(75) ? deck + off[below(8)] : between(-10.0f, 80.0f);
    } else {
        p.x = coord(-20.0f, maxx + 20.0f);
        p.y = coord(-20.0f, maxy + 20.0f);
        p.z = between(-10.0f, 60.0f);
    }
    return p;
}

static void query_rows(Object* probe, float maxx, float maxy, int count) {
    for (int q = 0; q < count; ++q) {
        Coord3D p = near_a_bridge(maxx, maxy);
        const Region2D r = random_region(p);
        probe->setPosition(&p);
        const std::vector<Bridge*> all = bridge_list();
        Bridge* at = TheTerrainLogic->findBridgeAt(&p);
        std::string out = "query " + c3(p) + " " + f32(r.lo.x) + " " + f32(r.lo.y) + " " +
                          f32(r.hi.x) + " " + f32(r.hi.y) + " " +
                          f32(probe->getGeometryInfo().getMinorRadius()) + " = " +
                          num(at ? at->getLayer() : 0) + " " +
                          num(TheTerrainLogic->getLayerForDestination(&p)) + " " +
                          num(TheTerrainLogic->getHighestLayerForDestination(&p, false)) + " " +
                          num(TheTerrainLogic->getHighestLayerForDestination(&p, true)) + " " +
                          num(pf()->isPointOnWall(&p));
        for (Bridge* b : all) {
            const int layer = b->getLayer();
            out += " | " + geometry_answers(b, p, r) + " " +
                   num(TheTerrainLogic->objectInteractsWithBridgeLayer(probe, layer, false)) +
                   num(TheTerrainLogic->objectInteractsWithBridgeLayer(probe, layer, true)) +
                   num(TheTerrainLogic->objectInteractsWithBridgeEnd(probe, layer));
        }
        row(out);
    }
}

// Bridges that are never added to the map, including flat and collinear ones.
static void geometry_rows(int count) {
    for (int n = 0; n < count; ++n) {
        BridgeInfo info;
        Coord3D* c[] = {&info.fromLeft, &info.fromRight, &info.toLeft, &info.toRight};
        const int shape = below(8);
        for (int k = 0; k < 4; ++k) {
            c[k]->x = coord(0.0f, 100.0f);
            c[k]->y = coord(0.0f, 100.0f);
            c[k]->z = between(0.0f, 20.0f);
            if (shape == 0) c[k]->y = 40.0f;                       // collinear in plan
            if (shape == 1) { c[k]->x = 30.0f; c[k]->y = 30.0f; }  // one point
            if (shape == 2) c[k]->y = c[k]->x;                     // a diagonal line
        }
        if (shape == 3) { info.toLeft = info.fromRight; info.toRight = info.fromRight; }
        info.from.x = (info.fromLeft.x + info.fromRight.x) / 2.0f;
        info.from.y = (info.fromLeft.y + info.fromRight.y) / 2.0f;
        info.from.z = (info.fromLeft.z + info.fromRight.z) / 2.0f;
        info.to.x = (info.toLeft.x + info.toRight.x) / 2.0f;
        info.to.y = (info.toLeft.y + info.toRight.y) / 2.0f;
        info.to.z = (info.toLeft.z + info.toRight.z) / 2.0f;
        info.bridgeObjectID = INVALID_ID;
        Dict props;
        Bridge* b = newInstance(Bridge)(info, &props, AsciiString("S2NoRoad"));
        row("shape " + c3(info.from) + " " + c3(info.to) + " " + c3(info.fromLeft) + " " +
            c3(info.fromRight) + " " + c3(info.toLeft) + " " + c3(info.toRight));
        for (int q = 0; q < 12; ++q) {
            Coord3D p;
            p.x = coord(-10.0f, 110.0f);
            p.y = coord(-10.0f, 110.0f);
            p.z = 0.0f;
            if (shape == 0 && q % 2) p.y = 40.0f;
            if (shape == 1 && q % 3 == 0) { p.x = 30.0f; p.y = 30.0f; }
            if (shape == 2 && q % 2) p.y = p.x;
            if (q % 5 == 0) p = *c[below(4)];
            const Region2D r = random_region(p);
            // A vertical deck leaves the height unset in the original. Corners
            // with no plane at all fall back to a flat one and are defined.
            const Vector3 u(info.fromRight.x - info.fromLeft.x, info.fromRight.y - info.fromLeft.y,
                            info.fromRight.z - info.fromLeft.z);
            const Vector3 v(info.toLeft.x - info.fromLeft.x, info.toLeft.y - info.fromLeft.y,
                            info.toLeft.z - info.fromLeft.z);
            Vector3 n3;
            Vector3::Cross_Product(u, v, &n3);
            const bool no_plane = n3.X == 0.0f && n3.Y == 0.0f && n3.Z == 0.0f;
            std::string heights = "-";
            if (no_plane || std::fabs(n3.Z) > 1.0f) {
                Coord3D normal;
                const float height = b->getBridgeHeight(&p, &normal);
                heights = f32(height) + " " + c3(normal);
            }
            row("probe " + c3(p) + " " + f32(r.lo.x) + " " + f32(r.lo.y) + " " + f32(r.hi.x) + " " +
                f32(r.hi.y) + " = " + num(b->isPointOnBridge(&p)) + num(b->isCellOnEnd(&r)) +
                num(b->isCellOnSide(&r)) + num(b->isCellEntryPoint(&r)) + " " + heights);
        }
        deleteInstance(b);
    }
    // Segments against regions: round numbers (so edges, corners and unit
    // differences come up) and arbitrary ones (so clipped ends land off by a bit).
    for (int n = 0; n < 6000; ++n) {
        Coord2D a, b;
        Region2D r;
        r.lo.x = 10.0f; r.lo.y = 10.0f; r.hi.x = 20.0f; r.hi.y = 20.0f;
        if (n % 3 == 2) {
            r.lo.x = between(0.0f, 15.0f);
            r.lo.y = between(0.0f, 15.0f);
            r.hi.x = r.lo.x + between(0.1f, 20.0f);
            r.hi.y = r.lo.y + between(0.1f, 20.0f);
        }
        auto pick = [n]() {
            static const float v[] = {0.0f, 10.0f, 15.0f, 20.0f, 30.0f};
            if (n % 3 == 1) return static_cast<float>(8 + below(15));
            return chance(n % 3 == 2 ? 5 : 50) ? v[below(5)] : between(-5.0f, 35.0f);
        };
        a.x = pick(); a.y = pick(); b.x = pick(); b.y = pick();
        row("line " + f32(a.x) + " " + f32(a.y) + " " + f32(b.x) + " " + f32(b.y) + " " +
            f32(r.lo.x) + " " + f32(r.lo.y) + " " + f32(r.hi.x) + " " + f32(r.hi.y) + " = " +
            num(LineInRegion(&a, &b, &r)));
    }
}

// ---- scenarios --------------------------------------------------------------------------

struct Scene {
    int index = 0;
    int w = 0, h = 0;
    UnsignedInt frame = 0;
    float maxx = 0, maxy = 0;
    std::vector<Object*> added;    // footprints on the map
    std::vector<Object*> removed;  // alive but off the map
    std::vector<Object*> pieces;   // wall pieces on the map
    std::vector<Object*> decks;    // bridge objects
    std::vector<Object*> everything;
    Object* probe = nullptr;
};

static void set_frame(Scene& s, UnsignedInt f) {
    s.frame = f;
    g_world->set_frame(f);
}

static Object* random_structure(Scene& s) {
    const bool fence = chance(25);
    const std::vector<Kind>& pool = fence ? g_fences : g_structures;
    const std::string name = pool[below(static_cast<int>(pool.size()))].name;
    const float x = coord(-15.0f, s.maxx + 15.0f), y = coord(-15.0f, s.maxy + 15.0f);
    const float ground = g_terrain->getGroundHeight(x, y);
    static const float lift[] = {0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 0.0f, 10.0f, 10.01f, 25.0f, -5.0f};
    Object* o = spawn(name, x, y, ground + lift[below(10)], angle());
    if (chance(5)) o->setDisabled(DISABLED_HACKED);  // a disabled immobile object stays still
    s.everything.push_back(o);
    return o;
}

static std::string fallen_ids(const std::vector<Object*>& units, const std::vector<float>& before) {
    std::string out;
    for (size_t i = 0; i < units.size(); ++i) {
        if (before[i] > 0.0f && units[i]->getBodyModule()->getHealth() <= 0.0f) {
            out += " " + num(units[i]->getID());
        }
    }
    return out;
}

static std::vector<Object*> units_on_layer(int layer) {
    std::vector<Object*> out;
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
        if (o->getLayer() == layer && o->getTemplate()->getName() == "S2Unit") out.push_back(o);
    }
    return out;
}

static std::vector<float> health_of(const std::vector<Object*>& units) {
    std::vector<float> out;
    for (Object* u : units) out.push_back(u->getBodyModule()->getHealth());
    return out;
}

static void add_bridge(Scene& s, float length_scale) {
    Coord3D from, to;
    const int style = below(5);
    const float len = between(30.0f, 40.0f + length_scale);
    float a = angle();
    from.x = coord(20.0f, s.maxx - 20.0f);
    from.y = coord(20.0f, s.maxy - 20.0f);
    from.z = chance(40) ? 0.0f : between(0.0f, 30.0f);
    to.x = from.x + len * std::cos(static_cast<double>(a));
    to.y = from.y + len * std::sin(static_cast<double>(a));
    to.z = chance(50) ? from.z : between(0.0f, 30.0f);
    if (style == 0) { to.y = from.y; to.x = from.x + std::floor(len / 10.0f) * 10.0f; }
    if (style == 1) { to.x = from.x; to.y = from.y + len; }
    const float width = chance(40) ? 30.0f : between(14.0f, 60.0f);
    float dx = to.x - from.x, dy = to.y - from.y;
    const float d = std::sqrt(dx * dx + dy * dy);
    const float px = -dy / d * width / 2.0f, py = dx / d * width / 2.0f;
    BridgeInfo info;
    info.from = from;
    info.to = to;
    info.bridgeWidth = width;
    info.fromLeft.x = from.x + px; info.fromLeft.y = from.y + py; info.fromLeft.z = from.z;
    info.fromRight.x = from.x - px; info.fromRight.y = from.y - py; info.fromRight.z = from.z;
    info.toLeft.x = to.x + px; info.toLeft.y = to.y + py; info.toLeft.z = to.z;
    info.toRight.x = to.x - px; info.toRight.y = to.y - py; info.toRight.z = to.z;
    Object* deck = spawn("S2Deck", from.x, from.y, from.z, a);
    s.decks.push_back(deck);
    s.everything.push_back(deck);
    info.bridgeObjectID = deck->getID();
    for (int k = 0; k < BRIDGE_MAX_TOWERS; ++k) info.towerObjectID[k] = INVALID_ID;
    info.bridgeIndex = 0;
    info.curDamageState = BODY_PRISTINE;
    info.damageStateChanged = false;
    Dict props;
    TheTerrainLogic->addBridgeToLogic(&info, &props, AsciiString("S2NoRoad"));
    row("bridge " + num(deck->getID()) + " " + c3(info.from) + " " + c3(info.to) + " " +
        c3(info.fromLeft) + " " + c3(info.fromRight) + " " + c3(info.toLeft) + " " +
        c3(info.toRight) + " = " + num(TheTerrainLogic->getFirstBridge()->getLayer()));
}

static void add_landmark(Scene& s) {
    const float x = coord(40.0f, s.maxx - 40.0f), y = coord(40.0f, s.maxy - 40.0f);
    Object* o = spawn("S2Landmark", x, y, between(0.0f, 20.0f), angle());
    s.decks.push_back(o);
    s.everything.push_back(o);
    TheTerrainLogic->addLandmarkBridgeToLogic(o);
    const Bridge* b = TheTerrainLogic->getFirstBridge();
    const BridgeInfo* i = b->peekBridgeInfo();
    row("landmark " + num(o->getID()) + " " + c3(*o->getPosition()) + " " + f32(o->getOrientation()) +
        " " + f32(o->getGeometryInfo().getMajorRadius()) + " " +
        f32(o->getGeometryInfo().getMinorRadius()) + " = " + num(b->getLayer()) + " " + c3(i->from) +
        " " + c3(i->to) + " " + c3(i->fromLeft) + " " + c3(i->fromRight) + " " + c3(i->toLeft) + " " +
        c3(i->toRight));
}

// A line of wall pieces.
static void add_wall(Scene& s) {
    const float a = chance(50) ? 0.0f : angle();
    float x = coord(30.0f, s.maxx * 0.5f), y = coord(30.0f, s.maxy - 30.0f);
    const int count = 1 + below(6);
    const std::string name = g_pieces[below(static_cast<int>(g_pieces.size()))].name;
    for (int n = 0; n < count; ++n) {
        Object* o = spawn(chance(75) ? name : g_pieces[below(static_cast<int>(g_pieces.size()))].name,
                          x, y, 0.0f, a);
        s.pieces.push_back(o);
        s.everything.push_back(o);
        const float step = o->getGeometryInfo().getMajorRadius() * (chance(80) ? 1.6f : 2.6f);
        x += step * std::cos(static_cast<double>(a));
        y += step * std::sin(static_cast<double>(a));
    }
}

static void begin_scene(Scene& s) {
    // Destroy the last scene's objects while their footprints can still be removed.
    row("scenario " + num(s.index));
}

static void end_scene(Scene& s) {
    for (Object* o : s.everything) TheGameLogic->destroyObject(o);
    TheGameLogic->processDestroyList();
    TheTerrainLogic->reset();
    pf()->reset();
    char crc[16];
    std::snprintf(crc, sizeof(crc), "%08x", oracle_wp51::World::snapshot_crc(pf()));
    row("reset = " + std::string(crc));
}

static void run_newmap(Scene& s, bool verbose) {
    order_row();
    for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) object_row(o);
    pf()->newMap();
    row("newmap " + num(s.frame));
    state_row(verbose);
}

static void one_op(Scene& s, bool verbose) {
    const int op = below(100);
    if (chance(35)) set_frame(s, s.frame + 1 + below(chance(20) ? 400 : 3));
    if (op < 24) {  // a new object
        Object* o = random_structure(s);
        object_row(o);
        pf()->addObjectToPathfindMap(o);
        s.added.push_back(o);
        row("add " + num(o->getID()) + " " + num(s.frame));
    } else if (op < 36 && !s.added.empty()) {  // remove
        const int k = below(static_cast<int>(s.added.size()));
        Object* o = s.added[k];
        object_row(o);
        pf()->removeObjectFromPathfindMap(o);
        s.added.erase(s.added.begin() + k);
        s.removed.push_back(o);
        row("remove " + num(o->getID()) + " " + num(s.frame) + " =");
    } else if (op < 41 && !s.removed.empty()) {  // put back
        const int k = below(static_cast<int>(s.removed.size()));
        Object* o = s.removed[k];
        object_row(o);
        pf()->addObjectToPathfindMap(o);
        s.removed.erase(s.removed.begin() + k);
        s.added.push_back(o);
        row("add " + num(o->getID()) + " " + num(s.frame));
    } else if (op < 49 && !s.added.empty()) {  // to rubble
        Object* o = s.added[below(static_cast<int>(s.added.size()))];
        if (o->getBodyModule()->getDamageState() == BODY_RUBBLE) return;
        const bool structure = o->isKindOf(KINDOF_STRUCTURE);
        o->getBodyModule()->setDamageState(BODY_RUBBLE);
        object_row(o);
        if (!structure) return;
        // `ActiveBody::setCorrectDamageState` takes the structure off the map
        // and puts it back, and `setDamageState` reaches it twice: once from
        // `internalChangeHealth` and once itself.
        for (int twice = 0; twice < 2; ++twice) {
            row("remove " + num(o->getID()) + " " + num(s.frame) + " =");
            row("add " + num(o->getID()) + " " + num(s.frame));
        }
        if (chance(60)) {  // and rubble classified afresh, as after a removal
            pf()->removeObjectFromPathfindMap(o);
            row("remove " + num(o->getID()) + " " + num(s.frame) + " =");
            if (chance(50)) {
                pf()->addObjectToPathfindMap(o);
                row("add " + num(o->getID()) + " " + num(s.frame));
            }
        }
    } else if (op < 55) {  // a temporary wall from a footprint
        Object* o = random_structure(s);
        object_row(o);
        pf()->createAWallFromMyFootprint(o);
        row("wallon " + num(o->getID()) + " " + num(s.frame));
        if (chance(70)) {
            state_row(false);
            pf()->removeWallFromMyFootprint(o);
            row("walloff " + num(o->getID()) + " " + num(s.frame));
        }
    } else if (op < 70) {  // the frame's zone pass
        if (chance(60)) set_frame(s, s.frame + 300);
        pf()->processPathfindQueue();
        row("zones " + num(s.frame));
    } else if (op < 74) {  // the terrain changes under the map
        blobs(chance(50) ? g_terrain->cliff : g_terrain->water, *g_terrain, 1 + below(2));
        if (chance(30)) g_terrain->water.assign(g_terrain->water.size(), '0');
        terrain_rows();
        pf()->forceMapRecalculation();
        row("recalc");
    } else if (op < 86 && !s.decks.empty()) {
        const int layer = 2 + below(15 - 2);
        const bool repaired = chance(45);
        pf()->changeBridgeState(static_cast<PathfindLayerEnum>(layer), repaired);
        row("bridgestate " + num(layer) + " " + num(repaired) + " " + num(s.frame));
    } else if (op < 94 && !s.decks.empty()) {  // damage states from the bridge objects
        const std::vector<Bridge*> all = bridge_list();
        if (all.empty()) return;
        // Units standing on the decks.
        for (int n = below(4); n > 0; --n) {
            Bridge* b = all[below(static_cast<int>(all.size()))];
            Coord3D p = near_a_bridge(s.maxx, s.maxy);
            p.z = b->getBridgeHeight(&p, nullptr) + (chance(20) ? 25.0f : 0.0f);
            Object* u = spawn("S2Unit", p.x, p.y, p.z, 0.0f);
            u->setLayer(b->getLayer());
            s.everything.push_back(u);
        }
        Object* deck = s.decks[below(static_cast<int>(s.decks.size()))];
        static const BodyDamageType states[] = {BODY_PRISTINE, BODY_DAMAGED, BODY_RUBBLE,
                                                BODY_RUBBLE, BODY_REALLYDAMAGED};
        deck->getBodyModule()->setDamageState(states[below(5)]);
        row("decks " + num(deck->getID()) + " " + num(deck->getBodyModule()->getDamageState()));
        std::vector<Object*> units;
        for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
            if (o->getTemplate()->getName() == "S2Unit") units.push_back(o);
        }
        for (Object* u : units) {
            row("unit " + num(u->getID()) + " " + c3(*u->getPosition()) + " " +
                f32(u->getGeometryInfo().getMinorRadius()) + " " + num(u->getLayer()) + " " +
                num(u->getBodyModule()->getHealth() > 0.0f));
        }
        const std::vector<float> before = health_of(units);
        TheTerrainLogic->updateBridgeDamageStates();
        std::string answer = "damage " + num(s.frame) + " = fallen" + fallen_ids(units, before) + " ; ";
        for (Bridge* b : bridge_list()) {
            answer += num(TheTerrainLogic->isBridgeRepaired(
                          TheGameLogic->findObjectByID(b->peekBridgeInfo()->bridgeObjectID))) +
                      num(TheTerrainLogic->isBridgeBroken(
                          TheGameLogic->findObjectByID(b->peekBridgeInfo()->bridgeObjectID))) +
                      num(b->peekBridgeInfo()->curDamageState) + " ";
        }
        row(answer);
        for (Object* u : units) TheGameLogic->destroyObject(u);
        for (size_t k = s.everything.size(); k-- > 0;) {
            if (s.everything[k]->getTemplate()->getName() == "S2Unit")
                s.everything.erase(s.everything.begin() + k);
        }
        TheGameLogic->processDestroyList();
        row("unitsgone");
    } else if (!s.pieces.empty()) {  // lose a wall piece
        const int k = below(static_cast<int>(s.pieces.size()));
        Object* o = s.pieces[k];
        std::vector<Object*> units;
        for (int n = 0; n < 5; ++n) {
            Coord3D p = *s.pieces[chance(70) ? k : below(static_cast<int>(s.pieces.size()))]->getPosition();
            p.x += between(-30.0f, 30.0f);
            p.y += between(-30.0f, 30.0f);
            Object* u = spawn("S2Unit", p.x, p.y, p.z, 0.0f);
            u->setLayer(chance(80) ? LAYER_WALL : LAYER_GROUND);
        }
        for (Object* u : units_on_layer(LAYER_WALL)) {
            units.push_back(u);
            row("unit " + num(u->getID()) + " " + c3(*u->getPosition()) + " " +
                f32(u->getGeometryInfo().getMinorRadius()) + " " + num(u->getLayer()) + " 1");
        }
        const std::vector<float> before = health_of(units);
        object_row(o);
        pf()->removeObjectFromPathfindMap(o);
        row("remove " + num(o->getID()) + " " + num(s.frame) + " =" + fallen_ids(units, before));
        s.pieces.erase(s.pieces.begin() + k);
        std::vector<Object*> all_units;
        for (Object* u = TheGameLogic->getFirstObject(); u; u = u->getNextObject()) {
            if (u->getTemplate()->getName() == "S2Unit") all_units.push_back(u);
        }
        for (Object* u : all_units) TheGameLogic->destroyObject(u);
        TheGameLogic->processDestroyList();
        row("unitsgone");
        if (chance(40)) {
            pf()->removeWallPiece(o);
            row("unwallpiece " + num(o->getID()));
        }
    } else {
        return;
    }
    state_row(verbose);
}

static void run_scene(int index) {
    Scene s;
    s.index = index;
    begin_scene(s);
    const bool ground_only = index < 16;
    s.w = index == 0 ? 24 : 10 + below(55);
    s.h = index == 0 ? 20 : 10 + below(40);
    if (index % 13 == 7) { s.w = 90 + below(20); s.h = 20 + below(10); }
    s.maxx = s.w * 10.0f;
    s.maxy = s.h * 10.0f;
    make_terrain(s.w, s.h, index % 5 != 4);
    g_terrain->extent.lo.x = 0.0f;
    g_terrain->extent.lo.y = 0.0f;
    g_terrain->extent.lo.z = 0.0f;
    g_terrain->extent.hi.x = s.maxx + (chance(50) ? 0.0f : between(0.0f, 9.9f));
    g_terrain->extent.hi.y = s.maxy + (chance(50) ? 0.0f : between(0.0f, 9.9f));
    g_terrain->extent.hi.z = 100.0f;
    static const float walls[] = {0.0f, 12.0f, 20.0f, 33.5f};
    const float wall_height = walls[below(4)];
    const_cast<TAiData*>(TheAI->getAiData())->m_wallHeight = wall_height;
    pf()->reset();
    row("map " + f32(g_terrain->extent.hi.x) + " " + f32(g_terrain->extent.hi.y) + " " + f32(wall_height));
    terrain_rows();
    set_frame(s, index % 3 == 0 ? 0 : static_cast<UnsignedInt>(below(index % 3 == 1 ? 3 : 5000)));

    for (int n = below(10); n > 0; --n) s.added.push_back(random_structure(s));
    if (!ground_only) {
        if (index % 4 != 1) {
            const int walls_here = 1 + below(2);
            for (int n = 0; n < walls_here; ++n) add_wall(s);
            for (Object* o : s.pieces) {
                pf()->addWallPiece(o);
                row("wallpiece " + num(o->getID()));
            }
        }
        if (index % 4 != 2) {
            int bridges = 1 + below(3);
            if (index % 19 == 3) bridges = 15;  // runs out of layers
            for (int n = 0; n < bridges; ++n) {
                if (chance(20)) add_landmark(s);
                else add_bridge(s, s.maxx * 0.4f);
            }
        }
    }
    s.probe = spawn("S2Probe", 0.0f, 0.0f, 0.0f, 0.0f);
    s.everything.push_back(s.probe);
    const bool verbose = index == 0;
    run_newmap(s, verbose);
    if (!ground_only) query_rows(s.probe, s.maxx, s.maxy, 25);

    const int ops = 16 + below(14);
    for (int n = 0; n < ops; ++n) {
        one_op(s, verbose);
        if (!ground_only && n % 6 == 5) query_rows(s.probe, s.maxx, s.maxy, 8);
        if (n == ops / 2 && index % 6 == 2) {
            // The same size again: the cells are kept and classified over.
            run_newmap(s, verbose);
        }
    }
    set_frame(s, s.frame + 400);
    pf()->processPathfindQueue();
    row("zones " + num(s.frame));
    state_row(verbose);

    // Which destroyed bridges would join two zones.
    if (!ground_only) {
        PathfindCell** map = pf()->*access_member(PfMap());
        const IRegion2D extent = pf()->*access_member(PfExtent());
        PathfindZoneManager& zm = pf()->*access_member(PfZones());
        S2LayerArray& layers = pf()->*access_member(PfLayers());
        for (int layer = 2; layer < LAYER_LAST; ++layer) {
            if (layers[layer].isUnused()) continue;
            pf()->changeBridgeState(static_cast<PathfindLayerEnum>(layer), false);
            row("bridgestate " + num(layer) + " 0 " + num(s.frame));
            std::vector<int> zones;
            for (int i = 0; i <= extent.hi.x; ++i)
                for (int j = 0; j <= extent.hi.y; ++j)
                    if (map[i][j].getConnectLayer() != LAYER_INVALID || chance(1))
                        zones.push_back(zm.getEffectiveTerrainZone(
                            zm.getEffectiveZone(LOCOMOTORSURFACE_GROUND, true, map[i][j].getZone())));
            ICoord2D at;
            layers[layer].getStartCellIndex(&at);
            ICoord2D to;
            layers[layer].getEndCellIndex(&to);
            for (const ICoord2D& c : {at, to}) {
                for (int dx = -2; dx <= 2; ++dx) {
                    const int i = c.x + dx;
                    if (i >= 0 && i <= extent.hi.x && c.y >= 0 && c.y <= extent.hi.y)
                        zones.push_back(zm.getEffectiveTerrainZone(
                            zm.getEffectiveZone(LOCOMOTORSURFACE_GROUND, true, map[i][c.y].getZone())));
                }
            }
            if (zones.empty()) continue;
            // Destroyed, then the same questions of the repaired bridge.
            for (int repaired = 0; repaired < 2; ++repaired) {
                if (repaired) {
                    pf()->changeBridgeState(static_cast<PathfindLayerEnum>(layer), true);
                    row("bridgestate " + num(layer) + " 1 " + num(s.frame));
                }
                for (int k = 0; k < (repaired ? 6 : 20); ++k) {
                    const int a = zones[below(static_cast<int>(zones.size()))];
                    const int b = k % 3 == 0 ? a : zones[below(static_cast<int>(zones.size()))];
                    const int mask = chance(70) ? LOCOMOTORSURFACE_GROUND
                                                : (LOCOMOTORSURFACE_GROUND | LOCOMOTORSURFACE_WATER);
                    LocomotorSet set;
                    set.*access_member(LsSurfaces()) = mask;
                    row("connects " + num(layer) + " " + num(mask) + " " + num(a) + " " + num(b) +
                        " = " + num(layers[layer].connectsZones(&zm, set, a, b)));
                }
            }
        }
        state_row(verbose);
    }
    // A map of another size without a reset: everything is allocated again.
    if (index % 5 == 1 && (pf()->*access_member(PfNumWalls()) == 0 || !s.pieces.empty())) {
        if (chance(50)) s.w = 10 + below(55);
        else s.h = 10 + below(40);
        s.maxx = s.w * 10.0f;
        s.maxy = s.h * 10.0f;
        make_terrain(s.w, s.h, true);
        g_terrain->extent.hi.x = s.maxx;
        g_terrain->extent.hi.y = s.maxy;
        terrain_rows();
        row("extent " + f32(g_terrain->extent.hi.x) + " " + f32(g_terrain->extent.hi.y));
        run_newmap(s, false);
        for (int n = 0; n < 6; ++n) one_op(s, false);
        pf()->processPathfindQueue();
        row("zones " + num(s.frame));
        state_row(false);
    }
    end_scene(s);
}

int main(int argc, char** argv) {
    if (argc < 2) return 2;
    oracle_wp51::World world;
    g_world = &world;
    make_templates();
    world.load_ini(g_ini);
    g_terrain = new S2Terrain;
    TheTerrainLogic = g_terrain;
    make_terrain(10, 10, false);

    row("scenario -1");
    geometry_rows(220);
    const size_t geometry = g_rows.size();
    for (int index = 0; index < 72; ++index) run_scene(index);

    std::FILE* out = std::fopen(argv[1], "wb");
    if (!out) return 1;
    std::fputs("{\n \"rows\": [", out);
    for (size_t r = 0; r < g_rows.size(); ++r) {
        std::fprintf(out, "%s\n  \"%s\"", r ? "," : "", g_rows[r].c_str());
    }
    std::fputs("\n ]\n}\n", out);
    std::fclose(out);
    std::fprintf(stderr, "%zu rows (%zu geometry)\n", g_rows.size(), geometry);
    return 0;
}
